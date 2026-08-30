#!/usr/bin/env perl
# check_links.pl — offline Markdown link and anchor checker.
# Anchors follow GitHub slug rules (github-slugger): lowercase, strip everything
# that is not a Unicode letter/number/underscore/hyphen/space, spaces become '-',
# duplicate headings get '-1', '-2', ... suffixes. Explicit <a id="..."> and
# <a name="..."> anchors are honored. External URLs (any scheme) are not fetched:
# the gate is offline and deterministic. See docs/research/docs-lint.md.
use strict;
use warnings;
use utf8;
use feature 'unicode_strings';
use open ':std', ':encoding(UTF-8)';
use File::Find;

my ($root, $allowlist) = @ARGV;
die "usage: check_links.pl <repo_tree_root> [allowlist]\n" unless defined $root;
unless (-e "$root/README.md") {
    if (-x "build/docs/materialize_repo_tree.sh") {
        print STDERR "note: $root missing; auto-running build/docs/materialize_repo_tree.sh\n";
        system("build/docs/materialize_repo_tree.sh") == 0
            or die "materialize_repo_tree.sh failed\n";
    }
}
unless (-d $root && -e "$root/README.md") {
    die "repo tree not found at $root — run src/build/docs/materialize_repo_tree.sh first\n";
}

my %allow;
if (defined $allowlist && -f $allowlist) {
    open my $fh, '<:encoding(UTF-8)', $allowlist or die "open $allowlist: $!";
    while (my $line = <$fh>) {
        chomp $line;
        $line =~ s/\s+$//;
        next if $line =~ /^\s*(#|$)/;
        my ($path, $substr) = split /\t/, $line, 2;
        push @{ $allow{$path} }, defined $substr ? $substr : '';
    }
    close $fh;
}

my @files;
find({ no_chdir => 1, wanted => sub { push @files, $File::Find::name if -f $_ && /\.md$/ } }, $root);
# .memo/** is excluded: those are point-in-time process records whose links may
# legitimately rot with their baseline (.memo/README: log/ files freeze, review/
# files are not edited). The gate covers product docs and root files.
@files = sort grep { !m{^\Q$root\E/\.memo/} } @files;

sub rel {
    my ($p) = @_;
    $p =~ s/^\Q$root\E\///;
    return $p;
}

sub slugify {
    my ($t) = @_;
    $t = lc $t;
    $t =~ s/[^\p{L}\p{N}_\- ]//g;
    $t =~ s/ /-/g;    # github-slugger: each literal space becomes one '-'
    return $t;
}

my %anchor_cache;
my %src_manifest;
my $src_manifest_loaded = 0;
sub src_tracked {
    my ($abs) = @_;
    # $abs is "$root/src/..."; the src tree itself is not copied, only its
    # tracked-file manifest at $root/src.manifest.
    unless ($src_manifest_loaded) {
        if (open my $mf, '<', "$root/src.manifest") {
            while (my $l = <$mf>) { chomp $l; $src_manifest{$l} = 1; }
            close $mf;
        }
        $src_manifest_loaded = 1;
    }
    my ($rel) = $abs =~ m{^\Q$root\E/(.+)$};
    return defined $rel && exists $src_manifest{$rel};
}

sub anchors_of {
    my ($abs) = @_;
    return $anchor_cache{$abs} if $anchor_cache{$abs};
    my %anchors;
    my %seen;
    if (open my $fh, '<:encoding(UTF-8)', $abs) {
        while (my $line = <$fh>) {
            if ($line =~ /^#{1,6}[ \t]+(.+?)[ \t]*$/) {
                my $text = $1;
                $text =~ s/\s+#+$//;    # closed ATX heading
                my $s = slugify($text);
                if ($s ne '') {
                    if (exists $seen{$s}) {
                        $seen{$s}++;
                        $anchors{"$s-$seen{$s}"} = 1;
                    } else {
                        $seen{$s} = 0;
                        $anchors{$s} = 1;
                    }
                }
            }
            while ($line =~ /<a\s+(?:id|name)="([^"]+)"/g) {
                $anchors{$1} = 1;
            }
        }
        close $fh;
    }
    $anchor_cache{$abs} = \%anchors;
    return \%anchors;
}

sub resolve_rel {
    my ($base_dir, $rel) = @_;
    my @out;
    for my $part (split m{/}, "$base_dir/$rel") {
        next if $part eq '' || $part eq '.';
        if ($part eq '..') { pop @out; next; }
        push @out, $part;
    }
    return join('/', @out);
}

my $fail = 0;
for my $file (@files) {
    my $rel = rel($file);
    open my $fh, '<:encoding(UTF-8)', $file or do { warn "read $file: $!"; next };
    my $lineno = 0;
    while (my $line = <$fh>) {
        $lineno++;
        while ($line =~ /!?\[[^\]\n]*\]\(\s*([^\s)]+?)(?:\s+"[^"]*")?\s*\)/g) {
            my $target = $1;
            next if $target =~ /^[A-Za-z][A-Za-z0-9+.-]*:/;    # http:, mailto:, ...
            next if $target =~ m{^//};
            my $exempt = 0;
            for my $s (@{ $allow{$rel} // [] }) {
                if ($s eq '' || index($target, $s) >= 0) { $exempt = 1; last }
            }
            next if $exempt;
            my ($path, $anchor) = split /#/, $target, 2;
            my $abs;
            if ($path eq '') {
                $abs = $file;
            } else {
                my $base = $rel;
                $base =~ s{[^/]+$}{};    # dir of the linking file, '' for root files
                my $resolved = resolve_rel($base, $path);
                $abs = "$root/$resolved";
                if ($resolved =~ m{^src/}) {
                    unless (src_tracked($abs)) {
                        print "$rel:$lineno: broken link: $target (not a tracked file: $resolved)\n";
                        $fail = 1;
                    }
                    next;    # no anchor checking into the code tree
                }
                unless (-e $abs) {
                    print "$rel:$lineno: broken link: $target (resolves to $resolved, not found)\n";
                    $fail = 1;
                    next;
                }
            }
            next if -d $abs;
            next unless defined $anchor && length $anchor;
            next unless $abs =~ /\.md$/;    # anchors are only meaningful in markdown
            my $anchors = anchors_of($abs);
            unless (exists $anchors->{$anchor}) {
                print "$rel:$lineno: broken anchor: $target (no such anchor in $path)\n";
                $fail = 1;
            }
        }
    }
    close $fh;
}

if ($fail) {
    print "check_links: FAILED\n";
} else {
    print "check_links: OK (" . scalar(@files) . " markdown files)\n";
}
exit($fail ? 1 : 0);

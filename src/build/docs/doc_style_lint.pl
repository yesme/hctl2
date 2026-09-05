#!/usr/bin/env perl
# Repository-specific Markdown terminology checks.  The policy comes from
# WRITING-GUIDE.md and docs/design/spec/README.md; this program only turns the
# already-approved routing rules into deterministic CI checks.
use strict;
use warnings;
use utf8;
use feature 'unicode_strings';
use open ':std', ':encoding(UTF-8)';
use File::Find;

my ($mode, $root, $allowlist, $terms_file) = @ARGV;
die "usage: doc_style_lint.pl <layer-terms|implementation-names|camelcase|first-use|spec-need|table-language-mix> <repo-tree> [allowlist] [terms-file]\n"
    unless defined $mode && defined $root;
die "repo tree not found at $root\n" unless -d $root && -f "$root/README.md";

my @architecture_files = qw(
    docs/design/README.md
    docs/design/architecture.md
    docs/design/context.md
    docs/design/participant.md
    docs/design/project.md
    docs/design/repo.md
    docs/design/run.md
    docs/design/task.md
);
my %architecture_file = map { $_ => 1 } @architecture_files;

my %allow;
if (defined $allowlist && -f $allowlist) {
    open my $fh, '<:encoding(UTF-8)', $allowlist or die "open $allowlist: $!";
    while (my $line = <$fh>) {
        chomp $line;
        $line =~ s/\r$//;
        next if $line =~ /^\s*(?:#|$)/;
        my ($path, $term, $contains) = split /\t/, $line, 3;
        die "$allowlist: malformed line: $line\n" unless defined $path && defined $term;
        push @{ $allow{$path}{$term} }, defined $contains ? $contains : '';
    }
    close $fh;
}

sub allowed {
    my ($path, $term, $raw) = @_;
    for my $contains (@{ $allow{$path}{$term} // [] }) {
        return 1 if $contains eq '' || index($raw, $contains) >= 0;
    }
    return 0;
}

sub all_markdown_files {
    my @files;
    find({
        no_chdir => 1,
        wanted => sub {
            return unless -f $_ && /\.md$/;
            my $rel = $File::Find::name;
            $rel =~ s{^\Q$root\E/}{};
            push @files, $rel;
        },
    }, $root);
    return sort @files;
}

sub active_product_file {
    my ($path) = @_;
    return 1 if $path eq 'README.md' || $path eq 'WRITING-GUIDE.md' || $path eq 'docs/usage.md';
    return 0 unless $path =~ m{^docs/design/};
    return 0 if $path eq 'docs/design/references/decision-history.md';
    return 0 if $path eq 'docs/design/references/glossary.md';
    return 1;
}

sub design_layer_file {
    my ($path) = @_;
    return 1 if $path eq 'docs/design/vision.md' || $architecture_file{$path};
    return 0;
}

sub read_clean_lines {
    my ($path) = @_;
    open my $fh, '<:encoding(UTF-8)', "$root/$path" or die "open $path: $!";
    my @out;
    my ($fence, $comment) = (0, 0);
    my $number = 0;
    while (my $raw = <$fh>) {
        $number++;
        chomp $raw;
        $raw =~ s/\r$//;
        if ($raw =~ /^\s{0,3}(?:```|~~~)/) {
            $fence = !$fence;
            push @out, [$number, $raw, ''];
            next;
        }
        if ($fence) {
            push @out, [$number, $raw, ''];
            next;
        }
        my $text = $raw;
        if ($comment) {
            if ($text =~ s/^.*?-->//) {
                $comment = 0;
            } else {
                push @out, [$number, $raw, ''];
                next;
            }
        }
        while ($text =~ /<!--/) {
            if ($text =~ s/<!--.*?-->//) {
                next;
            }
            $text =~ s/<!--.*$//;
            $comment = 1;
            last;
        }
        # Link labels are prose; destinations and inline code are not.
        $text =~ s{(!?\[[^\]]*\])\([^)]*\)}{$1}g;
        $text =~ s/`+[^`]*`+//g;
        $text =~ s/<[^>]+>//g;
        $text =~ s/[\[\]*_~]//g;
        push @out, [$number, $raw, $text];
    }
    close $fh;
    return @out;
}

sub term_re {
    my ($term) = @_;
    return qr/(?<![A-Za-z0-9_])\Q$term\E(?![A-Za-z0-9_])/;
}

sub load_vocabulary {
    my $path = "$root/docs/design/spec/README.md";
    open my $fh, '<:encoding(UTF-8)', $path or die "open $path: $!";
    my (%core, %high, %indexed);
    my $section = '';
    while (my $line = <$fh>) {
        if ($line =~ /^##\s+(.+)/) {
            $section = $1;
            next;
        }
        if ($section eq '核心产品词' && $line =~ /^\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|/) {
            my ($term, $zh, $category, $visibility) = ($1, $2, $3, $4);
            next if $term eq '词' || $term =~ /^-+$/;
            for ($term, $zh, $category, $visibility) { s/^\s+|\s+$//g; s/`//g; }
            $core{$term} = { zh => $zh, category => $category, visibility => $visibility };
        }
        if ($section eq '核心产品词' && $line =~ /另有八个高频约束词.*?：(.+)/) {
            my $tail = $1;
            while ($tail =~ /([A-Z][A-Za-z]*(?:\s+[A-Z][A-Za-z]*)*)（([^）]+)）/g) {
                $high{$1} = $2;
            }
        }
        if ($section eq '词汇索引' && $line =~ /^-\s+\*\*[^*]+\*\*.*?：(.+)/) {
            my $members = $1;
            # Explanatory prose follows a semicolon.  Before it, index members
            # are enumeration-separated and each English name leads its item.
            $members =~ s/；.*$//;
            for my $item (split /、/, $members) {
                $item =~ s/^\s+|\s+$//g;
                if ($item =~ /^([A-Z][A-Za-z]*(?:[–-][A-Z][A-Za-z]*)?(?:\s+[A-Z][A-Za-z]*(?:[–-][A-Z][A-Za-z]*)?)*)/) {
                    $indexed{$1} = 1;
                }
            }
        }
    }
    close $fh;
    die "failed to parse core product terms from $path\n" unless keys(%core) >= 20;
    die "failed to parse eight high-frequency terms from $path\n" unless keys(%high) == 8;
    die "failed to parse constraint vocabulary index from $path\n" unless keys(%indexed) >= 20;
    return (\%core, \%high, \%indexed);
}

sub report_hit {
    my ($path, $line, $term, $why, $raw) = @_;
    if (allowed($path, $term, $raw)) {
        print "ALLOWED $path:$line: $term\n";
        return 0;
    }
    print "FAIL $path:$line: $why '$term'\n";
    print "SUGGEST\t$path\t$term\t$raw\n" if $ENV{HCTL_DOC_LINT_SUGGEST};
    return 1;
}

sub check_terms_in_file {
    my ($path, $terms, $why) = @_;
    my $fail = 0;
    for my $line (read_clean_lines($path)) {
        my ($number, $raw, $text) = @$line;
        next if $text eq '';
        for my $term (@$terms) {
            my $re = term_re($term);
            next unless $text =~ /$re/;
            $fail = report_hit($path, $number, $term, $why, $raw) || $fail;
        }
    }
    return $fail;
}

sub run_layer_terms {
    my ($core, $high, $indexed) = load_vocabulary();
    my @vision_forbidden = sort grep { $core->{$_}{visibility} =~ /治理内部/ } keys %$core;
    my @constraint_only = sort grep { !exists $core->{$_} && !exists $high->{$_} } keys %$indexed;
    my $fail = check_terms_in_file('docs/design/vision.md', \@vision_forbidden, 'vision contains governance-internal term');
    for my $path (@architecture_files) {
        $fail = check_terms_in_file($path, \@constraint_only, 'architecture contains constraint-only term') || $fail;
    }
    print $fail ? "check_layer_terms: FAILED\n" : "check_layer_terms: OK\n";
    return $fail;
}

sub run_implementation_names {
    die "implementation-names mode requires a terms file\n" unless defined $terms_file && -f $terms_file;
    open my $fh, '<:encoding(UTF-8)', $terms_file or die "open $terms_file: $!";
    my @terms;
    while (my $line = <$fh>) {
        chomp $line;
        $line =~ s/\r$//;
        next if $line =~ /^\s*(?:#|$)/;
        push @terms, $line;
    }
    close $fh;
    my @scope = ('README.md', 'docs/design/vision.md', @architecture_files);
    my $fail = 0;
    $fail = check_terms_in_file($_, \@terms, 'design layer contains implementation product name') || $fail for @scope;
    print $fail ? "check_implementation_names: FAILED\n" : "check_implementation_names: OK\n";
    return $fail;
}

sub run_camelcase {
    my $fail = 0;
    for my $path (grep { active_product_file($_) } all_markdown_files()) {
        for my $line (read_clean_lines($path)) {
            my ($number, $raw, $text) = @$line;
            while ($text =~ /(?<![A-Za-z0-9_])([A-Z][a-z0-9]+(?:[A-Z][A-Za-z0-9]*)+)(?![A-Za-z0-9_])/g) {
                my $term = $1;
                next if $term eq 'ChangeSet' || $term eq 'ReviewSubjectRef';
                $fail = report_hit($path, $number, $term, 'unallowlisted CamelCase name', $raw) || $fail;
            }
        }
    }
    print $fail ? "check_camelcase_names: FAILED\n" : "check_camelcase_names: OK\n";
    return $fail;
}

sub longer_term_overlaps {
    my ($text, $term, $position, $all_terms) = @_;
    for my $longer (@$all_terms) {
        next unless length($longer) > length($term);
        my $re = term_re($longer);
        while ($text =~ /$re/g) {
            my ($start, $end) = ($-[0], $+[0]);
            return 1 if $start <= $position && $end >= $position + length($term);
        }
    }
    return 0;
}

sub has_translation {
    my ($text, $term, $zh) = @_;
    return 1 if $text =~ /\Q$term\E\s*[（(]\s*[^）)]*\Q$zh\E/;
    return 1 if $text =~ /\Q$zh\E\s*[（(]\s*[^）)]*\Q$term\E/;
    return 0;
}

sub run_first_use {
    my ($core, $high) = load_vocabulary();
    # doc-discipline.md distinguishes natural Chinese equivalents, which must
    # be repeated at first use, from terms whose explanation lives only in the
    # glossary.  The latter must not acquire noisy per-file parentheticals.
    my @natural = qw(Repo Change Kanban Participant Context Workflow Verdict Receipt Skill Artifact Memo);
    my %terms = ((map { $_ => $core->{$_}{zh} } @natural), %$high);
    my @all_terms = sort { length($b) <=> length($a) || $a cmp $b } keys %terms;
    my $fail = 0;
    for my $path ('docs/design/vision.md', @architecture_files) {
        my %seen;
        LINE: for my $line (read_clean_lines($path)) {
            my ($number, $raw, $text) = @$line;
            next if $text eq '';
            for my $term (@all_terms) {
                next if $seen{$term};
                my $re = term_re($term);
                while ($text =~ /$re/g) {
                    next if longer_term_overlaps($text, $term, $-[0], \@all_terms);
                    $seen{$term} = 1;
                    next if has_translation($text, $term, $terms{$term});
                    $fail = report_hit($path, $number, $term, 'first use lacks Chinese counterpart', $raw) || $fail;
                    last;
                }
            }
        }
    }
    print $fail ? "check_first_use_terms: FAILED\n" : "check_first_use_terms: OK\n";
    return $fail;
}

sub run_spec_need {
    my $fail = 0;
    for my $path (sort grep { m{^docs/design/spec/[^/]+\.md$} } all_markdown_files()) {
        for my $line (read_clean_lines($path)) {
            my ($number, $raw, $text) = @$line;
            my $offset = 0;
            while ((my $position = index($text, '需要', $offset)) >= 0) {
                $offset = $position + 2;
                next if substr($text, $position, 4) eq '需要关注';
                $fail = report_hit($path, $number, '需要', "constraint prose uses ambiguous word", $raw) || $fail;
            }
        }
    }
    print $fail ? "check_spec_need: FAILED\n" : "check_spec_need: OK\n";
    return $fail;
}

sub run_table_language_mix {
    my ($core, $high) = load_vocabulary();
    # Keep this deliberately narrow: only report one concept written in both
    # languages inside the same Markdown table, excluding explicit glosses.
    my @natural = qw(Repo Change Kanban Participant Context Workflow Verdict Receipt Skill Artifact Memo);
    my %terms = ((map { $_ => $core->{$_}{zh} } @natural), %$high);
    my $findings = 0;
    for my $path ('docs/design/vision.md', @architecture_files) {
        my @lines = read_clean_lines($path);
        my @table;
        my $flush = sub {
            return unless @table;
            if (@table < 2) {
                @table = ();
                return;
            }
            my $start = $table[0][0];
            my $text = join "\n", map { $_->[2] } @table;
            for my $term (sort keys %terms) {
                my $zh = $terms{$term};
                my $plain = $text;
                $plain =~ s/\Q$term\E\s*[（(][^）)]*\Q$zh\E[^）)]*[）)]//g;
                $plain =~ s/\Q$zh\E\s*[（(][^）)]*\Q$term\E[^）)]*[）)]//g;
                my $re = term_re($term);
                next unless $plain =~ /$re/ && $plain =~ /\Q$zh\E/;
                next if allowed($path, $term, $text);
                print "REPORT $path:$start: table mixes '$term' and '$zh'\n";
                $findings++;
            }
            @table = ();
        };
        for my $line (@lines) {
            if ($line->[2] =~ /^\s*\|/) {
                push @table, $line;
            } else {
                $flush->();
            }
        }
        $flush->();
    }
    print "report_table_language_mix: $findings finding(s); report only\n";
    return 0;
}

my %runner = (
    'layer-terms'          => \&run_layer_terms,
    'implementation-names'=> \&run_implementation_names,
    'camelcase'            => \&run_camelcase,
    'first-use'            => \&run_first_use,
    'spec-need'            => \&run_spec_need,
    'table-language-mix'   => \&run_table_language_mix,
);
die "unknown mode '$mode'\n" unless $runner{$mode};
exit($runner{$mode}->() ? 1 : 0);

#!/usr/bin/env perl
# inventory_concepts.pl <doc tree> — mechanical concept inventory for the four-axis review.
# Concept universe: every glossary table whose second column is 中文对照 (pairs of
# English name / Chinese name). Counts each form per file and per layer, checks
# whether the first English occurrence in a file carries its Chinese gloss, lists
# CamelCase tokens, and lists concepts written both ways inside one layer.
# Extraction and counting only; every judgement is left to the reviewer.
use strict; use warnings; use utf8; use open ':std', ':encoding(UTF-8)'; use File::Find; use Cwd qw(realpath);
my $root = shift or die "usage: inventory_concepts.pl <doc tree>\n";
$root = realpath($root) // $root;   # the Buck genrule hands us a symlink; File::Find will not descend into it
my $gloss = "$root/docs/design/references/glossary.md";
open my $gh, '<', $gloss or die "open $gloss: $!";
my (@concepts, %seen);
my $in = 0;
while (my $l = <$gh>) {
    chomp $l;
    if ($l =~ /^\|\s*[^|]+\|\s*中文对照/) { $in = 1; next; }
    if ($l !~ /^\|/) { $in = 0; next; }
    next unless $in; next if $l =~ /^\|\s*-+/;
    my @c = map { s/^\s+|\s+$//gr } split /\|/, $l; shift @c;
    my ($en, $zh) = ($c[0] // '', $c[1] // '');
    $en =~ s/`//g; $zh =~ s/[；（(].*$//; $zh =~ s/`//g;
    next if $en eq '' || $en =~ /^-+$/;
    $en = '' unless $en =~ /[A-Za-z]/;
    $zh = '' unless $zh =~ /\p{Han}/;
    next if $en eq '' && $zh eq '';
    my $key = $en ne '' ? $en : $zh;
    next if $seen{$key}++;
    push @concepts, { en => $en, zh => $zh };
}
close $gh;
sub layer_of {
    my $f = shift;
    return 'vision'    if $f =~ m{^(README\.md|docs/design/vision\.md)$};
    return 'arch'      if $f =~ m{^docs/design/(architecture|README|project|task|run|participant|context)\.md$};
    return 'spec'      if $f =~ m{^docs/design/spec/};
    return 'delivery'  if $f =~ m{^docs/(design/(delivery|contract-tests|doc-discipline)\.md|usage\.md)$};
    return 'reference' if $f =~ m{^docs/design/references/};
    return 'guide'     if $f =~ m{^WRITING-GUIDE\.md$};
    return '';
}
my @files;
find(sub { return unless -f && /\.md$/; my $rel = $File::Find::name; $rel =~ s{^\Q$root\E/}{}; push @files, $rel if layer_of($rel) }, $root);
@files = sort @files;
my (%cnt, %first_gloss, %camel, %layer_mix);
for my $f (@files) {
    my $layer = layer_of($f);
    open my $fh, '<', "$root/$f" or die; local $/; my $t = <$fh>; close $fh;
    my $body = $t; $body =~ s/```.*?```//gs;                    # drop code blocks
    for my $tok ($body =~ /\b([A-Z][a-z]+(?:[A-Z][a-z]+)+)\b/g) { $camel{$tok}{$f}++ }
    my @zh_all = sort { length($b) <=> length($a) } grep { $_ ne '' } map { $_->{zh} } @concepts;
    for my $c (@concepts) {
        my ($en, $zh) = ($c->{en}, $c->{zh}); my $key = $en ne '' ? $en : $zh;
        my $ne = 0; my $nz = 0; my $stripped = $body;
        if ($en ne '' && $zh ne '') { $stripped =~ s/\Q$en\E\s*[（(]\s*\Q$zh\E\s*[)）]//g; }
        if ($zh ne '') { for my $longer (@zh_all) { last if length($longer) <= length($zh); next unless index($longer, $zh) >= 0; $stripped =~ s/\Q$longer\E/\x{1}/g; } }
        if ($en ne '') { $ne = () = $body =~ /(?<![A-Za-z0-9_])\Q$en\E(?![A-Za-z0-9_])/g; }
        if ($zh ne '') { $nz = () = $stripped =~ /\Q$zh\E/g; }
        next unless $ne || $nz;
        $cnt{$key}{$f} = [$ne, $nz];
        $layer_mix{$key}{$layer}[0] += $ne; $layer_mix{$key}{$layer}[1] += $nz;
        if ($en ne '' && $zh ne '' && $ne) {
            my $pos = index($body, $en);
            my $after = substr($body, $pos, length($en) + 40);
            $first_gloss{$f}{$en} = ($after =~ /^\Q$en\E\s*[（(][^）)]*\Q$zh\E/) ? 'yes' : 'no';
        }
    }
}
print "# 概念清点（脚本产出，不含判断）\n\n";
print "概念宇宙：词汇表中带「中文对照」列的全部表格，共 ", scalar(@concepts), " 条。层按路径划分：vision / arch / spec / delivery / reference / guide。计数规则：英文名按词边界匹配，中文名按子串匹配，`英文（中文）` 形式的对照不计入中文次数。\n\n";
print "## 一、概念 × 层：英文次数 / 中文次数\n\n| 概念（英） | 中文 | vision | arch | spec | delivery | reference | guide |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n";
for my $c (@concepts) {
    my $key = $c->{en} ne '' ? $c->{en} : $c->{zh}; next unless $layer_mix{$key};
    my @cells = map { my $v = $layer_mix{$key}{$_} // [0,0]; ($v->[0]||$v->[1]) ? "$v->[0] / $v->[1]" : '—' } qw(vision arch spec delivery reference guide);
    printf "| %s | %s | %s |\n", ($c->{en} ne '' ? "`$c->{en}`" : '—'), ($c->{zh} ne '' ? $c->{zh} : '—'), join(' | ', @cells);
}
print "\n## 二、同一层里中英两写的概念（两种写法都出现，且不是对照形式）\n\n| 概念 | 层 | 英 / 中 | 涉及文件 |\n| --- | --- | --- | --- |\n";
for my $key (sort keys %layer_mix) {
    for my $layer (qw(vision arch spec delivery reference guide)) {
        my $v = $layer_mix{$key}{$layer} or next; next unless $v->[0] && $v->[1];
        my @fs = sort grep { layer_of($_) eq $layer && ($cnt{$key}{$_}[0] && $cnt{$key}{$_}[1]) } keys %{$cnt{$key}};
        printf "| `%s` | %s | %d / %d | %s |\n", $key, $layer, $v->[0], $v->[1], (@fs ? join('、', @fs) : '（分散在不同文件）');
    }
}
print "\n## 三、英文名首现处未带中文对照（按文件）\n\n| 文件 | 概念 |\n| --- | --- |\n";
for my $f (@files) { my @miss = sort grep { $first_gloss{$f}{$_} eq 'no' } keys %{$first_gloss{$f} || {}}; printf "| %s | %s |\n", $f, join('、', map { "`$_`" } @miss) if @miss; }
print "\n## 四、驼峰拼接词（正文，代码块已排除）\n\n| 词 | 总次数 | 文件 |\n| --- | --- | --- |\n";
for my $tok (sort { (sum(values %{$camel{$b}})) <=> (sum(values %{$camel{$a}})) || $a cmp $b } keys %camel) {
    my $n = sum(values %{$camel{$tok}}); printf "| `%s` | %d | %s |\n", $tok, $n, join('、', map { "$_ ($camel{$tok}{$_})" } sort keys %{$camel{$tok}});
}
sub sum { my $s = 0; $s += $_ for @_; $s }

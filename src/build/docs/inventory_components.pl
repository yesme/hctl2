#!/usr/bin/env perl
# inventory_components.pl <doc tree> — component skeleton from the research index
# (docs/research/README.md 条目索引): object, category, current reuse decision;
# the review fills 业界最佳 / 借用等级 / 取舍.
use strict; use warnings; use utf8; use open ':std', ':encoding(UTF-8)';
my $root = shift or die "usage: inventory_components.pl <doc tree>\n";
my $f = "$root/docs/research/README.md";
open my $fh, '<', $f or die "open $f: $!"; my @lines = <$fh>; close $fh;
my ($i) = grep { $lines[$_] =~ /^\|\s*文件\s*\|\s*对象/ } 0..$#lines;
die "index table not found\n" unless defined $i;
print "# 部件清点（脚本产出，不含判断）\n\n来源：`docs/research/README.md` §条目索引。后三列空着，由部件矩阵调研填。\n\n| 对象 | 类别 | 现有复用决策 | 业界最佳 | 借用等级 | 取舍 |\n| --- | --- | --- | --- | --- | --- |\n";
for my $k ($i + 2 .. $#lines) {
    last unless $lines[$k] =~ /^\|/;
    my @c = map { s/^\s+|\s+$//gr } split /\|/, $lines[$k]; shift @c;
    my ($obj, $cat, $dec) = ($c[1] // '—', $c[3] // '—', $c[4] // '—');
    print "| $obj | $cat | $dec | — | — | — |\n";
}

#!/usr/bin/env perl
# inventory_edges.pl <doc tree> — dump the cross-module handoff table of
# spec/connections.md normalised to the four review cells; empty cells print 「—」.
use strict; use warnings; use utf8; use open ':std', ':encoding(UTF-8)';
my $root = shift or die "usage: inventory_edges.pl <doc tree>\n";
my $f = "$root/docs/design/spec/connections.md";
open my $fh, '<', $f or die "open $f: $!"; my @lines = <$fh>; close $fh;
my ($i) = grep { $lines[$_] =~ /^\|\s*方向\s*\|/ } 0..$#lines;
die "handoff table not found in $f\n" unless defined $i;
print "# 边界清点（脚本产出，不含判断）\n\n来源：`docs/design/spec/connections.md` §连接约束总表。四格按原表列名映射：交付物 ← 耐久输入；写入者与准入 ← 目标准入与提交；恢复 ← 恢复依据；「失败处理」原表无独立列，标 —，由通读补。\n\n";
print "| # | 边（方向） | 交付物 | 写入者与准入 | 恢复依据 | 失败处理 |\n| --- | --- | --- | --- | --- | --- |\n";
my $n = 0;
for my $k ($i + 2 .. $#lines) {
    last unless $lines[$k] =~ /^\|/;
    my @c = map { s/^\s+|\s+$//gr } split /\|/, $lines[$k]; shift @c;
    $n++;
    my @cells = map { defined $_ && $_ ne '' ? $_ : '—' } @c[0..3];
    print "| $n | ", join(' | ', @cells), " | — |\n";
}
print "\n共 $n 条边。\n";

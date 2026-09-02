#!/usr/bin/env perl
# inventory_enforcement.pl <doc tree> — every normative sentence (必须/不得/只能) in
# docs/design/spec/*.md with the enforcement-mechanism words it mentions; sentences
# naming no mechanism are the review's I1/I2 candidates. Classification is the
# reviewer's; this only extracts.
use strict; use warnings; use utf8; use open ':std', ':encoding(UTF-8)';
my $root = shift or die "usage: inventory_enforcement.pl <doc tree>\n";
my @mech = ('比较并交换', 'CAS', '摘要', 'digest', '租约', 'Lease', '代次', 'generation', '归约', 'reducer',
            '回读', 'readback', 'outbox', '幂等', 'idempot', '事务', '栅栏', 'fence', 'Receipt', '凭证',
            'Snapshot', '快照', 'revision', 'Revision', 'CT-', '契约测试', '校验', '拒绝', 'Preview', '预览');
my @files = sort glob("$root/docs/design/spec/*.md");
print "# 强制手段清点（脚本产出，不含判断）\n\n范围：`docs/design/spec/*.md` 全部含「必须／不得／只能」的句子。机制词表：", join('、', @mech), "。无机制词的句子单列在第二节，是 I1/I2 的候选集；有机制词不等于已被机制强制，需通读判断。\n\n";
my (@none, %stat);
print "## 一、按文件统计\n\n| 文件 | 规范句 | 含机制词 | 无机制词 |\n| --- | --- | --- | --- |\n";
my @rows;
for my $f (@files) {
    (my $rel = $f) =~ s{^\Q$root\E/}{};
    open my $fh, '<', $f or die; my $sec = '（文件头）'; my ($tot, $with) = (0, 0);
    while (my $l = <$fh>) {
        chomp $l; if ($l =~ /^#{1,3}\s+(.+)/) { $sec = $1; next; }
        next if $l =~ /^\s*(\||```|>)/;
        for my $s (split /(?<=[。；])/, $l) {
            next unless $s =~ /必须|不得|只能/;
            $tot++;
            my @hit = grep { index($s, $_) >= 0 } @mech;
            if (@hit) { $with++ } else { (my $short = $s) =~ s/^\s+//; $short = substr($short, 0, 90) . (length($short) > 90 ? '…' : ''); push @none, [$rel, $sec, $short]; }
        }
    }
    close $fh; push @rows, [$rel, $tot, $with, $tot - $with];
}
printf "| %s | %d | %d | %d |\n", @$_ for @rows;
print "\n## 二、无机制词的规范句（I1/I2 候选）\n\n| 文件 | 节 | 句子（截 90 字） |\n| --- | --- | --- |\n";
printf "| %s | %s | %s |\n", @$_ for @none;

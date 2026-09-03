#!/usr/bin/env perl
# Generate/check the glossary's semantic-name-to-identifier table from inline
# code identifiers used by the normative spec.  New snake_case identifiers are
# rejected until they receive an explicit human-readable meaning here.
use strict;
use warnings;
use utf8;
use feature 'unicode_strings';
use open ':std', ':encoding(UTF-8)';
use File::Find;
use File::Temp qw(tempfile);

my ($action, $root) = @ARGV;
die "usage: generate_identifier_glossary.pl <--render|--check|--update> <repo-tree>\n"
    unless defined $action && defined $root && $action =~ /^--(?:render|check|update)$/;
die "repo tree not found at $root\n" unless -d $root && -f "$root/docs/design/spec/README.md";

my %meaning = (
    accepted                       => '接受',
    account_stable_id              => '账号稳定标识符',
    active                         => '活动',
    adapter_event                  => '适配器事件',
    artifact_revision_id           => '工件版本标识符',
    artifact_version               => '工件状态版本',
    attempt_generation             => '尝试代次',
    backend_authoritative          => '后端权威',
    binding_revision               => '绑定版本',
    board_scope_stable_id          => '看板范围稳定标识符',
    bundle_digest                  => '上下文包摘要',
    changes_requested              => '要求修改',
    completion_pending             => '等待完成',
    context_manifest_id            => '根上下文清单标识符',
    contract                       => '契约内',
    control_writer_generation      => '控制面写入者代次',
    current                        => '当前',
    direct_client                  => '直接客户端',
    engine_binding_generation      => '引擎绑定代次',
    execution_mode                 => '执行模式',
    execution_principal            => '执行主体',
    external_board_item_id         => '外部看板项标识符',
    external_entity_kind           => '外部实体类型',
    gate                           => '评审关卡',
    generation                     => '代次',
    group_anchor_stable_id         => '分组锚点稳定标识符',
    group_kind                     => '分组类型',
    hctl_authoritative             => '控制面权威',
    immutable_external_entity_id   => '不可变外部实体标识符',
    implementation                 => '实现内',
    in_process                     => '进程内',
    inline                         => '内联',
    internal_reducer               => '内部归约器',
    invocation_version             => '单次调用版本',
    known                          => '已知',
    linked_readonly                => '只读关联',
    managed_single_writer          => '受管单写者',
    manifest_digest                => '根上下文清单摘要',
    mechanical                     => '机械可判',
    memo_id                        => '备忘标识符',
    narrated                       => '转述',
    native_interactive_allowed     => '允许原生交互',
    placement_scope_stable_id      => '放置范围稳定标识符',
    pointer                        => '指针',
    port_kind                      => '端口类型',
    project_id                     => '项目标识符',
    project_scope                  => '项目范围',
    project_version                => '项目版本',
    provider_event                 => '供应端事件',
    'quorum-unreachable'            => '法定票数不可达',
    recall                         => '回忆',
    rejected                       => '驳回',
    repo_id                        => '仓库标识符',
    repo_instance_id               => '仓库实例标识符',
    repo_scope                     => '仓库范围',
    repo_version                   => '仓库版本',
    request_id                     => '请求卡标识符',
    request_version                => '请求卡版本',
    result_commit_sha              => '结果提交 SHA',
    run_id                         => 'Run 标识符',
    run_ref                        => 'Run 引用',
    run_version                    => 'Run 状态版本',
    runtime_generation             => '运行时代次',
    scope_stable_id                => '范围稳定标识符',
    site_generation               => '现场代次',
    state_version                  => '状态版本',
    task_lifecycle_version         => '任务生命周期版本',
    task_source                    => '任务来源',
    toolbox_readback               => '工具箱直接回读',
    unknown                        => '未知',
    unsupported                    => '不支持',
);

# Inline code also carries paths, commands and product names.  These are not
# schema fields or enum values and therefore do not belong in this table.
my %ignored = map { $_ => 1 } qw(
    cancel control design docs fail false git-common-dir hctl2 hctl2-control
    hctl2-tool hctl2-workbench human lock provider task true
);

my @spec_files;
find({
    no_chdir => 1,
    wanted => sub {
        return unless -f $_ && /\.md$/;
        my $path = $File::Find::name;
        return unless $path =~ m{/docs/design/spec/[^/]+\.md$};
        push @spec_files, $path;
    },
}, "$root/docs/design/spec");
@spec_files = sort @spec_files;

my %found;
for my $path (@spec_files) {
    (my $rel = $path) =~ s{^\Q$root\E/}{};
    open my $fh, '<:encoding(UTF-8)', $path or die "open $path: $!";
    my $fence = 0;
    while (my $line = <$fh>) {
        if ($line =~ /^\s{0,3}(?:```|~~~)/) {
            $fence = !$fence;
            next;
        }
        next if $fence;
        while ($line =~ /`([^`]+)`/g) {
            my $span = $1;
            while ($span =~ /(?<![A-Za-z0-9_-])([a-z][a-z0-9]*(?:[_-][a-z0-9]+)*)(?![A-Za-z0-9_-])/g) {
                my $identifier = $1;
                next if $ignored{$identifier};
                die "unmapped inline-code identifier '$identifier' in $rel\n"
                    unless exists $meaning{$identifier};
                $found{$identifier}{$rel} = 1;
            }
        }
    }
    close $fh;
}
die "no identifiers found in spec\n" unless keys %found;

my $begin = '<!-- BEGIN GENERATED IDENTIFIER GLOSSARY -->';
my $end = '<!-- END GENERATED IDENTIFIER GLOSSARY -->';
my $rendered = "$begin\n## 语义名与标识符\n\n";
$rendered .= "> 本节由 `src/build/docs/generate_identifier_glossary.pl` 从约束层正文的代码体字段与枚举值生成；请修改约束层来源或生成器映射，不要手改本表。\n\n";
$rendered .= "| 语义名 | 标识符 | 约束层出处 |\n| --- | --- | --- |\n";
for my $identifier (sort { $meaning{$a} cmp $meaning{$b} || $a cmp $b } keys %found) {
    my @sources = map {
        (my $link = $_) =~ s{^docs/design/}{../};
        my ($name) = $link =~ m{([^/]+)$};
        "[$name]($link)";
    } sort keys %{ $found{$identifier} };
    $rendered .= "| $meaning{$identifier} | `$identifier` | " . join('、', @sources) . " |\n";
}
$rendered .= "$end\n";

if ($action eq '--render') {
    print $rendered;
    exit 0;
}

my $glossary = "$root/docs/design/references/glossary.md";
open my $gh, '<:encoding(UTF-8)', $glossary or die "open $glossary: $!";
local $/;
my $current = <$gh>;
close $gh;

if ($action eq '--update') {
    if ($current =~ /\Q$begin\E.*?\Q$end\E\n?/s) {
        $current =~ s/\Q$begin\E.*?\Q$end\E\n?/$rendered/s;
    } else {
        $current =~ s/\s*\z/\n\n$rendered/;
    }
    open my $out, '>:encoding(UTF-8)', $glossary or die "write $glossary: $!";
    print {$out} $current;
    close $out;
    print "updated $glossary\n";
    exit 0;
}

my ($committed) = $current =~ /(\Q$begin\E.*?\Q$end\E\n?)/s;
unless (defined $committed) {
    print "check_identifier_glossary: generated section missing from docs/design/references/glossary.md\n";
    exit 1;
}
if ($committed ne $rendered) {
    my ($expected_fh, $expected_path) = tempfile('identifier-glossary-expected.XXXXXX', TMPDIR => 1, UNLINK => 1);
    my ($actual_fh, $actual_path) = tempfile('identifier-glossary-actual.XXXXXX', TMPDIR => 1, UNLINK => 1);
    binmode $expected_fh, ':encoding(UTF-8)';
    binmode $actual_fh, ':encoding(UTF-8)';
    print {$expected_fh} $rendered;
    print {$actual_fh} $committed;
    close $expected_fh;
    close $actual_fh;
    system('diff', '-u', $actual_path, $expected_path);
    print "check_identifier_glossary: generated section is stale; run --update\n";
    exit 1;
}
print "check_identifier_glossary: OK (" . scalar(keys %found) . " identifiers)\n";
exit 0;

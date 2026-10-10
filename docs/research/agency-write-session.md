# Agency 写入会话

> 状态：Ubuntu 本机已实跑；最终头两平台 CI 待核 · 2026-10-11。只记录第 3f 包的执行机制与证据，不新增准入或发布授权。

## 选型与边界

工作副本按 main `c58a82d` 的 3f 第 2 条，直接调用 P1 的 `hctl2-tool worktree materialize`。工具使用原生 [Git worktree](https://git-scm.com/docs/git-worktree)，负责分支、基线标记和 `<root>/<ChangeSet>` 目录。Agency 只核工具回读并认回保全副本，不创建裸仓库或自定义 ref。Repo 本地路径、目标正文和发布边界来自第 4 包 Bundle；租约、Repo、基线与发布策略摘要必须与 Spec 一致。

实现层脚注：当前公开参数为 `--repo`，原生分支为 `hctl2/changeset/<id>`，对应开工书的 Repo 路径和 ChangeSet 分支。后续 detach 时按目录、Git common dir 和冻结基线认回。封存前，只有 HEAD 与原生分支都仍为冻结基线，才用 Git checkout 恢复原分支；不 reset、不移动 ref、不放宽 `archive::require_worktree`。

两家继续使用 [3d/3e 的正文通道](./harness-dispatch-channel-20261006.md)，同一 selection 的队列与原生 session ID / thread 保留。切 cwd 或工具权限时关闭旧进程再原生 resume，下一份只读 Spec 不继承写权限。只读函数与旧 `herdr.rs / codex_fixture_test.rs` 用例原文未改。

- [Claude CLI](https://code.claude.com/docs/en/cli-reference)：会话级 `--restricted / --tools / --permission-mode / --settings` 开放编辑和测试；正文仍经插件交入，不走输入框。内层 sandbox 关闭，凭据边界由 Agency 的 OS 隔离承担，不改全局配置。
- [Codex app-server](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/src/protocol/v2/permissions.rs)：写入 turn 使用 `externalSandbox`，启动过程的 OS 边界隔离控制凭据；只读 `readOnly` 参数保留。Linux 不嵌套 bubblewrap。
- Codex 0.161.0 的 [Unix listener](https://github.com/openai/codex/blob/rust-v0.161.0/codex-rs/app-server-transport/src/transport/unix_socket.rs) 实际端点移至固定 `/tmp/codex-daemon-<uid>`；不把这个共享特权目录放进写入允许路径。写会话使用原生 localhost WebSocket 与独立 capability-token，Herdr 用原生 `--remote-auth-token-env` 接入。只读 Unix listener 不变。

物化副本共享源 Git 目录，harness 不获源 Git 配置和 Git 凭据访问权；代码编辑、编译和测试使用工作文件，Git 身份与封存由 Agency / 工具回读。Linux 保留 Landlock 的精确允许路径，环境不带 GH token、DBus 会话或 SSH agent。macOS 写入 Claude 的源 Git 拒读规则在 Herdr 启动前与控制凭据、钥匙串路径合成一层 profile，按源 Git common dir 复用实例；Codex app-server 直接应用一层 profile。不改 Herdr 制品或协议。

这个 macOS 调整来自 CI `38069437964`：pane 内新增 `sandbox-exec` 后 Claude pane 在派工前消失，Codex 与只读回归通过。它与 [Anthropic 仓库的嵌套 sandbox 复现](https://github.com/anthropics/sandbox-runtime/issues/67)一致。删除内层应用、在启动前合成 profile 后必须核新头 CI；Ubuntu 不能代替 macOS。macOS 原生登录与原生 keychain 内容访问仍 UNVERIFIED，不因夹具通过提升能力声明。

## 封存接口

#405 已合入 main `f526006`。Agency 只调用公开 `repo seal --path ... --change-set-ref ... --baseline ... --key ...`，不另算索引或结果树。Spec 的规范摘要作为固定关联键，工具重试不重派正文，同一个键重试固定同一个提交。

工具回读 `base_commit_sha / base_tree_sha / result_tree_sha / result_commit_sha`。Agency 比较两棵树：相同交 `no_changes`，有改动交固定 commit，输出严格为 Repo README 的 `hctl2.changeset-output.v1`。`parent_revision_id` 未经材料交付，保持 null。结果树由固定提交回读，不能在严格 schema 中另塞一个 `result_tree_sha` 字段。Proposal 的证据等级仍为 adapter_event；工具 Git 回读另留 unmediated observation，不提升未验证的工种能力。

原生回答先写 Agency 私有 `pending-answer.json`。seal 失败保留原错误码和工作文件，回合保持 Running，无 Proposal 或 TurnReturned；仅重试工具。撤销或超时结束等待，关闭原生写入进程并附 session_closed 证据。成功交回后仍保留 stop 句柄到撤租约或空闲回收，不让 DispatchReleased 提前使控制面失去停止入口。只读停止规矩保持原样。

[接口分工确认](https://github.com/yesme/hctl2/pull/405#issuecomment-6094092852)：Bundle 本地路径、完整目标与 no_changes 接收由 mac 席在 #405 补齐。[处理说明](https://github.com/yesme/hctl2/pull/405#issuecomment-6099906403)撤回之前放宽 detached seal 的请求。本 PR 不改工具、控制面、runtime.rs、tenant.rs 或 port.rs，不接准入与发布。

## 入口与回归实测

用例从真实 Agency RPC `prepare → activate → InstalledHerdr::start_for_tenant` 进入处理函数，再核进程和 Git 效果。不是只调内部函数。

```sh
./src/buck2 test root//agency:write_session_test root//agency:codex_fixture_test root//agency:herdr_test root//agency:unit_test root//agency:runtime_test root//agency:cli_test root//agency:port_test root//agency:control_port_test
```

Build `ccf67205-8aee-44a0-8a9f-430fd6875a0f`：8 个 target 通过；写入入口 `5 passed; 0 failed; 2 ignored; 191.00s`；旧回归依次 `6 / 52 / 16 / 7 / 1 / 26 / 31 passed; 0 failed`，Herdr 7 条本机登录用例仍 ignored。

两家夹具核真实工具的 schema、outcome、基线、规范 cwd、源 Repo common dir、无私有裸仓库、源代码不变、源 Git 配置拒读、控制配对密钥拒读、Linux keyring 试件拒读、gh auth 不成功，以及环境中无 GH token / DBus。停止后 detach，同一 ChangeSet 的第二代租约和 Repo 元数据版本变化仍认回同一目录、保留文件，并恢复原生分支封存；之后只读 Spec 正常交回。

严格反序列化 Proposal 输出，固定 commit 的树等于工具回读，再以独立 seal 键回读当前工作树核同一结果树；原关联键重试返回 reused=true 和同一提交。两家各核空结果。外部诚实 Git 操作切到另一分支，seal 报 `HCTL2_TOOL_WORKTREE_BRANCH_MOVED`，Running / 零 Proposal / 无 TurnReturned；恢复后只重试工具、一次 harness 执行、一次 Proposal。另一组在失败时撤销，实际关闭并保全回答与工作文件，没有 Proposal。

拒绝入口覆盖：Bundle 基线与持有者变化、本机路径或目标缺失、无 git.write、发布目标变而摘要未变、远端机器、只交租约或只交策略、租约形状损坏。均 CannotFulfill，不起工作树。

```sh
./src/buck2 build root//agency:clippy
./src/buck2 test root//apps/tool:archive_test root//crates/repo/...
./src/buck2 test root//build/docs:profile-research
```

Clippy Build `e92cf41b-c3c9-4c13-ba0f-0d86f356c4e1` 通过，12 份报告全为空。Tool / Repo Build `38e5681b-7e16-41aa-99bc-e426a2ff5ee6`：6 个 target 全过，`78 passed; 0 failed`。文档 Build `690cfcbd-813d-41ef-a55d-9362868fed63`：`check_links: OK (163 markdown files)`。

## 退回修正看红

旧工作树阶段删除 Bundle baseline 与 Spec base 比较，Build `b76d67ec-afca-4fed-be5d-a37d8f4cc74b` 的端口边界用例红：`left: Running / right: CannotFulfill / 0 passed; 1 failed`；恢复后回归绿。退回版本未提交。

只将 Codex Handle.stop 退回“finished 后忽略 stop”，保留字段读取以免 Rust 未使用字段诊断拦在编译阶段。运行：

```sh
./src/buck2 test root//agency:write_session_test -- --test-arg=codex_write_entry_uses_materialized
```

Build `ef500666-fa43-443d-9198-091c1fb0f93d`：

```text
codex_write_entry_uses_materialized_worktree_and_cannot_read_credentials ... FAILED
write revocation lacks session-close evidence
state: ResultReturned
0 passed; 1 failed
```

Proposal 已交回而原写入进程不能被撤销入口关闭。恢复精确原文件后，Build `1843ce00-3b52-4c78-a8c5-06f44edf51d4`：`1 passed; 0 failed; 16.89s`。退回版本未提交。第一次直接删除条件的变异被未使用字段诊断挡住，NO TESTS RAN；没有将那次编译失败算成用例变红。

## 两家真实会话

CI 默认 ignored，注释为 UNVERIFIED；本机原生登录显式启用，不改 ~/.claude 或 ~/.codex 全局配置。

```sh
./src/buck2 test root//agency:write_session_test -- \
  --env HCTL2_HARNESS_LIVE=1 --env CODEX_HOME=/home/jackywang/.codex \
  --test-arg=live_ --test-arg=--include-ignored --test-arg=--nocapture
```

Build `c5791889-380d-4ea0-b1aa-f599aeebd633`：`2 passed; 0 failed; 190.95s`。Claude Code `2.1.293`、模型 `claude-opus-5-5`；Codex CLI `0.161.0`。

两家都将 calculator.py 的 `return value + 2` 改为 `return value * 2`，原生测试与独立重跑一致：

```text
test_double (test_calculator.CalculatorTest.test_double) ... ok
Ran 1 test in 0.000s
OK
```

Claude session `7cb61625-651b-4bd2-afcb-24181009b73a`、turn `13e923be-0670-4a5b-add6-987b04f007cb`，原生记录 `~/.claude/projects/<工作树编码>/<session>.jsonl`；封存 baseline `6e328ffe4d24741dfe96ce54a2ca8b2c890d54d2`、commit `8dc99d8493be62b1f9e8c72dbf2261df2e182a4a`。Codex thread `01a126d2-b33c-7f51-bc05-367041baa404`，原生记录 `~/.codex/sessions/2026/10/11/rollout-2026-10-11T01-18-26-01a126d2-b33c-7f51-bc05-367041baa404.jsonl`；封存 baseline `61bcf343a245d3a98d2ae7c374bb024f5ff22184`、commit `8c583056553a524c6092dab5ca4a3be5cbea74d1`。二者 Proposal 与独立工作树回读的 tree 都是 `d09eea455a0755473856dd9d89c249d6f68ca220`。

Claude 原生 JSONL 中逐字比较 user 正文与插件 started.text，核原生工具的测试 OK。Codex 原生 rollout 中核 user 正文、ChangeSet / 租约 / 发布目标和 CommandExecution exit_code=0 / OK，运行时另逐字核 input_text。最终状态 ResultReturned，Proposal 的结果树与独立工作树回读相等。Claude 叙述中 Git status 的失败是隔离边界内的真实现象；Agency 在边界外的 Git 回读与 seal 成功，不能把模型对此原因的猜测当成事实。

## 复核记录

### 2026-10-11 · 第 9 包第 5 条：声明了 `no_network` 并没有断网

Grok 第二席（Ubuntu）核对第 3f 写入路径。Codex 写入 turn 把 `sandboxPolicy` 设成 `externalSandbox`，`networkAccess` 为 `enabled`。macOS profile 从 `(allow default)` 起。Linux 助手固定 Landlock ABI V1，只处理文件系统访问。工种名册的 `isolation_effects` 是空的，准备阶段会因名册里没有这个字符串而给出 `CAPABILITY_MISSING`；`InstalledHerdr::start` 不经过 `fulfills`。名册若写上 `no_network`，进程仍会带着网络启动。

本包不把网络说成已经切断。更高 ABI 的 Landlock 才有 TCP bind/connect 规则，这里没有套用；macOS 的 `(deny network*)` 会碰到 Codex 写入用的本机 WebSocket 和模型 API，和声明的 `no_network` 不是同一件事。任何已声明的隔离效果，包括 `no_network` 和未知名字，在准备和启动前拒绝，原因码 `ISOLATION_UNAVAILABLE`，恢复动作 `drop_unenforceable_isolation`。不拉起 harness。派工记录清掉这些效果，避免把未施加的承诺记成已生效。没有声明时照常启动。

只读 Codex 改为走同一层操作系统凭据边界，仍用 Unix 套接字和 `read-only`，不传 `danger-full-access`，也不把 `networkAccess` 设成 `enabled`。Herdr 每次启动都写上钥匙串和 `gh` 的文件拒绝；Linux 上该函数是空操作，边界仍是允许名单。没有改 Herdr 制品或协议，没有改 `~/.codex` 或 `~/.claude`。

Ubuntu 本机实测，Build `81fb6595-d84d-4d14-a9ce-6a2e7e4537a2`：

```text
write_session_test: 5 passed; 0 failed; 2 ignored; 203.35s
codex_fixture_test: 8 passed; 0 failed; 29.0s
herdr_test: 53 passed; 0 failed; 7 ignored; 178.68s
unit_test: 17 passed; 0 failed
runtime_test: 7 passed; 0 failed
cli_test: 1 passed; 0 failed
port_test: 26 passed; 0 failed; 17.43s
control_port_test: 31 passed; 0 failed; 86.83s
Tests finished: Pass 8. Fail 0.
```

两家写入沿既有探针拒读凭据根、Linux keyring 试件和 `gh auth`。只读补了同一组：Claude 走 Herdr pane，与 Claude 进程同一 Landlock；Codex 走受限制的 app-server 夹具。pane 里的 `gh auth status` 限时等待后仍非成功。macOS 的 profile 规则和钥匙串路径在同一份代码里。本席没有在 macOS 上跑，这一半记为推断，不把 Ubuntu 夹具当成 macOS keychain 实测。

把 `reject_unenforceable_isolation` 改成始终成功后，Build `3ddf0798-c095-4d05-8df8-815bb181184c`：`unit_test` 与 `codex_fixture_test` 各 0 passed、1 failed。前者对 `Ok` 调用 `unwrap_err`，后者报告 `declared no_network must not start`。恢复原函数后 Build `86296dfe-763c-48e6-a45e-77035e1b819b` 这两条重新通过。退回版本未提交。

Clippy Build `a377b543-c261-4725-8758-2a0affd6d60f`：12 份报告全为空。文档链接 Build `4c97f2e9-6d08-4150-bc00-d39f64d1f58a`：`check_links: OK (163 markdown files)`。真实登录会话没有重跑。


# Agency 写入会话

> 状态：部分验证 · 2026-10-10。本文记录第 3f 包的选型与本机证据，不新增领域授权。

Agency 继续使用 [3d/3e 的交正文接口](./harness-dispatch-channel-20261006.md)。写入只改变本轮的工作目录与工具权限；ChangeSet、租约和发布策略由 Execution Spec 与第 4 包 Bundle 交付，封存和准入沿第 6 包接口。

## 原生机制与边界

- [Git worktree](https://git-scm.com/docs/git-worktree) 原生支持 `worktree add --detach <path> <commit>`。原仓库配置及公共 Git 目录可能位于控制面凭据根，因此先用无凭据的本地 Git 对象传输建立 Agency 私有裸仓库，再在它上面建 detached 工作树；不把原仓库的配置、remote、凭据助手或 hooks 复制给 harness。工作树不随派工临时目录删除。
- [Claude CLI](https://code.claude.com/docs/en/cli-reference) 的 `--restricted`、`--tools`、`--permission-mode` 与 `--settings` 都可按启动指定。只读启动参数保留；写入启动显式开放读、编辑和测试所需工具，用会话 settings 配置权限，不修改用户配置。切换工作目录或权限时关闭当前 pane，再按原生会话 ID 续接；不让下一份只读 Spec 继承写入权限。
- [Codex app-server 协议](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/src/protocol/v2/permissions.rs) 的 `turn/start` 可逐轮指定 `cwd` 与 `sandboxPolicy`。`workspaceWrite` 不限制所有读路径，不能单靠它证明凭据不可读。Ubuntu 上 Landlock 还会使嵌套的 bubblewrap 挂载失败；写入型 app-server 使用原生 `externalSandbox`，由 Agency 的 OS 隔离提供凭据边界，逐项测试实际拒读。只读 `readOnly` 的逐轮参数保留。登录使用本机 harness 自己的原生登录，不复制凭据。
- 封存调用 `hctl2-tool` 的公开入口；不在 Agency 重新实现索引快照、结果树计算或版本准入。#405 合入前只实现准备和执行，不能把未封存的回答报告为已交回。

Codex 0.161.0 的 [Unix listener](https://github.com/openai/codex/blob/rust-v0.161.0/codex-rs/app-server-transport/src/transport/unix_socket.rs) 会把实际 socket 移到固定的 `/tmp/codex-daemon-<uid>`，不能经 HOME/TMPDIR 覆盖；该目录同时承载其他特权会话，不将它加入写入进程的允许路径。写入会话改用原生 localhost WebSocket 与 `--ws-auth capability-token`，每次启动生成独立令牌，只把摘要交给 listener；Herdr 通过原生 `--remote-auth-token-env` 接入同一会话。只读 Unix listener 不变。协议帧处理沿现有 3e 适配器，未引入新的协议或依赖。

## 本机事实

2026-10-10 Ubuntu：Claude Code `2.1.293`、模型 `claude-opus-5-5`；Codex CLI `0.161.0`。两家本机原生登录会话均已实跑代码修改和测试。以下记录仍不算第 3f 包六条全部验收通过：#405 尚未合入，Proposal 封存及其结果树一致性还未验证。

## 入口与回归记录

用例都从实际 Agency 的本机 RPC `prepare → activate → InstalledHerdr::start_for_tenant` 进入 Claude/Codex pool，再核进程效果。没有只调用 `write::prepare` 来代替端口派工。

```sh
./src/buck2 test root//agency:write_session_test root//agency:codex_fixture_test root//agency:unit_test root//agency:herdr_test root//agency:runtime_test
```

2026-10-10，Build ID `6a5344d3-d9b0-4d02-861d-0b0d65a8fde2`：

```text
write_session_test: 3 passed; 0 failed; 2 ignored
codex_fixture_test: 6 passed; 0 failed
unit_test: 16 passed; 0 failed
herdr_test: 52 passed; 0 failed; 7 ignored
runtime_test: 7 passed; 0 failed
Tests finished: Pass 5. Fail 0.
```

两家写入夹具均实际改 `calculator.py` 并执行 Python 测试，核 cwd、HEAD、detached、源代码未变、无 remote、控制面配对密钥拒读、Linux 允许路径之外的 keyring 试件拒读、`gh auth status` 不成功、GitHub token 与 DBus 会话环境不存在。撤销后文件保全；第二代租约仍使用同一个 ChangeSet 目录；之后只读 Spec 能交回只读回答。macOS 上本机 keychain 登录行为未实测；不把 Linux 的文件试件当作 macOS 原生钥匙串实测。

边界入口另核：Bundle 基线变、持有者变、本机 Repo 路径缺失、目标正文缺失、没有 `git.write`、发布目标变而冻结摘要未变、Repo 在另一台机器、只交租约或只交策略、租约形状损坏。它们均走 `CannotFulfill` 和对应拒绝码，不起工作树。

```sh
./src/buck2 test root//agency:cli_test root//agency:port_test root//agency:control_port_test
```

Build ID `e5f928d5-fadd-412b-ba30-da22e42c240b`：`1 / 22 / 27 passed; 0 failed`，三个 target 全过。

```sh
./src/buck2 build root//agency:clippy
```

Build ID `8082cf68-8688-453d-baba-5b5680af728a`：`BUILD SUCCEEDED`，Agency 12 份 `clippy.txt` 全为空；不是只据 build 的退出码说 Clippy 无诊断。

## 退回修正

只删除 `write::prepare` 中 `set["baseline_commit"] != *base` 的比较，其余未改，重跑：

```sh
./src/buck2 test root//agency:write_session_test -- --test-arg=write_entry_rejects
```

Build ID `097632c1-b71a-4237-bead-36d2cdea7ed5` 的正常比较版本通过。退回后的 Build ID `b76d67ec-afca-4fed-be5d-a37d8f4cc74b`：

```text
write_entry_rejects_mismatched_or_missing_bundle_authority ... FAILED
assertion `left == right` failed
left: Running
right: CannotFulfill
0 passed; 1 failed
```

错误基线进入了 harness，而不是按合同拒绝。恢复比较后，上面的全组回归通过。退回版本未提交。

## 两家真实会话

CI 默认不取本机登录，真实会话用例 `ignored`，注释标 `UNVERIFIED`；以下是 Ubuntu 本机显式启用的实录。

```sh
./src/buck2 test root//agency:write_session_test -- \
  --env HCTL2_HARNESS_LIVE=1 --env CODEX_HOME=/home/jackywang/.codex \
  --test-arg=live_ --test-arg=--include-ignored --test-arg=--nocapture
```

Build ID `efeb9f75-ef51-47a2-924c-bc9a15208d13`：

```text
live_claude_changes_non_documentation_code_and_runs_tests ... ok
live_codex_changes_non_documentation_code_and_runs_tests ... ok
2 passed; 0 failed; 118.38s
```

两家收到的目标是修正 `calculator.py` 的 `double(value)`，从 `value + 2` 改为 `value * 2`，运行 `/usr/bin/python3 -m unittest -v test_calculator`，保留未提交改动。两家的原生记录中都核到 Bundle 的 ChangeSet、基线、租约和完整发布目标；不是凭终端截图或模型自述认定正文送到。

Claude 原生 JSONL：session `dabff094-529c-466a-b91d-8481b49d43c2`、turn `faf0583f-271f-4b21-a623-b8b36ad6d171`，文件在 `~/.claude/projects/<工作树编码>/<session>.jsonl`。用例逐字比较原生 `user.message.content` 与会话插件的派工正文，核到原生 `Edit` 和 `Bash` 结果。Codex 原生 rollout：thread `01a12464-471f-7182-81ea-09223c462d77`，文件 `~/.codex/sessions/2026/10/10/rollout-2026-10-10T13-58-35-01a12464-471f-7182-81ea-09223c462d77.jsonl`；运行时逐字核 `input_text`，用例另核原生 `CommandExecution.exit_code = 0`。二者都实际留下相同代码 diff，测试原生结果为：

```text
test_double (test_calculator.CalculatorTest.test_double) ... ok
Ran 1 test in 0.000s
OK
```

独立地在工作树再执行同一个测试也通过。尚未封存，因此当前断言是 `Running + runtime:seal_failed/WRITE_SEAL_UNAVAILABLE`，没有 Proposal 或 `TurnReturned`。这只证明真实执行这半段，不能据此声称 Proposal 的结果树一致。

## 待接的接口

#405 的 Bundle 从 `e5ab3de` 起使用 `repo_local_path / repo_local_machine / objective / publication_target`。Agency 核整份 `publication_target` 的规范摘要等于 Spec 的 `review_publish_policy.digest`，同时核 Repo 与绑定版本。无改动出口是 `no_changes`，控制面再用 Git 核验，不能靠模型文字造版本。

[接口处理说明](https://github.com/yesme/hctl2/pull/405#issuecomment-6094092852)记录所有者已指定的 mac/Ubuntu 分工。[工作树与封存入口处理说明](https://github.com/yesme/hctl2/pull/405#issuecomment-6094386044)记录目前 `seal → archive::snapshot → require_worktree` 对 detached 的拒绝。验收主笔随后提议第 2 条改用 P1 的分支工作树，已向所有者报出这项调整；未得到调整指令之前不把它当成已改的验收。Agency 不改 #405 的工具、控制面、runtime.rs、tenant.rs 和 port.rs。


## 两平台 CI 的处理

草稿首头 `6c465f5` 的 Code CI 在 Linux 通过，macOS 的 Claude 写入入口发现 `/tmp` 与 `/private/tmp` 指向同一目录时 cwd 比较失败。工作树交给 harness 前改用 Git 副本路径的 `canonicalize` 回读，入口用例也比较规范路径。重跑：

```sh
./src/buck2 test root//agency:write_session_test
./src/buck2 build root//agency:clippy
```

2026-10-10：Build `f76e954b-7e6f-423a-be3e-ffc7ec126fd7` 为 `3 passed; 0 failed; 2 ignored`；Clippy Build `3f6c9556-d175-485b-9f05-52770b6f5bc6` 通过，12 份诊断为空。修正后的 macOS 检查尚待 CI 回跑，不能用本机 Linux 结果代替。

首头完整打包 CI 两平台还在 `human_output_renders_dispatch_preview_sections_and_invocation_table` 失败：旧样本写死 `/bin/sh` 的程序摘要，不同平台实物摘要与它不一致。该文件属于 #405，`9e2d221` 已改成回读实际 `/bin/sh` 摘要，其余字段继续按原样本比较；本 PR 不并行改该文件，等待依赖合入后验证。

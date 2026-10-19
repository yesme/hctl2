# 写入会话完成之后的去留：调研

> 状态：调研 · 日期：2026-10-19<br>
> 类别：① Coding Harness · ⑥ 机械后端与基础设施（跨候选归纳） · 证据编号：E-L1-WRITE-SESSION-AFTERLIFE<br>
> 对象：Claude Code 2.1.302；Codex CLI 0.163.0（上游源码 `rust-v0.163.0`，`83086188`）；本库 main `88865a3` 的 Agency（`src/agency/`）与控制面写入租约；#431 走查留在本机的现场；业界 23 个系统的官方文档与源码<br>
> 定位：Informative 研究备忘录。回答开工书 §六「专项 · 写入会话完成之后的去留：调研」的四个问题，给推荐，但不定案；所有者拍板后另行改设计与代码。发布后正文不改，只在文末追加复核记录。证据分三档标注：**官方文档说**（2026-10-19 打开过的官方页面）、**源码看到**（本库代码、上游源码或本机帮助文本，给路径）、**实测**（本机命令，或对本机现场的只读复核）。本库既往记录单标**本库记录**。没查到的写「未查到」，不猜。

## 定位

这道题的出处是 #431 走查：真会话按「雇着就一直待命」不退出，Task 完成后写入租约停在 `revoking`，`changeset show` 一直挂着一条残留（开工书 §二 第 37 条 ④、§七）。所有者倾向于让会话像本地 harness 那样休眠，之后再唤醒；[交付文档 §本地 Agency 参考实现](../design/delivery.md#本地-agency-参考实现)里「只有闲置超时才冬眠，被杀后按会话文件恢复，唤醒不算新派工」是同一个方向。

查下来，题目的前提有两处要改：

1. **Agency 已经记下会话关闭，残留另有原因。** 走查现场里，turn 交回后约 2 秒，控制面的停止就到了，Agency 记下了会话已关（§一）。进程是否真的全部退出，没有核过：Agency 只拿到 Herdr `pane.close` 的成功，而它在有残留进程时也返回成功。租约停在 `revoking`，是因为常驻会话的停止报告里没有退出码这一项，控制面不认它是停止证明。这是 Agency 与控制面之间的缺口，与休眠无关：不补上，选哪种休眠方案，租约都收不回来。补的时候还要分清两件事：把报告格式对齐一次性路径，能让控制面接受；但一次性路径自己也没有证明进程已物理停止（§五 甲）。
2. **休眠在 Agency 里已经有了。** 写入会话交回结果之后，进程最多再活 300 秒的闲置期（`HCTL2_AGENCY_IDLE_MS`），收到停止、来了下一份派工、或 Agency 退出时也会关。下一次写入派工起一个新进程，按会话号续接上一次的对话：Claude Code 用 `--resume`，Codex 用 `thread/resume`（§三）。这就是「冬眠，唤醒时按会话文件恢复」，只是没有用这个名字。

所以原题拆成三件互相独立的事，§五 分别给候选与推荐：

- **甲 · 停止证明**：常驻会话关掉之后，报什么才算「已停止」。这一件挡住了租约的收回。
- **乙 · 工作树的去留**：租约收回以后，隔离工作树什么时候拆、谁来拆。现在没人拆，`changeset show` 会一直列着它。
- **丙 · 休眠本身**：什么时候睡、按什么续接、唤醒后带不带上一次的东西。现状大体可用，要定的是续接的键。

范围：只出调研，不改代码，不改设计正文。没有起新的真会话来做这份调研：实测只读本机现场与帮助文本，不打印凭据，不改任何一家 harness 的全局配置。

## 一、#431 的残留是怎么来的

本节回答：走查里那条 `revoking` 到底卡在哪。

**现场**（**实测**：只读打开 `~/hctl2-demo3/agency2/tenants/…/tenant.sqlite` 与 `~/hctl2-demo3/wt-log/b6-changeset.json`）。走查里的两次写入派工，Agency 记下的事件一模一样：

| 序号 | 事件 | 负载（节选） | 距 turn 交回 |
| --- | --- | --- | --- |
| 1 | `prepared` | `{}` | — |
| 2 | `runtime:session_opened` | `{"resumed": false, "resume_failed": false}` | — |
| 3 | `runtime:git_sealed` | `association_ref: refs/hctl2/changesets/cs-94db…` | — |
| 4–6 | `native_answer`、`result_available`、`turn_returned` | — | 0 |
| 7 | `stopped` | `{"requested_stop": true, "session_closed": true}` | 1.9 秒（另一次 2.5 秒） |

同一时刻 `changeset show` 报租约 `revoking`，外加一条 `materialized_residual`，路径是 `/tmp/hctl2-exec-f0d7…/write-worktrees/…/cs-94db…`，恢复动作是 `preview_takeover_adopt_or_discard`。

**控制面认什么**（**源码看到**）。结果准入的同一个事务把租约改成 `revoking`，注释写明「Result admission does not prove the writer stopped」（`src/crates/project/src/invocation/results.rs:429-432`）。之后控制循环反复发停止、读事件，判据只有一条（`src/apps/control/src/dispatch/recovery.rs:6-21`）：

- 要有一个来自 Agency 适配层的 `stopped` 事件；
- 负载里要有 `exit_code` 这个键（值可以是 `null`），而且 `terminal_result_present` 是布尔值；
- 或者派工从没开始过（`never_started: true`）。

满足了才写停止清理记录，`complete_revocation` 才把租约改成 `revoked`；它的文档注释是「A turn result, a stop request or loss is not such proof」（`src/crates/repo/src/changeset/leases.rs:211-212`）。这一条对应约束原文：「进程确已退出时附退出码」（[Participant 约束 §派工与观测](../design/spec/participant.md#派工与观测)）。

**Agency 报的是什么**（**源码看到**）。同一个 Agency 里有两条起 harness 的路，报告格式不一样：

| 路径 | 关会话之后报什么 | 控制面认不认 |
| --- | --- | --- |
| 一次性的 Herdr 启动（`launch.rs`，3c 起的路径） | `cancel` 先 `pane.close`，再发 `Exited {code: None}`，落成 `stopped {"exit_code": null, "terminal_result_present": …}`（`src/agency/src/launch.rs:272-276`、`:1009-1013`；`src/agency/src/tenant.rs:416-436`） | 认 |
| 常驻会话（Claude Code 的 `standby.rs`，Codex 的 `codex.rs`） | 关会话之后发 `TurnStopped {session_closed: true}`，落成 `stopped {"requested_stop": …, "session_closed": true}`，没有 `exit_code`（`src/agency/src/standby.rs:321-331`；`src/agency/src/tenant.rs:447-468`） | 不认 |

两条路掌握的证据其实一样弱：一次性路径也只是 `pane.close` 返回成功，既没有读到真实的退出码，也没有核过进程是否全部退出。所以一次性路径的报告被控制面接受，只说明它符合控制面现行的判据，不说明它满足 [Participant 约束 §ChangeSet 与 Git 事实](../design/spec/participant.md#changeset-与-git-事实)三态报告里的「已停止」。常驻会话的四种收尾方式（收到停止、闲置到期、下一份派工到来、Agency 退出）都走 `release_returned`，报的都是不被认的那种格式。Agency 重启以后情况更糟：控制面再发停止，派工停在 `ResultReturned`，又没有活着的会话，`tenant.stop` 什么事件都不写（`src/agency/src/tenant.rs:706-727`），这条租约就永远收不回了。重启后内存里没有会话，也不等于原来的进程已经不在。

控制面自己的测试也明确把 `session_closed: true` 判为不足（**源码看到**：`src/crates/project/tests/invocation.rs` 里的「turn」一例）。端到端走到 `revoked` 的只有脚本执行体那条用例，`/bin/sh` 退出时会发 `Exited`（**本库记录**：`src/apps/cli/tests/room/project.rs`）。所以 #430 的真会话用例能走到 Task 完成，却没有核租约收没收回。

**就算租约收回了，残留也还在。** `changeset show` 对 `revoking` 和 `revoked` 两种租约都会去查 Repo 的 Git 工作树，`hctl2/changeset/<id>` 分支上的每个工作树都列成 `materialized_residual`（`src/apps/control/src/changesets.rs:259-273`）。Agency 从不删写入工作树：`release_execution_dir` 只删 `<执行目录>/<派工号>`（`src/agency/src/confine.rs:52-62`），写入工作树按「unsealed edits survive cancellation」原样留着（`src/agency/src/write.rs:251`）。现在唯一的拆法是人在预览后执行 `changeset discard`。

## 二、两家 harness 的会话怎么休眠、怎么唤醒（问题 1）

本节回答：Claude Code、Codex 自己的会话怎么存、怎么续接，续接后哪些还在。先给对照表，再分家补充。

| 项 | Claude Code 2.1.302 | Codex CLI 0.163.0 |
| --- | --- | --- |
| 会话记录在哪 | `~/.claude/projects/<cwd 换成 - 的目录名>/<会话号>.jsonl`，按 cwd 加会话号分目录（官方文档说：[sessions](https://code.claude.com/docs/en/sessions#where-transcripts-are-stored)；实测：本机目录与首条记录的键一致） | `$CODEX_HOME/sessions/YYYY/MM/DD/rollout-<时间>-<thread 号>.jsonl`，只按 thread 号，cwd 记在文件里的 `session_meta`（源码看到：`codex-rs/rollout/src/lib.rs:86-87`；实测：本机 14 份都是 `history_mode: "paginated"`） |
| 保留多久 | `cleanupPeriodDays`，缺省 30 天，启动后后台清扫（官方文档说：[settings-reference](https://code.claude.com/docs/en/settings-reference#cleanupperioddays)） | 未查到按时间自动删的机制；只有手动 `archive` / `delete`（源码与文档都查了） |
| 怎么续接 | `--resume <会话号>` 从任何目录都能续：先找当前项目及其 git 工作树，再找本机所有项目，只在恰好一处有记录时才续（官方文档说：[sessions §Resume by session ID or name](https://code.claude.com/docs/en/sessions#resume-by-session-id-or-name)） | app-server 的 `thread/resume`（按 `threadId` 从磁盘「冷续接」）；命令行 `codex resume <ID>`、`codex exec resume`（源码看到：`codex-rs/app-server-protocol/src/protocol/v2/thread.rs:340`） |
| 原目录不在了 | 会话自己的工作树被删了，就在当前目录续，并清掉工作树绑定（官方文档说：[sessions §Sessions in other worktrees and projects](https://code.claude.com/docs/en/sessions#sessions-in-other-worktrees-and-projects)） | 请求里的 `cwd` 覆盖记录里的 cwd；记录的目录已删时会怎样，未实测、源码里没找到存在性检查 |
| 续接后还在的 | 完整对话（含工具调用与结果）、模型、权限模式（终端续接时；`bypassPermissions` 不恢复）、未过期的定时任务（官方文档说：[sessions §What a resumed session restores](https://code.claude.com/docs/en/sessions#what-a-resumed-session-restores)） | 从 rollout 重建模型历史，从最近一次压缩点起回放；审批策略、沙箱档、cwd 按记录恢复，请求可覆盖（源码看到：`core/src/session/rollout_reconstruction.rs`、`app-server/src/request_processors/thread_processor.rs:4040-4160`） |
| 续接后不在的 | 后台 Bash、监视任务、后台子代理随旧进程结束，只在对话里留一条「没做完」的记录；`--settings`、`--plugin-dir`、`--mcp-config`、`--add-dir` 要重新传（官方文档说：同上）。「本次会话允许」的授权在新进程里要重批：文档只写了 `--fork-session` 的情形，普通续接也一样是推论 | 「本会话已批准」存在内存里，新会话重建（源码看到：`core/src/tools/sandboxing.rs:42`）；后台终端随会话关闭被杀（源码看到：`core/src/session/handlers.rs`） |
| 自带的闲置机制 | 后台会话（agent view）做完或在等回复、又没人接上，约一小时后由 supervisor 停掉进程，对话留在磁盘，下次接上或回复时续接（官方文档说：[agent-view](https://code.claude.com/docs/en/agent-view)）；本机 supervisor 没在跑（实测：`claude daemon status`） | 没有订阅者、没有活动的已加载 thread，过 `thread_unload_delay_secs`（缺省 1800 秒）卸载，客户端收到 `thread/closed`（源码看到：`app-server/src/request_processors/thread_lifecycle.rs:23-120`；官方文档说：[app-server](https://learn.chatgpt.com/docs/app-server)，30 分钟） |
| 进程级闲置退出 | `CLAUDE_CODE_EXIT_AFTER_STOP_DELAY`：查询循环闲下来多少毫秒后自动退出，SDK 模式用（官方文档说：[env-vars](https://code.claude.com/docs/en/env-vars)） | 未查到；stdio 模式的 app-server 在连接关闭时退出（源码看到：`app-server/src/lib.rs:793`）；本机有 ws 模式的 app-server 活了 7 天以上（实测：`ps`） |
| 一个会话能不能同时在两个进程里 | 同一会话在两个终端里续而不分叉，消息会交错写进同一份记录（官方文档说：[sessions §Branch a session](https://code.claude.com/docs/en/sessions#branch-a-session)） | 不能：每个 thread 有独占写锁 `$CODEX_HOME/thread-writer-locks/<id>.lock`，第二个进程报「already has an active writer」（源码看到：`codex-rs/rollout/src/writer_lock.rs`） |
| 隔了很久再续接的代价 | 续接会重发整段对话，只有仍在缓存时限内的前缀能命中；主对话缓存是 1 小时（订阅额度内）或 5 分钟（官方文档说：[prompt-caching](https://code.claude.com/docs/en/prompt-caching)） | `prompt_cache_key` 取 thread 号，续接后不变（源码看到：`core/src/client.rs:646`）；缓存时限按 OpenAI 的说明，内存档闲置 5–10 分钟、最长 1 小时，支持的模型缺省 24 小时（官方文档说：[prompt caching](https://developers.openai.com/api/docs/guides/prompt-caching)） |

两家的共同点有三条，§五 要用到：

- **能续接的是对话，不是进程。** 两家都把对话记录写在磁盘上，按会话号或 thread 号取回。进程里的东西——后台命令、会话内授权、MCP 连接——续接时都不带回来。
- **续接时的权限由新进程的启动参数决定。** Claude Code 的 `--settings`、`--plugin-dir` 要重新传，Codex 的审批与沙箱档请求可以覆盖。这正好让「下一次派工不续用前一次派工的权限」容易做到：换权限就是换进程。
- **两家都有闲置机制，但停的东西不一样。** Claude Code 的 agent view 闲置约一小时后停掉会话进程，对话留在磁盘。Codex 的 app-server 闲置 30 分钟后卸载的是 thread，并发 `thread/closed`；app-server 进程本身不因此退出，进程级的闲置退出未查到。所以不能拿这一条说明 Agency 下的 Codex 进程会自己释放。

## 三、Task 完成后谁占着什么（问题 2）

本节回答：一次写入派工交回结果后，各项资源现在什么时候收、谁收，休眠期间谁还占着什么。

**写入派工交回结果之后的时间线**（**源码看到**，Claude Code 路径；Codex 结构相同，见 `src/agency/src/codex.rs:178-313`）：

1. 控制面准入结果，同一事务把租约改成 `revoking`，作废这次派工的授权，排一个 `agency.stop`（`src/crates/project/src/invocation/results.rs:429-438`）。
2. Agency 把这份派工记成「已交回」，保留停止入口，原生会话不关（`src/agency/src/standby.rs:266-267`、`:289-291`）。这时进程还活着，cwd 在写入工作树里，手上有 `Read,Edit,Write,Bash,Glob,Grep` 和 `acceptEdits`。
3. 控制循环每秒推进一次，发停止（`src/apps/control/src/dispatch.rs:396-403`；`recovery.rs:92-203`）。Agency 收到后 `pane.close`，报 `TurnStopped {session_closed: true}`，然后释放派工（`standby.rs:222-232`、`:859-866`）。
4. 控制面读到的报告不够格（§一），回 `READBACK_REQUIRED`，此后每秒重试，租约一直停在 `revoking`。

**各项资源的去留：**

| 资源 | 现在什么时候收 | 谁收 | 休眠期间（进程已关、等下一份派工）还占着吗 | 证据 |
| --- | --- | --- | --- | --- |
| 写入租约 | 应当在 Agency 报出够格的停止报告后转 `revoked`；常驻会话永远到不了 | 控制面 | 占着，停在 `revoking`：同一个 ChangeSet 不能再授租约（`WRITE_LEASE_BUSY`），不带 `change_set_id` 的新写入会拿到新的 ChangeSet | 源码看到：`src/crates/repo/src/changeset/leases.rs:36-44`、`:80-86` |
| 这次派工的授权 | 结果准入时作废 | 控制面 | 不占 | 源码看到：`results.rs:429-438` |
| 原生 harness 进程 | 收到停止就关（走查里约 2 秒）；没有停止时闲置 300 秒关；下一份派工到来时关。「关」指 `pane.close` 成功，进程是否全部退出未核 | Agency | 按 Agency 的记录不占；有没有残留进程，未核 | 源码看到：`standby.rs:222-256`；`launch.rs:414-425`。实测：走查现场的事件记录 |
| Herdr pane | 随进程一起 `pane.close`；每次起会话新建一个 `agency-standby` 工作区 | Agency 经 Herdr | 不占 pane；工作区关后是否回收，未查到 | 源码看到：`standby.rs:714-777`、`:859-866` |
| 隔离工作树 | 不收：跨租约代次复用，不重置 | 只有人经 `changeset discard` 能拆 | 占着，位于 `/tmp/hctl2-exec-<摘要>/write-worktrees/<租户与 ChangeSet 的摘要>/<cs>` | 源码看到：`src/agency/src/write.rs:165-227`、`:251`；`confine.rs:11-26` |
| 会话记录 | harness 自己的保留期（Claude 30 天；Codex 未查到自动删） | harness | 占着磁盘，这是续接的依据 | 见 §二 |
| Agency 的续接指针与私有目录 | 不收：`standby-<键>/` 下的 `resume.json`、`settings.json`、`plugin/`、私有 `tmp/`；Codex 的 `codex-<键>/thread.json` | — | 占着，键是租户、绑定、Project、参与者选择四者的摘要 | 源码看到：`standby.rs:198-205`、`:655-671`；`codex.rs:187`、`:823-843` |
| 写凭据 | 不发给 harness：封存由 Agency 自己清空环境后调 `hctl2-tool` 完成；Claude 写入会话拒 `git push`、`gh`、`git credential` | — | 不占；唯一的写权是物理上的：一个活着、有写工具、cwd 在工作树里的进程 | 源码看到：`write.rs:310-326`、`:482-564`；`standby.rs:690-695`、`:760-766` |

两处补充：

- **下一次写入派工就是唤醒。** 同一参与者选择的下一份写入派工到来时，Agency 关掉旧进程（「A new frozen Spec must not enter a process retaining the old write lease」，`standby.rs:248-256`），按 `resume.json` 里的会话号起新进程续接；新会话号对不上就关掉、重开一个新会话，并报 `resume_failed`（`standby.rs:655-671`）。权限由新派工的执行规格重新写进 `settings.json`，旧派工的授权不进新进程，满足「下一次派工不续用前一次派工的权限」（[交付文档 §本地 Agency 参考实现](../design/delivery.md#本地-agency-参考实现)）。本库的 3f 调研对这一做法有记录（**本库记录**：[agency-write-session.md](./agency-write-session.md)「切 cwd 或工具权限时关闭旧进程再原生 resume」）。
- **续接的键是参与者，不是 Task。** 键里没有 Task 或 ChangeSet：同一个人雇的同一个参与者，在同一个 Project 里接下一个毫不相干的 Task，也会续上一次的对话。每次 cwd 都换成新的工作树，Claude Code 按会话号跨项目续接（§二），所以技术上走得通；要不要这样，是 §五 丙要定的事。

## 四、业界怎么做（问题 3）

本节回答：别家在「任务做完之后」怎么处理进程、工作区、凭据与续接。23 个系统里挑出与本题最近的 15 个列表；主要出处列在 §证据。「未查到」指官方文档与能读到的源码里都没找到。

| 系统 | 什么时候停 | 停了留什么 | 任务结束时凭据怎么办 | 续接 | 留多久 |
| --- | --- | --- | --- | --- | --- |
| Claude Code 网页版 | 闲置几分钟，VM 暂停；之后可能被回收 | 暂停：同一台 VM；回收：新 VM，只带回对话，子代理和 shell 命令不带 | GitHub 凭据不进 VM，VM 里只有代理会核对、替换的限定凭据 | 同一会话号；归档后不能再发消息 | 商业账号 30 天 |
| Claude Code 自托管 runner | `--release-idle-session-min`（缺省 0，不开）、`--kill-session-after-min` | 只留对话；续接时重新克隆，没推的改动丢 | 会话 JWT 缺省 4 小时、最长 8 小时，到期前一直有效，**没有吊销** | 下一条消息到来时在新 runner 上续 | — |
| OpenAI Codex 云端任务 | 未查到闲置计时 | 每个任务一台 VM，文件含未提交改动 | 程序只拿到占位符，代理替换真值；GitHub 凭据未查到 | 重开同一任务 | 最后一轮后 7 天 |
| GitHub Copilot coding agent | 任务做完即结束，最长 59 分钟 | 环境销毁，只剩推上去的提交 | 有自己的令牌，时限与吊销未查到 | PR 评论起**新会话**，从 PR 带回上下文 | 可归档，不能删 |
| Cursor 云端 agent | 一次运行闲下来后，按计时先休眠再删 | 快照存在 VM 外，对话在 | OIDC 令牌 5 分钟；密钥在启动时注入 | 同一个 agent 上起新一次运行，一次只许一个活跃运行 | 快照 30 天滚动，对话不限 |
| Devin | 闲置 30 分钟睡（企业 5–120 分钟） | 托管版睡着留什么未查到 | 未查到 | 发一条消息就醒；自托管时续接是一个新的队列认领 | 合作关系存续期间 |
| OpenHands | 云端闲置 30 分钟暂停；企业版另有 12 小时强制暂停 | 本机 `docker pause` 冻结进程；云端缩到 0，卷保留 | 沙箱拿到的令牌 14 天有效，**暂停、删除都不吊销** | 下次访问自动续 | 停 1 天删 K8s 资源，30 天删对象存储 |
| Vibe Kanban | 每轮一个 `claude -p` 进程，跑完就退 | 分支；工作树 72 小时无活动删，归档后 1 小时删 | 用人自己的 git | 新进程 `--resume <会话号>`，工作树从分支重建，未提交改动丢 | 到人删 |
| claude-squad | 只有人手动暂停 | 暂停时本地提交，删工作树、留分支，tmux 不关 | 用人自己的 git | 从分支重建工作树；进程已死就新起，不带 `--resume`，上下文丢 | 到人删 |
| Dagger container-use | 没有长驻容器 | 环境分支加 git notes，容器随时可重建 | 密钥是引用，每次执行时解析 | 新对话报环境号，在原环境续 | 手动；`prune` 缺省一周 |
| GitHub Actions | 作业结束；临时 runner 跑完一个作业就注销 | 常驻 runner 留 `_work` 目录 | **作业令牌在作业结束时失效**，`actions/checkout` 写盘的令牌在收尾时删 | 不续，新作业 | ARC 无作业时缩到 0 |
| GitLab Runner | 作业结束；自动扩缩有 `idle_time`、`max_use_count` | 卷跨作业保留 | **作业令牌在作业结束时吊销**；OIDC 令牌按到期 | 不续，新作业 | — |
| Buildkite | `--disconnect-after-job` 或闲置超时（缺省不超时） | 构建目录 | **作业令牌随作业结束失效**；注册令牌长期、会话令牌随连接 | 不续，新作业 | 弹性栈闲置 600 秒缩 |
| GitHub Codespaces | 闲置 30 分钟（5–240），最长 12 小时 | 磁盘留，进程停 | **每次创建或重启发新令牌、上新 VM** | 重启 | 停 30 天删，不管有没有没推的改动 |
| Coder | 定时停加活动顺延（缺省 1 小时）；长期不用转休眠再自动删 | 模板决定，家目录卷留到工作区删 | **每次构建发新 agent 令牌**，只有最新构建能认证；git 令牌按需取 | 新构建 | 休眠后自动删 |

五条共通做法：

1. **凭据跟着作业走，不跟着主机走。** GitHub Actions、GitLab、Buildkite 在作业开始时发令牌，作业结束时在服务端作废，runner 自己的长期凭据作业看不见；Codespaces、Coder 每次重启或构建都换新令牌。反例是自包含、不能吊销的令牌（Claude Code 自托管的 JWT、OpenHands 14 天的令牌），只能靠到期。更进一步的做法是凭据根本不进沙箱，由代理替换（Claude Code 网页版、Codex 云端）。
2. **停分三档，各有各的钟。** 闲置停进程按分钟（15–30 分钟居多），转冷存按天，删除按周（7–30 天）；另有一条连续运行的硬上限（Copilot 59 分钟、Codespaces 与 OpenHands 12 小时）。
3. **停进程留磁盘是常态，留内存是另一档、而且脆。** 托管的编码代理都不承诺进程跨休眠存活：Claude Code 回收后子代理不在，Conductor 云端明说进程不保留。能冻住进程的（E2B、Fly suspend、`docker pause`）是通用沙箱，不是编码代理。
4. **续接是一次新作业：新进程、常常是新机器和新令牌，带上旧对话和旧工作区。** Copilot、Claude Code 自托管、Vibe Kanban、container-use 都是这样。
5. **只有推上去的分支是保证。** Codespaces 删除时不管有没有没推的改动；Claude Code 自托管写明没推的工作会丢。缓解办法是拆之前先提交或推（Conductor 归档、claude-squad 暂停、Vibe Kanban 自动提交）。

对本题：第 1 条与「下一次派工不续用前一次派工的权限」同向，本库已经做到，而且更严，写凭据压根不进 harness（§三）。第 4 条与 Agency 现在「下一份派工起新进程续接」一致。本库缺的是第 2 条里「删」那一档：工作树没有任何一档会拆。第 5 条在本库有更好的前提：工作树里的改动在交回结果之前已经封存进 `refs/hctl2/changesets/…`，合入后还进了目标分支。封存之后又多出来的东西（包括被忽略的文件）不在其中，所以拆之前还要核一遍（§五 乙）。

## 五、候选方案与推荐（问题 4）

本节回答：三件事各有哪几种做法、各自的代价，推荐哪一种。推荐只是推荐，由所有者定。

### 甲 · 停止证明（挡着租约的收回，与休眠无关）

| 候选 | 做法 | 好处 | 坏处 |
| --- | --- | --- | --- |
| 甲 1 · 两条路报同一种格式 | 常驻会话关掉之后，照一次性路径的样子报 `stopped {"exit_code": …, "terminal_result_present": <实际值>}`：`terminal_result_present` 照一次性路径查持久的结果记录，不写死（`tenant.rs:416-436`）；取消、截止、协议错误关掉的会话可能没有终局结果。Codex 报自己 `wait` 到的真实退出状态（`codex.rs:679-688` 现在拿到了却丢掉）；Claude Code 经 Herdr 拿不到退出码，报 `null` | 改动只在 Agency 一侧，控制面的判据不动；租约能收回 | **它沿用了一次性路径现有的证据缺口，不是纯格式修补。** Herdr 的 `pane.close` 在仍有残留进程时也返回成功，API 不报残留（**本库记录**：[agency-runtime-validation-20260829.md](./runtime/agency-runtime-validation-20260829.md) 停止结果一行；[sdk/herdr.md](./sdk/herdr.md) `pane.exited` 没有退出码）。Claude 的 Bash 工具若起了脱离进程组的后台进程，它可能还在写工作树；这是诚实路径上的反例，不需要谁故意伪造 |
| 甲 2 · 甲 1 加残留回读 | 关掉之后，Agency 按会话的进程组回读还有没有活着的进程，有就不报停止，改报「不能证明」 | 满足三态报告的「已停止」；和 [Agency 运行服务验证清单](./runtime/agency-runtime-validation-20260829.md)「HUP/TERM/KILL 后回读残留进程」那条对上 | Herdr 不报 PID，要 Agency 自己按进程组查，Linux 与 macOS 各写一遍；自己脱离进程组的进程按进程组查不到，更强的归属办法本轮未核 |
| 甲 3 · 第二态「仍存活但已受限」 | 不关进程，由 Agency 证明它已被限制在旧工作树与旧 ChangeSet 里（[Participant 约束 §ChangeSet 与 Git 事实](../design/spec/participant.md#changeset-与-git-事实)的三态报告第二态） | 进程可以不停，真正的「待命」 | Claude Code 的工具与权限在启动时定死，进程不重启就收不回写工具；要靠操作系统层的边界在运行中收紧，本库没有这种机制。代码里 `isolation_confirmed` 现在恒为 `false` |

**推荐甲 1 与甲 2 一起做，两条路都上。** 分开看两件事：
- 「控制面接受」：甲 1 就够，真会话的写入租约从此能收回。现在它在任何收尾方式下都收不回来。
- 「满足三态报告的已停止」：要甲 2。只做甲 1，常驻路径就和一次性路径带上同一个缺口：`pane.close` 成功了，进程却可能还在。这个缺口一次性路径现在已经有了，甲 2 要一起补到它身上。

如果甲 2 赶不上同一个 ChangeSet 追加修改要用租约的时候，可以先单做甲 1 作过渡，但要明写它沿用了这个缺口。这由所有者定。甲 3 代价大、收益是「进程不停」，而 §二 已经说明续接不靠进程，不推荐。

另有一处要一并修：Agency 重启后再收到停止，派工停在 `ResultReturned` 又没有会话，现在什么都不写（`tenant.rs:706-727`）。应当补一条报告，但不能只凭内存里没有会话就报「已停止」。要按持久记录找到原来的会话，核过它的进程确已不在才报已停止；核不了就报「不能证明」。

### 乙 · 工作树的去留

现在没人拆，`changeset show` 会一直列着。[Repo 约束 §ChangeSet 与 Git 事实](../design/spec/repo.md#changeset-与-git-事实)的原文是「物理清理默认保全：`hctl2-tool` 先确认所有已跟踪、未跟踪且尚未封存的修改已有可恢复副本，现场资源得到该确认才可拆除」。所以自动拆是约束允许的，只是没人做。

| 候选 | 做法 | 好处 | 坏处 |
| --- | --- | --- | --- |
| 乙 1 · 维持现状 | 留到人预览后接管、采纳或丢弃 | 不改 | 每个做完的 Task 都留一条残留，残留不再能当成「要人处理」的信号 |
| 乙 2 · 租约收回就拆 | 租约转 `revoked` 时，`hctl2-tool` 先核工作树相对已封存提交没有多出的东西：已跟踪文件的改动、未跟踪文件、被忽略的文件都算。没有就拆，有就留着并列为残留 | 和 CI 一样，作业结束就清；残留只剩真要人看的 | 拆了之后，同一参与者的下一份写入若想接着改同一个 ChangeSet，要从封存提交重新物化一份（物化 3f 已有；能不能从封存提交起，本轮未核） |
| 乙 3 · 分档：Task 完成后拆，干净才拆 | Task 还开着时留着，方便追加修改；Task 完成（或按 Agency 的配置过 N 天）后照乙 2 的核对拆 | 对应业界「停进程按分钟、删按天」两档；追加修改不用重新物化 | 要有人把「Task 完成」告诉 Agency，或由 Agency 自己定时扫 |

**推荐乙 2。** 理由三条：租约收回本身就意味着这个 ChangeSet 的写入结束了，再改要新租约；改动在交回结果前已经封存进 Git，拆工作树不丢东西，正是约束要的「有可恢复副本」；乙 3 多出的那一档要新的通知或定时扫，换来的只是少物化一次。谁来拆：工作树在 Agency 的执行目录里，远程 Agency 的磁盘控制面摸不到，所以由 Agency 在报出够格的停止报告之后自己拆，控制面照旧只核 Git 事实。

实现层脚注：核「干净」不能用工作树相对 HEAD 的 `git status`。走查现场里封存之后 HEAD 仍是基线、`calculator.py` 显示为已修改，内容其实已经在 `refs/hctl2/changesets/…` 里了；核对要比工作树与封存提交的树。被忽略的文件也要算进去，不能只比 Git 跟踪的文件：它们同样没有可恢复的副本。

### 丙 · 休眠本身

现状（§三）已经是「交回就停进程、对话留在磁盘、下一份派工起新进程续接」，和业界第 3、4 条一致，也满足交付文档的两条不变量。要定的是两件：

| 问题 | 候选 | 推荐 |
| --- | --- | --- |
| 续接按什么键 | 丙 1 · 按参与者（现状：租户、绑定、Project、参与者选择）；丙 2 · 在现状四项之外再加 Task；丙 3 · 在现状四项之外再加 ChangeSet | **丙 3 · 再加 ChangeSet。** 现有的四项隔离一项不删，只是多分一层。 不相干的 Task 续上一次的对话，上下文是噪声，还会越积越长、每次续接都重付一遍缓存，所以不维持丙 1。业界最近的几家按任务或 PR 续（Codex 云端按任务、Copilot 按 PR、Vibe Kanban 按工作区），ChangeSet 正对应「一个 PR」：对评审意见的追加修改是同一个 ChangeSet 的新版本，续得上；新的 Task 拿新的 ChangeSet，从头开始。执行规格的写入租约引用里已经带着 ChangeSet，Agency 不用改端口。丙 2 语义更宽，但执行规格里没有 Task 号，要加字段，改的是冻结的端口清单。只读派工没有 ChangeSet，照旧按参与者 |
| 什么时候睡 | 丙 4 · 交回即停（现状：控制面的停止约 2 秒就到）；丙 5 · 交回后再闲置一段（300 秒，`HCTL2_AGENCY_IDLE_MS`）才停 | **维持丙 4，并把它写明。** 控制面在结果准入时就作废了授权；进程多活的那几分钟拿着写工具，却没有任何派工给它授权，正是租约要防的。300 秒的闲置只在控制面睡着、停止迟迟不到时兜底，保留。这也和交付文档「闲置与整理参数是它自己的配置，控制面不记也不核」一致 |

「唤醒不算新派工」这句，现状的读法是：唤醒只发生在新派工到来时，续接的是对话，不是授权，所以不会把唤醒算成新的派工，也不会把旧授权带进来。交付文档那一句不用改，但值得在设计正文里写明：唤醒带回的只有对话；权限、工作树、租约都来自新派工。

### 待所有者定的清单

1. **甲：常驻会话关掉之后报什么。** 推荐甲 1、甲 2 一起做，两条路都上：报告格式对齐一次性路径，让控制面接受；残留回读决定报「已停止」还是「不能证明」，补上 `pane.close` 成功却仍有进程的缺口。甲 2 赶不上时，先单做甲 1 作过渡、明写沿用了这个缺口，由所有者定。一并修 Agency 重启后停止无报告的那一处，补报要以核过的停止事实为准。
2. **乙：租约收回后，由 Agency 核对封存后自动拆工作树。** 推荐乙 2：已跟踪的改动、未跟踪与被忽略的文件，相对封存提交都没有多出的才拆；有就留着并列为残留。
3. **丙：写入会话续接的键，在现有四项之外再加 ChangeSet。** 推荐丙 3：租户、绑定、Project、参与者选择的隔离照旧；不改端口；同一个 ChangeSet 的追加修改续上下文，新的 Task 从头开始。要按 Task 续（丙 2），执行规格得加 Task 号。和乙 2 一起做时要核一处：工作树拆了之后，同一个 ChangeSet 的追加修改要从封存提交重新物化，`hctl2-tool worktree materialize` 能不能从封存提交起，本轮没有核。
4. **丙：交回即停写进设计正文，300 秒闲置只作控制面迟到时的兜底。** 推荐维持现状并写明。

甲、乙是 Agency 与控制面之间的缺口，演示 3 不经过它们（#430 的真会话用例没有核租约），可以排成演示 3 之后的小活；丙要改交付文档的一句话，随设计讨论落。

## 证据

**本库代码**（main `88865a3`，**源码看到**）：

- 控制面：`src/apps/control/src/dispatch/recovery.rs:6-21`（停止证明判据）、`:92-203`；`src/apps/control/src/dispatch.rs:396-403`；`src/apps/control/src/changesets.rs:259-273`；`src/crates/project/src/invocation/results.rs:429-438`；`src/crates/repo/src/changeset/leases.rs:36-44`、`:80-86`、`:211-347`。
- Agency：`src/agency/src/standby.rs:198-205`、`:222-256`、`:266-291`、`:321-331`、`:655-671`、`:690-777`、`:859-866`；`src/agency/src/codex.rs:178-313`、`:671-691`；`src/agency/src/tenant.rs:416-468`、`:706-727`；`src/agency/src/launch.rs:272-276`、`:414-425`、`:1002-1060`；`src/agency/src/write.rs:165-227`、`:251`、`:310-326`；`src/agency/src/confine.rs:11-26`、`:52-62`。

**本机实测**（2026-10-19，ubuntu）：

- `claude --version` → `2.1.302 (Claude Code)`；`codex --version` → `codex-cli 0.163.0`；`claude daemon status` → not running。
- #431 走查现场只读复核：`~/hctl2-demo3/agency2/tenants/628ec191…/tenant.sqlite` 以 `mode=ro` 打开，读 `events` 表；`~/hctl2-demo3/wt-log/b6-changeset.json`。
- 会话记录的目录结构与首条记录的键（没读正文）：`~/.claude/projects/`、`~/.codex/sessions/`。

**上游源码**：openai/codex `rust-v0.163.0`（`83086188ae0614bd882818ff1aa23f3194962012`，2026-10-15），路径相对 `codex-rs/`，见 §二 表内。业界开源项目：OpenHands（`openhands/app_server/sandbox/docker_sandbox_service.py`、`app_conversation/live_status_app_conversation_service.py`）、BloopAI/vibe-kanban（`crates/executors/src/executors/claude.rs`、`crates/local-deployment/src/container.rs`）、smtg-ai/claude-squad（`session/instance.go`、`session/git/worktree_ops.go`）、dagger/container-use（`environment/README.md`、`cmd/container-use/prune.go`）、coder/coder（`coderd/provisionerdserver/provisionerdserver.go`）；各读当天主干，未钉提交。

**官方文档**（均 2026-10-19 读取）：

- Claude Code：[sessions](https://code.claude.com/docs/en/sessions)、[agent-view](https://code.claude.com/docs/en/agent-view)、[settings-reference](https://code.claude.com/docs/en/settings-reference#cleanupperioddays)、[env-vars](https://code.claude.com/docs/en/env-vars)、[prompt-caching](https://code.claude.com/docs/en/prompt-caching)、[cloud-environments](https://code.claude.com/docs/en/cloud-environments)、[claude-code-on-the-web](https://code.claude.com/docs/en/claude-code-on-the-web)、[self-hosted-environments-reference](https://code.claude.com/docs/en/self-hosted-environments-reference)、[self-hosted-environments-deploy](https://code.claude.com/docs/en/self-hosted-environments-deploy)、[self-hosted-environments-identity](https://code.claude.com/docs/en/self-hosted-environments-identity)、[data-usage](https://code.claude.com/docs/en/data-usage)。
- Codex 与 OpenAI：[app-server](https://learn.chatgpt.com/docs/app-server)、[developer-commands](https://learn.chatgpt.com/docs/developer-commands?surface=cli)、[cloud-environments](https://learn.chatgpt.com/docs/environments/cloud-environments.md)、[prompt caching](https://developers.openai.com/api/docs/guides/prompt-caching)。
- 编码代理：[Copilot coding agent 概念](https://docs.github.com/en/copilot/concepts/agents/coding-agent/about-coding-agent)、[Copilot 负责任使用](https://docs.github.com/en/copilot/responsible-use/copilot-cloud-agent)、[Copilot 管理与跟踪](https://docs.github.com/en/copilot/how-tos/copilot-on-github/use-copilot-agents/manage-and-track-agents)、[Cursor cloud agent API](https://cursor.com/docs/cloud-agent/api/endpoints)、[Cursor 安全](https://cursor.com/docs/cloud-agent/security)、[Devin 用量](https://docs.devin.ai/admin/billing/usage)、[OpenHands 会话与沙箱](https://docs.openhands.dev/enterprise/conversations-and-sandboxes.md)、[OpenHands 沙箱归档](https://docs.openhands.dev/cookbook/archive-sandbox.md)、[Vibe Kanban 工作区](https://vibekanban.com/docs/workspaces/managing-workspaces.md)、[container-use 环境](https://container-use.com/environment-workflow.md)、[Conductor 云端工作区](https://www.conductor.build/docs/cloud-beta/working-with-cloud-workspaces)。
- 通用沙箱（§四 第 2、3 条）：[E2B 持久化](https://docs.e2b.dev/sandbox/persistence)、[Fly suspend 与 resume](https://docs.fly.io/reference/suspend-resume)、[Daytona 沙箱](https://www.daytona.io/docs/en/sandboxes)。
- CI：[GITHUB_TOKEN](https://docs.github.com/en/actions/concepts/security/github_token)、[自托管 runner 自动扩缩](https://docs.github.com/en/actions/hosting-your-own-runners/managing-self-hosted-runners/autoscaling-with-self-hosted-runners)、[GitLab CI_JOB_TOKEN](https://docs.gitlab.com/ci/jobs/ci_job_token/)、[Buildkite 令牌](https://buildkite.com/docs/agent/v3/tokens)、[Buildkite agent 配置](https://buildkite.com/docs/agent/v3/configuration)。
- 开发环境：[Codespaces 超时](https://docs.github.com/en/codespaces/setting-your-user-preferences/setting-your-timeout-period-for-github-codespaces)、[Codespaces 安全](https://docs.github.com/en/codespaces/reference/security-in-github-codespaces)、[Codespaces 自动删除](https://docs.github.com/en/codespaces/setting-your-user-preferences/configuring-automatic-deletion-of-your-codespaces)、[Coder 工作区排程](https://coder.com/docs/user-guides/workspace-scheduling)、[Coder 模板排程](https://coder.com/docs/admin/templates/managing-templates/schedule)。

**没查到的：** Herdr 0.9.3 的 `pane.close` 返回时 Claude 进程是否一定已退出；每次起会话新建的 Herdr 工作区关后是否回收；Codex 记录里的 cwd 已删时续接会怎样；Claude Code「本次会话允许」的授权在普通 `--resume` 新进程里是否保留（只查到 `--fork-session` 的情形）；Codex 云端、Cursor、Claude Code 网页版闲置计时的具体数值；Copilot agent 令牌的时限与吊销。

两处与本题无关、顺带看到的不一致，记下不展开：Vibe Kanban 文档说归档的工作树「Kept on disk」，代码是归档 1 小时后删；本库交付文档写「工作树的创建与清理复用 Herdr」（[交付文档 §本地 Agency 参考实现](../design/delivery.md#本地-agency-参考实现)），代码用的是 `hctl2-tool`。

## 复核记录

- 2026-10-19，合入前，Codex（mac）找问题（#459）。按意见改了正文四处：
  - §定位、§一：「会话其实停了」改成「Agency 记下了会话关闭，物理停止未核」；一次性路径被控制面接受，不等于满足三态报告。
  - §二：Codex 的闲置机制卸载的是 thread，不是进程。
  - §五 甲：甲 1 的 `terminal_result_present` 按持久的结果记录取值；甲 1 沿用了一次性路径的证据缺口，推荐改为甲 1、甲 2 一起做；重启后的补报以核过的停止事实为准。
  - §五 乙、丙：乙 2 的核对算上未跟踪与被忽略的文件；丙 3 是在现有四项隔离之外再加 ChangeSet，不删原有的维度。

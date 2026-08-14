# L1 · Execution & Runtime — Worktree / Harness / Terminal / Diff

> Status: Normative · Draft v0.7.0<br>
> Primary reference: Stably Orca<br>
> Parent: [四层设计规范](../README.md)

## 这一层为什么存在

L1 回答：**哪个具体执行实例在什么隔离边界中运行、使用哪个 Harness binding、写入哪个 ChangeSet，以及如何精确观察、恢复或接管。**

这是最接近操作系统和工具生态的一层，也是变化最快的一层。Harness、provider protocol、worktree strategy、runtime backend 和 terminal client 都可以替换；Project、Task、Run、Seat 和 evidence contract 不能随之漂移。

Stably Orca 对 worktree、daemon-owned PTY、terminal/session restore、remote reconnect 和 diff/ship workflow 的探索最深，因此是本层主要参考。其他产品顺带提供 embedded terminal/worktree，并不足以成为本层第二条产品主线。

## 本层负责什么

L1 负责：

- Harness Definition/Installation/Capability catalog 与 preflight；
- ACP、app-server/JSON-RPC、Vendor SDK、native CLI+PTY、hook/transcript 等 adapter binding；
- Attempt/RoomInvocation 的 process lifecycle、heartbeat、cancel 和 observed state；
- ChangeSet、worktree、branch、diff、test/SCM evidence；
- RuntimeBackend container、generation、fence 和 reconcile；
- exact terminal target、AttachDescriptor、TerminalInputLease 与 TerminalGateway；
- Execution Chat Projection、semantic resume 与 replay；
- xterm.js embedded client 和 WezTerm external escape path。

Attempt 是与 L2 共享的 seam：control 创建并拥有 Attempt identity、Seat relation、candidate decision 与 result admission；agentd 只实现 frozen AttemptSpec 并拥有 concrete process/runtime observation。L1 不创建第二个 Attempt lifecycle truth。

L1 不负责：

- 定义 Project/Task/Room/Run identity；
- 选择 semantic Gate outcome、candidate policy 或 quorum；
- 因 process exit、screen state、commit 或 Agent self-report 完成 Seat/Task；
- 把 donor session/worktree database 变成 HCTL truth；
- 自己启动未经 L2/L4 command admission 授权的写入。

## Harness Catalog 与 capability-first binding

Harness discovery 是 definition-first，而不是扫描任意 binary：

1. 内置 HarnessDefinition 或 ACP registry manifest 描述 probe/install；
2. 检查 PATH、标准目录和 package managers；
3. 运行 version probe 与安全 preflight；
4. 独立检测 auth/config；
5. 建立 capability snapshot；
6. 用户确认后创建 WorkerProfile。

`installed`、`authenticated`、`healthy`、`enabled`、`capacity` 必须分开。自动安装/升级是显式动作，discovery 不擅自联网或改 Harness 配置。

接入不是“ACP 或 raw PTY”两档：

| Binding | 适用情况 | 典型能力 |
| --- | --- | --- |
| ACP | 标准 structured session | prompt/stream/tool/permission/file/MCP/session |
| Provider JSON-RPC / app-server | provider 本地服务更完整 | structured event、resume、approval、usage |
| Vendor SDK | 保留专有或 remote capability | typed tool/session/control |
| Native CLI + PTY | 需要 native TUI 或 exact keyboard takeover | raw output/input、same PTY |
| Hook + transcript | 不改变 native process 的状态投影 | lifecycle、attention、tool/result projection |
| OSC/title/screen classifier | 无权威事件时的最低置信提示 | advisory working/attention/idle |

Adapter 按本次 Invocation/Attempt required capability 选择最高 fidelity 路径，而不是全局固定优先级。每次 binding 冻结 adapter kind/version、session identity、capability snapshot 与 degradation reason。

ACP、provider session 和 transcript 都不是 Room；它们只是 execution trace/control sources。

## Event normalization 与 observed-state authority

agentd 将 provider/runtime event 归一为 MessageDelta、ToolCall/Result、Plan、PermissionRequest、Question/Answer、Progress、FileDiff、Delegation、Usage、Process/Terminal state、FinalResult/Error。

Authority 按维度判断：

| 状态维度 | Authority order |
| --- | --- |
| Liveness | RuntimeBackend/process/lease > provider lifecycle/hook > OSC/title/screen |
| Semantic/action state | adapter 声明的 provider protocol/native hook > transcript-derived > OSC/title/screen |

每个 observation 携带 `source / confidence / evidence / observed_at`。即使最高置信 observation 也只能形成 result proposal，不能自行签发 Receipt。

## ChangeSet、Worktree 与 Git truth

Worktree 按 ChangeSet/write boundary 懒创建：

- Planning 与 read-only research 不创建；
- 首个 authorized write RoomInvocation 或 READY Obligation 才创建；
- 一个 ChangeSet 同时只有一个 `ChangeSetWriteLease` writer；
- reviewer 使用 read-only checkout 或隔离 worktree；
- retry 可以按 policy 复用 ChangeSet；fallback 必须 fence old writer；
- Task 可以没有 worktree，一个 Run 可以有多个 ChangeSet；
- cleanup 不删除 Project/Task/Room/Run history。

SCM result 不能相信 Agent 文案。core/adapter 必须核验 expected base/HEAD、commit ancestry、branch/worktree cleanliness、PR head SHA、required checks/reviews、write fence、merge result 和 target head。任何半完成状态返回 typed error 与 recovery action。

Git 保存共享、低频、可 review 的 Project/Workflow/Memo/code/doc/test/commit/PR；Room message、draft/scroll、Task rank、heartbeat、lease 和 raw trace 留在 SQLite/trace store。Git 不是 Room database。

## Runtime mapping

| HCTL object | Runtime meaning | Backend mapping |
| --- | --- | --- |
| Repo/Project/Task/Room/Participant | domain/collaboration | none |
| Run | automated boundary | 0..N RuntimeShard |
| RuntimeShard | host + isolation + generation | one backend container/scope |
| Obligation/Seat | logical responsibility/lease | none |
| Attempt | one Harness execution | 0..1 TerminalBundle |
| RoomInvocationRecord | bounded non-Run call | 0..1 InvocationRuntime |
| InvocationRuntime | host + isolation + generation | one container/scope or structured session |
| terminal channel | exact TUI/PTY/shell/log | mux pane、direct PTY、vendor target |

Backend name、pane index、vendor label 和 UI connection ID 只用于物理解析/展示，不能作为 database key 或 Receipt ref。

agentd 的统一 `RuntimeBackend` contract：

```text
create_container(spec)
spawn_attempt(container, invocation)
observe(attempt)
signal(attempt, input)
stop(attempt, reason)
attach_target(attempt, mode)
reconcile(observed, desired)
```

接口预留 `zellij | tmux | direct_pty | vendor_supervisor | remote_provider`。Phase 1 用同一 contract benchmark Zellij 与 tmux，只冻结并交付一个默认 mux backend；pane/session name 不成为 identity，两套 backend 也不能同时维护 execution truth。

## Attach capability taxonomy

`attach` 不是布尔值：

| Capability | 精确定义 | UI verb |
| --- | --- | --- |
| `native_pty_exact` | 同一仍存活 PTY/process、原生 TUI、受权双向 input | Attach Terminal |
| `native_agent_handoff` | 同一 provider conversation 的 native/remote handoff，不保证 same PTY | Hand off / Open Agent |
| `structured_live_inspect` | 实时 structured event/transcript 与受控 follow-up | Inspect Live Session |
| `semantic_resume` | 原 process 可已消失，以 provider session ID 恢复 context | Resume Conversation |
| `replay_only` | 只读 history/provenance | View Replay |

Capability 非互斥，由 live probe 产生。PTY lost 但 provider session 可 resume 时，原 Attempt 仍是 LOST；resume 创建新 Attempt/Invocation，不能伪装成 same-PTY reattach。

## AttachDescriptor 与 input ownership

从 Attempt 或 RoomInvocation 点击 attach 时，agentd 解析精确 logical owner、host、runtime generation 与 target，并签发短期 AttachDescriptor。Run 路径显示 Project/Run/Task/Role/Harness/Attempt/Host/Revision/Lease；RoomInvocation 路径显示 Room/Invocation/Participant/Harness/Host/ChangeSet/Lease。

权限分开：

- `trace.read`；
- `terminal.observe`；
- `terminal.input/takeover`；
- `attempt.control`；
- `secure-input`。

一个 target 可有 0..N observer，但默认至多一个 `TerminalInputLease`，同时拥有 input/resize。Takeover 原子撤销旧 lease，最终由 agentd 校验而非前端按钮。Descriptor expiry 或 generation mismatch 时重新解析；client 不能缓存旧 pane/session 后继续输入。

必须分离 conversation ID、attempt ID、runtime container ID、terminal target ID、connection ID 和 generation/fence。Adopt externally created runtime 需要显式用户动作并验证 cwd/process/owner/capability/generation，不能按名称自动收养。

## Embedded xterm.js 与 external WezTerm

Phase 1 默认 terminal client 是 `@xterm/xterm`，仅负责 emulation、render、selection、keyboard/mouse 和 IME。PTY、process、scrollback/snapshot recovery、input lease 和 runtime identity 属于 agentd/RuntimeBackend。

```text
EmbeddedXtermClient
  ↕ transferable binary MessagePort / ArrayBuffer
trusted Electron preload/main
  ↕ opaque connection_id
agentd TerminalGateway
  ↕ authorized RuntimeBackend channel
Zellij / tmux / direct PTY / vendor runtime
```

数据面与普通 command/query RPC 分离，使用 generation、sequence、ack/credit、有界 buffer 和 snapshot/resync。慢 renderer 可以丢弃 display deltas 并从 trusted snapshot resync，不能阻塞 PTY/control。

Renderer 不可 spawn shell、持有 PTY fd、提交 arbitrary argv/cwd/pane ID、访问 AttachDescriptor secret，或把 xterm buffer 当成 recovery/Receipt/Room history。Link、clipboard、OSC、file-open 经 trusted allowlist/confirmation。

WezTerm 是 optional “Open externally” escape path。Trusted side 通过 inherited pipe/local IPC 启动 single-use、UID/PID-bound、TTL handle 的 `hctl terminal attach` shim；secret/token 不进入 argv 或 shell history。WezTerm 不进入 React dependency graph，不把 internal mux protocol 变成 HCTL ABI。

Detach、关闭 panel 或退出 Workbench 不停止 Attempt。重开后从 logical owner 重新签发 descriptor、取得 snapshot 并继续 live stream。

## Execution Chat Projection

同一 native TUI/Attempt 可以从 provider transcript/hook/tool event 投影成 structured chat-like inspector，并把受支持 input 写回同一 execution owner；不支持的动作退回 terminal。

它必须且只能绑定一个 `attempt_id` 或 `invocation_runtime_id`：

- 不拥有独立 conversation identity；
- input 是针对精确 runtime owner 的 control action，不自动成为 Room message；
- 只有显式 Share to Room 才发布带 provenance 的 summary/ref；
- projection 消失、adapter degradation 或 runtime rebuild 不改变 Room identity。

## Skills 分发与 native session import

Central Skill store、Harness×Skill compatibility、symlink/junction/copy fallback 和 global/repo scope 属于 L1 delivery mechanism；Invocation 是否可用某 Skill、实际 digest 与权限仍由上层 binding 冻结。

Native session import 是 optional adapter：显式导入、保存 provider session ID/path provenance、只让用户选择的摘要/引用进入 ContextBundle，parser 使用 versioned fixtures。它不把 provider history 复制成 Room truth，也不是 Phase 1 success prerequisite。

## Workbench 原生交互

L1 在 Workbench 中表现为：

- Harness picker、installation/auth/health/capability/preflight；
- Attempt/Invocation inspector 与 normalized semantic cards；
- ChangeSet/worktree/diff/test/PR/CI view；
- precise capability verbs：Attach / Inspect / Handoff / Resume / Replay；
- embedded xterm panel、observer/input/takeover state；
- optional Open in WezTerm；
- crash/lost/stale/fenced/reconcile visibility。

用户在 happy path 不需要进入 terminal。Terminal 不是正常 status query，semantic cards 也不改变 domain truth。

## 没有 Workbench 时如何降级

- Attempt/process/runtime 按 agentd/Backend lifecycle 继续；GUI exit 只 detach。
- `hctl status`、structured trace/export 与 `hctl terminal attach <logical-owner>` 使用同一 command/descriptor seam。
- External WezTerm 是 Phase 1 planned high-fidelity fallback；若 exact PTY 不可用，CLI 必须准确提供 inspect/resume/replay，不能降级后仍叫 attach。
- 输入仍需 fresh descriptor、generation 与 TerminalInputLease；shell/mux client 不能通过 pane name 绕过 authority。
- Workbench 恢复后从 agentd registry + Backend observed state + HCTL desired state reconcile，重新签发 descriptor，不复用旧 connection。
- Rich semantic cards、diff grouping 和 cross-layer navigation 可能损失，但 runtime ownership、fence、evidence 与 audit 不变。

## 精选参考

### Anchor：Stably Orca

Stably Orca 的最高信息增益是 worktree-native execution environment：独立 branch/files/agent terminals、daemon-owned PTY、GUI 关闭后 process 继续、warm reattach 与 split/scrollback/focus restore、SSH/remote reconnect、terminal handle fencing、diff/review/ship lifecycle。Native Chat 也明确是 same terminal session 的 experimental structured projection，而不是 Project Room。

HCTL 采用其 terminal ownership、generation/fence/reconnect、worktree/diff/remote interaction 和 failure tests，但不继承：

- Workspace/worktree 作为 Project 或 Task identity；
- workspaceStatus/board 作为 Task lifecycle truth；
- Agent session/terminal handle 作为 durable Run identity；
- OSC/TUI state、worktree comment 或 `worker_done` 作为 semantic completion；
- Native Chat 作为 L4 Room；
- experimental Run/Task/Dispatch 作为 HCTL L2 truth。

版本与证据见 [E-L1-STABLY-ORCA](../references/implementation-evidence.md#e-l1-stably-orca)。

### Focused supporting evidence

- DeepSeek Harness / Cordis：参考 capability definition/provider/consumer seam、reversible in-process registration、reactive dependency 与 model-visible append-only session log；不让 plugin load order 决定 authority，不把 disposer 当外部 effect rollback，也不把 dynamic workflow 当 L2 truth。见 [E-L1-DEEPSEEK-HARNESS](../references/implementation-evidence.md#e-l1-deepseek-harness)。
- OpenCode、Pi、Kimi Code：分别作为 native app-server、strict RPC/embedded SDK、ACP degradation 的固定版本 contract fixtures；Pi/Kimi 进入 evidence bench 不自动扩大 Phase 1 scope。见 [E-L1-HARNESS-ACCESS](../references/implementation-evidence.md#e-l1-harness-access)。
- Termio：Harness manifest、stable session URI、watch/heartbeat/signal 和 schema-versioned session-control；ATP-inspired fixtures，不把 ATP 当行业协议或 RuntimeBackend truth。
- Herdr：server-owned PTY、agent-aware status authority、semantic/raw control 和 exact attach contract；不拥有 HCTL domain identity。
- Codex Remote Feishu：参考 provider workspace/thread 到 managed session 的 attach/route、queue/steer、approval 与 reconnect/degraded state machine；飞书只是 remote control/projection client，不是 Project Room。见 [E-L1-CODEX-REMOTE-FEISHU](../references/implementation-evidence.md#e-l1-codex-remote-feishu)。
- xterm.js：embedded renderer、IME/CJK/a11y/flow control；frontend only。
- WezTerm：mature external terminal/CLI escape path；不嵌入、不提供 HCTL ABI。
- Zellij/tmux：Phase 1 backend candidates，必须同 bench 后只选一个。

Superset、MindFS、Paseo、HAPI、Happy、Moshi、Remux、ServerCC、QuickTUI 和 Redock 只在 capability Radar 中保留各自最独特的 execution 切片，不在正文逐个列功能。详见[实现证据与参考组合](../references/implementation-evidence.md)。

## Failure 与 contract tests

- Harness presence/version/auth/health/capability degradation matrix；
- app-server request/SSE、strict RPC response/event、ACP supported/unsupported method fixtures；
- resolved Harness profile/plugin-set digest frozen；unknown required session event/version fail closed；
- plugin dispose/reload cannot revoke leaked filesystem/network effects or change an admitted Attempt；
- app-server/ACP/PTY/hook event normalization 与 confidence/authority；
- ChangeSet single writer、fallback fence、late write/result rejection；
- exact target、headless process persistence、Backend crash/adopt/reconcile；
- PTY lost vs semantic resume distinction；
- descriptor expiry、generation mismatch、observer/input/takeover 和 old lease revoke；
- xterm alternate screen、mouse、resize、wide glyph、CJK IME、bracketed paste；
- high-throughput backpressure、有界 buffer、snapshot/resync，Room/control RPC 仍响应；
- renderer/Workbench/WezTerm exit 不 stop Attempt；
- stale connection/pane name/renderer-supplied argv cannot bypass trusted resolution；
- Git merge interruption、HEAD/index/PR read-back 与 typed recovery；
- replacing terminal client or RuntimeBackend does not change Project/Task/Run identity or Receipt validity。

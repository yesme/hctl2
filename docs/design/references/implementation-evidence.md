# 实现证据与精选参考组合

> Status: Informative · Research snapshot 2026-08-14<br>
> Parent: [HCTL2 设计规范](../README.md)<br>
> Rule: 本文证明可行性与 sourcing boundary，不定义 HCTL 领域模型或产品路线。

## 引用准入

研究样本不是“覆盖四层越多越好”的竞品矩阵。许多产品同时具有 Chat、Task、Workflow、worktree 和 terminal，但只有它探索最深、对 HCTL 信息增益最大的部分值得进入对应 layer。

引用强度：

- **Anchor**：该层原因/方案的主要思考伙伴；每层最多一个外部 anchor；
- **Selected lineage evidence**：直接前代已经实现并验证的语义切片；说明继承与改写边界，不参与外部 anchor 竞争；
- **Selected behavior evidence**：公开产品行为足以支持一个 interaction/failure contract，但没有可移植源码或许可；
- **Selected implementation evidence**：只支持一个清晰协议、算法、UI primitive 或 failure test；
- **Radar only**：研究过且有局部价值，但不进入 layer 主叙事；
- **Research history only**：只解释设计如何演进，不再影响当前 sourcing。

每个 product effort 默认一个 strongest layer。标准、通用库和 Task provider 单列，不参与 product anchor 竞争。

## 精选组合总览

| Effort | Strongest layer | 强度 | 只保留的独特价值 |
| --- | --- | --- | --- |
| First Tree | L4 | **Anchor** | persistent Chat、Context durability、Human Request、durable Inbox、cross-surface Room |
| Claude Tag | L4 | Selected behavior | shared steerable thread、scope-owned identity/memory、durable collaboration 与 ephemeral runtime 分离 |
| OpenClaw | L4 | Selected | deterministic multi-channel identity/routing、pairing/allowlist 与 delivery degradation |
| Codeg | L3 | **Anchor** | async WorkTask/To-do、Needs You、review/follow-up/preflight、Git truth |
| Hermes Agent | L3 | Selected | durable Task/attempt、atomic claim、heartbeat/reclaim、dependency promotion 与 shared kernel surfaces |
| Multica | L3 | Radar | Issue Board、Inbox、Agent/Runtime separation、Trigger Preview |
| HCTL1 / yesme/hctl | L2 | Selected lineage evidence | Git-native Seat/claim/fence、exact Verdict/quorum、replayable Receipt 与 fail-closed corpus |
| HCTL2 semantic kernel | L2 | **Native anchor** | revision/evidence-bound Run、Seat fallback、quorum、regate、Receipt |
| Conductor OSS | L2 | Selected backend | external worker 与 token/timer/retry/history 的被动机械状态 |
| ZeroClaw | L2 | Selected adjacent | SOP admission、revision-scoped approval/quorum、restore 与 fail-closed policy tests |
| Dagu | L2 | Radar | data-first graph、runner/action/human approval 的选型对照 |
| Stably Orca | L1 | **Anchor** | worktree/terminal/diff/remote、daemon PTY、reattach、generation/fencing |
| DeepSeek Harness | L1 | Selected | capability seam、typed event/effect、model-visible append-only log 与 plugin composition |
| OpenCode | L1 | Selected | OpenAPI + SSE + typed SDK 的 server-first multi-client Harness surface |
| Pi | L1 | Selected | embedded SDK + strict JSONL RPC、steer/follow-up queue contract |
| Kimi Code | L1 | Selected | ACP/native capability matrix 与可验证 degradation |
| Termio | L1 | Selected | Harness manifest、session URI、watch/heartbeat/signal contract |
| Herdr | L1 | Selected | server-owned PTY、agent-aware status、semantic/raw exact control |
| Codex Remote Feishu | L1 | Selected behavior | managed session 的 attach/route、queue/steer、Request 与 reconnect state machine |
| Superset | L1 | Radar | multi-worktree、persistent terminal、Changes/PR/CI execution UX |
| MindFS | L1 | Radar | repo-local session、external session import/sync、light deployment |
| Paseo | L1 | Radar | daemon/client/provider adapter SDK 与 multi-device seam |
| HAPI | L1 | Radar | local native agent ↔ remote structured handoff |
| Happy | L1 | Radar | daemon、E2EE conversation sync、remote spawn/multi-device |
| Moshi | L1 | Radar | mobile terminal、hooks/attention、TUI chat projection |
| Remux | L1 | Radar | SSH + tmux control-mode exact session/window/pane attach |
| ServerCC | L1 | Radar | external takeover、provider session resume、mobile control |
| QuickTUI | L1 | Radar | self-hosted tmux + iOS/iPad/browser terminal surface |
| Redock | L1 | Radar | staged mobile input、CJK/voice、Activity deep link |

这个归类故意不重复。例如 Codeg 的 Composer/ACP 和 Stably Orca 的 experimental orchestration 仍可作为局部源码事实，但不会使它们成为 L4/L2 第二 anchor。Superset/Herdr 虽也有 Project 或 Agent 概念，只在 L1 记录独特 execution evidence。HCTL1 是内在谱系而非外部 anchor；它只证明 HCTL2 L2 语义内核的前代切片。

<a id="e-l4-first-tree"></a>
## E-L4-FIRST-TREE · First Tree

### 为什么只放 L4

First Tree 的产品闭环是 `Team → Agent → persistent Chat → Context Tree → human/SCM outcome`。它证明 context/chat-first 可以承载完整协作连续性，但 `task chat` 只是 Chat 创建模式，没有 HCTL 的 first-class Task；项目明确不是 orchestration framework，唯一 TUI path 也是内部 detached tmux driver、没有公开 exact attach API。因此它只作为 L4 anchor。

### 审计基线

两个基线必须分开陈述：

| Baseline | Status | 可支持的结论 |
| --- | --- | --- |
| [`v0.5.20` / `19e66032`](https://github.com/agent-team-foundation/first-tree/commit/19e66032af7f9f482168c350fe0b3998599388f3) · 2026-08-11 | Released | Context Tree、Chat/typed mention、Human Request、Inbox、GitHub/GitLab、provider runtime/recovery |
| [`main@92871401`](https://github.com/agent-team-foundation/first-tree/commit/928714016cc3bb82f0f3da0fab87044d61b70883) · 2026-08-14 06:57 UTC | Unreleased audit snapshot；release +35 commits | Feishu Agent Channel、new runtime authority split、ReplayFence/Reset direction |

[Release-to-snapshot comparison](https://github.com/agent-team-foundation/first-tree/compare/19e66032af7f9f482168c350fe0b3998599388f3...928714016cc3bb82f0f3da0fab87044d61b70883)。Feishu QA file 是验收合同，不是公开 PASS report；未发布能力不能写成 v0.5.20 feature。

### 源码审计结论

| Surface | 已验证 | 缺口 | HCTL 采用 |
| --- | --- | --- | --- |
| Product/objects | persistent Chat + Context Tree + human/SCM outcome | 无 HCTL Repo/Project/Task/WorkflowRevision/Run/Seat | 证明 L4 continuity；不移植 Team/Agent/Chat schema |
| Context Tree | Decision Test + Durability Test；code 默认 implementation truth；exact snapshot、route receipt、isolated worktree、re-preflight、verify、PR/MR review | Tree 绑定 donor Team/Reviewer/forge model | 改编为 Memo→Project knowledge admission；不新增 `ContextTree` object |
| Typed mention/Inbox | Web 发送 stable participant ID，server 验 membership/active；message+recipient fan-out 同事务；pending/delivered/acked、`SKIP LOCKED`、per-chat prefix ACK、reconnect recovery | CLI/API 仍有 name compatibility；普通 send 无 caller idempotency；message 可原地 edit 且无 revision/history/tombstone | stable identity、ACK custody、transaction tests；HCTL 补 command ID、append-only correction/tombstone、frozen MentionRef |
| Human Request | 精确 human；只有 explicit `metadata.resolves` closes；row lock + prior-resolution scan；FIFO/draft/attachment/Submit/Skip/mobile/tenant QA | 仍是 message format；不是 quorum/gate | 转为 first-class Request reducer/tests；不替代 Seat/quorum |
| GitHub integration | entity↔Chat binding、HMAC、delivery ID dedupe、follow/unfollow、publication/reauth | claim 早于 all Chat delivery、top-level hides per-chat failure、无 ordering watermark、部分 actor name mapping | 借 binding/normalization tests；HCTL 保留 inbox/outbox/watermark/reconcile/principal binding |
| Feishu main only | bot/chat binding、exact mention、echo suppression、author snapshot、event+message double dedupe、attachment hydration、lease+epoch | 1 Chat↔1 Feishu、Web read-only、无 edit/delete、ACK 可早于 canonical commit、final transaction 无 fence、outbound 无 DeliveryReceipt lifecycle | 借 conversion/dedupe/lease/QA；HCTL 补 multi-surface、RoomEvent+outbox atomicity、commit fence、receipt/reconcile |
| Provider runtime/Skills | start/resume/inject/suspend/shutdown、ACK/retry/recovery/persistence、catalog/capability、Skills lock/journal/digest/version fence、daemon supervisor；main 增 ReplayFence/reset authority | private client 强耦合 Hub/Chat；API churn；retry 仅同 provider/session；protocol receipt ≠ semantic Receipt | 选择性借 contract/failure/replay/Skills tests，不 package-link，不作为 Seat fallback |
| Session/terminal | provider session；多数 SDK/app-server/subprocess；internal tmux paste/capture | TUI option disabled for new selection；无 public attach/stable PTY target | 不作为 L1 donor |
| Workflow | cron 到点生成普通 addressed message | 无 run history、DAG、WorkflowRevision、Seat、candidate、quorum、regate、revision Receipt | 不作为 L2 donor |

Sourcing decision：**Assess / selective source port**，Apache-2.0。可直接改编 Context Policy/Double Test、Need You journey、Inbox ACK custody、Feishu/GitHub cross-surface QA；可选择性移植 pure schema/content conversion、binding/lease update、prefix ACK、ReplayFence/reset generation 和 managed-Skills transaction discipline。不能整仓 fork 或采用 central PostgreSQL/cloud truth。

主要源码：

- [Repository](https://github.com/agent-team-foundation/first-tree)；[v0.5.20](https://github.com/agent-team-foundation/first-tree/releases/tag/v0.5.20)
- [Architecture rules](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/AGENTS.md)；[Quickstart](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/docs/quickstart.md)
- [Context policy](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/packages/client/src/runtime/assets/context-tree-policy.md)；[Context schema](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/packages/shared/src/schemas/context-tree.ts)
- [Chat schema](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/packages/shared/src/schemas/chat.ts)；[Message service](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/packages/server/src/services/chat/message.ts)；[Inbox service](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/packages/server/src/services/chat/inbox.ts)
- [Need You QA](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/packages/qa/cases/cross-surface/need-you-request-review-journey.md)；[GitHub webhook processing](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/packages/server/src/services/scm/shared/webhook-processing.ts)
- [Provider contract](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/packages/client/src/providers/README.md)；[Runtime schema](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/packages/shared/src/schemas/runtime-provider.ts)；[Session-control CLI](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/apps/cli/src/commands/agent/session/control.ts)
- [Internal tmux TUI driver](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/packages/client/src/providers/claude/tui/tmux-session.ts)；[Cron table](https://github.com/agent-team-foundation/first-tree/blob/v0.5.20/packages/server/src/db/schema/cron-jobs.ts)
- Main snapshot: [bot binding](https://github.com/agent-team-foundation/first-tree/blob/928714016cc3bb82f0f3da0fab87044d61b70883/packages/server/src/db/schema/im-bot-bindings.ts)、[chat binding](https://github.com/agent-team-foundation/first-tree/blob/928714016cc3bb82f0f3da0fab87044d61b70883/packages/server/src/db/schema/im-chat-bindings.ts)、[inbound](https://github.com/agent-team-foundation/first-tree/blob/928714016cc3bb82f0f3da0fab87044d61b70883/packages/server/src/services/integrations/feishu/inbound.ts)、[manager](https://github.com/agent-team-foundation/first-tree/blob/928714016cc3bb82f0f3da0fab87044d61b70883/packages/server/src/services/integrations/feishu/manager.ts)、[Feishu QA](https://github.com/agent-team-foundation/first-tree/blob/928714016cc3bb82f0f3da0fab87044d61b70883/packages/qa/cases/cross-surface/feishu-agent-channel.md)

<a id="e-l4-claude-tag"></a>
## E-L4-CLAUDE-TAG · Claude Tag

Claude Tag 是 L4 selected behavior evidence，不与 First Tree 的 anchor 地位竞争。它最独特的产品证据是：一个 Slack thread 是多人可见、可由 channel member 继续或中途 steer 的 working session；thread/context 持久，而 hosted sandbox 可以回收后重建；Agent 使用 channel-scoped service identity 和 access/memory scope，而不是冒充触发者。Checklist、scheduled routine、channel watch 和 repository event 则展示了低噪声的异步协作投影。

HCTL 采用 durable Room 与 ephemeral runtime 分离、shared steering、Agent 独立身份与 scope-bound access，以及 checklist/routine 只作 projection/trigger 的思路。不把 Slack channel/thread 映射为 Project/Room，不让 checklist、memory 或 routine 成为 Task/Run/knowledge truth，也不让 channel membership 绕过 HCTL permission/Gate。它是 proprietary public beta，只能作 behavior evidence，不能作源码 donor 或 L1 runtime 方案。

基线按公开产品资料日期固定为 2026-06-23 Public Beta：[announcement](https://www.anthropic.com/news/introducing-claude-tag)、[how it works](https://claude.com/docs/claude-tag/concepts/how-it-works)、[agent identity](https://claude.com/docs/claude-tag/concepts/agent-identity)、[routines](https://claude.com/docs/claude-tag/users/proactivity)、[memory](https://claude.com/docs/claude-tag/users/memory)。

<a id="e-l4-openclaw"></a>
## E-L4-OPENCLAW · OpenClaw

OpenClaw 最值得放在 L4 external-channel edge：它把 account/peer/thread 归一成 deterministic routing key，支持 exact binding、thread inheritance、DM scope、pairing/allowlist、ambient room events、bot-loop protection 和 capability-aware delivery。它证明无 Workbench 的 Chat surface 需要 stable external identity、确定性路由与 per-channel degradation，而不是让 model 猜 channel 或按 display name 分发。

HCTL 只借 adapter/routing/pairing/loop/degradation tests；不把 OpenClaw channel/session/workspace/agent 映射为 Project/Room/Task/Run，不让 ambient chatter 自动成为 canonical Context，也不让 Gateway、cron 或 delegation 成为 L2 truth。固定 [`v2026.7.1-2 / 0790d9f5`](https://github.com/openclaw/openclaw/tree/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c)（MIT）；证据见 [channel routing](https://github.com/openclaw/openclaw/blob/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c/docs/channels/channel-routing.md)、[README](https://github.com/openclaw/openclaw/blob/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c/README.md) 与 [license](https://github.com/openclaw/openclaw/blob/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c/LICENSE)。

<a id="e-l3-codeg"></a>
## E-L3-CODEG · Codeg

### 为什么只放 L3

Codeg v0.24 的 To-dos/WorkTask 是独立 async queue，不是 terminal/worktree card：有 To do/In progress/Needs you/Done Board、ordering/schedule/concurrency、setup/running/awaiting-input/review/merge state compression、diff review、preflight、timeline、Rework/Keep going/Ask/Double-check 和 merge 后 Git truth recheck。`worktree_folder_id` 与 `conversation_id` 可以为空，说明 Task identity 不依赖 runtime。

Codeg 也有 Composer、ACP、Skills、worktree、Git、diff 和 embedded terminal；这些是有价值的 component evidence，但其最独特 product insight 仍是 L3。Stably Orca 在 persistent PTY/remote/ownership 上更深，因此不交换二者位置。

采用：Task/Needs You/review/follow-up/Git-truth interaction 与 tests；可选移植 Apache-2.0 Composer/ACP/event-card bounded files。

不采用：Conversation=Room、To-do=HCTL Task schema、fixed pipeline=Workflow、Task permanent worktree/session binding、drag-to-run、Agent `task_complete`/Done/Git landed=semantic completion、lead LLM routing=control truth。

主要证据：

- [Repository](https://github.com/xintaofei/codeg)；[v0.24.0 release](https://github.com/xintaofei/codeg/releases/tag/v0.24.0)
- [To-dos guide](https://docs.codeg.app/guide/tasks)；[Workspace guide](https://docs.codeg.app/guide/workspace)
- [v0.24 Board mapping](https://github.com/xintaofei/codeg/blob/v0.24.0/src/components/tasks/board-columns.ts#L4-L58)
- [v0.24 WorkTask engine](https://github.com/xintaofei/codeg/blob/v0.24.0/src-tauri/src/work_task/engine.rs#L1-L17)；[model](https://github.com/xintaofei/codeg/blob/v0.24.0/src-tauri/src/models/work_task.rs#L6-L58)
- [Composer](https://github.com/xintaofei/codeg/blob/v0.24.0/src/components/chat/composer/rich-composer.tsx)；[ACP registry](https://github.com/xintaofei/codeg/blob/v0.24.0/src-tauri/src/acp/custom_registry.rs)；[delegation schema](https://github.com/xintaofei/codeg/blob/v0.24.0/src-tauri/src/acp/delegation/tool_schema.json)

<a id="e-l3-hermes-agent"></a>
## E-L3-HERMES-AGENT · Hermes Agent

Hermes Agent 的独特增量是 agent-operated durable Task/attempt protocol：SQLite Board 保存 Task、Run/attempt、dependency、comment 与 workspace；dispatcher 做 atomic claim、heartbeat/stale/crashed-worker reclaim、dependency promotion 和 protocol-violation auto-block；CLI、Chat slash command 与 dashboard 共用同一 kernel。这是 Codeg 偏 product/review UX 之外有价值的 L3 recovery 与 no-Workbench implementation evidence。

HCTL 借 Task/Attempt 分离、claim/reclaim、durable comment 和 shared command kernel；不采用 Board=Project、profile/memory=Participant/Project、model 自报完成=Receipt、single-host dispatcher=L2 truth 或 LLM goal judge=semantic completion。固定 [`v2026.8.13 / f80f453a`](https://github.com/NousResearch/hermes-agent/tree/f80f453ae0679347e38abc917c7f94f717bf96c5)（release name `v0.20.1`，MIT）；证据见 [Kanban guide](https://github.com/NousResearch/hermes-agent/blob/f80f453ae0679347e38abc917c7f94f717bf96c5/website/docs/user-guide/features/kanban.md)、[README](https://github.com/NousResearch/hermes-agent/blob/f80f453ae0679347e38abc917c7f94f717bf96c5/README.md) 与 [license](https://github.com/NousResearch/hermes-agent/blob/f80f453ae0679347e38abc917c7f94f717bf96c5/LICENSE)。

<a id="e-l2-hctl1"></a>
## E-L2-HCTL1 · HCTL1 / yesme/hctl

HCTL1 是 HCTL2 L2 语义内核的直接前身与可执行谱系证据，不是外部 donor，也不与 HCTL2-native anchor 竞争。固定 [`main@3148042c`](https://github.com/yesme/hctl/tree/3148042cb2faf8df0dc8be92710b9468c8618516)（2026-07-28），Apache-2.0；该仓没有 tag/release，README 将 P1 kernel 标为已进入 main，P2/P3 仍为规划。

其独特证据是 daemon/DB-free 的 Git semantic kernel：per-seat append-only event ref、local/remote CAS、level-triggered reconcile、fail-closed incomplete facts、Obligation/CLAIM 与 claim-OID fencing、exact `{base, head}` Verdict、quorum，以及携带 fact tips、可无时钟重放的 squash merge Receipt。规范之外还有覆盖 stale gate、authority、race、JCS identity、composite quorum、late finding、regate carry 和 bootstrap cutover 的 executable corpus。

HCTL2 继承 revision/evidence、claim/fencing、quorum、Receipt 与 reconcile 思路，但不原样继承对象和事实源：

- HCTL1 `Seat = harness × model` collaboration identity；HCTL2 Seat 是 Obligation 内 logical executor/voter slot，下挂 `0..N` Attempts；
- HCTL1 Obligation 来自 static assignment 的 author/gate/merge；HCTL2 Obligation 对应 Conductor external-task execution；
- HCTL1 per-seat refs、PR 与 squash Receipt 是全域协调 truth；HCTL2 将 operational governance 放入 SQLite/control，以 Git 保存共享低频定义与 evidence，并由 Conductor 保存机械 workflow 位置；
- HCTL1 reclaim 不等于 candidate fallback，也没有 Project Room、Task Board、WorkflowRevision、Run、Attempt、process/PTY 或 provider sync；
- single-human trust、unique merge coordinator/capacity=1 和 PR-as-collaboration-atom 只适用于其窄 profile。

主要证据：

- [README scope](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/README.md#L7-L27)；[METHOD fact/Seat/claim](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/METHOD.md#L27-L114)；[Gate/carry/merge](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/METHOD.md#L108-L182)
- [Derive engine](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/internal/derive/derive.go#L47-L124)；[CAS/pending recovery](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/internal/store/store.go#L15-L191)；[Receipt replay](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/internal/receipt/receipt.go#L14-L187)
- [Executable corpus](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/tests/corpus/README.md#L1-L53)；[Apache-2.0 license](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/LICENSE)

<a id="e-l2-conductor"></a>
## E-L2-CONDUCTOR · Conductor mechanical backend

L2 的 anchor 是 HCTL2-native semantic kernel；HCTL1 是其 lineage evidence，而不是完整 HCTL2 Workflow donor。Conductor 仅证明 external worker、READY/wait/timer/retry/history 可以与 actual effect execution 分离，采用方式为 **selected dependency behind WorkflowEngineAdapter**。

- [Conductor OSS](https://github.com/conductor-oss/conductor)
- [Concepts](https://docs.conductor-oss.org/devguide/concepts/index.html)
- [Deployment](https://docs.conductor-oss.org/devguide/running/deploy.html)

Conductor 不选择 Harness、不创建 Seat/Attempt、不解释 semantic reject、不算 HCTL quorum、不签 Receipt、不直接写 Git/provider。Dagu 的 [data-first workflow/runner](https://github.com/dagu-org/dagu) 只作为 runner ownership 对照，不是 Phase 1 backend。

Stably Orca experimental orchestration 是 adjacent evidence：存在 Run/Task/Dispatch/Message/Decision Gate/DAG，但 [official docs](https://www.onorca.dev/docs/cli/orchestration) 明确为 experimental，Run 更接近 durable namespace/coordinator inbox，不负责 scheduling/placement；因此不推翻 HCTL L2 missing-piece 判断，也不成为第二 workflow truth。

<a id="e-l2-zeroclaw"></a>
## E-L2-ZEROCLAW · ZeroClaw SOP

ZeroClaw 不是 HCTL Workflow donor，但其 SOP engine 是罕见的 L2 adjacent implementation evidence：per-SOP admission 支持 parallel/hold/coalesce/drop，run 可持久化并在 restart 后恢复；HITL/checkpoint、authenticated approval group/quorum、append-only approval audit、revision-scoped stale-prompt refusal、amend/revise、step tool scope 与 retry/goto 形成了可借的 gate/admission failure corpus。

HCTL 只借 revision-bound human decision、admission/backpressure 和 fail-closed policy tests；不采用 SOP=WorkflowRevision、event trigger=Start authorization、agent `sop_advance`=Verdict、tool receipt=HCTL Receipt 或 ZeroClaw run DB=domain truth。固定 [`v0.8.4 / a56c345d`](https://github.com/zeroclaw-labs/zeroclaw/tree/a56c345d51dd8ab562e9351e0d4ab83f6a741db9)（MIT OR Apache-2.0）；[syntax](https://github.com/zeroclaw-labs/zeroclaw/blob/a56c345d51dd8ab562e9351e0d4ab83f6a741db9/docs/book/src/sop/syntax.md) 与 [runtime contract](https://github.com/zeroclaw-labs/zeroclaw/blob/a56c345d51dd8ab562e9351e0d4ab83f6a741db9/docs/book/src/sop/how-it-works.md) 对 persistence default 仍有冲突，初始化失败也会降级到 process-local memory，恰好说明它不能承担 HCTL authority。

<a id="e-l1-stably-orca"></a>
## E-L1-STABLY-ORCA · Stably Orca

### 为什么只放 L1

Stably Orca 的核心模型是 worktree-native execution environment：每个 worktree 有独立 branch/files/agent terminals；daemon owns PTY，GUI 关闭 process 继续；restart 可 warm reattach 并恢复 split/scrollback/focus；SSH/remote 支持 reconnect；terminal handle 具有 runtime/generation boundary。Native Chat 是 same terminal session 上的 experimental structured projection。

它虽有 Workspace Kanban 和 provider task entry，但 board fact 是 Worktree/Workspace status，后者用于 manual sidebar organization；external issue 主要用于创建/open worktree。因此 L3 只是 execution fleet projection，不如 Codeg 的 independent Task contract。

采用：terminal ownership、generation/fence/reconnect、worktree/diff/remote UX 和 failure tests。MIT，固定研究基线 `09ec516ae50b7b83fa65343d9ad96159e3fe71fc`。

不采用：Workspace/worktree=Project/Task、workspaceStatus=Task lifecycle、session/terminal handle=Run identity、OSC/worktree comment/`worker_done`=completion、Native Chat=Room、experimental Run 与 HCTL Run dual truth。

主要证据：

- [Repository](https://github.com/stablyai/orca)；[research baseline](https://github.com/stablyai/orca/tree/09ec516ae50b7b83fa65343d9ad96159e3fe71fc)
- [Worktrees](https://www.onorca.dev/docs/model/worktrees)；[Agents & sessions](https://www.onorca.dev/docs/model/agents-sessions)；[Session restore](https://www.onorca.dev/docs/model/session-restore)
- [Terminal](https://www.onorca.dev/docs/terminal)；[Native Chat](https://www.onorca.dev/docs/agents/native-chat)
- [Pinned CLI contract](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/skill-guides/orca-cli.md#L93-L205)
- [Pinned workspaceStatus definition](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/shared/types.ts#L684-L685)
- [Pinned orchestration ownership](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/skill-guides/orchestration.md#L102-L181)

<a id="e-l1-deepseek-harness"></a>
## E-L1-DEEPSEEK-HARNESS · DeepSeek Harness / Cordis

DeepSeek Harness 的最高信息增益是可运行时组合的 Harness capability graph，补足 Stably Orca 偏 worktree/PTY/attach 的视角：Service Definition/Provider/Consumer seam、显式 plugin dependency、带 disposer 的进程内 registration、reactive dependency，以及 append-only typed session log。其关键不变量是 model-visible input 必须可由日志重建；crash mid-turn 追加 interrupted closure，未知 required event/version 则 fail closed。

HCTL 可采用 capability seam、lifecycle/failure tests、model-visible logging，以及把 resolved profile/plugin-set digest 冻结进 Attempt/RunManifest 的做法。不能把 “Everything is a Plugin” 放大成公共领域模型：hook priority/load order 不得决定 command authority，in-flight Attempt 不接受 HMR 改义；disposer 也只撤销可信进程内 registration，不能回滚已经外泄的 file/network effect，后者仍需 idempotency、outbox、fence 与 Receipt。其 model-written workflow 尚缺 frozen revision、durable resume、Gate 和 semantic Receipt，不是 L2 donor。

固定 [`master@47f94385`](https://github.com/deepseek-ai/deepseek-harness/tree/47f943859bef60e4160492346772ded9b24f765a)（2026-08-13，`0.1.0-rc.5`，MIT）。官方明确标为 developer preview、允许 breaking change；[`BENCHMARK.md`](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/BENCHMARK.md) 只有运行方法，没有公开结果，因此只作 Selected implementation evidence。

Cordis [active-revision preprint](https://github.com/cordiverse/paper/tree/948a07b369c62adb3b12e102458be5c18dfb69b9)（Draft v8，2026-08-13，无 repository license）为 reversible effects/reactive coeffects 提供 theory，不证明 inverse 对外部 effect 正确，也不证明 sandbox、兼容性或性能。[architecture](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/architecture.md)、[Cordis primer](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/cordis-primer.md)、[session](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/subsystems/session.md)、[persistence](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/subsystems/persistence.md) 是 primary implementation evidence；[外部评论](https://mp.weixin.qq.com/s/O3A4RpQM4jZz_XkDFvORyQ) 未实际运行项目，只作为 plugin interference、load-order 与 version-conflict 的 secondary review questions。

<a id="e-l1-harness-access"></a>
## E-L1-HARNESS-ACCESS · OpenCode、Pi 与 Kimi Code

三者只作为 L1 Harness access implementations，分别代表 native app-server、language-neutral RPC/embedded SDK 与标准协议的 capability degradation；不因自身也有 Project、Session、Todo、subagent 或 Plan 概念而进入 L4/L3/L2。

| Harness baseline | 只采用的 contract | 明确边界 |
| --- | --- | --- |
| [OpenCode `v1.18.18 / 31406ccc`](https://github.com/anomalyco/opencode/tree/31406ccc51b4bd2a4e1e086b2bcaa5f7f804f26d) · MIT | OpenAPI 3.1 + SSE + generated typed SDK；health/version、session/control/diff/permission 的 server-first multi-client surface | native HTTP API 不是 universal standard；server event/session completion 不签 HCTL Verdict/Receipt |
| [Pi `v0.84.1 / 53fa77cc`](https://github.com/earendil-works/pi/tree/53fa77ccd8a279eb87e92294ef3687b03ff80112) · MIT | embedded `AgentSession` + strict LF-delimited JSONL RPC；correlated response 与 async event 分离；`steer`、`follow_up`、`abort` queue semantics | Pi RPC/session/tree 不是 HCTL wire/Room/Task/Run；local trust boundary 不是 sandbox |
| [Kimi Code `0.36.0 / b6144f94`](https://github.com/MoonshotAI/kimi-code/tree/b6144f94ea6b22455a4e750d1750d220987e7bc2) · MIT | 明示 supported/unsupported 的 ACP method matrix，结合 stream-json、native server 与 hooks 验证 per-binding degradation | “支持 ACP”不代表 capability 全等；fail-open hook 不承担 Gate/security/completion authority |

采用方式是把 request acceptance 与 execution outcome 分离，按 binding probe capability，保留 explicit unsupported methods，并将固定版本协议样本变成 adapter contract corpus。OpenCode 是 Phase 1 target；Pi 与 Kimi Code 进入 evidence bench 不自动扩大 Phase 1 Harness support。

主要证据：OpenCode [server](https://github.com/anomalyco/opencode/blob/31406ccc51b4bd2a4e1e086b2bcaa5f7f804f26d/packages/web/src/content/docs/server.mdx) / [SDK](https://github.com/anomalyco/opencode/blob/31406ccc51b4bd2a4e1e086b2bcaa5f7f804f26d/packages/web/src/content/docs/sdk.mdx)；Pi [RPC](https://github.com/earendil-works/pi/blob/53fa77ccd8a279eb87e92294ef3687b03ff80112/packages/coding-agent/docs/rpc.md) / [SDK](https://github.com/earendil-works/pi/blob/53fa77ccd8a279eb87e92294ef3687b03ff80112/packages/coding-agent/docs/sdk.md)；Kimi Code [ACP matrix](https://github.com/MoonshotAI/kimi-code/blob/b6144f94ea6b22455a4e750d1750d220987e7bc2/docs/en/reference/kimi-acp.md) / [server API](https://github.com/MoonshotAI/kimi-code/blob/b6144f94ea6b22455a4e750d1750d220987e7bc2/docs/en/reference/server-api.md) / [hook boundary](https://github.com/MoonshotAI/kimi-code/blob/b6144f94ea6b22455a4e750d1750d220987e7bc2/docs/en/customization/hooks.md)。

<a id="e-l1-codex-remote-feishu"></a>
## E-L1-CODEX-REMOTE-FEISHU · Codex Remote Feishu

### 为什么只放 L1

Codex Remote Feishu 没有 canonical Project Room；其最高信息增益是把 provider workspace/thread 接入 compatible managed session，并把 attach/detach、route freeze、queued input/steer、approval card、restart/reconnect、transport-degraded 和 connection epoch 做成显式 execution-control 状态机。Feishu 是该 runtime 的 remote control/projection client，而不是 donor 的 L4 truth。

HCTL 只借它的 managed-session takeover/recovery behavior 与 failure matrix：Feishu Chat 不能映射为 Room，provider workspace/thread 不能映射为 Project/Task/Run，`command_ack` 不是 semantic Receipt，它也没有 exact PTY contract。ordinary inbound 在进入 gateway-local FIFO 后即可 ACK，不等 canonical durable commit；未投递 replay 仍有进程内状态，因此不能据此宣称 durable Room bridge 已闭环。

固定 released [`v2.0.0 / b2091ffe`](https://github.com/kxn/codex-remote-feishu/tree/b2091ffee3330a94703b78a8a6b7b1876e667c65)（2026-08-10）。该基线没有 `LICENSE`、`COPYING` 或 `NOTICE`，GitHub API 也未识别 repository license；在获得明确授权前只能作 **Behavior reference / Adapt ideas only**，不得 port 源码或文档文本。

主要证据：[release](https://github.com/kxn/codex-remote-feishu/releases/tag/v2.0.0)、[architecture](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/general/architecture.md)、[relay protocol](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/general/relay-protocol-spec.md)、[remote surface state machine](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/general/remote-surface-state-machine.md)、[backpressure/epoch/degraded](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/implemented/relay-backpressure-hardening-design.md)、[request/approval](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/implemented/feishu-request-approval-design.md)。

## L4 supporting evidence

| Evidence | Unique slice | Boundary |
| --- | --- | --- |
| [assistant-ui](https://www.assistant-ui.com/docs/api-reference/primitives/message) | scoped Message/MessagePart/action renderer | 不采用 Thread/runtime/store/composer/cloud/queue |
| [virtua](https://github.com/inokawa/virtua) | dynamic-height React viewport | 不拥有 Room order/cursor/follow policy |
| [Rocket.Chat](https://github.com/RocketChat/Rocket.Chat/tree/develop/apps/meteor/client/views/room/MessageList)、[Mattermost](https://github.com/mattermost/mattermost/tree/master/webapp/channels/src/components/dynamic_virtualized_list)、[Zulip](https://github.com/zulip/zulip/blob/main/docs/subsystems/unread_messages.md) | prepend/around-message/unread/dynamic-height/a11y tests | 合并为 behavior evidence；不采用 backend/domain |

Tiptap/ProseMirror 是 L4 selected Composer primitive，不是 product effort：[custom extensions](https://tiptap.dev/docs/editor/extensions/custom-extensions)、[React node views](https://tiptap.dev/docs/editor/extensions/custom-extensions/node-views/react)。

## L3 provider 与 Radar

Linear 和 GitHub 是 external field authority/provider-native fallback，不是 HCTL Task model：

- Linear：[GraphQL](https://linear.app/developers/graphql)、[webhooks](https://linear.app/developers/webhooks)、[rate limits](https://linear.app/developers/rate-limiting)、[pagination](https://linear.app/developers/pagination)
- GitHub：[Projects API guide](https://docs.github.com/en/issues/planning-and-tracking-with-projects/automating-your-project/using-the-api-to-manage-projects)、[GraphQL reference](https://docs.github.com/en/graphql/reference/projects)、[webhooks](https://docs.github.com/en/webhooks/webhook-events-and-payloads)、[REST best practices](https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api)
- React Aria：[Kanban example](https://react-aria.adobe.com/examples/kanban)、[Drag and Drop](https://react-aria.adobe.com/dnd) — UI primitive only。

[Multica](https://multica.ai/docs/concepts) 提供 Issue Board、Inbox、Agent/Runtime separation 和 Trigger Preview UX；其余已被 Codeg/HCTL 覆盖，且 LLM coordinator 不作 control truth，因此 Radar only。

## L1 selected evidence 与 capability Radar

### Selected implementation evidence

| Evidence | Unique slice | Boundary |
| --- | --- | --- |
| [Termio `d1fdac8…`](https://github.com/termio-sh/termio/tree/d1fdac84046805d4056e082f982e6beb6072b61c) / [ATP](https://www.termio.sh/docs/atp) / [session control](https://www.termio.sh/docs/session-control) | manifest、stable session URI、watch/heartbeat/signal、schema-versioned control | MIT；ATP 不是 HCTL/行业 wire standard；不作 cross-platform Backend truth |
| [Herdr](https://herdr.dev/docs/concepts/) | server-owned PTY、agent-aware status、semantic/raw exact control | Apache-2.0；不拥有 HCTL domain identity |
| [xterm.js](https://github.com/xtermjs/xterm.js/) | embedded terminal renderer、CJK/IME/a11y/flow control | MIT frontend only；不拥有 PTY/process/session |
| [WezTerm](https://wezterm.org/cli/cli/index.html) | mature cross-platform external terminal/CLI | MIT；不嵌入、不以 mux protocol 作 ABI |
| [Zellij](https://zellij.dev/documentation/programmatic-control.html) / tmux | real mux/runtime candidates | 同 contract bench 后 Phase 1 只选一个；pane name 不作 identity |

### Radar-only product efforts

| Effort | 只保留的独特 evidence | Reuse boundary |
| --- | --- | --- |
| [Superset](https://docs.superset.sh/superset-model) | multi-worktree、persistent terminal、Changes/PR/CI integrated execution UX | Behavior only；Project/Workspace 不映射 HCTL |
| [MindFS](https://github.com/a9gent/mindfs) | repo-local session、external session import/sync | AGPL；protocol/behavior reference，Task Board 不定义 L3 |
| [Paseo](https://github.com/getpaseo/paseo) | daemon/client/provider adapter、public SDK、multi-device seam | AGPL；Phase 2 architecture reference |
| [HAPI](https://github.com/tiann/hapi) | native local agent ↔ remote structured handoff | AGPL；不是 exact PTY、Task/Workflow backend |
| [Happy](https://github.com/slopus/happy) | daemon、E2EE sync、remote spawn、multi-device | MIT；Phase 2 watch，not Phase 1 truth |
| [Moshi](https://getmoshi.app/docs/introduction) | mobile terminal、hooks/attention、TUI chat projection | Closed source；UX/interoperability only |
| [Remux](https://github.com/h3nock/remux) | SSH + tmux control-mode exact session/window/pane | MIT；不引入第二 domain state |
| [ServerCC](https://servercc.app/docs/sessions) | external takeover、vendor resume、mobile control | Closed source；identity/handoff product evidence |
| [QuickTUI](https://quicktui.ai/) | self-hosted tmux + mobile/browser terminal | App closed source；public repo only distribution evidence |
| [Redock](https://redock.dev/) | staged input、CJK/voice、Activity deep link | Closed source UX only |

## 标准与通用库，不属于 product anchor

- [Agent Client Protocol](https://agentclientprotocol.com/protocol/v1/overview) / [Rust SDK](https://github.com/agentclientprotocol/rust-sdk)：L1 Harness access standard。
- [Agent Skills](https://agentskills.io/specification)：L4 Expertise selection + L1 delivery/binding；Skill 是 guidance，不是 Gate。
- [MCP Resources](https://modelcontextprotocol.io/specification/2026-07-28/server/resources) / [Prompts](https://modelcontextprotocol.io/specification/2026-07-28/server/prompts)：Context/tool transport，不定义 Project/Task authority。
- [React Flow](https://reactflow.dev/) / [Dagre](https://github.com/dagrejs/dagre)：L2 read-only visualization/layout。
- [Electron security](https://www.electronjs.org/docs/latest/tutorial/security) / [MessagePorts](https://www.electronjs.org/docs/latest/tutorial/message-ports)：cross-layer trusted UI/data transport。

## Research-history only

Jira、BPMN、Duroxide 和 cmux 只保留在[设计演进记录](./decision-history.md)：当前没有 pin 完整、差异化且计划采用的 contract evidence，不进入四层正文或 sourcing table。

## Reuse decision vocabulary

所有 evidence 最终只落到：`Adopt dependency / Port bounded component / Adapt protocol / Behavior reference / Defer`。不得给整个产品一个“取代 HCTL”的总分，也不得把 donor 的 Session、Conversation、Project、Task、Run 名称或 internal database 带入 HCTL public schema。

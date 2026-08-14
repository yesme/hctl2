# L4 · Intent & Collaboration — Project Room

> Status: Normative · Draft v0.7.0<br>
> Primary reference: First Tree<br>
> Parent: [四层设计规范](../README.md)

## 这一层为什么存在

L4 回答：**我们要解决什么、为什么、依据是什么，以及哪些讨论已经足够稳定，可以成为承诺。**

Coding Harness 的 session、terminal 和 worktree 都会结束或被替换；Project 的目标、论证、参与者关系、来源和未决问题却必须继续存在。Project Room 因此是进入 Project 后的默认推进界面，也是多 Harness 消失后仍可恢复的协作身份。

Room-mediated 不等于所有工作都必须聊天。Task Board、Run View、Inspector 与 terminal 有自己的交互；Room 的特殊地位是承载 shaping continuity，而不是承载所有机械执行事件。

## 本层负责什么

L4 负责：

- Repo Room、Project Room、Scoped Room 的 identity 与 lifecycle；
- durable message/event、typed reference、source link 与 timeline projection；
- deterministic `@`、collaboration Recipe、Context 和 Skill binding；
- bounded RoomInvocation 与 Quick Compare；
- Request 的讨论、claim、resolution 与 Attention projection；
- 从 discussion 提炼 Task/Artifact/Workflow/Memo proposal；
- 未来外部 Chat surface 与 canonical Room 的可靠 bridge seam。

L4 不负责：

- 决定 Task 是否已承诺或完成；
- 冻结 Workflow、推进 token、retry/fallback 或 Gate；
- 拥有 process、worktree、PTY 或 provider session；
- 把普通聊天、reaction、reply 或 assistant 自述当成正式 command。

## Room topology

### Repo Room

RepoInstance 初始化时自动创建。它是开放研究大厅，适合探索、引用文件/Commit/Memo、并行邀请 Participant 做 bounded research，以及把成型话题提升为 Project。

Repo Room 默认只读代码。普通聊天不写 Git；只有显式 Memo publish 或受权的写入型 Invocation 才能创建 ChangeSet。

### Project Room

Project Room 是目标明确的长期协作空间，主要用于：

- 塑形 goal、scope、acceptance 与 Artifact；
- 形成 Task 和 Workflow proposal；
- 展示 Task、Run、Request、Receipt 的低噪声里程碑；
- 邀请不同 Participant 做 research、review 或 writing；
- 回答 Project 现在是什么状态、为什么。

Project 不预配常驻“包工头”。Participant 是可寻址 logical profile，只有显式调用才创建有边界的 Invocation。

### Scoped Room

Request 默认在卡片/详情中处理；只有需要多轮论述、多 Participant、共同编辑、跨多个对象或不同 ACL/secret/incident lifecycle 时，才升级为 Scoped Room。

Scoped Room 必须记录 parent Project/Run/Request、goal、completion condition、participants/facilitator、input ContextBundle、authority，以及结论应回填的 typed action。Facilitator 可以聚焦讨论和形成 proposal，但不能自行签发 DecisionReceipt。

## Message 与 timeline contract

Room timeline 可以展示 Comment、Participant Response、Invocation、Proposal、Request、Artifact Diff、Receipt/Verdict projection 和 System Milestone。只有最后两类可能投影已由权威层提交的事实；renderer action 仍只能发送 typed intent。

Canonical event/message 位于 repo-local SQLite；`RoomProjector` 生成可重建 view model，Workbench 中的 `RoomProjectionStore` 只是 cache：

```text
RoomTimelineVM {
  room_id,
  projection_revision,
  window { before_cursor, after_cursor, has_more },
  items[],
  active_streams{}
}

RoomTimelineItem {
  timeline_item_id,
  source_event_id,
  item_version,
  room_sequence,
  kind,
  actor { kind, stable_id, label_snapshot },
  provenance { correlation_id, invocation_id?, attempt_id?, trace_ref?, parent_event_id? },
  blocks[],
  actions[]
}
```

核心规则：

- `room_sequence` 由 control 在提交 source event 的同一事务中单调分配；timestamp、DOM index、完成顺序都不是排序 authority；
- fan-out 先按 deterministic participant/seat order 创建 stable placeholders，完成先后不重排；
- stream delta 以 `(room_id, item_id, block_id, stream_id, epoch, seq)` 幂等应用，旧 epoch 和倒退 seq 被丢弃；
- timeline 支持 cursor pagination、around-message、unread anchor、引用与附件；Phase 1 不增加 nested Task Room/thread；
- unknown block/card version 安全降级为可检查 fallback；message payload 不能注入 arbitrary React component；
- provider transcript、Execution Chat Projection 和 terminal scrollback 只能被显式分享为带 provenance 的引用或摘要。

## Semantic Composer 与 deterministic routing

Composer document、显示 label、wire envelope 和正式 RoomMessage 必须分开：

```text
Tiptap ComposerDocument
  → ComposerEnvelope { text, references[], attachments[], commands[] }
  → control authorize + resolve
  → RoomMessage + InvocationSpec / ActionProposal
```

| 形式 | 含义 | 示例 |
| --- | --- | --- |
| `@` | Participant 或 Project Role | `@codex`、`@role:security-reviewer` |
| `/` | typed action / collaboration Recipe | `/compare`、`/cross-review`、`/memo` |
| `$` | explicit Expertise/Skill overlay | `$architecture-review` |
| `#` | file、message、Artifact、Commit 等 input ref | `#file:src/auth.rs` |

Reference 保存 kind、stable ID、display label、scope 和可选 revision/digest。Label 只展示，不能 routing。

发送前，control 必须：

1. 解析 stable participant ID 或 ProjectRoleBinding；
2. 验证 membership、enabled、auth、health、capacity 和 required capability；
3. 应用显式 Expertise、Recipe 与 Project defaults；
4. 生成 ContextManifest/ContextBundle；
5. 预览 Harness、Context、Skill、permission 和 budget；
6. 用户发送后冻结 InvocationBinding；
7. 直接调用精确 adapter/runtime，不把 mention 字符串交给 lead LLM 猜测。

找不到 authorized candidate 时必须明确失败或要求选择，不能按 display name 模糊匹配或静默换人。

## Context、Skills 与知识晋升

`ContextAssembler` 是薄的 domain policy，而不是第二个 RAG platform：

```text
ContextProvider → ContextCandidate
ContextPolicy   → select / rank / budget / dedupe / authorize
ContextManifest → provenance / digest / why selected
ContextRenderer → adapter-specific ContextBundle
```

Phase 1 的确定性优先级为：显式 `#` refs → 当前 message/相关 Room window → Project goal/TaskRevision/Run/Request → Git diff/Artifact/Commit/Receipt → required Skill → FTS5 相关消息/Memo → provider session summary。压缩必须记录在 Manifest 中。

Skill 使用开放 `SKILL.md` 规范。用户显式 `$skill`、Recipe required skills、ProjectRoleBinding 和 Project defaults 是 deterministic；模型只能建议 optional skill，不能替换 required skill 或扩大权限。

长期知识采用 **proposal → preview → publish**：

- 默认 code 和已验证 Artifact 是实现 truth；显式 decision lock 可以声明一项稳定设计约束；
- Memo proposal 记录 exact source snapshot、route/provenance、适用范围与 repo revision；
- 用户可以删减、去敏和核验；
- 只有 publish 后的 Memo 写入 Git，原始 Room history 仍在 SQLite。

不引入 First Tree `Context Tree` 作为 HCTL 一级对象；其 durability/decision tests 被吸收到 Memo 和 Project knowledge promotion contract。

## RoomInvocation 与 compare

RoomInvocation 是显式、bounded、无隐藏自动后继的调用：

1. 先持久化 intent、InvocationBinding、ContextManifest 与 idempotency key；
2. 写入型调用在启动 Harness 前创建 ChangeSet/worktree 与 `ChangeSetWriteLease`；
3. agentd 以 invocation ID 幂等 start/reattach；
4. 完成后只展示 result/diff proposal，由用户 accept、follow-up 或提升为 Run；
5. control 重启时只 reattach identity/generation/lease 均匹配的现存 runtime；否则标记 Interrupted；
6. Retry 创建新 RoomInvocationRecord，旧结果仍受 revision/fence 约束。

`/compare` 是 Recipe，不是 Participant：

- Quick Compare 是一次 best-effort fan-out，不自动 retry、fallback、second round 或 synthesis；
- Durable Compare 涉及 durable join、fallback、gate/regate 或自动后继时，必须编译为 Workflow proposal，再由用户 Start Run。

## Request、Human Attention 与 resolution

Request 是 first-class object，不是特殊 message。最小字段包括 type/reason、problem statement、input schema、evidence/context links、affected revision、blocking scope、required actor/authority、deadline/default policy、claim/resolution 与 dedupe root cause。

行为合同：

- Request 可以精确指向一名 human 或一个 authorized role；
- Contributor、Facilitator、Required Actor、Authority Holder 与 Executor 必须分开；
- 普通 reply、clarification、reaction 或引用不会自动 resolve；只有显式 `resolves=request_id` 的 authorized typed action 才能关闭；
- claim/resolution 使用 expected revision/row lock，resolution reducer 扫描既有 resolution，防止重复关闭或重复 signal；
- 同一 root cause/revision 的 retry 更新原 Request；severity 升级或 deadline 才重新提醒；
- superseded revision 使 Request stale/cancelled；复杂论述可升级 Scoped Room，结论再 distill 为 typed proposal。

这部分借鉴 First Tree Need You/Human Request 的 strongest behavior，同时补上 HCTL 的 quorum、authority 和 revision boundary；Request 不替代 Gate。

## 外部 Chat surface contract

长期架构允许 Feishu/Slack/Discord/CLI 等消费同一 canonical Room，但 bridge 不是第二个 Chat truth。Ingress/egress 至少要求：

- stable external principal 与 Room binding；display name 不作 identity；
- signature/auth、tenant/room membership 和 exact mention validation；
- caller idempotency、provider event/message double dedupe、echo suppression；
- canonical RoomEvent 与 egress outbox 在同一 SQLite transaction；
- lease + epoch + commit-time fence；
- attachment hydration 与 immutable author snapshot；
- DeliveryReceipt 生命周期、retry/reconcile 与 prefix ACK；
- edit/delete 归一为 append-only correction/tombstone，不能抹掉已被引用的历史；
- multi-surface binding，不把 HCTL Room 永久限制为 1:1 外部 chat。

First Tree 的 Feishu main 快照只证明了其中一部分，且 outbound receipt、commit-time fence、多-surface 与 edit/delete 仍有缺口。因此该合同仍是 HCTL 应补的目标，不应误写成 donor 已完整实现或 Phase 1 已交付。

## Workbench 原生交互

Project Room 默认包含：

- Room/Project header 与 participant/capability/budget 状态；
- durable timeline、并发 Invocation streams、Request/Artifact/Receipt cards；
- Tiptap Composer 与 `@ / $ #` picker；
- Trigger Preview；
- 可选 Inspector；
- milestone-only Task/Run projection，避免机械日志刷屏。

Timeline 使用 HCTL scroll controller + `virtua` dynamic list。Prepend history、stream 扩高和 image/diff late layout 必须保持 anchor；用户离开底部后只显示 New activity。assistant-ui 仅在 `RoomMessageRendererPort` 后渲染单个 allowlisted message part/action，不使用它的 Thread/runtime/store/composer/cloud/queue。

## 没有 Workbench 时如何降级

### Phase 1 保证

- Canonical Room、draft、Invocation intent、Request 和 background Run 不因窗口退出而丢失；control/agentd 继续运行或安全恢复。
- Phase 1 的 CLI 至少提供 `status/doctor/export` 与只读 Room/Request inspection；未交付等价 composer client 时，新的复杂 shaping 安全暂停。
- 正在执行的 Run 不依赖 Room UI；blocking Request 保持 durable，直到 Workbench 恢复或通过已交付 typed CLI action 解决。
- 不允许用 sqlite shell、provider transcript 或直接调用 Harness 来“补写”canonical Room。

### Future seam，不是 Phase 1 承诺

- 外部 Chat bridge 和 transport-neutral CLI/API 可以读写同一 canonical Room；
- 即便交付，它们也可能缺少 rich diff、Context preview、multi-Invocation visualization 和 complex Request deliberation；
- Workbench 恢复后从 projection revision + cursor 重建，不从外部 client cache 回灌 truth。

无 Workbench 的 L4 是安全连续与可恢复，不是完全等价。

## 精选参考

### Anchor：First Tree

First Tree 的独特价值是 `Team → Agent → persistent Chat → Context Tree → human/SCM outcome` 闭环，证明 Chat/Context-first 可以成为完整协作主轴。HCTL 采用其 persistent Chat、typed mention、Context durability/decision tests、Human Request、durable Inbox 和 cross-surface QA 思路。

明确不继承：

- Team/Agent/Chat/Context Tree 领域模型与 hosted/PostgreSQL truth；
- task chat 作为 first-class Task——它只是 Chat 创建模式；
- provider/session retry 作为 Workflow fallback；
- internal detached tmux 作为 terminal attach 证据；
- in-place message edit、早于 canonical commit 的 ACK 或未闭环 egress；
- “First Tree 是 orchestration framework”的表述——其自身明确不是。

版本化研究、release/main 能力边界与链接见 [E-L4-FIRST-TREE](../references/implementation-evidence.md#e-l4-first-tree)。

### Focused supporting evidence

- Claude Tag：参考 channel member 可共同继续/steer 的 shared thread、scope-owned Agent identity/memory，以及 durable collaboration surface 与 ephemeral runtime 分离。Checklist/routine 只作 projection/trigger；Slack channel/thread 不映射为 Project/Room，也不绕过 HCTL permission/Gate。见 [E-L4-CLAUDE-TAG](../references/implementation-evidence.md#e-l4-claude-tag)。
- OpenClaw：参考 external channel 的 account/peer/thread identity、deterministic routing、pairing/allowlist、bot-loop protection 与 capability-aware degradation；不把 channel/session/workspace/agent 映射为 HCTL domain，也不让 ambient chatter 自动进入 canonical Context。见 [E-L4-OPENCLAW](../references/implementation-evidence.md#e-l4-openclaw)。
- Tiptap/ProseMirror：唯一 Composer engine；HCTL 自有 Reference schema、wire envelope 和 routing。
- `virtua`：dynamic viewport primitive，不拥有 Room identity/order/pagination。
- assistant-ui：scoped message/part/action renderer，不拥有 conversation semantics。
- Rocket.Chat、Mattermost、Zulip：合并为 timeline anchor/unread/a11y behavior tests，不成为 backend。

其余项目即使有 Chat，也不在 L4 引用，因为没有高于 First Tree 或这些 primitives 的独特增量。

## Failure 与 contract tests

最低验收包括：

- CJK IME、typed-reference round-trip、draft restore、paste/drop、async picker cancellation；
- 3–5 个 Participant 交错 stream，独立 cancel/retry，旧 epoch/seq 不串流；
- Room switch/reload 后 stale async event 不写入当前 Room；
- 10k dynamic items、prepend、height change、unread/focus/selection anchor 稳定；
- unknown card/version 可安全检查；renderer action 不能直接改变 domain；
- stable ID mention 与 membership/authority validation；display name collision 不误路由；
- Request FIFO/claim、draft/attachment、Submit/Skip、explicit resolves、double-resolution、revision supersede；
- external ingress duplicate/out-of-order/echo/reconnect，RoomEvent+outbox atomicity，lease/fence 与 DeliveryReceipt recovery；
- 完全移除 assistant-ui 或替换 virtualizer 后，Room identity、order、commands 和 recovery 不变。

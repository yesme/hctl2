# L3 · Commitment & Tracking — Task / Kanban

> Status: Normative · Draft v0.7.0<br>
> Primary reference: Codeg<br>
> External state references: Linear / GitHub<br>
> Parent: [四层设计规范](../README.md)

## 这一层为什么存在

L3 回答：**哪些讨论已经成为承诺，当前处于什么运营状态，以及什么证据才允许它完成。**

Room 中可以存在许多想法、proposal 和分歧；Workflow 中也可以存在许多机械 node。只有 Task 把 desired outcome、acceptance、来源、能力要求和可追踪状态冻结成用户愿意承担的工作单元。

Task 不能被 worktree、terminal、Agent job、provider Issue 或 Workflow Node 反向定义。一个 Task 可以没有 Run；一个 Run 可以支持多个 Task；一个 Task 也可以经历多次 Run。

## 本层负责什么

L3 负责：

- Project Overview 与 Task Kanban；
- Task identity、TaskRevision、TaskOperationalState 和 HCTL lifecycle；
- desired outcome、acceptance、source refs、required capability；
- Local/Linear/GitHub TaskSource binding、field authority 和 reconciliation；
- rank、stage、owner、blocker 与 Needs Attention projection；
- provider lifecycle 与 HCTL semantic completion 的双状态；
- Complete/Reopen/Cancel admission 与 TaskCompletionReceipt。

L3 不负责：

- 保存 Room 讨论或 Context；
- 因移动卡片而隐式启动 Harness/Run；
- 表达 Workflow token、retry、candidate、quorum 或 regate；
- 拥有 worktree、branch、session 或 terminal；
- 相信 provider Closed、Agent `task_complete` 或 Git commit 已满足 acceptance。

## Project Overview 与 Task Board

Project 是长期目标和聚合边界，可能同时存在 shaping、active Run、review 与下一版讨论，因此不进入线性 Kanban。Project Overview 展示 goal/health、Task counts、active Runs、Requests、Artifact/PR/CI、recent activity 和 risk。

Task 是 Board 唯一卡片类型。Run、Workflow Node、Request、Artifact 和 worktree 只能成为 badge、link 或 secondary view。

Phase 1 规范 lane 为：

- Backlog；
- Ready；
- In Progress；
- Review；
- Done；
- Cancelled（可过滤 terminal lane）。

Blocked 与 Needs Attention 是正交 health，不是 lane。`board_lane` 是从 HCTL lifecycle、provider/local non-terminal stage 和 verification policy 计算的 projection，不是 writer authority。

规则固定为：

```text
hctl_lifecycle_state = Completed → Done
hctl_lifecycle_state = Cancelled → Cancelled
otherwise                     → mapped non-terminal lane
```

若 provider 已 Closed/Cancelled 但 HCTL 仍 Open，保留最后一个有效 non-terminal lane（无历史时默认 Review），显示 `Closed externally · HCTL unverified` 与 Needs Attention。若 HCTL 已完成但 provider 写回未确认，显示 `Done · Verified · external sync pending`。

## Task contract 与 revision

一个 TaskRevision 至少冻结：

- title、desired outcome、acceptance；
- source refs；
- required role/capability；
- TaskSourceBinding revision 与已采用 source snapshot；
- contract projection 与 authority policy digest。

TaskOperationalState 保存 source workflow state、non-terminal stage、rank、priority、owner、blocker refs 和 sync state。Contract 与运营状态必须分离：移动卡片、重新排序或普通 assignee 变化不制造 TaskRevision；title/body/source contract 变化先形成 Snapshot/Proposal，adopt 后才创建新 Revision。

Active Run 使用已冻结 TaskRevision。外部 provider 或 Room proposal 不能原地改写它；用户必须结束/替换 Run，再采用新 TaskRevision。

## 从 L4 提炼承诺

Room 中的 discussion、Memo、Artifact 或 Request resolution 只能先形成 `DistillTaskProposal`。Adopt command 必须展示：

- 目标 Project；
- desired outcome 与 acceptance；
- source message/Artifact/Memo refs；
- proposed owner/priority/capability；
- 是否绑定 external source；
- expected Project/Task revision 与 authority。

只有经过 authorized adopt，才创建 Task 或新 TaskRevision。Room message 本身不会变成 Task，Project promotion 也不自动创建同名 Task。

## Card interaction 与 typed intents

卡片显示 title、Project/source badge、stable external key/link、priority/rank/owner、TaskRevision、health/attention、active Run、Request、Artifact/PR/CI、sync state，以及 provider lifecycle/HCTL verification 双状态。

拖拽只产生 non-terminal move intent：

```text
MoveTaskIntent {
  task_id,
  target_nonterminal_lane,
  before_task_id,
  expected_authority_policy_digest,
  expected_local_state_version?,
  expected_source_revision?,
  expected_source_binding_revision_id?,
  idempotency_key
}
```

进入 Done、离开 Done 和取消分别使用 `CompleteTaskIntent`、`ReopenTaskIntent`、`CancelTaskIntent`。普通 drag 不能生成这些命令。

Admission 规则：

- 拖入 In Progress 不 Start Run；
- Complete 必须满足当前 acceptance、required Receipt/evidence 和 active Run policy；
- local-owned field 在 SQLite transaction 内提交；
- external-owned field 先写 durable outbox 并显示 Pending Sync，经 pre-read、mutation、read-back 后 confirmed；
- provider terminal/HCTL Open 的 task 不参与普通 rank/reorder；
- 跨 provider 或 ordering scope 的 `before_task_id` 被拒绝；
- offline external field 默认只读；允许排队时也不能显示假成功；
- 必须有键盘/菜单等价路径，Drag-and-drop 不是唯一入口。

## Attention、review 与 follow-up

卡片把复杂 runtime 细节压缩成可行动信息：

- `Needs you` 聚合 Request、conflict、CI、source divergence 和 failed sync；
- setup/preflight 与真正 execution 分开；
- 保留 card timeline 和 provenance；
- 显示 review diff 与 acceptance evidence；
- 提供 Rework、Keep going、Ask、Double-check 等 explicit follow-up intent；
- Agent progress 始终标为 advisory；
- merge 或 provider transition 后重新核验 Git truth 与 acceptance。

这些 interaction 主要来自 Codeg To-dos 的成熟探索，但 HCTL 将它们置于 TaskRevision/authority/Receipt 之下。

## 无 Run 的 Task

轻量路径必须比直接使用单个 Harness 更轻：

1. 创建 TaskRevision；
2. 人直接编辑，或显式触发一个 bounded write RoomInvocation；
3. Invocation 只提交 result/diff proposal，不自动后继、retry 或 fallback；
4. 用户 review Artifact；
5. core 校验 file/revision/tests/review Receipt；
6. authorized actor Complete Task；
7. 同一事务写 CompletionReceipt、lifecycle event、Done projection 与 provider transition outbox。

整个过程可以没有 Workflow、Conductor 或 Run。

## Task Source 与 field authority

### Local

Local adapter 是 Phase 1 production baseline。SQLite/control 同时拥有 contract 与 operational fields，并使用单调 `state_version` 做 optimistic concurrency。

### Linear

Canonical identity 使用 provider account/organization + immutable `Issue.id`；human identifier 和名称只展示。WorkflowState ID/category、priority、rank 和 assignee 可以按 binding revision 成为 operational authority。Done/Closed 不拥有 HCTL completion。

### GitHub Issues / Projects v2

Issue node ID、Repository、ProjectV2、ProjectV2Item、Status field 和 option IDs 必须分开保存。Issue 与 Projects status/position 是多资源写入，adapter 以 outbox saga 处理 partial success；DraftIssue/PR/REDACTED item 在 Phase 1 不作为 Task。

### 共同同步合同

1. Provider snapshot 是 source observation；webhook 只是 invalidation hint。
2. 每个写入先 durable outbox，再 pre-read、mutation、read-back。
3. `clientMutationId`、delivery ID 与本地 key 只用于 correlation/dedupe，不是 exactly-once 或 CAS。
4. Timeout 进入 `uncertain`，先查询再决定 retry；create 结果不明时绝不盲目重复。
5. 同一 ordering scope 串行，rank/position 在最新 neighbor 上 rebase。
6. Contract projection 变化追加 Snapshot 并显示 PendingAdoption；active Run 不漂移。
7. Delete/archive 形成 tombstone；reauthorization、mapping drift 和 conflict 进入 Attention。
8. External assignee 只有经 stable principal binding 才可映射 Participant；comment 不自动进入 Room。

Phase 1 不依赖公网 webhook relay，使用 explicit refresh 与 overlap-window periodic reconcile。一个 Task 同时最多一个 writable external binding；provider/scope/item 变化是 migration。

## 从 L3 到 L2

Task 进入 Ready 或 In Progress 仍不意味着获得自动施工授权。L3 只有在用户：

1. 选择精确 TaskRevision；
2. 解决或显式接受 source divergence；
3. 批准 WorkflowRevision；
4. 预览 Run Manifest；
5. 执行 Start Run；

之后，才通过 typed seam 把 frozen commitment 交给 L2。Task Board position 永远不是 Workflow token。

## 从 L2 接收完成证据

Run Completed 只表示 Workflow execution 到达终态。L3 必须重新验证：

- 当前 TaskRevision acceptance；
- required Verdict/Receipt；
- Git/PR head、CI、review 与 merge eligibility；
- adopted source snapshot 与 provider current head；
- 是否存在未处理的 contract divergence；
- actor/policy 是否有 Complete authority。

通过后才签发 TaskCompletionReceipt 并进入 Done。Provider read-back 失败只留下 sync badge，不撤销已经有效的 HCTL completion；Reopen 保留旧 Receipt 历史。

## Workbench 原生交互

Workbench 提供：

- Project Overview 与 Repo/Project filtered Board；
- React Aria GridList + accessible DnD；
- stable key = `task_id`，每 lane 一个 collection；
- Card Inspector 中的 provider current / adopted contract 对照；
- Pending Sync、SourceChanged、Conflict、Tombstoned 和双完成状态；
- explicit Move/Complete/Reopen/Cancel/Start Run actions；
- Needs Attention、card timeline、review diff 和 follow-up intents。

Frontend collection 是 projection/cache，不拥有 Task truth。Optimistic state 必须可 rollback，并始终显示 pending/uncertain。

## 没有 Workbench 时如何降级

- Local Task 通过同一 command/query service 的 CLI 创建、adopt、move、complete/reopen；CLI 不直接写 SQLite。
- Linear/GitHub 原生 UI 可以修改各自拥有的 fields；HCTL periodic reconcile 后更新 snapshot/mirror。
- Provider 原生界面无法修改 HCTL acceptance、TaskRevision、Run binding、Receipt 或 semantic completion；外部 Closed 仍显示 unverified。
- `hctl status`/export/status card 可查看 adopted revision、provider divergence、active Run 和 open Request。
- 复杂 source adoption、conflict resolution 和 Completion preview 若没有等价 CLI，安全暂停到 Workbench 恢复。
- Workbench 恢复后从 canonical Task/event/source snapshots 重建 Board，不从浏览器 local state 或 provider order 反推 HCTL lifecycle。

L3 是四层中最自然的外部降级面，但仍受 field authority 限制。

## 精选参考

### Anchor：Codeg

Codeg 的最深探索不是 terminal，而是独立 To-do/WorkTask：异步 queue、四列 Board、Needs You、schedule/concurrency、setup/running/review/merge state compression、preflight、timeline、review diff、follow-up intents 与 Git truth recheck。Task 可以在没有 runtime/conversation 的情况下存在，这使它成为 L3 最高信息增益参考。

HCTL 采用这些 Task/attention/review interaction 和 behavior tests，但明确不继承：

- Conversation = Room；
- To-do = HCTL Task 或 fixed pipeline = Workflow；
- Task 永久绑定 worktree/session；
- 拖入 In Progress 自动施工；
- Agent `task_complete`、Done column 或 Git landed 自动满足 acceptance；
- lead-agent routing 作为 control truth。

Codeg 的 Composer、ACP registry 和 event cards 仍可作为组件 sourcing evidence，但不会因此在 L4/L1 再成为主要产品参考。详见 [E-L3-CODEG](../references/implementation-evidence.md#e-l3-codeg)。

### Focused implementation evidence：Hermes Agent

Hermes Agent 补的是 Codeg 未重点展开的 durable Task/attempt execution protocol：atomic claim、heartbeat/stale/crashed-worker reclaim、dependency promotion、protocol-violation auto-block，以及 CLI/Chat/dashboard 共用同一 Task kernel。HCTL 借 recovery 与 no-Workbench contract，不采用 Board=Project、model self-report=Receipt、single-host dispatcher=L2 truth 或 LLM judge=semantic completion。详见 [E-L3-HERMES-AGENT](../references/implementation-evidence.md#e-l3-hermes-agent)。

### Selected TaskSource references

- Linear：stable Issue identity、workflow/rank/priority/assignee field authority 与 provider-native no-Workbench surface。
- GitHub Issues/Projects：Issue/ProjectV2 identity 分离、多资源 saga、SCM 邻接与 provider-native surface。
- React Aria：Kanban DnD/a11y primitive，不是 Task model。

Multica 的 Issue Board、Inbox、Agent/Runtime separation 和 Trigger Preview 有局部 UX 价值，但未超过 Codeg + HCTL contract，保留在 Radar，不进入本层主叙事。以 worktree 为事实卡片的 execution board 也不定义 L3 Task。

## Failure 与 contract tests

- local version conflict、external snapshot divergence 与 mapping drift；
- create/move/close timeout、duplicate event、rate limit、partial saga、read-back mismatch；
- provider Closed/HCTL Open 与 HCTL Completed/provider pending 的双状态；
- active Run 期间 contract change 不改变 manifest digest；
- delete/tombstone/relink 保留 history；
- stale binding revision、cross-scope reorder 和 normal drag to Done 被拒绝；
- keyboard/screen reader 可 move、open Request、Complete/Reopen；
- Restart 后 outbox、cursor、snapshot、conflict 和 Board projection 可重建；
- Task claim race、heartbeat expiry、worker crash/reclaim、protocol violation 与 dependency promotion 可重放；
- merge/CI/provider transition 后仍重新验证 Git truth 与 current acceptance；
- Task 无 Run 路径完成一次真实 Artifact change，且不创建任何 hidden Run。

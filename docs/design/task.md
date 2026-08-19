# Task 与 Kanban

> 本文是 Task 模块的唯一领域权威；Kanban 是其操作合同，不拥有独立领域事实。模块交接见[连接合同](./connections.md)，通用机制见[系统边界](./system.md)。

## 模块职责

Task 是 Project 中一项可排序、可指派、可阻塞、可验收的长期承诺。它不等于自由讨论、外部 Issue、Workflow Node、worktree 或一次 Harness 作业。

| 对象 | 含义 |
| --- | --- |
| Task | 稳定身份、标题、目标结果和所属 Project |
| TaskRevision | 不可变的范围、验收标准、来源、所需角色和能力 |
| TaskOperationalState | 排序、优先级、负责人、阻塞、同步与派生健康状态 |
| TaskSourceConnectionRevision | Project 到外部 provider/account/scope 与 TaskSource 端口的不可变连接版本 |
| TaskSourceBindingRevision | Task 与外部来源、字段写入权和适配器版本的冻结绑定 |
| TaskSourceSnapshot | 外部系统一次只追加的原始与规范化观测 |
| TaskCompletionReceipt | 某次 CompleteTaskIntent 对精确 TaskRevision、规则、候选和证据的完成证明 |

Task lifecycle 只有 `Open | Completed | Cancelled`。契约变化创建新 TaskRevision；高频操作变化只更新 TaskOperationalState。历史 Revision、Run 和 Receipt 永不改写或物理删除。

`Backlog | Ready | InProgress | Review` 是 TaskOperationalState 中的本地非终态 stage，不是 Task lifecycle。Blocked 与 Needs Attention 是从 blocker、Request、Run、来源同步和验证事实派生的正交 health，不能覆盖 stage 或成为另一条 lifecycle。Kanban lane 由 local stage、lifecycle 与外部来源投影共同派生；Completed/Cancelled 由 lifecycle 决定，外部 Done/Closed 或拖卡都不能直接写成该终态。

## 契约与来源

TaskRevision 冻结验收合同，不冻结施工步骤。外部变化都先成为 Snapshot；其中会改变 TaskRevision 契约的内容才形成 `PendingAdoption`，用户采纳后才创建新 TaskRevision。由外部系统拥有的操作字段按 binding 与 Snapshot 投影，不经过 adoption。活动 Run 已冻结的 Revision 不能原地改写。

每个 Project 对同一 `provider + account_stable_id + scope_stable_id` 至多有一个 active TaskSource connection。每个外部规范实体在整个 RepoInstance 使用 `(provider, account_stable_id, external_entity_kind, immutable_external_entity_id)` 持久映射到一个 HCTL Task；该唯一键不含 connection、scope 或 placement，Disable/Rebind connection 或 placement 也不释放或重定向这份映射。TaskSourceBindingRevision 另行冻结可选的 placement identity（`placement_scope_stable_id + external_board_item_id`）及其写入权；移动 board placement 或更换 board-item binding 不会产生第二个 Task，也不能改写规范实体身份。

TaskSource connection/binding 的本地 current projection 使用 control 维护的单调 `state_version` 做 CAS；TaskSourceSnapshot 另行保存 provider 的 remote revision、digest 和 cursor。只有采用外部来源内容的 AdoptTaskRevisionIntent 才必须让 Snapshot、字段 authority policy 与新 TaskRevision 引用同一个 TaskSourceBindingRevision，并把 binding revision、snapshot、contract projection digest 和 authority-policy digest 一并写入 TaskRevision；采用本地 Room/Project 提案时改为冻结精确 Project 来源 refs、预期 contract version 和 proposal digest，不伪造 TaskSourceBindingRevision。任一适用 current pointer 已变化都使预览失效。远端 revision/digest 不能充当本地 `state_version`，本地版本也不能伪装成 provider 的并发令牌。

字段写入权由 TaskSourceBindingRevision 逐字段决定：

| 模式 | 规则 |
| --- | --- |
| local | HCTL 拥有契约与操作字段 |
| linked_readonly | 外部变化只形成快照、提案或 Needs Attention |
| external_authoritative | 外部系统拥有绑定中明确列出的字段 |

外部 Done/Closed/Reopen/Deleted 是来源事实，不会自动完成、重开、取消 HCTL Task，也不会停止 Run。删除只写 tombstone。

## 写入合同

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 不可变结果或边界 |
| --- | --- | --- | --- |
| Task / TaskRevision | contract version；`Open / Completed / Cancelled` 与独立 lifecycle version | control 处理 CreateTaskIntent、AdoptTaskRevisionIntent、CompleteTaskIntent、ReopenTaskIntent、CancelTaskIntent | TaskRevision 只追加；Reopen 不改写旧完成历史 |
| TaskOperationalState | `operational_state_version` | control 处理 UpdateTaskIntent 与 MoveTaskIntent；后者只改获准排序/位置 | 不启动 Run，不改变 TaskRevision 或 lifecycle |
| TaskSourceConnection / Binding | current revision + local `state_version`；`Active / Disabled / Replaced` | control 处理 Connect/Update/Disable 与 Bind/Rebind Intent，adapter 只返回观测 | 历史 Revision 不改写；规范实体到 Task 的 identity claim 持久唯一 |
| TaskSourceSnapshot | append-only sequence + remote revision/digest/cursor；可产生 `PendingAdoption` | control 持久化 refresh/reconcile 观测；AdoptTaskRevisionIntent 才消费内容变化 | Snapshot、tombstone 和外部 lifecycle 不能直接写 Task |
| TaskCompletionReceipt | immutable | 只有成功的 CompleteTaskIntent 事务可写 | 精确绑定该次 `task_lifecycle_version`、TaskRevision 与证据 |

CompleteTaskIntent 校验当前 Revision、验收规则、候选、Artifact/SCM/CI 和必需 Receipt，并对影响契约的 PendingAdoption 默认拒绝（fail-closed）：actor 必须先采纳并按新 Revision 重新验收，或显式选择“按当前冻结 Revision 完成”；后者必须冻结并 CAS 当前 TaskSourceBindingRevision/state version、source head 和全部未采纳的契约 Snapshot refs/digests，预览后新增或变化的 drift 一律使命令失效。StartRun 时的拒绝或延期不能代替这次选择。CompleteTaskIntent 与 CancelTaskIntent 在任何绑定该 Task 的非终态 Run 存在时都拒绝；必须先显式结束该 Run 并等到旧执行撤权、隔离，Task 命令不会隐式停止 Run。Reopen/Cancel 保留旧 Receipt 和历史。

Task 终结只有两个获准来源：有权 human actor 从 Kanban 场景提交 Task 命令，或绑定精确 TaskRevision 的 Run 正常进入 `Completed` 后由 Run reducer/control 机械提交同一个 CompleteTaskIntent。后者使用由 Run/Task 身份派生的稳定幂等键，仍经过本段全部 Task 准入；Run 已完成而 Task 校验失败时，Run 保持 `Completed`，Task 保持 `Open` 并显示 Needs Attention。`Failed / Cancelled / Superseded` Run 不能完成或取消 Task。CancelTaskIntent 只接受有权 human actor。这里的 Kanban 是命令场景而非某个窗口：Workbench、CLI 或适配后的第三方客户端都可以承载，但必须由认证入口证明 human provenance；外部 Closed、Participant、WorkerProfile、Harness、adapter、模型输出和 execution principal 都不是终结 actor。

TaskCompletionReceipt 至少固定 Task、CompleteTaskIntent、TaskRevision digest、验收策略、来源快照/head、逐项证据和 actor；若存在契约分歧，还必须固定显式 divergence choice、精确的未采纳 Snapshot refs/digests、BindingRevision/state version 与 authority-policy digest。Receipt、生命周期事件、当前投影与需要的外部写回 outbox 在同一事务提交。外部写回失败只显示 Needs Attention，不撤销已经成立的 HCTL 完成事实。

ReopenTaskIntent 只接受有权 human actor，必须以预期 `task_lifecycle_version` 把 `Completed/Cancelled → Open` 并推进版本；它不复活旧 Receipt。若当前来源契约已有未处理 drift，重开预览必须先采纳新 TaskRevision 或显式冻结继续使用的当前 Revision 与 divergence，不能让外部 Reopen 或旧完成证明静默决定新一轮施工。

Run 的裸终态、Harness 自述、Git commit、CI 绿色或外部 Closed 都不是 CompleteTaskIntent；唯一 Workflow 例外是 task-bound Run 正常完成后由 reducer 提交上述同一个 Task 命令。

## 无 Run 的轻量路径

```text
TaskRevision
  → 人工编辑或一次有边界的 RoomInvocation
  → ChangeSetRevision / ArtifactRevision + 测试证据
  → human actor 在 Kanban 预览精确 ReviewSubjectRef 与验收证据
  → CompleteTaskIntent
  → TaskCompletionReceipt
```

简单工作不需要先画 Workflow，也不伪造只能由 Run 产生的 Gate Receipt；TaskRevision 的验收合同若要求 HCTL 内部独立评审，必须使用带 Gate 的 [Run](./run.md)，若接受外部 SCM 评审则引用可回读的精确外部证据。需要持久重试、候选切换或 Gate 时，Task 才显式授权 Run。第一阶段一个 Run 绑定 0..1 个 TaskRevision，同一 Task 最多一个活动 Run。

StartRunIntent 预览必须列出会影响当前 TaskRevision 的全部 `PendingAdoption`，并要求 actor 明确采纳、拒绝或延期。采纳会先产生新 TaskRevision，再以新 Revision 重做 StartRun 预览；拒绝或延期必须随准入冻结当前 Revision 和精确来源快照，但未采纳的契约内容只作准入审计，不得进入 TaskRevision、Run Context/AttemptSpec。存在未处理的 PendingAdoption 时不得启动 Run，control 也不得自动采纳或静默越过；只有 AdoptTaskRevisionIntent 能让外部契约内容进入施工合同。external_authoritative 操作字段仍以当前 Snapshot 值和 binding version 作为 Start 的 CAS 前置，不能被 reject/defer 改写。

## Kanban 场景

Kanban 是 Task 的主要操作场景。卡片显示 TaskRevision、来源、排序、负责人、阻塞、Needs Attention、活跃 Run、Request、Artifact/PR/CI、外部生命周期和 HCTL 验证状态。Board lane 是投影，不是 Task lifecycle。

| 角色 | 可以做什么 | 不能做什么 |
| --- | --- | --- |
| 场景客户端：Workbench Board | 查询、预览、拖放和类型化 Task 命令 | 用拖卡启动 Run，或把 Done lane 当作完成事实 |
| 场景客户端：Local/CLI | 通过同一 command service 完整管理本地 Task | 直接改 SQLite |
| 受控端口 / 原生客户端：Linear/GitHub | 读写被授权字段、提供 Snapshot 与原生降级界面 | 接管 task_id、TaskRevision、Run 绑定或语义完成 |

HCTL 拥有排序字段时，MoveTaskIntent 必须冻结排序作用域、相邻 Task 和涉及的 `operational_state_version`；任何本地移动都推进受影响版本，使旧 CAS 失败并按当前投影重新计算位置。外部系统拥有排序字段时，同一命令改为冻结 TaskSource binding 的本地 `state_version` 与 provider remote token，经受控端口写入并回读 Snapshot；来源刷新推进 binding `state_version`，使旧预览失效。provider 若没有可条件写入的 remote token，adapter 不得伪造一个：只有 provider 提供等价原子版本前置时才开放相对排序写入，否则降级为只读或其确实支持且可回读的绝对移动。两种令牌不能混用，跨 provider 或排序作用域的相对移动必须拒绝。没有 Workbench 时，外部平台可以继续修改自己拥有的字段；复杂采纳、冲突解决和完成预览若无等价适配能力则安全暂停。

## 模块交接

以下只列所有权方向；字段、事务与故障语义由[四模块连接合同](./connections.md)统一定义。

- [Project](./project.md) 的提案经采纳产生 TaskRevision。
- Task 只通过显式 StartRunIntent 授权 [Run](./run.md)，移动卡片不会启动执行。
- Run 回传 Receipt、Verdict 和证据；task-bound Run 只有正常完成时才可由 reducer 提交同一个 CompleteTaskIntent，Task 仍独立准入。
- [Harness](./harness.md) 产生的代码或终端状态只有成为精确 Revision/证据后才可参与验收。

## 不可破坏的边界

- TaskRevision、TaskOperationalState、外部字段和 lifecycle 是不同事实。
- 外部规范实体只能映射一个 HCTL Task；它的稳定键与可变 scope/placement/item 身份分离。
- 活动 Run 的 TaskRevision 不漂移；改约必须结束或替代 Run。
- 任何适配器、Harness 或 execution principal 都不能绕过 CompleteTaskIntent 写 TaskCompletionReceipt，也不能冒充 human/reducer actor。
- Task 可以没有 Run；Run 完成只允许确定性提交 Task 命令，不等于该命令必然通过。

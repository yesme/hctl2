# Task 模块合同

> 状态：规范性合同 · 草案 v0.10.1<br>
> 本文是 Task 模块对象、状态机与写入合同的唯一权威；设计正文见 [Task 与 Kanban](../task.md)。族语义见[合同层总则](./README.md)，模块交接见[连接合同](./connections.md)，共享机制见[系统边界](./system.md)。

## 对象

| 对象 | 含义 |
| --- | --- |
| Task | 稳定身份、标题、目标结果和所属 Project |
| TaskRevision | 不可变的范围、验收标准、来源、所需角色和能力 |
| TaskOperationalState | 对后端操作字段（排序、优先级、负责人、阻塞）的本地投影与同步账，及派生健康状态；操作字段的 ground truth 在 content 后端 |
| TaskBinding | Task 与外部来源、字段写入权和适配器版本的冻结绑定 |
| TaskSourceSnapshot | 外部系统一次只追加的原始与规范化观测 |
| TaskCompletionReceipt | 某次 CompleteTaskIntent 对精确 TaskRevision、规则、候选和证据的完成证明 |

Repo 到外部 provider/account/scope 的连接由 ResolvedPortBinding（port_kind = task_source）承载；每个 Repo 对同一 `provider + account_stable_id + scope_stable_id` 至多解析一个 active binding。

Task lifecycle 只有 `Open | Completed | Cancelled`。契约变化创建新 TaskRevision；高频操作变化经受控端口写入 content 后端，回读为 TaskOperationalState 投影。历史 Revision、Run 和 Receipt 永不改写或物理删除。

`Backlog | Ready | InProgress | Review` 是 TaskOperationalState 中的本地非终态 stage，不是 Task lifecycle。Blocked 与 Needs Attention 是从 blocker、Request、Run、来源同步和验证事实派生的正交 health，不能覆盖 stage 或成为另一条 lifecycle。Kanban lane 由 local stage、lifecycle 与外部来源投影共同派生；Completed/Cancelled 由 lifecycle 决定，外部 Done/Closed 或拖卡都不能直接写成该终态。

## 契约与来源

任务 content 的家是 Repo 级选择：注册仓库或首次启用 Kanban 时为整个 Repo 选定一个 content 后端——本地任务服务器，或 GitHub/Linear 这类远端平台（场景客户端经 API 直访远端，客户端只是投影）。一个 Repo 一个 Board：Board 是该仓库任务 content 的容器；HCTL Project 在板上映射为后端的分组实体（父任务、milestone、Linear project 等，由适配器按能力声明，能力不足时降级为标签或过滤视图），Task 映射为分组之下的卡片。后端连接由 ResolvedPortBinding（port_kind = task_source）承载，更换后端是显式的绑定替换，不改变既有 Task 身份映射。

看板卡片是 content，粒度由后端自由承载（子任务、清单、微卡不受 HCTL 约束）。每张进入所选 scope 的卡都以稳定键映射一个 HCTL Task 身份（身份全量），但 TaskRevision 契约按需创建（契约惰性）：首次绑定 Run、首次提交完成命令或显式升格采纳时才冻结验收契约。没有契约的 Task 只有身份映射与操作投影，不进入治理；它在看板上的终态只是 content 投影，要让完成成为可核验事实，必须先升格出契约。

TaskRevision 冻结验收合同，不冻结施工步骤。后端与关联来源的变化都先成为 Snapshot；其中会改变 TaskRevision 契约的内容才形成 `PendingAdoption`，用户采纳后才创建新 TaskRevision。由 content 后端拥有的操作字段按 binding 与 Snapshot 投影，不经过 adoption。活动 Run 已冻结的 Revision 不能原地改写。

每个外部规范实体在用户级控制面账本内使用 `(provider, account_stable_id, external_entity_kind, immutable_external_entity_id)` 持久映射到一个 HCTL Task；该唯一键不含端口绑定、scope 或 placement，Disable/Rebind 端口绑定或 placement 也不释放或重定向这份映射。TaskBinding 另行冻结可选的 placement identity（`placement_scope_stable_id + external_board_item_id`）及其写入权；移动 board placement 或更换 board-item binding 不会产生第二个 Task，也不能改写规范实体身份。

task_source 端口绑定与 TaskBinding 的本地 current projection 使用 control 维护的单调 `state_version` 做 CAS；TaskSourceSnapshot 另行保存 provider 的 remote revision、digest 和 cursor。只有采用外部来源内容的 AdoptTaskRevisionIntent 才必须让 Snapshot、字段 authority policy 与新 TaskRevision 引用同一个 TaskBinding，并把 binding revision、snapshot、contract projection digest 和 authority-policy digest 一并写入 TaskRevision；采用本地 Room/Project 提案时改为冻结精确 Project 来源 refs、预期 contract version 和 proposal digest，不伪造 TaskBinding。任一适用 current pointer 已变化都使预览失效。远端 revision/digest 不能充当本地 `state_version`，本地版本也不能伪装成 provider 的并发令牌。

字段写入权由 TaskBinding 逐字段决定；契约、lifecycle 与完成凭证永远归控制面，不可配置：

| 模式 | 规则 |
| --- | --- |
| backend_authoritative | 所选 content 后端拥有该字段（卡片、流转、排序、评论等操作字段默认如此），外部变化按 Snapshot 投影 |
| hctl_authoritative | 控制面拥有该字段（契约、判决与验收类字段），后端只接收写回 |
| linked_readonly | 非后端的关联来源只形成快照、提案或 Needs Attention |

后端或关联来源的 Done/Closed/Reopen/Deleted 是 content 事实，不会自动完成、重开、取消 HCTL Task，也不会停止 Run。删除只写 tombstone。

## 写入合同

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 不可变结果或边界 |
| --- | --- | --- | --- |
| Task / TaskRevision | contract version；`Open / Completed / Cancelled` 与独立 lifecycle version | control 处理 CreateTaskIntent、AdoptTaskRevisionIntent、CompleteTaskIntent、ReopenTaskIntent、CancelTaskIntent | TaskRevision 只追加；Reopen 不改写旧完成历史 |
| TaskOperationalState | binding `state_version` + 后端并发令牌 | control 准入 UpdateTaskIntent 与 MoveTaskIntent，经受控端口写 content 后端并回读；投影只由回读推进 | 不启动 Run，不改变 TaskRevision 或 lifecycle |
| task_source 端口绑定 / TaskBinding | current revision + local `state_version`；`Active / Disabled / Replaced` | control 处理 Connect/Update/Disable 与 Bind/Rebind Intent，adapter 只返回观测 | 历史 Revision 不改写；规范实体到 Task 的 identity claim 持久唯一 |
| TaskSourceSnapshot | append-only sequence + remote revision/digest/cursor；可产生 `PendingAdoption` | control 持久化 refresh/reconcile 观测；AdoptTaskRevisionIntent 才消费内容变化 | Snapshot、tombstone 和外部 lifecycle 不能直接写 Task |
| TaskCompletionReceipt | immutable | 只有成功的 CompleteTaskIntent 事务可写 | 精确绑定该次 `task_lifecycle_version`、TaskRevision 与证据 |

CompleteTaskIntent 校验当前 Revision、验收规则、候选、Artifact/SCM/CI 和必需 Receipt，并对影响契约的 PendingAdoption 默认拒绝（fail-closed）：actor 必须先采纳并按新 Revision 重新验收，或显式选择“按当前冻结 Revision 完成”；后者必须冻结并 CAS 当前 TaskBinding/state version、source head 和全部未采纳的契约 Snapshot refs/digests，预览后新增或变化的 drift 一律使命令失效。StartRun 时的拒绝或延期不能代替这次选择。CompleteTaskIntent 与 CancelTaskIntent 在任何绑定该 Task 的非终态 Run 存在时都拒绝；必须先显式结束该 Run 并等到旧执行撤权、隔离，Task 命令不会隐式停止 Run。Reopen/Cancel 保留旧 Receipt 和历史。

Task 终结只有两个获准来源：有权 human actor 从 Kanban 场景提交 Task 命令，或绑定精确 TaskRevision 的 Run 正常进入 `Completed` 后由 Run reducer/control 机械提交同一个 CompleteTaskIntent。后者使用由 Run/Task 身份派生的稳定幂等键，仍经过本段全部 Task 准入；Run 已完成而 Task 校验失败时，Run 保持 `Completed`，Task 保持 `Open` 并显示 Needs Attention。`Failed / Cancelled / Superseded` Run 不能完成或取消 Task。CancelTaskIntent 只接受有权 human actor。这里的 Kanban 是命令场景而非某个窗口：Workbench、CLI 或适配后的第三方客户端都可以承载，但必须由认证入口证明 human provenance；外部 Closed、Participant、WorkerProfile、Harness、adapter、模型输出和 execution principal 都不是终结 actor。

TaskCompletionReceipt 至少固定 Task、CompleteTaskIntent、TaskRevision digest、验收策略、来源快照/head、逐项证据和 actor；若存在契约分歧，还必须固定显式 divergence choice、精确的未采纳 Snapshot refs/digests、TaskBinding revision/state version 与 authority-policy digest。Receipt、生命周期事件、当前投影与需要的外部写回 outbox 在同一事务提交。外部写回失败只显示 Needs Attention，不撤销已经成立的 HCTL 完成事实。

冻结契约（TaskRevision）、完成凭证与施工图是 Kanban 场景的结晶（“干什么的计划”与其完成证明）：权威在 metadata 账本，结晶副本按[双层保存政策](./system.md#事实与存储)写入 Git；施工图（WorkflowRevision）的对象定义与写入者仍归 [Run 模块合同](./run.md)，此处只归属其结晶类别。

ReopenTaskIntent 只接受有权 human actor，必须以预期 `task_lifecycle_version` 把 `Completed/Cancelled → Open` 并推进版本；它不复活旧 Receipt。若当前来源契约已有未处理 drift，重开预览必须先采纳新 TaskRevision 或显式冻结继续使用的当前 Revision 与 divergence，不能让外部 Reopen 或旧完成证明静默决定新一轮施工。

Run 的裸终态、Harness 自述、Git commit、CI 绿色或外部 Closed 都不是 CompleteTaskIntent；唯一 Workflow 例外是 task-bound Run 正常完成后由 reducer 提交上述同一个 Task 命令。

## StartRun 前置与排序令牌

StartRunIntent 预览必须列出会影响当前 TaskRevision 的全部 `PendingAdoption`，并要求 actor 明确采纳、拒绝或延期。采纳会先产生新 TaskRevision，再以新 Revision 重做 StartRun 预览；拒绝或延期必须随准入冻结当前 Revision 和精确来源快照，但未采纳的契约内容只作准入审计，不得进入 TaskRevision、Run Manifest、ContextManifest 或 ExecutionSpec。存在未处理的 PendingAdoption 时不得启动 Run，control 也不得自动采纳或静默越过；只有 AdoptTaskRevisionIntent 能让外部契约内容进入施工合同。backend_authoritative 操作字段仍以当前 Snapshot 值和 binding version 作为 Start 的 CAS 前置，不能被 reject/defer 改写。

排序与位置永远归 content 后端。MoveTaskIntent 冻结 TaskBinding 的本地 `state_version` 与后端的并发令牌，经受控端口写入并回读 Snapshot；来源刷新推进 binding `state_version`，使旧预览失效。后端若没有可条件写入的并发令牌，adapter 不得伪造一个：只有后端提供等价原子版本前置时才开放相对排序写入，否则降级为只读或其确实支持且可回读的绝对移动。本地任务服务器与远端平台各用自己的令牌，本地 `state_version` 与后端令牌不能互相冒充，跨后端或排序作用域的相对移动必须拒绝。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL | 任务后端（Linear / GitHub / 本地任务服务器） | 差异 |
| --- | --- | --- |
| Task | Issue / 任务卡 | 后端卡片承载 content；Task 的身份、契约与验收由 HCTL 拥有 |
| TaskOperationalState 的 stage | Linear workflow state / GitHub ProjectV2 status | 谁拥有该字段由 TaskBinding 逐字段决定 |
| 排序（rank） | Linear sortOrder / ProjectV2 排序 | 条件写入用 provider 自己的并发令牌；没有等价令牌就降级 |
| TaskBinding 的 placement | GitHub ProjectV2 item；Linear 无独立看板项，位置由 workflow state + sortOrder 派生 | 实体身份与看板位置分离；移动位置不产生第二个 Task |
| TaskSourceSnapshot | webhook / API payload | 先观测后采纳；会改契约的内容必须经用户采纳 |
| 后端 Closed | issue closed / 卡片终态 | 只是 content 事实，不等于验收完成 |
| TaskCompletionReceipt | 无对应 | HCTL 差异化语义：绑定精确契约与证据的完成证明 |

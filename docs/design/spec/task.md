# Task 模块约束

> 状态：规范性约束 · 草案 v0.16.0<br>
> 本文是 Task 模块对象、状态机与写入约束的唯一权威；设计正文见 [Task 与 Kanban](../task.md)。族语义见[约束层总则](./README.md)，模块交接见[连接约束](./connections.md)，共享机制见[系统边界](./system.md)。

## 对象

| 对象 | 含义 |
| --- | --- |
| Task | 稳定身份、标题、目标结果和所属 Project |
| Task Revision | 不可变的范围、验收标准、来源、所需角色和能力 |
| Task Binding | Task 与外部来源之间的冻结绑定。它固定外部身份、字段写入权、可接纳的供应端 human 动作和适配器版本。排序、优先级、负责人和阻塞等后端字段只形成操作投影，其事实源仍在 content 后端 |
| Task Source Snapshot | 外部系统一次只追加的原始与规范化观测 |
| Task Completion Receipt | 某次「完成 Task」命令对精确 Task Revision、规则、候选和证据的完成证明 |

Repo 到外部 provider/account/scope 的连接由 Resolved Port Binding（port_kind = task_source）承载；每个 Repo 对同一 `provider + account_stable_id + scope_stable_id` 至多解析一个活跃绑定。

Task lifecycle 只有开放 | 完成 | 已取消。第一阶段 `project_id` 是 Task 稳定身份的一部分，创建后不可改写；契约变化创建新 Task Revision，高频操作变化经受控端口写入 content 后端，回读为 Task Binding 的操作投影。历史 Revision、Run 和 Receipt 永不改写或物理删除。Project 已归档时拒绝创建、采纳、移动、重开、取消或完成 Task；归档前的静默条件见 [Project 约束](./project.md#repo-注册与-project-归档)。

Backlog、Ready、In Progress 和 Review 是本地阶段，不是 Task 生命周期。Blocked 和“需要关注”是独立于阶段的健康状态，由阻塞项、Request、Run、来源同步和验证事实派生。Kanban 泳道由本地阶段、Task 生命周期和外部来源投影共同计算。

完成和已取消由 Task 生命周期决定；外部 Done、Closed 或拖卡不能直接写成终态。满足本约束后文要求的 human Done 事件可以请求“完成 Task”，但只有命令成功才改变生命周期。

## 契约与来源

任务 content 的家是 Repo 级选择。注册仓库或首次启用 Kanban 时，系统为整个 Repo 选定一个 content 后端：本地任务服务器，或 GitHub/Linear 这类远端平台。场景客户端经 API 直访远端，客户端本身只是投影。

一个 Repo 一个 Board。Board 是该仓库任务 content 的容器；HCTL Project 在板上映射为后端的分组实体，如父任务、milestone 或 Linear project，能力不足时降级为标签或过滤视图；Task 映射为分组之下的卡片。后端连接由 Resolved Port Binding（`port_kind = task_source`）承载；更换后端是显式的绑定替换，不改变既有 Task 身份映射。

Board 与 Project 分组不是新聚合。它们的稳定锚定保存在 Repo 的 task-source 绑定元数据中，至少固定 `repo_id + board_scope_stable_id + project_id + group_kind + group_anchor_stable_id + binding_revision`。

分组锚点可以是后端父实体、milestone 或获准的标签与过滤器身份，但永远不是 Task、Task Binding 或某张“项目卡”。一个活跃 Project 在一个绑定中恰有一个获准锚点。适配器必须能按锚点稳定回读归属；做不到时，该后端只能显示过滤视图，不能声称支持 Task 身份导入或跨组移动。

看板卡片是 content，粒度由后端自由承载；子任务、清单和微卡不受 HCTL 约束。只有稳定归属到一个已准入 Project 分组的规范卡片，才能认领一个 HCTL Task 身份。未分组、同时落入多个 Project 分组，或分组锚点不可稳定回读的卡片只形成未认领 Snapshot 和需要关注；系统不得猜测 Project 或先创建 Task。

Task Revision 契约按需创建，但只能由显式“采纳契约”命令，或带已预览契约的“创建 Task”命令产生。无契约的“启动 Run”或“完成 Task”必须先要求该独立动作。没有契约的 Task 只有身份映射与操作投影，不进入治理；它在看板上的终态只是 content 投影。“完成 Task”不得在同一命令中隐式生成契约：预览必须要求先执行可审阅的“采纳契约”，再针对返回的精确 Revision 重新预览完成。

Task Revision 冻结验收契约，不冻结施工步骤；其不可变正文与位置、摘要在 Git，账本保存稳定身份、准入与 current pointer。后端与关联来源的变化先成为 Snapshot。只有会改变 Task Revision 契约的内容才形成待采纳；用户采纳且工具箱回读正文后，control 才准入新 Task Revision。content 后端拥有的操作字段按绑定与 Snapshot 投影，不经过采纳。

存在绑定该 Task 的非终态 Run 时，仍可“采纳契约”并推进 current Task Revision；活动 Run 已冻结的 Revision 不因此改写，Run 继续按冻结 Revision 执行。Run 正常完成路径只针对其冻结的 Revision。current 已前移时，Run 归约器的“完成 Task”按契约分歧拒绝，Task 保持开放并显示需要关注，不得静默按新 Revision 完成。

绑定 Task 的后端评论线是 Context 的萃取来源。组装器按当前 Snapshot 的引用和摘要把评论线冻结进 Context Manifest 并物化；交付方式见 [Project 约束](./project.md#context-memo-artifact)。评论线仍只能经“采纳契约”进入 Task Revision，物化本身不改变契约。

每个外部规范实体在用户级控制面账本内使用 `(provider, account_stable_id, external_entity_kind, immutable_external_entity_id)` 持久映射到一个 HCTL Task。该唯一键不含端口绑定、范围或放置位置；停用或重新绑定端口，或改变放置位置，都不释放或重定向这份映射。

Task Binding 另行冻结可选的放置身份——`placement_scope_stable_id + external_board_item_id`——及其写入权。移动看板位置或更换看板项绑定不会产生第二个 Task，也不能改写规范实体身份。

Task 有两条可恢复的创建路径：

1. HCTL-first：账本先固定 Task 身份并提交后端 outbox；携带初始契约时，再提交 Git 正文 outbox 和该 Revision 的准入意图。
2. content-first：对账过程先保存 Snapshot，再认领唯一外部实体并创建无契约 Task。

两条路径都按同一关联键恢复。确认回执未知时，Task 保持开放，并显示待确认或待同步；系统必须按精确关联键和摘要回读，不得盲目重投，也不得另建卡片或 Task。工具箱和适配器分别执行并回读，不能把 Git 或后端写入伪装成账本事务的一部分。content-first 路径只有在卡片恰好归属一个 Project 分组时才能认领，适配器不能自行选择 Project 或写 Task。并发命中同一实体时只能复用同一 Task 或返回类型化冲突。

外部卡随后移到另一 Project 分组、同时出现在多个分组或脱离原分组时，control 只追加 Snapshot，并把原 Task 标为需要关注。在恢复原位置或建立新 Task 前，系统必须阻止采纳、启动、完成和后端操作字段写入。

第一阶段不改变 Task 的 Project 归属，也不把不可变 `project_id` 改成新分组。“移动 Task”只能在原 Project 锚点内改变阶段和排序；跨 Project 分组的预览必须拒绝。需要改变 Project 时，用户显式取消或保留旧 Task，并在目标 Project 创建新 Task，再用来源引用连接历史。

task_source 端口绑定与 Task Binding 的本地 current 投影使用 control 维护的单调 `state_version` 做比较并交换；Task Source Snapshot 另行保存供应端的远端 revision、摘要和游标。采用外部来源内容的“采纳契约”命令必须让 Snapshot、字段权威策略与新 Task Revision 引用同一个 Task Binding，并把绑定版本、Snapshot、契约投影摘要和权威策略摘要一并写入 Task Revision。

采用本地 Room/Project 提案时，命令改为冻结精确 Project 来源引用、预期契约版本和提案摘要，不伪造 Task Binding。任一适用 current pointer 已变化时，预览失效。远端 revision 或摘要不能充当本地 `state_version`；本地版本也不能伪装成供应端的并发令牌。

字段写入权由 Task Binding 逐字段决定；契约、lifecycle 与完成凭证永远归控制面，不可配置：

| 模式 | 规则 |
| --- | --- |
| backend_authoritative | 所选 content 后端拥有该字段（卡片、流转、排序、评论等操作字段默认如此），外部变化按 Snapshot 投影 |
| hctl_authoritative | 控制面拥有该字段（契约、判决与验收类字段），后端只接收写回 |
| linked_readonly | 非后端的关联来源只形成快照、提案或需要关注 |

后端或关联来源的 Done/已关闭/Reopen/Deleted 是 content 事实，不会自动完成、重开、取消 HCTL Task，也不会停止 Run。删除只写 tombstone。

所选 task backend 的事件还可以承载 human 命令请求，但只对 Task Binding 明确列明的动作生效。第一阶段只允许一种动作归一为“完成 Task”命令草稿：已绑定的规范卡片由映射到归属 human 的账号从非终态进入 Done。

适配器必须固定绑定版本、规范外部实体 ID、Task ID、供应端 actor、变化前后值、远端 revision 或更新时间，以及可重复计算的幂等键，并在接纳前完成当前回读。Vikunja webhook 没有独立投递 ID 时，幂等键使用上述规范字段组，不把投递次数当身份。

HCTL 服务账号的写回、模型或 Harness、未知 actor、只看到当前 Done 而看不到一次明确变化，以及重复或迟到的旧 revision，都只追加 Snapshot，不能取得 human 来源。

control 对该完成请求执行与 Workbench/CLI 相同的预览和准入。只有预览不要求临场选择，而且绑定明确允许该供应端动作自动提交时，适配器才可以提交请求。否则，系统保留供应端 Done 与 HCTL 开放状态，并等待用户处理或返回类型化拒绝。

成功仍只由同一个“完成 Task”事务写 Task Completion Receipt。重开、取消、跨 Project 移动和契约采纳第一阶段没有供应端动作映射，必须使用公共命令入口。

## 写入约束

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 不可变结果或边界 |
| --- | --- | --- | --- |
| Task / Task Revision | contract version；开放 / 完成 / 已取消与独立 lifecycle version | control 处理「创建/采纳契约/完成/重开/取消 Task」命令 | Task Revision 只追加；Reopen 不改写旧完成历史 |
| 操作投影（Task Binding 字段组） | 绑定 `state_version`；后端并发前置按其能力使用 | control 准入「更新 Task」命令与「移动 Task」命令，经受控端口写 content 后端并回读；投影只由回读推进 | 不启动 Run，不改变 Task Revision 或 lifecycle |
| task_source 端口绑定 / Task Binding | current revision + local `state_version`；活跃 / 停用 / 已替换 | control 处理「接通/更新/停用」与「绑定/换绑」命令，adapter 只返回观测 | 历史 Revision 不改写；规范实体到 Task 的身份认领持久唯一 |
| Task Source Snapshot | append-only sequence + remote revision/digest/cursor；可产生待采纳 | control 持久化 refresh/reconcile 观测；「采纳契约」命令才消费契约变化；同一 provider event 若满足上文条件，adapter 另行归一出完成 command draft | Snapshot、tombstone 和外部 lifecycle 不能直接写 Task |
| Task Completion Receipt | immutable | 只有成功的「完成 Task」命令事务可写 | 精确绑定该次 `task_lifecycle_version`、Task Revision 与证据 |

每个 Task 在账本中至多有一个绑定 Run 的占用标记，状态为 `active | completion_pending`。“启动 Run”必须在创建 Run/Manifest 的同一用户级账本事务中，以比较并交换把空标记推进为 `active`。已有任一标记时，同一幂等键返回原 Run，其他启动必须拒绝。

替代只能走 [Run 约束](./run.md#启动与-manifest)规定的原子撤权和换代路径，不能先清空标记再留下两个可写执行。`completion_pending` 期间也拒绝另一次启动，以及来自 human 的 Task 完成或取消命令；只接受匹配 Run 归约器的内部完成命令。该命令成功或被 Task 持久拒绝时，control 在同一结果事务中清除标记。

“完成 Task”命令必须先校验当前 Revision、验收规则、候选和全部必需证据。存在未采纳的契约变化时，actor 必须先采纳新 Revision，或在预览中明确选择按当前 Revision 完成；后一选择必须冻结当前绑定、来源头和全部未采纳 Snapshot。预览后出现的新 Snapshot 或变化必须使命令失效。“启动 Run”命令预览时的拒绝或延期不能代替这次选择。

绑定该 Task 的非终态 Run 存在时，完成与取消命令都必须拒绝。用户必须先显式结束该 Run 并等待旧执行撤权、隔离；Task 命令不会隐式停止 Run。重开或取消必须保留旧 Receipt 和历史。

Task 终结只有两个获准 actor 来源：归属 human 的 Task 命令请求，或绑定精确 Task Revision 的 Run 正常完成后，由 Run 归约器和 control 提交的同一种“完成 Task”命令。human 请求可以来自 Workbench/CLI 的直接客户端连接，也可以来自上文已准入的供应端 Done 事件；两者生成同一命令信封，经过本段全部 Task 准入。

Run 路径使用由 Run/Task 身份派生的稳定幂等键。Run 已完成而 Task 校验失败时，Run 保持完成，Task 保持开放并显示需要关注。失败、已取消或被替代的 Run 不能完成或取消 Task。“取消 Task”命令只接受归属 human 的直接命令。这里的 Kanban 是动作语义而非某个窗口，客户端不产生权限等级。

Task Completion Receipt 至少固定 Task、“完成 Task”命令、Task Revision 引用与摘要和验收策略。每一条验收项还要分别固定通过或失败、Evidence/Verdict/Receipt 引用与摘要、来源 Snapshot、来源头或版本，以及适用的生产者与执行代次；不能用一个总括的“测试通过”替代逐项绑定。

若存在契约分歧，Receipt 还必须固定显式分歧选择、精确的未采纳 Snapshot 引用与摘要、Task Binding 版本与状态版本和权威策略摘要。Receipt、生命周期事件、current 投影、匹配的 `completion_pending` 占用标记清除和必要的外部写回 outbox 在同一事务提交。Run 路径若被 Task 拒绝，也在持久化拒绝结果与需要关注时清除同一标记。外部写回失败只显示需要关注，不撤销已经成立的 HCTL 完成事实。

冻结契约（Task Revision）与完成凭证是 Kanban 场景的结晶：Task Revision 的不可变正文字节以 Git 为 home，control 账本独占身份准入、digest、current 与 lifecycle；Task Completion Receipt 的权威在账本，Git 只有审计影子。完整边界见[系统存储约束](./system.md#git-的双重角色)；施工图（Workflow Revision）从 Room 讨论中结晶、归 Chat Room 场景，其对象与写入者归 [Run 模块约束](./run.md)。

「重开 Task」命令只接受有权 human actor，必须以预期 task_lifecycle_version 把完成/已取消 → 开放并推进版本；它不复活旧 Receipt。若当前来源契约已有未处理 drift，重开预览必须先采纳新 Task Revision 或显式冻结继续使用的当前 Revision 与 divergence，不能让外部 Reopen 或旧完成证明静默决定新一轮施工。

## 启动 Run 的前置与排序令牌

“启动 Run”命令预览必须列出会影响当前 Task Revision 的全部待采纳，并要求 actor 明确采纳、拒绝或延期。采纳会先产生新 Task Revision，再以新 Revision 重做“启动 Run”命令预览。拒绝或延期必须随准入冻结当前 Revision 和精确来源快照；未采纳的契约内容只作准入审计，不得进入 Task Revision、Run Manifest、Context Manifest 或 Execution Spec。

存在未处理的待采纳时不得启动 Run，control 也不得自动采纳或静默越过。只有“采纳契约”命令能让外部契约内容进入施工约束。`backend_authoritative` 操作字段仍以当前 Snapshot 值和绑定版本作为启动的比较并交换前置，不能被拒绝或延期动作改写。

Start、Complete、Adopt 与跨来源冲突判断若要求 task backend 的当前 placement、remote revision、source head 或完整 cursor，必须先完成当前回读；后端不可用、cursor 有 gap 或 readback 超出冻结 freshness 上限时类型化拒绝。只有验收策略明确允许某项已缓存证据时，命令才可固定其观测版本、时间和已知 gap 继续；“后端离线”本身不放宽 Project group、drift 或 CAS 前置。

排序与位置永远归 content 后端。「移动 Task」命令冻结 Task Binding 的本地 state_version，经受控端口按该后端提供的写入语义写入并回读 Snapshot；来源刷新推进绑定 state_version，使旧预览失效。后端的并发控制是后端自己的事：adapter 按能力声明使用它有的前置（条件写入、排序令牌），没有就以回读为准。provider 原生客户端把卡片移入 Done 时，content 变化已经由后端完成；adapter 不把它伪装成 HCTL「移动 Task」命令，只按上文另行产生「完成 Task」请求。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL | 任务后端（Linear / GitHub / 本地任务服务器） | 差异 |
| --- | --- | --- |
| Task | Issue / 任务卡 | 后端卡片承载 content；Task 的身份、契约与验收由 HCTL 拥有 |
| 操作投影的 stage | Linear workflow state / GitHub ProjectV2 status | 谁拥有该字段由 Task Binding 逐字段决定 |
| 排序（rank） | Linear sortOrder / ProjectV2 排序 | 归后端；adapter 按后端能力用其条件写入，以回读为准 |
| Task Binding 的 placement | GitHub ProjectV2 item；Linear 无独立看板项，位置由 workflow state + sortOrder 派生 | 实体身份与看板位置分离；移动位置不产生第二个 Task |
| Task Source Snapshot | webhook / API payload | 先观测后采纳；会改契约的内容必须经用户采纳 |
| 后端关闭态 | issue closed / 卡片终态 | 只是 content 事实，不等于验收完成 |
| Task Completion Receipt | 无对应 | HCTL 差异化语义：绑定精确契约与证据的完成证明 |

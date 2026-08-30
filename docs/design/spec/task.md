# Task 模块合同

> 状态：规范性合同 · 草案 v0.15.4<br>
> 本文是 Task 模块对象、状态机与写入合同的唯一权威；设计正文见 [Task 与 Kanban](../task.md)。族语义见[合同层总则](./README.md)，模块交接见[连接合同](./connections.md)，共享机制见[系统边界](./system.md)。

## 对象

| 对象 | 含义 |
| --- | --- |
| Task | 稳定身份、标题、目标结果和所属 Project |
| Task Revision | 不可变的范围、验收标准、来源、所需角色和能力 |
| Task Binding | Task 与外部来源、字段写入权、可接纳 provider human 动作和适配器版本的冻结绑定。后端操作字段（排序、优先级、负责人、阻塞）的本地投影、同步账与派生健康状态是它的字段组（下称"操作投影"），不是独立对象；操作字段的 ground truth 在 content 后端 |
| Task Source Snapshot | 外部系统一次只追加的原始与规范化观测 |
| Task Completion Receipt | 某次「完成 Task」命令对精确 Task Revision、规则、候选和证据的完成证明 |

Repo 到外部 provider/account/scope 的连接由 Resolved Port Binding（port_kind = task_source）承载；每个 Repo 对同一 `provider + account_stable_id + scope_stable_id` 至多解析一个 active binding。

Task lifecycle 只有开放 | 完成 | 已取消。第一阶段 `project_id` 是 Task 稳定身份的一部分，创建后不可改写；契约变化创建新 Task Revision，高频操作变化经受控端口写入 content 后端，回读为 Task Binding 的操作投影。历史 Revision、Run 和 Receipt 永不改写或物理删除。Project 已归档时拒绝创建、采纳、移动、重开、取消或完成 Task；归档前的静默条件见 [Project 合同](./project.md#repo-注册与-project-归档)。

Backlog | Ready | In Progress | Review 是操作投影中的本地非终态 stage，不是 Task lifecycle。Blocked 与需要关注是从 blocker、Request、Run、来源同步和验证事实派生的正交 health，不能覆盖 stage 或成为另一条 lifecycle。Kanban lane 由 local stage、lifecycle 与外部来源投影共同派生；完成/已取消由 lifecycle 决定，外部 Done/Closed 或拖卡不能直接写成该终态。满足本合同后文要求的 human Done 事件可以请求「完成 Task」，但只有命令成功才改变 lifecycle。

## 契约与来源

任务 content 的家是 Repo 级选择：注册仓库或首次启用 Kanban 时为整个 Repo 选定一个 content 后端——本地任务服务器，或 GitHub/Linear 这类远端平台（场景客户端经 API 直访远端，客户端只是投影）。一个 Repo 一个 Board：Board 是该仓库任务 content 的容器；HCTL Project 在板上映射为后端的分组实体（父任务、milestone、Linear project 等，由适配器按能力声明，能力不足时降级为标签或过滤视图），Task 映射为分组之下的卡片。后端连接由 Resolved Port Binding（port_kind = task_source）承载，更换后端是显式的绑定替换，不改变既有 Task 身份映射。

Board 与 Project 分组不是新聚合；其稳定锚定保存在 Repo 的 task-source 绑定元数据中，至少固定 `repo_id + board_scope_stable_id + project_id + group_kind + group_anchor_stable_id + binding_revision`。group anchor 可以是后端父实体、milestone 或获准的 label/filter identity，但永远不是 Task、Task Binding 或某张“项目卡”；一个 active Project 在一个 binding 中恰有一个获准 anchor。adapter 必须能按 anchor 稳定回读归属，做不到时该后端只能显示过滤视图，不能声称支持 Task 身份导入或跨组移动。

看板卡片是 content，粒度由后端自由承载（子任务、清单、微卡不受 HCTL 约束）。只有稳定落在恰好一个已准入 Project group anchor 下的规范卡片，才可 claim 一个 HCTL Task 身份；未分组、同时落入多个 Project group 或 anchor 不可稳定回读的卡片只形成未认领 Snapshot/需要关注，不得猜 Project 或先造 Task。Task Revision 契约按需创建（契约惰性），但只能由显式「采纳契约」或带已预览契约的「创建 Task」产生；无契约的「启动 Run」或「完成 Task」必须先要求该独立动作。没有契约的 Task 只有身份映射与操作投影，不进入治理；它在看板上的终态只是 content 投影。「完成 Task」不得在同一命令中隐式生成契约：预览必须要求先执行可审阅的「采纳契约」，再针对返回的精确 Revision 重新预览完成。

Task Revision 冻结验收合同，不冻结施工步骤；其不可变正文与 locator/digest 在 Git，账本保存稳定 identity、准入与 current pointer。后端与关联来源的变化都先成为 Snapshot；其中会改变 Task Revision 契约的内容才形成待采纳，用户采纳并由工具箱回读正文后才准入新 Task Revision。由 content 后端拥有的操作字段按 binding 与 Snapshot 投影，不经过 adoption。存在绑定该 Task 的非终态 Run 时仍可「采纳契约」推进 current Task Revision；活动 Run 已冻结的 Revision 不因此改写，Run 继续按冻结 Revision 执行。Run 正常完成路径只针对其冻结的 Revision：current 已前移时，Run reducer 的「完成 Task」按契约分歧拒绝，Task 保持开放并显示需要关注，不得静默按新 Revision 完成。绑定 Task 的后端评论线是 Context 的萃取来源：组装器按当前 Snapshot 的 ref+digest 把评论线冻结进 Context Manifest 并物化（投喂档见 [Project 合同](./project.md#context-memo-artifact)）；它进入 Task Revision 仍只经「采纳契约」，物化不改变契约。

每个外部规范实体在用户级控制面账本内使用 (provider, account_stable_id, external_entity_kind, immutable_external_entity_id) 持久映射到一个 HCTL Task；该唯一键不含端口绑定、scope 或 placement，Disable/Rebind 端口绑定或 placement 也不释放或重定向这份映射。Task Binding 另行冻结可选的 placement identity（placement_scope_stable_id + external_board_item_id）及其写入权；移动 board placement 或更换 board-item binding 不会产生第二个 Task，也不能改写规范实体身份。

Task 有两条可恢复的创建路径。HCTL-first 的「创建 Task」先在用户级账本事务固定 `task_id + immutable project_id`、规范命令 digest，并提交必需的后端卡片 create outbox；仅当命令同时携带已预览的初始 Task Revision 时，事务才另写该 Revision 的 admission intent 与 Git 正文 outbox。外部卡使用稳定 correlation/idempotency key 和该 Project group anchor。工具箱/adapter 随后分别执行并回读，不能假装 Git 或后端写入发生在账本事务里。任一适用写入的 ACK 未知时 Task 保持开放 lifecycle，并以待确认/待同步 health 显示；恢复按精确 key 与 digest 回读，不能盲重投、另造卡片或另造 Task，只有该分支要求的正文与必需外部 identity 回读后才推进相应 admission/binding。content-first 则由 reconcile 先保存 Snapshot，确认卡片恰属一个 Project group 后，在一个账本事务中 claim 或复用规范外部实体唯一键，并创建一个无契约 Task 身份、Task Binding 与操作投影；adapter 只观测，不能自己选择 Project 或写 Task。并发 HCTL-first/content-first 命中同一规范实体时，唯一 claim 胜出，另一条路径复用同一 Task 或返回 typed conflict。

外部卡随后漂移到另一 Project group、同时出现在多个 group 或脱离原 group，只追加 Snapshot 并把原 Task 标为需要关注；在恢复原 placement 或按下一句建立新 Task 前，它阻止 Adopt、Start、Complete 和后端操作字段写入。第一阶段不 reparent，也不把 immutable `project_id` 改成新 group。「移动 Task」仅能在原 Project anchor 内改 stage/rank，跨 Project group 的预览必须拒绝。需要改变 Project 时，显式取消/保留旧 Task 并在目标 Project 创建新 Task，用来源引用连接历史。

task_source 端口绑定与 Task Binding 的本地 current projection 使用 control 维护的单调 state_version 做 CAS；Task Source Snapshot 另行保存 provider 的 remote revision、digest 和 cursor。只有采用外部来源内容的「采纳契约」命令才必须让 Snapshot、字段 authority policy 与新 Task Revision 引用同一个 Task Binding，并把 binding revision、snapshot、contract projection digest 和 authority-policy digest 一并写入 Task Revision；采用本地 Room/Project 提案时改为冻结精确 Project 来源 refs、预期 contract version 和 proposal digest，不伪造 Task Binding。任一适用 current pointer 已变化都使预览失效。远端 revision/digest 不能充当本地 state_version，本地版本也不能伪装成 provider 的并发令牌。

字段写入权由 Task Binding 逐字段决定；契约、lifecycle 与完成凭证永远归控制面，不可配置：

| 模式 | 规则 |
| --- | --- |
| backend_authoritative | 所选 content 后端拥有该字段（卡片、流转、排序、评论等操作字段默认如此），外部变化按 Snapshot 投影 |
| hctl_authoritative | 控制面拥有该字段（契约、判决与验收类字段），后端只接收写回 |
| linked_readonly | 非后端的关联来源只形成快照、提案或需要关注 |

后端或关联来源的 Done/已关闭/Reopen/Deleted 是 content 事实，不会自动完成、重开、取消 HCTL Task，也不会停止 Run。删除只写 tombstone。

所选 task backend 的事件还可以承载 human 命令请求，但只对 Task Binding 明确列明的动作生效。第一阶段只允许“已绑定规范卡片由映射到 owner human 的账号从非终态进入 Done”归一为「完成 Task」command draft；adapter 必须固定 binding revision、规范外部实体 ID、Task ID、provider actor、变化前后值、remote revision/updated version 与可重复计算的幂等键，并在接纳前 fresh readback。Vikunja webhook 没有独立 delivery ID 时，幂等键使用上述规范 tuple，不把投递次数当身份。HCTL service account 的写回、模型/Harness、未知 actor、只看到当前 Done 而看不到一次明确变化、重复/迟到的旧 revision，都只追加 Snapshot，不能取得 human provenance。

control 对该 draft 执行与 Workbench/CLI 相同的 Preview 和准入。只有 preview 不要求采纳契约、选择分歧、停止活动 Run 等临场决定，且 binding 已明确授权这个 provider 动作表达“请求提交”时，adapter 才可继续 Submit；否则保留外部 Done + HCTL 开放的双重状态，并显示类型化拒绝或待用户处理。成功仍只由同一个「完成 Task」事务写 Task Completion Receipt。Reopen、Cancel、跨 Project 移动和契约采纳第一阶段没有 provider 动作映射，必须使用公共命令入口。

## 写入合同

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 不可变结果或边界 |
| --- | --- | --- | --- |
| Task / Task Revision | contract version；开放 / 完成 / 已取消与独立 lifecycle version | control 处理「创建/采纳契约/完成/重开/取消 Task」命令 | Task Revision 只追加；Reopen 不改写旧完成历史 |
| 操作投影（Task Binding 字段组） | binding `state_version`；后端并发前置按其能力使用 | control 准入「更新 Task」命令与「移动 Task」命令，经受控端口写 content 后端并回读；投影只由回读推进 | 不启动 Run，不改变 Task Revision 或 lifecycle |
| task_source 端口绑定 / Task Binding | current revision + local `state_version`；活跃 / 停用 / 已替换 | control 处理「接通/更新/停用」与「绑定/换绑」命令，adapter 只返回观测 | 历史 Revision 不改写；规范实体到 Task 的 identity claim 持久唯一 |
| Task Source Snapshot | append-only sequence + remote revision/digest/cursor；可产生待采纳 | control 持久化 refresh/reconcile 观测；「采纳契约」命令才消费契约变化；同一 provider event 若满足上文条件，adapter 另行归一出完成 command draft | Snapshot、tombstone 和外部 lifecycle 不能直接写 Task |
| Task Completion Receipt | immutable | 只有成功的「完成 Task」命令事务可写 | 精确绑定该次 `task_lifecycle_version`、Task Revision 与证据 |

每个 Task 在账本中至多有一个 task-bound Run claim，状态为 `active | completion_pending`。「启动 Run」必须在创建 Run/Manifest 的同一用户级账本事务把空 claim CAS 为 active；已有任一 claim 时，同一 idempotency key 返回原 Run，其他 Start 拒绝。替代只能走 [Run 合同](./run.md#启动与-manifest)规定的原子撤权/换代路径，不能先清空 claim 再留下两个可写执行。`completion_pending` 期间也拒绝另一 Start 与来自 human 的 Task 完成/取消命令，只接受匹配 Run reducer 的内部完成命令；该命令成功或被 Task 持久拒绝时，在同一结果事务清除 claim。

「完成 Task」命令校验当前 Revision、验收规则、候选、Artifact/SCM/CI 和必需 Receipt，并对影响契约的待采纳默认拒绝（fail-closed）：actor 必须先采纳并按新 Revision 重新验收，或显式选择“按当前冻结 Revision 完成”；后者必须冻结并 CAS 当前 Task Binding/state version、source head 和全部未采纳的契约 Snapshot refs/digests，预览后新增或变化的 drift 一律使命令失效。「启动 Run」命令预览时的拒绝或延期不能代替这次选择。「完成 Task」命令与「取消 Task」命令在任何绑定该 Task 的非终态 Run 存在时都拒绝；必须先显式结束该 Run 并等到旧执行撤权、隔离，Task 命令不会隐式停止 Run。Reopen/Cancel 保留旧 Receipt 和历史。

Task 终结只有两个获准 actor 来源：owner human 的 Task 命令请求，或绑定精确 Task Revision 的 Run 正常进入完成后由 Run reducer/control 机械提交同一个「完成 Task」命令。human 请求可以来自 Workbench/CLI direct client connection，也可以来自上文已准入的 provider Done event；两者生成同一 command envelope，经过本段全部 Task 准入。Run 路径使用由 Run/Task 身份派生的稳定幂等键；Run 已完成而 Task 校验失败时，Run 保持完成，Task 保持开放并显示需要关注。失败 / 已取消 / 被替代 Run 不能完成或取消 Task。「取消 Task」命令只接受 owner human 的 direct command。这里的 Kanban 是动作语义而非某个窗口，客户端不产生权限等级。

Task Completion Receipt 至少固定 Task、「完成 Task」命令、Task Revision ref+digest、验收策略，以及每一条验收项各自的 pass/fail、Evidence/Verdict/Receipt ref+digest、来源 snapshot/head/version 与适用的 producer/执行代次；不能用一个总括“tests passed”替代逐项绑定。若存在契约分歧，还必须固定显式 divergence choice、精确的未采纳 Snapshot refs/digests、Task Binding revision/state version 与 authority-policy digest。Receipt、生命周期事件、current 投影、匹配的 `completion_pending` claim 清除与需要的外部写回 outbox 在同一事务提交；Run 路径若被 Task 拒绝，也在持久化拒绝结果与需要关注时清除同一 claim。外部写回失败只显示需要关注，不撤销已经成立的 HCTL 完成事实。

冻结契约（Task Revision）与完成凭证是 Kanban 场景的结晶：Task Revision 的不可变正文字节以 Git 为 home，control 账本独占身份准入、digest、current 与 lifecycle；Task Completion Receipt 的权威在账本，Git 只有审计影子。完整边界见[系统存储合同](./system.md#git-的双重角色)；施工图（Workflow Revision）从 Room 讨论中结晶、归 Chat Room 场景，其对象与写入者归 [Run 模块合同](./run.md)。

「重开 Task」命令只接受有权 human actor，必须以预期 task_lifecycle_version 把完成/已取消 → 开放并推进版本；它不复活旧 Receipt。若当前来源契约已有未处理 drift，重开预览必须先采纳新 Task Revision 或显式冻结继续使用的当前 Revision 与 divergence，不能让外部 Reopen 或旧完成证明静默决定新一轮施工。

## 启动 Run 的前置与排序令牌

「启动 Run」命令预览必须列出会影响当前 Task Revision 的全部待采纳，并要求 actor 明确采纳、拒绝或延期。采纳会先产生新 Task Revision，再以新 Revision 重做「启动 Run」命令预览；拒绝或延期必须随准入冻结当前 Revision 和精确来源快照，但未采纳的契约内容只作准入审计，不得进入 Task Revision、Run Manifest、Context Manifest 或 Execution Spec。存在未处理的待采纳时不得启动 Run，control 也不得自动采纳或静默越过；只有「采纳契约」命令能让外部契约内容进入施工合同。backend_authoritative 操作字段仍以当前 Snapshot 值和 binding version 作为 Start 的 CAS 前置，不能被 reject/defer 改写。

Start、Complete、Adopt 与跨来源冲突判断若要求 task backend 的当前 placement、remote revision、source head 或完整 cursor，必须先完成 fresh readback；后端不可用、cursor 有 gap 或 readback 超出冻结 freshness 上限时类型化拒绝。只有验收策略明确允许某项 cached evidence 时，命令才可固定其观测版本、时间和已知 gap 继续；“后端离线”本身不放宽 Project group、drift 或 CAS 前置。

排序与位置永远归 content 后端。「移动 Task」命令冻结 Task Binding 的本地 state_version，经受控端口按该后端提供的写入语义写入并回读 Snapshot；来源刷新推进 binding state_version，使旧预览失效。后端的并发控制是后端自己的事：adapter 按能力声明使用它有的前置（条件写入、排序令牌），没有就以回读为准。provider 原生客户端把卡片移入 Done 时，content 变化已经由后端完成；adapter 不把它伪装成 HCTL「移动 Task」命令，只按上文另行产生「完成 Task」请求。

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

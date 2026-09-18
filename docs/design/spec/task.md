# Task 模块约束

> 状态：规范性约束 · 草案 v0.18.7<br>
> 本文是 Task 模块对象、状态机与写入约束的唯一权威；设计正文见 [Task 与 Kanban](../task.md)。族语义见[约束层总则](./README.md)，模块交接见[连接约束](./connections.md)，共享机制见[系统边界](./system.md)。

## 对象

| 对象 | 含义 |
| --- | --- |
| Task | 稳定身份、标题、目标结果和所属 Project |
| Task Revision | 不可变的范围、验收项（每项带校验等级）、来源、所需职责和能力 |
| Task–Backend Binding | Task 与外部来源之间的冻结绑定。它固定外部身份（实体键，即这张卡的**家指针**：卡在哪个源、是哪张卡，创建或认领时落定，此后不改写）、字段写入权、可接纳的供应端 human 动作和适配器版本。排序、优先级、负责人和阻塞等后端字段只形成操作投影，其事实源仍在 content 后端 |
| Task Backend Snapshot | 外部系统一次只追加的原始与规范化观测 |
| Task Completion Receipt | 某次「完成 Task」命令对精确 Task Revision、规则、候选和证据的完成证明 |

Repo 到任务源（provider/account/scope）的连接由 Port–Provider Binding（port_kind = task_source）承载，零到多个；每个 Repo 对同一 `provider + account_stable_id + scope_stable_id` 至多解析一个活跃绑定。

Task lifecycle 只有开放 | 完成 | 已取消。`project_id` 是 Task 稳定身份的一部分，创建后不可改写；契约变化创建新 Task Revision，高频操作变化经受控端口写入 content 后端，回读为 Task–Backend Binding 的操作投影。历史 Revision、Run 和 Receipt 永不改写或物理删除。Project 已归档时拒绝创建、采纳、移动、重开、取消或完成 Task；归档前的静默条件见 [Project 约束](./project.md#repo-注册与-project-归档)。

Backlog、Ready、In Progress 和 Review 是本地阶段，不是 Task 生命周期。Blocked 和“需要关注”是独立于阶段的健康状态，由阻塞项、Request、Run、来源同步和验证事实派生。Kanban 泳道由本地阶段、Task 生命周期和外部来源投影共同计算。

完成和已取消由 Task 生命周期决定；外部 Done、Closed 或拖卡不能直接写成终态。满足本约束后文要求的 human Done 事件可以请求“完成 Task”，但只有命令成功才改变生命周期。

## 契约与来源

任务 content 的家按仓库绑定的任务源来定。一个仓库可以绑零到多个任务源——仓库所绑平台自带的 issues、本地任务服务器、Linear 这类远端平台；一个都不绑时不启用看板，Room 与其他模块照常。注册仓库或首次启用看板时，人从候选列表显式选定仓库级**缺省任务源**：候选列表里的缺省建议项是平台自带的 issues，显式不挂平台的仓库缺省建议项是本地任务服务器；能作缺省源的只有能力声明含建卡与字段写回的绑定，只过了身份与快照的绑定在列表里标「不能作缺省源、可认领」；没有显式同意，系统不绑源、不建卡、不启用看板。Project 可以显式接入仓库已绑定的一个或多个任务源，分别保存**源引用**；只开聊天室时可以不接源。每个源引用指明绑定与获准范围，在 Project 的 Kanbans 中形成一个入口。采用仓库缺省建议仍须人确认；之后缺省建议变化不改已有引用，不搬卡或分组锚点。从 HCTL 新建 Task 时必须明确选择本 Project 已接入且具备建卡能力的源；未选定、能力不足或范围不符时拒绝，不按 Room 名字猜源。两层选择可以在同一个预览里确认。场景客户端经 API 直访远端，客户端本身只是投影。

每个 Project 的每个 Kanban 入口投影一个源引用所选范围的卡片，区分在本 Project 尚未认领的内容与本 Project 的 Task；看见卡片不等于认领。读取失败或结果不全必须标明，不能显示成空板。可选的**合并板**只汇总这些来源，不替代按源入口，不是对象或权威表。跨 Project 或控制面汇总仍分别展示各自 Task、验收状态与控制面/Project 来源，不合并治理身份；只去重同一外部内容的重复呈现。后端连接由 Port–Provider Binding（`port_kind = task_source`）承载；更换某个源的后端是显式的绑定替换，不改变既有 Task 身份映射。

源内的板范围与 Project 分组不是新聚合。板范围保存在该任务源绑定的元数据中，固定 `repo_id + board_scope_stable_id + binding_revision`；启用 Project 原生分组映射时另固定 `project_id + group_kind + group_anchor_stable_id`。每个源绑定各有自己的 `board_scope_stable_id`，合并板不另存；无可用原生分组时不伪造分组锚点。

分组锚点可以是后端父实体、milestone 或获准的标签与过滤器身份，但永远不是 Task、Task–Backend Binding 或某张“项目卡”。**两套分组并存**：源内分组是后端的父实体、milestone、标签或过滤视图；本控制面中 Task 到 Project 的固定归属是另一件事。启用原生分组映射时，在人选定的该 Project 源引用所指绑定内固定一个获准锚点；接入多个源不自动为每个源补建分组。稳定唯一的原生分组只作为自动认领的前置：适配器做不到按锚点稳定回读归属时，该源没有自动认领，只有人的显式认领与过滤视图。

看板卡片是 content，粒度由后端自由承载；子任务、清单和微卡不受 HCTL 约束。认领分两路：本 Project 已接入的源里，按本 Project 已准入的原生分组映射能稳定、无歧义识别的规范卡片，可由对账自动认领；没有这种映射或映射有歧义的卡，只能由有权的人经「认领卡片」命令显式认领进指定 Project，按实体身份、权限、所选 Project 与源引用核验；尚未接入的源先显式接入，不要求外源替它建分组；适配器不自选 Project、不先创建 Task，否则只形成未认领 Snapshot 和需要关注。**一张卡一个家，认领不搬家**：卡留在它所在的源，HCTL 只记实体键；用户用合适的客户端编辑那张卡照样可以，写发往卡实际所在的源，HCTL 不自动跨源同步；认领后，卡在外源的分组与它在 HCTL 的 Project 归属脱钩，各源看板与可选合并板的 Project 归属只认控制面自己的 Task 到 Project 记录，不去外源补建分组。

Task Revision 契约按需创建，但只能由显式“采纳契约”命令，或带已预览契约的“创建 Task”命令产生。无契约的“启动 Run”或“完成 Task”必须先要求该独立动作。没有契约的 Task 只有身份映射与操作投影，不进入治理；它在看板上的终态只是 content 投影。“完成 Task”不得在同一命令中隐式生成契约：预览必须要求先执行可审阅的“采纳契约”，再针对返回的精确 Revision 重新预览完成。

Task Revision 冻结验收契约，不冻结施工步骤；其不可变正文由控制面保存为治理材料，治理记录保存稳定身份、准入、精确定位与摘要和 current pointer，存取顺序见[系统边界](./system.md#控制面自己的存储)。

每条验收项必须声明**校验等级**：`mechanical`（机械可判——直报或旁路证据即可判定）、`gate`（需评审席位判）、`human`（需有权的人判）。缺等级的验收项使「采纳契约」预览失效。验收项用什么写法——EARS 句式、「必须有」清单、真值 / 工件 / 连线分类——归 Skill 与写作指引，约束不钉。后端与关联来源的变化先成为 Snapshot。只有会改变 Task Revision 契约的内容才形成待采纳；control 在正文已保存且摘要核验通过后，重新校验预期版本与权限，再准入新 Task Revision。content 后端拥有的操作字段按绑定与 Snapshot 投影，不经过采纳。

存在绑定该 Task 的非终态 Run 时，仍可“采纳契约”并推进 current Task Revision；活动 Run 已冻结的 Revision 不因此改写，Run 继续按冻结 Revision 执行。Run 正常完成路径只针对其冻结的 Revision。current 已前移时，Run 归约器的“完成 Task”按契约分歧拒绝，Task 保持开放并显示需要关注，不得静默按新 Revision 完成。

绑定 Task 的后端评论线是 Context 的萃取来源。组装器按当前 Snapshot 的引用和摘要把评论线冻结进 Context Manifest 并物化；交付方式见 [Project 约束](./project.md#context-memo-artifact)。评论线仍只能经“采纳契约”进入 Task Revision，物化本身不改变契约。

外部规范实体的身份仍是 `(provider, account_stable_id, external_entity_kind, immutable_external_entity_id)`；实体到 Task 的映射在**本控制面的每个 Project 内**持久唯一，即唯一性同时带 `project_id`。端口绑定、Source 范围或放置位置不进入实体身份；停用或重新绑定端口，或改变放置位置，都不释放或重定向本 Project 的映射。同一控制面的两个 Project 可以把各自的 Task 绑定到同一张外部卡，各自保有契约、Run、授权与完成凭证，不复制外部卡片，也不搬动另一 Project 的 Task。另一控制面也可独立认领（见下文多写通则实例）。键做身份、绑定做寻址：实体键识别卡片，读写这张卡经选定且有权访问该实体的端口绑定；同 provider 同账号不同 scope 可以各有活跃绑定，同一实体键就是同一张卡，不同 scope 的绑定只是寻址路径。

Task–Backend Binding 另行冻结可选的放置身份——`placement_scope_stable_id + external_board_item_id`——及其写入权。移动看板位置或更换看板项绑定不会在本 Project 产生第二个 Task，也不能改写规范实体身份。

Task 有两条可恢复的创建路径：

1. HCTL-first：携带初始契约时，先在事务外保存精确治理正文，再在准入事务核验并记录 Task、Task Revision、引用摘要、幂等结果和后端创建 outbox；不带契约时只固定 Task 身份与后端 outbox。
2. content-first：对账过程先保存 Snapshot，再认领唯一外部实体并创建无契约 Task。

两条路径都按同一关联键恢复。确认回执未知时，Task 保持开放，并显示待确认或待同步；系统必须按精确关联键和摘要回读，不得盲目重投，也不得另建卡片或 Task。正文保存与后端建卡分别恢复，适配器以关联键投递并回读；后端写入不与正文或控制面事务原子提交。content-first 对账以各 Project 已准入的源引用与分组映射为依据，满足上述无歧义映射才自动认领，其余只由人显式认领，适配器不能自行选择 Project。同一 Project 并发命中同一实体时只能复用同一 Task 或返回类型化冲突；另一 Project 已有 Task 不占用本 Project 的映射，也不自动成为本 Project 的承诺。仅查看卡片不认领，两个 Project 各自获准的认领则分别创建自己的 Task。

外部卡随后在源里移到另一分组、同时出现在多个分组或脱离原分组时，control 只追加 Snapshot：不改 Task 的 Project，不冻结采纳、启动、完成或字段写入——「偏离旧分组」本身不是冻结理由；仍冻结的只有依赖当前回读的动作（按[启动 Run 的前置与排序令牌](#启动-run-的前置与排序令牌)）与来源停用、卡被删除各自的规则。

系统不改变 Task 的 Project 归属，也不把不可变 `project_id` 改成新分组。“移动 Task”只改阶段与排序，不改 Project；跨源的相对移动拒绝。改变 Project 时，用户显式取消或保留旧 Task，并在目标 Project 创建新 Task，再用来源引用连接历史。迁往另一张卡（换家）不做：卡不搬家，卡所在的源挂了就是 content 后端挂了，ground truth 在那里。

**多写通则的任务后端实例**（[通则](./system.md#命令与跨服务正确性)）：另一 Project 或另一控制面把自己的 Task 绑到同一张卡，源上的标题、评论与关闭态是共享外部事实，各 Task 分别观测；一方的完成、取消、Run 与凭证不写入另一方。人经原生客户端做的 Done 可按各自绑定分别成为完成请求、各自验收，不等于两边都完成；任一 HCTL 自动写回不能冒充人的新动作，写回评论必须带控制面与 Task 标识。卡内容的并发写按任务源自己的能力处理，有条件写入的用，没有的以回读为准——未确认时不报成功，能确认的照常收口，不虚构并发保证；各 Project 各有一份 Task 绑定不产生第二张外部卡。

task_source 端口绑定与 Task–Backend Binding 的本地 current 投影使用 control 维护的单调 `state_version` 做比较并交换；Task Backend Snapshot 另行保存供应端的远端 revision、摘要和游标。采用外部来源内容的“采纳契约”命令必须让 Snapshot、字段权威策略与新 Task Revision 引用同一个 Task–Backend Binding，并把绑定版本、Snapshot、契约投影摘要和权威策略摘要一并写入 Task Revision。

采用本地 Room/Project 提案时，命令改为冻结精确 Project 来源引用、预期契约版本和提案摘要，不伪造 Task–Backend Binding。任一适用 current pointer 已变化时，预览失效。远端 revision 或摘要不能充当本地 `state_version`；本地版本也不能伪装成供应端的并发令牌。

字段写入权由 Task–Backend Binding 逐字段决定；契约、lifecycle 与完成凭证永远归控制面，不可配置：

| 模式 | 规则 |
| --- | --- |
| 后端权威（`backend_authoritative`） | 所选 content 后端拥有该字段（卡片、流转、排序、评论等操作字段默认如此），外部变化按 Snapshot 投影 |
| 控制面权威（`hctl_authoritative`） | 控制面拥有该字段（契约、判决与验收类字段），后端只接收写回 |
| 只读关联（`linked_readonly`） | 非后端的关联来源只形成快照、提案或需要关注 |

后端或关联来源的 Done/已关闭/Reopen/Deleted 是 content 事实，不会自动完成、重开、取消 HCTL Task，也不会停止 Run。删除只写 tombstone。

HCTL 对已有工作的缺省移除入口是“取消并归档”：按本模块的取消前置提交命令，成功后从活动列表收起，保留历史与源卡片，不增加 Task 生命周期状态。未提交草稿可以丢弃。删除源卡片是另一个有权用户显式确认的 content 动作；预览列出源、卡片、不可逆后果和活动 Run，存在活动 Run 时须明确其处理选择，不以删卡冒充停止。源删除确认未知时不报成功，已发生的外部删除按 Snapshot 处理，不清除 Task 历史。

所选 task backend 的事件还可以承载 human 命令请求，但只对 Task–Backend Binding 明确列明的动作生效。只允许一种动作归一为“完成 Task”命令草稿：已绑定的规范卡片由映射到归属 human 的账号从非终态进入 Done。

适配器必须固定绑定版本、规范外部实体 ID、Task ID、供应端 actor、变化前后值、远端 revision 或更新时间，以及可重复计算的幂等键，并在接纳前完成当前回读。Vikunja webhook 没有独立投递 ID 时，幂等键使用上述规范字段组，不把投递次数当身份。

HCTL 服务账号的写回、模型或 Harness、未知 actor、只看到当前 Done 而看不到一次明确变化，以及重复或迟到的旧 revision，都只追加 Snapshot，不能取得 human 来源。

control 对该完成请求执行与 Workbench/CLI 相同的预览和准入。只有预览不要求临场选择，而且绑定明确允许该供应端动作自动提交时，适配器才可以提交请求。否则，系统保留供应端 Done 与 HCTL 开放状态，并等待用户处理或返回类型化拒绝。

成功仍只由同一个“完成 Task”事务写 Task Completion Receipt。重开、取消和契约采纳没有供应端动作映射，必须使用公共命令入口；跨 Project 搬动既有 Task 不提供命令。

**Task 依赖**：卡与卡之间的阻塞与父子关系是任务源的原生语义，归源持有；HCTL 认得它但不另存：适配器从源读取，投影到卡片及其看板的四个只读字段——父卡、子卡、阻塞它的卡、它阻塞的卡；改关系走 content 写入通道，按后端能力写入、以回读为准，不走治理命令；Task 记录不另存依赖对象，跨源的依赖不设计。它只在一处进入治理：「启动 Run」的预览对绑定 Task 的卡列出源上未关闭的阻塞方，等人显式确认后才启动，不自动拒绝。源不提供依赖语义时字段为空，不阻拦。

## 写入约束

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 不可变结果或边界 |
| --- | --- | --- | --- |
| Task / Task Revision | contract version；开放 / 完成 / 已取消与独立 lifecycle version | control 处理「创建/采纳契约/完成/重开/取消 Task」命令 | Task Revision 只追加；Reopen 不改写旧完成历史 |
| 操作投影（Task–Backend Binding 字段组） | 绑定 `state_version`；后端并发前置按其能力使用 | control 准入「更新 Task」命令与「移动 Task」命令，经受控端口写 content 后端并回读；投影只由回读推进 | 不启动 Run，不改变 Task Revision 或 lifecycle |
| task_source 端口绑定 / Task–Backend Binding | current revision + local `state_version`；活跃 / 停用 / 已替换 | control 处理「接通/更新/停用」与「绑定/换绑」命令，adapter 只返回观测 | 历史 Revision 不改写；规范实体到 Task 的身份认领在本 Project 内持久唯一；家所在的源停用后 Task、历史与认领保留、标需要关注，人经「接通/换绑」恢复对同一实体的访问即恢复，Task 不换、不新建，恢复前依赖当前回读的动作拒绝，其余按冻结契约判 |
| Task Backend Snapshot | append-only sequence + remote revision/digest/cursor；可产生待采纳 | control 持久化 refresh/reconcile 观测；「采纳契约」命令才消费契约变化；同一 provider event 若满足上文条件，adapter 另行归一出完成 command draft | Snapshot、tombstone 和外部 lifecycle 不能直接写 Task |
| Task Completion Receipt | immutable | 只有成功的「完成 Task」命令事务可写 | 精确绑定该次 `task_lifecycle_version`、Task Revision 与证据 |

每个 Task 在控制面存储中至多有一个绑定 Run 的占用标记，状态为 `active | completion_pending`。“启动 Run”必须在创建 Run/Manifest 的同一用户级控制面事务中，以比较并交换把空标记推进为 `active`。已有任一标记时，同一幂等键返回原 Run，其他启动必须拒绝。

替代只能走 [Run 约束](./run.md#启动与-manifest)规定的原子撤权和换代路径，不能先清空标记再留下两个可写执行。`completion_pending` 期间也拒绝另一次启动，以及来自 human 的 Task 完成或取消命令；只接受匹配 Run 归约器的内部完成命令。该命令成功或被 Task 持久拒绝时，control 在同一结果事务中清除标记。

“完成 Task”命令必须先校验当前 Revision、验收规则、候选和全部必需证据，并逐项核对判定者与校验等级一致：`mechanical` 项只接受直报（`unmediated`）或旁路（`adapter_event`）证据，其中集成结果按契约接受 [Repo 模块](./repo.md#平台动作与命令)的 Integration Receipt，或由该模块回读核验的精确平台集成 Evidence；后者须已由契约事先声明接受，Task 不直接读平台，平台标签或自述不够；`gate` 项只接受 Gate Receipt 所含 Verdict，`human` 项只接受有权 human actor 的显式判定。验收策略可要求某项证据不低于某个证据通道等级（见 [Participant 约束](./participant.md#证据通道)）；等级不足时拒绝，转述不能补足。存在未采纳的契约变化时，actor 必须先采纳新 Revision，或在预览中明确选择按当前 Revision 完成；后一选择必须冻结当前绑定、来源头和全部未采纳 Snapshot。预览后出现的新 Snapshot 或变化必须使命令失效。“启动 Run”命令预览时的拒绝或延期不能代替这次选择。

接受外部集成事实的声明只经「采纳契约」或带预览契约的「创建 Task」形成精确 Task Revision，不由观测或完成命令补入。完成按当前 Task Revision 检查；活动 Run 仍冻结原 Revision，旧契约只接受自身 Integration Receipt 时不因新规则扩大。外部事实不自动完成 Task，不替代 Gate 或 human 验收项，也不补签本控制面的 Integration Receipt。

绑定该 Task 的非终态 Run 存在时，完成与取消命令都必须拒绝。用户必须先显式结束该 Run 并等待旧执行撤权、隔离；Task 命令不会隐式停止 Run。重开或取消必须保留旧 Receipt 和历史。

Task 终结只有两个获准 actor 来源：归属 human 的 Task 命令请求，或绑定精确 Task Revision 的 Run 正常完成后，由 Run 归约器和 control 提交的同一种“完成 Task”命令。human 请求可以来自 Workbench/CLI 的直接客户端连接，也可以来自上文已准入的供应端 Done 事件；两者生成同一命令信封，经过本段全部 Task 准入。

Run 路径使用由 Run/Task 身份派生的稳定幂等键。Run 已完成而 Task 校验失败时，Run 保持完成，Task 保持开放并显示需要关注。失败、已取消或被替代的 Run 不能完成或取消 Task。“取消 Task”命令只接受归属 human 的直接命令。这里的 Kanban 是动作语义而非某个窗口，客户端不产生权限等级。

Task Completion Receipt 至少固定 Task、“完成 Task”命令、Task Revision 引用与摘要和验收策略。每一条验收项还要分别固定通过或失败、校验等级与实际判定者（`hctl2-tool` / Gate 席位 / human actor）、Evidence/Verdict/Receipt 引用与摘要、来源 Snapshot、来源头或版本，以及适用的生产者与执行代次；不能用一个总括的“测试通过”替代逐项绑定。

若存在契约分歧，Receipt 还必须固定显式分歧选择、精确的未采纳 Snapshot 引用与摘要、Task–Backend Binding 版本与状态版本和权威策略摘要。Receipt、生命周期事件、current 投影、匹配的 `completion_pending` 占用标记清除和必要的外部写回 outbox 在同一事务提交。Run 路径若被 Task 拒绝，也在持久化拒绝结果与需要关注时清除同一标记。外部写回失败只显示需要关注，不撤销已经成立的 HCTL 完成事实。

冻结契约（Task Revision）与完成凭证是 Kanban 场景的结晶：Task Revision 正文属于控制面治理材料，身份、准入、摘要、当前指针与生命周期由治理记录维护；Task Completion Receipt 的权威在治理记录，审计副本在治理材料，公开范围不随代码仓库隐私自动推定。完整边界见[系统存储约束](./system.md#git-的双重角色)；施工图（Workflow Revision）从 Room 讨论中结晶、归 Room 场景，其对象与写入者归 [Run 模块约束](./run.md)。

「重开 Task」命令只接受有权 human actor，必须以预期 task_lifecycle_version 把完成/已取消 → 开放并推进版本；它不复活旧 Receipt。若当前来源契约已有未处理 drift，重开预览必须先采纳新 Task Revision 或显式冻结继续使用的当前 Revision 与 divergence，不能让外部 Reopen 或旧完成证明静默决定新一轮施工。

## 启动 Run 的前置与排序令牌

“启动 Run”命令预览必须列出会影响当前 Task Revision 的全部待采纳，并要求 actor 明确采纳、拒绝或延期。采纳会先产生新 Task Revision，再以新 Revision 重做“启动 Run”命令预览。拒绝或延期必须随准入冻结当前 Revision 和精确来源快照；未采纳的契约内容只作准入审计，不得进入 Task Revision、Run Manifest、Context Manifest 或 Execution Spec。

存在未处理的待采纳时不得启动 Run，control 也不得自动采纳或静默越过。只有“采纳契约”命令能让外部契约内容进入施工约束。后端权威（`backend_authoritative`）操作字段仍以当前 Snapshot 值和绑定版本作为启动的比较并交换前置，不能被拒绝或延期动作改写。

Start、Complete、Adopt 与跨来源冲突判断若要求 task backend 的当前 placement、remote revision、source head 或完整 cursor，必须先完成当前回读；后端不可用、cursor 有 gap 或 readback 超出冻结 freshness 上限时类型化拒绝。只有验收策略明确允许某项已缓存证据时，命令才可固定其观测版本、时间和已知 gap 继续；“后端离线”本身不放宽 Project group、drift 或 CAS 前置。

排序与位置永远归 content 后端。「移动 Task」命令冻结 Task–Backend Binding 的本地 state_version，经受控端口按该后端提供的写入语义写入并回读 Snapshot；来源刷新推进绑定 state_version，使旧预览失效。后端的并发控制是后端自己的事：adapter 按能力声明使用它有的前置（条件写入、排序令牌），没有就以回读为准。provider 原生客户端把卡片移入 Done 时，content 变化已经由后端完成；adapter 不把它伪装成 HCTL「移动 Task」命令，只按上文另行产生「完成 Task」请求。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL | 任务后端（Linear / GitHub / 本地任务服务器） | 差异 |
| --- | --- | --- |
| Task | Issue / 任务卡 | 后端卡片承载 content；Task 的身份、契约与验收由 HCTL 拥有 |
| 操作投影的 stage | Linear workflow state / GitHub ProjectV2 status | 谁拥有该字段由 Task–Backend Binding 逐字段决定 |
| 排序（rank） | Linear sortOrder / ProjectV2 排序 | 归后端；adapter 按后端能力用其条件写入，以回读为准 |
| 任务源（缺省建议：平台自带的 issues） | GitHub Issues 加 Projects V2 看板项；本地平台的 issues；本地任务服务器；Linear | 一张卡一个家；能否作缺省源看绑定的能力声明（建卡与字段写回）；条件写入有就用，没有以回读为准 |
| Task–Backend Binding 的 placement | GitHub ProjectV2 item；Linear 与本地平台的 issues 无独立看板项，位置由状态加 milestone、标签或 sortOrder 派生 | 实体身份与看板位置分离；移动位置不在同一 Project 产生第二个 Task |
| Task Backend Snapshot | webhook / API payload | 先观测后采纳；会改契约的内容必须经用户采纳 |
| 后端关闭态 | issue closed / 卡片终态 | 只是 content 事实，不等于验收完成 |
| Task Completion Receipt | 无对应 | HCTL 差异化语义：绑定精确契约与证据的完成证明 |

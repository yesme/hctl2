# Project 模块约束

> 状态：规范性约束 · 草案 v0.16.0<br>
> 本文是 Project 模块的约束附录，对象、状态机与写入者的唯一权威。设计正文见[Project 与 Chat Room](../project.md)；词汇分类与族规则见[总则](./README.md)；交接见[连接约束](./connections.md)。

## 对象

| 对象 | 含义 |
| --- | --- |
| Repo | Git 内容与共享配置的逻辑仓库 |
| Project | 具名目标、范围、角色、健康状态和长期交付物的稳定容器 |
| Project Role Binding | Project 角色到 Participant 候选的冻结绑定；Participant 本身由 [Participant 模块约束](./participant.md)定义 |
| Room | 持久协作空间的身份与治理事实：归属、名册、content 房间绑定、升格与来源关系；消息 content 的 ground truth 在 chat server |
| Chat 端口绑定 | Room 到 chat server 房间的 Resolved Port Binding。它固定外部账号与房间的稳定 ID、获准身份映射，以及可选的结构化 human 动作清单，并指明 Room content 的事实源。准入前置见[Room 与消息](#room-与消息) |
| Context Manifest / Context Bundle | 一次授权的根上下文清单，以及为某个消费执行实际物化并交付的消费上下文包 |
| Request | 向一个人或角色索取信息、授权或决定的一级对象 |
| Memo | 由用户明确提炼、预览、去敏并发布的稳定知识 |
| Artifact / Artifact Revision | 经 HCTL 登记的交付物身份及其不可变发布版本 |
| Room Invocation | 从 Room 发起的一次边界明确的 Harness 调用；其派发冻结由 [Execution Spec](./connections.md#project--run--participant从授权到物理执行) 承载 |

## 写入约束

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| Repo | stable `repo_id` + `repo_version`；待确认 / 活跃 | control 处理「注册 Repo」命令；工具箱只写入/回读 Git identity | 一个 Repo identity 只有一个 Repo 与 Repo Room；待确认的外部写入按关联键恢复，不重复注册 |
| Project | `project_version`；活跃 / 已归档 | control 处理「创建/更新/归档/恢复 Project」命令 | 已归档拒绝新 Task、Run 和写入型 Invocation；历史只读 |
| Project Role Binding | 绑定版本 | control 处理「绑定/换绑角色」命令 | 活动 Invocation/Run 永久引用准入时的绑定版本与 Participant revision |
| Room / 治理事件 | Room state version；活跃 / 只读 / 已归档；消息 content 由 chat server 承载 | 消息经 chat server 只追加（事务 ID 幂等）；control 只处理治理事件（升格、调用与 Request 关联）和 Scoped Room 的「创建/归档」命令，并以 chat server 事件 ID 精确引用消息 | chat server 时间线与治理事件账本都只追加；Project Room 随 Project 归档只读 |
| Chat 端口绑定 | immutable revision + current pointer；活跃 / 停用 / 已替换 | control 处理 Chat 端口绑定的「绑定/换绑/停用」命令，adapter 只投递/回读；「绑定/换绑」与 HCTL 自建房间的准入都以房间状态的当前回读证明目标房间未启用端到端加密 | 固定 Resolved Port Binding、外部 account/room stable IDs、身份映射策略、结构化 human 动作 allowlist 与降级能力；事后降级见[Room 与消息](#room-与消息) |
| Context Manifest / Context Bundle | immutable value + digest | Project control 按获准来源、scope、权限和预算物化；consumer 只读 | 后续 Room 消息、索引变化和 Harness 召回不能改写已冻结 Manifest/Bundle |
| Request | `request_version`；开放 / 已解决 / 已过期 / 已取消 / 被替代 | Project reducer/control 处理「创建/解决/取消」命令与 deadline | 终态不可复活；新问题创建新 Request |
| Room Invocation | `invocation_version`；待启动 / 运行中 / 等待输入 / 丢失 / 完成 / 失败 / 已取消 | Project reducer/control 处理「创建/取消/准入结果」命令，Agency 只提供观测 | 丢失和其他终态不可复活；重试创建新 Invocation |
| Memo | 发布 revision 只追加 | control 与工具箱处理「发布 Memo」命令 | 已发布内容不可改写；更新以 supersedes 连接新 revision |
| Artifact | `artifact_version`、current revision、活跃 / 已归档 | control 与工具箱处理「登记/发布/归档/恢复 Artifact」命令 | Artifact Revision 不可变，current pointer 只由 Publish 推进 |

## Repo 注册与 Project 归档

Repo 是逻辑仓库，不等于外部组织、工作区或仓库副本。注册命令固定 `repo_id`、预期 Git 身份、配置正文摘要和幂等键。control 先记录待确认注册和 outbox，工具箱再写入并回读 Git 身份。结果未知时只能按原身份和摘要恢复同一次注册；缺失或冲突必须要求用户处理，不得静默合并。

Git 已存在获准身份时，命令校验后复用它。注册确认事务激活唯一 Repo 身份并创建其唯一 Repo Room；待确认 Repo 不接受 Project、Task 或 Run。不同仓库副本通过[系统约束的显式现场挂接](./system.md#repo-与执行现场)连接同一 Repo，而不成为 Project 的子对象。

“创建 Project”命令在同一账本事务中创建该 Project 的唯一 Project Room。Room 身份与治理账本在用户级控制面；任何已挂接现场打开的都是同一个 Room，仓库副本只持有投影与现场操作状态，如草稿、未读和本地租约。进入 Project 默认打开该 Project Room。

Project Overview 是 Project 场景内按单个 Project 聚合目标、健康度、Task、Run、Request、Artifact/SCM/CI 和近期活动的只读投影，不是第五个场景或可写状态。Workbench 可以另行把同源 Request 和健康状态投影聚合为全局需要关注。

“归档 Project”要求系统进入静止状态。预览与提交都必须确认：不存在非终态 Run、非终态写入型 Room Invocation、活动输入或写租约，也不存在该 Project 所有且仍为待投递或结果未知的外部副作用。前置不满足时，系统必须列出阻塞项并拒绝命令。

开放 Task、开放 Request 与未归档 Scoped Room 不阻止归档。它们随 Project 一并转为只读，不被隐式完成或取消；恢复 Project 后保持原状态。仍在运行的只读 Invocation 也不阻止归档，其迟到结果按既有规则只留审计。

成功事务把 Project 与 Project Room 置为只读，并拒绝新的 Task、Run、Request、Artifact 发布与写入型 Invocation。恢复命令只恢复 Project 与 Project Room 接收新命令的资格，不复活历史 Task、Run、Invocation、Request、Scoped Room、租约或外部副作用。

Project 的目标、范围、角色和默认规则以单调 project_version 更新。创建 Task、Run 或 project_scope Room Invocation 时必须冻结获准的 Project version 与相关策略摘要；repo_scope Room Invocation 改为冻结 Repo Instance/repo/base 且只能只读。后续 Project 更新不改写已经接受的下游约束。

Project Role Binding 授予某个 Participant 在当前 Project 中的职责、权限和预算上限；Participant revision、Skill 与 Worker Profile 的定义见 [Participant 模块约束](./participant.md)。Execution Spec 必须分别冻结四者的精确引用。人不是 Participant：人的角色与权限由 human actor 的命令权限表达，不经角色绑定。

换绑不改写活动 Invocation、Seat 或 Run；`repo_scope` 可以没有 Project Role Binding。

从 Repo Room 创建 Project 时，先提供可编辑、可删减补充和去敏的提升预览，再提交「创建 Project」命令；该命令只能显式选择来源 Message 引用和/或已预览的 Context Manifest/Context Bundle 摘要，并冻结所选内容的可追溯来源链。Project 只保存这些引用和经确认的名称、目标、范围等创建字段；不得复制整段 Room、把隐式聊天窗口当作来源，或让后续 Room 消息改变既有 Project。父 Room 的滚动纪要（若有）可作为提升预览的预填材料；被采纳的部分同样以显式选择进入来源链，纪要本身不随子概念活体继承。

## Room 与消息

创建 Scoped Room 时必须冻结 parent Room、讨论目标、完成条件和回填动作。达到完成条件不会自动修改目标。归档只允许两条路径：回填动作成功，或有权 human actor 显式以 abandoned、no-decision 或 superseded 结案并记录理由。回填失败时，Room 和目标引用必须保留为可恢复状态。

Message 是只追加的协作事实，其 ground truth 在 chat server（Matrix 协议：编辑与撤回是新事件）；修正、删除和外部编辑形成新事件或 tombstone，不能抹掉已被引用的历史。

时间线顺序由 chat server 的线性事件顺序给出；这是单 homeserver 的约束前提，写入以事务 ID 保持幂等。稳定 ID、时间戳和 Invocation 完成顺序只用于身份或展示。HCTL 治理事件在控制面账本只追加，以 Chat 端口绑定和 chat server 事件 ID 精确引用消息。被治理引用的消息在引用时冻结事件 ID 与内容摘要，此后 content 分歧不改写已冻结引用。

冻结摘要、Context 萃取与桥接可读都以 control 能按事件 ID 读取明文正文为前提，因此 HCTL 创建或绑定的房间不启用端到端加密。chat server 不可用，或绑定后房间被开启端到端加密时，不依赖新消息、当前成员或新游标的 metadata 命令可以继续；依赖当前消息正文、成员身份或完整游标的命令必须类型化拒绝。聊天入口分别显示重同步中或需要关注，不能用缓存冒充当前事实。加密情形由有权 human actor 换绑到未加密房间恢复；已冻结的引用与摘要不受影响。

<a id="context-memo-artifact"></a>
## Context、Memo 与 Artifact

Context Bundle 是调用开工时交付给执行体的输入包，不代管执行体在会话中自行组织的工作上下文。

### 三种交付方式

- `inline`：直接物化原文。它只用于执行体无法自行读取或不适合自行翻找的内容，包括相关聊天、Task 评论、契约与范围说明、用户显式引用，以及策略要求必用的同 Run 前序结果。必用内容超出预算时，必须降为 `pointer` 并附分片建议，不得静默丢弃。
- `pointer`：只交付精确引用、摘要和一句说明。它只能指向执行体在获准范围内可自行打开的 Git 对象或 worktree 路径；账本和任务后端内容不得作为 `pointer`。
- `recall`：运行期间按召回策略追加的子包条目。

Gate Seat 的 ReviewSubjectRef diff 与返工 Seat 的 Verdict 正文属于必用的同 Run 前序结果，详细规则见 [Run 模块约束](./run.md#request重试与-gate)。

### 选材与排序

来源按以下顺序选择：用户显式引用；当前讨论窗口；Repo、Project、Task、Run 和 Request 的引用；Git 中的 Artifact、Verdict、Receipt 等结晶副本；必需 Skill；相关 Memo。绑定 Task 的任务后端评论线整条属于显式来源，以当前 Task Source Snapshot 的引用与摘要冻结进 Manifest，不经检索。序列化时，稳定内容排在前面，高频变化内容排在后面。

### 根 Context Manifest

每次顶层授权必须冻结一个根 Manifest，并包含：

- `context_manifest_id` 与 purpose/scope；
- 可选 parent Manifest 引用；
- 每个实际来源的稳定引用及 version/digest；
- selection-policy version；
- freshness、coverage 和 known gaps；
- 必需 Skill 的引用与 digest；
- permission、redaction 和 budget；
- `manifest_digest`。

Repo Room、Project Room 和 Run 之间只能通过这些 parent/source 引用传承 Context。搜索索引、`current` 指针和“最近消息”不能替代精确来源。

每个消费者都从根 Manifest 物化独立的 Context Bundle。Bundle 必须记录自身 ID、Manifest 引用、消费归属者及其精确版本或代次、按序条目及摘要、renderer/tokenizer/redaction 版本、压缩记录、交付计量、权限与预算、保留策略和 `bundle_digest`。

压缩记录包含压缩模型引用与摘要、压缩率和原文引用与摘要。交付计量包含候选、实选与实际交付的 token 估算量，以及实际交付的字节摘要。

Execution Spec 同时冻结根 Manifest 与该消费者的 Bundle，control 在派工交付前核对实际交付摘要。Bundle 内容至少保留到归属者终态，且保留策略定义的 Result Proposal 准入窗口关闭。此后可以丢弃明文，但必须保留位置、摘要、来源链、策略版本和丢弃事实，不得声称仍可重放。

后续 Room 消息、索引变化、Harness 自行召回或另一消费者的 Bundle 都不能改写已冻结记录。

萃取与相关性判定默认全部在本地完成，不消耗大模型 token。全文索引与可选相关性门都是可重建的派生投影，由 chat server 事件流、Task Source Snapshot 与账本增量维护，不进入权威账本；删除后可以完整重建。相关性门默认只以账本事实——提及、认领、Request 关联和游标——作为判定输入。

用户配置专用的小模型（small-brain）后，相关性门才可以读取消息正文并使用模型辅助判定；该模型必须引用用户级定义机制中的精确 revision 和 digest。无论采用哪种方式，每次判定都必须把输入事实引用与结论记为可审计观测；模型判定还要记录模型引用与摘要。观测不改写任何事实。

压缩缺省关闭。仅当用户配置了专用压缩模型时，Bundle 物化才可压缩；该模型不是新对象，而是经用户级定义机制固定 revision/digest 的 small-brain 引用。每个被压缩条目必须记录压缩模型引用与摘要、压缩率和原文引用与摘要；压缩产物的每个片段都必须能回到原文位置。

摘要、Receipt、验收标准原文和被治理引用冻结的消息原文属于证据类内容，永不压缩。压缩条目缺少来源记录，或压缩了证据类内容时，Bundle 必须拒绝交付。萃取与压缩产物可以作为以（Room、游标区间、消费者范围）为键的派生缓存跨调用复用；Bundle 记录所引用产物的引用与摘要，缓存可丢弃重建。

房间可以维护一份滚动纪要（前情提要）：它挂在 Room 与游标上，由组装器机械触发，并经 small-brain 增量折叠成派生缓存。未配置 small-brain 时不生成纪要；物化端改用近详远略裁剪，近期消息保留全文，更早消息降为标题加事件指针。

纪要逐条携带消息事件回源指针。它不是权威：治理引用不得指向纪要，只能指向精确事件；纪要不进权威账本，被使用时 Bundle 只记其引用与摘要，也不由房间内模型 Participant 书写或改写。

Memo 只由用户明确发布，至少固定 `memo_id`、来源 Message/Artifact refs、适用范围、作者、内容 digest/Git locator、取代关系和有效期。原始消息、执行日志和自动总结不会自动进入长期知识。组装的指针清单机械过滤已过有效期或已被取代的 Memo；显式引用不受此过滤。

Artifact 是 Project/Repo 中可引用、评审和交付的稳定身份；普通 Git 文件在登记前不是 Artifact。Artifact Revision 至少固定 `artifact_revision_id`、artifact_id、不可变内容位置、内容摘要、可选 ChangeSet Revision 来源和 revision_digest。

Artifact 的评审对象对 {artifact_revision_id, artifact_id, immutable_content_locator, content_digest, source_change_set_revision_ref?} 使用[共享摘要规则](./system.md#命令与跨服务正确性)生成独立 review_subject_digest；它不能与完整 Revision digest 互换。发布新版本只移动 current pointer，不改写历史。

## Room Invocation

Room Invocation 适合一次性的研究、比较或范围明确的写入。它可以持有一份 Execution Spec 和可选 Harness 运行时，但没有持久 DAG、候选自动切换、Gate 或自动后继；需要这些能力时则创建 [Run](./run.md)。

Room Invocation 的合法边只有待启动 → 运行中/失败/已取消/丢失、运行中 ↔ 等待输入，以及运行中/等待输入 → 完成/失败/已取消/丢失。执行身份无法证明时进入丢失；撤销租约、提交停止与隔离 outbox、迟到结果只留审计等动作由[连接约束的统一丢失处理规则](./connections.md#失败与恢复)定义一次，本模块不复述。

迟到流或 Result Proposal 不能准入语义结果，也不能附着到新调用。用户重试必须在旧授权失效后创建新的 Room Invocation、Execution Spec、运行时代次和必要的 ChangeSet，并保留原调用引用；系统不能重放或复活旧调用。

Room Invocation 的 Execution Spec 先固定范围：`repo_scope` 只读，`project_scope` 才能携带写入与 ChangeSet 规则。human 批准建议时，Spec 还必须固定来源建议、建议摘要、可选父执行、扇出位置和预期 Room/Project version。

若建议来自 Result Proposal，还必须逐项校验归属者、Execution Spec、绑定、Context Bundle 和物理执行代次；`in_process` 仅使用连接约束定义的缩减字段组。

上述字段的完整格式见[连接约束定义的共同字段](./connections.md#project--run--participant从授权到物理执行)。来源建议必须精确引用 chat server 事件 ID 或 Result Proposal；父执行必须精确引用 Room Invocation 或 Attempt。新执行体的载荷不能改写这些来源链字段。

Repo Room 可以在没有 Project 的情况下做只读研究；写入、Project Artifact 或 Project 范围权限必须选择精确 Project 与版本。

## Request

当执行需要输入时，拥有该阻塞事实的模块向 Project 提交类型化 Request 创建命令；Project 独占 Request lifecycle。解决 Request 必须经过预览和类型化动作；control 在一个事务中以比较并交换校验 Request 与来源 blocker，并写唯一 delivery outbox，来源模块只在匹配确认回执/观测后推进精确阻塞范围。开放式商议可以升级为 Scoped Room，但讨论结论仍需由有权 actor 提交原动作。

Request 的完整跨模块字段约束只在[连接约束](./connections.md#跨模块-request-回路)定义；本模块不另建一套同义字段。活动 Request 的问题、目标人或角色、归属者与受影响 revision、阻塞范围和归属者状态版本、去重根和获准解决动作不得原地修改。Attempt 另带 `attempt_generation`，Room Invocation 使用 `invocation_version`。

上述阻塞身份相同的重复创建必须去重到现有活动 Request，可以追加提醒事件。任一归属者、版本、范围或所需动作变化时，control 必须创建新 Request 并取代旧 Request；旧解决结果不得推进新阻塞项。

Request 的应答面按需升级：默认在卡片或详情中直接回答；需要多轮论述、多位 Participant 或共同编辑时才升级为 Scoped Room；涉及密钥等敏感内容时走安全输入通道，不进入普通消息、trace 或回放；只有诊断或接管精确执行时才连接终端。每一级应答面都绑定同一个 Request 与其阻塞范围，不创建平行事实。

## 场景约束

mention 提交前的 Trigger Preview 必须显示实际 Participant/Worker Profile/Harness、required/optional Skills、Context 来源与 token 估算、权限与写入范围、预算，以及将创建 Room Invocation/Run/Request 还是唤醒多个 worker。

普通 Room 的临场执行边只能由可稳定归属到 human 的动作在 Trigger Preview 后提交。动作可以来自 Workbench/CLI 的直接客户端连接，也可以来自 Chat 端口绑定明确允许的供应端结构化事件；两者都归一为同一命令草稿。系统按[系统约束](./system.md#客户端动作与-provider-事件)保留 actor 映射、来源事件、目标版本和幂等依据。chat server 里的普通消息本身不是入口。

模型 Participant 的 Message、Result Proposal、总结及其正文中的 `@` 只能形成下一位 Participant/Role 与扇出建议，不能自行创建 Room Invocation、唤醒执行体或递归委派。用户批准建议后，系统自动把原消息、稳定引用、Context Manifest、权限、预算和父 Invocation 关系带入新预览；系统不能要求用户复制粘贴 Context。

mention 的解析必须确定性：`@` 目标只按获准的 Participant/Role 绑定精确解析；无唯一授权候选时必须明确失败或要求人选择，不得按显示名模糊匹配、静默换人或把 mention 字符串交给模型猜测路由。

类型化命令的预览、准入与判决在控制面执行，结果可作为结构化事件写回 chat server；Chat provider 动作成为 human 命令请求的条件见[客户端动作与 provider 事件](./system.md#客户端动作与-provider-事件)。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL 词 | 外部体系 | 一句话差异 |
| --- | --- | --- |
| Room | Matrix room / Slack channel | HCTL Room 身份与治理在控制面；明文准入与事后降级见[Room 与消息](#room-与消息) |
| 消息 | Matrix event | 消息 content 本体就是 chat server 上的 Matrix event（编辑/撤回是新事件；非 Matrix 平台的消息经 homeserver 桥接生态落为 Matrix event）；HCTL 治理事件只在控制面账本追加，以事件 ID 精确引用消息，不占领域对象名额 |
| mention | @mention | HCTL 的 `@` 解析目标是逻辑 Participant/Role 而非平台账号，且必须经 Trigger Preview 准入 |
| Scoped Room | thread / 子频道 | 差异：有冻结的讨论目标与结论回填动作，不是自由分叉 |
| Chat 端口绑定 | AppService 注册 / homeserver 配置 | 差异：绑定指认 Room 的 content 家；chat server 拥有消息历史，但不拥有 Room 身份与治理；非 Matrix 平台桥接是 homeserver 生态的事，不是 HCTL 端口 |
| Participant | 平台成员 / bot 账号 | 差异：Participant 是逻辑档案，外部账号只是映射之一 |
| Request | 无直接对应 | 差异化语义：向指定人/角色索取输入的一级对象，只能由获准动作解决 |

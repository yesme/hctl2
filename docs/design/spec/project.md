# Project 模块合同

> 状态：规范性合同 · 草案 v0.14.1<br>
> 本文是 Project 模块的合同附录，对象、状态机与写入者的唯一权威。设计正文见[Project 与 Chat Room](../project.md)；词汇分类与族规则见[总则](./README.md)；交接见[连接合同](./connections.md)。

## 对象

| 对象 | 含义 |
| --- | --- |
| Repo | Git 内容与共享配置的逻辑仓库 |
| Project | 具名目标、范围、角色、健康状态和长期交付物的稳定容器 |
| Participant / Project Role Binding | 可寻址的逻辑参与者，以及 Project 角色到 Participant/Harness 候选的冻结绑定 |
| Room | 持久协作空间的身份与治理事实：归属、名册、content 房间绑定、升格与来源关系；消息 content 的 ground truth 在 chat server |
| Chat 端口绑定 | Room 到 chat server 房间的 Resolved Port Binding，及外部 account/room stable IDs 与获准身份映射策略（含经 homeserver 桥接接入的外部平台用户）；Room 的 content 家由它指认。被指认的房间对 control 明文可读——不启用端到端加密（Matrix `m.room.encryption`），这是绑定的准入前置 |
| Context Manifest / Context Bundle | 一次授权的根来源清单，以及为某个消费执行实际物化并交付的内容包 |
| Request | 向一个人或角色索取信息、授权或决定的一级对象 |
| Memo | 由用户明确提炼、预览、去敏并发布的稳定知识 |
| Artifact / Artifact Revision | 经 HCTL 登记的交付物身份及其不可变发布版本 |
| Room Invocation | 从 Room 发起的一次边界明确的 Harness 调用；其派发冻结由 [Execution Spec](./connections.md#project--run--agent从授权到物理执行) 承载 |

## 写入合同

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| Repo | stable `repo_id` + `repo_version`；待确认 / 活跃 | control 处理「注册 Repo」命令；工具箱只写入/回读 Git identity | 一个 Repo identity 只有一个 Repo 与 Repo Room；待确认的外部写入按关联键恢复，不重复注册 |
| Project | `project_version`；活跃 / 已归档 | control 处理「创建/更新/归档/恢复 Project」命令 | 已归档拒绝新 Task、Run 和写入型 Invocation；历史只读 |
| Participant / Project Role Binding | Participant immutable revision + current pointer；binding version | control 处理「创建/更新 Participant」与「绑定/换绑角色」命令 | 活动 Invocation/Run 永久引用准入时的 Participant/binding revision |
| Room / 治理事件 | Room state version；活跃 / 只读 / 已归档；消息 content 由 chat server 承载 | 消息经 chat server 只追加（事务 ID 幂等）；control 只处理治理事件（升格、调用与 Request 关联）和 Scoped Room 的「创建/归档」命令，并以 chat server 事件 ID 精确引用消息 | chat server 时间线与治理事件账本都只追加；Project Room 随 Project 归档只读 |
| Chat 端口绑定 | immutable revision + current pointer；活跃 / 停用 / 已替换 | control 处理 Chat 端口绑定的「绑定/换绑/停用」命令，adapter 只投递/回读；「绑定/换绑」与 HCTL 自建房间的准入都以 fresh 房间状态回读证明目标房间未启用端到端加密 | 固定 Resolved Port Binding、外部 account/room stable IDs、身份映射策略与降级能力；health、成员现状、同步 cursor 与加密状态观测是另行版本化的运行投影——绑定后房间被开启加密不改写历史 binding，只把该 Room 的聊天入口标为需要关注 |
| Context Manifest / Context Bundle | immutable value + digest | Project control 按获准来源、scope、权限和预算物化；consumer 只读 | 后续 Room 消息、索引变化和 Harness 召回不能改写已冻结 Manifest/Bundle |
| Request | `request_version`；开放 / 已解决 / 已过期 / 已取消 / 被替代 | Project reducer/control 处理「创建/解决/取消」命令与 deadline | 终态不可复活；新问题创建新 Request |
| Room Invocation | `invocation_version`；待启动 / 运行中 / 等待输入 / 丢失 / 完成 / 失败 / 已取消 | Project reducer/control 处理「创建/取消/准入结果」命令，Agency 只提供观测 | 丢失和其他终态不可复活；重试创建新 Invocation |
| Memo | 发布 revision 只追加 | control 与工具箱处理「发布 Memo」命令 | 已发布内容不可改写；更新以 supersedes 连接新 revision |
| Artifact | `artifact_version`、current revision、活跃 / 已归档 | control 与工具箱处理「登记/发布/归档/恢复 Artifact」命令 | Artifact Revision 不可变，current pointer 只由 Publish 推进 |

## Repo 注册与 Project 归档

Repo 不等于外部组织、工作区或某个 clone。「注册 Repo」命令固定新 `repo_id`、预期 Git identity、repo 配置正文 digest 与幂等键；control 先在账本持久化待确认注册与工具箱 outbox，工具箱再把稳定 identity 写入受跟踪的 Repo 配置并按关联键回读。写入或 ACK 结果未知时 Repo 保持待确认，恢复只能按 identity/digest 回读并完成同一注册，不能再生成一个 Repo 或 Repo Room；Git 已存在获准 identity 时，命令校验后复用它。缺失、冲突或仅有 remote URL 相似都不得静默合并。注册确认事务激活唯一 Repo 身份并创建其唯一 Repo Room；待确认 Repo 不接受 Project/Task/Run。不同 clone 通过[系统合同的显式现场挂接](./system.md#repo-与执行现场)连接同一 Repo，而不成为 Project 的子对象。

「创建 Project」命令在同一账本事务创建该 Project 的唯一 Project Room：一个 Project 一个 Room，Room 身份与治理账本在用户级控制面，任何已挂接现场打开的都是同一个 Room；clone 只持有投影与现场操作态（草稿、未读、本地租约）。进入 Project 默认打开该 Project Room。Project Overview 是 Project 场景内按单个 Project 聚合目标、健康度、Task、Run、Request、Artifact/SCM/CI 和近期活动的只读投影，不是第五个场景或可写状态；Workbench 可以另行把同源 Request/health 投影聚合为全局需要关注。

「归档 Project」是 quiescent transition，预览与提交都必须确认：不存在开放 Task、非终态 Run/Room Invocation、未归档 Scoped Room、开放 Request、活动输入/写租约，或该 Project 所有且仍为待投递/结果未知的外部副作用。归档不隐式完成或取消这些对象；前置不满足就列出 blocker 并拒绝。成功事务把 Project 与 Project Room 置为只读，并拒绝新的 Task、Run、Request、Artifact 发布与写入型 Invocation。Restore 只恢复 Project 与 Project Room 的接收新命令资格，不复活历史 Task、Run、Invocation、Request、Scoped Room、租约或外部副作用。

Project 的目标、范围、角色和默认规则以单调 project_version 更新。创建 Task、Run 或 project_scope Room Invocation 时必须冻结获准的 Project version 与相关策略摘要；repo_scope Room Invocation 改为冻结 Repo Instance/repo/base 且只能只读。后续 Project 更新不改写已经接受的下游合同。

Participant 使用稳定 `participant_id` 与不可变配置 revision；该 revision 描述逻辑身份、persona/沟通约束、可选 post-train 或模型资格约束、默认 Skill refs 和 Worker Profile 候选约束，但不携带 secret、Project 权限或运行时身份。Project Role Binding 再把 Project/role 固定到精确 Participant revision，并保存职责、候选约束及权限/预算上限；Skill revision 只是另行冻结的方法包，Worker Profile 则只是物理 Harness/model/runtime 候选。四者不能互相替代：显示名、外部账号、persona、Skill、Worker Profile、Harness session 或模型名都不能替代 Participant ID，也不能自行授予角色或权限。换版或换绑不改写活动 Invocation、Seat 或 Run；每次 Execution Spec 必须同时固定实际 Participant revision、Project Role Binding version/digest（repo_scope 可无）与实际 Skill refs/digests。

从 Repo Room 创建 Project 时，先提供可编辑、可删减补充和去敏的提升预览，再提交「创建 Project」命令；该命令只能显式选择来源 Message 引用和/或已预览的 Context Manifest/Context Bundle 摘要，并冻结所选内容的可追溯来源链。Project 只保存这些引用和经确认的名称、目标、范围等创建字段；不得复制整段 Room、把隐式聊天窗口当作来源，或让后续 Room 消息改变既有 Project。父 Room 的滚动纪要（若有）可作为提升预览的预填材料；被采纳的部分同样以显式选择进入来源链，纪要本身不随子概念活体继承。

## Room 与消息

Scoped Room 创建时必须冻结 parent Room、精确讨论目标（Request 或待提交的类型化动作）、完成条件和结论回填动作。达到讨论完成条件本身不修改目标；只有获准的回填动作成功后才能归档，失败时保留可恢复的讨论和目标引用。

Message 是只追加的协作事实，其 ground truth 在 chat server（Matrix 协议：编辑与撤回是新事件）；修正、删除和外部编辑形成新事件或 tombstone，不能抹掉已被引用的历史。普通回复、表情或模型总结不会修改 Project、解决 Request 或发布 Artifact。

时间线顺序由 chat server 的线性事件顺序给出（单 homeserver 合同前提，写入以事务 ID 幂等）；稳定 ID、时间戳和 Invocation 完成顺序只用于身份或展示。HCTL 治理事件在控制面账本只追加，以 Chat 端口绑定 + chat server 事件 ID 精确引用消息；被治理引用的消息（升格来源、Context 锚点）在引用时冻结事件 ID 与内容 digest，此后 content 漂移不改写已冻结引用。冻结 digest、Context 萃取与桥接可读都以 control 能按事件 ID 读取明文正文为前提，因此 HCTL 创建或绑定的房间不启用端到端加密（第二个合同前提，与单 homeserver 并列）。chat server 不可用，或绑定后房间被开启端到端加密时，不依赖新消息、当前成员或新 cursor 的 metadata 命令可以继续；需要 fresh message body、成员身份或完整 cursor 才能准入的命令类型化拒绝，聊天入口分别显示重同步中或需要关注，不能用缓存冒充当前事实；加密情形由有权 human actor 「换绑」到未加密房间恢复，已冻结的引用与 digest 不受影响。

<a id="context-memo-artifact"></a>
## Context、Memo 与 Artifact

Context 交付的是调用开工时给执行体的 prompt，不代管执行体在会话内自行组装的工作上下文。Bundle 的每个条目按投喂档记录为 inline / pointer / recall 之一。inline 物化原文，只用于执行体自己拿不到或不该自己翻的部分——从聊天史与绑定 Task 的任务后端评论线萃取的相关讨论、契约与范围说明、用户显式引用的原文，以及冻结策略列为必用的同 Run 前序节点结果（Gate Seat 的 ReviewSubjectRef diff、返工 Seat 的 Verdict 正文，见 [Run 模块合同](./run.md#request重试与-gate)）；必用条目超预算时改为 pointer 并附分片建议，不得静默丢弃。pointer 只记精确 ref+digest 与一句摘要，且只能指向执行体在获准范围内以自身工具可打开的位置——Git 对象与 worktree 路径：Repo/Git 内容、ChangeSet Revision、Artifact、Memo、Skill 与 Verdict/Receipt 的 Git 结晶副本；指向账本或任务后端的引用不得作为 pointer 交付。recall 是运行中经 recall policy 追加的子包条目。选择优先级固定为：显式引用 → 当前讨论窗口 → Repo/Project/Task/Run/Request 引用 → Git/Artifact/Verdict-Receipt 结晶副本指针 → 必需 Skill 指针 → 相关 Memo 指针；序列化以稳定内容在前、高频变动在后。一次顶层授权先冻结一个根 Context Manifest，至少包含 `context_manifest_id`、purpose/scope、可选 parent manifest refs、每个实际来源的 stable ref + version/digest、selection-policy version、freshness/coverage/known gaps、required Skill refs/digests、permission/redaction/budget 约束和 `manifest_digest`。Repo Room → Project Room → Run 的传承只能通过这些显式 parent/source 引用发生；搜索索引、`current` 指针或“最近消息”不能替代它们。

每个 Room Invocation 或 Attempt 消费者再从根 Manifest 物化自己的 Context Bundle；Bundle 至少固定 `context_bundle_id`、Manifest ref+digest、consumer owner ref + 精确 owner version/attempt generation、按序 materialized item refs/digests、renderer/tokenizer/redaction versions、逐项压缩记录（若压缩：compressor 模型 ref+revision/digest、压缩率与原文 ref+digest）、选材计量（候选/实选/实际交付 token 估算量）、实际交付 bytes digest、已应用的权限/预算、retention-policy ref/version 与 `bundle_digest`。Execution Spec 同时冻结根 Manifest 与该消费者 Bundle，control 在派工交付前核对实际交付 digest。Bundle 内容至少保留到 owner 终态且该 retention policy 定义的 Result Proposal 准入窗口关闭；之后允许丢弃明文，但必须保留 locator/digest、来源链、policy version 和丢弃事实，不得声称仍可 replay。后续 Room 消息、索引变化、Harness 自行召回或另一消费者的 Bundle 都不能改写已冻结记录。

萃取与相关性判定全部本地，不消耗大模型 token：第一级是显式引用与当前讨论窗口，绑定 Task 的任务后端评论线整条属于这一级——以当前 Task Source Snapshot 的 ref+digest 冻结进 Manifest，不经检索；第二级的全文索引与第三级的可选相关性门都是可重建派生投影——从 chat server 事件流、task_source Snapshot 与账本增量维护，不进权威账本，删除后可完整重建。相关性门只以账本事实（提及、认领、Request 关联、游标）为判定输入，不以消息措辞正文做路由；每次判定连同输入事实引用与结论记为可审计观测，观测不改写任何事实。

压缩缺省关闭。仅当用户配置了专用压缩模型（small-brain——经用户级定义机制固定 revision/digest 的模型引用，不是新对象）时，Bundle 物化才可压缩。每个被压缩条目必须记录 compressor ref+digest、压缩率与原文 ref+digest，且压缩产物的每个片段可回源到原文位置。证据类内容——digest、Receipt、验收标准原文与被治理引用冻结的消息原文——永不压缩；压缩条目缺来源记录或压缩了证据类内容的 Bundle 拒绝交付。萃取与压缩产物可作为以（room、cursor 区间、消费者范围）为键的派生缓存跨调用复用；复用时 Bundle 记录所引产物的 ref+digest，缓存可丢弃重建。

房间可维护一份滚动纪要（前情提要）：挂在（room、cursor）上、由组装器机械触发并经 small-brain 增量折叠的派生缓存。未配置 small-brain 时不生成纪要，物化端以近详远略裁剪代替（近期消息全文、更早消息降为标题加事件指针）。纪要逐条携带消息事件回源指针；它不是权威——治理引用不得指向纪要，只能指向精确事件；不进权威账本，被使用时 Bundle 只记其 ref+digest；不由房间内模型 Participant 书写或改写。

Memo 只由用户明确发布，至少固定 `memo_id`、来源 Message/Artifact refs、适用范围、作者、内容 digest/Git locator、取代关系和有效期。原始消息、执行日志和自动总结不会自动进入长期知识。组装的指针清单机械过滤已过有效期或已被取代的 Memo；显式引用不受此过滤。

Artifact 是 Project/Repo 中可引用、评审和交付的稳定身份；普通 Git 文件在登记前不是 Artifact。Artifact Revision 至少固定 `artifact_revision_id`、artifact_id、不可变内容定位、内容摘要、可选 ChangeSet Revision 来源和 revision_digest。Artifact 的评审 subject 对 {artifact_revision_id, artifact_id, immutable_content_locator, content_digest, source_change_set_revision_ref?} 使用[共享摘要规则](./system.md#命令与跨服务正确性)生成独立 review_subject_digest；它不能与完整 Revision digest 互换。发布新版本只移动 current pointer，不改写历史。

## Room Invocation

Room Invocation 适合一次性的研究、比较或范围明确的写入。它可以持有一份 Execution Spec 和可选 Harness 运行时，但没有持久 DAG、候选自动切换、Gate 或自动后继；需要这些能力时应创建 [Run](./run.md)。

Room Invocation 的合法边只有待启动 → 运行中/失败/已取消/丢失、运行中 ↔ 等待输入，以及运行中/等待输入 → 完成/失败/已取消/丢失。执行身份无法证明时进入丢失；撤销租约、提交 stop/fence outbox、迟到结果只留审计等动作由[连接合同的统一丢失处理规则](./connections.md#失败与恢复)定义一次，本模块不复述。迟到流或 Result Proposal 不能准入语义结果或附着到新调用。用户 Retry 必须在旧授权失效后创建新的 Room Invocation、Execution Spec、runtime generation 和必要的 ChangeSet，并保留原调用引用，不能重放或复活旧调用。

Room Invocation 的 Execution Spec 除[连接合同定义的共同字段](./connections.md#project--run--agent从授权到物理执行)外，还固定 scope（repo_scope | project_scope）与 human 批准 Agent 建议时的 lineage 字段：精确 source_suggestion_ref = 消息事件（chat server 事件 ID）| Result Proposal、建议摘要、可选 parent_execution_ref = Room Invocation | Attempt 与获准 fan-out 位置，并以预期 Room/Project version 和通用幂等键提交；Result Proposal 分支还要逐项匹配其 owner invocation_version、control writer generation、spec/binding/Context Bundle digests，以及物理执行时的 Execution Runtime/runtime_generation 与 site/backend fence generations。`in_process` 只能使用连接合同明确的缩减 tuple。这些 lineage 字段不能由新 worker 的 payload 改写。scope 中 Repo Room 可以在没有 Project 的情况下做只读研究；写入、Project Artifact 或 Project-scoped 权限必须选择精确 Project/version，且只有 project_scope 可以携带 ChangeSet 规则。

## Request

当执行需要输入时，拥有该阻塞事实的模块向 Project 提交类型化 Request 创建命令；Project 独占 Request lifecycle。解决 Request 必须经过预览和类型化动作；control 在一个事务中 CAS Request 与来源 blocker，并写唯一 delivery outbox，来源模块只在匹配 ACK/观测后推进精确阻塞范围。开放式商议可以升级为 Scoped Room，但讨论结论仍需由有权 actor 提交原动作。

Request 的完整跨模块字段合同只在[连接合同](./connections.md#跨模块-request-回路)定义；本模块不另建一套同义字段。活动 Request 的问题、目标人或角色、`owner_ref + affected_revision_ref + blocked_scope + owner state_version`（Attempt 另带 attempt_generation，Room Invocation 使用 invocation_version）、dedupe root 和获准解决动作不得原地修改。上述阻塞身份相同的重复创建必须去重到现有活动 Request，可以追加提醒事件；任一 owner/version/scope 或所需动作变化时必须创建新 Request 并 Supersede 旧 Request，旧解决结果不得推进新 blocker。

Request 的应答面按需升级：默认在卡片或详情中直接回答；需要多轮论述、多位 Participant 或共同编辑时才升级为 Scoped Room；涉及密钥等敏感内容时走安全输入通道，不进入普通消息、trace 或回放；只有诊断或接管精确执行时才连接终端。每一级应答面都绑定同一个 Request 与其阻塞范围，不创建平行事实。

## 场景合同

mention 提交前的 Trigger Preview 必须显示实际 Participant/Worker Profile/Harness、required/optional Skills、Context 来源与 token 估算、权限与写入范围、预算，以及将创建 Room Invocation/Run/Request 还是唤醒多个 worker。

普通 Room 的临场执行边只能由经过认证的 human actor 在 Trigger Preview 后提交；human 来自经认证的场景客户端会话——Workbench、CLI 或按公开合同适配的第三方场景客户端；chat server 里的消息本身不是入口。模型 Participant 的 Message、Result Proposal、总结及其正文中的 `@` 只可形成下一位 Participant/Role 与 fan-out 建议，不能自行创建 Room Invocation、唤醒 worker 或递归委派。用户批准建议后，系统自动把原消息、稳定引用、Context Manifest、权限、预算和父 Invocation 关系带入新预览，不能要求人复制粘贴 Context。

mention 的解析必须确定性：`@` 目标只按获准的 Participant/Role 绑定精确解析；无唯一授权候选时必须明确失败或要求人选择，不得按显示名模糊匹配、静默换人或把 mention 字符串交给模型猜测路由。

命令走 HCTL，记录落平台：类型化命令的预览、准入与判决都在控制面执行；结果可以作为结构化事件写回 chat server 供时间线展示，但平台里的记录只是记录——chat server 中的任何消息、反应、成员动作或自动化都不是命令，不能触发派发、解决 Request 或改变治理事实。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL 词 | 外部体系 | 一句话差异 |
| --- | --- | --- |
| Room | Matrix room / Slack channel | HCTL Room 身份与治理在控制面；它的 content 房间就是 chat server 上的 Matrix room，且不带 `m.room.encryption` 状态事件（绑定时校验，事后出现即需要关注） |
| 消息 | Matrix event | 消息 content 本体就是 chat server 上的 Matrix event（编辑/撤回是新事件；非 Matrix 平台的消息经 homeserver 桥接生态落为 Matrix event）；HCTL 治理事件只在控制面账本追加，以事件 ID 精确引用消息，不占领域对象名额 |
| mention | @mention | HCTL 的 `@` 解析目标是逻辑 Participant/Role 而非平台账号，且必须经 Trigger Preview 准入 |
| Scoped Room | thread / 子频道 | 差异：有冻结的讨论目标与结论回填动作，不是自由分叉 |
| Chat 端口绑定 | AppService 注册 / homeserver 配置 | 差异：绑定指认 Room 的 content 家；chat server 拥有消息历史，但不拥有 Room 身份与治理；非 Matrix 平台桥接是 homeserver 生态的事，不是 HCTL 端口 |
| Participant | 平台成员 / bot 账号 | 差异：Participant 是逻辑档案，外部账号只是映射之一 |
| Request | 无直接对应 | 差异化语义：向指定人/角色索取输入的一级对象，只能由获准动作解决 |

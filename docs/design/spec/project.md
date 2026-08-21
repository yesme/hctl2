# Project 模块合同

> 状态：规范性合同 · 草案 v0.11.1<br>
> 本文是 Project 模块的合同附录，对象、状态机与写入者的唯一权威。设计正文见[Project 与 Chat Room](../project.md)；词汇分类与族规则见[总则](./README.md)；交接见[连接合同](./connections.md)。

## 对象

| 对象 | 含义 |
| --- | --- |
| Repo | Git 内容与共享配置的逻辑仓库 |
| Repo Instance | 某个 clone/worktree 集合中的代码侧物理边界：worktree、ChangeSet 现场、运行时与单写锁；协作与治理事实不住在 clone 里 |
| Project | 具名目标、范围、角色、健康状态和长期交付物的稳定容器 |
| Participant / Project Role Binding | 可寻址的逻辑参与者，以及 Project 角色到 Participant/Harness 候选的冻结绑定 |
| Room | 持久协作空间的身份与治理事实：归属、名册、content 房间绑定、升格与来源关系；消息 content 的 ground truth 在 chat server |
| Chat 端口绑定 | Room 到 chat server 房间（及非 Matrix 桥接面）的 Resolved Port Binding，及外部 account/room/thread stable IDs、成员映射、去重游标的字段组；Room 的 content 家由它指认 |
| Context Manifest / Context Bundle | 一次调用所用来源、筛选、摘要、Skill 与权限的可解释快照 |
| Request | 向一个人或角色索取信息、授权或决定的一级对象 |
| Memo | 由用户明确提炼、预览、去敏并发布的稳定知识 |
| Artifact / Artifact Revision | 经 HCTL 登记的交付物身份及其不可变发布版本 |
| Room Invocation | 从 Room 发起的一次边界明确的 Harness 调用；其派发冻结由 [Execution Spec](./connections.md#project--run--agent从授权到物理执行) 承载 |

## 写入合同

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| Repo Instance | immutable repo identity + local writer generation | control 处理「初始化 Repo Instance」命令；core 校验 Git identity | 同一 git-common-dir 只在 metadata 账本注册一个物理现场身份，重试返回原 identity；clone 本地不设账本 |
| Project | `project_version`；活跃 / 已归档 | control 处理「创建/更新/归档/恢复 Project」命令 | 已归档拒绝新 Task、Run 和写入型 Invocation；历史只读 |
| Participant / Project Role Binding | Participant immutable revision + current pointer；binding version | control 处理「创建/更新 Participant」与「绑定/换绑角色」命令 | 活动 Invocation/Run 永久引用准入时的 Participant/binding revision |
| Room / Room Event | Room state version；活跃 / 只读 / 已归档；消息 content 由 chat server 承载 | 消息经 chat server 只追加（事务 ID 幂等）；control 只处理治理事件（升格、调用与 Request 关联）和 Scoped Room 的「创建/归档」命令，并以 chat server 事件 ID 精确引用消息 | chat server 时间线与治理事件账本都只追加；Project Room 随 Project 归档只读 |
| Chat 端口绑定 | immutable revision + current pointer；活跃 / 停用 / 已替换 | control 处理 Chat 端口绑定的「绑定/换绑/停用」命令，adapter 只投递/回读 | 固定 Resolved Port Binding、外部 account/thread stable IDs、成员映射、去重 cursor 与降级能力 |
| Context Manifest / Context Bundle | immutable value + digest | Project control 按获准来源、scope、权限和预算物化；consumer 只读 | 后续 Room 消息、索引变化和 Harness 召回不能改写已冻结 Manifest/Bundle |
| Request | `request_version`；开放 / 已解决 / 已过期 / 已取消 / 被替代 | Project reducer/control 处理「创建/解决/取消」命令与 deadline | 终态不可复活；新问题创建新 Request |
| Room Invocation | `invocation_version`；待启动 / 运行中 / 等待输入 / 中断 / 完成 / 失败 / 已取消 | Project reducer/control 处理「创建/取消/准入结果」命令，agentd 只提供观测 | 中断和其他终态不可复活；重试创建新 Invocation |
| Memo | 发布 revision 只追加 | control/core 处理「发布 Memo」命令 | 已发布内容不可改写；更新以 supersedes 连接新 revision |
| Artifact | `artifact_version`、current revision、活跃 / 已归档 | control/core 处理「登记/发布/归档/恢复 Artifact」命令 | Artifact Revision 不可变，current pointer 只由 Publish 推进 |

Repo 不等于外部组织或工作区。「创建 Project」命令在同一事务创建该 Project 的唯一 Project Room：一个 Project 一个 Room，Room 身份与治理账本在用户级控制面，任何 clone 打开的都是同一个 Room；clone 只持有投影与现场操作态（草稿、未读、本地租约）。Project Archive 使其只读，Restore 恢复活跃。进入 Project 默认打开该 Project Room。Project Overview 是 Project 场景内按单个 Project 聚合目标、健康度、Task、Run、Request、Artifact/SCM/CI 和近期活动的只读投影，不是第五个场景或可写状态；Workbench 可以另行把同源 Request/health 投影聚合为全局需要关注。

Project 的目标、范围、角色和默认规则以单调 project_version 更新。创建 Task、Run 或 project_scope Room Invocation 时必须冻结获准的 Project version 与相关策略摘要；repo_scope Room Invocation 改为冻结 Repo Instance/repo/base 且只能只读。后续 Project 更新不改写已经接受的下游合同。

Participant 使用稳定 `participant_id` 与不可变配置 revision；Project Role Binding 把 Project/role 固定到精确 Participant revision，并保存职责、候选约束及权限/预算上限。显示名、外部账号、persona、Worker Profile、Harness session 或模型名都不能替代 Participant ID；换版或换绑不改写活动 Invocation、Seat 或 Run。Participant 配置的进一步设计保留在专题 memo，在进入规范前不得成为第二套权限或委派来源。

从 Repo Room 创建 Project 时，先提供可编辑、可删减补充和去敏的提升预览，再提交「创建 Project」命令；该命令只能显式选择来源 Message 引用和/或已预览的 Context Manifest/Context Bundle 摘要，并冻结所选内容的可追溯来源链。Project 只保存这些引用和经确认的名称、目标、范围等创建字段；不得复制整段 Room、把隐式聊天窗口当作来源，或让后续 Room 消息改变既有 Project。

## Room 与消息

Scoped Room 创建时必须冻结 parent Room、精确讨论目标（Request 或待提交的类型化动作）、完成条件和结论回填动作。达到讨论完成条件本身不修改目标；只有获准的回填动作成功后才能归档，失败时保留可恢复的讨论和目标引用。

Message 是只追加的协作事实，其 ground truth 在 chat server（Matrix 协议：编辑与撤回是新事件）；修正、删除和外部编辑形成新事件或 tombstone，不能抹掉已被引用的历史。普通回复、表情或模型总结不会修改 Project、解决 Request 或发布 Artifact。

时间线顺序由 chat server 的线性事件顺序给出（单 homeserver 合同前提，写入以事务 ID 幂等）；稳定 ID、时间戳和 Invocation 完成顺序只用于身份或展示。HCTL 治理事件在控制面账本只追加，以 Chat 端口绑定 + chat server 事件 ID 精确引用消息；被治理引用的消息（升格来源、Context 锚点）在引用时冻结事件 ID 与内容 digest，此后 content 漂移不改写已冻结引用。chat server 不可用时治理命令照常执行，聊天入口安全降级。

## Context、Memo 与 Artifact

Context 组装顺序固定为：显式引用 → 当前讨论窗口 → Project/Task/Run/Request → Git/Artifact/Receipt → 必需 Skill → 相关 Memo。Room Invocation 的 Execution Spec 冻结最终 Context Manifest、逻辑参与者、Harness 候选、能力和权限；之后的 Room 消息不会偷偷改变已确认调用。

Memo 只由用户明确发布，至少固定 `memo_id`、来源 Message/Artifact refs、适用范围、作者、内容 digest/Git locator、取代关系和有效期。原始消息、执行日志和自动总结不会自动进入长期知识。

Artifact 是 Project/Repo 中可引用、评审和交付的稳定身份；普通 Git 文件在登记前不是 Artifact。Artifact Revision 至少固定 `artifact_revision_id`、artifact_id、不可变内容定位、内容摘要、可选 ChangeSet Revision 来源和 revision_digest。Artifact 的评审 subject 对 {artifact_revision_id, artifact_id, immutable_content_locator, content_digest, source_change_set_revision_ref?} 使用[共享摘要规则](./system.md#命令与跨服务正确性)生成独立 review_subject_digest；它不能与完整 Revision digest 互换。发布新版本只移动 current pointer，不改写历史。

## Room Invocation

Room Invocation 适合一次性的研究、比较或范围明确的写入。它可以持有一份 Execution Spec 和可选 Harness 运行时，但没有持久 DAG、候选自动切换、Gate 或自动后继；需要这些能力时应创建 [Run](./run.md)。

Room Invocation 的合法边只有待启动 → 运行中/失败/已取消/中断、运行中 ↔ 等待输入，以及运行中/等待输入 → 完成/失败/已取消/中断。恢复对账无法证明原 session/process 身份及 lease/generation 仍匹配时，control 在同一收口事务将其置为中断、撤销输入/写租约并提交旧 runtime 的 stop/fence outbox；其迟到流或 Result Proposal 只留审计，不能准入语义结果或附着到新调用。用户 Retry 必须在旧授权失效后创建新的 Room Invocation、runtime generation 和必要的 ChangeSet，并保留原调用引用，不能重放或复活旧调用。

Room Invocation 的 Execution Spec 除[连接合同定义的共同字段](./connections.md#project--run--agent从授权到物理执行)外，还固定 scope（repo_scope | project_scope）与 human 批准 Agent 建议时的 lineage 字段：精确 source_suggestion_ref = Room Event/Message | Result Proposal、建议摘要、可选 parent_execution_ref = Room Invocation | Attempt 与获准 fan-out 位置，并以预期 Room/Project version 和通用幂等键提交；Result Proposal 分支还要逐项匹配其 owner/generation。这些 lineage 字段不能由新 worker 的 payload 改写。scope 中 Repo Room 可以在没有 Project 的情况下做只读研究；写入、Project Artifact 或 Project-scoped 权限必须选择精确 Project/version，且只有 project_scope 可以携带 ChangeSet 规则。

## Request

当执行需要输入时，拥有该阻塞事实的模块向 Project 提交类型化 Request 创建命令；Project 独占 Request lifecycle。解决 Request 必须经过预览和类型化动作；control 在一个事务中 CAS Request 与来源 blocker，并写唯一 delivery outbox，来源模块只在匹配 ACK/观测后推进精确阻塞范围。开放式商议可以升级为 Scoped Room，但讨论结论仍需由有权 actor 提交原动作。

Request 的完整跨模块字段合同只在[连接合同](./connections.md#跨模块-request-回路)定义；本模块不另建一套同义字段。活动 Request 的问题、目标人或角色、`owner_ref + affected_revision_ref + blocked_scope + owner generation/state_version + dedupe root` 和获准解决动作不得原地修改。上述阻塞身份相同的重复创建必须去重到现有活动 Request，可以追加提醒事件；任一 owner/version/scope 或所需动作变化时必须创建新 Request 并 Supersede 旧 Request，旧解决结果不得推进新 blocker。

Request 的应答面按需升级：默认在卡片或详情中直接回答；需要多轮论述、多位 Participant 或共同编辑时才升级为 Scoped Room；涉及密钥等敏感内容时走安全输入通道，不进入普通消息、trace 或回放；只有诊断或接管精确执行时才连接终端。每一级应答面都绑定同一个 Request 与其阻塞范围，不创建平行事实。

## 场景合同

mention 提交前的 Trigger Preview 必须显示实际 Participant/Worker Profile/Harness、required/optional Skills、Context 来源与 token 估算、权限与写入范围、预算，以及将创建 Room Invocation/Run/Request 还是唤醒多个 worker。

普通 Room 的临场执行边只能由经过认证的 human actor 在 Trigger Preview 后提交；human 可以来自 Workbench、CLI 或适配后的外部 Chat 场景，但消息来源必须映射为人的 principal provenance。模型 Participant 的 Message、Result Proposal、总结及其正文中的 `@` 只可形成下一位 Participant/Role 与 fan-out 建议，不能自行创建 Room Invocation、唤醒 worker 或递归委派。用户批准建议后，系统自动把原消息、稳定引用、Context Manifest、权限、预算和父 Invocation 关系带入新预览，不能要求人复制粘贴 Context。

mention 的解析必须确定性：`@` 目标只按获准的 Participant/Role 绑定精确解析；无唯一授权候选时必须明确失败或要求人选择，不得按显示名模糊匹配、静默换人或把 mention 字符串交给模型猜测路由。

命令走 HCTL，记录落平台：类型化命令的预览、准入与判决都在控制面执行；结果可以作为结构化事件写回 chat server 供时间线展示，但平台里的记录只是记录——chat server 中的任何消息、反应、成员动作或自动化都不是命令，不能触发派发、解决 Request 或改变治理事实。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL 词 | 外部体系 | 一句话差异 |
| --- | --- | --- |
| Room | Matrix room / Slack channel | HCTL Room 身份与治理在控制面；它的 content 房间就是 chat server 上的 Matrix room |
| Room Event | Matrix event / Slack message | Matrix event 就是消息 content 本体（chat server 承载，编辑/撤回是新事件）；Slack 这类可原地编辑的平台经桥接按新事件或 tombstone 落账；HCTL 治理事件以事件 ID 精确引用消息 |
| mention | @mention | HCTL 的 `@` 解析目标是逻辑 Participant/Role 而非平台账号，且必须经 Trigger Preview 准入 |
| Scoped Room | thread / 子频道 | 差异：有冻结的讨论目标与结论回填动作，不是自由分叉 |
| Chat 端口绑定 | Matrix bridge / Slack app 安装 | 差异：绑定指认 Room 的 content 家；chat server 拥有消息历史，但不拥有 Room 身份与治理；非 Matrix 桥接只投递与回读 |
| Participant | 平台成员 / bot 账号 | 差异：Participant 是逻辑档案，外部账号只是映射之一 |
| Request | 无直接对应 | 差异化语义：向指定人/角色索取输入的一级对象，只能由获准动作解决 |

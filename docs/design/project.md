# Project 与 Chat Room

> 本文是 Project 模块的唯一领域权威；Chat Room 是其操作合同，不拥有独立领域事实。模块交接见[连接合同](./connections.md)，通用机制见[系统边界](./system.md)。

## 为什么存在

Harness 的会话、终端和 worktree（Git 工作树）都会结束或被替换；Project 的目标、论证、Participant（参与者）关系、来源和未决问题却必须继续存在。Project 模块保存的正是这份长期事实，Chat Room 则是所有 Harness 都消失之后仍然可以恢复的协作现场——它回答“我们要解决什么、为什么、依据是什么，以及哪些讨论已经足够稳定，可以成为承诺”。

以 Room 为中心不等于所有工作都必须聊天：Kanban、Workflow 图和 Terminal 各有自己的操作面。Room 的特殊地位在于承载目标塑形的连续性（shaping continuity），而不是承载所有机械执行事件。

Project 也不是施工管线：研究、规格说明、ADR（架构决策记录）和纯文档的 Project 可以从未创建 Run。Project 不预配常驻的“包工头”Agent；Participant 是可寻址的逻辑档案，只有显式调用才产生有边界的执行。

## 模块职责

Project 模块保存“为什么做、依据是什么、谁在参与”的长期事实。它不等于仓库、聊天串、Task 集合、Run、Harness 会话或 worktree。

| 对象 | 含义 |
| --- | --- |
| Repo | Git 内容与共享配置的逻辑仓库 |
| RepoInstance | 某个 clone/worktree 集合中的本地控制边界；拥有独立 SQLite 和 control writer |
| Project | 具名目标、范围、角色、健康状态和长期交付物的稳定容器 |
| Participant / ProjectRoleBinding | 可寻址的逻辑参与者，以及 Project 角色到 Participant/Harness 候选的冻结绑定 |
| Room | 持久协作空间，保存消息、引用、调用、Request 和来源关系 |
| ChatSurfaceBindingRevision | Room 到外部 Chat 端口的不可变身份、能力、路由与游标绑定 |
| ContextManifest / ContextBundle | 一次调用所用来源、筛选、摘要、Skill 与权限的可解释快照 |
| Request | 向一个人或角色索取信息、授权或决定的一级对象 |
| Memo | 由用户明确提炼、预览、去敏并发布的稳定知识 |
| Artifact / ArtifactRevision | 经 HCTL 登记的交付物身份及其不可变发布版本 |
| RoomInvocation / InvocationBinding | 从 Room 发起的一次边界明确的 Harness 调用及其冻结绑定 |

## 写入合同

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| RepoInstance | immutable repo identity + local writer generation | control 处理 InitRepoInstanceIntent；core 校验 Git identity | 同一 git-common-dir 只建立一个本地账本身份，重试返回原 identity |
| Project | `project_version`；`Active / Archived` | control 处理 Create/Update/Archive/Restore Project Intent | Archived 拒绝新 Task、Run 和写入型 Invocation；历史只读 |
| Participant / ProjectRoleBinding | Participant immutable revision + current pointer；binding version | control 处理 Create/Update Participant 与 Bind/Rebind ProjectRole Intent | 活动 Invocation/Run 永久引用准入时的 Participant/binding revision |
| Room / RoomEvent | Room state version；`Active / ReadOnly / Archived`；事件有 `room_sequence` | control 处理 AppendRoomEvent、Create/ArchiveScopedRoom Intent | RoomEvent 只追加；Project Room 随 Project 归档只读 |
| ChatSurface binding | immutable revision + current pointer；`Active / Disabled / Replaced` | control 处理 Bind/Rebind/Disable ChatSurface Intent，adapter 只投递/回读 | 固定 ResolvedPortBinding、外部 account/thread stable IDs、成员映射、去重 cursor 与降级能力 |
| ContextManifest / ContextBundle | immutable value + digest | Project control 按获准来源、scope、权限和预算物化；consumer 只读 | 后续 Room 消息、索引变化和 Harness 召回不能改写已冻结 Manifest/Bundle |
| Request | `request_version`；`Open / Resolved / Expired / Cancelled / Superseded` | Project reducer/control 处理 Create/Resolve/Cancel Intent 与 deadline | 终态不可复活；新问题创建新 Request |
| RoomInvocation | `invocation_version`；`Pending / Running / WaitingForInput / Interrupted / Completed / Failed / Cancelled` | Project reducer/control 处理 Create/Cancel/AdmitResult，agentd 只提供观测 | Interrupted 和其他终态不可复活；重试创建新 Invocation |
| Memo | 发布 revision 只追加 | control/core 处理 PublishMemoIntent | 已发布内容不可改写；更新以 supersedes 连接新 revision |
| Artifact | `artifact_version`、current revision、`Active / Archived` | control/core 处理 Register/Publish/Archive/Restore Artifact Intent | ArtifactRevision 不可变，current pointer 只由 Publish 推进 |

Repo 不等于外部组织或工作区。CreateProjectIntent 在同一事务创建当前 RepoInstance 的唯一 Project Room；Project Archive 使其 ReadOnly，Restore 恢复 Active。另一个 clone 对同一 Project 有自己的 Project Room 投影和本地操作账本。进入 Project 默认打开该 Project Room。Project Overview 是 Project 场景内按单个 Project 聚合目标、健康度、Task、Run、Request、Artifact/SCM/CI 和近期活动的只读投影，不是第五个场景或可写状态；Workbench 可以另行把同源 Request/health 投影聚合为全局 Needs Attention。

Project 的目标、范围、角色和默认规则以单调 `project_version` 更新。创建 Task、Run 或 `project_scope` RoomInvocation 时必须冻结获准的 Project version 与相关策略摘要；`repo_scope` RoomInvocation 改为冻结 RepoInstance/repo/base 且只能只读。后续 Project 更新不改写已经接受的下游合同。

Participant 使用稳定 `participant_id` 与不可变配置 revision；ProjectRoleBinding 把 Project/role 固定到精确 Participant revision，并保存职责、候选约束及权限/预算上限。显示名、外部账号、persona、WorkerProfile、Harness session 或模型名都不能替代 Participant ID；换版或换绑不改写活动 Invocation、Seat 或 Run。Participant 配置的进一步设计保留在专题 memo，在进入规范前不得成为第二套权限或委派来源。

从 Repo Room 创建 Project 时，先提供可编辑、可删减补充和去敏的提升预览，再提交 `CreateProjectIntent`；Intent 只能显式选择来源 Message 引用和/或已预览的 ContextManifest/ContextBundle 摘要，并冻结所选内容的可追溯来源链。Project 只保存这些引用和经确认的名称、目标、范围等创建字段；不得复制整段 Room、把隐式聊天窗口当作来源，或让后续 Room 消息改变既有 Project。

## Room 类型

| Room | 作用 | 生命周期 |
| --- | --- | --- |
| Repo Room | 无固定主题的研究、发现和 Project 入口 | 与 RepoInstance 同寿命 |
| Project Room | 围绕一个 Project 的长期协作和里程碑 | Project 归档后只读 |
| Scoped Room | 为复杂 Request 或决定临时建立的讨论空间 | 结论回填类型化动作后归档 |

Scoped Room 创建时必须冻结 parent Room、精确讨论目标（Request 或待提交的类型化动作）、完成条件和结论回填动作。达到讨论完成条件本身不修改目标；只有获准的回填动作成功后才能归档，失败时保留可恢复的讨论和目标引用。

Message 是只追加的协作事实；修正、删除和外部编辑形成新事件或 tombstone，不能抹掉已被引用的历史。普通回复、表情或模型总结不会修改 Project、解决 Request 或发布 Artifact。

## Context、Memo 与 Artifact

Context 组装顺序固定为：显式引用 → 当前讨论窗口 → Project/Task/Run/Request → Git/Artifact/Receipt → 必需 Skill → 相关 Memo。InvocationBinding 冻结最终 ContextManifest、逻辑参与者、Harness 候选、能力和权限；之后的 Room 消息不会偷偷改变已确认调用。

Memo 只由用户明确发布，至少固定 `memo_id`、来源 Message/Artifact refs、适用范围、作者、内容 digest/Git locator、取代关系和有效期。原始消息、执行日志和自动总结不会自动进入长期知识。

Artifact 是 Project/Repo 中可引用、评审和交付的稳定身份；普通 Git 文件在登记前不是 Artifact。ArtifactRevision 至少固定 `artifact_revision_id`、`artifact_id`、不可变内容定位、内容摘要、可选 ChangeSetRevision 来源和 `revision_digest`。Artifact 的评审 subject 对 `{artifact_revision_id, artifact_id, immutable_content_locator, content_digest, source_change_set_revision_ref?}` 使用[共享摘要规则](./system.md#命令与跨服务正确性)生成独立 `review_subject_digest`；它不能与完整 Revision digest 互换。发布新版本只移动 current pointer，不改写历史。

## RoomInvocation 与 Request

RoomInvocation 适合一次性的研究、比较或范围明确的写入。它可以持有一个 InvocationBinding 和可选 Harness 运行时，但没有持久 DAG、候选自动切换、Gate 或自动后继；需要这些能力时应创建 [Run](./run.md)。

InvocationBinding 的 scope 是 `repo_scope | project_scope`：Repo Room 可以在没有 Project 的情况下做只读研究；写入、Project Artifact 或 Project-scoped 权限必须选择精确 Project/version。RoomInvocation 的合法边只有 `Pending → Running/Failed/Cancelled/Interrupted`、`Running ↔ WaitingForInput`，以及 `Running/WaitingForInput → Completed/Failed/Cancelled/Interrupted`。恢复对账无法证明原 session/process 身份及 lease/generation 仍匹配时，control 在同一收口事务将其置为 Interrupted、撤销输入/写租约并提交旧 runtime 的 stop/fence outbox；其迟到流或 ResultProposal 只留审计，不能准入语义结果或附着到新调用。用户 Retry 必须在旧授权失效后创建新的 RoomInvocation、runtime generation 和必要的 ChangeSet，并保留原调用引用，不能重放或复活旧调用。

InvocationBinding 还固定 invocation generation、ContextManifest、逻辑 Participant、WorkerProfile、Harness/Runtime ResolvedPortBinding、repo/base、能力与权限、预算/截止和 binding digest；只有 project_scope 可以携带 ChangeSet 规则。由 human 批准 Agent 建议而创建的调用还必须固定精确 `source_suggestion_ref = RoomEvent/Message | ResultProposal`、建议摘要、可选 `parent_execution_ref = RoomInvocation | Attempt` 与获准 fan-out 位置，并以预期 Room/Project version 和通用幂等键提交；ResultProposal 分支还要逐项匹配其 owner/generation。这些 lineage 字段不能由新 worker 的 payload 改写。

当执行需要输入时，拥有该阻塞事实的模块向 Project 提交类型化 Request 创建命令；Project 独占 Request lifecycle。解决 Request 必须经过预览和类型化动作；control 在一个事务中 CAS Request 与来源 blocker，并写唯一 delivery outbox，来源模块只在匹配 ACK/观测后推进精确阻塞范围。开放式商议可以升级为 Scoped Room，但讨论结论仍需由有权 actor 提交原动作。

Request 的应答面按需升级，不是每个问题都要开一个房间：默认在卡片或详情里直接回答；需要多轮论述、多位 Participant 或共同编辑时才升级为 Scoped Room；涉及密钥等敏感内容时走安全输入通道，不进入普通消息、trace 或回放；只有诊断或接管精确执行时才连接终端。每一级都绑定同一个 Request 与阻塞范围，不创建平行事实。

Request 的完整跨模块字段合同只在[连接合同](./connections.md#跨模块-request-回路)定义；本模块不另建一套同义字段。活动 Request 的问题、目标人或角色、`owner_ref + affected_revision_ref + blocked_scope + owner generation/state_version + dedupe root` 和获准解决动作不得原地修改。上述阻塞身份相同的重复创建必须去重到现有活动 Request，可以追加提醒事件；任一 owner/version/scope 或所需动作变化时必须创建新 Request 并 Supersede 旧 Request，旧解决结果不得推进新 blocker。

## Chat Room 场景

Chat Room 是 Project 的主要操作场景，提供：

- Room 源事件与单调 `room_sequence` 在同一事务提交；时间线按该序号排序，稳定 ID、时间戳和 Invocation 完成顺序只用于身份或展示；
- `@` Participant/Role、`/` 类型化动作、`$` Skill、`#` 文件/Artifact/消息引用；
- 并发 RoomInvocation 的独立流、取消和结果卡；
- Request、Project 概览、Task/Run 里程碑和 Needs Attention 投影；
- mention 提交前的 Trigger Preview，必须显示实际 Participant/WorkerProfile/Harness、required/optional Skills、Context 来源与 token 估算、权限与写入范围、预算，以及将创建 RoomInvocation/Run/Request 还是唤醒多个 worker；
- Context 预览、Memo/Artifact 发布预览和权限说明。

在 Workbench 里同时管理多个仓库时，一个 Room 可以把另一个仓库 Room 的 Participant 阵容借用为预填选择，不必逐个重选。借用只是预填：Participant 与角色绑定仍在本 RepoInstance 内重新准入，权限、预算和绑定不跨仓库继承；将来若要沉淀为可共享的一等对象，再另行设计。

普通 Room 的临场执行边只能由经过认证的 human actor 在 Trigger Preview 后提交；human 可以来自 Workbench、CLI 或适配后的外部 Chat 场景，但消息来源必须映射为人的 principal provenance。Agent-authored Message、ResultProposal、模型总结及其正文中的 `@` 只可形成下一位 Participant/Role 与 fan-out 建议，不能自行创建 RoomInvocation、唤醒 worker 或递归委派。用户批准建议后，系统自动把原消息、稳定引用、ContextManifest、权限、预算和父 Invocation 关系带入新预览，不能要求人复制粘贴 Context。重复且无需临场判断的协作边应进入 [Workflow](./run.md)，由 reducer 只按冻结 WorkflowRevision 创建。

| 角色 | 可以做什么 | 不能做什么 |
| --- | --- | --- |
| 场景客户端：Workbench Room | 提供完整时间线、Composer、预览和命令入口 | 直接写 SQLite 或把渲染动作当成领域结果 |
| 场景客户端：CLI | 查询 Room/Request；第一阶段复杂编辑安全暂停 | 绕过预览、版本或权限检查 |
| 受控端口 / 原生客户端：外部聊天平台 | 在能力允许时投递/接收同一 Room 的消息与 Request | 以外部 thread/message ID 取代 Project/Room 身份 |

外部聊天桥接不是第一阶段出门条件；一旦交付，必须具备稳定身份、去重、回声抑制、outbox、重连和降级能力。

## 模块交接

以下只列所有权方向；字段、事务与故障语义由[四模块连接合同](./connections.md)统一定义。

- Project 中的提案只有通过采纳命令才会产生 [TaskRevision](./task.md)。
- Project 可以发起一次 RoomInvocation；持久自动施工必须显式创建 [Run](./run.md)。
- Task、Run 和 Harness 的状态只以投影或引用回到 Chat Room，不能由聊天反向改写。
- 稳定经验通过 Memo 回流；交付内容通过 ArtifactRevision 回流。

## 不可破坏的边界

- Room 只能形成提案，不能签发 Verdict、Receipt 或完成 Task。
- 普通 Room 中只有 human actor 能提交临场执行边；模型或 Agent 只能建议下一条边。
- Participant、ProjectRole、Room 和 Project 都不是 Harness 进程或外部账号。
- Context 必须可解释；模型自由总结不能替代来源和版本。
- Request 只能由获准动作解决，并且只推进其声明的阻塞范围。
- Project/Room 历史不因客户端关闭、外部编辑或运行时崩溃而丢失。

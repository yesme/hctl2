# 术语对照表

> 状态：非规范对照 · 草案 v0.15.5<br>
> 本表只提供中英对照与一句话释义；完整语义以[约束层](../spec/README.md)为准，Revision、Binding、Receipt、Lease、命令、Snapshot 六族的共同性质只在[约束总则](../spec/README.md#六族规则)定义。

## 核心产品词

| 术语 | 中文对照 | 一句话含义 | 权威定义 |
| --- | --- | --- | --- |
| Agent | 执行治理模块 | 第四个领域模块：把上层授权落实为 ChangeSet、运行时、终端、结果提案与证据 | [Agent](../agent.md) |
| Harness | 编码代理工具 | Codex、Claude Code、OpenCode 这类执行编码工作的工具 | [三面架构](../architecture.md#场景与系统) |
| Agency | 派出方 | 按冻结规格派出执行体并持有进程、PTY 与会话的 Terminal 受控端口；第一阶段由 Herdr 实现 | [spec/agent](../spec/agent.md#运行时与观测) |
| Repo | 仓库 | Git 仓库的逻辑身份；共享配置与结晶随它走 | [spec/project](../spec/project.md) |
| Project | 项目 | 具名目标、协作、承诺和交付物的长期容器 | [Project](../project.md) |
| Room | 协作聊天室 | 持久的多参与者协作空间，分 Repo Room、Project Room、Scoped Room | [Project](../project.md#room-类型) |
| Chat Room | 聊天室场景 | Project 模块的主场景，也是 Room 的交互视图 | [Project](../project.md#chat-room-场景) |
| Participant | 参与者 | 可寻址的逻辑协作者档案，独立于进程和外部账号 | [spec/project](../spec/project.md) |
| Request | 请求卡 | 向指定人或角色索取信息、授权或决定的一级对象 | [spec/project](../spec/project.md#request) |
| Memo | 备忘 | 经提炼、预览与发布形成的长期知识 | [Project](../project.md) |
| Artifact | 工件 | 登记后可稳定引用的交付物；发布版本属于 Revision 族 | [Project](../project.md) |
| Context | 上下文 | 顶层授权采用哪些来源，以及每个执行实际收到哪些字节 | [spec/project](../spec/project.md#context-memo-artifact) |
| Skill | 技能包 | 带版本与摘要的共享方法定义 | [spec/system](../spec/system.md) |
| Task | 任务承诺 | 可排序、可指派、可验收的长期承诺 | [Task](../task.md) |
| Kanban | 看板 | Task 的主场景；一个 Repo 一个 Board，Project 是分组，Task 是卡片 | [Task](../task.md#kanban-场景) |
| Run | 一次受治理施工 | 对冻结施工图、契约、候选与权限的一次授权执行 | [Run](../run.md) |
| Workflow | 施工图 | 与引擎无关的控制图与治理规则 | [Run](../run.md) |
| Obligation | 交付义务 | 一个外部节点必须产出的逻辑结果 | [spec/run](../spec/run.md) |
| Seat | 执行席位 | 交付义务中稳定的执行或评审位置 | [spec/run](../spec/run.md) |
| Attempt | 执行尝试 | 某个候选对一个席位的一次执行 | [spec/run](../spec/run.md) |
| Gate | 评审关卡 | 冻结在施工图中、决定结果如何通过的治理节点 | [spec/run](../spec/run.md) |
| Verdict | 裁决 | 对精确版本作出的语义评审结论 | [spec/run](../spec/run.md) |
| Receipt | 凭证 | 校验通过后签发的不可变证明 | [spec/README](../spec/README.md#六族规则) |
| Terminal | 终端场景 | Agent 模块用于观察、诊断和接管精确执行的场景 | [Agent](../agent.md#terminal-场景) |
| ChangeSet | 变更集 | 一次获准的代码写入边界 | [spec/agent](../spec/agent.md) |
| Evidence | 证据 | diff、测试输出、SCM 状态等可核验观测 | [spec/agent](../spec/agent.md) |
| Workbench | 工作台 | 组合四类 provider 客户端、联合投影和 HCTL 公共命令入口的桌面 | [spec/system](../spec/system.md) |

## 三类数据（数据类别，不是对象）

| 类别 | 中文对照 | 一句话含义 |
| --- | --- | --- |
| metadata | 治理元数据 | 身份、绑定、授权与判决，住在 HCTL 账本 |
| content | 场景内容 | 协作与执行记忆，住在对应场景系统 |
| artifact | 结晶 | 从 content 提炼出的不可变产物，进入 Git；与领域对象 Artifact 不同物 |

权威定义见[约束层总则](../spec/README.md#三类数据)。

## 客户端动作分类（不是对象）

| 类别 | 一句话含义 |
| --- | --- |
| content 写入与观测 | 客户端改变 provider 拥有的消息、卡片等 content，control 按 Snapshot/cursor 对账 |
| human 命令请求 | direct client 或模块接纳的 provider event 形成同一个 HCTL command draft，并经过 Preview 与准入 |
| 运行时输入 | 向精确 Execution Runtime 输入，并按 descriptor、lease、generation 能力记录保证等级 |
| Result Proposal | Harness/Agency 交给 owner 校验的结果与证据 |
| 不支持的 provider mutation | 先改变外部机械状态、无法保持 HCTL 副作用顺序的管理动作，只回读为分歧 |

分类取决于动作落点与信封，权威规则见[系统约束](../spec/system.md#客户端动作与-provider-事件)。

## 场景与系统

| 场景 | content 系统角色 | 拥有的 content |
| --- | --- | --- |
| Chat Room | chat server（聊天服务器） | 聊天记录、调用过程与结果卡 |
| Kanban | task backend（任务后端） | 任务卡、流转、排序、评论 |
| Workflow | workflow engine（工作流引擎） | 令牌位置、重试、定时器、机械执行历史 |
| Terminal | harness / Agency（第一阶段为 Herdr） | 会话转录、PTY 流 |

权威对照见[三面架构](../architecture.md#场景与系统)。Agent（模块）、Agency（派出方）与 agent（执行体的口语说法）是三个不同词；`provider` 泛指模块供应端，并非跨模块对象。

## Revision 族（不可变版本）

| 成员 | 中文对照 | 版本化的内容 |
| --- | --- | --- |
| Task Revision | 任务契约版本 | Task 的范围、验收标准与所需能力 |
| Workflow Revision | 施工图版本 | 与引擎无关的控制图和治理规则 |
| ChangeSet Revision | 变更集快照 | 写入边界内的不可变 Git 快照 |
| Artifact Revision | 工件版本 | 登记交付物的一次发布 |
| Extension Revision | 扩展版本 | 扩展的代码、接口、能力与信任级别 |
| Engine Deployment | 引擎部署版本 | 施工图为特定引擎编译出的产物 |

## Binding 族（冻结的身份连接）

| 成员 | 中文对照 | 连接的两端 |
| --- | --- | --- |
| Resolved Port Binding | 端口解析绑定 | 受控端口 ↔ 具体 provider 及实测能力 |
| Chat 端口绑定 | 聊天端口绑定 | Room ↔ chat server 房间；加密准入见 [Project 约束](../spec/project.md#room-与消息) |
| Task Binding | 任务来源绑定 | Task ↔ 外部实体、字段写入权与 adapter 版本 |
| Project Role Binding | 角色绑定 | Project 角色 ↔ 精确 Participant 版本 |
| Engine Execution Binding | 引擎执行绑定 | Run ↔ 外部引擎执行实例与关联键 |

## Receipt 族（校验后的证明）

| 成员 | 证明什么 |
| --- | --- |
| Gate Receipt | 评审关卡按冻结规则通过 |
| Task Completion Receipt | 完成命令对精确契约、规则与证据成立 |
| Integration Receipt | 精确变更集按授权合入目标并回读确认 |

## Lease 族（单持有者独占权）

| 成员 | 中文对照 | 独占什么 |
| --- | --- | --- |
| Write Lease | 写入租约 | 一个 ChangeSet 的当前写权 |
| Terminal Input Lease | 终端输入租约 | 一个受 HCTL 管理的终端目标输入权 |
| Agency binding owner lease | 派出方绑定的 owner 租约 | 一个 Agency 绑定范围同时只有一个 owner，与其代次成对；旧代次失权（见[单写者](../spec/system.md#单写者)） |

control 账本排他与 Repo 现场的 OS 锁不是 Lease 对象，是单写者约束的实现细节；约束本身见[系统边界](../spec/system.md#单写者)。

代次共六个：control writer、Repo Instance site、Agency binding owner、语义 owner（Attempt／Room Invocation）、Execution Runtime 与 Engine Execution Binding 各用自己范围内的一个，总表见[系统边界的代次家族](../spec/system.md#代次家族)；Participant/Binding revision、producer sequence 与 cursor 属于别的版本或顺序概念，不是代次。

## 命令族（持久命令与副作用）

类型化命令表达「完成 Task」「启动 Run」「采纳契约」「合入 ChangeSet」等动作。改变外部权威事实的命令持久化副作用意图，由 tool 或 adapter 执行，并在回读确认后签发 Receipt。

## Snapshot / 观测族（先观测后准入）

| 成员 | 中文对照 | 观测什么 |
| --- | --- | --- |
| Task Source Snapshot | 来源快照 | 外部任务系统的一次只追加观测 |
| Result Proposal | 结果提案 | Harness 提交、等待 owner 校验的结果 |
| 运行时观测 | — | 进程、心跳、屏幕等按证据分级的物理观测 |

## 票据与规格（步骤产物，不是领域对象）

| 名字 | 中文对照 | 哪个步骤的产物 |
| --- | --- | --- |
| Execution Spec | 派发规格 | 派发执行时冻结 owner、Context、Participant、Skill、Profile 与权限 |
| Run Manifest | 施工清单 | 启动 Run 时冻结 Project/Task/Workflow、候选、规则与预算 |
| Attach Descriptor | 连接票据 | 连接精确终端目标的短期凭据 |
| Context Manifest | 根上下文清单 | 顶层授权冻结的目的、来源、新鲜度、缺口与边界 |
| Context Bundle | 消费上下文包 | 一次执行实际收到的有序内容、工具版本与 bytes digest |

## 独立对象（不属六族）

| 名字 | 中文对照 | 一句话含义 |
| --- | --- | --- |
| Repo Instance | 仓库实例 | 系统挂接到逻辑 Repo 的 clone 与执行现场 |
| Room Invocation | 单次调用 | 从 Room 发起的一次有边界 Harness 调用 |
| Execution Runtime | 执行运行时 | 一次执行的主机、隔离域、代次与终端通道 |
| Worker Profile | 执行者配置 | Harness、模型、模式、权限与可选加固的复用组合 |

## 引用格式（不是对象）

ReviewSubjectRef 是 kind + ID + digest 的评审对象引用；`revision_digest` 与 `review_subject_digest` 含义不同，分别按拥有它们的约束使用。

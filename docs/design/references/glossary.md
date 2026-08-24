# 术语对照表

> 状态：非规范对照 · 草案 v0.12.2<br>
> 本表只提供中英对照和一句话含义，方便快速查阅；完整语义以[合同层](../spec/README.md)为准，六族（Revision、Binding、Receipt、Lease、命令、Snapshot）的共同性质在[总则](../spec/README.md#六族规则)只定义一次，本表不重复。

## 核心产品词

| 术语 | 中文对照 | 一句话含义 | 权威定义 |
| --- | --- | --- | --- |
| Agent | 执行治理模块 | 第四个领域模块：执行授权、写入边界、物理运行时观测与结果证据。与 Harness（工具）、agentd（组件）不同物 | [Agent](../agent.md) |
| Harness | 编码代理工具 | Codex、Claude Code、OpenCode 这类可以执行编码工作的工具；Terminal 场景的系统角色 | [三面架构](../architecture.md#场景与系统) |
| Repo | 仓库 | Git 仓库的逻辑身份；共享配置与结晶随它走 | [spec/project](../spec/project.md) |
| Project | 项目 | 具名目标、协作、承诺和交付物的长期容器 | [Project](../project.md) |
| Room | 协作聊天室 | 持久的多参与者协作空间；分 Repo Room、Project Room、Scoped Room | [Project](../project.md) |
| Chat Room | 聊天室场景 | Project 模块的主操作场景；Room 的场景视图 | [Project](../project.md) |
| Participant | 参与者 | 可寻址的逻辑协作者档案；不等于某个进程或外部账号 | [Project](../project.md) |
| Request | 请求卡 | 向指定的人或角色索取信息、授权或决定的一级对象 | [spec/project](../spec/project.md) |
| Memo | 备忘 | 用户明确提炼、预览并发布的长期知识 | [Project](../project.md) |
| Artifact | 工件 | 登记过的可引用交付物；发布版本见 Revision 族 | [Project](../project.md) |
| Context | 上下文 | 顶层授权为何、从哪些精确来源取材，以及每个消费执行实际收到哪些字节；由根清单与消费包分开承载 | [spec/project](../spec/project.md) |
| Skill | 技能包 | 带版本与摘要的共享方法定义；提供方法，不授予权限 | [spec/system](../spec/system.md) |
| Task | 任务承诺 | 可排序、可指派、可验收的长期承诺 | [Task](../task.md) |
| Kanban | 看板 | Task 的主操作场景；一个 Repo 一个 Board，Project 是板上分组，content 在所选任务后端；泳道（lane）只是投影 | [Task](../task.md) |
| Run | 一次受治理施工 | 对冻结施工图、契约、候选和权限的一次授权执行 | [Run](../run.md) |
| Workflow | 施工图 | 与引擎无关的控制图与治理规则；版本见 Revision 族 | [Run](../run.md) |
| Obligation | 交付义务 | 一个外部节点必须产出的逻辑结果 | [spec/run](../spec/run.md) |
| Seat | 执行席位 | 义务中稳定的逻辑执行者或投票位置 | [spec/run](../spec/run.md) |
| Attempt | 执行尝试 | 某个候选对席位的一次执行 | [spec/run](../spec/run.md) |
| Gate | 评审关卡 | 施工图中冻结的治理节点；决定结果凭什么通过 | [spec/run](../spec/run.md) |
| Verdict | 裁决 | 对精确版本的语义评审结论 | [spec/run](../spec/run.md) |
| Receipt | 凭证 | 校验通过后签发的证明；见 Receipt 族 | [spec/README](../spec/README.md#六族规则) |
| Terminal | 终端场景 | Agent 模块的观察、诊断和接管场景 | [Agent](../agent.md) |
| ChangeSet | 变更集 | 一次获准写入边界；快照见 Revision 族 | [spec/agent](../spec/agent.md) |
| Evidence | 证据 | diff、测试输出、SCM 状态等可核验的观测 | [spec/agent](../spec/agent.md) |
| Workbench | 工作台 | 集成四个场景的桌面客户端；只是客户端，没有额外权限 | [spec/system](../spec/system.md) |

## 三类数据（数据类别，不是对象）

| 类别 | 中文对照 | 一句话含义 |
| --- | --- | --- |
| metadata | 治理元数据 | 身份、绑定、授权与判决；住在 HCTL 自己的账本 |
| content | 场景内容 | 各场景的协作与执行记忆；住在该场景的 content 系统 |
| artifact | 结晶 | content 提炼出的不可变产物；进 Git。与领域对象 Artifact（工件）不同物 |

权威定义见[合同层总则](../spec/README.md#三类数据)。

## 场景-系统对照

| 场景 | 系统角色（中文对照） | 系统拥有的 content |
| --- | --- | --- |
| Chat Room | chat server（聊天服务器） | 聊天记录、调用过程与结果卡 |
| Kanban | task backend（任务后端） | 任务卡、流转、排序、评论 |
| Workflow | workflow engine（工作流引擎） | 令牌位置、重试、定时器、机械执行历史 |
| Terminal | harness（编码代理工具）/ 运行时后端 | 会话转录、PTY 流 |

权威定义见[三面架构](../architecture.md#场景与系统)。agentd 是组件实现名（Agent 模块的本机执行守护进程），与 Agent 模块、harness 系统角色都不同物。

## Revision 族（不可变版本）

| 成员 | 中文对照 | 版本化的是什么 |
| --- | --- | --- |
| Task Revision | 任务契约版本 | Task 的范围、验收标准与所需能力；不可变正文在 Git，准入/current 在账本 |
| Workflow Revision | 施工图版本 | 与引擎无关的控制图和治理规则；不可变正文在 Git，批准/current 在账本 |
| ChangeSet Revision | 变更集快照 | 一次写入边界内的不可变 Git 快照 |
| Artifact Revision | 工件版本 | 登记交付物的一次不可变发布 |
| Extension Revision | 扩展版本 | 一个扩展的代码、接口、能力与信任级别 |
| Engine Deployment | 引擎部署版本 | 某施工图经编译器给某引擎的产物 |

## Binding 族（冻结的身份连接）

| 成员 | 中文对照 | 连接的两端 |
| --- | --- | --- |
| Resolved Port Binding | 端口解析绑定 | 一个受控端口 ↔ 具体提供方（含实测能力与降级；动态 health/cursor 不进不可变 binding） |
| Task Binding | 任务来源绑定 | 一个 Task ↔ 外部实体、字段写入权与适配器版本 |
| Project Role Binding | 角色绑定 | 一个 Project 角色 ↔ 精确 Participant 版本 |
| Engine Execution Binding | 引擎执行绑定 | 一个 Run ↔ 外部引擎的执行实例与关联键 |

## Receipt 族（校验后的证明）

| 成员 | 证明什么 |
| --- | --- |
| Gate Receipt | 评审关卡按法定票数通过 |
| Task Completion Receipt | 一次完成命令对精确契约、规则和证据成立 |
| Integration Receipt | 精确变更集已按授权合入目标并回读确认 |

## Lease 族（单持有者独占权）

| 成员 | 中文对照 | 独占什么 |
| --- | --- | --- |
| Write Lease | 写入租约 | 一个变更集的写权；同时最多一个持有者 |
| Terminal Input Lease | 终端输入租约 | 一个终端目标的输入权；接管原子撤销旧租约 |

control writer、Repo Instance site 与 agentd/backend owner 的排他权以各自 generation（代次）表达；Attempt 的 owner generation 与 Execution Runtime 的 runtime generation 是另外两层身份，不能共用一个裸 `generation`。Participant/Binding revision、producer sequence 和 cursor 都不是代次。

## 命令族（持久命令与副作用）

各模块的类型化命令（「完成 Task」「启动 Run」「采纳契约」「合入 ChangeSet」等动宾语义名）是改变事实的唯一途径。**外部副作用命令**（副作用意图）承载会改变外部权威事实的动作：executor = tool 时是本地 Git 集成，executor = adapter 时是远端 SCM 或第三方平台写入；均为先持久记录、再执行、回读确认后才可签 Receipt。

## Snapshot / 观测族（先观测后准入）

| 成员 | 中文对照 | 观测什么 |
| --- | --- | --- |
| Task Source Snapshot | 来源快照 | 外部任务系统的一次只追加观测；改变契约的内容须采纳才生效 |
| Result Proposal | 结果提案 | Harness 提交、等待 owner 校验的结果；不是裁决或凭证 |
| 运行时观测 | — | 进程、心跳、屏幕等物理观测；按来源证据分级仲裁 |

## 票据与规格（步骤产物，不是领域对象）

| 名字 | 中文对照 | 哪个步骤的产物 |
| --- | --- | --- |
| Execution Spec | 派发规格 | 派发一次执行时冻结 owner version、根 Context Manifest、消费 Bundle、Participant/Role/Skill/Profile 与权限；runtime identity 在后续激活映射产生 |
| Run Manifest | 施工清单 | 启动 Run 时冻结 Project/Task/Workflow、根 Context、Participant/Role/Skill、候选、规则和预算 |
| Attach Descriptor | 连接票据 | 对精确终端目标的短期连接凭据 |
| Context Manifest | 根上下文清单 | 顶层授权冻结的目的、精确来源/父清单、freshness/gap、Skill 与权限/预算边界 |
| Context Bundle | 消费上下文包 | 某个 Invocation/Attempt 从根清单实际物化并交付的有序内容、工具版本与 bytes digest |

## 独立对象（不属六族的领域对象）

| 名字 | 中文对照 | 一句话含义 |
| --- | --- | --- |
| Repo Instance | 仓库实例 | 系统挂接到逻辑 Repo 的 clone/执行现场（worktree、ChangeSet 物化、运行时与 site fence）；不属于 Project，本地无账本 |
| Room Invocation | 单次调用 | 从 Room 发起的一次有边界 Harness 调用；有完整生命周期 |
| Execution Runtime | 执行运行时 | 一次执行的主机、隔离域、代次与终端通道；有完整生命周期与代次 |
| Worker Profile | 执行者配置 | Harness、模型、模式、权限的可复用组合 |

## 引用格式（不是对象）

ReviewSubjectRef（评审对象引用：kind + ID + 摘要）、revision_digest 与 review_subject_digest（两种不同语义的摘要，不能互换）。

# 术语对照表

> 状态：非规范对照 · 草案 v0.16.4<br>
> 本表只提供中英对照与一句话释义；完整语义以[约束层](../spec/README.md)为准，Revision、Binding、Receipt、Lease、命令、Snapshot 六族的共同性质只在[约束总则](../spec/README.md#六族规则)定义。

## 约束、契约与清单

| 词 | 偏离后果 | 使用范围 |
| --- | --- | --- |
| 约束 | 遵守或违反；违反表示实现有缺陷，契约测试失败 | 规范对实现提出的单向规则；constraint 一律译作“约束” |
| 契约 | 采纳或产生分歧；偏离成为等待裁决或重新采纳的领域事实 | Task Revision 承载的验收约定；“契约测试”沿用业界通称 |
| 清单 | 冻结后逐项核对 | 单侧声明的逐项要求，如施工清单、根上下文清单和终局结果清单 |

## 核心产品词

| 术语 | 中文对照 | 一句话含义 | 权威定义 |
| --- | --- | --- | --- |
| Agent | 编码代理（泛称） | 泛指 Codex、Claude Code 这类 AI 编码代理，不是模块名；第四个模块见 Participant | [Participant](../participant.md) |
| Harness | 编码代理工具 | Codex、Claude Code、OpenCode 这类执行编码工作的工具 | [三面架构](../architecture.md#场景与系统) |
| Agency | 派出方 | 参与者的供给方：维护可派出的名册与条款，按冻结规格交付执行体端点；默认为发布包自带的本地参考实现，运行时用 Herdr | [spec/participant](../spec/participant.md#运行时与观测) |
| worker | 执行体 | Agency 供给的一次具体运行：Harness 进程、装载的 Skill、PTY 与 TUI；接受派工、报告观测与结果提案 | [spec/participant](../spec/participant.md#运行时与观测) |
| Repo | 仓库 | Git 仓库的逻辑身份；共享配置与结晶随它走 | [spec/project](../spec/project.md) |
| Project | 项目 | 具名目标、协作、承诺和交付物的长期容器 | [Project](../project.md) |
| Room | 聊天室 | 持久的多参与者协作空间，分 Repo Room、Project Room、Scoped Room；也是 Project 模块的场景名 | [Project](../project.md#room-场景) |
| Participant | 参与者 | 第四个领域模块：数字参与者的稳定身份、人设、Skill 申报、执行者配置与一次物理执行；人不是 Participant | [Participant](../participant.md) |
| Request | 请求卡 | 向指定人或角色索取信息、授权或决定的一级对象 | [spec/project](../spec/project.md#request) |
| Memo | 备忘 | 经提炼、预览与发布形成的长期知识 | [Project](../project.md) |
| Artifact | 工件 | 登记后可稳定引用的交付物；发布版本属于 Revision 族 | [Project](../project.md) |
| Context | 上下文 | 顶层授权采用哪些来源，以及每个执行实际收到哪些字节 | [spec/project](../spec/project.md#context-memo-artifact) |
| Skill | 技能包 | 带版本与摘要的共享方法定义；由 Agency 安装并申报，账本只记引用、摘要与可核验性 | [spec/participant](../spec/participant.md#skill-与申报) |
| Task | 任务承诺 | 可排序、可指派、可验收的长期承诺 | [Task](../task.md) |
| Kanban | 看板 | Task 的主场景；一个 Repo 一个 Board，Project 是分组，Task 是卡片 | [Task](../task.md#kanban-场景) |
| Run | 一次受治理施工 | 对冻结施工图、契约、候选与权限的一次授权执行 | [Run](../run.md) |
| Workflow | 施工图 | 与引擎无关的控制图与治理规则 | [Run](../run.md) |
| Obligation | 交付义务 | 一个外部节点必须产出的逻辑结果 | [spec/run](../spec/run.md) |
| Seat | 席位 | 交付义务中稳定的执行或评审位置 | [spec/run](../spec/run.md) |
| Attempt | 尝试 | 某个候选对一个席位的一次执行 | [spec/run](../spec/run.md) |
| Gate | 评审关卡 | 冻结在施工图中、决定结果如何通过的治理节点 | [spec/run](../spec/run.md) |
| Verdict | 裁决 | 评审席位对精确版本投的一票：通过、要改、拒绝，可带分歧落点 | [spec/run](../spec/run.md) |
| Receipt | 凭证 | 控制面校验后开出的不可变证明：结论、依据的规则、指向哪几条证据 | [spec/README](../spec/README.md#六族规则) |
| Terminal | 终端场景 | Participant 模块的场景：观察、诊断和接管精确执行体，也是执行体暴露给 Workbench 的 TUI 接口（participant.tui） | [Participant](../participant.md#terminal-场景) |
| ChangeSet | 变更集 | 一次获准的代码写入边界 | [spec/participant](../spec/participant.md) |
| Evidence | 证据 | 被判定的事实记录（diff、测试输出、CI 状态、工具箱回读），本身不下结论；按证据通道分三级 | [spec/participant](../spec/participant.md#证据通道) |
| Workbench | 工作台 | 组合四类 provider 客户端、联合投影和 HCTL 公共命令入口的桌面 | [spec/system](../spec/system.md) |

## 系统组件与常用技术词

| 写法 | 中文对照或用法 |
| --- | --- |
| `hctl2-tool` | 工具箱；两者始终指同一个现场执行组件 |
| human actor | 有权的人；约束层用 `human actor`，设计层写「人」或「有权的人」 |
| owner | 归属者；在精确对象或字段名中保留 `owner` |
| fence | 代次栅栏；在字段名或能力名中保留 `fence` |
| worktree | Git 工作树；命令与路径中保留 `worktree` |
| ID | 标识符；字段名中保留 `id` |
| claim | 认领；字段名或外部 API 名中保留原形 |
| CAS | 比较并交换；字段名和实现机制名中保留 `CAS` |
| fresh readback | 当前回读；精确策略名或字段名中保留原形 |
| ACK | 确认回执；协议状态名中保留 `ACK` |

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
| 运行时输入 | 向精确 Execution Runtime 输入，并按连接票据、租约与代次能力记录恢复等级 |
| Result Proposal | Harness/Agency 交给归属者校验的结果与证据 |
| 不支持的 provider mutation | 先改变外部机械状态、无法保持 HCTL 副作用顺序的管理动作，只回读为分歧 |

分类取决于动作落点与信封，权威规则见[系统约束](../spec/system.md#客户端动作与-provider-事件)。

## 场景与系统

| 场景 | content 系统角色 | 拥有的 content |
| --- | --- | --- |
| Room | chat server（聊天服务器） | 聊天记录、调用过程与结果卡 |
| Kanban | task backend（任务后端） | 任务卡、流转、排序、评论 |
| Workflow | workflow engine（工作流引擎） | 令牌位置、重试、定时器、机械执行历史 |
| Terminal | Agency 供给的执行体（默认：本地参考实现，运行时 Herdr） | 会话转录、PTY 流 |

权威对照见[三面架构](../architecture.md#场景与系统)。Participant（模块）、Agency（派出方）、worker（执行体）与 Agent（编码代理的泛称）是四个不同词；`provider` 泛指模块供应端，并非跨模块对象。

## Revision 族（不可变版本）

| 成员 | 中文对照 | 版本化的内容 |
| --- | --- | --- |
| Task Revision | 任务契约版本 | Task 的范围、验收标准与所需能力 |
| Workflow Revision | 施工图版本 | 与引擎无关的控制图和治理规则 |
| ChangeSet Revision | 变更集快照 | 写入边界内的不可变 Git 快照 |
| Artifact Revision | 工件版本 | 登记交付物的一次发布 |
| Extension Revision | 扩展版本 | 扩展的代码、接口、能力与信任级别 |
| Engine Deployment | 引擎部署版本 | 施工图为特定引擎编译出的产物 |

## Binding 族（HCTL 的东西 ↔ 外部系统里对应的东西）

| 成员 | 中文对照 | 连接的两端 |
| --- | --- | --- |
| Port–Provider Binding | 端口与供应端的绑定 | 模块的受控端口 ↔ 具体供应端的制品、适配器、配置摘要与实测能力 |
| Room–Server Binding | Room 与聊天服务器房间的绑定 | Room ↔ chat server 上房间的稳定 ID；房间升级换 ID 是换绑，Room 身份不变；加密准入见 [Project 约束](../spec/project.md#room-与消息) |
| Task–Backend Binding | Task 与任务后端卡片的绑定 | Task ↔ 后端那张卡的外部身份、字段写入权、可接纳的人为动作与适配器版本 |
| Run–Engine Binding | Run 与工作流引擎执行的绑定 | Run ↔ 引擎里那次执行的部署、执行 ID、关联键与代次 |
| Participant–Agency Binding | Participant 与派出方名册项的绑定 | Participant ↔ 供给它的 Agency 名册项与条款；换派出方是换绑，身份不变 |

Project 的**参与者授权**（哪些 Participant 可在本 Project 出场、职责、权限与预算上限）不是 Binding：两端都在 HCTL 内部，它是 Project 版本化设置的一部分，「角色」只是其中的职责标签字段；权威见 [Project 约束](../spec/project.md#参与者授权)。

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
| Agency binding owner lease | 派出方端口的归属者租约 | 一个 Agency 端口 Port–Provider Binding 的范围（同一服务器、套接字或主机命名空间）同时只有一个归属者，与其代次成对；旧代次失权（见[单写者](../spec/system.md#单写者)） |

control 账本排他与 Repo 现场的 OS 锁不是 Lease 对象，是单写者约束的实现细节；约束本身见[系统边界](../spec/system.md#单写者)。

全系统共用六种彼此独立的代次：账本写入者、仓库现场、Agency 绑定归属者、Attempt／Room Invocation 的语义归属者、Execution Runtime，以及 Run–Engine Binding 各使用自己范围内的一种。成员、范围和推进时机见[系统边界的代次家族](../spec/system.md#代次家族)。Participant/Binding revision、producer sequence 与 cursor 属于版本或顺序概念，不是代次。

## 命令族（持久命令与副作用）

类型化命令表达「完成 Task」「启动 Run」「采纳契约」「合入 ChangeSet」等动作。改变外部权威事实的命令持久化副作用意图，由 tool 或 adapter 执行，并在回读确认后签发 Receipt。

## Snapshot / 观测族（先观测后准入）

| 成员 | 中文对照 | 观测什么 |
| --- | --- | --- |
| Task Backend Snapshot | 任务后端快照 | 任务后端的一次只追加观测 |
| Result Proposal | 结果提案 | Harness 提交、等待归属者校验的结果 |
| 运行时观测 | — | 进程、心跳、屏幕等按证据分级的物理观测 |

## 票据与规格（步骤产物，不是领域对象）

| 名字 | 中文对照 | 哪个步骤的产物 |
| --- | --- | --- |
| Execution Spec | 派发规格 | 派发执行时冻结归属者、Context、Participant、Skill、Profile 与权限 |
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

<!-- BEGIN GENERATED IDENTIFIER GLOSSARY -->
## 语义名与标识符

> 本节由 `src/build/docs/generate_identifier_glossary.pl` 从约束层正文的代码体字段与枚举值生成；请修改约束层来源或生成器映射，不要手改本表。

| 语义名 | 标识符 | 约束层出处 |
| --- | --- | --- |
| Run 引用 | `run_ref` | [run.md](../spec/run.md) |
| Run 标识符 | `run_id` | [connections.md](../spec/connections.md) |
| Run 状态版本 | `run_version` | [run.md](../spec/run.md) |
| 上下文包摘要 | `bundle_digest` | [project.md](../spec/project.md) |
| 不可变外部实体标识符 | `immutable_external_entity_id` | [task.md](../spec/task.md) |
| 不支持 | `unsupported` | [run.md](../spec/run.md) |
| 仓库实例标识符 | `repo_instance_id` | [system.md](../spec/system.md) |
| 仓库标识符 | `repo_id` | [project.md](../spec/project.md)、[system.md](../spec/system.md)、[task.md](../spec/task.md) |
| 仓库版本 | `repo_version` | [project.md](../spec/project.md) |
| 仓库范围 | `repo_scope` | [connections.md](../spec/connections.md)、[participant.md](../spec/participant.md)、[project.md](../spec/project.md) |
| 代次 | `generation` | [connections.md](../spec/connections.md)、[participant.md](../spec/participant.md) |
| 任务来源 | `task_source` | [task.md](../spec/task.md) |
| 任务生命周期版本 | `task_lifecycle_version` | [task.md](../spec/task.md) |
| 供应端事件 | `provider_event` | [system.md](../spec/system.md) |
| 允许原生交互 | `native_interactive_allowed` | [participant.md](../spec/participant.md) |
| 内联 | `inline` | [project.md](../spec/project.md)、[run.md](../spec/run.md) |
| 内部归约器 | `internal_reducer` | [system.md](../spec/system.md) |
| 分组类型 | `group_kind` | [task.md](../spec/task.md) |
| 分组锚点稳定标识符 | `group_anchor_stable_id` | [task.md](../spec/task.md) |
| 单次调用版本 | `invocation_version` | [connections.md](../spec/connections.md)、[project.md](../spec/project.md)、[system.md](../spec/system.md) |
| 受管单写者 | `managed_single_writer` | [participant.md](../spec/participant.md) |
| 只读关联 | `linked_readonly` | [task.md](../spec/task.md) |
| 后端权威 | `backend_authoritative` | [task.md](../spec/task.md) |
| 回忆 | `recall` | [project.md](../spec/project.md) |
| 备忘标识符 | `memo_id` | [project.md](../spec/project.md) |
| 外部实体类型 | `external_entity_kind` | [task.md](../spec/task.md) |
| 外部看板项标识符 | `external_board_item_id` | [task.md](../spec/task.md) |
| 契约内 | `contract` | [run.md](../spec/run.md) |
| 实现内 | `implementation` | [run.md](../spec/run.md) |
| 尝试代次 | `attempt_generation` | [connections.md](../spec/connections.md)、[project.md](../spec/project.md)、[run.md](../spec/run.md)、[system.md](../spec/system.md) |
| 工件版本标识符 | `artifact_revision_id` | [project.md](../spec/project.md) |
| 工件状态版本 | `artifact_version` | [project.md](../spec/project.md) |
| 工具箱直接回读 | `toolbox_readback` | [participant.md](../spec/participant.md) |
| 已知 | `known` | [run.md](../spec/run.md) |
| 引擎绑定代次 | `engine_binding_generation` | [run.md](../spec/run.md)、[system.md](../spec/system.md) |
| 当前 | `current` | [connections.md](../spec/connections.md)、[project.md](../spec/project.md) |
| 执行主体 | `execution_principal` | [system.md](../spec/system.md) |
| 执行模式 | `execution_mode` | [connections.md](../spec/connections.md) |
| 指针 | `pointer` | [project.md](../spec/project.md)、[run.md](../spec/run.md) |
| 接受 | `accepted` | [run.md](../spec/run.md) |
| 控制面写入者代次 | `control_writer_generation` | [system.md](../spec/system.md) |
| 控制面权威 | `hctl_authoritative` | [task.md](../spec/task.md) |
| 放置范围稳定标识符 | `placement_scope_stable_id` | [task.md](../spec/task.md) |
| 未知 | `unknown` | [run.md](../spec/run.md)、[system.md](../spec/system.md) |
| 机械可判 | `mechanical` | [task.md](../spec/task.md) |
| 根上下文清单摘要 | `manifest_digest` | [connections.md](../spec/connections.md)、[project.md](../spec/project.md) |
| 根上下文清单标识符 | `context_manifest_id` | [project.md](../spec/project.md) |
| 法定票数不可达 | `quorum-unreachable` | [run.md](../spec/run.md) |
| 活动 | `active` | [connections.md](../spec/connections.md)、[run.md](../spec/run.md)、[task.md](../spec/task.md) |
| 状态版本 | `state_version` | [task.md](../spec/task.md) |
| 现场代次 | `site_generation` | [system.md](../spec/system.md) |
| 直接客户端 | `direct_client` | [system.md](../spec/system.md) |
| 看板范围稳定标识符 | `board_scope_stable_id` | [task.md](../spec/task.md) |
| 端口类型 | `port_kind` | [task.md](../spec/task.md) |
| 等待完成 | `completion_pending` | [connections.md](../spec/connections.md)、[run.md](../spec/run.md)、[task.md](../spec/task.md) |
| 绑定版本 | `binding_revision` | [task.md](../spec/task.md) |
| 结果提交 SHA | `result_commit_sha` | [participant.md](../spec/participant.md) |
| 范围稳定标识符 | `scope_stable_id` | [task.md](../spec/task.md) |
| 要求修改 | `changes_requested` | [run.md](../spec/run.md) |
| 评审关卡 | `gate` | [task.md](../spec/task.md) |
| 请求卡标识符 | `request_id` | [connections.md](../spec/connections.md) |
| 请求卡版本 | `request_version` | [project.md](../spec/project.md) |
| 账号稳定标识符 | `account_stable_id` | [task.md](../spec/task.md) |
| 转述 | `narrated` | [participant.md](../spec/participant.md) |
| 运行时代次 | `runtime_generation` | [connections.md](../spec/connections.md)、[participant.md](../spec/participant.md)、[run.md](../spec/run.md)、[system.md](../spec/system.md) |
| 进程内 | `in_process` | [connections.md](../spec/connections.md)、[participant.md](../spec/participant.md)、[project.md](../spec/project.md)、[system.md](../spec/system.md) |
| 适配器事件 | `adapter_event` | [participant.md](../spec/participant.md) |
| 项目标识符 | `project_id` | [connections.md](../spec/connections.md)、[task.md](../spec/task.md) |
| 项目版本 | `project_version` | [project.md](../spec/project.md) |
| 项目范围 | `project_scope` | [connections.md](../spec/connections.md)、[participant.md](../spec/participant.md)、[project.md](../spec/project.md) |
| 驳回 | `rejected` | [run.md](../spec/run.md) |
<!-- END GENERATED IDENTIFIER GLOSSARY -->

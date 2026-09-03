# 约束层总则

> 状态：规范性 · 草案 v0.16.4<br>
> 日期：2026-08-31<br>
> 定位：本目录是 HCTL2 的约束层——精确的对象、状态机、写入者与共享机制。设计层（`docs/design/` 根目录）用产品语言回答为什么与怎么用；两层冲突时以约束层为准，但约束层不得引入设计层没有的产品行为。

## 词汇分类法

具名概念分四类；只有前两类可以引入新造的对象名，状态值与引用格式只是枚举与格式，不占概念名额：

| 类别 | 判据 | 例子 |
| --- | --- | --- |
| 领域对象 | 独立生命周期、恢复边界或权限边界至少居其一 | Task、Run、Seat、ChangeSet、Request |
| 票据与记录 | 某个步骤产生的只追加、短期或一次性记录，而且后续规则可以精确引用它 | Execution Spec、Attach Descriptor、Receipt、外部副作用命令、Snapshot |
| 状态值 | lifecycle 枚举，不是对象 | 开放、待采纳、结果未知 |
| 引用格式 | 指认他物的结构化引用，不是对象 | ReviewSubjectRef、digest |

只有领域对象、票据和记录可以引入新名字；能用日常语言或外部标准词说清的，不另造词。具名对象和票据使用带空格的专名，命令使用动宾语义名，状态值使用中文语义名。实现标识符另建“语义名 ↔ 标识符”对照表，以便回溯约束。

协议或 schema 字段、序列化格式标识，以及外部标准、产品或源码中的原名必须逐字指认时，可以保留原形并用代码格式标示；它们不因此成为新的领域概念。设计层正文只使用下节列出的核心产品词。

## 核心产品词

| 词 | 中文对照 | 类别 | 可见性 |
| --- | --- | --- | --- |
| Repo | 仓库 | 对象 | 用户可见 |
| Project | 项目 | 对象 | 用户可见 |
| Room | 聊天室 | 对象，也是 Project 模块的场景名 | 用户可见 |
| Participant | 参与者 | 对象 | 用户可见 |
| Request | 请求卡 | 对象 | 用户可见 |
| Memo | 备忘 | 对象 | 用户可见 |
| Artifact | 工件 | 对象 | 用户可见 |
| Context | 上下文 | 横切概念；其清单与包是票据 | 用户可见 |
| Skill | 技能包 | 对象 | 用户可见 |
| Task | 任务 | 对象 | 用户可见 |
| Kanban | 看板 | Task 模块的场景名 | 用户可见 |
| Run | 一次受治理施工 | 对象 | 用户可见 |
| Workflow | 施工图 | 对象，也是 Run 模块的场景名 | 用户可见 |
| Terminal | 终端 | Participant 模块的场景名 | 用户可见 |
| Workbench | 工作台 | 客户端产品 | 用户可见 |
| Receipt | 凭证 | 票据 | 用户可见——「完成不能自述」靠它，愿景层要讲 |
| Gate | 评审关卡 | 节点类型 | 治理内部；愿景层只以中文「评审关卡」出现 |
| Obligation | 交付义务 | 对象 | 治理内部 |
| Seat | 席位 | 对象 | 治理内部 |
| Attempt | 尝试 | 对象 | 治理内部 |
| ChangeSet | 变更集 | 对象 | 治理内部 |
| Verdict | 裁决 | 票据 | 治理内部 |
| Evidence | 证据 | 票据 | 治理内部 |

「类别」回答它是什么东西：对象有独立生命周期、恢复边界或权限边界；票据是步骤产物，只被引用不被改写；节点类型和场景名不是对象。「可见性」回答它能出现在哪一层：愿景层只用标「用户可见」的词（含 Receipt），治理内部的词从架构层起可用。愿景层讲执行内部时用日常语言——「一步要交出什么是固定的，谁来做可以换，做坏了从这一次重来」——不点治理词。

设计正文还可以使用六个系统角色名：harness（编码代理工具）、chat server（聊天服务器）、task backend（任务后端）、workflow engine（工作流引擎）、Agency（参与者的派出方）和 worker（执行体，Agency 供给的一次具体运行）；权威定义见[三面架构](../architecture.md#场景与系统)。Agent 不是模块名，只作编码代理的泛称；描述数字参与者一律用 Participant。人不是 Participant：约束层写 human actor，设计层写「人」或「有权的人」。`provider` 只是供应端的泛称，必须由具体模块说明它指哪一类供应端。

另有八个高频约束词可在设计正文携中文对照使用：Task Revision（任务契约版本）、Workflow Revision（施工图版本）、Room Invocation（单次调用）、Execution Spec（执行规格）、Result Proposal（结果提案）、Run Manifest（施工清单）、Context Manifest（根上下文清单）、Context Bundle（消费上下文包）。[交付文档](../delivery.md)描述工程选型、里程碑和契约测试，因此可以直接使用约束层词汇。设计层正文——含仓库 README 与设计地图——仍只用核心产品词与上述八词。

## 六族规则

族的共同语义只在这里定义一次；成员在各模块约束中一行导出，不再重复解释这些性质。

| 族 | 共同语义 |
| --- | --- |
| Revision | 只追加的不可变版本；以 digest 精确引用；current pointer 只由类型化命令推进，界面只读取 |
| Binding | 把 HCTL 里的一个东西和外部系统里对应的东西钉在一起，钉的那一刻冻结成版本；正在跑的工作永远引用它准入时的版本，换绑不改历史。族里只有外部连接：HCTL 内部的授权（如 Project 的参与者授权）不是 Binding |
| Receipt | control 与工具箱校验通过后签发的证明；它只证明已校验的结果，本身不是另一个写入者 |
| Lease | 有期限、单持有者、可撤销的独占权；配合代次使用，旧代次一律失权 |
| 命令（Intent） | 改变事实的持久命令或副作用记录；携带 actor 来源、目标版本与幂等键，重复提交返回原结果；请求可来自 direct client 或模块明确接纳的 provider event |
| Snapshot | 先观测后准入的只追加外部观测；观测无论置信度多高都不直接改写事实 |

## 三类数据

每个场景的持久数据分三类；类别的共同语义只在这里定义一次，各模块约束按场景导出，不再重复解释。三个类别词可在设计正文携中文对照使用：

| 类别 | 中文对照 | 含义 | 权威所在 |
| --- | --- | --- | --- |
| metadata | 治理元数据 | 身份、绑定、授权与判决——谁是谁、谁连着谁、谁批了什么、凭什么算数 | HCTL 自己的账本（控制面） |
| content | 场景内容 | 各场景的协作与执行记忆：消息、任务卡与流转、机械执行历史、会话转录 | 该场景的 content 系统（第三方 ground truth，事实源头） |
| artifact | 结晶 | content 提炼出的不可变产物：决议与 Memo、冻结契约与施工图、凭证链、代码变更 | Git |

统一律：**每个场景的 artifact 是该场景 content 的结晶**。结晶归产生它的场景所有：讨论产生的决议、Memo 与施工图归 Room，任务验收产生的冻结契约归 Kanban，引擎执行产生的凭证链归 Workflow，会话中的代码修改归 Terminal。没有产物的场景不必为了形式对称而补造一种结晶。消歧：小写 artifact 是数据类别，中文一律写“结晶”；Artifact（工件）仍指 Project 模块登记的交付物对象，两者不同物。

三条法贯穿全部模块约束，各处引用，不再各写一套：

1. **能承载不等于能裁决。** content 系统拥有场景内容的 ground truth，但永远不拥有治理：普通消息不能触发派发，provider Done 最多请求同一 Task 验收，引擎的机械完成不能签发凭证。判决只在 metadata 层产生。
2. **冻结摘要是 content 与治理之间的防火墙。** content 可变，治理引用不可变；既有的 Snapshot 观测、采纳与 digest 冻结机制原样构成这道墙——授权执行前把依赖的 content 冻结为带摘要的精确引用，此后 content 漂移不改写已授权的事实。
3. **命令走 HCTL，记录落平台。** 类型化命令的预览、准入与判决在 metadata 层执行；human 请求可以来自 Workbench/CLI，也可以来自模块绑定明确接纳的 provider 动作，但必须归一到同一命令。结果可以作为记录写回 content 系统，回写本身不得再取得 human provenance。

## 词汇索引

- **Revision 族**：Task Revision、Workflow Revision、ChangeSet Revision、Artifact Revision、Extension Revision、Engine Deployment
- **Binding 族**（每个都是「HCTL 对象 ↔ 外部对象」）：Port–Provider Binding（受控端口 ↔ 供应端）、Room–Server Binding（Room ↔ 聊天服务器房间）、Task–Backend Binding（Task ↔ 任务后端的卡）、Run–Engine Binding（Run ↔ 工作流引擎执行）、Participant–Agency Binding（Participant ↔ 派出方名册项）
- **Receipt 族**：Gate Receipt、Task Completion Receipt、Integration Receipt
- **Lease 族**：Write Lease、Terminal Input Lease；control writer 和 Agency 归属者虽然不是 Lease 对象，也必须遵守同样的排他规则：同一时刻只有一个持有者，旧代次失去权限
- **命令族**：各模块的类型化命令（动宾语义名，如「完成 Task」命令），以及「外部副作用」命令
- **Snapshot/观测族**：Task Backend Snapshot、Result Proposal、运行时观测
- **票据与规格**：Execution Spec、Run Manifest、Attach Descriptor、Context Manifest、Context Bundle（场景投影如 Execution Chat 不占概念名额）
- **引用格式**：ReviewSubjectRef、review_subject_digest、revision_digest
- **独立对象**（核心产品词之外的约束层领域对象）：Repo Instance、Room Invocation、Execution Runtime、Worker Profile

## 外部对齐原则

每个模块约束带一张“外部概念对齐表”：HCTL 词 ↔ 外部体系词 ↔ 一句话差异。对齐用于翻译与第三方接入，不转移权威；外部对象不因概念对应而获得 HCTL 字段的写权。能直接用外部词说清的场合直接用外部词；自造词只保留外部体系没有的差异化语义，如 Obligation、Verdict/Receipt 和 Write Lease。

对齐表中的“无对应”只是引入差异化语义的强信号，不是控制面账本的完整存储清单。事实是否进入账本仍取决于它是否有独立生命周期、恢复或权限边界，以及 HCTL 是否拥有该事实。外部系统原生承载的可变 content 与内部拓扑不在账本复制；HCTL 自己的稳定身份、领域关系、授权、判决及必要绑定与摘要仍由控制面保存。

各模块受控端口的选型隔离、能力声明与替换边界见[避免供应商锁定](../architecture.md#避免供应商锁定)。

## 文件

- [project.md](./project.md)：Project 模块约束 + Room 场景对齐（Matrix / Slack 系）
- [task.md](./task.md)：Task 模块约束 + Linear / GitHub 对齐
- [run.md](./run.md)：Run 模块约束 + Dagu / BPMN 对齐
- [participant.md](./participant.md)：Participant 模块约束 + Skill 申报 + PTY / Herdr / ACP 对齐
- [connections.md](./connections.md)：四模块交接、事务边界与跨切恢复
- [system.md](./system.md)：组件、共享机制、存储、单写者与恢复

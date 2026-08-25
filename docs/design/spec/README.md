# 合同层总则

> 状态：规范性 · 草案 v0.13.0<br>
> 日期：2026-08-25<br>
> 定位：本目录是 HCTL2 的合同层——精确的对象、状态机、写入者与共享机制。设计层（`docs/design/` 根目录）用产品语言回答为什么与怎么用；两层冲突时以合同层为准，但合同层不得引入设计层没有的产品行为。

## 词汇分类法

具名概念分四类；只有前两类可以引入新造的对象名，状态值与引用格式只是枚举与格式，不占概念名额：

| 类别 | 判据 | 例子 |
| --- | --- | --- |
| 领域对象 | 独立生命周期、恢复边界或权限边界至少居其一 | Task、Run、Seat、ChangeSet、Request |
| 票据与记录 | 某个步骤的产物：只追加、短期或一次性，但需要被精确引用 | Execution Spec、Attach Descriptor、Receipt、外部副作用命令、Snapshot |
| 状态值 | lifecycle 枚举，不是对象 | 开放、待采纳、结果未知 |
| 引用格式 | 指认他物的结构化引用，不是对象 | ReviewSubjectRef、digest |

新名字的引入门槛：不满足前两类判据的不得命名；能用日常语言或外部标准词说清的不另造词。自造语义名不冻结代码词形：具名对象与票据写成带空格的专名（如 Task Revision、Gate Receipt），命令写动宾语义名（如「完成 Task」命令），状态值写中文语义名；实现时附「语义名 ↔ 标识符」对照表回溯合同。合同需要逐字指认的协议或 schema 字段、序列化格式标识，以及外部标准、产品或源码中的原名，可以保留原形并用代码格式标示；它们不因此成为新的领域概念。设计层正文不使用合同层词汇，只使用下列核心产品词。

## 核心产品词

Repo、Project、Room、Participant、Request、Memo、Artifact、Context、Skill、Task、Kanban、Run、Workflow、Obligation、Seat、Attempt、Gate、Verdict、Receipt、Agent、Terminal、ChangeSet、Evidence、Workbench。

另设四个**系统角色名**，指各场景 content 的承载系统，可在设计正文直接使用：harness（编码代理工具，如 Codex、Claude Code、OpenCode）、chat server（聊天服务器）、task backend（任务后端）、workflow engine（工作流引擎）；权威定义见[三面架构](../architecture.md#场景与系统)。“Agent”一词专属第四模块；散文中的 AI 协作者用 Participant 表述，需要区分人与模型时加“模型”限定词。

另有六个高频合同词可在设计正文携中文对照使用：Task Revision（契约版本）、Workflow Revision（施工图版本）、Room Invocation（单次调用）、Execution Spec（执行规格）、Result Proposal（结果提议）、Run Manifest（施工清单）。[交付文档](../delivery.md)（工程选型、里程碑与契约测试）与合同层同侧，可直接使用合同层词汇；设计层正文——含仓库 README 与设计地图——仍只用核心产品词与上述六词。

## 六族规则

族的共同语义只在这里定义一次；成员在各模块合同中一行导出，不再重复解释这些性质。

| 族 | 共同语义 |
| --- | --- |
| Revision | 只追加的不可变版本；以 digest 精确引用；current pointer 只由类型化命令推进，界面只读取 |
| Binding | 把两个身份连起来的冻结解析；活动执行永远引用准入时的版本，换绑不改写历史 |
| Receipt | control 与工具箱校验通过后签发的证明；它只证明已校验的结果，本身不是另一个写入者 |
| Lease | 有期限、单持有者、可撤销的独占权；配合代次使用，旧代次一律失权 |
| 命令（Intent） | 改变事实的持久命令或副作用记录；携带 actor、目标版本与幂等键，重复提交返回原结果 |
| Snapshot | 先观测后准入的只追加外部观测；观测无论置信度多高都不直接改写事实 |

## 三类数据

每个场景的持久数据分三类；类别的共同语义只在这里定义一次，各模块合同按场景导出，不再重复解释。三个类别词可在设计正文携中文对照使用：

| 类别 | 中文对照 | 含义 | 权威所在 |
| --- | --- | --- | --- |
| metadata | 治理元数据 | 身份、绑定、授权与判决——谁是谁、谁连着谁、谁批了什么、凭什么算数 | HCTL 自己的账本（控制面） |
| content | 场景内容 | 各场景的协作与执行记忆：消息、任务卡与流转、机械执行历史、会话转录 | 该场景的 content 系统（第三方 ground truth，事实源头） |
| artifact | 结晶 | content 提炼出的不可变产物：决议与 Memo、冻结契约与施工图、凭证链、代码变更 | Git |

统一律：**每个场景的 artifact 是该场景 content 的结晶**——讨论结晶为决议、Memo 与施工图（“干什么的计划”从塑形讨论中长出），任务流转结晶为冻结契约，机械执行结晶为凭证链，会话字节流结晶为代码变更。归属以事实为准绳：结晶从哪个场景长出来就归哪个场景，不为对称硬填。消歧：小写 artifact 是数据类别，中文一律写“结晶”；Artifact（工件）仍指 Project 模块登记的交付物对象，两者不同物。

三条法贯穿全部模块合同，各处引用，不再各写一套：

1. **能承载不等于能裁决。** content 系统拥有场景内容的 ground truth，但永远不拥有治理：平台消息不能触发派发，拖卡不能完成 Task，引擎的机械完成不能签发凭证。判决只在 metadata 层产生。
2. **冻结摘要是 content 与治理之间的防火墙。** content 可变，治理引用不可变；既有的 Snapshot 观测、采纳与 digest 冻结机制原样构成这道墙——授权执行前把依赖的 content 冻结为带摘要的精确引用，此后 content 漂移不改写已授权的事实。
3. **命令走 HCTL，记录落平台。** 类型化命令的预览、准入与判决在 metadata 层执行；结果可以作为记录写回 content 系统，但平台里的记录只是记录，不是命令。

## 词汇索引（v0.9.1 归并后）

- **Revision 族**：Task Revision、Workflow Revision、ChangeSet Revision、Artifact Revision、Extension Revision、Engine Deployment
- **Binding 族**：Resolved Port Binding、Task Binding、Project Role Binding、Engine Execution Binding
- **Receipt 族**：Gate Receipt、Task Completion Receipt、Integration Receipt
- **Lease 族**：Write Lease、Terminal Input Lease；control writer 与 agentd owner 的排他权同族（以 generation 表达）
- **命令族**：各模块的类型化命令（动宾语义名，如「完成 Task」命令），以及「外部副作用」命令
- **Snapshot/观测族**：Task Source Snapshot、Result Proposal、运行时观测
- **票据与规格**：Execution Spec、Run Manifest、Attach Descriptor、Context Manifest、Context Bundle（场景投影如 Execution Chat 不占概念名额）
- **引用格式**：ReviewSubjectRef、review_subject_digest、revision_digest
- **独立对象**（核心产品词之外的合同层领域对象）：Repo Instance、Room Invocation、Execution Runtime、Worker Profile

## v0.9.1 归并对照

| 旧名 | 现状 |
| --- | --- |
| InvocationBinding / AttemptSpec | 合并为 Execution Spec（owner = Room Invocation \| Attempt） |
| RuntimeShard / InvocationRuntime | 合并为 Execution Runtime（owner 字段） |
| TerminalBundle | Execution Runtime 的终端通道字段组 |
| HarnessAdapterBinding | Execution Spec 冻结的接入方式字段组 |
| IntegrationIntent / ExternalEffectIntent | 合并为外部副作用命令（executor = tool 本地 Git \| adapter 远端） |
| TaskSourceConnection / TaskSourceConnectionRevision | 由 Resolved Port Binding（port_kind = task_source）承载 |
| ChatSurfaceBindingRevision | Room 的 Chat 端口绑定字段组（引用 Resolved Port Binding） |
| TaskSourceBindingRevision（及裸用 BindingRevision） | Task Binding |
| EngineDeploymentRevision | Engine Deployment |
| ChangeSetWriteLease | Write Lease |
| HarnessDefinition / Installation / Capability | “Harness 目录”的三类探测事实，无类名 |
| TerminalGateway / WorkflowEngineAdapter | 描述性说法：agentd 的终端网关 / workflow engine 端口适配器 |

## v0.10.3 清扫

| 旧名 | 现状 |
| --- | --- |
| RuntimeBackend | 描述性说法：运行时后端（受控端口与物理资源持有者，无对象名） |
| TaskSource | 端口种类 `port_kind = task_source`；散文写「任务源端口」 |
| WorkflowEngine | 系统角色小写 workflow engine；端口写「workflow engine 端口」 |
| HarnessAdapter | 描述性说法：harness 适配器 |

## v0.11.1 词形收敛

| 旧形 | 新形 |
| --- | --- |
| 驼峰对象/票据名（TaskRevision 等 25 个） | 带空格专名（Task Revision 等），对齐 Run Manifest / Gate Receipt 先例 |
| `*Intent` 命令名（16 个） | 动宾语义名（「完成 Task」命令等）；代码标识符由实现仓库定，实现时附对照表 |
| 状态值枚举拼写 | 中文语义名（待采纳、结果未知、等待输入等） |

ChangeSet 保留原形（核心产品词、业界成词）；字段与格式名（`port_kind`、`review_subject_digest`、ReviewSubjectRef 等）在合同需要逐字指认时保留原形，不受自造语义名的词形规则约束。

## v0.12.2 清扫

三类数据切分落地后按概念门槛复查的降级与统一：

| 旧名 / 旧词 | 现状 |
| --- | --- |
| Room Event | 除名：消息 content 本体就是 chat server 的 Matrix event；HCTL 侧只有账本内只追加的"治理事件"（以事件 ID 精确引用消息），两者都不占领域对象名额 |
| Task Operational State | 降级为 Task Binding 的字段组"操作投影"（后端操作字段的回读投影、同步账与派生健康状态）；ground truth 在 content 后端 |
| 状态值"中断"（Room Invocation） | 统一为"丢失"：执行身份无法证明时 Room Invocation 与 Attempt 进入同一状态；收口规则只在[连接合同](./connections.md#失败与恢复)定义一次 |

## v0.13.0 收窄

| 旧名 / 旧词 | 现状 |
| --- | --- |
| 用户在场证明 | 撤销：human provenance 由经认证的 Workbench/CLI 会话直接赋予；agentd 启动的执行环境内发出的 CLI 调用以 execution principal 提交 |
| OS 沙箱入场券 | 降为可选执行加固：由 Worker Profile 声明、Execution Spec 冻结、agentd 记录为事实；三条底线（工具不是人 / 合入钥匙不进工具 / 隔离工作树）单独保留 |
| “不得读取目标 ref/common-dir” | 删：Harness 可读 common-dir/refs 并在本 ChangeSet 分支提交；直写目标 ref 不取得集成 authority，只回读为 drift |
| Engine 检查点 execution identity / engine attempt generation | 退出 Obligation 身份：Obligation 按 Run、节点与观察序号铸造，Engine 的 run ID/step 名只作关联键；代次、deadline、完成谓词只在账本 |
| Room 的“加密/降级”状态 | 不设：房间端到端加密状态是 Chat 端口绑定的 health 投影，不进不可变 binding，也不是 Room 的 lifecycle 值；可观察结果只在[连接合同失败表](./connections.md#失败与恢复)登记一行，恢复动作是既有的「换绑」命令 |
| P0 中的第三方自身功能项 | 移出 P0：属选型资料判断或首次消费前的产品化；P0 只验 HCTL 与该系统的接缝 |

## 外部对齐原则

每个模块合同带一张“外部概念对齐表”：HCTL 词 ↔ 外部体系词 ↔ 一句话差异。对齐用于翻译与第三方接入，不转移权威——外部对象不因概念对应而获得 HCTL 字段的写权。能直接用外部词说清的场合直接用外部词；自造词只保留外部体系没有的差异化语义（如 Obligation、Verdict/Receipt、Write Lease）。对齐表中的「无对应」只是引入差异化语义的强信号，不是控制面账本的完整存储清单；是否进入账本仍取决于它是否有独立生命周期、恢复或权限边界，以及 HCTL 是否拥有该事实。外部系统原生承载的可变 content 与内部拓扑不在账本复制，但 HCTL 自己的稳定身份、领域关系、授权、判决及必要绑定与摘要仍由控制面保存。

## 文件

- [project.md](./project.md)：Project 模块合同 + Chat 场景对齐（Matrix / Slack 系）
- [task.md](./task.md)：Task 模块合同 + Linear / GitHub 对齐
- [run.md](./run.md)：Run 模块合同 + Dagu / BPMN 对齐
- [agent.md](./agent.md)：Agent 模块合同 + PTY / tmux / ACP 对齐
- [connections.md](./connections.md)：四模块交接、事务边界与跨切恢复
- [system.md](./system.md)：组件、共享机制、存储、单写者与恢复

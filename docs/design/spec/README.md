# 合同层总则

> 状态：规范性 · 草案 v0.9.1<br>
> 日期：2026-08-19<br>
> 定位：本目录是 HCTL2 的合同层——精确的对象、状态机、写入者与共享机制。设计层（`docs/design/` 根目录）用产品语言回答为什么与怎么用；两层冲突时以合同层为准，但合同层不得引入设计层没有的产品行为。

## 词汇分类法

具名概念分四类；只有前两类可以引入新造的对象名，状态值与引用格式只是枚举与格式，不占概念名额：

| 类别 | 判据 | 例子 |
| --- | --- | --- |
| 领域对象 | 独立生命周期、恢复边界或权限边界至少居其一 | Task、Run、Seat、ChangeSet、Request |
| 票据与记录 | 某个步骤的产物：只追加、短期或一次性，但需要被精确引用 | ExecutionSpec、AttachDescriptor、Receipt、EffectIntent、Snapshot |
| 状态值 | lifecycle 枚举，不是对象 | Open、PendingAdoption、ResultUnknown |
| 引用格式 | 指认他物的结构化引用，不是对象 | ReviewSubjectRef、digest |

新名字的引入门槛：不满足前两类判据的不得命名；能用日常语言或外部标准词说清的不另造词。设计层正文不使用合同层词汇，只使用下列核心产品词。

## 核心产品词

Repo、RepoInstance、Project、Room、Participant、Request、Memo、Artifact、Context、Skill、Task、Kanban、Run、Workflow、Obligation、Seat、Attempt、Gate、Verdict、Receipt、Agent、Terminal、ChangeSet、Evidence、Workbench。

另设四个**系统角色名**，指各场景 content 的承载系统，可在设计正文直接使用：harness（编码代理工具，如 Codex、Claude Code、OpenCode）、chat server（聊天服务器）、task backend（任务后端）、workflow engine（工作流引擎）；权威定义见[三面架构](../architecture.md#场景与系统)。“Agent”一词专属第四模块；散文中的 AI 协作者用 Participant 表述，需要区分人与模型时加“模型”限定词。

另有六个高频合同词可在设计正文携中文对照使用：TaskRevision（契约版本）、WorkflowRevision（施工图版本）、RoomInvocation（单次调用）、ExecutionSpec（执行规格）、ResultProposal（结果提议）、Run Manifest（施工清单）。`*Intent` 命令名只出现在合同层。

## 六族规则

族的共同语义只在这里定义一次；成员在各模块合同中一行导出，不再重复解释这些性质。

| 族 | 共同语义 |
| --- | --- |
| Revision | 只追加的不可变版本；以 digest 精确引用；current pointer 只由类型化命令推进，界面只读取 |
| Binding | 把两个身份连起来的冻结解析；活动执行永远引用准入时的版本，换绑不改写历史 |
| Receipt | control/core 校验通过后签发的证明；它只证明已校验的结果，本身不是另一个写入者 |
| Lease | 有期限、单持有者、可撤销的独占权；配合代次使用，旧代次一律失权 |
| Intent | 改变事实的持久命令或副作用记录；携带 actor、目标版本与幂等键，重复提交返回原结果 |
| Snapshot | 先观测后准入的只追加外部观测；观测无论置信度多高都不直接改写事实 |

## 三类数据

每个场景的持久数据分三类；类别的共同语义只在这里定义一次，各模块合同按场景导出，不再重复解释。三个类别词可在设计正文携中文对照使用：

| 类别 | 中文对照 | 含义 | 权威所在 |
| --- | --- | --- | --- |
| metadata | 治理元数据 | 身份、绑定、授权与判决——谁是谁、谁连着谁、谁批了什么、凭什么算数 | HCTL 自己的账本（控制面） |
| content | 场景内容 | 各场景的协作与执行记忆：消息、任务卡与流转、机械执行历史、会话转录 | 该场景的 content 系统（第三方 ground truth，事实源头） |
| artifact | 结晶 | content 提炼出的不可变产物：决议与 Memo、冻结契约与施工图、凭证链、代码变更 | Git |

统一律：**每个场景的 artifact 是该场景 content 的结晶**——讨论结晶为决议与 Memo，任务流转结晶为冻结契约与施工图，机械执行结晶为凭证链，会话字节流结晶为代码变更。消歧：小写 artifact 是数据类别，中文一律写“结晶”；Artifact（工件）仍指 Project 模块登记的交付物对象，两者不同物。

三条法贯穿全部模块合同，各处引用，不再各写一套：

1. **能承载不等于能裁决。** content 系统拥有场景内容的 ground truth，但永远不拥有治理：平台消息不能触发派发，拖卡不能完成 Task，引擎的机械完成不能签发凭证。判决只在 metadata 层产生。
2. **冻结摘要是 content 与治理之间的防火墙。** content 可变，治理引用不可变；既有的 Snapshot 观测、采纳与 digest 冻结机制原样构成这道墙——授权执行前把依赖的 content 冻结为带摘要的精确引用，此后 content 漂移不改写已授权的事实。
3. **命令走 HCTL，记录落平台。** 类型化命令的预览、准入与判决在 metadata 层执行；结果可以作为记录写回 content 系统，但平台里的记录只是记录，不是命令。

## 词汇索引（v0.9.1 归并后）

- **Revision 族**：TaskRevision、WorkflowRevision、ChangeSetRevision、ArtifactRevision、ExtensionRevision、EngineDeployment
- **Binding 族**：ResolvedPortBinding、TaskBinding、ProjectRoleBinding、EngineExecutionBinding
- **Receipt 族**：Gate Receipt、TaskCompletionReceipt、IntegrationReceipt
- **Lease 族**：WriteLease、TerminalInputLease；control writer 与 agentd owner 的排他权同族（以 generation 表达）
- **Intent 族**：各模块的 `*Intent` 命令，以及承载外部副作用的 EffectIntent
- **Snapshot/观测族**：TaskSourceSnapshot、ResultProposal、运行时观测
- **票据与规格**：ExecutionSpec、Run Manifest、AttachDescriptor、ContextManifest、ContextBundle（场景投影如 Execution Chat 不占概念名额）
- **引用格式**：ReviewSubjectRef、review_subject_digest、revision_digest
- **独立对象**（核心产品词之外的合同层领域对象）：RoomInvocation、RoomEvent、ExecutionRuntime、WorkerProfile、TaskOperationalState

## v0.9.1 归并对照

| 旧名 | 现状 |
| --- | --- |
| InvocationBinding / AttemptSpec | 合并为 ExecutionSpec（owner = RoomInvocation \| Attempt） |
| RuntimeShard / InvocationRuntime | 合并为 ExecutionRuntime（owner 字段） |
| TerminalBundle | ExecutionRuntime 的终端通道字段组 |
| HarnessAdapterBinding | ExecutionSpec 冻结的接入方式字段组 |
| IntegrationIntent / ExternalEffectIntent | 合并为 EffectIntent（executor = core 本地 Git \| adapter 远端） |
| TaskSourceConnection / TaskSourceConnectionRevision | 由 ResolvedPortBinding（port_kind = task_source）承载 |
| ChatSurfaceBindingRevision | Room 的 Chat 端口绑定字段组（引用 ResolvedPortBinding） |
| TaskSourceBindingRevision（及裸用 BindingRevision） | TaskBinding |
| EngineDeploymentRevision | EngineDeployment |
| ChangeSetWriteLease | WriteLease |
| HarnessDefinition / Installation / Capability | “Harness 目录”的三类探测事实，无类名 |
| TerminalGateway / WorkflowEngineAdapter | 描述性说法：agentd 的终端网关 / WorkflowEngine 端口适配器 |

## 外部对齐原则

每个模块合同带一张“外部概念对齐表”：HCTL 词 ↔ 外部体系词 ↔ 一句话差异。对齐用于翻译与第三方接入，不转移权威——外部对象不因概念对应而获得 HCTL 字段的写权。能直接用外部词说清的场合直接用外部词；自造词只保留外部体系没有的差异化语义（如 Obligation、Verdict/Receipt、WriteLease）。

## 文件

- [project.md](./project.md)：Project 模块合同 + Chat 场景对齐（Matrix / Slack 系）
- [task.md](./task.md)：Task 模块合同 + Linear / GitHub 对齐
- [run.md](./run.md)：Run 模块合同 + Conductor / BPMN 对齐
- [agent.md](./agent.md)：Agent 模块合同 + PTY / tmux / ACP 对齐
- [connections.md](./connections.md)：四模块交接、事务边界与跨切恢复
- [system.md](./system.md)：组件、共享机制、存储、单写者与恢复

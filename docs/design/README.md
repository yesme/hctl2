# HCTL2 设计地图

> 状态：规范性索引 · 草案 v0.8.1<br>
> 日期：2026-08-19

HCTL2 只有四个领域模块。每个模块拥有稳定身份、状态、命令和不变量；与它对应的场景只提供查询、预览、操作和事件投影。

| 权威模块 | 对应场景 | 模块拥有 | 场景客户端 / 受控端口示例 |
| --- | --- | --- | --- |
| [Project](./project.md) | Chat Room | RepoInstance、Project、Room、Context、Request、Memo、Artifact | Workbench Room / 外部 Chat 端口 |
| [Task](./task.md) | Kanban | Task、TaskRevision、操作态、来源绑定、完成证明 | Workbench Board / Linear、GitHub TaskSource |
| [Run](./run.md) | Workflow | WorkflowRevision、Run、Obligation、Seat、Attempt、Verdict/Receipt | Workbench Run 图 / Conductor 端口 |
| [Harness](./harness.md) | Terminal | Worker/Harness 绑定、ChangeSet、运行时、终端、执行证据 | Workbench xterm、CLI / ACP、Harness、RuntimeBackend |

场景与模块是一一对应的主视角，不是强制的调用链。Task 可以没有 Run；Project 可以发起一次 Harness 调用；Kanban 可以显示 Run 和 Artifact 投影。跨模块引用不转移事实所有权。

## 场景客户端与受控端口

Workbench、CLI 与第三方原生 UI 作为场景客户端，只使用四类操作：

1. 查询当前投影；
2. 预览类型化命令及前置条件；
3. 提交类型化命令；
4. 订阅带序号的领域事件或重同步快照。

Workbench 把四个场景集成在一个客户端中，但没有额外权限。第三方平台可以实现部分场景客户端，也可以通过 Chat/TaskSource/WorkflowEngine/Harness/RuntimeBackend 受控端口提供底层能力；受控端口只报告读写能力和降级方式，字段权威由对应模块的 authority binding 授予。同一产品兼任两者时 client binding 与 authority binding 仍须分开。未实现或无权执行的动作必须隐藏或安全拒绝。平台自己的 Session、Issue、Workflow Task、pane 或数据库都不能成为 HCTL 的第五个事实源。

## 共同规则

- 稳定对象使用稳定 ID；内容变化产生不可变 Revision，界面使用 current pointer 或操作投影。
- 正式变化只由类型化命令产生；命令携带 actor、目标、预期版本、权限范围和幂等键。
- 外部事件先成为观测或提案；获准命令与持久 outbox 原子提交，结果未知时先回读。
- Task 只由有权 human actor 的 Kanban 命令，或 task-bound Run 正常完成后 reducer 提交的同一个 CompleteTaskIntent 终结；Harness、模型、拖卡、进程退出、Git commit、CI 或外部 Closed 都不能冒充该命令，失败类 Run 也不能取消 Task。
- 普通 Room 的临场执行边只由 human actor 提交；Agent 可以建议下一位 Participant，但不能从消息正文自行 cue、扩大 fan-out 或递归委派。预授权自动边只由 Workflow reducer 按冻结 WorkflowRevision 创建。
- 运行中的绑定被冻结；能力、权限、候选或验收条件变化时创建新 Revision 或替代执行。
- Workbench 关闭不改变领域事实；缺少等价适配能力时安全暂停，而不是绕过 command service。
- 同一 RepoInstance 只有一个 control 写入者，同一 RuntimeBackend scope 只有一个 agentd owner；旧 generation 失权。

四模块之间“交什么、谁准入、怎样恢复”只在[连接合同](./connections.md)定义一次；CAS、outbox、单写者和适配器恢复等通用机制只在[系统边界](./system.md)定义一次。模块文档不再各写一套副本。

## 文档纪律

为避免再次形成补丁链，后续修改遵守以下硬边界：

- 只有四个模块文件可以定义模块特有的领域名词、状态、写入者和不变量；场景不得重定义它们。
- `connections.md` 只定义模块交接，`system.md` 只定义共享机制，`delivery.md` 只定义范围与验证，evidence 只记录来源。
- 一个概念只在拥有它的模块完整定义一次；其他文件用链接和可观察结果引用。
- 新持久对象必须对应第一阶段中的稳定引用、命令目标或恢复边界；否则先作为实现细节。
- 精简只针对重复权威、无第一阶段用途的对象和补丁衍生对象；能够回答独立实现选择、交接、故障或权限边界的设计不得因篇幅被删除。
- 新持久对象必须说明现有命令、引用或事件为什么无法承载该边界；若同一规则需要在多个权威位置同步修改，先选定唯一 owner，再把其他位置改为引用。
- 审计只针对稳定快照，最多两轮；第二轮若主要发现第一轮新增概念造成的问题，则回滚而不是继续打补丁。

## 支持文档

- [系统边界与适配器合同](./system.md)：组件、事实源、命令、单写者与恢复。
- [四模块连接与端到端闭环](./connections.md)：类型化交接、事务边界、版本链和跨切恢复。
- [第一阶段、验证与自举](./delivery.md)：交付范围、CLI、纵向切片、契约测试和未决项。
- [从 HCTL 到 HCTL2 的来时路](./references/decision-history.md)：关键决策转折的非规范说明；它不形成第二套合同。
- [实现证据](./references/implementation-evidence.md)：固定版本、许可证和采用边界；它不定义 HCTL 语义。

发生冲突时，四个模块文件解释连接端点，`connections.md` 解释交接，`system.md` 解释共享执行机制；`delivery.md` 不得改变领域含义，实现证据不得反向定义产品。

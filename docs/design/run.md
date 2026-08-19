# Run 与 Workflow

> 本文是 Run 模块的唯一领域权威；Workflow 是其操作合同，不拥有独立领域事实。模块交接见[连接合同](./connections.md)，通用机制见[系统边界](./system.md)。

## 为什么存在

行业并不缺工作流引擎：DAG、代理委派、重试、定时器和历史恢复都已被反复实现。真正缺失的，是一套面向项目语义、绑定精确版本与证据的治理层——它回答“这个节点对应哪份交付义务、谁有资格尝试、结果是否有效、凭什么可以算完成”。这件事不能委托给聊天室、看板、Harness、终端或任何通用引擎；它是 Run 模块的原生价值。

两种权威一句话可以分清：

> 引擎说：“这个外部节点 READY / IN_PROGRESS / COMPLETED 了。”<br>
> HCTL2 说：“它对应哪份交付义务（Obligation），谁可以尝试（Seat 与候选），结果是否有效（Verdict），是否可以完成（准入校验）。”

引擎拥有机械位置，HCTL 拥有语义治理；两边不能互相冒充。

## 模块职责

Run 模块保存“哪份自动施工已获授权、机械进度如何映射为语义结果”。Workflow 是它的操作与引擎场景，不是第五个领域模块。

| 对象 | 含义 |
| --- | --- |
| WorkflowRevision | 与引擎无关、不可变的控制图、节点、输入输出和治理规则 |
| EngineDeploymentRevision | 某个 WorkflowRevision 经固定编译器和 WorkflowEngineAdapter 生成的引擎版本 |
| Run | 对冻结 Workflow、Task、基线、角色、候选、权限和预算的一次授权执行 |
| EngineExecutionBinding | Run、EngineDeploymentRevision、ResolvedPortBinding、外部 execution ID/correlation key 和 generation 的稳定映射 |
| Obligation | 一个 HCTL 外部节点必须产出的逻辑结果 |
| Seat | Obligation 中稳定的逻辑执行者或投票位置 |
| Attempt / AttemptSpec | 某个候选对 Seat 的一次执行及其不可变分派规格 |
| ReviewSubjectRef | 对 ChangeSetRevision 或 ArtifactRevision 的精确 kind、ID 和 digest 引用 |
| Verdict / Receipt | 对精确版本的语义裁决，以及 control/core 校验后的正式证明 |

## 写入合同

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| WorkflowRevision / EngineDeploymentRevision | immutable revision + approval version | control 协调 Register、Compile、Approve Intent；固定 compiler/adapter 产出，core 校验摘要 | Revision 不改写；新内容创建新 Revision |
| Run / Manifest | `run_version`；`Starting / Running / Pausing / Paused / Cancelling / Completed / Failed / Cancelled / Superseded` | control 处理 StartRunIntent、Pause/Resume/Cancel/Replace Run Intent | Completed、Failed、Cancelled、Superseded 不复活；替代创建新 Run |
| EngineExecutionBinding | binding generation；`Starting / Bound / Closed / Diverged` | control 经 WorkflowEngineAdapter 启动、回读、关闭或标记分歧 | 外部 execution ID 只作绑定，不成为 Run 身份 |
| Obligation / Seat | state version；`Active / Satisfied / Failed / Cancelled / Superseded` | Run reducer/control 根据 Engine task、Attempt 结果和 Gate 策略推进 | 终态不可复活；Engine retry 创建新 Obligation |
| Attempt | `attempt_generation` + state version；合法边见下文 | control 创建、取消、替代和准入结果；agentd 只返回观测 | 终态不可复活；候选切换创建同 Seat 的新 Attempt |
| Verdict / Receipt | immutable | 只有 Run reducer 与 control/core 校验事务可写 | 精确绑定 ReviewSubjectRef、规则和证据 |

Run 合法边固定为：`Starting → Running/Failed/Cancelled/Superseded`；`Running → Pausing/Cancelling/Completed/Failed/Superseded`；`Pausing → Paused/Cancelling/Failed/Superseded`；`Paused → Running/Cancelling/Failed/Superseded`；`Cancelling → Cancelled/Failed/Superseded`。每个过渡态都必须能被取消、失败或替代路径收口，不能因 Engine 失联永久阻塞绑定 Task。外部 ACK 不直接写状态，control 只依据匹配 binding/generation 的回读推进。

Workflow Node、Engine task execution、Obligation、Seat 和 Attempt 是不同身份。Obligation 的不可变绑定固定 EngineExecutionBinding generation 与精确 Engine task execution identity（外部 task ID 及 retry/attempt generation）；其带版本的租约视图记录生效租约、截止时间和最近一次由 adapter 回读确认的续租，超时与备用候选准入只能依据该确认值。Engine retry 创建新 Obligation 前，control 必须在同一领域事务中把旧 Obligation 及其未终态 Seat/Attempt 置为 `Superseded`，令其派发、写入与输入授权失效，并提交物理隔离 outbox；旧执行的心跳、投票和迟到结果此后只留审计。技术性候选切换只在同一 Seat 下创建新 Attempt，不增加票数或更换逻辑裁判。

## Workflow 与 Run 授权

WorkflowRevision 使用 HCTL 规范化 JSON，经过数据结构、Profile 和语义校验后写入 Git。EngineDeploymentRevision 固定编译器、Profile、引擎适配器、绑定版本和引擎定义摘要。引擎产物不能反向定义 WorkflowRevision。

塑形与施工是[两种控制制度](./vision.md#两种控制制度)：批准施工图时推进权还在人手里，开工之后 control 才在冻结边界内自动推进。Run r1 按冻结版本施工时，Project Room 可以继续讨论 r2——讨论不必等施工结束，施工也不会随讨论漂移。

Approve Workflow 只确认施工图；`StartRunIntent` 才授予资源和副作用权。Run Manifest 至少冻结：

- Project、0..1 个 TaskRevision（第一阶段）、WorkflowRevision 与 EngineDeploymentRevision；
- repo/base revision、角色与逻辑 Seat；
- 获准 WorkerProfile 候选、切换规则、能力和权限；
- Gate、预算、放置和截止规则。

第一阶段，绑定 TaskRevision 的 Run 表示对该完整 Task 验收合同的一次施工授权，因此只有它正常 `Completed` 才具备提交 Task 完成命令的资格。只覆盖局部研究、咨询或中间步骤的自动化必须使用无 Task Run 或 RoomInvocation，并以稳定引用把结果交回 Task；不能绑定 Task 后再依靠 Prompt 声明“这次不算完整施工”。

运行中只有 Manifest 明确声明为可变的放置参数可以按冻结规则和边界调整；每次调整都校验预期 Run version，并留下固定前后值、适用规则、actor 和 Run version 的不可变审计事件。范围、验收、候选、权限、Gate 或超出获准边界的放置变化必须创建替代 Run，不能原地漂移。

第一阶段 Profile 允许外部执行、fork/join、switch、loop、dynamic fork、timer wait、noop 和经审计的纯数据转换；明确拒绝 `SUB_WORKFLOW`、Conductor `HUMAN` task 和绕过 control 的 HTTP/JDBC/Kafka/Git/Harness 副作用。dynamic fork 只能实例化 WorkflowRevision/Manifest 已冻结的有界 Seat 模板：候选 Participant/Role、最大基数、预算、选择函数和权限上限都必须预先固定；模型输出不能新增 recipient、扩大 fan-out 或扩权，无法机械校验时整次 fork 拒绝。

## 从节点到结果

本节定义 Run 内部归约；跨 Harness 的派发、结果信封和故障恢复见[连接合同](./connections.md)。

1. control 领取一个 HCTL 外部 Engine task，并按 Run、binding generation 与精确 Engine task execution identity 幂等创建唯一 Obligation；Engine 的 join/switch/wait 等机械节点不创建 Obligation。
2. control 按规则创建 Seat，并为候选产生 AttemptSpec。
3. [Harness](./harness.md) 执行 Attempt，只能返回 ResultProposal、Revision 和证据。
4. control/core 校验精确 binding、代次、权限、ReviewSubjectRef 和证据；通过后形成 Seat 结果、Verdict 或 Receipt。
5. 领域结果与 Engine completion outbox 先持久提交，再幂等完成外部任务。

AttemptSpec 至少冻结 attempt/seat/run/generation、逻辑 Participant/Seat identity、WorkerProfile、HarnessAdapterBinding、RuntimeBackend binding、可选 ChangeSet/写租约、Context/Skill/能力/权限摘要和截止时间。Attempt lifecycle 为 `Pending | Running | WaitingForInput | ResultProposed | Failed | Lost | Cancelled | Superseded`：`Pending` 可进入 `Running/Failed/Lost/Cancelled`；`Running` 与 `WaitingForInput` 可互转并进入 `ResultProposed/Failed/Lost/Cancelled`；任一尚未提交 Proposal 的非终态可进入 `Superseded`。`ResultProposed` 是“该 Attempt 已提交不可变 Proposal”的终态，不表示 Seat、Gate、Run 或 Task 成功；owner 对 Proposal 的准入或拒绝推进 Seat/Obligation，修正或重新施工创建新 Attempt/Proposal，而不复活旧 Attempt。状态只由 control 根据 agentd 观测推进，全部终态不可复活。

## Request、重试与 Gate

Run 需要输入时向 Project 提交类型化 [Request](./project.md) 创建命令，只阻塞声明的范围；Project 独占 Request lifecycle。Request 冻结 deadline 与 `fail | cancel` 默认策略；Resolve/Expire 的跨模块事务都 CAS 精确 Request 与 blocker version，只有 Resolve 可以写答案 delivery，Expire 不能猜测答案而是按冻结策略收口对应 Attempt/Seat/Obligation。Run 只在匹配 ACK/观测后恢复绑定执行；节点仍通过正常 ResultProposal/Receipt 路径完成，不存在第二条 human-task 完成路径。

“重试”不是一个概念，而是五种身份不同的路径；混用它们会复制票数、绕过验收或复活旧执行：

| 路径 | 触发 | 产生的新身份 | 不变的身份 |
| --- | --- | --- | --- |
| 传输重投 | 投递超时、ACK 丢失 | 无：同一幂等键重投，重复命令返回原结果 | 一切领域对象 |
| 候选切换 | 类型化技术故障 | 同一 Seat 下的新 Attempt | Obligation、Seat、票位 |
| Engine retry | 引擎机械重试 | 新 Obligation（旧 Obligation 及其 Seat/Attempt 置为 Superseded） | Run |
| 语义返工 | changes_requested 汇总 | 新 ChangeSetRevision/ArtifactRevision，旧票失效并完整 regate | Run、TaskRevision（通常） |
| 替代执行 | 范围、验收、候选或权限变化 | 替代 Run 或新 TaskRevision | Project、Task 身份 |

只有冻结策略列明的类型化技术故障，例如候选特有的认证/配额/网络故障、进程或运行时丢失、租约超时，才可以切换 Attempt。control 先隔离当前代次，再在候选、预算和剩余截止时间允许时于同一 Seat 创建新 Attempt；候选耗尽后，需要额外输入或授权则创建 Request，否则把 Seat/Obligation 收口为类型化技术失败，不能无限等待或伪装成语义驳回。单个 Seat 的 `accepted/rejected/changes_requested` 只是 reducer 输入；只有策略声明的否决权或汇总结果才触发返工，不能用负面票偷偷更换裁判。

Gate 是 Run 内由 WorkflowRevision 与 Run Manifest 冻结的治理节点/规则，不是独立模块。它的每个 Seat 绑定同一精确 ReviewSubjectRef、review-policy ref+digest、ContextManifest ref+digest、required Skill refs+digests 和 capability/permission-policy ref+digest，并各自冻结逻辑参与者。被评审 Revision 的作者或 subject producer 不得占用必需 reviewer Seat；必需 reviewer Seat 按 Gate 策略绑定彼此独立的逻辑 Participant。备用 Attempt 必须继承原 Seat 的参与者和全部评审依据，不能借更换 WorkerProfile 改变 Context、Skill、权限、票位或绕过分离。control/core 在计票时同时校验 producer、Participant、角色和权限；重复、越权、过期、身份冲突或 digest 不匹配的票不计数。同一 Seat 的备用 Attempt 不增加票。达到法定票数后，control 隔离未完成 Attempt，持久提交汇总 Verdict/Receipt，再完成 Engine task；剩余票数已不可能达到门槛时，Gate 产生类型化的 quorum-unreachable 结果，使 Obligation 失败并沿 WorkflowRevision 的失败边推进，不能无限等待。作者返工产生新 ChangeSetRevision/ArtifactRevision，旧必需 Verdict 因 subject digest 不匹配而失效并重新过 Gate；TaskRevision 只有在验收契约变化时才更新。

Run 终态只说明 Workflow 到达经 HCTL reducer 确认的终点，不直接改写 [Task](./task.md)。绑定精确 TaskRevision 的 Run 只有正常进入 `Completed` 后，control 才以稳定幂等键机械提交同一个 CompleteTaskIntent；Task 重新校验当前 Revision、来源 drift 和全部证据。Task 拒绝不回滚 Run。`Failed / Cancelled / Superseded` Run 只形成 Needs Attention/历史，不完成或取消 Task；Engine task 结束、进程退出或模型自述均不能触发该 handoff。

## Workflow 场景

Workflow 场景提供 Run Manifest 预览、只读图、节点/Seat/Attempt 渐进展开、Request、证据、暂停/取消和恢复状态。

| 角色 | 可以做什么 | 不能做什么 |
| --- | --- | --- |
| 场景客户端：Workbench Run 图 | 查询、预览并提交 Run 命令 | 直接修改 Engine 或签发结果 |
| 场景客户端：CLI | show/preview/start/pause/cancel；写操作先预览确认 | 绕过绑定、版本或权限 |
| 受控端口：WorkflowEngineAdapter / Conductor | 保存 token、机械任务、timer、retry 和历史 | 选择 Harness、创建 Seat、计算 HCTL Gate、签发 Receipt 或写 Git |

Workbench 关闭不停止 Run。Engine 管理界面只作诊断；发现越界修改时标记分歧并安全暂停。

## 模块交接

以下只列所有权方向；字段、事务与故障语义由[四模块连接合同](./connections.md)统一定义。

- Project 通过 StartRunIntent 交付冻结 Project/Workflow 引用；可选 TaskRevision 由 Task 提供，Run 独占 Manifest 与执行 lifecycle。
- Run 向 Harness 交付 AttemptSpec；Harness 只返回 ResultProposal、Revision、证据和物理观测。
- Run 向 Task 返回精确 Run/Verdict/Receipt 引用；正常完成的 task-bound Run 可由 reducer 提交同一个 CompleteTaskIntent。Run 向 Project 提交 Request 并投影低噪声里程碑。

## 不可破坏的边界

- Workflow Engine 拥有机械位置，control 拥有 Run 语义与所有 HCTL Engine mutation。
- Engine READY、进程退出和 Harness 自述都只是输入信号，不是语义结果。
- Attempt 的逻辑身份由 Run 拥有，物理进程由 Harness 模块观测；两边不维护两套状态。
- 候选切换、业务返工和 Engine retry 是三种不同路径。
- Verdict/Receipt 必须绑定精确 ReviewSubjectRef；current pointer 或文件路径不能替代版本。

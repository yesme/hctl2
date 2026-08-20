# Run 模块合同

> 状态：规范性合同 · 草案 v0.9.1<br>
> 本文是 Run 模块对象、状态机与写入者的唯一权威；设计正文见 [Run 与 Workflow](../run.md)，族规则与词汇分类见[合同层总则](./README.md)，模块交接见[连接合同](./connections.md)，共享机制见[系统边界](./system.md)。

## 对象

| 对象 | 含义 |
| --- | --- |
| WorkflowRevision | 与引擎无关、不可变的控制图、节点、输入输出和治理规则（Revision 族） |
| EngineDeployment | 某个 WorkflowRevision 经固定编译器和 WorkflowEngine 端口适配器生成的不可变引擎版本（Revision 族） |
| Run | 对冻结 Workflow、Task、基线、角色、候选、权限和预算的一次授权执行 |
| EngineExecutionBinding | Run、EngineDeployment、ResolvedPortBinding、外部 execution ID/correlation key 和 generation 的稳定映射（Binding 族） |
| Obligation | 一个 HCTL 外部节点必须产出的逻辑结果 |
| Seat | Obligation 中稳定的逻辑执行者或投票位置 |
| Attempt | 某个候选对 Seat 的一次执行。其派发冻结由 ExecutionSpec（定义见[连接合同](./connections.md)）承载；Attempt 侧特有字段只有 attempt/seat/run 身份与代次，Seat identity、截止时间等见共同字段 |
| ReviewSubjectRef | 对 ChangeSetRevision 或 ArtifactRevision 的精确 kind、ID 和 digest 引用（引用格式） |
| Verdict / Receipt | 对精确版本的语义裁决，以及 control/core 校验后的正式证明（Receipt 族） |

## 写入合同

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| WorkflowRevision / EngineDeployment | immutable revision + approval version | control 协调 Register、Compile、Approve Intent；固定 compiler/adapter 产出，core 校验摘要 | Revision 不改写；新内容创建新 Revision |
| Run / Manifest | `run_version`；`Starting / Running / Pausing / Paused / Cancelling / Completed / Failed / Cancelled / Superseded` | control 处理 StartRunIntent、Pause/Resume/Cancel/Replace Run Intent | Completed、Failed、Cancelled、Superseded 不复活；替代创建新 Run |
| EngineExecutionBinding | binding generation；`Starting / Bound / Closed / Diverged` | control 经 WorkflowEngine 端口适配器启动、回读、关闭或标记分歧 | 外部 execution ID 只作绑定，不成为 Run 身份 |
| Obligation / Seat | state version；`Active / Satisfied / Failed / Cancelled / Superseded` | Run reducer/control 根据 Engine task、Attempt 结果和 Gate 策略推进 | 终态不可复活；Engine retry 创建新 Obligation |
| Attempt | `attempt_generation` + state version；合法边见下文 | control 创建、取消、替代和准入结果；agentd 只返回观测 | 终态不可复活；候选切换创建同 Seat 的新 Attempt |
| Verdict / Receipt | immutable | 只有 Run reducer 与 control/core 校验事务可写 | 精确绑定 ReviewSubjectRef、规则和证据 |

Run 合法边固定为：`Starting → Running/Failed/Cancelled/Superseded`；`Running → Pausing/Cancelling/Completed/Failed/Superseded`；`Pausing → Paused/Cancelling/Failed/Superseded`；`Paused → Running/Cancelling/Failed/Superseded`；`Cancelling → Cancelled/Failed/Superseded`。每个过渡态都必须能被取消、失败或替代路径收口，不能因 Engine 失联永久阻塞绑定 Task。外部 ACK 不直接写状态，control 只依据匹配 binding/generation 的回读推进。

Workflow Node、Engine task execution、Obligation、Seat 和 Attempt 是不同身份。Obligation 的不可变绑定固定 EngineExecutionBinding generation 与精确 Engine task execution identity（外部 task ID 及 retry/attempt generation）；其带版本的租约视图记录生效租约、截止时间和最近一次由 adapter 回读确认的续租，超时与备用候选准入只能依据该确认值。Engine retry 创建新 Obligation 前，control 必须在同一领域事务中把旧 Obligation 及其未终态 Seat/Attempt 置为 `Superseded`，令其派发、写入与输入授权失效，并提交物理隔离 outbox；旧执行的心跳、投票和迟到结果此后只留审计。技术性候选切换只在同一 Seat 下创建新 Attempt，不增加票数或更换逻辑裁判。

Verdict、Gate Receipt 与凭证链是 Workflow 场景的结晶（“干成了的证明”）：权威在 metadata 账本，结晶副本按[双层保存政策](./system.md#事实与存储)写入 Git。

## Workflow 与 Run 授权

WorkflowRevision 使用 HCTL 规范化 JSON，经过数据结构、Profile 和语义校验后写入 Git。EngineDeployment 固定编译器、Profile、引擎适配器、绑定版本和引擎定义摘要。引擎产物不能反向定义 WorkflowRevision。

Approve Workflow 只确认施工图；`StartRunIntent` 才授予资源和副作用权。Run Manifest 至少冻结：

- Project、0..1 个 TaskRevision（第一阶段）、WorkflowRevision 与 EngineDeployment；
- repo/base revision、角色与逻辑 Seat；
- 获准 WorkerProfile 候选、切换规则、能力和权限；
- Gate、预算、放置和截止规则。

第一阶段，绑定 TaskRevision 的 Run 表示对该完整 Task 验收合同的一次施工授权，因此只有它正常 `Completed` 才具备提交 Task 完成命令的资格。只覆盖局部研究、咨询或中间步骤的自动化必须使用无 Task Run 或 RoomInvocation，并以稳定引用把结果交回 Task；不能绑定 Task 后再依靠 Prompt 声明“这次不算完整施工”。

运行中只有 Manifest 明确声明为可变的放置参数可以按冻结规则和边界调整；每次调整都校验预期 Run version，并留下固定前后值、适用规则、actor 和 Run version 的不可变审计事件。范围、验收、候选、权限、Gate 或超出获准边界的放置变化必须创建替代 Run，不能原地漂移。

第一阶段 Profile 允许外部执行、fork/join、switch、loop、dynamic fork、timer wait、noop 和经审计的纯数据转换；明确拒绝 `SUB_WORKFLOW`、Conductor `HUMAN` task 和绕过 control 的 HTTP/JDBC/Kafka/Git/Harness 副作用。dynamic fork 只能实例化 WorkflowRevision/Manifest 已冻结的有界 Seat 模板：候选 Participant/Role、最大基数、预算、选择函数和权限上限都必须预先固定；模型输出不能新增 recipient、扩大 fan-out 或扩权，无法机械校验时整次 fork 拒绝。

## 从节点到结果

本节定义 Run 内部归约；对 Agent 模块的派发、结果信封和故障恢复见[连接合同](./connections.md)。

1. control 领取一个 HCTL 外部 Engine task，并按 Run、binding generation 与精确 Engine task execution identity 幂等创建唯一 Obligation；Engine 的 join/switch/wait 等机械节点不创建 Obligation。
2. control 按规则创建 Seat，并为候选产生 ExecutionSpec。
3. [Agent](./agent.md) 模块执行 Attempt，只能返回 ResultProposal、Revision 和证据。
4. control/core 校验精确 binding、代次、权限、ReviewSubjectRef 和证据；通过后形成 Seat 结果、Verdict 或 Receipt。
5. 领域结果与 Engine completion outbox 先持久提交，再幂等完成外部任务。

Attempt 的派发在 ExecutionSpec 中至少冻结 attempt/seat/run/generation、逻辑 Participant/Seat identity、WorkerProfile、接入方式与降级能力、RuntimeBackend binding、可选 ChangeSet/WriteLease、Context/Skill/能力/权限摘要和截止时间。Attempt lifecycle 为 `Pending | Running | WaitingForInput | ResultProposed | Failed | Lost | Cancelled | Superseded`：`Pending` 可进入 `Running/Failed/Lost/Cancelled`；`Running` 与 `WaitingForInput` 可互转并进入 `ResultProposed/Failed/Lost/Cancelled`；任一尚未提交 Proposal 的非终态可进入 `Superseded`。`ResultProposed` 是“该 Attempt 已提交不可变 Proposal”的终态，不表示 Seat、Gate、Run 或 Task 成功；owner 对 Proposal 的准入或拒绝推进 Seat/Obligation，修正或重新施工创建新 Attempt/Proposal，而不复活旧 Attempt。状态只由 control 根据 agentd 观测推进，全部终态不可复活。

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

## Run → Task

Run 终态只说明 Workflow 到达经 HCTL reducer 确认的终点，不直接改写 [Task](./task.md)。绑定精确 TaskRevision 的 Run 只有正常进入 `Completed` 后，control 才以稳定幂等键机械提交同一个 CompleteTaskIntent；Task 重新校验当前 Revision、来源 drift 和全部证据。Task 拒绝不回滚 Run。`Failed / Cancelled / Superseded` Run 只形成 Needs Attention/历史，不完成或取消 Task；Engine task 结束、进程退出或模型自述均不能触发该 handoff。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL 词 | 外部体系词 | 差异 |
| --- | --- | --- |
| WorkflowRevision | Conductor 的 versioned workflow definition / BPMN 的 process definition | 引擎产物不能反向定义它；它先于任何引擎存在 |
| EngineDeployment | 引擎里注册的 definition 版本 | 额外固定编译器与 Profile，供分歧检测 |
| Run | workflow execution / process instance | Run 还冻结授权、候选、权限与预算，不只是一次实例 |
| Engine external task | Conductor 的 worker task（SIMPLE，poll/complete）/ Camunda 的 external task 模式 | HCTL 只经 control 领取与完成，场景客户端不得直接操作 |
| EngineExecutionBinding | execution id + correlation key | 只作绑定与恢复关联，不成为 Run 身份 |
| 引擎 retry / timer | 引擎原生能力 | 只产生机械位置变化，不产生 HCTL 语义结果 |
| Obligation / Seat / Verdict / Gate Receipt | 无对应 | 差异化核心：对 worker/external task，引擎只有队列与机械位置；BPMN/Camunda 的 user task 虽有候选人概念，但 HCTL 明确拒绝 HUMAN 任务，“谁有资格、结果是否有效、凭什么算完成”只在 HCTL 侧 |

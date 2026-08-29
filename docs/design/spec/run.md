# Run 模块合同

> 状态：规范性合同 · 草案 v0.15.0<br>
> 本文是 Run 模块对象、状态机与写入者的唯一权威；设计正文见 [Run 与 Workflow](../run.md)，族规则与词汇分类见[合同层总则](./README.md)，模块交接见[连接合同](./connections.md)，共享机制见[系统边界](./system.md)。

## 对象

| 对象 | 含义 |
| --- | --- |
| Workflow Revision | 与引擎无关、不可变的控制图、节点、输入输出和治理规则（Revision 族） |
| Engine Deployment | 某个 Workflow Revision 经固定编译器和 workflow engine 端口适配器生成的不可变引擎版本（Revision 族） |
| Run | 对冻结 Workflow、Task、基线、角色、候选、权限和预算的一次授权执行 |
| Engine Execution Binding | Run、Engine Deployment、Resolved Port Binding、外部 execution ID/correlation key 和 `engine_binding_generation` 的稳定映射（Binding 族） |
| Obligation | 一个 HCTL 外部节点必须产出的逻辑结果 |
| Seat | Obligation 中稳定的逻辑执行者或投票位置 |
| Attempt | 某个候选对 Seat 的一次执行。其派发冻结由 Execution Spec（定义见[连接合同](./connections.md)）承载；Attempt 侧特有字段只有 attempt/seat/run 身份与代次，Seat identity、截止时间等见共同字段 |
| ReviewSubjectRef | 对 ChangeSet Revision 或 Artifact Revision 的精确 kind、ID 和 digest 引用（引用格式） |
| Verdict / Receipt | 对精确版本的语义裁决，以及 control 与工具箱校验后的正式证明（Receipt 族） |

## 写入合同

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| Workflow Revision / Engine Deployment | immutable revision + approval version | control 协调「登记/编译/批准」命令；固定 compiler/adapter 产出，工具箱校验摘要 | Revision 不改写；新内容创建新 Revision |
| Run / Manifest | `run_version`；启动中 / 运行中 / 暂停中 / 已暂停 / 取消中 / 完成 / 失败 / 已取消 / 被替代 | control 处理「启动/暂停/恢复/取消/替代 Run」命令 | 完成、失败、已取消、被替代不复活；替代创建新 Run |
| Engine Execution Binding | `engine_binding_generation`；启动中 / 已绑定 / 已关闭 / 分歧 | control 经 workflow engine 端口适配器启动、回读、关闭或标记分歧 | 外部 execution ID 只作绑定，不成为 Run 身份 |
| Obligation / Seat | state version；活跃 / 已达成 / 失败 / 已取消 / 被替代 | control 在观察到 Engine 检查点进入等待态时铸造，此后按账本内的 Attempt 结果与 Gate 策略推进 | 终态不可复活；Engine 重试只是路标再次进入等待态，由 control 按新观察序号铸造新 Obligation |
| Attempt | `attempt_generation` + state version；合法边见下文 | control 创建、取消、替代和准入结果；Agency 只返回观测 | 终态不可复活；候选切换创建同 Seat 的新 Attempt |
| Verdict / Receipt | immutable | 只有 Run reducer 与 control/工具箱校验事务可写 | 精确绑定 ReviewSubjectRef、规则和证据 |

Run 合法边固定为：启动中 → 运行中/失败/已取消/被替代；运行中 → 暂停中/取消中/完成/失败/被替代；暂停中 → 已暂停/取消中/失败/被替代；已暂停 → 运行中/取消中/失败/被替代；取消中 → 已取消/失败/被替代。每个过渡态都必须能通过取消、失败或替代进入终态，不能因 Engine 失联永久阻塞绑定 Task。外部 ACK 与 Engine 回读都不直接写状态：Run 状态只由 control 按账本事实推进，Engine 位置只是路标，用于关联与分歧检测。

Run 是反应式状态机，但所有输入不共用一条无类型事件通道。human 的 Start/Pause/Resume/Cancel/Request answer 先进入公共 command service；Agent 的结果先进入 Result Proposal；timer 与 Engine 位置先成为带版本观测。control 归约并持久化领域结果、撤权与 outbox 后，adapter 才推动 Dagu。这个先后次序是 Run 正确性的一部分，Workbench、CLI 或 provider UI 都不能绕过。

Dagu 原生 UI/API 的 Start/Stop/Retry/Reschedule/Approve/Reject/Edit/Rename/Delete 会直接改变定义或机械执行，不能在副作用前携带 HCTL command envelope、Run expected version、Attempt/lease 撤权和同一账本事务。因此这些动作不是可接纳的 provider-origin human 命令请求：对未绑定的 Dagu 对象可作实例管理，对已绑定 Engine Deployment/Execution 的任何直接 mutation 只追加回读观测并把 Engine Execution Binding 标为分歧；control 不根据事后相似状态补造 Run 命令、Verdict 或 Receipt。将来只有 provider 支持 mutation 前拦截，并能让 control 完成同一 Preview/Submit/outbox 顺序后再执行，才可在新 binding revision 中声明相应能力。

`运行中 → 完成` 不是通用写入口，只能由确定性 reducer 在同一预览版本上证明以下正常完成谓词后执行：冻结 Workflow Revision 的全部 required Obligation、Seat、Gate 与声明输出均已在账本中以精确 subject 和 Evidence/Verdict/Receipt 达成；所有 Attempt 已终态，或已先撤销其 runtime、输入/写租约并标为被替代/已取消；不存在仍会影响 required output 的待处理/结果未知外部副作用；Run/Manifest、Engine Execution Binding 的账本记录和全部结果引用仍匹配当前账本版本。任一项未知都只能保持运行/暂停/需要关注或走类型化失败、取消、替代，Engine 检查点结束、进程退出、Harness/LLM 自述和单个 Proposal 都不能补足谓词；Engine 路标此时应停在 success terminal，路标不可读或与账本不一致只把 Engine Execution Binding 标为分歧待对账，既不补足也不阻止谓词。

任何失败、取消或替代终态在释放 Task Run claim 前，也必须在同一事务撤销旧 dispatch、输入/写租约与外部副作用资格，并提交 runtime stop/fence；若只能撤销逻辑权威而无法证明旧进程已静默，则隔离旧 worktree/ChangeSet，后续执行按 Agent 合同使用新 worktree **和**新 ChangeSet。不能证明旧执行被限制在该隔离边界内时，Run 保持取消中/需要关注且 claim 不释放，不能以“失败了”为由并发启动第二个 writer。

Workflow Node、Engine 检查点、Obligation、Seat 和 Attempt 是不同身份。Obligation 由 control 按 Run、节点与该节点的观察序号铸造，身份、deadline 与租约都是账本事实；Engine 侧的 DAG run ID 与不可变 step name 只作关联键记入 Engine Execution Binding，代次不在 Engine。超时与备用候选准入只依据账本自己的 Obligation deadline。Engine 重试只是路标再次进入等待态：control 先在同一领域事务把旧 Obligation 及其未终态 Seat/Attempt 置为被替代，令其派发、写入与输入授权失效并提交物理隔离 outbox，再按新的观察序号创建新 Obligation；旧执行的心跳、投票和迟到结果此后只留审计。技术性候选切换只在同一 Seat 下创建新 Attempt，不增加票数或更换逻辑裁判。

Verdict、Gate Receipt 与凭证链是 Workflow 场景的结晶（“干成了的证明”）：权威在 metadata 账本，结晶副本按[系统存储合同](./system.md#git-的双重角色)写入 Git。

## Workflow 与 Run 授权

Workflow Revision 使用 HCTL 规范化 JSON，经过数据结构、Profile 和语义校验后由工具箱写入/回读 Git；Git 保存不可变正文，control 账本独占 identity、admission、digest、approval/current pointer。Engine Deployment 固定编译器、Profile、引擎适配器、绑定版本和引擎定义摘要。引擎产物不能反向定义 Workflow Revision。

Approve Workflow 只确认施工图；「启动 Run」命令才授予资源和副作用权。Run Manifest 至少冻结：

- Project、0..1 个 Task Revision（第一阶段）、Workflow Revision 与 Engine Deployment；
- repo/base revision、根 Context Manifest ref+digest、角色与逻辑 Seat；
- 每个角色/Seat 的精确 Participant revision、Project Role Binding version/digest、required/optional Skill refs+digests；
- 获准 Worker Profile 候选、切换规则、能力和权限；
- Gate、预算、放置和截止规则。

第一阶段，绑定 Task Revision 的 Run 表示对该完整 Task 验收合同的一次施工授权，因此只有它正常完成才具备提交 Task 完成命令的资格。只覆盖局部研究、咨询或中间步骤的自动化必须使用无 Task Run 或 Room Invocation，并以稳定引用把结果交回 Task；不能绑定 Task 后再依靠 Prompt 声明“这次不算完整施工”。

### 启动与 Manifest

「启动 Run」必须 CAS 活跃 Project/version、可选 Task 的开放 lifecycle/current Revision 及该 Task 的 Run claim，并在同一用户级账本事务创建 Run、不可变 Manifest、幂等结果、`active` Task claim（有 Task 时）和 Engine start outbox。每个 Task 至多一个 `active | completion_pending` claim；相同 idempotency key 的第二次 Start 返回原 Run，其他 Start 返回 typed conflict，不能只靠 Engine correlation key 去重。Project 已归档、Task 无契约、Project 不匹配或已有 claim 都拒绝。

「替代 Run」不是先取消再另起：同一事务校验旧 Run/version，撤销旧 runtime、输入/写租约与 owner-specific fence，把旧 Run/Obligation/Seat/Attempt 置为被替代并提交 stop/fence outbox，同时创建新 Run/Manifest、把唯一 Task claim 从旧 ref 转到新 ref；不得为了替代一个 Run 任意推进共享 site generation 而误伤其他执行。新执行必须使用新的 Execution Spec/runtime generation；旧写入未能物理证明静默时还必须按 [Agent 合同](./agent.md#changeset-与-git-事实)使用新 ChangeSet 与新 worktree。事务任一步失败就不转移 claim。

运行中只有 Manifest 明确声明为可变的放置参数可以按冻结规则和边界调整；每次调整都校验预期 Run version，并留下固定前后值、适用规则、actor 和 Run version 的不可变审计事件。范围、验收、候选、权限、Gate 或超出获准边界的放置变化必须创建替代 Run，不能原地漂移。

第一阶段 HCTL Profile 允许外部执行、fork/join、switch、loop、dynamic fork、timer wait、noop 和经审计的纯数据转换；先以 schema、引用、Profile 和图结构 lint 拒绝格式或结构不合法的 Workflow Revision，再由固定编译器生成并验证 Dagu YAML。生成物只允许依赖、条件、等待等机械结构，以及无进程的 Dagu `human.task` 作为 HCTL 外部执行检查点；明确拒绝子 DAG、默认 command/script、action/HTTP/agent/Harness 和其他绕过 control 的副作用。这里的 `human.task` 只是 adapter 使用的被动检查点，不表示人类 Request 或人工裁决。dynamic fork 只能实例化 Workflow Revision/Manifest 已冻结的有界 Seat 模板：候选 Participant/Role、最大基数、预算、选择函数和权限上限都必须预先固定；模型输出不能新增 recipient、扩大 fan-out 或扩权，无法机械校验时整次 fork 拒绝。lint 不承诺证明任意 loop 终止；重复进入同一节点按观察序号产生新的 Obligation，不依赖 Engine 提供可隔离的检查点身份。

## 从节点到结果

本节定义 Run 内部归约；对 Agent 模块的派发、结果信封和故障恢复见[连接合同](./connections.md)。

1. control 观察到 Engine 路标在某个 HCTL 外部节点进入等待态，按 Run、节点与观察序号幂等创建唯一 Obligation；Dagu 的 dependency/condition/wait 等机械节点不创建 Obligation。
2. control 按规则创建 Seat，并为候选产生 Execution Spec。
3. [Agent](./agent.md) 模块执行 Attempt，只能返回 Result Proposal、Revision 和证据。
4. control 与工具箱校验精确 binding、代次、权限、ReviewSubjectRef 和证据；通过后形成 Seat 结果、Verdict 或 Receipt。
5. 领域结果与 Engine completion outbox 先持久提交，再经 Dagu `human.task` API 把路标推过该检查点；ACK 未知时先回读再重投。路标与账本不一致（检查点已被 Engine 自行推进、从 UI 完成或重试）只把 Engine Execution Binding 标为分歧并对账，不改写任何 HCTL 结果。

Attempt 的派发在 Execution Spec 中至少冻结 attempt/seat/run ref + `attempt_generation`、精确 Participant revision/Project Role Binding、Worker Profile、接入方式与降级能力、Agency binding、可选 ChangeSet/Write Lease、根 Context Manifest ref+digest、该 Attempt 的 Context Bundle ref+digest、实际 Skill refs+digests、能力/权限和截止时间。`attempt_generation` 是语义 owner 代次；它既不是后续激活时分配的 `runtime_generation`，也不是 control/site/backend 的基础设施 fence generation，所有层都必须分别携带并逐项校验。Attempt lifecycle 为待启动 | 运行中 | 等待输入 | 已交提案 | 失败 | 丢失 | 已取消 | 被替代：待启动可进入运行中/失败/丢失/已取消；运行中与等待输入可互转并进入已交提案/失败/丢失/已取消；任一尚未提交 Proposal 的非终态可进入被替代。已交提案是“该 Attempt 已提交不可变 Proposal”的终态，不表示 Seat、Gate、Run 或 Task 成功；owner 对 Proposal 的准入或拒绝推进 Seat/Obligation，修正或重新施工创建新 Attempt/Proposal，而不复活旧 Attempt。状态只由 control 根据 Agency 观测与网关第一方观测推进，全部终态不可复活。

Attempt 的 Context Bundle 按 [Project 合同](./project.md#context-memo-artifact)的投喂档装入同 Run 前序节点的结果：Gate Seat 的 ReviewSubjectRef 所指 Revision 与返工 Seat 所依据的 Verdict 正文是必用条目——预算内 inline，超预算改 pointer 并附分片建议；Verdict 以账本记录物化，其 Git 结晶副本只作 pointer。Engine 路标、step 日志与终端 trace 不是来源。同一 Seat 的备用 Attempt 复用同一 Bundle 并附旧 ChangeSet Revision 的 pointer；未形成 ChangeSet Revision 的工作树内容不传承。

## Request、重试与 Gate

Run 需要输入时向 Project 提交类型化 [Request](./project.md) 创建命令，只阻塞声明的范围；Project 独占 Request lifecycle。Request 冻结 deadline 与 `fail | cancel` 默认策略；Resolve/Expire 的跨模块事务都 CAS 精确 Request 与 blocker version，只有 Resolve 可以写答案 delivery，Expire 不能猜测答案而是按冻结策略结束对应 Attempt/Seat/Obligation。Run 只在匹配 ACK/观测后恢复绑定执行；节点仍通过正常 Result Proposal/Receipt 路径完成。Dagu `human.task` 只是每个 HCTL 外部节点的机械暂停原语，不构成第二条人类输入或完成路径。

“重试”不是一个概念，而是五种身份不同的路径；混用它们会复制票数、绕过验收或复活旧执行：

| 路径 | 触发 | 产生的新身份 | 不变的身份 |
| --- | --- | --- | --- |
| 传输重投 | 投递超时、ACK 丢失 | 无：同一幂等键重投，重复命令返回原结果 | 一切领域对象 |
| 候选切换 | 类型化技术故障 | 同一 Seat 下的新 Attempt | Obligation、Seat、票位 |
| Engine retry | 路标再次进入等待态 | control 按新观察序号铸造的新 Obligation（旧 Obligation 及其 Seat/Attempt 置为被替代） | Run |
| 语义返工 | changes_requested 汇总 | 新 ChangeSet Revision/Artifact Revision，旧票失效并完整 regate | Run、Task Revision（通常） |
| 替代执行 | 范围、验收、候选或权限变化 | 替代 Run 或新 Task Revision | Project、Task 身份 |

只有冻结策略列明的类型化技术故障，例如候选特有的认证/配额/网络故障、进程或运行时丢失、租约超时，才可以切换 Attempt。control 先隔离当前代次，再在候选、预算和剩余截止时间允许时于同一 Seat 创建新 Attempt；候选耗尽后，需要额外输入或授权则创建 Request，否则把 Seat/Obligation 标为类型化技术失败，不能无限等待或伪装成语义驳回。单个 Seat 的 `accepted/rejected/changes_requested` 只是 reducer 输入；只有策略声明的否决权或汇总结果才触发返工，不能用负面票偷偷更换裁判。

Gate 是 Run 内由 Workflow Revision 与 Run Manifest 冻结的治理节点/规则，不是独立模块。它的每个 Seat 绑定同一精确 ReviewSubjectRef、review-policy ref+digest、根 Context Manifest ref+digest、required Skill refs+digests 和 capability/permission-policy ref+digest，并各自冻结精确 Participant revision 与 Project Role Binding。被评审 Revision 的作者或 subject producer 不得占用必需 reviewer Seat；必需 reviewer Seat 绑定互不相同的 Participant revision，备用 Attempt 必须继承原 Seat 的逻辑身份和全部评审依据，不能借更换 Worker Profile 改变 Context、Skill、权限、票位或绕过分离。control 与工具箱在计票时同时校验 producer、Participant、角色和权限；重复、越权、过期、身份冲突或 digest 不匹配的票不计数。同一 Seat 的备用 Attempt 不增加票。

第一阶段这只证明**逻辑 Participant 分离与 producer/reviewer 分离**，不证明背后是不同人类 operator、公司、模型提供方、基础模型或 post-train。Manifest/Gate policy 应冻结 provider/model/operator refs 中受控端口实际认证的部分并在预览、Verdict 和审计中展示 `known / unknown`；Participant、Harness 或模型自报不能把 unknown 变成 known。策略若要求上述物理或组织独立，而当前端口不能机械认证，就必须把该 Gate 判为 unsupported，不能用不同 Participant 名称冒充独立。达到法定票数后，control 隔离未完成 Attempt，持久提交汇总 Verdict/Receipt，再完成 Engine 检查点；剩余票数已不可能达到门槛时，Gate 产生类型化的 quorum-unreachable 结果，使 Obligation 失败并沿 Workflow Revision 的失败边推进，不能无限等待。作者返工产生新 ChangeSet Revision/Artifact Revision，旧必需 Verdict 因 subject digest 不匹配而失效并重新过 Gate；Task Revision 只有在验收契约变化时才更新。

## Run → Task

Run 终态只说明 Workflow 到达经 HCTL reducer 确认的终点，不直接改写 [Task](./task.md)。绑定精确 Task Revision 的 Run 只有满足上述正常完成谓词后，完成事务才把该 Task 的 claim 从 `active` CAS 为 `completion_pending(run_ref)`，并以 Run/Task 派生的稳定幂等键写内部「完成 Task」command outbox；pending 阻止新 Run 插入。随后 Task 按同一个用户级账本中的当前 Revision、来源 freshness/drift 和逐项证据独立校验，并在成功 Receipt 或持久拒绝结果的事务清除该 claim。Task 拒绝不回滚 Run。失败 / 已取消 / 被替代 Run 只有满足上一段隔离前置，其终态事务才释放旧 active claim；它们只形成需要关注/历史，不提交完成或取消 Task。Engine 检查点结束、进程退出或模型自述均不能触发该 handoff。完成谓词只依据账本事实与外部证据的 fresh readback；Engine 路标不可读只把 Engine Execution Binding 标为分歧，不阻止 Run 完成。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL 词 | 外部体系词 | 差异 |
| --- | --- | --- |
| Workflow Revision | Dagu 的 YAML DAG definition / BPMN 的 process definition | 引擎产物不能反向定义它；它先于任何引擎存在 |
| Engine Deployment | 引擎里注册的 definition 版本 | 额外固定编译器与 Profile，供分歧检测 |
| Run | workflow execution / process instance | Run 还冻结授权、候选、权限与预算，不只是一次实例 |
| Engine 外部检查点 | Dagu 的 processless `human.task` / Camunda 的 external task 模式 | Dagu 名称虽含 human，在 HCTL 中只是路标：control 观察其等待态铸 Obligation，账本结果落定后再推过它；场景客户端不得直接操作 |
| Engine Execution Binding | execution id + correlation key | 只作绑定与恢复关联，不成为 Run 身份 |
| 引擎 retry / timer | 引擎原生能力 | 只产生机械位置变化，不产生 HCTL 语义结果 |
| Obligation / Seat / Verdict / Gate Receipt | 无对应 | 差异化核心：Dagu `human.task` 只有等待、参数与机械位置，不提供 HCTL 的身份、候选、法定票数或 Receipt；“谁有资格、结果是否有效、凭什么算完成”只在 HCTL 侧 |

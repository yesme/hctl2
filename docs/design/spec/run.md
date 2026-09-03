# Run 模块约束

> 状态：规范性约束 · 草案 v0.16.5<br>
> 本文是 Run 模块对象、状态机与写入者的唯一权威；设计正文见 [Run 与 Workflow](../run.md)，族规则与词汇分类见[约束层总则](./README.md)，模块交接见[连接约束](./connections.md)，共享机制见[系统边界](./system.md)。

## 对象

| 对象 | 含义 |
| --- | --- |
| Workflow Revision | 与引擎无关、不可变的控制图、节点、输入输出和治理规则（Revision 族） |
| Engine Deployment | 某个 Workflow Revision 经固定编译器和 workflow engine 端口适配器生成的不可变引擎版本（Revision 族） |
| Run | 对冻结 Workflow、Task、基线、角色、候选、权限和预算的一次授权执行 |
| Run–Engine Binding | Run、Engine Deployment、Port–Provider Binding、外部 execution ID/correlation key 和 `engine_binding_generation` 的稳定映射（Binding 族） |
| Obligation | 一个 HCTL 外部节点必须产出的逻辑结果 |
| Seat | Obligation 中稳定的逻辑执行者或投票位置 |
| Attempt | 某个候选对一个 Seat 的一次执行。派发所需的完整冻结记录由 Execution Spec 承载；Attempt 自身只增加 attempt、seat、run 引用和 `attempt_generation`。共同字段见[连接约束](./connections.md) |
| ReviewSubjectRef | 对 ChangeSet Revision 或 Artifact Revision 的精确 kind、ID 和 digest 引用（引用格式） |
| Verdict / Receipt | 对精确版本的语义裁决，以及 control 与工具箱校验后的正式证明（Receipt 族） |

## 写入约束

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| Workflow Revision / Engine Deployment | immutable revision + approval version | control 协调「登记/编译/批准」命令；固定 compiler/adapter 产出，工具箱校验摘要 | Revision 不改写；新内容创建新 Revision |
| Run / Manifest | `run_version`；启动中 / 运行中 / 暂停中 / 已暂停 / 取消中 / 完成 / 失败 / 已取消 / 被替代 | control 处理「启动/暂停/恢复/取消/替代 Run」命令 | 完成、失败、已取消、被替代不复活；替代创建新 Run |
| Run–Engine Binding | `engine_binding_generation`；启动中 / 已绑定 / 已关闭 / 分歧 | control 经 workflow engine 端口适配器启动、回读、关闭或标记分歧 | 外部 execution ID 只作绑定，不成为 Run 身份 |
| Obligation / Seat | state version；活跃 / 已达成 / 失败 / 已取消 / 被替代 | control 在观察到 Engine 检查点进入等待态时创建（只认当前观察，见“从节点到结果”），此后按账本内的 Attempt 结果与 Gate 策略推进 | 终态不可复活；引擎重试使检查点再次进入等待态，由 control 按新观察序号创建新 Obligation |
| Attempt | `attempt_generation` + state version；合法边见下文 | control 创建、取消、替代和准入结果；Agency 只返回观测 | 终态不可复活；候选切换创建同 Seat 的新 Attempt |
| Verdict / Receipt | immutable | 只有 Run reducer 与 control/工具箱校验事务可写 | 精确绑定 ReviewSubjectRef、规则和证据 |

Run 状态只由 control 根据获准命令和账本事实推进，workflow engine 回读只提供观测。下表列出全部合法转移；未列出的状态转换必须返回类型化拒绝。

| 当前状态 | 触发命令或条件 | 新状态 | 其他输入的处理 |
| --- | --- | --- | --- |
| 启动中 | 启动完成 / 失败 / 取消 / 替代 | 运行中 / 失败 / 已取消 / 被替代 | 类型化拒绝 |
| 运行中 | 暂停 / 取消 / 完成谓词成立 / 失败 / 替代 | 暂停中 / 取消中 / 完成 / 失败 / 被替代 | 类型化拒绝 |
| 暂停中 | 暂停完成 / 取消 / 失败 / 替代 | 已暂停 / 取消中 / 失败 / 被替代 | 类型化拒绝 |
| 已暂停 | 恢复 / 取消 / 失败 / 替代 | 运行中 / 取消中 / 失败 / 被替代 | 类型化拒绝 |
| 取消中 | 取消完成 / 失败 / 替代 | 已取消 / 失败 / 被替代 | 类型化拒绝 |

启动中、暂停中和取消中都必须能通过取消、失败或替代进入终态，不能因 workflow engine 失联永久阻塞绑定 Task。三个过渡态默认带墙钟超时，Run Manifest 可声明更长或更短；到期时 control 不做自动决定，只把 Run 标为需要关注并保留当前状态，等待人取消或替代。默认值见[交付文档](../delivery.md#运行默认值)。外部确认回执与引擎回读都不直接写状态；引擎报告的进度只用于关联与分歧检测。

Run 是反应式状态机，但所有输入不共用一条无类型事件通道。人的启动、暂停、恢复、取消和 Request 回答先进入公共命令服务。执行体的结果先进入 Result Proposal，定时器与引擎报告的进度先成为带版本观测。control 持久化归约结果、撤权和 outbox 后，适配器才推动 Dagu。这个先后次序是 Run 正确性的一部分，Workbench、CLI 或供应端界面都不能绕过。

Dagu 原生 UI/API 对已绑定 Engine Deployment/Execution 的直接修改只追加回读观测，并把 Run–Engine Binding 标为分歧。只有供应端支持修改前拦截，并能让 control 完成同一预览、提交和 outbox 顺序后再执行，新的绑定版本才可以声明相应 human 命令能力。

Run 进入完成前，control 必须逐项证明：

1. 所有必需 Obligation、Seat、Gate 和输出已达成；
2. 所有 Attempt 已终态或已撤权；
3. 没有影响必需输出的未决副作用；
4. Manifest、Run–Engine Binding 和结果引用仍匹配。

任何一项未知都不得完成 Run。Engine 检查点、进程退出、Harness 或模型自述和单个 Result Proposal 都不能补足上述谓词。引擎报告的进度只用于分歧检测；该进度不可读或与账本不一致时，control 只把 Run–Engine Binding 标为分歧待对账，既不补足也不否定上述谓词。

任何失败、取消或替代终态在释放 Task Run 占用标记前，都必须在同一事务中撤销旧派发、输入与写租约和外部副作用资格，并提交运行时停止与隔离 outbox。若只能撤销逻辑权威而无法证明旧进程已静默，则隔离旧 Git 工作树和 ChangeSet；后续执行按 Participant 约束使用新的 Git 工作树和新的 ChangeSet。

不能证明旧执行被限制在该隔离边界内时，Run 保持取消中或需要关注，并保留占用标记。系统不能以“失败了”为由并发启动第二个写入者。

Workflow Node 是施工图中的节点，Engine 检查点是供应端报告的具体进度点。control 每次观察到检查点重新进入等待态，都按新的观察序号创建 Obligation。Obligation 包含逻辑 Seat，Seat 的每次物理尝试是 Attempt。

Obligation 的身份、截止时间与租约都是账本事实；引擎侧的 DAG run ID 与不可变步骤名只作 Run–Engine Binding 的关联键，代次不在引擎。超时与备用候选准入只依据账本自己的 Obligation 截止时间。引擎重试替代旧 Obligation；control 先使旧 Obligation 及其未终态 Seat/Attempt 的派发、写入与输入授权失效并提交物理隔离 outbox，再按新的观察序号创建新 Obligation。候选切换只替代同一 Seat 下的 Attempt，不增加票数或更换逻辑裁判。

Verdict、Gate Receipt 与凭证链是 Workflow 场景的结晶（“干成了的证明”）：权威在 metadata 账本，结晶副本按[系统存储约束](./system.md#git-的双重角色)写入 Git。

## Workflow 与 Run 授权

Workflow Revision 使用 HCTL 规范化 JSON，经过数据结构、Profile 和语义校验后由工具箱写入并回读 Git。Git 保存不可变正文；control 账本独占身份、准入、摘要和批准/current pointer。Engine Deployment 固定编译器、Profile、引擎适配器、绑定版本和引擎定义摘要。引擎产物不能反向定义 Workflow Revision。

Approve Workflow 只确认施工图；「启动 Run」命令才授予资源和副作用权。Run Manifest 至少冻结：

- Project、0..1 个 Task Revision、Workflow Revision 与 Engine Deployment；
- repo/base revision、根 Context Manifest ref+digest、逻辑 Seat 与各席位职责；
- 每个 Seat 的精确 Participant revision、Project version 与参与者授权条目 digest、required/optional Skill refs+digests；
- 受控端口绑定、获准 Worker Profile 候选、切换规则、能力、权限与网络/secret 范围；
- Gate（法定票数、返工轮数上限、是否允许增量评审）、预算、放置、过渡态超时和截止规则。

绑定 Task Revision 的 Run 表示对该完整 Task 验收约束的一次施工授权，因此只有它正常完成才具备提交 Task 完成命令的资格。只覆盖局部研究、咨询或中间步骤的自动化必须使用无 Task Run 或 Room Invocation，并以稳定引用把结果交回 Task；不能绑定 Task 后再依靠 Prompt 声明“这次不算完整施工”。

### 启动与 Manifest

“启动 Run”必须以比较并交换校验活跃 Project/version、可选 Task 的开放生命周期/current Revision，以及该 Task 的 Run 占用标记。在同一用户级账本事务中，control 创建 Run、不可变 Manifest、幂等结果、`active` Task 占用标记和引擎启动 outbox。

每个 Task 至多一个 `active | completion_pending` 占用标记；相同幂等键的第二次启动返回原 Run，其他启动返回类型化冲突，不能只靠引擎关联键去重。Project 已归档、Task 无契约、Project 不匹配或已有占用标记时，命令必须拒绝。

“替代 Run”不是先取消再另起。同一事务校验旧 Run/version，撤销旧运行时、输入与写租约和归属者专用的代次栅栏，把旧 Run/Obligation/Seat/Attempt 置为被替代并提交停止与隔离 outbox。同时，事务创建新 Run/Manifest，并把唯一 Task 占用标记从旧引用转到新引用。系统不得为了替代一个 Run 任意推进共享 site generation，进而误伤其他执行。

新执行必须使用新的 Execution Spec 和运行时代次。旧写入未能在物理上证明静默时，还必须按 [Participant 约束](./participant.md#changeset-与-git-事实)使用新的 ChangeSet 和 Git 工作树。事务任一步失败时都不得转移占用标记。

运行中只有 Manifest 明确声明为可变的放置参数可以按冻结规则和边界调整；每次调整都校验预期 Run version，并留下固定前后值、适用规则、actor 和 Run version 的不可变审计事件。范围、验收、候选、权限、Gate 或超出获准边界的放置变化必须创建替代 Run，不能原地漂移。

HCTL Profile 的规则分三组：

1. 允许的图结构：外部执行、fork/join、switch、loop、dynamic fork、timer wait、noop 和纯数据转换；节点可附外部机械事实前置声明（见「从节点到结果」）。
2. 编译器拒绝的副作用：子 DAG、默认 command/script、HTTP/action/agent/Harness；Dagu `human.task` 仅作被动检查点。
3. dynamic fork 只能实例化 Manifest 中已冻结的有界 Seat 模板；loop 每次重新进入节点都创建新 Obligation。

编译前先以 schema、引用、Profile 和图结构 lint 拒绝格式或结构不合法的 Workflow Revision，再由固定编译器生成并验证 Dagu YAML。dynamic fork 的候选 Participant/Role、最大基数、预算、选择函数和权限上限都必须预先固定；模型输出不能新增接收者、扩大扇出或扩权，无法机械校验时整次 fork 必须拒绝。lint 不承诺证明任意 loop 终止，也不依赖引擎提供可隔离的检查点身份。

## 从节点到结果

本节定义 Run 内部归约；对 Participant 模块的派发、结果信封和故障恢复见[连接约束](./connections.md)。

1. control 观察到 Engine 检查点在某个 HCTL 外部节点进入等待态，按 Run、节点与观察序号幂等创建唯一 Obligation；Dagu 的依赖、条件和等待等机械节点不创建 Obligation。control 只依据当前有效 Run–Engine Binding 的当前观察创建义务。绑定分歧待对账、引擎停报进度，或观察来自缓存、迟到事件或旧游标时，control 不创建新 Obligation；已经创建的义务照常验收与判决。
2. control 按规则创建 Seat，并为候选产生 Execution Spec。若 Workflow Revision 为该节点声明了外部机械事实前置（某提交的 CI 状态、PR 是否合并、引用是否推进、路径存在且摘要匹配），control 先经工具箱回读该事实：成立才派发；不成立按冻结策略等待或沿失败边推进；读不到则不派发并标需要关注。事实只能来自工具箱回读，执行体、适配器转述或模型自述都不能满足前置。前置是准入条件：不占席位、不投票、不创建 Obligation。
3. [Participant](./participant.md) 模块执行 Attempt，只能返回 Result Proposal、Revision 和证据。
4. control 与工具箱校验精确绑定、代次、权限、ReviewSubjectRef 和证据；通过后形成 Seat 结果、Verdict 或 Receipt。
5. 领域结果与引擎完成 outbox 先持久提交，再经 Dagu `human.task` API 推进该检查点；确认回执未知时先回读再重投。引擎报告的进度与账本不一致时——例如检查点已被引擎自行推进、从界面完成或重试——control 只把 Run–Engine Binding 标为分歧并对账，不改写任何 HCTL 结果。

Execution Spec 必须固定 Attempt、Seat、Run、Participant、Project 参与者授权条目、Worker Profile、Participant–Agency Binding、Context、Skill、权限、预算和可选 ChangeSet 的精确引用。`attempt_generation` 标识语义执行，`runtime_generation` 标识物理执行，control/site/Agency 代次排除旧基础设施动作；三组代次必须分别校验。

Attempt 的状态与合法转移如下。未列出的状态转换必须返回类型化拒绝。

| 当前状态 | 合法新状态 |
| --- | --- |
| 待启动 | 运行中 / 失败 / 丢失 / 已取消 / 被替代 |
| 运行中 | 等待输入 / 已交提案 / 失败 / 丢失 / 已取消 / 被替代 |
| 等待输入 | 运行中 / 已交提案 / 失败 / 丢失 / 已取消 / 被替代 |

“已交提案”只表示 Proposal 已冻结，不表示 Seat、Gate、Run 或 Task 成功。归属者对 Proposal 的准入或拒绝推进 Seat/Obligation；修正或重新施工必须创建新的 Attempt 和 Proposal，不能复活旧 Attempt。状态只由 control 根据 Agency 观测与网关第一方观测推进，全部终态不可复活。

Attempt 的 Context Bundle 按 [Project 约束](./project.md#context-memo-artifact)的交付方式装入同 Run 前序节点的结果。Gate Seat 的 ReviewSubjectRef 所指 Revision 与返工 Seat 所依据的 Verdict 正文是必用条目：预算内使用 `inline`，超出预算时改为 `pointer` 并附分片建议。Verdict 以账本记录物化，其 Git 结晶副本只作 `pointer`。

引擎报告的进度、步骤日志与终端跟踪记录不是来源。同一 Seat 的备用 Attempt 复用同一 Bundle，并附旧 ChangeSet Revision 的 `pointer`；未形成 ChangeSet Revision 的 Git 工作树内容不传承。

## Request、重试与 Gate

Run 缺少输入时向 Project 提交类型化 [Request](./project.md) 创建命令，只阻塞声明的范围；Project 独占 Request 生命周期。Request 冻结截止时间与 `fail | cancel` 默认策略。解决与过期的跨模块事务都以比较并交换校验精确 Request 和阻塞项版本；只有解决可以写答案投递，过期不能猜测答案，而要按冻结策略结束对应 Attempt、Seat 或 Obligation。

Run 只在匹配确认回执或观测后恢复绑定执行；节点仍通过正常 Result Proposal/Receipt 路径完成。Dagu `human.task` 只是每个 HCTL 外部节点的机械暂停原语，不构成第二条人类输入或完成路径。

“重试”不是一个概念，而是五种身份不同的路径；混用它们会复制票数、绕过验收或复活旧执行：

| 路径 | 触发 | 产生的新身份 | 不变的身份 |
| --- | --- | --- | --- |
| 传输重投 | 投递超时、确认回执丢失 | 无：同一幂等键重投，重复命令返回原结果 | 一切领域对象 |
| 候选切换 | 类型化技术故障 | 同一 Seat 下的新 Attempt | Obligation、Seat、票位 |
| 引擎重试 | Engine 检查点再次进入等待态 | control 按新观察序号创建新的 Obligation（旧 Obligation 及其 Seat/Attempt 置为被替代） | Run |
| 语义返工 | changes_requested 汇总，且分歧落点在实现 | 新 ChangeSet Revision/Artifact Revision，旧票失效并重新过 Gate（全量或按策略增量） | Run、Task Revision |
| 替代执行 | 范围、验收、候选或权限变化 | 替代 Run 或新 Task Revision | Project、Task 身份 |

只有冻结策略列明的类型化技术故障才可以切换 Attempt，例如候选特有的认证、配额或网络故障，进程或运行时丢失，以及租约超时。control 先隔离当前代次，再在候选、预算和剩余截止时间允许时，于同一 Seat 创建新 Attempt。

候选耗尽且必须取得额外输入或授权时创建 Request；否则把 Seat/Obligation 标为类型化技术失败，不能无限等待或伪装成语义驳回。单个 Seat 的接受（`accepted`）、驳回（`rejected`）或要求修改（`changes_requested`）只是归约器输入；只有策略声明的否决权或汇总结果才触发返工，不能用负面票偷偷更换裁判。要求修改（`changes_requested`）可携带分歧落点实现内（`implementation`）或契约内（`contract`）：落在实现时按语义返工路径处理；落在契约时，归约器不进入返工也不自动替代，只把 Task 标为需要关注并建议采纳新 Task Revision，替代与否归人。

Gate 是 Run 内由 Workflow Revision 与 Run Manifest 冻结的治理节点和规则，不是独立模块。它的每个 Seat 绑定同一精确 ReviewSubjectRef、评审策略引用与摘要、根 Context Manifest 引用与摘要、必需 Skill 引用与摘要和能力与权限策略引用与摘要，并各自冻结精确 Participant revision 与 Project 参与者授权条目。

被评审 Revision 的作者或生产者不得占用必需评审 Seat；必需评审 Seat 绑定互不相同的 Participant revision。备用 Attempt 必须继承原 Seat 的逻辑身份和全部评审依据，不能借更换 Worker Profile 改变 Context、Skill、权限、票位或绕过分离。

control 与工具箱在计票时同时校验生产者、Participant、角色和权限。重复、越权、过期、身份冲突或摘要不匹配的票不计数；同一 Seat 的备用 Attempt 不增加票。

Gate 只证明逻辑 Participant 与生产者/评审者分离，不证明物理或组织独立。受控端口能认证的供应端、模型和操作者信息必须按已知（`known`）或未知（`unknown`）展示；Participant、Harness 或模型自报不能把未知变成已知。策略要求物理或组织独立、但当前端口无法认证时，Gate 必须返回不支持（`unsupported`）。

达到法定票数后，control 撤销剩余 Attempt 并提交汇总 Verdict/Receipt，再完成 Engine 检查点。剩余票已不可能达到门槛时，Gate 返回法定票数不可达（`quorum-unreachable`），使 Obligation 失败并沿 Workflow Revision 的失败边推进。返工产生新 Revision 后，旧票失效，新的 Revision 必须重新通过 Gate。Gate 策略可声明**返工轮数上限**（默认值见[交付文档](../delivery.md#运行默认值)）：达到上限后按法定票数不可达同路处理——Obligation 失败并沿失败边推进，或按策略创建 Request 找人；不得无上限返工。Gate 策略还可声明**增量评审**：新 Revision 的评审包附带与上一版的差异指针，席位可只读差异；未声明时每轮全量重评。Task Revision 只有在验收契约变化时才更新。

## Run → Task

Run 终态只说明 Workflow 到达经 HCTL 归约器确认的终点，不直接改写 [Task](./task.md)；Task 终结的获准来源见[Task 写入约束](./task.md#写入约束)。绑定精确 Task Revision 的 Run 只有满足上述正常完成谓词后，完成事务才把该 Task 的占用标记从 `active` 以比较并交换推进为 `completion_pending(run_ref)`，并以 Run/Task 派生的稳定幂等键写内部“完成 Task”命令 outbox。待完成状态阻止新 Run 插入。

随后 Task 按同一个用户级账本中的当前 Revision、来源的新鲜度与分歧和逐项证据独立校验，并在成功 Receipt 或持久拒绝结果的事务中清除该占用标记。Task 拒绝不回滚 Run。失败、已取消或被替代的 Run 只有满足上一段隔离前置，其终态事务才释放旧占用标记；它们只形成需要关注或历史，不提交完成或取消 Task。

完成谓词只依据账本事实与外部证据的当前回读。引擎停报进度时，control 只把 Run–Engine Binding 标为分歧，不阻止 Run 完成。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL 词 | 外部体系词 | 差异 |
| --- | --- | --- |
| Workflow Revision | Dagu 的 YAML DAG definition / BPMN 的 process definition | 引擎产物不能反向定义它；它先于任何引擎存在 |
| Engine Deployment | 引擎里注册的 definition 版本 | 额外固定编译器与 Profile，供分歧检测 |
| Run | workflow execution / process instance | Run 还冻结授权、候选、权限与预算，不只是一次实例 |
| 引擎外部检查点 | Dagu 的 processless `human.task` / Camunda 的 external task 模式 | Dagu 名称虽含 human，在 HCTL 中只是引擎报告的进度点：control 观察其等待态后创建 Obligation，账本结果落定后再推进检查点；场景客户端不得直接操作 |
| Run–Engine Binding | execution id + correlation key | 只作绑定与恢复关联，不成为 Run 身份 |
| 引擎 retry / timer | 引擎原生能力 | 只改变引擎报告的进度，不产生 HCTL 语义结果 |
| Obligation / Seat / Verdict / Gate Receipt | 无对应 | 差异化核心：Dagu `human.task` 只有等待、参数与引擎侧进度，不提供 HCTL 的身份、候选、法定票数或 Receipt；“谁有资格、结果是否有效、凭什么算完成”只在 HCTL 侧 |

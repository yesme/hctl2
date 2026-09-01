# Agent 模块约束

> 状态：规范性约束 · 草案 v0.15.5<br>
> 本文是 Agent 模块对象、状态机与写入约束的唯一权威。设计正文见 [Agent 与 Terminal](../agent.md)；模块交接见[连接约束](./connections.md)，共享机制见[系统边界](./system.md)，族语义与词汇分类见[约束层总则](./README.md)。

## 对象

Agent 模块只负责把获准的 Execution Spec 变成物理执行，并提供观察、隔离和恢复。Project 拥有 Room Invocation，Run 拥有 Attempt；两者各自拥有对应的 Execution Spec。Agent 不决定 Project、Task 或 Run 的领域结果，Repo Instance 也仍归系统层所有。

每次执行的接入方式——ACP、app-server、SDK、PTY（伪终端）、钩子及其降级能力——由该次 Execution Spec 冻结，不是独立对象。

| 对象 | 含义 |
| --- | --- |
| Worker Profile | Harness、模型、模式、权限、环境与可选执行加固声明的可复用配置 |
| Harness 目录 | 三类探测事实：定义（Harness 是什么）、本机安装（在哪里）、实测能力（实际支持什么）；不设类名 |
| ChangeSet / ChangeSet Revision | 一次获准写入边界及其不可变 Git 快照 |
| Write Lease | ChangeSet 的独占写入权与失权拦截（Lease 族） |
| 外部副作用命令（executor = tool）/ Integration Receipt | 把精确 ChangeSet Revision 集成到目标 ref 的持久授权及回读证明 |
| Execution Runtime | 一次执行的主机、隔离域和代次（owner = Attempt \| Room Invocation）；终端通道是其字段组 |
| Attach Descriptor / Terminal Input Lease | 对精确目标的短期连接票据和单输入者租约 |
| Result Proposal / Evidence | Harness 提交给上层校验的结果和观测，不是 Verdict/Receipt |

## 写入约束

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| Worker Profile / Harness 绑定 | immutable revision + current pointer | control 处理「创建/更新/解析绑定」命令；Agency 报告名册与探测能力 | 活动 Invocation/Attempt 始终引用原 revision |
| ChangeSet / Write Lease | change_set_version、current revision；lease 为待启动 / 活跃 / 撤销中 / 已撤销 | control 准入「预备/授予/撤销/封存」命令，工具箱物化并回读 Git 并执行失权 | 一个 ChangeSet 至多一个活跃 lease；ChangeSet Revision 只追加 |
| 外部副作用命令（executor = tool）/ Integration Receipt | intent state version；待启动 / 结果未知 / 成功 / 失败；Receipt immutable | control 准入「合入 ChangeSet」命令；工具箱执行本地 Git 集成并回读；远端 SCM 是同族外部副作用命令（executor = adapter，见[系统边界](./system.md#外部权威副作用)） | 同一 target ref/expected head 只允许一个获准结果；只有回读确认才能写 Receipt |
| Execution Runtime | `runtime_generation`；已预留 / 活跃 / 停止中 / 已停止 / 丢失 | control 记录 binding 并处理「激活/停止」命令；派出的 Agency 持有物理资源，control 记观测账 | 已停止/丢失不复活；恢复或接管使用新 runtime generation |
| Terminal Input Lease | lease generation；活跃 / 已撤销 / 已过期 | control 授予/撤销，Agency 适配代码只把当前租约的输入送入 API；provider 原生写入是否受租约约束按声明能力与 Execution Spec 输入策略冻结 | 一个受 HCTL 管理的目标最多一个活跃输入者；允许原生交互时不得宣称 provider 物理单写者 |
| Result Proposal / Evidence | immutable submission + producer sequence | Harness adapter 提交；control inbox 持久化；Project/Run 独占 admission | Proposal 不可改成 Verdict/Receipt；修正提交新 Proposal |

Worker Profile、Harness 名称或“支持 ACP”都不隐含能力。每次绑定都必须从实际探测结果中选择精确端口和降级方式，并冻结版本、配置、能力、信任级别和权限。

第一阶段，HCTL 启动的每个 Harness 都使用窄执行主体。以下三条底线不可关闭；账本单写者另有自己的[三条底线](./system.md#单写者)，两组互不替代。

### 不可关闭的三条底线

1. **工具不是人。** Harness、运行时钩子和模型只能提交 Result Proposal，不能提交治理命令。
2. **合入钥匙不进工具。** HCTL 不向 Harness 交付 control 客户端凭据、human principal credential、集成凭据或外部写凭据。目标引用、远端 SCM、任务后端和 chat 写入凭据只由持有当前代次栅栏的工具箱或适配器网关代用。
3. **隔离工作树。** Harness 只能在有效 Write Lease 下写当前 ChangeSet 的独立 Git 工作树和分支。它可以读取所属 Repo Instance 的 Git 公共目录与引用，也可以在当前 ChangeSet 分支提交。直接改写目标引用或其他 ChangeSet 现场不会取得集成权威，只会在回读时形成分歧。

### 可选执行加固

Worker Profile 可以声明 OS 沙箱、凭据代用范围、网络目的地和工具接口白名单。Execution Spec 必须冻结已声明项。未声明时，control 不施加这些加固，也不得记录为已生效；声明后若宿主或 Agency 无法可靠施加，control 必须拒绝激活并列出缺项。

### 有条件的安全输入

只有接入方式能保证敏感输入不进入环境变量、普通 stdin 历史、Room、Context、跟踪记录或回放时，Execution Spec 才能启用安全输入。

### 派工前校验

control 必须在交付派工前核对 Context Bundle 的实际交付摘要、Execution Spec 摘要和全部可验证的代次栅栏。

## ChangeSet 与 Git 事实

Git 工作树是 ChangeSet 的可替换物理资源，不永久属于 Project、Task、Room 或 Harness。一个 ChangeSet 同时最多有一个有效写入租约；候选切换、接管或取消必须先让旧写入者失权。

旧写入者无法证明已经失权时，control 默认保全并隔离原 Git 工作树和 ChangeSet，不授予新租约。有权 human actor 可以在预览残留后选择接管、封存、采用到另一 ChangeSet 或丢弃。只有自动恢复必须从获准基线创建新的 Git 工作树和 ChangeSet。系统不能把来源未知的未封存字节、旧租约或旧生产者身份自动带入新执行。

ChangeSet Revision 在有效租约下封存，至少固定：

```text
change_set_revision_id
+ change_set_id
+ parent_revision_id?
+ base_commit_sha
+ result_tree_sha
+ producer_ref             # human command，或精确 Invocation + invocation_version / Attempt + attempt_generation
+ revision_digest
```

评审 subject 对 {change_set_revision_id, change_set_id, parent_revision_id?, base_commit_sha, result_tree_sha} 使用[共享摘要规则](./system.md#命令与跨服务正确性)生成独立 review_subject_digest；它不是完整 ChangeSet Revision 的 revision_digest。`result_commit_sha` 只存在于后续 Integration/SCM evidence，不属于 ChangeSet Revision；因此给同一 Revision 增加不同 commit 包装不会改变其评审身份。返工或 result tree 变化创建新 Revision，旧 Revision 不改写。producer_ref 不进入 review subject digest，但 author/reviewer separation 必须沿它解析并校验当前逻辑身份。

模型自述不能证明集成成功。control 先持久化“合入 ChangeSet”意图和 outbox，工具箱再执行本地 Git 集成并回读；远端 push、PR 和 merge 走同族适配器命令。命令必须固定 ChangeSet Revision、来源与基线、目标引用、预期目标头、策略、适用 Verdict 和证据、actor 与权限、绑定和幂等键。成功回读后，control 才写唯一 Integration Receipt。

Harness 可以操作自己的 Git 工作树，但改写目标引用不产生 Receipt。下一次预览或回读只会显示预期目标头不匹配的分歧，由有权 actor 对账处理。工具箱校验 Git 基线、HEAD、tree、祖先关系、PR、检查、评审和目标分支头。

结果未知时，工具箱回读 Git 与 PR 状态；收敛前不得签发成功 Receipt 或清理现场。SCM 变更被中断时同样按结果未知处理，并返回类型化恢复动作。

失败、取消、租约撤销和资源清理都不等于放弃代码。物理清理默认保全：工具箱先确认所有已跟踪、未跟踪且尚未封存的修改已有可恢复副本，现场资源得到该确认才可拆除；有权 human actor 在预览残留后显式确认丢弃时，可以不留副本直接拆除。保全或封存失败且未获显式丢弃确认时，保留精确 worktree 路径、Git 状态和显式恢复动作，不能删除唯一副本。清理 worktree 也不删除领域历史。

## 运行时与观测

Run 经其 Attempt 可以有多个 Execution Runtime；Room Invocation 至多一个。Attempt 与 Room Invocation 各至多一组终端通道。Execution Runtime 可以是容器、隔离作用域或结构化会话，不以 TTY 存在为前提。

Room Invocation 拥有的 Execution Runtime 继承其 Execution Spec 的 `project_scope | repo_scope`；Attempt 拥有的运行时的 Project 范围来自 Run Manifest。repo-scoped 调用可以没有 Project ref，但仍必须保留精确 Room Invocation、Execution Runtime、binding、各层代次、权限和适用 fence；已知运行时不能被降级成无主进程或模糊仓库活动。

代次必须分层记录，不能共用一个模糊的 `generation`。语义归属者代次、物理运行时代次与基础设施代次栅栏是三层不同的身份；成员与推导规则见[代次家族总表](./system.md#代次家族)。替代任一层只使引用该层旧值的 HCTL 动作失效，不得顺带改写其他层的身份；Agency 不能执行的物理代次栅栏必须明确标为未生效。

Execution Runtime 由**Agency**（派出方）承载。Agency 是执行者供给受控端口：按冻结的 Execution Spec 受理派工，交付执行体、运行现场和访问通道，常驻持有现场并报告存活与恢复等级。第一阶段只采用 **Herdr**：它直接从已配置的 Harness 启动执行体，持有进程、PTY 和终端会话，并提供 API 与原生 TUI。HCTL 不再放置独立 Agency 组件或下一层终端运行服务。

control 是 Agency 的 HCTL 控制者，通过 Herdr 适配代码提交获准请求、核对交付结果并记账。替换未来的 Agency 不改变治理规则。派出交付物必须按冻结规格逐项核验后方可激活；缺项时列出缺项且不激活。Agency 在[七件事分层](../participant.md#七件事分层)中只提供模型、Worker Profile 和 Execution Runtime 这三类物理执行信息；Participant 身份和 Seat 仍由 control 账本拥有。

Agency 的接口约定**永不包含治理权威**：租约、代次、冻结规格、审计与恢复等级裁决只在 control 账本。Agency 自带的接管、单写者或“会话有效”记录只作执行协助与观测证据，不得写入或替代账本事实。

原生输入可以按下文输入策略成为正常的用户运行时输入，但不能承载要求物理代次栅栏的动作，也不能作为高证据类结果直接准入；绕过适配代码提交结构化结果仍不被接受。

进程、PTY、原始流与心跳由 Herdr 持有；control 经 Herdr 适配代码执行已获准的启动、输入、取消和停止，Attempt/Invocation 的领域生命周期仍由 control 推进。

判断进程是否存活及归谁所有时，优先采用 Herdr API 或进程证据；其次采用结构化生命周期事件或钩子；最后才参考标题和屏幕内容。判断语义状态时，优先采用结构化协议或原生钩子；其次采用转录推断；最后才参考标题和屏幕内容。低优先级信号不能覆盖仍有效的高优先级证据。每条观测记录来源、置信度、证据和观测时间，而且无论置信度多高都不能自行推进领域结果。

Execution Spec 必须冻结终端输入策略。`managed_single_writer` 要求所有输入经过当前连接票据、代次和 Terminal Input Lease 校验；供应端不能统一拦截全部写入时，系统必须关闭原生控制器。`native_interactive_allowed` 允许 Workbench 直连传输、Herdr TUI 或其他原生客户端向已映射的精确终端输入，并明确接受供应端无法逐次证明 actor、租约和代次。

后一模式中的输入是有效运行时输入，不是分歧，也不自动污染独立的 Git、SCM 或测试证据。执行记录必须标明输入来源不完整；不得声称物理单写者、完整回放，或由该输入产生 HCTL 命令或结果。切换策略必须创建新 Execution Spec 或替代执行，不能在活动执行背后静默放宽。

Agency 声明代次栅栏回显时，必须回显并校验代次与租约；未声明时只能记录 HCTL 入口校验。Agency 声明逐次输入记录时，每次输入必须关联 actor、租约和代次；未声明时不得声称来源完整。

Agency 声明事件游标时，必须报告序号和缺口；未声明时，事件流只能作为有界观测。Agency 声明退出与停止回读时，必须提供同一进程、PTY、退出码和停止结果的证据；证据不足时只能报告语义恢复、回放或丢失。

结构化事件统一归一为生命周期提示、工具调用、权限请求、文件变化、测试、用量和原始输出。未知事件保留原文并安全降级，不得凭渲染器猜测完成。

每个 harness 适配器必须为其接入端口声明终局结果清单，并逐项核对：

- 执行体进程正常退出但缺少清单要求的终局结果事件时，适配器必须合成类型化协议错误，不得默认成功。
- 由 control 主动取消导致的退出必须归因为取消，不得上报为执行失败。
- 观测上报通道失败时，只能显式标记该执行的观测截断并终结事件流，不得交付有缺口的事件流冒充完整历史。
- harness 内部派生的子执行体事件必须携带稳定的派生谱系引用，不得摊平进主执行流。

Proposal 头必须固定 proposal ID、归属者、运行时、适用的代次栅栏、Execution Spec、Context Bundle、绑定、生产者序号和幂等键。受信任的 `in_process` Proposal 使用缩减头，而且不得提交 ChangeSet。

每个输出项必须另带 schema key、content digest、候选产物引用和自己的代次字段组。只有 output schema 明确允许逐项准入时，归属者才能单独接受合格项；否则任一必需项不匹配都拒绝整组。任一代次、绑定、Bundle、租约或输出范围不匹配的项只能留作审计，不能让其他合格项替它背书。修正必须创建新 Proposal 和新的生产者序号，不得改写原项。

Harness、runtime hook 与模型只获得当前 Invocation/Attempt 所需的窄 execution principal，不能持有通用 command Submit credential、human principal credential、Task lifecycle 或 Room dispatch 权限。它们可以建议完成或建议下一位 Participant；建议经 Result Proposal 通道由 owner 准入，不是命令。

## 终端通道、连接与租约

恢复等级包括 exact attach、native handoff、structured inspect、semantic resume 和 replay，定义见[设计正文](../agent.md#terminal-场景)。这些能力可以并存；每项能力按自己的证据要求分别声明与降级，不能用一项的证据顶替另一项。

运行时绑定提交后，control 为 Execution Runtime 建立终端通道账目；物理通道、观察流与终端状态由 Agency 提供，HCTL 不转发或重放另一份 PTY 流。直接客户端按当前归属者、绑定与全部适用代次请求连接时，control 可以签发短期 Attach Descriptor，并为受管理写输入另行以比较并交换授予 Terminal Input Lease。Agency 适配代码只把仍匹配归属者、运行时、现场和绑定代次的获准动作送入 API。

Attach Descriptor 固定逻辑归属者、供应端终端 ID、主机、各层代次、能力、权限和过期时间。观察、终端输入或接管、Attempt 控制和安全输入分别授权，任一权限都不蕴含其他权限。一个目标可以有多个观察者；HCTL 管理的输入默认最多一个 Terminal Input Lease 持有者，接管必须原子撤销旧租约。

绑定声明 `native_interactive_allowed` 时，供应端原生客户端或 Workbench 直连传输可以不经该租约输入。control 把它记录为允许但无法逐次证明来源的运行时交互；该通道中的文字或所谓“完成”不能直接准入 HCTL 结果。

Execution Chat 投影是 Terminal 中绑定且只绑定一个精确 Room Invocation/invocation_version 或 Attempt/attempt_generation、对应 Execution Runtime/runtime_generation 与适用代次栅栏的结构化观察与控制视图。它不是 Room，也没有独立会话身份。适配器支持时，输入作为携带这些精确引用的获准 control 动作写回同一执行体；能力不足时准确降级为 structured inspect 或 terminal，不得改投另一个会话。

Execution Chat 中的输入和事件不会自动成为 Room 内容。只有显式 Share to Room 动作经 Project 命令准入后才能发布，并携带来源事件、执行归属者版本或代次、运行时代次，以及转录与证据来源。该投影消失或运行时被替代都不改变 Room 身份。

Workbench 或终端客户端退出不停止执行。断流按 runtime generation、来源流 sequence 和快照恢复；无法证明是同一进程时只能 semantic resume、replay 或新建执行，不能声称 exact attach。semantic resume 可以用自有观测留痕重建续跑输入；重建物按投影处理，不进入权威记录。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL | 外部体系 | 差异一句话 |
| --- | --- | --- |
| exact attach / detach | Herdr terminal connection | 连接或断开仍存活的 Herdr terminal；断开不停止执行，不能证明同一进程和 PTY 时不得声称 exact attach |
| Execution Runtime | Herdr workspace/tab/pane/terminal + 进程 | 可丢失、重建、接管；不承载 Project/Task/Run 的领域身份 |
| PTY | PTY（伪终端） | 同名同义的基础设施概念 |
| 结构化接入 | ACP（Agent Client Protocol）等代理协议 | HCTL 只把它当作受控端口能力之一；协议会话不是 HCTL 身份 |
| semantic resume | 各 Harness 原生的会话恢复（如 codex resume） | 恢复的是上下文，可能创建新进程；不等于 exact attach |
| replay | 终端录像回放 | 只读历史，不冒充存活会话 |
| Write Lease / Terminal Input Lease | 无对应 | 差异化语义：单写入者与单输入者，配代次失权 |

# Agent 模块约束

> 状态：规范性约束 · 草案 v0.15.5<br>
> 本文是 Agent 模块对象、状态机与写入约束的唯一权威。设计正文见 [Agent 与 Terminal](../agent.md)；模块交接见[连接约束](./connections.md)，共享机制见[系统边界](./system.md)，族语义与词汇分类见[约束层总则](./README.md)。

## 对象

Agent 模块把 [Project](./project.md) 拥有的 Room Invocation 或 [Run](./run.md) 拥有的 Attempt 所携带的 Execution Spec 变成可观察、可隔离、可恢复的物理执行。它不决定 Project 目标、Task 完成、Run Gate、下一条 Room 协作边或领域权限。每次执行的接入方式（ACP/app-server/SDK/PTY（伪终端）/钩子及其降级能力）由该次 Execution Spec 冻结，不是独立对象；Execution Spec 由 Project 与 Run 各自作为 owner 定义。所选 Repo Instance 只是[系统约束](./system.md#repo-与执行现场)中的物理执行现场引用，不由 Project 或 Agent 聚合拥有。

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

第一阶段 HCTL 启动的每个 Harness 都以窄 execution principal 运行，Harness 治理的三条底线不可声明关闭（账本单写者另有自己的三条底线，见[系统边界](./system.md#单写者)，两组互不顶替）：**工具不是人**——Harness、runtime hook 与模型只有 Result Proposal 通道，不是治理命令入口（两类入口见[系统边界](./system.md#命令与跨服务正确性)）；**合入钥匙不进工具**——HCTL 不向 Harness 交付集成与外部写凭据（目标 ref、远端 SCM、任务后端、chat 写入），这些凭据只由持有当前 control/site/backend fence 的工具箱或 adapter 网关代用，Harness 也不获交付 control 客户端凭据与 human principal credential；**隔离工作树**——写入只及于该 ChangeSet 的独立 worktree 与本 ChangeSet 分支，以有效 Write Lease 为前提。在此之内 Harness 是普通的 Git 用户：可读所属 Repo Instance 的 Git common-dir 与 refs（log、fetch、比对目标分支）并在本 ChangeSet 分支上提交；绕过「合入 ChangeSet」命令直接改写目标 ref 或其他 ChangeSet 现场不取得集成 authority，只回读为 drift。OS 强制的执行沙箱、凭据代用范围、网络目的地与工具接口白名单是可选执行加固：由 Worker Profile 声明、随 Execution Spec 冻结，并在启动前按所选 Agency 与宿主实际能力核验；未声明则不施加、不拦启动、也不得记录为已生效；已声明而当前实现不能可靠施加时，control 不激活并列出缺项。安全输入只有在所选接入方式声明并实现不进入环境变量、普通 stdin 历史、Room、Context、trace 或 replay 的保证时才能启用。control 在派工交付前必须核对 Context Bundle 的交付 bytes digest、spec digest 和全部可验证 fence。

## ChangeSet 与 Git 事实

Worktree（Git 工作树）是 ChangeSet 的可替换物理资源，不永久属于 Project、Task、Room 或 Harness。一个 ChangeSet 同时最多一个有效写入租约；候选切换、接管或取消必须先让旧 writer 失权。恢复时若无法证明旧 writer 已经静默并被 fence，默认不授予新写租约：原 worktree **和原 ChangeSet identity** 先保全、隔离并提示。有权 human actor 预览证据与残留后，可以显式选择接管（对原 worktree 与原 ChangeSet 推进代次并授予新 lease）、把残留封存为原 producer 的 Revision、采用到另一 ChangeSet，或显式丢弃。自动恢复路径必须从获准 baseline 创建新物理 worktree 与新 ChangeSet 再取得新 lease；不能把未知来源的未封存字节、旧 lease 或旧 producer identity 自动洗入新执行。

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

模型自述“已合并”不可信。本地「合入 ChangeSet」命令至少固定 ChangeSet Revision、source/base、target ref、expected target head、策略、适用 Verdict/evidence、actor/permission、binding 和幂等键；有权 human actor 或冻结 Workflow reducer 提交后，control 先持久化 intent/outbox，工具箱才执行并回读 Git，成功时写唯一 Integration Receipt。远端 push/PR/merge 是同族外部副作用命令（executor = adapter），字段与本地「合入 ChangeSet」命令等价。Harness/model 可以在 worktree 内做普通 Git 操作，但不能取得集成 authority：绕过该命令直接改写目标 ref 不产生 Integration Receipt，只在下一次预览或回读时表现为 expected target head 不匹配的 drift，由有权 actor 对账处理。工具箱校验 Git base/HEAD/tree、祖先关系、PR、检查、评审和目标分支头。SCM 变更中断或结果未知时，该命令保持结果未知，工具箱必须回读 HEAD、index、worktree/merge 状态、PR head 和目标分支头，返回类型化恢复动作；收敛前不得签发成功 Receipt 或清理所需现场。

失败、取消、租约撤销和资源清理都不等于放弃代码。物理清理默认保全：工具箱先确认所有已跟踪、未跟踪且尚未封存的修改已有可恢复副本，现场资源得到该确认才可拆除；有权 human actor 在预览残留后显式确认丢弃时，可以不留副本直接拆除。保全或封存失败且未获显式丢弃确认时，保留精确 worktree 路径、Git 状态和显式恢复动作，不能删除唯一副本。清理 worktree 也不删除领域历史。

## 运行时与观测

Run 经其 Attempt 可以有多个 Execution Runtime；Room Invocation 至多一个。Attempt 与 Room Invocation 各至多一组终端通道。Execution Runtime 可以是容器、隔离作用域或结构化会话，不以 TTY 存在为前提。

Room Invocation 拥有的 Execution Runtime 继承其 Execution Spec 的 `project_scope | repo_scope`；Attempt 拥有的运行时的 Project 范围来自 Run Manifest。repo-scoped 调用可以没有 Project ref，但仍必须保留精确 Room Invocation、Execution Runtime、binding、各层代次、权限和适用 fence；已知运行时不能被降级成无主进程或模糊仓库活动。

代次必须分层记录而不能共用一个模糊 `generation`：语义 owner（`invocation_version`／`attempt_generation`）、物理执行（`runtime_generation`）与基础设施 fence（control／site／Agency binding）是三层不同的身份，成员与推导禁令见[代次家族总表](./system.md#代次家族)。本模块的义务是：替代任一层只使引用该层旧值的 HCTL 动作失效，不得顺带把别层 identity 改写成新值；Agency 不能执行的物理 fence 必须明确标为未生效。

Execution Runtime 由**Agency**（派出方）承载。Agency 是执行者供给受控端口：按冻结的 Execution Spec 受理派工，交付执行体及其运行现场与访问通道，常驻持有现场并报告存活与恢复等级。第一阶段只采用 **Herdr**：它直接从已配置的 Harness 启动执行体，持有进程、PTY 和终端会话，并提供 API 与原生 TUI。HCTL 不再放置独立 Agency 组件或下一层终端运行服务。control 是 Agency 的 HCTL 控制者，通过 Herdr 适配代码提交获准请求、核对交付结果并记账；替换未来的 Agency 不改变治理约束。派出交付物必须按冻结规格逐项核验后方可激活，缺项列出且不激活；派出不转移参与者身份：Agency 供给的是七层身份链的下层（模型、执行者配置、一次物理执行），Participant 身份与席位仍由账本拥有并绑定。

Agency 约束**永不包含治理权威**：租约、代次、冻结规格、审计与恢复等级裁决只在 control 账本；Agency 自带的接管、单写者或“会话有效”记录只作执行协助与观测证据，不得写入或替代账本事实。原生输入可以按下文输入策略成为正常的用户运行时输入，但不能承载要求物理 fence 的动作，也不能作为高证据类结果直接准入；绕过适配代码提交结构化结果仍不被接受。

进程、PTY、原始流与心跳由 Herdr 持有；control 经 Herdr 适配代码执行已获准的 start/input/cancel/stop，Attempt/Invocation 的领域 lifecycle 仍由 control 推进。存活与所有权观测按 Herdr API/进程 > 结构化 lifecycle 事件或 hook > title/screen 仲裁，语义观测按结构化提供方协议或原生 hook > 转录推断 > title/screen 仲裁；低优先级信号不能覆盖仍有效的高优先级证据。每条观测记录 source、confidence、evidence 与 observed_at，且无论置信度多高都不能自行推进领域结果。

Execution Spec 必须冻结 terminal input policy：`managed_single_writer` 要求所有输入经当前 descriptor、generation 与 Terminal Input Lease 校验，provider 不能统一拦截所有写入时就关闭原生 controller；`native_interactive_allowed` 允许 Workbench 直连 transport、Herdr TUI 或其他原生客户端向已映射的精确 terminal 输入，并明确接受 provider 无法逐次证明 actor/lease/generation。后一模式中的输入是有效运行时输入，不是 drift，也不自动污染独立的 Git/SCM/Test evidence；但执行记录必须标明输入 provenance 不完整，不能声明物理单写者、完整 replay 或由该输入产生 HCTL 命令/结果。切换策略创建新 Execution Spec 或替代执行，不能在活动执行背后静默放宽。

Agency 声明栅栏回显时，原样携带并回显代次与租约引用，拒绝不匹配项；未声明时只在 HCTL 入口校验，物理 fence 记为未生效。Agency 声明逐次输入记录时，每次输入关联 actor/lease/generation；未声明时原生交互按来源不完整记录，物理单写者与完整 replay 降为不可用。Agency 声明事件游标（sequence/gap）时，事件流携带来源序号并显式报告缺口；未声明时事件流只作有界观测，不能表示完整 trace。Agency 声明退出与停止回读时，回报同一进程/PTY、退出码与 stop 结果的实际证据；未声明或证据不足时，只能报告 semantic resume、replay 或丢失。

结构化事件统一归一为生命周期提示、工具调用、权限请求、文件变化、测试、用量和原始输出。未知事件保留原文并安全降级，不得凭渲染器猜测完成。

每个 harness 适配器必须为其接入端口声明终局结果契约：执行体进程正常退出但缺少契约要求的终局结果事件时，适配器必须合成类型化协议错误，不得默认成功；由 control 主动取消导致的退出必须归因为取消，不得上报为执行失败。观测上报通道失败时，只能显式标记该执行的观测截断并终结事件流，不得交付有缺口的事件流冒充完整历史。harness 内部派生的子执行体事件必须携带稳定的派生谱系引用，不得摊平进主执行流。

物理执行的每个 Result Proposal 固定 proposal ID、owner kind/ID + `invocation_version | attempt_generation`、Execution Runtime ID + `runtime_generation`、`control_writer_generation`、Repo Instance + `site_generation`、backend/Agency owner generation、Execution Spec 与 Context Bundle digest、实际 Harness/Runtime binding revision、producer sequence、适用 lease generation/fence 和 idempotency key。只有 Execution Spec 明示的受信任 `in_process` 执行可省略 runtime/site/backend/lease，改为固定 owner、control writer、Extension/Resolved Port Binding、spec/bundle digest 与 producer sequence，也不得提交 ChangeSet。Proposal 内每个输出项还必须分别固定 schema key、content digest、ChangeSet Revision/Artifact candidate/Evidence refs，以及产生该项的同一适用 generation tuple；不得用顶层“本次执行”概括后混入旧 Attempt、旧 runtime 或另一现场的输出。Harness 可以提交提案，但 Project/Run 才能逐项校验；只有冻结 output schema 明确声明可独立准入时，合格项才能单独进入 owner，其他情况下任一 required 项不匹配就拒绝整组。任一代次、binding、bundle、lease 或输出范围不匹配的项只能留作审计，不能让其他合格项替它背书。修正创建新 Proposal 和新 producer sequence，不改写原项。

Harness、runtime hook 与模型只获得当前 Invocation/Attempt 所需的窄 execution principal，不能持有通用 command Submit credential、human principal credential、Task lifecycle 或 Room dispatch 权限。它们可以建议完成或建议下一位 Participant；建议经 Result Proposal 通道由 owner 准入，不是命令。

## 终端通道、连接与租约

Terminal 各能力（exact attach、native handoff、structured inspect、semantic resume、replay，见[设计正文](../agent.md#terminal-场景)）可以并存；每项能力按自己的证据要求分别声明与降级（见[运行时与观测](#运行时与观测)），不能用一项的证据顶替另一项。运行时绑定提交后，control 为 Execution Runtime 建终端通道账目；物理通道、观察流与终端状态由 Agency 提供，HCTL 不转发或重放另一份 PTY 流。direct client 按当前 owner/binding 与全部适用代次请求连接时，control 可以签发短期 Attach Descriptor，并为受管理写输入另行 CAS Terminal Input Lease；Agency 适配代码只把仍匹配 owner/runtime/site/binding generation 的获准动作送入 API。Attach Descriptor 固定逻辑 owner、provider terminal ID、host、各层代次、能力、权限和过期时间；观察、终端输入或接管、Attempt 控制和安全输入分别授权，任一权限都不蕴含其他权限。一个目标可以有多个观察者，HCTL 管理的输入默认最多一个 Terminal Input Lease 持有者；接管原子撤销旧租约。binding 声明 `native_interactive_allowed` 时，provider 原生客户端或 Workbench 直连 transport 可以不经该租约输入；control 把它记录为允许但无法逐次证明来源的运行时交互，该通道中的文字或所谓“完成”不能直接准入 HCTL 结果。

Execution Chat projection 是 Terminal 中绑定且只绑定一个精确 Room Invocation/invocation_version 或 Attempt/attempt_generation、对应 Execution Runtime/runtime_generation 与适用 fence 的结构化观察与控制视图，不是 Room，也没有独立 conversation identity。adapter 支持时，输入作为携带这些精确引用的获准 control action 写回同一执行体；能力不足时准确降级为 structured inspect 或 terminal，不得改投另一个会话。

Execution Chat 中的输入和事件不会自动成为 Room 内容。只有显式 Share to Room 动作经 Project 命令准入后才能发布，并携带 source event、execution owner version/generation、runtime generation 及 transcript/evidence provenance；该投影消失或 runtime 被替代都不改变 Room 身份。

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

# Agent 模块合同

> 状态：规范性合同 · 草案 v0.13.0<br>
> 本文是 Agent 模块对象、状态机与写入合同的唯一权威。设计正文见 [Agent 与 Terminal](../agent.md)；模块交接见[连接合同](./connections.md)，共享机制见[系统边界](./system.md)，族语义与词汇分类见[合同层总则](./README.md)。

## 对象

Agent 模块把 [Project](./project.md) 拥有的 Room Invocation 或 [Run](./run.md) 拥有的 Attempt 所携带的 Execution Spec 变成可观察、可隔离、可恢复的物理执行。它不决定 Project 目标、Task 完成、Run Gate、下一条 Room 协作边或领域权限。每次执行的接入方式（ACP/app-server/SDK/PTY（伪终端）/钩子及其降级能力）由该次 Execution Spec 冻结，不是独立对象；Execution Spec 由 Project 与 Run 各自作为 owner 定义。所选 Repo Instance 只是[系统合同](./system.md#repo-与执行现场)中的物理执行现场引用，不由 Project 或 Agent 聚合拥有。

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

## 写入合同

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| Worker Profile / Harness 绑定 | immutable revision + current pointer | control 处理「创建/更新/解析绑定」命令；agentd 只报告探测能力 | 活动 Invocation/Attempt 始终引用原 revision |
| ChangeSet / Write Lease | change_set_version、current revision；lease 为待启动 / 活跃 / 撤销中 / 已撤销 | control 准入「预备/授予/撤销/封存」命令，工具箱物化并回读 Git，agentd 执行失权 | 一个 ChangeSet 至多一个活跃 lease；ChangeSet Revision 只追加 |
| 外部副作用命令（executor = tool）/ Integration Receipt | intent state version；待启动 / 结果未知 / 成功 / 失败；Receipt immutable | control 准入「合入 ChangeSet」命令；工具箱执行本地 Git 集成并回读；远端 SCM 是同族外部副作用命令（executor = adapter，见[系统边界](./system.md#外部权威副作用)） | 同一 target ref/expected head 只允许一个获准结果；只有回读确认才能写 Receipt |
| Execution Runtime | `runtime_generation`；已预留 / 活跃 / 停止中 / 已停止 / 丢失 | control 记录 binding 并处理「激活/停止」命令；agentd 持有物理资源与观测 | 已停止/丢失不复活；恢复或接管使用新 runtime generation |
| Terminal Input Lease | lease generation；活跃 / 已撤销 / 已过期 | control 授予/撤销，agentd 输入门执行；接管原子撤销旧 lease | 一个目标最多一个活跃输入者 |
| Result Proposal / Evidence | immutable submission + producer sequence | Harness adapter 提交；control inbox 持久化；Project/Run 独占 admission | Proposal 不可改成 Verdict/Receipt；修正提交新 Proposal |

Worker Profile、Harness 名称或“支持 ACP”都不隐含能力。每次绑定都必须从实际探测结果中选择精确端口和降级方式，并冻结版本、配置、能力、信任级别和权限。

第一阶段 HCTL 启动的每个 Harness 都以窄 execution principal 运行，三条底线不可声明关闭：**工具不是人**——Harness、runtime hook 与模型只有 Result Proposal 通道，不是治理命令入口（三个入口见[系统边界](./system.md#命令与跨服务正确性)）；**合入钥匙不进工具**——HCTL 不向 Harness 交付集成与外部写凭据（目标 ref、远端 SCM、任务后端、chat 写入），这些凭据只由持有当前 control/site/backend fence 的工具箱或 adapter 网关代用，Harness 也不获交付 control 客户端凭据与 human principal credential；**隔离工作树**——写入只发生在该 ChangeSet 的独立 worktree 与有效 Write Lease 内。在此之内 Harness 是普通的 Git 用户：可读所属 Repo Instance 的 Git common-dir 与 refs（log、fetch、比对目标分支）并在本 ChangeSet 分支上提交；绕过「合入 ChangeSet」命令直接改写目标 ref 或其他 ChangeSet 现场不取得集成 authority，只回读为 drift。OS 强制的执行沙箱、凭据网关代用范围、网络目的地与工具接口白名单是可选执行加固：由 Worker Profile 声明、随 Execution Spec 冻结，agentd 按声明施加并把实际生效项记录为 Execution Runtime 事实；未声明或宿主不支持不阻止受治理执行启动，也不得记录为已生效。安全输入经 agentd 的一次性通道写给精确 runtime，不进入环境变量、普通 stdin 历史、Room、Context、trace 或 replay。agentd 启动前必须核对 Context Bundle 的交付 bytes digest、spec digest 和全部 fence。

## ChangeSet 与 Git 事实

Worktree（Git 工作树）是 ChangeSet 的可替换物理资源，不永久属于 Project、Task、Room 或 Harness。一个 ChangeSet 同时最多一个有效写入租约；候选切换、接管或取消必须先让旧 writer 失权。恢复时若无法证明旧 writer 已经静默并被 fence，原 worktree **和原 ChangeSet identity** 都不得授予新写租约，只能保全并隔离；新的执行必须从获准 baseline 创建新物理 worktree 与新 ChangeSet，再取得新 lease。显式人工恢复可以在预览旧残留后把其封存为原 producer 的 Revision，或通过新命令采用到另一 ChangeSet；不能把未知来源的未封存字节、旧 lease 或旧 producer identity 自动洗入新执行。

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

失败、取消、租约撤销和资源清理都不等于放弃代码。物理清理前，工具箱必须确认所有已跟踪、未跟踪且尚未封存的修改已有可恢复副本，agentd 只有得到该确认才可拆除资源；保全或封存失败时保留精确 worktree 路径、Git 状态和显式恢复动作，不能删除唯一副本。清理 worktree 也不删除领域历史。

## 运行时与观测

Run 经其 Attempt 可以有多个 Execution Runtime；Room Invocation 至多一个。Attempt 与 Room Invocation 各至多一组终端通道。Execution Runtime 可以是容器、隔离作用域或结构化会话，不以 TTY 存在为前提。

Room Invocation 拥有的 Execution Runtime 继承其 Execution Spec 的 `project_scope | repo_scope`；Attempt 拥有的运行时的 Project 范围来自 Run Manifest。repo-scoped 调用可以没有 Project ref，但仍必须保留精确 Room Invocation、Execution Runtime、binding、各层代次、权限和适用 fence；已知运行时不能被降级成无主进程或模糊仓库活动。

代次必须分层记录而不能共用一个模糊 `generation`：语义 owner 是 Room Invocation 的 `invocation_version` 或 Attempt 的 `attempt_generation`；物理 Execution Runtime 在激活映射时取得独立 `runtime_generation`；基础设施 envelope 另带 `control_writer_generation`、Repo Instance 的 `site_generation` 与运行时 backend/agentd owner generation。Participant revision、Project Role Binding version 与 producer sequence 都不是 generation。替代任一层只使引用该层旧值的动作失效，不得顺带把别层 identity 改写成新值。

agentd 拥有进程、PTY、原始流、心跳和主机观测，并执行 control 已获准的 start/input/cancel/stop；Attempt/Invocation 的领域 lifecycle 仍由 control 推进。存活与所有权观测按运行时后端/进程/租约 > 结构化 lifecycle 事件或 hook > title/screen 仲裁，语义观测按结构化提供方协议或原生 hook > 转录推断 > title/screen 仲裁；低优先级信号不能覆盖仍有效的高优先级证据。每条观测记录 source、confidence、evidence 与 observed_at，且无论置信度多高都不能自行推进领域结果。

结构化事件统一归一为生命周期提示、工具调用、权限请求、文件变化、测试、用量和原始输出。未知事件保留原文并安全降级，不得凭渲染器猜测完成。

每个 harness 适配器必须为其接入端口声明终局结果契约：执行体进程正常退出但缺少契约要求的终局结果事件时，适配器必须合成类型化协议错误，不得默认成功；由 control/agentd 主动取消导致的退出必须归因为取消，不得上报为执行失败。观测上报通道失败时，只能显式标记该执行的观测截断并终结事件流，不得交付有缺口的事件流冒充完整历史。harness 内部派生的子执行体事件必须携带稳定的派生谱系引用，不得摊平进主执行流。

物理执行的每个 Result Proposal 固定 proposal ID、owner kind/ID + `invocation_version | attempt_generation`、Execution Runtime ID + `runtime_generation`、`control_writer_generation`、Repo Instance + `site_generation`、backend/agentd owner generation、Execution Spec 与 Context Bundle digest、实际 Harness/Runtime binding revision、producer sequence、适用 lease generation/fence 和 idempotency key。只有 Execution Spec 明示的受信任 `in_process` 执行可省略 runtime/site/backend/lease，改为固定 owner、control writer、Extension/Resolved Port Binding、spec/bundle digest 与 producer sequence，也不得提交 ChangeSet。Proposal 内每个输出项还必须分别固定 schema key、content digest、ChangeSet Revision/Artifact candidate/Evidence refs，以及产生该项的同一适用 generation tuple；不得用顶层“本次执行”概括后混入旧 Attempt、旧 runtime 或另一现场的输出。Harness 可以提交提案，但 Project/Run 才能逐项校验；只有冻结 output schema 明确声明可独立准入时，合格项才能单独进入 owner，其他情况下任一 required 项不匹配就拒绝整组。任一代次、binding、bundle、lease 或输出范围不匹配的项只能留作审计，不能让其他合格项替它背书。修正创建新 Proposal 和新 producer sequence，不改写原项。

Harness、runtime hook 与模型只获得当前 Invocation/Attempt 所需的窄 execution principal，不能持有通用 command Submit credential、human principal credential、Task lifecycle 或 Room dispatch 权限。它们可以建议完成或建议下一位 Participant；建议经 Result Proposal 通道由 owner 准入，不是命令。

## 终端通道、连接与租约

Terminal 各能力（exact attach、native handoff、structured inspect、semantic resume、replay，见[设计正文](../agent.md#terminal-场景)）可以并存但不能互相冒充。运行时绑定提交后，control 为 Execution Runtime 建终端通道账目；agentd 实现物理终端网关。认证场景客户端请求连接时，control 按当前 owner/binding 与全部适用代次签发短期 Attach Descriptor，并为写输入另行 CAS Terminal Input Lease；写输入、接管和安全输入只接受认证入口赋予 human provenance 的请求。agentd 只接受由当前 control writer 签发且仍匹配 owner/runtime/site/backend generation 的 descriptor/lease。Attach Descriptor 固定逻辑 owner、后端目标、host、各层代次、能力、权限和过期时间；观察 trace/结构化流、终端输入或接管、Attempt 控制和安全输入分别授权，任一权限都不蕴含其他权限。一个目标可以有多个观察者，默认最多一个 Terminal Input Lease 持有者；接管原子撤销旧租约，安全输入不得进入普通 trace、Room 或 replay。绕过 agentd 直连 backend 只能标为带外诊断，该通道产生的输入、输出或“完成”不能准入 HCTL 结果。

Execution Chat projection 是 Terminal 中绑定且只绑定一个精确 Room Invocation/invocation_version 或 Attempt/attempt_generation、对应 Execution Runtime/runtime_generation 与适用 fence 的结构化观察与控制视图，不是 Room，也没有独立 conversation identity。adapter 支持时，输入作为携带这些精确引用的获准 control action 写回同一执行体；能力不足时准确降级为 structured inspect 或 terminal，不得改投另一个会话。

Execution Chat 中的输入和事件不会自动成为 Room 内容。只有显式 Share to Room 动作经 Project 命令准入后才能发布，并携带 source event、execution owner version/generation、runtime generation 及 transcript/evidence provenance；该投影消失或 runtime 被替代都不改变 Room 身份。

Workbench 或终端客户端退出不停止执行。断流按 runtime generation、provider sequence 和快照恢复；无法证明是同一进程时只能 semantic resume、replay 或新建执行，不能声称 exact attach。semantic resume 可以用自有观测留痕重建续跑输入；重建物按投影处理，不进入权威记录。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL | 外部体系 | 差异一句话 |
| --- | --- | --- |
| exact attach / detach | tmux attach / detach | 语义相同：连接或断开仍存活的会话；detach 不停止执行 |
| Execution Runtime | tmux session/pane + 进程 | 可丢失、重建、接管；不承载 Project/Task/Run 的领域身份 |
| PTY | PTY（伪终端） | 同名同义的基础设施概念 |
| 结构化接入 | ACP（Agent Client Protocol）等代理协议 | HCTL 只把它当作受控端口能力之一；协议会话不是 HCTL 身份 |
| semantic resume | 各 Harness 原生的会话恢复（如 codex resume） | 恢复的是上下文，可能创建新进程；不等于 exact attach |
| replay | 终端录像回放 | 只读历史，不冒充存活会话 |
| Write Lease / Terminal Input Lease | 无对应 | 差异化语义：单写入者与单输入者，配代次失权 |

# 契约测试矩阵

> 状态：验证文档 · 草案 v0.17.0<br>
> 本文列出十一族可观察行为的失败用例，不描述状态机、不新增约束；约束变更须先改 spec 再加用例。

交付测试检查可观察行为，不复述模块状态机。每族一个稳定的族标识符；模块新增约束必须在对应族里增加一个失败用例，而不是再建一份不变量文档。

### `CT-PROJECT` · Project / Room

- 注册确认事务激活 Repo 身份的同时创建唯一 Repo Room，待确认 Repo 不接受 Project/Task/Run；Repo 身份、clone 与 Repo Instance 的用例归 `CT-REPO`
- Project 分组与 Room anchor 可重建
- 有非终态 Run、写入型 Invocation、活动 Write Lease、待投递或结果未知的集成意图/发布评审意图或其他未决外部写副作用时归档 Project 拒绝，阻塞项列表能看见归 Repo 模块的租约与意图；仅开放 Task/Request 或未归档 Scoped Room 随归档转入只读，不被隐式终结
- CJK 输入、结构化引用、草稿/游标/未读、并发流隔离；时间线顺序以 chat server 给出的为准，治理引用只按事件 ID 冻结
- Repo Room 只把显式选中的来源链带入新 Project
- Scoped Room 回填和同根因 Request 去重；未回填且无显式结案（abandoned/no-decision/superseded 及理由）的 Scoped Room 归档拒绝
- Context 可解释、Room 历史可恢复（chat server 重同步 + 治理引用与冻结 digest 完整）
- chat server 不可用时，依赖 Room 来源、身份或 Context 当前回读的预览/命令 fail closed，不依赖这些读数的已接纳治理事实仍可使用
- Room–Server Binding 只接受未启用端到端加密的房间，HCTL 自建房间回读无 `m.room.encryption`；已绑定房间事后被加密与 chat server 不可用走同一条 fail-closed 规则并标为需要关注，换绑到未加密房间后恢复
- chat server 中的普通消息、反应或自动化不能成为命令；绑定未列明、actor 无法映射、source event/target/version 缺失的结构化动作同样拒绝
- 同一个 Matrix 用户动作分别从 Workbench 直连适配器和供应端事件适配器提交时，两条路径必须生成相同的命令摘要和结果。由 HCTL 服务或 bridge bot 发出的同形事件必须拒绝为 human 来源
- 模型 Participant 的 `@`/建议不能创建 Invocation 或 fan-out，human 批准后自动携带来源/Context
- 无法证明身份的 Invocation 撤权并终止，Retry 产生新调用且旧结果被拒绝
- mention 解析无唯一授权候选时明确失败，不按显示名模糊匹配或静默换人
- 原始消息、执行日志和模型总结不经「发布 Memo」命令不会成为 Memo
- 治理引用指向滚动纪要而非精确消息事件时拒绝
- Bundle 压缩条目缺 compressor/原文 digest 记录，或压缩了证据类内容时拒绝交付
- 萃取索引与纪要缓存删除后可完整重建且不丢事实；相关性门判定缺可审计记录时无效，未配置 small-brain 时以消息正文做路由拒绝
- 过期或被取代的 Memo 不进指针清单，显式引用除外
- 压缩片段或纪要条目的回源指针不是组装器赋予（例如由压缩模型输出）时，Bundle 拒绝交付
- Matrix 房间升级换 ID 后，换绑 Room–Server Binding 不改 Room 身份，旧事件引用与冻结 digest 仍可校验
- 参与者授权换人后，活动 Invocation 仍引用准入时的 Project version 与 Participant revision；Task 或卡片上不存在独立的参与者授权
- 评审发布策略随 Execution Spec 冻结：Trigger Preview 未写明授权的是发布不是合入、策略缺仓库/绑定版本/发布目标规则/创建或更新/描述来源任一项时拒绝；「须人显式确认」开关的值随授权冻结，之后改默认值不影响已接受的调用
- 平台评审评论线以精确 ChangeSet Revision 与评论标识冻结进 Context Manifest；评论被当作授权、契约或裁决时拒绝；纯本地仓库的 Manifest 没有这一项

### `CT-TASK` · Task / Kanban

- 外部卡片只改变 stage 或 health 时，Task lifecycle 和当前 Task Revision 必须保持不变；Kanban lane 只重新计算投影
- 非法 move/complete 拒绝
- local state version 与 remote revision 不混用，过期邻项移动重算
- 本地 adoption 不要求伪造 Task–Backend Binding，外部 adoption 混用 Task–Backend Binding 版本时拒绝
- 未采纳契约使 Start/Complete fail-closed，明确 divergence 后新增 drift 仍使旧预览失效
- active Run 尚未结束时 terminal intent 拒绝；活动 Run 期间采纳契约推进 current 后，Run reducer 的「完成 Task」按契约分歧拒绝，不静默完成新 Revision
- Workbench、CLI、已获准的 Vikunja Done 和正常完成 Run reducer 必须生成同一种“完成 Task”命令，并经过相同准入
- Result Proposal 或单独观察到的外部关闭态提交该命令时必须拒绝
- Vikunja Done event 缺 doer 映射、前后变化、remote revision/updated version、当前回读或规范幂等 tuple 时只追加 Snapshot；重复/迟到 webhook 不重复完成
- provider Done 请求遇到无契约、活动 Run、证据不足或新 drift 时保持外部 Done + HCTL 开放并返回类型化结果，不回滚 provider、不伪造 Receipt
- 同一规范实体跨 Project/connection/placement 不得产生第二个 Task，禁用绑定也不释放映射
- content 后端按 Repo 选定，跨后端相对移动拒绝
- Project 分组 anchor 在删除、重绑和重建后保持稳定；原生 UI 把卡片移到另一 Project 分组时，HCTL 必须保留原 `project_id`，把来源归属标为需要关注，并拒绝依赖原归属的新命令
- Project 分组映射（父任务/milestone/标签降级）有测试
- 无契约 Task 的看板终态只是 content 投影，完成命令仍需先升格契约
- 验收项缺校验等级时「采纳契约」预览失效
- `mechanical` 验收项只以转述证据提交，或证据通道低于验收策略要求时，「完成 Task」拒绝；Receipt 逐项记录校验等级与实际判定者
- 契约冻结要求远端检查或远端合入时，本地测试成功或本地 ref 前移不能顶替；集成结果只认 Repo 模块的 Integration Receipt，平台合并状态自身不能完成 Task
- 验收契约未要求代码集成的 Task（例如只交付已登记工件、由人验收）完成时不要求 Integration Receipt；要求集成时只认 Repo 模块的凭证

### `CT-RUN` · Run / Workflow

- Workflow Revision 不通过 schema、Profile 或图结构校验时，编译必须拒绝
- Run 绑定超过一个 Task Revision 时，启动必须拒绝
- 已绑定 workflow engine 执行的状态只能由 control 命令推进
- Dagu UI/API 直接 Start/Stop/Retry/Reschedule/Approve/Reject/Edit/Rename/Delete 时只标记 Run–Engine Binding 分歧，不倒推 Run 命令、Verdict 或 Receipt；停止路径若未先持久化 intent 与撤权则不能冒充 HCTL Cancel 成功
- 超时与候选切换只依据账本自己的 Obligation deadline
- 引擎停报进度（绑定分歧待对账）期间，或依据缓存、迟到事件和旧游标观察创建新 Obligation 时，创建必须拒绝；已经创建的义务照常验收与判决
- 派发确认回执丢失允许待启动→丢失并用新 Attempt 恢复，已交提案不被误当成功
- retry 只产生一个新 Obligation 并隔离旧 Seat/Attempt，候选耗尽和 Request expiry 产生明确的失败类型，所有 Run 过渡态可失败/替代
- dynamic fork 超出冻结 Seat 模板/recipient/基数/预算时拒绝
- placement 变更留下不可变审计
- Gate backup 改变参与者或任一 Context/Skill/policy ref 时拒绝，作者不能占必需 reviewer Seat
- quorum-unreachable 沿冻结失败边推进
- Run 正常完成只由账本谓词决定；引擎报告的进度与账本不一致时标为分歧待对账，既不补足也不阻止谓词
- 失败/已取消/被替代 Run 不终结 Task，quorum/regate 和迟到结果拒绝
- 启动中 / 暂停中 / 取消中到达默认或声明的超时后进入需要关注并保留状态，不自动取消或替代
- 节点声明的外部机械事实前置：工具箱读不到时不派发并标需要关注；执行体或模型转述的事实不满足前置；前置不创建 Obligation、不占席位
- 返工达到声明的轮数上限后按 quorum-unreachable 同路推进或创建 Request，不进入下一轮
- 未声明增量评审时新 Revision 全量重评；声明时评审包带与上一版的差异指针
- `changes_requested` 分歧落点为契约时不进入语义返工、不自动替代，Task 标需要关注并给出采纳建议
- Artifact Revision 作评审对象仍成立，不经 Repo 模块转交
- 同一结果树、不同基线或不同版本身份不能复用旧票；同一获准版本只换提交包装时评审身份不变
- 同一平台账号的多个批准不增加内部席位票数；有效席位的 Result Proposal 仍可计票；接受外部评审的无 Run 契约不被强行送去 Gate

### `CT-PARTICIPANT` · Participant / Terminal

- 未探测到声明能力时，绑定必须拒绝或降级
- Agency 申报的 Skill 引用与 digest 无法由工具箱回读核验时，Execution Spec 只能记 unknown，不得记 known；required Skill 缺失时解析失败
- 本地 Agency 参考实现申报的 Skill digest 与工具箱回读不一致时不激活，拒绝结果列出不一致项
- Revision 或 digest 不匹配的 Result Proposal 必须拒绝
- 旧 `runtime_generation` 的输入和结果必须拒绝
- 旧 writer 既不能证明已停止、也不能证明被限制在旧工作树与旧 ChangeSet 边界内时，本模块不得声称已隔离，Repo 模块因此不重授租约；能证明其一即算隔离，后续执行用新 ChangeSet、原 ChangeSet 不重授；写租约与保全的用例归 `CT-REPO`
- 冲突观测按来源证据仲裁
- Execution Chat 的错误归属者/代次输入和无 provenance Share 均拒绝
- 治理命令只有两类 actor 来源：映射到有权用户本人的 direct client/provider event 与 task-bound Run 正常完成的 reducer；Workbench、CLI 与 provider adapter 产生相同 command envelope，Result Proposal 通道提交不了治理命令
- 每个 Worker Profile：Harness 环境与进程取不到 HCTL 交付的 control/人类 credential 与集成/平台写凭据，凭据只由工具箱/平台适配器网关代用；Harness 在 worktree 内可读 common-dir/refs 并在本 ChangeSet 分支提交，但推不了远端；绕过「合入 ChangeSet」命令改写目标 ref 不产生 Integration Receipt，下一次 integration preview 显示 drift（预期目标头形态下是预期目标头不匹配，接受目标前移形态下是回读核对不符）
- 声明了执行加固的 Worker Profile：所声明项按声明生效并与 Execution Runtime 记录一致；已声明而宿主不支持时不激活，拒绝结果列出缺项；未声明时照常启动、不记录为已生效
- 人直接修改 Herdr workspace/pane 归属或已冻结派工结果只形成 drift，不能冒充结果；对精确 terminal 的输入则按 Execution Spec 输入策略处理
- `native_interactive_allowed` 下原生 TUI/Workbench 直连输入是有效运行时输入，该输入不能直接产生领域结果；Agency 未声明逐次输入记录能力时，还必须标明逐次 provenance、generation 和物理单写者保证不完整
- `managed_single_writer` 下不得同时开放 Herdr API 写入与原生 controller 写入，尝试原生写入时执行不得继续声称策略成立
- Agency 未声明事件游标能力时，不得把事件流当作完整持久 trace；重连后只能按可证明范围恢复观察
- Agency 未声明退出与停止回读能力，或不能证明同一进程和 PTY 仍存活时，不得声称 exact attach；缺失 exit/stop 回执的执行不得报告为成功停止
- Agency 状态检测以低层来源覆盖仍有效的结构化 hook 证据时拒绝；Agency 恢复报告无法翻译为[恢复等级](./participant.md#terminal-场景)时按丢失处理
- Agency 自带的接管/单写者/"会话有效"记录被当作账本事实或替代租约/代次时拒绝
- 未声明栅栏回显的 Agency 通道未按实际能力降级（原生输入仍宣称逐次受租约管理，或结果按高证据类准入）时拒绝；已声明栅栏回显的 Agency 放行不匹配代次时该绑定标记失信并需要关注
- control 签发连接票据、Herdr 适配代码校验 HCTL 授权，观察、输入、Attempt 控制与安全输入权限分离；Agency 未声明栅栏回显能力时，无法执行的代次栅栏不得被记录为已生效
- attach 只接通道，不能恢复 Run/Invocation 语义
- attach/replay、IME/背压/慢客户端隔离
- 未标注证据通道的 Evidence 按转述处理；转述不能通过重新标注升级为高证据类
- Participant 换绑到另一个 Agency 后身份不变，活动执行仍引用原 Participant–Agency Binding 版本

### `CT-REPO` · Repo / Change

- 首次注册生成稳定 Repo 身份，同一 Repo 的新 clone 只新增 Repo Instance，fork/身份碰撞明确拒绝或要求确认；相同 Git 公共目录重试返回原现场，移除现场不删除 Repo、Project、历史 Run 或已封存 ChangeSet
- 第二个写入者请求同一 ChangeSet 的活动租约时必须拒绝；新旧租约交接时 Repo 模块撤权、Participant 隔离、确认前不重授
- 无法证明旧 writer 已被隔离时默认保全并隔离旧 Git 工作树，不自动重授租约；接管、采用或丢弃缺少有权 human 显式确认时拒绝，失败清理不丢唯一未封存/未跟踪修改
- 未经 Project 或 Run 准入的提案不产生获准 ChangeSet Revision；Git 封存完成但归属者在准入前被取消或替代时，不产生可供下游消费的版本，也不触发发布评审；平台上出现的提交不是准入
- 同一获准版本只换提交包装（内容与基线相同）时 review_subject_digest 不变；基线或结果树变化时是新 Revision，旧 Verdict 失效；结果树相同不是充分条件
- 本地/远端集成都先持久 integration intent，由工具箱/平台适配器执行、工具箱 readback；预期目标头形态下的目标头竞争、确认回执状态未知、换绑或远端合并回执丢失时不得签成功 Integration Receipt，同一意图重试只得同一结果
- 源版本不变、其余前置满足、目标从 A 前移到 B：预期目标头形态拒绝；事前已选接受目标前移形态则成功，Receipt 记实际目标头 B；保护规则变化另按快照判定，不能替代形态判定
- 同一 target ref 上已有待决集成意图（任一形态）时，提交另一意图（不论形态）拒绝；前一意图终态后，同一目标的下一意图是新的授权并可成功
- 有权 human actor 预览残留后的显式封存不经 Invocation/Attempt，由 Repo 模块按该命令准入为 ChangeSet Revision；封存意图重试同一关联键返回同一结果；工具箱已写出但未准入的树或提交不是获准版本
- 关掉客户端后，获准结果仍按已冻结的评审发布策略发布，actor 信封沿用授权它的那次 human 提交；开关打开的仓库改为待处理、由人预览后提交
- 「已开启自动合并」「已进合并队列」「请求已接受」都不算已合入，Receipt 只在回读到合并提交与目标头后签发；Receipt 记实际目标头
- 绑定声明「不能保证预期目标头」而 actor 未显式选择「接受目标前移」形态时，集成意图拒绝；执行时不得由回读结果倒推放行
- 目标开了「要求与目标同步」，在预览与执行之间变化时仍按冻结的目标保护快照接受或拒绝，不凭「同步开关为真」或「几个检查为绿」通过；保护条件与快照不一致时拒绝或标需要关注
- 本地目标 ref 正被某个工作树检出时默认拒绝，返回精确工作树路径与「切离后重试」的恢复动作；人切离后重试同一意图成立；显式放行只推进 ref、不更新该工作树
- PR 被改投另一目标分支、关闭或重开时只更新映射状态并标需要关注，不对错误目标写 Receipt
- 推送成功但创建 PR 的确认丢失、更新 PR 后确认丢失时，按原意图与关联键回读、不重复创建；旧 Revision 的迟到重试不能覆盖新 Revision 的分支或映射
- 提前授权的发布评审只发布本次已获准产出；换仓库、换绑定、换发布地点或扩大发布范围须重新授权；Result Proposal 换发布地点时拒绝
- 平台批准、控制面自己写回的批准、重复事件、无法映射的动作各走正确的来源规则：批准只是外部评审证据，自写回事件排除，重复/迟到/乱序得到相同结果，无法映射只形成 Snapshot；「已合并」通知无匹配意图时只作外部事实与分歧，不倒补意图、不补签 Receipt
- 执行体拿不到远端写凭据；远端推送、PR 与合并只由平台适配器执行
- 契约冻结要求远端检查时本地成功不能顶替；契约要求合入远端 ref 时本地 ref 前移不能顶替；能力缺失使命令等待、拒绝或标需要关注，不改契约
- 没有平台绑定的 Repo 走完整路径：机械项只以本地事实为证据，集成只面向本地目标；之后绑定平台不改 Repo 身份、既有 Revision 与 Receipt，进行中的意图沿用原绑定版本
- 结果未知按目标分：本地意图只回读本地事实、不等 PR；远端意图确定未投递的可重投，可能已写的保持未知并继续占用冲突范围，本地已有同一结果树也不解锁
- 平台失联时本地物化、封存与面向本地目标的集成继续；发布评审、读 PR 状态与远端合入拒绝；平台丢失后不凭 Git 提交重建 Receipt

### `CT-CONNECTION` · 连接 / 端口

- 每条模块连接都必须固定来源引用、摘要和唯一绑定；缺任一项时目标准入必须拒绝
- Run/Invocation 冻结精确 Context Manifest ref+digest，每个实际 consumer 冻结对应 Context Bundle ref+digest，权限过滤、来源版本或预算变化使旧预览失效
- client/port 连接与绑定分离；同一产品同时作客户端与 provider 时不能借一侧身份写另一侧事实
- actor provenance 不能由 payload 自报
- dispatch/result 迟到拒绝
- 外部副作用的确认回执未知时不重复执行，adapter 也不写 Receipt
- provider 离线时，不要求当前回读的查询/命令可继续，要求当前 head/revision/租约/回读的准入统一 fail closed
- Harness 绕过受控端口的 API 写能力被拒绝，带外 drift 只形成 Snapshot/观测而不是结果
- Dagu、Vikunja、Herdr 的私有对象 ID 或状态被提升为 HCTL 稳定身份、权限或完成判定时拒绝
- 新 provider/adapter 未通过对应模块契约测试时不得产生 Port–Provider Binding；换绑不能改写活动 Run、Task、Room、Execution Runtime 或集成意图的冻结绑定
- Project 或 Run 准入提案与 Repo 模块准入 ChangeSet Revision 在同一账本事务；工具箱封存回读先于准入，缺任一步不产生获准版本
- Execution Spec 的评审发布策略作为字段冻结；Result Proposal 提供策略之外的发布地点或内容时拒绝
- Repo 模块不接收 Result Proposal；执行体直接向 Repo 模块提交版本或集成命令时拒绝
- Attempt 归属的版本按 Run Manifest 冻结进 Execution Spec 的评审发布策略发布，发布 outbox 挂在 Run 准入提案的事务上；席位不能自行推送远端
- 既有 content 迁移必须显式预览、导出、导入并回读校验；普通换绑不得冒充无损迁移或热切换
- 客户端无等级：Workbench 通过 provider 通道执行的消息/卡片/终端动作与原生客户端同语义，通过 command service 的动作与 CLI 同语义；Workbench 不得依赖 provider 私有导航或对象模型获得隐藏权限
- provider event 只有模块绑定明确列出且 actor/target/version/idempotency/freshness 齐全时可成为 human command request；否则只能成为 content/Snapshot/运行时观测
- 飞书、Slack、Discord 等 Chat 互通只经 Matrix bridge 接入；HCTL 只校验 Matrix 事件与桥接身份映射，不注册逐平台 Chat adapter

### `CT-SYSTEM` · 系统

- 同一用户级账本只能有一个 control writer，第二 writer 拒绝
- 多个执行现场可以登记（各有工具箱与 Herdr 绑定），但同一现场/仓库修改租约的旧代次必须被代次栅栏隔离。无法证明隔离已生效时默认不重授写权限，重授只能来自有权 human 预览证据后的显式确认
- 命令幂等
- 危险动作（不可逆、产生外部权威副作用或扩大权限）未经确认的直接 Submit 拒绝；普通命令直接 Submit 与经 Preview 提交结果一致
- executor 越界拒绝：远端推送、PR 与合并只归 Repo 模块的平台适配器，lint/检查不经工具箱 intent 回路；工具箱只受理现场 Git 职责内的已持久化意图，不执行远端副作用
- 同一 human action 经 Workbench、CLI 或 provider adapter 进入时使用同一准入规则；重复、迟到和乱序 provider event 不产生第二份领域效果
- commit/确认回执各崩溃点回读
- schema migration、投影重建
- metadata 账本执行一致性 backup、restore preview/apply、writer generation 重置与恢复后 content readback，每个首次消费的 content 服务器执行备份与恢复
- content 服务器宕机不抹掉已接纳事实，但依赖 provider 当前回读的命令 fail closed
- 从 Git 结晶回灌不得伪造未结晶判决
- clone 本地运行目录（锁与缓存）删除后可完整对账重建、不丢事实
- 一键启停下已消费服务器的启动顺序与健康检查
- 旧 generation 与越权适配器拒绝
- 等价对象的 JCS 规范摘要一致、内容篡改被 digest 校验拒绝
- 打包后的整窗启动/退出/升级和安全边界

### `CT-PACKAGING` · 扩展 / 打包

- 自声明 trust、有副作用的 discovery、未经用户配置的联网探测、静默 install/upgrade、非本地未认证 Dagu、renderer Node/raw IPC/远程脚本或不满足下述源码合规门禁时均拒绝
- 未声明的 capability/permission/scope、未声明的 IPC 或插件能力、远程运行时脚本/CDN 时拒绝；Electron 安全网形态下 renderer Node/raw IPC 同样拒绝

### `CT-WORKBENCH-IA` · Workbench 信息架构

- 单 Project Overview 与全局「需要关注」都是可重建的只读导航投影，不产生新场景或写状态；Change 场景是 Repo 模块的场景，Overview 只投影它
- 打开入口按 repo 选择并统一映射到控制面连接（打开本地 repo = 连接或拉起本机控制面再定位仓库；远程入口隐藏或安全拒绝）
- 进入 Project 默认打开 Project Room，deep link 保留返回路径
- 同一 Request ID 跨 Room/Task/Run 聚合且不能从聚合面直接改状态
- 「创建 Project」命令提升预览允许删减、补充、去敏并显示来源回链
- Trigger Preview 展示实际执行者、Context/Skill、权限、预算和 fan-out
- Workbench 不运行时，CLI、Matrix/Vikunja/Herdr 原生客户端、GitHub 页面与 `gh`、control 和已启动 Run 仍可独立工作，Change 场景对应的命令仍可达；安装 Workbench 不得增加公共命令服务尚未提供的 HCTL 命令

### `CT-WORKBENCH-INPUT` · Workbench 输入与无障碍

- Board 移动、Request 操作和 Run 浏览在 mouse/touch/keyboard/screen reader 下等价
- 输入优先级为 IME composition → 已聚焦 terminal → modal/composer → 当前场景 → 全局快捷键，任何上层快捷键都不能截获正在组合或发往 terminal 的输入

### `CT-PRODUCT` · 产品

- 用户十秒内能回答 Project 目标、Task 状态、Run 阻塞、所需动作、当前 Harness 和证据版本
- 正常成功保持安静
- HCTL2 仓库自举不使用隐藏的特例豁免或产品外补签事实
- 无 Run 路径、有契约的 Task、默认发布策略：从 ChangeSet 封存到完成凭证，人的预览不超过两次（合入、完成）；无契约卡只多一步采纳。发布评审的授权在封存之前的 Trigger Preview，不计入；显式确认开关的例外见下条
- 两条 B2 成功路径都成立：没有平台的普通仓库，和受保护的 GitHub `main`；默认路径都是两次预览，不借 Gate 或特例权限过关
- 开了「发布评审须人显式确认」开关的仓库单独验收它的三次预览，不拿它给没开开关的仓库过关
- 纯本地路径覆盖目标正被检出（人切离后重试）与确认丢失后的恢复

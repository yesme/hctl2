# K3 · 机械清单

> 状态：已落地（待语言/结构修订消化）<br>
> 基线：main @ `2863632`（草案 v0.15.4）<br>
> 去向：清单由本轮语言通读（`11-gpt-language.md`）与结构审计（`10-grok-structure.md`）消化，随对应修订 PR 核销

任务书（`03-briefs.md`）「K3 · 机械清单」一节的产出。全部是机械扫描：只报不改，不编辑任何 `docs/` 文件；每一项给出 `文件:行`，可复算。

## 范围与口径

- **范围**：任务书划定的 21 个文件——根 `README.md`、`docs/usage.md`、`docs/design/`（`vision`、`architecture`、`README`、`project`、`task`、`run`、`agent`、`participant`、`context`、`references/glossary`）、`docs/design/spec/`（`README`、`system`、`connections`、`project`、`task`、`run`、`agent`）、`docs/design/delivery.md`、`docs/design/contract-tests.md`。
- **基线**：扫描在 main @ `2863632` 的工作树上执行；已核实该基线与任务书基线 `40cfe3d` 在 21 个范围内文件上零差异（两基点之间只动了 `docs/research/`）。
- **两个统一处理**：代码围栏（``` 包围的块）内部一律不扫；其余各节按各自声明的口径分「原文行」（含行内代码与链接，grep 可复算）与「散文」（剔除行内代码与链接 URL，贴近阅读面）两种。
- **职责边界**：本清单只做提取与计数，不做「该不该改」的裁决；凡涉及判断处（如「需要」的两可标注）均给出分类依据，最终处置归语言通读与结构审计。

## 扫描 1 · 规范性关键词全量表

口径：21 个文件的原文行（含链接锚点，剔除代码围栏内部），可用 `grep -o "<词>" <21 文件> | wc -l` 复算。整句按标点（。！？；|）截取所在分句；同行多次命中逐次列行。

| 关键词 | 命中数 |
| --- | --- |
| 必须 | 125 |
| 不得 | 66 |
| 禁止 | 1 |
| 只能 | 46 |
| 可以 | 122 |
| 需要 | 80 |
| 应当 | 0 |
| 不应 | 1 |
| **合计** | **441** |

计数核实：

- 任务书称「需要」82 处；本基线（main @ `2863632`，与任务书基线 `40cfe3d` 在 21 个范围内文件上零差异）实测 **80** 处，`git grep -o "需要" 40cfe3d -- <21 文件>` 同为 80。以实测为准，80 处全部标注。
- 「应当」全库 **0** 处；「不应」**1** 处（`docs/usage.md:16`，下一字为「作」，非「不应用/不应对/不响应」类词内命中）。任务书提示的「相应/对应/供应/应用/响应」词内混入在本基线不发生（两词检索均不产生词内命中），剔除规则已备而无需触发。

- 附带发现（任务书关键词之外）：「应该」3 处——`docs/design/vision.md:80`、`docs/design/agent.md:9`、`docs/design/context.md:104`，均在设计层散文，无约束层命中。T1 三档只有「应当/不应」，「应该」是同档的口语变体，是否收编留给语言通读。

### 1.1 「需要」逐处标注（80 处）

标注分三类：

- **客观必要**：描述性必要条件（含「需要…时」条件从句与「需要的 X」定语），不是义务句。
- **规范应当**：实质是义务（按 T1 应改用三档关键词或改写），本轮语言通读的重点输入。
- **专名/锚点**：状态名「需要关注」、标题或链接锚点中的命中，不是情态动词。

汇总：客观必要 **50** 处，规范应当 **3** 处，专名/锚点 **27** 处。规范应当 3 处为：`docs/design/architecture.md:49`、`docs/design/spec/system.md:55`、`docs/design/spec/connections.md:145`。

| # | 文件:行 | 标注 | 整句 |
| --- | --- | --- | --- |
| 1 | README.md:17 | 客观必要 | ## 为什么需要它 |
| 2 | README.md:19 | 客观必要 | 完整的问题与失败模式见[为什么需要 HCTL2](./docs/design/vision.md#为什么需要-hctl2) |
| 3 | README.md:19 | 专名/锚点 | 完整的问题与失败模式见[为什么需要 HCTL2](./docs/design/vision.md#为什么需要-hctl2) |
| 4 | docs/usage.md:24 | 客观必要 | 是否需要安装 |
| 5 | docs/usage.md:29 | 客观必要 | 源码包必须和运行包保存在同一 Release 下载位置，但普通用户安装和运行 HCTL2 时不需要下载它 |
| 6 | docs/design/vision.md:15 | 客观必要 | ## 为什么需要 HCTL2 |
| 7 | docs/design/vision.md:74 | 客观必要 | 它不需要 Run |
| 8 | docs/design/vision.md:75 | 客观必要 | 5. 需要持久自动施工时，用户先批准 Workflow（施工图），再显式启动 Run，授予有边界的自主权 |
| 9 | docs/design/vision.md:76 | 客观必要 | 需要澄清、决定或授权时，系统创建 Request（请求卡）并投影回 Project |
| 10 | docs/design/vision.md:77 | 客观必要 | 7. 只有需要观察或接管某次精确执行（Attempt）时，用户才打开结构化执行投影或终端 |
| 11 | docs/design/vision.md:85 | 客观必要 | - 当前需要谁提供什么 |
| 12 | docs/design/vision.md:89 | 客观必要 | 由此形成注意力分配原则：多个 Harness 默认在后台执行，前台只保留需要人商议、授权或接管的事项 |
| 13 | docs/design/vision.md:143 | 客观必要 | Obligation、Seat、Attempt 只在需要时渐进展开 |
| 14 | docs/design/architecture.md:49 | 规范应当 | 既有 content 能否迁移取决于两端导入导出能力，需要单独预览和校验 |
| 15 | docs/design/architecture.md:90 | 客观必要 | 后端离线不等于全部治理命令不可用，也不等于全部治理命令照常可用：是否继续由该命令的准入约束是否需要 fresh provider readback 决定 |
| 16 | docs/design/architecture.md:90 | 专名/锚点 | 受影响的入口显示待处理 / 需要关注或安全暂停，不绕过命令服务 |
| 17 | docs/design/architecture.md:95 | 客观必要 | 需要核对 provider 当前事实或重建来源链的命令仍须 fail closed，旧结晶不充当 fresh readback |
| 18 | docs/design/README.md:35 | 客观必要 | Room 不拥有 Workflow token 或运行时，Run 也不需要自己的 Room |
| 19 | docs/design/README.md:78 | 客观必要 | 若同一规则需要在多个权威位置同步修改，先选定唯一 owner，再把其他位置改为引用 |
| 20 | docs/design/project.md:23 | 客观必要 | 应答面按需升级——默认在卡片或详情里回答，需要多轮论述、多人参与或共同编辑才开临时讨论空间，敏感输入走安全通道，只有诊断或接管才连接终端 |
| 21 | docs/design/project.md:27 | 客观必要 | 需要持久重试、候选切换或评审关卡时应创建 [Run](./run.md) |
| 22 | docs/design/project.md:48 | 专名/锚点 | - Request、Project 概览、Task/Run 里程碑和需要关注投影 |
| 23 | docs/design/task.md:21 | 专名/锚点 | 泳道由本地阶段、生命周期与外部来源投影共同派生，阻塞与需要关注是正交的健康标注 |
| 24 | docs/design/task.md:41 | 客观必要 | 简单工作不需要先画 Workflow，也不伪造只能由 Run 产生的 Gate（评审关卡）凭证 |
| 25 | docs/design/task.md:41 | 客观必要 | 需要持久重试、候选切换或 Gate 时，Task 才显式授权 Run |
| 26 | docs/design/task.md:45 | 专名/锚点 | 卡片显示契约版本、来源、排序、负责人、阻塞、需要关注、活跃 Run、Request（请求卡）、Artifact/PR/CI、外部生命周期和 HCTL 验证状态 |
| 27 | docs/design/task.md:59 | 客观必要 | 需要临场选择的采纳、冲突解决和完成预览仍回到公共命令客户端，不能由 adapter 代替人猜 |
| 28 | docs/design/agent.md:9 | 客观必要 | 正常路径上用户不需要进入终端：状态、diff、证据和 Request（请求卡）应该先把事情说清楚（默认无界面，headless by default） |
| 29 | docs/design/agent.md:26 | 客观必要 | - 执行体只拿到本次执行需要的窄能力 |
| 30 | docs/design/context.md:11 | 客观必要 | 仓库代码、Memo、Skill 这些材料，执行体自己会挖、自己管理——读文件、载技能本来就是它的本职，它比我们更清楚自己需要什么 |
| 31 | docs/design/context.md:52 | 客观必要 | 绑定 Task 的评论线整条结构相关，在这一级就全部命中，以快照 ref + digest 冻结，不需要检索 |
| 32 | docs/design/context.md:78 | 客观必要 | 父转子不需要新机制：父房间的纪要（若有）作为提升/创建预览的预填材料，人删减、补充、去敏并确认后冻结为子房间的出生来源链 |
| 33 | docs/design/spec/README.md:14 | 客观必要 | 某个步骤的产物：只追加、短期或一次性，但需要被精确引用 |
| 34 | docs/design/spec/README.md:18 | 客观必要 | 约束需要逐字指认的协议或 schema 字段、序列化格式标识，以及外部标准、产品或源码中的原名，可以保留原形并用代码格式标示 |
| 35 | docs/design/spec/README.md:24 | 客观必要 | 散文中的 AI 协作者用 Participant 表述，需要区分人与模型时加“模型”限定词 |
| 36 | docs/design/spec/system.md:34 | 客观必要 | 活动工作仍使用原 binding，已有 content 的迁移是另一个需要预览、导出、导入和回读校验的显式动作 |
| 37 | docs/design/spec/system.md:36 | 客观必要 | 需要 HCTL 先记账、撤权或核验前置的 provider mutation 仍只能由 control 经对应受控端口发起 |
| 38 | docs/design/spec/system.md:55 | 规范应当 | 不可信扩展需要操作系统强制隔离和能力削减的代理接口 |
| 39 | docs/design/spec/system.md:70 | 客观必要 | 需要提交 HCTL 命令却无法提供等价预览、版本或权限信息时，动作必须禁用、保留为待处理请求或安全拒绝 |
| 40 | docs/design/spec/system.md:79 | 客观必要 | 需要临场选择、危险动作未经确认或 binding 未允许该来源自动提交时，保留为待处理或返回类型化拒绝 |
| 41 | docs/design/spec/system.md:129 | 客观必要 | 控制面凭获准命令、精确映射与 Snapshot 对账需要治理的那部分外部关系 |
| 42 | docs/design/spec/system.md:154 | 专名/锚点 | 每类事实的不可用与永久丢失分开立约：不可用走降级约束（待处理 / 需要关注 / 安全暂停，不绕过命令服务），永久丢失走重建约束 |
| 43 | docs/design/spec/system.md:159 | 客观必要 | 需要新正文或 Git 回读的命令安全暂停 |
| 44 | docs/design/spec/system.md:160 | 专名/锚点 | 聊天入口降级（不可用显示重同步中，房间事后被加密显示需要关注） |
| 45 | docs/design/spec/connections.md:8 | 客观必要 | 连接不是一份可独立漂移的共享状态，也不需要 `Handoff` 聚合 |
| 46 | docs/design/spec/connections.md:119 | 专名/锚点 | Task 拒绝自动命令时 Run 保持完成，Task 保持开放并显示需要关注 |
| 47 | docs/design/spec/connections.md:127 | 客观必要 | 对需要恢复执行的 Request，control 在同一用户级 metadata 账本事务 CAS Project Request 与来源 blocker 的精确版本，并提交解决结果及唯一 signal/delivery outbox |
| 48 | docs/design/spec/connections.md:145 | 规范应当 | 范围、权限、候选或验收含义变化需要显式替代，而不是原地修补 |
| 49 | docs/design/spec/connections.md:147 | 客观必要 | 需要扩权时回到拥有该权限的上游重新预览和授权 |
| 50 | docs/design/spec/connections.md:157 | 专名/锚点 | 连接保持待启动/需要关注，来源不会被伪装成已交接 |
| 51 | docs/design/spec/connections.md:160 | 客观必要 | 需要 fresh chat readback 的准入拒绝，聊天入口显示重同步中 |
| 52 | docs/design/spec/connections.md:161 | 专名/锚点 | 聊天入口显示需要关注，已冻结引用与 digest 不受影响 |
| 53 | docs/design/spec/connections.md:162 | 客观必要 | 需要 placement/drift/head/cursor 的 Create/Adopt/Start/Complete/Move 拒绝，看板不显示假成功 |
| 54 | docs/design/spec/connections.md:165 | 专名/锚点 | 连接显示待启动/需要关注或安全暂停 |
| 55 | docs/design/spec/connections.md:168 | 客观必要 | 连接需要的新尝试或替代执行必须拥有新的 owner version/generation、Execution Spec 与 runtime generation |
| 56 | docs/design/spec/project.md:40 | 专名/锚点 | Workbench 可以另行把同源 Request/health 投影聚合为全局需要关注 |
| 57 | docs/design/spec/project.md:56 | 客观必要 | 需要 fresh message body、成员身份或完整 cursor 才能准入的命令类型化拒绝，聊天入口分别显示重同步中或需要关注，不能用缓存冒充当前事实 |
| 58 | docs/design/spec/project.md:56 | 专名/锚点 | 需要 fresh message body、成员身份或完整 cursor 才能准入的命令类型化拒绝，聊天入口分别显示重同步中或需要关注，不能用缓存冒充当前事实 |
| 59 | docs/design/spec/project.md:77 | 客观必要 | 需要这些能力时应创建 [Run](./run.md) |
| 60 | docs/design/spec/project.md:85 | 客观必要 | 当执行需要输入时，拥有该阻塞事实的模块向 Project 提交类型化 Request 创建命令 |
| 61 | docs/design/spec/project.md:89 | 客观必要 | 需要多轮论述、多位 Participant 或共同编辑时才升级为 Scoped Room |
| 62 | docs/design/spec/task.md:20 | 专名/锚点 | Blocked 与需要关注是从 blocker、Request、Run、来源同步和验证事实派生的正交 health，不能覆盖 stage 或成为另一条 lifecycle |
| 63 | docs/design/spec/task.md:28 | 专名/锚点 | 未分组、同时落入多个 Project group 或 anchor 不可稳定回读的卡片只形成未认领 Snapshot/需要关注，不得猜 Project 或先造 Task |
| 64 | docs/design/spec/task.md:30 | 专名/锚点 | Run 正常完成路径只针对其冻结的 Revision：current 已前移时，Run reducer 的「完成 Task」按契约分歧拒绝，Task 保持开放并显示需要关注，不得静默按新 Revision 完成 |
| 65 | docs/design/spec/task.md:36 | 专名/锚点 | 外部卡随后漂移到另一 Project group、同时出现在多个 group 或脱离原 group，只追加 Snapshot 并把原 Task 标为需要关注 |
| 66 | docs/design/spec/task.md:36 | 客观必要 | 需要改变 Project 时，显式取消/保留旧 Task 并在目标 Project 创建新 Task，用来源引用连接历史 |
| 67 | docs/design/spec/task.md:46 | 专名/锚点 | 非后端的关联来源只形成快照、提案或需要关注 |
| 68 | docs/design/spec/task.md:68 | 专名/锚点 | Run 已完成而 Task 校验失败时，Run 保持完成，Task 保持开放并显示需要关注 |
| 69 | docs/design/spec/task.md:70 | 客观必要 | Receipt、生命周期事件、current 投影、匹配的 `completion_pending` claim 清除与需要的外部写回 outbox 在同一事务提交 |
| 70 | docs/design/spec/task.md:70 | 专名/锚点 | Run 路径若被 Task 拒绝，也在持久化拒绝结果与需要关注时清除同一 claim |
| 71 | docs/design/spec/task.md:70 | 专名/锚点 | 外部写回失败只显示需要关注，不撤销已经成立的 HCTL 完成事实 |
| 72 | docs/design/spec/run.md:37 | 专名/锚点 | 任一项未知都只能保持运行/暂停/需要关注或走类型化失败、取消、替代，Engine 检查点结束、进程退出、Harness/LLM 自述和单个 Proposal 都不能补足谓词 |
| 73 | docs/design/spec/run.md:39 | 专名/锚点 | 不能证明旧执行被限制在该隔离边界内时，Run 保持取消中/需要关注且 claim 不释放，不能以“失败了”为由并发启动第二个 writer |
| 74 | docs/design/spec/run.md:85 | 客观必要 | Run 需要输入时向 Project 提交类型化 [Request](./project.md) 创建命令，只阻塞声明的范围 |
| 75 | docs/design/spec/run.md:97 | 客观必要 | 候选耗尽后，需要额外输入或授权则创建 Request，否则把 Seat/Obligation 标为类型化技术失败，不能无限等待或伪装成语义驳回 |
| 76 | docs/design/spec/run.md:105 | 专名/锚点 | 它们只形成需要关注/历史，不提交完成或取消 Task |
| 77 | docs/design/delivery.md:77 | 客观必要 | 4. 需要输入时创建 Project Request |
| 78 | docs/design/contract-tests.md:18 | 专名/锚点 | 已绑定房间事后被加密与 chat server 不可用走同一条 fail-closed 规则并标为需要关注，换绑到未加密房间后恢复 |
| 79 | docs/design/contract-tests.md:79 | 专名/锚点 | 已声明栅栏回显的 Agency 放行不匹配代次时该绑定标记失信并需要关注 |
| 80 | docs/design/contract-tests.md:127 | 专名/锚点 | - 单 Project Overview 与全局「需要关注」都是可重建的只读导航投影，不产生第五场景或写状态 |

### 1.2 「必须」全量（125 处）

| # | 文件:行 | 整句 |
| --- | --- | --- |
| 1 | docs/usage.md:29 | 源码包必须和运行包保存在同一 Release 下载位置，但普通用户安装和运行 HCTL2 时不需要下载它 |
| 2 | docs/design/vision.md:62 | 两点必须同时成立： |
| 3 | docs/design/vision.md:65 | 上端的事实必须在工具更换后继续存在 |
| 4 | docs/design/vision.md:112 | 即使把全部界面、聊天平台、Task 来源、工作流引擎和终端客户端都换掉，下列最小内核也必须保留： |
| 5 | docs/design/vision.md:114 | 必须保持的性质 |
| 6 | docs/design/vision.md:132 | 架构最小内核**回答换掉一切之后什么必须仍然成立 |
| 7 | docs/design/vision.md:146 | 5. **机械动作必须确定性 |
| 8 | docs/design/vision.md:148 | 7. **Context 必须可解释 |
| 9 | docs/design/vision.md:162 | 不让每项工作都必须聊天，也不让每项工作都必须先画一张施工图 |
| 10 | docs/design/vision.md:162 | 不让每项工作都必须聊天，也不让每项工作都必须先画一张施工图 |
| 11 | docs/design/architecture.md:94 | - **metadata**：控制面账本是唯一不可再生的权威，必须有备份 |
| 12 | docs/design/README.md:72 | 中文语境不常用的音译行话必须翻译 |
| 13 | docs/design/README.md:76 | - 新持久对象必须对应第一阶段中的稳定引用、命令目标或恢复边界 |
| 14 | docs/design/README.md:78 | - 新持久对象必须说明现有命令、引用或事件为什么无法承载该边界 |
| 15 | docs/design/project.md:7 | Project 的目标、论证、Participant（参与者）关系、来源和未决问题却必须继续存在 |
| 16 | docs/design/project.md:9 | 以 Room 为中心不等于所有工作都必须聊天：Kanban、Workflow 图和 Terminal 各有自己的场景界面 |
| 17 | docs/design/project.md:39 | 临时讨论空间必须先说清目标与完成后回填什么 |
| 18 | docs/design/project.md:54 | 将来若 Matrix widget/AppService 能提交显式结构化动作，它也必须归一到同一 Preview/Submit 约束 |
| 19 | docs/design/project.md:56 | 普通 Room 里的临场执行边只能来自可稳定归属到 human 的动作，并且必须先经过 Trigger Preview |
| 20 | docs/design/project.md:73 | 持久自动施工必须显式创建 [Run](./run.md) |
| 21 | docs/design/task.md:39 | 这条路径的产品标尺是：走完它，必须比直接开一个终端、用单个 Harness 干完这件事更轻 |
| 22 | docs/design/task.md:39 | 若简单工作必须先画一张施工图，那是产品设计的失败，不是用户的失败 |
| 23 | docs/design/task.md:41 | 契约要求 HCTL 内部独立评审时必须使用带 Gate 的 [Run](./run.md)，接受外部 SCM 评审时则引用可回读的精确外部证据 |
| 24 | docs/design/task.md:41 | 开工前必须把影响契约的外部变更摆到桌面上，明确采纳、拒绝或延期，不能静默越过——细则见[约束附录](./spec/task.md) |
| 25 | docs/design/task.md:51 | Board 不说谎：外部写回在确认前显示待同步，排队中的操作不显示假成功，乐观更新必须可回滚并始终标注不确定状态 |
| 26 | docs/design/run.md:22 | - **交付义务**：Obligation（交付义务）是一个外部节点必须产出的逻辑结果 |
| 27 | docs/design/run.md:36 | - 评审必须独立：被评对象的作者不占必需评审席位 |
| 28 | docs/design/agent.md:20 | - 清理任何资源之前，必须确认没有唯一副本会随之消失 |
| 29 | docs/design/participant.md:39 | 评审是第一个必须专业化的岗位 |
| 30 | docs/design/context.md:32 | 任何索引都必须能从来源和不可变引用重建 |
| 31 | docs/design/context.md:50 | 这一步必须快，缺省全本地、不花模型 token，走三级阶梯： |
| 32 | docs/design/references/glossary.md:27 | 一个外部节点必须产出的逻辑结果 |
| 33 | docs/design/spec/README.md:24 | 必须由所在模块的受控端口限定其含义 |
| 34 | docs/design/spec/README.md:57 | human 请求可以来自 Workbench/CLI，也可以来自模块 binding 明确接纳的 provider 动作，但必须归一到同一命令 |
| 35 | docs/design/spec/system.md:23 | 即使把全部界面、聊天平台、Task 来源、工作流引擎和终端客户端都换掉，内核所守的身份、权限、版本证据、治理与恢复边界也必须原样保留——这是[愿景文档](../vision.md#产品原生核心与架构最小内核)中“产品原生核心与架构最小内核”在系统层的落点 |
| 36 | docs/design/spec/system.md:34 | 每个 Resolved Port Binding 必须固定 provider 制品、模块专用 adapter 版本、配置摘要、实测能力与降级方式 |
| 37 | docs/design/spec/system.md:36 | 未来非本地 transport 必须认证客户端 |
| 38 | docs/design/spec/system.md:36 | 受 HCTL 单输入租约管理的 Terminal 输入必须先由 Herdr 适配代码校验精确票据、租约和当前代次，再调用 Herdr API |
| 39 | docs/design/spec/system.md:53 | 更新创建新 revision，current pointer 只用于选择，Execution Spec 与 Run Manifest 必须冻结精确 ref+digest |
| 40 | docs/design/spec/system.md:70 | 需要提交 HCTL 命令却无法提供等价预览、版本或权限信息时，动作必须禁用、保留为待处理请求或安全拒绝 |
| 41 | docs/design/spec/system.md:86 | webhook/通知只负责唤醒：接纳前仍以 provider current readback、cursor/gap 和模块 Snapshot 为准，重复、迟到或乱序投递必须得到相同结果 |
| 42 | docs/design/spec/system.md:103 | 验收策略、字段权威或冲突前置要求 fresh readback 时，不可用或 cursor gap 必须返回类型化拒绝 |
| 43 | docs/design/spec/system.md:125 | 它是全部 metadata（稳定身份、Revision 准入与 current、绑定、授权、租约、代次、现场记账、Run Manifest、Execution Spec、Result Proposal 准入、Verdict/Receipt）的唯一权威，一人多机连同一本，必须备份 |
| 44 | docs/design/spec/system.md:141 | 更新 current pointer 必须经唯一 control writer 做 expected-version CAS |
| 45 | docs/design/spec/system.md:158 | 唯一不可再生的完整权威，必须备份 |
| 46 | docs/design/spec/system.md:169 | 新 owner 必须先对账，HCTL 不再向旧 generation 签发输入、停止、接管或结果准入 |
| 47 | docs/design/spec/system.md:171 | 幂等键、generation、租约、outbox 和 readback 必须共同工作 |
| 48 | docs/design/spec/system.md:189 | metadata 备份必须是由唯一 writer 协调的一致备份集：完整账本快照，连同账本引用的精确用户级 Profile/Skill/Runtime 不可变定义字节与 digest——后者存放在账本之外，单备份账本文件会漏掉它们 |
| 49 | docs/design/spec/connections.md:50 | 确认时，「创建 Task」命令或「采纳契约」命令必须冻结： |
| 50 | docs/design/spec/connections.md:97 | Agency 本身不能回显的 fence 必须记录为未生效 |
| 51 | docs/design/spec/connections.md:103 | 如果冻结的端口明确是受信任的纯进程内同步调用，Execution Spec 必须写 `execution_mode = in_process`，可以没有 Repo Instance、Runtime/Terminal、runtime/site/backend generations 或 lease |
| 52 | docs/design/spec/connections.md:168 | 连接需要的新尝试或替代执行必须拥有新的 owner version/generation、Execution Spec 与 runtime generation |
| 53 | docs/design/spec/project.md:42 | 「归档 Project」是 quiescent transition，预览与提交都必须确认：不存在非终态 Run、非终态写入型 Room Invocation、活动输入/写租约，或该 Project 所有且仍为待投递/结果未知的外部副作用 |
| 54 | docs/design/spec/project.md:44 | 创建 Task、Run 或 project_scope Room Invocation 时必须冻结获准的 Project version 与相关策略摘要 |
| 55 | docs/design/spec/project.md:46 | 每次 Execution Spec 必须同时固定实际 Participant revision、Project Role Binding version/digest（repo_scope 可无）与实际 Skill refs/digests |
| 56 | docs/design/spec/project.md:52 | Scoped Room 创建时必须冻结 parent Room、精确讨论目标（Request 或待提交的类型化动作）、完成条件和结论回填动作 |
| 57 | docs/design/spec/project.md:63 | 之后允许丢弃明文，但必须保留 locator/digest、来源链、policy version 和丢弃事实，不得声称仍可 replay |
| 58 | docs/design/spec/project.md:67 | 每个被压缩条目必须记录 compressor ref+digest、压缩率与原文 ref+digest，且压缩产物的每个片段可回源到原文位置 |
| 59 | docs/design/spec/project.md:79 | 用户 Retry 必须在旧授权失效后创建新的 Room Invocation、Execution Spec、runtime generation 和必要的 ChangeSet，并保留原调用引用，不能重放或复活旧调用 |
| 60 | docs/design/spec/project.md:81 | 写入、Project Artifact 或 Project-scoped 权限必须选择精确 Project/version，且只有 project_scope 可以携带 ChangeSet 规则 |
| 61 | docs/design/spec/project.md:85 | 解决 Request 必须经过预览和类型化动作 |
| 62 | docs/design/spec/project.md:87 | 上述阻塞身份相同的重复创建必须去重到现有活动 Request，可以追加提醒事件 |
| 63 | docs/design/spec/project.md:87 | 任一 owner/version/scope 或所需动作变化时必须创建新 Request 并 Supersede 旧 Request，旧解决结果不得推进新 blocker |
| 64 | docs/design/spec/project.md:93 | mention 提交前的 Trigger Preview 必须显示实际 Participant/Worker Profile/Harness、required/optional Skills、Context 来源与 token 估算、权限与写入范围、预算，以及将创建 Room Invocation/Run/Request 还是唤醒多个 worker |
| 65 | docs/design/spec/project.md:97 | mention 的解析必须确定性：`@` 目标只按获准的 Participant/Role 绑定精确解析 |
| 66 | docs/design/spec/project.md:97 | 无唯一授权候选时必须明确失败或要求人选择，不得按显示名模糊匹配、静默换人或把 mention 字符串交给模型猜测路由 |
| 67 | docs/design/spec/project.md:109 | HCTL 的 `@` 解析目标是逻辑 Participant/Role 而非平台账号，且必须经 Trigger Preview 准入 |
| 68 | docs/design/spec/task.md:26 | adapter 必须能按 anchor 稳定回读归属，做不到时该后端只能显示过滤视图，不能声称支持 Task 身份导入或跨组移动 |
| 69 | docs/design/spec/task.md:28 | 无契约的「启动 Run」或「完成 Task」必须先要求该独立动作 |
| 70 | docs/design/spec/task.md:28 | 「完成 Task」不得在同一命令中隐式生成契约：预览必须要求先执行可审阅的「采纳契约」，再针对返回的精确 Revision 重新预览完成 |
| 71 | docs/design/spec/task.md:36 | 「移动 Task」仅能在原 Project anchor 内改 stage/rank，跨 Project group 的预览必须拒绝 |
| 72 | docs/design/spec/task.md:38 | 只有采用外部来源内容的「采纳契约」命令才必须让 Snapshot、字段 authority policy 与新 Task Revision 引用同一个 Task Binding，并把 binding revision、snapshot、contract projection digest 和 authority-policy digest 一并写入 Task Revision |
| 73 | docs/design/spec/task.md:50 | adapter 必须固定 binding revision、规范外部实体 ID、Task ID、provider actor、变化前后值、remote revision/updated version 与可重复计算的幂等键，并在接纳前 fresh readback |
| 74 | docs/design/spec/task.md:52 | Reopen、Cancel、跨 Project 移动和契约采纳第一阶段没有 provider 动作映射，必须使用公共命令入口 |
| 75 | docs/design/spec/task.md:64 | 「启动 Run」必须在创建 Run/Manifest 的同一用户级账本事务把空 claim CAS 为 active |
| 76 | docs/design/spec/task.md:66 | 「完成 Task」命令校验当前 Revision、验收规则、候选、Artifact/SCM/CI 和必需 Receipt，并对影响契约的待采纳默认拒绝（fail-closed）：actor 必须先采纳并按新 Revision 重新验收，或显式选择“按当前冻结 Revision 完成” |
| 77 | docs/design/spec/task.md:66 | 后者必须冻结并 CAS 当前 Task Binding/state version、source head 和全部未采纳的契约 Snapshot refs/digests，预览后新增或变化的 drift 一律使命令失效 |
| 78 | docs/design/spec/task.md:66 | 必须先显式结束该 Run 并等到旧执行撤权、隔离，Task 命令不会隐式停止 Run |
| 79 | docs/design/spec/task.md:70 | 若存在契约分歧，还必须固定显式 divergence choice、精确的未采纳 Snapshot refs/digests、Task Binding revision/state version 与 authority-policy digest |
| 80 | docs/design/spec/task.md:74 | 「重开 Task」命令只接受有权 human actor，必须以预期 task_lifecycle_version 把完成/已取消 → 开放并推进版本 |
| 81 | docs/design/spec/task.md:74 | 若当前来源契约已有未处理 drift，重开预览必须先采纳新 Task Revision 或显式冻结继续使用的当前 Revision 与 divergence，不能让外部 Reopen 或旧完成证明静默决定新一轮施工 |
| 82 | docs/design/spec/task.md:78 | 「启动 Run」命令预览必须列出会影响当前 Task Revision 的全部待采纳，并要求 actor 明确采纳、拒绝或延期 |
| 83 | docs/design/spec/task.md:78 | 拒绝或延期必须随准入冻结当前 Revision 和精确来源快照，但未采纳的契约内容只作准入审计，不得进入 Task Revision、Run Manifest、Context Manifest 或 Execution Spec |
| 84 | docs/design/spec/task.md:80 | Start、Complete、Adopt 与跨来源冲突判断若要求 task backend 的当前 placement、remote revision、source head 或完整 cursor，必须先完成 fresh readback |
| 85 | docs/design/spec/task.md:94 | 会改契约的内容必须经用户采纳 |
| 86 | docs/design/spec/run.md:14 | 一个 HCTL 外部节点必须产出的逻辑结果 |
| 87 | docs/design/spec/run.md:31 | 每个过渡态都必须能通过取消、失败或替代进入终态，不能因 Engine 失联永久阻塞绑定 Task |
| 88 | docs/design/spec/run.md:39 | 任何失败、取消或替代终态在释放 Task Run claim 前，也必须在同一事务撤销旧 dispatch、输入/写租约与外部副作用资格，并提交 runtime stop/fence |
| 89 | docs/design/spec/run.md:57 | 只覆盖局部研究、咨询或中间步骤的自动化必须使用无 Task Run 或 Room Invocation，并以稳定引用把结果交回 Task |
| 90 | docs/design/spec/run.md:61 | 「启动 Run」必须 CAS 活跃 Project/version、可选 Task 的开放 lifecycle/current Revision 及该 Task 的 Run claim，并在同一用户级账本事务创建 Run、不可变 Manifest、幂等结果、`active` Task claim（有 Task 时）和 Engine start outbox |
| 91 | docs/design/spec/run.md:63 | 新执行必须使用新的 Execution Spec/runtime generation |
| 92 | docs/design/spec/run.md:63 | 旧写入未能物理证明静默时还必须按 [Agent 约束](./agent.md#changeset-与-git-事实)使用新 ChangeSet 与新 worktree |
| 93 | docs/design/spec/run.md:65 | 范围、验收、候选、权限、Gate 或超出获准边界的放置变化必须创建替代 Run，不能原地漂移 |
| 94 | docs/design/spec/run.md:67 | dynamic fork 只能实例化 Workflow Revision/Manifest 已冻结的有界 Seat 模板：候选 Participant/Role、最大基数、预算、选择函数和权限上限都必须预先固定 |
| 95 | docs/design/spec/run.md:79 | 它既不是后续激活时分配的 `runtime_generation`，也不是 control/site/backend 的基础设施 fence generation，所有层都必须分别携带并逐项校验 |
| 96 | docs/design/spec/run.md:99 | 必需 reviewer Seat 绑定互不相同的 Participant revision，备用 Attempt 必须继承原 Seat 的逻辑身份和全部评审依据，不能借更换 Worker Profile 改变 Context、Skill、权限、票位或绕过分离 |
| 97 | docs/design/spec/run.md:101 | 策略若要求上述物理或组织独立，而当前端口不能机械认证，就必须把该 Gate 判为 unsupported，不能用不同 Participant 名称冒充独立 |
| 98 | docs/design/spec/agent.md:32 | 每次绑定都必须从实际探测结果中选择精确端口和降级方式，并冻结版本、配置、能力、信任级别和权限 |
| 99 | docs/design/spec/agent.md:34 | control 在派工交付前必须核对 Context Bundle 的交付 bytes digest、spec digest 和全部可验证 fence |
| 100 | docs/design/spec/agent.md:38 | 候选切换、接管或取消必须先让旧 writer 失权 |
| 101 | docs/design/spec/agent.md:38 | 自动恢复路径必须从获准 baseline 创建新物理 worktree 与新 ChangeSet 再取得新 lease |
| 102 | docs/design/spec/agent.md:52 | producer_ref 不进入 review subject digest，但 author/reviewer separation 必须沿它解析并校验当前逻辑身份 |
| 103 | docs/design/spec/agent.md:54 | SCM 变更中断或结果未知时，该命令保持结果未知，工具箱必须回读 HEAD、index、worktree/merge 状态、PR head 和目标分支头，返回类型化恢复动作 |
| 104 | docs/design/spec/agent.md:62 | repo-scoped 调用可以没有 Project ref，但仍必须保留精确 Room Invocation、Execution Runtime、binding、各层代次、权限和适用 fence |
| 105 | docs/design/spec/agent.md:64 | 代次必须分层记录而不能共用一个模糊 `generation`：语义 owner 是 Room Invocation 的 `invocation_version` 或 Attempt 的 `attempt_generation` |
| 106 | docs/design/spec/agent.md:64 | Agency 不能执行的物理 fence 必须明确标为未生效 |
| 107 | docs/design/spec/agent.md:66 | 派出交付物必须按冻结规格逐项核验后方可激活，缺项列出且不激活 |
| 108 | docs/design/spec/agent.md:72 | Execution Spec 必须冻结 terminal input policy：`managed_single_writer` 要求所有输入经当前 descriptor、generation 与 Terminal Input Lease 校验，provider 不能统一拦截所有写入时就关闭原生 controller |
| 109 | docs/design/spec/agent.md:72 | 但执行记录必须标明输入 provenance 不完整，不能声明物理单写者、完整 replay 或由该输入产生 HCTL 命令/结果 |
| 110 | docs/design/spec/agent.md:78 | 每个 harness 适配器必须为其接入端口声明终局结果契约：执行体进程正常退出但缺少契约要求的终局结果事件时，适配器必须合成类型化协议错误，不得默认成功 |
| 111 | docs/design/spec/agent.md:78 | 每个 harness 适配器必须为其接入端口声明终局结果契约：执行体进程正常退出但缺少契约要求的终局结果事件时，适配器必须合成类型化协议错误，不得默认成功 |
| 112 | docs/design/spec/agent.md:78 | 由 control 主动取消导致的退出必须归因为取消，不得上报为执行失败 |
| 113 | docs/design/spec/agent.md:78 | harness 内部派生的子执行体事件必须携带稳定的派生谱系引用，不得摊平进主执行流 |
| 114 | docs/design/spec/agent.md:80 | Proposal 内每个输出项还必须分别固定 schema key、content digest、ChangeSet Revision/Artifact candidate/Evidence refs，以及产生该项的同一适用 generation tuple |
| 115 | docs/design/delivery.md:34 | Terminal 命令必须指向精确 descriptor |
| 116 | docs/design/delivery.md:87 | 创建结果未知、限流、外部修改、tombstone、重新绑定和无 Workbench 原生操作都必须有测试 |
| 117 | docs/design/delivery.md:87 | 无契约的卡不进治理，惰性创建契约和 Done 请求被拒绝的路径都必须有测试 |
| 118 | docs/design/delivery.md:105 | 正式发布、升级与回滚仍必须通过 B6，不能把“已能自举”当成可分发版本 |
| 119 | docs/design/delivery.md:107 | 自举验收不得对 HCTL2 仓库、内置账号或测试环境设置隐藏的特例豁免：开发自身必须只使用公开的 Query/Preview/Submit/Subscribe、CLI 和受控端口，实际 Context、权限与证据均可检查 |
| 120 | docs/design/delivery.md:134 | - **必须原生**：Herdr、harness、`hctl2-control`、`hctl2-tool` 与 CLI——要碰真实 worktree、PTY 与 OS 密钥串，不进容器 |
| 121 | docs/design/delivery.md:143 | 任何采用、移植或 vendor 的外部源码都必须固定已审阅 commit，核验目标文件及依赖许可证，保留 license/copyright/attribution 与修改记录，并用 HCTL contract tests 隔离上游漂移 |
| 122 | docs/design/contract-tests.md:6 | 模块新增约束必须在对应族里增加一个失败用例，而不是再建一份不变量文档 |
| 123 | docs/design/contract-tests.md:73 | Agency 未声明逐次输入记录能力时，还必须标明逐次 provenance、generation 和物理单写者保证不完整 |
| 124 | docs/design/contract-tests.md:96 | - 既有 content 迁移必须显式预览、导出、导入并回读校验 |
| 125 | docs/design/contract-tests.md:104 | - 多个执行现场可以登记（各有工具箱与 Herdr 绑定），但同一 site/repo mutation lease 的旧 generation 必须被 fence，无法证明 fence 时默认不重授写权限，重授只能来自有权 human 预览证据后的显式确认 |

### 1.3 「不得」全量（66 处）

| # | 文件:行 | 整句 |
| --- | --- | --- |
| 1 | docs/design/README.md:69 | `delivery.md` 是验证文档，可引用约束层词汇以指认被测约束，但不得重定义 |
| 2 | docs/design/README.md:70 | 设计正文与场景不得重定义它们 |
| 3 | docs/design/README.md:71 | - 具名概念的引入门槛与族规则见[约束层总则](./spec/README.md)：没有独立生命周期、恢复边界或权限边界的不得命名 |
| 4 | docs/design/README.md:77 | 能够回答独立实现选择、交接、故障或权限边界的设计不得因篇幅被删除 |
| 5 | docs/design/README.md:96 | delivery.md 不得改变领域含义，实现证据不得反向定义产品 |
| 6 | docs/design/README.md:96 | delivery.md 不得改变领域含义，实现证据不得反向定义产品 |
| 7 | docs/design/context.md:76 | 治理引用不得指向纪要，只能指向精确事件 |
| 8 | docs/design/spec/README.md:5 | 两层冲突时以约束层为准，但约束层不得引入设计层没有的产品行为 |
| 9 | docs/design/spec/README.md:18 | 新名字的引入门槛：不满足前两类判据的不得命名 |
| 10 | docs/design/spec/README.md:57 | 结果可以作为记录写回 content 系统，回写本身不得再取得 human provenance |
| 11 | docs/design/spec/system.md:159 | 只有审计影子时仍不得自行重建判决权威 |
| 12 | docs/design/spec/system.md:191 | 不得合并两份分叉账本或把备份恢复成新的账本身份 |
| 13 | docs/design/spec/system.md:199 | - 日志与 trace 使用稳定关联 ID，但不得包含密钥和完整敏感 payload |
| 14 | docs/design/spec/connections.md:12 | 跨 Project、Repo Instance 或模块都不得拆成 clone 本地事务再拼接 |
| 15 | docs/design/spec/connections.md:96 | attempt_generation`，不得预填 runtime identity |
| 16 | docs/design/spec/connections.md:103 | 除此之外不得省略物理 tuple |
| 17 | docs/design/spec/connections.md:125 | 不得写一个无法判定属于哪层的裸 `generation` |
| 18 | docs/design/spec/connections.md:129 | 任何分支都不得投给替代 execution 或留下活动 Seat/Attempt |
| 19 | docs/design/spec/connections.md:168 | 系统对账完成前，各模块都不得表现为已完成交接 |
| 20 | docs/design/spec/project.md:38 | 缺失、冲突或仅有 remote URL 相似都不得静默合并 |
| 21 | docs/design/spec/project.md:48 | 不得复制整段 Room、把隐式聊天窗口当作来源，或让后续 Room 消息改变既有 Project |
| 22 | docs/design/spec/project.md:61 | 必用条目超预算时改为 pointer 并附分片建议，不得静默丢弃 |
| 23 | docs/design/spec/project.md:61 | 指向账本或任务后端的引用不得作为 pointer 交付 |
| 24 | docs/design/spec/project.md:63 | 之后允许丢弃明文，但必须保留 locator/digest、来源链、policy version 和丢弃事实，不得声称仍可 replay |
| 25 | docs/design/spec/project.md:69 | 它不是权威——治理引用不得指向纪要，只能指向精确事件 |
| 26 | docs/design/spec/project.md:87 | 活动 Request 的问题、目标人或角色、`owner_ref + affected_revision_ref + blocked_scope + owner state_version`（Attempt 另带 attempt_generation，Room Invocation 使用 invocation_version）、dedupe root 和获准解决动作不得原地修改 |
| 27 | docs/design/spec/project.md:87 | 任一 owner/version/scope 或所需动作变化时必须创建新 Request 并 Supersede 旧 Request，旧解决结果不得推进新 blocker |
| 28 | docs/design/spec/project.md:97 | 无唯一授权候选时必须明确失败或要求人选择，不得按显示名模糊匹配、静默换人或把 mention 字符串交给模型猜测路由 |
| 29 | docs/design/spec/task.md:28 | 未分组、同时落入多个 Project group 或 anchor 不可稳定回读的卡片只形成未认领 Snapshot/需要关注，不得猜 Project 或先造 Task |
| 30 | docs/design/spec/task.md:28 | 「完成 Task」不得在同一命令中隐式生成契约：预览必须要求先执行可审阅的「采纳契约」，再针对返回的精确 Revision 重新预览完成 |
| 31 | docs/design/spec/task.md:30 | Run 正常完成路径只针对其冻结的 Revision：current 已前移时，Run reducer 的「完成 Task」按契约分歧拒绝，Task 保持开放并显示需要关注，不得静默按新 Revision 完成 |
| 32 | docs/design/spec/task.md:78 | 拒绝或延期必须随准入冻结当前 Revision 和精确来源快照，但未采纳的契约内容只作准入审计，不得进入 Task Revision、Run Manifest、Context Manifest 或 Execution Spec |
| 33 | docs/design/spec/task.md:78 | 存在未处理的待采纳时不得启动 Run，control 也不得自动采纳或静默越过 |
| 34 | docs/design/spec/task.md:78 | 存在未处理的待采纳时不得启动 Run，control 也不得自动采纳或静默越过 |
| 35 | docs/design/spec/run.md:63 | 不得为了替代一个 Run 任意推进共享 site generation 而误伤其他执行 |
| 36 | docs/design/spec/run.md:99 | 被评审 Revision 的作者或 subject producer 不得占用必需 reviewer Seat |
| 37 | docs/design/spec/run.md:116 | 场景客户端不得直接操作 |
| 38 | docs/design/spec/agent.md:29 | 允许原生交互时不得宣称 provider 物理单写者 |
| 39 | docs/design/spec/agent.md:34 | 未声明则不施加、不拦启动、也不得记录为已生效 |
| 40 | docs/design/spec/agent.md:54 | 收敛前不得签发成功 Receipt 或清理所需现场 |
| 41 | docs/design/spec/agent.md:64 | 替代任一层只使引用该层旧值的 HCTL 动作失效，不得顺带把别层 identity 改写成新值 |
| 42 | docs/design/spec/agent.md:68 | Agency 自带的接管、单写者或“会话有效”记录只作执行协助与观测证据，不得写入或替代账本事实 |
| 43 | docs/design/spec/agent.md:76 | 未知事件保留原文并安全降级，不得凭渲染器猜测完成 |
| 44 | docs/design/spec/agent.md:78 | 每个 harness 适配器必须为其接入端口声明终局结果契约：执行体进程正常退出但缺少契约要求的终局结果事件时，适配器必须合成类型化协议错误，不得默认成功 |
| 45 | docs/design/spec/agent.md:78 | 由 control 主动取消导致的退出必须归因为取消，不得上报为执行失败 |
| 46 | docs/design/spec/agent.md:78 | 观测上报通道失败时，只能显式标记该执行的观测截断并终结事件流，不得交付有缺口的事件流冒充完整历史 |
| 47 | docs/design/spec/agent.md:78 | harness 内部派生的子执行体事件必须携带稳定的派生谱系引用，不得摊平进主执行流 |
| 48 | docs/design/spec/agent.md:80 | 只有 Execution Spec 明示的受信任 `in_process` 执行可省略 runtime/site/backend/lease，改为固定 owner、control writer、Extension/Resolved Port Binding、spec/bundle digest 与 producer sequence，也不得提交 ChangeSet |
| 49 | docs/design/spec/agent.md:80 | 不得用顶层“本次执行”概括后混入旧 Attempt、旧 runtime 或另一现场的输出 |
| 50 | docs/design/spec/agent.md:88 | 能力不足时准确降级为 structured inspect 或 terminal，不得改投另一个会话 |
| 51 | docs/design/spec/agent.md:100 | 断开不停止执行，不能证明同一进程和 PTY 时不得声称 exact attach |
| 52 | docs/design/delivery.md:81 | 只有确认目标事实后才写唯一 Integration Receipt，结果未知时不得签成功或盲重投 |
| 53 | docs/design/delivery.md:107 | 自举验收不得对 HCTL2 仓库、内置账号或测试环境设置隐藏的特例豁免：开发自身必须只使用公开的 Query/Preview/Submit/Subscribe、CLI 和受控端口，实际 Context、权限与证据均可检查 |
| 54 | docs/design/delivery.md:143 | 任一项缺失即不得进入分发产物 |
| 55 | docs/design/contract-tests.md:41 | - 同一规范实体跨 Project/connection/placement 不得产生第二个 Task，禁用 binding 也不释放映射 |
| 56 | docs/design/contract-tests.md:66 | - 本地/远端 SCM 集成都先持久 integration intent，由 tool/adapter 执行并 readback，target-head 竞争或 ACK 未知时不得签成功 Integration Receipt |
| 57 | docs/design/contract-tests.md:74 | - `managed_single_writer` 下不得同时开放 Herdr API 写入与原生 controller 写入，尝试原生写入时执行不得继续声称策略成立 |
| 58 | docs/design/contract-tests.md:74 | - `managed_single_writer` 下不得同时开放 Herdr API 写入与原生 controller 写入，尝试原生写入时执行不得继续声称策略成立 |
| 59 | docs/design/contract-tests.md:75 | - Agency 未声明事件游标能力时，不得把事件流当作完整持久 trace |
| 60 | docs/design/contract-tests.md:76 | - Agency 未声明退出与停止回读能力，或不能证明同一进程和 PTY 仍存活时，不得声称 exact attach |
| 61 | docs/design/contract-tests.md:76 | 缺失 exit/stop 回执的执行不得报告为成功停止 |
| 62 | docs/design/contract-tests.md:80 | Agency 未声明栅栏回显能力时，无法执行的 fence 不得被记录为已生效 |
| 63 | docs/design/contract-tests.md:95 | - 新 provider/adapter 未通过对应模块约束测试时不得产生 Resolved Port Binding |
| 64 | docs/design/contract-tests.md:96 | 普通换绑不得冒充无损迁移或热切换 |
| 65 | docs/design/contract-tests.md:97 | Workbench 不得依赖 provider 私有导航或对象模型获得隐藏权限 |
| 66 | docs/design/contract-tests.md:113 | - 从 Git 结晶回灌不得伪造未结晶判决 |

### 1.4 「禁止」全量（1 处）

| # | 文件:行 | 整句 |
| --- | --- | --- |
| 1 | docs/design/spec/system.md:196 | 禁止 remote runtime script/CDN，CSP 拒绝远程或未声明的可执行来源 |

### 1.5 「只能」全量（46 处）

| # | 文件:行 | 整句 |
| --- | --- | --- |
| 1 | docs/design/vision.md:130 | 外部平台的事件先按模块约束分类：content 变化进入观测/投影，显式且可归属的 human 动作可以成为同一 HCTL 命令请求，运行时输入只推动精确执行，结果只能作为提案 |
| 2 | docs/design/vision.md:145 | Run 只能在批准的边界内自动推进 |
| 3 | docs/design/architecture.md:77 | 执行只能提议 |
| 4 | docs/design/project.md:23 | - Request 只能由获准动作解决，只解锁它声明的阻塞范围 |
| 5 | docs/design/project.md:56 | 普通 Room 里的临场执行边只能来自可稳定归属到 human 的动作，并且必须先经过 Trigger Preview |
| 6 | docs/design/project.md:56 | 模型 Participant 的消息、结果提议和总结（包括正文里的 `@`）只能形成“下一位协作者”的建议，不能自行发起调用、唤醒 worker（执行体）或层层转包 |
| 7 | docs/design/task.md:41 | 简单工作不需要先画 Workflow，也不伪造只能由 Run 产生的 Gate（评审关卡）凭证 |
| 8 | docs/design/context.md:9 | 那只能恢复文字，回答不了真正要紧的问题：这次执行继承了哪些目标、决定、约束和未决问题 |
| 9 | docs/design/context.md:44 | 指针只能指向执行体在获准范围内用自身工具能打开的位置——Git 对象和 worktree 路径 |
| 10 | docs/design/context.md:76 | 治理引用不得指向纪要，只能指向精确事件 |
| 11 | docs/design/spec/system.md:36 | 需要 HCTL 先记账、撤权或核验前置的 provider mutation 仍只能由 control 经对应受控端口发起 |
| 12 | docs/design/spec/system.md:68 | 受控端口报告 provider 支持的读写能力，实际字段权威只能由对应模块的 authority binding 授予 |
| 13 | docs/design/spec/system.md:101 | 普通 Room 临场 fan-out 只接受有权 human actor，Workflow reducer 只能实例化 Workflow Revision 已冻结的边 |
| 14 | docs/design/spec/system.md:103 | 跨模块命令也只能使用这一个事务边界，不能由两个模块或两个 clone 事后拼接 |
| 15 | docs/design/spec/system.md:158 | Git 审计影子只能辅助显式恢复，不能伪造未结晶判决 |
| 16 | docs/design/spec/system.md:191 | 恢复只能在旧 writer 已停止且取得用户级排他锁后进行 |
| 17 | docs/design/spec/connections.md:10 | 1. 来源模块只能提供稳定 ID、Revision digest、状态版本、来源和已获授权的范围 |
| 18 | docs/design/spec/connections.md:63 | 提交后发生的上游更新不改写活动 Run，只能影响新 Run 或触发显式替代 |
| 19 | docs/design/spec/connections.md:109 | `in_process` 只能使用上段的缩减 tuple，不能把一个合格项的代次套给另一个旧项 |
| 20 | docs/design/spec/connections.md:147 | 权限只能逐级缩小：actor/Project role → Run Manifest（有 Run 时）→ Execution Spec → Agency/adapter envelope |
| 21 | docs/design/spec/project.md:38 | 写入或 ACK 结果未知时 Repo 保持待确认，恢复只能按 identity/digest 回读并完成同一注册，不能再生成一个 Repo 或 Repo Room |
| 22 | docs/design/spec/project.md:44 | repo_scope Room Invocation 改为冻结 Repo Instance/repo/base 且只能只读 |
| 23 | docs/design/spec/project.md:48 | 该命令只能显式选择来源 Message 引用和/或已预览的 Context Manifest/Context Bundle 摘要，并冻结所选内容的可追溯来源链 |
| 24 | docs/design/spec/project.md:61 | pointer 只记精确 ref+digest 与一句摘要，且只能指向执行体在获准范围内以自身工具可打开的位置——Git 对象与 worktree 路径：Repo/Git 内容、ChangeSet Revision、Artifact、Memo、Skill 与 Verdict/Receipt 的 Git 结晶副本 |
| 25 | docs/design/spec/project.md:61 | Repo Room → Project Room → Run 的传承只能通过这些显式 parent/source 引用发生 |
| 26 | docs/design/spec/project.md:69 | 它不是权威——治理引用不得指向纪要，只能指向精确事件 |
| 27 | docs/design/spec/project.md:81 | `in_process` 只能使用连接约束明确的缩减 tuple |
| 28 | docs/design/spec/project.md:95 | 普通 Room 的临场执行边只能由可稳定归属到 human 的动作在 Trigger Preview 后提交 |
| 29 | docs/design/spec/project.md:113 | 差异化语义：向指定人/角色索取输入的一级对象，只能由获准动作解决 |
| 30 | docs/design/spec/task.md:26 | adapter 必须能按 anchor 稳定回读归属，做不到时该后端只能显示过滤视图，不能声称支持 Task 身份导入或跨组移动 |
| 31 | docs/design/spec/task.md:28 | Task Revision 契约按需创建（契约惰性），但只能由显式「采纳契约」或带已预览契约的「创建 Task」产生 |
| 32 | docs/design/spec/task.md:64 | 替代只能走 [Run 约束](./run.md#启动与-manifest)规定的原子撤权/换代路径，不能先清空 claim 再留下两个可写执行 |
| 33 | docs/design/spec/run.md:37 | `运行中 → 完成` 不是通用写入口，只能由确定性 reducer 在同一预览版本上证明以下正常完成谓词后执行：冻结 Workflow Revision 的全部 required Obligation、Seat、Gate 与声明输出均已在账本中以精确 subject 和 Evidence/Verdict/Receipt 达成 |
| 34 | docs/design/spec/run.md:37 | 任一项未知都只能保持运行/暂停/需要关注或走类型化失败、取消、替代，Engine 检查点结束、进程退出、Harness/LLM 自述和单个 Proposal 都不能补足谓词 |
| 35 | docs/design/spec/run.md:39 | 若只能撤销逻辑权威而无法证明旧进程已静默，则隔离旧 worktree/ChangeSet，后续执行按 Agent 约束使用新 worktree **和**新 ChangeSet |
| 36 | docs/design/spec/run.md:67 | dynamic fork 只能实例化 Workflow Revision/Manifest 已冻结的有界 Seat 模板：候选 Participant/Role、最大基数、预算、选择函数和权限上限都必须预先固定 |
| 37 | docs/design/spec/run.md:75 | 3. [Agent](./agent.md) 模块执行 Attempt，只能返回 Result Proposal、Revision 和证据 |
| 38 | docs/design/spec/agent.md:74 | 未声明或证据不足时，只能报告 semantic resume、replay 或丢失 |
| 39 | docs/design/spec/agent.md:78 | 观测上报通道失败时，只能显式标记该执行的观测截断并终结事件流，不得交付有缺口的事件流冒充完整历史 |
| 40 | docs/design/spec/agent.md:80 | 任一代次、binding、bundle、lease 或输出范围不匹配的项只能留作审计，不能让其他合格项替它背书 |
| 41 | docs/design/spec/agent.md:92 | 无法证明是同一进程时只能 semantic resume、replay 或新建执行，不能声称 exact attach |
| 42 | docs/design/delivery.md:148 | - Repo Room 的隐私与保留期限——端到端加密不是答案（HCTL 房间对 control 明文可读），只能由 homeserver 侧访问控制、传输/存储加密与保留策略回答 |
| 43 | docs/design/contract-tests.md:75 | 重连后只能按可证明范围恢复观察 |
| 44 | docs/design/contract-tests.md:98 | 否则只能成为 content/Snapshot/runtime observation |
| 45 | docs/design/contract-tests.md:103 | - 同一用户级账本只能有一个 control writer，第二 writer 拒绝 |
| 46 | docs/design/contract-tests.md:104 | - 多个执行现场可以登记（各有工具箱与 Herdr 绑定），但同一 site/repo mutation lease 的旧 generation 必须被 fence，无法证明 fence 时默认不重授写权限，重授只能来自有权 human 预览证据后的显式确认 |

### 1.6 「可以」全量（122 处）

| # | 文件:行 | 整句 |
| --- | --- | --- |
| 1 | docs/usage.md:3 | HCTL2 仍处于早期实现阶段：现在可以运行 Chatroom、Kanban、Workflow、Terminal 四类打包依赖及一个 Rust 骨架程序，但公共 `hctl2` CLI、控制面和 Workbench 尚未实现 |
| 2 | docs/usage.md:39 | 如果 `$HOME/.local/bin` 尚未在 `PATH` 中，可以为当前 shell 加入： |
| 3 | docs/usage.md:45 | 也可以安装到另一个绝对路径： |
| 4 | docs/usage.md:93 | 因此它可以直接用于脚本、健康检查和 CI，但在启用了 `set -e` 的 shell 中也会使脚本立即退出 |
| 5 | docs/usage.md:141 | 当前 Tuwunel 配置禁用 federation 和房间加密，以便 HCTL2 控制面将来可以按消息 ID 读取 HCTL Room 正文 |
| 6 | docs/usage.md:194 | 也可以直接运行： |
| 7 | docs/design/vision.md:11 | HCTL2 的价值在于中间那座桥——人的意图如何变成机器可以领取的承诺，机器的产出如何变成人可以信任的结果 |
| 8 | docs/design/vision.md:11 | HCTL2 的价值在于中间那座桥——人的意图如何变成机器可以领取的承诺，机器的产出如何变成人可以信任的结果 |
| 9 | docs/design/vision.md:21 | 3. 哪些自动施工已获授权，凭什么可以继续或通过评审 |
| 10 | docs/design/vision.md:55 | 哪些讨论已经足够稳定，可以成为承诺 |
| 11 | docs/design/vision.md:57 | - **治理**：哪些自动施工获得了有边界的授权，凭什么可以继续、通过评审或算作有效结果 |
| 12 | docs/design/vision.md:64 | 一件事不必完整经历四个阶段：简单 Task 可以不创建 Run |
| 13 | docs/design/vision.md:64 | Project 可以直接发起一次边界明确的 Harness 调用 |
| 14 | docs/design/vision.md:64 | 纯研究或文档的 Project 可以从未施工 |
| 15 | docs/design/vision.md:65 | 下端的资源可以丢失、重建和接管 |
| 16 | docs/design/vision.md:97 | 系统可以研究、建议、汇总，但不能替用户决定目标 |
| 17 | docs/design/vision.md:100 | Run r1 按冻结的版本施工时，Project Room 可以继续讨论 r2——讨论不必等施工结束，施工也不会随讨论漂移 |
| 18 | docs/design/vision.md:102 | “自动推进”不等于施工没有外部输入：用户可以暂停、取消、回答 Request，执行者交回提案，定时器到期，引擎报告机械位置 |
| 19 | docs/design/vision.md:130 | 外部平台的事件先按模块约束分类：content 变化进入观测/投影，显式且可归属的 human 动作可以成为同一 HCTL 命令请求，运行时输入只推动精确执行，结果只能作为提案 |
| 20 | docs/design/vision.md:145 | 系统可以并行研究和建议，但不能替用户决定目标 |
| 21 | docs/design/architecture.md:21 | 终端票据也可以把观察者带到另一个执行现场 |
| 22 | docs/design/architecture.md:21 | 一套控制面因此可以服务多个客户端和执行现场 |
| 23 | docs/design/architecture.md:45 | 官方远程 Agent 可以直接实现 Agency 约束，或由专用适配器接入 |
| 24 | docs/design/architecture.md:47 | Terminal 的字节流和绘制性能敏感，Workbench 可以通过客户端侧 transport adapter 直连精确目标 |
| 25 | docs/design/architecture.md:49 | “可替换”分三档承诺，不能混为一谈：新工作可以在通过约束测试后选择另一 provider |
| 26 | docs/design/architecture.md:85 | 不依赖 Room 新消息、来源链或 fresh Context 的施工可以继续，依赖这些新鲜读数的预览与命令安全暂停 |
| 27 | docs/design/architecture.md:94 | 判决的结晶副本进 Git 后可以部分回灌，但回灌不能伪造未结晶的判决 |
| 28 | docs/design/architecture.md:95 | - **content**：丢失不会抹掉已经接纳的治理事实——已结晶的部分（决议、契约、凭证、代码）存活于 Git，有桥接来源的部分可以重放，丢掉的是尚未结晶的记忆 |
| 29 | docs/design/README.md:15 | Task 可以没有 Run |
| 30 | docs/design/README.md:15 | Project 可以发起一次 Harness 调用 |
| 31 | docs/design/README.md:15 | Kanban 可以显示 Run 和 Artifact 投影 |
| 32 | docs/design/README.md:35 | Room 与 Run 可以互相引用，但不存在包含关系：Project Room 可以展示多个 Run |
| 33 | docs/design/README.md:35 | Room 与 Run 可以互相引用，但不存在包含关系：Project Room 可以展示多个 Run |
| 34 | docs/design/README.md:35 | Scoped Room 可以由某个 Run 的 Request 派生 |
| 35 | docs/design/README.md:70 | - 只有约束层的四个模块约束可以定义模块特有的领域名词、状态、写入者和不变量 |
| 36 | docs/design/project.md:7 | Project 模块保存的正是这份长期事实，Chat Room 则是所有 Harness 都消失之后仍然可以恢复的协作现场——它回答“我们要解决什么、为什么、依据是什么，以及哪些讨论已经足够稳定，可以成为承诺” |
| 37 | docs/design/project.md:7 | Project 模块保存的正是这份长期事实，Chat Room 则是所有 Harness 都消失之后仍然可以恢复的协作现场——它回答“我们要解决什么、为什么、依据是什么，以及哪些讨论已经足够稳定，可以成为承诺” |
| 38 | docs/design/project.md:11 | Project 也不是施工管线：研究、规格说明、ADR（架构决策记录）和纯文档的 Project 可以从未创建 Run |
| 39 | docs/design/project.md:52 | 在 Workbench 里同时管理多个仓库时，一个 Room 可以把另一个仓库 Room 的 Participant 阵容借用为预填选择，不必逐个重选 |
| 40 | docs/design/project.md:56 | 动作可以由 Workbench/CLI 直接提交，也可以由按公开约束适配的 provider（供应端）结构化事件提交，客户端名称不改变规则 |
| 41 | docs/design/project.md:56 | 动作可以由 Workbench/CLI 直接提交，也可以由按公开约束适配的 provider（供应端）结构化事件提交，客户端名称不改变规则 |
| 42 | docs/design/project.md:58 | 可以做什么 |
| 43 | docs/design/project.md:73 | - Project 可以通过 [Agent](./agent.md) 模块发起一次 Room Invocation |
| 44 | docs/design/task.md:9 | 卡片的职责是把复杂的运行时细节压缩成可以直接行动的信息：一眼看到承诺内容、谁在做、卡在哪里、证据齐不齐、下一步该点什么 |
| 45 | docs/design/task.md:17 | 看板上的卡可以随后端自由增删拆合（子任务、清单都是 content），每张卡都有稳定的 Task 身份映射，但只有值得冻结验收契约的卡才升格为治理意义上的承诺——契约在首次绑定 Run、首次提交完成命令或显式升格时才创建 |
| 46 | docs/design/task.md:49 | 把已绑定卡片移入 provider 的 Done 是明确的完成意图，若事件具有操作者、版本和幂等依据，adapter（适配器）可以把它转成同一个完成请求 |
| 47 | docs/design/task.md:53 | 可以做什么 |
| 48 | docs/design/task.md:59 | 没有 Workbench 时，任务后端的原生界面可以继续修改 content 字段并在支持时请求完成 |
| 49 | docs/design/run.md:7 | 真正缺失的，是一套面向项目语义、绑定精确版本与证据的治理层——它回答“这个节点对应哪份交付义务、谁有资格尝试、结果是否有效、凭什么可以算完成” |
| 50 | docs/design/run.md:9 | 两种权威一句话可以分清： |
| 51 | docs/design/run.md:12 | > HCTL2 说：“它对应哪份交付义务（Obligation），谁可以尝试（Seat 与候选），结果是否有效（Verdict），是否可以完成（准入校验） |
| 52 | docs/design/run.md:12 | > HCTL2 说：“它对应哪份交付义务（Obligation），谁可以尝试（Seat 与候选），结果是否有效（Verdict），是否可以完成（准入校验） |
| 53 | docs/design/run.md:47 | 可以做什么 |
| 54 | docs/design/agent.md:7 | 进程、PTY（伪终端）、worktree（Git 工作树）和终端连接都是可替换的物理资源：它们可以丢失、重建和接管，但不能反过来定义 Project、Task 或 Run 的事实 |
| 55 | docs/design/agent.md:17 | - worktree 是变更集的可替换载体，可以丢弃重建，不永久属于任何 Project、Task、Room 或参与者 |
| 56 | docs/design/agent.md:21 | - Harness 在自己的工作树里可以正常用 Git：看分支、看日志、fetch、和目标分支比对、在自己的变更集分支上提交 |
| 57 | docs/design/agent.md:26 | 它可以建议完成、建议下一位协作者 |
| 58 | docs/design/agent.md:34 | 它可以渲染结构化执行流或真实 PTY，不要求每次执行都有 shell |
| 59 | docs/design/agent.md:44 | semantic resume 不必依赖厂商保留的会话文件：自有观测留痕（转录与结构化事件）足够时可以重建等价的最小续跑输入 |
| 60 | docs/design/agent.md:46 | Workbench 就位之前（P2），观察与接管可以经 `hctl2 terminal` 取得 control 为精确目标签发的短期票据，也可以在执行规格允许原生交互时使用 Herdr TUI |
| 61 | docs/design/agent.md:46 | Workbench 就位之前（P2），观察与接管可以经 `hctl2 terminal` 取得 control 为精确目标签发的短期票据，也可以在执行规格允许原生交互时使用 Herdr TUI |
| 62 | docs/design/agent.md:50 | 可以做什么 |
| 63 | docs/design/context.md:26 | 可以事后改写的东西 |
| 64 | docs/design/context.md:58 | 只有当用户配置了专用小模型（small-brain）时才启用**压缩**——摘要式或逐词裁剪式都可以，但压缩永远是清单里显式记录的一步：用了哪个模型、压了多少、原文指纹是什么，全部冻结 |
| 65 | docs/design/spec/README.md:9 | 只有前两类可以引入新造的对象名，状态值与引用格式只是枚举与格式，不占概念名额： |
| 66 | docs/design/spec/README.md:18 | 约束需要逐字指认的协议或 schema 字段、序列化格式标识，以及外部标准、产品或源码中的原名，可以保留原形并用代码格式标示 |
| 67 | docs/design/spec/README.md:57 | human 请求可以来自 Workbench/CLI，也可以来自模块 binding 明确接纳的 provider 动作，但必须归一到同一命令 |
| 68 | docs/design/spec/README.md:57 | human 请求可以来自 Workbench/CLI，也可以来自模块 binding 明确接纳的 provider 动作，但必须归一到同一命令 |
| 69 | docs/design/spec/README.md:57 | 结果可以作为记录写回 content 系统，回写本身不得再取得 human provenance |
| 70 | docs/design/spec/system.md:25 | 可以替换的端口包括： |
| 71 | docs/design/spec/system.md:36 | Terminal 的观察流和普通交互输入可以由 Workbench 直连精确 terminal |
| 72 | docs/design/spec/system.md:36 | 未来官方远程 Agent 可以直接实现 Agency 约束，或由专用 Agency adapter 接入 |
| 73 | docs/design/spec/system.md:51 | 它们可以阻止新准入或驱动对账，但不会改写历史 binding |
| 74 | docs/design/spec/system.md:68 | 外部平台可以拥有其场景 content 的 ground truth，以及明确授权的字段 |
| 75 | docs/design/spec/system.md:70 | 界面控件本身没有固定语义：同一个拖放可以只是 task content 移动，也可以在进入 Done 时产生一个 human 完成请求 |
| 76 | docs/design/spec/system.md:70 | 界面控件本身没有固定语义：同一个拖放可以只是 task content 移动，也可以在进入 Done 时产生一个 human 完成请求 |
| 77 | docs/design/spec/system.md:84 | 一个 provider 事件可以同时具有 content 含义和命令请求含义 |
| 78 | docs/design/spec/system.md:84 | 若事件还能证明是配置中映射的 human 所做，并携带稳定外部实体、前后 revision/updated version 和可重复计算的幂等依据，Task adapter 可以再把它归一为「完成 Task」请求 |
| 79 | docs/design/spec/system.md:86 | 来源简化不等于可以丢掉 target、expected version/generation、幂等键和事件顺序 |
| 80 | docs/design/spec/system.md:101 | human 动作既可以来自 Workbench/CLI 的 direct client connection，也可以来自模块 binding 明确接纳的 provider event |
| 81 | docs/design/spec/system.md:101 | human 动作既可以来自 Workbench/CLI 的 direct client connection，也可以来自模块 binding 明确接纳的 provider event |
| 82 | docs/design/spec/system.md:119 | 一个 Repo 可以显式挂接多个 Repo Instance，每个现场固定稳定 `repo_instance_id`、精确 `repo_id`、host/site identity、Git common-dir identity 与首次校验的 Git 证据 |
| 83 | docs/design/spec/system.md:125 | 仓库 clone 本地的 `<git-common-dir>/hctl2/`（当前 Repo Instance 及其 linked worktree 的共享运行目录）只有 OS 锁、traces 与可丢弃缓存——**不是账本，也不是事实源**：现场状态永远可以从 metadata 账本、Git 与运行时观测对账重建，删除该目录不丢失任何事实（无法证明身份的旧执行标为丢失并撤权） |
| 84 | docs/design/spec/system.md:127 | 获准的记录可以写回 content 系统（记录不是命令） |
| 85 | docs/design/spec/system.md:167 | writer 可以搬迁（换机器、上服务器），账本身份不变 |
| 86 | docs/design/spec/connections.md:13 | 来源和场景可以投影它们，但不能复制一套状态机 |
| 87 | docs/design/spec/connections.md:50 | Chat Room 可以生成 Task 提炼提案的预览，但预览不是第二个 Task |
| 88 | docs/design/spec/connections.md:103 | 如果冻结的端口明确是受信任的纯进程内同步调用，Execution Spec 必须写 `execution_mode = in_process`，可以没有 Repo Instance、Runtime/Terminal、runtime/site/backend generations 或 lease |
| 89 | docs/design/spec/connections.md:125 | Request 由 Project 模块保存，但可以阻塞 Task 待办、Run 中的 Attempt/Seat/Obligation，或直接 Room Invocation |
| 90 | docs/design/spec/project.md:40 | Workbench 可以另行把同源 Request/health 投影聚合为全局需要关注 |
| 91 | docs/design/spec/project.md:56 | chat server 不可用，或绑定后房间被开启端到端加密时，不依赖新消息、当前成员或新 cursor 的 metadata 命令可以继续 |
| 92 | docs/design/spec/project.md:77 | 它可以持有一份 Execution Spec 和可选 Harness 运行时，但没有持久 DAG、候选自动切换、Gate 或自动后继 |
| 93 | docs/design/spec/project.md:81 | scope 中 Repo Room 可以在没有 Project 的情况下做只读研究 |
| 94 | docs/design/spec/project.md:81 | 写入、Project Artifact 或 Project-scoped 权限必须选择精确 Project/version，且只有 project_scope 可以携带 ChangeSet 规则 |
| 95 | docs/design/spec/project.md:85 | 开放式商议可以升级为 Scoped Room，但讨论结论仍需由有权 actor 提交原动作 |
| 96 | docs/design/spec/project.md:87 | 上述阻塞身份相同的重复创建必须去重到现有活动 Request，可以追加提醒事件 |
| 97 | docs/design/spec/project.md:95 | 动作可以来自 Workbench/CLI 的 direct client connection，也可以来自 Chat 端口绑定明确允许的 provider 结构化事件 |
| 98 | docs/design/spec/project.md:95 | 动作可以来自 Workbench/CLI 的 direct client connection，也可以来自 Chat 端口绑定明确允许的 provider 结构化事件 |
| 99 | docs/design/spec/task.md:20 | 满足本约束后文要求的 human Done 事件可以请求「完成 Task」，但只有命令成功才改变 lifecycle |
| 100 | docs/design/spec/task.md:26 | group anchor 可以是后端父实体、milestone 或获准的 label/filter identity，但永远不是 Task、Task Binding 或某张“项目卡” |
| 101 | docs/design/spec/task.md:50 | 所选 task backend 的事件还可以承载 human 命令请求，但只对 Task Binding 明确列明的动作生效 |
| 102 | docs/design/spec/task.md:68 | human 请求可以来自 Workbench/CLI direct client connection，也可以来自上文已准入的 provider Done event |
| 103 | docs/design/spec/task.md:68 | human 请求可以来自 Workbench/CLI direct client connection，也可以来自上文已准入的 provider Done event |
| 104 | docs/design/spec/run.md:65 | 运行中只有 Manifest 明确声明为可变的放置参数可以按冻结规则和边界调整 |
| 105 | docs/design/spec/run.md:85 | Resolve/Expire 的跨模块事务都 CAS 精确 Request 与 blocker version，只有 Resolve 可以写答案 delivery，Expire 不能猜测答案而是按冻结策略结束对应 Attempt/Seat/Obligation |
| 106 | docs/design/spec/run.md:97 | 只有冻结策略列明的类型化技术故障，例如候选特有的认证/配额/网络故障、进程或运行时丢失、租约超时，才可以切换 Attempt |
| 107 | docs/design/spec/agent.md:38 | 有权 human actor 预览证据与残留后，可以显式选择接管（对原 worktree 与原 ChangeSet 推进代次并授予新 lease）、把残留封存为原 producer 的 Revision、采用到另一 ChangeSet，或显式丢弃 |
| 108 | docs/design/spec/agent.md:54 | Harness/model 可以在 worktree 内做普通 Git 操作，但不能取得集成 authority：绕过该命令直接改写目标 ref 不产生 Integration Receipt，只在下一次预览或回读时表现为 expected target head 不匹配的 drift，由有权 actor 对账处理 |
| 109 | docs/design/spec/agent.md:56 | 有权 human actor 在预览残留后显式确认丢弃时，可以不留副本直接拆除 |
| 110 | docs/design/spec/agent.md:60 | Run 经其 Attempt 可以有多个 Execution Runtime |
| 111 | docs/design/spec/agent.md:60 | Execution Runtime 可以是容器、隔离作用域或结构化会话，不以 TTY 存在为前提 |
| 112 | docs/design/spec/agent.md:62 | repo-scoped 调用可以没有 Project ref，但仍必须保留精确 Room Invocation、Execution Runtime、binding、各层代次、权限和适用 fence |
| 113 | docs/design/spec/agent.md:68 | 原生输入可以按下文输入策略成为正常的用户运行时输入，但不能承载要求物理 fence 的动作，也不能作为高证据类结果直接准入 |
| 114 | docs/design/spec/agent.md:80 | Harness 可以提交提案，但 Project/Run 才能逐项校验 |
| 115 | docs/design/spec/agent.md:82 | 它们可以建议完成或建议下一位 Participant |
| 116 | docs/design/spec/agent.md:86 | Terminal 各能力（exact attach、native handoff、structured inspect、semantic resume、replay，见[设计正文](../agent.md#terminal-场景)）可以并存但不能互相冒充 |
| 117 | docs/design/spec/agent.md:86 | direct client 按当前 owner/binding 与全部适用代次请求连接时，control 可以签发短期 Attach Descriptor，并为受管理写输入另行 CAS Terminal Input Lease |
| 118 | docs/design/spec/agent.md:86 | 一个目标可以有多个观察者，HCTL 管理的输入默认最多一个 Terminal Input Lease 持有者 |
| 119 | docs/design/spec/agent.md:86 | binding 声明 `native_interactive_allowed` 时，provider 原生客户端或 Workbench 直连 transport 可以不经该租约输入 |
| 120 | docs/design/spec/agent.md:92 | semantic resume 可以用自有观测留痕重建续跑输入 |
| 121 | docs/design/delivery.md:103 | 旧工具在事实切换前可以作为执行者或逃生通道，不能继续保有平行 Project/Task/Run 账本 |
| 122 | docs/design/contract-tests.md:104 | - 多个执行现场可以登记（各有工具箱与 Herdr 绑定），但同一 site/repo mutation lease 的旧 generation 必须被 fence，无法证明 fence 时默认不重授写权限，重授只能来自有权 human 预览证据后的显式确认 |

### 1.7 「应当」全量（0 处）

（零命中）

### 1.8 「不应」全量（1 处）

| # | 文件:行 | 整句 |
| --- | --- | --- |
| 1 | docs/usage.md:16 | `hctl2-tool` 暂时用于验证代码树、构建链和命令边界，不应作为后台服务部署 |

## 扫描 2 · 同一概念多种写法

口径：除特别注明外，位置在剔除代码围栏后的**原文行**上扫描（含行内代码与链接，保证 `grep` 可复算）；计数为出现次数（非行数）。

### 2.1 三组点名概念的全量位置

**harness / Harness**

| 形态 | 次数 | 位置 |
| --- | --- | --- |
| harness（小写） | 21 | `docs/design/architecture.md`: 34, 60；`docs/design/README.md`: 13×2；`docs/design/agent.md`: 24, 25, 34, 54；`docs/design/participant.md`: 44；`docs/design/context.md`: 88；`docs/design/references/glossary.md`: 67；`docs/design/spec/README.md`: 24；`docs/design/spec/system.md`: 12, 32, 81；`docs/design/spec/connections.md`: 164；`docs/design/spec/agent.md`: 78×2；`docs/design/delivery.md`: 16, 56, 134 |
| Harness（大写） | 97 | `README.md`: 3×2；`docs/design/vision.md`: 9, 17, 26, 34, 36, 64, 86, 89, 109, 149, 160×2, 162×2；`docs/design/architecture.md`: 34, 76；`docs/design/README.md`: 15；`docs/design/project.md`: 7×2, 15, 21；`docs/design/task.md`: 7, 13, 39, 68；`docs/design/run.md`: 7；`docs/design/agent.md`: 13×2, 21, 27, 39, 55, 59, 61；`docs/design/participant.md`: 9, 50；`docs/design/references/glossary.md`: 11, 55, 118, 136, 138；`docs/design/spec/system.md`: 13, 84, 101×2, 111×2, 113×2, 163, 200×2；`docs/design/spec/connections.md`: 45, 113；`docs/design/spec/project.md`: 12, 19, 30, 46×2, 63, 77, 93；`docs/design/spec/task.md`: 50；`docs/design/spec/run.md`: 37, 67, 101；`docs/design/spec/agent.md`: 12, 13×2, 19, 25, 30, 32, 34×5, 38, 54, 66, 80×2, 82, 104；`docs/design/delivery.md`: 64, 67, 76, 97, 125, 136；`docs/design/contract-tests.md`: 70×2, 93, 142 |
| HARNESS（全大写） | 0 | （零命中） |

备注（事实，非判断）：小写 `harness` 是五个系统角色名之一（`docs/design/spec/README.md:24` 的词汇表），大写 `Harness` 是核心产品词（`glossary.md:11`）；两种形态的具体分工是否被遵守，留给语言通读裁决。

**路标 / 机械位置 / Engine 位置**

| 形态 | 次数 | 位置 |
| --- | --- | --- |
| 路标 | 25 | `docs/design/architecture.md`: 87×2；`docs/design/run.md`: 22, 35, 41；`docs/design/context.md`: 29, 82；`docs/design/spec/system.md`: 162×3；`docs/design/spec/run.md`: 27, 31, 37×2, 41, 73×2, 77×2, 81, 93, 105, 116；`docs/design/contract-tests.md`: 52, 59 |
| 机械位置 | 5 | `docs/design/vision.md`: 102；`docs/design/run.md`: 14；`docs/design/spec/system.md`: 162；`docs/design/spec/run.md`: 118, 119 |
| Engine 位置 | 2 | `docs/design/spec/run.md`: 31, 33 |

**工具箱 / hctl2-tool**

| 形态 | 次数 | 位置 |
| --- | --- | --- |
| 工具箱 | 45 | `docs/design/agent.md`: 21, 27, 83；`docs/design/spec/README.md`: 36；`docs/design/spec/system.md`: 111×2, 113, 119, 121, 127, 147, 148, 159, 169, 177, 180, 197；`docs/design/spec/connections.md`: 44, 57；`docs/design/spec/project.md`: 25, 33, 34, 38×2；`docs/design/spec/task.md`: 30, 34；`docs/design/spec/run.md`: 18, 24, 29, 47, 76, 99；`docs/design/spec/agent.md`: 26, 27, 34, 54×3, 56；`docs/design/delivery.md`: 51, 56；`docs/design/contract-tests.md`: 70, 104, 107×2 |
| hctl2-tool | 14 | `docs/usage.md`: 10, 16, 20, 51, 178, 206；`docs/design/architecture.md`: 14；`docs/design/agent.md`: 61；`docs/design/spec/system.md`: 12, 169；`docs/design/delivery.md`: 56, 66, 81, 134 |

L3 允许「散文名 + 组件标识符」成对存在，但要求在词汇表显式声明一对一映射；`glossary.md` 现有词条未收「工具箱」（事实核查：`grep -n 工具箱 docs/design/references/glossary.md` 零命中）。

### 2.2 中文名变体（同一对象的两个中文说法）

口径：散文（剔除行内代码与链接 URL）。

| 对象（词汇表中文对照） | 变体 | 次数 | 位置 |
| --- | --- | --- | --- |
| Result Proposal = 结果提案 | 结果提案 ｜ 结果提议 | 4 ｜ 5 | 结果提案：`docs/design/agent.md`: 13, 82；`docs/design/references/glossary.md`: 10, 118<br>结果提议：`docs/design/architecture.md`: 77；`docs/design/project.md`: 56；`docs/design/run.md`: 60；`docs/design/spec/README.md`: 26；`docs/design/spec/system.md`: 81 |
| ChangeSet Revision = 变更集快照 | 变更集快照 ｜ 变更集版本 | 2 ｜ 4 | 变更集快照：`docs/design/agent.md`: 82；`docs/design/references/glossary.md`: 77<br>变更集版本：`docs/design/run.md`: 41；`docs/design/context.md`: 84, 86×2 |
| Context Bundle = 消费上下文包 | 消费上下文包 ｜ 内容包 | 1 ｜ 5 | 消费上下文包：`docs/design/references/glossary.md`: 129<br>内容包：`docs/design/context.md`: 5, 26, 108, 109；`docs/design/spec/project.md`: 15 |
| Task Revision = 任务契约版本 | 任务契约版本 ｜ 任务契约（短称） | 2 ｜ 5 | 任务契约版本：`docs/design/vision.md`: 73；`docs/design/references/glossary.md`: 75<br>任务契约（短称）：`docs/design/vision.md`: 28；`docs/design/architecture.md`: 58, 74；`docs/design/run.md`: 21, 33 |
| Context Manifest = 根上下文清单 | 根上下文清单 ｜ 上下文清单（短称） | 1 ｜ 3 | 根上下文清单：`docs/design/references/glossary.md`: 128<br>上下文清单（短称）：`docs/design/vision.md`: 148；`docs/design/context.md`: 5, 25 |

### 2.3 英文形态与大小写变体

口径：散文（剔除行内代码与链接 URL）；完整机械表见文末附表 A。

**点名四组以外的具体发现：**

| # | 发现 | 计数 | 位置 / 证据 |
| --- | --- | --- | --- |
| 1 | `Workflow Engine` 大写形态，制度形态为小写 `workflow engine`（`spec/README.md:24`） | 大写 10 处 / 小写 15 处 | `docs/design/spec/system.md`: 14, 31, 36, 162, 179, 180；`docs/design/spec/connections.md`: 163；`docs/design/delivery.md`: 7, 70, 99 |
| 2 | 小写 `id` 与大写 `ID` 并存（如 `run id` 与 `DAG run ID` 同在约束层表格） | id 9 处 / ID 46 处 | 小写 id：`docs/design/spec/connections.md`: 40, 41, 42, 43, 44, 45, 46；`docs/design/spec/project.md`: 58；`docs/design/spec/run.md`: 117 |
| 3 | `Human` 大写 3 处（连接约束的通道标签 `Human scene`/`Human Kanban`），其余 `human` 64 处小写 | Human 3 / human 64 | `docs/design/spec/connections.md`: 44, 45, 117 |
| 4 | 小写 `memo` 指 `.memo/` 过程稿（「专题 memo」等），与领域对象 `Memo`（已发布备忘）共用一词未加区分 | memo 4 处 / Memo 42 处 | `docs/design/README.md`: 80；`docs/design/participant.md`: 54；`docs/design/context.md`: 113；`docs/design/delivery.md`: 56 |
| 5 | `Composer`（输入区组件，project.md:60、delivery.md:13）与 `composer`（contract-tests.md:138「modal/composer」）大小写并存 | 2 / 1 | `docs/design/contract-tests.md:138` |
| 6 | `WebView` 与 `webview`（`spec/system.md:196` 同句两形：`WebView` 与 `window/webview`） | 2 / 1 | `docs/design/spec/system.md:195-196` |
| 7 | `codex resume`（`spec/agent.md:104`）命令示例未用代码格式，与产品词 `Codex` 同形小写 | Codex 5 / codex 1 | `docs/design/spec/agent.md:104` |
| 8 | `Reopen`（命令名，7 处）与 `reopen`（`delivery.md:32` 命令字面量内）| 7 / 1 | 字面值与散文双轨，按 0.9 协议第 3 条属正常；列出供核对 |

**族名/对象名的小写散文用法**（多为字段式搭配，列出供语言通读判断；全量见附表 A）：

| 词 | 小写 | 大写 | 小写代表位置 |
| --- | --- | --- | --- |
| binding | 71 | 71 | `docs/design/architecture.md`: 47, 49, 86；`docs/design/README.md`: 48, 59 |
| revision | 51 | 159 | `docs/design/architecture.md`: 86；`docs/design/references/glossary.md`: 107 |
| terminal | 17 | 46 | `docs/design/agent.md`: 68；`docs/design/spec/system.md`: 36 |
| runtime | 34 | 26 | `docs/design/spec/system.md`: 119, 129, 169, 196；`docs/design/spec/connections.md`: 43, 96, 99, 101, 103, 109, 127, 158× |
| lease | 20 | 23 | `docs/design/references/glossary.md`: 54；`docs/design/spec/system.md`: 80, 169, 191 |
| snapshot | 3 | 42 | `docs/usage.md`: 101；`docs/design/spec/task.md`: 38, 70 |
| manifest | 3 | 45 | `docs/design/spec/system.md`: 53；`docs/design/spec/connections.md`: 40 |
| room | 5 | 208 | `docs/design/spec/project.md`: 14, 29, 67, 69, 107 |
| run | 5 | 297 | `docs/design/spec/connections.md`: 40, 92；`docs/design/spec/run.md`: 16, 41, 79 |
| proposal | 5 | 42 | `docs/design/spec/connections.md`: 43, 55, 109；`docs/design/spec/task.md`: 38 |
| preview | 4 | 19 | `docs/design/run.md`: 50；`docs/design/spec/task.md`: 52 |
| evidence | 10 | 10 | `docs/design/vision.md`: 149；`docs/design/README.md`: 74 |
| spec | 12 | 53 | `docs/design/README.md`: 74×2, 96×2；`docs/design/spec/connections.md`: 43, 103, 109, 115 |
| attempt | 5 | 67 | `docs/design/spec/connections.md`: 42, 92；`docs/design/spec/project.md`: 63 |
| seat | 3 | 54 | `docs/design/spec/connections.md`: 92；`docs/design/spec/run.md`: 16, 79 |
| execution | 13 | 100 | `docs/design/spec/system.md`: 82, 84, 101×2；`docs/design/spec/connections.md`: 63, 129 |
| engine | 15 | 71 | `docs/design/architecture.md`: 33, 44, 59, 87；`docs/design/README.md`: 12×2 |
| workflow | 22 | 76 | `docs/usage.md`: 225；`docs/design/architecture.md`: 33, 44, 59, 87 |
| task | 15 | 361 | `docs/design/architecture.md`: 32, 43×2；`docs/design/references/glossary.md`: 65 |
| project | 2 | 255 | `docs/design/vision.md`: 106；`docs/design/spec/task.md`: 24 |

### 2.4 中英两个名字并存：词汇表对照对在散文中的计数

口径：散文（剔除行内代码与链接 URL）；英文侧按词界匹配，中文侧按子串。注意两类已知放大：定义性括注（「施工图（Workflow）」是 L4 允许的第一次出现格式）与中文常用词膨胀（「版本」「绑定」「证据」「快照」大量是普通名词）。

| 英文形 | 次数 | 中文对照 | 次数 |
| --- | --- | --- | --- |
| Agent | 54 | 执行治理模块 | 1 |
| Harness | 97 | 编码代理工具 | 7 |
| Agency | 81 | 派出方 | 7 |
| Repo | 100 | 仓库 | 41 |
| Project | 255 | 项目 | 17 |
| Room | 208 | 协作聊天室 | 2 |
| Chat Room | 23 | 聊天室场景 | 1 |
| Participant | 68 | 参与者 | 25 |
| Request | 73 | 请求卡 | 5 |
| Memo | 42 | 备忘 | 4 |
| Artifact | 51 | 工件 | 6 |
| Context | 62 | 上下文 | 40 |
| Skill | 36 | 技能包 | 6 |
| Task | 361 | 任务承诺 | 1 |
| Kanban | 30 | 看板 | 16 |
| Run | 297 | 一次受治理施工 | 1 |
| Workflow | 76 | 施工图 | 32 |
| Obligation | 39 | 交付义务 | 11 |
| Seat | 54 | 执行席位 | 3 |
| Attempt | 67 | 执行尝试 | 3 |
| Gate | 38 | 评审关卡 | 7 |
| Verdict | 38 | 裁决 | 30 |
| Receipt | 96 | 凭证 | 28 |
| Terminal | 46 | 终端场景 | 1 |
| ChangeSet | 75 | 变更集 | 16 |
| Evidence | 10 | 证据 | 79 |
| Workbench | 81 | 工作台 | 1 |
| metadata | 35 | 治理元数据 | 4 |
| content | 116 | 场景内容 | 8 |
| artifact | 9 | 结晶 | 51 |
| Task Revision | 53 | 任务契约版本 | 2 |
| Workflow Revision | 23 | 施工图版本 | 2 |
| ChangeSet Revision | 25 | 变更集快照 | 2 |
| Artifact Revision | 9 | 工件版本 | 1 |
| Extension Revision | 4 | 扩展版本 | 1 |
| Engine Deployment | 11 | 引擎部署版本 | 1 |
| Resolved Port Binding | 15 | 端口解析绑定 | 1 |
| Task Binding | 22 | 任务来源绑定 | 1 |
| Project Role Binding | 10 | 角色绑定 | 7 |
| Engine Execution Binding | 17 | 引擎执行绑定 | 1 |
| Write Lease | 9 | 写入租约 | 2 |
| Terminal Input Lease | 9 | 终端输入租约 | 1 |
| Task Source Snapshot | 7 | 来源快照 | 3 |
| Result Proposal | 32 | 结果提案 | 4 |
| Execution Spec | 53 | 派发规格 | 3 |
| Run Manifest | 15 | 施工清单 | 7 |
| Attach Descriptor | 6 | 连接票据 | 3 |
| Context Manifest | 14 | 根上下文清单 | 1 |
| Context Bundle | 13 | 消费上下文包 | 1 |
| Repo Instance | 19 | 仓库实例 | 1 |
| Room Invocation | 44 | 单次调用 | 6 |
| Execution Runtime | 22 | 执行运行时 | 1 |
| Worker Profile | 16 | 执行者配置 | 8 |
| chat server | 46 | 聊天服务器 | 6 |
| task backend | 8 | 任务后端 | 33 |
| workflow engine | 15 | 工作流引擎 | 13 |
| Snapshot | 42 | 快照 | 22 |
| Binding | 71 | 绑定 | 131 |
| Revision | 159 | 版本 | 129 |

制度背景（供阅读，不是豁免结论）：`spec/README.md` 允许六个高频约束词携中文对照进设计正文（Task Revision/任务契约版本、Workflow Revision/施工图版本、Room Invocation/单次调用、Execution Spec/派发规格、Result Proposal/结果提案、Run Manifest/施工清单）。

### 附表 A · 散文中同一小写形态的全部大小写变体（复算用）

口径：散文；词 = `[A-Za-z][A-Za-z0-9_.-]{1,}`；同一小写形态存在 ≥2 种实际大小写形态即列入。

| 小写形态 | 实际形态 × 次数 |
| --- | --- |
| agent | Agent×54、agent×5 |
| artifact | Artifact×51、artifact×9 |
| attach | attach×13、Attach×6 |
| attempt | Attempt×67、attempt×5 |
| binding | binding×71、Binding×71 |
| board | Board×14、board×1 |
| bundle | Bundle×25、bundle×5 |
| cancel | Cancel×4、cancel×2 |
| chat | chat×51、Chat×50 |
| client | client×10、Client×1 |
| closed | closed×6、Closed×1 |
| codex | Codex×5、codex×1 |
| commit | commit×4、Commit×1 |
| complete | Complete×4、complete×1 |
| completion | Completion×10、completion×1 |
| composer | Composer×2、composer×1 |
| create | create×2、Create×1 |
| deadline | deadline×6、Deadline×1 |
| descriptor | descriptor×7、Descriptor×6 |
| engine | Engine×71、engine×15 |
| evidence | evidence×10、Evidence×10 |
| execution | Execution×100、execution×13 |
| git | Git×113、git×3 |
| harness | Harness×97、harness×21 |
| head | head×16、HEAD×3 |
| homeserver | homeserver×17、Homeserver×1 |
| human | human×64、Human×3 |
| id | ID×46、id×9 |
| input | Input×9、input×2 |
| instance | Instance×19、instance×1 |
| integration | Integration×15、integration×5 |
| intent | intent×16、Intent×1 |
| invocation | Invocation×66、invocation×1 |
| issue | Issue×5、issue×1 |
| lease | Lease×23、lease×20 |
| local | local×4、Local×1 |
| manifest | Manifest×45、manifest×3 |
| memo | Memo×42、memo×4 |
| message | Message×6、message×1 |
| move | Move×1、move×1 |
| pause | pause×1、Pause×1 |
| port | Port×15、port×1 |
| preview | Preview×19、preview×4 |
| progress | progress×1、Progress×1 |
| project | Project×255、project×2 |
| prompt | prompt×9、Prompt×3 |
| proposal | Proposal×42、proposal×5 |
| query | Query×3、query×1 |
| ready | READY×1、Ready×1 |
| reject | reject×1、Reject×1 |
| release | Release×4、release×1 |
| reopen | Reopen×7、reopen×1 |
| repo | Repo×100、repo×10 |
| request | Request×73、request×2 |
| restore | Restore×2、restore×1 |
| result | Result×32、result×2 |
| resume | resume×8、Resume×1 |
| retry | Retry×5、retry×4 |
| review | Review×1、review×1 |
| revision | Revision×159、revision×51 |
| role | Role×17、role×3 |
| room | Room×208、room×5 |
| run | Run×297、run×5 |
| runtime | runtime×34、Runtime×26 |
| seat | Seat×54、seat×3 |
| session | session×2、Session×1 |
| shaping | shaping×3、Shaping×1 |
| snapshot | Snapshot×42、snapshot×3 |
| source | source×16、Source×7 |
| spec | Spec×53、spec×12 |
| start | Start×12、start×5 |
| stop | stop×7、Stop×2 |
| task | Task×361、task×15 |
| terminal | Terminal×46、terminal×17 |
| webview | WebView×2、webview×1 |
| worker | Worker×16、worker×5 |
| workflow | Workflow×76、workflow×22 |
| worktree | worktree×35、Worktree×1 |

## 扫描 3 · 长段落候选清单

口径：剔除代码围栏内部、表格行（`|` 开头）与标题行（`#` 开头）后，按行统计字符数（含标点与 Markdown 记号），阈值为 **≥ 300 字符**。该阈值取自行长分布（846 行散文，中位 87，p90≈316），能把明显偏长行找全；是否该拆由语言通读按含义判断。只列位置。

| 文件 | 行号 | 行数 |
| --- | --- | --- |
| `README.md` | 68, 69, 70 | 3 |
| `docs/usage.md` | 20, 141 | 2 |
| `docs/design/architecture.md` | 62 | 1 |
| `docs/design/project.md` | 15, 56 | 2 |
| `docs/design/context.md` | 5 | 1 |
| `docs/design/spec/README.md` | 24, 73 | 2 |
| `docs/design/spec/system.md` | 36, 84, 86, 101, 103, 111, 113, 121, 125, 169 | 10 |
| `docs/design/spec/connections.md` | 57, 103, 109, 119, 125, 127, 129, 172 | 8 |
| `docs/design/spec/project.md` | 38, 40, 42, 46, 56, 61, 63, 65, 67, 73, 79, 81, 87, 95 | 14 |
| `docs/design/spec/task.md` | 20, 24, 26, 28, 30, 32, 34, 36, 38, 50, 64, 66, 68, 70, 78 | 15 |
| `docs/design/spec/run.md` | 37, 41, 61, 63, 67, 79, 81, 85, 99, 101, 105 | 11 |
| `docs/design/spec/agent.md` | 8, 34, 38, 52, 54, 64, 66, 70, 72, 74, 80, 86, 88 | 13 |
| `docs/design/delivery.md` | 87, 122, 125, 126, 135, 141 | 6 |
| **合计** | | **88** |

零命中文件：`docs/design/vision.md`、`docs/design/README.md`、`docs/design/task.md`、`docs/design/run.md`、`docs/design/agent.md`、`docs/design/participant.md`、`docs/design/references/glossary.md`、`docs/design/contract-tests.md`。

## 扫描 4 · 文档头现状表

口径：每文件前 15 行内找 `> 状态：`、`> 日期：` 与「草案 vX.Y.Z」版本戳。根 `README.md` 的基线版本在 `[!IMPORTANT]` 提示块内（第 11 行），非「状态」行格式。

| 文件 | 状态行 | 日期 | 版本戳 |
| --- | --- | --- | --- |
| `README.md` | **无** | 无 | v0.15.4 |
| `docs/usage.md` | **无** | 无 | 无 |
| `docs/design/vision.md` | 规范性（愿景与原则层）· 草案 v0.15.4 | 2026-08-31 | v0.15.4 |
| `docs/design/architecture.md` | 规范性（架构层）· 草案 v0.15.4 | 2026-08-31 | v0.15.4 |
| `docs/design/README.md` | 规范性索引 · 草案 v0.15.4 | 2026-08-31 | v0.15.4 |
| `docs/design/project.md` | **无** | 无 | 无 |
| `docs/design/task.md` | **无** | 无 | 无 |
| `docs/design/run.md` | **无** | 无 | 无 |
| `docs/design/agent.md` | **无** | 无 | 无 |
| `docs/design/participant.md` | 规范性（横切设计正文）· 草案 v0.15.4 | 2026-08-31 | v0.15.4 |
| `docs/design/context.md` | 规范性（横切设计正文）· 草案 v0.15.4 | 2026-08-31 | v0.15.4 |
| `docs/design/references/glossary.md` | 非规范对照 · 草案 v0.15.4 | 无 | v0.15.4 |
| `docs/design/spec/README.md` | 规范性 · 草案 v0.15.4 | 2026-08-31 | v0.15.4 |
| `docs/design/spec/system.md` | 规范性约束 · 草案 v0.15.4 | 无 | v0.15.4 |
| `docs/design/spec/connections.md` | 规范性约束 · 草案 v0.15.4 | 无 | v0.15.4 |
| `docs/design/spec/project.md` | 规范性约束 · 草案 v0.15.4 | 无 | v0.15.4 |
| `docs/design/spec/task.md` | 规范性约束 · 草案 v0.15.4 | 无 | v0.15.4 |
| `docs/design/spec/run.md` | 规范性约束 · 草案 v0.15.4 | 无 | v0.15.4 |
| `docs/design/spec/agent.md` | 规范性约束 · 草案 v0.15.4 | 无 | v0.15.4 |
| `docs/design/delivery.md` | **无** | 无 | 无 |
| `docs/design/contract-tests.md` | 验证文档 · 草案 v0.15.4 | 无 | v0.15.4 |

核实结论：

- 任务书已知的五个缺状态行文件确认：`docs/design/agent.md`、`docs/design/delivery.md`、`docs/design/project.md`、`docs/design/run.md`、`docs/design/task.md`（五份均在标题下带一行「本文是/本文定义……」定位引用块，但均无状态行、日期行与版本戳）。
- 另有两份范围文件无文档头：根 `README.md`（版本戳在 IMPORTANT 块内，v0.15.4）与 `docs/usage.md`（无任何头部行）。L10 字面只要求 `docs/design/` 下文件；这两份是否豁免由结构审计裁决。
- `docs/design/references/glossary.md`、`docs/design/spec/system.md`、`docs/design/spec/connections.md`、`docs/design/spec/project.md`、`docs/design/spec/task.md`、`docs/design/spec/run.md`、`docs/design/spec/agent.md`、`docs/design/contract-tests.md` 共 8 份有状态行与版本戳但**无日期行**；`docs/design/spec/README.md` 有日期。
- 所有已存在的版本戳均为 v0.15.4，与根 README 基线一致。

## 扫描 5 · 中英混排术语清单

提取规则（机械、可复算）：在散文（剔除代码围栏、行内代码与链接 URL）上提取相邻的「英文词 + 中文词」对，两个方向都取；英文侧剔除专名（Git、Matrix、Tuwunel 等产品/平台名）与本库制度词（24 个核心产品词、六族名、五个系统角色名、三类数据名、六个高频约束词、HCTL 及 `provider`/`control`/`adapter`/`human`/`actor` 等制度用法；`workflow engine` 角色名内的 engine 不取，独立的 `Engine …` 照取）；中文侧在第一个功能字（的、和、与、在、是……）处截断，不足 2 字丢弃。这捞的是候选，不判断是否该改——是否违规由语言通读裁决。

### 5a 「英文词 + 中文词」：复现项（≥2 次，31 条）

| 复合候选 | 次数 | 第一次出现 |
| --- | --- | --- |
| Engine 路标 | 4 | `docs/design/spec/run.md:37` |
| step 日志 | 3 | `docs/design/run.md:41` |
| worktree 路径 | 3 | `docs/design/context.md:44` |
| reducer 提交 | 3 | `docs/design/spec/system.md:101` |
| CAS 推进 | 3 | `docs/design/spec/system.md:167` |
| lineage 字段 | 3 | `docs/design/spec/connections.md:92` |
| ID 精确引用消息 | 3 | `docs/design/spec/project.md:28` |
| Web 发行包 | 2 | `docs/usage.md:141` |
| PID 文件 | 2 | `docs/usage.md:165` |
| mention 提交 | 2 | `docs/design/project.md:49` |
| ID 引用讨论 | 2 | `docs/design/project.md:54` |
| ID 读取正文 | 2 | `docs/design/project.md:66` |
| token 估算 | 2 | `docs/design/context.md:104` |
| owner 校验 | 2 | `docs/design/references/glossary.md:55` |
| CI 强制 | 2 | `docs/design/spec/system.md:12` |
| owner 模块 | 2 | `docs/design/spec/system.md:81` |
| payload 自报 | 2 | `docs/design/spec/system.md:101` |
| gap 继续 | 2 | `docs/design/spec/system.md:103` |
| ACK 丢失 | 2 | `docs/design/spec/connections.md:63` |
| owner 身份 | 2 | `docs/design/spec/connections.md:101` |
| CAS 拒绝 | 2 | `docs/design/spec/connections.md:145` |
| homeserver 约束 | 2 | `docs/design/spec/project.md:56` |
| AppService 注册 | 2 | `docs/design/spec/project.md:111` |
| Engine 检查点进入 | 2 | `docs/design/spec/run.md:27` |
| fresh 观察 | 2 | `docs/design/spec/run.md:27` |
| Engine 重试 | 2 | `docs/design/spec/run.md:27` |
| Engine 检查点 | 2 | `docs/design/spec/run.md:41` |
| Engine 自行推进 | 2 | `docs/design/spec/run.md:77` |
| B5 全部发生 | 2 | `docs/design/delivery.md:51` |
| B6 对应 | 2 | `docs/design/delivery.md:51` |
| B1 产品化 | 2 | `docs/design/delivery.md:126` |

### 5b 「英文词 + 中文词」：一次项（205 条，按英文词归组为 118 组）

| 英文词 | 一次项数 | 示例（首见位置） |
| --- | --- | --- |
| owner | 8 | 「owner 模块机械校验身份」 `docs/design/architecture.md:77` |
| Engine | 8 | 「Engine 节点」 `docs/design/spec/connections.md:127` |
| readback | 5 | 「readback 决定」 `docs/design/architecture.md:90` |
| ACK | 5 | 「ACK 丢失保持」 `docs/design/spec/system.md:103` |
| homeserver | 5 | 「homeserver 桥接接入」 `docs/design/spec/project.md:14` |
| Bundle | 5 | 「Bundle 至少固定」 `docs/design/spec/project.md:63` |
| ID | 4 | 「ID 读取」 `docs/usage.md:141` |
| SCM | 4 | 「SCM 评审」 `docs/design/task.md:41` |
| policy | 4 | 「policy 根据允许」 `docs/design/spec/system.md:42` |
| claim | 4 | 「claim 一个」 `docs/design/spec/task.md:28` |
| Chatroom | 3 | 「Chatroom 随包浏览器客户端」 `docs/usage.md:136` |
| UI | 3 | 「UI 调用」 `docs/design/README.md:39` |
| P2 | 3 | 「P2 起承载治理命令」 `docs/design/project.md:61` |
| service | 3 | 「service 提交类型化」 `docs/design/task.md:55` |
| worktree | 3 | 「worktree 保全」 `docs/design/agent.md:61` |
| reducer | 3 | 「reducer 赋予」 `docs/design/spec/system.md:101` |
| key | 3 | 「key 投递」 `docs/design/spec/system.md:103` |
| fork | 3 | 「fork 语义」 `docs/design/spec/system.md:121` |
| clone | 3 | 「clone 本地」 `docs/design/spec/system.md:125` |
| pointer | 3 | 「pointer 必须经唯一」 `docs/design/spec/system.md:141` |
| writer | 3 | 「writer 协调」 `docs/design/spec/system.md:189` |
| CAS | 3 | 「CAS 校验活跃」 `docs/design/spec/connections.md:57` |
| cursor | 3 | 「cursor 重建」 `docs/design/spec/connections.md:166` |
| Start | 3 | 「Start 拒绝」 `docs/design/spec/task.md:64` |
| envelope | 2 | 「envelope 类型」 `docs/usage.md:11` |
| PID | 2 | 「PID 对应」 `docs/usage.md:129` |
| Release | 2 | 「Release 提供」 `docs/usage.md:204` |
| schema | 2 | 「schema 字段」 `docs/design/spec/README.md:18` |
| hook | 2 | 「hook 优先级」 `docs/design/spec/system.md:49` |
| account | 2 | 「account 回写造」 `docs/design/spec/system.md:84` |
| JCS | 2 | 「JCS 规范对象」 `docs/design/spec/system.md:107` |
| fresh | 2 | 「fresh 消息」 `docs/design/spec/system.md:160` |
| reconcile | 2 | 「reconcile 原子」 `docs/design/spec/connections.md:57` |
| fence | 2 | 「fence 必须记录」 `docs/design/spec/connections.md:97` |
| Retry | 2 | 「Retry 必须」 `docs/design/spec/project.md:79` |
| fan-out | 2 | 「fan-out 位置」 `docs/design/spec/project.md:81` |
| mention | 2 | 「mention 字符串交给模型猜测路」 `docs/design/spec/project.md:97` |
| ReviewSubjectRef | 2 | 「ReviewSubjectRef 所指」 `docs/design/spec/run.md:81` |
| quorum-unreachable | 2 | 「quorum-unreachable 结果」 `docs/design/spec/run.md:101` |
| P3 | 2 | 「P3 出门条件」 `docs/design/delivery.md:9` |
| P0 | 2 | 「P0 探针」 `docs/design/delivery.md:122` |
| P1 | 1 | 「P1 骨架」 `docs/usage.md:10` |
| shell | 1 | 「shell 加入」 `docs/usage.md:39` |
| socket | 1 | 「socket 权限」 `docs/usage.md:101` |
| Homeserver | 1 | 「Homeserver 固定」 `docs/usage.md:141` |
| Issue | 1 | 「Issue 关闭」 `docs/design/vision.md:36` |
| client-server | 1 | 「client-server 实质」 `docs/design/architecture.md:17` |
| prompt | 1 | 「prompt 习惯」 `docs/design/participant.md:41` |
| step | 1 | 「step 日志里没」 `docs/design/context.md:82` |
| AI | 1 | 「AI 协作者用」 `docs/design/spec/README.md:24` |
| BPMN | 1 | 「BPMN 对齐」 `docs/design/spec/README.md:81` |
| transport | 1 | 「transport 必须认证客户端」 `docs/design/spec/system.md:36` |
| hctl2-control | 1 | 「hctl2-control 托管执行面服务器」 `docs/design/spec/system.md:38` |
| discovery | 1 | 「discovery 默认」 `docs/design/spec/system.md:42` |
| store | 1 | 「store 条目」 `docs/design/spec/system.md:49` |
| gap | 1 | 「gap 必须返回类型化拒绝」 `docs/design/spec/system.md:103` |
| operation | 1 | 「operation 共用」 `docs/design/spec/system.md:111` |
| common-dir | 1 | 「common-dir 重试返回原现场」 `docs/design/spec/system.md:121` |
| scope | 1 | 「scope 至少覆盖」 `docs/design/spec/system.md:169` |
| CSP | 1 | 「CSP 拒绝远程」 `docs/design/spec/system.md:196` |
| content-first | 1 | 「content-first 卡片」 `docs/design/spec/connections.md:57` |
| rejection | 1 | 「rejection 列出缺项」 `docs/design/spec/connections.md:97` |
| supersedes | 1 | 「supersedes 连接新」 `docs/design/spec/project.md:33` |
| Publish | 1 | 「Publish 推进」 `docs/design/spec/project.md:34` |
| URL | 1 | 「URL 相似」 `docs/design/spec/project.md:38` |
| inline | 1 | 「inline 物化原文」 `docs/design/spec/project.md:61` |
| compressor | 1 | 「compressor 模型」 `docs/design/spec/project.md:63` |
| token | 1 | 「token 估算量」 `docs/design/spec/project.md:63` |
| payload | 1 | 「payload 改写」 `docs/design/spec/project.md:81` |
| Project-scoped | 1 | 「Project-scoped 权限必须选择精确」 `docs/design/spec/project.md:81` |
| bot | 1 | 「bot 账号」 `docs/design/spec/project.md:112` |
| task-source | 1 | 「task-source 绑定元数据」 `docs/design/spec/task.md:26` |
| anchor | 1 | 「anchor 稳定回读归属」 `docs/design/spec/task.md:26` |
| drift | 1 | 「drift 一律」 `docs/design/spec/task.md:66` |
| current | 1 | 「current 投影」 `docs/design/spec/task.md:70` |
| ProjectV2 | 1 | 「ProjectV2 排序」 `docs/design/spec/task.md:92` |
| sortOrder | 1 | 「sortOrder 派生」 `docs/design/spec/task.md:93` |
| answer | 1 | 「answer 先进入公共」 `docs/design/spec/run.md:33` |
| Prompt | 1 | 「Prompt 声明」 `docs/design/spec/run.md:57` |
| lint | 1 | 「lint 拒绝格式」 `docs/design/spec/run.md:67` |
| loop | 1 | 「loop 终止」 `docs/design/spec/run.md:67` |
| pending | 1 | 「pending 阻止新」 `docs/design/spec/run.md:105` |
| definition | 1 | 「definition 版本」 `docs/design/spec/run.md:114` |
| principal | 1 | 「principal 运行」 `docs/design/spec/agent.md:34` |
| stdin | 1 | 「stdin 历史」 `docs/design/spec/agent.md:34` |
| baseline | 1 | 「baseline 创建新物理」 `docs/design/spec/agent.md:38` |
| commit | 1 | 「commit 包装」 `docs/design/spec/agent.md:52` |
| tree | 1 | 「tree 变化创建新」 `docs/design/spec/agent.md:52` |
| separation | 1 | 「separation 必须沿它解析」 `docs/design/spec/agent.md:52` |
| repo-scoped | 1 | 「repo-scoped 调用」 `docs/design/spec/agent.md:62` |
| stop | 1 | 「stop 结果」 `docs/design/spec/agent.md:74` |
| dispatch | 1 | 「dispatch 权限」 `docs/design/spec/agent.md:82` |
| action | 1 | 「action 写回」 `docs/design/spec/agent.md:88` |
| pre-commit | 1 | 「pre-commit 一类」 `docs/design/delivery.md:56` |
| B5 | 1 | 「B5 晋级」 `docs/design/delivery.md:58` |
| signal | 1 | 「signal 回原执行」 `docs/design/delivery.md:77` |
| B1 | 1 | 「B1 首次消费」 `docs/design/delivery.md:122` |
| DAG | 1 | 「DAG 提交」 `docs/design/delivery.md:124` |
| tombstone | 1 | 「tombstone 验证延至」 `docs/design/delivery.md:128` |
| Mach-O | 1 | 「Mach-O 均声明」 `docs/design/delivery.md:135` |
| runner | 1 | 「runner 验证」 `docs/design/delivery.md:135` |
| Darwin | 1 | 「Darwin 制品」 `docs/design/delivery.md:135` |
| SHA-256 | 1 | 「SHA-256 锁定」 `docs/design/delivery.md:135` |
| target | 1 | 「target 共用锁定」 `docs/design/delivery.md:135` |
| GPUI | 1 | 「GPUI 原生备选」 `docs/design/delivery.md:141` |
| CJK | 1 | 「CJK 输入」 `docs/design/contract-tests.md:13` |
| lane | 1 | 「lane 投影」 `docs/design/contract-tests.md:32` |
| adoption | 1 | 「adoption 混用」 `docs/design/contract-tests.md:35` |
| doer | 1 | 「doer 映射」 `docs/design/contract-tests.md:39` |
| expiry | 1 | 「expiry 产生明确」 `docs/design/contract-tests.md:54` |
| backup | 1 | 「backup 改变参」 `docs/design/contract-tests.md:57` |
| target-head | 1 | 「target-head 竞争」 `docs/design/contract-tests.md:66` |
| Share | 1 | 「Share 均拒绝」 `docs/design/contract-tests.md:68` |
| controller | 1 | 「controller 写入」 `docs/design/contract-tests.md:74` |
| handoff | 1 | 「handoff 固定」 `docs/design/contract-tests.md:86` |
| consumer | 1 | 「consumer 冻结对应」 `docs/design/contract-tests.md:87` |
| bridge | 1 | 「bridge 接入」 `docs/design/contract-tests.md:99` |
| link | 1 | 「link 保留返回路径」 `docs/design/contract-tests.md:129` |

### 5c 「中文词 + 英文词」：复现项（≥2 次，61 条）

| 复合候选 | 次数 | 第一次出现 |
| --- | --- | --- |
| 草案 v0 | 15 | `README.md:11` |
| 远端 SCM | 5 | `docs/design/spec/system.md:12` |
| 外部 Issue | 3 | `docs/design/vision.md:36` |
| 事件 ID | 3 | `docs/design/spec/project.md:28` |
| 按事件 ID | 3 | `docs/design/spec/project.md:56` |
| 写唯一 Integration | 3 | `docs/design/spec/agent.md:54` |
| 在同一 Release | 2 | `docs/usage.md:29` |
| 按消息 ID | 2 | `docs/usage.md:141` |
| 仓库 clone | 2 | `docs/design/architecture.md:17` |
| 平台 homeserver | 2 | `docs/design/architecture.md:42` |
| 户端侧 transport | 2 | `docs/design/architecture.md:47` |
| 第三方 UI | 2 | `docs/design/README.md:39` |
| 息事件 ID | 2 | `docs/design/project.md:54` |
| 的直接 mutation | 2 | `docs/design/run.md:37` |
| 终端 trace | 2 | `docs/design/agent.md:22` |
| 部会话 ID | 2 | `docs/design/agent.md:41` |
| 有显式 Share | 2 | `docs/design/agent.md:48` |
| 花模型 token | 2 | `docs/design/context.md:17` |
| 对象 worktree | 2 | `docs/design/context.md:44` |
| 平台经 homeserver | 2 | `docs/design/spec/system.md:29` |
| 内部 reducer | 2 | `docs/design/spec/system.md:101` |
| 按同一 key | 2 | `docs/design/spec/system.md:103` |
| 和已知 gap | 2 | `docs/design/spec/system.md:103` |
| 不匹配 drift | 2 | `docs/design/spec/system.md:113` |
| 准入 current | 2 | `docs/design/spec/system.md:125` |
| 共享 policy | 2 | `docs/design/spec/system.md:147` |
| 不依赖 fresh | 2 | `docs/design/spec/system.md:160` |
| 未声明 IPC | 2 | `docs/design/spec/system.md:196` |
| 是语义 owner | 2 | `docs/design/spec/connections.md:101` |
| 础设施 fence | 2 | `docs/design/spec/connections.md:101` |
| 为固定 owner | 2 | `docs/design/spec/connections.md:103` |
| 的缩减 tuple | 2 | `docs/design/spec/connections.md:109` |
| 与来源 blocker | 2 | `docs/design/spec/connections.md:127` |
| 需 fresh | 2 | `docs/design/spec/connections.md:160` |
| 不要求 fresh | 2 | `docs/design/spec/connections.md:162` |
| 外部 account | 2 | `docs/design/spec/project.md:14` |
| 动作 allowlist | 2 | `docs/design/spec/project.md:14` |
| 事务 ID | 2 | `docs/design/spec/project.md:28` |
| 或完整 cursor | 2 | `docs/design/spec/project.md:56` |
| 消费者 Bundle | 2 | `docs/design/spec/project.md:63` |
| 在匹配 ACK | 2 | `docs/design/spec/project.md:85` |
| 其他 Start | 2 | `docs/design/spec/task.md:64` |
| 务清除 claim | 2 | `docs/design/spec/task.md:64` |
| 原生 UI | 2 | `docs/design/spec/run.md:35` |
| 已绑定 Engine | 2 | `docs/design/spec/run.md:35` |
| 隔离旧 worktree | 2 | `docs/design/spec/run.md:39` |
| 观察 Engine | 2 | `docs/design/spec/run.md:73` |
| 并完整 regate | 2 | `docs/design/spec/run.md:94` |
| 取得集 authority | 2 | `docs/design/spec/agent.md:34` |
| 证明旧 writer | 2 | `docs/design/spec/agent.md:38` |
| 不产生 Integration | 2 | `docs/design/spec/agent.md:54` |
| 直连 transport | 2 | `docs/design/spec/agent.md:72` |
| 得声称 exact | 2 | `docs/design/spec/agent.md:100` |
| 部发生 P2 | 2 | `docs/design/delivery.md:51` |
| 对应 P3 | 2 | `docs/design/delivery.md:51` |
| 型决定 decision-history | 2 | `docs/design/delivery.md:55` |
| 在隔 worktree | 2 | `docs/design/delivery.md:64` |
| 验精确 Integration | 2 | `docs/design/delivery.md:67` |
| 执行 readback | 2 | `docs/design/delivery.md:81` |
| 与原生 controller | 2 | `docs/design/delivery.md:125` |
| 周期留 B1 | 2 | `docs/design/delivery.md:126` |

### 5d 「中文词 + 英文词」：一次项（575 条，按英文词归组为 217 组）

| 英文词 | 一次项数 | 示例（首见位置） |
| --- | --- | --- |
| owner | 37 | 「定唯一 owner」 `docs/design/README.md:78` |
| Engine | 19 | 「行顺序 Engine」 `docs/design/vision.md:155` |
| fence | 16 | 「锁 fence」 `docs/design/spec/system.md:12` |
| worktree | 14 | 「面板 worktree」 `docs/design/vision.md:28` |
| ID | 13 | 「用稳定 ID」 `docs/design/README.md:52` |
| fresh | 11 | 「来源链 fresh」 `docs/design/architecture.md:85` |
| CAS | 11 | 「在账本 CAS」 `docs/design/spec/system.md:169` |
| claim | 10 | 「原子 claim」 `docs/design/spec/connections.md:57` |
| current | 9 | 「依赖 current」 `docs/design/architecture.md:86` |
| clone | 8 | 「属某个 clone」 `docs/design/architecture.md:66` |
| token | 8 | 「史也烧 token」 `docs/design/context.md:11` |
| producer | 8 | 「里 producer」 `docs/design/context.md:29` |
| drift | 8 | 「或变化 drift」 `docs/design/spec/task.md:66` |
| prompt | 7 | 「天塞进 prompt」 `docs/design/context.md:9` |
| schema | 7 | 「的协议 schema」 `docs/design/spec/README.md:18` |
| ACK | 7 | 「超 ACK」 `docs/design/spec/system.md:103` |
| writer | 7 | 「另一个 writer」 `docs/design/spec/system.md:105` |
| PID | 6 | 「时显示 PID」 `docs/usage.md:91` |
| secret | 6 | 「机本地 secret」 `docs/usage.md:141` |
| homeserver | 6 | 「受任意 homeserver」 `docs/usage.md:176` |
| Chatroom | 5 | 「以运行 Chatroom」 `docs/usage.md:3` |
| fail | 5 | 「成命令 fail」 `docs/design/architecture.md:86` |
| ground | 5 | 「结果卡 ground」 `docs/design/project.md:62` |
| trace | 5 | 「和终端 trace」 `docs/design/run.md:41` |
| cursor | 5 | 「重同步 cursor」 `docs/design/spec/system.md:51` |
| blocker | 5 | 「造语义 blocker」 `docs/design/spec/connections.md:125` |
| placement | 5 | 「需 placement」 `docs/design/spec/connections.md:162` |
| Start | 5 | 「绝另一 Start」 `docs/design/spec/task.md:64` |
| target | 4 | 「每个 target」 `docs/usage.md:22` |
| fan-out | 4 | 「临场 fan-out」 `docs/design/spec/system.md:101` |
| tuple | 4 | 「按完整 tuple」 `docs/design/spec/connections.md:99` |
| pointer | 4 | 「算时改 pointer」 `docs/design/spec/project.md:61` |
| Bundle | 4 | 「类内容 Bundle」 `docs/design/spec/project.md:67` |
| subject | 4 | 「的评审 subject」 `docs/design/spec/project.md:73` |
| stage | 4 | 「非终态 stage」 `docs/design/spec/task.md:20` |
| ReviewSubjectRef | 4 | 「确绑定 ReviewSubjectRef」 `docs/design/spec/run.md:29` |
| payload | 3 | 「行归档 payload」 `docs/usage.md:51` |
| loopback | 3 | 「只监听 loopback」 `docs/usage.md:141` |
| client-server | 3 | 「在结构 client-server」 `docs/design/vision.md:134` |
| known | 3 | 「证的标 known」 `docs/design/participant.md:35` |
| bytes | 3 | 「具版本 bytes」 `docs/design/references/glossary.md:129` |
| direct | 3 | 「可来自 direct」 `docs/design/spec/README.md:38` |
| immutable | 3 | 「并回读 immutable」 `docs/design/spec/system.md:147` |
| typed | 3 | 「露具名 typed」 `docs/design/spec/system.md:196` |
| lineage | 3 | 「准建议 lineage」 `docs/design/spec/connections.md:92` |
| signal | 3 | 「及唯一 signal」 `docs/design/spec/connections.md:127` |
| parent | 3 | 「须冻结 parent」 `docs/design/spec/project.md:52` |
| tombstone | 3 | 「新事件 tombstone」 `docs/design/spec/project.md:54` |
| replay | 3 | 「得声称 replay」 `docs/design/spec/project.md:63` |
| DAG | 3 | 「有持久 DAG」 `docs/design/spec/project.md:77` |
| stop | 3 | 「提交 stop」 `docs/design/spec/project.md:79` |
| delivery | 3 | 「写唯一 delivery」 `docs/design/spec/project.md:85` |
| active | 3 | 「析一个 active」 `docs/design/spec/task.md:16` |
| anchor | 3 | 「个获准 anchor」 `docs/design/spec/task.md:26` |
| adoption | 3 | 「不经 adoption」 `docs/design/spec/task.md:30` |
| group | 3 | 「在多个 group」 `docs/design/spec/task.md:36` |
| reducer | 3 | 「确定性 reducer」 `docs/design/spec/run.md:37` |
| required | 3 | 「的全部 required」 `docs/design/spec/run.md:37` |
| Prompt | 3 | 「后再依 Prompt」 `docs/design/spec/run.md:57` |
| reviewer | 3 | 「用必需 reviewer」 `docs/design/spec/run.md:99` |
| hook | 3 | 「事件 hook」 `docs/design/spec/agent.md:70` |
| provenance | 3 | 「明输入 provenance」 `docs/design/spec/agent.md:72` |
| B2 | 3 | 「不阻塞 B2」 `docs/design/delivery.md:57` |
| integration | 3 | 「与证据 integration」 `docs/design/delivery.md:66` |
| Coding | 2 | 「用多个 Coding」 `README.md:3` |
| envelope | 2 | 「供共享 envelope」 `docs/usage.md:11` |
| shell | 2 | 「以为当 shell」 `docs/usage.md:39` |
| socket | 2 | 「非托管 socket」 `docs/usage.md:91` |
| Web | 2 | 「是官方 Web」 `docs/usage.md:141` |
| shim | 2 | 「的独立 shim」 `docs/design/architecture.md:38` |
| transport | 2 | 「侧终端 transport」 `docs/design/architecture.md:45` |
| mention | 2 | 「所 mention」 `docs/design/project.md:54` |
| SCM | 2 | 「受外部 SCM」 `docs/design/task.md:41` |
| unknown | 2 | 「能的标 unknown」 `docs/design/participant.md:35` |
| step | 2 | 「路标 step」 `docs/design/context.md:29` |
| diff | 2 | 「的被评 diff」 `docs/design/context.md:40` |
| definition | 2 | 「已配置 definition」 `docs/design/spec/system.md:42` |
| authority | 2 | 「应模块 authority」 `docs/design/spec/system.md:68` |
| key | 2 | 「用同一 key」 `docs/design/spec/system.md:103` |
| cached | 2 | 「确允许 cached」 `docs/design/spec/system.md:103` |
| SHA-256 | 2 | 「范对象 SHA-256」 `docs/design/spec/system.md:107` |
| conflict | 2 | 「声明 conflict」 `docs/design/spec/system.md:111` |
| expected | 2 | 「只表现 expected」 `docs/design/spec/system.md:113` |
| HEAD | 2 | 「碰巧相 HEAD」 `docs/design/spec/system.md:121` |
| common-dir | 2 | 「一个 common-dir」 `docs/design/spec/system.md:121` |
| remote | 2 | 「禁止 remote」 `docs/design/spec/system.md:196` |
| kind | 2 | 「少包含 kind」 `docs/design/spec/connections.md:16` |
| create | 2 | 「化后端 create」 `docs/design/spec/connections.md:39` |
| correlation | 2 | 「和稳定 correlation」 `docs/design/spec/connections.md:57` |
| reparent | 2 | 「能创建 reparent」 `docs/design/spec/connections.md:57` |
| scope | 2 | 「侧固定 scope」 `docs/design/spec/connections.md:92` |
| Write | 2 | 「适用 Write」 `docs/design/spec/connections.md:98` |
| mutation | 2 | 「原生 mutation」 `docs/design/spec/connections.md:172` |
| stable | 2 | 「只携带 stable」 `docs/design/spec/connections.md:174` |
| deadline | 2 | 「命令 deadline」 `docs/design/spec/project.md:31` |
| post-train | 2 | 「可选 post-train」 `docs/design/spec/project.md:46` |
| inline | 2 | 「档记录 inline」 `docs/design/spec/project.md:61` |
| locator | 2 | 「须保留 locator」 `docs/design/spec/project.md:63` |
| compressor | 2 | 「须记录 compressor」 `docs/design/spec/project.md:67` |
| admission | 2 | 「进相应 admission」 `docs/design/spec/task.md:34` |
| refresh | 2 | 「持久化 refresh」 `docs/design/spec/task.md:61` |
| idempotency | 2 | 「同一 idempotency」 `docs/design/spec/task.md:64` |
| divergence | 2 | 「定显式 divergence」 `docs/design/spec/task.md:70` |
| freshness | 2 | 「出冻结 freshness」 `docs/design/spec/task.md:80` |
| Integration | 2 | 「于后续 Integration」 `docs/design/spec/agent.md:52` |
| commit | 2 | 「增加 commit」 `docs/design/spec/agent.md:52` |
| semantic | 2 | 「能报告 semantic」 `docs/design/spec/agent.md:74` |
| sequence | 2 | 「来源流 sequence」 `docs/design/spec/agent.md:92` |
| B0 | 2 | 「与覆盖 B0」 `docs/design/delivery.md:57` |
| webhook | 2 | 「赖公网 webhook」 `docs/design/delivery.md:87` |
| doer | 2 | 「变化 doer」 `docs/design/delivery.md:87` |
| P0 | 2 | 「各依赖 P0」 `docs/design/delivery.md:122` |
| Release | 1 | 「于同一 Release」 `docs/usage.md:20` |
| CI | 1 | 「康检查 CI」 `docs/usage.md:93` |
| hash | 1 | 「并启用 hash」 `docs/usage.md:101` |
| federation | 1 | 「置禁用 federation」 `docs/usage.md:141` |
| foreign | 1 | 「时报告 foreign」 `docs/usage.md:174` |
| arm64 | 1 | 「并测试 arm64」 `docs/usage.md:225` |
| lock | 1 | 「要写回 lock」 `docs/usage.md:225` |
| r2 | 1 | 「续讨论 r2」 `docs/design/vision.md:100` |
| SaaS | 1 | 「至外部 SaaS」 `docs/design/vision.md:109` |
| LLM | 1 | 「新实现 LLM」 `docs/design/vision.md:162` |
| head | 1 | 「来源 head」 `docs/design/architecture.md:86` |
| READY | 1 | 「部节点 READY」 `docs/design/run.md:11` |
| hunk | 1 | 「给精确 hunk」 `docs/design/context.md:84` |
| AI | 1 | 「散文 AI」 `docs/design/spec/README.md:24` |
| README | 1 | 「含仓库 README」 `docs/design/spec/README.md:26` |
| trusted | 1 | 「据允许 trusted」 `docs/design/spec/system.md:42` |
| registry | 1 | 「扩展 registry」 `docs/design/spec/system.md:42` |
| Extension | 1 | 「产生新 Extension」 `docs/design/spec/system.md:42` |
| UI | 1 | 「优先级 UI」 `docs/design/spec/system.md:49` |
| service | 1 | 「记 service」 `docs/design/spec/system.md:84` |
| RBAC | 1 | 「或复杂 RBAC」 `docs/design/spec/system.md:86` |
| RFC | 1 | 「一使用 RFC」 `docs/design/spec/system.md:107` |
| close | 1 | 「一资源 close」 `docs/design/spec/system.md:111` |
| principal | 1 | 「的窄 principal」 `docs/design/spec/system.md:111` |
| expected-version | 1 | 「做 expected-version」 `docs/design/spec/system.md:141` |
| cache | 1 | 「可丢弃 cache」 `docs/design/spec/system.md:189` |
| ledger | 1 | 「保留原 ledger」 `docs/design/spec/system.md:191` |
| WebView | 1 | 「桌面壳 WebView」 `docs/design/spec/system.md:195` |
| raw | 1 | 「不暴露 raw」 `docs/design/spec/system.md:196` |
| id | 1 | 「命令 id」 `docs/design/spec/connections.md:45` |
| deep | 1 | 「景卡片 deep」 `docs/design/spec/connections.md:174` |
| projection | 1 | 「可重建 projection」 `docs/design/spec/connections.md:174` |
| supersedes | 1 | 「更新 supersedes」 `docs/design/spec/project.md:33` |
| abandoned | 1 | 「显式 abandoned」 `docs/design/spec/project.md:52` |
| recall | 1 | 「行中经 recall」 `docs/design/spec/project.md:61` |
| materialized | 1 | 「按序 materialized」 `docs/design/spec/project.md:63` |
| retention | 1 | 「终态 retention」 `docs/design/spec/project.md:63` |
| Retry | 1 | 「用户 Retry」 `docs/design/spec/project.md:79` |
| label | 1 | 「或获准 label」 `docs/design/spec/task.md:26` |
| board-item | 1 | 「或更换 board-item」 `docs/design/spec/task.md:32` |
| HCTL-first | 1 | 「并发 HCTL-first」 `docs/design/spec/task.md:34` |
| Adopt | 1 | 「它阻止 Adopt」 `docs/design/spec/task.md:36` |
| refs | 1 | 「来源 refs」 `docs/design/spec/task.md:38` |
| contract | 1 | 「预期 contract」 `docs/design/spec/task.md:38` |
| task-bound | 1 | 「有一个 task-bound」 `docs/design/spec/task.md:64` |
| pass | 1 | 「项各自 pass」 `docs/design/spec/task.md:70` |
| Reopen | 1 | 「让外部 Reopen」 `docs/design/spec/task.md:74` |
| reject | 1 | 「不能被 reject」 `docs/design/spec/task.md:78` |
| compiler | 1 | 「固定 compiler」 `docs/design/spec/run.md:24` |
| success | 1 | 「时应停 success」 `docs/design/spec/run.md:37` |
| dispatch | 1 | 「撤销旧 dispatch」 `docs/design/spec/run.md:39` |
| owner-specific | 1 | 「写租约 owner-specific」 `docs/design/spec/run.md:63` |
| lint | 1 | 「图结构 lint」 `docs/design/spec/run.md:67` |
| recipient | 1 | 「能新增 recipient」 `docs/design/spec/run.md:67` |
| fork | 1 | 「时整次 fork」 `docs/design/spec/run.md:67` |
| loop | 1 | 「明任意 loop」 `docs/design/spec/run.md:67` |
| operator | 1 | 「同人类 operator」 `docs/design/spec/run.md:101` |
| unsupported | 1 | 「判 unsupported」 `docs/design/spec/run.md:101` |
| quorum-unreachable | 1 | 「类型化 quorum-unreachable」 `docs/design/spec/run.md:101` |
| retry | 1 | 「引擎 retry」 `docs/design/spec/run.md:118` |
| stdin | 1 | 「普通 stdin」 `docs/design/spec/agent.md:34` |
| baseline | 1 | 「从获准 baseline」 `docs/design/spec/agent.md:38` |
| review | 1 | 「不进入 review」 `docs/design/spec/agent.md:52` |
| push | 1 | 「远端 push」 `docs/design/spec/agent.md:54` |
| start | 1 | 「已获准 start」 `docs/design/spec/agent.md:70` |
| controller | 1 | 「闭原生 controller」 `docs/design/spec/agent.md:72` |
| output | 1 | 「有冻结 output」 `docs/design/spec/agent.md:80` |
| conversation | 1 | 「有独立 conversation」 `docs/design/spec/agent.md:88` |
| structured | 1 | 「确降级 structured」 `docs/design/spec/agent.md:88` |
| exact | 1 | 「能声称 exact」 `docs/design/spec/agent.md:92` |
| readback | 1 | 「集 readback」 `docs/design/delivery.md:66` |
| reconcile | 1 | 「与定期 reconcile」 `docs/design/delivery.md:87` |
| B6 | 1 | 「必须通 B6」 `docs/design/delivery.md:105` |
| ADR | 1 | 「不另造 ADR」 `docs/design/delivery.md:118` |
| barrier | 1 | 「是全局 barrier」 `docs/design/delivery.md:122` |
| B1 | 1 | 「探针 B1」 `docs/design/delivery.md:122` |
| B4 | 1 | 「针只须 B4」 `docs/design/delivery.md:122` |
| resize | 1 | 「输入 resize」 `docs/design/delivery.md:125` |
| ring | 1 | 「事件 ring」 `docs/design/delivery.md:125` |
| P2 | 1 | 「证延至 P2」 `docs/design/delivery.md:128` |
| Mach-O | 1 | 「种架构 Mach-O」 `docs/design/delivery.md:135` |
| release | 1 | 「依赖 release」 `docs/design/delivery.md:135` |
| Darwin | 1 | 「上游 Darwin」 `docs/design/delivery.md:135` |
| Docker | 1 | 「需安装 Docker」 `docs/design/delivery.md:136` |
| footprint | 1 | 「实测 footprint」 `docs/design/delivery.md:141` |
| vendor | 1 | 「移植 vendor」 `docs/design/delivery.md:143` |
| license | 1 | 「保留 license」 `docs/design/delivery.md:143` |
| family | 1 | 「个稳定 family」 `docs/design/contract-tests.md:6` |
| fail-closed | 1 | 「同一条 fail-closed」 `docs/design/contract-tests.md:18` |
| move | 1 | 「非法 move」 `docs/design/contract-tests.md:33` |
| jurisdictional | 1 | 「动只形 jurisdictional」 `docs/design/contract-tests.md:43` |
| credential | 1 | 「人类 credential」 `docs/design/contract-tests.md:70` |
| exit | 1 | 「缺失 exit」 `docs/design/contract-tests.md:76` |
| handoff | 1 | 「每条 handoff」 `docs/design/contract-tests.md:86` |
| consumer | 1 | 「个实际 consumer」 `docs/design/contract-tests.md:87` |
| effect | 1 | 「外部 effect」 `docs/design/contract-tests.md:91` |
| backup | 1 | 「一致性 backup」 `docs/design/contract-tests.md:111` |
| JCS | 1 | 「价对象 JCS」 `docs/design/contract-tests.md:117` |
| trust | 1 | 「自声明 trust」 `docs/design/contract-tests.md:122` |
| discovery | 1 | 「副作用 discovery」 `docs/design/contract-tests.md:122` |
| install | 1 | 「静默 install」 `docs/design/contract-tests.md:122` |
| capability | 1 | 「未声明 capability」 `docs/design/contract-tests.md:123` |
| renderer | 1 | 「网形态 renderer」 `docs/design/contract-tests.md:123` |
| mouse | 1 | 「浏览 mouse」 `docs/design/contract-tests.md:137` |
| IME | 1 | 「优先级 IME」 `docs/design/contract-tests.md:138` |

已知噪声（诚实标注）：① 阶段标签（P0/P2/B1/B5…）与缩写（ID、CI、PID、ACK…）后接中文语法时会被计入，如「B5 全部发生」；② 中文侧截断可能切在多字词中间（如「系统角…」类），位置信息不受影响；③ 「草案 v0」类是版本戳格式本身。这些由规则定义决定，不是漏扫。
## 基线文档检查结果

扫描完成后于本 worktree 实跑（含本文件）：

```text
$ cd src && build/docs/materialize_repo_tree.sh && ./buck2 test root//build/docs/...
materialize_repo_tree: copied 171 files into build/docs/repo_tree/
✓ Pass: root//build/docs:check_version_stamps   （baseline is v0.15.4）
✓ Pass: root//build/docs:check_dead_names
✓ Pass: root//build/docs:check_memo_review_baseline
Tests finished: Pass 5. Fail 0. Timeout 0. Fatal 0. Skip 0. Omit 0.
```

基线为绿，与任务书预期（Pass 5）一致。

## 对指南的异议

无。指南 S1–S3、L7 与任务书「松紧基准」在本轮机械扫描中自洽，未发现需要所有者裁决的指南层问题。两处执行层口径已在对应节内注明：「需要」实测 80 处（任务书口径 82，差异复核见扫描 1）；「应当/不应」的词内命中剔除规则在本基线无需触发。

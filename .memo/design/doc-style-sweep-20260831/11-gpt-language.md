# GPT · 机制层与交付层语言通读

> 审计基线：`origin/main` @ `2863632`（草案 v0.15.4）<br>
> 审计范围：任务书指定的 9 个机制层与交付层文件<br>
> 审计身份：未参与 HCTL2 设计讨论的资深工程师

本报告只检查语言是否能让首次接触 HCTL2 的工程师准确复述规则；不改设计、不改 `docs/`。条号沿用 `WRITING-GUIDE.md`。

## `docs/design/spec/README.md`

| # | 文件:行 | 条号 | 原文（截 40 字） | 问题 | 建议改法 |
| --- | --- | --- | --- | --- | --- |
| 1 | `spec/README.md:14` | T1 | 但需要被精确引用 | “需要”既可表示客观必要，也可表示规范义务；这里实际是在定义票据的可引用性质。 | 改为：“某个步骤产生的只追加、短期或一次性记录，而且后续规则可以精确引用它。” |
| 2 | `spec/README.md:18` | L5 | 新名字的引入门槛……设计层正文不使用 | 一段同时规定命名门槛、专名写法、代码词形和设计层词汇边界，读者无法先抓到适用于当前写作动作的规则。 | 改为四句：“只有领域对象、票据和记录可以引入新名字。具名对象和票据使用带空格的专名，命令使用动宾语义名，状态值使用中文语义名。实现标识符另建‘语义名 ↔ 标识符’对照表。设计层正文只使用下节列出的核心产品词。” |
| 3 | `spec/README.md:24` | S2 | 另设五个系统角色名……provider 只作 | 一句话连续引入五个角色、Agent 的专用含义和 provider 的限制，后两条容易被角色清单淹没。 | 改为：“设计正文还可以使用五个系统角色名：harness、chat server、task backend、workflow engine 和 Agency。Agent 专指第四模块；描述 AI 协作者时使用 Participant。provider 只是供应端的泛称，必须由具体模块说明它指哪一类供应端。” |
| 4 | `spec/README.md:26` | L7 | 交付文档……与约束层同侧 | “同侧”没有可复述的技术含义，读者要猜它指词汇权限还是文档层级。 | 改为：“交付文档描述工程选型、里程碑和约束测试，因此可以直接使用约束层词汇。” |
| 5 | `spec/README.md:51` | S1 | 归属以事实为准绳……不为对称硬填 | “长出来”“准绳”“硬填”组成口号，但没有直接说清归属判据。 | 改为：“结晶归产生它的场景所有：讨论产生的 Memo 归 Chat Room，任务验收产生的冻结约束归 Kanban，机械执行产生的凭证链归 Workflow，会话中的代码修改归 Terminal。没有产物的场景不必为了形式对称而补造一种结晶。” |
| 6 | `spec/README.md:64` | L7 | control writer 与 Agency owner 的排他权同族 | “同族”把两种并未列为 Lease 对象的排他权压成类比，读者不知道应复用哪些规则。 | 改为：“control writer 和 Agency owner 虽然不是 Lease 对象，也必须遵守同样的排他规则：同一时刻只有一个持有者，旧代次失去权限。” |

**文件判断。** 第一次明显读不懂出现在第 26 行的“与约束层同侧”，第 51 行的“结晶”段又需要把比喻反译成所有权规则。第 38 行的 `actor`、`direct client`、`provider event` 要到 `system.md` 才能确认含义，应该就近给一句中文释义或链接；第 64 行的 `control writer` 与 `Agency owner` 也没有指向各自的权威定义。第 18、24、51、73 行最像从英文概念分类和名词链翻过来的中文。

## `docs/design/spec/system.md`

| # | 文件:行 | 条号 | 原文（截 40 字） | 问题 | 建议改法 |
| --- | --- | --- | --- | --- | --- |
| 7 | `spec/system.md:12` | L5 | 现场执行者：worktree/ChangeSet 物化 | 职责、非职责、独立运行时语义和外部工具边界挤在一个表格单元中，四类信息没有层次。 | 改为：“`hctl2-tool` 执行现场操作：物化和隔离 worktree/ChangeSet，执行已持久化的意图，回读结果，并管理现场锁、封存和 Git 事实校验。它不负责 lint 或代码检查，也不执行远端 SCM 副作用。独立运行时，它只提供普通本地操作，不签发 HCTL 治理结果。” |
| 8 | `spec/system.md:14` | L7 | 保存 Run 的机械 token、task、timer | “机械 token”没有定义，`task` 又会与 HCTL Task 混淆。 | 改为：“Workflow Engine 通过适配器保存 Run 的引擎执行令牌、引擎步骤、定时器、重试次数和执行历史。” |
| 9 | `spec/system.md:17` | L7 | 两种 binding 与权威分离 | “两种”没有明确指代，读者不知道是客户端与端口，还是协议与治理绑定。 | 改为：“第三方场景平台可以同时提供场景客户端和受控端口；这两种接入分别绑定，任何一种都不会因此取得 HCTL 治理权威。” |
| 10 | `spec/system.md:34` | S2 | 这些端口就是供应端替换边界……活动执行 | 一段交替定义端口、绑定、适配器、新供应端准入和内容迁移，首次出现的概念彼此依赖。 | 改为：“受控端口是替换供应端的唯一边界，不再增加跨模块通用适配层。每个 Resolved Port Binding 固定供应端制品、模块适配器、配置摘要、实测能力和降级方式。新供应端通过本模块的约束测试后，只能用于新绑定；已有执行继续使用原绑定。迁移既有内容必须另走显式的预览、导出、导入和回读校验。” |
| 11 | `spec/system.md:36` | L2 | Workbench 的 HCTL 功能只依赖…… | 主语从 Workbench 连续切到终端、远程 Agent、Chat、执行面服务、provider mutation 和输入租约，读者无法判断哪条规则属于哪一方。 | 按下文“`system.md:36` 完整重排”替换为“客户端公开面”和“端点与输入信任边界”两组规则。 |
| 12 | `spec/system.md:40` | L2 | 提供方消失时安全暂停、失败或创建替代执行 | 没有说明谁选择三种结果、依据什么选择，也没有区分绑定不可用与活动执行终结。 | 改为：“绑定的供应端不可用时，control 必须按该绑定冻结的降级策略暂停或终结活动执行。只有 owner 提交显式替代命令后，control 才能创建替代执行。” |
| 13 | `spec/system.md:42` | L7 | trusted source 与精确 artifact digest | 中英名词交替出现，而且 discovery、install、upgrade 被写成仿 API 的普通名词，掩盖了动作之间的先后关系。 | 改为：“control policy 只信任来自允许来源且摘要匹配的扩展制品。扩展或注册表的自我声明不授予信任。发现操作默认只读取已配置定义、本地安装和无副作用探测；安装与升级必须由用户显式提交，并产生新的 Extension Revision。” |
| 14 | `spec/system.md:49` | S2 | 同一 `(port_kind, scope_id)`…… | 一句话并列端口解析唯一性、加载顺序独立、五种引用者和凭据边界，主规则不突出。 | 改为：“同一端口种类和作用域在一次准入中只能解析出一个绑定版本，结果不得受加载顺序或界面选择顺序影响。Room、Task、Run 和 Execution Spec 都必须引用精确绑定版本。凭据引用只指向 secret store 条目，不得包含密钥。” |
| 15 | `spec/system.md:51` | L7 | 当前 health……降级约束变化 | “当前 health”“重同步 cursor”是英文词序直译，“类似运行数据”没有边界，“降级约束”也是机械替换留下的生硬搭配。 | 改为：“健康状态、重同步游标和当前成员列表属于运行时投影。每份投影必须记录来源、版本或序号以及观测时间，但不参与不可变绑定的摘要计算。只有端点、配置、能力、信任、权限或降级规则变化时，control 才创建新的绑定版本。” |
| 16 | `spec/system.md:68` | S2 | 实际字段权威只能由对应模块的 authority binding | 同一句连续出现客户端、端口、字段权威、外部 content 和六种外部对象，读者要回到总则区分哪些是身份、哪些只是内容。 | 改为：“客户端只能声明自己的交互能力和降级方式，受控端口只能报告供应端能力。字段写入权由对应模块的绑定授予。外部平台可以拥有场景内容和明确授权的字段，但其 thread、Issue、workflow task、Session 或 pane 都不能成为 HCTL 的身份、授权或判决来源。” |
| 17 | `spec/system.md:70` | T1 | 需要提交 HCTL 命令却无法提供 | “需要”在规范句里承担 MUST，强度不明。 | 改为：“若某个动作必须提交 HCTL 命令，而客户端无法提供等价的预览、版本或权限信息，客户端必须禁用该动作、保留待处理请求或返回安全拒绝。” |
| 18 | `spec/system.md:84` | L5 | 一个 provider 事件可以同时具有 content 含义 | 一段同时讲双重含义、完成请求的准入、拒绝后的显示、字段缺失和回写循环，读者难以找出单个事件的判定顺序。 | 改为：“provider 事件先按 content 事实入账。只有事件能证明 actor、目标、前后版本和幂等依据时，Task adapter 才可以另行生成‘完成 Task’请求。请求被拒绝时，界面同时保留 provider Done 与 HCTL 开放状态。由 HCTL 回写产生、actor 无法映射或字段不全的事件只能形成 Snapshot。” |
| 19 | `spec/system.md:101` | S2 | actor source/provenance 只由 direct client | 一段重复列出来源授予、execution principal、Room fan-out、治理命令和 Harness 禁令，多个相近的 provenance 规则互相遮蔽。 | 改为：“actor 来源只能由直接客户端连接、绑定中的账号映射或 control 内部 reducer 赋予。治理命令只接受两类来源：可映射到 owner human 的动作，以及 task-bound Run 正常完成后由 control reducer 发出的内部命令。Harness、模型和 execution principal 只能提交 Result Proposal，不能自报为 human。” |
| 20 | `spec/system.md:103` | L5 | control 在用户级 metadata 账本的一个 SQLite | 事务一致性、外部投递、幂等、组件不可用和陈旧证据例外是五组独立规则，却串成一个流程。 | 改为：“control 必须在同一个 SQLite 事务中写入领域事件、幂等结果和 outbox。外部适配器使用同一幂等键投递并回读；结果未知时不得盲目重做。组件不可用不会改变命令前置条件：不依赖该组件当前事实的命令可以继续，依赖 fresh readback 的命令必须拒绝。只有冻结策略明确允许陈旧证据时，命令才能携带证据版本、观测时间和已知缺口继续。” |
| 21 | `spec/system.md:169` | S2 | 每个 Repo Instance 的 Git/worktree 资源 | 现场锁、site generation、Agency owner lease、适配器校验和低信任降级连续出现，必须往回翻才能分清三个排他域。 | 改为：“Git/worktree 的现场互斥由 `hctl2-tool` 的 OS 锁保证，control 只在账本中推进该现场的 `site_generation`。Agency binding 另有自己的 owner lease 和 generation。适配器在启动、输入和停止前必须校验适用的现场与运行时代次；供应端不能回显代次时，只能声明 HCTL 入口已校验，不能声明物理执行点已经隔离旧动作。” |
| 22 | `spec/system.md:171` | S1 | SQLite 锁不是外部副作用隔离 | 两个结论并排，没有说明账本锁为什么挡不住事务之外的外部动作；这是指南 S1 的反例原句。 | 改为：“SQLite 事务只保证账本内部一致，而事务提交与外部投递不在同一原子域。因此，外部副作用还必须由幂等键、代次、租约、outbox 和 readback 共同隔离。” |
| 23 | `spec/system.md:189` | L2 | 备份完成前验证快照边界 | 规范句没有执行验证的主语；这也是指南 L2 的反例句式。 | 改为：“备份流程必须在完成前验证快照边界、全部定义引用与校验和，以及 schema 的可读性。” |

**文件判断。** 第 36 行最难读：不是概念本身复杂，而是每句主语和规则对象都在变。第 169 行必须回到 Repo Instance、Agency binding 和 Execution Runtime 各节才能分出三个 generation。`command service`、`authority binding`、`fresh readback`、`conflict scope` 首次出现时没有中文释义或权威链接；第 14、17、42、51、101、169 行都有明显英文名词链和英文词序。第 171 行则不是翻译腔，而是压缩后只剩口号。

### `system.md:36` 完整重排

建议把原段拆成两个小节；第二节再按“端点—治理动作—终端输入”编号。可直接改成：

> **客户端使用的公开面。** Workbench 的 HCTL 功能只调用 Query、Preview、Submit、Subscribe 和各模块投影。provider 客户端功能使用供应端的公开协议或客户端 transport adapter。Workbench 可以直连精确终端的观察流和普通交互输入。未来的官方远程 Agent 必须直接实现 Agency 约束，或通过专用 Agency adapter 接入；非 Matrix 聊天平台仍通过 Matrix homeserver 的桥接生态接入。
>
> **端点与输入的信任边界。**
>
> 1. 第一阶段的 chat server、本地任务服务器、Workflow Engine 和 Herdr 管理/API 端点只能绑定 loopback 或 owner-restricted local socket。未来的非本地 transport 必须认证客户端。
> 2. 若某个 provider mutation 必须先由 HCTL 记账、撤权或校验，只有 control 可以通过对应受控端口发起它。chat server 与任务后端的 content 读写不受这条限制；Herdr 的普通交互输入也不是治理命令。
> 3. 当 binding 声明支持栅栏回显和逐次输入记录时，Herdr 适配代码必须在每次输入前校验 Attach Descriptor、Terminal Input Lease 和当前 generation。binding 未声明这些能力时，原生交互只能按来源不完整的运行时输入记录。
> 4. 若允许 Herdr TUI 原生输入，binding 必须明确记录它不提供 HCTL 单输入租约保证。无论采用哪种输入模式，HCTL 结果都只能从 Result Proposal 准入。

## `docs/design/spec/connections.md`

| # | 文件:行 | 条号 | 原文（截 40 字） | 问题 | 建议改法 |
| --- | --- | --- | --- | --- | --- |
| 24 | `spec/connections.md:16` | S2 | 引用至少包含 kind + stable_id | 字段格式、所属范围、producer、绑定版本和禁止替代项一次出现，读者难以区分必含字段与禁用捷径。 | 改为：“连接引用必须包含对象种类、稳定 ID，以及精确 revision digest 或 state version。引用还必须携带所属 Repo/Project、producer 和适用绑定版本。`current`、显示名、外部 ID、文件路径和界面选择都不能替代这些字段。” |
| 25 | `spec/connections.md:57` | L5 | Task 模块以 CAS 校验……普通消息、总结 | 创建、外部写结果未知、契约准入、content-first 认领和禁止项混在一段，无法按一条失败路径顺读。 | 改为四段：“Task 模块先以 CAS 校验 Project 和可选当前 Task Revision。创建命令先提交 Task 身份、后端 outbox 和关联键；只有命令携带初始契约时，才另写 Revision 准入意图和 Git 正文 outbox。ACK 未知时按原关联键分别回读，不能创建第二个 Task 或卡片。content-first 卡片只有唯一归属一个 Project 分组时，才能被认领为无契约 Task。” |
| 26 | `spec/connections.md:92` | S2 | Participant、Role Binding、Skill 与 Worker | 用一串概念加四个问句解释区别；读者仍要跳到四个文件才能知道哪些字段重叠。 | 改为：“Participant 确定逻辑身份；Project Role Binding 确定该身份在当前 Project 中的职责和权限上限；Skill 提供方法；Worker Profile 选择物理执行配置。Execution Spec 必须分别引用它们，任何一个都不能代替另一个。” |
| 27 | `spec/connections.md:97` | L5 | Agency adapter 校验当前 control/site | 一个步骤同时规定预留、返回值、加固拒绝和 fence 降级，正常路径与失败路径交叉。 | 改为：“Agency adapter 先校验当前 control、site 和 binding generation，再请求 Agency 预留资源。Agency 返回实际能力、物理目标、Execution Runtime ID 和新的 `runtime_generation`。实际能力缺少任何已声明的加固项时，control 必须拒绝激活并列出缺项。Agency 不能回显的 fence 必须记录为未生效。” |
| 28 | `spec/connections.md:101` | S2 | 前三类含义不可混写 | 一句要求同时区分语义 owner、物理执行、三种基础设施 fence，以及四种“不是 generation”的字段；这正是需要分组定义的概念串。 | 改为：“代次分三组记录。第一组标识语义 owner：`invocation_version` 或 `attempt_generation`。第二组标识物理执行：`runtime_generation`。第三组排除旧基础设施写入：control、site 和 Agency binding generation。Participant revision、binding revision、producer sequence 和 content cursor 都不属于代次。” |
| 29 | `spec/connections.md:109` | S2 | control inbox 先按 proposal ID | 去重键、九项准入条件、逐输出 tuple 和 `in_process` 例外在一句中展开，读者无法快速定位某次拒绝对应哪项。 | 改为：“control inbox 先按 proposal ID、producer sequence 和 owner 去重。随后逐项校验 owner 状态、owner 代次、适用 fence、spec/bundle/binding digest、租约、ChangeSet、输出范围、证据和权限。每个输出都必须携带自己的 producer tuple；`in_process` 输出只能使用上节定义的缩减 tuple。” |
| 30 | `spec/connections.md:125` | S2 | 创建时固定 owner_ref + affected_revision | Request 的归属、字段清单、禁止裸 generation 和 Agent 边界写在一个句群中，字段清单难以查阅。 | 改为：“创建 Request 时必须固定以下字段：owner 引用、受影响 revision、阻塞范围、owner state version、输入 schema、所需 actor/role、权限、截止策略和去重根。Attempt 还必须携带 `attempt_generation`，Room Invocation 使用 `invocation_version`；不得使用无法判断所属层级的裸 `generation`。Agent 只执行物理等待，不另建语义 blocker。” |
| 31 | `spec/connections.md:151` | L3 | writer/backend generation 和租约恢复算法 | `backend generation` 没有对应前文的统一名称；前文实际区分 control writer、site 和 Agency binding owner generation。 | 改为：“命令幂等、outbox/inbox、ACK 回读、control writer generation、site generation、Agency binding owner generation 和租约恢复算法只由[系统边界](./system.md#命令与跨服务正确性)定义。” |
| 32 | `spec/connections.md:158` | L5 | owner/runtime identity、lease 或任一 | 失败条件、原子处理、迟到结果和 Retry 身份要求挤在一个表格单元里，无法直接当恢复步骤执行。 | 改为：“若 owner、runtime、lease 或任一适用 fence generation 无法证明，Attempt 或 Room Invocation 必须进入丢失。control 在同一事务中撤销输入/写租约，并提交旧 runtime 的 stop/fence outbox。迟到流和结果只留审计；Retry 必须创建新的 owner、Execution Spec 和 runtime generation。” |

**文件判断。** 第 57 行的创建与恢复路径需要来回拆解，第 97—103 行又必须回看 Agent 和 system 才能区分四层代次。`producer tuple`、`fresh observation`、`owner-specific fence`、`backend generation` 没有统一的首次定义，其中 `backend generation` 还与其他文件的 Agency binding owner generation 不一致。第 16、92、101、109、125 行读起来最像把英文类型定义直接嵌入中文句子。

## `docs/design/spec/project.md`

| # | 文件:行 | 条号 | 原文（截 40 字） | 问题 | 建议改法 |
| --- | --- | --- | --- | --- | --- |
| 33 | `spec/project.md:14` | S2 | Chat 端口绑定……外部 account/room | 对象释义同时塞入绑定内容、动作 allowlist、权威边界和准入链接，读者看不出一句话定义的核心。 | 改为：“Chat 端口绑定是 Room 到 chat server 房间的 Resolved Port Binding。它固定外部账号与房间的稳定 ID、获准身份映射，以及可选的结构化 human 动作清单，并指明 Room content 的事实源。” |
| 34 | `spec/project.md:38` | L5 | Repo 不等于外部组织……不成为 Project | 注册输入、持久化顺序、结果未知恢复、冲突处理、Repo Room 创建和现场挂接是六组规则，无法按创建路径一次读通。 | 改为四段：“Repo 是逻辑仓库，不等于外部组织、工作区或 clone。注册命令固定 `repo_id`、预期 Git identity、配置正文摘要和幂等键。control 先记录待确认注册和 outbox，工具箱再写入并回读 Git identity。结果未知时只能按原 identity 和摘要恢复同一次注册；缺失或冲突必须要求用户处理，不得静默合并。” |
| 35 | `spec/project.md:46` | S2 | Participant 使用稳定……四者不能互相 | Participant revision、Role Binding、Skill、Worker Profile、运行时身份和 Execution Spec 的关系连续出现，读者必须记住四层对象才能读到结论。 | 改为：“Participant revision 描述逻辑身份及其默认方法和执行候选限制，不包含 secret、Project 权限或运行时身份。Project Role Binding 再授予当前 Project 中的职责、权限和预算上限。Skill 只提供方法，Worker Profile 只选择物理执行配置。Execution Spec 必须分别冻结四者的精确引用。” |
| 36 | `spec/project.md:52` | L5 | Scoped Room 创建时必须冻结……回填失败 | 创建字段、完成条件、两条归档路径和失败恢复混在一起，规则强度也没有按阶段展开。 | 改为：“创建 Scoped Room 时必须冻结 parent Room、讨论目标、完成条件和回填动作。达到完成条件不会自动修改目标。归档只允许两条路径：回填动作成功，或有权 human actor 显式以 abandoned、no-decision 或 superseded 结案并记录理由。回填失败时，Room 和目标引用必须保留为可恢复状态。” |
| 37 | `spec/project.md:61` | L5 | Context 交付的是……搜索索引、current | 三份投喂档、选材顺序、序列化顺序和 Manifest 字段清单互相穿插，查 pointer 限制或必含字段都必须重读整段。 | 按下文“`project.md:61` 完整重排”拆成三个编号小节。 |
| 38 | `spec/project.md:63` | S2 | 每个 Room Invocation 或 Attempt 消费者 | Bundle 身份、owner 版本、内容条目、渲染版本、压缩记录、计量、摘要、保留和 replay 连续出现，清单边界不清楚。 | 改为：“每个消费者都从根 Manifest 物化独立的 Context Bundle。Bundle 必须记录自身 ID、Manifest 引用、consumer owner 及其精确版本或代次、按序条目及摘要、renderer/tokenizer/redaction 版本、压缩记录、交付计量、权限与预算、保留策略和 `bundle_digest`。明文可以按保留策略丢弃，但 locator、digest、来源链、策略版本和丢弃事实必须保留。” |
| 39 | `spec/project.md:65` | L3 | 用户配置了 small-brain 时 | `small-brain` 在这里先参与规则，到第 67 行才解释，而且没有链接到用户级定义机制。 | 改为：“用户配置专用的小模型（small-brain）后，相关性门才可以读取消息正文并使用模型辅助判定；该模型必须引用用户级定义机制中的精确 revision 和 digest。” |
| 40 | `spec/project.md:81` | S2 | Execution Spec 除共同字段外……in_process | scope、建议 lineage、Proposal 校验、`in_process` 例外、只读 Repo Room 和 ChangeSet 权限依次出现，读者无法按 Invocation 的生命周期查规则。 | 改为：“Room Invocation 的 Execution Spec 先固定 scope：`repo_scope` 只读，`project_scope` 才能携带写入与 ChangeSet 规则。human 批准建议时，Spec 还必须固定来源建议、建议摘要、可选父执行、fan-out 位置和预期 Room/Project version。若建议来自 Result Proposal，还必须逐项校验 owner、spec、binding、Bundle 和物理执行代次；`in_process` 仅使用连接约束定义的缩减 tuple。” |

**文件判断。** 第 61 行开始无法凭一次阅读建立 Context 的数据形状；第 63 行必须回看第 61 行才能区分 Manifest 和 Bundle。`投喂档`、`selection-policy`、`freshness/coverage/known gaps`、`lineage`、`small-brain` 都在首次出现时缺少自然中文释义或精确链接。第 46、61、63、65、81 行明显保留了英文 schema 描述的句法。

### `project.md:61` 完整重排

建议保留节首总纲，然后分成三组。可直接改成：

> Context Bundle 是调用开工时交付给执行体的输入包，不代管执行体在会话中自行组织的工作上下文。
>
> **1. 三种投喂档。**
>
> - `inline`：直接物化原文。它只用于执行体无法自行读取或不应自行翻找的内容，包括相关聊天、Task 评论、契约与范围说明、用户显式引用，以及策略要求必用的同 Run 前序结果。必用内容超出预算时，必须降为 `pointer` 并附分片建议，不得静默丢弃。
> - `pointer`：只交付精确引用、摘要和一句说明。它只能指向执行体在获准范围内可自行打开的 Git 对象或 worktree 路径；账本和任务后端内容不得作为 `pointer`。
> - `recall`：运行期间按 recall policy 追加的子包条目。
>
> **2. 选材与排序。** 来源按以下顺序选择：用户显式引用；当前讨论窗口；Repo、Project、Task、Run 和 Request 的引用；Git 中的 Artifact、Verdict、Receipt 等结晶副本；必需 Skill；相关 Memo。序列化时，稳定内容排在前面，高频变化内容排在后面。
>
> **3. 根 Context Manifest。** 每次顶层授权必须冻结一个根 Manifest，并包含：
>
> - `context_manifest_id` 与 purpose/scope；
> - 可选 parent Manifest 引用；
> - 每个实际来源的稳定引用及 version/digest；
> - selection-policy version；
> - freshness、coverage 和 known gaps；
> - 必需 Skill 的引用与 digest；
> - permission、redaction 和 budget；
> - `manifest_digest`。
>
> Repo Room、Project Room 和 Run 之间只能通过这些 parent/source 引用传承 Context。搜索索引、`current` 指针和“最近消息”不能替代精确来源。

## `docs/design/spec/task.md`

| # | 文件:行 | 条号 | 原文（截 40 字） | 问题 | 建议改法 |
| --- | --- | --- | --- | --- | --- |
| 41 | `spec/task.md:12` | S2 | Task Binding……操作投影 | 一个对象定义同时引入字段权威、provider 动作、适配器版本、操作投影和 ground truth。 | 改为：“Task Binding 是 Task 与外部来源之间的冻结绑定。它固定外部身份、字段写入权、可接纳的 provider human 动作和适配器版本。排序、优先级、负责人和阻塞等后端字段只形成操作投影，其事实源仍在 content 后端。” |
| 42 | `spec/task.md:20` | L7 | stage……health……Kanban lane | 三个普通看板概念全部保留英文并串成术语，中文读者要先在脑中翻译才能理解正交关系。 | 改为：“Backlog、Ready、In Progress 和 Review 是本地阶段，不是 Task 生命周期。Blocked 和‘需要关注’是独立于阶段的健康状态。Kanban 泳道由本地阶段、Task 生命周期和外部来源投影共同计算。” |
| 43 | `spec/task.md:26` | L7 | 才可 claim 一个 HCTL Task 身份 | 把 `claim` 当中文动词使用，掩盖了这里实际是唯一身份认领。 | 改为：“只有稳定归属到一个已准入 Project 分组的规范卡片，才能认领一个 HCTL Task 身份。” |
| 44 | `spec/task.md:34` | L5 | Task 有两条可恢复的创建路径 | 两条创建路径、两个外部写入者、结果未知恢复和并发碰撞写成连续叙述，无法对照两条路径的相同与不同。 | 改为编号列表：“1. HCTL-first：账本先固定 Task 身份并提交后端 outbox；携带初始契约时再提交 Git 正文 outbox。2. content-first：reconcile 先保存 Snapshot，再认领唯一外部实体并创建无契约 Task。两条路径都按同一关联键恢复；并发命中同一实体时只能复用同一 Task 或返回冲突。” |
| 45 | `spec/task.md:50` | L7 | 才可继续 Submit……不能取得 human provenance | Preview、Submit、human provenance 与 draft 混在中文规范句中，像接口说明而不是动作规则。 | 改为：“control 对该完成请求执行与 Workbench/CLI 相同的预览和准入。只有预览不要求临场选择，而且绑定明确允许该 provider 动作自动提交时，adapter 才可以提交请求。否则，系统保留 provider Done 与 HCTL 开放状态，并等待用户处理或返回类型化拒绝。” |
| 46 | `spec/task.md:66` | L5 | 完成 Task 命令校验当前 Revision | 验收校验、契约分歧选择、CAS 字段、活动 Run 禁令和重开历史混在一个规则块中。 | 改为：“‘完成 Task’命令必须先校验当前 Revision、验收规则、候选和全部必需证据。存在未采纳的契约变化时，actor 必须先采纳新 Revision，或在预览中明确选择按当前 Revision 完成；后一选择必须冻结当前绑定、来源头和全部未采纳 Snapshot。绑定该 Task 的非终态 Run 存在时，完成与取消命令都必须拒绝。” |

**文件判断。** 第 34 行的两条创建路径最难在脑中对齐，第 66 行又必须回看第 30 和 38 行才能知道“契约分歧”究竟冻结哪些来源。`authority-policy digest`、`divergence choice`、`producer generation` 没有就近释义或统一链接。第 20、26、28、32、38、50、68、70 行都带明显的英文领域模型或 API 句法。

## `docs/design/spec/run.md`

| # | 文件:行 | 条号 | 原文（截 40 字） | 问题 | 建议改法 |
| --- | --- | --- | --- | --- | --- |
| 47 | `spec/run.md:16` | S2 | Attempt……派发冻结由 Execution Spec | 一格里同时解释 Attempt、Execution Spec、Attempt 特有字段和 Seat 共有字段，定义依赖未展开的“共同字段”。 | 改为：“Attempt 是某个候选对一个 Seat 的一次执行。派发所需的完整冻结记录由 Execution Spec 承载；Attempt 自身只增加 attempt、seat、run 引用和 `attempt_generation`。” |
| 48 | `spec/run.md:31` | T3 | Run 合法边固定为：启动中 → | 状态转移写成连续枚举，触发事件和非法输入的处理无法查找。 | 节首改为：“Run 状态只由 control 根据获准命令和账本事实推进，Engine 回读只提供观测。下表列出全部合法转移；未列出的状态转换必须返回类型化拒绝。启动中、暂停中和取消中都必须能通过取消、失败或替代进入终态。”随后把现有转移逐行放入“当前状态、触发命令或条件、新状态、其他输入的处理”四列表。 |
| 49 | `spec/run.md:33` | L7 | human 的 Start/Pause……timer 与 Engine | 中英动作、角色和队列术语交替，规则必须先被读者翻成中文流程。 | 改为：“人的启动、暂停、恢复、取消和 Request 回答先进入公共命令服务。Agent 结果先进入 Result Proposal，定时器与引擎位置先成为带版本观测。control 持久化归约结果、撤权和 outbox 后，adapter 才推动 Dagu。” |
| 50 | `spec/run.md:37` | L5 | 运行中 → 完成不是通用写入口 | 完成谓词、未知项处理、无效证据和 Engine 路标分歧连续出现，读者无法逐项实现或审查。 | 改为编号清单：“Run 进入完成前，control 必须逐项证明：1. 所有必需 Obligation、Seat、Gate 和输出已达成；2. 所有 Attempt 已终态或已撤权；3. 没有影响必需输出的未决副作用；4. Manifest、Engine binding 和结果引用仍匹配。任何一项未知都不得完成 Run。Engine 路标只用于分歧检测，不能补足或否定上述谓词。” |
| 51 | `spec/run.md:41` | S2 | Workflow Node、Engine 检查点、Obligation | 五种身份、观察序号、deadline、Engine retry 和候选切换同时解释，读者无法先建立最小身份图。 | 改为：“Workflow Node 是施工图中的节点，Engine 检查点是供应端路标。control 每次观察到检查点重新进入等待态，都按新的观察序号创建 Obligation。Obligation 包含逻辑 Seat，Seat 的每次物理尝试是 Attempt。Engine retry 替代旧 Obligation；候选切换只替代同一 Seat 下的 Attempt。” |
| 52 | `spec/run.md:67` | L5 | 第一阶段 HCTL Profile 允许外部执行 | 允许项、拒绝项、`human.task` 解释、dynamic fork 边界和 loop 限制连续出现，难以判断每条约束对应编译期还是运行期。 | 改为三个编号段：“1. Profile 允许的图结构：外部执行、fork/join、switch、loop、dynamic fork、timer wait、noop 和纯数据转换。2. 编译器拒绝的副作用：子 DAG、默认 command/script、HTTP/action/agent/Harness；Dagu `human.task` 仅作被动检查点。3. dynamic fork 只能实例化 Manifest 中已冻结的有界 Seat 模板；loop 每次重新进入节点都创建新 Obligation。” |
| 53 | `spec/run.md:79` | S2 | attempt_generation 是语义 owner 代次 | 一段先列派发字段，再比较五层代次，再定义整个 Attempt 状态机，概念密度超过首次读者的工作记忆。 | 改为：“Execution Spec 必须固定 Attempt、Seat、Run、Participant、Role Binding、Worker Profile、Agency binding、Context、Skill、权限、预算和可选 ChangeSet 的精确引用。`attempt_generation` 标识语义执行，`runtime_generation` 标识物理执行，control/site/Agency generations 排除旧基础设施动作，三组代次必须分别校验。Attempt 的状态与合法转移另用表格列出；‘已交提案’只表示 Proposal 已冻结，不表示 Seat、Gate、Run 或 Task 成功。” |
| 54 | `spec/run.md:101` | S2 | 逻辑 Participant 分离与 producer/reviewer | 逻辑分离、物理独立证明、unknown 展示、不支持判定、法定票数和返工 regate 混成一段。 | 改为：“第一阶段只证明逻辑 Participant 与 producer/reviewer 分离，不证明物理或组织独立。端口能认证的 provider、model 和 operator 信息必须按 `known/unknown` 展示；策略要求但端口无法认证时，Gate 必须返回 unsupported。达到法定票数后，control 撤销剩余 Attempt 并提交汇总结果；剩余票已不可能达到门槛时，Gate 返回 quorum-unreachable。返工产生新 Revision 后，旧票失效，新的 Revision 必须重新通过完整 Gate。” |

**文件判断。** 第 37 行的完成谓词和第 79 行的派发记录最难一次读懂；理解它们必须反复翻到 connections 和 agent 的代次定义。`HCTL Profile`、`owner-specific fence`、`success terminal`、`quorum-unreachable`、`regate` 首次出现时没有稳定的中文释义或权威链接。第 33、35、47、61、67、73、79、85、97、101 行都有英文动作和名词链直嵌中文的痕迹。

## `docs/design/spec/agent.md`

| # | 文件:行 | 条号 | 原文（截 40 字） | 问题 | 建议改法 |
| --- | --- | --- | --- | --- | --- |
| 55 | `spec/agent.md:8` | S2 | Agent 模块把 Project 拥有的 Room | 模块定位、非职责、接入方式、Execution Spec owner 和 Repo Instance 归属一次出现，读者还没看对象表就要区分五层概念。 | 改为：“Agent 模块只负责把获准的 Execution Spec 变成物理执行，并提供观察、隔离和恢复。Project 拥有 Room Invocation，Run 拥有 Attempt；两者各自拥有对应的 Execution Spec。Agent 不决定 Project、Task 或 Run 的领域结果，Repo Instance 也仍归系统层所有。” |
| 56 | `spec/agent.md:34` | L5 | 第一阶段 HCTL 启动的每个 Harness | 绝对底线、可选加固、安全输入条件和派工前校验挤在一起，读者无法快速判断哪些规则可以缺失。 | 按下文“`agent.md:34` 完整重排”拆成四组，并给每组明确的强度标题。 |
| 57 | `spec/agent.md:38` | L5 | Worktree 是 ChangeSet 的可替换物理资源 | 默认恢复、human 接管的四个选项和自动恢复路径交叉叙述，默认动作不突出。 | 改为：“旧 writer 无法证明已经失权时，control 默认保全并隔离原 worktree 和 ChangeSet，不授予新租约。有权 human actor 可以在预览残留后选择接管、封存、采用到另一 ChangeSet 或丢弃。只有自动恢复必须从获准 baseline 创建新的 worktree 和 ChangeSet。” |
| 58 | `spec/agent.md:54` | L5 | 模型自述已合并不可信……工具箱校验 | 本地集成、远端 SCM、Harness 权限、带外 drift、Git 校验和结果未知恢复全部写在一段，无法按一次集成命令追踪。 | 改为四段：“模型自述不能证明集成成功。control 先持久化‘合入 ChangeSet’意图和 outbox，工具箱再执行本地 Git 集成并回读；远端 push/PR/merge 走同族 adapter 命令。Harness 可以操作自己的 worktree，但改写目标 ref 不产生 Receipt。结果未知时，工具箱回读 Git 与 PR 状态；收敛前不得签发成功 Receipt 或清理现场。” |
| 59 | `spec/agent.md:64` | S2 | 代次必须分层记录而不能共用一个模糊 | owner、runtime、control、site、Agency、Participant、Role Binding 和 producer sequence 一次比较，恰好重现指南 S2 的反例模式。 | 改为：“代次分三层：Invocation/Attempt 代次标识语义 owner，`runtime_generation` 标识一次物理执行，control/site/Agency generations 排除旧基础设施动作。Participant revision、Role Binding version 和 producer sequence 不是代次。替代某一层只使引用该层旧值的动作失效。” |
| 60 | `spec/agent.md:66` | L3 | Agency 供给的是七层身份链的下层 | “七层身份链”在本文和链接中都没有列出，读者无法知道下层具体是哪几层。 | 改为：“Agency 只提供模型、Worker Profile 和 Execution Runtime 这三类物理执行信息；Participant 身份和 Seat 仍由 control 账本拥有。”如果确有七层，请链接到列全七层的唯一权威定义。 |
| 61 | `spec/agent.md:70` | L7 | API/进程 > 结构化 lifecycle 事件 | 用数学比较符号表达证据优先级，又把 `title/screen 仲裁` 当成中文术语，无法自然复述冲突处理。 | 改为：“判断进程是否存活及归谁所有时，优先采用 Herdr API 或进程证据；其次采用结构化生命周期事件或 hook；最后才参考标题和屏幕内容。判断语义状态时，优先采用结构化协议或原生 hook，其次采用转录推断，最后参考标题和屏幕内容。” |
| 62 | `spec/agent.md:74` | L5 | Agency 声明栅栏回显时……未声明时 | 四种能力各自包含“声明时”和“未声明时”的行为，却串成一段，难以按能力核查。 | 改为：“Agency 声明栅栏回显时，必须回显并校验代次与租约；未声明时只能记录 HCTL 入口校验。Agency 声明逐次输入记录时，每次输入必须关联 actor、lease 和 generation；未声明时不得声称来源完整。Agency 声明事件游标时，必须报告 sequence 和 gap；未声明时事件流只能作为有界观测。Agency 声明退出与停止回读时，必须提供同一进程、PTY、退出码和停止结果的证据；证据不足时只能报告 semantic resume、replay 或丢失。” |
| 63 | `spec/agent.md:80` | S2 | 每个 Result Proposal 固定 proposal ID | 顶层身份、`in_process` 例外、逐输出字段、部分准入和修正规则全部连续出现，无法确定哪一字段属于 Proposal、哪一字段属于输出项。 | 改为：“Proposal 头必须固定 owner、runtime、适用 fence、spec/bundle、binding、producer sequence 和幂等键。受信任的 `in_process` Proposal 使用缩减头，而且不得提交 ChangeSet。每个输出项必须另带 schema key、content digest、候选产物引用和自己的 generation tuple。只有 output schema 明确允许逐项准入时，owner 才能单独接受合格项；否则任一必需项不匹配都拒绝整组。” |

**文件判断。** 第 34 行最先造成强度误判，第 64、80 行则需要不断往回翻才能分清同名 generation 和字段归属。`七层身份链` 完全没有可达定义；`backend/Agency owner generation`、`四级恢复词汇`、`title/screen 仲裁` 也缺少统一来源。第 54、64、70、72、74、78、80、86 行保留了明显的英文控制流或类型定义句法。

### `agent.md:34` 完整重排

建议按规则强度拆成四个带标题的块。可直接改成：

> 第一阶段，HCTL 启动的每个 Harness 都使用窄 execution principal。
>
> **不可关闭的三条底线。**
>
> 1. **工具不是人。** Harness、runtime hook 和模型只能提交 Result Proposal，不能提交治理命令。
> 2. **合入钥匙不进工具。** HCTL 不向 Harness 交付 control 客户端凭据、human principal credential、集成凭据或外部写凭据。目标 ref、远端 SCM、任务后端和 chat 写入凭据只由持有当前 fence 的工具箱或 adapter 网关代用。
> 3. **隔离工作树。** Harness 只能在有效 Write Lease 下写当前 ChangeSet 的独立 worktree 和分支。它可以读取所属 Repo Instance 的 Git common-dir 与 refs，也可以在当前 ChangeSet 分支提交。直接改写目标 ref 或其他 ChangeSet 现场不会取得集成权威，只会在回读时形成 drift。
>
> **可选执行加固。** Worker Profile 可以声明 OS 沙箱、凭据代用范围、网络目的地和工具接口白名单。Execution Spec 必须冻结已声明项。未声明时，control 不施加这些加固，也不得记录为已生效；声明后若宿主或 Agency 无法可靠施加，control 必须拒绝激活并列出缺项。
>
> **有条件的安全输入。** 只有接入方式能保证敏感输入不进入环境变量、普通 stdin 历史、Room、Context、trace 或 replay 时，Execution Spec 才能启用安全输入。
>
> **派工前校验。** control 必须在交付派工前核对 Context Bundle 的实际交付摘要、Execution Spec 摘要和全部可验证 fence。

## `docs/design/delivery.md`

| # | 文件:行 | 条号 | 原文（截 40 字） | 问题 | 建议改法 |
| --- | --- | --- | --- | --- | --- |
| 64 | `delivery.md:9` | L7 | P2 出门条件……P3 出门条件 | “出门”是内部口头说法，无法让外部工程师判断是完成条件、发布门禁还是阶段目标。 | 改为：“范围按实现阶段分两组：P2 的验收条件可通过公共 CLI 和各 content 系统原生界面完成；P3 的验收条件覆盖 Workbench 场景。” |
| 65 | `delivery.md:36` | L7 | 不能用终端重连偷渡 lifecycle 推进 | “偷渡”是口号，`lifecycle 推进` 又是中英混排，没有直接写明被禁止的状态变化。 | 改为：“终端重连只恢复观察或输入通道，不得改变 Run 或 Room Invocation 的生命周期状态。” |
| 66 | `delivery.md:51` | S2 | P 表回答先建什么，B 表回答什么时候 | 施工顺序、首次产品化时机、P/B 两套阶段关系和“建完不等于敢用”同时出现，读者尚未见表就要记两条轴。 | 改为：“下面用两张表回答两个不同问题。P0—P3 表示实现顺序；B0—B6 表示 HCTL2 可以接管自身开发事实的程度。B0—B5 都发生在 P2，B6 在 P3 末验收。组件完成实现并不自动提高自举等级。” |
| 67 | `delivery.md:56` | L5 | 固定并打包 Herdr 官方二进制……此时尚无 | P1 表格单元同时列工具职责、代码检查边界、Herdr 验证方式和不得宣称自举，验收结论不易定位。 | 改为：“P1 固定并打包 Herdr，并实现 `hctl2-tool` 的现场 Git 职责。Harness 仍按仓库配置运行代码检查，CI 负责强制；这些检查不进入工具箱意图回路。P1 只验证 Herdr 的启动、观察和停止，不产生 HCTL metadata 或 Receipt，因此不得称为自举。” |
| 68 | `delivery.md:87` | L5 | 为 Repo 选定 content 后端 → 映射 | 正常步骤、两类后端、同步方式和八种失败测试连续出现，切片的验收主线被异常清单打断。 | 改为：“Kanban 切片依次完成后端选择、Project 分组映射、Snapshot 导入、按需采纳契约、字段写回和结果回读。本地任务服务器与一个远端后端各走通一次主线。另用独立失败用例覆盖结果未知、限流、外部修改、tombstone、重新绑定、无 Workbench 操作、无契约卡和 Done 请求拒绝。” |
| 69 | `delivery.md:122` | L5 | 各项选型已拍板，验证因此从选谁变为 | P0 的目的、排除项、探针产物、通过的非承诺、产品化时机和非全局 barrier 连续展开，结论出现多次。 | 改为：“P0 只验证 HCTL 实际依赖的 API 和行为，不重新比较已拍板的候选。探针使用可删除环境，只留下实现证据、固定版本和产品化要求；通过探针不等于已经具备一键生命周期、备份或升级。各探针在对应场景首次消费前完成，不构成全局门禁。” |

**文件判断。** 第 51 行的 P/B 双轴在第一次阅读时最容易混淆，第 122 行必须回看 P0、P1、B1、B2、B4 才能判断每个探针何时阻塞。`出门条件`、`事实切换`、`第一次真正自举`、`全局 barrier` 都没有在首次出现处给出可判定释义；第 13—16、56—57、87、122、125、135 行有明显的英文交付清单或发布说明句法。

## `docs/design/contract-tests.md`

| # | 文件:行 | 条号 | 原文（截 40 字） | 问题 | 建议改法 |
| --- | --- | --- | --- | --- | --- |
| 70 | `contract-tests.md:20` | L7 | direct Workbench adapter 或 provider | 一条测试同时混用 direct、provider event、command digest 和 provenance，输入与预期结果不易分辨。 | 改为：“同一个 Matrix 用户动作分别从 Workbench 直连适配器和 provider 事件适配器提交时，两条路径必须生成相同的命令摘要和结果。由 HCTL service 或 bridge bot 发出的同形事件必须拒绝为 human 来源。” |
| 71 | `contract-tests.md:32` | S1 | Task Revision、lifecycle、stage | 只有要检查的名词，没有输入、判据或预期结果，不能据此实现一条失败测试。 | 改为：“外部卡片只改变 stage 或 health 时，Task lifecycle 和当前 Task Revision 必须保持不变；Kanban lane 只重新计算投影。” |
| 72 | `contract-tests.md:38` | S2 | Workbench/CLI human 完成……Run reducer | 三种获准来源、统一命令和两种拒绝来源挤在一条用例中，无法看出应拆几个 case。 | 改为两条：“Workbench、CLI、已获准的 Vikunja Done 和正常完成 Run reducer 必须生成同一种‘完成 Task’命令并经过相同准入。Result Proposal 或单独观察到的外部关闭态提交该命令时必须拒绝。” |
| 73 | `contract-tests.md:43` | L7 | jurisdictional drift | 这是未定义的英文法务式复合词，读者无法从测试名判断可观察结果。 | 改为：“原生 UI 把卡片移到另一 Project 分组时，HCTL 必须保留原 `project_id`，把来源归属标为需要关注，并拒绝依赖原归属的新命令。” |
| 74 | `contract-tests.md:49` | S1 | 编译/Profile 拒绝、0..1 Task 绑定 | 三个主题只有名词，没有给出触发条件和期待结果。 | 改为三条：“Workflow Revision 不通过 schema、Profile 或图结构校验时，编译必须拒绝。Run 绑定超过一个 Task Revision 时，启动必须拒绝。已绑定 Engine execution 的状态只能由 control 命令推进。” |
| 75 | `contract-tests.md:64` | S1 | 能力探测、ChangeSet 单 writer | 四个检查主题没有测试动作和断言，不能判断何时算通过。 | 改为四条：“未探测到声明能力时，绑定必须拒绝或降级。第二个 writer 请求同一 ChangeSet 的活动租约时必须拒绝。Revision 或 digest 不匹配的 Proposal 必须拒绝。旧 `runtime_generation` 的输入和结果必须拒绝。” |
| 76 | `contract-tests.md:86` | L3 | 每条 handoff 固定 source ref | connections 第 8 行明确说不设 `Handoff` 聚合，这里却用 handoff 指连接，容易被理解为另一个对象。 | 改为：“每条模块连接都必须固定 source ref、digest 和唯一 binding；缺任一项时目标准入必须拒绝。” |

**文件判断。** 最难读的不是复杂场景，而是第 32、49、64 行这类只有主题、没有断言的条目；读者必须回到 spec 猜测试意图。`family ID`、`fail closed`、`handoff`、`jurisdictional drift`、`producer tuple` 都没有就近中文释义，其中 `handoff` 还与连接模型的命名裁决冲突。第 20、26—27、34—43、49—57、64—80、86—99、122—123 行普遍像从英文测试标题直接拼成中文名词短语。

## 机械替换遗留的生硬搭配

本轮范围内没有命中任务书举例的“准入约束”与“诚实约束”，但命中了“降级约束”“重建约束”和“公共约束”。除第 15 条已覆盖的一处外，还有三处：

| # | 文件:行 | 条号 | 原文（截 40 字） | 问题 | 建议改法 |
| --- | --- | --- | --- | --- | --- |
| 77 | `spec/system.md:154` | L7 | 不可用走降级约束……永久丢失走重建约束 | “走……约束”把规则名称当成流程目的地，是机械替换后的生硬搭配。 | 改为：“每类事实都要分别说明两种故障：暂时不可用时如何降级，永久丢失后如何重建。” |
| 78 | `delivery.md:18` | L7 | 仍按各 provider 公共约束处理 | “公共约束”没有明确指向，provider 又与中文修饰语混排。 | 改为：“消息、卡片和终端输入仍按各供应端的公开协议及其绑定中声明的能力处理。” |
| 79 | `contract-tests.md:133` | L7 | 不增加任何公共约束之外的 HCTL 命令 | “公共约束之外”不能说明命令集合的边界在哪里。 | 改为：“安装 Workbench 不得增加公共命令服务尚未提供的 HCTL 命令。” |

## 总结

最优先的语言改动有四组：

1. 先落三份指定重排：`system.md:36`、`project.md:61`、`agent.md:34`。它们分别解决主语漂移、清单散文化和规则强度混杂。
2. 把代次、来源和绑定的英文名词串改成分组定义；尤其统一 `backend generation`、`Agency binding owner generation` 和 `producer tuple` 的叫法。
3. 把 `claim`、`handoff`、`fail closed`、`jurisdictional drift`、`出门条件` 等内部口语或英文动词改成可直接复述的中文动作与结果。
4. 补全契约测试中的谓词。只有主题名的条目无法构成失败用例，也无法反向验证约束是否写清。

## 对指南的异议

无。

# 用户路径对照：新建项目、产生 topic、观测 RUN

> 状态：已归档 · 2026-09-19；不再以本稿或所引旧备忘的候选模型解释当前需求，正文保留作史料<br>
> 基线：main @ `31ccffc`（草案 v0.18.6）<br>
> 去向：[基础用户体验](../../docs/user-experience/README.md#基础用户体验)与[接手清单](../../docs/user-experience/open-questions.md#规范对齐清单)

> 覆盖范围：§二–§五 的 Project 退场候选、旧 Project 下的单板、主 Room 加 Scoped Rooms 列表、Topic 强制回填及向导改名建议，已由新组织结构接替；不能再整节宣称「逐步机械对照仍然成立」。异步状态、选人和观察分析仍有参考价值；显式不挂平台的第三选项、模板跳过读回等增量仅为未采纳建议，不是所有者确认的本轮体验。
> 说明（原状态）：讨论中 · 部分被覆盖——所有者 2026-09-18 指出路径里的「project」混用了仓库与话题（四点澄清录在 `closure-control-repo-20260918.md` §一），本文第二节原来的闭包改述随之作废，以该文为准；第三节的逐步机械对照仍然成立；第五节的拍板项已并入同题备忘的清单<br>
> 原去向（历史）：拍板清单以 `closure-control-repo-20260918.md` §九 与 `user-path-20260917.md` §六 为准；本文保留下来的独立增量见第五节末。本文只做对照与建议，不改 `docs/`

所有者给了一份三幕的用户路径（新建项目 → 产生 topic → 观测 RUN），问：与现有的多单元架构有没有冲突和问题，需要怎么改。本文逐段对照基线上的现行设计回答。

## 一、定位与重述

这份路径在整体架构里的位置：它几乎全部落在**展示面**——Workbench 这个组合客户端上的产品旅程。多单元架构（[三面架构 §单元与连接](../../docs/design/architecture.md#单元与连接)）管的是控制面、Agency、前端、内容系统与仓库底座各自怎么装、怎么连；这份路径不要求改任何单元划分。所以问题要换成一个更好回答的形式：**路径里的每个动作，现行的对象与命令承不承接得住。**

先把路径重述成现行词汇：

| 路径里的说法 | 现行对象或机制 |
| --- | --- |
| 新建一个 project 并选 SCM 地址 | 「注册 Repo」+「创建 Project」两条类型化命令的组合向导 |
| ctl + project 闭包 | 〈控制面，仓库〉（所有者 2026-09-18 澄清；本文初稿的三层改述作废，见第二节） |
| 为 project 选 task-kanban，有且只有一个 | 启用看板的 Project 恰有一个源引用 |
| 为 project 选取 participants、建好 chat room | 「选入 Room」命令往 Project Room 名册选规划者；Project Room 随 Project 同事务创建 |
| topic 聊天室 | Scoped Room（四家一致的建议形态；更大的处置——Project 退场、房间只留 Repo Room 与话题室两种——见 `closure-control-repo-20260918.md` §六，待拍板） |
| 聊天里操作 task | 建议卡 → Trigger Preview → 类型化命令；聊天消息本身不是入口 |
| 快捷模板生成 RUN 的 DAG 施工图 | 参数化模板实例化出 Workflow Revision（先例已有，治理路径待拍板） |
| Run 的内容介绍 / DAG 拓扑图 | Workflow 场景的施工清单预览与只读图（令牌位置回读、语义状态归控制面） |
| worker 的三种观测形态 | Terminal 场景的恢复等级与输入策略；grokbot 式图形观察面是已登记未实施的候选 |

按纪律先分清硬约束与可协商项。**硬约束**（路径若顶撞就得改路径或另行拍板）：聊天消息不是命令入口；批准施工图与开工是两条治理记录；Agency 是控制面与参与者之间的唯一通路；合并板在 Repo 级；通用可视化 Workflow 编辑器明确不做。**可协商**：向导把几条命令合成几步、通知的呈现位置与样式、左侧导航的分组方式——这些是 UI 组合，不动对象语义。

## 二、总裁决：与多单元架构不冲突，一处口径要改

**裁决：整条路径与多单元架构没有冲突。** 它恰好全部发生在[交付文档 §当前范围](../../docs/design/delivery.md#当前范围)第一期的形态里——单机装齐全部单元（一个控制面、本地 Agency 参考实现、随包内容系统与本地平台、前端）。即便推到第二期的跨机形态，路径里每一步的对象语义也不变：远程 Agency 供人走同一个 Agency 端口，grokbot 式远端观察面经票据接入，联合视图里每项事实保留它来自哪个控制面。

唯一要改的是「ctl + project 是一个闭包」这个心智模型表述。本文初稿（合入前被 main 前移打断）把它改述成三层——权威闭包在控制面、语义范围以 Repo 为界、Project 是 Repo 里的工作单元——并指出「participant 不越过闭包边界」不成立：工种名册在 Agency 上，跨 Project、跨控制面可见，选入记录按 Room 名册与 Run 席位各持一份（[Participant 约束 §对象](../../docs/design/spec/participant.md#对象)）。

**这段改述先认账、再作废。** 先认账：路径里的「project」一词担了三个意思——「新建 project 选 SCM 地址」说的是仓库，「project 聊天室」对「topic 聊天室」说的是话题，「ctl + project 闭包」说的是仓库；本文初稿把它们全对到了现行 Project 对象上，与同题四份备忘犯了同一个错。所有者 2026-09-18 的四点澄清（录在 [`closure-control-repo-20260918.md`](./closure-control-repo-20260918.md) §一）把口径定成了：闭包是〈控制面，仓库〉；用户指定仓库后看到的第一个房间就是 Repo Room，之后聊天里长出来的都是话题室；闭包里 rooms、tasks、runs 三个视图是关联关系，不是从属关系。逐处判读见该文 §十，本文不重复。

初稿里有一半在新口径下反而更干净：「participant 不越过边界不成立」——闭包边界画在〈控制面，仓库〉上之后，工种与名册供给本来就在闭包外流动（closure 备忘 §四列了跨闭包流动的四样东西），这不再是需要纠偏的例外，而是闭包定义的直接推论。另一半——「Project 是工作单元」——是否还成立取决于所有者对 Project 对象处置的拍板（closure 备忘 §六 甲/乙/丙，推荐甲：Project 退场为话题），本文不再另立改述。

## 三、逐段对照

读本节时注意：本节写作时把路径里的「project」一律对到了 v0.18.6 的 Project 对象；按 2026-09-18 澄清，N1 的 project 是仓库、T1 的「project 聊天室」对「topic 聊天室」是 Repo Room 对话题室、R1 的 project 是闭包，逐处判读见 `closure-control-repo-20260918.md` §十。本节的机制对照在两种读法下都成立——源引用、名册、Scoped Room 纪律、读回、Agency 能力声明都挂在 Repo 与 Room 上，不挂在容器叫什么上；但若所有者按 closure 备忘 §六 的甲拍板（Project 退场），本节涉及 Project 对象的句子要按该文 §七 的改写表过一遍。

### 《新建项目》

**1. 建立 project。** 成立，向导是 UI 组合。

- 选 remote 地址 → 完成：对应「注册 Repo」（绑定来源的外部平台）加「创建 Project」（同一控制面事务建 Project Room）。两条命令都在公共 CLI 清单里（[交付文档 §公共 CLI](../../docs/design/delivery.md#公共-cli)），向导合成一步不改语义。
- 选本地 git 地址：根据 `.git` 上溯 remote，正是现行机制——[Repo 注册](../../docs/design/spec/repo.md#repo-注册)规定远端地址只作辅助证据，证据冲突时预览列出供人确认，不替人重新认定仓库身份。（实现层脚注：`hctl2-tool repo inspect` 已把身份与辅助证据分组输出。）
  - 选项 (a) attach 到 remote = 绑定来源的外部平台。
  - 选项 (b) 本地另搞一套 = 人声明这是只在本地的仓库，缺省绑定随包的本地平台（Gitea），注册时按持久意图建仓、由持凭据单元交付初始代码。「detach 掉 remote」只是工作副本的便利操作，不是登记生效条件。
  - 纯本地（没有 remote）直接走 (b)，与现行缺省一致。
- 两个要点要进向导设计。其一，现行还有路径里没列的第三选项：**显式不挂平台的受限路径**（仓库太大、不想在本地平台再存一份时），建议向导给三个选项。其二，一条约束决定 (b) 的文案：按人声明的来源，外部平台仓库绑定或换绑到本地平台时**拒绝**——所以 (b) 必须表达为「人声明这个仓库只作本地仓库」，不能表达为「把一个外部平台的仓库改绑到本地平台」。
- 异步状态要露面：需在本地平台建仓时，Repo 先保持待确认，外部步骤确认后才激活，待确认时不接受 Project、Task 或 Run。
- 入口形态与一条已拍板决定相邻：交付文档[明确不做](../../docs/design/delivery.md#明确不做)「用户级总入口对话面」——用户进入产品即在某个 repo 之下操作。「主界面 + 新建 project 按钮」是管理与导航面，不是对话面，不违背这条；但建议入口文案叫「打开 / 登记仓库」而不是孤立的「新建 project」，与该决定及 CT-WORKBENCH-IA 的「打开入口按 repo 选择并统一映射到控制面连接」对齐。

**2. 绑定 kanban。** 完全成立，且现行写法比路径更细：[Task 约束 §契约与来源](../../docs/design/spec/task.md#契约与来源)规定仓库绑零到多个任务源，**启用看板的 Project 恰有一个源引用**——这就是「这个 project 有且只能有一个 task-kanban」。三个选项逐字对应现行候选：平台自带的 issues（缺省建议，须显式同意，没有同意就没有看板）、Linear 这类远端平台、本地任务服务器。两个提醒：

- 当前批 Linear 只过身份与快照验证，绑定标「不能作缺省源、可认领」（实现层脚注：这是交付期限制，不是架构限制）——选项 ② 在 P2 范围内的实际形态是认领 Linear 里已有的卡，从 HCTL 新建的卡仍落在缺省源。
- 「读回来并可视化」= 任务源的 Snapshot 观测加 Repo 级合并板投影；Project 在板上是分组（[Task §Kanban 场景](../../docs/design/task.md#kanban-场景)）。「看板在 Repo 级」与「project 只有一个 kanban」不矛盾：Project 保存自己的源引用，板是各源的投影。

**3. 建立聊天室。** 成立，顺序微调：Project Room 随「创建 Project」在同一事务里创建，不是选完人再建房间；「从 agency 选人」是「选入 Room」命令，把 Agency 名册里工种的实例选成规划者，候选须满足 Project 选人策略（[Project 约束 §Room 名册](../../docs/design/spec/project.md#room-名册)）。选人可以在创建预览里一并完成，也可以事后补；另一仓库 Room 的阵容可借用为预填。两个要在 UI 里说清的点：这里选的是**规划者**；施工者等启动 Run 时在预览里按席位要求另选，两拨人各选各的，像设计局与施工单位。

### 《产生 topic》

**1. 聊天归纳与 topic 聊天室。** 一半现成，一半是新表面。

- 现成的半边：模型 Participant 可以贴建议卡，采纳仍由人做（[Project §Room 场景](../../docs/design/project.md#room-场景)）；人批准建议后，系统自动把原消息、引用、上下文、权限、预算带进新预览，不要求复制粘贴（[Project 约束 §场景约束](../../docs/design/spec/project.md#场景约束)）。「Yes/No 点一下」= 人采纳建议，走 Trigger Preview。
- 缺的半边：「系统不断向用户更新和总结——之前讨论中怎样的 topic 倒是可以尝试来做」是一个新的模型在环表面。它与「没有新信息的协调不进模型循环」相容——归纳出「这段讨论长出了一个可做的题」是新判断，叫模型合法——但有两个成形要求：归纳必须做成显式、可审计的调用（冻结上下文的 Room Invocation，产物是建议卡），不做隐形的后台模型循环；每次调用按自举纪律记 token 账。通知的落点用既有的「需要关注」投影即可，不另造通知对象。
- topic 聊天室的归属待拍板（第五节第 1 条）。现行能承接的只有 Scoped Room：创建时必须冻结父 Room、讨论目标、完成条件与回填动作，结论回填或显式结案后归档，闲置满 14 天进需要关注（[Project 约束 §Room 与消息](../../docs/design/spec/project.md#room-与消息)）。路径里「topic room 里也能建 task、生成施工图」不构成障碍——那些命令的治理归属仍是 Project（Task 冻结 `project_id`、Run 冻结 Project 与版本），Scoped Room 只是讨论空间。
- 「选人环节和 project 聊天室一样」：成立。Scoped Room 以父 Room 名册的某个版本为预填来源生成自己的记录，不活体共享——UI 上就是同一个选人环节带预填。

**2. 建立或者删除 task；快捷模板生成 RUN 的施工图。**

- 聊天里操作 task：边界先说死——聊天消息本身不是命令入口。可行形态与 topic 建议同构：模型建议卡或 Room 里的结构化动作，经 Trigger Preview 落成类型化命令（「创建 / 采纳 / 完成 / 取消 Task」）。「删除 task」对齐现行词汇：治理侧是「取消 Task」（生命周期只有开放 / 完成 / 已取消）；卡片在任务后端里的删除是 content 操作，按后端能力做。
- 快捷模板 + 交互式表单生成 DAG：有先例、有缺口。先例是[运行默认值](../../docs/design/delivery.md#运行默认值)里的缺省施工图模板——「跟进到可交付」不用画图，模板随发布包提供，批准与开工可以在一次预览里提交。缺口在治理路径：「登记 Workflow」的前置是携带三张清单的快照与产出来源，批准默认要读回（[Run 约束 §Workflow 与 Run 授权](../../docs/design/spec/run.md#workflow-与-run-授权)）；表单实例化没有塑形讨论，清单从哪来、读回做不做，要拍板（第五节第 2 条）。
- 「为 task 生成 RUN」与现行一致：一个 Run 绑定至多一个 Task Revision，批准施工图与开工是两条治理记录，复用现成施工图时可以在一次预览里同时提交。

### 《观测 RUN》

**1. 左侧导航三类内容。** 成立，纯投影。Kanban 是 Repo 合并板按 Project 分组的视图；Room 列表 = 该 Project 的 Project Room 加各 Scoped Room；Run 列表 = 归属该 Project 的 Run。Project Overview 已是定义好的只读投影，Workbench 的跨场景卡片与 deep link 只携带稳定引用（[连接约束 §场景与第三方适配器](../../docs/design/spec/connections.md#场景与第三方适配器)）。导航树在 project 之上还有 repo 级入口（Repo Room、合并板、Change 场景），这是对象关系图的自然结果，不是缺漏。

**2. Run 条目的两种展示。** 成立。「内容介绍 → DAG 任务书」= 冻结的 Workflow Revision 与 Run Manifest 的只读预览；「DAG 拓扑图 → 执行到哪一步」= Workflow 场景的只读图与节点 / 席位 / 尝试渐进展开（[Run 与 Workflow §Workflow 场景](../../docs/design/run.md#workflow-场景)）。右侧「每个 worker 现在在具体做哪步」= 席位与尝试投影加 Agency 观测。两个标签页底下是同一份治理记录：引擎只报告机械进度，语义状态只按治理记录算——「无论①或②都是对 RUN 的内容展示」与这条双权威一致。（实现层脚注：技术基线已选 React Flow + Dagre。）

**3. worker 的三种观测形态。** 两种现成，一种是已登记的候选。

- ② herdr 式 TUI：现成。本地 Agency 参考实现的运行时是 Herdr，其官方 TUI 是原生 Terminal 客户端；Workbench 经 Agency 的公开通道连接某次派工（xterm / Execution Chat），票据由控制面签发、Agency 校验；输入策略分受管单写者与原生交互两种（[Participant 约束 §终端通道、连接与租约](../../docs/design/spec/participant.md#终端通道连接与租约)）。
- ③ headless 无输出：现成且是默认——默认无界面原则，派工不以 TTY 存在为前提（[Participant 约束 §派工与观测](../../docs/design/spec/participant.md#派工与观测)）；观察手段是阶梯：结构化执行流默认，PTY 转录二级诊断，终端接管最后。
- ① grokbot 式虚拟机画面：现状没有，但已登记为候选——[Grok Bot 调研](../../docs/research/workbench/grok-bot.md) 的 2026-09-17 复核记录：「已登记候选、未实施：按派工绑定的图形观察面」。它落在 Agency 门后（远程 Agency 的执行资源带图形工作表面），对控制面是 Agency 绑定上的一项新能力声明；要立项的话按六件事写死：观察目标（哪个派工）、能力类型（状态 / 事件 / 终端 / 截图 / 图形流 / 回放）、输入权限（四种分开）、生命周期（断线 / 替代 / 重建的降级）、隐私与共享、证据地位（观测不等于验收）。
- 「根据 agency 选择时的要求」= 能力按 Agency 绑定声明并冻结、如实标注，正好对接现行机制。

## 四、发现清单

| 编号 | 发现 | 性质 | 建议 |
| --- | --- | --- | --- |
| F1 | 「ctl + project 闭包」的口径 | 被 2026-09-18 澄清覆盖 | 以 `closure-control-repo-20260918.md` 为准：闭包是〈控制面，仓库〉；「participant 不越过边界不成立」这一半仍成立——工种名册在 Agency，跨闭包流动 |
| F2 | 「新建 project」入口 = 注册 Repo + 创建 Project 的组合向导 | 成立 | 入口文案对齐「打开 / 登记仓库」；主界面是管理面，不做成对话面 |
| F3 | 本地 git 地址的分支逻辑与现行注册机制同构，但少列了「显式不挂平台」；(b) 受「外部平台来源拒绝绑本地平台」约束 | 成立，补一个选项、守一条文案约束 | 向导给三个选项；(b) 表述为「人声明只作本地仓库」 |
| F4 | kanban 步骤与「Project 恰有一个源引用」精确对应 | 成立 | Linear 当前批只能认领不能作缺省源，UI 如实标 |
| F5 | 聊天室随 Project 同事务创建，先于选人 | 成立，顺序微调 | 选人并入创建预览；UI 区分规划者与施工者两次选人 |
| F6 | topic 归纳建议是新表面 | 半缺口 | 做成显式 Room Invocation + 建议卡 + 人采纳；通知用需要关注投影；计 token 账 |
| F7 | topic 聊天室的房间类型 | 待拍板 | 建议 Scoped Room（话题即讨论目标、必填回填动作）；若要长期平级房间则是新类型 |
| F8 | 聊天操作 task | 成立，守边界 | 消息不是入口；建议 → 预览 → 类型化命令；「删除」写「取消」 |
| F9 | 快捷模板 + 表单生成施工图 | 缺口，待拍板 | 最小改法见第五节第 2 条；划清与「通用可视化编辑器不做」的边界 |
| F10 | Run 观测（导航、双视图、worker 步骤） | 成立 | 无 |
| F11 | grokbot 式虚拟机观察面 | 已登记未实施的候选 | 立项与否待拍板；做的话按六件事设计、作 Agency 能力声明 |

## 五、需要怎么改

「架构不用改」这条结论在 09-18 澄清之后仍成立——四类单元与仓库底座不动；变的是容器层：Project 对象怎么处置（甲 / 乙 / 丙）是 `closure-control-repo-20260918.md` §九 的拍板项，不是本文的。

**本文原来的三处拍板并入同题备忘的清单，不另立第四份：**

1. **topic 聊天室** → 并入 `closure-control-repo-20260918.md` §九 第 2、4 项（Project 处置、Scoped Room 并入话题室）与 `user-path-20260917.md` §六 第 4 项。四家一致的下限是「落 Scoped Room」；若按甲走，话题室成为两类房间之一，回填义务降为可选声明。本文的建议不变：回填纪律值得留。
2. **模板实例化的治理路径** → 并入 `user-path-20260917.md` §六 第 5 项（模板生成的图默认跳过读回，人改过节点文字才读回）。本文补一个更锋利的机制候选：表单实例化产生的 Workflow Revision 以「无模型产出调用」声明产出来源，读回的回避检查缺省自动满足；读回做不做由模板声明。这与 GLM 备忘的「读回前置由产品串好、不默默取消」相容——跳过要是模板里的显式声明，不是表单的隐式行为。
3. **grokbot 式图形观察面** → 并入 `user-path-20260917.md` §六 第 6 项与 GLM 备忘的 H 项。四家合流：Agency 声明观察面三档、绑定精确派工、本地不做桌面档（与技术基线相抵）、远程 Agency 属第二期；六件事清单各家一致。

**本文保留下来的两个独立增量**（四份同题备忘都没写）：

- 向导的第三选项「显式不挂平台的受限路径」——仓库太大、不想在本地平台再存一份时的合法选择；其余备忘只给了 attach / Gitea 两岔。
- 注册的「待确认」中间态要在向导里露面：需在本地平台建仓时，Repo 待确认期间不接受 Project、Task 或 Run，界面要显示准备中，不能假装可用。

**拍板后的文档落点**：以 `closure-control-repo-20260918.md` §七 的改写表为准（它覆盖了本文原来列的落点）；约束若有变更，逐条裁决、bump patch、配 CT 失败用例（按 CONSTRAINTS.md 既有纪律）。

## 六、与四份同题备忘的关系

同一个问题，主线已有四份备忘：Grok 的 `grok-user-journey-20260917.md`、GLM 的 `user-journey-review-20260917.md`、Codex 的 `user-journey-alignment-20260917.md`、Claude 的 `user-path-20260917.md`，外加 Claude 按所有者 09-18 澄清写的 `closure-control-repo-20260918.md`。本文写作时基线上还没有它们（31ccffc），合并前 main 前移才发现；按指令累积与如实报告，本文保留原文并加修订，不装作读过再写。

四家与本文的总判断一致——与多单元架构不冲突、不必另起对象、落点在各模块既有机制上——五家独立收敛到同一结论，本身是个信号。本文的残余价值：第三节带 spec 锚点的逐步机制对照，可作将来 S3 用例写作的对照底稿；第五节的两个独立增量。本文的教训照 closure 备忘 §二 的话认账：遇到一个词在同一段话里换了指代，先停下来问，再往下对。

## 七、业界对照

本路径涉及的外部形态在仓库里都有调研底，不重新造词：

- **三种观测形态**：grokbot 式虚拟机见 [Grok Bot 调研](../../docs/research/workbench/grok-bot.md)（其 Agent Computer 视图是观察—接管—交还回路的一手证据，图形观察面已登记为候选）；herdr TUI 见 [Herdr 客户端层调研](../../docs/research/sdk/herdr.md)（本地 Agency 参考实现的运行时）；headless 是本库默认原则，无外部依赖。
- **kanban 三个源**：GitHub Issues（[sdk/github.md](../../docs/research/sdk/github.md)）、Linear（[sdk/linear.md](../../docs/research/sdk/linear.md)，当前批只过身份与快照）、本地任务服务器（[task-backends.md](../../docs/research/task-backends.md)，Vikunja 已拍板）。
- **本地另搞一套**：随包本地平台 Gitea，见 [gitea.md](../../docs/research/gitea.md)；外部平台 attach 的能力差异维度见 [scm-platforms.md](../../docs/research/scm-platforms.md)。
- **建议—采纳形态**：Grok Bot 的反面教材支持现行纪律——它的 routine（例程）创建删除不经审批被第三方审计点名；本库「模型只贴建议卡、采纳与提交由人经预览完成」已是同构的正解。
- **模板实例化**：业界通行做法是把模板做成参数化的既有定义（GitHub Actions 的 workflow templates、Linear 的 issue templates），表单只填参数、不自由建模——与第五节第 2 条建议的形态同构。

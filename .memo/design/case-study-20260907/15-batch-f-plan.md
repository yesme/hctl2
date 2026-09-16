# F 批方案 v1：Project 与 Task 解耦——Project 对 Task 是标签，不是容器

> 状态：v1 · 待独审三席（Codex、Grok、GLM）· 所有者 2026-09-17「开」<br>
> 基线：main @ `b50899f`（v0.18.3）；#243（v0.18.4）与 #244（v0.18.5）待合，本批落点写在它们之上<br>
> 流程：05 §三 的 P.1–P.7；从本批起陪审团三层审——先审题、再审解、再审改（§七）

## 零、先审题：这个问题存在吗、值得解吗

**问题陈述。** 约束层把 Task 对 Project 的归属写成了硬容器：`spec/task.md` §对象「`project_id` 是 Task 稳定身份的一部分，创建后不可改写」；§契约与来源「系统不改变 Task 的 Project 归属，也不把不可变 `project_id` 改成新分组……改变 Project 时，用户显式取消或保留旧 Task，并在目标 Project 创建新 Task，再用来源引用连接历史」；`spec/connections.md` §连接约束总表 Project → Task「创建 Task 命令固定不可变 `project_id`」，§跨模块 Request 回路「……都不能改变 Task 的 Project 归属」；`spec/project.md` §Repo 注册与 Project 归档「开放 Task……随 Project 一并转为只读」。这是 Jira 式的绑定：issue 换项目要换编号、断链接。

**它怎么来的。** `project_id` 从 8 月稿（729238e 2026-08-14）就在；「不可改写」「取消旧建新」的措辞至少自 v0.15.6 起如此。翻遍决策史，没有一章专门裁过「Project 对 Task 是容器还是标签」——它是默认继承下来的，不是决定出来的。D 批定「两套分组」「认领不搬家」时沿用了它；#243 因此多出一条「Project 拆分/合并不另造命令」，所有者 09-17 追问：「Task 到底有没有『自己属于某个 Project』的准确感知？这个感知给 Task 带来了什么？」

**Project 给 Task 的四样东西，谁在用。**

| Project 给的 | 谁真正用 | 出处 |
| --- | --- | --- |
| 讨论的家：Project Room，Task 的来源引用指向那里的消息，Run 的 Context 从那里萃取 | Run（Context）与人（讨论） | `spec/project.md` §Room 与消息、§根 Context Manifest |
| 干活的规矩：允许的 Agency 与工种、预算上限、多样性要求，名册给的命令权限与评审票 | Run（启动时冻结） | `spec/run.md` §启动与 Manifest、`spec/project.md` §场景约束 |
| 生命周期开关：Project 归档，Task 转只读 | 没人用，是顺手冻住 | `spec/project.md` §Repo 注册与 Project 归档 |
| 源里的分组锚点：milestone 对应 Project，卡在哪个 milestone 自动认领进哪个 Project | 看板投影与自动认领 | `spec/task.md` §契约与来源 |

前两样是 Run 的：Run 的 Manifest 本来就同时冻结「Project、0..1 个 Task Revision」（`spec/run.md` §启动与 Manifest）。第三样是被动的。第四样是一个标签的同步。Task 自己真正拥有的只有验收契约和它绑的那张卡；这两样的家已经定了——卡在源里（一张卡一个家），仓库是硬容器（Project 归属于一个 Repo）。

**症状。** 拆分或合并 Project 要逐张取消重建；Task 历史被切在旧 Project、靠来源引用缝；归档 Project 顺手冻 Task；出现「拆分/合并 Project 要不要造命令」这种伪问题（#243 §明确不做）；「认领卡片」命令把两件事绑在一起——为卡建 Task，和把它放进哪个 Project（`spec/task.md` §契约与来源「显式认领进指定 Project」）。

**它就是更大问题的症状——所有者 09-17 的补充（原话）。** 「你现在这个改动，是不是对 fundamental 的那个树状图也有影响 - 本质上讲 control 是一切的起点，而 repo 是所有讨论的 closure - 至于里边的 project、task，实际上已经平权了，是 repo 的不同视图。我说 task 和 project 之间解耦 - 说的是在同一个 project 下的不同子 project/room - 无论是 project、task、run，都是不能跨越 <control, repo> 这个限制/沙箱对的。」

所以本批改的不只是 Task 的一个指针，是树状图：现在 \`architecture.md\` §场景与系统 写「content 容器的层级……一个 Project 一个 Project Room……Project 是板上的分组，Task 是卡片……容器归属 Repo/Project 身份」，把 Project 画成 Task 的容器。改后的图只有两层硬的：控制面是起点，仓库是讨论与工作的闭包，<控制面, 仓库> 是沙箱对；Project、Task、Run 在这一对里面平权，是仓库的三种视图——Project 是一个 Room 加一套规矩加一个分组视图，Task 是一份契约加一张卡，Run 是一次授权执行，各自引用另外两者但互不为容器；三者都不跨 <控制面, 仓库>。备选丁（把 Project 再拆成授权域与标签）在这个图下不必要：Project 本来就不是容器，不用拆它，只要不让 Task 挂在它身上。

**业界。** 硬容器：Jira（issue 键带项目前缀，移项目换键）、Linear 的 team（换 team 换标识符）、GitHub 的仓库（09-17 沙箱验证：transfer 后 id 与 node_id 全换）。软标签：Asana 的 multi-homing（一个任务同时在多个 project）、Linear 的 project（可空指针，随便挪）、GitHub 的 milestone（一个可改字段）与 Projects 看板（一张卡进多个）、Vikunja（任务属一个 project、可移）。规律：硬容器只有一个，是身份与 ground truth 所在；分组都是标签。软件之外：人属于一个部门是硬的，参与哪个项目是软的。

**用例哪一步。** S1.2 mac_ctl 开两个 Project、各拿参与者——Project 在这里是授权域与 Room，解耦后不变；S1.X1 启用看板选缺省源、S1.X3 在 Linear 里的卡显式认领进 mac_jssdk_01——认领与归组分开后照走；S1.X4 源停用——Task 保留、当前分组不动。用例外（所有者 09-17 场景）：jssdk Project 二十张卡做到一半，把「认证适配」六张分出去单独一个 Project；两个尾声 Project 收成一个。

**陪审团在本节要答三件事**：问题存在吗；它是不是更大问题的症状、问法要不要换；有没有本文没列的框架。

## 一、备选（含不做、改前提、换问题）

| 选法 | 内容 | 代价谁付、什么时候付 |
| --- | --- | --- |
| 甲 · 不做 | 维持硬容器；#243「拆合不另造命令」留着；拆合靠取消重建 | 用的人付：每次拆合逐张取消重建；Task 历史被切；归档冻 Task |
| 乙 · 解耦，一张卡一个分组 | Task 的当前 Project 是可变分组记录，0..1；Run 冻结「在哪个 Project 下跑」与「跑哪个 Task 版本」，两者同仓库；归档 Project 冻 Room、拒新 Run，不冻 Task；拆合 = 改标签 | 一次约束层改动（本批）；Kanban 投影按当前分组；CT 换几行 |
| 丙 · 解耦且多归属 | 同乙，但 0..N（Asana multi-homing） | 源锚点只映射一个 milestone，多归属要在控制面另存；看板泳道、自动认领的「恰一分组」前置都要重写；S1 没有这种情形 |
| 丁 · 换问题 | 把 Project 拆成「授权域」（Room + 名册 + 规矩）与「分组标签」两个概念；Task 只挂标签，Run 只挂授权域 | 多一个概念；所有者的树状图（Project、Task、Run 平权、都是仓库的视图）已经不把 Project 当容器，拆它没有收益；只有当一个 Room 要管多个分组、或多个 Room 共用一套规矩时才值得重开 |

**推荐乙，并按所有者的树状图改写架构层的容器层级（K0）。** 丙以后放开是加法。丁记为触发条件「出现一个 Room 管多个分组，或多个 Project 要共用一套规矩」时再开。

## 二、改法逐条

每条：改哪些句子、来自哪条不变量或用例事实、几种改法、代价、推荐；已拍板的标出处。

**K0 · 树状图：两层硬容器，三种平权视图。** 改 \`architecture.md\` §场景与系统 容器层级句：「content 容器的层级随场景各得其所……容器归属 Repo/Project 身份，不归属某个 clone 或客户端」→「控制面是一切的起点，仓库是讨论与工作的闭包，<控制面, 仓库> 是沙箱对；Project、Task、Run 是仓库的三种视图，互相引用、互不为容器，三者都不跨这一对；content 容器随场景：聊天里一个仓库一个 Repo Room、一个 Project 一个 Project Room，看板里一个仓库一张合并板、Project 是分组、Task 是卡片；容器归属仓库身份，不归属某个 clone 或客户端」。\`spec/project.md\` §Repo 注册与 Project 归档「Project 归属于一个 Repo」保留，\`spec/task.md\` §对象 加「Task 归属于一个 Repo」，\`spec/run.md\` §启动与 Manifest 加「Run 的 Project 与 Task 同一仓库」（K3）。新增不变量：Project、Task、Run 的任何引用都不指向另一个仓库或另一个控制面的对象——CT-SYSTEM 或 CT-CONNECTION 一行「跨仓库归组、跨仓库绑定 Task、跨控制面引用时失败」；S1 §四 加 I18。来源：所有者 09-17 原话；这是本批的前提，不是待裁。

**K1 · Task 归属可变。** 改 `spec/task.md` §对象 Task 行「所属 Project」→「当前 Project（可变分组，0..1）」，lifecycle 句删「`project_id` 是稳定身份的一部分，创建后不可改写」，改为「Task 稳定身份是仓库加实体键（有卡）或仓库加本地 ID（无卡）；当前 Project 是有版本的分组记录，只经『归组』命令或源引用所指源的稳定分组对账改变」；§契约与来源 删「系统不改变 Task 的 Project 归属……再用来源引用连接历史」整句，保留「换卡不做」；`spec/connections.md` Project → Task 行「固定不可变 `project_id`」→「记初始当前 Project（可为空）」，§跨模块 Request 回路「不能改变 Task 的 Project 归属」→「不能改当前 Project；改它只经归组命令或稳定分组对账」；`glossary.md` 项目标识符行。来源：§零 四样东西表——归属只服务展示与认领。改法 A（推荐）如上；改法 B 保留不可变 `project_id` 另造「转移 Task」命令改写它——仍是容器思维，不推荐。

**K2 · 认领与归组分开。** 认领 = 为一张卡建 Task（进仓库）；归组 = 设当前 Project。自动认领（源引用所指的源里稳定归属恰一分组的规范卡片）同时自动归组；显式认领可以指定 Project 也可以不指定（无分组 Task）。源内分组变化：改法 A（推荐）在源引用所指的源里、锚点稳定时自动改当前 Project、只追加记录；其他源的分组变化只观测、标需要关注、等人归组。改法 B 一律只观测、等人。这与 D 批「认领后外源分组变化只当观测，不冻结」不冲突：D 裁的是不冻结，本批在不冻结之上让同源的稳定分组变化直接落到标签上。改 `spec/task.md` §契约与来源 两处（认领两路、content-first 自动认领）、CT-TASK 两行（「认领后拿外源分组改写 Project 归属……失败」「原生 UI 把卡片移到源里另一分组时 HCTL 保留原 `project_id`」）。**待裁 b。**

**K3 · Run 冻结 Project 与 Task，两者同仓库。** `spec/run.md` §启动与 Manifest 已写「Project、0..1 个 Task Revision」；加一句：两者同一仓库；Task 当前 Project 与 Run 的 Project 可以不同，启动预览缺省取当前 Project、不同时标出；规矩与 Room 从 Run 的 Project 来。改 CT-RUN 一行。**待裁 c**：不同时是否允许（推荐允许，只提示）。

**K4 · 归档 Project 不冻 Task。** `spec/project.md` §Repo 注册与 Project 归档：「开放 Task、开放 Request 与未归档 Scoped Room 不阻止归档。它们随 Project 一并转为只读」→ Request 与 Scoped Room 随 Room 转只读；开放 Task 不转只读，当前 Project 指向已归档 Project 的 Task 显示「分组已归档」，可归组到别处、可在别的 Project 下开 Run；已归档 Project 拒绝新 Run 与写入型 Invocation 不变。CT-PROJECT 一行。

**K5 · 看板投影。** `spec/task.md` §契约与来源 合并板句「跨源归组靠控制面投影——Task 到 Project 的记录」→「当前 Project 记录」；无分组的 Task 在仓库级单独一栏；`task.md` §Kanban 场景「Project 是分组，Task 是卡片」不变，加一句「Task 换分组不换身份」。

**K6 · 删掉因容器思维而生的条目。** #243 `delivery.md` §明确不做「Project 拆分与合并不另造命令」删（问题消失，不是不做）；`spec/task.md` 换 Project 取消重建句删（K1）。拆分与合并 = 在源里挪 milestone 或人归组，无命令、无事务。

**K7 · 不动的。** Scoped Room、Request 归 Project Room；一张卡一个家、认领不搬家、换卡不做；源引用与分组锚点「一个活跃 Project 在那一个绑定中恰有一个获准锚点」；依赖归源；Run 与评审请求一对一。

**K8 · 配套。** CT-TASK、CT-PROJECT、CT-RUN 各改行；S1 §六 加 X8（拆分场景）；`glossary.md` Task、Project、项目标识符行；决策史一章（这是转折：Project 从容器改为标签）；版本 bump patch。

## 三、用例走查

| 步骤 / 情形 | 改后规则下 | 判 |
| --- | --- | --- |
| S1.2 mac_ctl 开两个 Project 各拿参与者 | Project 仍是授权域与 Room；Run 冻结它 | 走得通 |
| 人把 gh-jssdk 的一个 Task 归组到另一个仓库的 Project | 跨仓库归组拒绝（K0 不变量） | 走得通（拒绝即正确） |
| S1.X1 启用看板、选缺省源 | 不变 | 走得通 |
| S1.X3 Linear 的卡显式认领进 mac_jssdk_01 | 认领建 Task 并归组到 mac_jssdk_01；卡留在 Linear | 走得通 |
| S1.X4 Linear 绑定停用 | Task 保留、当前 Project 不动、标需要关注 | 走得通 |
| N2 两控制面各对同一张卡认领一张 Task | 各自的当前 Project 各记各的 | 走得通 |
| 失败 e 两控制面同改一张卡 | 多写通则不变 | 走得通 |
| 用例外 X8 拆分：把六张卡分出去 | GitHub 上建新 milestone、挪六张 issue；HCTL 新建 Project 指向新 milestone；同源稳定分组对账把六个 Task 的当前 Project 改成新 Project（K2 改法 A）或人归组（改法 B）；旧 Room 历史留在旧 Project，来源引用照指；正在跑的 Run 不受影响，它冻结的是旧 Project | 走得通 |
| 用例外 X8' 合并：两个尾声 Project 收成一个 | 挪 milestone 或人归组；归档被撤的 Project（静止前置不变）；剩余 Task 不转只读 | 走得通 |
| 归档 Project 时有开放 Task 正在别的 Project 下跑 Run | 归档只看归属该 Project 的 Run；Task 不冻 | 走得通 |

## 四、落点

| 改法 | 文件 §节 |
| --- | --- |
| K0 | `architecture.md` §场景与系统；`spec/task.md` §对象；`spec/run.md` §启动与 Manifest；CT-SYSTEM 或 CT-CONNECTION；S1 §四 I18 |
| K1 | `spec/task.md` §对象、§契约与来源；`spec/connections.md` §连接约束总表、§跨模块 Request 回路；`glossary.md` |
| K2 | `spec/task.md` §契约与来源；CT-TASK |
| K3 | `spec/run.md` §启动与 Manifest；CT-RUN |
| K4 | `spec/project.md` §Repo 注册与 Project 归档；CT-PROJECT |
| K5 | `spec/task.md` §契约与来源；`task.md` §Kanban 场景 |
| K6 | `delivery.md` §明确不做（#243 之后） |
| K8 | `contract-tests.md`、`scenarios/S1-multi-unit.md` §六、`decision-history.md` 新章、版本戳 |

## 五、待裁项

- **a · 归属基数**：乙 0..1（推荐，与 milestone 一致）还是丙 0..N。
- **b · 同源分组变化**：自动改标签（K2 改法 A，推荐）还是只观测等人（改法 B）。
- **c · Run 的 Project 与 Task 当前分组不一致**：允许并提示（推荐）还是拒绝。
- **d · 丁 是否现在拆**：推荐不拆，记触发条件。

## 六、已拍板不重开

所有者 09-17 树状图（控制面起点、仓库闭包、<控制面, 仓库> 沙箱对、Project/Task/Run 平权视图）是本批前提；一张卡一个家、认领不搬家、两套分组、合并板是投影、换卡不做（#230）；依赖归源、只有阻塞进启动预览（#243 补记）；Run 与评审请求一对一（#243）；「偏离旧分组不再冻结」（#230）——本批只在其上加同源自动归组，不重开不冻结的裁决。

## 七、陪审团怎么审本批

三层，按顺序：**审题**——§零 的问题存在吗、是不是症状、问法要不要换、有没有本文没列的框架；**审解**——§一 的四个备选够不够、推荐对不对、有没有更优方案（所有者多次选了作者方案之外的选项，陪审团要主动补）；**审改**——§二 每条改法忠不忠于推荐、层对不对、CT 能不能失败、§三 走查有没有走不通的。三层各表态，再逐条维持 / 修正 / 推翻。

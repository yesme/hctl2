# F 批方案 v2：Task 身份与 Project 分组解耦——在 <控制面, 仓库> 沙箱对里，Project、Task、Run 平权

> 状态：v2 · 待交叉一轮（四席：Codex、Grok、GLM、K3）· 所有者 2026-09-17「开」<br>
> 基线：main（v0.18.6，#243/#244 已合）<br>
> 流程：05 §三 的 P.1–P.7；从本批起陪审团三层审——先审题、再审解、再审改（§七）<br>
> v2 与 v1 的差别（听了谁的哪条见 PR 上「作者说明 · 第一轮汇总与 v2」）：来历改写（v0.12 就有、v0.16.1 去限定未重裁、D 批保留了「不改 Project」、决策史 §32 裁过归档随只读）；「四样东西」改成四种关系；问法改准；Jira 事实改正；K1 不再给实体键加仓库维度，唯一范围「本控制面」不动；K2 推荐从「同源自动写回标签」改为「只观测 + 人显式归组 + 一键按源同步」（Grok 的戊 / Codex 的 B 加一键），不再翻 D 批「不改 Project」；K3、K4 补上会顶掉新规则的旧句；K6 改写（#243 已删那条，本批只核对不写回）；新增 K8 无分组 Task 路径；落点节名逐条改正；走查表改正出处并加五行

## 零、先审题：这个问题存在吗、值得解吗

**问题陈述。** 约束层把 Task 对 Project 的归属写成了硬容器：`spec/task.md` §对象「`project_id` 是 Task 稳定身份的一部分，创建后不可改写」「Project 已归档时拒绝创建、采纳、移动、重开、取消或完成 Task」；§契约与来源「系统不改变 Task 的 Project 归属……改变 Project 时，用户显式取消或保留旧 Task，并在目标 Project 创建新 Task，再用来源引用连接历史」；`spec/connections.md` §连接约束总表 Project → Task「创建 Task 命令固定不可变 `project_id`」，§Project → Task：从讨论到承诺「……也不能改变 Task 的 Project 归属」；`spec/project.md` §Repo 注册与 Project 归档「开放 Task……随 Project 一并转为只读」；`spec/run.md` §启动与 Manifest「Project 不匹配……命令必须拒绝」。

准确的问法（Grok、Codex 改）：**怎样调整一个 Task 挂在哪个 Project 下，而不改它的身份、已采纳的契约、既有的执行授权？** 换句话说，三种平权视图之间，哪些引用进身份、哪些可变、哪些只在 Run 里冻结。所有者的直觉「Project 对 Task 只是个 tag」是这个问法的白话版。

**它怎么来的（Codex 替代写法，Grok、K3 同达）。** 08-14 四层稿的对象图就是 `Project --> Task`（`729238e`）；v0.12 审计批（`454d800`）明写「第一阶段 `project_id`……不可改写」「第一阶段不 reparent，取消或保留旧 Task 后新建」，是带阶段限定的限制；v0.16.1 写法批（`249168f`）去掉「第一阶段」时声明不改约束语义，没有重裁；D 批（#230 拍板 v4，13 §K3）保留了「认领后原生分组变化不改 Task 的 Project，也不冻结」；决策史 §32（v0.15.2）显式裁过「归档只被非终态 Run、写入型 Invocation、活动租约与未决外部副作用阻塞，开放 Task、Request、Scoped Room 随归档转只读」——那是为收窄归档阻塞付的代价。**未找到最初专门比较「容器还是标签」的所有者裁决。** 本批依据所有者 09-17 的新前提重新决定迁组语义；它改的是什么写清楚，不把旧决定说成不存在。

**Task 与 Project 之间的四种关系（Codex 改，替代 v1 的「四样东西谁在用」）。** 它们是已有对象之间不同用途的引用，不是新对象：

| 关系 | 现行 | 本批 |
| --- | --- | --- |
| Task 的仓库归属 | 经 Project 间接（Project 归属于一个 Repo） | 直接、固定：Task 归属于一个仓库，与控制面一起构成沙箱对 |
| Task 的当前分组（挂在哪个 Project 下） | 不可变 `project_id` | 可空、有版本的组织关系；只经「归组」命令改，或初次认领时按唯一锚点预填 |
| 契约的历史来源 | 采纳时冻结 Project 来源引用与版本 | 不变：随每个 Task Revision 冻结，不随分组重写 |
| 每次执行选择的 Project | Run、project_scope Room Invocation 与创建 Task 时冻结 Project version 与策略；Run 的 Manifest 同时写 Project 与 0..1 个 Task Revision | 不变：执行 Project 由 Run 在启动预览确认并冻结；规矩与 Room 从它来 |

Project 仍然拥有 Project Room、Memo、Artifact、Request 与选人规矩——本批只把 Task 从它身上解下来，Project 从「Task 的容器」退成「讨论、承诺与规矩的家」（GLM、K3）。人的权限来自 human actor 的命令权限，不来自 Room 名册（`spec/project.md` §Repo 注册与 Project 归档）；人可以不经 Run 完成 Task（`spec/task.md` §写入约束）——这两条非 Run 路径本批保留。

**症状。** 拆分或合并 Project 要逐张取消重建，而外部卡的身份映射不能靠第二个 Task 继续绑原卡；Task 历史被切在旧 Project、靠来源引用缝；归档 Project 把 Task 一起转只读（§32 的代价）；出现过「拆分/合并 Project 要不要造命令」这种伪问题（#243 曾加、后删）；「认领卡片」把两件事绑在一起——为卡建 Task，和把它放进哪个 Project。

**所有者的树状图（本批前提，不重审）。** 原话：「本质上讲 control 是一切的起点，而 repo 是所有讨论的 closure - 至于里边的 project、task，实际上已经平权了，是 repo 的不同视图。……无论是 project、task、run，都是不能跨越 <control, repo> 这个限制/沙箱对的。」落到设计上：硬容器只有两层，控制面是起点，仓库是闭包；Project、Task、Run 在 <控制面, 仓库> 里平权，是仓库的三种视图，互相引用、互不为容器。「不跨」的精确含义（Codex）：三者各自的治理归属固定在一个 <控制面, 仓库> 内，归组、Run 选执行 Project、Run 绑 Task 都校验这一对相同；外部内容、证据与联合显示仍按既有来源、交付与准入规则处理，材料可见不转移治理权威，这不是「任何引用都不能来自外部」，也不另定义按仓库划分的租户隔离。

**业界。** 硬容器只有一个、分组皆标签这个规律成立，但各家的名字不能只按名归类（Codex）：Jira 的 issue 键带项目前缀，移项目后显示键改变、旧键重定向（Atlassian 官方说明），所以「取消重建」的代价要由 HCTL 自己的规则论证，不能归因于「Jira 式」；Linear 的 team 换了要换标识符，project 是可空指针；GitHub 的仓库转移换 id（09-17 沙箱验证），milestone 是可改字段，Projects 看板一张卡可进多个；Asana 的 multi-homing 一个任务同时在多个 project；Vikunja 任务属一个 project、可移。

**用例哪一步。** S1.1 开两个 Project——人已声明，Project 是授权域与 Room，解耦后不变；S1.X1 启用看板、S1.X3 在 Linear 里的卡显式认领进 mac_jssdk_01——认领与归组分开后照走；S1.X4 源停用——Task 保留、当前分组不动；S1.X2 两个控制面各认领同一张卡——各自的当前分组各记各的。用例外（所有者 09-17 场景）：jssdk Project 二十张卡做到一半，把「认证适配」六张分出去单独一个 Project；两个尾声 Project 收成一个。本批推论出的用例外：无分组 Task 的创建与采纳（K8）。

**陪审团在本节要答**：问题存在吗；问法对吗；有没有本文没列的框架。

## 一、备选（含不做、改前提、换问题）

| 选法 | 内容 | 代价谁付、什么时候付 |
| --- | --- | --- |
| 甲 · 不做 | 维持硬容器；拆合靠取消重建 | 用的人付：每次拆合逐张取消重建，且外部卡的身份映射不能靠第二个 Task 继续绑原卡，「取消重建」不是无代价的迁组替代；Task 历史被切；归档冻 Task |
| 乙 · 身份与分组分开 | Task 稳定身份与仓库归属固定；当前 Project 是可空、有版本的组织关系，0..1；Run 在启动预览确认并冻结执行 Project；归档 Project 不再冻 Task | 真正的代价（Codex）：无分组时的建卡来源与授权入口（K8）、当前分组与执行 Project 不同时怎么展示（K3）、已归档 Room 里的未决 Request 怎么办（K4）；不是「CT 换几行」 |
| 乙的子选择 b · 认领后源里分组变化怎么办 | A 同源锚点稳定变化时自动写回当前 Project；B 只观测、标需要关注、人显式归组；C = B 加预览里「按源同步」一键（人显式提交，走归组命令，可批量） | A 要另立「这个字段谁说了算」的规则：人归组到 Q 后卡仍在 P 的锚点，下一次对账改不改回去？乱序旧观测、无匹配锚点、多匹配、目标 Project 的源不同，都要写；并且翻 D 批「不改 Task 的 Project」。B、C 由操作者付：原生客户端挪卡后要再点一下 |
| 丙 · 多归属 | 0..N（Asana multi-homing） | 多成员关系、缺省执行 Project 的选择、重复展示；09-17 的六卡拆分不需要它；「以后加法」也不是无成本承诺 |
| 丁 · 换问题 | 把 Project 拆成「授权域」与「分组标签」两个概念 | 多一个概念；平权不证明两项职责永远不能拆，只是当前用例不要求另立授权域；触发条件「同一仓库要一套规矩管多个 Room，或多个 Project 共用一套规矩」时再开 |

**推荐乙 + C。** 身份解耦是四席都同意的方向；b 选 C 的理由：不翻 D 批「不改 Project」；显式归组不会被源里挪卡覆盖；六卡拆分靠一键「按源同步」付一次确认；所有者多次在「自动」与「人点一下」之间选后者（缺省源显式同意、派工阻塞人确认）。A 留作待裁 b 的备选，附它必须补的规则。

## 二、改法逐条

每条：改哪些句子、来自哪条不变量或用例事实、几种改法、代价、推荐；已拍板的标出处。

**K0 · 树状图：两层硬容器，三种平权视图。** 改 `architecture.md` §5×3 归属矩阵 之后的容器层级句「content 容器的层级随场景各得其所……容器归属 Repo/Project 身份，不归属某个 clone 或客户端」→「控制面是一切的起点，仓库是讨论与工作的闭包，<控制面, 仓库> 是沙箱对；Project、Task、Run 是仓库的三种视图，互相引用、互不为容器，各自的治理归属固定在这一对里；content 容器随场景：聊天里一个仓库一个 Repo Room、一个 Project 一个 Project Room，看板里一个仓库一张合并板、Project 是分组、Task 是卡片；容器归属仓库身份，不归属某个 clone 或客户端」。`glossary.md` Project「具名目标、协作、承诺和交付物的长期容器」与 `spec/project.md` §对象 Project「……的稳定容器」→「具名目标、Room、承诺与规矩的协作域」（Grok）。不变量进 `spec/system.md` 或 `spec/connections.md` §连接模型 一句：「Project、Task、Run 各自的治理归属固定在一个 <控制面, 仓库> 内；Task 归组、Run 选执行 Project、Run 绑定 Task 均校验这一对相同；外部内容与证据仍按既有来源、交付和准入规则处理」。CT 只写跨仓库（跨控制面不在同一存储，配不出失败输入——Grok）：同控制面把 gh-jssdk 的 Task 归组到另一仓库的 Project 必须拒绝；Run 的执行 Project 与 Task 不同仓库必须拒绝；正例：同仓库不同 Project 的 Task 与 Run 按 K3 通过。S1 §四 加 I18。来源：所有者 09-17 原话；这是本批前提，不是待裁。

**K1 · Task 身份与仓库归属固定，当前分组可变。** `spec/task.md` §对象 Task 行「稳定身份、标题、目标结果和所属 Project」→「稳定身份、标题、目标结果、所属仓库；当前 Project（可变分组，0..1）」；lifecycle 句删「`project_id` 是 Task 稳定身份的一部分，创建后不可改写」，改为「Task 的稳定身份与仓库归属不随分组或建卡确认改变；外部卡继续经原有实体键唯一映射到 Task，唯一范围仍是本控制面；当前 Project 是可空、有版本的组织关系，不参与身份或历史 Revision 的判定，只经『归组』命令改变，或初次认领时按唯一锚点预填」。§写入约束 补归组的合法写入者：有权的 human actor，经类型化命令、预期版本校验与幂等键，与「移动 Task」同例（GLM、K3）。§契约与来源 删「系统不改变 Task 的 Project 归属……再用来源引用连接历史」，保留「『移动 Task』只改阶段与排序，不改 Project；跨源相对移动拒绝」与「换卡不做」。`spec/connections.md` §连接约束总表 Project → Task 行「固定不可变 `project_id`」→「记初始当前 Project（可空）；契约来源 Project 与版本按采纳时冻结，不随分组重写」；§Project → Task：从讨论到承诺「也不能改变 Task 的 Project 归属」→「也不能改当前 Project；拖卡与父分组实体不改标签，改它只经归组命令」；§连接模型「引用还必须携带所属 Repo/Project」→ 区分固定的仓库归属与契约来源 Project，不携带可变分组。`glossary.md` 项目标识符行改释义。**不采纳**给实体键加仓库维度（GLM、K3 的推论）：那会把唯一范围从本控制面悄悄收窄到 <控制面, 仓库>，两个 HCTL Repo 绑同一个平台仓库时同一张卡各认领一张，与 D 批「一张卡一个家」相抵，且用例不需要；HCTL-first 的 Task 在卡建成前就有身份，收到建卡确认不能换身份（Codex）。CT：归组 T 从 P1 到 P2 时系统取消 T、在 P2 新建 T' → 失败；创建 T 后建卡成功但确认丢失，重试拿到卡身份时变成 T2 → 失败；同控制面从另一仓库再认领同一实体产生第二份映射 → 失败；用过期分组版本覆盖新分组 → 失败。

**K2 · 认领与归组分开；认领后源里的分组变化只观测。** 认领 = 为一张卡建 Task（进仓库）；归组 = 设当前 Project。自动认领（源引用所指的源里稳定归属恰一个已准入 Project 分组的规范卡片）同时按该锚点预填当前 Project——这是现行前置，不变；显式认领可以指定 Project 也可以不指定（无分组，K8）。认领后，同源与外源的分组变化一律只追加 Snapshot、标需要关注，不改当前 Project——`spec/task.md` §契约与来源「control 只追加 Snapshot：不改 Task 的 Project，不冻结……」原句保留，只把「不改 Task 的 Project」写成「不改当前 Project」；显式归组之后源再挪卡不覆盖。新增：启动预览与看板提供「按源同步」——把当前分组改成源上稳定锚点所指的 Project，人显式提交，走归组命令，逐张，可批量；无匹配锚点（挪到的分组不是任何活跃 Project 的锚点）或稳定多分组时不提供同步、只标需要关注（GLM、K3 的排除分支）。改法 A（同源稳定变化自动写回）作为待裁 b 的备选，选它须同时定：字段主人（人的归组与源对账谁优先）、乱序旧观测、无匹配与多匹配、目标 Project 的源不同时的行为，并写明它改写 D 批「不改 Task 的 Project」。CT-TASK 两行改：「原生 UI 把卡片移到源里另一分组时，HCTL 保留原 `project_id`」→「保留当前 Project」；加「源里挪卡后 HCTL 静默改当前 Project 时失败；按源同步不经人提交、或对无匹配锚点与多分组的卡提供同步时失败；人已归组到 Q、回读卡仍在 P，系统改回 P 或冻结 T 时失败；初次认领按唯一锚点预填 P 应通过」。

**K3 · Run 冻结执行 Project 与 Task，两者同仓库；迁组不动活动 Run。** `spec/run.md` §Workflow 与 Run 授权 Manifest 字段句「Project、0..1 个 Task Revision」加「执行 Project 与 Task 同一仓库」；§启动与 Manifest「Project 已归档、Task 无契约、Project 不匹配或已有占用标记时，命令必须拒绝」→「执行 Project 已归档、Task 无契约、Task 与执行 Project 不同仓库、或已有占用标记时拒绝；Task 当前分组与执行 Project 不同时预览标出两行——规矩与 Room 来自执行 Project、卡与契约来自 Task——人确认后冻结；当前分组为空或已归档时预览必选一个同仓库的活跃 Project，不自动猜」（Grok、Codex）。加一句：迁组不改活动 Run 的 Manifest，不释放、不转移 Task 的唯一占用标记；旧 Run 按原契约正常完成不因 Task 换了分组而拒收。CT-RUN：不同仓库 → 拒绝；当前分组为空、预览未选执行 Project 就启动成功 → 失败；允许不同却预览不标 → 失败；T 在 P 下有 active Run，迁组 Q 后再从 Q 启动一个 Run，绕过唯一占用 → 失败；旧 Run 正常完成仅因换分组被拒收 → 失败。**待裁 c** 只剩「允许并提示」与「拒绝」两选；四席都选允许。

**K4 · 归档 Project 不冻 Task（修订决策史 §32 的 Task 部分，Request 与 Scoped Room 部分不动）。** `spec/project.md` §Repo 注册与 Project 归档「开放 Task、开放 Request 与未归档 Scoped Room 不阻止归档。它们随 Project 一并转为只读」→「开放 Request 与未归档 Scoped Room 随 Room 转为只读；开放 Task 不转只读，显示『分组已归档』，可归组到别处、可在别的执行 Project 下开 Run；不依赖该 Project 新授权的动作（完成、采纳、取消、重开、归组）照常」；归档前置「不存在非终态 Run、非终态写入型 Room Invocation、活动输入租约……」收窄为只看执行 Project 为本 Project 的（Grok），Task 在别的执行 Project 下跑的 Run 不挡；§对象 写入表「已归档拒绝新 Task、Run 和写入型 Invocation」→「已归档拒绝新 Run、写入型 Invocation 与把 Task 归组进来，新建 Task 不能选它作当前分组」；`spec/task.md` §对象「Project 已归档时拒绝创建、采纳、移动、重开、取消或完成 Task」→「当前分组已归档不阻止采纳、完成、取消、重开或归组；阻止的是在该 Project 下启动新 Run 与新建 Task 归入它」。Request（Codex）：T 在 P Room 里的开放 Request 随 Room 只读，归档预览列出仍影响存续 Task 的未决 Request，写清先处理、或以后恢复 P 再处理；不自动迁 Request、不代人结案；「不因归档冻 Task」不等于「原 Project 的未决事项不再影响 Task」。CT-PROJECT：归档 P 后仍挂在 P 上的开放 Task 变只读或归组被拒 → 失败；T 在 Q 下跑 Run、归档 P（T 的当前分组）被该 Run 挡住 → 失败；把 Task 归组进已归档 Project 成功 → 失败；验收齐全的 T 当前分组为已归档 P，人完成 T 仅因 P 归档被拒 → 失败；在已归档 P 下申请新 Run 应拒绝；T 的旧 Request 被迁组或归档隐式解决 → 失败。**依赖待裁 c**：c 若选拒绝，分组已归档的 Task 须先归组才能跑，K4 的收益减半（GLM、K3）。

**K5 · 看板投影。** 约束层（`spec/task.md` §契约与来源）：合并板句「跨源归组靠控制面投影——Task 到 Project 的记录」→「当前 Project 记录」；无分组的 Task 仍在合并板、与未认领的卡区分。呈现（`task.md` §Kanban 场景）：无分组 Task 单独一栏；Task 换分组不换身份（Codex：布局是呈现选择，不钉进约束）。CT：已认领但无分组的 T 被当作未认领卡再建一个 Task、或在合并板消失 → 失败。

**K6 · 拆分与合并不是命令也不是总事务，但每一步都是命令。** #243 已删「Project 拆分与合并不另造命令」（`f313aea`），本批动手时核对没有写回；`spec/task.md` 取消重建句按 K1 删。不新增专用的拆合对象或跨供应端的总事务；建 Project、逐项归组（或一键按源同步）、归档各走自己的命令与本地事务，跨供应端不承诺整体原子（Codex：v1「无命令、无事务」与 K1 的归组命令相抵，改口）。「拆合 = 重新组织任务」只覆盖这次六张卡的重组，不表示目标、Room 历史、Memo 或规矩自动合并。CT：六张只完成前三张归组即中断，恢复若报六张全成、重建 Task 或重写旧 Run 授权 → 失败；按已完成项继续并保持原身份是正例。

**K7 · 不动的。** 一张卡一个家、认领不搬家、换卡不做、两套分组、合并板是投影（#230）；实体键与唯一范围「本控制面」（D 批）；源锚点「一个活跃 Project 在那一个绑定中恰有一个获准锚点」，只对启用看板且有源引用的 Project，不推广成每个 Project 都要有锚点；依赖归源、只有阻塞进启动预览（#243）；Run 与评审请求一对一（#243）；「不冻结」（#230）与「不改 Project」（#230，本批在 C 下继续遵守）；Scoped Room、Request、Memo、Artifact 归 Project；Task 可改分组不授权移动原生卡、不改契约、不换卡（把 Linear 卡的 Task 归到以 GitHub 为源的 Project 时系统自动新建 GitHub 卡并换绑 → 失败）；依赖仍回读原卡所在源。

**K8 · 无分组 Task 的路径（本批推论，用例外）。** 创建预览显式确认建卡的源绑定，仓库缺省源只作建议；从 Room 采纳提案时保留该 Room/Project 的来源版本，不强迫它成为当前分组；无分组 Task 的命令由有权的 human actor 发，权限不来自 Project；`spec/project.md` §Repo 注册与 Project 归档「创建 Task、Run 或 project_scope Room Invocation 时必须冻结获准的 Project version 与相关策略摘要」→ 带当前分组的 Task 创建时冻结该 Project version，无分组 Task 只冻结仓库；Run 与 Room Invocation 不变。CT：无分组创建与采纳的正例；一面允许当前 Project 为空、一面仍要求「所有 Task 都须冻结所属 Project、往该 Project 所选源建卡」→ 失败。

**K9 · 配套。** CT-TASK、CT-PROJECT、CT-RUN、CT-CONNECTION 各改行（上文逐条）；S1 §四 I18、§六 X8（拆分）、X8'（合并）、X9（无分组创建）；`glossary.md` Project、Task、项目标识符行；决策史新章「Task 身份与 Project 分组解耦」（不写成「Project 从容器改为标签」——它仍持有 Room、目标与规矩；Codex、Grok）；版本在实际合入基线（v0.18.6）上 bump patch。

## 三、用例走查

| 步骤 / 情形 | 改后规则下 | 判 |
| --- | --- | --- |
| S1.1 mac_ctl 开两个 Project（人已声明） | Project 仍是授权域与 Room；Run 冻结执行 Project | 走得通 |
| 同控制面把 gh-jssdk 的 Task 归组到另一仓库的 Project（所有者 09-17 边界的反例，非 S1 需求） | 拒绝（K0） | 走得通（拒绝即正确） |
| S1.X1 启用看板、选缺省源 | 不变 | 走得通 |
| S1.X3 Linear 的卡显式认领进 mac_jssdk_01 | 认领建 Task 并归组到人指定的 Project；卡留在 Linear；不靠卡在哪个源猜 | 走得通 |
| S1.X4 Linear 绑定停用 | Task、分组与身份映射保留、标需要关注；各动作是否可做按是否依赖当前回读判 | 走得通 |
| S1.X2 两个控制面各认领同一张卡 | 各自的当前分组各记各的；多写通则不变 | 走得通 |
| 同控制面两个 Repo 绑同一个平台仓库、同一张卡（K3 提的行） | 唯一范围仍是本控制面：一张 Task，归属先认领的仓库；不因加仓库键产生第二份映射 | 走得通（行为不变） |
| 人把 T 从 P 归组到 Q，源上卡仍在 P 的锚点 | C：不改回、不冻结，标需要关注，预览可「按源同步」回 P（人再点） | 走得通 |
| X8 拆分：把六张卡分出去 | 先显式建目标 Project、确认它的源引用与锚点；GitHub 上建新 milestone、挪六张 issue（可选的 content 操作）；对六个既有 Task 逐张归组或一键按源同步；对账命中既有实体键则复用原 Task 只改当前分组，不得再认领出第二张；旧 Room 历史留在旧 Project，来源引用照指；正在跑的 Run 冻结的是旧执行 Project，不受影响 | 走得通 |
| X8' 合并：两个尾声 Project 收成一个 | 迁组后归档被撤的 Project；原 Project 的活动 Run 仍阻止它归档；Room 历史与规矩不自动合并；未决 Request 按 K4 展示 | 走得通 |
| 归档 P 时，T（当前分组 P）正在 Q 下跑 Run | 归档只看执行 Project 为 P 的 Run，不按 Task 标签倒推；T 不转只读 | 走得通（K4 收窄前置后） |
| T 在 P 下有 active Run，迁组到 Q，再从 Q 启动 Run | 唯一占用冲突，拒绝（K3） | 走得通（拒绝即正确） |
| X9 无分组 Task 创建与采纳 | 创建预览显式确认建卡源；从 Room 采纳保留来源版本；启动 Run 时预览必选执行 Project | 走得通（K8） |
| 归档 P 后，T 改组到 Q，T 在 P Room 里的旧 Request | 随 Room 只读；归档预览已列出；先处理或恢复 P 再处理；不自动迁、不代人结案 | 走得通（K4） |

## 四、落点

| 改法 | 文件 §节 |
| --- | --- |
| K0 | `architecture.md` §5×3 归属矩阵（容器层级句）；`glossary.md` Project；`spec/project.md` §对象；`spec/system.md` 或 `spec/connections.md` §连接模型（不变量）；CT-CONNECTION；S1 §四 I18 |
| K1 | `spec/task.md` §对象、§写入约束、§契约与来源；`spec/connections.md` §连接模型、§连接约束总表、§Project → Task：从讨论到承诺；`glossary.md` 项目标识符；CT-TASK |
| K2 | `spec/task.md` §契约与来源；CT-TASK |
| K3 | `spec/run.md` §Workflow 与 Run 授权、§启动与 Manifest；CT-RUN |
| K4 | `spec/project.md` §对象、§Repo 注册与 Project 归档、§Request；`spec/task.md` §对象；CT-PROJECT |
| K5 | `spec/task.md` §契约与来源；`task.md` §Kanban 场景；CT-TASK |
| K6 | `delivery.md` §明确不做（只核对）；`spec/task.md` §契约与来源；CT-PROJECT |
| K8 | `spec/task.md` §写入约束、§契约与来源；`spec/project.md` §Repo 注册与 Project 归档；CT-TASK |
| K9 | `contract-tests.md`、`scenarios/S1-multi-unit.md` §四 §六、`decision-history.md` 新章与台账、版本戳 |

## 五、待裁项

- **a · 归属基数**：乙 0..1（四席一致推荐；与 milestone、「恰一分组」自动认领同构，S1 无多归属）。
- **b · 认领后源里分组变化**：C 只观测 + 人显式归组 + 一键按源同步（推荐，Grok；Codex 选 B——C 是 B 加一键）；A 自动写回为备选（GLM、K3 选 A 加排除分支），选 A 须同时定字段主人等四条并改写 D 批「不改 Project」。
- **c · Run 的执行 Project 与 Task 当前分组不一致**：允许并在预览标出两行、人确认后冻结（四席一致；Codex 附加：保持 Task 唯一占用与原契约验收，不因迁组重新授予并行 Run）。K4 依赖它。
- **d · 丁 是否现在拆**：不拆（四席一致），触发条件照 §一。

## 六、已拍板不重开

所有者 09-17 树状图（本批前提；「不跨」按 §零 的精确含义，不扩大成「任何引用」）；一张卡一个家、认领不搬家、两套分组、合并板是投影、换卡不做（#230）；实体键与唯一范围「本控制面」（#230，本批不动）；「不冻结」与「不改 Task 的 Project」（#230 拍板 v4 的 K3，两半都裁了；本批在 C 下继续遵守，A 若被选中则是改写、不是推论）；依赖归源、只有阻塞进启动预览、Run 与评审请求一对一（#243 已合）。**本批修订的旧裁决**（声明，不是暗改）：决策史 §32「开放 Task 随归档转只读」的 Task 部分；v0.12 起「`project_id` 不可改写、不 reparent」的阶段性限制。

## 七、陪审团怎么审本批

三层，按顺序：**审题**——§零 的问题存在吗、问法对不对、有没有本文没列的框架；**审解**——§一 的备选够不够、推荐对不对、有没有更优方案（所有者多次选了作者方案之外的选项，陪审团要主动补）；**审改**——§二 每条改法忠不忠于推荐、层对不对、CT 能不能失败、§三 走查有没有走不通的。三层各表态，再逐条维持 / 修正 / 推翻。第一轮四席已审 v1，本轮交叉：对作者拒绝的每一条接受或反驳，反驳要有新证据；对他家意见同意的说同意、不同意的说理由；撤回自己上一轮说错的要写明是哪条、为什么。

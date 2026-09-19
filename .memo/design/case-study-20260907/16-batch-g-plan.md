# G 批方案 v3：全库对照 v0.18.7 核对——错的改、多的删、含混的定

> 状态：已落地（2026-09-19；六个动手 PR #264、#265、#267、#268 与 Codex 的 #266、小尾巴 #269 全部合入 main，基线 v0.18.7 → v0.18.11；合并后核对 Grok 贴在 #268；落地记录见 §九）· v3 已拍板（2026-09-19，所有者：「5项 我都同意 codex的选择。」A 档五项按 Codex 的选法，B 档无反对；落法见 §八）· 第一轮与交叉轮四席（Codex、Grok、GLM、Muse）都走完，交叉轮总态 Codex「改后可动手」、Grok「改后可动手」、GLM「可动手」、Muse「改后可动手」，连带落点已并入 · 清点表 89 行（甲 8、乙 63、丙 5、缺落点 13）· 读回是否走、动手开工，待所有者定<br>
> v3 与 v2 的差别（听了谁的哪条见 PR 上「作者说明 · 交叉汇总与 v3」）：交叉轮的连带落点——A2 的定义句与 Run 入口配对（Codex-6）、A2 同段 :118（GLM-5）、A5 的正反例写成 CT 输入（Codex-7，合 Grok-5、Muse-3）、B11 连带改 :47（Grok-3）；C4、E4、B17、D5、C8、D8 六行改成 §八 的单分支写法（GLM-6、Grok-4、Muse-1、Muse-2）；E8 改引三行描述文本（:144、:137、:138）；GLM-4 用 Codex 的约束句；K3「用户能读到吗」收准、K6 抽查不替代逐处核、A5 归第 2 批；§六、§七 与页眉的过程文字改到当前阶段；C2 的归因改准（Grok）。<br>
> v2 与 v1 的差别：§零 三处归因改准（#261 的评价不写成 #257 自述、F1 收成「写得像唯一来源」、删「没有一家靠通读润色」）；裁决清单 U3 改口（一人可多机连同一控制面、也可运行多个控制面，前端可连多个）、O9 补范围（本控制面已知的全部绑定 Task，不替其他 Project 作决定）；K3 缺口定义收准并加「用户能读到吗」；K6 改按族分 PR、同族四层一起改、乙行抽查、状态文字两阶段；清点表改判 5 行（A1、A9、B15 降，D1 升，C8 升 A 档）、二十余行按席位替代写法改改法、新增 9 行（Codex-1–5、Grok-1–2、GLM-2、GLM-4）；§四 补「应保留的规则」；§五 A 档 4 项变 5 项（读回无来源 Room 升入），B 档加归档恢复语义。<br>
> 基线：main @ `459cda6`（草案 v0.18.7；#257 体验澄清、#259 CI 修正、#261 Codex 抽查备忘已合）<br>
> 流程：05 §三 的 P.1–P.7；陪审团三层审（§七）。本批是核对批：不改约束语义的条目直接落；要改语义的单列待裁，所有者逐条裁<br>
> 所有者 2026-09-19 的期待（原话）：「依据我们 #257 的精细梳理，把整个 design doc 调整到与之相符的情况。错误的假设要修正、多余的表达要删除、含混的说法要确定下来。」四个视角（Unit / Module / 对象归属 / 用户导航）与「#257 主要澄清了后两项，并影响前两项的表达；它没有把每个 Project 变成一套独立部署的系统；反过来，共用 Agency、Repo 或源卡，也不意味着共用工作身份和授权」是本批判据；#261（Codex 抽查）与 #251、#232–#237（集体智能与 Grok Bot 对照）的视角一并考虑（§二 K4）<br>
> 陪审团读法：先读 [04 Project 入口](../../../docs/user-experience/04-project-navigation.md) 与 [已定事项与接手清单](../../../docs/user-experience/open-questions.md)，再读本文 §二.1 裁决清单，然后审 §三 清点表

## 零、先审题：这个问题存在吗、值得解吗

**问题陈述。** #257 把体验目录（`docs/user-experience/`）定为 v0.18.7 的 ground truth，并同步了直接相关的约束、正文、CT、场景与术语；Codex 的抽查指出它「没有承诺审完其余所有章节」（[#261 §一](../post-257-design-sweep/codex-20260919.md)），接手清单也写「下一步审核体验、约束、场景与 CT 是否一致」（`open-questions.md:86`）。剩下的问题是：全库还有哪些句子按旧模型讲（Repo 与 Project 一对一、仓库级 Room、Scoped Room 作为独立概念、合并板是仓库级权威、唯一范围本控制面、`repo_scope`/`project_scope` 二分、施工图只能从主 Room 长出……）；哪些规则在两层各写一遍且措辞不同；哪些说法含混到读者会补造行为。准确的问法：**每一个用户步骤，在体验、架构、约束、验收四层是否得到同一个答案；答案不同的地方，是旧句、是含混，还是要新裁一条规则。**

**它怎么来的。** 三个来源叠在一起：(1) 09-07 起的改写 DAG 六批（修正、R、A、C、B、D）加 E、S 两批，每批只核自己的范围；(2) 09-17/18 的用户路径讨论把 Repo / Project / Topic 混用（[03 术语纠正](../../../docs/user-experience/03-terminology-confession.md)），#257 于 09-19 一次澄清；(3) 09-16 的 Grok Bot 对照（#232–#237，已随 #245 落 v0.18.4）与 09-18 的集体智能研究（#251）给模块职责换了一套更准的说法——Room 承载塑形的连续性、Task 是可验收的承诺、Run 是有边界的自动行动、Context 是材料交付合同——正文里旧的比喻式说法没有跟着收。没有专门裁过「全库怎么核」；所有者 09-19 两次指示（「#257 完成后，不单是它，整个 design doc 都需要被重新核对一遍 sweep 一遍」；今日原话见文首）。

**症状。** 已见的：`run.md` 把「从 Project Room 的塑形讨论中长出」写得像施工图的唯一来源，读者会把它当必经步骤（#261 F1：叙事过窄，不足以证明约束强制）；同节还带「私有仓库默认全文，公开仓库可降为仅摘要」的旧括号（#261 F2）；CT-PROJECT / CT-TASK 两条分组概括笼统到判不了失败（#261 F3）；体验 README 仍写「仍待复审、合入」（#261 F4）；塑形技能的产出里还有「开 Scoped Room」（`src/agency/README.md:17`）。主笔抽查五处（闲置 14 天、待你处理来源、已保存计划、Run 与 Task 的 Project 一致、归档转只读）三层一致——#257 列过的落点大体已改对，问题集中在它没列的篇章与跨层重复。

**是不是症状。** 是更大问题的症状：多批分片改写之后没有一次「按关系走到底」的全库核对，而 #257 后正是做这件事的时点——体验目录第一次成了完整的 ground truth。不解决会怎样：实现者读到旧句补造行为（为满足「从主 Room 长出」给模板路径伪造聊天；按「唯一范围本控制面」拒绝第二个 Project 认领）；陪审团继续在已废前提上审题（F 批就是这样，见 [15](./15-batch-f-plan.md) 页眉）。

**业界两种做法。** (a) 架构决策记录（ADR，Nygard 2011）的「被取代」状态：一条决策被新决策取代时，旧记录标 Superseded 并指向新记录，参考文档随之改写——对应我们的决策史「只记转折」加接手清单「旧讨论怎样接手」，本批把它做到正文。(b) 需求追溯矩阵（RTM，系统工程惯例；IEEE 29148 的追溯要求）：每条需求列出设计、实现、测试的落点，缺一格就是缺口——对应本批的「裁决→落点矩阵」。两种都是先列表、再改。

**用例哪一步。** 本批不改用例；核对沿 S1（四个 Project、多机）与 S3（P1–P7、T1–T10、R1–R7）走。#261 §四 的五条主线（同 Repo 多 Project；同卡不同 Project 两个 Task；Room / Task / Run 交叉引用；从计划到交付；共用单元与材料交付）就是「用例哪一步」的索引，§三 清点表每行的「依据」引到步骤或裁决。

**陪审团在本节要答**：问题存在吗；「核对批」这个问法对不对（还是该等 P2 实现碰到再改）；四个视角够不够，有没有本文没列的框架。

## 一、备选（含不做、改前提、换问题）

| 选法 | 内容 | 代价谁付、什么时候付 |
| --- | --- | --- |
| 甲 · 不做 | 等 P2 实现碰到旧句再改 | 实现者付：按旧句补造行为再返工；陪审团付：在已废前提上审题 |
| 乙 · 逐文件润色 | 每篇通读，见旧词就改 | 快，但漏跨层不一致，还会「各自润色」出新的不一致（#261 §四 明确反对） |
| 丙 · 按关系走到底（推荐） | 先列裁决清单与清点表，按五条主线核到底；处置分三类；要改约束语义的另列待裁 | 主笔付一次清点；陪审团审清点表，并沿各自主线通读所负责的权威文件补漏 |
| 丁 · 换问题：以体验目录为纲重写整套设计文档 | 推翻重写 | 代价最大；#261 §一「不需要推翻重写」；决策史与 CT 失去连续性 |
| 戊 · 改前提：体验目录升为权威、设计正文降为解释 | 反转分层 | 违反文档纪律的唯一权威分层；体验目录自己也说「不代替模块约束」（`open-questions.md:7`） |

**推荐丙。** 它是所有者今日原话的直译（错的改、多的删、含混的定），也是 #261 的建议；乙 是最容易滑进去的做法，本批用清点表的「性质」一列挡住它——把该待裁的藏成措辞清理是本批最大的错。

## 二、改法：本批怎么做

### 二.1 裁决清单（本批的不变量；编号供 §三 引用）

用户导航（N）：
- N1 Project 是顶层工作范围，每个 Project 当前关联一个 Repo；同一 Control 可为同一 Repo 建多个 Project（04:7；C2）。
- N2 点 Project 名进入唯一主 Room（Project Room，所有者说的主 Repo Room 指它）；「待你处理 N」是同一行的第二入口（04:28、04:163）。
- N3 Rooms 只列 Topic Rooms、初始为空；Project Room 不在列表里；Scoped Room 用途并入 Topic Room；新 Topic 不建 Project（04:57–61）。
- N4 Topic Room 开场：从本 Project 主 Room 相关讨论构造前情提要（带确认）；Request 升级为 Topic 时按请求与所阻塞工作准备提要（04:59）。
- N5 Kanbans 每个 Source 一个入口；Source 是实际来源及范围；Task 保留自己的 Source；跨 Kanban 拖放不换源；本轮不要求合并板，汇总不替代按源入口（04:65–71）。
- N6 Runs 列出本 Project 所有活动 Run；点击默认 DAG；任务书是另一视图、共用 Worker 侧栏；保存未开工的计划从 Task 找回（04:75–79）。
- N7 待你处理只汇总确需人决定、输入或授权的既有事项，每条答四个问题，处理后退出；进度、未读、建议不进（04:36–53；`spec/project.md:43` 枚举来源）。

对象归属（O）：
- O1 固定的是归属，可交叉的是引用；不强制 Room → Task → Run 包含链（04:85）。
- O2 Project 的 Room / Task / Run 不从 A 搬到 B；同 Repo 不合并归属（04:89；`spec/project.md:41`；`spec/task.md:87`）。
- O3 消息不换 Room；Task 不换 Source；引用保留出处（04:90–91）。
- O4 Room 与 Task / Run 多对多引用，不改归属、不继承授权；关 Room 不停 Run、不解决 Request、不取消 Task（04:92–93；`spec/project.md:62`）。
- O5 Task 与 Run 互引不等于承担交付；Run 完成不自动完成更多 Task；Run 的 Project 与 Task 的 Project 必须一致；执行语义本轮不变（04:94、04:98；`spec/run.md:74`）。
- O6 Project 是独立 Namespace：同一外部卡在各 Project 各有 Task；唯一范围 Project 内；共享源卡不共享契约、Run、授权、验收（04:69、04:181；`spec/task.md:46`）。
- O7 Participant 分别选入：每个 Room、每个 Run 独立选人；Project 无自动灌入的名单；推荐不继承授权；观察与输入指向具体执行、经 Agency（04:102–106）。
- O8 归档：开放 Task、Request、Topic 随 Project 归档转只读（决策史 §32 保留；`spec/project.md:47`）。
- O9 删除 Task（Q3）：草稿可删；已有工作默认「取消并归档」，保留历史、不删源卡；删源卡另行确认，预览列出本控制面已知的全部绑定 Task（不声称列全其他控制面的使用，删卡确认不替其他 Project 取消 Task 或 Run）；有活动 Run 时明确去向（`open-questions.md:31–35`；`spec/task.md:77`；v2 按 Codex 补范围）。
- O10 本地 detach（Q2）：默认另建独立副本，不换绑原 Repo 身份（`open-questions.md:23–27`）。

Module（M）：
- M1 Repo 负责代码来源、平台绑定、版本与集成；Project 负责各自工作的组织（04:112；`open-questions.md:15`）。
- M2 Task 是承诺尺度（契约采纳）；卡是 content；一张卡一个家、认领不搬家；两套分组（源内 vs Project 归属）；分组锚点只作自动认领前置（`spec/task.md:32–34`）。
- M3 Run 模块保存计划关联（登记 Workflow 携带 Task 时核对归属一致），Task 只投影；保存不等于启动（`spec/run.md:72`）。
- M4 主 Room 与 Topic Room 都能走有边界调用；`repo_scope`/`project_scope` 旧二分已废（`open-questions.md:62`）。
- M5 闲置提醒只对承接开放 Request 的活跃 Topic（缺省 14 天），普通 Topic 不适用、不增待处理计数（`spec/project.md:64`；`delivery.md:184`）。
- M6 候选交付的定义（`spec/task.md:105–107`）；无 Run 路径保留。
- M7 #251 的视角只作「含混说法要确定」的判据：Room 承载塑形的连续性、不是权威数据库或授权渠道；Task 是可验收的承诺、不是待办清单；Run 是有边界的自动行动、不是更聪明的流程引擎；Context 是材料交付合同、不是全知视角；Repo 把工作锚定到外部事实；Memo 与 Skill 让组织延续；认知、协调、治理三种规则分开——「建议找谁」不等于「有权调用谁」，「大家觉得差不多」不等于「正式完成」；独立性分程序、信息、统计三层；人是委托人不是调度员（[#251 §2.3、§5.3、§6、§7](../../../docs/research/collective-intelligence-research-20260918.md)，复用决策「仅参考行为」）。

Unit（U）：
- U1 Project 不是部署单元；多个 Project 可共用 Agency、内容服务、Git 对象库与外部代码事实；不要求每 Project 独立服务器（04:32）。
- U2 共用 Agency、Repo 或源卡不等于共用工作身份与授权（所有者今日原话；04:32、04:69）。
- U3 多 Control 联合界面标清每个 Project 来自哪个 Control（04:32）；一个人可以多机连同一控制面，也可以运行多个控制面，前端可连一个或多个，各控制面的身份与授权分别确认（`architecture.md:17`、`:21`、`:35`；v2 按 Codex 改口，v1 的「只连同一控制面」是错的）；租户隔离按 Control（`spec/system.md` 安全策略面）。
- U4 工作副本跟执行位置走，不跟前端走（01:42）。

### 二.2 改法逐条

**K1 · 清点表（§三）。** 每行八格：编号、位置（`path:line`，固定 `459cda6`）、原句逐字、视角 U/M/O/N、依据（裁决编号加 ground-truth 位置，或 #261 F 号）、性质、建议改法、受影响验收。主笔分六组走查合成（A 模块正文与设计地图；B 愿景、架构、交付、纪律、首页；C 约束一：spec/README、project、task、run；D 约束二：system、connections、repo、participant；E 验收、场景、词汇；F 体验目录自审、Agency 技能、写作指南），陪审团审表并沿各自主线补漏。

**K2 · 处置只分三类加缺落点（#261 §四）。** 甲 被取代的旧句：按裁决纠正，不进待裁；乙 含混或多余：收紧或删，不改语义——若动的是约束句的措辞，随批 bump 一次 patch，不新配 CT；丙 需要新的行为取舍：只登记，写选法、代价、场景，所有者裁；缺落点：某条裁决在某层该有落点却没有——正文缺解释补一句，约束缺规则归丙，CT 缺失败例补例。

**K3 · 按关系走到底，不按文件润色。** §四 的五条主线各走一遍，每条主线的「需要一致的地方」在相关层核答案相容、能追到唯一权威；缺少本层应有的答案或权威引用才是缺口，不适用的层写明不适用，不为凑四层补复述（Codex）；找到两句不一样就是乙或甲。另核「用户能读到吗」：影响用户理解路径的关键关系，正文或其直接权威指针没有交代时登记为镜像型缺落点；技术性准入条件可只留在约束层，不为此把字段规则搬进正文（GLM 提、Codex 收准）。每条主线先列「应保留的规则」，核对时不把它们再复述进四层（§四）。

**K4 · #251 与 Grok Bot 对照怎么进本批。** 只作 M7 的判据：正文里与之相抵或含混的句子登记为乙（例：把 Room 写成权威记录、把 Run 写成流程引擎、把 Context 写成全部上下文）；不搬 #251 的格式、字段或对象进设计；Grok Bot 对照的候选池（[主笔处置 §三](../../notes/grokbot-compare/claude-disposition-20260917.md)）仍登记不实施；要把 #251 的某个说法写成规则的，一律归丙。

**K5 · 四个视角的边界句。** 架构层是否已有「Project 不是部署单元；多个 Project 可共用 Agency、内容服务、Git 对象库与外部代码事实；共用不等于共用工作身份与授权」的等价句，清点表核；没有则补一句到 `architecture.md` §单元与连接（性质缺落点；依据 04:32、04:69，不是新规则）。

**K6 · 落地分批。** 清点表拍板后按族开动手 PR，同一族的正文、约束、CT、术语在同一个 PR 里一起改，不先合新正文、后合决定其含义的约束（Codex）：(1) 机械族——状态文字、账本残留、Agency 技能、术语表、评审技能两条；(2) 施工图来源族（含 A5 读回例外：C8、Codex-5、Codex-7）与「哪份 Git」族；(3) 语义范围族、归属与 Namespace 各行、重复收口族（A1、A2 落点）；(4) Overview、Topic 关闭、归档恢复（A3 与 B 档落点）。约束句措辞动了的 PR 各 bump 一次 patch。轻审两席，仍逐处核改出来的文字是否忠于方案；另对乙行每篇抽两行问「除了措辞真的什么都没改吗」（GLM 提、Codex 限定：抽查是附加，不替代逐处核）；合并后核对一家（05 §七 的四种情况）。首页与接手页的状态文字分两阶段：首批只写「#257 已同步直接相关关系，其余由 G 批核对」，全部子批合入后才写「已按体验目录核对」（Codex）。

**K7 · 完成判据（#261 §五）。** 同一个用户步骤在体验、架构、约束、验收里得到同一个答案；每项发现有修正、保留理由或待裁去向；没有因修辞调整新增身份、权限或执行义务。文档机械检查不替代这条内容判据。

**K8 · 配套。** 08 §四 加 G 批行；评审技能「六批里反复出现的错」加两条（F 批复盘）：所有者口头给的新前提先过用例再开批；造成症状的那条旧规则要进备选表。

## 三、清点表（v1）

主笔分六组走查（每组把文件从头读到尾），合成后逐条用 grep 核对原句在 `459cda6` 的位置，全部命中。性质一列是主笔判定；与走查席判定不同的地方标「改判」，v2 按第一轮席位意见再改的标「v2 改判」并写明听了谁的。「族」把同一件事的多处并在一起，动手时一族一次改完。改判的依据只有一条：行为变不变——变的才是丙；落实既有裁决、只需所有者核措辞的，标「请核措辞」而不升丙（Codex）。

| 编号 | 位置 | 原句（逐字，≤160 字） | 视角 | 依据 | 性质 | 建议改法 | 受影响验收 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A1 | docs/design/project.md:58 | 在 Workbench 里同时管理多个仓库时，一个 Room 可以把另一个仓库 Room 的 Participant 阵容借用为预填选择，不必逐个重选。借用只是预填：规划者仍在本 Room 重新选入，权限、预算和绑定不跨仓库继承 | O | O7 04:102–104；N1 04:7；N3 04:57 | 乙（v2 改判：Codex——旧词，行为不变；走查席判甲） | 「仓库 Room」是旧的仓库级说法；同 Repo 两个 Project 的 Room 之间同样只预填、不继承。改为：「在 Workbench 里同时打开多个 Project 时，一个 Room 可以把另一个 Room（同一或另一 Project）的 Participant 阵容借用为预填选择……权限、预算和绑定不跨 Room 继承」 | S3.T4；contract-tests.md:151「名册预填未经确认自动选入、复用另一 Room 或 Run 的授权时失败」（Codex；S1.I3 只验主 Room 身份，不再引） |
| A2 | docs/design/run.md:30 | 施工图（“干什么的计划”）从 Project Room 的塑形讨论中长出，作为结晶归 Room 场景 | M | #261 F1；N6 04:77；M3 spec/run.md:72；02 §T3 | 乙 · 族「施工图来源」（A2、A4、B5、B6、C6、C8、Codex-5） | 来源叙事过窄，会被读成必经路径（不是已证实的唯一来源约束——Codex）。改为：「施工图可以由 Room（主 Room 或 Topic Room）的塑形讨论凝结，也可以从 Task 选模板填表保存而来；其版本、批准与 Task 关联由 Run 模块管理，讨论凝结的做法见 §施工图怎么凝结」 | contract-tests.md:89 |
| A3 | docs/design/run.md:30 | 判决的权威在控制面存储，结晶副本进 Git（私有仓库默认全文，公开仓库可降为仅摘要） | M | #261 F2；repo.md:74「审计公开按授权」；spec/repo.md §发布评审；spec/system.md §控制面自己的存储 | 甲 · 族「哪份 Git」（A3、C7） | 旧括号把存放、公开、交付三件事压成仓库公私属性。改为：「判决的权威在控制面存储；审计副本属于治理材料，公开关联与返工正文按各自获准范围交付（见 Repo 正文 §关键规则「审计公开按授权」）」 | 无（约束未变） |
| A4 | docs/design/run.md:65 | 施工图从 Room 的塑形讨论里长出来。 | M | 同 A2 | 乙 · 族「施工图来源」 | 本节只讲讨论凝结这一条路，句首加限定：「从讨论凝结施工图时（模板与表单路径见 Workflow 场景一节），塑形产出三张清单……」 | 无 |
| A5 | docs/design/project.md:30 | 需要多轮论述、多人参与或共同编辑才开临时讨论空间 | N | N3 04:61；N4 04:59；project.md:43 | 乙 | 「临时讨论空间」是 Scoped Room 时代的说法，现在只有 Topic Room。改为：「才升级为关联该 Request 的 Topic Room（开场提要按请求与所阻塞的工作准备）」 | contract-tests.md:13；S3.T6 |
| A6 | docs/design/README.md:12 | 目标与范围、协作现场的身份与来源记录、参与者、上下文、请求、备忘与工件 | O | O7 04:102；participant.md:47「项目本身不持有成员名单，只持有选人策略」；spec/project.md §Repo 注册与 Project 归档 | 乙 | 「参与者」读成 Project 持有成员名单。改为「各 Room 的名册与 Project 的选人策略」（Codex：策略归 Project、名册归 Room，参与者身份与配置仍归 Participant 模块） | 无 |
| A7 | docs/design/README.md:36 | R["Repo（Repo 模块）"] --> P["Project 0..N"] | M | N1 04:7、04:9；M1 open-questions.md:15 | 乙 | 无标签的箭头读成 Repo 包含 Project。给边加标签「每个 Project 关联一个 Repo；同一 Control 可多个」，或改成 P 指向 R 的「关联」边 | 无 |
| A8 | docs/design/task.md:27 | Project 可连接多个任务源，各有一个 Kanban 入口 | M | N5 04:65–67；spec/task.md:26（仓库绑源、Project 存源引用） | 乙 | 「连接」与第 18 行「仓库绑零到多个」没说清是同一批源。改为：「需要看板时，Project 可选用该 Repo 已绑定的一个或多个任务源，按源进入；只开聊天室也可以」（Codex：正文不解释「源引用」字段，也不把接源写成每个 Project 的前置） | contract-tests.md:62–63；S3.P6 |
| A9 | docs/design/task.md:40（§无 Run 的轻量路径） | 有权的人在看板预览精确的评审对象与验收证据 | N | N7 04:49；M6 spec/task.md:105–107；spec/project.md:43 | 乙（v2 改判：GLM——规则已在 spec/project.md:43，正文缺的是指针句） | 加一句：「开放 Task 有当前契约的候选交付、没有 Run 占用时，向有权验收的人提示待确认，从 Project『待你处理』进入；具体判定见 Task 约束」（Codex：已完成 Task 与旧契约的交付不重新进待办） | contract-tests.md:77；S3.T9 |
| A10 | docs/design/repo.md:39 | 第二种是只在本地的仓库：没有任何平台承载它。注册时按持久意图在本地平台建仓 | M | O10 open-questions.md:23–27；02 §P1 | 缺落点 | 正文没讲带 remote 的本地检出想另起独立工作怎么登记。加一句：「已有 remote 的本地检出仍推荐登记为第一种；要用本地平台另起独立工作时，默认另建独立副本并登记为新的本地 Repo，不把原外部 Repo 的身份换绑到本地平台」；输入目录与 remote、原地切换须确认两条分别验 | S3.P2、S3.P3（Codex：S3.P1 只验双 Project）；CT-REPO 注册行 |
| A11 | docs/design/repo.md:51 | 有权的人从 Project Room 发起写入型 Room Invocation（单次调用） | M | M4 open-questions.md:62；spec/project.md §Room Invocation | 乙（低） | 走查示例可保留，补「Topic Room 同样可以发起，范围按具体调用批准」，免得读成只有主 Room 能发起写入型调用 | 无 |
| B1 | docs/design/vision.md:9 | 交付物与承诺以 Git 仓库为边界，协作与治理随用户走 | O | O6 04:69、04:181；N1 04:7、04:9 | 乙 · 请核措辞（v2 改判：Codex——落实既有裁决，行为不变；v1 判丙）· 族「语义范围」（B1、B2、B3、D1）· 待裁 A1 | 候选 a（Codex，v2 推荐）：「围绕代码仓库开展工作，每个 Project 分别组织自己的讨论、承诺与验收；协作与治理随用户走」；候选 b（主笔 v1）：「交付物落在 Git 仓库里；承诺、授权与验收以 Project 为范围——一个 Project 当前关联一个 Repo，同一仓库可以有多个 Project；协作与治理随用户走」——b 的「交付物落在 Git」是新的存放断言，不取 | 无 |
| B2 | docs/design/vision.md:113 | 按仓库划分语义范围的项目语义控制面（project semantic control plane：控制面归用户级，语义范围以 Repo 为界） | O | O6 spec/task.md:46；N1 04:9；同族 spec/system.md:22 | 乙 · 请核措辞 · 族「语义范围」· 待裁 A1 | 候选（Codex）：「控制面随用户走，以 Project 组织各份独立工作；Project 关联 Repo，但同 Repo 不合并承诺与授权」——不从「全以 Repo 为界」摆到「全以 Project 为界」，Repo 注册与共享配置仍有自己的范围 | 无 |
| B3 | README.md:3 | 它的交付物与承诺以 Git 仓库为边界，协作与治理随用户走 | O | 同 B1 | 乙 · 请核措辞 · 族「语义范围」· 待裁 A1 | 与愿景一句话定位同改（B1 候选 a） | 无 |
| B4 | docs/design/vision.md:169 | 仓库级 Harness 目录与能力探测 | U | U1/U2 04:32；spec/participant.md:20「由 Agency 申报…安装位置与逐主机清单归 Agency，控制面不持有」 | 甲 | 「发现并选择可用的参与者，清楚知道它们能做什么」（Codex：愿景层不写「Agency 申报」这种机制语言；申报留在 Participant 正文与约束） | 无 |
| B5 | docs/design/architecture.md:92 | 例如施工图从 Room 的塑形讨论中产生，因此归 Room，而它的批准与版本对象归 Run | M | #261 F1；N6 04:77；02 §T3；M3 spec/run.md:72 | 乙 · 族「施工图来源」 | 「例如施工图可以从 Room 的塑形讨论中结晶，也可以从 Task 上按模板填表而来；结晶归产生它的场景，施工图的版本、批准与 Task 关联归 Run」 | contract-tests.md:89 |
| B6 | docs/design/architecture.md:86 | 决议、Memo（备忘）、施工图 | M | 同 B5 | 乙 · 族「施工图来源」 | Room 行的 artifact 格改「决议、Memo（备忘）、从讨论结晶的施工图」 | 无 |
| B7 | docs/design/architecture.md:31 | **四类单元和一个底座。** 单元的判据是两条：能独立安装、启动、停止，与别的单元只靠连接来往 | U | U1/U2 04:32；所有者今日原话；architecture.md:33 已有「同一控制面也可为它开多个独立 Project」 | 缺落点（K5） | 在控制面条目（:33）末尾只补：「Project 不是单元；多个 Project 可以复用所连的 Agency、内容服务与 Git 存储，也可以用不同的已接入服务或工作副本；共享资源不合并归属与授权」（Codex、Grok：04 说的是「可共用」，不写成必选拓扑；:33 已有同 Repo 多 Project 一句，不重复） | S1.V1a、S1.V1b 都要成立 |
| B8 | docs/design/architecture.md:21 | 打开仓库会连接本次选定或缺省的控制面——本机连接可在必要时拉起本机控制面 | N | N1 04:7、04:9；:21 前一句已写「创建或打开 Project…由人选定」 | 乙 | 保留本机拉起、远程选择与「仓库在哪不决定选哪个控制面」原句，只补「同 Repo 有多个 Project 时仍由人选定」（Codex；v1 的替换句会漏掉自动拉起） | CT-WORKBENCH-IA 自动拉起行 |
| B9 | docs/design/delivery.md:48 | 用户级“总入口对话面”：用户进入产品即在某个 repo 之下操作，这是显式设计决定 | N | N1/N2 04:7、04:28；architecture.md:21「产品从选择仓库、创建或打开 Project 开始」 | 甲 | 「用户进入产品即在某个 Project 之下操作（创建 Project 时选定它关联的 Repo），这是显式设计决定」 | 无 |
| B10 | docs/design/delivery.md:206 | 暂无。已裁决条目的去向见 | N | open-questions.md:78（三项仍需实现设计）；S3.T3 | 乙 | 列出「持续建议的触发与费用控制、提要的选材范围与生成方式、图形观察能力怎样交付——留待实现设计，不改约束，出处接手清单」 | S3.T3 |
| B11 | docs/design/delivery.md:78 | 或通过已验证的 Vikunja Done 映射请求同一命令 | M | delivery.md:17、:99、:111（B2 看板在平台 issues 上，本地任务服务器之后加绑）；非 #257 引起 | 乙 | 「或通过已验证的任务源原生 Done 映射（B2 时为平台 issues）请求同一命令」 | 切片 A 第 7 步 |
| B12 | docs/design/delivery.md:99 | Kanban 切片依次完成后端选择、Project 分组映射、Snapshot 导入 | M | #261 F3；M2 spec/task.md:32（映射可选） | 乙 | 「可选的原生分组映射（启用时才建锚点）」 | 无 |
| B13 | README.md:17 | 它记录所有者最新要求；上面的现行设计基线尚未完成对应的结构调整，不能将两者视为已经一致 | N | open-questions.md:5、:86 | 乙 · 族「状态文字」 | 两阶段（Codex）：首批改「#257 已同步直接相关关系，其余由 G 批核对；差异与未实施能力见接手清单」；全部子批合入后再改「已按体验目录核对」；同行「待拍板清单」改「已定事项与接手清单」 | 无 |
| B14 | README.md:69 | 下一步见[待拍板与接手清单] | N | open-questions.md:1 | 乙 · 族「状态文字」 | 链接文字改「已定事项与接手清单」，锚点保留 | 无 |
| B15 | docs/design/doc-discipline.md:15 | spec/connections.md 只定义模块交接，spec/system.md 只定义共享机制，delivery.md 只定义范围与验证，证据文档只记录来源。 | N | 04:3、open-questions.md:7「不代替模块约束」、README.md:17 | 乙 · 待裁 B6（v2 改判：Codex——体验目录已声明不定义状态与命令，这是补纪律句，不是行为取舍；v1 判丙 B 档） | 加一条：「`docs/user-experience/` 记录所有者确认的体验与原话，是已确认的需求来源，不定义对象、状态或命令；与约束冲突时修改规范并验收（决策史记转折、约束 bump、配 CT），落地前不把未改的约束冒充已对齐，也不以体验正文覆盖约束」 | 无 |
| B16 | docs/design/vision.md:83 | 需要澄清、决定或授权时，系统创建 Request（请求卡）并投影回 Project | N | N7 04:28、04:36；S3.R2 他人 Request 反例 | 缺落点 | 「需要用户处理的澄清、决定或授权，从 Project 的「待你处理」进入；其余 Request 仍作进度或阻塞投影」（Codex：不把所有 Request 都计给当前用户） | S3.R2 |
| B17 | docs/design/delivery.md:16 | 时间线、Composer、Trigger Preview、只读 Project Overview | N | N2/N3/N7 04:7、04:28；delivery.md:24 已要求走通；已裁 A3 = c | 缺落点 | P3 格补「Project 入口（主 Room 与待处理面板两入口、Rooms / Kanbans / Runs 并列列表）」；「只读 Project Overview」保留，与双入口并存、不多出必选导航入口（GLM-6：单分支） | 无 |
| C1 | docs/design/spec/project.md:25（对照 :49） | 已归档拒绝新 Task、Run 和写入型 Invocation；历史只读 | O | O8；spec/project.md:49「拒绝新的 Task、Run、Request、Artifact 发布与写入型 Invocation」；spec/task.md:18 第三份清单 | 乙 · 族「重复收口」（C1、C5、D4、D7） | 先把 task:18 的动作清单（拒绝创建、采纳、移动、重开、取消、完成）完整并进 project.md:49 的权威句，再把 :25 与 task:18 改成指针（Codex：三份清单不等价，直接删会让归档后老 Task 读成可改） | contract-tests.md:15 |
| C2 | docs/design/spec/project.md:49（对照 :47） | 恢复命令只恢复 Project 与 Project Room 接收新命令的资格，不复活历史 Task、Run、Invocation、Request、Topic Room、租约或外部副作用。 | O | O8；决策史 §32；:47「它们随 Project 一并转为只读……恢复 Project 后保持原状态」 | 乙 · B 档 11（已裁：无反对；Grok 要所有者点头，GLM、Muse 判两句矛盾归乙，Codex 按 §32 读成已有规则——v2 写「三席都读成已定」不准，Grok 交叉指出；「已关闭 Topic 不随 Project 恢复」是 B 档 11 的产品内容，不是从 :47 推出的） | :49 改「恢复后，随归档转只读的开放 Task、开放 Request 与未归档 Topic Room 恢复接收命令；已终态的 Task、Run、Invocation、Request、租约与外部副作用不复活；原本已关闭的 Topic 不随 Project 恢复」；:47 同句改（Grok-3），不留两种恢复语义 | contract-tests.md:15（补「恢复后开放 Task 仍只读时失败」与「已关闭 Topic 随 Project 恢复可写时失败」） |
| C3 | docs/design/spec/project.md:27（对照 :62、:183） | control 只处理治理事件（来源关联、调用与 Request 关联）和 Topic Room 的「创建/归档」命令 | O | O4；04:93；:62「人可以关闭 Topic Room」；CT:13 用「关闭」 | 乙 · 待裁 B7（命令叫「归档」、规则与 CT 叫「关闭」；若所有者认为关闭与归档是两态则升 A 档） | 命令统一为「创建/关闭 Topic Room」，:62 写「人可以关闭 Topic Room（关闭即该 Room 已归档；随 Project 归档则转只读）」 | contract-tests.md:13 |
| C4 | docs/design/spec/project.md:43 | Project Overview 是按单个 Project 聚合目标、健康度、Task、Run、Request、Artifact、变更与检查状态和近期活动的只读投影，不是独立场景或可写状态。 | N | N2/N3；04:28「子列表仍只有 Rooms、Kanbans、Runs」；04:110–112；CT:263 | 丙 · 已裁 A3 = c 最小版本（§八） | 定义句保留，其后加「不要求独立导航入口；不能替代主 Room 与『待你处理』入口」；:43 拆成两个锚点——Overview 定义句一锚，「待你处理」投影一锚（D8）；不删能力、不新增入口（Grok-4、GLM-6：改法列只留已裁的单分支） | contract-tests.md:263（按 E4 新句）；S3.R2 / R3 链接改指「待你处理」锚点 |
| C5 | docs/design/spec/task.md:18、:57、:59、:87 | :57「control 只追加 Snapshot：不改 Task 的 Project」；:59「系统不改变 Task 的 Project 归属，也不把不可变 `project_id` 改成新分组」（另 :18 身份句、:87 无命令句） | O | O2；spec/project.md:41 | 乙 · 族「重复收口」 | 同一规则同文四写。保留 :18（身份）与 :87（无命令）；:57 只留「只追加 Snapshot，不冻结采纳、启动、完成或字段写入」；:59 删首句，只留「移动 Task 只改阶段与排序；跨源的相对移动拒绝；换家不做」 | contract-tests.md:62、:63 |
| C6 | docs/design/spec/README.md:81；docs/design/spec/task.md:123 | README:81「结晶归属与对象归属分开，先例是施工图：它从 Room 讨论中结晶、归 Room 场景，对象与写入者归 Run」；task:123「施工图（Workflow Revision）从 Room 讨论中结晶、归 Room 场景」 | M | #261 F1；02 §T3；spec/run.md:70「显式标记的直接文本」、:72 从 Task 登记 | 乙 · 族「施工图来源」 | README:81 改「先例是施工图：由 Room 讨论形成时，其讨论结晶归 Room；模板等其他来源不要求先有 Room 讨论；版本和批准归 Run。按实际来源判断，不按登记形式推断」（Codex：直接文本也可能来自 Room 消息，v1 的「模板与直接文本没有 Room 结晶」不成立）；task:123 删该句只留指针 | 无 |
| C7 | docs/design/spec/run.md:64 | Verdict、Gate Receipt 与凭证链是 Workflow 场景的结晶（“干成了的证明”）：权威在控制面存储，结晶副本按[系统存储约束]写入 Git。 | M | #261 F2；spec/README.md:79「治理正文与审计副本在控制面材料存储」 | 乙 · 族「哪份 Git」 | 没说是材料库的 Git 还是代码仓库。改「权威在治理记录，审计副本在治理材料；公开范围不随代码仓库隐私推定」 | 无 |
| C8 | docs/design/spec/run.md:70 | 产出调用的 Context Bundle 条目只含施工图与清单快照，不含来源 Room 的任何消息条目；产出者的选入记录不属于来源 Room 在批准时刻的名册快照 | M | #261 F1；02 §T3（模板路径也要独立读回）；CT:89、:115、:116；已裁 A5 = a（§八） | 丙 · 已裁 A5（§八）· 族「施工图来源」 | 加窄例外句：「按实际来源判断；确无来源 Room 时，只有该 Room 名册回避项记不适用；从零调用、只交付图与清单、无任何 Room 消息读取权限仍检查；产出配置回避仍按现有显式声明及缺失 / 未知规则判断」；正反例写进 CT:115 紧随其后（Codex-7）；Codex-5 两处联动；本行随施工图来源族在第 2 批落 | contract-tests.md:115、:116（Codex-7）；S3.T8 |
| C9 | docs/design/spec/task.md:133 | “后端离线”本身不放宽 Project group、drift 或 CAS 前置 | M | M2；spec/task.md:32 | 乙 | 未定义词「Project group」。改「后端离线本身不放宽该动作适用的当前回读、契约分歧或版本比较前置」；是否需要分组只按自动认领条件判（Codex：直接换成「原生分组映射前置」会与「认领后脱离分组不冻结」冲突） | 无 |
| C10 | docs/design/spec/task.md:26 | 注册仓库或首次启用看板时，人从候选列表显式选定仓库级**缺省任务源** | N | N5；02 §P2（接入 Kanbans / Source）；CT:61「接入源」 | 乙 | 术语三套（约束「启用看板」、体验「接入 Source」、CT「接入源」）；「首次」指仓库第一次还是每个 Project 第一次未写清。统一为「Project 接入任务源」；缺省源句改「注册仓库时，或该仓库第一个 Project 接入任务源时」 | contract-tests.md:61 |
| C11 | docs/design/spec/task.md:28（另 :30、:34；术语表 :60；CT:63） | 可选的**合并板**只汇总这些来源，不替代按源入口，不是对象或权威表 | N | N5；04:71；open-questions.md:66「汇总投影是否提供另行设计」 | 乙（低） | 约束层给体验层「另行设计」的东西定了名；名字不一致（合并板 vs 汇总显示 / 汇总投影）。三处改条件句「若提供跨源汇总投影……」并改口「汇总投影」；统一名称时术语表 :60 与 CT:63 一并改，不留两个像是不同的概念（Codex） | contract-tests.md:63；glossary.md:60 |
| D1 | docs/design/spec/system.md:22 | 它对每个 Repo 保持独立的语义范围，而不是在每个仓库副本各起一套控制面；多个控制面对同一仓库各有一份语义范围，互不归并。 | U | N1、U1；04:7、04:9；spec/project.md:37 | 甲（v2 改判：Grok——同仓多 Project 之后这是旧句，不是含混）· 族「语义范围」（约束句先改；愿景句见待裁 A1） | 「控制面管理自己登记的 Repo 及各 Project；同 Repo 的 Project 各自保存工作和授权，Repo 级代码与平台事实仍按 Repo 管理；不按工作副本另起控制面」（Codex：不写「Repo 及 Project 各有独立语义范围」继续含混；本控制面同目标意图互斥与单写者不因此拆成每 Project 一份） | contract-tests.md:242、:264 |
| D2 | docs/design/spec/connections.md:16 | 引用还必须携带所属 Repo/Project、生产者和适用绑定版本。 | O | O2、N1；04:9；connections.md:216 | 乙 | 「引用保留对象既有作用域；Project 内的工作显式携带该 Project，涉及 Repo 时核其与 Project 的关系；共享定义与绑定（Worker Profile、工种、Skill、端口绑定）按原作用域引用；生产者、精确版本与适用绑定仍必需」（Codex：不能令所有共享对象虚构 Project 归属，也不能省掉生产者与授权归属） | contract-tests.md:264 |
| D3 | docs/design/spec/connections.md:75（拒绝条件在 :79） | Project 是必需且活跃的授权来源，Task Revision 是 0..1 个可选绑定； | O | O5；spec/run.md:74；04:98 | 缺落点 | 连接权威没写同 Project 条件：:75 加「所绑定的 Task 必须属于同一 Project」，:79 拒绝清单加「Task 的 Project 与本次 Project 不一致」 | contract-tests.md:89 已有失败例 |
| D4 | docs/design/spec/connections.md:50（同句另见 spec/repo.md:47、spec/project.md:37、:150；权威 :39；CT:10） | 主 Room 随独立的创建 Project 命令建立，不随 Repo 注册建立；不写代码树身份、不挂接工作副本 | N | N2 | 乙 · 族「重复收口」 | 「注册不另建仓库级 Room」在约束层写了五遍：权威留 project.md:39，其余改指针或删；只删重复的「主 Room 建立」断言，保留「不写代码树身份、不挂接工作副本」、待确认 Repo 限制与「纯研究仍归精确 Project」各自的规则；CT 是验收不是第二权威（Codex） | contract-tests.md:10 |
| D5 | docs/design/spec/repo.md:120（同段 :118；spec/project.md:144；正文 repo.md:54、:61、task.md:47、delivery.md:82；CT:40、:286、:169） | 策略中的「须人显式确认」开关随本次授权冻结，仓库或 Project 之后改默认值不影响已接受的调用。 | M | M1、U2；04:112；spec/project.md:51；已裁 A2 = a 窄版本（§八） | 丙 · 已裁 A2（§八） | 开关的版本化缺省归 Project——Project 版本化设置的一部分，随对应 project_version 带入授权预览（Codex-6），本次授权冻结进 Execution Spec；:120 改「Project 之后改默认值不影响已接受的调用」；同段 :118 与 spec/project.md:144 同改归属词（GLM-5）；正文四处「仓库」改「Project」；不写「Repo 只声明平台能力」的「只」；不做双层覆盖；「同仓两 Project 不同缺省」作 CT-REPO 的用例外配置例，不进 S1 事实表 | contract-tests.md:40、:286、:169（Codex-4）、:213（Codex-6 的 Run 路径配对） |
| D6 | docs/design/spec/repo.md:155 | 本控制面的契约若事先声明接受他人或其他控制面完成的精确平台集成事实，Repo 模块回读并核验平台目标、源版本、实际合入结果和证据，作为既有 Evidence 交给 Task； | O | O6；spec/task.md:61「另一 Project 或另一控制面」；04:69 | 乙 | 「接受他人、本控制面另一 Project 或其他控制面完成的精确平台集成事实」；仍是本 Task 依事先契约独立验收，不共用完成凭证。场景：同卡两 Project，_01 的 Run 已合入，_02 的契约要求远端 ref 已合入 | contract-tests.md:75「事先声明接受精确平台集成证据且 Repo 回读核验版本、目标与结果匹配时，该机械项通过」；S1.I5（Codex：v1 引的 :65 测的是写回事件来源，不验合入证据） |
| D7 | docs/design/spec/participant.md:82 | 为每个配对的控制面提供一个隔离租户，并对租户内的每次派工负责： | U | U3；system.md:270「本节是全库安全文本的唯一落点…别处只引用」 | 乙（低）· 族「重复收口」 | 加指针：「（定义见安全策略面「租户隔离」行）」 | contract-tests.md:136、:212 |
| D8 | docs/design/spec/connections.md:216；S3.R2 / R3 的链接文字 | 主 Room、Topic Rooms、按源看板、Runs 与[待处理投影]均以该 Project 为查询范围 | N | N7；spec/project.md:43 定义「待你处理」却挂在「Repo 注册与 Project 归档」节下；已裁 A3 = c | 乙（结构） | spec/project.md 给「待你处理」投影单独加锚点（Overview 定义句另有自己的锚点，见 C4），本行链接改指它；S3.R2 / R3 现写的 `[Project Overview](../spec/project.md#repo-注册与-project-归档)` 改文字为「待你处理」并改指新锚点（Muse-1）；旧归档锚点仍用于归档规则，不全局替换（Codex） | contract-tests.md:21、:265；S3.R2 / R3 |
| D9 | docs/design/spec/connections.md:12 | 同一控制面的跨 Project 或模块命令不得拆成工作副本的本地事务再拼接。 | O | O2、O9；spec/task.md:77 删源卡预览列本控制面全部绑定 | 乙（低） | 改「同一控制面内一次合法连接命令的领域结果、来源关联与必要 outbox 同事务提交，不按工作副本拆开」（Codex、Grok：删源卡预览就是跨 Project 的 content 动作，v1 的「现无跨 Project 命令」不成立；新句也不暗授跨 Project 写权） | 无 |
| E1 | docs/design/contract-tests.md:11 | Project 分组与 Room anchor 可重建 | M | #261 F3；M2；CT:38 已覆盖 Room 换绑重建、CT:67 已覆盖 Task 锚点重建 | 乙 | 删；不再留一条判不了失败的概括 | contract-tests.md:11、:38、:67 |
| E2 | docs/design/contract-tests.md:68 | Project 分组映射（父任务/milestone/标签降级）有测试 | M | #261 F3；M2；与 CT:67 同题且无失败输入；spec/task.md:32 允许父实体、milestone 或获准标签中任一稳定映射 | 乙 | 并入 :67 时写「所选获准映射不能稳定、无歧义回读时仍自动认领则失败；缺其他分组类型、不影响当前映射时应通过；无能力伪造锚点失败」，删本行（Codex 推翻 v1 写法：只有稳定获准标签也能自动认领，v1 的失败条件比约束强，等于新增限制） | contract-tests.md:67–68 |
| E3 | docs/design/contract-tests.md:67 | 或跨源归组去读外源分组时失败 | O | M2（spec/task.md:34 现句「不去外源补建分组」；「跨源归组」一词已不在约束层） | 乙 | 改「Task 已认领后，因源内分组变化改写它的 Project 归属，或为此向外源补建分组时失败」，并保留有获准无歧义映射的首次自动认领正例（Codex：v1 的「拿外源分组决定 Task 的 Project」会把合法的首次自动认领也拒掉） | contract-tests.md:67 |
| E4 | docs/design/contract-tests.md:263 | 单 Project Overview 与全局「需要关注」都是可重建的只读导航投影，不产生新场景或写状态 | N | N7；已裁 A3 = c；「需要关注」是对象标记，不是「待你处理」 | 丙 · 已裁 A3（§八） | 改成（Grok-4、Muse-2、Codex 同向）：「单 Project Overview 是可重建的只读投影，不产生新场景、写状态或独立导航入口，不能替代主 Room 与『待你处理』；跨 Project 的『需要关注』汇总若提供，保留各 Project 来源与授权、不归并事项；『需要关注』标记本身不增加『待你处理』条目；Change 场景仍归 Repo，Overview 只投影它」。检验：读 Overview 不得解决 Request 或提交发布；只有关注标记、没有待本人处理事项的对象不增加待处理计数；CT:265 / :266 继续验两个入口与动作回原模块，不复制成另一套用例 | contract-tests.md:263、:265、:266 |
| E5 | docs/design/references/glossary.md:14（核心产品词表） | 表内无「前情提要」「待你处理」「候选交付」条目；Topic Room 只在别的词条里带过；「任务源」条目无英文对照 | N | N2/N4/N5/N7；M6；open-questions.md:69 只写了保留旧词 | 缺落点 · 待裁 B9 | 补：Topic Room、前情提要、待你处理各一行（简释加权威指针）；「候选交付」写成「满足条件的既有 ChangeSet / Artifact 版本，不是新对象或 Task 状态」；Source 不新造词条——「任务源」现有条目补英文对照与「实际来源及范围」，Kanban 入口是它在 Project 里的呈现（Codex） | 无（词汇表非规范） |
| E6 | docs/design/references/glossary.md:26 | 持久的多参与者协作空间，分 Project Room（主 Room）与 Topic Room；也是 Project 模块的场景名 | N | N2 04:7；spec/project.md:39 三个名字并用 | 乙 | 别称挂在 Project Room 上，不挂泛称 Room：「Project Room（主 Room；所有者与用户流程说的『主 Repo Room』指同一间，不是另一种 Room）」（Codex） | 无 |
| E7 | docs/design/scenarios/S1-multi-unit.md:66、:68 | 「01 §五 推论；所有者把「两控制面同改一张卡」归多写通则（原 S1.I12）」；:68「01 §五（Kimi）」 | O | 「01 §五」指案例研究 `.memo/design/case-study-20260907/01-unit-model.md` §五「看板：多源（v3）」，不是体验 01（Codex；主笔核该节存在） | 乙 | 改成这份历史材料的准确链接，并指向现行 `spec/task.md` §契约与来源、§写入约束 的处置；保持「用例外」，不改成「必然发生」 | S1.X2、S1.X4 |
| E8 | docs/design/scenarios/S3-user-journey.md:43 | CT-PARTICIPANT 的票据与输入用例、CT-WORKBENCH-IA「Run 导航隐藏等待或暂停中的活动 Run」 | N | contract-tests.md:292 要求引用描述文本；:144 票据与权限分离、:137 允许原生交互（`native_interactive_allowed`）、:138 受管输入（`managed_single_writer`）、:136 是租户隔离（主笔核） | 乙 | 分别逐字引用三行描述文本：票据与权限分离（:144）、允许原生交互（:137）、受管输入（:138「`managed_single_writer` 下绕开有效输入授权的动作不得落到该派工；两个客户端同时持有同一目标的输入租约时失败」），按 S3.R4 所验路径引用；保留 CT-WORKBENCH-IA 的 Run 导航反例（Codex 交叉：v2 把 :137 叫「受管输入行」不准） | contract-tests.md:144、:137、:138 |
| E9 | docs/design/contract-tests.md:13（对照 :88、:101） | 关闭 Topic Room 后关联 Request 被解决、Task 被取消、Run 被停止或其待处理入口消失时失败 | O | O4 04:93；O5 04:94 | 缺落点 | CT-RUN 补一行：「Room 或 Task 引用 Run 不改其 Manifest、任务书或授权；Run 因被第二个 Task 引用而对该 Task 签完成、或引用方关闭 / 取消时 Run 被停止、Manifest 被改写时失败」 | contract-tests.md:13、:88、:101 |
| E10 | docs/design/contract-tests.md:16 等 13 行（:16、:23、:50、:52、:130、:148、:149、:226、:230、:231、:236、:245、:282） | CJK 输入、结构化引用、草稿/游标/未读、并发流隔离；时间线顺序以 chat server 给出的为准，治理引用只按事件 ID 冻结 | M | contract-tests.md:6「失败用例」；早于本批 | 丙 · 待裁 A4（范围） | 主笔逐行分两组（Codex）：只有题目、没有失败输入——:16、:23、:50、:130、:149、:226、:230、:231、:236、:245 十行；已可判失败——:52（过期邻项移动重算）、:148（attach 不能恢复 Run/Invocation 语义）、:282（正常成功保持安静）三行留在验收里不动。十行的处置见待裁 A4；本批新改的约束照配失败例，不借「另批」搁置 | 上列各行 |
| E11 | docs/design/contract-tests.md:22（对照 spec/project.md:43、S3:41） | 存在待本人采纳的契约 Snapshot 的 Task | N | N7 | 乙 | CT 与 S3 用同一词「存在待本人采纳的契约变化的 Task」 | contract-tests.md:22；S3.R2 |
| F1 | docs/user-experience/README.md:74 | 仍待复审、合入以及产品实现和行为测试 | 状态 | #261 F4；#257 已合 7a4fd40 | 乙 · 族「状态文字」（F1–F8、B10、B13、B14） | 「…（#257，v0.18.7 已合入）；产品实现和行为测试仍待分别验证」 | 无 |
| F2 | docs/user-experience/open-questions.md:86 | 尚未完成：本 PR 复审与合入、产品实现和真实行为验收。下一步审核体验、约束、场景与 CT 是否一致，不再等待重拍 Q1–Q3 或 C1/C2 | 状态 | #261 F4 | 乙 · 族「状态文字」 | 「尚未完成：产品实现和真实行为验收。一致性核对由 G 批承接（#262），不再等待重拍 Q1–Q3 或 C1/C2」 | 无 |
| F3 | docs/user-experience/03-terminology-confession.md:3 | 待本 PR 复审确认后再判退役，不提前删历史 | 状态 | #257 已合；03:56–60 三项退役条件 | 乙 · 族「状态文字」· 待裁 B10 | 「#257 已合入；退役按 §什么时候可以退役 的三项、随 G 批收口判定，不提前删历史」 | 无 |
| F4 | docs/user-experience/03-terminology-confession.md:54 | 本 PR 扩大同步范围后，仍待复审核对以下条件，不由作者提前宣布完成。 | 状态 | 同 F3；三项条件与 #261 §五 完成判据同义 | 乙 · 族「状态文字」 | 「#257 已合入；以下三项由 G 批逐条核对，不由作者提前宣布完成。」 | 无 |
| F5 | docs/user-experience/03-terminology-confession.md:36 | 读 v0.18.6 的设计对象时逐项核对：同 Repo 可有多个 Project 仍成立；把 Project 主 Room、Topic Room 与额外的仓库级 Room 混在一起的地方需要改。 | N | 决策史 §38（基线已到 v0.18.7） | 乙 | 「v0.18.7 已按此同步设计对象；读更早版本时逐项核对……残留由 G 批处理」 | 无 |
| F6 | docs/user-experience/README.md:39 | 提到既有模型时标明“v0.18.6 的设计对象”，逐项核对与当前体验的异同 | N | 同 F5 | 乙 | 「提到设计对象时以当前基线（v0.18.7）为准；追溯更早版本时标明版本号并逐项核对异同」 | 无 |
| F7 | docs/user-experience/README.md:5、:31；01-multi-unit.md:106；03-terminology-confession.md:47；open-questions.md:19、:27、:35、:53、:55、:78、:85 | 以下是本 PR 已同步的文档落点，不是新的待拍板问题。（open-questions:53；其余各行同用「本 PR」，共 14 处） | 状态 | #257 已合入，从 main 进入时「本 PR」无所指 | 乙 · 族「状态文字」 | 「本 PR」一律改「#257」 | 无 |
| F8 | docs/user-experience/open-questions.md:80 | F 批应由其作者和所有者据此前提决定关闭或改题，不再重开已定归属与唯一范围。本轮不修改或关闭那份 PR。 | O | 所有者 09-19「归档」；#246 已于 09-19 合入 `fc2aad8`（晚于本批基线 `459cda6`） | 乙 · 族「状态文字」 | 「F 批已由作者与所有者于 2026-09-19 决定归档：#246 改为方案备忘的归档 PR 并合入，前提被 v0.18.7 取代」 | 无 |
| F9 | src/agency/README.md:17 | 产出只有四种建议（创建 Request、开 Scoped Room、雾毕业为 Task、更新 Project 范围） | N | N3 04:61；技能正文 hctl2-shaping/SKILL.md:35 已写「建议开 Topic Room」 | 甲 | 「开 Topic Room」 | 无 |
| F10 | src/agency/skills/hctl2-shaping/SKILL.md:3 | 在 Project Room 里把一个还说不清的目标审问成可拍板的决定、可指派的问题和可承诺的 Task。 | M | M4 open-questions.md:62；04:57 | 乙（Grok：这是塑形场所，不并入「施工图来源」族，动手时不与 A2 绑成一次语义改写） | 「在 Project Room 或 Topic Room 里把…」 | 无 |
| F11 | src/agency/README.md:3 | 参与者身份、授权、租约、代次和结果验收都不在这里，它们在 control 账本。 | U | CONSTRAINTS「禁用账本」；根 `BUCK` 的 `docs_tree` 不含 `src/**`，死名检查沿用该树 | 乙 · 族「账本残留」（F11、F12、F13、G1） | 「它们在控制面存储（治理记录）」；扫描扩围另列落点：让第一方 Markdown 经 Buck2 原生输入进检查、保护历史豁免与检查器夹具，先 dry-run 看命中量再定（Codex、GLM；待裁 B8）；本文件另有代理边界旧句见 Codex-2 | 无 |
| F12 | src/agency/skills/hctl2-design-review/SKILL.md:50 | 这条「必须」由什么强制：账本机制、适配器回读、还是只有文字？ | M | 同 F11 | 乙 · 族「账本残留」 | 「控制面机制（比较并交换、租约、代次、归约）、适配器回读，还是只有文字？」 | 无 |
| F13 | src/agency/skills/hctl2-shaping/SKILL.md:53 | 「一切产出只是建议，门在账本」「不写执行许可」是 HCTL2 的绑定 | M | 同 F11；引号内是本仓库措辞，不是外部引文 | 乙 · 族「账本残留」 | 「一切产出只是建议，门在控制面」 | 无 |
| F14 | docs/user-experience/02-user-journey.md:16 | 也提供断开原 remote、用 Gitea 另起一份独立工作的选项。 | M | O10 open-questions.md:23–25（默认另建副本、保留原目录及其 remote） | 乙 | 「也提供不接原 remote、用 Gitea 另起一份独立工作的选项（默认另建独立副本，见下段）。」 | 无 |
| F15 | docs/user-experience/open-questions.md:66 | 汇总投影是否提供另行设计，本轮既不要求保留强制合并板，也不禁止可选汇总 | N | N5 04:71；spec/task.md:28 已定义可选合并板口径 | 乙（与 C11 同题） | 「可选汇总投影沿用 spec/task.md §契约与来源 的口径（只汇总、不替代按源入口、不是权威）；本轮不要求实现强制合并板」 | 无 |
| F16 | src/agency/skills/hctl2-design-review/SKILL.md:117 | 把评审共识当裁决、把 04 旧稿当用例事实 | 术语 | 「04」现有两份：案例研究 04 验证器旧稿与体验目录 04 | 乙 | 「把案例研究 04（验证器）旧稿当用例事实」 | 无 |
| G1 | src/README.md:7 | 不产生账本或 Receipt | U | 同 F11 | 乙 · 族「账本残留」 | 「不产生治理记录或 Receipt」 | 无 |
| Codex-1 | docs/design/spec/system.md:259（对照 :184；spec/participant.md:29） | 所引用的精确用户级 Profile/Skill 定义与公开冻结配置的字节与摘要 | U/M | spec/participant.md:29「Skill 的内容不归 HCTL 存放：由 Agency 安装并申报」；:184 只存 revision/digest 引用；:259 后半句又排除「Agency 内部的运行与安装定义」（主笔核） | 甲（两条约束句冲突；B 批已定分责） | 分开写：备份控制面自己的 Profile 定义、已冻结的 Skill 引用 / 摘要 / 可核验性、自己承诺可取的精确治理材料与交付字节；不因引用 Skill 就要求控制面保管 Agency 的安装定义；已进入承诺保存范围的 Bundle 原文仍保留 | CT-SYSTEM 一致备份：仅缺 Agency 安装包不判备份不完整；缺控制面承诺保存的 Bundle 字节仍失败 |
| Codex-2 | src/agency/README.md:3、:11 | 按冻结的执行规格交付执行体端点；落在 `hctl2-control` 的 Herdr 适配代码旁 | U/M | B 批（#228 裁决，决策史 §37）：Agency 是唯一通路，控制面只经公开端口派工、取得派工引用；spec/participant.md §派工与观测 | 甲（代理边界旧句） | 改「经 Agency 公开端口接受冻结规格并返回派工引用」；控制面侧适配的是 Agency 的公开端口，运行时适配在 Agency 内部；保留「待建」状态，不借改 README 宣称代码已实现 | CT-CONNECTION「Execution Spec 不携内部端点」；CT-PARTICIPANT 代理与故障恢复行 |
| Codex-3 | docs/design/run.md:40 | 执行体崩了，同一把椅子开新一次尝试。它是对象：崩了从这里重来，是恢复边界。 | U/M | contract-tests.md:135「控制面凭内部进程故障直接切换 Attempt 时失败，切换只在 Agency 报无法履约后按冻结规则」；S1.N8 | 甲（正文仍把内部故障当控制面换 Attempt 的条件） | 改「Attempt 是席位上的一次获准执行；Agency 能在原规格内恢复时不另开 Attempt，报告无法履约后，Run 才按冻结规则决定是否重试」；恢复细则指向 Participant 约束，联系不上不直接写成失败 | 复用 contract-tests.md:135 与 S1.N8，不新增 CT 族 |
| Codex-4 | docs/design/contract-tests.md:169 | 开关打开的仓库改为待处理、由人预览后提交 | O/M | 待裁 A2（D5）；spec/project.md:144 Execution Spec 冻结发布策略 | 缺落点（随 A2 联动） | 若缺省归 Project：本行按该 Project 及本次已冻结策略判，不再按 Repo 开关判 | 同 Repo 的 P1 开确认、P2 不开：各自新调用采用各自策略；改缺省不回写旧调用；把 P1 的开关套给 P2 或改活动 Execution Spec 均失败 |
| Codex-5 | docs/design/run.md:73；src/agency/skills/hctl2-shaping/SKILL.md:57 | 批准前找一个不在这个 Room 名册里的规划者；控制面找一个不在这个 Room 名册里的规划者 | M | 待裁 A5（C8）；S3.T8 | 缺落点（随 A5 联动）· 族「施工图来源」 | 若保留讨论路径的限定，写明只说有来源 Room 的情形并链接统一读回条件；没有来源 Room 时仍核从零调用、输入范围、消息权限与产出配置回避，不凭「直接文本」免检 | CT-RUN 读回行；S3.T8 |
| Grok-1 | docs/design/spec/task.md:32 | 两套分组并存：源内分组是后端的父实体、milestone、标签或过滤视图；本控制面中 Task 到 Project 的固定归属是另一件事。 | O | O6；spec/task.md:46 唯一范围已是 Project 内 | 乙 | 「本控制面中」易读成唯一范围仍在控制面。改「控制面记录的 Task 到 Project 固定归属是另一件事（实体到 Task 的唯一范围是 Project 内，见本节后文）」 | contract-tests.md:64；S3.P5 |
| Grok-2 | docs/design/architecture.md:98 | Project 关联一个 Repo，同一 Repo 可以对应多个 Project；各 Project 有自己的主 Room、Topic Rooms、任务源入口与 Runs。…Task 的卡留在原任务源 | O | O6、U2；04:69；S3.P5 | 缺落点 | 段后补「同一外部卡可被多个 Project 各自认领为 Task；共用源卡不等于共用契约、Run、授权或验收」——04 已裁、架构层未落的那句 | contract-tests.md:64–66；S3.P5 / P7 / T7 |
| GLM-2 | docs/design/run.md（§一次评审合入走一遍）、docs/design/task.md §关键规则 | （无——04:92「Task 不从属于讨论它的某间 Room」与 04:94「引用不等于承担完成该 Task 的责任」只在约束层有：spec/project.md:41、spec/run.md:74） | O | O4、O5 | 缺落点（镜像型：约束有、正文没向用户解释；GLM-1 并入此行） | 正文加一句用户可读的话：「Run 完成只说明它自己的完成条件成立；别的 Task 引用了这次 Run，不等于那个 Task 也完成了，各 Task 的验收各自独立；Task 也不从属于讨论它的某间 Room」 | 无（约束已有；CT 由 E9 补） |
| GLM-4 | docs/design/spec/connections.md:69–71（§Project → Task：从讨论到承诺） | Task 模块先以比较并交换校验 Project 和可选当前 Task Revision | O | O2、O5；connections.md:216「从本 Project 发起的关联与命令须核对目标归属」；spec/system.md:114 两类命令来源、CT:132 Result Proposal 通道不能提交治理命令（Codex 引） | 乙（缺指针：Run 侧 D3 补了同 Project 校验，Room Invocation 侧只在 :216 有总括句） | 加句（Codex 交叉改写）：「采用写入型 Room Invocation 的提案创建 Task 或采纳契约时，控制面核该调用冻结的 Project 与本次命令的 Project 一致；涉及已有 Task，还核其固定归属。Room Invocation 只提交提案，不因本句取得采纳命令权；直接客户端的合法命令不要求先有 Invocation」 | CT-CONNECTION 补：由同时有 A、B 权限的 human actor 提交，来源 Invocation 属 A、目标 Task 属 B，同 Repo 也拒绝；改为 A 的 Task 则成功；保留无 Invocation 的直接客户端合法采纳正例（Codex：直接拿模型 Proposal 发采纳命令会因 actor 非法被拒，测不出归属条件） |
| Codex-6 | docs/design/spec/project.md:51（§Repo 注册与 Project 归档）、:144（§Room Invocation）；spec/repo.md:118–:122；contract-tests.md:213 | Project 的目标、范围、角色和默认规则以单调 project_version 更新（:51）；……是否须人显式确认（:144） | O/M | 已裁 A2；两种授权入口——Room Invocation 在 Trigger Preview 冻结、Attempt 沿 Run Manifest 与 Execution Spec（CT:213） | 缺落点（A2 的定义句与 Run 入口的配对） | 在 Project 权威处（:51 附近）明确「『须人显式确认』开关的缺省是 Project 版本化设置的一部分，随对应 project_version 带入授权预览，所用策略随本次授权冻结」；Repo（:118–:122）只引用其归属、执行时只读已冻结策略；只替换开关持有者，不把策略里的 Repo、平台绑定版本等合法字段换成 Project；Room Invocation 直接冻结，Attempt 沿已冻结 Run Manifest、不在后续派工时重读当前缺省 | CT:40、:169、:286 保留并按 Project 判；CT:213 纳入同一配对：同 Repo 的 P1 / P2 采用不同缺省，分别启动 Invocation 与 Run；之后改 Project 缺省再派后续 Attempt，旧授权仍用原值；另一 Project 的缺省污染本次策略、或旧 Run 偷换值时失败；不另造 CT 族 |
| Codex-7 | docs/design/contract-tests.md:115、:116（对照 :89）；scenarios/S3-user-journey.md S3.T8 的验收指针 | 产出调用的 Bundle 含来源 Room 消息条目、产出调用带任何 Room 的消息读取权限、产出者在来源 Room 名册快照里……都拒绝（:115） | M | 已裁 A5 = a；spec/run.md:70「清单分散在多条消息时走直接文本并在来源链列出事件 ID」；Grok-5、Muse-3 同题并入 | 缺落点（A5 的正反例写成确切输入） | 写进 CT:115 紧随其后。正例：人从模板登记 W，确无来源 Room、显式声明无模型产出调用；读回者虽在本 Project 某 Room 名册中，本次仍从零调用、只获图与清单、无 Room 消息权限、各摘要匹配，应可批准，只把「来源 Room 名册回避」记不适用；缺这三项隔离仍拒绝。反例：把真实 Room 消息整理成直接文本、来源链列有该 Room 的事件 ID，按有来源 Room 判——产出者在该 Room 名册快照里或 Bundle 含该 Room 消息则拒绝，不凭「直接文本」免检。:116 的产出来源集合、显式无模型产出、普通空集合各用例原样保留作回归；S3.T8 指向这些例子，不把整段条件再抄一遍 | contract-tests.md:115、:116；S3.T8 |
| Grok-3 | docs/design/spec/project.md:47（对照 :49） | 开放 Task、开放 Request 与未归档 Topic Room 不阻止归档。它们随 Project 一并转为只读，不被隐式完成或取消；恢复 Project 后保持原状态。 | O | B 档 11（无反对）；决策史 §32 | 乙（随 B11 连带） | :47 删「恢复 Project 后保持原状态」，与 :49 写成同一句话：「恢复后这些开放对象恢复接收命令；已终态的不复活；归档前已关闭的 Topic 不随 Project 恢复」——不留两种恢复语义 | contract-tests.md:15：补「恢复后开放 Task 仍只读时失败」「已关闭 Topic 随 Project 恢复可写时失败」 |
| GLM-5 | docs/design/spec/repo.md:118（与 D5 的 :120 同段）；spec/connections.md:106 | 发布评审是本模块执行的持久外部副作用命令，其授权来源是归属者的 Execution Spec 冻结的**评审发布策略** | M | 已裁 A2；D5 同族 | 乙（A2 连带） | D5 改 :120 时同段过 :118 的策略描述与字段列举：字段清单不逐个加「Project」定语，段内凡说仓库的开关处与 :120 同改；connections.md:106 的字段块核过无归属词，不改字，动手 PR 说明里记一笔 | 随 D5 |

**计数（v3）**：89 行——甲 8、乙 63、丙 5（五项已裁，见 §八）、缺落点 13。v2 为 85 行，交叉轮新增 4 行（Codex-6、Codex-7、Grok-3、GLM-5），Grok-4、Grok-5、Muse-1、Muse-2、Muse-3、GLM-6 并入既有行的单分支改写；v1 为 76 行；v2 改判 5 行、按席位替代写法改改法二十余行、新增 9 行、拒绝 6 条补漏（理由见 PR「作者说明 · 第一轮汇总与 v2」）。六组与四席都没有发现需要推翻 #257 裁决的句子；#257 列过的落点已改对，问题集中在它没列的篇章、跨层重复与合入后的状态文字。

**走查席的覆盖声明**：组 A 七篇、组 B 五篇、组 C 四篇、组 D 四篇、组 E 五篇、组 F 体验五篇与四份技能文件都从头读到尾；WRITING-GUIDE.md 只按旧词扫描并读命中行；决策史与 docs/research 不在范围。

## 四、用例走查：五条主线

| 主线 | 走查输入 | 需要一致的地方 | 清点表对应行 |
| --- | --- | --- | --- |
| 同 Repo 的多个 Project | S1.1、S3.P1：人在同 Control 显式建 A、B | 主 Room、查询范围、默认设置、授权、归档不因 Repo 相同而合并 | A1、A7、B1–B3、B8、B9、C10、D1、D2、D4、D5、Codex-4、Codex-6、GLM-5 |
| 同卡、不同 Project、两个 Task | S3.P5 / P7 / T7：分别认领，观测标题与 Done，再取消一方或删共享源卡 | 身份、对账、自动写回、完成、幂等、删除预览保留各自 Task 的含义；不复制源卡、不互相完成 | A8、C5、C11、D6、E1–E3、F15、Grok-1、Grok-2 |
| Room / Task / Run 交叉引用 | S3.T4 / T6 / T10：独立选人，关一间 Topic，Request 升级为 Topic | 引用不搬归属、不继承授权；关 Room 不结束关联工作；来源无需补造 | A5、A6、C2、C3、C4、D9、E4、E9、GLM-2、GLM-4、Grok-3 |
| 从计划到交付 | S3.T8 / T9、R1–R7：保存模板计划、开工、观察、验收、失败处理 | 保存计划不等于启动 Run；执行、检查、评审、集成、Task 完成各自成立；待处理只汇总已有事项 | A2、A4、A9、B5、B6、B16、B17、C6、C7、C8、E11、F10、Codex-5、Codex-7 |
| 共用单元与材料交付 | S1.N3 / N4 / N8：共用 Git 对象库、异机凭据、Agency 不可达 | 部署共用不改治理归属；副本可读不等于获准；失联不等于执行已丢失 | A3、A10、B4、B7、D3、D7、F11–F13、F14、G1、Codex-1、Codex-2、Codex-3 |

**应保留的规则**（Codex，主线「共用单元与材料交付」；核对时不把它们再复述进四层，动手 PR 不得顺手改弱）：

| 用例步骤与系统要决定的事 | 现行答案 |
| --- | --- |
| S1.N3 多方共用 Git 存储，谁能写哪份结果 | `spec/system.md` §Repo 与执行现场、§单写者；`spec/repo.md` §ChangeSet 与 Git 事实、§写入约束 已分清对象库共用、工作副本互斥与获准目标更新；不因同 Repo 多 Project 制造多个控制面写者，也不放宽本控制面同目标待决意图互斥。B7、D1、D2 改后须与此一致 |
| S1.N4 不同机器与授权来源之间怎么交付精确材料 | `spec/system.md` §控制面自己的存储 的保存、准入、交付顺序；`spec/repo.md` §发布评审 的公开范围；`spec/project.md` §三种交付方式、§根 Context Manifest；`spec/connections.md` §从授权到派工——同摘要或已发指针不等于保全；交付精确获准副本不等于公开材料仓库或转交凭据；不需要新材料对象、新传输协议或每 Project 一套服务器 |
| S1.N8 Participant 不响应或 Agency 失联，是否另发工作 | `spec/participant.md` §派工与观测、`spec/system.md` §启动与恢复 已区分未知、无法履约与已完成待交；真正的冲突只在 Codex-2、Codex-3 的旧说明 |

交叉引用主线另核「用户能读到吗」：04:92–94 三句在约束层有 `spec/project.md:41`、`spec/run.md:74`，正文几乎没有（GLM），见 GLM-2。

## 五、待裁项（已拍板，落法见 §八）

改判的依据只有「行为变不变」。A 档里 A1 是请所有者核措辞，其余四项是行为取舍；B 档默认按建议做，所有者只标反对。

**A 档（所有者必须逐条答）。**

1. **「以仓库为边界 / 语义范围以 Repo 为界」这一族怎么改——请核措辞**（B1、B2、B3；约束句 D1 已判甲、先改）。背景：愿景一句话定位与 README 首句写「交付物与承诺以 Git 仓库为边界」，v0.18.7 定了承诺、授权、验收以 Project 为范围，同一仓库可有多个 Project。现状：读者按现句会把 mac_jssdk_01 和 mac_jssdk_02 读成一个范围。四席都选「四句同改」；Codex 指出这是落实既有裁决、不是新取舍，且 v1 文案的「交付物落在 Git 仓库里」是新的存放断言。选法：a）Codex 文案——vision:9「围绕代码仓库开展工作，每个 Project 分别组织自己的讨论、承诺与验收；协作与治理随用户走」，vision:113「控制面随用户走，以 Project 组织各份独立工作；Project 关联 Repo，但同 Repo 不合并承诺与授权」，README:3 同 vision:9；b）主笔 v1 文案；c）愿景保留「仓库为边界」只加半句「同一仓库可以有多个 Project，各自独立」。推荐 a。一句话定位是所有者的话，请过目。
2. **「发布评审须人显式确认」开关谁持有缺省**（D5、Codex-4）。背景：正文四处说「仓库自愿打开」，约束两处说「仓库或 Project」，CT 两处说「开了开关的仓库」。问题：同 Repo 可开多个 Project 之后，mac_jssdk_01 要求人再确认、mac_jssdk_02 不要求，两者同用 gh-jssdk——仓库级开关做不到。选法：a）这个开关的版本化缺省归 Project，本次授权冻结进 Execution Spec（四席一致；Codex 要求不写「Repo 只声明平台能力」的「只」——Repo 仍有平台绑定与权限等既有职责）；b）保留仓库级；c）两级、Project 覆盖仓库缺省，须写优先级规则。推荐 a。Codex 另提醒：「两个 Project 恰好要不同缺省」是本批提出的用例外配置，不由「独立 Namespace」自动推出，请所有者明答是否纳入。落点：正文四处、约束两处、CT:40、:286、:169。
3. **Project Overview 与「全局需要关注」**（C4、E4、B17、D8）。事实：Overview 在库里只出现四处——`spec/project.md:43` 的定义句、CT:263、delivery:16 P3 的一格、S3.R2/R3 两处链接文字（都指向「待你处理」所在的节）；它没有查询接口或字段定义，只是投影的名字；04 确认的导航里 Project 行只有主 Room 与「待你处理」，子列表只有 Rooms / Kanbans / Runs；约束层没有「全局需要关注」投影的定义，「需要关注」是对象上的标记，与「待你处理」不是同一个东西。席位三比一：Muse、GLM、Grok 选删；Codex 推翻——导航不决定对象是否存在，物理入口与只读汇总是两件事，保留定义句并注明「不要求独立导航入口；不能替代主 Room 与待处理入口」，CT:263 保留只读、不新增写权的测试，不能只因搜不到「全局」就改成只验待处理。选法：a）删定义句与 CT:263 的 Overview / 全局半句，「需要关注」只作对象标记，S3 链接文字改指「待你处理」自己的锚点（D8），跨 Project 汇总留给可选派生投影（同合并板处理）；b）Overview 并入主 Room 的只读摘要、不设入口；c）保留定义句，加「不要求独立入口；不能替代主 Room 与待处理入口」。推荐 a：删的是一个没有入口、没有字段的名字，不删任何已定义的能力；若所有者想给主 Room 头部摘要留个名字，选 c。
4. **十三行只写题目的 CT 短行**（E10）。主笔分组后：只有题目、没有失败输入的十行（:16、:23、:50、:130、:149、:226、:230、:231、:236、:245）；已可判失败的三行（:52、:148、:282）留在验收里不动。选法：a）本批补十行的失败输入；b）另立小批，本批只在 CT 文首标明这十行待展开（四席一致）；c）不动。推荐 b；本批新改的约束照配失败例，不借「另批」搁置（Codex）。
5. **读回规则在「没有来源 Room」时怎么判**（C8、Codex-5；v2 新升 A 档，GLM、Codex）。背景：`spec/run.md:70` 的读回规则四处引用「来源 Room」——不含来源 Room 的消息、产出者不在来源 Room 名册等；从 Task 选模板或直接文本登记的施工图可能没有来源 Room，条件无所指。不补则模板路径要么无法读回，要么为满足条件补造聊天，正是 §零 说的补造行为。选法：a）Codex 窄例外——「按实际来源判断；确无来源 Room 时，只有该 Room 名册回避项记不适用；从零调用、只交付图与清单、无任何 Room 消息读取权限仍检查；产出配置回避仍按现有显式声明及缺失 / 未知规则判断」，配无 Room 正例与「直接文本仍来自 Room」反例；b）主笔 v1 写法——「与来源 Room 有关的三项按满足处理；读回仍须是不进本 Project 任一名册的新调用，或另一 Room 的规划者」（Codex 指出它豁免了只交付图与清单的输入限制、又多加了选人限制）；c）不补，模板路径由人显式跳过读回。推荐 a；Grok、Muse 第一轮接受了 b，交叉轮再看。落点：`spec/run.md:70`、`run.md:73`、shaping 技能 :57、CT:89、:115。

**B 档（默认按建议做，只标反对）。**

6. 文档纪律加一条体验目录的分层句（B15；性质改乙，用 Codex 文案）。
7. Topic Room 命令词统一为「创建 / 关闭」，关闭对应现有「已归档」，与因 Project 归档而只读区分，不加第四种状态（C3；四席同意；若所有者认为「关闭」与「已归档」是两态，升 A 档）。
8. 评审技能「六批里反复出现的错」加两条（K8）——用短例补进原节，不平铺禁令（Codex）；dead-names 扫描扩围到第一方 Markdown**先不做**：先列 Buck2 落点（`docs_tree` 不含 `src/**`）、历史豁免与夹具保护，跑一遍 dry-run 看命中量再定（GLM、Codex）。
9. 术语表补 Topic Room、前情提要、待你处理，「候选交付」按既有版本解释，「任务源」补英文与范围、不新造 Source 词条（E5，按 Codex 修正）。
10. 03 术语纠正的退役随本批收口判定，按它自己的三项条件，不因 G 批合入自动退役（F3、F4；Codex）。
11. 归档恢复语义（C2）：:49 改「恢复后，随归档转只读的开放 Task、开放 Request 与未归档 Topic Room 恢复接收命令；已终态的不复活；原本已关闭的 Topic 不随 Project 恢复」——Grok 认为要所有者点头，Codex、GLM、Muse 认为 :47「保持原状态」已定；放 B 档，所有者一眼否决或放行。

## 六、已拍板不重开

#257 的全部裁决（04、`open-questions.md` Q1–Q3、C1/C2、决策史 §38；`open-questions.md:76`「不按旧拍板表逐项追问」）；#230 D 批（一张卡一个家、认领不搬家、两套分组、合并板是可选投影、换卡不做）；#243（依赖归源、只有阻塞进启动预览、Task 与 Run 一对多、Run 与评审请求一对一）；#244 安全策略面；决策史 §32 归档转只读；F 批已归档（#246）。本批的五项取舍已由所有者 2026-09-19 裁定（§八），按裁定落地、不另增取舍。

## 七、陪审团怎么审本批

三层，按顺序：**审题**——§零 的问题存在吗、「核对批」这个问法对不对、四个视角够不够、有没有本文没列的框架；**审解**——§一 的备选够不够、丙 是不是最优、清点表这种形状能不能承担「错的改、多的删、含混的定」；**审改**——§三 清点表逐行：原句引对没有（对照 `459cda6`）、性质判对没有（甲 / 乙 与 丙 的界线是重点：把该待裁的藏成措辞清理，或把措辞清理抬成待裁）、建议改法忠不忠于裁决、层对不对、缺落点有没有漏。三层各表态，再逐条维持 / 修正 / 推翻；补漏的行接编号（席位字母 + 序号）。每席另沿一条主线补漏：Codex——共用单元与材料交付、约束二；Grok——同卡不同 Project；GLM——交叉引用、体验目录；K3——从计划到交付。第一轮独立，不读他家。交叉轮（09-19 已走完，四席「可动手 / 改后可动手」，处置见 PR「作者说明 · 交叉汇总与 v3」）：核 §八 的落法忠不忠于裁定、连带落点有没有漏；撤回自己上一轮说错的要写明是哪条、为什么。

## 八、拍板（2026-09-19）

所有者原话：「5项 我都同意 codex的选择。」B 档六项无反对，按 §五 6–11 的建议做。逐项落法：

| 项 | 裁定（Codex 的选法） | 落到清点表哪些行、怎么写 |
| --- | --- | --- |
| A1 语义范围族 | a，用 Codex 文案；不写「交付物全落 Git」的新存放断言；保留 Repo 级代码与平台事实、共享配置的范围 | B1、B3：「围绕代码仓库开展工作，每个 Project 分别组织自己的讨论、承诺与验收；协作与治理随用户走」；B2：「控制面随用户走，以 Project 组织各份独立工作；Project 关联 Repo，但同 Repo 不合并承诺与授权」；D1 按其行改法（甲） |
| A2 发布评审确认开关的缺省 | a 的窄版本：这个开关的版本化缺省归 Project，本次授权冻结进 Execution Spec；不做双层覆盖；不写「Repo 只声明平台能力」的「只」 | D5：`spec/repo.md:120`、`spec/project.md:144` 改「Project 之后改默认值不影响已接受的调用」；正文 `repo.md:54`、`:61`、`task.md:47`、`delivery.md:82` 的「仓库」改「Project」；CT:40、:286、:169（Codex-4）按该 Project 与本次冻结策略判；「同仓两 Project 不同缺省」作 CT-REPO 的用例外配置例（标来源：本批），不进 S1 事实表 |
| A3 Project Overview 与「需要关注」 | c 的最小版本：保留只读投影定义，不要求独立入口，不能替代主 Room 与待处理入口；不删能力；「需要关注」不等于「待你处理」；跨 Project 汇总如保留仍只投影、不归并授权与事项 | C4：`spec/project.md:43` 定义句后加「不要求独立导航入口；不能替代主 Room 与『待你处理』入口」；E4：CT:263 保留只读、不新增写权的测试，措辞改成不把「需要关注」与「待你处理」混为一谈；B17：P3 格补双入口与三列表，「只读 Project Overview」保留；D8：「待你处理」在 `spec/project.md` 单独加锚点，S3.R2 / R3 的链接文字与落点改指它 |
| A4 十三行 CT 短行 | b 加分组：只有题目的十行登记为待细化、另立小批；已可判的三行不动；本批新改的约束照配失败例，不借「另批」搁置 | E10：CT 文首标明十行待展开（:16、:23、:50、:130、:149、:226、:230、:231、:236、:245），本批不改这十行；:52、:148、:282 不动；新改约束的失败例随各族 PR |
| A5 读回规则在没有来源 Room 时怎么判 | a：按实际来源判断；确无来源 Room 时只有该 Room 名册回避项记不适用，从零调用、只交付图与清单、无 Room 消息读取权限仍检查，产出配置回避仍按现有规则；反对「模板 / 直接文本必无来源」、三项视为满足、排除本 Project 全部名册 | C8：`spec/run.md:70` 加窄例外句，配无 Room 正例与「直接文本仍来自 Room」反例（CT:89、:115）；Codex-5：`run.md:73` 与 shaping 技能 :57 写明只说有来源 Room 的情形并链接统一读回条件 |

B 档（无反对）：B15 纪律句用 Codex 文案；Topic Room 命令词「创建 / 关闭」、关闭对应已归档；评审技能两条用短例补进原节，dead-names 扫描扩围先 dry-run、本批不做；术语表按 E5 修正版补；03 退役按它自己的三项条件判；C2 恢复语义按 v2 改法（含「已关闭 Topic 不随 Project 恢复」）。

交叉轮与读回：所有者在交叉轮前拍板。是否仍走交叉与读回由所有者定；主笔建议跳过——每行改法已有原句与新句，剩余风险由动手 PR 的轻审（乙行每篇抽两行）与合并后核对承担。动手按 §二.2 K6 分四个 PR：(1) 机械族；(2) 施工图来源族与「哪份 Git」族（含 A5 落点）；(3) 语义范围族、归属与 Namespace 各行、重复收口族（A1、A2 落点）；(4) Overview、Topic 关闭、归档恢复（A3、B 档落点）。

**交叉轮后的连带落点（v3）**：A2 → Codex-6（定义句与 Run 入口配对）、GLM-5（同段 :118）；A3 → C4、E4、B17、D8 改成单分支（Grok-4、GLM-6、Muse-1、Muse-2）；A5 → Codex-7（正反例写进 CT:115，Grok-5、Muse-3 并入）；B11 → Grok-3（:47 同句改）。E8 改引 :144、:137、:138 三行。动手分批见 §二.2 K6（A5 在第 2 批）。

## 九、落地记录（2026-09-19）

| 批 | PR | 版本 | 作者 | 轻审 | 内容 |
| --- | --- | --- | --- | --- | --- |
| 1 机械族 | #266 | 不动版本 | Codex | Muse、GLM 可合 | 状态文字、账本残留、Scoped Room 旧称、术语表五词、S1/S3 引用、评审技能两条 |
| 2 施工图来源与读回例外 | #264 | v0.18.8 | 主笔 | Grok、Codex 修正后可合（三处已修） | A2–A4、B5、B6、C6–C8、Codex-5、Codex-7；A5 窄例外与 CT 正反例 |
| 3a 语义范围与单元 | #265 | v0.18.9 | 主笔 | Codex 修正后可合（四处已修）、GLM 可合 | B1–B4、B7–B9、B15、B16、D1、D2、D7、Codex-1、Codex-3、A1、A6、A7；#263 三处落点 |
| 3b Task 归属与开关缺省 | #267 | v0.18.10 | 主笔 | Codex、GLM、Grok 可合 | A8–A11、C1、C5、C9–C11、D3–D6、D9、E1–E3、Grok-1、Grok-2、GLM-4、Codex-4、Codex-6、GLM-5 |
| 4 Overview 与归档恢复 | #268 | v0.18.11 | 主笔 | Muse、Grok 可合 | C2–C4、E4、B17、D8、A5、E9、GLM-2、Grok-3 |
| 5 小尾巴 | #269 | 不动版本 | 主笔 | GLM 可合 | Codex-2（Agency README 代理边界句） |

合入顺序按所有者定：#266 → #264 → #265 → #267 → #268 → #269，均 squash；栈式分支每合一个 rebase 一次，台账与 S3 各解过一次冲突。教训两条：别家的 PR 由原作者合，主笔不替（#266 主笔误合，所有者点名）；rebase 后改 base 的 PR 不触发 CI，关开一次才跑。

合并后核对（Grok，#268）：热点未改坏；一处不一致（词汇表「待你处理」链接指向归档锚点）随收口 PR 修正；已登记、本轮故意没做的：A4 的十行 CT 短行另立小批（收口 PR 在 CT 文首标明）、B8 的 dead-names 扫描扩围先 dry-run、#263 的归纳机制单独出方案；K6 的状态文字第二阶段随收口 PR 落。03 术语纠正按其三项条件判退役，随收口 PR 落。

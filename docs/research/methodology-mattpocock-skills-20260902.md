# mattpocock/skills（Skills for Real Engineers）逐源码审计：wayfinder 与 grill 系、完成判定权专项、初审结论核验

> 类别：方法论生态 · 单对象补充审计——[方法论生态审计](./methodology-landscape-20260824.md)第八族「Harness 技能包当方法论」的头部样本<br>
> 日期：审计 2026-09-01 · 落库 2026-09-02<br>
> 状态：Informative 研究备忘录，不定义 HCTL2 语义；发布后正文不改，只在文末追加复核记录。复用判断用 [docs/research/README.md](./README.md) 的五种复用决策用语。<br>
> 方法：完整 clone 后全技能清单逐文件读源码（37 个 SKILL.md + 全部附件 + docs/ 下 23 篇人读长文 + .changeset + CHANGELOG + .out-of-scope），grilling/wayfinder/triage 逐 commit 演化考古（关键 diff 与 commit message 全文核对），GitHub API 独立复核仓库元数据，再对照 HCTL2 设计文档逐条裁决。宣传文（README 定位语、aihero.dev 链接）只当线索，结论一律钉源码。<br>
> 证据钉：mattpocock/skills @ `6654f6b60cd9d5be8b54c6fafe44346dabeb3b76`（2026-08-24，HEAD）。API 实测（2026-09-01）：243,499★、20,699 fork、created 2026-02-03、last push 2026-08-24、MIT、445 open issues。457 commits，Matt Pocock 一人 319 commits，其余为 agent 合著（Claude/Remote Box Agent 等）与 bot。HCTL2 侧的条款引用以节名给出，核对于 main @ `2863632`（草案 v0.15.4）；落库时已在 main @ `aaf9fd4`（草案 v0.15.6）逐条复核引文仍成立。

## 结论先行

1. **族归属维持：「Harness 技能包当方法论」族的头部样本，杂交「系统记录寄生」。** 全仓库的机制载体只有 prompt 文本——setup 自述「This is a prompt-driven skill, not a deterministic script」（setup-matt-pocock-skills/SKILL.md:15）；仅有的可执行代码（wizard 的 template.sh、hitl-loop 模板、装链脚本）不承载任何治理。规划状态全部持久化到 issue tracker：wayfinder 的地图是一张打 `wayfinder:map` 标签的 issue、票是子 issue（wayfinder/SKILL.md:21）、阻塞用 tracker 原生依赖关系（:69），to-spec/to-tickets 的产物直接是 issue（to-spec/SKILL.md:19、to-tickets/SKILL.md:63）。族的既有定性——技能层给不出「证据高于自述」、只当注入通道——被本仓库自己的事故记录三次实锤（见结论 2），不因样本头部化而改变。

2. **完成判定权：计划侧同向、施工侧违反、全线无强制——「同向碎片、无系统边界」定性的又一强证。** 这是 11 家横评里自觉度最高的一家：grilling 有人的确认门（grilling/SKILL.md:28）、triage 推荐后等人指令（triage/SKILL.md:72）、implement 故意没有完成步骤、关单留给人（docs/engineering/implement.md:51-53）。但每一道门都是 prompt 句子：`ready-for-agent` 状态标签由模型自贴（to-spec/SKILL.md:19「no need for additional triage」）；wayfinder 决策票由 agent 自己 close（wayfinder/SKILL.md:125）；implement 的 TDD 与 code-review 全是自述性证据——模型跑测试自报、模型评审自己刚写的代码，作者在 docs 里自己承认偏向（docs/engineering/implement.md:67），且实测常在 commit 前评审一个空 diff（:65，两侧未修）。他自己的记录提供三个「prompt 当强制层失败」实锤：agent 自答 HITL 票（commit e5932a7，学员实报，只能改措辞修复）；agent 往自己拥有的地图 Notes 里写执行豁免、后续会话读回当许可、在真服务器上开工（docs/engineering/wayfinder.md:69-70，「no hard in-skill stop」）；prototype 票 agent 三选一自选自关（:75-76，未修）。

3. **初审六组结论的核验结果：维持三组半、修正一组半、推翻一组。** 族归属维持；缺口一（决策票走无 Run 轻量路径）结论走得通但要补两个约束前置（答案文档须登记为 Artifact、契约须显式采纳），且映射比初审说的富——纯讨论型决策票对应 Scoped Room、索决定型对应 Request，只有产出答案工件的才需要轻量 Task，「不新增对象」的建议因此更稳；缺口二（雾无一等位置）缺口维持、处置修正为「Project 正文加一节，不新增对象」；缺口三（HCTL2 没管住「决定不能自述」）**推翻**——Request 冻结 required actor/role、普通 Room 回复不能解决 Request、模型不能自触发执行、纪要不由模型书写、作者不占评审席位，五处合起来就是决定判定权的机械覆盖，HCTL2 缺的只是 mattpocock 那句话，不缺那道门；两条印证维持；「不学机制」维持且查无例外。全文见核验表。

4. **最值得抄的不是流程，是几个被 457 commits 打磨过的词和两张纪律清单，全部仅参考行为。** (a) frontier/fog 词对：「前沿=前置全定、现在能问/能领的边」（grilling/SKILL.md:8、wayfinder/SKILL.md:69），「雾=范围内、还说不准问题的已知未知」，配判据「能否现在把问题问准，而不是能否回答」（wayfinder/SKILL.md:88）——可直接用于 HCTL2 的 Project 叙事与看板 readiness 文案。(b) grilling 轮次协议：一轮=整个前沿、按依赖分轮、每题带推荐答案、事实归 agent 决定归人、空前沿+人确认才收工（grilling/SKILL.md:8-28）——Trigger Preview 与 Request 应答面的交互参考。(c) AGENT-BRIEF 的「耐久优先于精确」写票纪律（写接口与行为约束、禁文件路径与行号，AGENT-BRIEF.md:9-17）与 HCTL2「契约冻结验收约束、不冻结施工步骤」（spec/task.md §契约与来源）同构，可进 Task Revision 写作指引。无一处够到采用为依赖、适配协议或移植组件。

5. **初审漏掉、真有价值的是三件，其余明说不值得。** to-questionnaire 的「问投递、不问议题」反转（采访只问给谁/要什么回来，问题瞄准知识差距，异步一遍过、每题带答案桩与 why-this-matters，to-questionnaire/SKILL.md:9-40）——HCTL2 Request 升级 Scoped Room 之前的「问卷式应答面」现成交互设计，仅参考行为。wizard 的「只给人做的步骤」纪律（不可逆动作先确认、agent 不得代跑、能自己做的不许调它，wizard/SKILL.md:3,37,43）——Request 的 human-action 类解决动作的交付形状，仅参考行为。.out-of-scope/ 按概念（非按 issue）组织的拒绝先例库＋「上次因 X 拒了，你现在还这么想吗」查重复述（OUT-OF-SCOPE.md:17,76）——与 HCTL2 decision-history 同构，产品侧 wontfix 治理可参考，仅参考行为。判不值得看的：handoff/PHASE-BOUNDARIES（HCTL2 不代管会话内上下文，接力只经结晶）、triage 五状态机（被其用户证明不够用）、teach/writing 三件套/wait-what（与本仓库无关）、retro（自认 STUB）、implement-spec（frontier 并发编排雏形，HCTL2 用 Run/Workflow 治理对象已解决）。

6. **两条反面教材可直接写进设计文档当否决理由。** (a) 一个 `ready-for-agent` 标签同时当「规格完备度」与「AFK 可领取」用，AFK 轮询 agent 把整份 spec 当票领走施工——作者自认「the most-reported rough edge」（docs/engineering/to-spec.md「Why does the spec get the ready-for-agent label?」节）——「同一 content 标签承载两种治理语义」的实锤事故，佐证 HCTL2 lifecycle/stage/health 三层分离、外部标签只作投影（spec/task.md §对象）。(b) 五状态机没有 blocked/deferred/implemented 的位置是该仓库用户最高频抱怨：「ready-for-agent technically true but misleading」、无终态凭证导致「an AFK runner can re-queue finished tickets」（docs/engineering/triage.md:76-77）——别家用户在替 HCTL2 要「阻塞是正交 health」（spec/task.md §对象）与「完成凭证是终态证明」（spec/task.md §写入约束）这两样东西。

7. **二审补充的分界原则：方法归 Skill，方法产出的东西归对象，方法管不住自己的地方归机制。** 判据只有一个：换一种方法论，这个东西还需不需要。按这条尺子，grilling 的轮次协议、wayfinder 的四型票与前沿判断全部归 Skill；「有人拍板的决定」「必须由人回答的问题」「范围边界」「还问不准的已知未知」归对象，HCTL2 只缺最后一格；「问题不能被模型代答」「完成不能自述」「规划空间不能给自己签执行权」归机制，HCTL2 已经齐了。这条原则同时回答了「机制会不会固化方法论」：HCTL2 固化的是「什么算作数」，不是「该怎么干活」。详见[二审补充](#二审补充新发现与分界原则)。

## 机制解剖

### 对象是什么

一人（Matt Pocock，319/457 commits，大量 commit 由 Claude 合著）维护的 agent 技能包仓库，2026-02 创建，2026-08-24 最后推送，MIT，243,499★。README 的自我定位：「Approaches like GSD, BMAD, and Spec-Kit try to help by owning the process. But while doing so, they take away your control」（spec/README.md §词汇分类法）——刻意反对方法论工具「拥有流程」，主张小而可改的组合件。源码大体支撑这个定位（每个技能几十到一百多行 prompt，无锁进），但控制并未真正回到人手里：约束与豁免同文（wayfinder Notes 事故），门全靠模型自觉。

骨架是一条主流水线加三个上匝道（ask-matt/SKILL.md:13-46）：

```
grill-with-docs（审讯出共识）→ to-spec（会话→规格 issue）→ to-tickets（规格→带阻塞边的 tracer-bullet 票）→ implement（逐票施工，内驱 tdd，收尾 code-review，commit）
上匝道：triage（外来 issue/PR 分诊）、diagnosing-bugs（硬 bug 诊断）、wayfinder（超会话规模的规划）
词汇底座：domain-modeling（CONTEXT.md 词汇表 + ADR）、codebase-design（深模块词汇）
```

技能分两种可达性（.agents/invocation.md:5-6）：user-invoked（只有人能敲，`disable-model-invocation: true`，负责编排）与 model-invoked（模型可自取，承载可复用纪律）；user-invoked 技能互相不可调用（:22）。每仓库一次 `/setup-matt-pocock-skills` 生成 `docs/agents/issue-tracker.md`（含「Wayfinding operations」节）、`triage-labels.md`、`domain.md`，技能经 CLAUDE.md 指针解析 tracker——一层朴素的「端口绑定」（d869d45 修过硬编码路径破坏该间接层的 bug）。

### wayfinder：一张 issue 当地图，四型决策票，雾与前沿

核心对象（wayfinder/SKILL.md）：

- **地图**：单张 issue，标签 `wayfinder:map`，「canonical artifact」（:21）；**索引不是仓库**——「a decision lives in exactly one place, its ticket, so the map never restates it, only gists it and links」（:23，commit 9272935 专门为此重构）。地图体四节：Destination / Notes / Decisions so far / Not yet specified + Out of scope（:31-53）；开放票不列在地图上，靠查询发现（:29）。
- **票**：地图的子 issue，正文只有一节 `## Question`，尺寸钉在「one 100K token agent session」（:57）；答案不进正文，解决时以评论落地（:71,125）。四型：research（AFK，子代理并行烧掉，:77,115）、prototype（HITL）、grilling（HITL，默认型）、task（唯一「做而不是决定」的型，靠解锁决定挣位置，:80）。
- **HITL 规则**：「A HITL ticket only resolves through that live exchange; the agent never stands in for the human's side of it (a grilling agent that answers its own questions has broken this)」（:75）。
- **claim**：开工前把票 assign 给驱动地图的 dev，「That assignee _is_ the claim」（:67）；GitHub 操作即 `--add-assignee @me` 作为会话第一笔写入（issue-tracker-github.md:44）。
- **阻塞与前沿**：优先 tracker 原生依赖关系，理由写明「it renders the frontier _visually_ in the tracker's own UI, so the human sees what's takeable without opening the map」（:69，commit b289481）；前沿=开放、未被阻塞、未认领的子票。GitHub 用 dependencies API 的 blocked_by 边（数据库 id，非 issue number，issue-tracker-github.md:42）。
- **雾**：「don't chart what you can't yet see」（:84）；雾还是票的判据是「whether you can state the question precisely now, _not_ whether you can answer it」（:88）；解决一票把雾里已能说准的部分「毕业」为新票并从雾里清掉（:126）。Out of scope 与雾严格分开：雾只朝目的地聚集，界外工作 close+留一行，永不毕业（:95-101）。
- **节奏**：每会话至多解决一票（research 例外，:105）；制图会话不解决任何票（:116）；解决=评论落答案+close+往地图 Decisions-so-far 追加一行（:125）。

演化考古（约 50 commits 集中在 2026-06-15 至 07-16）：decision-mapping 诞生（ab7196a）→ slug 当票 id（a116824）→ 更名 wayfinding/wayfinder（4027ea6/01f0b7e）→ 地图从本地 markdown 搬上 tracker（5c3c49d「make the map collaborative」）→ 索引化（9272935）→ destination 定为头号词（53c6219）→ 雾与界外拆分（7d34a8d/3ea0131）→ task 型加入（64d9f3d）→ claim 从 label 改 assignee（6f9e995，理由：在 tracker 原生 UI 里更自然）→ 原生阻塞边（b289481）→ HITL/AFK 分类修自答 bug（e5932a7）→ 「decision ticket」定名 + research 子代理并行（7d694b7/2602257）。曾短暂 model-invocable（a5c124e）后回退为 user-invoked。**演化方向一致：每个机制都从自造惯例挪向系统记录的原生结构（label→assignee、正文约定→原生依赖边、本地文件→issue）。** 这就是「系统记录寄生」的引力，也反向佐证 HCTL2 把操作字段交给任务后端原生能力、治理留在账本的拆分（spec/task.md 操作投影）。

### grill 系：一个原语、两扇门、一个轮次协议

grilling 是唯一 model-invoked 的原语；grill-me（无仓库时）与 grill-with-docs（有仓库时，加载 domain-modeling 留下 CONTEXT.md/ADR 纸迹）是两扇一行长的门（grill-me/SKILL.md:7、grill-with-docs/SKILL.md:7）；triage/wayfinder/improve-codebase-architecture 内部复用它。

协议（grilling/SKILL.md）：把议题建模为**设计树**（:6）；**前沿**=前置已定、现在能问的所有问题；一轮问整个前沿，每题编号并带推荐答案，等齐答案再重算前沿（:8,24）；**事实是 agent 的活**（派子代理去查，不许问人查得到的事），**决定是人的**（:26）；前沿空+人确认「shared understanding」才算完（:28）。演化：2026-03 还是「一次只问一题」（a6bdfd9「Asking multiple questions at once is bewildering」）→ 2026-07-16 改为轮次制（a4b2009，「Same 13 questions land in ~3 rounds instead of 13」）；确认门 2026-07-03 才补上（0e9a072）；事实/决定二分 2026-07-06 因自答事故补上（e5932a7）。docs 页诚实记录：轮次制「genuinely contested」，一次一题的退路受支持；「前沿是 agent 的判断，不是计算出来的图」；弱模型仍会冲破确认门（docs/productivity/grilling.md）。拒绝加问题数上限，理由是上限混淆两种失败（欠规格 vs 低质量问题），治理面是自然语言（.out-of-scope/question-limits.md:7-14）。

### 流水线其余各件（一句话+判点）

- **to-spec**：不采访、纯合成；先勾勒测试 seam 并与人核对（:15-17）；按模板发 issue 并自贴 `ready-for-agent`（:19）；规格里禁文件路径与代码片段，prototype 产出的决定性片段例外（:55-57）。
- **to-tickets**：切 tracer-bullet 竖片（每片穿全栈、可独立演示、尺寸=一个新上下文窗口，:31-35）；宽重构走 expand–contract 例外（:40）；发布前列清单向人质询颗粒度与阻塞边、迭代到人批准（:44-56）；真 tracker 上用原生阻塞边+自贴 `ready-for-agent`（:63）；「Do NOT close or modify any parent issue」（:67）。docs 页记录横切事故：26 票按层切、每票均值约 20 次 agent 运行、四分之三是返工（docs/engineering/to-tickets.md:30-31）；验收项「有些在起点就为真」的三种失效形状（:66-67）。
- **implement**：全文 15 行——按 spec/票施工、在预先议定的 seam 上驱 tdd、常跑类型检查、末尾跑全套测试、用 code-review 自审、commit 到当前分支（implement/SKILL.md:7-15）。没有关单步骤（详见完成判定权专项）。
- **triage**：两类别五状态的状态机（:30-37）；每条评论强制打头「This was generated by AI during triage」（:13-17）；先查重（按域概念搜已实现）与查先例（.out-of-scope/），再**推荐并等待指令**（:70-72）；写票前先验证声明（复现 bug / checkout PR 跑测试，:74）；wontfix 三分（已实现/拒 bug/拒 enhancement，只有最后一种进先例库，:82-85）。
- **code-review**：双轴（Standards/Spec）平行子代理，互不污染上下文；Standards 轴带 12 条 Fowler 坏味基线，仓库成文标准可压过基线、坏味永远是 judgement call（:38-56）；聚合端「Do not merge or rerank」（:76）。产出是报告，无裁决语义。
- **tdd**：红→绿循环参考件；「Test only at pre-agreed seams…confirm them with the user. No test is written at an unconfirmed seam」（:22）；反模式三条（实现耦合、同构断言、横切批量写测试）。重构被逐出循环、划给评审阶段（:39）。
- **diagnosing-bugs**：全仓库唯一把「完成判据」写成可核对 checklist 的地方——Phase 1 完成=存在一条已跑过、red-capable、确定性、秒级、agent 可独跑的命令（:57-66）；「No red-capable command, no Phase 2」。
- **wizard**：为「只有人能做的步骤」生成交互式 bash 向导；固定库+生成 stage；不可逆动作前 `confirm`（:37）；「Don't run it end-to-end yourself」（:43）。
- **domain-modeling / codebase-design**：CONTEXT.md 是纯词汇表（「totally devoid of implementation details」，domain-modeling/SKILL.md:64）、ADR 三条件（难逆/无上下文会困惑/真权衡，:68-73）；深模块词汇（module/interface/depth/seam/adapter/leverage/locality）与「一个 adapter 是假想 seam、两个才是真的」（codebase-design/SKILL.md:65）。
- **writing-for-agents**：给 agent 写文档的写作学——context pointer、两种负载、完成判据的 clarity/demand 与 premature completion、leading words、negation 失效、cache/sediment/no-op 测试（:12-81）。
- **handoff / PHASE-BOUNDARIES**：会话交接文档（临时目录、不复制已落盘工件只留指针、去敏、建议技能清单，handoff/SKILL.md:8-14）；相位边界五选项决策树（continue/clear/handoff/subagent/compact），核心概念是「除 Continue 外每一步都把一手来源变二手」（PHASE-BOUNDARIES.md:44-49）。
- **to-questionnaire**：把答不了的决定变成给特定他人的问卷；只审讯「投递」（给谁/要什么回来），问题瞄准知识差；最重要在前（异步可能只有一轮）、每题一个意思+答案桩+必要时一行 why-this-matters、明说「partial answers and "I don't know" are useful」（:9-40）。
- **in-progress 系**：implement-spec（整份 spec 单分支 PR，票当任务图、前沿并发、每实现子代理一个 worktree、merger 子代理合并、PR 标记 closing 全部票，:11-33）；retro（自认 STUB）；loop-me（个人工作流规格审讯，词汇里有 checkpoint/push right/brief）；claude-handoff、setup-ts-deep-modules、writing 三件套。
- **misc 系**：git-guardrails（PreToolUse 钩子拦危险 git 命令——全仓库唯一硬强制，但只防误操作不做治理）、setup-pre-commit、migrate-to-shoehorn、scaffold-exercises。

### 全技能清单（37 个，对 HCTL2 值不值得看）

| 技能 | 一句话 | 对 HCTL2 |
| --- | --- | --- |
| engineering/ask-matt | 全套技能的人读路由器 | 不值得（导航文档，无机制） |
| engineering/grill-with-docs | grilling+domain-modeling 的组合门 | 间接（机制在两个被组合件里） |
| engineering/triage | 外来 issue/PR 的五状态分诊 | **值得看**：verify-before-brief、AI 声明、wontfix 三分；状态机本身是反面教材 |
| engineering/improve-codebase-architecture | 扫码找「加深机会」出 HTML 报告再 grill | 不值得（编码方法论本体，Skill 层素材） |
| engineering/setup-matt-pocock-skills | 每仓库一次的 tracker/标签/文档布局配置 | 弱参考（tracker 指针间接层≈朴素端口绑定） |
| engineering/to-spec | 会话→规格 issue，自贴 ready-for-agent | **值得看**：标签双语义事故；seam 前置核对 |
| engineering/to-tickets | 规格→tracer-bullet 票+阻塞边 | **值得看**：竖片纪律、expand–contract、人批准门、验收项失效形状 |
| engineering/implement | 逐票施工+tdd+自审+commit | **值得看**（作横评证据：无完成步骤、自述证据链） |
| engineering/wayfinder | 超会话规划：地图+决策票+雾+前沿 | **本次主对象**：词汇与失败模式全值得，机制不学 |
| engineering/prototype | 抛弃式原型答一个设计问题，留档为一手来源 | 弱参考（「验证过的决定进主干、探索留分支+指针」与结晶立场同构） |
| engineering/diagnosing-bugs | 硬 bug 诊断循环，红能力回路当门票 | **值得看**：完成判据 checklist 化；专业化 Participant Skill 样板 |
| engineering/research | 后台代理查一手来源出带引文的 markdown | 弱参考（≈HCTL2 Room Invocation 的 research 场景） |
| engineering/tdd | 红绿循环参考件，seam 须与人预先议定 | 弱参考（Skill 层素材） |
| engineering/domain-modeling | 活的词汇表纪律+ADR 三条件 | 弱参考（ADR 三条件一句话可进写作指引） |
| engineering/codebase-design | 深模块词汇与原则 | 不值得（编码方法论本体） |
| engineering/code-review | 双轴平行子代理评审+Fowler 基线 | 弱参考（双轴分离、不合并重排的呈现纪律） |
| engineering/resolving-merge-conflicts | 按意图溯源逐 hunk 解冲突，永不 --abort | 不值得 |
| engineering/wizard | 给「只有人能做的步骤」生成交互向导 | **值得看**：Request human-action 解决动作的交付形状 |
| productivity/grill-me | grilling 的无仓库门 | 间接 |
| productivity/grilling | 轮次+前沿+事实/决定二分的采访原语 | **值得看**：交互协议与演化史 |
| productivity/handoff | 会话→可携带交接文档 | 不值得（HCTL2 接力只经结晶；三条纪律已被 Bundle/指针档覆盖） |
| productivity/teach | 多会话教学工作区 | 不值得 |
| productivity/to-questionnaire | 把答不了的决定变成给他人的异步问卷 | **值得看**：Request 问卷式应答面 |
| productivity/wait-what | 一句「没听懂，重讲」的纠偏词 | 不值得 |
| productivity/writing-for-agents | 给 agent 写文档的写作学 | 弱参考（未来给 Participant 写 Skill 的方法） |
| misc/git-guardrails-claude-code | 钩子拦危险 git 命令 | 不值得（误操作护栏，非治理） |
| misc/setup-pre-commit、migrate-to-shoehorn、scaffold-exercises | 环境脚手架三件 | 不值得 |
| in-progress/implement-spec | 整 spec 单 PR、票当任务图并发施工 | 横评证据（完成外包给 PR merge 机械关单） |
| in-progress/retro | 会话复盘出环境改进建议 | 不值得（自认 STUB；七类清单一瞥即可） |
| in-progress/loop-me | 个人工作流规格审讯 | 弱参考（checkpoint/push right/brief 词汇） |
| in-progress/claude-handoff | 交接文档直接种一个后台代理 | 不值得 |
| in-progress/setup-ts-deep-modules | dependency-cruiser 锁深模块边界 | 不值得 |
| in-progress/writing-beats / -fragments / -shape | 写作三件套（explore/exploit、beat、grounding） | 不值得（与 HCTL2 无关） |
| deprecated/（空） | 退役即删除，changeset 记继任者 | 弱参考（退役纪律） |

## 完成判定权专项

口径对齐 [方法论生态审计](./methodology-landscape-20260824.md) 第三节横评表。HCTL2 立场基准：Task 终结只有两个获准 actor 来源——owner human 的命令请求，或绑定精确契约版本的 Run 正常完成后由归约器机械提交同一个「完成 Task」命令（spec/task.md §写入约束）；模型自述始终只是参考信息（task.md §Kanban 场景）。

### 全流水线逐点核对：「done」由谁判

| 环节 | 「done/ready」的判定实况 | 证据 | 与 HCTL2 立场 |
| --- | --- | --- | --- |
| grilling（讨论完成） | 双门：前沿为空 + **人确认** shared understanding；不确认不得动工 | grilling/SKILL.md:28；0e9a072（确认门 2026-07-03 才补） | 同向（人有终审），但 docs 自认弱模型冲门（docs/productivity/grilling.md「It ran out of questions and started building」） |
| to-spec（规格就绪） | seam 与人核对（:17），但发布与 **`ready-for-agent` 状态由模型自贴**，明写「no need for additional triage」 | to-spec/SKILL.md:17,19 | **违反**：状态判定权在模型。后果实录：AFK 轮询 agent 把整份 spec 当票施工，「the most-reported rough edge」（docs/engineering/to-spec.md） |
| to-tickets（票就绪） | 拆分需人迭代批准（:56）；发布后 **模型自贴 `ready-for-agent`**，「the tickets are agent-grabbable by construction」；「Do NOT close or modify any parent issue」是仅有的一条禁令，仍是 prompt | to-tickets/SKILL.md:56,63,67 | 混合：内容有人批，状态无门 |
| triage（外来件就绪） | 最同向一段：先验证声明（复现 bug/checkout PR 跑测试，:74），**推荐并等待指令**（:72「Wait for direction.」），快速改状态也要人先说（:90）；但 wontfix 的 close 动作由 agent 执行（:82-85），标签写入无机械准入 | triage/SKILL.md:72,74,82-90 | 同向（人指令驱动），执行面无强制 |
| wayfinder（决策票完成） | HITL 票经与人的对话解决，但 **close 动作是 agent 的**（:125「post the answer…**close** the issue」）；AFK research 票由子代理独立解决（:77,115）；地图完成=前沿空+雾清，无人签收对象 | wayfinder/SKILL.md:125,77,115 | **违反**（决定的落锤无准入）。三个实锤：e5932a7 自答事故；Notes 自豁免（docs/engineering/wayfinder.md:69-70「the constraint and its exemption live in the same file the constrained party owns…There is no hard in-skill stop」）；prototype 票自选自关（:75-76） |
| implement（施工票完成） | **技能没有完成步骤**：跑完测试、自审、commit 即止，「It ends at the commit and never touches the work item…Close the ticket and reconcile the criteria yourself」；副作用：无人关单则前沿饿死（「If nothing gets closed, nothing ever becomes visibly unblocked」） | implement/SKILL.md:7-15；docs/engineering/implement.md:51-53 | 同向但靠省略：关单默认留给人，却无任何东西阻止 agent 去关；关单质量无凭证对象 |
| implement-spec（beta，整 spec 完成） | 完成外包给 GitHub：**PR 标记 closing 全部票**，人 merge 时机械关单；merge 前有模型 code-review+修复 | implement-spec/SKILL.md:23,31,33 | 系统记录寄生标准形状（同 CCPM）：人在 merge 门上，票的关闭是平台副作用，无逐票验收 |

### 「TDD 与 code review 是证据还是自述？」——自述

- TDD 的红绿是**过程纪律不是归约**：测试由模型在自己会话里跑、结果以行文自报；没有任何对象把「测试运行」绑定到 commit 或票（HCTL2 的对照：Receipt 逐项绑定 Evidence ref+digest，「不能用一个总括 "tests passed" 替代逐项绑定」，spec/task.md §写入约束）。
- code-review 是**模型评审自己会话刚写的代码**：implement/SKILL.md:13 在 commit 前调用；作者自己在 docs 承认「an agent reviewing the code it just wrote is biased toward its own solution」（docs/engineering/implement.md:67）；更硬的事实是顺序 bug——code-review 只看 `git diff <fixed-point>...HEAD`，不含未提交改动，而 implement 在 commit 前调它，「unless an interim commit already exists there is nothing in that diff to review. Multiple people have reported this and it is unfixed on both sides」（:65）。自述证据甚至可能是**空集**。
- 评审产出无裁决语义：两轴子代理的指令都是「Report…」（code-review/SKILL.md:64,70），聚合端不合并不重排（:76），没有 pass/fail、没有阻断 commit 的门。

### 「ready-for-agent 谁打？」——同一仓库三个答案

to-spec：模型打，明言免triage（:19）。to-tickets：模型打，「unless instructed otherwise」（:63）。triage：maintainer 指令后 agent 执行（:72,90）。三个答案并存，加上标签同时承载「规格完备」与「可领取」两种语义，直接产出了 AFK agent 领走整份 spec 的事故。HCTL2 的对照物是把这三件事拆成三个对象：契约采纳（人显式动作）、操作投影 stage（content，后端拥有）、Run 授权（显式启动命令）——spec/task.md §对象,28 与 task.md §无 Run 的轻量路径。

### 横评表新增一行（并入 methodology-landscape 口径）

| 工具 | 完成判定实况 | 与 HCTL2 立场 |
| --- | --- | --- |
| mattpocock/skills | 讨论完成=人确认门（grilling:28）；规格/票就绪=模型自贴标签（to-spec:19、to-tickets:63）；决策票=人在对话回路但 agent 落锤自关（wayfinder:125）；施工票=技能无完成步骤、关单留人（implement docs:51-53）；测试与评审证据全程模型自跑自报，评审常评空 diff（implement docs:65-67）；唯一硬强制是防误操作 git 钩子，不做治理 | 计划侧同向、施工侧违反、全线 prompt 无机械边界；自带三个「prompt 当强制层失败」实锤（e5932a7、Notes 自豁免、prototype 自关），是「同向碎片散落、无系统边界」总判的最佳单一佐证样本 |

## 对初审六组结论的逐条核验

初审指全量审计之前只读了 grilling、wayfinder、to-spec、to-tickets 四个 SKILL.md 就下的第一轮判断（2026-09-01，Fable 5.1 主会话）。下表把每条初审结论完整写出，再给裁决与双方原文，读者不必回头找初审文本。

| # | 初审结论 | 裁决 | 证据（双方原文） |
| --- | --- | --- | --- |
| 1 | 族归属：「Harness 技能包当方法论」族，杂交「系统记录寄生」（规划状态持久化到 issue tracker）；族的既有定性（技能层给不出证据高于自述、只当注入通道）不因此改变 | **维持** | 技能层：全仓库机制均为 prompt（setup/SKILL.md:15「prompt-driven skill, not a deterministic script」），分发即 harness 插件/skills.sh（CHANGELOG v1.2.0 #536）。寄生：地图=issue、票=子 issue、阻塞=原生依赖边（wayfinder/SKILL.md:21,69）、spec/票直接落 tracker（to-spec:19、to-tickets:63）；implement-spec:23 完成外包给 PR merge。定性不变的实锤：e5932a7 commit message（学员实报 agent 自答 HITL 票，修复手段是措辞改写）；docs/engineering/wayfinder.md:69-70（Notes 自豁免，「There is no hard in-skill stop」）；:75-76（prototype 自选自关，未修）。演化史整体从自造惯例滑向 tracker 原生结构（6f9e995、b289481、5c3c49d），寄生是引力不是点缀 |
| 2 | 缺口一：HCTL2 的 Task 默认产出是变更集/工件，「决策票」无处安放；建议不新增对象：决策票=走无 Run 轻量路径、结晶为答案文档、有权 human 提交完成——在 spec/task.md 写入约束下是否走得通？ | **修正**（结论走得通，前提补两条，映射改三分） | 逐条核对：轻量路径产出「变更集/工件的新版本」（task.md §无 Run 的轻量路径）——答案文档须**登记为 Artifact** 才是工件（spec/project.md §Context、Memo 与 Artifact「普通 Git 文件在登记前不是 Artifact」）；「完成 Task」前须有契约且**不得隐式生成**（spec/task.md §契约与来源「无契约的…『完成 Task』必须先要求该独立动作」「不得在同一命令中隐式生成契约」），可在「创建 Task」时带已预览初始契约一并落（spec/task.md §契约与来源）；测试证据非硬性——验收按契约定义的规则校验（spec/task.md §写入约束），契约不要求评审即不必开 Run（task.md §无 Run 的轻量路径）；完成 actor=owner human（spec/task.md §写入约束）✓。但对照 wayfinder 四型票，「决策票=轻量 Task」只是四分之一答案：grilling 型（纯讨论出决定）在 HCTL2 是 **Scoped Room**（spec/project.md §Room 与消息 冻结讨论目标、完成条件、结论回填动作——正是「decision ticket」的形状）；索决定/授权型是 **Request**（spec/project.md §对象 一级对象）；task 型（开通访问等）是 Request 的 human-action 解决或普通卡片；只有 research/prototype 型（产出答案工件、值得冻结验收）才走轻量 Task+Artifact。「不新增对象」的建议因映射更富而**更稳** |
| 3 | 缺口二：「雾」（范围内、未成形的已知未知）在 HCTL2 无一等位置，只活在聊天记录里——Project/Memo/Context Manifest 真没有格子？ | **维持缺口，修正处置** | 逐格核对确认没有：Context Manifest 的「known gaps」是**本次组装**的来源覆盖缺口（spec/project.md §Context、Memo 与 Artifact），不是业务未知；Memo 是「显式提炼…的长期知识」（spec/project.md §Context、Memo 与 Artifact），雾不是知识；滚动纪要装「讨论过、口头定了、尚未结晶的共识」（context.md §前情提要：房间的滚动上下文），是已定项不是未决项，且非权威（spec/project.md §Context、Memo 与 Artifact）；Request 要求冻结精确问题与目标人（spec/project.md §Request、spec/connections.md §跨模块 Request 回路），而雾按定义「还问不准」（wayfinder/SKILL.md:88 的判据正好把雾挡在 Request 门外）。修正处置：不需要新对象——雾是 Project 的目标/范围正文里一节「尚未定形」清单，用「更新 Project」命令推进 project_version（spec/project.md §Repo 注册与 Project 归档）即可承载；值得借的是**毕业纪律**（雾→票即从雾里清掉、雾单调收缩当健康信号，wayfinder/SKILL.md:126、docs/engineering/wayfinder.md:96），仅参考行为 |
| 4 | 缺口三：「决定不能自述」（HITL 票 agent 不得自问自答）是完成判定权在决策上的镜像，HCTL2 只管住了完成没管住决定 | **推翻** | HCTL2 在每个决定成为类型化对象的地方都已管住：① Request 创建冻结「required actor/role + permission」（spec/connections.md §跨模块 Request 回路），解决命令固定 actor/delegation，且「普通 Room 回复不能解决 Request」（spec/connections.md §跨模块 Request 回路）——模型不能替被点名者作答；② 模型 Participant「不能自行创建 Room Invocation、唤醒 worker 或递归委派」，须用户批准（spec/project.md §场景约束）——不能替人决定开工；③ 「被评对象的作者不占必需评审席位」（run.md §关键规则）——不能自评；④ 纪要「不由房间内模型 Participant 书写或改写」（spec/project.md §Context、Memo 与 Artifact），「『agent 自己决定记什么』是自述」（context.md §前情提要：房间的滚动上下文）——连记什么都不让模型自定；⑤ 讨论结论「仍需由有权 actor 提交原动作」（spec/project.md §Request）。mattpocock 的自答事故之所以致害，是因为在他的世界里讨论产物直接变成关票动作（wayfinder:125）；HCTL2 的架构把这条路封死（普通消息不是入口，spec/project.md §场景约束），模型在 Room 里自问自答只剩说服层危害，且自述已被强制标注（task.md §Kanban 场景）。缺的只是那句话，不缺那道门 |
| 5 | 印证两条：地图=索引 ↔ 投喂三档的内联/指针；阻塞边放 tracker 原生关系 ↔ 依赖边归 Kanban 不归 Workflow | **维持** | 索引：wayfinder/SKILL.md:23「the map…only gists it and links」+ 9272935 ↔ context.md §投喂三档：内联、指针、代取 指针档「精确地址（ref+digest）加一句话说明」、清单是「不可变说明」而非内容本身（context.md §与相邻概念的分工）。阻塞边：wayfinder/SKILL.md:69 原生依赖边+b289481 理由（前沿在 tracker 自家 UI 可见）↔ HCTL2「普通移动和排序归任务后端…以回读为准」（task.md §Kanban 场景）、Blocked 是从 blocker 派生的正交 health（spec/task.md §对象）、引擎只拥有机械位置（run.md §为什么存在）。两组同构均成立 |
| 6 | 不学机制：assignee 当锁、label 当类型、issue 查询当 frontier、token 预算人盯——无平台下的替代品；有无例外值得抄？ | **维持，查无例外** | assignee 当锁（wayfinder:67）vs HCTL2 账本 claim CAS+幂等键（spec/task.md §写入约束）；label 当类型（wayfinder:65）vs 类型化对象与命令；issue 查询当 frontier（issue-tracker-github.md:43）vs 归约派生 readiness；token 预算人盯（wayfinder:57 每票 100K、ask-matt:32 smart zone ~150k 人自己看着办）vs Trigger Preview 显示 token 估算（spec/project.md §场景约束）+ Bundle 冻结选材计量（spec/project.md §Context、Memo 与 Artifact）。逐一复查后最接近例外的两条都不成立为新采纳：「原生阻塞边让前沿在后端 UI 可见」已是 HCTL2 适配协议立场的既有内容（task.md §Kanban 场景 卡片显示阻塞、外部概念对齐表）；「票尺寸=一个新上下文窗口」是好的切分口诀但属产品文案/方法论层，不构成机制。另记一条 tracker 侧硬教训供适配器实现参考：GitHub 依赖边 API 要数据库 id 而非 issue number（issue-tracker-github.md:42），及子 issue/阻塞边在实践中常退化为正文文字（docs/engineering/to-tickets.md:58-61，#554/#513）——外部原生关系的写入可靠性本身需要回读核验，正合 HCTL2「按后端能力写入、以回读为准」 |

## 二审补充：新发现与分界原则

> 本节是落库时补的第二轮判断（2026-09-02，Fable 5.1 主会话），写在上文全量审计之后，回答所有者的两个问题：对他的方案还有什么新发现；这些 skills 应该成为 HCTL 的机制，还是成为预先装载、更专门的 Participant。

### 四处新发现，以及一个两家共有的未解问题

**一、他自己画了一条机制与方法论的分界线，线画在阶段切换上。** 每个 skill 分两种触发方式（`.agents/invocation.md`）：一种只有人敲名字才能触发（grill-with-docs、to-spec、to-tickets、implement、wayfinder、triage），另一种模型可以自取（grilling、tdd、domain-modeling、prototype、research、code-review、wizard）。不变式只有一条：人触发的 skill 可以调用模型可取的 skill，但永远不能调用另一个人触发的 skill。翻成大白话：从一个阶段跨到下一个阶段，只有人能按；阶段内部怎么干，模型可以自己挑方法。这正是治理与方法的分界，他用 frontmatter 里的一个开关做出来了，因为他手里只有这个。他还给出了判断某个 skill 该不该让模型自取的尺子：「模型能不能有意义地自己伸手拿它」，而不是「它是否可复用」。这句话对 HCTL2 设计 Skill 的装载规则直接有用。

**二、三次失守是同一个形状：约束和它的豁免住在同一份文件里，而这份文件由被约束的一方持有。** agent 自问自答、agent 往自己拥有的地图 Notes 里写「本图允许执行」再读回来当许可、prototype 票三选一后自选自关，三件事的共同点都在这里。他的文档把话说到「skill 里没有硬停止」就止住了，没有往上抽成原则。抽出来就是一条可以进写作指南「作恶四问」的检查：约束和豁免是不是住在被约束方能写的地方？HCTL2 目前过得了这一关，因为 Context Manifest 由 control 冻结、Skill 由 control 冻结进派发规格、Project 范围只能经人的「更新 Project」命令改；但这条检查值得写下来，它比「哪个角色能绕过」更具体。

**三、他已经替我们把问题的两端都站过了。** README 明说：GSD、BMAD、Spec-Kit 这类工具「靠拥有流程来帮忙，但拥有流程就拿走了你的控制权」，所以他选了另一个极端：什么流程都不拥有，skill 小而可改，任何人可以 fork。代价就是上面三次失守，因为不拥有流程等于没有任何东西能强制。「机制会不会把方法论固化」这个担忧，他的整套东西就是对照实验：完全不固化，结果是完全不设防。

**四、人的注意力在他那套里是没有预算的资源。** 用户最尖锐的活抱怨是「grilling 让人筋疲力尽，每个问题三段长」，而且长问题把「为什么问这个」挤掉了，读者跟不上决策链（docs/engineering/wayfinder.md「The grilling is exhausting」）。HCTL2 的预算体系管的全是模型侧：token、写入范围、成本。人这一侧没有任何计量。这不是马上要改的东西，但值得记下：当塑形工作的瓶颈变成人回答问题的耐心时，Request 一次问一个，和批量问卷一轮问完一批、每题带推荐答案，这两种形状的差别会很大。

**两家共有的未解问题：一个已经关掉的决定后来被推翻，下游哪些东西建在它上面？** 他的文档承认「没有官方指引，agent 的本能是绕着错误决定设计而不是挑战它」。HCTL2 这边，下游对象冻结旧的 Project 版本，提交前发生分歧由比较并交换拒绝，提交后「由冻结约束继续执行到终态，新的顶层授权使用新版本」（spec/connections.md §版本、权限与替代）。也就是说，冻结在旧版本上的 Task 会照旧版本走完，条款没有要求把它标成「需要关注」。这条记为问题，不记为缺口：它可能就是正确的设计，只是尚未写明理由。

### 分界原则：方法归 Skill，产出归对象，管不住的归机制

判断一样东西该归哪层，问一个问题就够：**换一种方法论，这个东西还需不需要？**

- **换了方法论就不需要的，归 Skill，装进 Participant。** 一轮把前沿上的问题问完、每题附推荐答案、一个 session 只解一票、票要切成穿透全栈的竖片、ADR 只在难逆转且有真取舍时才写。这些都是「怎么干活」，BMAD 有另一套，spec-kit 又有另一套。HCTL2 托管它们，但不拥有。这正好落在既有设计上：participant.md §专业化 Participant 已经写了「方法论是 Skill，不是临时提示习惯」，评审是第一个专业化的岗位；塑形与规划就是第二、第三个岗位。一个塑形 Participant 装载一份 grilling 类 Skill，七件事分层一个字不用改。
- **换了方法论仍然需要的，归对象，而且要用方法论中立的名字。** 每种方法论最后都会产出同样几类东西：一个有人拍板、别人可以依赖的决定；一个必须由指定的人回答的问题；一个明确划出去的范围边界；一个在范围内但还问不准的已知未知；一份冻结了验收标准的工作项。HCTL2 已经有 Request、Scoped Room、Task Revision、Memo、Artifact 和 Project 的目标与范围。真正缺的只有「还问不准的已知未知」这一格（见核验表第 3 行）。补法是在 Project 范围正文里加一节，用「更新 Project」推进版本，不加新对象。「前沿」不做成对象：他自己的文档承认「前沿是 agent 的判断，不是计算出来的图」，那它就不配当机制。
- **方法论自己管不住自己的地方，归机制。** 必须由人回答的问题不能被模型代答；完成不能自述；规划空间不能给自己签发执行权；被约束方不能写自己的豁免。他的三次失守证明这一层放在 prompt 里必然失守，所以它只能是账本里的门。HCTL2 在这一层已经齐了（见核验表第 4 行）。

**对「固化」的正面回答：HCTL2 固化的是「什么算作数」，不是「该怎么干活」。** 无契约的 Task 只有身份映射、不进治理；无 Run 的轻量路径明说「必须比直接开终端干完更轻」。不受治理的路一直是开着的，谁想用什么方法都行。真正关上的只有一扇门：不给证据就说自己做完了。类比 Git：它规定什么是 commit、谁能 push、合并要满足什么，但从不规定你用主干开发还是 gitflow。类比的失效边界：Git 对内容完全不管，而 HCTL2 多管了一样东西，就是验收约束的形状；所以 HCTL2 比 Git 固化得多一点，但固化的仍然是产出物的边界，不是过程。

**整合的核心一句话：他的 Skill 内容，加上我们的门，正好是他造不出来的那个东西。** 落到 wayfinder 与 grill 上：

| 他那边 | HCTL2 这边 | 归哪层 |
| --- | --- | --- |
| grilling 轮次协议、四型票、竖片切法 | 塑形 Participant 装载的 Skill；它的输出只能是类型化的建议（建议创建 Request、建议开一间 Scoped Room、建议把某片雾毕业成 Task），人经 Trigger Preview 批准后才变成动作 | Skill |
| HITL 票不能自答 | Request 冻结应答人、普通消息不是入口。同一条规则，只是强制方从 prompt 换成了账本 | 机制（已有） |
| 地图 | Project Room 本身：Decisions so far 是已回填的 Scoped Room 结论，开放的票是开放的 Request 与 Scoped Room，雾是 Project 范围正文里的那一节 | 对象（已有，只缺雾这一格） |
| 前沿 | 留在 Skill 里，由塑形 Participant 判断并提建议 | Skill |
| 调度（一 session 一票、手工分派） | 人在 Project Room 里挑下一个要解决的问题。不为规划开 Run：Run 的价值是持久重试、候选切换与评审关卡，一场对话三样都用不上 | 人 |

## 借鉴决策（五种复用决策用语，逐条）

- **采用为依赖：零。** 无服务、无 CLI、无运行时；prompt 文本无可依赖之物。
- **适配协议：零。** 无 schema、无线协议。唯一接近协议的东西（triage-labels 的角色→标签映射表、issue-tracker 文档的操作词表）过于朴素，HCTL2 的 Task Binding 逐字段写入权（spec/task.md §契约与来源）已强于它。
- **移植有边界的组件：零。** 仓库里仅有的代码（wizard template.sh、hitl-loop.template.sh、link-skills.sh、git 钩子脚本）与 HCTL2 无对接面。
- **仅参考行为（全部借鉴都落在这一档）：**
  1. frontier/fog 词对与「问准判据」「毕业纪律」（wayfinder/SKILL.md:8,69,84-93,126）→ Project 叙事、看板 readiness 文案、Project 正文「尚未定形」节的操作规则；
  2. grilling 轮次协议：整前沿一轮、依赖分轮、每题带推荐、事实归 agent 决定归人、空前沿+人确认（grilling/SKILL.md:8-28）→ Trigger Preview 前的塑形交互与 Request 应答面；连同其演化教训（一次一题→轮次，且轮次制仍受争议、保留退路）；
  3. AGENT-BRIEF 写票纪律：耐久优先于精确、写行为约束不写文件路径行号、验收项独立可验（AGENT-BRIEF.md:9-31）→ Task Revision 写作指引（与 spec/task.md §契约与来源「冻结验收约束，不冻结施工步骤」互为印证）；
  4. to-questionnaire 的「审讯投递不审讯议题」+ 问卷模板（最重要在前、一题一意、答案桩、why-this-matters、允许「不知道」）（to-questionnaire/SKILL.md:9-40）→ Request 默认应答面升级为 Scoped Room 之前的中间形态；
  5. wizard 的 human-action 纪律：只做人才能做的、不可逆先确认、agent 不代跑（wizard/SKILL.md:3,37,43）→ Request 的 human-action 类解决动作交付形状；
  6. .out-of-scope/ 概念级拒绝先例库与查重复述话术（OUT-OF-SCOPE.md:17,76-82；「已实现不入库，防止污染查重」:84-88）→ 产品侧 wontfix 治理与 HCTL2 自身 decision-history 的操作细则；
  7. diagnosing-bugs 的「完成判据 checklist 化」（红能力/确定性/快/可独跑，diagnosing-bugs/SKILL.md:57-66）与 tdd/code-review/domain-modeling → 专业化 Participant 的方法论 Skill 素材库（participant.md §专业化 Participant：告别 BYOA 的「评审是第一个必须专业化的岗位」正需要这类内容）；writing-for-agents 是给这些 Skill 写作时的方法参考；
  8. 反面教材两条入设计文档否决理由：`ready-for-agent` 单标签双语义事故（docs/engineering/to-spec.md）；五状态机缺 blocked/deferred/terminal 引发的用户抱怨与 AFK 重复领取（docs/engineering/triage.md:76-77）。另一条转给适配器实现：外部原生关系写入常退化，须回读核验（#554/#513）。
- **暂缓：** 无。本对象没有「等成熟再看」的部件——prompt 文本的价值即时兑现，机制性的部分已裁定不学。

**落地建议**（待所有者裁决；分三步，前两步不加新对象）：

1. **只改文字。** 本文与方法论生态审计的补记入库。设计正文四处小改：participant.md 专业化一节把塑形与规划列为评审之后的岗位，并写明「Skill 给方法，账本给门」；project.md 范围正文加「尚未定形」一节与毕业规则（问准了就开 Request 或 Scoped Room，这一节只准越来越短）；task.md 轻量路径明说产出的工件可以是一份答案文档，并带上「须登记为 Artifact、契约随创建 Task 一并携带」两个前置；写作指南的作恶四问加一问「约束与豁免是否住在被约束方能写的地方」。另建议 vision.md 加一句愿景层的表述：HCTL2 拥有工作的边界，不拥有工作的方法。这个立场目前只写在研究层。
2. **第一份塑形 Skill。** 把 grilling 改编为 HCTL2 用的 Skill（MIT 许可允许），改编的重点是把「关票、贴标签」全部换成「提出建议」。Request 的应答面在卡片与 Scoped Room 之间加问卷形态：一个 Request 冻结一组子问题，每题带推荐答案，一次类型化动作回答全部。先在 HCTL2 自己的设计讨论上试，`.memo/README.md` 的待拍板表本来就是一张雾清单。
3. **两条待裁。** 其一，Skill 内容的家在哪里：context.md 把 Skill 与仓库代码、Memo 并列为执行体在 worktree 里自己取用的材料，这读作仓库侧；participant.md「方法论出新版，所有项目同步受益」与远程数字员工的表述又读作参与者侧，随参与者跨仓库走。两种读法现有文本都有支撑，第一份塑形 Skill 放哪里取决于这次裁决。其二，决定被推翻后的下游提示：spec/connections.md 已定「提交后由冻结约束继续执行到终态」，是否要给冻结在旧版本上的 Task 加「需要关注」，需要一句明确的话，哪怕结论是不加。

## 遗漏补充（初审四文件之外的收获与明确不值得）

1. **docs/ 长文是比 SKILL.md 更硬的证据层。** 每技能的人读页带「Common questions / It's working if」，逐条记录真实事故（Notes 自豁免、AFK 领走 spec、并行 implement 的 git 灾难与 refs/stash 跨 worktree 共享、code-review 评空 diff、triage 批量跑退化到便宜列表证据）。初审没读它们，等于漏掉了该仓库的「事故台账」。本报告的完成判定权与反面教材证据大半来自这里。
2. **演化史里的三次「被咬」**：一次一题→轮次（a6bdfd9→a4b2009，被采访节奏咬）；自答事故→事实/决定二分+HITL 标签（e5932a7，被 prompt 复用语境咬——写给「人在场」的句子换个框架就成了自答许可证）；自造惯例→tracker 原生结构（6f9e995/b289481/5c3c49d，被状态腐坏咬）。第三条对 HCTL2 是正向印证：操作状态就该住在系统记录的原生字段里，HCTL2 把它做成了带写入权与回读的适配器约束。
3. **明确不值得的裁决**（逐个点名，不留「值得进一步研究」）：handoff/claude-handoff/PHASE-BOUNDARIES——HCTL2 不代管会话内上下文（context.md §为什么存在,30），席位间接力只经结晶（context.md §同一 Run 内的接力：节点之间传什么），交接文档是无治理层时的手工替代，其三条纪律（不复制已落盘工件、去敏、指针化）已被 Context Bundle/指针档覆盖；triage 五状态机——不移植，缺的三个状态 HCTL2 以正交 health 与 Receipt 原生解决；retro——自认 STUB（in-progress/README.md），其「评审代理无上下文压力、应负责标准」的洞察与 HCTL2 评审席位设计弱同构，一行带过；implement-spec——frontier 并发编排的 prompt 雏形（worktree 隔离、merger 子代理、PR 机械关单），HCTL2 的 Run/Seat/Attempt/Gate 是它的治理化完全体，只作横评证据；teach/writing 三件套/wait-what/misc 四件——与 HCTL2 问题域不相交。
4. **仓库自我管理的两个小纪律可顺手参考**（仅参考行为）：CLAUDE.md:21 的「router lies」规则——技能增删改必须重核路由文档，否则路由器在说谎（对 HCTL2 的文档索引维护同构）；deprecated 桶的「退役即删除、changeset 记继任者」（skills/deprecated/README.md）。
5. **一处与直觉相反的确认**：该仓库**没有**任何「agent 自动领票循环」——领取、并行、调度全是人的动作（docs/engineering/to-tickets.md:68「Dispatch is manual」；wayfinder:120 用户选择并行）。初审把它当成「AFK 流水线」的印象需要校正：它是**人当调度器**的会话多路复用姿态，AFK 只出现在单票内部（research 子代理、implement 单票施工）。这使它的完成判定权画像比 Taskmaster/MetaGPT 类「模型自把 done」的工具好看一档——但好看的原因是把裁决外包给了人的自觉，而不是收进了系统边界。

## 复核记录

- 2026-09-01 首发。仓库元数据经 GitHub API 独立复核（243,499★ / created 2026-02-03 / pushed 2026-08-24 / MIT / HEAD 6654f6b）；全部源码引用手工核对行号。
- 2026-09-02 落库。HCTL2 条款引用由行号改为节名，并在 main @ `aaf9fd4`（草案 v0.15.6）逐条复核引文仍成立；初审措辞改为可独立阅读；补「二审补充」一节与落地建议。
- 2026-09-02 所有者裁决（同日，草案 v0.16.0）：落地建议第 3 条的两个待裁已裁。Skill 的内容归 Agency（参与者的供给方）安装并申报，HCTL 只记引用、指纹与可核验性；Participant 立为第四个模块，Agency 定为 provider 而非工厂；人不是 Participant。设计正文的对应改动见来时路第 34 章。

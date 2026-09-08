# 模型协作为什么需要 DAG：Prove2Me 与费马大定理形式化的复原与借鉴

> 类别：跨候选归纳 · 证据编号：E-PROVE2ME-DAG<br>
> 状态：调研 · 日期：2026-09-08；发布后正文不改，只在文末追加复核记录<br>
> 总览与复用决策用语见 [docs/research/README.md](./README.md)。

## 问题

所有者听说的版本：最近有 AI 验证费马大定理（FLT），一开始各个模型无法协作，后来用了一个叫 prove2me 的东西才成功。要查清：这是什么项目、谁做的、什么时候、什么形式系统、prove2me 是什么、依赖怎么表示、多个模型怎么分工、之前为什么失败、之后为什么成功。然后回答三个命题：① 这证明了 DAG 重要；② 证明了「要动态地把依赖关系用机械方式存下来，而不是写散文」；③ 学习 prove2me，对我们「从 Task / Room 的讨论里凝结出 DAG 施工图」有什么可借鉴。顺带比较业界同题工作里「图由代码持有并可机械检查」与「图写在提示词或散文里」的结果差异。

先说与所有者听说的版本有出入的两处，细节在正文：

- 不是「各个模型」协作失败。参与的是 Anthropic 同一个内部模型的几十个并行 agent，不涉及多家厂商的模型。
- 不是「中途换上 prove2me 才成功」。成功的那次运行从第一天就跑在 Prove2Me 之上；此前失败的若干次尝试用的是什么基础设施，Anthropic 没有公开。二手报道里「Prove2Me 中途加入」的说法在一手材料里找不到依据。

## 事件复原：AI 与费马大定理

### 一句话

2026 年 8 月 7 日至 17 日，Anthropic 一个小团队让几十个 Claude agent 在 Columbia 大学的 Prove2Me 平台上并行工作，11 天写出费马大定理在 Lean 4 中的完整形式化证明：29,511 条定理、约 1,300 万行 Lean、只依赖 Lean 三条标准公理、没有 sorry；9 月 4 日公开博客、技术报告与仓库。这是把 Wiles–Taylor–Wiles 的**已知证明**翻译成机器可检的形式，不是新数学。

### 时间线

| 时间 | 事件 | 来源 |
| --- | --- | --- |
| 2024-10 → 2029-09 | Kevin Buzzard 领导的 Imperial College FLT 项目（EPSRC 资助）：人写 Lean、走「21 世纪路线」、用 leanblueprint 管依赖图；第一阶段目标是把 FLT 归约到 1980 年代末已知的深结果，不是完整证明 | E9、E10 |
| 2026 春 | Columbia 课程「Machine-Assisted Mathematics」；Prove2Me 由课堂上的证明验证游戏与博士生 Shuze Chen 的 agent 协作研究合并而来 | E5 |
| 2026-06 中 → 07 末 | Prove2Me 四个案例 mission 完成：17K–151K 行，3–9 个 agent，7–16 天 | E4 |
| 2026-07-03 | prove2me_workspace 仓库创建（agent 技能与工作区） | E6 |
| 2026-08 初（日期未公开） | Anthropic 若干次初始尝试失败：「agent 起初有些进展，但很快弄丢了项目状态、不再有效协作」；这些失败尝试的产出占最终证明非样板行的约 7% | E1 |
| 2026-08-07（Day 1） | 基于 Prove2Me 的新 harness 上运行开始；FLT 初等陈述那张卡放到树顶 | E2 |
| 08-08（Day 2） | 约 2,100 条定理证毕；Mazur 定理 19 阶挠点情形、3–5 切换证毕 | E2 |
| 08-11（Day 5） | Mazur 定理所需情形证毕（Frey 曲线 mod p 表示不可约）；当天上午 agent 的「共享计划」还估「1–3 周」，晚上 9:40 完成 | E2 |
| 08-13（Day 7） | 依赖树一次「重接线」：进度图上的凹陷不是丢工作，是换了一段子树 | E2 |
| 08-17（Day 11）22:00:57 ET | 根卡在平台上标 Proved，open_leaves 为 0；agent 自己给同事的措辞是「proved on prove2me，pending the independent re-check」 | E2 |
| 08-18 → 19 | 全部 29,511 张卡离站重编译；整棵树作为单一 Lean 项目构建，`#print axioms` 只剩三条标准公理；comparator 与 nanoda 两个第三方检查器通过 | E2、E3 |
| 2026-08-28 / 31 | Prove2Me 论文 arXiv v1 / v2 | E4 |
| 2026-09-03 → 04 | 仓库单提交 `aa2d8b34`（Lean 4.33.1 / Mathlib v4.33.0 移植版）；09-04 博客、报告、Buzzard 博文同日发出 | E1、E2、E3、E8 |

### 参与者与形式系统

- **做的人**：Anthropic 小团队，负责人 Tianyi Peng（Anthropic 研究员，同时在 Columbia 带 Prove2Me 组）。人不写数学、不写 Lean（除了目标定理那一行陈述），只偶尔给优先级提示，博客里的原话是「Jacobian as a scheme 听起来优先级高」「把 Mazur 早点推完」。
- **模型**：Anthropic 内部研究模型，「大致相当于 Claude Fable 5.1」；约 60 亿输出 token；「几十个」并行 agent，确切数未公布。
- **形式系统**：Lean 4。运行时是 4.30.0，发布版移植到 4.33.1（Mathlib v4.33.0 @ `db584cd6`），移植改了 26% 的证明文件但定理、公理与论证结构不变。站在 Mathlib、Imperial FLT、flt-regular 三个 Apache-2.0 项目之上，106 个文件带署名改编。
- **路线**：经典 1990 年代路线，按 Darmon–Diamond–Taylor 1995 的讲法：Frey 曲线 → Mazur 不可约 → Langlands–Tunnell 与 3–5 切换 → 所需情形的 R = T → Ribet 降级到 level 2。深定理都只证到论证需要的受限形态，仓库自己声明这些「不应被引为一般经典定理的形式化」。

### 失败与成功的证据

- **失败**：一手材料只有博客的一句话（见时间线）和 7% 这个数字，没有说失败尝试用了什么基础设施、几次、多久。官方给的失败原因是「弄丢项目状态、不再有效协作」。
- **成功**：报告把 Prove2Me 的贡献写成三条：(a) 维护一个定理陈述的 DAG，agent 用它决定下一步证什么，「对缓解记忆退化、让多个 agent 并行特别有帮助」；(b) 陈述与证明分文件、链接单独维护，加速 Lean 编译、降低资源占用；(c) 每条陈述带自然语言描述，可搜索可复用，「证明路径更简单」。
- **平台的 Proved 不是终检**：平台逐卡编译，每张卡只对着子卡的陈述检查。终检是整棵树离站重编译、装进单一 Lean 环境、依赖图「无环且落地」、comparator 核对所证陈述与 Mathlib 自己的 FLT 陈述一致、nanoda 独立内核复放。
- **代价**（报告里 Claude 自己的评估）：五分之二的定理陈述在别的证明文件里逐字重复；一条基础引理在 300 多个文件里重新声明；31% 的字节是关掉实例与 simp 引理的生成前言；96 核 512 GiB 上干净构建 5 小时 52 分。Buzzard 的说法：编译时间约为 Mathlib 的 20 倍；「数学上这项工作基本没告诉我们任何东西」，但作为自动形式化的能力信号令他兴奋，他的项目照常继续。

## prove2me 是什么

**形态**。不是一篇论文里的算法，也不是一个库。它是三样东西合在一起：一个公开网站与 REST API（`prove2.me/api/v1`，托管定理库与验证服务）；一份 agent 技能与工作区（`prove2me_workspace`：`SKILL.md`、`references/`、`Definitions/` `Theorems/` `Solutions/` 三个与服务端镜像的目录）；一篇论文（arXiv 2608.28433）。作者 Shuze Chen、Kunal Marwaha、Xiaoyang Lu、Henry Yuen、Tianyi Peng，分属 Columbia 商学院与计算机系、UChicago、Purdue。技能、工作区与公开定理开放；服务端代码是否开源本次未核实；workspace 仓库没有声明许可证。

**对象模型**，六个词：

- **Theorem**：一条 Lean 4 陈述，正文以 `sorry` 结尾，带自然语言描述，钉在一个 Lean / Mathlib 版本环境；**接受后不可变**。
- **Solution**：对某条 Theorem 的一次提交。顶层定理必须叫 `solution`，类型与目标的 `formal_statement` 逐字一致，自身不含 sorry，不得 import 自己。
- **Proof-sketch（归约）**：一种特殊 Solution——它 `import Theorems.Thm_<子引理>`，用子引理证父定理。子引理可以尚未证明，在服务端只是 sorry 桩。
- **Definition**：可复用定义，同样不可变。
- **Mission**：一个形式化项目（一篇论文、一本书），有 captain。
- **Milestone**：captain 从原文逐字抄下的里程碑陈述，链到一条「canonical」Theorem，给所有 solver 一条公认的攻击路径，防止各自重述同一命题。

**依赖怎么表示**。边不是元数据，是 Lean 的 `import` 行。服务端解析归约提交里的 import，把每个被 import 的子定理登记为父的孩子；子定理必须已存在，不存在要先 `POST /submit-problem` 创建，否则判 `unknown import`。父定理在所有孩子 Proved 时**自动**变 Proved——论文的 Property 1：「若所有被 import 的子引理都被验证，则该定理被验证」。因此图有三个机械性质：边是被编译器检查过的引用；节点状态由图归约而非人标；`GET /theorems/{id}/graph` 与 `/open-leaves` 可随时算出整棵分解树与「当前可攻击的叶子」。FLT 运行收尾时 agent 就是靠 `open_leaves` 从几千降到 0 判断的，不是靠记忆。

**分工方式**。没有锁、没有领取。一条定理允许多个证明；防重复靠三件事——milestone 给权威陈述、Formalpedia 搜索让 agent 先找再写、mission 讨论区贴进度。任何 solver 可以把难题归约成子题（新增节点），任何 solver 可以证任何 Open 叶子。captain 只审 mission 核心（目标、定义、里程碑），中间引理不审。审核用「盲读回」：一个没看过原文的 auditor 子 agent 把 Lean 陈述翻回自然语言，人对照原文。

**机械检查点**。Lean 内核编译；sorry 与非白名单公理拒收，附机器可读原因；类型逐字匹配；禁自引；环境钉版；不可变——改错等于废弃后重传，captain 再把里程碑链到新陈述。**不机械的地方**有两处：陈述是否忠实于原文，这是人的活（Lean Atlas 把这类问题叫「语义幻觉」：通过类型检查却没说对话）；叶子粒度，文档只给经验规则——不要做平凡归约，也不要拆成一堆不可复用的琐碎引理，分解要贴证明的自然结构。

**图是动态的，节点不是**。图只增节点、增边、废弃节点，不改节点。Day 7 的「重接线」是把树的一段换成另一组节点，旧节点留在平台上——这也是平台总数约 30,300 与最终树 29,511 有差的原因。

## 三个命题的证据

### ① 「这证明了 DAG 重要」——支持，但证据是一个团队的前后对照，不是对照实验

支持的部分：

- Anthropic 自己把成败归因写得很直白：失败原因是「弄丢项目状态、不再有效协作」；成功的第一条贡献就是 DAG，作用是「缓解记忆退化、让多个 agent 并行」。7% 是唯一的定量前后数字。
- 图在运行中确实承担了协调：agent 用 open_leaves 选题、用父子归约判完成、在动手前由别的 agent 审陈述；三个 agent 在 25 秒内各自读到 N1 变 Proved——状态在图上，不在任何一个 agent 的上下文里。
- 旁证方向一致：ProofFlow 把证明按步骤拆成引理 DAG，ProofScore 0.545，对整体翻译 0.123、逐步独立翻译 0.072；CodePlan 用依赖分析生成编辑计划，6 个仓库通过 5 个，同样上下文但无计划的基线一个也没通过；LLMCompiler 的 DAG 任务图对 ReAct 延迟 3.7 倍、成本 6.7 倍、准确率约提高 9 个点；TDP 的子目标 DAG 最多省 82% token。

要打折的部分：

- 没有消融。Prove2Me harness 同时带来编译加速与自然语言搜索，博客没有把三者的贡献分开。
- 「DAG」在这里承担的首要功能是**共享的、外置的、可机械归约的项目状态**；无环只是最后才作为终检条件出现（「acyclic and grounded」）。更准确的结论是：外置且机械归约的依赖图重要；「有向无环」是这种图的必要形状，不是它的全部。
- Meta FAIR 的 Automatic Textbook Formalization 用 3 万个 Claude 4.5 Opus agent 在共享 git 代码库上一周形式化一本 500 页教材（13 万行、5,900 条声明、约 10 万美元），说明「共享代码库 + 版本控制 + blueprint」也能把大规模并行跑通。Prove2Me 论文用自己 151K 行、6 个 agent、约 400 美元的案例与之对比，但两边任务、模型、时间都不同，不是受控比较。
- 报告自陈的重复率说明图解决了协调，没解决复用：每张卡孤立证明，五分之二的陈述重复。DAG 让并行成立，但没有共享词表的图会把重复做成合法。

### ② 「要动态地把依赖关系用机械方式存下来，而不是写散文」——支持，且这次事件里恰好有散文与图并存的对照

- 图是机械的：边等于编译器检查过的 import；父状态等于子状态的归约；前沿等于一次查询。这三件事都不是任何 agent「记住」的。
- 散文同时存在并且不可靠：agent 维护着一份带工期估计的「共享计划」——Day 5 上午还写「1–3 周」，当晚就证完；Day 4 一条引理被另一个 agent 标「days」，实际依赖 Ribet 降级，是「months」级。估计错了不影响图，因为图不存估计。
- 平台自己也划出了机械存储的边界：Proved 标记只保证「对着子卡陈述可编译」，整棵树要离站再检查一遍无环与落地。「机械存下来」必须配「机械地整体校验」，逐节点的局部检查不等于全局成立。
- 业界工具的演化方向一致：leanblueprint 把 `\uses{}` 写在 LaTeX 里，`checkdecls` 只查声明名存在，边本身是人写的注解；LeanArchitect 改成从 Lean 代码里**推断**依赖并生成 blueprint，作者报告它「暴露了现有 blueprint 里潜藏的不一致」——这是「注解里的图」与「代码里的图」会分叉的直接证据；Lean Atlas 从项目依赖图出发，把需要人做语义审查的节点集合缩小 94–99%（证明密集型项目）。
- 「动态」要说准：Prove2Me 的节点不可变，动态的是增节点、增边、废弃。这与 HCTL2 的 Task Revision、Workflow Revision 不可变、改动走新版本一致，不冲突。
- 反面：机械存下来的只是引用关系，陈述本身对不对仍是人的判断（mission 核心盲读回）。没有这层，图会忠实地把错误陈述的依赖也存好。

### ③ 「学习 prove2me 对我们从讨论里凝结 DAG 施工图有借鉴」——部分支持

成立的部分：Prove2Me 的 captain 流程就是一条「讨论 → 图」的管线——captain（一个 agent）从原文抄目标、定义、里程碑，按依赖排序成提案 → auditor 盲读回 → 人对照拍板 → 上线 → solver 用归约往下长 → captain 事后只做重新链接与废弃。角色与门的位置都与「塑形 Participant 出建议、人经 Trigger Preview 批准、施工图冻结后开工」同构。

不能直接搬的部分：

- 节点的验收器不同。Prove2Me 每个节点自带内核可判的验收（类型匹配 + 无 sorry）；HCTL2 的义务大多没有这样的内核，验收靠 Gate 席位裁决与证据分级。可借的不是「用 Lean 判」，而是「每个节点进图前必须说清自己的机械判据，说不清的不许进图」。
- 图的生长时机不同。Prove2Me 允许 solver 在运行中把节点归约成新节点；HCTL2 施工图开工前冻结。这是一个真实的取舍，下节单列。
- 人审的范围不同。Prove2Me 只审 mission 核心；我们的 Gate 审的是结果。这次事件里最有价值的一个行为是**动手前由另一个 agent 审陈述**，「早早抓出好几条假陈述」——审的是义务定义，不是交付物。

## 同题的其他案例

| 案例 | 时间 | 图由谁持有 | 机械检查点 | 结果与数字 | 钉 |
| --- | --- | --- | --- | --- | --- |
| Anthropic FLT on Prove2Me | 2026-08 | 平台数据库；边即 Lean import | 逐卡内核编译；终检整树无环落地 + comparator + nanoda | 29,511 定理、11 天、60 亿 token；此前尝试失败、产出留存 7% | anthropics/fermats-last-theorem @ `aa2d8b34` |
| Prove2Me 四个案例 mission | 2026-06 → 07 | 同上 | 同上 | 17K–151K 行，3–9 agent，7–16 天，每个 mission 约 200–600 美元（按消费级订阅估） | arXiv 2608.28433 v2 |
| Meta FAIR · Automatic Textbook Formalization（RepoProver） | 2026-04 | 共享 git 代码库 + blueprint 站点 | Lean 编译；协调细节本次只读摘要 | 3 万个 Claude 4.5 Opus agent、13 万行、5,900 声明、1 周、约 10 万美元 | facebookresearch/repoprover @ `386adba3`；arXiv 2604.03071 |
| Math Inc · Gauss / Strong PNT | 2025-09 | 人给自然语言脚手架 + blueprint 站点 | Lean 编译 | 约 2.5 万行、1.1K 定理与定义、3 周；人类此前 18 个月部分进展 | math-inc/strongpnt @ `2f5835c3` |
| Imperial FLT（Buzzard） | 2024-10 → | leanblueprint：LaTeX 里 `\uses` 声明边 | `checkdecls` 只查声明存在；PR 人审 | 86 页 blueprint；目标是归约到 1980 年代结果 | ImperialCollegeLondon/FLT @ `789312cb` |
| Equational Theories（Tao 等） | 2024-09 → 2025-12 | 蕴含图本身是数据：4,694 条律、22,028,942 条边 | Lean 验证每条边；传递闭包机械算；仪表盘 | 形式化完成度 99.9988%；ATP 承担绝大多数，AI 作用小 | teorth/equational_theories @ `5c20aeba`；arXiv 2512.07087 |
| leanblueprint | 2023 → | LaTeX 注解 | 声明名存在性 | PFR、FLT、sphere eversion、Carleson 在用 | PatrickMassot/leanblueprint v0.0.20 @ `56e066d3` |
| LeanArchitect | ITP 2026 | 从 Lean 代码推断依赖 | 与 Lean 同步，无双份 | 转换多个既有 blueprint 项目，暴露潜藏不一致 | LIPIcs.ITP.2026.25 |
| Lean Atlas / Lean Compass | 2026-03 | 项目依赖图可视化 | 从目标定理反推需人审语义的节点 | 证明密集项目缩减 94–99%，FLT 59.8% | NyxFoundation/lean-atlas @ `3a81e194` |
| ProofFlow | 2025-10 | 证明步骤 DAG，每步一引理 | Lean 编译 + ProofScore | 0.545 对整体 0.123 对逐步 0.072（184 题） | arXiv 2510.15981 |
| DeepSeek-Prover-V2 | 2025-04 | 单模型：sorry 占位的子目标链，递归求解 | Lean | miniF2F 88.9%；PutnamBench 49/658 | arXiv 2504.21801 |
| Seed-Prover | 2025-07 | 每题一个引理池，存依赖关系 | Lean | IMO 2025 银牌分 | arXiv 2507.23726 |
| Aristotle（Harmonic） | 2025-10 | 引理式分解 + 证明状态上的 Monte Carlo 图搜索 | Lean | IMO 2025 六题过五 | arXiv 2510.01346 |
| LLMCompiler | ICML 2024 | 规划器输出工具调用 DAG | 依赖满足即调度 | 对 ReAct：延迟 3.7 倍、成本 6.7 倍、准确率约 +9 | arXiv 2312.04511 |
| TDP（Task-Decoupled Planning） | 2026-01 | 子目标 DAG，作用域上下文 | 重规划限制在当前子任务 | token 最多省 82% | arXiv 2601.07577 |
| CodePlan（Microsoft） | 2023-09 | 增量依赖分析 + 变更影响分析生成编辑计划 | 构建通过 | 6 仓库过 5；同上下文无计划基线 0 | arXiv 2309.12499 |
| beads / Taskmaster（我们已审） | 2025 → | JSONL / JSON 任务图，带类型边与 readiness 归约 | 无：任何 actor 可标完成 | 图机械、完成自述——反例 | [方法论生态审计](./methodology-landscape-20260824.md) §一、§三 |

「图由代码持有并可机械检查」对「图在提示词或散文里」：能拿到的数字都是**有图对无图或线性计划**的比较（ProofFlow、CodePlan、LLMCompiler、TDP），没有一篇把「同一张图写成散文喂模型」当基线。最接近「散文对图」的证据是 FLT 事件本身——失败尝试对 Prove2Me 运行、散文计划的估计对图的状态——与 LeanArchitect 的「注解里的图与代码里的图会分叉」。定性结论一致，定量对照缺位，这一点要如实写。

## 对 HCTL2 的借鉴

复用决策：**仅参考行为**。Prove2Me 是数学形式化平台，不作为二进制、SDK 或组件进入 HCTL2；它的 `graph`、`open-leaves`、归约判决（`SKETCH_ACCEPTED` / `ACCEPTED` / `FAILED`）三处接口形状，可在施工图 schema 定形后作**适配协议**候选重估。

下面区分「事实支持的」（F：在 Prove2Me 或 FLT 运行里实际发生并有链接）与「我们的推断」（I：映射到 HCTL2 后的设计判断，需要人拍板）。

### 1. 三张清单怎么接到图

- F：Prove2Me 的 milestone 必须链到一条 canonical Theorem 才算存在；没有形式陈述的里程碑只是文字，不是节点。
- I：把 hctl2-shaping 的三张清单当施工图的来料，映射规则只要三条——「已决」里能写成「交出什么 + 凭什么算交出」的条目才可成为义务节点候选；「尚未定形」一律不进图，只能成为 Request 或 Scoped Room（对应 Prove2Me 的「先 submit-problem 再引用」：节点得先存在）；「出界」不进图也不进依赖。塑形 Skill 现在只说「一题的答案依赖另一题时属于下一轮」，可以把这种依赖显式写成边，让三张清单本身就是一张小图，结晶时直接抬进施工图。

### 2. 节点进图的门：机械判据先说清

- F：Prove2Me 每个节点自带内核可判的验收（类型逐字匹配、无 sorry、公理白名单），判决附机器可读原因。
- F：我们已裁决「外部机械事实作为节点准入条件由工具箱读回」，证据分三级可信度。
- I：施工图结晶时每个义务节点必须声明二选一——「达成的机械判据」（某 CI 绿、某文件出现、某 PR 合并，由工具箱读回）或「需 Gate 席位裁决 + 证据不低于某级」。两者都说不清的义务不许进图，退回 Room 继续塑形。这把 Prove2Me「说不出形式陈述就不是节点」的纪律搬到没有内核的领域。

### 3. 边由机械物承载，前沿由查询给出

- F：Prove2Me 的边是编译器检查过的 import；前沿是 `/open-leaves` 的查询结果；父节点由子节点状态归约。FLT 收尾时 agent 靠 open_leaves 等于 0 判断，而不是靠记忆。
- I：施工图里的边应是「下游节点的准入条件引用上游节点的结晶或凭证标识」，由 control 在批准时校验无环且落地（Prove2Me 终检的两个词），运行中由 control 计算「现在可开工的节点」。引擎的 READY 是机械进度，语义准入仍归 HCTL2，与 run.md 的「引擎报告、HCTL 裁决」分工不变。

### 4. 先审义务陈述，再审交付物

- F：FLT 运行里「动手前通常有别的 agent 检查陈述是否为真」，早早抓出多条假陈述；Prove2Me 的 mission 核心上线前要经一个没看过原文的 auditor 盲读回，人对照原文。
- I：施工图批准前加一步「盲读回」：一个没参加讨论的 Participant 把冻结的施工图读回成自然语言——每个节点交什么、凭什么算交、依赖谁——人对照三张清单拍板。这是塑形阶段的 Gate，审的是义务定义；它比事后 Gate 便宜，因为假陈述不会被证明。

### 5. 运行中的分解：需要人拍板的方向问题

- F：Prove2Me 允许 solver 在运行中把一个节点归约成若干新子节点（新增节点，不改旧节点），父节点的达成条件自动变为「所有子节点达成」。FLT 的 Day 7 重接线就是这样发生的；29,511 个节点没有一个是人事先画好的。
- I：HCTL2 施工图开工前冻结，运行中只有放置参数可变。两种可选姿态：(a) 维持冻结，节点内的子分解属于尝试内部，不进图，不产生新席位与票；(b) 引入「可归约节点」——施工图里显式声明某义务允许在运行中派生子义务，子义务继承父义务的验收规则与席位策略，父义务在子义务全达成时归约达成，图变更以新 Workflow Revision 落账。Prove2Me 的证据支持「分解发生在施工中而不是全部在讨论里」；但 HCTL2 的义务有席位与票，节点增多就是票增多，这是 Prove2Me 没有的成本。这一条不是接口细节，是边界，留给人。

### 6. 粒度与重复：图不替你做复用

- F：报告自陈五分之二的陈述逐字重复、一条基础引理在 300 多个文件重声明；Prove2Me 文档的粒度规则只是经验句（不要平凡归约、不要过度拆分、贴自然结构）。
- F：task.md 已写「Task 是承诺尺度，不是工作项尺度」。
- I：施工图节点的粒度用承诺尺度卡住——一个节点是一份值得独立验收的义务，不是一步操作；哪些结晶可被多个节点引用，在结晶时显式标出，防止图把重复做成合法。

### 7. 散文与图并存时，让图当权威

- F：FLT 运行里 agent 的散文「共享计划」估时反复错，图的状态从未错；agent 收尾时对同事的措辞是「proved on prove2me，pending re-check」，把平台状态与终检状态分开说。
- F：我们已裁决「节点之间只经结晶接力、不传聊天史」。
- I：施工图的状态只住在账本与结晶里，Room 里只投影里程碑（run.md 已这样写）。补一条：估时、优先级这类散文不进图，进图的只有义务、判据、边。

## 未查到与边界

- **Zulip 讨论**：Lean 社区 Zulip 关于此事的讨论串未能通过搜索引擎索引到，本文未引用社区一手反应；Buzzard 的博文是唯一的社区一手评价。
- **失败尝试的细节**：几次、用什么基础设施、跨多久，Anthropic 未公开；「中途换上 Prove2Me」只见于二手报道，一手材料的措辞是「一些初始尝试失败」与「这次运行用了基于 Prove2Me 的新 harness」。
- **多模型**：整个事件只涉及 Anthropic 一个模型的多个实例；关于「不同厂商模型协作失败」的说法未找到任何依据。
- **无对照实验**：Prove2Me 论文没有「有图 / 无图」的消融；FLT 的 7% 是唯一定量前后数字，且 harness 的三项贡献未拆分。
- **成本与规模**：Anthropic 未公布金额，Buzzard 的「我猜他们花的比我多」是猜测；「几十个 agent」无确数。
- **Prove2Me 开放程度**：技能、工作区与公开定理开放；服务端代码是否开源未核实；workspace 仓库无许可证声明。
- **Meta 教材形式化**：只读了摘要，3 万 agent 在 git 上怎样分派、怎样处理冲突未核实。
- **Buzzard 说的「只对 p ≥ 17」**：报告里 Mazur 论证覆盖 p = 11 与 p ≥ 17、小素数另证；两处说法的精确边界本文未逐卡核对。

## 证据

- E1 Anthropic 博客《Formalizing Fermat's Last Theorem》，2026-09-04：<https://www.anthropic.com/research/formalizing-fermats-last-theorem>（失败尝试、7%、Prove2Me 三项贡献、人类介入的原话）
- E2 Anthropic 技术报告《Formalizing Fermat's Last Theorem in Lean: A timeline and selected excerpts from Claude's reasoning》：<https://www-cdn.anthropic.com/9e431dff043da6538d99d6c2d231b670aa3da263.pdf>（时间线、card 定义、Day 7 重接线、终检、重复率与构建成本）
- E3 仓库 anthropics/fermats-last-theorem @ `aa2d8b34`（2026-09-03，Apache-2.0）：<https://github.com/anthropics/fermats-last-theorem>（Lean 4.33.1 / Mathlib v4.33.0 @ `db584cd6`；1,450 定义模块、29,511 陈述模块、29,511 证明模块；comparator 与 nanoda 脚本；ATTRIBUTION.md 列 106 个文件）
- E4 Prove2Me 论文 arXiv 2608.28433（v1 2026-08-28，v2 08-31）：<https://arxiv.org/abs/2608.28433>；HTML 全文 <https://arxiv.org/html/2608.28433>（对象模型、Property 1、四个案例、与 Gloeckle 等的对比、开放问题）
- E5 Prove2Me 网站 About / FAQ：<https://prove2.me/about>、<https://prove2.me/faq>（起源、不可变、三个钉版环境 4.33.1 / 4.30.0 / 4.29.0-rc3、废弃重传）
- E6 prove2me/prove2me_workspace @ `6b46503a`（v0.9.8，2026-09-07；仓库建于 2026-07-03）：<https://github.com/prove2me/prove2me_workspace>；`SKILL.md`、`references/prove.md`（归约、import 边、unknown import、auto-resolve、粒度规则）、`references/mission_captain.md`（提案、里程碑、盲读回）
- E7 Prove2Me agent 入口：<https://prove2.me/start.md>
- E8 Kevin Buzzard《FLT: Anthropic has beaten me to it》，2026-09-04：<https://xenaproject.wordpress.com/2026/09/04/flt-anthropic-has-beaten-me-to-it/>
- E9 ImperialCollegeLondon/FLT @ `789312cb`：<https://github.com/ImperialCollegeLondon/FLT>；blueprint <https://imperialcollegelondon.github.io/FLT/blueprint.pdf>
- E10 Lean 社区对 FLT 项目的介绍：<https://leanprover-community.github.io/blog/posts/FLT-announcement/>、<https://lean-lang.org/use-cases/flt/>
- E11 PatrickMassot/leanblueprint v0.0.20 @ `56e066d3`：<https://github.com/PatrickMassot/leanblueprint>
- E12 LeanArchitect，ITP 2026：<https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ITP.2026.25>
- E13 Lean Atlas，arXiv 2604.16347：<https://arxiv.org/abs/2604.16347>；NyxFoundation/lean-atlas @ `3a81e194`
- E14 Equational Theories Project，arXiv 2512.07087：<https://arxiv.org/abs/2512.07087>；teorth/equational_theories @ `5c20aeba`
- E15 Gloeckle 等《Automatic Textbook Formalization》，arXiv 2604.03071：<https://arxiv.org/abs/2604.03071>；facebookresearch/repoprover @ `386adba3`；作者说明 <https://x.com/FabianGloeckle/status/2040082785851904401>
- E16 Math Inc Gauss / Strong PNT：<https://www.math.inc/gauss>；math-inc/strongpnt @ `2f5835c3`
- E17 ProofFlow，arXiv 2510.15981：<https://arxiv.org/abs/2510.15981>
- E18 DeepSeek-Prover-V2，arXiv 2504.21801：<https://arxiv.org/abs/2504.21801>
- E19 Seed-Prover，arXiv 2507.23726：<https://arxiv.org/abs/2507.23726>
- E20 Aristotle，arXiv 2510.01346：<https://arxiv.org/abs/2510.01346>
- E21 LLMCompiler，arXiv 2312.04511（ICML 2024）：<https://arxiv.org/abs/2312.04511>
- E22 Task-Decoupled Planning，arXiv 2601.07577：<https://arxiv.org/abs/2601.07577>
- E23 CodePlan，arXiv 2309.12499：<https://arxiv.org/abs/2309.12499>
- E24 HCTL2 内部：[run.md](../design/run.md)、[task.md](../design/task.md)、[hctl2-shaping SKILL.md](../../src/agency/skills/hctl2-shaping/SKILL.md)、[方法论生态审计](./methodology-landscape-20260824.md) §一、§三

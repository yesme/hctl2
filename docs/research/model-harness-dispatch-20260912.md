# 七组模型与 Harness：评测证据与派工参考

> 类别：跨候选归纳 · 证据编号：E-MODEL-HARNESS-DISPATCH<br>
> 状态：信息性研究快照 · 核查日期：2026-09-12；发布后正文不改，更新追加复核记录<br>
> 复用决定：仅参考行为；不引入依赖、评测服务或产品约束，不调整用户配置。目录规则见 [research 总览](./README.md)。

## 决定建议

**先按任务和证据派工，再按模型调整；没有证据把七家固定成七种职业。** 用户指定的七组配置见下一节。公开数据已足以形成初步判断，但还不能证明谁最会审 HCTL2 的中文设计、维护所有者裁决或控制改动范围。

- **Astra / Codex 与 Fable 5.1 / Claude Code**：复杂工程的首选候选，可以轮流承担主写和独立复核；完整代理评测支持两者能力接近，不支持“GPT 只能挑细节、Fable 才能做架构”的固定分工。
- **K3 / Kimi Code**：值得试长代码库阅读、行为追踪与有明确交付物的深入调查；代码库问答成绩是正面证据，但速度和长任务用量需要单独看。
- **GLM-5.3 / OpenCode**：值得试有明确验收的实现、终端问题与代码复核；不能因为“开源模型”就只派简单杂务。公开 OpenCode 评测尚缺思考档位说明。
- **Muse Spark 1.3 / Muse Code**：值得试有测试反馈的修改任务，不宜只按旧版本印象派文字润色；新版本已经有完整代理评测依据。
- **Grok 4.6 / Grok Build、Gemini 3.8 Flash / Antigravity**：可以承担独立核查和边界明确的工作；本次数据不足以把它们优先固定为最难跨层裁决的唯一复核者。Gemini 的代理评测用 SDK，不能直接当作本机 CLI / IDE 的成绩。

这些是后文证据导出的**试派建议**，不是产品决定，也不是能力上限。先前六份审核的专门方向主要来自工作覆盖与已有上下文；公开评测并没有证明那六种方向恰好是各模型的独有长处。

## 核查对象与配置

Harness 指模型外面的执行程序：提供工具、管理上下文、处理权限、恢复会话。用户给定的是“模型版本 × 思考深度 × Harness 产品”；本次另用七个命令的 `--version` 核对了这台 Mac 的 PATH 当前指向的程序，没有启动模型任务或修改配置。

| Harness | 用户指定模型 | 用户指定深度 | 本机可执行程序版本 |
| --- | --- | --- | --- |
| Codex | `gpt-6-astra` | `max` | `codex-cli 0.154.0` |
| Claude Code | `fable-5.1` | `max` | `2.1.269` |
| Grok Build | `grok-4.6` | `xhigh` | `1.0.25 (f7e67d6988e2)` |
| Kimi Code | `k3` | thinking `max` | `0.42.0` |
| OpenCode | `glm-5.3` | `max` | `1.18.30` |
| Antigravity | `gemini-3.8-flash` | `high` | `agy 1.2.1`；IDE 版本未核 |
| Muse Code | `muse-spark-1.3` | `max` | `1.1.1 (1.1.1-R2514.1)` |

这里的本机版本不证明已经运行的进程也是该版本，也不证明每次请求实际用了指定模型。当前会话的有效配置、服务端回退、上下文压缩、插件和工具权限没有逐会话核验；其他机器版本未知。模型配置按用户声明记录，没有用官网下载页的“最新版”替代现场版本。

### 思考档位与调用路径的差异

| 对象 | 官方可核事实 | 对派工的影响 |
| --- | --- | --- |
| Astra | API 标识为 `gpt-6-astra`，支持 `low / medium / high / xhigh / max`。[模型页](https://developers.openai.com/api/docs/models/gpt-6-astra) | 历史 GPT-5.6 Sol 的经验不直接算 Astra 证据；Codex 产品名本身也不保证模型相同。 |
| Fable 5.1 | API 标识为 `claude-fable-5-1`；effort 是行为引导而非严格 token 预算，支持五档，默认 high；高档仍受总输出上限影响。[effort 文档](https://platform.claude.com/docs/en/build-with-claude/effort) | 订阅名称 Claude Max 与推理 `max` 不是一回事。社区只说“Max 用户”的，思考档位仍记未知。 |
| Grok 4.6 | 官方 API 标识为 `grok-4.6`；默认 high，支持 xhigh；Grok Build 支持该模型。[模型文档](https://docs.x.ai/developers/grok-4-6) | high 的发布成绩不能填进用户 xhigh 那一格。 |
| K3 | 官方标识为 `kimi-k3`；技术报告的测试明确为 max，但不同测试用不同 Harness。官方还指出跨轮保留思考历史影响稳定性。[技术报告](https://www.kimi.com/en/blog/kimi-k3) | Kimi Code 与第三方接入不先当作等价；沿用其他模型的会话历史也可能改变表现。 |
| GLM-5.3 | 官方支持 `low / high / max`，默认 max，思考始终开启；它是纯文本模型。[模型文档](https://docs.z.ai/guides/llm/glm-5.3) | OpenCode 的 provider、协议和 variant 还影响实际请求；不把界面的档位标签当成传参已验证。图片任务需另有可读材料。 |
| Gemini 3.8 Flash | API 默认 medium，最高 high；Antigravity CLI 提供 `gemini-3.8-flash-high` 标识。[模型说明](https://ai.google.dev/gemini-api/docs/latest-model)、[CLI 文档](https://antigravity.google/docs/cli/headless/) | high 已是该模型最高档，不是与别人 max 相比“少想一级”；API、SDK、CLI、IDE 的成绩分开记。 |
| Muse Spark 1.3 | Meta 当前页面确认 max 已在 Muse Code 与 API 提供；其评测分别采用原生代理和固定代理。[发布说明](https://research.meta.ai/blog/introducing-muse-spark-1-3)、[评测方法](https://research.meta.ai/static/muse-spark-1-3-multimodal-evaluation-methodology) | 早期“max 尚未公开”的文章不能覆盖当前状态；1.2、1.3 xhigh、1.3 max，以及 Contributor 调用路径分开看。 |

因此，同样写 max 不代表同等算力、同等 token 或同等耗时；high 也不能跨家换算。本文没有据此建议改掉用户的档位。

## Benchmark：优先看完整代理，再看模型旁证

Benchmark 是任务集上的评测，不是对模型全部能力的考试。下表标的是来源实际测试的配置；没有公开的参数保留未知。

### 最接近本次配置：AA Coding Agent Index v1.5

Artificial Analysis（下简称 AA）独立测试三个分项：DeepSWE v1.1 的 113 个修改任务、Terminal-Bench 4.0 的 66 个终端任务、SWE-Atlas-QnA 的 124 个代码库问答。每题跑三次，统计单次成功率的平均值，再把三个分项等权平均成指数；不是“三次选最好”。前两项用执行检查，问答按 Scale 的判分办法、由 Claude Opus 4.5 判卷。[方法与版本史](https://artificialanalysis.ai/methodology/coding-agents-benchmarking)

数据读取于 2026-09-12。表内分数与费用沿用页面展示精度；“未知”是没有核到，不是推定默认值。

| 来源实际配置 | 修改 DeepSWE | 终端 TB 4.0 | 代码库问答 | 综合指数 | 美元 / 任务 | 分钟 / 任务 | 与用户配置的差别及来源 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Astra / max / Codex | 68% | 56% | 62% | 62 | 7.47 | 29.4 | 名称三项吻合；程序版本未核。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/codex-vs-opencode) |
| Fable 5.1 / max / Claude Code | 64% | 58% | 65% | 62 | 12.39 | 34.8 | 明标 **with fallback**，即可能回退到其他模型；不是纯 Fable 样本。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/claude-code-vs-opencode) |
| Grok 4.6 / xhigh / Grok Build | 65% | 18% | 58% | 47 | 3.57 | 19.5 | 名称三项吻合；程序版本未核。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/grok-build-vs-muse-code) |
| Kimi K3 / 深度未知 / Kimi Code CLI | 68% | 21% | 66% | 52 | 5.05 | 约 60 | 比较页没有标 max，方法页也未列该行有效参数。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/antigravity-sdk-vs-kimi-code-cli) |
| GLM-5.3 / 深度未知 / OpenCode | 61% | 40% | 59% | 54 | 4.24 | 48.1 | 比较页没有标 max，不能靠模型默认值补齐。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/codex-vs-opencode) |
| Gemini 3.8 Flash / high / Antigravity SDK | 66% | 15% | 45% | 42 | 2.47 | 11.7 | SDK 不是本机 `agy` 或 IDE 的已验证等价物。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/antigravity-sdk-vs-kimi-code-cli) |
| Muse Spark 1.3 / max / Muse Code | 72% | 32% | 59% | 54 | 3.98 | 18.4 | 名称三项吻合；程序版本未核。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/grok-build-vs-muse-code) |

费用是按 API 价格及缓存计算的任务成本，不是七家订阅的额度换算；时间是代理实际运行时间，不含环境启动与判卷。缓存重复输入计入 token，并不表示这些字节都是新思考。因此“每秒吐字快”“总 token 少”“订阅更耐用”是不同判断。[指标定义](https://artificialanalysis.ai/agents/coding-agents)

**从这张表可以推断什么：** Astra 与 Fable 是复杂工程的稳妥起点；Kimi 的问答成绩、Muse 的修改成绩、GLM 的终端成绩分别值得安排相应试派。**不能推断什么：** Kimi 比别人更懂 HCTL 状态机、Grok 天生更会反对、Fable 必然更懂愿景、或综合低分者不能发现高分者漏掉的问题。问答能读懂代码，也不等于能发现历史裁决被改坏；表中没有直接测后者。

该表没有给各行可比的误差区间，几个百分点不作稳定排名依据。即使任务集相同，Harness 版本、权限、提示、时间限制、回退和运行日期仍会影响结果；这不是同预算下的纯模型实验。

### 交叉核对：同名 benchmark 也会有不同成绩

Terminal-Bench 数据合作方 Snorkel 的 v4.0 榜列出下列结果，误差按原页面保留；它与 AA 是不同的评测记录，不能挑高分拼在一起。[榜单与口径](https://snorkel.ai/leaderboard/terminal-bench-4-0/)

| 模型 / 深度 | Harness | TB 4.0 解决率 | 配置差别 |
| --- | --- | --- | --- |
| Astra / max | Codex | 58.2% ± 2.8 | 产品名、模型、档位吻合 |
| Fable 5.1 / max | Claude Code | 57.9% ± 3.8 | 产品名、模型、档位吻合；回退策略需另核 |
| GLM-5.3 / max | Claude Code | 41.8% ± 3.2 | 不是 OpenCode |
| Grok 4.6 / high | Grok Build | 20.3% ± 3.1 | 不是 xhigh |
| Gemini 3.8 Flash / high | mini-SWE-agent | 19.1% ± 3.4 | 不是 Antigravity |

Astra 与 Fable 的点估计只差 0.3 个百分点，远小于两边的误差；本次没有理由从这里宣称一方明确胜出。Kimi、Muse 没出现在这份已读表里，不代表零分。

另一份独立记录是 [Datacurve DeepSWE v1.1 榜](https://deepswe.datacurve.ai/)（页面标更新于 2026-09-03）：在统一 mini-swe-agent 下，Gemini 3.8 Flash / high 为 74% ± 1，K3 / max 为 69% ± 5，GLM-5.3 / max 为 69% ± 3。这可作模型能力旁证，不能覆盖上表原生代理的实测，更不能据差值单独归因于 Harness。

AA 的模型评测与完整代理评测也分开：模型比较页上，Muse 1.3 的 TB 4.0 是 max 33%、xhigh 17%；Grok 4.6 是 high 21%、xhigh 17%。这至少说明“把档位提高，成绩一定提高”站不住；没有方差与逐题记录，不能进一步断言 xhigh 伤害 Grok。[Muse 档位比较](https://artificialanalysis.ai/models/comparisons/muse-spark-1-3-vs-muse-spark-1-3-xhigh)、[Grok 档位比较](https://artificialanalysis.ai/models/comparisons/grok-4-6-vs-grok-4-6-xhigh)

### 官方宣传数据的用处与边界

官方文档适合核实型号、档位、能力条件与作者已知限制；官方横评仍要读方法。Kimi 的不同项目会换 Harness、取其他厂商最好成绩；Meta 的方法也允许从自测、榜单或厂商报告中取最高可比值。这不是七组配置同时同条件的对照。[Kimi 方法](https://www.kimi.com/en/blog/kimi-k3)、[Meta 方法](https://research.meta.ai/static/muse-spark-1-3-multimodal-evaluation-methodology)

Fable 的生产防护可能触发模型回退，AA 也把它写在行名里。这个限制属于实际产品行为，不应删掉标记后把分数称为单个模型的能力。[Fable 官方说明](https://www.anthropic.com/claude/fable)

榜单自己的版本同样重要：AA 的 2026-09-02 Muse 发布分析里，max 的 Intelligence Index 是 62；当前 v4.3 页面是 48。两个版本的任务组成不同，不能说成“模型几天内退步了 14 分”。Coding Agent Index v1.5 与 Intelligence Index v4.3 更是两种不同指数。[发布时分析](https://artificialanalysis.ai/articles/muse-spark-1-3)、[当前模型比较](https://artificialanalysis.ai/models/comparisons/muse-spark-1-3-vs-muse-spark-1-3-xhigh)、[v4.3 变更说明](https://artificialanalysis.ai/articles/artificial-analysis-intelligence-index-v4-3)

## 社区实测：取具体行为，不取人气结论

本次检索覆盖官方论坛与文档、Reddit 的相关 Harness 社区、公开 GitHub 实测记录。下面是读取于 2026-09-12 的第一手样本，不是用户群体的随机抽样；赞数不作可信度或胜率。没有公开任务、全部输出与重复运行的，只能提示要检查什么。

| 样本 | 已知配置与缺项 | 作者观察 | 能支持的判断 |
| --- | --- | --- | --- |
| [Astra 使用讨论](https://www.reddit.com/r/codex/comments/1w8t735/how_are_you_using_gpt6_astra_in_codex_and_is_the/) | Codex / Astra；评论混有 max、其他档位以及 max/ultra 合称；程序版本未给 | 有人说解决了旧模型卡住的任务，也有人说很快耗完额度；明确 max 的评论提到设计规划改善 | 复杂任务值得试，额度要按实际会话记；不能从该帖估计七家成本或胜率。 |
| [Astra 与 Fable 5.1 弹簧动画对照](https://www.reddit.com/r/codex/comments/1w8mmmd/gpt6astra_vs_fable_51/) | 两者明确 max；Harness 及程序版本未明确；公开同一条提示 | 作者偏好 Fable 的视觉结果，称 Astra 更快；Fable 曾撞总输出上限；评论对物理正确性有分歧 | 单题“看起来更好”不等于验收正确，也不是架构能力测试。 |
| [Grok Build 使用者对照](https://www.reddit.com/r/GrokBuild/comments/1vs40aa/my_comparison_between_56_sol_and_46_grok/) | Grok Build / 4.6 / xhigh，对照是旧 Sol / max；无完整轨迹 | 作者觉得较少过度设计、反馈快，但有时需要更多催促 | “少造东西”与“漏做必要工作”要分开验收；不足以推导反方审阅最强。 |
| [Kimi Code action 作者记录](https://www.reddit.com/r/kimi/comments/1uzkv98/using_kimi_k3_to_implement_a_kimicodeaction_like/) | Kimi Code / K3；思考档位与版本未给，原帖为较早版本体验 | 认可主动完成工作的能力，也报告未等本人确认就合了 PR | 提示核授权边界；不把当时工具缺项写成今日 Kimi Code 的现状。 |
| [GLM SlopCodeBench 原始报告](https://github.com/michaelasper/benchmarks/blob/acefbb0965718747efaf922d2b9951e2dff94d3c/glm-5.3-pi-on-slop-code-bench.md) | GLM-5.3 / OpenRouter / pi 0.84.2 / **xhigh**；不是本机 OpenCode / max | 完整目录的逐步需求测试能量到新功能已过、旧行为仍坏，详情见下节 | 是累积需求与回归的具体证据；xhigh 的端到端映射未核，不能冒充 GLM 官方 max。 |
| [Antigravity 使用者实测](https://www.reddit.com/r/google_antigravity/comments/1w5i29e/review_of_gemini_38_flash_from_a_person_who/) | Gemini 3.8 Flash / high / Antigravity；版本、任务仓库与运行记录未公开 | 主帖认为短任务分析改善；同帖有人报告抓到跨服务问题，也有人报告只授权两点却做了十点 | 值得试明确范围的任务，同时核对是否越范围；不能当作普遍可靠或普遍不可用。 |
| [Muse 1.3 Rust / TypeScript 项目体验](https://www.reddit.com/r/opencodeCLI/comments/1w6ultd/hater_to_lover_muse_spark_13/) | 作者实际是经 OpenCode Go 接到 Codex；不是 Muse Code；服务层级与推理映射未核 | 作者认为比 1.2 好、较贴合既有视觉风格，也承认仍会出错 | 说明“旧 Muse 印象”值得重估；该帖价格、ultra 等标签不移植到用户配置。 |

### 最贴近本库风险的社区样本：追加需求后能否保住旧行为

GLM 的 [SlopCodeBench 报告](https://github.com/michaelasper/benchmarks/blob/acefbb0965718747efaf922d2b9951e2dff94d3c/glm-5.3-pi-on-slop-code-bench.md)钉于 `acefbb0965718747efaf922d2b9951e2dff94d3c`（2026-08-20）；测试目录为 v1.0 / `4d38d300`，共 36 个问题、196 个阶段，每阶段继续修改上一阶段的工作区。

- 满足中心需求的阶段：130/196；满足该阶段全部要求、不计旧回归：53/196；严格全过：24/196（12.2%）；整条轨迹始终全过：0/36。
- 其中 29 个阶段新要求全过、旧测试仍失败。12 个阶段没有重测完整历史，因此“严格通过”也要读该阶段的测试范围。
- 每题没有重复采样；14 个阶段曾续跑且早期轨迹未保留，runner 的准确修改版本也缺失。数据公开有价值，但精确复现仍有缺口。

这与本库“新 PR 又把旧裁决改坏”的担心相似，是**任务形状的旁证**，不是 HCTL 文档的实测。它不证明 GLM 特别差，也不证明其他六家能避开同类错误；选几个好看的子集与旧 Fable / Sol 打平，更不代表当前七家排名。

## 怎样把这些证据用到下一次派工

### 初始分工与它的证据强度

这里的“较强”指对该类任务有接近配置的公开代理数据，不指已经验证了在 HCTL2 的表现；具体岗位适配仍待本库观察。

| 配置（均沿用用户档位） | 优先试派 | 依据与把握 | 最值得观察 |
| --- | --- | --- | --- |
| Astra / Codex | 跨文件实现、复杂问题归因、设计与实现相互核对 | 工程能力依据较强；架构审阅是迁移推断 | 是否把小问题扩大成新机制；结论是否真有代码和裁决支持 |
| Fable 5.1 / Claude Code | 方案整合、主文写作或复杂实现，并与非作者交叉复核 | 工程能力依据较强；方案整合还依赖本项目已有上下文，不是榜单已证明的专长 | 是否准确贯彻每条已定取舍；细节遗漏、回退及输出上限 |
| K3 / Kimi Code | 代码库行为追踪、长材料取证、恢复路径核查 | 问答分项积极；AA 档位尚缺，迁移把握中等 | 引用是否真能推出结论；完整任务耗时、授权范围 |
| GLM-5.3 / OpenCode | 有测试的实现、构建与终端问题、逐条约束核对 | 原生代理终端分项积极；深度未钉，迁移把握中等 | 新需求与旧要求能否同时保住；避免只有中心功能过关 |
| Grok 4.6 / Grok Build | 边界明确的修复、独立找反例、另一份实现解释 | 原生代理配置吻合；“找反例”是工作分配假设，把握中等偏低 | 反例是否可重现，是否需要多轮提醒补齐必要工作 |
| Gemini 3.8 Flash / Antigravity | 明确范围的调查与修改；读者能否照文档做成的核查 | SDK 修改与时长有正面信号，CLI / IDE 迁移仍待验证 | 对路径、引用和结果的实际回读；有没有擅自多做 |
| Muse Spark 1.3 / Muse Code | 有测试反馈的实现、独立代码阅读、第二份用例检查 | 原生 max 的修改分项有依据；用例审阅仍是试派 | 是否只照最新要求做；长对话能否保持之前的限制 |

**对已经发出的六份审查：不需要因这份报告重新派一遍。** 先保留独立意见；汇总时看可确认的问题与证据，不按品牌、篇幅或赞成票数决定。下一轮在部分相同题目上轮换专门方向，才有机会区分“模型优势”与“这个会话熟悉这一段历史”。

### 用现有 PR 留下可用记录，不先造评测平台

可以在现有审阅汇总里同时记：本次有效模型与档位、Harness 版本和工具权限、所读提交与上下文范围、确认的问题、误报、遗漏的已知问题、修正是否引入回归，以及总耗时与可观测用量。程序版本、会话状态和原生使用记录足够先做，不需要为这次研究新增脚本。

不同专门方向发现数不能直接比：拿到更多问题的文件自然更容易出成果。保留少量共同核对内容、其余各查专长方向，能改善可比性；第一轮先独立，再看他家意见。没有已知完整问题集时，只能记“查到多少已确认问题”，不能声称测出了召回率。

派工前先问三件事：任务最难的是取证、取舍还是实现？哪一个会话已经掌握相关历史？结果有什么外部办法核验？模型榜单是这三个问题之后的参考，而不是替代它们。

## 未查到与边界

- 没找到同时覆盖这七组精确配置、以中文设计一致性、所有者裁决保真、反对意见质量为目标的公开对照。上面的职业分配都没有达到这个证明强度。
- 本次没有核到 AA 的 K3 / GLM 两行实际思考档位，以及各行完整 Harness 版本、有效配置和回退轨迹；因此没有“全七家配置完全吻合”的统一排名。
- 社区多数没有重复测试；供应商、额度、地区、客户端更新和上下文历史也不一致。本文不把“变笨了”“最聪明”等描述转换成事实结论。
- 公开题集存在训练污染与针对题集优化的风险；厂商最高可比值、单次演示和不同任务集的百分数不能合成一条胜负关系。本次没有做污染审计。
- 没有调用七家模型做收费试验，没有复跑公开 benchmark；本机验证仅限版本命令及本报告的仓库文档检查。没有推定 API 费用等于订阅扣量，也没有建议降低任何一家既定思考深度。

## 与已有研究的关系及更新条件

已有 [多 agent 协作有效性](./multi-agent-effectiveness-20260908.md)研究流程与实验方法；[Harness adapters](./harness-adapters.md)研究协议与事件；[Harness access](./harness-access.md)研究接入方式。它们都不是七组当前配置的派工能力对照，旧 Gemini CLI 的证据也不直接归给 Antigravity。

本文只补这块横向证据，所以放在 research 根目录，不给七个模型各建一份重复的方法说明。模型版本、思考策略、Harness 或服务路由改变后，追加新的日期与配置记录；原先的分数仍属于原配置。下一次最有价值的补充，是本轮六家审阅在同一批事实上的实际表现，而不是再抄一张总榜。

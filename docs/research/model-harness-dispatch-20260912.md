# 七组模型与 Harness：评测证据与派工参考

> 类别：跨候选归纳 · 证据编号：E-MODEL-HARNESS-DISPATCH<br>
> 状态：信息性研究快照 · 核查日期：2026-09-12；发布后正文不改，更新追加复核记录<br>
> 复用决定：仅参考行为；不引入依赖、评测服务或产品约束，不调整用户配置。目录规则见 [research 总览](./README.md)。

## 决定建议

**保留七组指定配置；编码任务参考对应分项，中文设计评审参考本库已核案例与现有分工。** 模型版本、思考深度、Harness 和调用路径一起记录，配置见下一节。此次结论分三件事：

- **有测试的代码修改**：Muse Spark 1.3 / max / Muse Code 值得加入候选；终端操作密集的任务，Astra / max / Codex 与 Fable 5.1 / max / Claude Code 是优先候选。依据是相应分项，不是把综合指数解释成架构能力；Fable 的回退标记保留。
- **代码库问答与静态行为追踪**：K3 / Kimi Code 值得试，但 AA 未公开该行思考档位，不能把成绩直接填给本机 max，也不能从阅读成绩推出动态排障或恢复设计专长。其余配置的可用线索与限制见[按任务选候选](#按任务选候选)。
- **HCTL2 设计与约束评审**：沿用已定的[改写分工](../../.memo/design/case-study-20260907/05-rewrite-process.md#三人手)，不因榜单改派。本库案例支持继续使用这些搭配，也支持继续观察 Muse 的逐项核对；尚不足以把岗位变成某个模型的专属能力，或取消其他家的评审资格。

公开评测和本库经验各有用处：前者提供编码任务的先验线索，后者更贴近设计评审。两者都没有完成七组精确配置的同题、同预算对照；本文不调整产品约束、人员安排或推理档位。

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

这里的本机版本不证明已经运行的进程也是该版本，也不证明每次请求实际用了指定模型。本机版本不能用来补下文评测的 Harness 版本，后者尚未核齐。当前会话的有效配置、服务端回退、上下文压缩、插件和工具权限没有逐会话核验；其他机器版本未知。模型配置按用户声明记录，没有用官网下载页的“最新版”替代现场版本。

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

Artificial Analysis（下简称 AA）测试三个分项：DeepSWE v1.1 的 113 个修改任务、Terminal-Bench 4.0 的 66 个终端任务、SWE-Atlas-QnA 的 124 个代码库问答。每题跑三次取平均，再分别汇总分项、把三项等权平均成指数；不是“三次选最好”。修改和终端任务有执行检查；问答按 Scale 的判分办法由 Claude Opus 4.5 判卷，终端任务另用 Claude Code / Sonnet 5 检查是否绕过测试或获取外部答案。因此并非所有分数都只靠机械判定。判卷模型与部分被测产品同属一家，是待检查的偏差来源，不是偏袒已被证实；本次未核到独立判卷者的校准结果。[方法与版本史](https://artificialanalysis.ai/methodology/coding-agents-benchmarking)

数据读取于 2026-09-12。表内分数与费用沿用页面展示精度；“未知”是没有核到，不是推定默认值。

| 来源实际配置 | 修改 DeepSWE | 终端 TB 4.0 | 代码库问答 | 综合指数 | 美元 / 任务 | 分钟 / 任务 | 与用户配置的差别及来源 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Astra / max / Codex | 68% | 56% | 62% | 62 | 7.47 | 29.4 | 名称三项吻合；程序版本未核。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/codex-vs-opencode) |
| Fable 5.1 / max / Claude Code | 64% | 58% | 65% | 62 | 12.39 | 34.8 | 明标 **with fallback**；未核到逐次回退轨迹，不能确认是纯 Fable 样本。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/claude-code-vs-opencode) |
| Grok 4.6 / xhigh / Grok Build | 65% | 18% | 58% | 47 | 3.57 | 19.5 | 名称三项吻合；程序版本未核。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/grok-build-vs-muse-code) |
| Kimi K3 / 深度未知 / Kimi Code CLI | 68% | 21% | 66% | 52 | 5.05 | 约 60 | 比较页没有标 max，方法页也未列该行有效参数。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/antigravity-sdk-vs-kimi-code-cli) |
| GLM-5.3 / 深度未知 / OpenCode | 61% | 40% | 59% | 54 | 4.24 | 48.1 | 比较页没有标 max，不能靠模型默认值补齐。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/codex-vs-opencode) |
| Gemini 3.8 Flash / high / Antigravity SDK | 66% | 15% | 45% | 42 | 2.47 | 11.7 | SDK 不是本机 `agy` 或 IDE 的已验证等价物。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/antigravity-sdk-vs-kimi-code-cli) |
| Muse Spark 1.3 / max / Muse Code | 72% | 32% | 59% | 54 | 3.98 | 18.4 | 名称三项吻合；程序版本未核。[AA](https://artificialanalysis.ai/agents/coding-agents/comparisons/grok-build-vs-muse-code) |

GLM 行展示的三个分项均值为 53.33，而页面指数为 54。这里保留来源数字；可能涉及未展示的小数精度，但未核到原始精度，差额尚未解释，不能直接认定是舍入，也不自定容差。

费用和时间把三类任务的尝试合在一起取均值，缺失的计量值不当零；这与综合指数的分项等权不是同一权重。费用按 API 价格及缓存计，不是订阅扣量；表内时间是代理实际运行时间，不含环境启动与判卷。失败尝试也在其中，较早失败可能更快、更便宜；这些均值不是交付一个合格结果的成本，也不是本库某类任务的耗时预测。缓存重复输入计入 token，不等于新思考。[指标定义](https://artificialanalysis.ai/agents/coding-agents)、[汇总方法](https://artificialanalysis.ai/methodology/coding-agents-benchmarking)

表内点估计支持按任务选候选：Muse 的修改、Kimi 的问答、Astra 与 Fable 的终端分项有相应证据，GLM 的终端分项也支持纳入比较。它没有测中文设计、所有者裁决保真或反方评审，不能由此推出这些岗位的模型排名。问答确实包含追踪代码与解释行为，但不等于运行中的排障。

该表没有给各行可比的误差区间，几个百分点不作稳定排名依据；同为 62 也不证明两者能力等价。Harness 版本、权限、提示、时间限制、回退和运行日期仍会影响结果，这不是同预算下的纯模型实验。

### 交叉核对：同名 benchmark 也会有不同成绩

Terminal-Bench 数据合作方 Snorkel 的 v4.0 榜列出下列结果，± 数字按原页面保留；本次未核定它表示的统计量与区间水平。这是另一份公开榜单，不因发布方不同就推定与 AA 独立复测；两边共享题集，逐次运行记录的关系未核。[榜单与口径](https://snorkel.ai/leaderboard/terminal-bench-4-0/)

| 模型 / 深度 | Harness | TB 4.0 解决率 | 配置差别 |
| --- | --- | --- | --- |
| Astra / max | Codex | 58.2% ± 2.8 | 产品名、模型、档位吻合 |
| Fable 5.1 / max | Claude Code | 57.9% ± 3.8 | 产品名、模型、档位吻合；回退策略需另核 |
| GLM-5.3 / max | Claude Code | 41.8% ± 3.2 | 不是 OpenCode |
| Grok 4.6 / high | Grok Build | 20.3% ± 3.1 | 不是 xhigh |
| Gemini 3.8 Flash / high | mini-SWE-agent | 19.1% ± 3.4 | 不是 Antigravity |

Astra 与 Fable 的点估计只差 0.3 个百分点；没有成对比较或等价检验，既不宣布胜者，也不据区间重叠宣布能力相同。Kimi、Muse 没出现在这份已读表里，不代表零分。

另一份公开记录是 [Datacurve DeepSWE v1.1 榜](https://deepswe.datacurve.ai/)（页面标更新于 2026-09-03）：在统一 mini-swe-agent 下，Gemini 3.8 Flash / high 为 74% ± 1，K3 / max 为 69% ± 5，GLM-5.3 / max 为 69% ± 3，± 的定义同样未核。这提供不同代理配置的旁证，不能覆盖上表原生代理的实测，更不能据差值单独归因于 Harness。

AA 的模型评测与完整代理评测也分开：模型比较页上，Muse 1.3 的 TB 4.0 是 max 33%、xhigh 17%；Grok 4.6 是 high 21%、xhigh 17%。展示值并不随档位单调增加；没有方差与逐题记录，不能据此判定提高档位的因果效果，更不据此改变用户配置。[Muse 档位比较](https://artificialanalysis.ai/models/comparisons/muse-spark-1-3-vs-muse-spark-1-3-xhigh)、[Grok 档位比较](https://artificialanalysis.ai/models/comparisons/grok-4-6-vs-grok-4-6-xhigh)

### 官方宣传数据的用处与边界

官方文档适合核实型号、档位、能力条件与作者已知限制；官方横评仍要读方法。Kimi 的不同项目会换 Harness、取其他厂商最好成绩；Meta 的方法也允许从自测、榜单或厂商报告中取最高可比值。这不是七组配置同时同条件的对照。[Kimi 方法](https://www.kimi.com/en/blog/kimi-k3)、[Meta 方法](https://research.meta.ai/static/muse-spark-1-3-multimodal-evaluation-methodology)

Fable 官方说明，生产防护对部分网络安全与生物安全请求会路由到 Opus，API 使用方需配置回退。这是已公开的产品机制；AA 的 with fallback 标记本身不说明本次有多少请求触发、为何触发或去了哪个模型。保留标记，不把可能回退写成已知混合比例，也不把成绩称为已核验的单模型能力。[Fable 官方说明](https://www.anthropic.com/claude/fable)

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
- 其中 29 个阶段新要求全过、旧测试仍失败；不是 29 次“新改动引入回归”。报告的 `execution_server` 案例中，两项校验从第一阶段就没过，并持续到后续阶段，属于旧缺陷未修。12 个阶段没有重测完整历史，因此“严格通过”也要读该阶段的测试范围。
- 196 个阶段依赖此前工作区，不是 196 次独立实验；每题没有重复采样。14 个阶段曾续跑且早期轨迹未保留，runner 的准确修改版本也缺失。数据公开有价值，但精确复现仍有缺口。

它支持检查累积要求，而不是只验最新需求；其中旧缺陷持续与新引入回归仍要分开。这是**任务形状的旁证**，不是 HCTL 文档实测，不证明 GLM 特别差或所有模型必然退化。也不能由一个配置推出七家的回归率，或直接决定新增检查器。

## 本库审阅记录能证明什么

本库已有设计评审案例，不是只能等将来采样。下表核对原评论与 [#216 修订报告](https://github.com/yesme/hctl2/blob/d2521afcd0568f4d7a62eb143db87470b6e3e448/.memo/review/20260912-v0.17.5/codex-20260912.md#六家复核后的修订2026-09-12)中的处置；#211 的例子另对照 `4e535e7` 的计票文字。它们证明具体问题被发现或原判断被纠正，不是按品牌统计胜负。

| 评论署名与来源 | 已核到的内容与处置 | 对下次派工的有限启发 |
| --- | --- | --- |
| [Codex · #211](https://github.com/yesme/hctl2/pull/211#issuecomment-5625111419) | 指出完整 Bundle 含消费者身份，不能直接作两席材料相同的计票键；终稿区分完整包与交付内容摘要 | 有直接的约束机制审阅经验，不必从代码榜单猜架构能力 |
| [Fable · #216](https://github.com/yesme/hctl2/pull/216#issuecomment-5639868658) | 纠正节点范围、C 批排期与作者推论冒充裁决；其“派发时永久定重复组”的修法未采纳 | 原意与历史核对有用，修法仍须检查候选切换等反例 |
| [Grok · #216](https://github.com/yesme/hctl2/pull/216#issuecomment-5639668272) | 指出独立票组先到先关本来合法，不能把所有顺序差都当缺陷；报告收窄前提 | 有反驳错误前提的实际案例，不等于天生最擅长反方 |
| [GLM · #216](https://github.com/yesme/hctl2/pull/216#issuecomment-5639675506) | 把“控制面决定并记录”纠正为“准入并记录”，区分人的选择与控制面职责；另补过时入口 | 支持继续试对象职责与引用核对；不限定它只能做词汇工作 |
| [Kimi · #216](https://github.com/yesme/hctl2/pull/216#issuecomment-5639819401) | 补未投否决席、中途候选切换后成组、产出调用不唯一等情形；报告纳入 F1/R3 | 失败路径与边界核查有直接例子，不是从代码库问答迁移来的结论 |
| [Muse · #216](https://github.com/yesme/hctl2/pull/216#issuecomment-5639826556) | 补设计层同句、规则计数、清单来源与证据通道判定时机；后两项纳入 R1/R2 | 值得再试跨文件逐项核对；一次案例不能定为最强 |
| [Gemini · #216](https://github.com/yesme/hctl2/pull/216#issuecomment-5639669251) | 补合入拓扑与残留句；把端口绑定代次当成已退休对象的判断被现文证伪，只读路径例外也未采纳 | 核对有贡献，语义判断有误报；尚不足以取消设计评审资格 |

作者也在被检验：Codex 的 #216 初审 F3 对设计范围、F4 对立即修复、F5 对所有者裁决的归因，均被同一份修订报告收窄或撤回。F2 的部分错句在 #211 评审时已有报告，PR 描述称已修，合入后却仍在；“已采纳”与实际改对不是同一事实。这个案例既不能归咎于某家模型的榜单弱项，也不支持把作者身份排除后宣称该家少有错误。

#216 有共同报告与基线，但各家专门方向、会话历史不同；评论标题也不能证明每次实际模型、档位和 Harness 版本与本页七组一致。[Fable 的本库汇总](https://github.com/yesme/hctl2/pull/217#issuecomment-5642451305)提供了更多检索入口，本次没有复算其全部计数与采纳率，不采用据此给出的能力排名。采纳受作者判断影响，条目可拆可合，任务暴露的缺陷数也不同；“维持”可能是正确确认，“修正”也可能是误报，二者都要凭核对过程与结果判断。

这些案例与[改写规矩 §三](../../.memo/design/case-study-20260907/05-rewrite-process.md#三人手)的安排相容：Claude 写偏架构、愿景的稿，Codex 主审；交付、计划可以反过来；Grok 副审，K3 或 GLM 按任务性质陪审。它是所有者基于使用经验作的工作安排，不是对各家能力上限的声明。继续用它有实际依据，升级为模型独占岗位则证据不足。

## 怎样把这些证据用到下一次派工

### 按任务选候选

下面只把评测直接覆盖的任务列作公开证据建议；中文设计的方向来自上一节，不混成一张模型总排名。均沿用用户给定档位，不填补评测的未知参数。

| 要做的工作 | 当前可用的选择依据 | 还需核对什么 |
| --- | --- | --- |
| 有测试反馈的代码修改 | 七组都有修改分项；Muse / max / Muse Code 的 72% 是加入候选的积极信号，不据几个百分点宣布唯一首选 | 相似语言、仓库与验收；Grok 的原生配置有直接数据，Gemini 的 SDK 数据到本机 CLI / IDE 仍有差别 |
| 终端操作、环境配置与排障 | Astra 与 Fable 的两份榜单点估计较高，作为优先候选；GLM / OpenCode 的 40% 支持作为替代候选比较 | GLM 实测深度、Fable 回退、具体子任务；低分行不是所有终端任务的禁用依据 |
| 读代码解释静态行为 | Kimi 的 66% 是试用线索，Astra / Fable 等也有相近分项数据 | Kimi 实测档位、长任务时间；不把 QnA 成绩当恢复机制设计成绩 |
| 中文设计、裁决保真与约束复核 | 延续已定分工与本库经验；Muse 的逐项核对可继续试，方向轮换需有共同题便于比较 | 看实际反例、引用和修订，不用 AA 指数或一轮采纳数授予专属岗位 |

费用、时间是取舍的一部分，但先比较相似任务的合格结果，再计上补查、返工和交接成本。不能只凭较低平均耗时就让便宜模型筛掉全部问题、昂贵模型只裁留下的部分：首轮漏掉的问题不会自动进入复核。这里没有足以选择这种分级流程的本库对照。

### 用现有 PR 留下可用记录，不先造评测平台

可以在现有审阅汇总里同时记：本次有效模型与档位、Harness 版本和工具权限、所读提交与上下文范围、确认的问题、误报、遗漏的已知问题、修正是否引入回归，以及总耗时与可观测用量。程序版本、会话状态和原生使用记录足够先做，不需要为这次研究新增脚本。

保留少量共同核对内容、其余轮换方向，能改善可比性；第一轮先独立，再看他家意见。没有已知完整问题集时，只能记“查到多少已确认问题”，不能声称测出了召回率。对每一家同样核原意、授权边界、实际修订和旧要求；社区个案不变成某个品牌专属的缺点清单。

派工前先问三件事：任务最难的是取证、取舍还是实现？哪一个会话已经掌握相关历史？结果有什么外部办法核验？模型榜单是这三个问题之后的参考，而不是替代它们。

## 未查到与边界

- 没找到同时覆盖这七组精确配置、以中文设计一致性、所有者裁决保真、反对意见质量为目标的公开对照。本库评论补上了相关案例，但不是可比较的成功率或模型能力排名。
- 本次没有核到 AA 的 K3 / GLM 两行实际思考档位，以及各行完整 Harness 版本、有效配置和回退轨迹；因此没有“全七家配置完全吻合”的统一排名。
- 社区多数没有重复测试；供应商、额度、地区、客户端更新和上下文历史也不一致。本文不把“变笨了”“最聪明”等描述转换成事实结论。
- 公开题集存在训练污染与针对题集优化的风险；厂商最高可比值、单次演示和不同任务集的百分数不能合成一条胜负关系。本次没有做污染审计。
- 没有调用七家模型做收费试验，没有复跑公开 benchmark；本机验证仅限版本命令及本报告的仓库文档检查。没有推定 API 费用等于订阅扣量，也没有建议降低任何一家既定思考深度。

## 与已有研究的关系及更新条件

已有 [多 agent 协作有效性](./multi-agent-effectiveness-20260908.md)研究流程与实验方法；[Harness adapters](./harness-adapters.md)研究协议与事件；[Harness access](./harness-access.md)研究接入方式。它们都不是七组当前配置的派工能力对照，旧 Gemini CLI 的证据也不直接归给 Antigravity。

本文只补这块横向证据，所以放在 research 根目录，不给七个模型各建一份重复的方法说明。模型版本、思考策略、Harness 或服务路由改变后，追加新的日期与配置记录；原先的分数仍属于原配置。后续优先积累同类任务的实际修订结果及有效配置，核对已采纳意见有没有落实、落实后有没有改错，而不只追加总榜或采纳计数。

# mu

> 类别：① Coding Harness · 证据编号：E-L1-MU<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与复用决策用语见 [docs/research/README.md](../README.md)。原始材料（问题描述与 GPT、Grok、Claude 三家结论原文）见 [`.memo/notes/mu-research-raw-20261002.md`](../../../.memo/notes/mu-research-raw-20261002.md)。

<a id="e-l1-mu"></a>
## E-L1-MU · mu

本条只记录。mu 的判断机制对 HCTL2 Room 阶段有没有用、怎么用，所有者 2026-10-02 明确「不要着急定，先记录」；文末[对 HCTL2 的位置](#对-hctl2-的位置只记录不裁决)只列三家的建议与分歧，不作采纳结论。

### 审计基线

| 对象 | 版本 | 许可 |
| --- | --- | --- |
| [mu `8dfebe3`](https://github.com/qybaihe/mu/tree/8dfebe36508ac0c2508735bb866756dab9283e74)（2026-09-30） | Pi 0.87.1 的分叉；qybaihe 自 2026-09-22 起 277 个提交；判断内核 `packages/kyrn-judge` 0.1.0 | 根目录 MIT（沿用 Pi）；`kyrn-judge` MIT；桌面壳 `desktop/` Apache-2.0（AionUi 分叉） |
| [Pi `v0.99.0 / 4b060d3`](https://github.com/earendil-works/pi/tree/4b060d3a98618019adb9985d517516c8e99a2bbe)（2026-09-29） | 上游对照：引入原生分类模型的版本；Pi 的当前审计基线见 [`harness-access.md` 复核记录](../harness-access.md#复核记录) | MIT |

作者的设计材料在 mu 仓库 `kyrn/docs/`：`01-jev-integration-brainstorm.md`（选题来源）、`03-local-judge.md`（本地判断器与措辞消融）、`08-jev-retrospective.md`（回测）、`09-test-log-admission.md`（测试日志对照实验）等。本条未运行真实 Jev，未做桌面实机测试；效果数字都是作者自己的记录。

### 它是什么

mu 是 Pi 的深度二开：主模型照常规划、写代码、调工具；`packages/kyrn-judge` 挂在 Pi 的扩展事件上，在输入、工具调用、工具结果、回合结束等位置插入一层**有限选项的判断**，判断后由本地代码决定动作。桌面端是 AionUi 的 Electron 壳（React 19、Arco Design、UnoCSS），不是 GPUI；判断内核是纯 TypeScript 库，与界面无关。

### 判断内核（源码核对）

**一个抽象：`DecisionSpec`**（`packages/kyrn-judge/src/decision.ts`）。每个决策点声明 `id` 与 `version`（题面或策略一改就加一）、`questions` 或按候选动态出题的 `questionsFor`、把输入压成小状态的 `buildState`、把概率变成动作或弃权的 `policy`、没有判断时的 `fallback`（等于宿主原生行为），以及能力需求 `capabilities`（classify / relate / rate / meta）、`cacheImpact`、`latency`（inline / parallel / background）。题型三种：是非、选择、打分。

**引擎纪律：**

- **三种模式**：`off` 不问；`shadow` 问、记录、但返回 fallback；`active` 才生效。缺省全部 `shadow`。
- **失败放行**：判断器超时、报错、答案畸形或弃权，一律走 fallback，`decide()` 不抛错。安全侧例外：`tool.risk` 拿不到答案就问用户，不放行。
- **三区间加逃生选项**：是非题 P ≥ 0.8 算是、P ≤ 0.2 算否、中间弃权；选择题选中项低于 0.6 弃权，但提供方不返回概率分布时这道门槛被跳过，只剩逃生选项检查（`policy.ts:45, 57`）；选择题必须含 `none / other / unclear / unknown` 之一，否则定义时报错，除非该点显式声明选项集是闭集（`allowChoicesWithoutEscape`）。
- **级联与能力档案**：可配 `laya,jev`，本地先答、拿不准的再送下一级；判断器在档案里声明不擅长的能力（本地 Laya 不做关系题与打分），该类题直接跳级；没有任何一级可信时答案置为中性。
- **判定记录**：每次判定一条，含决策 id 与版本、模式、提供方与模型、答案与概率、最终动作来自判断还是 fallback、耗时、用量，以及状态的 SHA-256 摘要。缺省不存完整输入状态，所以记录不等于能完整回放。
- **判断与写字分开**：判断器不生成文字；任务帧、经验条目、人话看板由另配的 writer 模型写，代码校验 writer 输出（约束条目须是用户原句，格式不对整条拒收、保留上一版）。任务帧的 writer 缺省开启，未配 writer 时用会话模型，连续失败两次退回规则合帧、约束逐字保留（`extension/features/frame.ts:97–98, 215–235`）。

**shadow 不等于全无作用。** 引擎在 `shadow` 下把 fallback 放进 `outcome`，同时暴露原始判断 `judged`；有几处调用方直接读 `judged`：

| 位置 | 读法 | 源码注释或含义 |
| --- | --- | --- |
| `extension/features/permissions.ts:224, 247` | `decision.judged ?? decision.outcome` | 用户选了「Jev 审批」时，shadow 判断也计入审批 |
| `extension/features/swarm.ts:549–552` | `route.judged` 选代理与模型档位 | 「判断器在任何模式下都负责路由：派出的子任务没有『原行为』可回退」 |
| `extension/features/hive.ts:449` | `routes[at].judged ?? routes[at].outcome` | 蜂群成员的分派同上 |
| `extension/features/compaction.ts:139–141` | `decision.judged ?? decision.outcome` | 只打分并报告压缩计划，shadow 下不执行 |

GPT 的结论只指出了权限一处，其余三处是本条核对时补上的。

### 35 个决策点

**数量。** `src/manifest.ts` 登记 35 个唯一决策 id，`manifest.test.ts` 收集源码里全部 `defineDecision` 的 id 与之比对。决策点数 ≠ 原子题数 ≠ 模型请求次数：`input.preflight` 一个点就有 10 道题（1 道分类、6 道是非、3 道打分），共享一份状态、一次请求；另一些点按候选出题（一条技能一题、一块输出一题）。README 所说「每一轮 35 个判定点」是目录规模，不是每轮问 35 次。

| 阶段 | 决策点 |
| --- | --- |
| 输入 | `input.preflight`、`task.frame`、`input.interjection` |
| 上下文 | `skills.disclosure`、`capability.disclosure`、`tool.admission`、`tool.admission.test-log`、`context.forget`、`context.compact`、`memory.recall`、`memory.capture`、`memory.outcome`、`memory.worth`、`memory.merge`、`memory.applied`、`cache.warming` |
| 工具 | `tool.risk`、`tool.approval`、`tool.constraint`、`files.locate`、`browser.step`、`review.triage`、`diagnostics.delivery` |
| 回合 | `turn.drift`、`turn.rewind`、`turn.completion`、`output.drift`、`goal.met`、`board.read`、`notify.routing` |
| 协作 | `swarm.routing`、`swarm.patch`、`hive.publish`、`hive.deliver`、`hive.relate` |

**什么时候问。** 全部挂在 Pi 的事件钩子上，没有定时轮询，且大多先过确定性规则：漂移监测每 6 次工具调用在后台问一次 `turn.drift`，而「同一调用同一结果连续 3 次」的打转由规则判、不问模型（`monitor.ts`：`every: 6, repeats: 3`）；工具输出超过 6000 字符才分块（每块 1200 字符）问 `tool.admission`，源码读取与写入、已整理的子代理报告、不到 6000 字符的短输出都不判（`admission.ts:53, 62–63`）；用户每条消息并行问 `input.preflight` 与 `task.frame`；回合结束问 `turn.completion` 与经验类；蜂群成员每轮结束问 `hive.*`。

**怎么选出来的。** 不是 Jev 自带题库，也不是优化计算的结果，是人工工程分解：

1. 2026-09-20 的 `01-jev-integration-brainstorm.md` §4 按 Pi 一轮的生命周期列出 33 个候选：编号的 32 个（A 输入 8、B 循环内 9、C 蜂群 5、D 收尾 4、E 会话级 6），外加不编号的 F「把判断器直接给主模型当工具」。原始材料 Claude 节记作 36，按表格复数是 33，§5 把作者自己用 harness 时反复遇到的八件事逐条对上去。
2. 入选按五条铁律筛：判断器判断、代码执行、主模型思考；规则先吃确定区，判断器只管灰区；错了必须便宜且可逆；入口准入优于事后遗忘；每次判断入记录、题面带版本。另列「不该给判断器的」：要写字的、要多步前瞻的、没有逃生选项的开放集、超过 32K 的状态、单独充当安全边界的。
3. 淘汰与改题：B7 逐轮调思考强度不做（换参数会丢 prompt cache）；A2「先澄清」删除（真实会话里模型反问而不去查工作区）；`goal.met` 降为兜底（用户反馈读不出测试仍是红的，改由生成模型结合事实判断）；`tool.admission` 从「和目标相关吗」改成「这是什么类型的输出」（关系题把噪声也判成相关）；B6 自动重试、E2 旁支隔离、C4 子代理结果去重因要改 Pi 内核搁置；F 未做。`task.frame`、`tool.constraint`、`tool.approval`、`turn.rewind`、`board.read`、`hive.relate` 与几个经验类点在 9 月 22–23 日补入。
4. 作者自己的刹车（产品需求文档 §12）：比起继续加决策点，优先任务状态准确、原文可靠召回、完成证据可信、效果可量化。

**措辞比选题更要紧。** `03-local-judge.md` §4.2 用 31 条有标注的消息做措辞消融：同一模型、同一状态，「是否要创建、编辑或删除文件」问法 42%，「`user_message` 是否要求改代码」问法 74%。扩展规则写在 `input-preflight.ts` 头注释：一题一个短谓词，状态字段用反引号点名，改措辞就升版本。同一文件另有 judge-bench 的 37 条手工标注场景，用于 `input.preflight` v3 的三区间阈值体检，与上面的 31 条不是同一组。

**怎么扩展。** 写一个 `defineDecision`；在对应 feature 里决定何时触发、候选怎么筛、怎么批、最多等多久、迟到结果怎么处理；登记 manifest（中英文标题与摘要，测试校验归属）；缺省 shadow；用 `MockJudgeProvider` 写测试；在 `features/` 写设计说明并列出「哪些没在真实 Jev 上跑过」。在 `mu.json` 里改配置不能凭空加一道新题。

### 效果证据（作者回测）

`08-jev-retrospective.md` 累计 943 条判定记录（直接 Jev 911 条），结论原话：「Jev 已有局部正向作用，但当前记录不足以证明整体价值很大或净收益为正。」分项：

- 工具输出准入：194 批、2,412 块，一块未省略，却占已记录 Jev 输入约 54.0%（1,966,585 tokens）；
- 技能披露：确有隐藏说明、减少上下文暴露，未证明隐藏正确或任务质量提高；
- 蜂群：发布门 662 个候选通过 89 个（约 13.4%），89 条中 40 条送达至少一只蜂；筛选率不是准确率，未证明改变了任务结果；
- 浏览器：直接 Jev 14 次，2 次完成、10 次受阻、2 次执行异常；
- 经验捕获 6 次全部跳过；漂移监测有动作、收益未闭环。

两处有标注或对照的：`packages/kyrn-judge/src/decisions/hive.ts:105–112`（常量 `ACROSS_BEES_SUPERSEDES` 的注释）记录 2026-09-24 用 11 次真实运行里 153 对人工标注笔记校准跨成员「取代」——0.6 门槛下判了 6 次取代、全错，最高置信 0.87，于是跨成员取代的门槛升到 0.9；同一成员修正自己的笔记仍用 0.6（那一类 12 次判对 5 次）；`09-test-log-admission.md` 的测试日志对照里，真实回放时判断器没产出候选，收益来自一条确定性规则（重复段换回指标记，7 份真实长日志净省 51.0%）。35 个点里有标注集的只有 `input.preflight` 与测试日志两处，其余阈值未校准。

### 上游与 Jev

Jev 是 TypeSafe 的托管判断模型（输入 0.042 美元每百万 token、输出免费），正文要发到第三方。Pi 在 v0.99.0 把分类模型做成原生能力：`Models.classify()` 采用与提供方无关的 choice / score / bool 契约，Jev 经 TypeSafe、OpenRouter、Cloudflare Workers AI、Vercel AI Gateway、OpenCode Zen 提供（[CHANGELOG](https://github.com/earendil-works/pi/blob/4b060d3a98618019adb9985d517516c8e99a2bbe/packages/ai/CHANGELOG.md)）。同一版本还加了 [`llama-cpp-classify`](https://github.com/earendil-works/pi/blob/4b060d3a98618019adb9985d517516c8e99a2bbe/packages/ai/src/api/llama-cpp-classify.ts)：用本机 llama-server 上的普通对话模型答同一套题，把选项映射为单 token 标签，取下一个 token 的概率做 softmax，不生成文字。三家结论都没提到这条本地路径；它未经本条实测。mu 钉在 Pi 0.87.1，尚未合入这些上游能力。

### 社区

三家对社区的读数不一致。主笔的调研环境访问 GitHub 接口与网页返回 403；#299 的审查者（DeepSeek、Qwen）2026-10-02 用 `gh api` 核到：339 star、32 fork、1 watcher，issue 全状态 3 个、全部关闭（#5 Windows 首次启动报错、#4 Windows 默认安装路径在空格处被截断、#3 上手向导提案），PR 全状态 2 个、均关闭、同一作者（#2 设置里可编辑思考档位、#1 允许内网 HTTP base URL）。npm 下载数未复核。

- GPT 读到了 LINUX DO 发布帖的回复（2026-09-23 至 29）：「人话看板」有明确好评；多人希望以插件形式用在已有 Pi、Claude Code、Codex 习惯里，而不是换整套环境；有人嫌配置多；有人对判断可靠性观望；作者 9 月 27 日回复过修复。
- Claude 当时被出口代理拦住、未读到该帖；记 339 星、32 fork、npm 累计 837 次下载、3 个 issue 均为 Windows 安装问题（星与 fork 核对成立；3 个 issue 中 #3 是上手向导提案，不是安装问题）；Theo Browne 的批评与「社区复测准确率约 73.8%」只见搜索摘要，未读原文。
- Grok 记 5 个 issue 全部关闭、都是桌面端问题，并称 linux.do 没有可核对的长帖（5 是把 2 个 PR 算进了 issue）。

三家一致的判断：这是值得研究的架构实验，尚无独立的长期使用证据；可依赖的评价主要是作者自己的回测。

### 三家结论的分歧与核对

| 说法 | 出处 | 核对 |
| --- | --- | --- |
| `.memo/design/p2-control-20260906/06-small-brain-engine-research.md` 拉取 404，TODO 只是指针 | Grok | 404 的说法不成立：该任务书在 main 上，体验目录 `open-questions.md` 也指向它。但它据此指出的「TODO 仍是指针」成立：按任务书产出的 [#290](https://github.com/yesme/hctl2/pull/290) 未合入 |
| shadow 只在权限审批有例外；hctl2 应把 shadow 的无副作用保证放在最终动作准入处，不靠引擎返回值的约定 | GPT | 前提不全：另有蜂群路由、蜂群分派与压缩打分，见上文表格。结论因此更成立：四处调用方直接读 `judged`，shadow 无副作用靠的是每个调用方自律，不是一道闸门 |
| 提供方不给概率分布时，部分选择题策略跳过概率门槛；换后端后「接口相同」不保证「置信度策略相同」 | GPT | 成立：`policy.ts:45, 57`。对本机 `llama-cpp-classify` 路径直接相关，它的概率来自标签 token 的 softmax |
| 任务帧仍是替身：goal 还不是 writer 维护的帧，题目没有锚 | Grok | 在 `8dfebe3` 上前半不成立：任务帧 writer 缺省开启并已接入（见上文），mu 的 `kyrn/docs/04-injection-points.md` §11 仍列为未接，是文档滞后。后半有作者背书：PRD §12 把「任务状态准确」列为产品化缺口 |
| 任务书里 B 的缺口边界前后不一 | GPT | 成立：§二写「上次收到信息以来」，§四样本写「上次发言到这次被点名」 |
| 本地判断器做不了关系题，本地档只能另找小生成模型 | Claude | Laya 一档源码属实；Pi v0.99.0 的 `llama-cpp-classify` 已提供「本地对话模型按标签概率答题」的现成路径，三家均未提及。#290 的 `models/llama-cpp.md` 写的是「不提供只准选这些选项的约束」，只能在提示里写死选项再校验输出；Pi 的做法是读 llama-server 对标签 token 的概率，选项约束与概率一并得到 |
| issue 数（3 个或 5 个）、LINUX DO 帖能否读到 | Claude、Grok、GPT | issue 数已由审查者核到：3 个，Grok 把 2 个 PR 算了进去，见[社区](#社区)；LINUX DO 帖未复核 |

### 对 HCTL2 的位置（只记录，不裁决）

**三家都同意的：** 借的是机制与纪律，不是 35 道题——那 35 个点几乎全是单 agent 回合内的判断（上下文、工具、子代理），没有一道是「开不开新房间、建不建 Task、收不收敛已有 Task」。判断器只出建议或选材，治理动作仍走现有预览、采纳与人的确认；fallback 等于现行机械行为；先 shadow；事件触发，不按固定频率盘问。另有「来源指针由组装器给出，模型只能选」由 GPT 明确提出，Claude 指出 HCTL2 前情提要已采同一方向。

**与已有位置的对应：** v0.18.12 的 small-brain 三档（不配只挑不写，配了才分节，能生成才改写）与 mu 的「判断与写字分开」是同一判断；任务书 06 的 A（持续建议开 Topic）、B（@ 时补缺口）、C（前情提要起草）是三家共同对上的落点；[#290](https://github.com/yesme/hctl2/pull/290)（Grok，未合入）的结论是「缺省机械规则，模型只留形状」，卡在样本不够。

**三家提出的 Room 判断点候选**，按 GPT 提出的三个轴归并：

| 轴 | GPT | Grok | Claude |
| --- | --- | --- | --- |
| 讨论组织：值不值得另开 Topic Room | `room.topic.suggest`（先判状态变化，再判值不值得隔离） | `room.drift`、`room.fork`、`frame.kind` | `room.topic-shift` |
| 承诺：要不要形成或修订 Task | `task.proposal.ready`、`task.proposal.relate` | `task.crystallize`、`task.converge` | `room.commitment`、`room.converge` |
| 证据与决定：能不能收敛 | `room.frontier.check` | —— | —— |
| 上下文选材（B、C） | `context.gap.select`、`room.preface.classify` | —— | `room.gap-relevance`、`topic.brief-section` |
| 注意力投影 | —— | —— | `room.needs-you` |

**分歧：**

- 先做哪个：GPT 先做 B、C 的只挑不写，A 只跑 shadow 评提示质量，Task 关系判断最后；Claude 先做 `room.topic-shift` 与 `topic.brief-section`；Grok 五题一起在后台跑 shadow，只出建议卡，看采纳率。
- 标注从哪来：Claude 与 Grok 主张让 shadow 记录在产品里自己攒标注（人手动开 Topic 即正例，驳回即负例），以绕开 #290 的样本瓶颈；GPT 主张沿用任务书的样本计划并补动作级指标（候选召回、误报漏报弃权、策略、打扰次数、陈旧结果、总成本），不因 mu 有吸引力跳过对照。
- 「收敛」：GPT 拆成四件（问题答清、有权的人已决定、需要新 Task Revision、证据足以完成精确版本），反对任何「完成度分数」；Grok 与 Claude 用一道选择题（相同 / 更精确 / 矛盾 / 无关，或收窄 / 完成 / 否定）。三家都排除自动完成与自动取代。

**待与所有者讨论（本条不定）：** Room 阶段的判断要不要按 mu 这套机制走（决策点规格、三区间加逃生选项、shadow 先行、判定入记录）；托管判断器与本地判断器各自的策略点缺省；shadow 攒标注与任务书样本计划的关系；任务书 B 的缺口边界以哪个为准。

### 复用决策

**暂缓。** 本轮只记录；mu 的产品形态（Pi 分叉加 Electron 壳）不进 HCTL2 依赖树，判断机制是否「仅参考行为」待上述讨论后在复核记录里补定。

## 复核记录

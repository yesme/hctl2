# ponytail：一份「懒惰资深工程师」提示词纪律，怎么让模型少写代码

> 类别：⑧ Harness 技能包 · 证据编号：E-SKILL-PONYTAIL<br>
> 状态：调研 · 日期：2026-09-11；发布后正文不改，只在文末追加复核记录<br>
> 总览与复用决策用语见 [docs/research/README.md](./README.md)。

所有者在别处看到一句话：有个叫 ponytail 的 Skill 能大幅减少模型写代码时的「膨胀病」——越写越多、越写越绕、抽象层叠床架屋。本文回答四件事：它是什么、靠什么起作用、别人测出来到底有多大用、对 HCTL2 的 Agency 与 Participant 模板意味着什么。全文把「文档说」（README、SKILL.md、作者的 benchmark 写作）与「源码里看到」（钉在具体 commit 的 hook 与脚本）分开标注；第三方评测单独归为「别人测的」。

## 对象与基线

**是什么。** [DietrichGebert/ponytail](https://github.com/DietrichGebert/ponytail) 是一个面向编码 agent 的技能包：一份主 SKILL.md（把模型设定为「见过一切、只写一行的懒惰资深工程师」，附一张七级决策梯子），五个配套子 Skill（`ponytail-review`、`ponytail-audit`、`ponytail-debt`、`ponytail-gain`、`ponytail-help`），加上一组把这份文本塞进各家 harness 上下文的适配层（Claude Code / Codex 的三个生命周期 hook、OpenCode 插件、Gemini 扩展、Pi 扩展、Hermes 插件、一个 MCP 服务器，以及给 Cursor / Windsurf / Cline / Kiro / Qoder 等复制用的规则文件）。README 自称覆盖 20 个 agent。

**同名对象排查。** GitHub 上名叫 ponytail 的仓库不止一个：`linsomniac/ponytail` 是 Python 的日志 `tail -F` 库，`i17c/PonyTail` 是 IDEA 的日志跟踪插件，`coldbricks/paisley-ponytail` 是 Webshots 老照片找回工具，都与代码膨胀无关。所有者说的只能是 DietrichGebert 这一个——它的描述就是「Makes your AI agent think like the laziest senior dev in the room. The best code is the code you never wrote」，主题标签含 `yagni`、`claude-code-plugin`。此外有一批衍生品：`ilindaniel/ponytail-lite`（只留一份 AGENTS.md，去掉插件层）、`anshaneja5/scalpel`（自称在 ponytail 自己的 benchmark 上全面胜出）、`decapostos/lean-code`（去掉 `ponytail:` 水印注释的派生）、多个 DeepSeek Harness / Kimi / Pi 移植，以及把 ponytail 与 caveman、superpowers 等打包的组合包。还有一个已被 issue 点名的木马克隆（`0xwilliamortiz/ponytail-improved`，通过改名的 gup.exe 侧载 DLL 传播恶意软件），说明它已经红到被人拿去当供应链诱饵。

**谁做的。** 作者 GitHub 账号 DietrichGebert，2023 年注册，个人资料没有公司、简介与所在地；package.json 的作者字段写 Dietrich Gebert。项目主页 ponytail.dev，README 顶部挂着「Something's coming, join the waitlist」的横幅和一个赞助商（GreenPT），已建成案例栏放着一个第三方产品（Retriever）。推断：个人项目，正在走向商业化，方向未公开。

**钉定版本。**

| 项 | 值 |
| --- | --- |
| 最新 release | v4.9.0，2026-08-07，commit `0a4dd63ad4541f4f655c4108a295916f3c1d8fda`；npm `@dietrichgebert/ponytail` 同版本 |
| 本文读的源码 | main @ `356918eba965ee1eac64bd3a7f0dd02108350de5`（2026-09-07），领先 v4.9.0 十五个提交，都是文档与 logo 改动 |
| 许可证 | MIT（LICENSE 在仓库根） |
| 创建 | 2026-06-12；同一天发 v1.0.0 与 v4.0.0，两周内发到 v4.7.0，此后节奏放缓 |
| 热度 | 134,451 star、7,193 fork（GitHub API，2026-09-10）；聚合站写 132.6k |
| 活跃度 | 约 222 个提交，头号贡献者 DietrichGebert 独占 112 个，其余贡献者最多十个；88 个开放 issue、213 个已关闭、536 个 PR |
| 体量 | 约 150 个受跟踪文件，多数是 markdown；主 SKILL.md 六千余字节，五个子 Skill 各一到三千字节；hook 目录七个 JS 文件合计约三万字节；benchmark 目录是几个 Python / JS 脚本与九份结果写作 |

一句话定性：一个人在三个月里做出来、以传播速度而非工程深度出名的提示词纪律包，附带一层为了「常驻」而写的 hook 胶水，以及一套比多数同类项目认真的自测 benchmark。

## 原理

它抑制膨胀的机制只有一种：**改变模型动笔前的搜索顺序**，其余全部是围绕这份文本的投递与开关。下面按「哪部分是提示词、哪部分是代码、代码到底做了什么」拆开。

### 提示词部分（SKILL.md，文档说）

主 SKILL.md 的骨架是五块：

1. **人设**：「You are a lazy senior developer. Lazy means efficient, not careless. ... The best code is the code never written.」
2. **梯子**：动笔前从第一级往下走，停在第一个成立的台阶——

   > 1. Does this need to exist at all? Speculative need = skip it, say so in one line. (YAGNI)
   > 2. Already in this codebase? A helper, util, type, or pattern that already lives here → reuse it. Look before you write.
   > 3. Stdlib does it? Use it.
   > 4. Native platform feature covers it? `<input type="date">` over a picker lib, CSS over JS, DB constraint over app code.
   > 5. Already-installed dependency solves it? Use it. Never add a new one for what a few lines can do.
   > 6. Can it be one line? One line.
   > 7. Only then: the minimum code that works.

   梯子后面紧跟一条 2026-06-22 才加进去的「先理解再偷懒」护栏：「The ladder is a reflex, not a research project — but it runs *after* you understand the problem, not instead of it」，以及一条操作性指令：「**Bug fix = root cause, not symptom.** ... Before you edit, grep every caller of the function you're about to touch. ... one guard in the shared function is a smaller diff than a guard in every caller」。
3. **规则**：不写没人要的抽象（单实现的 interface、单产品的 factory、永不变的配置项）、不为「以后」搭脚手架、删优于加、无聊优于聪明、文件最少、diff 最短；复杂请求就「先交最小版，同一条回复里反问要不要完整版」；有意识削掉的角落必须用 `ponytail:` 注释标出**上限**与**升级路径**（例：`# ponytail: global lock, per-account locks if throughput matters`）。
4. **输出格式**：代码先行，解释最多三短行——「skipped: [X], add when [Y]」；「If the explanation is longer than the code, delete the explanation」。
5. **不许偷懒的清单**：信任边界上的输入校验、防数据丢失的错误处理、安全措施、无障碍基础、用户明确要的东西；硬件校准旋钮；非平凡逻辑必须留下一个可运行的最小检查（一个 `assert` 自检或一个小测试文件，不上框架）。

另有三档强度 lite / full / ultra。文档说 lite 是「照做，但用一行点出更懒的替代方案」，full 是「梯子生效」，ultra 是「YAGNI 极端派，先删再说，一句话交一行代码同时质疑需求」。

子 Skill 全是同一路数的短提示词：`ponytail-review` 只审 diff 里的过度工程，每条发现一行，格式固定为 `L<line>: <tag> <what>. <replacement>.`，五个标签（`delete:` / `stdlib:` / `native:` / `yagni:` / `shrink:`），结尾报 `net: -<N> lines possible.`，明确把正确性、安全、性能划到范围外；`ponytail-audit` 是全仓库版；`ponytail-debt` 让模型 `grep -rnE '(#|//) ?ponytail:' .` 把所有削角注释收成一张台账，没写升级触发条件的标 `no-trigger`；`ponytail-gain` 只是把 benchmark 的中位数印出来。

### 代码部分（hooks/，源码里看到）

Claude Code 与 Codex 路径挂三个 hook（`hooks/claude-codex-hooks.json`）：

- **SessionStart**（匹配 startup / resume / clear / compact）→ `ponytail-activate.js`：把当前强度写进配置目录下的状态文件 `.ponytail-active`；读 `skills/ponytail/SKILL.md`，剥掉 front-matter，按强度过滤后整段作为 additionalContext 打到 stdout；若用户 settings.json 里没有 statusLine，再追加一段「STATUSLINE SETUP NEEDED ... Proactively offer to set this up for the user on first interaction」，让模型主动向用户提议替其改写 settings.json。
- **SubagentStart** → `ponytail-subagent.js`：状态文件说开着，就把同一段文本以 `hookSpecificOutput.additionalContext` 注进每个子代理。注释写明原因：SessionStart 的上下文到不了子代理（issue #252），不补这一刀，「干活最多的那些子代理全是 ponytail-unaware」。可用环境变量 `PONYTAIL_SUBAGENT_MATCHER` 按 agent_type 正则限定，解析失败一律「fail open」（照注）。
- **UserPromptSubmit** → `ponytail-mode-tracker.js`：只做开关。识别 `/ponytail lite|full|ultra|off`、`/ponytail default <mode>`、整句 `stop ponytail` / `normal mode`，写或删状态文件并回一行「PONYTAIL MODE CHANGED」。在 Claude Code 路径上它**不**逐轮重注规则；只有 Qoder（没有 SessionStart 事件）才在这里每轮重注。

三处值得单独指出的源码事实：

- **强度差异极小。** `ponytail-instructions.js` 的 `filterSkillBodyForMode` 只做一件事：把强度表里不属于当前档的行、以及例子里 `- lite: "..."` 这类不属于当前档的句子删掉，其余原文保留。也就是说 lite / full / ultra 之间的全部差别，是那张表里对应的一行和例子里对应的一句；「三档强度」本身也只是提示词。
- **hook 从不拦截、从不度量。** 三个 hook 全部「best-effort, never block」：一秒超时、出错静默、stdout 关闭也吞掉。没有 PreToolUse、没有对产出 diff 的任何检查、没有行数阈值、没有依赖清单比对。代码层负责的只是「这段文本每次会话、每个子代理都在」和「一个开关标志」；「少写代码」这件事本身零机械保障。
- **作者明确拒绝把纪律工具化。** issue #217 提出「梯子少了一级：这个仓库里是不是已经写过了」并顺带推荐自己的重复代码检测工具，作者接受加一级，但回绝了工具：「ponytail will not take on a tool or dependency to enforce a habit the prompt can state in one sentence. The rung stays prose.」这是它的设计立场，不是疏漏。

其他 host 的投递方式同理：OpenCode 插件与 Hermes 插件在每轮 LLM 调用前注入，Gemini 扩展作为常驻上下文加载，Cursor / Windsurf / Cline 等只是规则文件，MCP 服务器给没有注入原语的 host 用。`docs/agent-portability.md` 列了文件到 host 的映射；`scripts/check-rule-copies.js` 在测试里保证各份拷贝的规则文本一致。

### 它为什么有效（把两边证据合起来看）

把作者与第三方的数据放在一起，能看出效果集中在一种情形：**任务存在「过度建造陷阱」**——基线 agent 会手写一个组件（日期选择器、颜色选择器、文件拖放区），而梯子第四级把模型的注意力先引到原生 `<input type="date">` / `type="color"` / `type="file"`。作者 benchmark 里这三题分别减少约 94%、92%、62% 的行数；后端 CRUD 端点这种「本来就没什么可省」的题四臂几乎相同。JetBrains 的 80 题多数是数据处理类，测出的中位减幅只有 15%，且「效果集中在基线过度建造的题上，本来就精简的题接近零」，与作者的分布描述一致。

另一处直接证据是 #245 的修复实验：作者先用一句劝诫式散文「trace the flow end to end」，三次零命中；换成操作性指令「grep every caller of the function you touch, fix the shared function once」，Sonnet 4.6 与 Opus 4.8 上从六分之一变成六分之六。同一份 Skill、同一个模型、只换了「叫模型去看什么」，效果天差地别。这说明起作用的不是人设与态度形容词，而是**它让模型在动笔前多取了哪些信息、先看哪里**。

## 证据：对照实验与失效案例

### 作者自测（文档说；方法与数据在仓库 `benchmarks/`）

- **单发 benchmark（2026-06-13）**：五个小任务、三臂（无 Skill / caveman / ponytail）、三个 Claude 模型、每格十次。报 80–94% 更少代码、42–75% 更省、3–6 倍更快。Colin Eberhardt 在 issue #126 指出基线没有 harness 式 system prompt、像聊天机器人一样给多个方案带评论，行数被灌水；作者当天承认「Not a fair fight」。README 现已把这组数字折叠到「Older single-shot numbers」里并注明失真。
- **agentic benchmark（2026-06-18，主打数字的来源）**：Claude Code 2.1.177 无头模式，Haiku 4.5，在钉定 commit 的 fastapi/full-stack-fastapi-template 上跑 12 个功能票，四臂（基线 / ponytail 插件 / caveman / Eberhardt 的七个词「Follow YAGNI principles, and prefer one-liner solutions」），每格四次，以 `git diff` 新增行数计。均值：ponytail −54% 行、−22% token、−20% 成本、−27% 时间；另有六道以对抗输入实际执行的安全题，ponytail 与基线 20/20 安全，七个词的提示 19/20（一次路径穿越逃出目录，少写的那三行正是校验）。自陈局限：只一个模型、n=4、安全只是「不掉已知护栏」的下限、四格因 Windows 超时被强杀。
- **一次被撤回的结果**：2026-06-17 那版只测出约 4% 的差距，作者发现是插件的 SessionStart hook 在基线臂上也触发了——基线在偷偷跑 ponytail——于是隔离后重跑。这条自曝值得记一笔：**hook 型「常驻」纪律很容易在评测中污染对照臂**。
- **理解优先修复验证（2026-06-22）**：为 #245 造的 `trace-transfer` 题（bug 报告只点名 transfer，真因在 transfer 与 withdraw 共用的 `_debit`），Sonnet 4.6 / Opus 4.8 基线六分之一、修复后六分之六；Haiku 两臂皆零，作者定性为模型能力上限。为 #217 造的两道「仓库里已有 helper」题，基线与 ponytail 都复用了，重复失败没有复现，新加的第二级「正确但收益未证」。
- **小模型迁移差**：本地 llama3.2 上多级决策梯子不被可靠遵循；README 还提到「a terse reasoning model that spends thinking tokens deliberating the rungs can go the other way (on GPT-5.5 it does)」——省 token 的结论对边琢磨边写的推理模型可能反转。

### 别人测的

- **JetBrains（Denis Shiryaev，2026-07）**：SkillsBench 80 道配对任务、Harbor 0.18 沙箱与校验器、Claude Code 2.1.201、claude-sonnet-5 中等推理、ponytail v4.8.4（commit 16f2980）、251 次计费试验。结果：代码 −15.4%（广告 −54%）、成本 −10.3%（p=0.004，bootstrap 区间刚触零）、时间 −11%；质量 9 题略差、6 题略好、65 题相同，「a null result, not a clean bill of health」。三条对 HCTL2 特别有用的观察：不用 hook 强注入、只装 Skill 让模型自取，十次会话里**自激活零次**；`ponytail:` 注释约定 80 次试验里只被遵守一次；10 题的预热跑出了相反符号（更贵、质量塌），样本小会得出完全相反的结论。
- **KuldeepB19 / Deepusleepy 的 v3 benchmark（2026-06 至 07）**：Opus 4.8，24 个任务 × 四臂（无插件 / lite / full / ultra）× 五次 = 480 次构建，以真实插件运行并从状态文件核验激活（不信模型自报），按解析器统计逻辑语句数，正确性跑隐藏测试、安全跑真实攻击、健壮性探未言明的边界。结论：full 档少写约 44% 代码（语句数少 53%），正确性两边都 99%，安全无差；**代价在健壮性**——24 题里有 5 题，一旦题目没写明边界情况，ponytail 会把防御性处理削掉，坏输入下的通过率从接近满分掉到接近零，而且**强度越高越脆**。这份评测的作者自评：这是此前所有只数行数、靠读代码判质量的评测都没抓到的成本。
- **RicardoCostaGit（Cursor SDK 多轮，2026-06-16）**：产出更精简，但在大型、必须做完的任务上过程成本更高（更多工具调用与 token）；省钱落在容易被卡住、容易雪球的任务上。
- **Scott Logic（Colin Eberhardt，2026-06-16）**：批评它是「一份约百行 markdown 重新包装 YAGNI」，七个词就能在其单发 benchmark 上以更少行数、同样 100% 正确赢过它；模型本来就懂 YAGNI，Skill 只是把已知常识说出来。作者接受了方法批评并重建了 benchmark，但重建后的 agentic 数据也反过来显示：七个词的提示在题间「erratic」，且是唯一掉过安全护栏的臂。
- **dev.to（yashddesai，2026-06-25）**提出一个至今没人回答的问题：梯子把「原生平台特性」排在「已装依赖」之前，在有成熟设计系统（shadcn/ui 之类）的仓库里会不会用裸 `<input>` 破坏组件库约定？所有 benchmark 都在没有设计系统的模板仓库上跑，这一情形没测过。

### issue 里的失效案例

- **#245「Dangerously lazy」**（已关）：Sonnet 4.6 在「最短 diff 赢」反射下扫到最近的可疑处就改，没有端到端追信号流，交了一个自信的错修。模型自己的复盘原话被作者写进了 Skill：「laziness that skips understanding and dresses up as efficiency」。
- **#745**（开放）：修好 #245 之后新加的「Read fully, then be lazy」在大 markdown 文件上把 agent 卡死在半途——每轮重注同一条常驻指令，agent 反复尝试吞下整份几千行的文件。一条护栏修一种失效，制造另一种。
- **#584**（开放）：在 Claude Code 里 `/ponytail` 走的是 Skill 分发，UserPromptSubmit 拿到的 prompt 是整段 SKILL.md 正文而非命令文本，正则 `^[/@$]ponytail` 永不匹配，模式从未被设置。更早的 #161 是同一根因的另一面：旧的停用正则匹配到了 SKILL.md 自己那句「Off only: "stop ponytail" / "normal mode"」，**Skill 读到自己就把自己关了**。
- **#332**（开放）：与 caveman 同装时两边都逐轮注入，模型收到互相冲突的风格指令；caveman 已做了检测让步，ponytail 未回应。
- **#406 / #419 / #423**（已关）：作者精选的 `examples/` 里，被略去的错误处理没出现在「Skipped」段、削角处没打 `ponytail:` 注释——「说清你跳过了什么」这条规则连示范样本都没稳定做到。
- **#735**：木马克隆传播恶意软件，见上文。
- **#252**（已关）：规则到不了子代理，补了 SubagentStart hook；投递面的每一处遗漏都需要再加一个 hook。

## 优缺点

| | 优点 | 缺点 |
| --- | --- | --- |
| 效果 | 在「有过度建造陷阱」的任务上减幅巨大（原生控件替代手写组件时 60–94%），且独立评测（JetBrains、480 次执行评分）都确认代码量与成本的下降是真实且统计上站得住的 | 平均减幅远小于宣传：JetBrains 测得 −15% 代码、−10% 成本；本来精简的任务接近零；对边琢磨边写的推理模型省 token 可能反转 |
| 安全与质量 | 作者与独立评测都没测出正确性下降；对抗测试里没掉已知安全护栏，比裸「写一行」提示稳 | 未言明的边界情况会被削掉：24 题里 5 题健壮性从近满分掉到近零，且强度越高越脆；「never simplify away validation」这类护栏靴子是散文，靠模型自觉 |
| 机制 | 决策梯子把「先找已有、先找 stdlib、先找原生」变成动笔前的固定动作，是对模型搜索顺序的实质改变；#245 的 A/B 直接证明操作性指令胜过态度散文 | 零机械保障：hook 只投递文本与记开关，不拦截、不度量、不比对；作者明确拒绝工具化 |
| 投递 | 覆盖面广（20 个 host），把「上下文到不了子代理」「compact 后丢失」这些坑都踩过并补上 | 投递层本身脆弱且有副作用：模式开关在 Claude Code 上从未生效（#584）、Skill 读到自己会自关（#161）、常驻规则与其他常驻插件冲突（#332）、hook 会让模型主动提议替用户改写 settings.json；不靠 hook 强注入时模型十次里零次自取 |
| 可审计 | `ponytail:` 注释约定 + `ponytail-debt` 台账，让削掉的角落留痕；`ponytail-review` 的一行一发现格式（位置、标签、替代物）干净可读 | 约定遵守率极低（80 次里一次）；子 Skill 的「一行一发现」仍是模型自报，没有工具读 diff 核对 |
| 项目 | MIT；benchmark 比同类认真，肯自曝污染 bug、肯撤回数字 | 单人项目、三个月内从零到 13 万星、主页挂着商业化候补名单；已有木马克隆；含脚本的插件是供应链输入 |

## 对 HCTL2 的意义

先把 HCTL2 这边的坐标摆出来。[Participant 与 Terminal](../design/participant.md#专业化-participant告别-byoa)刚裁定：Skill 改变的是参与者去拿什么信息、关注什么方面，也就是它的上下文；人设是交流与界面属性，对 Run 里干活的 worker 型参与者没有帮助；「Skill 给方法，控制面给门」。[七件事分层](../design/participant.md#七件事分层)里 Skill 与执行者配置（Worker Profile）是 Agency 供给的两层，人设是控制面存储的一层。[Skill 与申报](../design/spec/participant.md#skill-与申报)规定 Skill 带 revision 与 digest、三态（declared / available / activated）、required 缺失则不激活，且「含脚本的 Skill 是代码供应链输入」。本地 Agency 参考实现（[src/agency/README.md](../../src/agency/README.md)）把 Skill 分「人发起」与「模型可自取」两种触发，判据是「模型能不能有意义地自己伸手拿它」。

### 一问：它是「人发起」还是「模型可自取」

按判据，ponytail 的核心梯子是**阶段内纪律**，理论上属于「模型可自取」——它不切换阶段，是施工时时刻刻该做的事。但 JetBrains 的实测把这个分类击穿了：只装 Skill 不强注入，模型十次会话零次自取。原因不难懂：一个正在过度建造的模型并不知道自己在过度建造，它没有理由伸手去拿一份「别过度建造」的方法。作者自己的解法是 hook 常驻——这是我们两分法之外的第三种触发：**常驻纪律**，不由人按也不由模型取，而是装载时就绑在执行体上。

对 HCTL2 的结论：这类 Skill 应作为 **Worker Profile 的默认 required Skill** 在执行规格里冻结并装载，不能放进「模型可自取」桶里指望模型自己拿。子 Skill 里的 `ponytail-review` / `ponytail-audit` 则是典型的**人发起**：由评审席位对一个封存的 diff 或仓库跑一次。`ponytail-debt` 不是 Skill，是工具箱动作（下文）。

### 二问：它改变的是模型获取的信息与关注点，还是人设式叮嘱

两种都有，要拆开：

- **人设部分**（长马尾、椭圆眼镜、什么都不说、看着你）：按所有者裁定，对 worker 无用，是界面属性。它对传播有用，对工程没用；不进 Agency。
- **梯子**：这是对模型**搜索顺序**的规定——动笔前先看这个仓库里有没有、stdlib 有没有、平台原生有没有、已装依赖有没有。这四步每一步都是「去取一份信息再决定」，正是所有者说的「改变参与者去拿什么信息、关注什么方面」。作者 benchmark 里减幅最大的题，减掉的全是「本来会手写、现在先看了一眼原生控件」的差额。
- **操作性指令**（「grep every caller of the function you touch」）：纯信息获取步骤，#245 的 A/B 是这条裁定最干净的实验证据——同一段态度散文零命中，换成「去看哪里」六分之六。
- **输出格式**（代码先行、三短行、`skipped: X, add when Y`）：不改变信息获取，改变的是交付形状；对 HCTL2 有用的部分是「说清跳过了什么」，但实测遵守率低，不能当证据来源。
- **护栏散文**（「never simplify away validation」）：最像叮嘱，也是实测最不可靠的部分（5/24 健壮性塌陷）。

所以值得借的是梯子的第二到第五级与操作性指令，不值得借的是人设与护栏散文。

### 三问：放在哪一层

- **塑形 Skill？不放。** 塑形阶段不写代码，且梯子第一级「这需要存在吗」在 HCTL2 里已经有更硬的家：[hctl2-shaping](../../src/agency/skills/hctl2-shaping/SKILL.md) 的「出界」清单与契约冻结的「不做什么」。YAGNI 在我们这里是契约条款，不是施工时的反射；把它再写进施工 Skill 只是重复，而且是弱形式的重复。
- **施工 worker 的 Worker Profile 默认 Skill？放，但改写。** 一份 HCTL2 自己的施工纪律 Skill，保留梯子第二到第五级（先找仓库已有、stdlib、平台原生、已装依赖）、「先读全再动手」「改共享函数不改每个调用方」这类操作性指令、以及削角必须留注释的约定；去掉人设、三档强度（源码证明它们只差一行）、输出格式里的「解释比代码长就删解释」（与我们要求的 Result Proposal 与 Evidence 冲突）。作为 required Skill 冻结进 Execution Spec，工具箱回读 digest 记 known。
- **评审席位的方法论 Skill？放，作为一条轴。** `ponytail-review` 的边界划得很好：只审过度工程，正确性、安全、性能显式出界。它可以成为 [hctl2-design-review](../../src/agency/skills/hctl2-design-review/SKILL.md) 同款结构里的一条「过度工程轴」，或代码评审 Skill 的一个席位方法：读真 diff，一行一发现，五个标签，结尾报可删行数。但它不能是唯一的评审席位——它自己说了不看正确性与安全，而 480 次评测恰恰显示膨胀纪律的代价落在健壮性上，所以过度工程轴必须与正确性 / 安全轴分席位、分 Skill，才算两票。
- **人设层？不放。** 上文已述。

### 四问：与「能用代码管的不用提示词管」怎么对上

ponytail 的立场与我们相反——它把能一句话说清的习惯一律留在提示词里。按我们的原则逐项看它有哪些能机械化：

| ponytail 里的做法 | 机械化落点 | 说明 |
| --- | --- | --- |
| hook 在 SessionStart / SubagentStart 注入规则文本 | 本地 Agency 参考实现拉起前注入 harness 配置（同[harness 钩子调研](./harness-hooks-20260903.md)的路径） | 投递方式我们已有；差别是 HCTL2 把 Skill 引用与 digest 冻结在 Execution Spec 里，不靠 `~/.claude/.ponytail-active` 这种宿主目录里的状态文件；「强度」不是开关，是不同 Skill revision |
| `ponytail:` 削角注释 + `ponytail-debt` 让模型 grep 收台账 | 工具箱检查器：扫 ChangeSet 新增行里的削角标记，缺上限或缺升级触发条件的直接判失败，结果作为 Evidence 进评审包 | 这是全包里最值得机械化的一条：约定本身有价值，遵守率靠模型只有 1/80；改成检查器后「留痕」才成立。标记词换成我们自己的 |
| 梯子第五级「不为几行代码新加依赖」 | 我们已有同形机制：PR 改动三方依赖时 CI 的 `PR contract` 要求调研节引用 `docs/research/`；对 ChangeSet 同理——lockfile / manifest 新增条目触发评审清单上「为什么不是 stdlib / 原生 / 已装依赖」一问 | 机械检测、人判断；不由提示词管 |
| 梯子第二级「仓库里已经有了」 | 现有重复代码检测器作为评审 Evidence（作者拒绝的正是这类工具） | 是否引入、引哪一个，另开调研文件，本文不决定 |
| 「非平凡逻辑留一个可运行检查」 | Run 里跑测试是 Gate 的机械事实，不是 Skill 条款 | 已覆盖 |
| 「never simplify away validation」 | 契约的验收约束里写对抗用例；Gate 跑它 | 提示词护栏实测会塌，验收约束不会 |
| 行数、文件数、新增依赖数 | 评审包里的观测字段，**不当门** | 行数可被格式化与注释操纵，480 次评测用解析器数语句就是为了防这个；即便如此也只宜作 Evidence |

一条反向教训也要记：ponytail 的「一次会话、一个开关标志、模型可随口关掉（`normal mode`）」与我们「约束不能住在被约束方能写的地方」直接冲突。HCTL2 里 Skill 是否装载由 Execution Spec 冻结，执行体在会话里说什么都改不了；这不是我们要补的，是我们已经有的。

## 采用结论

**仅参考行为。** 理由与边界：

- **不采用二进制 / SDK。** 它没有二进制；npm 包只是把 hook 与 SKILL.md 打包，hook 是 Claude Code / Codex 形状的、写宿主目录状态文件、会让模型提议改写用户的 settings.json，与「执行规格冻结、宿主配置由参考实现在拉起前注入」的做法冲突。含脚本的 Skill 是供应链输入，且已有木马克隆先例。
- **不移植组件。** 值得移植的代码为零：hook 逻辑我们不需要，`ponytail-debt` 的 grep 一句话我们要重写成检查器。
- **不适配协议。** 没有协议；`ponytail:` 注释约定的形状（上限 + 升级路径）可借，但标记词与检查规则是我们自己的。
- **借行为。** 借三样：梯子第二到第五级作为施工 Skill 的搜索顺序；「grep 全部调用方、改共享函数」这类操作性指令的写法（写「去看哪里」，不写「要认真」）；`ponytail-review` 的一行一发现格式与「只审过度工程、其余出界」的席位边界。若落成 Agency 目录里的 Skill，按 hctl2-shaping 的先例改编文本并随目录附 MIT 许可证与来源说明。
- **暂缓一件事。** 它的 agentic benchmark 骨架（钉定仓库、配对臂、隔离每臂插件、以 `git diff` 计量、对抗输入执行）是评估「一份 Skill 到底改了多少行为」的可复用方法；HCTL2 将来若要给 Skill revision 配效果证据，可回头看 `benchmarks/agentic/`。此刻不动。

## 未查到与边界

- **作者身份**：GitHub 资料空白，没有公司、博客或所在地；ponytail.dev 候补名单指向商业化，方向未公开。钉定版本可防未来许可证变化影响本文引用。
- **设计系统仓库上的表现**：没有任何 benchmark 在带组件库的仓库上跑过，dev.to 提出的「原生控件排在已装依赖之前会不会破坏设计系统」无人回答。
- **非 Claude 模型**：作者的成本核验含 GPT 与 Gemini 臂，本文未逐读；README 只留一句 GPT-5.5 上省 token 反转的说法，没有数据表。
- **长期维护成本**：所有评测都只看一次性产出，没人测过几周后削角注释是否被兑现、精简代码是否更难改。`ponytail-debt` 的台账机制在实测里几乎没被触发（1/80）。
- **本文没做的**：没有复跑任何 benchmark；hook 只逐读了 Claude / Codex 路径的五个文件，OpenCode / Pi / Hermes / MCP 适配层与 `docs/platform-native.md` 依赖第三方架构分析（MartianLee）的摘要；「20 个 agent」的覆盖数字未核。
- **本仓库现状**：`docs/`、`.memo/`、`src/` 里此前没有任何对 ponytail 的提及；本文是首个条目。
- **星数与 issue 数**是 2026-09-10 至 11 的 API 读数，增长很快，引用时看日期。

## 证据

上游（源码里看到，钉 main @ `356918eb`，v4.9.0 @ `0a4dd63a`）：

- 仓库与元数据：https://github.com/DietrichGebert/ponytail （GitHub API 读数 2026-09-10：134,451★、7,193 fork、MIT、created 2026-06-12、pushed 2026-09-07）
- 主 Skill：`skills/ponytail/SKILL.md`；子 Skill：`skills/ponytail-review/SKILL.md`、`skills/ponytail-audit/SKILL.md`、`skills/ponytail-debt/SKILL.md`
- hook：`hooks/claude-codex-hooks.json`、`hooks/ponytail-activate.js`、`hooks/ponytail-instructions.js`、`hooks/ponytail-mode-tracker.js`、`hooks/ponytail-subagent.js`、`hooks/ponytail-runtime.js`
- 作者 benchmark：`benchmarks/README.md`、`benchmarks/results/2026-06-18-agentic.md`、`benchmarks/results/2026-06-17-agentic-safety.md`（已标 SUPERSEDED）、`benchmarks/results/2026-06-22-issue-245-217-comprehension.md`
- Releases：https://github.com/DietrichGebert/ponytail/releases （v1.0.0 与 v4.0.0 均 2026-06-12；v4.9.0 2026-08-07）
- issue：#126（benchmark 基线失真，2026-06-16）、#161 / #584（模式开关失效）、#217（缺「仓库已有」一级，作者拒绝工具化）、#245（Dangerously lazy）、#252（子代理收不到规则）、#332（与 caveman 冲突）、#406 / #419 / #423（示例未标注跳过项）、#735（木马克隆）、#745（常驻「读全」卡死大文件）

别人测的（第三方文章与仓库，读于 2026-09-11）：

- JetBrains AI 博客，Denis Shiryaev，2026-07：https://blog.jetbrains.com/ai/2026/07/ponytail-skill-claude-tested/
- Scott Logic 博客，Colin Eberhardt，2026-06-16：https://blog.scottlogic.com/2026/06/16/ponytail-yagni-and-the-problem-with-prompt-benchmarks.html
- MartianLee（SeongHwa Lee）架构分析，2026-06-30：https://martianlee.github.io/posts/2026-06-30-ponytail-architecture
- 独立 480 次执行评分 benchmark v3：https://github.com/Deepusleepy/ponytail-benchmark （MIT，2026-06-21 创建，2026-07-22 最后推送；ponytail README 链接的 KuldeepB19 页面数字与之一致）
- dev.to，yashddesai，2026-06-25：https://dev.to/yashddesai/ponytail-the-ai-coding-skill-taking-github-by-storm-and-the-one-question-nobodys-answered-yet-46mc
- 衍生与同名对象（gh search，2026-09-11）：ilindaniel/ponytail-lite、anshaneja5/scalpel、decapostos/lean-code、gongyijie85/dsh-ponytail、linsomniac/ponytail（无关）、i17c/PonyTail（无关）

HCTL2 侧引用（核对于本分支当前 main）：

- [docs/design/participant.md](../design/participant.md)：专业化 Participant、七件事分层、Agency 与执行体
- [docs/design/spec/participant.md](../design/spec/participant.md)：Skill 与申报
- [src/agency/README.md](../../src/agency/README.md)：两种触发方式与判据
- [docs/research/harness-hooks-20260903.md](./harness-hooks-20260903.md)：钩子由参考实现在拉起前注入
- [docs/research/methodology-mattpocock-skills-20260902.md](./methodology-mattpocock-skills-20260902.md)：技能包类的既有先例与「方法归 Skill、方法管不住自己的地方归机制」的分界原则

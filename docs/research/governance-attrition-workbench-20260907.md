# 治理功能的消亡：工作台类产品逐家核对

> 类别：跨候选归纳 · 证据编号：E-GOVERNANCE-ATTRITION-WORKBENCH<br>
> 状态：调研 · 日期：2026-09-07（emdash、Yoda 两家）；其余十三家 2026-09-08 补齐<br>
> 总览与复用决策用语见 [docs/research/README.md](./README.md)；方法论类工具的同题调研见 [governance-attrition-methodology-20260907.md](./governance-attrition-methodology-20260907.md)。

## 问题

在我们已调研过的工作台 / Agent 协作平台里，**治理性或结构性的功能**——看板与任务状态机、阶段门与审批、规格或契约、多 agent 编排与角色、评审门、自动化工作流、验收与证据——被加进来之后，有没有被删掉、弃用、藏进开关、或长期无人迭代？社区反馈里有没有「太重、我只想开个终端」一类导致简化的声音？触发这个问题的是 [Yoda](./workbench/yoda.md) 的 Feature Delivery：一个下午加了一万六千行、活了 33 天、在一个 `fix(privacy)` 提交里被顺手删掉。这里逐家核对，只记事实与维护者原话，推断单列；对 HCTL2 的含义在归纳篇写。

## 逐家发现

### emdash（基线 `a5a8379`，2026-09-06）

先钉住 v1 重写的时间骨架：v1 分支 2026-03-04 起头；砍功能的实际提交在 v1 分支上——[`532299479` cleanup](https://github.com/generalaction/emdash/commit/532299479a9d2d373bcc3879d8e2eca62226a604)（03-20，+36 −7,939，70 文件）一次删掉看板、多 agent 与浏览器组件，[`8e1b72e41` cleaning](https://github.com/generalaction/emdash/commit/8e1b72e41)（04-09）删看板 store，[PR #1728](https://github.com/generalaction/emdash/pull/1728)（04-16）删对应文档；[PR #1764「promote v1 to main」](https://github.com/generalaction/emdash/pull/1764)（04-25，+91,408 −131,609，1,499 文件，正文只有一个表情）进主干。用户拿到的第一个删除版本是 v1 beta（04-16）与 v1.1.5（04-30）。

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| Kanban 看板（To-do / In-progress / Ready，自动状态迁移） | [`1dc0ee7fc`](https://github.com/generalaction/emdash/commit/1dc0ee7fc) 2025-11-10，[PR #265](https://github.com/generalaction/emdash/pull/265)；需求源是维护者自提的 [#195](https://github.com/generalaction/emdash/issues/195) | **删除，未回来** | v1 分支 03-20 与 04-09 → v1 beta 04-16；最后一次功能维护 03-12（修 [#1443](https://github.com/generalaction/emdash/issues/1443)「看板不显示任务」） | 见上 | [#339](https://github.com/generalaction/emdash/issues/339) 维护者 04-29：「closing because we temporarily kicked out the kanban board when switching to v1」；v1 beta 博文完全没提看板 | 「temporarily」之后四个半月无任何提交或 issue 重提；最接近的对比板 [#2374](https://github.com/generalaction/emdash/issues/2374) 仍开着。看板已事实放弃。#339 里 Discord 用户提的「状态钩子触发另一 agent 复核」属于阶段门设想，随看板一起关闭 |
| Best-of-N（同一供应端多实例并跑） | 维护者提 [#353](https://github.com/generalaction/emdash/issues/353) 2025-11-29，外部贡献者当天实现 [PR #362](https://github.com/generalaction/emdash/pull/362)；此后有过开关、每变体评论注入等迭代到 2026-03-11 | **删除，未回来** | 03-20 → v1 beta | 见上 | 博文：「Best-of-N (might come back)」 | 配套的按键注入机制也在 07-14 删除（v1.2.0）；多实例对比这条线整体退出 |
| Multi-agent workspace（同一提示词发给多个 CLI） | [PR #247](https://github.com/generalaction/emdash/pull/247) 2025-11-01 | **删除，未回来** | 03-20 | 见上 | 无 | Best-of-N 的宿主；Show HN 原帖仍宣传「start one or a few agents on the same problem」，v1 后只能靠多任务 |
| 内嵌浏览器预览 | [PR #259](https://github.com/generalaction/emdash/pull/259) 2025-11-08 | **删除 → 两个半月后重做回来** | 删 03-12 到 04-02；回 [PR #2373](https://github.com/generalaction/emdash/pull/2373) 06-09（v1.1.32），此后持续加功能到 09-04 | [#1890](https://github.com/generalaction/emdash/issues/1890) | 博文：「will come back」；维护者 05-06：「bringing back an in app browser is on our roadmap!」 | 被删三项里唯一回来的，是**非治理性**的工具类功能 |
| Plan Mode UI（统一计划模式与 planning.md） | 2025-10-23 | **删除 → 七个月后以供应端原生形态回来** | [PR #600](https://github.com/generalaction/emdash/pull/600) 2026-01-12；回 [PR #3061](https://github.com/generalaction/emdash/pull/3061) 08-29（Codex ACP 的 plan mode） | 见链接 | PR #600：「simplifies the UI by removing the provider selection and plan mode features, reducing the codebase by ~974 lines」；HN 原帖：「If a provider starts supporting plan mode, we don't have to add that first」 | 自建计划层被拆，改为透传 CLI 自带能力 |
| Docker 容器化工作区 | 2025-10-27 | **删除** | [PR #608](https://github.com/generalaction/emdash/pull/608) 2026-01-13（−4,623，分支名带 temp）；[#205](https://github.com/generalaction/emdash/issues/205) 04-29 关闭 | 见链接 | PR #608：「removes the Docker containerization feature … to simplify the codebase and focus on the core agent orchestration functionality」 | 分支名带 temp，但未回来 |
| Diff 行级评论发给 agent | [PR #540](https://github.com/generalaction/emdash/pull/540) 2026-01-03（数据库持久化） | **被重构误删 → v1 保留但降级为内存草稿** | 误删链 #600 → #611 → PR #825；用户 [#1388](https://github.com/generalaction/emdash/issues/1388) 03-10 发现；v1 改为内存草稿（[`8a968fe18`](https://github.com/generalaction/emdash/commit/8a968fe18)、[`e06e2663e`](https://github.com/generalaction/emdash/commit/e06e2663e)），最后触碰 07-20 | 见链接 | 维护者：「We'll look into reintroducing the popover」 | 留住了，但「评审证据」从持久化降为一次性草稿 |
| BYOI / workspace provider（自带基础设施） | v0：03-16 [藏进 PostHog 功能开关](https://github.com/generalaction/emdash/commit/bfc68a7b8)；v1 重做 [PR #1802](https://github.com/generalaction/emdash/pull/1802) 04-29，同日 [PR #1826](https://github.com/generalaction/emdash/pull/1826) 加开关 | **开关后 → 三个月删除** | [`89f129a05`](https://github.com/generalaction/emdash/commit/89f129a05ea4ff24d68ea238ce44c9bbb1dd921b) 08-04（v1.2.0） | 见链接 | v1.1.5 发布说明：「groundwork for upcoming features like remote workflows and BYOI workspaces」 | 两次都是先藏开关再删 |
| 内部架构收敛（不是用户功能，记作参照） | v1 期间 | operations kernel 与操作日志删除（[`ffe97a71c`](https://github.com/generalaction/emdash/commit/ffe97a71c) 08-05，−14,525，149 文件）；workspaceHost 运行时退役（[ADR 0007](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/docs/adr/0007-workspace-host-runtime-retired.md) 08-07）；脚本工作流托管删除（08-08） | — | 见链接 | ADR 0007：「workspaceHost had shrunk to a residue of one observation and three dead or single-caller verbs」 | 维护者自己的收敛纪律：残留即删并写 ADR |
| 合并编排、编排者子 agent（含 approval gates）、层级规划、AI 评审、agent 打分 | 只有 issue（[#322](https://github.com/generalaction/emdash/issues/322)、[#534](https://github.com/generalaction/emdash/issues/534)、[#533](https://github.com/generalaction/emdash/issues/533)、#562、#897） | **未实现，全部在 2026-04-29 同一天关闭**（当日共关 43 个 issue） | — | 见链接 | #533 关闭语：「Have you had a chance to try Emdash v1 yet? We've made a lot of changes since the earlier versions」 | v1 上线借机清空了编排与治理类愿望单；#534 里「approval gates before resuming work」是维护者自己写的 |
| 工单状态回写 | 从未有 | **开着，无进展** | — | [#1930](https://github.com/generalaction/emdash/issues/1930) | 维护者 05-10：「i dont think we do that rn (for any provider)」；用户 07-23 追问无回复 | — |

**社区反馈里的信号**：两篇 v1 博文（[beta](https://emdash.com/blog/public-v1-beta)、[stable](https://emdash.com/blog/emdash-v1-stable)）都没有说为什么拿掉，只说「不在这版里」，核心叙事是「fast, calm, and reliable」。[Show HN](https://news.ycombinator.com/item?id=47140322)（2026-02-24）里「CLI 自己会进化，我为什么要在你的 ADE 上花时间」「我最后还是回到 Claude Code 分屏」两条没有得到维护者回应；「为什么不让一个 agent 管五到十个」得到的回答是「CLI 自己在原生做编排，我们站在更高的任务层」。[#619](https://github.com/generalaction/emdash/issues/619)「右上角按钮太多」直接导向了 v1 重设计；[#2285](https://github.com/generalaction/emdash/issues/2285) 用户要「不建任务的独立聊天」，维护者 09-07 关闭：不计划做；[#1462](https://github.com/generalaction/emdash/issues/1462) 要 spec 驱动框架，无回应。

**留住并持续迭代的**：Automations（03-29 合入，04-06 标 Beta，06-12 摘 Beta，08-25 仍在修）；工单集成从三家扩到十二家（但不回写）；diff 评审与 PR 流程被称为「very reliable pull request engine」；行级评论以草稿形态留住；plan mode 以供应端原生形式回归；ACP 聊天界面先藏开关、07-13 摘掉开关转正——这是反向案例。

### Yoda（基线 `960fd322`，v0.20.3）

单人产品：3,159 个提交里 3,153 个出自一人，71 星，Discussions 关闭，全史 5 个 issue，没有一条关于治理、复杂度或「只想要终端」的社区声音；所有砍功能决定的唯一理由来源是提交正文（均标注 Co-Authored-By Claude Opus）。

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| Feature Delivery（六阶段、仅人批准的治理流程） | 07-13 三个提交一个晚上加完（[`7dcc2411`](https://github.com/lovstudio/yoda/commit/7dcc2411)、[`89f5860d`](https://github.com/lovstudio/yoda/commit/89f5860d641bfb449340596426c1e3ac658c7d3f)、[`cf5f191b`](https://github.com/lovstudio/yoda/commit/cf5f191bb8ce4256d6b636d4f7de6c4eed716a0f)，合计约一万八千行），v0.16.1 CHANGELOG 有条目 | **整体删除，CHANGELOG 未提** | 代码在 [`b1a1a5ff` fix(privacy)](https://github.com/lovstudio/yoda/commit/b1a1a5ff67de53e7eaf73e6a790e8a195616b156) 08-15（+103 −4,937，31 文件，正文只讲隐私白名单）；数据域在同日 [`2aaed52f`](https://github.com/lovstudio/yoda/commit/2aaed52fc2f17d07129b89291c29ae0e9c39f197)「Drop the feature-workflow domain … Migration 0060 drops the retired tables」 | 见链接 | 2aaed52f：「`BUILTIN_TEAMS` and the `feature-workflow` room preset go with them — a builtin is a default a user can edit, and these were neither reachable nor editable」 | 33 天里只有一次布局修补，无功能迭代；文档 `agents/architecture/features.md` 与 `docs/features/feature-development-workflow.md` 残留至基线，仍指向已不存在的文件 |
| review 模式（实现者与审查者并排、审查编排状态机） | 06-02 随运行模式框架出现，06-16 「first usable cut」 | **退役，并删 review-orchestration 表** | 2aaed52f 08-15（v0.20.0） | 见链接 | 「three of them were shells: `spec` and `review` duplicated single-agent launch with a different prompt … Each still paid for its own slots, capabilities, params schema, config panel, i18n namespace and — for review — a persisted orchestration table. Delete them recursively so a paradigm kind stays something a reader can hold in mind」 | 存活 74 天 |
| brainstorm（spec）模式（苏格拉底追问产出 PRD） | [`c71ecf42`](https://github.com/lovstudio/yoda/commit/c71ecf42) 06-06 | **退役**，持久化值强制折回 normal | 2aaed52f 08-15 | 见链接 | 同上 | 存活 70 天 |
| build（app-build → AI Lab） | [`97f09921`](https://github.com/lovstudio/yoda/commit/97f09921) 07-18 | **退役** | 2aaed52f 08-15 | 见链接 | 「`app-build` only existed to hand a project off to AI Lab」 | 存活 28 天 |
| compare 模式 | 06-13 前 | **被通用「多配置对比」替换**，HEAD 里只剩描述符不在选择器里 | [`cd83fc45`](https://github.com/lovstudio/yoda/commit/cd83fc45) 06-19（v0.13.4） | 见链接 | CHANGELOG 0.13.4：「replaced by the general multi-config comparison」 | 第一个被砍的模式 |
| Team Room 确定性 review-loop（PASS / FAIL 状态机、轮次上限、stall 兜底） | [`94a109c8`](https://github.com/lovstudio/yoda/commit/94a109c8) 06-16 09:43 | **次日删除**，数据库 `verdict` 列残留、全库无调用方写入 | [`6e4d804d`](https://github.com/lovstudio/yoda/commit/6e4d804d) 06-17 18:23（v0.13.0） | 见链接 | 「删除 conductor 里 review-loop 的确定性状态机、onReviewerVerdict、review 回合检测/stall fail-forward」；「取舍：失去确定性 PASS/FAIL、轮次上限、codex stall 兜底，换取架构纯通用」 | 存活 33 小时；机械评审门让位给「一个通用的提示词驱动引擎」 |
| 任务即会话（一任务一会话） | 多会话本是常态（07-19 多会话状态管理器） | **收紧为 1:1 并回填拆分历史数据** | [`04e4dc67`](https://github.com/lovstudio/yoda/commit/04e4dc67) 08-17、[`e3635056`](https://github.com/lovstudio/yoda/commit/e3635056) 08-18（v0.20.1 到 0.20.3） | 见链接 | 「任务与会话不是 1:1，而是同一。点任务不该先落在一张任务信息页、再点一次才进会话」；AGENTS.md 新增铁律「一个任务只有一个会话」 | 理由全是「少一次点击 / 概念同一」，未引用任何用户反馈；结构层级被主动压扁 |
| 看板加列级钩子（拖卡触发提示词注入 / 命令 / 通知） | [`92512632`](https://github.com/lovstudio/yoda/commit/92512632) 06-12（v0.9.0，标 Alpha） | **停滞**：Alpha 标未摘，钩子文件全史 4 个提交、唯一功能提交就是加入那天 | — | 见链接 | CHANGELOG 0.9.0：「Cross-project Agent kanban (Alpha)」 | Yoda 在 06-12 重建了 emdash 03-20 刚砍掉的看板；8 月新做的独立看板窗明确「只观察不接管」（CHANGELOG 0.20.1） |
| Issue Worker（自治工单队列） | [`226d27b8`](https://github.com/lovstudio/yoda/commit/226d27b8) 08-06（v0.18.8） | **停滞**：同日之后再无提交，CHANGELOG 无条目 | — | 见链接 | — | 一天完工即封存 |
| Automation | 05-31（v0.3.5） | 活跃到 8 月中，基线时三周无动 | — | — | — | — |

**三件最大的砍功能——五范式砍成二、Feature Delivery 删除、review-loop 确定性删除——都不进 CHANGELOG**；0.20.0 只写「新增并完善开发范式与 Agent roster 配置」；README 基线仍宣传「运行模式：normal / brainstorm / compare / review / team」。

**留住并持续迭代的**：Agent Teams / team 范式（Team Room 68 个提交，08-19 仍在改）；工单集成同步上游新增五家；交付摘要（07-13 起持续）；观察型看板（08-17 新建）；MaaS、通知中心、runtime bar。

### Superset（基线 `main@4e18e1fa`，2026-08-13；发布到 desktop-v1.27.0，09-07）

YC 团队，付费墙与云服务贯穿全程；223 个 release 全是 PR 清单，没有一条 release 说明写过「移除」某个用户可见功能。治理面有三块：Tasks（从 Linear 双向同步起家）、Workspaces board（按运行状态分泳道）、Automations；多 agent 编排只以一个 plugin skill 存在，产品内没有编排状态机。

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| Tasks（Linear 双向同步、表格 / 看板两种视图） | 集成 2025-12-27 [PR #503](https://github.com/superset-sh/superset/pull/503)（desktop-v0.0.37）；看板视图 2026-03-19 [PR #2618](https://github.com/superset-sh/superset/pull/2618)（v1.2.2） | **留住并持续迭代** | — | 07-10 Linear project / cycle 过滤（[v1.14.3](https://github.com/superset-sh/superset/releases/tag/desktop-v1.14.3)）；08-26 文档刷新 [`a6b566aa`](https://github.com/superset-sh/superset/commit/a6b566aa)；HEAD 文档写「listed alongside Superset-native tasks you create in the app」 | [#1926](https://github.com/superset-sh/superset/issues/1926)（03-01 要独立看板）07-11 作为 stale-issue sweep 以 duplicate 归入 tracker [#5599](https://github.com/superset-sh/superset/issues/5599) | 独立本地任务后来做了，但 issue 是被清扫关掉而不是「已实现」关掉；治理停在「工单镜像 + 状态字段」，没有阶段门。用户 #1926 原话「I understand the task management feature is a paid feature」 |
| Workspaces board（Idle / Working / Needs attention / Needs review / Merged / Deleted 六泳道） | 08-17 [PR #6563](https://github.com/superset-sh/superset/pull/6563)（board view by default）、[PR #6568](https://github.com/superset-sh/superset/pull/6568)（泳道可隐藏） | **留住，新增** | — | 09-04 v1.26.0 清理卡片 #7150、主工作区进 Needs review 列 #7142 | PR #6568：「Linear's board (our reference) offers per-column "Hide column" plus a "Show empty columns" toggle」 | 泳道是从 workspace 运行状态与 PR 状态推导的，不是人工拖动的任务状态机；「Needs review」= 有 PR 待看 |
| Automations | 04-30 前已有 | **开关 → 付费墙 → 摘开关，持续迭代** | 04-30 v1.7.3 进付费墙 [#3850](https://github.com/superset-sh/superset/pull/3850) → 05-08 v1.8.8「remove automations-access feature flag」[#4213](https://github.com/superset-sh/superset/pull/4213) → 05-20 v1.9.9「ungate automations」[#4734](https://github.com/superset-sh/superset/pull/4734) | 08-18 Linear 触发器；09-07 PR assigned / review-requested 触发器 [#7163](https://github.com/superset-sh/superset/pull/7163) | — | 与 emdash Automations 同轨：先藏、后收费、再放开；是反向案例 |
| 编排 skill（plugin，提示词层） | 08-02 v1.18.3 [#6088](https://github.com/superset-sh/superset/pull/6088) | **留住；只在 skill 层** | — | 基线 [`plugins/superset/skills/orchestrate/SKILL.md`](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/plugins/superset/skills/orchestrate/SKILL.md)；08-26 营销「position Superset as agent-independent orchestration」 | — | 编排没有进产品状态机；与 emdash「CLI 自己在原生做编排，我们站在更高的任务层」是同一立场 |
| Plugins MVP、内部 chat agent | 08-21 [#6722](https://github.com/superset-sh/superset/pull/6722)「behind internal flag」；08-26 [#6852](https://github.com/superset-sh/superset/pull/6852)「gate the Superset chat agent choice behind the chat-v3 flag」 | **藏进开关，进行中** | — | 见链接 | — | 开关是常规发布手段，结局待看 |
| v1 → v2 迁移残留（参照） | — | 内部收敛 | 05-07 删 v1→v2 导入横幅；06-08 [#5185](https://github.com/superset-sh/superset/pull/5185)「remove legacy v1 starter surfaces」 | 见链接 | — | 重写后清理旧入口，没有砍用户功能 |

**社区反馈里的信号**：[Launch HN](https://news.ycombinator.com/item?id=48236770)（2026-05-22，135 条评论）是「只想开个终端」声音最集中的一处。hmokiguess：「iTerm2 tabs / tmux is plenty. This, or tools like Warp, feel so heavy for me and I get overwhelmed」；创始人 hoakiet98 回应：「us and similar categories of tools prescribe a lot of our specific workflow for worktrees, review, code editing where iterm + tmux is much more flexible. we have to tow the line of what's useful as a built-in feature vs being very agnostic」；whinvik：「my NeoVim setup + tmux + Ghostty is good enough and I am not ready to learn a whole another system for modest gains」；dbbk：「I also don't believe in this multi-agent swarm thing」；ChicagoDave：「no llm can be left unsupervised … For every agent you deploy as a "team" you will have multiplied diminishing returns」；tdi 抱怨「No linear integration in free version and taxing it 20$/m is a bit steep」。正面声音 cpan22：「the native Codex/Claude Code TUIs without any of the bloat」。[Show HN](https://news.ycombinator.com/item?id=46368739)（2025-12-23）kaffekaka：这种工作流「feels a lot like "making the horse ten times faster"」。issue 区没有治理类抱怨，唯一的独立看板需求 #1926 被 stale sweep 关掉。

**留住并持续迭代的**：Tasks 与 Linear 同步、Workspaces board、Automations（触发器持续加）、PR 评审与合并（09-07 从 Changes 面板回复 PR review threads）、编排 skill。没有任何治理性功能被删。

### Orca（基线 `09ec516a`，2026-08-12，1.4.178-rc.2；发布到 v1.4.197，09-04）

日更节奏（3 月至 9 月 947 个 release）。编排目录 `src/main/runtime/orchestration` 的提交按月是 4 月 1、5 月 7、6 月 11、7 月 22、8 月 46、9 月 12——投入在加大，但最重的一块「自动调度器」在 7 月被退役。

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| 编排调度器（`orchestration run` / `run-stop` 协调者循环，`--max-concurrent` 自动派发） | 04-28 [PR #1188](https://github.com/stablyai/orca/pull/1188)（v1.3.24）：「Phase 3 — Coordinator loop: `orchestration run` starts a coordinator loop in the runtime via RPC」；05-04 [PR #1403](https://github.com/stablyai/orca/pull/1403) 加心跳与 stale 检测 | **退役（命令名保留，不再有效果）** | 07-27 [PR #9925](https://github.com/stablyai/orca/pull/9925)（+23,286 −977，127 文件；改指南的提交 [`cd05f2ff`](https://github.com/stablyai/orca/commit/cd05f2ff)） | 07-14 的指南（[`31f643ca`](https://github.com/stablyai/orca/blob/31f643ca/skill-guides/orchestration.md)）仍写 `orca orchestration run --spec <text> … --max-concurrent <n>`；基线指南写「`coordinator-start`, `coordinator-stop`, `run`, and `run-stop` are retired scheduler commands. They perform no effects and return the current-skill recovery action」 | PR #9925：「This intentionally does not add a dashboard, scheduler, automatic placement or retry, capacity allocator, commit/integration tracking … Agents still choose decomposition, placement, concurrency, conflict avoidance, and recovery」 | 活了三个月；机械调度让位给「agent 自己决定分解、落点与并发」，与 Yoda review-loop「换取架构纯通用」同向；退役只记在 PR 正文与 skill guide 一句里，release 说明没提 |
| Decision gates（`gate-create / resolve / list`，人在环节点） | 同 PR #1188 04-28 | **留住** | — | 07-31「Fix orchestration gate authorization to scope by Run binding」[#11802](https://github.com/stablyai/orca/pull/11802)；基线指南「Use `gate-create` only for coordinator-managed task DAG decisions, not for answering a worker's `ask`」 | — | 留下来的是「停下来问人」原语，删掉的是「自动往前推」原语 |
| Agent Dashboard / Agents view | 04-25 v1.3.18「agent-status plumbing behind AGENT_DASHBOARD_ENABLED (dark launch)」 | **开关翻转四次后转正** | 04-28 experimental → 04-30「replace bottom-panel dashboard with inline per-card agents list」→ 05-08 [v1.3.41](https://github.com/stablyai/orca/releases/tag/v1.3.41)「default-on, remove experimentalAgentDashboard toggle」→ 05-18 [v1.4.5](https://github.com/stablyai/orca/releases/tag/v1.4.5)「Gate Agents view behind Experimental」→ 07-21 [v1.4.149](https://github.com/stablyai/orca/releases/tag/v1.4.149)「Singleton pop-out dashboard with attention / working / idle columns」[#9604](https://github.com/stablyai/orca/pull/9604) → 08-07 experimental agent map view [#12168](https://github.com/stablyai/orca/pull/12168) → 08-26 快捷键 [#15353](https://github.com/stablyai/orca/pull/15353) | 见链接 | — | 「关注 / 工作中 / 空闲」三列是观察面不是任务状态机；四次翻转说明团队对「要不要给用户一张全局板」拿不准，最后留了 |
| Workspace board（看板） | 05-30 前已有（[#3420](https://github.com/stablyai/orca/pull/3420)「Allow sidebar workspaces to drop onto Kanban」） | **留住，小修** | 05-31「Remove workspace board compact mode」；07-01 重排默认列 [#6934](https://github.com/stablyai/orca/pull/6934)；07-31 加搜索 [#11244](https://github.com/stablyai/orca/pull/11244)；08-17 快捷键改 toggle [#14240](https://github.com/stablyai/orca/pull/14240) | 见链接 | — | 卡片身份是 worktree，`workspaceStatus` 是人工侧栏分类（见 [stably-orca.md](./workbench/stably-orca.md)）；用户仍另开 issue 要「任务看板」（[#15573](https://github.com/stablyai/orca/issues/15573) 08-20、[#16885](https://github.com/stablyai/orca/issues/16885) 08-27，各 0 回复）——这张板没被当成任务状态机 |
| Tasks 页（Linear / GitHub / Jira 工单） | 04-22 前已有 | **留住；可整体隐藏** | 04-30「replace task-icon toggle with Tasks-button toggle, expose in View menu」 | 见 v1.3.25 | — | 与 Superset 同型：工单镜像而非自建任务对象 |
| Automations | 07-05 前已有（headless 调度器 [#7296](https://github.com/stablyai/orca/pull/7296)） | **留住并加运行面板** | 09-04「Add automation runs dashboard with pagination and filtering」[#18226](https://github.com/stablyai/orca/pull/18226) | 见链接 | — | — |
| 实验开关的双向流动（参照） | — | 两个方向都走 | persistent terminal daemon 04-17 进 experimental；per-workspace environments 06-30 移入 Experimental [#6926](https://github.com/stablyai/orca/pull/6926)；APFS shared paths 07-11「Promote … from experimental」[#8318](https://github.com/stablyai/orca/pull/8318) | 见链接 | — | Orca 把开关当常规工具，不能单看「藏进开关」判死刑 |
| auto-review-fix skill（维护者自用的多 agent 评审循环，参照） | 03-23 [PR #50](https://github.com/stablyai/orca/pull/50)，放在 `.claude/skills` | **移出仓库** | 04-29 [`c85f487e`](https://github.com/stablyai/orca/commit/c85f487e)「chore: move agent skills to internal delivery」 | 见链接 | — | 开发者自用工具不是用户功能，只记作参照 |

**社区反馈里的信号**：[Show HN](https://news.ycombinator.com/item?id=47549197)（03-27，10 条评论）没有治理讨论。issue 区没有「太重」的声音，方向相反：要更多编排能力（[#4376](https://github.com/stablyai/orca/issues/4376) 06-01「Improved orchestration」、[#13360](https://github.com/stablyai/orca/issues/13360)「worker-start --spec」、[#14856](https://github.com/stablyai/orca/issues/14856) 在 supervised Run 里跑第三方 skill、[#17973](https://github.com/stablyai/orca/issues/17973) AI 生成 issue spec），以及编排不好用（[#14303](https://github.com/stablyai/orca/issues/14303)「Orchestration not working」08-13、[#14104](https://github.com/stablyai/orca/issues/14104) 示例卡与指南给出相反的 handoff 路由 08-12、[#13047](https://github.com/stablyai/orca/issues/13047) worker 终端 settle 后不关 08-07）。

**留住并持续迭代的**：decision gates、Run / Dispatch / Receipt 持久结构（09-06「make multi-agent workflows durable」[#16904](https://github.com/stablyai/orca/pull/16904)）、Agent Dashboard、workspace board、Tasks、Automations、hosted review 与 review notes、Linear 状态同步。

### Codeg（基线 v0.24.0 `df7a872d` 08-11 / `main@a34a047a` 08-14；发布到 v0.30.5，09-08）

单人维护（xintaofei），3.3k 星，3 月起 180 个 release，说明双语。治理面是 08-02 才上的 Task Board，五周内每周都在加东西，是本次样本里唯一「越做越厚」的任务状态机。

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| Task Board（beta）→ To-dos | 08-02 [v0.23.0](https://github.com/xintaofei/codeg/releases/tag/v0.23.0) | **留住、改名、持续加功能** | 08-07 [v0.23.5](https://github.com/xintaofei/codeg/releases/tag/v0.23.5) 改名 | 08-03 v0.23.1 per-stage prompts、Preparing 状态、设置拆 General / Workflow / Prompts；08-06 v0.23.3 follow-up intent（Rework / Keep going / Ask / Double-check）、无改动任务直接 Complete；08-07 列表视图、定时启动、agent 可从对话建 to-do（默认关）；08-11 v0.24.0 auto-merge；08-14 v0.25.0 merge 排队；08-21 [v0.27.0](https://github.com/xintaofei/codeg/releases/tag/v0.27.0) Repository panel 把 issue / PR 变 to-do；08-24 v0.28.1 侧栏行可关（Automations、To-dos、Repository）；08-29 v0.29.0 PR 三页签加 Merge 按钮 | v0.23.0：「A finished task waits for you to look it over, then merges when you say yes」；v0.23.5：「The Task Board is called To-dos. The page, the sidebar entry, the settings copy, and the tool description an agent reads all use the new name」 | 改名把「看板」降成「待办」，但状态机（Queued / Preparing / In progress / Reviewed / merge）反而变厚；「可隐藏」是给用户的退路，不是收回 |
| 人工评审门（Reviewed → Merge） | 08-02 | **留住，但加了绕过开关** | 08-11 v0.24.0 | 「Merge automatically. A folder can land reviewed tasks on its own, exactly as pressing Merge would. Anything that fails stops and waits for you」 | 门没删，默认仍要人点；自动合并是按文件夹开的选项 |
| 「Process all」按钮 | 08-02 | **删除** | 08-07 v0.23.5 | 「The toolbar's "Process all" button is gone. Start a task from the card itself, or turn on "Process automatically" in Task settings」 | 批量推进的中间档被去掉，只剩每卡一键或全自动 |
| Automations | 06-24 前已有 | **留住，重做** | 08-02「The Automations page has been rebuilt」；可产出任务而不是会话 | 见 v0.23.0 | — |
| Delegation（子 agent）、审批策略 | 05-26 v0.14.3；07-25 v0.21.9 Codex「Sandbox & approval」设置；08-14 OpenCode 权限卡 | **留住** | — | 见链接 | — | 审批策略是透传各 CLI 的，不是自建 |
| Infinite Conversations（空间画布） | 09-01 [v0.30.0](https://github.com/xintaofei/codeg/releases/tag/v0.30.0) | 新增 | — | 「a board that lays your workspace out in space」 | — | 在 To-dos 之外再开一张「板」，方向是可视化而非流程 |

**社区反馈里的信号**：[#532](https://github.com/xintaofei/codeg/issues/532)（08-21，0 回复）：「现在的待办总感觉很割裂，放着大输入框和文件树不用，跑去单独的待办界面很憋屈」，要在对话里直接建待办、侧栏手风琴面板、worktree 统一目录；[#453](https://github.com/xintaofei/codeg/issues/453)（08-14）要「more worktree-centric」。没有「太重」或「拿掉看板」的声音；抱怨方向是「离对话太远」。

**留住并持续迭代的**：全部。Codeg 是反例：治理面进了主路径、周更、越来越机械（排队、阶段提示词、自动合并的失败即停）。

### Multica（基线 `main@2c0912b6` 08-14，v0.4.26；发布到 v0.4.41，09-07）

Linear 式工单是产品本体，看板不可能被删；被删的是工单上的治理字段和触发器上的状态门，被挪走的是运行时 brief 里的纪律。社区里出现了本次样本中最尖锐的「人在环太重」长文，维护者正面捍卫。

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| Issue 上的 acceptance_criteria 与 context_refs 字段 | 加入日期未查到（03-25 v0.1.8 之前） | **删除（DB 列保留）** | 03-26 [`a5000010`](https://github.com/multica-ai/multica/commit/a500001093ed3bd42c21b08144a819e023c7cb8d)（v0.1.9 03-30，17 文件 +77 −382） | 见链接 | 「These fields were unused in practice. Removed from frontend types, issue detail UI, backend handlers, daemon prompt/context, protocol messages, SQL queries, and tests. DB columns retained with defaults」 | 一等验收字段活了不到两个月，理由是「没人用」 |
| 触发器状态门（on_assign 只对 todo 生效、on_mention 排除 done / cancelled） | — | **删除** | 04-01 [`eb5aaf00`](https://github.com/multica-ai/multica/commit/eb5aaf003c03bd9aea8514c258cb4b87272ba7a3)（v0.1.14） | 见链接 | 「Assignment is an explicit human action — if someone assigns an agent to a done/in_progress issue, they want the agent triggered … The agent can reopen the issue if needed」 | 状态机对触发的约束被拿掉，改信「人的显式动作」 |
| Issue Metadata 写入纪律（runtime brief 里的推荐键与禁令） | — | **从 brief 挪进 skill（提示词层）** | 08-04 [PR #6351](https://github.com/multica-ai/multica/pull/6351)（MUL-5442，v0.4.18）；08-05 / 08-06 stage-1、stage-2「judgment rewrite」 | 见链接 | 「metadata is deliberately free-form custom key-value state — the recommended-keys block never matched the feature's intent and is removed outright. The full write discipline defers to the `multica-working-on-issues` skill」 | brief 从 16,099 字节减到 15,579；纪律没消失，从每次必带的 brief 变成按需加载的 skill |
| 领域提示词（focused-testing skill、Repository Setup Preflight） | 07 月中 | **回滚** | 07-24 [PR #5913](https://github.com/multica-ai/multica/pull/5913)（MUL-5261） | 见链接 | 「Built-in skills and the runtime brief are platform/meta surfaces: they describe Multica's own contracts」 | 平台层拒绝装领域流程 |
| Squads（多 agent + leader 角色 + stage） | 05-15 前 | **留住并迭代；一次同版本回滚** | 05-15 v0.3.1「Squad archive dialog + role editor + transactional DeleteSquad」合入又 Revert（[#2680](https://github.com/multica-ai/multica/pull/2680) / [#2687](https://github.com/multica-ai/multica/pull/2687)）；07-08 成员可自建 squad；07-23「align parent issue status with agent-managed model」；08-07「derive leader role from wire fields, not instructions text」[#6519](https://github.com/multica-ai/multica/pull/6519) | 见链接 | — | 角色从提示词文本改为结构字段，方向是加结构不是减 |
| Autopilots（定时 / webhook 触发） | 04-17 前 | **留住，最活跃** | 06-30 View / Write 权限层；07-17 调度编辑器重做；08-19 配额；09-07「judge every autopilot write as the human it acts for」 | 见 release 列表 | — | — |
| 后端强制的工作流状态机 | 从未有 | **只有 issue 与一份被关闭的外部 PR** | [#1943](https://github.com/multica-ai/multica/issues/1943)（04-30 open）、[#5972](https://github.com/multica-ai/multica/issues/5972)（07-27 open）；外部贡献者 [PR #8012](https://github.com/multica-ai/multica/pull/8012)（09-03，+4,899 −11）09-06 由作者自行关闭，无维护者评论 | #5972：「Multica's workflow orchestration … is purely prompt-based … There is no enforcement layer — no state machine, no step validation」 | 治理停在提示词层不是没人要，而是维护者没接 |
| Onboarding 流程与 starter kit（参照） | — | 删除 | 04-16「remove onboarding flow」[#1175](https://github.com/multica-ai/multica/pull/1175)；05-20「remove starter-content kit」[#2884](https://github.com/multica-ai/multica/pull/2884) | 见链接 | — | 结构性简化，非治理 |

**社区反馈里的信号**：[#6876](https://github.com/multica-ai/multica/issues/6876)（英文）/ [#6875](https://github.com/multica-ai/multica/issues/6875)（中文，08-13，同日被 collaborator Bohan-J 以 duplicate 关闭）「An Indictment of Multica: Human Bureaucracy Wearing an AI Skin」：「Every Issue receives preconditions, state transitions, acceptance gates, a Reviewer, test steps, and handoff records … Then the whole thing starts getting stuck」「A screen full of `in_review` simply hands all the work back to the human」「When I used Codex directly, I gave it a task and it … finished the job」。Bohan-J 回复：「坦白来说真实企业的工作场景里本来就是这样的，如果一个结果没有人为此负责，那么出问题的时候怎么办呢 … 接受批评，我们也很难做一款产品符合每个人的预期，你可以再去试试其他产品」。其他用户：52216108「我们白天人在的时候，人自己用终端肯定 … 用蒙迪卡最多的还是晚上人下班之后 … 第二天早上收菜」；Pegasucc「哲学问题：human-in-loop 是结构性强制的；UX 问题：loop 被表达得极差 … 单个自治 harness（Codex / Claude Code）确实吊打编排层 … 编排产品的真正用武之地是无人值守的异步 / 批量」。反向声音同样在：#5972 要状态机、[#3216](https://github.com/multica-ai/multica/issues/3216) 要状态变更触发 autopilot、[#6454](https://github.com/multica-ai/multica/issues/6454) / [#2559](https://github.com/multica-ai/multica/issues/2559) 要自定义看板列。

**留住并持续迭代的**：issues 与看板、Squads、Autopilots、默认 `in_review` 流程（维护者明确捍卫）。

### Helio（基线 0.5.0-alpha 官网快照 08-23；heliox `f0c8b46c` v0.2.62 08-20 → v0.2.94 09-03）

闭源 SaaS，helio.im/changelog 不存在（404），公开可查的只有官网文档与三个外围仓库；heliox marketplace 的 59 个提交全是 publish。

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| 「closing a task is a human-only step」（关单人类专属） | 早期 helio.im/product 文案 | **文案下线；工具面把 `task done` 给了 agent** | 基线前（考古见 [helio.md](./workbench/helio.md)：当前页面全文与 meta 已无此句） | heliox task skill：「Prefer closing (`--status done\|cancelled`)」；done 流程「comment aborts and leaves the task open, then the status flips to done」 | — | 本次样本里唯一「营销层承诺的人类门 → 实现层交给 agent」的软化；`in_review` 只剩「等请求方看」的语义 |
| Task 状态机（open / in_progress / blocked / in_review / done / cancelled） | — | **留住不变** | — | 08-20 → 09-03 的 [task SKILL.md](https://github.com/heliohq/marketplace/blob/f0c8b46c743c2e49e776619865696d23d2d93593/heliox/skills/task/SKILL.md) 对比：状态列表逐字不变，变化只有 `--channel` 从必填改可选（org 级任务）、描述改 Markdown 并可文档化 | — | — |
| 审批（Approve / Deny 进频道与 Inbox、remember windows、凭证策略） | — | **留住** | — | [控制文档](https://www.helio.im/docs/ai-teammates/control)：「Approve — the teammate takes the action」「Deny — the teammate stops and is told why」 | — | 无弃用或 beta 标记 |
| heliox skill 面 | 基线 19 个 | **只增不减** | 09-03 多出 `automation-recorder`；`vault-approval`、`channel-charter-creator`、三个 automation skill 两端都在 | [skills 目录](https://github.com/heliohq/marketplace/tree/main/heliox/skills) | — | — |
| ship（机械 stop-gate、phase-guardrail 角色隔离） | 07-04 前 | **两个月无提交** | 最后提交 2026-07-04「docs: remove stale hn-post.txt」 | [heliohq/ship](https://github.com/heliohq/ship) | — | 开源外围里最「治理」的一件停更；是否被产品内吸收未查到 |

**社区反馈里的信号**：未查到 HN / Reddit 讨论；三个外围仓库没有用户关于治理的 issue。

**留住并持续迭代的**：Tasks、审批收件箱、Automation 三元归约、Vault。

### Cumora（基线 `main@bd8dba8e` 08-22；仓库 08-17 公开，到 09-06 v0.16.2 共 255 个提交）

单人主导，公开只有三周，样本太年轻；记录的是投入方向而不是消亡。

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| Shipping 八态状态机（DB CHECK、gate 谓词、构建者不得验收自己） | 08-17 随开源首发（私有史未公开） | **留住，零迭代** | — | [`shipping-router.ts`](https://github.com/yetone/cumora/commits/main/server/src/api/shipping-router.ts)、[`ShippingView.tsx`](https://github.com/yetone/cumora/commits/main/src/desktop/ShippingView.tsx)、[`docs/SHIPPING.md`](https://github.com/yetone/cumora/commits/main/docs/SHIPPING.md) 全史各只有开源首发那 1 个提交；v0.4.0（08-26）只改了暗色模式 | — | 三周没有功能提交也没有 issue；不能判停滞，但它不是维护者当前投入方向 |
| Kanban（agent 可 claim 卡片） | 同上 | **留住并迭代** | 08-27 外部贡献者修「assign 卡片时丢 wake」[#79](https://github.com/yetone/cumora/pull/79)；08-29 [`4f4f2460`](https://github.com/yetone/cumora/commit/4f4f2460)「give board columns a meaning, and make claim advance the card」；08-29「carry Kanban briefs through wakes」[#104](https://github.com/yetone/cumora/pull/104) | 见链接 | 4f4f2460：「the board sat at Todo 2 / Doing 1 / Done 0 while the work was finished and delivered in chat」「a wrong guess would silently reclassify someone's workflow in every existing deployment」 | 方向是让状态机更机械（列语义机器可读、claim 只前进不后退、非默认列名不自动归类），不是拆 |
| Triage / 协同门 | 08-17 | **留住** | 08-29「make Gemini pairing and triage safe」[#101](https://github.com/yetone/cumora/pull/101) | 见链接 | — | — |

**社区反馈里的信号**：issue 多为 i18n、登录、平台 bug；无治理或复杂度抱怨。

**留住并持续迭代的**：Kanban（活跃）、triage；Shipping 留住但静止。

### first-tree（基线 v0.5.20 08-11 / `main@f0d46f9e` 08-14；发布到 v0.5.22 08-25，最后 push 09-03）

141 星；497 个 issue 里前五位作者都是团队成员（bestony 139、Gandy2025 121、liuchao-001 98、baixiaohang 58、yuezengwu 37），issue 区是内部工单，外部声音接近零。治理面（Context Tree 归属、Need you、Context Reviewer）全部留住；被删的是集成层和结构层，且每次退役都写进 release notes。

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| Need-Human-Attention → Request / Need you 队列 | 05-28 [v0.5.3](https://github.com/agent-team-foundation/first-tree/releases/tag/v0.5.3) NHA primitive | **留住并迭代** | — | 06-11 v0.5.6 显式 open-question resolution；07-30 [v0.5.18](https://github.com/agent-team-foundation/first-tree/releases/tag/v0.5.18) 桌面与移动端 Need you 队列、可追问不结单；08-07 v0.5.19 在原聊天内解决 | — | — |
| Context Tree 归属与 Context Reviewer（自动评审） | 06-17 v0.5.7 authorship；06-24 v0.5.9 Context Reviewer PR 自动化 | **留住** | — | 07-17 v0.5.16 review / audit skills 与 QA workflow；07-24 [v0.5.17](https://github.com/agent-team-foundation/first-tree/releases/tag/v0.5.17) Automatic Review 进 Settings → Setup、「exact decision-influence receipts」；08-07 [#2238](https://github.com/agent-team-foundation/first-tree/issues/2238) 评审员重复评论五次（bug，已关） | — | — |
| 定时任务 | 07-24 v0.5.17 | 留住 | — | 「Scheduled jobs are now first-class」 | — |
| agent 重绑定（换 provider / computer） | — | **删除** | 06-17 v0.5.7 | 「Removed agent re-bind (provider/computer switching); `login --override` rotates the local client identity instead」 | 结构收紧：身份不可换绑 |
| agent 最终文本镜像 | — | **先 staging 开关隐藏，三天后退役** | 06-21 v0.5.8「Staging-only toggle to hide agent final-text mirrors」→ 06-24 v0.5.9「retired the agent-final-text mirror」 | 见 release | — | 与 emdash BYOI 同一「先藏开关再删」路径，只是快得多 |
| Feishu / Kael 适配器、legacy GitHub Scan、内置 GitLab skill、手动 commit follow | — | **删除 / 退役** | 06-11、06-10、07-15（「simplifying GitLab operations around native `glab` guidance instead of a bundled GitLab skill」）、08-20 [`d48aacb5`](https://github.com/agent-team-foundation/first-tree/commit/d48aacb5) | 见 release | — | 集成层收敛 |
| 新建 agent 的 Responsibilities 选择器与模板目录 | — | **从常规路径移除** | 08-14 [PR #2339](https://github.com/agent-team-foundation/first-tree/pull/2339)「Simplify ordinary Team agent creation」 | 「remove the Responsibilities picker and Template catalog request from ordinary Team `New agent` opens」 | 「职责」是 first-tree 的治理概念之一，被从默认创建流程拿掉 |
| agent briefing 体积 | — | **缩减** | 07-10 v0.5.13 | 「Reduce generated agent briefing size while preserving the hard collaboration, worktree, Context Tree, and skill rules」 | 提示词层瘦身但保留硬规则 |
| 任务生命周期（Planned / In progress / Done / Cancelled + 责任人） | 从未有 | **只是设计 issue** | [#1868](https://github.com/agent-team-foundation/first-tree/issues/1868)（07-20 open，0 评论） | 「Do not add Needs attention or Failed as durable lifecycle states」 | 团队自己给任务状态机设了上限 |

**社区反馈里的信号**：没有外部「太重」声音。团队自提 [#1890](https://github.com/agent-team-foundation/first-tree/issues/1890)（07-20 open）「Remove tracking chat and auto-follow from first-tree-file-bug」——GitHub App 与组织 1:1 绑定，自动跟踪不可靠，所以删。

**留住并持续迭代的**：Need you、Context Tree、Context Reviewer、Inbox、定时任务、Team Skills。

### LobeHub（基线 v2.2.14 08-16 / canary `18269f43` 08-22；canary 到 v2.2.17-canary.11，09-08）

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| Verify 管道（Agent Run delivery checker）→ Acceptance | 06-08 [#15489](https://github.com/lobehub/lobehub/pull/15489) | **留住、改名、拆成独立应用** | 08-19 `apps/workbench` SSR 应用 [#18473](https://github.com/lobehub/lobehub/pull/18473)；08-20「group acceptance deliveries by project」[#18448](https://github.com/lobehub/lobehub/pull/18448)；08-27「move features/Verify to features/Acceptance」[#18703](https://github.com/lobehub/lobehub/pull/18703) | 见链接 | — | 近六周增速最快的面（[lobehub.md](./workbench/lobehub.md) 复核记录）；「验证」改叫「验收」 |
| 群组编排（supervisor LLM 路由、机械编排状态机） | 2025-12 前 | **七个月无功能提交** | [`groupOrchestration`](https://github.com/lobehub/lobehub/commits/canary/packages/agent-runtime/src/groupOrchestration) 最后功能提交 01-27「group support client agent task」[#11875](https://github.com/lobehub/lobehub/pull/11875)，之后只有 01-29 样式修复，到 09-08 无变化 | 见链接 | — | 研究文件把它列为反面证据（supervisor 当路由权威）；用户 bug [#16118](https://github.com/lobehub/lobehub/issues/16118)、[#16074](https://github.com/lobehub/lobehub/issues/16074)（06-19 / 20）是群组 topic 消失，已关 |
| 7 态 AgentState 状态机 / 看门狗 | — | 留住（迭代情况未单独核查） | — | — | — | — |
| 默认 system prompt 过长 | — | **维护者拒绝精简** | [#11340](https://github.com/lobehub/lobehub/issues/11340)（01-08）closed not_planned | 见链接 | arvinxx：「LobeAI is an agent, need long prompts and tools」 | — |

**社区反馈里的信号**：[#9077](https://github.com/lobehub/lobehub/issues/9077)（2025-09-04）「LobeChat is anything but lightweight」——维护者 arvinxx：「truth, we need to update the description」，同日改 README 关闭。这是「重」的声音，但针对包体，不针对治理功能；没有查到关于 acceptance 或群组编排「太重」的 issue。

**留住并持续迭代的**：acceptance、状态机、context-engine。

### Grok Bot（基线 2026-08-22 官方文档快照；闭源，补 x.ai 09-03 两篇文章与 [重建源码审计](./workbench/grok-bot-reconstructed-audit-20260825.md)）

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| 任务请求五要素与 acceptance criteria | 08-11 发布文档 | **留在文档层** | — | 09-03 [Grok Bot for Enterprise](https://x.ai/news)：「You delegate real tasks … it comes back when the work is done or it needs a decision」 | — | 没有任务对象、生命周期或契约版本（[grok-bot.md](./workbench/grok-bot.md) L3）；08-22 到 09-03 无变化 |
| Auto Review（Require Approval / Always Allow）与七类必审批 | 08-11 | **留住** | — | 08-21「only pull you in for judgment calls」 | 官方：「An approval controls the proposed action. It does not reverse work already completed」 | 相邻产品 Grok Build 9 月放宽自动模式（更多只读 git 命令自动放行、glob 白名单）——方向是放松不是收紧 |
| 组织图 / 多 Bot 网络、append-only 事务日志、TodoWrite、示范教学 | 0.18 客户端里 | **藏在开关后** | — | 重建审计：`sand_agent_network`（org chart）、`sand_memory_dreaming`、`sand_new_transcript_journal`（append-only WAL，默认关）、`sand_teach_by_demonstration`；TodoWrite 只在 `sand_multitask` 开启时提供 | — | 「Bot 间组织结构」这一治理形态在发布版里被开关压着；能否上线未查到 |
| 界面上的监督面 | — | **主动拿掉** | 09-03 [设计文章](https://x.ai/news/designing-grok-bot) | 「By the end of the project, much of the design work involved taking things away. We removed window and panel controls, computer-view options, and agent metadata」「The more prominent we made the computer, the more the product encouraged users to supervise it」 | 维护者明说：越显眼的监督面越诱导人去监工；砍的是观察面，不是审批门 |
| Routine 运行记录 | — | 只留 20 条、删除无撤销 | — | 重建审计 | — | 证据保全在提示词里要求、在实现里不保留 |

**社区反馈里的信号**：未查到「太重」讨论；第三方实测「Work stops just short of done」是验收与执行不分离的代价（研究文件已记）。

**留住并持续迭代的**：Auto Review、审批对象、接管-交还回路、Routines。

### vibe-kanban（BloopAI，284 个 release：2025-06-27 → 2026-04-24；28k 星）

整个产品以看板为主路径，[Show HN](https://news.ycombinator.com/item?id=44533004)（2025-07-11，195 分）是这一类别里最早的爆款。它是样本里唯一「整个产品停业」的案例，停业前两个月的云化又先拿掉了本地模式和单步看板。

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| 看板（To Do / In Progress / In Review / Done，一步拖动） | 2025-06 首发 | **新 UI 后变两步 → 云版默认 → 整体停业** | 01-12 「new UI access」埋点；02-03 [v0.1.2](https://github.com/BloopAI/vibe-kanban/releases/tag/v0.1.2-20260203101746)「We've launched Vibe Kanban Cloud」；02-10「Louis/default cloud kanban」[#2674](https://github.com/BloopAI/vibe-kanban/pull/2674)；02-19「legacy cleanup」[#2790](https://github.com/BloopAI/vibe-kanban/pull/2790)；04-24 [v0.1.44](https://github.com/BloopAI/vibe-kanban/releases/tag/v0.1.44-20260424091429) [PR #3387](https://github.com/BloopAI/vibe-kanban/pull/3387)「Sunset project routes to an export-only page」 | 见链接 | PR #3387：「replaces the interactive Kanban experience with an export-only shutdown screen … the remaining supported capability is exporting data before shutdown」 | — |
| 本地（无云）项目 | 首发即有 | **随云化消失** | 01-20 [v0.0.158](https://github.com/BloopAI/vibe-kanban/releases/tag/v0.0.158-20260120131110)「Remote projects/workspaces schema, Electric Sync」；02-13 文档「remove legacy nav groups, promote Workspaces/Cloud/Settings as default」 | [#3354](https://github.com/BloopAI/vibe-kanban/issues/3354)（04-13）：「I just want to have a local db with projects and issues … hosting a full service … seems overkill」 | — | 云化让单机用户失去入口；停业博文又承诺「transition to a fully local architecture」 |
| 远端版 review | — | **关闭** | 02-11 remote-v0.1.3「remote: disable review」[#2693](https://github.com/BloopAI/vibe-kanban/pull/2693) | 见链接 | 无说明 | — |
| 「等待审批」列、spec 驱动、状态变更 webhook | 从未有 | **开着，无回应** | [#879](https://github.com/BloopAI/vibe-kanban/issues/879)（2025-09-27，用户 10-16 主动说可以提 PR，无人回）、[#1848](https://github.com/BloopAI/vibe-kanban/issues/1848)（01-08）、[#3294](https://github.com/BloopAI/vibe-kanban/issues/3294)（03-29） | 见链接 | — | 治理类需求全部未接 |
| 整个产品 | — | **停业** | 04-10 [博文](https://www.vibekanban.com/blog/shutdown)；04-24 最后提交 | 见链接 | 「Thousands of software engineers use Vibe Kanban every day … but the vast majority are free users and we couldn't find a business model that we could get excited about」「software engineers are spending more time planning and reviewing code, and we wanted to build the best interface for this new way of working」 | 死因是商业模式不是功能；但它是唯一把看板做成主路径且用户量最大的样本，退出后各家比较页（Nimbalyst、Pane、kandev）都拿它做参照 |

**社区反馈里的信号**：Show HN 2025-07：deepdarkforest「whenever i try to parallelize, they clash … It's chaos」；TeMPOraL「It's not "a kanban board"? It's a coding agent orchestrator that's made in the shape of a Kanban board」；doritosfan84 质疑「Why does a kanban board need to see the code or my deploy keys」。[#2730](https://github.com/BloopAI/vibe-kanban/issues/2730)（02-13，0 回复）：「The previous kanban board had a streamlined UX … With the new UI, completing the same workflow now takes 2 steps, which adds friction」。#3354 里 mrThe：「kandev seems way more suitable for everyday use, without stupid requirements of domain name with dozen of subdomains and ton of ssl certs」。

**留住并持续迭代的**：停业前一直在迭代（03-20 批量编辑、03-31「Hide blocked」过滤）；停业后无。

### claude-squad（smtg-ai，8.4k 星；20 个 release：2025-04-03 → 2026-08-20 v1.0.20，2026 年只有 4 个）

tmux + worktree 的 TUI，对照组：从未加过治理功能，也就没有可删的。

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| 治理性功能 | 从未加入 | — | — | 功能史只有 yolo 模式、diff 面板、分支前缀、预设 profile（03-12 v1.0.17）、Terminal 页签（03-01 v1.0.16）、实例排序（05-23 v1.0.18） | README：「Manage instances and tasks in one terminal window」「Review changes before applying them」 | 「任务」在这里等于一个 tmux 会话 |
| 用户需求 | — | — | — | 搜 kanban / workflow / task queue / orchestrat / review 无命中；有的是多仓库（[#89](https://github.com/smtg-ai/claude-squad/issues/89) closed）、桌面版（[#242](https://github.com/smtg-ai/claude-squad/issues/242) closed）、紧凑模式（[#296](https://github.com/smtg-ai/claude-squad/issues/296) open） | — | 用户群不向它要治理 |

**社区反馈里的信号**：无「太重」声音（本来就轻）。

**留住并持续迭代的**：全部现有功能，低频维护。

### Crystal（stravu，28 个 release：2025-06-12 → 2026-02-26 v0.3.5；3.1k 星）

02-26 起停更，README 改「Crystal is now Nimbalyst」；后继者 [nimbalyst/nimbalyst](https://github.com/nimbalyst/nimbalyst)（2025-10-30 建，1.7k 星，102 个 release 到 v0.77.2，09-07）。

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| Project Dashboard | 2025-07-23 v0.1.17 | **随产品终止** | 02-26 [v0.3.5](https://github.com/stravu/crystal/releases/tag/v0.3.5) 迁移弹窗 | 见 release | — | — |
| Setup Tasks Panel（引导式项目设置） | 2025-10-03 v0.3.1 | 同上 | 同上 | — | — | — |
| 看板规划 | 从未有 | **需求开着** | [#87](https://github.com/stravu/crystal/issues/87)（2025-07-20 open，「Kanban Planning System with Claude Plan Mode Integration」） | 见链接 | — | Crystal 未接，后继者接了 |
| 产品整体 → Nimbalyst | — | **迁移** | 02-26 | [官网](https://nimbalyst.com/)：「The visual workspace for Codex and Claude Code」；08-28 v0.75.5 trackers（list / table / board / timeline）、Project Canvas、带投票的 design feedback requests；09-03 v0.76.3 kanban 视图快捷键、commit 逐仓库审批 | — | 后继者把看板（按阶段 backlog / planning / implementing / complete）、trackers、commit 审批做成主路径并持续迭代——是「治理进主路径并留住」的正例，但它是新品不是原品 |

**社区反馈里的信号**：Conductor 的 HN 帖里 SOLAR_FIELDS 夸 Crystal「Does One Thing Really Well … No opinionation about how I organize my code, no custom config files I have to think about」——被夸的正是没有治理。

**留住并持续迭代的**：Crystal 本身无；Nimbalyst 全部。

### Conductor（conductor.build，Melty Labs，闭源 Mac 应用；changelog 2025-07-24 → 2026-09-04 共 197 条）

| 功能 | 加入 | 结局 | 结局时间 | 证据 | 维护者原话 | 我们的推断 |
| --- | --- | --- | --- | --- | --- | --- |
| Plan Mode（Claude）/ Codex Plan Mode | [0.7.3](https://conductor.build/changelog/0.7.3-plan-mode) 2025-08-22；0.12.1 09-27；[0.21.0](https://conductor.build/changelog/0.21.0-plan-mode) 11-10；Codex [0.41.0](https://conductor.build/changelog/0.41.0-tool-approval-codex-plan-mode-beta)（beta）2026-03-19、0.43.0 03-20 | **留住，持续跟进 CLI** | — | changelog 标题 | — | 全部是透传各 CLI 的原生计划模式，与 emdash 最终形态一致 |
| Code Review（diff 评审） | [0.10.0](https://conductor.build/changelog/0.10.0-code-review) 09-08；[0.22.0](https://conductor.build/changelog/0.22.0-code-review-and-historical-diffs) 11-12 历史 diff | 留住 | — | 0.84.0 09-02「Edit files directly in diffs」 | — | — |
| Checkpoints | [0.19.0](https://conductor.build/changelog/0.19.0-checkpoints) 11-05；0.23.2 修复；0.44.0 Codex checkpoints 03-24 | 留住 | — | 见链接 | — | — |
| Approve Plans with Feedback、Tool Approval | [0.25.5](https://conductor.build/changelog/0.25.5-approve-plans-with-feedback-perf-improvements-gh-token-support) 12-09；0.41.0 03-19 | 留住 | — | 见链接 | — | 审批门都是会话级（批计划、批工具），没有任务级阶段门 |
| Todos → Tasks | [0.28.4](https://conductor.build/changelog/0.28.4-todos) 12-30；[0.33.0](https://conductor.build/changelog/0.33.0-tasks-typography) 2026-01-27 | 留住 | — | 0.33.0：「Conductor now supports tasks, a way for Claude to organize its work better and complete longer projects」 | — | 「任务」是 agent 内部的工作清单，不是看板；197 条里从未出现看板、多 agent 编排、规格或契约 |
| 移除 / 弃用 | — | **全史无一条** | — | 197 条里没有 remove / deprecate / sunset 字样；唯一「simplified」是 [0.29.2](https://conductor.build/changelog/0.29.2-vercel-deployments-simplified-thinking-levels)（01-09）「simplified thinking levels」 | — | 没删过，因为没加过结构性功能 |

**社区反馈里的信号**：[Show HN](https://news.ycombinator.com/item?id=44594584)（2025-07-17，228 分）：simonbw「I tried a couple other apps but they seemed to change too much without providing enough value for that change. This feels like just a nice clean simple extension of how Claude code already works」；_1tem「I wanted a simple git worktree manager for my existing, already-checked-out repository. Instead, it requests Github permissions and clones the repo」；freedomben「it's pretty easy to run each in a different tmux window」；greggh、ryanar 转向 par（tmux 包装）；vibudhi「This is a simple problem」。

**留住并持续迭代的**：全部（计划审批、工具审批、checkpoints、评审、Linear / GitHub issues）。

## 未查到

- emdash Discord 内容与 #339 引用的原帖；emdash v0.3.29 / v0.3.30 的发布日期（API 分页只回到 v0.3.37，标签顺序可证）。
- Yoda「Feature 开发闭环」这一字面标题在基线全文无命中，对应文件是 `docs/features/feature-development-workflow.md`（标题「Feature development」）；review 模式最早的提交（已由 CHANGELOG 0.3.6 钉在 06-02 之前）。
- GitHub 搜索接口限流两次，关键词 orchestrat、spec、gate、complicated 的结果为空或已列出，未再穷举。
- Superset：Tasks 何时脱离 Linear 成为本地任务（只在 08-26 的文档里看到「Superset-native tasks」）；Automations 首次加入的日期；Launch HN 之外的 Reddit 讨论。
- Orca：`orchestration run` 退役的独立公告（只在 PR #9925 正文与 skill guide 里各一句）；Agent Dashboard 05-18 重新藏进 Experimental 的理由。
- Multica：acceptance_criteria 字段的加入日期；PR #8012 被作者关闭的原因；#6876 关闭时标为 duplicate 的目标。
- Helio：公开 changelog（helio.im/changelog 返回 404）；ship 停更是否因为 stop-gate 被产品内吸收；任何 HN / Reddit 讨论。
- Cumora：Shipping 的私有开发史（08-17 前）。
- first-tree：GitHub 搜索接口对该仓库返回「cannot be searched」，改用列表接口；外部用户声音基本为零，不构成样本。
- LobeHub：7 态状态机与看门狗在 08-22 后的提交未单独核查。
- Grok Bot：`sand_*` 开关是否已上线；Auto Review 规则 08-22 后是否变化（只有第三方转述）。
- vibe-kanban：02-11「remote: disable review」的原因；停业博文的 HN 讨论（Algolia 未见 story）。
- Crystal：Project Dashboard 与 Setup Tasks Panel 在 Nimbalyst 里的对应物未逐项核对。
- Conductor：changelog 之外没有找到关于计划审批或 checkpoints 被简化的公开讨论。

## 初步归纳

样本 15 家：emdash、Yoda、Superset、Orca、Codeg、Multica、Helio、Cumora、first-tree、LobeHub、Grok Bot、vibe-kanban、claude-squad、Crystal、Conductor。只归纳事实层面的规律，每条给样本数。

1. **加进来的治理或结构性功能被删、退役、藏开关或停滞，9 / 15。** emdash（看板、Best-of-N、多 agent、Plan Mode UI、Docker、BYOI）、Yoda（Feature Delivery、review-loop、三个运行范式）、Orca（自动调度器退役）、Multica（验收字段与触发状态门删除、写入纪律挪进 skill）、vibe-kanban（本地模式、单步看板、远端 review，最后整个产品停业）、LobeHub（群组编排七个月无提交）、Helio（「关单人类专属」文案下线、ship 停更）、Grok Bot（组织图等治理形态藏在 `sand_*` 开关后、监督面主动拿掉）、first-tree（职责选择器移出默认路径、最终文本镜像退役）。没出现的 6 家里，Superset 与 Codeg 是真正的反例，Cumora 太新，claude-squad 与 Conductor 从未加过结构性功能，Crystal 是整个产品换代。
2. **活得短。** 有明确加入与退出时间的自造治理功能：emdash 看板四个月、Best-of-N 四个月、Plan Mode UI 三个月、Docker 两个半月；Yoda Feature Delivery 33 天、review-loop 33 小时、brainstorm 70 天、review 74 天；Orca 调度器三个月；Multica 验收字段不到两个月；first-tree 文本镜像从藏开关到退役三天。11 例里没有一例活过半年。
3. **删得静，5 / 15。** emdash 博文只说「不在这版里」；Yoda 三件大砍不进 CHANGELOG；Orca 调度器退役只在 PR 正文与 skill guide 各一句，release 说明没提；Multica 验收字段只在 commit 正文；vibe-kanban 远端 review 关闭无说明。反例 first-tree：每次退役都写进 release notes（06-10、06-17、06-24、07-15、08-20）。Conductor 197 条 changelog 无一条 remove 字样，但它也没加过结构性功能。
4. **维护者明说的删除理由集中在「没人用 / 不可达 / 残留」，3 家；其余是重写顺手（emdash v1、vibe-kanban 云化）、架构纯度（Yoda「换取架构纯通用」、Orca「Agents still choose … concurrency」）、监督面诱导监工（Grok Bot「The more prominent we made the computer, the more the product encouraged users to supervise it」）。** 明说「没人用」的：Multica「These fields were unused in practice」、Yoda「neither reachable nor editable」、emdash ADR 0007「shrunk to a residue」。
5. **治理从产品状态机退到提示词 / skill 层，7 / 15。** Orca（调度器 → agent-owned loop）、Multica（Issue Metadata 纪律 → skill；工作流至今 prompt-only）、Superset（编排只有一个 plugin skill）、emdash（自建 Plan Mode UI → 透传 CLI 原生）、Yoda（review-loop 状态机 → 通用提示词引擎）、Grok Bot（验收标准只在文档与提示词里）、Conductor（计划 / 工具审批全是 CLI 原生透传）。方向相反的只有 Cumora（列语义从散文变机器可读的 kind 列）、Multica Squads（leader 角色从提示词文本改为结构字段）、Codeg（阶段提示词、排队、失败即停）。
6. **先藏开关再决定，6 / 15；开关的结局两边都有。** emdash BYOI 两次藏后删、ACP 界面藏后转正；Orca Agent Dashboard 四次翻转后转正、daemon 与 per-workspace env 进 Experimental、APFS 转正；Superset Automations 开关 → 付费墙 → 摘开关转正、Plugins 与 chat-v3 在开关后；Grok Bot `sand_*` 四个开关；Codeg「agent 建 to-do」默认关、侧栏行可隐藏；first-tree 文本镜像藏三天后删。转正 4 例（Superset Automations、Orca Dashboard、Orca APFS、emdash ACP），删除 3 例（emdash BYOI、first-tree 镜像、Orca compact mode）。
7. **留住并持续迭代的治理面都绑在外部既有事实上。** 工单镜像（Superset、Orca、emdash、Codeg Repository panel；Multica 自身即工单）5 家；PR / diff 评审门（Superset、Orca、Codeg、emdash、Conductor、first-tree Context Reviewer、Nimbalyst commit 审批）7 家；透传 CLI 原生的 plan / approval（Conductor、emdash、Codeg、Grok Build）4 家；Automations（emdash、Superset、Orca、Codeg、Multica Autopilots、Yoda）6 家。自造的跨任务阶段门或角色编排里留住并迭代的只有 Multica Squads、Orca gates、first-tree Need you、Helio 审批收件箱、Codeg To-dos 状态机 5 例；退出或停滞的有 Yoda Feature Delivery 与 review-loop、Orca 调度器、emdash 编排愿望单、LobeHub group supervisor、Multica 工作流引擎 PR、Helio ship、Cumora Shipping（零迭代）7 例。
8. **看板本身：加入 9 家，删 2，降格 2，另要 3。** 加入：emdash、Yoda、Superset（两张）、Orca、Codeg、Multica、vibe-kanban、Cumora、Nimbalyst。删：emdash、vibe-kanban（停业）。降格：Codeg 改名「To-dos」、Yoda 新板「只观察不接管」。用户另开 issue 要看板而没被接：Orca 两条、Crystal、Superset #1926（stale 关闭）。留住的看板列都绑到机器可判定的状态——Superset 泳道来自 workspace 与 PR 状态、Cumora kind 列、Codeg Queued / Preparing / Reviewed；人工拖动的自由列（emdash、vibe-kanban 单步拖动）是被拿掉的那种。
9. **「太重、只想开个终端」的声音出现在 HN 而不是 issue 区，4 / 15。** Superset Launch HN 三条（「iTerm2 tabs / tmux is plenty … I get overwhelmed」）、Conductor Show HN 四条（「pretty easy to run each in a different tmux window」）、emdash Show HN 两条、Multica 一篇 issue 长文（「Human Bureaucracy Wearing an AI Skin」）。其余 11 家没查到；LobeHub 的「不轻量」抱怨针对包体不针对治理。维护者回应三型：承认取舍（Superset「we have to tow the line of what's useful as a built-in feature vs being very agnostic」）、捍卫人在环（Multica「如果一个结果没有人为此负责，那么出问题的时候怎么办呢」）、不回应（emdash、vibe-kanban #879、Orca 看板 issue、Codeg #532）。
10. **issue 区里要「更多机械约束」的声音多于要「更少」的，7 / 15，且无一被实现。** Multica #5972 要后端状态机（外部 PR 被关）、vibe-kanban #879 等待审批列（志愿者愿写 PR 无人回）与 #1848 spec 驱动、Orca #13360 / #14856 `--spec`、emdash #1462 spec 框架、Crystal #87 看板规划、first-tree #1868 任务生命周期（团队内部并自设上限）、Superset #1926 独立看板（后来做了但 issue 被清扫关）。
11. **重写或迁移是集中删除的时机，5 / 15。** emdash promote v1（1,499 文件）、vibe-kanban 云化（本地模式、单步看板）、Superset v1 → v2（只删旧入口）、Crystal → Nimbalyst（整个产品）、LobeHub 2.0（研究文件记）。Yoda 与 Orca 的删除不靠重写，靠单个大 PR（Orca #9925 +23,286）或顺手提交（Yoda fix(privacy)）。
12. **单人还是团队不是决定因素；商业压力才和整体退出相关。** 单人产品三家结局相反——Yoda 大砍、Codeg 周周加、Cumora 静止；有融资或付费墙的 vibe-kanban（YC，停业「couldn't find a business model」）、emdash（v1 重写）、Superset（付费墙）出现的是整体性动作。Conductor 与 claude-squad 一头一尾都稳定：前者只做 CLI 原生门的透传，后者什么都不做。

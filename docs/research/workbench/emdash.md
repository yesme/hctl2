# emdash

> 类别：② Agent 协作平台 · 证据编号：E-EMDASH<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-emdash"></a>
## E-EMDASH · emdash

### L1 核心价值与产品闭环

emdash 自称「Agentic Development Environment（ADE）」，实际是一个桌面应用：把每个「任务」放进独立的 Git worktree，在里面跑一个编码 Harness（Claude Code、Codex、OpenCode 等 36 家 CLI），让用户同时开很多个、逐个看 diff、挑合适的合进主干。官方 README 一句话概括：「Each task runs in its own Git worktree, so you can explore multiple fixes or features at once, review the diffs, and merge what works.」它给谁用：一个人同时驾驭多个 Harness 的开发者；主机可以是本机，也可以是 SSH 远端。

完整闭环（按官方文档与源码核对）：

1. **注册 Project**：Project 只是桌面端对一个仓库的分组（`CONTEXT.md` 原话：「The desktop-side grouping that organizes tasks around a repository. An app concept only.」）。
2. **Add Task**：起点选分支、Issue（12 家外部工单系统）或 PR，填任务名，选 Harness 与模型。任务名与分支名是两个独立字段，任务名允许任意 Unicode，分支名才受 Git 约束。
3. **宿主建 worktree**：宿主侧的 workspace registry 跑一条前台流水线 `inspect → resolve-base → add-worktree → verify`，然后后台跑 `copy-artifacts / push-branch / fetch-refs`，再按 `.emdash.json` 跑 `prepare / setup / run` 脚本。每一步都落成持久的「生命周期步骤」记录。
4. **Harness 开工**：两条执行路径——PTY 里跑 TUI（可选 tmux 包一层，方便断线重连），或走 ACP 协议的结构化聊天。Harness 状态（working / awaiting-input / completed）只来自它自己的生命周期钩子，emdash 明确不从终端输出猜。
5. **人审**：Diff 视图里暂存、丢弃、提交、推送、「Push & Create PR」；行级草稿评论可以一键格式化后发回给 Harness；Checks 标签同步 GitHub 检查；合并按钮的可用性由 GitHub 的 `mergeStateStatus` 决定，合并调 GitHub API 并拿回 sha。
6. **收尾**：合并后任务与 worktree 仍留在侧栏，由人手动归档或删除（自动归档是两个仍开着的 issue）。

旁路：Automations 按 cron 或手动触发，自动「建任务 → 建 worktree → 启动会话」，一次运行以「会话已启动」记为 done；运行结果可以被人「转成普通任务」。远端有两种：Remote Project（长期 SSH 主机，桌面通过 SSH 转发的 Unix socket 连宿主侧 `workspace-server` 守护进程）与 Remote Task（用户自己的脚本按任务临时开机、用完销毁）。官网另有「Emdash Cloud」页，只写「Cloud workspaces for AI coding agents」与「Get in touch」，未查到价格与可用状态。

它的心智模型可以概括为「桌面协调、宿主执行」（`CONTEXT.md`：「The desktop app coordinates; workspace hosts execute.」）。没有 HCTL 意义上的 Room、Task 契约、Workflow；深的地方全在 L1——宿主权威的 worktree 账本、会话身份与恢复、远端连接监督。

### 审计基线与许可

| 项 | 实测（2026-09-07，`gh api`） |
| --- | --- |
| 仓库 | [generalaction/emdash](https://github.com/generalaction/emdash)，General Action, Inc.（YC W26），官网 emdash.com |
| 钉定基线 | [`main@a5a83795`](https://github.com/generalaction/emdash/tree/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa)（2026-09-06，合并 PR #3127「fix(hosts): recover remote connections after sleep and network interruptions」）；同一 commit 打了 `v1.2.4-canary.90` 标签 |
| 最新正式版 | [`v1.2.3 / 44e1b866`](https://github.com/generalaction/emdash/releases/tag/v1.2.3)（2026-09-02）；主干比它多四天 hosts/pty/wire 修复 |
| 许可 | [`LICENSE.md`](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/LICENSE.md) 原文是 Apache License 2.0，「Copyright 2026 General Action, Inc.」；2026-04-29 之前是 MIT（[切换提交 `1a124974`](https://github.com/generalaction/emdash/commit/1a124974)），Show HN 时网友问的「为什么 MIT」已过时 |
| 规模与活跃 | 5,619 stars / 579 forks；仓库创建 2025-08-28，可见历史最早提交 [`779fa400` 2025-09-12「first commit」](https://github.com/generalaction/emdash/commit/779fa400)；约 9,982 次提交；118 位贡献者，但前五人（arnestrickmann、jschwxrz、Davidknp、janburzinski、rabanspiegel）合计约 9,180 次提交，第六名只有 70 次——是一支五人核心团队加长尾社区；706 个 issue、2,408 个 PR；最近八周每周 63 到 396 次提交，最后推送 2026-09-06 |
| 版本节奏 | 2025-09-23 `v0.2.0` → 2026-04-16 [v1 beta 博文](https://emdash.com/blog/public-v1-beta)（自称「close to a full rewrite」，Kanban、In-app browser、Best-of-N 在重写中被拿掉）→ 2026-04-30 `v1.1.5` → 2026-08-28 `v1.2.0`（远端开发）→ 2026-09-02 `v1.2.3` |

技术栈：TypeScript 单仓（pnpm workspace + Nx），Electron + React + MobX 桌面壳，SQLite（Drizzle + better-sqlite3），node-pty，zod，Vitest + Playwright。仓库结构（[根目录树](https://github.com/generalaction/emdash/tree/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa)）：

- `apps/emdash-desktop/`：桌面应用；`src/core/features/` 按垂直切片组织，文件数最多的是 projects、tasks、conversations、source-control、workbench、workspaces、editor、settings、github、machines、automations、terminals。
- `apps/workspace-server/`：跑在远端的 Node 守护进程，通过 SSH 转发的 Unix socket 暴露与桌面同一套运行时；私有 Wire 协议版本 `7.0.0`，包版本 0.1.4。
- `packages/core/`：运行时（acp、workspace-registry、git、file-search、files、automations、tui-agents、conversations、scripts、terminals、host-settings）、服务（agent-plugins、pty、fs-watch、host-dependencies、session-lifecycle…）、原语三层，运行时之间禁止互相依赖，这条规则用 lint 机械执行。
- `packages/plugins/`：36 家 Harness 的插件定义（其中 23 家可走 ACP），12 家工单系统集成。
- `packages/wire/`：自研的类型化 RPC、live model、job、worker 宿主框架。
- `agents/`（面向 Agent 的架构文档）、`docs/adr/`（8 篇 ADR，带「被谁取代」标注）、`CONTEXT.md`（受控词汇表，每条带「Avoid」）、`.scratch/`（设计稿）。

开发重心估计（README 要求按 2026 年提交路径直方图；本文用两个代理）：最近 300 次提交（2026-08-23 到 09-06）的 Conventional Commits 作用域里 acp 26、release 22、projects 13、workbench 11、chat-ui 10、hosts 8、workspaces 与 workspace-registry 合计 14、conversations 7、terminals 5、pty 5，tasks 只有 4；`core/features/` 文件数也是 projects/tasks/conversations/source-control 领先，automations 与 terminals 中游。两者都说明投入落在「会话 + 现场 + 远端」，任务模型是薄的一层。

官网文档会滚动更新，能力判断以钉定源码为准，官网只补充产品行为。

### 家族归属

按[方法论家族地图](../methodology-landscape-20260824.md#一家族地图已知五族的校准--新七族)，emdash 主要属于**「会话多路复用 / 并行 worktree」族**（claude-squad、crystal、container-use 的同类），特征完全吻合：人保留全部分派与裁决权，工具只管 N 个并行会话的生命周期。证据：

- 任务的默认初始状态是 `in_progress`，状态只能由人在界面上改（[`updateTaskStatus`](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/features/tasks/node/operations/updateTaskStatus.ts) 的唯一调用链是 wire 契约 `tasks.setStatus` → 渲染端 store），没有任何自动推进。
- 没有代码持有的拓扑：一个任务里可以开多个会话，但会话之间没有依赖、没有交接；创始人在 Show HN（2026-02-24）里的说法是 emdash 站在「higher abstraction/task level」，子 Agent 分工交给各家 CLI 自己。
- 「合并编排」需求（[#322](https://github.com/generalaction/emdash/issues/322)）关闭时未见实现；「跨 Agent 比较板」（[#2374](https://github.com/generalaction/emdash/issues/2374)）仍开着，维护者回复只是「会记着」。

为辅的两点：**「系统记录寄生」只寄生了一半**——能把 Linear、Jira、GitHub、GitLab、Asana、Monday.com、Trello、Plane、Forgejo、Featurebase、Plain、Notion 的工单拉进来当上下文，但不回写状态（[#1930](https://github.com/generalaction/emdash/issues/1930) 里维护者原话：「i dont think we do that rn (for any provider)」）；**Automations 是调度器不是 Workflow**——只负责按点开工，不追踪结果。跟 vibe-kanban 相比它更极端：vibe-kanban 好歹有一块从 PR 事实归约出来的看板，emdash 在 v1 重写时把 Kanban 直接拿掉了（[#339](https://github.com/generalaction/emdash/issues/339) 关闭理由：「we temporarily kicked out the kanban board when switching to v1」）。

### 四层设计亮点与边界

| 层 | 真正深入且独特的证据 | HCTL 的采用方式与边界 |
| --- | --- | --- |
| L4 | 很弱。Project 是桌面端分组（「An app concept only」）；没有 Room、没有共享意图账本、没有知识准入。最接近「上下文」的两处：外部 Issue 附到任务后可插入对话，字段固定为 provider、identifier、title、URL、description、status、assignees、project；Library 收纳可复用的 prompts / skills / MCP 服务器，但没有来源与版本。 | 不进入 L4 参考组合。Issue 上下文块的字段集可以当 Context Manifest 里「外部工单来源块」的形状参考，我们再补来源指针与抓取时间；Library 是「无来源知识」的反例。 |
| L3 | Task 有独立身份（不是 worktree），八个生命周期状态 `todo / in_progress / review / done / cancelled / backlog / duplicate / triage`（[`tasks.ts`](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/primitives/tasks/api/tasks.ts)），记录 `statusChangedAt`；任务名与分支名分开、各自可改；Linear 工单自带的分支名优先于本地规则（外部字段权威的一个实例）；`type` 区分 `task` 与 `automation-run`，自动化跑出来的东西要人点「转成任务」才成为任务。TaskConfig 是带版本号的 JSON 列（`initialConversation`、`initialStatus`）。但没有验收标准、没有冻结的契约版本、没有评审或完成凭证；状态改变全靠人点；Kanban 在 v1 被拿掉。 | 只留边界证据与三个小形状：任务名与 Git 标识分离；「进入当前列的时刻」单独记；「运行产物被人采纳后才成为任务」与我们 Run→Task 分离同向。不把它的 Task 或状态枚举当 HCTL Task。 |
| L2 | Automations 的运行有显式状态机 `scheduled → queued → provisioning_workspace → starting_session → done / failed / skipped / cancelled`，转移表写死 from→to（[`transitions.ts`](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/automations/node/runs/transitions.ts)），每次运行冻结一份 `configSnapshot`，守护进程重启时把在途运行标成 `failed(interrupted_by_restart)` 而不是悬着。但 `done` 的定义是「会话已启动」（[`executor.ts`](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/automations/node/runs/executor.ts) 在 `sessionPort.start` 成功后立刻 `markDone`），不追踪 Harness 做了什么。人审在 Diff 视图：行级草稿评论最多 200 条，`formatCommentsForAgent` 后回灌给 Harness；合并可用性从 GitHub `mergeStateStatus`（CLEAN / DIRTY / BEHIND / BLOCKED / HAS_HOOKS / UNSTABLE）加检查汇总派生，`canBypassRequirements` 给有权限的账号留了绕过位。没有 Workflow 图、重试、评审门、法定票数。Best-of-N 曾在 2025-11 上线，v1 重写时拿掉。 | 边界证据为主：「投递成功 / 会话已启动」不等于执行结果，和 Superset 的 Automation 一样。可借三个形状：运行状态转移表与「重启即判失败并写明原因」；每次运行冻结配置快照（我们的 Run Manifest 的最小实例）；行级评论作为「要求修改」裁决的载荷形状（文件、行号、锚点键、内容）。 |
| L1 | **核心参考。**（一）宿主权威的 worktree 账本：文件系统是事实，桌面只留镜像加批注；「正向断言不变量」——扫描失败或部分失败什么都不写，「宿主可达但扫描出错」必须与「宿主不可达」不可区分，绝不能被当成「仓库没有 worktree」；用户手工建的 worktree 被收编（adopt），手工删的记成 missing，永不「修复」；镜像表只有一个写者，动词表 register / adopt / refresh / untrack / revertUntrack / resurrect / annotate / purge；宿主动词是六个 fail-fast 的普通 RPC，不设 outbox；中断的创建留下持久的 `lastCreateOutcome: started`，只能由客户端用同一 UUID 加相同 spec 重放来解决；删除是唯一要在宿主不可达时也存活的意图，用镜像行上的墓碑加「实体无关的对账清扫」（启动、重连、十分钟兜底）完成，失败分 transient / terminal 两类由宿主判，界面给 Retry 与 Untrack-anyway；`deleteWorktree` 永不因脏或未推送拒绝——知情确认是客户端的事（[ADR 0001](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/docs/adr/0001-host-authoritative-workspaces.md)、[0002](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/docs/adr/0002-registry-single-writer.md)、[0005](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/docs/adr/0005-host-workspace-registry-plain-rpc-verbs.md)、[0006](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/docs/adr/0006-tombstone-and-reconcile-deletion.md)）。（二）十二种生命周期步骤乘六种状态的持久账本，同时驱动重放、门控、重试与 Activity 时间线；后台步骤重启后幂等重放，失败步骤只能显式重试。（三）会话身份：Conversation 用 emdash 自己发的 UUID 做身份，Harness 的 session id 只是「最后观察到的恢复句柄」，恢复结果三态 `loaded / replaced-by-new / never-resumed` 诚实入账；ACP 路径是纯状态机（starting / replaying / ready / working / cancelling / closed），会话可以被驱逐再重物化，历史拉不回来时返回 `unavailable: true` 而不是空转录；三层代次围栏（会话代次、供应端连接代次、宿主代次）拒绝迟到结果。（四）Harness 状态只信钩子不猜屏幕：运行时起一个只听 127.0.0.1、随机端口、UUID 令牌的钩子服务器，往各家 Harness 的**用户级**配置里写带版本标记的条目，条目在没有 emdash 环境变量时静默退出；事件归一成 start / stop / error / notification（permission_prompt / idle_prompt / elicitation_dialog 判为 awaiting-input）。（五）远端：每个宿主一个连接监督者（[ADR 0008](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/docs/adr/0008-host-connection-supervisor.md)），状态 idle / connecting / ready / checking / recovering / blocked / paused / stopped，15 秒健康探测、5 秒期限，「SSH 通不等于运行时可用」，「传输恢复不重放已发出的变更」，每次重连必须重新 `initialize` 协商协议版本。（六）tmux 包裹用确定性会话名 `emdash-` 加 PTY 会话 id 的 base64url，历史上限十万行；宿主侧 TUI 进程永不因闲置过期，ACP 会话按闲置策略驱逐。 | 采用为 L1 与 Repo 现场的行为参考：正向断言不变量、收编与 missing、永不修复、单写者动词表、中断创建的持久结果加客户端重放、墓碑加对账清扫、失败分类由宿主判、生命周期步骤账本、会话身份与恢复三态、代次围栏、钩子优先且不猜屏幕、连接监督的「证据不是保证」、协议版本每次重连再协商。它是 Electron/TypeScript，我们通过 Herdr 与自己的 Rust 运行时实现同等语义，不移植实现。 |

### 「恢复」与「宿主权威」实际保证到哪里

跟 Superset 一样，emdash 的几类恢复必须分开说：

- **tmux 开着**：桌面重启或 SSH 断线后按确定性会话名重新 attach，进程一直活着；官方文档同时说清了反面——不开 tmux 时「if Emdash quits or an SSH connection drops, the active agent or terminal process can exit and the terminal view goes blank」，本地 Windows 不包 tmux，退出应用不清理 tmux 会话要自己收拾。
- **宿主守护进程活着**：TUI 进程用 `always` 策略不过期，重连只是接回输出游标；但源码文档明说「A replacement process starts a fresh output generation and terminal display; exact scrollback across process restarts is not promised」。
- **ACP 会话被驱逐后**：只保留唤醒描述与展示快照，重物化时调供应端的 `loadSession`；供应端不能回放时 `loadHistory` 返回 `unavailable: true`，客户端保留旧转录。
- **宿主账本**：只承诺「记录跟随现实」——不重建被人删掉的 worktree、不撤销人手工建的；「宿主不可达」与「扫描失败」都不写任何东西。它没有、也明确拒绝「期望状态 + 收敛控制器」这一套（ADR 0001 拒绝 Kubernetes 式 spec/status，理由是「Emdash is a guest on the host, not the owner」）。

对 HCTL2 的含义：Repo 执行现场（worktree、仓库实例）恰好也是「我们是客人」的地方，可以照抄这套立场；但 Task/Run 的治理事实是我们自己拥有的账本，不能套用「没有期望状态」——两边问题不同。

### 完成判定权

按[方法论审计 §三](../methodology-landscape-20260824.md#三完成判定权横评本次审计最硬的一张表)的口径：HCTL2 只接受有权人类命令，或绑定 Task 的 Run 正常完成后的确定性归约。

| 判什么 | emdash 实况 | 与 HCTL2 立场 |
| --- | --- | --- |
| Task 完成 | 只有人点：wire 契约 `tasks.setStatus` → `updateTaskStatus`，任意状态间自由切换，无前置条件、无验收项、无证据字段。模型没有通道：没有 MCP 服务器、没有本地 API（[#1995](https://github.com/generalaction/emdash/issues/1995)、[#1456](https://github.com/generalaction/emdash/issues/1456) 都还开着）；PR 合并也不自动改状态或归档（[#766](https://github.com/generalaction/emdash/issues/766)、[#1345](https://github.com/generalaction/emdash/issues/1345) 开着）。 | 同向，但是「因为没有别的入口」而不是设计出来的边界：人点 done 不需要任何证据，也没有不可变契约可对照。 |
| Harness「做完了」 | 只来自钩子 Stop 事件 → 会话状态 `completed`，是通知信号，不碰 Task 状态；官方原则「Emdash does not infer agent status from terminal output」。Claude Code 的 Notification 没有类型字段，emdash 用正则 `/permission\|approval/i` 从消息文本猜是权限提示还是空闲提示。 | 同向的好边界：Harness 自述只当信号。正则分类是脆弱点，我们的 Harness 适配器不应靠它。 |
| Automation 运行完成 | `done` = 会话已启动；在途运行遇守护进程重启标 `failed(interrupted_by_restart)`。 | 违反（与 Superset `created` 同类）；重启即判失败并写明原因这一点同向。 |
| 合并 | 由 GitHub 判可合并性，emdash 只展示与转发；`mergePullRequest` 返回 `{ sha, merged }`。有权限账号可 `canBypassRequirements`。 | 平台条件是源头条件不是目标头保证；返回的 sha 没有被绑到 Task 上当凭证。同[代码协作平台调研](../scm-platforms.md)口径。 |
| 现场步骤（worktree 创建、推送、脚本） | 机械事实：每步 `succeeded / failed / skipped / cancelled` 加时间戳与参数，失败只能显式重试，`ABANDON` 类似语义没有但 `cancelled` 与 `failed` 分开。 | 同向，且是全场最像「证据高于自述」的一层——只不过它判的是现场就位，不是工作完成。 |
| 不可变契约 | 无。TaskConfig 是创建时的带版本快照但可改名、可换工单；Automation 的 `configSnapshot` 每次运行冻结，只冻结启动参数。 | 没有可对照的契约，谈不上证据绑定。 |

一句话：emdash 把「Harness 说完了」与「任务完成」分开是对的，但「任务完成」这一侧只有一个按钮，按钮后面没有契约也没有证据。

### 与 HCTL2 的关系与启发

**赛道**：同赛道相邻。它与 Superset、Stably Orca、Codeg 一样是 ② Agent 协作平台，产品中心在 Terminal/现场层，L3 薄、L2 缺、L4 无。HCTL2 与它重叠的是「多 Harness 并行、隔离工作树、人审后合入」这一段体验；不重叠的是意图塑形、承诺契约、施工治理与凭证。

**它做对了、我们还没写清的**：

- 把「我们是宿主上的客人」写成了可检验的不变量（正向断言、收编、missing、永不修复），并配了 ADR 与 prior-art 研究（K8s、Terraform、git worktree 管理记录）。我们的 Repo 现场观察目前只有原则，没有这条「扫描失败什么都不写」的硬规则。
- 「结果是事实不是期望状态」：中断的创建留下 `started` 记录，只能由客户端重放解决，宿主不自己收敛。
- 代次围栏分三层且各有名字（会话代次、供应端连接代次、宿主代次），每层解决一种迟到结果；连接可用性被明确定义为「证据」而不是「保证」。
- 会话恢复结果三态入账，`unavailable: true` 比空转录诚实。
- 受控词汇表 `CONTEXT.md` 每条带「Avoid」，ADR 标注被谁取代——与我们的文档纪律同路，可作写法参考。要注意：词汇表里的 Prompt attempt、Provider dispatch、Outcome unknown、Command receipt、Owner generation 在钉定 commit 里只出现在 `CONTEXT.md`，源码状态机现有的是 Prompt / QueuePrompt / Cancel 等命令与 SessionPhase——设计走在实现前面，引用时要分清。

**我们明确不采用的**：Project 作为桌面分组；Task 没有契约、状态自由切换；「会话已启动」当运行完成；PR 平台的可合并性当合入保证；Library 无来源的知识复用；从消息文本正则猜 Harness 状态；Kanban 与看板身份缺位；Electron/TypeScript 实现本身。

**对四个模块加 Repo 的具体启发**：

- **Project / Room**：只有反例与一个形状。外部工单当上下文时的字段集（provider、identifier、title、URL、description、status、assignees、project）可作 Context Manifest「外部工单块」的最小字段，我们加来源指针、抓取时刻与冻结摘要。Library 提醒我们：Skill 与 prompt 进入 Room 必须带来源与版本，否则就是它这种「可复用但不可追溯」。
- **Task / Kanban**：任务名与分支名彻底分离、各自可改；操作态投影单独记「进入当前列的时刻」（`statusChangedAt`）；Linear 工单自带分支名优先，是「外部字段按字段授权」的现成例子；`automation-run` 要人「采纳」才成为任务，与我们「Run 产出不自动成为 Task」同向，可借「采纳」这个动作名。它拿掉 Kanban 再次印证[来时路结论](../README.md#lineage-scene-map)：看板最晚出现、最易与 worktree 混同。
- **Run / Workflow**：运行状态转移表显式写 from→to 并拒绝非法转移；control 重启后在途 Attempt 标 `failed` 并写 `interrupted_by_restart` 一类原因码，而不是留悬挂态；每次运行冻结配置快照（我们要冻结的远不止启动参数，但形状一致）；行级草稿评论（文件、行号、锚点键、内容，上限 200 条）作为评审席位「要求修改」裁决的载荷参考。边界：它的「done」不能当 Run 完成。
- **Participant / Terminal**：Harness 状态只认结构化钩子、不猜屏幕，与[八家钩子调研](../harness-hooks-20260903.md)一致；钩子安装写用户级配置、带版本标记、无 emdash 环境时静默退出、损坏的配置不碰只记日志——PTY 模式的钩子安装器可以照这套纪律做；Conversation 身份与供应端 session id 分离、恢复三态——对应我们「丢失」的判定与找回入账；连接监督者单一所有权、「SSH 通不等于运行时可用」、健康探测有代次、不重放已发变更——对应 Participant 运行时与 Agency 端点的丢失判定；TUI 永不过期而 ACP 按闲置驱逐——两条执行路径的生命周期策略应分开声明。tmux 的确定性命名是历史对照（我们已用 Herdr），但「运行时对象名由治理 id 确定性派生、可反解」这条值得保留。
- **Repo / Change**：正向断言不变量、收编与 missing、永不修复、单写者动词表、fail-fast 动词加墓碑清扫、失败 transient/terminal 由宿主判、`deleteWorktree` 不因脏拒绝而把知情确认留给上层——整套可作仓库实例与执行现场的观察与销毁参考；生命周期步骤账本（十二步、六态、后台步骤幂等重放、失败显式重试）是「物化 / 封存」步骤记录的形状参考；Git 观察字段（branch、dirty、diffStats、ahead/behind、locked、prunable、headOid、upstream、`branch.<b>.emdash-pr-url` 面包屑）给 ChangeSet 快照一个字段清单，其中用 git config 存 PR 面包屑是不靠数据库把 PR 绑到分支的巧招；worktree 创建的卫生细节（`--no-track`、fetch 时 `--no-tags --no-write-fetch-head --no-auto-maintenance`、preservePatterns 用 copy-on-write 复制、best-effort 回滚、残渣交给收编暴露）可直接进工具箱的实现清单；`mergeStateStatus` 到界面状态的映射是集成预检的展示参考，但只是源头条件。

### 采用结论

**仅参考行为（Behavior reference）为主**。理由：emdash 的价值在 L1/Repo 现场层的一组立场与机制（宿主权威账本、结果即事实、代次围栏、钩子优先、连接监督），这些是语义不是代码；它的实现是 Electron/TypeScript 单体，没有可独立采用的二进制或 SDK——桌面应用不可嵌入，`workspace-server` 是 Node 守护进程且协议私有（Wire v7）、随桌面版本演进；Apache-2.0 允许抄代码，但我们是 Rust 工作区，抄 TypeScript 没有意义。

两处可按需升级的候选，都很小：

- **适配协议候选**：workspace registry 的[生命周期步骤 schema](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/workspace-registry/api/schemas/lifecycle-steps.ts)与[Git 观察 schema](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/workspace-registry/api/schemas/git-observation.ts)的形状，可借给 Repo 现场记录与 ChangeSet 快照。
- **移植有边界的组件候选**：Harness 用户级配置的钩子安装纪律（[标记、版本、守卫命令](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/services/agent-plugins/api/plugins/helpers/hooks.ts)、[读改写 JSON/TOML 不碰用户条目](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/services/agent-plugins/api/plugins/helpers/hook-config.ts)、[事件归一类型](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/services/agent-plugins/api/plugins/capabilities/hooks-types.ts)），逻辑可照搬进我们 PTY 模式的钩子安装器，以八家钩子调研为准。

不采用二进制、不采用 SDK。

明确不采用：Project 作为分组当 HCTL Project；八态自由切换的 Task 状态与「人点即 done」当 HCTL Task 完成；Automation `done` 当 Run 成功；GitHub `mergeStateStatus` 或 `canBypassRequirements` 当合入授权；Conversation、Workspace、Host 等名称与数据库结构进入 HCTL 公开模型；「没有期望状态」的立场套到 Task/Run 治理账本上；用消息文本正则判 Harness 状态；Library 式无来源知识复用；Electron/TypeScript 实现与私有 Wire 协议。

主要证据：

- 官方产品行为：[README](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/README.md)、[文档总览](https://emdash.com/docs/)、[Tasks](https://emdash.com/docs/tasks)、[Issues](https://emdash.com/docs/issues)、[Automations](https://emdash.com/docs/automations)、[Providers](https://emdash.com/docs/providers)、[Diff view](https://emdash.com/docs/diff-view)、[CI checks](https://emdash.com/docs/ci-checks)、[tmux sessions](https://emdash.com/docs/tmux-sessions)、[Remote development](https://emdash.com/docs/remote-development)、[Remote tasks](https://emdash.com/docs/remote-development/remote-tasks)、[Project config](https://emdash.com/docs/project-config)、[v1 beta 博文](https://emdash.com/blog/public-v1-beta)、[Cloud 页](https://emdash.com/cloud)、[Show HN（2026-02-24）](https://news.ycombinator.com/item?id=47140322)
- 架构与词汇：[AGENTS.md](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/AGENTS.md)、[CONTEXT.md 受控词汇表](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/CONTEXT.md)、[核心模块分层](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/agents/architecture/core-modules.md)、[状态所有权](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/agents/architecture/state-ownership.md)、[对账模型 prior art](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/agents/research/reconciliation-models.md)
- ADR：[0001 宿主权威](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/docs/adr/0001-host-authoritative-workspaces.md)、[0002 单写者](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/docs/adr/0002-registry-single-writer.md)、[0005 普通 RPC 动词](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/docs/adr/0005-host-workspace-registry-plain-rpc-verbs.md)、[0006 墓碑与对账](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/docs/adr/0006-tombstone-and-reconcile-deletion.md)、[0007 workspaceHost 退役](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/docs/adr/0007-workspace-host-runtime-retired.md)、[0008 连接监督者](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/docs/adr/0008-host-connection-supervisor.md)
- Task 与状态：[任务类型与八态枚举](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/primitives/tasks/api/tasks.ts)、[TaskConfig 版本化列](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/primitives/tasks/api/task-config.ts)、[wire 契约 `tasks.setStatus`](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/features/tasks/api/wire-contract.ts)、[`updateTaskStatus`](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/features/tasks/node/operations/updateTaskStatus.ts)、[`createTask` 默认 `in_progress`](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/features/tasks/node/operations/createTask.ts)、[分支名解析与 Linear 优先](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/primitives/tasks/api/resolve-task-branch-name.ts)、[桌面 SQLite schema](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/services/app-db/node/schema.ts)
- Automations：[运行 schema 与状态枚举](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/automations/api/run.ts)、[状态转移表](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/automations/node/runs/transitions.ts)、[执行器（会话启动即 done）](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/automations/node/runs/executor.ts)、[运行采纳为任务](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/features/automations/api/automation-run.ts)
- 现场与 worktree：[生命周期步骤 schema](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/workspace-registry/api/schemas/lifecycle-steps.ts)、[记录与运行时叠加层](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/workspace-registry/api/schemas/records.ts)、[创建结果与 git setup](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/workspace-registry/api/schemas/creation.ts)、[Git 观察字段](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/workspace-registry/api/schemas/git-observation.ts)、[前台创建流水线](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/workspace-registry/node/create-worktree.ts)、[强制删除与失败分类](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/workspace-registry/node/delete-worktree.ts)、[worktree 工作流说明](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/agents/workflows/worktrees.md)
- 会话、钩子与状态：[Conversation 记录与恢复三态](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/conversations/api/schemas.ts)、[ACP 运行时架构](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/agents/architecture/acp-runtime.md)、[ACP 纯状态机](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/acp/node/machine/machine.ts)、[钩子服务器](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/tui-agents/node/hooks/hook-server.ts)、[钩子事件管线](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/tui-agents/node/hooks/hook-pipeline.ts)、[Agent 状态归一](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/runtimes/tui-agents/node/runtime/agent-state.ts)、[Claude 钩子与正则分类](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/plugins/src/agents/impl/claude/hooks.ts)、[钩子命令与标记](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/services/agent-plugins/api/plugins/helpers/hooks.ts)、[供应端集成说明（不从终端输出推断）](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/agents/integrations/providers.md)、[桌面侧状态投影](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/main/core/agent-status/agent-status-service.ts)、[会话闲置策略](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/services/session-lifecycle/node/session-lifecycle.ts)、[tmux 会话命名](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/packages/core/src/services/pty/api/tmux.ts)
- 远端：[workspace-server 架构与协议协商](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/agents/architecture/workspace-server.md)、[守护进程说明](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/workspace-server/docs/daemon.md)、[连接监督者验收测试](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/services/hosts/node/connection-supervisor.acceptance.test.ts)
- 评审与合并：[合并按钮状态派生](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/features/source-control/browser/diff-view/changes-panel/components/pr-entry/merge-ui-state.ts)、[PR 契约 `mergePullRequest`](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/services/pull-requests/api/contract.ts)、[行级草稿评论](https://github.com/generalaction/emdash/blob/a5a8379518017f5ddf5a79a7c74ca6b4f1203baa/apps/emdash-desktop/src/core/features/source-control/api/browser/diff-view/stores/draft-comments-store.ts)
- Issue 证据：[#1930 工单状态不回写](https://github.com/generalaction/emdash/issues/1930)、[#766](https://github.com/generalaction/emdash/issues/766) 与 [#1345 合并后不自动归档](https://github.com/generalaction/emdash/issues/1345)、[#339 Kanban 在 v1 被拿掉](https://github.com/generalaction/emdash/issues/339)、[#353 Best-of-N](https://github.com/generalaction/emdash/issues/353)、[#322 合并编排](https://github.com/generalaction/emdash/issues/322)、[#2374 跨 Agent 比较板](https://github.com/generalaction/emdash/issues/2374)、[#1995 本地 HTTP API](https://github.com/generalaction/emdash/issues/1995)、[#1269 本地合并](https://github.com/generalaction/emdash/issues/1269)、[许可切换提交](https://github.com/generalaction/emdash/commit/1a124974)

## 复核记录

- **2026-09-07 · 血缘提示**：[Yoda](./yoda.md)（lovstudio/yoda）是本项目的硬分叉，git 历史完整继承、许可证注明 Portions © General Action；读 Yoda 条目时 L1 内核的证据以本条目为上游。

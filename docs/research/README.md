# docs/research · 实现证据与精选参考组合

> 状态：信息性文档 · 研究快照 2026-08-31；按产品类别组织，单案入同层子目录<br>
> 上级文档：[HCTL2 设计规范](../design/README.md)<br>
> 规则：本目录只说明可行性和复用边界，不定义 HCTL 的领域模型或产品路线。条目是产物（Task 交付的 Artifact）：钉定 commit / 版本与许可，发布后正文不改，只在文末追加复核记录；中间过程与备忘在 `.memo/`。<br>
> 组织方式：本文按**产品类别**给总览（回答"它是什么"）；各条目文件内的研究标签沿用原始脉络记录**证据层级**（回答"我们在哪一层借它"）：L4 → Project / Chat Room，L3 → Task / Kanban，L2 → Run / Workflow，L1 → Harness / Terminal。这两个分类维度互相独立：一个产品重心在 Terminal 的产品可以贡献 L3 证据，反之亦然。

目录规则：研究根目录只放跨候选的归纳、选型汇总、方法论和总索引；以单个产品或项目为对象的 case 放进同层同类子目录。同一 case 的补充源码审计、实测或复核记录与主条目放在一起。当前单案目录为 [`workbench/`](./workbench/README.md)、[`harness/`](./harness/README.md)、[`context/`](./context/README.md)、[`runtime/`](./runtime/README.md)、[`remote-control/`](./remote-control/README.md)、[`build-tools/`](./build-tools/README.md) 和 [`lineage/`](./lineage/README.md)。

## 引用准入

研究样本不是“覆盖四层越多越好”的竞品矩阵。许多产品同时具有 Chat、Task、Workflow、worktree 和终端；只有经过源码和产品行为验证、在某一层形成独特设计亮点的部分，才值得进入该层。一个项目可以在多个层留下不同的深度证据，但不会因为顺带具备某个普通功能就被重复罗列。

参考角色：

- **核心参考**：其产品模型或实现深度直接影响该层的主要方案；
- **专项参考**：在该层的一项关键机制上形成了值得采用的完整设计；
- **直接谱系证据**：前代已经实现并验证的语义切片，用来说明继承与改写边界；
- **行为、实现或边界证据**：分别支持交互契约、可移植机制，或证明某类信号不能越权；
- **观察清单**：研究过且有局部价值，但不进入层内主方案；

同一项目跨层出现时，每一处都必须说明该层独有的亮点与不采用边界；平庸重叠仍然删除。标准、通用库和 Task 来源系统单列。

## 产品归类与借鉴总览

研究样本按产品类别归组，把三个问题分开回答：它是什么（类别与产品重心）、我们在哪一层借它（证据层级与参考角色）、怎么借（复用决策）。产品重心按 2026 年提交路径直方图口径估计——开发投入实际落在四个场景的哪里；宣传口径与投入口径不一致时以后者为准，并在条目内说明。

完整的协作工作台与相邻 Agent 产品统一归档在 [`workbench/`](./workbench/README.md)；本页保留跨类别总表。协议、后端、运行服务、桌面壳和 Context 等专题的跨候选比较留在研究根目录，其中的单个案例归入对应类别子目录。

"怎么借"只有六种决策（定义与偏好顺序见文末[复用决策用语](#复用决策用语)）。与常见问法的对应：直接用它的二进制或服务＝**采用二进制**；用它的库＝**采用 SDK**；借它的 schema/协议形状＝**适配协议**；抄它的代码＝**移植有边界的组件**；借它的思想/阶段/交互＝**仅参考行为**。

### ① Coding Harness · 编码代理本体

HCTL2 驱动 Coding Harness，借 OpenCode、Pi、Kimi Code 的接入协议以及 DeepSeek Harness、Grok Build 的架构边界，不借其产品模型；文件、证据编号与复用决策见[条目索引](#条目索引)。

### ② Agent 协作平台 · 人机混合协作系统

Agent 协作平台与 HCTL2 同赛道，重点比较持久协作、任务承诺、受管运行时和完成判定边界；First Tree、Cumora 的有边界移植，其他平台的行为或协议参考，以及 Termio 的 Manifest/Session URI/监听协议均在[条目索引](#条目索引)及对应单案中记录。

### ③ 独立 Agent 产品 · 单助理 / bot 平台

独立 Agent 产品自己驱动模型循环，HCTL2 只借 OpenClaw、Hermes Agent、Rakazo 与 ZeroClaw 的治理、领取、恢复和适配机制，不借产品形态；详见[条目索引](#条目索引)。

### ④ Context 管理

Context 管理以 MyContext 的成本纪律、LobeHub 的机械组装管道、First Tree 的有来源知识晋升及横向生态审计为行为证据；详见[条目索引](#条目索引)。

### ⑤ 远程操控与会话同步

远程操控把本机 Harness 会话远程化或多端化而不拥有任务语义，Codex Remote Feishu 与九个观察对象只作行为或协议证据；逐对象审计及横向清单见 [`remote-control/`](./remote-control/README.md)，文件与决策见[条目索引](#条目索引)。

<a id="l1-selected-evidence"></a>
### ⑥ 机械后端与基础设施 · 已选依赖与选型对照

不拥有 HCTL 业务决定权的部件中，Dagu、Tuwunel、Vikunja、Herdr、Cinny 采用二进制，Tauri 2 与 UI 通用库采用 SDK，Linear/GitHub 及 [Termio/ATP](https://www.termio.sh/docs/atp) 适配协议，tmux 和旧运行服务候选只留历史对照；补充证据只取 [xterm.js](https://github.com/xtermjs/xterm.js/) 的终端渲染与输入、[WezTerm](https://wezterm.org/cli/cli/index.html) 的外部终端行为、[assistant-ui](https://www.assistant-ui.com/docs/api-reference/primitives/message) 的消息部件、[virtua](https://github.com/inokawa/virtua) 的动态视口、[Rocket.Chat](https://github.com/RocketChat/Rocket.Chat/tree/develop/apps/meteor/client/views/room/MessageList)/[Mattermost](https://github.com/mattermost/mattermost/tree/master/webapp/channels/src/components/dynamic_virtualized_list)/[Zulip](https://github.com/zulip/zulip/blob/main/docs/subsystems/unread_messages.md) 的时间线行为及 [Tiptap/ProseMirror](https://tiptap.dev/docs/editor/extensions/custom-extensions) 的 Composer 扩展，不采用其领域模型，文件与完整边界见[条目索引](#条目索引)和[运维表](#已选外部服务的运维与资源占用)。

### ⑦ 直接谱系

直接谱系只取 HCTL1 的 Git 原生 Seat、隔离栅栏、Verdict、法定票数、Receipt 与失败时默认拒绝测试来解释 [HCTL2 Run 语义内核](../design/run.md)的继承边界；详见[条目索引](#条目索引)。

<a id="lineage-scene-map"></a>
### ⑧ 来时路与场景落点

同赛道产品的来时路决定叙事中心却很少决定工程中心，工程投入普遍漂向 Terminal，Workflow 没有稳定的产品主人，Kanban 往往最晚出现且容易与 worktree 或活工单混同；各对象的来源与场景证据见[条目索引](#条目索引)，方法论工具的落点见[方法论生态审计](./methodology-landscape-20260824.md)。

## 四层如何组合这些亮点

| 层 | 主要设计来源 | HCTL2 的组合方式 |
| --- | --- | --- |
| L4 · Project Room | First Tree 的持久 Chat、显式寻址、可见 handoff、Need You、Context 提升与跨渠道连续性；Multica 的共享 Issue 与私密探索发布边界；Claude Tag 的持久讨论串与临时沙箱分离；OpenClaw 的外部身份和路由 | HCTL2 用规范 Room、一级 Request、Context Manifest 和 Memo 提升流程统一这些经验；外部渠道只作同一 Room 的输入输出面，私聊和执行记录不会自动成为项目知识，协作边的创建权也不随消息作者身份下放给 Agent |
| L3 · Task / Kanban | Codeg 的独立 `WorkTask`、Needs You、评审、后续动作和 Git 恢复；Multica 对 Issue 与单次运行、运行结束与承诺完成的明确分离；Hermes 的领取与重新领取；Linear/GitHub 的原生字段状态 | HCTL2 将长期承诺冻结为 Task Revision，把高频操作状态、外部字段权威和 Task Completion Receipt 分开；启动 Run 与移动卡片分离，完成必须重新校验验收标准和证据 |
| L2 · Workflow / Run / Gate | HCTL1 的版本/证据、领取/隔离栅栏、法定票数和 Receipt；Dagu 的机械图状态与被动等待检查点；Stably Orca 的持久监督协议；Multica 的租约/重试/恢复/归属；ZeroClaw 的审批准入；Herdr、Superset 的边界反例 | HCTL2 自己定义 Workflow Revision、Run Manifest、Obligation、Seat、Attempt、Verdict 和 Receipt；外部机制只补机械推进、可靠领取、消息交付和故障测试，不能用执行者状态或会话传输替代语义治理 |
| L1 · 执行 / 运行时 | Stably Orca 的 PTY 所有权、冷热恢复、远程和交付；Superset 的 `epoch:seq` 重连、守护进程接管和分阶段清理；Herdr 的观察/控制分离；Multica 的多 Harness 能力和不丢代码；DeepSeek Harness 的组合式能力端口；OpenCode/Pi/Kimi/Termio 的接入协议 | Herdr 负责 PTY、终端会话和 Harness 运行；HCTL2 通过 Harness 适配器和 Herdr 适配代码传入参数、记录身份与权限、验收结果。终端状态和厂商会话不能反向定义 Project、Task 或 Run |

这张表是“整合关系”，不是对象映射。每个来源项目只贡献表中写明的机制；L4–L1 是本研究保留的历史标签，最终身份、权限、版本和证据由 HCTL2 的 Project、Task、Run、Agent 四模块定义。

## 条目索引

| 文件 | 对象 | 证据编号 | 类别 | 复用决策 |
| --- | --- | --- | --- | --- |
| [harness-access.md](./harness-access.md) | OpenCode、Pi 与 Kimi Code | E-L1-HARNESS-ACCESS | ① Coding Harness | 适配协议 |
| [deepseek-harness.md](./harness/deepseek-harness.md) | DeepSeek Harness / Cordis | E-L1-DEEPSEEK-HARNESS | ① Coding Harness | 仅参考行为 |
| [first-tree.md](./workbench/first-tree.md) | First Tree | E-L4-FIRST-TREE | ② Agent 协作平台 | 移植有边界的组件 |
| [claude-tag.md](./workbench/claude-tag.md) | Claude Tag | E-L4-CLAUDE-TAG | ② Agent 协作平台 | 仅参考行为 |
| [grok-bot.md](./workbench/grok-bot.md) | Grok Bot 与 Grok Build | E-GROK-BOT | ② Agent 协作平台 | 仅参考行为 |
| [cumora.md](./workbench/cumora.md) | Cumora | E-CUMORA | ② Agent 协作平台 | 移植有边界的组件 |
| [lobehub.md](./workbench/lobehub.md) | LobeHub | E-LOBEHUB | ② Agent 协作平台 | 仅参考行为 |
| [multica.md](./workbench/multica.md) | Multica | E-MULTICA | ② Agent 协作平台 | 仅参考行为 |
| [helio.md](./workbench/helio.md) | Helio | E-HELIO | ② Agent 协作平台 | 核心仅参考行为；开源外围有边界移植并适配协议 |
| [codeg.md](./workbench/codeg.md) | Codeg | E-L3-CODEG | ② Agent 协作平台 | 仅参考行为为主，可按需移植 |
| [stably-orca.md](./workbench/stably-orca.md) | Stably Orca | E-L1-STABLY-ORCA、E-L2-STABLY-ORCA | ② Agent 协作平台 | 仅参考行为为主，可按需移植 |
| [superset.md](./workbench/superset.md) | Superset | E-SUPERSET | ② Agent 协作平台 | 仅参考行为 |
| [openclaw.md](./workbench/openclaw.md) | OpenClaw | E-L4-OPENCLAW | ③ 独立 Agent 产品 | 仅参考行为并适配协议 |
| [hermes-agent.md](./workbench/hermes-agent.md) | Hermes Agent | E-L3-HERMES-AGENT | ③ 独立 Agent 产品 | 仅参考行为 |
| [rakazo.md](./workbench/rakazo.md) | Rakazo | E-RAKAZO | ③ 独立 Agent 产品 | 仅参考行为为主，可按需移植 |
| [zeroclaw.md](./workbench/zeroclaw.md) | ZeroClaw SOP | E-L2-ZEROCLAW | ③ 独立 Agent 产品 | 仅参考行为 |
| [mycontext.md](./context/mycontext.md) | MyContext | E-MYCONTEXT | ④ Context 管理 | 仅参考行为 |
| [codex-remote-feishu.md](./remote-control/codex-remote-feishu.md) | Codex Remote Feishu | E-L1-CODEX-REMOTE-FEISHU | ⑤ 远程操控与会话同步 | 仅参考行为 |
| [remote-control/mindfs.md](./remote-control/mindfs.md) | MindFS | E-L1-MINDFS | ⑤ 远程操控与会话同步 | 仅参考行为 |
| [remote-control/paseo.md](./remote-control/paseo.md) | Paseo | E-L1-PASEO | ⑤ 远程操控与会话同步 | 适配协议 |
| [remote-control/hapi.md](./remote-control/hapi.md) | HAPI | E-L1-HAPI | ⑤ 远程操控与会话同步 | 仅参考行为 |
| [remote-control/happy.md](./remote-control/happy.md) | Happy | E-L1-HAPPY | ⑤ 远程操控与会话同步 | 仅参考行为 |
| [remote-control/remux.md](./remote-control/remux.md) | Remux | E-L1-REMUX | ⑤ 远程操控与会话同步 | 仅参考行为 |
| [remote-control/moshi.md](./remote-control/moshi.md) | Moshi | E-L1-MOSHI | ⑤ 远程操控与会话同步 | 仅参考行为 |
| [remote-control/servercc.md](./remote-control/servercc.md) | ServerCC | E-L1-SERVERCC | ⑤ 远程操控与会话同步 | 仅参考行为 |
| [remote-control/quicktui.md](./remote-control/quicktui.md) | QuickTUI | E-L1-QUICKTUI | ⑤ 远程操控与会话同步 | 仅参考行为 |
| [remote-control/redock.md](./remote-control/redock.md) | Redock | E-L1-REDOCK | ⑤ 远程操控与会话同步 | 仅参考行为 |
| [herdr.md](./runtime/herdr.md) | Herdr | E-L1-HERDR、E-L2-HERDR-BOUNDARY | ⑥ 机械后端与基础设施 | 采用二进制 |
| [harness-hooks-20260903.md](./harness-hooks-20260903.md) | 八家编码 Harness 的「工具调用前」钩子与 ACP 权限请求 | E-L1-HARNESS-HOOKS | ① Coding Harness | 适配协议：PTY 模式用各家原生钩子、ACP 模式用协议权限请求；白名单与检查入口与 harness 无关 |
| [harness-adapters.md](./harness-adapters.md) | Claude Code / Codex CLI / Gemini CLI 的无界面事件、终局、会话与审批 | E-L1-HARNESS-ADAPTERS | ① Coding Harness | 适配协议：三家共用骨架；JSONL 观察与双向审批分开声明，钉 2.1.263 / 0.153.4 / 0.58.0 |
| [sdk/github.md · P2.4 写侧复核](./sdk/github.md#2026-09-06--p24-写侧调用与恢复复核) | gh 发布、合入、正式评审与保护条件 | E-SDK-GITHUB | ⑥ 机械后端与基础设施 | 采用二进制 gh 2.99.0，无本批 SDK 降级项；源头条件写不冒充目标头保证 |
| [sdk/README.md](./sdk/README.md) | 七个供应端客户端层对象（Matrix、Vikunja、Dagu、Herdr、GitHub、Linear，以及作工具箱现场引擎的 Git） | E-SDK-* | ⑥ 机械后端与基础设施 | 官方 SDK > 从接口描述生成 > 手写，逐家判定见子目录 |
| [sdk/git.md](./sdk/git.md) | Git 现场引擎（宿主二进制） | E-SDK-GIT | ⑥ 机械后端与基础设施 | 采用二进制：宿主 git，下限 2.39，不随包；libgit2 / gitoxide 不进依赖树 |
| [libs/README.md](./libs/README.md) | 五处通用机制的现成库加文件监听（JCS、文件锁、SQLite 备份、钥匙串、FTS5、notify） | E-LIB-* | ⑥ 机械后端与基础设施 | 采用 SDK，逐项见子目录；outbox / 租约 / 代次维持自研 |
| [protobuf-rpc.md](./libs/protobuf-rpc.md) | Protobuf 生成链与本地 RPC | E-LIB-PROTOBUF-RPC | ⑥ 机械后端与基础设施 | 采用 SDK：prost / tonic / pbjson；采用二进制：钉 protoc，Buck 原生生成独立 crate |
| [sqlite-migrations.md](./libs/sqlite-migrations.md) | SQLite 停机迁移 | E-LIB-SQLITE-MIGRATIONS | ⑥ 机械后端与基础设施 | 采用 SDK：rusqlite_migration 2.6.0，一致备份后事务升级 |
| [process-compose.md](./runtime/process-compose.md) | 随包服务生命周期 | E-RUNTIME-PROCESS-COMPOSE | ⑥ 机械后端与基础设施 | 采用二进制：1.122.0，经现有 UDS、CLI JSON 与组件动作接入 |
| [workflow-engines.md](./workflow-engines.md) | Dagu 机械状态后端与 workflow 候选复审 | E-L2-DAGU | ⑥ 机械后端与基础设施 | 采用 Dagu 为依赖，其余候选暂缓 |
| [matrix-homeserver.md](./matrix-homeserver.md) | chat server 选型（限时验证） | E-L4-MATRIX-HOMESERVER | ⑥ 机械后端与基础设施 | 采用 Tuwunel 为依赖，Continuwuity 暂缓 |
| [task-backends.md](./task-backends.md) | L3 外部系统与观察清单 | E-L3-VIKUNJA、E-L3-GIT-BUG | ⑥ 机械后端与基础设施 | 采用 Vikunja 为依赖，git-bug 暂缓，Linear/GitHub 适配协议 |
| [scm-platforms.md](./scm-platforms.md) | 代码协作平台市场调研：GitHub、GitLab、Gitea、Forgejo、Gerrit、Bitbucket、Azure DevOps 在合入调用面上的能力（源头校验、目标头保证、队列语义、官方命令行） | E-SCM-PLATFORMS | ⑥ 机械后端与基础设施 | 采用二进制、GitHub 缺省；其余平台暂缓，按能力声明维度按需接入 |
| [tmux-runtime.md](./tmux-runtime.md) | 运行时后端复审 | E-L1-TMUX-RUNTIME | ⑥ 机械后端与基础设施 | 历史选型，不再采用 |
| [agency-runtime-validation-20260829.md](./runtime/agency-runtime-validation-20260829.md) | Herdr 作为 Agent / Terminal 运行服务的验证清单、源码核对与 macOS 实测 | — | ⑥ 外部运行服务与基础设施 · 补充审计 | 采用 Herdr 的验证证据 |
| [agentd-runtime-candidates-20260829.md](./agentd-runtime-candidates-20260829.md) | Agent 运行服务候选的源码复审：Termio、tty7、cmux、Pilotty 及相邻候选 | — | ⑥ 外部运行服务与基础设施 · 补充审计 | 旧结论废止，源码与实测证据保留 |
| [workbench-shell.md](./workbench-shell.md) | Workbench 桌面壳：Electron 与 Tauri 2 | E-WORKBENCH-SHELL | ⑥ 机械后端与基础设施 | 采用 Tauri 2，Electron 为安全网 |
| [hctl1.md](./lineage/hctl1.md) | HCTL1 / yesme/hctl | E-L2-HCTL1 | ⑦ 直接谱系 | 仅参考行为（直接谱系证据） |
| [methodology-landscape-20260824.md](./methodology-landscape-20260824.md) | 方法论工具十二族与完成判定权横评（11 个仓库各钉 commit） | — | 方法论生态 | 逐项适配协议、有边界移植或仅参考行为 |
| [methodology-mattpocock-skills-20260902.md](./methodology-mattpocock-skills-20260902.md) | mattpocock/skills（Skills for Real Engineers）：wayfinder 与 grill 系逐源码审计、完成判定权专项、方法 / 对象 / 机制的分界原则 | — | 方法论生态 · 单对象补充审计 | 仅参考行为；采用为依赖、适配协议、移植组件均为零 |
| [methodology-boundaries-20260902.md](./methodology-boundaries-20260902.md) | 方法论工具阶段边界审计：19 家逐边抽表（18 仓库钉 HEAD + Kiro 文档快照）对照 HCTL2 四模块交接，三类归纳逐条裁决 | — | 方法论生态 · 横向补充审计 | 仅参考行为；四条改写建议（外部机械事实节点、Verdict 分歧落点、验收项形状、Evidence 生产者）均不新增对象 |
| [component-matrix-20260902.md](./component-matrix-20260902.md) | 部件矩阵：content 系统、第一方组件、构建与发行工具、约束层通用机制的目的 / 现选择 / 业界最佳实践 / 借用等级 / 建议（40 行，全部钉版本） | — | 选型汇总 | 维持为主；Reindeer 换官方二进制、运行时生命周期改 Process Compose（候选）、六项机制由手写改 SDK |
| [methodology-sweep-2026h2-20260902.md](./methodology-sweep-2026h2-20260902.md) | 2026 下半年方法论生态限时扫尾：软件工厂新亚种（SSSF、Foreman）、第十二族改名扩容为「机械守卫与验收账本」（unlazy、stop-that-shit、old-coder、procoder）、ProductSpec / icm-architect / better-harness 补行、非方法论高星清单（qm、KiroCrew 转 workbench 线） | — | 方法论生态 · 限时扫尾 | 仅参考行为为主；unlazy 账本格式与 stop-that-shit `ControlEvent v1` 为适配协议候选，`gate-check.mjs` 为有边界移植候选；采用为依赖为零 |
| [context-landscape-20260824.md](./context-landscape-20260824.md) | Context 处理生态四族与快省准横评（链接级） | — | ④ Context 管理 | 仅参考行为 |
| [grok-bot-reconstructed-audit-20260825.md](./workbench/grok-bot-reconstructed-audit-20260825.md) | Grok Bot 0.18 客户端重建源码审计（`a9f633e`），[grok-bot.md](./workbench/grok-bot.md) 的补充证据 | — | ② Agent 协作平台 | 仅参考行为的补充证据 |
| [workbench-shell-reopen-20260826/](./workbench-shell-reopen-20260826/README.md) | Workbench 桌面壳重开调研：GPUI / Iced / Flutter / Web 壳，含 7 份附录 | — | ⑥ 机械后端与基础设施 | 采用 Tauri 2 的选型证据 |
| [install-dotslash.md](./build-tools/install-dotslash.md) | DotSlash 官方 GitHub Action 与本地引导安装审计 | E-TOOL-DOTSLASH | ⑥ 机械后端与基础设施 | Linux CI 锁 Action 提交；macOS CI 与开发机使用摘要锁定安装器 |
| [buck2-change-detector.md](./build-tools/buck2-change-detector.md) | Buck2 Change Detector 的源码、官方制品与失败回退审计 | E-TOOL-BTD | ⑥ 机械后端与基础设施 | 采用官方 `btd` 二进制；不自行构建 `supertd` |
| [jq.md](./build-tools/jq.md) | BTD JSON Lines 解析工具与官方制品审计 | E-TOOL-JQ | ⑥ 机械后端与基础设施 | 采用摘要锁定的官方单文件制品；不依赖宿主预装 jq |
| [github-actions-incremental-validation.md](./build-tools/github-actions-incremental-validation.md) | GitHub Actions 增量重验证 | E-TOOL-GHA-REVALIDATION | ⑥ 机械后端与基础设施 | 采用平台原生 workflow 证据；快进更新增量验证，失败时全量回退 |
| [sdk/matrix.md · P2.2 复核](./sdk/matrix.md#2026-09-06--p22-appservice-实际调用面) | Tuwunel AppService 注册、虚拟用户与加密回读 | E-SDK-MATRIX | ⑥ 机械后端与基础设施 | 采用 SDK：ruma 0.16.0，精确 features 与原生身份方法已核到源码 |
| [sdk/vikunja.md · P2.2 复核](./sdk/vikunja.md#2026-09-06--p22-映射条件写入与生成实验) | Vikunja 2.5.0 的分组映射、条件写反例与生成实验 | E-SDK-VIKUNJA | ⑥ 机械后端与基础设施 | 服务采用二进制；progenitor 0.14.0 直接生成失败，暂缓采用；任务写入无 If-Match 保护 |

## 已选外部服务的运维与资源占用

当前已选的四个外部系统是 Dagu、Tuwunel、Vikunja 和 Herdr。Tuwunel 与 Cinny 合为 Chatroom 解决方案，Static Web Server 只负责提供 Cinny 静态文件。React/Tiptap/xterm.js 等随 Workbench 打包的库不需要独立运维。下表同时保留 tmux 的历史数据。文件大小取官方 release asset 或实际 HCTL2 发行包；RSS 在对应记录注明的平台上用空数据、默认或文中注明的最小配置启动，稳定后读取，且不含 control、Workbench 和 Harness 子进程。

| 模块 | 固定版本与许可 | 发布 / 分发 footprint | 空载实测 / 数据 | 运维判断 |
| --- | --- | --- | --- | --- |
| **Dagu** | [`v2.15.1 / 532c5129`](https://github.com/dagucloud/dagu/releases/tag/v2.15.1)，GPL-3.0-or-later | macOS arm64 archive **45.9 MiB**、binary **148.1 MiB**；Linux amd64 为 48.3/154.6 MiB | `start-all`、coordinator 关闭：**92.4 MiB RSS**；空数据目录约 84 KiB | **低—中**：一个进程、文件备份；主要风险是 adapter/fencing，不是日常运维 |
| **Tuwunel** | [`v1.9.0 / 5b366914`](https://github.com/matrix-construct/tuwunel/releases/tag/v1.9.0)，Apache-2.0 | Linux x86_64 GNU zstd **31.2 MiB**、binary **98.1 MiB**；上游无 Darwin asset；HCTL2 托管的 macOS arm64/x86_64 archive **32.6/35.5 MiB**，binary **77.2/85.2 MiB** | 原生空服务约 **60 MiB RSS**，版本与 health endpoint、非加密/非 federation 配置及整包生命周期均通过 | **中**：单原生进程，不再有 VM；仍须固定低内存配置，并一致备份 RocksDB、media 与 secret |
| **Cinny** | [`v4.12.6 / 33f4ba36`](https://github.com/cinnyapp/cinny/releases/tag/v4.12.6)，AGPL-3.0-only | 官方 Web 发行包 **18.5 MiB**，随包配置后内容 **59.0 MiB**；对应源码归档 **2.1 MiB** | Homeserver 锁定、Chrome 登录页渲染和 HTTP lifecycle 已自动通过，注册、输入与消息交互待人工验收 | **低**：无独立数据库；浏览器存储不是权威事实，客户端不获得 HCTL2 治理权限；上游 SDK 替换期升级需复测 |
| **Static Web Server** | [`v2.44.0 / 27aa3450`](https://github.com/static-web-server/static-web-server/releases/tag/v2.44.0)，MIT OR Apache-2.0 | 官方 Linux x86_64 musl archive **3.48 MiB**、静态 binary **7.99 MiB**；macOS x86_64 为 **3.15/6.83 MiB**，arm64 为 **2.87/5.96 MiB** | Ubuntu 以官方 musl binary 服务 Cinny 时空载 **12.4 MiB RSS**；loopback 绑定、HTML/JSON/WASM/音频 MIME、Range、整包 lifecycle 均通过 | **低**：单进程、无数据库、无需解释器或随包动态库；只作为 Cinny 的内部静态文件服务，不增加执行面类型 |
| **Vikunja** | [`v2.5.0 / ef2200e9`](https://github.com/go-vikunja/vikunja/releases/tag/v2.5.0)，AGPL-3.0-or-later | macOS arm64 full zip **46.9 MiB**、binary **107.3 MiB** | SQLite 空服务 **56.7 MiB RSS**；初始 DB/WAL 约 2.3 MiB | **低**：一个进程 + SQLite；备份 DB、attachments 和 secret，升级前做 migration/restore 演练 |
| **Herdr** | [`v0.8.2 / 9eb5214`](https://github.com/herdrdev/herdr/tree/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c)，Apache-2.0 | 官方 macOS arm64 binary **18.09 MiB**（18.97 MB），x86_64 **20.55 MB**；同时提供 Linux arm64/x86_64 | 空 headless server **17.97 MiB 平均 / 18.39 MiB 峰值**；10 个空闲 workspace **26.20/26.28 MiB**；不含被托管 shell | **低安装 / 中集成**：单二进制、无需自主编译；集成重点是输入所有权、原生输入记录、事件游标和停止结果 |
| **tmux（历史数据）** | [`3.7c / e476c123`](https://github.com/tmux/tmux/releases/tag/3.7c)，ISC | 官方 `tmux-builds` macOS arm64 archive **0.62 MiB**、binary **1.62 MiB**；x86_64 为 **0.65/1.66 MiB** | 一个 server + 10 个 detached session **3.7 MiB RSS**；每 runtime 独立 server 时十个约 **37 MiB RSS** | 不再采用；保留为 Herdr 的功能和资源占用对照 |

换用 Cinny、拆分源码并以 Static Web Server 官方二进制替换内部 HTTP 实现后，Linux x86_64 于 2026-08-27 实测为：用户需要的运行安装包 **152.2 MiB**、解压文件约 **379.3 MiB**；同 Release 单独提供、不参与安装的源码伴随包 **31.6 MiB**，两者合计约 **183.8 MiB**。2026-08-28 的 macOS arm64 包使用官方 tmux：依赖运行包 **144.29 MiB**、源码伴随包 **31.63 MiB**、加入第一方产物的完整运行包 **144.66 MiB**。

2026-08-30 将 tmux 换成 Herdr v0.8.2 并移除第一方 `hctl2-agentd` 后，macOS arm64 实际组装结果为：依赖运行包 **150.39 MiB**（归档内文件合计 **413.94 MiB**）、源码伴随包 **41.13 MiB**，加入 `hctl2-tool` 的完整运行包 **150.58 MiB**（归档内文件合计 **414.43 MiB**）。依赖包与完整 Release 都通过离线安装、五个进程的启停和健康检查、Herdr 协议/API/socket 权限、摘要、许可证与 SBOM 测试。历史四服务 **212.8 MiB RSS** 仍是 tmux 基线；Herdr 的单独与 10-workspace 数据见上表，整套 Herdr 方案的稳态 RSS 待按同口径重测。

## 标准与通用库，不作为产品主参考

- [Agent Client Protocol](https://agentclientprotocol.com/protocol/v1/overview) / [Rust SDK](https://github.com/agentclientprotocol/rust-sdk)：L1 的 Harness 接入标准。
- [Agent Skills](https://agentskills.io/specification)：用于 L4 的 Expertise 选择，以及 L1 的交付与绑定；Skill 只提供指导，不是 Gate。
- [MCP Resources](https://modelcontextprotocol.io/specification/2026-07-28/server/resources) / [Prompts](https://modelcontextprotocol.io/specification/2026-07-28/server/prompts)：传输 Context 和工具信息，不定义 Project/Task 的决定权。
- [React Flow](https://reactflow.dev/) / [Dagre](https://github.com/dagrejs/dagre)：用于 L2 的只读可视化与布局。
- [Tauri 2 capability/permission/scope](https://v2.tauri.app/security/capabilities/) / [IPC](https://v2.tauri.app/concept/inter-process-communication/)：桌面壳权限与跨层数据传输的机械声明；[Electron 安全指南](https://www.electronjs.org/docs/latest/tutorial/security) / [MessagePorts](https://www.electronjs.org/docs/latest/tutorial/message-ports)保留用于安全网发行形态。

## 复用决策用语

所有证据最终只归入六种复用决策：**采用二进制（Adopt binary）**、**采用 SDK（Adopt SDK）**、**移植有边界的组件（Port bounded component）**、**适配协议（Adapt protocol）**、**仅参考行为（Behavior reference）**、**暂缓（Defer）**。前四种有偏好顺序：**跨平台二进制 > SDK > 复制代码 > 借鉴想法**——能借二进制的不借库，能借库的不抄代码，能抄代码的不只借想法；越靠后我们要自己维护的越多。「采用为依赖」是 2026-09-03 前的旧写法，等于前两种的合称。不得给整个产品一个“取代 HCTL”的总分，也不得把参考项目中的 Session、Conversation、Project、Task、Run 名称或内部数据库带入 HCTL 的公开数据结构。

与常见问法的对应关系：直接用它的二进制或服务＝采用二进制；用它的库＝采用 SDK；借它的 schema/协议形状＝适配协议；抄它的代码＝移植有边界的组件；借它的思想/阶段/交互＝仅参考行为。许可证只决定上限（闭源/无许可证/非 OSI 的最多到仅参考行为），不决定选择：许可宽松也可以只借行为。

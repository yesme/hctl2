# docs/research · 实现证据与精选参考组合

> 状态：信息性文档 · 研究快照 2026-08-24（来时路归类补于 2026-08-26，2026-08-27 由单文件拆为一个对象一个文件，2026-08-29 将单个案例按同层同类归入子目录，2026-08-30 远程操控观察清单升格为逐对象审计）<br>
> 上级文档：[HCTL2 设计规范](../design/README.md)<br>
> 规则：本目录只说明可行性和复用边界，不定义 HCTL 的领域模型或产品路线。条目是产物（Task 交付的 Artifact）：钉定 commit / 版本与许可，发布后正文不改，只在文末追加复核记录；中间过程与备忘在 `.memo/`。<br>
> 组织方式：本文按**产品类别**给总览（回答"它是什么"）；各条目文件内的研究标签沿用原始脉络记录**证据层级**（回答"我们在哪一层借它"）：L4 → Project / Chat Room，L3 → Task / Kanban，L2 → Run / Workflow，L1 → Harness / Terminal。这两个分类维度互相独立：一个产品重心在 Terminal 的产品可以贡献 L3 证据，反之亦然。

目录规则：研究根目录只放跨候选的归纳、选型汇总、方法论和总索引；以单个产品或项目为对象的 case 放进同层同类子目录。同一 case 的补充源码审计、实测或复核记录与主条目放在一起。当前单案目录为 [`workbench/`](./workbench/README.md)、[`harness/`](./harness/README.md)、[`context/`](./context/README.md)、[`runtime/`](./runtime/README.md)、[`remote-control/`](./remote-control/README.md) 和 [`lineage/`](./lineage/README.md)。

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

"怎么借"只有五种决策（定义见文末[复用决策用语](#复用决策用语)）。与常见问法的对应：直接用它的 CLI/服务＝**采用为依赖**；借它的 schema/协议形状＝**适配协议**；抄它的代码＝**移植有边界的组件**；借它的思想/阶段/交互＝**仅参考行为**。

### ① Coding Harness · 编码代理本体

HCTL2 驱动的对象；借的是接入协议与架构边界，不是产品模型。

| 项目 | 复用决策 | 证据层级 | 只保留的独特价值 |
| --- | --- | --- | --- |
| [OpenCode](./harness-access.md#e-l1-harness-access) | 适配协议（第一阶段目标 harness） | L1 专项 | OpenAPI + SSE + 类型化 SDK 的服务端优先、多客户端 Harness 控制接口 |
| [Pi](./harness-access.md#e-l1-harness-access) | 适配协议 | L1 专项 | 内嵌 SDK + 严格 JSONL RPC，以及 `steer`/`follow_up` 队列契约 |
| [Kimi Code](./harness-access.md#e-l1-harness-access) | 适配协议 | L1 专项 | ACP/原生能力矩阵和可以验证的降级行为 |
| [DeepSeek Harness](./harness/deepseek-harness.md#e-l1-deepseek-harness) | 仅参考行为 | L1；横切架构边界 | 能力端口、类型化事件和可撤销注册、模型可见只追加日志，以及插件组合的收益与风险 |
| [Grok Build](./workbench/grok-bot.md#e-grok-bot) | 仅参考行为 | L1 | 开源 Rust CLI；作为 ACP agent 可被任何应用托管编排的单向开放（详见 Grok Bot 条目） |

### ② Agent 协作平台 · 人机混合协作系统

与 HCTL2 同赛道的产品。产品重心一列回答"它把工程投入花在哪个场景"。

| 项目 | 产品重心（直方图口径） | 证据层级 · 参考角色 | 复用决策 | 只保留的独特价值 |
| --- | --- | --- | --- | --- |
| [First Tree](./workbench/first-tree.md#e-l4-first-tree) | 多中心：terminal 50 / context 25 / room 20（叙事在 Chat/Context，工程在受管运行时） | L4 核心；L2/L1 专项 | 移植有边界的组件（选择性移植，Apache-2.0） | 持久 Chat、显式寻址与可见 handoff、Context 筛选与治理、Need You、可靠 Inbox、跨渠道协作和托管运行时连续性 |
| [Claude Tag](./workbench/claude-tag.md#e-l4-claude-tag) | room（闭源，行为口径） | L4 行为参考 | 仅参考行为 | 共享且可继续引导的讨论串、按作用域拥有的身份和记忆、持久协作与临时运行时分离 |
| [Grok Bot](./workbench/grok-bot.md#e-grok-bot) | room + terminal（闭源；客户端源码经非授权重建印证，服务端不可见） | L4 行为；L3/L2/L1 边界 | 仅参考行为 | Bot 作为应用原生一等参与者与 handoff 可见性原则、审批对象绑定执行指纹与用户消息代际、"审批不可逆已完成工作"的显式声明、观察-接管-交还回路、动作审计事件 schema；账号级共享云机与凭证、模型分类器当 Gate 且默认只影子运行的反面证据 |
| [Cumora](./workbench/cumora.md#e-cumora) | room 主导（场景内归一化 room 67 / terminal 21 / kanban 7 / workflow 5） | L4/L3/L1 专项；L2 边界 | 移植有边界的组件（选择性移植，MIT） | 唤醒 triage 门与协同门（seen 游标/HELD/hold-token）、Shipping 验收覆盖矩阵与 builder/verifier 分离、BYOA 引擎适配矩阵与保守会话重置、设备配对与不可伪造身份；自由文本证据与全权引擎派发的反面实证 |
| [LobeHub](./workbench/lobehub.md#e-lobehub) | 多中心 room 为主（场景内 room 54 / workflow 16 / terminal 16 / kanban 14；task/验收面增速最快） | L4/L1 专项；L2 边界 | 仅参考行为（非 OSI 许可，只作设计研究） | Context 组装的纯机械处理器管道与增量持久化压缩；外部 Harness 子进程适配器的终局结果契约、状态提取与会话重建；supervisor LLM 路由和默认工具 token 重量的反面实证 |
| [Multica](./workbench/multica.md#e-multica) | 多中心（terminal 37 / kanban 29 / room 13 / workflow 12） | L4/L3/L2/L1 专项 | 仅参考行为（自定义许可排除移植；协议与测试形状可仿） | L4 的 Project/Issue/私聊发布边界；L3 的 Issue 与单次运行分离；L2 的领取、租约、重试、恢复与归属；L1 的多 Harness 能力矩阵和无损 worktree |
| [Helio](./workbench/helio.md#e-helio) | 文档面 workflow 35 / kanban 25 / room 25 / terminal 15；开源外围投入压倒性在工具集成与 workflow 治理 | L4/L2/L1 专项；L3 边界 | 核心仅参考行为；开源外围（anycli/ship）移植有边界的组件 + 适配协议 | 消息面 CAS/cede/receipts/turn 级出处与人批 charter、三元归约与未收尾看门狗、机械 stop-gate 与证据分级、side_effect 安全默认与临时凭证注入；"关单人类专属"营销与实现落差、agent 自行关单的反面证据 |
| [Codeg](./workbench/codeg.md#e-l3-codeg) | terminal 70 / 远程操控 15 / kanban 8 / workflow 4（L3 亮点是其产品的次要模块） | L3 核心；L1/L2 专项 | 仅参考行为为主（Apache-2.0，可按需移植） | 独立异步 `WorkTask`、评审/合并/恢复、ACP/worktree/差异集成，以及自动化与固定流程的边界 |
| [Stably Orca](./workbench/stably-orca.md#e-l1-stably-orca) | terminal 47 / kanban 39 / 交付评审 6 / workflow 4 / room 4 | L1 核心；L2 专项 | 仅参考行为为主（MIT，可按需移植） | L1 的 PTY 所有权、冷热恢复、代际隔离和 worktree/差异/远程/交付；L2 的持久 Run 收件箱、Dispatch 权威、可靠交付、幂等收据和执行者资源生命周期 |
| [Superset](./workbench/superset.md#e-superset) | terminal 60-70 / 远程操控 20 / kanban 5 / workflow 3 | L1 核心；L2 边界 | 仅参考行为（ELv2 排除移植） | L1 的 PTY 守护进程、断线重连与重放、Agent 会话恢复和 worktree 分阶段清理；L2 证明分派与会话传输不等于执行结果或 Workflow 事实 |
| [Termio](#l1-selected-evidence) | 桌面 ADE：terminal 65 / worktree 组织 20 / 移动伴侣 15 | L1 专项 | 适配协议 | Harness Manifest、会话 URI，以及监听/心跳/信号契约 |

### ③ 独立 Agent 产品 · 单助理 / bot 平台

自己驱动模型循环的助理产品；HCTL2 借它们的治理与适配机制，不借产品形态。

| 项目 | 产品重心 | 证据层级 · 参考角色 | 复用决策 | 只保留的独特价值 |
| --- | --- | --- | --- | --- |
| [OpenClaw](./workbench/openclaw.md#e-l4-openclaw) | 个人助理网关（自驱运行时 + 多渠道接入为主；2026-07 起出现编码代理监督面，约 2-3%） | L4 专项 | 仅参考行为 + 适配协议（MIT） | 确定性的多渠道身份与路由、配对/白名单和按渠道降级投递 |
| [Hermes Agent](./workbench/hermes-agent.md#e-l3-hermes-agent) | 自驱 agent 为核；场景内 room 52 / terminal 42 / kanban 3（Kanban 仅占其 2026 投入约 1.5%） | L3 专项 | 仅参考行为（MIT） | 持久 Task/Attempt、原子领取、心跳/回收、依赖推进和多客户端共用内核 |
| [Rakazo](./workbench/rakazo.md#e-rakazo) | 执行运行时 35 / room 27 / 机械后端 24（治理工程严谨度超过产品成熟度） | L2/L1 专项；L4 专项 | 仅参考行为为主（Apache-2.0，可按需移植） | 三层带隔离栅栏的租约与幂等效果账本、挂起前强制 checkpoint 的等人状态、供应商中立的可移植工作区、人/机双租约接管；记忆修订携带 run 级出处；提示词代替策略引擎的反面实证 |
| [ZeroClaw](./workbench/zeroclaw.md#e-l2-zeroclaw) | 自驱运行时为主（SOP 子系统占全史提交路径不足 1%） | L2 相邻实现参考 | 仅参考行为 | SOP 准入、按版本审批与法定票数、恢复和失败时默认拒绝的规则测试 |

### ④ Context 管理

| 项目 | 证据层级 · 参考角色 | 复用决策 | 只保留的独特价值 |
| --- | --- | --- | --- |
| [MyContext](./context/mycontext.md#e-mycontext) | Context 成本纪律对照样本 | 仅参考行为（Elastic 2.0） | 多来源增量采集、零费用常驻检索、RRF 机械融合、三级可见降级；"能用规则就不用模型"的执行实例 |

Context 管理不止一个专门产品：[LobeHub 的 context-engine](./workbench/lobehub.md#e-lobehub)（纯机械组装管道）与 [First Tree 的 Context Tree](./workbench/first-tree.md#e-l4-first-tree)（有来源支撑的知识晋升）在各自条目内，是同一关切的平台内实现。

### ⑤ 远程操控与会话同步

把本机 Harness 会话远程化/多端化的一族；与协作平台的区别是不拥有任务语义。全部只作行为或协议证据；九个产品的逐对象审计（代码与 changelog 级）在 [`remote-control/`](./remote-control/README.md)。

| 项目 | 复用决策 | 只保留的独特价值 |
| --- | --- | --- |
| [Codex Remote Feishu](./remote-control.md#e-l1-codex-remote-feishu) | 仅参考行为（无许可证，不得移植） | 托管会话的连接与路由、输入排队与引导、Request 和重连状态机 |
| [Paseo](./remote-control/paseo.md#e-l1-paseo) | 适配协议（Apache-2.0） | 时间线 epoch+seq 游标与 staleCursor/gap 语义、跨厂商结构化审批消息、steer/interrupt 输入二分、注意力在场路由、adapter 契约（厂商会话为持久层） |
| [MindFS](./remote-control/mindfs.md#e-l1-mindfs) | 仅参考行为（AGPL-3.0） | 外部会话导入与「路径/大小/mtime」游标增量同步、双游标断点重放；工具审批全自动放行的反面证据 |
| [HAPI](./remote-control/hapi.md#e-l1-hapi) | 仅参考行为（AGPL-3.0-only） | 结构化/字节流双投影与「无观众不上传」门控、本地/远程互斥交接、SSE 代次化事件 id 与重放环 |
| [Happy](./remote-control/happy.md#e-l1-happy) | 仅参考行为（MIT） | 端到端加密核实到代码、配对即授钥、seq+localId 幂等与版本 CAS 同步、审批落加密状态断线不丢 |
| [Remux](./remote-control/remux.md#e-l1-remux) | 仅参考行为（MIT） | 移动客户端附着即定位、tmux 原生 ID 为唯一路由身份、前台探活一次性重连、粘贴/提交两段式确认 |
| [Moshi](./remote-control/moshi.md#e-l1-moshi) | 仅参考行为（闭源；线协议公开成文） | 注入前屏幕校验的 verified TUI bridge、摘要/全量数据分通道、在场感知提醒抑制；与 Herdr 深度耦合 |
| [ServerCC](./remote-control/servercc.md#e-l1-servercc) | 仅参考行为（闭源） | 按 sessionId 精确恢复与 session store 反向索引、外部会话收养与离场 keep/stop、每后台期一次的提醒节流 |
| [QuickTUI](./remote-control/quicktui.md#e-l1-quicktui) | 仅参考行为（server/app 闭源；qscreen 为 MIT） | 公开安装验证器 CI 门、能力自描述端点、结构化屏幕帧 attach；隐式尺寸所有权为输入租约反面样本 |
| [Redock](./remote-control/redock.md#e-l1-redock) | 仅参考行为（闭源，无公开代码） | 分阶段输入暂存区、BYO 语音引擎与 CJK、tmux/Herdr/psmux 可插拔持久层 |

### ⑥ 机械后端与基础设施 · 已选依赖与选型对照

不拥有 HCTL 业务决定权的执行部件。已选外部系统为 Dagu、Tuwunel、Vikunja、Herdr 四项；Chatroom 解决方案另随包提供 Cinny 互操作客户端。再加桌面壳与 UI 基础组件。

| 部件 | 复用决策 | 角色 |
| --- | --- | --- |
| [Dagu](./workflow-engines.md#e-l2-dagu) | 采用为依赖 | L2 机械状态后端（Conductor 等七个候选为已评估对照/暂缓） |
| [Tuwunel](./matrix-homeserver.md#e-l4-matrix-homeserver) | 采用为依赖（Continuwuity 备选暂缓） | Chat Room 的 content 系统（Matrix 协议） |
| [Cinny](https://github.com/cinnyapp/cinny/releases/tag/v4.12.6) | 采用官方 Web 发行包作为随包客户端 | Chatroom 的 Matrix 互操作与人工查看界面；不是 Workbench，不拥有治理权威 |
| [Vikunja](./task-backends.md#e-l3-vikunja) | 采用为依赖（限时验证中；[git-bug](./task-backends.md#e-l3-git-bug) 为对照） | Kanban 场景本地 content 后端 |
| [Herdr](./runtime/herdr.md#e-l1-herdr) / [运行服务验证](./runtime/agency-runtime-validation-20260829.md) | 采用为依赖 | Agent 模块 / Terminal 场景的运行服务；验证记录要适配、修改上游或暂不支持的功能 |
| [tmux](./tmux-runtime.md#e-l1-tmux-runtime) | 历史选型，不再采用 | 保留 P0 功能、Harness 兼容性和资源占用数据，作为 Herdr 的对照 |
| [运行服务候选全量复审](./agentd-runtime-candidates-20260829.md) | 旧选型结论已废止；源码与实测证据继续有效 | Termio、cmux、tty7、Pilotty、完整 Agent 产品及产品内 PTY daemon 的发行、协议、测试与 footprint |
| [Tauri 2](./workbench-shell.md#e-workbench-shell) | 采用为依赖（GPUI 原生备选；Electron 安全网） | Workbench 桌面壳 |
| Linear / GitHub | 外部字段权威（[适配协议](./task-backends.md#l3-外部系统与观察清单)） | Task 外部来源系统 |
| xterm.js / virtua / assistant-ui / Tiptap / React Aria / React Flow | 采用为依赖 | UI 基础组件（详见各补充证据表） |

### ⑦ 直接谱系

| 项目 | 证据层级 | 角色 | 只保留的独特价值 |
| --- | --- | --- | --- |
| [HCTL1 / yesme/hctl](./lineage/hctl1.md#e-l2-hctl1) | L2 | 直接谱系证据 | Git 原生 Seat 领取与隔离栅栏、精确 Verdict 与法定票数、可重放 Receipt 和失败时默认拒绝的测试集 |
| [HCTL2 Run 语义内核](../design/run.md) | L2 | 原生语义核心 | 与版本和证据绑定的 Run、Seat 候选切换、法定票数、重新过 Gate 和 Receipt |

这个归类按产品类别分组、按"设计亮点"取证据，不按产品名排他。Codeg 可以同时贡献 L3 的独立 Task 生命周期和 L1 的集成边界；Stably Orca 同时贡献 L1 的执行连续性与 L2 的持久监督协议；Herdr 的主要实现价值在 L1，其状态仲裁还为 L2 提供"运行信号不等于语义完成"的边界证据。任何项目若只顺带拥有 Project、Agent 或 Board 概念，仍不会因此进入对应层。

方法论工具生态（spec 驱动、任务图驱动、流程/角色模拟、轻量纪律、编排器等家族：openspec、spec-kit、Kiro、beads、Taskmaster、vibe-kanban、BMAD、MetaGPT、agent-os、GSD、Gas Town）的逐仓库源码审计与借鉴决策记录在 [方法论生态审计备忘录](methodology-landscape-20260824.md)；其结论（不采用为依赖、重借 schema、少量移植、大量借阶段）与本文准入相互独立，未立正式条目。

<a id="lineage-scene-map"></a>
### ⑧ 来时路与场景落点

同赛道产品的产品重心不是随机分布的：一个产品从哪条路走来，决定它把界面和叙事放在四个场景（Chat Room / Kanban / Workflow / Terminal）的哪一个；而提交直方图显示的工程投入又常常落在另一个场景。下表把这两件事分开写。百分比沿用上文"产品重心（直方图口径）"一列，闭源产品按行为口径。

| 来时路 | 产品 | 叙事中心 | 工程中心（直方图） | 后来长出的第二场景 |
| --- | --- | --- | --- | --- |
| 聊天客户端 / 助理网关 | [LobeHub](./workbench/lobehub.md#e-lobehub)（LobeChat 原地演化） | Room | room 54 / workflow 16 / terminal 16 / kanban 14 | task/验收 workbench 是增速最快的面 |
| | [Cumora](./workbench/cumora.md#e-cumora)（agent 当一等队友的团队聊天） | Room | room 67 / terminal 21 / kanban 7 / workflow 5 | Shipping 八态状态机往 Kanban 长 |
| | [Hermes Agent](./workbench/hermes-agent.md#e-l3-hermes-agent) | Room | room 52 / terminal 42 / kanban 3 | Kanban 占 1.5%，2026-04 才出现 |
| | [OpenClaw](./workbench/openclaw.md#e-l4-openclaw) / [ZeroClaw](./workbench/zeroclaw.md#e-l2-zeroclaw) | Room（多渠道） | 自驱运行时 | OpenClaw 2026-07 起出现 workboard/worktrees 页，约 2-3% |
| | [Claude Tag](./workbench/claude-tag.md#e-l4-claude-tag)（寄生 Slack） | Room | 闭源，行为口径 | Checklist/Routine 只是投影 |
| 带电脑的 Bot（IDE 厂商的云 agent 转助理） | [Grok Bot](./workbench/grok-bot.md#e-grok-bot)（Cursor 栈） | Room + Terminal（Agent Computer 视图） | 闭源；客户端源码印证 | 没有任务对象，L3 只有隐藏的 TodoWrite |
| | [Rakazo](./workbench/rakazo.md#e-rakazo)（Grok Bot 自托管替代） | Room（一 bot 一线程） | 执行运行时 35 / room 27 / 机械后端 24 | 没有 L3 |
| 工单 / Issue 跟踪器 | [Multica](./workbench/multica.md#e-multica) | Kanban（Project/Issue） | terminal 37 / kanban 29 / room 13 / workflow 12 | 四场景全触及且无一超四成，唯一的均衡样本 |
| | [Helio](./workbench/helio.md#e-helio)（票据 + 审批 + Vault） | Kanban + 审批收件箱 | 文档面 workflow 35 / kanban 25 / room 25 / terminal 15；开源外围集中在工具集成与 workflow 治理 | ship 插件的 stop-gate 与证据分级是 TDD/eval 那条路在协作平台里的落点 |
| 终端复用 / 并行 worktree | [Superset](./workbench/superset.md#e-superset) | Terminal | terminal 60-70 / 远程 20 / kanban 5 / workflow 3 | Board 分栏从 PR 与运行信号派生 |
| | [Herdr](./runtime/herdr.md#e-l1-herdr) | Terminal | ≈100% terminal | 无 room/kanban/workflow 目录 |
| | [Termio](#l1-selected-evidence) | Terminal（桌面 ADE） | terminal 65 / worktree 20 / 移动 15 | — |
| | [Codeg](./workbench/codeg.md#e-l3-codeg) | Terminal（多 Agent 会话工作台） | ACP/会话面约 70 / WorkTask 看板 8 | 本文借的 L3 是它的次要模块 |
| | [Stably Orca](./workbench/stably-orca.md#e-l1-stably-orca) | Terminal + worktree Board | terminal 47 / kanban 39 | 看板卡片身份仍是 worktree；有 Decision Gate |
| 上下文 / 知识工程 | [First Tree](./workbench/first-tree.md#e-l4-first-tree)（Context Tree） | Room + Context | terminal 50 / context 25 / room 20；kanban≈0 | 叙事在 Chat/Context，工程在受管运行时 |
| 远程操控 | [Codex Remote Feishu](./remote-control.md#e-l1-codex-remote-feishu) 及[观察清单](./remote-control.md#观察清单远程操控与会话同步) | Terminal（Room 只是投影） | 会话与中继 | 刻意不拥有任务语义 |

方法论工具没有产品形态，但来时路最清楚，落点按驱动机制归：spec 驱动（Kiro、spec-kit、OpenSpec）落 Kanban，只是"文件里的任务清单"而没有板；任务图驱动（beads、Taskmaster）落 Kanban；编排器（Gas Town、vibe-kanban）落 Workflow + Terminal；群体自治（ruflo）与流程/角色模拟（BMAD、MetaGPT）落 Workflow；TDD/eval（tdd-guard）落 Workflow 的门但活在 harness hook 里；会话复用（claude-squad、crystal）落 Terminal。三个 Workflow 落点的家族表面相似、内里不同，辨析见[方法论生态审计备忘录](methodology-landscape-20260824.md)的 2026-08-26 补记。

从表里读出三条规律：

1. **来时路决定叙事中心，几乎不决定工程中心。** 工程重心不约而同漂向 Terminal：First Tree 讲 Chat/Context 但一半提交在受管运行时，Multica 讲 Issue 但 terminal 高于 kanban，Codeg 七成在会话面，LobeHub 与 Cumora 都是聊天出身却各自长出 harness 适配层与 BYOA 守护进程。把 harness 真跑起来、跑得住是最难的工程，谁都绕不过。
2. **Workflow 场景没有产品主人。** 每家都有一个 routine/automation/autopilot，投入都不超过 16%，而且都是"定时触发 + 至少一次投递"的薄层，没有版本、没有 Gate。真正把 Workflow 当中心的全是方法论工具，它们又没有产品面。这与方法论备忘录的完成判定权横评是同一事实的两面。
3. **Kanban 是最晚长出、也最容易长歪的场景。** 聊天出身的最后才补看板；终端出身的看板卡片身份是 worktree；工单出身的两家看板是叙事中心，但任务是可随时改写的活行，没有冻结契约。

对 HCTL2 的含义：没有一家的四个场景是同时有主人的，四场景齐备本身就是空位；HCTL2 押的差异（Kanban 的契约冻结、Workflow 的完成判定权）恰好落在两个没人做主的场景上。这一节只回答"它们从哪来、落在哪"，不改变各条目的复用决策。

## 四层如何组合这些亮点

| 层 | 主要设计来源 | HCTL2 的组合方式 |
| --- | --- | --- |
| L4 · Project Room | First Tree 的持久 Chat、显式寻址、可见 handoff、Need You、Context 提升与跨渠道连续性；Multica 的共享 Issue 与私密探索发布边界；Claude Tag 的持久讨论串与临时沙箱分离；OpenClaw 的外部身份和路由 | HCTL2 用规范 Room、一级 Request、Context Manifest 和 Memo 提升流程统一这些经验；外部渠道只作同一 Room 的输入输出面，私聊和执行记录不会自动成为项目知识，协作边的创建权也不随消息作者身份下放给 Agent |
| L3 · Task / Kanban | Codeg 的独立 `WorkTask`、Needs You、评审、后续动作和 Git 恢复；Multica 对 Issue 与单次运行、运行结束与承诺完成的明确分离；Hermes 的领取与重新领取；Linear/GitHub 的原生字段状态 | HCTL2 将长期承诺冻结为 Task Revision，把高频操作状态、外部字段权威和 Task Completion Receipt 分开；启动 Run 与移动卡片分离，完成必须重新校验验收标准和证据 |
| L2 · Workflow / Run / Gate | HCTL1 的版本/证据、领取/隔离栅栏、法定票数和 Receipt；Dagu 的机械图状态与被动等待检查点；Stably Orca 的持久监督协议；Multica 的租约/重试/恢复/归属；ZeroClaw 的审批准入；Herdr、Superset 的边界反例 | HCTL2 自己定义 Workflow Revision、Run Manifest、Obligation、Seat、Attempt、Verdict 和 Receipt；外部机制只补机械推进、可靠领取、消息交付和故障测试，不能用执行者状态或会话传输替代语义治理 |
| L1 · 执行 / 运行时 | Stably Orca 的 PTY 所有权、冷热恢复、远程和交付；Superset 的 `epoch:seq` 重连、守护进程接管和分阶段清理；Herdr 的观察/控制分离；Multica 的多 Harness 能力和不丢代码；DeepSeek Harness 的组合式能力端口；OpenCode/Pi/Kimi/Termio 的接入协议 | Herdr 负责 PTY、终端会话和 Harness 运行；HCTL2 通过 Harness 适配器和 Herdr 适配代码传入参数、记录身份与权限、验收结果。终端状态和厂商会话不能反向定义 Project、Task 或 Run |

这张表是“整合关系”，不是对象映射。每个来源项目只贡献表中写明的机制；L4–L1 是本研究保留的历史标签，最终身份、权限、版本和证据由 HCTL2 的 Project、Task、Run、Agent 四模块定义。

## 条目索引

| 文件 | 对象 | 证据编号 | 类别 |
| --- | --- | --- | --- |
| [harness-access.md](./harness-access.md) | OpenCode、Pi 与 Kimi Code | E-L1-HARNESS-ACCESS | ① Coding Harness |
| [deepseek-harness.md](./harness/deepseek-harness.md) | DeepSeek Harness / Cordis | E-L1-DEEPSEEK-HARNESS | ① Coding Harness |
| [first-tree.md](./workbench/first-tree.md) | First Tree | E-L4-FIRST-TREE | ② Agent 协作平台 |
| [claude-tag.md](./workbench/claude-tag.md) | Claude Tag | E-L4-CLAUDE-TAG | ② Agent 协作平台 |
| [grok-bot.md](./workbench/grok-bot.md) | Grok Bot 与 Grok Build | E-GROK-BOT | ② Agent 协作平台 |
| [cumora.md](./workbench/cumora.md) | Cumora | E-CUMORA | ② Agent 协作平台 |
| [lobehub.md](./workbench/lobehub.md) | LobeHub | E-LOBEHUB | ② Agent 协作平台 |
| [multica.md](./workbench/multica.md) | Multica | E-MULTICA | ② Agent 协作平台 |
| [helio.md](./workbench/helio.md) | Helio | E-HELIO | ② Agent 协作平台 |
| [codeg.md](./workbench/codeg.md) | Codeg | E-L3-CODEG | ② Agent 协作平台 |
| [stably-orca.md](./workbench/stably-orca.md) | Stably Orca | E-L1-STABLY-ORCA、E-L2-STABLY-ORCA | ② Agent 协作平台 |
| [superset.md](./workbench/superset.md) | Superset | E-SUPERSET | ② Agent 协作平台 |
| [openclaw.md](./workbench/openclaw.md) | OpenClaw | E-L4-OPENCLAW | ③ 独立 Agent 产品 |
| [hermes-agent.md](./workbench/hermes-agent.md) | Hermes Agent | E-L3-HERMES-AGENT | ③ 独立 Agent 产品 |
| [rakazo.md](./workbench/rakazo.md) | Rakazo | E-RAKAZO | ③ 独立 Agent 产品 |
| [zeroclaw.md](./workbench/zeroclaw.md) | ZeroClaw SOP | E-L2-ZEROCLAW | ③ 独立 Agent 产品 |
| [mycontext.md](./context/mycontext.md) | MyContext | E-MYCONTEXT | ④ Context 管理 |
| [remote-control.md](./remote-control.md) | Codex Remote Feishu | E-L1-CODEX-REMOTE-FEISHU | ⑤ 远程操控与会话同步 |
| [remote-control/mindfs.md](./remote-control/mindfs.md) | MindFS | E-L1-MINDFS | ⑤ 远程操控与会话同步 |
| [remote-control/paseo.md](./remote-control/paseo.md) | Paseo | E-L1-PASEO | ⑤ 远程操控与会话同步 |
| [remote-control/hapi.md](./remote-control/hapi.md) | HAPI | E-L1-HAPI | ⑤ 远程操控与会话同步 |
| [remote-control/happy.md](./remote-control/happy.md) | Happy | E-L1-HAPPY | ⑤ 远程操控与会话同步 |
| [remote-control/remux.md](./remote-control/remux.md) | Remux | E-L1-REMUX | ⑤ 远程操控与会话同步 |
| [remote-control/moshi.md](./remote-control/moshi.md) | Moshi | E-L1-MOSHI | ⑤ 远程操控与会话同步 |
| [remote-control/servercc.md](./remote-control/servercc.md) | ServerCC | E-L1-SERVERCC | ⑤ 远程操控与会话同步 |
| [remote-control/quicktui.md](./remote-control/quicktui.md) | QuickTUI | E-L1-QUICKTUI | ⑤ 远程操控与会话同步 |
| [remote-control/redock.md](./remote-control/redock.md) | Redock | E-L1-REDOCK | ⑤ 远程操控与会话同步 |
| [herdr.md](./runtime/herdr.md) | Herdr | E-L1-HERDR、E-L2-HERDR-BOUNDARY | ⑥ 机械后端与基础设施 |
| [workflow-engines.md](./workflow-engines.md) | Dagu 机械状态后端与 workflow 候选复审 | E-L2-DAGU | ⑥ 机械后端与基础设施 |
| [matrix-homeserver.md](./matrix-homeserver.md) | chat server 选型（限时验证） | E-L4-MATRIX-HOMESERVER | ⑥ 机械后端与基础设施 |
| [task-backends.md](./task-backends.md) | L3 外部系统与观察清单 | E-L3-VIKUNJA、E-L3-GIT-BUG | ⑥ 机械后端与基础设施 |
| [tmux-runtime.md](./tmux-runtime.md) | 运行时后端复审 | E-L1-TMUX-RUNTIME | ⑥ 机械后端与基础设施 |
| [agency-runtime-validation-20260829.md](./runtime/agency-runtime-validation-20260829.md) | Herdr 作为 Agent / Terminal 运行服务的验证清单、源码核对与 macOS 实测 | — | ⑥ 外部运行服务与基础设施 · 补充审计 |
| [agentd-runtime-candidates-20260829.md](./agentd-runtime-candidates-20260829.md) | Agent 运行服务候选的源码复审：Termio、tty7、cmux、Pilotty 及相邻候选 | — | ⑥ 外部运行服务与基础设施 · 补充审计 |
| [workbench-shell.md](./workbench-shell.md) | Workbench 桌面壳：Electron 与 Tauri 2 | E-WORKBENCH-SHELL | ⑥ 机械后端与基础设施 |
| [hctl1.md](./lineage/hctl1.md) | HCTL1 / yesme/hctl | E-L2-HCTL1 | ⑦ 直接谱系 |
| [methodology-landscape-20260824.md](./methodology-landscape-20260824.md) | 方法论工具十二族与完成判定权横评（11 个仓库各钉 commit） | — | 方法论生态 |
| [context-landscape-20260824.md](./context-landscape-20260824.md) | Context 处理生态四族与快省准横评（链接级） | — | ④ Context 管理 |
| [grok-bot-reconstructed-audit-20260825.md](./workbench/grok-bot-reconstructed-audit-20260825.md) | Grok Bot 0.18 客户端重建源码审计（`a9f633e`），[grok-bot.md](./workbench/grok-bot.md) 的补充证据 | — | ② Agent 协作平台 |
| [workbench-shell-reopen-20260826/](./workbench-shell-reopen-20260826/README.md) | Workbench 桌面壳重开调研：GPUI / Iced / Flutter / Web 壳，含 7 份附录 | — | ⑥ 机械后端与基础设施 |

<a id="l1-selected-evidence"></a>
## L1 精选实现证据

| 证据 | 独特价值 | 边界 |
| --- | --- | --- |
| [Herdr 运行服务验证](./runtime/agency-runtime-validation-20260829.md) | 记录派工、身份、观察、输入、栅栏、停止、恢复和 footprint 测试；补齐 Herdr Apple Silicon 10-pane 与重输出数据 | Herdr 已确定采用；缺失能力必须由 HCTL 适配、明确标为不支持或推动 Herdr 补充，不能另写一套终端服务 |
| [Agent 运行服务候选复审](./agentd-runtime-candidates-20260829.md) | 逐源码核对 Termio、tty7-server、cmuxd-remote、Pilotty、tmux client 库、产品内 daemon 与远程控制候选；统一发行物、RSS、协议和所需修改的统计口径 | 选型结论是 PR #36—#42 前的快照；v0.14.1 后只沿用机制、测试与资源占用证据，当前 Herdr 验证要求见上项 |
| [Termio `d1fdac8…`](https://github.com/termio-sh/termio/tree/d1fdac84046805d4056e082f982e6beb6072b61c) / [ATP](https://www.termio.sh/docs/atp) / [会话控制](https://www.termio.sh/docs/session-control) | Manifest、稳定的 Session URI、监听/心跳/信号，以及带数据结构版本的控制协议 | MIT；ATP 不是 HCTL 或行业通用的传输标准，也不作为跨平台 Backend 的权威实现。2026-08-24 复核：其产品形态实为桌面 ADE（并行 worktree 侧栏 + harness hooks 状态），归 Agent 协作平台类，仍按 L1 专项引用 |
| [Herdr `v0.8.0 / 346411fa`](https://github.com/herdrdev/herdr/tree/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7) / [专项审计](./runtime/herdr.md#e-l1-herdr) | 后台服务持有 PTY；观察与控制、原始输入与 Agent 命令彼此分开；单写者接管；状态信号仲裁与分级恢复 | Apache-2.0；控制方不是持久租约，运行状态不等于领域完成；完整边界见专项审计 |
| [xterm.js](https://github.com/xtermjs/xterm.js/) | 嵌入式终端渲染器，以及 CJK、输入法、无障碍和流量控制 | MIT；只负责前端，不拥有 PTY、进程或 Session |
| [WezTerm](https://wezterm.org/cli/cli/index.html) | 成熟的跨平台外部终端与 CLI | MIT；不嵌入应用，也不把 Mux 协议当作 ABI |
| [tmux `3.7c / e476c123`](https://github.com/tmux/tmux/tree/e476c1230b958df0cb12977517d24b3dc931375b) / [专项复审](./tmux-runtime.md#e-l1-tmux-runtime) | 公开 control mode、稳定 pane ID、headless 查询应答、捕获/转发、退出状态和很小的 native footprint | ISC；历史选型已由 Herdr 取代；三分发目标测试与资源数据继续作为 Herdr 的行为和容量对照 |

## L4 补充证据

| 证据 | 独特价值 | 边界 |
| --- | --- | --- |
| [assistant-ui](https://www.assistant-ui.com/docs/api-reference/primitives/message) | 有明确作用域的 Message/`MessagePart`/Action 渲染器 | 不采用 Thread、运行时、Store、Composer、Cloud 或 Queue |
| [virtua](https://github.com/inokawa/virtua) | 支持动态高度的 React 视口 | 不负责 Room 的顺序、游标或跟随策略 |
| [Rocket.Chat](https://github.com/RocketChat/Rocket.Chat/tree/develop/apps/meteor/client/views/room/MessageList)、[Mattermost](https://github.com/mattermost/mattermost/tree/master/webapp/channels/src/components/dynamic_virtualized_list)、[Zulip](https://github.com/zulip/zulip/blob/main/docs/subsystems/unread_messages.md) | 前插消息、定位到指定消息、未读状态、动态高度和无障碍测试 | 合并为行为证据；不采用其后端或领域模型 |

Tiptap/ProseMirror 是 L4 精选的 Composer 基础组件，不是产品参考项目：[自定义扩展](https://tiptap.dev/docs/editor/extensions/custom-extensions)、[React 节点视图](https://tiptap.dev/docs/editor/extensions/custom-extensions/node-views/react)。

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

所有证据最终只归入五种复用决策：**采用为依赖（Adopt dependency）**、**移植有边界的组件（Port bounded component）**、**适配协议（Adapt protocol）**、**仅参考行为（Behavior reference）**、**暂缓（Defer）**。不得给整个产品一个“取代 HCTL”的总分，也不得把参考项目中的 Session、Conversation、Project、Task、Run 名称或内部数据库带入 HCTL 的公开数据结构。

与常见问法的对应关系：直接用它的 CLI/服务＝采用为依赖；借它的 schema/协议形状＝适配协议；抄它的代码＝移植有边界的组件；借它的思想/阶段/交互＝仅参考行为。许可证只决定上限（闭源/无许可证/非 OSI 的最多到仅参考行为），不决定选择：许可宽松也可以只借行为。

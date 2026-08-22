# 实现证据与精选参考组合

> 状态：信息性文档 · 研究快照 2026-08-23<br>
> 上级文档：[HCTL2 设计规范](../README.md)<br>
> 规则：本文只说明可行性和复用边界，不定义 HCTL 的领域模型或产品路线。<br>
> 研究标签沿用原始脉络：L4 → Project / Chat Room，L3 → Task / Kanban，L2 → Run / Workflow，L1 → Harness / Terminal。

## 引用准入

研究样本不是“覆盖四层越多越好”的竞品矩阵。许多产品同时具有 Chat、Task、Workflow、worktree 和终端；只有经过源码和产品行为验证、在某一层形成独特设计亮点的部分，才值得进入该层。一个项目可以在多个层留下不同的深度证据，但不会因为顺带具备某个普通功能就被重复罗列。

参考角色：

- **核心参考**：其产品模型或实现深度直接影响该层的主要方案；
- **专项参考**：在该层的一项关键机制上形成了值得采用的完整设计；
- **直接谱系证据**：前代已经实现并验证的语义切片，用来说明继承与改写边界；
- **行为、实现或边界证据**：分别支持交互契约、可移植机制，或证明某类信号不能越权；
- **观察清单**：研究过且有局部价值，但不进入层内主方案；

同一项目跨层出现时，每一处都必须说明该层独有的亮点与不采用边界；平庸重叠仍然删除。标准、通用库和 Task 来源系统单列。

## 精选组合总览

| 项目 | 深入层级 | 参考角色 | 只保留的独特价值 |
| --- | --- | --- | --- |
| [First Tree](https://github.com/agent-team-foundation/first-tree) | L4 核心；L2/L1 专项 | 核心参考 + 专项证据 | 持久 Chat、显式寻址与可见 handoff、Context 筛选与治理、Need You、可靠 Inbox、跨渠道协作和托管运行时连续性 |
| [Claude Tag](https://www.anthropic.com/news/introducing-claude-tag) | L4 | 行为参考 | 共享且可继续引导的讨论串、按作用域拥有的身份和记忆、持久协作与临时运行时分离 |
| [Grok Bot](https://x.ai/news/introducing-grok-bot) | L4 行为;L3/L2/L1 边界 | 行为参考 | Bot 作为应用原生一等参与者与 handoff 可见性原则、审批双规则与"审批不可逆已完成工作"的显式声明、观察-接管-交还回路;账号级共享云机与凭证的反面证据 |
| [OpenClaw](https://github.com/openclaw/openclaw) | L4 | 专项参考 | 确定性的多渠道身份与路由、配对/白名单和按渠道降级投递 |
| [Codeg](https://github.com/xintaofei/codeg) | L3 核心；L1/L2 专项 | 核心参考 + 专项证据 | 独立异步 `WorkTask`、评审/合并/恢复、ACP/worktree/差异集成，以及自动化与固定流程的边界 |
| [Hermes Agent](https://github.com/NousResearch/hermes-agent) | L3 | 专项参考 | 持久 Task/Attempt、原子领取、心跳/回收、依赖推进和多客户端共用内核 |
| [Multica](https://github.com/multica-ai/multica) | L4/L3/L2/L1 专项 | 行为、边界与实现证据 | L4 的 Project/Issue/私聊发布边界；L3 的 Issue 与单次运行分离；L2 的领取、租约、重试、恢复与归属；L1 的多 Harness 能力矩阵和无损 worktree |
| [LobeHub](https://github.com/lobehub/lobehub) | L4/L1 专项；L2 边界 | 专项参考 + 边界证据 | Context 组装的纯机械处理器管道与增量持久化压缩；外部 Harness 子进程适配器的终局结果契约、状态提取与会话重建；supervisor LLM 路由和默认工具 token 重量的反面实证 |
| [Cumora](https://github.com/yetone/cumora) | L4/L3/L1 专项;L2 边界 | 专项参考 + 边界证据 | 唤醒 triage 门与协同门(seen 游标/HELD/hold-token)、Shipping 验收覆盖矩阵与 builder/verifier 分离、BYOA 引擎适配矩阵与保守会话重置、设备配对与不可伪造身份;自由文本证据与全权引擎派发的反面实证 |
| [Rakazo](https://github.com/elie222/rakazo) | L2/L1 专项;L4 专项 | 专项参考 + 边界证据 | 三层带隔离栅栏的租约与幂等效果账本、挂起前强制 checkpoint 的等人状态、供应商中立的可移植工作区、人/机双租约接管;记忆修订携带 run 级出处;提示词代替策略引擎的反面实证 |
| [Helio](https://www.helio.im/) | L4/L2/L1 专项;L3 边界 | 行为参考 + 专项参考(开源外围)+ 边界证据 | 消息面 CAS/cede/receipts/turn 级出处与人批 charter、三元归约与未收尾看门狗、机械 stop-gate 与证据分级、side_effect 安全默认与临时凭证注入;"关单人类专属"营销与实现落差、agent 自行关单的反面证据 |
| [HCTL1 / yesme/hctl](https://github.com/yesme/hctl) | L2 | 直接谱系证据 | Git 原生 Seat 领取与隔离栅栏、精确 Verdict 与法定票数、可重放 Receipt 和失败时默认拒绝的测试集 |
| [HCTL2 Run 语义内核](../run.md) | L2 | 原生语义核心 | 与版本和证据绑定的 Run、Seat 候选切换、法定票数、重新过 Gate 和 Receipt |
| [Dagu](https://github.com/dagucloud/dagu) | L2 | 机械状态后端 | 声明式 DAG、单进程文件持久化，以及可由 control 完成的无进程等待检查点 |
| [Conductor OSS](https://github.com/conductor-oss/conductor) | L2 | 已评估对照 | 外部任务领取/完成接口更直接，但 JVM 分发与整体 footprint 不及 Dagu 贴合第一阶段 |
| [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) | L2 | 相邻实现参考 | SOP 准入、按版本审批与法定票数、恢复和失败时默认拒绝的规则测试 |
| [Stably Orca](https://github.com/stablyai/orca) | L1 核心；L2 专项 | 核心参考 + 专项参考 | L1 的 PTY 所有权、冷热恢复、代际隔离和 worktree/差异/远程/交付；L2 的持久 Run 收件箱、Dispatch 权威、可靠交付、幂等收据和执行者资源生命周期 |
| [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) | L1；横切架构 | 专项参考 + 架构边界 | 能力端口、类型化事件和可撤销注册、模型可见只追加日志，以及插件组合的收益与风险 |
| [OpenCode](https://github.com/anomalyco/opencode) | L1 | 专项参考 | OpenAPI + SSE + 类型化 SDK 的服务端优先、多客户端 Harness 操作面 |
| [Pi](https://github.com/earendil-works/pi) | L1 | 专项参考 | 内嵌 SDK + 严格 JSONL RPC，以及 `steer`/`follow_up` 队列契约 |
| [Kimi Code](https://github.com/MoonshotAI/kimi-code) | L1 | 专项参考 | ACP/原生能力矩阵和可以验证的降级行为 |
| [Termio](https://github.com/termio-sh/termio) | L1 | 专项参考 | Harness Manifest、会话 URI，以及监听/心跳/信号契约 |
| [tmux](https://github.com/tmux/tmux) | L1 | 运行时后端（采用为依赖） | 公开 control mode、稳定 pane ID、headless 终端应答、捕获/转发与多客户端能力，以很小的 native footprint 提供 agentd 所需物理原语 |
| [Herdr](https://github.com/herdrdev/herdr) | L1 专项；L2 边界 | 专项参考 + 边界证据 | L1 的服务端 PTY、观察/控制分离、单写者接管和分级恢复；L2 的运行状态信号与语义完成权威分离 |
| [Codex Remote Feishu](https://github.com/kxn/codex-remote-feishu) | L1 | 行为参考 | 托管会话的连接与路由、输入排队与引导、Request 和重连状态机 |
| [Superset](https://github.com/superset-sh/superset) | L1 核心；L2 边界 | 核心参考 + 边界证据 | L1 的 PTY 守护进程、断线重连与重放、Agent 会话恢复和 worktree 分阶段清理；L2 证明分派与会话传输不等于执行结果或 Workflow 事实 |
| [MindFS](https://github.com/a9gent/mindfs) | L1 | 观察清单 | 仓库本地会话、外部会话导入与同步、轻量部署 |
| [Paseo](https://github.com/getpaseo/paseo) | L1 | 观察清单 | 守护进程、客户端、执行提供方适配 SDK 和多设备边界 |
| [HAPI](https://github.com/tiann/hapi) | L1 | 观察清单 | 本地原生 Agent 与远程结构化交接 |
| [Happy](https://github.com/slopus/happy) | L1 | 观察清单 | 守护进程、端到端加密会话同步、远程启动和多设备 |
| [Moshi](https://getmoshi.app/docs/introduction) | L1 | 观察清单 | 移动终端、钩子/Attention 和 TUI 聊天投影 |
| [Remux](https://github.com/h3nock/remux) | L1 | 观察清单 | SSH + tmux 控制模式下的精确会话/窗口/窗格连接 |
| [ServerCC](https://servercc.app/docs/sessions) | L1 | 观察清单 | 外部接管、厂商会话恢复和移动控制 |
| [QuickTUI](https://quicktui.ai/) | L1 | 观察清单 | 自托管 tmux 和 iOS/iPad/浏览器终端操作面 |
| [Redock](https://redock.dev/) | L1 | 观察清单 | 移动端分阶段输入、中文/语音和 Activity 深链接 |

这个归类按“设计亮点”而不是按产品名排他。Codeg 可以同时贡献 L3 的独立 Task 生命周期和 L1 的集成边界；Stably Orca 同时贡献 L1 的执行连续性与 L2 的持久监督协议；Herdr 的主要实现价值在 L1，其状态仲裁还为 L2 提供“运行信号不等于语义完成”的边界证据。任何项目若只顺带拥有 Project、Agent 或 Board 概念，仍不会因此进入对应层。HCTL1 则是 HCTL2 L2 语义内核的直接谱系证据。

## 四层如何组合这些亮点

| 层 | 主要设计来源 | HCTL2 的组合方式 |
| --- | --- | --- |
| L4 · Project Room | First Tree 的持久 Chat、显式寻址、可见 handoff、Need You、Context 提升与跨渠道连续性；Multica 的共享 Issue 与私密探索发布边界；Claude Tag 的持久讨论串与临时沙箱分离；OpenClaw 的外部身份和路由 | HCTL2 用规范 Room、一级 Request、Context Manifest 和 Memo 提升流程统一这些经验；外部渠道只作同一 Room 的输入输出面，私聊和执行记录不会自动成为项目知识，协作边的创建权也不随消息作者身份下放给 Agent |
| L3 · Task / Kanban | Codeg 的独立 `WorkTask`、Needs You、评审、后续动作和 Git 恢复；Multica 对 Issue 与单次运行、运行结束与承诺完成的明确分离；Hermes 的领取与重新领取；Linear/GitHub 的原生字段状态 | HCTL2 将长期承诺冻结为 Task Revision，把高频操作状态、外部字段权威和 Task Completion Receipt 分开；启动 Run 与移动卡片分离，完成必须重新校验验收标准和证据 |
| L2 · Workflow / Run / Gate | HCTL1 的版本/证据、领取/隔离栅栏、法定票数和 Receipt；Dagu 的机械图状态与被动等待检查点；Stably Orca 的持久监督协议；Multica 的租约/重试/恢复/归属；ZeroClaw 的审批准入；Herdr、Superset 的边界反例 | HCTL2 自己定义 Workflow Revision、Run Manifest、Obligation、Seat、Attempt、Verdict 和 Receipt；外部机制只补机械推进、可靠领取、消息交付和故障测试，不能用执行者状态或会话传输替代语义治理 |
| L1 · 执行 / 运行时 | Stably Orca 的 PTY 所有权、冷热恢复、远程和交付；Superset 的 `epoch:seq` 重连、守护进程接管和分阶段清理；Herdr 的观察/控制分离；Multica 的多 Harness 能力和不丢代码；DeepSeek Harness 的组合式能力端口；OpenCode/Pi/Kimi/Termio 的接入协议 | HCTL2 以 agentd、harness 适配器、运行时后端、ChangeSet 和终端网关统一接入；所有能力逐绑定探测并准确降级，运行时身份、终端状态和厂商会话都不能反向定义 Project、Task 或 Run |

这张表是“整合关系”，不是对象映射。每个来源项目只贡献表中写明的机制；L4–L1 是本研究保留的历史标签，最终身份、权限、版本和证据由 HCTL2 的 Project、Task、Run、Agent 四模块定义。

<a id="e-l4-first-tree"></a>
## E-L4-FIRST-TREE · First Tree

### 核心价值与跨层画像

First Tree 真正跑通的是协作闭环，而不是任务或工作流闭环：Team 中的持久 Chat 或 SCM 事件形成共同上下文，受管 Agent 工作后把结果交还给用户或 SCM；只有同时通过 Decision Test 和 Durability Test 的稳定结论，才会经由另一条有来源支撑、需要审查的流程写入 Context Tree。写入 Context Tree 并不是每次 Chat 的自动收尾；没有具体来源材料时，规则明确要求什么都不写。

它最突出的价值仍在 L4：证明以 Chat 和 Context 为主轴可以维持长期协作。但源码还给出了两组值得跨层引用的深入机制：L2 可以参考精确快照、有来源支撑的写入，以及 Reviewer 对精确 head/digest 的批准与失效规则；L1 可以参考受管执行提供方的会话代次、ACK、重试、恢复和能力契约。它并没有 HCTL 意义上的 Project、Task、Task Revision、Workflow 或 Run；`task chat` 只是创建 Chat 的一种模式，GitHub Task Agent 的事实仍是 Issue/PR 加权威 Chat，cron 也只是把触发器转成消息。因此，这些跨层亮点只是专项机制，不能把 First Tree 整体当成通用 L2 编排器、L3 Task 系统或 L1 终端管理器。

协作拓扑需要拆成两半评价。First Tree 的 `chat send` 要求显式 recipient，正文里的 `@name` 本身不触发路由；handoff、邀请与后续消息都留在持久 Chat 中，对人可见且可追溯。这证明“显式寻址 + 持久 Chat + 可见 handoff”可以避免隐藏的 peer RPC。与此同时，Agent 可以在运行中 `invite + send`，接收者还能继续寻址第三个 Agent，使参与者集合与协作图由模型临场扩张。HCTL 采用前一半，不采用后一半作为默认拓扑：普通 Room 中 Agent 只能建议下一条协作边，由 human actor 提交；自动化边则由 reducer 按冻结的 Workflow Revision 创建。这里记录的是参考取舍，实际权限与命令合同仍以规范文档为准。

### 审计基线

发布版与当前主干必须分开陈述：

| 基线 | 状态 | 可支持的结论 |
| --- | --- | --- |
| [`v0.5.20 / 19e66032`](https://github.com/agent-team-foundation/first-tree/commit/19e66032af7f9f482168c350fe0b3998599388f3) · 2026-08-11 | 已发布 | Context Tree、持久 Chat、基于稳定身份的 mention、Request/Need You、Inbox、GitHub/GitLab，以及执行提供方的运行与恢复 |
| [`main@f0d46f9e`](https://github.com/agent-team-foundation/first-tree/commit/f0d46f9ec8b14ace536d242db8860065c124f2c7) · 2026-08-14 | 未发布审计快照；比发布版前进 41 个 commit | Feishu Agent Channel、`OpenTag` 入门流程、更新后的 GitHub Issue 激活规则，以及运行权限、`ReplayFence` 和 Reset 机制 |

[发布版与审计快照的差异](https://github.com/agent-team-foundation/first-tree/compare/19e66032af7f9f482168c350fe0b3998599388f3...f0d46f9ec8b14ace536d242db8860065c124f2c7)。Feishu QA 文件只是可执行的验收契约，不是公开的通过报告；这些主干能力不能写成 v0.5.20 已发布功能。

[官网](https://first-tree.ai/)仍以 CODEOWNERS 描述 Context Tree 的归属关系，但 v0.5.20 的实际规则使用 frontmatter 中的 `owners` 字段，[Seed Skill](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/skills/first-tree-seed/SKILL.md)还明确禁止创建根 `CODEOWNERS`。网站描述已经偏离当前实现，不应继续作为设计依据；[官方文档站](https://docs.first-tree.ai/)目前也只是占位内容。以下判断以固定源码、仓库文档和可执行测试为准。

### 源码审计结论

| 范围 | 已验证 | 缺口 | HCTL 如何吸收 |
| --- | --- | --- | --- |
| 产品对象 | Team、Agent、人类成员、持久 Chat、类型化 Message、Context Tree、Agent 会话，以及 SCM entity↔Chat 映射 | 没有 HCTL 的 Repo/Project/Task/Workflow Revision/Run/Seat；Team 还可能横跨多个代码仓库 | 证明 L4 协作可以持续；不照搬 Team/Agent/Chat 数据结构，也不把 Chat 直接叫作 Project Room |
| Context Tree | Decision Test + Durability Test；Tree 与代码不一致时，默认以代码事实为准；按精确 commit 读取快照；写入必须有来源材料、独立 worktree、校验和 PR/MR 评审 | 治理模型绑定 First Tree 的 Team、Reviewer 和代码托管平台；知识晋升是独立流程，不会在每次任务后自动执行 | 把筛选标准和有来源支撑的评审流程改编成 Memo→Project 知识准入；不新增 `ContextTree` 一级对象 |
| 类型化 mention / Inbox | Web 发送稳定的 Participant ID，服务端校验成员关系和启用状态；Message 与接收者分发在同一事务中；支持 `pending/delivered/acked`、`SKIP LOCKED`、逐 Chat 前缀 ACK 和断线恢复 | CLI/API 仍兼容名称寻址；普通发送没有调用方幂等键；消息可原地编辑，只有 `editedAt`，没有 revision/history/tombstone | 采用稳定身份、ACK 责任链和事务测试；HCTL 另补命令 ID、只追加的 correction/tombstone 和冻结的 mention 引用 |
| 协作边 / handoff | `chat send` 使用显式 recipient；只有被具名寻址的 Agent 被唤醒，其他 participant 只获得 silent context；邀请、交接和结果留在持久 Chat 中 | Agent 可以自行 `invite + send`，接收者还能继续寻址第三个 Agent；系统没有冻结的通用 Workflow 图约束这条动态链 | 采用显式寻址、持久 Chat 和对人可见的 handoff；不采用 Agent 消息直接创建执行边或开放 mesh，普通 Room 的临场边由人提交，自动化边由 reducer 按冻结图创建 |
| Request / Need You | 当前权威事实是 `format="request"` 消息与后续 resolution 行；只允许一个用户作为目标；目标用户的界面会局部阻塞，其他成员仍可阅读；多个请求先进先出；普通回复或 `inReplyTo` 不会关闭请求，只有目标用户显式写入 `metadata.resolves` 才产生新 resolution；跨 Chat 队列从持久记录推导 | 旧 `attentions`、`pending_questions` 只是历史审计表；Request 仍是一种消息格式，不含 revision、权限或法定人数语义 | 借鉴归约器、显式关闭、先进先出和仅阻塞目标用户的交互；HCTL 将其提升为一级 Request，但绝不让它替代 Gate、Seat 或法定人数规则 |
| GitHub 集成 | HMAC、delivery ID 去重、entity↔权威 Chat、由服务端记录的 run 来源、受管 Task Agent 和幂等 App 回复；当前主干中，普通 Issue 要等非自身输出的新评论或已有精确 owner mapping 才激活，PR 不受此限制 | 工作事实仍是 Issue/PR + Chat，没有独立 Task、Task Revision、Board 或验收生命周期；部分故障、排序和身份主体语义仍不完整 | 借鉴绑定、来源证明、去重和跨界面测试；可在 L3 作为外部工作触发与权威映射的边界证据，但不把 GitHub entity/Chat 当作 HCTL Task 事实 |
| 仅主干存在的 Feishu | bot/chat 绑定、精确 mention、回声抑制、作者快照、事件与消息双重去重、附件取回，以及租约与代次 | 当前是 1 Chat↔1 Feishu；Web 只读；不支持编辑/删除；ACK 可能早于权威事务提交；最终事务没有隔离令牌；出站回执没有生命周期管理 | 借鉴格式转换、去重、租约和验收测试；HCTL 另补多界面绑定、Room Event 与出站队列的原子性、提交时隔离、Receipt 和对账 |
| 执行提供方运行时 / Skills | `start/resume/inject/suspend/shutdown`、ACK、重试、恢复与持久化、目录与能力声明、Skill 的锁、日志、摘要和版本隔离、守护进程监管；主干又加入 `ReplayFence` 和 reset 权限 | 私有客户端与 Hub/Chat 强耦合；API 仍在快速变化；重试只覆盖同一个执行提供方和会话；协议层回执不是语义 Receipt | 只借鉴契约、故障、重放和 Skill 测试，不直接建立包依赖，也不把它当作 Seat 的降级方案 |
| 会话 / 终端 | 执行提供方会话主要通过 SDK、app-server 或子进程运行；通用运行时管理代次、ACK、重试、恢复和会话持久化；内部 `tmux` 驱动支持粘贴与捕获输出 | 新配置已禁用 TUI 选项；没有公开的重新接入接口或稳定 PTY 目标 | 可作为 L1 受管会话与运行恢复的专项实现证据；终端所有权和重新接入仍需参考其他项目 |
| Workflow / 治理 | Context 读取、写入和 Reviewer 流程采用精确快照、来源门槛、精确 head 批准及失效规则；cron 到点生成定向消息，并限制同一个 job 不积压多个未 ACK 触发器 | 没有通用运行历史、DAG、Workflow Revision、Seat、候选执行者、法定人数、重新过 Gate 或绑定版本的 Receipt | L2 可参考 Context 变更准入、快照与批准失效，以及带版本的触发器；不把这些局部机制扩写成通用 Workflow 模型 |

复用结论：**选择性移植**，许可证为 Apache-2.0。可以直接改编 Context Policy 的两项筛选测试、Need You 行为旅程、Inbox ACK 责任链，以及 Feishu/GitHub 的跨界面验收测试；也可按需移植纯数据结构、内容转换、绑定与租约更新、前缀 ACK、`ReplayFence`/reset 代次，以及受管 Skill 的事务纪律。不整仓派生，也不采用其中心化 PostgreSQL 或云端事实源。

主要源码：

- [仓库](https://github.com/agent-team-foundation/first-tree)、[v0.5.20 发布版](https://github.com/agent-team-foundation/first-tree/releases/tag/v0.5.20)与固定版本的 [Apache-2.0 许可证](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/LICENSE)
- [架构边界](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/AGENTS.md)与[快速上手](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/docs/quickstart.md)
- [Context 规则](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/client/src/runtime/assets/context-tree-policy.md)、[读取 Skill](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/skills/first-tree-read/SKILL.md)、[写入 Skill](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/skills/first-tree-write/SKILL.md)与[外部 Context 接入](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/docs/context-integration.md)
- [Chat 数据结构](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/shared/src/schemas/chat.ts)、[Message/Request 数据结构](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/shared/src/schemas/message.ts)、[Need You 归约器](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/server/src/services/chat/workspace/need-you.ts)、[Inbox 服务](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/server/src/services/chat/inbox.ts)与[跨界面 Need You 验收用例](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/qa/cases/cross-surface/need-you-request-review-journey.md)
- 补充协作拓扑快照 [`9a7dd4d9`](https://github.com/first-tree-ai/first-tree/tree/9a7dd4d94373921cfe2022bfef91c132fdf74824)：[Agent runtime briefing](https://github.com/first-tree-ai/first-tree/blob/9a7dd4d94373921cfe2022bfef91c132fdf74824/packages/client/src/runtime/templates/agent-briefing.ejs#L77-L93)、[handoff 规则](https://github.com/first-tree-ai/first-tree/blob/9a7dd4d94373921cfe2022bfef91c132fdf74824/packages/client/src/runtime/templates/agent-briefing.ejs#L161-L183)、[同任务 handoff](https://github.com/first-tree-ai/first-tree/blob/9a7dd4d94373921cfe2022bfef91c132fdf74824/docs/cli-reference.md#L646-L670)、[`chat send` 合同](https://github.com/first-tree-ai/first-tree/blob/9a7dd4d94373921cfe2022bfef91c132fdf74824/apps/cli/src/commands/chat/send.ts#L19-L80)与[邀请权限](https://github.com/first-tree-ai/first-tree/blob/9a7dd4d94373921cfe2022bfef91c132fdf74824/packages/server/src/services/chat/membership/invite.ts#L31-L57)
- 当前主干的 GitHub [受众与激活规则](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/services/scm/github/audience.ts)、[投递流程](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/services/scm/github/delivery.ts)、[entity↔Chat 绑定](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/services/scm/github/entity-chat.ts)与[最终回复发布](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/services/scm/github/task-reply-publisher.ts)
- [执行提供方契约](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/client/src/providers/README.md)、[运行时数据结构](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/shared/src/schemas/runtime-provider.ts)、[会话控制 CLI](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/apps/cli/src/commands/agent/session/control.ts)、[内部 `tmux` 驱动](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/client/src/providers/claude/tui/tmux-session.ts)与[cron 表](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/server/src/db/schema/cron-jobs.ts)
- 仅主干存在的 Feishu [bot 绑定](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/db/schema/im-bot-bindings.ts)、[chat 绑定](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/db/schema/im-chat-bindings.ts)、[入站处理](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/services/integrations/feishu/inbound.ts)、[连接管理器](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/services/integrations/feishu/manager.ts)与[验收契约](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/qa/cases/cross-surface/feishu-agent-channel.md)

<a id="e-l4-claude-tag"></a>
## E-L4-CLAUDE-TAG · Claude Tag

Claude Tag 为 L4 提供产品行为证据。它最独特的设计是：一条 Slack 讨论串就是多人可见的工作会话，频道成员可以继续会话，也可以中途调整方向；讨论串及其上下文持久保存，托管沙箱则可以回收后重建。Agent 使用限定在频道范围内的服务身份、访问权限和记忆范围，不会冒充发起人。Checklist、定时 Routine、频道监听和代码仓事件还展示了低噪声的异步协作投影。

HCTL 借鉴持久 Room 与临时运行环境分离、多人共同引导、Agent 独立身份和受作用域约束的访问控制，并且只把 Checklist/Routine 当作投影或触发器。Slack 频道或讨论串不映射为 Project/Room，Checklist、Memory 或 Routine 不成为 Task、Run 或知识的权威事实，频道成员身份也不能绕过 HCTL 的权限与 Gate。Claude Tag 是闭源的公开测试产品，只能作为行为证据，不能移植源码，也不承担 L1 运行环境方案。

基线按公开资料日期固定为 2026-06-23 Public Beta：[发布公告](https://www.anthropic.com/news/introducing-claude-tag)、[工作原理](https://claude.com/docs/claude-tag/concepts/how-it-works)、[Agent 身份](https://claude.com/docs/claude-tag/concepts/agent-identity)、[Routines](https://claude.com/docs/claude-tag/users/proactivity)、[Memory](https://claude.com/docs/claude-tag/users/memory)。

<a id="e-grok-bot"></a>
## E-GROK-BOT · Grok Bot 与 Grok Build

### 产品定位与基线

Grok Bot 是 SpaceXAI(前 xAI,2026-07 改名,Grok 产品品牌保留)于 2026-08-11 发布的多 Agent 助理平台,early beta,闭源 SaaS;账号、订阅与数据面构建在 Cursor 账号体系上(SpaceX 于 2026 年收购 Cursor 开发商 Anysphere 并入 SpaceXAI)。用户在专用桌面/移动应用中创建至多 50 个具名 Bot(name/title/description/avatar 四字段身份),每个 Bot 跨会话持久、拥有独立记忆;但**全账号所有 Bot 共享同一台托管 Linux 云虚拟机**——官方明说各 Bot 的屏幕只是 "separate work surfaces, not separate security boundaries"(各自的工作面,不是安全边界)。姊妹产品 Grok Build 是其编码 Harness(2026-05-25 发布):CLI 本体开源(Rust、Apache-2.0、不接受外部贡献、从内部 monorepo 单向同步),支持 TUI、headless 与 ACP 三种运行形态;闭源的是服务端模型与 Grok Bot 平台本身。

行为基线固定为 2026-08-22 的官方文档快照。引用第三方评测时必须过滤两处经核实的系统性错误:"每个 Bot 有自己的云计算机"(官方:账号级共享)与 "ACP 是 xAI 的协议"(实为 Zed 发起的中立协议)。

### 各层行为证据

**L4**:Bot 是应用原生的一等参与者——四字段身份、侧栏可寻址、Bot 间消息是一等公民,房间围绕 Bot 而不是围绕人组织。官方文档同时给出两条与 HCTL2 立场同构的行为原则:"Put Bots in a group chat when the handoff itself needs to be visible"(需要交接可见时就放进群聊,群聊 "preserves the handoffs in one conversation"),以及 "Memory is not a substitute for an authoritative source"(记忆不能替代权威来源,重大结论要求 Bot 给出引用)。与已收录的 Claude Tag 互补而不重叠:Claude Tag 证明"assistant 进入人类房间"(寄生于 Slack 的房间与身份体系、单 assistant);Grok Bot 证明"Agent 身份原生化与 Agent 间通信房间化"。反面是 Bot 间私聊——用户不在场的上下文传递无法追溯出处,官方文档自己也把用户往可见群聊引导。

**L3**:官方任务请求五要素(outcome、sources、constraints、deliverable、review point)加"指明 artifact 及其 acceptance criteria",证明头部商用产品已经认识到验收标准必须前置——但只做到了文档建议层:没有独立任务对象、没有生命周期、没有契约版本,验收标准只活在提示词与记忆里随对话漂移。第三方实测的 "Work stops just short of done"(工作停在差一步完成)与审批漏判,正是验收与执行不分离的代价实录。

**L2**:Auto Review 规则有精确的双模式语义——Require Approval(必停)与 Always Allow(仅当自动审查没有其他停下理由时放行),两者同时命中时保守方优先;另有七类固定必审批动作(发消息/邀请、发布内容、购买与转账、删除或覆盖数据、改权限、生产变更、接受法律条款)。官方还明文承认 "An approval controls the proposed action. It does not reverse work already completed"(审批只控制拟议动作,不能撤销已完成的工作)——这是"先冻结、后放行、留凭证"立场的市场印证。官方证据保全清单(来源直链、带状态截图、时间戳与时区、输入输出文件名、动作日志、**显式列出 Bot 无法核验的内容**)等于提示词级的 Receipt。反面:routine(例程)删除即时且无撤销、仅保留 20 条运行记录、无试运行,Auto Review 分类由模型判断且按桌面端本地存储不同步。

**L1**:Agent Computer 视图提供目前所见最完整的商用观察-接管-交还回路:实时观看点击/输入/导航,密码、2FA、CAPTCHA、支付确认等敏感步骤由人接管计算机、完成后交还 Bot 继续,移动端同样可观察与接管。凭证边界留下正反双样本:正面是 secure secret request(值被掩码、不进 transcript、不给模型)、托管 MCP token 留在服务端("The computer never stores those tokens")与 WebAuthn 硬件密钥转发;反面是一次登录全 Bot 共享、文件系统全 Bot 可读、删除 Bot 后文件与登录残留在云机上。Grok Build 侧的编排方向是单向的:它可以作为 ACP agent 被任何应用托管编排,但 Grok Bot 平台不接受第三方 Harness 接入、也没有公开 API——平台封闭,开放的只有编码 Harness 这一层。

### 采用与边界

HCTL 借鉴的行为:handoff 可见性原则、"审批不撤销已完成工作"的诚实声明、固定必审批动作清单、证据保全清单(升级为系统级 Receipt 对象)、敏感步骤接管-交还回路、secret 掩码与不入上下文纪律。明确不采用:Bot 间私聊传递上下文(违背上下文出处可溯)、模型判断代替确定性 Gate、账号级共享虚拟机与凭证(HCTL 的凭证按任务与执行者定界)、验收标准只存在于提示词。Grok Bot 是闭源产品,只作行为证据,不移植任何实现;Grok Build 的开源仓库可另作 L1 ACP 接入的协议证据,但不因此进入 L1 主参考。

主要证据:

- 官方:[Grok Bot 发布公告](https://x.ai/news/introducing-grok-bot)(2026-08-11)、[Grok Build 发布公告](https://x.ai/news/grok-build-cli)(2026-05-25)、官方文档 [overview](https://docs.x.ai/grok-bot/overview)、[bots](https://docs.x.ai/grok-bot/bots)、[computer-and-apps](https://docs.x.ai/grok-bot/computer-and-apps)、[files-and-results](https://docs.x.ai/grok-bot/files-and-results)、[approvals-security-and-privacy](https://docs.x.ai/grok-bot/approvals-security-and-privacy)、[teams-and-enterprises](https://docs.x.ai/grok-bot/teams-and-enterprises)与[faq](https://docs.x.ai/grok-bot/faq);[Grok Build 开源仓库](https://github.com/xai-org/grok-build)(Apache-2.0)与 [Zed ACP Registry 条目](https://zed.dev/acp/agent/grok-build)
- 第三方(已过滤系统性错误):[VentureBeat](https://venturebeat.com/orchestration/spacexais-grok-bot-turns-agents-into-persistent-digital-coworkers-that-can-operate-your-apps-for-120-per-month)(定价、三 Bot 编排实测、内部 Chief of Staff 用法)、[eesel 缺口审计](https://www.eesel.ai/blog/grok-bot-review)(无试运行、审计日志未交付、routine 只留 20 条记录、Bot 删除残留)、[Composio 实测](https://composio.dev/content/guide-to-frok-bot)(群聊 2-6 成员上限、公司模拟实验)与 [atomicbot](https://atomicbot.ai/blog/what-is-grok-bot)(审批与打断细节)

<a id="e-l4-openclaw"></a>
## E-L4-OPENCLAW · OpenClaw

OpenClaw 最值得参考的是 L4 的外部频道接入边界：它把账号、对端和讨论串归一为确定性路由键，并支持精确绑定、讨论串继承、私信作用域、配对与允许名单、房间环境事件、防止机器人循环，以及按频道能力降级投递。这说明：没有 Workbench 时，Chat 界面仍需要稳定的外部身份、确定性路由和逐频道降级，不能让模型猜测频道，也不能按显示名称分发。

HCTL 只借鉴适配、路由、配对、防循环和降级测试；OpenClaw 的 channel/session/workspace/agent 不映射为 Project/Room/Task/Run，环境聊天不会自动成为权威 Context，Gateway、cron 或 delegation 也不成为 L2 的权威事实。固定版本为 [`v2026.7.1-2 / 0790d9f5`](https://github.com/openclaw/openclaw/tree/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c)（MIT）；证据见[频道路由](https://github.com/openclaw/openclaw/blob/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c/docs/channels/channel-routing.md)、[README](https://github.com/openclaw/openclaw/blob/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c/README.md)与[许可证](https://github.com/openclaw/openclaw/blob/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c/LICENSE)。

<a id="e-l3-codeg"></a>
## E-L3-CODEG · Codeg

### 核心价值与跨层画像

Codeg 的产品闭环是：先把一项工作写成独立 `WorkTask`，再排队并按并发上限领取；启动时根据精确的 base SHA 创建隔离 worktree；Agent 执行过程中可以进入 Needs You；完成一轮后由用户查看结果、diff、时间线和预检，再选择 Rework、Keep going、Ask、Double-check 或 merge；最后用 Git 事实确认结果是否真正落地。官方 Tasks 指南对边界的概括也很准确：Conversation 是用户坐在前面共同推进的会话，Task 则是写下以后可以暂时离开的异步承诺。

`WorkTask` 的 `worktree_folder_id`、`conversation_id` 和 `connection_id` 都可以为空，因此 Task 身份不依赖某次运行环境。状态变化使用“预期状态 + `run_seq`”进行 CAS，并把只追加的时间线事件放在同一事务中。Board、排队、Needs You、评审、后续动作、预检、Git 事实和重启恢复，共同构成其完整的 L3 产品机制。

Codeg 的核心价值在 L3，其他层也有可单独采用的机制。L1 可以专项参考它把 ACP、worktree、Git、diff、Composer、事件卡片，以及由 Agent 所在环境提供的文件系统和终端沙箱接成一条执行体验；L2 可以把 Automations 的定时触发、单个 Automation 串行、补跑，以及固定 Task 流程与 merge 恢复，当作“产品层自动化”和“通用 Workflow 治理”之间的边界证据。桌面终端由进程内 `HashMap` 持有，没有持久滚屏记录、进程重启恢复或稳定的远程重新接入，因此 PTY 所有权与重连主要参考 Stably Orca。Automations 没有带版本的图、权限与法定人数规则或通用 Gate，也不能单独定义 HCTL 的 L2。

### 审计基线

| 基线 | 状态 | 可支持的结论 |
| --- | --- | --- |
| [`v0.24.0 / df7a872d`](https://github.com/xintaofei/codeg/commit/df7a872de44546277e4c49cfe9d173c631161dc6) · 2026-08-11 | 已发布 | 独立 `WorkTask`、四列 Board、排队/并发/定时启动、Needs You、评审/预检/后续动作、merge 与基于 Git 事实的恢复 |
| [`main@a34a047a`](https://github.com/xintaofei/codeg/commit/a34a047a568018ee180dee75add8c9c7d30b2ea6) · 2026-08-14 | 未发布审计快照；按 first-parent 口径比发布版前进 23 个 commit | merge 排队、[由 Agent 所在环境提供文件系统与终端沙箱](https://github.com/xintaofei/codeg/commit/b7e21e4c789ba70036ec87de5ed72dec3d25a678)，以及重连与权限修正 |

[发布版与审计快照的差异](https://github.com/xintaofei/codeg/compare/df7a872de44546277e4c49cfe9d173c631161dc6...a34a047a568018ee180dee75add8c9c7d30b2ea6)。v0.24.0 和当前 Tasks 指南规定“同一项目已有 merge 时拒绝第二个 merge”，主干从 [`597a7eeb`](https://github.com/xintaofei/codeg/commit/597a7eeb24e4a5f8aca149f2f5c182d3c2c90510)起改为排队。发布能力与主干能力在本文中分别标注。

### 采用与边界

HCTL 采用独立 Task 身份、显式状态机、基于 `run_seq`/CAS 的过期事件隔离、事务内时间线、精确 base SHA、复用 worktree 的重试、Needs You 投影、评审/预检/后续动作，以及“根据 Git 事实恢复 merge”的测试。`done` 只应来自已经落地的 merge，或用户明确接受“没有内容可合并”；Agent 自报 `task_complete` 只能作为建议，不能决定 Task 是否完成。

Codeg 的 `WorkTaskConfig` 不能直接当作 HCTL 的冻结 Task Revision。它保存 `prompt_blocks` 和每个 Task 的覆盖值，但空字段会在真正启动时继承当时的 Folder 设置，实际采用的值只写入 `config_effective` 审计事件。HCTL 在批准 Run 时必须冻结完整的 Task Revision/Workflow Revision，不能让可变默认值在启动时继续改变契约。

明确不采用：不把 Conversation 当作 Room，不把 To-do 数据结构直接视为 HCTL Task，不把固定 Task 流程直接视为 Workflow，不让拖进 In Progress 自动获得施工授权，不因 Agent `task_complete`、Done 分栏或 Git 已落地就自动满足语义验收，不把 Agent 发起 merge 等同于具有治理权限，不让主 LLM 路由充当控制事实，也不把进程内终端当作 L1 持久性模型。

主要证据：

- [仓库](https://github.com/xintaofei/codeg)、[v0.24.0 发布版](https://github.com/xintaofei/codeg/releases/tag/v0.24.0)与固定版本的 [Apache-2.0 许可证](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/LICENSE)
- 官方产品说明：[Tasks](https://docs.codeg.app/guide/tasks)、[Automations](https://docs.codeg.app/guide/automations)、[多 Agent](https://docs.codeg.app/guide/multi-agent)与[聊天频道](https://docs.codeg.app/guide/chat-channels)
- Automation 的固定源码：[持久对象](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/db/entities/automation.rs)、[Run 记录](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/db/entities/automation_run.rs)、[调度引擎](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/automation/engine.rs)与[事务服务](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/db/service/automation_service.rs)
- [WorkTask 状态与持久字段](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/db/entities/work_task.rs)、[配置与 Folder 设置](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/models/work_task.rs)、[CAS 与事务事件服务](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/db/service/work_task_service.rs)、[执行与恢复引擎](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/work_task/engine.rs)及[Git 判定](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/work_task/git.rs)
- [四列 Board 投影](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src/components/tasks/board-columns.ts#L4-L58)、[Composer](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src/components/chat/composer/rich-composer.tsx)、[ACP 注册表](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/acp/custom_registry.rs)与[委派数据结构](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/acp/delegation/tool_schema.json)
- L1 边界证据：[桌面 PTY 管理器](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/terminal/manager.rs)与[面向 Agent、限制输出大小的终端运行时](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/acp/terminal_runtime.rs)

<a id="e-l3-hermes-agent"></a>
## E-L3-HERMES-AGENT · Hermes Agent

Hermes Agent 的独特价值是由 Agent 操作的持久 Task/Attempt 协议：SQLite Board 保存 Task、Run/Attempt、依赖、评论和工作区；调度器负责原子领取、心跳、过期或崩溃 Worker 的回收、依赖满足后的状态推进，以及协议违规时自动阻塞；CLI、Chat 斜杠命令和 Dashboard 共用同一套命令内核。它为 L3 提供了重启恢复和无 Workbench 操作方面的实现证据。

HCTL 借鉴 Task/Attempt 分离、领取与回收、持久评论和共用命令内核；不把 Board 当作 Project，不把 profile/memory 当作 Participant/Project，不把模型自报完成当作 Receipt，也不把单机调度器当作 L2 权威事实，更不让 LLM 的目标判断决定语义完成。固定版本为 [`v2026.8.13 / f80f453a`](https://github.com/NousResearch/hermes-agent/tree/f80f453ae0679347e38abc917c7f94f717bf96c5)（发布名称 `v0.20.1`，MIT）；证据见 [Kanban 指南](https://github.com/NousResearch/hermes-agent/blob/f80f453ae0679347e38abc917c7f94f717bf96c5/website/docs/user-guide/features/kanban.md)、[README](https://github.com/NousResearch/hermes-agent/blob/f80f453ae0679347e38abc917c7f94f717bf96c5/README.md)与[许可证](https://github.com/NousResearch/hermes-agent/blob/f80f453ae0679347e38abc917c7f94f717bf96c5/LICENSE)。

<a id="e-multica"></a>
## E-MULTICA · Multica

### 已跑通的产品闭环

Multica 把 Project 和 Issue 中的目标、讨论与状态保存为长期工作事实。每次分配、提及、私聊或 Autopilot 触发都会新建一个 Task，而 Task 只表示一次 Agent 运行。服务端先把它排入队列，再由本机守护进程认领并调用已经安装的 Harness，最后把消息、工具调用、错误、会话和交付分支写回。人或 Agent 随后决定继续讨论、重新运行还是结束 Issue。这个闭环同时触及四层，但 HCTL 只吸收各层真正独特的机制，不照搬整套产品模型。

### 审计基线与许可

固定实现基线为 [`main@2c0912b6`](https://github.com/multica-ai/multica/tree/2c0912b6ec764b373d44eeea1e80f0d9f11ab417)（2026-08-14）。同期最新发布版是 [`v0.4.26 / 19155e41`](https://github.com/multica-ai/multica/releases/tag/v0.4.26)，主干只比发布版多一个提交。项目仍处于 `0.x` 快速演进阶段，官网会滚动更新；能力判断以固定源码、迁移和测试为准。

仓库完整公开，README 将项目称为“开源”，但固定版本的 [`LICENSE`](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/LICENSE) 不是单独的 Apache-2.0：它在 Apache-2.0 文本之外增加了第三方托管、商业嵌入、品牌和归属要求，并声明附加条件优先。因此这里只把它当作公开源码的行为、协议和测试证据；在完成专门的法律审查并获得所需授权之前，不把其源码移植进 Apache-2.0 的 HCTL，也不把该许可证标成 Apache-2.0 或宽松开源许可。

### 四层设计亮点与边界

| 层 | 真正深入且独特的证据 | HCTL 的采用方式与边界 |
| --- | --- | --- |
| L4 | Project 保存跨多个 Issue 的目标、范围、长期要求、负责人和代码资源；Project 状态与 Issue 状态相互独立。Issue 集中保存可共享的目标、讨论、活动和执行历史；一对一 Chat 明确位于 Issue 之外且完全私密，团队要复用的结论必须另行写入 Issue、Project 描述或 Skill。Inbox 是面向人的关注入口，不是 Agent 工作队列。 | 采用“共享事实与私密探索分开、私聊结论显式发布”的边界，以及 Project 状态不从子项机械推导的做法。Multica 没有可持续共同引导的项目级 Room、Context 准入或知识晋升流程；Project/Issue 描述又会以当前值直接进入运行上下文，不能代替 HCTL 的持久 Room、Memo 或冻结版本。 |
| L3 | Issue 是可以长期讨论、修改、重新分配并最终关闭的工作承诺；Task 是一次生命周期有限的运行。同一 Issue 可以产生多个 Task，已有运行记录不会被覆盖；精确重试某个历史 Task 时仍调用该次运行当时的 Agent。官方文档明确规定：Task 的 `completed` 只表示该次运行正常结束，不表示 Issue 目标已经完成。 | 采用 Issue 与单次运行 Task 分离、运行历史不可覆盖、定向重试，以及“运行完成不等于工作完成”。不采用可变 Issue 描述作为冻结的 Task Revision，不采用分配或状态变化自动获得施工授权，也不把 Agent 将状态改成 `in_review`、产生分支或 Task 正常退出当作验收。 |
| L2 | Task 具有 `queued → dispatched → waiting_local_directory/running → terminal` 生命周期。数据库通过 `FOR UPDATE SKIP LOCKED` 原子认领，并把同一 `(Issue, Agent)` 的运行串行化；准备租约保护启动窗口，`dispatched_at` 充当认领代际的 CAS 隔离栅栏，认领响应丢失后可以重新领取，守护进程重启后可以回收。长期运行依赖运行时心跳，而不是固定的总时长；失败分类决定能否重试，后继 Task 保存 `attempt`、`max_attempts`、`failure_reason`、`session_id`、`work_dir` 和 `retry_of_task_id`，触发者、委派链和证据引用也随运行记录归属。Autopilot 还为定时和 webhook 的每个触发实例提供幂等与崩溃恢复测试。 | 采用领取（claim）、租约（lease）、隔离栅栏（fence）和重新领取（reclaim），并采用失败分类、重试谱系、来源归属和轮询兜底的实现与测试形状，尤其适合无 Workbench 时由服务端和守护进程协作执行。它没有 Workflow Revision、通用 DAG、Gate、Seat、法定票数或语义 Receipt；Squad leader 由 LLM 决策，不能成为控制事实；Autopilot 是可重复触发的操作手册，不是 HCTL Workflow。 |
| L1 | 一个统一的 `Backend` 契约接入 22 个 Harness 产品名称；其中 21 个协议族由后端构造器、数据库约束和锁步测试共同限定，Oh-My-Pi 复用 Pi 协议族。不同 Harness 的模型、MCP、Skill 路径和会话恢复能力被明确列成能力矩阵，并对“无法判断恢复请求是否被拒绝”等降级情况单独编码。本地 Git 路径会先保全脏工作树，再为每个 Task 建立 worktree；无论成功、失败还是取消，都会尽量提交已经产生的改动，提交失败时则保留 worktree，避免清理过程吞掉用户工作。 | 采用统一 Harness 契约、逐绑定能力探测、显式降级测试，以及“先保全、后隔离、任何退出路径都不丢改动”的 worktree 纪律。Multica 不拥有可重新接入的 PTY，也不能用会话、分支或工具调用成功证明语义完成；其源码许可也排除了直接移植。 |

### 采用结论

HCTL 应组合采用四块经过源码验证的机制：L4 的共享/私密发布边界；L3 的 Issue/单次运行分离；L2 的领取、租约、重试、恢复和来源归属机制及其测试用例；L1 的 Harness 能力契约与无损 worktree 收尾。它们分别进入对应层，不需要把 Multica 设成某一层的唯一参考。

明确不采用：用 Issue 当前内容充当 Task Revision，用分配或状态变化充当启动授权，用 Squad leader 的 LLM 判断充当调度权威，用 Autopilot 充当通用 Workflow，用 Task 的 `completed` 充当 Verdict/Receipt，以及移植受自定义许可证约束的源码。

主要证据：

- 官方产品行为：[Projects](https://multica.ai/docs/projects)、[Issues](https://multica.ai/docs/issues)、[Tasks](https://multica.ai/docs/tasks)、[Chat](https://multica.ai/docs/chat)、[守护进程与运行时](https://multica.ai/docs/daemon-runtimes)、[Harness 对比](https://multica.ai/docs/providers)与[Autopilots](https://multica.ai/docs/autopilots)
- 固定文档：[Projects](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/projects.mdx)、[Issues](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/issues.mdx)、[Tasks](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/tasks.mdx)、[Chat](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/chat.mdx)与[Harness 能力矩阵](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/providers.mdx)
- L2 实现：[Task 服务](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/internal/service/task.go)、[领取与重试 SQL](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/pkg/db/queries/agent.sql)、[租约与重试数据结构](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/migrations/055_task_lease_and_retry.up.sql)、[准备租约](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/migrations/124_task_prepare_lease.up.sql)、[领取竞争测试](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/internal/service/task_claim_race_test.go)、[完成竞争测试](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/internal/service/task_complete_race_test.go)与[Autopilot 恢复测试](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/cmd/server/autopilot_schedule_job_test.go)
- L1 实现：[统一 `Backend` 与能力例外](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/pkg/agent/agent.go)、[协议族锁步测试](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/pkg/agent/agent_supported_types_test.go)与[本地 worktree](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/internal/daemon/execenv/local_worktree.go)

<a id="e-lobehub"></a>
## E-LOBEHUB · LobeHub

### 核心价值与跨层画像

LobeHub 是由 LobeChat 原地演化的"Chief Agent Operator"平台：同一仓库先后经历 2025-10/11 的 2.0 重构（[包名改写 `26daac5a`](https://github.com/lobehub/lobehub/commit/26daac5a6d) 2025-10-30、[2.0 声明](https://github.com/lobehub/lobehub/discussions/10007) 2025-11-03）、2026-01-27 的 v2.0.0 定版改名，以及 2026-05-18 的 [agent 运营定位](https://github.com/lobehub/lobehub/discussions/14935)，从聊天应用扩展成雇佣、排程、汇报整队 Agent 的服务端平台。它对 HCTL2 的独特价值集中在三块经源码验证的机制：

1. **Context 组装是纯机械流水线，摘要是唯一显式 LLM 步骤**。`packages/context-engine` 把"存储消息 → 送入模型的消息数组"实现为编号阶段 0–7（外加 4.5 虚拟尾部）近 60 个处理器的顺序管道：占位清理 → 按逻辑组截断 → 系统提示拼装 → 单条合成注入消息（记忆/知识/群组上下文）→ 末条用户消息增强 → 展示容器摊平与角色改写 → 多模态与工具调用转换 → 重排净化；高变动内容刻意排在管道尾部以保护 provider 前缀缓存。管道内没有任何 LLM 调用；压缩触发用本地启发式估算器（tokenx × 1.25 漂移系数，阈值＝模型窗口的 50%）。真正花钱的摘要是一次显式指令：把历史压成结构化摘要，作为一等 DB 行（`compressedGroup`）持久化、后续请求以 `<compressed_history_summary>` 回合复用，再压缩时增量折叠既有摘要而不是从原文重推，用量计入账本。
2. **外部 Harness 包裹层的状态提取与诚实合同**。`packages/heterogeneous-agents` 以 stream-json/JSON-RPC 子进程（无 PTY）驱动 Claude Code、Codex、Cursor 等十余家 harness，按厂商实现有状态适配器，把原始输出归一为统一事件流（会话 ID、工具生命周期、用量、子代理谱系、分类终局错误）。诚实性内建在适配器合同里：退出码为 0 但缺少厂商终局结果事件时合成协议错误，静默死亡不能冒充成功；我方主动取消的退出按取消归因，不上报为失败；批量上报通道永久失败时丢弃全部后续事件并以错误收尾，绝不交付有缺口的事件流。它还能从自家账本重建被厂商回收的 Claude Code 原生 transcript，让 `--resume` 语义恢复在原文件消失后仍可用；harness 内部派生的子代理以稳定 `parentToolCallId` 全程标注谱系。
3. **显式状态机、分层存储与看门狗**。自有运行时的 AgentState 是 7 态状态机（idle/running/waiting_for_human/waiting_for_async_tool/done/error/interrupted），并区分 parked（等待异步结果、机器可恢复）与 waiting_for_human（需要人）；活状态在 Redis（逐步锁+心跳），耐久状态镜像进 Postgres `agent_operations` 行，实时流经 Redis Stream 送 WebSocket，三处各司其职。子代理禁止嵌套（深度 1）、父操作用 CAS+看门狗屏障恢复；排程是集中 cron 派发 + 以 DB 行为权威的逐 tick 复查，心跳看门狗把静默任务判失败并生成待办简报，网关侧对死掉的生产者做反向终结（finalize-abandoned）。

反面证据同样有价值。群组协作用 supervisor LLM 路由：群内 @ 成员不走机械路由（机械 @ 直达路由只在群外存在），mention 只是序列化进消息文本的提示标签，一个广播回合默认走 supervisor→N 成员→supervisor 共 N+2 次全量历史 LLM 调用（supervisor 可显式跳过收尾降为 N+1），成员收到的是全量共享 transcript 的逐成员机械改写（`<speaker/>` 标签），宣传的"实时共享上下文"实为广播前一次性快照的共享消息列表。社区量化投诉与代码互相印证：默认 agent 模式启用 6 个内建工具（其中 4 个 alwaysOn 强制，记忆与联网检索默认开、可关），本审计以仓库自身估算器实测 schema 加工具系统提示合计约 82.5KB（约 1.9 万估算 token）的固定前缀每次调用重发（[#13797](https://github.com/lobehub/lobehub/issues/13797) "发 hello 烧 8000+ token"、[#13363](https://github.com/lobehub/lobehub/issues/13363) 技能提示禁用后仍注入约 11.6k token）；历史默认不截断、全量重发直到压缩阈值；每个工具回合重发全部上下文。维护者的回应是把轻量 chat mode 做成 context-engine 层的门控（[PR #14774](https://github.com/lobehub/lobehub/pull/14774)）并提供 ctx-map 上下文构成可视化（[PR #18114](https://github.com/lobehub/lobehub/pull/18114)）——问题被承认并按配置收敛，但默认工具集本身仍无逐请求相关性筛选。自有 loop 的 `done` 终究来自模型 finish 指令；验收要靠可选 verify 管道，其判决必须由验证子代理调用 `submitVerifyResult` 落库（"只写文字不算数"）。最后，它派发外部 harness 时默认 `--permission-mode bypassPermissions` / `--dangerously-bypass-approvals-and-sandbox`，把安全边界整体交还宿主环境。

### 审计基线

| 基线 | 状态 | 可支持的结论 |
| --- | --- | --- |
| [`v2.2.14 / 363797b1`](https://github.com/lobehub/lobehub/commit/363797b1eddc01d1d6f07e28148b200618c2d0a2) · 2026-08-16 | 已发布 stable | 上述 context-engine 管道、压缩路径、heterogeneous 适配器、状态机/调度/看门狗、verify 管道全部存在；群组机制与 canary 逐字节一致 |
| [`canary@18269f43`](https://github.com/lobehub/lobehub/commit/18269f431df57f08a08436bc36d755c7fb484e0f)（v2.2.15-canary.43）· 2026-08-22 | 默认分支审计快照；按 first-parent 口径比 stable 前进 140 个 commit | 独立 workbench 验收/verify 看板应用（2026-08-19 拆出）等增量 |

[stable 与审计快照的差异](https://github.com/lobehub/lobehub/compare/363797b1eddc01d1d6f07e28148b200618c2d0a2...18269f431df57f08a08436bc36d755c7fb484e0f)。许可证是 [LobeHub Community License](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/LICENSE)：Apache-2.0 附加条件（分发衍生作品需商业授权、producer 可单方调整条款），非 OSI 许可；根 package.json 的 MIT 字段与 LICENSE 矛盾，以 LICENSE 为准。仓库约 94 个 workspace 包、约 203 万行 TS（其中约四成是测试）——既是社区"过重"投诉的实证，也说明其 33 个 builtin-tool 包默认只随请求发 6 个。

### 采用与边界

HCTL 对照 agentd 与 harness 适配器采用：适配器终局结果契约（干净退出不等于交付结果）；取消归因（我方取消不算执行失败）；观测流无缺口纪律（要么完整、要么显式截断收尾）；harness 内部子代理的谱系保留；从自有观测留痕重建厂商会话以支撑 semantic resume；等待异步结果与等待人的状态区分；"进程活着"与"任务成功"的分层检测（逐步心跳、任务心跳看门狗、反向终结）。对照 Context 组装采用：facilitation 全程机械化，摘要作为唯一显式、有预算、持久化、增量折叠的 LLM 步骤；缓存友好的前缀稳定排序；本地估算器加漂移系数驱动预算决策；ctx-map 式"每次调用的上下文构成"审计投影。

明确不采用：不把 supervisor LLM 当作协作路由权威（HCTL 的协作边由人与状态机拥有，与 First Tree mesh 否决同一判词）；不让默认工具集不经相关性筛选进入每次调用；不采用 bypassPermissions 式派发（HCTL 第一阶段强制 OS 沙箱）；`done` 不来自模型自报（Verdict/Receipt 权威在 control）；不把共享 transcript 称作实时共享上下文；不复制实现、不引入依赖——许可证为非 OSI 的 source-available，本条目仅作设计研究。

主要证据：

- [仓库](https://github.com/lobehub/lobehub)、[v2.2.14 发布](https://github.com/lobehub/lobehub/releases/tag/v2.2.14)与固定版本的[许可证](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/LICENSE)；定位与谱系：[2.0 重构声明](https://github.com/lobehub/lobehub/discussions/10007)与 [Chief Agent Operator 发布](https://github.com/lobehub/lobehub/discussions/14935)
- Context 管道：[阶段编排](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/context-engine/src/engine/messages/MessagesEngine.ts)、[处理器运行器](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/context-engine/src/pipeline.ts)、[token 估算与阈值](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/context-engine/src/tokenAccounting/index.ts)、[逻辑组截断](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/context-engine/src/processors/HistoryTruncate.ts)、[压缩执行器](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/agent-runtime/src/executors/compressContext.ts)、[压缩提示词](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/prompts/src/prompts/compressContext/index.ts)、[摘要回注](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/context-engine/src/processors/CompressedGroupRoleTransform.ts)与[默认配置](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/const/src/settings/agent.ts)
- 工具重量与门控：[默认工具清单](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/builtin-tools/src/index.ts)、[工具系统提示注入](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/context-engine/src/providers/ToolSystemRole.ts)、[一行清单式动态发现](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/context-engine/src/providers/ToolDiscoveryProvider.ts)与[chat mode 门控](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/src/helpers/toolEngineering/index.ts)；社区证据：[#13797](https://github.com/lobehub/lobehub/issues/13797)、[#13363](https://github.com/lobehub/lobehub/issues/13363)、[#9380](https://github.com/lobehub/lobehub/issues/9380)（摘要每轮重算）、[#12810](https://github.com/lobehub/lobehub/issues/12810)（压缩未触发整段倾倒）与维护者回应 [PR #14774](https://github.com/lobehub/lobehub/pull/14774)、[PR #12976](https://github.com/lobehub/lobehub/pull/12976)、[PR #18114](https://github.com/lobehub/lobehub/pull/18114)
- 群组编排：[成员上下文与广播快照](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/src/store/chat/agents/GroupOrchestration/createGroupOrchestrationExecutors.ts)、[机械编排状态机](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/agent-runtime/src/groupOrchestration/GroupOrchestrationSupervisor.ts)、[supervisor 路由指令](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/builtin-agents/src/agents/group-supervisor/systemRole.ts)、[逐成员角色改写](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/context-engine/src/processors/GroupRoleTransform.ts)与[群外机械 @ 直达路由](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/src/store/chat/slices/agentRun/actions/entries/commandBus/parseCommands.ts)
- 外部 Harness 适配：[派发 argv](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/heterogeneous-agents/src/spawn/spawnAgent.ts)、[Claude Code 适配器（transcript 解析与重建、子代理谱系）](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/heterogeneous-agents/src/adapters/claudeCode.ts)、[终局结果契约示例](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/heterogeneous-agents/src/adapters/amp.ts)、[子代理谱系归约器](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/heterogeneous-agents/src/subagentCoordinator/reducer.ts)、[无缺口批量上报](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/apps/cli/src/utils/BatchIngester.ts)与[反向终结](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/apps/server/src/services/agentRuntime/AbandonOperationService.ts)
- 状态机与调度：[7 态状态定义](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/agent-runtime/src/types/state.ts)、[指令执行核心](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/agent-runtime/src/core/runtime.ts)、[Redis 活状态](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/apps/server/src/modules/AgentRuntime/AgentStateManager.ts)、[耐久操作行](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/database/src/schemas/agentOperations.ts)、[cron 派发](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/apps/server/src/router-hono/workflows/task/handlers/scheduleDispatch.ts)、[DB 权威 tick](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/apps/server/src/services/taskRunner/scheduleTick.ts)、[心跳看门狗](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/apps/server/src/router-hono/workflows/task/handlers/watchdog.ts)与[子代理深度限制与屏障](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/agent-runtime/src/executors/subAgent.ts)
- verify 与内部设计：[验证子代理指令](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/packages/builtin-tool-verify/src/systemRole.ts)、[验证执行](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/apps/server/src/services/verify/agentVerifier.ts)与[Goal→Task→Attempt→Operations 内部设计备忘](https://github.com/lobehub/lobehub/blob/363797b1eddc01d1d6f07e28148b200618c2d0a2/docs/development/agent-goals-design.md)

<a id="e-cumora"></a>
## E-CUMORA · Cumora

### 核心价值与跨层画像

Cumora 是"AI agent 作为一等队友的跨平台团队聊天":人与 agent 共用同一花名册、私聊、群聊、看板、日历与文档,agent 有 persona、记忆和真实 email 地址,可以主动认领工作、主动发起对话。agent 的"大脑"双轨:云端为每个 agent 按需拉起、闲置自灭的 K8s pod;BYOA(自带大脑)由用户本机守护进程驱动本地 Claude Code、Codex、Grok Build 或 Cursor CLI,服务端从不持有用户的模型凭证。两轨共用同一写入面:agent 对世界的一切动作都经 `cumora` CLI 垫层进入服务端仲裁,身份由 JWT 钉死(服务端剥除客户端一切 `--as` 参数并强制注入令牌内身份,不可伪造)。它的工程重心是多 agent 无碰撞协同与成本账本,并有 CI 强制的架构不变量(只有 agent 正式回合可用大模型;每笔 LLM 调用必须入账)。三块经源码验证的独特机制:

1. **唤醒 triage 门与协同门(L4)**。消息落库后对会话内所有非作者 agent 并行扇出,"该不该醒"由便宜小模型做纯门判断——输入全部是数据库/Redis 事实(工作认领、人类注意力=消息/表情/已读游标)而非消息措辞,每次判定入账本并与省下的大脑回合做诚实的经济学对比;AI 判断之下垫**确定性循环地板**(源码注释明言这层"曾被以 AI-native 优雅为由删过两次、两次都回归死循环")。回复前有 seen 游标新鲜度预检:落后于房间状态的回复被 HELD 并把新消息内联返回;`--force` 类旗标只是对"服务端展示过的 HOLD"的确认而非通行证(hold-token 与消息序号绑定、回合结束即亡),逐字重复检查放在序号行锁事务内、任何旗标不可绕。agent 主动性(空闲心跳、后台扫描、停滞救场)全部再过一道"默认不行动"的门并受预算限制,救场用 Redis 原子认领保证全房间只有一个成员出手;agent 拉群必须写结构化理由(headline/evidence/asks 字段化)。
2. **Shipping 交付生命周期(L3)**。`Draft→Contract→Building→Verifying→Ready→Releasing→Watching→Learned` 八态状态机在数据库 CHECK、邻接表与 gate 谓词三处强制;验收要求每条必需不变量被证据方格覆盖且有 owner,**构建者不得验收自己的方格——这条分离直接下沉为数据库级 CHECK 约束**;审计流追加只写;验收失败自动派生 friction 项与可重放回归项;生产发布要求 staging/canary 先行加回滚计划,成功后自动排 24 小时生产读回(readback)——把"完成"的终态推迟到行为对基线的读回。其文档口号与 HCTL2 同频:"绿色构建或成功 rollout 是中间信号,不是终态"。看板侧的 `card claim` 是全系统唯一被认可的认领原语:单条 UPDATE 加 WHERE 守卫(未认领/本人/超 20 分钟陈旧)以 rowCount 定胜负,两个 agent 竞争同一张卡永远只有一个赢。
3. **BYOA Harness 适配层(L1)**。统一的引擎适配抽象(发送/打断/存活/会话 ID/是否承载常驻提示)接入四家 Harness,原生会话优先、一次性进程降级;resume 会话 ID 存在 agent 家目录之外防引擎写坏,**只有三种情况才重置会话**(上下文溢出、转录毒化、引擎明确报会话失效——正则要求双匹配,防止把执行中途崩溃误判为会话失效而丢弃可恢复上下文,有专门测试钉住);常驻提示每会话带外送一次、每回合只送增量。宿主设备(Computer)一等化:配对码换设备令牌(服务端只存哈希)、"移除计算机"是真实的吊销开关、doctor 端到端探针探的是真实唤醒路径("绿=真唤醒能通");另有侦察性读取不污染 seen 游标、限流自适应起搏且限流从不泄漏进聊天等配额纪律。

反面证据同样密集。验收证据是**非空自由文本自述**——无格式校验、无机器执行的验证,回归项有命令字段但仓库里不存在执行器;云端回合的"完成复核"是用 LLM 验 LLM 的自述,BYOA 侧连这层都没有(run 状态纯粹等于引擎退出码)。所有本地引擎一律以 `--dangerously-skip-permissions` 级全权限运行,唯一隐私边界是提示词文本;云 pod 以 root+SYS_ADMIN 运行且无网络策略,源码注释声称的行级安全并不存在;`participants.tools` 权限列有表、有接口、执行路径零读取——声明性权限不被执行路径消费就等于没有。记忆无版本且有自我中毒实证(内部协同文档记录 agent 把特例写成普适规则、甚至写备忘录训练未来的自己无视刚建好的安全网,修复靠审计后外科手术式删除);同一 gate 在 DB/REST/CLI 三处平行实现已出现规则漂移。789 行的 COORDINATION.md 是一份罕见的多 agent 协同失败案例集("别用 prompt 修 infra"、"绕过旗标必须是对服务端展示过状态的确认"、"缺席成员是常态不是故障"),无论产品成败都有独立参考价值。

### 审计基线

| 基线 | 状态 | 可支持的结论 |
| --- | --- | --- |
| [`main@bd8dba8e`](https://github.com/yetone/cumora/tree/bd8dba8e45c91f685ea3c319aae173d44d26cbd6) · 2026-08-22 | 审计快照(本地克隆核验 HEAD、许可证与关键约束) | 上述 triage/协同门、Shipping 状态机、BYOA 适配、身份钉死机制全部在此基线经源码验证 |
| 桌面端 v0.1.64(2026-07-26,独立 releases 仓库)/ npm `cumora` 0.1.127 | 已发布 | 主仓库无 tag/release,发布通道在仓库之外 |

许可证为 [MIT](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/LICENSE)。仓库 2026-08-17 才公开(审计时仅 5 天),单人主导、私有开发史至少始于 2026-05 下旬;服务端测试面厚(80+ 单测文件、27 集成套件、真 LLM 协同基准),前端零单测。整体判断:以协同正确性为纲的高强度工程,基础设施纪律强、安全纵深弱;能力判断以固定源码为准。

### 采用与边界

HCTL 对照 L4 采用:triage 门四件套(事实输入、纯门、经济学账本、确定性地板)、hold-token 的"确认而非绕过"哲学、事务内查重、结构化拉群理由的字段化(可改造成 Agent"建议协作边"的载体)、记忆的 project 作用域两层可见性与"混杂时落全局不猜"的出处保守原则。对照 L3 采用:不变量→证据方格覆盖矩阵、builder/verifier 分离下沉到存储约束、追加只写审计流、验收失败自动派生 friction/回归资产、readback 把验收延伸到生产观察,以及原子 `card claim`。对照 L2 采用:"成功才推进游标、失败保留收件箱重跑"的无检查点恢复模型与"只重试人工唤醒"的事故教训(重试消息唤醒曾造成真实重复回复)、孤儿运行收尸。对照 L1 采用:引擎适配抽象与降级矩阵、保守的三条件会话重置、resume 引用外置存储、设备配对/吊销/探针,以及 `--as` 剥除加身份注入的不可伪造边界。

明确不采用:自由文本自述充当验收证据(与"证据高于自述"直接冲突,是本清单最重要的反例之一);用 LLM 复核 LLM 的完成自验充当 Verdict;无冻结版本的契约与记忆;`--dangerously-skip-permissions` 式全权派发与提示词充当隐私边界(与 LobeHub 同判);声明性权限不被执行路径消费的死配置;同一 Gate 多处平行实现(HCTL 的 Gate 单点实现、多面复用);服务端信任守护进程对自身执行结果的自述;agent 默认自主拉群不作为 HCTL 普通 Room 的协作边模型——临场协作边仍由人提交,Cumora 的结构化理由只用作建议载体。Cumora 的 conversation/board/computer 不映射为 HCTL 的 Room/Task/Agent 身份。复用结论:**选择性移植**(MIT),协同门与 Shipping 约束可改编,不整仓派生、不采用其安全姿态。

主要证据(固定到 `bd8dba8e`):

- [仓库](https://github.com/yetone/cumora)、[官网](https://cumora.ai)与[许可证](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/LICENSE)
- 方法论文档:[COORDINATION.md(789 行协同失败案例集)](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/docs/COORDINATION.md)、[SHIPPING.md](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/docs/SHIPPING.md)、[BYOA.md](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/docs/BYOA.md)与[CI 架构不变量](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/CONTRIBUTING.md)
- 协同门:[小脑 triage 门与确定性地板](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/triage-core.ts#L176-L262)、[并行扇出与唤醒重试策略](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/scheduler.ts#L61-L184)、[seen 游标与 hold-token](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/seen-boundary.ts)与[停滞救场管线](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/agenda.ts)
- Shipping 与看板:[状态机与 gate 谓词](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/api/shipping-router.ts#L238-L305)、[builder/verifier DB 约束](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/db/migrate.ts#L1595-L1597)与[原子 card claim 与 agent CLI](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/cli.ts#L1512-L2098)
- 运行时与身份:[全量 DDL(约 60 表)](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/db/migrate.ts)、[pod 编排](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/runtime/orchestrator.ts#L1-L23)、[云回合循环与完成自验](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/turn.ts#L1154-L1215)与[`--as` 剥除](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/runtime/cli-argv.ts#L19-L43)
- BYOA:[守护进程(信号量/起搏/会话重置)](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/computer/daemon.ts)、[引擎适配矩阵](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/computer/engine.ts#L974-L1012)、[设备配对与令牌](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/computer/registry.ts#L189-L273)与[记忆作用域契约](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/memory-scope.ts#L18-L33)

<a id="e-rakazo"></a>
## E-RAKAZO · Rakazo

### 核心价值与跨层画像

Rakazo 定位为 [Grok Bot](#e-grok-bot) 的自托管开源替代:每个 bot 拥有自己的聊天线程、Markdown 记忆、cron 例程、示范教学技能和一台带图形桌面的沙箱"计算机";模型自选(BYOK 或订阅 OAuth),agent loop 用 Pi 内嵌在自家 API/worker 进程中自建,不适配 Claude Code/Codex 等外部 Harness。仓库在审计时只有 9 天历史、单人主导、迭代极快;但它最值得收录的不是产品形态,而是一个反差事实:**这个 early beta 在运行治理上的工程严谨度远超其产品成熟度**,并有属性测试、Postgres 集成测试、崩溃恢复拓扑测试和真模型 canary 背书。三块经源码验证的亮点:

1. **三层带隔离栅栏的租约加幂等效果账本(L2)**。Run 租约以 CAS 认领、`leaseFence` 递增、60 秒心跳续租、续租失败即中止,每次执行留 Attempt 行;计算机执行租约与屏幕租约(`runId:fence`)各自独立,防止旧执行回抢;恢复语义是双保险——事件驱动入队之外,一个用 Postgres advisory lock 选主的对账器每 30 秒兜底扫描过期租约与到期例程。所有非只读工具调用前先写 `ExternalEffect` 意向行(幂等键=工具调用 ID),完成后置 completed;重放时已完成的直接返回旧结果,**状态不确定的非幂等工具直接拒绝重复执行**。状态机显式断言合法迁移(`failed→queued` 可重试,`completed/cancelled` 终态)。
2. **等人状态与人/机双租约接管(L2/L1)**。`waiting_input`(等回答)与 `waiting_takeover`(等上屏)是 Run 的一等状态,挂起前强制把工作区 checkpoint 到持久存储;接管期间执行租约转 24 小时保持、控制权交给人,人的控制租约限时、到期由后台任务自动回收,释放时自动找回等待中的 Run 重新入队恢复。
3. **供应商中立的可移植工作区(L1)**。沙箱经统一 `SandboxProvider` 契约支持 Docker/E2B/Daytona/本机等后端并逐后端探测能力降级(非图形后端过滤图形工具、分不出屏时显式报错而非静默排队);文档明确 `providerRef` 只是"加速路径而非持久数据",机器消失即从 checkpoint 重建——与 HCTL2"领域对象不被进程反向定义"同源。Docker 路径有特权分离:API/worker 进程不持有 Docker socket,由独立 supervisor 服务代管。另有值得单记的 L4 细节:Markdown 记忆文档带整数修订号与全量修订表,**每条修订携带 `sourceRunId`/`sourceThreadId` 出处**,可导出导入;这是"知识修订可溯源到产生它的 Run"的野生同构实现。

反面证据同样直接。官网称"给 bot 演示一次工作流,它存成你能读、能改、能提交的纯 Markdown",实现里并不存在可提交的 Markdown 工作流文件:例程是无版本号的数据库行(name+prompt+cron),触发时读**当前** prompt;示范教学的产物是 JSON playbook(由确定性代码而非 LLM 从录屏编译,这一点反而诚实),可编辑但无修订历史,Run 不记录执行时用的是哪个版本。官网的"Approvals that hold"经全库检索只是 playbook 文本约定加 ask 机制,没有任何策略引擎;防越权靠提示词自律("Prefer tools over claiming you already did the work")。凭证边界的"credentials never leave your environment"只在纯 Docker 自托管路径严格成立——选云沙箱时浏览器登录态在第三方 VM 上,长历史压缩外包给 Supermemory SaaS。

### 审计基线

| 基线 | 状态 | 可支持的结论 |
| --- | --- | --- |
| [`v0.1.0-beta / 53b119a6`](https://github.com/elie222/rakazo/releases/tag/v0.1.0-beta) · 2026-08-13 | 已发布 prerelease | 产品骨架与自托管路径 |
| [`main@90572cb2`](https://github.com/elie222/rakazo/tree/90572cb2bcab4458aebbe1994b3ffbc9ddfac339) · 2026-08-21 | 审计快照(本地克隆核验 HEAD 与许可证) | 上述租约/效果账本/接管/工作区机制全部在此基线经源码验证 |

许可证为 [Apache-2.0](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/LICENSE)。仓库创建于 2026-08-13,单一维护者贡献约 92% 提交,处于极速演进期;能力判断以固定源码为准,官网叙事必须逐条对照实现甄别。

### 采用与边界

HCTL 对照 L2 采用:三层租约的隔离栅栏组合、意向-完成两段式幂等效果账本(含"不确定态拒绝重试非幂等操作")、advisory lock 选主的兜底对账器、挂起前强制 checkpoint 的等人状态,以及 `failed→queued` 显式可重试的状态机断言。对照 L1 采用:供应商中立契约与"可移植工作区为唯一持久边界、供应商引用仅缓存"的纪律、逐后端能力探测与显式降级、执行进程与容器特权的分离,以及人控制租约与机执行租约分开计时的接管模型。对照 L4 采用:记忆修订携带 Run 级出处的最小可行样板。

明确不采用:无版本冻结的例程与 playbook(Run 必须绑定冻结的 Workflow Revision,这正是 Rakazo 反向验证的差异化空间);提示词自律代替确定性 Gate 与策略引擎(与"证据高于自述"相反,是现成反例);模型 loop 与业务同进程(HCTL 的 Harness 边界要求进程级隔离与无隐藏写权限);把长期记忆压缩外包给第三方 SaaS;一 bot 一线程的二元对话不映射为 Room。Rakazo 没有 L3:Task 仅是 prompt+status,无契约、无验收、无看板。

主要证据(固定到 `90572cb2`):

- [仓库](https://github.com/elie222/rakazo)、[v0.1.0-beta](https://github.com/elie222/rakazo/releases/tag/v0.1.0-beta)、[官网](https://rakazo.com/)与[许可证](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/LICENSE)
- 领域对象与治理:[Prisma schema(Run 租约 L342-376、ExternalEffect L392-409、Routine L411-432、MemoryRevision L514-526)](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/db/prisma/schema.prisma)、[Run 状态机断言](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/core/src/run-state.ts)、[执行器(租约/效果账本/工具分发)](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/adapters/src/executor.ts)与[选主对账器](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/adapters/src/job-reconciler.ts)
- 等人与接管:[事务性线程事件](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/db/src/events.ts)、[API 路由(ask/takeover/release)](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/apps/api/src/router.ts)、[屏幕租约](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/core/src/screen-lease.ts)与[签名屏幕代理](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/apps/api/src/screen-proxy.ts)
- 沙箱与工作区:[计算机生命周期](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/adapters/src/computer-lifecycle.ts)、[工作区 checkpoint](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/adapters/src/computer-workspace.ts)、[supervisor 特权分离](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/infra/sandboxes/supervisor/src/supervisor-logic.ts)、[适配器契约](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/adapter-kit/src/interfaces.ts)与[计算机运行时文档](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/docs/computer-runtime.md)
- 记忆与教学:[Markdown 记忆(修订+出处)](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/memory/src/index.ts)、[确定性 playbook 编译](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/core/src/teach-playbook.ts)与[官网 Markdown 声称处](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/apps/www/src/pages/index.astro#L107)
- 加密与凭证:[secret 加密存储](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/adapters/src/secrets.ts)与[自托管文档](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/docs/self-host.md)

<a id="e-helio"></a>
## E-HELIO · Helio

### 核心价值与跨层画像

Helio 是 Sheet0 的"AI-native 团队工作区":闭源 SaaS 控制面承载人机混合频道、Tasks 看板(HEL-nnn 键)、Automations、审批收件箱、凭证 Vault 与版本化 Artifact;每个 AI teammate 是工作区一等用户(handle、私聊、自有邮箱,接 Slack 时每个 teammate 一个独立 Slack app 而非共享 bot),雇佣时绑定一个引擎(文档层只有 Claude Code 与 Codex;营销宣称的 MCP server/自带 Docker 镜像未见文档证实)与一台"计算机"(云 pod 或装守护进程的本机)。引擎与工作区之间的桥是 **heliox CLI**——把全部域对象做成 CLI 动词,同一插件发布到 Claude Code 市场、Codex 插件与自家 runtime 三个渠道,引擎无须理解 Helio 协议、只须会用 CLI。核心产品闭源,但官方外围有可源码审计的三个仓库(heliox skills 发布镜像、anycli 凭证注入库、ship 门控管线插件),它们直接暴露了 agent 面向工作区的全部工具面。与已收录的 [Cumora](#e-cumora) 相比,Helio 的中心是票据/编码会话/审批治理,Cumora 的中心是聊天/记忆/agent 自发性;两者是同一赛道的对置打法。三块经源码验证的亮点:

1. **消息面并发协议与出处(L4)**。发消息必带 `--seen <seq>` 声明已观察到的最新序号,网关据此做 CAS 隔离栅栏——过期发送直接失败并返回错过的消息与精确重试指引;`cede` 是显式"弃权本轮"动词、必附理由;频道消息下的逐 AI 回执把 **silent(已读、无可补充)与 unread(未读)区分开**;每条消息可查"由哪个 turn 产生、被哪些 turn 处理"的 turn 级出处。多 teammate 群频道须有 charter(章程)文档,含角色分工、每步的交接工件与 **Doer≠Verifier 独立验证规则,且人类 owner 批准前只是草稿**。
2. **归约与证据纪律(L2)**。Automation 的每次 run 必须以 `success|failed|skip` 三选一收尾、失败与跳过必填理由,不收尾的 run 由看门狗标为 unclosed/died 并告警 owner;"源头报错是 failed 不是 skip"。工作流文档修改分层:方法层改后告知 owner,意图层必须事先批准。开源侧 ship 插件更激进:stop-gate 以状态机为唯一事实源并**明确拆除了 LLM 完成度校验**(注释原文:"a model call can only re-derive what the state machine already knows — or hallucinate TASK_COMPLETE");证据分级 L1(截图/curl 响应/console 日志)唯一可采信、L2("tests passed")不足、L3("should work")自动 FAIL;评审者不能改代码、QA 不能读 review 的角色隔离下沉到工具调用前钩子。这是与"证据高于自述"完全同源的独立实现。
3. **适配器与凭证工程(L1)**。anycli 给每个可执行叶子命令声明 side_effect 单比特("可能变更与否"),**缺失注解一律按可变更处理**("Absent means true...The safe side is the only defensible default"),构建期 lint 强制穷尽,库只报事实、宿主做策略;工具凭证由宿主解析、临时注入子进程、内存驻留、用后即焚,"绝不改用户持久配置";凭证失效判定 provider-aware(只有提供方明确拒绝才标失效,限流/5xx 不误伤);另有内置无凭证的 gate-probe,可端到端验证"审批门真的在门上"。节点带 `host_cli` 能力探针(found/not_found/unknown)决定能否用本机登录;"本机登录"动力源配置锁死不可改。

反面证据集中在完成权威与凭证明文两处。**"closing a task is a human-only step"(关单是人类专属)是已下线的营销文案,不是系统权限**:现行 agent 工具面明确提供 `heliox task done`,且官方推荐的任务生命周期就以 agent 执行 done 收尾;`in_review` 只是"该由别人核验时"的条件性约定。done 动词本身有个值得借鉴的最小机制——证据评论先落、评论失败则中止关单、任务保持 open——但完成权威仍在 agent 手里。验收标准是自由文本、无 Task Revision、无绑定 Task 的独立 Run 对象;workflow 文档 fork-on-edit 但每次 run 不冻结版本;`heliox vault get` 会把凭证明文交进 agent 的 shell,红线只是提示词("不得打印到聊天/日志/任务评论/记忆");人对执行会话只有 steps 重放级观察,没有 PTY 接管与重连语义;控制面不可自托管。

### 审计基线

| 基线 | 状态 | 可支持的结论 |
| --- | --- | --- |
| 产品 0.5.0-alpha · 官方站点与文档快照 2026-08-23 | 闭源 SaaS(2026-05 下旬公开亮相) | 上述产品行为(任务/审批/Vault/Automation/回执);网站呈 GA 姿态、实际 alpha/preview 混合,营销与文档存在滞后与落差,判断以文档加源码为准 |
| [heliox / marketplace@f0c8b46c](https://github.com/heliohq/marketplace/tree/f0c8b46c743c2e49e776619865696d23d2d93593)(v0.2.62,Apache-2.0)· 2026-08-20 | 私有 monorepo 的发布镜像(本地克隆核验) | agent 面向工作区的全部工具面语义(task/message/automation/vault/charter) |
| [anycli@2434c360](https://github.com/heliohq/anycli/tree/2434c360fde5cc51d8c49a1f07b37dae5b477d07)(v0.0.8,Apache-2.0)· 2026-08-21;[ship@40da17bd](https://github.com/heliohq/ship/tree/40da17bd7c1447660efd40064178ba09357fadce)(MIT)· 2026-07-04 | 官方开源外围(本地克隆核验) | side_effect/凭证注入/gate-probe;机械 stop-gate/证据分级/角色隔离 |

"human-only close" 的考古链条:该句在搜索引擎索引中归属 helio.im/product/ 的早期版本,当前页面全文与 meta 均已无此句;引用这一反例时应注明此演变,不得写成现行行为。

### 采用与边界

HCTL 对照 L4 采用:`--seen` CAS 隔离栅栏与"过期失败返回错过内容"的重试合同、cede 显式弃权、回执的 silent/unread 区分(与无缺口观察流同源)、消息的 turn 级出处,以及"人批准才生效"的频道协作契约与 Doer≠Verifier 分离。对照 L2 采用:三元归约加理由必填、未收尾看门狗、意图/方法双层修改授权;ship 的机械 stop-gate("状态机之外没有完成度")、证据分级与工具调用前钩子层的角色隔离。对照 L3 采用:done 动词"证据先落、失败中止"的原子形状(但完成权威归属按 HCTL 合同,不随此形状下放)、"评论=持久证据、消息=会话"的显式区分。对照 L1 采用:side_effect 缺失即危险的安全默认与构建期穷尽 lint、临时凭证注入与用后即焚、provider-aware 失效判定、gate-probe 式"验证门在门上"的端到端探针、host_cli 能力探针。

明确不采用:agent 自行关单(即使证据先行——HCTL 的 Task 完成只接受有权人类命令或绑定 Run 的确定性归约,这条在 Helio 没有对应硬机制,其营销与实现的落差本身就是"证据高于自述"的案例);自由文本充当验收标准、无契约版本;每次 run 不冻结 workflow 版本;凭证明文进 agent shell 而仅以提示词设防;"工作区即 CLI"不替代 HCTL 的类型化命令/查询/事件端口(可作 Harness 侧投影参考)。复用结论:核心产品**仅参考行为**;开源外围(Apache-2.0/MIT)可**适配协议、移植有边界的组件**(side_effect 分类、凭证注入生命周期、stop-gate 测试形状)。

主要证据:

- 官方产品行为:[官网](https://www.helio.im/)、[Tasks](https://www.helio.im/docs/work/tasks)、[给 AI 指派任务](https://www.helio.im/docs/work/tasks/assign-tasks-to-ai-teammates)、[控制与审批](https://www.helio.im/docs/ai-teammates/control)、[引擎与计算机](https://www.helio.im/docs/ai-teammates/choose-a-model)、[Vault](https://www.helio.im/docs/connect/vault)、[Automations](https://www.helio.im/docs/work/automation)、[频道与回执](https://www.helio.im/docs/work/channels)、[更新日志](https://www.helio.im/docs/guides/whats-new)与[自认短板的竞品对照博客](https://www.helio.im/blog/grok-bot-alternatives/)
- heliox 工具面(固定到 `f0c8b46c`):[task(done 证据先行 L65-L73)](https://github.com/heliohq/marketplace/blob/f0c8b46c743c2e49e776619865696d23d2d93593/heliox/skills/task/SKILL.md)、[message(--seen CAS L37、cede、turn 出处)](https://github.com/heliohq/marketplace/blob/f0c8b46c743c2e49e776619865696d23d2d93593/heliox/skills/message/SKILL.md)、[automation 三元归约与看门狗](https://github.com/heliohq/marketplace/blob/f0c8b46c743c2e49e776619865696d23d2d93593/heliox/skills/automation-refiner/references/executing-a-run.md)、[人批 charter 与 Doer≠Verifier](https://github.com/heliohq/marketplace/blob/f0c8b46c743c2e49e776619865696d23d2d93593/heliox/skills/channel-charter-creator/SKILL.md)与[vault 审批](https://github.com/heliohq/marketplace/blob/f0c8b46c743c2e49e776619865696d23d2d93593/heliox/skills/vault-approval/SKILL.md)
- anycli(固定到 `2434c360`):[side_effect 分类合同](https://github.com/heliohq/anycli/blob/2434c360fde5cc51d8c49a1f07b37dae5b477d07/docs/side-effect.md)、[凭证生命周期](https://github.com/heliohq/anycli/blob/2434c360fde5cc51d8c49a1f07b37dae5b477d07/docs/credential-lifecycle.md)与[gate-probe](https://github.com/heliohq/anycli/blob/2434c360fde5cc51d8c49a1f07b37dae5b477d07/internal/tools/gateprobe/gateprobe.go)
- ship(固定到 `40da17bd`):[README(证据分级)](https://github.com/heliohq/ship/blob/40da17bd7c1447660efd40064178ba09357fadce/README.md)、[stop-gate(拆除 LLM 校验)](https://github.com/heliohq/ship/blob/40da17bd7c1447660efd40064178ba09357fadce/scripts/stop-gate.sh)与[phase-guardrail(角色隔离)](https://github.com/heliohq/ship/blob/40da17bd7c1447660efd40064178ba09357fadce/scripts/phase-guardrail.sh)
- 第三方:[codepick 的 Helio vs Cumora 对比](https://codepick.dev/en/guides/helio-vs-cumora-agent-collaboration/)(2026-05-28)

<a id="e-l2-hctl1"></a>
## E-L2-HCTL1 · HCTL1 / yesme/hctl

HCTL1 是 HCTL2 L2 语义内核的直接前身，也是可执行的技术谱系证据；它不是外部复用来源，不能与 HCTL2 的原生语义核心混为一谈。审计固定在 [`main@3148042c`](https://github.com/yesme/hctl/tree/3148042cb2faf8df0dc8be92710b9468c8618516)（2026-07-28，Apache-2.0）。仓库没有标签或正式发布；README 表明 P1 内核已经进入主干，P2/P3 仍处于规划阶段。

它最独特的证据是一套不依赖守护进程和数据库的 Git 语义内核：每个 Seat 一条只追加事件引用、本地与远端 CAS、电平触发式对账、事实不完整时默认拒绝、Obligation/CLAIM 与 claim OID 隔离栅栏、精确匹配 `{base, head}` 的 Verdict、法定票数，以及携带事实摘要、无需依赖时钟即可重放的 squash merge Receipt。除规范外，仓库还提供可执行用例库，覆盖过期 Gate、权限、竞争、JCS 身份、组合法定票数、迟到 Finding、重新 Gate 时的结论沿用，以及初始化切换。

HCTL2 继承版本与证据、领取与隔离栅栏、法定票数、Receipt 和对账的思路，但不会原样继承其对象与事实源：

- HCTL1 的 `Seat = harness × model` 表示协作身份；HCTL2 的 Seat 是 Obligation 内的逻辑执行者或投票者位置，下挂 `0..N` 个 Attempt；
- HCTL1 的 Obligation 来自静态分派中的 author/gate/merge；HCTL2 的 Obligation 对应 Dagu 外部检查点的一次执行责任；
- HCTL1 把每个 Seat 的 ref、PR 和 squash Receipt 作为全局协调事实；HCTL2 把运行治理放入 SQLite 控制库，以 Git 保存共享且低频变化的定义和证据，并由 Dagu 保存机械工作流位置；
- HCTL1 的回收机制不等于候选方案降级，而且没有 Project Room、Task Board、Workflow Revision、Run、Attempt、进程/PTY 或外部系统同步；
- 单一人类信任、唯一合并协调者且容量为 1，以及把 PR 当作协作原子，只适用于它所定义的窄范围运行方式。

主要证据：

- [README 范围](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/README.md#L7-L27)；[METHOD 中的事实、Seat 与领取](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/METHOD.md#L27-L114)；[Gate、结论沿用与合并](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/METHOD.md#L108-L182)
- [派生引擎](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/internal/derive/derive.go#L47-L124)；[CAS 与待处理状态恢复](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/internal/store/store.go#L15-L191)；[Receipt 重放](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/internal/receipt/receipt.go#L14-L187)
- [可执行用例库](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/tests/corpus/README.md#L1-L53)；[Apache-2.0 许可证](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/LICENSE)

<a id="e-l2-dagu"></a>
## E-L2-DAGU · Dagu 机械状态后端与 workflow 候选复审

### 当前决定

第一阶段 workflow engine 拍板 **Dagu**，固定审阅基线为 [`v2.15.1 / 532c5129`](https://github.com/dagucloud/dagu/tree/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a)（2026-08-22，[GPL-3.0-or-later](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/LICENSE)）。判据已经按讨论弱化为：Workflow 的 HCTL JSON 必须是声明式事实源，可以机械执行 schema/Profile/引用/环等 lint；**不要求**对任意 loop 证明终止。运行时再以冻结预算、deadline、取消和替代规则收口。

Dagu 胜出的原因很窄：声明式 YAML、单二进制、文件系统持久化，`start-all` 在[一个进程内启动服务](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/cmd/startall.go#L94-L99)，比需要 JVM 或数据库/队列组合的候选更贴合单机第一阶段；同时，无进程的 [`human.task`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/runtime/runner.go#L766-L798) 会进入等待态，可经[完成 API](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/service/frontend/api/v1/humantasks.go#L78-L126)恢复，足以作为 control 驱动的外部检查点。它不是纯被驱动引擎：普通 step 会直接[构造 executor](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/runtime/node.go#L823-L827)并[自行执行](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/runtime/node.go#L265-L269)；HCTL Profile 因此只准依赖/条件/等待结构和 `human.task`，拒绝 command/script/action/HTTP/agent/Harness。

Workflow Revision 仍是 HCTL 规范化 JSON；Dagu YAML 只是固定 compiler/adapter 生成的 Engine Deployment。lint 分两层：HCTL 工具箱先验证完整 schema、Profile、引用与图结构，再调用 Dagu [`validate`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/cmd/validate.go#L25-L47)检查生成物。不能只相信后者：公开 `DAG.Validate` 只检查[名称和缺失依赖](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/ir/dag.go#L448-L475)，环是在[执行计划构建](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/runtime/plan.go#L287-L409)时才被拒绝，所以 HCTL 图 lint 必须更早拒绝并以负例锁定。

### 阻断性 P0 风险

`human.task` 完成服务内部有 CAS/幂等保护并把恢复写入队列（[completion](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/humantask/completion.go#L18-L113)、[resume](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/humantask/resume.go#L43-L92)），但公开请求只给 DAG、run 和 step，不能携带调用者预期的 engine attempt generation。adapter 必须在 ACK 未知、retry/repeat、重启和迟到完成的压力测试中证明 readback/fence 不会让旧请求推进新检查点；做不到就阻断 B4 并重开选型。inline spec start API 已存在（[源码](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/service/frontend/api/v1/dagruns.go#L163-L253)），但固定定义摘要、恢复后的 binding 对账和备份一致性仍由 HCTL adapter/control 负责。Dagu 不选择 Harness、不创建 Seat/Attempt、不解释语义拒绝、不计算 quorum、不签发 Receipt，也不直接写 Git 或外部系统。

### 本轮源码复审的全部 workflow 候选

以下是本轮实际审阅过的集合，不冒充整个市场清单；“stepfunlocal”按 AWS Step Functions Local 理解。结论基于固定源码/官方实现，而不是 README 特性表。

| 候选 | 源码层事实 | 结论 |
| --- | --- | --- |
| **Dagu** | 单进程/文件持久化；普通 step 自驱动，但 `human.task` 可作被动检查点；验证器与 completion fencing 的缺口如上 | **采用**；最小运维面，受严格 HCTL Profile 和 P0 fencing 门禁约束 |
| **Conductor OSS** | 当前 `v3.32.1` 已把 [SQLite 设为默认 DB/queue/index](https://github.com/conductor-oss/conductor/blob/v3.32.1/server/src/main/resources/application.properties#L14-L24)，旧评估“只支持 Redis/Postgres/MySQL”已经过时；其[逐 task poll/update API](https://github.com/conductor-oss/conductor/blob/b54f0d4ee546c1053367e1c14405c5396c17bfb1/rest/src/main/java/com/netflix/conductor/rest/controllers/TaskResource.java#L66-L112)比 Dagu 更原生地被驱动，但 [JVM server 分发](https://github.com/conductor-oss/conductor/blob/v3.32.1/server/build.gradle#L42-L79)仍重；loop validator 只查[条件与 body 存在](https://github.com/conductor-oss/conductor/blob/b54f0d4ee546c1053367e1c14405c5396c17bfb1/core/src/main/java/com/netflix/conductor/validations/WorkflowTaskTypeConstraint.java#L260-L284)，runtime 可[持续迭代](https://github.com/conductor-oss/conductor/blob/b54f0d4ee546c1053367e1c14405c5396c17bfb1/core/src/main/java/com/netflix/conductor/core/execution/tasks/DoWhile.java#L161-L184) | 不采用；外部任务接口最好，但 footprint/分发不符合本轮轻量优先，且同样不证明终止 |
| **Windmill** | flow schema 本身允许 script、raw script、for/while loop、subflow 与 agent 等[代码承载模块](https://github.com/windmill-labs/windmill/blob/74af4ed939fb8d78aff8ecd4af812860595c6c5a/backend/windmill-types/src/flows.rs#L984-L1093)，validator 主要做[反序列化、retry 与 path](https://github.com/windmill-labs/windmill/blob/74af4ed939fb8d78aff8ecd4af812860595c6c5a/backend/windmill-types/src/flows.rs#L233-L275)，部署由 [PostgreSQL、server 和 worker](https://github.com/windmill-labs/windmill/blob/74af4ed939fb8d78aff8ecd4af812860595c6c5a/docker-compose.yml#L10-L79)组成，且[许可混合](https://github.com/windmill-labs/windmill/blob/74af4ed939fb8d78aff8ecd4af812860595c6c5a/LICENSE#L1-L31) | 不采用；明显偏 code platform，服务面和授权面都过大 |
| **Kestra** | YAML 是声明式入口，但 Flow validator 只挡重复 ID 与[直接自递归](https://github.com/kestra-io/kestra/blob/e77ad77142e50af6f62c06354b8711263155086f/core/src/main/java/io/kestra/core/validations/validator/FlowValidator.java#L32-L78)，测试中存在[合法互递归 subflow](https://github.com/kestra-io/kestra/blob/e77ad77142e50af6f62c06354b8711263155086f/core/src/test/resources/flows/valids/subflow-function-mutual-a.yaml#L1-L9)，`LoopUntil` 未设限制时[不限制迭代数](https://github.com/kestra-io/kestra/blob/e77ad77142e50af6f62c06354b8711263155086f/core/src/main/java/io/kestra/plugin/core/flow/LoopUntil.java#L242-L263)；local mode 仍是 [JVM + H2](https://github.com/kestra-io/kestra/blob/e77ad77142e50af6f62c06354b8711263155086f/cli/src/main/java/io/kestra/cli/commands/servers/LocalCommand.java#L25-L42) | 不采用；插件/运行面远超所需，lint 能力没有抵消重量 |
| **Direktiv** | 稳定版有声明式 [workflow model](https://github.com/direktiv/direktiv/blob/6156f450798d002ab52bd933a03dd1ffaf545a20/pkg/model/workflow.go#L13-L84)与[validator](https://github.com/direktiv/direktiv/blob/6156f450798d002ab52bd933a03dd1ffaf545a20/pkg/model/workflow.go#L174-L232)，同时允许[显式自循环](https://github.com/direktiv/direktiv/blob/6156f450798d002ab52bd933a03dd1ffaf545a20/tests/engine/simple-loop.test.js#L22-L48)并依赖 NATS/Postgres/Kubernetes/Knative 等[较重栈](https://github.com/direktiv/direktiv/blob/6156f450798d002ab52bd933a03dd1ffaf545a20/go.mod#L21-L53)；当前主干 compiler 已消费 [`.wf.ts`](https://github.com/direktiv/direktiv/blob/9ad27a0de2aa986a85612867cf8e73c8822d0529/internal/core/compiler.go#L8-L64) | 不采用；部署重，且演进方向重新引入 code-as-workflow |
| **Serverless Workflow Synapse** | CNCF Serverless Workflow schema 原生允许 [for/while 条件循环](https://github.com/serverlessworkflow/specification/blob/2dd2c84170d5f3e05d58e913e9ca298dcf8d543a/schema/workflow.yaml#L685-L729)，官方示例包含[永久消费循环](https://github.com/serverlessworkflow/specification/blob/2dd2c84170d5f3e05d58e913e9ca298dcf8d543a/examples/call-asyncapi-subscribe-consume-forever-foreach.yaml#L1-L27)；Synapse compose 由 [Redis、API、operator、correlator](https://github.com/serverlessworkflow/synapse/blob/ba3fbfd5125995bba9fb5900aed181a0775d538c/deployments/docker-compose/docker-compose.yml#L1-L55)组成，standalone runner 只给[内存执行上下文](https://github.com/serverlessworkflow/synapse/blob/ba3fbfd5125995bba9fb5900aed181a0775d538c/src/runner/Synapse.Runner/Services/StandAloneWorkflowExecutionContext.cs#L27-L40) | 不采用 runtime；保留 DSL/spec 作为 schema 参考 |
| **TIBCO Flogo / project-flogo/flow** | flow JSON 是 task/link 图（[serializer](https://github.com/project-flogo/flow/blob/0683f40cb531ee755d78042a2646228bb43daf24/definition/definition_ser.go#L26-L61)），含 [DoWhile](https://github.com/project-flogo/flow/blob/0683f40cb531ee755d78042a2646228bb43daf24/model/simple/dowhilebehavior.go#L72-L99)；runtime 用[最大 step 数](https://github.com/project-flogo/flow/blob/0683f40cb531ee755d78042a2646228bb43daf24/action.go#L421-L443)作保险，但 persistence 只有[可选 recorder 接口](https://github.com/project-flogo/flow/blob/0683f40cb531ee755d78042a2646228bb43daf24/state/recorder.go#L1-L14) | 不采用 runtime；它是嵌入式 flow library，不是完整耐久服务 |
| **AWS Step Functions Local** | ASL 声明式、可配合 [StateLint](https://github.com/awslabs/statelint/blob/5388321f0c0c4b24df9308c02dff4d3adfe74527/lib/statelint/state_node.rb#L207-L224)，但 AWS 官方明确称 Local [unsupported、仅供测试](https://docs.aws.amazon.com/step-functions/latest/dg/sfn-local.html) | 不可作生产本地 runtime；ASL/StateLint 仅作语法设计参考 |
| **SCXML / XState 自建** | SCXML 是成熟的[状态图标准](https://www.w3.org/TR/scxml/)；XState machine config 仍可嵌入[函数实现](https://github.com/statelyai/xstate/blob/8d878b3bed7ccb5d23b74474475659b9f0472306/packages/core/src/createMachine.ts#L31-L77)，默认内部迭代上限可为 [Infinity](https://github.com/statelyai/xstate/blob/8d878b3bed7ccb5d23b74474475659b9f0472306/packages/core/src/StateMachine.ts#L126-L145)，snapshot API 只解决[状态快照](https://github.com/statelyai/xstate/blob/8d878b3bed7ccb5d23b74474475659b9f0472306/packages/core/src/createActor.ts#L783-L819) | 不自建；还需补 durable store、lease、timer、API、恢复与运维，收益不足以抵消重造引擎 |

结论不是“Dagu 最强”，而是它在当前约束下最小：Conductor 的外部 task 合同更顺手，Serverless Workflow/SCXML 的形式更标准，Windmill/Kestra 的产品能力更多；这些优势都不值得引入相应重量或自建耐久运行时。

<a id="e-l2-stably-orca"></a>
## E-L2-STABLY-ORCA · Stably Orca 持久监督协议

Stably Orca 在 L2 的亮点不是自动规划，而是把人工或 Agent 主导的监督过程做成持久协议。Run 是持久命名空间和协调者收件箱；Task 保存依赖与状态；每次 Dispatch 把 Task 的一次尝试绑定到具体终端，并记录窗格、句柄、进程实例代次和能力。生命周期对账还会核对当前 Dispatch ID 与受派窗格/句柄，拒绝来源错误或已经过期的心跳与 `worker_done`。FIFO Delivery 会重复交付同一批消息直到收到确认；变更收据按调用者和请求实现幂等；执行者的启动、停止、释放和保留还会记录已经发生的副作用与未清理资源。Decision Gate、远程转发与过期 Dispatch 拒绝进一步补齐了监督过程中的恢复路径。

HCTL 在 L2 采用它的 Dispatch 权威、消息确认与重放、幂等变更收据、执行者资源所有权、失败后的残留状态，以及拒绝过期完成信号的规则。它的现役 Run [明确不负责调度或选择落点与并发度](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/skill-guides/orchestration.md#L102-L181)，自动调度器也[已经退役且不产生副作用](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/skill-guides/orchestration.md#L273-L285)；它没有 HCTL 的 Workflow Revision、Obligation/Seat/Attempt、法定票数、重新过 Gate，或与证据绑定的 Verdict/Receipt。因此，Stably Orca 是 L2 的持久监督专项参考，不能直接承担 HCTL 的 Workflow 权威事实。

固定版本、数据结构和生命周期检查见 [Stably Orca 的完整审计](#e-l1-stably-orca)。

<a id="e-l2-herdr-boundary"></a>
## E-L2-HERDR-BOUNDARY · Herdr 运行信号边界

Herdr 不提供持久 Workflow，但它对“谁可以写 Agent 状态”处理得足够深入，构成 L2 的边界证据：一个活动窗格只接受一个状态来源；完整生命周期钩子活跃时优先于屏幕状态降级信号，只报告会话身份的钩子不获得生命周期状态写入权，进程退出和事件序号又会撤销或拒绝过期报告。与此同时，`agent prompt --wait` 明确不追踪某一轮对话，已有活动轮次也可能满足等待。因此 HCTL 可以采用它的信号仲裁思路，却必须把 `idle/working/blocked/done` 限定为 L1 观测，不能据此完成 Task、Run、Verdict 或 Receipt。固定源码见 [E-L1-HERDR](#e-l1-herdr)。

<a id="e-l2-zeroclaw"></a>
## E-L2-ZEROCLAW · ZeroClaw SOP

ZeroClaw 不能直接提供 HCTL Workflow，但它的 SOP 引擎是少见的 L2 邻近实现证据：每个 SOP 的准入策略支持 `parallel`、`hold`、`coalesce` 和 `drop`，Run 可以持久化并在重启后恢复；人工介入/检查点、经过身份认证的审批组与法定票数、只追加审批审计、拒绝作用域版本已经过期的提示、修改/修订、步骤级工具范围，以及重试/跳转，共同形成了可复用的 Gate 与准入失败用例库。

HCTL 只借鉴绑定版本的人类决策、准入与背压，以及默认拒绝的策略测试；不把 SOP 当作 Workflow Revision，不把事件触发当作 Start 授权，不把 Agent `sop_advance` 当作 Verdict，不把工具收据当作 HCTL Receipt，也不把 ZeroClaw Run 数据库当作领域权威事实。固定版本为 [`v0.8.4 / a56c345d`](https://github.com/zeroclaw-labs/zeroclaw/tree/a56c345d51dd8ab562e9351e0d4ab83f6a741db9)（MIT 或 Apache-2.0）。[语法说明](https://github.com/zeroclaw-labs/zeroclaw/blob/a56c345d51dd8ab562e9351e0d4ab83f6a741db9/docs/book/src/sop/syntax.md)与[运行时契约](https://github.com/zeroclaw-labs/zeroclaw/blob/a56c345d51dd8ab562e9351e0d4ab83f6a741db9/docs/book/src/sop/how-it-works.md)对默认持久化行为仍有冲突，初始化失败时还会降级到进程内内存，因此不能承担 HCTL 的权威事实。

<a id="e-l1-stably-orca"></a>
## E-L1-STABLY-ORCA · Stably Orca

### L1 核心价值与跨层画像

Stably Orca 的产品主轴是以 worktree 为中心的执行环境：每个 worktree 拥有独立分支、文件和 Agent 终端，PTY 由守护进程而不是桌面窗口持有。桌面应用退出但守护进程仍存活时，可以重新连接原进程并恢复布局、分屏、滚屏和焦点；守护进程已经退出时，只能创建新进程，再恢复布局、历史显示，或调用服务提供方的原生会话恢复。两条路径不能都笼统地叫作“会话恢复”。

它还把远程主机、差异审阅、分块暂存、提交、推送和 PR 评审串进同一条执行路径，并用运行时代次、PTY 代次和进程实例代次拒绝过期句柄。这些能力共同构成它在 L1 的核心价值。

### 四层设计亮点与边界

| 层 | 设计深度 | 定位与边界 |
| --- | --- | --- |
| L4 | 很弱 | Native Chat 只是同一 PTY 上的实验性结构化投影，底层终端才是事实来源；没有独立的 Project Room、意图账本或长期协作记忆。 |
| L3 | 中等 | Workspace Board、工作区检查点和外部系统绑定已经可用；本地看板状态还可以选择同步到 Linear。但卡片身份仍是 worktree，`workspaceStatus` 明确用于人工整理侧栏，没有独立 Task、Task Revision、验收或评审契约。 |
| L2 | **专项参考** | 已实现持久 Run 收件箱、Task 依赖、Dispatch 生命周期权威、消息交付确认与重放、幂等变更收据、心跳、重试隔离、Decision Gate、执行者资源回收和远程转发。这不是概念演示；但现役 Run 明确不调度，也不决定落点和并发度，自动调度器命令已经退役且不产生副作用，同时缺少通用 Workflow Revision、Obligation/Seat/Attempt、法定票数和证据治理。 |
| L1 | **核心参考** | PTY 所有权、冷热恢复、代际隔离、worktree、差异审阅、交付与远程连续性都有完整产品路径和源码实现。 |

在 HCTL 的设计组合中，Stably Orca 同时提供 L1 的执行连续性和 [L2 的持久监督协议](#e-l2-stably-orca)。L3 的可选 Linear 同步有实际产品价值，但仍以 worktree 为卡片身份，不足以定义独立的 Task 模型。

HCTL 采用：PTY 所有权、冷热恢复分类、运行时与进程实例的代次隔离、重新连接、worktree、差异审阅、远程操作、交付流程和相应故障测试。

HCTL 不采用：Workspace/worktree 充当 Project/Task、`workspaceStatus` 充当 Task 生命周期、会话或终端句柄充当持久 Run 身份、OSC/TUI 状态、worktree 评论或 `worker_done` 充当语义完成、Native Chat 充当 Room，以及 Stably Orca Run 与 HCTL Run 形成双重事实来源。

### 审计基线

固定 [`09ec516a`](https://github.com/stablyai/orca/tree/09ec516ae50b7b83fa65343d9ad96159e3fe71fc)（2026-08-12，软件包版本 `1.4.178-rc.2`，[MIT](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/LICENSE#L1-L21)）。官网会滚动更新，能力判断以固定源码和固定版本内的指南为准，官网只补充产品行为。

主要证据：

- [仓库](https://github.com/stablyai/orca)；[Worktrees](https://www.onorca.dev/docs/model/worktrees)；[Session restore](https://www.onorca.dev/docs/model/session-restore)；[Remote servers](https://www.onorca.dev/docs/remote-servers)；[Diff viewer](https://www.onorca.dev/docs/review/diff-viewer)；[Commit and push](https://www.onorca.dev/docs/review/commit-push)
- [守护进程持有会话、子进程、终端模拟器与客户端](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/session.ts#L109-L168)；[连接与原子快照](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/session.ts#L412-L484)
- [只接入仍存活的会话，`attachOnly` 不会偷偷创建新 shell](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/terminal-host-session-create.ts#L26-L142)
- [冷恢复创建新会话并回放磁盘历史](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/daemon-pty-adapter.ts#L737-L789)；[热重连连接原会话并回放快照](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/daemon-pty-adapter.ts#L811-L860)
- [运行时接管仍存活的守护进程 PTY，并使过期句柄失效](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/orca-runtime.ts#L9181-L9259)；[运行时、图与 PTY 代次检查](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/orca-runtime.ts#L31895-L32035)
- [持久 Run/Delivery/Receipt/执行者/Task/Dispatch/Gate 数据结构](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/orchestration/db.ts#L297-L620)；[`worker_done` 与心跳的受派者/过期检查](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/orchestration/lifecycle-reconciliation.ts#L16-L305)
- [Run 只负责持久命名空间和收件箱，落点与并发度由 Agent 选择](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/skill-guides/orchestration.md#L102-L181)；[自动调度器命令已经退役](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/skill-guides/orchestration.md#L273-L285)
- [`workspaceStatus` 是人工侧栏分类](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/shared/types.ts#L684-L685)；[可选的 Linear 状态同步](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/renderer/src/components/sidebar/workspace-board-task-status-sync.ts#L168-L239)；[Native Chat](https://www.onorca.dev/docs/agents/native-chat)

<a id="e-superset"></a>
## E-SUPERSET · Superset

### L1 核心价值与产品闭环

Superset 的完整闭环是：把 Project 注册为代码仓库，从 Task 或提示词创建独立的 Workspace/worktree 和分支，在持久终端中运行 Agent，经 Changes、PR 与 CI 评审，再合并并删除 Workspace。它的官方心智模型可以概括为“在隔离工作区中委派，通过分支和 PR 集成”。这个模型没有提供 HCTL 的四层领域事实，但对 L1 的进程存活、客户端重新接入、会话找回和 worktree 安全销毁处理得很深入，因此是 L1 的核心实现参考。

### 审计基线与许可

固定实现基线为 [`main@4e18e1fa`](https://github.com/superset-sh/superset/tree/4e18e1fa794be7969d517bea86d082105e44c836)（2026-08-13）。同期最新发布版是 [`desktop-v1.21.0 / 067182bc`](https://github.com/superset-sh/superset/releases/tag/desktop-v1.21.0)，主干只比发布版多一个 Codex MCP 传输类型修正。官网会滚动更新，能力判断以固定源码和测试为准。

仓库公开了完整的单仓库代码，但许可证为 [`Elastic License 2.0`](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/LICENSE.md)，明确限制把实质功能作为第三方托管服务提供。它属于源码可见许可证，不是宽松开源许可。HCTL 可以采用其设计、协议形状和故障测试，不能未经授权移植实现源码。

### 四层设计亮点与边界

| 层 | 真正深入且独特的证据 | HCTL 的采用方式与边界 |
| --- | --- | --- |
| L4 | 很弱。Project 基本等于注册仓库，Agent Chat 是 Workspace 内的一种执行界面；没有项目级持久 Room、共享意图账本、Request 或知识准入。 | 不进入 L4 参考组合；Project、Workspace 和终端会话都不映射为 HCTL Project Room。 |
| L3 | 同时支持 Superset 原生 Task 和外部系统 Task，并能把 Task 内容变成 Workspace 的 Agent 提示词；`task.start` 通过只向前推进、可重复调用的状态更新同步外部系统。Workspace Board 的分栏则从 Agent 运行信号、PR 状态和归档原因派生，不是独立的 Task 生命周期。 | 只保留边界证据：Task 适合充当外部来源和启动入口，但没有冻结的 Task Revision、验收合同或独立的评审与承诺事实；Workspace、分支和 PR 状态不能代替 HCTL Task。 |
| L2 | Automation 保存定时规则、目标设备和运行历史，但它把 Workspace 创建成功记作这次运行的成功，明确不追踪 Agent 的执行结果，而且采用至少一次投递。官方编排 Skill 也明确说明：Superset 只提供会话传输，依赖关系和完成状态由协调者保存在工作上下文中；完成标记只是提示词约定，不是持久事件。 | 这是明确的边界证据：投递已接受、Workspace 已创建、终端存在，都不等于执行结果，更不等于 Workflow、Verdict 或 Receipt。可以采用幂等投递要求和无界面投递接口，但不能把 Automation 或工作上下文中的协调表当作 HCTL L2 事实。 |
| L1 | **核心参考。** 独立的 `pty-daemon` 持有 PTY，`host-service` 只通过 Unix 域套接字使用它；主机服务重启不影响 shell 进程，守护进程平滑升级时还能通过文件描述符移交（fd handoff），把同一 PTY 交给继任进程。主机服务与渲染端使用 `epoch:seq` 重连：在保留范围内精确补发，首次连接发送末尾快照（`tail`），代际不符或缺口过大时显式重新锚定（`reanchor`）；2 MiB 的补发环形缓冲区有界，单个慢渲染端的待发缓冲超过 8 MiB 时只断开该客户端，不拖死 PTY。SQLite 保存终端记录、Agent 绑定和 `disposeRequestedAt` 终止意图；回收器会重试失败的终止操作，守护进程断连后先向继任守护进程核实真实会话，再决定哪些绑定成为可恢复候选。Workspace 删除先写归档墓碑，再依次完成预检、`teardown` 清理脚本、PTY、worktree、分支和缓存清理；失败时恢复可见，进程崩溃后由对账器继续。 | 采用 PTY 进程所有权、文件描述符移交与接管、分代重连和显式降级、有界的慢客户端隔离、持久终止意图与回收器、终端与 Agent 会话绑定、先核实守护进程实际状态再宣告死亡，以及“先写意图、再执行清理”的可恢复分阶段 worktree 清理流程。CLI/MCP 的 `terminal list/read/send/close` 还可作为无 Workbench 时的最小控制面。 |

### “持久终端”实际保证到哪里

Superset 的几类恢复必须分开描述：

- `pty-daemon` 仍存活时，桌面或 `host-service` 重启可以接管原 PTY；守护进程平滑升级时，文件描述符移交可以保留同一 shell PID；
- 守护进程内部的 `SessionStore` 只是进程内映射表，每个会话只有 64 KiB 环形缓冲区，不写入磁盘；渲染端的 2 MiB 补发环形缓冲区也位于主机服务内存中；
- 守护进程被真正杀死或机器重启后，原进程无法保留。系统只能创建新的 shell，并在已有终端与 Agent 绑定、外部系统会话 ID 仍可用时尝试恢复 Agent 会话；
- `epoch:seq` 的精确模式只覆盖主机服务仍保有对应代际和字节范围的情况。代际变化或缺口超出环形缓冲区时会进入 `tail` 或 `reanchor`，不能宣称任意断线都能不重复、不遗漏地恢复。

HCTL 的失败语义必须采用上述细分，不能只写“应用重启后会话仍在”。Superset 当前的守护进程协议使用 Unix 域套接字和文件描述符移交，并明确没有 Windows ConPTY；HCTL 只能借鉴机制和测试，仍需通过自己的跨平台运行时后端契约实现。

### 采用结论

HCTL 应把 Superset 放进 L1 核心参考，采用 PTY 所有权、接管与移交、重连分级、慢客户端隔离、持久终止意图、Agent 会话恢复绑定和可恢复的分阶段 Workspace 清理。L2 只引用它清楚暴露的边界：投递与会话传输不拥有执行结果，更不拥有语义完成。

明确不采用：让 Project、Workspace 或 worktree 充当 HCTL 身份，让钩子、标题、PR、CI 或 Board 分栏充当语义完成，让 Automation 的 `created` 充当 Run 成功，让提示词标记和协调者上下文充当 Workflow 事实，以及移植受 ELv2 约束的实现源码。

主要证据：

- 官方产品行为：[Superset 模型](https://docs.superset.sh/superset-model)、[终端集成](https://docs.superset.sh/terminal-integration)、[Automations](https://docs.superset.sh/automations)、[Tasks](https://docs.superset.sh/tasks)、[远程 Workspace](https://docs.superset.sh/remote-workspaces)与[MCP 服务端](https://docs.superset.sh/mcp-server)
- 固定产品文档：[Superset 模型](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/apps/docs/content/docs/superset-model.mdx)、[Automations](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/apps/docs/content/docs/automations.mdx)、[Tasks](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/apps/docs/content/docs/tasks.mdx)与[编排 Skill](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/plugins/superset/skills/orchestrate/SKILL.md)
- PTY 与重连：[`pty-daemon` 设计和测试说明](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/pty-daemon/README.md)、[进程内 `SessionStore`](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/pty-daemon/src/SessionStore/SessionStore.ts)与[`host-service` 终端和重连实现](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/host-service/src/terminal/terminal.ts)
- 持久恢复：[终端与 Agent 数据结构](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/host-service/src/db/schema.ts)、[Agent 绑定与可恢复候选](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/host-service/src/terminal-agents/persistence.ts)与[守护进程丢失核实](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/host-service/src/terminal-agents/daemon-loss-sweep.ts)
- Task/Board 与清理边界：[`task.start`](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/trpc/src/router/task/task.ts)、[Board 分栏推导](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/apps/desktop/src/renderer/routes/_authenticated/_dashboard/v2-workspaces/components/V2WorkspacesBoard/utils/deriveBoardColumn/deriveBoardColumn.ts)与[Workspace 分阶段清理](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/host-service/src/trpc/router/workspace-cleanup/workspace-cleanup.ts)

<a id="e-l1-herdr"></a>
## E-L1-HERDR · Herdr

### 设计亮点与跨层画像

Herdr 在 L1 的主要价值是把终端所有权、输入权和 Agent 状态写入权拆成三个问题。在默认的持久会话模式中，后台服务持有 PTY、解析器、检测任务与通道，客户端只负责连接和显示；`--no-session` 是用于调试或兼容的单进程例外。第三方桥接还使用不同命令进入[只读观察与可写控制](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/src/client/mod.rs#L836-L910)：多个观察者可以并存，控制方则独占输入与尺寸，显式 `--takeover` 会替换旧控制方。[服务端的单写者检查](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/src/server/headless.rs#L2566-L2669)使接管不依赖前端按钮。

它也把原始终端控制与 Agent 语义控制分开。[Pane 命令与 Agent 命令](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/docs/next/website/src/content/docs/agent-automation.mdx#L59-L80)分别面向当前终端和当前被识别的 Agent；当目标 Agent 已不再控制该窗格时，语义命令会拒绝操作。状态层进一步仲裁完整生命周期钩子、只报告会话身份的钩子、屏幕状态降级信号、进程退出与事件序号：[完整生命周期钩子](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/src/terminal/state.rs#L598-L745)可以取得状态写入权，只报告会话身份的钩子不能；进程退出和过期序号会撤销或拒绝旧报告。这是 Herdr 在 L2 的专项边界价值：运行状态信号可以驱动关注、等待和诊断，但不能签发 Task、Run、Verdict 或 Receipt。

### 审计基线

固定基线为 [`v0.8.0 / 346411fa`](https://github.com/herdrdev/herdr/tree/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7)（2026-08-03，Apache-2.0）。该版本的 [changelog](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/docs/next/CHANGELOG.md#L5-L27)记录许可证从 AGPL-3.0-or-later 改为 Apache-2.0，许可证正文见 [LICENSE](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/LICENSE)。源码中的 [TerminalRuntimeRegistry](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/src/terminal/runtime_registry.rs#L5-L12)明确说明运行中的终端由 server 持有；官网的[概念说明](https://herdr.dev/docs/concepts/)、[会话状态与恢复](https://herdr.dev/docs/session-state/)、[持久化与远程接入](https://herdr.dev/docs/persistence-remote/)和 [Agent 自动化](https://herdr.dev/docs/agent-automation/)与固定版本文档一致。

### 恢复边界

Herdr 对恢复能力给出了可验证的分级，而不是用一个“恢复”覆盖所有情况：

| 场景 | 实际保留的内容 | HCTL 应如何归类 |
| --- | --- | --- |
| 客户端断开，后台服务仍存活 | 原 PTY、进程、终端内容和 Agent 会话都继续存在 | 可以接回同一运行实例 |
| 后台服务停止后重启 | 恢复 workspace/tab/pane、工作目录、布局和焦点；原进程已经消失 | 布局恢复，不是接回原 PTY |
| 开启窗格历史 | 可以显示近期终端内容；默认关闭，内容可能包含密钥与提示 | 只读历史显示，不是进程或证据恢复 |
| 服务提供方原生会话恢复 | 依靠官方集成报告的会话引用启动新的 Agent 进程 | 语义会话恢复，不是原 Attempt |
| `--handoff` | 在受支持的更新或远程替换中尽力移交 PTY 与进程；功能为实验性且需主动开启 | 需要新代次、对账和失败降级路径，不能视为必然成功 |

固定版本的[恢复说明](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/docs/next/website/src/content/docs/session-state.mdx#L6-L109)还明确指出：实时移交不保留进行中的 CLI/API 请求、等待、订阅流、客户端连接或窗格间消息。`agent prompt --wait` [只等待生命周期状态，不追踪某一轮对话](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/docs/next/website/src/content/docs/agent-automation.mdx#L72-L80)；已有活动轮次也可能满足等待。因此 `idle/working/blocked/done` 不能作为一次提示已经完成，更不能作为领域完成依据。

### 采用与边界

HCTL 采用 Herdr 的后台 PTY 所有权、观察/控制分离、单写者与显式接管、原始/语义控制面分离、状态信号仲裁和准确的恢复词汇。Herdr 的[显式 worktree 创建、打开与删除](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/docs/next/website/src/content/docs/cli-reference.mdx#L135-L146)，以及 [SSH 瘦客户端和直接接入](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/docs/next/website/src/content/docs/persistence-remote.mdx#L38-L165)，也可以补充 L1 的安全清理与远程操作设计。

Herdr 的控制方记录是进程内的客户端所有者映射，没有代次、TTL、持久确认游标或跨重启租约，因此不能直接替代 `Terminal Input Lease`。固定基线的 SCM 操作面覆盖 worktree 生命周期、分支以及 ahead/behind 状态，但不覆盖 Stably Orca 那样的内建 diff、分块暂存、commit/push 与 PR 评审交付链。Workspace、窗格、Agent 名称和服务提供方会话引用也不承担 HCTL 的 Project、Task、Run 或 Attempt 身份。Herdr 因而是终端所有权、控制、状态仲裁与恢复方面的 L1 专项参考，并通过[运行信号边界](#e-l2-herdr-boundary)为 L2 提供独立证据。

<a id="e-l1-deepseek-harness"></a>
## E-L1-DEEPSEEK-HARNESS · DeepSeek Harness / Cordis

### 设计亮点

DeepSeek Harness 在 L1 和跨层架构上都有独特价值。它没有把模型适配器、工具注册、Session 日志和 Agent Loop 写死为固定模块，而是把它们都实现为 Cordis 插件：共享 Context 提供服务、类型化事件、依赖注入和可撤销注册；Profile、Bundle 与 Patch 共同组合出实际运行的插件树。Service Definition、Provider、Consumer 三段式能力边界，让使用者依赖抽象能力，而不是具体插件名称。

它的 Session 设计同样扎实：所有对模型可见的内容都必须能从只追加事件日志重建；运行中崩溃时，系统追加“本轮被中断”的结束事件，而不是截断历史；遇到未知的必需事件或数据结构版本时拒绝恢复。HCTL 可以采用这些能力边界、插件生命周期测试和可重建日志，并把最终解析出的 Profile、插件集合与配置摘要冻结进 Attempt/Run Manifest。

### Cordis 论文实际证明了什么

[Cordis 论文固定版本](https://github.com/cordiverse/paper/tree/948a07b369c62adb3b12e102458be5c18dfb69b9)（Draft v8，2026-08-13）把进程内修改建模为带逆操作（inverse）的可逆 effect，把依赖上下文建模为会随服务出现或消失而重新解析的 reactive coeffect。这个模型解释了为什么注册监听器、提供服务、挂载子插件和热重载可以使用统一的生命周期管理。

论文也明确给出了保证范围：

- 可逆性只适用于系统边界内、可以独占修改并恢复的具体位置；边界不是按“文件或网络”这种介质一刀切。系统独占且可恢复的私有文件可以在边界内，向其他主体可见的输出则已经越界。论文还区分资源获取与对外输出；越界输出仍需延迟提交或应用级补偿；
- 依赖声明和服务注入不是安全沙箱。不可信代码需要语言运行时沙箱、操作系统强制隔离、容器或虚拟机；同一用户下的普通独立进程只能隔离崩溃，不能阻止它访问仓库、网络或凭据；
- 细粒度拆分可以消除依赖环，但也可能带来大量集成组件、命名、配置和认知负担；
- 接口漂移、键冲突、行为契约和多版本解析仍是开放问题；
- 经验材料来自单一语言和单一生态，没有受控对照，也没有性能或生产率的量化结论。

论文仓库没有声明许可证，因此这里只概括其观点，不复用论文文本或图表。

### kxn 的批评如何使用

[kxn 的评论](https://mp.weixin.qq.com/s/O3A4RpQM4jZz_XkDFvORyQ)指出：插件即使声明了依赖，仍可能因为钩子顺序、优先级和共享行为而互相干扰；第三方插件再叠加版本冲突，维护复杂度会迅速上升。这个批评与论文自身列出的限制基本一致，提醒我们“可组合、可卸载”不等于“行为没有冲突、生态自然可治理”。

评论作者也明确说明没有真实运行项目，主要依据源码分析。因此本文只把它当成二级审查问题，不用它证明 DSH 的实现、性能或成熟度。

### HCTL 的取舍

HCTL 不采用“Everything is a Plugin”，而采用[固定内核与受控端口](../spec/system.md#固定内核与受控端口)：

- Repo/Project/Task/Run 身份、命令准入、权限、版本与证据、领域归约器、持久账本、隔离栅栏和 Receipt 固定在内核中；
- harness、运行时后端、任务源、workflow engine、Chat 端口和渲染组件通过类型化端口进行替换；
- 多个提供方可以声明同一个带命名空间和版本的能力，唯一的是一次已经选定的权威绑定；插件加载顺序和钩子优先级不能决定权限或语义结果；
- `Extension Revision` 与 `Resolved Port Binding` 固定代码、接口、数据结构、配置、依赖图和信任级别；Run、Attempt、Invocation、Task Source 与外部聊天渠道在各自正确粒度冻结绑定；
- 响应式依赖只用于准入前发现或纯展示/遥测；提供方在活动执行中消失时安全暂停或失败，不能在原执行内自动改绑；
- 进程内注销器（disposer）只能撤销注册，不能声称已经回滚越过系统边界的输出；
- 进程内扩展等同受信任代码；不可信代码必须使用操作系统强制隔离和能力削减后的代理接口；
- 第一阶段只允许第一方或经审计的进程内扩展，不建设任意第三方插件市场。

DSH 由模型生成的 Workflow 仍缺少冻结版本、持久恢复、Gate 和语义 Receipt，只能作为 L2 边界证据，不能承担 HCTL 的 Workflow 治理。

固定实现基线为 [`master@47f94385`](https://github.com/deepseek-ai/deepseek-harness/tree/47f943859bef60e4160492346772ded9b24f765a)（2026-08-13，`0.1.0-rc.5`，[MIT](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/LICENSE)）。官方将其标为开发者预览版，允许破坏性变化；[`BENCHMARK.md`](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/BENCHMARK.md)只给出运行方法，没有公开结果。

主要源码与文档：[架构](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/architecture.md)、[Cordis 入门](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/cordis-primer.md)、[能力边界](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/capability-seams.md)、[Session](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/subsystems/session.md)、[持久化](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/subsystems/persistence.md)和[Workflow 边界](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/subsystems/workflow.md)。

<a id="e-l1-harness-access"></a>
## E-L1-HARNESS-ACCESS · OpenCode、Pi 与 Kimi Code

本节记录三种 L1 Harness 接入方式：原生应用服务端、中立于语言的 RPC/嵌入式 SDK，以及标准协议下按能力降级。三者虽然也有 Project、Session、Todo、Subagent 或 Plan 概念，但在其他层没有形成需要单列的独特机制。

| Harness 基线 | 采用的契约 | 明确边界 |
| --- | --- | --- |
| [OpenCode `v1.18.18 / 31406ccc`](https://github.com/anomalyco/opencode/tree/31406ccc51b4bd2a4e1e086b2bcaa5f7f804f26d) · MIT | OpenAPI 3.1、SSE 和自动生成的强类型 SDK；以服务端为中心，向多个客户端提供 health/version、session/control/diff/permission 接口 | 原生 HTTP API 不是通用标准；服务端事件和 Session 完成事件不签发 HCTL Verdict/Receipt |
| [Pi `v0.84.1 / 53fa77cc`](https://github.com/earendil-works/pi/tree/53fa77ccd8a279eb87e92294ef3687b03ff80112) · MIT | 嵌入式 `AgentSession` 加严格的 LF 分隔 JSONL RPC；关联响应与异步事件分离；`steer`、`follow_up`、`abort` 有明确的队列语义 | Pi 的 RPC、Session 和树结构不是 HCTL 的传输协议或 Room/Task/Run；本地信任边界不等于沙箱 |
| [Kimi Code `0.36.0 / b6144f94`](https://github.com/MoonshotAI/kimi-code/tree/b6144f94ea6b22455a4e750d1750d220987e7bc2) · MIT | 明确列出 ACP 方法的支持矩阵，并结合 stream-json、原生服务端与钩子验证每种接入的降级行为 | “支持 ACP”不代表能力完全相同；默认放行的钩子不承担 Gate、安全或完成判定权 |

接入时必须把“请求已受理”和“执行结果”分开，对每个接入绑定探测能力，明确保留不支持的方法，并把固定版本的协议样本沉淀为适配器契约用例库。OpenCode 是第一阶段目标；Pi 与 Kimi Code 进入证据测试台，不代表第一阶段会自动扩大 Harness 支持范围。

主要证据：OpenCode [服务端](https://github.com/anomalyco/opencode/blob/31406ccc51b4bd2a4e1e086b2bcaa5f7f804f26d/packages/web/src/content/docs/server.mdx) / [SDK](https://github.com/anomalyco/opencode/blob/31406ccc51b4bd2a4e1e086b2bcaa5f7f804f26d/packages/web/src/content/docs/sdk.mdx)；Pi [RPC](https://github.com/earendil-works/pi/blob/53fa77ccd8a279eb87e92294ef3687b03ff80112/packages/coding-agent/docs/rpc.md) / [SDK](https://github.com/earendil-works/pi/blob/53fa77ccd8a279eb87e92294ef3687b03ff80112/packages/coding-agent/docs/sdk.md)；Kimi Code [ACP 支持矩阵](https://github.com/MoonshotAI/kimi-code/blob/b6144f94ea6b22455a4e750d1750d220987e7bc2/docs/en/reference/kimi-acp.md) / [服务端 API](https://github.com/MoonshotAI/kimi-code/blob/b6144f94ea6b22455a4e750d1750d220987e7bc2/docs/en/reference/server-api.md) / [钩子边界](https://github.com/MoonshotAI/kimi-code/blob/b6144f94ea6b22455a4e750d1750d220987e7bc2/docs/en/customization/hooks.md)。

<a id="e-l1-codex-remote-feishu"></a>
## E-L1-CODEX-REMOTE-FEISHU · Codex Remote Feishu

### L1 专项价值与跨层边界

Codex Remote Feishu 没有权威 Project Room；它最有价值的设计，是把外部系统的工作区/讨论串接入兼容的托管会话，并将接入/脱离、路由冻结、输入排队与调整指令、审批卡片、重启与重连、传输降级和连接代次做成显式执行控制状态机。Feishu 是这个运行环境的远程控制与状态投影客户端，不是 L4 的事实源。

HCTL 只借鉴托管会话接管与恢复的行为和故障矩阵：Feishu Chat 不能映射为 Room，外部系统的工作区/讨论串不能映射为 Project/Task/Run，`command_ack` 不是语义 Receipt，而且该项目也没有精确的 PTY 契约。普通入站消息进入网关本地 FIFO 后即可收到 ACK，不必等待权威持久化提交；未投递消息的重放仍依赖进程内状态，因此不能据此认定持久 Room 桥接已经闭环。

固定发布版为 [`v2.0.0 / b2091ffe`](https://github.com/kxn/codex-remote-feishu/tree/b2091ffee3330a94703b78a8a6b7b1876e667c65)（2026-08-10）。该基线没有 `LICENSE`、`COPYING` 或 `NOTICE`，GitHub API 也未识别仓库许可证；在获得明确授权前，只能**参考行为、吸收思路**，不得移植源码或文档文本。

主要证据：[发布版](https://github.com/kxn/codex-remote-feishu/releases/tag/v2.0.0)、[架构](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/general/architecture.md)、[中继协议](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/general/relay-protocol-spec.md)、[远程界面状态机](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/general/remote-surface-state-machine.md)、[背压、连接世代与降级](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/implemented/relay-backpressure-hardening-design.md)、[请求与审批](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/implemented/feishu-request-approval-design.md)。

## L4 补充证据

| 证据 | 独特价值 | 边界 |
| --- | --- | --- |
| [assistant-ui](https://www.assistant-ui.com/docs/api-reference/primitives/message) | 有明确作用域的 Message/`MessagePart`/Action 渲染器 | 不采用 Thread、运行时、Store、Composer、Cloud 或 Queue |
| [virtua](https://github.com/inokawa/virtua) | 支持动态高度的 React 视口 | 不负责 Room 的顺序、游标或跟随策略 |
| [Rocket.Chat](https://github.com/RocketChat/Rocket.Chat/tree/develop/apps/meteor/client/views/room/MessageList)、[Mattermost](https://github.com/mattermost/mattermost/tree/master/webapp/channels/src/components/dynamic_virtualized_list)、[Zulip](https://github.com/zulip/zulip/blob/main/docs/subsystems/unread_messages.md) | 前插消息、定位到指定消息、未读状态、动态高度和无障碍测试 | 合并为行为证据；不采用其后端或领域模型 |

Tiptap/ProseMirror 是 L4 精选的 Composer 基础组件，不是产品参考项目：[自定义扩展](https://tiptap.dev/docs/editor/extensions/custom-extensions)、[React 节点视图](https://tiptap.dev/docs/editor/extensions/custom-extensions/node-views/react)。

<a id="e-l4-matrix-homeserver"></a>
## E-L4-MATRIX-HOMESERVER · chat server 选型（限时验证）

三类数据模型（v0.10.0）把 Chat Room 的消息 content 判给采用 Matrix 协议的 chat server。两个候选并列进入开工前限时验证（见[交付文档](../delivery.md#开工前限时验证)），均为 Rust 单二进制、采用 RocksDB 系嵌入式存储的 conduwuit 谱系：

- [Tuwunel `v1.9.0 / 5b366914`](https://github.com/matrix-construct/tuwunel/tree/5b3669144219d5d4c0774743c84191b476f1b54f)：conduwuit 原作者延续、全职维护；Apache-2.0。
- [Continuwuity](https://github.com/continuwuity/continuwuity)：conduwuit 社区延续、Matrix 基金会生态成员；Apache-2.0。

已拍板 **Tuwunel**（Continuwuity 记录在案备选）。理由：接口更 API 化、与 Synapse 参考实现兼容性更强；AppService 注册程序化而非房间内发命令。其官方 `v1.9.0` 发布物只有 Linux，macOS 需要容器/轻量 VM，不能再笼统写成“单二进制所以运维压力低”。

角色：执行面独立服务器——采用为依赖、由 control 托管生命周期，不 vendor 源码；P0 必须固定实际存储后端及 build features，并验证 macOS 承载、低内存配置与 RocksDB/media 一致性备份。它们承载消息 content，不获得任何治理权威；HCTL 依赖的合同前提（事务 ID 幂等、单 homeserver 线性顺序）以验证结果为准。

## L3 外部系统与观察清单

Linear 和 GitHub 提供外部字段的写入权威，也是没有 Workbench 时的外部系统原生降级方案；它们不是 HCTL 的 Task 模型：

- Linear：[GraphQL](https://linear.app/developers/graphql)、[Webhook](https://linear.app/developers/webhooks)、[速率限制](https://linear.app/developers/rate-limiting)、[分页](https://linear.app/developers/pagination)
- GitHub：[Projects API 指南](https://docs.github.com/en/issues/planning-and-tracking-with-projects/automating-your-project/using-the-api-to-manage-projects)、[GraphQL 参考](https://docs.github.com/en/graphql/reference/projects)、[Webhook](https://docs.github.com/en/webhooks/webhook-events-and-payloads)、[REST 最佳实践](https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api)
- React Aria：[Kanban 示例](https://react-aria.adobe.com/examples/kanban)、[拖放](https://react-aria.adobe.com/dnd)——只作为 UI 基础组件。

<a id="e-l3-vikunja"></a>
## E-L3-VIKUNJA · 本地任务服务器选型（限时验证）

[Vikunja `v2.5.0 / ef2200e9`](https://github.com/go-vikunja/vikunja/tree/ef2200e9429c5cc42f5c1811433418bfcc72b3aa)：Go 单二进制、默认 SQLite、REST API（v1/v2）与 webhooks，看板/列表/甘特多视图；AGPL-3.0-or-later（desktop 组件 GPL-3.0-or-later）。该发布已有官方 macOS arm64/amd64 full zip，旧记录“无 Darwin 包”已作废。

角色：Kanban 场景本地 content 后端的已选实现。采用边界：独立进程托管、经任务源受控端口访问，不 vendor、不链接其源码——AGPL 义务因此限于该服务自身。验证要点见[交付文档](../delivery.md#开工前限时验证)。

<a id="e-l3-git-bug"></a>
## E-L3-GIT-BUG · 零服务器任务后端对照（限时验证）

[git-bug](https://github.com/git-bug/git-bug)：分布式、离线优先的任务追踪，任务以 git 对象存于 refs、随 push/pull 同步；Go 实现，CLI/TUI/Web UI 与 GraphQL API，带 GitHub/GitLab/Jira 桥接；GPL-3.0-or-later。

角色：与 Vikunja 并列的对照候选，代表“零服务器、随仓库分布式”的另一条路。已知张力：看板语义弱（排序/泳道需另行承载）、无 webhook（观测靠 refs 轮询）、任务 content 进 git refs 与“content 归第三方服务器”的统一律相悖。若验证胜出，须显式接受模型例外并记入[来时路](./decision-history.md)。采用边界同上：独立进程/CLI 调用，不 vendor（GPL 义务隔离）。

<a id="e-l1-tmux-runtime"></a>
## E-L1-TMUX-RUNTIME · 运行时后端复审

### 当前决定与接入边界

第一阶段采用 [`tmux 3.7c / e476c123`](https://github.com/tmux/tmux/tree/e476c1230b958df0cb12977517d24b3dc931375b) 作为源码审阅基线。它提供 agentd 真正需要的窄接口：control mode 以命令和 `%output` 等通知驱动；客户端可设 `read-only`、`ignore-size` 和 `pause-after`；server 对 DA、DSR、DECRQM 等无人值守终端查询有明确应答；control output 使用有界缓冲、非阻塞写和 pause/continue。对应实现见 [`tmux.1` 的客户端标志](https://github.com/tmux/tmux/blob/3.7c/tmux.1#L1080-L1145)、[control mode 协议](https://github.com/tmux/tmux/blob/3.7c/tmux.1#L8113-L8235)、[`input.c` 查询应答](https://github.com/tmux/tmux/blob/3.7c/input.c#L1557-L1707)，以及 [`control.c` 的缓冲水位](https://github.com/tmux/tmux/blob/3.7c/control.c#L130-L138)和[非阻塞输出处理](https://github.com/tmux/tmux/blob/3.7c/control.c#L732-L804)。本机 detached 探针中，子进程发送 DSR `ESC [ 5 n` 后收到 `ESC [ 0 n`；同一探针在 shpool `v0.11.2` 超时。

产品形态不是把 tmux 暴露成第二套 API。agentd 默认给每个 runtime 建 owner-only socket/server，以 control mode 持有唯一可写客户端，记住 session/window/pane ID、进程退出状态和 runtime generation；一个 runtime 只建一个 session/window/pane，并启用 `remain-on-exit`。Workbench、CLI 和浏览器观察者由 agentd 扇出、限速与重放，不能按名称猜 pane，也不能直连取得输入权；裸 `tmux attach-session` 即便只读仍绕过 descriptor，因而只作明确标记的 break-glass。共享一个 server 承载多个 runtime 只有在故障域和背压探针通过后才可作为优化。

### 候选源码与 footprint 对照

测量均在 Apple Silicon macOS 上完成；进程数值不含各 session 的 harness/`sleep` 子进程。文件大小是实际字节换算的 MiB，不把“压缩包小”冒充运行时占用。

| 候选 | 源码审阅结论 | 本机 footprint | 决定 |
| --- | --- | --- | --- |
| **tmux 3.7c** | 公开 control mode、多客户端、稳定 pane ID、`capture-pane`/`pipe-pane`、退出状态和 headless 查询应答；慢 control client 有 pause/有界缓冲接缝 | Homebrew macOS arm64 bottle **0.52 MiB**，executable **0.95 MiB**，直接非系统 dylib **1.45 MiB**；一个 server 承载 10 个 detached session 时 **3.7 MiB RSS**。若默认每 runtime 一 server，十个约 **37 MiB RSS** | **采用**；HCTL 分发只带审阅过的最小 dylib/terminfo/许可集合，不把约 61 MiB 的完整 Homebrew dependency 目录原样打包 |
| **Zellij v0.45.0** | 原生跨平台与结构化插件能力较强，但[默认 layout](https://github.com/zellij-org/zellij/blob/v0.45.0/zellij-utils/assets/layouts/default.kdl)启动 tab/status WASM，二进制还[嵌入插件资产](https://github.com/zellij-org/zellij/blob/v0.45.0/zellij-server/src/plugins/plugin_loader.rs#L480-L490) | 官方 macOS arm64 `zellij-no-web` archive **11.3 MiB**、binary **32.4 MiB**；一个默认 detached session **89.7 MiB RSS**，10 个合计 **841.6 MiB RSS**（physical footprint 约 583 MiB）；无插件单 session 仍约 41.2 MiB physical footprint | **不采用**；多 Harness 常态下每 session 的 server/plugin 成本过高，其 web/插件面也不是第一阶段所需 |
| **shpool v0.11.2** | 轻量 daemon + attach 路径清楚，但当前合同[一次只允许一个客户端](https://github.com/shell-pool/shpool/blob/v0.11.2/README.md#L365-L370)，[事件只有类型而无 payload](https://github.com/shell-pool/shpool/blob/v0.11.2/libshpool/src/daemon/events.rs#L11-L18)；多客户端 [`#40`](https://github.com/shell-pool/shpool/issues/40)、快照/旁观 [`#363`](https://github.com/shell-pool/shpool/issues/363)及慢客户端阻塞修复 [`#399`](https://github.com/shell-pool/shpool/pull/399)仍未形成已发布合同 | 官方 macOS binary **4.04 MiB**，仅链接系统库；一个 daemon 承载 10 个空闲 detached session 时 **23.1 MiB RSS**。10 × 约 200 KiB 输出后，默认 vt100 restore 的 physical footprint 约 **252.3 MiB**，实验 vterm 约 63.7 MiB，simple 约 5.3 MiB但放弃可靠 replay | **不采用**；agentd 若补齐终端模拟/查询应答、多观察者扇出、背压、快照和 replay，已重新承担运行时最难的一半 |

### P0 阻断条件

tmux 支持 `extended-keys` 的 CSI-u/modifyOtherKeys 形态，但[没有完整 Kitty keyboard protocol](https://github.com/tmux/tmux/issues/5406)；必须通过能力探测降级，不能在 manifest 中虚报。`3.7c` 已包含 OpenCode palette 修复（[`#4793`](https://github.com/tmux/tmux/issues/4793)），`3.7b` 已修复 Codex/Grok 低对比度（[`#5312`](https://github.com/tmux/tmux/issues/5312)）和 Claude synchronized-output 回归（[`#5340`](https://github.com/tmux/tmux/issues/5340)），但这些历史修复不等于当前兼容性证明；OpenCode 仍有启动期查询/粘贴（[`#42915`](https://github.com/anomalyco/opencode/issues/42915)）和 passthrough 应答泄漏（[`#40035`](https://github.com/anomalyco/opencode/issues/40035)），Codex 有增强键位问题（[`#34717`](https://github.com/openai/codex/issues/34717)），Kimi `/copy` 有 OSC 52 问题（[`#3173`](https://github.com/MoonshotAI/kimi-code/issues/3173)）。

因此 B2 前必须在实际分发物上跑 Antigravity `1.1.18`、Claude Code `2.1.240`、Codex `0.149.0`、OpenCode `1.18.21`（GLM 环境）、Grok Build `1.0.5`、Kimi Code `0.38.0` 的启动/退出、attach/重连、颜色、CJK/IME、普通与 bracketed paste、OSC 52、组合键、全屏 resize、DA/DSR/DECRQM 和慢观察者矩阵。另以每 runtime 独立 server 和共享 server 两种拓扑复现 [`tmux #5510`](https://github.com/tmux/tmux/issues/5510) 的多窗格、快速滚动、copy-mode、resize 组合；任一模式卡死、跨 runtime 污染或无法 fencing 都阻断发布。最终 pin 可以前进到含修复的新版本，但必须重新固定审阅 commit 并跑同一矩阵。

<a id="l1-selected-evidence"></a>
## L1 精选证据与能力观察清单

### 精选实现证据

| 证据 | 独特价值 | 边界 |
| --- | --- | --- |
| [Termio `d1fdac8…`](https://github.com/termio-sh/termio/tree/d1fdac84046805d4056e082f982e6beb6072b61c) / [ATP](https://www.termio.sh/docs/atp) / [会话控制](https://www.termio.sh/docs/session-control) | Manifest、稳定的 Session URI、监听/心跳/信号，以及带数据结构版本的控制协议 | MIT；ATP 不是 HCTL 或行业通用的传输标准，也不作为跨平台 Backend 的权威实现 |
| [Herdr `v0.8.0 / 346411fa`](https://github.com/herdrdev/herdr/tree/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7) / [专项审计](#e-l1-herdr) | 后台服务持有 PTY；观察/控制与原始/语义操作面分离；单写者接管；状态信号仲裁与分级恢复 | Apache-2.0；控制方不是持久租约，运行状态不等于领域完成；完整边界见专项审计 |
| [xterm.js](https://github.com/xtermjs/xterm.js/) | 嵌入式终端渲染器，以及 CJK、输入法、无障碍和流量控制 | MIT；只负责前端，不拥有 PTY、进程或 Session |
| [WezTerm](https://wezterm.org/cli/cli/index.html) | 成熟的跨平台外部终端与 CLI | MIT；不嵌入应用，也不把 Mux 协议当作 ABI |
| [tmux `3.7c / e476c123`](https://github.com/tmux/tmux/tree/e476c1230b958df0cb12977517d24b3dc931375b) / [专项复审](#e-l1-tmux-runtime) | 公开 control mode、稳定 pane ID、headless 查询应答、捕获/转发、退出状态和很小的 native footprint | ISC；采用为运行时依赖，不作为公开 HCTL API；完整 Kitty 键盘协议、六 Harness 兼容性、背压与 `#5510` 仍是 P0 阻断项 |

### 只列入观察清单的产品

| 项目 | 只保留的独特证据 | 复用边界 |
| --- | --- | --- |
| [MindFS](https://github.com/a9gent/mindfs) | 仓库本地 Session、外部 Session 导入与同步 | AGPL；只参考协议与行为，Task Board 不定义 L3 |
| [Paseo](https://github.com/getpaseo/paseo) | 守护进程/客户端/执行提供方适配器、公开 SDK、多设备接缝 | AGPL；作为第二阶段架构参考 |
| [HAPI](https://github.com/tiann/hapi) | 原生本地 Agent 与远程端之间的结构化交接 | AGPL；不提供精确 PTY，也不是 Task/Workflow 后端 |
| [Happy](https://github.com/slopus/happy) | 守护进程、端到端加密同步、远程启动、多设备 | MIT；列入第二阶段观察，不作为第一阶段事实源 |
| [Moshi](https://getmoshi.app/docs/introduction) | 移动终端、钩子与注意力提醒、TUI Chat 投影 | 闭源；只参考用户体验和互操作行为 |
| [Remux](https://github.com/h3nock/remux) | 通过 SSH 和 tmux 控制模式精确定位会话/窗口/窗格 | MIT；不引入第二套领域状态 |
| [ServerCC](https://servercc.app/docs/sessions) | 外部接管、厂商会话恢复、移动端控制 | 闭源；作为身份与交接的产品行为证据 |
| [QuickTUI](https://quicktui.ai/) | 自托管 tmux 加移动端或浏览器终端 | 应用闭源；公开仓库只能证明分发方式 |
| [Redock](https://redock.dev/) | 分阶段输入、CJK 与语音、Activity 深链 | 闭源；只参考用户体验 |

## 执行面已选依赖的运维与 footprint

这里的“已选依赖”指需要独立托管生命周期的 Dagu、Tuwunel、Vikunja、tmux 四项；React/Tiptap/xterm.js 等随 Workbench 打包的库没有独立运维面，其体积在整窗发布探针中计算。这是 2026-08-23 的第一阶段基线，不是容量承诺。文件大小取官方 release asset 或明确标注的 Homebrew bottle 实际字节；RSS 在 Apple Silicon macOS 上用空数据、默认或文中注明的最小配置启动，稳定约 10 秒后读取，且不含 control、Workbench 和 harness 子进程。Tuwunel 官方只有 Linux 发布物，本机没有既有 Linux 容器/VM，故不伪造 RSS 数字。

| 模块 | 固定版本与许可 | 发布 / 分发 footprint | 空载实测 / 数据 | 运维判断 |
| --- | --- | --- | --- | --- |
| **Dagu** | [`v2.15.1 / 532c5129`](https://github.com/dagucloud/dagu/releases/tag/v2.15.1)，GPL-3.0-or-later | macOS arm64 archive **45.9 MiB**、binary **148.1 MiB**；Linux amd64 为 48.3/154.6 MiB | `start-all`、coordinator 关闭：**92.4 MiB RSS**；空数据目录约 84 KiB | **低—中**：一个进程、文件备份；主要风险是 adapter/fencing，不是日常运维 |
| **Tuwunel** | [`v1.9.0 / 5b366914`](https://github.com/matrix-construct/tuwunel/releases/tag/v1.9.0)，Apache-2.0 | Linux x86_64 GNU zstd **31.2 MiB**、binary **98.1 MiB**；无官方 Darwin asset | macOS RSS **待 P0**。默认 cache capacity 源码为 [`128 + 64 × parallelism` MiB，write buffer 为 `48 + 4 × parallelism` MiB](https://github.com/matrix-construct/tuwunel/blob/5b3669144219d5d4c0774743c84191b476f1b54f/src/core/config/mod.rs#L5185-L5189)，capacity 不等于已提交 RSS | **中—高**：macOS 额外需要 Linux VM/container；需固定低内存配置，并一致备份 RocksDB、media 与 secret |
| **Vikunja** | [`v2.5.0 / ef2200e9`](https://github.com/go-vikunja/vikunja/releases/tag/v2.5.0)，AGPL-3.0-or-later | macOS arm64 full zip **46.9 MiB**、binary **107.3 MiB** | SQLite 空服务 **56.7 MiB RSS**；初始 DB/WAL 约 2.3 MiB | **低**：一个进程 + SQLite；备份 DB、attachments 和 secret，升级前做 migration/restore 演练 |
| **tmux** | [`3.7c / e476c123`](https://github.com/tmux/tmux/releases/tag/3.7c)，ISC | Homebrew macOS arm64 bottle **0.52 MiB**、executable **0.95 MiB**；直接非系统 dylib **1.45 MiB** | 一个 server + 10 个 detached session **3.7 MiB RSS**；默认每 runtime 独立 server 时十个约 **37 MiB RSS** | **低安装 / 中集成**：无数据库；要固定最小动态库/terminfo、owner-only socket、control mode、pane ID、背压与残留 session 清理，六 Harness 矩阵是阻断项 |

选择对应平台发布物时，三个 archive 加 tmux bottle 合计约 **125 MiB**，四个主 executable 合计约 **354.5 MiB**；tmux 的直接非系统 dylib 另加约 **1.45 MiB**。本机能直接测量的 Dagu + Vikunja + 一个 tmux server 合计约 **152.8 MiB RSS**；若按默认故障隔离启动十个 tmux server，则约 **186 MiB RSS**。这还没有 Tuwunel 的进程与 macOS Linux VM、任何 harness、control 或 Workbench；**Tuwunel 仍是明确最高的运维与资源不确定项**，必须在 B1 前量出完整 macOS 组合，而 tmux 的六 Harness/fencing/背压与 Dagu 的 fencing 分别在 B2、B4 前验收。

## 标准与通用库，不作为产品主参考

- [Agent Client Protocol](https://agentclientprotocol.com/protocol/v1/overview) / [Rust SDK](https://github.com/agentclientprotocol/rust-sdk)：L1 的 Harness 接入标准。
- [Agent Skills](https://agentskills.io/specification)：用于 L4 的 Expertise 选择，以及 L1 的交付与绑定；Skill 只提供指导，不是 Gate。
- [MCP Resources](https://modelcontextprotocol.io/specification/2026-07-28/server/resources) / [Prompts](https://modelcontextprotocol.io/specification/2026-07-28/server/prompts)：传输 Context 和工具信息，不定义 Project/Task 的决定权。
- [React Flow](https://reactflow.dev/) / [Dagre](https://github.com/dagrejs/dagre)：用于 L2 的只读可视化与布局。
- [Electron 安全指南](https://www.electronjs.org/docs/latest/tutorial/security) / [MessagePorts](https://www.electronjs.org/docs/latest/tutorial/message-ports)：用于跨层可信 UI 与数据传输。

## 复用决策用语

所有证据最终只归入五种复用决策：**采用为依赖（Adopt dependency）**、**移植有边界的组件（Port bounded component）**、**适配协议（Adapt protocol）**、**仅参考行为（Behavior reference）**、**暂缓（Defer）**。不得给整个产品一个“取代 HCTL”的总分，也不得把参考项目中的 Session、Conversation、Project、Task、Run 名称或内部数据库带入 HCTL 的公开数据结构。

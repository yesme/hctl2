# 实现证据与精选参考组合

> 状态：信息性文档 · 研究快照 2026-08-19<br>
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
| [OpenClaw](https://github.com/openclaw/openclaw) | L4 | 专项参考 | 确定性的多渠道身份与路由、配对/白名单和按渠道降级投递 |
| [Codeg](https://github.com/xintaofei/codeg) | L3 核心；L1/L2 专项 | 核心参考 + 专项证据 | 独立异步 WorkTask、评审/合并/恢复、ACP/worktree/差异集成，以及自动化与固定流程的边界 |
| [Hermes Agent](https://github.com/NousResearch/hermes-agent) | L3 | 专项参考 | 持久 Task/Attempt、原子领取、心跳/回收、依赖推进和多客户端共用内核 |
| [Multica](https://github.com/multica-ai/multica) | L4/L3/L2/L1 专项 | 行为、边界与实现证据 | L4 的 Project/Issue/私聊发布边界；L3 的 Issue 与单次运行分离；L2 的领取、租约、重试、恢复与归属；L1 的多 Harness 能力矩阵和无损 worktree |
| [HCTL1 / yesme/hctl](https://github.com/yesme/hctl) | L2 | 直接谱系证据 | Git 原生 Seat 领取与隔离栅栏、精确 Verdict 与法定票数、可重放 Receipt 和失败时默认拒绝的测试集 |
| [HCTL2 Run 语义内核](../run.md) | L2 | 原生语义核心 | 与版本和证据绑定的 Run、Seat 候选切换、法定票数、重新过 Gate 和 Receipt |
| [Conductor OSS](https://github.com/conductor-oss/conductor) | L2 | 机械状态后端 | 外部执行者与 token、定时器、重试和历史的被动机械状态 |
| [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) | L2 | 相邻实现参考 | SOP 准入、按版本审批与法定票数、恢复和失败时默认拒绝的规则测试 |
| [Dagu](https://github.com/dagu-org/dagu) | L2 | 观察清单 | 数据优先的图、运行器/动作/人工审批，用作后端选型对照 |
| [Stably Orca](https://github.com/stablyai/orca) | L1 核心；L2 专项 | 核心参考 + 专项参考 | L1 的 PTY 所有权、冷热恢复、代际隔离和 worktree/差异/远程/交付；L2 的持久 Run 收件箱、Dispatch 权威、可靠交付、幂等收据和执行者资源生命周期 |
| [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) | L1；横切架构 | 专项参考 + 架构边界 | 能力端口、类型化事件和可撤销注册、模型可见只追加日志，以及插件组合的收益与风险 |
| [OpenCode](https://github.com/anomalyco/opencode) | L1 | 专项参考 | OpenAPI + SSE + 类型化 SDK 的服务端优先、多客户端 Harness 操作面 |
| [Pi](https://github.com/earendil-works/pi) | L1 | 专项参考 | 内嵌 SDK + 严格 JSONL RPC，以及 `steer`/`follow_up` 队列契约 |
| [Kimi Code](https://github.com/MoonshotAI/kimi-code) | L1 | 专项参考 | ACP/原生能力矩阵和可以验证的降级行为 |
| [Termio](https://github.com/termio-sh/termio) | L1 | 专项参考 | Harness Manifest、会话 URI，以及监听/心跳/信号契约 |
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
| L4 · Project Room | First Tree 的持久 Chat、显式寻址、可见 handoff、Need You、Context 提升与跨渠道连续性；Multica 的共享 Issue 与私密探索发布边界；Claude Tag 的持久讨论串与临时沙箱分离；OpenClaw 的外部身份和路由 | HCTL2 用规范 Room、一级 Request、ContextManifest 和 Memo 提升流程统一这些经验；外部渠道只作同一 Room 的输入输出面，私聊和执行记录不会自动成为项目知识，协作边的创建权也不随消息作者身份下放给 Agent |
| L3 · Task / Kanban | Codeg 的独立 WorkTask、Needs You、评审、后续动作和 Git 恢复；Multica 对 Issue 与单次运行、运行结束与承诺完成的明确分离；Hermes 的领取与重新领取；Linear/GitHub 的原生字段状态 | HCTL2 将长期承诺冻结为 TaskRevision，把高频操作状态、外部字段权威和 TaskCompletionReceipt 分开；启动 Run 与移动卡片分离，完成必须重新校验验收标准和证据 |
| L2 · Workflow / Run / Gate | HCTL1 的版本/证据、领取/隔离栅栏、法定票数和 Receipt；Conductor 的机械图状态；Stably Orca 的持久监督协议；Multica 的租约/重试/恢复/归属；ZeroClaw 的审批准入；Herdr、Superset 的边界反例 | HCTL2 自己定义 WorkflowRevision、Run Manifest、Obligation、Seat、Attempt、Verdict 和 Receipt；外部机制只补机械推进、可靠领取、消息交付和故障测试，不能用执行者状态或会话传输替代语义治理 |
| L1 · 执行 / 运行时 | Stably Orca 的 PTY 所有权、冷热恢复、远程和交付；Superset 的 `epoch:seq` 重连、守护进程接管和分阶段清理；Herdr 的观察/控制分离；Multica 的多 Harness 能力和不丢代码；DeepSeek Harness 的组合式能力端口；OpenCode/Pi/Kimi/Termio 的接入协议 | HCTL2 以 agentd、HarnessAdapter、RuntimeBackend、ChangeSet 和 TerminalGateway 统一接入；所有能力逐绑定探测并准确降级，运行时身份、终端状态和厂商会话都不能反向定义 Project、Task 或 Run |

这张表是“整合关系”，不是对象映射。每个来源项目只贡献表中写明的机制；L4–L1 是本研究保留的历史标签，最终身份、权限、版本和证据由 HCTL2 的 Project、Task、Run、Harness 四模块定义。

<a id="e-l4-first-tree"></a>
## E-L4-FIRST-TREE · First Tree

### 核心价值与跨层画像

First Tree 真正跑通的是协作闭环，而不是任务或工作流闭环：Team 中的持久 Chat 或 SCM 事件形成共同上下文，受管 Agent 工作后把结果交还给用户或 SCM；只有同时通过 Decision Test 和 Durability Test 的稳定结论，才会经由另一条有来源支撑、需要审查的流程写入 Context Tree。写入 Context Tree 并不是每次 Chat 的自动收尾；没有具体来源材料时，规则明确要求什么都不写。

它最突出的价值仍在 L4：证明以 Chat 和 Context 为主轴可以维持长期协作。但源码还给出了两组值得跨层引用的深入机制：L2 可以参考精确快照、有来源支撑的写入，以及 Reviewer 对精确 head/digest 的批准与失效规则；L1 可以参考受管执行提供方的会话代次、ACK、重试、恢复和能力契约。它并没有 HCTL 意义上的 Project、Task、TaskRevision、Workflow 或 Run；`task chat` 只是创建 Chat 的一种模式，GitHub Task Agent 的事实仍是 Issue/PR 加权威 Chat，cron 也只是把触发器转成消息。因此，这些跨层亮点只是专项机制，不能把 First Tree 整体当成通用 L2 编排器、L3 Task 系统或 L1 终端管理器。

协作拓扑需要拆成两半评价。First Tree 的 `chat send` 要求显式 recipient，正文里的 `@name` 本身不触发路由；handoff、邀请与后续消息都留在持久 Chat 中，对人可见且可追溯。这证明“显式寻址 + 持久 Chat + 可见 handoff”可以避免隐藏的 peer RPC。与此同时，Agent 可以在运行中 `invite + send`，接收者还能继续寻址第三个 Agent，使参与者集合与协作图由模型临场扩张。HCTL 采用前一半，不采用后一半作为默认拓扑：普通 Room 中 Agent 只能建议下一条协作边，由 human actor 提交；自动化边则由 reducer 按冻结的 WorkflowRevision 创建。这里记录的是参考取舍，实际权限与命令合同仍以规范文档为准。

### 审计基线

发布版与当前主干必须分开陈述：

| 基线 | 状态 | 可支持的结论 |
| --- | --- | --- |
| [`v0.5.20 / 19e66032`](https://github.com/agent-team-foundation/first-tree/commit/19e66032af7f9f482168c350fe0b3998599388f3) · 2026-08-11 | 已发布 | Context Tree、持久 Chat、基于稳定身份的 mention、Request/Need You、Inbox、GitHub/GitLab，以及执行提供方的运行与恢复 |
| [`main@f0d46f9e`](https://github.com/agent-team-foundation/first-tree/commit/f0d46f9ec8b14ace536d242db8860065c124f2c7) · 2026-08-14 | 未发布审计快照；比发布版前进 41 个 commit | Feishu Agent Channel、OpenTag 入门流程、更新后的 GitHub Issue 激活规则，以及运行权限、ReplayFence 和 Reset 机制 |

[发布版与审计快照的差异](https://github.com/agent-team-foundation/first-tree/compare/19e66032af7f9f482168c350fe0b3998599388f3...f0d46f9ec8b14ace536d242db8860065c124f2c7)。Feishu QA 文件只是可执行的验收契约，不是公开的通过报告；这些主干能力不能写成 v0.5.20 已发布功能。

[官网](https://first-tree.ai/)仍以 CODEOWNERS 描述 Context Tree 的归属关系，但 v0.5.20 的实际规则使用 frontmatter 中的 `owners` 字段，[Seed Skill](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/skills/first-tree-seed/SKILL.md)还明确禁止创建根 `CODEOWNERS`。网站描述已经偏离当前实现，不应继续作为设计依据；[官方文档站](https://docs.first-tree.ai/)目前也只是占位内容。以下判断以固定源码、仓库文档和可执行测试为准。

### 源码审计结论

| 范围 | 已验证 | 缺口 | HCTL 如何吸收 |
| --- | --- | --- | --- |
| 产品对象 | Team、Agent、人类成员、持久 Chat、类型化 Message、Context Tree、Agent 会话，以及 SCM entity↔Chat 映射 | 没有 HCTL 的 Repo/Project/Task/WorkflowRevision/Run/Seat；Team 还可能横跨多个代码仓库 | 证明 L4 协作可以持续；不照搬 Team/Agent/Chat 数据结构，也不把 Chat 直接叫作 Project Room |
| Context Tree | Decision Test + Durability Test；Tree 与代码不一致时，默认以代码事实为准；按精确 commit 读取快照；写入必须有来源材料、独立 worktree、校验和 PR/MR 评审 | 治理模型绑定 First Tree 的 Team、Reviewer 和代码托管平台；知识晋升是独立流程，不会在每次任务后自动执行 | 把筛选标准和有来源支撑的评审流程改编成 Memo→Project 知识准入；不新增 `ContextTree` 一级对象 |
| 类型化 mention / Inbox | Web 发送稳定的 Participant ID，服务端校验成员关系和启用状态；Message 与接收者分发在同一事务中；支持 `pending/delivered/acked`、`SKIP LOCKED`、逐 Chat 前缀 ACK 和断线恢复 | CLI/API 仍兼容名称寻址；普通发送没有调用方幂等键；消息可原地编辑，只有 `editedAt`，没有 revision/history/tombstone | 采用稳定身份、ACK 责任链和事务测试；HCTL 另补命令 ID、只追加的 correction/tombstone 和冻结的 MentionRef |
| 协作边 / handoff | `chat send` 使用显式 recipient；只有被具名寻址的 Agent 被唤醒，其他 participant 只获得 silent context；邀请、交接和结果留在持久 Chat 中 | Agent 可以自行 `invite + send`，接收者还能继续寻址第三个 Agent；系统没有冻结的通用 Workflow 图约束这条动态链 | 采用显式寻址、持久 Chat 和对人可见的 handoff；不采用 Agent 消息直接创建执行边或开放 mesh，普通 Room 的临场边由人提交，自动化边由 reducer 按冻结图创建 |
| Request / Need You | 当前权威事实是 `format="request"` 消息与后续 resolution 行；只允许一个用户作为目标；目标用户的界面会局部阻塞，其他成员仍可阅读；多个请求先进先出；普通回复或 `inReplyTo` 不会关闭请求，只有目标用户显式写入 `metadata.resolves` 才产生新 resolution；跨 Chat 队列从持久记录推导 | 旧 `attentions`、`pending_questions` 只是历史审计表；Request 仍是一种消息格式，不含 revision、权限或法定人数语义 | 借鉴归约器、显式关闭、先进先出和仅阻塞目标用户的交互；HCTL 将其提升为一级 Request，但绝不让它替代 Gate、Seat 或法定人数规则 |
| GitHub 集成 | HMAC、delivery ID 去重、entity↔权威 Chat、由服务端记录的 run 来源、受管 Task Agent 和幂等 App 回复；当前主干中，普通 Issue 要等非自身输出的新评论或已有精确 owner mapping 才激活，PR 不受此限制 | 工作事实仍是 Issue/PR + Chat，没有独立 Task、TaskRevision、Board 或验收生命周期；部分故障、排序和身份主体语义仍不完整 | 借鉴绑定、来源证明、去重和跨界面测试；可在 L3 作为外部工作触发与权威映射的边界证据，但不把 GitHub entity/Chat 当作 HCTL Task 事实 |
| 仅主干存在的 Feishu | bot/chat 绑定、精确 mention、回声抑制、作者快照、事件与消息双重去重、附件取回，以及租约与代次 | 当前是 1 Chat↔1 Feishu；Web 只读；不支持编辑/删除；ACK 可能早于权威事务提交；最终事务没有隔离令牌；出站没有 DeliveryReceipt 生命周期 | 借鉴格式转换、去重、租约和验收测试；HCTL 另补多界面绑定、RoomEvent 与出站队列的原子性、提交时隔离、Receipt 和对账 |
| 执行提供方运行时 / Skills | `start/resume/inject/suspend/shutdown`、ACK、重试、恢复与持久化、目录与能力声明、Skill 的锁、日志、摘要和版本隔离、守护进程监管；主干又加入 ReplayFence 和 reset 权限 | 私有客户端与 Hub/Chat 强耦合；API 仍在快速变化；重试只覆盖同一个执行提供方和会话；协议层回执不是语义 Receipt | 只借鉴契约、故障、重放和 Skill 测试，不直接建立包依赖，也不把它当作 Seat 的降级方案 |
| 会话 / 终端 | 执行提供方会话主要通过 SDK、app-server 或子进程运行；通用运行时管理代次、ACK、重试、恢复和会话持久化；内部 `tmux` 驱动支持粘贴与捕获输出 | 新配置已禁用 TUI 选项；没有公开的重新接入接口或稳定 PTY 目标 | 可作为 L1 受管会话与运行恢复的专项实现证据；终端所有权和重新接入仍需参考其他项目 |
| Workflow / 治理 | Context 读取、写入和 Reviewer 流程采用精确快照、来源门槛、精确 head 批准及失效规则；cron 到点生成定向消息，并限制同一个 job 不积压多个未 ACK 触发器 | 没有通用运行历史、DAG、WorkflowRevision、Seat、候选执行者、法定人数、重新过 Gate 或绑定版本的 Receipt | L2 可参考 Context 变更准入、快照与批准失效，以及带版本的触发器；不把这些局部机制扩写成通用 Workflow 模型 |

复用结论：**选择性移植**，许可证为 Apache-2.0。可以直接改编 Context Policy 的两项筛选测试、Need You 行为旅程、Inbox ACK 责任链，以及 Feishu/GitHub 的跨界面验收测试；也可按需移植纯数据结构、内容转换、绑定与租约更新、前缀 ACK、ReplayFence/reset 代次，以及受管 Skill 的事务纪律。不整仓派生，也不采用其中心化 PostgreSQL 或云端事实源。

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

<a id="e-l4-openclaw"></a>
## E-L4-OPENCLAW · OpenClaw

OpenClaw 最值得参考的是 L4 的外部频道接入边界：它把账号、对端和讨论串归一为确定性路由键，并支持精确绑定、讨论串继承、私信作用域、配对与允许名单、房间环境事件、防止机器人循环，以及按频道能力降级投递。这说明：没有 Workbench 时，Chat 界面仍需要稳定的外部身份、确定性路由和逐频道降级，不能让模型猜测频道，也不能按显示名称分发。

HCTL 只借鉴适配、路由、配对、防循环和降级测试；OpenClaw 的 channel/session/workspace/agent 不映射为 Project/Room/Task/Run，环境聊天不会自动成为权威 Context，Gateway、cron 或 delegation 也不成为 L2 的权威事实。固定版本为 [`v2026.7.1-2 / 0790d9f5`](https://github.com/openclaw/openclaw/tree/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c)（MIT）；证据见[频道路由](https://github.com/openclaw/openclaw/blob/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c/docs/channels/channel-routing.md)、[README](https://github.com/openclaw/openclaw/blob/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c/README.md)与[许可证](https://github.com/openclaw/openclaw/blob/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c/LICENSE)。

<a id="e-l3-codeg"></a>
## E-L3-CODEG · Codeg

### 核心价值与跨层画像

Codeg 的产品闭环是：先把一项工作写成独立 WorkTask，再排队并按并发上限领取；启动时根据精确的 base SHA 创建隔离 worktree；Agent 执行过程中可以进入 Needs You；完成一轮后由用户查看结果、diff、时间线和预检，再选择 Rework、Keep going、Ask、Double-check 或 merge；最后用 Git 事实确认结果是否真正落地。官方 Tasks 指南对边界的概括也很准确：Conversation 是用户坐在前面共同推进的会话，Task 则是写下以后可以暂时离开的异步承诺。

WorkTask 的 `worktree_folder_id`、`conversation_id` 和 `connection_id` 都可以为空，因此 Task 身份不依赖某次运行环境。状态变化使用“预期状态 + `run_seq`”进行 CAS，并把只追加的时间线事件放在同一事务中。Board、排队、Needs You、评审、后续动作、预检、Git 事实和重启恢复，共同构成其完整的 L3 产品机制。

Codeg 的核心价值在 L3，其他层也有可单独采用的机制。L1 可以专项参考它把 ACP、worktree、Git、diff、Composer、事件卡片，以及由 Agent 所在环境提供的文件系统和终端沙箱接成一条执行体验；L2 可以把 Automations 的定时触发、单个 Automation 串行、补跑，以及固定 Task 流程与 merge 恢复，当作“产品层自动化”和“通用 Workflow 治理”之间的边界证据。桌面终端由进程内 `HashMap` 持有，没有持久滚屏记录、进程重启恢复或稳定的远程重新接入，因此 PTY 所有权与重连主要参考 Stably Orca。Automations 没有带版本的图、权限与法定人数规则或通用 Gate，也不能单独定义 HCTL 的 L2。

### 审计基线

| 基线 | 状态 | 可支持的结论 |
| --- | --- | --- |
| [`v0.24.0 / df7a872d`](https://github.com/xintaofei/codeg/commit/df7a872de44546277e4c49cfe9d173c631161dc6) · 2026-08-11 | 已发布 | 独立 WorkTask、四列 Board、排队/并发/定时启动、Needs You、评审/预检/后续动作、merge 与基于 Git 事实的恢复 |
| [`main@a34a047a`](https://github.com/xintaofei/codeg/commit/a34a047a568018ee180dee75add8c9c7d30b2ea6) · 2026-08-14 | 未发布审计快照；按 first-parent 口径比发布版前进 23 个 commit | merge 排队、[由 Agent 所在环境提供文件系统与终端沙箱](https://github.com/xintaofei/codeg/commit/b7e21e4c789ba70036ec87de5ed72dec3d25a678)，以及重连与权限修正 |

[发布版与审计快照的差异](https://github.com/xintaofei/codeg/compare/df7a872de44546277e4c49cfe9d173c631161dc6...a34a047a568018ee180dee75add8c9c7d30b2ea6)。v0.24.0 和当前 Tasks 指南规定“同一项目已有 merge 时拒绝第二个 merge”，主干从 [`597a7eeb`](https://github.com/xintaofei/codeg/commit/597a7eeb24e4a5f8aca149f2f5c182d3c2c90510)起改为排队。发布能力与主干能力在本文中分别标注。

### 采用与边界

HCTL 采用独立 Task 身份、显式状态机、基于 `run_seq`/CAS 的过期事件隔离、事务内时间线、精确 base SHA、复用 worktree 的重试、Needs You 投影、评审/预检/后续动作，以及“根据 Git 事实恢复 merge”的测试。`done` 只应来自已经落地的 merge，或用户明确接受“没有内容可合并”；Agent 自报 `task_complete` 只能作为建议，不能决定 Task 是否完成。

Codeg 的 `WorkTaskConfig` 不能直接当作 HCTL 的冻结 TaskRevision。它保存 `prompt_blocks` 和每个 Task 的覆盖值，但空字段会在真正启动时继承当时的 Folder 设置，实际采用的值只写入 `config_effective` 审计事件。HCTL 在批准 Run 时必须冻结完整的 TaskRevision/WorkflowRevision，不能让可变默认值在启动时继续改变契约。

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
| L3 | Issue 是可以长期讨论、修改、重新分配并最终关闭的工作承诺；Task 是一次生命周期有限的运行。同一 Issue 可以产生多个 Task，已有运行记录不会被覆盖；精确重试某个历史 Task 时仍调用该次运行当时的 Agent。官方文档明确规定：Task 的 `completed` 只表示该次运行正常结束，不表示 Issue 目标已经完成。 | 采用 Issue 与单次运行 Task 分离、运行历史不可覆盖、定向重试，以及“运行完成不等于工作完成”。不采用可变 Issue 描述作为冻结的 TaskRevision，不采用分配或状态变化自动获得施工授权，也不把 Agent 将状态改成 `in_review`、产生分支或 Task 正常退出当作验收。 |
| L2 | Task 具有 `queued → dispatched → waiting_local_directory/running → terminal` 生命周期。数据库通过 `FOR UPDATE SKIP LOCKED` 原子认领，并把同一 `(Issue, Agent)` 的运行串行化；准备租约保护启动窗口，`dispatched_at` 充当认领代际的 CAS 隔离栅栏，认领响应丢失后可以重新领取，守护进程重启后可以回收。长期运行依赖运行时心跳，而不是固定的总时长；失败分类决定能否重试，后继 Task 保存 `attempt`、`max_attempts`、`failure_reason`、`session_id`、`work_dir` 和 `retry_of_task_id`，触发者、委派链和证据引用也随运行记录归属。Autopilot 还为定时和 webhook 的每个触发实例提供幂等与崩溃恢复测试。 | 采用领取（claim）、租约（lease）、隔离栅栏（fence）和重新领取（reclaim），并采用失败分类、重试谱系、来源归属和轮询兜底的实现与测试形状，尤其适合无 Workbench 时由服务端和守护进程协作执行。它没有 WorkflowRevision、通用 DAG、Gate、Seat、法定票数或语义 Receipt；Squad leader 由 LLM 决策，不能成为控制事实；Autopilot 是可重复触发的操作手册，不是 HCTL Workflow。 |
| L1 | 一个统一的 `Backend` 契约接入 22 个 Harness 产品名称；其中 21 个协议族由后端构造器、数据库约束和锁步测试共同限定，Oh-My-Pi 复用 Pi 协议族。不同 Harness 的模型、MCP、Skill 路径和会话恢复能力被明确列成能力矩阵，并对“无法判断恢复请求是否被拒绝”等降级情况单独编码。本地 Git 路径会先保全脏工作树，再为每个 Task 建立 worktree；无论成功、失败还是取消，都会尽量提交已经产生的改动，提交失败时则保留 worktree，避免清理过程吞掉用户工作。 | 采用统一 Harness 契约、逐绑定能力探测、显式降级测试，以及“先保全、后隔离、任何退出路径都不丢改动”的 worktree 纪律。Multica 不拥有可重新接入的 PTY，也不能用会话、分支或工具调用成功证明语义完成；其源码许可也排除了直接移植。 |

### 采用结论

HCTL 应组合采用四块经过源码验证的机制：L4 的共享/私密发布边界；L3 的 Issue/单次运行分离；L2 的领取、租约、重试、恢复和来源归属机制及其测试用例；L1 的 Harness 能力契约与无损 worktree 收尾。它们分别进入对应层，不需要把 Multica 设成某一层的唯一参考。

明确不采用：用 Issue 当前内容充当 TaskRevision，用分配或状态变化充当启动授权，用 Squad leader 的 LLM 判断充当调度权威，用 Autopilot 充当通用 Workflow，用 Task 的 `completed` 充当 Verdict/Receipt，以及移植受自定义许可证约束的源码。

主要证据：

- 官方产品行为：[Projects](https://multica.ai/docs/projects)、[Issues](https://multica.ai/docs/issues)、[Tasks](https://multica.ai/docs/tasks)、[Chat](https://multica.ai/docs/chat)、[守护进程与运行时](https://multica.ai/docs/daemon-runtimes)、[Harness 对比](https://multica.ai/docs/providers)与[Autopilots](https://multica.ai/docs/autopilots)
- 固定文档：[Projects](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/projects.mdx)、[Issues](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/issues.mdx)、[Tasks](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/tasks.mdx)、[Chat](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/chat.mdx)与[Harness 能力矩阵](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/providers.mdx)
- L2 实现：[Task 服务](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/internal/service/task.go)、[领取与重试 SQL](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/pkg/db/queries/agent.sql)、[租约与重试数据结构](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/migrations/055_task_lease_and_retry.up.sql)、[准备租约](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/migrations/124_task_prepare_lease.up.sql)、[领取竞争测试](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/internal/service/task_claim_race_test.go)、[完成竞争测试](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/internal/service/task_complete_race_test.go)与[Autopilot 恢复测试](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/cmd/server/autopilot_schedule_job_test.go)
- L1 实现：[统一 `Backend` 与能力例外](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/pkg/agent/agent.go)、[协议族锁步测试](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/pkg/agent/agent_supported_types_test.go)与[本地 worktree](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/internal/daemon/execenv/local_worktree.go)

<a id="e-l2-hctl1"></a>
## E-L2-HCTL1 · HCTL1 / yesme/hctl

HCTL1 是 HCTL2 L2 语义内核的直接前身，也是可执行的技术谱系证据；它不是外部复用来源，不能与 HCTL2 的原生语义核心混为一谈。审计固定在 [`main@3148042c`](https://github.com/yesme/hctl/tree/3148042cb2faf8df0dc8be92710b9468c8618516)（2026-07-28，Apache-2.0）。仓库没有标签或正式发布；README 表明 P1 内核已经进入主干，P2/P3 仍处于规划阶段。

它最独特的证据是一套不依赖守护进程和数据库的 Git 语义内核：每个 Seat 一条只追加事件引用、本地与远端 CAS、电平触发式对账、事实不完整时默认拒绝、Obligation/CLAIM 与 claim OID 隔离栅栏、精确匹配 `{base, head}` 的 Verdict、法定票数，以及携带事实摘要、无需依赖时钟即可重放的 squash merge Receipt。除规范外，仓库还提供可执行用例库，覆盖过期 Gate、权限、竞争、JCS 身份、组合法定票数、迟到 Finding、重新 Gate 时的结论沿用，以及初始化切换。

HCTL2 继承版本与证据、领取与隔离栅栏、法定票数、Receipt 和对账的思路，但不会原样继承其对象与事实源：

- HCTL1 的 `Seat = harness × model` 表示协作身份；HCTL2 的 Seat 是 Obligation 内的逻辑执行者或投票者位置，下挂 `0..N` 个 Attempt；
- HCTL1 的 Obligation 来自静态分派中的 author/gate/merge；HCTL2 的 Obligation 对应 Conductor 外部任务的一次执行责任；
- HCTL1 把每个 Seat 的 ref、PR 和 squash Receipt 作为全局协调事实；HCTL2 把运行治理放入 SQLite 控制库，以 Git 保存共享且低频变化的定义和证据，并由 Conductor 保存机械工作流位置；
- HCTL1 的回收机制不等于候选方案降级，而且没有 Project Room、Task Board、WorkflowRevision、Run、Attempt、进程/PTY 或外部系统同步；
- 单一人类信任、唯一合并协调者且容量为 1，以及把 PR 当作协作原子，只适用于它所定义的窄范围运行方式。

主要证据：

- [README 范围](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/README.md#L7-L27)；[METHOD 中的事实、Seat 与领取](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/METHOD.md#L27-L114)；[Gate、结论沿用与合并](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/METHOD.md#L108-L182)
- [派生引擎](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/internal/derive/derive.go#L47-L124)；[CAS 与待处理状态恢复](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/internal/store/store.go#L15-L191)；[Receipt 重放](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/internal/receipt/receipt.go#L14-L187)
- [可执行用例库](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/tests/corpus/README.md#L1-L53)；[Apache-2.0 许可证](https://github.com/yesme/hctl/blob/3148042cb2faf8df0dc8be92710b9468c8618516/LICENSE)

<a id="e-l2-conductor"></a>
## E-L2-CONDUCTOR · Conductor 机械状态后端

L2 的语义核心由 HCTL2 原生建设；HCTL1 是它的直接谱系证据，但不能直接提供完整的 HCTL2 Workflow。Conductor 证明外部 Worker，以及 READY、等待、定时、重试和历史记录等机械状态，可以与真正产生副作用的执行过程分开。HCTL 把它作为 `WorkflowEngineAdapter` 后面的精选依赖。

- [Conductor OSS](https://github.com/conductor-oss/conductor)
- [核心概念](https://docs.conductor-oss.org/devguide/concepts/index.html)
- [部署](https://docs.conductor-oss.org/devguide/running/deploy.html)

Conductor 不选择 Harness，不创建 Seat/Attempt，不解释语义拒绝，不计算 HCTL 法定票数，不签发 Receipt，也不直接写入 Git 或外部系统。Dagu 的[数据优先工作流与 Runner](https://github.com/dagu-org/dagu)只用于对照 Runner 的所有权设计，不是第一阶段后端。

<a id="e-l2-stably-orca"></a>
## E-L2-STABLY-ORCA · Stably Orca 持久监督协议

Stably Orca 在 L2 的亮点不是自动规划，而是把人工或 Agent 主导的监督过程做成持久协议。Run 是持久命名空间和协调者收件箱；Task 保存依赖与状态；每次 Dispatch 把 Task 的一次尝试绑定到具体终端，并记录窗格、句柄、进程实例代次和能力。生命周期对账还会核对当前 Dispatch ID 与受派窗格/句柄，拒绝来源错误或已经过期的心跳与 `worker_done`。FIFO Delivery 会重复交付同一批消息直到收到确认；变更收据按调用者和请求实现幂等；执行者的启动、停止、释放和保留还会记录已经发生的副作用与未清理资源。Decision Gate、远程转发与过期 Dispatch 拒绝进一步补齐了监督过程中的恢复路径。

HCTL 在 L2 采用它的 Dispatch 权威、消息确认与重放、幂等变更收据、执行者资源所有权、失败后的残留状态，以及拒绝过期完成信号的规则。它的现役 Run [明确不负责调度或选择落点与并发度](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/skill-guides/orchestration.md#L102-L181)，自动调度器也[已经退役且不产生副作用](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/skill-guides/orchestration.md#L273-L285)；它没有 HCTL 的 WorkflowRevision、Obligation/Seat/Attempt、法定票数、重新过 Gate，或与证据绑定的 Verdict/Receipt。因此，Stably Orca 是 L2 的持久监督专项参考，不能直接承担 HCTL 的 Workflow 权威事实。

固定版本、数据结构和生命周期检查见 [Stably Orca 的完整审计](#e-l1-stably-orca)。

<a id="e-l2-herdr-boundary"></a>
## E-L2-HERDR-BOUNDARY · Herdr 运行信号边界

Herdr 不提供持久 Workflow，但它对“谁可以写 Agent 状态”处理得足够深入，构成 L2 的边界证据：一个活动窗格只接受一个状态来源；完整生命周期钩子活跃时优先于屏幕状态降级信号，只报告会话身份的钩子不获得生命周期状态写入权，进程退出和事件序号又会撤销或拒绝过期报告。与此同时，`agent prompt --wait` 明确不追踪某一轮对话，已有活动轮次也可能满足等待。因此 HCTL 可以采用它的信号仲裁思路，却必须把 `idle/working/blocked/done` 限定为 L1 观测，不能据此完成 Task、Run、Verdict 或 Receipt。固定源码见 [E-L1-HERDR](#e-l1-herdr)。

<a id="e-l2-zeroclaw"></a>
## E-L2-ZEROCLAW · ZeroClaw SOP

ZeroClaw 不能直接提供 HCTL Workflow，但它的 SOP 引擎是少见的 L2 邻近实现证据：每个 SOP 的准入策略支持 `parallel`、`hold`、`coalesce` 和 `drop`，Run 可以持久化并在重启后恢复；人工介入/检查点、经过身份认证的审批组与法定票数、只追加审批审计、拒绝作用域版本已经过期的提示、修改/修订、步骤级工具范围，以及重试/跳转，共同形成了可复用的 Gate 与准入失败用例库。

HCTL 只借鉴绑定版本的人类决策、准入与背压，以及默认拒绝的策略测试；不把 SOP 当作 WorkflowRevision，不把事件触发当作 Start 授权，不把 Agent `sop_advance` 当作 Verdict，不把工具收据当作 HCTL Receipt，也不把 ZeroClaw Run 数据库当作领域权威事实。固定版本为 [`v0.8.4 / a56c345d`](https://github.com/zeroclaw-labs/zeroclaw/tree/a56c345d51dd8ab562e9351e0d4ab83f6a741db9)（MIT 或 Apache-2.0）。[语法说明](https://github.com/zeroclaw-labs/zeroclaw/blob/a56c345d51dd8ab562e9351e0d4ab83f6a741db9/docs/book/src/sop/syntax.md)与[运行时契约](https://github.com/zeroclaw-labs/zeroclaw/blob/a56c345d51dd8ab562e9351e0d4ab83f6a741db9/docs/book/src/sop/how-it-works.md)对默认持久化行为仍有冲突，初始化失败时还会降级到进程内内存，因此不能承担 HCTL 的权威事实。

<a id="e-l1-stably-orca"></a>
## E-L1-STABLY-ORCA · Stably Orca

### L1 核心价值与跨层画像

Stably Orca 的产品主轴是以 worktree 为中心的执行环境：每个 worktree 拥有独立分支、文件和 Agent 终端，PTY 由守护进程而不是桌面窗口持有。桌面应用退出但守护进程仍存活时，可以重新连接原进程并恢复布局、分屏、滚屏和焦点；守护进程已经退出时，只能创建新进程，再恢复布局、历史显示，或调用服务提供方的原生会话恢复。两条路径不能都笼统地叫作“会话恢复”。

它还把远程主机、差异审阅、分块暂存、提交、推送和 PR 评审串进同一条执行路径，并用运行时代次、PTY 代次和进程实例代次拒绝过期句柄。这些能力共同构成它在 L1 的核心价值。

### 四层设计亮点与边界

| 层 | 设计深度 | 定位与边界 |
| --- | --- | --- |
| L4 | 很弱 | Native Chat 只是同一 PTY 上的实验性结构化投影，底层终端才是事实来源；没有独立的 Project Room、意图账本或长期协作记忆。 |
| L3 | 中等 | Workspace Board、工作区检查点和外部系统绑定已经可用；本地看板状态还可以选择同步到 Linear。但卡片身份仍是 worktree，`workspaceStatus` 明确用于人工整理侧栏，没有独立 Task、TaskRevision、验收或评审契约。 |
| L2 | **专项参考** | 已实现持久 Run 收件箱、Task 依赖、Dispatch 生命周期权威、消息交付确认与重放、幂等变更收据、心跳、重试隔离、Decision Gate、执行者资源回收和远程转发。这不是概念演示；但现役 Run 明确不调度，也不决定落点和并发度，自动调度器命令已经退役且不产生副作用，同时缺少通用 WorkflowRevision、Obligation/Seat/Attempt、法定票数和证据治理。 |
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
| L3 | 同时支持 Superset 原生 Task 和外部系统 Task，并能把 Task 内容变成 Workspace 的 Agent 提示词；`task.start` 通过只向前推进、可重复调用的状态更新同步外部系统。Workspace Board 的分栏则从 Agent 运行信号、PR 状态和归档原因派生，不是独立的 Task 生命周期。 | 只保留边界证据：Task 适合充当外部来源和启动入口，但没有冻结的 TaskRevision、验收合同或独立的评审与承诺事实；Workspace、分支和 PR 状态不能代替 HCTL Task。 |
| L2 | Automation 保存定时规则、目标设备和运行历史，但它把 Workspace 创建成功记作这次运行的成功，明确不追踪 Agent 的执行结果，而且采用至少一次投递。官方编排 Skill 也明确说明：Superset 只提供会话传输，依赖关系和完成状态由协调者保存在工作上下文中；完成标记只是提示词约定，不是持久事件。 | 这是明确的边界证据：投递已接受、Workspace 已创建、终端存在，都不等于执行结果，更不等于 Workflow、Verdict 或 Receipt。可以采用幂等投递要求和无界面投递接口，但不能把 Automation 或工作上下文中的协调表当作 HCTL L2 事实。 |
| L1 | **核心参考。** 独立的 `pty-daemon` 持有 PTY，`host-service` 只通过 Unix 域套接字使用它；主机服务重启不影响 shell 进程，守护进程平滑升级时还能通过文件描述符移交（fd handoff），把同一 PTY 交给继任进程。主机服务与渲染端使用 `epoch:seq` 重连：在保留范围内精确补发，首次连接发送末尾快照（`tail`），代际不符或缺口过大时显式重新锚定（`reanchor`）；2 MiB 的补发环形缓冲区有界，单个慢渲染端的待发缓冲超过 8 MiB 时只断开该客户端，不拖死 PTY。SQLite 保存终端记录、Agent 绑定和 `disposeRequestedAt` 终止意图；回收器会重试失败的终止操作，守护进程断连后先向继任守护进程核实真实会话，再决定哪些绑定成为可恢复候选。Workspace 删除先写归档墓碑，再依次完成预检、`teardown` 清理脚本、PTY、worktree、分支和缓存清理；失败时恢复可见，进程崩溃后由对账器继续。 | 采用 PTY 进程所有权、文件描述符移交与接管、分代重连和显式降级、有界的慢客户端隔离、持久终止意图与回收器、终端与 Agent 会话绑定、先核实守护进程实际状态再宣告死亡，以及“先写意图、再执行清理”的可恢复分阶段 worktree 清理流程。CLI/MCP 的 `terminal list/read/send/close` 还可作为无 Workbench 时的最小控制面。 |

### “持久终端”实际保证到哪里

Superset 的几类恢复必须分开描述：

- `pty-daemon` 仍存活时，桌面或 `host-service` 重启可以接管原 PTY；守护进程平滑升级时，文件描述符移交可以保留同一 shell PID；
- 守护进程内部的 `SessionStore` 只是进程内映射表，每个会话只有 64 KiB 环形缓冲区，不写入磁盘；渲染端的 2 MiB 补发环形缓冲区也位于主机服务内存中；
- 守护进程被真正杀死或机器重启后，原进程无法保留。系统只能创建新的 shell，并在已有终端与 Agent 绑定、外部系统会话 ID 仍可用时尝试恢复 Agent 会话；
- `epoch:seq` 的精确模式只覆盖主机服务仍保有对应代际和字节范围的情况。代际变化或缺口超出环形缓冲区时会进入 `tail` 或 `reanchor`，不能宣称任意断线都能不重复、不遗漏地恢复。

HCTL 的失败语义必须采用上述细分，不能只写“应用重启后会话仍在”。Superset 当前的守护进程协议使用 Unix 域套接字和文件描述符移交，并明确没有 Windows ConPTY；HCTL 只能借鉴机制和测试，仍需通过自己的跨平台 `RuntimeBackend` 契约实现。

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

Herdr 的控制方记录是进程内的客户端所有者映射，没有代次、TTL、持久确认游标或跨重启租约，因此不能直接替代 `TerminalInputLease`。固定基线的 SCM 操作面覆盖 worktree 生命周期、分支以及 ahead/behind 状态，但不覆盖 Stably Orca 那样的内建 diff、分块暂存、commit/push 与 PR 评审交付链。Workspace、窗格、Agent 名称和服务提供方会话引用也不承担 HCTL 的 Project、Task、Run 或 Attempt 身份。Herdr 因而是终端所有权、控制、状态仲裁与恢复方面的 L1 专项参考，并通过[运行信号边界](#e-l2-herdr-boundary)为 L2 提供独立证据。

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

HCTL 不采用“Everything is a Plugin”，而采用[固定内核与受控端口](../system.md#固定内核与受控端口)：

- Repo/Project/Task/Run 身份、命令准入、权限、版本与证据、领域归约器、持久账本、隔离栅栏和 Receipt 固定在内核中；
- Harness、RuntimeBackend、Task Source、Workflow Engine、Chat Surface 和渲染组件通过类型化端口进行替换；
- 多个提供方可以声明同一个带命名空间和版本的能力，唯一的是一次已经选定的权威绑定；插件加载顺序和钩子优先级不能决定权限或语义结果；
- `ExtensionRevision` 与 `ResolvedPortBinding` 固定代码、接口、数据结构、配置、依赖图和信任级别；Run、Attempt、Invocation、Task Source 与外部聊天渠道在各自正确粒度冻结绑定；
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
| [assistant-ui](https://www.assistant-ui.com/docs/api-reference/primitives/message) | 有明确作用域的 Message/MessagePart/Action 渲染器 | 不采用 Thread、运行时、Store、Composer、Cloud 或 Queue |
| [virtua](https://github.com/inokawa/virtua) | 支持动态高度的 React 视口 | 不负责 Room 的顺序、游标或跟随策略 |
| [Rocket.Chat](https://github.com/RocketChat/Rocket.Chat/tree/develop/apps/meteor/client/views/room/MessageList)、[Mattermost](https://github.com/mattermost/mattermost/tree/master/webapp/channels/src/components/dynamic_virtualized_list)、[Zulip](https://github.com/zulip/zulip/blob/main/docs/subsystems/unread_messages.md) | 前插消息、定位到指定消息、未读状态、动态高度和无障碍测试 | 合并为行为证据；不采用其后端或领域模型 |

Tiptap/ProseMirror 是 L4 精选的 Composer 基础组件，不是产品参考项目：[自定义扩展](https://tiptap.dev/docs/editor/extensions/custom-extensions)、[React 节点视图](https://tiptap.dev/docs/editor/extensions/custom-extensions/node-views/react)。

## L3 外部系统与观察清单

Linear 和 GitHub 提供外部字段的写入权威，也是没有 Workbench 时的外部系统原生降级方案；它们不是 HCTL 的 Task 模型：

- Linear：[GraphQL](https://linear.app/developers/graphql)、[Webhook](https://linear.app/developers/webhooks)、[速率限制](https://linear.app/developers/rate-limiting)、[分页](https://linear.app/developers/pagination)
- GitHub：[Projects API 指南](https://docs.github.com/en/issues/planning-and-tracking-with-projects/automating-your-project/using-the-api-to-manage-projects)、[GraphQL 参考](https://docs.github.com/en/graphql/reference/projects)、[Webhook](https://docs.github.com/en/webhooks/webhook-events-and-payloads)、[REST 最佳实践](https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api)
- React Aria：[Kanban 示例](https://react-aria.adobe.com/examples/kanban)、[拖放](https://react-aria.adobe.com/dnd)——只作为 UI 基础组件。

<a id="l1-selected-evidence"></a>
## L1 精选证据与能力观察清单

### 精选实现证据

| 证据 | 独特价值 | 边界 |
| --- | --- | --- |
| [Termio `d1fdac8…`](https://github.com/termio-sh/termio/tree/d1fdac84046805d4056e082f982e6beb6072b61c) / [ATP](https://www.termio.sh/docs/atp) / [会话控制](https://www.termio.sh/docs/session-control) | Manifest、稳定的 Session URI、监听/心跳/信号，以及带数据结构版本的控制协议 | MIT；ATP 不是 HCTL 或行业通用的传输标准，也不作为跨平台 Backend 的权威实现 |
| [Herdr `v0.8.0 / 346411fa`](https://github.com/herdrdev/herdr/tree/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7) / [专项审计](#e-l1-herdr) | 后台服务持有 PTY；观察/控制与原始/语义操作面分离；单写者接管；状态信号仲裁与分级恢复 | Apache-2.0；控制方不是持久租约，运行状态不等于领域完成；完整边界见专项审计 |
| [xterm.js](https://github.com/xtermjs/xterm.js/) | 嵌入式终端渲染器，以及 CJK、输入法、无障碍和流量控制 | MIT；只负责前端，不拥有 PTY、进程或 Session |
| [WezTerm](https://wezterm.org/cli/cli/index.html) | 成熟的跨平台外部终端与 CLI | MIT；不嵌入应用，也不把 Mux 协议当作 ABI |
| [Zellij](https://zellij.dev/documentation/programmatic-control.html) / tmux | 真实的 Mux 与运行环境候选 | 经过同一套契约测试后，第一阶段只选择一个；Pane 名称不作为身份标识 |

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

## 标准与通用库，不作为产品主参考

- [Agent Client Protocol](https://agentclientprotocol.com/protocol/v1/overview) / [Rust SDK](https://github.com/agentclientprotocol/rust-sdk)：L1 的 Harness 接入标准。
- [Agent Skills](https://agentskills.io/specification)：用于 L4 的 Expertise 选择，以及 L1 的交付与绑定；Skill 只提供指导，不是 Gate。
- [MCP Resources](https://modelcontextprotocol.io/specification/2026-07-28/server/resources) / [Prompts](https://modelcontextprotocol.io/specification/2026-07-28/server/prompts)：传输 Context 和工具信息，不定义 Project/Task 的决定权。
- [React Flow](https://reactflow.dev/) / [Dagre](https://github.com/dagrejs/dagre)：用于 L2 的只读可视化与布局。
- [Electron 安全指南](https://www.electronjs.org/docs/latest/tutorial/security) / [MessagePorts](https://www.electronjs.org/docs/latest/tutorial/message-ports)：用于跨层可信 UI 与数据传输。

## 复用决策用语

所有证据最终只归入五种复用决策：**采用为依赖（Adopt dependency）**、**移植有边界的组件（Port bounded component）**、**适配协议（Adapt protocol）**、**仅参考行为（Behavior reference）**、**暂缓（Defer）**。不得给整个产品一个“取代 HCTL”的总分，也不得把参考项目中的 Session、Conversation、Project、Task、Run 名称或内部数据库带入 HCTL 的公开数据结构。

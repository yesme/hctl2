# Grok Bot 与 Grok Build

> 类别：② Agent 协作平台 · 证据编号：E-GROK-BOT<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

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
- 补充证据（非授权重建，可能下架）：[b-nnett/grok-bot-0.18-reconstructed](https://github.com/b-nnett/grok-bot-0.18-reconstructed)（commit `a9f633e`，无许可证）——只引用行为与协议形状、不移植任何代码；proto 生成物与字符串常量可信，模块名为重建者推断，`frontend/` 与作者自加扩展不作证据；主证据仍为官方文档

## 复核记录

- **2026-08-25**：以 [b-nnett/grok-bot-0.18-reconstructed](https://github.com/b-nnett/grok-bot-0.18-reconstructed)（commit `a9f633e`）对本条目做了客户端源码复核，方法、可信度分层与逐条锚点见 [Grok Bot 重建源码审计备忘录](grok-bot-reconstructed-audit-20260825.md)。该仓库是对 0.18.0 公开安装包（bundle `com.anysphere.sand`，Anysphere 签名、`downloads.cursor.com` 分发）的非授权重建：上游 Node 侧 bundle 未压缩且带源路径标记，所以 proto/RPC、字符串常量、系统提示词是一手证据，模块/类型名是重建者推断，`frontend/`、实验开关默认值与作者自加的 inference router / local Docker / usage tracking 不作证据；无许可证并以 LFS 再分发原厂安装包，随时可能下架；服务端逻辑（审批分类器 prompt、云机编排、灰度）不可见。复用决策不变（仅参考行为），客户端侧证据等级升为"源码印证"。**印证**：账号级共享云机（系统提示原文 "The box is ONE persistent Linux machine shared by all of this user's agents… the desktop is per-agent"，`source/host/runner/prompt-collector-glue.ts:192`；每个 Bot 是同一台机上的一个 X display 窗口）、50 Bot / 6 成员上限、secret request 的值从 `submitSecret` 直接落 `connector-secrets/<agentId>/<platform>.json` 而 transcript 只记 `secretProvided:true`、MCP token 留服务端、WebAuthn 逐次同意、`request_box_help` 的 reason 枚举 auth/captcha/payment/other、routine 记录硬截 20 条且删除即 rmSync、没有任务对象（L3 只有模型自报且对用户隐藏的 TodoWrite）、Auto Review 走服务端 `classifySandAutoReview`。**需改写**：(1) Bot 间私信在两端 transcript 都落 `fromAgent/toAgent` 条目且用户可见（`agent-to-agent-messaging.ts:275-335`），不可见的是接收方 `hidden:true` 的处理轮次，且唤醒不依赖用户在场；(2) Auto Review 运行时是 off/shadow/enforce 三态，设置开启后默认 shadow（分类器跑但不拦），enforce 由 Statsig gate `sand_auto_review` 决定（代码兜底 false）；allow/block 指令只是喂给服务端分类器的文本，客户端没有"保守方优先"逻辑；shell/MCP 面 BLOCK 后首轮直接拒，由被审模型带 `request_smart_mode_approval=true` 重发才弹卡（`sand-auto-review.ts:61-69`、`create-shell-tool.ts:614-620`）；(3) "七类固定必审批动作"在客户端不存在，确定性的只有 cloud agent 生命周期动作与不可审批的团队 denylist；(4) routine 的创建/修改/暂停/删除在 0.18 任何路径都不经审批（`automationWrite` 恒为 off，`sand-auto-review.ts:18,61-67`），Bot 经 `update_state` 可自改 routine、退出 project、断开 channel、改名；(5) "审计日志未交付"应为"采集与上报链路已实现，本地 `audit.jsonl` 始终写入，上报受 gate `sand_action_audit_logs` 控制"；(6) 接管不是互斥锁——`request_box_help` 只结束本轮，其他 Bot 仍可驱动同一台机，交还后的复活是隐藏提示词；(7) 群聊默认是单账号内多 Bot，跨用户共享房间由 gate `sand_multiplayer` 灰度，且共享房间/群聊里的 Bot 不注入团队规则（`resolveRules` 固定 `[]`）——"可见"与"受约束"在上游是分离的；(8) "记忆不能替代权威来源"在代码与提示词中无对应物，只有 "Never fabricate data"；(9) 凭证有三个作用域：Cursor 登录 token（桌面 keychain）、box secrets（整台云机环境，全 Bot 共享）、connector 凭证（Bot×平台，删 Bot 不清理）。**新增可作适配协议的形状**：审批对象 `{id, agentId, surface, fingerprint, reason, summary, proposedRule, userMessageEpoch, hostGeneration, expiresAtMs, status}` + 失效原因枚举（ttl/cancelled/user_redirect/settings_change/session_end/quiesce，"过期≠拒绝"）+ 执行前复核脚本 hash 与屏幕状态 hash；`SandAuditEvent`（agent_id/turn_id/box_id + 四类动作）；transcript 事件 `ordered{replicaKey, epoch, sequence}` + snapshot coverage；人类命令入口 `clientNonce + inputDigest` 幂等账本；本机工具权限 allow-once/deny/always/never 与按 agentId+toolCallId 定界、回合结束自动退休的授权；Cloud Agent 的六态 PR 归约（none/unknown/open/draft/merged/closed）。**完成权威**：turn 的最低完成证据是一次真实 SendMessage/reaction（否则最多 3 次隐藏 nudge）；routine run 的 ok 由宿主写但只表示 turn 没崩；Cloud Agent 完成由服务端状态归约但 finished 无 PR 仍算 completed——全部止于"过程结束"，没有一处归约到交付物。

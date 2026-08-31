# First Tree

> 类别：② Agent 协作平台 · 证据编号：E-L4-FIRST-TREE<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-l4-first-tree"></a>
## E-L4-FIRST-TREE · First Tree

### 核心价值与跨层画像

First Tree 真正跑通的是协作闭环，而不是任务或工作流闭环：Team 中的持久 Chat 或 SCM 事件形成共同上下文，受管 Agent 工作后把结果交还给用户或 SCM；只有同时通过 Decision Test 和 Durability Test 的稳定结论，才会经由另一条有来源支撑、需要审查的流程写入 Context Tree。写入 Context Tree 并不是每次 Chat 的自动收尾；没有具体来源材料时，规则明确要求什么都不写。

它最突出的价值仍在 L4：证明以 Chat 和 Context 为主轴可以维持长期协作。但源码还给出了两组值得跨层引用的深入机制：L2 可以参考精确快照、有来源支撑的写入，以及 Reviewer 对精确 head/digest 的批准与失效规则；L1 可以参考受管执行提供方的会话代次、ACK、重试、恢复和能力契约。它并没有 HCTL 意义上的 Project、Task、Task Revision、Workflow 或 Run；`task chat` 只是创建 Chat 的一种模式，GitHub Task Agent 的事实仍是 Issue/PR 加权威 Chat，cron 也只是把触发器转成消息。因此，这些跨层亮点只是专项机制，不能把 First Tree 整体当成通用 L2 编排器、L3 Task 系统或 L1 终端管理器。

协作拓扑需要拆成两半评价。First Tree 的 `chat send` 要求显式 recipient，正文里的 `@name` 本身不触发路由；handoff、邀请与后续消息都留在持久 Chat 中，对人可见且可追溯。这证明“显式寻址 + 持久 Chat + 可见 handoff”可以避免隐藏的 peer RPC。与此同时，Agent 可以在运行中 `invite + send`，接收者还能继续寻址第三个 Agent，使参与者集合与协作图由模型临场扩张。HCTL 采用前一半，不采用后一半作为默认拓扑：普通 Room 中 Agent 只能建议下一条协作边，由 human actor 提交；自动化边则由 reducer 按冻结的 Workflow Revision 创建。这里记录的是参考取舍，实际权限与命令约束仍以规范文档为准。

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
- 补充协作拓扑快照 [`9a7dd4d9`](https://github.com/first-tree-ai/first-tree/tree/9a7dd4d94373921cfe2022bfef91c132fdf74824)：[Agent runtime briefing](https://github.com/first-tree-ai/first-tree/blob/9a7dd4d94373921cfe2022bfef91c132fdf74824/packages/client/src/runtime/templates/agent-briefing.ejs#L77-L93)、[handoff 规则](https://github.com/first-tree-ai/first-tree/blob/9a7dd4d94373921cfe2022bfef91c132fdf74824/packages/client/src/runtime/templates/agent-briefing.ejs#L161-L183)、[同任务 handoff](https://github.com/first-tree-ai/first-tree/blob/9a7dd4d94373921cfe2022bfef91c132fdf74824/docs/cli-reference.md#L646-L670)、[`chat send` 约束](https://github.com/first-tree-ai/first-tree/blob/9a7dd4d94373921cfe2022bfef91c132fdf74824/apps/cli/src/commands/chat/send.ts#L19-L80)与[邀请权限](https://github.com/first-tree-ai/first-tree/blob/9a7dd4d94373921cfe2022bfef91c132fdf74824/packages/server/src/services/chat/membership/invite.ts#L31-L57)
- 当前主干的 GitHub [受众与激活规则](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/services/scm/github/audience.ts)、[投递流程](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/services/scm/github/delivery.ts)、[entity↔Chat 绑定](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/services/scm/github/entity-chat.ts)与[最终回复发布](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/services/scm/github/task-reply-publisher.ts)
- [执行提供方契约](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/client/src/providers/README.md)、[运行时数据结构](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/shared/src/schemas/runtime-provider.ts)、[会话控制 CLI](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/apps/cli/src/commands/agent/session/control.ts)、[内部 `tmux` 驱动](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/client/src/providers/claude/tui/tmux-session.ts)与[cron 表](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/server/src/db/schema/cron-jobs.ts)
- 仅主干存在的 Feishu [bot 绑定](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/db/schema/im-bot-bindings.ts)、[chat 绑定](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/db/schema/im-chat-bindings.ts)、[入站处理](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/services/integrations/feishu/inbound.ts)、[连接管理器](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/server/src/services/integrations/feishu/manager.ts)与[验收契约](https://github.com/agent-team-foundation/first-tree/blob/f0d46f9ec8b14ace536d242db8860065c124f2c7/packages/qa/cases/cross-surface/feishu-agent-channel.md)

## 复核记录

- **2026-08-24**：审计快照之后主干仅前进 19 个提交（v0.5.21），上述结论仍有效。按提交路径直方图，其工程投入最大单块是受管执行/provider 层（client + cli 约四分之一路径变更，provider 清单已扩到 9 家），聊天与 Context 是叙事中心、运行时是工程中心——与本文"L4 核心 + L2/L1 专项"的双重定位一致；直方图同时证实它没有任何看板/任务卡片模块（kanban≈0）。

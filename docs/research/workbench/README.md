# Workbench 与相邻完整产品

本目录集中保存“可以被当作完整工作台或 Agent 产品来比较”的候选研究。它们通常同时涉及 Room、Kanban、Workflow、Terminal 中的多个场景，单独放在研究根目录会把产品比较与协议、后端、运行时、上下文等专题研究混在一起。

总表、证据准入规则、分层口径和复用决策仍以 [`docs/research/README.md`](../README.md) 为准；这里负责归档和导航，不另写一份选型结论。

## 协作工作台候选

| 候选 | 研究记录 | 主要观察角度 |
| --- | --- | --- |
| First Tree | [first-tree.md](./first-tree.md) | Chat/Context、受管运行时与跨渠道协作 |
| Claude Tag | [claude-tag.md](./claude-tag.md) | Slack 内的持久协作与身份 |
| Grok Bot / Grok Build | [grok-bot.md](./grok-bot.md) | 一等参与者、审批、观察与接管 |
| Cumora | [cumora.md](./cumora.md) | 团队聊天、记忆、唤醒与 Agent 自主性 |
| LobeHub | [lobehub.md](./lobehub.md) | Chat、Context 管道与异构 Harness |
| Multica | [multica.md](./multica.md) | Project/Issue、租约、恢复与 worktree |
| Helio | [helio.md](./helio.md) | 工单、审批、凭据与确定性门禁 |
| Codeg | [codeg.md](./codeg.md) | 多 Agent 会话、异步 WorkTask 与 ACP |
| Stably Orca | [stably-orca.md](./stably-orca.md) | 终端、worktree Board 与可靠派发 |
| Superset | [superset.md](./superset.md) | 终端复用、远程控制与会话恢复 |

## 相邻的完整 Agent 产品

这些产品不一定以 Workbench 为主，但它们提供完整运行时、协作入口或任务机制，研究时需要与工作台候选放在一起比较。

| 候选 | 研究记录 | 主要观察角度 |
| --- | --- | --- |
| OpenClaw | [openclaw.md](./openclaw.md) | 多渠道身份、路由与个人助理网关 |
| Hermes Agent | [hermes-agent.md](./hermes-agent.md) | 持久 Task/Attempt、领取和回收 |
| Rakazo | [rakazo.md](./rakazo.md) | 自托管 Bot、租约与可移植工作区 |
| ZeroClaw | [zeroclaw.md](./zeroclaw.md) | SOP 准入、审批与失败处理 |

## 补充审计

- [Grok Bot 0.18 客户端重建源码审计](./grok-bot-reconstructed-audit-20260825.md)：对主条目的补充证据，不单独作为候选。

协议与基础设施专题继续留在上一级，例如 Matrix homeserver、task backend、workflow engine、Herdr/tmux 运行时、Harness 接入、桌面壳和 Context 研究。这样按“完整产品”和“可替换基础部件”分开，查找候选时有一个入口，又不会把所有研究都塞进 Workbench 分类。

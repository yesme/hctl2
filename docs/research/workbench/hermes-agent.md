# Hermes Agent

> 类别：③ 独立 Agent 产品 · 证据编号：E-L3-HERMES-AGENT<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-l3-hermes-agent"></a>
## E-L3-HERMES-AGENT · Hermes Agent

Hermes Agent 的独特价值是由 Agent 操作的持久 Task/Attempt 协议：SQLite Board 保存 Task、Run/Attempt、依赖、评论和工作区；调度器负责原子领取、心跳、过期或崩溃 Worker 的回收、依赖满足后的状态推进，以及协议违规时自动阻塞；CLI、Chat 斜杠命令和 Dashboard 共用同一套命令内核。它为 L3 提供了重启恢复和无 Workbench 操作方面的实现证据。

HCTL 借鉴 Task/Attempt 分离、领取与回收、持久评论和共用命令内核；不把 Board 当作 Project，不把 profile/memory 当作 Participant/Project，不把模型自报完成当作 Receipt，也不把单机调度器当作 L2 权威事实，更不让 LLM 的目标判断决定语义完成。固定版本为 [`v2026.8.13 / f80f453a`](https://github.com/NousResearch/hermes-agent/tree/f80f453ae0679347e38abc917c7f94f717bf96c5)（发布名称 `v0.20.1`，MIT）；证据见 [Kanban 指南](https://github.com/NousResearch/hermes-agent/blob/f80f453ae0679347e38abc917c7f94f717bf96c5/website/docs/user-guide/features/kanban.md)、[README](https://github.com/NousResearch/hermes-agent/blob/f80f453ae0679347e38abc917c7f94f717bf96c5/README.md)与[许可证](https://github.com/NousResearch/hermes-agent/blob/f80f453ae0679347e38abc917c7f94f717bf96c5/LICENSE)。

## 复核记录

- **2026-08-24**：按提交路径直方图，Kanban 子系统仅占其 2026 年开发投入约 1.5%（2026-04 才出现的年轻模块），当前投入重心是桌面聊天客户端——本文借鉴的是其边缘功能而非主轴，对其长期维护承诺应保守估计；这不否定代码证据本身。
- **2026-10-01**：主干 [`233434b4`](https://github.com/NousResearch/hermes-agent/tree/233434b414476e5f1ed0f6836922128e74d7d2f4)（2026-10-01），最新发布 v2026.9.24（即 v0.21.5），较审计基线约 2.46 万个提交，许可证仍为 MIT。[Kanban 指南](https://github.com/NousResearch/hermes-agent/blob/233434b414476e5f1ed0f6836922128e74d7d2f4/website/docs/user-guide/features/kanban.md)还在，Task/Run、依赖、评论、领取与回收的模型没变，加固集中在完成边界：空证据不能完成；创建时声明 PR 完成条件后，完成时按精确 head 读取必需检查，第一个匹配的 PR 永久绑定，检查缺失、挂起或失败都不能完成。Kanban 仍只占约 1.3% 的提交，桌面约占 20%。复用决策不变，这道完成门可补作「完成不靠模型自报」的行为证据。会话结构：[网关](https://github.com/NousResearch/hermes-agent/blob/233434b414476e5f1ed0f6836922128e74d7d2f4/gateway/session.py)按平台、聊天和讨论串拼会话键，群聊默认按人分开，讨论串默认共享，每个会话是一条线性记录；会话之间以 parent_session_id 连成谱系，分压缩续接、`/new` 重开和 `/branch` 分叉三种边；`/undo`、`/retry` 以持久记录为准回退，`/rollback` 恢复文件检查点。新增的是：在 Discord、Telegram、Slack、Matrix 上，[`/branch`](https://github.com/NousResearch/hermes-agent/blob/233434b414476e5f1ed0f6836922128e74d7d2f4/gateway/slash_commands_branch_thread.py) 默认另开一个同级讨论串承载分叉。桌面侧栏把分叉会话嵌在父会话下（[会话树](https://github.com/NousResearch/hermes-agent/blob/233434b414476e5f1ed0f6836922128e74d7d2f4/apps/desktop/src/lib/session-branch-tree.ts)）；Kanban 依赖是父子 DAG，核心面板只有依赖标签和进度，未见图形视图，社区插件 kanban-gantt 另给带依赖连线的时间线。

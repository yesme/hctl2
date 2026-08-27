# Hermes Agent

> 类别：③ 独立 Agent 产品 · 证据编号：E-L3-HERMES-AGENT<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-l3-hermes-agent"></a>
## E-L3-HERMES-AGENT · Hermes Agent

Hermes Agent 的独特价值是由 Agent 操作的持久 Task/Attempt 协议：SQLite Board 保存 Task、Run/Attempt、依赖、评论和工作区；调度器负责原子领取、心跳、过期或崩溃 Worker 的回收、依赖满足后的状态推进，以及协议违规时自动阻塞；CLI、Chat 斜杠命令和 Dashboard 共用同一套命令内核。它为 L3 提供了重启恢复和无 Workbench 操作方面的实现证据。

HCTL 借鉴 Task/Attempt 分离、领取与回收、持久评论和共用命令内核；不把 Board 当作 Project，不把 profile/memory 当作 Participant/Project，不把模型自报完成当作 Receipt，也不把单机调度器当作 L2 权威事实，更不让 LLM 的目标判断决定语义完成。固定版本为 [`v2026.8.13 / f80f453a`](https://github.com/NousResearch/hermes-agent/tree/f80f453ae0679347e38abc917c7f94f717bf96c5)（发布名称 `v0.20.1`，MIT）；证据见 [Kanban 指南](https://github.com/NousResearch/hermes-agent/blob/f80f453ae0679347e38abc917c7f94f717bf96c5/website/docs/user-guide/features/kanban.md)、[README](https://github.com/NousResearch/hermes-agent/blob/f80f453ae0679347e38abc917c7f94f717bf96c5/README.md)与[许可证](https://github.com/NousResearch/hermes-agent/blob/f80f453ae0679347e38abc917c7f94f717bf96c5/LICENSE)。

## 复核记录

- **2026-08-24**：按提交路径直方图，Kanban 子系统仅占其 2026 年开发投入约 1.5%（2026-04 才出现的年轻模块），当前投入重心是桌面聊天客户端——本文借鉴的是其边缘功能而非主轴，对其长期维护承诺应保守估计；这不否定代码证据本身。

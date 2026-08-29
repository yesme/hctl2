# OpenClaw

> 类别：③ 独立 Agent 产品 · 证据编号：E-L4-OPENCLAW<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-l4-openclaw"></a>
## E-L4-OPENCLAW · OpenClaw

OpenClaw 最值得参考的是 L4 的外部频道接入边界：它把账号、对端和讨论串归一为确定性路由键，并支持精确绑定、讨论串继承、私信作用域、配对与允许名单、房间环境事件、防止机器人循环，以及按频道能力降级投递。这说明：没有 Workbench 时，Chat 界面仍需要稳定的外部身份、确定性路由和逐频道降级，不能让模型猜测频道，也不能按显示名称分发。

HCTL 只借鉴适配、路由、配对、防循环和降级测试；OpenClaw 的 channel/session/workspace/agent 不映射为 Project/Room/Task/Run，环境聊天不会自动成为权威 Context，Gateway、cron 或 delegation 也不成为 L2 的权威事实。固定版本为 [`v2026.7.1-2 / 0790d9f5`](https://github.com/openclaw/openclaw/tree/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c)（MIT）；证据见[频道路由](https://github.com/openclaw/openclaw/blob/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c/docs/channels/channel-routing.md)、[README](https://github.com/openclaw/openclaw/blob/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c/README.md)与[许可证](https://github.com/openclaw/openclaw/blob/0790d9f593ad30c940ed93b5872a8cf6d6f3cf8c/LICENSE)。

## 复核记录

- **2026-08-24**：2026-04 起出现 extensions/codex（监督原生 Codex 会话），2026-07 起 UI 出现 workboard/worktrees 页——正朝编码代理监督面扩张（目前约 2-3% 投入）。按现行"只借频道边界"的立场无需改动；后续做 L1/L3 邻近证据扫描时可补充观察。

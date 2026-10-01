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
- **2026-10-01**：主干 [`bb40ea11`](https://github.com/openclaw/openclaw/tree/bb40ea1130122fd73ad22bff85ad45cb29374fae)（2026-10-01），最新发布 v2026.9.7（2026-09-29，打在发布分支上）；审计基线标签同样在发布分支上，主干自 2026-07-18 起约 3.3 万个提交，许可证仍为 MIT。[频道路由](https://github.com/openclaw/openclaw/blob/bb40ea1130122fd73ad22bff85ad45cb29374fae/docs/channels/channel-routing.md)更严了：多个 agent 都没有匹配的绑定时，不再默认挑名单里第一个；插件回执报失败、被抑制或空跑时，存储的路由和投递记录不变；广播群扩成带轮数和回合上限的 agent 群讨论串；会话存储改为每个 agent 一个 SQLite。另有[多用户模式](https://github.com/openclaw/openclaw/blob/bb40ea1130122fd73ad22bff85ad45cb29374fae/docs/concepts/multi-user.md)，明说会话归属和可见性只是易用功能，不是安全边界。Workboard 插件默认关闭，占提交不到 1%，codex 扩展约 3%。「只借频道边界」的立场和复用决策都不变。会话结构：会话键按 agent、频道、群或私信、讨论串拼成，私信默认并入主会话；[转录](https://github.com/openclaw/openclaw/blob/bb40ea1130122fd73ad22bff85ad45cb29374fae/src/config/sessions/transcript-tree.ts)是只追加的树，靠叶指针确定当前路径，聊天界面可「回退到此」或「从此分叉」；控制台侧栏以 Home 为根，把分叉和持久子会话嵌在父会话下（[会话树](https://github.com/openclaw/openclaw/blob/bb40ea1130122fd73ad22bff85ad45cb29374fae/ui/src/components/app-sidebar-session-tree.ts)），未见 DAG 视图。

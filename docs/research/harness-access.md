# OpenCode、Pi 与 Kimi Code

> 类别：① Coding Harness · 证据编号：E-L1-HARNESS-ACCESS<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

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

## 复核记录

- **2026-10-01（Pi 会话树补记）**：基线更新为 [Pi `v0.99.2 / 005af57d`](https://github.com/earendil-works/pi/tree/005af57d88ee23b33778f343a9595b32e67ff788)（2026-09-30 发布；本条引用的文件与当日 main `0f8740bb` 一致）。按[会话格式](https://github.com/earendil-works/pi/blob/005af57d88ee23b33778f343a9595b32e67ff788/packages/coding-agent/docs/session-format.md)，一个会话一份 JSONL，条目靠 `id`/`parentId` 连成树；`/tree`（默认双击 Esc 打开，缩进列表加连线）在同一文件里移动叶指针，离开分支时可选写入 `branch_summary`（`fromId` 指向被离开的叶），`label` 条目充当书签；`/fork`、`/clone` 另起文件，头部用 `parentSession` 指回来源；送给模型的上下文只取叶到根这一条路径。HTML 导出在侧栏画出整棵树，JSONL 导出只含当前分支。[树导航](https://github.com/earendil-works/pi/blob/005af57d88ee23b33778f343a9595b32e67ff788/packages/coding-agent/src/core/agent-session.ts#L3888)只改会话位置，不还原工作区文件；内置文件回退的提议已按 not planned 关闭（[#8152](https://github.com/earendil-works/pi/issues/8152)），这一块交给扩展：[pi-rewind-hook](https://github.com/nicobailon/pi-rewind-hook/tree/a62e7c2c89d130b3a02f7f799c15de62743d3fa8) 把 git 快照绑到树节点，并沿 `parentSession` 追溯；npm 上还有 [pi-rewind](https://github.com/arpagon/pi-rewind/tree/91611ad87992fb7b635a41ba68f67916ff6e6ae3) 等几十个同类包。浏览器侧，[pi-session-manager](https://github.com/Dwsy/pi-session-manager/tree/6fc25b685fe851f58421c840dfecba7706da834b) 读取本机会话并画出分支图；[pi-tree](https://github.com/shuowu/pi-tree/tree/3cf6ee90d1d4f9134985123272e8a46d587e3c5d) 是基于 Pi SDK 的独立阅读应用，把同一套会话树搬进网页。

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

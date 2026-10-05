# 自建方案盘点 · 设计层的协议与对象（Claude 席）

> 状态：已核销 · 结论与待裁项已写回任务书 §六
> 基线：main @ 1d09ad6（草案 v0.19.3）
> 去向：.memo/design/community-first-audit-20261005.md §六

## 范围与方法

任务书 §六 待裁清单第 5 项，所有者 2026-10-05 裁「查」。三份代码层的报告都没有逐项对照的三样：控制面与 Agency 之间的端口（`src/crates/agency-proto/proto/agency.proto` 的 13 个调用）、Context Bundle（`docs/design/context.md`、`spec/participant.md`）、Worker Profile（`spec/participant.md` §名册与选入）。每样问两件事：社区里有没有同类的标准；有的话，我们该换、该对齐，还是该留。

社区方案的事实由一个研究助手打开原始出处收集（规范站点、GitHub 仓库与发布页、npm）。其中关键的几条主笔又直接查了一遍，标「主笔复核」；其余标「助手查证」，都附链接。只出对照与建议，不改约束。

## 一、先说结论

1. **三样都没有可以整块替换的社区标准。** 它们承载的是治理语义——多租户配对、冻结并按内容摘要标识的规格与上下文、按派工签发的票据、输入租约、结果保管到确认、事件游标报缺口——这些在 ACP、A2A、MCP 里都没有。这一点和 2026-09-02 部件矩阵对「约束层机制」的结论一致。
2. **但每样在「交给 harness 的那一侧」都有社区格式可以用，我们没有用。** 上下文交进 harness 时，ACP 与 MCP 有通用的内容块；Worker Profile 落到 harness 时，Claude Code 与 Codex 各有原生的 agent 定义文件。现在这两处都是我们自己拼命令行参数。
3. **库里有一处前后不一致**：`docs/research/README.md` 把 ACP 写成「L1 的 Harness 接入标准」，而 `harness-adapters.md` 的决定建议是用各家自己的 JSONL 与协议，实现也是这么做的。按今天的事实，ACP 只能经适配器接 Claude Code 与 Codex，Herdr 不支持 ACP。
4. **和第 3c 包直接相关的一条**：「这一轮答完了」在社区里有三种取法，其中和「Herdr 实际持有 harness 与会话」相容的，是 harness 自己的钩子加 Herdr 的 agent 状态，不是 ACP。见 §二.2。

## 二、Agency 端口

### 1. 对照

| 我们 | ACP v1（Agent Client Protocol） | A2A v1（Agent2Agent） |
| --- | --- | --- |
| 定位 | 编辑器一类的客户端驱动一个编码 agent；JSON-RPC 走标准输入输出，客户端把 agent 当子进程起 | 互不透明的 agent 系统之间远程互调；JSON-RPC、gRPC 或 HTTP，生产环境要求 HTTPS 地址 |
| `pair` 配对、租户 | 没有；`authenticate` 是登录模型服务商 | 没有；`AgentInterface.tenant` 只是不透明的路由字段 |
| `catalog` 名册 | 协议里没有；协议外有 ACP Registry（41 个 agent 条目：id、name、version、distribution、license） | `AgentCard`：name、description、version、capabilities、`skills[]`，可带 JWS 签名 |
| `prepare` 冻结规格加 Bundle | `session/new`（工作目录、附加目录、MCP 服务器），不冻结、不按摘要标识 | 没有对应；直接 `SendMessage` |
| `activate`、`input` | `session/prompt`，一次一轮；会话跨多轮存活 | `SendMessage`；任务处于等输入时用同一个 `taskId` 续 |
| `observe` 带游标、报缺口 | `session/update` 推送：消息片段、思考、工具调用及其状态、计划、用量等；没有序号，不报缺口 | `SendStreamingMessage`、`SubscribeToTask`；重订阅先给一份快照，没有序号，不报缺口 |
| `stop` | `session/cancel`、`session/close` | `CancelTask` |
| `results` 保管到确认 | 只有一轮的 `stopReason`；没有结果对象，也没有确认 | `Artifact`（id、name、parts）；没有「保管到确认」 |
| 票据、输入租约 | 没有 | 没有 |

事实出处：ACP 的方法清单与枚举取自 [schema/v1](https://github.com/agentclientprotocol/agent-client-protocol/blob/main/schema/v1/schema.json)，传输见 [transports](https://agentclientprotocol.com/protocol/transports)，治理见 [governance](https://agentclientprotocol.com/community/governance)（Zed 与 JetBrains 共管，准备移交独立基金会）；A2A 取自 [v1.0.1 的 proto](https://github.com/a2aproject/A2A/blob/v1.0.1/specification/a2a.proto) 与 [规范](https://github.com/a2aproject/A2A/blob/v1.0.1/docs/specification.md)，归 Linux 基金会。ACP 最新稳定发布 v1.10.2（2026-10-01）、A2A 最新 v1.0.1（2026-05-28）为**主笔复核**，其余为助手查证。

**判断：留。** 端口位于控制面与 Agency 之间，承担的正是两份协议都没有的部分。A2A 是跨组织远程调用的协议，拿来做本机托管要补上我们这一整层；ACP 驱动的是一个会话，不是一次受治理的派工。

**可以对齐、不急的一处**：`observe` 的事件种类与 ACP 的 `sessionUpdate`、`ToolKind`、`ToolCallStatus` 有大片重合（约束里的七类观测：生命周期、工具调用、权限请求、文件变化、测试、用量、原始输出）。将来调整观测的格式时，名字与取值可以直接借 ACP 的，省得自己起名。

### 2. Agency 背后：harness 怎么接（和第 3c 包相关）

约束 `spec/participant.md` 的外部概念对齐表已经写了：ACP 一类的结构化接入是「Agency 门后的接入方式之一」，和 PTY 并列。今天的事实：

| 事实 | 出处 | 把握 |
| --- | --- | --- |
| ACP 的 `session/prompt` 返回 `stopReason`，取值 `end_turn`、`max_tokens`、`max_turn_requests`、`refusal`、`cancelled`；会话跨轮存活，客户端可以接着发下一轮 | [schema/v1](https://github.com/agentclientprotocol/agent-client-protocol/blob/main/schema/v1/schema.json)、[prompt-turn](https://agentclientprotocol.com/protocol/prompt-turn) | 主笔复核取值 |
| ACP v2 还在 alpha（2.0.0-alpha.7）：完成信号改成一条 `idle` 状态更新带 `stopReason`，并增加 `error` | [v2 CHANGELOG](https://github.com/agentclientprotocol/agent-client-protocol/blob/main/schema/v2/CHANGELOG.md) | 助手查证 |
| Claude Code 只能经适配器说 ACP：`agentclientprotocol/claude-agent-acp` v0.85.1（2026-10-02），Apache-2.0，基于 Claude Agent SDK | [仓库](https://github.com/agentclientprotocol/claude-agent-acp) | 主笔复核 |
| Codex 只能经适配器说 ACP：`agentclientprotocol/codex-acp` v2.1.1（2026-10-01），LICENSE 文件为 Apache-2.0（版权 JetBrains），内部起 Codex 的 app-server 再翻译 | [仓库](https://github.com/agentclientprotocol/codex-acp) | 主笔复核 |
| 两个适配器在 ACP 组织下，主要提交者是 Zed 与 JetBrains 的人，不是 Anthropic 或 OpenAI 自己维护 | 同上的贡献者列表 | 助手查证 |
| Claude Code 原生：`claude -p --output-format stream-json` 每轮一条同会话的 `result`；Codex 原生：app-server 的 `turn/completed`，状态 `completed`、`interrupted`、`failed` | [harness-adapters.md 终局清单](../../../docs/research/harness-adapters.md)、[Codex app-server](https://learn.chatgpt.com/docs/app-server) | 库内调研加助手查证 |
| Herdr 到 0.9.3 不支持 ACP、A2A、MCP，接口是它自己的套接字 API；一个含「ACP registry adoption」的功能批量请求 2026-09-24 被关为 not planned，维护者请到 Discussions 提 | [issue #4567](https://github.com/herdrdev/herdr/issues/4567)、[README v0.9.3](https://github.com/herdrdev/herdr/blob/v0.9.3/README.md) | 主笔复核关闭状态 |
| Herdr 0.9.x 有自己的 agent 状态（工作中、等输入、等审批、完成），来源是 harness 的钩子上报或屏幕识别；0.9.2 起钩子上报的状态在热交接后保留，第一件任务也算完成 | [v0.9.2 发布说明](https://github.com/herdrdev/herdr/releases/tag/v0.9.2) | 助手查证加主笔读过发布说明 |

**「这一轮答完了」的三种社区取法**：

1. **ACP 的 `stopReason`。** 最干净，但 ACP 要求客户端把 agent 当子进程、走标准输入输出；这和「Herdr 实际持有 harness 与会话」是两条接入路线，不能同时成立。Herdr 也已经表示不做 ACP。
2. **harness 自己的钩子。** Claude Code 有 Stop 一类的钩子；钩子在 harness 的配置里，Herdr 只负责在 PTY 里拉起它（`harness-hooks-20260903.md` 已调研七个问题）。
3. **Herdr 的 agent 状态。** 它本身就是吃钩子上报做出来的，Herdr 0.9.x 在这块持续投入。

和所有者定的方向（§二 第 18 条：Herdr 实际持有；第 19 条：答完了与做完了分开）相容的是 2 与 3：用 Herdr 社区版已有的 agent 状态，状态来自钩子上报而不是屏幕识别。屏幕识别出来的状态按约束只能作低优先级观测。ACP 留作将来另一条路线的标准：如果有一天要无终端的批量派工，走 ACP 适配器，而不是再自己写一套。

**判断**：第 3c 包按 2、3 做，不引入 ACP；`docs/research/README.md` 里「ACP：L1 的 Harness 接入标准」这一句和现行做法不一致，应改成「结构化接入路线的标准；本机现行路线是 Herdr 持有会话」。研究正文按规矩只追加复核记录，这一句由下一次动研究索引的 PR 改。

## 三、Context Bundle

| 我们的 Bundle 条目 | MCP（2026-07-28 版）与 ACP 的内容块 |
| --- | --- |
| 内联交付（Inline bytes） | MCP `EmbeddedResource`（`TextResourceContents` 或 `BlobResourceContents`）；ACP 的 `resource` 块，需要 `embeddedContext` 能力 |
| 指针交付（Pointer） | MCP 与 ACP 的 `resource_link`（uri、name、mimeType） |
| 召回（Recall） | 没有对应 |
| 来源引用（id、revision、digest）、字节摘要 | 没有：整份 schema 里没有摘要、版本或不可变字段 |
| 必需／可选 | 只有 `annotations.priority`（0 到 1，1 表示「实际上必需」），是提示不是约束 |
| 渲染器、分词器、去敏、预算、权限摘要、保留期、token 计数 | 没有 |

出处：[MCP schema 2026-07-28](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/schema/2026-07-28/schema.json)、[ACP content](https://agentclientprotocol.com/protocol/content)（写明「与 MCP 用同一种 ContentBlock 结构」）。MCP 已捐给 Linux 基金会下的 Agentic AI Foundation。助手查证。

通用的「带摘要的内容描述」有现成格式——OCI 描述符（mediaType、digest、size、annotations）、in-toto 声明（subject 带 digest）——但都不管上下文交付的语义（token、预算、去敏、必需与否）。**没有面向 agent 上下文的、冻结且带来源的社区标准。**

**判断：留 Bundle 作治理对象；交付那一步改用社区的内容块。** Bundle 是控制面冻结、按摘要核对的记录，社区没有对应物。但把条目交进 harness 时，现在是 Agency 把正文拼成一段提示词；能用的时候，内联条目应交成 `resource` 块、指针条目交成 `resource_link`，字段名沿用 `uri`、`mimeType`、`annotations.priority`。这要等 harness 的接入路线提供内容块入口才有意义：PTY 路线（现行）下 Claude Code 吃的是文字，所以不急，记作「将来走结构化接入时的做法」。

## 四、Worker Profile

| 格式 | 字段（节选） | 版本与摘要 |
| --- | --- | --- |
| Claude Code subagent（`.claude/agents/*.md`，Markdown 加 YAML 头） | name、description、tools、disallowedTools、model、permissionMode、maxTurns、skills、mcpServers、hooks、effort、isolation | 无 |
| Codex custom agent（`.codex/agents/*.toml`） | name、description、developer_instructions、model、model_reasoning_effort、sandbox_mode、mcp_servers、skills | 无 |
| Agent Skills（`SKILL.md`，开放标准，本库已在用） | name、description、license、compatibility、allowed-tools | 无 |
| A2A `AgentCard` | name、version、capabilities、skills，可签名 | 有签名，不按摘要标识 |
| AGENTS.md | 自由格式，没有必填字段 | 无 |

出处：[Claude Code sub-agents](https://code.claude.com/docs/en/sub-agents)、[Codex subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents)、[Agent Skills 规范](https://agentskills.io/specification)、[agents.md](https://agents.md/)。跨厂商的 agent 定义标准有 AGNTCY OASF（v1.1.0），采用很少。助手查证。

**判断：留 Worker Profile 作治理记录；落到 harness 时改写成各家原生的 agent 定义。** 社区格式都没有不可变、按摘要标识、上下文上限、必需能力集这些；Profile 是选入时冻结的承诺，要留。但 Agency 拉起 harness 时，把 Profile 渲染成 Claude Code 的 subagent 文件或 Codex 的 agent TOML（模型、权限模式或沙箱模式、技能），比自己拼一串命令行参数更贴社区，harness 升级时也由厂商负责兼容。字段对得上的有：模型、权限（对 permissionMode、sandbox_mode、tools）、技能（对 skills）。

**一处可以借的**：Agency 的 `catalog`（可派的工种）将来要列「本机装了哪些 harness」时，ACP Registry 的条目（id、name、version、distribution、license）是现成的数据源与字段名，不必自己维护一份 harness 清单。

## 五、该自己写的（这一层）

- 端口里的配对与租户、冻结规格、票据、输入租约、结果保管到确认、游标缺口：两份协议都没有，是控制面治理的落点。
- Bundle 的冻结、摘要核对、预算、去敏与证据类不压缩：社区格式只管搬内容。
- Worker Profile 的不可变版本与选入时的冻结：厂商格式都是可变的本地配置。

## 六、建议（按好处与代价排）

| # | 建议 | 好处 | 代价 | 时机 |
| --- | --- | --- | --- | --- |
| 1 | 第 3c 包「这一轮答完了」用 Herdr 的 agent 状态加 harness 钩子，不引入 ACP | 符合所有者定的方向，借 Herdr 社区在这块的持续投入 | 要核 0.9.3 的状态语义并实测（3c 验收第 2 条本来就要求） | 现在，3c 内 |
| 2 | Agency 拉起 harness 时，把 Worker Profile 渲染成各家原生的 agent 定义文件 | 少拼命令行参数；厂商负责兼容 | 文件要放在不进仓库工作树的位置（钩子调研的第 5 问同类）；两家格式各一份 | 第二家 harness 接入时 |
| 3 | 改 `docs/research/README.md` 里 ACP 那一句 | 消除前后不一致 | 一行 | 下一次动研究索引时 |
| 4 | 观测事件的种类与取值借 ACP 的名字 | 少自己起名，以后接 ACP 路线不用翻译 | 动端口格式，要配迁移与用例 | 调整观测格式时 |
| 5 | Bundle 条目交付时用 MCP／ACP 的内容块 | 交付格式与社区一致 | PTY 路线下没有入口，现在做不了 | 走结构化接入时 |

## 七、证据边界

- 主笔复核的五条：ACP v1 `StopReason` 的五个取值与最新稳定发布 v1.10.2；两个适配器的仓库、最新版本、许可；Herdr #4567 的关闭状态与原因；A2A 最新 v1.0.1。
- 其余事实由研究助手 2026-10-05 打开原始出处收集，附了链接，主笔没有逐条复核。
- 没有做任何实测：没有起 ACP 适配器，没有用 Herdr 0.9.3 的 agent 状态跑过真实会话（这是第 3c 包的验收内容，由 Codex 实测）。

# HAPI

> 类别:⑤ 远程操控与会话同步 · 证据编号:E-L1-HAPI<br>
> 状态:证据审计 · 钉定版本与许可见文内「审计基线」;发布后正文不改,只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-l1-hapi"></a>
## E-L1-HAPI · HAPI

### 它是什么(产品形态与投入口径)

面向个人自托管的「编码代理远程分身」:把本机运行的多家 Harness(编码代理 CLI)会话镜像到自有 hub(中枢服务),从 Web/PWA/Telegram Mini App/原生 iOS/Android 远程观察、输入、审批,并支持本机终端与远程端之间无损交接。当前登记 11 种 flavor(适配口味):Claude Code、Codex、Cursor、Grok Build、OpenCode、Kimi、Copilot、Antigravity、Pi、DeepSeek Harness,外加已停用只读保留的 Gemini([modes.ts#L10-L21](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/shared/src/modes.ts#L10-L21))。

拓扑与自托管边界:CLI 侧含常驻 runner 守护进程(远程拉起会话、心跳、自升级,[runner/README.md](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/cli/src/runner/README.md)),经 Socket.IO 连 hub;hub 是 Bun 单二进制(内嵌 SQLite 与 Web 资产),客户端只走 REST+SSE;远程接入自选 tunwg(WireGuard+TLS)公共中继或 Cloudflare Tunnel/Tailscale,中继只转发密文、不存数据,明文只在 hub 本地落盘([why-hapi.md](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/docs/guide/why-hapi.md))。Fork 自 Happy(happy-cli,MIT),重写为去中心化形态。投入口径:2025-12-16 起 1400 个提交、99 名贡献者、近 90 天 623 个提交;发行物 darwin-arm64 压缩包 47.8 MB(解压约 109 MB,与上次复核实测一致)。

### 设计亮点(远程操控与会话同步视角,含代码证据链接)

- **结构化投影为主、终端字节流为辅,两种投影并存**。
  - 结构化通路:本地模式下 CLI 尾随(tail)厂商原生 transcript JSONL,过滤内部事件后镜像为 hub 结构化消息([claudeLocalLauncher.ts#L11-L38](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/cli/src/claude/claudeLocalLauncher.ts#L11-L38));远程端看到的是消息/工具调用/审批卡,不是终端字节。
  - 字节流通路:通用 `AgentPtyManager` 用 Bun terminal 托管 PTY([AgentPtyManager.ts#L52-L70](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/cli/src/agent/AgentPtyManager.ts#L52-L70));输出上行有「无观众不上传、订阅时重放本地屏幕缓冲」的门控([apiSession.ts#L1108-L1126](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/cli/src/api/apiSession.ts#L1108-L1126));hub 侧再留 256 KB 滚动屏幕缓冲、按 alt-screen 转义序列修剪后重放给迟到订阅者([agentTerminalBuffer.ts#L1-L26](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/hub/src/socket/agentTerminalBuffer.ts#L1-L26));远程还能发原始按键与 resize([handlers/terminal.ts#L316-L328](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/hub/src/socket/handlers/terminal.ts#L316-L328))。
  - 但注意:PTY 投影迄今只有 Antigravity 用过,2026-08-16 #1591 改 headless 后,截至钉定 commit 已无现役 flavor 以 PTY 模式运行,基础设施保留待用。
- **本地/远程互斥交接,天然单写者**。
  - 同一会话在 local(原生 CLI 独占真终端)/remote(headless 驱动)两态间轮转([loopBase.ts#L39-L87](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/cli/src/agent/loopBase.ts#L39-L87))。
  - 本地模式收到任何远程消息即杀掉本地原生进程、切入远程模式([BaseLocalLauncher.ts#L93-L97](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/cli/src/modules/common/launcher/BaseLocalLauncher.ts#L93-L97));远程模式在终端按双空格经 `handoff-local` RPC 切回([localHandoff.ts#L17-L29](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/cli/src/agent/localHandoff.ts#L17-L29))。
  - 两态续接同一厂商会话:远程模式直接驱动官方 `claude` 二进制的 `--input-format stream-json` 控制协议(自研 SDK 客户端,[sdk/query.ts#L330-L367](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/cli/src/claude/sdk/query.ts#L330-L367));厂商会话 id 记在会话元数据(claudeSessionId/codexSessionId 等),恢复与分叉全部复用厂商自身 resume/fork 机制。
- **可复现的多端同步协议(有独立客户端契约文档与 golden fixtures)**。
  - hub SQLite 是唯一权威;消息按 `(invokedAt??createdAt, seq)` 复合游标分页、epoch 重置、`local_id` 幂等对账乐观发送([pagination.md](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/docs/api/client-contract/pagination.md)、[store/index.ts#L397-L459](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/hub/src/store/index.ts#L397-L459))。
  - SSE 事件 id `{epoch}:{seq}:{nsTag}` 绑定 hub 进程代次与 namespace,配 256 条/2 MiB 重放环做断线续传([sse.md](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/docs/api/client-contract/sse.md))。
  - 会话元数据与运行状态走带版本号的 CAS(比较并交换)更新([versionedUpdates.ts#L20-L42](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/hub/src/store/versionedUpdates.ts#L20-L42))。
- **输入排队、打断与审批路由**。
  - 远程消息先入库为 `delivery_state='queued'`,可取消、可定时(`scheduled_at`),空闲时消费;进行中回合可经 `steer-queued-message` RPC 中途注入,`abort` 可远程打断([rpcMethods.ts#L1-L52](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/shared/src/rpcMethods.ts#L1-L52))。
  - 审批请求作为 CAS 字段存进会话运行状态的 `requests/completedRequests`([schemas.ts#L195-L206](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/shared/src/schemas.ts#L195-L206));Claude 侧由 PreToolUse hook 与 SDK canCallTool 双入口桥接,应答走 `permission` RPC 回 CLI。
- **注意力提醒与隐私可控的推送**。统一通知通道抽象四类事件:ready(等待输入,带冷却)/审批请求(去抖)/任务通知/会话结束([notificationTypes.ts#L10-L15](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/hub/src/notifications/notificationTypes.ts#L10-L15)),扇出到 Telegram/Web Push/FCM/APNs。iOS 走官方 push relay:hub 先用每设备 AES-256-GCM 密钥加密载荷,relay 无账号、只见密文与 token 元数据、限速且不落盘,设备端通知扩展本地解密改写占位横幅([relay/README.md](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/relay/README.md))。
- **正在长出跨会话协作语义**。MCP peer 工具(list_peers/ping_peer/inspect_peer)让会话间互发消息;2026-08-10 起新增 A2A work-graph 账本(events/event_links 表,非人 principal 必须 `on_behalf_of` 标注负责人,事件型如 work_ad/handoff/handoff_receipt,[workGraph.ts#L1-L23](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/shared/src/workGraph.ts#L1-L23))。

### 审计基线(钉定 commit/版本/许可 + changelog 与演化脉络)

- 钉定 commit:默认分支 main HEAD `980a921ba15665c54998a6ddb658103d467ff4cb`(2026-08-29)。最新 release v0.29.0(2026-08-19,tag 指向 `240ab2f`)。勘误:上次复核记录的 `bc9df82` 实为 2026-08-27 的 main 提交,不是 v0.29.0 tag。
- 许可:仓库 LICENSE 为 GNU AGPL v3 全文,cli 与 hub 的 package.json 均声明 `AGPL-3.0-only`;NOTICE 载明源自 happy-cli(MIT)。npm 包名 `@twsxtd/hapi`。
- v0.29.0 以来 26 个提交,方向:DeepSeek Harness 经 ACP(Agent Client Protocol)接入(#1632)、Cursor 回合中途 Steer(#1609)、会话拉起前 agent 可用性校验与 workspace 目录浏览(e5a8212f)、Android Google Play 合规与签名流水线、iOS 配对 UX、Bun 升 1.4.0。未见对外协议 breaking change(客户端契约声明 protocolVersion=1,能力键只增不改)。
- 更早脉络:v0.28→v0.29 共 126 个提交,引入/完善原生 iOS/Android、push relay、A2A work-graph;节奏约两周一个 minor,活跃贡献者 15+,提交注记显示重度依赖 AI 代理开发。

### 采用与边界(建议复用决策 + 不采用边界 + 对既有观察清单行的确认/修正)

- **复用决策:仅参考行为**。值得参照的行为:
  - 双投影并存,与字节流「无观众不上传、订阅重放屏幕缓冲」的门控;
  - 本地/远程互斥交接(远程输入即接管、双空格交还),作 Terminal 场景单写者交接的交互参照;
  - SSE 代次化事件 id + 重放环 + 复合游标/epoch 对账的多端同步协议(有可执行契约与 golden fixtures,适合当协议设计对照);
  - push relay 的「密文信封 + 无账号 + 元数据最小化」推送形态。
- **不采用边界**:AGPL-3.0-only 与作为依赖引入相斥;引 CLI 即连带 hub、Web 与 11 家 Harness 业务(单二进制约 109 MB,上次复核实测 `runner --help` 峰值约 133 MiB),不适合作 HCTL 运行服务;其审批/输入没有输入租约(TTL/代次/CAS 意义上的租约)概念,信任模型是单用户共享密钥加 namespace 后缀换 4 小时 JWT([auth.md](https://github.com/tiann/hapi/blob/980a921ba15665c54998a6ddb658103d467ff4cb/docs/api/client-contract/auth.md)),治理事实由 CLI 与 hub 双写在会话运行状态里,达不到控制面唯一权威要求;其运行状态信号对 HCTL 只是 L1 观测。
- **对既有观察清单行**:
  - 「原生本地 Agent 与远程端之间的结构化交接」:确认。
  - 「AGPL」:确认,细化为 AGPL-3.0-only。
  - 「不提供精确 PTY」:维持 2026-08-29 的推翻结论并进一步细化——PTY 通路(AgentPtyManager/agent-terminal 事件/双端屏幕缓冲)完整存在,但截至钉定 commit 无现役 flavor 以 PTY 模式运行(唯一用户 Antigravity 已于 #1591 转 headless),字节流是辅投影,结构化才是主通路。
  - 「不是 Task/Workflow 后端」:大体成立,建议改写为「正在长出轻量任务账本(A2A work-graph、team_state),仍非任务/工作流权威」。

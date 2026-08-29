# Paseo

> 类别:⑤ 远程操控与会话同步 · 证据编号:E-L1-PASEO<br>
> 状态:证据审计 · 钉定版本与许可见文内「审计基线」;发布后正文不改,只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-l1-paseo"></a>
## E-L1-PASEO · Paseo

### 它是什么(产品形态与投入口径)

自托管的多 Harness(编码代理 CLI)控制台:一个 Node.js daemon(守护进程)在开发机上持有并管理 Claude Code / Codex / Copilot / OpenCode / Pi 等代理会话与真实 PTY 终端;iOS/Android/Web(Expo)、桌面(Electron)、CLI(Commander)多端经同一 WebSocket 协议观察与操控([docs/architecture.md#L1-L35](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/docs/architecture.md#L1-L35))。远程路径可选零知识加密中继(relay),另有可选 Paseo Hub(云侧编排入口)。

投入口径:单作者主导的独立开源项目,2025-10-13 首提交,累计 5194 提交;近 90 天 1548 提交、约 119 位署名作者,其中 Mohamed Boudra 本人 1201 条;最近提交 2026-08-29,发布节奏极快(8 月内 v0.4.0→v0.7.0-beta.2)。完整平台投入面很宽(语音、插件系统、Git forge 集成、浏览器自动化、定时任务均在内),但 daemon/协议/SDK 三个包可单独辨认。

### 设计亮点(远程操控与会话同步视角,含代码证据链接)

- **拓扑与自托管边界**:客户端直连 daemon,或经中继——daemon 出站拨号中继、客户端在中继会合,无需开端口([relay-transport.ts#L109-L120](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/server/relay-transport.ts#L109-L120))。端到端加密用 Curve25519 密钥交换 + XSalsa20-Poly1305([relay/src/crypto.ts#L1-L15](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/relay/src/crypto.ts#L1-L15)),中继零知识、被攻破也无法伪造命令(SECURITY.md 威胁模型)。配对信任锚是二维码/URL 携带的 daemon 公钥与 serverId(兼作中继会话标识)([connection-offer.ts#L3-L22](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/protocol/src/connection-offer.ts#L3-L22))。生产中继在另一仓库(getpaseo/paseo-relay,Elixir);monorepo 内 Cloudflare 版是遗留代码。直连侧仅有可选共享口令(bcrypt,经 WS 子协议 `paseo.bearer.<token>` 传递),无短期票据概念([auth.ts#L61-L75](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/server/auth.ts#L61-L75))。
- **协议形状与版本纪律**:单 WebSocket,JSON hello 携带 clientId/clientType/protocolVersion/capabilities([messages.ts#L6936-L6958](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/protocol/src/messages.ts#L6936-L6958)),此后是 requestId 相关的 RPC + 类型化事件流(全量 Zod schema)。能力开关集中登记、每条带引入版本与删除期限([client-capabilities.ts#L1-L33](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/protocol/src/client-capabilities.ts#L1-L33));「app 与 daemon 分开发版、任意组合必须互通」的双向兼容契约成文于 [docs/protocol-compatibility.md](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/docs/protocol-compatibility.md)。二进制帧用 1 字节 opcode + 1 字节 slot 复用终端字节流与文件传输([binary-frames/terminal.ts#L10-L24](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/protocol/src/binary-frames/terminal.ts#L10-L24))。
- **事件流与断线恢复**:代理时间线是 epoch+seq 游标的追加序列,tail/before/after 分页,响应显式携带 reset/staleCursor/gap 标志([agent-timeline-store-types.ts#L3-L46](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/server/agent/agent-timeline-store-types.ts#L3-L46));客户端纪律是「live 流管即时、分页 fetch 才是权威」。目录级同步(项目/工作区/代理列表)用每实体单调 seq + daemon generation + 有界墓碑,过期游标直接回全量快照(docs/architecture.md)。跨设备指同一会话靠 serverId+agentId 深链 `paseo:/h/<serverId>/agent/<agentId>`([agent-deep-link.ts#L19-L28](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/protocol/src/agent-deep-link.ts#L19-L28))。
- **输入通路与审批路由**:发消息带客户端 messageId 去重和 activeTurnBehavior=interrupt|steer——运行中的回合可被「转向」而非只能打断([messages.ts#L1356-L1366](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/protocol/src/messages.ts#L1356-L1366)、[agent-sdk-types.ts#L217-L226](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/server/agent/agent-sdk-types.ts#L217-L226))。审批(permission)是跨厂商归一的结构化请求/响应:kind(tool/plan/question/mode)、可选 actions 按钮、allow 可改写输入、deny 可附带打断([messages.ts#L470-L508](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/protocol/src/messages.ts#L470-L508));未决审批进 daemon 权威快照 pendingPermissions,多端一致、谁先答谁生效([messages.ts#L816-L845](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/protocol/src/messages.ts#L816-L845))。
- **观察双通道**:代理会话走结构化消息投影(timeline item:用户/助手消息、推理、工具调用、todo、压缩);终端走字节流 + 服务端 headless xterm(@xterm/headless)维护的快照——订阅可选 live/visible-snapshot/full-snapshot,快照含 grid/scrollback 及逐行软换行标志供客户端按新宽度重排([messages.ts#L5890-L5904](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/protocol/src/messages.ts#L5890-L5904)、[#L2841-L2857](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/protocol/src/messages.ts#L2841-L2857))。终端尺寸有单一所有者:resize 带 claim/update 意图,非所有者的被动 resize 被拒,避免多端抢尺寸([terminal-size-ownership.ts#L11-L38](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/terminal/terminal-size-ownership.ts#L11-L38))。
- **Provider adapter(执行提供方适配器)契约**:AgentClient/AgentSession 双接口约 140 行定义了会话生命周期、事件订阅、streamHistory、审批响应、打断、回退(rewind)等全部原语([agent-sdk-types.ts#L636-L775](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/server/agent/agent-sdk-types.ts#L636-L775))。厂商会话恢复统一为 AgentPersistenceHandle{provider, sessionId, nativeHandle}——Claude 用 Agent SDK 的 resume id、Codex 用 app-server 线程 id([agent-sdk-types.ts#L195-L201](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/server/agent/agent-sdk-types.ts#L195-L201));内置 claude(@anthropic-ai/claude-agent-sdk)/codex/copilot/opencode/pi/omp,cursor、kimi 等经 ACP(Agent Client Protocol)泛化适配([provider-manifest.ts#L199-L271](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/protocol/src/provider-manifest.ts#L199-L271)、[server/package.json#L71-L93](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/package.json#L71-L93))。importSession 可吸收用户在原生 CLI 里已有的会话。
- **注意力与推送路由**:daemon 统一决策——180 秒在场阈值;有在场且可见的客户端正聚焦该代理/终端则完全抑制;有人「在场」则只给最近活跃客户端发应用内提醒;无人在场才走 Expo 推送,error 类事件永不推送([agent-attention-policy.ts#L42-L80](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/server/agent-attention-policy.ts#L42-L80)、[push-service.ts#L24-L60](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/server/push/push-service.ts#L24-L60))。
- **数据模型**:daemon 权威只存代理元数据 JSON(`$PASEO_HOME/agents/<项目>/<id>.json`:配置、状态、persistence 句柄、未决注意力)([agent-storage.ts#L45-L79](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/server/agent/agent-storage.ts#L45-L79)、[config.ts#L621](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/server/config.ts#L621));时间线内存持有,重启后从厂商原生会话 streamHistory() 重建([agent-manager.ts#L676](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/server/agent/agent-manager.ts#L676)、[#L3662](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/packages/server/src/server/agent/agent-manager.ts#L3662))——转录的持久层被有意留在 Harness 自己手里。Hub 侧创建用 executionId 幂等重试,Hub 连接只授予 `hub.execution.*` 权限、不能管理关系本身([docs/hub.md](https://github.com/getpaseo/paseo/blob/aeb240be98ebfc257f663b85ab681e58cb0d102d/docs/hub.md))。

### 审计基线(钉定 commit/版本/许可 + changelog 与演化脉络)

- 钉定:默认分支 main HEAD `aeb240be98ebfc257f663b85ab681e58cb0d102d`(2026-08-29);最新稳定 release v0.6.1(2026-08-25),最新预发布 v0.7.0-beta.2(2026-08-28)。
- 许可:**Apache-2.0**。演化:2026-02-05 MIT → 2026-02-25 AGPL-3.0 → 2026-08-27 经 PR #3944(commit `a8734a972`)改为 Apache-2.0,写入 0.7.0-beta.1 changelog。根 LICENSE 与根 package.json 一致。
- 公开 SDK 发包:`@getpaseo/server`/`client`/`protocol`/`cli`/`relay`/`plugin` 六包已发 npm(审计时线上均为 0.6.1);子包 package.json 未写 license 字段,npm 元数据缺许可声明,采信仓库 LICENSE。
- changelog 脉络(v0.6.1 后 62 提交):0.5.0 加 active-turn steering、实验性本地插件、Hub 引导、`paseo project`/`paseo reload`;0.6.0 重做 Explorer 侧栏;0.7.0-beta.1 换 Apache 许可、插件可从 Git 安装并参与时间线渲染、F-Droid 元数据。协议层未见破坏性变更(受上述兼容契约约束,旧字段只停写不停读)。
- 上次复审钉定的 `463415a`(2026-08-29 00:00)到本次 HEAD 仅 2 个提交(条款页面与一个轮询修复),无实质演化。
- 活跃度:近 90 天 1548 提交、约 119 位署名作者(核心仍是单作者,近 90 天本人 1201 条),最近提交 2026-08-29。

### 采用与边界(建议复用决策 + 不采用边界 + 对既有观察清单行的确认/修正)

- **复用决策:适配协议**。不采用为依赖(完整自托管平台,体量与 HCTL2 控制面职责冲突),但它的协议与 SDK 是 ⑤ 类里最完整的公开参照:时间线 epoch+seq 游标与 staleCursor/gap 语义、结构化跨厂商审批消息、steer/interrupt 二分的输入语义、终端 snapshot+reflow 投影、注意力在场路由,均可作为 HCTL2 终端远程观察/接管与提醒通路的协议设计对照;adapter 契约(AgentPersistenceHandle + streamHistory 重建)是「厂商会话为持久层、自身只存元数据」路线的成熟样本。
- **不采用边界**:
  - 它不拥有任务语义,一切状态是 Harness 会话的投影,审批是 UI 转发而非治理事实——不能当 HCTL2 控制面账本的先例。
  - 没有输入租约:任何已连接客户端都可发消息、写终端字节,唯一的单写者机制只覆盖终端尺寸(claim/update);HCTL2 要求的 TTL/代次/CAS 输入租约在此无对应物。
  - 直连信任模型是「够得着 socket 即拥有 daemon」(Docker 式),远程凭据是长期口令或配对公钥,无短期票据签发;运行状态信号仍只是 L1 观测。
  - 文中 Session/Agent/Workspace/Project 均为 Paseo 内部名词,不进入 HCTL 公开数据结构。
- **对既有结论**:
  - 观察清单行「守护进程、客户端、执行提供方适配器、公开 SDK 和多设备连接」确认;「AGPL」**已过时,修正为 Apache-2.0**(2026-08-27 起)。
  - 2026-08-29 复审的「真实 Node node-pty daemon(node-pty 1.2.0-beta.15 + @xterm/headless)、provider adapters 与公开 SDK、完整自托管平台、mac arm zip 153.6 MB(实测 153,627,390 字节)、Nix/源码构建需 node-gyp 原生插件」逐条确认。
  - 其版本标注「463415a / v0.6.1」不准确:`463415a` 实为 v0.7.0-beta.1 后第 17 个提交,v0.6.1 只是当时最新稳定 release。
  - 「协议/SDK 参考、不作为 Terminal 运行服务」的定位维持(Terminal 运行服务仍是 Herdr)。

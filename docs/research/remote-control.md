# 远程操控与会话同步：Codex Remote Feishu 与观察清单

> 类别：⑤ 远程操控与会话同步 · 证据编号：E-L1-CODEX-REMOTE-FEISHU<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-l1-codex-remote-feishu"></a>
## E-L1-CODEX-REMOTE-FEISHU · Codex Remote Feishu

### L1 专项价值与跨层边界

Codex Remote Feishu 没有权威 Project Room；它最有价值的设计，是把外部系统的工作区/讨论串接入兼容的托管会话，并将接入/脱离、路由冻结、输入排队与调整指令、审批卡片、重启与重连、传输降级和连接代次做成显式执行控制状态机。Feishu 是这个运行环境的远程控制与状态投影客户端，不是 L4 的事实源。

HCTL 只借鉴托管会话接管与恢复的行为和故障矩阵：Feishu Chat 不能映射为 Room，外部系统的工作区/讨论串不能映射为 Project/Task/Run，`command_ack` 不是语义 Receipt，而且该项目也没有精确的 PTY 契约。普通入站消息进入网关本地 FIFO 后即可收到 ACK，不必等待权威持久化提交；未投递消息的重放仍依赖进程内状态，因此不能据此认定持久 Room 桥接已经闭环。

固定发布版为 [`v2.0.0 / b2091ffe`](https://github.com/kxn/codex-remote-feishu/tree/b2091ffee3330a94703b78a8a6b7b1876e667c65)（2026-08-10）。该基线没有 `LICENSE`、`COPYING` 或 `NOTICE`，GitHub API 也未识别仓库许可证；在获得明确授权前，只能**参考行为、吸收思路**，不得移植源码或文档文本。

主要证据：[发布版](https://github.com/kxn/codex-remote-feishu/releases/tag/v2.0.0)、[架构](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/general/architecture.md)、[中继协议](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/general/relay-protocol-spec.md)、[远程界面状态机](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/general/remote-surface-state-machine.md)、[背压、连接世代与降级](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/implemented/relay-backpressure-hardening-design.md)、[请求与审批](https://github.com/kxn/codex-remote-feishu/blob/b2091ffee3330a94703b78a8a6b7b1876e667c65/docs/implemented/feishu-request-approval-design.md)。

<a id="观察清单远程操控与会话同步"></a>
## 观察清单：远程操控与会话同步产品

研究过且有局部价值，但不进入层内主方案；均为 L1 邻近证据。

| 项目 | 只保留的独特证据 | 复用边界 |
| --- | --- | --- |
| [MindFS](https://github.com/a9gent/mindfs) | 仓库本地 Session、外部 Session 导入与同步 | AGPL；只参考协议与行为，Task Board 不定义 L3 |
| [Paseo](https://github.com/getpaseo/paseo) | 守护进程/客户端/执行提供方适配器、公开 SDK、多设备接缝 | AGPL；作为第二阶段架构参考 |
| [HAPI](https://github.com/tiann/hapi) | 原生本地 Agent 与远程端之间的结构化交接 | AGPL；不提供精确 PTY，也不是 Task/Workflow 后端 |
| [Happy](https://github.com/slopus/happy) | 守护进程、端到端加密同步、远程启动、多设备 | MIT；列入第二阶段观察，不作为第一阶段事实源 |
| [Moshi](https://getmoshi.app/docs/introduction) | 移动终端、钩子与注意力提醒、TUI Chat 投影 | 闭源；只参考用户体验和互操作行为 |
| [Remux](https://github.com/h3nock/remux) | 通过 SSH 和 tmux 控制模式精确定位会话/窗口/窗格 | MIT；不引入第二套领域状态 |
| [ServerCC](https://servercc.app/docs/sessions) | 外部接管、厂商会话恢复、移动端控制 | 闭源；作为身份与交接的产品行为证据 |
| [QuickTUI](https://quicktui.ai/) | 自托管 tmux 加移动端或浏览器终端 | 应用闭源；公开仓库只能证明分发方式 |
| [Redock](https://redock.dev/) | 分阶段输入、CJK 与语音、Activity 深链 | 闭源；只参考用户体验 |

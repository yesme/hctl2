# 远程操控与会话同步单案

本目录保存「把本机 Harness 会话远程化/多端化」一族产品的逐对象审计(代码与 changelog 级,逐项钉定 commit/版本与许可)。跨对象归纳见本文件，Codex Remote Feishu 主条目见[单案](./codex-remote-feishu.md#e-l1-codex-remote-feishu)。这一族不拥有任务语义,均为 L1 邻近证据;复用决策与不采用边界见各文件。

单案(2026-08-30 审计):

- [MindFS](./mindfs.md):外部会话导入与「路径/大小/mtime」游标增量同步、双游标断点重放;工具审批全自动放行的反面证据;
- [Paseo](./paseo.md):⑤ 类最完整的公开协议/SDK 参照——时间线 epoch+seq 游标、跨厂商结构化审批、注意力在场路由、adapter 契约;
- [HAPI](./hapi.md):结构化/字节流双投影与「无观众不上传」门控、本地/远程互斥交接;
- [Happy](./happy.md):端到端加密多设备同步的活体参照(密文中继、配对即授钥、CAS 同步);
- [Remux](./remux.md):移动客户端「附着-定位-渲染-重连」的行为形状,tmux 原生 ID 为唯一路由身份;
- [Moshi](./moshi.md):注入前屏幕校验与在场感知提醒;与 Herdr 深度耦合的现成集成用法;
- [ServerCC](./servercc.md):厂商会话精确恢复与外部会话收养;「接管无控制权/交还语义」的空白写实;
- [QuickTUI](./quicktui.md):公开安装验证器 CI 门与能力自描述端点;qscreen 开源会话后端;
- [Redock](./redock.md):分阶段输入暂存区、BYO 语音引擎与 CJK;tmux/Herdr/psmux 可插拔持久层。

全量分类与复用结论见[研究总表](../README.md)。

<a id="观察清单远程操控与会话同步"></a>
## 观察清单：远程操控与会话同步产品

研究过且有局部价值，但不进入层内主方案；均为 L1 邻近证据。

| 项目 | 只保留的独特证据 | 复用边界 |
| --- | --- | --- |
| [MindFS](https://github.com/a9gent/mindfs) | 仓库本地 Session、外部 Session 导入与同步 | AGPL；只参考协议与行为，Task Board 不定义 L3 |
| [Paseo](https://github.com/getpaseo/paseo) | 守护进程、客户端、执行提供方适配器、公开 SDK 和多设备连接 | AGPL；作为第二阶段架构参考 |
| [HAPI](https://github.com/tiann/hapi) | 原生本地 Agent 与远程端之间的结构化交接 | AGPL；不提供精确 PTY，也不是 Task/Workflow 后端 |
| [Happy](https://github.com/slopus/happy) | 守护进程、端到端加密同步、远程启动、多设备 | MIT；列入第二阶段观察，不作为第一阶段事实源 |
| [Moshi](https://getmoshi.app/docs/introduction) | 移动终端、钩子与注意力提醒、TUI Chat 投影 | 闭源；只参考用户体验和互操作行为 |
| [Remux](https://github.com/h3nock/remux) | 通过 SSH 和 tmux 控制模式精确定位会话/窗口/窗格 | MIT；不引入第二套领域状态 |
| [ServerCC](https://servercc.app/docs/sessions) | 外部接管、厂商会话恢复、移动端控制 | 闭源；作为身份与交接的产品行为证据 |
| [QuickTUI](https://quicktui.ai/) | 自托管 tmux 加移动端或浏览器终端 | 应用闭源；公开仓库只能证明分发方式 |
| [Redock](https://redock.dev/) | 分阶段输入、CJK 与语音、Activity 深链 | 闭源；只参考用户体验 |

### 2026-08-29 Agent 运行服务复核

本表原先对 HAPI 的“不给精确 PTY”判断已被当前源码推翻：[`bc9df82`](https://github.com/tiann/hapi/tree/bc9df82dc6e24140a4c76dfd6a86c0e53df9f8d2) 已有通用 `AgentPtyManager`、Bun terminal、远程观察/输入与恢复。它仍不适合作为 HCTL 的运行服务：v0.29.0 macOS arm64 binary 109.1 MB，`runner --help` 峰值约 133 MiB，且会连同整个 hub、Web 和多 Harness 业务一起引入，增加权限管理和升级工作。Paseo、MindFS、Happy、Remux 与闭源观察项也已逐源码或按公开资料重新分类；完整证据与资源占用见 [Agent 运行服务候选复审](../agentd-runtime-candidates-20260829.md)。本复核不删除旧快照，明确覆盖其中关于 HAPI PTY 能力的事实判断。

### 2026-08-30 逐对象补全审计

观察清单九项全部升格为一对象一文件的完整审计（代码与 changelog 级，逐项钉定 commit/版本与许可），入 [`remote-control/`](./README.md) 子目录；上表保留为历史快照，现行结论以各对象文件为准。主要修正：Paseo 许可已于 2026-08-27 由 AGPL 改为 Apache-2.0，复用决策升为**适配协议**；QuickTUI 的公开仓库不止证明分发（安装验证器钉出 server v2 API 面与 E2E 配对协议名），Windows 默认后端为其开源 qscreen，「公开证据仍指向 tmux」不再成立；Moshi 五个接口面的线协议公开成文，且与 Herdr 深度耦合（hooks 是 Herdr 上唯一 prompt 来源）；Happy 默认不依赖 tmux，远程观察是结构化事件投影，端到端加密核实到代码；HAPI 的 PTY 通路完整但已无现役 flavor 使用，另已长出 A2A work-graph 轻量账本；ServerCC 无浏览器端，「接管」为 tmux/herdr 共享 attach、无控制权与交还语义；Redock 的「Activity 深链」降级为未核实，2.0 起有状态通知但机制未公开。其余判断确认。

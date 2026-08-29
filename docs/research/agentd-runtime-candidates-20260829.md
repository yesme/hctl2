# Agent 运行服务候选复审

> 类别：⑥ 外部运行服务与基础设施 · 补充审计<br>
> 状态：2026-08-29 研究快照；不定义 HCTL 领域模型<br>
> 范围：复审 reference 中所有可能减少 `hctl2-agentd` 自行实现代码的候选，包括过去只列名的 cmux、tty7、Pilotty、远程操控产品与产品内 PTY daemon。固定源码、许可、发行物和测试口径见文末。

> **v0.14.1 重解释**：本审计早于 PR #36—#42 的 Agency 定名及其后续裁决。“继续 tmux”是当时方案的选型快照，现已由“直接采用 Herdr 作为 Agent / Terminal 的运行服务”取代。全部源码、发行、测试和资源占用证据继续有效，用于 Herdr 的回归测试、容量对照和将来重新评估；当前验证清单见 [Herdr 运行服务验证记录](./runtime/agency-runtime-validation-20260829.md)。

## 当时结论（已由 Herdr 选型取代）

这轮研究最初得出“tmux 3.7c + `tmuxctl` + 薄的 HCTL policy broker”的结论。v0.14.1 最终改为直接采用 Herdr，因此这段旧结论不再指导实现。它仍解释了 tmux 已经替我们完成哪些通用能力，以及旧 `hctl2-agentd` 代码中哪些部分不应继续保留。

本轮出现了三个过去没有充分研究的真实替代项：

1. **Termio 的独立 `termiod` 是当时最值得重新评估的会话服务。** 它的 PTY、原始字节流、多观察者、单 writer、慢客户端隔离、ring replay、快照、退出墓碑与本机 footprint 都已经成形，测试也很扎实。问题在交付与权限边界：crate 自称 POC、协议仍为 v0.1、没有任何官方 `termiod` release asset；源码构建需要 Zig 与 Ghostty 源码；daemon 还会把控制 socket 注入子进程，并暴露 `send`/`inject`、文件、Git 与 agent hook 功能。不能把这些无条件放到 Harness 可触达的同 uid socket 上。
2. **tty7-server 排第二，cmuxd-remote 排第三。** tty7 的控制/观察分离、退出记录和 live-upgrade handoff 最完整，但稳定版没有 macOS standalone server，创建时也没有显式 env map。cmux 已有四目标稳定二进制，运行也很轻，但当前 PTY 创建仍是 `/bin/sh -c` 单字符串、所有 attachment 都可写，退出事件没有退出码；需要的不是 wrapper 小修，而是明确的上游协议补丁。
3. **Pilotty 是小而完整的终端自动化工具，不是合格的原始观察后端。** 它有四目标官方二进制、精确 argv/cwd、有上限的原始输出保留、快照、退出记录和很低的 RSS；但协议是请求/响应与屏幕等待，没有持续原始输出流、多观察者独立限速、单一写入者或重启后重新采用会话的协议。若要满足 HCTL 的要求，基本等于重写它的会话协议核心。

HAPI、Paseo、Herdr 等“整个二进制只用一个子集”的方案也按用户提出的准则评估过：**多余功能本身不是拒绝理由**。旧排名过度偏向“最窄的 PTY 守护进程”，因此低估了 Herdr 直接提供完整 Terminal 运行服务的价值。Herdr 的 18.97 MB 二进制和约 18–26 MiB 空闲内存可以接受；它当前缺少的输入所有权、输入记录和事件游标等功能，按“HCTL 适配、上游修改、暂不支持”逐项处理，不再作为排除 Herdr 的理由。

## HCTL 真正需要什么

候选不按功能多少评分，而是逐项对照 HCTL 对终端会话的要求：

| 能力 | 第一阶段要求 | 可以交给上游 | 必须留在 HCTL |
| --- | --- | --- | --- |
| 创建 | 摘要校验后，以数组 argv、cwd、显式 env 和终端尺寸启动 | PTY、进程组、resize、退出码 | Context/命令/environment digest；敏感凭证剥离；目标代次 |
| 连续性 | agentd 客户端断线或 broker 重启不杀 Harness；能按精确物理 ID 回读 | daemon/session 持有、稳定 ID、退出墓碑 | owner/runtime/control/site/backend generation；采用与孤儿判定 |
| 观察 | 原始字节、多观察者、每个观察者独立有界；截断必须显式 gap | raw ring、fanout、slow-consumer 处理 | 对外游标、HCTL 事件 envelope、权限与审计边界 |
| 输入 | 同时只有一个有效 writer | 底层单 writer/排他原语可作加固 | TTL、generation、CAS、过期拒绝、secure input 不入 trace/replay/env |
| 清理 | 只有物理身份和所有权证明都匹配才 kill | kill/process-group 清理 | proof、幂等、失联/孤儿/采用政策 |
| 安全 | owner-only 本地 transport；Harness 不能得到 control/integration credentials | 私有 socket/token、协议上限 | capability 收口；不让直接后端入口绕过 HCTL writer gate |

输入租约、五层代次、digest 和采用证明属于 HCTL，不应该要求上游终端项目替我们定义。外部运行服务应负责通用终端和 Harness 管理，HCTL adapter 负责把自身的治理要求翻译成外部 API 调用并记录结果。

2026-08-29 工作树中曾有一份约 2.8k 行的第一方实验实现，其中 tmux adapter、观察 journal 与 daemon 的 observe/fanout 部分都是 Herdr 已经提供的通用能力，不应继续扩写。这份实验及其 agentd 专用协议已在源码改用 Herdr 时删除；验证、代次、租约、采用/清理与状态持久化仍须按 v0.14.1 分别进入 control、tool 和 Herdr adapter，不能因为删除旧代码就当作这些 HCTL 责任已经完成。

## 统一 footprint 口径

本机数值在 Apple Silicon macOS 上测得，RSS 不含被托管的 `/bin/sleep` 子进程。发行物列实际文件字节的十进制 MB；RSS 用 MiB。不同拓扑不能混算：tmux 同时给出“一个 server 承载十会话”的公平值和 HCTL 当前“每 runtime 一个 server”的隔离值；其他 daemon 默认一个进程承载十会话。`—` 表示没有可比实测，不用猜值补表。

| 候选 | 可用发行物 / 实际文件 | 空载 RSS | 10 会话 RSS | 构建与分发备注 |
| --- | ---: | ---: | ---: | --- |
| **tmux 3.7c** | 官方 macOS arm64 archive 0.65 MB；binary 1.69 MB | — | **3.7 MiB**（共享一个 server）；当前隔离拓扑约 **37 MiB**（十个 server） | 四目标官方单二进制、仅系统 dylib；无需自主构建 |
| **termiod `376a944`** | 无官方 asset；本机 release binary **3.88 MB** | **7.7 MiB** | **11.1 MiB** | 冷构建的 `target` 647 MiB；Zig 解压 408 MiB、下载 52 MB；还会取得 Ghostty 源码 |
| **tty7-server `a2b5ae5`** | nightly macOS arm64 **7.85 MB** | **6.5 MiB** | **10.9 MiB** | 稳定版 standalone server 只发 Linux；macOS server 仅 nightly，完整 app zip 23.6 MB |
| **cmuxd-remote `84098ae`** | v0.64.22 四目标稳定制品；macOS arm64 **6.14 MB**；本机 current nightly 6.18 MB | **9.9 MiB** | **14.8 MiB** | 仅系统库；稳定版有 checksum，但当前 daemon 又比稳定版增加约 1,451 行可靠性/日志改动 |
| **Pilotty v0.0.11** | 官方四目标；macOS arm64 archive 1.89 MB、binary **5.37 MB** | **7.6 MiB** | **9.7 MiB** | 仅系统 dylib；本机源码 release binary 5.39 MB |
| **shpool v0.11.4** | macOS 仅 arm64 archive 1.60 MB；先前实测 binary 约 **4.04 MB** | — | **23.1 MiB**（v0.11.2） | 没有 macOS x86_64 asset；可靠 restore 模式在输出后内存会明显增长 |
| **Herdr v0.8.2** | macOS arm64 **18.97 MB**、x86_64 20.55 MB | **20.4 MiB** | — | 四目标原生 binary；同时携带完整终端产品、agent 检测、插件、worktree 等面 |
| **Zellij v0.45.x no-web** | macOS arm64 archive 11.90 MB；v0.45.0 binary **32.4 MB** | 单 detached session **89.7 MiB** | **841.6 MiB** | v0.45.1 发行尺寸复核；RSS 为 v0.45.0 同口径实测，结论不受小版本改变 |
| **WezTerm mux** | 无 standalone mux-server asset；完整 macOS zip **102.97 MB** | — | — | mux 与配置、Lua、VT、GUI crate 图紧耦合；最新稳定发行仍是 2024-02-03 |

Termio 的关键差异值得单独强调：它不是“大 binary”。运行 binary 和 RSS 都很好，真正重的是**供应链与构建缓存**。如果上游发布四目标 standalone asset，当前最主要的运维反对理由会立刻消失。

## 当时排名较高的候选：功能差距与所需修改

### Termio / `termiod`

固定 [`v0.46.0 / 376a944`](https://github.com/termio-sh/termio/tree/376a944d47b4019a2e5d12243881f0c046549c37)（MIT）。独立 crate 的 package metadata 仍写着 “durable PTY session host (POC)” 和协议 v0.1；[framed protocol](https://github.com/termio-sh/termio/blob/376a944d47b4019a2e5d12243881f0c046549c37/termiod/src/protocol.rs)保留原始字节，并提供 `CreateSpec { argv, cwd, env }`、interact/observe、writer claim、snapshot/history/grid、退出事件；[session actor](https://github.com/termio-sh/termio/blob/376a944d47b4019a2e5d12243881f0c046549c37/termiod/src/session.rs)实现 128 KiB raw ring、每客户端 4 MiB backlog、慢客户端 resync/drop，VT sidecar 自己还有 16 MiB 上限，过载时降级而不堵 PTY。墓碑跨 daemon 重启保留，但 daemon 自己死亡时原进程不能被重新接管；这一点与 tmux server 自身死亡同类。

本机 release build 成功，209 个 unit/integration test 全部通过，包括 mid-flood attach、graceful shutdown、stdio/WSS byte identity、backlog、replay gap、墓碑和资源订阅。它是本轮源码与故障测试最完整的候选。

不能直接采用的点：

- `send`/内部 `inject` 可以绕过 attachment writer token；HCTL 不能把底层 writer token误当 Terminal Input Lease。
- daemon 会在 client env 之后强制加入 `TERMIOD_SESSION_ID` 与 `TERMIOD_SOCK`。这正适合 Termio hook，却把同 uid 控制入口交给 Harness；此外文件、上传、Git、agent hook 能力远超 HCTL 的终端会话需求。额外功能占用小可以接受，额外**写权限**不能默认接受。
- CLI 的普通 create 路径没有完整暴露 env；HCTL 需要直接使用 protocol/client，或补 CLI。协议源码在 binary crate 内，不是稳定发布的独立 SDK。
- 没有任何 release asset；`libghostty-vt-sys` 构建需要 Zig 0.16 与 Ghostty source。把它纳入 Buck 可以做到可复现，但会重新引入刚从 tmux 分发删除的自主工具链维护。

当时的重新评估条件是：上游发布四目标 standalone asset 与 checksum；稳定 client/protocol crate；提供不向子进程注入 socket、可关闭 `send`/`inject` 及非 PTY capabilities 的 host mode。这些条件现在只用于将来重新考虑 Termio，不是当前实现计划。

### tty7-server

固定 [`a2b5ae5`](https://github.com/l0ng-ai/tty7/tree/a2b5ae56d920fa8fadb05b7e9141cb9fe8e23a48)（Apache-2.0）。[daemon protocol](https://github.com/l0ng-ai/tty7/blob/a2b5ae56d920fa8fadb05b7e9141cb9fe8e23a48/crates/tty7-core/src/daemon/protocol.rs)、[pane owner](https://github.com/l0ng-ai/tty7/blob/a2b5ae56d920fa8fadb05b7e9141cb9fe8e23a48/crates/tty7-core/src/daemon/pane.rs)和 [handoff](https://github.com/l0ng-ai/tty7/blob/a2b5ae56d920fa8fadb05b7e9141cb9fe8e23a48/crates/tty7-core/src/daemon/handoff.rs)支持数组 argv/cwd、稳定 pane ID、一个 controller 与显式 observe、多读者、8 MiB observer budget、8 MiB replay ring、退出码和 Unix live binary upgrade 的 PTY FD 继承。普通 crash 后所谓 cold restore 是启动新 shell 并显示说明，不是保留原进程，文档中应继续使用“重建”而非“恢复”。

缺口是 spawn 没有逐会话 env map，`SendInput` 控制请求又能绕开 controller；两项都可以小范围补丁，但必须进入上游或维护 fork。v26.8.3 稳定版只发布 Linux standalone server；macOS arm64/x86_64 server 只在 nightly，稳定 macOS zip 中也没有隐藏的 server binary。发布门尚未过，因此暂列第二。

### cmuxd-remote

固定 current [`84098ae`](https://github.com/manaflow-ai/cmux/tree/84098ae5b2d20a7a5e3ec85b691bc55969e319b4/daemon/remote)（GPL-3.0-or-later），并核对稳定 [`v0.64.22 / ddd4a01`](https://github.com/manaflow-ai/cmux/tree/ddd4a01bc5d8ebac19643930f5fd7d40e85f1534/daemon/remote)。稳定版已发布 macOS/Linux × arm64/x86_64 的 `cmuxd-remote` 和 checksum；这修正了旧观察清单把 cmux 只当完整桌面 app 的印象。

[PTY hub](https://github.com/manaflow-ai/cmux/blob/84098ae5b2d20a7a5e3ec85b691bc55969e319b4/daemon/remote/cmd/cmuxd-remote/ws_pty.go)有命名 session、多 attachment、1 MiB raw scrollback、每 attachment 256-frame queue、写超时与慢 attachment 丢弃、input sequence ACK、min-size resize；[persistent daemon](https://github.com/manaflow-ai/cmux/blob/84098ae5b2d20a7a5e3ec85b691bc55969e319b4/daemon/remote/cmd/cmuxd-remote/main.go)使用 owner-only Unix socket、每 slot token、断线重连、list/close/status 和生命周期日志。本机 `go test ./...` 通过；current nightly 的 stdio daemon 10 会话实测 14.8 MiB RSS。

它需要的修改仍比 tty7 多：创建参数只有 shell command 字符串并执行 `/bin/sh -c`，没有独立 argv/cwd/env；所有 attachment 都可以 `pty.write`，没有 observe-only 或 writer claim；`pty.exit` 没有退出码；ring 只报告 replay byte 数，没有可续接 cursor/gap。稳定版已有核心 persistent daemon，但 current 又比 v0.64.22 多 1,451 行 daemon 日志、进程输出与生命周期 hardening；选稳定版和选最新行为目前不是同一件事。

### Pilotty

固定 [`v0.0.11 / c53722b`](https://github.com/msmps/pilotty/tree/c53722b3ac7ed3f44b76cf85b1da0714efce875d)（Cargo metadata 声明 MIT；仓库没有单独 LICENSE 文件）。官方 release 已覆盖 HCTL 四目标。其 [daemon](https://github.com/msmps/pilotty/tree/c53722b3ac7ed3f44b76cf85b1da0714efce875d/crates/pilotty-cli/src/daemon)支持数组 argv/cwd、bounded raw retention 及精确 dropped/total accounting、VT screen snapshot、等待变化、resize/kill/status 和 10 分钟退出墓碑；188 个 unit test 与 1 个 doc test 通过。

它的“observer”是 server 内部 screen watcher，外部协议仍是一次一答的 snapshot/output/wait；没有持续 raw subscription、每 observer backlog、写者身份、逐会话 env 或 daemon 重启采用。HCTL 若把 output 轮询包装成 stream，会重新制造延迟、重复传输和 gap 边界；若直接补协议，则已进入 fork 的核心。保留为自动化探针/测试工具候选，不作为 Agent 运行服务。

### 历史方案：tmux + `tmuxctl`

tmux 的完整 P0 与 footprint 见[运行时后端复审](./tmux-runtime.md#e-l1-tmux-runtime)。本轮额外核对两个 Rust control-mode 库：

| 库 | 源码结论 | 决定 |
| --- | --- | --- |
| [`ace-rs/tmuxctl` `c43b793`](https://github.com/ace-rs/tmuxctl/tree/c43b7930cbbc8ec8f38bccdd80f8647e17f1dd08)（MIT OR Apache-2.0） | 约 2.9k Rust 行；按 bytes 解析、reply 关联、unknown notification 容忍、layout parser，blocking/Tokio/smol 三种 transport；57 个 unit/transcript test 通过 | 历史 agentd 已直接依赖它；改用 Herdr 后不再进入正式产品 |
| [`tmux-cmc` `f256230`](https://github.com/ArcavenAE/tmux-cmc/tree/f2562306469db18bd9b7dcce4423817efc949538)（MIT） | 约 1.8k 行、测试通过，但基于 `BufRead::lines()` 与 `String`，不能无损保留任意非 UTF-8 PTY bytes；命令和重连覆盖也较窄 | 不切换；代码更少不是优点，原始字节合同不成立是硬缺口 |

tmux 本身不提供 HCTL 所需的输入租约或观察 API，所以历史方案还需要 HCTL 自己补上这些功能。它的发行与故障行为很成熟，资源占用也很低，因此相关测试仍作为 Herdr 的对照；它不再是产品实现计划。

## 其他候选与完整产品

| 项目 | 源码审计后的真实形态 | 发行 / footprint | 决定 |
| --- | --- | --- | --- |
| **shpool v0.11.4** | daemon 持有 PTY，但一次仍只允许一个 attach client；事件无足够 payload，多客户端/旁观/慢客户端修复没有形成发布合同；可靠 vt100 restore 在有输出时内存昂贵 | arm64 mac archive 1.60 MB；十会话 23.1 MiB，输出后的 restore 可远高于此；无 mac x86_64 asset | 不采用；会迫使 HCTL 自写终端模拟、多观察者转发和慢客户端处理，正是要避免的重复工作 |
| **Herdr `c2637dc` / v0.8.2** | daemon 持有 PTY，支持 observe/controller takeover、事件和 agent 状态判定；同时提供完整 TUI/server/API/plugins/worktree，live handoff 仍实验性 | mac arm binary 18.97 MB；空 headless 约 18–20 MiB，10 个空闲 workspace 约 26 MiB | **已选定**；多余功能和资源占用可以接受，缺少的输入权、记录和事件游标等功能逐项适配、修改上游或明确为暂不支持 |
| **Zellij v0.45.1** | 完整 terminal workspace 与 WASM plugin runtime；不是窄 session host | no-web archive 11.90 MB、binary 约 32.4 MB；十 session 841.6 MiB | 排除；多 Harness 日常使用时的内存远高于其他候选 |
| **WezTerm `wezterm-mux-server`** | 有真实 mux server、版本化 PDU、spawn/write/kill/get-lines/render-changes；但与 WezTerm config/Lua/VT crate 图紧耦合 | 无 standalone server；完整 mac zip 102.97 MB；最新 stable 为 2024-02-03 | 机制参考；拆包维护成本高于收益 |
| **Superset `@superset/pty-daemon` `f528adf`** | 约 7.6k TS 行，精确 argv/cwd/env、64 KiB ring、订阅、退出、Node PTY FD live-upgrade；确实是好机制，不是 README 幻觉 | ELv2；无 standalone asset；依赖 Node/Bun + `node-pty`；完整 arm mac zip 579.4 MB | 只参考机制；许可和 native addon 分发都不适合移植/采用 |
| **Stably Orca `orcad` `4461108`** | 当前已有 plain-Node headless `orcad`，但它打包整个 Orca store/runtime/RPC/browser helper，并外置 patched `node-pty`、watcher | 无 standalone orcad asset；完整 arm mac zip 206.1 MB | 修正旧结论“没有 daemon”，但仍不是轻量 PTY binary；不采用 |
| **HAPI `bc9df82` / v0.29.0** | 当前已有通用 `AgentPtyManager` 和 Bun terminal，能托管多家 Harness、远程观察/输入/恢复；不是“没有精确 PTY” | 四/五目标 Bun executable；mac arm archive 47.8 MB、实际 binary 109.1 MB；仅 `runner --help` 峰值约 133 MiB | 可以只用子集，但这个子集捆着 hub、Web 和多 Harness 业务，体积大且与 HCTL 的职责重复，不划算 |
| **Paseo `463415a` / v0.6.1** | 真实 Node `node-pty` daemon、provider adapters 与公开 SDK，同时是完整自托管 Agent 平台 | mac arm zip 153.6 MB；Nix/source 构建需 node-gyp/native addon | 协议/SDK 参考，不作为 Terminal 运行服务 |

## 不能直接管理 PTY 会话的项目

下面这些也读了当前源码，但它们解决的是别的问题；把它们当成 Agent 运行服务只会让 HCTL 多包一层，不会减少代码：

| 项目 | 源码里实际拥有的东西 | 对 HCTL 的角色 |
| --- | --- | --- |
| **MindFS `8f5bd3e`** | Go server、SQLite/task/sync、Codex/Claude SDK 与 ACP session pool；`creack/pty` 主要用于命令 shell，Agent 主会话不是任意 TUI raw PTY | 可参考结构化 provider/session pool；不能管理 Terminal 的 PTY 会话 |
| **Happy `b824cd0`** | Node CLI/daemon 与远程同步；本地进程路径优先借 tmux，否则普通 child spawn | 远程控制与 Harness adapter；底层仍需要 tmux/PTY host |
| **Remux `bc39fb6`** | iOS Swift client，通过 SSH 驱动既有 tmux/control mode | 客户端与精确定位参考，不是 server |
| **Codeg / First Tree / Multica** | managed runtime、provider/SDK、worktree 和上层监督，各自没有可独立采用的窄 PTY host；Multica 还有自定义许可 | 继续保留既有行为/协议证据，不作为可直接采用的依赖 |
| **claude-squad / crystal / Gas Town** | 人工并行/worktree 编排，通常直接用 tmux 管理终端会话 | 说明 tmux 在行业中的常见用法，不提供替代服务 |
| **container-use / vibe-kanban** | 容器生命周期、executor/harness adapter、日志归一化或看板 | 可参考 adapter 和生命周期测试；不提供持久 PTY 会话服务 |
| **Moshi / ServerCC / QuickTUI / Redock** | 公开资料只能验证移动/浏览器控制行为；核心 server 闭源。QuickTUI 的公开证据仍指向 tmux | 行为证据；不能满足“看代码后采用”的准入条件 |

## v0.14.1 后怎样使用这些证据

当前实现纪律如下：

- 直接采用 Herdr，不再自写 PTY、terminal emulator、session multiplexer、终端观察转发或 tmux control-mode parser。
- 旧 agentd 代码只保留 HCTL 独有的治理与验证逻辑，并按新职责迁移；tmux 专用代码不进入正式实现。
- 本文对 Termio、tty7、cmux、Pilotty、tmux 和其他项目的测试用于设计 Herdr 回归用例、估算资源和将来重新评估，不启动并行实现。
- Herdr 缺少的通用运行服务能力优先推动上游补充；不能补的能力在产品中明确标为暂不支持，不在 HCTL 里重新写一套。

这样既保留 HCTL 自己必须拥有的治理规则，也不因为已经写过 agentd/tmux 代码而拒绝一个更完整、资源占用可以接受的上游产品。

## 审计基线与复现记录

固定源码：Termio [`376a944`](https://github.com/termio-sh/termio/tree/376a944d47b4019a2e5d12243881f0c046549c37)、cmux [`84098ae`](https://github.com/manaflow-ai/cmux/tree/84098ae5b2d20a7a5e3ec85b691bc55969e319b4)、tty7 [`a2b5ae5`](https://github.com/l0ng-ai/tty7/tree/a2b5ae56d920fa8fadb05b7e9141cb9fe8e23a48)、Pilotty [`c53722b`](https://github.com/msmps/pilotty/tree/c53722b3ac7ed3f44b76cf85b1da0714efce875d)、Herdr [`c2637dc`](https://github.com/herdrdev/herdr/tree/c2637dc182ddc5425108824d5ed15d24ce38c4e3)、shpool [`3a1020c`](https://github.com/shell-pool/shpool/tree/3a1020c74d1ac8eb191270922d834aa452cac816)、WezTerm [`27d55be`](https://github.com/wez/wezterm/tree/27d55bef144f34a73e23585302838a36ec3aa30e)、Superset [`f528adf`](https://github.com/superset-sh/superset/tree/f528adf8169ec015e050b4eb6d554de4c8be42ef)、Orca [`4461108`](https://github.com/stablyai/orca/tree/446110810c63687c73274920f60d279526d41f36)、MindFS [`8f5bd3e`](https://github.com/a9gent/mindfs/tree/8f5bd3e8090e9a638a41a550d2b0e633003da7e9)、Paseo [`463415a`](https://github.com/getpaseo/paseo/tree/463415ae846cbcfef0df691e413a1a73a9213757)、HAPI [`bc9df82`](https://github.com/tiann/hapi/tree/bc9df82dc6e24140a4c76dfd6a86c0e53df9f8d2)、Happy [`b824cd0`](https://github.com/slopus/happy/tree/b824cd0a4681d41af631a8e422a813873e4455b0)、Remux [`bc39fb6`](https://github.com/h3nock/remux/tree/bc39fb6d713f9679de43f92674cff03a633db0f1)。

本机执行记录：`termiod` release build 与 209 tests 通过；Pilotty release build 与 189 tests 通过；cmux `go test ./...` 通过；`tmuxctl` 与 `tmux-cmc` 各自测试通过。release asset 名称和原始 byte size 通过 GitHub release API 核对；RSS 在隔离临时 socket/state 目录启动后由 `ps` 读取，测试会话均已显式关闭。源码行数只用于估计 patch surface，不作为质量评分：termiod 约 24.6k、tty7 server/core relevant path 约 43.5k、cmux remote 约 25.7k、Pilotty 约 10.8k、Herdr 约 230k Rust 行；产品内 monorepo 更大，不拿总行数直接比较能力。

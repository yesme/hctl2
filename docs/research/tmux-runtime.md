# 运行时后端：tmux 与候选对照

> 类别：⑥ 机械后端与基础设施 · 证据编号：E-L1-TMUX-RUNTIME<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-l1-tmux-runtime"></a>
## E-L1-TMUX-RUNTIME · 运行时后端复审

### 当前决定与接入边界

第一阶段采用 [`tmux 3.7c / e476c123`](https://github.com/tmux/tmux/tree/e476c1230b958df0cb12977517d24b3dc931375b) 作为源码审阅基线。它提供 agentd 真正需要的窄接口：control mode 以命令和 `%output` 等通知驱动；客户端可设 `read-only`、`ignore-size` 和 `pause-after`；server 对 DA、DSR、DECRQM 等无人值守终端查询有明确应答；control output 使用有界缓冲、非阻塞写和 pause/continue。对应实现见 [`tmux.1` 的客户端标志](https://github.com/tmux/tmux/blob/3.7c/tmux.1#L1080-L1145)、[control mode 协议](https://github.com/tmux/tmux/blob/3.7c/tmux.1#L8113-L8235)、[`input.c` 查询应答](https://github.com/tmux/tmux/blob/3.7c/input.c#L1557-L1707)，以及 [`control.c` 的缓冲水位](https://github.com/tmux/tmux/blob/3.7c/control.c#L130-L138)和[非阻塞输出处理](https://github.com/tmux/tmux/blob/3.7c/control.c#L732-L804)。本机 detached 探针中，子进程发送 DSR `ESC [ 5 n` 后收到 `ESC [ 0 n`；同一探针在 shpool `v0.11.2` 超时。

产品形态不是把 tmux 暴露成第二套 API。agentd 默认给每个 runtime 建 owner-only socket/server，以 control mode 持有唯一可写客户端，记住 session/window/pane ID、进程退出状态和 runtime generation；一个 runtime 只建一个 session/window/pane，并启用 `remain-on-exit`。Workbench、CLI 和浏览器观察者由 agentd 扇出、限速与重放，不能按名称猜 pane，也不能直连取得输入权；裸 `tmux attach-session` 即便只读仍绕过 descriptor，因而只作明确标记的 break-glass。共享一个 server 承载多个 runtime 只有在故障域和背压探针通过后才可作为优化。

### 候选源码与 footprint 对照

测量均在 Apple Silicon macOS 上完成；进程数值不含各 session 的 harness/`sleep` 子进程。文件大小是实际字节换算的 MiB，不把“压缩包小”冒充运行时占用。

| 候选 | 源码审阅结论 | 本机 footprint | 决定 |
| --- | --- | --- | --- |
| **tmux 3.7c** | 公开 control mode、多客户端、稳定 pane ID、`capture-pane`/`pipe-pane`、退出状态和 headless 查询应答；慢 control client 有 pause/有界缓冲接缝 | Homebrew macOS arm64 bottle **0.52 MiB**，executable **0.95 MiB**，直接非系统 dylib **1.45 MiB**；一个 server 承载 10 个 detached session 时 **3.7 MiB RSS**。若默认每 runtime 一 server，十个约 **37 MiB RSS** | **采用**；HCTL 分发只带审阅过的最小 dylib/terminfo/许可集合，不把约 61 MiB 的完整 Homebrew dependency 目录原样打包 |
| **Zellij v0.45.0** | 原生跨平台与结构化插件能力较强，但[默认 layout](https://github.com/zellij-org/zellij/blob/v0.45.0/zellij-utils/assets/layouts/default.kdl)启动 tab/status WASM，二进制还[嵌入插件资产](https://github.com/zellij-org/zellij/blob/v0.45.0/zellij-server/src/plugins/plugin_loader.rs#L480-L490) | 官方 macOS arm64 `zellij-no-web` archive **11.3 MiB**、binary **32.4 MiB**；一个默认 detached session **89.7 MiB RSS**，10 个合计 **841.6 MiB RSS**（physical footprint 约 583 MiB）；无插件单 session 仍约 41.2 MiB physical footprint | **不采用**；多 Harness 常态下每 session 的 server/plugin 成本过高，其 web/插件面也不是第一阶段所需 |
| **shpool v0.11.2** | 轻量 daemon + attach 路径清楚，但当前合同[一次只允许一个客户端](https://github.com/shell-pool/shpool/blob/v0.11.2/README.md#L365-L370)，[事件只有类型而无 payload](https://github.com/shell-pool/shpool/blob/v0.11.2/libshpool/src/daemon/events.rs#L11-L18)；多客户端 [`#40`](https://github.com/shell-pool/shpool/issues/40)、快照/旁观 [`#363`](https://github.com/shell-pool/shpool/issues/363)及慢客户端阻塞修复 [`#399`](https://github.com/shell-pool/shpool/pull/399)仍未形成已发布合同 | 官方 macOS binary **4.04 MiB**，仅链接系统库；一个 daemon 承载 10 个空闲 detached session 时 **23.1 MiB RSS**。10 × 约 200 KiB 输出后，默认 vt100 restore 的 physical footprint 约 **252.3 MiB**，实验 vterm 约 63.7 MiB，simple 约 5.3 MiB但放弃可靠 replay | **不采用**；agentd 若补齐终端模拟/查询应答、多观察者扇出、背压、快照和 replay，已重新承担运行时最难的一半 |

### 使用期兼容性风险（不属于 P0）

tmux 支持 `extended-keys` 的 CSI-u/modifyOtherKeys 形态，但[没有完整 Kitty keyboard protocol](https://github.com/tmux/tmux/issues/5406)；必须通过能力探测降级，不能在 manifest 中虚报。`3.7c` 已包含 OpenCode palette 修复（[`#4793`](https://github.com/tmux/tmux/issues/4793)），`3.7b` 已修复 Codex/Grok 低对比度（[`#5312`](https://github.com/tmux/tmux/issues/5312)）和 Claude synchronized-output 回归（[`#5340`](https://github.com/tmux/tmux/issues/5340)），但这些历史修复不等于任意 Harness TUI 的兼容性保证；OpenCode 仍有启动期查询/粘贴（[`#42915`](https://github.com/anomalyco/opencode/issues/42915)）和 passthrough 应答泄漏（[`#40035`](https://github.com/anomalyco/opencode/issues/40035)），Codex 有增强键位问题（[`#34717`](https://github.com/openai/codex/issues/34717)），Kimi `/copy` 有 OSC 52 问题（[`#3173`](https://github.com/MoonshotAI/kimi-code/issues/3173)）。

这些问题是使用期的能力降级、升级或换家触发条件，不是 HCTL 与 tmux 接缝的 P0。各 Harness adapter 只需验证自己声明的规范化 Terminal 能力，不维护随供应商版本漂移的六家 TUI 矩阵。第一阶段固定每个 runtime 一个 server，不引入共享 server 的跨 runtime 故障域；[`tmux #5510`](https://github.com/tmux/tmux/issues/5510) 因而不是当前拓扑的阻断项。若以后重开共享 server 优化，才须把其多窗格、快速滚动、copy-mode 与 resize 组合纳入该优化的准入测试。

### 2026-08-28 分发复核

[`tmux/tmux-builds v3.7c`](https://github.com/tmux/tmux-builds/releases/tag/v3.7c) 已提供 HCTL2 三个目标所需的官方预编译包，因此分发方案从 Homebrew 探针和自主源码构建改为直接消费这些摘要锁定的单二进制。macOS arm64 归档为 **647,750 bytes（0.62 MiB）**、解压后二进制 **1,694,408 bytes（1.62 MiB）**；macOS x86_64 为 **676,533 bytes（0.65 MiB）**和 **1,735,464 bytes（1.66 MiB）**。两个 Mach-O 都只链接 `/usr/lib` 系统库，并声明 `minos 15.0`；Linux x86_64 制品是静态 ELF。HCTL2 因而将 macOS 最低基线升为 15，删除 tmux、libevent、ncurses、utf8proc 的自主构建和 Homebrew bison/pkgconf 前置，只保留运行归档、官方 `LICENSES.tar.gz` 与上游源码三类独立摘要。上表的 Homebrew 数字和 RSS 仍是 2026-08-23 的选型测量，不再描述当前分发形态。

### 2026-08-29 P0 接缝验证

P0 已通过，P1 的 `hctl2-agentd` 实现不再被 tmux 选型假设阻塞。可丢弃探针 [`p0_control_mode.py`](../../src/testing/tmux/p0_control_mode.py) 和 [`p0_fixture.py`](../../src/testing/tmux/p0_fixture.py) 只使用临时目录与 Python 标准库，直接测试 Buck 按摘要组装出的 `tmux 3.7c` 分发二进制，不进入产品生命周期，也不建立长期的 Harness 兼容矩阵。[三目标远端复核](https://github.com/yesme/hctl2/actions/runs/33229161287)连同既有依赖生命周期与精确缓存断言全部通过：

| 实际目标 | 用时 | control output | 慢观察者有界队列丢弃 |
| --- | ---: | ---: | ---: |
| Linux x86_64 | 1.442 s | 1,416,999 bytes | 2,115 chunks |
| macOS arm64 | 1.980 s | 1,416,999 bytes | 1,677 chunks |
| macOS x86_64 | 3.667 s | 1,416,999 bytes | 2,040 chunks |

三者都验证了临时根目录 `0700`、socket `0600`、恰好一个非只读 control client；输入与 `111 × 37` resize 可下发；快速观察者完整收到 12,000 条 burst，慢观察者队列满时只丢自己的 chunk，压力后 server 与输入仍可用；control client 断开重连前后 session/window/pane ID 与 pane PID 稳定；进程以 `17` 退出后无残留，`remain-on-exit` pane 仍可读回同一 ID 和退出码。没有 attach 的 fixture 也分别收到 DSR `ESC[0n`、DA `ESC[?1;2c` 与 DECRQM `ESC[?2004;2$y`。同一 macOS arm64 分发二进制本机连续运行十次也全部通过。这里证明的是 agentd 所需 control-mode 接缝可实现，不宣称 tmux 已替 agentd 完成租约、持久化或任意 Harness 的 TUI 兼容。

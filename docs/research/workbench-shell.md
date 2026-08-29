# Workbench 桌面壳：Electron 与 Tauri 2

> 类别：⑥ 机械后端与基础设施 · 证据编号：E-WORKBENCH-SHELL<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

> **v0.14.2 重解释**：本条目正文是 2026-08-23 Electron 决定的证据快照，探针、抽样与权限边界数据继续有效。2026-08-30 所有者依据[重开调研与 Ubuntu 实机探针](./workbench-shell-reopen-20260826/README.md)拍板：主选改为 **Tauri 2 + TS/React**，GPUI 原生备选，Electron 降为安全网；见文末 2026-08-30 复核记录与 [decision-history §30](../design/references/decision-history.md#30-workbench-桌面壳改选-tauri-2v0142)。

<a id="e-workbench-shell"></a>
## E-WORKBENCH-SHELL · Workbench 桌面壳：Electron 与 Tauri 2

### 当前决定

第一阶段保持 **Electron + React 19**，不改用 Tauri 2，也不直接基于 Wry 自搭桌面壳。这个决定不是声称 Electron 的启动或空载资源更优；它是在 Workbench 同时承载 xterm、富文本、DAG 图、CJK/IME、键盘和无障碍交互，并须交付 macOS/Linux 的条件下，以一次性的 Chromium 体积换取固定渲染器、较一致的平台行为和更直接的自动化测试。Workbench 仍只是控制面的薄客户端：Electron main/preload 不获得领域权威，也不得直接操作 tmux、PTY、Git、数据库或执行面服务器。

Tauri 2 保留为有明确重开条件的对照：若发布物体积成为产品硬门槛，或第一阶段收窄到可以固定 WebKitGTK 基线的少数发行版，则重开本决定并先做可丢弃探针；不能仅因“Rust”或官方最小示例很小而切换。直接 Wry 只提供 WebView 抽象，窗口生命周期、IPC 权限、更新、签名、打包和跨平台测试仍要自行补齐，不符合“不重造桌面框架”的取舍。

### 同口径 footprint 探针

探针于 2026-08-23 在 Apple Silicon、macOS 26.6.1 上完成。[Electron `v43.4.0`](https://releases.electronjs.org/release/v43.4.0) 使用官方 macOS arm64 ZIP；最小应用只创建一个隐藏 `BrowserWindow`，加载本地 `data:` HTML，并启用 `sandbox: true`、`contextIsolation: true`、`nodeIntegration: false`。稳定 6 秒后同时读取 `ps` 和 macOS `footprint`：

| 基线 | 分发与磁盘 | 空壳运行时 | 证据边界 |
| --- | --- | --- | --- |
| **Electron 43.4.0** | ZIP **116.46 MiB**（122,121,746 bytes）；解压后的 `Electron.app` **275.86 MiB**，其中 `Electron Framework` 约 **273.9 MiB** | main、GPU、network、renderer 四进程；RSS 直接相加约 **346.1 MiB**，但会重复计算共享页；`footprint` 的总 physical footprint 为 **77.58 MiB**（81,351,240 bytes） | 只是隐藏空窗下限；真实 React/xterm、缓存、字体和滚动缓冲仍须在整窗发布探针中测量 |
| **[Tauri 2.11.5](https://github.com/tauri-apps/tauri/releases/tag/tauri-v2.11.5)** | 官方说明[最小应用可低于 600 KiB](https://v2.tauri.app/start/)，因为[不捆绑系统 WebView](https://v2.tauri.app/concept/process-model/) | 本机未构建同口径 Tauri 空壳，不填写推测内存 | 600 KiB 是最小示例，不是 HCTL Workbench 的安装包或内存承诺；React 资产、Rust 依赖、图标、签名和更新组件都会进入实际发布物 |

按上一节当前 macOS arm64 离线包做下限算术，Electron 路线约为 **274 MiB 下载 / 643 MiB 已安装**，再加 Workbench 代码；Tauri 路线仍约为 **157 MiB 下载 / 367 MiB 已安装**，再加尚未实测的 Tauri/Workbench 壳。两者都仍排除 control、agentd、harness 和业务数据，因而不是整套产品容量承诺。

Electron 的成本属于一个 Workbench，而不随 Harness session 数线性复制：十个并行 Harness 应仍在一个主窗口/renderer 中承载十个 xterm 视图，不能为每个 runtime 新建一份 Electron 应用。xterm 的 DOM/canvas、字体和 scrollback 成本在 Electron 与 Tauri 中都会存在；P0 必须分别量出一窗一终端和一窗十终端，不能把空壳值当成完整场景。

### 性能、可移植性与 AI 代码友善度

| 维度 | Electron | Tauri 2 | 对 HCTL 的含义 |
| --- | --- | --- | --- |
| **启动与空载** | 自带 Chromium/Node，固定成本较高 | 复用系统 WebView，通常有更小的启动与空载基线 | Tauri 胜在壳；持续输入、滚动和图渲染仍主要由 WebView、React 和组件实现决定，不能笼统宣称“Rust UI 更快” |
| **渲染一致性** | macOS/Linux/Windows 随应用携带同版 Chromium | Windows 为 WebView2、macOS 为 WKWebView、Linux 为 WebKitGTK | xterm/IME/快捷键是发布合同，固定 Chromium 能减少平台变量；[xterm.js](https://github.com/xtermjs/xterm.js/) 官方同时支持现代 Safari 和 Electron，所以 Tauri 并非不可行，但仍须重新跑完整矩阵 |
| **分发与运维** | 包大且要随应用更新 Chromium 安全修复；运行时更自包含 | 包小且 WebView 多随 OS 更新；Linux 还要管理 GTK/WebKit 依赖和发行版差异 | Tauri 官方也说明 [Linux WebKitGTK 版本很难完整映射](https://v2.tauri.app/reference/webview-versions/)；当前 2.11 还有默认 AppImage 在新 Mesa/WebKit 组合下[窗口不起](https://github.com/tauri-apps/tauri/issues/15665)及[强制 X11](https://github.com/tauri-apps/tauri/issues/15781)的未结用户报告，足以要求发行版实机门禁，但不能外推为所有 Tauri 应用必然失败 |
| **自动化与 AI 开发** | 主路径可保持 TypeScript/React；[WebdriverIO 与 Playwright](https://www.electronjs.org/docs/latest/tutorial/automated-testing)可直接启动桌面应用，Chromium 调试面固定 | 前端同样是 React；Rust command、`Send`/`Sync`、插件与 capability 增加跨语言面，但编译器和 schema 能机械拒绝一部分错误；新版 [WebdriverIO Tauri service](https://v2.tauri.app/develop/tests/webdriver/) 已覆盖 macOS/Linux/Windows | Electron 对生成、调试和视觉回归更直接；Tauri 对权限配置更具机械约束。若重开 Tauri，必须固定 v2 文档/schema/Cargo.lock，拒绝混入 v1 allowlist 用法 |
| **权限边界** | 必须自行把 preload/IPC 收窄；`nodeIntegration: true` 会破坏 renderer sandbox，[Electron 20 起 renderer 默认 sandbox](https://www.electronjs.org/docs/latest/tutorial/sandbox/) | [capability/permission/scope](https://v2.tauri.app/security/capabilities/) 可按 window/webview 声明并生成 schema | HCTL 无论使用哪种壳，都只允许类型化 Query/Preview/Submit/Subscribe；Tauri capability 不能替代 control 准入，Electron main 也不能成为第二控制面 |

“单二进制”不能直接套到桌面壳：macOS 两者都要形成 `.app`，Electron 带 Framework/Helpers 而更自包含，Tauri 核心可很小但 Linux 依赖系统 GTK/WebKit。前者把运维成本装进发布物，后者把一部分兼容性成本留给目标 OS。

从 AI 编码角度，Electron 的优势是端到端 TypeScript、公开样本多、构建反馈短；风险则是模型很容易生成宽泛 preload、raw IPC、`shell`/`fs` 暴露或关闭 sandbox。Tauri 的 React 层同样容易生成，Rust 编译和 capability schema 更适合机械检查，但跨语言 glue、插件版本与系统 WebView 差异增加了诊断成本。对当前 HCTL，后端和领域权威本来就在 Rust control/agentd，桌面壳中再加入有权限的 Rust Core 没有架构收益；因此先选择更可预测的渲染与测试面，并用静态检查和合同测试锁死 Electron 权限。

### 本机 AI 工具形态抽样

下表检查的是同一台机器上的实际安装产物，而不是从产品界面猜测实现；大小只说明当前包形态和数量级，不适合作为不同产品功能量的横向 benchmark。尤其 Claude Desktop 是 universal binary，天然比单架构包更大。

| 产品与本机版本 | 实际形态 | 本机 footprint | 对桌面壳选型的证据 |
| --- | --- | --- | --- |
| **Codex CLI 0.149.0 / Codex.app 26.217.1959** | CLI 是带 Cargo registry 路径的 Rust arm64 Mach-O；桌面包直接链接 `Electron Framework` 并含 `app.asar` | CLI **210.32 MiB**；桌面包 **372.56 MiB** | CLI 与富桌面端采用不同交付形态；Codex 桌面端是 Electron，不是 Tauri |
| **Claude Code 2.1.240 / Claude Desktop 1.26832.0** | Claude Code 是含 Bun runtime 符号的 JavaScript 原生 Mach-O，不是桌面 WebView；Claude Desktop 是 universal Electron | CLI **310.00 MiB**；桌面包 **794.68 MiB** | “Claude Code”本身不能作为 Electron/Tauri 对照；对应富桌面端仍选择 Electron；Anthropic 同时提供[原生 CLI 安装](https://docs.anthropic.com/en/docs/claude-code/getting-started) |
| **Gemini CLI 0.46.0 / Gemini.app 1.96.4.775** | [Gemini CLI](https://github.com/google-gemini/gemini-cli/blob/main/GEMINI.md) 是 Node.js + TypeScript + React/Ink；macOS 应用直接链接 SwiftUI、WebKit、JavaScriptCore，没有 Electron Framework 或 `app.asar` | CLI 安装目录 **97.70 MiB**；桌面包 **270.96 MiB** | Gemini macOS 是原生 SwiftUI/WebKit 壳但不是 Tauri；“原生”也不自动等于小包 |
| **Antigravity `agy` / Antigravity IDE 2.1.1** | `agy` 的 Go build metadata 可直接读取；[VS Code 扩展会安装本地 `agy` 后端](https://antigravity.google/docs/ide/extensions/vscode/)；独立 IDE 的主程序名为 `Electron` 并链接 Electron Framework | `agy` **169.67 MiB**；IDE **697.49 MiB** | 后端/CLI 原生，富 IDE 使用 Electron/VS Code 系壳 |
| **Grok Build 1.0.5 / Grok.app 1.0** | [Grok Build](https://github.com/xai-org/grok-build) 是 Rust CLI/TUI；本机消费级 Grok.app 是 Safari Web App wrapper | CLI **128.13 MiB**；PWA 壳 **0.20 MiB** | coding Harness 不需要桌面 WebView；消费级 PWA 也不是 HCTL 富客户端的同类样本 |

抽样中没有一个已检查产物使用 Tauri：终端 Harness 多为 Rust、Go、Node/Bun CLI，复杂 coding 桌面端则明显偏 Electron，另有 Gemini 的原生 SwiftUI/WebKit 路线。这只能证明行业交付形态和成熟路径，不能代替 HCTL 自己的合同；大厂愿意承担数百 MiB 包体，也不能反向证明 footprint 不重要。

### 采用约束与重开门槛

- 只允许一个主 `BrowserWindow`/renderer 承载多场景与多终端；不得按 Project、Run 或 runtime 启动独立 Electron 应用。
- renderer 固定 `sandbox: true`、`contextIsolation: true`、`nodeIntegration: false`；不暴露 raw `ipcRenderer`、`fs`、`shell`、child process 或 PTY，preload 只导出窄而有版本的类型化客户端。
- React 场景代码保持普通浏览器可运行，Electron 专有代码限制在薄壳与打包层；这样未来若满足重开条件，可以替换为 Tauri 而不重写四个场景。
- 整窗 P0 同时记录 installer/archive、安装后磁盘、冷/热启动、idle physical footprint、一窗十 xterm 的 physical footprint/CPU/GPU，以及 macOS/Linux 的 CJK/IME、快捷键、粘贴、OSC 52、resize 和 screen reader；任一输入正确性问题优先于渲染帧率。
- 若重开 Tauri 2，探针必须覆盖 WKWebView 与每个承诺支持的 Linux WebKitGTK 基线、`.deb`/`.rpm` 和升级路径；在上述 AppImage 问题关闭并复测前，不把“单 AppImage 覆盖任意 Linux”写成能力。探针失败即保留 Electron，不下沉到 Wry 自研。

## 复核记录

- **2026-08-26**：所有者判断 Electron 体积与启动成本过高、Tauri 2 的 Linux 支持不足，重开桌面壳调研（GPUI / Iced / Flutter / Web 壳，含 macOS 与 Ubuntu NVIDIA/Wayland 实机探针），见 [workbench-shell-reopen-20260826/](./workbench-shell-reopen-20260826/README.md)；该调研拍板前本条目的现行决定不变。
- **2026-08-30**：重开调研回填 Ubuntu 实机探针（所有者主开发机 Ubuntu 26.04 / GNOME 50.1 / Wayland / NVIDIA Quadro P620，附录 A7）：Tauri 2.11.5 全项通过（含 10 个 xterm.js WebGL 终端与中文 IME；候选窗定位由应用侧 caret 修正解决，已反馈 tauri#11412），正文引用的 AppImage/强制 X11 未结报告未打中该环境的 `.deb` 发行；GPUI + gpui-component 同机通过；Flutter 因最大化稳定 SIGSEGV（flutter#191775）暂时淘汰；Electron 44 原生 Wayland 有顶边 1 px 闪动，此类环境须默认 `--ozone-platform=x11`。所有者拍板改选 **Tauri 2 + TS/React 主选，GPUI 原生备选，Electron 安全网**（v0.14.2，[decision-history §30](../design/references/decision-history.md#30-workbench-桌面壳改选-tauri-2v0142)）。「采用约束」按壳中立迁移：单主窗口承载多场景多终端、React 场景代码保持浏览器可运行、类型化窄 IPC 面不变；权限边界改由 Tauri capability/permission/scope 按窗口声明；整窗 P0 清单移植到 Tauri（WKWebView 与承诺的 Linux WebKitGTK 基线、`.deb`/`.rpm` 与升级路径、CJK/IME、一窗十终端），探针失败即回退 Electron 安全网，不下沉 Wry 自研；「重开门槛」一节自此了结。

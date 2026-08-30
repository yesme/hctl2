# Workbench 桌面壳重开调研:GPUI / Iced / Flutter / Web 壳(讨论备忘)

> 日期:2026-08-26<br>
> 对象:main @ `12c9b44`(草案 v0.13.0);当时的决定见 [`E-WORKBENCH-SHELL`](../workbench-shell.md#e-workbench-shell)(Electron + React 19,Tauri 2 有条件重开)。<br>
> 触发:所有者判断 Electron 太肥太慢、Tauri 2 的 Linux 支持太差,提出考察 GPUI(Zed)、Iced 与 Flutter;并把第 4 条诉求从"必须 React"改为"复用现成 UI 轮子、用公共的前端方法论,事件循环这类东西不自己造"。<br>
> 目标环境:macOS Apple Silicon;**Ubuntu 物理机、NVIDIA 显卡、Wayland(GNOME)**(所有者主开发机);Windows。<br>
> 模式:六条调研线(GPUI 本体、GPUI 控件生态、Iced、Flutter、Linux 上的 Web 路线、本机同口径探针 ×3)+ 主线复核(gpuix、libghostty、awesome-gpui、tty7/gpui_xterm、Zed GPL crate 依赖扇出)。每条事实附来源;未核实项单列。<br>
> 性质:**讨论备忘,不是决定**。是否重开 E-WORKBENCH-SHELL、是否改 delivery.md 的 Workbench 技术栈、是否给 `hctl2-workbench` 换许可证,由所有者拍板。<br>
> 附录(原始调研报告,含全部 URL 与数字):A1 GPUI 本体 `…-a1-gpui-core.md` · A2 GPUI 控件生态 `…-a2-gpui-ecosystem.md` · A3 Linux 上的 Web 壳路线 `…-a3-linux-web-routes.md` · A4 Iced `…-a4-iced.md` · A5 Flutter `…-a5-flutter.md` · A6 macOS 本机探针 `…-a6-local-probes.md` · A7 Ubuntu NVIDIA/Wayland 实机探针 `…-a7-ubuntu-probes.md`(文件名前缀均为 `workbench-shell-reopen-20260826`)。

## Ubuntu 实测回填（2026-08-26）

下列 Ubuntu 物理机实测**覆盖本备忘原先仅根据上游 issue 得出的平台推断**，完整环境、步骤、数值、上游反馈和产物校验见附录 A7：

1. **Tauri 2.11.5 在所有者的 Ubuntu 26.04 / GNOME 50.1 / Wayland / Quadro P620 / NVIDIA 580.173.02 上通过。** Canvas 2D、WebGL、10 个 xterm.js WebGL terminal、长列表滚动、最大化和中文输入均可用；应用把前端 caret 矩形传给 WebKitGTK `InputMethodContext` 后，所有者确认各输入控件的 IME 候选窗也正常。此前“在主开发机上可以判死”的结论被实测证伪。
2. **GPUI + gpui-component Gallery 通过。** 原生 Wayland、NVIDIA GPU、输入、滚动和 IME 正常，完整 release 构建及独立二进制运行成功。它保留为原生备选，剩余问题是生态与开发总成本，不是本机兼容性。
3. **Flutter 暂时淘汰。** 普通窗口和 IME 正常，但默认应用最大化时在 Wayland/X11 均稳定 SIGSEGV；已提交 [flutter/flutter#191775](https://github.com/flutter/flutter/issues/191775)。
4. **Electron 44 可用但原生 Wayland 有顶边 1 px 闪动。** XWayland 对照消失；若采用 Electron，此类 NVIDIA/分数缩放环境应默认 `--ozone-platform=x11`。统一重载探针下 Electron `.deb` 95.69 MiB、PSS 371.71 MiB，Tauri `.deb` 2.80 MiB、PSS 283.95 MiB；GPUI Gallery 的不同负载 PSS 204.97 MiB，不能严格横比。
5. **实测后的工程建议：Tauri 2 + TS/React 优先，GPUI 为原生备选，Electron 为安全网，Flutter 等上游修复。** 本备忘仍是讨论记录，正式技术栈决定需另行落入 implementation evidence / delivery，而不是由探针备忘暗改。

Tauri IME 候选窗定位的复现和应用侧修正已反馈到 [tauri-apps/tauri#11412](https://github.com/tauri-apps/tauri/issues/11412#issuecomment-5420366006)。组合下划线在已有文本前可能延伸到行末，是 WebKitGTK 的 preedit 绘制瑕疵；候选、选字和最终输入不受影响，所有者接受不在应用层叠加特判。

## Ubuntu 探针前的调研阶段结论（保留原文供追溯）

1. **Tauri 2 在所有者的主开发机上可以判死。** WebKitGTK + NVIDIA + Wayland 的黑屏/Error 71 问题从 2024-08 开到今天(tauri#10702,2026-08 仍有 WebKitGTK 2.52.5 + 驱动 580 的报告),WebKit 上游把"为 NVIDIA 禁用 DMA-BUF"标成 WONTFIX,Tauri 官方文档(2026-06-15 更新)仍教三级环境变量降级。替代运行时全灭:Verso 归档(2025-10),CEF 分支仅 X11 且"source available",qtwebengine 方向暂停。
2. **GPUI + WebView 不是出路。** gpui-component 的 WebView 就是 wry(Tauri 的 WebView 层),Linux 仍是 WebKitGTK,且 gpui-component 自己的 Linux 示例标注 "doesn't work yet"。混合方案 = Tauri 的短板 + GPUI 的复杂度。
3. **Electron 的"肥"是磁盘和空载内存,不是不可用。** 同口径:空窗 physical footprint Electron 77.6 MB vs GPUI/Iced 各 42 MB;磁盘 276 MiB vs 4–8 MiB。Electron 38.2+ 在 Wayland 会话默认原生 Wayland,NVIDIA 相关只有一个几乎无人跟进的 VS Code 黑屏 issue。它仍是轮子最全、AI 最熟、三平台最稳的路线;"慢"(冷启动)本轮没有量。
4. **GPUI 是"IDE 形态开发工具"的最佳原生候选,但要接受三件事**:crates.io 的 gpui 冻结在 0.2.2(2025-10-22),要用现版必须 git 依赖 Zed main(或社区转发布),README 自述 pre-1.0 常有破坏性变更;Linux 上 NVIDIA 仍有两个 S1 open 且 GPU 初始化失败不会自动回退;terminal / kanban / DAG 三个界面没有成熟轮子,只有种子——但都有可抄的源码。
5. **Iced 是"方法论最公共、测试最省心、软件回退最稳"的原生候选**,但对 HCTL 这种"多面板 + 多终端 + 长聊天流"的形态证据更少:没有虚拟列表(#160 开了六年,scrollable 上千项有退化 issue)、无障碍上游没有、发布节奏 15 个月一版、主要应用全在 git fork 上。
6. **Flutter 是原生路线里 AI/TDD/无障碍最完整的一条**(80k 仓库、官方 `llms.txt` 与 AI rules、Dart MCP server、无 GPU widget test、三平台读屏骨架、`graphview` 有 dagre 级分层布局、同形应用 Alera/AppFlowy/FluffyChat 真实存在),**但 Linux 面有几个 open 正好打在目标机与 HCTL 的使用形态上**——注意这不是"不支持 Ubuntu":Ubuntu 20.04–24.04 在官方支持表、CI 跑 22.04、Canonical 自家 App Center/安装器/固件更新器都是 Flutter,且 Canonical 接手桌面三个月已合入 Wayland 子表面渲染器。窄化后的事实是:嵌入层是 GTK3(上游已年更维护、无 fractional scaling),渲染走 OpenGL/EGL 且不自动回退;Wayland + NVIDIA 黑屏 #188966 open,但案例是 **Optimus 混合显卡 + KDE**,P2 有 workaround,是否打中单 dGPU + GNOME 台式机未知;Linux 嵌入层没有 vsync 回调而退到固定 60 Hz 定时器(#191245)、FPS 随窗口尺寸下降(#191425:1080p ≈30)——Canonical 的小窗口工具类应用不会撞上,HCTL 的常驻大窗口 + 多终端会;Linux IME 有 fatal crash open(#190046)。terminal 同样只有种子(`xterm.dart` 停更、`ghostty_vte_flutter` 新)。桌面路线图 2026-05 移交 Canonical,是双刃剑。
7. **推荐:GPUI + gpui-component 为主候选,Flutter 并列进入同一轮 Ubuntu NVIDIA/Wayland 一次性丢弃探针;任一通过才重开决定;Iced 记录为备选;探针失败保留 Electron**(与现行"探针失败即保留 Electron,不下沉自研"的规则一致)。探针清单见末节。
8. **Workbench GPL 化在法律上成立、在架构上与现行三面切分吻合、在工程上只买到"抄代码的自由"而不是"cargo add 的自由"**;它的独特收益集中在 GPUI 路线(Zed 的 `terminal`/`terminal_view` 约 680 KB Rust 可抄;`agent_ui` 的 @ 补全解析可抄,但其 2.4 MB 编辑器面板绑死 GPL `editor`,抄不动)。是否值得,取决于是否选 GPUI 且是否真要抄 Zed 终端;仅为 terminal 一项,MIT/Apache 的替代种子已经存在。Flutter 路线下最值钱的同形应用(AppFlowy、FluffyChat)是 **AGPL**,GPL-3 的 Workbench 也抄不了,只能当行为参考。

## 所有者四条诉求(重述为可核验的轴)

| # | 诉求 | 可核验的问法 |
| --- | --- | --- |
| 1 | 方便 AI 开发:AI 熟悉的路径、工具、TDD | 公共代码量与示例数;API 是否分叉(模型最易写错处);无显示 CI 能否跑 UI 测试;能否模拟输入、截图回归 |
| 2 | 控件覆盖:侧栏、Chat Room(CJK/IME、@ 提示)、Kanban、DAG Workflow、Terminal | 每个界面是"现成 / 有种子要补 / 自己造"(见轮子覆盖矩阵) |
| 3 | mac / Ubuntu(NVIDIA+Wayland)/ Windows 直接跑 | 各平台后端成熟度;目标机的已知故障;IME 三平台 |
| 4 | 复用现成 UI 轮子、公共方法论,事件循环等不自造 | 事件循环/窗口/IME/无障碍/渲染谁提供;方法论是否公共知识 |

## 四条路线的现状事实

### Web 壳:Electron / Tauri 2 / 其他

- **Electron**:v44.0.0(2026-08-25,Chromium 152,Node 24),macOS arm64 zip 129.7 MB。Electron 38 起 `--ozone-platform` 默认 `auto`,Wayland 会话即原生 Wayland(PR #48301,2025-09;官方博客 2026-03 "38.2 起开箱支持");可用 `--ozone-platform=x11` 强制 XWayland。NVIDIA+Wayland:vscode#280464 黑屏(2025-12,open,1 条评论);electron#49247 OSR 无绘制(NVIDIA+Wayland,2026-06 因不活跃关闭)。无官方 lite 版。
- **Tauri 2**:2.11.5(2026-07-01);Ubuntu 22.04/24.04/26.04 的 webkit2gtk-4.1 分别为 2.50.4 / 2.52.3 / 2.52.3。官方 Linux Graphics Issues 页:"most often NVIDIA GPUs … anything from a blank window to subtle rendering problems",按 `__NV_DISABLE_EXPLICIT_SYNC=1` → `WEBKIT_DISABLE_DMABUF_RENDERER=1` → `WEBKIT_DISABLE_COMPOSITING_MODE=1` 逐级降级。#10702(Error 71,open,50 条评论,2026-08-24 仍活跃)、#14924(NVIDIA 590+ 透明窗口崩溃,open)、#9394(NVIDIA 问题文档,open 自 2024-04);WebKit bug 262607 WONTFIX。AppImage 在 Mesa 25+ 发行版起不来(#15665 open)、无 XWayland 时硬崩(#15902 open)。IME:#11412 候选框错位(open 自 2024-10)。维护者 2026-02-17:CEF 分支 "more like source available",Verso "halted",qtwebengine 方向暂停。
- **其他保留 TS/React 的壳**:Electrobun v2.0.1 在 Linux 默认仍是 WebKitGTK(CEF 可选捆绑);Neutralino、Wails v3 同样 WebKitGTK;Tauri `feat/cef` 分支活跃但仅 `linux-x11`、透明窗口黑屏 open;React Native 无 Linux 目标;Lynx 明确 "Linux UI 尚未计划";react-nodegui 2023-11 起停更。**结论:除 Electron/CEF 外,所有 Web 壳在 Linux 上都是 WebKitGTK。**
- **同类工具的选择**:Multica、Cumora、Orca(53k★)、Superset、Cursor 都是 Electron;Codeg 是 Tauri 2 且有 Linux 包;Conductor 是 Tauri 但仅 macOS;Zed 与 Warp 是自研 Rust GPU UI(Warp 的 Linux/Wayland IME 直到 2026-04 才修好——原生路线 IME 风险的实证)。

### GPUI(Zed)

- **发布与可用性**:crates.io `gpui` 0.2.2(2025-10-22)后 10 个月无发布;Zed main 于 2026-02-19 拆出 `gpui_platform`/`gpui_linux`/`gpui_macos`/`gpui_windows`/`gpui_wgpu`/`gpui_web`,全部 `publish = false`。0.2.2 与 main 的 hello-world 入口已不同(`Application::new()` vs `gpui_platform::application()`)。gpui-component 因此用 `git = "https://github.com/zed-industries/zed"`(无 rev,Cargo.lock 解析到 2026-08-17 的 commit)。社区转发布:`gpui-unofficial`(按 Zed tag,1.16.2 @ 2026-08-24)、`gpui-ce`、`wgpui`。文档:gpui.rs 单页 + 两份 md;README:"最好的学习方式是读 Zed 源码或去 Discord 问"。工具链:edition 2024,Zed 钉 Rust 1.97.1。
- **许可证**:`gpui*`、`gpui_tokio`、`sum_tree`、`util`、`collections` 等 Apache-2.0;`ui`、`ui_input`、`editor`、`terminal`、`terminal_view`、`workspace`、`project`、`theme`、`settings`、`markdown`、`agent_ui` 等 **GPL-3.0-or-later**;未发现 AGPL crate。
- **平台**:macOS Metal;Linux 自 2026-02-13 起 wgpu(Vulkan | GL),适配器优先级 discrete > integrated > … > CPU(llvmpipe 最后兜底);Windows DX11 + DirectWrite(Zed Windows 稳定版 2025-10-15)。Zed 1.0 于 2026-04-29 三平台同发,最新 1.16.2(2026-08-24)。**后端选择仅由环境变量决定(`WAYLAND_DISPLAY` → Wayland,否则 X11),GPU 初始化失败不自动回退**,手动回退是 `WAYLAND_DISPLAY="" zed`。
- **NVIDIA + Wayland(目标机)**:zed#52944 "Zed crashes the host on Linux with NVIDIA GPU since 0.230.0"(open,S1,`graphics:nvidia`;根因 wgpu 29 GL/EGL 路径在 NVIDIA 上 panic,GNOME 50 Wayland + RTX 5090 + 595.71 复现;临时 `__NV_FORCE_ENABLE_X11_EGL_PLATFORM=1`;2026-08-07 标 stale);zed#62998 设备丢失恢复后 atlas 越界崩溃(open,S1,GTX 1650 Ti,2026-08-21);zed#53522 GNOME 50/Wayland 卡顿(open,39 条评论,未定位);已关闭:#35948(NVIDIA 580 早期驱动挂死,驱动 580.82.07 修复)、#39097(blade 时代冻结,wgpu 切换后过时)。仓库内无 `linux_drm_syncobj` 引用,explicit sync 是否由 wgpu 协商未核实。
- **IME**:macOS `NSTextInputClient`;Wayland `zwp_text_input_v3`(GNOME 下 fcitx5 须经 IBus 前端,候选框不能盖在 gnome-shell UI 上——fcitx 官方 wiki);X11 XIM(fcitx5 相关 open:#54959、#58192、#59662、#57131);Windows **IMM32 而非 TSF**,中文 IME open:#59882(S2)、#59193、#56149、#61724。`area:controls/ime` 开着 14 个。Zed 是重度 IME 用户,修得快,但不是"没问题"。
- **无障碍**:AccessKit 于 2026-05-27 进 main(PR #56065),0.2.2 没有;Zed 自身接入早期(10 个文件用 `role`),Windows 读屏 #41138 仍 open。
- **测试**:`#[gpui::test]` + `TestAppContext`/`VisualTestContext`(模拟按键、鼠标、resize、prompt),`HeadlessAppContext` 可真实排版;截图回归 `VisualTestPlatform` **仅 macOS**;Zed CI 三平台跑 `cargo nextest` 不需要显示。Zed 内 332 个文件用 `gpui::test`。下游曾被 `TestWindow::window_handle` panic 打断(#62510,2026-08 修)。
- **原语**:`div`(taffy flex/grid,Tailwind 风格链式样式)、text/InteractiveText、img/svg/canvas(`Path` 只有二次贝塞尔 `curve_to`,无三次)、`list`/`uniform_list`、anchored/deferred、drag-and-drop(`on_drag`/`drag_over`/`on_drop`/`can_drop`)、`Surface`(**仅 macOS** CVPixelBuffer,无跨平台外部纹理入口)。**不含文本输入控件**(只有 `EntityInputHandler` trait,`examples/input.rs` 手搓);表格/树/菜单/tooltip/tabs/dock 都不在 gpui 本体。
- **gpui-component(Longbridge)**:13.5k★,Apache-2.0,129 贡献者,main 2026-08-25;crates.io 0.5.1(2026-02-05)后无 tag。两层(`gpui-base` 无样式行为层 + `gpui-component`)。已有:Sidebar 全套、DockArea/TabPanel/Tiles(可拖拽、可序列化)、Textarea(rows/auto_grow/soft-wrap)、`EntityInputHandler` 完整实现(`marked_text_range`/`replace_and_mark_text_in_range`,Longbridge Pro 中文券商桌面端在用)、LSP 式 `CompletionProvider`(`is_completion_trigger` 可对 `@` 返回 true,但挂在 Editor 模式而非纯 Textarea——待验)、`VirtualList`/`Table`/`DataTable`(十万行级)、markdown/html `TextView`(不支持内联插件/CSS)、代码编辑器(Tree-sitter,50+ 语法)、charts、Dialog/Sheet/Popover/ContextMenu/Tooltip/Tabs、FocusTrap、`skills/` 目录(给 AI 代理)。**没有 terminal、kanban、graph。** WebView = `gpui-wry`(lb-wry 0.53.3 fork),Linux 标 TODO。
- **JS/React 绑定**:`remorses/gpuix`(Apache-2.0,1,123★,2026-01 创建,8 月仍活跃):React reconciler → napi-rs 突变协议 → Rust 保留树 → GPUI;元素 div/text/code/diff/markdown/input/textarea/virtual-list/img/svg/anchored;声称 IME;npm 预编译含 linux-x64-gnu/win32;**约 92% 提交来自一人;README 自述 "Windows runtime validation is pending",Linux 无验证声明,测试渲染器只写 Metal;无 terminal,canvas 与多窗口在待办**。证明"TS 描述 GPUI"可行,不到押注程度。
- **生态规模**:awesome-gpui 101 条,Libraries 区 23 个(gpui-component、guise 104★、gpui-router、gpui-storybook、gpui-whiteboard、plotters-gpui、gpui-flow、ferrum-flow),**Libraries 区无 terminal、无 kanban**;Apps 区多面板开发工具:tty7(Apache,780★)、Arbor(MIT,806★)、waku(GPL,1.2k★)、Codux(GPL)、oxideterm(GPL,1.3k★)、navop、GitComet(AGPL)。crates.io 反向依赖 gpui 128。

### Iced

- **发布**:0.14.0(2025-12-07,MIT,MSRV 1.88);24 个月内 3 个版本,0.13.1→0.14.0 隔 15 个月;master 领先 336 提交、`0.15.0-dev`、MSRV 1.92,用 **iced 自己 fork 的 winit**(rev 2025-09)与 wgpu 29。Halloy、libcosmic、hermes-chat 都依赖 git fork 而非 crates.io。0.13 破坏性:`Program`/`Task`/`Daemon`/闭包样式/`Sandbox` 静默移除;0.14:`Widget::update` 签名、Rust 2024、`iced::application(boot, update, view)`。Book 缺 Layout/Styling/Widgets/Testing 章节;55 个官方示例。
- **渲染与回退**:wgpu 主 + `tiny-skia` 软件回退;`fallback::Compositor` 运行时先试 wgpu 失败再试 tiny-skia;`ICED_BACKEND`/`WGPU_BACKEND` 环境变量可强制。**这是三条原生路线中唯一自动软件回退的**。
- **NVIDIA + Wayland**:未找到 iced 特有的 2025–26 黑屏/闪烁 open issue;历史 #2297(NVIDIA 冻结,驱动 550.67 修复)。open:#3138(GL 后端花屏)、#3229(Wayland 漏 Closed 事件)、#3418(无 ext-data-control-v1 的合成器剪贴板坏)。wgpu#7475(NVIDIA+AMD 混合 SURFACE_LOST,open)。注意 iced 的 Linux 实战量主要在 COSMIC(cosmic-comp 之下),不等于 GNOME Wayland 之下。
- **IME**:#979 从 2021-08 开到 2025-02;PR #2777 "Input Method Support"(2025-02-03 合并,over-the-spot 预编辑)**首次发布在 0.14.0**;Halloy 中文输入 issue 因此关闭;open:#3189(Windows 日文候选框错位)、Halloy #2211(Mozc)、#1490(韩文渲染)。winit 平台注记:X11 只支持位置不支持区域。
- **测试**:`iced_test` 0.14(`Simulator`:click/tap_key/typewrite/find/snapshot,PNG 或 sha256 快照,`ICED_TEST_BACKEND=tiny-skia` 无 GPU),iced 自己 CI 三平台就这么跑;0.14 加 `.ice` 端到端脚本(F12 录放)。但 COSMIC 与 Halloy 都没用它(code search 0 命中);反向依赖 16。
- **无障碍**:#552 自 2020 open;2026-03-14 社区 PR #3281(AccessKit 0.24,覆盖多控件)被维护者关闭:"I'll work on this myself";只有 libcosmic 的 `iced_accessibility`(pop-os/iced fork,领先上游 258、落后 336)和 `plushie-iced` fork 有。
- **控件**:自带 `pane_grid`(分栏、拖拽、resize)、markdown/rich_text、text_input/text_editor、table、grid、combo_box、tooltip、stack/opaque/mouse_area(拼 modal)、canvas、`lazy`。**不含**树、上下文菜单、modal 控件、通用跨控件拖拽、**虚拟列表**(维护者 2026-08-17:"we don't have a `list` widget")。`iced_aw` 0.14.1(2026-04):SideBar、Tabs、Menu、ContextMenu、DropDown、Card 等。`iced_term` 0.8.0(2026-03,alacritty 0.25 引擎,自述三平台测过、不完整、API 不稳)。`iced_drop` 0.2.42(2026-08,有 Trello 式看板示例)。`iced_nodegraph` 0.4.2(2026-07,**仅 wgpu**,无自动布局)。`iced_webview` 停在 2024-11。libcosmic(MPL-2.0)补 nav_bar/segmented_button/context_drawer/dialog/menu/tab_bar/dnd 等,但 Windows/macOS 2024 有失败报告、2026 无正面证据。
- **规模信号**:crates.io 反向依赖 iced 367;GitHub `iced language:rust` 1,728 仓库 vs `gpui` 1,036;已出现 LLM 编造不存在控件的 issue(#3429)。
- **应用**:Halloy(IRC,4.4k★,GPL,pane_grid + text_editor + 自制 anchored_overlay 做 @/命令补全)、Sniffnet(40k★)、COSMIC 全家(cosmic-term 用 alacritty_terminal + cosmic-text 自绘)、Icebreaker(本地 LLM 聊天)。

### Flutter

- **发布与治理**:3.47.0(2026-08-12,BSD-3;本机装到 3.47.1 / Dart 3.13.1,2026-08-19),季度一版(3.41 二月、3.44 五月、3.47 八月、3.50 十一月)。**3.44(2026-05-20)起 Canonical 成为 Flutter Desktop 的 "lead maintainer and Strategic Steward",领导桌面路线图并维护 Linux/Windows/macOS 三个嵌入层**;已落地的 Canonical 提交:Linux tooltip/popup 窗口、`fl_view_new_sized_to_content`、Wayland EGL 子表面渲染器(2026-08-24 合并)、ATK 读屏回归修复、`flutter doctor` 显示 Linux GPU 驱动。支持表:Ubuntu 20.04–24.04、Debian 10–13、Windows 10/11、macOS 12–26(Intel 退役中)。**多窗口仍是实验特性且只在 main 频道**(`enable-windowing`,预发布清单 0/20)。3.47 把 Material/Cupertino 拆成 `material_ui`/`cupertino_ui` 1.0,SDK 内置版计划 11 月正式弃用——又一层 API 变动。
- **Linux 嵌入层与渲染**:**GTK3**(需 `libgtk-3-dev`;GTK4 issue #94804 自 2021-12 无人认领,P3);GTK 3 上游 2026-03 宣布降为年更、下次发布 2027-03;GTK3 不实现 `wp_fractional_scale_v1`,Flutter #127768 "字体缩放了别的没缩放"(2023 起 open)。Wayland 经 GDK 后端,2026-08-24 起用 EGL 子表面呈现;拿不到 `xdg_toplevel`(#187837 open)。**3.47 起 Impeller 在 Linux 默认开启,后端 OpenGL/EGL**,Vulkan 仍是 open PR(#189584,2026-07);`FLUTTER_LINUX_RENDERER=software` 可选软件渲染但"not recommended";**不自动回退**(#177548 open)。
- **NVIDIA + Wayland(目标机)**:**#188966 "GTK embedder: Failed to create OpenGL context on Wayland + NVIDIA Optimus, results in black screen"(open,2026-07-04,P2,NVIDIA 610 + Mesa 26.1,KDE Wayland;`GDK_BACKEND=x11` 与 `LIBGL_ALWAYS_SOFTWARE=1` 均无效,只有 `__NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia` 可用)**;#152099 NVIDIA `glBlitFramebuffer` 黑屏(2024,已修);#190059 GL 帧超时后 SIGSEGV(2026-07,关闭)。**帧同步:Linux 嵌入层不提供 `vsync_callback`,引擎退到固定 60 Hz `VsyncWaiterFallback`(#191245,2026-08-18 open);定时器毫秒量化导致掉帧(#190620 open);FPS 随窗口尺寸下降——720p ≈60、1080p ≈30、3072 宽 ≈10(#191425,2026-08-20 open)**。`platform-linux` open issue 共 279 个。
- **IME**:Linux 走 `gtk_im_multicontext`(GNOME Wayland 下经 ibus/fcitx5 的 GTK3 模块),有 `set_cursor_location` 定位候选框;**#190046 "输入法输入时 abort:`DeleteSurrounding` 拆分 UTF-16 代理对"(open,2026-07-26,P2,fatal crash,中文五笔/ibus 触发,修复 PR #190514 未合)**;#154072 emoji 插入三平台崩溃(open 自 2024)。macOS:日文重复输入已修(2025-06);open:#190704 第三方中文输入法切换 SIGABRT(2026-08)、#190525 IME 确认的 Enter 同时触发 Shortcuts。Windows:仍用 **IMM32** 非 TSF(#182876);open:#191196 组合串泄漏到下一个输入框、#189491 韩文音节丢失、#173526 中文候选框位置。
- **无障碍**:三平台都有(Orca 经 ATK/AT-SPI、VoiceOver、NVDA/JAWS);Linux 读屏曾在 3.35–3.37 静默失效(2025-10 修);open:选中态/切换态不播报、页面标题不读、Windows 缺 `IValueProvider`/`ITextEditProvider`。是四条路线里除 Web 外唯一"骨架齐全"的。
- **测试与 AI**:widget test 无 GPU 无显示(本机 1 测试热跑 1.2 s);golden 有跨平台字体差异告警(默认 Ahem 字体);desktop `integration_test` 在 Linux CI 须 `xvfb-run`;热重载仅 debug。官方 `llms.txt`、AI rules 文件(含 `CLAUDE.md` 映射)、Dart MCP server(实验)。GitHub topic `flutter` 80,582 仓库(tauri 11.7k、egui 1.1k、iced 329)。已知模型写旧 API 的坑:`withOpacity`→`withValues`、`MaterialStateProperty`→`WidgetStateProperty`。
- **轮子**:dock/分栏现成(`docking` 1.16.2 / `multi_split_view` 3.6.2,MIT;`NavigationRail` 内置);composer:内置 `Autocomplete`(FluffyChat 就用它做 `@`/`#`/`:`/`/`)+ `flutter_quill` 11.5 / `fleather` 1.28 / `super_editor`(pub 页自述 Windows/Linux 未验证、桌面无 popover),`flutter_mentions` 2021 停更;**kanban 全部停更 ≥13 个月**(`appflowy_board` 0.1.2 @ 2024-04,AGPL/MPL 双许可;`drag_and_drop_lists` 2024-11);**DAG:`graphview` 1.5.1 有 Sugiyama 分层布局 + Buchheim + Fruchterman,pan/zoom 走 `InteractiveViewer`,节点拖拽后重排未证实**;terminal:`xterm.dart` 4.0.0(2024-02,最后提交 2025-06,"新版 Flutter 打不了字" #207 无人回应)、`flutter_pty` 2025-01、`ghostty_vte_flutter` 0.1.3(2026-05,Ghostty VT + `portable_pty`)、`libghostty` Dart 绑定 0.0.12(2026-07);markdown:`flutter_markdown` 被 Google 停更(2025-04-30 六包一起),继任 `flutter_markdown_plus` 1.0.12(2026-07)、`gpt_markdown` 1.2.1(2026-08,流式);虚拟列表内置(`ListView.builder`、`super_sliver_list`);菜单/快捷键/tooltip/modal 内置,`super_context_menu` 走原生菜单;桌面风格包 `yaru`(Canonical,MPL)、`fluent_ui`、`macos_ui`、`shadcn_ui`、`forui`。
- **同形应用**:**Alera**(2026,"native, performance-first ADE",Flutter + Rust `portable_pty` + Ghostty VTE + flutter_rust_bridge,MIT,17★,macOS/Windows/Linux)、**AppFlowy**(Flutter + Rust,AGPL,76k★;0.13.2 的 Linux .deb 73 MB / AppImage 182 MB、macOS arm64 dmg 99 MB;Rust 互操作是自研 `dart-ffi` + `allo-isolate` + protobuf,不是 FRB)、**FluffyChat**(Matrix 客户端,AGPL,3k★;Linux tar.gz 30 MB、Flathub 33 MiB;**无 Windows/macOS 发行**,Windows 支持 issue 自 2023 open)、**Ubuntu 自家桌面应用四件都是 Flutter(均 GPL-3.0,2026-08 仍在推)**:App Center(`ubuntu/app-center`,Dart 651 KB,`pubspec` 要求 Flutter ≥3.44.2,且仍依赖已被 Google 停更的 `flutter_markdown ^0.7.3`)、Security Center(`canonical/desktop-security-center`,"Flutter-based security center for Ubuntu Desktop",Dart 2.2 MB)、Firmware Updater(`canonical/firmware-updater`,Dart 1.2 MB)、安装器(`canonical/ubuntu-desktop-provision`,Dart 8.2 MB),外加 `ubuntu/yaru.dart` 控件库(MPL-2.0);三个应用的 README 都没有任何 NVIDIA/Wayland/`GDK_BACKEND` 注记。Rive/Superlist(闭源)。Zulip Flutter 自述桌面"仅供开发"。
- **Rust 互操作**:`flutter_rust_bridge` 2.13.0(2026-08-23,MIT,Flutter Favorite);Dart 3.10 起 build hooks 稳定。HCTL 的 Workbench 走命令服务,不必 FFI。
- **风险**:Google 2024-04 裁员波及 Flutter/Dart;Flock 分叉(2024-10,批评"桌面停滞")已于 2025-12 停更;Impeller 迁移期 Windows 新崩溃(#191468/#191437/#191353,2026-08)、macOS Intel 闪烁;`scrollable_positioned_list`(google.dev)2023 后未更新;macOS 默认开 App Sandbox,出网需 entitlement;支持表最新只到 Ubuntu 24.04。

## 轮子覆盖矩阵

"现成"=拿来即用;"种子"=有可抄/可依赖的雏形,要补;"自造"=无参照。许可证按 Workbench 保持 Apache-2.0 计;括号内是 Workbench GPL 化后新增可抄的。

| 面 | Web 壳(Electron) | GPUI(+ gpui-component) | Iced(+ iced_aw / libcosmic) | Flutter |
| --- | --- | --- | --- | --- |
| 事件循环 / 窗口 / 渲染 | 现成(Chromium) | 现成(Zed 自用,Apache) | 现成(winit fork + wgpu / tiny-skia) | 现成(GTK3 / AppKit / Win32 嵌入层 + Impeller;Linux 无 vsync 回调) |
| IME 三平台 | 现成 | 现成(Win 为 IMM32;X11 fcitx5 有 open) | 现成(0.14 起;Win 日文 open) | 现成(Win 为 IMM32;Linux 有 fatal crash open) |
| 无障碍 | 现成 | 种子(2026-05 进 main,0.2.2 无) | 自造(上游无;fork 有) | 现成(三平台读屏,有 gap) |
| 侧栏 / dock / 分栏 | 现成(任选) | 现成(Sidebar、DockArea/Tiles) | 现成(pane_grid;iced_aw SideBar;libcosmic nav_bar) | 现成(`docking`、`multi_split_view`、NavigationRail) |
| Chat 输入框(CJK + @) | 现成(Tiptap + mention 扩展) | 现成 Textarea + `CompletionProvider` 触发 `@`(挂 Editor 模式,待验)(GPL:`agent_ui` 的 `MentionCompletion::try_parse` 可抄) | 种子(text_editor + 自制 anchored_overlay,照 Halloy 抄,GPL) | 现成 `Autocomplete`(FluffyChat 同法)+ 富文本 `flutter_quill`/`fleather`;桌面 IME 可靠性无包级证据 |
| 长消息流 / 日志(虚拟化) | 现成 | 现成(uniform_list、VirtualList) | **自造**(无虚拟列表) | 现成(`ListView.builder`、`super_sliver_list`) |
| Markdown / 代码高亮 | 现成 | 现成(TextView + Tree-sitter) | 现成(markdown 控件;highlighter) | 现成(`flutter_markdown_plus`/`gpt_markdown`;官方包已停更) |
| Kanban(列 + 卡拖拽) | 现成(dnd-kit 等) | 种子(DnD 原语齐;无看板件;aviary 内嵌) | 种子(iced_drop 看板示例) | 种子(`appflowy_board` AGPL/MPL、`drag_and_drop_lists`,均停更 ≥13 月) |
| DAG Workflow(节点 + 边 + 自动布局) | 现成(React Flow + dagre) | 种子(gpui-flow MIT 单日项目、ferrum-flow Apache 76★、gpui-whiteboard;**无 dagre 级布局**;canvas 无三次贝塞尔) | 种子(iced_nodegraph,仅 wgpu,无布局) | 种子偏现成(`graphview` **有 Sugiyama 分层布局** + pan/zoom;拖拽重排未证实) |
| Terminal(VT + PTY + 渲染 + 选区/链接/IME) | 现成(xterm.js 全包) | 引擎现成(`alacritty_terminal` Apache 含 ConPTY;`libghostty-vt` 需 Zig、API 未稳);渲染层种子:`gpui_xterm` 0.1.1(MIT,~97 KB Rust,alacritty 0.25)、`gpui-terminal`(MIT/Apache,无选区无滚动)、tty7 内嵌(Apache)、Paneflow(GPL,libghostty-vt)(GPL:Zed `terminal` 269 KB + `terminal_view` 412 KB,其中 `terminal_element.rs` 114 KB 是渲染器) | 种子(`iced_term` 0.8;cosmic-term GPL) | 种子(`xterm.dart` 停更且新版打不了字 #207;`ghostty_vte_flutter` 0.1.3 + `portable_pty`,Alera 在用) |
| 表格 / 树 | 现成 | 现成(Table/DataTable/Tree) | 现成 table;树自造(iced_aw 无) | 现成(DataTable 内置;树靠第三方) |
| 测试:模拟输入 / 无显示 CI / 截图 | 现成(Playwright/WebdriverIO) | 现成 / 现成 / 仅 macOS | 现成 / 现成 / 三平台(tiny-skia) | 现成 / 现成(widget test)/ golden 有字体差异;desktop 集成测试 Linux 需 xvfb |
| 方法论公共度 | 完全公共 | 一半(Tailwind 链式公共;Entity/Context 私有) | 公共(Elm) | 公共(声明式 widget 树,80k 仓库,官方 AI rules) |
| 用 TS 描述 UI | 原生 | 社区 gpuix(单人,Linux/Win 未验证) | 无 | 无(Dart;与 Rust 控制面是第二语言) |

## 目标环境专项:NVIDIA + Wayland(GNOME)

| 路线 | 已知状态 | 回退手段 |
| --- | --- | --- |
| Tauri / WebKitGTK | 未解决,上游 WONTFIX,2026-08 仍有报告 | 三级环境变量,代价是禁用加速 |
| Electron / Chromium | 38.2+ 默认原生 Wayland;NVIDIA 黑屏仅一条弱证据 | `--ozone-platform=x11` 走 XWayland;Chromium 是 Linux 上测试量最大的 GPU 栈 |
| GPUI / wgpu | #52944(S1 open,GL/EGL panic)、#62998(S1 open)、GNOME 50 卡顿 open | **无自动回退**;手动 `WAYLAND_DISPLAY=""`;llvmpipe 只是最后兜底且有 Wayland 不可见 bug |
| Iced / wgpu | 无 iced 特有 open;field exposure 主要在 COSMIC 下 | 自动 wgpu→tiny-skia;`ICED_BACKEND` |
| Flutter / GTK3 + OpenGL | #188966 Wayland + NVIDIA 黑屏(open,`GDK_BACKEND=x11` 无效);Linux 固定 60 Hz 无 vsync(#191245 open);FPS 随窗口尺寸下降(#191425 open) | **无自动回退**(#177548);`FLUTTER_LINUX_RENDERER=software` "not recommended";NVIDIA PRIME offload 环境变量 |

四条路线在 Linux 上走的是**三条互不相通的 GPU 路径**:Flutter 与 Tauri 的窗口都是 GTK3,但 Flutter 用 GDK 建的 EGL/OpenGL 上下文渲染,WebKitGTK 在独立 WebProcess 里走 DMA-BUF;GPUI 不用 GTK,自己做 Wayland 客户端,由 wgpu 同时探测 Vulkan 与 GL(#52944 就是 GL/EGL 探测在 NVIDIA 上 panic,与最终选 Vulkan 无关);Iced 是 winit + wgpu,与 GPUI 同族但有 tiny-skia 兜底。**一条路线在目标机上顺畅,不能推出另一条也顺畅**;各自的零安装预测器是 App Center/Security Center(Flutter)、Zed(GPUI)、Epiphany(WebKitGTK)。

fcitx5 在 GNOME Wayland 下须经 IBus 前端、候选框不能盖在 gnome-shell UI 上——这是 GNOME 的限制,对三条路线一视同仁,但原生路线要自己把候选框位置报对(text-input-v3 cursor rectangle),Web 壳由引擎代劳。

## 同口径探针(本机,2026-08-26)

Apple M4 / 16 GiB / macOS 26.6.2 / rustc 1.98.0;GPUI/Iced 探针时无 Xcode(仅 CLT),Flutter 探针时已装 Xcode 26.6;无 sccache。空窗 + 文本 + 计数器,release 干净构建,运行 6 秒读 `ps` 与 `footprint`。

| 指标 | GPUI 0.2.2(`runtime_shaders`) | Iced 0.14.0(默认特性) | Flutter 3.47.1(默认,universal) | Electron 43.4.0(既有探针,隐藏空窗) |
| --- | --- | --- | --- | --- |
| 工具链 | Rust(已有) | Rust(已有) | SDK **3.91 GiB** + pub-cache 522 MiB + CocoaPods/ruby 183 MiB(cask 安装 730 s,网络受限);macOS 桌面构建需完整 Xcode | 无 |
| 干净 release 构建 | 76.9–81.6 s,451 crates,540 包 | 35.4 s,154 crates,193 包 | **18.5 s**(首次 22.2 s);`flutter create` 1.6 s | n/a |
| 构建产物目录 | `target/release` 1.2 GB | 494 MB | `build/` 764 MB | n/a |
| 二进制 / 包 | 5.2 MiB(strip 后 4.0 MiB) | 10.4 MiB(strip 后 7.9 MiB) | `.app` **36.5 MiB**(FlutterMacOS.framework 29.5 MiB + App.framework 6.6 MiB,均 x86_64+arm64 双架构;arm64 slice 约 14.6 + 3.2 MiB) | ZIP 116.5 MiB;`.app` 275.9 MiB |
| 空载 RSS | ~74 MB | ~96 MB | ~107 MB | 四进程直加 ~346 MB(重复计共享页) |
| 空载 physical footprint | **42.5 MB** | **42.1 MB** | **54.7–55.9 MB** | **77.6 MB** |
| 启动峰值 footprint | 95–103 MB | 138 MB | 175–184 MB | 未测 |
| 首帧 | 未测 | 未测 | **96 ms**(`--trace-startup`,profile) | 未测 |
| 测试回路 | `gpui::test`(未在探针中跑) | `iced_test`(未在探针中跑) | widget test 冷 6.1 s / 热 **1.2 s**;`flutter analyze` 4.2 s;`dart format` 0.3 s | n/a |

- GPUI 0.2.2 默认特性在无 Xcode 的 Mac 上**构建失败**(`build.rs` 调 `xcrun metal` 预编译 shader);开 `runtime_shaders` 即可,是 Cargo 开关不是系统安装。
- 三者 API 都按各自文档一次编译通过;Iced 0.13→0.14 的 `application()` 签名变化需从源码注释取(crates.io 包不带示例源码)。Flutter 无插件项目走 SwiftPM,CocoaPods 只是为了让 `flutter doctor` 全绿。
- Flutter 空载 footprint 比 GPUI/Iced 高 ~13 MB(Impeller/Metal 的 IOSurface 与图形内存),比 Electron 低 ~22 MB;磁盘在 GPUI/Iced(4–8 MiB)与 Electron(276 MiB)之间,单架构约 20 MiB。
- **磁盘差距 GPUI:Electron ~70×,空载内存差距 ~2×;真实 Workbench(十个终端、富文本、DAG)的差距未测**,不能把空壳值当整套产品结论。Electron "慢"(冷启动)本轮无数据,Flutter 96 ms 首帧是唯一有的原生冷启动数字。

## 许可证:Workbench GPL 化的账

所有者提议:`hctl2-workbench` 改 GPL,以便直接复用 GPL 代码(依赖或抄源码皆可);CLI 承担全部治理动作,Workbench "只是好看"。

**法律与架构上成立。**
- Apache-2.0 代码可以进 GPL-3.0 作品,反向不行。Zed 本身就是按 crate 分别标 Apache/GPL 的先例;做法是 `src/workbench/` 自带 `LICENSE-GPL`、crate `license = "GPL-3.0-or-later"`,仓库根 LICENSE 不变。
- 与现行设计吻合:Workbench 已被定义为"控制面的薄客户端",只走 Query/Preview/Submit/Subscribe 命令服务,不直接碰 tmux/PTY/Git/DB;`hctl2-control`/`hctl2-tool`/agentd 保持 Apache。仓库已经把 GPL 的 Dagu、AGPL 的 Vikunja 当独立进程依赖,"独立程序 + 明确协议"的边界是既有立场。
- 需要立的规矩:**共享代码只允许 Apache → GPL 方向流动**。Workbench 用到的通用件(命令类型、客户端 SDK、可能复用到 CLI 的 TUI/格式化)必须放在 Apache crate 里再被 Workbench 链接;在 Workbench 内写出的东西若日后想搬回 control,只有全部作者(所有者/其代理)同意才能改许可,外部贡献者的代码就搬不回来。
- 代价要说清:GPL-3 与 Mac App Store 分发被普遍认为不兼容(直接下载不受影响);未来若想卖闭源 Workbench 或授权第三方嵌入,GPL 关上这扇门;`-or-later` 与否要定。

**工程上买到的是"抄"的自由,不是"依赖"的自由。**
- Zed 的 GPL crate 都不发布到 crates.io,且互相咬死:`terminal_view` 依赖 `editor`、`project`、`workspace`、`settings`、`theme`、`ui`、`db`、`language`;`workspace` 又依赖 `client`、`fs`、`git`、`remote`、`sqlez`、`telemetry`、`node_runtime`。作为依赖引入 = 把 Zed 单体仓库整个编进来(Zed 二进制 274 MiB),不可取。
- 可抄的实体(大小为源码字节):`terminal` 269 KB + `terminal_view` 412 KB(`terminal_element.rs` 114 KB 渲染器、`terminal_view.rs` 117 KB、`terminal_panel.rs` 127 KB)——这是 GPL 化最有价值的一块,能拿到选区、超链接、搜索、IME 在终端内的处理、滚动条等打磨;`agent_ui/completion_provider.rs` 的 `@`/`/` 解析逻辑(小,可抄);`markdown` 440 KB(pulldown-cmark → gpui,但 gpui-component 已有 Apache 版);`agent_ui` 整体 2.4 MB **绑死 GPL `editor`,抄不动**。
- Iced 路线下 GPL 也有对应收益:Halloy 的聊天面板/补全 overlay、cosmic-term 的自绘终端。
- **不 GPL 也能走的路**:terminal 已有 MIT/Apache 种子(`gpui_xterm`、`gpui-terminal`、tty7 内嵌实现、`alacritty_terminal` 引擎),差距在打磨而非可行性;`@` 补全可用 gpui-component 的 `CompletionProvider` 自写。

**建议**:GPL 化作为"选定 GPUI 且决定抄 Zed 终端"的配套决定,而不是独立先做;若探针阶段用 `gpui_xterm`/tty7 就够用,则不必动许可证。无论怎样,先把"Apache → GPL 单向流动"写进 delivery.md 的边界条款。

## 推荐与探针清单

**推荐**:主候选 GPUI + gpui-component,Flutter 并列进入同一轮探针;备选 Iced;安全网 Electron(现行决定不动,直到探针通过)。理由:HCTL Workbench 的形态(多面板 dock + 多终端 + 长聊天流 + 表格)与 Zed/tty7/Arbor/waku 同形,GPUI 在虚拟列表、dock、代码编辑器上有现成 Apache 件;Flutter 在 AI/TDD、无障碍、DAG 布局上补齐了 GPUI 的短板,同形应用(Alera、AppFlowy、FluffyChat)也真实存在,但 Linux 面的三个 open(NVIDIA 黑屏、固定 60 Hz 无 vsync、FPS 随窗口尺寸下降)直接打在目标机和"常驻大窗口"这个使用形态上,只能实测定夺;Iced 的优势(Elm、tiny-skia 回退、三平台截图测试)真实但不覆盖 HCTL 最重的面。两条原生路线在目标机上的探针成本都低(GPUI 跑 gpui-component gallery,Flutter 跑一个 `flutter create` 的桌面应用),应同轮并排跑。

**一次性丢弃探针(在 Ubuntu NVIDIA/Wayland 机上跑,任一失败即保留 Electron)**:
1. **依赖钉法**:选定 Zed main 某 rev(或 `gpui-unofficial` 某 tag)+ gpui-component 对应 rev,记录 Cargo.lock;测干净构建时间(Linux/CI)与 sccache 后的增量时间。
2. **目标机渲染**:gpui-component 的 story gallery 在 GNOME Wayland + NVIDIA 当前驱动下运行 30 分钟,记录是否触发 #52944/#62998/#53522;同时测 `WAYLAND_DISPLAY=""` 回退与 llvmpipe 兜底是否可见。
3. **IME**:fcitx5(经 IBus)与 ibus 直连下,在 gpui-component Textarea 与 Editor 模式各输入中文,核候选框位置、预编辑、Esc 取消;同样在 macOS(拼音/Squirrel)和 Windows(微软拼音)各测一次。
4. **@ 提示**:验证 `CompletionProvider::is_completion_trigger` 能否挂在聊天用 Textarea(而非仅 Editor);不能则测自制 Popover 的可行性。
5. **Terminal**:用 `gpui_xterm` 或 tty7 的实现接 tmux control mode,跑 CJK 输出、一窗十终端的 CPU/GPU/footprint、resize、OSC 52、粘贴;记录与 xterm.js 的功能差(选区、链接、搜索、滚动条)。
6. **DAG**:用 ferrum-flow 或 gpui-flow 画 30 节点 DAG,自动布局临时用任一 Rust 分层布局 crate(待选,未核实),核 canvas 曲线与拖拽。
7. **测试回路**:为一个面板写 `#[gpui::test]`(模拟点击/按键 → 断言实体状态),在无显示 CI 跑通三平台;记录 AI 代理在钉定 rev 下生成代码的一次通过率(对照 gpui-component 自带 `skills/`)。
8. **Windows**:同一代码在 Windows 构建、启动、IME、终端(ConPTY)各过一遍。
9. **整窗探针**(与现行 E-WORKBENCH-SHELL 同口径):installer/安装后磁盘/冷热启动/idle footprint/一窗十终端 footprint,各路线并排。
10. **Flutter 专项**(与 2–8 同机同轮):a)`flutter create --platforms=linux` 的空应用在 GNOME Wayland + NVIDIA 当前驱动下能否起窗(#188966),记录是否需要 `__NV_PRIME_RENDER_OFFLOAD`/`__GLX_VENDOR_LIBRARY_NAME`;b)把窗口拉到全屏/4K,测滚动一个 5000 行 `ListView.builder` 的实际 FPS(#191245/#191425);c)用所有者的输入法在 `TextField` 输入含 emoji/生僻字的中文(#190046 的代理对崩溃);d)`ghostty_vte_flutter` + `portable_pty` 接 tmux control mode,CJK 输出、一窗十终端;e)`graphview` Sugiyama 布局 30 节点 DAG + 节点拖拽;f)`docking` 拖拽分栏;g)widget test 在无显示 CI 跑通三平台;h)Linux 包体(`flutter build linux` 产物 + GTK3 运行时依赖)。

### 明日 Ubuntu 实测清单(所有者自测,每项 15–30 分钟)

先采集机器画像,附在结果里(缺了这些数字,任何结论都不可复现):

```bash
lsb_release -ds; uname -r; echo $XDG_SESSION_TYPE; gnome-shell --version
nvidia-smi --query-gpu=name,driver_version --format=csv,noheader
lspci | grep -i -E 'vga|3d'            # 有没有第二块(核显)——判断是否 Optimus 形态
glxinfo -B 2>/dev/null | grep -E 'OpenGL (vendor|renderer|version)'   # 无则 sudo apt install mesa-utils
vulkaninfo --summary 2>/dev/null | grep -E 'GPU id|deviceName|driverName'  # 无则 apt install vulkan-tools
echo "IM: $GTK_IM_MODULE / $QT_IM_MODULE / $XMODIFIERS"; pgrep -l -E 'fcitx5|ibus-daemon'
```

每条路线都记同一组结果:能否起窗 / 是否需要环境变量 / 拖到全屏后 resize 与滚动是否流畅 / 在文本框里用中文输入法能否正常预编辑与上屏 / 退出有无崩溃日志。

1. **Electron(对照组,零安装)**:任何已装的 Electron 应用(VS Code、Cursor、Multica)全屏跑一会;`ps -o args -p <pid> | grep ozone` 看是否原生 Wayland;若黑屏,试 `--ozone-platform=x11`。
2. **Flutter(零安装数据点 + 正式探针)**:先看已装的 App Center(`snap list | grep snap-store`,24.04 起是 Flutter)——全屏、快速滚动、切到 GNOME Wayland 分数缩放看字体/布局;再正式的:`sudo snap install flutter --classic`(或官方 tar),`sudo apt install clang cmake ninja-build pkg-config libgtk-3-dev`,`flutter doctor -v`(会打印 Linux GPU 驱动信息),`flutter create --platforms=linux hello && cd hello && flutter run -d linux --release`;然后把窗口拉到 4K/全屏,在 `lib/main.dart` 里换成 5000 行 `ListView.builder` 快速滚动观察掉帧;在 `TextField` 用输入法输入含 emoji 和生僻字的中文(#190046)。黑屏时依次试 `__NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia`、`GDK_BACKEND=x11`、`FLUTTER_LINUX_RENDERER=software`,记录哪个有效。
3. **GPUI(零安装数据点 + 正式探针)**:先装 Zed 1.16(`curl -f https://zed.dev/install.sh | sh`)——它就是 GPUI 在这台机器上的最大规模实证:全屏、开终端面板跑 `htop`、在编辑器和 Agent Panel 输入中文、`ZED_LOG=wgpu=info zed --foreground` 看选中的适配器与后端(Vulkan 还是 GL,是否踩 #52944);再 `git clone https://github.com/longbridge/gpui-component && cd gpui-component && cargo run`(按 README 跑 gallery;需 `libxkbcommon-dev libwayland-dev libvulkan-dev` 之类,README 有清单),重点看 Textarea/Editor 的中文输入与 Dock 拖拽。故障时试 `WAYLAND_DISPLAY="" cargo run`(走 X11)与 `__NV_FORCE_ENABLE_X11_EGL_PLATFORM=1`。
4. **Tauri 2(零安装数据点 + 正式探针)**:先 `sudo apt install epiphany-browser`(GNOME Web 就是 WebKitGTK),开一个 WebGL 页与 contenteditable 输中文——它踩的是 WebProcess 的 DMA-BUF 路径,与 App Center 的 GTK3+EGL 不是同一条路,Flutter 顺畅推不出这里顺畅;再正式的:`sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`,`npm create tauri-app@latest`(选 vanilla TS 即可),`npm run tauri dev`;起窗后在页面里打开一个 WebGL 页(xterm.js 的 WebGL 渲染器依赖它)与一个 contenteditable 输入中文;黑屏或 Error 71 时依次试 `__NV_DISABLE_EXPLICIT_SYNC=1`、`WEBKIT_DISABLE_DMABUF_RENDERER=1`、`WEBKIT_DISABLE_COMPOSITING_MODE=1`,记录哪一级才正常以及正常后滚动是否变慢。
5. **(可选)Iced**:`git clone https://github.com/squidowl/halloy && cargo run --release`(Halloy 是 Iced 上最像聊天室的应用),看 pane 拖拽与中文输入;`ICED_BACKEND=tiny-skia` 对照。

结果直接回填到本备忘"目标环境专项"表,并把机器画像写进"未核实项"里那条"Ubuntu 机器具体版本"下。

**若探针通过,需要动的正文**:E-WORKBENCH-SHELL 当前决定与重开门槛;delivery.md 技术栈一行(Electron + React 19 / Tiptap / React Aria / React Flow / xterm.js → GPUI + gpui-component + 终端/DAG 方案)与"React 场景代码保持普通浏览器可运行"的约束(GPUI 路线下该约束失效,需改写为"场景代码不依赖 Workbench 壳的特权");decision-history 记一节;若 GPL 化,delivery.md 加许可证边界条款。**这是 UI 栈的整体更换,不只是换壳**——四个场景的前端实现全部重写,Tiptap/React Flow/xterm.js 三个已选轮子全部放弃。

## 未核实项

- GPUI/wgpu 是否在 NVIDIA Wayland 上协商 explicit sync;#52944 在 Zed 1.16 是否仍复现。
- gpui-component 的 Linux/Wayland IME 已由附录 A7 实机核实；Windows IME 仍未实测，`CompletionProvider` 能否挂 Textarea 仍未核实。
- gpui-component WebView 在 Linux 的官方支持声明(仅见示例 TODO)。
- Rust 侧是否有 dagre 级分层布局 crate 可直接配 gpui/iced(未搜)。
- Electron 冷启动时间与最终真实 Workbench 场景的内存仍未核实；附录 A7 已补一窗 10 个 xterm.js WebGL terminal + 长列表的中间负载数据。
- gpuix 在 Linux/Windows 的实际运行状态(README 自述未验证)。
- `gpui-ghostty` 0.0.1(2026-08-04)仓库已 404;`gpui_xterm` 功能完整度(选区/滚动/IME)未运行验证。
- libcosmic 在 macOS/Windows 的 2026 构建状态。
- Iced 的 NVIDIA + GNOME Wayland 实机表现(无 issue ≠ 无问题)。
- Flutter:Impeller 在 Linux 的官方"可用性"表(Google Sheet 不可抓取),"OpenGL 后端"结论来自 PR #189584 的自述;`graphview` 节点拖拽后能否重排;`xterm.dart` 的 OSC 8 链接与选区;`super_editor`/`flutter_quill` 的桌面 IME 可靠性;AppFlowy 空载 RSS;FluffyChat 是否有非 CI 的 Windows/macOS 构建;本机 Flutter `.app` 为双架构,单架构数字是按 slice 估算。

## 主要证据

- Tauri:https://v2.tauri.app/develop/debug/linux-graphics/ · https://v2.tauri.app/reference/webview-versions/ · https://github.com/tauri-apps/tauri/issues/10702 · /issues/14924 · /issues/9394 · /issues/15665 · /issues/15902 · /issues/11412 · /issues/14963(维护者 2026-02-17 状态)· https://bugs.webkit.org/show_bug.cgi?id=262607 · https://github.com/versotile-org/verso(归档)
- Electron:https://github.com/electron/electron/releases/tag/v44.0.0 · https://github.com/electron/electron/blob/main/docs/breaking-changes.md(§38)· https://github.com/electron/electron/pull/48301 · https://www.electronjs.org/blog/tech-talk-wayland · https://github.com/microsoft/vscode/issues/280464
- GPUI:https://crates.io/crates/gpui · https://github.com/zed-industries/zed/pull/49277(拆 gpui_platform)· /pull/46758(wgpu 渲染器)· /pull/56065(AccessKit)· /issues/52944 · /issues/62998 · /issues/53522 · /issues/54959 · /issues/59882 · https://github.com/zed-industries/zed/blob/main/docs/src/linux.md · https://github.com/zed-industries/zed/blob/main/crates/gpui/README.md · https://zed.dev/blog/zed-1-0 · https://github.com/iamnbutler/gpui-unofficial · https://github.com/zed-industries/awesome-gpui
- gpui-component:https://github.com/longbridge/gpui-component(Cargo.toml git 依赖;`crates/base/src/input/base/state.rs` IME 实现;`crates/base/src/input/editor/lsp/completions.rs`;`examples/webview/src/main.rs` Linux TODO)
- 终端:https://github.com/alacritty/alacritty/tree/master/alacritty_terminal · https://libghostty.tip.ghostty.org/ · https://github.com/Uzaaft/libghostty-rs · https://crates.io/crates/libghostty-vt · https://paneflow.dev/blog/libghostty-linux · https://github.com/arthjean/paneflow · https://github.com/ghostty-org/ghostling · https://crates.io/crates/gpui_xterm · https://github.com/zortax/gpui-terminal · https://github.com/l0ng-ai/tty7 · https://github.com/zed-industries/zed/tree/main/crates/terminal_view
- DAG / 看板:https://github.com/pacifio/gpui-flow · https://github.com/tu6ge/ferrum-flow · https://github.com/Catvert/aviary · https://github.com/jhannyj/iced_drop · https://github.com/tuco86/iced_nodegraph
- gpuix:https://github.com/remorses/gpuix · https://www.npmjs.com/package/@gpuix/native
- Iced:https://crates.io/crates/iced · https://github.com/iced-rs/iced/releases/tag/0.14.0 · /pull/2777(IME)· /pull/2698、/pull/3059(测试)· /issues/552、/pull/3281(无障碍)· /issues/160、/issues/2603、/issues/3429(列表)· /issues/3138 · https://github.com/iced-rs/iced/compare/0.14.0...master · https://github.com/iced-rs/iced_aw · https://github.com/Harzu/iced_term · https://github.com/squidowl/halloy · https://github.com/pop-os/libcosmic · https://github.com/gfx-rs/wgpu/issues/7475
- 同类工具:https://github.com/multica-ai/multica/releases · https://github.com/yetone/cumora · https://github.com/stablyai/orca · https://github.com/superset-sh/superset · https://github.com/xintaofei/codeg/releases · https://github.com/warpdotdev/warp/issues/9383
- Flutter:https://flutter.dev/blog/whats-new-in-flutter-3-44(Canonical 主维护)· https://flutter.dev/blog/whats-new-in-flutter-3-47 · https://docs.flutter.dev/reference/supported-platforms · https://docs.flutter.dev/perf/impeller · https://github.com/flutter/flutter/pull/187573(Linux Impeller 默认)· /pull/189584(Vulkan,open)· /issues/188966(NVIDIA Wayland 黑屏)· /issues/177548(无自动回退)· /issues/191245(无 vsync)· /issues/191425(FPS 随尺寸)· /issues/127768(fractional scaling)· /issues/94804(GTK4)· /issues/190046(IME crash)· /issues/182876(IMM32)· /issues/177586(多窗口清单)· /pull/191389(Wayland 子表面)· https://docs.flutter.dev/testing/integration-tests · https://docs.flutter.dev/ai/ai-rules · https://docs.flutter.dev/ai/mcp-server · https://pub.dev/packages/docking · https://pub.dev/packages/graphview · https://pub.dev/packages/xterm · https://github.com/TerminalStudio/xterm.dart/issues/207 · https://pub.dev/packages/ghostty_vte_flutter · https://pub.dev/packages/appflowy_board · https://pub.dev/packages/flutter_markdown(停更声明)· https://github.com/flutter/flutter/issues/162960 · https://github.com/leynier/alera · https://github.com/AppFlowy-IO/AppFlowy/releases/latest · https://github.com/krille-chan/fluffychat/releases/latest · https://github.com/krille-chan/fluffychat/blob/main/lib/pages/chat/input_bar.dart · https://www.phoronix.com/news/GTK3-Annual-Release-Cadence · https://lwn.net/Articles/996147/(Flock)
- 本机探针文件:scratchpad `gpui-probe/`(`gpui-hello`、`iced-hello`、`measure.sh`、构建日志)、`flutter-probe/`(`flutter_hello`、`doctor-*.log`、`build-*.log`、`launch-*.stderr`、`trace-startup.log`)。

## 拍板记录（2026-08-30）

所有者拍板:Workbench 桌面壳正式选型改为 **Tauri 2 + TS/React 主选,GPUI 原生备选,Electron 安全网**,Flutter 等上游修复后再评。落点:[decision-history §30](../../design/references/decision-history.md#30-workbench-桌面壳改选-tauri-2v0142)(v0.14.2)、[E-WORKBENCH-SHELL 复核记录](../workbench-shell.md#e-workbench-shell)、delivery.md 技术基线与系统边界安全条款。本备忘使命完成,后续只追加勘误。

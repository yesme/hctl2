# 附录 A7 · Ubuntu NVIDIA/Wayland 实机探针记录（2026-08-26）

> 主备忘：`README.md`。本记录回填主备忘要求的 Ubuntu 物理机一次性探针，测试时仓库基线为 main @ `314f6fc`。探针源码、构建目录与原始日志位于本机磁盘暂存目录 `~/tmp/hctl2-probes/`，不作为产品源码提交；本文件保留可复核的版本、结果、测量口径、产物摘要和上游反馈链接。

## 结论

1. **Tauri 2 不能再按调研阶段的上游 issue 推断“在本机可以判死”。** Tauri 2.11.5 + WebKitGTK 2.52.3 在本机原生 Wayland/NVIDIA 路径成功运行 Canvas 2D、WebGL、10 个 xterm.js WebGL terminal、长列表滚动和中文 IME。候选窗最初缺失，但应用把前端光标矩形通知给 WebKitGTK `InputMethodContext` 后，所有者确认候选窗与输入均正常。
2. **GPUI 通过本机可用性门槛。** gpui-component Gallery 在原生 Wayland 上使用 Quadro P620/Vulkan，输入、滚动、再次输入和 IME 均正常；release 构建和源码目录外独立运行成功。它仍有生态、开发成本和与 TS/React 舒适度之间的产品取舍，不是兼容性淘汰。
3. **Flutter 暂时淘汰。** 普通窗口、滚动和 IME 正常，但默认应用一最大化就稳定 SIGSEGV；人工双击、自动最大化、Wayland、X11 均可复现。问题已提交为 [flutter/flutter#191775](https://github.com/flutter/flutter/issues/191775)。
4. **Electron 可用但较重。** 功能探针通过；原生 Wayland 下窗口顶边 1 px 持续闪动，去掉 `WaylandWindowDecorations` 仍存在，强制 XWayland 后消失。此机采用 Electron 时应默认 XWayland，不必关闭硬件加速。
5. **当前工程建议是 Tauri 2 + TS/React 优先，GPUI 为原生备选，Electron 为兼容安全网，Flutter 等上游修复后再评估。** 这是讨论备忘的实测建议，不自动修改正式技术栈决定。

## 机器画像

| 项目 | 值 |
|---|---|
| OS | Ubuntu 26.04 LTS (Resolute Raccoon), Linux 7.0.0-30-generic, x86_64 |
| Desktop | GNOME Shell 50.1，原生 Wayland 会话 |
| 显示器 | Dell P3223QE，3840×2160 @ 59.997 Hz，GNOME scale 1.5 |
| Mutter | `scale-monitor-framebuffer`、`xwayland-native-scaling` 已启用 |
| GPU | NVIDIA Quadro P620 2 GiB |
| Driver | 580.173.02 |
| GTK / WebKitGTK | GTK 3.24.52；WebKitGTK 2.52.3 (`webkit2gtk-4.1`) |
| Rust | rustc/cargo 1.98.0 |
| Node / npm | Node 22.22.1；npm 9.2.0 |
| 已知本机背景 | Chrome 原生 Wayland + 硬件加速曾出现 Canvas 绘制破裂，因此所有 Web 探针同时做像素读回和人工目测，不能只看“窗口打开” |

`/tmp` 是 16 GiB tmpfs；完整探针在构建中迁移到磁盘目录 `/home/jackywang/tmp/hctl2-probes`，避免 GPUI/Flutter 构建缓存耗尽内存文件系统。

## 统一 Web 重载探针与内存口径

Tauri 2 和 Electron 使用同一份 TypeScript/Vite 负载：

- Canvas 2D：固定 RGB 色块、渐变、曲线和中英文文本，读取精确像素并计算 checksum；
- WebGL：编译 shader、绘制固定颜色并 `readPixels`；
- terminal：10 个 xterm.js 6.0 + WebGL addon 实例；
- scrolling：5000 行列表，连续 300 帧滚动并记录 FPS、p95 frame time；
- IME：顶部单行输入、底部单行输入、`textarea`、`contenteditable`；
- 人工检查：Canvas 是否破裂、窗口最大化、滚动、中文组合输入、候选窗与光标附近定位。

负载结束并稳定约 10 秒后，从应用根进程递归取进程树；每个进程读取 `/proc/<pid>/smaps_rollup`：

- **RSS 求和**直观但会重复计算共享页；
- **PSS 求和**按共享比例分摊，更接近该应用实际占用的物理内存；
- GPUI 测的是官方 Gallery 默认页，不是相同的 10-terminal Web 负载，因此只能比较量级，不能把三列当严格 benchmark 排名。

## Tauri 2

### 版本和构建

| 项目 | 值 |
|---|---|
| Tauri crate | 2.11.5 |
| Tauri CLI | 2.11.4 |
| wry | 0.55.1 |
| WebKitGTK | 2.52.3 |
| 前端 | TypeScript + Vite 6 + xterm.js 6；探针未引入 React，验证的是壳/WebView/IME 路径 |
| Release | Rust release 二进制和 `.deb` 均构建成功 |

### 渲染、滚动和 GPU

结构化结果：

- Canvas 2D PASS，RGB 读回分别为 `[239,68,68,255]`、`[34,197,94,255]`、`[59,130,246,255]`，checksum `52621a3b`；
- WebGL PASS，中心像素 `[51,178,229,255]`；
- xterm.js WebGL 10/10；
- 5000 行、300 帧滚动 PASS，本轮 59.23 FPS，p95 21 ms；
- 所有者人工确认 Canvas 没有出现本机 Chrome 的绘制破裂；
- 原生 Wayland 起窗，系统 GPU 进程观测确认 NVIDIA 路径在工作。

WebKit 的 `WEBGL_debug_renderer_info` 在此环境返回泛化的 `Apple GPU` 字符串，不能拿该字符串判断真实显卡；真实 GPU 以系统进程观测为准。

### IME

初始 Tauri 2/WebKitGTK 页面可以输入组合文字，但中文候选窗完全不可见。修正分两部分：

1. Linux setup 取得 WebKitGTK `InputMethodContext` 并关闭 WebKit 自己的 preedit 绘制；
2. 前端在 focus/click/input/composition/selection/scroll/resize 时计算当前 editable/caret 的 viewport 矩形，通过 Tauri command 调用：

```rust
input_context.notify_cursor_area(x, y, width, height);
```

所有者随后在顶部单行、底部单行、`textarea` 和 `contenteditable` 中确认：字母组合串可见、中文候选窗可见、选字和最终输入均正常。

剩余问题是 WebKitGTK 的组合下划线：把光标放在已有文本之前开始组合时，下划线可能延伸到后续无关字符或行末。三种 editable 都能复现，候选窗和最终输入不受影响；判断为 WebKitGTK 组合串绘制细节，不应在 hctl2 里叠加 DOM 特判。所有者接受该取舍。

候选窗定位的复现与应用侧修正已反馈到 [tauri-apps/tauri#11412 的评论](https://github.com/tauri-apps/tauri/issues/11412#issuecomment-5420366006)，供 Tauri/wry 上游考虑把光标矩形传递做成通用修复。

## GPUI / gpui-component

| 项目 | 值 |
|---|---|
| gpui-component source | `26849e063f53a5d1c18a25d42bc6b34f75720176` |
| Zed/GPUI git rev | `8b1497dbd22fb06f5838a7c0b84a1e54fafa71bc` |
| Gallery crate | `gpui-component-story` 0.5.1 |
| Release command | `CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build --release` |
| Clean release wall | 8 min 05 s |
| Raw release ELF | 116,986,304 B，未 strip |
| Stripped ELF | 93,292,432 B |
| tar.gz | 23,463,515 B |

Gallery 在原生 Wayland 上起窗，`nvidia-smi pmon` 将进程标为 Quadro P620 上的 `C+G` 客户端。所有者完成输入、滚动、再次输入和 IME 检查，均正常。资源由 `RustEmbed` 内嵌；stripped 二进制复制到源码目录之外仍可直接运行。

本探针验证的是 GPUI/gpui-component 在目标机上的构建、原生 Wayland/GPU、基础控件和 IME 可用性，没有验证最终需要的多 terminal、Kanban 和 DAG 组件完整度。

## Flutter

| 项目 | 值 |
|---|---|
| Flutter | 3.47.1 stable，framework `6655482ec0` |
| Engine | `5d53178869` |
| Dart | 3.13.1 |
| Renderer | Linux 默认 Impeller `OpenGLESSDF` |

Linux toolchain doctor 通过，默认 `flutter create --platforms=linux` 应用和重载探针均能 release 构建。普通窗口下输入、中文 IME 候选窗、滚动、再次输入均正常。

最大化稳定崩溃：

- 所有者多次双击标题栏复现“闪烁后进程退出”；
- 默认最小应用在首帧后 1 秒调用 maximize 同样退出 139；
- `GDK_BACKEND=wayland` 与 `GDK_BACKEND=x11` 均复现；
- 日志先出现两次 `eglMakeCurrent failed`，随后 SIGSEGV；
- GDB 主线程栈落在 `gdk_window_end_draw_frame()` / `gtk_main_do_event()`，另有线程在 `libEGL_nvidia.so.0`；
- `FLUTTER_LINUX_RENDERER=software` 不是可用回退，会因 `Impeller DlText cannot be drawn to a Skia canvas` 直接 fatal。

最小复现、环境、Wayland/X11 对照、自动 maximize patch、完整 GDB 栈已提交到 [flutter/flutter#191775](https://github.com/flutter/flutter/issues/191775)。这与已关闭的 #190059 相似，但旧修复已存在于 3.47.1，而当前默认应用仍在不同的 GDK draw-frame 路径崩溃。

## Electron

| 项目 | 值 |
|---|---|
| Electron | 44.0.0 |
| Chromium | 152.0.7977.54 |
| 前端 | TypeScript + Vite 7 + xterm.js 6 |
| 构建 | `npm run build:linux`；AppImage 和 `.deb` 成功 |

自动探针结果：

- Canvas 2D、WebGL、xterm.js WebGL 10/10、5000 行/300 帧滚动全部 PASS；
- XWayland 本轮滚动 51.30 FPS，p95 16.8 ms；
- ANGLE 明确报告 Quadro P620，`nvidia-smi` 同时确认 GPU process；
- 所有者确认窗口、最大化、Canvas、滚动和输入没有功能性问题。

显示瑕疵 A/B：

1. `--ozone-platform=wayland --enable-features=WaylandWindowDecorations`：窗口最上方 1 px 持续跳动；
2. 原生 Wayland 去掉 `WaylandWindowDecorations`：仍跳动；
3. `--ozone-platform=x11`：不再跳动。

因此把问题限定为 Electron 44/Chromium 152 的原生 Ozone Wayland 路径与本机 NVIDIA/150% scale 组合；hctl2 若采用 Electron，应对该环境默认 XWayland。

未安装目录和 AppImage extract-and-run 需要 `--no-sandbox`，原因是 Ubuntu 24+ AppArmor 限制 unprivileged user namespace，且未安装的 `chrome-sandbox` 没有 setuid。生成的 `.deb` post-install 已包含 electron-builder 提供的 Ubuntu 24+ AppArmor profile 安装和 sandbox 权限设置；本轮没有 sudo 安装 `.deb`，只核验了包内容和脚本。AppImage 直接运行还缺本机 `libfuse.so.2`，`APPIMAGE_EXTRACT_AND_RUN=1` 可运行。

## 包大小和稳定态内存

| 方案 | 发行包 | 解包/安装载荷 | RSS 求和 | PSS 求和 | 测量负载 |
|---|---:|---:|---:|---:|---|
| Tauri 2.11.5 | `.deb` 2.80 MiB | 二进制 10.55 MiB；Installed-Size 10,810 KiB | 607,068 KiB（592.84 MiB） | 290,765 KiB（283.95 MiB） | 统一 Web 重载探针 |
| GPUI Gallery | `.tar.gz` 22.38 MiB | stripped ELF 88.97 MiB | 289,092 KiB（282.32 MiB） | 209,888 KiB（204.97 MiB） | Gallery 默认页，非同负载 |
| Electron 44 / XWayland | `.deb` 95.69 MiB | 290.47 MiB | 784,964 KiB（766.57 MiB） | 380,634 KiB（371.71 MiB） | 统一 Web 重载探针 |

补充数据：Electron 原生 Wayland 同负载为 RSS 839,500 KiB（819.82 MiB）、PSS 421,413 KiB（411.54 MiB）；AppImage 为 120.93 MiB。

包大小也不能脱离依赖模型解释：Tauri `.deb` 依赖系统 WebKitGTK/GTK，GPUI ELF 依赖常见系统动态库，Electron 把 Chromium/V8 一并携带。对最终 hctl2 安装器，应另外计算从干净目标系统出发的依赖闭包；本表回答的是当前本地可运行产物自身。

进程分解（KiB）：

| 方案 | 进程 | RSS | PSS |
|---|---|---:|---:|
| Tauri 2 | `tauri-probe` | 231,280 | 94,041 |
| Tauri 2 | `WebKitNetworkProcess` | 52,460 | 12,584 |
| Tauri 2 | `WebKitWebProcess` | 323,328 | 184,140 |
| Electron/X11 | browser | 165,808 | 77,476 |
| Electron/X11 | zygote ×2 | 117,744 | 26,785 |
| Electron/X11 | GPU process | 231,228 | 119,799 |
| Electron/X11 | network utility | 77,416 | 24,783 |
| Electron/X11 | renderer | 192,768 | 131,791 |
| GPUI | `gpui-component-story` | 289,092 | 209,888 |

## 本地产物与校验值

这些路径是本机验证证据，不是仓库或正式 release 的稳定路径：

| 产物 | 路径 | SHA-256 |
|---|---|---|
| Tauri 2 `.deb` | `~/tmp/hctl2-probes/tauri2/tauri-probe/src-tauri/target/release/bundle/deb/tauri-probe_0.1.0_amd64.deb` | `76c6f8c9bb6b5b3c0062c536fa1d28da9fda50db6a20f3aca647e7f6de10ab94` |
| GPUI tar.gz | `~/tmp/hctl2-probes/gpui-component/gpui-component/artifacts/gpui-component-story-0.5.1-linux-x86_64.tar.gz` | `9b01f14c00d0733a4a1db4d6cad05098e4da2addf39279fbd62af0f9ef6fb3c6` |
| Electron `.deb` | `~/tmp/hctl2-probes/electron/release/hctl2-electron-probe_0.1.0_amd64.deb` | `4631c44dd045cc752e7c48644a4e3a14a0df99a80d6e3c64006b044ca0bbf0dd` |
| Electron AppImage | `~/tmp/hctl2-probes/electron/release/HCTL2 Electron Probe-0.1.0.AppImage` | `e6e6994aef54791efdb759d98c569513c9cc6a5190a2839ac3df072e701f2d7c` |

Flutter 上游提交正文留在 `~/tmp/hctl2-probes/flutter-3.47.1/flutter-resize-issue.md`，GDB 原始日志留在同目录的 `gdb-auto-maximize-wayland.log`。

## 对正式决定的含义

本轮回答了“这些壳能否在所有者的 Ubuntu NVIDIA/Wayland 物理机上构建并实际操作”，没有替代 Workbench 组件生态、跨平台打包、维护成本和许可证的完整选择。

- Tauri 2 的本机 Linux 阻断已解除，并保留 TS/React、xterm.js、React Flow、Tiptap 等公共 Web 方法论；IME 候选窗修正需要在正式原型中产品化并补自动回归。
- GPUI 的 Linux/GPU/IME 阻断已解除，但 terminal、DAG、Kanban 和 AI 对快速变化 API 的开发效率仍需算总账。
- Electron 是确认可工作的安全网，但当前探针中包体与 PSS 均高于 Tauri 2；本机原生 Wayland 还需 XWayland 回退。
- Flutter 在最基本的窗口生命周期上有可复现崩溃，不进入下一轮 Workbench coding，等待上游 issue 进展。

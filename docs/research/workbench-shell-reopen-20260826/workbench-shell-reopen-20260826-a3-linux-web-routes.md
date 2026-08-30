# 附录 A3 · Linux 上的 Web 壳路线调研原始报告(2026-08-26)

> 主备忘:`README.md`。调研代理原始英文报告。范围:TS/React UI(xterm.js、Tiptap CJK/@、React Flow、kanban)在 macOS + Ubuntu(物理机、NVIDIA、GNOME Wayland)+ Windows 上的壳选择。"[secondary]" 为非一手来源,"[unverified]" 为未核实。

## 1. Tauri 2 on Linux (mid-2026)

Latest releases (GitHub, 2026-08-26): `tauri-v2.11.5` 2026-07-01; `tauri-cli-v2.11.4` 2026-06-28; `tauri-runtime-wry-v2.11.4` 2026-06-30. A user's `tauri info` (2026-08-10) shows wry 0.56.0 / tao 0.36.0 (issue #15859).

WebKitGTK (`libwebkit2gtk-4.1-0`) on Ubuntu (packages.ubuntu.com, 2026-08-26): jammy 22.04 = 2.50.4-0ubuntu0.22.04.1; noble 24.04 = 2.52.3-0ubuntu0.24.04.1; resolute 26.04 = 2.52.3-0ubuntu0.26.04.3; dev series = 2.52.4-1ubuntu1. Upstream stable 2.52.6 (2026-08-19). Tauri prerequisites still require `libwebkit2gtk-4.1-dev` (GTK3).

Official statement (https://v2.tauri.app/reference/webview-versions/): "Tauri uses WebKit on macOS (through WKWebView) and Linux (through webkit2gtk)." … "The diverse nature of the Linux ecosystem means it is very hard to compile accurate information about WebKitGTK on the various distros. The table below is a very incomplete list… You should always check your distro's repositories." (Table is stale: lists Ubuntu 22.04 as 2.36.)

Bundling a fixed WebKitGTK: not supported. FabianLars 2023-12-11: static linking WebKitGTK "is super hard and at this point i doubt it's really possible with our current stack" (discussion #8367). 2024-06-10: AppImage "will include all dependencies it needs… you'll have to build on the oldest system you want the app to run on"; "that's the glibc problem… Really one of the biggest problems we have with linux" (#10026). **#15665** (open, 2026-07-07): "AppImages from default bundler settings fail on Mesa 25+ distros" — WebKitWebProcess aborts on Ubuntu 26.04 because the AppImage over-bundles libwayland/glib/gstreamer. PR #12491 "Truly portable appimage (experimental)" still open.

Wayland vs X11 forcing: AppImage GTK hook exported `GDK_BACKEND=x11` unconditionally. #15781 (opened 2026-07-26, closed 2026-07-27) fixed by PR #15786 "respect configured GDK backend". **#15902** (open, 2026-08-20): AppImage "still hard-crashes with no XWayland when GDK_BACKEND is unset". Non-AppImage (.deb) builds run native Wayland through GTK3.

NVIDIA + Wayland + DMA-BUF:
- Official "Linux Graphics Issues" page (updated 2026-06-15): "On some setups, most often NVIDIA GPUs, WebKitGTK and the graphics driver disagree and you get anything from a blank window to subtle rendering problems." Order: `__NV_DISABLE_EXPLICIT_SYNC=1` ("often fixes the Wayland Error 71 crash without a performance cost"), then `WEBKIT_DISABLE_DMABUF_RENDERER=1` ("at the cost of the faster rendering path"), then `WEBKIT_DISABLE_COMPOSITING_MODE=1` ("Last resort… disables accelerated compositing entirely"). Setting them unconditionally "disables a faster path for everyone" — https://v2.tauri.app/develop/debug/linux-graphics/
- **#10702** "Error 71 (Protocol error) dispatching to Wayland display" — OPEN, opened 2024-08-20, 50 comments, `status: upstream`, last activity 2026-08-24. Reports span webkit2gtk 2.44.3 → 2.46.5 (Arch, GNOME Wayland, 2025-01-22) → 2.48.3 (2025-07-18) → **2.52.5 with NVIDIA 580.173.02 (2026-08-07)**. 2025-10-16: `WEBKIT_DISABLE_DMABUF_RENDERER=1` "works, but is causing really slow animations"; `__NV_DISABLE_EXPLICIT_SYNC=1` smoother but "display buffer glitches". Root cause traced to GTK3 (gitlab GNOME/gtk work item 8056); 2026-08-24 app-level build workaround claimed on RTX 3080/2060/GTX 1080.
- **#14924** "Linux/Nvidia: Crash (GBM/Error 71) or Visual Artifacts… with Transparent Windows" — OPEN, 2026-02-10, NVIDIA 590+, `transparent: true` crashes.
- **#9394** "[docs] Documenting Nvidia problems in Tauri" — OPEN since 2024-04-06.
- Did WebKitGTK 2.4x fix it? No evidence. WebKit bug 262607 "[GTK] Disable DMABuf renderer for NVIDIA proprietary drivers" — RESOLVED WONTFIX (opened 2023-10-04, last modified 2024-09-18). Ubuntu bug 2041664 duplicate/incomplete with same workaround. 2.52.0 release notes mention nothing on NVIDIA/DMA-BUF. Reports with 2.52.5 (2026-08) show the workaround still needed.
- wry #1366 "Wry cannot create windows on Arch Linux with Nvidia" — open (last update 2024-09-23).

Performance: #7761 "Window freeze with WebGL" — open, `status: upstream`, 2023-09-06; workaround `WEBKIT_DISABLE_COMPOSITING_MODE=1`. #8989 "expose control over webkit settings" open. Third-party marlin#23 "Linux scrolling lag on WebKitGTK" [secondary]. wry #1727 (May 2026) text blurry after resize on Wayland with WebKit2GTK 2.50.6. #14286 font-weight offset open. No xterm.js-specific issue found [unverified].

IME / CJK: #8264 (fcitx5 Debian 12 KDE/Wayland candidate window detached) closed 2026-05-06 (fixed in wry 0.29 / Tauri v2). **#11412** "IME window position appears out of input/textarea (cannot inline-input) on Tauri v2 apps in Linux" — OPEN since 2024-10-19, fcitx/mozc and ibus/mozc. **#15859** compose key ignored on KDE Wayland (webkit2gtk 2.52.5, Tauri 2.11.5) — OPEN, `status: upstream`. No open "ime" issues in wry.

Transparency / decorations: #14924; #12955 decoration regression (KDE); discussion #15371 frame extents on Wayland; #15656 child webview bounds wrong on Ubuntu 26.04 + WebKitGTK 2.52.3 (open).

Maintainer status (FabianLars, 2026-02-17, #14963): CEF branch "currently more like source available than open source"; Servo/Verso "further development is halted for now since we're missing funding and/or manpower"; "looking at qtwebengine as a replacement for webkitgtk" (wusyong/cxx-qt-widgets), paused.

## 2. Tauri + Servo (Verso)

- versotile-org/verso: ARCHIVED 2025-10-08, 5,392 stars. README: "Verso is currently no longer maintained", recommends Servoshell.
- tauri-runtime-verso: `versotile-org/tauri-runtime-verso` (133 stars, last commit 2025-10-03; engine bump "versoview 0.0.9" 2025-09-27). Build script downloads a pre-built `versoview` binary as `externalBin` ("pre-built versoview for x64 Linux, Windows, MacOS and arm64 MacOS"); Linux binary "requires a more recent version of glibc". Bundles the engine (no WebKitGTK dependency), but frozen since Oct 2025.
- Tauri blog "Experimental Tauri Verso Integration" (2025-03-17): "experimental", "only a subset of windowing/webview features supported". NLnet grant completed (Aug 2022–Mar 2024).
- Servo 2026: v0.3.0 (2026-06-25), v0.4.0 (2026-08-04), LTS v0.1.2/v0.1.3. June 2026 report: `WebView::rendering_context()`, "designing a wrapper C API" for prebuilt shared library, experimental WebGPU, compat work (lichess, Google Photos), "interactive features remain incomplete on Google Maps"; donations $7,681/mo; no production-readiness claim.

## 3. Bundled-Chromium / lighter alternatives keeping TS/React

| Option | Engine on Linux | Windows / macOS | Coverage & maturity | License | Solves WebKitGTK? |
|---|---|---|---|---|---|
| Tauri `feat/cef` | Bundled CEF `cef = "=151.1.0"`, features `["build-util","linux-x11"]` (X11 → XWayland on Wayland) | CEF | Branch active (last commit 2026-08-25); "source available", "you're a bit on your own"; CEF "not shared across apps"; #15718 "[cef] Black screen… with transparency" OPEN (2026-07-15); a user migrated back 2026-08-23 ("wasn't a smooth experience") | MIT/Apache | Yes (bundled), but Linux X11-only and unreleased |
| tauri-apps/cef-rs | CEF bindings; Linux/macOS/Windows x86_64 + ARM64 | same | `cef-v151.8.0+151.3.24` 2026-08-23, 449 stars | Apache-2.0/MIT | Library only |
| CEF upstream Wayland | Embedded Ozone/Wayland windows still open since 2019-11-13 (cef#2804, updated 2026-08-04) | — | — | BSD | CEF-on-Linux = X11/XWayland today |
| Electrobun (Bun/TS) | **WebKitGTK 4.1 + GTK3** by default; optional `bundleCEF`; `bundleWGPU` | Windows x64 WebView2; macOS ARM64 WKWebView | v2.0.1 2026-08-22, 12,695 stars; official: macOS 14+, Windows 11+, Ubuntu 24.04+ | MIT | Default repeats WebKitGTK; CEF opt-in |
| Neutralinojs | gtk-webkit2 | WebView2 / WebKit | v6.9.0 2026-07-24, 8,616 stars | MIT | No |
| Wails v3 (Go) | WebKitGTK: GTK4 + WebKitGTK 6.0 experimental `-tags gtk4` (issue #4957 closed 2026-04-29); later docs say 6.0 default with `-tags gtk3` for webkit2gtk-4.1 distros [unverified verbatim] | WebView2 / WKWebView | v3.0.0-beta.13 2026-08-25 (prerelease) | MIT | No |
| Electron | Bundled Chromium | Bundled | v44.0.0 2026-08-25, Chromium 152.0.7977.54, Node 24.18.1, V8 15.2; `electron-v44.0.0-darwin-arm64.zip` = 129,743,965 B (~123.7 MiB); v44 removed 32-bit builds, Unity DE, macOS 12; ANGLE statically linked | MIT | Yes |

Electron slim/lite: none official; Performance tutorial only lists app-level advice. `electron-lite` on npm is unrelated (2019). Tauri+CEF "~170MB added to the app" on macOS (2026-05-22) [secondary].

Electron/Chromium on Wayland + NVIDIA:
- Electron 38 breaking change: "The default value of the `--ozone-platform` flag changed to `auto`… Electron now defaults to running as a native Wayland app when launched in a Wayland session (when `XDG_SESSION_TYPE=wayland`). Users can force XWayland by passing `--ozone-platform=x11`"; `ELECTRON_OZONE_PLATFORM_HINT` removed. Fix PR #48301 merged 2025-09-12, backport to 38-x-y 2025-09-25. Electron blog (2026-03-17): "Wayland is supported out of the box in Electron 38.2 and newer". #41551 "support Wayland by default" closed 2025-12-12. Chromium 140 shipped `--ozone-platform-hint=auto` (Aug 2025).
- NVIDIA+Wayland issues: microsoft/vscode **#280464** "VS Code Insiders blank screen on NVIDIA + Wayland after Electron 38 update" — OPEN, 2025-12-01, 1 comment. electron #49247 "OSR paint events do not fire in an NVIDIA GPU + Wayland environment" — closed for inactivity 2026-06-13. #36633 "Zero GPU Acceleration on Wayland" closed 2026-03-27. #51941 grey screen (Electron 41/42) open, not NVIDIA-confirmed. #53050 fractional-DPR input mis-scale on Wayland open 2026-08-20. Historic #30878 (Electron 13) closed 2021.

## 4. Non-web declarative UI with TS/React syntax, native rendering

- React Native desktop: react-native-macos v0.81.8 (2026-06-26); react-native-windows 0.84.0 (2026-06-30), Fabric-only since 0.82. **No Linux target** (out-of-tree platform list: Windows, macOS, visionOS, OpenHarmony, tvOS, Web, Skia). react-native-skia platform repo last pushed 2023-06-12; react-native-gtk "in its infancy".
- Lynx (ByteDance): Lynx 3.7 (2026-04-16) "officially supports macOS and Windows"; maintainer 2025-06-20: "the support for actual UI on Linux is yet to be planned". Lynxtron (Electron-like shell) 27 stars.
- NodeGui / React NodeGui: nodegui v0.74.2 (2026-05-03), Qt6, 9,226 stars; "Works on major Linux flavours and Windows. Help is requested to bring it to ARM based MacOS". **react-nodegui last pushed 2023-11-03**.
- Valence Native: npm 0.0.1, 2022-05-23.
- Conclusion: RN has no Linux platform, Lynx has no planned Linux UI, NodeGui's React layer unmaintained since Nov 2023.

## 5. Flutter (one paragraph; superseded by appendix A5)

Flutter 3.44 (2026-05-20) announced Canonical as "lead maintainer and Strategic Steward for Flutter Desktop"; multi-window experimental (main channel). Linux IME: umbrella #66880 closed 2020; **#190046** "[Linux] App aborts… when typing with an IME — DeleteSurrounding splits UTF-16 surrogate pairs" open (updated 2026-08-19). Terminal: `xterm` 4.0.0 (2024-02-27), repo last push 2025-06-19. No LLM/Dart benchmark found.

## 6. What comparable "agent desktop" tools ship on Linux

| Tool | Shell | Linux | Evidence |
|---|---|---|---|
| Cursor | VS Code fork / Electron [secondary] | AppImage, .deb, .rpm (x64 + ARM64), 3.17 | cursor.com/downloads |
| Windsurf (Devin Desktop) | VS Code fork / Electron [secondary] | Linux download page exists | windsurf.com/editor/download-linux |
| Zed | Rust + GPUI | Yes; 1.0 on 2026-04-29 for Linux/macOS/Windows; Linux renderer on wgpu (PR #46758, 2026-02-13) | zed.dev/blog/zed-1-0 |
| Warp | Rust, own UI framework (wgpu, winit, cosmic-text), now AGPL-3.0 (64.5k stars) | Linux since 2024-02-22 | warp.dev/blog/warp-for-linux; IME risk: #9383 "IME never enabled on Linux/Wayland" (opened 2026-04-29, closed via PR #11032) |
| Ghostty | Zig; Linux GTK4 (+libadwaita), macOS AppKit/SwiftUI; no Windows | Yes | ghostty.org/docs/about |
| Conductor | Tauri 2.6.2, WKWebView [secondary] | macOS only | conductor.build |
| Multica | Electron `^39.2.6` | v0.4.34 (2026-08-25): .deb/.rpm/.AppImage x86_64 + arm64 | multica-ai/multica/releases |
| Cumora | `yetone/cumora`, Electron `^33.4.11` | v0.3.0 (2026-08-25): AppImage + amd64 .deb (+ mac dmg, Windows exe) | yetone/cumora-releases |
| Rakazo | `elie222/rakazo`, Electron `^43.4.0`, Apache-2.0 | v0.1.0-beta (2026-08-13) has no release assets | |
| Superset | Electron 41.10.3 + `@xterm/xterm 6.1.0-beta.289`; ELv2 | x64 AppImage "experimental"; Windows "not yet available"; canary 1.24.2 2026-08-25 | superset-sh/superset |
| Codeg | Tauri 2 (`src-tauri`), Apache-2.0 | v0.28.1 (2026-08-24): AppImage amd64, .deb amd64/arm64, .rpm | xintaofei/codeg/releases |
| first-tree | No desktop shell (`cli`, `doc-website`) | n/a | first-tree-ai/first-tree |
| Orca ADE | Electron `^43.1.0` + `@xterm/xterm 6.1.0-beta.287`; MIT; 53,469 stars, created 2026-03-17 | AppImage + AUR | stablyai/orca |
| Helio | not found by this line (documented separately in [E-HELIO](../workbench/helio.md#e-helio)) | — | |

## Claims not verified

Helio in this search; Windsurf's current Linux package formats; Conductor's Tauri stack (third-party); Wails v3 Linux page verbatim (403); xterm.js on Verso/Servo; any production Servo embedder; WebKitGTK 2.52.x NEWS on NVIDIA; Tauri `feat/cef` Linux bundle size; Rakazo shipped Linux binary; LLM proficiency Dart vs TS; Electron v44 Linux RSS/disk.

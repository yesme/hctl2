# 附录 A1 · GPUI 本体调研原始报告(2026-08-26)

> 主备忘:`workbench-shell-reopen-20260826.md`。本文为调研代理的原始英文报告,经主线复核后保留;主备忘引用的事实以此为据。"gpui on main" 指 `zed-industries/zed` 当日 `main`,"gpui 0.2.2" 指 crates.io 发布版;两者已明显分叉(§1)。

## 1. Release / packaging status

crates.io versions of `gpui` (https://crates.io/crates/gpui/versions):

| Version | Published | Note |
|---|---|---|
| 0.2.2 | 2025-10-22 | latest; Apache-2.0; publisher mikayla-maki; crate size 5.29 MB |
| 0.2.1 | 2025-10-14 | |
| 0.2.0 | 2025-10-09 | announced by Zed: https://x.com/zeddotdev/status/1976309201744937039 |
| 0.1.0 | 2022-06-23 | yanked; MIT |

- Cadence: three releases in one week (Oct 9–22, 2025), then no release in the following ~10 months. Total downloads 229,530.
- Stability: 0.x. README on main: "GPUI is still in active development as we work on the Zed code editor, and is still pre-1.0. There will often be breaking changes between versions. You'll also need to use the latest version of stable Rust." — https://github.com/zed-industries/zed/blob/main/crates/gpui/README.md
- Concrete API break between 0.2.2 and main: 0.2.2 hello-world starts with `Application::new().run(...)`; main requires `gpui_platform::application().run(...)` because the platform layer was extracted in PR #49277 "gpui: Extract gpui_platform out of gpui", 2026-02-19 (https://github.com/zed-industries/zed/pull/49277). Renderers are now in `gpui_wgpu`/`gpui_apple`, backends in `gpui_linux`/`gpui_macos`/`gpui_windows`/`gpui_web`. The web target landed in PR #50228 "GPUI on the web", 2026-02-26; `rust-toolchain.toml` lists `wasm32-unknown-unknown # gpui on the web`.
- Usable as a normal Cargo dependency? `gpui = "0.2.2"` from crates.io works standalone (still contains the old in-crate platform code; verified by building it). It depends on Zed-forked crates on crates.io (`zed-font-kit 0.14.1-zed`, `zed-xim 0.4.0-zed`, `zed-scap 0.0.8-zed`, `gpui_util`, `gpui_collections`, …). **Current main is not consumable from crates.io:** root `Cargo.toml` has `[workspace.package] publish = false`; `gpui_platform`, `gpui_linux`, `gpui_macos`, `gpui_windows`, `gpui_web`, `gpui_wgpu`, `gpui_tokio` use `publish.workspace = true` and return 404 on crates.io. `crates/gpui/Cargo.toml` on main still says `version = "0.2.2"`. Anything newer than Oct-2025 requires git dependencies. gpui-component's workspace uses `gpui = { version = "0.2.2", git = "https://github.com/zed-industries/zed" }` and `gpui_platform = { git = ... }`.
- Third-party republishing: **gpui-unofficial** ("publishes gpui releases on zed release tags"): 38 versions since 2026-04-03, latest `1.16.2` on 2026-08-24 — https://github.com/iamnbutler/gpui-unofficial, https://crates.io/crates/gpui-unofficial. **gpui-ce** 0.3.3 (2025-12-27) — https://github.com/gpui-ce/gpui-ce. **wgpui** 0.3.4 (2026-08-19), "Independent wgpu + winit UI framework (community fork of GPUI / GPUI-CE 0.3.3)" — https://crates.io/crates/wgpui.
- Docs: https://www.gpui.rs is a single landing page (hello-world, README, crate-root rustdoc, `docs/contexts.md`, `docs/key_dispatch.md`, examples list). No book. "for the near future gpui is tied to Zed, so contributions will need to be made there and kept in sync with it." README: "Currently, the best way to learn about these APIs is to read the Zed source code or drop a question in the Zed Discord." docs.rs built 0.2.2 successfully. In-repo docs: `crates/gpui/docs/{contexts.md,key_dispatch.md}` plus guide modules `_ownership_and_data_flow.rs` and `_accessibility.rs`.
- MSRV / toolchain: no `rust-version` field; `edition = "2024"` (Rust ≥ 1.85); README says latest stable; Zed's `rust-toolchain.toml` pins `1.97.1`.
- Build time (measured 2026-08-26): clean `cargo build --release` of the 0.2.2 `hello_world.rs` example, `gpui = { version = "0.2.2", features = ["runtime_shaders"] }`, Apple M4, macOS 26.6.2, rustc 1.98.0: **66 s wall / 504 s CPU, 451 crates, binary 5.5 MB**. Without `runtime_shaders` the build fails unless full Xcode is installed: `xcrun: error: unable to find utility "metal"`. Third-party (dev.to, 2025-01-01): "Compilation takes 10+ minutes" first build, ~4 MB release binary.

## 2. License

Repo root has `LICENSE-APACHE` and `LICENSE-GPL` (no AGPL file). Per-crate `license =` on main:

- Apache-2.0: `gpui`, `gpui_macros`, `gpui_platform`, `gpui_wgpu`, `gpui_web`, `gpui_linux`, `gpui_macos`, `gpui_windows`, `gpui_apple`, `gpui_tokio`, `http_client`, `scheduler`, `collections`, `util`, `sum_tree`, `extension_api` (`gpui_util`, `gpui_shared_string` have no license line).
- GPL-3.0-or-later: `ui`, `ui_input`, `editor`, `terminal`, `terminal_view`, `project`, `workspace`, `theme`, `zed`, `language`, `multi_buffer`, `settings`, `fs`, `text`, `rope`, `client`, `agent_ui`, `title_bar`, `picker`, `menu`, `component`, `sqlez`, `db`, `feature_flags`, `telemetry`, `rpc`, `proto`, `extension`, `lsp`, `collab`.
- No AGPL crate found. Implication for an Apache-2.0 consumer: only the `gpui*` family + a few utility crates are Apache; everything widget-like (`ui`, `ui_input`), editor, terminal, workspace/dock/panels, theme, settings is GPL.
- gpui-component (Longbridge) is Apache-2.0.

## 3. Platform backends

**macOS** — Metal renderer in `crates/gpui_apple`; windowing in `gpui_macos`. README: "Rendering uses Metal and is always available, but glyph rasterization needs `font-kit`." macOS 12–26 supported, 10.15/11 partially (docs/src/installation.md).

**Linux** — `gpui_linux` with `wayland` and `x11` features (default both). Renderer: **wgpu** since PR #46758 "gpui: Remove blade, reimplement linux renderer with wgpu" merged 2026-02-13 (blade port dated to PR #7343, 2024-02-07). wgpu instance: `backends: wgpu::Backends::VULKAN | wgpu::Backends::GL` (`crates/gpui_wgpu/src/wgpu_context.rs`). Adapter selection ranks compositor-GPU match first, then DiscreteGpu > IntegratedGpu > Other > VirtualGpu > Cpu (llvmpipe); CPU adapters are only skipped when `reject_software` is set (device-loss recovery path only). Zed docs: "have a Vulkan compatible GPU available"; "If you see `Zed failed to open a window: NoSupportedDeviceFound` this means that Vulkan cannot find a compatible GPU"; env `ZED_DEVICE_ID`, `MESA_VK_DEVICE_SELECT`, `DRI_PRIME`; log `ZED_LOG=wgpu=info` — docs/src/linux.md. Text: cosmic-text 0.19 + zed-font-kit. `gpui_linux/src/linux/headless.rs` provides a `HeadlessClient` (used when `ZED_HEADLESS` is set or no display).

NVIDIA / Wayland (target machine):
- #35948 "Hang after drawing first frame with NVIDIA 580 drivers before 580.82.07 on Linux Wayland" (opened 2025-08-10, closed 2025-09-08, `meta:upstream`; fixed by NVIDIA 580.82.07; workaround was XWayland).
- #39097 "Zed freeze when running on NVIDIA GPU" (blade `surface is out of date`; closed 2026-02-23 as obsolete after the wgpu switch).
- **#52944 "Zed crashes the host on Linux with NVIDIA GPU since 0.230.0" (open, S1, `graphics:nvidia`, opened 2026-04-02).** Root cause per thread: wgpu 29 GL/EGL backend panics in `khronos-egl` on NVIDIA (`EGL_EXT_platform_xcb` handle mismatch); a GNOME 50 Wayland + RTX 5090 + driver 595.71 user reports the same; workaround `__NV_FORCE_ENABLE_X11_EGL_PLATFORM=1 zed` (2026-05-06). Marked stale 2026-08-07. PRs #55525 and #62300 "gpui_wgpu: Don't crash when a GPU backend panics during init" not merged.
- **#62998 "Linux/X11: WgpuAtlas index-out-of-bounds crash after GPU device-loss recovery" (open, S1, NVIDIA GTX 1650 Ti, driver 580, opened 2026-08-21).**
- #60303 "Characters disappear and reappear…" (open, `graphics:nvidia`, 2026-07-02). Only 2 open `graphics:nvidia` issues today (#52944, #60303).
- GNOME Wayland (not NVIDIA-specific): #53522 "Sluggishness in GNOME 50 / Wayland" (open, 39 comments, `state:needs research`; workaround `WAYLAND_DISPLAY=""`); #59397 busy cursor persists ~15 s (open); #62621 amdgpu MODE2 reset kills the Wayland session (open, AMD); #55550 visual artifacts on Wayland closed 2026-05-06 via PR #54214.
- Explicit sync: no `linux_drm_syncobj` or `wp_fifo` references in the zed repo; whether wgpu 29 negotiates it — not verified.
- Fallback: `guess_compositor()` returns `"Headless"` if `ZED_HEADLESS`, else `"Wayland"` if `WAYLAND_DISPLAY` non-empty, else `"X11"` if `DISPLAY` set. **No automatic Wayland→X11 fallback on GPU-init failure**; documented manual fallback `WAYLAND_DISPLAY="" zed`. llvmpipe bug #52062 "Window invisible on Wayland with llvmpipe + Mesa 26.0" closed by stale bot 2026-07-31 without fix.

**Windows** — `gpui_windows`: Direct3D 11 (feature levels 11.1/11.0/10.1), DirectWrite, Win32. Zed blog "Windows When? Windows Now", 2025-10-15. Docs: "Zed requires a DirectX 11 compatible GPU". Zed 1.0.0 shipped 2026-04-29; latest stable 1.16.2, 2026-08-24, with `Zed-x86_64.exe` and `Zed-aarch64.exe`.

**Headless / CI** — Zed CI runs `cargo nextest run --workspace` on Ubuntu, Windows and macOS runners with no xvfb/DISPLAY setup; tests use `TestPlatform` (`crates/gpui/src/platform/test/`), which needs no display.

## 4. IME / CJK input

Implementation (main): macOS `NSTextInputClient` (`gpui_macos/src/window.rs`); Linux Wayland `zwp_text_input_v3` (`gpui_linux/src/linux/wayland/client.rs`, PR #11712 2024-05-16); Linux X11 XIM via `zed-xim` fork (PR #11657, CJK preedit fix PR #17373 2024-09-17); Windows **IMM32** (`WM_IME_COMPOSITION`, `ImmSetCompositionWindow`, `ImmSetCandidateWindow` in `gpui_windows/src/events.rs`), no TSF.

Issue landscape (`area:controls/ime`: 14 open; `area:internationalization`: 38 open; no umbrella tracking issue):
- Linux X11 + fcitx5 (open): #54959 "Chinese input with Fcitx5 fails in Zed while working in other apps" (Ubuntu GNOME X11; reproduced 2026-08-23); #58192 XIM `Can't read xim message: Invalid Data`; #59662 "`zed_xim` violates the XIM protocol sequence by sending `CreateIc` before `OpenReply`" (S2); #57131 fcitx5 stops working after 1.2.6; #52952 fcitx5 switching intermittently fails.
- Linux Wayland: #54975 "IME breaks when switching virtual desktops" (KDE, open); fixed recently: #61034 memory leak in pre-edit (PR #61079, 2026-07-17), #62084 candidate-window flicker with two windows (PR #62086, 2026-08-05), PR #58712 multi-window IME (2026-06-06), PR #37600 scaled-display IME position (2025-09-05), PR #55876 anchor candidate window to visual line (2026-06-03).
- GNOME/Mutter + fcitx5/ibus: Fcitx wiki: GNOME "uses text-input-v3" but "Compositor/Input Method uses ibus dbus protocol, so ibus frontend is required"; fcitx5 on autostart "will replace any existing ibus-daemon"; "Popup candidate window is not able to be displayed over gnome-shell UI. Only solution is to use Kimpanel" — https://fcitx-im.org/wiki/Using_Fcitx_5_on_Wayland. No zed issue specifically about GNOME-Wayland IME failure found.
- Windows (open): #56149 candidate window at top of screen in terminal; #61724 preedit width mismatch in terminal; #59193 text shifts vertically with Chinese IME; #59882 Chinese IME intermittently fails (S2); #59636 TSF GetTextExt caret rectangle. Closed: #42201 MS-IME (2026-01), #41656 RIME, #41223 KeyUp not sent to IME.
- macOS (open): #62661 IME composition fails in nonactivating panels (2026-08-15); #59464 "Cannot type until change language on Mac"; #61937 menu bar flashes with Chinese IME; #55835 Japanese IME menu bar regression; Squirrel (RIME) broke in Zed 1.10.0 (comment 2026-07-09 on #59882).

## 5. Accessibility

AccessKit integrated on main since PR #56065 "gpui: Accesskit support", merged 2026-05-27 ("ONLY adds AccessKit support to GPUI, and doesn't touch Zed"). Deps: `accesskit 0.24`, `accesskit_macos 0.26`, `accesskit_unix 0.21`, `accesskit_windows 0.33.1`. Guide `crates/gpui/src/_accessibility.rs`; internals `window/a11y.rs`; example `examples/a11y.rs`; `Application::inaccessible()` (PR #57954); author IDs (PR #61926, 2026-08-07); dev action "dump accessibility tree". Not in gpui 0.2.2. Adoption in Zed early: `.role(Role::` in 10 files; PRs "a11y: Settings UI" (2026-06-17), "a11y: Landmarks and menu improvements" (2026-07-13). #41138 "Windows: Screen reader accessibility missing completely" open (maintainer 2026-05-22: "#56065 is mostly working… core primitives are in place"). #7895 "Voice Over Support" closed 2025-12-19 without stated reason.

## 6. Testing story

- `#[gpui::test]` (`crates/gpui_macros/src/test.rs`): sync/async tests, injects `TestAppContext`s and a seeded `StdRng`; args `seed`, `seeds(..)`, `iterations`, `retries`, `on_failure`; env `SEED`, `ITERATIONS`; emits plain `#[test]`. Also `#[gpui::property_test]`, `#[gpui::bench]`.
- `TestAppContext`: `simulate_keystrokes`, `simulate_input`, `dispatch_keystroke`, `dispatch_action`, `simulate_window_resize`, `simulate_prompt_answer`, `simulate_new_path_selection`, `run_until_parked`, `advance_clock`; `VisualTestContext` adds `simulate_mouse_move/down/up`, `simulate_click`, `simulate_modifiers_change`, `simulate_resize`, `simulate_event`, `simulate_close`.
- Headless/offscreen: `HeadlessAppContext` ("cross-platform headless app context for tests that need real text shaping… optionally real GPU rendering and screenshot capture via `capture_screenshot`"); `VisualTestPlatform` ("real rendering (macOs-only for now)"); `visual_test_context.rs` `open_offscreen_window`, `capture_screenshot`. Zed's screenshot regression runner `crates/zed/src/visual_test_runner.rs` is macOS-only (Metal), baseline-image comparison, `UPDATE_BASELINE=1`.
- No DOM-like snapshot API; tests read entity state and drive input. `inspector` feature, `debug_a11y_tree_json`.
- 332 files in Zed contain `gpui::test`. Example `crates/gpui/examples/testing.rs`. No testing chapter in `docs/src/development/`.
- Downstream regression: #62510 "`TestWindow::window_handle` panics… breaking headless tests in dependent crates" (fixed PR #62775, 2026-08-17).

## 7. Primitives shipped vs not shipped

Shipped in `crates/gpui/src/elements/`: `div` (Taffy flex/grid, Tailwind-style; `taffy = 0.9.0`), `text`/`StyledText`/`InteractiveText`, `img`, `svg`, `canvas`, `list`, `uniform_list`, `anchored`, `deferred`, `animation`, `surface`, `container_query`, `image_cache`; plus `tab_stop`, gestures, `path_builder`, app menus, popups/layer-shell. Examples: `data_table.rs`, `tree.rs`, `popover.rs`, `scrollable.rs`, `drag_drop.rs`, `grid_layout.rs`, `tab_stop.rs`.

Not shipped: no text-input widget — only `EntityInputHandler`/`ElementInputHandler` traits (`crates/gpui/src/input.rs`); `examples/input.rs` hand-implements a `TextInput`. Zed's single-line field is `crates/ui_input` (GPL), the editor `crates/editor` (GPL). No table, tree, menu, tooltip, tabs, dock in gpui itself — Zed's are in `crates/ui` (GPL). Apache alternative: gpui-component.

## 8. Binary size / memory

- Zed binaries: #34376 "Huge Binary Size" (2025-07-13, v0.194.3): Linux `zed-editor` 274 MiB, macOS `zed` 277 MiB; closed `state:unactionable`. Discussion #28524: macOS binary >220 MB while the DMG <100 MB.
- Memory: #58182 (2026-05-31, Zed 1.3.6): Linux uses ~300 MiB more than Windows on a blank project; open. Third-party claims Zed idle ≈222 MB (methodology unknown).
- Minimal gpui app: 5.5 MB release binary (0.2.2 hello-world, macOS arm64). No official numbers.

## 9. Language bindings / declarative layers

- Zed: Rust only. Extensions cannot draw UI (languages, themes, debuggers, snippets, MCP servers, slash commands, icon themes, agent servers; `wasm32-wasip2`).
- Third-party: **GPUIX** "Node.js & React bindings for Zed GPUI" — React reconciler over napi-rs, `bun --hot chat.tsx`, examples for macOS/Windows and a WebGPU web build (needs nightly Rust); "Browser event callbacks are not supported yet" — https://github.com/remorses/gpuix (1,123 stars, pushed 2026-08-25). awesome-gpui also lists "React Native GPUI" (nucleus-os/nucleus, 52 stars), Crepuscularity (JSX/DSL → GPUI), gpui-hooks, declarative-gpui.
- Verdict: official surface Rust-only; a JS/React path exists only via community GPUIX.

## 10. Ecosystem adoption (non-Zed apps)

| Project | What | Stars | Last push | Multi-panel dev tool? |
|---|---|---|---|---|
| longbridge/gpui-component | 60+ components; "Used to build Longbridge Pro from day one" | 13,483 | 2026-08-25 | library; dock/tabs/table/editor |
| AprilNEA/OpenLogi | Logitech Options+ alternative | 16,326 | 2026-08-25 | no |
| vicanso/zedis | Redis GUI | 2,019 | 2026-08-23 | partly |
| 66HEX/frame | FFmpeg GUI (GPL-3.0) | 1,948 | 2026-08-24 | no |
| MatthiasGrandl/Loungy | launcher | 1,735 | 2025-10-05 | archived |
| AnalyseDeCircuit/oxideterm | AI-native workspace for shells/remote machines (GPL-3.0) | 1,341 | 2026-08-25 | yes |
| egoist/waku | native app for coding agents (GPL-3.0) | 1,217 | 2026-08-25 | yes |
| remorses/gpuix | React bindings | 1,123 | 2026-08-25 | n/a |
| penso/arbor | Git worktrees, terminals, diffs (MIT) | 806 | 2026-06-12 | yes |
| l0ng-ai/tty7 | terminal workbench, SSH, agents (Apache-2.0) | 780 | 2026-08-25 | yes |
| Auto-Explore/GitComet | Git UI (AGPL-3.0) | 768 | 2026-08-25 | yes |
| feigeCode/navop | DB/SSH/SFTP/terminal workspace | 736 | 2026-08-25 | yes |
| polachok/helix-gpui | Helix frontend | 537 | 2024-06-10 | dormant |
| zed-industries/create-gpui-app | scaffolder (official) | 377 | 2025-04-13 | dormant |

Also DB clients (pgui, DBFlux, dbui, zqlz, OpenMango), lgtm, hunk, termy/Ghostex/Nebula. GitHub code search ~30 non-Zed repos with `gpui` in `Cargo.toml` on the first page.

## Claims not verified

1. wgpu 29 explicit sync on NVIDIA Wayland. 2. Whether #52944 still reproduces on Zed 1.16.x. 3. GNOME-Wayland-specific IME bug in gpui — none found. 4. Official Zed statements on idle RSS / binary size. 5. Linux/Windows clean-build times. 6. Longbridge Pro built on gpui-component end-to-end. 7. Why #7895 closed. 8. Whether Zed plans another crates.io release of `gpui` / will publish `gpui_platform`.

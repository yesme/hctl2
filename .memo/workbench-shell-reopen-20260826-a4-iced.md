# 附录 A4 · Iced 调研原始报告(2026-08-26)

> 主备忘:`workbench-shell-reopen-20260826.md`。调研代理原始英文报告(GitHub/crates.io API + 仓库源码 + 网页);§12 含该代理在本机的独立测量,与附录 A6 的探针代理数字相互印证。

## 1. Release status

| Item | Fact |
|---|---|
| Latest crates.io `iced` | **0.14.0**, 2025-12-07; MIT; `rust_version` 1.88 |
| Release list (Aug 2024→Aug 2026) | 0.13.0 (2024-09-18), 0.13.1 (2024-09-19), 0.14.0 (2025-12-07) — 3 releases; 0.13.1→0.14.0 ≈ 15 months. Earlier: 0.12.0 2024-02-15, 0.12.1 2024-02-22, 0.10.0 2023-07-28 (no 0.11) |
| Sub-crate patches | `iced_widget` 0.14.1 (2025-12-08), 0.14.2 (2025-12-11); umbrella has no 0.14.x patch |
| Downloads | 2,586,673 total; 677,091 recent |
| 0.13 breaking | `Program` API (#2331), `Task` replacing `Command` (#2463), `Daemon` API (#2469), functional/closure styling (#2312, #2326), class-based theming (#2350), winit 0.30 (#2427), `rich_text`/`markdown`/`stack`/`hover` widgets. `Sandbox` removed without changelog entry (#2622, 2024-10-02) |
| 0.14 breaking/major | `Widget::update` takes `Event` by reference; `Overlay::is_over` removed; `color!` shorthand removed; Rust 2024 (#2809); wgpu 27 (#3097), cosmic-text 0.15 (#3098); new widgets `table` (#3018), `grid` (#2885), `sensor` (#2751), `float` (#2916), `pin` (#2673); "Input method support" (#2777); "Headless mode testing" (#2698); "First-class end-to-end testing" (#3059); "Time travel debugging" (#2910); "Hot reloading" (#3000); "Reactive rendering" (#2662) |
| MSRV | 0.14.0: 1.88. master: 1.92, `0.15.0-dev`, edition 2024 |
| master vs release | `compare 0.14.0...master`: ahead 336 commits, behind 3 (2026-08-26). master deps: `wgpu = "29"`, `cosmic-text = "0.19"`, `winit` = git fork `iced-rs/winit` rev `05b8ff17` (2025-09-15, Cargo 0.30.8), `cryoglyph` git rev |
| Apps on git master | Halloy: `iced` patched to `squidowl/iced` rev `8d458d2` (2026-08-01). hermes-chat: `iced = "0.15.0-dev"` git. libcosmic: git submodule `pop-os/iced` |
| Book | Chapters: Introduction, Philosophy, Architecture, First Steps, The Runtime, Text, Container, Additional Resources, FAQ. Unwritten: Layout, Styling, Concurrency, Structure, State, Laziness, Widgets, Subscriptions, Themes, Shells, Renderers. No testing chapter |
| Examples | 55 directories |

## 2. Rendering backends & Linux

- Features: `wgpu` ("Vulkan, Metal, DX12, OpenGL, WebGPU"), `wgpu-bare`, `tiny-skia` (software), `x11`, `wayland`. With both: `Renderer = fallback::Renderer<iced_wgpu::Renderer, iced_tiny_skia::Renderer>`; with none, release build fails.
- Runtime fallback: `fallback::Compositor::new` tries primary then secondary. Env `ICED_BACKEND` (`wgpu` / `tiny-skia`), `WGPU_BACKEND`, `wgpu::PowerPreference::from_env()`. History: software renderer + runtime fallback in 0.10 (#1748); type-driven fallback in 0.13 (#2351).
- winit: crates.io `iced_winit` 0.14.0 → `winit ^0.30`; winit latest 0.30.13 (2026-03-02), 0.31.0-beta.2. master uses fork. Wayland fractional scaling via `wp-fractional-scale` in winit (issue #3183). iced master (unreleased) 2026-06-30 "Notify `window::Event::Resized` when `scale_factor` changes", 2026-07-17 "Separate `window` from `application` scale factor hints". #2404 "App resolution is too high…" open since 2024-04-24 (X11).
- NVIDIA / Wayland log:

| Issue | Status |
|---|---|
| iced #2297 "GUI freezes upon interacting with a widget under Wayland" (NVIDIA) | 2024-02-26 → closed 2024-03-27 ("fixed with the 550.67 driver") |
| iced #2558 "Very high CPU usage on Wayland" (Radeon) | closed 2024-09-29 |
| iced #2750 "Laggy window resizing on Wayland WGPU" | closed 2025-11-08 ("solved on main") |
| iced #2878 cosmic-0.13 breaks font rendering on wayland+gnome 47 | closed 2025-04-15 |
| iced #3023 "[Wayland (Hyprland)] not a valid new object id" (AMD) | closed 2025-08-04 |
| iced #3143 "0.14.0 used dGPU instead of iGPU by default" | closed 2026-06-22 |
| iced #3138 "'gl' as the wgpu backend causes major graphical glitches" | **open** (2025-12-06) |
| iced #3229 "The last `Event::Closed` is missed in subscriptions on Wayland" | **open** (2026-01-29) |
| iced #3418 "Clipboard broken on wayland only clients on compositor w/o ext-data-control-v1" | **open** (2026-08-11) |
| iced #3317 "Taskbar takes 20s to bind the window after app starts (Linux)" | **open** (2026-04-27) |
| wgpu #4775 hang on Nvidia 545.29.06 GNOME/Wayland | closed, external driver bug |
| wgpu #7475 "ERROR_SURFACE_LOST_KHR on Wayland with Nvidia dedicated GPU plus AMD integrated GPU" | **open** (2025-04-03) |
| wgpu #8996 explicit-sync `Queue::add_wait_semaphore()` | open |

No iced-specific "black window"/"flicker" issue tied to NVIDIA explicit sync found. Multi-window since 0.12.0 (#1964); `iced::daemon`; `multi_window` example.

## 3. IME / CJK input

- **#979 "IME input method on iced application (fcitix on linux)"** opened 2021-08-02, closed 2025-02-04. PR #1474 "basic IME supporting" (2022-10) closed unmerged 2023-05. **PR #2777 "Input Method Support"** (kenz-gelsoft, refactored by hecrj) merged 2025-02-03: IME in `TextInput` and `TextEditor` with over-the-spot preedit; widgets opt out via `InputMethod::Open = None`; author: "next step is implementing on-the-spot pre-edits". **First released in 0.14.0 (2025-12-07).**
- Follow-ups in 0.14.0: #2790, #2785, #2792 (macOS candidate window top-left → #2793), #2795, #2798, #2806, #2918 "Cursor size awareness for input methods".
- Widget source (master): `text_input.rs`, `text_editor.rs` call `shell.request_input_method(…)` anchored to field bounds. Master unified editing under `core::text::editor` / `text::Input` (PR #3404, 2026-08-01) — unreleased.
- Remaining: **#3189 "Windows Japanese IME conversion pop-up behaves strangely"** open (2026-01-09); #3258 "IME candidate window misplaced on Linux fcitx5 after #2918" closed 2026-04-19 "COSMIC specific"; #2971 Debian 12 closed 2025-08-19.
- winit 0.30.13: `WindowEvent::Ime` (`Enabled/Preedit/Commit/Disabled`), `set_ime_allowed`; `set_ime_cursor_area`: "X11: area is not supported, only position"; Wayland `zwp_text_input_v3`.
- Halloy: #339 Chinese input (macOS, 2024-04-17) closed 2025-04-07 "fixed: iced#2777"; #240 CJK display closed 2025-03-09. Open: #2211 Mozc (2026-07-26), #1490 Korean rendering (2026-02-16), #2187 macOS emoji picker (2026-07-15).

## 4. Testing story

- `iced_test` 0.14.0: `Simulator` — `click(Selector)`, `tap_key`, `typewrite`, `find`, `snapshot`, `into_messages`. Snapshot renders at 2.0 scale to RGBA; `matches_image` writes `{name}-{renderer}.png`, `matches_hash` `{name}-{renderer}.sha256`; missing references are created and pass; delete to regenerate. Backend via `ICED_TEST_BACKEND`.
- Headless: `iced_wgpu` `Headless::new` (`wgpu::Backends::from_env()` or PRIMARY); `iced_tiny_skia` accepts `"tiny-skia" | "tiny_skia" | "software"`; fallback tries wgpu then tiny-skia. **iced's CI sets `ICED_TEST_BACKEND: tiny-skia` on ubuntu/windows/macOS** — GPU-less CI works.
- PR #2698 "Headless Mode Testing" merged 2024-12-17; snapshot names include OS because "font selection is platform-dependent". PR #3059 "First-class end-to-end testing" merged 2025-09-23: `tester` feature (F12 record/play/export), `Emulator`, `Preset`s, `.ice` files, `iced_test::run()`, `iced_selector`; syntax "very likely to change".
- Adoption: `iced_test` reverse deps 16; GitHub Cargo.toml mentions 178; `pop-os` org and `squidowl/halloy` = 0 hits. No official TDD doc.

## 5. Accessibility

- **#552 "Implement accessibility support"** open since 2020-10-05. PRs: #1849 WIP by wash2 (System76) open since 2023-05-11; #3111 draft AccessKit (2025-11-11, Button+Text only); **#3281 "Accessibility support" by dhedlund (AccessKit 0.24, many widgets) opened and closed 2026-03-14 — hecrj: "Thanks! But I'll work on this myself."**
- Forks: `plushie-iced` 0.8.4 (2026-05-08) full AccessKit tree. libcosmic: `a11y = ["iced/a11y", "iced_accessibility"]` default; `iced_accessibility` in `pop-os/iced` pins `wash2/accesskit` tag `cosmic-0.14`. `pop-os/iced` master is 258 ahead / 336 behind upstream (2026-08-26).

## 6. Widgets in `iced` 0.14.0

Modules: button, canvas, checkbox, combo_box, container, float, grid, image, keyed, markdown, operation, overlay, pane_grid, pick_list, progress_bar, qr_code, radio, row, rule, scrollable, selector, sensor, shader, slider, space, svg, table, text, text_editor, text_input, theme, toggler, tooltip, vertical_slider. Functions: `hover`, `opaque`, `stack`, `pin`, `float`, `sensor`, `grid`, `table`, `keyed_column`, `lazy`, `responsive`, `rich_text`, `span`, `markdown`, `mouse_area`, `themer`, `component`.
- `pane_grid`: dynamic splits, mouse resizing, drag-and-drop of panes, hotkeys, `State` API.
- `markdown` + `rich_text` since 0.13 (#2508); incremental parsing, images, quotes, tasklists in 0.14.
- `lazy`: caches a view until a dependency changes; no virtualization.
- **Not built in**: tree view, context menu, modal widget (example only), generic inter-widget DnD, virtualized list. Maintainer on #3429 (2026-08-17): "Sir, I'm afraid we don't have a `list` widget."

## 7. Ecosystem widgets

| Crate | Version / date | iced compat | Notes |
|---|---|---|---|
| `iced_aw` | 0.14.1, 2026-04-27 | `iced ^0.14`, `iced_widget ^0.14.2` | Badge, Card, ColorPicker, ContextMenu, DatePicker, DropDown, Menu/MenuBar, NumberInput, SelectionList, SideBar, SlideBar, Spinner, TabBar/Tabs, TimePicker, TypedInput, Wrap, LabeledFrame. 678★. No Modal |
| `iced_term` (Harzu) | 0.8.0, 2026-03-27 | `iced ^0.14`, `alacritty_terminal ^0.25.1` | "tested on macOS, Linux, and Windows"; not "full terminal features", API unstable until iced 1.0. 177★ |
| `frozen_term` (Frostbyte) | crates 0.1.0 (2024-11); repo 0.8.1 with iced git + wezterm git | git master | wezterm parser; Windows + Linux; 17★, pushed 2026-04-13 |
| `iced_nodegraph` (tuco86) | 0.4.2, 2026-07-23 (repo pushed 2026-08-24) | iced 0.14, **wgpu only** | nodes on infinite pan/zoom canvas, typed pins, drag-to-connect, box select; **no auto-layout**; 500-node demo; MIT; 29★ |
| `iced-node-editor` (mkmarek) | no crates release | — | 17★, pushed 2026-03-26 |
| `iced-graph-editor` (tarkah) | — | old | pushed 2022-12-08 |
| `iced_drop` | 0.2.42, 2026-08-21 | 0.14 | `droppable` + `find_zones`; **todo-board (Trello-like) example**; 34★ |
| `iced_reorderable`, `iced_draggable_tabs`, `dragking-iced` | — | — | small |
| `iced_layershell` | 0.19.1, 2026-07-12 | — | Wayland layer-shell |
| `iced_video_player` | 0.6.0, 2025-12-14 | — | GStreamer |
| `iced_webview` | 0.0.5, 2024-11-03 (repo pushed 2026-03-20) | stale | "only supports Ultralight/Webkit" |
| `cryoglyph` | git rev | — | glyphon fork; text shaping via `cosmic-text` (0.15 in 0.14.0, 0.19 master) with browser-like fallback lists |

- "@"-mention: no built-in overlay besides `combo_box`. Halloy implements nick/command/channel/emoji completion with its own `anchored_overlay` over `text_editor`.
- Virtualized lists: none. #160 "`InfiniteList` widget" open since 2020; #2603 "Noticeable performance degradation for scrollables in 0.13" open (2024-09-24).
- awesome-iced lists iced_term, frozen_term, iced_video_player, plotters-iced, iced_code_editor, bevy_iced, nih-plug; no node editor or kanban.

## 8. libcosmic / COSMIC

- libcosmic: MPL-2.0; 934★; pushed 2026-08-25; `version = "1.0.0"`, edition 2024, `rust-version = "1.93"`; iced as git submodule of `pop-os/iced` (features `advanced, image-without-codecs, lazy, svg, web-colors, tiny-skia`); default features `winit, tokio, a11y, dbus-config, x11, wayland, multi-window`.
- Widgets: nav_bar, nav_bar_dnd, segmented_button, segmented_control, context_drawer, context_menu, dialog, dropdown, menu, responsive_menu_bar, header_bar, tab_bar, toaster, popover, dnd_source/dnd_destination, reorderable_flex_row, list_column, settings, search_input, inline_input, editable_input, spin_button, calendar, color_picker, table, text_context_menu, warning, wayland.
- Cross-platform: book claims "any Linux distribution (X11 & Wayland), Redox OS, Windows, Mac, and even… Android". Counter: #505 "no crossplatform support" (2024-06-19; Windows trait-bound errors, macOS import errors; closed via PR #507 same day); discussion #489 "windows support" (2024-06-10 → 2024-11-02, users report no success, no maintainer answer).
- COSMIC: Alpha 1 2024-08-08 … Beta 1 2025-09-26; **Epoch 1.0 stable 2025-12-11**; **Epoch 1.6 2026-08-18**. NVIDIA track record is cosmic-comp (compositor), not apps under GNOME Wayland: cosmic-epoch #168 "NVIDIA graphics meta-issue" open since 2024-01-03; 2026 cosmic-comp issues #2323, #2216, #2341, #2227, #2232.

## 9. Notable apps

| App | Facts |
|---|---|
| Halloy (IRC) | 4.4k★, GPL-3.0-or-later, release 2026.8 (2026-07-26); Linux/macOS/Windows; iced git fork. `pane_grid` + `TitleBar` + `pane_grid::Controls`, `sensor`, custom `modal`; composer `text_editor`; own widgets anchored_overlay, context_menu, combo_box, selectable_rich_text, selectable_text, tooltip, modal, decorate, double_pass. Assets: linux tar.gz 21 MB, Windows installer 15 MB, dmg 38 MB |
| Sniffnet | 40,685★; `iced = "0.14.0"`; v1.5.1 2026-07-22; packages ~13–20 MB |
| COSMIC apps | cosmic-files 286★, cosmic-edit 343★, cosmic-settings 254★, cosmic-term 595★ (alacritty_terminal + cosmic-text custom renderer; wgpu with softbuffer/tiny-skia fallback), cosmic-comp 821★; all GPL-3.0 |
| Icebreaker (hecrj) | local AI chat, iced + llama.cpp; 457★, MIT; pushed 2026-02-16 |
| AI/chat | ollama-chat-iced 16★; hermes-chat (0.15.0-dev git); Concerto (iced 0.14, "AI coding harness", 0★) |
| Terminal-bearing | oryxis "SSH Client & Terminal Emulator" 291★ (iced fork); frost (iced 0.14, Linux terminal); Frostbyte |

## 10. Language bindings
No TypeScript/JavaScript or React-style layer found. Python wrappers only: `IcedPyGui` (PyO3, iced 0.13.1), `pyiced` 0.3.0a7.

## 11. AI-friendliness signals
- GitHub repo search: `iced language:rust` 1,728 vs `gpui language:rust` 1,036; `topic:iced` 321, `topic:iced-rs` 180, `topic:gpui` 327. Cargo.toml code search: `"iced ="` 3,889 vs `"gpui ="` 6,060 (gpui inflated by Zed monorepo/forks — inference). crates.io reverse deps: `iced` 367, `gpui` 128, `iced_aw` 37, `iced_test` 16.
- 55 official examples. Elm architecture (README/book FAQ).
- Stale-API evidence: `Sandbox` removed undocumented (#2622). #3429 (2026-08-17) describes a non-existent `iced::widget::list::List` with line numbers — consistent with LLM-fabricated content (inference). No `llms.txt`/AGENTS.md.

## 12. Binary size / memory
- Local (2026-08-26, macOS 26.6.2 arm64, rustc 1.98.0, iced 0.14.0 default features, release + thin LTO): counter app 11,177,440 B unstripped / 8,632,784 B stripped; idle RSS ~93.8 MB; 383 crates in lockfile.
- Discussion #1531 (2022-11-12, Windows 10 counter): release 6 MB/76 MB RAM; `opt-level="z"+lto` 3.1 MB; glow 1.5 MB/27 MB. Maintainer (Dec 2024): "memory usage is just not an issue at all".
- Lukas Kalbertodt (2023-02-03, Ubuntu 20.04, `todos` example): 17 MB binary, ~230 ms startup.
- Halloy 2026.8 Linux tar.gz 21 MB, Windows installer 15 MB; Sniffnet 1.5.1 deb 13.8 MB, AppImage 20 MB.

## Claims not verified
1. discourse.iced.rs unreachable (DNS), so the CJK IME design thread and "thousands of items" thread are unverified. 2. `iced_aw` 0.14.1 date (crates.io 2026-04-27 vs docs.rs "July 10, 2026"). 3. iced-rs/winit fork patches beyond `handleUrl`. 4. iced 0.14.0 Windows/X11 IME behaviour not tested. 5. libcosmic on macOS/Windows in 2026. 6. Icebreaker's iced pin; Termherd. 7. `iced_nodegraph` stress claims. 8. Cross-OS pixel snapshot reproducibility. 9. Search counts fluctuate. 10. NVIDIA + GNOME Wayland behaviour of iced 0.14 on a physical Ubuntu machine.

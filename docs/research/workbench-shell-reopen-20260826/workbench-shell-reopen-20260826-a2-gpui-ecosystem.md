# 附录 A2 · GPUI 控件生态调研原始报告(2026-08-26)

> 主备忘:`README.md`。调研代理原始英文报告,主线复核补充见文末。gpui-component 的源码事实核对于 shallow clone `d5821f27`(2026-08-25,`main`)。

Conventions: **exists** = shipped and verified in source; **partial** = building blocks exist but the surface must be assembled; **missing** = not found.

## 1. longbridge/gpui-component

| Fact | Value | Source |
|---|---|---|
| Stars / forks / contributors | 13,483 / 811 / 129 | https://github.com/longbridge/gpui-component |
| Last push | 2026-08-25; HEAD `d5821f27` | commits/main |
| License | Apache-2.0 (`LICENSE-APACHE`; `license = "Apache-2.0"` in `crates/ui/Cargo.toml`) | |
| Latest GitHub release | v0.5.1, 2026-02-05 (main at 0.5.2 unreleased) | |
| Latest crates.io | `gpui-component` 0.5.1 (2026-02-05) | |
| Crates | `crates/{assets, base, fps, macros, story, story-web, ui, webview}` — `gpui-base` (headless behavior) + `gpui-component` (styled) + `gpui-wry` (webview) | |

How it pins gpui: `main` workspace `Cargo.toml` line 43: `gpui = { version = "0.2.2", git = "https://github.com/zed-industries/zed" }` plus `gpui_platform` (features `font-kit,x11,wayland,runtime_shaders`), `gpui_web`, `gpui_macros` from the same git, no explicit `rev`; `Cargo.lock` resolves to zed commit `8b1497db` (2026-08-17). Published 0.5.1 declares `gpui ^0.2.2` (crates.io, last published 2025-10-22); README recommends the git dependency. Consequence: you inherit Zed's `main` (and vendored `zed-reqwest`, `zed-font-kit`, `zed-sum-tree` forks); expect to pin a `rev` yourself.

Coverage against required surfaces:

| Surface | Status | Component (path in `crates/ui/src/`) |
|---|---|---|
| Sidebar / activity bar | exists | `sidebar/` — `Sidebar`, `SidebarMenu`, `SidebarMenuItem`, `SidebarGroup`, `SidebarHeader/Footer`, `SidebarCollapsible`, `SidebarToggleButton`; example `examples/sidebar` |
| Dockable panels | exists | `dock/` — `DockArea`, `Panel`/`PanelView`, `TabPanel`, `Tiles`, `DragPanelPreview`, `DragMoving/DragResizing`; "Resizable panels, draggable tabs, nested splits, edge docks, and serializable freeform Tiles" |
| Chat composer, multi-line | exists | `input/textarea.rs` → `TextareaState` with `.rows(n)`, `.auto_grow(min,max)`, soft-wrap (`crates/base/src/input/base/state.rs` ~4997–5099) |
| IME / CJK | exists | `impl<M: InputModeKind> EntityInputHandler for InputBaseState<M>` at `crates/base/src/input/base/state.rs:2643`: `marked_text_range` (2668), `unmark_text` (2677), `replace_text_in_range` (2686), `replace_and_mark_text_in_range` (2826); comments reference macOS `insertText:` commit semantics and Esc-abort of IME composition. README claims "CJK Support: Yes". Used in production by Longbridge Pro. |
| "@" mention popup | partial | No "mention" in the repo. LSP-style `CompletionProvider` trait in `crates/base/src/input/editor/lsp/completions.rs:40` with `completions(text: &Rope, offset, trigger, ...)` and `is_completion_trigger(&self, offset, new_text: &str, cx) -> bool` (l.102), rendered by `input/popovers/completion_menu.rs`. Return `true` when `new_text == "@"` and serve your own items — but wired to Editor mode (`InputState::as_editor` / `.lsp(...)`, `crates/base/src/input/editor/mod.rs:176`), not plain Textarea. |
| Kanban / DnD | missing (kanban) / partial (DnD) | No kanban, no sortable list. `list/list_item.rs:282-284` has stub `on_drag`/`on_drop`; real DnD in `dock/tiles.rs`, `dock/tab_panel.rs`, `table/state.rs` (column moving). |
| DAG / node graph | missing | No graph/node/dag/dagre code. `chart/sankey_chart.rs` exists (not editable). |
| Terminal | missing | The word "terminal" appears only in dock/sidebar story labels. |
| Markdown | exists | `text/` — `markdown(source) -> TextView`, `html(source) -> TextView`; `markdown` 1.0 crate + `html5ever` 0.27; Tree-sitter code blocks via `highlighter/`. Limits: "TextView does not support inline custom [plugins]", no inline images in `InteractiveText`, HTML "we not support CSS". |
| Virtual lists / tables | exists | `virtual_list.rs`, `list/`, `table/` (`Table`, `DataTable`, `col_resizable`, `col_movable`, `sortable`, `fixed_left`, `row_selectable`, `stripe`, `loop_selection`); README: "hundreds of thousands of rows". |
| Tabs / tooltip / context menu / modal | exists | `tab/`, `tooltip.rs`, `menu/context_menu.rs`, `menu/popup_menu.rs`, `menu/dropdown_menu.rs`, `menu/app_menu_bar.rs`, `native_menu/`, `dialog/`, `sheet.rs`, `popover.rs`, `command/` (palette), `notification.rs` |
| Keyboard navigation | partial | `crates/base/src/focus_trap.rs`, `tab_index(...)` on Input/Textarea/Editor, `kbd.rs`; dialogs/sheets trap focus; `docs/ACCESSIBILITY-UI-TESTING.md`. No global roving-tabindex layer beyond gpui actions/keybindings. |
| Code editor | exists | `input/editor.rs` → `InputState::as_editor`, `.language(...)`, `.line_number(...)`, `.lsp(...)`, `.diagnostics(...)`; completion/hover/code-action/diagnostic popovers; 50+ Tree-sitter grammars behind `tree-sitter-languages`. README: "200K lines". |
| WebView | exists (macOS/Windows), broken on Linux | `crates/webview` = `gpui-wry` 0.5.0 (Apache-2.0) wrapping `lb-wry` 0.53.3. `examples/webview/src/main.rs:23-39`: native path `cfg(any(windows, macos, ios, android))`; GTK/Linux branch annotated `// doesn't work yet // TODO`. |
| Charts | exists | `chart/` — area, bar, candlestick, line, pie, radar, sankey; `plot/` primitives. |
| Web/WASM | exists (experimental) | `crates/story-web` + `gpui_web`. |

Other components: accordion, alert, avatar, badge, breadcrumb, button, checkbox, clipboard, collapsible, color_picker, combobox, description_list, form, group_box, hover_card, icon, label, link, number_input, otp_input, pagination, progress, radio, rating, scroll(bars), searchable_list, select, separator, setting(s panel), skeleton, slider, spinner, status_bar, stepper, switch, tag, theme, time (calendar/date picker), title_bar, tree, window_border. 60+ stories. Repo ships `skills/` (`gpui-component`, `gpui`) for AI coding agents and `docs/ARCHITECTURE.md`.

## 2. Zed's own reusable crates — licenses (main @ `f42c6e87`, 2026-08-25)

| Crate | license | publish | Reusable from Apache-2.0 project? |
|---|---|---|---|
| `gpui` | Apache-2.0 | true (0.2.2) | Yes |
| `gpui_platform`, `gpui_macos`, `gpui_windows`, `gpui_linux`, `gpui_web` | Apache-2.0 | — | Yes (git) |
| `gpui_macros` | Apache-2.0 | false | Yes via git |
| `gpui_tokio` | Apache-2.0 | — | Yes |
| `sum_tree`, `util` | Apache-2.0 | false | Yes via git (`zed-sum-tree` published) |
| `terminal` | GPL-3.0-or-later | — | No |
| `terminal_view` | GPL-3.0-or-later | — | No; depends on `editor`, `project`, `workspace`, `settings`, `theme`, `ui` (all GPL) |
| `editor` | GPL-3.0-or-later | — | No |
| `markdown` | GPL-3.0-or-later | — | No (deps `pulldown-cmark`, `language`, `theme`, `ui`) |
| `ui` | GPL-3.0-or-later | — | No |
| `theme`, `settings`, `project`, `multi_buffer`, `text`, `rope`, `language_model`, `agent`, `agent_ui`, `acp_thread` | GPL-3.0-or-later | — | No |

Everything named `gpui*` is Apache; everything editor/workspace-level is GPL. GPL crates are not on crates.io and pull the whole Zed workspace graph — only usable by vendoring the monorepo.

Main-line verification (2026-08-26): `terminal_view` `[dependencies]` = anyhow, async-recursion, breadcrumbs, collections, db, dirs, editor, futures, gpui, itertools, language, log, menu, pretty_assertions, project, regex, schemars, task, serde, serde_json, settings, shellexpand, shlex, terminal, theme, theme_settings, ui, util, workspace, zed_actions. `workspace` `[dependencies]` = any_vec, agent_settings, anyhow, async-recursion, client, chrono, clock, collections, component, db, dirs, futures-lite, fs, futures, git, gpui, http_client, itertools, language, log, menu, markdown, node_runtime, parking_lot, postage, project, remote, schemars, serde, serde_json, session, settings, smallvec, sqlez, strum, task, telemetry, theme, theme_settings, ui, ui_input, url, util, uuid, zed_actions. `project` has 73 dependencies. Source sizes: `terminal/src` 269,300 B; `terminal_view/src` 412,425 B (`terminal_element.rs` 113,635; `terminal_panel.rs` 127,044; `terminal_view.rs` 116,637; `terminal_path_like_target.rs` 33,846; `persistence.rs` 18,740; `terminal_scrollbar.rs` 2,523); `agent_ui/src` 2,387,612 B; `markdown/src` 439,717 B; `ui_input/src` 10,798 B.

## 3. Terminal options on gpui

- `alacritty_terminal` — Apache-2.0, crates.io 0.26.0 (2026-04-06). Exposes `Term::grid()`, `grid_mut()`, `renderable_content() -> RenderableContent`. Ships PTY layer: `tty/windows/{conpty.rs, child.rs, blocking.rs, mod.rs}` — ConPTY on Windows, forkpty on Unix.
- Zed: `crates/terminal/src/alacritty.rs` builds `tty::Options` and calls `tty::new(options, window_size, window_id)` with `#[cfg(windows)]` branches; pins fork `zed-industries/alacritty` rev `4c129667…`. No `portable-pty`.
- Standalone gpui terminal widgets: `zortax/gpui-terminal` — `TerminalView`; VTE via `alacritty_terminal`; generic `Read`/`Write` streams (README uses `portable-pty`); MIT OR Apache-2.0; crates.io 0.1.0 (2025-12-24); 46 stars; last push 2026-07-30; README: "Mouse text selection not yet implemented", "No scrollback navigation". `l0ng-ai/tty7` — full terminal workbench on gpui + alacritty_terminal, Apache-2.0, 780 stars, pushed 2026-08-25, macOS/Windows/Linux, daemon-owned persistent sessions; terminal is app-internal (`tty7-core` has `client core daemon host`). Others with in-tree terminals: `duxweb/codux` (GPL-3.0), `penso/arbor` (MIT), `lassejlv/termy`.
- Main-line additions: `gpui_xterm` 0.1.1 (crates.io 2026-03-11, MIT, repo `Modolet/gpui_xterm`, 2 stars, ~97 KB Rust, `alacritty_terminal = "0.25.1"`, `arboard` with `wayland-data-control`); `gpui-ghostty` 0.0.1 (2026-08-04, repo `prabirshrestha/gpui-ghostty` now 404); `HarryJhin/crux` (macOS-only, 0 stars). libghostty-vt: zero-dependency C library (parser, terminal state, scrollback/reflow, key/mouse encoding, `RenderState`), no PTY/rendering/fonts; API unstable; Rust crates `libghostty-vt` 0.2.1 (2026-07-18, MIT/Apache-2.0, 77.6k downloads) + `libghostty-vt-sys` by `Uzaaft/libghostty-rs` (380 stars, pushed 2026-08-21; requires Zig 0.16 on PATH; `!Send/!Sync`). Paneflow (`arthjean/paneflow`, GPL-3.0, GPUI) switched Linux builds to libghostty-vt on 2026-07-18 (static pinned archive `ae52f97d`); macOS and Windows still Alacritty. gpui's `Surface` element is macOS-only (`CVPixelBuffer`) — no cross-platform external-texture path to embed a full renderer.
- Recommendation-grade fact: the Apache-2.0 path is `alacritty_terminal` (VTE + ConPTY/forkpty) + your own gpui element that paints `renderable_content()`; `gpui-terminal`/`gpui_xterm` are small reference implementations.

## 4. Drag-and-drop & graphs on gpui

- Built-in DnD in `crates/gpui/src/elements/div.rs`: `on_drag_move`, `on_drop`, `can_drop`, `drag_over`, `on_drag<T, W>(value, constructor -> Entity<W>)` on `StatefulInteractiveElement`. gpui-component's dock uses exactly this.
- Canvas: `gpui::canvas(prepaint, paint)` (`elements/canvas.rs`). `Path::new(start)`, `move_to`, `line_to`, `curve_to(to, ctrl)` (quadratic Bézier) — no cubic helper; `Window::paint_path`, `paint_quad`, `paint_svg`, `paint_image`.
- Node-graph crates: `pacifio/gpui-flow` — React-Flow-style: Bezier/Straight/SmoothStep edges, pan/zoom, drag, multi-select, handles, minimap, undo/redo, viewport culling, 1000-node example; MIT; 27 stars; created and last pushed 2026-03-23; no crates.io; no auto-layout. `tu6ge/ferrum-flow` — "high-performance, extensible node-based editor framework", Apache-2.0, 76 stars, pushed 2026-06-07. `eliheuer/gpui-node` — minimal demo, Apache-2.0, 0 stars. Also `gpui-whiteboard` (infinite pan/zoom canvas with shapes/arrows, in `packetThrower/zorite`). No gpui-specific dagre.
- Kanban: only `Catvert/aviary` (email/calendar/kanban client, Apache-2.0, 3 stars, pushed 2026-08-19) — app-internal.

## 5. Other gpui apps and Zed's Agent Panel

- Longbridge Pro — commercial closed-source (only Flathub packaging repo exists).
- Loungy — archived (last push 2025-10-05).
- Chat / AI-agent desktop apps on gpui: `egoist/waku` (GPL-3.0, 1,217★), `zeronsh/comet` (MIT, 1,114★), `penso/arbor` (MIT, 806★), `duxweb/codux` (GPL-3.0, 415★), `s0lda/hadron` (Apache-2.0), `cking000bigdemon/GPUI-Pi` (MIT); `nhtera/OxiMux`, `mauscoelho/rabbitty-app`, Ronin (not individually verified).
- `remorses/gpuix` — Node.js & React bindings (napi-rs mutation protocol; `div/text/input/textarea/<markdown>/<code>/<diff>/<virtual-list>`, headless Select/Combobox/Tooltip), Apache-2.0, 1,123★, created 2026-01-29, pushed 2026-08-25; pinned GPUI fork; does not use gpui-component. Main-line verification: contributors remorses 182 / oxura 9 / chrissm79 7; npm `@gpuix/native` 0.4.0 (2026-08-23) with optional deps darwin-arm64/x64, linux-x64-gnu, linux-arm64-gnu, win32-x64-msvc, win32-arm64-msvc; README: "On Windows and Linux, GPUI runs its normal blocking native event loop on one dedicated Rust UI thread… Windows runtime validation is pending."; test renderer "fully rendered by Metal"; roadmap unchecked: Canvas element, Multiple windows, React Refresh under `bun --hot`, hot reload of native addon; limitations: nested scrolling unsupported, no springs/keyframes/exit transitions, `white-space: pre` unsupported.
- Other component libraries: `crabtalk/bezel` (MIT, 74★), `LukeTandjung/base-gpui`, `wess/guise` (104★), `Augani/adabraka-ui`.
- Zed Agent Panel — `crates/agent_ui` (GPL-3.0-or-later). `completion_provider.rs`: `PromptCompletionProvider<T>` (l.478), `enum PromptCompletion { SlashCommand, Mention }` (l.1987), `MentionCompletion::try_parse` scanning for the rightmost `@` with a word boundary before and no whitespace after (l.2075–2091), `SlashCommandCompletion::try_parse` (l.2016), `is_completion_trigger` (l.1896). Composer `message_editor.rs` (210 KB) + `mention_set.rs`, built on GPL `editor`. Supporting crates `agent`, `acp_thread`, `prompt_store` GPL. Not reusable from Apache; good design reference.

awesome-gpui (Zed official, CC0, pushed 2026-08-25): 101 entries; Libraries section 23 (adabraka-ui, Base GPUI, Bezel, declarative-gpui, ferrum-flow, gpui-component, gpui-d3rs, gpui-flow, gpui-form, gpui-hooks, gpui-nav, gpui-pdf, gpui-px, gpui-router 97★, gpui-storybook, gpui-symbols, gpui-tea, gpui-video-player, gpui-whiteboard, guise 104★, plotters-gpui, ratex-gpui, rhythm-gpui) — no terminal or kanban library; Developer Tools section lists Arbor, arc, based, Baudrun, Codux, DBFlux, dbui, Fulgur, GitComet, Hadron, helix-gpui, Herdr GUI, hunk, JayJay, lgtm, Lumi, mobie-studio, onetcli, OpenMango, OxiMux, pgui, postman-gpui, Rabbitty, RED, setu, termy, TokenMonitor, tty7, Waku, zed, zedis, Zeron, zlyph, zqlz.

## 6. Zed-independent Rust toolkits (side comparison)

| Toolkit | License | Desktop platforms | IME status | Sidebar/dock/table/terminal ecosystem | Drive from TS/React? |
|---|---|---|---|---|---|
| Iced 0.14.0 (2025-12-07) | MIT | macOS/Linux/Windows | Input-method support in 0.14 (PR #2777) | `iced_aw` 0.14.1, `iced_term` 0.8.0 (alacritty-based); no official dock | No |
| egui 0.36.1 (2026-08-07) | MIT OR Apache-2.0 | macOS/Linux/Windows/web | Native IME works; 0.35 improved IME visuals; historic CJK font gaps (#3060) | `egui_dock` 0.21.1, `egui_tiles` 0.17.1, `egui_extras`, `egui_term` 0.1.0, `egui_graphs` 0.31.0 | No |
| Slint 1.17.1 (2026-07-07) | GPL-3.0-only OR Royalty-free-2.0 OR Commercial | macOS/Linux/Windows (+embedded/mobile) | Chinese IME #1644 closed 2022; open #10861 | Std widgets; no dock/terminal | Yes: `slint-ui` npm 1.17.1 (API "still early") |
| Makepad (`makepad-widgets` 1.0.0, 2025-05-13) | MIT OR Apache-2.0 | macOS/Linux/Windows/web/mobile | `platform/src/ime.rs` exists | Own dock; no terminal crate | No |
| Dioxus 0.8.0-alpha.1 / Blitz 0.3.0-beta.2 (2026-08-24) | MIT OR Apache-2.0 | native via Blitz; desktop via webview | Blitz IME issues #272/#287/#288 closed Nov 2025 | `dioxus-components`, `dioxus-primitives`; no terminal | RSX (Rust) |
| Xilem 0.4.0 / Masonry 0.4.0 (2025-10-29) | Apache-2.0 | macOS/Linux/Windows/Android | IME re-added PR #762 (2024-11-30) | Minimal widget set | No |

## Claims not verified

1. gpui-component IME/CJK verified from source only; not run on Linux/Wayland or Windows. 2. Whether `CompletionProvider` popups attach to plain Textarea mode. 3. gpui-component WebView Linux support statement. 4. `ferrum-flow` feature set / gpui rev. 5. `gpui-flow` gpui pin (likely stale). 6. Rust dagre-equivalent layout crate. 7. Makepad IME quality. 8. Slint Royalty-free terms for a developer tool. 9. Ronin, OxiMux, Rabbitty, termy not individually fetched. 10. Longbridge Pro closed-source inferred.

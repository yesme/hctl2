# 附录 A5 · Flutter 调研原始报告(2026-08-26)

> 主备忘:`README.md`。调研代理原始英文报告;pub.dev 日期取自 API `published` 字段,GitHub 数字取自 2026-08-26 的 API。**[unverified]** 为未核实。主线复核补充(Canonical 四件 Flutter 应用)见 §7 末尾。

## 1. Release & stewardship

- Latest stable: Flutter 3.47.0, 2026-08-12 — https://flutter.dev/blog/whats-new-in-flutter-3-47 . 3.47.1 hotfix 2026-08-19 with Dart 3.13.1 (local probe confirmed via `flutter --version`). Most docs.flutter.dev pages still banner "reflects Flutter 3.44.7".
- Dart 3.13 (2026-08-12); 3.44 shipped Dart 3.12 (2026-05-20).
- Cadence 2026: 3.41 Feb, 3.44 May, 3.47 Aug, 3.50 Nov (branch cutoff 2026-10-06).
- Canonical stewardship (3.44 blog, 2026-05-20, verbatim): "We are excited to announce an expanded partnership with Canonical, who will now serve as the lead maintainer and Strategic Steward for Flutter Desktop. With their deep technical expertise, Canonical will lead the Flutter Desktop roadmap and oversee the maintenance of our Linux, Windows, and macOS embedders." Framed as "the first step in a broader ecosystem expansion" of governance.
- Concrete Canonical deliverables (robert-ancell / mattkae): tooltip windows on Linux (PR #182348, 2026-04-02; "Due to a limitation of the Wayland xdg_popup implementation they do not reposition after being shown"); Linux popup windows (#185866, 2026-05-05); `fl_view_new_sized_to_content()` (#182924); Wayland subsurface renderer `FlViewRendererSubsurface` (EGL, "use this renderer automatically whenever running on Wayland", #191389 merged 2026-08-24); ATK a11y regression fix (#176991, 2025-10-23); Linux GPU driver info in `flutter doctor` (#163980, 2025-02-27). 3.47 blog credits "Robert Ancell and Matt Kaestner" for experimental windowing APIs. Canonical multi-window write-up (2025-01-22): https://ubuntu.com/blog/multiple-window-flutter-desktop
- Desktop support: no "beta" caveat on the desktop page. Supported-platforms table (2026-08-12): Windows 10/11 (x64, Arm64); macOS 12–26 (Intel deprecation in progress); Debian 10–13; Ubuntu 20.04–24.04 LTS (CI-tested: Ubuntu 22.04, Debian 12, Windows 10, macOS 15).
- Multi-window: experimental, **main channel only** (`features.dart`: `configSetting: 'enable-windowing'`, env `FLUTTER_WINDOWING`, `master: available: true` only). 3.47: "Linux and Windows now support popup windows", `windowHandle` exposes HWND/NSWindow/GtkWindow. Pre-launch checklist #177586 open, 0/20 at creation (2025-10-27). Design doc warns "SHOULD NOT ship applications to production".
- License: BSD-3-Clause.

## 2. Linux embedder & rendering

- Embedder = **GTK 3** (`libgtk-3-dev` build, `libgtk-3-0` runtime). GTK4 request #94804 open since 2021-12-07, P3, unassigned. No GTK4/Wayland-native plan found.
- Wayland via GDK backend since PR #66519 (#57932 closed). As of 2026-08-24 Wayland presents through an EGL child subsurface (#191389). No supported way to reach `xdg_toplevel` (#187837 open since 2026-06-11).
- Renderer in 3.47: **Impeller by default on Linux, OpenGL (ES/EGL) backend.** "Turn linux impeller on by default" #187573 merged 2026-06-24; docs: "available and enabled by default as of Flutter 3.47. In a future release, the ability to opt out of using Impeller will be removed" (opt-out `fl_dart_project_set_enable_impeller(project, FALSE)`). Open Vulkan PR #189584 (2026-07-16): "OpenGL remains the default and the fallback, and the existing software renderer stays opt-in through the `FLUTTER_LINUX_RENDERER` environment variable"; "[Linux] Add support for Vulkan" #181711 open since 2026-01-30; design doc #183495. Desktop text uses SDF rendering (3.47).
- Software fallback: `FLUTTER_LINUX_RENDERER=software` — "Not all features are supported. This is not recommended." (`fl_engine.cc`). No automatic fallback: #177548 "Flutter doesn't fall back to OpenGL ES when Vulkan software rendering is available" (open, 2025-10-25, P2); older #76578 "Unable to create a GL context".
- Wayland + NVIDIA issue list:
  - **#188966 (OPEN, 2026-07-04, P2, "workaround available", e: opengl)** "GTK embedder: Failed to create OpenGL context on Wayland + NVIDIA Optimus, results in black screen" (NVIDIA 610.43.02 + Mesa 26.1.3, KDE Wayland). `GDK_BACKEND=x11` and `LIBGL_ALWAYS_SOFTWARE=1` did not help; working: `__NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia`; related X11 case #184259.
  - #152099 "Black screen on NVIDIA drivers when using glBlitFramebuffer" (2024-07-22, CLOSED fixed).
  - #151098 black screen/flicker in debug on 3.22+ (2024-07-01, CLOSED fixed).
  - #166607 "[Wayland] The window does not resize" (2025-04-04, CLOSED "waiting for response").
  - #190059 mismatched framebuffer after "Timed out waiting for OpenGL frame" → SIGSEGV (2026-07-27, CLOSED).
  - #61125 Wayland+GNOME "Bad Native Window" (2020, CLOSED).
  - Third-party: Flutter AppImage broken on Wayland by bundled libGL/epoxy, "fixed" by forcing `GDK_BACKEND=x11` (industrialflutter.com).
  - Explicit-sync flicker is driver/compositor level (NVIDIA 555.58+, GNOME 46.1+); no Flutter-specific explicit-sync issue.
- Fractional scaling on Wayland: GTK3 does not implement `wp_fractional_scale_v1`; Flutter #127768 "fonts scaled while everything else — not" at 1.5× (2023-05-28, OPEN, P2); #65517 Xorg `Xft.dpi`.
- Performance / frame pacing (2026): #191245 "frame pacing and display synchronization gaps" (2026-08-18, OPEN): "the Linux embedder still does not provide a platform `vsync_callback`, so the engine falls back to the generic fixed-60 Hz `VsyncWaiterFallback`"; #190620 fallback vsync timer quantizes to ms → skipped frames (2026-08-05, OPEN, 3.44); #191425 FPS falls with window size (720p ≈60, 1080p ≈30, 3072×1637 ≈10 FPS, 2026-08-20, OPEN). Idle CPU: #79267 (2021), #125388 (2023). Open Impeller+Linux: #191185 CustomPainter wiggles (2026-08-16), #187537 pixelated lines, #183267, #181562. Open `platform-linux` total: 279.

## 3. IME / CJK

- Linux: `gtk_im_multicontext_new()`, `gtk_im_context_set_cursor_location()`, `preedit-start`/`preedit-changed`/`commit`/`delete-surrounding` (`fl_text_input_handler.cc`). Under GNOME Wayland via the GTK IM module (ibus/fcitx5).
- **#190046 confirmed:** "[Linux] App aborts with `wstring_convert: to_bytes error` when typing with an IME — TextInputModel::DeleteSurrounding splits UTF-16 surrogate pairs" — OPEN, 2026-07-26, P2, `c: fatal crash`, Chinese Wubi on ibus (fcitx also); fix PR #190514 open.
- Other Linux: #97174 CJK on single-line fields (2022, P1, closed); #154072 emoji insertion crash on all three desktops (2024-08-25, OPEN, P2, fatal crash); #155741 Wayland clipboard; #159977/#161123 OSK regressions. 27 open `a: text input` + `platform-linux`. No open issue specifically about fcitx5 candidate-window placement.
- macOS: Japanese IME duplication (#160935) fixed 2025-06-21 (#166291). Open: #190704 SIGABRT in `UpdateComposingText` with third-party Chinese IME during input-source switch (2026-08-07); #190525 Enter that commits IME conversion also fires `Shortcuts` (2026-08-04); #153065; #142493. 23 open macOS IME/CJK text-input issues. #182443 `Utf16ToUtf8` crash (closed dup).
- Windows: Korean caret (#140739) fixed by #186353 in 3.47. Open: #191196 IME composition leaks into next TextField (2026-08-17); #189491 Hangul syllable loss (2026-07-15); #173526 Chinese candidate box position (2025-08-11); #171319; #182876 notes Flutter "still relies on the deprecated IMM32" (2026-02-25, P2). 26 open Windows IME-related.

## 4. Accessibility

- Docs claim Orca (Linux), VoiceOver (macOS), NVDA/JAWS (Windows).
- Linux (ATK → AT-SPI): engine support 2020 (engine PR #19634); screen reader silently broke in 3.35.0–3.37.x (#176360, fixed by #176991 2025-10-23); #101513 (2022, fixed). 13 open a11y+linux incl. #188059 selected state (2026-06-16), #188022 non-focusable content unreadable (2026-06-15), #184568 headers, #183660 toggle state not announced on all three desktops (2026-03-13), #178024 page titles not read (Windows+Linux), #159460 proposal "Replace ATK with AT-SPI directly", #133614 TextField navigation.
- Windows (MSAA/IAccessible via Chromium AX; UIA partial): opt-in `IAccessibleEx` restored (#175406, 2026-04-01); missing `IValueProvider`/`ITextEditProvider` (#182876). 35 open a11y+windows.
- macOS VoiceOver: #167318 live regions (2025-04-16, open); #128915; 9 open.

## 5. Testing & TDD

- `WidgetTester`/`testWidgets`/`pumpWidget`/`find.*` (docs 2026-05-05). Widget tests run on the Dart VM without a display/GPU (local probe: 1 test 6.1 s cold / 1.2 s warm).
- Goldens: `matchesGoldenFile` warns "a golden file generated on Windows with fonts will likely differ from the one produced by another operating system"; default font `Ahem`. Community: Alchemist, bundled fonts.
- Desktop integration tests: `flutter test integration_test/app_test.dart`; on Linux CI "you must invoke an X server first" via `xvfb-run`. No GPU-less headless Linux runner documented.
- Hot reload/restart: debug-mode only.
- Analyzer/LSP with strict modes; **Dart MCP server** (`dart mcp-server`, Dart 3.12; experimental; hot reload/tests/analysis tools) — https://docs.flutter.dev/ai/mcp-server

## 6. Widget wheels per surface

### 6a. Sidebar / dock / split panes — exists
| Package | Version (date) | License | Activity |
|---|---|---|---|
| `multi_split_view` | 3.6.2 (2026-05-24) | MIT | 355 likes |
| `docking` | 1.16.2 (2026-03-18) | MIT | pushed 2026-03-18, 9 open issues |
| `flutter_resizable_container` | 4.2.0 (2025-05-10) | MIT | |
| `NavigationRail` | SDK | BSD-3 | |

### 6b. Rich composer with @-mentions — partial
| Package | Version (date) | License | Notes |
|---|---|---|---|
| `super_editor` | 0.2.7 (2024-06-11); 0.3.0-dev.52 | MIT | 1,929★, 320 open issues, pushed 2026-07-01; pub page: "Unverified: Windows, Linux", "does not include any popovers on desktop" |
| `flutter_quill` | 11.5.1 (2026-05-20) | MIT | 2.1k likes; desktop listed |
| `fleather` | 1.28.0 (2026-08-24) | MIT/BSD-3 | Delta/OT |
| `flutter_typeahead` | 6.0.0 (2026-04-04) | BSD-2 | 2.13k likes |
| `flutter_mentions` | 2.0.1 (2021-05-24) | MIT | stale |
| `flutter_portal` | 1.1.4 (2023-05-24) | MIT | |
| Built-in `Autocomplete`/`RawAutocomplete` | SDK | BSD-3 | FluffyChat's composer uses it with regex triggers `@`, `#`, `:`, `/` (`lib/pages/chat/input_bar.dart`) |

No package-level desktop-IME statements; platform issues in §3 apply to every `TextField`-derived composer.

### 6c. Kanban — partial (all stale ≥13 months)
| Package | Version (date) | License | Activity |
|---|---|---|---|
| `appflowy_board` | 0.1.2 (2024-04-24) | dual AGPL-3.0 / MPL-2.0 | repo pushed 2026-01-04, 110★, 17 open issues |
| `boardview` | 1.0.0 (2025-07-14) | BSD-2 | |
| `kanban_board` | 1.0.0+2 (2025-05-21) | MIT | riverpod |
| `drag_and_drop_lists` | 0.4.2 (2024-11-26) | BSD-3 | 457 likes |

### 6d. DAG / node graph with auto-layout — partial
| Package | Version (date) | License | Layout / interaction |
|---|---|---|---|
| `graphview` | 1.5.1 (2025-10-17) | MIT | **Sugiyama (layered)**, Buchheim-Walker, Fruchterman-Reingold, Balloon/Circle/Radial/Mindmap; pan/zoom via `InteractiveViewer`; repo pushed 2026-04-24, 469★, 25 open |
| `flutter_flow_chart` | 4.1.1 (2025-12-17) | MIT | manual placement, no auto-layout |
| `vs_node_view` | 2.1.1 (2024-03-10) | BSD-3 | node editor, no auto-layout |
| `flow_graph` | 0.0.9 (2022-03-09) | BSD-3 | "DAG graph", draggable nodes |

Layered auto-layout exists (`graphview`); a package combining it with interactive node dragging/re-layout not evidenced [unverified].

### 6e. Terminal — partial
| Package | Version (date) | License | Activity / features |
|---|---|---|---|
| `xterm` (TerminalStudio/xterm.dart) | 4.0.0 (2024-02-27) | MIT | 654★, 107 open issues, last commit 2025-06-19; README claims CJK/emoji wide chars, 60 fps, IME compatibility (3.0.0+); open #207 "Major issues with the latest flutter" (2025-06-06: "keyboard can't type anything", "view ID is null"), no maintainer reply. OSC 8 and selection unconfirmed [unverified] |
| `flutter_pty` | 0.4.2 (2025-01-06) | MIT | 18 open issues |
| `libghostty` (Dart bindings to libghostty-vt) | 0.0.12 (2026-07-28) | MIT | unverified uploader |
| `ghostty_vte_flutter` (widgets on Ghostty VT + `portable_pty`) | 0.1.3 (2026-05-02) | MIT | |
| `flutter_ghostty` (jiahaog; "all code is written by AI", macOS-first) | repo | — | |

No `alacritty_terminal`-via-FRB package found. Shipping 2026 Flutter terminal-centric app: Alera (§7).

### 6f. Markdown + highlighting — exists
- `flutter_markdown` 0.7.7+1 (2025-05-06), BSD-3, **discontinued** ("This project has been discontinued, and will not receive further updates", recommends `flutter_markdown_plus`). Umbrella "Packages planned to be discontinued" 2025-04-30 (#162960, #162966): `ios_platform_images`, `css_colors`, `palette_generator`, `flutter_image`, `flutter_adaptive_scaffold`, `flutter_markdown`.
- `flutter_markdown_plus` 1.0.12 (2026-07-10), BSD-3, Foresight Mobile. `markdown_widget` 2.3.2+8 (2025-04-26), MIT. `gpt_markdown` 1.2.1 (2026-08-23), BSD-3, streaming, LaTeX. `flutter_highlight` 0.7.0 (2021-03-07); `re_highlight` 0.0.3 (2024-02-05).

### 6g. Virtualized lists — exists
`ListView.builder`/`SliverList` (SDK); `scrollable_positioned_list` 0.3.8 (2023-05-08, google.dev); `super_sliver_list` 0.4.1 (2024-03-26); `flutter_chat_ui` 2.11.1 (2025-12-11), Apache-2.0.

### 6h. Menus / shortcuts / tabs / tooltips / modals / kits — exists
`Shortcuts`/`Actions`/`Intent`, `CallbackShortcuts`; `MenuAnchor`/`MenuBar` (SDK); `super_context_menu` 0.9.1 (2025-06-11), MIT, native menus on macOS/Linux/iOS; `contextmenu` 3.0.0 (2022). Kits: `fluent_ui` 4.16.1 (2026-08-03); `macos_ui` 2.2.2 (2025-10-19); `yaru` 10.2.0 (2026-06-08) MPL-2.0, publisher ubuntu.com; `shadcn_ui` 0.56.1 (2026-08-04); `forui` 0.26.0 (2026-08-24). Note: 3.47 splits Material/Cupertino into `material_ui`/`cupertino_ui` 1.0; in-SDK libraries "scheduled for formal deprecation in the upcoming Fall stable release in November".

## 7. Comparable shipping desktop apps

- **AppFlowy** — AGPL-3.0, 75,941★, pushed 2026-08-11. 0.13.2 (2026-08-11): Linux `.deb` 73.1 MB, AppImage 182.1 MB, tar.gz 174.8 MB; macOS arm64 dmg 98.6 MB, universal 182.5 MB; Windows exe 78.8 MB. Rust interop: custom `dart-ffi` crate (`staticlib`) with `allo-isolate` + protobuf, not flutter_rust_bridge. RSS not published.
- **FluffyChat** (Matrix) — AGPL-3.0, 3,075★, pushed 2026-08-25. v2.9.1 (2026-08-17): linux-x64 tar.gz 30.23 MB, arm64 28.51 MB, web 19.9 MB, **no Windows/macOS assets**; website: Linux (Snap, Flathub), web, iOS, Android; Flathub 33 MiB; "Windows support" issue #162 open since 2023-07-15. Composer: built-in `Autocomplete`. #336 "Chinese Characters are displayed as square blocks on Linux" (closed).
- **Zulip Flutter** — Apache-2.0; "intended for use on mobile platforms"; desktop "for development but not for general use".
- **Ubuntu apps (main-line verification 2026-08-26):** App Center `ubuntu/app-center` (918★, GPL-3.0, pushed 2026-08-24, "App Store for Ubuntu made with Flutter"; Dart 650,789 B; `pubspec` requires `flutter: ">=3.44.2"` and still depends on discontinued `flutter_markdown: ^0.7.3+1`, plus `snapd ^0.7.4`); Security Center `canonical/desktop-security-center` ("Flutter-based security center for Ubuntu Desktop", GPL-3.0, created 2023-12-04, pushed 2026-08-25, Dart 2,217,529 B); Firmware Updater `canonical/firmware-updater` (137★, GPL-3.0, pushed 2026-08-25, Dart 1,229,704 B); installer `canonical/ubuntu-desktop-provision` (149★, GPL-3.0, pushed 2026-08-21, Dart 8,185,651 B); `ubuntu/yaru.dart` (390★, MPL-2.0, pushed 2026-07-22). None of the three app READMEs mentions NVIDIA/Wayland/`GDK_BACKEND`.
- **Rive** editor (Flutter; web + macOS/Windows; proprietary). **Superlist** (Flutter; macOS, web, mobile; proprietary).
- **Coding-agent desktop tools in Flutter:** **Alera** — "native, performance-first ADE", Flutter + Rust (`portable_pty`) + Ghostty VTE (`ghostty_vte_flutter`) + `flutter_rust_bridge`, MIT, 17★, updated 2026-08-25, macOS 14+/Windows/Linux (Ubuntu 24.04+, Debian 13+, Fedora) — https://github.com/leynier/alera . FlutterFlow Desktop embeds Claude Code/Codex (closed source).

## 8. Rust interop
`flutter_rust_bridge` 2.13.0 (2026-08-23), MIT, 5,386★, 59 open issues, Flutter Favorite; v2.0.0 2024-06-21. Alternatives: `dart:ffi` + `ffigen`; build hooks stable since Dart 3.10, link hooks in 3.13; `allo-isolate` (AppFlowy). gRPC/WebSocket needs no FFI.

## 9. Footprint (published)
Linux default app ≈18 MB bundle, RSS ≈96–99 MB idle (Flutter 2.6 dev, 2021, #92318 open P3). macOS hello-world ≈30 MB, ≈170 MB RAM vs Electron (2021 benchmark, getstream.io). AppFlowy/FluffyChat asset sizes in §7. Local 2026 measurement: appendix A6.

## 10. AI-friendliness
- GitHub topics (2026-08-26): `flutter` 80,582 repos; `tauri` 11,693; `egui` 1,095; `iced` 329; `slint` 259.
- Official LLM aids: `llms.txt` (references 3.44/Dart 3.12) — https://docs.flutter.dev/llms.txt ; AI rules `rules.md`/`rules_10k.md`/`rules_4k.md`/`rules_1k.md` with `CLAUDE.md` mapping (updated 2026-08-19) — https://docs.flutter.dev/ai/ai-rules ; Dart/Flutter MCP server.
- Stale-API hazard: `withOpacity` deprecated in 3.27 → `withValues`; `MaterialStateProperty` → `WidgetStateProperty`; GitHub Community thread reports Copilot denying `WidgetStateProperty` exists. Nov-2026 Material/Cupertino split adds churn.

## 11. Non-obvious risks
- Google investment: ~200 layoffs across Flutter/Dart/Python, April 2024. Flock fork (Oct 2024) cites "desktop platforms remaining mostly stagnant"; repo 382★, last push 2025-12-15. Canonical stewardship (2026-05) shifts desktop ownership to a partner.
- Impeller migration churn: default on all three desktops in 3.47, opt-out slated for removal; Linux on OpenGL with Vulkan open; Windows Impeller crashes/slowdowns (#191468, #191437, #191353, Aug 2026); macOS Intel flicker (#191538).
- Package discontinuations (six, 2025-04-30); `scrollable_positioned_list` last 2023-05-08.
- GTK3 trajectory: GTK 3.24.52 announcement: "decrease the frequency of GTK3 releases and limit changes to important bug and crash fixes… next GTK3 release is expected in March 2027" (2026-03-23); no fractional-scale protocol; GTK4 migration P3 unassigned.
- Linux gaps (2026): no vsync callback (#191245), Wayland+NVIDIA Optimus black screen with no automatic fallback (#188966, #177548), open IME crash (#190046).
- macOS: App Sandbox on by default; `com.apple.security.network.client` needed; Hardened Runtime + notarization; 3.47 raises min macOS to 12, winding down Intel.
- Supported-distro window: newest listed Ubuntu is 24.04 LTS.

## Claims not verified
1. 3.47.1 hotfix date from third-party (later confirmed locally). 2. Impeller Linux/Windows API per official sheet (Google Sheet unfetchable). 3. Official GTK4/Wayland-native plan — none found. 4. xterm.dart OSC 8/selection/desktop IME. 5. `graphview` node dragging with re-layout. 6. Desktop IME reliability of `super_editor`/`flutter_quill`/`fleather`. 7. AppFlowy idle RSS. 8. 2025–2026 measured minimal-app footprint (now in A6). 9. FluffyChat non-CI Windows/macOS builds. 10. OMG! Ubuntu article body (403). 11. Desktop hot-reload sentence reproduced from search excerpt.

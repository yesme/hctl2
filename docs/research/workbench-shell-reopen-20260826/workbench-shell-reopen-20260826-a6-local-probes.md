# 附录 A6 · 本机同口径探针原始记录(2026-08-26)

> 主备忘:`README.md`。两位探针代理的原始报告合并;GPUI/Iced 探针在无 Xcode(仅 Command Line Tools)时完成,Flutter 探针在所有者装好 Xcode 26.6 后完成。Electron 基线取自 `implementation-evidence.md#e-workbench-shell` 的既有探针(2026-08-23)。所有源码与日志留在会话 scratchpad `gpui-probe/`、`flutter-probe/`,不入库。

## 环境

| Item | Value |
|---|---|
| Machine | Apple M4, 10 cores, 16 GiB RAM, arm64 |
| OS | macOS 26.6.2 (25G83) |
| Rust | rustc 1.98.0 (2026-08-18), cargo 1.98.0, rustup 1.29.0 |
| Xcode | GPUI/Iced run: not installed (CLT only, no `xcrun metal`). Flutter run: Xcode 26.6 (17F113), license accepted |
| `footprint` | `/usr/bin/footprint` works without sudo on own processes |
| Cargo caches | no `~/.cargo/config.toml`, no sccache, no shared target dir |
| Homebrew | 6.0.19 |

Methodology: minimal app = one ~800×600 window, a text label, a counter button; clean release build timed from empty target dir; run binary, wait 6 s, capture `ps -o rss` and `footprint -f bytes`; kill; repeat.

## GPUI 0.2.2 vs Iced 0.14.0

| Metric | GPUI | Iced |
|---|---|---|
| Crate / version | `gpui` 0.2.2 (crates.io 2025-10-22; 10 months stale) | `iced` 0.14.0 (2025-12-07, MSRV 1.88) |
| Features | `runtime_shaders` (default features fail without Xcode — below) | default (wgpu + tiny-skia + …) |
| Clean `cargo build --release` wall | 76.9 s (run 1) / 81.6 s (run 2, quiet machine) | 35.4 s (quiet; a run contaminated by a concurrent sibling build took 69.7 s) |
| "Compiling" lines | 451 | 154 |
| `cargo tree --edges normal \| wc -l` | 1099 lines (540 unique packages) | 367 lines (193 unique packages) |
| `target/release` size | 1.2 GB (1,288,764 KiB) | 494 MB (506,116 KiB) |
| Release binary raw | 5,489,424 B (5.2 MiB) | 10,947,696 B (10.4 MiB) |
| After `strip` | 4,221,744 B (4.0 MiB) | 8,286,528 B (7.9 MiB) |
| Clean debug build | 56.3 s, 451 crates, binary 30,259,048 B | not requested |
| Linked libraries | 18 system frameworks/dylibs, none bundled | 14 system frameworks/dylibs |
| Idle RSS @6 s (KiB) | 72,320–74,512 across 6 runs (~74 MB) | 93,648–94,448 across 3 runs (~96 MB) |
| Idle physical footprint @6 s | 42,484,552 B (42.48 / 42.70 / 42.29 MB; one outlier 50.71 MB) | 42,124,128 B (42.12 / 42.66 / 42.65 MB) |
| `phys_footprint_peak` during startup | 95.3–103.4 MB | 137.9 MB |
| Window appeared | yes (process alive, LaunchServices entry, 15 MB IOSurface + IOAccelerator regions, no panic) | yes |
| stderr | empty in 4/6 runs; 2/6 one benign macOS IMK line | empty |
| Build warnings | future-incompat: `block v0.1.6`, `proc-macro-error2 v2.0.1` | future-incompat: `block v0.1.6` |

Findings:
1. GPUI 0.2.2 default features do not build on a Mac without Xcode: `build.rs` runs `xcrun -sdk macosx metal` to precompile `shaders.metal`; fails after 88.7 s (443 crates) with `error: gpui@0.2.2: metal shader compilation failed` / `xcrun: error: unable to find utility "metal"`. Fix: the crate's `runtime_shaders` feature (Cargo flag).
2. No API mismatch for gpui — app written from the crate's bundled `examples/hello_world.rs` and `opacity.rs`, compiled first try. `.id()` is mandatory before `.on_click()`.
3. Iced API churn: 0.14 changed `iced::application(title, update, view)` (0.13) to `iced::application(boot, update, view)`; crates.io package ships no example sources, signature taken from `src/application.rs` doc comments. Compiled first try.
4. Memory: by `footprint` both idle at ~42 MB; by RSS gpui ~20 MB lighter. Iced's startup peak (138 MB) ~35 MB higher (wgpu init).
5. Build cost: gpui pulls ~3× the dependency graph, ~2.3× the compile time, 2.4× the target dir; release binary half the size of iced's.
6. gpui's occasional stderr line — `error messaging the mach port for IMKCFRunLoopWakeUpReliable` — is a macOS Input Method Kit log when the app takes key focus via `cx.activate(true)`, not a gpui panic.
7. Caveat: the scratchpad was wiped once mid-session by a sibling agent; gpui numbers were captured before the wipe and reproduced byte-identically after rebuild. The Iced 35.4 s figure was re-measured with zero foreign cargo/rustc processes.

`gpui-hello/Cargo.toml`
```toml
[package]
name = "gpui-hello"
version = "0.1.0"
edition = "2024"

[dependencies]
gpui = { version = "0.2.2", features = ["runtime_shaders"] }
```

`gpui-hello/src/main.rs`
```rust
use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};

struct Hello {
    count: u32,
}

impl Render for Hello {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("root")
            .flex()
            .flex_col()
            .gap_4()
            .size_full()
            .justify_center()
            .items_center()
            .bg(rgb(0x202020))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child("hello")
            .child(format!("count: {}", self.count))
            .on_click(cx.listener(|this, _event, _window, cx| {
                this.count += 1;
                cx.notify();
            }))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(|_cx| Hello { count: 0 }),
        )
        .unwrap();
        cx.activate(true);
    });
}
```

`iced-hello/Cargo.toml`
```toml
[package]
name = "iced-hello"
version = "0.1.0"
edition = "2024"

[dependencies]
iced = "0.14.0"
```

`iced-hello/src/main.rs`
```rust
use iced::widget::{button, column, text, Column};

#[derive(Default)]
struct Counter {
    value: i32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
}

fn update(counter: &mut Counter, message: Message) {
    match message {
        Message::Increment => counter.value += 1,
    }
}

fn view(counter: &Counter) -> Column<'_, Message> {
    column![
        text("hello").size(24),
        text(format!("count: {}", counter.value)),
        button("+").on_press(Message::Increment),
    ]
    .spacing(12)
    .padding(24)
}

fn main() -> iced::Result {
    iced::application(Counter::default, update, view)
        .window_size((800.0, 600.0))
        .run()
}
```

Commands (essential): `cargo new …`; `rm -rf target && cargo build --release` (timed with `date +%s.%N`, "Compiling" lines via `grep -c`); `stat -f %z`; `cp … && strip … && stat -f %z`; `cargo build` (debug); `./bin & sleep 6; ps -o pid,rss,vsz,comm -p $PID; /usr/bin/footprint $PID; /usr/bin/footprint -f bytes --noCategories $PID; kill $PID`; `du -sh target/release`; `cargo tree --edges normal | wc -l`; `cargo tree --edges normal --prefix none | sort -u | wc -l`; `xcrun --find metal`; `xcode-select -p`.

Independent cross-check by the Iced research agent (same day, same machine, release + thin LTO): counter app 11,177,440 B unstripped / 8,632,784 B stripped; idle RSS ~93.8 MB; 383 crates in lockfile — consistent.

## Flutter 3.47.1 (macOS, after Xcode 26.6)

| Item | Value |
|---|---|
| Pre-state | no `flutter`, no `pod`, no `~/.pub-cache` |
| `brew install --cask flutter` | 730.07 s wall, rc=0 (2,259,049,326 B zip at ~3 MB/s — network-bound; unpack/link seconds). Installs to `/opt/homebrew/share/flutter`, symlinks `/opt/homebrew/bin/{flutter,dart}` |
| Flutter version | 3.47.1 stable, framework 6655482ec0 (2026-08-19), engine 5d53178869, Dart 3.13.1, DevTools 2.60.0 |
| First `flutter --version` | 9.09 s (first-run bootstrap; populates `~/.pub-cache`) |
| `flutter precache --macos` | 10.98 s (macOS engine already in cask zip) |
| SDK size | 4,098,376 KiB = 3.91 GiB (`bin/cache` 3.4 GiB: engine artifacts 2.63 GiB incl. iOS/Android/darwin-x64, dart-sdk 624 MiB, web sdk 117 MiB) |
| `flutter doctor -v` | 11.63 s first run; flagged CocoaPods missing |
| `brew install cocoapods` | 16.66 s, CocoaPods 1.17.0 (+ ruby 4.0.6_1 118 MiB, libyaml) → Xcode section green |
| `flutter create --platforms=macos` | 1.62 s (39 files; no Podfile — SwiftPM on by default) |
| `flutter test` run1 / run2 | 6.14 s / 1.23 s (1 test, passed) |
| `flutter analyze` | 4.22 s ("No issues found! ran in 3.3s") |
| `dart format --output=none --set-exit-if-changed .` | 0.33 s |
| `flutter clean && flutter build macos --release` run1 / run2 | 22.21 s / 18.50 s (rc=0; run1 includes first SwiftPM/xcodebuild setup) |
| `.app` size (`du -sk`) | 37,368 KiB (36.5 MiB); flutter reports "38.2MB" decimal |
| `Contents/MacOS/flutter_hello` | 193,040 B — universal x86_64+arm64 (arm64 slice 94,736 B) |
| `FlutterMacOS.framework` | 30,192 KiB; binary 30,035,744 B universal (arm64 14,618,400 B, x86_64 15,399,840 B) |
| `App.framework` | 6,780 KiB; AOT binary 6,529,040 B universal (arm64 3,203,088 B) + ~380 KB flutter_assets |
| Idle memory run1 (6 s) | RSS 112,048 KiB (109.4 MiB); footprint 57,312,096 B (54.7 MiB), peak 184,009,568 B |
| Idle memory run2 | RSS 108,864 KiB (106.3 MiB); footprint 58,311,544 B (55.6 MiB), peak 175,014,776 B |
| Idle memory run3 | RSS 109,248 KiB (106.7 MiB); footprint 58,639,200 B (55.9 MiB), peak 175,194,976 B |
| Process alive after 6 s | yes, all 3 runs; stderr: `[IMPORTANT:flutter/shell/platform/embedder/embedder_surface_metal_impeller.mm(53)] Using the Impeller rendering backend (MetalSDF).` |
| Cold start (`flutter run -d macos --profile --trace-startup`) | Time to first frame 96 ms (`timeToFirstFrameMicros` 96,804; framework init 62,982; first frame rasterized 171,558 µs); build+run+trace wall 18.26 s; Profile .app 60.5 MB |
| Disk footprint added | SDK 3.91 GiB + `~/.pub-cache` 522 MiB (174 packages, from SDK workspace pub get) + cocoapods/ruby/libyaml 183 MiB + Homebrew zip cache 2.10 GiB = 6.71 GiB; project `build/` 764 MiB, `.dart_tool` 54 MiB |

Footprint breakdown (run1): MALLOC_SMALL 15.4 MB, IOSurface 7.8 MB, graphics owned-unmapped 7.2 MB, VM_ALLOCATE 4.6 MB, IOAccelerator 4.3 MB, MALLOC_LARGE 4.2 MB; clean __TEXT 13.5 MB; 47 MB "reclaimable" graphics.

Notes: build is universal by default (arm64-only would roughly halve framework/App sizes). Window presence via `osascript` was refused (no Accessibility permission); inferred from process alive with Metal/Impeller surface and IOSurface/CoreAnimation allocations. `flutter run --trace-startup` printed `Failed to foreground app; open returned 1` (cosmetic). Xcode warning each build: `Run script build phase 'Run Script' will be run during every build because it does not specify any outputs` (target 'Flutter Assemble'). CocoaPods not actually needed for a plugin-free SwiftPM project; installed only to satisfy doctor.

`lib/main.dart`
```dart
import 'package:flutter/material.dart';

void main() => runApp(const HelloApp());

class HelloApp extends StatelessWidget {
  const HelloApp({super.key});

  @override
  Widget build(BuildContext context) {
    return const MaterialApp(
      debugShowCheckedModeBanner: false,
      home: Scaffold(body: Center(child: Counter())),
    );
  }
}

class Counter extends StatefulWidget {
  const Counter({super.key});

  @override
  State<Counter> createState() => _CounterState();
}

class _CounterState extends State<Counter> {
  int _count = 0;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Text('hello', style: TextStyle(fontSize: 32)),
        Text('count: $_count', key: const Key('count')),
        ElevatedButton(
          key: const Key('increment'),
          onPressed: () => setState(() => _count++),
          child: const Text('Increment'),
        ),
      ],
    );
  }
}
```

`test/widget_test.dart`
```dart
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:flutter_hello/main.dart';

void main() {
  testWidgets('counter increments on tap', (tester) async {
    await tester.pumpWidget(const HelloApp());

    expect(find.text('hello'), findsOneWidget);
    expect(find.text('count: 0'), findsOneWidget);

    await tester.tap(find.byKey(const Key('increment')));
    await tester.pump();

    expect(find.text('count: 1'), findsOneWidget);
    expect(find.text('count: 0'), findsNothing);
  });
}
```

`flutter doctor -v` (after CocoaPods):
```
[✓] Flutter (Channel stable, 3.47.1, on macOS 26.6.2 25G83 darwin-arm64, locale zh-Hans-SG) [115ms]
    • Flutter version 3.47.1 on channel stable at /opt/homebrew/share/flutter
    • Framework revision 6655482ec0 (6 days ago), 2026-08-19 10:07:23 -0700
    • Engine revision 5d53178869
    • Dart version 3.13.1
    • DevTools version 2.60.0
    • Feature flags: enable-web, enable-linux-desktop, enable-macos-desktop, enable-windows-desktop, enable-android, enable-ios, cli-animations, enable-native-assets, enable-record-use, enable-swift-package-manager, omit-legacy-version-file, enable-lldb-debugging, enable-uiscene-migration
[✗] Android toolchain - develop for Android devices [71ms]
    ✗ Unable to locate Android SDK.
[✓] Xcode - develop for iOS and macOS (Xcode 26.6) [337ms]
    • Xcode at /Applications/Xcode.app/Contents/Developer
    • Build 17F113
    • CocoaPods version 1.17.0
[✓] Chrome - develop for the web [3ms]
[✓] Connected device (2 available) [5.9s]
    • macOS (desktop) • macos  • darwin-arm64   • macOS 26.6.2 25G83 darwin-arm64
    • Chrome (web)    • chrome • web-javascript • Google Chrome 151.0.7922.174
[✓] Network resources [519ms]
! Doctor found issues in 1 category.
```

Clean uninstall (not executed):
```
brew uninstall --cask flutter
brew uninstall cocoapods && brew autoremove
brew cleanup --prune=all
rm -rf ~/.pub-cache ~/.dart-tool ~/.dartServer
rm -rf ~/Library/Developer/Xcode/DerivedData/Runner-*
```

## 汇总(与 Electron 既有探针并排)

| 指标 | GPUI 0.2.2 | Iced 0.14.0 | Flutter 3.47.1(universal) | Electron 43.4.0(隐藏空窗) |
|---|---|---|---|---|
| 干净 release 构建 | 76.9–81.6 s | 35.4 s | 18.5 s | n/a |
| 二进制 / 包 | 5.2 MiB(strip 4.0) | 10.4 MiB(strip 7.9) | `.app` 36.5 MiB(单架构约 20 MiB) | ZIP 116.5 MiB;`.app` 275.9 MiB |
| 空载 RSS | ~74 MB | ~96 MB | ~107 MB | 四进程直加 ~346 MB |
| 空载 physical footprint | 42.5 MB | 42.1 MB | 54.7–55.9 MB | 77.6 MB |
| 启动峰值 footprint | 95–103 MB | 138 MB | 175–184 MB | 未测 |
| 首帧 | 未测 | 未测 | 96 ms | 未测 |

# HCTL2 构建环境

这个目录描述 HCTL2 自己的构建环境。第一方代码使用 Buck2 原生目标；Tuwunel、Vikunja、Dagu、Herdr、Cinny 和 Static Web Server 以粗粒度外部子系统目标接入同一 action graph，但不把它们内部的 Cargo、Go 或 C 构建图改写成 Buck rules。正常发行只校验、整理锁定的二进制；需要更新 HCTL2 托管的 macOS Tuwunel 制品时，单独的 Buck 目标再调用上游 Cargo 构建。

`src/build/tools/buck2-bin` 是 Buck2 官方发行的 DotSlash 清单；`src/buck2` 是保持命令入口不变的薄启动器。Actionlint 1.7.12 与 ShellCheck 0.11.0 也用同一种官方 release + SHA-256 清单钉定，CI 用它们检查 workflow 和 shell action body。开发机可以安装 [DotSlash](https://dotslash-cli.com/)，也可以在 `src/` 产品工作区用固定版本和 SHA-256 的安装器准备它：

```bash
build/tools/install-dotslash /path/to/tools
export PATH=/path/to/tools:$PATH
```

随后在 `src/` 内直接使用 `./buck2` 入口。启动器会按 `src/build/tools/bazel-remote-bin` 中固定的官方二进制和 SHA-256，自动启动只监听 loopback 的标准 REAPI action cache；在 macOS 上还会把 Xcode version/build 与 SDK version 注入 Buck 配置，使宿主工具链升级参与 action key。缓存数据默认位于 `${XDG_CACHE_HOME:-$HOME/.cache}/hctl2/buck2-reapi`，最多 10 GiB，所有本机 hctl2 workplaces 共用；它只缓存 Buck 声明的 CAS/action results，不共享 daemon 或 `buck-out`。可用 `HCTL2_BUCK2_CACHE=0` 临时禁用，或用 `HCTL2_BUCK2_CACHE_DIR`、`HCTL2_BUCK2_CACHE_MAX_GIB` 调整位置和上限。

`src/.buckroot` 与 `src/.buckconfig` 定义完整的产品 workspace，仓库根不放置 Buck 配置或 target。发行所需的许可证和用户文档以显式快照放在 `src/packaging/release/assets/`，CI 机械校验它们与仓库文档事实源一致。Buck2、REAPI cache server 和匹配的 Prelude 都按清单自动下载并校验；第一方 Rust 构建及显式的 Tuwunel 原生构建目标使用锁定的 Rust 官方工具链。这些都是开发/CI 构建工具，不进入 HCTL2 最终用户运行包。

验证构建基础设施：

```bash
./buck2 targets root//build/platforms: toolchains//:
./buck2 build root//build/tests:rust_toolchain_probe
./buck2 test root//build/tests:rust_toolchain_test
./buck2 run toolchains//:rustc -- --version
./buck2 run toolchains//:rustfmt -- --version
```

构建和测试当前全部第一方 Rust 目标：

```bash
./buck2 build root//apps/... root//crates/... root//build/tests/...
./buck2 test root//apps/... root//crates/... root//build/tests/...
./buck2 build root//:clippy
```

`root//...` 还包含外部依赖和完整发行目标；日常第一方开发应使用上面的显式 target 集合。macOS Tuwunel 的原生重建是独立的 `root//packaging/dependencies:tuwunel-native-build`，默认标记为不兼容，因而不会被 `root//...` 意外请求；只有显式传入 `--config hctl2.tuwunel_native_build=1` 才启用。

当前发行目标平台为：

- `root//build/platforms:linux_x86_64_gnu`；
- `root//build/platforms:macos_x86_64`；
- `root//build/platforms:macos_arm64`。

其中 `macos` 是 HCTL2 产品平台名；映射到 Rust 或 Go 生态时，分别使用 `*-apple-darwin` 和 `darwin/*`。默认本机开发构建使用 Prelude 的宿主平台检测；发行任务必须显式选择上面的目标平台。

Rust 版本、三个目标平台的官方归档地址与 SHA-256 都在 `toolchains/rust/defs.bzl` 中声明。修改工具链或平台定义属于构建输入，会使对应 Buck2 action 失效并重新验证。

外部 crate 仍在 Cargo manifests 与 `Cargo.lock` 中声明，由固定版本的 Reindeer 生成 `third-party/rust/BUCK`。更新方法见[Rust 第三方依赖说明](../third-party/rust/README.md)。

CI 的第一方主检查分别运行在 Linux x86_64、macOS x86_64 和 macOS arm64 原生 runner，显式选择对应 Buck platform；Cargo 只在 Linux 保留为迁移期一致性检查。外部依赖与完整离线安装生命周期位于独立 workflow，不用第一方绿色检查代替发行健康状态。

当前没有采购或部署 Remote Execution，也没有托管的远端 cache。开发机由 loopback `bazel-remote` 提供标准 REAPI CAS/action cache，供多个 worktree 复用第一方构建结果，也可复用显式请求的 Tuwunel 原生重建。CI 不保存 `buck-out` 或本地 REAPI 数据：日常 macOS 发布从 HCTL2 的 GitHub Release 下载约 33–36 MiB、按 SHA-256 锁定的 Tuwunel 原生包，不再恢复 0.5–1 GiB cache 或执行 24 分钟以上的源码编译。未来切换到托管 REAPI 只需更换 endpoint 和凭证，不改 target 定义。

Buck2 OSS Prelude 只有在 genrule 明确标注本地执行倾向时才允许上传其 action result；六个正常组件 action、工具链拼装和大目录组包使用 `large_copy` label，只有显式的 `tuwunel-native-build` 需要 Cargo 联网并使用 `network_access`。不可变 action 内原有的 `.hctl2-*-sha256` marker 已删除，失效判断只由 Buck action key 负责。

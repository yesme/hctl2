# HCTL2 构建环境

这个目录只描述 HCTL2 第一方代码的构建环境。Tuwunel、Vikunja、Dagu、tmux、Cinny 和 Static Web Server 仍由 `src/packaging/dependencies` 按各自上游方式获取或编译，不进入它们内部的 Buck2 构建图。

`src/buck2` 是 Buck2 官方发行的 DotSlash 清单。开发机可以安装 [DotSlash](https://dotslash-cli.com/)，也可以在 `src/` 产品工作区用固定版本和 SHA-256 的安装器准备它：

```bash
build/tools/install-dotslash /path/to/tools
export PATH=/path/to/tools:$PATH
```

随后在 `src/` 内直接使用 `./buck2` 入口；同目录的 `.buckroot` 与 `.buckconfig` 把构建图限制在产品工作区。Buck2 和匹配的 Prelude 会按清单自动下载并校验，第一次执行还会下载当前平台的 Rust 官方工具链归档。

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
./buck2 build root//...
./buck2 test root//...
./buck2 build root//:clippy
```

当前发行目标平台为：

- `root//build/platforms:linux_x86_64_gnu`；
- `root//build/platforms:macos_x86_64`；
- `root//build/platforms:macos_arm64`。

其中 `macos` 是 HCTL2 产品平台名；映射到 Rust 或 Go 生态时，分别使用 `*-apple-darwin` 和 `darwin/*`。默认本机开发构建使用 Prelude 的宿主平台检测；发行任务必须显式选择上面的目标平台。

Rust 版本、三个目标平台的官方归档地址与 SHA-256 都在 `toolchains/rust/defs.bzl` 中声明。修改工具链或平台定义属于构建输入，会使对应 Buck2 action 失效并重新验证。

外部 crate 仍在 Cargo manifests 与 `Cargo.lock` 中声明，由固定版本的 Reindeer 生成 `third-party/rust/BUCK`。更新方法见[Rust 第三方依赖说明](../third-party/rust/README.md)。

CI 的第一方主检查分别运行在 Linux x86_64、macOS x86_64 和 macOS arm64 原生 runner，显式选择对应 Buck platform；Cargo 只在 Linux 保留为迁移期一致性检查。外部依赖的 native build 与完整离线安装生命周期位于独立 workflow，不用第一方绿色检查代替发行健康状态。

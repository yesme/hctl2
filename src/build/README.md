# HCTL2 构建环境

这个目录只描述 HCTL2 第一方代码的构建环境。Tuwunel、Vikunja、Dagu、tmux、Cinny 和 Static Web Server 仍由 `src/packaging/dependencies` 按各自上游方式获取或编译，不进入它们内部的 Buck2 构建图。

仓库根目录的 `buck2` 是 Buck2 官方发行的 DotSlash 清单。开发机需要先安装 [DotSlash](https://dotslash-cli.com/)，随后直接使用仓库内入口；Buck2 和匹配的 Prelude 会按清单自动下载并校验。第一次执行还会下载当前平台的 Rust 官方工具链归档。

验证构建基础设施：

```bash
./buck2 targets root//src/build/platforms: toolchains//:
./buck2 build root//src/build/tests:rust_toolchain_probe
./buck2 test root//src/build/tests:rust_toolchain_test
./buck2 run toolchains//:rustc -- --version
./buck2 run toolchains//:rustfmt -- --version
```

当前发行目标平台为：

- `root//src/build/platforms:linux_x86_64_gnu`；
- `root//src/build/platforms:macos_x86_64`；
- `root//src/build/platforms:macos_arm64`。

其中 `macos` 是 HCTL2 产品平台名；映射到 Rust 或 Go 生态时，分别使用 `*-apple-darwin` 和 `darwin/*`。默认本机开发构建使用 Prelude 的宿主平台检测；发行任务必须显式选择上面的目标平台。

Rust 版本、三个目标平台的官方归档地址与 SHA-256 都在 `toolchains/rust/defs.bzl` 中声明。修改工具链或平台定义属于构建输入，会使对应 Buck2 action 失效并重新验证。

# HCTL2 构建环境

这个目录描述 HCTL2 自己的构建环境。第一方代码使用 Buck2 原生目标；Tuwunel、Vikunja、Dagu、tmux、Cinny 和 Static Web Server 以粗粒度外部子系统目标接入同一 action graph，但不把它们内部的 Cargo、Go 或 C 构建图改写成 Buck rules。Buck2 决定何时获取或构建，真正的子系统编译仍调用上游原生构建系统。

`src/buck2` 是 Buck2 官方发行的 DotSlash 清单。开发机可以安装 [DotSlash](https://dotslash-cli.com/)，也可以在 `src/` 产品工作区用固定版本和 SHA-256 的安装器准备它：

```bash
build/tools/install-dotslash /path/to/tools
export PATH=/path/to/tools:$PATH
```

随后在 `src/` 内直接使用 `./buck2` 入口。仓库根的 `.buckroot` 与 `.buckconfig` 定义 Buck workspace；`src/` 仍是名为 `root` 的产品 cell，因此既能保持既有 `root//...` 标签，又能把根目录的许可证和用户文档作为发行目标的声明输入。Buck2 和匹配的 Prelude 会按清单自动下载并校验，第一次执行还会下载当前平台的 Rust 官方工具链归档。

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

当前没有采购或部署 Remote Execution/Remote Action Cache。相同 `buckd` 生命周期内，未变化的 action 由 Buck2 原生复用；GitHub-hosted runner 是临时机器，每个新 job 会从空的 action cache 开始。CI 不用另一套自制 fingerprint 或成品缓存冒充 Buck cache，未来接入标准 REAPI 服务时也不需要改 target 定义。

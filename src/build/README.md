# HCTL2 构建环境

这个目录描述 HCTL2 自己的构建环境。第一方代码使用 Buck2 原生目标；Tuwunel、Vikunja、Dagu、tmux、Cinny 和 Static Web Server 以粗粒度外部子系统目标接入同一 action graph，但不把它们内部的 Cargo、Go 或 C 构建图改写成 Buck rules。Buck2 决定何时获取或构建，真正的子系统编译仍调用上游原生构建系统。

`src/build/tools/buck2-bin` 是 Buck2 官方发行的 DotSlash 清单；`src/buck2` 是保持命令入口不变的薄启动器。开发机可以安装 [DotSlash](https://dotslash-cli.com/)，也可以在 `src/` 产品工作区用固定版本和 SHA-256 的安装器准备它：

```bash
build/tools/install-dotslash /path/to/tools
export PATH=/path/to/tools:$PATH
```

随后在 `src/` 内直接使用 `./buck2` 入口。启动器会按 `src/build/tools/bazel-remote-bin` 中固定的官方二进制和 SHA-256，自动启动只监听 loopback 的标准 REAPI action cache。缓存数据默认位于 `${XDG_CACHE_HOME:-$HOME/.cache}/hctl2/buck2-reapi`，最多 10 GiB，所有本机 hctl2 workplaces 共用；它只缓存 Buck 声明的 CAS/action results，不共享 daemon 或 `buck-out`。可用 `HCTL2_BUCK2_CACHE=0` 临时禁用，或用 `HCTL2_BUCK2_CACHE_DIR`、`HCTL2_BUCK2_CACHE_MAX_GIB` 调整位置和上限。

`src/.buckroot` 与 `src/.buckconfig` 定义完整的产品 workspace，仓库根不放置 Buck 配置或 target。发行所需的许可证和用户文档以显式快照放在 `src/packaging/release/assets/`，CI 机械校验它们与仓库文档事实源一致。Buck2、REAPI cache server 和匹配的 Prelude 都按清单自动下载并校验，第一次执行还会下载当前平台的 Rust 官方工具链归档。这些都是开发/CI 构建工具，不进入 HCTL2 最终用户运行包。

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

当前没有采购或部署 Remote Execution，也没有托管的远端 cache。开发机由 loopback `bazel-remote` 提供标准 REAPI CAS/action cache，因此多个 worktree 不再重复编译同一 Tuwunel 输入；macOS CI 用 `actions/cache` 持久化这个 REAPI 服务自己的数据目录，按工具链和外部依赖输入换代并从上一代 CAS 增量恢复。它不是另一套 fingerprint、Cargo cache 或最终成品缓存。Linux 外部包只解压官方二进制、冷构建足够短，CI 不为它持久化 REAPI cache。未来切换到托管 REAPI 只需更换 endpoint 和凭证，不改 target 定义。

Buck2 OSS Prelude 只有在 genrule 明确标注本地执行倾向时才允许上传其 action result；因此 Tuwunel 准备目标使用语义相符的 `network_access` label，工具链拼装和大目录组包使用 `large_copy` label。仅写 `cacheable = True` 不足以跨 workspace 复用；本次变更以第二个干净 workspace 的 cache-hit 作为验收。

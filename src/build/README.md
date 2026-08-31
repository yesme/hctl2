# HCTL2 构建环境

这个目录描述 HCTL2 自己的构建环境。第一方代码使用 Buck2 原生目标；Tuwunel、Vikunja、Dagu、Herdr、Cinny 和 Static Web Server 以粗粒度外部子系统目标接入同一 action graph，但不把它们内部的 Cargo、Go 或 C 构建图改写成 Buck rules。正常发行只校验、整理锁定的二进制；需要更新 HCTL2 托管的 macOS Tuwunel 制品时，单独的 Buck 目标再调用上游 Cargo 构建。

`src/build/tools/buck2-bin` 是 Buck2 官方发行的 DotSlash 清单；`src/buck2` 是保持命令入口不变的薄启动器。Actionlint 1.7.12、ShellCheck 0.11.0 与 Buck2 Change Detector 2026-08-20 也由官方 release 的 DotSlash 清单钉定，CI 分别用它们检查 workflow/shell 与计算受影响的 Buck targets。源码审计和 `btd` / `supertd` 边界见 [BTD 研究记录](../../docs/research/build-tools/buck2-change-detector.md)。这些工具只进入开发与 CI 环境。开发机可以安装 [DotSlash](https://dotslash-cli.com/)，也可以在 `src/` 产品工作区用固定版本和 SHA-256 的安装器准备它：

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

构建和测试当前全部第一方目标：

```bash
./buck2 test --build-default-info \
  root//apps/... root//crates/... root//build/tests/... \
  root//:clippy root//packaging/release:first-party
```

`root//...` 还包含外部依赖和完整发行目标；日常第一方开发应使用上面的显式 target 集合。各 production target 通过 Buck 原生 `tests` 关联测试，Clippy 先在 crate/app 内聚合，再由 `root//:clippy` 提供全量入口。`ci:fast`、`ci:platform`、`ci:integration`、`ci:release` 和 `ci:full` labels 表达验证类别，不编码 runner 厂商。macOS Tuwunel 的原生重建是独立的 `root//packaging/dependencies:tuwunel-native-build`，默认标记为不兼容，因而不会被 `root//...` 意外请求；只有显式传入 `--config hctl2.tuwunel_native_build=1` 才启用。

当前发行目标平台为：

- `root//build/platforms:linux_x86_64_gnu`；
- `root//build/platforms:macos_x86_64`；
- `root//build/platforms:macos_arm64`。

其中 `macos` 是 HCTL2 产品平台名；映射到 Rust 或 Go 生态时，分别使用 `*-apple-darwin` 和 `darwin/*`。默认本机开发构建使用 Prelude 的宿主平台检测；发行任务必须显式选择上面的目标平台。

Rust 版本、三个目标平台的官方归档地址与 SHA-256 都在 `toolchains/rust/defs.bzl` 中声明。修改工具链或平台定义属于构建输入，会使对应 Buck2 action 失效并重新验证。

外部 crate 仍在 Cargo manifests 与 `Cargo.lock` 中声明，由固定版本的 Reindeer 生成 `third-party/rust/BUCK`。更新方法见[Rust 第三方依赖说明](../third-party/rust/README.md)。

CI 在 Linux x86_64 导出 base/diff 两份 Buck 图，用 BTD 保守选择带 `ci:fast` 或 `ci:platform` 的受影响 target，再分别到 Linux x86_64、macOS x86_64 和 macOS arm64 原生 runner 显式验证对应 Buck platform。BTD 或导图失败时回退到上面的全量入口。Cargo 只在 Rust/Cargo 输入变化及定期基线中保留 `fmt` 与 locked metadata 检查；编译、测试和 Clippy 不再平行执行第二遍。外部依赖与完整离线安装生命周期位于 Release workflow，不用第一方绿色检查代替发行健康状态。

根目录文档在 Buck cell 之外，由 `.gitattributes` 的 `hctl-doc` 属性分为 design、spec、delivery、research、memo 和 memo-review。CI 将非 memo profile 映射到 `root//build/docs:profile-*` test suite；普通 memo 不启动文档 runner，`.memo/review` 只运行基线检查。语气、架构重复和术语必要性仍由 human/LLM 按对应层审阅，不伪装成确定性 lint。

当前没有采购或部署 Remote Execution，也没有托管的远端 cache。开发机由 loopback `bazel-remote` 提供标准 REAPI CAS/action cache，供多个 worktree 复用第一方构建结果，也可复用显式请求的 Tuwunel 原生重建。CI 不保存 `buck-out` 或本地 REAPI 数据：日常 macOS 发布从 HCTL2 的 GitHub Release 下载约 33–36 MiB、按 SHA-256 锁定的 Tuwunel 原生包，不再恢复 0.5–1 GiB cache 或执行 24 分钟以上的源码编译。未来切换到托管 REAPI 只需更换 endpoint 和凭证，不改 target 定义。

Buck2 OSS Prelude 只有在 genrule 明确标注本地执行倾向时才允许上传其 action result；六个正常组件 action、工具链拼装和大目录组包使用 `large_copy` label，只有显式的 `tuwunel-native-build` 需要 Cargo 联网并使用 `network_access`。不可变 action 内原有的 `.hctl2-*-sha256` marker 已删除，失效判断只由 Buck action key 负责。

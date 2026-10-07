# HCTL2 构建环境

发行压缩器由 `build/tools/xz.bzl` 钉定为 [xz 5.8.4](../../docs/research/build-tools/xz.md)，使用 pkgx 的三平台预编译包及包内 liblzma。Buck 原生下载、验摘要；仅构建时使用，不要求安装 pkgx，也不进入用户运行包。回归入口为 `./buck2 test root//build/tests:xz_test`。

这个目录描述 HCTL2 自己的构建环境。第一方代码使用 Buck2 原生目标；Tuwunel、Vikunja、Dagu、Herdr、Cinny 和 Static Web Server 以粗粒度外部子系统目标接入同一 action graph，但不把它们内部的 Cargo、Go 或 C 构建图改写成 Buck rules。正常发行只校验、整理锁定的二进制；需要更新 HCTL2 托管的 macOS Tuwunel 制品时，单独的 Buck 目标再调用上游 Cargo 构建。

`src/build/tools/buck2-bin` 是 Buck2 官方发行的 DotSlash 清单；`src/buck2` 是保持命令入口不变的薄启动器。Actionlint 1.7.12、ShellCheck 0.11.0、Buck2 Change Detector 2026-08-20 与 jq 1.8.2 也由官方发布页的 DotSlash 清单固定版本。宿主 Python（cpython 3.12.14，python-build-standalone `install_only_stripped`）固定于 `build/tools/host-bin/python3`：Buck2 action 环境不含 PATH，默认裸名 `python3` 会落到系统解释器（macOS 3.9.6，低于 prelude build-script shim 要求的 3.12），因此 `src/buck2` 启动器解析钉定解释器的绝对路径并注入 `hctl2.python` config；同一趟还注入 `hctl2.cc/cxx/ar` 绝对路径（`from_any_dir` 以 `os.execl` 重入编译器，不搜 PATH，裸名必然失败；macOS 取自动注 SDK 的 `/usr/bin` xcrun shim）。CI 分别用它们检查工作流与 Shell 脚本、计算受影响的 Buck 目标，以及解析 BTD 输出的 JSON Lines。源码审计和职责边界见 [DotSlash 安装](../../docs/research/build-tools/install-dotslash.md)、[BTD](../../docs/research/build-tools/buck2-change-detector.md)、[jq](../../docs/research/build-tools/jq.md)及[宿主 Python](../../docs/research/build-tools/python-build-standalone.md)研究记录。这些工具只进入开发与 CI 环境。

GitHub Actions 的 Linux 执行器通过锁定到 40 位提交的官方 `facebook/install-dotslash` Action 安装 DotSlash，再核对结果必须等于 `dotslash.env` 的固定版本。官方 Action 当前跟随上游最新 Release，没有版本输入；额外核对使版本变化先失败、再经显式升级，不会静默漂移。它在 GitHub 托管的 macOS 15 上会把 BSD `sha256sum` 当成 GNU 实现并错误使用 `--check`；修复进入上游前，macOS CI 与开发机共用仓库的摘要锁定安装器，不降级到缺少摘要校验的旧版 Action。开发机在 `src/` 产品工作区运行一次即可，默认建议放入已经加入 `PATH` 的用户级目录：

```bash
build/tools/install-dotslash "${XDG_BIN_HOME:-$HOME/.local/bin}"
dotslash --version
```

如果该目录尚未加入 `PATH`，由开发者按所用 shell 的正常方式加入一次。安装器不修改 shell 配置；它按 `dotslash.env` 选择宿主平台官方包并校验 SHA-256。此后 Buck2、BTD、jq、Actionlint、ShellCheck 和 cache server 均由各自 DotSlash 清单准备，不再分别安装。 例外只有 Code 与 Release workflow 的 path-filter job：它在安装 DotSlash 之前就要判断本次 PR 更新的验证区间，因此那一步用 runner 自带 GitHub CLI 的内置 jq（`gh api --jq`）读取上一 head 的 workflow 结论，不经 `jq-bin`；查询失败、答案无法求值与无法归类的错误分别如实报出并回退完整检查（对 gh stderr 的分类是诊断用的启发式，不是完备分类），回归用例在 `build/tests/check_validation_range.sh`。更新 Action 提交或 DotSlash 版本属于构建供应链变更，应连同三平台验证一起审阅。

随后在 `src/` 内直接使用 `./buck2` 入口。启动器通过固定版本的 Process Compose 和声明式 [`reapi/local-cache.yaml`](./reapi/local-cache.yaml) 启动只监听 loopback 的标准 REAPI action cache；进程的后台化、PID、健康检查、重启、日志和关闭全部由 Process Compose 管理，cache server 仍是 `src/build/tools/bazel-remote-bin` 固定的官方 `bazel-remote`。在 macOS 上，启动器还会把 Xcode version/build 与 SDK version 注入 Buck 配置，使宿主工具链升级参与 action key。缓存数据默认位于 `${XDG_CACHE_HOME:-$HOME/.cache}/hctl2/buck2-reapi`，最多 10 GiB，所有本机 hctl2 worktree 共用；它只缓存 Buck 声明的 CAS/action results，不共享 daemon 或 `buck-out`。可用 `HCTL2_BUCK2_CACHE=0` 临时禁用，或用 `HCTL2_BUCK2_CACHE_DIR`、`HCTL2_BUCK2_CACHE_MAX_GIB` 调整位置和上限。

`HCTL2_BUCK2_CACHE` 有三种模式：默认 `local` 使用上述本机缓存，`0` 关闭 REAPI 并进入离线构建，`remote` 使用外部缓存且不启动本机服务。外部缓存采用 Buck2 原生 `.buckconfig.local` 覆盖：

```bash
cp build/reapi/remote-cache.buckconfig.example .buckconfig.local
# 修改端点；证书与私钥留在权限为 0600 的 PEM 文件中，只传路径。
export HCTL2_REAPI_CA_CERTS=/path/to/ca.pem
export HCTL2_REAPI_CLIENT_CERT=/path/to/client-and-key.pem
HCTL2_BUCK2_CACHE=remote ./buck2 test root//build/tests:rust_toolchain_test
```

`.buckconfig.local` 已排除版本控制；示例使用 Buck2 官方的 `action_cache_address`、`cas_address`、TLS CA 和 mTLS 客户端证书，不在启动器参数或动作图中复制端点、凭据或另一份指纹。当前 Buck2 有一个[公开问题](https://github.com/facebook/buck2/issues/1445)：`http_headers` 可能进入事件日志，因此 HCTL2 模板不接受持有者令牌（bearer token）；只有在固定版本确认修复后才能改判。仓库当前没有持久远端端点，所以 GitHub 托管的 PR 执行器默认不配置凭据，并继续显式关闭缓存。将来只有受信开发机、`main` 或标签执行器取得写证书后才启用。来自分叉仓库的 PR 不接收写凭据，个人日用 Mac 也不注册为公开仓库的自动执行器。

仓库根的 `.buckroot` 与 `.buckconfig` 定义 Buck2 项目：仓库根是 `repo` cell（根文档、`docs/`、`.memo/` 与 `LICENSE`），`src/` 是名为 `root` 的产品 workspace cell，所有 `root//...` 标签不变。发行所需的许可证与用户文档由 `repo//:LICENSE`、`repo//:usage` 直接从仓库文档事实源导出，不再保存快照副本；决定过程见[Buck2 项目根调研](../../docs/research/build-tools/buck2-project-root.md)。Buck2、REAPI cache server 和匹配的 Prelude 都按清单自动下载并校验；第一方 Rust 构建及显式的 Tuwunel 原生构建目标使用锁定的 Rust 官方工具链。这些都是开发/CI 构建工具，不进入 HCTL2 最终用户运行包。

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
  root//agency/... root//apps/... root//crates/... root//build/tests/... \
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

同一套目标选择不依赖 GitHub Actions。`src/build/ci/affected-targets` 对两个提交生成 Buck 图并调用 BTD；GitHub Actions 把输出的同一组目标送到三个原生平台执行器，选择失败时扩大为全量第一方目标。其他 CI 可以复用该入口和同一组 Buck targets，并把结果作为 GitHub 提交状态报回现有分支保护规则。仓库不绑定执行器厂商，也不把三平台证据折叠成单平台。

PR 的 Code 与 Release workflow 会读取 `pull_request/synchronize` 的旧、新 head。旧 head 是新 head 的祖先，且旧 head 上对应 workflow 已成功时，本轮只把旧 head 到 GitHub 当前 test merge 的变化交给路径分类和 BTD；这样仍覆盖最新 base 的兼容性。没有受影响 target 或发行输入时，当前 head 的 required gate 仍成功出现，但不启动三平台重构建。旧结果缺失、API 查询失败、rebase 或强推造成历史改写时，回退到 base 到当前 test merge 的完整 PR diff。strict branch protection 继续生效；更新落后分支时应使用 GitHub 的 merge 形式 `Update branch`，保留可验证的提交链。

## Ubuntu 开发机准备

新机器只需要装下面三样，其余（buck2、rustc、Python、jq 等）由 `./buck2` 经 DotSlash 与 Buck2 按校验和自己准备。下面是所有者 2026-10-07 在 Ubuntu 26.04 上的安装过程。

1. **gh**，用 GitHub 官方 apt 源。开 PR、看 CI 用的是宿主上的 gh；Ubuntu 自带的版本太旧（2.46），`--json` 缺新字段，比如发行资产的 `digest`。构建与测试另用 `build/tools/gh-bin` 钉住的版本，不受宿主 gh 影响。

   ```bash
   sudo apt update && sudo apt install software-properties-common curl -y
   curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/etc/apt/keyrings/githubcli-archive-keyring.gpg
   echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
   sudo apt update
   sudo apt install gh -y
   ```

2. **DotSlash**，版本与摘要以 `build/tools/dotslash.env` 为准，解压到 `/usr/local/bin`。在 `src/` 内执行：

   ```bash
   . build/tools/dotslash.env
   curl -fsSLO "https://github.com/facebook/dotslash/releases/download/$DOTSLASH_VERSION/$DOTSLASH_LINUX_X86_64_ASSET"
   echo "$DOTSLASH_LINUX_X86_64_SHA256  $DOTSLASH_LINUX_X86_64_ASSET" | sha256sum -c -
   sudo tar -xzf "$DOTSLASH_LINUX_X86_64_ASSET" -C /usr/local/bin
   dotslash --version
   ```

   `dotslash --version` 必须等于 `DOTSLASH_VERSION`。PATH 前面不要留别的 dotslash，比如 `cargo install` 装的旧版。

3. **clang 与 lld**。这是 Buck2 prelude 默认的 C/C++ 工具链：`system_cxx_toolchain` 用 clang 编译、`clang++` 链接，Linux 上加 `-fuse-ld=lld`。只有 GCC 不够。缺 clang、clang++ 或 lld 时，`./buck2 build`、`run`、`test` 会直接报错，并提示下面这条安装命令。CI 的 ubuntu-24.04 镜像自带 clang 18 与 lld；Ubuntu 26.04 的 apt 装出的是 21。

   ```bash
   sudo apt install clang lld
   ```

装好后在 `src/` 跑一遍 CI Linux 作业的全量目标，确认全绿：

```bash
./buck2 test --build-default-info root//agency/... root//apps/... root//crates/... root//build/tests/... root//:clippy root//packaging/release:first-party --target-platforms root//build/platforms:linux_x86_64_gnu
```

## macOS 开发机准备

新机器同样只需要装下面三样，其余由 `./buck2` 经 DotSlash 与 Buck2 自己准备。macOS 的 C/C++ 工具链仍是 prelude 默认的 clang，链接用系统 `ld64`，不装 lld。下面是 2026-10-07 在这台 Apple Silicon 开发机上核对过的装法（Darwin arm64，Xcode 26.6，Apple clang 21.0.0，gh 2.102.0，DotSlash 0.5.9）。

1. **gh**，用 Homebrew。开 PR、看 CI 用宿主上的 gh。构建与测试另用 `build/tools/gh-bin` 钉住的版本。

   ```bash
   brew install gh
   ```

2. **DotSlash**，版本与摘要以 `build/tools/dotslash.env` 为准，解压到 `/usr/local/bin`。在 `src/` 内执行。Apple Silicon 用 arm64 那一行；Intel 把资产名和摘要换成 `DOTSLASH_MACOS_X86_64_*`。macOS 的 `shasum` 不是 GNU `sha256sum`，不要加 GNU 的 `--check` 写法。

   ```bash
   . build/tools/dotslash.env
   curl -fsSLO "https://github.com/facebook/dotslash/releases/download/$DOTSLASH_VERSION/$DOTSLASH_MACOS_ARM64_ASSET"
   echo "$DOTSLASH_MACOS_ARM64_SHA256  $DOTSLASH_MACOS_ARM64_ASSET" | shasum -a 256 -c -
   sudo tar -xzf "$DOTSLASH_MACOS_ARM64_ASSET" -C /usr/local/bin
   dotslash --version
   ```

   `dotslash --version` 必须等于 `DOTSLASH_VERSION`。PATH 前面不要留别的 dotslash。

3. **Command Line Tools**。要装的是这一包，不是完整的 Xcode.app。`/usr/bin/clang` 在没装它时也在，只是个垫片，`--version` 跑不起来。装好之后 clang 能用，macOS SDK 也在。`xcodebuild` 只有完整 Xcode 才有，构建不需要它；装了完整 Xcode 时启动器会多记下它的版本，没装就留空。`./buck2 build`、`run`、`test` 在 clang 还不能跑时直接报错，并提示下面这条命令。`--version` 不要求这套工具链。

   ```bash
   xcode-select --install
   ```

装好后在 `src/` 跑一遍本机平台的全量目标。Apple Silicon：

```bash
./buck2 test --build-default-info root//agency/... root//apps/... root//crates/... root//build/tests/... root//:clippy root//packaging/release:first-party --target-platforms root//build/platforms:macos_arm64
```

Intel 把平台换成 `root//build/platforms:macos_x86_64`。

## 本机 cache 运维

`./buck2` 会按需启动 Process Compose 项目；如果同一 loopback endpoint 已经有旧版或用户自行管理的 `bazel-remote`，则直接复用而不争夺端口。平时不需要单独管理。需要检查日志或主动停止由 Process Compose 启动的实例时，直接使用它的原生命令，不再经过仓库自建的同义 CLI：

```bash
cache_root="${HCTL2_BUCK2_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/hctl2/buck2-reapi}"
XDG_CONFIG_HOME="$cache_root/state/config" build/tools/process-compose-bin --use-uds --unix-socket "$cache_root/state/process-compose.sock" process list
XDG_CONFIG_HOME="$cache_root/state/config" build/tools/process-compose-bin --use-uds --unix-socket "$cache_root/state/process-compose.sock" process logs buck2-reapi-cache
XDG_CONFIG_HOME="$cache_root/state/config" build/tools/process-compose-bin --use-uds --unix-socket "$cache_root/state/process-compose.sock" down
```

之前的冷构建、写入缓存和复用测量属于一次性选型证据，结果保留在 memo；仓库不长期维护一套 benchmark 生命周期框架。未来接入真正的远端 cache 时，再用 Buck 事件日志针对真实网络测量。

根目录文档是 `repo` cell 的普通输入（`repo//:docs_tree`），由 `.gitattributes` 的 `hctl-doc` 属性分为 design、spec、delivery、research、memo 和 memo-review。CI 将非 memo profile 映射到 `root//build/docs:profile-*` test suite；普通 memo 不启动文档 runner，`.memo/review` 只运行基线检查。语气、架构重复和术语必要性仍由 human/LLM 按对应层审阅，不伪装成确定性 lint。

当前没有采购或部署远端执行（Remote Execution），也没有持久远端缓存。开发机由回环地址上的 `bazel-remote` 提供标准 REAPI CAS 与动作缓存，供多个工作树复用第一方构建结果，也可复用显式请求的 Tuwunel 原生重建。CI 不保存 `buck-out` 或本地 REAPI 数据：日常 macOS 发布从 HCTL2 的 GitHub Release 下载约 33–36 MiB、按 SHA-256 固定的 Tuwunel 原生包，不再恢复 0.5–1 GiB 缓存或执行 24 分钟以上的源码编译。未来接入持久 REAPI 时，只配置 `.buckconfig.local`、mTLS 证书路径和 `HCTL2_BUCK2_CACHE=remote`，不改目标定义。

Buck2 OSS Prelude 只有在 genrule 明确标注本地执行倾向时才允许上传其 action result；六个正常组件 action、工具链拼装和大目录组包使用 `large_copy` label，只有显式的 `tuwunel-native-build` 需要 Cargo 联网并使用 `network_access`。不可变 action 内原有的 `.hctl2-*-sha256` marker 已删除，失效判断只由 Buck action key 负责。

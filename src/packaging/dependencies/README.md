# 依赖打包

这个目录把 HCTL2 的四类外部运行依赖制作成按目标平台区分的两份归档：可离线安装的运行包，以及同 Release 发布、不参与安装的源码伴随包。四类依赖是 Chatroom（Tuwunel 服务端与 Cinny 浏览器客户端）、Kanban（Vikunja）、Workflow（Dagu）和 Terminal（tmux）。联网下载、必要的源码编译、动态库收集、签名和许可证归档都发生在发布构建机；最终用户不需要 Rust、Python、Node.js、Homebrew 或 Linux 构建工具。

## 代码树边界

构建代码按三层组织，避免把 OS 差异塞进公共流程：

```text
common/action.sh        组件 action 共用的最小宿主 helper
common/                 预编译组件准备、包组装、生命周期和整包测试
platforms/linux/        Linux 制品准备、ELF 校验、运行时与 GNU tar 归档
platforms/macos/        Mach-O 公共校验、独立 Tuwunel 构建、组包与运行时
lock.json               版本、target identity、运行参数、外部制品与工具链的唯一事实源
defs.bzl、BUCK           原生下载校验、metadata、准备、组包与测试目标
test-package.sh         Buck sh_test 调用的整包生命周期测试体
```

Buck 从 `lock.json` 为选定平台生成只读的 `build-metadata.sh`；脚本不再维护第二份版本、URL、摘要或 `uname` 分发表。新增 CPU 架构时应扩展 lock、Buck platform 和对应 platform 实现；CI 与发布入口仍只选择 Buck platform。

三个发布 target 都要求原生构建，不在另一架构上伪装交叉构建：

| target | 构建宿主 | 组件来源 |
| --- | --- | --- |
| `linux-x86_64` | Linux x86_64 | 六项第三方运行内容均消费上游官方发行包 |
| `macos-aarch64` | Apple Silicon macOS 15+ | Vikunja/Dagu/tmux/Cinny/Static Web Server 官方包；Tuwunel 锁定源码 |
| `macos-x86_64` | Intel macOS 15+ | Vikunja/Dagu/tmux/Cinny/Static Web Server 官方包；Tuwunel 锁定源码 |

Intel 发布包必须在 Intel Mac runner 上产出；Apple Silicon 上的 Rosetta 或临时 `--target x86_64-apple-darwin` 不能替代它，因为 Tuwunel 原生构建、Mach-O 闭包和最终生命周期都要按真实目标验证。

## 构建与验证

发布和 CI 的入口都在 `src/` 内。目标根据显式 Buck platform 选择外部制品；URL 与 SHA-256 由 `http_file`/`http_archive` 规则声明，macOS Tuwunel 则由一个粗粒度 action 调用锁定的 Cargo 与 Rust 工具链：

```bash
./buck2 build root//packaging/dependencies:package \
  --target-platforms root//build/platforms:linux_x86_64_gnu \
  --out /absolute/path/dependency-packages

./buck2 test root//packaging/dependencies:package-test \
  --target-platforms root//build/platforms:linux_x86_64_gnu
```

`metadata`、六个组件 action、`package` 和 `package-test` 是同一张 action graph 上的连续合同。`tuwunel` 只声明 Tuwunel 源码或官方包、固定 Rust 工具链、最小公共 helper、macOS Mach-O helper 和独立原生构建体；`vikunja`、`dagu`、`tmux`、`cinny` 与 `static-web-server` 各自只声明对应官方制品和必要配置。源码伴随包所需归档由 `package` 直接消费 Buck 下载目标，不再复制进组件 action。任一预编译组件的制品、配置或专属脚本变化因而不会使其他组件失效，更不会让 macOS Tuwunel 重新编译。workflow 不理解这些 action 内部的下载、编译和组包顺序。测试会校验运行包与源码包、离线安装、幂等重装、完整启动、smoke 和停止。

开发机由 loopback `bazel-remote` 在各 worktree 之间共享标准 REAPI CAS/action results；macOS CI 用 `actions/cache` 持久化同一服务的数据目录。外层 key 包含 runner image、Buck/cache 工具、执行配置、Rust 工具链以及 Tuwunel action 的全部受控输入，精确恢复时 CI 还会机械断言 `tuwunel` 来自 REAPI cache。第一方 Code workflow 不缓存 `buck-out`，也不另建平行 fingerprint、Cargo cache 或最终依赖包缓存。导出的目录包含：

```text
hctl2-<version>-<target>.tar.gz
hctl2-<version>-<target>.tar.gz.sha256
hctl2-<version>-<target>-sources.tar.gz
hctl2-<version>-<target>-sources.tar.gz.sha256
```

Linux 构建只需基本归档工具和用于解开 Tuwunel 官方包的 `dpkg-deb`，不需要 Rust 或 C toolchain，也不调用 `apt-get`。Static Web Server 和 tmux 都使用上游完全静态链接的二进制；其他动态链接产物仍必须使用支持范围内最旧的 glibc 构建基线。

macOS 构建需要 Xcode Command Line Tools。`./buck2` 会把当前 Xcode version/build 与 macOS SDK version 注入 Buck 配置，因此工具链升级会改变 Tuwunel action key；action 开始时再次核对身份，并把它写入构建环境 manifest。Buck 直接下载并校验 Tuwunel 要求的 Rust 1.95.0 官方组件，把该工具链交给 Cargo。Tuwunel 编译 action 同时收集其非系统 dylib 闭包作为声明输出，最终组包不再从另一个 action 的宿主绝对路径偷读库文件。tmux 与 Static Web Server 直接使用上游目标架构二进制，不参与本地构建。产品最低基线为 **macOS 15**，与 `tmux/tmux-builds` 官方 Darwin 发行物声明的最低系统一致；兼容性检查同时识别现代 `LC_BUILD_VERSION` 和 Intel 链接器仍可能生成的 `LC_VERSION_MIN_MACOSX`，任何随包 Mach-O 都不得要求高于这个基线。发布包会把其他组件可能存在的非系统 dylib 改写为 `@loader_path`、做 ad-hoc 签名，并拒绝残留 `/opt/homebrew`、`/usr/local`、`@rpath` 或构建缓存路径的依赖。

## 供应链内容

运行安装包包括：

- Tuwunel、Vikunja、Dagu、tmux 四个上游可执行文件和所需的非系统动态库；tmux 来自官方 `tmux-builds`，Linux 为静态 ELF，macOS 只链接系统 dylib；
- Cinny 官方 Web 发行内容，以及只绑定 loopback 的官方 `static-web-server` 单二进制；
- `hctl2-services` 以及公共生命周期代码和目标平台 runtime hook；
- target、构建环境、版本、commit、构建输入 digest 和最终二进制 digest；
- HCTL2 与所有分发依赖的许可证；
- 根归档与 payload 两层 SHA-256 清单。

源码伴随包包括四类依赖和 Static Web Server 的锁定上游源码。`sources.tsv` 记录版本、commit、归档 digest 与纳入原因，`target.tsv` 标识对应平台，`SOURCE-MANIFEST.sha256` 校验整包。Dagu、Vikunja 和 Cinny 标为 `corresponding-source`，其余标为 `reproducibility`。tmux 运行包另纳入 `tmux-builds` 官方许可证集合，覆盖其静态链接依赖。源码包没有安装器，也不进入安装前缀。

运行包与源码包必须在同一个 Release 下载位置以相同方式、无额外费用提供；运行包内的 `SOURCES.md` 指向精确的源码包名。真正通过 U 盘或其他离线介质再次分发时，应同时携带两份归档。当前各平台体积见[实现证据](../../../docs/research/README.md#执行面已选依赖的运维与-footprint)；Intel macOS 仍需在对应 runner 上刷新。

## 用户流程

把 `<target>` 换成下载包名中的目标：

```bash
tar -xzf hctl2-0.0.0-<target>.tar.gz
cd hctl2-0.0.0-<target>
./install.sh
~/.local/bin/hctl2-services start
~/.local/bin/hctl2-services smoke
```

默认安装到 `$HOME/.local`；`--prefix` 可以指定其他绝对路径。状态、数据、secret、日志和 PID 位于版本化安装之外的 `${XDG_STATE_HOME:-$HOME/.local/state}/hctl2`。macOS 为避开 Unix socket 路径上限，把 tmux socket 放在 owner-only 的短 `/tmp/hctl2-tmux-<uid>/` 目录中，并用状态根目录哈希隔离并行 harness；Linux socket 仍位于状态根目录。

完整命令见[HCTL2 使用说明](../../../docs/usage.md)。

## 运行策略

| 组件 | 版本 | 本地端点 |
| --- | --- | --- |
| Tuwunel | 1.9.0 | `http://127.0.0.1:6167` |
| Cinny | 4.12.6 | `http://127.0.0.1:6168/` |
| Vikunja | 2.5.0 | `http://127.0.0.1:3456` |
| Dagu | 2.15.1 | `http://127.0.0.1:18080` |
| tmux | 3.7c | owner-only Unix socket |

所有 listener 都绑定 loopback。Cinny 与 Tuwunel 合在一起是 Chatroom 解决方案，并不增加第五类执行依赖；它是随包的互操作与查看客户端，不是 HCTL2 Workbench。Cinny 只允许连接随包 Tuwunel，并启用 hash router 适配内部静态服务。HCTL Room 的本地 Tuwunel 配置关闭 federation 与房间加密；Dagu 只在 loopback listener 上关闭认证；Vikunja 首次启动时生成随机本地 secret。

# 依赖打包

这个目录把 HCTL2 的四类外部运行依赖制作成按目标平台区分的两份归档：可离线安装的运行包，以及同 Release 发布、不参与安装的源码伴随包。四类依赖是 Chatroom（Tuwunel 服务端与 Element Web 浏览器客户端）、Kanban（Vikunja）、Workflow（Dagu）和 Terminal（tmux）。联网下载、源码编译、动态库收集、签名和许可证归档都发生在发布构建机；最终用户不需要 Rust、Python、Node.js、Homebrew 或 Linux 构建工具。

## 代码树边界

构建代码按三层组织，避免把 OS 差异塞进公共流程：

```text
common/                 平台无关的下载校验、包组装、生命周期和整包测试
platforms/linux/        Linux 构建、ELF 依赖收集、运行时与 GNU tar 归档
platforms/macos/        macOS 原生构建、Mach-O 重定位/签名、运行时与 BSD tar 归档
targets/*.sh            每个 OS/CPU 组合的 target ID、Rust triple、发行物 URL 和 SHA-256
*-<target>.sh           明确的 bootstrap、build-package、test-package 入口
bootstrap.sh            兼容入口：只按当前宿主机分派到对应 target
build-package.sh        同上
test-package.sh         同上
```

`common/versions.sh` 锁定跨平台版本和源码 commit；`platforms/<os>/versions.sh` 只锁定该 OS 的构建输入。`targets/` 不包含构建过程。新增 CPU 架构时应增加 target 描述和三个显式入口；只有构建机制发生变化时才修改 platform 层。

三个发布 target 都要求原生构建，不在另一架构上伪装交叉构建：

| target | 构建宿主 | 组件来源 |
| --- | --- | --- |
| `linux-x86_64` | Linux x86_64 | Tuwunel/Vikunja/Dagu/Element Web 官方包；tmux 锁定源码 |
| `macos-aarch64` | Apple Silicon macOS | Vikunja/Dagu/Element Web 官方包；Tuwunel/tmux 锁定源码 |
| `macos-x86_64` | Intel macOS | Vikunja/Dagu/Element Web 官方包；Tuwunel/tmux 锁定源码 |

Intel 发布包必须在 Intel Mac runner 上产出；Apple Silicon 上的 Rosetta 或临时 `--target x86_64-apple-darwin` 不能替代它，因为 Homebrew 头文件、链接库、Mach-O 闭包和最终生命周期都要按真实目标验证。

## 构建与验证

在当前宿主机生成原生包：

```bash
src/packaging/dependencies/build-package.sh
```

也可以显式选择与宿主架构一致的入口：

```bash
src/packaging/dependencies/build-package-linux-x86_64.sh
src/packaging/dependencies/build-package-macos-aarch64.sh
src/packaging/dependencies/build-package-macos-x86_64.sh
```

把 `build-package` 换为 `bootstrap` 只准备依赖和内部静态文件服务；换为 `test-package` 会继续校验运行包与源码包的内容，执行离线安装、幂等重装、完整启动、smoke 和停止，是发布前必须通过的完整验证。

构建输入默认放在 `${XDG_CACHE_HOME:-$HOME/.cache}/hctl2/dependencies/<target>`。设置绝对路径 `HCTL2_BUILD_CACHE` 可以隔离或复用另一份缓存；不同 target 即使共享缓存根目录也不会混用产物。构建在 `src/dist/` 同时输出以下文件，不提交 Git：

```text
hctl2-<version>-<target>.tar.gz
hctl2-<version>-<target>.tar.gz.sha256
hctl2-<version>-<target>-sources.tar.gz
hctl2-<version>-<target>-sources.tar.gz.sha256
```

Linux 构建需要 Ubuntu/Debian 的 `apt-get`、`dpkg-deb`、GCC、binutils、make、pkg-config，以及 HCTL2 workspace 锁定的 Rust 1.98.0 toolchain。正式发行必须使用支持范围内最旧的 glibc 构建基线。

macOS 构建需要 Xcode Command Line Tools、Homebrew `rustup`，以及锁定版本的 Homebrew `bison` 和 `pkgconf`。脚本自行安装 Tuwunel 要求的 Rust 1.95.0 toolchain，并从锁定源码为 tmux 构建 libevent、ncurses 和 utf8proc；这与开发 HCTL2 Rust workspace 使用的 1.98.0 分开。所有自建 Mach-O 都以 macOS 13 为 deployment target。发布包会把非系统 dylib 改写为 `@loader_path`、做 ad-hoc 签名，并拒绝残留 `/opt/homebrew`、`/usr/local`、`@rpath` 或构建缓存路径的依赖。

## 供应链内容

运行安装包包括：

- Tuwunel、Vikunja、Dagu、tmux 四个上游可执行文件和所需的非系统动态库；
- Element Web 官方发行内容，以及只绑定 loopback 的内部 `hctl2-web-server`；
- `hctl2-services` 以及公共生命周期代码和目标平台 runtime hook；
- target、构建环境、版本、commit、构建输入 digest 和最终二进制 digest；
- HCTL2 与所有分发依赖的许可证；
- 根归档与 payload 两层 SHA-256 清单。

源码伴随包包括四类依赖的锁定上游源码；macOS 包还包括随包 dylib 的锁定源码。`sources.tsv` 记录版本、commit、归档 digest 与纳入原因，`target.tsv` 标识对应平台，`SOURCE-MANIFEST.sha256` 校验整包。Dagu、Vikunja 和 Element Web 标为 `corresponding-source`，其余标为 `reproducibility`。源码包没有安装器，也不进入安装前缀。

运行包与源码包必须在同一个 Release 下载位置以相同方式、无额外费用提供；运行包内的 `SOURCES.md` 指向精确的源码包名。真正通过 U 盘或其他离线介质再次分发时，应同时携带两份归档。当前 Linux x86_64 双包体积见[实现证据](../../../docs/design/references/implementation-evidence.md#执行面已选依赖的运维与-footprint)；两个 macOS target 需要原生重建后刷新体积。

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
| Element Web | 1.12.26 | `http://127.0.0.1:6168/` |
| Vikunja | 2.5.0 | `http://127.0.0.1:3456` |
| Dagu | 2.15.1 | `http://127.0.0.1:18080` |
| tmux | 3.7c | owner-only Unix socket |

所有 listener 都绑定 loopback。Element Web 与 Tuwunel 合在一起是 Chatroom 解决方案，并不增加第五类执行依赖；它是随包的互操作与查看客户端，不是 HCTL2 Workbench。HCTL Room 的本地 Tuwunel 配置关闭 federation 与房间加密；Dagu 只在 loopback listener 上关闭认证；Vikunja 首次启动时生成随机本地 secret。

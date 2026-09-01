# HCTL2 使用说明

本文说明当前代码树里每个 `hctl2-*` 入口的实际用途。HCTL2 仍处于早期实现阶段：现在可以运行 Chatroom、Kanban、Workflow、Terminal 四类打包依赖及一个 Rust 骨架程序，但公共 `hctl2` CLI、控制面和 Workbench 尚未实现。

## 当前入口一览

| 名称 | 当前状态 | 面向谁 | 现在能做什么 |
| --- | --- | --- | --- |
| `hctl2-services` | 可用 | 安装包用户、开发者 | 启停并检查 Chatroom（Tuwunel + Cinny）、Vikunja、Dagu 和 Herdr |
| `hctl2-tool` | P1 骨架 | HCTL2 开发者 | 显示英文帮助和版本；尚不能执行 Git/SCM 操作 |
| `hctl2-protocol` | Rust 库，不是命令 | HCTL2 开发者 | 为进程间通信提供共享信封类型 |
| `hctl2` | 尚未实现 | 最终用户 | 未来的公共治理 CLI；当前不要尝试安装或调用 |
| `hctl2-control` | 尚未实现 | HCTL2 内部组件 | 未来的控制面进程 |
| `hctl2-workbench` | 尚未实现 | 最终用户 | 未来的图形客户端 |

目前只有 `hctl2-services` 是可执行真实操作的用户命令。`hctl2-tool` 暂时用于验证代码树、构建链和命令边界，不是后台服务。Herdr 是随包提供的外部运行服务，不是 HCTL2 自建命令。

## 安装当前离线包

当前代码树为 Linux x86_64、macOS arm64 和 macOS x86_64 分别定义离线包；macOS 系统要求以[交付文档的打包策略](./design/delivery.md#打包策略选型判断首次消费时产品化)为准。

运行安装包内含固定版本的 Tuwunel、Cinny、Vikunja、Dagu、Herdr、内部静态文件服务、许可证、`hctl2-services` 与 `hctl2-tool`。锁定的上游源码位于同一 Release 中单独发布的源码伴随包。

安装过程不联网，也不在用户机器上编译，不依赖 Rust、Python、Node.js、Homebrew 或 Linux 构建工具。

每个 target 同时发布两份归档：

| 文件 | 用途 | 是否需要安装 |
| --- | --- | --- |
| `hctl2-0.0.0-<target>.tar.gz` | 运行安装包 | 是 |
| `hctl2-0.0.0-<target>-sources.tar.gz` | GPL/AGPL 对应源码与其余构建审计源码 | 否 |

两份归档各有独立的 `.sha256` 文件。源码包必须和运行包保存在同一 Release 下载位置，但普通用户安装和运行 HCTL2 时不需要下载它。

解压并按默认位置安装：

```bash
tar -xzf hctl2-0.0.0-<target>.tar.gz
cd hctl2-0.0.0-<target>
./install.sh
```

默认安装前缀是 `$HOME/.local`。如果 `$HOME/.local/bin` 尚未在 `PATH` 中，可以为当前 shell 加入：

```bash
export PATH="$HOME/.local/bin:$PATH"
```

也可以安装到另一个绝对路径：

```bash
./install.sh --prefix /absolute/path/to/hctl2
```

安装器会校验整个运行归档及 payload 的 SHA-256，随后创建 `$PREFIX/bin/hctl2-services` 与 `$PREFIX/bin/hctl2-tool` 符号链接。重复安装同一个完整包是安全的；安装不会自动启动任何进程。运行包根目录的 `SOURCES.md` 会明确指出与它对应的源码伴随包名。

查看安装器的英文命令帮助：

```bash
./install.sh --help
```

## 使用 `hctl2-services`

先查看命令自身的英文帮助：

```bash
hctl2-services --help
```

### 启动

启动全部四类依赖：

```bash
hctl2-services start
```

只启动一个或多个指定组件：

```bash
hctl2-services start tuwunel
hctl2-services start vikunja dagu
hctl2-services start cinny
```

可用组件名固定为 `tuwunel`、`cinny`、`vikunja`、`dagu` 和 `herdr`。其中 Tuwunel 与 Cinny 共同构成 Chatroom，后者不是第五类执行依赖。不指定组件时依次启动 Tuwunel、Cinny、Vikunja、Dagu、Herdr，并打印 Chatroom、Kanban、Workflow 三个浏览器地址；单独启动 `cinny` 也会先确保 Tuwunel 已启动。重复执行 `start` 不会重复启动已经由 HCTL2 管理的进程。

### 查看状态

```bash
hctl2-services status
```

`status` 总是检查五个受管组件。每行会显示 `ready`、`running`、`stopped`、`unhealthy` 或非托管 socket 等状态，并在可用时显示 PID、端点或 socket。

只有五个受管组件全部就绪时，`status` 才返回退出码 `0`；任一组件未就绪都会返回非零退出码。因此它可以直接用于脚本、健康检查和 CI，但在启用了 `set -e` 的 shell 中也会使脚本立即退出。

### 运行冒烟检查

```bash
hctl2-services smoke
```

`smoke` 不只检查进程是否存在，还会检查四个 HTTP 端点、Cinny 是否只指向随包 Tuwunel 并启用 hash router、Tuwunel 的非加密房间策略，以及 Herdr 的协议回读、API snapshot 和 socket 权限。全部检查通过时返回退出码 `0`。

### 重启或停止

重启全部组件：

```bash
hctl2-services restart
```

只重启指定组件：

```bash
hctl2-services restart dagu
```

停止全部组件：

```bash
hctl2-services stop
```

只停止指定组件：

```bash
hctl2-services stop herdr
```

不指定组件时，停止顺序与启动顺序相反。停止未运行的受管组件是安全的。为了避免误伤复用 PID 的其他进程，脚本会在发送信号前核对 PID 对应的实际可执行文件；核对失败时会拒绝停止并报告错误。

### 本地端点

| 组件 | 用途 | 本地位置 |
| --- | --- | --- |
| Tuwunel | HCTL Room 的 Matrix homeserver | `http://127.0.0.1:6167` |
| Cinny | Chatroom 随包浏览器客户端 | `http://127.0.0.1:6168/` |
| Vikunja | Kanban 浏览器客户端与本地任务后端 | `http://127.0.0.1:3456/` |
| Dagu | Workflow 浏览器客户端与本地工作流引擎 | `http://127.0.0.1:18080/` |
| Herdr | Agent / Terminal 运行服务 | 仅归属者可访问的 Unix socket；Linux 位于状态目录，macOS 位于短 `/tmp/hctl2-herdr-<uid>/` 目录 |

这些网络服务只监听本机回环地址，不对局域网或公网开放。Cinny 是官方 Web 发行包的静态内容，由随包的官方 `static-web-server` 单二进制提供；它的 homeserver 固定为 `http://127.0.0.1:6167`，不能改连任意服务器。Cinny 主要用于 Matrix 互操作和人工查看，不是 HCTL2 Workbench，也没有 HCTL2 治理权限。

Dagu 还会占用内部端口 `18090`、`15055` 和 `18091`。当前 Tuwunel 配置禁用联邦互通和房间加密，以便 HCTL2 控制面将来可以按消息 ID 读取 HCTL Room 正文；Dagu 仅在本机回环地址上关闭认证；Vikunja 首次启动时生成随机本地密钥。

### 状态、日志和数据

默认状态根目录按以下优先级确定：

1. 绝对路径环境变量 `HCTL2_STATE_ROOT`；
2. `$XDG_STATE_HOME/hctl2`；
3. `$HOME/.local/state/hctl2`。

例如，为一次开发测试隔离全部状态：

```bash
export HCTL2_STATE_ROOT=/absolute/path/to/hctl2-state
hctl2-services start
```

请对同一组 `start`、`status`、`smoke`、`restart` 和 `stop` 命令使用相同的 `HCTL2_STATE_ROOT`。状态根目录主要包含：

| 路径 | 内容 |
| --- | --- |
| `config/` | 自动生成的配置与本地 secret |
| `data/` | Tuwunel、Vikunja、Dagu 与 Herdr 的持久数据 |
| `logs/` | `tuwunel.log`、`cinny.log`、`vikunja.log`、`dagu.log`、`herdr.log` |
| `pids/` | 受管进程的 PID 文件 |
| `runtime/` | Linux 的 Herdr socket 等运行时文件；macOS socket 因路径上限放在 `/tmp/hctl2-herdr-<uid>/`，以状态根哈希命名 |

状态目录与安装目录相互独立，升级或重装同一发行包不会主动删除用户数据。

### 常见故障

- 如果启动报告端口被占用，先用 `hctl2-services status` 判断是否已有本安装管理的实例；若不是，应先处理占用对应端口的其他程序。
- 如果 HTTP 服务未能就绪，查看状态根目录下 `logs/<component>.log` 的末尾信息。
- 如果停止时报告 foreign PID、stale PID 或 unmanaged socket，不要直接向该 PID 发信号；先确认当前使用的 `HCTL2_STATE_ROOT` 是否与启动时一致，再检查 PID 文件和实际进程。
- 如果只启动了部分组件，`status` 和 `smoke` 返回非零是预期行为，因为这两个命令检查的是完整依赖集合。
- 如果 Chatroom 页面可打开但无法连接，先检查 Tuwunel 与 `cinny` 两行状态；客户端配置固定指向 `http://127.0.0.1:6167`，不接受任意 homeserver URL。

## 使用 `hctl2-tool`

从源码构建：

```bash
cd src
cargo build --locked -p hctl2-tool
```

查看英文帮助或版本：

```bash
./target/debug/hctl2-tool --help
./target/debug/hctl2-tool --version
```

也可以直接运行：

```bash
cargo run --locked -p hctl2-tool -- --help
```

当前无参数调用等同于 `--help`。除此之外的参数会返回结构化错误码 `HCTL2_TOOL_UNSUPPORTED_ARGUMENT`。Git/SCM 操作、事实回读和其他机械能力尚未落地。

## 安装完整离线包

完整离线包的下载、校验、解压和安装步骤见[安装当前离线包](#安装当前离线包)。最终用户只需下载同一版本和目标平台的运行包及其 `.sha256` 文件；源码伴随包与它的校验文件在同一 Release 提供，供源码与供应链审计按需下载，不参与安装。

安装后提供 `hctl2-tool` 与 `hctl2-services` 两个命令。运行 `hctl2-services start` 会启动 Tuwunel、Cinny、Vikunja、Dagu 和 Herdr；Tuwunel 与 Cinny 共同组成 Chatroom，Herdr 由 `hctl2-services` 管理。

## 制作外部子系统包

这一节面向发布与打包开发者，不是最终用户安装步骤。日常组包消费上游官方制品和 HCTL2 托管的 macOS Tuwunel 预编译制品；版本、URL、SHA-256 和 target identity 统一由 `packaging/dependencies/lock.json` 锁定。进入 `src/`，显式选择平台并运行 Buck：

```bash
./buck2 build root//packaging/dependencies:package \
  --target-platforms root//build/platforms:macos_arm64 \
  --out /absolute/path/dependency-packages
```

完整验证两份归档的内容与校验清单，以及运行包的离线安装、幂等重装、启动、冒烟检查和停止：

```bash
./buck2 test root//packaging/dependencies:package-test \
  --target-platforms root//build/platforms:macos_arm64
```

源码构建只用于更新 HCTL2 托管的 macOS Tuwunel 预编译制品，不进入日常组包依赖。更新时由 `Tuwunel macOS assets` workflow 在对应架构的原生 macOS runner 上构建并测试 arm64 与 x86_64 制品，再把发布地址和摘要写回 lock；普通安装、组包和完整包验证继续消费锁定的预编译制品。

外部运行包、源码伴随包及各自的 `.sha256` 位于导出的 Buck 目录，不会提交到 Git。

在源码仓库中，更详细的供应链、版本锁定与平台范围记录在 `src/packaging/dependencies/README.md`；Buck2 第一方导出、确定性组装和完整包验收记录在 `src/packaging/release/README.md`。

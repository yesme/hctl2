# HCTL2 使用说明

本文说明当前代码树里每个 `hctl2-*` 入口的实际用途。HCTL2 仍处于早期实现阶段：现在可以运行 Chatroom、Kanban、Workflow、Terminal 四类打包依赖及两个 Rust 骨架程序，但公共 `hctl2` CLI、控制面和 Workbench 尚未实现。

## 当前入口一览

| 名称 | 当前状态 | 面向谁 | 现在能做什么 |
| --- | --- | --- | --- |
| `hctl2-services` | 可用 | 安装包用户、开发者 | 启停并检查 Chatroom（Tuwunel + Element Web）、Vikunja、Dagu 和 tmux |
| `hctl2-agentd` | P1 骨架 | HCTL2 开发者 | 显示英文帮助和版本；尚不能创建 Harness 会话或持有 PTY |
| `hctl2-tool` | P1 骨架 | HCTL2 开发者 | 显示英文帮助和版本；尚不能执行 Git/SCM 操作 |
| `hctl2-protocol` | Rust 库，不是命令 | HCTL2 开发者 | 为进程间通信提供共享 envelope 类型 |
| `hctl2` | 尚未实现 | 最终用户 | 未来的公共治理 CLI；当前不要尝试安装或调用 |
| `hctl2-control` | 尚未实现 | HCTL2 内部组件 | 未来的控制面进程 |
| `hctl2-workbench` | 尚未实现 | 最终用户 | 未来的图形客户端 |

目前只有 `hctl2-services` 是可执行真实操作的用户命令。`hctl2-agentd` 和 `hctl2-tool` 暂时用于验证代码树、构建链和命令边界，不应作为后台服务部署。

## 安装当前离线包

当前代码树为 Linux x86_64、macOS arm64 和 macOS x86_64 分别定义离线包。含 Element Web 的 Linux x86_64 包已通过完整生命周期验证；macOS arm64 此前验证的是未含 Element Web 的基线，两个 Mac target 都需要在对应机器上重跑当前包。运行安装包内含固定版本的 Tuwunel、Element Web、Vikunja、Dagu、tmux、内部静态文件服务、许可证及 `hctl2-services`；锁定的上游源码位于同一 Release 中单独发布的源码伴随包。安装过程不联网，也不在用户机器上编译，不依赖 Rust、Python、Node.js、Homebrew 或 Linux 构建工具；当前包尚未纳入 `hctl2-agentd` 和 `hctl2-tool`。

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

安装器会校验整个运行归档及 payload 的 SHA-256，随后创建 `$PREFIX/bin/hctl2-services` 符号链接。重复安装同一个完整包是安全的；安装不会自动启动任何进程。运行包根目录的 `SOURCES.md` 会明确指出与它对应的源码伴随包名。

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
hctl2-services start element-web
```

可用组件名固定为 `tuwunel`、`element-web`、`vikunja`、`dagu` 和 `tmux`。其中 Tuwunel 与 Element Web 共同构成 Chatroom，后者不是第五类执行依赖。不指定组件时依次启动 Tuwunel、Element Web、Vikunja、Dagu、tmux，并打印 Chatroom、Kanban、Workflow 三个浏览器地址；单独启动 `element-web` 也会先确保 Tuwunel 已启动。重复执行 `start` 不会重复启动已经由 HCTL2 管理的进程。

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

`smoke` 不只检查进程是否存在，还会检查四个 HTTP 端点、Element Web 是否只指向随包 Tuwunel、Tuwunel 的非加密房间策略，以及 tmux 的无界面查询、稳定 ID 格式和 socket 权限。全部检查通过时返回退出码 `0`。

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
hctl2-services stop tmux
```

不指定组件时，停止顺序与启动顺序相反。停止未运行的受管组件是安全的。为了避免误伤复用 PID 的其他进程，脚本会在发送信号前核对 PID 对应的实际可执行文件；核对失败时会拒绝停止并报告错误。

### 本地端点

| 组件 | 用途 | 本地位置 |
| --- | --- | --- |
| Tuwunel | HCTL Room 的 Matrix homeserver | `http://127.0.0.1:6167` |
| Element Web | Chatroom 随包浏览器客户端 | `http://127.0.0.1:6168/` |
| Vikunja | Kanban 浏览器客户端与本地任务后端 | `http://127.0.0.1:3456/` |
| Dagu | Workflow 浏览器客户端与本地工作流引擎 | `http://127.0.0.1:18080/` |
| tmux | 无界面终端会话承载 | Linux 位于状态目录；macOS 位于 owner-only 的短 `/tmp` 目录 |

这些网络服务只监听 loopback，不对局域网或公网开放。Element Web 是官方发行包的静态内容，由随包的内部 `hctl2-web-server` 提供；它主要用于 Matrix 互操作和人工查看，不是 HCTL2 Workbench，也没有 HCTL2 治理权限。Dagu 还会占用内部端口 `18090`、`15055` 和 `18091`。当前 Tuwunel 配置禁用 federation 和房间加密，以便 HCTL2 控制面将来可以按消息 ID 读取 HCTL Room 正文；Dagu 仅在 loopback 上关闭认证；Vikunja 首次启动时生成随机本地 secret。

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
| `data/` | 三个服务的持久数据 |
| `logs/` | `tuwunel.log`、`element-web.log`、`vikunja.log`、`dagu.log` |
| `pids/` | 受管进程的 PID 文件 |
| `runtime/` | Linux 的 tmux socket 等运行时文件；macOS socket 因路径上限放在 `/tmp/hctl2-tmux-<uid>/`，以状态根哈希命名 |

状态目录与安装目录相互独立，升级或重装同一发行包不会主动删除用户数据。

### 常见故障

- 如果启动报告端口被占用，先用 `hctl2-services status` 判断是否已有本安装管理的实例；若不是，应先处理占用对应端口的其他程序。
- 如果 HTTP 服务未能就绪，查看状态根目录下 `logs/<component>.log` 的末尾信息。
- 如果停止时报告 foreign PID、stale PID 或 unmanaged socket，不要直接向该 PID 发信号；先确认当前使用的 `HCTL2_STATE_ROOT` 是否与启动时一致，再检查 PID 文件和实际进程。
- 如果只启动了部分组件，`status` 和 `smoke` 返回非零是预期行为，因为这两个命令检查的是完整依赖集合。
- 如果 Chatroom 页面可打开但无法连接，先检查 Tuwunel 与 `element-web` 两行状态；客户端配置固定指向 `http://127.0.0.1:6167`，不接受任意 homeserver URL。

## 使用 `hctl2-agentd`

从源码构建：

```bash
cd src
cargo build --locked -p hctl2-agentd
```

查看英文帮助或版本：

```bash
./target/debug/hctl2-agentd --help
./target/debug/hctl2-agentd --version
```

也可以不保留构建产物而直接运行：

```bash
cargo run --locked -p hctl2-agentd -- --help
```

当前无参数调用等同于 `--help`。除此之外的参数会返回结构化错误码 `HCTL2_AGENTD_UNSUPPORTED_ARGUMENT`。Harness 发现、进程持有、PTY、流与主机观测尚未落地。

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

## 从源码制作离线包

这一节面向发布与打包开发者，不是最终用户安装步骤。在支持的原生构建宿主上运行：

```bash
src/packaging/dependencies/build-package.sh
```

兼容入口按当前 `uname` 自动分派。发布流水线也可以显式调用三个互不混用的 target 入口：

```bash
src/packaging/dependencies/build-package-linux-x86_64.sh
src/packaging/dependencies/build-package-macos-aarch64.sh
src/packaging/dependencies/build-package-macos-x86_64.sh
```

macOS arm64 与 Intel 必须分别在对应架构的 Mac 上原生构建和测试；不能只在 Apple Silicon 上交叉编译 Intel 包，因为 Tuwunel、tmux、链接 dylib 和运行验证都属于目标合同。

完整验证两份归档的内容与校验清单，以及运行包的离线安装、幂等重装、启动、冒烟检查和停止：

```bash
src/packaging/dependencies/test-package.sh
```

运行安装包、源码伴随包及各自的 `.sha256` 位于 `src/dist/`，不会提交到 Git。依赖下载和编译缓存默认为 `${XDG_CACHE_HOME:-$HOME/.cache}/hctl2/dependencies`；需要隔离或复用缓存时，设置绝对路径 `HCTL2_BUILD_CACHE`。

在源码仓库中，更详细的供应链、版本锁定与平台范围记录在 `src/packaging/dependencies/README.md`。

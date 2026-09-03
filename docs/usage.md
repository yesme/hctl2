# HCTL2 使用说明

本文说明当前代码树里每个 `hctl2-*` 入口的实际用途。HCTL2 仍处于早期实现阶段：现在可以运行 Chatroom、Kanban、Workflow、Terminal 四类打包依赖，并用 `hctl2-tool wait` 回读闭集外部事实；公共 `hctl2` CLI、控制面和 Workbench 尚未实现。

## 当前入口一览

| 名称 | 当前状态 | 面向谁 | 现在能做什么 |
| --- | --- | --- | --- |
| `hctl2-services` | 可用 | 安装包用户、开发者 | 通过 Process Compose 启停并检查 Chatroom（Tuwunel + Cinny）、Vikunja、Dagu 和 Herdr |
| `hctl2-tool` | P1 可用子集 | HCTL2 开发者、Harness | `wait` 回读 CI、PR、引用、文件摘要与进程退出事实 |
| `hctl2` | 尚未实现 | 最终用户 | 未来的公共治理 CLI；当前不要尝试安装或调用 |
| `hctl2-control` | 尚未实现 | HCTL2 内部组件 | 未来的控制面进程 |
| `hctl2-workbench` | 尚未实现 | 最终用户 | 未来的图形客户端 |

`hctl2-tool` 不是后台服务，也不是治理命令入口；它只读外部事实并输出结构化结果。Herdr 是随包提供的外部运行服务，不是 HCTL2 自建命令。

## 安装当前离线包

当前代码树为 Linux x86_64、macOS arm64 和 macOS x86_64 分别定义离线包；macOS 系统要求以[交付文档的打包策略](./design/delivery.md#打包策略选型判断首次消费时产品化)为准。

运行安装包内含固定版本的 Tuwunel、Cinny、Vikunja、Dagu、Herdr、Static Web Server、Process Compose、供 `hctl2-tool` 使用的 GitHub CLI、许可证、`hctl2-services` 与 `hctl2-tool`。锁定的上游源码位于同一 Release 中单独发布的源码伴随包。

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

可用组件名固定为 `tuwunel`、`cinny`、`vikunja`、`dagu` 和 `herdr`。其中 Tuwunel 与 Cinny 共同构成 Chatroom，后者不是第五类执行依赖。不指定组件时由 Process Compose 启动全部五个组件；单独启动 `cinny` 也会按声明依赖启动 Tuwunel。命令在所选组件的声明式就绪探针通过后返回；重复执行 `start` 不会产生第二组进程。

### 查看状态

```bash
hctl2-services status
```

`status` 显示 Process Compose 对五个受管组件的进程状态、就绪状态、PID、运行时长、重启次数和退出码。

只有五个受管组件全部就绪时，`status` 才返回退出码 `0`；任一组件未就绪都会返回非零退出码。因此它可以直接用于脚本、健康检查和 CI，但在启用了 `set -e` 的 shell 中也会使脚本立即退出。

### 运行冒烟检查

```bash
hctl2-services smoke
```

`smoke` 要求五个 Process Compose 就绪探针已经通过，再检查 Cinny 是否只指向随包 Tuwunel 并启用 hash router、Tuwunel 的非加密房间策略，以及 Herdr 的协议回读、API snapshot 和 socket 权限。全部检查通过时返回退出码 `0`。

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

不指定组件时，Process Compose 按声明的依赖反序停止并退出；停止未运行的受管组件是安全的。进程身份、重启、就绪与关停全部由 Process Compose 管理，HCTL2 不再维护 PID 文件或自行发送信号。

### 本地端点

| 组件 | 用途 | 本地位置 |
| --- | --- | --- |
| Tuwunel | HCTL Room 的 Matrix homeserver | `http://127.0.0.1:6167` |
| Cinny | Chatroom 随包浏览器客户端 | `http://127.0.0.1:6168/` |
| Vikunja | Kanban 浏览器客户端与本地任务后端 | `http://127.0.0.1:3456/` |
| Dagu | Workflow 浏览器客户端与本地工作流引擎 | `http://127.0.0.1:18080/` |
| Herdr | 本地 Agency 参考实现的运行服务（Participant / Terminal） | 仅归属者可访问的 Unix socket；Linux 位于状态目录，macOS 位于短 `/tmp/hctl2-herdr-<uid>/` 目录 |

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
| `logs/` | 各服务日志与 `process-compose.log` |
| `runtime/` | Linux 的 Herdr socket 等运行时文件；macOS Herdr 与 Process Compose socket 因路径上限放在 owner-only 的短 `/tmp` 目录，以状态根哈希命名 |

状态目录与安装目录相互独立，升级或重装同一发行包不会主动删除用户数据。

### 常见故障

- 如果组件未能就绪，先看 `hctl2-services status`，再查看状态根目录下 `logs/<component>.log` 与 `logs/process-compose.log`。
- 如果另一组进程占用了固定端口，Process Compose 会把对应组件标成失败；停止或重新配置冲突的实例后再重启。
- 如果命令找不到 Process Compose 实例，先确认当前使用的 `HCTL2_STATE_ROOT` 是否与启动时一致；不同状态根使用不同的控制 socket。
- 如果只启动了部分组件，`status` 和 `smoke` 返回非零是预期行为，因为这两个命令检查的是完整依赖集合。
- 如果 Chatroom 页面可打开但无法连接，先检查 Tuwunel 与 `cinny` 两行状态；客户端配置固定指向 `http://127.0.0.1:6167`，不接受任意 homeserver URL。

## 使用 `hctl2-tool`

从源码构建：

```bash
cd src
cargo build --locked -p hctl2-tool
```

查看英文帮助、版本和事实词表：

```bash
./target/debug/hctl2-tool --help
./target/debug/hctl2-tool --version
```

也可以直接运行：

```bash
cargo run --locked -p hctl2-tool -- --help
```

当前无参数调用等同于 `--help`。`wait` 接受绝对 Unix 秒截止时间与一个事实；一次调用只在标准输出写一条 JSON 事实记录：

```bash
hctl2-tool wait --deadline 1788451200 commit-ci \
  --repo yesme/hctl2 --commit <commit-sha>
hctl2-tool wait --deadline 1788451200 pr-merged \
  --repo yesme/hctl2 --number 82
hctl2-tool wait --deadline 1788451200 ref-advanced \
  --repo yesme/hctl2 --ref heads/main --from <old-sha>
hctl2-tool wait --deadline 1788451200 path-digest \
  --path /absolute/path/to/artifact --sha256 <lowercase-sha256>
hctl2-tool wait --deadline 1788451200 process-exited --pid 12345
```

记录的 `outcome` 固定为 `established`（成立）、`not_established`（已确定不成立）、`unreadable`（读不到）或 `timeout`（截止前没有结论），`evidence_level` 固定为 `toolbox_readback`。对应退出码依次为 `0`、`3`、`4`、`5`；参数或启动错误返回 `1` 并在标准错误输出稳定错误码。GitHub 三类事实调用随包固定版本的 `gh` 并复用用户已有登录；它不会发起交互登录，使用前可由用户运行 `gh auth login` 建立凭据，或按 GitHub CLI 支持的环境变量提供令牌。

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

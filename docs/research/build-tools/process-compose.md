# Process Compose 本机构建服务编排审计

> 状态：采用为开发依赖 · 2026-09-02
> 固定制品：[`v1.122.0`](https://github.com/F1bonacc1/process-compose/releases/tag/v1.122.0)，官方二进制自报 [`23b0aca`](https://github.com/F1bonacc1/process-compose/commit/23b0acacc937d745279fb1551337f4031c4fc865) · Apache-2.0

## 定位

Process Compose 只管理开发机 loopback `bazel-remote` 的进程生命周期；它不进入 Buck action graph、不进入用户安装包，也不实现 REAPI。Buck2 继续通过标准 REAPI endpoint 读取和写入 action cache，`bazel-remote` 继续拥有 CAS、LRU 和 `/status`；Process Compose 只替代仓库原有的 PID 文件、启动锁、后台化、健康等待、停止和状态 shell 实现。

## 上游能力

[Process Compose](https://github.com/F1bonacc1/process-compose) 面向非容器本地进程提供声明式 YAML、detached mode、HTTP/exec readiness probe、restart policy、日志以及 `process list`、`down` 等客户端命令。`v1.122.0` Release 提供三种 HCTL2 开发平台的官方压缩包和 checksums；它是无需额外运行时的 Go 单二进制。

| 平台 | 官方制品 | SHA-256 |
| --- | --- | --- |
| Linux x86_64 | `process-compose_linux_amd64.tar.gz` | `9b6dbc38324c0b0481f1cd1dd828ffdc78117129ec797678f4bf8c4023311281` |
| macOS x86_64 | `process-compose_darwin_amd64.tar.gz` | `b4f76e881759e13f7913a5b2fed16a0e75275c04d3a61e656d40448921f1c3ec` |
| macOS arm64 | `process-compose_darwin_arm64.tar.gz` | `eaa1238a1d6c300e928ef855d36a3d95aab8da64997104d74ea73d91d3cb6c60` |

## 边界与取舍

systemd user service 能覆盖 Linux，launchd 能覆盖 macOS，但会产生两套安装位置和模板，且仓库路径、cache 上限等调用期变量需要额外安装器。Process Compose 用一份配置覆盖三平台，并能由现有 DotSlash 机制固定官方制品；代价是开发工具下载增加约 15–16 MB。HCTL2 只保留启动前的一次客户端探测：已有编排器则复用；若 loopback `/status` 已有外部或旧版管理的 `bazel-remote`，也直接复用；两者都不存在才执行官方 detached mode。健康状态、重启、PID 和关闭均不再在 shell 中实现。

## 决定

- 采用官方二进制和一份声明式 `process-compose.yaml`；只监听 loopback 的独立控制端口。
- 删除 `build/tools/buck2-cache` 及其一次性 benchmark 子命令；历史基准保留在 memo，不把测量框架长期留在生产入口。
- `src/buck2` 仍负责选择 `off/local/remote` 模式和注入 macOS 工具链身份；local 模式只确保标准 cache endpoint 已经可用，不争夺用户自行管理的同一服务。
- 操作者需要显式查看或停止时，直接使用 Process Compose 原生 `process list`、`process logs` 和 `down`，仓库不再包一层同义 CLI。

## 追加：发行侧用法（2026-09-03）

本节回答：如果用同一个 Process Compose 二进制接管用户包里五个服务（tuwunel、cinny、vikunja、dagu、herdr）的生命周期，上游给了什么、还缺什么。正文结论不变；这里只补发行侧事实，是否改借用由所有者拍板（见[部件矩阵表 D 末行](../component-matrix-20260902.md#d--约束层里靠第一方代码实现的通用机制)）。

**官方发行物覆盖**（`gh release view v1.122.0 -R F1bonacc1/process-compose`，2026-09-03 核对）：v1.122.0 发布于 2026-08-17，带 checksums，共八个平台制品——`darwin_amd64`（16.0 MB）、`darwin_arm64`（15.1 MB）、`linux_386`（15.1 MB）、`linux_amd64`（15.9 MB）、`linux_arm`（15.2 MB）、`linux_arm64`（14.6 MB）、`windows_amd64`（16.0 MB，zip）、`windows_arm64`（14.6 MB，zip）。HCTL2 当前三平台与将来的 Windows x86_64 都有官方制品，不需要自建。Go 单二进制，无运行时依赖。

**接管五个服务需要的能力，上游都有**：

| 需要 | 上游对应 | 备注 |
| --- | --- | --- |
| 后台常驻 | `process-compose up -D`（`--detached`），`-t=false` 关 TUI；`process-compose attach` 远程接回 TUI | `--detached-with-tui` 是另一种形态 |
| 就绪探针 | `readiness_probe.http_get`（host/scheme/path/port/headers/status_code）或 `readiness_probe.exec`；`liveness_probe` 同两种模式；`initial_delay_seconds / period_seconds / timeout_seconds / failure_threshold` | 文档写明 `success_threshold` 目前是占位、不生效 |
| 启动顺序 | `depends_on.<name>.condition`：`process_started / process_completed / process_completed_successfully / process_healthy / process_log_ready` | `process_healthy` 配就绪探针即得「先起 tuwunel 再起 cinny」 |
| 重启策略 | `availability.restart`：`no`（默认）/ `on_failure` / `always` / `exit_on_failure`；`backoff_seconds`（默认 1）、`max_restarts`（默认 0 = 不限）；`exit_on_end`、`exit_on_skipped`；`success_exit_codes` 允许 130/143 这类信号退出码算成功 | 后台守护型进程要 `is_daemon: true` 配 `shutdown.command` |
| 有序停机 | `--ordered-shutdown` 按依赖反序停；`shutdown.signal / timeout_seconds` | — |
| control 驱动 | REST：`GET /processes`、`GET /process/:name`、`POST /process/start/:name`、`PATCH /process/stop/:name`、`POST /process/restart/:name`、`GET /project/state`、`POST /project/stop`、`POST /project`（热更新配置）；`GET /swagger/index.html` 给 OpenAPI；WebSocket `GET /process/states/ws` 推送状态变化（客户端 `process-compose process monitor -o json`） | 默认 `localhost:8080`，`PC_API_TOKEN`（≥20 字符）或 `--token-file` 开鉴权，头为 `X-PC-Token-Key` |
| Unix 套接字 | `-U`（自动 `<TempDir>/process-compose-<pid>.sock`）或 `--unix-socket PATH` / `PC_SOCKET_PATH` | 文档限定「*nix 系统」；Windows 只能走 TCP loopback |
| 配置热更新 | `process-compose project update -f …` 或 `POST /project`：只重启有改动的进程，新增的起、删掉的停 | 保留启动时的进程选择 |
| 文件变化触发重启 | v1.122.0 新增 `watch` 块（include/exclude glob、`debounce`、`cascade`） | 与 HCTL2 无关，`--no-watch` 可关 |

**按服务分文件的配置组织**：文档里没有 `include` 指令。可用的是三种机制：

1. 多个 `-f`：`process-compose -f base.yaml -f a.yaml -f b.yaml` 按命令行顺序合并；同名进程的单值字段（`command`、`working_dir`、`disabled`）后者覆盖前者，`environment`、`depends_on` 按键合并且后者优先；片段文件不必是完整配置；**所有相对路径都相对第一个 `-f` 文件**。`PC_CONFIG_FILES` 环境变量等价。
2. 约定的 override：默认读 `process-compose.yml` 加可选的 `process-compose.override.yml`。
3. `extends`：文件级 `extends: "base.yaml"`（路径相对于扩展方文件，链式继承，禁止循环；用了 `extends` 的文件不能再与被扩展文件同时 `-f`）；进程级 `extends: base_process` 继承另一进程的配置（标量覆盖、列表追加、映射按键合并，在所有 `-f` 合并完成后解析）。

对 HCTL2 发行包，最贴合的形状是「一个基座文件定义公共环境与 `depends_on` 骨架 + 每个服务一个片段文件，由 control 以固定顺序传 `-f`」；`.env` 只从当前目录读，路径都以基座文件为准。布尔字段 `disabled` 在 override 里不能可靠地由 `true` 改回 `false`，上游为此加了字符串型 `is_disabled: "false"`，分文件时要用后者。

**未由上游提供、仍属 control 的部分**：服务的备份与恢复编排、升级顺序、Herdr 适配代码、以及「哪个服务算健康」的产品判定——Process Compose 只回答进程活着、探针通过、退出码几何。

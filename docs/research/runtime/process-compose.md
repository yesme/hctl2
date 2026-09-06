# Process Compose：托管服务的调用入口

> 对象：Process Compose `1.122.0 / 23b0acacc937d745279fb1551337f4031c4fc865`（本库 lock；2026-09-06 复核）<br>
> 许可证：Apache-2.0；Go 二进制，Rust MSRV 不适用<br>
> 定位：P2.1 control 管理随包服务的现成进程管理器，研究证据 E-RUNTIME-PROCESS-COMPOSE；不管理 HCTL 的业务状态。

## 上游能力

CLI 与 REST 是同一实例的两种入口，REST 是通过 HTTP 操作资源的接口。默认监听 `localhost:8080`；`--use-uds --unix-socket PATH` 改走指定的本机 Unix 套接字。`--token-file` 可配置 API token；本库已用独立目录与 UDS，不需要为了接 control 开 TCP。

| 需要的能力 | CLI | REST（1.122.0 `src/api/routes.go`） | 返回与边界 |
| --- | --- | --- | --- |
| 列表、单进程状态 | `process list/get NAME --output json` | `GET /processes`、`GET /process/:name` | CLI get 也是数组；REST 列表有 `data` 信封，不能按同一 JSON 形状硬读 |
| 组件启动 | `process start NAME` | `POST /process/start/:name` | CLI 文本 / 退出码；回读状态才知道是否就绪 |
| 组件停止 | `process stop NAME` | `PATCH /process/stop/:name` | 同上；停止进程不是停止 control |
| 组件重启 | `process restart NAME` | `POST /process/restart/:name` | 重试可能再次重启，确认丢失先回读 |
| 健康、就绪 | 同列表 / get | 同状态端点；`GET /process/info/:name` 读配置 | `is_running`、`is_ready`、`has_ready_probe`、`exit_code`、PID、起止时间；`GET /live` 只证明管理器活着 |
| 连续状态 | `process monitor --output json` | WebSocket `/process/states/ws` | 连接先给快照，再给状态变化；无持久恢复游标，断线重新取快照 |
| 日志 | `process logs NAME --tail N --follow` | `GET /process/logs/:name/:endOffset/:limit`；WebSocket `/process/logs/ws` | CLI 是日志文本，不是统一 JSON 事件；日志偏移不是 HCTL 业务事件游标 |

健康探针由 Process Compose 执行，control 读结果，不再自己启动探针循环。无探针时 `is_ready` 可以是 `-`；本库已给选定服务配探针并要求 `is_running=true` 且 `is_ready="Ready"`，所以不能把任意“Running”当 ready。

## 候选比较

| 候选 | 钉定版本 / 许可证 / MSRV | 代价 | 决定 |
| --- | --- | --- | --- |
| 已随包 Process Compose CLI | 1.122.0 / Apache-2.0 / 不适用 | 状态已有 JSON，动作后再回读；单次调用多一个短进程 | 采用二进制，优先路径 |
| 同一实例的 REST / WebSocket | 同上 | 少 spawn；需 HTTP over UDS 客户端、响应解析与重连 | CLI 覆盖不了的返回细节才直接调用，无需换管理器 |
| 自写 Rust 进程管理器 | 不适用 | 重做依赖启动、探针、日志、信号、关停 | 不采用 |

## 边界与取舍

control 先连接本库指定 UDS 的已有实例，核对项目与服务清单；不存在时用随包二进制 `up --detached --tui=false --keep-project` 拉起，再连接和等待所需组件。选择的是“按需启动、重启后重连”，不是让服务依附 control 子进程句柄而随 control 崩溃退出。`attach` 是给人用的 TUI，不是 control 需要的 API 附着步骤。

可直接复用 [hctl2-services](../../../src/packaging/dependencies/hctl2-services) 的配置物化、组件白名单、Cinny 对 Tuwunel 的依赖、按组件启停顺序，以及 [runtime.sh](../../../src/packaging/dependencies/common/runtime.sh) 的 UDS / 配置 / 日志路径和 readiness 判定。面向人的 `status` 输出是 `wide` 表格且会检查全部组件，control 按当前所需组件读 JSON，不解析这张表。已有 shell 里的正则 JSON 检查也不移植成 Rust 正则，直接反序列化。

先检查已有实例再启动仍可能与人手工启动相撞，启动报错后重新读同一路径核验；不连接任意端口上碰巧存在的进程。业务重试、账本、命令幂等仍在 control；Process Compose 的成功响应不能替代供应端内容回读。日志保留现有文件布局与上游轮转配置，服务日志和管理器日志分开，读取时不把凭据配置当日志展示。

本机 macOS arm64 隔离验证：用锁定二进制、独立 UDS、`sleep` 测试进程和成功 readiness 探针，实际执行了 detached up、CLI JSON get、REST 列表、REST stop/start、CLI restart、down；返回 `is_running=true`、`is_ready="Ready"`，动作均成功。只启动测试进程，没有操作开发者的现有服务。Linux / macOS x86_64 此次未重跑；三平台资产仍由现有 lock 提供。

升级沿用 [dependencies/lock.json](../../../src/packaging/dependencies/lock.json) 的版本 / commit / 各平台摘要与既有打包、smoke 测试；[开发工具 pin](../../../src/build/tools/process-compose-bin) 也用 1.122.0，不能只更新一处后混用。未来升级复核状态 JSON、探针和按组件启停；本批不更新二进制。

## 决定建议

**采用二进制，维持 Process Compose `1.122.0 / 23b0acac`。** control 优先用现成 CLI 的 JSON 状态与组件动作，沿用独立 UDS 和现有服务配置；实例不存在才 detached 启动，control 重启后重连。理由是启停、探针和日志已经具备且本机验证成立，不需要再造进程管理器；动作完成后回读，实时观察可用 `process monitor --output json`。

## 证据

- 钉定源码：[REST 路由](https://github.com/F1bonacc1/process-compose/blob/23b0acacc937d745279fb1551337f4031c4fc865/src/api/routes.go)、[进程状态与就绪判定](https://github.com/F1bonacc1/process-compose/blob/23b0acacc937d745279fb1551337f4031c4fc865/src/types/process.go)、[JSON monitor](https://github.com/F1bonacc1/process-compose/blob/23b0acacc937d745279fb1551337f4031c4fc865/src/cmd/process_monitor.go)、[日志 CLI](https://github.com/F1bonacc1/process-compose/blob/23b0acacc937d745279fb1551337f4031c4fc865/src/cmd/logs.go)。
- [v1.122.0 官方发布](https://github.com/F1bonacc1/process-compose/releases/tag/v1.122.0)、[上游许可证](https://github.com/F1bonacc1/process-compose/blob/23b0acacc937d745279fb1551337f4031c4fc865/LICENSE)、[现有服务管理说明](../../../docs/usage.md)。本次证据同时来自源码和实际二进制，不把 latest 文档当钉定版行为。

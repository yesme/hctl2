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

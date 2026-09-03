# Dagu 客户端层：从 OpenAPI v1 生成

> 状态：调研 · 日期：2026-09-03<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-SDK-DAGU<br>
> 对象：[Dagu `v2.15.1 / 532c5129`](https://github.com/dagucloud/dagu/tree/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a)（2026-08-22；上游最新 [`v2.16.2`](https://github.com/dagucloud/dagu/releases/tag/v2.16.2)，2026-09-02）· 仓库已从 `dagu-org/dagu` 迁到 `dagucloud/dagu`（旧地址 301 跳转）· 生成器候选 [progenitor `0.14.0`](https://crates.io/crates/progenitor)<br>
> 许可证：Dagu GPL-3.0-or-later（独立进程；`api/v1/api.yaml` 的 `info.license` 也标 GPL-3.0）；progenitor MPL-2.0

## 定位

Workflow 场景的机械引擎。HCTL 把 Workflow Revision 编译成受限的 Dagu YAML，再通过 REST 驱动。需要的调用面对照 v2.15.1 的实际端点：

| 调用面 | 实际端点 | 说明 |
| --- | --- | --- |
| DAG 提交 | `POST /dags`（建 DAG 文件）、`PUT /dags/{fileName}/spec`、`POST /dags/validate`；或 `POST /dag-runs/enqueue` 直接带内联 YAML `spec` 入队 | 内联入队不需要预先存在 DAG 文件 |
| 启动 | `POST /dags/{fileName}/start`、`/start-sync`、`/enqueue` | |
| 暂停 / 恢复 | **没有 run 级 pause / resume。** 只有 DAG 级 `POST /dags/{fileName}/suspend`（开关调度器是否按 cron 建 run）；run 级动作是 `stop`（终止或取消）、`retry`、`reschedule`、`dequeue` | 设计文档里的"暂停"要落到 `human.task` 等待态，而不是引擎的 pause |
| 取消与回读 | `POST /dag-runs/{name}/{dagRunId}/stop`；`GET /dag-runs/{name}/{dagRunId}`（`latest` 可作 dagRunId）；`GET …/steps/{stepName}/status` | `Status` 枚举含 `Waiting`(7)、`Rejected`(8)；`NodeStatus` 另有 `Skipped`、`Retrying` |
| `human.task` 等待节点 | `POST /dag-runs/{name}/{dagRunId}/human-tasks/{stepId}/complete`（带类型化表单输入；spec 描述："Human task completed or an identical prior completion confirmed"，即重复提交幂等）；`POST …/human-tasks/resume`（不带表单重新入队已完成的检查点） | 请求里没有调用方期望的 attempt generation，栅栏靠 HCTL 侧回读（见 [workflow-engines](../workflow-engines.md)） |
| 审批类 | `…/steps/{stepName}/approve`、`/reject`、`/push-back` | HCTL Profile 不用，但生成客户端会带出来 |

## 上游能力

**官方 SDK：没有。** 官方 API 参考页（[docs.dagu.sh/reference/api](https://docs.dagu.sh/reference/api)）没列任何语言的客户端库，只提到 MCP server。源码里看到的"客户端"都不是 SDK：`npm/` 目录是各平台二进制的 npm 包装（`dagu-linux-x64`、`dagu-win32-arm64` 等）；`ui/` 用 `pnpm gen:api` 从 OpenAPI 生成前端 TS 类型（仓库 `CLAUDE.md` 说）；Go 服务端由 oapi-codegen 生成（`api/v1/api.gen.go`，1.45 MB）；`proto/` 是 coordinator 与 worker 之间的 gRPC，不是控制 API。crates.io 没有 `dagu` crate。

**接口描述：有，OpenAPI 3.0.0，只有 v1。**

- 文件：[`api/v1/api.yaml`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/api/v1/api.yaml)，508,383 字节，首行 `openapi: "3.0.0"`，`info.version: "1.0.0"`，165 个路径；旁边的 `oapi_20241018_mod.json` 是给编辑器用的 OpenAPI 元 schema。
- **`api/v2/` 不存在**：v2.15.1 和 main（2026-09-03）的 `api/` 目录都只有 `v1`；`v2.16.2` 的 `api.yaml` 仍是 3.0.0、165 路径。任务描述里猜的 `api/v2/api.yaml` 没有这个文件；旧版 readthedocs 与部分博客提到的 `/api/v2` 与当前仓库不符。
- 运行时也给：`GET /api/v1/openapi.json`，`servers[0].url` 反映实际挂载路径（`/api/v1`、`/dagu/api/v1` 等）。
- 安全方案（spec）：`basicAuth`、`apiToken`（bearer）；官方文档补充：API key 形如 `dagu_<key>`（需开 Builtin Auth）、JWT 登录、每 DAG 的入站 webhook token `dagu_wh_<token>`、本地开发默认无鉴权。

**观测通道：只有轮询是公开契约。** spec 里没有 SSE / WebSocket 路径；源码里有 `internal/service/frontend/sse/`（`app_stream.go` 用 fsnotify 合并出低频"失效"事件让 UI 重拉），但它不在 OpenAPI 里、payload 只是"什么变了去重拉"的提示，不是可依赖的事件流。`/event-logs` 与 `/audit` 需要 manager / admin 权限。Dagu 的 `/webhooks` 与 `/dags/{fileName}/webhook` 全是**入站**触发（外部打 Dagu 起 run），出站只有通知渠道（聊天 / 事故平台），没有机器可消费的 run 状态推送。

## 候选比较

| 候选 | 版本 / 许可 | MSRV | 输入 | 判定 |
| --- | --- | --- | --- | --- |
| progenitor | 0.14.0 / MPL-2.0 | 1.88 | `api/v1/api.yaml`（3.0.0，正好在它支持的范围内） | **首选**，需一次生成实验 |
| openapi-generator `rust` | Apache-2.0，STABLE | — | 同上 | 备选（Java 运行时） |
| 手写 reqwest + serde | — | — | 读文档 | 兜底；HCTL 实际只用十几个端点 |
| 官方 / 社区 SDK | 无 | — | — | 不存在 |

165 个路径全部生成会带出 wiki、sync、license、secrets 等大量无关代码。两种处理：接受死代码（简单、与上游同步零成本）；或在 Buck action 里先按路径白名单裁剪 spec 再生成（更小，但裁剪脚本要维护）。建议先接受死代码，编译时间成问题再裁。

## 边界与取舍

- **鉴权**：本地部署给 control 发一把 `dagu_` API key，走 bearer；不要用 basic。
- **速率限制**：未查到；本地服务由我们控制。
- **事件与 webhook**：没有公开事件流，run 与 `human.task` 状态靠 `GET /dag-runs/{name}/{dagRunId}` 轮询；是否支持条件请求（ETag）未查到，按不支持设计。`human.task` 完成 API 的"相同完成幂等确认"是引擎侧保证，但 HCTL 的代次栅栏仍要自己回读（[workflow-engines 已记](../workflow-engines.md)）。
- **Windows**：客户端纯 Rust；Dagu 自己有 windows 386 / amd64 / arm64 发行包，但不在第一阶段验证矩阵。
- **许可证（需要所有者拍板）**：`api/v1/api.yaml` 与仓库同为 GPL-3.0；把它复制进 HCTL 仓库作生成输入，文件本身要保留 GPL 声明。**从接口描述生成的客户端代码是否构成衍生作品**，是个法律判断而不是工程判断——建议：生成物隔离在单独 crate（如 `hctl2-dagu-api`），来源、许可与生成命令写进该 crate 的 README；若所有者不接受这个风险，退到按官方文档手写子集（不复制 yaml）。
- **版本耦合**：`info.version` 一直是 `1.0.0`，spec 变化只能靠 Dagu 版本号追。升级 Dagu 时刷新 yaml 快照、重跑生成、跑 Run 端口 CT。

## 决定建议

- 三级判定：**第二级（从接口描述生成）**。没有 SDK；有维护中的 OpenAPI 3.0.0。
- 借用等级：**采用 SDK**——progenitor `0.14.0` 作 build 依赖，输入钉定 commit 的 `api/v1/api.yaml`；Dagu 二进制本身**采用二进制**（已定）。
- 先做一次生成实验（验收：`enqueue` 内联 spec、`start`、`stop`、`GET dag-run`、`human-tasks/{stepId}/complete`、`human-tasks/resume`、`steps/{stepName}/status` 能编译并 round-trip）。
- 两个纠偏：设计文档若有"引擎级 pause / resume"的表述，应改为 `human.task` 等待态；`api/v2` 的假设作废。
- GPL 生成物问题列为所有者待决项，不阻塞实验。

## 证据

- 仓库与发布：[dagucloud/dagu](https://github.com/dagucloud/dagu)（GPL-3.0）· [v2.15.1 树](https://github.com/dagucloud/dagu/tree/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a) · [v2.16.2（2026-09-02）](https://github.com/dagucloud/dagu/releases/tag/v2.16.2)
- 接口描述：[`api/v1/api.yaml` @ v2.15.1](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/api/v1/api.yaml) · [`api/` 目录 @ v2.15.1](https://github.com/dagucloud/dagu/tree/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/api)（只有 v1）· [`api/` @ main](https://github.com/dagucloud/dagu/tree/main/api) · [`CLAUDE.md`](https://github.com/dagucloud/dagu/blob/main/CLAUDE.md)（oapi-codegen、`pnpm gen:api`）
- 官方文档：[REST API Reference](https://docs.dagu.sh/reference/api)（鉴权方式、`/api/v1/openapi.json`、无 SDK）
- 源码：[`internal/service/frontend/sse/app_stream.go`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/service/frontend/sse/app_stream.go)（UI 失效流，非公开契约）· [`internal/service/frontend/api/v1/humantasks.go`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/service/frontend/api/v1/humantasks.go) · [`npm/`](https://github.com/dagucloud/dagu/tree/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/npm)（二进制包装）
- 生成器：[progenitor](https://crates.io/crates/progenitor) · [openapi-generator rust](https://openapi-generator.tech/docs/generators/rust/)
- 本仓库：[workflow engine 条目](../workflow-engines.md) · [Run 约束 §外部概念对齐](../../design/spec/run.md) · [部件矩阵](../component-matrix-20260902.md)

## 复核记录

- 2026-09-04 所有者裁决：从 GPL 许可的接口描述文件生成或重写的客户端代码**不算衍生作品，可以入库**。生成物按普通源码入库并随构建复现；不需要另做「构建期产物不入库」的绕法。

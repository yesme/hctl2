# Vikunja 客户端层：从 OpenAPI v2 生成

> 状态：调研 · 日期：2026-09-03<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-SDK-VIKUNJA<br>
> 对象：[Vikunja `v2.5.0 / ef2200e9`](https://github.com/go-vikunja/vikunja/tree/ef2200e9429c5cc42f5c1811433418bfcc72b3aa)（2026-08-04；上游最新 [`v2.6.0`](https://github.com/go-vikunja/vikunja/releases/tag/v2.6.0)，2026-08-31）· 生成器候选 [progenitor `0.14.0`](https://crates.io/crates/progenitor)（2026-04-24）、[openapi-generator rust](https://openapi-generator.tech/docs/generators/rust/)<br>
> 许可证：Vikunja AGPL-3.0-or-later（独立进程，不链接、不 vendor）；progenitor MPL-2.0（构建期依赖，生成物归我们）；openapi-generator Apache-2.0（Java 工具）

## 定位

Kanban 场景的本地 content 后端。`hctl2-control` 的任务后端适配器要做：

| 调用面 | v2 端点 | 说明 |
| --- | --- | --- |
| 卡片与分组读写 | `GET/POST /projects/{project}/tasks`；`GET/PUT/PATCH/DELETE /tasks/{task}`；`PUT /projects/{project}/views/{view}/buckets/{bucket}/tasks`（移桶）；`PUT /tasks/{task}/position` | 移桶幂等，移入 done 桶有"标记完成"副作用（见 [任务后端复核](../task-backends.md#2026-08-30-provider-动作复核)） |
| 稳定归属回读 | Task 字段 `id`、`identifier`、`index`、`project_id`、`bucket_id`、`position`、`created_by`、`updated`、`done`、`done_at` | HCTL 以 bot 用户写入，webhook 里的 `doer` 就能区分"我们写的"和"人写的" |
| 条件写入 | 单资源 `GET` 回 `ETag`；`If-None-Match` → 304；`If-Match` / `If-Unmodified-Since` 前置条件；`PATCH` 为 JSON Merge Patch（也接受 JSON Patch） | 官方文档说前置条件受支持；spec 里只在 `GET` 参数列出 `If-*` 头，`PUT`/`PATCH` 未列——要实测 |
| webhook / 轮询观测 | `GET/POST /projects/{project}/webhooks`；`GET /webhooks/events` 列可订阅事件；轮询用 `If-None-Match` | 出站 webhook 有 HMAC 签名、无投递 ID |
| 实体 ID | 整数 `id` 为主键；`identifier` / `index` 是项目内人类可读编号 | 见下 |

## 上游能力

**官方 SDK：没有，任何语言都没有。** 官方文档说 v2 spec "是生成客户端 SDK 的可靠基础"，但没列任何生成器或官方客户端。

**社区 Rust 客户端：未查到可复用的库。** crates.io 上没有 `vikunja` crate；GitHub 上用 Rust 写的 Vikunja 项目都是应用（cria TUI 29 星、vikunja-tui、vk CLI、vikunja-cli、vikunja-rust-mcp），没有一个是独立发布的客户端库。

**接口描述：有两套，目标是 v2。**

| | v1 | v2 |
| --- | --- | --- |
| 规格 | Swagger 2.0，由 swaggo 注解在构建期生成 | OpenAPI 3.1，由 Huma 从 Go 类型**运行时反射**（源码里看到 `pkg/routes/api/v2/huma.go`、`pkg/modules/humabridge`；官方文档没点名 Huma，只说 "reflected from the Go types at runtime"） |
| 仓库里有没有 | 有：`pkg/swagger/swagger.json`（425,628 字节）、`swagger.yaml` | **没有**：不 check-in，`pkg/cmd` 也没有导出命令 |
| 服务端路径 | `/api/v1/docs.json` | `/api/v2/openapi.json`、`.yaml`；另有 **`/api/v2/openapi-3.0.json`、`.yaml` 降级版**（官方文档："for tools that don't understand 3.1 yet"） |
| 状态 | 2.4.0 起冻结，3.0 弃用，4.0 删除 | 2.4.0 起是新集成的唯一推荐目标 |

实测 `try.vikunja.io`（跑的是 `v2.6.0-23-gd2852649` 开发版，不是钉定版本）：`/api/v2/openapi.json` 为 `openapi: 3.1.0`，136 个路径、138 个 schema；`/api/v2/openapi-3.0.json` 为 `openapi: 3.0.3`，路径数相同。安全方案三种：`APITokenAuth`（`tk_` 前缀 bearer，按 `/routes` 列出的路由粒度授权）、`JWTKeyAuth`（登录会话）、`BasicAuth`（只给 Atom feed）。

v2 还带来 HCTL 关心的几件事（官方文档）：验证失败回 422（v1 是 412）；分页有信封（`items` / `page` / `per_page` / `total`）；响应体里带 `max_permission`；JSON 响应带 `$schema` 字段指向 `/api/v2/schema/…`。Bot 用户（`/user/bots`，用户名以 `bot-` 开头，只有所有者能读写，用 `/tokens?owner_id=` 给它发 token）是 HCTL 写入身份的现成机制。

## 候选比较

| 候选 | 版本 / 许可 | MSRV | 输入 | 输出 | 判定 |
| --- | --- | --- | --- | --- | --- |
| progenitor（Oxide） | 0.14.0 / MPL-2.0 | 1.88 | **OpenAPI 3.0.x 专用**（README 明说；不吃 3.1）→ 用 Vikunja 的 `openapi-3.0.json` | reqwest 0.13 客户端；macro / build.rs / 独立 crate 三种模式；positional 或 builder 风格 | **首选**，需一次生成实验 |
| openapi-generator `rust` | 工具 Apache-2.0；生成器标 STABLE | — | OpenAPI 3.x（3.1 支持未在文档里提及） | reqwest / hyper；Java 运行时，进 Buck action 要带 JRE | 备选 |
| 手写 reqwest + serde | — | — | 人读文档 | 只写用到的十几个端点 | 兜底 |
| 社区 Rust 库 | 无 | — | — | — | 不存在 |

progenitor README 自己承认 "may fail for some OpenAPI documents"。Huma 3.1 → 3.0 降级会把 `type: ["array","null"]` 之类转成 `nullable`，还会留下每个响应体的 `$schema` 字段——这些是生成实验要盯的点。

## 边界与取舍

- **鉴权**：为 HCTL 建一个 bot 用户、给它发按路由收窄的 API token；明文 token 只在创建响应里出现一次。写入者身份稳定后，webhook 的 `doer` 与 `created_by` 才能用于"归属回读"。
- **速率限制**：自托管、本地进程，限速由我们配置；本次未核对 Vikunja 的限速配置项。
- **事件与 webhook**：出站 webhook 按项目配置，payload 为 `{event_name, time, data:{task, doer}}`，有 `X-Vikunja-Signature`（HMAC-SHA256 覆盖原始 body）；**没有投递 ID 头**，重试策略官方文档未写（两点已在 [任务后端复核](../task-backends.md#2026-08-30-provider-动作复核) 按源码确认）。所以 webhook 只当"去看一眼"的提示，事实以带 `If-None-Match` 的回读为准。
- **条件写入**：文档说 `If-Match` / `If-Unmodified-Since` 受支持，但 spec 的 `PUT` / `PATCH` 参数表没列这些头（`GET` 列了）。生成的客户端可能没有对应参数——要么实测后手动补头，要么把它记为上游 spec 缺口报给 Vikunja。
- **规格快照怎么进仓库**：v2 spec 只能从运行中的钉定二进制取。建议把钉定版本的 `openapi-3.0.json` 连同 SHA-256 一起 check-in 到 `src/` 下的适配器 crate，生成在 Buck action 内做；再加一条 CT：起钉定二进制、抓 `/api/v2/openapi-3.0.json`、与快照比对，漂移即红。升级 Vikunja 时刷新快照、重跑生成。
- **Windows**：客户端是纯 Rust HTTP，没有平台问题；Vikunja 服务端的平台矩阵不在本文件范围。
- **许可证**：AGPL 义务限于 Vikunja 进程本身；OpenAPI 文档是接口事实描述，生成的客户端代码归我们。若所有者想把这点说死，可在适配器 crate 的 README 里记一句来源。

## 决定建议

- 三级判定：**第二级（从接口描述生成）**。没有官方或社区 Rust SDK；有机器可读的 OpenAPI 3.1 与 3.0 降级版。
- 借用等级：**采用 SDK**——生成器 progenitor `0.14.0` 作 build 依赖，输入钉定版本的 `/api/v2/openapi-3.0.json` 快照；Vikunja 服务本身仍是**采用二进制**（已定）。
- 顺序：P2 开工先做一次 progenitor 生成实验（验收：`/tasks/{task}` 四个方法、移桶、position、webhooks、`/routes` 能编译并通过一轮 round-trip）；不过就换 openapi-generator；两个都不行才手写子集。
- 与 [部件矩阵](../component-matrix-20260902.md) 的差异：矩阵写的是"Swagger 2.0 先转 OpenAPI 3 再生成"，那是基于 v1；本文改为直接用 v2 自带的 3.0 降级版，省掉转换步骤，也避开 v1 的弃用时间表。

## 证据

- 官方文档：[API v2](https://vikunja.io/docs/api-v2/)（2.4.0 起、OpenAPI 3.1、`openapi-3.0.*` 降级版、ETag / If-Match、Merge Patch、v1 时间表）· [API Documentation](https://vikunja.io/docs/api-documentation/)（v1 Swagger 位置）· [Webhooks](https://vikunja.io/docs/webhooks/)（payload、`X-Vikunja-Signature`）
- 实例实测（2026-09-03，`try.vikunja.io`，开发版）：[`/api/v2/openapi.json`](https://try.vikunja.io/api/v2/openapi.json)（3.1.0，136 路径）· [`/api/v2/openapi-3.0.json`](https://try.vikunja.io/api/v2/openapi-3.0.json)（3.0.3）· [`/api/v1/docs.json`](https://try.vikunja.io/api/v1/docs.json)（swagger 2.0，126 路径）
- 源码（v2.5.0）：[`pkg/routes/api/v2/huma.go`](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/routes/api/v2/huma.go) · [`pkg/swagger/`](https://github.com/go-vikunja/vikunja/tree/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/swagger) · [`pkg/cmd/`](https://github.com/go-vikunja/vikunja/tree/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/cmd)（无 spec 导出命令）
- 发布：[v2.5.0](https://github.com/go-vikunja/vikunja/releases/tag/v2.5.0) · [v2.6.0（2026-08-31）](https://github.com/go-vikunja/vikunja/releases/tag/v2.6.0)
- 生成器：[progenitor crates.io](https://crates.io/crates/progenitor)（0.14.0，MPL-2.0，rust-version 1.88）· [progenitor README](https://github.com/oxidecomputer/progenitor/blob/main/README.md)（"OpenAPI 3.0.x"）· [openapi-generator rust](https://openapi-generator.tech/docs/generators/rust/)
- 社区 Rust 项目（均非库）：[cria](https://github.com/frigidplatypus/cria) · [vikunja-tui](https://github.com/mark-pitblado/vikunja-tui) · [vk](https://github.com/JMARyA/vk) · [vikunja-rust-mcp](https://github.com/brianluby/vikunja-rust-mcp)
- 本仓库：[任务后端条目与 2026-08-30 复核](../task-backends.md) · [部件矩阵](../component-matrix-20260902.md)

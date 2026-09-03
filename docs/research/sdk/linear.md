# Linear 客户端层：从 GraphQL schema 生成

> 状态：调研 · 日期：2026-09-03<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-SDK-LINEAR<br>
> 对象：Linear GraphQL API（`https://api.linear.app/graphql`）· schema 快照取 [linear/linear `@linear/sdk@92.0.0` / 873e009c](https://github.com/linear/linear/tree/873e009ca1c5) 的 `packages/sdk/src/schema.graphql`（该文件最近一次更新 2026-08-27，commit `cda6e4ee`）· 生成器候选 [graphql_client `0.16.0`](https://crates.io/crates/graphql_client)（2026-01-15）、[cynic `3.14.0`](https://crates.io/crates/cynic)（2026-07-12）<br>
> 许可证：linear/linear 仓库（含 schema 文件与 TS SDK）MIT；graphql_client Apache-2.0 OR MIT；cynic MPL-2.0；Linear API 的使用受 Linear 开发者条款约束（不是开源许可）

## 定位

远端任务后端之一（另一是 GitHub）。Linear 只提供外部字段的写入权威，不是 HCTL 的 Task 模型（[Task 约束 §外部概念对齐](../../design/spec/task.md)）。适配器要做：

| 调用面 | GraphQL 形状 | 说明 |
| --- | --- | --- |
| 读 Issue 与字段 | `issue(id)`、`issues(filter, orderBy: updatedAt, first/after)`；`workflowStates`、`teams`、`viewer` | 分页是 Relay 风格游标，默认 50 条 |
| 写 Issue | `issueCreate`、`issueUpdate`（状态、负责人、标签、描述） | 无条件写入机制，见边界 |
| 增量回读 | `issues(filter: {updatedAt: {gt: …}}, orderBy: updatedAt)` | 官方建议按 `updatedAt` 排序、避免翻完整集 |
| webhook 观测 | `webhookCreate(url, teamId | allPublicTeams, resourceTypes)` | 或 OAuth app 安装时自动建 |
| 实体 ID | `id`（UUID）为主键；`identifier`（如 `ENG-123`）人类可读 | 两个都回读、只以 `id` 做绑定 |

## 上游能力

**官方 SDK：只有 TypeScript / JavaScript。** [`@linear/sdk`](https://github.com/linear/linear/tree/master/packages/sdk) `92.0.0`，MIT，仓库里另有 `codegen-sdk` / `codegen-doc` / `codegen-test` 三个包——SDK 是从 GraphQL schema 用自家 GraphQL Code Generator 插件生成的。官方 SDK 文档只提 TypeScript（"written in TypeScript but can also be used in any JavaScript environment"）；Rust、Python、Go 官方 SDK **未查到**。

**社区 Rust SDK：已死。** crates.io 上 `linear_sdk` / `linear-sdk` 都是 `0.0.1`（2022-10-29，maxdeviant，MIT），仓库 1 星、最后推送 2024-08-19。不可用。

**接口描述：有，两种取法。**

- 内省：端点公开 introspection（官方文档："It supports introspection so you can query the whole schema"）；本次实测不带任何凭据 POST `{ __schema { queryType { name } } }` 即返回 `Query`。
- 快照：官方 SDK 仓库 check-in 了 [`packages/sdk/src/schema.graphql`](https://github.com/linear/linear/blob/master/packages/sdk/src/schema.graphql)（SDL，1,298,655 字节）和 [`schema.json`](https://github.com/linear/linear/blob/master/packages/sdk/src/schema.json)（内省结果，5,762,531 字节）。按 SDK 版本 tag 取即可钉住。schema 改得频繁（最近一次 2026-08-27，提交信息 "feat(sdk): update schema"）。
- Apollo Studio 上有公开图谱可浏览。

## 候选比较

| 候选 | 版本 / 许可 | MSRV | 工作方式 | 大 schema 应对 | 判定 |
| --- | --- | --- | --- | --- | --- |
| graphql_client | 0.16.0 / Apache-2.0 OR MIT | 1.66 | 查询写成 `.graphql` 文件，`#[derive(GraphQLQuery)]` 指 `schema_path` + `query_path`，编译期生成变量与响应类型；schema 接受 SDL 或内省 JSON；`graphql_client_cli introspect-schema` 抓 schema；可选 reqwest feature | 每个 derive 都读一遍 1.3 MB schema，编译期成本随查询数线性涨——HCTL 只有十几个操作，预计可接受，**需实测** | **首选** |
| cynic | 3.14.0 / MPL-2.0 | 1.85 | 查询写成 Rust 结构体 + derive，`build.rs` 里 `cynic_codegen::register_schema("linear").from_sdl_file(…).as_default()` 预处理一次；`cynic-cli introspect` 抓 schema；有在线 generator 把 GraphQL 查询转成结构体 | 文档专门有"Working with Large APIs"：预注册 schema + `rkyv` feature 降编译时间与 rust-analyzer 负担 | 备选：graphql_client 编译期不可接受时换 |
| 手写 serde_json 查询 | — | — | 字符串拼查询、手写响应结构 | 无编译期校验 | 兜底 |
| 官方 TS SDK 走 Node 边车 | @linear/sdk 92.0.0 | — | 起 Node 进程 | — | 不做：给 control 引入 Node 运行时 |

两个生成器仓库都活着：graphql-client 最后推送 2026-08-11（1,263 星）；cynic 2026 年迁到 Codeberg（crates.io 的 repository 指 `codeberg.org/obmarg/cynic`，GitHub 镜像最后推送 2026-03-07），crates.io 上 2026 年发了三个版本。

## 边界与取舍

- **鉴权**：个人 API key 或 OAuth2 access token，都放 `Authorization` 头（OAuth 形式 `Bearer <token>`；API key 的确切写法以官方 Getting started 页为准，本次未抓到该段）。官方推荐给多人用的应用走 OAuth2；HCTL 第一阶段单用户可先用 API key，OAuth actor authorization 留给多用户阶段。
- **速率限制（官方数字）**：按请求——API key 每用户 2,500 次/时，OAuth app 每用户 5,000 次/时，未认证每 IP 600 次/时；按复杂度——API key 3,000,000 点/时，OAuth app 2,000,000 点/时，未认证 100,000 点/时；**单查询上限 10,000 点**，超了直接拒。响应头 `X-RateLimit-Requests-Limit/Remaining/Reset`、`X-Complexity`、`X-RateLimit-Complexity-Limit/Remaining/Reset`，个别端点另有 `X-RateLimit-Endpoint-*`。超限时 **HTTP 400** + `extensions.code = "RATELIMITED"`（不是 429）。工作区级 OAuth app 用 actor authorization 时按付费人数动态提额。官方明确不鼓励轮询。
- **事件与 webhook**：UI 或 `webhookCreate` 创建；支持 Issue、Comment、Label、Project、Cycle、Document、Initiative、Customer 等类型。payload：`action`（create / update / remove）、`type`、`data`、`updatedFrom`（update 时旧值）、`webhookTimestamp`（毫秒）。**`Linear-Signature`** 是签名密钥对原始 body 的 HMAC-SHA256 十六进制；**`Linear-Delivery`** 是每次投递的 UUID v4（这点比 Vikunja 好，可直接做幂等键）；官方建议 `webhookTimestamp` 与本地时间差在一分钟内防重放；失败最多重试 3 次，间隔 1 分钟、1 小时、6 小时。OAuth app 可在安装时自动建 webhook（需 `admin` scope）。
- **条件写入**：Linear 没有 ETag / If-Match / 版本号一类的乐观并发机制（**未查到**）。HCTL 的"条件写入"只能在应用层做：写前读 `updatedAt`，写后回读比较，不一致按分歧对账——与 Task 约束的 Snapshot / 回读路径一致。
- **schema 漂移**：schema 月内多次更新；钉 `@linear/sdk` 的 tag 取快照并 check-in，升级时刷新快照重编；加一条 CT 用内省结果与快照比对（cynic 文档也建议"定期 CI 任务拉 schema 提交"）。
- **Windows**：纯 Rust HTTP，无问题。
- **许可证**：cynic 是 MPL-2.0，作为依赖使用没有传染问题，生成 / 派生的查询结构体是我们的代码；graphql_client 双许可更省心，这是它作首选的次要理由。

## 决定建议

- 三级判定：**第二级（从接口描述生成）**。官方只有 TS SDK；GraphQL schema 可内省也有官方快照。
- 借用等级：**采用 SDK**——graphql_client `0.16.0`（查询写 `.graphql`，便于评审），schema 用 `@linear/sdk@92.0.0` tag 下的 `schema.graphql` 快照；cynic `3.14.0` 作编译期不可接受时的替换项。Linear 服务本身是**适配协议**（外部 SaaS，无二进制可采）。
- 观测以 webhook 为主（`Linear-Delivery` 做幂等键、`Linear-Signature` 校验、一分钟时间窗），轮询只作补漏，按 `updatedAt` 增量拉。
- 开工前核对：API key 的 `Authorization` 头格式；graphql_client 对 Linear 自定义标量（`DateTime`、`JSON`、`TimelessDate` 等）的映射清单。

## 证据

- 官方文档：[GraphQL Getting started](https://linear.app/developers/graphql)（端点、内省、鉴权）· [Rate limiting](https://linear.app/developers/rate-limiting)（全部限额数字、响应头、400 + RATELIMITED）· [Webhooks](https://linear.app/developers/webhooks)（payload、`Linear-Signature`、`Linear-Delivery`、重试）· [Pagination](https://linear.app/developers/pagination) · [TypeScript SDK](https://linear.app/developers/sdk)
- 官方 SDK 仓库：[linear/linear](https://github.com/linear/linear)（MIT）· [`packages/sdk/package.json`](https://github.com/linear/linear/blob/master/packages/sdk/package.json)（92.0.0）· [`packages/sdk/src/schema.graphql`](https://github.com/linear/linear/blob/master/packages/sdk/src/schema.graphql) · [`schema.json`](https://github.com/linear/linear/blob/master/packages/sdk/src/schema.json) · tag `@linear/sdk@92.0.0` → `873e009c`
- 内省实测：2026-09-03 对 `https://api.linear.app/graphql` 无凭据 POST `__schema` 查询返回 `Query`
- 社区 Rust：[crates.io linear_sdk](https://crates.io/crates/linear_sdk)（0.0.1，2022）· [maxdeviant/linear-sdk](https://github.com/maxdeviant/linear-sdk)
- 生成器：[graphql_client crates.io](https://crates.io/crates/graphql_client)（0.16.0，MSRV 1.66）· [graphql-client README](https://github.com/graphql-rust/graphql-client/blob/main/README.md) · [cynic crates.io](https://crates.io/crates/cynic)（3.14.0，MPL-2.0，MSRV 1.85）· [cynic 文档](https://cynic-rs.dev/)（schema 预注册、Working with Large APIs、introspect）· [cynic Codeberg](https://codeberg.org/obmarg/cynic)
- 本仓库：[任务后端条目（Linear 外部来源）](../task-backends.md) · [Task 约束](../../design/spec/task.md) · [部件矩阵](../component-matrix-20260902.md)

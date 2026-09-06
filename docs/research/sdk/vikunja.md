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

## 复核记录

### 2026-09-06 · P2.2 映射、条件写入与生成实验

> 对象：Vikunja `v2.5.0 / ef2200e9429c5cc42f5c1811433418bfcc72b3aa` · progenitor `0.14.0`<br>
> 许可证：Vikunja AGPL-3.0-or-later；progenitor MPL-2.0；备选 OpenAPI Generator Apache-2.0<br>
> 定位：核验 P2.2 对钉定服务的真实调用，不以在线 demo 或 README 的承诺代替；本次有本机写入反例与生成失败记录。

#### 上游能力

| HCTL 对象 / 操作 | Vikunja v2.5.0 的落点 | 稳定回读与限制 |
| --- | --- | --- |
| Repo 的 Board（任务内容容器） | 一个顶层 Vikunja project 的 ID 作为范围锚点 | 这是容器映射，不承诺顶层的一个 Kanban 页面自动展示所有子项目卡片 |
| HCTL Project 分组 | 顶层之下的一个直接子 project，记录 ID 与 `parent_project_id` | `models.Project` 有明确的单一父 ID，比父任务关系更适合作分组锚点；回读父子关系与已准入绑定 |
| HCTL Task 卡片 | 子 project 中的 task，以 `id` 与 `project_id` 认定内容身份、分组归属 | 看板 bucket 属于某个 view 的阶段，不作 Project 分组；子任务与清单也不自动成为 HCTL Task |
| 读写与移动 | `/tasks/{projecttask}` 的 GET / PUT / PATCH / DELETE；`/tasks/{task}/position`；project view 下的 bucket 移动接口 | 以实际 OpenAPI 占位名为准。阶段、排序移动留在原分组，原生客户端改 `project_id` 后只记 Snapshot 与需要关注 |
| 富文本 | 默认 HTML；`format=markdown` 做格式转换 | 转换不是字节无损往返；只改阶段时不顺便回写转换后的描述 |

该映射沿用[Task §契约与来源](../../design/spec/task.md#契约与来源)的 Repo Board / Project 分组，不增加 HCTL 对象；父任务关系可多条，不能仅凭关系类型猜出唯一 Project。根、子 project 被移动或删除也需重核绑定，不能只盯 task 自己的字段。

**条件写入实测：推翻「只要补 If-Match 请求头就能防止覆盖」的假设。** GET 的 ETag 是内容版本标签，但服务端并未把它用于任务写入的原子比较。源码 `tasksRead` 接受 `conditional.Params`；`tasksUpdate` 只接 ID、format 与 body，直接 `DoUpdate`。在隔离的 v2.5.0 实例中，对刚创建的 task 分别执行：

| 请求 | 故意给错的条件 | 实际响应及回读 |
| --- | --- | --- |
| PUT `/api/v2/tasks/1`，修改 title | `If-Match: "impossible-stale-etag"` | HTTP 200，标题确实变成 PUT 的新值 |
| PATCH 同一路径，`application/merge-patch+json`，再次改 title | 同一错误 ETag | HTTP 200，标题确实变成 PATCH 的新值 |

所以 `updated`、写前读、写后读都不是服务端比较并交换；它们能发现部分漂移，不能消除两次读之间的并发覆盖。此处换 SDK 也不会补出服务端能力。GET 的 `If-None-Match` 仍适合轮询；写端按真实能力声明，不能声称具备条件写保护。

webhook 以子 project 为单位订阅。`models/webhooks.go` 在配置 secret 后才生成 `X-Vikunja-Signature`（原始 body 的 HMAC-SHA256）；无 secret 就没有这项真实性校验。任务事件包括 `event_name`、`time`、`data.task` 与 `data.doer`，未找到稳定投递 ID 或可回放的服务端事件游标。发送代码直接发 HTTP 请求，不能据此承诺持久重试。沿用「通知触发回读 + 定期分页核对」，补足漏投、删除和项目归属变化；不假定订阅父 project 就覆盖子 project。

用 `/user/bots` 建 HCTL bot，再经 `/tokens?owner_id=…` 给它发按 `/routes` 收窄的 API token；token 所属账号决定写入身份，不能任填 `doer`。创建时返回的明文 token 进入现有密钥存储。限流配置默认 `enabled=false`；开启后的默认值为按 user、60 秒 100 次、内存存储，未认证请求另有限额。控制端尊重部署值和 429，不把这些默认数值当接口保证。

#### 生成实验

2026-09-06，macOS arm64、rustc 1.98.0。在仓库之外用本库 `bb25282` 的构建配置建隔离实验，**通过 Buck2 的 rust_binary → genrule → rust_library** 调用 progenitor `0.14.0`；不改本库依赖，也未用 Cargo 绕过 Buck 构建。

服务是随包所钉的官方 `vikunja-v2.5.0-darwin-10.15-arm64`，仅监听 `127.0.0.1:23456`，独立 SQLite 与文件目录，创建临时用户和卡片；没有向用户实例写数据。GET `/api/v2/openapi-3.0.json` 导出的原件为 OpenAPI **3.0.3**，`info.version=v2.5.0`，**128 路径、135 schemas**，SHA-256：

```text
d3c8fc29a7c14717dc4ea60e8626b0f22af9eb7b6e7b61e12253c21faa0b9570
```

摘要包含本次 public URL 配置；同配置重复导出一致，不能要求不同实例地址的整份文档摘要相同。正文的 136 路径是 9 月 3 日在线开发版数据，不是 2.5.0。原件保存在本机实验目录 `/tmp/hctl2-p2-research.kpvgTx/experiment/src/probe/openapi.json`，本轮重算摘要仍一致；这是临时证据位置，不是可依赖的长期制品地址。

P2.2 把原件收入适配器的版本快照时，保留这个原件摘要作生成输入校验，另算排除部署地址的规格摘要供跨实例比较。本次地址只出现在 `/servers/1/url`；建议仅排除由 `service.publicurl` 产生的绝对地址条目，保留相对 `/api/v2` 条目，其余内容统一 JSON 排序后计算 SHA-256。排除规则也随快照固定，不改 paths、schemas 或媒体类型；跨部署一致性还待下一轮实验，不把排除后的文档替换成生成器输入。

| 输入 / 实验步骤 | 真实失败点 | Buck Build ID |
| --- | --- | --- |
| 原始完整文档；解析为 `openapiv3::OpenAPI`，`Generator::default().generate_tokens(&spec)` | `progenitor-impl/src/method.rs:2057`：`more media types than expected for patch-filters-read: 3`；该 PATCH 同时描述 JSON Patch、Merge Patch 与 shorthand | `b27c7473-5d31-4858-adc3-271246fd401a` |
| 只作诊断：请求体选 JSON 或 Merge Patch 的 schema，统一成一个 JSON media type，仍保留全部路径 | `UnexpectedFormat("unexpected content type: multipart/form-data")` | `ba683000-ff21-442b-92a4-13175d94859a` |
| 再缩到任务、分组、bucket、position、webhook、routes、token、bot 相关的 17 个路径，保留 components，沿用上述诊断处理 | `method.rs:1220`：`assertion failed: response_types.len() <= 1` | `b44a2838-e573-426e-a6a2-17257d94734f` |

三次均在生成阶段退出，**没有生成客户端编译通过的结果，更没有 round-trip 通过**。后两次只用于定位兼容范围，不是可采用的规格改写方案：把 Merge Patch 媒体类型改成 JSON 已改变线上请求含义，也不能为让生成器过关删除条件响应。17 路径试验不是完整调用面验收。原始文档已足以重现首个失败，后续没有继续叠兼容补丁。

#### 候选比较

| 候选 | 钉定版本 / 许可证 | MSRV / 构建依赖 | 本次判定 |
| --- | --- | --- | --- |
| progenitor | `0.14.0` / MPL-2.0 | MSRV 1.88；生成器本身在 1.98 + Buck 上已运行 | **推翻直接可用的判断**；输入兼容失败，不是 Rust 版本不足 |
| OpenAPI Generator Rust 生成器 | `7.25.0` / Apache-2.0 | Java 构建期工具，不以 Rust MSRV 描述；生成客户端的 MSRV 待实际构建核验 | 暂缓采用；按原文的后备顺序，下一次用同一快照验证它，不能写成本次已通过 |
| 手写 HTTP 子集 | 无版本 / 本库代码 | 本库 1.98 | 暂缓；本次失败不构成跳过第二个现成生成器的理由 |

#### 边界与取舍

**分组映射尚待 P2.2 庚开工前确定。** 子 project 能提供唯一归属，但父 project 不汇总子项目任务：只按前面的容器映射接入，用户看到的是每个 HCTL Project 各一块看板，尚未兑现[Task 设计 §Kanban 场景](../../design/task.md#kanban-场景)的 Repo 级总板。两条可比较的路线如下；本次不替所有者定产品取舍。

| 路线 | 原生能力与创建方式 | 产品代价与维护者 |
| --- | --- | --- |
| 子 project 分组，saved filter（保存的跨项目查询）汇总 | v2.5.0 支持 `POST /api/v2/filters`，以 `filters.filter = "project in 12, 13"` 列出已准入子 project 的 ID；创建时同时建 List / Gantt / Table / Kanban 视图，任务仍在原来的子 project | filter 只属于创建账号，不能分享；HCTL bot 创建的视图不会出现在人的账号里。若采用，由人建并维护个人 filter；control 若要代维护，需另获该人的凭据与授权，不能沿用 bot token。分组增删后还要更新 ID 列表，不会自动递归包含新子项目。汇总视图的 bucket 按 view 独立，不是子看板列的并集，也不自动提供「Project 分组 × 阶段」两级展示 |
| 单个 project 作 Repo Board，以获准标签 ID 作 Project 分组 | 一个共享原生看板容纳全部任务，标签由用户或获授权的适配器创建并记录稳定 ID，可按标签过滤；不为每个 HCTL Project 另建 project | 标签是多对多关系，可缺失或同时挂多个分组标签，也没有每组独立的 project 权限。原生界面没有自动强制唯一分组；control 按 [Task §契约与来源](../../design/spec/task.md#契约与来源)回读恰好一个获准标签后才认领，歧义与改组只记 Snapshot / 需要关注。标签过滤不等于总板上自动出现独立分组，分组呈现仍待界面验收 |

以上核到固定 commit 的过滤解析、owner 权限、默认视图与标签关系源码，未做两种路线的浏览器体验验收。saved filter 能提供个人 Repo 总视图，但不能被写成已经解决共享总板；单 project 加标签也没有消除分组歧义。

P2.2 庚的绑定实测能力应明确记录：任务 PUT / PATCH 不具备 `If-Match` 条件写保护，按后端已有语义写入并回读；本地绑定版本与过期预览检查仍照常执行。这是[系统边界 §固定内核与受控端口](../../design/spec/system.md#固定内核与受控端口)要求记录的「实测能力与降级方式」，不是用写前回读冒充服务端原子保护。

Vikunja 服务与 OpenAPI 客户端路线维持；原文「progenitor 0.14.0 作 build 依赖」的实施前提尚未成立。P2.2 的生成客户端工作应先解决上述生成器兼容性，随后才是 tasks、bucket、position、webhook 与 routes 的编译和实际往返验证；不把 Task 映射研究完成写成客户端可交付。

OpenAPI Generator `7.25.0` 是 Java 工具：运行官方 JAR 至少需要 Java 11 运行时，并不要求完整 JDK，但仍新增构建机依赖。若采用，单独论证并钉定 JAR 与三平台 Java 工具链，作为 Buck action 输入，不依赖宿主预装，也不成为用户运行依赖。如果它仍不能正确生成并通过实际往返验证，按既定四级顺序降到手写实际调用子集是合法兜底；以本次 17 路径诊断集为起点核齐所需操作，用钉定 OpenAPI 快照与真实服务做契约测试，不把 17 路径当作完整能力清单，也不无限叠生成器补丁。本轮未运行这个后备生成器。

生成器许可和输入规范许可分开记录。这里仅确认发布包所列许可证，不用「构建期」或「独立进程」直接推出生成物的许可结论；原文对生成物归属的断言仍需发行时核验，不在本次扩展为新许可决策。

#### 决定建议

**服务继续采用二进制 Vikunja `v2.5.0 / ef2200e9`；客户端维持从 OpenAPI 生成的方向，但暂缓采用 progenitor `0.14.0`。** 理由是已实测的三种生成失败，原件无法产出可编译客户端；按原定后备顺序，下一候选钉 OpenAPI Generator **`7.25.0`**，尚未验证，采用前须解决新增 Java 构建依赖。它也失败时才进入手写实际调用子集与契约测试，不继续无限修补生成器。

分组映射暂缓定案：子 project 加 saved filter 能提供个人总视图，但不共享；单 project 加标签保留共享总板，却有分组歧义。P2.2 庚开工前按这些代价确定映射并验原生界面。PUT/PATCH **不具备 If-Match 条件写保护**，进绑定的实测能力与降级说明。以上只推翻被实验否定的两项判断，不重开 Vikunja 服务选型。

#### 证据

- 钉定服务：[v2.5.0 官方制品](https://github.com/go-vikunja/vikunja/releases/tag/v2.5.0)、[`huma.go` 的规格导出](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/routes/api/v2/huma.go)、[`tasks.go` 读写参数与处理](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/routes/api/v2/tasks.go)。上表 HTTP 响应与 Buck 错误是本机实测，不是文档引述。
- 映射与观测：[project 父 ID](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/project.go)、[webhook 发送](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/webhooks.go)、[限流处理](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/routes/rate_limit.go)、[默认配置](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/config/config.go)。
- 生成器：[progenitor 0.14.0](https://crates.io/crates/progenitor/0.14.0)、[失败分支所在的 method.rs](https://docs.rs/crate/progenitor-impl/0.14.0/source/src/method.rs)、[OpenAPI Generator v7.25.0](https://github.com/OpenAPITools/openapi-generator/releases/tag/v7.25.0)。
- 本轮映射复核（同一 `ef2200e9`）：[filter 的 v2 路由](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/routes/api/v2/saved_filters.go)、[创建与默认视图](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/saved_filters.go)、[owner-only 权限](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/saved_filters_permissions.go)、[`project in` 解析测试](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/task_collection_filter_test.go)、[按 view 存储 bucket](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/kanban_task_bucket.go)、[标签关系](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/label_task.go)。[官方 saved filter 说明](https://vikunja.io/help/saved-filters/)作交叉核对，行为以钉定源码为准。
- Java 依赖：[OpenAPI Generator 官方安装说明 §JAR](https://openapi-generator.tech/docs/installation/#jar)要求最低 Java 11 runtime；本轮未下载或运行 JAR，未验证其生成客户端。

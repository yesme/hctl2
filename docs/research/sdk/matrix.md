# Matrix 客户端层：ruma 与 matrix-sdk

> 状态：调研 · 日期：2026-09-03<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-SDK-MATRIX<br>
> 对象：Matrix 协议 spec v1.18（2026-03-26 发布）· homeserver Tuwunel `v1.9.0`（备选 Continuwuity）· 候选库 [ruma `0.16.0`](https://crates.io/crates/ruma)（2026-05-31）、[matrix-sdk `0.18.0`](https://crates.io/crates/matrix-sdk)（2026-06-02）<br>
> 许可证：ruma 全部 crate MIT；matrix-sdk Apache-2.0；Tuwunel Apache-2.0；reqwest MIT OR Apache-2.0

## 定位

`hctl2-control` 以 **AppService（应用服务）** 身份接到 Tuwunel，不是普通客户端登录。需要的调用面分两个方向：

| 方向 | 调用 | 说明 |
| --- | --- | --- |
| homeserver → control | `PUT /_matrix/app/v1/transactions/{txnId}` 事件投递；`POST …/ping` 连通性检查；`GET …/users/{userId}`、`…/rooms/{alias}` 查询 | control 得起一个 HTTP 服务端点接收推送；按 `txnId` 去重 |
| control → homeserver | AppService 注册（as_token / hs_token / namespaces / url）；注册虚拟用户（`m.login.application_service`）、建房、邀请、加入；按事件 ID 读正文 `GET /_matrix/client/v3/rooms/{roomId}/event/{eventId}`；读房间加密状态 `GET /_matrix/client/v3/rooms/{roomId}/state/m.room.encryption/` | 用 as_token 鉴权；必要时以 `user_id` 查询参数假冒命名空间内的虚拟用户 |

HCTL 不做端到端加密：读 `m.room.encryption` 只是为了识别并拒绝或标记加密房间，因此不需要密钥库、设备管理和 `/sync` 循环。

## 上游能力

**官方 SDK：AppService 场景没有。** matrix-org 维护的 [matrix-rust-sdk](https://github.com/matrix-org/matrix-rust-sdk) 定位是"客户端库"：

- 源码里看到：`matrix-sdk 0.18.0` 的 `crates/` 目录只有 base / common / crypto / sqlite / indexeddb / ui / search / qrcode / contentscanner / store-encryption 和主 crate，**没有 appservice crate**；`client/builder` 里也没有 appservice、as_token、assert_identity 字样。
- 历史：`matrix-sdk-appservice` 于 2023-09-05 被 [PR #2509](https://github.com/matrix-org/matrix-rust-sdk/pull/2509) 移除；crates.io 上同名 crate 只有 `0.0.1-reserved` 占位（2022-05-12，描述写着 "SOON"），从未发布正式版。
- 版本：`0.18.0`（2026-06-02）在 crates.io 标 `rust-version = 1.93`；main 分支 `Cargo.toml` 已升到 1.95。MSRV 抬得快（0.16.1 还是 1.88）。它依赖 ruma `0.16.0`。

**事实标准类型库：ruma。** [ruma](https://github.com/ruma/ruma) 是 Matrix 协议的 Rust 类型与 trait 集合，matrix-sdk 建在它上面，Tuwunel 也用它（fork：`matrix-construct/ruma`，源码里看到 `Cargo.toml` 用 `git` 依赖锁定 rev）。

- 版本：`ruma 0.16.0`（2026-05-31）、`ruma-appservice-api 0.16.0`（2026-05-31）、`ruma-client-api 0.24.0`（2026-05-31）；全部 MIT；crates.io 标 `rust-version = 1.89`，README 说 "Ruma currently requires Rust 1.89"，并承诺 crates.io 发布不依赖 beta/nightly。
- 与 rustc 1.98.0 兼容。
- feature 切分（源码里看到 `crates/ruma/Cargo.toml`）：`appservice-api-c` / `appservice-api-s`（分别是 AppService 作客户端、作服务端的一侧）、`client-api-c` / `client-api-s`、`events`、`rand`、`html`、`markdown`，以及一组 `compat-*` 容错开关。
- `ruma-appservice-api` 模块：`event`（transactions 推送）、`ping`、`query`（用户 / 别名查询）、`thirdparty`；文档链接指向 spec v1.18。`Namespace` / `Namespaces` 注册结构体也在这里。
- `ruma-client-api` 有 `room::get_room_event`（按事件 ID 读正文）、`state::get_state_event_for_key`（读 `m.room.encryption`）、`account::register`、`room::create_room`、`membership::*` 等。
- `ruma-common::api::SendAccessToken` 有 `Appservice(&str)` 变体，专门用于 as_token；请求 / 响应通过 `OutgoingRequest` / `IncomingRequest` 与 `http::Request` 互转，**不自带 HTTP 客户端或服务端**——需要配 [reqwest `0.13.4`](https://crates.io/crates/reqwest)（发）和 axum 之类（收）。
- `user_id` 假冒参数是否有内建帮助方法本次未逐一核对，P2 开工时确认；最坏情况是在 URL 上自己附加查询参数。

**接口描述：有但不必用。** Matrix 规范仓库 [matrix-spec](https://github.com/matrix-org/matrix-spec/tree/main/data/api) 以 OpenAPI 维护各端点定义，ruma 已把它们做成类型；再从 OpenAPI 生成一套没有意义。

**homeserver 侧（Tuwunel）。** 官方文档 [`docs/appservices.md`](https://github.com/matrix-construct/tuwunel/blob/v1.9.0/docs/appservices.md)（v1.9.0）说注册有三种方式并存：

1. 管理房间发 `!admin appservices register` 并附 YAML——持久化进数据库、无需重启、重复 ID 覆盖；
2. `tuwunel.toml` 内联 `[global.appservice.<id>]`——每次重启重新加载，不能用 `unregister` 删；
3. `appservice_dir` 指向一个 YAML 目录——与 Synapse 桥接产出的注册文件同格式，增删要重启。

对 control 来说 2 / 3 是可编程路径（写文件再起服务），1 是运维路径。相关字段：`rate_limited` 默认 `false`（bot 用户永远免限速）、`receive_ephemeral`、`msc3202_transaction_extensions`、`appservice_timeout` 默认 35 秒。Continuwuity 的注册方式本次未核对。

## 候选比较

| 候选 | 版本 / 许可 | MSRV | 覆盖 HCTL 调用面 | 代价 | 判定 |
| --- | --- | --- | --- | --- | --- |
| ruma（`client-api-c` + `appservice-api-s` + `events`） | 0.16.0 / MIT | 1.89 | 全部：注册结构、transactions 接收类型、账号 / 房间 / 事件 / 状态端点 | 自己接 HTTP（reqwest 发、axum 收）；类型层没有重试、限速、去重 | **采用** |
| matrix-sdk | 0.18.0 / Apache-2.0 | 1.93（main 已 1.95） | 客户端端点可用，但没有 appservice 模式、不接收 transactions | 拖入 E2EE、存储、`/sync`；MSRV 抬升快 | 仅参考 |
| 从 matrix-spec OpenAPI 生成 | spec v1.18 | — | 理论上全覆盖 | 重造 ruma 已有的类型 | 不做 |
| 手写 serde 结构 | — | — | 按需 | 自己追 spec 变化 | 不做 |

## 边界与取舍

- **鉴权**：control → homeserver 用 as_token（`Authorization: Bearer`），homeserver → control 用 hs_token；两把令牌都在注册 YAML / TOML 里，由 control 生成并写入 Tuwunel 配置。虚拟用户假冒靠 `user_id` 查询参数，只能落在注册的命名空间内。
- **速率限制**：Tuwunel 对 AppService 的 `rate_limited` 默认关闭，bot 用户永远免限速；HCTL 自己的写入节流仍要做（同一房间线性顺序依赖单 homeserver，见 [chat server 选型](../matrix-homeserver.md)）。
- **事件与 webhook**：Matrix 没有 webhook，AppService 推送本身就是"服务端主动投递"——homeserver 会重试同一 `txnId`，control 必须以 `txnId` 幂等；`appservice_timeout` 35 秒内要回 200。`ping` 端点用于确认 control 端点可达。
- **Windows**：ruma 与 reqwest 都是纯 Rust（rustls），客户端层没有平台问题；Tuwunel 本身不在第一阶段 Windows 验证矩阵，那是 homeserver 条目的事。
- **不选 matrix-sdk 的另一个理由**：它的 store / crypto 是为完整客户端准备的，HCTL 只读 `m.room.encryption` 状态、不解密，引入后大部分代码是死重。

## 决定建议

- 三级判定：**第一级（官方 / 事实标准 SDK）——类型层成立，选 ruma**。Matrix 官方没有 AppService Rust SDK；ruma 是 matrix-sdk 与 Tuwunel 共同的底层类型库，属事实标准。
- 借用等级：**采用 SDK**（ruma `0.16.0`，features `client-api-c`、`appservice-api-s`、`events`、`rand`）+ reqwest `0.13.4` 发请求 + axum 接收 transactions；matrix-sdk **仅参考**（看它怎么组织 `Client` 与 `Room` 的错误处理即可）。
- 注册走 Tuwunel 的配置文件 / 目录路径，由 control 生成注册文件；管理房间命令只作运维兜底。
- 开工前要做的核对：ruma 0.16 的 `user_id` 假冒参数写法；`compat-*` 开关是否有 Tuwunel 需要的项（Tuwunel 自己用的是带 `__compat` 的 fork）。

## 证据

- matrix-sdk：[crates.io](https://crates.io/crates/matrix-sdk)（0.18.0，2026-06-02，Apache-2.0，rust-version 1.93）· [crates/ 目录](https://github.com/matrix-org/matrix-rust-sdk/tree/matrix-sdk-0.18.0/crates) · [main Cargo.toml](https://github.com/matrix-org/matrix-rust-sdk/blob/main/Cargo.toml)（rust-version 1.95）· [PR #2509 Remove matrix-sdk-appservice](https://github.com/matrix-org/matrix-rust-sdk/pull/2509) · [matrix-sdk-appservice 占位 crate](https://crates.io/crates/matrix-sdk-appservice)
- ruma：[crates.io ruma](https://crates.io/crates/ruma) · [ruma-appservice-api](https://crates.io/crates/ruma-appservice-api) · [ruma-client-api](https://crates.io/crates/ruma-client-api) · [README（MSRV 1.89）](https://github.com/ruma/ruma/blob/ruma-0.16.0/README.md) · [ruma Cargo.toml features](https://github.com/ruma/ruma/blob/ruma-0.16.0/crates/ruma/Cargo.toml) · [ruma-appservice-api 源码](https://github.com/ruma/ruma/tree/ruma-appservice-api-0.16.0/crates/ruma-appservice-api/src) · [ruma-client-api room/](https://github.com/ruma/ruma/tree/ruma-client-api-0.24.0/crates/ruma-client-api/src/room) 与 [state/](https://github.com/ruma/ruma/tree/ruma-client-api-0.24.0/crates/ruma-client-api/src/state) · [SendAccessToken](https://github.com/ruma/ruma/blob/ruma-0.16.0/crates/ruma-common/src/api/auth_scheme.rs)
- Matrix 规范：[Application Service API v1.18](https://spec.matrix.org/v1.18/application-service-api/) · [v1.18 发布博文（2026-03-26）](https://matrix.org/blog/2026/03/26/matrix-v1.18-release/) · [matrix-spec OpenAPI 数据](https://github.com/matrix-org/matrix-spec/tree/main/data/api)
- Tuwunel：[docs/appservices.md @ v1.9.0](https://github.com/matrix-construct/tuwunel/blob/v1.9.0/docs/appservices.md) · [Cargo.toml（ruma fork 依赖）](https://github.com/matrix-construct/tuwunel/blob/main/Cargo.toml) · [Release v1.9.0（2026-08-19）](https://github.com/matrix-construct/tuwunel/releases/tag/v1.9.0) · [Continuwuity](https://github.com/continuwuity/continuwuity)（备选，未核对）
- 本仓库：[chat server 选型](../matrix-homeserver.md) · [部件矩阵 provider 适配客户端行](../component-matrix-20260902.md)

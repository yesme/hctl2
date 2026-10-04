# Herdr 客户端层：socket 协议与 JSON Schema

> 状态：调研 · 日期：2026-09-03<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-SDK-HERDR<br>
> 对象：[Herdr `v0.8.2 / 9eb52145`](https://github.com/herdrdev/herdr/tree/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c)（2026-08-19；协议 `protocol: 20`，`schema_version: 1`；上游最新稳定版仍是 v0.8.2）· 生成器候选 [typify `0.7.0`](https://crates.io/crates/typify)（2026-06-05）<br>
> 许可证：Herdr Apache-2.0（v0.8.0 起，之前 AGPL-3.0-or-later）；typify Apache-2.0

## 定位

Agency 参考实现的运行时。HCTL 的 Agency adapter 通过 Herdr 的本地 socket 驱动 Harness 进程、终端会话与观察：

| 调用面 | Herdr 方法（原始 socket 名） | 备注 |
| --- | --- | --- |
| 按规格启动 harness | `agent.start`；底层容器 `workspace.create` → `tab.create` → `pane.split` | 启动参数与冻结 spec 的逐项核对见 [验证记录](../runtime/agency-runtime-validation-20260829.md) |
| workspace / tab / pane / terminal 创建与定位 | `workspace.*`、`tab.*`、`pane.list/get/current`；公开 pane id 形如 `w1:p1`，另有稳定 `terminal_id` 与 `revision` | `session.snapshot` 一次性拉全量作本地缓存的起点 |
| 输入与 resize | `pane.send_text`、`pane.send_keys`、`pane.send_input`、`pane.resize`；`agent.send_keys`、`agent.prompt`（可带 `wait`） | 语义层（agent.*）在目标 Agent 不再占据该 pane 时会拒绝 |
| 观察与断线重连 | `pane.read`（`visible` / `recent` / `recent-unwrapped` / `detection`）、`pane.wait_for_output`、`events.subscribe`、`events.wait`、`agent.wait`；重连后再调 `session.snapshot` 重建缓存 | 订阅连接保持打开，事件按行推送 |
| 停止与退出状态 | `pane.close`、`server.stop`；事件 `pane.exited`、`pane.agent_status_changed` | `PaneExited` / `PaneInfo` **没有退出码**（验证记录已按源码确认） |
| 事件游标 | 无持久游标。`seq` 只用于 hook 上报 Agent 状态时的乱序保护；`EventHub` 只留内存里最近 512 条，内部序号不进 envelope | 完整 trace 要 Herdr 上游补 output sequence / gap 事件，或由 Harness adapter 另存 |

## 上游能力

**官方 SDK：没有独立 SDK。** 官方文档把接入分三层：Agent skill（教 Harness 在 pane 里用 Herdr）、CLI 包装（`herdr … --json`）、原始 socket API——"三层共享同一控制面"。没有任何语言的客户端库。

- Herdr 是 **bin-only crate**（`Cargo.toml` 只有 `[package]`，没有 `[lib]`），不能作为 cargo 依赖引入类型。
- crates.io 上的 [`herdr 0.1.0`](https://crates.io/crates/herdr)（2026-03-27，AGPL-3.0-or-later，仓库 `ogulcancelik/herdr`）是迁到 `herdrdev` 组织之前的旧发布，既不是 SDK、许可也不同，**不要用**。
- 工具链：`rust-toolchain.toml` 钉 `1.96.1`；HCTL 用 1.98.0 不受影响（我们不编译 Herdr）。

**接口形态（官方文档 socket-api）：** 本地 socket 上的 **newline-delimited JSON**。Unix 是 Unix domain socket（`~/.config/herdr/herdr.sock`，命名会话在 `sessions/<name>/herdr.sock`；解析顺序 `--session` → `HERDR_SOCKET_PATH` → `HERDR_SESSION` → 默认），Windows 是 named pipe。一行一个请求 `{"id","method","params"}`；成功回 `{"id","result":{"type",…}}`，失败回 `{"id","error":{"code","message"}}`；订阅的第一条响应是确认，之后的行是事件。形似 JSON-RPC 但不是 2.0（没有 `jsonrpc` 字段；源码里看到 `Method` 枚举用 serde `tag = "method", content = "params"`）。文档要求客户端"处理未知字段"，并先用 `ping` / `herdr status` 核对协议版本。

**接口描述：有，JSON Schema 2020-12。**

- 生成方式：`herdr api schema --json`（或 `--output PATH`）由钉定二进制导出；源码里看到类型在 `src/api/schema/*.rs`，用 schemars `1.2.1` derive。
- 仓库里 check-in 了一份：[`docs/next/api/herdr-api.schema.json`](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/docs/next/api/herdr-api.schema.json)（255,484 字节）。顶层 `{ "$schema": …/draft/2020-12/schema, "title": "Herdr API", "protocol": 20, "schema_version": 1, "schemas": {…} }`，`schemas` 里是五个独立子文档：`request`（`oneOf` 91 个变体、107 个 `$defs`）、`success_response`（67 个 `$defs`）、`error_response`、`event`（16）、`subscription_event`（10）。
- 发布包里没有这份 schema（v0.8.2 资产只有五个平台二进制），要从仓库 tag 取或让钉定二进制导出。

## 候选比较

| 候选 | 版本 / 许可 | 做法 | 风险 | 判定 |
| --- | --- | --- | --- | --- |
| typify 从 JSON Schema 生成 | 0.7.0 / Apache-2.0 | 把 `schemas.request` 等五个子文档分别喂给 typify，得到 serde 类型 | typify 内部用 schemars `0.8.22` 的数据模型（draft-07 时代），对 2020-12 的支持是"进行中"（[issue #579](https://github.com/oxidecomputer/typify/issues/579) 开放）；`#/$defs/` 引用能走（[issue #828](https://github.com/oxidecomputer/typify/issues/828) 反证它只认这种形式）；91 变体的 internally-tagged `oneOf` 能否还原成 `method`/`params` 平铺形状**未验证** | **首选尝试**，需生成实验 |
| 移植 `src/api/schema/` 的类型 | Apache-2.0 | 抄 `schema.rs` + `schema/{agents,common,events,panes,…}.rs`（去掉 `tests.rs`，约 90 KB），保留版权声明 | 每次升级 Herdr 要 diff 同步；但 wire 形状与 Herdr 自己的 client 完全一致 | typify 不行时的**兜底** |
| 调 `herdr` CLI（`--json`） | 二进制已采用 | 一次性操作 spawn 一个进程 | 订阅 / 长连接不适合进程模型；每次调用多一次进程启动；输出仍要解析 | 只作调试与 skill 路径 |
| 手写子集 | — | 按文档写十几个 method 的结构 | 自己追协议版本 | 不做 |

## 边界与取舍

- **鉴权**：本地 socket 没有令牌，文件系统权限就是鉴权；远程接入靠 SSH 瘦客户端 / `--remote`（见 [Herdr 条目](../runtime/herdr.md)）。socket 层的额外认证未查到。控制权（单控制者、显式 `--takeover`）是 Herdr 进程内映射，没有代次与 TTL——HCTL 的输入租约仍在 control 侧。
- **速率限制**：无。
- **事件与 webhook**：没有 webhook，只有 socket 订阅。事件流可丢（内存环 512 条）、无持久游标、无 gap 通告；断线只能 `session.snapshot` 重锚。`agent.wait` 只看语义状态，不等于某一轮提示完成。这些边界已在验证记录里定性为"可作 UI 与诊断观察，不能冒充完整 trace"。
- **退出事实**：pane 退出事件与 pane 信息都没有退出码；`agent.start` 起的 Harness 退出后通常回到 pane 里的 shell，pane 仍活着。通用 PTY 路径要靠进程 incarnation 与退出码回读补齐。
- **Windows**：传输是 named pipe，文档说"原始 socket 客户端自己负责用平台原生的本地 socket 形式"；上游有 `herdr-windows-x86_64.zip`。第一阶段不验证 Windows，但传输层抽象时留出 named pipe 位。
- **协议版本耦合**：`protocol: 20` 是 Herdr 私有协议，不是行业标准；升级 Herdr 时用钉定二进制 `herdr api schema --json` 重新导出，与 check-in 快照比对（CT），漂移即重跑生成。

## 决定建议

- 三级判定：**第二级（从接口描述生成）**。没有 SDK；有官方导出的 JSON Schema。
- 借用等级：**采用 SDK**（typify `0.7.0` 作 build 依赖，输入钉定版本导出的 schema 快照）；若生成实验失败或 wire 形状不符，退到**复制代码**——把 `src/api/schema/` 移植为有边界组件（只取类型，不取 server / client 逻辑，保留 Apache-2.0 声明）。Herdr 二进制本身**采用二进制**（已定）。
- 传输层（NDJSON over Unix socket / named pipe、请求 id 配对、订阅分流）自己写，很薄。
- 生成实验验收：用 `herdr api snapshot` 与文档示例的真实 JSON 做 round-trip；`pane.resize`、`events.subscribe`、`agent.prompt` 三个请求与对应响应 / 事件能无损往返。

## 证据

- 发布与源码：[Release v0.8.2（2026-08-19）](https://github.com/herdrdev/herdr/releases/tag/v0.8.2) · [`Cargo.toml` @ v0.8.2](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/Cargo.toml)（bin-only、schemars 1.2.1、Apache-2.0）· [`rust-toolchain.toml`](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/rust-toolchain.toml) · [`src/api/schema.rs`](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/src/api/schema.rs) 与 [`src/api/schema/`](https://github.com/herdrdev/herdr/tree/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/src/api/schema)
- 接口描述：[`docs/next/api/herdr-api.schema.json` @ v0.8.2](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/docs/next/api/herdr-api.schema.json)
- 官方文档：[socket-api.mdx @ v0.8.2](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/docs/next/website/src/content/docs/socket-api.mdx)（三层接入、schema 导出、方法表、传输、socket 路径、事件订阅、响应形状、协议稳定性）· [herdr.dev/docs](https://herdr.dev/docs)
- 旧 crate：[crates.io herdr 0.1.0](https://crates.io/crates/herdr)（AGPL，非 SDK）
- 生成器：[typify crates.io](https://crates.io/crates/typify)（0.7.0，Apache-2.0）· [typify 工作区 Cargo.toml（schemars 0.8.22）](https://github.com/oxidecomputer/typify/blob/v0.7.0/Cargo.toml) · [issue #579 2020-12 计划](https://github.com/oxidecomputer/typify/issues/579) · [issue #828 `$defs` 引用](https://github.com/oxidecomputer/typify/issues/828)
- 本仓库：[Herdr 条目](../runtime/herdr.md) · [运行服务验证记录](../runtime/agency-runtime-validation-20260829.md) · [部件矩阵](../component-matrix-20260902.md)

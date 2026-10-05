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

## 复核记录


- **2026-10-05 · 3c 结束与隔离复核（PR #335）**：锁定版的 `pane.exited` 仍没有退出码。退出文件放在执行目录中时，程序能提前写出假退出码。只把文件移到另一目录也不能证明拒写。macOS 实测在已经受限的 Herdr 下再次运行 `sandbox-exec` 返回 71，原因是 `sandbox_apply: Operation not permitted`，不能用嵌套配置补出第二层限制。
- **同日 · 当前适配选择**：非交互 Claude print 由 Agency 用标准库 `Command` 启动、`Child::try_wait` 取得内核退出状态，stdout / stderr 用原生 Unix socket 成对收取，不从屏幕解析。Herdr 继续持有窗格和 PTY；固定启动脚本只交回该 PTY 的设备路径，Agency 把收取的字节显示到这个终端。真实执行单独应用一次既有 sandbox / Landlock，拒绝 Agency 凭据根、Herdr 状态及 socket 目录；无需新依赖、PTY 实现或服务协议。相比上游持有 Harness 子进程，这是明确的内部适配取舍：本版不承诺该 Harness 是 Herdr 的前台进程，不激活原生交互、exact attach、Herdr Agent 检测或会话恢复。以后启用这些能力须另验，不把终端显示当作进程归属证明。接口依据仍为上面的锁定源码；子进程与退出依据见 [Rust 标准库 Child](https://doc.rust-lang.org/std/process/struct.Child.html)。

### 2026-10-05 · Herdr 0.9.3 的 C3 类型生成实验（PR #335）

所有者已撤销上一条由 Agency 持有 Claude 进程的选择。3c 改用官方 Herdr 0.9.3 持有 Harness 与会话；本次 C3 只核接口类型生成，不代替 3c 的四条验收。

**决定建议：本轮不接入 typify 生成的整套协议类型，保留手写的小范围调用。** typify 0.7.0、0.8.0 都能生成并编译，但生成的 Request 没有按方法选择参数类型，不能替代 Herdr 的正式请求类型。不另写 schema 转换器，也不复制上游整套类型。此前「手写子集不做、失败就复制代码」的候选建议由本次所有者裁定覆盖；3c 仍须升级官方 Herdr，C3 的回退不阻挡它。

#### 基线与做法

- 官方 [Herdr v0.9.3 / 7b116c05](https://github.com/herdrdev/herdr/tree/7b116c05bfda646af39d2524c54e70c751f57ee8) 的 macOS arm64 二进制，SHA-256 `5173a3e0ae42d5d1ab7ebfa5d5e6329f7c3d23f8e1a3677c7ce3231da2884157`。本次没有编译或修改 Herdr。
- 输入用 `herdr api schema --json` / `--output` 导出，两种形式同源。导出文件与该 tag 的 [`herdr-api.schema.json`](https://github.com/herdrdev/herdr/blob/7b116c05bfda646af39d2524c54e70c751f57ee8/docs/next/api/herdr-api.schema.json) 摘要一致：`9e2af207e9aa8183d4aeca5fde9cc48e7909bb40cdbd7cf21608a6d3ea78075b`。协议 22，schema_version 1；request 有 102 个变体、118 个 `$defs`，success_response / error_response / event / subscription_event 分别有 72 / 1 / 16 / 10 个 `$defs`。
- 生成器分别钉 [typify 0.7.0](https://github.com/oxidecomputer/typify/tree/v0.7.0) 与 [0.8.0](https://github.com/oxidecomputer/typify/releases/tag/v0.8.0)，关闭 macro 特性；两者使用 schemars 0.8.22。通过 Cargo 解析依赖、Reindeer 生成依赖目标，构建与执行都走临时 Buck2 原生 Rust 目标；rustc 1.98.0。
- 原样取五个 `schemas` 子文档，各自调用下列官方 API。不改 `$ref`、`const` 或 `oneOf`；输出由 genrule 生成，再用 Rust 目标编译五个模块。生成的正则校验类型需要 `regress`，补齐该依赖后两个版本都编译通过。

```rust
let schema: schemars::schema::RootSchema =
    serde_json::from_value(bundle["schemas"][name].clone())?;
let mut types = typify::TypeSpace::default();
types.add_root_schema(schema)?;
let generated = types.to_stream().to_string();
```

#### 实测结果

| 核验 | 0.7.0 | 0.8.0 | 判定 |
| --- | --- | --- | --- |
| 五份 schema 生成、五个模块编译 | 通过 | 通过 | `$defs` 与 Herdr 的引用路径不是本次阻碍；0.7.0 输出共 638,260 字节，含生成的说明 |
| ping、pane.resize、events.subscribe、agent.prompt 的正常请求 JSON 往返 | 字段保留 | 字段保留 | 只证明字节内容可传回，不证明参数类型选对 |
| pane.resize / agent.prompt 的 Request 反序列化 | 都选 `Variant0`，参数是 `PingParams` | 同左 | `PingParams` 是通用键值对象，不是该方法的参数类型 |
| 不存在的方法 `not.a.method` | 接受 | 接受 | 官方 0.9.3 服务实测拒绝，`invalid_request` |
| pane.resize 的 direction 为 diagonal | 接受 | 接受 | 官方服务实测拒绝，`invalid_request` |
| agent.prompt 缺少 text | 接受 | 接受 | 官方服务实测拒绝，`invalid_request` |
| 直接反序列化生成的 PaneResizeParams / AgentPromptParams | 拒绝上述非法参数 | 同左 | 单个参数类型有价值，问题在完整 Request 的方法与参数配对 |
| 官方服务的 ping、session.snapshot 与三种拒绝响应，经生成类型往返 | 五份响应都保留 | 同左 | 使用独立临时 socket，不接触正在工作的 Herdr 会话 |

原因已核源码：两个版本的 [`convert.rs`（0.7.0）](https://github.com/oxidecomputer/typify/blob/v0.7.0/typify-impl/src/convert.rs)、[`convert.rs`（0.8.0）](https://github.com/oxidecomputer/typify/blob/v0.8.0/typify-impl/src/convert.rs) 都把 `const_value` 去掉再转换。Herdr 的请求恰好用 `method.const` 区分 102 个分支；生成结果变成未标记的枚举，首个宽松的 PingParams 就能接住其他请求。正常往返通过，仍然没有得到预期的接口类型。这不是官方 Herdr 接受了非法请求，也不是字段在传输时丢失。

临时目标为 `root//agency/schema-experiment:experiment` 与 `:roundtrip`，均经 `./buck2 run` 实跑；完整往返的 Build ID 分别为 `3ef813f2-28a5-41ef-971c-61608b80c730`（0.7.0）与 `243b9302-1f4a-483e-a131-158cb8e7a166`（0.8.0），各含请求、反例与真实响应。实验源码与输出留在本机临时目录，临时目标和依赖不进入生产构建。

#### 回退范围与未验证

保留范围是薄 NDJSON 传输、id 配对、协议握手，以及 3c 正式路径实际调用的少量方法和响应字段。它们对照钉定版本的官方类型与 schema，并由原生接口用例核验；不承诺支持 Herdr 的全部方法。没有给生产构建增加 typify、schemars、regress 或一套生成工具。

本次没有做真实订阅事件的生成类型往返，也没有用生成类型调用 Claude；不声称整套协议已验证。C3 到此采用允许的回退。锁文件升级、3b 回归、受限环境里的真实 Claude 派工仍按 3c 四条验收另核，本条不代表 3c 已完成。

- **2026-10-05 · 3c 新验收的接法**：官方 Herdr 0.9.3，源码 `7b116c05bfda646af39d2524c54e70c751f57ee8`、协议 22。三平台发布摘要已对照发布页并下载核对：Linux x86_64 `18a8dc65f1c2fa485884344356dea1cfd911c6f06cf46fa78e193f4087f4dba7`，macOS arm64 `5173a3e0ae42d5d1ab7ebfa5d5e6329f7c3d23f8e1a3677c7ce3231da2884157`，macOS x86_64 `db62d548ff3e832b087a96b1894a08d26be3905f1830309cd556783f215d4054`。原生 `layout.apply` 的 `command` 是 argv 数组，走 `create_tab_argv_command` → `TerminalRuntime`，不把调用方正文敲进交互式 shell。Agency 只交固定脚本路径，Herdr 持有脚本与 Claude 子进程。Claude print 的原生 `stream-json` 写入本次调用的私有文件；逐行读取，匹配 `system/init` 与 `result` 的会话标识，一轮结果到达就交回 Proposal，不等退出、不从屏幕取。错误结果保留原因，不伪报成功。`pane.exited` 只确认物理退出，脚本记录的退出码只作观测；`pane.close` 沿 Herdr 原生 `shutdown` 停止进程。文件都在执行目录之外；继承的沙箱拒绝 Agency 凭据根。官方版没有逐 pane 的状态目录拒写，恶意伪造这一档留给策略面，不承诺这份输出是不可伪造的工具直报。来源：[原生布局启动](https://github.com/herdrdev/herdr/blob/7b116c05bfda646af39d2524c54e70c751f57ee8/src/app/api/layouts.rs)、[事件定义](https://github.com/herdrdev/herdr/blob/7b116c05bfda646af39d2524c54e70c751f57ee8/src/api/schema/events.rs)、[终端停止](https://github.com/herdrdev/herdr/blob/7b116c05bfda646af39d2524c54e70c751f57ee8/src/pane.rs)。本记录撤销上文 Agency 持有实际进程的接法；实际用例结果另追加，不以源码核对代替运行验证。

- **同日 · 本机新路径实测**：macOS arm64，官方 Herdr 0.9.3，已登录的 Claude Code 2.1.289。在 Aqua 用户会话中运行 Buck 构建的 `herdr_test`，设 `HCTL2_HARNESS_LIVE=1`。`Runtime::start` 得到 `Proposal(schema=claude.result.v1, source=adapter_event): HCTL2_REAL_OK`，随后 `TurnReturned`；真实 `agency start → pair → catalog → prepare → activate → results` 得到 `ResultReturned` 与 `HCTL2_PORT_REAL_OK`。两条都通过受限环境，不复制或修改登录材料。另有确定性用例：Claude 输出一轮结果后仍睡眠，Proposal 与 `TurnReturned` 先到，实际进程祖先含 Herdr；原生停止后进程消失。错误结果、取消前未答完、不同目录同时派工、凭据拒读、状态目录在执行目录之外也通过。接法只交回原始输出，不判断 Task 完成、sysone 结论或生成物合格。
- **同日 · 验证边界**：两条真实 Claude 用例默认用 Rust 原生 `ignore` 标明未验证，不计为通过；本机真实验证同时设 live 开关与 `--include-ignored`。当前本机实跑了 macOS arm64；Linux 与 macOS x86_64 的程序回归交 CI，真实登录会话未验证。没有第二家 Harness、原生交互、会话复用或恢复。逐 pane 状态目录防篡改仍未实现。C3 的类型生成回退不变，没有生产生成器依赖。
- **同日 · 完整包协议检查**：升级后，Linux CI 与本机 macOS 完整包都在旧的 `status server` 文本匹配处失败。0.9.3 的原生输出把 `protocol`、`compatible` 改名为 `private_protocol`、`private_protocol_compatible`；JSON 字段仍是 `protocol`、`compatible`。按钉定版的文本名称更新安装包烟测，保留精确版本、协议、兼容性和 owner-only socket 检查，并补明确的失败信息。依据：[官方状态命令](https://github.com/herdrdev/herdr/blob/7b116c05bfda646af39d2524c54e70c751f57ee8/src/cli/status.rs)。
- **同日 · 私有服务停止**：本机清理核对发现，Agency 退出时后台事件线程可能仍持有 Server 的引用，靠析构不能保证私有 Herdr 随之停止。真实服务端口用例加入三秒内进程消失断言，先失败、再通过。改为 Agency 关闭各 Session 后调用官方 `server.stop`；失败不报告停止成功。单次派工仍只关自己的 pane，不关闭共用服务。依据：[官方原生停止请求](https://github.com/herdrdev/herdr/blob/7b116c05bfda646af39d2524c54e70c751f57ee8/src/api/server.rs)。没有新写进程监督器。

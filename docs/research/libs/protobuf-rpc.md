# Protobuf：control 接口生成与本地传输

> 对象：Protobuf / protoc `36.1`；prost / prost-build `0.14.4`；tonic 系列 `0.14.6`；pbjson 系列 `0.9.0`（2026-09-06 核对）<br>
> 许可证：Protobuf BSD-3-Clause；prost Apache-2.0；tonic、pbjson MIT<br>
> 定位：P2.1 的 control—CLI 接口生成链与本地传输；研究证据 E-LIB-PROTOBUF-RPC，不定义领域正文格式。

## 上游能力

Protobuf 是消息 schema（字段与类型的机器可读定义），gRPC 是使用这些消息的调用协议。已定方向见 [P2 计划 §五](../../../.memo/design/p2-control-20260906/01-plan.md#五需要所有者一句话的取舍)：同包 Rust CLI 与 control 传二进制 Protobuf，`--json` 才渲染 ProtoJSON（Protobuf 规定的 JSON 映射）。

本次读取 crates.io 稀疏索引与对应发布包的 `Cargo.toml`、生成器和传输源码；下表是查询时最新稳定发布，不把 GitHub 的最新 tag 等同于 crates.io 最新包。MSRV 是库声明的最低 Rust 版本，不代表已验证所有传递依赖。

另在本库 `bb25282` 的隔离副本以 Reindeer 生成依赖目标，用钉定 Rust 1.98 / Buck2 编译检查 prost、prost-build、tonic、tonic-prost-build、pbjson、pbjson-build，macOS arm64 通过（`root//probe:versions`，Build ID `49358cc6-aa6b-4b77-a01a-b1b41abaa5ef`）。这补足 pbjson 未声明 MSRV 的本机证据，不等于三平台 RPC 已验收。接入使用 prelude 原生 Cargo build-script action；tonic 需要声明 `CARGO_PKG_VERSION`，相关依赖的 build script 需要 Reindeer fixup，没有改写上游源码。

| 候选 / 部件 | 钉定版本 | 许可证 | MSRV | 核对结果 |
| --- | --- | --- | --- | --- |
| prost / prost-build | 0.14.4 / 0.14.4 | Apache-2.0 | 1.85 | 消息类型、编码、描述符生成；不自带 RPC |
| tonic / tonic-prost / tonic-prost-build | 0.14.6，各包同版 | MIT | 1.88 | HTTP/2 RPC、客户端与服务端生成；0.14 的 prost codec / 生成器已拆包，不能只加 tonic-build |
| pbjson / pbjson-build | 0.9.0 / 0.9.0 | MIT | 未声明；edition 2024 要求至少 1.85 | 从同一描述符生成 serde 实现；pbjson-build 依赖 prost 0.14；未声明 MSRV 不写成已获上游保证 |
| protoc 官方二进制 | 36.1 | BSD-3-Clause | 不适用 | 三个目标均有官方 zip，含编译器与标准 `.proto` |
| protox | 0.9.1 | MIT OR Apache-2.0 | 包声明 1.74；其 prost 0.14 依赖实际至少 1.85 | 纯 Rust 编译到描述符，可交 `compile_fds`；不是独立的 Rust RPC 生成器 |
| Protobuf-ES / protoc-gen-es（P3） | 2.14.1 / 2.14.1 | Apache-2.0 AND BSD-3-Clause / Apache-2.0 | 不适用；生成器 Node ≥22 | Buf 维护的 JS/TS 实现与 protoc 插件，不是 Google 自带的 TypeScript 生成器 |

### Buck2 接法

展开仓库钉定的 Buck2 `2026-08-22` 内置 prelude，检索 `decls/`、`rust/` 和 proto 文件：有 Android 工具目录的 `protobuf_src_gen`，但它写死 `--java_out` / `--grpc-java_out`，最后调用 Java 规则；没有可直接消费的 Rust proto 规则。这里用原生 `genrule` 与 `rust_library`，不是另搭调度脚本。

生成分两步：钉定 protoc 把显式列出的 `.proto`、imports 和标准 include 目录变成描述符集；一个 Buck `rust_binary` 调 `tonic_prost_build::compile_fds` 与 `pbjson_build::Builder::register_descriptors` 产出 Rust 文件。protoc 本身不直接生成 prost 类型。描述符也可经 prost-build 的 `file_descriptor_set_path` / `skip_protoc_run` 输入，均不用宿主 `PATH` 找编译器。

生成物放独立 package、独立 `rust_library`；control 与 CLI 依赖这个 crate。工具、schema、imports 都是 action 输入，消费方不重复运行生成器。protoc 只在开发 / CI 使用，不进入用户运行依赖。P2.1 乙起草 schema 时就确定是否使用 Timestamp 等标准消息；若使用，同时钉定 `prost-types` 与 `pbjson-types` 的兼容版本和映射，避免字段定稿后再换类型，不手写第二份类型。

## 候选比较

| 传输 | 版本 / 许可证 / MSRV | 已有能力 | 我们仍负责的部分 | 结论 |
| --- | --- | --- | --- | --- |
| tonic gRPC 经 Unix domain socket（本机进程间套接字） | tonic 0.14.6 / MIT / 1.88 | 方法分派、状态码、流、取消、超时、HTTP/2 流控 | socket 生命周期与权限；Subscribe 的业务游标和重同步 | 首选；多出 hyper/h2/tower 依赖，换掉通用 RPC 自建工作 |
| 长度前缀 + Protobuf 帧 | 自建；版本 / 许可证 / MSRV 不适用 | prost 只负责编解码 | 帧上限、请求配对、流结束、取消、错误、背压、重连全要自行补齐 | 不采用；同包客户端减少兼容负担，不消除这些机制 |

tonic `Endpoint::connect_with_connector` 接 `tokio::net::UnixStream`，客户端用 `hyper_util::rt::TokioIo` 适配 hyper 的 I/O trait；服务端用 `serve_with_incoming` 接 Unix listener 流。它不是 hyper 自动提供的 Unix URL scheme，也不需要为了本机通信开 TCP 端口。这里已核源码接口，未做 control 端到端吞吐或 RSS 测量。

## 边界与取舍

Subscribe 可用服务端流；断线后客户端带业务游标重新订阅，过期时重新取快照。gRPC 不保存事件、不自动补历史，也不使 Submit 自动幂等；这些沿用[系统边界 §命令与跨服务正确性](../../design/spec/system.md#命令与跨服务正确性)。收到传输错误但提交结果不明时，仍靠命令幂等键回读。

编译器选官方 protoc，不选 protox：两者都能摆脱宿主安装，但官方制品已覆盖三平台，沿用 DotSlash 即可，不需维护另一种 `.proto` 前端。建议 DotSlash 固定 `36.1`、下面的资产大小与 SHA-256，入口放在 `src/build/tools/`，与 gh / Process Compose 同列；Buck 中把已物化的编译器和 include 文件声明为输入，下载不藏在生成 action 里。

| 平台 | 官方资产 | 字节数 | SHA-256 |
| --- | --- | --- | --- |
| Linux x86_64 | `protoc-36.1-linux-x86_64.zip` | 3678715 | `c4bc672d9d49214dc8cafdceadf4df92182d6ca8e3ec65a56b2d7de5602669b4` |
| macOS arm64 | `protoc-36.1-osx-aarch_64.zip` | 2622007 | `de56d57afe30c5d191b11d24ff93dd4025728d7fb43b773886b2d3613e0bdbb2` |
| macOS x86_64 | `protoc-36.1-osx-x86_64.zip` | 2756492 | `ee2c5496e4af0aa6a224894bc0f7025145260e004d890487d510725ce8b473eb` |

P3 仍从同一 `.proto` 经 `protoc-gen-es` 生成 TypeScript；Buf CLI 是可选的 lint / 生成调度工具，Buf 远程注册服务不是前提。Tauri Rust 核心转发字节，webview 用 Protobuf-ES 解码，不要求浏览器直接连 Unix socket。本轮只核生成路线，不引入 Node 或 Workbench 代码。

进 Git 的 Task Revision 等领域正文继续用 RFC 8785 规范化 JSON；ProtoJSON 不是 JCS，Protobuf 字节也不保证规范化。传输中需要携带原正文时保留其原始字节和既有摘要，不拿传输重编码结果重新认定领域事实。出处是[接口备忘 §当前决定](../../../.memo/notes/control-api-schema-20260902.md#当前决定)；其中“暂不引入”的时机决定已由 P2 计划覆盖，事实格式边界仍成立。

P2.1 甲 / 乙起草接口时还有一个待定衔接点：[hctl2-tool 的 JSON 记录](../../../src/apps/hctl2-tool/src/lib.rs) 已有带版本的 `schema`、`evidence_level`、`outcome` 与 `error.code`，将由 control 消费。这些记录是否也由同一份 `.proto` 定义并以 ProtoJSON 输出，要在实现前明确，避免两份定义分别演进；[P2 计划 §三](../../../.memo/design/p2-control-20260906/01-plan.md#三p2-的形状) 已定 P1 接口不重塑，不能因换生成方式改变已有字段或语义。本文只登记这个问题，不据此改工具箱。

## 决定建议

**维持 Protobuf schema；采用 SDK：prost / prost-build `0.14.4`、tonic / tonic-prost / tonic-prost-build `0.14.6`、pbjson / pbjson-build `0.9.0`；采用二进制：protoc `36.1`，按三平台官方资产由 DotSlash 钉摘要。** 传输选 tonic gRPC over Unix socket，用现成流控、取消和错误机制；Buck 原生 action 生成独立 crate，省去自建 RPC 帧协议。P3 的 Protobuf-ES `2.14.1` 只记生成路线，Git 领域正文继续 JCS。

## 证据

- 发布包与源码：[prost-build 0.14.4 `config.rs`](https://docs.rs/crate/prost-build/0.14.4/source/src/config.rs)、[tonic 0.14.6 transport](https://docs.rs/crate/tonic/0.14.6/source/src/transport/)、[tonic-prost-build 0.14.6](https://docs.rs/crate/tonic-prost-build/0.14.6/source/src/lib.rs)、[pbjson-build 0.9.0](https://docs.rs/crate/pbjson-build/0.9.0/source/src/lib.rs)、[protox 0.9.1](https://docs.rs/crate/protox/0.9.1/source/Cargo.toml)。
- 官方制品：[protoc v36.1](https://github.com/protocolbuffers/protobuf/releases/tag/v36.1)；表中大小与摘要取发布 API，未把下载包大小说成运行内存。
- Buck：[本库二进制 pin](../../../src/build/tools/buck2-bin)、[上游 prelude 的 Android 生成规则](https://github.com/facebook/buck2-prelude/blob/main/toolchains/android/tools/protobuf.bzl)；判断基于本机展开的钉定 prelude，网页 main 仅方便定位。
- TypeScript：[Protobuf-ES](https://github.com/bufbuild/protobuf-es)、[2.14.1 包](https://www.npmjs.com/package/@bufbuild/protobuf/v/2.14.1)、[生成器 2.14.1](https://www.npmjs.com/package/@bufbuild/protoc-gen-es/v/2.14.1)。

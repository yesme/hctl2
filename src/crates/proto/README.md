# proto · 控制面传输合同

P2.1 乙的生成 crate。`.proto` 是 Query / Preview / Submit / Subscribe 的唯一接口合同；Rust 类型由 Buck `genrule` 经钉定 protoc 36.1、`tonic-prost-build` 与 `pbjson-build` 生成。消费方只依赖 `root//crates/proto:proto`。

错误对象三个字段是 `code`、`message`、`recovery_action`，与 `store` 对齐，不另定义 RPC 枚举。Git 领域正文仍是 JCS；本 crate 只承载进程间传输。`get` / `versions` 不是本服务的公共 Query。

生成链与版本钉定见 [protobuf-rpc.md](../../../docs/research/libs/protobuf-rpc.md) §决定建议。

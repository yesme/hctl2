# Control API schema 与传输选型记录

> 状态：当前边界已拍板，具体技术待 `hctl2-control` 开工时调研

## 定位

Query、Preview、Submit、Subscribe 是 HCTL 客户端访问 control 的语义边界；schema、序列化和 RPC transport 是实现选择。当前代码树尚无 control、公共 CLI 或 Workbench 调用链，也不存在两个 HCTL 进程之间需要兼容的 wire contract。一个仅包装 `code + message`、且只有 `hctl2-tool` 单一消费者的公共 crate 不是协议边界，而是过早抽象。

## 当前决定

- 删除 `hctl2-protocol`；现有稳定错误码和说明归 `hctl2-tool` 自己所有。
- 不为将来可能出现的共享类型预建 crate，不手写并行 DTO，也不现在引入 Protobuf、Buf、gRPC 或 Connect 依赖。
- Git 中作为领域事实、参与规范化与摘要的 HCTL 正文继续使用规范化 JSON。未来的传输 schema 只能承载这些语义，不能反向取代事实源格式。

## 重新选型的触发点

`hctl2-control` 出现首条真实的客户端调用链时，先固定客户端组合、进程边界、升级方式和兼容期限，再做对象级调研。若 Rust control、Rust CLI 与 TypeScript Workbench 需要共享一份长期演进的多语言合同，Protobuf 是优先候选；若调用只在 Tauri 2 的本地 Rust/TypeScript IPC 内发生且无独立版本兼容需求，原生 serde/JSON 可能更小。

选型必须同时回答两件事：消息 schema 用什么，以及 RPC/transport 用什么。Protobuf 只解决前者，不能成为自建 framing、路由、重试或流协议的理由。后者优先评估成熟实现，如 gRPC 或 Connect；若采用 Protobuf，`.proto` 是唯一 API schema，Rust/TypeScript 类型由官方工具链或 Buf 在 Buck action 中生成，不再维护手写公共 DTO crate。

正式引入任何候选前，按仓库纪律分别在 `docs/research/` 核对官方制品、许可证、Rust/TypeScript 生成链、Buck2 接法、兼容性检查和跨平台发行影响。

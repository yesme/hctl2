# Agency 端口合同

第 2 包的共享边界。`agency-proto` 不依赖任何一侧的数据库、命令内核或运行时。独立的 `.proto` target 定义 RPC；已有 protoc / prost / tonic 链生成传输代码，不另写 RPC 框架。

## 文件与目标

| 文件 | 职责 |
| --- | --- |
| `proto/agency.proto` | `hctl2.agency.v1` 的配对、目录、准备、激活、回读、输入、停止、观测与结果保全方法 |
| `src/model.rs` | 冻结引用、工种、能力、Execution Spec、票据、观测与 Proposal 头 |
| `src/context.rs` | Manifest、Bundle 与三种交付方式的记录类型 |
| `src/bytes_base64.rs` | SDK 驱动的 RFC 4648 字节字符串编码 |
| `src/client.rs` | 仅归属者可访问的本地 Unix socket 客户端；超时不是成功 |

Buck：`root//crates/agency-proto:generated`、`:agency_proto`、`:contract_test`、`:clippy`。传输的 `document` 是精确 JCS 字节；领域字段的类型在共享 Rust 模型中，不借 ProtoJSON 改写摘要。合同目前是本机 Unix 端口，不声称远程认证与加密已完成。

## 本包已固定的接口

- `prepare` 只接单；`lookup` 按原幂等键回读。`activate` 才执行。控制面先保存归属者到派工的映射。
- 客户在初次配对前保存随机租户凭据；重试证明持有该凭据才返回原租户端口，知道控制面 ID 不够。配对钥不授予服务停机权。业务请求没有租户选择字段；每个服务处理器固定到一个租户。
- `TicketClaims` 带派工、归属者语义代次、规格摘要、控制面写者代次、actor、权限、可选输入租约与过期时间。HMAC 只认证这些字节，不替代权限与租约判定。
- `Catalog.skills` 是 Agency 可供装载的精确引用。Skill 的 `verification=None` 表示 unknown；有直报回读报告且摘要一致才可作为 known 的依据。目录的声明不自动变成核验。
- Proposal 头不带主机、进程、会话或物理代次。每个输出单独携带 schema、摘要、候选引用与归属者、派工、授权引用。本版每份 Proposal 交一个精确输出；业务准入由第 5 包完成，写入型候选由第 6 包接线。
- `Trace.complete` 只表示当前页已追到持久游标；`gap=true` 时不能当作完整历史，也不表示执行成功。
- `Input.bytes`、`Proposal.output` 与交付材料的字节在 JSON 中使用 RFC 4648 标准带填充 Base64 字符串；内容摘要覆盖原始字节。公共字段支持任意字节，可信脚本成果帧只提供 UTF-8 文本。4 MiB 上限仍包括整个编码后的信封。
- `Client::call_outcome` 区分对端错误答复与没有答复。适配器只对原动作的前置校验拒绝结为 rejected；存储故障与回读拒绝不能证明原动作没发生。输入重试保留原票据和原 key，换票据是不同请求，不自动重发未知输入。
- 结果查询按提案 id 游标分页（`ResultQuery.after`，`ResultPage.complete`）。单份成果仍受传输信封限制；多份成果可以分次取回。

`Client` 缺省连接预算为 2 秒、单次 RPC 预算为 5 秒。进程可用 `HCTL2_AGENCY_REQUEST_TIMEOUT_MS` 声明请求预算，单位毫秒，非法值或零拒绝调用；调用方的 `with_request_timeout(Duration)` 优先于进程声明，克隆后配置不改变原客户端。`request_timeout()` 返回实际预算。预算交给 tonic 原生超时，不改变 Execution Spec 的截止或权限，不重发请求；超时仍返回 `CallFailure::NoReply`，由调用方回读原动作。

## 第 3、4、5 包共同接法

记录先 `Sealed::new`，接收后核对 `verify` 和实际交付摘要。权限、截止、预算与消费者改变就重建规格，不在活动派工里改字段。能力按效果声明，缺少要求就拒绝激活。脚本没有终端、工具直报或 OS 加固能力。

权威仍是 [Participant 约束](../../../docs/design/spec/participant.md)、[连接约束](../../../docs/design/spec/connections.md)。各包的任务说明分别在 [Agency](../../agency/README.md#第-3-包任务说明)、[Context](../context/README.md#第-4-包任务说明)、[Participant](../participant/README.md#第-5-包任务说明)。

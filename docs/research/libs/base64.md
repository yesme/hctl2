# base64 · Agency 字节字段

> 状态：采用 · 复核：2026-10-04 · #317 首轮修正

## 定位与证据

Agency 的规范 JSON 要携带任意字节。JSON 数字数组会把每个字节放大到 2–4 个字符，也会影响冻结摘要的长期口径。本机已核 Cargo 缓存中的 [base64 0.22.1 上游源码](https://github.com/marshallpierce/rust-base64/tree/v0.22.1)：`engine/general_purpose/mod.rs` 的 `STANDARD` 使用 RFC 4648 标准字母表与带填充配置，解码拒绝不规范填充及尾部位。docs.rs 本次访问失败，未拿它作新证据。

## 候选与决定建议

采用 **base64 =0.22.1**，已有 Cargo.lock 中同版 SDK；不手写编码器。字节字段在 JSON 中统一用带填充的标准 Base64 字符串，摘要仍覆盖解码后的原始字节；规范 JSON 摘要覆盖该固定编码。长度约为原字节的 4/3，不声称消除了 4 MiB 传输上限。

不采用 `serde_bytes`：它在 JSON 上仍可表现为数组，不能单独确定这里的字符串合同。不只用 UTF-8 文本：Bundle 与成果的共享类型需要表达非文本材料。可信脚本探针的成果帧仍是 UTF-8 字符串，不等于公共端口只支持文本。

许可证 MIT OR Apache-2.0。验证包括全部 256 种字节的往返、拒绝数字数组及非规范填充、原始字节摘要不变；Buck 与 Cargo 共用同一锁定版本。

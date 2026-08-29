# chat server 选型：Tuwunel 与 Continuwuity（限时验证）

> 类别：⑥ 机械后端与基础设施 · 证据编号：E-L4-MATRIX-HOMESERVER<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-l4-matrix-homeserver"></a>
## E-L4-MATRIX-HOMESERVER · chat server 选型（限时验证）

三类数据模型（v0.10.0）把 Chat Room 的消息 content 判给采用 Matrix 协议的 chat server。两个候选并列进入开工前限时验证（见[交付文档](../design/delivery.md#开工前限时验证)），均为 Rust 单二进制、采用 RocksDB 系嵌入式存储的 conduwuit 谱系：

- [Tuwunel `v1.9.0 / 5b366914`](https://github.com/matrix-construct/tuwunel/tree/5b3669144219d5d4c0774743c84191b476f1b54f)：conduwuit 原作者延续、全职维护；Apache-2.0。
- [Continuwuity](https://github.com/continuwuity/continuwuity)：conduwuit 社区延续、Matrix 基金会生态成员；Apache-2.0。

已拍板 **Tuwunel**（Continuwuity 记录在案备选）。理由：接口更 API 化、与 Synapse 参考实现兼容性更强；AppService 注册程序化而非房间内发命令。上游 `v1.9.0` 发布物只有 Linux；HCTL2 因而在自己的 GitHub Release 托管由锁定 commit、Rust 1.95.0 和明确 feature 集生成的 macOS arm64/x86_64 包，分别约 32.6/35.5 MiB，并在 `lock.json` 固定 URL 与 SHA-256。正常组包只下载、检查 Mach-O 并完成生命周期，不再编译源码；更新制品时才手动触发隔离的原生构建 workflow。Apple Silicon 的交叉构建只能生成候选，仍须由 Intel runner 验证同一制品。

角色：执行面独立服务器——采用为依赖、由 control 托管生命周期，不 vendor 源码；P0 必须固定实际存储后端及 build features，并验证 macOS 承载、低内存配置与 RocksDB/media 一致性备份。它们承载消息 content，不获得任何治理权威；HCTL 依赖的合同前提（事务 ID 幂等、单 homeserver 线性顺序）以验证结果为准。

# Context 组装框架

本 crate 只定第 4 包的类型和接口，还没有选材器、来源读取器或 CLI 实现。共享记录来自 `agency-proto::context`；不复制一套 Bundle 类型。Buck：`root//crates/context:context`、`:clippy`。

## 第 4 包任务说明

依据：[演示线开工书第 4 包](../../../.memo/design/p2-control-20260906/07-demo-kickoff.md#第-4-包--context-组装)、[Context 约束](../../../docs/design/spec/project.md#contextmemo-与-artifact)。

| 文件 | 要补什么 |
| --- | --- |
| `src/lib.rs` | 保留 `Sources::exact`、`Assembler::assemble`、`AssemblyRequest` 和 `Assembly` 作为公共接口 |
| `src/sources.rs` | Room、Task 评论与评审评论的来源适配；评审读取先留接口，第 6 包接上 |
| `src/assembler.rs` | 本地选材、排序、权限过滤、三档交付与计量；不调用模型、不生成纪要 |
| `src/records.rs` | Manifest / Bundle 的控制面保存与冻结引用，复用 Store 的材料与事务接口 |
| `apps/control/src/context.rs`、`apps/cli/src/context.rs` | `context show|preview` 接线，网络读取留事务外 |

Manifest 固定目的、范围、来源版本、策略、时效、覆盖、缺口、Skill、权限与预算。Bundle 固定消费者、渲染器、tokenizer、脱敏、压缩链与计量。`Sources::exact` 返回精确版本和字节；版本不符不能装成旧预览。没有可用 tokenizer 时如实表示未计量，不编 token 数字；若需要调整本包占位的计量字段，说明兼容改法。

必需离线材料不能退成代取。Pointer 交付是精确字节加执行侧相对文件名，不是生产者机器的目录；Agency 自己决定私有落点。代取的 grant 类型已留，真实通道在组装与派工接线时落实，未具备时不能把必需项发成 Recall。

失败用例落原生 Buck 测试：来源版本变化、权限与预算变化使旧预览失效；实际字节摘要不符；必需离线材料未交付；不同消费者不能混包；重复 Pointer 文件名拒绝。对照 CT-PROJECT、CT-CONNECTION 的 Context 行，不用「有摘要」替代「材料可用」。

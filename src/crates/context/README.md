# Context 组装（第 4 包）

本 crate 实现选材、组装、冻结与读取；共享记录来自 `agency-proto::context`，不复制一套 Bundle 类型。组装全程本地、机械：不调用模型、不生成纪要、不压缩（small-brain 是后续引擎的事）；未配置 tokenizer 时如实报未计量，不编 token 数。

## 模块

| 文件 | 职责 |
| --- | --- |
| `src/lib.rs` | 公共接口：`Sources::exact`、`Assembler::assemble`、`AssemblyRequest`、`Assembly`、`SourceKind`、`Selection` |
| `src/sources.rs` | Room（chat 冻结记录+已准入材料）、Task 评论线（冻结 Snapshot 记录）、平台评审评论线（第 6 包接线前返回类型化 `REVIEW_LINE_NOT_CONFIGURED`，不冒充空成功） |
| `src/assembler.rs` | 本地选材与排序（稳定内容在前、高频在后）、权限与预算门、三档交付（必用预算内内联；超限降为带字节副本的 pointer+分片建议，绝不静默丢弃；未达材料由组装器代取）、计量（无 tokenizer 为 None） |
| `src/records.rs` | Manifest/Bundle 作为治理记录追加保存（同键重放返回 `replayed:true`，不改写已冻结记录），冻结文档字节进材料库 |
| `apps/control/src/context_query.rs` | `context.preview`（读端组装，不保存）与 `context.show`（读冻结记录）查询 |
| `apps/cli`（main.rs） | `hctl2 context preview --input manifest.json`、`hctl2 context show PROJECT [--manifest-id ID] [--bundle-id ID]` |

Buck：`root//crates/context:{context,domain_test,clippy}`。

## CT 对照（第 4 包范围）

| CT 现行行（按内容定位） | 本包证据 | 未覆盖及原因 |
| --- | --- | --- |
| Context 可解释：条目可追溯到 Manifest 精确来源引用与 version/digest；Manifest 缺 selection-policy/freshness/coverage/known gaps/权限/预算任一项失败 | D：`manifest_completeness_is_enforced`（空 freshness 拒绝）、条目仅由 Manifest 来源构造（结构性）；Bundle.manifest 指向冻结摘要 | 「后续消息改写已冻结记录」由 records 的追加语义与 `saved_assemblies_are_append_only_and_replayable` 承担（代码有、D 已测重放）；索引删除重建是 chat 侧行为，归其包 |
| 必需材料未送达、摘要不符或只给定位符时拒绝派发 | D：`delivered_bytes_digest_mismatch_is_rejected`、`required_over_budget_degrades_to_pointer_with_copy_never_drops`（超限给副本指针+分片建议）；共享 `validate_delivery` 拒绝 required 走 Recall | 「预算内内联超限给副本」整链在派工（第 5 包）侧的最终核对未接 |
| 授权材料集合不授予整库读权；读他 Project 私有材料拒绝 | D：`permission_change_invalidates_the_preview`（权限集变化→旧预览失效）、非许可来源 `PERMISSION_DENIED`（assembler 门） | 跨 Project 材料读取的负例在 Store 作用域层已有拒绝，本包未单独造例（代码有、未测） |
| 来源版本变化使旧预览失效 | D：`source_version_change_invalidates_the_preview`（引用不可解析）、`store_sources_return_exact_bytes_or_stale`（记录移动→`SOURCE_VERSION_CHANGED`） | — |
| 预算变化使旧预览失效 | D：`budget_change_invalidates_the_preview` | — |
| 不同消费者独立 Bundle；共同条目相同不等于整包相同 | D：`bundles_are_per_consumer`（不同 consumer 不同 bundle id/digest） | 消费者间内容隔离的运行时执行归第 5 包 |
| Bundle 压缩条目缺 compressor/原文记录或压缩证据类内容拒绝 | 本包不压缩：`compression` 恒为空；类型上 `Compression` 强制模型+原文引用+摘要 | small-brain 接入后补压缩负例（代码有、未测——未配置时不产生压缩条目） |
| 压缩片段/纪要回源指针不是组装器赋予时拒绝 | 本包不生成纪要/压缩，指针仅由组装器构造（`pointer_name`） | 同上，随引擎包补例 |
| 平台评审评论线以精确 ChangeSet Revision 冻结；评论不当授权 | 接口已留：`SourceKind::ReviewComments` → `ReviewComments` 适配器返回 `REVIEW_LINE_NOT_CONFIGURED`（D：`review_comment_line_reports_not_configured`） | 真实评审线读取第 6 包接（按计划） |
| 归属者终态+准入窗口关闭后按保留策略丢弃正文 | `Bundle.retention` 字段携带策略；丢弃执行是保留期治理，非组装职责 | 归属者生命周期事件接线在派工/收口包 |
| token 计量：候选/实选/交付 | D：`metering_without_tokenizer_is_none`（无 tokenizer 全 None） | 配置 tokenizer 后的实数计量未测（无实现，如实标注） |
| 重复 Pointer 文件名拒绝 | D：`duplicate_pointer_names_rejected_by_delivery_validation`（共享 `validate_delivery`） | — |

「没有可用 tokenizer 时如实表示未计量」是占位字段上的兼容承诺：`candidate_tokens/selected_tokens/delivered_tokens` 保持 `Option`，配置 tokenizer 后填实数，不改字段形状。

D = `root//crates/context:domain_test`（12 项全绿）。

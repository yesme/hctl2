# Context 组装（第 4 包）

本 crate 实现选材、组装、冻结与读取；共享记录来自 `agency-proto::context`，不复制一套 Bundle 类型。组装全程本地、机械：不调用模型、不生成纪要、不压缩（small-brain 是后续引擎的事）；没有 tokenizer 实现，计量字段如实为 `None`，不编 token 数。

## 模块

| 文件 | 职责 |
| --- | --- |
| `src/lib.rs` | 公共接口：`Sources::exact`、`Assembler::assemble`、`AssemblyRequest`、`Assembly`、`SourceKind` |
| `src/sources.rs` | Room 线＝`chat_source_reference` 冻结记录＋已准入材料字节（chat 命令路径写入的真实种类）；Task 评论线＝冻结 `task_snapshot` 记录（跨作用域按 id 找，交付字节为其规范 JSON）；平台评审评论线第 6 包接线，按 kind 分派返回类型化 `REVIEW_LINE_NOT_CONFIGURED`。`select_room_manifest` 是机械选材入口：按 Project＋Room 对象 ID，从治理记录把该 Room 冻结的来源引用选进 Manifest（许可集来自策略点占位＝本项目全部已准入来源，不取调用方输入）；`permitted_source_ids` 即该占位。 |
| `src/assembler.rs` | 排序（稳定在前）；权限门（manifest 冻结的权限摘要 vs 策略点当前答案）与预算门（变化即 `PERMISSION_CHANGED`/`BUDGET_CHANGED`）；交付：预算内内联，超限降为**带精确字节副本的 pointer**，建议写进条目 `description`（共享类型无建议字段，不动共享类型），条目保持 `required=true`，绝不静默丢弃；`Entry.bytes_digest` 如实记录实际交付字节摘要（不与来源摘要互比——来源版本由适配器核，字节摘要供第 5 包派发核对）；指针相对名＝净化名＋id 摘要后缀（防碰撞）。 |
| `src/records.rs` | Manifest/Bundle 追加冻结：**按内容判重放**——记录在且摘要相同＝`replayed:true`；同 id 不同内容＝`CONTEXT_CONFLICT`；manifest 已在而 bundle 不在＝补存 bundle；bundle id 含消费者代次。 |
| `apps/control/src/context_query.rs` | `context.preview`（按 `project_id`＋`room_id` 选材组装，不保存；错误保留原 code 与 recovery_action）与 `context.show`。 |
| `apps/cli`（main.rs） | `hctl2 context preview --input ctx.json`（`{"project_id","room_id","budget"?}`）、`hctl2 context show PROJECT --manifest-id ID --bundle-id ID`。 |

Buck：`root//crates/context:{context,domain_test,clippy}`。

## CT 对照（第 4 包范围）

| CT 现行行 | 证据 | 未覆盖及原因 |
| --- | --- | --- |
| 可解释：条目追溯到 Manifest 来源引用；缺必备字段失败 | N（chat 命令路径建 Topic→按对象 ID 预览→条目读回来源正文字节）；D（completeness、条目构造结构性、重放不改写） | 索引删除重建归 chat 侧 |
| 必需材料未送达/摘要不符拒绝 | D（篡改副本被 `validate_delivery` 拒、超限降级不丢弃）；派发前最终核对归第 5 包 | — |
| 材料集合不授整库读权 | 许可集来自策略点占位（store 派生），非输入文件；跨 Project 读取由 Store 作用域拒绝 | 真实权限策略未实现——**代码有、未测**（占位明标，第 5 包接） |
| 来源版本变化→旧预览失效 | D（`StoreSources` 记录移动→`SOURCE_VERSION_CHANGED`） | — |
| 权限/预算变化→旧预览失效 | D | — |
| 不同消费者独立 Bundle | N（同 manifest 两消费者各自存、各自读回；代次不同不撞 id）；D（bundle id 含代次） | — |
| 同 id 不同内容 | N（`CONTEXT_CONFLICT`） | — |
| 压缩条目/回源指针 | 本包不产生压缩/纪要（`compression` 恒空；指针名仅组装器构造） | small-brain 接入后补例——代码有、未测 |
| 评审评论线精确冻结、不当授权 | D（store 适配器按 kind 分派返回 `REVIEW_LINE_NOT_CONFIGURED`） | 真实评审线第 6 包接（按计划） |
| 保留策略丢弃正文 | `retention` 字段携带 | 执行归收口包——代码有、未测 |
| token 计量 | D（无 tokenizer 实现，三字段恒 `None`） | tokenizer 实数未实现——如实标注 |
| 重复 Pointer 名拒绝 | D（共享 validate；名字加摘要后缀防碰撞） | — |

D = `root//crates/context:domain_test`；N = `tests/native.rs`（真实 Store，chat 命令路径）。

## 范围收窄（写明不做）

- **当前讨论窗口不在选材里**：约束选材顺序含「当前讨论窗口」，但这部分内容不在控制面存储里（在 chat server）；本包选材只认治理记录。第 5 包派工接 Room 时间线读取时补。
- **显式用户引用/Artifact/Memo/Skill 选材**未做：本包的机械路径只选 Room 冻结来源线＋（接口级）Task 快照；完整选材顺序随派工链补。
- **tokenizer**：无实现，计量如实 `None`；字段形状兼容后续填实数，不改共享类型。

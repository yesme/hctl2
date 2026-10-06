# Context 组装（第 4 包）

本 crate 提供本地选材、组装、冻结与读取。记录复用 `agency-proto::context`，没有第二套 Manifest / Bundle 类型。组装不调用模型、不生成纪要、不压缩。没有 tokenizer 时，计量字段为 `None`。

## 模块与接线

| 文件 | 职责 |
| --- | --- |
| `src/selection.rs` | `select_context(SelectionRequest)` 接收精确消费者、消费 Room、可选 Task 与预算。Topic 只选自己的已确认前情提要及其中的来源列表，不按来源所在 Room 搜集所有 Topic。Manifest ID 由完整冻结输入的规范摘要生成，新增来源或权限答案变化会得到新 ID。 |
| `src/sources.rs` | Room 线读取已准入材料。Task 线按 Project 内的 Task 定位 Repo、已接入的源、当前完整 Snapshot 与绑定卡片，只交付这张卡的整条评论线，不交付整个看板。来源版本变化返回 `SOURCE_VERSION_CHANGED`；评审评论线未接线时返回 `REVIEW_LINE_NOT_CONFIGURED`。 |
| `src/assembler.rs` | 校验冻结的权限摘要与预算，稳定内容在前。预算内内联，超限转为携带精确字节副本的 Pointer，分片建议在条目说明中；必需材料不静默丢弃，也不退为 Recall。指针使用安全相对名，不携带生产者目录。 |
| `src/records.rs` | 保存前核对封存摘要、交付字节、Manifest 引用、Project、策略与完整来源集合。正文先保存，准入和记录写入在同一事务中。同 ID 同内容重放，同 ID 异内容拒绝；读取时复核封存摘要与已准入材料。 |
| `apps/control/src/context_query.rs` | 只读 `context.preview` / `context.show`；RPC 外层 Query 白名单已接通。保留类型化错误与恢复动作。 |
| `apps/cli` | `hctl2 context preview --input ctx.json`；`hctl2 context show PROJECT --manifest-id ID --bundle-id ID`。 |

Buck：`root//crates/context:{context,domain_test,clippy}`。

## 第 5 包调用约定

`SelectionRequest.consumer` 是已有的 `Owner`：Project、`room_invocation` / `run_attempt`、ID、精确代次。派工端负责提供真实归属者；组装器不创建 Invocation，也不把 Room ID 冒充消费执行。

`room_id` 指消费上下文的 Room；`task_id` 是这个 Project 内显式选择的 Task。没有 Task 就不混入任务评论。Task 来源校验复用 `task::source`、`task::latest` 与 Project 的来源准入记录。Task 评论引用的 `revision` 保存规范 JSON：Task、Snapshot、来源准入与源绑定的精确引用、版本和摘要；`digest` 覆盖这些身份与评论。Snapshot 位置和摘要可直接检查，不只有一个无法追溯的合成摘要。

CLI 预览输入示例：

```json
{
  "project_id": "P",
  "room_id": "topic-1",
  "task_id": "T",
  "consumer": {"project": "P", "kind": "room_invocation", "id": "invocation-1", "generation": 1},
  "budget": 65536
}
```

`task_id`、`consumer`、`budget` 可省略。未指定消费者时只读预览使用 `preview` 标识，不签发授权、不创建消费执行；预算缺省为 65536。`select_room_manifest` 是同样的只读便利入口。保存由库接口 `save_assembly` 提供，CLI 预览不保存。

当前权限答案是 Project 已准入来源集合的占位，不是正式派工授权。Store 读取仍校验调用者的 Project 范围，不能靠输入的 Manifest 自授读权。第 5 包须接入真实权限策略和授权校验，再冻结 Execution Spec、交付并核对实际字节。

## CT 对照（本包范围）

| CT 现行条款 | 已验证 | 未实现或后续接线 |
| --- | --- | --- |
| 来源可解释、必备字段齐全 | D：缺字段拒绝；N：真实 chat 命令创建 Topic，选自己的前情提要与来源，另一 Topic 不混入；Task 评论保留精确 Snapshot 引用和摘要 | 在线当前窗口、显式 Artifact / Memo / Skill 的选材由第 5 包接入 |
| 必需材料交付、摘要核对 | D：必需 Recall、篡改副本、重新封存坏字节、错误 Manifest、重复条目替掉必需来源均拒绝；超限 Pointer 保留字节 | Agency 派发前最后核对由第 5 包接入 |
| 材料集合不授整库读权 | N：跨 Project 权限不足拒绝；同名源跨 Repo 不串读，只读取所选 Task 的卡片评论 | 正式权限策略未实现；占位明示 |
| 来源、权限或预算变化使旧预览失效 | D：来源版本、权限、预算变化；N：Task Snapshot 更新使旧引用失效，不完整 Snapshot 拒绝 | 在线读取变化由派工链接入 |
| 每个消费者独立冻结，旧记录不改写 | N：同 Manifest 两消费者分别保存与读取；同 ID 异内容拒绝；代次变化独立保存；新增 Topic 后再次预览能保存，旧 Manifest 仍可读 | — |
| CLI 查询可用、错误可观察 | C：真实 control RPC 读冻结包；preview 返回领域错误而非未知 Query；消费者 Project 不匹配拒绝；D：坏封存摘要、准入副本与记录不符、缺材料引用拒绝读回 | 有效选材路径由 N 覆盖；未运行真实 chat server |
| 评审评论线不当授权 | D：未接线返回类型化错误 | 精确平台评审评论读取由第 6 包接入 |
| 压缩来源与证据保护 | 本包不产生压缩条目 | 压缩引擎未实现，不声称已测 |
| 保留策略与丢弃事实 | Bundle 携带保留至归属者终态且准入窗口关闭的策略名 | 实际保留与清理未实现 |
| token 计量与 Pointer 名 | D：无 tokenizer 时计量为 `None`；Pointer 名碰撞拒绝 | 实数 token 计量未实现 |

D = `tests/domain.rs`；N = `tests/native.rs`，两者由 `root//crates/context:domain_test` 运行。C = `root//apps/cli:cli_test`。

## 范围收窄

独立 `context.preview` 仍不读在线窗口；没有确认提要、也没有显式 Task 时返回 `SOURCE_UNAVAILABLE`。第 5b 包的派工入口在 `apps/control/src/dispatch/context.rs` 读取本 Room 的在线窗口，复用本 crate 的 `select_context` 读取已确认提要及来源、显式 Task 评论，再用 `assemble_resolved` 复用交付、预算和封存逻辑。它校验精确来源集合、消费者权限答案和预算；来源适配器负责冻结版本与原文，不能靠改名把调用者提供的字节当成服务器事实。

派工只选择 `context.read` 范围内已明确的请求、当前 Room 和可选 Task，不调用只读预览的整 Project 占位权限。独立 `context.preview` 的占位仍未替换，不产生执行授权。额外 Memo / Artifact 选材、必需 Skill 原文取回仍未接；平台评审评论线留第 6 包。没有新增约束或共享对象。

# L2 · Governance & Orchestration — Workflow / Run / Gate

> Status: Normative · Draft v0.7.0<br>
> Primary reference: HCTL2-native semantic governance<br>
> Foundational predecessor: HCTL1 / yesme/hctl<br>
> Mechanical state backend: Conductor OSS<br>
> Parent: [四层设计规范](../README.md)

## 这一层为什么存在

L2 回答：**哪些自动施工已经获得授权，哪个逻辑责任应由谁尝试，技术失败是否可以换候选，以及什么 revision-bound evidence 才允许 Gate 通过。**

外部产品已经探索 DAG、agent delegation、retry、run/task/dispatch 和 human approval；因此不能笼统说“行业没有 workflow”。真正缺失的是一套成熟的、面向 Project 语义、绑定 revision/evidence 的治理层：TaskRevision 与 WorkflowRevision freeze、bounded Run Manifest、Obligation/Seat/Attempt、candidate fallback、quorum、regate 和正式 Receipt 能够共同工作。

HCTL2 不是从零开始：直接前代 HCTL1 / yesme/hctl 已把 Git-native Seat/claim/fencing、exact Verdict、quorum、Receipt 与 fail-closed reconcile 做成规范、Go kernel 和 executable corpus。仍然缺失的是把这组语义与 Project/Task revision、bounded Run、candidate fallback、Attempt、被动 Engine 和 runtime control 统一起来的完整产品治理层；这才是 HCTL2 的 missing piece，也是不能委托给 Chat、Task Board、Harness、terminal 或通用 Workflow Engine 的原生价值。

## 本层负责什么

L2 负责：

- typed WorkflowModel、WorkflowRevision、compiler 与 HCTL Profile；
- Approve Workflow / Start Run 的授权边界；
- Run Manifest、TaskRevision binding、repo base 与 policy freeze；
- Conductor external task → Obligation → Seat → Attempt 的治理映射；
- candidate selection、technical fallback、deadline 和 fencing；
- semantic Verdict、quorum、reject/rework/regate；
- Request/blocking scope 与 bounded human intervention；
- domain result journal、Receipt 与 completion admission；
- control/Engine/agentd/core 之间的 idempotency 与 reconciliation。

L2 不负责：

- 决定 Project 的目标或替用户采用 Task contract；
- 把 Task Board stage 当成 Workflow token；
- 让 Conductor 直接启动 Harness、写 Git、访问 secret 或发送领域通知；
- 让 Harness 自行选择 semantic completion；
- 拥有 PTY、process、terminal renderer 或 provider session；
- 自研通用 Workflow Engine。

## 两种权威：机械位置与语义治理

Conductor 保存 definition deployment、token、READY state、fork/join/switch/loop、timer、wait、retry 和 task history 的机械 truth。

HCTL control/core 保存 Project/Task/Run binding、effect authority、candidate set、Seat identity、quorum、revision、Verdict、Receipt 和 semantic completion admission。

```text
Conductor: “这个 external node 已 READY / IN_PROGRESS / COMPLETED”
HCTL2:    “它对应哪个承诺、谁可尝试、结果是否有效、是否可以 complete”
```

Workbench 与任何外部 Workflow View 都只能 query Conductor/HCTL projection。Start/Pause/Cancel 是独立 HCTL typed command；不能通过 UI 或 Conductor 运维页面直接 complete/fail/signal node。

## Workflow compiler 与 HCTL Profile

HCTL 不自创另一种通用 YAML，也不部署模型自由生成的 JSON 文本：

1. UI/Participant 形成 typed `WorkflowModel` proposal；
2. Rust compiler 用 JSON library 构造 definition；
3. canonical serialization；
4. JSON Schema validation；
5. HCTL Profile validation；
6. semantic validation：unique refs、dependency、join/loop、taskBindingSlot、effect restrictions；
7. 写入 Git并计算 digest；
8. 注册固定 Conductor definition version。

Phase 1 Profile 允许 SIMPLE external task、FORK_JOIN/JOIN、SWITCH、DO_WHILE、DYNAMIC_FORK、SUB_WORKFLOW、HUMAN/WAIT、NOOP 和经审计的纯数据 transform。

Canonical Workflow 禁止绕过 control 的 HTTP/JDBC/Kafka 领域副作用、arbitrary Script、native LLM/MCP task、Git/GitHub、agentd/Harness launch、external notification 和 secret write。Conductor 可能支持这些系统 task，但 HCTL Profile 不允许使用。

Git 中 canonical JSON + digest 是共享 definition truth；Conductor registry 是 deployed copy；Run 固定一个 version/digest，新 definition 不改变旧 Run。

## Approve Workflow 与 Start Run

Approve Workflow 确认施工图；Start Run 才授予实际资源和副作用 authority。Start Run 前必须：

- 选择 0..N precise TaskRevision；
- 显示任何 source divergence，并要求 adopt new revision、explicitly pin old snapshot 或 resolve conflict；
- 冻结 repo base SHA；
- 冻结 logical roles、required expertise、authorized candidate sets；
- 冻结 fallback、capability、permission、network/secret、budget 和 placement policy；
- 生成可检查 Run Manifest；
- 程序化 validate Workflow definition。

启动顺序使用 durable intent first：

1. SQLite transaction 创建 `Run(status=Starting) + Manifest + StartWorkflow outbox`；
2. dispatcher 幂等注册/确认 definition；
3. 以 `run_id` correlation 启动或查找 execution；
4. timeout/unknown outcome 先 query/reconcile，不盲目重复；
5. 回写 Conductor execution ID 并置 Running；
6. 只有 READY external task 出现后才懒创建 Obligation、Seat、Attempt、worktree 和 runtime。

查看或讨论 Workflow 不创建 runtime。

## External task 到 Obligation

Control poll 到 `(workflowExecutionId, conductorTaskId, node_ref, run inputs)` 后，以 `(run_id, conductor_task_id)` 幂等 find-or-create Obligation。Workflow input 不预先携带 runtime `obligation_id`，因为 loop/dynamic fork 会动态创建 task execution。

每个 HCTL external task execution 恰对应一个 Obligation；JOIN、WAIT、SWITCH 等 Engine control task 不对应 Obligation。一个 Obligation 拥有 1..N Seat；Seat 在真正 dispatch 前可以没有 Attempt。

普通 author/test node 通常一个 Seat。2-of-3 gate 有三个逻辑 voter Seat。Seat 冻结 role、subject revision、required capability、candidate set、policy 和 lease，具体 WorkerProfile/Harness/Skill render 在每个 Attempt 创建时冻结。

## Retry、candidate fallback 与 rework

| 机制 | Owner | 新 identity | 语义 |
| --- | --- | --- | --- |
| transport retry | adapter/agentd | 可留在 Attempt | 短暂连接恢复，不改变 logical execution |
| primary → backup | hctl2-control | 同 Seat 下新 Attempt | 仅 typed technical failure |
| Engine task retry | Conductor | 新 task execution → 新 Obligation | Engine deadline/retry 语义 |
| semantic reject → rework | Workflow + control/core | 新 subject revision、required Seats/Attempts | 有效业务结果，不是 worker failure |
| replan | user + compiler | 新 WorkflowRevision/Run | topology 或授权 envelope 变化 |

只有 auth/permission、rate limit/quota、network/transport、process/runtime lost、lease timeout 等 policy 允许的 typed technical outcome 才可能 fallback。Semantic reject 不换裁判。

Candidate fallback 流程：

1. 当前 Attempt 返回 typed technical outcome 或 lease timeout；
2. control fence generation；迟到 result 只进入 history；
3. 若 candidate policy、budget 和 Obligation remaining deadline 允许，在同 Seat 新建 Attempt；
4. Conductor external task 仍保持 IN_PROGRESS；
5. candidate exhausted 时创建 Request 或向 Engine 报告 terminal technical failure。

Adapter 必须维护 Engine task lease/deadline、control lease 和 safety margin。若 Engine 已 timeout/retry，旧 Obligation Superseded，所有旧 Seat/Attempt 被 fence；旧结果不能完成新 task。

## Verdict、quorum 与 regate

Gate 的 N 个 logical voters 由 HCTL control 创建为 N 个 Seat；Conductor 只看一个 external gate task：

1. 每个 Seat 绑定同一 immutable subject revision、review policy、logical actor、Context/Skill digest 与 candidate set；
2. Seats 可并行 dispatch；同一 Seat 的 backup Attempt 不增加票数；
3. duplicate、stale、unauthorized vote 不计数；
4. reducer 计算 accepted、rejected、changes_requested 或 quorum impossible；
5. policy 明确的 veto 或 aggregate reject 才进入 author rework；
6. 达到 outcome 后 fence/cancel 未完成 Attempts，迟到结果留 history；
7. control 先 durable commit aggregate Verdict/Receipt，再幂等 complete Conductor task。

Author rework 产生新 ChangeSetRevision/ArtifactRevision，TaskRevision 通常不变。所有 required old Verdict 因 subject digest mismatch stale，并创建新 review round/Seats。只有 scope、acceptance 等 Task contract 改变时才需要新 TaskRevision 与 replacement Run。

## Human Request 与 blocking scope

Run 需要输入时，control 创建 durable Request 并标注精确 blocking scope；无关子图继续。简单 schema 在卡片中回答，开放式论述回 L4 Project Room，多人复杂商议升级 Scoped Room，secret 使用 secure prompt，交互 shell 进入 L1 attach。

回答必须先形成 preview，显示 object/revision/Run scope。Authorized actor 确认后，control/core 签发 Receipt 或 applied command，再 resolve Request 并 signal/complete Engine。普通 Room reply 不会推进 Run。

## Completion 与证据回流

L1 只返回 `ResultProposal + EvidenceRefs + observed execution outcome`。L2 验证：

- Attempt generation、Seat/Obligation identity 与 lease；
- subject revision、Context/Skill/Capability digests；
- required test/diff/Artifact/SCM evidence；
- actor/role separation 与 review policy；
- duplicate/stale/late result；
- aggregate gate/quorum policy。

只有验证通过才形成 Verdict/Receipt，并 complete Engine node。Run terminal state 再交给 L3 重新做 Task acceptance/completion admission；Run Completed 不自动完成 Task。

## WorkflowEngineAdapter

Domain 不把 Conductor-specific ID 变成公共对象。Adapter 提供：

- validate/register definition；
- start/cancel/pause execution；
- poll external work；
- complete/fail/signal；
- query snapshot/history；
- health/version/migration。

Mutation methods 只供 hctl2-control 内部使用。Every schedule/poll/complete/fail/signal 使用 idempotency key、durable inbox/outbox、expected revision、fence、result journal 和 startup reconciliation。

Control 必须先持久化 HCTL result/outbox 再调用 Engine complete；崩溃后重放 complete，而不是重做 Harness effect。

## Workbench 原生交互

Workbench 提供：

- Workflow proposal form/diff 与 Trigger Preview；
- Approve Workflow、Start/Pause/Cancel typed commands；
- frozen topology + LayoutCache + dynamic RunOverlay；
- node/Obligation/Seat/Attempt progressive Inspector；
- candidate、deadline、vote、stale/fence、Receipt 和 Request visibility；
- low-noise milestone projection 到 Room/Task。

Run Graph 使用 React Flow + Dagre，`nodesDraggable=false`、`nodesConnectable=false`。Status update 只 patch overlay，不 relayout；编辑发生在 proposal/form/diff，不在运行图直接拖边。

## 没有 Workbench 时如何降级

- `hctl2-control + Conductor + agentd` 按 durable state headless 继续；关闭 UI 不 pause 或 cancel Run。
- CLI/API/status card 可以 query Run、node、Obligation、Seat、Attempt、Request、evidence 和 trace。
- Start/Pause/Cancel/answer Request 必须调用同一 HCTL typed command service；没有 equivalent preview 的高风险 command 应安全拒绝。
- Conductor UI 若启用始终只读，不能成为 emergency mutation path。
- Blocking Request 保持 durable；无已交付 typed client 时只暂停受影响 scope，不猜默认答案。
- Workbench 恢复后由 Conductor snapshot + HCTL ledger + agentd/core observation 对账重建 overlay，不从旧 graph cache 恢复 truth。

L2 天然应可 headless 运转；Workbench 的损失主要是图形化解释和复杂授权体验，而不是治理正确性。

## 精选参考

### Anchor：HCTL2-native semantic governance

没有外部项目同时提供 Project/Task revision freeze、bounded Run authority、Obligation/Seat/Attempt、typed candidate fallback、quorum、regate、revision-bound Verdict/Receipt 和 Task semantic completion。HCTL2 因此在本层原生定义 contract，而不是伪造一个 donor。

现有 experimental Run/Task/Dispatch、cron-addressed message 和 fixed Task pipeline 证明了一些相邻机制可行，但都没有同时形成 project-semantic revision/evidence governance。具体负证据留在 Radar，不把这些项目重复列为 L2 reference，也不允许其 runtime/pipeline 成为第二 Workflow truth。

### Foundational lineage：HCTL1 / yesme/hctl

HCTL1 是 L2 语义内核的直接前代和可执行谱系证据。HCTL2 保留它的 revision/evidence、claim/fencing、quorum、Receipt 与 level-triggered reconcile 思路，但重新定义 Seat 和 Obligation，并把 Run、candidate fallback、Attempt 与 effect authority 扩展到 `SQLite + Conductor + agentd` control plane。

HCTL1 的 `Seat = harness × model`、per-seat Git refs、static assignment obligation、PR-as-atom、single merge coordinator 和 Git-only operational truth 都不能直接成为 HCTL2 public contract。版本、测试语料与精确差异见 [E-L2-HCTL1](../references/implementation-evidence.md#e-l2-hctl1)。

### Mechanical backend：Conductor

Conductor 的独特价值是 external worker、READY/wait/timer/retry/history 与 effect execution 可以分离。HCTL 通过 WorkflowEngineAdapter 采用它，但不把 candidate、Harness、Git、quorum、Receipt 或 domain effect 委托给 Engine。详见 [E-L2-CONDUCTOR](../references/implementation-evidence.md#e-l2-conductor)。

### Adjacent implementation evidence：ZeroClaw SOP

ZeroClaw SOP 只补 admission/backpressure、revision-scoped human approval/quorum、restart restore 与 fail-closed policy 的实现/失败样本；它不定义 HCTL WorkflowRevision、Start authority、Verdict、Receipt 或 domain truth。其 persistence 文档与实现默认值仍不一致，初始化失败还会降级到 process-local memory，因此更不能成为第二个 governance kernel。详见 [E-L2-ZEROCLAW](../references/implementation-evidence.md#e-l2-zeroclaw)。

Dagu 只作为 runner-centric 选型对照；React Flow/Dagre 是 visualization primitives，不是 governance donor。

## Failure 与 contract tests

- compiler/schema/profile/semantic validation 拒绝 hidden effect 和 malformed join/loop；
- Start Run unknown outcome 通过 correlation/reconcile 不重复 execution；
- poll/complete/signal duplicate 与 control crash 可重放；
- timeout/429/runtime-lost 在同 Seat fallback，semantic reject 不 fallback；
- Engine retry 创建新 Obligation，旧 Attempt late result 无效；
- 2-of-3 vote、same-seat backup no extra vote、veto、quorum impossible；
- reject → new subject revision → all required regate；
- stale/unauthorized/duplicate Verdict 不计数；
- stale approval prompt、wrong revision、missing approval policy/adapter 与 persistence downgrade fail closed；
- Request 只阻塞声明 scope，普通 Room reply 不 signal Engine；
- 在任意两个步骤间重启 Workbench/control/Conductor/agentd，无重复 effect；
- 任何 UI/CLI 绕过 HCTL 直接 mutate Conductor 的尝试被拒绝或检测为 divergence。

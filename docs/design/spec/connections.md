# 四模块的端到端连接

> 状态：规范性合同 · 草案 v0.15.3<br>
> 本文是 Project、Task、Run、Agent 之间连接合同的唯一权威。它不是第五个领域模块：连接的两端仍由对应模块合同（本目录）与[设计正文](../README.md)定义，共享命令、适配器与恢复机制见[系统边界](./system.md)。

## 连接模型

连接不是一份可独立漂移的共享状态，也不需要 `Handoff` 聚合。每条连接都由“目标模块的类型化命令 + 来源模块的不可变引用”组成：

1. 来源模块只能提供稳定 ID、Revision digest、状态版本、来源和已获授权的范围；不能直接写目标模块。
2. 目标模块在自己的命令准入中校验来源引用、当前版本、actor、权限和幂等键，并拥有新产生的状态。
3. 目标状态、来源关联、幂等结果和必要 outbox 由唯一 control 在同一本用户级 metadata 账本的一个事务中提交；跨 Project、Repo Instance 或模块都不得拆成 clone 本地事务再拼接。
4. 目标只以稳定引用和有序事件返回结果；来源和场景可以投影它们，但不能复制一套状态机。
5. 涉及 Engine、Agency、Git/SCM 或第三方平台时，提交本地意图先于外部动作；ACK 不确定时按稳定关联键回读。

连接中的引用至少包含 kind + stable_id + revision_digest 或 state_version，并携带所属 Repo/Project、producer 和适用的绑定版本。`current`、显示名、外部 ID、文件路径或界面选择不能替代精确引用。这是字段约束，不是新的持久领域对象。

## 连接图

```mermaid
flowchart LR
    P["Project<br/>Chat Room"] -->|采纳 Task 契约| T["Task<br/>Kanban"]
    T -->|「启动 Run」命令 + Run Manifest| R["Run<br/>Workflow"]
    P -->|无 Task 的「启动 Run」命令| R
    P -->|Room Invocation + Execution Spec| H["Agent<br/>Terminal"]
    R -->|Attempt + Execution Spec| H
    H -->|Result Proposal + Revision / Evidence| P
    H -->|Result Proposal + Revision / Evidence| R
    R -->|Run ref + Verdict / Receipt；正常完成可提交 Task 命令| T
    T -->|里程碑与稳定引用| P
```

Project → Agent 是无 Run 的显式短路；Agent → Task 不存在原始状态通道，只有经过校验的 Revision、Evidence、Verdict 或 Receipt 才能进入 Task 验收。

## 连接合同总表

| 方向 | 耐久输入 | 目标准入与提交 | 恢复依据 |
| --- | --- | --- | --- |
| Project → Task | Project/version、来源 Message/Artifact/Memo/Request refs、可选拟议 Task 契约及 digest、Repo Board/Project group anchor | Task 的「创建 Task」命令固定 immutable project_id 并持久化后端 create outbox；携带初始契约时才另写正文 outbox，后续「采纳契约」准入 Task Revision | command/idempotency/correlation key → 同一 Task、外部卡与可选 Task Revision ref |
| Project / Task → Run | Project/version、可选精确 Task Revision、Workflow/Deployment refs、repo baseline、根 Context Manifest、Participant/Role/Skill、候选、权限、预算和 Gate | Run 命令原子写 Run Manifest、Task Run claim、Run 账本和 Engine start outbox | run id + manifest digest → Engine Execution Binding/readback |
| Project → Agent | Room Invocation + Execution Spec | Project 先持久化调用授权，Agent 模块再预留、绑定和激活运行时 | invocation id + invocation_version + Execution Spec digest |
| Run → Agent | Attempt + Execution Spec | Run 先持久化派发授权，Agent 模块再预留、绑定和激活运行时 | attempt id + attempt_generation + Execution Spec digest |
| Agent → Project/Run | Result Proposal、逐输出的 owner/runtime/site/backend generations、Revision/Evidence refs | owner 模块去重并逐项校验身份、代次、Context Bundle、权限、写租约和输出 schema 后准入 | proposal id + producer sequence + owner/spec digest；迟到结果只留历史 |
| Human scene / Run reducer → Agent | 「合入 ChangeSet」命令、精确 ChangeSet Revision/target/evidence refs | Agent 模块准入授权并持久化 intent/outbox，工具箱执行与回读；Integration Receipt 返回发起模块作证据 | intent id + expected target head → 唯一 Receipt；结果未知不重投 |
| Human Kanban / Run reducer → Task | human provenance，或正常完成 Run ref；被冻结的 Task Revision ref、Revision/Evidence/Verdict/Receipt refs | human actor 或 task-bound Run reducer 提交同一个「完成 Task」命令；Task 按当前验收合同独立校验 | 「完成 Task」命令 id → Task Completion Receipt；Harness 只提供证据 |
| Task/Run/Agent → Project | source ref、event id/sequence、版本、敏感级别 | Project 只建低噪声投影；Memo/Artifact 仍需 Project 命令发布 | source event cursor，可从源账本重建 |

## Project → Task：从讨论到承诺

Chat Room 可以生成 Task 提炼提案的预览，但预览不是第二个 Task。确认时，「创建 Task」命令或「采纳契约」命令必须冻结：

- `project_id` 与预期 Project version；
- 来源 Message、Artifact、Memo、Request 的精确引用；
- 标题、预期结果、验收合同、角色/能力和可选外部来源绑定；
- 规范化 proposal digest、actor/permission 与 idempotency key。

Task 模块以 CAS 校验活跃 Project 和可选当前 Task Revision。「创建 Task」固定 immutable project_id 与该 Project 的 Board group anchor，先提交 Task identity、必需的后端 outbox 和稳定 correlation key；只有携带已预览初始契约时才同时写 Revision admission intent 与 Git 正文 outbox。ACK 未知按同一 key 分别回读适用的正文和外部卡，不能重试成第二个 Task/卡片。「采纳契约」在工具箱回读 Git 正文后准入不可变 Task Revision 并返回精确引用。反向的 content-first 卡片只有在唯一 Project group 下才由 reconcile 原子 claim 为无契约 Task，完整恢复合同见 [Task 模块](./task.md#契约与来源)。Room 中继续编辑或删除显示内容不会改写已采纳 Revision；普通消息、总结、父分组实体和拖放都不能创建或 reparent Task。

## Project / Task → Run：授权自动施工

批准 Workflow 只确认施工图；「启动 Run」命令才建立自动施工连接。Project 是必需且活跃的授权来源，Task Revision 是第一阶段 0..1 个可选绑定；Run Manifest 的冻结清单见[Run 合同](./run.md#workflow-与-run-授权)。

control 在一个用户级账本事务中写 Run、Manifest、幂等结果、可选 Task Run claim 和 Engine start outbox。外部执行实例用 `run_id + manifest_digest` 作为关联键；commit 后崩溃或 ACK 丢失时先回读，不能再启动第二个 execution。若 Project/Task/Workflow 在提交前已不匹配预期版本、Project 已归档或 Task 已有 `active | completion_pending` claim，命令拒绝；提交后发生的上游更新不改写活动 Run，只能影响新 Run 或触发显式替代。

## Project / Run → Agent：从授权到物理执行

两条入口共用同一个派发协议，但保留不同 owner：

- Project 入口先持久化 [Project 模块定义的](./project.md#room-invocation) Room Invocation 与其 Execution Spec；`repo_scope` 永远只读。它没有自动候选切换或 Gate。
- Run 入口先持久化 [Run 模块定义的](./run.md#从节点到结果) Attempt 与其 Execution Spec；候选、Seat 和语义归约仍由 Run 拥有。

两条入口共用同一份派发冻结记录 Execution Spec（票据，owner = Room Invocation \| Attempt）。它至少固定：

```text
execution owner stable ref + invocation_version | attempt_generation
+ root Context Manifest ref + digest
+ consumer Context Bundle ref + digest
+ exact Participant revision
+ Project Role Binding ref + version + digest（repo_scope 可无）
+ required/optional Skill refs + digests
+ Worker Profile revision
+ Harness/Runtime Resolved Port Binding + 接入方式与降级能力
+ terminal input policy（managed_single_writer | native_interactive_allowed；无 Terminal 时省略）
+ repo_id/base + 可选 selected Repo Instance ref / placement constraints
+ 能力与权限摘要
+ 预算与截止
+ 可选 ChangeSet / Write Lease 规则
+ 可选执行加固声明（来自 Worker Profile：OS 沙箱、凭据代用范围、网络目的地与工具接口白名单）
+ spec digest 与幂等键
```

owner 特有字段各自补充：Room Invocation 侧固定 scope（`repo_scope | project_scope`）、`invocation_version` 与 human 批准建议时的 lineage 字段；Attempt 侧固定 attempt/seat/run 身份与 `attempt_generation`。Participant、Role Binding、Skill 与 Worker Profile 分别回答“谁”“在该 Project 以什么职责/上限”“带哪些方法”“用什么物理执行配置”，不能用其中一个替代另一个。两侧不各建一份“执行规格”。

外部运行时的启动顺序固定为：

1. owner 模块提交 Execution Spec 与 dispatch outbox；此时只有 `invocation_version | attempt_generation`，不得预填 runtime identity；
2. Agency adapter 校验当前 control/site/binding generation，再请求选定的 Agency 进行无副作用预留，并返回实际能力、物理目标、Execution Runtime ID 与新的 `runtime_generation`；实际能力缺任一 Execution Spec 声明的加固项时，control 不进入下一步，以 typed rejection 列出缺项；Agency 本身不能回显的 fence 必须记录为未生效；
3. control 在用户级账本事务记录 owner 到 Execution Runtime 的精确映射、适用的 Write Lease 和 activate outbox；
4. outbox 同时携带 owner version/generation、runtime generation、control writer generation、site generation 与 Agency binding owner generation；adapter 按完整 tuple 再校验后调用 Agency 并回读。声明栅栏回显的 Agency 拒绝旧代次、旧租约和重复激活；未声明栅栏回显的 Agency 只在 HCTL 入口校验，绕过入口的动作按低信任处理。

前三类含义不可混写：Invocation/Attempt version/generation 是语义 owner 身份，runtime generation 是一次物理执行身份，control/site/backend generations 是防旧进程写入的基础设施 fence。Participant revision、binding revision、producer sequence 和 content cursor 都不是这三类中的任一种。

如果冻结的端口明确是受信任的纯进程内同步调用，Execution Spec 必须写 `execution_mode = in_process`，可以没有 Repo Instance、Runtime/Terminal、runtime/site/backend generations 或 lease；其 Result Proposal 改为固定 owner version/generation、control writer generation、Extension/Resolved Port Binding、spec/bundle digest 和 producer sequence，且不能提交 ChangeSet 或声称物理隔离/attach。除此之外不得省略物理 tuple。dispatch/activate ACK 只证明执行已被接受，不证明产生了语义结果。

## Agent → Project / Run：结果准入

Result Proposal 使用 [Agent 模块定义的字段合同](./agent.md#运行时与观测)，并精确引用本次连接的 Execution Spec；连接本身不再维护一份可变结果状态。

control inbox 先按 proposal ID + producer sequence + owner 去重，再逐输出校验：owner 仍接受结果，`invocation_version | attempt_generation`、control writer generation 与适用的 runtime/site/backend fence、spec/bundle/binding digest 和 lease 全部一致，写入来自获准 ChangeSet，输出属于声明范围，证据可回读且权限没有扩大。每项都携带自己的完整 producer tuple；`in_process` 只能使用上段的缩减 tuple，不能把一个合格项的代次套给另一个旧项。通过后：

- Room Invocation 的结果由 Project 记录并投影到 Chat Room；
- Attempt 的结果由 Run 归约为 Seat 结果、Verdict 或 Receipt；
- Task 不消费 Harness 的进程状态、自述、终端屏幕或未经准入的 Proposal。

任一旧代次、被取消/替代 owner 或不匹配 spec/bundle 的结果只保留审计记录，不能推进 Project、Run 或 Task。

## Human Kanban / Run reducer → Task → Project：验收与回流

无 Run 路径中，有权 human actor 在 Kanban 预览精确 ChangeSet Revision/Artifact Revision、ReviewSubjectRef 和测试/SCM 证据后提交「完成 Task」命令，不生成 Run 专属的 Gate Receipt；验收合同要求内部独立 Gate 时，Task 先授权 Run。有 Run 路径中，Run 返回冻结的 Task Revision、终止原因及 Verdict/Receipt/subject refs，并按 completion_pending 机制提交同一命令。两条获准来源与 Task 独立验收规则见[Task 写入合同](./task.md#写入合同)；Task 拒绝自动命令时 Run 保持完成，Task 保持开放并显示需要关注。

Task、Run 和 Agent 以有序领域事件向 Project 返回里程碑。事件携带 source module、稳定引用、event ID/sequence、版本和敏感级别；Project Room 只显示 Request、失败、已验证 Task、Artifact 就绪等低噪声投影。发布 Memo/Artifact 或归档 Project 仍需 Project 自己的类型化命令，不能由投影反向触发。

## 跨模块 Request 回路

Request 由 Project 模块保存，但可以阻塞 Task 待办、Run 中的 Attempt/Seat/Obligation，或直接 Room Invocation。创建时固定 owner_ref + affected_revision_ref + blocked_scope + owner state_version（Attempt 另带 attempt_generation，Room Invocation 使用 invocation_version）+ input_schema + required actor/role + permission + deadline/default policy + dedupe root；不得写一个无法判定属于哪层的裸 `generation`。被阻塞模块只保存 request_id 和自己的阻塞状态，不复制 Request lifecycle。Agent 模块只执行所属 Attempt/Room Invocation 的物理等待，不另造语义 blocker。

「解决 Request」命令固定 request/expected version、resolution digest、actor/delegation 和 idempotency key。对需要恢复执行的 Request，control 在同一用户级 metadata 账本事务 CAS Project Request 与来源 blocker 的精确版本，并提交解决结果及唯一 signal/delivery outbox；Project 或来源模块都不能在事务外再次 signal。接收方只接受匹配 owner state version、适用 Attempt/runtime/fence generations 和 binding 的投递，ACK/观测后才由来源模块推进 blocker。普通 Room 回复不能解决 Request，也不能直接完成 Engine 节点。目标已失效时安全拒绝或保留为过期历史。

Deadline 到达以同样的版本 CAS 写已过期，但不伪造答案，也不产生 Task terminal 命令。它只把冻结动作投回精确 owner：Task/Project 的待办动作失败或放弃并保留 Task lifecycle；Run owner 按 [Attempt/Seat/Obligation 的 fail/cancel 规则](./run.md#request重试与-gate)结束；直接 Room Invocation 的 `fail|cancel` 分别进入失败|已取消，并撤销其输入/写租约；Agent 模块本身没有独立语义终态，只执行所属 Attempt/Room Invocation 的结束动作。任何分支都不得投给替代 execution 或留下活动 Seat/Attempt。

## 版本、权限与替代

端到端可追溯链固定为：

```text
Project sources → Task Revision → Run Manifest → Execution Spec → Result Proposal
Project sources → Run Manifest（0 Task）→ Execution Spec → Result Proposal
Project sources → project_scope Execution Spec → Result Proposal
Repo sources    → repo_scope Execution Spec（只读）→ Result Proposal

获准 Result Proposal → Revision / Evidence → ReviewSubjectRef / Verdict / Receipt
Task 路径的验收证据 → Task Completion Receipt
```

每一步保存上一步的 ID + digest/version；current pointer 只用于预览，不能替代历史引用。上游版本变化不改写已接受的下游连接：提交前漂移则 CAS 拒绝，提交后由冻结合同继续执行到终态，新的顶层授权使用新版本；范围、权限、候选或验收含义变化需要显式替代，而不是原地修补。

权限只能逐级缩小：actor/Project role → Run Manifest（有 Run 时）→ Execution Spec → Agency/adapter envelope。任何下游都不能扩展网络、secret、Git、任务源、Engine 或终端输入范围；需要扩权时回到拥有该权限的上游重新预览和授权。

## 失败与恢复

命令幂等、outbox/inbox、ACK 回读、writer/backend generation 和租约恢复算法只由[系统边界](./system.md#命令与跨服务正确性)定义。本节只规定连接恢复后四模块可观察到的结果：

| 失败点 | 连接语义 |
| --- | --- |
| 目标事务提交前来源已变化 | CAS 拒绝，不创建下游事实 |
| 目标已提交、调用方未收到结果 | 恢复后返回同一目标引用，不出现第二个下游对象 |
| owner 身份可证明但外部结果仍未知 | 连接保持待启动/需要关注，来源不会被伪装成已交接 |
| owner/runtime identity、lease 或任一适用 fence generation 无法证明 | Attempt 与 Room Invocation 都进入丢失；同一事务撤销输入/写租约并提交旧 runtime 的 stop/fence outbox，迟到流与结果只留审计，Retry 使用新 owner、Execution Spec 与 runtime generation。此行是执行身份丢失处理规则的唯一定义，模块合同引用而不复述 |
| owner 取消或被替代 | 停止新派发，撤销写入/输入权并等待物理执行静默；迟到结果只留历史 |
| chat server 不可用 | 不依赖新消息/成员/cursor 的 metadata 命令可继续；需要 fresh chat readback 的准入拒绝，聊天入口显示重同步中 |
| 已绑定房间被开启端到端加密 | 聊天入口显示需要关注，已冻结引用与 digest 不受影响；可继续/拒绝与换绑恢复规则见[Room 与消息](./project.md#room-与消息) |
| 任务后端不可用 | 已冻结且策略不要求 fresh source 的 metadata 命令可继续；需要 placement/drift/head/cursor 的 Create/Adopt/Start/Complete/Move 拒绝，看板不显示假成功 |
| Workflow Engine 不可用 | 已冻结的本地事实继续存在；Run 的完成与评审只依据账本推进，Engine Execution Binding 标为分歧待对账 |
| harness / Agency 不可用 | 执行安全暂停或按代次结束，不冒充成功 |
| 其他外部适配器不可用 | 已冻结的本地事实继续存在；连接显示待启动/需要关注或安全暂停 |
| 场景投影丢失 | 从四模块账本和 source event cursor 重建，不从外部界面反推事实 |

系统对账完成前，各模块都不得表现为已完成交接。连接需要的新尝试或替代执行必须拥有新的 owner version/generation、Execution Spec 与 runtime generation；不能复活旧 owner。

## 场景与第三方适配器

Workbench 与第三方 Chat/Kanban/Workflow/Terminal 平台都通过上述目标命令、投影和事件编排连接；适配器只使用目标模块已有的连接。各模块分别声明可接受的 provider 动作：Chat 的普通消息只作 content，显式结构化动作才可能成为命令请求；Task 允许满足来源信封的 Done 产生完成请求；Run 的用户输入和 Agent 结果先进入 control，持久化后再由 outbox 推动 Dagu，Dagu 原生 mutation 只形成分歧；Agent 按 Execution Spec 声明的保证等级接纳原生终端输入。能力不足时隐藏动作、保留待处理请求或安全拒绝。动作分类见[系统合同](./system.md#客户端动作与-provider-事件)。

Workbench 的跨场景卡片和 deep link 只携带 stable ref 与可重建 projection；选择、焦点、展开状态和窗口布局都是客户端状态。用户从 Chat Room 跳到 Task、从 Kanban 打开 Run、从 Workflow 连接 Terminal 时，动作仍路由到目标模块的 Query/Preview/Submit；第三方客户端遵守同一规则。

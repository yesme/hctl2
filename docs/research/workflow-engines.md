# 工作流引擎：Dagu 与七个候选对照

> 类别：⑥ 机械后端与基础设施 · 证据编号：E-L2-DAGU<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-l2-dagu"></a>
## E-L2-DAGU · Dagu 机械状态后端与 workflow 候选复审

### 当前决定

第一阶段 workflow engine 拍板 **Dagu**，固定审阅基线为 [`v2.15.1 / 532c5129`](https://github.com/dagucloud/dagu/tree/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a)（2026-08-22，[GPL-3.0-or-later](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/LICENSE)）。判据已经按讨论弱化为：Workflow 的 HCTL JSON 必须是声明式事实源，可以机械执行 schema/Profile/引用/环等 lint；**不要求**对任意 loop 证明终止。运行时再以冻结预算、deadline、取消和替代规则收口。

Dagu 胜出的原因很窄：声明式 YAML、单二进制、文件系统持久化，`start-all` 在[一个进程内启动服务](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/cmd/startall.go#L94-L99)，比需要 JVM 或数据库/队列组合的候选更贴合单机第一阶段；同时，无进程的 [`human.task`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/runtime/runner.go#L766-L798) 会进入等待态，可经[完成 API](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/service/frontend/api/v1/humantasks.go#L78-L126)恢复，足以作为 control 驱动的外部检查点。它不是纯被驱动引擎：普通 step 会直接[构造 executor](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/runtime/node.go#L823-L827)并[自行执行](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/runtime/node.go#L265-L269)；HCTL Profile 因此只准依赖/条件/等待结构和 `human.task`，拒绝 command/script/action/HTTP/agent/Harness。

Workflow Revision 仍是 HCTL 规范化 JSON；Dagu YAML 只是固定 compiler/adapter 生成的 Engine Deployment。lint 分两层：HCTL 工具箱先验证完整 schema、Profile、引用与图结构，再调用 Dagu [`validate`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/cmd/validate.go#L25-L47)检查生成物。不能只相信后者：公开 `DAG.Validate` 只检查[名称和缺失依赖](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/ir/dag.go#L448-L475)，环是在[执行计划构建](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/runtime/plan.go#L287-L409)时才被拒绝，所以 HCTL 图 lint 必须更早拒绝并以负例锁定。

### 阻断性 P0 风险

`human.task` 完成服务内部有 CAS/幂等保护并把恢复写入队列（[completion](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/humantask/completion.go#L18-L113)、[resume](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/humantask/resume.go#L43-L92)），但公开请求只给 DAG、run 和 step，不能携带调用者预期的 engine attempt generation。adapter 必须在 ACK 未知、retry/repeat、重启和迟到完成的压力测试中证明 readback/fence 不会让旧请求推进新检查点；做不到就阻断 B4 并重开选型。inline spec start API 已存在（[源码](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/service/frontend/api/v1/dagruns.go#L163-L253)），但固定定义摘要、恢复后的 binding 对账和备份一致性仍由 HCTL adapter/control 负责。Dagu 不选择 Harness、不创建 Seat/Attempt、不解释语义拒绝、不计算 quorum、不签发 Receipt，也不直接写 Git 或外部系统。

### 本轮源码复审的全部 workflow 候选

以下是本轮实际审阅过的集合，不冒充整个市场清单；“stepfunlocal”按 AWS Step Functions Local 理解。结论基于固定源码/官方实现，而不是 README 特性表。

| 候选 | 源码层事实 | 结论 |
| --- | --- | --- |
| **Dagu** | 单进程/文件持久化；普通 step 自驱动，但 `human.task` 可作被动检查点；验证器与 completion fencing 的缺口如上 | **采用**；最小运维面，受严格 HCTL Profile 和 P0 fencing 门禁约束 |
| **Conductor OSS** | 当前 `v3.32.1` 已把 [SQLite 设为默认 DB/queue/index](https://github.com/conductor-oss/conductor/blob/v3.32.1/server/src/main/resources/application.properties#L14-L24)，旧评估“只支持 Redis/Postgres/MySQL”已经过时；其[逐 task poll/update API](https://github.com/conductor-oss/conductor/blob/b54f0d4ee546c1053367e1c14405c5396c17bfb1/rest/src/main/java/com/netflix/conductor/rest/controllers/TaskResource.java#L66-L112)比 Dagu 更原生地被驱动，但 [JVM server 分发](https://github.com/conductor-oss/conductor/blob/v3.32.1/server/build.gradle#L42-L79)仍重；loop validator 只查[条件与 body 存在](https://github.com/conductor-oss/conductor/blob/b54f0d4ee546c1053367e1c14405c5396c17bfb1/core/src/main/java/com/netflix/conductor/validations/WorkflowTaskTypeConstraint.java#L260-L284)，runtime 可[持续迭代](https://github.com/conductor-oss/conductor/blob/b54f0d4ee546c1053367e1c14405c5396c17bfb1/core/src/main/java/com/netflix/conductor/core/execution/tasks/DoWhile.java#L161-L184) | 不采用；外部任务接口最好，但 footprint/分发不符合本轮轻量优先，且同样不证明终止 |
| **Windmill** | flow schema 本身允许 script、raw script、for/while loop、subflow 与 agent 等[代码承载模块](https://github.com/windmill-labs/windmill/blob/74af4ed939fb8d78aff8ecd4af812860595c6c5a/backend/windmill-types/src/flows.rs#L984-L1093)，validator 主要做[反序列化、retry 与 path](https://github.com/windmill-labs/windmill/blob/74af4ed939fb8d78aff8ecd4af812860595c6c5a/backend/windmill-types/src/flows.rs#L233-L275)，部署由 [PostgreSQL、server 和 worker](https://github.com/windmill-labs/windmill/blob/74af4ed939fb8d78aff8ecd4af812860595c6c5a/docker-compose.yml#L10-L79)组成，且[许可混合](https://github.com/windmill-labs/windmill/blob/74af4ed939fb8d78aff8ecd4af812860595c6c5a/LICENSE#L1-L31) | 不采用；明显偏 code platform，服务面和授权面都过大 |
| **Kestra** | YAML 是声明式入口，但 Flow validator 只挡重复 ID 与[直接自递归](https://github.com/kestra-io/kestra/blob/e77ad77142e50af6f62c06354b8711263155086f/core/src/main/java/io/kestra/core/validations/validator/FlowValidator.java#L32-L78)，测试中存在[合法互递归 subflow](https://github.com/kestra-io/kestra/blob/e77ad77142e50af6f62c06354b8711263155086f/core/src/test/resources/flows/valids/subflow-function-mutual-a.yaml#L1-L9)，`LoopUntil` 未设限制时[不限制迭代数](https://github.com/kestra-io/kestra/blob/e77ad77142e50af6f62c06354b8711263155086f/core/src/main/java/io/kestra/plugin/core/flow/LoopUntil.java#L242-L263)；local mode 仍是 [JVM + H2](https://github.com/kestra-io/kestra/blob/e77ad77142e50af6f62c06354b8711263155086f/cli/src/main/java/io/kestra/cli/commands/servers/LocalCommand.java#L25-L42) | 不采用；插件/运行面远超所需，lint 能力没有抵消重量 |
| **Direktiv** | 稳定版有声明式 [workflow model](https://github.com/direktiv/direktiv/blob/6156f450798d002ab52bd933a03dd1ffaf545a20/pkg/model/workflow.go#L13-L84)与[validator](https://github.com/direktiv/direktiv/blob/6156f450798d002ab52bd933a03dd1ffaf545a20/pkg/model/workflow.go#L174-L232)，同时允许[显式自循环](https://github.com/direktiv/direktiv/blob/6156f450798d002ab52bd933a03dd1ffaf545a20/tests/engine/simple-loop.test.js#L22-L48)并依赖 NATS/Postgres/Kubernetes/Knative 等[较重栈](https://github.com/direktiv/direktiv/blob/6156f450798d002ab52bd933a03dd1ffaf545a20/go.mod#L21-L53)；当前主干 compiler 已消费 [`.wf.ts`](https://github.com/direktiv/direktiv/blob/9ad27a0de2aa986a85612867cf8e73c8822d0529/internal/core/compiler.go#L8-L64) | 不采用；部署重，且演进方向重新引入 code-as-workflow |
| **Serverless Workflow Synapse** | CNCF Serverless Workflow schema 原生允许 [for/while 条件循环](https://github.com/serverlessworkflow/specification/blob/2dd2c84170d5f3e05d58e913e9ca298dcf8d543a/schema/workflow.yaml#L685-L729)，官方示例包含[永久消费循环](https://github.com/serverlessworkflow/specification/blob/2dd2c84170d5f3e05d58e913e9ca298dcf8d543a/examples/call-asyncapi-subscribe-consume-forever-foreach.yaml#L1-L27)；Synapse compose 由 [Redis、API、operator、correlator](https://github.com/serverlessworkflow/synapse/blob/ba3fbfd5125995bba9fb5900aed181a0775d538c/deployments/docker-compose/docker-compose.yml#L1-L55)组成，standalone runner 只给[内存执行上下文](https://github.com/serverlessworkflow/synapse/blob/ba3fbfd5125995bba9fb5900aed181a0775d538c/src/runner/Synapse.Runner/Services/StandAloneWorkflowExecutionContext.cs#L27-L40) | 不采用 runtime；保留 DSL/spec 作为 schema 参考 |
| **TIBCO Flogo / project-flogo/flow** | flow JSON 是 task/link 图（[serializer](https://github.com/project-flogo/flow/blob/0683f40cb531ee755d78042a2646228bb43daf24/definition/definition_ser.go#L26-L61)），含 [DoWhile](https://github.com/project-flogo/flow/blob/0683f40cb531ee755d78042a2646228bb43daf24/model/simple/dowhilebehavior.go#L72-L99)；runtime 用[最大 step 数](https://github.com/project-flogo/flow/blob/0683f40cb531ee755d78042a2646228bb43daf24/action.go#L421-L443)作保险，但 persistence 只有[可选 recorder 接口](https://github.com/project-flogo/flow/blob/0683f40cb531ee755d78042a2646228bb43daf24/state/recorder.go#L1-L14) | 不采用 runtime；它是嵌入式 flow library，不是完整耐久服务 |
| **AWS Step Functions Local** | ASL 声明式、可配合 [StateLint](https://github.com/awslabs/statelint/blob/5388321f0c0c4b24df9308c02dff4d3adfe74527/lib/statelint/state_node.rb#L207-L224)，但 AWS 官方明确称 Local [unsupported、仅供测试](https://docs.aws.amazon.com/step-functions/latest/dg/sfn-local.html) | 不可作生产本地 runtime；ASL/StateLint 仅作语法设计参考 |
| **SCXML / XState 自建** | SCXML 是成熟的[状态图标准](https://www.w3.org/TR/scxml/)；XState machine config 仍可嵌入[函数实现](https://github.com/statelyai/xstate/blob/8d878b3bed7ccb5d23b74474475659b9f0472306/packages/core/src/createMachine.ts#L31-L77)，默认内部迭代上限可为 [Infinity](https://github.com/statelyai/xstate/blob/8d878b3bed7ccb5d23b74474475659b9f0472306/packages/core/src/StateMachine.ts#L126-L145)，snapshot API 只解决[状态快照](https://github.com/statelyai/xstate/blob/8d878b3bed7ccb5d23b74474475659b9f0472306/packages/core/src/createActor.ts#L783-L819) | 不自建；还需补 durable store、lease、timer、API、恢复与运维，收益不足以抵消重造引擎 |

结论不是“Dagu 最强”，而是它在当前约束下最小：Conductor 的外部 task 约束更顺手，Serverless Workflow/SCXML 的形式更标准，Windmill/Kestra 的产品能力更多；这些优势都不值得引入相应重量或自建耐久运行时。

## 2026-08-30 provider 管理动作复核

v0.15.0 重新核对“Dagu 原生界面能否像普通客户端一样驱动 HCTL”。结论来自固定 commit 的 UI 源码与 E2E，不是 README 功能表：DAG run action 测试覆盖 Stop、Retry 与 Reschedule（[`dag-run-actions.spec.ts`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/ui/e2e/dag-run-actions.spec.ts)）；action component 直接提供 Stop/cancel、Retry/reschedule、Dequeue 与 Reject（[`DAGRunActions.tsx`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/ui/src/features/dag-runs/components/common/DAGRunActions.tsx)）；approval tab 可以 Approve 或带输入 Retry/push-back（[`ApprovalTab.tsx`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/ui/src/features/dags/components/approval/ApprovalTab.tsx)）；editor 还能 Rename/Delete definition（[`DAGEditButtons.tsx`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/ui/src/features/dags/components/dag-editor/DAGEditButtons.tsx)）。

这些是 Dagu 自身完整管理产品的合理能力，但 mutation 会在 Dagu 里先发生。HCTL 的 Cancel/Retry/Approve 必须先在 metadata 账本 CAS Run/Attempt，撤销旧 lease/generation、持久化 outbox，再调用 engine；事后看到相似 Dagu 状态无法补回这个顺序，也无法证明旧执行已失权。因此 Dagu UI 对未绑定对象可作实例管理，对 HCTL 已绑定 definition/execution 只作观察和应急管理；直接 mutation 回读为分歧，不归一成 human Run 命令。若上游将来提供 mutation 前拦截/外部 task 控制约束，再在新 binding revision 重新评估，而不是在 HCTL 里重写 Dagu 管理界面。

<a id="2026-10-19-pre-impl-probe"></a>
## 2026-10-19 开工前限时验证（v2.15.1 实测）

作为 **P2.5 第 0 包**（对应 `.memo/design/p2-control-20260906/07-demo-kickoff.md` §六「专项 · workflow 引擎（Dagu）的开工前限时验证」与 `.memo/design/p2-control-20260906/09-p25-kickoff.md` §四「第 0 包」），在 **P2.5 第 1 包（施工图）** 与 **P2.5 第 2 包（workflow engine 端口与 Dagu 适配器）** 开工前，使用仓库随包锁定的 Dagu `v2.15.1`（`root//packaging/dependencies:dagu`，上游提交 [`532c5129`](https://github.com/dagucloud/dagu/tree/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a)），在一处用完即删的临时目录（`/var/tmp/hctl2-dagu-probe.XXXXXX`，动态偏移独立端口，不触碰 `~/.local/state/hctl2` 或任何运行中实例）逐项实测三组能力与 P0 代次栅栏判据。

### 逐项判定汇总

**总结论：三组验证项与 P0 代次栅栏判据全部闭环，不触发重开选型，照常进 P2.5 第 1 包（施工图）与第 2 包（workflow engine 端口与 Dagu 适配器）。** 其中引擎层标为「做不到」的四项均属已知边界，已在 [Run 约束](../design/spec/run.md) 与 [sdk/dagu.md](./sdk/dagu.md) 中由控制面语义承接。

| 组别 | 验证子项 | 引擎端点 / 机制 | 实测结论 | 关键行为与源码/约束落点 |
| --- | --- | --- | --- | --- |
| **1. DAG 提交、启动、暂停、恢复、取消与回读** | 1A-1 合法 `human.task` 规范校验 | `POST /api/v1/dags/validate` | **通过** | HTTP 200 `valid: true`；引擎自动为 `with.form` 注入 `"additionalProperties": false` 并从表单属性推导类型化 `step.outputs` |
| | 1A-2 缺失依赖校验 | `POST /api/v1/dags/validate` | **通过** | HTTP 200 `valid: false`，`errors: ["field 'depends': step s1 depends on non-existent step missing_step ..."]` |
| | 1A-3 依赖环静态校验 vs 运行时拦截 | `POST /api/v1/dags/validate`<br>`POST /api/v1/dag-runs` | **静态 `validate` 做不到检环；运行时启动拒绝通过** | `s1 <-> s2` 在 `/dags/validate` 返回 `valid: true`（[`DAG.Validate`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/ir/dag.go#L448-L475) 不检环）；`POST /dag-runs` 在构建执行计划时以 HTTP 500 `DAG start process exited before publishing status: exit status 1` 拒绝。印证上文「HCTL 图 lint 必须先拒绝环」（落 P2.5 第 1 包） |
| | 1B-1 文件式提交与启动 | `POST /api/v1/dags`<br>`PUT /api/v1/dags/{fileName}/spec`<br>`POST /api/v1/dags/{fileName}/start` | **通过** | 分别返回 HTTP 201、HTTP 200 `{"errors":null}`、HTTP 200 `{"dagRunId":"run-file-001"}`；重复 `dagRunId` 返回 HTTP 409 `already_exists` |
| | 1B-2 内联规范直接启动与入队 | `POST /api/v1/dag-runs`<br>`POST /api/v1/dag-runs/enqueue` | **通过** | 均返回 HTTP 200 `{"dagRunId":...}`，无需预建 DAG 文件；队列处理器拉起后 `gate_a` 进入 `status: 7 ("waiting")` |
| | 1B-3 Run 与步骤状态回读 | `GET /api/v1/dag-runs/{name}/{dagRunId}`<br>`GET …/steps/{stepName}/status` | **主回读通过；独立步骤 GET 端点做不到（405）** | `GET /dag-runs/{name}/{dagRunId}` 完整返回 run `status`/`statusLabel` 及 `.dagRunDetails.nodes[]`（含每步 `status`、`statusLabel`、`doneCount`、`startedAt`、`finishedAt`、插值后 `prompt`）；`/steps/{stepName}/status` 在 OpenAPI 中仅定义 `PATCH`，`GET` 返回 HTTP 405 |
| | 1C-1 Run 级暂停 / 恢复端点 | `POST /api/v1/dag-runs/{name}/{dagRunId}/pause`、`/suspend`<br>`POST /api/v1/dags/{fileName}/suspend`<br>`POST …/human-tasks/resume` | **引擎无 Run 级 pause（做不到）；控制面停派发方案通过** | Run 级 `/pause`、`/suspend` 均为 HTTP 404；DAG 级 `/suspend`（HTTP 200）仅切换 `.suspended` 布尔值以启停 cron 调度；`/human-tasks/resume` 仅用于 `complete` 落盘后入队失败（`humanTaskResumePending`）的补偿重入队，对仍等待输入的步骤返回 HTTP 409。HCTL 的暂停/恢复落在控制面停发派工与停调 `complete`，引擎保持无进程 `waiting (7)` |
| | 1D-1 取消 / 篡改 `Waiting (7)` 态（无进程 `human.task`）run | `POST …/stop`、`GET …/dequeue`、`POST …/reject`、`POST …/approve`、`POST …/push-back`、`PATCH …/status` | **引擎侧做不到改写为 `aborted`（不触发重开选型）** | `human.task` 进入 `waiting` 时 runner 进程已退出：`/stop` 返回 HTTP 400 `DAG is not running`（[`TerminateDAGRun`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/service/frontend/api/v1/dagruns.go#L3264-L3305) 只接受 `Running` 或待自动重试的 `Failed`）；`/dequeue` 返回 HTTP 400 `not queued: waiting`；`/reject`、`/approve`、`/push-back` 与 `PATCH .../status` 均因 `human.task` 保护返回 HTTP 400。由于无进程 `human.task` 零进程、零定时器，HCTL 取消/替代 Run 时撤销本地租约、关闭 `Run–Engine Binding` 并永不再对该 `dagRunId` 调 `complete` |
| | 1D-2–1D-3 取消 `Running (1)` / `Queued (5)` 态 run 与 `max_active_runs` 队列槽验证 | `POST /api/v1/dag-runs/{name}/{dagRunId}/stop`<br>`GET /api/v1/dag-runs/{name}/{dagRunId}/dequeue`<br>`POST /api/v1/dag-runs/enqueue` | **通过** | `Running` 态经 `POST .../stop`（HTTP 200）转为 `status: 3 ("aborted")`；`Queued` 态经 `GET .../dequeue`（HTTP 200，调用 [`queue.AbortQueuedDAGRun`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/queue/queue_abort.go#L34-L93)）移除空 run 记录（后续 `GET` 返回 HTTP 404）。注意：直接启动（`POST /dag-runs` 与 `POST /dags/{name}/start`）**不受** `max_active_runs: 1` 限制（会直接并发进入 `running`），只有走入队（`POST /dag-runs/enqueue`、`POST /dags/{name}/enqueue`）才受 `max_active_runs` 限制；而在入队路径下，处于 `waiting (7)` 的 `human.task` run **不占用** `max_active_runs: 1` 的并发槽，同 DAG 后续 `enqueue` 的新 `dagRunId` 会立即出队进入 `waiting (7)` |
| **2. `human.task` 等待、完成与回读** | 2.1–2.2 进入等待态与跨进程重启持久化 | `dagu start-all` 重启 + `GET /api/v1/dag-runs/{name}/{dagRunId}` | **通过** | 无进程 `human.task` 进入 `status: 7 ("waiting")`；`SIGTERM` 停止并重启 `dagu start-all` 后，文件存储原样恢复 `status: 7 ("waiting")` 与 `startedAt` |
| | 2.3–2.4 表单 Schema 校验、完成推进与下游插值 | `POST /api/v1/dag-runs/{name}/{dagRunId}/human-tasks/{stepId}/complete` | **通过（含队列过渡态约束）** | 缺必填字段返回 HTTP 400 `human task form validation failed`；合法输入返回 HTTP 200 `{"alreadyCompleted":false,"queued":true,"remainingWaitingSteps":0}`。完成会将新 attempt 入队（短暂经过 `status: 5 ("queued")`），随后拉起下游 `gate_b` 进入 `waiting (7)`，并将 `${steps.gate_a.outputs.receipt_id}` 插值进 `gate_b` 的 `prompt` |
| | 2.5–2.6 幂等重投 vs 冲突输入拒绝 | `POST …/human-tasks/{stepId}/complete` | **通过** | 相同 canonical JSON 重投返回 HTTP 200 `{"alreadyCompleted":true,"queued":false}`（不重复入队）；不同 JSON（如不同 `attempt_generation` 或 `receipt_id`）返回 HTTP 409 `human task step "gate_a" was already completed with different input` |
| | 2.7 终态回读与 `/outputs` 行为 | `GET /api/v1/dag-runs/{name}/{dagRunId}`<br>`GET /api/v1/dag-runs/{name}/{dagRunId}/outputs` | **通过** | `gate_b` 完成后 run 转为 `status: 4 ("succeeded")`；注意 `human.task` 禁止顶层 `output:` 字段，其表单输出存入 `StepOutputsValue` 供图内 `${steps.<id>.outputs.<field>}` 插值，不写入 `outputs.json`（`/outputs` 返回空映射 `{}`） |
| **3. 引擎自推进 / 重试与 HCTL 回读分歧检查（含 P0 栅栏）** | 3.1–3.4 引擎禁止 `human.task` 自动/手动重试、重复、并行与状态篡改 | `POST /dags/validate`<br>`POST …/retry`<br>`PATCH …/steps/{stepName}/status` | **通过** | 规范层拒绝 `human.task` 配置 `retry_policy` / `repeat_policy` / `parallel` / `timeout_sec`（[`human_task.go#L13-L43`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/internal/spec/human_task.go#L13-L43)）；API 层拒绝单步 retry `human.task`（HTTP 409）、拒绝在 `waiting` 态整盘 retry（HTTP 409）、对已 `succeeded` 的 run 整盘 retry（HTTP 200，重试后连续观察 2s/4s）不会把已完成的 `human.task` 打回 `waiting`（保持 `succeeded`、`doneCount: 1`）、拒绝 `PATCH` 改写 `human.task` 状态（HTTP 400）；`reschedule` 强制要求新 `dagRunId`（原 ID 返回 HTTP 409） |
| | 3.5–3.7 旁路 `complete` 分歧识别与「ACK 未知 + `SIGKILL` 崩溃恢复」压力验证 | `GET /api/v1/dag-runs/{name}/{dagRunId}`<br>`POST …/human-tasks/{stepId}/complete` | **通过** | 在 HCTL Profile 下（P2.5 第 1 包编译器禁止把 loop / 动态分支编译成重入同一 `(dagRunId, step)` 的结构），同一 `(dagName, dagRunId)` 的每个 `human.task` 最多进入 `waiting` 一次。把 `obligation_id`、`attempt_generation`、`receipt_id` 写入必填 `form` 后：① 旁路直接调 `complete` 会使回读看到 `gate_a = succeeded`、`gate_b = waiting` 而控制面无对应完成 outbox（且 HCTL 事后提交合法结果会收到 HTTP 409 Conflict），control 立即将 `Run–Engine Binding` 标为**分歧**并拒绝为 `gate_b` 创建 Obligation；② `complete` 返回 200 后立即 `kill -KILL` `dagu start-all` 并重启，立即重投同一份 `complete` 幂等返回 HTTP 200 `alreadyCompleted: true, queued: false`，队列恢复后 `gate_b` 正常进入 `waiting` 且 `prompt` 已插值 `rcpt-crash-001`，此时调 `/human-tasks/resume` 返回 HTTP 409 |

### P2.5 第 1、2 包落盘的三条机械约束

1. **静态无环 lint 与不重入约束前置（落 P2.5 第 1 包 · 施工图）**：HCTL 编译器在调用 `POST /api/v1/dags/validate` 之前必须先完成 HCTL 图结构拓扑检环（不得依赖 Dagu `validate` 检环），且不得把 loop 或动态分支（`foreach`）编译成在单次 `dagRunId` 内重入同一 `step` 的 Dagu repeat 或子 DAG 结构，确保同一 `(dagName, dagRunId, stepId)` 最多进入 `waiting` 一次。
2. **轮询回读等待 `waiting (7)` 就绪后再操作，并处理 `SIGKILL` 后的 scheduler 目录锁恢复窗口（落 P2.5 第 2 包 · workflow engine 端口与 Dagu 适配器）**：`complete` 返回 `"queued": true` 后，run 会先进入 `status: 5 ("queued")` 再由队列协程拉起新 attempt 使下游节点进入 `status: 7 ("waiting")`（若在 `queued` 窗口内对下游节点调 `complete` 会收到 HTTP 409 `DAG-run ... is not waiting (status: queued)`）。控制面观察器与重投器一律以 `GET /api/v1/dag-runs/{name}/{dagRunId}` 中 `dagRunDetails.status == 7` 且目标节点 `node.status == 7` 为准。另外，`dagu start-all` 若因 `SIGKILL` 非正常退出，`$DAGU_HOME/data/scheduler/locks/.dagu_lock` 目录锁不会自动触发退出清理，有两种处理方式供 P2.5 第 2 包取舍：① **不删锁（推荐，不耦合 Dagu 内部目录结构）**：由 Dagu `dirlock` 自身 stale 超时机制接管，实测重启后约 26s–30s 自动过期并恢复队列推进，代价是 `SIGKILL` 重启后约半分钟内队列不推进；② **重启前删锁**：确认旧 `dagu` 进程已退出后删除 `.dagu_lock` 可立即推进，但依赖 Dagu 内部锁目录路径，升级版本可能变化。
3. **代次与凭证写入 `human.task` 必填 `form`（生成侧落 P2.5 第 1 包，写入与回读对账侧落 P2.5 第 2 包）**：编译器为每个外部检查点生成的 `human.task` 必须包含 `obligation_id`、`attempt_generation`、`verdict`、`receipt_id` 四个 `required` 表单字段。ACK 未知重投同一表单天然幂等（HTTP 200 `alreadyCompleted: true, queued: false`）；旧代次迟到写入或旁路篡改均触发 HTTP 409 `conflict` 或回读状态不匹配，从而将 `Run–Engine Binding` 收口为分歧。

### 跨平台一键复现命令（macOS / Linux 通用，依赖 `curl`、`jq`）

在仓库根执行以下脚本即可在独立临时目录（`/var/tmp/hctl2-dagu-probe.XXXXXX`）中从零跑完上述全部验证并自动清理进程与目录（可通过 `DAGU_PROBE_PORT_BASE` 指定起始端口，默认按 `$$` 动态偏移）：

```bash
bash <<'EOF'
set -euo pipefail

for cmd in curl jq awk find; do
    command -v "$cmd" >/dev/null 2>&1 || { echo "error: required command '$cmd' not found" >&2; exit 1; }
done

DAGU_CACHE_ROOT="$(cd src && ./buck2 build root//packaging/dependencies:dagu --show-full-output | awk '/root\/\/packaging\/dependencies:dagu/ {print $2}')"
DAGU_BIN="$(find "$DAGU_CACHE_ROOT" -type f -path '*/bin/dagu' -print -quit)"
[[ -x "$DAGU_BIN" ]] || { echo "error: dagu binary not found under $DAGU_CACHE_ROOT" >&2; exit 1; }

PROBE_DIR="$(mktemp -d /var/tmp/hctl2-dagu-probe.XXXXXX)"
DAGU_PID=""

stop_dagu() {
    if [[ -n "$DAGU_PID" ]] && kill -0 "$DAGU_PID" 2>/dev/null; then
        kill -TERM "$DAGU_PID" 2>/dev/null || true
        wait "$DAGU_PID" 2>/dev/null || true
    fi
    DAGU_PID=""
}

kill9_dagu() {
    if [[ -n "$DAGU_PID" ]] && kill -0 "$DAGU_PID" 2>/dev/null; then
        kill -KILL "$DAGU_PID" 2>/dev/null || true
        wait "$DAGU_PID" 2>/dev/null || true
    fi
    rm -rf "$PROBE_DIR/data/data/scheduler/locks/.dagu_lock"
    DAGU_PID=""
}

cleanup() {
    stop_dagu
    rm -rf "$PROBE_DIR"
}
trap cleanup EXIT

mkdir -p "$PROBE_DIR/data" "$PROBE_DIR/logs"

PORT_BASE="${DAGU_PROBE_PORT_BASE:-$((28000 + ($$ % 200) * 10))}"
export DAGU_HOME="$PROBE_DIR/data"
export DAGU_HOST="127.0.0.1"
export DAGU_PORT="$((PORT_BASE + 0))"
export DAGU_SCHEDULER_PORT="$((PORT_BASE + 1))"
export DAGU_COORDINATOR_HOST="127.0.0.1"
export DAGU_COORDINATOR_PORT="$((PORT_BASE + 2))"
export DAGU_COORDINATOR_HEALTH_PORT="$((PORT_BASE + 3))"
export DAGU_AUTH_MODE="none"
export DAGU_TERMINAL_ENABLED="false"

BASE_URL="http://127.0.0.1:${DAGU_PORT}/api/v1"

start_dagu() {
    "$DAGU_BIN" start-all >>"$PROBE_DIR/logs/dagu.log" 2>&1 &
    DAGU_PID=$!
    for _ in $(seq 1 60); do
        if curl -fsS "${BASE_URL}/health" >/dev/null 2>&1; then
            return 0
        fi
        sleep 0.2
    done
    echo "error: dagu start-all did not become healthy" >&2
    exit 1
}

wait_for_run_status() {
    local dag_name="$1"
    local run_id="$2"
    local expected_status="$3"
    for _ in $(seq 1 50); do
        local st
        st="$(curl -fsS "${BASE_URL}/dag-runs/${dag_name}/${run_id}" 2>/dev/null | jq -r '.dagRunDetails.statusLabel' 2>/dev/null || true)"
        if [[ "$st" == "$expected_status" ]]; then
            return 0
        fi
        sleep 0.2
    done
    echo "error: timed out waiting for ${dag_name}/${run_id} status=${expected_status}" >&2
    return 1
}

wait_for_step_status() {
    local dag_name="$1"
    local run_id="$2"
    local step_id="$3"
    local expected_status="$4"
    for _ in $(seq 1 50); do
        local st
        st="$(curl -fsS "${BASE_URL}/dag-runs/${dag_name}/${run_id}" 2>/dev/null | jq -r --arg sid "$step_id" '.dagRunDetails.nodes[] | select(.step.id == $sid or .step.name == $sid) | .statusLabel' 2>/dev/null || true)"
        if [[ "$st" == "$expected_status" ]]; then
            return 0
        fi
        sleep 0.2
    done
    echo "error: timed out waiting for ${dag_name}/${run_id}/${step_id} status=${expected_status}" >&2
    return 1
}

echo "=================================================================="
echo "0. VERSION & HEALTH"
echo "=================================================================="
"$DAGU_BIN" version
start_dagu
curl -sS -w "\nHTTP_STATUS:%{http_code}\n" "${BASE_URL}/health"

SPEC_TWO_GATE='steps:
  - id: gate_a
    action: human.task
    with:
      prompt: "Obligation gate_a waiting for HCTL completion"
      form:
        type: object
        properties:
          obligation_id:
            type: string
          attempt_generation:
            type: integer
          verdict:
            type: string
            enum: [accepted, rejected]
          receipt_id:
            type: string
        required: [obligation_id, attempt_generation, verdict, receipt_id]
  - id: gate_b
    depends: gate_a
    action: human.task
    with:
      prompt: "Obligation gate_b after gate_a (${steps.gate_a.outputs.receipt_id})"
      form:
        type: object
        properties:
          obligation_id:
            type: string
          attempt_generation:
            type: integer
          verdict:
            type: string
            enum: [accepted, rejected]
          receipt_id:
            type: string
        required: [obligation_id, attempt_generation, verdict, receipt_id]
'

echo "=================================================================="
echo "1A. DAG VALIDATION (VALID / MISSING DEP / CYCLE)"
echo "=================================================================="
echo "--- POST /api/v1/dags/validate (valid 2-step human.task DAG) ---"
jq -n --arg spec "$SPEC_TWO_GATE" '{spec: $spec}' | \
    curl -sS -X POST "${BASE_URL}/dags/validate" -H "Content-Type: application/json" -d @- -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- POST /api/v1/dags/validate (missing dependency) ---"
jq -n '{spec: "steps:\n  - id: s1\n    depends: missing_step\n    action: human.task\n    with:\n      prompt: test\n"}' | \
    curl -sS -X POST "${BASE_URL}/dags/validate" -H "Content-Type: application/json" -d @- -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- POST /api/v1/dags/validate (cycle s1 <-> s2: passes static validate!) ---"
SPEC_CYCLE='steps:
  - id: s1
    depends: s2
    action: human.task
    with:
      prompt: "step 1"
  - id: s2
    depends: s1
    action: human.task
    with:
      prompt: "step 2"
'
jq -n --arg spec "$SPEC_CYCLE" '{spec: $spec}' | \
    curl -sS -X POST "${BASE_URL}/dags/validate" -H "Content-Type: application/json" -d @- -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- POST /api/v1/dag-runs (cycle s1 <-> s2 fails at runtime plan build with HTTP 500) ---"
jq -n --arg spec "$SPEC_CYCLE" '{name: "hctl-cycle-probe", dagRunId: "run-cycle-001", spec: $spec}' | \
    curl -sS -X POST "${BASE_URL}/dag-runs" -H "Content-Type: application/json" -d @- -w "\nHTTP_STATUS:%{http_code}\n"

echo "=================================================================="
echo "1B. DAG SUBMISSION & START (FILE-BACKED + INLINE + ENQUEUE)"
echo "=================================================================="
echo "--- POST /api/v1/dags & PUT /api/v1/dags/hctl-file-probe/spec ---"
curl -sS -X POST "${BASE_URL}/dags" -H "Content-Type: application/json" \
    -d '{"name":"hctl-file-probe"}' -w "\nHTTP_STATUS:%{http_code}\n"
jq -n --arg spec "$SPEC_TWO_GATE" '{spec: $spec}' | \
    curl -sS -X PUT "${BASE_URL}/dags/hctl-file-probe/spec" -H "Content-Type: application/json" -d @- -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- POST /api/v1/dags/hctl-file-probe/start (run-file-001) ---"
curl -sS -X POST "${BASE_URL}/dags/hctl-file-probe/start" -H "Content-Type: application/json" \
    -d '{"dagRunId":"run-file-001"}' -w "\nHTTP_STATUS:%{http_code}\n"
wait_for_step_status "hctl-file-probe" "run-file-001" "gate_a" "waiting"

echo "--- Duplicate dagRunId rejection on POST /api/v1/dags/hctl-file-probe/start ---"
curl -sS -X POST "${BASE_URL}/dags/hctl-file-probe/start" -H "Content-Type: application/json" \
    -d '{"dagRunId":"run-file-001"}' -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- POST /api/v1/dag-runs (inline spec: run-inline-001) ---"
jq -n --arg spec "$SPEC_TWO_GATE" '{name: "hctl-inline-probe", dagRunId: "run-inline-001", spec: $spec}' | \
    curl -sS -X POST "${BASE_URL}/dag-runs" -H "Content-Type: application/json" -d @- -w "\nHTTP_STATUS:%{http_code}\n"
wait_for_step_status "hctl-inline-probe" "run-inline-001" "gate_a" "waiting"

echo "--- POST /api/v1/dag-runs/enqueue (inline spec enqueue: run-enqueue-001) ---"
jq -n --arg spec "$SPEC_TWO_GATE" '{name: "hctl-inline-probe", dagRunId: "run-enqueue-001", spec: $spec}' | \
    curl -sS -X POST "${BASE_URL}/dag-runs/enqueue" -H "Content-Type: application/json" -d @- -w "\nHTTP_STATUS:%{http_code}\n"
wait_for_step_status "hctl-inline-probe" "run-enqueue-001" "gate_a" "waiting"
curl -sS "${BASE_URL}/dag-runs/hctl-inline-probe/run-enqueue-001" | jq '{dagRunId: .dagRunDetails.dagRunId, status: .dagRunDetails.status, statusLabel: .dagRunDetails.statusLabel, triggerType: .dagRunDetails.triggerType, nodes: [.dagRunDetails.nodes[] | {id: .step.id, status: .status, statusLabel: .statusLabel}]}'

echo "=================================================================="
echo "1C. PAUSE & RESUME CHECKS"
echo "=================================================================="
echo "--- Non-existent run-level pause/suspend endpoints ---"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/pause" -w "HTTP_STATUS:%{http_code}\n"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/suspend" -w "HTTP_STATUS:%{http_code}\n"

echo "--- DAG-level suspend POST /api/v1/dags/hctl-file-probe/suspend ---"
curl -sS -X POST "${BASE_URL}/dags/hctl-file-probe/suspend" -H "Content-Type: application/json" \
    -d '{"suspend":true}' -w "\nHTTP_STATUS:%{http_code}\n"
curl -sS "${BASE_URL}/dags/hctl-file-probe" | jq '{name: .dag.name, suspended: .suspended}'
curl -sS -X POST "${BASE_URL}/dags/hctl-file-probe/suspend" -H "Content-Type: application/json" \
    -d '{"suspend":false}' -w "\nHTTP_STATUS:%{http_code}\n"
curl -sS "${BASE_URL}/dags/hctl-file-probe" | jq '{name: .dag.name, suspended: .suspended}'

echo "--- POST /api/v1/dag-runs/hctl-file-probe/run-file-001/human-tasks/resume while step still waiting ---"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/human-tasks/resume" -w "\nHTTP_STATUS:%{http_code}\n"

echo "=================================================================="
echo "1D. CANCEL (STOP / DEQUEUE) ACROSS RUN STATES"
echo "=================================================================="
echo "--- 1D.1 Cancel/mutate a Waiting (7) human.task run via /stop, /dequeue, /reject, /approve, /push-back, /status ---"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-inline-probe/run-enqueue-001/stop" -w "\nHTTP_STATUS:%{http_code}\n"
curl -sS -X GET "${BASE_URL}/dag-runs/hctl-inline-probe/run-enqueue-001/dequeue" -w "\nHTTP_STATUS:%{http_code}\n"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-inline-probe/run-enqueue-001/steps/gate_a/reject" \
    -H "Content-Type: application/json" -d '{"reason":"cancel"}' -w "\nHTTP_STATUS:%{http_code}\n"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-inline-probe/run-enqueue-001/steps/gate_a/approve" \
    -H "Content-Type: application/json" -d '{}' -w "\nHTTP_STATUS:%{http_code}\n"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-inline-probe/run-enqueue-001/steps/gate_a/push-back" \
    -H "Content-Type: application/json" -d '{"reason":"rework"}' -w "\nHTTP_STATUS:%{http_code}\n"
curl -sS -X PATCH "${BASE_URL}/dag-runs/hctl-inline-probe/run-enqueue-001/steps/gate_a/status" \
    -H "Content-Type: application/json" -d '{"status":3}' -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- 1D.2 Cancel a Queued (5) run via GET /dequeue and a Running (1) run via POST /stop (plus direct start vs enqueue on max_active_runs=1) ---"
SPEC_RUNNING='max_active_runs: 1
steps:
  - id: long_step
    command: sleep 30
'
curl -sS -X POST "${BASE_URL}/dags" -H "Content-Type: application/json" \
    -d '{"name":"hctl-running-cancel"}' -w "\nHTTP_STATUS:%{http_code}\n"
jq -n --arg spec "$SPEC_RUNNING" '{spec: $spec}' | \
    curl -sS -X PUT "${BASE_URL}/dags/hctl-running-cancel/spec" -H "Content-Type: application/json" -d @- -w "\nHTTP_STATUS:%{http_code}\n"

curl -sS -X POST "${BASE_URL}/dags/hctl-running-cancel/start" -H "Content-Type: application/json" \
    -d '{"dagRunId":"run-cancel-running-001"}' -w "\nHTTP_STATUS:%{http_code}\n"
wait_for_run_status "hctl-running-cancel" "run-cancel-running-001" "running"

echo "Direct start (POST /dags/{name}/start and POST /dag-runs) bypasses max_active_runs=1:"
curl -sS -X POST "${BASE_URL}/dags/hctl-running-cancel/start" -H "Content-Type: application/json" \
    -d '{"dagRunId":"run-direct-start-002"}' -w "\nHTTP_STATUS:%{http_code}\n"
wait_for_run_status "hctl-running-cancel" "run-direct-start-002" "running"
curl -sS "${BASE_URL}/dag-runs/hctl-running-cancel/run-direct-start-002" | jq '{dagRunId: .dagRunDetails.dagRunId, statusLabel: .dagRunDetails.statusLabel}'
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-running-cancel/run-direct-start-002/stop" -w "\nHTTP_STATUS:%{http_code}\n"
wait_for_run_status "hctl-running-cancel" "run-direct-start-002" "aborted"

echo "Enqueue (POST /dag-runs/enqueue) respects max_active_runs=1 and stays queued while run-cancel-running-001 is running:"
jq -n --arg spec "$SPEC_RUNNING" '{name: "hctl-running-cancel", dagRunId: "run-cancel-queued-002", spec: $spec}' | \
    curl -sS -X POST "${BASE_URL}/dag-runs/enqueue" -H "Content-Type: application/json" -d @- -w "\nHTTP_STATUS:%{http_code}\n"
wait_for_run_status "hctl-running-cancel" "run-cancel-queued-002" "queued"
curl -sS -X GET "${BASE_URL}/dag-runs/hctl-running-cancel/run-cancel-queued-002/dequeue" -w "\nHTTP_STATUS:%{http_code}\n"
curl -sS "${BASE_URL}/dag-runs/hctl-running-cancel/run-cancel-queued-002" -w "\nHTTP_STATUS:%{http_code}\n"

curl -sS -X POST "${BASE_URL}/dag-runs/hctl-running-cancel/run-cancel-running-001/stop" -w "\nHTTP_STATUS:%{http_code}\n"
wait_for_run_status "hctl-running-cancel" "run-cancel-running-001" "aborted"
curl -sS "${BASE_URL}/dag-runs/hctl-running-cancel/run-cancel-running-001" | jq '{dagRunId: .dagRunDetails.dagRunId, status: .dagRunDetails.status, statusLabel: .dagRunDetails.statusLabel, nodes: [.dagRunDetails.nodes[] | {id: .step.id, status: .status, statusLabel: .statusLabel}]}'

echo "--- 1D.3 Waiting (7) human.task run does NOT occupy maxActiveRuns=1 queue slot when subsequent runs are enqueued ---"
jq -n --arg spec "$SPEC_TWO_GATE" '{name: "hctl-inline-probe", dagRunId: "run-enqueue-002", spec: $spec}' | \
    curl -sS -X POST "${BASE_URL}/dag-runs/enqueue" -H "Content-Type: application/json" -d @- -w "\nHTTP_STATUS:%{http_code}\n"
wait_for_step_status "hctl-inline-probe" "run-enqueue-002" "gate_a" "waiting"
curl -sS "${BASE_URL}/dag-runs/hctl-inline-probe/run-enqueue-002" | jq '{dagRunId: .dagRunDetails.dagRunId, statusLabel: .dagRunDetails.statusLabel, gateA: (.dagRunDetails.nodes[] | select(.step.id == "gate_a") | .statusLabel)}'

echo "=================================================================="
echo "2. HUMAN.TASK WAITING, COMPLETION, RESTART PERSISTENCE, IDEMPOTENCY & OUTPUTS"
echo "=================================================================="
echo "--- 2.1 Readback initial Waiting state on run-file-001 ---"
curl -sS "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001" | jq '{dagRunId: .dagRunDetails.dagRunId, status: .dagRunDetails.status, statusLabel: .dagRunDetails.statusLabel, nodes: [.dagRunDetails.nodes[] | {id: .step.id, status: .status, statusLabel: .statusLabel, doneCount: .doneCount, startedAt: .startedAt, finishedAt: .finishedAt, prompt: .step.humanTask.prompt, outputs: .step.outputs}]}'
curl -sS "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/steps/gate_a/status" -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- 2.2 Server restart while run-file-001 is Waiting at gate_a ---"
stop_dagu
start_dagu
echo "--- Readback run-file-001 after Dagu server restart ---"
curl -sS "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001" | jq '{dagRunId: .dagRunDetails.dagRunId, status: .dagRunDetails.status, statusLabel: .dagRunDetails.statusLabel, nodes: [.dagRunDetails.nodes[] | {id: .step.id, status: .status, statusLabel: .statusLabel, startedAt: .startedAt}]}'

echo "--- 2.3 Invalid form input rejected with HTTP 400 ---"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/human-tasks/gate_a/complete" \
    -H "Content-Type: application/json" \
    -d '{"verdict":"accepted"}' -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- 2.4 Valid completion of gate_a ---"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/human-tasks/gate_a/complete" \
    -H "Content-Type: application/json" \
    -d '{"obligation_id":"obl-1","attempt_generation":1,"verdict":"accepted","receipt_id":"rcpt-001"}' -w "\nHTTP_STATUS:%{http_code}\n"

wait_for_step_status "hctl-file-probe" "run-file-001" "gate_b" "waiting"
echo "--- Readback run-file-001 after gate_a succeeded and gate_b entered waiting ---"
curl -sS "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001" | jq '{dagRunId: .dagRunDetails.dagRunId, status: .dagRunDetails.status, statusLabel: .dagRunDetails.statusLabel, nodes: [.dagRunDetails.nodes[] | {id: .step.id, status: .status, statusLabel: .statusLabel, doneCount: .doneCount, startedAt: .startedAt, finishedAt: .finishedAt, prompt: .step.humanTask.prompt, completedBy: .humanTaskCompletedBy}]}'

echo "--- 2.5 Idempotent repeat of gate_a completion (same input -> HTTP 200 alreadyCompleted=true) ---"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/human-tasks/gate_a/complete" \
    -H "Content-Type: application/json" \
    -d '{"obligation_id":"obl-1","attempt_generation":1,"verdict":"accepted","receipt_id":"rcpt-001"}' -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- 2.6 Conflicting repeat of gate_a completion (different input -> HTTP 409 conflict) ---"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/human-tasks/gate_a/complete" \
    -H "Content-Type: application/json" \
    -d '{"obligation_id":"obl-1","attempt_generation":2,"verdict":"accepted","receipt_id":"rcpt-STALE"}' -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- 2.7 Complete gate_b and readback final Succeeded state + outputs ---"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/human-tasks/gate_b/complete" \
    -H "Content-Type: application/json" \
    -d '{"obligation_id":"obl-2","attempt_generation":1,"verdict":"accepted","receipt_id":"rcpt-002"}' -w "\nHTTP_STATUS:%{http_code}\n"

wait_for_run_status "hctl-file-probe" "run-file-001" "succeeded"
curl -sS "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001" | jq '{dagRunId: .dagRunDetails.dagRunId, status: .dagRunDetails.status, statusLabel: .dagRunDetails.statusLabel, startedAt: .dagRunDetails.startedAt, finishedAt: .dagRunDetails.finishedAt, nodes: [.dagRunDetails.nodes[] | {id: .step.id, status: .status, statusLabel: .statusLabel, startedAt: .startedAt, finishedAt: .finishedAt}]}'
curl -sS "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/outputs" | jq .

echo "=================================================================="
echo "3. ENGINE SELF-ADVANCE / RETRY VS HCTL READBACK DIVERGENCE CHECKS"
echo "=================================================================="
echo "--- 3.1 Engine blocks retry_policy / repeat_policy / parallel on human.task at spec validation ---"
jq -n '{spec: "steps:\n  - id: s1\n    action: human.task\n    retry_policy:\n      limit: 2\n    with:\n      prompt: test\n"}' | \
    curl -sS -X POST "${BASE_URL}/dags/validate" -H "Content-Type: application/json" -d @- | jq '{valid: .valid, errors: .errors}'
jq -n '{spec: "steps:\n  - id: s1\n    action: human.task\n    repeat_policy:\n      repeat: true\n    with:\n      prompt: test\n"}' | \
    curl -sS -X POST "${BASE_URL}/dags/validate" -H "Content-Type: application/json" -d @- | jq '{valid: .valid, errors: .errors}'
jq -n '{spec: "steps:\n  - id: s1\n    action: human.task\n    parallel:\n      items: [a, b]\n    with:\n      prompt: test\n"}' | \
    curl -sS -X POST "${BASE_URL}/dags/validate" -H "Content-Type: application/json" -d @- | jq '{valid: .valid, errors: .errors}'

echo "--- 3.2 Engine blocks direct step retry on human.task (even after run succeeded) ---"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/retry" \
    -H "Content-Type: application/json" \
    -d '{"dagRunId":"run-file-001","stepName":"gate_a"}' -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- 3.3a Engine blocks run retry while run is Waiting on human.task ---"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-inline-probe/run-inline-001/retry" \
    -H "Content-Type: application/json" \
    -d '{"dagRunId":"run-inline-001"}' -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- 3.3b Full run retry after run Succeeded does NOT re-enter waiting on completed human.task steps (observed across 2s/4s post-retry ticks) ---"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/retry" \
    -H "Content-Type: application/json" \
    -d '{"dagRunId":"run-file-001"}' -w "\nHTTP_STATUS:%{http_code}\n"
for tick in 2 4; do
    sleep 2
    echo "post-retry t+${tick}s:"
    curl -sS "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001" | jq '{dagRunId: .dagRunDetails.dagRunId, statusLabel: .dagRunDetails.statusLabel, nodes: [.dagRunDetails.nodes[] | {id: .step.id, statusLabel: .statusLabel, doneCount: .doneCount}]}'
done

echo "--- 3.4 Engine blocks manual step status PATCH on human.task (even after run succeeded) ---"
curl -sS -X PATCH "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/steps/gate_a/status" \
    -H "Content-Type: application/json" \
    -d '{"status":2}' -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- 3.5 Out-of-band human.task completion (UI/CLI bypasses HCTL outbox) -> Readback Divergence ---"
echo "Before out-of-band completion (run-inline-001 is waiting at gate_a):"
curl -sS "${BASE_URL}/dag-runs/hctl-inline-probe/run-inline-001" | jq '{dagRunId: .dagRunDetails.dagRunId, statusLabel: .dagRunDetails.statusLabel, nodes: [.dagRunDetails.nodes[] | {id: .step.id, statusLabel: .statusLabel, finishedAt: .finishedAt}]}'

echo "Out-of-band caller completes gate_a directly without HCTL receipt:"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-inline-probe/run-inline-001/human-tasks/gate_a/complete" \
    -H "Content-Type: application/json" \
    -d '{"obligation_id":"oob-untrusted","attempt_generation":99,"verdict":"accepted","receipt_id":"rcpt-oob"}' -w "\nHTTP_STATUS:%{http_code}\n"
wait_for_step_status "hctl-inline-probe" "run-inline-001" "gate_b" "waiting"

echo "HCTL periodic readback sees gate_a=succeeded and gate_b=waiting while HCTL Obligation(gate_a) is still active:"
curl -sS "${BASE_URL}/dag-runs/hctl-inline-probe/run-inline-001" | jq '{dagRunId: .dagRunDetails.dagRunId, statusLabel: .dagRunDetails.statusLabel, nodes: [.dagRunDetails.nodes[] | {id: .step.id, statusLabel: .statusLabel, startedAt: .startedAt, finishedAt: .finishedAt, prompt: .step.humanTask.prompt}]}'

echo "If HCTL later attempts to submit its legitimate completion for gate_a, Dagu rejects with 409 Conflict:"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-inline-probe/run-inline-001/human-tasks/gate_a/complete" \
    -H "Content-Type: application/json" \
    -d '{"obligation_id":"obl-inline-1","attempt_generation":1,"verdict":"accepted","receipt_id":"rcpt-legit"}' -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- 3.6 Reschedule / Edit-Retry creates a NEW dagRunId (cannot mutate bound dagRunId) ---"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/reschedule" \
    -H "Content-Type: application/json" \
    -d '{"dagRunId":"run-file-001"}' -w "\nHTTP_STATUS:%{http_code}\n"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-file-001/reschedule" \
    -H "Content-Type: application/json" \
    -d '{"dagRunId":"run-file-rescheduled-002"}' -w "\nHTTP_STATUS:%{http_code}\n"

echo "--- 3.7 ACK-unknown + immediate kill -KILL crash recovery stress test ---"
curl -sS -X POST "${BASE_URL}/dags/hctl-file-probe/start" -H "Content-Type: application/json" \
    -d '{"dagRunId":"run-crash-001"}' -w "\nHTTP_STATUS:%{http_code}\n"
wait_for_step_status "hctl-file-probe" "run-crash-001" "gate_a" "waiting"

echo "Complete gate_a on run-crash-001 and immediately kill -KILL dagu start-all:"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-crash-001/human-tasks/gate_a/complete" \
    -H "Content-Type: application/json" \
    -d '{"obligation_id":"obl-crash-1","attempt_generation":1,"verdict":"accepted","receipt_id":"rcpt-crash-001"}' -w "\nHTTP_STATUS:%{http_code}\n"
kill9_dagu
start_dagu

echo "Re-deliver identical gate_a completion immediately after restart (ACK-unknown retry -> 200 alreadyCompleted=true, queued=false):"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-crash-001/human-tasks/gate_a/complete" \
    -H "Content-Type: application/json" \
    -d '{"obligation_id":"obl-crash-1","attempt_generation":1,"verdict":"accepted","receipt_id":"rcpt-crash-001"}' -w "\nHTTP_STATUS:%{http_code}\n"

wait_for_step_status "hctl-file-probe" "run-crash-001" "gate_b" "waiting"
echo "Readback run-crash-001 after queue recovery advances gate_b to waiting:"
curl -sS "${BASE_URL}/dag-runs/hctl-file-probe/run-crash-001" | jq '{dagRunId: .dagRunDetails.dagRunId, statusLabel: .dagRunDetails.statusLabel, nodes: [.dagRunDetails.nodes[] | {id: .step.id, statusLabel: .statusLabel, prompt: .step.humanTask.prompt}]}'
echo "Calling /human-tasks/resume once gate_b is waiting returns HTTP 409:"
curl -sS -X POST "${BASE_URL}/dag-runs/hctl-file-probe/run-crash-001/human-tasks/resume" -w "\nHTTP_STATUS:%{http_code}\n"
EOF
```

### 实测原始输出（macOS aarch64，Dagu 2.15.1）

```text
==================================================================
0. VERSION & HEALTH
==================================================================
2.15.1
{"status":"healthy","timestamp":"2026-10-19T20:26:02+08:00","uptime":0,"version":"2.15.1"}

HTTP_STATUS:200
==================================================================
1A. DAG VALIDATION (VALID / MISSING DEP / CYCLE)
==================================================================
--- POST /api/v1/dags/validate (valid 2-step human.task DAG) ---
{"dag":{"env":[],"histRetentionDays":30,"maxActiveRuns":1,"maxActiveSteps":10,"name":"validated-dag","params":[],"preconditions":[],"schedule":[],"steps":[{"commands":[],"executorConfig":{"config":{}},"humanTask":{"form":{"additionalProperties":false,"properties":{"attempt_generation":{"type":"integer"},"obligation_id":{"type":"string"},"receipt_id":{"type":"string"},"verdict":{"enum":["accepted","rejected"],"type":"string"}},"required":["obligation_id","attempt_generation","verdict","receipt_id"],"type":"object"},"prompt":"Obligation gate_a waiting for HCTL completion"},"id":"gate_a","name":"gate_a","outputs":[{"name":"attempt_generation","type":"json"},{"name":"obligation_id","type":"string"},{"name":"receipt_id","type":"string"},{"name":"verdict","type":"string"}],"preconditions":[]},{"commands":[],"depends":["gate_a"],"executorConfig":{"config":{}},"humanTask":{"form":{"additionalProperties":false,"properties":{"attempt_generation":{"type":"integer"},"obligation_id":{"type":"string"},"receipt_id":{"type":"string"},"verdict":{"enum":["accepted","rejected"],"type":"string"}},"required":["obligation_id","attempt_generation","verdict","receipt_id"],"type":"object"},"prompt":"Obligation gate_b after gate_a (${steps.gate_a.outputs.receipt_id})"},"id":"gate_b","name":"gate_b","outputs":[{"name":"attempt_generation","type":"json"},{"name":"obligation_id","type":"string"},{"name":"receipt_id","type":"string"},{"name":"verdict","type":"string"}],"preconditions":[]}],"type":"graph"},"errors":null,"valid":true}

HTTP_STATUS:200
--- POST /api/v1/dags/validate (missing dependency) ---
{"dag":{"env":[],"histRetentionDays":30,"maxActiveRuns":1,"maxActiveSteps":10,"name":"validated-dag","params":[],"preconditions":[],"schedule":[],"steps":[{"commands":[],"depends":["missing_step"],"executorConfig":{"config":{}},"humanTask":{"prompt":"test"},"id":"s1","name":"s1","preconditions":[]}],"type":"graph"},"errors":["field 'depends': step s1 depends on non-existent step missing_step (value: missing_step)"],"valid":false}

HTTP_STATUS:200
--- POST /api/v1/dags/validate (cycle s1 <-> s2: passes static validate!) ---
{"dag":{"env":[],"histRetentionDays":30,"maxActiveRuns":1,"maxActiveSteps":10,"name":"validated-dag","params":[],"preconditions":[],"schedule":[],"steps":[{"commands":[],"depends":["s2"],"executorConfig":{"config":{}},"humanTask":{"prompt":"step 1"},"id":"s1","name":"s1","preconditions":[]},{"commands":[],"depends":["s1"],"executorConfig":{"config":{}},"humanTask":{"prompt":"step 2"},"id":"s2","name":"s2","preconditions":[]}],"type":"graph"},"errors":null,"valid":true}

HTTP_STATUS:200
--- POST /api/v1/dag-runs (cycle s1 <-> s2 fails at runtime plan build with HTTP 500) ---
{"code":"internal_error","message":"failed to start dag-run: internal_error: DAG start process exited before publishing status: exit status 1"}

HTTP_STATUS:500
==================================================================
1B. DAG SUBMISSION & START (FILE-BACKED + INLINE + ENQUEUE)
==================================================================
--- POST /api/v1/dags & PUT /api/v1/dags/hctl-file-probe/spec ---
{"name":"hctl-file-probe"}

HTTP_STATUS:201
{"errors":null}

HTTP_STATUS:200
--- POST /api/v1/dags/hctl-file-probe/start (run-file-001) ---
{"dagRunId":"run-file-001"}

HTTP_STATUS:200
--- Duplicate dagRunId rejection on POST /api/v1/dags/hctl-file-probe/start ---
{"code":"already_exists","message":"dag-run ID run-file-001 already exists for DAG hctl-file-probe"}

HTTP_STATUS:409
--- POST /api/v1/dag-runs (inline spec: run-inline-001) ---
{"dagRunId":"run-inline-001"}

HTTP_STATUS:200
--- POST /api/v1/dag-runs/enqueue (inline spec enqueue: run-enqueue-001) ---
{"dagRunId":"run-enqueue-001"}

HTTP_STATUS:200
{
  "dagRunId": "run-enqueue-001",
  "status": 7,
  "statusLabel": "waiting",
  "triggerType": "manual",
  "nodes": [
    {
      "id": "gate_a",
      "status": 7,
      "statusLabel": "waiting"
    },
    {
      "id": "gate_b",
      "status": 0,
      "statusLabel": "not_started"
    }
  ]
}
==================================================================
1C. PAUSE & RESUME CHECKS
==================================================================
--- Non-existent run-level pause/suspend endpoints ---
404 page not found
HTTP_STATUS:404
404 page not found
HTTP_STATUS:404
--- DAG-level suspend POST /api/v1/dags/hctl-file-probe/suspend ---

HTTP_STATUS:200
{
  "name": "hctl-file-probe",
  "suspended": true
}

HTTP_STATUS:200
{
  "name": "hctl-file-probe",
  "suspended": false
}
--- POST /api/v1/dag-runs/hctl-file-probe/run-file-001/human-tasks/resume while step still waiting ---
{"code":"conflict","message":"DAG-run hctl-file-probe:run-file-001 still has manual steps waiting for input"}

HTTP_STATUS:409
==================================================================
1D. CANCEL (STOP / DEQUEUE) ACROSS RUN STATES
==================================================================
--- 1D.1 Cancel/mutate a Waiting (7) human.task run via /stop, /dequeue, /reject, /approve, /push-back, /status ---
{"code":"not_running","message":"DAG is not running"}

HTTP_STATUS:400
{"code":"bad_request","message":"DAGRun status is not queued: waiting"}

HTTP_STATUS:400
{"code":"bad_request","message":"human-task state requires the human-task completion API: step gate_a is a human task"}

HTTP_STATUS:400
{"code":"bad_request","message":"human-task state requires the human-task completion API: step gate_a is a human task"}

HTTP_STATUS:400
{"code":"bad_request","message":"human-task state requires the human-task completion API: step gate_a is a human task"}

HTTP_STATUS:400
{"code":"bad_request","message":"dag-run ID run-enqueue-001 for DAG hctl-inline-probe is still active"}

HTTP_STATUS:400
--- 1D.2 Cancel a Queued (5) run via GET /dequeue and a Running (1) run via POST /stop (plus direct start vs enqueue on max_active_runs=1) ---
{"name":"hctl-running-cancel"}

HTTP_STATUS:201
{"errors":null}

HTTP_STATUS:200
{"dagRunId":"run-cancel-running-001"}

HTTP_STATUS:200
Direct start (POST /dags/{name}/start and POST /dag-runs) bypasses max_active_runs=1:
{"dagRunId":"run-direct-start-002"}

HTTP_STATUS:200
{
  "dagRunId": "run-direct-start-002",
  "statusLabel": "running"
}

HTTP_STATUS:200
Enqueue (POST /dag-runs/enqueue) respects max_active_runs=1 and stays queued while run-cancel-running-001 is running:
{"dagRunId":"run-cancel-queued-002"}

HTTP_STATUS:200

HTTP_STATUS:200
{"code":"not_found","message":"dag-run ID run-cancel-queued-002 not found for DAG hctl-running-cancel"}

HTTP_STATUS:404

HTTP_STATUS:200
{
  "dagRunId": "run-cancel-running-001",
  "status": 3,
  "statusLabel": "aborted",
  "nodes": [
    {
      "id": "long_step",
      "status": 3,
      "statusLabel": "aborted"
    }
  ]
}
--- 1D.3 Waiting (7) human.task run does NOT occupy maxActiveRuns=1 queue slot when subsequent runs are enqueued ---
{"dagRunId":"run-enqueue-002"}

HTTP_STATUS:200
{
  "dagRunId": "run-enqueue-002",
  "statusLabel": "waiting",
  "gateA": "waiting"
}
==================================================================
2. HUMAN.TASK WAITING, COMPLETION, RESTART PERSISTENCE, IDEMPOTENCY & OUTPUTS
==================================================================
--- 2.1 Readback initial Waiting state on run-file-001 ---
{
  "dagRunId": "run-file-001",
  "status": 7,
  "statusLabel": "waiting",
  "nodes": [
    {
      "id": "gate_a",
      "status": 7,
      "statusLabel": "waiting",
      "doneCount": 1,
      "startedAt": "2026-10-19T20:26:02+08:00",
      "finishedAt": "",
      "prompt": "Obligation gate_a waiting for HCTL completion",
      "outputs": [
        {
          "name": "attempt_generation",
          "type": "json"
        },
        {
          "name": "obligation_id",
          "type": "string"
        },
        {
          "name": "receipt_id",
          "type": "string"
        },
        {
          "name": "verdict",
          "type": "string"
        }
      ]
    },
    {
      "id": "gate_b",
      "status": 0,
      "statusLabel": "not_started",
      "doneCount": 0,
      "startedAt": "",
      "finishedAt": "",
      "prompt": "Obligation gate_b after gate_a (${steps.gate_a.outputs.receipt_id})",
      "outputs": [
        {
          "name": "attempt_generation",
          "type": "json"
        },
        {
          "name": "obligation_id",
          "type": "string"
        },
        {
          "name": "receipt_id",
          "type": "string"
        },
        {
          "name": "verdict",
          "type": "string"
        }
      ]
    }
  ]
}

HTTP_STATUS:405
--- 2.2 Server restart while run-file-001 is Waiting at gate_a ---
--- Readback run-file-001 after Dagu server restart ---
{
  "dagRunId": "run-file-001",
  "status": 7,
  "statusLabel": "waiting",
  "nodes": [
    {
      "id": "gate_a",
      "status": 7,
      "statusLabel": "waiting",
      "startedAt": "2026-10-19T20:26:02+08:00"
    },
    {
      "id": "gate_b",
      "status": 0,
      "statusLabel": "not_started",
      "startedAt": ""
    }
  ]
}
--- 2.3 Invalid form input rejected with HTTP 400 ---
{"code":"bad_request","message":"invalid input for human task step \"gate_a\": human task form validation failed: parameter validation failed: validating root: required: missing properties: [\"obligation_id\" \"attempt_generation\" \"receipt_id\"]"}

HTTP_STATUS:400
--- 2.4 Valid completion of gate_a ---
{"alreadyCompleted":false,"dagName":"hctl-file-probe","dagRunId":"run-file-001","queued":true,"remainingWaitingSteps":0,"stepId":"gate_a"}

HTTP_STATUS:200
--- Readback run-file-001 after gate_a succeeded and gate_b entered waiting ---
{
  "dagRunId": "run-file-001",
  "status": 7,
  "statusLabel": "waiting",
  "nodes": [
    {
      "id": "gate_a",
      "status": 4,
      "statusLabel": "succeeded",
      "doneCount": 1,
      "startedAt": "2026-10-19T20:26:02+08:00",
      "finishedAt": "2026-10-19T12:26:07Z",
      "prompt": "Obligation gate_a waiting for HCTL completion",
      "completedBy": null
    },
    {
      "id": "gate_b",
      "status": 7,
      "statusLabel": "waiting",
      "doneCount": 1,
      "startedAt": "2026-10-19T20:26:10+08:00",
      "finishedAt": "",
      "prompt": "Obligation gate_b after gate_a (rcpt-001)",
      "completedBy": null
    }
  ]
}
--- 2.5 Idempotent repeat of gate_a completion (same input -> HTTP 200 alreadyCompleted=true) ---
{"alreadyCompleted":true,"dagName":"hctl-file-probe","dagRunId":"run-file-001","queued":false,"remainingWaitingSteps":1,"stepId":"gate_a"}

HTTP_STATUS:200
--- 2.6 Conflicting repeat of gate_a completion (different input -> HTTP 409 conflict) ---
{"code":"conflict","message":"human task step \"gate_a\" was already completed with different input"}

HTTP_STATUS:409
--- 2.7 Complete gate_b and readback final Succeeded state + outputs ---
{"alreadyCompleted":false,"dagName":"hctl-file-probe","dagRunId":"run-file-001","queued":true,"remainingWaitingSteps":0,"stepId":"gate_b"}

HTTP_STATUS:200
{
  "dagRunId": "run-file-001",
  "status": 4,
  "statusLabel": "succeeded",
  "startedAt": "2026-10-19T20:26:13+08:00",
  "finishedAt": "2026-10-19T20:26:13+08:00",
  "nodes": [
    {
      "id": "gate_a",
      "status": 4,
      "statusLabel": "succeeded",
      "startedAt": "2026-10-19T20:26:02+08:00",
      "finishedAt": "2026-10-19T12:26:07Z"
    },
    {
      "id": "gate_b",
      "status": 4,
      "statusLabel": "succeeded",
      "startedAt": "2026-10-19T20:26:10+08:00",
      "finishedAt": "2026-10-19T12:26:10Z"
    }
  ]
}
{
  "metadata": {
    "attemptId": "",
    "completedAt": "0001-01-01T00:00:00Z",
    "dagName": "",
    "dagRunId": "",
    "params": "",
    "status": ""
  },
  "outputs": {}
}
==================================================================
3. ENGINE SELF-ADVANCE / RETRY VS HCTL READBACK DIVERGENCE CHECKS
==================================================================
--- 3.1 Engine blocks retry_policy / repeat_policy / parallel on human.task at spec validation ---
{
  "valid": false,
  "errors": [
    "field 'steps': field 'retry_policy': action human.task does not support retry_policy (value: map[limit:2])"
  ]
}
{
  "valid": false,
  "errors": [
    "field 'steps': field 'repeat_policy': action human.task does not support repeat_policy (value: map[repeat:true])"
  ]
}
{
  "valid": false,
  "errors": [
    "field 'steps': field 'parallel': action human.task does not support parallel (value: map[items:[a b]])"
  ]
}
--- 3.2 Engine blocks direct step retry on human.task (even after run succeeded) ---
{"code":"conflict","message":"human task step \"gate_a\" cannot be retried directly"}

HTTP_STATUS:409
--- 3.3a Engine blocks run retry while run is Waiting on human.task ---
{"code":"conflict","message":"DAG-run hctl-inline-probe:run-inline-001 is waiting and cannot be retried"}

HTTP_STATUS:409
--- 3.3b Full run retry after run Succeeded does NOT re-enter waiting on completed human.task steps (observed across 2s/4s post-retry ticks) ---

HTTP_STATUS:200
post-retry t+2s:
{
  "dagRunId": "run-file-001",
  "statusLabel": "succeeded",
  "nodes": [
    {
      "id": "gate_a",
      "statusLabel": "succeeded",
      "doneCount": 1
    },
    {
      "id": "gate_b",
      "statusLabel": "succeeded",
      "doneCount": 1
    }
  ]
}
post-retry t+4s:
{
  "dagRunId": "run-file-001",
  "statusLabel": "succeeded",
  "nodes": [
    {
      "id": "gate_a",
      "statusLabel": "succeeded",
      "doneCount": 1
    },
    {
      "id": "gate_b",
      "statusLabel": "succeeded",
      "doneCount": 1
    }
  ]
}
--- 3.4 Engine blocks manual step status PATCH on human.task (even after run succeeded) ---
{"code":"bad_request","message":"human task step gate_a must be completed through the human-task API"}

HTTP_STATUS:400
--- 3.5 Out-of-band human.task completion (UI/CLI bypasses HCTL outbox) -> Readback Divergence ---
Before out-of-band completion (run-inline-001 is waiting at gate_a):
{
  "dagRunId": "run-inline-001",
  "statusLabel": "waiting",
  "nodes": [
    {
      "id": "gate_a",
      "statusLabel": "waiting",
      "finishedAt": ""
    },
    {
      "id": "gate_b",
      "statusLabel": "not_started",
      "finishedAt": ""
    }
  ]
}
Out-of-band caller completes gate_a directly without HCTL receipt:
{"alreadyCompleted":false,"dagName":"hctl-inline-probe","dagRunId":"run-inline-001","queued":true,"remainingWaitingSteps":0,"stepId":"gate_a"}

HTTP_STATUS:200
HCTL periodic readback sees gate_a=succeeded and gate_b=waiting while HCTL Obligation(gate_a) is still active:
{
  "dagRunId": "run-inline-001",
  "statusLabel": "waiting",
  "nodes": [
    {
      "id": "gate_a",
      "statusLabel": "succeeded",
      "startedAt": "2026-10-19T20:26:02+08:00",
      "finishedAt": "2026-10-19T12:26:21Z",
      "prompt": "Obligation gate_a waiting for HCTL completion"
    },
    {
      "id": "gate_b",
      "statusLabel": "waiting",
      "startedAt": "2026-10-19T20:26:22+08:00",
      "finishedAt": "",
      "prompt": "Obligation gate_b after gate_a (rcpt-oob)"
    }
  ]
}
If HCTL later attempts to submit its legitimate completion for gate_a, Dagu rejects with 409 Conflict:
{"code":"conflict","message":"human task step \"gate_a\" was already completed with different input"}

HTTP_STATUS:409
--- 3.6 Reschedule / Edit-Retry creates a NEW dagRunId (cannot mutate bound dagRunId) ---
{"code":"already_exists","message":"dag-run ID run-file-001 already exists for DAG hctl-file-probe"}

HTTP_STATUS:409
{"dagRunId":"run-file-rescheduled-002","queued":true}

HTTP_STATUS:200
--- 3.7 ACK-unknown + immediate kill -KILL crash recovery stress test ---
{"dagRunId":"run-crash-001"}

HTTP_STATUS:200
Complete gate_a on run-crash-001 and immediately kill -KILL dagu start-all:
{"alreadyCompleted":false,"dagName":"hctl-file-probe","dagRunId":"run-crash-001","queued":true,"remainingWaitingSteps":0,"stepId":"gate_a"}

HTTP_STATUS:200
Re-deliver identical gate_a completion immediately after restart (ACK-unknown retry -> 200 alreadyCompleted=true, queued=false):
{"alreadyCompleted":true,"dagName":"hctl-file-probe","dagRunId":"run-crash-001","queued":false,"remainingWaitingSteps":0,"stepId":"gate_a"}

HTTP_STATUS:200
Readback run-crash-001 after queue recovery advances gate_b to waiting:
{
  "dagRunId": "run-crash-001",
  "statusLabel": "waiting",
  "nodes": [
    {
      "id": "gate_a",
      "statusLabel": "succeeded",
      "prompt": "Obligation gate_a waiting for HCTL completion"
    },
    {
      "id": "gate_b",
      "statusLabel": "waiting",
      "prompt": "Obligation gate_b after gate_a (rcpt-crash-001)"
    }
  ]
}
Calling /human-tasks/resume once gate_b is waiting returns HTTP 409:
{"code":"conflict","message":"DAG-run hctl-file-probe:run-crash-001 still has manual steps waiting for input"}

HTTP_STATUS:409
```

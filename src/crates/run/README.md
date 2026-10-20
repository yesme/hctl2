# Run：施工图、生命周期与冻结施工清单

本模块拥有 Workflow Revision、Engine Deployment、批准版本、Workflow–Task 关联，以及 Run 生命周期与不可变 Manifest。共享 Store 保存材料、版本与外部意图；Task 占用更新由 Task 提供事务片段，与 Run 一起提交。施工图批准不等于启动许可。

第 1 包实现施工图，第 3 包实现下文生命周期；真实引擎接线归第 2 包，Obligation、Seat、Attempt 与派工归第 4a 包。本模块不启动 Dagu 服务或 harness。

设计权威：[Run 约束](../../../docs/design/spec/run.md)、[运行默认值](../../../docs/design/delivery.md#运行默认值)。施工图类型权威在 `model.rs` 与 `commands.rs`；运行类型与准入在 `lifecycle/`。

## 命令

```text
hctl2 --root <root> --json workflow list <project>
hctl2 --root <root> --json workflow list <project> --task <task>
hctl2 --root <root> --json workflow show <project> <workflow> [--revision <n>]
hctl2 --root <root> --json workflow register --input register.json --key <key>
hctl2 --root <root> --json workflow register --input register.json --key <key> --preview-token <token>
hctl2 --root <root> --json workflow compile --input compile.json --key <key>
hctl2 --root <root> --json workflow compile --input compile.json --key <key> --preview-token <token>
hctl2 --root <root> --json workflow approve --input approve.json --key <key>
hctl2 --root <root> --json workflow approve --input approve.json --key <key> --preview-token <token>
```

经真实 CLI、控制面认证 socket、公共 preview/submit 进入本模块。`--input` 是 action 对象；CLI 补入与子命令一致的 kind，不一致则拒绝。公共 payload 是 `{key, action}`，操作名 `workflow.register|compile|approve`，command id 为 `workflow:<key>`。同 key 绑定原输入与 actor，响应丢失重试返回原结果；不增加版本、关联或批准。确认 token 绑定原负载，提交重新准备并比较计划，再在一个事务里复核全部读到的版本。

登记示例：

```json
{
  "project_id": "P",
  "project_version": 1,
  "workflow_id": "delivery",
  "supersedes": null,
  "task": null,
  "workflow": {
    "schema": "hctl2.workflow.v1",
    "profile": "hctl2.passive.v1",
    "context": "背景文字，不参与结构编译，但参与版本摘要与读回",
    "graph": {
      "nodes": [{
        "id": "build",
        "outputs": {"code": "change_set_revision"},
        "operation": {
          "kind": "external",
          "criterion": {"kind": "mechanical", "output": "code", "fact": {"kind": "ci", "check": "test"}},
          "seats": [],
          "gate": null
        }
      }],
      "edges": []
    }
  },
  "snapshot": {
    "decided": [{"id": "D1", "text": "交出实现与测试", "delivery_obligation": true}],
    "unresolved": [],
    "out_of_scope": []
  },
  "provenance": {
    "snapshot_source": {"kind": "direct_text"},
    "source_chain": [],
    "production": {"kind": "none"}
  }
}
```

`compile.json` 为 `{project_id, project_version, workflow_id, revision}`；`approve.json` 另加精确 `readback` Reference。新版本须用 supersedes 声明当前版号；旧正文与批准不改写。task 是同 Project 的精确 Task Reference，与版本一起准入关联。`show` 返回原图、判据、边、提示、部署、批准；`list` 只列本 Project 指针。

`list --task <task>` 从 Run 保存的关联投影该 Task 的全部施工图历史版本，返回 workflow_id、revision、revision_reference 与登记时的精确 task。按稳定 Task 身份检索，Task 升版、施工图最新版本解绑或转绑都不隐藏旧关联；按图 id、数字版号排序，不含别的 Project。取到入口后用 `show <project> <workflow_id> --revision <revision>` 读原文，继续编辑时以当前图的版号声明 supersedes。查询不另写 Task 计划，公共查询 payload 为 `{project_id, task_id?}`；Task 不存在则拒绝。

`startable` 只表示版本已批准，不替代启动时 Task、候选、预算与租约准入。后续 Run 启动必须调用 `require_approved`。

## 规范 JSON 与 Profile

所有对象拒绝未知字段；执行种类为闭集，不透传 Dagu DSL。内部标记枚举的无字段种类也用空结构体变体，使 serde 对多余字段报错；合法 JSON 形状和规范摘要不变。补齐缺省值后，使用现有 RFC 8785 实现保存规范 JSON、计算 SHA-256。摘要不是 Git 对象编号。

- Workflow：schema、profile、context、readback、graph。读回缺省启用，缺省回避所有产出调用的 Worker Profile。显式 `readback: {required:false, different_worker_profile:true}` 才关闭；模板无特权。
- Graph：nodes、edges。Node 是 id、outputs、operation；node/output/Seat id 使用 ASCII 字母、数字与下划线。输出种类：change_set_revision、artifact_revision、verdict、receipt、mechanical_fact。
- external 节点须声明 criterion：绑定本节点输出的机械事实（ci / pr_merged / path_digest），或 Gate 加证据下限。机械事实仍须运行时由工具回读精确版本；可派发不等于已达成。
- 结构件为 fork、join、switch（cases）、loop（body、max_reworks、on_exhausted）、dynamic_fork、timer_wait（seconds）、noop、transform（fields）。这些由未来 control 归约器处理，不能执行命令或调用模型。
- Edge：from、to、可选 artifact（output、kind）、outcome（缺省 success，也可 failure）、可选 case。switch 出边必须引用声明内的 case，其他边不得带 case。产物引用须在本层静态图解析到源节点的同类型输出，可以尚未产生；两外部节点直接相连必须引用产物。HCTL 自己先检环。
- Gate：quorum、distinct_worker_profiles。Seat：id、required、profession、evidence、required_skills、review_subject_node、template。同一身份的相同声明重复引用只算一个席位，编译映射去重；同身份声明互相冲突则拒绝。法定票数超过不同必需席位数拒绝。相同模板不证明实际同键；声明层同要求、材料且不要求 Profile 互异只提示「可能合成一票」、缺失信息和可选处理。实际计票键仍是实际 Worker Profile 摘要与交付条目加必需 Skill 的内容摘要，排除消费者身份、计量与保留字段；本包不做运行时计票。

## 有界展开与引擎定义

编译器在分配展开图前检查资源上限 16384 个检查点，超过时报 WORKFLOW_EXPANSION_LIMIT，不能因一个很大的有限轮数耗尽控制面内存。

loop 是静态结构，不是回边。max_reworks 缺省 2，编译前补齐并冻结，展开初始轮 r0 与有限返工轮 r1/r2，每轮独立检查点；未来每次进入创建新 Obligation。on_exhausted 是 fail 或 request；用满轮数后本轮合格仍可成功，只有继续返工才走失败边或请人。

dynamic_fork 冻结 template、候选 selection 引用与职责、max_cardinality、budget、selection（all / first count）、permission_ceiling。缺项、空项、越界拒绝；批准前按最大基数展开所有槽，运行时只选冻结槽的子集，不扩图。

Compiled 固定 compiler=hctl2.workflow.v1、dagu_version=2.15.1，含 checkpoints（独立 id、原节点、depends、role、activation）、frozen_rework_limits、warnings、yaml。role 区分节点与结构出口；activation 显式记录每个嵌套 loop 的轮次与 fork 的槽号，后续归约器不得从步骤名猜授权身份。**Manifest 须直接冻结 frozen_rework_limits，不能启动时重新套缺省值。** 原 Revision 保留有类型的边和条件，部署映射不替代它。只改 context 时版本摘要变化，需新读回，编出的检查点、边与席位不变。

YAML 使用 JSON 子集，每步固定 human.task（包括结构件的被动推进点），表单必填 obligation_id、attempt_generation、verdict、receipt_id。结构入口/出口、未选分支、无需再返工的预展开轮次，由后续 Run reducer 按冻结结构生成相应凭证再推进，不能派成模型义务，适配器不能自行完成它们。本包只生成与校验，不实现推进。没有 repeat、foreach、子 DAG、跨 dagRun 或任意 command/script/HTTP/agent/Harness。

NativeDagu 只调用锁定二进制的 version、validate，各最多 20 秒且输出有上限。临时 HOME/DAGU_HOME 私有且用完删除，不配置 auth、不起服务。Dagu 文件入口不含顶层 name，从文件名取名。生产取既有安装根 libexec/hctl2/dagu；开发测试可显式设 HCTL2_DAGU_BIN，版本仍须完全匹配。

## 清单、来源与读回

snapshot 只含 decided、unresolved、out_of_scope；decided 条目固定 id、text、delivery_obligation，其余为文字数组。provenance 不进入读回 Bundle：snapshot_source 是冻结消息（room_id、event）或 direct_text；直接文本提交者与摘要在 Revision 的 author/snapshot_digest。source_chain 列真实冻结消息，事件须经 Room 命令保存且绑定 id 与 room_id 一致。按实际事件收集所有来源 Room，直接文本不免检。

production 为非空 invocations references（精确 room_invocation 引用）、显式 none、或 missing。普通空集合拒绝，missing 可保存但要求 Profile 回避时不能批准。每版来源冻结；批准逐一回读真实产出调用的 spec/profile，不取最近一次调用。

`readback::Readbacks` 是第 7 包的可信端口，不是公开写入接口。Readback 固定 workflow_digest、snapshot_digest、invocation、selection、bundle_digest、body_digest；InvocationEvidence 固定新起会话证明、实际 Sealed ExecutionSpec、实际送达 Sealed Bundle、selection、可选 source_room。第 7 包须在真实派工送达后写，不能从模型所报摘要或旧 Bundle 拼装；本包用内部测试替身。**尚无真实读回生产入口**，approve 只收引用，不收自报证明。

批准检查读回正文已准入；登记、读回、实际交付三处图/快照原摘要一致；Spec 冻结 Bundle 与实际交付相同；Bundle 只含图和清单且是精确 bytes 的 inline/pointer，无压缩/召回替代；无任何 Room 消息读取权限；新起、无 Run；selection 不在任何来源 Room 的批准时名册内。无来源 Room 仅名册项不适用。开启 Profile 回避时与每个真实产出调用比较；未知、缺失不当通过，显式 none 记无需比较。证据和名册版本进入批准事务 CAS。

## Store 布局

下列施工图记录全部在 Project scope；Task 关联由 Task 投影。

| kind / id | 版本与内容 |
| --- | --- |
| workflow / workflow_id | current pointer，version 为 Revision 序号 |
| workflow_revision / workflow_id:revision | immutable version=1；图、清单、来源、actor、摘要、提示、Task |
| workflow_task_link / workflow_id:revision | immutable version=1；精确 Workflow 与 Task；Task 只投影 |
| engine_deployment / canonical_digest | immutable version=1；精确 Workflow 引用与摘要、Profile、adapter/version、固定编译器/引擎、映射、YAML |
| workflow_deployment / workflow_id:revision | 当前 Deployment 指针；升级编译器或适配器产生新部署，不覆盖旧部署 |
| workflow_approval / workflow_id:revision | approval version；精确 Revision、Deployment、读回依据、actor |
| workflow_command / sha256(key) | 原 actor、Input、Plan 与幂等结果 |
| workflow_readback / id | 第 7 包写的 immutable Readback，加正文 material |
| workflow_invocation_evidence / invocation_id | 第 7 包送达后写的 immutable InvocationEvidence；本包只读 |

原图、清单先保存规范 JSON 候选材料，在同一个 Store submit 中准入材料、不可变 Revision、pointer、Task link 和幂等结果。编译产物亦同，YAML 为独立 material。保存失败不准入，事务失败不留关联；没有新数据库或文件协议。部署本身的规范摘要在 Record.revision_digest，YAML 精确摘要在 material.byte_digest。

## 验证

```bash
./src/buck2 test root//crates/run:domain_test
```

目标覆盖真实 CLI → control → Run 的预览/确认、随包 Dagu 原生 validate、内部读回测试替身，以及 schema/Profile、判据、产物边、检环、有界 loop/fork、票数提示、关联原子性、幂等、新版本、三处摘要、权限、名册和实际 Profile。无平台或模型凭据。退回检环、产物落地、实际送达摘要比较的三份红灯输出随 PR 附上。

## Run 命令与启动授权

```text
hctl2 --root <root> --json run list <project>
hctl2 --root <root> --json run show <project> <run>
hctl2 --root <root> --json run preview --input start.json --key <key>
hctl2 --root <root> --json run start --input start.json --key <key> --preview-token <token>
hctl2 --root <root> --json run pause --input action.json --key <key> [--preview-token <token>]
hctl2 --root <root> --json run resume --input action.json --key <key> [--preview-token <token>]
hctl2 --root <root> --json run cancel --input action.json --key <key> [--preview-token <token>]
hctl2 --root <root> --json run replace --input replace.json --key <key> [--preview-token <token>]
hctl2 --root <root> --json run place --input place.json --key <key> [--preview-token <token>]
```

没有 token 的写命令只预览；`run preview` 是 start 预览的别名。`start.json` 为 `{ "spec": <lifecycle::Start> }`，暂停、恢复、取消为 `{project_id, run_id, version}`，替代另加新 `spec`，调整放置另加精确 `destination` Reference。CLI 补入 kind；公共负载 `{key, action}`，操作 `run.start|pause|resume|cancel|replace|place`，command id `run:<key>`。没有公共 finish、complete 或引擎回报接口。

Start 必须明确 Project 版本、独立 Run id、批准的 Workflow 版本、零或一个 Task Revision、Repo、40 位基线 SHA、精确根 Context Manifest 与摘要、接受的 workflow_engine_binding、每个编译后外部席位的候选、预算、期限、集成方式与放置范围。基线在这里冻结输入；Git 提交是否存在由后续工具读取，不把语法检查称为 Git 回读。引擎绑定的真实生产入口归第 2 包。

带 Task 时，控制面先经现有任务源端口刷新源快照，再准备预览；确认时重新刷新并比较计划，最后事务内复核全部版本。只接受开放 Project、开放且未归档的 Task、已采纳且仍为 current 的完整契约、匹配 Repo、无占用。`blocked_by` 中未关闭的阻塞方列在 `blockers_requiring_confirmation`，等人用预览 token 确认；父子关系本身不作阻塞。源故障如实拒绝。原幂等输入响应丢失后的重放不重新访问平台，返回原 Run。

Manifest 冻结精确 Project/Task/Workflow/批准/Deployment/Repo/根上下文引用、基线、批准的读回依据、Project 选择策略摘要、逐席位选择、候选 Profile 摘要、必需 Skill、权限、预算、期限、技术故障换候选策略、网络与秘密作用域、初始放置及获准放置范围、集成授权。`frozen_rework_limits` 直接复制批准部署编译时的数，不重新套缺省值或展开节点。

集成必填目标分支、target_kind、form、strategy；mode 缺省 automatic，也可 human_preview。预览和 show 都展示这些字段；人的启动确认就是对这份冻结授权的确认。目标保护、预期目标头和 Integration Intent 的实际准入属于后续集成包，不能从启动清单推断已合入。运行中换授权用 replace，不改原 Manifest。

选择复用 Participant 的接受名册与 Project 策略校验，不取 Room 名册。必需 Skill 是 Workflow 席位与 Profession 声明的并集。票数上界复用 gate-tally，检查全部获准候选；初始选定候选另查声明的 Profile 互异。`Pick.frozen_bundle` 可声明本席位未来交付的完整不可变内容约定，不代表已经向新消费者送达。未知内容只警告，不凭模板判同键；明确的内容按源条目、精确字节摘要与必需 Skill 引用求摘要，排除消费者身份、预算、保留和定位路径。第 4a 包交付前须调用 `validate_delivery_contract`，后续计票复用 `delivered_content_digest`；改正文不得沿用旧内容约定。

## 生命周期、事务与引擎端口

`State::allows` 按设计的状态转移表闭集校验。公开 resume 只接受 Paused，不借 Starting → Running 的引擎确认边绕过启动。终态不可恢复。Starting、Pausing、Cancelling 到默认 900000ms 或声明的墙钟期限只标需要关注，保留状态和 Task 占用，不自动取消、替代、重试或虚构引擎观察。

Start 在一个 Store submit 写入 Run、准入规范 Manifest 材料、逐席位选择、Task 占用、引擎启动意图及幂等结果。所有引用版本进入 CAS。Replace 先准备完整新清单，同一事务撤旧授权、撤所属活动、转移占用、新建 Run 与清单、入队新启动和旧停止意图；任何一步失败全部回滚。其他 Run 不被连带撤销。

取消或失败先撤资格与未发送意图。未知写入是否隔离不能按空集合或停止回执推断；保留占用并标关注，等控制面准入同一 Manifest、同一 Attempt 的 confined 或 silent 证明。第 4a 包必须在真实派工的同一事务中调用 `authorize_activity_in`，传入实际 Sealed ExecutionSpec 与 Bundle；它校验冻结选择、候选、权限、预算、内容及当前 Run 授权。Activity 和 IsolationProof 是可信内部接缝，本包只有内部夹具生产者，没有真实工作树隔离证明或物理租约撤销的生产入口。

`engine::Port` 接收已提交意图，测试使用有状态假端口。投递前用 Store 标 Unknown；响应丢失先回读，只有 NotApplied 且授权仍有效才按原幂等键投递。epoch、Manifest、绑定与 scope 不匹配的迟到观察不推进 Run；引擎观察没有“完成 Run”的种类。`engine_binding_generation` 是端口读回的独立引擎代次，不借控制面写者代次或操作 epoch 充数。生产控制面本包只运行超时标记器，不把未接的真实引擎调用说成成功。

`finish` 仅供冻结 Manifest 授权的内部 reducer。正常完成还须控制面证明必需义务已满足、Attempt 已终结或撤销、无未解决输出副作用、引用一致，并检查所属活动及 outbox；缺失/未知不通过。证明生产者归后续归约器包。完成把 Task 占用推进为 completion_pending，不自动完成 Task；Task 独立验收归后续包。失败或取消只有满足撤销边界才释放占用。

外部 Room/Task 引用只展示关联，不拥有 Run。`place` 只在冻结范围内调整向外结果投递位置，每次写不可变 before/after/actor/rule 审计，不改 Manifest，也不改 harness 工作目录、权限或执行身份。

新增 Project scope 记录：

| kind / id | 内容 |
| --- | --- |
| run / run_id | 生命周期、授权有效位、操作 epoch、墙钟进入时刻、关注与当前放置引用 |
| run_manifest / run_id | immutable version=1，准入规范材料与所有冻结字段 |
| run_selection / run:checkpoint:seat | immutable version=1，接受选择及 Manifest 来源 |
| run_command / sha256(key) | 原 actor、Input、Plan 和幂等结果 |
| run_engine_binding / run_id | 精确清单、部署、引擎执行 id/代次、分歧或关闭状态 |
| run_activity / attempt_id | 内部派工登记、资格与隔离证明引用 |
| run_isolation / proof_id | 内部可信物理隔离证明；第 4a 包负责真实生产 |
| run_placement / audit_id | 不可变放置调整审计 |
| run_finish_command / sha256(key) | 内部完成/失败的原输入与幂等结果 |

Task 模块的 `plan_run_occupancy` 返回保留材料和来源的 task_state 更新片段，不自行提交。调用方在同一事务复核旧版本；Manifest 不符、占用不属原 Run 或 completion_pending 都不当成可空闲槽。

```bash
./src/buck2 test root//crates/run:lifecycle_test
```

19 条用例覆盖真实 CLI → 认证 socket → control → Run 五命令、预览 token、版本与原子回滚、全部状态对、候选与票数、阻塞方、冻结授权、隔离未知、完成证明及超时。平台/批准读回/引擎/Attempt 隔离使用明示内部夹具，不需要平台或模型凭据。逐条证据与三次退回见 [生命周期实测](../../../docs/research/runtime/run-lifecycle-validation.md)。

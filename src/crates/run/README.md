# Run：施工图的登记、编译与批准

本模块拥有 Workflow Revision、Engine Deployment、批准版本、Workflow–Task 关联，使用共享 Store 与材料保存机制；Task 只投影关联。本包不创建 Run、Obligation、Seat 或 Attempt，不起 Dagu、不派工。后续包以这里的对象、命令与存储布局为准，不能把施工图登记当成启动许可。

设计权威：[Run 约束](../../../docs/design/spec/run.md)、[运行默认值](../../../docs/design/delivery.md#运行默认值)。类型权威在 `model.rs` 与 `commands.rs`。

## 命令

```text
hctl2 --root <root> --json workflow list <project>
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

`compile.json` 为 `{project_id, project_version, workflow_id, revision}`；`approve.json` 另加精确 `readback` Reference。新版本须用 supersedes 声明当前版号；旧正文与批准不改写。task 是同 Project 的精确 Task Reference，与版本一起准入关联。`show` 返回原图、判据、边、提示、部署、批准；`list` 只列本 Project 指针。`startable` 只表示版本已批准，不替代启动时 Task、候选、预算与租约准入。后续 Run 启动必须调用 `require_approved`。

## 规范 JSON 与 Profile

所有对象拒绝未知字段；执行种类为闭集，不透传 Dagu DSL。补齐缺省值后，使用现有 RFC 8785 实现保存规范 JSON、计算 SHA-256。摘要不是 Git 对象编号。

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

下列记录全部在 Project scope；Run 不写 Task 内部对象。

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

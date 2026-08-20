# fable-20260815a 备忘核销记录（对照 v0.9.1）

> 日期：2026-08-19<br>
> 对象：`.memo/fable-20260815a.md`（评审 v0.8.0 @ 36db492）↔ 当前草案 v0.9.1（3aa2950 及随本记录的修订）<br>
> 状态：Informative · 核销记录。v0.9.0/v0.9.1 两轮修订未按惯例附带「前次备忘跟进」小节，本文补记；只记录处置结论与现状依据，不新增合同。

## 阻断级（2/2 已解决）

| 编号 | 结论 | 现状依据 |
| --- | --- | --- |
| H1 无 Run 评审 Receipt 无写入者 | 已解决（按建议 (b)+(c) 组合） | 无 Run 路径不再要求评审 Receipt："不伪造只能由 Run 产生的 Gate 凭证；契约要求 HCTL 内部独立评审时必须使用带 Gate 的 Run，接受外部 SCM 评审时则引用可回读的精确外部证据"（`task.md` 设计正文「无 Run 的轻量路径」、`spec/connections.md`「验收与回流」）；纵向切片 A 第 5 步同步改写 |
| H2 Attempt 缺 `Pending → Lost` 边 | 已解决 | Attempt 合法边含 `Pending → Running/Failed/Lost/Cancelled`（`spec/run.md`「从节点到结果」）；契约测试矩阵含 "dispatch ACK 丢失允许 Pending→Lost 并用新 Attempt 恢复" |

## medium（24 条）

| 编号 | 结论 | 现状依据 |
| --- | --- | --- |
| M1 adapter 写成功 Receipt 违反单写者 | 已解决 | `spec/system.md`「外部权威副作用」："adapter 只投递并回读；只有在它确认目标、版本和结果后，control/core 的校验事务才能写成功 Receipt" |
| M2 AdoptTaskRevisionIntent 前置只覆盖外部来源 | 已解决 | `spec/task.md`「契约与来源」区分外部来源 adoption 与本地 Room/Project 提案 adoption 两种前置；矩阵含"本地 adoption 不要求伪造 TaskBinding" |
| M3 Request 字段两套词汇 | 已解决 | 字段合同只在 `spec/connections.md`「跨模块 Request 回路」定义一次；`spec/project.md` 明文"本模块不另建一套同义字段" |
| M4 合并/交付无 owner 与命令 | 已解决 | 本地集成收敛为 EffectIntent（executor = core）+ IntegrateChangeSetIntent + IntegrationReceipt（`spec/harness.md`）；远端 SCM 为同族 EffectIntent（executor = adapter，`spec/system.md`）；授权 actor（有权 human 或冻结 Workflow reducer）、outbox 与回读纪律齐备 |
| M5 TerminalGateway/attach provider 无 owner、端口命名不一 | 已解决 | 归并为描述性说法"agentd 的终端网关"（`spec/README.md` 归并对照）；受控端口统一为 HarnessAdapter / RuntimeBackend |
| M6 Skill 无定义 | 已解决 | `spec/system.md`「固定内核与受控端口」正式定义（稳定 ID/revision/digest、manifest/来源/license；不授予权限、票权、委派或完成权） |
| M7 Gate 不在对象表、策略存放与 quorum-unreachable 无下文 | 已解决 | Gate 定位为"WorkflowRevision 与 Run Manifest 冻结的治理节点/规则，不是独立模块"；quorum-unreachable 产生类型化结果并沿冻结失败边推进（`spec/run.md`） |
| M8 project.md 写入合同缺半 | 已解决 | `spec/project.md` 写入合同表补齐 RepoInstance、Participant/ProjectRoleBinding、Chat 端口绑定、ContextManifest/Bundle 等全部聚合 |
| M9 AttachDescriptor 无签发命令、TerminalBundle 缺创建规则 | 已解决 | control 按 owner/binding/generation 签发短期 AttachDescriptor，TerminalInputLease 另行 CAS（`spec/harness.md`）；TerminalBundle 归并为 ExecutionRuntime 终端通道字段组 |
| M10 「本地 TaskSource」对象不存在 | 已解决 | 交付措辞改为"纯本地 Task"；local 只是字段权威模式 |
| M11 ArtifactRevision 评审 subject 字段集未定义 | 已解决 | 五字段集与独立 `review_subject_digest` 已定义（`spec/project.md`）；ChangeSet 侧写明 `revision_digest` 与 subject digest 不同义（`spec/harness.md`） |
| M12 ResultProposed 无出边 | 已解决 | ResultProposed 重定义为 Attempt 终态（"已提交不可变 Proposal"）；准入/拒绝推进 Seat/Obligation，修正走新 Attempt（`spec/run.md`）；矩阵含"ResultProposed 不被误当成功" |
| M13 Run 状态机缺边 | 已解决 | 合法边全表重写，"每个过渡态都必须能被取消、失败或替代路径收口，不能因 Engine 失联永久阻塞绑定 Task"（`spec/run.md`） |
| M14 Request deadline/default policy 无定义 | 已解决 | Request 冻结 deadline 与 `fail\|cancel` 默认策略；Expire 按冻结策略收口各 owner 并撤销租约（`spec/run.md`、`spec/connections.md`） |
| M15 Lost 旧 writer 的 worktree 复用规则缺失 | 已解决 | 无法证明旧 writer 已 fence 时"原 worktree/ChangeSet 不得授予新写租约，只能保全并隔离，新的执行使用新物理 worktree"（`spec/harness.md`）；矩阵有对应负例 |
| M16 generation 持久位置与恢复顺序矛盾 | 已解决 | 恢复顺序改为"先取得 OS/资源侧排他权 → 打开权威账本 → CAS 推进 writer/backend generation"，control 与 agentd 主体分列（`spec/system.md`「单写者」「启动与恢复」） |
| M17 agentd 单 owner 无互斥机制 | 已解决 | agentd 必须先取得资源侧 OS lock/broker token 等排他原语；scope 定义为"相同资源 broker/socket/host namespace"；不能强制排他的 backend 只可观察（`spec/system.md`） |
| M18 Project 权威存储 SQLite/Git 不明 | 部分解决 | 分界已写明：Git 存不可变 Revision 内容，每实例账本存 admission/current pointer/lifecycle 投影（`spec/system.md`「事实与存储」）；多 clone 并发写共享配置的收敛合同仍未定，已并入 Room ground-truth 开放问题（`.memo/room-ground-truth-20260819.md`、delivery 未决项） |
| M19 Gate 身份链不完整、单用户三选二无法构成 | 部分解决 | producer→Participant 身份链已补（ExecutionSpec 冻结逻辑 Participant/Seat identity；producer_ref 解析校验）；"彼此独立"的判定维度与单用户构成移交 `.memo/participant-design-20260819.md` §5 及其开放问题 9，第一阶段按逻辑 Participant 分离执行 |
| M20 `~/.hctl2/` 共享无锁无版本合同 | 已解决 | 用户级 current 更新需排他锁 + expected-version CAS；活动执行只读冻结 revision（`spec/system.md`「事实与存储」） |
| M21 外部排序强制 remote token | 已解决 | 无等价 remote token 时"adapter 不得伪造"，降级为只读或可回读的绝对移动（`spec/task.md`「StartRun 前置与排序令牌」及外部对齐表） |
| M22 Harness 适配器无契约测试出门条件、OpenCode 范围写在 evidence | 已解决 | 出门条件补"至少一个 HarnessAdapter 和一个 RuntimeBackend 通过完整契约测试"；Codex/Claude Code/OpenCode 进入 delivery 范围表 |
| M23 矩阵缺非完成信号负例 | 基本解决 | 完成权威与协作边负例已入矩阵（"Harness/LLM/adapter/外部 Closed 冒充 actor 均拒绝"、"active Run 未收口时 terminal intent 拒绝"、agent-authored `@` 系列）；剩余两条随本记录补上（Memo 不经 PublishMemoIntent 不成立、JCS 等价摘要与篡改拒绝）；稳定测试编号仍缺（见 low） |
| M24 evidence 以规范口吻新增产品规则 | 已解决 | 扩展治理规则移入 `spec/system.md`（响应式改绑禁止、进程内扩展等同受信任代码）与 delivery「明确不做」（不建市场）；evidence 该节改为链接内核合同的对照叙述 |

## low（47 条，抽查处置）

**已确认解决（抽查）**：`docs/design/{layers,cross-layer,delivery}/` 空目录已清；`.agents/`、`.codex/` 已不存在；recipe 无残留，fan-out 由冻结 Seat 模板与 human-only 规则承载并有定义；Restore Project 后 Room 回 Active；Reopen 前置已定义；digest 规则唯一 owner（`spec/system.md` 共享摘要规则，project/harness 引用）；根 README 流程图补齐 Project→Harness 短路边与结果回流边；decision-history 版本与日期齐；Needs Attention 由 Task 合同定义为派生 health；`DistillTaskProposal` 命名与 `memory/` 目录随本记录修订（见下）。

**确认仍开放**：契约测试矩阵无稳定测试编号；mermaid 节点标签仍用 `\n`；裸 "HCTL" 指称混用；CLI 与场景合同缺口（`run` 无 resume/replace、`export` 无定义、terminal 无接管命令）；切片 B 的 "B/C/D" 投票记号未定义；追加型数据保留策略、时钟权威、Linux headless secret store、Subscribe cursor 作用域、"安全暂停"可观察形态均未定义；外部证据 pinned commit/许可证仍未联网逐一复核。其余未逐条重查，随后续修订按需处理。

## 随本记录的修订

- `spec/task.md`：未定义术语 "Run Context" 改为 "Run Manifest、ContextManifest 或 ExecutionSpec"；
- `spec/system.md`：存储拓扑 `memory/` 改为 `memos/`，对应 Memo 发布版本；
- `spec/project.md` 场景合同：补 mention 确定性解析与失败语义（恢复 v0.7.0 规则：无唯一授权候选时明确失败，不按显示名模糊匹配、不交给模型猜测路由）；
- `spec/connections.md`：`DistillTaskProposal` 改为描述性说法（不满足命名门槛）；
- `delivery.md` 契约测试矩阵：补 mention 解析失败、Memo 非自动发布、JCS 等价摘要三个负例；
- 两份 2026-08-19 专题 memo 补术语说明（成稿于 v0.9.1 归并前，InvocationBinding/AttemptSpec 已并入 ExecutionSpec）。

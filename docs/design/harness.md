# Harness 与 Terminal

> 本文是 Harness 模块的唯一领域权威；Terminal 是其操作合同，不拥有独立领域事实。模块交接见[连接合同](./connections.md)，通用机制见[系统边界](./system.md)。

## 模块职责

Harness 模块把 [Project](./project.md) 的一次 InvocationBinding 或 [Run](./run.md) 的 AttemptSpec 变成可观察、可隔离、可恢复的物理执行。它不决定 Project 目标、Task 完成、Run Gate 或领域权限。

| 对象 | 含义 |
| --- | --- |
| WorkerProfile | Harness、模型、模式、权限和环境的可复用配置 |
| HarnessDefinition / Installation / Capability | Harness 是什么、当前主机安装在哪里、实际支持哪些能力 |
| HarnessAdapterBinding | 一次执行选定的 ACP/app-server/SDK/PTY/钩子接入方式与降级能力 |
| ChangeSet / ChangeSetRevision | 一次获准写入边界及其不可变 Git 快照 |
| ChangeSetWriteLease | ChangeSet 的独占写入权与 fence |
| RuntimeShard / InvocationRuntime | Run Attempt 或无 Run Invocation 的主机、隔离域和代次 |
| TerminalBundle | 一个 Attempt 或 InvocationRuntime 的终端通道集合 |
| AttachDescriptor / TerminalInputLease | 对精确目标的短期连接描述和单输入者租约 |
| ResultProposal / Evidence | Harness 提交给上层校验的结果和观测，不是 Verdict/Receipt |

## 写入合同

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| WorkerProfile / Harness binding | immutable revision + current pointer | control 处理 Create/Update/Resolve Binding Intent；agentd 只报告探测能力 | 活动 Invocation/Attempt 始终引用原 revision |
| ChangeSet / WriteLease | `change_set_version`、current revision；lease 为 `Pending / Active / Revoking / Revoked` | control 准入 Prepare/Grant/Revoke/Seal，core 物化并回读 Git，agentd 执行失权 | 一个 ChangeSet 至多一个 Active lease；ChangeSetRevision 只追加 |
| RuntimeShard / InvocationRuntime | `runtime_generation`；`Reserved / Active / Stopping / Stopped / Lost` | control 记录 binding 并处理 Activate/Stop；agentd 持有物理资源与观测 | Stopped/Lost 不复活；恢复或接管使用新 generation |
| TerminalInputLease | lease generation；`Active / Revoked / Expired` | control 授予/撤销，agentd 输入门执行；takeover 原子撤销旧 lease | 一个目标最多一个 Active 输入者 |
| ResultProposal / Evidence | immutable submission + producer sequence | Harness adapter 提交；control inbox 持久化；Project/Run 独占 admission | Proposal 不可改成 Verdict/Receipt；修正提交新 Proposal |

WorkerProfile、Harness 名称或“支持 ACP”都不隐含能力。每次绑定都必须从实际探测结果中选择精确端口和降级方式，并冻结版本、配置、能力、信任级别和权限。

## ChangeSet 与 Git 事实

Worktree 是 ChangeSet 的可替换物理资源，不永久属于 Project、Task、Room 或 Harness。一个 ChangeSet 同时最多一个有效写入租约；候选切换、接管或取消必须先让旧 writer 失权。

ChangeSetRevision 在有效租约下封存，至少固定：

```text
change_set_revision_id
+ change_set_id
+ parent_revision_id?
+ base_commit_sha
+ result_tree_sha
+ result_commit_sha?       # 仅 SCM 证据
+ revision_digest
```

评审 subject 使用 `{change_set_revision_id, change_set_id, parent_revision_id?, base_commit_sha, result_tree_sha}` 的 RFC 8785 JCS / SHA-256 摘要；commit 包装不同不会改变相同 base/result tree 的评审身份。返工创建新 Revision，旧 Revision 不改写。

Agent 自述“已合并”不可信。core 校验 Git base/HEAD/tree、祖先关系、PR、检查、评审和目标分支头。SCM 变更中断或结果未知时，core 必须回读 HEAD、index、worktree/merge 状态、PR head 和目标分支头，返回类型化恢复动作；收敛前不得签发成功 Receipt 或清理所需现场。

失败、取消、租约撤销和资源清理都不等于放弃代码。物理清理前，core 必须确认所有已跟踪、未跟踪且尚未封存的修改已有可恢复副本，agentd 只有得到该确认才可拆除资源；保全或封存失败时保留精确 worktree 路径、Git 状态和显式恢复动作，不能删除唯一副本。清理 worktree 也不删除领域历史。

## 运行时与观测

Run 可以有多个 RuntimeShard；RoomInvocation 至多一个 InvocationRuntime；Attempt 和 InvocationRuntime 各至多一个 TerminalBundle。InvocationRuntime 可以是容器、隔离作用域或结构化会话，不以 TTY 存在为前提。

InvocationRuntime 继承 InvocationBinding 的 `project_scope | repo_scope`。repo-scoped 调用可以没有 Project ref，但仍必须保留精确 RoomInvocation、Runtime、binding、generation、权限和适用 fence；已知运行时不能被降级成无主进程或模糊仓库活动。

agentd 拥有进程、PTY、原始流、心跳和主机观测，并执行 control 已获准的 start/input/cancel/stop；Attempt/Invocation 的领域 lifecycle 仍由 control 推进。存活与所有权观测按 `RuntimeBackend/进程/租约 > 结构化 lifecycle 事件或 hook > title/screen` 仲裁，语义观测按 `结构化提供方协议或原生 hook > 转录推断 > title/screen` 仲裁；低优先级信号不能覆盖仍有效的高优先级证据。每条观测记录 source、confidence、evidence 与 observed_at，且无论置信度多高都不能自行推进领域结果。

结构化事件统一归一为生命周期提示、工具调用、权限请求、文件变化、测试、用量和原始输出。未知事件保留原文并安全降级，不得凭渲染器猜测完成。

每个 ResultProposal 固定 proposal ID、owner kind/ID、execution generation、InvocationBinding 或 AttemptSpec digest、实际 Harness/Runtime binding revision、输出 schema、ChangeSetRevision/Artifact candidate、Evidence refs、producer sequence、适用 lease/fence 和 idempotency key。Harness 可以提交提案，但 Project/Run 才能准入；旧 generation 或越界输出只能留作审计。

## Terminal 场景

Terminal 是 Harness 的观察、诊断和接管场景；它可以渲染结构化执行流或真实 PTY，不要求每次执行都有 shell。

| 能力 | 含义 |
| --- | --- |
| exact attach | 连接仍存活的精确 PTY/进程 |
| native handoff | 交接同一个外部 Harness 会话 |
| structured inspect | 查看实时结构化事件与原始记录 |
| semantic resume | 以外部会话 ID 恢复上下文，可能创建新进程 |
| replay | 只读历史 |

Execution Chat projection 是 Terminal 中绑定且只绑定一个精确 RoomInvocation 或 Attempt 逻辑 owner、对应 runtime binding 和 generation 的结构化观察与控制视图，不是 Room，也没有独立 conversation identity。adapter 支持时，输入作为携带这些精确引用的获准 control action 写回同一执行体；能力不足时准确降级为 structured inspect 或 terminal，不得改投另一个会话。

Execution Chat 中的输入和事件不会自动成为 Room 内容。只有显式 Share to Room 动作经 Project 命令准入后才能发布，并携带 source event、execution owner、generation 及 transcript/evidence provenance；该投影消失或 runtime 被替代都不改变 Room 身份。

这些能力可以并存但不能互相冒充。AttachDescriptor 固定逻辑 owner、后端目标、host、runtime generation、能力、权限和过期时间；观察 trace/结构化流、终端输入或接管、Attempt 控制和安全输入分别授权，任一权限都不蕴含其他权限。一个目标可以有多个观察者，默认最多一个 TerminalInputLease 持有者；接管原子撤销旧租约，安全输入不得进入普通 trace、Room 或 replay。

| 角色 | 可以做什么 | 不能做什么 |
| --- | --- | --- |
| 场景客户端：Workbench Terminal | xterm、Execution Chat/结构化检查、精确 attach、能力说明 | 用 UI 状态推进 Task/Run，或把执行投影当作 Room |
| 场景客户端：CLI / WezTerm | 使用短期 descriptor 观察或接管精确目标 | 提交任意 argv/cwd/pane ID 绕过 agentd |
| 受控端口：HarnessAdapter | ACP、原生服务端、SDK、PTY 或钩子能力 | 把厂商 Session 当成 HCTL 身份 |
| 受控端口：RuntimeBackend | 持有进程/PTY/容器/mux 资源 | 决定领域权限、Gate 或完成 |

Workbench 或终端客户端退出不停止执行。断流按 generation/sequence 和快照恢复；无法证明是同一进程时只能 semantic resume、replay 或新建执行，不能声称 exact attach。

## 模块交接

以下只列所有权方向；派发、结果准入与恢复由[四模块连接合同](./connections.md)统一定义。

- Project 传入 InvocationBinding，Run 传入 AttemptSpec；Harness 不重新解释上层范围。
- Harness 只返回 ResultProposal、ChangeSetRevision、测试/SCM 证据和运行时观测。
- control/core 校验后才可形成 Run Verdict/Receipt 或 Task 验收证据。
- Skill 提供指导和上下文，不获得命令、Gate 或写入权。

## 不可破坏的边界

- Project、Task、Run、Participant 和 Seat 都没有直接终端映射。
- 同一 ChangeSet 只有一个 writer，同一终端目标只有一个输入租约持有者。
- 新 runtime generation 使旧 descriptor、结果和输入权失效。
- 进程、会话、屏幕和 Hook 不能签发 Verdict、Receipt 或语义完成。
- 运行时恢复必须准确区分热重连、原生会话恢复、新进程和历史重放。

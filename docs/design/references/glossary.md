# 术语对照表

> 状态：非规范对照 · 草案 v0.9.0<br>
> 本表只提供中英对照和一句话含义，方便快速查阅；完整语义以拥有该名词的模块文档为准，冲突时以模块文档为准。

## 通用

| 术语 | 中文对照 | 一句话含义 | 权威定义 |
| --- | --- | --- | --- |
| Harness | 编码代理工具 | Codex、Claude Code、OpenCode 这类可以执行编码工作的 Agent 工具 | [Harness](../harness.md) |
| Workbench | 工作台 | 集成四个场景的桌面客户端；只是客户端，没有额外权限 | [系统边界](../system.md) |
| Scene | 操作场景 | 一个模块的主操作界面（Chat Room / Kanban / Workflow / Terminal），不拥有领域事实 | [设计地图](../README.md) |
| Revision | 不可变版本 | 内容变化产生的一份不可改写快照；新内容创建新 Revision，current pointer 只由类型化命令推进，界面只读取它 | [设计地图](../README.md) |
| Intent（命令后缀） | 类型化命令 | 改变事实的唯一途径，携带 actor、目标、预期版本和幂等键 | [系统边界](../system.md) |
| actor | 行动者 | 提交命令的人、reducer 或执行体；其身份由认证入口赋予，不能自报 | [系统边界](../system.md) |
| reducer | 归约器 | control 内按冻结规则机械推进状态的确定性逻辑，不做自由判断 | [Run](../run.md) |
| digest | 摘要 | 对规范化内容计算的 SHA-256 指纹，用于精确引用 | [系统边界](../system.md) |
| generation | 代次 | 单调递增的所有权版本；旧代次的写入和结果一律失权 | [系统边界](../system.md) |
| fence | 失权拦截 | 让旧执行者、旧租约或旧代次不再能写入的机制 | [系统边界](../system.md) |
| lease | 租约 | 有期限、可撤销的独占权（如写入权、终端输入权） | [Harness](../harness.md) |
| outbox / readback | 发件箱 / 回读 | 先持久记录要做的外部动作，再执行并回读确认，避免结果未知时盲目重做 | [系统边界](../system.md) |
| CAS | 带版本前置的写入 | compare-and-set：写入时校验预期版本，版本不符即拒绝 | [系统边界](../system.md) |
| fail-closed | 默认拒绝 | 信息不完整或校验不过时拒绝动作，而不是放行 | [Task](../task.md) |
| headless | 无界面后台运行 | 正常执行不需要打开任何窗口或终端 | [愿景](../vision.md) |

## Project 模块（Chat Room 场景）

| 术语 | 中文对照 | 一句话含义 |
| --- | --- | --- |
| Repo / RepoInstance | 仓库 / 仓库实例 | Git 仓库的逻辑身份；某个本地 clone 的控制边界，拥有独立账本 |
| Project | 项目 | 具名目标、协作、承诺和交付物的长期容器 |
| Room | 协作聊天室 | 持久的多参与者协作空间；分 Repo Room、Project Room、Scoped Room 三种 |
| Participant | 参与者 | 可寻址的逻辑协作者档案；不等于某个进程或外部账号 |
| ProjectRoleBinding | 角色绑定 | 把 Project 角色固定到精确 Participant 版本的冻结绑定 |
| Context / ContextManifest / ContextBundle | 上下文 / 上下文清单 / 上下文包 | 一次调用看到了什么、为什么、来源是什么的可解释快照 |
| Request | 请求卡 | 向指定的人或角色索取信息、授权或决定的一级对象 |
| Memo | 备忘 | 用户明确提炼、预览并发布的长期知识 |
| Artifact / ArtifactRevision | 工件 / 工件版本 | 登记过的可引用交付物及其不可变发布版本 |
| RoomInvocation / InvocationBinding | 单次调用 / 调用绑定 | 从 Room 发起的一次有边界 Harness 调用及其冻结的授权 |

## Task 模块（Kanban 场景）

| 术语 | 中文对照 | 一句话含义 |
| --- | --- | --- |
| Task | 任务承诺 | 可排序、可指派、可验收的长期承诺 |
| TaskRevision | 任务契约版本 | 不可变的范围、验收标准和所需能力 |
| TaskOperationalState | 任务操作态 | 排序、优先级、负责人、阻塞等高频变化的状态 |
| Kanban / Board / lane | 看板 / 泳道 | Task 的主操作场景；泳道只是投影，不是生命周期 |
| TaskSource | 任务来源 | Linear、GitHub 等外部任务系统的受控端口 |
| TaskSourceSnapshot | 来源快照 | 外部系统的一次只追加观测；会改变契约的内容需采纳才生效，外部拥有的操作字段按绑定直接投影 |
| PendingAdoption | 待采纳变更 | 外部契约变化在被用户采纳前的状态 |
| TaskCompletionReceipt | 完成凭证 | 一次完成命令对精确契约、规则和证据的证明 |

## Run 模块（Workflow 场景）

| 术语 | 中文对照 | 一句话含义 |
| --- | --- | --- |
| Workflow / WorkflowRevision | 施工图 / 施工图版本 | 与引擎无关的不可变控制图和治理规则 |
| Run | 一次受治理施工 | 对冻结施工图、契约、候选和权限的一次授权执行 |
| Run Manifest | 施工清单 | 启动 Run 时冻结的全部绑定、规则和预算 |
| Workflow Engine / Conductor | 工作流引擎 | 只保存机械位置（token、重试、定时器）的外部引擎 |
| Obligation | 交付义务 | 一个外部节点必须产出的逻辑结果 |
| Seat | 执行席位 | 义务中稳定的逻辑执行者或投票位置 |
| Attempt / AttemptSpec | 执行尝试 / 尝试规格 | 某个候选对席位的一次执行及其不可变派发规格 |
| Gate | 评审关卡 | 施工图中冻结的治理节点；决定结果凭什么通过 |
| Verdict | 裁决 | 对精确版本的语义评审结论（通过 / 驳回 / 需修改） |
| Receipt | 凭证 | control/core 校验后签发的正式证明 |
| quorum | 法定票数 | 达到多少有效票才算通过（如三选二） |
| regate | 重新评审 | 被评对象换版本后作废旧票、完整重过关卡 |
| ReviewSubjectRef | 评审对象引用 | 对精确变更集或工件版本的 kind + ID + 摘要引用 |

## Harness 模块（Terminal 场景）

| 术语 | 中文对照 | 一句话含义 |
| --- | --- | --- |
| WorkerProfile | 执行者配置 | Harness、模型、模式、权限和环境的可复用组合 |
| HarnessAdapterBinding | 接入绑定 | 一次执行选定的接入方式（ACP/SDK/PTY 等）与降级能力 |
| ChangeSet / ChangeSetRevision | 变更集 / 变更集版本 | 一次获准写入边界及其不可变 Git 快照 |
| ChangeSetWriteLease | 写入租约 | 变更集的独占写入权；同时最多一个持有者 |
| IntegrationIntent / IntegrationReceipt | 集成命令 / 集成凭证 | 把精确变更集合入目标分支的授权及回读证明 |
| worktree | Git 工作树 | 变更集的可替换物理载体；可以丢弃重建 |
| RuntimeShard / InvocationRuntime | 运行时分片 / 调用运行时 | 一次执行的主机、隔离域和代次 |
| RuntimeBackend | 运行时后端 | 持有进程/PTY/容器资源的底层设施（如 Zellij、tmux） |
| agentd | 执行守护进程 | 发现 Harness、持有物理运行时并上报观测的本机组件 |
| TerminalBundle | 终端通道集 | 一次执行的全部终端通道 |
| AttachDescriptor | 连接描述 | 对精确目标的短期连接凭据 |
| TerminalInputLease | 终端输入租约 | 谁可以往这个终端打字；同时最多一个 |
| ResultProposal | 结果提案 | Harness 提交、等待上层校验的结果；不是裁决或凭证 |
| Evidence | 证据 | diff、测试输出、SCM 状态等可核验的观测 |
| attach / resume / replay | 接管 / 恢复 / 回放 | 三种不同的连接能力，不能互相冒充 |

# Stably Orca

> 类别：② Agent 协作平台 · 证据编号：E-L1-STABLY-ORCA、E-L2-STABLY-ORCA<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-l1-stably-orca"></a>
## E-L1-STABLY-ORCA · Stably Orca

### L1 核心价值与跨层画像

Stably Orca 的产品主轴是以 worktree 为中心的执行环境：每个 worktree 拥有独立分支、文件和 Agent 终端，PTY 由守护进程而不是桌面窗口持有。桌面应用退出但守护进程仍存活时，可以重新连接原进程并恢复布局、分屏、滚屏和焦点；守护进程已经退出时，只能创建新进程，再恢复布局、历史显示，或调用服务提供方的原生会话恢复。两条路径不能都笼统地叫作“会话恢复”。

它还把远程主机、差异审阅、分块暂存、提交、推送和 PR 评审串进同一条执行路径，并用运行时代次、PTY 代次和进程实例代次拒绝过期句柄。这些能力共同构成它在 L1 的核心价值。

### 四层设计亮点与边界

| 层 | 设计深度 | 定位与边界 |
| --- | --- | --- |
| L4 | 很弱 | Native Chat 只是同一 PTY 上的实验性结构化投影，底层终端才是事实来源；没有独立的 Project Room、意图账本或长期协作记忆。 |
| L3 | 中等 | Workspace Board、工作区检查点和外部系统绑定已经可用；本地看板状态还可以选择同步到 Linear。但卡片身份仍是 worktree，`workspaceStatus` 明确用于人工整理侧栏，没有独立 Task、Task Revision、验收或评审契约。 |
| L2 | **专项参考** | 已实现持久 Run 收件箱、Task 依赖、Dispatch 生命周期权威、消息交付确认与重放、幂等变更收据、心跳、重试隔离、Decision Gate、执行者资源回收和远程转发。这不是概念演示；但现役 Run 明确不调度，也不决定落点和并发度，自动调度器命令已经退役且不产生副作用，同时缺少通用 Workflow Revision、Obligation/Seat/Attempt、法定票数和证据治理。 |
| L1 | **核心参考** | PTY 所有权、冷热恢复、代际隔离、worktree、差异审阅、交付与远程连续性都有完整产品路径和源码实现。 |

在 HCTL 的设计组合中，Stably Orca 同时提供 L1 的执行连续性和 [L2 的持久监督协议](#e-l2-stably-orca)。L3 的可选 Linear 同步有实际产品价值，但仍以 worktree 为卡片身份，不足以定义独立的 Task 模型。

HCTL 采用：PTY 所有权、冷热恢复分类、运行时与进程实例的代次隔离、重新连接、worktree、差异审阅、远程操作、交付流程和相应故障测试。

HCTL 不采用：Workspace/worktree 充当 Project/Task、`workspaceStatus` 充当 Task 生命周期、会话或终端句柄充当持久 Run 身份、OSC/TUI 状态、worktree 评论或 `worker_done` 充当语义完成、Native Chat 充当 Room，以及 Stably Orca Run 与 HCTL Run 形成双重事实来源。

### 审计基线

固定 [`09ec516a`](https://github.com/stablyai/orca/tree/09ec516ae50b7b83fa65343d9ad96159e3fe71fc)（2026-08-12，软件包版本 `1.4.178-rc.2`，[MIT](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/LICENSE#L1-L21)）。官网会滚动更新，能力判断以固定源码和固定版本内的指南为准，官网只补充产品行为。

主要证据：

- [仓库](https://github.com/stablyai/orca)；[Worktrees](https://www.onorca.dev/docs/model/worktrees)；[Session restore](https://www.onorca.dev/docs/model/session-restore)；[Remote servers](https://www.onorca.dev/docs/remote-servers)；[Diff viewer](https://www.onorca.dev/docs/review/diff-viewer)；[Commit and push](https://www.onorca.dev/docs/review/commit-push)
- [守护进程持有会话、子进程、终端模拟器与客户端](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/session.ts#L109-L168)；[连接与原子快照](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/session.ts#L412-L484)
- [只接入仍存活的会话，`attachOnly` 不会偷偷创建新 shell](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/terminal-host-session-create.ts#L26-L142)
- [冷恢复创建新会话并回放磁盘历史](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/daemon-pty-adapter.ts#L737-L789)；[热重连连接原会话并回放快照](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/daemon-pty-adapter.ts#L811-L860)
- [运行时接管仍存活的守护进程 PTY，并使过期句柄失效](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/orca-runtime.ts#L9181-L9259)；[运行时、图与 PTY 代次检查](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/orca-runtime.ts#L31895-L32035)
- [持久 Run/Delivery/Receipt/执行者/Task/Dispatch/Gate 数据结构](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/orchestration/db.ts#L297-L620)；[`worker_done` 与心跳的受派者/过期检查](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/runtime/orchestration/lifecycle-reconciliation.ts#L16-L305)
- [Run 只负责持久命名空间和收件箱，落点与并发度由 Agent 选择](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/skill-guides/orchestration.md#L102-L181)；[自动调度器命令已经退役](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/skill-guides/orchestration.md#L273-L285)
- [`workspaceStatus` 是人工侧栏分类](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/shared/types.ts#L684-L685)；[可选的 Linear 状态同步](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/renderer/src/components/sidebar/workspace-board-task-status-sync.ts#L168-L239)；[Native Chat](https://www.onorca.dev/docs/agents/native-chat)


<a id="e-l2-stably-orca"></a>
## E-L2-STABLY-ORCA · Stably Orca 持久监督协议

Stably Orca 在 L2 的亮点不是自动规划，而是把人工或 Agent 主导的监督过程做成持久协议。Run 是持久命名空间和协调者收件箱；Task 保存依赖与状态；每次 Dispatch 把 Task 的一次尝试绑定到具体终端，并记录窗格、句柄、进程实例代次和能力。生命周期对账还会核对当前 Dispatch ID 与受派窗格/句柄，拒绝来源错误或已经过期的心跳与 `worker_done`。FIFO Delivery 会重复交付同一批消息直到收到确认；变更收据按调用者和请求实现幂等；执行者的启动、停止、释放和保留还会记录已经发生的副作用与未清理资源。Decision Gate、远程转发与过期 Dispatch 拒绝进一步补齐了监督过程中的恢复路径。

HCTL 在 L2 采用它的 Dispatch 权威、消息确认与重放、幂等变更收据、执行者资源所有权、失败后的残留状态，以及拒绝过期完成信号的规则。它的现役 Run [明确不负责调度或选择落点与并发度](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/skill-guides/orchestration.md#L102-L181)，自动调度器也[已经退役且不产生副作用](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/skill-guides/orchestration.md#L273-L285)；它没有 HCTL 的 Workflow Revision、Obligation/Seat/Attempt、法定票数、重新过 Gate，或与证据绑定的 Verdict/Receipt。因此，Stably Orca 是 L2 的持久监督专项参考，不能直接承担 HCTL 的 Workflow 权威事实。

固定版本、数据结构和生命周期检查见 [Stably Orca 的完整审计](#e-l1-stably-orca)。

## 复核记录

- **2026-08-24**：主干已到 v1.4.188（较固定基线约前进 10 个版本），出现较新的 AiVault 会话面板与 hosted review 组件；引用交付/评审能力前应刷新固定版本。提交直方图（terminal 47 / kanban 39）支持本文边界判断：看板投入接近终端，但卡片身份仍是 worktree，不足以定义独立 Task 模型。

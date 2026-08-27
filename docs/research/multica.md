# Multica

> 类别：② Agent 协作平台 · 证据编号：E-MULTICA<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-multica"></a>
## E-MULTICA · Multica

### 已跑通的产品闭环

Multica 把 Project 和 Issue 中的目标、讨论与状态保存为长期工作事实。每次分配、提及、私聊或 Autopilot 触发都会新建一个 Task，而 Task 只表示一次 Agent 运行。服务端先把它排入队列，再由本机守护进程认领并调用已经安装的 Harness，最后把消息、工具调用、错误、会话和交付分支写回。人或 Agent 随后决定继续讨论、重新运行还是结束 Issue。这个闭环同时触及四层，但 HCTL 只吸收各层真正独特的机制，不照搬整套产品模型。

### 审计基线与许可

固定实现基线为 [`main@2c0912b6`](https://github.com/multica-ai/multica/tree/2c0912b6ec764b373d44eeea1e80f0d9f11ab417)（2026-08-14）。同期最新发布版是 [`v0.4.26 / 19155e41`](https://github.com/multica-ai/multica/releases/tag/v0.4.26)，主干只比发布版多一个提交。项目仍处于 `0.x` 快速演进阶段，官网会滚动更新；能力判断以固定源码、迁移和测试为准。

仓库完整公开，README 将项目称为“开源”，但固定版本的 [`LICENSE`](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/LICENSE) 不是单独的 Apache-2.0：它在 Apache-2.0 文本之外增加了第三方托管、商业嵌入、品牌和归属要求，并声明附加条件优先。因此这里只把它当作公开源码的行为、协议和测试证据；在完成专门的法律审查并获得所需授权之前，不把其源码移植进 Apache-2.0 的 HCTL，也不把该许可证标成 Apache-2.0 或宽松开源许可。

### 四层设计亮点与边界

| 层 | 真正深入且独特的证据 | HCTL 的采用方式与边界 |
| --- | --- | --- |
| L4 | Project 保存跨多个 Issue 的目标、范围、长期要求、负责人和代码资源；Project 状态与 Issue 状态相互独立。Issue 集中保存可共享的目标、讨论、活动和执行历史；一对一 Chat 明确位于 Issue 之外且完全私密，团队要复用的结论必须另行写入 Issue、Project 描述或 Skill。Inbox 是面向人的关注入口，不是 Agent 工作队列。 | 采用“共享事实与私密探索分开、私聊结论显式发布”的边界，以及 Project 状态不从子项机械推导的做法。Multica 没有可持续共同引导的项目级 Room、Context 准入或知识晋升流程；Project/Issue 描述又会以当前值直接进入运行上下文，不能代替 HCTL 的持久 Room、Memo 或冻结版本。 |
| L3 | Issue 是可以长期讨论、修改、重新分配并最终关闭的工作承诺；Task 是一次生命周期有限的运行。同一 Issue 可以产生多个 Task，已有运行记录不会被覆盖；精确重试某个历史 Task 时仍调用该次运行当时的 Agent。官方文档明确规定：Task 的 `completed` 只表示该次运行正常结束，不表示 Issue 目标已经完成。 | 采用 Issue 与单次运行 Task 分离、运行历史不可覆盖、定向重试，以及“运行完成不等于工作完成”。不采用可变 Issue 描述作为冻结的 Task Revision，不采用分配或状态变化自动获得施工授权，也不把 Agent 将状态改成 `in_review`、产生分支或 Task 正常退出当作验收。 |
| L2 | Task 具有 `queued → dispatched → waiting_local_directory/running → terminal` 生命周期。数据库通过 `FOR UPDATE SKIP LOCKED` 原子认领，并把同一 `(Issue, Agent)` 的运行串行化；准备租约保护启动窗口，`dispatched_at` 充当认领代际的 CAS 隔离栅栏，认领响应丢失后可以重新领取，守护进程重启后可以回收。长期运行依赖运行时心跳，而不是固定的总时长；失败分类决定能否重试，后继 Task 保存 `attempt`、`max_attempts`、`failure_reason`、`session_id`、`work_dir` 和 `retry_of_task_id`，触发者、委派链和证据引用也随运行记录归属。Autopilot 还为定时和 webhook 的每个触发实例提供幂等与崩溃恢复测试。 | 采用领取（claim）、租约（lease）、隔离栅栏（fence）和重新领取（reclaim），并采用失败分类、重试谱系、来源归属和轮询兜底的实现与测试形状，尤其适合无 Workbench 时由服务端和守护进程协作执行。它没有 Workflow Revision、通用 DAG、Gate、Seat、法定票数或语义 Receipt；Squad leader 由 LLM 决策，不能成为控制事实；Autopilot 是可重复触发的操作手册，不是 HCTL Workflow。 |
| L1 | 一个统一的 `Backend` 契约接入 22 个 Harness 产品名称；其中 21 个协议族由后端构造器、数据库约束和锁步测试共同限定，Oh-My-Pi 复用 Pi 协议族。不同 Harness 的模型、MCP、Skill 路径和会话恢复能力被明确列成能力矩阵，并对“无法判断恢复请求是否被拒绝”等降级情况单独编码。本地 Git 路径会先保全脏工作树，再为每个 Task 建立 worktree；无论成功、失败还是取消，都会尽量提交已经产生的改动，提交失败时则保留 worktree，避免清理过程吞掉用户工作。 | 采用统一 Harness 契约、逐绑定能力探测、显式降级测试，以及“先保全、后隔离、任何退出路径都不丢改动”的 worktree 纪律。Multica 不拥有可重新接入的 PTY，也不能用会话、分支或工具调用成功证明语义完成；其源码许可也排除了直接移植。 |

### 采用结论

HCTL 应组合采用四块经过源码验证的机制：L4 的共享/私密发布边界；L3 的 Issue/单次运行分离；L2 的领取、租约、重试、恢复和来源归属机制及其测试用例；L1 的 Harness 能力契约与无损 worktree 收尾。它们分别进入对应层，不需要把 Multica 设成某一层的唯一参考。

明确不采用：用 Issue 当前内容充当 Task Revision，用分配或状态变化充当启动授权，用 Squad leader 的 LLM 判断充当调度权威，用 Autopilot 充当通用 Workflow，用 Task 的 `completed` 充当 Verdict/Receipt，以及移植受自定义许可证约束的源码。

主要证据：

- 官方产品行为：[Projects](https://multica.ai/docs/projects)、[Issues](https://multica.ai/docs/issues)、[Tasks](https://multica.ai/docs/tasks)、[Chat](https://multica.ai/docs/chat)、[守护进程与运行时](https://multica.ai/docs/daemon-runtimes)、[Harness 对比](https://multica.ai/docs/providers)与[Autopilots](https://multica.ai/docs/autopilots)
- 固定文档：[Projects](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/projects.mdx)、[Issues](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/issues.mdx)、[Tasks](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/tasks.mdx)、[Chat](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/chat.mdx)与[Harness 能力矩阵](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/apps/docs/content/docs/providers.mdx)
- L2 实现：[Task 服务](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/internal/service/task.go)、[领取与重试 SQL](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/pkg/db/queries/agent.sql)、[租约与重试数据结构](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/migrations/055_task_lease_and_retry.up.sql)、[准备租约](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/migrations/124_task_prepare_lease.up.sql)、[领取竞争测试](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/internal/service/task_claim_race_test.go)、[完成竞争测试](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/internal/service/task_complete_race_test.go)与[Autopilot 恢复测试](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/cmd/server/autopilot_schedule_job_test.go)
- L1 实现：[统一 `Backend` 与能力例外](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/pkg/agent/agent.go)、[协议族锁步测试](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/pkg/agent/agent_supported_types_test.go)与[本地 worktree](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/internal/daemon/execenv/local_worktree.go)

## 复核记录

- **2026-08-24**：主干已从固定基线前进约 210 个提交（v0.4.26→v0.4.32），Harness 适配器从 22 个名称增至 23 个（新增 antigravity 等）；引用能力矩阵前应按新 HEAD 复核适配器清单，其余机制结论不变。提交直方图证实它是罕见的四场景全触及产品且无一场景超过四成：看板是叙事中心，运行时是工程中心（terminal 37 / kanban 29 / room 13 / workflow 12）。

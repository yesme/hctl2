# 第一阶段、验证与自举

> 本文只定义“交付什么、怎样证明”；对象和状态以四个模块文件为准，端到端步骤按[连接合同](./connections.md)验收。

## 第一阶段范围

第一阶段面向单用户、单机、单 RepoInstance 下的多个 Project，并交付 macOS/Linux 打包后的 Workbench/control/agentd/Workflow Engine 生命周期。领域服务不依赖 Workbench 窗口存活，Windows 只保留原生适配边界。

| 模块 | 集成场景必须交付 | 第三方适配必须交付 |
| --- | --- | --- |
| [Project](./project.md) | Repo/Project/Scoped Room、时间线、Composer、Context、Request、Memo/Artifact、至少两个并发 Invocation | 第一阶段外部聊天桥接不作为出门条件 |
| [Task](./task.md) | Project Overview、可访问 Kanban、本地 TaskSource、完成预览 | Linear/GitHub 均通过身份/快照测试，其中一个通过完整字段读写与对账 |
| [Run](./run.md) | WorkflowRevision 编译、Run 预览、只读图、Request、三选二 Gate、返工/regate | Conductor 经 WorkflowEngineAdapter 通过本地分发与恢复测试 |
| [Harness](./harness.md) | ChangeSet/diff/证据、结构化执行检查、xterm、精确 attach | Codex/Claude Code/OpenCode 能力探测；一个 RuntimeBackend 通过完整契约测试；WezTerm 可选 |

四个场景由 Workbench 集成，但其命令必须可以由同一 service 供 CLI 或外部适配器使用。

## 公共 CLI

公共二进制固定为 `hctl2`：

| 范围 | 第一阶段命令 |
| --- | --- |
| 运维 | `init`、`start`、`status`、`doctor`、`export` |
| Project / Chat Room | `room list\|show`、`request list\|show`；复杂编辑暂由 Workbench 提供 |
| Task / Kanban | `task create\|update\|adopt\|move\|complete\|reopen\|cancel` |
| Run / Workflow | `run show\|preview\|start\|pause\|cancel`；修改动作先预览确认 |
| Harness / Terminal | `terminal inspect\|attach\|resume\|replay`；必须指向精确 descriptor |

CLI 没有隐藏权限，也不直接写 SQLite、Workflow Engine 或 RuntimeBackend。

## 明确不做

- 多用户组织/RBAC、云队列、多主机调度和 Conductor 高可用；
- Windows 正式版本、浏览器/移动客户端和通用远程中继；
- 完整外部聊天桥接、任意第三方插件市场；
- 通用可视化 Workflow 编辑器或模型自由生成后直接部署；
- 任意带外外部写的全局检测与自动补偿；第一阶段只管理受控端口发出的 intent，并把 provider 外部变化当作 drift/Snapshot 回读；
- 同时完成 Linear 与 GitHub 两套完整双向适配器；
- 多 Task Run 的分支/合并政策；第一阶段每个 Run 只绑定 0..1 个 TaskRevision。

## 纵向切片 A：无 Run 自举

1. 初始化 RepoInstance，创建 Project 与 TaskRevision。
2. 从 Project Chat Room 发起一次写入型 RoomInvocation，冻结 InvocationBinding。
3. Harness 在隔离 worktree 和有效写租约下修改代码，产出 ChangeSetRevision 与测试证据。
4. Project 场景展示精确 diff；评审绑定 ReviewSubjectRef。
5. 通过 CompleteTaskIntent 写 TaskCompletionReceipt。
6. 重启 Workbench/control/agentd 后，账本、worktree 归属、证据和投影一致且不重复副作用。

这是 B2 的第一次真正自举；它不等待 Workflow Engine 或 quorum。

## 纵向切片 B：完整治理

1. 从 Project 提炼 Task，批准 WorkflowRevision 和 EngineDeploymentRevision。
2. 预览并启动绑定一个 TaskRevision 的 Run。
3. Engine external task 产生 Obligation/Seat/Attempt，Harness 执行并返回提案。
4. 需要输入时创建 Project Request；答案 signal 回原执行。
5. B/C/D 对同一 ReviewSubjectRef 投票；备用候选只替换同一 Seat 的技术失败。
6. `changes_requested` 产生新 ChangeSetRevision，旧票失效并完整 regate。
7. 达到法定票数后写 Gate Receipt，再由 core 校验 SCM/合并事实。
8. Run 结束后 Task 独立准入 CompleteTaskIntent。
9. 任意步骤崩溃后通过 generation、outbox 和 readback 恢复，不重复外部效果。

## 外部 Task Source 切片

连接 provider → 创建稳定实体映射 → 导入 Snapshot → 采用为 TaskRevision → 按字段权威写回 → 回读确认。支持显式 refresh 与定期 reconcile，不依赖公网 webhook。创建结果未知、限流、外部修改、tombstone、重新绑定和无 Workbench 原生操作都必须有测试；外部 Closed 永远不直接产生 HCTL 完成。

## 自举阶段

| 阶段 | 事实切换 |
| --- | --- |
| B0 | ID、SQLite、command/query/event、进程和恢复底座 |
| B1 | Project Chat Room 与本地 Task 影子试用 |
| B2 | 无 Run 切片成为真实开发入口 |
| B3 | 接管自身待办、并发 Invocation、Request、Receipt 和冷启动恢复 |
| B4 | 引入 Workflow Engine、Run、Seat 和独立 Gate |
| B5 | 候选切换、三选二、regate 和完整故障恢复；第一阶段目标 |
| B6 | 稳定版本 N 构建、验证、升级和回滚隔离环境中的 N+1 |

旧工具在事实切换前可以作为执行者或逃生通道，不能继续保有平行 Project/Task/Run 账本。降级超过约定能力时回退到上一自举级别并留下审计记录。

B5 是第一阶段功能成熟度目标；正式发布、升级与回滚仍必须通过 B6，不能把“已能自举”当成可分发版本。

自举验收不得对 HCTL2 仓库、内置账号或测试环境设置隐藏特判：开发自身必须只使用公开的 Query/Preview/Submit/Subscribe、CLI 和受控端口，实际 Context、权限与证据均可检查；手工推进 Engine、直接改库、隐藏 Prompt/Context 或在产品外补签 Receipt 都不算通过。

## 契约测试矩阵

| 范围 | 必须证明 |
| --- | --- |
| Project / Chat Room | CJK 输入、结构化引用、草稿/游标/未读、并发写入得到一致 room_sequence、并发流隔离；Repo Room 只把显式选中的来源链带入新 Project；Scoped Room 回填和同根因 Request 去重；Context 可解释、Room 历史可恢复 |
| Task / Kanban | TaskRevision、lifecycle、stage、lane 投影与外部状态分离；非法 move/complete 拒绝；local state version 与 remote revision 不混用，过期邻项移动重算；影响 TaskRevision 的 PendingAdoption 未处理不得 Start，拒绝/延期的契约内容不得进入 Run Context，external_authoritative 操作字段不能被本地否决；Snapshot/adopt/tombstone、双状态与无 Run 完成；同一外部实体跨 Project/重复 connection 仍冲突，external_entity_kind 不同则保持独立 |
| Run / Workflow | 编译/Profile 拒绝、0..1 Task 绑定、Engine mutation 只有 control；retry 原子替代并隔离旧 Obligation/Seat/Attempt，候选耗尽类型化收口；placement 变更留下绑定 Run version 的不可变审计事件；作者不能占必需 reviewer Seat；quorum/regate 和迟到结果拒绝 |
| Harness / Terminal | 能力探测、ChangeSet 单 writer、精确 Revision/digest、runtime generation；失败清理不丢唯一未封存/未跟踪修改，SCM 中断先回读；冲突观测按来源证据仲裁；观察、输入、Attempt 控制与安全输入权限分离；attach/resume/replay、IME/背压/慢客户端隔离 |
| 连接 / 端口 | 每条 handoff 固定 source ref/digest 与唯一 binding；client/port 权限分离；dispatch/result 迟到拒绝；外部 effect ACK 未知不重复；Harness 绕过端口的写能力被拒绝，外部 drift 只形成 Snapshot/观测而不是结果 |
| 系统 | 第二 control/agentd 只读或拒绝；命令幂等；commit/ACK 各崩溃点回读；schema migration、backup/restore、投影重建；旧 generation 与越权适配器拒绝；打包后的整窗启动/退出/升级和安全边界 |
| 产品 | 用户十秒内能回答 Project 目标、Task 状态、Run 阻塞、所需动作、当前 Harness 和证据版本；正常成功保持安静；HCTL2 仓库自举不使用隐藏特判或产品外补签事实 |

交付测试检查可观察行为，不复述模块状态机。模块新增合同必须在这里增加一个失败用例，而不是再建一份不变量文档。

## 开工前 Spike

1. **Conductor 本地分发**：固定版本、JVM/SQLite 打包、启动、升级、备份和恢复；失败则重开 Engine ADR，不自研第二引擎。
2. **RuntimeBackend**：Zellij 与 tmux 用同一套 attach、输入、resize、重启、残留进程、macOS/Linux 测试，第一阶段只选一个。
3. **外部 Task Source**：Linear/GitHub 用同一身份、字段权威、outbox/readback、限流和 tombstone 样本，选择首个完整双向适配器。

## 技术基线

Rust control/core/agentd；Electron + React 19 Workbench；SQLite + FTS5 与 Git；Tiptap、React Aria、React Flow + Dagre、xterm.js；Conductor 经适配器接入。选择受契约测试约束，不能为了保留依赖而削弱模块边界。

## 未决问题

- ChangeSet/PR 默认基数与后续多 Task Run 的集成策略；
- Repo Room 跨 clone 迁移、隐私和保留期限；
- Project 拆分/合并和 Task 依赖的产品表达；
- Scoped Room 自动归档策略；
- 首批原生会话导入范围与长期维护预算；
- 多主机、Windows、远程/多设备和多用户权限；
- 成本/预算硬上限及运行中耗尽的交互；
- 第一阶段之后首个外部 Chat Room 适配器。

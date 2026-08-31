# Superset

> 类别：② Agent 协作平台 · 证据编号：E-SUPERSET<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-superset"></a>
## E-SUPERSET · Superset

### L1 核心价值与产品闭环

Superset 的完整闭环是：把 Project 注册为代码仓库，从 Task 或提示词创建独立的 Workspace/worktree 和分支，在持久终端中运行 Agent，经 Changes、PR 与 CI 评审，再合并并删除 Workspace。它的官方心智模型可以概括为“在隔离工作区中委派，通过分支和 PR 集成”。这个模型没有提供 HCTL 的四层领域事实，但对 L1 的进程存活、客户端重新接入、会话找回和 worktree 安全销毁处理得很深入，因此是 L1 的核心实现参考。

### 审计基线与许可

固定实现基线为 [`main@4e18e1fa`](https://github.com/superset-sh/superset/tree/4e18e1fa794be7969d517bea86d082105e44c836)（2026-08-13）。同期最新发布版是 [`desktop-v1.21.0 / 067182bc`](https://github.com/superset-sh/superset/releases/tag/desktop-v1.21.0)，主干只比发布版多一个 Codex MCP 传输类型修正。官网会滚动更新，能力判断以固定源码和测试为准。

仓库公开了完整的单仓库代码，但许可证为 [`Elastic License 2.0`](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/LICENSE.md)，明确限制把实质功能作为第三方托管服务提供。它属于源码可见许可证，不是宽松开源许可。HCTL 可以采用其设计、协议形状和故障测试，不能未经授权移植实现源码。

### 四层设计亮点与边界

| 层 | 真正深入且独特的证据 | HCTL 的采用方式与边界 |
| --- | --- | --- |
| L4 | 很弱。Project 基本等于注册仓库，Agent Chat 是 Workspace 内的一种执行界面；没有项目级持久 Room、共享意图账本、Request 或知识准入。 | 不进入 L4 参考组合；Project、Workspace 和终端会话都不映射为 HCTL Project Room。 |
| L3 | 同时支持 Superset 原生 Task 和外部系统 Task，并能把 Task 内容变成 Workspace 的 Agent 提示词；`task.start` 通过只向前推进、可重复调用的状态更新同步外部系统。Workspace Board 的分栏则从 Agent 运行信号、PR 状态和归档原因派生，不是独立的 Task 生命周期。 | 只保留边界证据：Task 适合充当外部来源和启动入口，但没有冻结的 Task Revision、验收约束或独立的评审与承诺事实；Workspace、分支和 PR 状态不能代替 HCTL Task。 |
| L2 | Automation 保存定时规则、目标设备和运行历史，但它把 Workspace 创建成功记作这次运行的成功，明确不追踪 Agent 的执行结果，而且采用至少一次投递。官方编排 Skill 也明确说明：Superset 只提供会话传输，依赖关系和完成状态由协调者保存在工作上下文中；完成标记只是提示词约定，不是持久事件。 | 这是明确的边界证据：投递已接受、Workspace 已创建、终端存在，都不等于执行结果，更不等于 Workflow、Verdict 或 Receipt。可以采用幂等投递要求和无界面投递接口，但不能把 Automation 或工作上下文中的协调表当作 HCTL L2 事实。 |
| L1 | **核心参考。** 独立的 `pty-daemon` 持有 PTY，`host-service` 只通过 Unix 域套接字使用它；主机服务重启不影响 shell 进程，守护进程平滑升级时还能通过文件描述符移交（fd handoff），把同一 PTY 交给继任进程。主机服务与渲染端使用 `epoch:seq` 重连：在保留范围内精确补发，首次连接发送末尾快照（`tail`），代际不符或缺口过大时显式重新锚定（`reanchor`）；2 MiB 的补发环形缓冲区有界，单个慢渲染端的待发缓冲超过 8 MiB 时只断开该客户端，不拖死 PTY。SQLite 保存终端记录、Agent 绑定和 `disposeRequestedAt` 终止意图；回收器会重试失败的终止操作，守护进程断连后先向继任守护进程核实真实会话，再决定哪些绑定成为可恢复候选。Workspace 删除先写归档墓碑，再依次完成预检、`teardown` 清理脚本、PTY、worktree、分支和缓存清理；失败时恢复可见，进程崩溃后由对账器继续。 | 采用 PTY 进程所有权、文件描述符移交与接管、分代重连和显式降级、有界的慢客户端隔离、持久终止意图与回收器、终端与 Agent 会话绑定、先核实守护进程实际状态再宣告死亡，以及“先写意图、再执行清理”的可恢复分阶段 worktree 清理流程。CLI/MCP 的 `terminal list/read/send/close` 还可作为无 Workbench 时的最小控制面。 |

### “持久终端”实际保证到哪里

Superset 的几类恢复必须分开描述：

- `pty-daemon` 仍存活时，桌面或 `host-service` 重启可以接管原 PTY；守护进程平滑升级时，文件描述符移交可以保留同一 shell PID；
- 守护进程内部的 `SessionStore` 只是进程内映射表，每个会话只有 64 KiB 环形缓冲区，不写入磁盘；渲染端的 2 MiB 补发环形缓冲区也位于主机服务内存中；
- 守护进程被真正杀死或机器重启后，原进程无法保留。系统只能创建新的 shell，并在已有终端与 Agent 绑定、外部系统会话 ID 仍可用时尝试恢复 Agent 会话；
- `epoch:seq` 的精确模式只覆盖主机服务仍保有对应代际和字节范围的情况。代际变化或缺口超出环形缓冲区时会进入 `tail` 或 `reanchor`，不能宣称任意断线都能不重复、不遗漏地恢复。

HCTL 的失败语义必须采用上述细分，不能只写“应用重启后会话仍在”。Superset 当前的守护进程协议使用 Unix 域套接字和文件描述符移交，并明确没有 Windows ConPTY；HCTL 只能借鉴机制和测试，仍需通过自己的跨平台运行时后端契约实现。

### 采用结论

HCTL 应把 Superset 放进 L1 核心参考，采用 PTY 所有权、接管与移交、重连分级、慢客户端隔离、持久终止意图、Agent 会话恢复绑定和可恢复的分阶段 Workspace 清理。L2 只引用它清楚暴露的边界：投递与会话传输不拥有执行结果，更不拥有语义完成。

明确不采用：让 Project、Workspace 或 worktree 充当 HCTL 身份，让钩子、标题、PR、CI 或 Board 分栏充当语义完成，让 Automation 的 `created` 充当 Run 成功，让提示词标记和协调者上下文充当 Workflow 事实，以及移植受 ELv2 约束的实现源码。

主要证据：

- 官方产品行为：[Superset 模型](https://docs.superset.sh/superset-model)、[终端集成](https://docs.superset.sh/terminal-integration)、[Automations](https://docs.superset.sh/automations)、[Tasks](https://docs.superset.sh/tasks)、[远程 Workspace](https://docs.superset.sh/remote-workspaces)与[MCP 服务端](https://docs.superset.sh/mcp-server)
- 固定产品文档：[Superset 模型](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/apps/docs/content/docs/superset-model.mdx)、[Automations](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/apps/docs/content/docs/automations.mdx)、[Tasks](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/apps/docs/content/docs/tasks.mdx)与[编排 Skill](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/plugins/superset/skills/orchestrate/SKILL.md)
- PTY 与重连：[`pty-daemon` 设计和测试说明](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/pty-daemon/README.md)、[进程内 `SessionStore`](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/pty-daemon/src/SessionStore/SessionStore.ts)与[`host-service` 终端和重连实现](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/host-service/src/terminal/terminal.ts)
- 持久恢复：[终端与 Agent 数据结构](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/host-service/src/db/schema.ts)、[Agent 绑定与可恢复候选](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/host-service/src/terminal-agents/persistence.ts)与[守护进程丢失核实](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/host-service/src/terminal-agents/daemon-loss-sweep.ts)
- Task/Board 与清理边界：[`task.start`](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/trpc/src/router/task/task.ts)、[Board 分栏推导](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/apps/desktop/src/renderer/routes/_authenticated/_dashboard/v2-workspaces/components/V2WorkspacesBoard/utils/deriveBoardColumn/deriveBoardColumn.ts)与[Workspace 分阶段清理](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/host-service/src/trpc/router/workspace-cleanup/workspace-cleanup.ts)

## 复核记录

- **2026-08-24**：主干新增 SDK 结构化会话面（packages/chat-runtime 的 claude/codex 适配器与 journal/replay/projection 设计）——绕过 PTY、经 SDK 驱动 Harness 的第二执行路线，既是 L1 会话恢复的增量参考，也再次佐证"会话传输不等于 Workflow 事实"。tasks/automations 虽在增长（2026 年各约 400/357 次文件变更）但仍是薄 UI 层，不改变 L3/L2 边界定位。

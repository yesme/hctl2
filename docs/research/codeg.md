# Codeg

> 类别：② Agent 协作平台 · 证据编号：E-L3-CODEG<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-l3-codeg"></a>
## E-L3-CODEG · Codeg

### 核心价值与跨层画像

Codeg 的产品闭环是：先把一项工作写成独立 `WorkTask`，再排队并按并发上限领取；启动时根据精确的 base SHA 创建隔离 worktree；Agent 执行过程中可以进入 Needs You；完成一轮后由用户查看结果、diff、时间线和预检，再选择 Rework、Keep going、Ask、Double-check 或 merge；最后用 Git 事实确认结果是否真正落地。官方 Tasks 指南对边界的概括也很准确：Conversation 是用户坐在前面共同推进的会话，Task 则是写下以后可以暂时离开的异步承诺。

`WorkTask` 的 `worktree_folder_id`、`conversation_id` 和 `connection_id` 都可以为空，因此 Task 身份不依赖某次运行环境。状态变化使用“预期状态 + `run_seq`”进行 CAS，并把只追加的时间线事件放在同一事务中。Board、排队、Needs You、评审、后续动作、预检、Git 事实和重启恢复，共同构成其完整的 L3 产品机制。

Codeg 的核心价值在 L3，其他层也有可单独采用的机制。L1 可以专项参考它把 ACP、worktree、Git、diff、Composer、事件卡片，以及由 Agent 所在环境提供的文件系统和终端沙箱接成一条执行体验；L2 可以把 Automations 的定时触发、单个 Automation 串行、补跑，以及固定 Task 流程与 merge 恢复，当作“产品层自动化”和“通用 Workflow 治理”之间的边界证据。桌面终端由进程内 `HashMap` 持有，没有持久滚屏记录、进程重启恢复或稳定的远程重新接入，因此 PTY 所有权与重连主要参考 Stably Orca。Automations 没有带版本的图、权限与法定人数规则或通用 Gate，也不能单独定义 HCTL 的 L2。

### 审计基线

| 基线 | 状态 | 可支持的结论 |
| --- | --- | --- |
| [`v0.24.0 / df7a872d`](https://github.com/xintaofei/codeg/commit/df7a872de44546277e4c49cfe9d173c631161dc6) · 2026-08-11 | 已发布 | 独立 `WorkTask`、四列 Board、排队/并发/定时启动、Needs You、评审/预检/后续动作、merge 与基于 Git 事实的恢复 |
| [`main@a34a047a`](https://github.com/xintaofei/codeg/commit/a34a047a568018ee180dee75add8c9c7d30b2ea6) · 2026-08-14 | 未发布审计快照；按 first-parent 口径比发布版前进 23 个 commit | merge 排队、[由 Agent 所在环境提供文件系统与终端沙箱](https://github.com/xintaofei/codeg/commit/b7e21e4c789ba70036ec87de5ed72dec3d25a678)，以及重连与权限修正 |

[发布版与审计快照的差异](https://github.com/xintaofei/codeg/compare/df7a872de44546277e4c49cfe9d173c631161dc6...a34a047a568018ee180dee75add8c9c7d30b2ea6)。v0.24.0 和当前 Tasks 指南规定“同一项目已有 merge 时拒绝第二个 merge”，主干从 [`597a7eeb`](https://github.com/xintaofei/codeg/commit/597a7eeb24e4a5f8aca149f2f5c182d3c2c90510)起改为排队。发布能力与主干能力在本文中分别标注。

### 采用与边界

HCTL 采用独立 Task 身份、显式状态机、基于 `run_seq`/CAS 的过期事件隔离、事务内时间线、精确 base SHA、复用 worktree 的重试、Needs You 投影、评审/预检/后续动作，以及“根据 Git 事实恢复 merge”的测试。`done` 只应来自已经落地的 merge，或用户明确接受“没有内容可合并”；Agent 自报 `task_complete` 只能作为建议，不能决定 Task 是否完成。

Codeg 的 `WorkTaskConfig` 不能直接当作 HCTL 的冻结 Task Revision。它保存 `prompt_blocks` 和每个 Task 的覆盖值，但空字段会在真正启动时继承当时的 Folder 设置，实际采用的值只写入 `config_effective` 审计事件。HCTL 在批准 Run 时必须冻结完整的 Task Revision/Workflow Revision，不能让可变默认值在启动时继续改变契约。

明确不采用：不把 Conversation 当作 Room，不把 To-do 数据结构直接视为 HCTL Task，不把固定 Task 流程直接视为 Workflow，不让拖进 In Progress 自动获得施工授权，不因 Agent `task_complete`、Done 分栏或 Git 已落地就自动满足语义验收，不把 Agent 发起 merge 等同于具有治理权限，不让主 LLM 路由充当控制事实，也不把进程内终端当作 L1 持久性模型。

主要证据：

- [仓库](https://github.com/xintaofei/codeg)、[v0.24.0 发布版](https://github.com/xintaofei/codeg/releases/tag/v0.24.0)与固定版本的 [Apache-2.0 许可证](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/LICENSE)
- 官方产品说明：[Tasks](https://docs.codeg.app/guide/tasks)、[Automations](https://docs.codeg.app/guide/automations)、[多 Agent](https://docs.codeg.app/guide/multi-agent)与[聊天频道](https://docs.codeg.app/guide/chat-channels)
- Automation 的固定源码：[持久对象](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/db/entities/automation.rs)、[Run 记录](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/db/entities/automation_run.rs)、[调度引擎](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/automation/engine.rs)与[事务服务](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/db/service/automation_service.rs)
- [WorkTask 状态与持久字段](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/db/entities/work_task.rs)、[配置与 Folder 设置](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/models/work_task.rs)、[CAS 与事务事件服务](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/db/service/work_task_service.rs)、[执行与恢复引擎](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/work_task/engine.rs)及[Git 判定](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/work_task/git.rs)
- [四列 Board 投影](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src/components/tasks/board-columns.ts#L4-L58)、[Composer](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src/components/chat/composer/rich-composer.tsx)、[ACP 注册表](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/acp/custom_registry.rs)与[委派数据结构](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/acp/delegation/tool_schema.json)
- L1 边界证据：[桌面 PTY 管理器](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/terminal/manager.rs)与[面向 Agent、限制输出大小的终端运行时](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/acp/terminal_runtime.rs)

## 复核记录

- **2026-08-24**：主干已推进到 v0.28.x（较审计快照前进 4 个 minor 版本）；本文结论仍以固定基线为准，续引主干能力前应重审 v0.24.0→v0.28.x 的 WorkTask/Automation 变更。按提交路径直方图，Codeg 的开发重心是多 Agent 会话工作台（约七成路径变更在 ACP/会话面），WorkTask 看板约占 8%——本文取其 L3 机制为核心参考是按设计亮点归类，引用时应意识到这是该产品的次要模块，演进优先级可能低于会话面。

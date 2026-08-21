# 第一阶段、验证与自举

> 本文定义“交付什么、按什么顺序建、怎样证明”；对象和状态以[合同层](./spec/README.md)的四个模块合同为准，端到端步骤按[连接合同](./spec/connections.md)验收。本文属验证文档：可引用合同层词汇以指认被测合同，但不重定义它们。

## 第一阶段范围

第一阶段面向单用户、单机、单 Repo Instance 下的多个 Project，并交付 macOS/Linux 打包后的 Workbench/control/agentd/Workflow Engine/chat server/本地任务服务器生命周期。领域服务不依赖 Workbench 窗口存活，Windows 只保留原生适配边界。

| 模块 | 集成场景必须交付 | 第三方适配必须交付 |
| --- | --- | --- |
| [Project](./project.md) | Repo Room、Project Room（含只读 Project Overview）、Scoped Room、时间线、Composer、Context、Request、Memo/Artifact、至少两个并发 Invocation | chat server（Matrix 协议）经限时验证后作为第一阶段组件交付，Matrix 生态客户端即互操作面——这是对旧范围的显式反转；非 Matrix 平台桥接不作为出门条件 |
| [Task](./task.md) | 可访问 Kanban、以本地任务服务器为默认 content 后端、完成预览 | 本地任务服务器经限时验证后作为默认后端交付；Linear/GitHub 远端后端均通过身份/快照测试，其中一个通过完整字段读写与对账 |
| [Run](./run.md) | Workflow Revision 编译、Run 预览、只读图、Request、三选二 Gate、返工/regate | Conductor 经 workflow engine 受控端口通过本地分发与恢复测试 |
| [Agent](./agent.md) | ChangeSet/diff/证据、Execution Chat/结构化执行检查、xterm、精确 attach | Codex/Claude Code/OpenCode 能力探测；至少一个 harness 适配器和一个运行时后端通过完整契约测试；WezTerm 可选 |

四个场景由 Workbench 集成，但其命令必须可以由同一 service 供 CLI 或外部适配器使用。

## 公共 CLI

公共二进制固定为 `hctl2`：

| 范围 | 第一阶段命令 |
| --- | --- |
| 运维 | `init`、`start`、`status`、`doctor`、`export` |
| Project / Chat Room | `room list\|show`、`request list\|show`；复杂编辑暂由 Workbench 提供 |
| Task / Kanban | `task create\|update\|adopt\|move\|complete\|reopen\|cancel` |
| Run / Workflow | `run show\|preview\|start\|pause\|cancel`；修改动作先预览确认 |
| Agent / Terminal | `terminal inspect\|attach\|resume\|replay`；必须指向精确 descriptor |

CLI 没有隐藏权限，也不直接写治理账本、执行面 content 服务器或运行时后端。

## 明确不做

- 多用户组织/RBAC、云队列、多主机调度和 Conductor 高可用；
- 用户级“总入口对话面”：用户进入产品即在某个 repo 之下操作，这是显式设计决定（见[来时路 §12](./references/decision-history.md)），不是待补功能；
- Windows 正式版本、浏览器/移动客户端和通用远程中继；
- 非 Matrix 平台的完整聊天桥接、任意第三方插件市场；
- 通用可视化 Workflow 编辑器或模型自由生成后直接部署；
- 不对绕过受控端口的外部写入（带外写入）做全局检测与自动补偿；第一阶段只管理受控端口发出的意图，并把外部平台上的变化当作漂移/快照回读；
- 同时完成 Linear 与 GitHub 两套完整双向适配器；
- 多 Task Run 的分支/合并政策；第一阶段每个 Run 只绑定 0..1 个 Task Revision。

## 实现阶段

施工顺序与信任阶梯是两个维度：本节的 P 表回答「先建什么」，「自举阶段」的 B 表回答「什么时候敢切换事实」。建完不等于敢用——P2 的代码可以很快写完，B1 的晋级要靠真实试用去挣；两表一一映射，工程进度与敢用程度互相校验。

| 阶段 | 建什么 | 达成 |
| --- | --- | --- |
| P0 · 选型确认 | 跑「开工前限时验证」四项：Tuwunel、Vikunja、conductor-oss 单机栈、Zellij；产出 ADR 定案与固定版本 | 四个执行面系统各定一个可运维版本 |
| P1 · 底座 | 账本 schema、ID、command/query/event、单写者与恢复、`init/start/status/doctor`、四服务器一键启停 | B0 |
| P2 · 场景接入 | chat server 接入与 Repo/Project Room；任务服务器接入与身份映射/快照（「Kanban content 后端切片」前半） | B1 |
| P3 · 无 Run 自举 | 「纵向切片 A」全流程：Room Invocation、写租约、ChangeSet/diff/证据、完成预览 | B2（第一次真正自举） |
| P4 · 日常自举 | 并发 Invocation、Request、Receipt、冷启动恢复；此后按需启动远端任务后端验证 | B3 |
| P5 · 治理 | workflow engine 接入、Run/Seat/Gate、「纵向切片 B」、候选切换/三选二/regate | B4 → B5 |
| P6 · 发布 | 打包、升级、回滚隔离 | B6 |

## 纵向切片 A：无 Run 自举

1. 初始化 Repo Instance，创建 Project 与 Task Revision。
2. 从 Project Room 发起一次写入型 Room Invocation，冻结其 Execution Spec。
3. Harness 在隔离 worktree 和有效写租约下修改代码，产出 ChangeSet Revision 与测试证据。
4. Project 场景展示精确 diff；评审绑定 ReviewSubjectRef。
5. 有权 human actor 从 Kanban 完成预览提交「完成 Task」命令，写 Task Completion Receipt；Harness 不能代为提交。
6. 重启 Workbench/control/agentd 后，账本、worktree 归属、证据和投影一致且不重复副作用。

这是 B2 的第一次真正自举；它不等待 Workflow Engine 或 quorum。

## 纵向切片 B：完整治理

1. 从 Project 提炼 Task，批准 Workflow Revision 和 Engine Deployment。
2. 预览并启动绑定一个 Task Revision 的 Run。
3. Engine external task 产生 Obligation/Seat/Attempt，Harness 执行并返回提案。
4. 需要输入时创建 Project Request；答案 signal 回原执行。
5. B/C/D 对同一 ReviewSubjectRef 投票；备用候选只替换同一 Seat 的技术失败。
6. `changes_requested` 产生新 ChangeSet Revision，旧票失效并完整 regate。
7. 达到法定票数后写 Gate Receipt，再由 core 校验 SCM/合并事实。
8. task-bound Run 正常完成后，Run reducer 以稳定幂等键提交同一个「完成 Task」命令；Task 独立准入，失败类 Run 不终结 Task。
9. 任意步骤崩溃后通过 generation、outbox 和 readback 恢复，不重复外部效果。

## Kanban content 后端切片

为 Repo 选定 content 后端 → 映射 Project 分组与稳定实体 → 导入 Snapshot →（按需）升格采纳为 Task Revision → 按字段权威写回 → 回读确认。两类后端各验一条：本地任务服务器（默认）与远端 GitHub/Linear 直访。支持显式 refresh 与定期 reconcile，不依赖公网 webhook。创建结果未知、限流、外部修改、tombstone、重新绑定和无 Workbench 原生操作都必须有测试；后端关闭态或拖卡永远不直接产生 HCTL 完成；无契约的卡不进治理，惰性创建契约的升格路径必须有测试。

## 自举阶段

HCTL2 不会等到第一阶段完整交付才用来开发自己。自举按能力分级，而不是“上线前/上线后”二分（施工顺序见「实现阶段」，P 与 B 一一映射）；每一级都走普通的命令与查询入口，并包含真实的失败路径。打开过自己的仓库，或替自己生成过一次代码，都不算完成自举。

| 阶段 | 事实切换 | 晋级验收 |
| --- | --- | --- |
| B0 | ID、SQLite、command/query/event、进程和恢复底座 | 干净 clone 可启动；重启不丢状态；脚本只管进程和恢复 |
| B1 | Project Room 与本地 Task 影子试用 | Room/Task/草稿重启可恢复；引用稳定；明确不切换事实 |
| B2 | 无 Run 切片成为真实开发入口 | 从 Project Room 在隔离 worktree 完成一次真实的非文档代码改动和测试；越界写入被拒绝。第一次真正自举 |
| B3 | 接管自身待办、并发 Invocation、Request、Receipt 和冷启动恢复 | 连续至少 5 个真实变更，覆盖核心/界面/适配器与故障重启，全程无手工改库、无人肉转发 Prompt |
| B4 | 引入 Workflow Engine、Run、Seat 和独立 Gate | 一个真实变更走完“驳回 → 返工 → 重新评审 → 合并”，期间重启任一组件；无手工推进引擎或绕过 Receipt |
| B5 | 候选切换、三选二、regate 和完整故障恢复；第一阶段目标 | 完整治理切片在 HCTL 自身的真实变更上通过，而不只是测试样例 |
| B6 | 稳定版本 N 构建、验证、升级和回滚隔离环境中的 N+1 | N 驱动 N+1 的构建、测试、打包、升级和回滚；被测进程不覆盖治理它的 control 与数据库 |

旧工具在事实切换前可以作为执行者或逃生通道，不能继续保有平行 Project/Task/Run 账本。降级超过约定能力时回退到上一自举级别并留下审计记录。

B5 是第一阶段功能成熟度目标；正式发布、升级与回滚仍必须通过 B6，不能把“已能自举”当成可分发版本。

自举验收不得对 HCTL2 仓库、内置账号或测试环境设置隐藏的特例豁免：开发自身必须只使用公开的 Query/Preview/Submit/Subscribe、CLI 和受控端口，实际 Context、权限与证据均可检查；手工推进 Engine、直接改库、隐藏 Prompt/Context 或在产品外补签 Receipt 都不算通过。

## 契约测试矩阵

| 范围 | 必须证明 |
| --- | --- |
| Project / Chat Room | CJK 输入、结构化引用、草稿/游标/未读、并发写入得到一致时间线顺序（chat server 线性顺序 + 事务 ID 幂等）、并发流隔离；Repo Room 只把显式选中的来源链带入新 Project；Scoped Room 回填和同根因 Request 去重；Context 可解释、Room 历史可恢复（chat server 重同步 + 治理引用与冻结 digest 完整）；chat server 中的消息、反应或自动化不能成为命令，chat server 不可用时治理与施工命令照常、聊天入口安全降级；模型 Participant 的 `@`/建议不能创建 Invocation 或 fan-out，human 批准后自动携带来源/Context；无法证明身份的 Invocation 撤权并终止，Retry 产生新调用且旧结果被拒绝；mention 解析无唯一授权候选时明确失败，不按显示名模糊匹配或静默换人；原始消息、执行日志和模型总结不经「发布 Memo」命令不会成为 Memo |
| Task / Kanban | Task Revision、lifecycle、stage、正交 health、lane 投影与外部状态分离；非法 move/complete 拒绝；local state version 与 remote revision 不混用，过期邻项移动重算；本地 adoption 不要求伪造 Task Binding，外部 adoption 混用 Task Binding 版本时拒绝；未采纳契约使 Start/Complete fail-closed，明确 divergence 后新增 drift 仍使旧预览失效；active Run 未收口时 terminal intent 拒绝；human Kanban 完成与正常 Run reducer handoff 都走同一「完成 Task」命令，Harness/LLM/adapter/外部已关闭冒充 actor 均拒绝；同一规范实体跨 Project/connection/placement 不得产生第二个 Task，禁用 binding 也不释放映射；content 后端按 Repo 选定，跨后端相对移动拒绝；Project 分组映射（父任务/milestone/标签降级）有测试；无契约 Task 的看板终态只是 content 投影，完成命令仍需先升格契约 |
| Run / Workflow | 编译/Profile 拒绝、0..1 Task 绑定、Engine mutation 只有 control；过期或未回读确认的 Engine lease/deadline 不能触发超时与候选切换；dispatch ACK 丢失允许待启动→丢失并用新 Attempt 恢复，已交提案不被误当成功；retry 只产生一个新 Obligation并隔离旧 Seat/Attempt，候选耗尽和 Request expiry 类型化收口，所有 Run 过渡态可失败/替代；dynamic fork 超出冻结 Seat 模板/recipient/基数/预算时拒绝；placement 变更留下不可变审计；Gate backup 改变参与者或任一 Context/Skill/policy ref 时拒绝，作者不能占必需 reviewer Seat；quorum-unreachable 沿冻结失败边推进；失败/已取消/被替代 Run 不终结 Task，quorum/regate 和迟到结果拒绝 |
| Agent / Terminal | 能力探测、ChangeSet 单 writer、精确 Revision/digest、runtime generation；无法证明旧 writer 已 fence 时隔离旧 worktree且不得重授租约，失败清理不丢唯一未封存/未跟踪修改；本地/远端 SCM 集成都先持久 intent 并回读，target-head 竞争或 ACK 未知时不得签成功 Receipt；冲突观测按来源证据仲裁；Execution Chat 的错误 owner/generation 输入和无 provenance Share 均拒绝；Harness 内调用 Task terminal/Room fan-out 命令因 execution provenance 拒绝；control 签发 descriptor、agentd 终端网关校验，观察、输入、Attempt 控制与安全输入权限分离；attach/resume/replay、IME/背压/慢客户端隔离 |
| 连接 / 端口 | 每条 handoff 固定 source ref/digest 与唯一 binding；client/port 权限分离；actor provenance 不能由 payload 自报；dispatch/result 迟到拒绝；外部 effect ACK 未知不重复且 adapter 不写 Receipt；Harness 绕过端口的写能力被拒绝，外部 drift 只形成 Snapshot/观测而不是结果 |
| 系统 | 第二 control/agentd 只读或拒绝；命令幂等；commit/ACK 各崩溃点回读；schema migration、投影重建；metadata 账本备份/恢复，每个执行面 content 服务器（chat server、本地任务服务器、Workflow Engine）的备份与恢复；content 服务器宕机不阻断治理命令，从 Git 结晶回灌不得伪造未结晶判决；clone 本地运行目录（锁与缓存）删除后可完整对账重建、不丢事实；一键启停下各服务器的启动顺序与健康检查；旧 generation 与越权适配器拒绝；等价对象的 JCS 规范摘要一致、内容篡改被 digest 校验拒绝；打包后的整窗启动/退出/升级和安全边界 |
| 扩展 / 打包 | 自声明 trust、有副作用的 discovery、静默 install/upgrade、非本地未认证 Conductor、renderer Node/raw IPC/远程脚本或不满足下述源码合规门禁时均拒绝 |
| Workbench 信息架构 | 单 Project Overview 与全局「需要关注」都是可重建的只读导航投影，不产生第五场景或写状态；打开入口按 repo 选择并统一映射到控制面连接（打开本地 repo = 连接或拉起本机控制面再定位仓库；第一阶段远程入口隐藏或安全拒绝）；进入 Project 默认打开 Project Room，deep link 保留返回路径；同一 Request ID 跨 Room/Task/Run 聚合且不能从聚合面直接改状态；「创建 Project」命令提升预览允许删减、补充、去敏并显示来源回链；Trigger Preview 展示实际执行者、Context/Skill、权限、预算和 fan-out |
| Workbench 输入与无障碍 | Board 移动、Request 操作和 Run 浏览在 mouse/touch/keyboard/screen reader 下等价；输入优先级为 IME composition → 已聚焦 terminal → modal/composer → 当前场景 → 全局快捷键，任何上层快捷键都不能截获正在组合或发往 terminal 的输入 |
| 产品 | 用户十秒内能回答 Project 目标、Task 状态、Run 阻塞、所需动作、当前 Harness 和证据版本；正常成功保持安静；HCTL2 仓库自举不使用隐藏的特例豁免或产品外补签事实 |

交付测试检查可观察行为，不复述模块状态机。模块新增合同必须在这里增加一个失败用例，而不是再建一份不变量文档。

## 选型判据

执行面各系统的实现选型按以下准入标准评估；实现名只出现在本文与[实现证据](./references/implementation-evidence.md)，设计层正文只用系统角色名：

1. **接口公开干净**：优先公开协议与文档化 API，保证第三方 UI 与场景客户端互操作——chat 场景直接采用 Matrix 协议；任务场景没有同级的开放协议，选择 API 完整、支持条件写入的实现并以受控端口隔离。
2. **运维简单**：本机执行模式优先单二进制、内嵌存储、低资源占用的后端。
3. **生命周期可托管**：随 HCTL 一键启停（由 control 编排），支持备份、恢复与固定版本升级。
4. **Conductor 先例三件套**：每个新增系统都设开工前限时验证；验证失败重开对应 ADR；不自研第二个同类系统。

## 开工前限时验证

P0 的内容就是本节。四项选型已全部拍板，验证因此从“选谁”变为“确认能落地”：按 Conductor 先例三件套照跑，通过则固定版本进入 P1，失败才重开对应 ADR。

1. **workflow engine（conductor-oss，已拍板）**：固定版本打包、启动、升级、备份和恢复。其持久化仅支持 Redis/Postgres/MySQL（无 SQLite），验证目标是单机可运维的最小持久化组合与打包重量；失败重开 Engine ADR，不自研第二引擎。
2. **运行时后端（Zellij，已拍板；tmux 为记录在案的降级方向）**：attach、输入、resize、重启、残留进程、macOS/Linux 全套测试，另加三点——headless 启动时的终端查询应答（无人 attach 期间 TUI 的能力查询须有应答）、增强键盘协议（kitty keyboard protocol）下各 harness 的按键实测、内建 web 客户端保持默认关闭（需要瘦身时可用 `zellij-no-web` 构建裁剪）；打包重量与常驻内存实测记入实现证据。
3. **chat server（Tuwunel，已拍板；Continuwuity 为记录在案的备选）**：Rust 单二进制 + SQLite 的 Matrix homeserver。拍板理由：接口更 API 化、与 Synapse 参考实现兼容性更强；单二进制预编译包、运维压力低；AppService 注册程序化，不靠房间内发命令。验证要点：本地分发、账号与房间管理 API、事务 ID 幂等、单 homeserver 线性顺序、重同步、备份恢复与一键启停。
4. **task server（Vikunja，已拍板）**：Go 单二进制、SQLite、REST API + webhooks。验证要点：看板语义（排序令牌、泳道）、观测机制（webhook/轮询）、身份稳定性、备份恢复与一键启停。git-bug（零服务器、任务存于 git refs）降为记录在案的对照——仅在验证失败重开 ADR 时再取，且须显式接受“任务 content 也在 Git”的模型例外并记入决策历史。
5. **远端任务后端（移出 P0）**：Linear/GitHub 的身份、字段权威、outbox/readback、限流和 tombstone 验证延至 P4 之后按需启动——合同未押注它，双向适配是五项中最贵的一项。

## 技术基线

Rust control/core/agentd；Electron + React 19 Workbench；SQLite + FTS5 与 Git；Tiptap、React Aria、React Flow + Dagre、xterm.js。执行面服务器经受控端口接入、由 control 托管一键启停：conductor-oss（workflow engine）、Matrix homeserver（Tuwunel；Continuwuity 备选）、本地任务服务器（Vikunja）、运行时后端（Zellij；tmux 为降级方向）。选择受契约测试约束，不能为了保留依赖而削弱模块边界。

任何采用、移植或 vendor 的外部源码都必须固定已审阅 commit，核验目标文件及依赖许可证，保留 license/copyright/attribution 与修改记录，并用 HCTL contract tests 隔离上游漂移；任一项缺失即不得进入分发产物。

## 未决问题

- ~~Room 的协作历史到底放哪里~~ 已了结：消息 content 归 chat server，metadata 随用户级控制面走，结晶进 Git（见[来时路 §12](./references/decision-history.md)）；
- ChangeSet/PR 默认基数与后续多 Task Run 的集成策略；
- Repo Room 跨 clone 迁移、隐私和保留期限；
- Project 拆分/合并和 Task 依赖的产品表达；
- Scoped Room 自动归档策略；
- 首批原生会话导入的范围与长期维护预算（能力定义见 [Agent 设计正文](./agent.md#原生会话导入)）；
- 多主机与远程的实施：架构方向已定（Workbench 连接本机或远程控制面，见[三面架构](./architecture.md#三个面)），未决的是远程连接的认证与传输、多主机执行现场的编排、Windows 与多用户权限；
- 成本/预算硬上限及运行中耗尽的交互；
- 第一阶段之后首个非 Matrix 聊天平台桥接（Matrix 生态客户端已随 chat server 天然可用）。

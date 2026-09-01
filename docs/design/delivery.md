# 第一阶段、验证与自举

> 状态：交付文档（非规范） · 草案 v0.15.6<br>
> 日期：2026-09-02

> 本文定义“交付什么、按什么顺序建、怎样证明”；对象和状态以[约束层](./spec/README.md)的四个模块约束为准，端到端步骤按[连接约束](./spec/connections.md)验收。本文属验证文档：可引用约束层词汇以指认被验证的约束条款，但不重定义它们。

## 第一阶段范围

第一阶段面向单用户、单机、单 Repo Instance 下的多个 Project，并交付 macOS/Linux 打包后的 Workbench、control、Herdr、workflow engine、chat server 和本地任务服务器生命周期。领域服务不依赖 Workbench 窗口存活，Windows 只保留原生适配边界。

范围按实现阶段分两组：P2 的验收条件可通过公共 CLI 和各 content 系统原生界面完成；P3 的验收条件覆盖 Workbench 场景。

| 模块 | P2 出门（control + CLI + content 系统） | P3 出门（Workbench 场景） | 执行面与第三方适配 |
| --- | --- | --- | --- |
| [Project](./project.md) | Repo Room、Project Room、Scoped Room 的治理事实与命令、Context、Request、Memo/Artifact、至少两个并发 Invocation——治理走 CLI，聊天走 Matrix 客户端 | 时间线、Composer、Trigger Preview、只读 Project Overview | chat server（Matrix 协议）经限时验证后作为第一阶段组件交付，Matrix 生态客户端可直接访问；非 Matrix 平台经 Matrix 桥接生态接入，HCTL 不自建桥接 |
| [Task](./task.md) | 以本地任务服务器为默认 content 后端、CLI 完整 Task 管理与完成预览；Vikunja 原生 Done 在能力满足时可请求同一完成命令 | Workbench Board（拖放、泳道、后续动作入口） | 本地任务服务器经限时验证后作为默认后端交付；Linear/GitHub 远端后端均通过身份/快照测试，其中一个通过完整字段读写与对账 |
| [Run](./run.md) | Workflow Revision 编译、Run 预览/启动/暂停/取消、三选二 Gate、返工/regate、Request | 只读图与节点/席位/尝试的渐进展开 | Dagu 经 workflow engine 受控端口通过检查点等待/完成/回读的接口测试 |
| [Agent](./agent.md) | ChangeSet/diff/证据、写租约与代次、terminal inspect/attach/replay；按 Execution Spec 验证受租约输入与原生交互输入两种恢复等级 | Execution Chat/结构化执行检查、xterm、精确 attach UI | Codex/Claude Code/OpenCode 能力探测；至少一个 harness 适配器与 Herdr v0.8.2 通过第一阶段契约测试；Herdr 官方 TUI 是原生 Terminal 客户端，WezTerm 可选 |

P3 的 Workbench 把四类供应端客户端与 HCTL 命令入口组合到一个桌面，但不引入任何 CLI 不可达的 HCTL 命令；同一命令服务供 CLI、Workbench 与外部适配器使用。消息、卡片和终端输入仍按各供应端的公开协议及其绑定中声明的能力处理。Workbench 不因集成而升权；关掉 Workbench 不影响服务和执行。

客户端动作与 provider 事件的分类及准入以[系统约束](./spec/system.md#客户端动作与-provider-事件)为准；P2 用 CLI 提供全部 HCTL 命令，不把 Workbench 设成必需组件。

## 公共 CLI

公共二进制固定为 `hctl2`：

| 范围 | 第一阶段命令 |
| --- | --- |
| 运维 | `init`、`start`、`status`、`doctor`、`export`、`backup create\|verify`、`restore preview\|apply` |
| Repo / Project | `repo register\|list\|show`、`repo instance attach\|list\|show\|detach`、`project create\|list\|show\|update\|archive\|restore` |
| Participant / Context | `participant create\|update\|list\|show`、`role bind\|unbind\|list`、`context show\|preview` |
| Project / Chat Room | `room list\|show`、`invocation list\|show\|preview\|start\|cancel\|retry`、`request list\|show\|resolve` |
| Task / Kanban | `task create\|update\|adopt\|move\|complete\|reopen\|cancel` |
| Run / Workflow | `workflow list\|show\|register\|compile\|approve`、`run list\|show\|preview\|start\|pause\|resume\|replace\|cancel`；修改动作先预览确认 |
| Agent / Integration / Terminal | `changeset show\|diff`、`integration preview\|submit\|show`、`terminal inspect\|attach\|replay`；Terminal 命令必须指向精确连接票据 |

CLI 没有隐藏权限，也不直接写治理账本、执行面 content 服务器或 Agency。`terminal attach` 只建立观察或输入通道，不恢复任何领域对象。Run 的语义恢复或替换使用 `run resume|replace`；Room Invocation 的再次施工使用创建新 Invocation 与新代次的 `invocation retry`。终端重连只恢复观察或输入通道，不得改变 Run 或 Room Invocation 的生命周期状态。

## 明确不做

- 多用户组织/RBAC、云队列、多主机调度和 Dagu coordinator/worker 集群；
- 用户级“总入口对话面”：用户进入产品即在某个 repo 之下操作，这是显式设计决定（见[来时路 §12](./references/decision-history.md#12-场景数据的三分metadata--content--artifact)），不是待补功能；
- Windows 正式版本、浏览器/移动客户端和通用远程中继；
- 自建聊天桥接（永久不做，不只是第一阶段：非 Matrix 平台经 homeserver 侧 Matrix 桥接生态接入，HCTL 只保留桥接用户的身份映射）、任意第三方插件市场；
- 通用可视化 Workflow 编辑器或模型自由生成后直接部署；
- 不对任意外部写入做全局检测与自动补偿；各模块只接纳明确列出的 content、human 命令请求和运行时输入路径，其余 provider mutation 与「合入 ChangeSet」命令之外的目标 ref 改写只作分歧/漂移/快照回读；
- 同时完成 Linear 与 GitHub 两套完整双向适配器；
- 多 Task Run 的分支/合并政策；第一阶段每个 Run 只绑定 0..1 个 Task Revision。

## 实现阶段

施工顺序从最小可用链路开始：先做可丢弃的实现验证，再准备 Herdr 与本地工具箱，随后由 control 和 CLI 接管治理。各 content 系统的产品打包、备份恢复与一键生命周期在 control 出现后，按其首次被使用的阶段完成，而不是先把四套服务器全部产品化。

下面用两张表回答两个不同问题。P0—P3 表示实现顺序；B0—B6 表示 HCTL2 可以接管自身开发事实的程度。B0—B5 都发生在 P2，B6 在 P3 末验收。组件完成实现并不自动提高自举等级。

| 阶段 | 建什么 | 达成 |
| --- | --- | --- |
| P0 · 探路 | 只对 HCTL 与已选实现实际使用的 API 和行为做限时、可丢弃的协议验证并记录实现证据，不替第三方验其自身功能；临时数据与拼装环境不进入产品生命周期。失败则重新评估并修订对应选型决定与 decision-history | 关键假设有证据，不宣称四个外部服务已可运维 |
| P1 · 备装 | 固定并打包 Herdr，并实现 `hctl2-tool` 的现场 Git 职责。Harness 仍按仓库配置运行代码检查，CI 负责强制；这些检查不进入工具箱意图回路。P1 只验证 Herdr 的启动、观察和停止，不产生 HCTL metadata 或 Receipt，因此不得称为自举 | Herdr 与本地工具箱就位，未切换治理事实 |
| P2 · 接钥匙 | `hctl2-control` 与覆盖 B0–B5 的公共 `hctl2` CLI 承载治理；各系统首次使用时完成打包、备份恢复和一键生命周期。Matrix/Vikunja 原生界面承担 content；Herdr TUI 按 Execution Spec 输入策略使用；Dagu console 只用于管理和诊断，且到 B4 才是必需项 | B0 → B5 |
| P3 · 装门面 | `hctl2-workbench` 与发布链；Workbench 不承担任何 B0–B5 晋级 | B6 |

## 纵向切片 A：无 Run 自举

1. 注册 Repo、挂接 Repo Instance，创建 Project 与 Task Revision。
2. 从 Project Room 发起一次写入型 Room Invocation，冻结其 Execution Spec。
3. Harness 在隔离 Git 工作树和有效写租约下修改代码，产出 ChangeSet Revision 与测试证据。
4. Project 场景展示精确 diff；评审绑定 ReviewSubjectRef。
5. 有权 human actor 提交固定 ChangeSet Revision、target ref、expected target head 与证据的 integration intent；control 先持久化，`hctl2-tool` 执行本地 Git 集成并 readback，确认后写唯一 Integration Receipt。
6. 有权用户本人通过 CLI 完成预览提交「完成 Task」命令，或通过已验证的 Vikunja Done 映射请求同一命令；Task 准入校验精确 Integration Receipt 后写 Task Completion Receipt，Harness 不能代为提交，provider Done 本身也不是 Receipt。
7. 重启 control、Herdr 与已使用的 content 后端后，账本、Git 工作树归属、integration intent/Receipt、证据和 CLI 投影一致且不重复副作用。

这是 B2 的第一次真正自举；它不等待 workflow engine 或法定票数。

## 纵向切片 B：完整治理

1. 从 Project 提炼 Task，批准 Workflow Revision 和 Engine Deployment。
2. 预览并启动绑定一个 Task Revision 的 Run。
3. control 观察到 Engine 检查点进入等待态，在账本创建 Obligation/Seat/Attempt，Harness 执行并返回提案。
4. 需要输入时创建 Project Request；答案 signal 回原执行。
5. B/C/D 对同一 ReviewSubjectRef 投票；备用候选只替换同一 Seat 的技术失败。
6. `changes_requested` 产生新 ChangeSet Revision，旧票失效并完整 regate。
7. 达到法定票数后写 Gate Receipt；有权 human actor 或冻结 reducer 再提交固定 ChangeSet Revision、target、expected head 与 Gate 证据的 integration intent。
8. control 先持久化 intent/outbox，`hctl2-tool`（本地）或 adapter（远端）执行并 readback；只有确认目标事实后才写唯一 Integration Receipt，结果未知时不得签成功或盲重投。
9. task-bound Run 按约束正常完成后，Run reducer 以稳定幂等键提交同一个「完成 Task」命令；Task 独立准入并校验精确 Integration Receipt，失败类 Run 不终结 Task。
10. 任意步骤崩溃后按[连接约束的失败与恢复](./spec/connections.md#失败与恢复)对账恢复，不重复外部效果；本文不另写一份恢复算法。

## Kanban content 后端切片

Kanban 切片依次完成后端选择、Project 分组映射、Snapshot 导入、按需采纳契约、字段写回和结果回读。本地任务服务器与一个远端后端各走通一次主线，并支持显式刷新与定期对账，不依赖公网 webhook。

另用独立失败用例覆盖结果未知、限流、外部修改、tombstone、重新绑定、无 Workbench 操作、无契约卡和 Done 请求拒绝。外部终态永远不直接写 HCTL 完成；Vikunja 明确的 Done 变化只有在操作者、版本、幂等依据和当前回读齐全时，才能请求同一完成命令。

## 自举阶段

HCTL2 不会等到第一阶段完整交付才用来开发自己。自举按能力分级，而不是“上线前/上线后”二分（施工顺序见「实现阶段」：B0–B5 全部发生在 P2 内部，B6 对应 P3 末）；每一级都走普通的命令与查询入口，并包含真实的失败路径。打开过自己的仓库，或替自己生成过一次代码，都不算完成自举。

| 阶段 | 事实切换 | 晋级验收 |
| --- | --- | --- |
| B0 | ID、SQLite、command/query/event、进程和恢复底座 | 干净 clone 可启动；重启不丢状态；脚本只管进程和恢复 |
| B1 | Project Room 与本地 Task 影子试用 | Room/Task/草稿重启可恢复；引用稳定；明确不切换事实 |
| B2 | 无 Run 切片成为真实开发入口 | 从 Project Room 在隔离 Git 工作树与有效写租约下完成一次真实的非文档代码改动和测试；Harness 环境中取不到 HCTL 交付的集成/外部写凭据；声明了执行加固的 Profile 按声明生效并留记录，宿主施加不了则不启动。第一次真正自举 |
| B3 | 接管自身待办、并发 Invocation、Request、Receipt 和冷启动恢复 | 连续至少 5 个真实变更，覆盖核心/界面/适配器与故障重启，全程无手工改库、无人肉转发 Prompt |
| B4 | 引入 workflow engine、Run、Seat 和独立 Gate | 一个真实变更走完“驳回 → 返工 → 重新评审 → 合并”，期间重启任一组件；无手工推进引擎或绕过 Receipt |
| B5 | 候选切换、三选二、regate 和完整故障恢复；第一阶段目标 | 完整治理切片在 HCTL 自身的真实变更上通过，而不只是测试样例 |
| B6 | 稳定版本 N 构建、验证、升级和回滚隔离环境中的 N+1 | N 驱动 N+1 的构建、测试、打包、升级和回滚；被测进程不覆盖治理它的 control 与数据库 |

旧工具在事实切换前可以作为执行者或逃生通道，不能继续保有平行 Project/Task/Run 账本。降级超过约定能力时回退到上一自举级别并留下审计记录。

B5 是第一阶段功能成熟度目标；正式发布、升级与回滚仍必须通过 B6，不能把“已能自举”当成可分发版本。

自举验收不得对 HCTL2 仓库、内置账号或测试环境设置隐藏的特例豁免：开发自身必须只使用公开的 Query/Preview/Submit/Subscribe、CLI 和受控端口，实际 Context、权限与证据均可检查；手工推进引擎、直接改库、隐藏 Prompt/Context 或在产品外补签 Receipt 都不算通过。

契约测试矩阵见 [contract-tests.md](./contract-tests.md)。

## 选型判据

执行面各系统的实现选型按以下准入标准评估；实现名只出现在本文与[实现证据](../research/README.md)，设计层正文只用系统角色名：

1. **接口公开干净**：优先公开协议与文档化 API，保证第三方 UI 与场景客户端互操作——chat 场景直接采用 Matrix 协议；任务场景没有同级的开放协议，选择 API 完整、支持条件写入的实现并以受控端口隔离。
2. **运维简单**：本机执行模式优先单二进制、内嵌存储、低资源占用的后端。
3. **生命周期可托管**：随 HCTL 一键启停（由 control 编排），支持备份、恢复与固定版本升级。
4. **选型三件套**：每个新增系统都设限时、可丢弃的开工前验证，且只验 HCTL 实际依赖的 API 与行为（边界见「开工前限时验证」）；验证失败就重开并修订对应选型决定及 [decision-history](./references/decision-history.md)，不另造 ADR catalog；不自研第二个同类系统。

## 开工前限时验证

P0 只验证 HCTL 实际依赖的 API 和行为，不重新比较已拍板的候选。探针使用可删除环境，只留下实现证据、固定版本和产品化要求；通过探针不等于已经具备一键生命周期、备份或升级。各探针在对应场景首次消费前完成，不构成全局门禁。

chat 与 task 探针在 B1 首次消费前完成，Herdr 探针在 B2 前完成，workflow engine 探针只须在 B4 前完成，不能阻塞 B2。失败时重开选型并修订对应决定与 decision-history。

1. **workflow engine（Dagu，已拍板）**：DAG 提交与启动/暂停/恢复/取消回读、`human.task` 等待/完成/回读和引擎自行推进或重试的分歧检查均按 HCTL 实际调用面核对；生成物只用机械结构与无进程的 `human.task`，Obligation 身份与隔离仍由 HCTL 账本承担，结论与固定源码证据见 [Dagu 与候选复审](../research/workflow-engines.md#e-l2-dagu)。
2. **Agency（Herdr，已拍板）**：固定基线为 [`v0.8.2`](https://github.com/herdrdev/herdr/releases/tag/v0.8.2)（Apache-2.0；HCTL 当前消费 macOS/Linux × arm64/x86_64 官方单二进制）。Herdr 直接按规格启动 Harness，持有进程、PTY 和终端会话，并提供 API 与原生 TUI；HCTL 只实现适配代码，不再放置独立 Agency 组件或下一层终端运行服务。
   P0 验证版本协商，workspace/tab/pane/terminal 创建与定位，输入与 resize，观察与断线重连，停止与退出状态，以及恢复等级能否如实翻译。已确认的限制包括：原生输入不经 HCTL 输入租约、API 与原生 controller 可交错写入、事件 ring 没有公开 sequence/gap、退出和停止回读不足。
   这些功能在补齐前按低信任或不支持处理，不在 HCTL 内另写终端服务。源码、API、macOS RSS 与历史运行时对照数据见 [Herdr 运行服务验证记录](../research/runtime/agency-runtime-validation-20260829.md)。
3. **chat server（Tuwunel，已拍板；Continuwuity 为备选）**：账号与房间管理、AppService 注册和事件投递、按事件 ID 读取正文及房间加密状态回读，均按 Chat 端口调用面核对。
   事务 ID、事件顺序与重同步沿用 Matrix homeserver 约束；低内存配置、RocksDB/media 备份和托管生命周期留到 B1 产品化。结论见 [homeserver 选型证据](../research/matrix-homeserver.md#e-l4-matrix-homeserver)及[运维与资源占用](../research/README.md#已选外部服务的运维与资源占用)。
4. **task server（Vikunja，已拍板）**：卡片与分组读写、稳定归属回读、条件写入、webhook/轮询变化观测和实体 ID 均按 Task 端口调用面核对；排序与看板语义沿用 Vikunja，备份恢复和托管生命周期留到 B1 产品化，git-bug 只保留为重开选型时的对照，结论与固定源码证据见 [任务后端复审](../research/task-backends.md#e-l3-vikunja)。
5. **远端任务后端（移出 P0）**：Linear/GitHub 的身份、字段权威、outbox/readback、限流和 tombstone 验证延至 P2 的日常自举子阶梯之后按需启动——约束未押注它，双向适配是五项中最贵的一项。

## 打包策略（选型判断，首次消费时产品化）

分界线是**碰不碰宿主机现场**：

- **必须原生**：Herdr、harness、`hctl2-control`、`hctl2-tool` 与 CLI——要碰真实 Git 工作树、PTY 与 OS 密钥串，不进容器；macOS/Linux 原生分发。Herdr 直接消费摘要锁定的官方单二进制并随包保留上游许可证，不维护另一套终端运行服务或自主构建链。
- **服务器按服务声明形态**：control 出现后，生命周期托管器在服务首次被消费前声明原生发行目标。Linux x86_64、Linux arm64、macOS arm64 与 macOS x86_64 分别构建。macOS 最低基线为 15：`lock.json` 固定 15.0，托管 Tuwunel 的两种架构 Mach-O 均声明 `minos 15.0`，macOS 依赖与发布 CI 均在 macOS 15 arm64/Intel runner 验证；Herdr 官方制品声明 11.0，尚未落入源码树的 Tauri 2 Workbench 没有更高要求。
  Dagu、Vikunja、Herdr 使用官方原生发布物。Tuwunel 上游无 Darwin 制品，HCTL2 在自己的 GitHub Release 托管按 SHA-256 锁定的 macOS 包；日常打包消费托管制品，源码构建只用于更新托管制品。各发行目标共用锁定的 Cinny 官方 Web 发行包，不混用缓存、动态库闭包或生命周期验证。
- **Docker 不做统一打包方式，也不做 Harness 的沙箱或桌面形态**：执行面一半天生进不了容器；macOS/Windows 上容器即 Linux 虚拟机，有授权与资源开销问题。第一阶段的 Linux/macOS 发行均为原生包，最终用户无需安装 Docker Desktop；执行加固只按宿主 OS 原生机制施加。
- Windows 不在第一阶段范围。Herdr v0.8.2 已提供官方 Windows x86_64 发行物，但 HCTL 当前的构建与生命周期验证矩阵只有 Linux/macOS，Tuwunel 也未见官方 Windows 包；未来须让完整 Windows 包重新通过同一约束与兼容矩阵，当前不宣称支持 Windows。

## 技术基线

技术栈包括 Rust control/tool 与 Herdr 适配代码；Tauri 2 + React 19 Workbench（GPUI 原生备选，Electron 安全网）；SQLite + FTS5 与 Git；以及 Tiptap、React Aria、React Flow + Dagre、xterm.js。

执行面服务器经受控端口接入，由 control 托管一键启停：Dagu（workflow engine）、Matrix homeserver（Tuwunel；Continuwuity 备选）、本地任务服务器（Vikunja）和 Herdr（Agency）。Chat Room 另随包提供 Cinny 内容客户端。

精确版本、实测 footprint 与运维分级见[实现证据](../research/README.md#已选外部服务的运维与资源占用)。Workbench 桌面壳的选型证据、实机探针与安全网回退条件见[桌面壳证据](../research/workbench-shell.md#e-workbench-shell)与[重开调研](../research/workbench-shell-reopen-20260826/README.md)。选择受契约测试约束，不能为了保留依赖而削弱模块边界。

任何采用、移植或 vendor 的外部源码都必须固定已审阅 commit，核验目标文件及依赖许可证，保留 license/copyright/attribution 与修改记录，并用 HCTL contract tests 隔离上游漂移；任一项缺失即不得进入分发产物。

## 未决问题

- ChangeSet/PR 默认基数与后续多 Task Run 的集成策略；
- Repo Room 的隐私与保留期限——端到端加密不是答案（HCTL 房间对 control 明文可读），只能由 homeserver 侧访问控制、传输/存储加密与保留策略回答；
- Project 拆分/合并和 Task 依赖的产品表达；
- Scoped Room 自动归档策略；
- 首批原生会话导入的范围与长期维护预算（能力定义见 [Agent 设计正文](./agent.md#原生会话导入)）；
- 多主机与远程的实施：架构方向已定（Workbench 连接本机或远程控制面，见[三面架构](./architecture.md#三个面)），未决的是远程连接的认证与传输、多主机执行现场的编排、Windows 与多用户权限；
- 成本/预算硬上限及运行中耗尽的交互；

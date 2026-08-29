# 第一阶段、验证与自举

> 本文定义“交付什么、按什么顺序建、怎样证明”；对象和状态以[合同层](./spec/README.md)的四个模块合同为准，端到端步骤按[连接合同](./spec/connections.md)验收。本文属验证文档：可引用合同层词汇以指认被测合同，但不重定义它们。

## 第一阶段范围

第一阶段面向单用户、单机、单 Repo Instance 下的多个 Project，并交付 macOS/Linux 打包后的 Workbench/control/agentd/Workflow Engine/chat server/本地任务服务器生命周期。领域服务不依赖 Workbench 窗口存活，Windows 只保留原生适配边界。

范围按施工序两段陈列（见「实现阶段」）：P2 出门条件经公共 CLI 与各 content 系统原生界面即可达、可测，不等 Workbench；P3 出门条件是 Workbench 场景本身。

| 模块 | P2 出门（control + CLI + content 系统） | P3 出门（Workbench 场景） | 执行面与第三方适配 |
| --- | --- | --- | --- |
| [Project](./project.md) | Repo Room、Project Room、Scoped Room 的治理事实与命令、Context、Request、Memo/Artifact、至少两个并发 Invocation——治理走 CLI，聊天走 Matrix 客户端 | 时间线、Composer、Trigger Preview、只读 Project Overview | chat server（Matrix 协议）经限时验证后作为第一阶段组件交付，Matrix 生态客户端即互操作面；非 Matrix 平台经 Matrix 桥接生态接入，HCTL 不自建桥接 |
| [Task](./task.md) | 以本地任务服务器为默认 content 后端、CLI 完整 Task 管理与完成预览 | Workbench Board（拖放、泳道、后续动作入口） | 本地任务服务器经限时验证后作为默认后端交付；Linear/GitHub 远端后端均通过身份/快照测试，其中一个通过完整字段读写与对账 |
| [Run](./run.md) | Workflow Revision 编译、Run 预览/启动/暂停/取消、三选二 Gate、返工/regate、Request | 只读图与节点/席位/尝试的渐进展开 | Dagu 经 workflow engine 受控端口通过检查点等待/完成/回读的接缝测试 |
| [Agent](./agent.md) | ChangeSet/diff/证据、写租约与代次、terminal inspect/attach/replay 凭精确票据 | Execution Chat/结构化执行检查、xterm、精确 attach UI | Codex/Claude Code/OpenCode 能力探测；至少一个 harness 适配器和一个运行时后端通过完整契约测试；WezTerm 可选 |

P3 的 Workbench 把四个场景集成到一个客户端，但不引入任何 CLI 不可达的命令：同一 command service 供 CLI、Workbench 与外部适配器使用。

第一阶段区分三类外部界面：Matrix/Vikunja 等原生界面是对应系统的 **content 客户端**，可以读写该系统拥有的消息或卡片，但不能提交 HCTL 治理命令；Engine console 是 provider 诊断面；裸 `tmux attach-session` 等运行时 provider 原生客户端是执行面的内容原生界面——不校验 control descriptor 与 input lease，因此不是合规 Terminal 客户端，其终端输入由 agentd 按带外输入入账。合规的第三方场景客户端必须使用公开的 Query/Preview/Submit/Subscribe，Terminal 通道则使用 control 签发、agentd 校验的 descriptor。P2 用公共 CLI 承载 B0–B5 所需的治理面，原生界面只验证 content 互操作，不把 provider 控制台冒充成 HCTL 客户端。

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
| Agent / Integration / Terminal | `changeset show\|diff`、`integration preview\|submit\|show`、`terminal inspect\|attach\|replay`；Terminal 命令必须指向精确 descriptor |

CLI 没有隐藏权限，也不直接写治理账本、执行面 content 服务器或运行时后端。`terminal attach` 只建立观察或输入通道，不恢复任何领域对象；Run 的语义恢复 / 替换使用 `run resume|replace`，Room Invocation 的再次施工使用创建新 Invocation 与新 generation 的 `invocation retry`，不能用终端重连偷渡 lifecycle 推进。

## 明确不做

- 多用户组织/RBAC、云队列、多主机调度和 Dagu coordinator/worker 集群；
- 用户级“总入口对话面”：用户进入产品即在某个 repo 之下操作，这是显式设计决定（见[来时路 §12](./references/decision-history.md)），不是待补功能；
- Windows 正式版本、浏览器/移动客户端和通用远程中继；
- 自建聊天桥接（永久不做，不只是第一阶段：非 Matrix 平台经 homeserver 侧 Matrix 桥接生态接入，HCTL 只保留桥接用户的身份映射）、任意第三方插件市场；
- 通用可视化 Workflow 编辑器或模型自由生成后直接部署；
- 不对绕过受控端口的外部写入（带外写入，含 Harness 或人在「合入 ChangeSet」命令之外直接改写本地目标 ref）做全局检测与自动补偿；第一阶段只管理受控端口发出的意图，并把外部平台与本地目标分支上的变化当作漂移/快照回读；
- 同时完成 Linear 与 GitHub 两套完整双向适配器；
- 多 Task Run 的分支/合并政策；第一阶段每个 Run 只绑定 0..1 个 Task Revision。

## 实现阶段

施工顺序以最小纵向切片闭环：先做可丢弃的实现探针，再做物理执行原语，随后由 control + CLI 接管治理；各 content 系统的产品打包、备份恢复与一键生命周期在 control 出现后，按其首次被切片消费时落地，而不是先把四套服务器全部产品化。P 表回答「先建什么」，「自举阶段」的 B 表回答「什么时候敢切换事实」——B0–B5 全部发生在 P2 内部（按其子阶梯晋级），B6 对应 P3 末。建完不等于敢用，两表互相校验。

| 阶段 | 建什么 | 达成 |
| --- | --- | --- |
| P0 · 探路 | 只对 HCTL 与已选实现之间的接缝做限时、可丢弃的协议探针并记录实现证据，不替第三方验其自身功能；探针脚本、临时数据与拼装环境不进入产品生命周期。失败则重开并修订对应选型决定与 decision-history | 关键假设有证据，不宣称四服务器已可运维 |
| P1 · 备装 | `hctl2-agentd`（会话持有、观测、租约原语）与 `hctl2-tool`（机械工具箱：commit 署名、lint、PR 正文机械拼装、memo 写入、git 有效变化侦测）。两者不依赖 control，standalone 可辅助开发；此时尚无 HCTL metadata、公开治理入口或 Receipt，因此明确不称真正自举 | 物理工具链就位，未切换治理事实 |
| P2 · 接钥匙 | `hctl2-control`（账本+命令服务）与覆盖 B0–B5 的公共 `hctl2` CLI 承载治理；按 B 阶梯首次消费 chat/task/runtime/workflow 时，分别完成对应系统的产品打包、备份恢复和一键生命周期。Matrix/任务后端原生界面只验证 content，Engine console 只诊断，raw tmux attach 只作 break-glass；合规第三方客户端必须走公开命令或 agentd 网关。Dagu 到 B4 才是必需项，不阻塞 B2 无 Run 切片 | B0 → B5 |
| P3 · 装门面 | `hctl2-workbench` 与发布链；Workbench 不承担任何 B0–B5 晋级 | B6 |

## 纵向切片 A：无 Run 自举

1. 注册 Repo、挂接 Repo Instance，创建 Project 与 Task Revision。
2. 从 Project Room 发起一次写入型 Room Invocation，冻结其 Execution Spec。
3. Harness 在隔离 worktree 和有效写租约下修改代码，产出 ChangeSet Revision 与测试证据。
4. Project 场景展示精确 diff；评审绑定 ReviewSubjectRef。
5. 有权 human actor 提交固定 ChangeSet Revision、target ref、expected target head 与证据的 integration intent；control 先持久化，`hctl2-tool` 执行本地 Git 集成并 readback，确认后写唯一 Integration Receipt。
6. 有权 human actor 从 Kanban 完成预览提交「完成 Task」命令，Task 准入校验精确 Integration Receipt 后写 Task Completion Receipt；Harness 不能代为提交。
7. 重启 control/agentd 与已消费的 content 后端后，账本、worktree 归属、integration intent/Receipt、证据和 CLI 投影一致且不重复副作用。

这是 B2 的第一次真正自举；它不等待 Workflow Engine 或 quorum。

## 纵向切片 B：完整治理

1. 从 Project 提炼 Task，批准 Workflow Revision 和 Engine Deployment。
2. 预览并启动绑定一个 Task Revision 的 Run。
3. control 观察到 Engine 检查点进入等待态，在账本创建 Obligation/Seat/Attempt，Harness 执行并返回提案。
4. 需要输入时创建 Project Request；答案 signal 回原执行。
5. B/C/D 对同一 ReviewSubjectRef 投票；备用候选只替换同一 Seat 的技术失败。
6. `changes_requested` 产生新 ChangeSet Revision，旧票失效并完整 regate。
7. 达到法定票数后写 Gate Receipt；有权 human actor 或冻结 reducer 再提交固定 ChangeSet Revision、target、expected head 与 Gate evidence 的 integration intent。
8. control 先持久化 intent/outbox，`hctl2-tool`（本地）或 adapter（远端）执行并 readback；只有确认目标事实后才写唯一 Integration Receipt，结果未知时不得签成功或盲重投。
9. task-bound Run 按合同正常完成后，Run reducer 以稳定幂等键提交同一个「完成 Task」命令；Task 独立准入并校验精确 Integration Receipt，失败类 Run 不终结 Task。
10. 任意步骤崩溃后通过 generation、outbox 和 readback 恢复，不重复外部效果。

## Kanban content 后端切片

为 Repo 选定 content 后端 → 映射 Project 分组与稳定实体 → 导入 Snapshot →（按需）升格采纳为 Task Revision → 按字段权威写回 → 回读确认。两类后端各验一条：本地任务服务器（默认）与远端 GitHub/Linear 直访。支持显式 refresh 与定期 reconcile，不依赖公网 webhook。创建结果未知、限流、外部修改、tombstone、重新绑定和无 Workbench 原生操作都必须有测试；后端关闭态或拖卡永远不直接产生 HCTL 完成；无契约的卡不进治理，惰性创建契约的升格路径必须有测试。

## 自举阶段

HCTL2 不会等到第一阶段完整交付才用来开发自己。自举按能力分级，而不是“上线前/上线后”二分（施工顺序见「实现阶段」：B0–B5 全部发生在 P2 内部，B6 对应 P3 末）；每一级都走普通的命令与查询入口，并包含真实的失败路径。打开过自己的仓库，或替自己生成过一次代码，都不算完成自举。

| 阶段 | 事实切换 | 晋级验收 |
| --- | --- | --- |
| B0 | ID、SQLite、command/query/event、进程和恢复底座 | 干净 clone 可启动；重启不丢状态；脚本只管进程和恢复 |
| B1 | Project Room 与本地 Task 影子试用 | Room/Task/草稿重启可恢复；引用稳定；明确不切换事实 |
| B2 | 无 Run 切片成为真实开发入口 | 从 Project Room 在隔离 worktree 与有效写租约下完成一次真实的非文档代码改动和测试；Harness 环境中取不到 HCTL 交付的集成/外部写凭据；声明了执行加固的 Profile 按声明生效并留记录，宿主施加不了则不启动。第一次真正自举 |
| B3 | 接管自身待办、并发 Invocation、Request、Receipt 和冷启动恢复 | 连续至少 5 个真实变更，覆盖核心/界面/适配器与故障重启，全程无手工改库、无人肉转发 Prompt |
| B4 | 引入 Workflow Engine、Run、Seat 和独立 Gate | 一个真实变更走完“驳回 → 返工 → 重新评审 → 合并”，期间重启任一组件；无手工推进引擎或绕过 Receipt |
| B5 | 候选切换、三选二、regate 和完整故障恢复；第一阶段目标 | 完整治理切片在 HCTL 自身的真实变更上通过，而不只是测试样例 |
| B6 | 稳定版本 N 构建、验证、升级和回滚隔离环境中的 N+1 | N 驱动 N+1 的构建、测试、打包、升级和回滚；被测进程不覆盖治理它的 control 与数据库 |

旧工具在事实切换前可以作为执行者或逃生通道，不能继续保有平行 Project/Task/Run 账本。降级超过约定能力时回退到上一自举级别并留下审计记录。

B5 是第一阶段功能成熟度目标；正式发布、升级与回滚仍必须通过 B6，不能把“已能自举”当成可分发版本。

自举验收不得对 HCTL2 仓库、内置账号或测试环境设置隐藏的特例豁免：开发自身必须只使用公开的 Query/Preview/Submit/Subscribe、CLI 和受控端口，实际 Context、权限与证据均可检查；手工推进 Engine、直接改库、隐藏 Prompt/Context 或在产品外补签 Receipt 都不算通过。

## 契约测试矩阵

交付测试检查可观察行为，不复述模块状态机。每族一个稳定 family ID；模块新增合同必须在对应族里增加一个失败用例，而不是再建一份不变量文档。

### `CT-PROJECT` · Project / Chat Room

- 首次注册生成稳定 Repo 身份，同一 Repo 的新 clone 只新增 Repo Instance，fork/身份碰撞明确拒绝或要求确认
- Project 分组与 Room anchor 可重建
- 有活动 Invocation/Run 或开放 Request 时归档 Project 拒绝
- CJK 输入、结构化引用、草稿/游标/未读、并发流隔离；时间线顺序以 chat server 给出的为准，治理引用只按事件 ID 冻结
- Repo Room 只把显式选中的来源链带入新 Project
- Scoped Room 回填和同根因 Request 去重
- Context 可解释、Room 历史可恢复（chat server 重同步 + 治理引用与冻结 digest 完整）
- chat server 不可用时，依赖 fresh Room 来源、身份或 Context 的预览/命令 fail closed，不依赖这些读数的已接纳治理事实仍可使用
- Chat 端口绑定只接受未启用端到端加密的房间，HCTL 自建房间回读无 `m.room.encryption`；已绑定房间事后被加密与 chat server 不可用走同一条 fail-closed 规则并标为需要关注，换绑到未加密房间后恢复
- chat server 中的消息、反应或自动化不能成为命令
- 模型 Participant 的 `@`/建议不能创建 Invocation 或 fan-out，human 批准后自动携带来源/Context
- 无法证明身份的 Invocation 撤权并终止，Retry 产生新调用且旧结果被拒绝
- mention 解析无唯一授权候选时明确失败，不按显示名模糊匹配或静默换人
- 原始消息、执行日志和模型总结不经「发布 Memo」命令不会成为 Memo
- 治理引用指向滚动纪要而非精确消息事件时拒绝
- Bundle 压缩条目缺 compressor/原文 digest 记录，或压缩了证据类内容时拒绝交付
- 萃取索引与纪要缓存删除后可完整重建且不丢事实；相关性门判定缺可审计记录时无效
- 过期或被取代的 Memo 不进指针清单，显式引用除外

### `CT-TASK` · Task / Kanban

- Task Revision、lifecycle、stage、正交 health、lane 投影与外部状态分离
- 非法 move/complete 拒绝
- local state version 与 remote revision 不混用，过期邻项移动重算
- 本地 adoption 不要求伪造 Task Binding，外部 adoption 混用 Task Binding 版本时拒绝
- 未采纳契约使 Start/Complete fail-closed，明确 divergence 后新增 drift 仍使旧预览失效
- active Run 未收口时 terminal intent 拒绝
- human Kanban 完成与正常 Run reducer handoff 都走同一「完成 Task」命令；Result Proposal 通道与外部关闭态提交不了它
- 同一规范实体跨 Project/connection/placement 不得产生第二个 Task，禁用 binding 也不释放映射
- content 后端按 Repo 选定，跨后端相对移动拒绝
- Project 分组 anchor 在删除、重绑和重建后保持稳定，原生 UI 把卡跨 Project 分组移动只形成 jurisdictional drift，不静默改写 `project_id`
- Project 分组映射（父任务/milestone/标签降级）有测试
- 无契约 Task 的看板终态只是 content 投影，完成命令仍需先升格契约

### `CT-RUN` · Run / Workflow

- 编译/Profile 拒绝、0..1 Task 绑定、Engine mutation 只有 control
- 超时与候选切换只依据账本自己的 Obligation deadline
- dispatch ACK 丢失允许待启动→丢失并用新 Attempt 恢复，已交提案不被误当成功
- retry 只产生一个新 Obligation 并隔离旧 Seat/Attempt，候选耗尽和 Request expiry 类型化收口，所有 Run 过渡态可失败/替代
- dynamic fork 超出冻结 Seat 模板/recipient/基数/预算时拒绝
- placement 变更留下不可变审计
- Gate backup 改变参与者或任一 Context/Skill/policy ref 时拒绝，作者不能占必需 reviewer Seat
- quorum-unreachable 沿冻结失败边推进
- Run 正常完成只由账本谓词决定；引擎路标与账本不一致时标分歧待对账，既不补足也不阻止谓词
- 失败/已取消/被替代 Run 不终结 Task，quorum/regate 和迟到结果拒绝

### `CT-AGENT` · Agent / Terminal

- 能力探测、ChangeSet 单 writer、精确 Revision/digest、runtime generation
- 无法证明旧 writer 已 fence 时隔离旧 worktree 且不得重授租约，失败清理不丢唯一未封存/未跟踪修改
- 本地/远端 SCM 集成都先持久 integration intent，由 tool/adapter 执行并 readback，target-head 竞争或 ACK 未知时不得签成功 Integration Receipt
- 冲突观测按来源证据仲裁
- Execution Chat 的错误 owner/generation 输入和无 provenance Share 均拒绝
- 治理命令只有两类入口：认证的场景客户端会话（human：Workbench、CLI 或适配的第三方客户端）与 task-bound Run 正常完成的 reducer（system）；Result Proposal 通道提交不了治理命令
- 每个 Worker Profile：Harness 环境与进程取不到 HCTL 交付的 control/人类 credential 与集成/外部写凭据，凭据只由工具箱/adapter 网关代用；Harness 在 worktree 内可读 common-dir/refs 并在本 ChangeSet 分支提交，绕过「合入 ChangeSet」命令改写目标 ref 不产生 Integration Receipt，下一次 integration preview 因 expected target head 不匹配显示 drift
- 声明了执行加固的 Worker Profile：所声明项按声明生效并与 Execution Runtime 记录一致；已声明而宿主不支持时不激活，拒绝结果列出缺项；未声明时照常启动、不记录为已生效
- 人在 HCTL 外直接改 provider 只形成 drift，不能冒充结果
- provider 原生客户端输入缺带外入账记录，或被赋予输入租约语义时无效
- 未声明观察扇出能力的 provider 直连观察者拒绝；已声明的 provider 缺缺口披露时该通道降级为带外诊断
- provider 状态检测以低层来源覆盖仍有效的结构化 hook 证据时拒绝；provider 恢复报告无法翻译为四级恢复词汇时按丢失处理
- control 签发 descriptor、agentd 终端网关校验，观察、输入、Attempt 控制与安全输入权限分离
- attach 只接通道，不能恢复 Run/Invocation 语义
- attach/replay、IME/背压/慢客户端隔离

### `CT-CONNECTION` · 连接 / 端口

- 每条 handoff 固定 source ref/digest 与唯一 binding
- Run/Invocation 冻结精确 Context Manifest ref+digest，每个实际 consumer 冻结对应 Context Bundle ref+digest，权限过滤、来源版本或预算变化使旧预览失效
- client/port 权限分离
- actor provenance 不能由 payload 自报
- dispatch/result 迟到拒绝
- 外部 effect ACK 未知不重复且 adapter 不写 Receipt
- provider 离线时，不要求 fresh readback 的查询/命令可继续，要求 current head/revision/lease/readback 的准入统一 fail closed
- Harness 绕过受控端口的 API 写能力被拒绝，带外 drift 只形成 Snapshot/观测而不是结果

### `CT-SYSTEM` · 系统

- 同一用户级账本只能有一个 control writer，第二 writer 拒绝
- 多个 agentd 可以登记为不同 execution site，但同一 site/repo mutation lease 的旧 generation 必须被 fence，无法证明 fence 时不得重授写权限
- 命令幂等
- commit/ACK 各崩溃点回读
- schema migration、投影重建
- metadata 账本执行一致性 backup、restore preview/apply、writer generation 重置与恢复后 content readback，每个首次消费的 content 服务器执行备份与恢复
- content 服务器宕机不抹掉已接纳事实，但依赖 fresh provider readback 的命令 fail closed
- 从 Git 结晶回灌不得伪造未结晶判决
- clone 本地运行目录（锁与缓存）删除后可完整对账重建、不丢事实
- 一键启停下已消费服务器的启动顺序与健康检查
- 旧 generation 与越权适配器拒绝
- 等价对象的 JCS 规范摘要一致、内容篡改被 digest 校验拒绝
- 打包后的整窗启动/退出/升级和安全边界

### `CT-PACKAGING` · 扩展 / 打包

- 自声明 trust、有副作用的 discovery、静默 install/upgrade、非本地未认证 Dagu、renderer Node/raw IPC/远程脚本或不满足下述源码合规门禁时均拒绝

### `CT-WORKBENCH-IA` · Workbench 信息架构

- 单 Project Overview 与全局「需要关注」都是可重建的只读导航投影，不产生第五场景或写状态
- 打开入口按 repo 选择并统一映射到控制面连接（打开本地 repo = 连接或拉起本机控制面再定位仓库；第一阶段远程入口隐藏或安全拒绝）
- 进入 Project 默认打开 Project Room，deep link 保留返回路径
- 同一 Request ID 跨 Room/Task/Run 聚合且不能从聚合面直接改状态
- 「创建 Project」命令提升预览允许删减、补充、去敏并显示来源回链
- Trigger Preview 展示实际执行者、Context/Skill、权限、预算和 fan-out

### `CT-WORKBENCH-INPUT` · Workbench 输入与无障碍

- Board 移动、Request 操作和 Run 浏览在 mouse/touch/keyboard/screen reader 下等价
- 输入优先级为 IME composition → 已聚焦 terminal → modal/composer → 当前场景 → 全局快捷键，任何上层快捷键都不能截获正在组合或发往 terminal 的输入

### `CT-PRODUCT` · 产品

- 用户十秒内能回答 Project 目标、Task 状态、Run 阻塞、所需动作、当前 Harness 和证据版本
- 正常成功保持安静
- HCTL2 仓库自举不使用隐藏的特例豁免或产品外补签事实

## 选型判据

执行面各系统的实现选型按以下准入标准评估；实现名只出现在本文与[实现证据](../research/README.md)，设计层正文只用系统角色名：

1. **接口公开干净**：优先公开协议与文档化 API，保证第三方 UI 与场景客户端互操作——chat 场景直接采用 Matrix 协议；任务场景没有同级的开放协议，选择 API 完整、支持条件写入的实现并以受控端口隔离。
2. **运维简单**：本机执行模式优先单二进制、内嵌存储、低资源占用的后端。
3. **生命周期可托管**：随 HCTL 一键启停（由 control 编排），支持备份、恢复与固定版本升级。
4. **选型三件套**：每个新增系统都设限时、可丢弃的开工前验证，且只验 HCTL 与该系统的接缝（第 2、3 条是选型时的资料判断，在首次消费时产品化，不进 P0）；验证失败就重开并修订对应选型决定及 [decision-history](./references/decision-history.md)，不另造 ADR catalog；不自研第二个同类系统。

## 开工前限时验证

P0 的内容就是本节。各项选型已拍板，验证因此从“选谁”变为“关键假设能否落地”；关键假设只指 HCTL 与该系统的**接缝**——我们的适配器、受控端口或 agentd 实际调用的那几个 API 与行为。第三方自身的功能（它自己的备份恢复、重启、渲染、内存配置、发布物形态）不在 P0：要么是选型时的资料判断，要么在首次消费时产品化。每项探针使用可删除的数据、脚本和拼装环境，只产出实现证据、固定版本与产品化约束；通过不代表已经具备 HCTL 一键生命周期、备份恢复或升级。真正的托管由 control 出现后在对应场景首次被消费前完成。各依赖的 P0 探针也不是全局 barrier：chat 与 task 探针在 B1 首次消费前完成，运行时探针在 B2 前完成，workflow engine 探针只须在 B4 前完成，不能阻塞 B2；失败就重开并修订对应选型决定与 decision-history。

1. **workflow engine（Dagu，已拍板）**：固定基线为 [`v2.15.1 / 532c5129`](https://github.com/dagucloud/dagu/tree/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a)。采用单进程 `start-all`、文件系统持久化和声明式 YAML；Workflow Revision 仍以 HCTL 规范化 JSON 为事实源，由固定编译器生成 Dagu DAG。生成物只用依赖/条件/等待等机械结构与无进程的 `human.task` 作为 HCTL 外部执行检查点，不允许 Dagu 自行运行 command/script/action/HTTP/Harness。P0 只验接缝：DAG 提交与启动/暂停/恢复/取消 API 的应答与状态回读，`human.task` 检查点的等待态观察、完成与回读，以及路标被 Engine 自行推进或重试时能否回读为分歧。代次不在 Dagu：Obligation 的身份与隔离由 HCTL 账本承担，Dagu 只当路标，"完成 API 的代次隔离"不再是 B4 阻断项。
2. **运行时后端（tmux，已拍板）**：源码审阅基线为 [`3.7c / e476c123`](https://github.com/tmux/tmux/tree/e476c1230b958df0cb12977517d24b3dc931375b)。agentd 为每个 runtime 建 owner-only socket/server，以 control mode 持有唯一可写客户端，并持久化 session/window/pane ID 与 generation；Workbench/CLI 观察者只消费 agentd 的转发，不直连 tmux。P0 只验 agentd 依赖的 control mode 接缝够不够用：能建 owner-only socket 并持有唯一可写 control client、观察者能经 agentd 扇出、输入与 resize 能下发、session/pane ID 稳定可持久化、退出码与残留进程能回读、无人 attach 时能查询应答。键盘协议子集、慢观察者背压、各 Harness 的 TUI 兼容与 [`#5510`](https://github.com/tmux/tmux/issues/5510) 之类是 tmux 自己的轮子：选型时作资料判断，使用中不够用就按选型三件套升 commit 或换家，不为它们维护验证矩阵。2026-08-29，三个实际分发目标的上述接缝 [P0 均已通过](../research/tmux-runtime.md#2026-08-29-p0-接缝验证)；完整取舍、实测 footprint 与 shpool/Zellij 对照见[实现证据](../research/tmux-runtime.md#e-l1-tmux-runtime)。
3. **chat server（Tuwunel，已拍板；Continuwuity 为记录在案的备选）**：Rust 单二进制、采用 RocksDB 系嵌入式存储的 Matrix homeserver。固定基线为 [`v1.9.0 / 5b366914`](https://github.com/matrix-construct/tuwunel/tree/5b3669144219d5d4c0774743c84191b476f1b54f)。拍板理由：接口更 API 化、与 Synapse 参考实现兼容性更强；AppService 注册程序化，不靠房间内发命令。P0 只验 Chat 端口依赖的接缝够不够用：账号与房间管理 API、AppService 注册与事件投递、按事件 ID 读取正文、房间加密状态可回读（自建房间不带 `m.room.encryption`、绑定前能读到该状态、事后开启能被观测）。事务 ID 幂等、事件顺序、重同步是 homeserver 自己的合同，HCTL 拿来用、不替它验。Chatroom 发行形态同时固定 Cinny `v4.12.6` 官方 Web 发行包为随包互操作/查看客户端；它不是 Workbench，也不拥有治理权威。Tuwunel 官方发布物虽只有 Linux，锁定源码已在 Apple Silicon 原生构建；含 Cinny 的 Linux 包已通过完整生命周期，两个 macOS target 仍须分别原生重建验证。低内存配置与 RocksDB/media 一致性备份不在 P0，连同由 control 托管的一键启停和恢复演练一起，到 B1 首次消费前产品化。
4. **task server（Vikunja，已拍板）**：固定基线为 [`v2.5.0 / ef2200e9`](https://github.com/go-vikunja/vikunja/tree/ef2200e9429c5cc42f5c1811433418bfcc72b3aa)，Go 单二进制、SQLite、REST API + webhooks，并有官方 macOS/Linux 发布物。探针只验 Task 端口依赖的接缝够不够用：卡片与分组的读写 API、按分组稳定回读归属、条件写入是否可用、webhook 或轮询能否观测变化、实体 ID 稳定；排序算法与看板语义是它自己的轮子；备份恢复不在 P0，连同由 control 托管的一键启停和恢复演练一起，到 B1 首次消费前产品化。git-bug（零服务器、任务存于 git refs）降为记录在案的对照——仅在验证失败、重开并修订 task server 选型决定与 decision-history 时再取，且须显式接受“任务 content 也在 Git”的模型例外并记入决策历史。
5. **远端任务后端（移出 P0）**：Linear/GitHub 的身份、字段权威、outbox/readback、限流和 tombstone 验证延至 P2 的日常自举子阶梯之后按需启动——合同未押注它，双向适配是五项中最贵的一项。
6. **运行时 provider 第二实现（herdr，限时验证）**：运行时后端在合同层收口为可插拔的运行时 provider（合同见 [Agent 合同](./spec/agent.md#运行时与观测)），tmux 是内置最简实现、选型不变。herdr 固定基线 [`v0.8.2`](https://github.com/herdrdev/herdr/releases/tag/v0.8.2)（Apache-2.0，官方单静态二进制 + SHA-256/attestation，免编译）作为第二实现候选走限时验证，探针只验 provider 合同的接缝：headless server 与 socket API 的版本协商回读、pane 创建/输入/读取/wait-output、多观察者与单写者接管的能力声明回读、跨重启会话保持到四级恢复词汇的翻译、状态检测事件的来源与置信度标注、原生客户端（TUI/`--remote`）输入可被带外入账观测。2026-08-29 的容器探针已给出 footprint 与 API 形态证据（见[实现证据](../research/herdr.md)：重输出下 RSS 约为 tmux 同负载的 5 倍）。通过则 herdr 作为可选 provider 进入交付面，按打包策略补许可证与摘要锁定；失败则维持 tmux 单 provider 并修订本决定与 decision-history。

## 打包策略（选型判断，首次消费时产品化）

分界线是**碰不碰宿主机现场**：

- **必须原生**：tmux、harness、`hctl2-agentd`、`hctl2-control`、`hctl2-tool` 与 CLI——要碰真实 worktree、PTY 与 OS 密钥串，不进容器；macOS/Linux 原生分发。tmux 直接消费摘要锁定的官方 `tmux-builds` 单二进制：Linux 制品静态链接，macOS 制品只链接系统 dylib；随包保留上游许可证集合，不再维护自主 C 构建链。
- **服务器按服务声明形态**：control 出现后，生命周期托管器在服务首次被消费前声明原生发行 target。Linux x86_64、macOS arm64 与 macOS x86_64 分别构建；macOS 最低基线为 15。Dagu、Vikunja、tmux 使用官方原生发布物，Tuwunel 因上游没有 Darwin 二进制而从锁定源码原生构建；各 target 共用锁定的 Cinny 官方 Web 发行包。不同 OS/CPU 不混用缓存、动态库闭包或生命周期验证。
- **Docker 不做统一打包方式，也不做 Harness 的沙箱或桌面形态**：执行面一半天生进不了容器；macOS/Windows 上容器即 Linux 虚拟机，有授权与资源开销问题。第一阶段的 Linux/macOS 发行均为原生包，最终用户无需安装 Docker Desktop；执行加固只按宿主 OS 原生机制施加。
- Windows 不在第一阶段范围；tmux 没有原生 Windows 后端，未来 Windows 版本须在同一运行时合同下另选实现并重新过兼容矩阵，当前选型不宣称跨平台。Dagu/Vikunja 有 Windows 发布物，Tuwunel 未见官方包。

## 技术基线

Rust control/tool/agentd；Electron + React 19 Workbench；SQLite + FTS5 与 Git；Tiptap、React Aria、React Flow + Dagre、xterm.js。执行面服务器经受控端口接入、由 control 托管一键启停：Dagu（workflow engine）、Matrix homeserver（Tuwunel；Continuwuity 备选）、本地任务服务器（Vikunja）、运行时 provider（tmux 内置；herdr 限时验证为第二实现候选）；Chatroom 另随包提供 Cinny 内容客户端。精确版本、实测 footprint 与运维分级见[实现证据](../research/README.md#执行面已选依赖的运维与-footprint)，Workbench 的 Electron/Tauri 2 取舍、竞品产物抽样与重开门槛见[桌面壳证据](../research/workbench-shell.md#e-workbench-shell)。选择受契约测试约束，不能为了保留依赖而削弱模块边界。

任何采用、移植或 vendor 的外部源码都必须固定已审阅 commit，核验目标文件及依赖许可证，保留 license/copyright/attribution 与修改记录，并用 HCTL contract tests 隔离上游漂移；任一项缺失即不得进入分发产物。

## 未决问题

- ~~Room 的协作历史到底放哪里~~ 已了结：消息 content 归 chat server，metadata 随用户级控制面走，结晶进 Git（见[来时路 §12](./references/decision-history.md)）；
- ChangeSet/PR 默认基数与后续多 Task Run 的集成策略；
- ~~Repo Room 跨 clone 迁移~~ 已了结：Room 身份随用户级控制面、消息 content 在 chat server，与 clone 无关（见[三面架构](./architecture.md)）；仍开放的是 Repo Room 的隐私与保留期限——端到端加密不是答案（HCTL 房间对 control 明文可读），只能由 homeserver 侧访问控制、传输/存储加密与保留策略回答；
- Project 拆分/合并和 Task 依赖的产品表达；
- Scoped Room 自动归档策略；
- 首批原生会话导入的范围与长期维护预算（能力定义见 [Agent 设计正文](./agent.md#原生会话导入)）；
- 多主机与远程的实施：架构方向已定（Workbench 连接本机或远程控制面，见[三面架构](./architecture.md#三个面)），未决的是远程连接的认证与传输、多主机执行现场的编排、Windows 与多用户权限；
- 成本/预算硬上限及运行中耗尽的交互；
- ~~第一阶段之后首个非 Matrix 聊天平台桥接~~ 已了结：HCTL 永不自建聊天桥接——非 Matrix 平台经 homeserver 侧 Matrix 桥接生态接入（content 层），HCTL 只保留桥接用户的身份映射策略（见 [Project 设计正文](./project.md#chat-room-场景)与来时路）。

# 系统边界与适配器合同

> 状态：规范性合同 · 草案 v0.12.3<br>
> 本文只定义四个模块共享的运行机制，不拥有 Project、Task、Run 或 Agent 的领域状态。

## 组件

| 组件 | 职责 |
| --- | --- |
| `hctl2-workbench` | 四个场景的集成客户端；提交命令、查询投影、显示事件 |
| `hctl2-control` | 唯一领域 command service、路由、权限、账本、outbox 和对账 |
| `hctl2-tool` | 机械工具箱：Git/SCM 操作与事实校验、Revision/digest、合并核验、lint、PR/memo 的机械拼装、有效变化侦测。独立可用（standalone 时只做普通本地操作、不产生治理宣称）；被 control 编排时执行已持久化的意图并回读 |
| `hctl2-agentd` | Harness 发现、物理运行时、PTY、终端网关、按声明施加的执行加固与主机观测（散文简称 agentd） |
| Workflow Engine | 通过适配器保存 Run 的机械 token、task、timer、retry 和历史 |
| chat server | 经 Chat 端口访问的聊天服务器（Matrix 协议）；承载 Room 消息 content 的 ground truth |
| task backend | 经任务源端口访问的任务后端（本地任务服务器或远端平台，按 Repo 选择）；承载任务卡 content 的 ground truth |
| 第三方场景平台 | 提供部分场景客户端、受控端口或两者；两种 binding 与权威分离 |

Workbench 不是特殊内核。Workbench、CLI 和外部 UI 是场景客户端，使用 Query/Preview/Submit/Subscribe；外部 Chat、任务源、workflow engine、harness 和运行时后端是由内核调用的五类受控端口。同一产品可以同时提供客户端与端口，但两者的 binding、权限和事实权威必须分开。agentd 是组件实现名（Agent 模块的本机执行守护进程），不是 Agent 模块本身。

## 固定内核与受控端口

固定内核是一个用户级 command service；它对每个 Repo 保持独立的语义范围，而不是在每个 clone 各起一套控制面。固定内核实现四模块定义的稳定身份、Revision 准入、权限、字段权威、领域归约与 Receipt；本文件只拥有共享 command envelope、扩展绑定、outbox/inbox、单写者和恢复机制。

即使把全部界面、聊天平台、Task 来源、工作流引擎和终端客户端都换掉，内核所守的身份、权限、版本证据、治理与恢复边界也必须原样保留——这是[愿景文档](../vision.md#产品原生核心与架构最小内核)中“产品原生核心与架构最小内核”在系统层的落点。

可以替换的端口包括：

| 场景 | 端口 |
| --- | --- |
| Chat Room | chat server 连接：消息/附件读写、账号与房间管理、身份映射（非 Matrix 平台经 homeserver 侧 Matrix 桥接接入，不是 HCTL 端口） |
| Kanban | 任务源端口读取、字段写回与快照 |
| Workflow | Workflow Engine 编译、注册、执行和回读 |
| Terminal | harness、运行时后端、终端网关与 attach provider |

第一阶段全部执行面 content 服务器（chat server、本地任务服务器、Workflow Engine）的管理/API 端点只绑定 loopback 或 owner-restricted local socket。未来非本地 transport 必须认证客户端；治理变更仍只能由 control 经对应受控端口发起，不能把服务器 mutation 暴露给场景客户端或其他本地进程。chat server 与任务后端的 content 读写不在此限——那是场景内容，不是治理。Terminal 的写输入例外不成立：受治理的 attach/input 必须经 agentd 网关校验精确票据和当前代次，直连运行时后端只能作为明确标注的带外诊断，不得写回 HCTL 结果。

hctl2-control 托管执行面服务器的生命周期：随 HCTL 一键启停，启动顺序、健康检查、备份与升级由 control 统一编排。托管不授予 content 之外的任何权威；服务器进程的死活只影响对应场景的可用性，不改变治理事实。

每个扩展绑定都冻结代码版本、接口/schema、配置摘要、依赖、能力和信任级别。运行中不得因“发现更好的插件”而响应式改绑；提供方消失时安全暂停、失败或创建替代执行。

`trust_level` 只能由 control policy 根据允许的 trusted source 与精确 artifact digest 授予，扩展或 registry 的自我声明不能授信。discovery 只读取已配置 definition 和本地安装并执行无副作用探测，不得联网、安装/升级或修改 Harness/adapter 配置；install/upgrade 必须是用户显式提交的类型化动作，产生新 Extension Revision，后续解析再产生新 Resolved Port Binding，不能改写活动绑定。

跨模块可引用的扩展信息只有两个最小概念：

- `Extension Revision`：扩展稳定身份的一份不可变版本，固定代码/制品摘要、接口与 schema、依赖、声明能力和信任级别；
- Resolved Port Binding：一次已解析端口选择，固定 binding_revision_id、Extension Revision、provider/installation、配置摘要、credential reference、实测能力、权限作用域和降级策略。

同一 `(port_kind, scope_id)` 的一次准入只能解析出一个 binding revision；提供方加载顺序、hook 优先级或 UI 选择顺序不得决定事实。Room 的 Chat 端口绑定、Task Binding、Engine Deployment、Engine Execution Binding 和 Execution Spec（含其接入方式字段组）都引用精确 Resolved Port Binding；历史执行继续使用原 binding。credential reference 只定位 secret store 条目，不包含密钥。

当前 health、重同步 cursor、成员现状与类似运行数据是带来源、版本/序号与观测时间的可变投影，不进入不可变 binding digest。它们可以阻止新准入或驱动对账，但不会改写历史 binding；只有 endpoint、配置、能力、信任、权限或降级合同变化才产生新 binding revision。

跨 Project 使用的 Skill 是带稳定 ID、revision 和 digest 的共享定义，至少固定 manifest/instructions/assets/scripts、来源/license、兼容能力与依赖；更新创建新 revision，current pointer 只用于选择，Execution Spec 与 Run Manifest 必须冻结精确 ref+digest。Skill 可以提供方法并请求能力，不能授予权限、票权、委派或 Task 完成权。

进程内扩展等同受信任代码。普通独立进程只隔离崩溃；不可信扩展需要操作系统强制隔离和能力削减的代理接口。

## 场景端口

所有场景客户端共享四类操作：

```text
Query(filters, cursor) -> Projection
Preview(command draft) -> Effect summary + preconditions
Submit(typed command) -> accepted result or typed rejection
Subscribe(cursor) -> ordered events or resync snapshot
```

场景客户端只声明交互能力与降级行为；受控端口报告 provider 支持的读写能力，实际字段权威只能由对应模块的 authority binding 授予。外部平台可以拥有其场景 content 的 ground truth，以及明确授权的字段；但它不拥有治理——其数据库、thread、Issue、workflow task、Session 或 pane 不成为 HCTL 的身份、授权或判决来源。

渲染器、拖放、按钮和终端输入都只是 command client。未能提供等价预览、版本或权限信息时，动作必须禁用或安全暂停。

## 命令与跨服务正确性

改变事实的命令至少携带：

```text
command_id / idempotency_key
authenticated actor principal + actor kind/provenance + permission scope
target stable ID
expected revision or state version
frozen adapter binding
canonical input digest
```

actor kind/provenance 由认证场景入口或 control 内部 reducer 赋予，调用 payload、Room 消息、Harness 进程和 adapter 都不能自报为 human 或 workflow reducer。execution principal 只获得 Invocation/Attempt 冻结的窄能力。Task 完成只接受有权 human actor 或 task-bound Workflow 的正常完成 reducer，Task 已取消只接受有权 human actor；普通 Room 临场 fan-out 只接受有权 human actor，Workflow reducer 只能实例化 Workflow Revision 已冻结的边。

human provenance 由经认证的场景客户端会话直接赋予：用户在 Workbench 或 CLI 里提交的命令就是 human。agentd 启动受治理执行时交付执行凭据，凭该凭据发出的 CLI 调用以 execution principal 提交，只获得该 Invocation/Attempt 冻结的窄能力。Task 终结、普通 Room 执行边、扩权、安全输入与集成授权只接受这样赋予的 human provenance 或获准的 reducer，不设额外的用户在场证明。Workbench 与 CLI 使用同一验证规则，这是 actor 认证而非界面隐藏特权。

control 在用户级 metadata 账本的一个 SQLite 事务中写领域事件、幂等结果和 outbox；跨模块命令也只能使用这一个事务边界，不能由两个模块或两个 clone 事后拼接。外部适配器按同一 key 投递并回读；超时或 ACK 丢失保持“结果未知”，不能盲目重做。重复命令返回原结果，异载荷复用同一 key 被拒绝。

组件或 content 系统不可用不得降低命令前置：与该系统无关、且所需引用已冻结的 metadata 命令可继续；验收策略、字段权威或冲突前置要求 fresh readback 时，不可用或 cursor gap 必须返回类型化拒绝。只有冻结策略明确允许 cached/stale 证据，命令才可携带其精确版本、观测时间和已知 gap 继续。

Receipt 证明的是已经校验的结果，不是另一个 writer。投影可以从事件重建，缓存或界面状态不能反向成为事实。

跨模块引用的规范摘要统一使用 RFC 8785 JCS 规范对象的 SHA-256，摘要字段自身不参与计算；每个领域 owner 只定义自己规范对象包含哪些字段。完整 Revision 的 revision_digest 与为评审选取字段生成的 review_subject_digest 是不同语义，即使某次字节恰好相同也不能互换。

## 外部权威副作用

包括远端 SCM 在内、会改变第三方权威事实的动作统一写成持久外部副作用命令/outbox 记录（executor = adapter），固定 owner ref、Resolved Port Binding、operation、target、adapter 声明的 conflict scope、权限、规范输入摘要和幂等键。`conflict_scope` 表示同一远端资源的互斥域，不能仅因 close/reopen/update 等 operation 不同而拆开。本地 Git 变更是同族外部副作用命令（executor = tool）：先由 control 持久化 intent/outbox，再由工具箱执行和回读；Harness/model 不直接取得集成权。

adapter 只投递并回读；只有在它确认目标、版本和结果后，control 与工具箱的校验事务才能写成功 Receipt。投递超时或 ACK 丢失保持结果未知，并占用 conflict scope，阻止同一资源上的重叠写。Harness 第一阶段以窄 execution principal 运行：HCTL 不向它交付 control 客户端凭据、OS secret store、SSH agent 或未授权 provider 配置中的密钥；集成与外部写凭据只在校验当前 control writer 及适用 site/backend generation 的工具箱/adapter 网关内使用，不注入 Harness 环境。Harness 可读所属 Repo Instance 的 Git common-dir 与 refs 并在本 ChangeSet 分支提交；对目标 ref 或其他 ChangeSet 现场的直接写入不取得集成权，只回读为 drift。OS 沙箱、网络白名单等执行加固由 Worker Profile 声明、Execution Spec 冻结并记录为运行时事实，不是受治理执行的启动前置。

第一阶段不承诺自动发现或补偿任意带外写：Harness 不获得可绕过受控端口的外部写凭据；provider 被人在 HCTL 外修改，或本地目标 ref 被 Harness/人在「合入 ChangeSet」命令之外直接改写时，只由对应端口或工具箱回读为 Snapshot/drift（后者表现为 expected target head 不匹配），并阻止依赖旧版本的命令，直到用户通过该模块既有的采纳或对账动作处理。带外观测不能成为 Result Proposal、Artifact、Verdict 或 Receipt。

## 事实与存储

### Repo 与执行现场

Repo 是四模块可归属的逻辑仓库，由 [Project 模块](./project.md#repo-注册与-project-归档)注册；Repo Instance 是本系统拥有的物理执行现场，不属于任何 Project。一个 Repo 可以显式挂接多个 Repo Instance，每个现场固定稳定 `repo_instance_id`、精确 `repo_id`、host/site identity、Git common-dir identity 与首次校验的 Git 证据；worktree、ChangeSet 物化、工具箱锁与本机 runtime 都通过该现场引用。

「挂接 Repo Instance」命令先由工具箱无副作用读取 Git identity，再由 control 预览并写入账本。相同 Git common-dir 重试返回原现场；不同现场只有在 Git 中的稳定 Repo identity 与命令指定的 `repo_id` 一致时才能挂接。remote URL、目录名或碰巧相同的 HEAD 都不能单独证明身份；缺失 identity、fork 语义不明、一个 common-dir 已归属另一 Repo 或证据相互冲突时必须类型化拒绝并要求用户选择注册新 Repo 或修复来源。移除现场只撤销其新执行资格，不删除 Repo、Project、历史 Run 或已封存 ChangeSet。

### 控制面自己的存储

hctl2-control 的存储只有一本库：**用户级 metadata 账本**。它是全部 metadata（稳定身份、Revision 准入与 current、绑定、授权、租约、代次、现场记账、Run Manifest、Execution Spec、Result Proposal 准入、Verdict/Receipt）的唯一权威，一人多机连同一本，必须备份。仓库 clone 本地的 `<git-common-dir>/hctl2/`（当前 Repo Instance 及其 linked worktree 的共享运行目录）只有 OS 锁、traces 与可丢弃缓存——**不是账本，也不是事实源**：现场状态永远可以从 metadata 账本、Git 与运行时观测对账重建，删除该目录不丢失任何事实（无法证明身份的旧执行按丢失收口）。

control 也会把结果写到自己的库以外，但那些是外部副作用的目标，不是另一份 metadata 账本：获准的不可变正文与判决审计影子经工具箱写入 Git（见下节）；获准的记录可以写回 content 系统（记录不是命令）。

账本保存 HCTL 自己的领域关系、授权与判决，以及 HCTL 身份到外部 content/runtime 身份的跨系统锚定；它不复制承载系统内部的完整拓扑（如任务后端里与 HCTL 无关的卡片层级）。控制面凭获准命令、精确映射与 Snapshot 对账需要治理的那部分外部关系。账本与其余本地存储（锁、缓存、定义文件）的物理布局是控制面的**私事**：事实经服务接口流通，路径和表结构不构成对外 API，也不进 Git；但“唯一用户级账本、权威归属和备份传承”仍是架构合同，不能被实现选择改变。

存储拓扑固定为：

```text
~/.hctl2/                      # 用户级配置、Harness/Profile/Skill/Runtime 定义
                               # control.sqlite、control.lock —— 用户级 metadata 账本与写锁
<repo>/.hctl2/                # Git tracked · repo.toml、projects/、workflows/
                               # memos/、policies/、skills/、schemas/
<git-common-dir>/hctl2/       # untracked · lock、traces/、cache/ —— 仅 OS 锁与可丢弃缓存
```

HCTL 不写 Git 内部命名空间；密钥使用系统 secret store，不进入 Git、Room 或 Context。用户级 Profile/Skill/Runtime 定义以不可变 revision/digest 被引用；更新 current pointer 必须经唯一 control writer 做 expected-version CAS。某个 Repo Instance 的活动执行只读已冻结 revision，不因另一个实例更新用户级 current pointer 而漂移。

### Git 的双重角色

Git 里与 HCTL 相关的持久内容分两种；混淆“正文”和“准入”，或把审计影子当判决，会把 Git 误读成控制面的第二本账：

- **不可变正文**（家在 Repo）：Task Revision、Workflow Revision、Memo、Artifact/ChangeSet Revision 的正文，以及 Repo 共享 policy/Skill/schema revision。工具箱按已持久化 intent 写入并回读其 immutable locator 与 digest；control 账本独占稳定身份、准入决定、规范摘要、current pointer 和 lifecycle。Git 中出现一份正文不表示已被 HCTL 准入，账本也不复制一份可漂移正文。
- **判决的结晶副本**（metadata 的审计影子）：Verdict/Receipt 的权威在用户级 metadata 账本产生并保存；副本由工具箱写入 Git，用于审计与随仓库同步。副本不是第二权威——从 Git 回灌只能恢复可验证、已结晶的判决候选，仍须显式恢复流程确认，不能伪造未结晶的判决或覆盖现存账本。副本粒度按仓库策略可配：私有仓库默认全文，公开仓库可降为仅摘要。

Run Manifest、Execution Spec、绑定、租约、代次与 Result Proposal 准入是执行授权记录，不因“不可变”就自动成为 Git 正文；它们的权威只在账本。反过来，Git 正文的字节权威也不会因为账本保存了 digest 就转移到账本。

### 全系统事实权威地图

下表回答“哪类事实由谁拥有”，覆盖控制面、Git 与执行面 content 系统——它是系统地图，不是任何单一组件的存储清单。每类事实的不可用与永久丢失分开立约：不可用走降级合同（待处理 / 需要关注 / 安全暂停，不绕过命令服务），永久丢失走重建合同；产品层叙述见[三面架构](../architecture.md#数据丢了怎么办)，逐场景降级的可观察结果见[连接合同](./connections.md#失败与恢复)。

| 事实 | 权威来源 | 不可用时（降级合同） | 永久丢失时（重建合同） |
| --- | --- | --- | --- |
| 四模块 metadata：稳定身份、准入/current、Room/Request、Participant/角色绑定、权限、租约、代次、现场记账、Run Manifest、Execution Spec、Result Proposal 准入与 Verdict/Receipt | 用户级 metadata 账本 + control；一人多机连同一控制面账本 | 控制面不可用即系统不可写；客户端只读缓存投影 | 唯一不可再生的完整权威，必须备份；Git 审计影子只能辅助显式恢复，不能伪造未结晶判决 |
| Task/Workflow Revision、Memo、Artifact/ChangeSet Revision 的不可变正文与 Repo 共享 policy/Skill/schema revision；Verdict/Receipt 审计影子 | 正文字节在 Git，由工具箱写入/回读；账本保存准入、digest、current/lifecycle，且独占 Verdict/Receipt 权威 | 需要新正文或 Git 回读的命令安全暂停；结果未知先回读 | Git 分布式冗余可恢复正文；只有审计影子时仍不得自行重建判决权威 |
| Room 消息、调用过程与结果卡（content） | chat server（Matrix 协议，房间对 control 明文可读、不启用端到端加密）；控制面治理事件只保留精确事件引用与冻结 digest | 聊天入口降级（不可用显示重同步中，房间事后被加密显示需要关注）；不依赖 fresh 消息/成员/cursor 的命令可继续，依赖者拒绝 | 未结晶讨论丢失；决议与 Memo 存活于 Git，治理引用与冻结 digest 仍可校验；桥接来源可部分重放 |
| 任务卡、流转、排序、评论（content） | Repo 所选任务后端（本地任务服务器或 Linear/GitHub 等远端）；本地只存 Snapshot、身份映射和同步账本 | 看板显示待同步；不依赖 fresh placement/drift/head/cursor 的命令可继续，依赖者拒绝且不显示假成功 | 卡片与流转丢失；Task Revision 正文存活于 Git，完成权威留在账本及其可验证审计影子；远端后端由 provider 负责持久 |
| Workflow 机械位置（路标） | 通过绑定访问的 Workflow Engine | 已冻结的本地事实继续存在；Run 的完成与评审只依据账本推进，路标停更只让 Engine Execution Binding 待对账 | 路标丢失不丢任何判决：Run 按账本继续收口或显式替代；凭证链权威在 metadata 账本，审计影子在 Git |
| Harness 进程、PTY、容器、主机与原始流 | agentd / 运行时后端仅提供物理观测；绑定、租约和 lifecycle 仍由 control 记账 | 执行安全暂停或按代次收口，不冒充成功 | 转录丢失只损失回放；观测账在 metadata、ChangeSet 在 Git 存活；物理观测本就可丢弃重建 |

## 单写者

用户级 metadata 账本同时只有一个 control writer：先取得 `~/.hctl2/control.lock` 排他锁，再以 CAS 推进其 `control_writer_generation`；writer 可以搬迁（换机器、上服务器），账本身份不变。不存在 Repo 级或 Project 级的第二个 control writer。

每个 Repo Instance 的 Git/worktree 资源由 `hctl2-tool` 取得 `<git-common-dir>/hctl2/` 下的 OS 排他锁，并由唯一 control 在账本 CAS 推进该现场的 `site_generation`；这是外部资源 fence，不是本地 control 服务或第二本账。agentd 对该现场启动、输入或清理时还必须校验同一 site generation 与自己的资源 lease。失败的第二工具箱/agentd 只能只读诊断；所有改变现场事实的下游 envelope 同时携带当前 control writer generation 与适用的 site/backend generation，任一旧代次都被执行端拒绝。

每个运行时后端 ownership scope 同时只有一个 agentd owner lease 和单调 generation；scope 至少覆盖相同资源 broker/socket/host namespace。agentd 必须先取得该资源侧的 OS lock、broker token 或等价排他原语才可执行输入、停止和接管。新 owner 必须先对账，旧 generation 的输入、停止、接管和结果一律失权；不能强制排他的 backend 只可观察，不开放这些写能力。

SQLite 锁不是外部副作用隔离。幂等键、generation、租约、outbox 和 readback 必须共同工作。

## 启动与恢复

恢复顺序固定为：

1. 取得用户级 control 锁，并经工具箱/agentd 取得适用现场与 backend 的 OS/资源侧排他权，禁止旧 owner 继续执行动作；
2. 打开权威账本、验证 schema，恢复 inbox/outbox/租约，并 CAS 推进 control writer、site 与 backend generation；
3. 回读全部已绑定 content 系统的游标（chat server、任务后端、Workflow Engine、runtime）和未确认副作用；
4. 查询 Workflow Engine、agentd runtime 和工具箱 Git/SCM；
5. 将观测分类为运行、等待、丢失、被替代、孤儿或结果未知；
6. 隔离旧 generation，只重放可证明幂等且仍获准的动作；
7. 对账完成后才授予新的写入或输入租约。

UI 重载只重建投影。无法证明同一执行身份时，宁可标记丢失或要求人工对账，也不能自动接管或伪造成功。

### 备份与恢复

metadata 备份必须是由唯一 writer 协调的一致备份集：完整账本快照，连同账本引用的精确用户级 Profile/Skill/Runtime 不可变定义字节与 digest——后者存放在账本之外，单备份账本文件会漏掉它们。secret value、可丢弃 cache、PTY 原始流和场景 content 正文不进入该备份集。备份完成前验证快照边界、所有定义引用/校验和与 schema 可读性；Repo Git、content 系统与 secret store 按各自合同另行备份，缺少其中之一不能用伪造 Receipt 补齐。

恢复只能在旧 writer 已停止且取得用户级排他锁后进行；不得合并两份分叉账本或把备份恢复成新的账本身份。恢复保留原 ledger identity，推进 control writer 及所有可能仍存活的 site/backend generation，令旧 descriptor、lease、outbox 执行权和 Result Proposal 失效，再按上述顺序回读结果未知的外部副作用。secret reference 仍在但值缺失时，对应 binding 标为不可用并阻止依赖命令，不能把空值当凭据或静默降权。

## 安全边界

- Electron renderer、Web 内容、终端转义序列和外部消息都视为不可信输入。
- 打包后的 Electron 固定 `nodeIntegration=false`、`contextIsolation=true`、sandbox=true；narrow preload 只暴露具名 typed command，不暴露 raw ipcRenderer。禁止 remote runtime script/CDN，CSP 拒绝远程或未声明的可执行来源。
- 文件、Git、网络、凭据和进程能力由 control、工具箱和 agentd 授权，不交给渲染器。
- 敏感输入不进入 Room、日志、Context 或终端回放。
- 日志与 trace 使用稳定关联 ID，但不得包含密钥和完整敏感 payload。
- 第一阶段是单用户授权模型，不提供多租户隔离，也不声称在任意同 OS 用户恶意进程已经取得账户权限后仍能保护整个账户。HCTL 自己启动的 Harness 仍以窄 execution principal 运行、不获交付集成与外部写凭据、只在独立 worktree 与 Write Lease 内写入并受 site/backend fence 约束；OS 执行沙箱、凭据网关代用范围与网络白名单是 Worker Profile 可声明的加固，未启用时 Harness 与同 OS 用户的其他进程处于同一信任域，合同只承诺上述三条底线在治理面成立，不承诺宿主文件系统层面的隔离。

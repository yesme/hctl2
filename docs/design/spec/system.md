# 系统边界与适配器约束

> 状态：规范性约束 · 草案 v0.16.3<br>
> 本文只定义四个模块共享的运行机制，不拥有 Project、Task、Run 或 Participant 的领域状态。

## 组件

| 组件 | 职责 |
| --- | --- |
| `hctl2-workbench` | 无特权的组合客户端；承载四个场景的 provider 交互、HCTL 公共命令入口、联合投影与导航 |
| `hctl2-control` | 唯一领域命令服务，负责路由、权限、账本、outbox 和对账；内含 Herdr 适配代码，但不实现终端会话服务 |
| `hctl2-tool` | 执行现场操作：物化和隔离 Git 工作树/ChangeSet，执行已持久化的意图，回读结果，回读外部机械事实（CI 状态、合并状态、引用推进、路径与摘要）供节点准入与证据使用，并管理现场锁、封存和 Git 事实校验。它不负责 lint 或代码检查，也不执行远端 SCM 副作用。独立运行时，它只提供普通本地操作，不签发 HCTL 治理结果 |
| Herdr | 本地 Agency 参考实现的运行时，实现 Agency 端口：按规格启动 Harness，持有进程、PTY 和终端会话，并提供 API 与原生 TUI |
| workflow engine | 通过适配器保存 Run 的引擎执行令牌、引擎步骤、定时器、重试次数和执行历史 |
| chat server | 经 Chat 端口访问的聊天服务器（Matrix 协议）；承载 Room 消息 content 的 ground truth |
| task backend | 经任务源端口访问的任务后端（本地任务服务器或远端平台，按 Repo 选择）；承载任务卡 content 的 ground truth |
| 第三方场景平台 | 可以同时提供场景客户端和受控端口；这两种接入分别绑定，任何一种都不会因此取得 HCTL 治理权威 |

## 固定内核与受控端口

固定内核是一个用户级命令服务；它对每个 Repo 保持独立的语义范围，而不是在每个仓库副本各起一套控制面。固定内核实现四模块定义的稳定身份、Revision 准入、权限、字段权威、领域归约与 Receipt；本文件只拥有共享命令信封、扩展绑定、outbox/inbox、单写者和恢复机制。

换掉全部界面与供应端之后内核必须保留什么，[愿景文档](../vision.md#产品原生核心与架构最小内核)已经回答；本文只立它在系统层的精确约束。

可以替换的端口包括：

| 场景 | 端口 |
| --- | --- |
| Room | chat server 连接：消息/附件读写、账号与房间管理、身份映射（非 Matrix 平台经 homeserver 侧 Matrix 桥接接入，不是 HCTL 端口） |
| Kanban | 任务源端口读取、字段写回与快照 |
| Workflow | workflow engine 编译、注册、执行和回读 |
| Terminal | harness、Agency，以及终端连接与输入能力 |

受控端口是替换供应端的唯一边界，不再增加跨模块通用适配层。每个 Port–Provider Binding 固定供应端制品、模块适配器、配置摘要、实测能力和降级方式。适配器只翻译本模块实际使用的命令、查询和事件，不把供应端私有对象提升为 HCTL 对象。新供应端通过本模块的契约测试后，只能用于新绑定；已有执行继续使用原绑定。迁移既有 content 必须另走显式的预览、导出、导入和回读校验。

### 客户端使用的公开面

Workbench 的 HCTL 功能只调用 Query、Preview、Submit、Subscribe 和各模块投影。供应端客户端功能使用供应端的公开协议或客户端传输适配器。Workbench 可以直连精确终端的观察流和普通交互输入。未来的远程 Agency 必须直接实现 Agency 约束，或通过专用适配器接入；非 Matrix 聊天平台仍通过 Matrix homeserver 的桥接生态接入。

### 端点与输入的信任边界

1. chat server、本地任务服务器、workflow engine 和 Herdr 管理/API 端点只能绑定本机回环地址或仅归属者可访问的本地套接字；非本地传输必须认证客户端。
2. 若某个供应端修改必须先由 HCTL 记账、撤权或校验，只有 control 可以通过对应受控端口发起它。chat server 与任务后端的 content 读写不受这条限制；Herdr 的普通交互输入也不是治理命令。
3. 当绑定声明支持代次栅栏回显和逐次输入记录时，Herdr 适配代码必须在每次输入前校验 Attach Descriptor、Terminal Input Lease 和当前代次。绑定未声明这些能力时，原生交互只能按来源不完整的运行时输入记录。
4. 若允许 Herdr TUI 原生输入，绑定必须明确记录它不提供 HCTL 单输入租约保证。无论采用哪种输入模式，HCTL 结果都只能从 Result Proposal 准入。

hctl2-control 托管执行面服务器的生命周期：随 HCTL 一键启停，启动顺序、健康检查、备份与升级由 control 统一编排。托管不授予 content 之外的任何权威；服务器进程的死活只影响对应场景的可用性，不改变治理事实。

每个扩展绑定都冻结代码版本、接口/schema、配置摘要、依赖、能力和信任级别；活动执行固定使用准入时的绑定。绑定的供应端不可用时，control 必须按该绑定冻结的降级策略暂停或终结活动执行。只有归属者提交显式替代命令后，control 才能创建替代执行。

control policy 只信任来自允许来源且摘要匹配的扩展制品。扩展或注册表的自我声明不授予信任。发现操作默认只读取已配置定义、本地安装和无副作用探测；联网探测只能由用户显式启用。安装与升级必须由用户显式提交，并产生新的 Extension Revision；后续解析再产生新的 Port–Provider Binding，活动执行继续使用原绑定。

跨模块可引用的扩展信息只有两个最小概念：

- `Extension Revision`：扩展稳定身份的一份不可变版本，固定代码/制品摘要、接口与 schema、依赖、声明能力和信任级别；
- Port–Provider Binding：一次已解析端口选择，固定 binding_revision_id、Extension Revision、供应端与安装实例、配置摘要、凭据引用、实测能力、权限作用域和降级策略。

同一端口种类和作用域在一次准入中只能解析出一个绑定版本，结果不得受加载顺序或界面选择顺序影响。Room、Task、Run 和 Execution Spec 都必须引用精确绑定版本。历史执行继续使用原绑定。凭据引用只指向 secret store 条目，不得包含密钥。

健康状态、重同步游标和当前成员列表属于运行时投影。每份投影必须记录来源、版本或序号以及观测时间，但不参与不可变绑定的摘要计算。它们可以阻止新准入或驱动对账，但不会改写历史绑定。只有端点、配置、能力、信任、权限或降级规则变化时，control 才创建新的绑定版本。

Skill 的定义、三态与申报规则见 [Participant 模块约束](./participant.md#skill-与申报)：Skill 的内容由 Agency 安装并申报，Execution Spec 与 Run Manifest 冻结的是申报的精确 ref+digest 及其可核验性。

进程内扩展等同受信任代码。普通独立进程只隔离崩溃；不可信扩展必须使用操作系统强制隔离和能力削减的代理接口。

## 场景端口

所有 HCTL 命令客户端共享四类操作：

```text
Query(filters, cursor) -> Projection
Preview(command draft) -> Effect summary + preconditions
Submit(typed command) -> accepted result or typed rejection
Subscribe(cursor) -> ordered events or resync snapshot
```

客户端只能声明自己的交互能力和降级方式，受控端口只能报告供应端能力。字段写入权由对应模块的绑定授予。外部平台可以拥有场景 content 和明确授权的字段，但其 thread、Issue、workflow task、Session 或 pane 都不能成为 HCTL 的身份、授权或判决来源。

界面控件本身没有固定语义：同一个拖放可以只是 Task content 移动，也可以在进入 Done 时产生一个 human 完成请求；终端输入推动精确运行时，不是领域命令。具体分类见下一节。若某个动作必须提交 HCTL 命令，而客户端无法提供等价的预览、版本或权限信息，客户端必须禁用该动作、保留待处理请求或返回安全拒绝。

## 客户端动作与 provider 事件

control 不根据“来自哪个产品”判定动作——客户端没有等级是愿景层的产品承诺（见[设计原则](../vision.md#设计原则)）——而根据动作落点和可验证信封分成以下几类：

| 类别 | 例子 | control 怎样处理 |
| --- | --- | --- |
| content 写入与观测 | Matrix 消息；Vikunja 创建、编辑、排序、非终态移动 | provider 先拥有该 content；control 按 cursor/Snapshot 对账，按模块约束更新投影或建立无契约身份，不把 content 直接当治理事实 |
| human 命令请求 | Workbench/CLI 的类型化提交；已配置的 Matrix 结构化动作；绑定卡片进入 Vikunja Done | 归一到同一个 HCTL command draft，按同一准入规则处理；危险动作（不可逆、产生外部权威副作用或扩大权限）默认先经 Preview 确认，普通命令可直接 Submit；需要临场选择、危险动作未经确认或绑定未允许该来源自动提交时，保留为待处理或返回类型化拒绝 |
| 运行时输入 | Workbench Terminal、Herdr TUI 或其他终端客户端向精确 Execution Runtime 输入 | 立即推动该运行时；按连接票据、租约和代次能力记录恢复等级，但不因此产生 Task/Run/Project 结果 |
| 执行结果提案 | harness/Agency 的结构化终局事件与证据 | 只进入 Result Proposal；归属模块按版本、代次和证据准入 |
| 不支持的供应端修改 | 直接在 Dagu UI 启动、停止、重试、批准，或改写已绑定执行 | 记录当前机械事实并标记绑定分歧；不倒推一条 HCTL 命令，也不补签 Receipt |

供应端事件先按 content 事实入账。只有事件能证明 actor、目标、前后版本和幂等依据时，Task 适配器才可以另行生成“完成 Task”请求。请求被拒绝时，界面同时保留供应端 Done 与 HCTL 开放状态。由 HCTL 回写产生、actor 无法映射或字段不全的事件只能形成 Snapshot。

授权模型是单用户的：只要求把直接客户端连接或供应端账号稳定映射到归属 human，并保留 `direct_client | provider_event | internal_reducer | execution_principal | unknown` 来源；不引入组织、角色层级或复杂 RBAC。来源简化不等于可以丢掉目标、预期版本或代次、幂等键和事件顺序。

webhook 和通知只负责唤醒。接纳前仍以供应端当前回读、游标及其缺口和模块 Snapshot 为准；重复、迟到或乱序投递必须得到相同结果。各模块可接受的供应端动作见[场景与第三方适配器](./connections.md#场景与第三方适配器)。

## 命令与跨服务正确性

改变事实的命令至少携带：

```text
command_id / idempotency_key
actor principal + actor source/provenance + permission scope
target stable ID
expected revision or state version
frozen adapter binding
canonical input digest
```

actor 来源只能由直接客户端连接、绑定中的账号映射或 control 内部归约器赋予。治理命令只接受两类来源：可映射到归属 human 的动作，以及绑定 Task 的 Run 正常完成后由 control 归约器发出的内部命令。普通 Room 的临场扇出只接受有权的 human actor；workflow 归约器只能实例化 Workflow Revision 已冻结的边。Harness、模型和执行主体只能提交 Result Proposal，不能自报为 human。

control 必须在同一个 SQLite 事务中写入领域事件、幂等结果和 outbox；跨模块命令也只能使用这一个事务边界。外部适配器使用同一幂等键投递并回读；结果未知时不得盲目重做。重复命令返回原结果，异载荷复用同一幂等键时必须拒绝。

组件不可用不会改变命令前置条件：不依赖该组件当前事实的命令可以继续，依赖当前回读的命令必须拒绝。只有冻结策略明确允许陈旧证据时，命令才能携带证据版本、观测时间和已知缺口继续。

Receipt 证明的是已经校验的结果，不是另一个 writer。投影从事件重建；缓存和界面状态保持为派生数据。

跨模块引用的规范摘要统一使用 RFC 8785 JCS 规范对象的 SHA-256，摘要字段自身不参与计算。每个领域归属者只定义自己规范对象包含哪些字段。完整 Revision 的 revision_digest 与为评审选取字段生成的 review_subject_digest 是不同语义，分别按各自语义使用，与某次字节是否相同无关。

## 外部权威副作用

包括远端 SCM 在内，会改变第三方权威事实的动作统一写成持久外部副作用命令和 outbox 记录。记录固定归属者引用、Port–Provider Binding、操作、目标、适配器声明的冲突范围、权限、规范输入摘要和幂等键。同一远端资源的关闭、重开、更新等操作共用一个冲突范围。

本地 Git 变更属于同一类外部副作用命令：control 先持久化意图和 outbox，工具箱再执行和回读；Harness 和模型不直接取得集成权。适配器只投递并回读。只有适配器确认目标、版本和结果后，control 与工具箱的校验事务才能写成功 Receipt。投递超时或确认回执丢失时，结果保持未知，并继续占用冲突范围以阻止重叠写。Harness 的窄执行主体、凭据与独立 Git 工作树边界见[Participant 写入约束](./participant.md#写入约束)。

系统不承诺自动补偿任意外部写。供应端事件只有在对应模块明确列为 content、human 命令请求或运行时输入时，才按相应路径处理；其余修改由对应端口或工具箱回读为 Snapshot 或分歧，并阻止依赖旧版本的命令，直到用户通过该模块既有的采纳或对账动作处理。

Harness 不获得可绕过受控端口的外部写凭据。本地目标引用被 Harness 或用户在“合入 ChangeSet”命令之外直接改写时，只表现为预期目标头不匹配的分歧。外部观测只进入 Snapshot 或 Result Proposal；Artifact、Verdict 与 Receipt 仍由对应归属者按约束产生。

## 事实与存储

### Repo 与执行现场

Repo 是四模块可归属的逻辑仓库，由 [Project 模块](./project.md#repo-注册与-project-归档)注册；Repo Instance 是本系统拥有的物理执行现场，不属于任何 Project。一个 Repo 可以显式挂接多个 Repo Instance。每个现场固定稳定 `repo_instance_id`、精确 `repo_id`、主机与站点身份、Git 公共目录身份，以及首次校验的 Git 证据；Git 工作树、ChangeSet 物化、工具箱锁与本机运行时都通过该现场引用。

“挂接 Repo Instance”命令先由工具箱无副作用读取 Git 身份，再由 control 预览并写入账本。相同 Git 公共目录的重试返回原现场；不同现场只有在 Git 中的稳定 Repo 身份与命令指定的 `repo_id` 一致时才能挂接。远端 URL、目录名或碰巧相同的 HEAD 只作辅助证据。

身份缺失、分支来源语义不明、一个公共目录已归属另一 Repo，或证据互相冲突时，系统不得静默挂接。界面展示全部证据；用户明确选择挂接到指定 Repo、注册新 Repo 或修复来源后，系统才按该选择继续。移除现场只撤销其新执行资格，不删除 Repo、Project、历史 Run 或已封存 ChangeSet。

### 控制面自己的存储

hctl2-control 的存储只有一本库：**用户级 metadata 账本**。它是全部 metadata 的唯一权威，包括稳定身份、Revision 准入与 current、绑定、授权、租约、代次、现场记账、Run Manifest、Execution Spec、Result Proposal 准入，以及 Verdict/Receipt。一人多机连接同一本账本，账本必须备份。

仓库副本本地的 `<git-common-dir>/hctl2/` 是当前 Repo Instance 及其关联 Git 工作树的共享运行目录，只保存 OS 锁、跟踪记录与可丢弃缓存。它**不是账本，也不是事实源**。现场状态始终可以从 metadata 账本、Git 与运行时观测对账重建；删除该目录不丢失事实，无法证明身份的旧执行会被标为丢失并撤权。

control 也会把结果写到自己的库以外，但那些是外部副作用的目标，不是另一份 metadata 账本：获准的不可变正文与判决审计影子经工具箱写入 Git（见下节）；获准的记录可以写回 content 系统（记录不是命令）。

账本只保存 HCTL 自己的领域关系、授权与判决，以及 HCTL 身份到外部 content/运行时身份的跨系统锚定；承载系统内部的完整拓扑（如任务后端里与 HCTL 无关的卡片层级）仍由提供方拥有。控制面凭获准命令、精确映射与 Snapshot 对账需要治理的那部分外部关系。账本与其余本地存储（锁、缓存、定义文件）的物理布局是控制面的**私事**：事实经服务接口流通，路径和表结构不构成对外 API，也不进 Git；“唯一用户级账本、权威归属和备份传承”由架构约束固定，独立于实现选择。

存储拓扑默认为（路径与布局是实现选择，三类存储的职责边界才是约束）：

```text
~/.hctl2/                      # 用户级配置、Harness/Profile/Skill/Runtime 定义
                               # control.sqlite、control.lock —— 用户级 metadata 账本与写锁
<repo>/.hctl2/                # Git tracked · repo.toml、projects/、workflows/
                               # memos/、policies/、schemas/
<git-common-dir>/hctl2/       # untracked · lock、traces/、cache/ —— 仅 OS 锁与可丢弃缓存
```

是否使用 Git 内部命名空间（refs/notes）是实现选择，默认不写；使用时它们同样只是缓存或审计影子，不构成第二本账。密钥使用系统 secret store，不进入 Git、Room 或 Context。用户级 Profile/Skill/Runtime 定义以不可变 revision/digest 被引用；更新 current pointer 必须经唯一 control writer 做 expected-version CAS。某个 Repo Instance 的活动执行只读已冻结 revision，不因另一个实例更新用户级 current pointer 而漂移。

### Git 的双重角色

Git 里与 HCTL 相关的持久内容分两种；混淆“正文”和“准入”，或把审计影子当判决，会把 Git 误读成控制面的第二本账：

- **不可变正文**（家在 Repo）：Task Revision、Workflow Revision、Memo、Artifact/ChangeSet Revision 的正文，以及 Repo 共享 policy/schema revision。工具箱按已持久化 intent 写入并回读其 immutable locator 与 digest；control 账本独占稳定身份、准入决定、规范摘要、current pointer 和 lifecycle。Git 中出现一份正文不表示已被 HCTL 准入，账本也不复制一份可漂移正文。
- **判决的结晶副本**（metadata 的审计影子）：Verdict/Receipt 的权威在用户级 metadata 账本产生并保存；副本由工具箱写入 Git，用于审计与随仓库同步。副本不是第二权威——从 Git 回灌只恢复可验证、已结晶的判决候选，仍须显式恢复流程确认；未结晶的判决和现存账本保持原权威。副本粒度按仓库策略可配：私有仓库默认全文，公开仓库可降为仅摘要。

Run Manifest、Execution Spec、绑定、租约、代次与 Result Proposal 准入是执行授权记录，不因“不可变”就自动成为 Git 正文；它们的权威只在账本。反过来，Git 正文的字节权威也不会因为账本保存了 digest 就转移到账本。

### 全系统事实权威地图

下表回答“哪类事实由谁拥有”，覆盖控制面、Git 与执行面 content 系统；它是系统地图，不是任何单一组件的存储清单。每类事实都要分别说明两种故障：暂时不可用时如何降级，永久丢失后如何重建。产品层叙述见[三面架构](../architecture.md#数据丢了怎么办)，逐场景降级的可观察结果见[连接约束](./connections.md#失败与恢复)。

| 事实 | 权威来源 | 不可用时怎么降级 | 永久丢失时怎么重建 |
| --- | --- | --- | --- |
| 四模块 metadata：稳定身份、准入/current、Room/Request、参与者授权、权限、租约、代次、现场记账、Run Manifest、Execution Spec、Result Proposal 准入与 Verdict/Receipt | 用户级 metadata 账本 + control；一人多机连同一控制面账本 | 控制面不可用即系统不可写；客户端只读缓存投影 | 唯一不可再生的完整权威，必须备份；Git 审计影子只能辅助显式恢复，不能伪造未结晶判决 |
| Task/Workflow Revision、Memo、Artifact/ChangeSet Revision 的不可变正文与 Repo 共享 policy/schema revision；Verdict/Receipt 审计影子 | 正文字节在 Git，由工具箱写入/回读；账本保存准入、digest、current/lifecycle，且独占 Verdict/Receipt 权威 | 需要新正文或 Git 回读的命令安全暂停；结果未知先回读 | Git 分布式冗余可恢复正文；只有审计影子时仍不得自行重建判决权威 |
| Room 消息、调用过程与结果卡（content） | chat server（Matrix 协议，房间对 control 明文可读、不启用端到端加密）；控制面治理事件只保留精确事件引用与冻结 digest | 聊天入口降级（不可用显示重同步中，房间事后被加密显示需要关注）；不依赖当前消息、成员或游标的命令可继续，依赖者拒绝 | 未结晶讨论丢失；决议与 Memo 存活于 Git，治理引用与冻结 digest 仍可校验；桥接来源可部分重放 |
| 任务卡、流转、排序、评论（content） | Repo 所选任务后端（本地任务服务器或 Linear/GitHub 等远端）；本地只存 Snapshot、身份映射和同步账本 | 看板显示待同步；不依赖当前放置位置、分歧、来源头或游标的命令可继续，依赖者拒绝且不显示假成功 | 卡片与流转丢失；Task Revision 正文存活于 Git，完成权威留在账本及其可验证审计影子；远端后端由 provider 负责持久 |
| workflow engine 报告的执行进度 | 通过绑定访问的 workflow engine | 已冻结的本地事实继续存在；Run 的完成与评审只依据账本推进，引擎停报进度只让 Run–Engine Binding 待对账 | 进度报告丢失不丢任何判决：Run 按账本继续结束或显式替代；凭证链权威在 metadata 账本，审计影子在 Git |
| Harness 进程、PTY、主机与原始流 | Herdr 持有物理资源并提供观测；绑定、租约和 lifecycle 仍由 control 记账 | 执行安全暂停或按代次结束，不冒充成功 | 转录丢失只损失回放；观测账在 metadata、ChangeSet 在 Git 存活；物理观测本就可丢弃重建 |

## 单写者

用户级 metadata 账本「只允许唯一写者」的约束只有三条底线：同时只有一个逻辑 control writer、已确认副作用不重复执行、旧结果不覆盖新结果。当前实现先取得 `~/.hctl2/control.lock` 排他锁，再以 CAS 推进其 `control_writer_generation`——锁路径与推进机制是实现细节，不构成对外约束。writer 可以搬迁（换机器、上服务器），账本身份不变。不存在 Repo 级或 Project 级的第二个 control writer。

Git 工作树的现场互斥由 `hctl2-tool` 的 OS 锁保证，control 只在账本中以比较并交换推进该现场的 `site_generation`。该代次栅栏不是本地 control 服务或第二本账。

Agency 端口的 Port–Provider Binding 另有自己的归属者租约和代次；范围至少覆盖同一服务器、套接字或主机命名空间。新的归属者必须先对账，HCTL 不再向旧代次签发输入、停止、接管或结果准入。

适配器在启动、输入和停止前必须校验适用的现场与运行时代次。只有工具箱持有的 OS 锁能在现场强制排除旧 Git 写入。供应端不能接收或回显代次时，只能声明 HCTL 入口已校验，不能声明物理执行点已经隔离旧动作。

SQLite 事务只保证账本内部一致，而事务提交与外部投递不在同一原子域。因此，外部副作用还必须由幂等键、代次、租约、outbox 和回读共同隔离。

单写者、CAS 与 current pointer 是三件事，不互相替代：单写者回答此刻谁有权写账本（本节）；expected-version CAS 回答一条命令对哪个版本生效（见[命令与跨服务正确性](#命令与跨服务正确性)）；current pointer 回答界面此刻读哪个不可变版本（推进规则由各模块写入约束定义）。ChangeSet 的单 writer 是 Participant 模块的写权（见[Participant 约束](./participant.md#写入约束)），不属于 control 单写者。

### 代次家族

全系统共有六个代次（generation）：同族——都单调递增、旧值失权——但不同槽，各管一块资源，不得共用一个字段，也不得从彼此推导。

| 成员 | 管哪块资源 | 权威定义 | 何时产生或推进 |
| --- | --- | --- | --- |
| `attempt_generation` ／ `invocation_version` | 这一次逻辑执行归谁（语义归属者） | [Run 约束](./run.md#对象)、[Project 约束](./project.md#room-invocation) | 派发时已有；不得预填运行时身份 |
| `runtime_generation` | 这一次物理进程／PTY | [Participant 约束](./participant.md#运行时与观测) | 激活映射时由 Agency 预留返回 |
| `control_writer_generation` | 用户级账本此刻的逻辑写入者 | 本节 | 取得账本写权时 CAS 推进 |
| `site_generation` | 某个 Repo Instance 的 Git 工作树现场 | 本节 | 同一本账对本现场以比较并交换推进；现场 OS 锁是它的物理伴生，不是它 |
| Agency binding owner generation | 某个 Agency 端口 Port–Provider Binding 的范围（同一服务器、套接字或主机命名空间） | 本节 | 新归属者对账后推进；旧代次不再获签发输入、停止、接管或结果准入 |
| `engine_binding_generation` | 某次 Run 与引擎 execution 的绑定 | [Run 约束](./run.md#从节点到结果) | 启动、关闭或标记分歧时推进；走 Run↔引擎连接，不进执行体出站元组 |

四条推导规则彼此独立：

- `runtime_generation` 不能从 `attempt_generation` 推导，因为派发时还没有物理身份；
- `site_generation` 不能从 `control_writer_generation` 推导，因为写入者可以搬家，而现场固定在仓库实例上；
- Agency binding owner generation 不能从 `site_generation` 推导，因为 Git 锁管不了另一台机器上的 PTY；
- `engine_binding_generation` 不能从 `attempt_generation` 推导，因为引擎重试与候选切换是两条独立的换代路径。

执行体出站结果必带哪些代次、`in_process` 何时可以缩减，见[连接约束](./connections.md#project--run--participant从授权到物理执行)。Participant revision、binding revision、producer sequence 与 content cursor 都不是代次。

## 启动与恢复

恢复顺序固定为：

1. 取得用户级 control 锁，并经工具箱取得适用现场的 OS 排他权；Herdr 绑定不支持物理代次栅栏时明确记录该限制；
2. 打开权威账本、验证 schema，恢复 inbox/outbox/租约，并 CAS 推进 control writer、site 与 Agency binding generation；
3. 回读全部已绑定 content 系统的游标（chat server、任务后端、workflow engine）以及 Herdr 运行状态和未确认副作用；
4. 查询 workflow engine、Herdr API 和工具箱 Git/SCM；
5. 将观测分类为运行、等待、丢失、被替代、孤儿或结果未知；
6. 隔离旧 generation，只重放可证明幂等且仍获准的动作；
7. 对账完成后才授予新的写入或输入租约。

UI 重载只重建投影。无法证明同一执行身份时，系统标记丢失或要求人工对账，自动接管与成功判定保持关闭。

### 备份与恢复

metadata 备份必须是由唯一写入者协调的一致备份集：完整账本快照，连同账本引用的精确用户级 Profile/Skill/Runtime 不可变定义字节与摘要。后者存放在账本之外，单备份账本文件会漏掉它们。密钥值、可丢弃缓存、PTY 原始流和场景 content 正文不进入该备份集。

备份流程必须在完成前验证快照边界、全部定义引用与校验和，以及 schema 的可读性。Repo Git、content 系统与 secret store 按各自约束另行备份；缺少其中之一时，系统不得用伪造 Receipt 补齐。

恢复只能在旧写入者已经停止且取得用户级排他锁后进行；不得合并两份分叉账本，也不得把备份恢复成新的账本身份。恢复保留原账本身份，推进 control writer、site 与所有可能仍存活的 Agency binding owner generation，使旧连接票据、租约、outbox 执行权和 Result Proposal 失效，再按上述顺序回读结果未知的外部副作用。

密钥引用仍在但值缺失时，对应绑定标为不可用并阻止依赖命令；系统不得把空值当凭据或静默降权。

## 安全边界

- 桌面壳 WebView/renderer、Web 内容、终端转义序列和外部消息都视为不可信输入。
- 打包后的桌面壳固定最小权限面，WebView 只暴露具名 typed command：Tauri 2 按 window/webview 以 capability/permission/scope 显式声明，不开放未声明的 IPC 与插件能力；以 Electron 安全网形态发行时固定 `nodeIntegration=false`、`contextIsolation=true`、sandbox=true，narrow preload 不暴露 raw ipcRenderer。禁止远程运行时脚本/CDN，CSP 拒绝远程或未声明的可执行来源。
- 文件、Git、网络、凭据和进程能力由 control 与工具箱授权，不交给渲染器。
- 敏感输入不进入 Room、日志、Context 或终端回放。
- 日志与 trace 使用稳定关联 ID，但不得包含密钥和完整敏感 payload。
- 授权模型是单用户的，不提供多租户隔离；Harness 的三条底线与可选执行加固见[Participant 写入约束](./participant.md#写入约束)。未启用加固时，Harness 与同 OS 用户的其他进程处于同一信任域。

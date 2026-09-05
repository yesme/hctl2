# Repo 模块约束

> 状态：规范性约束 · 草案 v0.17.0<br>
> 本文是 Repo 模块对象、状态机与写入约束的唯一权威。设计正文见 [Repo 与 Change](../repo.md)；模块交接见[连接约束](./connections.md)，共享机制见[系统边界](./system.md)，族语义与词汇分类见[约束层总则](./README.md)。

## 对象

Repo 模块拥有逻辑仓库的稳定身份与注册、物理执行现场、获准的写入边界及其不可变 Git 快照、写入租约、代码集成的持久意图与凭证，以及变更与代码协作平台之间的映射。Project 拥有 Repo Room 与 Project 对 Repo 的归属；Participant 拥有执行体、运行时与观测；Run 拥有 Gate、Seat 与 Verdict。本模块不决定 Project、Task 或 Run 的领域结果，也不接收 Result Proposal。

`hctl2-tool`（工具箱）是本模块的现场执行者：物化与隔离 Git 工作树、封存与保全 ChangeSet、执行面向本地目标的集成、回读 Git 事实与闭集外部机械事实。平台适配器是本模块经平台端口接入代码协作平台的适配代码，只做平台上才有的写动作与 content 读取：推送变更集分支、创建或更新评审请求、请求合并、写回记录，以及为代取读取评审评论正文。平台上的机械事实——提交的检查状态、评审请求当前头与是否合并、线程是否解决、正式评审状态、目标 ref 的保护条件——仍由工具箱回读，证据通道等级为工具箱回读，Run 的节点前置只认它。两者是分工：有平台的仓库照样由工具箱物化、封存与回读事实。

| 对象 | 含义 |
| --- | --- |
| Repo | Git 内容与共享配置的逻辑仓库；稳定身份写入 Git 并记账 |
| Repo Instance | 本系统拥有的物理执行现场：某台主机上一个 Git 公共目录及其工作树；不属于任何 Project |
| ChangeSet / ChangeSet Revision | 一次获准写入边界及其不可变 Git 快照（Revision 族） |
| Write Lease | ChangeSet 的独占写入权与失权拦截（Lease 族） |
| 集成意图（外部副作用命令，executor = tool \| adapter）/ Integration Receipt | 把精确 ChangeSet Revision 集成到目标 ref 的持久授权，及回读确认后的唯一证明（命令族 / Receipt 族） |
| 发布评审意图（外部副作用命令，executor = adapter） | 按 Execution Spec 冻结的评审发布策略，把精确 ChangeSet Revision 推送到平台并创建或更新评审请求的持久授权（命令族） |
| ChangeSet–Platform Binding | ChangeSet Revision 到平台上对应提交与评审请求的映射（Binding 族）：证据部分冻结，状态部分为回读 |
| 平台端口的 Port–Provider Binding | 仓库一级的平台连接：供应端、平台上的仓库身份、凭据引用、账号映射、目标保护条件与实测能力；族定义见[系统边界](./system.md#固定内核与受控端口) |
| 目标保护快照 | 集成意图的字段组，不是对象：预览时回读的目标 ref 保护条件（必须经评审请求与否、必需检查清单、是否要求与目标同步、是否要求线程解决、要求的批准数） |
| 外部机械事实 | 工具箱回读的闭集事实（Snapshot/观测族）：提交的检查状态、评审请求当前头与是否合并、线程是否解决、正式评审状态、目标保护条件、引用是否推进、路径与摘要 |

ChangeSet 在[核心产品词](./README.md#核心产品词)中仍是治理内部词：Change 场景给人看的是精确 diff、评审讨论、检查与集成状态的投影，不是这个对象本身。

## 写入约束

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| Repo | stable `repo_id` + `repo_version`；待确认 / 活跃 | control 处理「注册 Repo」命令；工具箱只写入/回读 Git identity | 一个 Repo identity 只有一个 Repo 与 Repo Room；待确认的外部写入按关联键恢复，不重复注册 |
| Repo Instance | stable `repo_instance_id` + `site_generation`；活跃 / 已移除 | control 处理「挂接/移除 Repo Instance」命令；工具箱无副作用读取 Git 身份并持有现场 OS 锁 | 相同 Git 公共目录的重试返回原现场；移除只撤销新执行资格，不删除 Repo、Project、历史 Run 或已封存 ChangeSet |
| ChangeSet / Write Lease | change_set_version、current revision；租约为待启动 / 活跃 / 撤销中 / 已撤销 | control 准入「预备/授予/撤销/封存」命令，工具箱物化并回读 Git 并执行失权 | 一个 ChangeSet 至多一个活跃租约；ChangeSet Revision 只追加 |
| 集成意图 / Integration Receipt | intent state version；待启动 / 结果未知 / 成功 / 失败；Receipt immutable | control 准入「合入 ChangeSet」命令；面向本地目标由工具箱执行与回读，面向远端目标由平台适配器投递、工具箱回读 | 同一集成意图只有一个获准结果与唯一 Receipt；同一 target ref 同时至多一个待决（待启动或结果未知）的集成意图，不论授权形态，共用[系统边界](./system.md#外部权威副作用)定义的冲突范围；前一意图终态后，同一目标的下一意图是新的授权；只有回读确认才能写 Receipt |
| 发布评审意图 | intent state version；待启动 / 结果未知 / 成功 / 失败 | control 按 Execution Spec 冻结的评审发布策略准入「发布评审」命令；平台适配器推送并创建或更新评审请求 | 同一 ChangeSet Revision 与同一发布目标只允许一条评审请求映射；确认丢失时按原意图回读，不重复创建 |
| ChangeSet–Platform Binding | immutable evidence + readback state version | control 在发布评审、更新评审请求与集成回读时记录；平台适配器只回读 | 「某版本对应平台上哪个提交」冻结后不改写；评审请求的当前头、检查与线程状态为回读，随时变 |
| 平台端口的 Port–Provider Binding | immutable revision + current pointer；活跃 / 停用 / 已替换 | control 处理「绑定/换绑/停用平台」命令；适配器只探测与回读 | 活动意图沿用准入时的绑定版本；换绑不改写历史映射与 Receipt |

## Repo 注册与 Repo Instance 挂接

Repo 是逻辑仓库，不等于外部组织、工作区或仓库副本。注册命令固定 `repo_id`、预期 Git 身份、配置正文摘要和幂等键。control 先记录待确认注册和 outbox，工具箱再写入并回读 Git 身份。结果未知时只能按原身份和摘要恢复同一次注册；缺失或冲突必须要求用户处理，不得静默合并。

Git 已存在获准身份时，命令校验后复用它。注册确认事务激活唯一 Repo 身份，并由 [Project 模块](./project.md#repo-注册与-project-归档)在同一账本事务中创建其唯一 Repo Room；待确认 Repo 不接受 Project、Task 或 Run。不同仓库副本通过下文的显式现场挂接连接同一 Repo，而不成为 Project 的子对象。

Repo Instance 是本系统拥有的物理执行现场，不属于任何 Project。一个 Repo 可以显式挂接多个 Repo Instance。每个现场固定稳定 `repo_instance_id`、精确 `repo_id`、主机与站点身份、Git 公共目录身份，以及首次校验的 Git 证据；Git 工作树、ChangeSet 物化、工具箱锁与本机运行时都通过该现场引用。现场的 `site_generation` 与工具箱持有的现场 OS 锁由[系统边界](./system.md#单写者)统一定义。

「挂接 Repo Instance」命令先由工具箱无副作用读取 Git 身份，再由 control 预览并写入账本。相同 Git 公共目录的重试返回原现场；不同现场只有在 Git 中的稳定 Repo 身份与命令指定的 `repo_id` 一致时才能挂接。远端 URL、目录名或碰巧相同的 HEAD 只作辅助证据。

身份缺失、分支来源语义不明、一个公共目录已归属另一 Repo，或证据互相冲突时，系统不得静默挂接。界面展示全部证据；用户明确选择挂接到指定 Repo、注册新 Repo 或修复来源后，系统才按该选择继续。移除现场只撤销其新执行资格，不删除 Repo、Project、历史 Run 或已封存 ChangeSet。

## ChangeSet 与 Git 事实

Git 工作树是 ChangeSet 的可替换物理资源，不永久属于 Project、Task、Room 或 Harness。一个 ChangeSet 同时最多有一个有效写入租约；候选切换、接管或取消必须先让旧写入者失权。失权由两个模块各做自己的动作：本模块撤销租约并拒绝重授；[Participant 模块](./participant.md#changeset-与-git-事实)停止或隔离旧执行并提供证据。两边不各建一份写租约。

旧写入者无法证明已经失权时，control 默认保全并隔离原 Git 工作树和 ChangeSet，不授予新租约。有权 human actor 可以在预览残留后选择接管、封存、采用到另一 ChangeSet 或丢弃。只有自动恢复必须从获准基线创建新的 Git 工作树和 ChangeSet。系统不能把来源未知的未封存字节、旧租约或旧生产者身份自动带入新执行。

ChangeSet Revision 在有效租约下封存，至少固定：

```text
change_set_revision_id
+ change_set_id
+ parent_revision_id?
+ base_commit_sha
+ result_tree_sha
+ producer_ref             # human command，或精确 Invocation + invocation_version / Attempt + attempt_generation
+ revision_digest
```

封存与准入是两件事：工具箱按有效租约把工作树内容封存成 Git 字节并回读，这是保存；账本接受这个版本，这是准入。

由执行结果提案产生的版本，顺序固定为：工具箱按提案的 ChangeSet 输出封存并回读；control 复核归属者状态、代次与租约仍然有效；Project 或 Run 准入 Result Proposal 的同一账本事务里，本模块准入 ChangeSet Revision。提案中的 ChangeSet 输出至少固定：ChangeSet 的稳定 ID、所持 Write Lease 引用、声明的基线提交，以及结果的位置——执行体分支上的提交，或工作树本身。工具箱以此为封存输入：结果是提交时校验基线与祖先关系并取其树，结果是工作树时先封存成 Git 字节；回读 base_commit_sha 与 result_tree_sha，生成 change_set_revision_id 与 revision_digest，交给准入事务。封存的 Git 写入在账本事务之外（不同原子域）：封存意图以提案标识符与 producer sequence 为关联键，重试返回同一封存结果；工具箱已经写出的树或提交不是获准版本，没有归属者准入的封存只留审计。封存期间被取消或替代的归属者，其 Git 对象即使已经存在也不得成为获准版本；平台上出现一个提交也不是准入。

有权 human actor 的显式封存——预览残留后接管、封存或采用，producer_ref 为 human command——不经 Invocation 或 Attempt：本模块按该 human 命令准入版本，不伪造一次调用或一份 Result Proposal。

评审对象对 {change_set_revision_id, change_set_id, parent_revision_id?, base_commit_sha, result_tree_sha} 使用[共享摘要规则](./system.md#命令与跨服务正确性)生成独立 review_subject_digest；它不是完整 ChangeSet Revision 的 revision_digest。评审身份由这五个字段决定：同一获准版本只换提交包装（内容与基线相同、提交对象不同）时身份不变；基线或结果树任一变化都是新 Revision，旧 Verdict 按 [Run 约束](./run.md#request重试与-gate)失效。基线不同而结果树碰巧相同，也是新 Revision——结果树相同不是同一版本的充分条件。

`result_commit_sha` 只存在于后续集成与平台证据，不属于 ChangeSet Revision，因此给同一 Revision 增加不同提交包装不会改变其评审身份。

返工或结果树变化创建新 Revision，旧 Revision 不改写。producer_ref 不进入 review_subject_digest，但作者与评审者分离必须沿它解析并校验当前逻辑身份。

Harness 可以操作自己的 Git 工作树，但改写目标引用不产生 Receipt。下一次预览或回读只会显示分歧——预期目标头形态下是预期目标头不匹配，接受目标前移形态下是目标保护快照或回读核对不符——由有权 actor 对账处理。工具箱校验 Git 基线、HEAD、tree、祖先关系与目标分支头，也回读评审请求当前头、检查、线程与正式评审状态、保护条件这些平台机械事实；评审评论正文由平台适配器为代取读取。

失败、取消、租约撤销和资源清理都不等于放弃代码。物理清理默认保全：工具箱先确认所有已跟踪、未跟踪且尚未封存的修改已有可恢复副本，现场资源得到该确认才可拆除；有权 human actor 在预览残留后显式确认丢弃时，可以不留副本直接拆除。保全或封存失败且未获显式丢弃确认时，保留精确 Git 工作树路径、Git 状态和显式恢复动作，不能删除唯一副本。清理 Git 工作树也不删除领域历史。

## 集成：目标、两个头与两种授权形态

模型自述不能证明集成成功。control 先持久化「合入 ChangeSet」意图和 outbox，再由执行者执行并回读；成功回读后，control 才写唯一 Integration Receipt。命令必须固定 ChangeSet Revision、来源与基线、目标、所选授权形态、目标保护快照、合并方式、适用 Verdict 和证据、actor 与权限、绑定版本和幂等键。

**目标按种类固定。** 目标是某个 Repo Instance 上的本地 ref，或平台端口所绑定平台上的远端 ref；意图固定目标种类、ref 名与所用绑定版本。面向本地目标的意图由工具箱以比较并交换推进 ref；面向远端目标的意图由平台适配器请求平台合并。同名的本地分支与远端分支是两个目标，对本地 ref 的集成不得表现为远端 ref 已合入。

**两个头。** 源头是待合入的提交，即 ChangeSet Revision 对应的提交；目标头是预览时目标 ref 所在的提交。执行时源头必须匹配，否则拒绝。目标头的处理取两种授权形态之一，由有权 human actor 在预览时显式选择并冻结进意图：

1. **预期目标头形态**：意图固定预期目标头；执行时目标头不等于预期值即拒绝，不重试。面向本地目标的意图总能采用它；面向远端目标的意图只有在绑定声明「能保证预期目标头」时才能采用它。
2. **接受目标前移形态**：意图固定精确源版本、合并方式与目标保护快照，接受执行期间目标前移；平台按其保护条件合并，回读只负责核对不负责扩权；Receipt 记实际的目标头。

绑定声明「不能保证预期目标头」而 actor 未显式选择第二种形态时，集成意图拒绝。放行不是同一条命令的执行时降级，也不由回读结果倒推。第二种形态存在的依据是：已调研的代码协作平台都只校验源头，没有一家提供预期目标头的比较并交换，快进策略也只保证祖先关系而不保证目标未动，合并队列的语义是接受目标前移后重新验证；证据与各家对照见[代码协作平台市场调研](../../research/scm-platforms.md)。

**目标保护快照冻结进意图。** 预览时回读的保护条件作为快照冻结；执行与回读时对照。声明的检查、线程或批准条件与快照不一致时，命令拒绝或标为需要关注，不凭「同步开关为真」或「几个检查为绿」通过。上游在预览之后修改保护规则不改写已持久化的意图。

**本地目标正被检出时的正常入口。** 目标 ref 正被某个 Git 工作树检出时，工具箱默认拒绝，并返回精确的工作树路径与恢复动作；正式走法是人先把该工作树切离目标 ref，再重试同一意图。显式放行只推进 ref、不更新该工作树的文件与索引，是运维例外，不是正常路径。

**已接受不等于已合入。** 平台已开启自动合并、已进合并队列或返回「已接受」都只是请求被接受；Receipt 只在回读到合并提交与目标头之后签发。合并后的回读不能撤销已经发生的、超出授权的远端写入。

结果未知时，工具箱回读 Git 事实与评审请求的合并状态、合并提交与目标头；收敛前不得签发成功 Receipt 或清理现场。远端写入被中断时同样按结果未知处理，并返回类型化恢复动作。

## 平台绑定与能力声明

平台连接分两层，不得混为一个名字。仓库一级是平台端口的 Port–Provider Binding：这个 Repo 有没有平台、哪个平台、平台上的仓库身份、凭据引用、账号映射、目标 ref 的保护条件回读方式与实测能力。版本一级是 ChangeSet–Platform Binding：某个冻结版本当时对应平台上的哪个提交、哪条评审请求。

绑定至少声明以下能力：评审线程、正式评审状态、检查、远端合入、身份映射、能否保证预期目标头、评审评论正文回读、目标保护条件回读。能力声明在两个时刻起作用：采纳契约与预览集成时，control 据此判断契约里的验收项能不能由这个仓库的供应端满足；执行时，缺了声明过的能力，命令等待、拒绝或标为需要关注。能力声明不改变已冻结契约的前置：契约要求平台检查通过时，本地测试事实不能顶替；契约要求合入远端 ref 时，本地 ref 前移不能顶替。这是[组件不可用不改变命令前置条件](./system.md#命令与跨服务正确性)在本模块的应用。

没有平台绑定的 Repo 是完整的正常路径：契约在采纳时不得要求平台上才有的机械证据；机械验收项以工具箱回读的本地事实为证据；集成只面向本地目标。之后绑定平台只是新增一份绑定与一类目标；Repo 身份、既有 ChangeSet Revision 与 Receipt 不变，进行中的意图沿用原绑定版本。

## 发布评审

发布评审是本模块执行的持久外部副作用命令，其授权来源是归属者的 Execution Spec 冻结的**评审发布策略**：Room Invocation 由 [Project 约束](./project.md#room-invocation)在 Trigger Preview 冻结，Attempt 由 Run Manifest 与 Execution Spec 冻结。策略至少固定：Repo、平台端口的 Port–Provider Binding 版本、发布到的分支或评审请求的规则、允许创建还是也允许更新、评审请求描述的来源（Result Proposal 中被允许的文本产出）、是否须人显式确认。

真正发布时再固定精确 ChangeSet Revision 与描述摘要。Result Proposal 只能提供策略允许的内容；换发布地点、换绑定或扩大发布范围都不在授权内，必须回到归属者重新预览与授权。策略中的「须人显式确认」开关随本次授权冻结，仓库或 Project 之后改默认值不影响已接受的调用。

「发布评审」命令的提交者是 control，不是执行体，也不是第三种 actor 来源：它是授权它的那次 human 提交——Room Invocation 的 Trigger Preview，或 Run 的启动预览经 Run Manifest 冻结进 Execution Spec——的后续动作，与「注册确认后创建 Repo Room」同类。control 在归属者准入提案与本模块准入版本的同一账本事务里持久化发布意图与 outbox，actor 信封沿用那次 human 提交与冻结策略的引用；客户端是否在线不影响它。开关「须人显式确认」打开时，意图改为待处理，由有权 human 预览后提交。Attempt 归属的版本同理：发布 outbox 挂在 Run 准入提案的事务上，席位不自行推送远端。

发布评审的执行者只有平台适配器；执行体不持有平台写凭据，不能自行推送远端。推送成功而创建评审请求的确认丢失，或更新评审请求后确认丢失时，按原意图与关联键回读，不重复创建；旧 Revision 的迟到重试不能覆盖新 Revision 的分支或映射。

## 变更与平台的映射

ChangeSet–Platform Binding 记两类东西。证据部分——某个冻结 ChangeSet Revision 对应平台上哪个提交、哪条评审请求——在发布评审、更新评审请求与集成回读时写入，冻结后不改写。状态部分——评审请求的当前头、检查结果、线程是否解决、正式评审状态、合并状态——是回读事实，随时变，只用于预览、准入前核对与投影。

评审请求的头变化时，工具箱重算基线与结果树，按[ChangeSet 与 Git 事实](#changeset-与-git-事实)判定是否为新 Revision；新 Revision 由归属者准入后才映射，平台上的提交自身不产生获准版本。一个 ChangeSet 对应几条评审请求、一条评审请求能否换 ChangeSet，见[交付文档的未决问题](../delivery.md#未决问题)；本约束不默认一对一。

评审评论正文是代取来源：组装器按精确 ChangeSet Revision 与评论标识冻结进 Context Manifest，规则见 [Project 约束](./project.md#context-memo-artifact)。评论进入上下文只供作者阅读，不构成授权、契约或裁决。

## 平台动作与命令

平台事件先按[系统边界的动作分类](./system.md#客户端动作与-provider-事件)入账。本模块逐项规定各类平台动作的去向；控制面自己写回产生的事件一律排除，不再形成人的请求：

| 平台动作 | 去向 |
| --- | --- |
| 普通评论、评论线程解决 | content；经代取进入上下文，不成为命令 |
| 批准、请求修改、正式评审状态 | 外部评审证据（Snapshot），能证明什么由验收契约定；账号映射只证明来源，不把它变成 Gate 的一票；HCTL 席位的投票走 Result Proposal |
| 检查结果 | 外部机械事实，由工具箱回读；只对当前候选版本的提交计数 |
| 合并按钮、「已合并」通知 | 不是事前授权。有匹配的持久集成意图时，该事件是该意图的回读事实之一，不构成新的意图，也不改变所选授权形态；没有时只作外部事实与分歧记录，不倒补意图，不补签 Receipt |
| 评审请求被改投另一目标 ref、被关闭或重开 | 只更新映射的状态部分并标为需要关注；不对错误目标写 Receipt |
| 直接改写受保护 ref 或绑定的评审请求分支 | 不支持的供应端修改：记录当前机械事实并标记分歧 |

映射到有权用户本人的平台动作要成为 human 命令请求，须满足系统边界要求的 actor、目标、前后版本与幂等依据，且绑定明确列出该动作；本约束当前不列出任何这样的平台动作：平台上的批准、评论与合并按钮都不能提交合入或发布。合入由 Workbench/CLI 的直接客户端连接，或按冻结规则推进 Run 的归约器提交；发布由 control 在版本获准后按已冻结的评审发布策略发出（见[发布评审](#发布评审)），开关打开时才由人再预览一次。同一平台账号发出的多次批准映射不出多个席位；参与者是否各有平台身份由交付文档决定，不为此新建账号系统。

## 恢复

恢复按目标分，状态按是否可能已写分：

- 面向本地目标的意图，确认丢失时只回读它的本地目标与同一意图的 Git 证据，不等待任何评审请求；
- 面向远端目标的意图，确定尚未投递的仍是待启动，可以照原意图重投；已经可能产生远端写入而无法确认的才是结果未知，保持未知并继续占用冲突范围，即使本地已有同一结果树也不解锁；两种状态都不能改道，也不能签成功 Receipt；
- 发布评审意图同样按此处理；确认丢失后按关联键回读评审请求，不重复创建。

代码协作平台不可用时，依赖平台当前回读的命令安全拒绝：远端合入、读评审请求状态、发布评审；不依赖它的操作继续：本地物化、封存、面向本地目标的集成。全系统事实权威地图中的对应行见[系统边界](./system.md#全系统事实权威地图)。Receipt 的权威在账本，Git 里只有审计影子；平台丢失后不得凭 Git 提交重建 Receipt。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL | 外部体系 | 差异一句话 |
| --- | --- | --- |
| Repo | GitHub repository、GitLab project、Gerrit project | 逻辑身份写在 Git 里并记账，不随平台上的仓库改名或迁移漂移 |
| Repo Instance | clone、worktree 所在的 Git 公共目录 | 物理现场，可丢失、可重挂接；不承载 Project 身份 |
| ChangeSet | PR 的源分支、MR 的源分支、Gerrit change | 写入边界与租约在账本，分支只是载体 |
| ChangeSet Revision | PR head commit、Gerrit patchset | 身份按基线与结果树算，换提交包装不换身份 |
| 集成意图 | 平台的 merge 请求、命令行的 merge 子命令、Gerrit submit | 先持久意图后执行；平台一般只校验源头，预期目标头保证靠授权形态而不靠平台 |
| Integration Receipt | merge commit 与 merged 状态 | 凭证只在回读到合并提交与目标头后签发，权威在账本 |
| 目标保护快照 | branch protection、rulesets、merge queue 条件 | 预览时冻结进意图，执行与回读时对照 |
| 平台账号映射 | 平台用户与机器人账号 | 只证明来源，不授予票权或命令权 |
| Write Lease | 无对应 | 差异化语义：单写入者，配代次失权 |

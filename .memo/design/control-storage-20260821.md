# 控制面存储怎么做——五储对照与判据（2026-08-21）

> 状态：待拍板：五储对照总表是否成文<br>
> 基线：main @ f7357de（草案 v0.10.2）<br>
> 去向：若成文：spec/system.md「事实与存储」<br>
> 说明：本轮讨论的结论沉淀 + 所有者要的五储对照总表（草案，是否成文进 doc 待裁定）。
> 关联：[scene-data-model-20260820](./scene-data-model-20260820.md)、`docs/design/architecture.md`（三面架构 / 4×3 矩阵）、`docs/design/spec/system.md`（事实与存储）、`docs/design/delivery.md`（选型与限时验证）。
> 背景：本轮讨论与 v0.10.x 大修（另一会话落地并推 main）并行发生。下文逐条标注：哪些结论 v0.10.2 已落地（本轮独立推演相互印证）、哪些是新增输入待裁定。

## 一、本轮结论

1. **跨系统边规则**（新增，建议成文）：控制面账本只存**跨系统的边**——身份、锚定/绑定、ID 映射；承载系统内部的边留在该系统里，控制面只管写入规则。repo→project→task 从属关系的 content 表达整棵住在任务后端（Board→分组实体→卡片），账本存的是身份映射与逐字段权威（TaskBinding），不做树的第二副本。task→subtask 不是 HCTL 概念（已落地：子任务归后端原生能力）。跨系统的边有个天然优势：机制上只有 HCTL 造得出来（kanban 客户端造不出 Matrix room），不需要额外防护；需要费心管控的只有系统内部的边。

2. **改管辖走治理**（新增原则；机制已落地）：改变管辖范围的写操作（挪动/删除从属、变更契约相关字段）走治理路径；范围内的日常写操作（建卡、评论、排序）场景自便。这条原则能推导出"kanban 对从属关系只能 CR、不能 UD"，不用硬记 CRUD 表。执行手段是**对账，不是三方 ACL**——第三方客户端直连后端，后端权限模型（如 Vikunja 的 project 级 read/write/admin）表达不了这种粒度；v0.10.2 的落地机制正是这条原则的实现：**身份全量 + 契约惰性 + 先观测后采纳（TaskSourceSnapshot，会改契约的内容必须经用户采纳）**。场景里先建出内容、HCTL 采纳时补上身份与锚定——"先有内容，后有锚定"。

3. **名册与阵容分开**（新增措辞；机制已落地）：
   - **名册** = Participant 逻辑档案：是谁、怎么够到（harness 端点/模型配置）、被授权做什么。跨场景身份，权威在账本（已落地："Participant 是逻辑档案，外部账号只是映射之一"）。
   - **阵容** = 某个 Room/Board/任务上的在场与角色：用各场景**原生概念**承载——m.room.member、assignee/label。这是 content/记录，"命令走 HCTL、记录落平台"。
   - **特化** = 从名册选人组阵容（repo room 的名册 → project/task 层选子集）：裁决进账本，阵容记录落平台。
   - 曾考虑更激进方案：repo room 的 Matrix state event 直接做名册权威存储（HCTL 独占高 power level 写入）。不采，理由：名册是跨场景的（kanban/workflow/terminal 都要解析身份），不能让所有场景依赖 chat server 存活；换聊天后端时名册陪葬；凭证/端点绝不能进房间状态（成员可见、随联邦复制）。Matrix 化运营成熟后可重议。

4. **控制面存储形态已收敛**（v0.10.2 已落地；本轮独立推演得到同构结论，相互印证）：一本**用户级 metadata 账本**（`~/.hctl2/control.sqlite`，单写者 + CAS/代次 + outbox）+ 用户级定义文件（Harness/Profile/Skill/Runtime 的不可变 revision）+ **OS secret store**（密钥不进账本、Git、Room、Context）；clone 本地 `<git-common-dir>/hctl2/` 只有 OS 锁与可丢弃缓存，**不是账本**。判决双层保管：权威在账本，结晶副本进 Git，可部分回灌但不能伪造未结晶判决。通俗形态：**接线板（绑定）+ 公证处（判决）+ 保险箱（密钥）**，数据的家在四个场景系统和 Git，控制面不是数据枢纽。

5. **勘误两条**：
   - 任务后端选型（Vikunja 首选，git-bug 并列对照，五项限时验证）**早已记录**于 `delivery.md` 与 `implementation-evidence.md`（E-L3-VIKUNJA）。此前会话里"没有记录"的说法，是在未拉取远端更新的旧树上搜索所致。
   - 此前讨论中"Repo↔Vikunja 顶层 project、Project↔子 project 嵌套"的映射，被落地的「**一个 Repo 一个 Board；Project = 后端分组实体；Task = 卡片**」取代；分组实体由适配器按能力声明（父任务 / milestone / Linear project，能力不足降级为标签或过滤视图），Vikunja 的具体承载随限时验证定夺。

## 二、五储对照总表

所有者提出的梳理方法：hctl-control 列自己的概念；四个执行面系统以 HCTL 概念为主、映射到该系统的原生概念。表 2–5 的映射均取自各模块合同的「外部概念对齐」表（spec/project.md、task.md、run.md、agent.md），此处按"存储视角"重组。

### 表 1 · hctl-control（全部是 HCTL 自有概念，无外部映射）

| 族 | 存什么（例） |
| --- | --- |
| 身份 | Repo / Project / Task / Run / Room / Request / Participant 的身份与从属（Room 归属 Project；Task 身份映射的稳定键） |
| 绑定（锚定） | ResolvedPortBinding（chat / task_source 端口）、TaskBinding（placement + 逐字段权威）、EngineDeployment、EngineExecutionBinding、桥接配置、clone 现场的实例注册 |
| 授权 | Participant 名册与角色绑定、Run 授权/候选/预算、Gate 规则 |
| 租约与现场记账 | WriteLease、TerminalInputLease、writer/backend 代次、worktree/ChangeSet 归属、观测账 |
| 判决 | Verdict / Receipt、冻结契约权威（TaskRevision / WorkflowRevision）、TaskCompletionReceipt、凭证链、快照采纳记录、消息升格记录；对 Git 内容只存 admission、current pointer、lifecycle 投影 |
| 旁支·用户级定义 | `~/.hctl2/` 的 Harness / Profile / Skill / Runtime 定义（不可变 revision/digest + current pointer） |
| 旁支·密钥 | OS secret store，独立于账本 |

### 表 2 · chat server（Matrix 协议；Tuwunel / Continuwuity 候选）

| HCTL 概念 | 系统原生概念 | 存的信息 |
| --- | --- | --- |
| Room 的 content 家 | Matrix room | 聊天两级：一个 Repo 一个 Repo Room、一个 Project 一个 Project Room |
| RoomEvent / 消息 | Matrix event | 聊天记录、调用过程与结果卡；编辑/撤回是新事件 |
| Scoped Room | thread / 子频道 | 带冻结讨论目标的限定讨论 |
| 阵容（房间在场） | m.room.member | 谁在场、什么成员关系；名册权威在账本 |
| 发言权投影 | power levels | 粗粒度近似，不承载治理 |
| Participant 聊天化身 | Matrix user | 外部账号只是名册的映射之一 |

### 表 3 · task backend（Vikunja 首选；GitHub / Linear 远端直访）

| HCTL 概念 | 系统原生概念 | 存的信息 |
| --- | --- | --- |
| Board（Repo 任务容器） | Vikunja project / GitHub ProjectV2 / Linear 空间 | 一个 Repo 一个 Board，注册仓库时选定后端 |
| Project 在板上的分组 | 分组实体：父任务 / milestone / Linear project；降级为标签/过滤视图 | 由适配器能力声明 |
| Task | 任务卡 / issue | 卡片 content 本体；身份、契约、验收在 HCTL |
| stage | workflow state / ProjectV2 status | 字段权威由 TaskBinding 逐字段决定 |
| 排序（rank） | sortOrder / 位置令牌 | 条件写入用后端并发令牌，无则降级 |
| 阵容（任务在场） | assignee / label | |
| 子任务、清单、微卡 | 后端原生能力 | 不入 HCTL 概念 |
| 观测入口 | webhook / API payload | TaskSourceSnapshot 来源；先观测后采纳 |

### 表 4 · workflow engine（conductor-oss）

| HCTL 概念 | 系统原生概念 | 存的信息 |
| --- | --- | --- |
| WorkflowRevision 的注册形态 | versioned workflow definition | 引擎产物不能反向定义它 |
| Run 的机械执行 | workflow execution | 令牌位置、重试、定时器、机械执行历史 |
| 步骤领取/完成 | worker task（SIMPLE，poll/complete） | 只经 control 领取与完成 |
| 恢复关联 | execution id + correlation key | 只作绑定与恢复，不成为 Run 身份 |
| Obligation / Seat / Verdict / Gate Receipt | **无对应** | 留在账本 |

### 表 5 · harness 与运行时（agentd 托管；Codex / Claude Code 等）

| HCTL 概念 | 系统原生概念 | 存的信息 |
| --- | --- | --- |
| Terminal 会话 content | harness 会话转录、PTY 流 | agentd 持有与观测（它之于 Terminal 如 chat server 之于 Chat Room） |
| ExecutionRuntime | tmux session/pane + 进程 | 可丢弃、重建、接管；不承载领域身份 |
| 结构化接入 | ACP 会话 | 协议会话不是 HCTL 身份 |
| semantic resume | harness 原生 resume | 恢复上下文，不等于 exact attach |
| ChangeSet 物理形态 | worktree diff / commit | 结晶进 Git |
| WriteLease / TerminalInputLease | **无对应** | 留在账本 |

## 三、判据：从对照表反推账本清单

把四张对齐表倒过来读，得到一条干净的判据：

> **凡外部体系「无对应」的概念，就是控制面账本的存储清单；凡外部有原生概念的，content 归它，账本最多存绑定与摘要。**

验证：四张对齐表里的全部「无对应」行——Request、TaskCompletionReceipt、Obligation / Seat / Verdict / Gate Receipt、WriteLease / TerminalInputLease——一个不落，全部落在表 1 的五族里。这条判据同时是未来引入新概念的守门规则：想进账本，先证明外部体系确实没有它。

于是「hctl-control 的存储该怎么做」收敛为（均已落地或与落地合同一致）：

- **内容**：五族（身份 / 绑定 / 授权 / 租约记账 / 判决）+ 用户级定义文件 + secret store，仅此而已；
- **形态**：单文件 SQLite、单写者、CAS/代次、outbox——小而事务性的注册表，契合"本机部署尽量轻"；
- **韧性**：账本是唯一不可再生权威，必须备份；Git 结晶副本部分回灌 + content 系统游标对账重建现场，clone 本地目录随删随弃。

## 四、遗留待裁定

1. 三条新增表述（跨系统边规则、改管辖走治理、名册/阵容/特化）是否成文进 doc；若进，候选去处：`spec/README.md` 外部对齐原则旁（判据）+ `architecture.md`（名册/阵容的产品语言）。
2. 本表（五储对照）是否作为 `architecture.md` 附录或留在 memo。
3. 阵容投影的对账（房间成员与账本名册漂移时的收口）是否已有合同覆盖，待核对 `spec/project.md` 失败与恢复条款。

（进展：第三节判据的表述已获所有者认可，去处与其余成文项一起定。）

## 五、存储私事原则（同日第二轮）

所有者结论，经推敲成立并采纳：**这些存储都是 hctl-control 的私事，不用放 git——本地存储、不进 git 的东西只与当前服务器有关，别的服务器无需知晓也无需继承。**七族数据怎么建表因此不进设计文档，实现时临场决定。

- **澄清**：「无需知晓」针对存储而非事实。账本是试金石——一人多机需要的是其中的**事实**，经控制面服务接口获取；没有谁需要知道那个 SQLite 文件的路径、格式、表结构。场景服务器同理。这条是「架构是 client-server 的」在存储层的等价表述：事实经服务流通，存储是每台服务器的私事。
- **时间维度脚注**：「无需继承」对空间成立、对时间不成立——不可再生的账本必须由**同一服务器的下一任**传承（备份、writer 搬迁，合同已有）；可丢的（锁、缓存、游标、收口后的观测）则无保留成立。
- **反向刀法**：要不要进 git = 是否要让「拿到 repo 的人」不依赖任何在线服务器就能继承它。决议/Memo/代码/结晶副本/policies 进 git ✔；任务卡/聊天记录经服务访问 ✔；密钥恰恰**不许**被继承，git 与账本备份都不进 ✔。这是 4×3 的「为什么住那」层。
- **撤回记录**：此前第二轮讨论中提议的六条存储「纪律」（分区修剪、content-addressed 统一表、定义按 digest 入账、权威/缓存分文件、备份机制、凭证链预留）全部撤回——实质已被合同覆盖（账本唯一权威、必须备份、结晶可回灌、其余可重建），其余属实现期决定。教训：不立「临时想也来得及」的小纪律，否则纪律=五百句废话。

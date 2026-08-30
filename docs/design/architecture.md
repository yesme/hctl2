# 三面架构

> 状态：规范性（架构层）· 草案 v0.15.2<br>
> 日期：2026-08-31<br>
> 定位：本文回答部署与数据视角——系统由哪三个面组成，每个场景的数据分哪三类、住在哪里、不可用或丢失时怎么办。模块的语义分责见[设计地图](./README.md)；对象、状态机与三类数据的权威定义在[合同层](./spec/README.md)；具体实现选型与验证在[交付文档](./delivery.md)。

## 三个面

四个领域模块（Project、Task、Run、Agent）是**语义分责**；部署视角上，系统由三个面组成。两者正交：每个模块的 metadata（治理元数据）都在控制面，content（场景内容）都在执行面的对应系统，场景都由展示面呈现。

| 面 | 组成 | 拥有什么 |
| --- | --- | --- |
| 展示面 | Workbench、CLI 与第三方场景客户端 | 不因客户端身份拥有事实或特权；按动作目标查询/提交 HCTL 命令，或读写 provider content 与精确运行时 |
| 控制面 | HCTL 自己的命令服务与账本（合同层组件 `hctl2-control` / `hctl2-tool`） | 全部 metadata：身份、绑定、授权、判决 |
| 执行面 | 四个场景的 content 系统与物理执行；Agent / Terminal 第一阶段由 Herdr 承载 | 全部 content 与机械状态；接收 provider 自有内容/运行时动作，也执行控制面按顺序发出的副作用 |

控制面归**用户级**：一人多机连的是同一个控制面，仓库 clone 只是代码侧的物理现场。第一阶段单机部署时三个面同装一台机器——这是 local-first 的默认部署形态，不改变 client-server 实质。

Workbench 把四类 provider 客户端、跨模块导航、联合投影和 HCTL 公共命令入口组合成一个产品桌面，但没有额外权限。消息、卡片和终端输入走各模块的 provider 合同，治理命令与 CLI 共用 command service；动作分类与接纳规则见[系统边界](./spec/system.md#客户端动作与-provider-事件)。Workbench 关闭后，领域服务和执行照常继续。

产品仍是普通的“打开仓库”：打开本地 repo 会连接（必要时拉起）本机控制面，打开远端 repo 会连接远程控制面并选择其名下仓库；终端票据也可以把观察者带到另一个执行现场。一套控制面因此可以服务多个客户端和执行现场。第一阶段只交付本机连接，远程认证与传输仍属[交付文档的未决问题](./delivery.md#未决问题)。

执行面的各系统是独立进程或服务器，不内嵌进控制面。“轻量”指轻量的实现选择、低资源占用，以及随 HCTL 一键启停的生命周期托管，不是把服务塞进控制面进程。

## 场景与系统

场景是用户视角的名字，系统是 content 承载者的角色名；每个场景由一个专职系统承载它的 content。角色名是产品词，具体采用哪个实现由[交付文档](./delivery.md)的选型判据与限时验证决定。

| 场景 | 系统角色 | 系统拥有的 content | 备注 |
| --- | --- | --- | --- |
| Chat Room | chat server（聊天服务器） | 聊天记录、调用过程与结果卡 | 采用 Matrix 协议；Matrix 生态客户端可直接访问；HCTL 房间不开端到端加密，准入与降级见[Project 合同](./spec/project.md#room-与消息) |
| Kanban | task backend（任务后端） | 任务卡、流转、排序、评论 | 注册仓库时选择：本地任务服务器，或 GitHub/Linear 这类远端平台直访；一个 Repo 一个 Board |
| Workflow | workflow engine（工作流引擎） | 令牌位置、重试、定时器、机械执行历史 | 引擎只拥有机械状态，不拥有语义 |
| Terminal | harness（编码代理工具）与 Agency | 会话转录、PTY 流 | Agency 之于 Terminal，如同 chat server 之于 Chat Room。第一阶段直接采用 Herdr：它按规格启动 Harness，持有进程、PTY 和终端会话，并提供 API 与原生 TUI。HCTL 不再提供另一套第一方终端运行服务 |

## 避免供应商锁定

Tuwunel、Vikunja、Dagu 和 Herdr 是第一阶段默认实现，不是 HCTL 的产品合同。HCTL 不再增加一个通吃四个模块的独立 shim（垫片服务）；四个模块的语义不同，把它们压进同一套通用接口只会形成新的私有协议。稳定边界放在每个模块自己的受控端口，具体产品由薄适配代码接入：

| 场景 | 稳定边界 | 第一阶段默认实现 | 后续替换方式 |
| --- | --- | --- | --- |
| Chat Room | Matrix 协议 + HCTL 的 Chat 端口绑定 | Tuwunel | 可换其他 Matrix homeserver；飞书、Slack、Discord 等非 Matrix 平台由 homeserver/bridge 生态接入，HCTL 不逐个平台写聊天适配器 |
| Kanban | HCTL task backend 端口 | Vikunja | GitHub、Linear 等各写一个 task backend 适配器，共用 Task 身份、字段权威、Snapshot 与命令合同 |
| Workflow | HCTL 的 Workflow Revision 中间表示 + workflow engine 编译/回读端口 | Dagu | 为新引擎增加编译器和回读适配器；Run、Gate、Obligation 与完成判定不随引擎改变 |
| Terminal | HCTL Agency 端口 + 客户端侧终端 transport adapter | Herdr | 官方远程 Agent 可以直接实现 Agency 合同，或由专用适配器接入；执行授权、身份、租约、证据与恢复等级不随 Agency 改变 |

Workbench 是四个场景的稳定组合界面，但只使用公开合同：HCTL 命令与 CLI 同路，content 和运行时动作与对应原生客户端同路。第三方私有对象模型不进入模块合同。Terminal 的字节流和绘制性能敏感，Workbench 可以通过客户端侧 transport adapter 直连精确目标；输入租约、记录与恢复保证按 binding 声明的能力如实标注。

“可替换”分三档承诺，不能混为一谈：新工作可以在通过合同测试后选择另一 provider；活动执行继续使用冻结的 binding，不热切换；既有 content 能否迁移取决于两端导入导出能力，需要单独预览和校验。HCTL 自己的 metadata、不可变引用和 Git 结晶不依赖默认实现，但这不等于所有第三方 content 都能无损搬家。

## 4×3 归属矩阵

行是场景，列是[三类数据](./spec/README.md#三类数据)；每格回答“什么数据、存在哪”。

| 场景 | metadata（控制面账本） | content（执行面系统） | artifact（Git 结晶） |
| --- | --- | --- | --- |
| Chat Room | Room 身份、归属 Project、Participant 名册与角色绑定、身份映射配置、消息升格记录 | 聊天记录、调用过程与结果卡（chat server） | 决议、Memo、施工图 |
| Kanban | Task 身份映射、字段权威绑定、冻结契约及其摘要、完成凭证 | 任务卡、流转、排序、评论（所选任务后端） | 冻结的任务契约 |
| Workflow | Run 授权、引擎绑定、代次、Gate 规则、裁决 | 令牌位置、重试、定时器、机械执行历史（workflow engine） | 凭证链 |
| Terminal | 执行授权与派发规格、写租约、输入租约、代次、观测账 | 会话转录、PTY 流（harness 会话 / Agency） | ChangeSet 与合入的代码变更 |

矩阵里的 artifact 是一种**解释性结晶规律**：重要结果通常会形成可审阅、可分发的 Git 工件；它不是把 metadata 或 content 逐字节变换成 Git 文件的存储定律。结晶的归属以事实为准绳——它从哪个场景长出来就归哪一格（施工图从 Room 的塑形讨论中长出，故归 Chat Room），不为对称硬填。对于合同明确以 Git 为 home 的不可变正文，Git 保存正文，控制面仍独占其身份、准入、摘要、current pointer（当前指针）与裁决；Git 中出现一份正文或副本本身不能证明它已经被 HCTL 接纳。Receipt、绑定、授权和 lifecycle 等以合同层标定的 metadata 仍以控制面账本为权威。

统一律与三条法（能承载不等于能裁决；冻结摘要是防火墙；命令走 HCTL、记录落平台）见[合同层总则](./spec/README.md#三类数据)，此处不重复。

content 容器的层级随场景各得其所：聊天两级——一个 Repo 一个 Repo Room，一个 Project 一个 Project Room；看板一级——一个 Repo 一个 Board，Project 是板上的分组，Task 是卡片，子任务归后端原生能力；机械执行历史与执行会话随各自的 Run 与执行归属。容器归属 Repo/Project 身份，不归属某个 clone 或客户端。

## 模块交接

模块之间的每次交接都交付一个**冻结的、带摘要的不可变对象**；下表是[连接合同](./spec/connections.md)总表的产品语言投影，精确字段以合同为准。

| 交接 | 交付物 | 一句话 |
| --- | --- | --- |
| Project → Task | 冻结的任务契约（Task Revision）+ 来源回链 | 讨论经采纳命令升格为承诺 |
| Task / Project → Run | 冻结契约 + 施工图，由施工清单（Run Manifest）冻结引用 | 承诺进入治理；批准施工图与开工是两个动作 |
| Run / Project → Harness | 派发规格（Execution Spec） | 治理派发执行；执行侧照单干活，不做语义判断 |
| 执行回程 | 结果提议（Result Proposal）→ 裁决与凭证 | 执行只能提议；owner 模块机械校验身份、代次、权限与证据后才算数 |

## 数据丢了怎么办

两种情形分开立约：**不可用**（进程或服务暂时失联）保住已接纳事实并标明新鲜度，**永久丢失**（数据没了）按三类数据重建。精确降级与恢复见[系统边界](./spec/system.md)和[连接合同](./spec/connections.md#失败与恢复)。

**不可用——保住已接纳事实，不伪造新鲜度：**

- chat server 宕机不抹掉已经接纳的治理事实；不依赖 Room 新消息、来源链或 fresh Context 的施工可以继续，依赖这些新鲜读数的预览与命令安全暂停；
- 任务后端失联时，看板显示待同步，排队中的操作不显示假成功；依赖 current binding、remote revision、来源 head 或 readback 的采纳、移动与完成命令 fail closed；
- workflow engine 失联时，已冻结的本地事实继续存在；完成与评审都在控制面账本，引擎只是路标，路标停更只待对账，不拦判决；
- 运行时失联时，执行安全暂停，不冒充成功。

后端离线不等于全部治理命令不可用，也不等于全部治理命令照常可用：是否继续由该命令的准入合同是否需要 fresh provider readback 决定。受影响的入口显示待处理 / 需要关注或安全暂停，不绕过命令服务。

**永久丢失——按三类数据分别回答：**

- **metadata**：控制面账本是唯一不可再生的权威，必须有备份；判决的结晶副本进 Git 后可以部分回灌，但回灌不能伪造未结晶的判决。
- **content**：丢失不会抹掉已经接纳的治理事实——已结晶的部分（决议、契约、凭证、代码）存活于 Git，有桥接来源的部分可以重放，丢掉的是尚未结晶的记忆；需要核对 provider 当前事实或重建来源链的命令仍须 fail closed，旧结晶不充当 fresh readback。
- **artifact**：靠 Git 分布式冗余，每个 clone 与 remote 都是备份。
- 物理观测（进程、心跳、屏幕）本就可丢弃重建，不入此列。

# Project 与 Chat Room

> 状态：规范性（架构层） · 草案 v0.15.6<br>
> 日期：2026-09-02

> 本文是 Project 模块的设计正文：它为什么存在、拥有什么、按什么规则运转。精确对象、状态机与写入约束见[约束附录](./spec/project.md)；模块交接见[连接约束](./spec/connections.md)，共享机制见[系统边界](./spec/system.md)。

## 为什么存在

Harness 的会话、终端和 worktree（Git 工作树）都会结束或被替换；Project 的目标、论证、Participant（参与者）关系、来源和未决问题却必须继续存在。Project 模块保存的正是这份长期事实，Chat Room 则是所有 Harness 都消失之后仍然可以恢复的协作现场——它回答“我们要解决什么、为什么、依据是什么，以及哪些讨论已经足够稳定，可以成为承诺”。

以 Room 为中心不等于所有工作都必须聊天：Kanban、Workflow 图和 Terminal 各有自己的场景界面。Room 的特殊地位在于承载目标塑形的连续性（shaping continuity），而不是承载所有机械执行事件。

Project 也不是施工管线：研究、规格说明、ADR（架构决策记录）和纯文档的 Project 可以从未创建 Run。Project 不预配常驻的“包工头”；Participant 是可寻址的逻辑档案，只有显式调用才产生有边界的执行。Participant 的完整设计（[七件事分层](./participant.md#七件事分层)、专业化与评审方法论）见[横切正文](./participant.md)；Context 的完整设计（冻结、传承与成本纪律）见[横切正文](./context.md)。

## 模块拥有什么

Project 模块保存“为什么做、依据是什么、谁在参与”的长期事实：目标与范围，协作现场（Room）的身份、名册与升格记录，Participant 与角色，每次调用可解释的上下文（Context），向人索取输入的 Request（请求卡），沉淀的 Memo（备忘）与登记的 Artifact（工件），以及从聊天室发起的单次调用（Room Invocation）。

消息本体是场景内容（content），住在 chat server。讨论产生的决议、Memo 和施工图进入 Git。Project 不等于仓库、聊天串、Task 集合、Run、Harness 会话或 Git 工作树。

## 关键规则

- 普通聊天形成提案；正式变化走带预览的类型化命令，结构化 human 动作的准入见[场景约束](./spec/project.md#场景约束)。
- 普通 Room 的临场执行边由 human actor 提交；模型 Participant 只建议下一条边。
- Participant、角色、Room 和 Project 使用独立于 Harness 进程与外部账号的稳定身份，换工具不改写已授权执行。
- 上下文保留来源和版本，使每次执行看到的内容可解释。
- Request 只能由获准动作解决，只解锁它声明的阻塞范围；应答面按需升级——默认在卡片或详情里回答，需要多轮论述、多人参与或共同编辑才开临时讨论空间，敏感输入走安全通道，只有诊断或接管才连接终端。无论升到哪一级，都还是同一个请求、同一个阻塞范围。
- 消息只追加，修正、删除和外部编辑保留已被引用的历史。
- HCTL 房间对控制面明文可读；端到端加密的绑定前置、降级与换绑恢复见[Project 约束](./spec/project.md#room-与消息)。
- Memo 经显式提炼、预览并发布后成为长期知识。
- 单次调用适合一次性的研究、比较或范围明确的写入；需要持久重试、候选切换或评审关卡时则创建 [Run](./run.md)。
- 从 Repo Room 提升 Project 时只带显式选中的来源，可删减、补充、去敏；之后的聊天不会偷偷改变既有 Project。
- Project/Room 历史独立于客户端与运行时存活。

## Room 类型

| Room | 作用 | 生命周期 |
| --- | --- | --- |
| Repo Room | 无固定主题的研究、发现和 Project 入口 | 与 Repo 注册同寿命；身份在用户级控制面 |
| Project Room | 围绕一个 Project 的长期协作和里程碑 | Project 归档后只读 |
| Scoped Room | 为复杂 Request 或决定临时建立的讨论空间 | 结论回填或显式结案后归档 |

临时讨论空间创建时必须先说清目标与完成后回填什么；结束时把结论回填，或由有权的人显式结案，然后归档。归档的前置条件只在[约束附录](./spec/project.md#room-与消息)定义。

## Chat Room 场景

Chat Room 是 Project 的主要操作场景，提供：

- 消息顺序由 chat server 的线性时间线统一给出，不靠客户端时间戳或渲染顺序；
- `@` Participant/Role、/ 类型化动作、$ Skill、`#` 文件/Artifact/消息引用；
- 并发 Room Invocation 的独立流、取消和结果卡；
- Request、Project 概览、Task/Run 里程碑和需要关注投影；
- mention 提交前的 Trigger Preview：发出前先看清楚谁来执行、带什么上下文、有什么权限和预算、会创建什么；
- Context 预览、Memo/Artifact 发布预览和权限说明。

在 Workbench 里同时管理多个仓库时，一个 Room 可以把另一个仓库 Room 的 Participant 阵容借用为预填选择，不必逐个重选。借用只是预填：Participant 与角色绑定仍在本 Project 内重新准入，权限、预算和绑定不跨仓库继承；将来若要沉淀为可共享的一等对象，再另行设计。

Workbench 就位之前，Matrix 客户端负责读写消息、引用和讨论，治理命令走公共命令入口；先后路径见[交付文档的实现阶段](./delivery.md#实现阶段)。这是产品路径的先后，不是说 Matrix 客户端低一等。聊天文字本身不包含命令类型、目标版本和预览选择，所以 mention 不会自动触发；将来若 Matrix widget/AppService 能提交显式结构化动作，也必须归一到同一 Preview/Submit 入口。

普通 Room 里的临场执行边只能来自可稳定归属到 human 的动作，并且必须先经过 Trigger Preview。动作可以由 Workbench/CLI 直接提交，也可以由供应端适配器提交结构化事件；客户端名称不改变规则。聊天消息本身不是入口。

模型 Participant 的消息、结果提案和总结（包括正文里的 `@`）只能形成“下一位协作者”的建议，不能自行发起调用、唤醒执行体或层层转包。用户批准建议后，系统自动把原消息、引用、上下文、权限、预算和上一次调用的关系带进新预览，不要求人复制粘贴。

重复且无需临场判断的协作进入 [Workflow](./run.md)，由确定性规则按冻结的施工图创建。精确规则见[约束附录](./spec/project.md#场景约束)。

| 角色 | 可以做什么 |
| --- | --- |
| 场景客户端：Workbench Room | 通过 Matrix 写消息 content；提供完整时间线、Composer（输入区）、预览和公共命令入口 |
| 场景客户端：CLI | 承载调用、Request、升格、Memo/Artifact 的预览与提交，以 chat server 消息事件 ID 引用讨论内容；聊天读写走 Matrix 客户端 |
| content 系统：chat server（Matrix 协议） | 承载消息、调用过程与结果卡的 ground truth（事实源头）；Workbench 与 Matrix 生态客户端可直接读写聊天 |

chat server 是第一阶段组件（选型与验证见[交付文档](./delivery.md)），Matrix 生态客户端天然可用；非 Matrix 平台由 homeserver/bridge 生态接入，HCTL 只处理身份映射。职责边界见[三面架构](./architecture.md#避免供应商锁定)。

HCTL 创建或绑定的房间不开端到端加密，因为冻结引用、Context 萃取和桥接都要求 control 能按消息 ID 读取正文。绑定校验、事后加密的降级与换绑恢复由[Project 约束](./spec/project.md#room-与消息)定义。

## 模块交接

以下只列所有权方向；字段、事务与故障语义由[连接约束](./spec/connections.md)统一定义。

- Project 中的提案只有通过采纳命令才会产生 [Task](./task.md) 契约的新版本。
- Project 可以通过 [Agent](./agent.md) 模块发起一次 Room Invocation；持久自动施工必须显式创建 [Run](./run.md)。
- Task、Run 和 Agent 模块的状态以投影或引用回到 Chat Room；显式 human 动作经对应模块的公共命令入口请求变化。
- 稳定经验通过 Memo 回流；交付内容通过 Artifact 的不可变发布版本回流。

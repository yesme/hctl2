# Project 与 Chat Room

> 本文是 Project 模块的设计正文：它为什么存在、拥有什么、按什么规则运转。精确对象、状态机与写入合同见[合同附录](./spec/project.md)；模块交接见[连接合同](./spec/connections.md)，共享机制见[系统边界](./spec/system.md)。

## 为什么存在

Harness 的会话、终端和 worktree（Git 工作树）都会结束或被替换；Project 的目标、论证、Participant（参与者）关系、来源和未决问题却必须继续存在。Project 模块保存的正是这份长期事实，Chat Room 则是所有 Harness 都消失之后仍然可以恢复的协作现场——它回答“我们要解决什么、为什么、依据是什么，以及哪些讨论已经足够稳定，可以成为承诺”。

以 Room 为中心不等于所有工作都必须聊天：Kanban、Workflow 图和 Terminal 各有自己的操作面。Room 的特殊地位在于承载目标塑形的连续性（shaping continuity），而不是承载所有机械执行事件。

Project 也不是施工管线：研究、规格说明、ADR（架构决策记录）和纯文档的 Project 可以从未创建 Run。Project 不预配常驻的“包工头”Agent；Participant 是可寻址的逻辑档案，只有显式调用才产生有边界的执行。

## 模块拥有什么

Project 模块保存“为什么做、依据是什么、谁在参与”的长期事实：目标与范围，协作现场（Room）与消息，Participant 与角色，每次调用可解释的上下文（Context），向人索取输入的 Request（请求卡），沉淀的 Memo（备忘）与登记的 Artifact（工件），以及从聊天室发起的单次调用（RoomInvocation）。它不等于仓库、聊天串、Task 集合、Run、Harness 会话或 worktree。

## 关键规则

- 聊天只能形成提案；正式变化必须走带预览的类型化命令，Room 不能签发 Verdict、Receipt 或完成 Task。
- 普通 Room 中只有 human actor 能提交临场执行边；模型或 Agent 只能建议下一条边。
- Participant、角色、Room 和 Project 都不是 Harness 进程或外部账号；换人、换工具不改写已授权的执行。
- 上下文必须能解释它当时看到了什么；模型自由总结不能替代来源和版本。
- Request 只能由获准动作解决，只解锁它声明的阻塞范围；应答面按需升级——默认在卡片或详情里回答，需要多轮论述、多人参与或共同编辑才开临时讨论空间，敏感输入走安全通道，只有诊断或接管才连接终端。无论升到哪一级，都还是同一个请求、同一个阻塞范围。
- 消息只追加；修正、删除和外部编辑留痕，不能抹掉已被引用的历史。
- 原始消息和执行日志不会自动变成长期知识；Memo 必须显式提炼、预览并发布。
- 单次调用适合一次性的研究、比较或范围明确的写入；需要持久重试、候选切换或评审关卡时应创建 [Run](./run.md)。
- 从 Repo Room 提升 Project 时只带显式选中的来源，可删减、补充、去敏；之后的聊天不会偷偷改变既有 Project。
- Project/Room 历史不因客户端关闭、外部编辑或运行时崩溃而丢失。

## Room 类型

| Room | 作用 | 生命周期 |
| --- | --- | --- |
| Repo Room | 无固定主题的研究、发现和 Project 入口 | 与 RepoInstance 同寿命 |
| Project Room | 围绕一个 Project 的长期协作和里程碑 | Project 归档后只读 |
| Scoped Room | 为复杂 Request 或决定临时建立的讨论空间 | 结论回填类型化动作后归档 |

临时讨论空间必须先说清目标与完成后回填什么；只有回填动作成功才能归档。

## Chat Room 场景

Chat Room 是 Project 的主要操作场景，提供：

- 消息顺序由账本统一给出，不靠时间戳或渲染顺序；
- `@` Participant/Role、`/` 类型化动作、`$` Skill、`#` 文件/Artifact/消息引用；
- 并发 RoomInvocation 的独立流、取消和结果卡；
- Request、Project 概览、Task/Run 里程碑和 Needs Attention（需要关注）投影；
- mention 提交前的 Trigger Preview：发出前先看清楚谁来执行、带什么上下文、有什么权限和预算、会创建什么；
- Context 预览、Memo/Artifact 发布预览和权限说明。

在 Workbench 里同时管理多个仓库时，一个 Room 可以把另一个仓库 Room 的 Participant 阵容借用为预填选择，不必逐个重选。借用只是预填：Participant 与角色绑定仍在本 RepoInstance 内重新准入，权限、预算和绑定不跨仓库继承；将来若要沉淀为可共享的一等对象，再另行设计。

普通 Room 里的临场执行边只能由经过认证的人提交，并且必须先看过 Trigger Preview；人可以在 Workbench、CLI 或适配后的外部聊天平台上操作，但消息来源必须能证明是人。Agent 的消息、结果提议和总结（包括正文里的 `@`）只能形成“下一位协作者”的建议，不能自行发起调用、唤醒 worker 或层层转包；用户批准建议后，系统自动把原消息、引用、上下文、权限、预算和上一次调用的关系带进新预览，不要求人复制粘贴。重复且无需临场判断的协作应进入 [Workflow](./run.md)，由确定性规则按冻结的施工图创建。精确规则见[合同附录](./spec/project.md#场景合同)。

| 角色 | 可以做什么 | 不能做什么 |
| --- | --- | --- |
| 场景客户端：Workbench Room | 提供完整时间线、Composer（输入区）、预览和命令入口 | 直接写 SQLite 或把渲染动作当成领域结果 |
| 场景客户端：CLI | 查询 Room/Request；第一阶段复杂编辑安全暂停 | 绕过预览、版本或权限检查 |
| 受控端口 / 原生客户端：外部聊天平台 | 在能力允许时投递/接收同一 Room 的消息与 Request | 以外部 thread/message ID 取代 Project/Room 身份 |

外部聊天桥接不是第一阶段出门条件；一旦交付，必须具备稳定身份、去重、回声抑制、outbox、重连和降级能力。

## 模块交接

以下只列所有权方向；字段、事务与故障语义由[连接合同](./spec/connections.md)统一定义。

- Project 中的提案只有通过采纳命令才会产生 [Task](./task.md) 契约的新版本。
- Project 可以发起一次 RoomInvocation；持久自动施工必须显式创建 [Run](./run.md)。
- Task、Run 和 Harness 的状态只以投影或引用回到 Chat Room，不能由聊天反向改写。
- 稳定经验通过 Memo 回流；交付内容通过 Artifact 的不可变发布版本回流。

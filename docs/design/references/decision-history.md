# 从 HCTL 到 HCTL2 的来时路

> 状态：Informative · 草案 v0.8.0<br>
> 定位：本文只解释关键决策为什么转向，不定义当前对象、状态、命令或交付范围。当前合同以[设计地图](../README.md)及其链接的模块、连接和系统文档为准；可复核的版本与源码依据见[实现证据](./implementation-evidence.md)。

HCTL2 不是从一张完整产品蓝图一次推导出来的。它从 HCTL1 的治理内核出发，先面对多 Harness 终端与工作树的现实问题，再逐步把用户意图、任务承诺、受治理执行和物理运行时分开。下面记录的是这条边界收敛路径，而不是另一份规范。

## 1. HCTL1：Git-native 治理内核

HCTL1 首先解决的是“谁可以对哪个精确版本作出什么裁决”。它把协调事实放在 Git 上，形成 append-only 事件、local/remote CAS、claim OID fence、精确 `{base, head}` Verdict、quorum、可重放的 squash merge Receipt，以及事实不完整时 fail-closed 的 level-triggered reconciliation。这套可执行内核证明：治理可以依靠稳定身份、精确版本和可重放证据，而不依赖常驻 daemon、数据库时钟或界面状态。

这条谱系没有被 HCTL2 丢弃。HCTL2 延续了版本化证据、claim/fence、法定票数、Receipt 和对账思想；但 HCTL1 的对象模型和 Git 事实布局只适合它当时的窄范围，不是 HCTL2 的存储或产品蓝图。具体基线与限制见 [`E-L2-HCTL1`](./implementation-evidence.md#e-l2-hctl1)。

## 2. 多 Harness 终端是产品问题的起点

HCTL2 最初面对的产品表象，是同时管理多个 Harness、终端、session、pane 和 worktree。Zellij、WezTerm、cmux、Termio、Herdr 等实现说明了进程托管、终端重连、布局和运行信号应怎样做得可靠，也暴露了一个边界：终端只能说明“哪里在运行”，不能说明“为什么运行、代表哪个用户目标、谁批准了写入、结果如何验收”。

因此 Terminal 从产品中心退为 Harness 的操作场景。进程、PTY、pane 和 runtime 都是物理承载；它们可以丢失和重建，却不能反向定义 Project、Task 或 Run 的事实。这一转折把 HCTL2 从“更好的多终端管理器”推向了治理工作台。

## 3. 从工作区直觉到四个权威模块

接下来需要把几个生命周期不同的问题拆开：用户围绕什么持续协作、什么构成可验收承诺、何时值得启动自动施工、哪个执行体真正接触资源。早期研究曾用四层来整理 donor 的长处；收敛后的领域所有权则落在四个模块，而不是四个实现层：

- **Project** 保存长期目标、协作上下文、Request、Memo 与 Artifact；
- **Task** 保存可排队、可分派、可独立验收的承诺及其来源绑定；
- **Run** 保存 WorkflowRevision、受治理执行、Obligation、Seat、Attempt、Verdict 与 Receipt；
- **Harness** 保存 Worker/Harness 绑定、ChangeSet、runtime、终端与执行证据。

拆分的关键不是把流程强制串成四步，而是让可选关系保持真实：Task 可以不启动 Run，Project 也可以直接发起一次 Harness 调用；只有需要耐久、可恢复的自动施工时才创建 Run。历史证据中的 L1–L4 标签仍可用于追溯参考实现，但不再构成 HCTL2 的领域层级。

## 4. 四个 Scene 与模块对仗

模块回答“谁拥有事实”，Scene 回答“用户从哪里理解和操作这些事实”。最终形成一组稳定对仗：

| 权威模块 | 主 Scene | 这次转折解决的问题 |
| --- | --- | --- |
| Project | Chat Room | 对话、上下文与协作不再依附某个常驻 foreman 或终端 |
| Task | Kanban | 承诺、来源同步和完成验收不再退化为聊天消息或 Run 终态 |
| Run | Workflow | 施工图、机械进度与语义 Gate 可以展开查看，但图本身不是事实源 |
| Harness | Terminal | 进程、输入输出和物理恢复有明确归属，不冒充上层成功 |

Scene 是投影和操作面，不是第五个 writer。Room 的 repo/project/scoped 拓扑也与控制拓扑正交：共享协作不要求某个 Harness 永久在线，终端重连更不能接管 Room 或 Project 的身份。

## 5. Workbench、第三方客户端与受控端口分离

Workbench 随后被明确为四个 Scene 的集成客户端，而不是新的领域层。CLI、Workbench 和第三方原生 UI 都通过同一类 Query、Preview、Submit、Subscribe 合同工作；关闭客户端不改变领域事实，也不授予额外权限。

与此同时，Chat、TaskSource、WorkflowEngine、Harness 和 RuntimeBackend 被收敛为受控端口。它们提供外部能力、报告版本与降级方式，但不能凭平台自身的 Session、Issue、workflow task、pane 或数据库取得 HCTL 字段权威。一个产品可以同时提供原生客户端和受控端口，例如第三方看板既展示 Task 又承载部分外部字段；此时 client binding 与 authority binding 仍须分开，避免“能显示”被误写成“能决定”。

## 6. Conductor 只拥有机械状态

引入 Conductor 的目的，是复用耐久 external task、wait/timer、retry 与历史恢复，而不是把 HCTL 的语义交给工作流引擎。这个边界最终固定为：Conductor 保存机械工作流位置，HCTL control 领取外部任务并建立 Obligation/Seat/Attempt，core/control 校验精确版本、证据、权限、Gate 和 Receipt。

因此 Conductor 不选择 Harness、不创建逻辑 Seat、不解释语义驳回、不计算 HCTL quorum、不签发 Receipt，也不直接写 Git 或第三方系统。机械 retry 与 HCTL 的替代执行、返工和 regate 是不同身份；把两者分开，才使引擎恢复不会复制票数、绕过验收或把旧执行复活。

## 7. 延续治理合同，同时更换对象与事实源

HCTL2 对 HCTL1 的继承是合同层的，而不是表结构层的：

| 延续的思想 | HCTL2 中的收敛 |
| --- | --- |
| 稳定身份与精确版本证据 | 不可变 Revision、digest、ReviewSubjectRef 与冻结的执行规格 |
| claim、CAS 与 fence | command expected version、generation、租约和单写者边界 |
| 精确 Verdict、quorum 与 Receipt | Run Gate 及 core/control 校验后的正式证明 |
| fail-closed reconciliation | outbox/readback、未知结果保留和冷启动恢复 |

与此同时，对象含义发生了变化。HCTL1 的 `Seat = harness × model`；HCTL2 的 Seat 是 Obligation 内稳定的逻辑执行者或投票位置，并允许多个 Attempt。HCTL1 的 Obligation 主要承载静态 author/gate/merge 责任；HCTL2 的 Obligation 对应一次 Engine external task 的逻辑产出责任。HCTL1 以 Seat refs、PR 和 squash Receipt 作为主要协调事实；HCTL2 把四模块操作账本放入 RepoInstance SQLite，把共享定义和内容证据放入 Git，把机械工作流位置交给 Conductor，并通过连接合同交换精确引用。

所以 Git 的地位从“几乎全部协调状态”变成“共享、低频、可审计的定义与内容事实”；Receipt 仍是已经校验之结果的证明，却不再暗示旧对象或旧存储布局继续有效。

## 8. 参考实现各取所长

HCTL2 没有选择一个 donor 作为总模型，而是按边界吸收可验证的长处：First Tree、Claude Tag 和 OpenClaw 帮助厘清 Project/Room 与外部协作；Codeg、Hermes Agent、Multica 及 Linear/GitHub 研究帮助厘清 Task、看板和来源同步；HCTL1、Conductor、Stably Orca、Herdr 与 ZeroClaw 帮助厘清治理、机械状态和监督边界；Stably Orca、Superset、Herdr、DeepSeek Harness 及多种 Harness access protocol 帮助厘清 runtime、终端与受控访问。

这些实现也提供 UI、协议、恢复和运维原语，但都不定义 HCTL2 的公共对象、状态机或事实源。采用原则始终是“保留可复核能力，重画权威边界”：参考实现证明某个局部机制可行，四模块与连接合同决定它在 HCTL2 中能拥有什么。

## 9. 当前落点

这条来时路最终收敛为四个权威模块、四个对仗 Scene、共享但受控的连接与执行机制。阅读当前设计时，应从 [Project](../project.md)、[Task](../task.md)、[Run](../run.md)、[Harness](../harness.md) 开始，再查看[连接合同](../connections.md)、[系统边界](../system.md)和[第一阶段交付](../delivery.md)。本文用于解释这些边界为什么存在；发生冲突时，它不覆盖任何当前规范。

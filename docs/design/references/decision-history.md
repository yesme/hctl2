# 从 HCTL 到 HCTL2 的来时路

> 状态：Informative · 对应草案 v0.9.1 · 2026-08-19<br>
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

Workbench 随后被明确为四个 Scene 的集成客户端，而不是新的领域层。自建它的理由不是重写通用 UI，而是四模块的导航结构无法无损套入任何 donor 的会话、终端或工作树主导航。CLI、Workbench 和第三方原生 UI 都通过同一类 Query、Preview、Submit、Subscribe 合同工作；关闭客户端不改变领域事实，也不授予额外权限。

与此同时，Chat、TaskSource、WorkflowEngine、Harness 和 RuntimeBackend 被收敛为受控端口。它们提供外部能力、报告版本与降级方式，但不能凭平台自身的 Session、Issue、workflow task、pane 或数据库取得 HCTL 字段权威（第 12 节后来把这条精确化为“可拥有 content、不可拥有治理”）。一个产品可以同时提供原生客户端和受控端口，例如第三方看板既展示 Task 又承载部分外部字段；此时 client binding 与 authority binding 仍须分开，避免“能显示”被误写成“能决定”。

## 6. Conductor 只拥有机械状态

引入 Conductor 的目的，是复用耐久 external task、wait/timer、retry 与历史恢复，而不是把 HCTL 的语义交给工作流引擎。这个边界最终固定为：Conductor 保存机械工作流位置，HCTL control 领取外部任务并建立 Obligation/Seat/Attempt，core/control 校验精确版本、证据、权限、Gate 和 Receipt。

因此 Conductor 不选择 Harness、不创建逻辑 Seat、不解释语义驳回、不计算 HCTL quorum、不签发 Receipt，也不直接写 Git 或第三方系统。机械 retry 与 HCTL 的替代执行、返工和 regate 是不同身份；把两者分开，才使引擎恢复不会复制票数、绕过验收或把旧执行复活。

## 7. 从 prompt 约束到机械终结权

早期实践曾试图通过 prompt 要求 Harness 在“确实完成”时才报告完成，并期待模型自行维持 Task、Run 与执行结果之间的边界。实际使用表明，这种约束不能稳定提供终结权：模型输出仍是受上下文影响的建议，Harness 也只能观察本次执行，无法替 Task 的冻结验收合同作出权威裁决。由此形成一条更一般的教训：凡是能够由身份、版本、状态与证据机械判断的事项，就不再交给 LLM 灵活解释。

当前合同因而把 Task 完成收窄为两条可审计路径。其一是有权 human actor 从 Kanban Scene 提交 `CompleteTaskIntent`；其二是绑定该 TaskRevision 的 Workflow 正常完成后，由 reducer 按冻结规则提交同一个 `CompleteTaskIntent`。第二条路径不是让 `Run.Completed` 静默传染给 Task：Task 仍须重新校验版本、证据和来源状态，校验失败便保持 Open；Workflow 的失败、取消或替代终态也不会终结 Task。`Task.Cancelled` 则仍由有权 human actor 显式决定。

Harness、Participant、模型输出和 runtime signal 可以提供 ResultProposal、Artifact 与 Evidence，却不成为 terminal actor。这个 pivot 把过去依赖提示词维持的行为期望，下沉为 core/control 可以拒绝的 actor provenance、命令与 reducer 规则。

## 8. 从开放 Agent mesh 到受控协作边

First Tree 证明了持久 Chat、稳定且显式的 recipient，以及发生在共享对话中的可见 handoff，能够维持多 Agent 协作连续性；这些优点被保留。它也允许 Agent 在运行中 `invite + send`，并由接收者继续寻址第三个 Agent，使协作图临场生长。HCTL2 没有把这后一项作为普通 Room 的默认能力，因为创建下一条执行边同时改变参与者、Context 披露、权限、预算与终止条件。

因此当前设计只承认两类边：普通 Chat Room 的临场协作边由有权 human actor 创建，Workflow 的执行边由 reducer 按冻结的 WorkflowRevision 创建。Agent-authored message、ResultProposal 或模型总结可以建议下一位 Participant，却不能自行 cue 新 worker、扩大 fan-out 或递归委派。在 Chat Room 中，人本来就处于讨论焦点；系统应把建议变成可一键批准、自动携带引用与 Context 的 handoff，而不是让人重新复制和转述内容。

这不是否定多 Agent 协作，也不是宣称 Agent-to-Agent 永远无效；它把开放的临场 mesh 换成“人拥有即时星形拓扑、状态机拥有预授权有界图”的当前产品选择。First Tree 的具体采用与拒绝证据见 [`E-L4-FIRST-TREE`](./implementation-evidence.md#e-l4-first-tree)。

## 9. 延续治理合同，同时更换对象与事实源

HCTL2 对 HCTL1 的继承是合同层的，而不是表结构层的：

| 延续的思想 | HCTL2 中的收敛 |
| --- | --- |
| 稳定身份与精确版本证据 | 不可变 Revision、digest、ReviewSubjectRef 与冻结的执行规格 |
| claim、CAS 与 fence | command expected version、generation、租约和单写者边界 |
| 精确 Verdict、quorum 与 Receipt | Run Gate 及 core/control 校验后的正式证明 |
| fail-closed reconciliation | outbox/readback、未知结果保留和冷启动恢复 |

与此同时，对象含义发生了变化。HCTL1 的 `Seat = harness × model`；HCTL2 的 Seat 是 Obligation 内稳定的逻辑执行者或投票位置，并允许多个 Attempt。HCTL1 的 Obligation 主要承载静态 author/gate/merge 责任；HCTL2 的 Obligation 对应一次 Engine external task 的逻辑产出责任。HCTL1 以 Seat refs、PR 和 squash Receipt 作为主要协调事实；HCTL2 把四模块操作账本放入 RepoInstance SQLite，把共享定义和内容证据放入 Git，把机械工作流位置交给 Conductor，并通过连接合同交换精确引用。

所以 Git 的地位从“几乎全部协调状态”变成“共享、低频、可审计的定义与内容事实”；Receipt 仍是已经校验之结果的证明，却不再暗示旧对象或旧存储布局继续有效。

## 10. 参考实现各取所长

HCTL2 没有选择一个 donor 作为总模型，而是按边界吸收可验证的长处：First Tree、Claude Tag 和 OpenClaw 帮助厘清 Project/Room 与外部协作；Codeg、Hermes Agent、Multica 及 Linear/GitHub 研究帮助厘清 Task、看板和来源同步；HCTL1、Conductor、Stably Orca、Herdr 与 ZeroClaw 帮助厘清治理、机械状态和监督边界；Stably Orca、Superset、Herdr、DeepSeek Harness 及多种 Harness access protocol 帮助厘清 runtime、终端与受控访问。

这些实现也提供 UI、协议、恢复和运维原语，但都不定义 HCTL2 的公共对象、状态机或事实源。采用原则始终是“保留可复核能力，重画权威边界”：参考实现证明某个局部机制可行，四模块与连接合同决定它在 HCTL2 中能拥有什么。

## 11. 过度精简的教训与愿景层的恢复

v0.8.0 把规范收敛为四个模块时，以“移除平行权威”为由删除了大部分愿景表述：一句话定位、失败模式叙事、四阶段心智模型、目标体验旅程、命名设计原则和各模块的“为什么存在”。合同收紧本身是对的，但这次删除混淆了两类内容：权威去重针对的是**合同**——一个对象只能有一个定义；它不适用于**解释**——愿景、论证和体验叙事不与合同竞争权威，删掉它们只会让合同失去裁决新问题的方向，也让新读者只知道规则是什么、不知道规则为什么存在。

v0.9.0 因此把“为什么”与“是什么”分开维护：愿景、原则和目标体验回到独立的[愿景文档](../vision.md)，四个模块文档各自恢复“为什么存在”的开篇，术语表恢复为非规范对照；全部合同保持 v0.8.1 收紧后的形态不变。四阶段链“意图 → 承诺 → 治理 → 运行”作为心智模型恢复，同时明确一件事不必完整经历四个阶段——用一句话消解线性误读，而不是删除整个模型。

v0.9.1 处理概念层面的同类问题。全量清点暴露出 75 个以上具名概念平铺在设计正文里——很多是执行路径上的一个步骤、一份冻结字段组或一个状态值，被顺手建成了“类”。修订按三个动作收口：设计正文与合同层分离（`docs/design/spec/`，设计层只用核心产品词加日常语言）；概念按六族归类并归并（复合名 41 → 28，如派发规格、物理运行时、外部副作用意图各归一名）；场景概念对齐外部标准（Chat ↔ Matrix/Slack，Kanban ↔ Linear/GitHub，Workflow ↔ Conductor/BPMN，Terminal ↔ PTY/tmux/ACP），自造词只保留差异化语义。这轮方法论的完整记录见 `.memo/design-doc-method-20260819.md`。

## 12. 场景数据的三分：metadata / content / artifact

v0.9.1 之前，四模块操作账本整体放在 RepoInstance SQLite 里（第 9 节）。随后的多设备讨论暴露了一个产品语义错误：同一个人在两台机器各 clone 一份仓库，Room 的协作历史却被锁在单个 clone 的本地账本里——clone 丢了，协作记忆就丢了。`.memo/room-ground-truth-20260819.md` 曾比较四个候选，当时把“Matrix 作为权威”判为违反端口纪律，倾向用户级 hub。

`.memo/scene-data-model-20260820.md` 给出了更精确的刀法：每个场景的持久数据分三类——metadata（治理元数据：身份、绑定、授权、判决）、content（场景内容：消息、任务卡与流转、机械执行历史、会话转录）、artifact（结晶：决议、冻结契约与施工图、凭证链、代码变更）。Workflow 与 Terminal 两个场景本来就这样运作——工作流引擎拥有机械历史，harness 会话拥有转录，HCTL 只留绑定与治理；三分法把这条规则推广到 Chat 与 Kanban，补掉了不对称。

这次转向显式推翻了两条旧结论。其一，“平台不能成为第五事实源”精确化为**可以拥有 content、不能拥有治理**——room-ground-truth memo 对 Matrix 候选的否决在三分下失效：平台拥有的是记忆，不是裁决。其二，metadata 账本的归属从 RepoInstance 上移到用户级控制面（一人多机连同一个控制面），RepoInstance 只剩代码侧的物理事实。该 memo 的两个遗留分叉同时裁决：用户级 hub 以“控制面即 hub”的形式采纳；用户级“总入口对话面”否决——用户进入产品就在某个 repo 之下，这是显式设计决定，不是遗漏。

这不是把治理交给平台，也不是回到 HCTL1 的“Git 承载一切”：判决仍只在 metadata 层产生，冻结摘要仍是 content 与治理之间的防火墙（第 9 节的继承表原样成立）。另有一条词汇裁决随本次转向生效：自 v0.10.0 起，“Agent”一词专属第四模块（原 Harness 模块更名），“Harness”专指编码代理工具这一系统角色，散文中的 AI 协作者用 Participant 表述。类别的权威定义见[合同层总则](../spec/README.md#三类数据)；候选系统与限时验证见[交付文档](../delivery.md)。

## 13. 当前落点

这条来时路最终收敛为四个权威模块、四个对仗 Scene、共享但受控的连接与执行机制。阅读当前设计时，应从[愿景](../vision.md)开始，再读 [Project](../project.md)、[Task](../task.md)、[Run](../run.md)、[Harness](../harness.md)，再查看[连接合同](../spec/connections.md)、[系统边界](../spec/system.md)和[第一阶段交付](../delivery.md)。本文用于解释这些边界为什么存在；发生冲突时，它不覆盖任何当前规范。

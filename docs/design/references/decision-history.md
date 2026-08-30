# 从 HCTL 到 HCTL2 的来时路

> 状态：Informative · 对应草案 v0.15.4 · 2026-08-31<br>
> 定位：本文只解释关键决策为什么转向，不定义当前对象、状态、命令或交付范围。当前合同以[设计地图](../README.md)及其链接的模块、连接和系统文档为准；可复核的版本与源码依据见[实现证据](../../research/README.md)。

HCTL2 不是从一张完整产品蓝图一次推导出来的。它从 HCTL1 的治理内核出发，先面对多 Harness 终端与工作树的现实问题，再逐步把用户意图、任务承诺、受治理执行和物理运行时分开。下面记录的是这条边界收敛路径，而不是另一份规范。

## 1. HCTL1：Git-native 治理内核

HCTL1 首先解决的是“谁可以对哪个精确版本作出什么裁决”。它把协调事实放在 Git 上，形成 append-only 事件、local/remote CAS、claim OID fence、精确 `{base, head}` Verdict、quorum、可重放的 squash merge Receipt，以及事实不完整时 fail-closed 的 level-triggered reconciliation。这套可执行内核证明：治理可以依靠稳定身份、精确版本和可重放证据，而不依赖常驻 daemon、数据库时钟或界面状态。

这条谱系没有被 HCTL2 丢弃。HCTL2 延续了版本化证据、claim/fence、法定票数、Receipt 和对账思想；但 HCTL1 的对象模型和 Git 事实布局只适合它当时的窄范围，不是 HCTL2 的存储或产品蓝图。具体基线与限制见 [`E-L2-HCTL1`](../../research/lineage/hctl1.md#e-l2-hctl1)。

## 2. 多 Harness 终端是产品问题的起点

HCTL2 最初面对的产品表象，是同时管理多个 Harness、终端、session、pane 和 worktree。Zellij、WezTerm、cmux、Termio、Herdr 等实现说明了进程托管、终端重连、布局和运行信号应怎样做得可靠，也暴露了一个边界：终端只能说明“哪里在运行”，不能说明“为什么运行、代表哪个用户目标、谁批准了写入、结果如何验收”。

因此 Terminal 从产品中心退为第四模块的操作场景；该模块在当时称 Harness，v0.10.0 后正名为 Agent。进程、PTY、pane 和 runtime 都是物理承载；它们可以丢失和重建，却不能反向定义 Project、Task 或 Run 的事实。这一转折把 HCTL2 从“更好的多终端管理器”推向了治理工作台。

## 3. 从工作区直觉到四个权威模块

接下来需要把几个生命周期不同的问题拆开：用户围绕什么持续协作、什么构成可验收承诺、何时值得启动自动施工、哪个执行体真正接触资源。早期研究曾用四层来整理 donor 的长处；v0.8/v0.9 当时收敛出的领域所有权落在四个模块，而不是四个实现层：

- **Project** 保存长期目标、协作上下文、Request、Memo 与 Artifact；
- **Task** 保存可排队、可分派、可独立验收的承诺及其来源绑定；
- **Run** 保存 Workflow Revision、受治理执行、Obligation、Seat、Attempt、Verdict 与 Receipt；
- **第四模块（当时称 Harness，现为 Agent）** 保存 Worker/Harness 绑定、ChangeSet、runtime、终端与执行证据。

拆分的关键不是把流程强制串成四步，而是让可选关系保持真实：Task 可以不启动 Run，Project 也可以通过第四模块发起一次 Harness 调用；只有需要耐久、可恢复的自动施工时才创建 Run。历史证据中的 L1–L4 标签仍可用于追溯参考实现，但不再构成 HCTL2 的领域层级。

## 4. 四个 Scene 与模块对仗

模块回答“谁拥有事实”，Scene 回答“用户从哪里理解和操作这些事实”。最终形成一组稳定对仗：

| 权威模块 | 主 Scene | 这次转折解决的问题 |
| --- | --- | --- |
| Project | Chat Room | 对话、上下文与协作不再依附某个常驻 foreman 或终端 |
| Task | Kanban | 承诺、来源同步和完成验收不再退化为聊天消息或 Run 终态 |
| Run | Workflow | 施工图、机械进度与语义 Gate 可以展开查看，但图本身不是事实源 |
| 第四模块（当时称 Harness，现为 Agent） | Terminal | 进程、输入输出和物理恢复有明确归属，不冒充上层成功 |

Scene 是投影和用户交互方式，不是第五个 writer。Room 的 repo/project/scoped 拓扑也与控制拓扑正交：共享协作不要求某个 Harness 永久在线，终端重连更不能接管 Room 或 Project 的身份。

## 5. Workbench、第三方客户端与受控端口分离

Workbench 随后被明确为四个 Scene 的集成客户端，而不是新的领域层。自建它的理由不是重写通用 UI，而是四模块的导航结构无法无损套入任何 donor 的会话、终端或工作树主导航。CLI、Workbench 和适配后的第三方 Scene UI 都通过同一类 Query、Preview、Submit、Subscribe 合同工作；provider 原生 UI 若只操作本系统 content，仍只是 content 客户端或诊断面。关闭任何客户端不改变领域事实，也不授予额外权限。

与此同时，Chat、任务源、workflow engine、harness 和运行时后端被收敛为受控端口。它们提供外部能力、报告版本与降级方式，但不能凭平台自身的 Session、Issue、workflow task、pane 或数据库取得 HCTL 字段权威（第 12 节后来把这条精确化为“可拥有 content、不可拥有治理”）。一个产品可以同时提供原生客户端和受控端口，例如第三方看板既展示 Task 又承载部分外部字段；此时 client binding 与 authority binding 仍须分开，避免“能显示”被误写成“能决定”。

## 6. 当时的 Conductor 只拥有机械状态（后由 §18 取代实现选型）

当时引入 Conductor，是为了复用耐久 external task、wait/timer、retry 与历史恢复，同时只让它保存机械工作流位置；Harness 选择、Seat/Attempt、语义 Gate、quorum、Receipt 与外部写入仍由 HCTL 掌握，机械 retry 也不等同于替代执行、返工或 regate。实现选型后来在 §18 换成 Dagu，这条权威边界继续保留。细节见本仓库 Git 历史。

## 7. 从 prompt 约束到机械终结权

早期实践曾试图通过 prompt 要求 Harness 在“确实完成”时才报告完成，并期待模型自行维持 Task、Run 与执行结果之间的边界。实际使用表明，这种约束不能稳定提供终结权：模型输出仍是受上下文影响的建议，Harness 也只能观察本次执行，无法替 Task 的冻结验收合同作出权威裁决。由此形成一条更一般的教训：凡是能够由身份、版本、状态与证据机械判断的事项，就不再交给 LLM 灵活解释。

这轮改判把 Task 完成收窄为两条可审计路径：有权 human actor 提交「完成 Task」命令，或绑定 Workflow 正常完成后由 reducer 提交同一命令；同时拒绝让 Run 终态直接改写 Task。现行验收、取消与失败处理见 [Task 合同的写入合同](../spec/task.md#写入合同)。

Harness、Participant、模型输出和 runtime signal 可以提供 Result Proposal、Artifact 与 Evidence，却不成为 terminal actor。这个 pivot 把过去依赖提示词维持的行为期望，下沉为工具箱与 control 可以拒绝的 actor provenance、命令与 reducer 规则。

## 8. 从开放 Agent mesh 到受控协作边

First Tree 证明了持久 Chat、稳定且显式的 recipient，以及发生在共享对话中的可见 handoff，能够维持多 Agent 协作连续性；这些优点被保留。它也允许 Agent 在运行中 `invite + send`，并由接收者继续寻址第三个 Agent，使协作图临场生长。HCTL2 没有把这后一项作为普通 Room 的默认能力，因为创建下一条执行边同时改变参与者、Context 披露、权限、预算与终止条件。

这轮改判把协作边收窄为两类：有权 human actor 创建普通 Chat Room 的临场边，reducer 按冻结的 Workflow Revision 创建执行边；Agent 只建议下一位 Participant。现行预览、授权与执行边规则见 [Project 合同的场景合同](../spec/project.md#场景合同)。

这不是否定多 Agent 协作，也不是宣称 Agent-to-Agent 永远无效；它把开放的临场 mesh 换成“人拥有即时星形拓扑、状态机拥有预授权有界图”的当前产品选择。First Tree 的具体采用与拒绝证据见 [`E-L4-FIRST-TREE`](../../research/workbench/first-tree.md#e-l4-first-tree)。

## 9. 延续治理合同，同时更换对象与事实源

HCTL2 对 HCTL1 的继承是合同层的，而不是表结构层的：

| 延续的思想 | HCTL2 中的收敛 |
| --- | --- |
| 稳定身份与精确版本证据 | 不可变 Revision、digest、ReviewSubjectRef 与冻结的执行规格 |
| claim、CAS 与 fence | command expected version、generation、租约和单写者边界 |
| 精确 Verdict、quorum 与 Receipt | Run Gate 及 control 与工具箱校验后的正式证明 |
| fail-closed reconciliation | outbox/readback、未知结果保留和冷启动恢复 |

与此同时，对象含义发生了变化。HCTL1 的 `Seat = harness × model`；HCTL2 的 Seat 是 Obligation 内稳定的逻辑执行者或投票位置，并允许多个 Attempt。HCTL1 的 Obligation 主要承载静态 author/gate/merge 责任；HCTL2 的 Obligation 对应一次 Engine external task 的逻辑产出责任。HCTL1 以 Seat refs、PR 和 squash Receipt 作为主要协调事实；在 v0.8/v0.9 当时的设计里，HCTL2 把四模块操作账本放入 Repo Instance SQLite，把共享定义和内容证据放入 Git，把机械工作流位置交给 Conductor，并通过连接合同交换精确引用。

所以 Git 的地位从“几乎全部协调状态”变成“共享、低频、可审计的定义与内容事实”；Receipt 仍是已经校验之结果的证明，却不再暗示旧对象或旧存储布局继续有效。

## 10. 参考实现各取所长

HCTL2 没有选择一个 donor 作为总模型，而是按边界吸收可验证的长处：First Tree、Claude Tag 和 OpenClaw 帮助厘清 Project/Room 与外部协作；Codeg、Hermes Agent、Multica 及 Linear/GitHub 研究帮助厘清 Task、看板和来源同步；HCTL1、Conductor、Stably Orca、Herdr 与 ZeroClaw 帮助厘清治理、机械状态和监督边界；Stably Orca、Superset、Herdr、DeepSeek Harness 及多种 Harness access protocol 帮助厘清 runtime、终端与受控访问。

这些实现也提供 UI、协议、恢复和运维原语，但都不定义 HCTL2 的公共对象、状态机或事实源。采用原则始终是“保留可复核能力，重画权威边界”：参考实现证明某个局部机制可行，四模块与连接合同决定它在 HCTL2 中能拥有什么。

## 11. 过度精简的教训与愿景层的恢复

v0.8.0 把规范收敛为四个模块时，以“移除平行权威”为由删除了大部分愿景表述：一句话定位、失败模式叙事、四阶段心智模型、目标体验旅程、命名设计原则和各模块的“为什么存在”。合同收紧本身是对的，但这次删除混淆了两类内容：权威去重针对的是**合同**——一个对象只能有一个定义；它不适用于**解释**——愿景、论证和体验叙事不与合同竞争权威，删掉它们只会让合同失去裁决新问题的方向，也让新读者只知道规则是什么、不知道规则为什么存在。

v0.9.0 因此把“为什么”与“是什么”分开维护：愿景、原则和目标体验回到独立的[愿景文档](../vision.md)，四个模块文档各自恢复“为什么存在”的开篇，术语表恢复为非规范对照；全部合同保持 v0.8.1 收紧后的形态不变。四阶段链“意图 → 承诺 → 治理 → 运行”作为心智模型恢复，同时明确一件事不必完整经历四个阶段——用一句话消解线性误读，而不是删除整个模型。

v0.9.1 处理概念层面的同类问题。全量清点暴露出 75 个以上具名概念平铺在设计正文里——很多是执行路径上的一个步骤、一份冻结字段组或一个状态值，被顺手建成了“类”。修订做了三件事：设计正文与合同层分离（`docs/design/spec/`，设计层只用核心产品词加日常语言）；概念按六族归类并归并（复合名 41 → 28，如派发规格、物理运行时、外部副作用意图各归一名）；场景概念对齐外部标准（Chat ↔ Matrix/Slack，Kanban ↔ Linear/GitHub，Workflow ↔ Conductor/BPMN，Terminal ↔ PTY/tmux/ACP），自造词只保留差异化语义。这轮方法论的完整记录见 `.memo/notes/design-doc-method-20260819.md`。

### v0.9.1 归并对照

核销记录（自 spec/README 迁入）

| 旧名 | 现状 |
| --- | --- |
| InvocationBinding / AttemptSpec | 合并为 Execution Spec（owner = Room Invocation \| Attempt） |
| RuntimeShard / InvocationRuntime | 合并为 Execution Runtime（owner 字段） |
| TerminalBundle | Execution Runtime 的终端通道字段组 |
| HarnessAdapterBinding | Execution Spec 冻结的接入方式字段组 |
| IntegrationIntent / ExternalEffectIntent | 合并为外部副作用命令（executor = tool 本地 Git \| adapter 远端） |
| TaskSourceConnection / TaskSourceConnectionRevision | 由 Resolved Port Binding（port_kind = task_source）承载 |
| ChatSurfaceBindingRevision | Room 的 Chat 端口绑定字段组（引用 Resolved Port Binding） |
| TaskSourceBindingRevision（及裸用 BindingRevision） | Task Binding |
| EngineDeploymentRevision | Engine Deployment |
| ChangeSetWriteLease | Write Lease |
| HarnessDefinition / Installation / Capability | “Harness 目录”的三类探测事实，无类名 |
| TerminalGateway / WorkflowEngineAdapter | 描述性说法：Agency 客户端适配代码 / workflow engine 端口适配器；不形成独立服务或领域对象 |

## 12. 场景数据的三分：metadata / content / artifact

v0.9.1 之前，四模块操作账本整体放在 Repo Instance SQLite 里（第 9 节记录的是这一时期的设计，并非当前落点）。随后的多设备讨论暴露了一个产品语义错误：同一个人在两台机器各 clone 一份仓库，Room 的协作历史却被锁在单个 clone 的本地账本里——clone 丢了，协作记忆就丢了。`.memo/design/room-ground-truth-20260819.md` 曾比较四个候选，当时把“Matrix 作为权威”判为违反端口纪律，倾向用户级 hub。

`.memo/design/scene-data-model-20260820.md` 给出了更精确的刀法：每个场景的持久数据分三类——metadata（治理元数据：身份、绑定、授权、判决）、content（场景内容：消息、任务卡与流转、机械执行历史、会话转录）、artifact（结晶：决议、冻结契约与施工图、凭证链、代码变更）。Workflow 与 Terminal 两个场景本来就这样运作——工作流引擎拥有机械历史，harness 会话拥有转录，HCTL 只留绑定与治理；三分法把这条规则推广到 Chat 与 Kanban，补掉了不对称。

这次转向显式推翻了两条旧结论。其一，“平台不能成为第五事实源”精确化为**可以拥有 content、不能拥有治理**——room-ground-truth memo 对 Matrix 候选的否决在三分下失效：平台拥有的是记忆，不是裁决。其二，metadata 账本的归属从 Repo Instance 上移到用户级控制面（一人多机连同一个控制面），Repo Instance 只剩代码侧的物理事实。该 memo 的两个遗留分叉同时裁决：用户级 hub 以“控制面即 hub”的形式采纳；用户级“总入口对话面”否决——用户进入产品就在某个 repo 之下，这是显式设计决定，不是遗漏。

这不是把治理交给平台，也不是回到 HCTL1 的“Git 承载一切”：判决仍只在 metadata 层产生，冻结摘要仍是 content 与治理之间的防火墙（第 9 节的继承表原样成立）。v0.10.1 对落点做了一处修正：Board 从 Project 级上移为 Repo 级——一个 Repo 一个 Board，Project 是板上的分组实体，Task 是卡片；这让 GitHub issues 这类天然 repo 级的后端直接对齐，也让支持父子任务的本地服务器以“任务–子任务”承载 Project–Task。v0.10.2 进一步取消了仓库侧的独立物理账本：现场记账本可由 metadata 账本、Git 与运行时观测推导，单立一本账是把实现细节写进合同——clone 本地从此只有 OS 锁与可丢弃缓存，实例注册与现场记账并入用户级账本，存储从“两本账 + Git”收敛为“一本账 + Git”。另有一条词汇裁决随本次转向生效：自 v0.10.0 起，“Agent”一词专属第四模块（原 Harness 模块更名），“Harness”专指编码代理工具这一系统角色，散文中的 AI 协作者用 Participant 表述。类别的权威定义见[合同层总则](../spec/README.md#三类数据)；候选系统与限时验证见[交付文档](../delivery.md)。

### v0.10.3 清扫

核销记录（自 spec/README 迁入）

| 旧名 | 现状 |
| --- | --- |
| RuntimeBackend | 描述性说法：运行时后端（受控端口与物理资源持有者，无对象名） |
| TaskSource | 端口种类 `port_kind = task_source`；散文写「任务源端口」 |
| WorkflowEngine | 系统角色小写 workflow engine；端口写「workflow engine 端口」 |
| HarnessAdapter | 描述性说法：harness 适配器 |

## 13. 实现计划：P/B 双表与 P0 选型（v0.11.0）

三份外部评审把开工前限时验证识别为“事实上的 phase 0”，v0.11.0 因而建立 P0–P6 实现阶段与 B0–B6 自举阶梯双表：P 表回答先建什么，B 表回答何时敢切换事实。该轮 P0 曾选择 conductor-oss、Vikunja 与 Zellij，并让 Tuwunel/Continuwuity 并列验证，远端任务后端延至 P4 后按需；这些实现与阶段安排后来分别由 §14、§18、§19、§29 和现行交付计划改判。细节见本仓库 Git 历史。

## 14. chat server 定夺 Tuwunel（v0.11.1）

同轮拍板 chat server 产品方向为 Tuwunel（接口更 API 化、与 Synapse 参考实现兼容性更强；单二进制预编译包、运维压力低；AppService 注册程序化），Continuwuity 记录在案备选；精确发布版本、存储后端与 build features 仍由 P0 验证后固定。该轮同时完成的词形收敛（驼峰名、`*Intent` 命令名与状态值拼写）见[小修订台账](#34-小修订台账)。

### v0.11.1 词形收敛

核销记录（自 spec/README 迁入）

| 旧形 | 新形 |
| --- | --- |
| 驼峰对象/票据名（TaskRevision 等 25 个） | 带空格专名（Task Revision 等），对齐 Run Manifest / Gate Receipt 先例 |
| `*Intent` 命令名（16 个） | 动宾语义名（「完成 Task」命令等）；代码标识符由实现仓库定，实现时附对照表 |
| 状态值枚举拼写 | 中文语义名（待采纳、结果未知、等待输入等） |

ChangeSet 保留原形（核心产品词、业界成词）；字段与格式名（`port_kind`、`review_subject_digest`、ReviewSubjectRef 等）在合同需要逐字指认时保留原形，不受自造语义名的词形规则约束。

## 15. 四段施工序与组件正名（v0.12.0）

v0.12.0 最初把 P0–P6 收敛为按三面架构分层的四段：P0 立营（先让四个执行面系统可运维）→ P1 备装（agentd + 工具箱）→ P2 接钥匙（control + CLI，B0–B5）→ P3 装门面（Workbench + B6）。它也把 `hctl2-core` 正名为机械工具箱 `hctl2-tool`，把 agentd 在组件表中正名为 `hctl2-agentd`，并写下“commit 署名、lint、PR 正文拼装、memo 写入、有效变化侦测等机械工作不交给模型”的施工纪律。

同一基线的复审保留四段骨架，但修正了三处过强表达：P0 改为限时、可丢弃的实现探针，content 系统在 P2 首次被纵向切片消费时才产品化；P1 的 standalone 工具只能辅助开发，B2 才是第一次真正自举；P2 的完整治理入口是公共 CLI，Matrix/Vikunja 原生界面只验证 content 互操作，Engine console 与裸 Zellij 分别只是诊断和 break-glass，除非适配器实现公开合同，否则不能统称“第三方场景客户端”。同轮也收窄了两条方法论：“外部无对应”是差异化语义的证据而非账本存储清单；自造语义名不冻结代码词形，但精确协议、字段、格式与外部原名可以保留。原生优先的打包方向不变，精确版本、后端与 build features 在 P0 证据通过后固定。

## 16. 缺口审计：转向传播与五个新机制（v0.12.0 审计）

v0.10–v0.12 的数次横切转向（用户级 control、Repo 级 Board、三类数据、Agent 正名、四段施工序）各自合理，但旧摘要、连接与验收仍残留前一版假设。一轮根因审计据此修正（方法与全部发现见 `.memo/review/20260821-v0.10.2/v07-v012-retrospective-and-architecture-audit-20260821.md`）：恢复缺失的交接与恢复合同，消除同一事实的两种解释，不恢复 v0.7 的对象爆炸。其中一部分是纯反漂移修正，如固定内核的“以仓库为边界”残句改为用户级 command service，以及三处“后端不可用时命令照常/入口降级”统一为“需要 fresh readback 的命令 fail closed，不依赖者可继续”。

除传播既有决定外，这轮引入了五个此前不存在的机制，在此补记决策：

- **用户在场证明**（v0.13.0 撤销，见 [§22](#22-信任模型收窄三条底线不可关外层笼子可选cli-即人v0130)）：以 human provenance 提交 Task 终结、普通 Room 执行边、扩权、安全输入或集成授权时，认证入口必须给出一次性、绑定规范命令摘要的用户在场证明。动因是旧威胁模型自相矛盾——execution principal 复制 human payload 调用公共 CLI 即可冒充人，单靠“human actor 才能终结 Task”挡不住。权威文本在[系统边界](../spec/system.md#命令与跨服务正确性)。
- **受治理 Harness 的 OS 沙箱入场券**（v0.13.0 降为可选加固，见 [§22](#22-信任模型收窄三条底线不可关外层笼子可选cli-即人v0130)）：第一阶段受治理执行必须运行在操作系统强制的沙箱中，凭据只经网关代用；不能强制这些边界的候选不得作为受治理执行启动。这是把“managed Harness”与“不隔离同 OS 用户”的互斥承诺改成一致，代价是 P0/B2 新增阻断性工程验证。
- **Run 正常完成谓词与 Task claim 双态**：Run 进入正常完成前须机械满足 required Obligation/Seat/Gate/output、Receipt、Attempt 撤权与全部外部副作用处理完毕；Task 对 Run 的绑定以 active / completion_pending 双态消除交接竞态。填补“Engine 报告完成即 Run 完成”的空洞。
- **挂接 Repo Instance**：Repo 与 clone 之间补显式身份链——工具箱无副作用读取 Git identity、control 预览后入账；remote URL、目录名或碰巧相同的 HEAD 不单独证明身份。这是用户级 control 转向早该有的配套。
- **账本备份集合同**：备份必须是唯一 writer 协调的一致快照（含各层 generation 与账本引用的不可变定义字节），恢复保留账本身份、推进全部代次、令旧凭证失效。把既有的“账本必须备份”从一句话落成合同。

方法论记录：这五项随审计一次合并落地，未按「真语义变更单独立项」拆批——同根问题跨文件归并可取，但代价是回滚粒度整体化、新机制险些没有决策记录（本节即补记）。后续审计若再引入新机制，机制本身应单独立项提交。

## 17. 适配器诚实合同：终局结果、观测截断与派生谱系（v0.12.0 补充）

Agent 模块的观测合同原本回答的是“信号如何仲裁”（优先级阶梯、观测不推进领域结果），但留着两个物理层必须表态的空洞：进程干净退出算什么？上报中断后的残缺事件流还算不算历史？本轮据 LobeHub 源码审计（[E-LOBEHUB](../../research/workbench/lobehub.md#e-lobehub)，其 heterogeneous-agents 适配器层的经验证做法）补齐三条 harness 适配器义务：

- **终局结果契约**：每个接入端口声明其终局结果事件；执行体进程正常退出但缺少该事件时，适配器必须合成类型化协议错误，不得默认成功——静默死亡不能冒充交付。由 control/agentd 主动取消导致的退出归因为取消，不上报为执行失败。
- **观测流完整性**：观测上报通道失败时，只能显式标记该执行的观测截断并终结事件流；有缺口的事件流不得冒充完整历史。
- **派生谱系保留**：harness 内部再派生的子执行体事件必须携带稳定的派生谱系引用，不摊平进主执行流。

同轮补一句能力边界：semantic resume 可以用自有观测留痕重建续跑输入，重建物按投影处理，不进入权威记录——恢复能力不再依赖厂商保留会话文件。术语取“终局”而非“终端”，避免与 Terminal 通道词族冲突。权威文本在 [Agent 模块合同](../spec/agent.md)的「运行时与观测」与「终端通道」。本条按 §16 的方法论教训单独立项提交并当轮补记决策。

## 18. workflow engine 从 Conductor 改为 Dagu（v0.12.1）

2026-08-23 重新按源码而非 README 审阅 workflow 候选后，第一阶段选型从 Conductor 改为 **Dagu**。本节与 §19 的两项选型改判构成 v0.12.1；该轮当时未随文件重 stamp，随 v0.12.2 补记。判据收敛为三条：Workflow Revision 必须是声明式事实源并可机械 lint schema、引用、Profile 与图结构；不要求一般性证明 loop 终止；本机运行优先单二进制、文件或嵌入式持久化，避免为机械状态引入 JVM 与数据库/队列组合。Conductor 当前版其实已默认 SQLite，旧记录“没有 SQLite”已经过时；它的逐任务 poll/complete 仍比 Dagu 更贴近被驱动模型，但总体分发和 footprint 仍较大。

Dagu 也不是天然的 external-worker engine：普通 step 会自行执行。采用边界因此固定为 HCTL JSON 经 compiler 生成受限 Dagu YAML，只使用依赖/条件/等待等机械结构和无进程 `human.task` 检查点；control 观察等待态，在 HCTL 结果先落账后经 API 完成它。Dagu 不获得 Harness 选择、Seat/Attempt、语义 Gate、quorum、Receipt 或外部副作用权威。当时还把 completion API 的 engine attempt generation 缺口列为 B4 前阻断项，后由 §23 以“代次不在 Dagu”撤销。细节见本仓库 Git 历史。

这条决定取代 §6 和 §13 的 Conductor 实现选型，不推翻“引擎只拥有机械状态”的边界。Dagu、Conductor、Windmill、Kestra、Direktiv、Serverless Workflow Synapse、Flogo、Step Functions Local 与 SCXML/XState 自建路线的源码证据和完整取舍，见 [`E-L2-DAGU`](../../research/workflow-engines.md#e-l2-dagu)；四个现役执行面依赖的版本、体积和运维账见[实现证据](../../research/README.md#已选外部服务的运维与资源占用)。

## 19. 运行时后端从 Zellij 改为 tmux（v0.12.1）

2026-08-23 对 tmux、Zellij 与 shpool 的源码、发布物和多会话实测曾把第一阶段运行时后端改为 tmux：control mode、稳定 pane ID、只读观察、查询应答和背压能力满足 agentd 的最小原语，且体积与多会话 RSS 显著低于 Zellij，shpool 又缺少所需的多观察者与终端协议能力；当时仍保留键盘协议和卡死场景的产品化验证。2026-08-28 改用 tmux 官方单二进制，并因两个 Darwin 制品的最低版本把 macOS 基线升到 15；§29 后来以 Herdr 直接实现 Agency，取代 `hctl2-agentd + tmux` 方案，当前 macOS 基线依据见[交付文档的打包策略](../delivery.md#打包策略选型判断首次消费时产品化)。细节见本仓库 Git 历史。

## 20. 桥接退役、结晶归位与概念清扫（v0.12.2）

一轮按最新方法论（三类数据、概念门槛）对 README 与设计/合同骨架的复查，解决了六件事：

- **自建聊天桥接退役（永久，不只是第一阶段后置）**：非 Matrix 平台（飞书、Slack、Discord 等）经 homeserver 侧的 Matrix 桥接生态在 content 层接入。依据是三条法的直接推论——聊天里不跑治理、记录不是命令，所以桥接纯属 content 层，而 Matrix 生态已有成熟桥接体系；HCTL 只保留 Chat 端口绑定中对桥接用户的身份映射策略。原“非 Matrix 完整聊天桥接”工作线与对应未决问题删除。
- **施工图结晶归位 Chat Room**：施工图（“干什么的计划”）从 Room 的塑形讨论中长出，不是任务流转的结晶；4×3 矩阵与统一律相应改判，并明确结晶归属以事实为准绳、不为对称硬填。施工图的对象与写入者仍归 Run 模块合同——结晶归属不随对象所有权走。
- **概念清扫**：Room Event 除名、Task Operational State 降级为操作投影、执行身份无法证明的终态统一为“丢失”（处理规则唯一定义在连接合同的失败表）；裁决与去向见[小修订台账](#34-小修订台账)与[核销表](#v0122-清扫)。
- **content 客户端与治理客户端在 README 分离**：架构图改为 content 客户端（任意 Matrix 客户端、任务后端原生界面）直连 content 系统、治理客户端连命令服务；“任意 Matrix 客户端开箱即用、桥接交给 Matrix 生态”上升为正面能力表述。
- **P2 双手模式立为正面形态**：Workbench（P3）之前，治理动作走公共 CLI、场景内容走各 content 原生界面；mention 触发与 Trigger Preview 是治理客户端的能力——聊天文字不是命令，也不是能赋予 human provenance 的认证入口。交付范围表按 P2/P3 出门条件重切。
- **措辞修正**：产品原生核心从“以仓库为边界的控制面”改为“随用户走、按仓库划分语义范围的控制面”，与系统合同的用户级 command service 一致（§16 已修系统层，本次补愿景层残句）。

### v0.12.2 清扫

核销记录（自 spec/README 迁入）

三类数据切分落地后按概念门槛复查的降级与统一：

| 旧名 / 旧词 | 现状 |
| --- | --- |
| Room Event | 除名：消息 content 本体就是 chat server 的 Matrix event；HCTL 侧只有账本内只追加的"治理事件"（以事件 ID 精确引用消息），两者都不占领域对象名额 |
| Task Operational State | 降级为 Task Binding 的字段组"操作投影"（后端操作字段的回读投影、同步账与派生健康状态）；ground truth 在 content 后端 |
| 状态值"中断"（Room Invocation） | 统一为"丢失"：执行身份无法证明时 Room Invocation 与 Attempt 进入同一状态；结束规则只在[连接合同](../spec/connections.md#失败与恢复)定义一次 |

## 21. Context 合同裁决轮（v0.12.3）

按"逐条裁决、不整篇升格"把 Context 横切正文已拍板的设计落进合同（Project 模块合同的 Context 一节与交付验证），全轮零新增具名对象：

- **管辖与物化/指针两分**（随 v0.12.2 后的澄清先行落地，本节补记决策）：Context 交付的是开工 prompt，不代管执行体会话内自组装的工作上下文；物化只限聊天萃取、契约与范围、显式引用原文，Repo/Git 内容、Artifact、Memo、Skill 一律指针化由执行体自取；选择优先级与序列化顺序分离（稳定在前、变动在后）。依据：聊天史是执行体唯一拿不到也不该自己翻的来源。
- **萃取全本地**：全文索引与相关性门是可重建派生投影，不进权威账本；门只以账本事实为判定输入、不以消息措辞做路由，判定记为可审计观测；选材不消耗大模型 token。
- **压缩合同**：缺省关闭；small-brain 是经用户级定义机制固定 revision/digest 的模型引用（无新对象）；逐条记录 compressor、压缩率与原文 digest 且片段可回源；证据类内容永不压缩，违规 Bundle 拒绝交付；萃取/压缩产物可按（room、cursor 区间、消费者范围）作派生缓存跨调用复用。
- **前情提要**：滚动纪要为组装器机械触发、small-brain 增量折叠的派生缓存——逐条回源、不可作治理引用目标、不进账本、不由房间内模型书写；无 small-brain 时以近详远略裁剪代替（缺省态的缺陷是故意保留的结晶压力）；父 Room 纪要只作提升预览预填，不活体继承。
- **省的计量**：Bundle 记录候选/实选/实际交付 token 量，使"省"成为逐次调用可审计的指标；Memo 指针清单机械过滤过期与被取代项。
- 交付验证在 `CT-PROJECT` 新增对应失败用例。仍留 memo 待裁决：派生索引的具体形态、检索融合策略、跨 Repo 传承。设计正文见 Context 横切正文，底稿与生态对照见 `.memo/design/context-design-20260819.md` 与 `docs/research/context-landscape-20260824.md`。

## 22. 信任模型收窄：三条底线不可关、外层笼子可选、CLI 即人（v0.13.0）

复审 v0.12.0 §16 引入的两项机制后改判。当时把 OS 强制沙箱写成受治理执行的入场券、把“一次性用户在场证明”当作防止执行体冒充人的机制，两者都把治理面必须成立的底线和宿主机层面的加固绑在了一起：前者让第一阶段在 macOS 上启动不了多数真实 Harness，并把容器反向推成事实上的桌面沙箱；后者让 CLI 变成需要二次交互的入口，与“Workbench 与 CLI 同一验证规则、CLI 是 P2 正面形态”相抵。

正确道路只有一条：**三条底线**在治理面成立且不可声明关闭——工具不是人（治理命令只有两类入口：经认证的场景客户端会话——Workbench、CLI 或按公开合同适配的第三方客户端——以及施工图走完后 reducer 提交的同一「完成 Task」命令；Harness 的产出只经 Result Proposal 通道进来，不是入口——只验入口，不判断客户端是被人还是子进程启动）；合入钥匙不进工具（HCTL 不向 Harness 交付集成与外部写凭据，合入目标 ref 的权威只在「合入 ChangeSet」命令与 Integration Receipt，Harness 绕过命令直接改写目标 ref 只回读为 expected target head 不匹配的 drift）；隔离工作树（每个 ChangeSet 独立 worktree 与单一 Write Lease）。在此之内 Harness 是普通的 Git 用户：可读 Git common-dir 与 refs，可 fetch、比对目标分支、在本 ChangeSet 分支提交——linked worktree 本就共享 common-dir，refs/对象层面原来也藏不住。**外层笼子是加固**：OS 沙箱、凭据网关代用范围、网络与工具接口白名单由 Worker Profile 声明、随 Execution Spec 冻结；该轮的实现安排由 agentd 按声明施加并记录为运行时事实，后来由 §29 取代。未声明不拦启动、也不得记录为已生效；已声明项是该次执行的要求，宿主施加不了则不激活并列出缺项——不声明即可启动，所以不是入场券回潮，只是不让下游把 Execution Spec 冻结的范围悄悄放宽。Docker 不作为第一阶段桌面部署或沙箱形态。威胁模型据此诚实收窄：未启用加固时，Harness 与同 OS 用户的其他进程处于同一信任域，合同只承诺三条底线在治理面成立。

本条撤销 §16 的“用户在场证明”与“OS 沙箱入场券”两项，并把 CT-AGENT 里按沙箱写的一串负例改成三条底线的正面陈述；对应合同句在 [Agent 模块合同](../spec/agent.md)的写入合同与运行时两节、[系统边界](../spec/system.md)的命令与跨服务正确性、外部权威副作用与安全边界三节，以及交付验证 B2/CT-AGENT。

### v0.13.0 收窄

核销记录（自 spec/README 迁入）

| 旧名 / 旧词 | 现状 |
| --- | --- |
| 用户在场证明 | 撤销：治理命令只有两类 actor 来源——可映射到 owner human 的 direct client/provider event 与施工图走完的 reducer；只验来源信封，不判断客户端是被人还是子进程启动，不引入复杂 RBAC |
| OS 沙箱入场券 | 降为可选执行加固：由 Worker Profile 声明、Execution Spec 冻结、Agency 报告为事实，已声明而宿主施加不了则该次执行不激活；三条底线（工具不是人 / 合入钥匙不进工具 / 隔离工作树）单独保留 |
| “不得读取目标 ref/common-dir” | 删：Harness 可读 common-dir/refs 并在本 ChangeSet 分支提交；直写目标 ref 不取得集成 authority，只回读为 drift |
| Engine 检查点 execution identity / engine attempt generation | 退出 Obligation 身份：Obligation 按 Run、节点与观察序号铸造，Engine 的 run ID/step 名只作关联键；代次、deadline、完成谓词只在账本 |
| Room 的“加密/降级”状态 | 不设：房间端到端加密状态是 Chat 端口绑定的 health 投影，不进不可变 binding，也不是 Room 的 lifecycle 值；可观察结果只在[连接合同失败表](../spec/connections.md#失败与恢复)登记一行，恢复动作是既有的「换绑」命令 |
| P0 中的第三方自身功能项 | 移出 P0：属选型资料判断或首次消费前的产品化；P0 只验 HCTL 实际使用的 API 与行为 |
| “后端无并发令牌则降级只读”“不能强制排他的 backend 只可观察” | 归还给依赖：任务后端的并发控制归后端，adapter 按能力用其前置、以回读为准；运行时租约与代次只在账本，后端排他原语是加固 |

## 23. 代次不在 Dagu：完成与评审都在 HCTL，Dagu 只当路标（v0.13.0）

§18 换用 Dagu 时留下一个阻断项：要求 `human.task` 完成 API 能隔离迟到的完成请求，否则 B4 重开选型。复审发现这等于把 Engine 当成半个权威——Obligation 身份、租约与超时都绑到“adapter 回读证明的 engine attempt/status generation”上，Run 完成还要等 Engine success terminal 回读。正确道路是：完成与评审只在 HCTL 账本发生。Obligation 由 control 按 Run、节点与观察序号铸造，deadline、候选切换、Verdict/Receipt 与 Run 完成谓词全部只依据账本；Engine 只是路标——control 观察它进入等待态就铸 Obligation，账本结果落定后再把路标推过检查点；路标被 Engine 自行推进、重试或读不到时，只把 Engine Execution Binding 标为分歧待对账，既不补足也不阻止任何判决。重复进入同一节点按观察序号产生新 Obligation，不再要求 Engine 提供可隔离的检查点身份。本条撤掉 §18 的 B4 阻断项，P0 的 Dagu 探针相应只验实际调用的 API（见 §25）。权威文本在 [Run 模块合同](../spec/run.md)。

## 24. Room 对控制面明文可读：不启用端到端加密（v0.13.0）

§12 把消息 content 交给 chat server 时隐含了一个没写出来的前提：控制面能按事件 ID 读到消息正文——冻结升格来源与 Context 锚点的 digest、本地萃取相关讨论、AppService 写结果卡与 homeserver 侧桥接都建立在这一点上。Matrix 的端到端加密（`m.room.encryption`）会把正文锁在客户端设备里，房间一旦开启，这些能力整体失效而账本并不知情。v0.13.0 把“房间对 control 明文可读”写成 Chat 端口绑定的准入前置：创建或绑定时以 fresh 房间状态回读校验，已加密房间拒绝；事后被开启加密视同 control 读不到正文，走与 chat server 不可用同一条 fail-closed 规则并标为需要关注，恢复只有换绑到未加密房间。这不是新对象或新状态值，加密状态只是绑定的 health 投影。隐私与敏感输入的答案由此收窄：敏感输入走安全通道，仓库级隐私靠 homeserver 访问控制与保留策略，不靠给房间加密。权威文本在 [Project 模块合同](../spec/project.md#room-与消息)。

## 25. P0 只验证实际接口（v0.13.0）

复审开工前限时验证时发现，P0 条目里混进了大量第三方产品自身的功能项：Dagu 的重启与备份恢复、tmux 在六个 Harness 下的颜色/粘贴/复制/组合键/全屏 TUI 矩阵与 `#5510` 卡死回归、Tuwunel 在 macOS 的容器/VM 形态、内存配置与 RocksDB/media 一致性备份、Vikunja 的备份恢复，以及 tmux 动态库/terminfo/许可文件的最小化。这些不是 HCTL 的假设，是在替第三方验轮子。本轮把 P0 收窄为只验证适配器、受控端口或 agentd 实际调用的 API：Dagu 的 DAG 控制 API 与 `human.task` 等待/完成/回读；tmux control mode 的 owner-only socket、观察者扇出、输入/resize、ID 稳定与查询应答；Matrix 的账号与房间管理 API、AppService、按事件 ID 读正文与房间加密状态回读；Vikunja 的卡片/分组读写、条件写入可用性、webhook/轮询与身份稳定。事务 ID 幂等、事件顺序、排序令牌、背压这类是依赖自己的合同，HCTL 拿来用、不替它验——同一原则也删除了合同里两处替依赖立法的句子：任务后端的并发控制归后端（adapter 按能力用其前置、以回读为准），运行时后端的排他原语只是租约之上的加固。第三方自身功能分两类处置：发布物形态、键盘协议子集、footprint 之类属选型时的资料判断（已记在实现证据）；备份恢复、一键启停、TUI 矩阵、分发 commit 固定与打包最小化属对应场景首次消费前的产品化项（B1/B2/B4）。本条改判 §19 中“最终分发 commit 必须通过六个 Harness 矩阵与卡死场景阻断测试”的时点归属（B2 前产品化，不再是 P0），不改变任何选型。权威文本在[交付文档](../delivery.md#开工前限时验证)。

## 26. Context 投喂三档与 Run 内接力（v0.13.1）

2026-08-26 所有者围绕 Context 的两轮讨论收束为三处补充，讨论记录见 `.memo/design/context-feeding-20260826.md`。

- **切法从"四个依赖服务器"改为"执行体够不够得着"。** 开工包每样东西只有三种给法：每次必用且放得下的内联；可能用且执行体用自身工具够得着的给指针加摘要；够不着的开工时物化、运行中走受限召回。指针只指向 Git 对象与 worktree 路径——指向账本或任务后端的引用执行体解不开，不是指针。分档依据是成本：贵的不是指针而是那一轮往返（全上下文重读、工具输出永久驻留、串行延迟），所以规则是少一轮而不是少一个指针；反过来"全内联"正是同类工具固定前缀过重的主因。
- **补一类传承：同一 Run 内的接力。** 原有两类（聊天史萃取、父子传承）没有覆盖 producer → reviewer → 返工之间横向传什么。它的存储是账本与 Git，不是工作流引擎；评审席位拿被评版本（必用，预算内内联）、返工席位拿 Verdict 正文（账本记录物化，Git 结晶副本只作指针）、备用尝试拿同一份包加旧 ChangeSet Revision 指针；没提交的对继任者不存在。Terminal 场景由此定下：harness 之间只有结晶过的东西能过去，trace 不自动进任何人的上下文。
- **补一个来源：绑定 Task 的任务后端评论线。** 它和聊天史同性质——讨论、执行体自己翻不到——所以走同一条萃取阶梯，且整条结构相关、第一级命中、以 Snapshot ref+digest 冻结；改契约的内容仍只经「采纳契约」进 Task Revision。Kanban 与 Workflow 的其余 metadata（stage、优先级、负责人、排序）不进上下文，依赖/阻塞关系只以相关 Task Revision 指针进骨架。

落点：[Context 横切正文](../context.md)新增投喂三档与 Run 内接力两节；[Project 合同](../spec/project.md#context-memo-artifact)记录投喂档、指针可达范围与评论线来源；[Run 合同](../spec/run.md#request重试与-gate)记录 Seat Bundle 的接力内容；[Task 合同](../spec/task.md#契约与来源)记录评论线快照冻结。

## 27. 运行时 provider：Terminal 场景同构化（v0.13.2）

2026-08-29 的四轮裁决先把 Terminal 从第一方特例改为运行时 provider 受控端口，由 agentd 拉齐治理、tmux 作为内置原语、Herdr 作为候选；随后明确 provider 不含治理权威，栅栏回显只是能力声明，并把边界从 mux 上移到能够派出执行体、持有现场和报告恢复等级的执行服务，中文对照定为“派出方”。这条演进保持“供应端与 Participant 身份正交”，其实现安排后来由 §29 的 Herdr 方案取代；现行落点见 [Agent 设计正文](../agent.md#agency-与-herdr)、[Agent 合同](../spec/agent.md#运行时与观测)和[交付文档 P0 第 2 项](../delivery.md#开工前限时验证)。细节见本仓库 Git 历史。

## 28. 中间方案：Agency 定名并拆分 agentd（v0.14.0；已由 §29 取代）

2026-08-29 的 v0.14.0 中间方案把 Terminal 供给侧定名为 Agency，并让 agentd 退场：派遣归独立 `hctl2-agency`，治理与终端网关归 control，现场保管归工具箱；同轮把“讨论不落盘，拍板才提交”写入工作纪律。
`hctl2-agency` 与控制面终端网关的实现安排随后由 §29 取消，本节只保留当时的分工与命名转折。细节见本仓库 Git 历史。

## 29. Herdr 直接实现 Agency，并固定各模块的 provider 替换边界（v0.14.1）

2026-08-29 所有者继续裁决。Agency 保留为 Agent 模块 / Terminal 场景的供应端角色，但第一阶段不再建设 `hctl2-agency`，也不保留 `hctl2-agentd + tmux` 的并行方案；Herdr 直接按规格启动 Harness，持有进程、PTY 和终端会话，并提供 API 与原生 TUI。`hctl2-control` 只保留 Herdr 适配代码和 HCTL 自己的权限、租约、代次、审计、结果准入与恢复等级判断，`hctl2-tool` 负责 worktree/Git 现场。Herdr 当前不能执行的输入 fence、原生输入记录、事件 sequence/gap、退出与停止回读必须按实际能力降级或明确不支持，不能在 HCTL 里再写一套终端服务来补成另一个产品。

同一轮明确了四个默认二进制都不能成为产品合同。HCTL 不增加一个跨模块的通用 shim 服务，而由每个模块自己的受控端口和薄 adapter 隔离实现：Chat Room 使用 Matrix 协议，Tuwunel 可换成其他 Matrix homeserver；Kanban 通过 task backend adapter 接 Vikunja、GitHub 或 Linear；Workflow 以 HCTL 的 Workflow Revision 中间表示接 Dagu 或其他引擎；Terminal 通过 Agency adapter 接 Herdr 或未来官方远程 Agent。Workbench 只依赖 HCTL 场景合同，供应端原生 UI 只是可选 content 客户端或诊断工具。飞书、Slack、Discord 等聊天平台的互通属于 Matrix homeserver/bridge 生态，HCTL 只维护桥接身份映射，不逐个平台实现聊天 adapter；这与 HCTL 自己需要实现 GitHub/Linear task adapter 和未来远程 Agency adapter 是两类责任。

此前对 tmux、Zellij、shpool、Termio、tty7、cmux、Pilotty 与 Herdr 的源码、发布物、资源占用和行为验证继续作为选型证据、容量基线与回归用例，不再代表产品同时维护多个终端实现。当前决定见 [Agent 设计正文](../agent.md#agency-与-herdr)、[Agent 合同](../spec/agent.md#运行时与观测)、[三面架构](../architecture.md#避免供应商锁定)和[交付文档](../delivery.md#开工前限时验证)。

## 30. Workbench 桌面壳改选 Tauri 2（v0.14.2）

2026-08-30 所有者拍板。依据是桌面壳重开调研与所有者主开发机（Ubuntu 26.04 / GNOME 50.1 / Wayland / NVIDIA Quadro P620）的实机探针：Tauri 2.11.5 全项通过——Canvas/WebGL、10 个 xterm.js WebGL 终端、长列表滚动、最大化与中文 IME 均可用，候选窗定位由应用侧 caret 修正解决并已反馈上游——“WebKitGTK + NVIDIA + Wayland 在主开发机可判死”的文献推断被实测证伪；GPUI + gpui-component 同机通过；Flutter 因默认应用最大化稳定 SIGSEGV 暂时淘汰，等上游修复；Electron 44 可用但原生 Wayland 有顶边闪动，此类环境须回退 X11。

正式选型改为 **Tauri 2 + TypeScript/React 主选，GPUI 原生备选，Electron 安全网**。React 场景代码保持普通浏览器可运行、壳专有代码限制在薄壳与打包层的既有约束不变，四场景不随壳切换重写；已选 UI 轮子（Tiptap、React Aria、React Flow、xterm.js）全部保留。原 E-WORKBENCH-SHELL 的探针、产物抽样与权限边界证据继续有效，原“重开门槛”转化为反向回退条件：Tauri 整窗 P0（WKWebView 与承诺的 Linux WebKitGTK 基线、CJK/IME、一窗十终端、`.deb`/`.rpm` 与升级路径）失败即回退 Electron，不下沉 Wry 自研。为复用 GPUI 生态代码而将 `hctl2-workbench` 改为 GPL 的议题随之搁置。合同层同步：[系统边界](../spec/system.md#安全边界)的桌面壳安全条款改为壳中立——Tauri 的 capability/permission/scope 声明与 Electron 安全网的 sandbox 配置并列为两种发行形态各自的固定要求。证据见[桌面壳证据](../../research/workbench-shell.md#e-workbench-shell)与[重开调研](../../research/workbench-shell-reopen-20260826/README.md)。

## 31. 客户端没有等级，动作合同成为真正边界（v0.15.0）

2026-08-30 所有者拍板。此前把界面分成“治理客户端、content 客户端、诊断界面、带外终端”虽然守住了账本权威，却错误地让客户端产品类别承担了权限语义：Workbench 看起来比四个原生客户端更高一等，Vikunja/Herdr 的正常用户动作也被一概降成只读或带外。新的裁决是：**Workbench、CLI 与 provider 原生客户端没有等级；动作落到哪个端点、携带什么来源/目标/版本/幂等依据，才决定它按哪份合同处理。** Workbench 可以理解为把四类客户端、联合投影、跨模块导航和 HCTL 公共命令入口装进同一个桌面，它不是内核；关闭或不安装 Workbench，control、CLI、provider 和已启动执行都继续工作。

系统合同据此把动作分为 content 写入与观测、human 命令请求、运行时输入、Result Proposal 和不支持的 provider mutation。provider event 可以在模块明确允许时表达 human 命令请求，但必须归一到与 Workbench/CLI 相同的 command draft 和 reducer；这不是让 provider 数据库获得治理权，也不是建立一套通用 CRUD/shim。第一阶段单用户只需把 direct connection 或 provider account 稳定映射到 owner human，不建设组织/RBAC；仍必须保留来源类别、目标、expected version/generation、幂等键、cursor/gap 和 fresh readback，HCTL service、模型与未知 actor 不能自报为 human。

四个模块按实际用户路径分别裁决：

- Chat：Matrix 客户端和 Workbench 都可正常写消息 content；普通消息、反应、mention 和模型建议不含完整命令语义，不能自动派发。将来只有 Chat binding 明确列明、绑定精确 source event 且能映射 human 的结构化动作，才可请求同一 HCTL 命令；第一阶段实际入口仍是 Workbench/CLI。
- Task：Vikunja 原生 UI 把卡拖入 Done 会在后端真实修改 `done`，并发出包含 `doer` 的 `task.updated` webhook；因此对已绑定卡片，这个明确动作在 remote revision/updated version、fresh readback 和规范幂等 tuple 齐全时可以请求同一个「完成 Task」命令。它仍不能直接写 lifecycle/Receipt，无契约、活动 Run、证据不足或漂移时保留“provider Done、HCTL 开放”的双重状态。
- Run：Dagu 的 UI 是完整管理界面，Start/Stop/Retry/Reschedule/Approve/Reject/Edit/Rename/Delete 会先改变定义或机械执行，无法让 control 在副作用前持久化 intent、撤销 Attempt/租约并安排停止。因此用户意图本身没有问题，但该接口顺序不适合作为普通 Run 入口；对已绑定 execution 的直接 mutation 只标记分歧，不能事后补造 Run 命令或 Receipt。
- Agent：Workbench Terminal 与 Herdr TUI 都是 Terminal 客户端。原生输入是有效的用户运行时输入，不是 Task/Run 结果；Execution Spec 必须在“全部输入受 descriptor/lease 管理”和“允许原生交互、接受 provenance/单写者保证不完整”之间冻结选择。Herdr v0.8.2 缺统一 writer gate 与逐次输入事件，所以前者关闭原生写入，后者如实降级，不再把所有原生输入笼统说成 drift。

这次变化同时移动 Workbench 定位、human 命令入口和 Task/Agent 的可接受行为，故从 v0.14.2 升为 v0.15.0，而不是补丁修订。实现证据分别见 [Vikunja 复核](../../research/task-backends.md#2026-08-30-provider-动作复核)、[Dagu 复核](../../research/workflow-engines.md#2026-08-30-provider-管理动作复核)与 [Herdr 验证](../../research/runtime/agency-runtime-validation-20260829.md#2026-08-30-原生客户端定位复核)。

## 32. 过强断言放宽：十一条边界回到用户确认（v0.15.2）

2026-08-31 所有者拍板。v0.12.3 评审沉淀的 13 条过强断言（[清扫待办](../../../.memo/notes/doc-cleanup-backlog-20260825.md)）中，两条已随组件退场核销，其余 11 条一次放宽。共同方向：把「绝不 / 永久 / 全部」形态的硬性禁令改回「默认安全 + 有权 human 显式确认后可越过」，或「底线保留、机制降为实现细节」——正确的路写清之后，错误 case 不必逐条堵死。

逐条落点：discovery 默认本地，联网探测可配置且不静默安装改配（系统边界）；危险动作默认 Preview 确认，普通命令可直接 Submit（系统边界）；不可证静默的旧 worktree/ChangeSet 默认隔离，human 确认后可接管、封存、采用或丢弃（Agent 合同）；清理默认保全，确认后可丢弃（Agent 合同）；单写者只保留三条底线——一个逻辑 writer、已确认副作用不重复、旧结果不覆盖新结果，锁路径与推进机制降为实现细节（系统边界）；存储拓扑与 Git 内部命名空间用法降为实现选择（系统边界）；Repo 挂接证据缺失或冲突时展示证据、由用户确认归属（系统边界）；Project 归档只被非终态 Run/写入型 Invocation、活动租约与未决外部写副作用阻塞，开放 Task/Request/Scoped Room 随归档转只读（Project 合同）；Scoped Room 可显式以 abandoned/no-decision/superseded 结案归档（Project 合同）；Context 萃取与相关性门缺省全本地零模型，配置 small-brain 后可模型辅助（Project 合同与横切正文）；活动 Run 期间可采纳新 Task Revision，Run 只按冻结 Revision 完成、current 前移时按分歧拒绝（Task 合同）。

各族新增失败用例见[契约测试矩阵](../contract-tests.md)。

## 33. hctl2-tool 定界为现场执行者（v0.15.3）

2026-08-31 所有者拍板（大修停车位 #2）。定界锚定三个业界成熟模式：git 的 porcelain/plumbing 分层（上层只编排底层，不重实现）、pre-commit 的声明式检查编排（配置进仓库、工具零自建）、credential helper/ssh-agent 的凭据代签（钥匙不离代理）。

hctl2-tool 收束为**现场执行者**五项：worktree/ChangeSet 物化与隔离、已持久化意图的执行与回读、现场 OS 锁与 fence、封存保全、判决结晶副本写入；进程级动作一律转调业界工具，零重实现。lint 与代码检查移出第一方组件——仓库声明式配置由 harness 本地执行、CI 作强制层；远端 SCM 副作用维持 executor = adapter。治理编排唯一归 control；control 可搬迁（换机、上服务器），现场职责钉在 Repo Instance，两者不得合并——这是「一人多机连同一本账」的物理前提。

同轮显式拒绝停车位 #1：Herdr 无统一 writer gate 不再作为缺口立项。用户绕过 HCTL 直改任何 provider（原生终端输入、看板拖卡、聊天直发）是同一类外部事实，一律按对账与能力条件句降级处理，不建隔离机制；合同自 v0.15.0 起已如实覆盖，无需改动。

## 34. 小修订台账

本文件只为重要设计变更单列章节，例如核心边界、实现选型或权威归属发生变化。词汇、词形与概念清理类修订各记一行，细节放在合同层清理表，不再单独成章：

| 版本 | 日期 | 内容 | 详情 |
| --- | --- | --- | --- |
| v0.10.3 | 2026-08-19 | 词汇清扫：RuntimeBackend、TaskSource、WorkflowEngine、HarnessAdapter 四个未入册高频名降级为描述语或端口种类；交付文档定为与合同层同侧 | [清扫表](#v0103-清扫) |
| v0.11.1 | 2026-08-21 | 词形收敛：25 个驼峰名改带空格专名、16 个 `*Intent` 命令名改动宾语义名、状态值改中文语义名；“不含任何代码标识符”的绝对化后被 v0.12.0 复审收窄（协议/schema 字段与外部原名保留原形） | [词形表](#v0111-词形收敛) |
| v0.12.2 | 2026-08-24 | 概念清扫：Room Event 除名、Task Operational State 降级为操作投影、无法证明执行身份时统一标为“丢失” | [清扫表](#v0122-清扫) |
| v0.15.1 | 2026-08-31 | 全库文档大修：门户收束、设计层与合同层同构合并、禁令按白名单三分、CT 矩阵拆出 contract-tests.md、来时路折叠与历史表迁入；不改合同语义 | [大修施工图](../../../.memo/design/doc-overhaul-20260830/README.md) |
| v0.15.4 | 2026-08-31 | 路标停更与迟到观察不铸新 Obligation：把隐含边界写成明文（spec/run 铸造条件、连接合同失败表、architecture 半句），已铸义务照常判决 | [大修停车位 #4](../../../.memo/design/doc-overhaul-20260830/parking-lot.md) |

## 35. 当前设计

这条来时路最终收敛为四个权威模块、四个对仗 Scene、共享但受控的连接与执行机制。阅读当前设计时，应从[愿景](../vision.md)开始，再读 [Project](../project.md)、[Task](../task.md)、[Run](../run.md)、[Agent](../agent.md)，再查看[连接合同](../spec/connections.md)、[系统边界](../spec/system.md)和[第一阶段交付](../delivery.md)。本文用于解释这些边界为什么存在；发生冲突时，它不覆盖任何当前规范。

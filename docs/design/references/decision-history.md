# 从 HCTL 到 HCTL2 的来时路

> 状态：Informative · 对应草案 v0.12.3 · 2026-08-24<br>
> 定位：本文只解释关键决策为什么转向，不定义当前对象、状态、命令或交付范围。当前合同以[设计地图](../README.md)及其链接的模块、连接和系统文档为准；可复核的版本与源码依据见[实现证据](./implementation-evidence.md)。

HCTL2 不是从一张完整产品蓝图一次推导出来的。它从 HCTL1 的治理内核出发，先面对多 Harness 终端与工作树的现实问题，再逐步把用户意图、任务承诺、受治理执行和物理运行时分开。下面记录的是这条边界收敛路径，而不是另一份规范。

## 1. HCTL1：Git-native 治理内核

HCTL1 首先解决的是“谁可以对哪个精确版本作出什么裁决”。它把协调事实放在 Git 上，形成 append-only 事件、local/remote CAS、claim OID fence、精确 `{base, head}` Verdict、quorum、可重放的 squash merge Receipt，以及事实不完整时 fail-closed 的 level-triggered reconciliation。这套可执行内核证明：治理可以依靠稳定身份、精确版本和可重放证据，而不依赖常驻 daemon、数据库时钟或界面状态。

这条谱系没有被 HCTL2 丢弃。HCTL2 延续了版本化证据、claim/fence、法定票数、Receipt 和对账思想；但 HCTL1 的对象模型和 Git 事实布局只适合它当时的窄范围，不是 HCTL2 的存储或产品蓝图。具体基线与限制见 [`E-L2-HCTL1`](./implementation-evidence.md#e-l2-hctl1--hctl1--yesmehctl)。

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

Scene 是投影和操作面，不是第五个 writer。Room 的 repo/project/scoped 拓扑也与控制拓扑正交：共享协作不要求某个 Harness 永久在线，终端重连更不能接管 Room 或 Project 的身份。

## 5. Workbench、第三方客户端与受控端口分离

Workbench 随后被明确为四个 Scene 的集成客户端，而不是新的领域层。自建它的理由不是重写通用 UI，而是四模块的导航结构无法无损套入任何 donor 的会话、终端或工作树主导航。CLI、Workbench 和适配后的第三方 Scene UI 都通过同一类 Query、Preview、Submit、Subscribe 合同工作；provider 原生 UI 若只操作本系统 content，仍只是 content 客户端或诊断面。关闭任何客户端不改变领域事实，也不授予额外权限。

与此同时，Chat、任务源、workflow engine、harness 和运行时后端被收敛为受控端口。它们提供外部能力、报告版本与降级方式，但不能凭平台自身的 Session、Issue、workflow task、pane 或数据库取得 HCTL 字段权威（第 12 节后来把这条精确化为“可拥有 content、不可拥有治理”）。一个产品可以同时提供原生客户端和受控端口，例如第三方看板既展示 Task 又承载部分外部字段；此时 client binding 与 authority binding 仍须分开，避免“能显示”被误写成“能决定”。

## 6. 当时的 Conductor 只拥有机械状态（后由 §18 取代实现选型）

当时引入 Conductor 的目的，是复用耐久 external task、wait/timer、retry 与历史恢复，而不是把 HCTL 的语义交给工作流引擎。当时固定的边界是：Conductor 保存机械工作流位置，HCTL control 领取外部任务并建立 Obligation/Seat/Attempt，工具箱与 control 校验精确版本、证据、权限、Gate 和 Receipt。

因此当时的 Conductor 不选择 Harness、不创建逻辑 Seat、不解释语义驳回、不计算 HCTL quorum、不签发 Receipt，也不直接写 Git 或第三方系统。机械 retry 与 HCTL 的替代执行、返工和 regate 是不同身份；这条权威边界在 §19 换成 Dagu 后仍保留。

## 7. 从 prompt 约束到机械终结权

早期实践曾试图通过 prompt 要求 Harness 在“确实完成”时才报告完成，并期待模型自行维持 Task、Run 与执行结果之间的边界。实际使用表明，这种约束不能稳定提供终结权：模型输出仍是受上下文影响的建议，Harness 也只能观察本次执行，无法替 Task 的冻结验收合同作出权威裁决。由此形成一条更一般的教训：凡是能够由身份、版本、状态与证据机械判断的事项，就不再交给 LLM 灵活解释。

当前合同因而把 Task 完成收窄为两条可审计路径。其一是有权 human actor 从 Kanban Scene 提交「完成 Task」命令；其二是绑定该 Task Revision 的 Workflow 正常完成后，由 reducer 按冻结规则提交同一个「完成 Task」命令。第二条路径不是让 Run 的完成态静默传染给 Task：Task 仍须重新校验版本、证据和来源状态，校验失败便保持开放；Workflow 的失败、取消或替代终态也不会终结 Task。Task 的取消则仍由有权 human actor 显式决定。

Harness、Participant、模型输出和 runtime signal 可以提供 Result Proposal、Artifact 与 Evidence，却不成为 terminal actor。这个 pivot 把过去依赖提示词维持的行为期望，下沉为工具箱与 control 可以拒绝的 actor provenance、命令与 reducer 规则。

## 8. 从开放 Agent mesh 到受控协作边

First Tree 证明了持久 Chat、稳定且显式的 recipient，以及发生在共享对话中的可见 handoff，能够维持多 Agent 协作连续性；这些优点被保留。它也允许 Agent 在运行中 `invite + send`，并由接收者继续寻址第三个 Agent，使协作图临场生长。HCTL2 没有把这后一项作为普通 Room 的默认能力，因为创建下一条执行边同时改变参与者、Context 披露、权限、预算与终止条件。

因此当前设计只承认两类边：普通 Chat Room 的临场协作边由有权 human actor 创建，Workflow 的执行边由 reducer 按冻结的 Workflow Revision 创建。Agent-authored message、Result Proposal 或模型总结可以建议下一位 Participant，却不能自行 cue 新 worker、扩大 fan-out 或递归委派。在 Chat Room 中，人本来就处于讨论焦点；系统应把建议变成可一键批准、自动携带引用与 Context 的 handoff，而不是让人重新复制和转述内容。

这不是否定多 Agent 协作，也不是宣称 Agent-to-Agent 永远无效；它把开放的临场 mesh 换成“人拥有即时星形拓扑、状态机拥有预授权有界图”的当前产品选择。First Tree 的具体采用与拒绝证据见 [`E-L4-FIRST-TREE`](./implementation-evidence.md#e-l4-first-tree--first-tree)。

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

v0.9.1 处理概念层面的同类问题。全量清点暴露出 75 个以上具名概念平铺在设计正文里——很多是执行路径上的一个步骤、一份冻结字段组或一个状态值，被顺手建成了“类”。修订按三个动作收口：设计正文与合同层分离（`docs/design/spec/`，设计层只用核心产品词加日常语言）；概念按六族归类并归并（复合名 41 → 28，如派发规格、物理运行时、外部副作用意图各归一名）；场景概念对齐外部标准（Chat ↔ Matrix/Slack，Kanban ↔ Linear/GitHub，Workflow ↔ Conductor/BPMN，Terminal ↔ PTY/tmux/ACP），自造词只保留差异化语义。这轮方法论的完整记录见 `.memo/design-doc-method-20260819.md`。

## 12. 场景数据的三分：metadata / content / artifact

v0.9.1 之前，四模块操作账本整体放在 Repo Instance SQLite 里（第 9 节记录的是这一时期的设计，并非当前落点）。随后的多设备讨论暴露了一个产品语义错误：同一个人在两台机器各 clone 一份仓库，Room 的协作历史却被锁在单个 clone 的本地账本里——clone 丢了，协作记忆就丢了。`.memo/room-ground-truth-20260819.md` 曾比较四个候选，当时把“Matrix 作为权威”判为违反端口纪律，倾向用户级 hub。

`.memo/scene-data-model-20260820.md` 给出了更精确的刀法：每个场景的持久数据分三类——metadata（治理元数据：身份、绑定、授权、判决）、content（场景内容：消息、任务卡与流转、机械执行历史、会话转录）、artifact（结晶：决议、冻结契约与施工图、凭证链、代码变更）。Workflow 与 Terminal 两个场景本来就这样运作——工作流引擎拥有机械历史，harness 会话拥有转录，HCTL 只留绑定与治理；三分法把这条规则推广到 Chat 与 Kanban，补掉了不对称。

这次转向显式推翻了两条旧结论。其一，“平台不能成为第五事实源”精确化为**可以拥有 content、不能拥有治理**——room-ground-truth memo 对 Matrix 候选的否决在三分下失效：平台拥有的是记忆，不是裁决。其二，metadata 账本的归属从 Repo Instance 上移到用户级控制面（一人多机连同一个控制面），Repo Instance 只剩代码侧的物理事实。该 memo 的两个遗留分叉同时裁决：用户级 hub 以“控制面即 hub”的形式采纳；用户级“总入口对话面”否决——用户进入产品就在某个 repo 之下，这是显式设计决定，不是遗漏。

这不是把治理交给平台，也不是回到 HCTL1 的“Git 承载一切”：判决仍只在 metadata 层产生，冻结摘要仍是 content 与治理之间的防火墙（第 9 节的继承表原样成立）。v0.10.1 对落点做了一处修正：Board 从 Project 级上移为 Repo 级——一个 Repo 一个 Board，Project 是板上的分组实体，Task 是卡片；这让 GitHub issues 这类天然 repo 级的后端直接对齐，也让支持父子任务的本地服务器以“任务–子任务”承载 Project–Task。v0.10.2 进一步取消了仓库侧的独立物理账本：现场记账本可由 metadata 账本、Git 与运行时观测推导，单立一本账是把实现细节写进合同——clone 本地从此只有 OS 锁与可丢弃缓存，实例注册与现场记账并入用户级账本，存储从“两本账 + Git”收敛为“一本账 + Git”。另有一条词汇裁决随本次转向生效：自 v0.10.0 起，“Agent”一词专属第四模块（原 Harness 模块更名），“Harness”专指编码代理工具这一系统角色，散文中的 AI 协作者用 Participant 表述。类别的权威定义见[合同层总则](../spec/README.md#三类数据)；候选系统与限时验证见[交付文档](../delivery.md)。

## 13. 实现计划：P/B 双表与 P0 选型（v0.11.0）

三份外部评审一致指出开工前限时验证是“事实上的 phase 0”。v0.11.0 据此给交付文档补上施工视角：P0–P6 实现阶段表与 B0–B6 自举阶梯一一映射——P 表回答先建什么，B 表回答什么时候敢切换事实，建完不等于敢用。P0 选型同轮拍板：conductor-oss；Vikunja（git-bug 降为记录在案的对照）；运行时后端 Zellij（同栈、结构化接口、内建 web 客户端默认关闭、可作额外访问门；tmux 为降级方向，验证新增 headless 查询应答与增强键盘协议两点）；chat server 在 Tuwunel 与 Continuwuity 并列验证后定夺。远端任务后端验证移出 P0、延至 P4 后按需。

## 14. chat server 定夺 Tuwunel（v0.11.1）

同轮拍板 chat server 产品方向为 Tuwunel（接口更 API 化、与 Synapse 参考实现兼容性更强；单二进制预编译包、运维压力低；AppService 注册程序化），Continuwuity 记录在案备选；精确发布版本、存储后端与 build features 仍由 P0 验证后固定。该轮同时完成的词形收敛（驼峰名、`*Intent` 命令名与状态值拼写）见[小修订台账](#26-小修订台账)。

## 15. 四段施工序与组件正名（v0.12.0）

v0.12.0 最初把 P0–P6 收敛为按三面架构分层的四段：P0 立营（先让四个执行面系统可运维）→ P1 备装（agentd + 工具箱）→ P2 接钥匙（control + CLI，B0–B5）→ P3 装门面（Workbench + B6）。它也把 `hctl2-core` 正名为机械工具箱 `hctl2-tool`，把 agentd 在组件表中正名为 `hctl2-agentd`，并写下“commit 署名、lint、PR 正文拼装、memo 写入、有效变化侦测等机械工作不交给模型”的施工纪律。

同一基线的复审保留四段骨架，但修正了三处过强表达：P0 改为限时、可丢弃的实现探针，content 系统在 P2 首次被纵向切片消费时才产品化；P1 的 standalone 工具只能辅助开发，B2 才是第一次真正自举；P2 的完整治理入口是公共 CLI，Matrix/Vikunja 原生界面只验证 content 互操作，Engine console 与裸 Zellij 分别只是诊断和 break-glass，除非适配器实现公开合同，否则不能统称“第三方场景客户端”。同轮也收窄了两条方法论：“外部无对应”是差异化语义的证据而非账本存储清单；自造语义名不冻结代码词形，但精确协议、字段、格式与外部原名可以保留。原生优先的打包方向不变，精确版本、后端与 build features 在 P0 证据通过后固定。

## 16. 缺口审计：转向传播与五个新机制（v0.12.0 审计）

v0.10–v0.12 的数次横切转向（用户级 control、Repo 级 Board、三类数据、Agent 正名、四段施工序）各自合理，但旧摘要、连接与验收仍残留前一版假设。一轮根因审计据此收口（方法与全部发现见 `.memo/v07-v012-retrospective-and-architecture-audit-20260821.md`）：恢复缺失的交接与恢复合同，消除同一事实的两种解释，不恢复 v0.7 的对象爆炸。其中一部分是纯反漂移修正，如固定内核的“以仓库为边界”残句改为用户级 command service，以及三处“后端不可用时命令照常/入口降级”统一为“需要 fresh readback 的命令 fail closed，不依赖者可继续”。

除传播既有决定外，这轮引入了五个此前不存在的机制，在此补记决策：

- **用户在场证明**（v0.12.4 撤销，见 [§22](#22-信任模型收窄三条底线不可关外层笼子可选cli-即人v0124)）：以 human provenance 提交 Task 终结、普通 Room 执行边、扩权、安全输入或集成授权时，认证入口必须给出一次性、绑定规范命令摘要的用户在场证明。动因是旧威胁模型自相矛盾——execution principal 复制 human payload 调用公共 CLI 即可冒充人，单靠“human actor 才能终结 Task”挡不住。权威文本在[系统边界](../spec/system.md#命令与跨服务正确性)。
- **受治理 Harness 的 OS 沙箱入场券**（v0.12.4 降为可选加固，见 [§22](#22-信任模型收窄三条底线不可关外层笼子可选cli-即人v0124)）：第一阶段受治理执行必须运行在操作系统强制的沙箱中，凭据只经网关代用；不能强制这些边界的候选不得作为受治理执行启动。这是把“managed Harness”与“不隔离同 OS 用户”的互斥承诺改成一致，代价是 P0/B2 新增阻断性工程验证。
- **Run 正常完成谓词与 Task claim 双态**：Run 进入正常完成前须机械满足 required Obligation/Seat/Gate/output、Receipt、Attempt 撤权与外部副作用收口；Task 对 Run 的绑定以 active / completion_pending 双态消除交接竞态。填补“Engine 报告完成即 Run 完成”的空洞。
- **挂接 Repo Instance**：Repo 与 clone 之间补显式身份链——工具箱无副作用读取 Git identity、control 预览后入账；remote URL、目录名或碰巧相同的 HEAD 不单独证明身份。这是用户级 control 转向早该有的配套。
- **账本备份集合同**：备份必须是唯一 writer 协调的一致快照（含各层 generation 与账本引用的不可变定义字节），恢复保留账本身份、推进全部代次、令旧凭证失效。把既有的“账本必须备份”从一句话落成合同。

方法论记录：这五项随审计一次合并落地，未按「真语义变更单独立项」拆批——同根问题跨文件归并可取，但代价是回滚粒度整体化、新机制险些没有决策记录（本节即补记）。后续审计若再引入新机制，机制本身应单独立项提交。

## 17. 适配器诚实合同：终局结果、观测截断与派生谱系（v0.12.0 补充）

Agent 模块的观测合同原本回答的是“信号如何仲裁”（优先级阶梯、观测不推进领域结果），但留着两个物理层必须表态的空洞：进程干净退出算什么？上报中断后的残缺事件流还算不算历史？本轮据 LobeHub 源码审计（[E-LOBEHUB](./implementation-evidence.md#e-lobehub)，其 heterogeneous-agents 适配器层的经验证做法）补齐三条 harness 适配器义务：

- **终局结果契约**：每个接入端口声明其终局结果事件；执行体进程正常退出但缺少该事件时，适配器必须合成类型化协议错误，不得默认成功——静默死亡不能冒充交付。由 control/agentd 主动取消导致的退出归因为取消，不上报为执行失败。
- **观测流完整性**：观测上报通道失败时，只能显式标记该执行的观测截断并终结事件流；有缺口的事件流不得冒充完整历史。
- **派生谱系保留**：harness 内部再派生的子执行体事件必须携带稳定的派生谱系引用，不摊平进主执行流。

同轮补一句能力边界：semantic resume 可以用自有观测留痕重建续跑输入，重建物按投影处理，不进入权威记录——恢复能力不再依赖厂商保留会话文件。术语取“终局”而非“终端”，避免与 Terminal 通道词族冲突。权威文本在 [Agent 模块合同](../spec/agent.md)的「运行时与观测」与「终端通道」。本条按 §16 的方法论教训单独立项提交并当轮补记决策。

## 18. workflow engine 从 Conductor 改为 Dagu（v0.12.1）

2026-08-23 重新按源码而非 README 审阅 workflow 候选后，第一阶段选型从 Conductor 改为 **Dagu**。本节与 §19 的两项选型改判构成 v0.12.1；该轮当时未随文件重 stamp，随 v0.12.2 补记。判据收敛为三条：Workflow Revision 必须是声明式事实源并可机械 lint schema、引用、Profile 与图结构；不要求一般性证明 loop 终止；本机运行优先单二进制、文件或嵌入式持久化，避免为机械状态引入 JVM 与数据库/队列组合。Conductor 当前版其实已默认 SQLite，旧记录“没有 SQLite”已经过时；它的逐任务 poll/complete 仍比 Dagu 更贴近被驱动模型，但总体分发和 footprint 仍较大。

Dagu 也不是天然的 external-worker engine：普通 step 会自行执行。采用边界因此固定为 HCTL JSON 经 compiler 生成受限 Dagu YAML，只使用依赖/条件/等待等机械结构和无进程 `human.task` 检查点；control 观察等待态，在 HCTL 结果先落账后经 API 完成它。Dagu 不获得 Harness 选择、Seat/Attempt、语义 Gate、quorum、Receipt 或外部副作用权威。公开 completion API 不能携带调用者预期的 engine attempt generation，迟到请求是否可能推进 retry/repeat 后的新检查点是 B4 前的阻断性 P0；失败就重开选型，而不是自建第二个引擎。（v0.12.4 改判：代次不在 Dagu，见 [§23](#23-代次不在-dagu完成与评审都在-hctldagu-只当路标v0124)。）

这条决定取代 §6 和 §13 的 Conductor 实现选型，不推翻“引擎只拥有机械状态”的边界。Dagu、Conductor、Windmill、Kestra、Direktiv、Serverless Workflow Synapse、Flogo、Step Functions Local 与 SCXML/XState 自建路线的源码证据和完整取舍，见 [`E-L2-DAGU`](./implementation-evidence.md#e-l2-dagu)；四个现役执行面依赖的版本、体积和运维账见[实现证据](./implementation-evidence.md#执行面已选依赖的运维与-footprint)。

## 19. 运行时后端从 Zellij 改为 tmux（v0.12.1）

2026-08-23 按源码、发布物和本机多会话实测复审 tmux、Zellij 与 shpool 后，第一阶段运行时后端改为 **tmux**。决定性的不是功能最多，而是 agentd 所需的最小、可控原语：tmux 有公开 control mode、稳定 pane ID、只读/不参与尺寸协商的观察客户端、`capture-pane`/`pipe-pane`、`remain-on-exit` 和明确的终端查询应答；control mode 的输出暂停/恢复与有界缓冲也给慢观察者隔离留下了可靠接缝。采用形态固定为 agentd 持有 owner-only socket 与唯一可写 control client，默认每个 runtime 独立 server，其他客户端经 agentd 扇出；裸 attach 仍只是 break-glass。

体积使取舍进一步明确。本机 Apple Silicon 基线中，tmux 可执行文件约 **0.95 MiB**，直接非系统动态库约 **1.45 MiB**；一个 server 承载十个 detached `/bin/sleep` session 时约 **3.7 MiB RSS**。Zellij 的 `zellij-no-web` 可执行文件约 **32.4 MiB**，一个默认 detached session 约 **89.7 MiB RSS**，十个约 **841.6 MiB RSS**；它的结构化接口与原生跨平台优势不足以抵消多 Harness 常态下的成本。shpool 可执行文件约 **4.04 MiB**、十个空闲 session 的 daemon 约 **23.1 MiB RSS**，但当前公开合同仍以单客户端 attach 为中心，没有 headless 终端查询应答、可承载 payload 的结构化事件、多观察者扇出或成熟的慢客户端背压；采用它会迫使 agentd 自建终端模拟、快照/重放与流量控制，等于把难题搬回自身。

tmux 也不是无条件通过：它支持 CSI-u/modifyOtherKeys 子集，不实现完整 Kitty keyboard protocol；`3.7c` 还有一个特定多窗格、快速滚动、copy-mode 与 resize 组合下的 [`#5510`](https://github.com/tmux/tmux/issues/5510) 卡死报告。因此 `3.7c / e476c123` 只是源码审阅基线，最终分发 commit 必须通过六个 Harness 的颜色、粘贴、复制、组合键、全屏 TUI、退出码，以及 headless 查询、背压和该卡死场景的阻断测试。完整源码证据、候选差异和测量口径见 [`E-L1-TMUX-RUNTIME`](./implementation-evidence.md#e-l1-tmux-runtime)。本条取代 §13 的 Zellij 实现选型和 §15 的裸 Zellij 表述，不改变“运行时后端只拥有物理会话、不拥有领域事实”的合同。

## 20. 桥接退役、结晶归位与概念清扫（v0.12.2）

一轮按最新方法论（三类数据、概念门槛）对 README 与设计/合同骨架的复查，收口了六件事：

- **自建聊天桥接退役（永久，不只是第一阶段后置）**：非 Matrix 平台（飞书、Slack、Discord 等）经 homeserver 侧的 Matrix 桥接生态在 content 层接入。依据是三条法的直接推论——聊天里不跑治理、记录不是命令，所以桥接纯属 content 层，而 Matrix 生态已有成熟桥接体系；HCTL 只保留 Chat 端口绑定中对桥接用户的身份映射策略。原“非 Matrix 完整聊天桥接”工作线与对应未决问题删除。
- **施工图结晶归位 Chat Room**：施工图（“干什么的计划”）从 Room 的塑形讨论中长出，不是任务流转的结晶；4×3 矩阵与统一律相应改判，并明确结晶归属以事实为准绳、不为对称硬填。施工图的对象与写入者仍归 Run 模块合同——结晶归属不随对象所有权走。
- **概念清扫**：Room Event 除名、Task Operational State 降级为操作投影、执行身份无法证明的收口状态统一为“丢失”（收口规则唯一定义在连接合同的失败表）；裁决与去向见[小修订台账](#26-小修订台账)与[合同层清扫表](../spec/README.md#v0122-清扫)。
- **content 客户端与治理客户端在 README 分离**：架构图改为 content 客户端（任意 Matrix 客户端、任务后端原生界面）直连 content 系统、治理客户端连命令服务；“任意 Matrix 客户端开箱即用、桥接交给 Matrix 生态”上升为正面能力表述。
- **P2 双手模式立为正面形态**：Workbench（P3）之前，治理动作走公共 CLI、场景内容走各 content 原生界面；mention 触发与 Trigger Preview 是治理客户端的能力——聊天文字不是命令，也不是能赋予 human provenance 的认证入口。交付范围表按 P2/P3 出门条件重切。
- **措辞修正**：产品原生核心从“以仓库为边界的控制面”改为“随用户走、按仓库划分语义范围的控制面”，与系统合同的用户级 command service 一致（§16 已修系统层，本次补愿景层残句）。

## 21. Context 合同裁决轮（v0.12.3）

按"逐条裁决、不整篇升格"把 Context 横切正文已拍板的设计落进合同（Project 模块合同的 Context 一节与交付验证），全轮零新增具名对象：

- **管辖与物化/指针两分**（随 v0.12.2 后的澄清先行落地，本节补记决策）：Context 交付的是开工 prompt，不代管执行体会话内自组装的工作上下文；物化只限聊天萃取、契约与范围、显式引用原文，Repo/Git 内容、Artifact、Memo、Skill 一律指针化由执行体自取；选择优先级与序列化顺序分离（稳定在前、变动在后）。依据：聊天史是执行体唯一拿不到也不该自己翻的来源。
- **萃取全本地**：全文索引与相关性门是可重建派生投影，不进权威账本；门只以账本事实为判定输入、不以消息措辞做路由，判定记为可审计观测；选材不消耗大模型 token。
- **压缩合同**：缺省关闭；small-brain 是经用户级定义机制固定 revision/digest 的模型引用（无新对象）；逐条记录 compressor、压缩率与原文 digest 且片段可回源；证据类内容永不压缩，违规 Bundle 拒绝交付；萃取/压缩产物可按（room、cursor 区间、消费者范围）作派生缓存跨调用复用。
- **前情提要**：滚动纪要为组装器机械触发、small-brain 增量折叠的派生缓存——逐条回源、不可作治理引用目标、不进账本、不由房间内模型书写；无 small-brain 时以近详远略裁剪代替（缺省态的缺陷是故意保留的结晶压力）；父 Room 纪要只作提升预览预填，不活体继承。
- **省的计量**：Bundle 记录候选/实选/实际交付 token 量，使"省"成为逐次调用可审计的指标；Memo 指针清单机械过滤过期与被取代项。
- 交付验证在 `CT-PROJECT` 新增对应失败用例。仍留 memo 待裁决：派生索引的具体形态、检索融合策略、跨 Repo 传承。设计正文见 Context 横切正文，底稿与生态对照见 `.memo/context-design-20260819.md` 与 `.memo/context-landscape-20260824.md`。

## 22. 信任模型收窄：三条底线不可关、外层笼子可选、CLI 即人（v0.12.4）

复审 v0.12.0 §16 引入的两项机制后改判。当时把 OS 强制沙箱写成受治理执行的入场券、把“一次性用户在场证明”当作防止执行体冒充人的机制，两者都把治理面必须成立的底线和宿主机层面的加固绑在了一起：前者让第一阶段在 macOS 上启动不了多数真实 Harness，并把容器反向推成事实上的桌面沙箱；后者让 CLI 变成需要二次交互的入口，与“Workbench 与 CLI 同一验证规则、CLI 是 P2 正面形态”相抵。

正确道路只有一条：**三条底线**在治理面成立且不可声明关闭——工具不是人（human provenance 只由经认证的 Workbench/CLI 会话赋予；Harness 适配器、受控端口、Room 消息、adapter payload 与模型输出都不是入口；agentd 交给受治理执行的执行凭据使其中发出的 CLI 调用以 execution principal 提交）；合入钥匙不进工具（HCTL 不向 Harness 交付集成与外部写凭据，合入目标 ref 的权威只在「合入 ChangeSet」命令与 Integration Receipt，Harness 绕过命令直接改写目标 ref 只回读为 expected target head 不匹配的 drift）；隔离工作树（每个 ChangeSet 独立 worktree 与单一 Write Lease）。在此之内 Harness 是普通的 Git 用户：可读 Git common-dir 与 refs，可 fetch、比对目标分支、在本 ChangeSet 分支提交——linked worktree 本就共享 common-dir，refs/对象层面原来也藏不住。**外层笼子是加固**：OS 沙箱、凭据网关代用范围、网络与工具接口白名单由 Worker Profile 声明、随 Execution Spec 冻结、agentd 按声明施加并记录为运行时事实；未声明或宿主不支持不阻止启动，也不得记录为已生效。Docker 不作为第一阶段桌面部署或沙箱形态。威胁模型据此诚实收窄：未启用加固时，Harness 与同 OS 用户的其他进程处于同一信任域，合同只承诺三条底线在治理面成立。

本条撤销 §16 的“用户在场证明”与“OS 沙箱入场券”两项，并把 CT-AGENT 里按沙箱写的一串负例改成三条底线的正面陈述；对应合同句在 [Agent 模块合同](../spec/agent.md)的写入合同与运行时两节、[系统边界](../spec/system.md)的命令与跨服务正确性、外部权威副作用与安全边界三节，以及交付验证 B2/CT-AGENT。

## 23. 代次不在 Dagu：完成与评审都在 HCTL，Dagu 只当路标（v0.12.4）

§18 换用 Dagu 时留下一个阻断项：要求 `human.task` 完成 API 能隔离迟到的完成请求，否则 B4 重开选型。复审发现这把 Engine 当成了半个权威——Obligation 身份、租约与超时都绑到“adapter 回读证明的 engine attempt/status generation”上，Run 完成还要等 Engine success terminal 回读。正确道路是：完成与评审只在 HCTL 账本发生。Obligation 由 control 按 Run、节点与观察序号铸造，deadline、候选切换、Verdict/Receipt 与 Run 完成谓词全部只依据账本；Engine 只是路标——control 观察它进入等待态就铸 Obligation，账本结果落定后再把路标推过检查点；路标被 Engine 自行推进、重试或读不到时，只把 Engine Execution Binding 标为分歧待对账，既不补足也不阻止任何判决。重复进入同一节点按观察序号产生新 Obligation，不再要求 Engine 提供可隔离的检查点身份。本条撤掉 §18 的 B4 阻断项，P0 的 Dagu 探针相应只验接缝（见 §25）。权威文本在 [Run 模块合同](../spec/run.md)。

## 24. Room 对控制面明文可读：不启用端到端加密（v0.12.4）

§12 把消息 content 交给 chat server 时隐含了一个没写出来的前提：控制面能按事件 ID 读到消息正文——冻结升格来源与 Context 锚点的 digest、本地萃取相关讨论、AppService 写结果卡与 homeserver 侧桥接都建立在这一点上。Matrix 的端到端加密（`m.room.encryption`）会把正文锁在客户端设备里，房间一旦开启，这些能力整体失效而账本并不知情。v0.12.4 把“房间对 control 明文可读”写成 Chat 端口绑定的准入前置：创建或绑定时以 fresh 房间状态回读校验，已加密房间拒绝；事后被开启加密视同 control 读不到正文，走与 chat server 不可用同一条 fail-closed 规则并标为需要关注，恢复只有换绑到未加密房间。这不是新对象或新状态值，加密状态只是绑定的 health 投影。隐私与敏感输入的答案由此收窄：敏感输入走安全通道，仓库级隐私靠 homeserver 访问控制与保留策略，不靠给房间加密。权威文本在 [Project 模块合同](../spec/project.md#room-与消息)。

## 26. 小修订台账

来时路只收转向：承重墙移动、实现选型更换、权威归属变化各自成章。词汇、词形与概念清扫类修订各记一行，细节在合同层清扫表，不再单独成章：

| 版本 | 日期 | 内容 | 详情 |
| --- | --- | --- | --- |
| v0.10.3 | 2026-08-19 | 词汇清扫：RuntimeBackend、TaskSource、WorkflowEngine、HarnessAdapter 四个未入册高频名降级为描述语或端口种类；交付文档定为与合同层同侧 | [清扫表](../spec/README.md#v0103-清扫) |
| v0.11.1 | 2026-08-21 | 词形收敛：25 个驼峰名改带空格专名、16 个 `*Intent` 命令名改动宾语义名、状态值改中文语义名；“不含任何代码标识符”的绝对化后被 v0.12.0 复审收窄（协议/schema 字段与外部原名保留原形） | [词形表](../spec/README.md#v0111-词形收敛) |
| v0.12.2 | 2026-08-24 | 概念清扫：Room Event 除名、Task Operational State 降级为操作投影、收口状态统一为“丢失” | [清扫表](../spec/README.md#v0122-清扫) |

## 27. 当前落点

这条来时路最终收敛为四个权威模块、四个对仗 Scene、共享但受控的连接与执行机制。阅读当前设计时，应从[愿景](../vision.md)开始，再读 [Project](../project.md)、[Task](../task.md)、[Run](../run.md)、[Agent](../agent.md)，再查看[连接合同](../spec/connections.md)、[系统边界](../spec/system.md)和[第一阶段交付](../delivery.md)。本文用于解释这些边界为什么存在；发生冲突时，它不覆盖任何当前规范。

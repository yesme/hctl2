# Participant 模块约束

> 状态：规范性约束 · 草案 v0.18.9<br>
> 本文是 Participant 模块对象、状态机与写入约束的唯一权威。设计正文见 [Participant 与 Terminal](../participant.md)；模块交接见[连接约束](./connections.md)，共享机制见[系统边界](./system.md)，族语义与词汇分类见[约束层总则](./README.md)。

## 对象

Participant 模块拥有数字参与者的身份与配置，并负责把获准的 Execution Spec 变成一次经 Agency 的派工，持有观察、隔离和恢复的控制面半边。Project 拥有 Room Invocation 与 Room 名册，Run 拥有 Attempt 与 Seat；两者各自拥有对应的 Execution Spec。Participant 模块不决定 Project、Task 或 Run 的领域结果；Repo、ChangeSet 与 Write Lease 归 [Repo 模块](./repo.md)所有，参与者只在有效租约下物理写入。

人不是 Participant：人不干活，人拍板。人以 human actor 出现在命令和裁决里，不进规划者名册，也不进入本模块的对象表。

控制面与参与者之间的全部交互都经 Agency：要人、派工、输入、停止、观测、终端连接、工具报告与结果回传。控制面知道的只有三样：派工与语义归属者代次；自己签发的授权对象（Execution Spec、租约、票据）；归到派工的结果、证据与观测。Agency 门后的进程、会话、主机、接入协议、待命与换人是它的内部，不进本模块的对象表。

| 对象 | 含义 |
| --- | --- |
| Profession | 工种：Agency 名册项的冻结引用——Harness、模型、Skill 配置、人设默认、默认职责倾向（规划 / 施工）、条款，以及控制面接受该项时记下的接受条件（名册项版本、条款、Skill 申报与承诺能力及其可核验性）；带版本与摘要 |
| Participant | 被选进某个 Room（规划者）或某个 Run 席位（施工者）的一位工种实例，以一份选入记录存在（字段只在[连接约束](./connections.md#project--run--participant从授权到派工)定义一次）；只存在于被选进的地方，不跨阶段共享身份；不含密钥或运行时身份 |
| Skill | 带 revision 与 digest 的共享方法定义；三态与申报见[Skill 与申报](#skill-与申报) |
| Worker Profile | Harness、模型、模式、权限、环境与可选的隔离效果要求的可复用配置 |
| Harness 目录 | 两类申报事实：定义（Harness 是什么）、实测能力（实际支持什么），由 Agency 申报并随工种引用冻结；安装位置与逐主机清单归 Agency，控制面不持有；不设类名 |
| 派工（Dispatch） | 向 Agency 提交一次 Execution Spec 并被接受后得到的引用（归属者 = Attempt \| Room Invocation），Agency 对它负责；控制面记它被接受、履约中、已交回结果、报无法履约、被取消或替代这些可观察结果，不记主机、隔离域、进程或物理代次；终端通道是其字段组 |
| Attach Descriptor / Terminal Input Lease | 对一次派工的短期连接票据和单输入者租约，由控制面签发、Agency 校验 |
| Result Proposal / Evidence | 参与者经 Agency 提交给上层校验的结果和观测，不是 Verdict/Receipt；每条 Evidence 标注证据通道等级（见[证据通道](#证据通道)） |

显示名、外部账号、人设、Skill、Worker Profile、Harness session 或模型名都不能替代选入记录，也不能自行授予职责或权限。Room 名册或 Run 席位换人不改写活动 Invocation、Seat 或 Run。

## Skill 与申报

Skill 是带稳定 ID、revision 和 digest 的共享方法定义，至少固定 manifest/instructions/assets/scripts、来源/license、兼容能力与依赖；更新创建新 revision，current pointer 只用于选择。Skill 提供方法并请求能力；权限、票权、委派与 Task 完成权仍由对应领域约束授予。Skill 的内容不归 HCTL 存放：由 Agency 安装并申报，由参与者装载；控制面存储只保存引用与 digest。

Skill 分三态：**declared**（参与者档案或 Agency 名册声称会）、**available**（Agency 申报精确 revision 已安装、可回读、依赖满足）、**activated**（本次 Execution Spec 已冻结并装载）。Execution Spec 与 Run Manifest 必须冻结精确 ref+digest，并为每个 Skill 记录可核验性：经 Agency 取得的直报核验报告回读到同一 digest 的记 known，只有申报、没有可核验报告的记 unknown；远程参与者不天然 unknown，本机参与者不天然 known；不得把 unknown 记为 known。required Skill 缺失，或申报的 digest 与核验回读不一致时，解析失败、不激活；optional Skill 缺失显示降级。含脚本的 Skill 是代码供应链输入。

## 写入约束

| 聚合 | version / lifecycle | 合法命令与唯一写入者 | 终态或不可变结果 |
| --- | --- | --- | --- |
| Participant（Room 名册记录 / Run 席位记录） | 随所在 Room 名册版本或 Run Manifest 冻结 | control 处理「选入 Room」「移出 Room」「席位选人」命令；Agency 只报名册与申报能力 | 活动 Invocation/Attempt 永久引用选入时的记录 |
| Worker Profile / Profession 引用 | immutable revision + current pointer | control 处理「创建/更新 Worker Profile」与「收进/更新工种引用」命令，收进或更新时记接受条件；Agency 报告名册与申报能力，名册变化只产生新版本供后续选择，新版本被接受须经显式命令 | 活动 Invocation/Attempt 始终引用原 revision；申报与可核验性分开记，冻结引用不把 unknown 变成 known |
| 派工 | 随归属者的 `invocation_version` \| `attempt_generation`；可观察结果：已接受 / 履约中 / 已交回 / 报无法履约 / 已取消 / 被替代（联系不上是叠加的观测，不是状态） | control 处理「派工」「输入」「停止」「取消」命令并经 Agency 端口向自己的租户提交；Agency 持有门后资源并报告；control 记观测记录 | 已取消/被替代不复活；同一派工内 Agency 的内部重启、改派、搬机不改变派工引用；新的尝试是新的派工 |
| Terminal Input Lease | 租约代次；活跃 / 已撤销 / 已过期 | control 授予/撤销，Agency 只把当前租约的输入送到该派工；原生写入是否受租约约束按 Agency 声明能力与 Execution Spec 输入策略冻结 | 一个受 HCTL 管理的目标最多一个活跃输入者；允许原生交互时不得宣称物理单写者 |
| Result Proposal / Evidence | immutable submission + producer sequence | 参与者经 Agency 提交；control inbox 持久化；Project/Run 独占 admission | Proposal 不可改成 Verdict/Receipt；修正提交新 Proposal |

Worker Profile、Harness 名称或“支持某协议”都不隐含能力。Agency 端口的 Port–Provider Binding 只冻结 Agency 对控制面的公开承诺：实测能力、信任级别、权限作用域、降级策略，以及是否具备「代为执行工具并直报」（见[证据通道](#证据通道)）；接入协议与门后怎么做不进绑定。

每次派工都只授予窄执行主体。授权、能力与证明是三件事：授权决定可做哪些动作，由上游逐级缩小（见[连接约束](./connections.md#版本权限与替代)）；凭据的作用范围、受控接口与已落实的隔离限制实际能做什么；证据核验决定结果能证明什么。凭据的作用域按[安全策略面](./system.md#安全策略面)「凭据」行。以下三条底线不可关闭；控制面存储的单写者另有自己的[三条底线](./system.md#单写者)，两组互不替代。

### 不可关闭的三条底线

1. **工具不是人。** Harness、运行时钩子和模型只能提交 Result Proposal，不能提交治理命令。
2. **合入钥匙不进工具。** HCTL 不向模型交付通用 control 客户端凭据、human principal credential 或集成凭据与权限。获准源分支的 Git 交付由持相应凭据的单元执行，不包含合入目标权限；目标集成、任务后端与 chat 写入凭据仍由获准的工具或适配器网关代用，评审请求仍按原授权与适配器路径执行，见 [Repo 发布评审](./repo.md#发布评审)；底线汇总见[安全策略面](./system.md#安全策略面)。
3. **隔离工作树。** Harness 只能在有效 Write Lease 下写当前 ChangeSet 的独立 Git 工作树和分支。它可以在获准范围读取 Git 对象与引用，并在当前 ChangeSet 分支提交；工作副本由参与者管理，独立引用在写入前核验不相交。发布源版本与更新目标是不同权限，后者按 [Repo 模块](./repo.md#集成目标两个头与两种授权形态)的持久意图与回读处理；目标变化不自动取得本控制面的集成凭证。

### 可选执行加固

Worker Profile 可以声明要求的隔离效果：凭据代用范围、可访问的网络目的地、允许调用的工具能力。Execution Spec 必须冻结已声明项；Agency 在接受派工时逐项承诺能否施加，缺任一项时 control 拒绝激活并列出缺项。未声明时，control 不要求这些效果，也不得记录为已生效。用什么 OS 机制、钩子或代理达成这些效果是 Agency 的内部，不进约束。

### 有条件的安全输入

只有 Agency 声明并承诺敏感输入不进入[安全策略面](./system.md#安全策略面)「敏感数据」行所列去处时，Execution Spec 才能启用安全输入。

### 派工前校验

control 必须在提交派工前核对 Context Bundle 的实际交付摘要、Execution Spec 摘要，以及 Agency 承诺的能力与隔离效果。

## ChangeSet 与 Git 事实

ChangeSet、ChangeSet Revision、Write Lease、封存、保全与集成的对象与写入约束由 [Repo 模块约束](./repo.md#changeset-与-git-事实)拥有；本节只保留执行侧的半边。

旧写入者失权时，两个模块各做自己的动作：Repo 模块撤销租约并拒绝重授；本模块经 Agency 停止或隔离旧执行，并从 Agency 取得对旧写者的三态报告：已停止；仍存活但已被限制在旧 Git 工作树与旧 ChangeSet 的边界内；不能证明。前两态是隔离成立的两种证明——第二态按 [Run 约束](./run.md#写入约束)允许后续执行改用新工作树与新 ChangeSet，原 ChangeSet 不重授。控制面核的是哪个 ChangeSet、哪份授权受保护，不查进程、PTY 或目录。第三态时本模块不得声称已隔离，Repo 模块因此不授予新租约，原 Git 工作树与 ChangeSet 按其约束保全并隔离。

参与者在有效租约下写自己的 Git 工作树与分支，封存由 `hctl2-tool` 执行。经 Agency 提交的 Result Proposal 中，ChangeSet 输出至少固定 ChangeSet 的稳定 ID、所持 Write Lease 引用、声明的基线提交，以及可解析的精确结果引用——参与者分支上的提交，或交给现场工具的工作树；跨单元交付不要求公开生产目录，封存输入与回读字段见 [Repo 模块约束](./repo.md#changeset-与-git-事实)。版本准入在归属者准入提案的同一事务里由 Repo 模块完成。源版本按获准范围由持 Git 凭据单元交付，评审请求与目标合入仍按 [Repo 约束](./repo.md#发布评审)处理；交代码不等于取得集成权。

<a id="运行时与观测"></a>
## 派工与观测

Run 经其 Attempt 可以有多次派工；Room Invocation 至多一次。Attempt 与 Room Invocation 各至多一组终端通道。派工不以 TTY 存在为前提。

Room Invocation 的派工继承其 Execution Spec 中的所属 Project 与本次读写范围；Attempt 的 Project 范围来自 Run Manifest。两条路径都必须保留精确归属者、Project、派工引用、绑定、语义代次和权限；已知派工不能被降级成无主活动或模糊仓库活动。

代次分层记录，不能共用一个模糊的 `generation`：语义归属者代次标识这一次逻辑执行归谁；`control_writer_generation` 标识控制面存储此刻的写入者；成员与推导规则见[代次家族总表](./system.md#代次家族)。控制面不记录物理代次：Agency 门后换了几次进程、会话或主机不是控制面可见的代次。替代语义归属者只使引用旧值的 HCTL 动作失效，不得顺带改写其他层的身份。

**Agency**（派出方）是控制面与参与者之间唯一的通路。它维护可派出的名册与条款，回应 control 的「要人」请求，为每个配对的控制面提供一个隔离租户（定义见[安全策略面](./system.md#安全策略面)「租户隔离」行），并对租户内的每次派工负责：接受冻结的 Execution Spec 后返回派工引用与实际能力；此后 control 的输入、停止、票据签发与观测订阅都以这次派工为对象、经 Agency 转到门后；结果、证据与工具报告经 Agency 转交回来；参与者的机器不持有控制面的地址与凭据。名册项就是工种；换派出方就是选另一个工种的实例，不存在换绑；Agency 一侧的绑定归受控端口（Port–Provider Binding），与工种实例无关。Agency 的接口约定**永不包含治理权威**：转接不是准入；租约、代次、冻结规格、审计与结果准入的权威只归控制面，恢复等级由 Agency 声明并负责、控制面只记录；Agency 自带的接管、单写者或“会话有效”记录只作执行协助与观测证据，不得写入或替代治理记录。

派工是控制面对 Agency 的委托，不是对某个进程的引用。同一派工内的 failover——Agency 在冻结的 Execution Spec 内重启会话、改派备份、换主机——是 Agency 的内部：不重新派工、不找人、不改变派工引用，control 不记录也不要求报告门后换了几次。回来的旧执行由 Agency 在门后按当前派工核对（图还是那张图、席位还归自己）；失效的旧执行不能以当前派工的名义发起新的写入、封存或结果准入，它在原授权内已保管的结果仍可按原提案交回，control 再按原授权判能否准入。Agency 在规格内自愈不了——含旧写者停不下来、隔离不了——时报「无法履约」及原因；control 按所报故障与冻结规则处理：候选切换、Request、失败或取消。换 Agency 是有权的人显式提交的替代，选另一个工种实例。

Agency 对每次派工的报告责任：终局结果按它为接入端口声明的终局结果清单逐项核对——参与者正常退出但缺少清单要求的终局结果事件时，Agency 必须报类型化协议错误，不得默认成功；由 control 主动取消导致的退出必须归因为取消，不得上报为执行失败；观测上报通道失败时，只能显式标记该派工的观测截断并终结事件流，不得交付有缺口的事件流冒充完整历史；派工的停止结果与退出码随停止报告返回。参与者内部再派出的队友、子进程与团队模式是它的内部；Agency 只报派工的用量，报不全的标未知，control 不要求重建内部谱系。

观测由 Agency 提供：结构化事件统一归一为生命周期提示、工具调用、权限请求、文件变化、测试、用量和原始输出，未知事件保留原文并安全降级，不得凭渲染器猜测完成。每条观测记录来源、置信度、证据和观测时间；判断语义状态时，结构化协议或原生钩子优先于转录推断，最后才参考标题和屏幕内容，低优先级信号不能覆盖仍有效的高优先级证据；无论置信度多高，观测都不能自行推进领域结果。Agency 声明事件游标时，必须报告序号和缺口；未声明时，事件流只能作为有界观测。

单纯不可达只记**联系不上**：它是叠在派工上的观测，记断开的事实与最后观测时间，不撤权、不延长授权。派工能否继续按原授权、冻结的截止与取消条件判；Agency 的回应是核对的输入，不是必要前置。联系不上不因时间长变成丢失，变成丢失的只有语义归属者或租约核不上（唯一定义见[连接约束](./connections.md#失败与恢复)）。截止已过按截止规则处理，原因记超时；控制面睡眠不改变已冻结的截止，「只算活跃时间」须事先在 Execution Spec 约定。Agency 明确报无法履约时按所报故障处理，不记成联系不上。三只钟各算各的：Agency 的闲置条件、控制面的执行截止、平台的有效期；控制面的判定只对本控制面的授权、准入与领域结果是权威。

Execution Spec 必须冻结终端输入策略。受管单写者（`managed_single_writer`）要求所有受管输入经当前连接票据与 Terminal Input Lease 校验，由 Agency 对本租户执行；Agency 不能统一拦截全部写入时，必须关闭原生控制器。允许原生交互（`native_interactive_allowed`）允许经 Agency 的原生客户端向该派工输入，并明确接受 Agency 无法逐次证明 actor 与租约。后一模式中的输入是有效运行时输入，不是分歧，也不自动污染独立的 Git、SCM 或测试证据；执行记录必须标明输入来源不完整，不得声称物理单写者、完整回放，或由该输入产生 HCTL 命令或结果。切换策略必须创建新 Execution Spec 或替代执行，不能在活动执行背后静默放宽。Agency 声明逐次输入记录时，每次输入必须关联票据与 actor；未声明时不得声称来源完整。

这是[多写通则](./system.md#命令与跨服务正确性)在 Agency 上的实例：一次派工只属于一个租户、只听它的控制面；受管输入同一目标一次只有一个 Terminal Input Lease 持有者，接管原子撤销旧租约；原生输入允许但不能逐次证明来源，其中的文字与“完成”不准入；跨租户在结构上不可达（[安全策略面](./system.md#安全策略面)），不是“不算命令”而放行。变更集单写者与引用不相交见 [Repo 模块约束](./repo.md#changeset-与-git-事实)。

Proposal 头必须固定 proposal ID、归属者与语义代次、派工引用、Execution Spec 与 Context Bundle 摘要、Agency 绑定、生产者序号和幂等键。控制面内部的纯计算与引擎 noop 可以进程内执行，但不是工种派工，不进 Proposal 通道。

每个输出项必须另带 schema key、content digest、候选产物引用和自己的归属者、派工与授权引用。只有 output schema 明确允许逐项准入时，归属者才能单独接受合格项；否则任一必需项不匹配都拒绝整组。任一代次、绑定、Bundle、租约或输出范围不匹配的项只能留作审计，不能让其他合格项替它背书。修正必须创建新 Proposal 和新的生产者序号，不得改写原项。

Harness、运行时钩子与模型只获得当前 Invocation/Attempt 所需的窄 execution principal，不能持有通用 command Submit credential、human principal credential、Task lifecycle 或 Room dispatch 权限。它们可以建议完成或建议下一位 Participant；建议经 Result Proposal 通道由归属模块准入，不是命令。

<a id="证据通道"></a>
### 证据通道

Evidence 是被判定的事实记录，本身不下结论。每条 Evidence 必须标注证据通道，按“经没经过模型的嘴”分三档：

| 等级 | 通道 | 例子 |
| --- | --- | --- |
| `unmediated` | 直报：获准的采集方经独立于模型输出的通道提交、来源及所证的目标与版本可核验的原始报告 | 控制面自己的适配器读到的 PR 与检查状态、CI 结果；控制面本机运行的 `hctl2-tool` 回读到的 Git 基线与 HEAD、路径与摘要；参与者内部的 `hctl2-tool` 经具备「代为执行工具并直报」能力的 Agency 转交的测试退出码与输出 |
| `adapter_event` | 旁路：适配器从参与者旁路观察到的结构化事件 | Agency 归一的工具调用、测试、文件变化事件 |
| `narrated` | 转述：模型输出里的声称 | 参与者或模型在输出里说「我跑了测试，过了」 |

「高证据类」指前两级。未标注通道的 Evidence 按 `narrated` 处理；转述不能通过重新标注升级。Task 验收策略可要求某验收项的证据不低于某一级，Gate 策略可要求 Verdict 引用的证据不低于某一级。Run 节点的外部机械事实前置与 Integration Receipt 的回读只认直报（见 [Run 约束](./run.md#从节点到结果)与 [Repo 约束](./repo.md#集成目标两个头与两种授权形态)）；Task 的 `mechanical` 项接受直报或旁路，并服从该项冻结的证据等级要求。证据等级与物理代次无关。

直报有三路来源：控制面自己的适配器读平台；控制面本机运行的工具；参与者内部的工具经 Agency 转交。前两路不经参与者，不受“参与者交互必须经 Agency”约束，也不因参与者一侧的加固与否而降级。第三路要成为直报，Agency 须具备并在其 Port–Provider Binding 里声明一项公开能力：**代为执行工具并直报**——工具由 Agency 代为执行、结果经 Agency 的通道上报并归到本次派工与所证的目标/版本，模型写不进这条通道、不能替换这份报告；怎么做到留实现。这项能力配对时记录、派工时按 Execution Spec 核，不是每次运行临场声明，也不看沙箱开没开。没有这项能力的 Agency，它转来的工具输出按旁路或转述记。Agency 在信封上声明转交者身份与改写原始报告是两回事，要核的是谁采集、谁转交；来源不符或已识别为伪造的报告不作为直报准入。派工前查 CI、无 Run 的人工封存按对应获准动作归因，不伪造一个派工。

## 终端通道、连接与租约

恢复等级包括 exact attach、native handoff、structured inspect、semantic resume 和 replay，定义见[设计正文](../participant.md#terminal-场景)。它们是 Agency 声明的能力词汇：每项按自己的证据要求由 Agency 分别声明与降级，Agency 对其声明的等级负责，不能用一项的证据顶替另一项；控制面不核进程同一性，只记 Agency 报告的等级。

派工被接受后，control 为它建立终端通道记录；物理通道、观察流与终端状态由 Agency 提供，HCTL 不转发或重放另一份 PTY 流。直接客户端按当前归属者与派工请求连接时，control 可以签发短期 Attach Descriptor，并为受管理写输入另行以比较并交换授予 Terminal Input Lease；票据经 Agency 校验后生效，前端连接的对手方是 Agency。

Attach Descriptor 固定派工引用、权限和过期时间；归属者与能力经派工引用解析后仍核。观察、终端输入或接管、Attempt 控制和安全输入分别授权，任一权限都不蕴含其他权限。一个目标可以有多个观察者；HCTL 管理的输入默认最多一个 Terminal Input Lease 持有者，接管必须原子撤销旧租约。

绑定声明允许原生交互（`native_interactive_allowed`）时，经 Agency 的原生客户端可以不经该租约输入。control 把它记录为允许但无法逐次证明来源的运行时交互；该通道中的文字或所谓“完成”不能直接准入 HCTL 结果。

Execution Chat 投影是 Terminal 中绑定且只绑定一个精确 Room Invocation/invocation_version 或 Attempt/attempt_generation 及其派工的结构化观察与控制视图。它不是 Room，也没有独立会话身份。Agency 支持时，输入作为携带这些精确引用的获准 control 动作经 Agency 写回同一派工；能力不足时准确降级为 structured inspect 或 terminal，不得改投另一个会话。

Execution Chat 中的输入和事件不会自动成为 Room 内容。只有显式 Share to Room 动作经 Project 命令准入后才能发布，并携带来源事件、执行归属者版本或代次、派工引用，以及转录与证据来源。该投影消失或派工被替代都不改变 Room 身份。

Workbench 或终端客户端退出不停止执行。断流按派工引用、来源流 sequence 和快照恢复；Agency 不能按其声明的等级证明连接对象未变时只能 semantic resume、replay 或新建派工，不能声称 exact attach。semantic resume 可以用自有观测留痕重建续跑输入；重建物按投影处理，不进入权威记录。

## 外部概念对齐

对齐用于翻译与接入，不转移权威。

| HCTL | 外部体系 | 差异一句话 |
| --- | --- | --- |
| 派工 | 云平台上按租户提交的执行实例引用 | 宿主内部迁移不改变租户看到的引用；HCTL 的派工没有物理字段，授权与判定的权威在控制面 |
| exact attach / detach | 终端多路复用器的 attach / detach | 连接或断开仍存活的会话；断开不停止执行；能否声称 exact attach 由 Agency 按它声明的等级负责 |
| PTY | PTY（伪终端） | 同名同义的基础设施概念，在 Agency 门后 |
| 结构化接入 | ACP（Agent Client Protocol）等代理协议 | 是 Agency 门后的接入方式之一；协议会话不是 HCTL 身份 |
| semantic resume | 各 Harness 原生的会话恢复（如 codex resume） | 恢复的是上下文，可能创建新进程；不等于 exact attach |
| replay | 终端录像回放 | 只读历史，不冒充存活会话 |
| Terminal Input Lease | 无对应 | 差异化语义：单输入者，接管原子换人；写入侧的 Write Lease 见 [Repo 模块约束](./repo.md#外部概念对齐) |

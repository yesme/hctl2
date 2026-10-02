# 演示线开工书：九个包，从聊天到合入

> 状态：已拍板 · 所有者 2026-10-03「落」；§二 各项为所有者同日裁定；第 1 包 #307 已合，演示 1 已验收（§三，所有者同日裁「按建议来」）<br>
> 基线：main @ `cdb6c66`（草案 v0.19.0；本文所在的 PR 把基线升到 v0.19.1）；演示 1 验收后 main @ `a075a66`，验收 PR 把基线升到 v0.19.2<br>
> 去向：`src/apps/*`、`src/crates/*`、`src/agency`；`docs/design/delivery.md` §本地 Agency 参考实现（形态已定）；不改约束层<br>
> 读法：先读 [`README.md`](./README.md) 状态板，再读本文 §三、§四。旧任务书在 [`01-plan.md`](./01-plan.md) 与 [`05-p22-kickoff.md`](./05-p22-kickoff.md)，与本文不一致处以本文和现行约束为准

## 一、定位与重述

所有者 2026-10-03 的要求：尽快把代码写到能看 demo。现役 harness 增到十家；所有者仍是各家之间唯一的转发通道，所以工作要切成大块、不并行。

现状：P2.1 已收口。P2.2 的 Repo 注册（#287）、任务源与 Task（#288）、聊天端口与 Room 树（#292）已合入。还不能用命令创建 Project，测试里用的是预置数据。

原计划（`01-plan.md` §四、§十二）的大顺序不变。本文做四件事：

1. 定义三次演示（§三）。
2. 把到第三次演示为止的工作排成九个包，一包一个 PR，单线串行（§四）。
3. 记录所有者对 Agency 形态的裁定（§五）。
4. 按新的席位规则分工（§六）。

范围之外：Run 与施工图（P2.5）、Workbench（P3），以及 §七 列的先不做项。

## 二、所有者 2026-10-03 的裁定

1. **第一个 demo 不带 Workbench。** 用公共 CLI 加各内容系统的原生界面看：Cinny 看聊天，Gitea 网页看任务和评审，Herdr 终端看执行。
2. **Agency 的形态。** 原话：「agency未来一定是个独立运行的东西，可以类比于各个模块的独立后端：完全独立于hctl、有自己的(web)client、可以自己独立启动。注意：agency是个『本机所有agents的集合化proxy』，其实和herdr蛮像的哇！所有与agent的互动都要经由它来完成。它还可以主动启动agent、checkout worktree。」展开见 §五，所有者已确认那份展开。
3. **席位。** 原话：「design doc的部分，主要还是claude。核心由我定、由kimi审 (其他模型也可以审)」「coding的部分，可以codex定框架分任务，glm / qwen / deepseek写，codex / grok审。难的复杂的各种勾稽关系的，也可以codex / grok写，glm / qwen / deepseek审」「agy 和 minimax，可以给他们一些小活儿，但还需要关注他们的performance」「工作尽量分割得大块一些，不要并行」。落法见 §六。
4. **流程三条。**
   - Fable 不再逐个审代码 PR，只做三次演示的验收；所有者点名时再审。
   - 两席都写「可合」且 CI 绿，作者直接合，合后报所有者。
   - 新写手的包两轮修不完，交回 Codex 接手。
5. **这一段先不做的**见 §七。
6. **编号。** 包与批次用数字、罗马数字或英文字母，不用天干地支。旧任务书里的天干标题不回改，本文只在指向它们时照抄。
7. **看板的理解。** 原话：「要区分source和UI。如果source是SCM的issues，或者专门的系统如Linear/JIRA，那么在hctl上就是yet another UI/client - 因为source的源体也有自己的UI。如果开发者想在看板source之外再用自己『本地』的source，那么就应该用Vikunja作为后台。」这与 `spec/task.md` §契约与来源 一致：卡片内容以任务源为准，HCTL 只多存承诺身份、验收契约与完成凭证；不改约束。
8. **各家自己盯 PR。** 所有者同日补充，原话：「大家都会监控PR，有comments了可以自己看，不用再等我贴过来」。起因是 #302 的评审回来之后，主笔没有自己去看，等所有者来转告。落法见 §六。

## 三、三次演示

每次演示由 Fable 验收：照约束与验收用例把演示路径实际走一遍，如实报告哪些通过、哪些没有。Kimi 对照 `contract-tests.md` 走查覆盖。

| 演示 | 做完哪个包 | 能看到什么 | 验收依据 |
| --- | --- | --- | --- |
| 1 · 协作现场 | 第 1 包 | 用命令建 Project；在 Cinny 里看到主 Room，开 Topic、带前情提要；平台上的 issue 认领成 Task；重启后都还在 | `delivery.md` §自举阶段 B1 行 |
| 2 · 只读派工 | 第 5 包 | 从 Room 发起一次只读的调用（调研、比较一类）：命令行给出派工预览，确认后 harness 在隔离的工作副本里运行，Herdr 终端里实时看，结果回到 Room | `spec/project.md` §Room Invocation；CT-PROJECT、CT-PARTICIPANT 的相关行 |
| 3 · 从聊天到合入 | 第 9 包 | 写入型调用改代码，封存成版本，平台上开出评审，人预览合入，再预览完成 Task，两张凭证落库；本地 Gitea 与受保护的 GitHub `main` 各走一遍，Codex 与 Claude Code 各跑一遍 | `delivery.md` §纵向切片 A：无 Run 自举、§自举阶段 B2 行；CT-PRODUCT |

写入型调用（改代码并封存）要到第 6 包才接上，所以演示 2 是只读的。

### 演示 1 验收 · 2026-10-03

Fable 用 main `a075a66`（含第 1 包 #307）构建的安装包从零装了一套，照上表四步手工走完；命令与看到的东西逐条记在 [`2026-10-03-演示1验收.md`](../../log/2026-10-03-演示1验收.md)。

| 步骤 | 结果 |
| --- | --- |
| 用命令建 Project | 通过。注册仓库、两步确认、`project create`，Project 与主 Room 一次事务建出，重投同一 key 返回同一个 |
| 在 Cinny 里看到主 Room | 通过。`project members` 邀请后，人类账号看到「Apollo」与自己发的消息（核的是 Cinny 用的同一套 Matrix 接口，没用眼睛看屏幕） |
| 开 Topic、带前情提要 | 通过，带缺口。草稿是逐字片段、报「未配置自动归纳」；Topic 建出并挂在主 Room 的承载 Space 下；提要作为材料可反复读 |
| 平台 issue 认领成 Task | 通过，带缺口。`connect / attach / refresh / claim` 后 Task 开出，和 issue 对得上 |
| 重启后都还在 | 通过。`kill -KILL` control、停全部服务、`hctl2 start`，约 1 秒恢复，10 个查询重启前后逐字节一致，未发送草稿也在 |

缺口（都不违反当时的约束，是「人用 Cinny 看」时的问题）：

1. 开 Topic 的人自己不在 Topic 房间里，要再跑一次 `project members` 才看得到。
2. 人不在承载 Space 里，Cinny 把主 Room 和 Topic 显示成两间平铺的房间，看不到树。
3. Topic 房间里是空的：提要只存在控制面，没有发进房间。
4. 人没有进本地 Gitea 的路：control 建的仓库是私有的，人没有账号也没有协作权，hctl2 没有命令给人开。
5. 钥匙串：新构建的 control 二进制每次读令牌都弹授权框；没屏幕的会话直接失败（Codex 本机 B1 红就是这个）。代码里有不碰钥匙串的私有文件后端，没有开关。
6. 小的：`hctl2 start` 不拉 Cinny，停过要手动起；第一个注册的人成为 Tuwunel 管理员。

所有者裁定（2026-10-03「就按你的建议来吧」）：1、2、3 合成一题「人在聊天客户端里看到什么」，三条缺省写进约束（v0.19.2，`spec/project.md` §Repo 注册与 Project 归档「创建 Topic Room」一段、§Room 与消息 承载 Space 一句；CT-PROJECT 两行）：Topic 建成后把确认的提要作为开场消息发进房间；按预览确认的名单邀请聊天成员，缺省是来源 Room 的人类成员；承载 Space 的成员跟主 Room 走。代码改动是第 1 包的补丁 1a（§四）。4、5 各是一件小活（§六 的 D、E）。6 不做。另按 §六 原定，演示 1 的手册由 Antigravity 写、Grok 重跑（小活 C）。

## 四、九个包

| # | 内容 | 写 | 审 | 依赖 | 旧任务书里的名字 |
| --- | --- | --- | --- | --- | --- |
| 1 | Project、名册、Request | Codex | Grok、GLM | 已满足 | 辛 |
| 2 | Agency 服务骨架与端口 | Codex | Grok、DeepSeek | 1 | 壬的前半 |
| 3 | Agency 运行时，接 Herdr、Codex、Claude Code | Grok | Codex、Qwen | 2 | 壬的后半 |
| 4 | Context 组装 | GLM | Codex、Grok | 2 | 子 |
| 5 | 派工 | Codex | Grok、Qwen | 1、3、4 | 癸 |
| 6 | 变更与合入 | Codex | Grok、GLM | 5 | 丑 |
| 7 | 完成 Task 与发布 Memo | DeepSeek | Codex、Grok | 6 | 寅，加发布 Memo |
| 8 | 命令行的人读输出 | Qwen | Codex、GLM | 7 | 新增 |
| 9 | 端到端收口 | Grok | Codex、DeepSeek | 8 | 卯 |

和原计划比，调了四处：

- Context 组装提到派工前面。派工预览要展示并冻结上下文。
- Agency 拆成两包。它是这段路上风险最大的一块：第 2 包先用脚本执行体把端口跑通，第 3 包再接真 harness。
- 新增第 8 包。`hctl2` 现在只打印 JSON，派工预览与合入预览是给人看的。
- 「发布 Memo」原计划没有包认领（`delivery.md` §纵向切片 A 第 8 步），并进第 7 包。

「Codex 定框架」的落法：不单开只有框架的 PR。Codex 在第 2 包里给第 3、4、5 包定好类型、接口和任务说明，在第 6 包里给第 7 包定，写在对应 crate 的 README。下面各包只写边界与依据，代码级的任务说明以这些 README 为准。

各包共同的规矩沿 `04-p21-kickoff.md` §五：Buck2 原生目标、同一 rustc 版本、新依赖先补 `docs/research` 对象文件、禁用词、PR 描述三节、不放会话链接。

### 第 1 包 · Project、名册、Request

- **任务书**：`05-p22-kickoff.md` §四 里标题为「辛 · Project 与 Request」的一节，写于 v0.18.11；与 v0.19.0 约束不一致处以约束为准。
- **v0.19.0 带来的增量**：Project 归档按 Room 归属逐间转只读，不按层级读数；多房间动作逐间投递并回读，部分失败不报全体成功（`spec/project.md` §Room 与消息）。
- **做完**：P2.2 收口，演示 1。已合 #307；演示 1 验收见 §三。

### 第 1 包的补丁 · 1a · 人在聊天客户端里看到什么（GLM 写，Grok、Qwen 审）

- **为什么做**：§三 演示 1 验收的缺口 1、2、3。人在 Cinny 里看不到自己开的 Topic、看不到树、Topic 房间里没有提要。
- **依据**：`spec/project.md` v0.19.2 §Repo 注册与 Project 归档 里「创建 Topic Room」一段新加的两句（开场消息、邀请名单）与 §Room 与消息 里承载 Space 成员那句；`contract-tests.md` CT-PROJECT 对应的两行。现有接线：`src/crates/chat/README.md`、`src/crates/project/README.md`。
- **要做的**，三件：
  1. `room create-topic` 建成房间后，由 control 把经确认的提要正文与来源作为开场消息发进新房间。事务 ID 幂等，重试不发第二条；控制面保存的材料仍是权威，`room show` 的 `brief` 不变。
  2. `room create-topic` 的预览列出要邀请的聊天成员；缺省是来源 Room（按 Request 升级时是主 Room）当前的人类聊天成员——不在 control 应用服务命名空间里、也不是 control 自己账号的成员；输入可删减、补充。逐人投递并回读，部分失败照 `project members` 的规矩报逐房间结果与 `ROOMS_PARTIAL`。
  3. 承载 Space 的成员跟主 Room 走：新建承载 Space 时邀请主 Room 当前的人类聊天成员；`project members` 对主 Room 的邀请与移除，同步到本 Project 全部承载 Space。Space 成员不进名册，不改加入规则（仍是 invite）。
- **失败用例**：CT-PROJECT 那两行逐条落到测试；原生测试（`chat_native_test`）加三条：建 Topic 后房间第一条消息与确认的提要一致、人类成员在邀请列表而 control 与数字参与者账号不在、Space 成员随主 Room 同步；B1（`room-cli-test`）加一段，用人类账号的视角核房间与 Space 成员。
- **不做**：人类身份与聊天账号的绑定（控制面不知道「跑命令的这个人」是哪个聊天账号，缺省名单从来源 Room 的成员来，不从命令的 actor 来）；Workbench；改加入规则。
- **分支**：`glm/topic-opening-and-space-members`，base main，一个 PR。和第 2 包并行：本包改 `crates/chat`、`crates/project`、`apps/control/src/chat`，第 2 包在 `src/agency` 与新端口，不碰同一文件。两轮修不完交回 Codex。

### 第 2 包 · Agency 服务骨架与端口

- **目标**：一个能独立运行的 Agency 服务，加控制面一侧的 Agency 端口。不接真 harness，用脚本执行体把配对、要人、派工、输入与停止、观测、结果交回整条链走通。
- **端口合同**：`.proto`，独立 target。覆盖配对与租户、名册（工种、Harness 目录、Skill 申报）、提交 Execution Spec 并取得派工引用与实际能力、输入与停止、带游标与缺口声明的观测、连接票据校验、结果与证据交回并确认保全、能力声明（含「代为执行工具并直报」）。
- **Agency 服务**：位置 `src/agency`。自己的可执行文件、启动与停止命令、数据目录和持久存储；配对认证，每个配对的控制面一个租户；结果保管到接收方确认保全。不依赖控制面的存储与命令内核，两边只共享端口合同。
- **控制面一侧**：Agency 端口的 Port–Provider Binding；把名册项接受为 Profession 的冻结引用；派工记录及其可观察结果；联系不上与无法履约的记录。发布包带上 Agency，由 control 在首次消费时拉起。
- **定框架**：给第 3 包定运行时与 harness 适配的内部接口；给第 4 包定 Manifest、Bundle 的记录类型和组装器接口；给第 5 包定 Execution Spec 与 Proposal 头的类型。
- **不做**：真 harness 与 Herdr 接入（第 3 包）；Agency 自己的 web 客户端（§七）。
- **依据**：`spec/participant.md` §对象、§Skill 与申报、§派工与观测、§终端通道、连接与租约；`spec/connections.md` §Project / Run → Participant：从授权到派工；`spec/system.md` §安全策略面；`architecture.md` §单元与连接；`delivery.md` §本地 Agency 参考实现；`docs/research/sdk/herdr.md`、`docs/research/libs/protobuf-rpc.md`。
- **失败用例**：CT-PARTICIPANT、CT-CONNECTION 里关于 Agency 的现行行，脚本执行体测得了的都测。
- **PR 描述**：本包新增第一方组件，调研节引用 `delivery.md` §本地 Agency 参考实现 的形态结论与本文 §五，不以「不适用」开头。

### 第 3 包 · Agency 运行时与两家 harness

- **目标**：把脚本执行体换成真的。Agency 经 Herdr 持有终端与会话，在隔离的工作副本里拉起 Codex 与 Claude Code，把它们的事件归一成观测，把结果交回。
- **范围**：
  - Herdr 客户端，类型按 `docs/research/sdk/herdr.md` 生成。
  - 工作副本的创建、清理与隔离，复用 Herdr 的工作树能力与 `hctl2-tool`。
  - 两家 harness 的启动、钩子注入、事件归一、终局与恢复，按 `docs/research/harness-adapters.md` §决定建议。输入策略先做「允许原生交互」；逐项审批的路径没做过真实会话验收，不激活。
  - 在执行体一侧运行 `hctl2-tool`（封存、`wait`）并把输出作为工具报告转回；能力声明如实填。
  - 本机名册：装了哪些 harness、精确版本、Skill 指纹。
  - harness 的环境里拿不到控制面凭据与 human 凭据。
  - 待命、闲置与会话恢复按 `delivery.md` §本地 Agency 参考实现。
- **依据**：`spec/participant.md` §写入约束、§ChangeSet 与 Git 事实、§派工与观测；`docs/research/harness-hooks-20260903.md`、`docs/research/runtime/herdr.md`；`delivery.md` §开工前限时验证 第 2 项列出的 Herdr 已知限制。
- **失败用例**：CT-PARTICIPANT 里执行体与终端的现行行。原生测试用锁定的 Herdr 制品和两家 harness 的真实会话；环境里没有凭据时如实标未验证。

### 第 4 包 · Context 组装

- **目标**：派工前把相关内容挑出来打包，产出根 Manifest 和每个消费者的 Bundle，冻结并可核对。
- **范围**：
  - 三处来源的萃取：Room 聊天、Task 评论线、平台评审评论线。评审评论线在第 6 包接上之前只留接口。
  - 全部在本机做，不用模型。未配置 small-brain 时不生成纪要、不压缩，相关性门只看治理记录。
  - 选材与排序；三种交付方式；Manifest 的必备项；Bundle 的交付计量与摘要；派发前核对实际交付摘要。
  - `hctl2 context show|preview`。
- **依据**：`spec/project.md` §Context、Memo 与 Artifact；`docs/design/context.md`；CT-PROJECT 里 Context 的现行行。
- **框架**：记录类型与组装器接口来自第 2 包。

### 第 5 包 · 派工

- **目标**：从 Room 发起一次调用：选人、派工预览、冻结执行规格、经 Agency 派工、结果准入、投影回 Room。
- **范围**：
  - Profession、Participant（选入记录的候选校验从这里开始生效）、Worker Profile。
  - 派工预览：执行者、Context、权限、预算、评审发布策略。
  - Room Invocation 的合法边与丢失处理；Execution Spec 冻结；派发的四步启动顺序；Result Proposal 准入。
  - 提及解析：没有唯一的授权候选时明确失败。模型的提及与建议不能创建调用。
  - 连接票据的签发。
  - `hctl2 profession …`、`room roster …`、`invocation list|show|preview|start|cancel|retry`、`terminal inspect|attach|replay`。
  - 本包交付只读调用的全链；写入型调用的租约与封存在第 6 包接上。
- **依据**：`spec/project.md` §Room Invocation、§场景约束（派工预览的内容、提及解析、模型的提及只算建议）；`spec/connections.md` §Project / Run → Participant：从授权到派工、§Participant → Project / Run：结果准入、§失败与恢复；`spec/participant.md`；CT-PROJECT、CT-PARTICIPANT、CT-CONNECTION 的相关行。
- **做完**：演示 2。

### 第 6 包 · 变更与合入

- **目标**：写入型调用的产出变成可评审、可合入的版本。
- **范围**：
  - ChangeSet 与 Write Lease；失权证明。
  - 封存与准入：执行结果提案的路径，以及人的显式封存。
  - 发布评审：按冻结的策略，分段确认 Git 交付与评审请求的创建或更新。
  - 变更与平台的映射；评审评论线接进第 4 包留的接口。
  - 集成意图：两种授权形态、目标保护快照；本地路径与平台路径；Integration Receipt。
  - `hctl2 changeset show|diff`、`review publish|show`、`integration preview|submit|show`。
  - 顺序：先走通本地 Gitea，再走受保护的 GitHub `main`。
  - 给第 7 包定框架。
- **依据**：`spec/repo.md` §ChangeSet 与 Git 事实、§集成：目标、两个头与两种授权形态、§发布评审、§变更与平台的映射、§平台动作与命令、§恢复；`spec/system.md` §外部权威副作用；`docs/research/sdk/github.md` 的复核记录、`docs/research/gitea.md`；CT-REPO 的全部现行行。

### 第 7 包 · 完成 Task 与发布 Memo

- **范围**：
  - 「完成 Task」逐项校验；机械项接受 Integration Receipt，或契约事先接受的平台集成证据。
  - Task Completion Receipt 与相关记录在同一事务写入。
  - 平台 issue 的原生 Done 映射为同一条完成命令的请求。
  - `hctl2 task complete|reopen|cancel` 里尚未实现的部分。
  - 「发布 Memo」命令。
- **依据**：`spec/task.md` §写入约束；`spec/connections.md` §Human Kanban / Run reducer → Task → Project：验收与回流；`spec/project.md` §Context、Memo 与 Artifact 的 Memo 一段；CT-TASK、CT-PROJECT 的相关行。
- **框架**：来自第 6 包。

### 第 8 包 · 命令行的人读输出

- **范围**：
  - 缺省输出改成给人读的版式：列表成表；派工、发布评审、合入、完成四种预览写清对象、原因、动作的后果。
  - `--json` 保持原样，它是机器接口。
  - 错误的 `code`、`message`、`recovery_action` 原样呈现。
  - 只改 `src/apps/cli`，不动命令语义与 RPC。
- **依据**：`delivery.md` §公共 CLI；`spec/system.md` §场景端口；`scenarios/S3-user-journey.md` 各行对预览内容的要求。

### 第 9 包 · 端到端收口

- **范围**：
  - 在试验仓库上按纵向切片 A 全程走通：两条平台路径各一次，两家 harness 各一次，改的是真实的非文档代码。
  - 缺省路径人只预览两次：合入与完成。打开了「发布评审须人显式确认」开关的 Project 多一次发布预览，单独验收（CT-PRODUCT）。
  - 重启 control、Agency 与用到的内容系统后，状态一致，不重复外部副作用。
  - harness 的环境取不到控制面凭据；声明了执行加固的配置按声明生效，施加不了就不启动。
  - 接进完整包测试。
- **依据**：`delivery.md` §纵向切片 A：无 Run 自举、§自举阶段 B2 行；CT-PRODUCT；`01-plan.md` §六 里「卯」一条。
- **做完**：演示 3。

## 五、Agency 的定位

架构层本来就写着 Agency「生命周期与任何控制面无关，一台只跑 Agency 的机器是正常形态」（`architecture.md` §单元与连接）。交付文档此前把本地参考实现的形态留作未选。所有者的裁定把它定下来：

- **独立的服务**：有自己的可执行文件、启动命令、数据目录和公开端口。不装 HCTL 控制面也能单独运行。
- **控制面只是它的一个客户**：控制面和它配对之后才能派工；同一个 Agency 可以同时服务多个控制面。
- **本机所有 agent 的统一入口**：本机装了哪些 harness 和技能、启动 agent、建工作副本、输入与停止、观察、保管结果，全部经它。
- **整包里照样带着它**：单机安装时由 control 在第一次用到时拉起，但它不依赖 control。
- **代码上隔开**：Agency 不依赖控制面的存储与命令内核，两边只共享端口合同。

它和 Herdr 像：两者都是本机 agent 的集合入口，都是后台服务加客户端。所以 Agency 做薄。终端与会话、多人观察与单人接管、工作树的创建与清理，直接用 Herdr 的；自己只写 Herdr 没有的四样：

1. 和控制面配对，每个控制面一块隔离的租户。
2. 名册：本机能派出哪些工种，带什么技能。
3. 按执行规格接单并给出派工引用，校验控制面签发的连接票据。
4. 结果和证据先由它保管，控制面确认保全后才放手。

一个例子：仓库根的 `./run` 脚本是手工版的 Agency。十家 harness 各有工作树和 session，脚本负责启动和续接。Agency 把这件事做成常驻服务，再让控制面能通过它派工。

两处缺省：

- 它自己的 web 客户端这一段不做，端口先留好，演示时用 Herdr 的原生界面和命令行看。
- 可执行文件的工作名是 `agency`，不带 `hctl2` 前缀；所有者可以另起名字。

没有采用的两种形态：写进控制面进程（harness 与控制面隔不开，接远程 Agency 时要重写）；让 Herdr 直接承担（它没有配对、租户、票据与成果保管，要给上游打补丁）。

## 六、席位、模型与小活

| 席位 | 这一段做什么 | 依据（本库里的表现） |
| --- | --- | --- |
| Claude Code（即 Fable 席） | 设计文档、开工书、状态板；三次演示的验收 | — |
| Codex | 定框架；写第 1、2、5、6 包；审别家的包 | 存储内核、Repo 注册、任务源、聊天与 Room 树四个大包都是它写的，评审意见都是一轮改完 |
| Grok | 写第 3、9 包；审 Codex 的包 | #292 里独立找到三个问题，含一个崩溃恢复窗口；P2.1 的守护进程与托管服务是它写的 |
| GLM | 写第 4 包；审第 1、6、8 包 | P2.1 的评审都实际跑了测试，#282 查出完整包测试其实是红的 |
| DeepSeek | 写第 7 包；审第 2、9 包 | #296 里指出过聊天服务器一次最多读 10 层这类实现上限；写代码还没在本库验证 |
| Qwen | 写第 8 包；审第 3、5 包 | #296、#299 审得最细，有三条建议没被采纳；写代码还没在本库验证 |
| Kimi | 审设计文档；三次演示时对照验收用例走查 | #246 里查出一条别家没提的旧裁决 |
| MiniMax | 小活 | #296 里独立找到一个 P1；写代码还没在本库验证 |
| Antigravity | 小活 | #296 的意见最少 |
| Muse | 暂时不可用，不排 | — |

规矩：

- 每个包的两位审阅者里，至少一位是 Codex 或 Grok。新审阅者的意见拿来和他们对照。
- 开工提示词由 Fable 给指针版。评审提示词由作者贴在 PR 里，所有者把它贴给两位审阅者，一包只贴这一次。
- 开了 PR 之后各家自己盯，不等所有者转告：作者定时查看评论与 CI，评审到了自己处理、写处理说明，条件满足就合；审阅者发完意见后，定时查看作者的处理说明，自己复核。所有者只在开工和开审时各贴一次提示词，之后只收结果。
- 哪家 harness 做不到一直等着（一轮对话里等不了那么久），如实报告，由所有者催一次；不要假装在盯。
- 合入与两轮规则见 §二 第 4 条。
- 小活一次只放一个，不占主线。

模型：新模型没有本库证据，只定一条规则。写代码、审代码用各家的主力档，flash 档只跑机械小活。Antigravity 用 gemini-3.8-flash。Kimi 的 K3 留给设计评审和演示走查。DeepSeek 与 MiniMax 哪一档写代码更稳，没有依据，先用所有者平时用的那档，看结果再调。

小活的头两件，编号 A、B。

### 小活 A · CI 整体超时与 Clippy 关卡（MiniMax 写，Codex 审）

- **现状**：`.github/workflows/` 下的工作流都没有设 `timeout-minutes`。#287 那次 macOS 作业挂了 40 多分钟，只能手工取消。`root//:clippy` 只把诊断写进产物文件（各目标的 `clippy.txt`），文件不为空 CI 也不失败。
- **要做的**：
  - 给每个工作流的每个 job 加整体超时。取值按近期实际耗时留余量，依据写进 PR 描述（用 `gh run list` 查近十次的耗时）。
  - 加一道检查：Clippy 的诊断产物只要不为空，CI 就失败，并把诊断打印出来。
  - 先确认当前 main 是零诊断。不是零就把诊断列出来报给所有者，不在这个包里顺手改业务代码。
- **规矩**：构建动作走 Buck2 原生目标，能用 Buck 目标或测试表达的不另写脚本；改工作流要过 actionlint 与 `root//build/tests:validation_range_test`；在 `docs/research/build-tools/github-actions-incremental-validation.md` 文末追加一条复核记录。
- **不做**：不动路径筛选、缓存与平台矩阵。
- **分支**：`minimax/ci-timeout-clippy`。

### 小活 B · 调研：Room 树和 Run 图在界面上怎么画（Antigravity 写，Fable 审）

- **为什么做**：这是 Workbench 的前置。`delivery.md` §未决问题 写明，Room 树与 Run 图的原生视角要先调研信息可视化研究与业界做法，再出设计。起点是 [`room-tree-20261001.md`](../room-tree-20261001.md)。
- **交付物**：研究根目录一份跨候选对照 `docs/research/tree-and-dag-views-<完成日期>.md`，并在 `docs/research/README.md` 的条目索引加一行。格式照 [`02-research-brief.md`](./02-research-brief.md) §通用要求。
- **要回答的**：
  1. 有层级的聊天空间怎么导航：Matrix 客户端的 Space、Slack、Discord、Zulip 一类产品，平铺列表与树各在什么时候用，层级深了怎么办。
  2. 有向无环图怎么看：GitHub Actions、Dagu、Airflow、Argo、Buildkite 一类产品怎么画运行图，节点多了怎么折叠，怎么从图跳到单个节点的日志。
  3. 信息可视化研究对树和图的导航有哪些可引用的结论，比如缩进列表、节点连线图、面包屑、焦点加上下文。
  4. 对 HCTL 的建议：只列候选画法和各自的取舍，不替所有者做决定。
- **规矩**：每条结论给出处链接和读取日期；没亲眼核对过的写「未核实」，不编数字；只写调研，不写代码，不改设计文档。
- **分支**：`agy/tree-dag-views`。已合 #304、#305。

之后每次演示完，由 Antigravity 照真实命令写一份演示手册，Grok 照着重跑来验。

这两家看三样：是否一次过，有没有多做，报告是否如实。小活 B 的观察：不是一次过（第一版把没核过的来源标成核过），范围没多做，改得快，有一次没等「可合」就合了；之后的提示词都写明「评论里没有『可合』不许合入，『修正后可合』不算」。

演示 1 之后的三件，编号 C、D、E。同一家一次只做一件：Antigravity 先 C 后 E，MiniMax 等 A 合入后做 D。

### 小活 C · 演示 1 的手册（Antigravity 写，Grok 重跑）

- **交付物**：`docs/demos/demo-1.md`（新目录），从 `docs/usage.md` 加一行链接。照 [`2026-10-03-演示1验收.md`](../../log/2026-10-03-演示1验收.md) 的顺序写：构建、安装、起服务、注册人类账号、四步、重启核对。每条命令给完整命令行与输入 JSON，输出只写要看哪几个字段。
- **规矩**：每条命令自己真跑一遍；要手工绕的（人进 Gitea、Cinny 要单独起）如实写「目前要手工做」，不写成已经有命令；跑不通的写「跑不通」并留原样输出，不编。不改代码、不改设计文档。
- **验**：Grok 照手册在干净目录从零跑一遍，哪一步跑不通写在 PR 评论里；都跑通才「可合」。
- **分支**：`agy/demo-1-manual`。

### 小活 D · 密钥后端开关（MiniMax 写，Codex 审）

- **现状**：`SecretStore::detect`（`src/crates/foundation/src/lib.rs`）只要系统钥匙串可用就用它，没有办法显式选私有文件后端；`user_file` 构造函数已经存在，注释写明它是获准的后端。后果见 §三 缺口 5。
- **要做的**：给 control 加一个显式设置，选 `system-keyring` 或 `user-file`（落在 control 的配置里，`hctl2 init` 或 `hctl2 start` 能指定；缺省不变，仍是有钥匙串就用钥匙串）；`hctl2 status` 的 `policy.credential_storage` 如实报告实际在用的后端；`room-cli-test` 等原生测试显式用 `user-file`，不再碰开发者的钥匙串；`docs/usage.md` 加一句怎么选；`docs/research/libs/keyring.md` 文末追加一条复核记录。
- **规矩**：不改安全策略面的约束（密钥进 secret store；私有文件后端本来就是获准的）；不把缺省改成文件；不加新依赖；选项名和配置形状自己定，写进 README。
- **分支**：`minimax/secret-backend-switch`，小活 A 合入后开。

### 小活 E · 人进本地 Gitea（Antigravity 写，Codex 审）

- **现状**：control 用自己的管理员账号在本地 Gitea 建仓库，仓库私有；人没有账号，也没有协作权，hctl2 没有命令给人开。验收时是用包里的 `gitea admin user create` 和 control 的管理员令牌手工做的，见 [`2026-10-03-演示1验收.md`](../../log/2026-10-03-演示1验收.md) 第 4 步。
- **要做的**：一条公共命令（挂在 `hctl2 repo` 或 `hctl2 services` 下，自己定），输入用户名，为本机的人在本地 Gitea 建账号（已存在就复用）并给 control 建的那个仓库协作权；两步确认；令牌或初始密码按安全策略面处理，不进日志，不打印到普通 stdout 以外的地方；原生测试加一条。
- **规矩**：复用 control 已有的 `gitea admin` 与平台适配器路径（`src/apps/control/src/scm.rs`），不自写 HTTP 客户端；不碰 GitHub 路径；不改约束。
- **分支**：`agy/gitea-human-account`，小活 C 合入后开。两轮修不完交回 Codex。

## 七、先不做的与之后的

这一段先不做：

- **Vikunja**：本地任务服务器，只在要用本地任务源时才需要（§二 第 7 条）。演示用平台的 issue。
- **判断点第一批**：[`room-judgment-20261002.md`](../room-judgment-20261002.md) §五 做哪个仍待所有者拍板，开工排在演示 3 之后。这期间人开 Topic、改提要的动作都有记录，以后可以当标注用。
- **第三家起的 harness 适配**：演示 3 只接 Codex 与 Claude Code。其余各家以后各写各的，可以当小包。
- **Linear 的验证。**
- **Agency 的 web 客户端。**

不挡演示的遗留问题：

- 结果未知的外部写入要不要有一个有界的出口（#288 评审里留下的约束层问题），待所有者定。
- 判断点任务书里「@ 时的缺口」从哪条消息算起，随判断点第一批一起定。

演示 3 之后还有两大段：Run 与施工图、评审关卡（P2.5，接 Dagu），以及 Workbench（P3）。先做哪段，到演示 3 时由所有者定。Workbench 的界面零件互相独立，适合多家一起写。

## 八、开工提示词

第 1 包，给 Codex：

```
你在 yesme/hctl2 做「第 1 包 · Project、名册、Request」（P2.2 的最后一包）。开工书：main 上 .memo/design/p2-control-20260906/07-demo-kickoff.md §四 第 1 包；任务书是同目录 05-p22-kickoff.md §四 里标题为「辛 · Project 与 Request」的一节（先读 05 的 §一、§二，与 04-p21-kickoff.md §二、§五）。任务书写于 v0.18.11，与现行约束不一致处以约束为准。主 Room 的接线见 src/crates/chat/README.md。分支 codex/p22-project-request，base main，一个 PR；评审席位 Grok 与 GLM 各自独立审，评审提示词由你贴在 PR 里。开了 PR 之后自己盯评论与 CI，不等所有者转告：评审到了自己处理并写处理说明；两席都写「可合」且 CI 绿后由你合，合后报所有者。回报：PR 编号、分支、crate 与 target 清单、失败用例清单（对照 CT-PROJECT、CT-REPO 各行）、没按任务书做的地方及原因。
```

第 2 包，给 Codex，第 1 包合入后发：

```
你在 yesme/hctl2 做「第 2 包 · Agency 服务骨架与端口」。开工书：main 上 .memo/design/p2-control-20260906/07-demo-kickoff.md §四 第 2 包与 §五（先读 §一 到 §三）。本包还要给第 3、4、5 包定框架，写在对应 crate 的 README。分支 codex/agency-port，base main，一个 PR；评审席位 Grok 与 DeepSeek 各自独立审，评审提示词由你贴在 PR 里。开了 PR 之后自己盯评论与 CI，不等所有者转告：评审到了自己处理并写处理说明；两席都写「可合」且 CI 绿后由你合，合后报所有者。回报：PR 编号、分支、crate 与 target 清单、失败用例清单（对照 CT-PARTICIPANT、CT-CONNECTION 各行）、给后面三个包留的任务说明在哪、没按开工书做的地方及原因。
```

第 1 包的补丁 1a，给 GLM，可与第 2 包同时发：

```
你在 yesme/hctl2 做「第 1 包的补丁 1a · 人在聊天客户端里看到什么」。开工书：main 上 .memo/design/p2-control-20260906/07-demo-kickoff.md §四「第 1 包的补丁 · 1a」（先读 §一 到 §三，§三 末尾的演示 1 验收说明了为什么要做）。约束是 docs/design/spec/project.md（v0.19.2）里「创建 Topic Room」一段与 §Room 与消息 里承载 Space 成员那句；失败用例在 docs/design/contract-tests.md CT-PROJECT 对应的两行。现有接线看 src/crates/chat/README.md 与 src/crates/project/README.md。分支 glm/topic-opening-and-space-members，base main，一个 PR；评审席位 Grok 与 Qwen 各自独立审，评审提示词由你贴在 PR 里。开了 PR 之后自己盯评论与 CI，不等所有者转告：评审到了自己处理并写处理说明；两席都写「可合」且 CI 绿后由你合，合后报所有者。两轮修不完交回 Codex。回报：PR 编号、分支、改了哪些 crate 与 target、失败用例清单（对照 CT-PROJECT 那两行）、没按开工书做的地方及原因。
```

小活 C，给 Antigravity，现在可发：

```
你在 yesme/hctl2 做「小活 C · 演示 1 的手册」。任务说明：main 上 .memo/design/p2-control-20260906/07-demo-kickoff.md §六 小活 C；照着写的实录是 .memo/log/2026-10-03-演示1验收.md。每条命令自己真跑一遍，跑不通的如实写「目前要手工做」或「跑不通」，不编输出。分支 agy/demo-1-manual，base main，一个 PR；Grok 照手册从零重跑来验，评审提示词由你贴在 PR 里。开了 PR 之后自己盯评论与 CI，不等所有者转告；评论里没有「可合」不许合入，「修正后可合」不算；Grok 写「可合」且 CI 绿后由你合，合后报所有者。回报：PR 编号、分支、手册路径、哪几步要手工做、哪几步没跑通。
```

小活 D，给 MiniMax，小活 A 合入后发：

```
你在 yesme/hctl2 做「小活 D · 密钥后端开关」。任务说明：main 上 .memo/design/p2-control-20260906/07-demo-kickoff.md §六 小活 D；起因见 .memo/log/2026-10-03-演示1验收.md「钥匙串」一节。分支 minimax/secret-backend-switch，base main，一个 PR；Codex 审，评审提示词由你贴在 PR 里。开了 PR 之后自己盯评论与 CI，不等所有者转告；评论里没有「可合」不许合入，「修正后可合」不算；Codex 写「可合」且 CI 绿后由你合，合后报所有者。回报：PR 编号、分支、开关怎么用、哪些测试改成了不碰钥匙串、没按任务说明做的地方及原因。
```

小活 E，给 Antigravity，小活 C 合入后发：

```
你在 yesme/hctl2 做「小活 E · 人进本地 Gitea」。任务说明：main 上 .memo/design/p2-control-20260906/07-demo-kickoff.md §六 小活 E；起因见 .memo/log/2026-10-03-演示1验收.md 第 4 步。分支 agy/gitea-human-account，base main，一个 PR；Codex 审，评审提示词由你贴在 PR 里。开了 PR 之后自己盯评论与 CI，不等所有者转告；评论里没有「可合」不许合入，「修正后可合」不算；Codex 写「可合」且 CI 绿后由你合，合后报所有者。两轮修不完交回 Codex。回报：PR 编号、分支、命令怎么用、失败用例、没按任务说明做的地方及原因。
```

Kimi 对演示 1 的契约走查（§三 的规矩），现在可发：

```
你在 yesme/hctl2 做演示 1 的契约覆盖走查，不改文件、不开 PR。先 git fetch origin，读 main 上 .memo/design/p2-control-20260906/07-demo-kickoff.md §三（含末尾的演示 1 验收），再读 docs/design/contract-tests.md 的 CT-PROJECT 与 CT-REPO 两节。对照 #307 的 PR 描述里「CT-PROJECT / CT-REPO 失败用例对照」那张表，以及 src/crates/project/README.md「Buck 与 CT 对照」一节，逐行回答：这一行 PR 说覆盖了，代码和测试里是否真有对应的失败用例；说没覆盖的，理由是否成立。只列对不上的行，每行写：CT 原文位置、PR 怎么说、你看到的。写在 #307 的评论里，以「Kimi · 演示 1 · CT 走查」开头。
```

之后各包的开工提示词，在前一包合入后由 Fable 给。

## 九、轻审怎么审本文

三样：

1. §二 的裁定有没有记错或记漏。
2. §四 九个包的范围与依据是否忠于现行约束（v0.19.0）与交付文档；有没有漏掉 `delivery.md` §纵向切片 A 的步骤，有没有把 P2.5 的活提前塞进来。
3. `delivery.md` §本地 Agency 参考实现 的改写是否忠于所有者原话，有没有借交付文档去改约束。

不审 P2 计划本身，`01-plan.md` 已拍板。

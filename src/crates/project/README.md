# Project、Room 名册与 Request（P2.2 第 1 包）

实现依据：[任务书](../../../.memo/design/p2-control-20260906/05-p22-kickoff.md#辛--project-与-requestgrok)、[Project 约束](../../../docs/design/spec/project.md)、[跨模块 Request 回路](../../../docs/design/spec/connections.md#跨模块-request-回路)。本文说明实现入口与验证范围，不定义新约束。现行约束优先于旧任务书。

## 模块与边界

| 位置 | 职责 |
| --- | --- |
| `src/model.rs` | Project 定义、不可改写的 Room 选入记录、类型化 Request 与命令输入 |
| `src/commands.rs` | 纯读取预览、Project / Room 事务、归档与恢复、名册与多房间动作准入 |
| `src/requests.rs` | 去重与取代、解决 / 取消 / 截止、唯一投递与接收回执 |
| `src/invocation.rs`、`src/invocation/lifecycle.rs`、`src/invocation/results.rs`、`src/invocation/write.rs` | Invocation 的预览、授权、状态、撤权与重试、只读回答准入及投影意图、写入边界与租约启动；外部动作由 Control 主链接 |
| `src/views.rs` | 阻塞列表、待你处理、Overview 与闲置提醒的只读投影 |
| `src/memo.rs` | 发布 Memo 的预览与准入、只追加 revision、指针清单与精确读回 |
| `apps/control/src/projects.rs` | Query / Preview / Submit、逐房间外部投递、Request 重启恢复与每 5 秒截止检查 |
| `apps/control/src/memo.rs` | Memo 的 Query / Preview / Submit 适配，`memo.list` 用控制面自己的时钟 |
| `apps/cli/src/project.rs` | 公共 `hctl2 project` 与 `hctl2 request` 两步确认入口 |
| `apps/cli/src/memo.rs` | 公共 `hctl2 memo`：正文按文件提交，两步确认 |

沿用 Store 的 SQLite 事务、命令内核与治理材料 Git 裸库，不建另一套存储。Project 身份记录引用精确版本的 `project_details`；目标、范围、角色、默认规则与设置可追溯。Task 身份与契约 Revision 保存接受时的 Project 引用，后续改缺省不回写历史。

创建 Project 在同一事务写 Project、唯一主 Room 与建房 outbox；随后沿 Chat 已有原生 Matrix 路径创建房间。外部失败保留已准入事实与原意图，重试原 key 或 `project resume`，不另建 Project / Room。同 Repo 下两条创建命令产生两个独立 Namespace。未选 Participant 也能查看主 Room。

归档按 Project 归属列出全部 Room，与 Matrix 层级读数无关。开放 Room 转只读，Task / Request 不改生命周期；它们的写命令由 Project 的只读前置拒绝。恢复只复原本次归档转只读的 Room，预先关闭或只读的 Room 不复活。非终态 Run、写入型 Invocation、活动租约、未决意图与副作用拒绝归档；Repo 范围的对象按来源 / 归属者或明确 `project_id` 关联。未来模块需沿此形状接入检查，未知状态保守视为未决，本包不声称已经实现这些模块。

Room 名册与 Matrix 成员不是同一件事。`select` 冻结连接约束里的选入字段，只更新该 Room 的名册引用，不改外部 Binding。既有选入记录不回写。可先选入精确 `chat::topic_id(project_id, topic_command_key)`，再创建 Topic；创建核对确认的名册版本。第 5 包主体首批已接入接受目录、Project 选人策略、Profile 与 Skill 核验，依赖版本在名册事务中再核；optional Skill 缺失在预览结果中列出。Invocation 的授权与实际激活见下文，Run 仍未接线；候选校验详见 [Participant 的实现范围](../participant/README.md#第-5-包主体--选入校验与-worker-profile)。选入名册本身不派工。

`members` 明确列出本 Project 的 Room Binding 引用和 Matrix 用户 ID，逐房间写原生成员状态并回读；不沿 Space 推断继承。部分失败返回逐房间结果、`ROOMS_PARTIAL` 与非零退出，已成功部分不伪称回滚。重试沿原 effect，邀请 / 移除先回读当前成员状态；Unknown 只回读，不能证明原结果时不重发。权限等级调整未提供公共命令，后续若接入须复用逐房间结果规则。补丁 1a（v0.19.2）：目标为主 Room 时，同一意图的投递把同样的邀请 / 移除同步到本 Project 全部承载 Space（逐 Space 回读；这是 content，不进名册、不改加入规则）。

## Request 的当前来源

本阶段只接 Task 契约采纳这一种已有类型化动作。创建须固定精确 `task_state` 与版本、受影响 Revision、问题、目标人 / 角色、阻塞范围、去重根、权限、schema 和截止策略。`permissions` 当前只接受 `{"action":"task.adopt"}`，`input_schema` 只接受 `hctl2.task.Adoption.v1`；其他动作拒绝，不把任意 JSON 解释成命令。受影响 Revision 可为空（首次采纳），有值则核当前 Task 的精确 Revision。Role 的处理人由 Project 的显式 `role_members` 决定，不按 Participant 名册猜人。

Project 保存 Request 生命周期，Task 仅保存自己的 `RequestBlocker`（Request ID、冻结归属者、等待 / 投递 / 处理结果）。同根同字段去重；变更字段另建并取代旧 Request。这个 Task 适配器对同一个精确状态的契约输入不同时开多个独立去重根，避免两个答案互相阻塞；新版本仍可另建 Request，旧答案不能推进它。

解决先预览，再同事务比较 Request、Task blocker 和 Task 状态，写解决摘要、actor / 委派与唯一 delivery outbox。原 Task reducer 校验答案；契约字节在事务外保存，解决事务准入并冻结原 Task Plan。接收方在一个事务里应用它、推进精确 blocker、保存回执并确认投递，响应丢失后的重试不多建 Revision。直接 `task adopt` 不能绕过开放 Request，包括预览之后才创建的 Request。已接受的授权不被后续 Project 缺省修改撤销；归档仍拒绝推进。

截止和取消只把来源的待办动作标为失败 / 放弃，不伪造答案，不完成或取消 Task。来源已变时保留过期历史，拒绝推进新状态，且不阻止其他 Request 截止。`task show` 投影 `request_blockers`；打开 Topic 或阅读待处理面板不解决 Request。升级 Topic 使用 Chat 的 Request 来源和冻结 blocker，仍须人确认提要；无 Message 也能机械起草，不补造消息。

待你处理本阶段接开放且有权回答的 Request、待采纳契约变化的 Task。按原动作去重，列对象、原因、动作后果、未处理影响与返回入口；失败不移除，成功后退出，查询不写事实。其他人或角色无权处理的 Request 不计。待投递发布评审、候选交付、Run 超时来源尚无业务模块，留后续包。Overview 提供目标、Task 健康度、计数、按已准入事件顺序的近期活动引用与归档阻塞列表，不复制聊天历史、不代替主 Room。闲置提醒从当前原生时间线回读；仅未截止的开放 Request Topic 活跃且闲置超过 14 天才标关注，不增加待处理数，读不到单列 `unread`。

## CLI

输入字段以 [`Action`](src/model.rs) 为准，写命令共享 `--input / --key / --preview-token` 两步。CLI 自动补子命令的 `kind`，输入文件可不带它。创建示例：

```json
{"repo_id":"REPO_ID","definition":{"name":"Apollo","goal":"交付目标","scope":"工作范围","roles":[],"role_members":{},"defaults":{},"settings":{"selection_policy":{},"publish_review_requires_confirmation":false}}}
```

```sh
hctl2 project create --input project.json --key create-apollo
hctl2 project create --input project.json --key create-apollo --preview-token TOKEN
hctl2 project list
hctl2 project show PROJECT_ID
hctl2 project overview PROJECT_ID
hctl2 project pending PROJECT_ID
hctl2 project attention PROJECT_ID
hctl2 project archive-blockers PROJECT_ID
hctl2 project roster PROJECT_ID ROOM_ID
hctl2 request list PROJECT_ID
hctl2 request show PROJECT_ID REQUEST_ID
```

`update` 带 `project_id / version / definition`；`archive`、`restore` 带 `project_id / version`；`select`、`members` 带 `project_version`；`resume` 带原 `effect_id`。Request 的 `create` 带 `project_id / project_version / request`；`resolve` 带 `project_id / request_id / version / adoption`（沿用 Task `Adoption`）；`cancel` 带 `project_id / request_id / version`。查询和观察结果是 stdout JSON，失败带 `error.code / recovery_action`、非零退出，外部未确认不报成功。

## Buck 与 CT 对照

| 现行 CT-PROJECT / CT-REPO 条目（按内容定位） | 本包失败输入与证据 | 未覆盖及原因 |
| --- | --- | --- |
| 创建 Project 与唯一主 Room；同 Repo 两 Project；待确认 Repo 拒绝 | D：outbox 冲突回滚无半个 Project、同 key 重投、第二 Project 独立、待确认 Repo 拒绝；B1：实际注册与两主 Room | 本地 B1 的 Keychain 写入问题见 PR 实测记录，不冒称已通过；「未选 Participant 不能查看主 Room」这一失败情形要等派工包有数字参与者才能测，归第 5 包 |
| Project 归档拒绝清单与 Repo 所属阻塞；恢复不复活终态 | D：逐类阻塞、Repo 租约、预览后新增 Room / effect、关闭 Topic 保留；不读取 Matrix 树 | 未来真实 Run / Invocation / 租约形状在各包接入时再验 |
| Room 归属不变、Request 去重；讨论不解决 Request | D：跨 Project 拒绝、同根去重与取代、关闭 Topic 不改 Request；Chat 原有 D / R | 普通 Topic 提要内容语义校验仍沿 Chat README 的未覆盖项 |
| 闲置 15 天的开放 Request Topic，普通 Topic 不提醒 | D：13 / 15 天、未读、普通 / 已解决 Topic；N：当前原生消息时间回读 | UI 标记未实现 |
| 挂靠不继承；多房间部分失败不报全体成功；归档不依赖层级 | D：独立不可改写名册、跨 Project / 重复目标拒绝、归档全体 Room；N：原生成员邀请与移除、重复回读 | 真实多房间部分拒绝、权限等级调整与 UI 未验 |
| 待你处理去重、四问与返回入口、其他人不计、只读查询不解决 | D：单 Request 一项、Task 同动作去重、其他人不计、处理后退出、读前后完全一致；B1：Request 经公共 CLI 解决重试 | 其余三类来源归后续包 |
| Request 无 Message 升级 Topic、机械提要、关闭不解决 | D：真实 Request 记录供 Chat reducer 创建 / 关闭、归属与版本检查 | Chat D：Request Topic 机械草稿在 Chat 已有路径；完整来源链需后续 Context 交付 |
| Project 版本更新不改已接受约束、名册换人不回写旧记录 | D：保存旧选入与 Binding、Task Revision 接受版本冻结、改缺省后原 Request delivery 可确认；第 5 包首批再测真实候选与旧 Profile 引用 | 活动调用 / Run 的冻结设置仍待派工与 Run 接线 |
| Request 比较并交换、唯一投递、崩溃重试、截止不伪造 Task 终态 | D：来源在 Submit / 回执前变化拒绝，Unknown 后重新开库仅一回执一 Revision，旧 Task 预览不能绕过新 Request，过期旧来源不挡其他截止 | Run / Invocation 的来源 builder 与接收方尚未实现 |
| CT-REPO 其余注册、平台绑定、集成与评审行 | 原 Repo 测试与完整安装包测试保留；本包不改 Repo reducer | 变更 / 发布评审 / 集成仍按任务书后续包交付 |
| CT-PROJECT 其余 Invocation、Context、模型、派发与审计行 | 本包不派工；Memo 三行见[第 7 包](#第-7-包--发布-memo) | 第 2–6、8 包及 Workbench，不标本包完成 |

D = `root//crates/project:domain_test`（连用 Task / Chat reducer）；N = `root//apps/control:chat_native_test`；B1 = `root//packaging/release:room-cli-test` 中的原生安装包验收；R = 该目标保留的 Room 验收。原生测试用 lock.json 的 Tuwunel / Gitea / tea，在私有目录和独立回环端口运行，不操作开发者实例。

```sh
cd src
./buck2 test root//crates/project:domain_test root//crates/task:domain_test root//crates/chat:domain_test root//apps/control:boundary_test root//apps/control:chat_native_test root//apps/cli:cli_test root//apps/cli:task_cli_test root//build/docs/...
./buck2 test root//packaging/release:room-cli-test --config hctl2.zstd_preset=fast
./buck2 build root//:clippy root//packaging/release:room-cli-clippy --config hctl2.zstd_preset=fast
```

新 crate / target 为 `root//crates/project:{project,domain_test,clippy}`；Control 和 CLI 使用既有 target，完整包 B1 纳入 Release 目标，不让普通 Code 检查重复制作完整包。没有新三方依赖或脚本。

## 第 5 包后半段任务说明

`Selection` / `Skill` 保留原 JSON 形状，从 Participant crate 重导出；Project 继续拥有 Room 名册及其写入事务。后半段的 `room roster` 命令复用 `Action::Select`，不另造名册存储或改变 Binding。候选字段、只读 Profile 范围和策略格式见 [Participant 接口说明](../participant/README.md#第-5-包主体--选入校验与-worker-profile)。

输入：`project_id / project_version / room_id / topic_command_key? / roster_version? / selections`，以及命令幂等 key。`prepare` 返回 `Plan`，结果含新 `roster_version`、选入引用、候选校验结论和 `optional_skill_degradations`；`admit` 提交经预览确认的计划，旧 Project、Room 或名册版本拒绝。已有 `project select` 的 CLI / RPC 接法可以直接复用。`project.roster` 展示记录，不重新授予派工资格；历史名册是否已执行候选校验，应看原选入命令，当前派工仍须预览。

Invocation 的领域入口见下节，最少 preview / start / show 已由主链提供。后半段只接其余命令与只读投影，不复制 reducer，不把名册提交成功当作派工成功。

| 后半段命令 | 输入 → 既有入口 → 输出 | CT 与边界 |
| --- | --- | --- |
| `invocation list` | Project ID → `Store::list("room_invocation")` 按获准 Project 过滤，加 `invocation::lifecycle` → 原授权引用、当前状态及独立 `state_version` | CT-PROJECT：只读查询不授予派工权，不读另一 Project |
| `invocation cancel` | `End {key, project_id, invocation_id, state_version, outcome: cancelled, reason}` → 确认后 `invocation::end` → 撤权、终态与 `cleanup_pending` | CT-CONNECTION：重投不新建停止意图，不能把待清理报告为已隔离；沿既有 Preview / Submit token 接线，不另写状态机 |
| `invocation retry` | `Input` 新 key、原调用精确 `retry_of` → 既有 `invocation.start` 预览 / 提交 → 新 Invocation ID、原授权引用与待启动状态 | CT-PROJECT：旧调用须终态且无有效授权，不能复用旧 Bundle；CLI 可转换输入，不需要新 reducer |
| `invocation show` | Project ID 与 Invocation ID → 已有 `invocation.show` Query → 冻结调用、状态、派工意图、准入回答字节、待投影 / 清理意图 | 已实现最少命令；不把 Room 投影成功当作 Task 完成 |

## 第 5b 包 · Invocation 领域与主链接线

`invocation::prepare` 读取当前 Project、Room、名册、接受过的工种与 Profile；`start` 重新核对预览，在一个 Store 事务里写授权、Execution Spec、待启动状态、派工意图与 prepare outbox。领域入口没有预填 Dispatch，也没有网络调用。Control 的 `dispatch.rs` 再走准备、持久化映射、激活与回读；Context 先用第 4 包的 `save_assembly` 保存、准入，只保存 Context 不授予派工权。本次用脚本执行体走通真实 CLI，不声称已经完成真 harness 的演示 2 验收。

授权在 `room_invocation`，状态在 `invocation_state`，两者是同一个 Invocation 的存储部分，不是两个业务对象。`state_version` 供状态比较并交换；`invocation_version` 在授权根的引用里，不由状态变化推出。确认激活后，原 Invocation 的内部 reducer 提交 `record_started`，普通客户端不能提交运行状态。取消、失败或丢失在同一事务里推进状态并使授权失效；待投递动作撤销，已经尝试的动作保留 Unknown，并登记按原派工定位的清理 outbox。Control 回读停止报告，不把停止请求当报告，也不把报告当隔离证明。

`end` 对 human 只接受取消；失败、丢失由带原授权的内部 reducer 提交。它不接受“完成”命令，不从进程退出或屏幕内容推断完成。`admit_result` 核精确归属者、绑定、Spec / Bundle、逐项输出授权、schema、证据与保全字节，在一个事务写只读回答、Completed 状态与 Room 投影 outbox，不完成 Task。终态本身使执行授权失效，授权根不必因成功回答而改版。重试仍用新 key、新 Invocation 与 Bundle，`retry_of` 留原调用的精确引用；旧调用未终态或仍有有效授权时拒绝。等待输入的 Request 接线尚未实现，合法边表不是该能力的证明。

`prepare` 冻结只读 Profile、Project 版本与选人策略、Room、独立名册、必需 Skill、预算、截止和发布确认缺省。`start` 从它生成 Spec，不能注入更宽权限或换执行者；读取已经准入的 Manifest / Bundle 与材料原文，核实际 consumer、预算、请求正文、必需 Skill 字节和 Topic 提要。token 数未知仍是 `null`。Control 组装器只读本 Room 的服务器窗口、已确认提要及来源、显式 `task_id` 的评论；真实必需 Skill 原文端口尚无，遇到该项拒绝，不补造。范围与缺口见 [Control README](../../apps/control/README.md#第-5b-包--只读派工主链)。

`current_authorization(store, owner, now_ms)` 核原授权、非终态、冻结截止和 Project 状态，不重新套更新后的名册或策略。Control 的 Pending prepare / activate 对正式 `room_invocation` 使用它；测试端口的 `authorized_invocation` 不是领域授权。Unknown 仍只回读。历史 Root 与 Context 保留，不在重放时复活旧授权。

| CT 内容 | 领域段已有的失败输入（`domain_test`） | 本次主链 / 后续分工 |
| --- | --- | --- |
| CT-PROJECT：human 发起、精确候选、只读调用归 Project | 模型来源、显示名、跨 Room / Project、未获准 Profile、超预算、过期截止 | 本次接 CLI preview / start / show；provider human 事件与批准建议的来源链未接 |
| CT-CONNECTION：冻结与四步启动 | 旧预览、未准入 Context、错 consumer、请求或必需 Skill 未送达；outbox 冲突后四类记录均回滚 | 本次接在线选材、真实四步启动、选定来源权限过滤与脚本执行体 |
| CT-PROJECT / CT-PARTICIPANT：合法边、取消、重试 | 激活未确认、终态复活、取消重投、新调用复用旧 Bundle、普通客户端提交失败 / 丢失 / 完成 | 本次接截止、明确身份丢失与 Proposal 准入；等待输入仍未接 |
| CT-CONNECTION：撤权与未知外部结果 | Unknown prepare 后取消仍须保留停止依据；停止 outbox 冲突时撤权和状态一起回滚 | 本次接原派工停止与回读、重启恢复；不报告隔离成功 |
| 活动执行引用原记录 | 换名册或改选人策略后旧 Spec 被改写时失败；运行状态改变使原授权引用变更时失败 | 本次接结果投影、Task 不被完成、内部观察 / 停止票据；公共 Terminal 留后半段 |

后半段可消费的输入与输出：`invocation::Input {key, project_id, room_id, task_id?, target, profile, request, budget, deadline_ms, retry_of?}` → `Preview`；`start(Preview, Assembly)` → Invocation ID、原授权引用、`state_version`、prepare effect 与 Spec 摘要。控制面只接受原输入，不接受客户端自行组装的 Preview / Bundle。`invocation / lifecycle` 分别读冻结记录与状态；取消用上表的 `End`。`record_started / admit_result` 及失败 / 丢失路径仅供原内部 reducer，不开放成客户端命令。

### 本次主链的 CT 与失败输入

| CT 内容 | 本次证据 | 未覆盖部分 |
| --- | --- | --- |
| CT-PROJECT：human、候选、Room 状态、Topic 提要 | D：终态根版本仍匹配却失权，普通客户端不能推进运行；缺提要 / 非活跃 Room 拒绝，已确认提要未实际交付拒绝；B：从真实配对到一次调用 | provider 的 human 事件来源与批准建议未接；B 的 Topic / 显式 Task 派工分支代码有、未测 |
| CT-CONNECTION：冻结 Context、四步、未知响应与字节保全 | B：错 token 拒绝，同 key 重投与重启不重派；P：保全后确认、响应丢失原映射回读、第一份冲突不挡第二份；Context 原生用例保留 | required Skill 字节取回未实现；额外 Memo / Artifact 和平台评审评论线未接 |
| CT-PROJECT / CT-CONNECTION：终态与结果准入 | D：取消 / 截止后仅留保全，schema 不支持拒绝后另一回答可准入，投影冲突回滚准入与状态；Task 仍未完成 | Agency 明确报已映射派工不存在时自动丢失代码有、未测；waiting_input Request 未接 |
| CT-CONNECTION / CT-PARTICIPANT：停止、票据与恢复 | D：未投递取消、Unknown 保留清理、确认无副作用拒绝不清理；P：票据分权、写者栅栏、停止与截止；B：Matrix 停止后结果仍准入，重启且 Agency 离线时投影恢复 | 主链取消 / 自动截止的跨服务停止与超过一页观测的回读代码有、未测；不报告物理隔离成功 |

D = `root//crates/project:domain_test`；P = `root//agency:control_port_test`；B = `root//packaging/release:room-cli-test`。原 Invocation 领域段的失败输入仍在上表对应目标中；本次不改约束版本。

## 第 6 包 · 拆分 2：写入预览、租约与封存准入

`invocation::Input` 新增可选 `write: WriteInput { change_set_id?, baseline_commit, target_branch, allow_update }`。省略 ChangeSet ID 时，按本控制面、Repo 和调用键派生新 ID；有 ID 时只接受同 Repo、同平台绑定版本、同基线且旧租约已撤销的 ChangeSet。基线要求 40 位小写 SHA-1，Git 对象存在性及封存回读由 control 在准入事务外执行。只读 Profile 不能附写入边界；写入型必须给边界。

`review_policy_input` 从当前 Project、Repo 与 Profile 派生策略输入和内容寻址的策略 ID。Control 调 `repo::review::freeze_policy` 保存，领域预览只读核对它。策略固定 `branch_rule = hctl2/{change_set}`、`description_source = none`、`audit_scope = minimal`，确认缺省来自 Project。`WritePreview` 列出待启动租约、策略精确引用及 `publish_for_review_not_integration`。保存策略定义不发租约，也不登记发布意图；未启动的旧预览不因策略记录已存在而获得授权。

确认启动在原授权事务里比较当前版本并取得租约，同时保存原调用、Spec、状态与 prepare outbox。Spec 的 `base`、`write_lease`、`review_publish_policy` 固定预览边界；Context 必须实际交付 `write-boundary/<consumer>` 的规范字节，只写引用或送错字节不能启动。Project 此后改变确认缺省不改已启动调用。脚本执行体沿原四步启动；Herdr 的只读限制不在本段放开，真 harness 写入留第 9 包。

取消、失败或丢失同事务把当前租约转为撤销中。prepare 从未发出并在事务内取消，可以直接撤销；其他情况要保存原派工的停止报告，`confirm_write_stop` 在写报告的事务里核原调用、Spec 与租约。Agency 在停止时确认从未激活，或证明脚本执行体退出，才可撤销；清理开始时看到 Prepared 不算证明。逻辑取消、停止请求、叙述和一轮结束都不算退出。没有证据继续拒绝下一个写入者，不假称已经隔离。

同一输入新增可选 `review_change_set_revision: store::Reference`，限本 Repo 的精确 `changeset_revision`。旧输入省略这两项时序列化形状不变。Control 的 Context 读取钩子已经留好；拆分 2 选择该来源仍返回 `REVIEW_LINE_NOT_CONFIGURED`，第 10 条接原生评论读取与冻结，不暗中忽略选择。

`admit_result` 对没有封存输入的写入型返回 `CHANGESET_RESULT_REQUIRED`，防止只读回答绕过 ChangeSet 准入。`sealing_input` 核原提案字节、活跃归属者和租约，给 control 在事务外封存；`admit_sealed_result` 重核这些边界，同一事务接受 Revision、结果、Room 投影与发布意图。`Publication` 的策略必须等于 Spec 冻结的精确引用，提交者取原调用保存的 DirectClient，不取模型提案。事务只登记发布，worker 由 Claude 提供，本段不复制它。

| CT / 边界 | 会失败的输入与目标 |
| --- | --- |
| CT-REPO：租约待启动、同一 ChangeSet 单写、撤销证明 | D：`write_invocation_preview_does_not_grant_and_start_freezes_policy_baseline_and_lease`、`cancelling_unsent_write_revokes_in_the_same_transaction_and_next_writer_has_new_generation`、`cancelling_possibly_started_write_without_exit_proof_keeps_the_original_lease_busy`；R：`second_writer_cannot_take_active_or_revoking_lease_even_after_restart` |
| CT-REPO：停止请求不等于撤销证明 | D：`stopped_write_requires_original_dispatch_exit_not_a_turn_or_stop_request`、`agency_confirmed_never_started_write_can_release_its_lease`；C：`writer_stop_page_skips_cancelled_or_turn_only_pages_until_physical_exit`；A：`prepared_cancellation_records_never_started_before_activation_can_race_it` |
| CT-CONNECTION：实际送达冻结边界 | D：`writer_cannot_activate_without_delivered_exact_changeset_boundary` |
| CT-REPO：只读与写入结果分开 | D：`read_only_profile_cannot_smuggle_a_write_boundary_into_the_preview`、`writing_call_cannot_be_completed_as_a_read_only_answer`；P：`write_profiles_require_scoped_write_permission_and_cannot_grant_commands` |
| CT-REPO：精确评审版本来源 | D：`selected_review_version_is_exact_repo_scoped_and_old_inputs_keep_their_encoding`；平台评论实际冻结留第 10 条 |
| CT-REPO / CT-CONNECTION：封存与提案准入共用事务 | D：`write_result_and_revision_share_admission_and_completion_does_not_prove_writer_stopped`、`write_projection_conflict_rolls_back_revision_result_completion_and_lease_revocation`、`cancelling_after_git_seal_but_before_admission_leaves_only_saved_proposal` |
| CT-REPO：精确写入边界、人的独立封存 | D：`write_result_rejects_each_boundary_change_without_revision_result_or_completion`、`human_seal_is_an_independent_admission_and_replay_keeps_its_first_observation`；CLI：`human_seal_real_cli_reads_exact_git_identity_diff_and_replays_after_input_disappears` |

D = `root//crates/project:domain_test`；R = `root//crates/repo:changeset_test`；C = `root//apps/control:unit_test`；P = `root//crates/participant:profiles_test`；A = `root//agency:port_test`。残留预览、人的接管与丢弃沿既定拆分 5，不以本段的租约领域测试冒充验收第 8 条全链。

## 第 5 包后半段 · 其余命令与只读投影

`list` 是只读投影，不是新的授权入口：Control 先按 Project 过范围门，再把 `Store::list("room_invocation")` 按获准 Project 的 `Scope` 过滤，对留下的每条走 `invocation::lifecycle`，输出原授权引用、当前状态、`reason` 与独立的 `state_version`。它不读另一 Project 的记录；换别的 Project 的 ID 在取记录时以 `NOT_FOUND` 失败，不按裸 ID 跨范围取。

`cancel` 不另开状态路径。`cancel_preview` 是 `end` 那批规则的只读投影，供人确认后果：它什么都不写、什么都不发放，声明的 `state_version` 必须是当前值（否则 `VERSION_CONFLICT`），已终态或已失权的调用以 `INVALID_TRANSITION` 拒绝，`cleanup_pending` 与 `isolation_confirmed=false` 分开报——排队的停止不是已确认的隔离。确认后 Control 把原 `End` 交给既有 `invocation::end`，撤权、终态、待投递动作撤销与清理 outbox 都在它的事务里；`end` 的比较并交换仍是「提交时什么才算当前」的权威，预览不替代它，也不在这里重算。失败与丢失仍由带原授权的内部 reducer 提交，`end` 对 human 只接受取消。

`retry` 没有新的领域入口。CLI 读 `--retry-of` 指向的原调用文件，取它的精确 owner 引用填进 `Input.retry_of`，再用自己的命令 key 走既有 `invocation.start`；输入文件自己声称的 `retry_of` 与 `--retry-of` 不一致时 CLI 先拒绝，不发请求。旧调用未终态或仍有有效授权由 `start` 的既有检查以 `RETRY_NOT_ALLOWED` 拒绝，Bundle 由新调用自己冻结，不复用旧的。

`room roster show` 复用 `project.roster`，返回选入记录本身，读取不重新授予派工资格；`room roster select` 复用 `Action::Select`，不另造名册存储、不改 Binding。按领域对象命名的 `room roster …` 是正名，既有的 `project roster | select` 降为文档里写明的别名：本批两套都留，收敛成一套放第 8 包（所有者 2026-10-06 裁定）。保留一个候选就是把整份名册按它的确切版本重发，版本不符以 `VERSION_CONFLICT` 拒绝。

## 第 7 包 · 发布 Memo

依据 [Project 约束](../../../docs/design/spec/project.md#contextmemo-与-artifact)的 Memo 一段与[写入约束](../../../docs/design/spec/project.md)「Memo | 发布 revision 只追加」。Memo 只由 `memo.publish` 一条命令产生：预览纯读、确认在同一个 Store 事务里保存正文材料并准入、写 revision 与指针。原始消息、执行日志与模型总结都不经这条路，因此不会成为 Memo；Room 提要、冻结来源与自动归纳都不触发它。

| 形状 | 说明 |
| --- | --- |
| `memo_revision/{memo_id}:{revision}` | 只追加的一个已发布 revision，记录版本恒为 1；它的唯一准入材料就是正文定位符，由准入填写，调用方不能提供 |
| `memo/{memo_id}` | 当前指针，版本号等于 revision 号，值是精确 revision 引用。发布新版本只移动它，不改写历史 |
| `memo_command/{sha256(key)}` | 命令记录，供同 key 重放回答原结果 |

`Memo` 固定 `memo_id / revision / applicability / author / sources / content_digest / supersedes / expires_at`。`author` 取认证连接的 principal，不是 payload 字段（`deny_unknown_fields` 使带 `author` 的输入直接解析失败）。`sources` 是逐条核对过 digest 的精确冻结 Message / Artifact 引用，跨 Project 的引用拒绝；`content_digest` 是正文字节的摘要，与准入材料的 `byte_digest` 必须一致。

更新是 `supersedes` 指名精确前一 revision 的新 revision：首个 revision 的 `supersedes` 为空，对已发布的 Memo 提交空 `supersedes` 以 `SUPERSEDES_REQUIRED` 拒绝，`supersedes` 不是当前已发布的那个 revision 以 `VERSION_CONFLICT` 拒绝。旧 revision 仍按精确 revision 读回，读回不依赖指针、搜索或"最近"。已发布 revision 不可改写：同版本换内容被记录投影拒绝；即便记录被换掉，指针与 `at()` 也不再为它服务。

正文经 `--body` 的文件字节进入命令，准入时保存为治理材料并绑定本命令的幂等 key。保存不等于准入：未准入的正文读不回来，别的命令 key 保存的字节也不能被本命令认领。归档 Project 拒绝发布（`PROJECT_READ_ONLY`），读回仍可用——[归档拒绝清单](../../../docs/design/spec/project.md)没有逐项列 Memo，这一条按「写入型命令由 Project 的只读前置拒绝」的既有规则处理。

`memo list` 给出指针清单：机械过滤已过有效期的 Memo，已被取代的 revision 本来就不是指针；`items` 保留全部当前 revision 与 `expired` 事实供人看。`now` 由控制面自己的时钟给出，调用方不能指定过滤时刻。显式 `memo show --revision` 不受这份清单影响。

### CLI

```sh
hctl2 memo publish --input memo-action.json --body memo-body.md --key publish-one
hctl2 memo publish --input memo-action.json --body memo-body.md --key publish-one --preview-token TOKEN
hctl2 memo show PROJECT_ID MEMO_ID [--revision N]
hctl2 memo list PROJECT_ID
```

`--input` 是动作本身：`{"project_id","project_version","memo_id","applicability","sources","supersedes","expires_at"}`，`kind` 与 `body` 由 CLI 补。正文用文件而不是 JSON 字符串，写下的字节就是成为治理材料的字节；`--input` 自带 `body` 而与 `--body` 不一致时 CLI 先拒绝，不发请求。预览之后改正文，原 token 不再匹配（`PREVIEW_REQUIRED`）；确认成功即消耗 token，重放要重新预览。

### CT 与失败输入

| CT 内容 | 本次证据 | 未覆盖及原因 |
| --- | --- | --- |
| 原始消息、执行日志和模型总结不经「发布 Memo」命令不会成为 Memo | D：确认过的 Room 提要与冻结来源不产生任何 Memo 记录；引用未冻结消息 `SOURCE_UNAVAILABLE`；空白正文、跨 Project 来源、payload 带 `author` 均拒绝；预览前后读数一致且三类记录为空；同 key 被别的命令占用后 `IDEMPOTENCY_CONFLICT`。C：发布前 `memo list` 为空，预览不写；预览后改正文，原 token 确认失败 | 模型总结的产生路径本身归派工与 Agency 包 |
| 过期或被取代的 Memo 不进指针清单，显式引用除外 | D：`expires_at` 前后各读一次清单，过期者出局、当前 revision 换成新的一条；`show --revision` 仍读回旧正文；另一 Project 的清单为空。C：同形走真实命令行 | 组装器消费这份清单属 Context 交付，本包只提供它 |
| 材料 Git 中新增正文但未准入时不自动成为 Memo 或推进版本 | D：`save_material` 之后 `read_material` 仍 `MATERIAL_NOT_ADMITTED`；同字节在另一命令 key 下得到不同定位符，本命令不能认领它；手工换掉 revision 记录不推进指针，`at()` 与两种 `show` 全部拒绝 | Artifact 登记未接，`artifact_revision` 只是预留的合法来源种类 |
| 发布 revision 只追加，更新以 supersedes 连接 | D：同版本不同内容 `VERSION_CONFLICT`；revision 1 / 2 / 3 逐条读回，`supersedes` 指向精确前一条；`--revision` 越界 `MEMO_NOT_FOUND`。C：revision 2 换引用、旧 revision 仍可读 | — |

D = `root//crates/project:domain_test`（`tests/memo.rs`）；C = `root//apps/cli:cli_test`（`tests/cli/memo.rs`，预置 Project、主 Room 与一条冻结消息，命令全走真实 CLI 与 RPC）。本次不改约束版本。

```sh
cd src
./buck2 test root//crates/project:domain_test root//apps/cli:cli_test root//apps/control:unit_test root//apps/control:boundary_test
```

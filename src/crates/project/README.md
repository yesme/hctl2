# Project、Room 名册与 Request（P2.2 第 1 包）

实现依据：[任务书](../../../.memo/design/p2-control-20260906/05-p22-kickoff.md#辛--project-与-requestgrok)、[Project 约束](../../../docs/design/spec/project.md)、[跨模块 Request 回路](../../../docs/design/spec/connections.md#跨模块-request-回路)。本文说明实现入口与验证范围，不定义新约束。现行约束优先于旧任务书。

## 模块与边界

| 位置 | 职责 |
| --- | --- |
| `src/model.rs` | Project 定义、不可改写的 Room 选入记录、类型化 Request 与命令输入 |
| `src/commands.rs` | 纯读取预览、Project / Room 事务、归档与恢复、名册与多房间动作准入 |
| `src/requests.rs` | 去重与取代、解决 / 取消 / 截止、唯一投递与接收回执 |
| `src/views.rs` | 阻塞列表、待你处理、Overview 与闲置提醒的只读投影 |
| `apps/control/src/projects.rs` | Query / Preview / Submit、逐房间外部投递、Request 重启恢复与每 5 秒截止检查 |
| `apps/cli/src/project.rs` | 公共 `hctl2 project` 与 `hctl2 request` 两步确认入口 |

沿用 Store 的 SQLite 事务、命令内核与治理材料 Git 裸库，不建另一套存储。Project 身份记录引用精确版本的 `project_details`；目标、范围、角色、默认规则与设置可追溯。Task 身份与契约 Revision 保存接受时的 Project 引用，后续改缺省不回写历史。

创建 Project 在同一事务写 Project、唯一主 Room 与建房 outbox；随后沿 Chat 已有原生 Matrix 路径创建房间。外部失败保留已准入事实与原意图，重试原 key 或 `project resume`，不另建 Project / Room。同 Repo 下两条创建命令产生两个独立 Namespace。未选 Participant 也能查看主 Room。

归档按 Project 归属列出全部 Room，与 Matrix 层级读数无关。开放 Room 转只读，Task / Request 不改生命周期；它们的写命令由 Project 的只读前置拒绝。恢复只复原本次归档转只读的 Room，预先关闭或只读的 Room 不复活。非终态 Run、写入型 Invocation、活动租约、未决意图与副作用拒绝归档；Repo 范围的对象按来源 / 归属者或明确 `project_id` 关联。未来模块需沿此形状接入检查，未知状态保守视为未决，本包不声称已经实现这些模块。

Room 名册与 Matrix 成员不是同一件事。`select` 冻结连接约束里的选入字段，只更新该 Room 的名册引用，不改外部 Binding。既有选入记录不回写。可先选入精确 `chat::topic_id(project_id, topic_command_key)`，再创建 Topic；创建核对确认的名册版本。候选可用性、Project 选人策略执行、必需 Skill 的未知等级拒派及活动 Invocation / Run 冻结由后续派工包接上，本包不派工。

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
| Project 版本更新不改已接受约束、名册换人不回写旧记录 | D：保存旧选入与 Binding、Task Revision 接受版本冻结、改缺省后原 Request delivery 可确认 | 调用 / Run 已冻结设置和候选校验由派工包接入 |
| Request 比较并交换、唯一投递、崩溃重试、截止不伪造 Task 终态 | D：来源在 Submit / 回执前变化拒绝，Unknown 后重新开库仅一回执一 Revision，旧 Task 预览不能绕过新 Request，过期旧来源不挡其他截止 | Run / Invocation 的来源 builder 与接收方尚未实现 |
| CT-REPO 其余注册、平台绑定、集成与评审行 | 原 Repo 测试与完整安装包测试保留；本包不改 Repo reducer | 变更 / 发布评审 / 集成仍按任务书后续包交付 |
| CT-PROJECT 其余 Invocation、Context、Memo、模型、派发与审计行 | 本包不派工、不发布 Memo；Chat / Task 已有回归目标保留 | 第 2–7 包及 Workbench，不标本包完成 |

D = `root//crates/project:domain_test`（连用 Task / Chat reducer）；N = `root//apps/control:chat_native_test`；B1 = `root//packaging/release:room-cli-test` 中的原生安装包验收；R = 该目标保留的 Room 验收。原生测试用 lock.json 的 Tuwunel / Gitea / tea，在私有目录和独立回环端口运行，不操作开发者实例。

```sh
cd src
./buck2 test root//crates/project:domain_test root//crates/task:domain_test root//crates/chat:domain_test root//apps/control:boundary_test root//apps/control:chat_native_test root//apps/cli:cli_test root//apps/cli:task_cli_test root//build/docs/...
./buck2 test root//packaging/release:room-cli-test --config hctl2.zstd_preset=fast
./buck2 build root//:clippy root//packaging/release:room-cli-clippy --config hctl2.zstd_preset=fast
```

新 crate / target 为 `root//crates/project:{project,domain_test,clippy}`；Control 和 CLI 使用既有 target，完整包 B1 纳入 Release 目标，不让普通 Code 检查重复制作完整包。没有新三方依赖或脚本。

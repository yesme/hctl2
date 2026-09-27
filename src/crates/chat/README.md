# Chat：聊天端口与 Room

P2.2 己的实现说明；依据 [Project 的 Room 与消息](../../../docs/design/spec/project.md#room-与消息)（v0.18.12）与 [P2.2 开工书](../../../.memo/design/p2-control-20260906/05-p22-kickoff.md#己--聊天端口与-roomgrok)。本包不改设计约束，也不实现 Project 创建、Request 生命周期、Invocation 或模型引擎。

## 模块与接线

| 文件 | 当前职责 |
| --- | --- |
| `src/model.rs`、`commands.rs` | Room 记录、绑定版本、预览计划、命令准入与外部写意图；复用 `store` 的事务、材料库与幂等机制 |
| `src/brief.rs` | 精确来源、机械摘录、人工确认提要；不按正文推断、不生成文字 |
| `src/actions.rs` | 结构化 human 动作的共同归一化入口；按精确身份、绑定允许清单、目标与版本生成同一个键和摘要 |
| `../../apps/control/src/chat/` | ruma 协议类型 + reqwest 出站；axum AppService 接收；来源回读、草稿观测、投递与恢复 |
| `../../apps/cli/src/room.rs` | `hctl2 room`，走已有 Query / Preview / Submit，不直接写存储 |

辛创建 Project 时调用 `chat::main_room(project, server, name, command_key)`，把返回的 Room 记录与外部意图放进创建 Project 的同一事务，再由己的投递器建原生房间。函数不建 Project、不联网；同 Project 的主 Room ID 固定，数据库唯一索引再拒绝第二间。另一个 Project 即使指同 Repo，也得到独立主 Room。`Server` 的绑定与端点来自 AppService 配置，不含令牌。

Topic 的 `participants` 是本 Project 已有 `room_selection` 记录的精确引用；空名册也需确认。选入记录的创建与候选校验分别归辛、P2.3，本包不把主 Room 名册自动拷给 Topic。Request 来源读取已准入 `request` 记录的 `question` 与 `blockers` 字段，以及每个阻塞对象的冻结版本；这个只读形状是辛的接线接口，不是另一套 Request 生命周期。

本批为双入口提供 `normalize_human_action`，尚没有 Workbench 按钮或 Matrix 客户端插件。AppService 收到的原始事件只作观测，不直接准入命令；结构化动作须经该函数校验，再交确认与命令入口。默认没有身份映射与动作允许项，普通消息、反应、服务和 bridge bot 都不会因此获得 human 权限。

## 存储、外部写入与恢复

- `room` 是身份与状态，`room_binding` 是版本化绑定。`room_command` 保存幂等结果，`room_effect_receipt` 保存原生回读确认；均在甲的存储内。
- 提要与被治理引用的源字节先存入甲的 Git 材料库，再在事务内准入；`chat_source_reference` 固定消息或对象来源及材料摘要。`room show` 可直接读确认提要，`room reference` 可离线校验冻结原文。它们不替代聊天服务器的当前消息。
- 建房使用命令关联的 Matrix alias 和初始关联状态；发送使用 Matrix 事务 ID。网络预检失败不消费尚未投递的建房意图。投递前持久化结果未知，丢确认后回读原关联，不改键、不盲建第二间。
- 依赖当前正文的操作实时核对未加密；离线、无权限与加密分别报类型化错误。关闭 Topic 不依赖当前聊天正文，不解决 Request 或取消 Task。换绑只改变绑定版本，旧材料引用仍可校验。
- 时间线使用服务器顺序与原生游标；草稿和阅读位置按客户端保存在 Matrix account data，未读计数来自 sync，不进入治理记录或绑定摘要。一间房加密不影响其他房间按各自绑定同步。
- AppService 令牌保存在生命周期管理的私有注册文件；控制面先写注册再启服务，已有 Tuwunel 进程只重启该组件以加载新注册，不停止其他组件。服务备份包含原生数据与注册配置；恢复后监听器重载回调与令牌。收件去重、消息观测和健康信息在可删除的 `cache/chat-inbox.sqlite`，收到事务持久化后才回 200。

结果未知仍可能需要人核查：如果进程在标记投递后、实际创建前退出，服务器又没有关联房间，恢复不会猜测「一定没创建」。这是没有原生 createRoom 幂等键的边界，不报告已成功。

## CLI 示例与提要

示例（Project 由辛创建，本包不添加临时 `project create` 替身）：

```bash
hctl2 room list --project-id A
hctl2 room show A ROOM_ID
hctl2 room timeline A ROOM_ID
hctl2 room sync A ROOM_ID --cursor MATRIX_CURSOR
hctl2 room draft --input draft.json
hctl2 room create-topic --key topic-1 --input topic.json
hctl2 room create-topic --key topic-1 --input topic.json --preview-token PREVIEW_TOKEN
```

`draft.json` 声明 `project_id`、`project_version`、`origin` 与机械 `selection`。聊天来源支持明确事件 ID、服务器顺序的起止消息、回复关系和协议 mentions；不扫描正文找关键词。Request 路径不要求 Message。响应只含逐字片段、精确来源、未读来源、规则引用和摘要，`automatic_summary` 为 `not_configured`；每次可审计的选材结论写入 `brief_observation`。未读来源的空摘要表示未知，不伪造源内容。

`topic.json` 的字段对应 `Action::CreateTopic`（CLI 加 `kind`）：`project_id`、`project_version`、`name`、`origin`、`brief`、`participants`、`roster_confirmed`。`brief` 用五个字段分别表达缘起与目标、已定事实与理由、分歧与待答、约束与材料、精确来源。人可编辑四类正文，系统只核来源范围与版本，不拿逐字摘录规则拒绝人的补写。预览不创建 Room；确认后冻结编辑版，不自动接入主 Room 后续消息。

其他写入口是 `close`、`rebind`、`send`、`freeze`、`resume`，使用同一 `--key / --input / --preview-token` 形式。`save-view-state` 只写派生客户端状态，无治理命令记录。失败输出 stdout JSON 的 `error.code` 与 `recovery_action`，退出码非零。

## Buck 目标与 CT-PROJECT 对照

| CT-PROJECT 条目（按现行描述定位） | 本批失败输入与验证 | 后续范围 |
| --- | --- | --- |
| 创建唯一主 Room，同 Repo 两 Project | 两个 Project 经事务辅助函数各建一间，重复命令不多建；`domain_test`、`room-cli-test` | 真正 `project create` 归辛 |
| 普通 Topic 创建/关闭，关闭不解决其他对象 | 不填完成条件仍能建关；Request 升级后关闭，其记录版本不变；关闭后发送拒绝 | 闲置提醒与 Project 归档/恢复业务命令归辛 |
| CJK、引用、派生草稿、服务器顺序、事务幂等 | 原生 CJK 发送重投一条；倒挂时间戳与不排序 ID 的选择测试；跨房事件拒绝、加密房与其他房同步隔离；重启恢复草稿 | Workbench IME、Execution Chat 与 Share to Room 归后续界面与调用包 |
| 提要正文/来源、人工编辑与后续主 Room 消息 | 缺正文、外 Project、旧版本、未确认名册拒绝；人工去敏和未决字段冻结；创建后主 Room 新消息不流入 Topic | 文义判断不声称已由机械摘录完成 |
| 系统起草、未配置模型 | 草稿不建 Room、不调用 Participant；生成或改字的假草稿校验失败；显式 `not_configured` | small-brain 分节/改写只留 `BriefDrafter` 接口 |
| 逐字与来源校验、未读、起草观测 | 改一个字、引外 Project 拒绝；原生不存在事件列入 unread；观测含规则摘要/来源/结论；人工补写不走摘录校验 | 配置模型后的指针分配与生成输出验证随引擎实现 |
| Request 无 Message 的正例 | 同 Project 的 Request 问题原文与冻结阻塞版本可摘取、创建；换阻塞版本/跨 Project 拒绝 | Request 创建/解决 API 归辛 |
| Context 可解释、Room 历史恢复 | 删除全部本地聊天缓存后，原生回读和冻结材料分别重建；Store 备份恢复到新目录校验字节 | Manifest、Bundle、首轮交付归子 |
| 离线/加密、换绑后旧引用 | 原生加密与停服拒绝依赖当前读；换绑不改 Room ID，冻结源仍可读 | 安全策略的完整配置界面非本包 |
| 普通事件不能命令、双路径同摘要、bot 拒绝 | 共同归一化函数拒绝普通消息、bridge、缺映射/目标/版本及目标不符；相同事件得到相同输入和键；收件箱不写治理命令 | 客户端结构化按钮接线随实际客户端，不把原始推送当命令 |
| Room 名册、Invocation、Request、Memo、发布评审、Context 其余行 | 本包不实现、不标通过 | 辛、P2.3、P2.4 及 Context 各包 |

运行入口：

```bash
cd src
./buck2 test root//crates/chat:domain_test root//apps/control:unit_test
./buck2 test root//apps/control:chat_native_test root//packaging/release:room-cli-test --config hctl2.xz_preset=fast
./buck2 build root//crates/chat:clippy root//apps/control:clippy root//apps/cli:clippy --config hctl2.xz_preset=fast
```

`domain_test` 不联网；`chat_native_test` 用锁定 Tuwunel 制品；`room-cli-test` 解包真实依赖包，走公共 CLI → daemon → 原生服务。完整包测试及其 `room-cli-clippy` 跟随 Release 工作流，普通 Code 检查不因此制作完整包。它们不操作开发者现有房间或默认控制面根。

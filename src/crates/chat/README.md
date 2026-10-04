# Chat：聊天端口与 Room

P2.2 己的实现说明；依据 [Project 的 Room 与消息](../../../docs/design/spec/project.md#room-与消息)（v0.19.0）与 [P2.2 开工书](../../../.memo/design/p2-control-20260906/05-p22-kickoff.md#己--聊天端口与-roomgrok)。开工书仍停在 v0.18.12 的部分以现行约束为准。Room 树在本包实现，不另开包。本包不改设计约束，也不实现 Project 创建、Request 生命周期、Invocation 或模型引擎。

## 模块与接线

| 文件 | 当前职责 |
| --- | --- |
| `src/model.rs`、`commands.rs` | Room 记录、绑定版本、预览计划、命令准入与外部写意图；复用 `store` 的事务、材料库与幂等机制 |
| `src/brief.rs` | 精确来源、机械摘录、人工确认提要；不按正文推断、不生成文字 |
| `src/actions.rs` | 结构化 human 动作的共同归一化入口；按精确身份、绑定允许清单、目标与版本生成同一个键和摘要 |
| `../../apps/control/src/chat/` | ruma 协议类型 + reqwest 出站；axum AppService 接收；来源回读、草稿观测、投递与恢复 |
| `../../apps/control/src/chat/tree.rs` | Matrix 承载 Space、原生挂靠回读与 content 改挂；不保存权威层级 |
| `../../apps/cli/src/room.rs` | `hctl2 room`，走已有 Query / Preview / Submit，不直接写存储 |

辛创建 Project 时调用 `chat::main_room(project, server, name, command_key)`，把返回的 Room 记录与外部意图放进创建 Project 的同一事务，再由己的投递器建原生房间。函数不建 Project、不联网；同 Project 的主 Room ID 固定，数据库唯一索引再拒绝第二间。另一个 Project 即使指同 Repo，也得到独立主 Room。`Server` 的绑定与端点来自 AppService 配置，不含令牌。

Topic 的 `participants` 是本 Project 已有 `room_selection` 记录的精确引用，记录的 `room_id` 指向 `chat::topic_id(project, command_key)`，不能拿另一间 Room 的选入记录代替；空名册也需确认。选入记录的创建与候选校验分别归辛、P2.3，本包不把主 Room 名册自动拷给 Topic。Request 来源读取已准入 `request` 记录的 `question` 与 `blockers` 字段，以及每个阻塞对象的冻结版本；这个只读形状是辛的接线接口，不是另一套 Request 生命周期。

本批为双入口提供 `normalize_human_action`，尚没有 Workbench 按钮或 Matrix 客户端插件。AppService 收到的原始事件只作观测，不直接准入命令；结构化动作须经该函数校验，再交确认与命令入口。默认没有身份映射与动作允许项，普通消息、反应、服务和 bridge bot 都不会因此获得 human 权限。

## 存储、外部写入与恢复

- `room` 是身份与状态，`room_binding` 是版本化绑定。`room_command` 保存幂等结果，`room_effect_receipt` 保存原生回读确认；均在甲的存储内。
- 提要与被治理引用的源字节先存入甲的 Git 材料库，再在事务内准入；`chat_source_reference` 固定消息或对象来源及材料摘要。`room show` 可直接读确认提要，`room reference` 可离线校验冻结原文。它们不替代聊天服务器的当前消息。
- 建房与承载 Space 使用命令关联的 Matrix alias 和初始关联状态。未知结果先查原 alias，再查创建者的原生 joined-room 清单及精确标记；两者均无匹配才以原 alias 重投，别名冲突后回读原目标。任何清单读取失败不当作不存在。发送始终重用原 Matrix 事务 ID，再按返回的事件 ID 回读，不扫描整段历史。
- 依赖当前正文的操作实时核对未加密；离线、无权限与加密分别报类型化错误。关闭 Topic 不依赖当前聊天正文，不解决 Request 或取消 Task。换绑只改变绑定版本，旧材料引用仍可校验。
- 时间线使用服务器顺序与原生游标；草稿和阅读位置按客户端保存在 Matrix account data，未读计数来自 sync，不进入治理记录或绑定摘要。一间房加密不影响其他房间按各自绑定同步。
- AppService 令牌保存在生命周期管理的私有注册文件；控制面先写注册再启服务，已有 Tuwunel 进程只重启该组件以加载新注册，不停止其他组件。服务备份包含原生数据与注册配置；恢复后监听器重载回调与令牌。收件去重、消息观测和健康信息在可删除的 `cache/chat-inbox.sqlite`，收到事务持久化后才回 200。

Topic 可以从本 Project 任一 Room 开出，缺省挂在来源 Room 下；Request 路径挂在主 Room 下，并可附本 Project 的相关消息。出处与确认提要在控制面固定，挂靠在 Matrix 固定。非叶 Room 多一个原生承载 Space，Space ID 不进 Room–Server Binding；自身消息房间不投影成自身下级。Space 创建意图先准入，再联网写入。创建完成的原生标记防止重试把后来改过的挂靠复原。

补丁 1a（v0.19.2 两句约束）：

- 房间建成后，control 把经确认的提要正文与来源渲染成开场消息发进新房间（`brief.rs::opening_body` 是唯一渲染器，测试与实现共用）。开场消息是同事务准入的独立外部意图（`chat.opening`，稳定关联键，重试不发第二条）；材料仍是权威，`room show` 的 `brief` 不变。
- 创建预览列出邀请名单：缺省是来源 Room（Request 路径是主 Room）当前的人类聊天成员——不属于控制面账号、也不在 AppService 命名空间（`@hctl2_` 前缀，从注册文件派生）里的成员；`invite` 态（已邀请未加入）也计为当前成员，是刻意取的超集。输入 `invites` 可删减、补充；最终名单在计划里冻结，逐人一条 `chat.members` 意图（每人独立冲突域），逐人投递并回读，部分失败报逐目标结果与 `ROOMS_PARTIAL`，不报全体成功。
- 承载 Space 的聊天成员跟主 Room 走：新建或补投承载 Space 时，把主 Room 当前的人类成员邀请进 Space（先读后写、幂等收敛）；`project members` 对主 Room 的邀请与移除，在同一意图的投递里同步到本 Project 全部承载 Space（逐 Space 回读，失败按未知回读重试）。Space 成员是 content，不进名册、不带来授权，加入规则仍是 invite。关闭 Topic 会一并撤回未发送的开场与逐人邀请意图。

`room hierarchy` 按本 Project 全部 Room 的即时 state 读取投影，不递归调用有 10 层响应限制的 hierarchy 接口，不用本地权威副本。多个上级全部保留，canonical 只作提示；环边不显示、标需要关注；外 Project 或未知上级列为外部链接；读不到的字段为空，不阻拦关闭等无关命令。`room reparent` 是 content Submit，不更改出处、绑定或消息；原生双向 state 写入逐项回读，部分失败明确返回 `partial`。挂靠不复制名册、授权、成员或加入规则，关闭不级联。Project 全体归档与多房间成员/权限业务动作仍由辛接线，不在本包冒充完成。

讨论串使用原生 `m.thread`；发送拒绝把讨论串消息作为另一个讨论串的根，消息仍按所在 Room 的事件 ID 冻结。Room 列表 API 是平铺查询，不以树代替全量入口；Workbench 的列表和原生视图尚未实现。

## CLI 示例与提要

示例（Project 由辛创建，本包不添加临时 `project create` 替身）：

```bash
hctl2 room list --project-id A
hctl2 room show A ROOM_ID
hctl2 room hierarchy A ROOM_ID
hctl2 room timeline A ROOM_ID
hctl2 room sync A ROOM_ID --cursor MATRIX_CURSOR
hctl2 room draft --key draft-1 --input draft.json
hctl2 room reparent --input reparent.json
hctl2 room create-topic --key topic-1 --input topic.json
hctl2 room create-topic --key topic-1 --input topic.json --preview-token PREVIEW_TOKEN
```

`draft.json` 声明 `project_id`、`project_version`、`origin` 与机械 `selection`。来源为 `kind: room`（兼容旧输入 `main_room`）或 `request`。聊天选材支持事件 ID、起止范围、含根消息的回复关系、原生讨论串和协议 mentions；范围从终点的原生 context 游标反向读，回复/带 `after` 的 mentions 从锚点向后读，不从最早消息找近期窗口，也不扫描正文找关键词。Request 路径不要求 Message，可用 `messages` 补充精确相关消息。响应只含逐字片段、精确来源、未读来源、规则引用和摘要，`automatic_summary` 为 `not_configured`。起草走非危险 Submit，以调用者身份记录结论；相同 `--key` 重试读同一冻结结果，不写成 Query。正文存 Git 材料，`brief_observation` 保存引用及摘要。未读来源的空摘要表示未知，不伪造源内容。

`reparent.json` 带 Project、Room、目标上级 Room、双方 `binding_version` 和明确要移除的 `old_space_ids`；旧 Space 必须能回读为本 Project 的 Room。空旧集合只新增挂靠，不声称原生操作具有跨房间原子性。目标仍是叶子时，先保存创建 Space 的原意图再执行；原生指针已写、意图尚未确认时，先回读原意图。改挂按原生边检查，目标是自己的下级则拒绝，不用已过滤环边的展示结果作判据。`send` 可带 `thread_root`，不带时是主时间线消息。

`topic.json` 的字段对应 `Action::CreateTopic`（CLI 加 `kind`）：`project_id`、`project_version`、`name`、`origin`、`brief`、`participants`、`roster_confirmed`。`brief` 用五个字段分别表达缘起与目标、已定事实与理由、分歧与待答、约束与材料、精确来源。人可编辑四类正文，系统只核来源范围与版本，不拿逐字摘录规则拒绝人的补写。预览不创建 Room；确认后冻结编辑版，不自动接入主 Room 后续消息。

其他写入口是 `close`、`rebind`、`send`、`freeze`、`resume`，使用同一 `--key / --input / --preview-token` 形式。关闭尚未建成的 Topic 在同一事务撤回未发送的建房意图；已发送但结果未知的保留，关闭后只回读，不补建房或挂靠。`save-view-state` 只写派生客户端状态，无治理命令记录。失败输出 stdout JSON 的 `error.code` 与 `recovery_action`，退出码非零。

## Buck 目标与 CT-PROJECT 对照

| CT-PROJECT 条目（v0.19.0，按现行描述开头逐行定位） | 本批失败输入与验证 | 未覆盖及原因 |
| --- | --- | --- |
| 创建 Project 同时建立它唯一的主 Room | 夹具调用事务辅助函数，同键一间，同 Repo 两 Project 两间；D、R | 实际 Project 命令及待确认 Repo 前置归辛 |
| 主 Room 与 Topic Room 均可按本次授权发起调用 | 未实现 | Invocation / Execution Spec 归 P2.3 |
| 普通 Topic Room 因未填完成条件而不能创建或关闭 | 无完成条件能建关；关后 Request 版本不变、不能发送；未发送建房撤回、未知建房只回读、过期关闭不撤回；D、N、R | Task/Run 联动与待处理入口归其业务包 |
| 两间活跃 Topic 同样闲置 15 天 | 未实现 | Request 闲置提醒归辛 |
| 有非终态 Run 等时归档 Project 拒绝 | 活跃 Project 是写入前置；D | Project 归档/恢复及跨模块阻塞清单归辛 |
| CJK 输入、结构化引用、草稿/游标/未读、并发流隔离 | CJK、原生顺序、事务重投、客户端 account data、冻结引用与恢复；D、N、R | IME、Execution Chat / Share to Room、真实双客户端并发与未读恢复全链尚未验 |
| Topic Room 的前情提要缺正文或精确来源 | 缺正文/来源拒绝，人的修改冻结，后续消息不流入；D、R | 「把未决写成已定」文义校验与「复制整段来源 Room 历史」结构检测不覆盖，留后续提要选材/生成包；当前人工确认不冒充这两项通过 |
| 前情提要草稿由 Room 内模型 Participant 书写 | 只走系统机械选材、不派工、不生成、不分节、显式未配置；D、R | 模型配置后的能力限制留引擎包 |
| 系统草稿的原文片段不逐字一致 | 改字/越范围拒绝、缺来源列 unread、人工补写不按逐字规则拒绝、Submit 幂等；D、R | 模型改写句与组装器赋指针留引擎包 |
| Run 的 Request 在主 Room 没有相关 Message | 精确 Request/冻结阻塞版本无消息能建，旧版/外 Project 拒绝；可附本 Project 消息、外 Project 消息拒绝；D | Request 创建/解决命令与其运行草稿接线归辛；适配器已接只读形状 |
| Topic Room 首次调用只给原聊天链接 | 可读确认提要与未配置状态；R | 首次调用 Bundle 交付归子 |
| Room 的 Project 归属或消息所属 Room 被引用动作改写 | 跨 Project 引用拒绝、Topic 关闭不解决 Request；D、R | 同根因 Request 去重归辛 |
| Topic Room 可从主 Room 或另一间 Topic Room 的 Message 创建 | 嵌套 Topic、缺省父节点、Space 唯一、清除旧双向边、别名缺失回读、未知建房恢复、承载指针确认前恢复、改挂不改出处/Binding、重复创建不复原改挂、缓存可丢弃；N、R | Request 缺省父节点由领域计划验证；真实 Matrix 客户端 UI 未验 |
| 挂靠按回读投影 | 双非 canonical/双 canonical、环、外 Project、鉴权失败字段空且标关注；HCTL 成环改挂拒绝，已有环不因展示过滤漏判；N、U、R | 仅声明本地 Matrix 能力，不支持其他聊天协议；跨 server 层级未验 |
| 挂靠不带来继承 | 独立名册、禁止借其他 Room 的选入记录、关闭不级联；D、R；原生私有房间权限未复制 | 多房间成员/权限调整 API、Project 全体归档归辛；成员拒绝路径尚未实测 |
| 一间 Room 只有一条时间线和一层讨论串 | 原生 m.thread、拒嵌套、事件 ID 与正文冻结、主时间线含串消息；N、R | Workbench 讨论串 UI 与所有成员同时读取未验 |
| Topic 建成后开场消息与确认名单逐人投递（v0.19.2 行） | 开场消息与 `opening_body` 逐字一致且仅一条；unknown 态重发走同一事务 ID（N）；缺省名单=来源/主 Room 人类成员（控制面与 `@hctl2_` 排除，B1 走主 Room 来源；非主来源与 Request 来源的缺省计算走同一条 `default_topic_invites` 路径，未单独做原生用例）；删减者不邀、名单外不邀（域测试断言意图，N 断言真房间成员回读）；逐人回读；`brief` 在开场后不变（N） | `ROOMS_PARTIAL` 输出与 `invite_defaults` 漂移判 stale 只有代码路径、未注入故障用例；「邀请不进名册/不带授权」无专门负例（全 diff 不写 `room_selection` 与权限记录——代码有、未测） |
| 承载 Space 成员跟主 Room 走（v0.19.2 行） | 新建 Space 邀请主 Room 人类成员；主 Room 邀请与移除均同步到全部承载 Space（逐 Space 回读，各有原生用例；同步用例在全部 Space 已存在后加入新人，区分同步与创建期收敛）；进 Space 不自动进 Topic Room（N）；收敛与同步在 unknown 重试时恒可写（先读后写幂等，N 有重试用例） | 「Space 成员不进名册/不带授权」无专门负例（同上，代码有、未测） |
| 待你处理按现有事项去重 | 未实现 | 聚合投影与业务动作归辛及后续包 |
| 待处理来源分别注入 | 未实现 | 五类事项随业务包逐项接入 |
| Context 可解释、Room 历史可恢复 | 丢聊天缓存、重同步与冻结源重读、异目录恢复；N、R | Manifest / Bundle 解释与纪要索引归子 |
| chat server 不可用时依赖当前回读 fail closed | 原生停服拒读写、冻结材料离线可读；N、R | Invocation / Context 的当前回读归其业务包 |
| Room–Server Binding 只接受未启用端到端加密的房间 | 原生 guard 加密拒绝、独立明文房正常、换绑不换身份；D、N | 真实升级/换绑后继续使用的完整端到端链未验 |
| chat server 中普通消息、反应或自动化不能成为命令 | 缺映射、缺目标/版本及普通消息拒绝；收件仅观测；U | 原生客户端结构化按钮未接 |
| 同一个 Matrix 用户动作分别从两路径提交 | 共同归一化函数同摘要同键、bridge/service 拒绝；U | 两个真实客户端端到端链未接 |
| 模型 Participant 的 @/建议不能创建 Invocation | 收件不派工；U | 人批准调用与 fan-out 归 P2.3 |
| 无法证明身份的 Invocation 撤权并终止 | 未实现 | P2.3 |
| 无 Run 返工是人发起的新 Room Invocation | 未实现 | P2.3 / P2.4 |
| mention 解析无唯一授权候选时明确失败 | 未实现；机械 mentions 选材不是授权选人 | P2.3 |
| 原始消息、执行日志和模型总结不经发布不会成为 Memo | 收件不发布 Memo；U | 发布 Memo 命令归辛 |
| 治理引用指向滚动纪要而非精确消息事件时拒绝 | Source 类型只含精确事件/对象与版本摘要；D | Context 侧引用校验归子 |
| Bundle 压缩条目缺记录或压缩证据 | 未实现 | 子 / 引擎包 |
| 萃取索引与纪要缓存删除后可完整重建 | 聊天缓存删除重同步；R；机械选材不按正文路由 | 纪要索引与相关性门归引擎/Context 包 |
| 过期或被取代的 Memo 不进指针清单 | 未实现 | 辛 / 子 |
| 压缩片段或纪要条目的回源指针不是组装器赋予 | 未实现 | 子 / 引擎包 |
| Matrix 房间升级换 ID 后换绑不改身份 | 领域换绑测试保留旧冻结源、Room ID；D | 原生 tombstone 升级完整链未验 |
| Room 名册换人只影响将来的调用 | 精确 room_selection 必须属于本 Room；D | 名册编辑、选人策略及活动 Invocation/Run 归辛/P2.3 |
| 评审发布策略随 Execution Spec 冻结 | 未实现 | P2.3 / P2.4 |
| 平台评审评论线冻结进 Context Manifest | 未实现 | P2.4 / 子 |
| 必需材料未送达或摘要不符时拒绝派发 | 未实现 | 子 |
| 授权本次材料集合不授予整库读权 | 未实现执行侧交付 | 子；本包冻结材料沿甲的 Project 权限 |
| 归属者终态与准入窗口均满足后才丢 Bundle | 未实现 | 子 |
| 同文字登记用途与未准入正文不自动生效 | 提要与源字节先保存、同事务准入；D、R | Artifact / Memo 发布及登记用途归辛 |

D = `domain_test`，U = control `unit_test`，N = `chat_native_test`，R = `room-cli-test`。表中的「部分验证」不等于整条 CT 已通过。

CT-WORKBENCH-IA 新增的平铺 Topic 列表行：`room list --project-id` 返回全部层级的 Room 记录，R 验证嵌套 Topic 未丢失；主 Room 也在通用 API 中并标类型，界面须另行筛出 Topic。左侧按钮、平铺与原生视图切换仍归 Workbench，不标本批 UI 验收通过。

运行入口：

```bash
cd src
./buck2 test root//crates/chat:domain_test root//apps/control:unit_test
./buck2 test root//apps/control:chat_native_test root//packaging/release:room-cli-test --config hctl2.zstd_preset=fast
./buck2 build root//crates/chat:clippy root//apps/control:clippy root//apps/cli:clippy --config hctl2.zstd_preset=fast
```

`domain_test` 不联网；`chat_native_test` 用锁定 Tuwunel 制品；`room-cli-test` 解包真实依赖包，走公共 CLI → daemon → 原生服务。完整包测试及其 `room-cli-clippy` 跟随 Release 工作流，普通 Code 检查不因此制作完整包。它们不操作开发者现有房间或默认控制面根。

# Chat Room 的 ground truth 该放在哪（讨论备忘）

> 日期：2026-08-19<br>
> 主题：Room 协作历史的权威归属；RepoInstance 的边界；OpenClaw/Hermes 场景的启示<br>
> 状态：讨论备忘，尚未修改合同。两个分叉待所有者拍板（见文末）。

## 问题

现行合同（草案 v0.9.1）把 Room 事件账本的权威放在 RepoInstance 的本地 SQLite 里，并承诺“另一个 clone 对同一 Project 有自己的 Project Room 投影和本地操作账本”。这与文档自己的两条承诺相撞：

- 愿景层：“Room 是所有 Harness 消失之后仍然可以恢复的协作现场”“换掉工具，项目仍可继续”；
- 设计原则 11：“运行时身份可替换——进程、worktree、终端都不是 Room 的身份”。

把协作记忆的权威放在一份 clone 的本地文件里，是把灵魂存在肉体里：一人两机 = 两个平行聊天室；clone 即坟墓。RepoInstance 概念本身没错（锁与运行现场必须有物理归属），错的是让它拥有了协作事实。

## Room 事实的四层拆解

| 层 | 内容 | ground truth 应在 | 现状 | 有争议 |
| --- | --- | --- | --- | --- |
| 1. Room 身份 | room_id、归属 Project | 所有权层（随 Project/用户） | 基本是，但被“每实例唯一 Project Room”搅浑 | 措辞 |
| 2. 协作历史 | 消息、调用记录、Request 往来 | **争议核心** | instance SQLite | **是** |
| 3. 提炼产物 | Memo、Artifact、Task 契约 | Git | Git ✓ | 否 |
| 4. 现场操作态 | 草稿、未读、租约、运行时 | instance | instance ✓ | 否 |

争议全部压缩在第 2 层。

## 第 2 层的四个候选

| 候选 | 优点 | 致命/主要代价 |
| --- | --- | --- |
| A. 维持现状（clone SQLite 权威） | 实现最简 | 产品语义错误：多机多室、clone 即坟墓；靠第一阶段单机范围掩盖 |
| B. 进 Git（HCTL1 老路） | 随仓库同步 | **隐私一票否决**：仓库可能公开，聊天史进 Git 等于向所有未来 clone 者广播；另有高频污染与离线合并难 |
| C. Matrix 作为权威 | 多设备与成员制是现成标准 | 平台成为权威违反端口纪律；引入 homeserver 依赖 |
| D. 单 writer 跨机延伸（hub 模式） | 零同步协议；合同已有种子（“失败的第二实例只能连接现有服务”） | 主机需在线可达 |

## OpenClaw / Hermes 场景的启示

OpenClaw（常驻个人 Agent 网关：多渠道确定性路由、配对白名单、按渠道降级）证明：对话连续性既不在 WhatsApp 也不在 Telegram，而在常驻网关自己的存储里；平台只是可替换的桥接表面。Hermes（Agent 操作的持久任务内核：SQLite 板、原子领取、心跳回收，CLI/聊天命令/Dashboard 共用一套命令内核）证明：黏住用户的是内核，agent 只是内核的一个操作者。

映射回 HCTL：

| 东西 | 归属 |
| --- | --- |
| Room 身份 + 协作历史账本 | 用户级常驻服务（hub）的账本；writer 可搬迁（台式机 → VPS），身份不动 |
| Matrix/Slack/Telegram/Feishu | 一律桥接表面（受控端口）；现有 Chat 端口绑定合同不需要改 |
| RepoInstance | 只剩代码侧物理事实：worktree、ChangeSet、运行时、单写锁；Room 引用仓库，不住在 clone 里 |

选项 C 由此归位：Matrix 是表面/传输候选，不是权威——平台会换，hub 的记忆不换。

## “通用对话 agent 是不是 Room 的好 anchor”

拆成两半：

- **作为模型/进程/Participant：不是。** 它是可替换运行时（原则 11）；把 Room 挂在它身上会复活来时路 §4 已杀掉的“常驻包工头”，并重开“Agent 记得 = 项目事实”的混淆。
- **作为用户级常驻 hub 的证据：是。** OpenClaw/Hermes 的黏性来自“永远在线、有记忆、路由确定的对话入口”。正确表述：**anchor 是所有权身份（用户，将来是团队），writer 是 hub 服务（可搬迁、可替换），通用对话 agent 只是每个 Room 里预接线的默认 Participant。** 这是 Repo/RepoInstance 那刀（身份 vs writer）在上一层的重演。

## 对 HCTL2 的连锁反应（若采纳）

1. “以 Git 仓库为边界”精确化：仓库仍是交付物与承诺的边界（Task 契约、ChangeSet、Artifact）；协作的边界是用户（将来是团队）。Room 账本移到用户级账本（`~/.hctl2/` 已有用户级存储先例）。
2. 撤销 spec/project.md“每 RepoInstance 唯一 Project Room”的承诺 → 一个 Project 一个 Room，账本在 hub，clone 只有投影。
3. 单写者纪律不变，多一个作用域：hub 账本一个 writer；每 RepoInstance 账本一个 writer（只管代码物理事实）。
4. 自然长出产品形态：用户级“总入口对话面”——跨 repo 先跟常驻 agent 说，话题成熟再提升进 Repo/Project Room（现有提升流程在上一层的镜像，不需要新概念名）。
5. 第一阶段退化无痛：单机上 hub 与 repo instance 同进程，行为不变；概念先改对。
6. 多人未来：Project Room 所有权升到团队 scope；hub 间联通（共享服务 vs Matrix 联邦）留到那时决策。

## 待拍板的分叉

1. 大方向：**Room 归用户级 hub、仓库只挂物理执行**——是否采纳？
2. 用户级“总入口对话面”是否进设计（它让通用对话 agent 成为产品正门，而不只是实现细节）？

拍板后动作：修订 spec/project.md、spec/system.md（存储与单写者的作用域）、design/README 对象关系图与相关承诺句，并在 decision-history 记录这次转向。

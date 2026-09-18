# 已定事项与接手清单

<a id="待拍板与接手清单"></a>

> 状态：Q1–Q3 与 C1 均已由所有者于 2026-09-19 确认，无需重拍；C2 只核 Mac 用例旧标签对应哪间 Room，尚待确认。<br>
> 日期：2026-09-19<br>
> 对照基线：草案 v0.18.6；本文记录体验裁决与后续工作，不代替新规范。

已明确的体验不再重复设成问题：入口叫 Project，当前关联一个 Repo；点击名称进入唯一的 Project Room，点击“待你处理”标记打开待处理面板；Rooms 只列 Topic Rooms、初始为空并吸收 Scoped Room 的用途；Kanbans 每个 Source 一个入口；Runs 独立列出。内容归属固定，引用可以交叉；Room 与 Run 分别选 Participant。组织结构的完整记录见 [04](./04-project-navigation.md#project-入口与-rooms--kanbans--runs)。

## Q1：旧设计的两层对象怎样对齐一个 Project 入口

**已定（所有者确认，2026-09-19）。** 正式设计跟随新组织方式，不再保留“同一 Repo 下把不同目标或话题各建成一个旧 Project”的业务层；也不只包装界面，把那一层藏到内部继续决定 Task 归属与 Topic 的生命周期。

Project 是这份工作的共同范围；Repo 保留代码来源、平台绑定、版本与集成等职责；Topic Room 承接讨论，Task、Run 各有自己的工作与结果。主 Room 只有一间，本目录统一称 Project Room，所有者所说的主 Repo Room 指同一间。

这是领域含义的方向，不是要求把 Project 与 Repo 合成一张表或删除 Repo 模块。旧 Project 的选人策略、预算默认值、具体授权和归档效果仍需逐项安放；字段、模块与存储组织由设计者提出，不再把每个内部拆分都交给所有者选择。现行 [Project 约束](../design/spec/project.md#对象)与 [Repo 约束](../design/spec/repo.md#对象)尚未完成对应改写。

**后续工作：** 按本页落点清单对齐规范与验证。Q1 不再阻塞方案；本轮没有修改 spec、产品代码、数据或设计版本。依据见 [04 的组织结论](./04-project-navigation.md#对旧模型的结论)与[确认原话](./04-project-navigation.md#2026-09-19归属与选人)。

## Q2：本地 detach 会改哪份目录

**已定（所有者逐项回复“同意”，2026-09-19）。** 有 remote 的本地 Repo 仍推荐接原 remote；用户选择用 Gitea 另起独立工作时，默认另建独立副本，保留原目录及其 remote。原地切换作为显式选择，操作前讲清修改范围与后果。

独立副本多一份存储，但不改动原来的工作环境；原地切换可能影响共用配置的工作树。建立副本不表示原目录也已 detach。

**后续工作：** 按此对齐[新建流程 P1](./02-user-journey.md#p1选择代码来源)和 [Repo 注册](../design/spec/repo.md#repo-注册)：另起独立工作时登记新的本地 Repo，不把原外部 Repo 的身份换绑到本地平台；原地切换目录也不等于沿用原 Repo 身份。仍需说明带入的代码范围，不默认迁移平台历史；本轮只更新体验文档。

## Q3：删除 Task 的后果

**已定（所有者逐项回复“同意”，2026-09-19）。** 两种 Room 都能通过聊天操作 Task。未提交草稿可以删除；已有工作的默认动作明确叫“取消并归档”，保留历史、不删 Source 卡片；真正“删除 Source 卡片”另行确认，并标清动作与后果。

有活动 Run 时明确处理其去向，不把删卡冒充执行已经停止。归属固定不等于禁止删除；取消工作、删除 Source 卡片和清除历史不是同一个动作。

这里的“归档”是用户动作效果，不是在体验文档里新增 Task 状态。后续按此对齐 [T2](./02-user-journey.md#t2在两种-room-里操作-task)、Task 生命周期与 Run 的相关处理；本轮没有改规范或实现。

## C1：cloud 用例的 Room 位置

**已确认（2026-09-19）。** 所有者原话：“对，这是主repo room”。`cloud_jstui_01` 与 `cloud_jssdk_01` 分别是 `gl-jstui`、`gh-jssdk` 对应 Project 的主 Room，也就是本目录所称的 Project Room，不是额外的 Topic Room。

[01](./01-multi-unit.md#修正后的-project-与-room-对照)已据此明确 Room 位置，不再标为整理假设。机器、人员和 Control 连接关系不变。

## C2：Mac 用例的 Room 位置

**待核对。** 旧用例的 `mac_jssdk_01`、`mac_jssdk_02` 都在 `mac_ctl` 的 `gh-jssdk` 工作范围内，分别有一组已明确的 Participant；原文混用 Project 一词，没有明确它们是否包括主 Room。此前整理成“两间 Topic Room，另有主 Room”是整理者的推断，不是 Q1 或 C1 的裁决。

需要核对的是旧标签：`mac_jssdk_01` 是主 Room、`mac_jssdk_02` 是 Topic Room，还是两者都是 Topic Room、另有主 Room；也可由所有者更正其他对应关系。确认前，[01](./01-multi-unit.md#修正后的-project-与-room-对照)只保留一个 Project、必有的唯一主 Room 和原文两组名册，不预定 Topic 数量或增造第三组人员。

这是 [Fable 与 Grok 的 #257 评审](https://github.com/yesme/hctl2/pull/257)指出的出处缺口，不影响已定的 Project 组织结构、独立选人和跨机连接关系。

## 规范对齐清单

以下是设计者接手后要完成的工作，不是新的待拍板问题，也不代表已经实施。

| 范围 | 现文位置 | 要对齐的内容 |
| --- | --- | --- |
| 愿景中的完整旅程 | [愿景 §目标体验](../design/vision.md#目标体验)、[一句话定位](../design/vision.md#一句话定位) | Project 在选择 Repo 时建立；把“话题成型后提升为具名 Project”改为展开 Topic Room，主 Room 不再分两间；Project-scoped 按新的工作范围解释，不必删除该短语 |
| 工作边界与命名 | [三面架构 §单元与连接](../design/architecture.md#单元与连接)、[设计地图](../design/README.md#对象关系)、[Project 约束](../design/spec/project.md#对象)、[Repo 约束](../design/spec/repo.md#repo-注册) | 落实 Q1：Project 作为工作范围，Repo 保留代码职责；不把 Topic 当旧 Project；独立控制面可使用同一外部 Repo；Q2 另起本地工作登记新 Repo，不换绑原外部身份 |
| 架构中的场景与容器 | [场景与系统](../design/architecture.md#场景与系统)、[5×3 归属矩阵](../design/architecture.md#53-归属矩阵) | 对齐表后“一个 Repo 一个 Repo Room、一个 Project 一个 Project Room”两级聊天容器，以及“一仓一张合并板、Project 是分组”的旧叙述；按 Source 进入看板不改变内容仍归各任务源的分责 |
| 内容归属与引用 | [Project 约束](../design/spec/project.md#repo-注册与-project-归档)、[Task 约束](../design/spec/task.md#对象)、[连接约束](../design/spec/connections.md#连接模型) | 既有内容不换 Project；消息不换 Room；Task 不换 Source；同 Project 的交叉引用不迁移归属、不自动改变承诺与授权 |
| Room 创建、分类与入口 | [Project 正文](../design/project.md#room-类型)、[Repo 注册与 Project 归档](../design/spec/project.md#repo-注册与-project-归档)、[Room 与消息](../design/spec/project.md#room-与消息) | 合并注册 Repo 与创建 Project 各建一间主 Room 的旧路径，对齐“无 Project 的 Repo Room”及“从 Repo Room 提升为 Project”的规则；只留唯一主 Room 与 Topic Rooms。新 Topic Room 用前情提要接续主 Room，参与者不必通读原聊天 |
| 主 Room 的调用范围 | [Repo 注册与 Project 归档](../design/spec/project.md#repo-注册与-project-归档)、[Room Invocation](../design/spec/project.md#room-invocation) | 对齐现行 `repo_scope` 只读、`project_scope` 可携带写入规则的区分，让主 Room 和 Topic Room 都能走 T3 的有边界调用；合并房间不自动授予写权限，仍按具体调用的批准范围做事 |
| Topic 的关闭与闲置提示 | [Room 与消息](../design/spec/project.md#room-与消息)、[交付 §运行默认值](../design/delivery.md#运行默认值) | Scoped Room 的用途并入 Topic Room，但冻结完成条件、回填与结案理由、闲置 14 天提示不能整体套给普通 Topic。实际请求仍有处理路径，不给所有 Topic 强加结案手续；具体请求如何关联、提示由什么触发由规范批对齐 |
| 待人处理入口 | [Project 正文](../design/project.md#room-场景)、[Run 正文](../design/run.md#workflow-场景) | 按 [04](./04-project-navigation.md#待你处理从-project-标记进入)对齐 Project 标记与待处理面板；汇总已有事项，处理历史留在原处；普通进度与确需人处理的请求分开 |
| Participant 选择 | [Participant 正文](../design/participant.md#agency-与执行体)、[派工连接](../design/spec/connections.md#project--run--participant从授权到派工) | 每个 Room、每个 Run 分别选人；推荐与预填不自动继承授权；TAMP 延长线不冒充当前已实现 |
| 多 Source | [Task 正文](../design/task.md#kanban-场景)、[Task 约束 §对象](../design/spec/task.md#对象)、[契约与来源](../design/spec/task.md#契约与来源) | 每 Source 一个入口；新 Task 目标 Source 明确；任务来源固定与源内状态、排序变化分开；对齐旧 Project 分组及唯一源引用。汇总投影是否提供另行设计，本轮既不要求保留强制合并板，也不禁止可选汇总 |
| Run 关联与导航 | [Run 约束](../design/spec/run.md#对象)、[Run 正文](../design/run.md#workflow-场景) | 独立活动列表、默认 DAG、任务书与共同 Worker 侧栏；Task 上可找回其已保存、尚未开工的计划，不另造活动 Run。执行、检查、评审、合入与 Task 验收分别可见，异常时保留原因与处理入口；引用不自动承担完整 Task 交付，不擅自扩大基数或并发规则 |
| 持续建议、模板、观察 | [交付文档](../design/delivery.md#当前范围)、[运行默认值](../design/delivery.md#运行默认值)、[Participant 正文](../design/participant.md#terminal-场景) | 对齐真实能力与验收；模板沿用批准前默认读回，显式跳过仍走既有规则；不把未配置归纳、只读 Source、没有图形能力冒充完整覆盖 |
| 术语与验证 | [术语表](../design/references/glossary.md#核心产品词)、[旧 S1 映射](../design/scenarios/S1-multi-unit.md#四不变量)、[契约测试](../design/contract-tests.md) | 术语首字母大写。CT-PROJECT 的主 Room 创建、来源提升、Scoped Room 闲置与回填，CT-REPO 的同 Repo 两个旧 Project 分别授权，CT-TASK 的合并板与分组，以及 CT-WORKBENCH-IA 的入口要随各自约束对齐；不能把改过用例名称当成已有新覆盖 |
| 决策史与版本 | [决策史](../design/references/decision-history.md#当前设计) | 正式设计收敛时记一章 Project 从仓库下的目标容器改为仓库级工作范围的转折，随约束改动更新全库版本与可失败的用例；本轮仍是体验整理，保持 v0.18.6 |

**用例外的核对项：** Fable 在 [#257 评审](https://github.com/yesme/hctl2/pull/257#issuecomment-5734517467)提出“同一 Control 的两个 Project 接入范围重叠的同一 Source”。现行 [Task 实体映射](../design/spec/task.md#契约与来源)以本 Control 为唯一范围；规范批需区分读到同一外部卡和认领为本 Project 的工作，核对这条规则与固定 Project 归属怎样共同成立。本轮没有裁定自动归属、复制 Task 或扩大唯一键，也不把这个反例冒充所有者原用例。

## 旧讨论怎样接手

09-17/18 各 Harness 的同题备忘已归档，文件头写明哪些判断被覆盖、哪些分析仍可参考。当前需求读本目录，原始用例与聊天记录仍供溯源；不再按旧备忘的拍板表逐项追问，也不将整篇建议算作所有者裁决。

持续建议的触发与费用控制、模板与读回怎样衔接、图形观察能力怎样交付，仍需主笔在规范与交付设计里安排。本轮已明确目标体验，没有拍定旧稿提出的特定机制或排期。[旧主笔稿 §六](../../.memo/design/user-path-20260917.md#六请所有者拍板)的模板跳过读回、[K3 稿 §五](../../.memo/design/user-path-alignment-20260918.md#五需要怎么改)的显式不挂平台第三选项，都只是历史候选，不进入本轮已定体验。

[F 批 PR #246](https://github.com/yesme/hctl2/pull/246)仍基于旧 Project 含义讨论 Task 标签；后续方案应先按本目录重核前提，不沿原文直接施工。本轮不修改或关闭那份 PR，也不把归档备忘当作正式规范已经失效。

## 接手时的完成边界

- 本目录已记录：四份体验正文、第一手出处、09-19 确认的组织结构，以及 Q1–Q3 和 C1 的结论，这四项均已定；C2 的旧标签核对仍待答复。随后按所有者要求补写前情提要与 Run 体验供查看；待处理标记、面板内容、处理后行为与降噪规则也已确认。
- 尚未完成：正式设计与约束收敛、契约测试改写、产品实现与真实行为验收。文档检查通过不等于这些能力已完成。
- 下一步：按已定结构与行为提出规范改法，连同验证一起对齐；不再等待重拍 Q1–Q3 或 C1。具体接口、默认 Source 的预填方式及历史列表折叠由设计者处理。
- 跨机接手以已合入 main 的修订为准；本地未提交修改不算 Git 交付，接手不依赖某台机器的临时目录或 Harness Session。

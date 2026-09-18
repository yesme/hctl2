# 已定事项与接手清单

<a id="待拍板与接手清单"></a>

> 状态：Q1–Q3、C1/C2 均已由所有者于 2026-09-19 确认；Q1 的此前归纳已按 C2 纠正，同 Control、同 Repo 的多个 Project 保留。<br>
> 日期：2026-09-19<br>
> 对照基线：从草案 v0.18.6 对齐到 v0.18.7；本文记录体验裁决、同步落点与未实施的工作，不代替模块约束。

已明确的体验不再重复设成问题：每个 Project 当前关联一个 Repo，同一 Control 可以为同一 Repo 建多个 Project；点击各自名称进入各自唯一的 Project Room，点击“待你处理”标记打开各自面板；Rooms 只列 Topic Rooms、初始为空并吸收 Scoped Room 的用途；Kanbans 每个 Source 一个入口；Runs 独立列出。内容归属固定，引用可以交叉；Room 与 Run 分别选 Participant。完整记录见 [04](./04-project-navigation.md#project-入口与-rooms--kanbans--runs)。

## Q1：旧设计的两层对象怎样对齐一个 Project 入口

**已定；此前归纳按 C2 纠正（2026-09-19）。** Project 是用户直接进入的一份工作，Repo 是它关联的代码来源，Topic Room 是这份工作内的讨论场所。每个 Project 关联一个 Repo，但同一 Control 可以为同一 Repo 建多个 Project；原本正确的这条关系保留。

每个 Project 有自己的唯一主 Room，本目录统一称 Project Room，所有者所说的主 Repo Room 指它。主 Room 与 Topic Room、Tasks、Runs 都明确属于哪份 Project 工作；同 Repo 不使其归属、名单或授权合并。Repo 保留代码来源、平台绑定、版本与集成等职责。

此前写成“不再保留同 Repo 下多个 Project 的业务层”，把“Topic 不等于 Project”扩大成了“Repo 与 Project 一对一”，是整理者的错误推断，现撤回。用户在当前 Project 内展开 Topic，不自动新建 Project；用户明确新建另一个 Project，也不被降为 Topic。Project 与 Repo 的记录、模块不要求合并，Project 原有默认设置、授权与归档效果逐项核对，不整套丢掉或移给 Topic。

**本 PR 已同步：** 按本页清单对齐规范与验证，保留 [Project](../design/spec/project.md#对象)、[Repo](../design/spec/repo.md#repo-注册)中已经一致的关系。本轮经所有者同意扩大为体验澄清与场景验收同步，相关 spec 与 CT 已改、版本推进 v0.18.7；未改产品代码或数据。依据见 [04 的组织结论](./04-project-navigation.md#对旧模型的结论)、[归属确认](./04-project-navigation.md#2026-09-19归属与选人)及 [C2 原话](./04-project-navigation.md#2026-09-19c2-确认)。

## Q2：本地 detach 会改哪份目录

**已定（所有者逐项回复“同意”，2026-09-19）。** 有 remote 的本地 Repo 仍推荐接原 remote；用户选择用 Gitea 另起独立工作时，默认另建独立副本，保留原目录及其 remote。原地切换作为显式选择，操作前讲清修改范围与后果。

独立副本多一份存储，但不改动原来的工作环境；原地切换可能影响共用配置的工作树。建立副本不表示原目录也已 detach。

**本 PR 已同步：** 按此对齐[新建流程 P1](./02-user-journey.md#p1选择代码来源)和 [Repo 注册](../design/spec/repo.md#repo-注册)：另起独立工作时登记新的本地 Repo，不把原外部 Repo 的身份换绑到本地平台；原地切换目录也不等于沿用原 Repo 身份。带入的代码范围沿原注册预览与授权，不默认迁移平台历史；实现仍须验证实际副作用。

## Q3：删除 Task 的后果

**已定（所有者逐项回复“同意”，2026-09-19）。** 两种 Room 都能通过聊天操作 Task。未提交草稿可以删除；已有工作的默认动作明确叫“取消并归档”，保留历史、不删 Source 卡片；真正“删除 Source 卡片”另行确认，并标清动作与后果。

有活动 Run 时明确处理其去向，不把删卡冒充执行已经停止。归属固定不等于禁止删除；取消工作、删除 Source 卡片和清除历史不是同一个动作。

这里的“归档”是用户动作效果，不是在体验文档里新增 Task 状态。本 PR 按此对齐 [T2](./02-user-journey.md#t2在两种-room-里操作-task)、Task 约束与失败用例，Run 原取消前置保留；未改实现。

## C1：cloud 用例的 Room 位置

**已确认（2026-09-19）。** 所有者原话：“对，这是主repo room”。`cloud_jstui_01` 与 `cloud_jssdk_01` 分别是 `gl-jstui`、`gh-jssdk` 对应 Project 的主 Room，也就是本目录所称的 Project Room，不是额外的 Topic Room。

[01](./01-multi-unit.md#修正后的-project-与-room-对照)已据此明确 Room 位置，不再标为整理假设。机器、人员和 Control 连接关系不变。

## C2：Mac 用例的 Room 位置

**已确认（2026-09-19）。** 所有者明确两者都是主 Room，同一 Control 可以为同一 Repo 开两个 Project，[原话见 04](./04-project-navigation.md#2026-09-19c2-确认)。`mac_jssdk_01`、`mac_jssdk_02` 分别是 `mac_ctl` 基于 `gh-jssdk` 创建的两个 Project 的主 Room，原有两组 Participant 名单分别保留。

前轮提出的“两间 Topic Room 加主 Room”和“一间主 Room 加一间 Topic Room”都不正确。[01](./01-multi-unit.md#修正后的-project-与-room-对照)恢复为 Mac 两个 Project、cloud 两个 Project，共四个 Project；没有凭空增加 Topic Room 或第三组 Mac 名册。

这项确认不只是补一个标签：它纠正了本目录对 Q1 和原用例的归纳。每 Project 的导航、固定归属、独立选人、跨机连接及共享 Git 对象库等确认仍保留，不因同 Repo 多 Project 而改变。

## 规范对齐清单

以下是本 PR 已同步的文档落点，不是新的待拍板问题。约束与验收要求已更新，产品实现及行为实测仍未因文档变化而完成；新路径的具体输入与反例见 [S3](../design/scenarios/S3-user-journey.md)。

| 范围 | 本 PR 同步位置 | 同步内容与保留边界 |
| --- | --- | --- |
| 愿景中的完整旅程 | [愿景 §目标体验](../design/vision.md#目标体验)、[一句话定位](../design/vision.md#一句话定位) | 新建 Project 时选择 Repo，再从本 Project 主 Room 展开 Topic；不把展开 Topic 写成必建 Project，也不禁止用户为同 Repo 再开 Project。Project-scoped 与固定 Project 归属保持一致 |
| 工作边界与命名 | [三面架构 §单元与连接](../design/architecture.md#单元与连接)、[设计地图](../design/README.md#对象关系)、[Project 约束](../design/spec/project.md#对象)、[Repo 约束](../design/spec/repo.md#repo-注册) | Project 作为工作范围，Repo 保留代码职责；保留同 Control、同 Repo 多 Project 及各自授权，只凭 Control 与 Repo 不能确定是哪份工作。Q2 另起本地工作才登记新 Repo，普通的同 Repo 新 Project 不需要另一份 Repo 身份 |
| 架构中的场景与容器 | [场景与系统](../design/architecture.md#场景与系统)、[5×3 归属矩阵](../design/architecture.md#53-归属矩阵) | 对齐表后“一个 Repo 一个 Repo Room、一个 Project 一个 Project Room”两级聊天容器，以及“一仓一张合并板、Project 是分组”的旧叙述；按 Source 进入看板不改变内容仍归各任务源的分责 |
| 内容归属与引用 | [Project 约束](../design/spec/project.md#repo-注册与-project-归档)、[Task 约束](../design/spec/task.md#对象)、[连接约束](../design/spec/connections.md#连接模型) | 既有内容不换 Project；消息不换 Room；Task 不换 Source；同 Project 的交叉引用不迁移归属、不自动改变承诺与授权 |
| Room 创建、分类与入口 | [Project 正文](../design/project.md#room-类型)、[Repo 注册与 Project 归档](../design/spec/project.md#repo-注册与-project-归档)、[Room 与消息](../design/spec/project.md#room-与消息) | 对齐登记 Repo 另建仓库级 Room、无 Project 的 Repo Room 及来源提升的旧路径；唯一性按每个 Project 的主 Room，不按 Repo 把多个 Project 的主 Room 合成一间。Topic Room 用本 Project 主 Room 的前情提要开场 |
| 主 Room 的调用范围 | [Repo 注册与 Project 归档](../design/spec/project.md#repo-注册与-project-归档)、[Room Invocation](../design/spec/project.md#room-invocation) | 替换旧 `repo_scope` 只读、`project_scope` 可携带写入规则的区分，让主 Room 和 Topic Room 都能走 T3 的有边界调用；合并房间不自动授予写权限，仍按具体调用的批准范围做事 |
| Topic 的关闭与闲置提示 | [Room 与消息](../design/spec/project.md#room-与消息)、[交付 §运行默认值](../design/delivery.md#运行默认值) | Scoped Room 的用途并入 Topic Room，但冻结完成条件、回填与结案理由、闲置 14 天提示不能整体套给普通 Topic。实际请求仍有处理路径，不给所有 Topic 强加结案手续；真正待办按已有 Request 或对象状态投影，处理仍走原命令 |
| 待人处理入口 | [Project 正文](../design/project.md#room-场景)、[Run 正文](../design/run.md#workflow-场景) | 按 [04](./04-project-navigation.md#待你处理从-project-标记进入)对齐 Project 标记与待处理面板；汇总已有事项，处理历史留在原处；普通进度与确需人处理的请求分开 |
| Participant 选择 | [Participant 正文](../design/participant.md#agency-与执行体)、[派工连接](../design/spec/connections.md#project--run--participant从授权到派工) | 每个 Room、每个 Run 分别选人；推荐与预填不自动继承授权；TAMP 延长线不冒充当前已实现 |
| 多 Source | [Task 正文](../design/task.md#kanban-场景)、[Task 约束 §对象](../design/spec/task.md#对象)、[契约与来源](../design/spec/task.md#契约与来源) | 每 Source 一个入口；新 Task 目标 Source 明确；任务来源固定与源内状态、排序变化分开；对齐旧 Project 分组及唯一源引用。汇总投影是否提供另行设计，本轮既不要求保留强制合并板，也不禁止可选汇总 |
| Run 关联与导航 | [Run 约束](../design/spec/run.md#对象)、[Run 正文](../design/run.md#workflow-场景) | 独立活动列表、默认 DAG、任务书与共同 Worker 侧栏；Task 上可找回其已保存、尚未开工的计划，不另造活动 Run。执行、检查、评审、合入与 Task 验收分别可见，异常时保留原因与处理入口；引用不自动承担完整 Task 交付，不擅自扩大基数或并发规则 |
| 持续建议、模板、观察 | [交付文档](../design/delivery.md#当前范围)、[运行默认值](../design/delivery.md#运行默认值)、[Participant 正文](../design/participant.md#terminal-场景) | 对齐真实能力与验收；模板沿用批准前默认读回，显式跳过仍走既有规则；不把未配置归纳、只读 Source、没有图形能力冒充完整覆盖 |
| 术语与验证 | [术语表](../design/references/glossary.md#核心产品词)、[S1 映射](../design/scenarios/S1-multi-unit.md#四不变量)、[契约测试](../design/contract-tests.md) | 保留 CT-REPO 的同 Repo 两个 Project 分别授权及 CT-PRODUCT 的四 Project 多机走查；纠正 S1 §二/§四/§五和矩阵导言把两个 Project 误判为两间 Topic Room 的说明。CT-PROJECT 的主 Room 创建、来源提升、闲置回填，CT-TASK 的看板与分组、CT-WORKBENCH-IA 的入口按实际变化对齐；S1.I3 的 Room 唯一范围改为每 Project |
| 决策史与版本 | [决策史](../design/references/decision-history.md#当前设计) | 正式收敛时记录 Project 入口、Room 分类与 Source 导航等实际变化，保留同 Repo 多 Project，不把取消它写成转折。随实际约束变化更新版本与可失败用例；本轮已扩为体验澄清与场景验收同步，更新到 v0.18.7 |

**共享 Source 的验收走查：** C2 的两个 Mac Project 若都选择同 Repo 的 GitHub Issues，会读到同一 Source 的卡片。所有者随后明确 [Project 是独立 Namespace](./04-project-navigation.md#2026-09-19project-独立-namespace)：[Task 实体映射](../design/spec/task.md#契约与来源)的唯一范围由 Control 收窄为各 Project 内；双方分别认领可各得自己的 Task，契约、Run、授权与验收独立，外部卡仍只有一张。S3.P5 配对验证，不把新建独立 Task 当成搬动既有 Task。Fable 在 [#257 评审](https://github.com/yesme/hctl2/pull/257#issuecomment-5734517467)举的两个 Repo 接同一个 Linear team 是用例外的另一例，不能把它冒充原文；同 Repo 两个 Project 则已有 C2 的明确依据。

## 旧讨论怎样接手

09-17/18 各 Harness 的同题备忘已归档，文件头逐项写明哪些判断被覆盖、哪些仍成立。C2 已纠正前轮的过度核销：同 Control、同 Repo 开多个 Project 不是过时结论，把 Mac 两个 Project 改成 Topic Room 才是误读。当前需求读本目录，原文仍供溯源；不按旧拍板表逐项追问，也不将整篇建议算作所有者裁决。

持续建议的触发与费用控制、图形观察能力怎样交付，仍需主笔安排实现设计；模板与读回沿用既有批准规则，本 PR 已同步引用及反例。本轮已明确目标体验，没有拍定旧稿提出的特定机制或排期。[旧主笔稿 §六](../../.memo/design/user-path-20260917.md#六请所有者拍板)的模板跳过读回、[K3 稿 §五](../../.memo/design/user-path-alignment-20260918.md#五需要怎么改)的显式不挂平台第三选项，都只是历史候选，不进入本轮已定体验。

[F 批 PR #246](https://github.com/yesme/hctl2/pull/246)讨论 Task 与 Project 的归属、可变分组。C2 保留同 Repo 多 Project，不能再以“这一层已删除”宣布 F 批无题可审；其改法仍要与所有者“内容不搬 Project”的要求逐条核对。本轮不修改或关闭那份 PR，也不把归档备忘当作正式规范已经失效。

## 接手时的完成边界

- 本目录已记录：四份体验正文、第一手出处、09-19 确认的组织结构，以及 Q1–Q3、C1/C2；Q1 的归纳按 C2 纠正，不再留下待答标签。前情提要、Run 体验、待处理标记、面板内容、处理后行为与降噪规则保留。
- 本 PR 同步：主 Room 与 Topic Room、Project 多 Source、固定归属、待处理投影、已保存计划等直接相关约束与设计引用；S1 保留正确事实并纠错，S2 补界面核对，S3 及现有 CT 族补失败用例，版本为 v0.18.7。
- 尚未完成：本 PR 复审与合入、产品实现和真实行为验收。下一步审核体验、约束、场景与 CT 是否一致，不再等待重拍 Q1–Q3 或 C1/C2；具体接口、默认 Source 的预填方式及历史列表折叠由设计者处理。文档检查通过不等于能力已交付。
- 跨机接手以已合入 main 的修订为准；本地未提交修改不算 Git 交付，接手不依赖某台机器的临时目录或 Harness Session。

# 基础用户体验

> 状态：用户体验基线；组织结构与 Q1–Q3、C1 已确认，Mac 两个旧标签的 Room 位置待 C2 核对；正式规范尚待对齐，不是实现完成报告。<br>
> 整理日期：2026-09-19<br>
> 对照基线：main @ `41594b714cd7df596df605f8fce2b55750319571`，现行设计草案 v0.18.6；本目录的组织结构要求尚待规范对齐。

这个目录回答“用户想怎样使用 HCTL2”，是讨论架构和对象关系的第一手需求基线。依据是所有者的原始用例、用户流程、术语混用说明，以及 09-18/19 确认的导航与组织结构；各 Harness 的评估不是需求来源。

## 四份正文

| 内容 | 文件 | 职责 |
| --- | --- | --- |
| 多 Control、多 Repo 的用例 | [01 · 多单元用例](./01-multi-unit.md#多-control多-repo-的用例) | 保留机器、Agency、人员选择和远程连接，纠正讨论分组的指代；沿用 S1 编号 |
| 用户体验流程 | [02 · 用户流程](./02-user-journey.md#用户体验流程) | 建 Project、接 Source、选人、前情提要带入 Topic Room、操作 Task、模板生成 DAG、观察 Run 及交付与异常处理 |
| 术语混用说明 | [03 · 术语纠正](./03-terminology-confession.md#project--repo--topic-混用的-confession) | 保留原话，说明新旧含义、纠正进度与记录退役条件；具体未对齐位置见接手清单 |
| 导航与组织结构 | [04 · 导航与关系](./04-project-navigation.md#project-入口与-rooms--kanbans--runs) | 记录 Apollo 侧栏、待处理入口与行为、固定归属与交叉引用、Room/Run 分别选人，以及 Q1 的结论 |

[已定事项与接手清单](./open-questions.md#已定事项与接手清单)记录 Q1–Q3、C1 的确认结论和设计者的后续工作，这四项无需重拍；[C2](./open-questions.md#c2mac-用例的-room-位置)只核 Mac 用例的旧标签，不重开组织结构。后续体验补充另按所有者确认更新，不让接手者靠恢复聊天记录继续。

## 已确认的组织方式

Project 是顶层工作范围，当前关联一个 Repo；点击 Project 名称进入唯一的 Project Room，点击旁边的“待你处理”标记打开[待处理面板](./04-project-navigation.md#待你处理从-project-标记进入)。原流程里的 Repo Room 就是这间主 Room，不另建第二间。其下 Rooms 只列 Topic Rooms，初始为空；Kanbans 每个 Source 一个入口；Runs 独立列出活动执行，点击默认看 DAG。

组织关系的完整定义只放在 [04](./04-project-navigation.md#松散耦合具体意味着什么)：内容归属固定，引用可以交叉。Room、Task、Run 不形成强制包含链，每个 Room 与每个 Run 分别选 Participant。未来 TAMP 推荐不等于自动继承权限。

旧设计中“Repo 下按目标再建多个 Project”的业务层不继续保留。Project 的新含义与 Repo 的代码职责可以并存，不意味着删除 Repo 模块或合并数据库表；这个方向已经确认，不再作为待拍板项，见 [Q1](./open-questions.md#q1旧设计的两层对象怎样对齐一个-project-入口)。

## 怎么使用这份基线

- 讨论结构时，先指出要满足哪个用例、哪一步，再提出改法。不能因为旧对象不支持，就把用户想要的体验改写掉。
- 本目录记录需求，不定义数据库对象、状态机或协议。旧设计与体验不一致时，按接手清单对齐，不把本次整理说成规范与代码已经完成改造。
- 现行实现的精确行为仍查[设计约束](../design/spec/README.md#文件)。它与新体验的差异不是体验无效，也不是实现已经支持。
- 原始材料保留溯源；当前体验只在本目录维护。旧 [S1](../design/scenarios/S1-multi-unit.md#参考用例-s1多单元协作)保留既有设计的验证映射，不再另养一份混用术语的用户叙事。

## 本目录的用词

正文使用中文，作为术语的英文名称首字母大写，复合名称各词首字母大写，例如 Project、Repo、Task、Topic Room；CLI、DAG、TAMP 等缩写保持通常写法。这是所有者 2026-09-19 的要求，取代此前正文统一小写的安排。

历史原话、文件名、命令和代码标识符保持原样；例如 `mac_ctl` 不改成另一种拼写。不能靠大小写区分新旧 Project 的含义，提到旧模型时明确写“v0.18.6 的旧 Project”。

| 用词 | 本目录里的含义 |
| --- | --- |
| Control | 持有这份工作及其决定的控制面实例，原文常简写为 ctl；换前端不等于换 Control |
| Project | 用户的顶层工作范围；当前代码协作场景关联一个 Repo，名字保留未来扩展余地 |
| Repo | Project 当前关联的 Git 代码来源及相应代码协作职责；不是 Topic，也不是本机某一份工作副本 |
| Project Room | 点击 Project 名称进入的唯一主 Room；原流程与最新示意中的 Repo Room 指同一间 |
| Topic / Topic Room | 一项讨论主题 / 围绕它建立的 Room；不是新 Project，吸收旧 Scoped Room 的讨论用途 |
| Room | Project Room 与 Topic Room 的共同叫法 |
| Source / Kanban | Task 的来源 / 该 Source 在 Project 内的可视化入口；Kanbans 每项对应一个 Source |
| Task / Run | 要完成的工作 / 一次有计划的执行；不是同一东西，也不是 Room 的子容器 |
| Participant / Worker | 选入协作的一位参与者 / 在 Run 中施工时的称呼；Room 与 Run 分别选入 |
| Agency | 提供候选 Participant、承接执行与观察的供给方，可被多个 Control 使用 |
| Workbench / CLI | 用户入口，可以远程连接 Control，不拥有另一份 Project 事实 |

## 第一手来源与整理边界

| 来源 | 在本目录中的处理 |
| --- | --- |
| 所有者原始[HCTL 案例](../../.memo/notes/HCTL_case_study.md#hctl案例)，2026-09-06；后续补充至 09-13 | 01 保留 S1.1–S1.11、S1.N1–S1.N8 与共享/独立克隆两种布局；cloud 两处主 Room 已确认，Mac 两个旧标签是否包括主 Room 待 C2 核对 |
| 所有者 2026-09-17 的流程；仓内录文见[用户路径 §一](../../.memo/design/user-path-20260917.md#一所有者的路径原文加编号) | 02 承接原步骤；单 Kanban 与旧 Project 指代按后续要求更新，不搬入同文件后面的作者方案 |
| 所有者 2026-09-18 的[术语混用原文](../../.memo/log/2026-09-18-project-repo-topic-混用.md#第一轮) | 03 摘录所有者原话，不把同一记录中的 Harness 答复当裁决 |
| 所有者 09-18 的导航提议、09-19 的 Apollo 示意及归属/选人说明 | 04 保留[原话](./04-project-navigation.md#本轮原话)，记录当前组织结构与固定归属；不追加部署或存储结构 |
| 所有者 09-19 确认按讨论整体修订本目录、要求术语首字母大写，随后同意 Q2/Q3 并确认 C1 是主 Repo Room | 本目录六份文件对齐；Q1–Q3 与 C1 均记为已定；正式规范另批修改 |
| 所有者 09-19 明确 Topic Room 用前情提要接续主 Room，同意补写 Run 体验供查看 | 02 补开场材料、交付结果、观察与异常路径；04 记录[原话](./04-project-navigation.md#2026-09-19前情提要与-run-体验补充) |
| 所有者 09-19 确认 Project 名称与“待你处理”标记分别进入聊天室和待处理面板，认可四个问题、处理后行为与降噪规则 | 04 记录完整[行为与入口](./04-project-navigation.md#待你处理从-project-标记进入)及[确认原话](./04-project-navigation.md#2026-09-19待你处理入口)，02 引用同一处理路径 |

## 从另一台机器接手

在自己的工作分支更新到 origin/main，从本页进入：先读 04 确认组织结构，再走 01、02，最后读已定事项与接手清单。03 只在需要追溯旧说法时读；跨机使用新修订前先确认它已合入 main。

当前已记录体验与组织方向，尚未完成旧 Project / Repo 结构改造、规范版本更新、产品实现或行为测试。下一步按 Q1–Q3 与 C1 的结论对齐设计正文、spec、术语表和契约测试。接手以仓内记录为依据，不直接按某份旧 Memo 的推荐开工，也不依赖某台机器的 Harness Session 或临时文件。

# 基础用户体验

> 状态：用户体验基线；已明确的要求、整理假设与待拍板项分别标明，不是实现完成报告。<br>
> 整理日期：2026-09-18<br>
> 对照基线：main @ `27be8cefbd80a20dcdfb1f8fc2a6449a24d9d850`，现行设计草案 v0.18.6。

这个目录集中回答“用户想怎样使用 HCTL2”，作为讨论架构和对象关系的 ground truth。依据是所有者的原始 use cases、用户流程、术语混用 confession，以及 2026-09-18 本轮提出的 project 导航。各 harness 的评估不是需求来源。

## 四份正文

| 本轮要求 | 文件 | 内容 |
| --- | --- | --- |
| 1 · 多 ctl、多 repo 的 use cases | [01 · 多单元用例](./01-multi-unit.md#多-ctl多-repo-的-use-cases) | 保留机器、agency、人员选择和远程连接，修正 project / repo / topic 的指代；沿用 S1 编号 |
| 2 · 用户体验流程 | [02 · 用户流程](./02-user-journey.md#用户体验流程) | 建 project、接 source、选人、聊天产生 topic/task、模板生成 DAG、观察 run |
| 3 · 混用 confession | [03 · 术语纠正](./03-terminology-confession.md#project--repo--topic-混用的-confession) | 原话、当前读法、已纠正和未纠正的位置、退役条件 |
| 4 · project 与三个列表 | [04 · 导航与关系](./04-project-navigation.md#project-入口与-rooms--kanbans--runs) | project 对应 repo；点击 project 进入 project room；三个并列列表及松散关联 |

[待拍板与接手清单](./open-questions.md#待拍板与接手清单)单列真正未定的边界、整理时不能唯一还原的原文，以及后续落点。接手者不需要找聊天记录才能继续。

## 怎么使用这份基线

- 讨论结构时，先指出要满足哪个用例、哪一步，再提出改法。不能因为旧对象不支持，就把用户想要的体验改写掉。
- 本目录记录需求，不定义数据库对象、状态机或协议。旧设计与体验不一致时，列为待对齐；本次没有把规范或代码一并改完。
- 现行实现的精确行为仍查[设计约束](../design/spec/README.md#文件)。它与新体验的差异不是“体验无效”，也不是“实现已经支持”。
- 原始材料保留溯源；当前体验只在本目录维护。旧 [S1](../design/scenarios/S1-multi-unit.md#参考用例-s1多单元协作)保留既有设计的验证映射，不再另养一份混用术语的用户叙事。

本轮新增要求明确更新了旧说法：`project` 不再指 topic；“一个 project 只能有一个 task-kanban”改为 `kanbans` 可列多个 source；`scoped room` 并入 `topic room`；project room 是点击 project 时的主入口，不是另一层容器。

## 本目录的用词

正文使用中文，同级产品概念统一用英文小写；复数只表示列表或多个实例。这里按所有者本轮要求采用这一写法，不在同一层交替使用中文译名。原话、旧规范对象名及文件名保留原样，并标清它们属于历史材料或设计对照。

| 用词 | 本目录里的含义 |
| --- | --- |
| ctl | 持有这份工作及其决定的 control 实例；换前端不等于换 ctl |
| project | 用户的顶层工作入口；当前代码协作场景对应一个 repo，名字保留未来扩展余地 |
| repo | project 当前使用的 Git 代码来源；不是 topic，也不是本机某一份 checkout |
| project room | project 的主 room；直接点击 project 进入 |
| topic / topic room | 一项讨论主题 / 围绕它建立的 room；不是新 project；吸收旧 scoped room 的用户用途 |
| room | project room 与 topic room 的共同叫法 |
| source / kanban | task 的来源 / 该 source 在 project 内的可视化入口；`kanbans` 每项对应一个 source |
| task / run | 要完成的工作 / 一次有计划的执行；不是同一东西，也不是 room 的子容器 |
| participant / worker | 选入协作的一位参与者 / 在 run 中施工时的称呼 |
| agency | 提供可选 participant、承接执行与观察的供给方；可被多个 ctl 使用 |
| workbench / cli | 用户入口；可以远程连接 ctl，不拥有另一份 project 事实 |

这里的 `project` 不能机械等同于 v0.18.6 的 `Project` 对象；`repo` 也不表示已决定删除现有 `Repo` 模块。后端两层对象怎么对齐见[待拍板 Q1](./open-questions.md#q1旧设计的两层对象怎样对齐一个-project-入口)。

## 第一手来源与整理边界

| 来源 | 在本目录中的处理 |
| --- | --- |
| 所有者原始[HCTL 案例](../../.memo/notes/HCTL_case_study.md#hctl案例)，2026-09-06；后续补充至 09-13 | 01 保留 S1.1–S1.11、S1.N1–S1.N8 与共享/独立 clone 两种布局；只按本轮语义纠正分组名称 |
| 所有者 2026-09-17 的流程；仓内录文见[用户路径 §一](../../.memo/design/user-path-20260917.md#一所有者的路径原文加编号) | 02 承接原步骤；单 kanban 与旧 project 指代按本轮更新，不搬入同文件后面的作者方案 |
| 所有者 2026-09-18 的[confession 原文](../../.memo/log/2026-09-18-project-repo-topic-混用.md#第一轮) | 03 摘录所有者原话，不把 Claude 的答复当裁决 |
| 所有者 2026-09-18 本轮导航提议 | 04 收录原话与体验整理；不另加第四个顶层列表，不预定新的部署或存储结构 |
| 所有者本轮交付要求 | 1–4 全部落盘；待拍板项留在同目录；PR 自验、自合到 main，让另一台机器通过 Git 接手 |

## 从另一台机器接手

在自己的干净工作树更新到 `origin/main`，从本页进入：先读 04 确认当前口径，再走 01、02，最后处理待拍板清单。03 只在需要追溯旧说法时读。

本次交付完成的是体验归拢、术语纠正、来源对照及旧入口指路；**没有完成**旧 `Project` / `Repo` 结构改造、规范版本更新、产品实现或行为测试。后续应先处理 Q1，再成批对齐设计正文、spec、术语表和契约测试。不要直接按某份旧 memo 的推荐开工。

所有接手材料都在 Git 中；不依赖这台 Ubuntu 的临时文件、harness session 或本地未提交修改。

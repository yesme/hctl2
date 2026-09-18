# Project / Repo / Topic 混用的 confession

> 状态：保留中；本目录已纠正体验叙述，旧设计和 CT 尚未全量对齐，不满足删除条件。<br>
> 日期：2026-09-19<br>
> 原始记录：[所有者与 Claude 的对话 §第一轮](../../.memo/log/2026-09-18-project-repo-topic-混用.md#第一轮)。原记录保留，不改写历史。

这份记录解释旧材料为什么会误导后续结构讨论，不是额外的产品概念。修正的是词的指代，不是让用户放弃原来的多机、多来源、并行协作需求。当前组织方式以后续确认的 [Project 入口与三列表](./04-project-navigation.md#project-入口与-rooms--kanbans--runs)为准。

## 所有者原话

以下只摘录所有者的 confession，保留原文写法；同一 log 中 Harness 的后续方案不作为所有者裁决。

> 我意识到，好像是我自己混用了术语，导致后边有很多的混乱（主要责任在我，但你也有次要责任 - 你要记得规劝我啊）：
>
> 1. 用户在建立『第一个』room的时候，不是project room! 用户只是指定了repo，所以这个第一个『无主题』聊天的room，就是repo room！后边在聊天的过程中产生的都是topic room。
> 2. project这个词和repo, topic这些都混用了。到底指代的是repo，还是topic，需要明确区分。(我在use case里可能混用了，后续一错再错。)
> 3. <control+repo>这对限制，是所有的闭包。
> 4. 在这个闭包里，有三个view: rooms, tasks, runs。『他们之间并没有必然的从属关系，都是关联关系、甚至是1对多的关系』← 我现在是这么想的，但是需要你来帮我做验证。

## 当前怎么读旧词

09-18 所有者明确顶层入口仍叫 Project，当前对应 Repo，主 Room 叫 Project Room；09-19 又确认 Rooms、Kanbans、Runs 并列、内容归属固定、引用可以交叉、各 Room 和 Run 独立选人。这不恢复旧混用：**Project 是顶层工作范围，Repo 是当前关联的代码来源，Topic 是其中讨论的主题。**

| 旧材料中的写法 | 当前体验读法 | 不能顺手推出什么 |
| --- | --- | --- |
| “新建 project，选择 SCM 地址” | 为这份 Repo 建 Project 入口 | 不是建一个 Topic，也不再套一层旧 Project 容器 |
| 同一 Control、同 Repo 的两个讨论 Project | 同一 Project 内各自选人的 Room；主 Room 与 Topic Room 的位置逐例核对 | 不是两份 Repo；Mac 的名册与执行位置保留，旧标签具体映射待 [C2](./open-questions.md#c2mac-用例的-room-位置) 确认 |
| “ctl + repo 是闭包” | 当前代码场景即同一 Control 中这份 Project 的工作边界 | 不推出全世界同一 Repo 只能由一个 Control 使用 |
| confession 中的 repo room | 当前统一称为 Project Room 的主 Room | 不在旁边再增加一间重复的 Repo Room |
| Topic Room、Scoped Room | 用户只面对 Topic Room | 不因合并名称就抹掉实际工作仍需处理的结果或请求 |
| 三个 view：rooms、tasks、runs | 导航为 Rooms、Kanbans、Runs；Task 在 Source 的 Kanban 中展示 | 不是删除 Task，也不把三列表变成包含链 |
| “有且只能一个 task-kanban” | Kanbans 按 Source 分项，可同时接多个 Source | 不再拿旧单板表述否定 GitHub Issues + Linear |
| “彼此没有必然从属” | 不要求 Room → Task → Run 的包含链；它们仍各有固定归属 | 不推出内容可跨 Project、消息可跨 Room、Task 可跨 Source 搬家 |
| Room 中已经选过 Participant | Run 仍独立选人；未来可用 TAMP 推荐与预填 | 不推出 Room 名单、权限与上下文自动继承给 Run |

读旧 Project 对象定义时标明“v0.18.6 的设计对象”，不凭拼写相同就判它等于当前 Project，也不把 Topic 机械替换成该对象。Q1 的产品方向已经确认：旧的“按目标或话题再建 Project”业务层退出，不能只在界面里藏起来；Repo 的代码与集成职责仍保留。内部模块与存储如何对应，是后续规范改写的工作，见[Q1 结论](./open-questions.md#q1旧设计的两层对象怎样对齐一个-project-入口)。

## 本轮已纠正到哪里

| 材料 | 本轮状态 |
| --- | --- |
| 多 Control、多 Repo 的用户叙事 | 已在 [01](./01-multi-unit.md#修正后的-project-与-room-对照)纠正工作层次；cloud 两处主 Room 已确认，Mac 两个旧标签的 Room 位置待 C2 核对 |
| 新建 Project、产生 Topic、观测 Run 的路径 | 已在 [02](./02-user-journey.md#用户体验流程)按最新口径整理 |
| Project Room 与三列表、内容归属与分别选人 | 已在 [04](./04-project-navigation.md#project-入口与-rooms--kanbans--runs)收录所有者原话与确认的组织结构 |
| 原始用例、confession log | 保留作史料，不继续在原话里改词 |
| 旧 S1 | 用户叙事转到本目录；原 CT 映射留存并标出尚未对齐 |
| 现行设计正文、spec、术语表的领域定义与 CT | 未全量改写；旧的 Repo → Project、三种 Room、单板/分组等仍需正式对齐 |
| 旧 Harness Memo | 本轮同题七份备忘已在文件头归档、注明覆盖范围并指向本目录；旧待拍板入口已撤下，正文与原话仍保留，见[旧讨论接手说明](./open-questions.md#旧讨论怎样接手) |

本目录的英文术语按所有者要求首字母大写，具体表在[目录用词](./README.md#本目录的用词)。原话、文件路径和场景 ID 保持原写法；正文不靠大小写区分新旧含义，而是明确指出使用哪一版的定义。

## 什么时候可以退役

所有者允许：所有混用修好后，confession 可以 deprecated 或 removed。当前还未到那一步。

退役时核对三件事：

1. 活跃体验、设计正文、spec、术语表、CT 使用同一套 Project / Repo / Topic 指代，引用链不再把旧讨论分组当顶层 Project。
2. 旧材料已明确标成历史或已被后续文件接替，读者不用靠这份 confession 才能判断当前含义。
3. 待核对与待拍板项已处理，正式改动有对应记录和验证，而不只是把搜索结果里的单词替换掉。

满足后可以从当前阅读路径移除此文件，必要史料仍可在原 log 与 Git 历史中找到。不为保存“认错记录”让它永久挡在新读者面前；也不因为本目录写对了就提前宣布全库已纠正。

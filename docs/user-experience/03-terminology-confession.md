# project / repo / topic 混用的 confession

> 状态：保留中；本目录已纠正体验叙述，旧设计和 CT 尚未全量对齐，不满足删除条件。<br>
> 日期：2026-09-18<br>
> 原始记录：[所有者与 Claude 的对话 §第一轮](../../.memo/log/2026-09-18-project-repo-topic-混用.md#第一轮)。原记录保留，不改写历史。

这份记录解释旧材料为什么会误导后续结构讨论，不是额外的产品概念。修正的是词的指代，不是让用户放弃原来的多机、多来源、并行协作需求。

## 所有者原话

以下只摘录所有者的 confession，保留原文写法；同一 log 中 harness 的后续方案不作为所有者裁决。

> 我意识到，好像是我自己混用了术语，导致后边有很多的混乱（主要责任在我，但你也有次要责任 - 你要记得规劝我啊）：
>
> 1. 用户在建立『第一个』room的时候，不是project room! 用户只是指定了repo，所以这个第一个『无主题』聊天的room，就是repo room！后边在聊天的过程中产生的都是topic room。
> 2. project这个词和repo, topic这些都混用了。到底指代的是repo，还是topic，需要明确区分。(我在use case里可能混用了，后续一错再错。)
> 3. <control+repo>这对限制，是所有的闭包。
> 4. 在这个闭包里，有三个view: rooms, tasks, runs。『他们之间并没有必然的从属关系，都是关联关系、甚至是1对多的关系』← 我现在是这么想的，但是需要你来帮我做验证。

## 当前怎么读旧词

本轮又明确提出：顶层入口仍叫 `project`，当前 `project = repo`，主 room 因而叫 `project room`；三列表叫 `rooms / kanbans / runs`。这不恢复旧混用：**project 始终是顶层工作入口，topic 始终是其中讨论的主题。**

| 旧材料中的写法 | 当前体验读法 | 不能顺手推出什么 |
| --- | --- | --- |
| “新建 project，选择 SCM 地址” | 为这份 repo 建 project 入口 | 不是建一个 topic，也不要求再露出一层旧 Project 容器 |
| 同一 ctl、同 repo 的两个讨论 project | 同一 project 下的两间 topic room | 不是两份 repo；Mac 用例的名册与执行位置照旧保留 |
| “ctl + repo 是闭包” | 当前代码场景即 ctl + project 的工作边界 | 不推出全世界同一 repo 只能由一个 ctl 使用 |
| confession 中的 repo room | 本轮名为 project room 的主入口 | 不在旁边再增加一间重复的 repo room |
| topic room、scoped room | 用户只面对 topic room | 不因合并名称就抹掉实际工作仍需处理的结果或请求 |
| 三个 view：rooms、tasks、runs | 本轮导航为 rooms、kanbans、runs；task 在 source 的 kanban 中展示 | 不是删除 task 这个概念，也不把列表变成包含链 |
| “有且只能一个 task-kanban” | kanbans 按 source 分项，可同时接多个 source | 不再拿旧单板表述否定 GitHub Issues + Linear |

读旧 `Project` 对象定义时先标明“v0.18.6 的设计对象”，不凭拼写相同就判它等于当前 project，也不把 topic 机械替换成该对象。当前产品名与后端模块映射是两层问题。

## 本轮已纠正到哪里

| 材料 | 本轮状态 |
| --- | --- |
| 多 ctl、多 repo 的用户叙事 | 已在 [01](./01-multi-unit.md#修正后的-project-与-room-对照)纠正；cloud 两处原文不足以唯一还原的 room 位置显式待核对 |
| 新建 project、产生 topic、观测 run 的路径 | 已在 [02](./02-user-journey.md#用户体验流程)按最新口径整理 |
| 主 room 与三列表 | 已在 [04](./04-project-navigation.md#project-入口与-rooms--kanbans--runs)收录本轮原话与当前结构 |
| 原始 use case、confession log | 保留作史料，不继续在原话里改词 |
| 旧 S1 | 用户叙事转到本目录；原 CT 映射留存并标出尚未对齐 |
| 现行设计正文、spec、术语表的领域定义与 CT | 未全量改写；旧的 Repo → Project、三种 Room、单板/分组等仍需正式对齐 |
| 旧 harness memo | 是当时基线上的解释，不是当前需求权威；包括 Codex 上一轮 memo 的旧闭包读法 |

同级术语在本目录统一用英文，具体表在[目录用词](./README.md#本目录的用词)。原话里的中文或中英混用为了溯源保留，不将它复制成新正文的写法。

## 什么时候可以退役

所有者允许：所有混用修好后，confession 可以 deprecated 或 removed。当前还未到那一步。

退役时核对三件事：

1. 活跃体验、设计正文、spec、术语表、CT 使用同一套 project / repo / topic 指代，引用链不再把旧讨论分组当顶层 project。
2. 旧材料已明确标成历史或已被后续文件接替，读者不用靠这份 confession 才能判断当前含义。
3. 待核对与待拍板项已处理，正式改动有对应记录和验证，而不只是把搜索结果里的单词替换掉。

满足后可以从当前阅读路径移除此文件，必要史料仍可在原 log 与 Git 历史中找到。不为保存“认错记录”让它永久挡在新读者面前；也不因为新目录写对了就提前宣布全库已纠正。

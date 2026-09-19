# Project 入口与 Rooms / Kanbans / Runs

> 状态：所有者已确认的体验与组织结构；对应约束与验收随 #257 同步，不是产品已实现的报告。<br>
> 日期：2026-09-19<br>
> 来源：所有者 09-18 的导航提议、09-19 的侧栏示意、归属说明、待处理入口与 C2 确认，以及随后按此修订体验目录的要求；原话见文末。

**Project 是顶层工作范围，每个 Project 当前关联一个 Repo；同一 Control 可以为同一 Repo 建多个 Project。** 点击一个 Project 的名称进入它自己的唯一主 Room。本目录把这间主 Room 称为 Project Room，所有者所说的主 Repo Room 指同一间；不是同一 Repo 的所有 Project 共用一间。每个 Project 下的 Rooms、Kanbans、Runs 是并列入口；内容归属固定，引用可以交叉。

保留 Project 这个产品名，是为了未来能扩展到不完全由 Repo 定义的工作，不是本轮立刻实现那些扩展。当前做事时需明确哪个 Control 中的哪个 Project；只知道 Control 与 Repo，不足以区分同仓的两个 Project。名称的澄清过程见[术语纠正](./03-terminology-confession.md#当前怎么读旧词)。

## Project 是主入口

下图按所有者的 Apollo 示例整理，表示用户导航，不规定数据库表或部署结构。新建时 Rooms 为空，图中两个 Topic Room 是后续讨论产生的。

```text
Project Apollo (Repo: github.com/yesme/apollo-mission-decrypt)  [待你处理 2]
├─ Rooms                                  点击 → 展开 / 收起列表
│  ├─ 优化构建系统 (Topic Room)             点击 → 这间聊天室
│  └─ 整理设计文档 (Topic Room)
├─ Kanbans                                点击 → 展开 / 收起列表
│  ├─ Code Collaboration (GitHub Issues)   点击 → 这个 Source 的看板
│  └─ Project Management (Linear)
└─ Runs                                   点击 → 展开 / 收起列表
   ├─ Run 1                               点击 → DAG 与各步骤执行状态
   └─ Run 2
```

点击 Project 名称，右侧打开主 Room；点击旁边的“待你处理 2”，打开待处理面板。两处分别响应点击，子列表仍只有 Rooms、Kanbans、Runs。

Project Apollo 是工作名称，Repo 地址说明它当前关联的代码来源。Code Collaboration、Project Management 是用户给 Source 入口取的显示名称，不是新增模块或另一层 Project。

同一 Control 中，两个 Project 即使显示相同 Repo 地址，也有各自的主 Room、Rooms、Kanbans、Runs 与待处理入口，不合成一份工作。多 Control 的联合界面还要标清每个 Project 来自哪个 Control。这个边界不阻止多个 Project 共用 Agency、内容服务或读取同一外部代码事实，也不要求每个 Project 有独立的服务器或 Git 对象库。

## 待你处理：从 Project 标记进入

“待你处理 2”表示本 Project 有两件确实需要当前用户处理的事。面板汇总相关 Room、Task、Run 已有的请求与状态，是这些事项的处理入口，不另建一套 Task 或决定权限。

每条事项回答四个问题：

1. **哪件事？** 显示主题与所在的 Room、Task 或 Run，能回到原处查看。
2. **为什么需要你？** 说明缺少哪项决定、输入或授权，以及已有依据。
3. **你能做什么？** 给出可执行的选择及后果；需要深入讨论时回到相关 Room。
4. **不处理会影响什么？** 说明哪些工作在等这次处理，不把局部等待说成整个 Project 都停了。

例如，事项可以是：

- Run 12 的检查与评审已通过，但本次授权要求人在合入前批准，已有 Request 向你索取这项决定。用户查看变更和依据，按原有合入预览批准或拒绝；不是所有 Run 都新增这一关。
- “优化构建系统”Topic Room 的两种做法互斥，已明确发出需要你选择的 Request；普通讨论分歧本身不生成待办。条目列出各自收益与代价，用户选择、补充要求或回到 Room 继续讨论，答复留在原讨论中。
- 一项无 Run 的文档 Task 已有交付，但其验收要求还需要人确认。用户从条目回到 Task 查看结果并提交原有验收动作，不为此新建 Run 或另一份 Task。

打开、阅读或关闭面板都不算处理完成。答复或操作实际生效后，事项退出待处理列表，处理历史保留在原 Room、Task 或 Run。同一事项即使在多个页面、设备或客户端可见，也只处理一次；别处已处理的，面板随之更新。

等待 CI、自动重试或 Agency 自行恢复，显示为进度；普通消息显示未读，允许忽略的 Topic 建议留在建议入口。只有确实需要人作决定、提供输入或补充授权的事项才进入“待你处理”，已有授权足够的工作照常推进。

## Rooms：只列 Topic Rooms

Rooms 是 Topic Rooms 列表，初始为空；Project Room 由 Project 名称进入，不在列表里重复出现。每间 Topic Room 有自己的主题、消息与 Participant 名单，可以与主 Room 使用相同的选人方式。

从聊天展开的 Topic Room 以本 Project 主 Repo Room 的相关讨论构造前情提要；Participant 读这份开场材料就能开始后续讨论，不以读完原 Room 历史为前提。出处用于追溯，前情提要承接背景，后续会话各自独立；完整流程见 [T1](./02-user-journey.md#t1聊天归纳接受或忽略-topic-建议)。已有 Request 升级为 Topic 时，提要根据请求及所阻塞的工作准备，不要求主 Room 已有讨论。

新 Topic 不创建新 Project。旧 Scoped Room 的讨论用途并入 Topic Room，不为临时澄清和普通讨论再造两个并列产品概念。若一个 Topic 真承接了需要回复的事项，可以显示这种关联；不是所有 Topic 一出生就有 Task、Run 或预定交付物。承接未解决 Request 的房间仍有闲置关注提醒，但不是另一份待办；普通 Topic 没有这项提醒。

## Kanbans：每个 Source 一个入口

Kanbans 接受多个 Source；每个 Source 是列表中的一个入口。用户可以同时看 GitHub Issues 和 Linear，不用为了第二个 Source 再建 Project。

Source 指选定的实际来源及范围，不只是品牌名字。Task 保留自己的 Source；Kanban 是该 Source 的可视化入口，不另拥有一套复制出来的任务内容。GitHub Issues 中的 Task 不会通过跨 Kanban 拖放变成 Linear 中的 Task。

Project 是独立的 Namespace（工作与授权的命名范围）。两个 Project 可以各自把同一张外部卡认领为自己的 Task，各有契约、Run 与验收状态；同一 Project 内才去重为一个 Task。它们不复制外部卡，源端的标题、评论或关闭态仍是双方可见的共享事实；Project A 的 Task 完成不替 Project B 作验收。

同一 Source 内的状态、泳道与排序变化，不是跨 Source 搬家。本轮没有要求另加一张强制的合并板；后续的汇总显示也不能替代按 Source 的入口。

## Runs：完整列出活动执行

Runs 列出本 Project 所有活动 Run，无论它从哪个 Room、哪个 Task 或哪个入口开始。等待输入或处理中的 Run 不因为当前没有执行节点就消失。

为 Task 保存、尚未开工的计划从该 Task 找回，见 [T3](./02-user-journey.md#t3模板与表单生成-run-计划)；不必为了保留计划先启动执行。

点击 Run 默认展示 DAG，标清哪些步骤正在执行、哪些已经执行；并行节点同时可见。任务书仍是同一 Run 的另一种视图，二者共用 Worker 侧栏，见[观察流程 R2/R3](./02-user-journey.md#r2任务书与-dag-共享-worker-侧栏)。历史如何折叠由界面设计处理，不在活动列表不等于已删除执行记录。

执行、检查、评审、合入与 Task 验收的结果分别可见，不把“Worker 停止输出”当作交付完成；异常与结果未知时的展示见 [R4/R5](./02-user-journey.md#r4从执行结果到交付完成)。

## 松散耦合具体意味着什么

**固定的是归属，可以交叉的是引用。** 不强制形成 Room → Task → Run 的包含链，也不因此取消各自内容的归属。

| 内容或关系 | 已确认的体验 |
| --- | --- |
| Project 的工作内容 | 既有 Room、Task、Run 不从 Project A 搬到 Project B；两者使用同一 Repo 也不合并归属，换一个讨论入口不改变所属工作范围 |
| Room 的讨论内容 | Topic Room 1 的消息不能挪成 Topic Room 2 的原始消息；Room 2 可以引用它，并保留原出处 |
| Task 的来源 | Kanban 1 对应 Source 中的 Task 不搬到 Kanban 2 对应的 Source；引用不复制卡片、不改变来源 |
| Room 与 Task | 一间 Room 可以讨论不同 Source 的多项 Task，同一 Task 可以被多间 Room 引用；Task 不从属于讨论它的某间 Room |
| Room 与 Run | 多间 Room 可以引用和讨论同一个 Run；关 Room 不自动停止 Run，也不改它的任务书或授权 |
| Task 与 Run | 可以互相引用工作与结果；引用不等于承担完成该 Task 的责任，Run 完成也不因多了一条引用就自动完成更多 Task |

例如，“修改构建指南”是 GitHub Issues 中的一项 Task；“优化构建系统”和“整理设计文档”两个 Topic Room 都讨论它，并引用 Run 1。关闭其中一间 Topic Room 后，Task 仍在原 Source 的 Kanban 中，Run 1 仍可从 Runs 进入，不需要为它们找一个新的所属 Room。

纯讨论可以没有 Task 或 Run；已有 Task 可以先读回，不先创建 Topic Room；已有 Run 可以直接从 Runs 观察。**松散耦合不等于所有关联都必须任意多对多。** 一个 Run 承担几项 Task 的完整交付、同一 Task 能否并发多个施工 Run、完成怎样判定，仍是执行语义；本次没有借导航与归属调整取消现有相关限制。

## Participant 分别选入

每个 Room 的 Participant 名单独立选择，每个 Run 的施工与评审 Participant 也独立选择。可以选相同的工种或配置，但这不把两个 Room、两个 Run 或 Room 与 Run 合成一份名单与授权。Project 不因能显示全部 Participant，就额外拥有一份会自动灌入所有 Room 和 Run 的成员名单。

未来 TAMP 相关的推荐可以利用历史选择、配置和履约记录，形成继承或派生关系，帮助推荐与预填。本次选入的职责、工作范围与授权仍分别明确；推荐不自动继承原来的权限或整份上下文。本轮没有规定推荐算法或自动任用机制。

观察和输入仍指向这次具体执行，并经 Agency 提供相应能力；同名 Participant 不成为跨 Room 或 Run 转投输入的理由。

## 组织层可观测

系统替用户组织得越多，用户越需要看得见「为什么这样安排」。可观测分三层，前两层现行已有，本轮补第三层：

- **事情层**：「待你处理」与它的四个问题——哪件事、为什么需要你、你能做什么、不处理会影响什么。
- **执行层**：结构化的执行流、终端观察的阶梯（图形、终端、仅 Headless）与证据的三档；模型的自述不替代观察。
- **组织层**：每张卡、每间房、每次 Run 都能打开来源链——这张卡由哪段讨论采纳、这间房从哪份前情提要开始、这次 Run 冻结了哪份清单——从「这个承诺为什么没完成」追到「哪个执行、哪个版本、哪个检查在阻塞」。HCTL 的每一步组织都是带来源的类型化记录，缺的只是把来源链摆出来的视图。

可观测要接上干预的边界，按轻重分：改一个还没采纳的候选很轻，直接改；改活动 Run 的契约与权限走版本与替代，不在原地改；停止一次执行不等于撤回它已经发生的副作用，副作用按原对象的规则处理。本节只定体验，不新增对象或命令；机制与约束另行安排。

## 对旧模型的结论

**Project 与 Repo 不合成一对一关系；同一 Control、同一 Repo 上的多个 Project 保留。** 当前 Project 内展开话题是建立 Topic Room，不另造一层 Project；用户明确新建 Project 则是独立的工作范围，不能被整理成别的 Project 下的一间 Topic Room。

此前把 Q1 概括成“同 Repo 下多个 Project 的业务层退出”扩大了所有者的意思，现按 [C2 原话](#2026-09-19c2-确认)纠正。Repo 仍负责代码来源、平台绑定、版本与集成，Project 负责各自工作的组织。Project 的默认设置、执行授权和归档规则逐项核对，不因纠词丢掉，也不整套复制到 Topic Room；同步落点见[接手清单](./open-questions.md#规范对齐清单)。

## 本轮原话

### 2026-09-18：入口与列表

以下保留所有者原写法；历史引文不随正文的术语大小写调整。

> 用户还是建立个project，project=repo。然后有个project chatroom，就在用户在左侧边栏一点击project就是。叫project有另一个好处，就是并不完全被绑定到repo上，未来或许我们还能拓宽应用场景。
>
> 在project下边，有三类view/list：rooms, kanbans, runs - rooms本质上就是topic rooms list (一开始是空的) (也吸收了scoped room，合二为一了)；kanbans可以接受超过1个source (每个source是list里的一个item)，即解决“又要github issues又要linear”的问题；runs则列出了所有在跑的runs, UI前边讨论过了。
>
> room / kanban / runs 之间是松散耦合。

### 2026-09-19：归属与选人

所有者的 Apollo 侧栏原示意如下；其中 Run 条目已明确写了点击后展示 DAG，不是整理者另定的缺省：

```text
Project Apollo (Repo: github.com/yesme/apollo-mission-decrypt) ← 点这里右边展示repo room讨论/日常讨论
+ Rooms ← 点这里toggle展开/收起Rooms列表
  - 优化构建系统 (Topic Room) ← 点这里右边展示聊天室
  - 整理设计文档 (Topic Room)
+ Kanbans ← 点这里toggle展开/收起Kanbans列表
  - Code Collaboration (Github Issues) ← 点这里右边展示看板列表
  - Project Management (Linear)
+ Runs ← 点这里toggle展开/收起Runs列表
  - Run1 ← 点这里右边展示DAG图（以及现在有哪些正在执行、哪些已经执行）
  - Run2
```

同一条消息继续说明：

> 这里也天然地形成了层次感的闭包：
>
> - Project A里的内容，不能搬到Project B里去
> - Project A里的Rooms/Kanbans/Runs，之间可以互相reference
> - Topic Room 1里的内容，不能『挪』到Topic Room 2里边去。
> - Kanban 1里的task，也不能『挪』到Kanban 2里边去。
> - 每个Room的participants，都是独立选出来的；每个Runs里的participants，也是独立选出来的 - 当然，未来可能是可以有一定的继承和派生关系，即自动化的推荐 (TAMP相关讨论)。

随后所有者确认按上述讨论整体修订本目录、记下组织结构。本页记录的是这一确认，不把其他 Harness 的评估当作需求来源。

### 2026-09-19：前情提要与 Run 体验补充

> 新 Topic Room 在创立时，需要从repo room里构造出一份『前情提要』，并用这份『前情提要』来解耦与原长篇大论room之间的关系。这样新topic room里的participants只需要阅读该『前情提要』，就能开始后边的讨论和协作。

同轮所有者同意补写 Run 的交付、观察与异常体验供其查看。

### 2026-09-19：待你处理入口

> 有意思，『等人处理』其实把Project那一行分成了两个部分：点project的名字是打开右边的聊天室，点project旁边的『待你处理 2』则是打开待处理面板来处理内容。我觉得这个设计是okay的。包括后边的4个问题、处理后行为、避免变成噪声中心，这些都可以。

据此确认同一 Project 行上的两个独立入口，以及上文的条目内容、处理后行为和降噪规则。

### 2026-09-19：C2 确认

所有者针对 Mac 用例的 `mac_jssdk_01`、`mac_jssdk_02` 明确：

> 两个都是主room. 事实上一个control当然可以对同一份repo开两个project (repo room).

据此保留两个 Project、各自主 Room 与原名册。“每个 Project 一个主 Room”不等于“每个 Repo 只有一个 Project”；此前把 Mac 两组工作合为一个 Project 的整理撤回，其余导航、内容归属和独立选人的确认不变。

### 2026-09-19：Project 独立 Namespace

所有者指出此前拟写的“两个 Project 读到同一张卡时不能复制成两个 Task”错误：

> 如果是两个project，这就是两个independent的namespace。这样在各自的namesapce里各自有自己的task，不是很正常么？

据此，外部实体到 Task 的唯一映射改为每个 Project 内；各 Project 可以有自己的 Task，共享外部卡片不共享承诺、Run 或验收。这是本轮明确修订的约束，不再把旧的 Control 级唯一范围当作新体验的前提。

### 2026-09-19：三条原则里的可观测性

> 可观测性非常重要：AI越能自组织、用户越需要可观测性 － 例如，可以看到harness在干什么

出处 [群体智能三原则](../../.memo/log/2026-09-19-群体智能三原则.md)（所有者 2026-09-19）；三条原则的整理见[备忘](../../.memo/design/intent-entry-principles-20260919.md)。上文「组织层可观测」按此补写；归纳机制不在本轮。

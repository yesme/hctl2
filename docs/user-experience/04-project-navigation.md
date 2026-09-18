# project 入口与 rooms / kanbans / runs

> 状态：所有者 2026-09-18 本轮提出的体验方向；作为结构讨论基线，后端映射尚待对齐。<br>
> 日期：2026-09-18

**当前用户建立的是 project，它对应 repo；点击 project 进入 project room。** project 下的三个列表是并列入口，不是 room → kanban → run 的包含链。保留 project 这个产品名，是为了未来能扩展到不完全由 repo 定义的工作，不是本轮立刻实现那些扩展。

## 本轮原话

以下摘录所有者本轮的第 4 项，保留原写法：

> 用户还是建立个project，project=repo。然后有个project chatroom，就在用户在左侧边栏一点击project就是。叫project有另一个好处，就是并不完全被绑定到repo上，未来或许我们还能拓宽应用场景。
>
> 在project下边，有三类view/list：rooms, kanbans, runs - rooms本质上就是topic rooms list (一开始是空的) (也吸收了scoped room，合二为一了)；kanbans可以接受超过1个source (每个source是list里的一个item)，即解决“又要github issues又要linear”的问题；runs则列出了所有在跑的runs, UI前边讨论过了。
>
> room / kanban / runs 之间是松散耦合。

所有者同时要求：同一级别的概念不要一会儿中文、一会儿英文；这些单个词可以全程英文。本目录采用[统一用词](./README.md#本目录的用词)。

## project 是主入口

以下仅表示用户导航，不表示数据库外键或部署结构。图中的 topic rooms 是后续展开的例子；刚建好 project 时该列表为空。

```text
project：gh-jssdk                 点击这里 → project room
├─ rooms
│  ├─ topic room：接口设计
│  └─ topic room：性能排查
├─ kanbans
│  ├─ GitHub Issues              一个 source，一个入口
│  └─ Linear                     另一个 source，另一个入口
└─ runs
   ├─ run A
   │  ├─ 内容介绍 / 任务书
   │  └─ DAG 拓扑图
   └─ run B
```

新 topic 不创建新 project。主 room 与 topics 都属于同一 project；不再同时要求用户区分 repo room、旧 Project room 和 scoped room 三种产品入口。

多 ctl 的联合界面仍标清每个 project 来自哪个 ctl。同一外部 repo 在另一个 ctl 中有自己的 project，不与当前工作合成一份身份。这个边界不会阻止多个 project 共用 agency 或访问同一外部代码事实。

## rooms：只列 topic rooms

`rooms` 是 topic rooms 列表，初始为空；project room 由 project 本身进入，不在列表里重复出现。topic room 有自己的主题、消息与选入名单，可以与主 room 使用相同的选人方式。

scoped room 的讨论用途并入 topic room，不为“临时澄清”和“普通讨论”再造两个并列产品概念。若一个 topic 真承接了需要回复的事项，可以显示这种关联；不是所有 topic 一出生就有 task、run 或预定交付物。

## kanbans：每个 source 一个入口

`kanbans` 接受多个 source；每个 source 是列表中的一个 item。用户可以既看 GitHub Issues，也看 Linear，不用为了第二个 source 再建 project。

一个 source 指所选定的实际来源及范围，不只是品牌名字。每项 task 保留来源，跨 source 关联不自动搬卡或同步同名卡。本轮没有要求另加一张强制的合并板；如果后续需要汇总视图，应作为显示选择，而不能替代按 source 的入口。

## runs：完整列出活动执行

`runs` 列出本 project 所有活动 run，无论它从哪个 room、哪个 task 或哪个入口开始。等待输入或处理中的 run 不因为当前没在执行一个节点就消失。

每个 run 的任务书、DAG 两入口及共同 worker 侧栏沿用[观察流程 R2/R3](./02-user-journey.md#r2任务书与-dag-共享-worker-侧栏)。切换 room 或 kanban 不改变 run 的所属范围。历史如何折叠可由界面设计处理，不能把“不在活动列表”当成已删除执行记录。

## 松散耦合具体意味着什么

这张表说明关联可以怎样发生，不预定底层关系表。

| 两类入口 | 用户应能做的事 | 不强制的关系 |
| --- | --- | --- |
| room ↔ kanban/task | 在一个 room 讨论不同 source 的多项 task，也从 task 回到相关讨论 | 不要求一 room 对应一个 source；task 不从属于 room |
| room ↔ run | 在不同 room 引用、讨论同一个 run；从 room 发起计划或跳到执行 | 不要求每个 run 自带新 room；关 room 不自动停止 run |
| kanban/task ↔ run | 从 task 查看关联 run，从 run 找到它要完成的工作与来源 | 不是整张 kanban 对应一次 run；没 run 的 task 也能存在 |

因此，纯讨论可以没有 task 或 run；已有 task 可以先读回，不先创建 topic room；已有 run 可以直接从 `runs` 观察，不先经过来源 room。

**松散耦合不等于所有关联都必须任意多对多。** 一个 run 能对应几项 task、同一 task 能否并发多个 run，以及完成记录怎样判定，属于执行语义。当前规范仍有相应限制，本轮不借导航重排擅自取消；待对齐位置见[后续清单](./open-questions.md#规范对齐清单)。

本目录也不把“都在 project 下”解释成每 project 一套服务器、Git 对象库或 agency。它描述用户工作的归属和入口，不增加部署要求。

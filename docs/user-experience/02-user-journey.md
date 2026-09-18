# 用户体验流程

> 状态：所有者的目标体验；未声称完整应用已交付。<br>
> 日期：2026-09-18<br>
> 来源：2026-09-17 用户流程；按本轮 [project 导航](./04-project-navigation.md#project-入口与-rooms--kanbans--runs)更新名称与多 source 选择。

这是一条从“有一份代码”到“展开讨论、安排工作、观察执行”的用户路径，不是代码施工顺序，也不是强制每个 room 都先有 task、每个 task 都先有 run。

## 新建 project

### P1：选择代码来源

用户在 workbench 选择新建 project，并给出 SCM 地址或本地 Git 路径。

- **远端地址：** 以所选 repo 建立当前 project 的工作入口。
- **本地路径、有 remote：** 读出对应 remote 供用户选择。推荐接原 remote，继续使用它；也提供断开原 remote、用 Gitea 另起一份独立工作的选项。
- **纯本地：** 直接走 Gitea 本地方案。

这保留原流程的两个选择，不把“本地另起”删掉。是否改输入目录本身的 remote，还是另建独立副本，原话没有完全定清，见[待拍板 Q2](./open-questions.md#q2本地-detach-会改哪份目录)。本轮不替用户决定物理修改范围。

多 ctl 场景里，用户知道正在为哪个 ctl 建 project。本地路径指用户提供的那台机器上的路径，不是远端 ctl 上碰巧同名的目录。读取失败、远端不可达、未支持的平台都如实显示，不当成“纯本地”悄悄换路径。

当前 `project = repo` 是工作入口的口径：不再要求用户为了同一 repo 的新 topic 再建 project。名字保留 project，是为了未来不把产品永久限定在 Git；本轮没有追加多 repo 或非代码 project 的实现需求。

### P2：接入 kanbans

用户选择要接入的 source，读回 tasks 并可视化。推荐顺序保留：SCM 自带 issues 优先；已有外部系统（如 Linear）其次；也可使用本地方案。

**一个 project 可以同时接多个 source。** 例如 GitHub Issues 与 Linear 分别占 `kanbans` 的一个条目，用户不必二选一。原流程“有且只能有一个 task-kanban”在这里按本轮要求更新，不藏成“底层多源、界面仍只能看一张板”。

每个入口显示所选 source 范围中的 tasks；卡片保留来源。跨 source 展示和关联不意味着复制卡片、搬家或自动同步同名 task。尚不支持的 source 操作要可见，不能因能读列表就声称能完成所有写操作。

### P3：选人并进入 project room

用户从 agency 选择 participant，完成 project room 的初始阵容。点击左侧 project 本身就进入这个主 room。

此时 `rooms` 列表仍为空，因为它列的是后来展开的 topic rooms，不重复列 project room。选人是协作准备，不自动启动 run。当前步骤顺序服务首次使用，不把“先建 kanban”变成所有聊天的前置条件。

## 产生 topic 与 task

### T1：聊天归纳，接受或忽略 topic 建议

聊天中，系统持续归纳值得展开的 topic，在输入框上方或通知位置提示“这个 topic 值得尝试（Yes / No）”。

用户接受后，可以确认主题并按同样的方式选择 participant，建立新的 topic room；它出现在当前 project 的 `rooms` 中。忽略则不创建。主 room 与 topic room 都可以继续讨论，不因有了 topic room 就生成另一个 project。

这个体验要求包括持续建议，不只是一颗“手动新建 room”按钮。归纳尚未配置或暂时失败时应如实显示；手动创建可以作为替代入口，但不能算持续建议已经实现。用什么模型、怎样触发和控制费用是后续设计，不在这里预定常驻 agent 或新的 workflow。

### T2：在两种 room 里操作 task

project room 和 topic room 都能通过聊天建立、修改或删除 task。用户看到准备操作哪个 source 的哪项 task、内容和后果，再作决定；不能只收到模型一句“做完了”。

接了多个 source 时，新 task 的目标 source 需要明确。由哪种默认值减少重复选择交给产品实现；没有明确目标时，不因当前 room 的名字猜 source。

“删除 task”暂不偷换成“取消 task”：草稿、已有执行记录的 task、外部 source 卡片的删除后果不同，见[待拍板 Q3](./open-questions.md#q3删除-task-的后果)。删除一个讨论入口也不自动删除它提到的 task 或停止关联 run。

### T3：模板与表单生成 run 计划

用户可以为 task 选择快捷模板，填写交互式表单，得到 run 的 DAG 和任务书。它们是同一份计划的两种表达，不让人维护两份相互漂移的内容。

生成计划不等于立即开工。进入实际执行前，用户能确认做什么、谁做、交付什么、如何验收以及所需资源。并非每个 task 都要 run；也不把讨论、task 管理、执行导航做成只能单向逐级进入的流程。

## 观测 run

### R1：从 project 的 runs 进入

左侧 project 下只有本轮指定的三类列表：`rooms`、`kanbans`、`runs`。活动 run 独立列在 `runs`，不藏在发起它的 room 或某张 task 卡下面；从关联对象也可以跳转过去。

### R2：任务书与 DAG 共享 worker 侧栏

每个 run 有两个入口：内容介绍、DAG 拓扑图。前者展示任务书，后者展示实际执行到哪里；并行时可以同时看到多个进行中的步骤。

无论看哪一个，最右侧都展示这个 run 的各个 worker 及其当前步骤。切换视图不切换执行，不把两个入口变成两次 run。

### R3：按供给能力观察 worker

点击 worker，按选人时约定的能力进入图形工作面（例如 Grok Bot 式计算机）、Herdr 式 TUI，或 headless 方式。headless 没有交互屏幕，不等于没有进度、结果或需要人处理的请求。

这里描述用户想看到的能力，不强迫每个 worker 都有一台 VM，也不承诺某个第三方已有可嵌入接口。缺能力要在选择时讲清，不能等用户点开才伪装成另一种已支持的体验。

## 从另一台机器继续

用户可以在另一台机器的 workbench/cli 连接原 ctl，继续查看同一 project 的 rooms、kanbans、runs，并处理需要人的动作。关闭原前端不停止远端工作；新前端也不自动接管 harness session。

这是[多单元用例 S1.5、S1.9–S1.11](./01-multi-unit.md#操作步骤)在日常流程中的体现。通过 Git 接手本目录的文档工作则只需更新 main；两种“接手”不要混为执行迁移。

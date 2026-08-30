# C2 设计层:Grok 删除安全抽审

> 状态:讨论中 · C2 闸门报告(只列表,不改十个设计文件)<br>
> 对象:PR #82 `codex/doc-overhaul-c2-design` @ `1c46ca9`(十个设计层文件;C2 未改 `spec/**` / `delivery.md`)<br>
> 对照:[施工图](./README.md) §2 红线、[`03-approved-plan.md`](./03-approved-plan.md) §3 C2 行、[`02-target-map.md`](./02-target-map.md) §4/§5、[`04-prohibition-whitelist.md`](./04-prohibition-whitelist.md)、K3 [`01-inventory-prohibitions.md`](./01-inventory-prohibitions.md) 四模块表、[`review-c2-fable.md`](./review-c2-fable.md)(Fable 四条发现不重复)<br>
> 去向:所有者裁决后与 Fable 通读合成一次修正,推回 PR #82;本文件不进合同<br>
> 结论取值只能是:通过 / 回滚哪几句 / 进停车位

## ① 四模块「关键规则」否定句:有无把白名单 A 类当 C 类删掉

白名单 A 类是「权威位置上规则本体」。§1 点名的 A 类全部在合同层:`spec/agent.md:34/:38`、`spec/run.md:37/:39`、`spec/task.md:64/:66/:68`、`spec/system.md:109/:111/:179–185`、`connections.md:10/:16/:115/:168`。C2 的十文件 diff 不含 `spec/**`,这些句子仍在 main 原文。白名单把设计层关键规则点名为 **C 类**(删或改正面句 + 指向合同):`agent.md:23/:27/:30`、`task.md:22/:26`、`run.md:38/:44`、`project.md:19/:22/:24/:74`。

逐条对照 K3 表里四模块正文「关键规则」命中(角色表整列见 ②;`:74` 在交接节,附记不另开问):

| K3 文件:行 | 原否定(截) | 白名单 | PR #82 处置 | A 类仍在 |
| --- | --- | --- | --- | --- |
| project.md:19 | 消息/反应/模型建议不能推断成命令;Room 不能签发 Verdict/Receipt/完成 Task | C | 改正面句 + 指向 `spec/project.md#场景合同`(新 :19) | `spec/project.md:99`(普通消息不能触发派发;结构化动作才可能成为命令请求) |
| project.md:22 | 模型自由总结不能替代来源和版本 | C | 改正面句「上下文保留来源和版本」(新 :22);本条未另加 spec 链 | `spec/project.md:61` Manifest 必须冻结每个来源的 stable ref + version/digest |
| project.md:24 | 不能抹掉已被引用的历史 | C | 改正面句「保留已被引用的历史」(新 :24) | `spec/project.md:54` 只追加;不能抹掉已被引用的历史 |
| task.md:22 | 契约永不漂移;历史永不改写或物理删除 | C | 改正面句「固定原契约版本 / 保持只追加」(新 :22) | `spec/task.md:58` Task Revision 只追加;Reopen 不改写旧完成历史 |
| task.md:26 | 适配器/Harness/执行体不能绕过完成命令写凭证,也不能冒充人或归约器 | C | 整条删除;两来源人话留在新 :25 并指向 `spec/task.md#写入合同` | `spec/task.md:68` 两个获准 actor;`:76` 裸终态/自述/Git/CI 都不是命令 |
| run.md:38 | 除声明可变放置参数外不得原地漂移 | C | 改正面句「只有清单声明的放置参数可变;其余通过结束或替代生效」(新 :34) | `spec/run.md:25` 替代创建新 Run;`connections.md:145` 范围/权限/验收变化要显式替代 |
| run.md:44 | 当前指针或文件路径不能替代版本 | C | 改正面句「绑定精确的评审对象版本」(新 :40) | `connections.md:16`(A 类点名)`current`/路径不能替代精确引用 |
| agent.md:23 | 观测无论多可信都只是观测;不能自动变成领域结果 | C | 改正面句(见 ③;新 :23) | `spec/agent.md:70` 无论置信度多高都不能自行推进领域结果 |
| agent.md:27 | 三条底线不可关闭(设计层人话) | C;白名单 §2 从必留清单转走 | 人话一句 + 指向 `spec/agent.md#写入合同`(新 :27) | `spec/agent.md:34`(A 类点名)完整定义仍在 |
| agent.md:30 | 证明不了同一进程就不能自称精确接管 | C | 改正面句「按可证明的运行时身份标注恢复等级」(新 :30) | `spec/agent.md:92` 无法证明同一进程时不能声称 exact attach |

另外两条关键规则里的否定不在 K3「不得/不能」命中、但是 C 类复述,一并核过:run 旧 :41 Dagu 直接 mutation 改引用 `spec/run.md#写入合同`(合同句仍在 `spec/run.md:35`);run 旧 :43「失败类 Run 不终结 Task」收成新 :39 正常完成路径 + 指向 Task 写入合同(合同仍写「失败 / 已取消 / 被替代 Run 不能完成或取消 Task」,`spec/task.md:68` 与 `spec/run.md:105`)。project 交接旧 :74(白名单 C 点名,非关键规则)改引用,权威在 `spec/project.md:99` / 连接合同。

未见把 A 类合同定义句从权威位置删掉,也未见设计层把仅存在于 spec 的排他定义搬走。C2 删的是否定复述,不是合同本体。

**结论:通过。**

## ② 四张角色表删「不能做什么」列:13 格是否都有 spec 或 CT 落点

白名单 §4 规则 3:整列删除;某格在 spec 与 CT 都找不到落点则停、进停车位,不补 spec。GPT 自述「各格均能回到现有 spec 或 CT」。按下表逐格核(旧表行号 = C2 前 main;CT 行号 = 现行 `delivery.md` 契约测试矩阵,C4 拆出前)。

| # | 旧格(文件:行) | 被删内容 | 落点(spec 或 CT 行级,不是族级) |
| --- | --- | --- | --- |
| 1 | project.md:60 Workbench Room | 绕过命令服务直接写治理账本,或把消息/渲染动作当成领域结果 | `spec/system.md:38` Workbench 的 HCTL 功能只依赖 Query/Preview/Submit/Subscribe;`spec/system.md:115` 缓存或界面状态不能反向成为事实;`spec/project.md:99` 普通消息不能改变治理事实;`delivery.md:201` Workbench 不得靠私有导航获得隐藏权限 |
| 2 | project.md:61 CLI | 绕过预览、版本或权限检查 | `spec/system.md:74` 无法提供等价预览、版本或权限时禁用/拒绝;`spec/system.md:109` 没有界面隐藏特权;`delivery.md:36` CLI 没有隐藏权限 |
| 3 | project.md:62 chat server | 从普通消息、反应或自动化自行推断派发/Request;只有另行配置的显式结构化 human 动作可提交命令请求 | `spec/project.md:99` 与 `spec/system.md:92`;`delivery.md:124` 普通消息/反应/自动化不能成为命令,缺项结构化动作同样拒绝 |
| 4 | task.md:56 Workbench Board | 用普通移动启动 Run,或把 Done 显示状态当作完成事实 | `spec/task.md:59` 操作投影不启动 Run、不改 lifecycle;`spec/task.md:20` 拖卡不能直接写成终态;`connections.md:172` Done 不能顺带启动 Run;`delivery.md:137–138` 非法 move/complete 拒绝 |
| 5 | task.md:57 Local/CLI | 绕过命令服务直接改治理账本或任务后端数据库 | `delivery.md:36` CLI 不直接写治理账本或执行面 content 服务器;`spec/task.md:59` 经受控端口写后端并以回读推进;`spec/system.md:166` 降级合同不绕过命令服务 |
| 6 | task.md:58 任务后端 | 接管 Task 身份、契约版本、Run 绑定或语义完成;Done 事件最多只能请求同一套 HCTL 验收 | `spec/README.md:55` provider Done 最多请求同一 Task 验收;`spec/task.md:68`;`spec/system.md:88`;`delivery.md:143–145` Done 缺信封只追加 Snapshot,不伪造 Receipt;`delivery.md:198` 私有对象提升为 HCTL 身份/完成判定时拒绝 |
| 7 | run.md:53 Workbench Run 图 | 直接修改 Engine 或签发结果;没有 CLI 之外的权限 | `spec/run.md:33` Workbench/CLI/provider UI 都不能绕过 command service;`spec/run.md:29` Receipt 只有 reducer 与 control/工具箱可写;`delivery.md:154–155` 已绑定 Engine mutation 只有 control,不倒推 Verdict/Receipt;`delivery.md:201` 经 command service 与 CLI 同语义 |
| 8 | run.md:54 CLI | 绕过绑定、版本或权限 | `spec/run.md:61` Start 必须 CAS Project/version/claim;`spec/system.md:74`;`connections.md:145`;`delivery.md:36` |
| 9 | run.md:55 workflow engine | 选择 Harness、创建 Seat、计算 HCTL Gate、签发 Receipt 或写 Git | `connections.md:172` Engine task 不能直接启动 Harness;`spec/run.md:27` Seat 由 control 铸造;`:29` Receipt 写入者;`:67` 生成物拒绝自行跑 Harness;`delivery.md:260` 不允许 Dagu 自行运行 Harness;`delivery.md:155` 不倒推 Receipt |
| 10 | agent.md:52 Workbench Terminal | 用 UI 状态推进 Task/Run,或把执行投影当作 Room;没有 Herdr TUI 之外的隐藏运行时权限 | `spec/system.md:115` 界面状态不能反向成为事实;`spec/agent.md:88/:90` Execution Chat 不是 Room,输入不自动进 Room;`delivery.md:172` 错误 owner/generation 与无 provenance Share 拒绝;`delivery.md:201` 无隐藏权限;`delivery.md:185` attach 只接通道 |
| 11 | agent.md:53 CLI / 其他终端 | 提交任意 argv/cwd/Herdr terminal ID 绕过 HCTL 适配代码 | 规则本体:`spec/agent.md:86` Attach Descriptor 固定 terminal ID 与代次,适配代码只送仍匹配的获准动作;`:68` 绕过适配代码提交结构化结果仍不被接受;`delivery.md:184` control 签发 descriptor、适配代码校验;`delivery.md:185` attach 不能恢复语义。`argv`/`cwd` 在 spec/CT 无独立句,被「必须走适配代码 + 精确 descriptor」吸收,不单开停车位 |
| 12 | agent.md:54 harness 适配器 | 把厂商 Session 当成 HCTL 身份 | `spec/agent.md:103` 协议会话不是 HCTL 身份;`spec/README.md:133` / `spec/system.md:36` 不得把私有对象提升为 HCTL 对象;`delivery.md:198` |
| 13 | agent.md:55 Agency | 决定领域权限、评审或完成 | `spec/agent.md:68` Agency 合同永不包含治理权威;`spec/agent.md:66` 派出不转移参与者身份;`delivery.md:182` Agency 自带接管/会话有效记录被当作账本事实时拒绝 |

13 格都能在现行 spec 或 `delivery.md` CT 矩阵找到行级落点。没有一格需要按规则 3 进停车位。

**结论:通过。**

## ③ agent.md:23 观测句;§5 五个易混边界是否各自成立

**观测句。** 旧句(C2 前 `agent.md:23`):「观测(进程、屏幕、心跳、钩子)无论多可信都只是观测;它可以触发关注,不能自动变成领域结果。」新句(PR #82 `agent.md:23`):「观测(进程、屏幕、心跳、钩子)用于触发关注;领域结果只经提案与上层准入形成。」

原意是高置信度不能把观测升格为领域结果。新句用排他路径写同一件事:结果只经提案与上层准入,观测不在该路径上,可信度没有第二条入口。同节未改的邻句仍挡「干净退出 ≠ 交付」(新 :24)和「建议不是命令」(新 :26)。A 类仍在 `spec/agent.md:70`:「无论置信度多高都不能自行推进领域结果。」`delivery.md:177` 原生输入不能直接产生领域结果。不是把「只是观测」改成了「观测可以当结果」。

**五个易混边界**(总图 §5 例子;C2 只动设计层,合同层原文仍在):

1. Run 失败类终态不终结 Task vs Reopen/Deleted 只作来源事实——主语仍分开。Task 新 :23 仍写「Reopen/Deleted 第一阶段只作来源事实」;Run 新 :39 只写正常完成路径并指向 Task 写入合同,该权威段仍含「失败 / 已取消 / 被替代 Run 不能完成或取消 Task」(`spec/task.md:68`),Run 侧机制仍在 `spec/run.md:105`;`delivery.md:164` 仍是失败类不终结 Task。没有并成一条「外部终态」规则。
2. 三种排他(ChangeSet lease / Task Run claim / Agency owner)——未并成一条「单写者」。ChangeSet 单写仍在 agent 新 :18;Task「同一 Task 最多一个活动 Run」仍在轻量路径段;Agency owner 范围改由设计地图新 :59 指向 `spec/system.md#单写者`(`spec/system.md:183` 仍是 Agency binding scope 一个 owner lease)。地图新 :59 概括的是 control / 执行现场 / Agency 三处写入者,没有把 ChangeSet lease 或 Task claim 折进去。
3. chat server 不可用 vs 房间事后加密——connections 失败表两行仍在(`connections.md:160` 重同步中、`:161` 需要关注);`spec/project.md:56` 与 `spec/system.md:172` 仍分写两种可观察结果。设计层改为指向 Room 与消息,没有把两行收成一种入口状态。
4. Dagu 直接 mutation 只标分歧 vs Vikunja Done 可成完成请求——裁决仍相反,原因仍各写。Run 新 :37「会先改变机械执行,因此只回读为分歧」;Task 新 :23 / Kanban 段仍接受满足信封的 owner human Done 为完成请求。合同对照:`spec/run.md:35` vs `spec/system.md:88` / `spec/task.md:68`。
5. Harness 可读 common-dir/refs vs 不获交付集成凭据——两句都在 agent 关键规则:新 :21 保留可读 Git / 本分支提交与绕过合入只算漂移;新 :27 保留「集成凭据由工具箱或 adapter 代用」并指向 `spec/agent.md:34`(该段同时写可读 common-dir/refs 与凭据不进工具)。

**结论:通过。**

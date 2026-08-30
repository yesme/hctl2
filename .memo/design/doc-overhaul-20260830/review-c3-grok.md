# C3 合同层:Grok 删除安全审计

> 状态:讨论中 · C3 闸门报告(只列表,不改七个 spec 文件)<br>
> 对象:PR #88 `codex/doc-overhaul-c3-contracts` @ `659d35b`(七件 `docs/design/spec/**`;未改 delivery / 设计层 / 五张历史表)<br>
> 对照:[施工图](./README.md) §2 红线、[`02-target-map.md`](./02-target-map.md) §4/§5、[`04-prohibition-whitelist.md`](./04-prohibition-whitelist.md)、[`01-inventory-dead-text.md`](./01-inventory-dead-text.md) 最危险三条、[`06-c3-card.md`](./06-c3-card.md) 审计卡<br>
> 去向:所有者裁决后交 GPT 一次修正,推回 PR #88;本文件不进合同<br>
> 结论取值只能是:通过 / 回滚哪几句 / 进停车位

旧来源行号 = 审计时 `origin/main`(`ed27de5`);新位置 = PR #88 head。

## ① 四组同构:权威是否仍在指定位置;改引用是否丢掉限定条件

**§4.1 加密前置。** 权威 `spec/project.md`「Room 与消息」(新旧皆 :56)未改:第二合同前提、fresh 正文可读、不可用/事后加密的可继续与拒绝、重同步中 vs 需要关注、换绑恢复、已冻结引用不受影响。对象表改引用(新 :14);写入合同仍留「以 fresh 房间状态回读证明目标房间未启用端到端加密」(新 :29,旧 :29)。connections 失败表两行都在:「chat server 不可用」仍写重同步中(新 :160);「已绑定房间被开启端到端加密」仍写需要关注,可继续/拒绝与换绑改指向 Room 与消息(新 :161,旧 :161)。写入合同终态里「不改写历史 binding」改引用后,同句仍在 `spec/system.md:51`(旧 :55)。

**§4.2 三条底线。** 权威 `spec/agent.md:34` 完整保留,含「在此之内 Harness 是普通 Git 用户」、凭据不进工具、独立 worktree 与可选加固;仅把「按 Herdr 核验」改成「按所选 Agency 核验」,安全输入改为「声明并实现才启用」。system 命令段保留两类 actor 来源(新 :101,旧 :109),并按 A5 删了「第一阶段不设额外的用户在场证明」。外部副作用缩为引用 Agent 写入合同(新 :111);安全边界保留「未启用加固时 Harness 与同 OS 用户的其他进程处于同一信任域」(新 :197)。

**§4.3 能力条件句。** 权威从旧 :74 的 v0.8.2 缺项清单改成四项「声明则如何、未声明则低信任降级」(新 :74);:68 栅栏回显同样改成未声明只在 HCTL 入口校验。详见 ③。

**§4.4 两来源。** 权威 `spec/task.md:68` 未改。旧 :76 裸终态反例按 B 类删除,正确道路已在 :68。Run→Task(新 :105)保留 `completion_pending` 与「失败 / 已取消 / 被替代 Run…不提交完成或取消 Task」,终结来源改引用 Task 写入合同。connections :119 来源复述改引用,仍写不生成 Run 专属 Gate Receipt。system 改引用 Task 写入合同。

被改成引用的其它位置:connections 旧 :61 Manifest 散文改引用 Run 清单(见 ④);system 四条 provider 路径改引用 connections 新 :172。connections 旧 :172 的「Done 不能顺带启动 Run / Engine task 不能直接启动 Harness / 终端不能直接 Complete Task」收成「适配器只使用目标模块已有的连接」加四条接纳路径——正确道路已排除捷径,不另回滚。

**结论:通过。**

## ② 白名单 A 类 15 处;被删 B 类判据;必留 31 条是否留了一句

点名 A 类对照:

| 点名 | 旧 | 新 | 判定 |
| --- | --- | --- | --- |
| agent:34 三条底线 | :34 | :34 | 留;措辞按拍板点 11 去 Herdr 点名 |
| agent:38 ChangeSet 单写租约 | :38 | :38 | 原样留 |
| run:37 正常完成谓词 | :37 | :37 | 原样留 |
| run:39 失败隔离/不并发第二 writer | :39 | :39 | 原样留 |
| task:64 Run claim | :64 | :64 | 原样留 |
| task:66 完成命令 fail-closed | :66 | :66 | 原样留 |
| task:68 两个终结来源 | :68 | :68 | 原样留 |
| system:109 两类 actor | :109 | :101 | 两类来源留;在场证明半句按计划删。**同段「都不能自报为 human 或 workflow reducer」(旧 :107)改成「保留实际来源」(新 :101)**——主语还是 payload/Room/Harness/adapter,条件从「不能自报」变成「保留实际来源」,可读成保留 payload 自报的来源。白名单 §4 规则 4:A 类改正面句不得改条件。前半「只由 direct client / binding / reducer 赋予」是正确正面句,后半把禁令写反了 |
| system:111 ACK 未知不盲重投 | :111 | :103 | 「结果未知,不能盲目重做」留 |
| system:179–185 单写者 | :179/:181/:183 | :167/:169 | control writer、site fence、Agency owner lease 都在;Herdr 点名改成「不接收也不回显 generation 的 Agency」条件句 |
| connections:10 不能直接写目标 | :10 | :10 | 原样留 |
| connections:16 精确引用 | :16 | :16 | 原样留 |
| connections:115 旧代次只留审计 | :115 | :115 | 原样留 |
| connections:168 对账前不表现为已交接、不复活旧 owner | :168 | :168 | 原样留 |

被删 B 类均可指认:task 旧 :76 裸终态清单 = 白名单 §1 B 举例,正确道路 :68 已排除;project 旧 :54 末句「普通回复、表情或模型总结不会修改 Project」同;system 旧 :109 在场证明 = A5 / 白名单 B。未把未决补进 spec,无新停车位。

必留 tricky 6 条在 spec 的 5 条仍在(connections :115/:168、system :103、task :74 不复活旧 Receipt、project :79 不复活旧调用);delivery:81 本簇未碰。特别容易犯留下的 spec 句仍在:mention 模糊匹配(project :97)、Dagu 直接 mutation(run 新 :35 仍写已绑定 mutation 只标分歧)、权限逐级缩小(connections :147)、模型自述已合并(agent :54)、来源不能直接写目标(connections :10)等。转走的 agent.md:27 / delivery.md:36 不在本簇。

**结论:回滚** `docs/design/spec/system.md:101`(PR #88)分号后「调用 payload、Room 消息、Harness 进程和 adapter 保留实际来源」,恢复旧 `spec/system.md:107`「都不能自报为 human 或 workflow reducer」。前半「只由…赋予」可留。

## ③ Herdr 能力条件句;无则降级与不另写终端服务;v0.8.2 缺项是否已离开 spec

四项在新 `spec/agent.md:74` 均写成「声明则如何、未声明则低信任降级」:

- 栅栏回显:声明则回显代次与租约并拒绝不匹配;未声明只在 HCTL 入口校验,物理 fence 记为未生效(:68 同)。
- 逐次输入记录:声明则每次输入关联 actor/lease/generation;未声明则来源不完整,物理单写者与完整 replay 不可用。
- 事件游标:声明则带序号并报告缺口;未声明只作有界观测,不能表示完整 trace。
- 退出与停止回读:声明则回报同一进程/PTY、退出码与 stop 证据;未声明或证据不足只能 semantic resume / replay / 丢失。

`managed_single_writer` 仍要求 provider 不能统一拦截时关闭原生 controller(新 :72)。system 单写者(新 :169)与 connections 启动第 4 步(新 :99)改为「未声明栅栏回显则只在 HCTL 入口」。system 新 :36 仍留「受 HCTL 单输入租约管理的 Terminal 输入必须先由 Herdr 适配代码校验精确票据、租约和当前代次」(施工卡要求保留的 HCTL 侧规则)。

「按实测降级、禁止另写终端服务」:agent 新 :66 仍写「HCTL 不再放置独立 Agency 组件或下一层终端运行服务」;delivery P0 第 2 项(main `delivery.md:261`)仍是 v0.8.2 缺项清单的家(512 条 ring、交错写入、原生输入不经租约)。新 spec 全文已无 `v0.8.2` 字符串。

**结论:通过。**

## ④ Manifest 并集

旧 `spec/run.md:54` 有放置、无端口绑定/网络 secret;旧 `connections.md:61` 有端口绑定、网络/secret 范围,无放置。新 `spec/run.md:54` 为「受控端口绑定、获准 Worker Profile 候选、切换规则、能力、权限与网络/secret 范围」;:55 仍为「Gate、预算、放置和截止规则」。新 `connections.md:61` 改为引用 `[Run 合同](./run.md#workflow-与-run-授权)`,不再自列一份。不是只删一份。

**结论:通过。**

## ⑤ §5 五个易混边界与三种排他

1. 失败类 Run 不终结 Task vs Reopen/Deleted 只作来源事实——主语仍分开。Run 新 :105「失败 / 已取消 / 被替代 Run…不提交完成或取消 Task」;task 新 :48「Done/已关闭/Reopen/Deleted 是 content 事实,不会自动完成、重开、取消 HCTL Task」, :68「失败类 Run 不能完成或取消 Task」且取消只接受 human。
2. 三种排他未并成一条单写者。ChangeSet 至多一个活跃 lease:agent 新 :26/:38。Task 至多一个 Run claim:task 新 :64。Agency binding scope 一个 owner lease:system 新 :169。对象与代次仍分。
3. chat server 不可用 vs 事后加密——connections 新 :160/:161 两行都在,可观察结果仍是重同步中 vs 需要关注;`spec/system.md:160` 丢失表同样分写。
4. Dagu 直接 mutation 只标分歧 vs Vikunja Done 可成完成请求——裁决仍相反。run 新 :35 已绑定 Dagu mutation 只标分歧,原因在 :33 的先记账后推引擎;system 新 :84 Vikunja Done 在信封齐全时可归一为完成请求,缺项只是 Snapshot。connections 新 :172 把两条并列,没有抽成一条通用 provider 规则。
5. Harness 可读 common-dir/refs vs 不获集成凭据——两句都在 agent 新 :34(普通 Git 用户可读 common-dir/refs 并在本 ChangeSet 分支提交;不向 Harness 交付集成与外部写凭据)。

**结论:通过。**

## 所有者裁决(2026-08-31,「按此修」)

1. 接受 ② 的回滚:`spec/system.md` 恢复「调用 payload、Room 消息、Harness 进程和 adapter 都不能自报为 human 或 workflow reducer」;前半「只由…赋予」保留。
2. 追加(Fable 核 diff 发现):恢复同段被顺带删除的「Harness、模型与 execution principal 只有 Result Proposal 通道,不能借 provider service account 或 payload 自报为 human」——命令入口处「工具不是人」的正面锚,删除超出白名单授权。
3. 其余 ①③④⑤ 通过,无修改。GPT 在 PR #88 上一次修正后合入;本簇第二轮即最后一轮。

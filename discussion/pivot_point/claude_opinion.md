# HCTL2 的方向取舍：工作台、治理内核与 Room

Claude · 2026-11-23 · 产品讨论意见，供项目所有者决定下一轮投入。

**我建议不按当前范围继续，改为一个有期限的收窄回合：只留治理内核，把执行面和界面交给 T3 Code 或 Orca，把讨论的源留在团队已有的平台上。** 回合结束时，用真实变更的数据决定去留。

本文记录与所有者的两轮讨论。第一轮比较 HCTL2 与 T3 Code、Orca，回答还要不要继续。第二轮评价「功能澄清在聊天室里决定」这个假设，以及它与两家的方法是否契合。两轮结论都在读到 [Codex 的意见](./codex_opinion.md)之前形成，与它的异同单列在最后一节。

## 三家现在的样子

下表对照三家在七个问题上的做法。T3 Code 与 Orca 两列取自 11 月 23 日（北京时间）读到的源码与文档，提交号和出处见[实现层脚注](#实现层脚注依据与口径)。

| 问题 | T3 Code | Orca | HCTL2 |
| --- | --- | --- | --- |
| 组织单元 | Thread：一个人与一个代理的一段对话 | Worktree，加上驻在 Slack、Linear、GitHub 里的 bot | Project → Task 契约 → Run |
| 谁决定下一步 | 模型，经 MCP 的 `delegate_task` 跨家派活 | 协调者模型决定拆分、落点与并发 | 控制面按已记录的事实判定 |
| 「完成」指什么 | 一轮对话结束，PR 已关联 | worker 报 `worker_done`，PR 开出 | Git 回读后签发合入凭证与完成凭证 |
| 澄清发生在哪 | Thread 里，讨论与施工是同一段对话 | 团队已有的 Slack 讨论串、Linear issue、GitHub PR | 自己托管的 Room，多个模型与人同处一条时间线 |
| 第二个人怎么进来 | 访客链接：限一人、24 小时，只能看和提问，每条消息由主人放行 | 队友在原处提问，bot 落款「asked by X via bot」，批准与合入只听主人 | 设计上多方，当前范围不做多用户组织与 RBAC |
| 连续性挂在谁身上 | Thread | bot，有自己的 Home Chat 与记忆 | Project |
| 界面 | 桌面、Web、iOS、Android、远程 | 桌面、手机、SSH 主机、云主机 | 命令行；Workbench 尚未实现 |

## 第一轮：还要不要继续

**按当前范围继续不值得；差异点本身值得用一个收窄的回合检验。**

### 相同与不同

- **相同。** 三家都支持多个 Harness，都是本地优先、自带订阅、工作树隔离、走 PR。T3 Code 的 README 自称「The open-source control plane for coding agents」，与 HCTL2 的「控制面」是同一个词。
- **底座相近。** T3 Code 的编排 v2 同样建在事件溯源、命令幂等回执与 outbox 上。
- **不同在中段判断由谁做。** T3 Code 的 receipt 是命令幂等回执，评审轮次由发起委派的模型组织。Orca 的 worker 自报 `worker_done` 即结算。两家都没有用代码判定「验收过了没有、评审可不可以跳」。[治理功能流失调研](../../docs/research/governance-attrition-20260908.md#一句话结论)在 9 月的结论是 33 家里没有一家用代码守住中段判断；Orca 在那 33 家里，T3 Code 不在，按这两个提交看，两家都是同一个形状。

### 不该照原样继续的四个理由

1. **工作台这一层的胜负已经分出。** 11 月 8 日起的两周里，T3 Code 合入 3,318 个提交，Orca 合入 3,820 个；HCTL2 的 Workbench 尚未实现。第三方报道称 Conductor 已被 Meta 收购，这个品类在收拢。
2. **「没人做」也可能是「没人要」。** 同一份调研写明，这类门的死因是「从不触发」加成本。Orca 退役了机械调度，改由模型决定拆分与并发；领头者是朝反方向走的。
3. **核心假设十周后仍未经真实开发检验。** 所有者在 [10 月 3 日的记录](../../.memo/log/2026-10-03-演示线计划.md)里写的是「目前还是我是central bus」。[演示 3](../../.memo/log/2026-10-20-演示3验收-ubuntu.md) 走通了从聊天到合入的全链路，但改动是一行 `return value * 2` 和 canary 仓库里的两个新文件，不是 HCTL2 自身的开发。
4. **投入倒挂。** `src/` 下有 11.5 万行 Rust，有差异的只是治理内核。打包五个随包服务、三平台构建、Agency 与各家 Harness 适配，正是两家已经做成现成件的部分。

### 建议

- **砍：** Workbench、随包的 Matrix、Vikunja 与 Dagu、自建 Agency 与各家 Harness 适配、第二期跨机连接。
- **留：** Task 契约冻结、变更封存、`hctl2-tool wait` 回读、合入凭证与完成凭证、合入钥匙不交给模型。
- **换载体：** 做成 MCP server 或命令行，加一个 GitHub 必需检查，让 T3 Code 或 Orca 当执行面与界面。这与[设计原则 14](../../docs/design/vision.md#设计原则)「复用实现，不继承边界」一致。
- **设止损：** 用它治理连续 5 个真实变更，全程不人肉转发 Prompt，这是[自举阶段](../../docs/design/delivery.md#自举阶段) B3 的验收。随后回答交付文档自己列的三个问题：比直接操作 Harness 省事吗；同等验收要求下省 token 吗；出了问题之后更容易继续吗。
- **答案为否就停。** 停下后把 33 家治理功能流失的调研整理发布。我认为它是这个仓库目前对外最有价值的部分。
- **只为自用时门槛更低。** 它比 `./run` 脚本加 T3 Code 更省事，就值得留作自用工具。

## 第二轮：以聊天室为核心这个假设

**这个假设对了一半。「澄清是对话式的，要有持久、多方、可回源的地方，最后结晶成契约」成立，两家都没有做到；「所以产品要以自己托管的聊天室为核心」不成立。**

### 与两家是否契合

三家都以对话为入口，但对话放在哪里各不相同，对照见[第一张表](#三家现在的样子)里从「澄清发生在哪」起的三行。

- **T3 Code 不契合。** 它是单人模型，对话可以分叉、可以回滚。这正是[设计原则 19](../../docs/design/vision.md#设计原则) 描述的「一个人对着模型」的情形；HCTL2 的 Room 假设有第二个观察者，所以同一间 Room 不做分支。
- **Orca 结论相近、方法相反。** 它也认为澄清发生在团队聊天里，做法是让 bot 去团队已经在的地方，而不是请团队来新的聊天室。它还把连续性放在 bot 身上，这是[愿景](../../docs/design/vision.md#一句话定位)明确不作为组织中心的那条轴。

### 不认同「以自己托管的聊天室为核心」的四个理由

1. **团队已经有聊天室。** 让团队把讨论搬进新的 Matrix 房间，是采纳成本最高的一步。HCTL2 接入其他平台靠 Matrix 桥接生态，自建桥接在[明确不做](../../docs/design/delivery.md#明确不做)里。我判断这条路径的采纳阻力大于 bot 直接进 Slack；这一点没有实测。
2. **聊天室只解决团队协作里容易的部分。** 共享时间线与成员制由 Matrix 提供。难的是三件事：谁有权采纳契约，谁有权批准合入，一次调用花谁的订阅。
3. **两家都卡在这三件事上。** T3 Code 的访客消息要主人放行，Orca 的 bot 以主人身份行事，两家都把权限收到一个人身上。HCTL2 的设计在这里走得更远：[临场执行边](../../docs/design/project.md#room-场景)必须归属到具体的人，并先过 Trigger Preview；合入钥匙在系统手里。解锁团队场景的是这套归属与准入规则，不是聊天室。
4. **HCTL2 自己的开发过程没有用 Room 塑形。** 10 月 3 日的记录里，塑形发生在所有者与主笔 Harness 的一对一对话中，多方评审发生在 GitHub PR 评论里（所有者：「大家都会监控PR，有comments了可以自己看」）。实际承担多方房间职责的是 PR 讨论串。我只读了日志，演示 1 之后是否改变没有核对。

### 建议

把看板的裁定用到 Room 上。所有者 10 月 3 日定过「要区分source和UI」：任务卡的源在 issue，HCTL 是又一个客户端。聊天是目前唯一由 HCTL2 托管源的场景。

- **留：** 结晶这一步。讨论按消息 ID 冻结引用，前情提要由人确认，提案经采纳命令才成为契约（见 [Project 与 Room 的关键规则](../../docs/design/project.md#关键规则)）。两家都没有这一步。
- **放：** 讨论的源。GitHub issue、Slack 讨论串、Linear issue 都具备线性时间线、稳定消息 ID 与成员身份。
- **代价：** 在外部平台读消息受限流与权限约束；Trigger Preview 需要另找入口承载。

这与第一轮的结论一致：值得继续的是治理内核，Room 里有价值的那部分属于内核。

## 实现层脚注：依据与口径

本节只列事实与出处，不改变前两节的判断层级。

**规模。** star 来自 11 月 23 日的页面快照；提交数是 11 月 8 日起两周内主分支的提交，作者按不同署名计。

| 项目 | star | 两周提交 | 两周作者 | 读到的提交 |
| --- | --- | --- | --- | --- |
| T3 Code | 约 71.2k | 3,318 | 538 | [`7ddc81d`](https://github.com/pingdotgg/t3code/tree/7ddc81d639aa75ca6e0cbffb0f394ea3f0d83308) |
| Orca | 约 132.7k | 3,820 | 175 | [`052d0af`](https://github.com/stablyai/orca/tree/052d0af1c67ca03f44e739c8e4a2df4ddd5c0763) |
| HCTL2 | 0 | 0 | 0 | `86fd366`，写本文时 main 在 `ba8354f` |

**T3 Code。**

- 自我定位、支持的 Harness、三端与工作树：[README](https://github.com/pingdotgg/t3code/blob/7ddc81d639aa75ca6e0cbffb0f394ea3f0d83308/README.md)。
- Thread、Run、执行节点的模型，分叉与回滚，命令回执与 outbox：[编排 v2 总览](https://github.com/pingdotgg/t3code/blob/7ddc81d639aa75ca6e0cbffb0f394ea3f0d83308/docs/orchestration-v2/README.md)。
- `delegate_task` 与评审轮次的组织方式：[编排 MCP 服务](https://github.com/pingdotgg/t3code/blob/7ddc81d639aa75ca6e0cbffb0f394ea3f0d83308/docs/orchestration-v2/orchestrator-mcp-server.md)。
- 「Command receipt」的定义是让重试幂等的命令结果记录：[术语表](https://github.com/pingdotgg/t3code/blob/7ddc81d639aa75ca6e0cbffb0f394ea3f0d83308/docs/internals/glossary.md)。
- 访客链接的范围与放行规则：[Guest links](https://github.com/pingdotgg/t3code/blob/7ddc81d639aa75ca6e0cbffb0f394ea3f0d83308/docs/user/guest-links.md)。
- 在 `apps/server/src` 与 `packages` 下检索 quorum、acceptance criteria、review gate，均无命中；kanban 只命中 GitHub 的 GraphQL schema。

**Orca。**

- 自我定位与 bot 的入口：[README](https://github.com/stablyai/orca/blob/052d0af1c67ca03f44e739c8e4a2df4ddd5c0763/README.md)。
- 协调者怎样起 worker、选机器、`worker_done` 怎样结算：[编排指南](https://github.com/stablyai/orca/blob/052d0af1c67ca03f44e739c8e4a2df4ddd5c0763/skill-guides/orchestration.md)。
- bot 以主人身份行事、落款规则、批准与合入的限制、Home Chat：[bot 命令参考](https://github.com/stablyai/orca/blob/052d0af1c67ca03f44e739c8e4a2df4ddd5c0763/skill-guides/orca-cli/references/bots.md)。
- 机械调度退役的经过，取自本仓库的 [Orca 审计](../../docs/research/workbench/stably-orca.md#e-l2-stably-orca)与治理流失调研，我没有重新核对那个 PR。

**HCTL2。**

- main 上有 1,488 个提交（含合并提交），起于 8 月 14 日。10 月 22 日的 #506 到 11 月 23 日的 #508 之间，main 没有提交。
- `src/` 下 Rust 共 115,166 行，其中路径含 `tests` 的 54,098 行；`docs/` 与 `.memo/` 的 Markdown 共 57,208 行。数字在 `86fd366` 上量得，#508 只新增了讨论文档。
- Workbench 的状态取自[使用说明的入口一览](../../docs/usage.md#当前入口一览)。

**口径与局限。**

- 我没有试用 T3 Code 与 Orca 的产品，只读了源码与文档。
- HCTL2 只看了 main；9 个未合并 PR 里的工作没有计入。
- Conductor 被收购一条来自[第三方报道](https://flaviocopes.com/conductor-alternatives/)，没有向一手来源核对。

## 与 Codex 意见的对照

本节写于读过 [codex_opinion.md](./codex_opinion.md) 之后，前面各节没有据此改动。

- **一致。** 两份意见都认为不该与两家比通用工作台，都把「讨论结晶成有来由的契约」算作 HCTL2 的差异点，都认为团队愿不愿意另开一个聊天室要单独验证，也都把收窄成命令行或 GitHub Check 列为一条出路。
- **分歧。** Codex 建议继续，把 Room 当作需求澄清的主要场所去验证，收窄是试用结果不支持 Room 时的退路。我建议从收窄出发，先放开讨论的源。
- **同一次试验可以裁决这个分歧。** 在 Codex 设计的四周外部试用里加一组：讨论留在 GitHub issue 或 Slack，只用 HCTL2 做采纳与验收。两组比接手者的追问次数，就能回答 Room 是否必须由 HCTL2 托管。我的止损标准只用自用数据，测不到「没参加讨论的人能否接手」；在这一点上，Codex 的试验设计更合适。
- **我没有核对的事实。** Codex 引用了 #309、#318（Room 可用性修复）与 #462、#492（计票与 Run 冻结）。我对实现进度的判断只依据 README、使用说明和两份日志。

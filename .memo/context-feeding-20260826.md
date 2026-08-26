# Context 讨论记录：投喂三档、Run 内接力与任务评论线

> 日期：2026-08-26<br>
> 状态：Informative 讨论记录。裁决已进入 [Context 横切正文](../docs/design/context.md)与各模块合同（基线 v0.13.1），来时路见 [decision-history §26](../docs/design/references/decision-history.md#26-context-投喂三档与-run-内接力v0131)。本文只保留推导过程、被否掉的切法和一个待拍板项。<br>
> 前序底稿：`context-design-20260819.md`（合同底稿）、`context-landscape-20260824.md`（生态四族）。

## 1. 起点：所有者的两类判定与一个问句

文档在本轮之前对 Context 的判定有两类：① 聊天室的历史（萃取）；② 父子概念间的传承（Repo Room → Project → Task/Run → 执行者包）。所有者的问句是：有没有漏？比如从 Task 的讨论里继承内容？本质上是不是"从四个依赖服务器（Matrix、Vikunja、Dagu、tmux）的存储里抽取内容"的过程——Terminal 主要靠 harness 之间显式的 git/memo，Kanban 和 Workflow 的存储与 metadata 怎么算？

## 2. 换刀：不按服务器切，按"执行体够不够得着"切

`context.md` 原本就埋了这条规则——开工包只物化执行体拿不到或不该自己翻的东西，其余给指针。用它把四个场景的存储一分：

| 场景存储 | 存了什么 | 执行体自己够得着吗 | 进开工包的方式 |
| --- | --- | --- | --- |
| Chat Room（Matrix） | 讨论 | 不——聊天凭据不交付 | 萃取后物化 |
| Kanban（Vikunja / Linear / GitHub） | 卡片、评论线、附件、流转 | 不——任务后端凭据不交付 | 契约进骨架（Task Revision）；评论线萃取后物化 |
| Workflow（Dagu） | 路标 | 无关 | 什么都不取；Run 内前序节点的结果在账本和 Git |
| Terminal（tmux） | 会话 trace | 自己的够得着（semantic resume），别人的不 | 只经结晶：ChangeSet 提交、Memo、Proposal 引用的 Evidence |

四台服务器里只有两台的内容需要萃取（chat、task backend），一台给指针（Git 结晶），一台空（Dagu）。所以"四个依赖服务器"不是好的切法；可达性才是。

## 3. 漏的一类：同一 Run 内的接力

纵向传承（父 → 子）之外，横向的没写：producer → reviewer → 返工之间传什么、谁装包。它的存储是账本 + Git，不是 Dagu：

- reviewer 要 producer 的 ChangeSet Revision——`spec/run.md` 已有 ReviewSubjectRef，在 Git 里；
- 返工 seat 要 reviewer 的 Verdict 正文——权威在账本，执行体没有 control 客户端凭据。

复核时发现 `spec/run.md:39` 与 `spec/system.md` "Git 的双重角色"早已规定 Verdict/Receipt 有 Git 结晶副本，所以"这一步断了"的初判不成立；但副本粒度按仓库策略可降为仅摘要（公开仓库），所以合同最终写成：Verdict 以账本记录物化（必用、内联），Git 副本只作指针。

## 4. 少的一个来源：绑定 Task 的评论线

"从 task 的讨论里继承"原本只有一半：改契约的部分经「采纳契约」进 Task Revision；不改契约的讨论（澄清、"先试 X"、后端是 GitHub/Linear 时 HCTL 之外的人写的评论）只落 Snapshot，执行体看不见。它和聊天史同性质——讨论、执行体自己翻不到——所以走同一条萃取阶梯；而且整条结构相关、第一级命中，不需要检索，以 Snapshot ref+digest 冻结。

Kanban 与 Workflow 的其余 metadata（stage、优先级、负责人、排序）不进 context——那是操作投影；依赖/阻塞关系只以相关 Task Revision 指针进骨架。

Terminal 按所有者判断写死：harness 之间只有结晶过的东西能过去；没提交的对继任者不存在（与替代执行换新 worktree 同一条规则）。

## 5. "指针"是什么：三档投喂

所有者追问：LLM 没法实时通过指针拿数据。答：指针是写进 prompt 的精确地址（ref + digest），由**执行体的工具**去取——harness 是模型加工具循环，`git show`、读文件是它的普通工具调用，和 CLAUDE.md 里写 `@file` 一回事。由此得出三档：

| 档 | 给什么 | 怎么给 |
| --- | --- | --- |
| 内联 | 每次必用、放得下 | 原文进 prompt |
| 指针 | 可能用、执行体够得着 | 地址 + digest + 一句摘要 |
| 代取 | 执行体够不着 | 开工时萃取物化；运行中走受限召回（模型提请求 → control 校验 recall policy → HCTL 检索器取 → 子包塞回） |

硬边界：指针只能指向 Git 对象和 worktree 路径；指向账本或任务后端的引用不是指针。

## 6. 为什么同类工具 token 多、速度慢

所有者假设："给太多指针，harness 反复通过模型读文本再 tool call"。一半对。按调研库审过的样本，烧钱的原因有四个：

1. **内联太多**（与指针相反的错误）：LobeHub 默认工具 schema + 系统提示约 82KB/1.9 万 token 固定前缀每次重发（#13797 "发 hello 烧 8000+ token"）。
2. **每轮重读整个上下文，拉回来的东西永久驻留**：这才是指针的真实成本——不是几百 token，是多一轮往返 = 全上下文重读 + 一段永久驻留的工具输出 + 一次串行延迟。前缀缓存能压到十分之一，前提是前缀稳定；把时间戳、动态工具列表放开头的框架每轮击穿缓存。
3. **该用代码的地方用了模型**：LobeHub 群聊 N+2 次全量历史调用；mem0 每条消息过 LLM；Cumora 用 LLM 复核 LLM。串行模型调用是"慢"的主因。
4. **多 agent 扇出把前三条相乘。**

结论：**贵的不是指针，是那一轮**；"全内联"更贵。分水岭是指针的质量——40 个"你可能想看看"的文件路径诱发探索式阅读，3 个精确到 hunk、带一句话的地址让模型能跳过。推论：必用且放得下 → 内联，不给指针（reviewer 的 diff 该内联，"够得着"只是指针的前提不是理由）；可能用 → 指针 + 摘要；探索式阅读是 harness 自己的子上下文的事，我们不逼它探索。省 token 的核心是**少一轮**，即"能用规则就不用模型"从检索侧推到投喂侧。

## 7. 落点

- `docs/design/context.md`：新增「投喂三档」「同一 Run 内的接力」两节；萃取来源扩为聊天史 + 绑定 Task 评论线；关键规则加两条；分工表加两行。
- `docs/design/spec/project.md`：Bundle 条目按 inline / pointer / recall 记录；必用条目与超预算处理；pointer 可达范围；评论线为第一级来源。
- `docs/design/spec/run.md`：Attempt Bundle 装入前序节点结果的规则；备用 Attempt 与未提交内容。
- `docs/design/spec/task.md`：评论线快照冻结进 Manifest，不经采纳不进契约。
- `task.md` / `run.md` / `agent.md` 设计正文各一句。
- 基线 v0.13.0 → v0.13.1；`decision-history.md` §26。

## 8. 待拍板：执行中产生的经验怎么沉淀

所有者随后的问句：执行过程中可能有经验值得以 Skill（或别的形式）保存下来，这算不算 Context？能不能自动做，又不让复杂度爆炸？

这不是 Context（Context 是进去的方向），是结晶（出来的方向），归 Memo 与 Skill 的晋升。现有合同已经有正面路径：Memo 必须显式提炼、预览、去敏、发布，带来源、适用范围和有效期；Skill 是 Repo 共享的 Git revision，改它就是改代码。调研库里的反面也齐：Cumora 记忆无版本且有自我中毒实证（agent 把特例写成普适规则、写备忘录训练未来的自己无视安全网）；Letta 式 agent 自编辑记忆 = 自述；claude-mem 自动注入"最近十次"与显式来源链相悖；First Tree 的 Decision Test + Durability Test + 有来源支撑的评审写入是正例，已改编为 Memo 晋升门槛。

建议的形状——**自动化提案，绝不自动化晋升**，零新对象：

1. **提案自动**：Result Proposal 的 output schema 加一个可选项 `lesson`（一句经验 + 适用范围 + 必填来源 refs：Run/Attempt/ChangeSet/消息事件）。执行体本来就要写总结，多写一项几乎免费；它是自述，所以只是候选。没有来源 refs 的项机械丢弃——这是 First Tree "没有来源材料就什么都不写"的机械版。
2. **登记自动**：control 把 lesson 作为 Evidence 挂在 Run 上，Room 里投影一张"本次执行提了 N 条经验"的卡；可选 small-brain 做与既有 Memo/Skill 的近重复比对并建议 supersedes——建议而已，无晋升权。
3. **晋升不自动**，且只有两条既有路：知识 → 人走 Memo 发布（预览、去敏、来源、范围、有效期）；做法 → 作为一次改 `.hctl/skills/` 的 ChangeSet，和代码一样过 Gate 合入。执行体可以在本 ChangeSet 分支上直接提这个改动，但它就是普通代码变更，不是记忆写入。

复杂度守门：不建记忆库，不自动注入历史经验（下次开工看到的仍只是发布过的 Memo/Skill 指针），不让执行体改目标分支上的 SKILL.md。唯一新增是一个 output schema key 和一个投影。是否采纳、`lesson` 是否进第一阶段 output schema，待所有者拍板；拍板后落点是 Agent 合同的 Result Proposal 输出项与 Project 合同的 Memo 段。

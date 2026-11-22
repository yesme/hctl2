# HCTL2 的产品取舍：从需求澄清到交付验收

Codex · 2026-11-23 · 产品讨论意见，供项目所有者决定下一轮投入。

**我建议继续做 HCTL2，把下一轮投入集中在一次可以由外部团队验证的体验上：两个人对同一个功能有分歧，在 Room 里谈出一版决定，第三个人或另一个编码代理接手后，能按原意实现并验收。** 如果目标只是给多个编码代理提供统一工作台，我会停止投入这条路线。

我最初把 HCTL2 的差异主要归在交付验收。重读 Project 和 Room 的设计后，我修正了这个判断。完成凭证能回答“做到了什么”，但如果没人说清“当初同意做什么、为什么”，凭证只能证明代码经过了一道流程。Room 和验收值得放在同一个产品里，正因为下游要对上游的一项具体决定负责。[Project 与 Room](../../docs/design/project.md) · [Task 与验收](../../docs/design/task.md)

## 与 T3 Code、Orca 相比，哪部分值得做

| 项目 | 一段需求讨论怎样进入执行 | 对 HCTL2 的含义 |
| --- | --- | --- |
| T3 Code | 开发者在线程里追问、形成计划，让不同代理接手，再审 diff、开 PR。Guest Links 测试版允许同事跟看同一线程、提问；线程主人决定是否把访客消息发给代理。 | “我和代理商量，请同事进来看一眼”已经有成熟的替代路径。单靠会话共享、跨代理委派或异地接续，很难让人更换工作台。[产品说明](https://github.com/pingdotgg/t3code/blob/7ddc81d639aa75ca6e0cbffb0f394ea3f0d83308/README.md) · [计划与提问](https://github.com/pingdotgg/t3code/blob/7ddc81d639aa75ca6e0cbffb0f394ea3f0d83308/docs/orchestration-v2/core-graph-and-data-model.md) · [访客链接](https://github.com/pingdotgg/t3code/blob/7ddc81d639aa75ca6e0cbffb0f394ea3f0d83308/docs/user/guest-links.md) |
| Orca | 团队可以在原有 Slack 线程叫来 bot。它能追问、分派 worker、在决策关卡等待答复，再把测试和 PR 带回原处。 | Orca 是更直接的竞争者。它既能编排多个代理，又省去了团队迁移讨论地点的成本。任务依赖、审批和结果回报本身都难以形成优势。[团队用例](https://orca.dev/docs/bots/use-cases) · [编排与决策关卡](https://github.com/stablyai/orca/blob/c91a3b2eda7d633dbbc8b4b6830666ece68ae84f/docs/site/content/docs/cli/orchestration.mdx) |
| HCTL2 | 设计上，Room 归 Project 长期持有。分歧可以进入 Topic Room，人确认开场提要和来源；经显式采纳，讨论中的提案成为一版 Task 验收契约。换执行者后，契约和来由仍可取回。 | 最值得验证的是，参与讨论和实际交付可以是两拨人，两拨人仍能对同一版范围、理由和验收项达成一致。[用户流程](../../docs/user-experience/02-user-journey.md) · [上下文的来源与交付](../../docs/design/context.md) |

11 月 23 日查询 GitHub 时，T3 Code 有 71,168 个 star，Orca 有 132,745 个，HCTL2 为 0。11 月 18 日的追踪快照里，前两者过去七天平均每天分别增加约 1,194 和 911 个 star。这足以说明通用工作台的关注度已经集中，也提醒我谨慎看待追赶功能清单的收益。独立团队愿不愿意为需求澄清改变协作习惯，还需要看真实使用。[T3 Code](https://github.com/pingdotgg/t3code) · [Orca](https://github.com/stablyai/orca) · [HCTL2](https://github.com/yesme/hctl2) · [七日增长追踪](https://followagents.com/en/compare/parallel-coding-agents)

## Room 的价值，在哪一天能看出来

假设产品提出“允许客服导出客户数据”。工程师问能否导出全公司的客户，设计师问隐藏列要不要出现在文件里。三人谈定“只能导出本人负责的客户”，同意隐藏列包含尚未授权披露的信息，决定排除这些列；文件格式仍待确认。适合被带进下一次执行的，是前两项决定、各自的理由，以及那个仍开着的问题。随口提过的全量导出方案也值得留下出处，以便解释当时为何舍弃。

**真正的考验在第二天。** 一位没参加讨论的工程师接手，能否直接看出哪些事由谁确认、哪项还需找人？过了一周，如果有人提议导出隐藏列，团队能否找到原先的理由，明确修订哪一版范围？倘若大家仍要把原作者叫来重讲一遍，或代理把一次试探当成承诺，房间里保存了再多消息也无济于事。

我看好 Room 作为需求澄清的主要场所，同时会把“团队愿意另开一个聊天室”单独拿出来验证。明确的小修改可以沿用 issue，评审中的局部疑问可以留在 PR；遇到多人多轮才能解决的分歧，再开 Room。HCTL2 的上下文设计已经允许取用任务卡和评审评论。未来多人协作的第一步，是让后来的人看懂并参与一个决定。组织和角色权限仍在当前交付范围之外。[讨论来源](../../docs/design/context.md) · [Room 与 Request](../../docs/design/project.md) · [当前范围](../../docs/design/delivery.md)

## 实现层脚注：已经看到什么

- **Room 的可用性经受过一次具体检验。** 第一次演示的后台流程走通了，人在 Cinny 里却看不到新 Topic，进房后也缺开场提要。随后的补丁把确认过的提要发进房间，并按名单逐人邀请。这正是会影响第二位参与者能否接上讨论的问题。[首次验收](https://github.com/yesme/hctl2/pull/309) · [修复与验证](https://github.com/yesme/hctl2/pull/318)
- **验收链已有可反查的交付。** 10 月 20 日的演示中，Codex 从已采纳的契约取得规格，完成 `digit_sum` 和三个测试。受保护的 GitHub 分支通过检查后合入，PR 的合并提交与完成记录一致。记录还区分了平台读回的事实与人的验收陈述；另一张缺少契约的卡在尝试完成时被拒。这证实了交付路径的可行性。原任务很小，还不足以证明团队在复杂需求中能省下多少返工。[交付实录](https://github.com/yesme/hctl2/pull/482) · [实际 PR 与测试](https://github.com/yesme/hctl2-canary/pull/42)
- **采用成本有现实参照。** 同一次演示里，接入 GitHub 任务源到建卡约五分钟，Codex 写完约 59 秒。前者包含首次接入，不能当成每次任务的开销。另一方面，GitHub 已可要求 CI、独立审批，并在新提交改变 diff 时撤销旧批准。简单任务很难靠多一道验收流程省回时间。[演示耗时](https://github.com/yesme/hctl2/pull/482) · [GitHub 分支保护](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
- **公开介绍影响首次试用。** 这次查阅时，首页还写着没有可用的公共 CLI，而使用说明和实录已经给出了可操作路径。反过来，多席位计票算法和 Run 启动时冻结施工清单虽已实现，相关 PR 也标明了引擎和派工的接线边界。我建议首页先展示一个能点开核验的例子，再交代成熟度。[仓库首页](../../README.md) · [使用说明](../../docs/usage.md) · [计票实现](https://github.com/yesme/hctl2/pull/462) · [Run 实现边界](https://github.com/yesme/hctl2/pull/492)

## 我会怎样决定下一轮投入

给这轮验证四周，找 3–5 位平时混用编码代理的外部维护者。请其中两位带来一个确有分歧、最近发生过返工的需求，保留他们惯用的 Slack、issue、PR 和 CI 规则作比较。让他们在 Room 里形成决定，再把工作交给没参加讨论的人。分别记录首次接入和日常操作所花的人力时间、接手者还要追问几次，以及最后是否把代码交到了约定版本。

我会请接手者独立回答三个问题：当初承诺了什么，为什么这样定？哪一版代码满足了哪些条款？还有什么要由人确认？第一次试用完成后，再观察团队下一次改需求会不会主动回到原 Room，以及是否愿意自行带来第二个真实任务。

我的继续投入门槛是，至少两位外部维护者能在无人陪跑时再次使用，并各自指出一次省下的复述、复核或返工。若价值集中在把验收结果送回现有 PR，就收窄为 CLI 或 GitHub Check；若团队主动回到 Room 修订决定，就继续投资聊天室的澄清与交接体验。若连第二次使用都没有，我会暂停独立产品投入，按维护者自己的实际收益决定是否保留为自用工具。这个门槛是我的建议，外部试用的结果仍待观察。

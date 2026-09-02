# 方法论生态扫尾（2026 下半年）：软件工厂亚种、守卫与账本族扩容、非方法论清单

> 日期：2026-09-02<br>
> 状态：Informative 研究备忘录，不定义 HCTL2 语义；是[方法论生态审计](./methodology-landscape-20260824.md)（十二族，2026-08-24）的限时扫尾，只看 2026-07-01 之后新出现的对象，已审的十一个仓库与 [mattpocock/skills](./methodology-mattpocock-skills-20260902.md) 不重复审。复用判断用 [docs/research/README.md](./README.md) 的五种复用决策用语。<br>
> 方法：GitHub 搜索 API 约 40 组查询（关键词与 topic，`created:>2026-06-30`，星数 150 到 1000 分档，另做一轮 `agent stars:>=1000` 全量目检 100 条），加 WebSearch（Hacker News、Latent Space、各家 harness 官方博客与文档）与 HN Algolia 复核。每个候选用 gh api 实测创建日期、最后推送、星数、提交数（Link 头）、贡献者、非作者且非 bot 的 issue/PR 数与作者数、许可证；再用 contents API 读源码或机制文档。宣传语与 README 承诺不作数，只看阶段与判定机制在源码或文档里实际长什么样。<br>
> 门槛：判「有真实用户」需要至少 200 星且有非作者的 issue 或 PR，或被某家 harness 官方文档引用；不达门槛的只列入「查过不入表」。<br>
> 证据钉：全部 API 数据为 2026-09-02 实测；各仓库钉 HEAD 短 SHA，见候选表。时间预算控制在一个工作日内，够用即收，没有做完整 clone 审计——凡写「值得单独立条目」的，都是建议下一轮按 mattpocock 条目的口径补做逐源码审计。

## 结论先行

1. **严格意义的第十三族没有，但有一个值得单列的新亚种：软件工厂（software factory）。** 2026-08 同一个月里，两个互不相干的来源做出了同一个形状：教学者 IndyDevDan 的 [super-simple-software-factory](https://github.com/disler/super-simple-software-factory)（791 星）与 Vercel Labs 官方模板 [Foreman](https://github.com/vercel-labs/eve-software-factory-template)（1,084 星）。驱动机制是「代码持有控制面，agent 是无状态的有界节点，交接靠类型化数据，验收靠代码事后核对 agent 的声明」。它落在旧地图「多 agent 拓扑家族」的三个亚种之外：拓扑在代码里（同编排器），但 agent 不是长生命进程也不是提示词角色，而是一次输入一个信封、输出一个信封的函数调用。商业源头是 Factory.ai 2026-06-15 的「Factory 2.0」，讨论源头是 HumanLayer 2026-07-23 的《Why Software Factories Fail》（HN 394 分、272 评论）。两个开源样本都不满一个月、都由单人写成，这个亚种能不能活过半年是未知数；现在记下来是因为它在阶段边界上的画法跟 HCTL2 直接可比（见第四节）。
2. **旧地图第十二族「TDD/eval 驱动（弱家族）」该改名扩容为「机械守卫与验收账本」，并且不再弱。** 两个月里冒出四个同机制样本：[unlazy](https://github.com/Leonxlnx/unlazy)（2,966 星，可执行的 `GATES.md` 完成账本 + Stop 钩子）、[stop-that-shit](https://github.com/lennney/stop-that-shit)（1,218 星，五家 harness 的 PreToolUse 钩子按声明的约束拒绝越界动作）、[old-coder](https://github.com/AmazingAng/old-coder)（705 星，人批测试计划、审证据报告、不读 diff）、[procoder](https://github.com/azrtydxb/procoder)（200 星，Go 二进制提交闸门，「未检查计作失败」）。共同点不是 TDD，而是把「prompt 当强制层」换成 harness 侧的机械拒绝或机械账本——这正是旧审计里全生态缺的那一环。其中 **unlazy 值得单独立条目**：它是本轮唯一把「勾了框但证据是 pending 就算没完成」写进代码的对象。
3. **值得补进现有家族的头部样本还有三个，都只需一行，不必单立条目：** [ProductSpec](https://github.com/gokulrajaram/ProductSpec)（spec 族的上游一层：产品意图的开放格式，`spec_revision`、`AC-n` 稳定键、scope 的 in/out/cut 三分，7 月中旬起停更）；[icm-architect](https://github.com/RinDig/icm-architect)（上下文工程族变体：用编号目录当排序、用 markdown 当状态，「结构替代编排代码」，有 arXiv 论文）；[better-harness](https://github.com/QoderAI/better-harness)（2,137 星、28 位贡献者，不是协作方法论而是方法论的评测层——用会话证据给 harness 配置打分；它的证据状态词表「Observed / Missing / Unobserved / Not applicable」与「配置了不等于用过了」两条原则值得借）。
4. **判定为非方法论的高星现象**：本期星数最高的两个同赛道对象是产品不是方法论——Y Combinator 的 [qm](https://github.com/yc-software/qm)（14,443 星，「Multiplayer agent harness for work」，MIT）与 AWS 的 [KiroCrew](https://github.com/kirodotdev/KiroCrew)（3,547 星，持久工作区 + 编排层，Apache-2.0）——它们属于实现证据的「② Agent 协作平台」类别，建议转给 workbench 研究线。其余高星是领域技能包、Context 与 token 基础设施、模型路由、电脑操作 harness、教学材料，清单见第五节。
5. **借鉴决策的总答案不变：采用为依赖为零。** 新增两个适配协议候选（unlazy 的账本文件格式、stop-that-shit 的 `ControlEvent v1` 五家钩子归一化事件），一个有边界移植候选（unlazy 的零依赖 `gate-check.mjs`），其余全部仅参考行为。

## 一、候选表

「验证数据」列口径：星数 / fork，创建 → 最后推送，提交数，贡献者数，非作者且非 bot 的 issue+PR 数（去重作者数），许可。「族」列用旧地图的编号：①spec 驱动 ②任务图 ③流程/角色模拟 ④轻量纪律 ⑤编排器 ⑥Ralph 循环 ⑦上下文工程 ⑧技能包当方法论 ⑨会话多路复用 ⑩系统记录寄生 ⑪Swarm ⑫TDD/eval（本文建议改为「机械守卫与验收账本」）。

| 仓库 @ 钉定 | 族归属 | 验证数据 | 一句话机制 | 单独立条目？ |
| --- | --- | --- | --- | --- |
| [disler/super-simple-software-factory](https://github.com/disler/super-simple-software-factory) @ `de313748` | **新亚种·软件工厂**（多 agent 拓扑家族第四亚种） | 791/203，08-02 → 08-04，main 仅 1 提交（`example` 分支有完整落地），1 人，16（11 人），MIT | Python ADW 脚本持有排序、重试与验收；每个 agent 调用绑定一个 Pydantic 信封类型，`status` 是硬字段；`gate(envelope, run)` 事后核对声明（产物存在、非空、JSON 可解析、评审结论自洽）；`writes:` 边界在每次调用后由 `permissions.py` 核对并回滚；`run.finish(accepted=)` 把「阶段都过了」与「这次运行被接受」分开；「写得出命令的就是 `kind="code"` 阶段，不给 agent」 | 值得，与 Foreman 合为一条 |
| [vercel-labs/eve-software-factory-template](https://github.com/vercel-labs/eve-software-factory-template)（Foreman）@ `0d630a28` | **新亚种·软件工厂** × ⑩系统记录寄生 | 1,084/59，08-12 → 08-20，9 提交，1 人（Vercel Labs），非作者 PR 仅 1 人（其余 7 个 PR 是作者本人走 PR 合入）；Show HN 10 分 4 评论；Vercel 模板页与 KB 文档引用，MIT | 编排器是 prompt，但四个工位（classifier → analyst → implementer → reviewer）都是带 `outputSchema` 的 task-mode 子 agent，跑完必须返回结构化输出、不能中途问人；reviewer 用另一家厂商的模型、必须 `checkout_branch` 读真 diff、逐条验收标准给 pass/fail；返工最多两轮；信任在 webhook 派发时盖章，不在 prompt 里判；`approval.ts` 的审批谓词就是授权策略：无人值守一律 denied 而不是挂起等人；草稿 PR 是无人值守的天花板（草稿合并不了），标 ready 与 merge 不在工具面里 | 值得，与 SSSF 合为一条（严格说不达「非作者 issue/PR」门槛，靠厂商官方文档与 Show HN 入表） |
| [Leonxlnx/unlazy](https://github.com/Leonxlnx/unlazy) @ `473d4b80` | ⑫（改名后的头部样本） | 2,966/187，08-09 → 08-29，48 提交，10 人，29（18 人），MIT | `GATES.md` 是机器校验的完成账本：每个门有 id、`CHECK:` 命令、`EXPECT:` 匹配、`EVIDENCE:`；只有进程退出 0 且输出匹配才算过；**勾了框但证据缺失或 pending 仍算未完成**；`ABANDON:` 是终态但永不等于完成，退出码 1 打 `HANDOFF REQUIRED`；执行 `CHECK:` 需要人审批，审批绑定精确命令、期望、工作目录、shell、超时、平台与 PATH，任一变动需重批；父节点用 `--reverify` 重跑子节点全部门；Stop 钩子只做结构性拦截不执行命令。37 KB 零依赖 Node 脚本，7 组测试。Depth Tree（层层拆、每叶给全预算）是外面那层 prompt | **值得**，按 mattpocock 口径做单对象逐源码审计 |
| [lennney/stop-that-shit](https://github.com/lennney/stop-that-shit) @ `68f4a7a1` | ⑫（守卫一侧） | 1,218/37，08-11 → 09-01，52 提交，5 人，10（9 人），MIT | 用户一句话声明约束（原文叫 contract）：模式 answer/review/change/monitor、级别 watch/guard/lock、`agents=N`、`files=`、`deps=`、`hash=`；五家 harness（Codex、Claude Code、OpenCode、Hermes、Pi）的钩子事件归一为 `ControlEvent v1`，在动作发生前分类可变性并 **返回 deny**（`MODE_FORBIDS_MUTATION`、`PATH_OUTSIDE_CONTRACT`、`AGENT_BUDGET_EXHAUSTED`、`UNBOUNDED_DELEGATION`）；`Workflow` 类工具因子代理扇出不可证而在 guard 下整体拒绝；运行时记录 deny 但把宿主效果标为 `unobserved`——「返回了 deny 不等于宿主真的没做」 | 不单立；进 unlazy 条目的同族对照，或在旧地图第十二族补一行 |
| [AmazingAng/old-coder](https://github.com/AmazingAng/old-coder) @ `a0eb529d` | ⑧形态 × ⑫内容 | 705/55，07-27 → 08-18，78 提交，4 人，9（4 人），MIT | 纯 prompt（21.9 KB SKILL.md + 4 篇参考，无脚本）。人只碰两个工件：写代码前批 SPEC（可执行验收标准 + 环境变更计划），写完后读 EVIDENCE（每层命令与真实数字），不读 diff；SPEC → RED → GREEN → REFACTOR → GAUNTLET（测试/类型/lint/改动行覆盖/突变/属性/真实运行/供应链）→ EVIDENCE；「对问题的回答不是批准」「拿不出批准这版 SPEC 的原话就没有批准」；反作弊六条；Tier 3 可选一个新上下文的独立核验者，人给结论定级 | 不单立；旧地图第十二族补一行，第八族「纪律正被技能包吸收」的新例 |
| [azrtydxb/procoder](https://github.com/azrtydxb/procoder) @ `1c6a1fa8` | ⑫（守卫一侧） | 200/14，08-16 → 09-02，508 提交，实为 1 人，外部作者约 2 人，Apache-2.0 | 单个 Go 二进制挂在写入与会话开始的钩子上算提交闸门：格式、冲突标记、垃圾文件、越界；「工具跑失败了永远不报 clean——unchecked 计作失败，并且说出来」；二进制只算不改，agent 自己动手 | 不单立；刚过星数门槛，外部作者太少 |
| [QoderAI/better-harness](https://github.com/QoderAI/better-harness) @ `cfdf0d6d` | **相邻元层**：方法论的评测层（不是协作方法论） | 2,137/170，07-21 → 09-02，420 提交，28 人（phodal 329），84（29 人），MIT | 以 skill/plugin 形式跑在 Claude Code、Codex、Cursor、Qoder 等宿主里，收集项目资产与会话「Task Episode」证据，按 Agent Work Loop 五维（任务理解、受控执行、变更验证、可靠交付、经验沉淀）出带验收检查的优先级发现；证据状态四值 Observed / Missing / Unobserved / Not applicable，「配置了的能力不等于观察到的使用」，缺证据的维度分数封顶 59；引用 Fowler/Böckeler 的 feedforward/feedback 框架 | 不单立；旧地图补一段「评测层」相邻现象，词表进 Receipt 设计参考 |
| [gokulrajaram/ProductSpec](https://github.com/gokulrajaram/ProductSpec) @ `97b90b62` | ①spec 驱动（上游一层） | 280/30，07-04 → 07-19（此后无推送），115 提交，4 人，27（5 人），MIT | `.product-spec.md` 开放格式：frontmatter 有 `artifact_type`（hypothesis / prd / openspec_proposal）与 `spec_revision`（意图修订号，与格式版本分开）；六个必填节 problem / hypothesis / product_summary / scope / acceptance_criteria / success_metrics；`productspec-scope` 围栏块三分 in / out / cut，`productspec-acceptance-criteria` 块给 `AC-n` 稳定 id；带 conformance 用例、解析器与 GitHub Action；明确把 OpenSpec 当下游 | 不单立；旧地图 spec 族补一行，适配协议候选但项目已停更两月 |
| [RinDig/icm-architect](https://github.com/RinDig/icm-architect) @ `e16cafe6` | ⑦上下文工程（结构即编排变体） | 1,400/199，07-18 → 08-25，8 提交，1 人，11（7 人），MIT | ICM（Interpretable Context Methodology，arXiv 2603.16021）：编号目录承担排序、层级承担上下文作用域、markdown 文件承担状态，「一个 agent 在正确时刻读正确文件，顶一个多 agent 框架」；六种形态（Pipeline / Umbrella / Record library / Knowledge bundle / Context map / System map）；每个结果过「walk test」——无记忆 agent 只凭文件能定位、行动、报状态 | 不单立；与 Ralph 族「状态全在仓库文件」同源，补一行 |
| [Sahir619/fable-method](https://github.com/Sahir619/fable-method) @ `88b5cf36` | ⑧技能包当方法论 | 2,274/327，07-06 → 07-15（九天后停更），15 提交，1 人，9（8 人），MIT | 四个技能（think / act / prove / grow）+ LLM 裁判评测集：classify → define done → evidence → decide → act → verify → report，带硬界（verify 三次失败即停、两次查无即停）。星数来自「记录一个即将下线的模型怎么工作」的话题热度 | 不单立；热度型样本，第八族普通一员 |
| [cbrock84/headcount](https://github.com/cbrock84/headcount) @ `d58a7ee2` | ③流程/角色模拟 | 1,012/156，08-28 → 09-02（五天），64 提交（一半署名 claude），非作者 3（2 人），MIT | 把 Claude Code 技能包按公司部门组织（16 部门 172 技能），每部门一个可独立安装的插件加一份 `.claude/agents/` 章程，「加一个部门，不是加一个 prompt」 | 不单立；不达用户门槛；是旧地图「角色模拟往技能包塌缩」判断的又一例 |
| [aws-samples/sample-specship](https://github.com/aws-samples/sample-specship) @ `3c75e79f` | ①spec 驱动 × ⑧ | 217/5，07-10 → 07-13，1 提交，1 人，非作者 0，MIT | Kiro Power：RECON → PLAN → BUILD → VALIDATE → SHIP，明说自己是「叠在 superpowers 与 gstack 两套技能包之上的编排」；八个对抗式校验器给「typed verdict」但全是 steering 文件；唯一脚本 `specship-verify.sh` 只做 steering 文件的完整性哈希 | 不入正表；方法论叠方法论的例子，机制全在 prompt |
| [osolmaz/pi-workflows](https://github.com/osolmaz/pi-workflows) @ `e3566df5` | 基础设施（L2 工作流引擎），非方法论 | 255/15，07-18 → 09-02，434 提交，2 人，4（4 人），MIT | Pi 的工作流扩展：TypeScript 图定义节点，agent 步骤在当前会话内跑并经 JSON `workflow` 工具返回结构化输出，SQLite 单库存运行、决策、租约；**自带 `herdr-plugin.toml`（`min_herdr_version = "0.7.0"`）在 Herdr 面板里开 `piw` 查看器** | 不入方法论表；转 runtime 研究线一句话备注（Herdr 生态已有第三方插件） |

## 二、两处需要改地图的地方

### 2.1 多 agent 拓扑家族加第四亚种：软件工厂

旧地图 2026-08-26 补记把编排器、群体自治、流程/角色模拟合成「多 agent 拓扑家族」三个亚种，并断言稳得住的只有两头（拓扑在代码里进程是真的；拓扑在模型里进程是假的）。本轮两个样本落在第三个位置——拓扑在代码里，agent 是函数：

| | 编排器（Gas Town） | 群体自治（ruflo） | 角色模拟（BMAD） | **软件工厂（SSSF、Foreman）** |
| --- | --- | --- | --- | --- |
| 拓扑由谁持有 | 代码写死 | 模型临场生成 | 阶段文档链 | 代码写死（SSSF 是 Python 脚本；Foreman 的工位顺序写在 prompt 里，但每个工位的输入输出形状与授权在代码里） |
| 「agent」是什么 | 真进程，长生命，有角色 | 同运行时内一批 LLM 调用 | 提示词角色 | **一次调用一个工位：进一个信封，出一个信封**，没有跨工位记忆（Foreman：「工位看不到编排器的对话历史，每条委派消息必须自足」；SSSF：只有 `context_handoff/` 里的文件与上一封信封） |
| 交接靠什么 | issue 库 + 邮件 | 共享内存 | 文档 | **类型化数据**：SSSF 的 `EnvelopeBase` 子类（`status` 硬字段，解析失败在同一会话里纠正而不是重启）；Foreman 的 `outputSchema` + 按 id 传的 Blob 工件 |
| 完成怎么判、人在哪 | 机械关单，人在门上 | consensus 投票 | 人批阶段门 + 模型自勾 | **代码事后核对 agent 的声明**（SSSF 的 gate：产物存在、非空、可解析、评审结论不自相矛盾；`accepted=` 与「阶段都过」分离），**评审是独立工位**（Foreman 换厂商模型、读真 diff、逐条 AC），**人在最后一道**（Foreman：草稿 PR 是无人值守天花板，标 ready 与 merge 只能人做；SSSF：`run.finish(accepted=)` 之后是工程师） |

两处必须写明的实况，防止把宣传当机制：

- SSSF 的「gate 核对声明」在 v1 只核对到「声明改过的文件存在于磁盘」，**没有对照 git diff**——非作者 issue [#6](https://github.com/disler/super-simple-software-factory/issues/6)（2026-08-04）实测「声明改了 3 个文件，实际 diff 5,000 多行，gate 放行」，PR [#15](https://github.com/disler/super-simple-software-factory/pull/15) 才补上对照真 diff，至今未合。另有 issue #13「无预算上限 + 重试进死会话」。这说明「代码持有验收」在样本里还是设计立场多于成品，跟旧审计「同向碎片散落、没有系统边界」的总判一致。
- Foreman 的 reviewer 结论（approve / request_changes / reject）是模型产出，代码只保证它「独立、读真 diff、逐条 AC、最多两轮」；真正机械的是**权限上限**，不是判定：`approval.ts` 按调用者类别返回 not-applicable / user-approval / denied，无人值守直接 denied 而非挂起，理由是「没人在看，服务端拒一步比留一个悬着的会话便宜」——跟 spec-kit「无人时 PAUSED」是同一问题的另一种解。

这个亚种的行业意义：HumanLayer 那篇 394 分的文章列出了正在建「软件工厂」的公司（StrongDM 的 lights-off 工厂、OpenAI Symphony、Stripe Minions、WorkOS Project Horizon、Ramp、Brex），并给出失败原因——模型训练里没有「侵蚀可维护性」的惩罚，测试秒级反馈而坏架构的代价按月年计，所以「lights-off 不成立」，需要人在编码前批四道（产品评审、系统架构、程序设计、垂直切片）并且代码评审不可省。开源样本与这篇文章的分歧正好是 HCTL2 关心的：文章要人**在前面**多批，样本把人放在**最后一道**。

### 2.2 第十二族改名扩容：从「TDD/eval 驱动（弱）」到「机械守卫与验收账本」

旧地图第十二族只有 tdd-guard 一个过千星样本，定义写死在「下一步由失败测试决定」。本轮四个样本证明驱动机制其实是「把纪律放到 harness 能机械执行的位置」，TDD 只是其中一种纪律：

| 样本 | 机械边界在哪 | 拦什么 | 谁写规则 | 谁判完成 |
| --- | --- | --- | --- | --- |
| tdd-guard（旧） | PreToolUse 钩子 | 违反红-绿次序的写入 | 工具内置 | 测试套件 |
| stop-that-shit | PreToolUse 钩子（五家归一） | 越出声明约束的写入、加依赖、哈希、子代理扇出 | 人一句话声明模式与边界 | 不判完成，只判每一步能不能做 |
| unlazy | 账本检查器 + Stop 钩子 | 「勾了框但没证据」的完成声明 | agent 写门与 `CHECK:`，人审批命令 | 检查器：退出 0 且输出匹配；父节点 `--reverify` |
| procoder | 写入与会话开始钩子里的 Go 二进制 | 未格式化、冲突标记、垃圾文件、越界；工具失败计作失败 | 工具内置 + 项目配置 | 提交闸门（卫生层面，不是验收） |
| old-coder | 无（纯 prompt） | 靶向同一批问题，但靠 agent 自觉 | agent 起草 SPEC，人批 | 人读 EVIDENCE |

这一族与 HCTL2 立场最接近的两句话都出自 unlazy：「Count a checked box with missing or pending evidence as unmet」「Abandonment is terminal but never successful completion」。它的漏洞也清楚：门与 `CHECK:` 命令是 agent 自己写的，审批只绑定命令文本与环境，不哈希被调用的脚本与夹具（SECURITY.md 自认），所以「命令可以跑」不等于「命令测的是那句英文」。

## 三、完成判定权横评补行

口径同旧地图第三节：HCTL2 立场是 Task 完成只接受有权人类命令，或绑定 Task 的 Run 正常完成后的确定性归约。

| 工具 | 完成判定实况 | 与 HCTL2 立场 |
| --- | --- | --- |
| SSSF | 阶段通过由代码判（信封解析 + gate），运行接受由 `run.finish(accepted=)` 判，两者分离；但 `ReviewOutput.approved` 是模型写的，gate 只核对它与自己的 findings 不矛盾；`diff_matches_claims` v1 未对照 diff（issue #6） | 结构同向（代码持有 accepted），实质靶心仍是模型结论；gate 的「核对声明」是正确的形状、未完成的实现 |
| Foreman | reviewer 模型给 approve；代码保证独立性与轮数上限；草稿 PR 是无人值守天花板，ready/merge 只能人做；无人值守下写操作一律 denied | 出货边界同向（人决定 ready，merge 不在工具面），流水线内部违反（approve 是自述）；「用制品的能力定权限天花板」是新画法 |
| unlazy | 门过不过由检查器按退出码与输出匹配判；勾框无证据视为未完成；父节点必须重跑；ABANDON 永不等于完成 | 判定机制同向且机械；但规格（门与命令）是模型写的，人只批「命令可以跑」，规格权没收回 |
| stop-that-shit | 不判完成；判每一步能不能做，越界返回 deny；诚实标注宿主效果 `unobserved` | 同向的是「prompt 不是强制层」这一点；与完成判定权正交 |
| old-coder | 人批 SPEC（施工前），人读 EVIDENCE（施工后）；EVIDENCE 里每个数字由 agent 自跑自报；Tier 3 可加独立核验者、人给结论定级 | 前后两个人类触点同向（≈ Task Revision 与 Receipt），中间全靠 prompt 自觉，施工侧违反 |
| procoder | 提交闸门机械，「未检查计作失败」；但只覆盖卫生与范围，不覆盖验收标准 | 同向但窄 |
| better-harness | 不判完成；给已发生的工作打证据分，「Missing / Unobserved」封顶 59 | 与判定权正交；词表可借 |

## 四、对本轮四轴评审的意义

问题是：这些新工具在阶段边界上有没有跟 HCTL2 四场景（Project/Room、Task/Kanban、Run/Workflow、Agent/Terminal）不同的画法。有四处不同，两处相同。

**不同一：分诊是一道独立边界。** Foreman 的第一个工位是 classifier（类型、优先级、复杂度、是否可行动、是否需澄清），`needs_clarification` 直接终止流水线——有人就问人，无人值守就把问题贴回 issue 然后停。SSSF 的 `adw_scout` 是只读侦察 ADW，「什么都不改」。HCTL2 把这一步隐含在 Room 的塑形里，没有一个显式的「可行动 / 需澄清 / 不做」三态出口。值得在 Project→Task 的边界上问一句：Task 冻结之前要不要有一个机械可见的分诊结论。

**不同二：评审是流水线里的一个工位，独立性靠换人（换厂商、换上下文），不靠确定性。** Foreman 的 reviewer 用另一家模型、只看真 diff、逐条 AC 给 pass/fail；old-coder 的 Tier 3 核验者「新上下文、盲审、只给四个输入」；SSSF 的 reviewer 有 `verdict_consistent` gate 兜底。HCTL2 的 Verdict 在 Run 里，独立性来自「确定性归约或有权人类」。两种画法的分歧不在位置而在信任来源：软件工厂相信「换一个模型再看一遍」能买到独立性，HCTL2 不相信。这条分歧值得在 Run 的设计文档里写成显式的否决理由，因为它会是评审里最常被拿来质疑「为什么不直接让另一个模型 review」的点。

**不同三：权限天花板由制品的能力定义，不由状态机定义。** Foreman 的「草稿 PR 对任何调用者都放行，因为草稿合并不了；能出货的动作一律等人或拒绝」，是用 GitHub 制品的固有能力当无人值守边界，代码里没有一张状态表。HCTL2 用 Task Completion Receipt 与有权人类命令画这条线。两者不冲突，但 Foreman 的画法提示：Run 产出物的「能力上限」（能不能合、能不能发、能不能关票）可以作为 Seat 权限的表达方式之一，比枚举动作更稳。

**不同四：边界可以画在一次工具调用上。** stop-that-shit 与 procoder 把边界画在 harness 的 PreToolUse 一步，unlazy 画在 Stop 钩子与账本检查器。HCTL2 目前最细的治理粒度是 Run/Seat，没有到「这一次写入合不合法」。这不必然要改 HCTL2 的模型，但给 Harness 适配器提出一个具体问题：Run Manifest 里的边界（可写路径、依赖策略、子代理预算）要不要经钩子下发到 harness 里机械执行，而不是只写进 prompt。`ControlEvent v1` 已经把五家 harness 的钩子事件归一成一个形状，是这条路的现成协议候选。

**相同一：没有一家有不可变的 Task。** Foreman 里 issue 就是任务，analyst 的计划与 AC 是按 id 传的 Blob 临时工件；SSSF 的计划是 `context_handoff/` 里的文件；unlazy 的 `GATES.md` 是活文件，agent 可以改门（只是不能悄悄删，要 `ABANDON`）。旧地图「约束冻结谁都没有」（原文作契约冻结）的判断在 2026 下半年样本里继续成立。

**相同二：「先塑形再施工」的两触点人类接口在收敛。** old-coder 的「人批 SPEC、读 EVIDENCE、不读 diff」，HumanLayer 的「编码前批四道、代码评审不可省」，Foreman 的「AC 在前、逐条 pass/fail 在后」，都是 Task Revision 在前、Receipt 在后的手工版。old-coder 还写了一条跟本 repo 工作纪律四一字不差的规则：「对问题的回答不是批准」——可以直接引进设计文档当外部佐证。

## 五、判定为非方法论的高星现象

全部为 2026-07-01 之后创建、2026-09-02 实测星数。列出来是为了下次不必再查。

- **同赛道产品（应归实现证据「② Agent 协作平台」）**：yc-software/qm 14,443（YC 内部全公司多人 agent harness，Slack + Web，每人每房间独立记忆、文件、权限、cron 与沙箱，MIT，07-29 起 219 提交、28 位外部作者）；kirodotdev/KiroCrew 3,547（AWS，内部工具 MeshClaw 开源，持久工作区 + 多 agent 编排 + 定时任务 + Slack/Discord 接入，「纠正与失败会改变后续行为，重复模式沉淀为 skill」，Apache-2.0，4,796 提交、27 位外部作者）；yetone/cumora 3,412 与 elie222/rakazo 1,746 已有条目；agentconnect-md/agentconnect 1,228（Claude Tag 的多 agent 开源替代）；milind-soni/OpenMausBot 2,018；useagenthq/useagent 272。
- **Coding harness 本体**：xai-org/grok-build 26,368（已有条目）；vercel-labs/fx 2,684；truefoundry/trueforge 5,089；fuxicodex/Fuxi 3,229；ShenSeanChen/waku-agent 1,647。
- **电脑操作 / 长程 harness**：AMAP-ML/LongHorizon-Harness 1,435；ShawnPana/phone-harness 2,133。
- **agent 应用开发平台（不是编码协作）**：Prism-Shadow/penguin-harness 1,918（hiyouga，「agents build agents」自演化，Swarm 族相邻但对象是 agent 应用而非代码库）；unicity-aos/aos-ce 8,550；CopilotKit/OpenBot 3,832。
- **Context 与 token 基础设施**：trailhq/Graft 5,407；Paritok-official/paritok-4b-v1 1,450；yc-duan/fastctx 1,272；VictorTaelin/OptMem 1,493；MemTensor/memmy-agent 1,228；vshulcz/deja-vu 758。
- **模型路由与接入**：duolahypercho/codex-router 3,006；miuuyy/codex-chatgpt-web 3,945；XiaoDuoYa/codex-with-chatgpt 2,226。
- **领域技能包**：img2threejs 14,965；video-shotcraft 7,066；jakubkrehel/skills 4,752；ip-as-logo-skill 4,755；gzh-design-skill 3,468；lieflat-charts 3,371；J-Space-Cognition-Suite 3,002（伪科学向）。
- **教学材料与论述**：lopopolo/harness-engineering 2,658（CC-BY-4.0 文选，单日提交）；hahhforest/pi-textbook 1,302；SaladDay/pi-from-scratch 1,163；mouredev/hello-sdd 401；DEEP-JLU/Awesome-Graph-Engineering 273；humanlayer 的 wsff.md（仓库 2025-08 建，文章 2026-07-24 提交，2,584 星）——是本期最重要的论述但不是工具。
- **评审与可视化 UX 工具**：petergyang/human-review 1,209（在 HTML/MD 上像 Google Doc 一样批注并回传 agent）；cosmtrek/mindwalk 1,309；sodiumsun/agenttrail 610。
- **会话多路复用族的新样本（旧族，不新）**：YoanWai/agent-manager 384；cristicretu/diri 279；LodyAI/Lody 863（远程操控向）。
- **agent 间消息基础设施**：Get-Concord-AI/concord-mcp 302。
- **DeepSeek Harness 生态**：Dominic789654/awesome-deepseek-harness 217 列了带「Plan Mode / TDD / delivery-gate / acceptance-checklist」「tool guards + audit ledger」字样的插件，实测星数全是个位数（dsh-omni-router 4、dsh-rule-engine 4、helmsman 0、oh-my-tianshu 39），不达门槛。

## 六、查过但不入表的候选，与本轮方法的局限

- 不达用户门槛：Hoylon/peerbridge-mcp 194 星、0 fork、0 外部 issue（「可审计多 agent 编码、评审、证据与发布的治理层」，题目与 HCTL2 最近，但没有人用）；Vuk97/forward-implementation-first 161 星（与 stop-that-shit 同题：拦 agent 自发的收据、哈希、锁）；smthdagg 与 k-telux 两个「evidence-gated」仓库是领域应用。
- 不在窗口内：ZhangHanDong/agent-spec（449 星，BDD/规格校验，2026-03 建，8 月仍活跃）、anthropics/cwc-long-running-agents（2026-05 建）、Claude Code agent teams（2026-02）、Fowler/Böckeler 的 harness engineering 文章（2026-04-02）、Databricks Omnigent（2026-06）。agent-spec 是上一轮扫尾漏掉的 spec 族样本，与本轮无关，记在这里备查。
- 官方引用核查：本轮没有发现任何一家 coding harness 的官方文档引用上述新对象；Codex 插件目录与第三方榜单里被反复推荐的仍是 superpowers、mattpocock/skills、Trail of Bits skills，都在旧地图内。
- 方法局限：GitHub 搜索 API 对多词查询召回极差（同一批仓库用两三个词能搜到、五个词就空），所以补了一轮 `agent stars:>=1000` 全量目检；HN Algolia 显示除 HumanLayer 文章外，本轮所有对象的 HN 讨论都在 10 分以下，星数主要来自 X 与 GitHub trending，不能当社区共识看。没有 clone 与完整提交历史审计，「单人还是团队」只看贡献者分布。

# Muse 视角：Grok Bot 与 hctl2 对照的收口建议

> 关联：`.memo/notes/hctl2_grokbot_compare.md`（`c84c2b6`，Gemini 主笔，ChatGPT 协作，1088 行，27 项一手来源，基线 `e0cc1ee / v0.18.2`）是与 codex 就 https://chatgpt.com/share/6aaaa1ae-8dec-83ec-bd22-0033464d53f4 的系统讨论后沉淀的完整对照。本文不重复那份报告的一手核验，只对其中提炼的三条核心观点做“是否值得为此完善、甚至推翻现有 design doc”的裁定，并以建议形式交给 design doc 主笔 Claude。
> 定位：三条中第 1 条定方向，第 2 条定证据口径，第 3 条定推荐层。均不要求推翻重写，只需在现有愿景与架构的半句间隙上收口。

## 一、三条核心观点逐条定性

### 1. grok bot 核心是“数字伙伴”，hctl2 核心是“项目与承诺”

**定性：成立，且与现有稿同构。**

`docs/design/vision.md:一句话定位` 已写的就是后者：人主导的目标塑形与机器驱动的可验证施工之间的桥，`Project-scoped、Room-mediated shaping、Task-tracked、Run-executed`。`docs/design/architecture.md:5×3 归属矩阵` 把 `Project/Room ↔ 意图 / Task/Kanban ↔ 承诺 / Run/Workflow ↔ 治理 / Participant/Terminal ↔ 执行 / Repo/Change ↔ 落地` 分开，正好说明为什么 hctl2 要按承诺存事实，而不是按 Bot 记记忆。

对话录 `hctl2_grokbot_compare.md:896-912` 那张“长期主体”对照表与 `§核心判断` 已把区别写准：Grok 让伙伴持续存在、工作围绕伙伴展开；hctl2 让项目与承诺持续存在、为其选入合适的执行者。Grok 的设计文章也明说从聊天历史转向 Bot 名册。

对话后半段你把“任务本身”校成“项目与承诺”（892 行），与稿内 `Project 驱动的控制 / Participant 是席位级选入记录` 一致。不存在推翻点。

**若要完善，只补半句括号：**

- *建议位置* `vision.md:一句话定位` 末尾，*不改文件，仅描述*：
  > 原句“……多 Harness 项目协作系统”后加括号“（数字伙伴在 hctl2 里是作用域内的任用，不是跨项目身份；连续性放在项目与制度上）”。
  理由：把 892 行那句已经够准的表述抬到愿景，避免后人把 `Participant` 当成 Grok 的长期 Bot 直接套用。

### 2. agents 间通过聊天来“协作”不产生额外价值，价值来自“多样性”与外部“机械制度”

**定性：research 支持，但需收窄为“无约束的聊天不产生稳定价值”。**

Fact-check（你标注的 `/ fact-check here`）：

- `docs/research/multi-agent-effectiveness-20260908.md:三问` 的结论就是这个。问 1 引用 `Stop Overvaluing Multi-Agent Debate`：近两年二十余家复测净效应接近零；问 2 把 Gate 换成模型互辩的收益远低于换席位异构；问 3 列出有效的是异构 + 机械门禁，不是多聊几轮。`SILO-BENCH (ACL 2026)`、`Nature SciRep: When collaboration fails` 也复现了“恶意 agent 带偏、合作反而损失四成准确率”的失败。
- 对话录 `860-868` 已自行收窄：Debate 在 `Debate or Vote (2508.17536)` 与 `The Cost of Consensus (2605.00914)` 的测试中，多数投票已解释大部分收益，且有收益的配置不能推广为全有效；增加通信不能当作质量依据。
- 现有设计稿已是收窄版：`vision.md §设计原则 4`（协作拓扑与控制拓扑正交）、`CONSTRAINTS.md` 与 `spec/run.md §Gate 多样性策略 + 法定票数` 均要求**多样性要显式声明席位互异，Gate 多样性默认不声明**，而不是让 agents 聊出多样性。

**完善建议（不改文件）：**

- 在 `vision.md §省` 末尾加证据锚点“见 `multi-agent-effectiveness`”，把这份调研从 `docs/research/` 抬到原则层。写法示例（建议稿）：
  > `多样性策略与机械门禁见 multi-agent-effectiveness（异构优于同质辩论）。`
  理由：避免后人把 Grok 的 `Auto Review 双模式同时命中保守方优先`（备忘 `§3.2 #3-4` 已证它在 Grok 客户端默认是 shadow、不拦）抄进 Gate。
- `spec/run.md §Gate` 若以后要写席位多样性，一律按“法定票数默认 0，要多样性就显式声明”写，不因对抗 Grok 而另起一套。

### 3. hctl2 通过扩展 TAMP 来实现“数字伙伴”的管理

**定性：方向对，现状是“制度为主，伙伴为推荐层”。**

你在对话末尾 `960-998` 已自我修正：

> 制度与伙伴是可以同时存在的。事实上，在 HCTL2 的研究/讨论中，专门有一章 TAMP 模型，就是为了在未来、通过在实践中不断获得检验的能力模型，Agency 的工种模板逐渐获得了相对比较稳定的能力评价 - 因此，主动“选伙伴”的过程就可以进一步被优化为被动“被推荐伙伴”的 discovery/recommendation 过程：能力相似度相当于 relevance 召回，按需求/供给的 `$` 做最后的精排。这样，我们既有机械化的制度，又有高度匹配需求的伙伴 - 有一点还是保持了 - HCTL 的 core 依然是“项目与承诺”。

紧接着 Codex 在 `965-998` 把它展开为 `Recall → Gate → Rank → Award` 四段，`$` 是经济性，回读时已强调不改变 core。

与现有稿对照：

- `.memo/notes/TAMP-design-doc.md` 与 `tamp-review-20260905a.md` 在仓库里仍是**讨论与演进材料**，状态不是已生效约束，与你说的“未来通过实践逐渐形成”一致。
- 现行 `spec/participant.md` 的选入是 `Room/Seat 作用域 + 工种模板 + 执行配置分开`，正好对应 `长期角色（稳定身份/职责）≠ 本次任用（预算/授权）≠ 本次执行（Dispatch）` 三层（`hctl2_grokbot_compare.md:502-506`）。

**完善建议（不改文件）：**

- 在 `docs/research/README.md` 的跨候选总览里，把 TAMP 那行已有的“跨候选归纳”描述，追加一句“伙伴发现/推荐是制度上的召回+精排，不另起 Task/Run，需求画像来自现有 Project/Task 契约”（即你那句 `$` 精排）。写法示例：
  > `TAMP 能力模型：按任务族维护校准，Recall（能力相似度）→ Gate（做审分离与可行性）→ Rank（`$` 经济性）→ Award（任用），不改变 Project 为核心的组织。`
  让后人不再把 TAMP 当成要另建一套 Task/Run。

## 二、对现有 design doc 是否值得推翻重写的总判

**不推翻。** 三条中第 1 条定方向，第 2 条定证据口径，第 3 条用 TAMP 的召回+精排把“伙伴”从产品中心降成推荐层而不动 core。现有愿景与架构的三处半句间隙补齐即可；对话录 1053 行的 27 项核验已把需推翻的点（如“看 participant 运行环境/PC 桌面”的默认 TUI 与 headless 关系、`@` 不能派工、账号级共享盒子等）落在备忘与建议，不进主线。

补充一处与设计稿的印证：

- Grok 多团队以 `Notion Projects/Tasks` 为 Kanban（`hctl2_grokbot_compare.md:35`），作者自标实验性；hctl2 的 `Task Revision → Receipt` 与 `Kanban 场景` 分离已足够，无需为此改 Kanban。
- 你的原文 778-798 那段“制度的边际成本远低于中心协调的边际成本”与 `vision.md §省` 的“把不需要智能的工作从模型推理循环里移出去”（`hctl2_grokbot_compare.md:803-832` 制度表）同构，正好解释为什么把 `谁已获权、结果交给谁、前置是否满足、什么算完成` 预先定义好能省 token。

## 三、具体想怎么改（仅描述，不在本 PR 落）

以下为交给 Claude 的建议稿，均标注“描述，不落”：

1. `vision.md:一句话定位` — 加括号句（见一节末）。
2. `vision.md:§省` — 末尾加 `多 Agent 有效性见 multi-agent-effectiveness` 锚点。
3. `architecture.md:场景与系统` — 加一句“隔离单位是 Task/Run/Seat”对照 Grok 的账号级盒子（见 `hctl2_grokbot_compare.md:61`）。
4. `docs/research/README.md:TAMP` — 追加召回+精排一句（见上）。
5. `docs/design/README.md:Participant 入口` — 追加“Participant 是 TAMP 中 Action 的执行者抽象”一句。

> 说明：以上均不改变 `spec/` 的裁决、不新增对象与状态机、不把 `@` 改成可派工；真正需要改 `spec/` 时，按 `CONSTRAINTS.md` 另起约束批并配 CT。

## 四、来源与边界

- 对话录：`c84c2b6`；hctl2 基线 `e0cc1ee / v0.18.2`；Grok 侧取 2026-09-03 设计文章与 `docs.x.ai` 专门文档，未做登录后端到端实测。
- Research：`multi-agent-effectiveness-20260908.md`、`grok-bot.md` 与 `grok-bot-reconstructed-audit` 的 §3–§4。

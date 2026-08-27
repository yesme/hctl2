# Context 处理生态审计：四族地图、快省准横评与 HCTL2 落点

> 日期：2026-08-24<br>
> 状态：Informative 研究备忘，不定义 HCTL2 语义。<br>
> 深度标注：本轮是**链接级 web survey**，与 methodology-landscape 的逐仓库克隆审计不同级；其中四个样本（MyContext、LobeHub context-engine、First Tree Context Tree、Cumora triage 门）已在[实现证据](../design/references/implementation-evidence.md)完成源码级深审，直接引用其结论。值得升级为克隆审计的候选在文末列出。<br>
> 触发：所有者拍板 Context 的中心设计为"萃取上下文 + 省 token"，检验标准为**快、省、准**；本备忘回答生态里各家怎么做、HCTL2 的两步管线落在地图哪里。

## 结论先行

1. **方案分四族**：相关性识别（拿哪些进上下文）、压缩（怎么变小）、长短期记忆（存哪里、怎么老化）、树形组织（协调器怎么分层）。没有一家同时做到快省准——重准的方案（时序知识图谱）建图烧 token 不省，重省的方案（小模型压缩）有损，重快的方案（本地词法检索）不深。
2. **HCTL2 已深审的四个样本恰好各占一族的最优实践**：MyContext（快+省：本地词法检索 + RRF 机械融合，零模型费用）、LobeHub（省：组装全机械、摘要是唯一显式模型步骤且持久化增量折叠）、Cumora（快+省：便宜小模型 triage 门 + 确定性地板，判定输入用账本事实不用消息措辞）、First Tree（准：双测试准入 + 有来源支撑的知识晋升）。本轮 survey 的主要新增是压缩族的小模型路线（LLMLingua 系——正是 small-brain 的现成技术形状）和长短期/树形两族的地图。
3. **HCTL2 有一个别家没有的结构优势：长期记忆不用发明。** 生态里每家都在自建第五套记忆库（Letta 的三层、mem0 的事实库、Zep 的图谱）；HCTL2 的三类数据已经回答了存哪里——结晶进 Git、content 留在各场景系统、metadata 在账本。Context 要解决的只剩"工作记忆怎么组装"，这正是所有者两步管线的范围：**① 本地快速萃取（零 API token）→ ② 可选 small-brain 压缩（缺省不压）**。

## 一、四族地图

### A. 相关性识别（拿哪些）

| 方案 | 代表 | 快省准画像 |
| --- | --- | --- |
| 结构化引用 / 显式寻址 | HCTL 组装顺序第一级；First Tree 显式 recipient + silent context（已深审） | 快✓✓ 省✓✓ 准✓（只覆盖显式部分，兜不住隐式关联） |
| 本地词法检索 + 机械融合 | MyContext：FTS5 CJK bigram 常驻零费用检索 + RRF 多路融合，无本地相关性分类器（已深审） | 快✓✓ 省✓✓ 准△（浅层匹配；作为第二级足够） |
| 小模型相关性门 | Cumora：便宜小模型做纯门判断（"该不该醒"），输入全部是数据库/Redis 事实而非消息措辞，每次判定入账本并与省下的成本做经济学对比；AI 之下垫确定性循环地板（已深审） | 快✓ 省✓✓ 准△（门判断而非内容选择；"事实做输入"是关键设计） |
| 向量 / 嵌入检索 | 通用 RAG；MyContext 本地向量索引已实现但因 embedding 费用不接线（已深审） | 快△ 省△（本地 embedding 免 API 费但吃算力）准△ |
| 时序知识图谱检索 | [Zep / Graphiti](https://arxiv.org/abs/2501.13956)：每条边带生效/失效时间窗与置信度；自称 LongMemEval 比 mem0 高 15 点、比全量塞上下文延迟低 90%；Graphiti 2026 年 20k★ | 快✓（查询侧）省✗（建图每条消息都过 LLM）准✓✓（时间维度一等） |

### B. 压缩（怎么变小）

| 方案 | 代表 | 快省准画像 |
| --- | --- | --- |
| 不压缩（基线） | 预算内原文直给，按组装顺序裁剪 | HCTL 缺省；准✓✓ 省取决于萃取质量 |
| 显式摘要作为一等对象 | LobeHub：压缩触发用本地启发式估算器，摘要是唯一显式 LLM 步骤——持久化为 DB 行、后续复用、再压缩时增量折叠而非从原文重推、用量入账（已深审）；[claude-mem](https://docs.claude-mem.ai/introduction)：观察分层压缩（近详远略），Endless Mode 把工具输出压成约 500 token 的观察、全文留档 | 快✗（要跑模型）省△（一次花钱多次省）准✓（摘要可回源时） |
| harness 原生 compaction | Claude Code 约 95% 容量自动压缩，有损；[生态共识](https://www.tembo.io/blog/claude-code-subagents)是尽量延迟它、别依赖它 | 反面基线：不可控、不可解释、不入账 |
| 小模型 token 级压缩 | [LLMLingua](https://github.com/microsoft/LLMLingua)（MIT）：小 LM 按困惑度删 token，最高 20x；[LLMLingua-2](https://www.microsoft.com/en-us/research/blog/llmlingua-innovating-llm-efficiency-with-prompt-compression/)：从大模型蒸馏出的小型双向编码器做 token 保留分类，当前有损压缩的公认强基线，2–5x 压缩区间可降约 2.9x 端到端延迟 | 快✓ 省✓✓ 准△（有损；2–5x 是安全区）——**这就是 small-brain 的现成技术路线** |
| 结构化事实抽取 | [mem0](https://arxiv.org/html/2504.19413v1)：每对消息抽取原子事实，写入前与既有记忆比对做 ADD/UPDATE/DELETE/NOOP 去重 | 省存储不省 token：每条消息都过 LLM，常态化成本 |

### C. 长短期记忆（存哪里、怎么老化）

| 方案 | 代表 | 与 HCTL2 的关系 |
| --- | --- | --- |
| OS 式分层 + agent 自编辑 | [Letta / MemGPT](https://vectorize.io/articles/mem0-vs-letta)：core（常驻上下文）/ recall（对话史可检索）/ archival（冷存）三层，agent 自己调函数决定记什么、搜什么 | 词汇可借（core≈常驻清单、recall≈content 系统里的会话史、archival≈Git 结晶）；机制是反例——记忆管理本身消耗模型回合（不省），且"agent 决定记什么"是自述（与证据立场冲突） |
| 抽取型事实库 | mem0（见上） | 同上：常态化 LLM 成本；写时去重的形状可参考 |
| 时序知识图谱 | Zep / Graphiti（见上） | HCTL 的 Snapshot 本就带 observed_at；图谱降为"派生索引候选"（可重建投影，不是权威），建图成本违反"能用规则就不用模型"，暂缓 |
| 会话观察压缩 | claude-mem：自动捕获工具使用为"观察"，压缩后按类型/概念/文件标签入库，新会话注入最近十次会话摘要 | 近详远略的分层压缩节奏可参考；自动注入"最近十次"与 HCTL 的显式来源链相悖 |
| **HCTL2 现状** | 三类数据：结晶进 Git、content 留各场景系统、metadata 在账本 | **长期记忆已有家，不引入第五套记忆库**；Context 只做工作记忆的组装 |

### D. 树形组织（协调器怎么分层）

| 方案 | 代表 | 与 HCTL2 的关系 |
| --- | --- | --- |
| 子代理上下文隔离 | [Claude Code subagents](https://www.tembo.io/blog/claude-code-subagents)：每个子代理独立上下文窗口，只回传摘要；生态实测省 40–70%；GSD"计划即给全新上下文子代理的 prompt"（methodology-landscape 已录） | HCTL 的 Run/Invocation 天然就是这个形状：每个执行者拿自己的内容包，主上下文从不被污染——**结构性省 token，已在合同里** |
| 知识树 | First Tree Context Tree：Decision Test + Durability Test 准入、有来源支撑的写入（已深审） | 已改编为 Memo 晋升门槛 |
| 检索树 | [RAPTOR](https://arxiv.org/pdf/2401.18059)：递归嵌入-聚类-摘要自底向上建树，细节与概览两级可检索；QuALITY 绝对精度 +20% | 派生索引候选，后置；建树也要跑模型 |
| 层级记忆 | Letta 三层（见 C） | 词汇对照用 |

## 二、快省准横评

| 方案 | 快 | 省 | 准 | 一句话 |
| --- | --- | --- | --- | --- |
| 结构化引用 | ✓✓ | ✓✓ | ✓ | 显式部分零成本全对，隐式关联兜不住 |
| 本地词法 + RRF | ✓✓ | ✓✓ | △ | 毫秒级零费用，深度有限 |
| 小模型 triage 门 | ✓ | ✓✓ | △ | 便宜的"该不该拿"，不是"拿什么" |
| 本地嵌入检索 | △ | △ | △ | 免 API 费但吃算力，收益不稳 |
| 时序知识图谱 | ✓ | ✗ | ✓✓ | 查询快而准，建图持续烧 token |
| 显式摘要（大模型） | ✗ | △ | ✓ | 一次花钱多次省，必须持久化+增量折叠才划算 |
| 小模型 token 压缩 | ✓ | ✓✓ | △ | 2–5x 安全区内的最省路线，有损要入账 |
| 子代理/执行者隔离 | △ | ✓✓ | ✓ | 结构性省，HCTL 已内建 |
| OS 分层自编辑 | ✗ | ✗ | △ | 记忆管理烧模型回合，且记什么靠自述 |

## 三、HCTL2 落点（对齐所有者两步管线）

**① 识别——三级阶梯，全本地、零 API token：**

1. 结构化引用（合同已有组装顺序的第一级：显式引用 → 讨论窗口 → 对象引用链）；
2. 本地词法索引（MyContext 形状：FTS5 bigram + RRF，从 chat server 事件流增量维护，可随时重建的派生投影，不是权威）；
3. 可选小模型相关性门（Cumora 形状：判定输入用账本事实——谁被提及、谁认领了什么、游标位置——而不是消息措辞；判定本身入账可审计）。

**② 压缩——缺省不压，small-brain 可选：**

- 缺省：原文直给，预算内按组装顺序裁剪；
- 用户配置专用小模型（small-brain）后启用压缩，两条技术路线都留：**摘要式**（小模型生成，采 LobeHub 的持久化 + 增量折叠 + 用量入账形状）与 **token 级**（LLMLingua-2 形状，2–5x 安全区）；
- 合同要求：压缩是清单里显式记录的一步——压缩模型 ref + 版本、压缩率、原文 digest 全部冻结，可解释性不因压缩打折；证据类内容（digest、凭证、验收标准原文）永不压缩；
- small-brain 在 Participant 七层里就是一个执行者配置里的模型 ref，不是新对象。

**不做的：** 不建第五套记忆库（Letta/mem0 式）；时序图谱与检索树降为派生索引候选、暂缓；不采用 agent 自编辑记忆（记什么靠自述，与证据立场冲突）；不依赖 harness 原生 compaction（不可控不入账，只作兜底）。

## 四、借鉴决策（五种复用决策用语）

| 决策 | 对象 |
| --- | --- |
| 适配协议 | LLMLingua-2 的压缩接口形状（small-brain 的候选实现，MIT）；MyContext 的采集/检索/降级分层（已定，见 context memo） |
| 仅参考行为 | Letta 三层词汇对照；mem0 写时去重（ADD/UPDATE/DELETE/NOOP）；Zep 的时间窗语义（Snapshot 已有 observed_at，不引图谱）；claude-mem 近详远略；RAPTOR 检索树 |
| 暂缓 | 时序知识图谱、向量库接线、检索树 |
| 反面参照 | OS 分层的"记忆管理烧模型回合"；mem0 的每消息 LLM 抽取常态成本；harness 自动 compaction 的不可控有损；agent 自编辑记忆 = 自述 |

## 五、基线与后续

链接级基线（2026-08-24 检索）：[LLMLingua 仓库](https://github.com/microsoft/LLMLingua)（MIT）、[LLMLingua-2 介绍](https://www.microsoft.com/en-us/research/blog/llmlingua-innovating-llm-efficiency-with-prompt-compression/)、[Zep 论文](https://arxiv.org/abs/2501.13956)、[mem0 论文](https://arxiv.org/html/2504.19413v1)、[RAPTOR 论文](https://arxiv.org/pdf/2401.18059)、[claude-mem 文档](https://docs.claude-mem.ai/introduction)、[Letta 对比](https://vectorize.io/articles/mem0-vs-letta)、[子代理隔离实践](https://www.tembo.io/blog/claude-code-subagents)。已深审条目见实现证据（MyContext、LobeHub、First Tree、Cumora）。

值得升级为克隆级审计的候选（按采用可能性排序）：**LLMLingua**（若 small-brain 采其路线，需审模型分发、本地推理依赖与许可链）；**claude-mem**（分层压缩与观察类型化的实现形状）；Letta/mem0/Zep 维持链接级即可（均判暂缓或仅参考）。

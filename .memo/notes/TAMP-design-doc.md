# TAMP：任务–参与者匹配与多 Harness 协作系统

**Task–Actor Matching Protocol & Multi-Participant Collaboration System**

合成设计说明书。本文不是「共识摘要」，而是把两份独立蓝图焊成一套可实现系统：

- **DeepSeek v4** 侧：人力资源管理可落地的能力管理蓝图——SFIA / e-CF / ESCO 词典、冰山模型、技能矩阵、岗位分析与关键事件法、360 / KPI / OKR、Bloom 与 Cynefin、合同网与联盟形成、信誉模型、推荐系统四件套（内容 / 协同 / 上下文 / 多目标）、技能云数据模型。
- **TAMP 侧**：数字伙伴落地时必须补上的测量与成交函数——KSAO-X 执行包络、四级证据、校准卡、Demands–Abilities + Needs–Supplies、可行性门控、轨迹四层评估、做/审分离、harness 运行时。

合成规则见 §1.4。冲突处已取舍，不并存两套真相。

| 项 | 内容 |
|---|---|
| 文档类型 | 系统设计说明书（Design Document） |
| 版本 | 0.2 |
| 状态 | Draft · Synthesis |
| 合成来源 | DeepSeek v4 能力管理蓝图 × TAMP 测量与匹配栈 |
| 适用范围 | 以软件开发生命周期为起点，可扩展至一般知识工作 |
| 读者 | 系统架构、Agent/Harness 工程、产品、评估、HR/技能治理 |

---

## 目录

1. [背景、问题与合成立场](#1-背景问题与合成立场)
2. [目标、非目标与成功标准](#2-目标非目标与成功标准)
3. [核心概念与术语](#3-核心概念与术语)
4. [设计原则](#4-设计原则)
5. [总体架构](#5-总体架构)
6. [统一词典与坐标系](#6-统一词典与坐标系)
7. [任务画像（Task Card）](#7-任务画像task-card)
8. [参与者画像（Actor Card）](#8-参与者画像actor-card)
9. [技能矩阵与组织视图](#9-技能矩阵与组织视图)
10. [证据层级、绩效通道与能力更新](#10-证据层级绩效通道与能力更新)
11. [匹配与调度](#11-匹配与调度)
12. [团队组建、合同网与信誉](#12-团队组建合同网与信誉)
13. [执行、交接与运行时](#13-执行交接与运行时)
14. [评估体系](#14-评估体系)
15. [在软件开发生命周期中的落地](#15-在软件开发生命周期中的落地)
16. [数据模型](#16-数据模型)
17. [接口与协议](#17-接口与协议)
18. [可观测性与可解释性](#18-可观测性与可解释性)
19. [治理、安全与权限](#19-治理安全与权限)
20. [人机混合协作](#20-人机混合协作)
21. [泛化到一般知识工作](#21-泛化到一般知识工作)
22. [实施路径](#22-实施路径)
23. [风险、反模式与开放问题](#23-风险反模式与开放问题)
24. [附录](#24-附录)

---

## 1. 背景、问题与合成立场

### 1.1 系统要解决什么

本系统面向一类正在快速出现的工作方式：一项知识工作不再由单一角色或单一模型完成，而是由多个 **participant（参与者 / 数字伙伴）** 协同完成。参与者可以是：

- 人类专家
- 绑定了工具、权限、记忆与验证器的 AI agent（harness 包裹下的模型）
- 更小的技能包、子代理、或人类 + AI 的组合席位

「Harness」被抽象为 **可调度的参与者运行时**：模型只是其中一层。真正决定「能不能把活干完」的是工具面、上下文、生命周期、观测、验证与治理。

软件开发生命周期是第一个垂直场景：需求分析、技术架构、环境搭建、实现、测试、运维。框架不绑定软件工程——只要工作能被拆成可验收的任务单元即可。

DeepSeek v4 把这件事说成：**动态能力匹配平台**，融合人力资源管理、任务分配、推荐系统和多智能体协作。这个定位成立，并作为本系统对外叙事。对内则必须补上数字伙伴特有的测量、校准和成交函数，否则平台会退化成技能表。

### 1.2 今天的断裂

1. **任务描述过粗**：只写「做后端」「做架构」，无法区分做与审、明确与歧义、可逆与不可逆。
2. **能力描述过虚**：Agent Card 或简历式技能列表停留在 Declared，缺少在本组织任务分布上的实测。
3. **匹配函数用错**：用向量相似度或「谁更强」排序，而不是「谁能补上这个缺口、且接得住这个活」。
4. **自评不可用**：参与者（尤其是 LLM）对自身成功率和成本校准差，直接招标会系统性配错。
5. **配完无法一起干活**：能力上最优的组合，在交接、互斥、风格、权限上不可行。
6. **词典与调度脱节**：SFIA 能和 HR 对话，却调度不了「从 OpenAPI 生成契约测试」；自建叶子能调度，却无法对人和外包解释。

### 1.3 学科归属

| 来源 | 贡献 | 主要进入本系统的位置 |
|---|---|---|
| 工作分析 / 岗位分析 | 任务清单、关键事件法、从证据反推需求 | Task Card 的生成与 Q 的学习 |
| 胜任力模型 / 冰山模型 | 显性技能与隐性行为特征分离 | Actor 的 KSAO 与 O/风格层 |
| Person–Job Fit | Demands–Abilities + Needs–Supplies | 成交函数 \(S_{cap}, S_{need}\) |
| SFIA / e-CF / ESCO | 可对外沟通的 ICT 技能语言 | 词典 L1 |
| Bloom / Cynefin | 认知层次与问题情境 | Task Card 的情境字段与策略 |
| 技能矩阵 / 技能云 | 组织级可见性与筛选 | Registry 的矩阵视图 |
| 360 / KPI / OKR | 多通道绩效回写 | 证据通道，不是唯一真相 |
| 推荐系统 | 内容、协同、上下文、多目标 | **召回与短名单**，不是成交 |
| Contract Net / 联盟形成 | 招标、组队 | 协商与团队层 |
| 信誉与信任模型 | 开放系统中的可靠性 | trust_tier 与信誉分 |
| Agent 工程 | A2A / MCP、harness、轨迹评估 | 运行时与 Measured 证据 |
| 市场式调度 | 投标、成本、成功概率 | 必须叠加校准卡 |

### 1.4 合成立场（取舍表）

两份蓝图在高层同构：双画像、标准化语言、动态更新、硬过滤 + 软排序、人机共用坐标系、可解释、先窄域验证。真正要写进系统的是下列取舍——这是合成版的核心，不是附录。

| 设计点 | DeepSeek v4 | 纯测量栈 | **本系统采用** |
|---|---|---|---|
| 对外叙事 | 动态能力匹配平台 | 协议与运行时 | 两者并存：对外用平台叙事，对内用协议 |
| 词典 | SFIA、e-CF、Bloom、Cynefin 为主 | KSAO-X + 自建叶子 | **分层词典**：L0 情境、L1 SFIA/e-CF/ESCO、L2 工作单元叶子 |
| 冰山模型 | 显性技能 + 隐性特质 | 少谈特质 | **隐喻保留，数据模型可操作化**：隐性层只存可观察行为（约束遵守、抢权、过度自信），不存无法测的「动机」 |
| 技能矩阵 | 一等工具 | 未强调 | **一等视图**，由 Actor Card 投影，不另作真源 |
| 匹配函数 | 加权余弦、协同过滤、规则 + ML | 双向缺口覆盖 | **召回用推荐与余弦，成交用门控 + \(S_{cap}/S_{need}\)**；ML 预测成功率作为特征，不单独授标 |
| 能力证据 | 问卷、基准、成功率和耗时 | 四级证据 + \(\sigma\) | **四级证据为主，问卷/基准进入 Declared/Attested** |
| 更新公式 | 贝叶斯或 EMA | \(p,\sigma\) 回写 | **EMA 或 Beta 更新 \(p\)，必须保留 \(\sigma\)** |
| 绩效 | 360、KPI、OKR | 轨迹四层 | **多通道汇入 EvidenceRecord**，权重按通道信度 |
| 分配协议 | 合同网、联盟、信誉 | 校准后市场、做/审分离 | **合同网做协议，校准卡做报价，政策做分离，联盟做组队** |
| 人机差异 | 冰山特质映射到模型行为 | 执行包络 X | **X 必填**；行为特征进 O，不替代 X |
| 实施顺序 | 词汇表 → 标注 → 画像 → 引擎 → 扩展 | 切片闭环再长词典 | **先切片闭环，但切片内必须先有 L1 子集 + L2 叶子** |

一句话：

> DeepSeek v4 解决「用什么词描述、用什么组织过程运转」；测量栈解决「为什么不能用像不像来成交、以及配完为什么翻车」。合成版两层都做，并且规定哪一层有权做最终决定。

---

## 2. 目标、非目标与成功标准

### 2.1 目标

1. 用同一套分层语言描述任务需求与参与者能力，人与 AI 可互换比较，且能导出给 HR / 外包。
2. 对任务做多角度刻画：技能、认知、情境、协作结构、工具、风险、约束。
3. 对参与者做分级证据评估，并维护组织级技能矩阵。
4. 按双向拟合成交：覆盖缺口，同时满足参与者约束。
5. 支持单点匹配、团队组建、做/审分离、合同网协商、动态重配。
6. 每次执行经多通道回写证据，画像与匹配政策可进化。
7. 匹配结果可解释、可审计、可人工覆盖。
8. 先在 SDLC 打透，再以同一框架扩展到其他知识工作。

### 2.2 非目标

- 不一次建成覆盖所有职业的十万级技能本体。
- 不替代项目管理工具或源代码平台。
- 不以人格测验或价值观匹配作为主排序轴。
- 不假设参与者能准确报告自己的成功率与成本。
- 不在 v1 实现开放 agent 市场与链上结算。
- 不对模型做预训练；调度与评估的是 *model–harness–权限包* 版本。
- 不把冰山模型的「动机 / 价值观」写成必填数据库字段。

### 2.3 成功标准

**匹配质量**

- 相对基线（人工指定、纯余弦、纯最强模型、纯合同网裸报价）提升一次通过率、降低返工率。
- R2+ 任务 100% 满足做/审分离与权限门控。
- 解释中指出的缺口，经扰动后确实改变决策。

**校准与绩效**

- 自报成功率 / 成本的校准误差随样本下降。
- 多通道（轨迹、360、KPI）不一致时能被发现，而不是被平均掉。

**协作与组织**

- 交接丢失、重复劳动、互斥冲突可单独计量并下降。
- 「配了但无法开工」比例下降。
- 技能矩阵可按项目导出，人类参与者看得懂自己的叶子与 SFIA 映射。

**运营**

- 新参与者在有限探针预算内进入可调度池。
- Prompt / 模型 / 工具 / 权限变更都能跑回归门禁。

---

## 3. 核心概念与术语

| 术语 | 定义 |
|---|---|
| **Participant / Actor** | 可被调度的工作主体。人类、AI、技能包、人机席位。 |
| **Harness** | 把模型变成可执行参与者的运行时。 |
| **Task Unit** | 可验收的工作原子，不是岗位，也不是阶段本身。 |
| **Task Card** | 任务画像。 |
| **Actor Card** | 参与者画像。真源。 |
| **Skill Matrix** | Actor × 叶子熟练度的组织视图，由 Actor Card 投影。 |
| **Need / Offer** | 参与者约束与任务供给。 |
| **Demands–Abilities Fit** | 任务要求 vs 能力。 |
| **Needs–Supplies Fit** | 参与者需求 vs 任务供给。 |
| **Feasibility Gate** | 硬约束过滤。 |
| **Recall / Rank / Award** | 召回、排序、授标。三阶段不得合并成一次余弦。 |
| **Probe** | 测量用任务。 |
| **Trajectory** | 一次执行的完整轨迹。 |
| **Evaluator** | 与执行者分离的评价者。 |
| **Reputation** | 跨任务族的可靠性汇总，受证据层级约束。 |
| **Contract Net** | 招标–投标–授标–汇报协议。 |
| **Coalition** | 为覆盖缺口而组成的临时团队。 |

---

## 4. 设计原则

1. **先分析工作，再谈人。** 没有 Task Card 的雷达图是装饰。
2. **词典分层，而不是选一个全国通用表。**
3. **同一坐标系。** 需求与能力必须能逐项对齐。
4. **召回求全，成交求缺口。**
5. **双向拟合。** 只看「他会不会」会配出接不住的活。
6. **声明不是证据。** Declared → Attested → Measured → Verified。
7. **自评必须校准。**
8. **做与审默认按风险分离。**
9. **互补优于同类堆叠。**
10. **行为锚点，不用形容词。**
11. **技能矩阵是视图，Actor Card 是真源。**
12. **绩效多通道，权重按信度，禁止单 KPI 暴政。**
13. **评估进 CI。** Prompt、工具 Schema、权限都是代码。
14. **人有最终覆盖权。**
15. **问题驱动长词典。**
16. **评价 model–harness 对，不评价裸模型名。**

---

## 5. 总体架构

```
┌─────────────────────────────────────────────────────────────┐
│  Workbench / 编排台                                          │
│  任务拆解 · 技能矩阵 · 人工覆盖 · 解释 · 审批 · OKR 视图       │
├─────────────────────────────────────────────────────────────┤
│  Matching & Staffing                                         │
│  多路召回 · 门控 · 双向打分 · 多目标 · 合同网 · 联盟          │
├─────────────────────────────────────────────────────────────┤
│  Registry                                                    │
│  词典 L0–L2 · Task / Actor Cards · 矩阵投影 · 校准 · 信誉    │
├─────────────────────────────────────────────────────────────┤
│  Execution Fabric                                            │
│  Harness · MCP · A2A · 工作区 · 钩子 · 交接信封              │
├─────────────────────────────────────────────────────────────┤
│  Evidence & Eval                                             │
│  探针 · 轨迹 · 360/KPI 通道 · 分层评估 · 回归门禁            │
├─────────────────────────────────────────────────────────────┤
│  Governance                                                  │
│  权限 · 审计 · 风险政策 · 数据驻留 · 人工在环                 │
└─────────────────────────────────────────────────────────────┘
```

主数据流：

1. 工作分析（模板 + 关键事件 + 可选自动拆解）得到 Task Graph 与 Task Card。
2. Registry 维护词典、Actor Card、技能矩阵投影、信誉。
3. Matcher 多路召回 → 门控 → 双向分 → 多目标短名单。
4. 需多人则联盟形成 + 合同网授标。
5. Fabric 按最小权限拉起 harness。
6. Evidence 多通道回写 \(p,\sigma\)、矩阵、信誉与任务需求 Q。
7. 失败与范围蔓延触发重配。

---

## 6. 统一词典与坐标系

### 6.1 三层词典（合成的骨架）

```
L0  情境        Cynefin · 信息状态 · Bloom 认知层次 · 风险级
L1  对外技能    SFIA 8 · e-CF · ESCO（跨域扩展时）
L2  调度叶子    本组织 Task Unit / 能力叶子（带行为锚点）
```

- L0 决定 **策略**：简单问题走规则，复杂问题留探索预算，混沌问题先稳定再匹配。
- L1 决定 **沟通**：和人、HR、采购、外包说同一种 ICT 语言。
- L2 决定 **调度**：匹配器只对叶子打分。

L2 每条叶子必须映射到至少一条 L1（SFIA 技能码或 e-CF 能力码），否则不能进入对外导出。允许一对多。

### 6.2 SFIA（L1 主词典，IT / 软件优先）

采用 SFIA 作为软件与数字工作的主对外词典。本系统不改写 SFIA 定义，只做两件事：映射到内部 4 级、把等级变成可调度的门槛。

SFIA 七级（保留原码）：

| SFIA | 名称（意译） | 内部 4 级 | 调度含义 |
|---|---|---|---|
| 1 | Follow | Aware | 通常不独立授标 |
| 2 | Assist | Working | 可在模板与评审下执行 |
| 3 | Apply | Working / Proficient | 常规变体可独立执行 |
| 4 | Enable | Proficient | 可设计方法并带他人 |
| 5 | Ensure / Advise | Expert | 可设标准；人优先，agent 需 Verified |
| 6 | Initiate / Influence | Expert + 策略标记 | 默认人类或混合席位 |
| 7 | Set strategy | Expert + 策略标记 | 不自动配纯执行 agent |

软件向常用 SFIA 类别（起步子集，不是全集）：

- 策略与架构：企业架构、解决方案架构、数据架构、信息安全
- 变更与转型：需求分析、业务分析
- 开发与实施：编程/软件开发、系统集成、测试、数据库设计
- 交付与运维：发布与部署、事件管理、问题管理、基础设施
- 技能与质量：质量保证、方法与工具、知识管理
- 人际：关系管理、利益相关方管理（人类权重高）

每个 L2 叶子写 `sfia_ref: PROG/5` 这种引用。匹配对内仍用叶子的 \(p\)，对外展示 SFIA 码。

### 6.3 e-CF 与 ESCO

- **e-CF**：40 项 ICT 能力 × 5 级（e-1 到 e-5），用于欧洲语境或与 e-CF 已对齐的组织导出。与 SFIA 双写，不双算分数。
- **ESCO**：当系统越出 ICT、进入更广知识工作时，L1 增加 ESCO 职业/技能 URI。SDLC 阶段不必强行上 ESCO。

参考技能云产品（Workday Skills Cloud、IBM Watson Talent）的做法：L1 是受治理的受控词表，L2 是组织私有扩展，扩展要有负责人与过期策略。

### 6.4 Bloom：任务认知标签，不是人的量表

Bloom 修订版六级只标在 Task Card 上：

| 级 | 用于何种任务 | 匹配含义 |
|---|---|---|
| Remember / Understand | 检索、摘要、复述现状 | 可低熟练度 + 强工具 |
| Apply | 按既有模式实现 | Working 即可 |
| Analyze | 冲突检测、根因、权衡拆解 | 要 Evaluation 轴 |
| Evaluate | ADR 评审、代码审、方案否决 | 强制独立评价者倾向 |
| Create | 新架构、新协议、新问题定义 | 复杂情境 + 探索预算 |

禁止把 Bloom 级直接写成 Actor 的能力分。同一人可以 Apply 很强而 Create 很弱；那是不同叶子，不是一个 Bloom 分。

### 6.5 Cynefin：情境决定策略，不只是标签

| 域 | 识别信号 | 本系统策略 |
|---|---|---|
| Clear（简单） | 规则已知、验收确定 | 规则优先；agent 可高自动 |
| Complicated（繁杂） | 需要专家分析，因果事后可讲清 | 配 Proficient+；允许分析时间 |
| Complex（复杂） | 因果事后才显、需探测 | 小探针 + 迭代；留探索预算；禁止一次授死 |
| Chaotic（混沌） | 先要止血 | 先稳定（runbook / 人类），再重新出 Task Card |
| Confusion | 域未判定 | 先配「情境分类」小任务，不得直接配实现 |

Cynefin 域变化必须重算匹配。从 Complex 被误标成 Clear，是一类高危事故。

### 6.6 KSAO-X

叶子挂到同一结构：

| 维 | 含义 | 例子 |
|---|---|---|
| **K** | 领域知识 | 支付清算、HIPAA、服务边界 |
| **S** | 程序性技能 | 写迁移、威胁建模、用户故事地图 |
| **A** | 较稳定能力 | 长程规划、歧义消解、工具可靠性 |
| **O** | 可观察风格与约束特征 | 同步偏好、保守/探索、抢权、过度自信 |
| **X** | 执行包络 | 工具、权限、上下文、验证器、成本、并发、恢复 |

对应冰山模型的可操作化：

```
水面以上   K, S, 部分 X     ← 声明、证书、工具清单
水面以下   A, O, 校准、信誉  ← 只能从轨迹与 360 推断
运行时     X                ← 冰山没有、数字伙伴必须有
```

「价值观 / 动机」不入库。能入库的是：是否遵守 write_scope、是否改写验收标准、失败后是否扩大范围。

### 6.7 内部熟练度与行为锚点

内部统一 4 级。SFIA 7 级与 e-CF 5 级都映射到这 4 级，原码保留。

| 级 | 名称 | 锚点原则 |
|---|---|---|
| 1 | Aware | 能识别概念，不能独立交付 |
| 2 | Working | 模板和评审下可交付 |
| 3 | Proficient | 常规变体可独立交付 |
| 4 | Expert | 可处理冲突约束，并能评价他人 |

示例：

> Proficient · 威胁建模：能对给定服务边界列出 STRIDE 中至少四类具体威胁，给出可验证缓解与残留风险，且不把「用 HTTPS」当作唯一控制。

没有行为锚点的条目不得进入成交权重。

---

## 7. 任务画像（Task Card）

任务不是岗位说明书。Task Card 描述 **这一次** 的工作单元。DeepSeek v4 的七维需求全部保留，并补上做/审、协调结构和信息状态。

### 7.1 任务需求的完整维度

| # | 维度 | 来源 | 作用 |
|---|---|---|---|
| 1 | 知识领域 Domain | DS v4 | L1/L2 知识叶子、业务域 |
| 2 | 技能类型 Skill | DS v4 | S/L2 叶子 |
| 3 | 认知复杂度 | DS v4 Bloom | 标签 + 评价器选择 |
| 4 | 问题情境 | DS v4 Cynefin | 策略与探索预算 |
| 5 | 协作需求 | DS v4，细化 | 协调结构：分配 / 顺序 / 互斥 / 交接 |
| 6 | 工具与环境 | DS v4 | X 的需求侧 |
| 7 | 风险与不确定性 | DS v4 | 风险级、信息状态 |
| 8 | 时间与资源约束 | DS v4 | Offers 与门控 |
| 9 | 做 vs 审 | 测量栈 | Execution / Evaluation 比例 |
| 10 | 验收与过程需求 | 合成 | 产物、解释、人类在环 |

### 7.2 工作分析如何生成 Task Card

三种输入，可叠加：

1. **任务清单 / 模板**：SDLC 各阶段的 Task Unit 模板，项目经理或系统实例化。
2. **关键事件法（Critical Incident）**：从历史事故、返工、优秀交付提取「决定成败的行为」，反写叶子与锚点。这是胜任力建模的正统做法，也是词典生长的主来源。
3. **轨迹挖掘**：上线后从失败归因更新 Q（任务需求向量），相当于把工作分析自动化。

v0 允许人工填模板。系统不接受无验收任务进入自动匹配。

### 7.3 字段示例

```yaml
task_card:
  id: tsk_...
  title: 为结算服务撰写 ADR：消息队列选型
  goal: 产出可执行的架构决策，包含否决项
  taxonomy:
    sfia: [ARCH, DESN]
    bloom: evaluate
    cynefin: complicated
    information_state: conflicting
  graph:
    parents: [tsk_req_settlement_nfr]
    children: [tsk_impl_publisher]
    mutex_with: []
    handoff_to: [tsk_review_adr]
  acceptance:
    type: rubric
    artifacts: [ADR.md, decision_log.json]
    criteria:
      - 至少比较 3 个方案
      - 明确与现有 outbox 的兼容性
      - 给出回滚与迁移代价
  demands:
    - leaf_id: adr.writing
      min_level: 3
      weight: 0.25
      hard: false
    - leaf_id: messaging.tradeoff
      min_level: 3
      weight: 0.35
      hard: true
  offers:
    timebox_hours: 8
    budget_tokens: 2.0e6
    workspace: repo:settlement
    human_reviewer: required
    learning_value: medium          # 可供培养性匹配
  constraints:
    risk_level: R2
    write_scope: none
    production_access: false
    must_separate_evaluator: true
  work_mode:
    execution_vs_evaluation: 30/70
    coordination: [sequential, handoff]
  context:
    domain: payments
    stack: [java, kafka, postgres]
```

### 7.4 任务图

阶段是导航，调度单位是图：

- 边：`depends_on` / `mutex` / `handoff` / `reviews`
- 拆分：一种主验收物一个节点；做与审拆开；不同权限拆开；超过一个工作时段继续拆

Cynefin = Complex 的图应包含显式探测节点，禁止把探测和一次性交付画成同一节点。

---

## 8. 参与者画像（Actor Card）

### 8.1 版本即身份

```
actor_id + harness_id + model_id + toolset_hash + policy_hash → actor_version
```

换模型、MCP、系统提示、权限都升版本。旧成绩只作先验。

### 8.2 字段

```yaml
actor_card:
  id: act_architect_adr_01
  kind: human | agent | hybrid_seat | skill_pack
  version: 2026.09.04+h3
  identity:
    display_name: ADR Architect
    owner_team: platform-arch
  capabilities:
    - leaf_id: adr.writing
      sfia_ref: ARCH/4
      level_p: 0.78
      sigma: 0.12
      evidence: measured
    - leaf_id: messaging.tradeoff
      sfia_ref: DESN/4
      level_p: 0.64
      sigma: 0.21
      evidence: attested
  needs:
    max_cost_usd: 12
    max_latency_min: 90
    sync_style: async
    requires_human_pair: false
    concurrency_limit: 2
    development_goals: [threat.modeling]   # 人类培养；agent 通常为空
  execution_envelope:                      # X，AI 必填
    tools: [repo_read, mermaid, search]
    permissions: [contents:read]
    context_budget_tokens: 200000
    verifier: rubric.adr.v2
    recoverable: true
  observable_style:                        # 冰山水下，可观察
    conservative: 0.7
    scope_expansion: 0.22
    overconfidence: 0.31                   # 自报 - 实测
    handoff_success: 0.91
  calibration:
    domain: architecture.adr
    reported_success_ece: 0.19
    cost_mape: 0.47
    n: 38
  performance_channels:
    kpi_on_time: 0.84
    okr_contrib: "架构决策周期下降"
    feedback_360:
      peers: 0.80
      reviewers: 0.86
  reputation:
    score: 0.81
    n: 38
    decay: 90d
  load:
    active_tasks: 1
    queue_eta_min: 40
  trust_tier: measured
```

### 8.3 Needs 是一等公民

成本、时延、同步性、是否要人类搭档、可接受风险级、负载、工作区亲和、学习目标。没有 Needs 就只是单向用工。

人类的 `development_goals` 进入 Needs–Supplies：低风险任务允许标记 `match_reason=development` 的次优匹配。R2+ 默认关闭。

### 8.4 四类 Actor

| 类型 | 能力来源 | 特殊点 |
|---|---|---|
| 人类 | 证书、交付、360、可选探针 | 工时、时区、注意力、发展目标、SFIA 对外职级 |
| Agent | 探针、轨迹、基准、工具包 | X 必填、校准卡必填 |
| 混合席位 | 两者叠加 | 人批步骤是图上硬节点 |
| 技能包 | 窄域 Measured | 不能独立接复杂图 |

---

## 9. 技能矩阵与组织视图

DeepSeek v4 把技能矩阵当作筛选工具。本系统采纳，但规定真源。

### 9.1 定义

技能矩阵是投影：

```
rows    = ActorVersion（可按人聚合最新版本）
columns = L2 叶子（可按 SFIA 类别折叠）
cell    = (level_p, sigma, evidence_tier, last_seen)
```

另外提供折叠视图：按 SFIA 技能码聚合，供 HR 与管理层。

### 9.2 用途

- 硬筛选：「谁在 `PROG` 上至少 Working 且 Measured」
- 缺口热力图：项目 Task Graph 的 Q 对当前团队矩阵做差
- 培养计划：development_goals × 低风险任务
- 冷启动规划：哪一列 \(\sigma\) 普遍大，就优先建探针

### 9.3 禁止

- 不在矩阵单元格里手填一个与 Actor Card 冲突的分数
- 不用过期超过窗口且无新证据的单元格做 R2+ 成交
- 不把矩阵导出当绩效排行榜（避免刷分）

---

## 10. 证据层级、绩效通道与能力更新

### 10.1 四级证据

| 级 | 名称 | 来源 | 默认可用于 |
|---|---|---|---|
| 0 | Declared | 自评、Agent Card、简历、问卷 | 仅召回 |
| 1 | Attested | 外部基准（含 HELM 类能力套件）、证书、SFIA 认证、他人背书 | 低风险软排 |
| 2 | Measured | 本系统探针与真实轨迹 | 常规成交 |
| 3 | Verified | 对抗、权限边界、回归、独立评价者一致 | 高风险成交 |

R2+ 不得仅凭 Declared 成交。

### 10.2 多通道绩效（吸纳 360 / KPI / OKR）

DeepSeek v4 建议用绩效评估持续更新能力。采纳为通道，而不是单一分数。

| 通道 | 测什么 | 默认权重 | 风险 |
|---|---|---|---|
| 轨迹 / 验收 | 这次是否做成、过程是否可审 | 高 | 需好的验收器 |
| 确定性检查 | 测试、schema、权限 | 高（局部） | 开放任务过脆 |
| 同行 360 | 协作、交接、沟通 | 中 | 人情与互抬 |
| 评审者评分 | 质量 | 中高 | 评价器偏差 |
| KPI | 按时率、返工率、事故率 | 中 | 短视、拒绝难活 |
| OKR 贡献 | 是否对准目标 | 低到中 | 难归因到叶子 |
| 自评 / 投标 | 成功概率、成本 | 低，需校准 | 系统性误校准 |

每条通道写入 `EvidenceRecord`，带通道信度。更新叶子时按信度加权，不允许 KPI 直接覆盖 Measured 轨迹。

360 对人类默认启用，对 agent 翻译为「协作对象评分」（交接信封质量、是否抢权）。同模型家族互评需降权或交叉家族锚点。

### 10.3 探针

- **Capability probes**：打还不稳的点，允许低通过率。
- **Regression probes**：版本变更必过，失败降信任。

AI 参与者的 Attested 层可引用公共基准（HELM、BigBench、SWE-bench 等）作为先验，但 **公共基准不能替代本组织探针**。分布外是常态。

### 10.4 更新公式

对叶子 \(j\)、参与者 \(v\)、观测 \(x \in [0,1]\)、信息量 \(w\)（任务权重 × 相关性 × 通道信度）：

\[
p \leftarrow p + \alpha w (x - p)
\]

\[
\sigma \leftarrow \sqrt{(1-\beta w)\sigma^2 + \beta w (x-p_{\text{old}})^2}
\]

等价地可用 Beta-Binomial。实现选一种并冻结。

规则：

- Verified 失败的更新幅度大于 Declared 成功
- 时间衰减：长期不使用的叶子 \(\sigma\) 回升，而不是 \(p\) 永久冻结
- 版本升级：新版本继承收缩后的先验，不继承旧 \(\sigma\) 的「我已经很确定」

### 10.5 校准卡

每个 Actor × 任务族：

- 自报成功率 vs 实测（ECE / Brier）
- 自报成本 vs 实测（MAPE）
- 样本数与衰减

协商读取：

\[
\hat{p}_{\text{eff}} = \mathrm{recalibrate}(\hat{p}_{\text{self}}, \text{calibration card})
\]

无校准卡则保守折减并加大 \(\sigma\)。这是对 DeepSeek v4 合同网的硬补丁：协议可用招标，报价必须过校准。

---

## 11. 匹配与调度

匹配是四段，不是一次相似度。

```
Recall → Gate → Rank → Award
```

### 11.1 多路召回（DeepSeek v4 推荐系统的位置）

四路召回并集，截断为候选池（如 Top 50）：

1. **内容 / 倒排**：硬叶子、风险级、权限族、SFIA 码。
2. **加权余弦 / 向量**：Task Card 文本与需求向量 vs Actor 摘要与能力向量。权重来自任务维重要性；可用 AHP 给「技能 / 风险 / 工具」定权，但只用于召回。
3. **协同过滤**：历史上相似 Task Card 最终成功的 Actor。冷启动弱，作辅路。
4. **上下文感知**：当前负载、时区、工作区热度、变更窗口、可用人类评审。

召回阶段允许「看起来像」。此阶段的余弦 **不得** 写成最终分。

### 11.2 可行性门控 \(\sigma \in \{0,1\}\)

- 硬技能门槛
- 权限不足或过权
- 许可与数据驻留
- 风险级超出授权
- 上下文 / 工具包装不下必要工件
- 同步任务的时间窗不可行
- 政策要求分离时，评价者与执行者同一版本

失败返回机器可读原因，供拆任务或升级权限。

### 11.3 成交函数：双向拟合

缺口：

\[
\mathrm{Gap}_j = \max(0, q_j - \tilde{p}_{u,j})
\]

冷启动乐观可达：

\[
c_{v,j} = \min(1, \tilde{p}_{v,j} + \kappa_t \tilde{\sigma}_{v,j})
\]

\[
S_{\mathrm{cap}} = \frac{\sum_j w_j \min(c_{v,j}, \mathrm{Gap}_j)}{\sum_j w_j \mathrm{Gap}_j + \varepsilon}
\]

\[
S_{\mathrm{need}} = \sum_k \pi_k \cdot \mathrm{satisfy}(\mathrm{offer}_k, \mathrm{need}_k)
\]

\[
M=\sigma\big[\lambda S_{\mathrm{cap}}+(1-\lambda)S_{\mathrm{need}}\big]
\]

\(\lambda\)：评审 / 合规 / 生产变更偏高；探索 / 培养偏低。

可选过程分 \(S_{\mathrm{proc}}\)（交接史、风格、时间可行性、过程预演）只在短名单重排：

\[
R = \eta M + (1-\eta) S_{\mathrm{proc}}
\]

### 11.4 规则 + ML（DeepSeek v4 的第三种算法）

保留，但降级为特征与辅助：

- 规则：即门控与政策（分离、过权、R3 必须有人）。
- 模型：用历史 `(Task Card, Actor Card, context) → 成功 / 成本 / 返工` 训练成功预测器 \(\hat{s}\)。
- \(\hat{s}\) 可进入短名单的次级排序或探索值，**不能单独授标**。
- 特征必须可解释；禁止只输出黑盒 0.87。

### 11.5 多目标，而不是单标量

短名单上同时呈现：

- 缺口覆盖 \(S_{cap}\)
- 供给满足 \(S_{need}\)
- 预期成本与时延
- 风险与分离合规
- 负载
- 培养价值（可选）
- 团队多样性（组队时）

默认政策给一个加权 \(R\) 便于自动授标；Workbench 必须能看到帕累托拆解。这对应 DeepSeek v4 的「能力适配 + 负载均衡 + 成本 + 学习机会」。

### 11.6 解释

必须拆到条目：覆盖哪条缺口、哪条门控差点拒绝、哪条 Need 只是软匹配、相对候选 B 差在哪。无法分解的「综合 87 分」不得作为唯一对外输出。

---

## 12. 团队组建、合同网与信誉

### 12.1 何时组队

单 Actor 覆盖不了硬缺口；政策要求做/审分离；必须并行的互斥技能；时延超过 SLA。

### 12.2 联盟形成（Coalition）

目标向量：

1. 缺口覆盖
2. 互补 / 多样性（避免全是规划者）
3. 协调代价（人数、交接边、时区）
4. 总成本与总风险

先枚举或启发式生成可行联盟（每人有并发上限），再出帕累托短名单。v1 不求全局最优。

做/审分离：

| 风险级 | 执行者 | 评价者 |
|---|---|---|
| R0 | 可同一 Actor | 确定性检查即可 |
| R1 | 可同一 Actor + 独立 rubric | 抽检 |
| R2 | 必须不同 Actor 版本 | 人 or Verified evaluator |
| R3 | 必须不同 Actor，评价者含人 | 人 + 验证器 |

### 12.3 合同网

协议采用 Contract Net，语义改为带校准投标：

1. **Announce**：Task Card 公开子集（不含隐藏探针）。
2. **Bid**：是否接受、\(\hat{p}_{\text{eff}}\)、成本、最早开始、依赖条件、所需工具权限。
3. **Award**：按政策在 \(R\)、负载、分离、多样性上授标。
4. **Report**：心跳、完成、失败、请求重配。
5. **Close**：验收结果写入证据与信誉。

多轮拍卖仅在等价候选争用稀缺任务时启用。v1 用内部积分，满足个体理性即可，不上真实货币。

禁止用未校准自报成功率作为唯一授标依据。

### 12.4 信誉与信任

开放或半开放参与者池需要信誉，但不替代叶子能力。

```
reputation = decayed_mean(task_success, handoff_success, constraint_obedience)
             × trust_tier_factor
```

- 信誉低：召回降权，或只能接 R0/R1
- 信誉高：不能免探针，不能跳过门控
- 新版本信誉收缩，防止「换皮继承神分」

Trust tier（Declared…Verified）是硬门槛；reputation 是软排序。两者都要，不合并成一个「可信分」糊弄过去。

### 12.5 重配

循环、连续工具失败、范围蔓延、SLA 击穿、主动弃权、评价者判定方向错但仍可救。重配保留工件与已确认结论，写入新 Task Card context。

---

## 13. 执行、交接与运行时

### 13.1 Harness 分层（X 的运行时展开）

| 层 | 职责 |
|---|---|
| Execution | 有界迭代、暂停/恢复、超时 |
| Tooling | MCP、内部 API、仓库/终端 |
| Context | Task Card、工件、压缩与记忆 |
| Lifecycle | 写前审批、失败恢复、交接 |
| Observability | 轨迹、成本、工具审计 |
| Verification | 测试、schema、rubric |
| Governance | 权限、出域、策略即代码 |

改任一层即升 Actor 版本。

### 13.2 工作区

一任务一隔离区。代码走分支；文档限路径；运维写操作走变更单。长任务用工件交接，禁止只靠上下文续跑。

### 13.3 交接信封

```yaml
handoff:
  from: act_impl_01
  to: act_review_01
  intent: 按验收标准审这个补丁
  artifacts: [diff, test_log, residual_risks]
  claims:
    - 只改了 billing
    - 新增 3 个回归测试
  open_questions:
    - 税率舍入是否保持旧行为
  constraints_respected: [write_scope, no_secrets]
```

接收方先验信封。claims 与 diff 不一致记交接失败，回写双方 `handoff_success`，并进入 360 通道。

### 13.4 范围控制

R2+ 必须可被外部终止。超出 write_scope、未授权重构、执行者改写验收标准 → 拦截或重配。

---

## 14. 评估体系

### 14.1 四种评估器

| 类型 | 用途 |
|---|---|
| 确定性 | 测试、schema、工具参数、权限 |
| 模型评价 | rubric、成对比较；必须与人校准 |
| 人类专家 | 黄金标准、R3、rubric 校准 |
| 过程模拟 | 预估能否一起干活，不能替代实测 |

禁止执行者自评自判作为 Verified。

### 14.2 四层评分

1. Component：工具、参数、角色边界、格式
2. Trajectory：必要性、循环、交接、重复劳动
3. Task：验收、事实错误、残留风险
4. System：分配、顺序、互斥、交接节律

### 14.3 能力评估 vs 回归评估

能力评估指导上限，通过率可以低。  
回归评估保护已会的不退，进 CI，失败阻断该版本调度。

### 14.4 对 AI 参与者的基准位置

HELM / BigBench / SWE-bench 等进入 **Attested 先验** 与探针设计参考，不进入 Measured。Measured 只承认本组织任务分布上的轨迹。

### 14.5 匹配器也要评估

- 离线回放：当时可见信息 vs 事后最优
- 在线小流量探索（UCB / ε）降 \(\sigma\)
- 解释忠实度扰动
- 基线对比：人工、纯余弦、纯最强模型、裸合同网

---

## 15. 在软件开发生命周期中的落地

阶段用于组织模板，调度仍按 Task Unit。

### 15.1 阶段矩阵

| 阶段 | 典型 Task Unit | 高权能力 | Bloom 常标 | Cynefin 常标 | 风险 |
|---|---|---|---|---|---|
| 需求 | 访谈→冲突表；用户故事+可测验收 | 歧义消解、领域、提问 | Analyze | Complex / Complicated | R1 |
| 架构 | ADR；威胁模型；容量 | 权衡、评价、跨切面 | Evaluate / Create | Complicated | R2 |
| 环境 | 可复现环境；权限基线 | 幂等、IaC、排障 | Apply | Complicated | R2 |
| 实现 | 切片补丁；API | 最小 diff、测试、仓库熟悉 | Apply | Clear / Complicated | R1–R2 |
| 测试 | 边界用例；回归归因 | 评价、变异 | Analyze / Evaluate | Complicated | R1 |
| 运维 | 告警归因；变更；回滚 | 时间压力工具链、可逆性 | Apply / Analyze | Complicated / Chaotic | R2–R3 |

混沌态事故先走止血 runbook（人类或 Verified 运维席位），再出「归因 / 修复 / 复盘」三张 Card。

### 15.2 模板：实现切片

```
Demands: 仓库熟悉, 测试设计, 最小变更, 接口兼容
Bloom:   apply
Cynefin: complicated
Mode:    execution 70 / evaluation 30
Coord:   sequential + handoff to reviewer
Gate:    不得碰迁移与密钥
Offer:   热工作区、复现测试、4h timebox
```

### 15.3 模板：需求冲突消解

```
Demands: 领域, 提问, 利益方映射, 验收可测性
Bloom:   analyze
Cynefin: complex
Info:    conflicting
Mode:    execution 40 / evaluation 60
Gate:    禁止直接改代码；列出未决问题
Offer:   可约访谈的人类席位
```

### 15.4 第一批 L2 叶子

需求：访谈结构化、冲突检测、验收可测性、范围裁剪  
架构：ADR、技术权衡、威胁建模、容量与成本  
实现：最小补丁、API 契约、数据迁移（单独叶）、重构克制  
测试：用例设计、失败归因、回归选择、契约测试  
运维：观测查询、变更单、回滚、事故时间线  
通用：工具可靠性、长程规划、交接信封、约束遵守

每条映射 SFIA，写四档锚点。先这些，再按关键事件增词。

---

## 16. 数据模型

真源实体：

- `TaxonomyLeaf`：叶子、锚点、SFIA/e-CF/ESCO 映射、负责人、过期
- `SFIANode` / `ECFNode`：L1 受控词
- `TaskTemplate` / `TaskCard` / `TaskGraph`
- `ActorCard` / `ActorVersion`
- `SkillMatrixCell`：物化视图
- `EvidenceRecord`：通道、叶子、x、w、轨迹引用
- `CalibrationCard`
- `ReputationSnapshot`
- `MatchDecision`：召回理由、门控、分数分解、覆盖
- `Bid` / `Assignment`
- `Trajectory` / `HandoffEnvelope`
- `EvalSuite` / `EvalRun`
- `Policy`
- `OKRLink`（可选，弱归因）

参考技能云产品时只借鉴「受控词表 + 组织扩展 + 证据」，不复制其岗位 HR 全栈。

标识：

```
tax_leaf:sdlc.arch.adr.writing
sfia:ARCH/4
task:org/proj/tsk_...
actor:org/act_...@v...
match:org/mch_...
run:org/run_...
```

---

## 17. 接口与协议

对内逻辑 API：

- `POST /tasks` 从模板或关键事件实例化
- `POST /tasks/{id}/match` 短名单 + 分解 + 召回路由
- `POST /tasks/{id}/announce` 合同网招标
- `POST /bids`
- `POST /assignments`
- `POST /runs/{id}/handoff`
- `POST /runs/{id}/eval`
- `POST /evidence` 写入 360 / KPI / 轨迹
- `GET /matrix`
- `POST /actors/{id}/probes`
- `POST /overrides`

外部：

| 协议 | 用途 |
|---|---|
| A2A Agent Card | Declared 入口 |
| MCP | 工具与资源 |
| 内部 UCP | 统一能力剖面 |
| SCM / CI | 工作区与回归 |
| ITSM | R2+ 写操作 |
| HRIS（可选） | 导出 SFIA 视图，不倒流未校准自评 |

招标只发 Task Card 公开子集。

---

## 18. 可观测性与可解释性

因果链：

```
Work Analysis → TaskCard → Recall → Gate → Rank → Award
    → Trajectory → Multi-channel Eval → EvidenceUpdate → Matrix / Reputation
```

必须能回答：为什么选 A 不选 B；当时 \(p,\sigma\)；哪一步偏离；回写了哪条叶子；人工覆盖是否改善结果；召回是余弦还是协同还是倒排。

三类消费者：调度器看分解；负责人看缺口与风险；审计看不可变日志。

---

## 19. 治理、安全与权限

- 按任务 Offers 签发短时凭证；过权与欠权都拒绝。
- 密钥不得进入可导出轨迹。
- 出域与驻留按风险级。
- 政策即代码：R3 必须有人；生产写必须变更单；未 Measured 不得接 R2+。
- 监控：技能声明膨胀、拒接难活刷 KPI、同家族互抬、过程分把弱候选抬进 R2。

词典治理：L1 变更走评审；L2 由领域负责人增删；无证据且过期的叶子降权下线。

---

## 20. 人机混合协作

- 人类是 Actor，拥有完整 Actor Card，不是系统外插件。
- 人类同时是治理角色：改派、签核、校准 rubric、360。
- 混合席位把「人必须出现」写成图上硬节点。
- 对人解释用缺口与风险语言，少堆 \(\sigma\)。
- 人的负载按注意力片段，不按 token。
- 培养性匹配显式标记并计入配额。
- SFIA 5–7 的策略型工作默认人主、agent 辅。

---

## 21. 泛化到一般知识工作

可迁移条件：能拆验收单元、能写锚点、能记轨迹或工件评价、失败代价可分级。

迁移：选窄域 → 20–40 个 Task Unit → L1 用 ESCO/e-CF 补词 → L2 自建 → 复用门控与双向分 → 只换验收器与工具。  
不迁移：SDLC 模板、代码工作区、部分运维政策。

---

## 22. 实施路径

合成 DeepSeek v4 的五步与测量栈的分阶段闭环：**每一步都做，但范围锁在一个切片里。**

### 阶段 A — 切片上的最小闭环

DeepSeek 五步在切片内一次走完：

1. **词汇表**：从 SFIA 抽本切片相关技能码；写 10–15 个 L2 叶子与锚点。
2. **任务分解与标注**：用模板填齐 §7.1 十维；验收必填。
3. **画像初始化**：人用问卷 / 职级 / 历史交付进 Declared/Attested；agent 跑探针。
4. **匹配**：门控 + \(S_{cap}\) + 人工确认；余弦只作召回。
5. **回写**：轨迹 EMA 更新 \(p,\sigma\)；投影出小矩阵。

建议切片：ADR、最小补丁、契约测试、告警归因。含至少 1 个人类席位、3–5 个 Actor。

退出：相对「固定指定」有可叙述胜出，负责人能读懂解释。

### 阶段 B — 把 v4 的组织机制接进来

- Needs / Offers 与 \(S_{need}\)
- 技能矩阵上墙
- 360 / 评审通道
- R2 强制独立评价者
- 校准卡与自报折减
- 回归探针进 CI

### 阶段 C — 图、合同网、联盟

- Task Graph 与交接信封
- 合同网招标
- 互补组队与多目标短名单
- 信誉快照
- 匹配回放 vs 纯余弦 / 裸招标基线

### 阶段 D — 平台化与第二域

- 词典治理、AHP 权重维护
- A2A/MCP 注册为 Declared
- 政策引擎、培养配额
- 向第二知识工作域复制（换 L1 子集与验收器）

禁止阶段 A 尚未跑通就建设全集 SFIA 或企业技能云。

---

## 23. 风险、反模式与开放问题

### 23.1 反模式

- 用余弦当最终匹配
- 用最强模型或「全面人才」接所有活
- 技能表只增不删
- 执行者写标准并自评
- 年度绩效式低频更新
- 先造十万词表
- 人格测验当主轴
- 忽略 X，只评裸模型
- 招标信自称成功率
- 一个总分同时代表能力、协作、安全、KPI
- 矩阵手填与 Card 分叉
- 用 OKR 直接改叶子分数

### 23.2 风险

| 风险 | 缓解 |
|---|---|
| 标签腐烂 | 叶子挂近期证据，过期降级 |
| 评价器偏差 | 抽检、多评价者、跨家族锚点 |
| 校准不收敛 | 折减 + 限制 R2+ |
| 解释事后合理化 | 扰动测试 |
| 权限扩散 | 短时凭证、过权拒绝 |
| KPI 拒难活 | 通道降权、难活单独记账 |
| 人不信任 | 先解释后自动，保留覆盖 |

### 23.3 开放问题

1. \(\lambda,\eta,\kappa\) 学习如何避免伤害长期互补。
2. 开放任务 rubric 的长期漂移。
3. 同模型家族互评。
4. 培养配额与 SLA。
5. 跨组织共享 Measured 证据的虚报激励。
6. 自动拆图如何保证可验收。
7. SFIA 5–7 与 agent 的职责边界随模型能力上移如何改政策，而不默默放开 R3。

---

## 24. 附录

### A. 合成来源对照

| 方法 | 在本系统中的地位 |
|---|---|
| DeepSeek v4 双画像七维 | Task Card 维度 1–8 的骨架 |
| SFIA / e-CF / ESCO | 词典 L1 |
| Bloom | 任务认知标签 |
| Cynefin | 情境策略 |
| 冰山模型 | 显隐分离的隐喻；O 层可观察化 |
| 技能矩阵 | Registry 视图 |
| 岗位分析 / 关键事件法 | Task Card 与叶子生长 |
| 360 / KPI / OKR | 证据通道 |
| 加权余弦 / 协同 / 上下文推荐 | 召回 |
| 规则 + ML | 门控 + 辅助预测 |
| 多目标优化 | 短名单与组队 |
| 合同网 | 协商协议 |
| 联盟形成 | 团队生成 |
| 信誉模型 | 软排序与准入，不替代叶子 |
| Person–Job Fit | 成交函数 |
| KSAO-X / ETCLOVG | Actor 真源与运行时 |
| 四级证据 / 校准卡 | 数字伙伴补丁 |
| CoWeaver MapScore | \(M\) 与解释分解的结构来源 |
| MarketBench | 校准卡存在的原因 |
| HELM / BigBench / SWE-bench | Attested 先验与探针参考 |

### B. 匹配伪代码

```
function staff(task, actors, team):
  pool = union(
    invert_index(task.hard_leaves, task.risk, task.permissions),
    cosine_topk(task.vector, actors, k=50),
    collab_filter(task, k=20),
    context_filter(task, actors)
  )
  feasible = [a for a in pool if gate(task, a, team)]
  scored = []
  for a in feasible:
    s_cap  = coverage(task.demands, team, a)      # 缺口，非余弦
    s_need = satisfy(task.offers, a.needs)
    M = task.lambda_ * s_cap + (1 - task.lambda_) * s_need
    s_hat = success_model(task, a)                # 辅助，不授标
    scored.append((a, M, s_hat, explain(...)))
  short = pareto_or_top(scored, k=8)
  for item in short:
    item.R = eta * item.M + (1 - eta) * process_fit(task, item.a, team)
  apply_policies(short, task)   # 分离、多样性、培养、信誉门槛
  return short
```

### C. 字段核对清单

**Task Card**：id, title, goal, sfia[], bloom, cynefin, information_state, graph, acceptance, demands[], offers, constraints, work_mode, context, risk_level, hidden_probes_ref, public_subset_policy, template_id, learned_Q_delta

**Actor Card**：id, kind, version_hash, capabilities[]（含 sfia_ref）, needs, execution_envelope, observable_style, calibration[], performance_channels, reputation, load, trust_tier, owner, allowed_risk_levels

**EvidenceRecord**：actor_version, leaf_id, source_channel, task_or_probe_id, x, weight, trajectory_ref, evaluator_id, at

**Bid**：actor_version, p_self, p_eff, cost, eta, conditions, calibration_ref

### D. 风险级

| 级 | 含义 | 例子 |
|---|---|---|
| R0 | 可逆、低影响 | 草稿、探针 |
| R1 | 可逆、局部 | 功能分支、文档 |
| R2 | 难逆或影响大 | ADR 生效、schema、预发配置 |
| R3 | 不可逆或生产伤害 | 热修、数据修复、密钥轮换 |

### E. 修订记录

| 版本 | 说明 |
|---|---|
| 0.1 | 首版测量栈为主，附录对照其他方法 |
| 0.2 | **合成版**。DeepSeek v4 的词典、矩阵、工作分析、绩效通道、推荐召回、合同网、联盟、信誉、Bloom/Cynefin 策略全部升为一等设计；成交函数与四级证据保持否决权 |

---

*落地以一个 SDLC 切片上的「L1 子集 + L2 锚点 + 门控成交 + 多通道回写」为最小闭环。词典与合同网按阶段生长，不先建企业技能云。*

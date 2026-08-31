# 文风与分层复审（doc-style-sweep）

> 状态：讨论中 · 量尺为草案，与首轮 findings 一并校准<br>
> 基线：main @ `b41ff6c`（草案 v0.15.4）<br>
> 去向：`docs/design/references/writing-style.md`（待裁决）+ 分层结构修订 PR + 语言修订 PR

## 本轮回答什么

上一轮全库大修（`../doc-overhaul-20260830/`，已收口 v0.15.1）解决的是**内容权威**问题：谁定义什么、去重、断链、禁令。本轮解决剩下的两件事，都是所有者点名的：

1. **讲人话。** 部分中文压过了头——结论留下、判据被删（“量级差三个数量级才值得动”被压成“量差三则动”那类）。还要清掉翻译腔与自造词堆砌。
2. **分层清晰。** 纵向按话题聚类（同类机制能识别、能归并，安全机制尤其），横向按颗粒度分级（愿景 / 架构 / 机制 / 实现，目的不同、文风不同）。本质目的是管理读者的 context——不必深入细节时不占用 thinking power。

## 方法：不自造量尺，把既有指南改编进库

文风量尺不新写。所有者裁定直接把 `yesme/abacistopia` 的 `SPEC-WRITING-GUIDE.md` 复制进本库并按需改写，成果是根目录的 [`WRITING-GUIDE.md`](../../../WRITING-GUIDE.md)（**发包与评审都以它为准**）。改编要点、放弃的旧方案与库内缺口种子见 [`01-style-posture.md`](./01-style-posture.md)。横向分层的可执行形态是四层路由表，已进指南 §0.4；其审计口径与纵向聚类种子见 [`02-layer-routing.md`](./02-layer-routing.md)。

## 分工

| 环节 | 谁 | 产出落到 |
| --- | --- | --- |
| 横向越层审计 + 纵向机制聚类矩阵 | Grok | `10-grok-structure.md` |
| 全量语言首读（21 文件对半分） | GPT / GLM | `11-gpt-language.md` / `12-glm-language.md` |
| 机械件（关键词、术语形态、超长块、文档头） | K3 | `13-k3-mechanical.md` |
| 量尺、语义分流、汇总、裁决包 | Fable | 本目录 `0x` 系列 + `20-verdict-packet.md` |
| 裁决后落地 PR | GPT | — |

任务书见 [`03-briefs.md`](./03-briefs.md)（一家一节，可整节投喂）。校准样本见 [`04-calibration.md`](./04-calibration.md)。

## 范围

**进**：根 `README.md`、`docs/usage.md`、`docs/design/` 全部正文与合同、`contract-tests.md`、`delivery.md`、`references/glossary.md`。约 21 个文件、2600–2800 行。

**不进**：`references/decision-history.md` 的历史条目（史料冻结，只审导言与台账格式）、`docs/research/` 正文（证据按当时原样，只审索引页）、`.memo/`（过程件）。

## 流程

1. 四家按本目录任务书并行产出 findings（只报不改）。
2. Fable 汇总复核，把归并候选分成「纯表述」与「触语义」两流，组装裁决包。
3. 所有者一次裁决：量尺定型 + 结构条目逐条 + 语言松紧基准。
4. 落地分两个 PR：**先结构后语言**（避免改完文字又搬家返工）。纯表述修订每波一次 patch bump + 小修订台账一行；**触语义的机制归并不在本轮落**，单独走设计变更路径（决策史 + CT 失败用例）。

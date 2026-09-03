# 设计文档四轴评审 · 状态板

> 状态：第三轮进行中——写法（#148）、约束（#149）、愿景与架构（#150）、Fable 写作与调研（#153）已合入 main（v0.16.4），两条补充裁决记入 #154；Codex 文档工具链已完成本地实现与验证，运行时与部件及 Grok 的测试 PR 待做，任务书见 `30-r3-remaining-brief.md`。待所有者表态一处：I-03 的 112 句判为不做全量加词（理由见任务书一进度注），不同意可回清单逐句再议<br>
> 基线：main @ `99b0bfb`（草案 v0.16.0）<br>
> 计划：[`01-plan.md`](./01-plan.md)；原则：[`../../notes/review-four-axes-20260902.md`](../../notes/review-four-axes-20260902.md)；状态与方法：[`00-brief.md`](./00-brief.md)

| 阶段 | 状态 | 产出 | 最后更新 |
| --- | --- | --- | --- |
| 1.1 定标与脚手架 | 完成 | `02-checklists.md`、`src/build/docs/inventory_*.pl`、`root//build/docs:inventory` | 见分支 |
| 1.2 机械清点 | 完成 | `10-inventory.md`（四张表） | 见分支 |
| 1.3 业界调研 | 完成 | `docs/research/methodology-boundaries-20260902.md`、`methodology-sweep-2026h2-20260902.md`、`component-matrix-20260902.md` | 见分支 |
| 1.4 通读与发现 | 完成（24 个文件，调研证据已回填） | `11-findings-draft.md` | 见分支 |
| 1.5 对抗核验 | 完成：四轴四份，维持 44 / 修正 64 / 推翻 5 / 补 27 | `12-adversarial-{A,I,W,M}.md`、`13-findings.md` | 见分支 |
| 1.6 裁决包 | 完成，随第一轮 PR 合入 | `20-verdict-packet.md` | 见分支 |
| 2 拍板 | 完成：30 条全部裁决，新增 A-57；过程见 `21-r2-rulings.md` | `20-verdict-packet.md`、`21-r2-rulings.md` | 见分支 |
| 3.1 写法统一（Room、当前范围、词表分类、人的写法） | 完成 | PR #148（v0.16.1） | 见 main |
| 3.2 约束含义（Binding 族重构、六项机制、CT 用例） | 完成 | PR #149（v0.16.2） | 见 main |
| 3.3 愿景与架构层改写（Fable） | 完成 | PR #150（v0.16.3） | 见 main |
| 3.4 写作与调研（Fable，任务书一：研究对象文件、写法遗留、研究层追加、评审 Skill、Foreman 备忘） | 完成 | PR #153（v0.16.4）；补充裁决 GPL 生成物入库、钥匙串回退 PR #154 | 见 main |
| 3.5 代码：文档工具链 PR 甲、运行时与部件 PR 乙（Codex + GPT，任务书二） | 进行中：文档工具链已完成本地实现与验证，运行时与部件待做 | `30-r3-remaining-brief.md` §任务书二 | 2026-09-04 |
| 3.6 测试计划与对抗核验（Grok，任务书三） | 待开工（跟在每个 Codex PR 合入之后） | `30-r3-remaining-brief.md` §任务书三 | — |

接手的会话：先读三个链接，再看这张表，从第一个「待开始」或「进行中」的阶段接着做；每完成一个阶段改这张表并提交。

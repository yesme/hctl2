# 用例反推 · 单元模型 · 状态板

> 状态：讨论中 · 两轮审阅完成；§九 三处里用例修改与"账本"禁用已落（#195、#194），第 13 条治理正文已拍板（`03-governance-text.md` v3，五家一致，所有者授权收口）；下一步开"设计文档怎么改"<br>
> 基线：main @ `9bbab56`（草案 v0.17.1）<br>
> 去向：`docs/design/architecture.md`、`vision.md`、五份模块正文与约束、`delivery.md` §当前范围、decision-history 新节；本 PR 不改约束层

起点是所有者的用例 [`.memo/notes/HCTL_case_study.md`](../../notes/HCTL_case_study.md)（PR #191 落地，2026-09-07 又修订：gh-jssdk 同时归 mac 与 cloud 两个控制面的三个 Project）与 Antigravity 的反馈 [`case-study-feedback-20260906.md`](../../notes/case-study-feedback-20260906.md)。所有者的要求是先讨论清楚"这个用户体验对不对、应不应该这样"，再从用例按第一性原理反推现有设计文档的问题；设计怎么改、已完成实现怎么改、计划怎么改、怎么把用例物化成机械测试，这四件事在本轮之后。

| 文件 | 作者 | 内容 |
| --- | --- | --- |
| `01-unit-model.md` | Fable | v3：单元模型（四类单元加底座、三种关系、总规则）、所有者对五个追问的回答、五组十七条设计问题、看板多源、雇佣与待命、失败路径、用例文本要对齐的七处、两轮后的收敛结果与待拍板三处 |
| `02-review-brief.md` | Fable | 四家主审的任务书：共同部分（审整个 PR 加 #191，给独立意见）与各家的专门方向 |
| `03-governance-text.md` | Fable | 第 13 条治理正文放哪（v4，已拍板，第十节待 Codex 看）：参与者读任务书靠冻结输入、实际交付、保留与可再取；缺省独立治理材料仓库、本地裸库必有、平台托管是写穿镜像、`.hctl2/` 跟踪目录取消、旁路 ref 显式选项、审计关联发布到评审请求；约束层落点清单；控制面存储的两半与 Git 后端（实现层脚注） |

流程（所有者定）：Codex、Grok、Kimi、GLM 都作为主审，第一轮独立审阅、第二轮交叉审阅，Antigravity 作为 #191 反馈的作者在第二轮加入；两轮后由 Fable 以 merge commit 合入。评论都在 PR #193。

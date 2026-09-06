# 用例反推 · 单元模型 · 状态板

> 状态：讨论中 · 第一轮独立审阅完成（Codex、Grok、Kimi、GLM 均维持方向）；底稿已出 v2 并入四家意见；待第二轮交叉审阅<br>
> 基线：main @ `9bbab56`（草案 v0.17.1）<br>
> 去向：`docs/design/architecture.md`、`vision.md`、五份模块正文与约束、`delivery.md` §当前范围、decision-history 新节；本 PR 不改约束层

起点是所有者的用例 [`.memo/notes/HCTL_case_study.md`](../../notes/HCTL_case_study.md)（PR #191 落地，2026-09-07 又修订：gh-jssdk 同时归 mac 与 cloud 两个控制面的三个 Project）与 Antigravity 的反馈 [`case-study-feedback-20260906.md`](../../notes/case-study-feedback-20260906.md)。所有者的要求是先讨论清楚"这个用户体验对不对、应不应该这样"，再从用例按第一性原理反推现有设计文档的问题；设计怎么改、已完成实现怎么改、计划怎么改、怎么把用例物化成机械测试，这四件事在本轮之后。

| 文件 | 作者 | 内容 |
| --- | --- | --- |
| `01-unit-model.md` | Fable | v2：单元模型（四类单元加底座、三种关系、总规则）、所有者对五个追问的回答、五组十七条设计问题、看板多源、雇佣与待命、失败路径、用例文本要对齐的清单、第二轮要收敛的点 |
| `02-review-brief.md` | Fable | 四家主审的任务书：共同部分（审整个 PR 加 #191，给独立意见）与各家的专门方向 |

流程（所有者定）：本 PR 不自合；Codex、Grok、Kimi、GLM 都作为主审，各写一条评论，前半是对整个 PR 与 #191 的独立意见，后半是各自专门方向的深审；反复一两轮后由 Fable 合入（merge commit）。

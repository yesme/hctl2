# HCTL2：产品与系统设计规范

> 状态：Draft v0.7.0<br>
> 日期：2026-08-14<br>
> 权威入口：[docs/design/README.md](./docs/design/README.md)

设计规范已从单一长文档拆分为 `docs/design/`。本文件保留为兼容入口；规范正文、事实边界、交付范围与 evidence 都在模块化文档中维护。

## 核心四层

```text
L4  Intent & Collaboration
    Project Room
    主要参考：First Tree

            ↓ 提炼目标、范围、acceptance

L3  Commitment & Tracking
    Task / Kanban
    主要参考：Codeg；外部状态参考 Linear/GitHub

            ↓ 冻结 TaskRevision、批准自动施工

L2  Governance & Orchestration
    Workflow / Run / Gate
    外部主参考：缺少成熟的 project-semantic governance
    内在谱系：HCTL1 / yesme/hctl；HCTL2 原生扩展
    机械状态后端：Conductor

            ↓ Obligation / Seat / Attempt

L1  Execution & Runtime
    Worktree / Harness / Terminal / Diff
    主要参考：Stably Orca
```

这四层是 HCTL2 的产品和语义主轴：

- L4 负责意图、讨论来源、Context 与 Human Request；
- L3 把 discussion 提炼成带 acceptance 的 commitment；
- L2 冻结 bounded authority，并以 revision/evidence 治理自动施工；
- L1 承载可替换 Harness、worktree、process、PTY、terminal 与 diff。

结果从 L1 以 evidence 回到 L2，经 Verdict/Receipt 到 L3 完成 admission，再以 milestone/learning 回到 L4。任何层都不能通过自由文本、共享 UI state 或 donor database 偷改相邻层。

## 现在从哪里读

1. [设计规范首页与四层地图](./docs/design/README.md)
2. [基础、目标与原则](./docs/design/foundation.md)
3. [权威领域模型](./docs/design/domain-model.md)
4. [L4 · Project Room](./docs/design/layers/l4-project-room.md)
5. [L3 · Task / Kanban](./docs/design/layers/l3-task-kanban.md)
6. [L2 · Workflow / Run / Gate](./docs/design/layers/l2-workflow-governance.md)
7. [L1 · Execution / Runtime](./docs/design/layers/l1-execution-runtime.md)
8. [跨层生命周期](./docs/design/cross-layer/lifecycle.md)
9. [系统架构与事实边界](./docs/design/cross-layer/system-architecture.md)
10. [Workbench 与交互集成](./docs/design/cross-layer/workbench.md)
11. [Phase 1 与分级自举](./docs/design/delivery/phase-1-and-dogfooding.md)
12. [验证、ADR 与未决事项](./docs/design/delivery/validation.md)

辅助索引：

- [术语表](./docs/design/references/glossary.md)
- [规范性不变量](./docs/design/references/invariants.md)
- [实现证据与精选参考组合](./docs/design/references/implementation-evidence.md)
- [设计演进记录](./docs/design/references/decision-history.md)

## Reference policy

外部项目不做四层全覆盖能力打分。一个 effort 只在其 highest-information-gain layer 进入正文；顺带存在但没有独特增量的功能留在 evidence Radar 或不再引用。

因此 Codeg 和 Stably Orca 没有放反：Codeg 的 strongest product evidence 是 independent Task/Needs You/review/Git-truth lifecycle，属于 L3；Stably Orca 的 strongest evidence 是 worktree/daemon PTY/terminal restore/remote/fencing，属于 L1。Superset、Multica、Herdr 等同样只在各自最擅长的层留下 targeted evidence。

## 原长文档迁移表

| 旧章节 | 新位置 |
| --- | --- |
| §0–§3 执行摘要、Vision、问题、原则 | [Foundation](./docs/design/foundation.md) |
| §4 领域模型、附录 A 术语 | [Domain model](./docs/design/domain-model.md)、[Glossary](./docs/design/references/glossary.md) |
| §5 Planning/Build、§9 完整流程 | [Lifecycle](./docs/design/cross-layer/lifecycle.md) |
| §6–§7 Room/Composer/Context、§12 Request | [L4](./docs/design/layers/l4-project-room.md) |
| §10 Project Overview/Task Kanban | [L3](./docs/design/layers/l3-task-kanban.md) |
| §11 Workflow/Conductor | [L2](./docs/design/layers/l2-workflow-governance.md) |
| §8 Harness、§13 Git/worktree、§14 runtime/terminal | [L1](./docs/design/layers/l1-execution-runtime.md) |
| §15–§16 persistence/components、§21–§22 security/recovery | [System architecture](./docs/design/cross-layer/system-architecture.md) |
| §17–§18 UI/GUI | [Workbench](./docs/design/cross-layer/workbench.md) |
| §19–§20 sourcing/implementation reuse | [Implementation evidence](./docs/design/references/implementation-evidence.md) 与各 layer |
| §23–§25 MVP/bootstrap/repo layout | [Phase 1 and dogfooding](./docs/design/delivery/phase-1-and-dogfooding.md) |
| §26–§28 ADR/tests/metrics/open questions | [Validation](./docs/design/delivery/validation.md) |
| §29 选型演进 | [Decision history](./docs/design/references/decision-history.md) |
| 附录 B invariants | [Invariants](./docs/design/references/invariants.md) |
| 附录 C–D external references/radar | [Implementation evidence](./docs/design/references/implementation-evidence.md) |

Git history 保留了拆分前的完整版本；新内容不再同时维护一份 legacy monolith，以避免双重规范和漂移。

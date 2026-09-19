# P2 · 接钥匙 · 状态板

> 状态：已拍板 · 2026-09-20 起 P2.1 按 [`04-p21-kickoff.md`](./04-p21-kickoff.md) 开工（所有者 09-19「开」；按 v0.18.11 重切，变化映射见其 §二）；此前：四项取舍与里程碑重切已定，2026-09-15 工作包按 `01-plan.md` §十二 补记重切（v0.18.3），Repo 身份题已撤，戊解冻，丙撤销；六份前置研究已由 Codex 完成<br>
> 基线：main @ `2c6b2e4`（草案 v0.18.11）；`01-plan.md` 正文写于 v0.17.0，历史不改<br>
> 去向：`src/apps/hctl2-control`、`src/apps/hctl2`、`src/crates/*`、`docs/research/`；不改约束层

> 2026-09-20 命名更新：已有源码目录 `apps/hctl2-tool`、`crates/hctl2-facts`、`crates/hctl2-foundation`、`crates/hctl2-store` 分别改为 `apps/tool`、`crates/facts`、`crates/foundation`、`crates/store`（均相对 `src/`）；私有 package 与库目标同步用短名，对外命令仍为 `hctl2-tool`。本目录已拍板任务书的历史名称不回写，继续开工时按 [src/README 命名说明](../../../src/README.md)定位现行代码；尚未建立的 control / CLI 目录也按职责取名，不照抄上述历史路径。

| 文件 | 作者 | 内容 |
| --- | --- | --- |
| `01-plan.md` | Fable | P2 计划：定位与重述、起点核对、P2 的形状、工作包与分工、五项取舍的讨论与拍板、任务书要点、审核方式、研究层先行清单、延后与遗留、里程碑重切 |
| `02-research-brief.md` | Fable | 六份前置研究的任务书，Codex 写、GLM 审 |
| `03-repo-identity-discussion.md` | Fable | Repo 稳定身份的讨论底稿：Fable 的原理解（所有者判「理解问题很大」）与待聊的问题；已随 C 批撤题 |
| `04-p21-kickoff.md` | Fable | P2.1 开工书：v0.18.3 之后的约束变化落到哪个包、三包任务书（v0.18.11）、给 Codex 的第一包、待所有者定 |
| `05-p22-kickoff.md` | Fable | P2.2 开工书：落到戊己庚辛的裁决、四包任务书（v0.18.11）、顺序与前置、给 Codex 的第一包（指针版）、待所有者定 |

| 级 | 工作包 | 状态 |
| --- | --- | --- |
| P2.1（旧 B0） | 0a/0b/0c 研究（已完成）→ 甲 控制面存储（两半）与命令内核 → 乙 进程与客户端边界 → 丁 托管生命周期与收口（丙已撤销，见 §十二） | 开工书 `04` 已合 #271；甲 Codex 交付见 #276（三席评审与作者说明）；乙、丁待开 |
| P2.2（旧 B1） | 1a 研究复核（已完成）→ 戊 Repo 注册（三选一，已解冻）→ 己 聊天端口（等 chat 探针）∥ 庚 任务源端口（平台 issues 先，本地任务服务器可加绑）→ 辛 Project 与 Request、收口 | 开工书 `05` 待轻审；四包等甲合入 |
| P2.3 / P2.4（旧 B2） | 2a/2b 研究 → 壬 Agency 端口、癸 Participant 与调用、子 Context、丑 Repo 模块、寅 Task 完成 → 卯 收口 | 未开始 |
| P2.5（旧 B4/B5）与自举等级 A1–A4 | 入口见 `01` §四、§十 | 未开始 |

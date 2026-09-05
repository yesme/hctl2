# P1 收口 · 状态板

> 状态：已拍板 · 执行中（所有者 2026-09-04「你开始吧」）· 计划见 `01-plan.md`<br>
> 基线：main @ `033da13`（草案 v0.16.5）<br>
> 分工（所有者 2026-09-04）：Grok / Codex 主写，Claude（Fable）/ GLM 主审；写作与调研归 Fable

| 序 | 阶段 | 写 | 状态 | 产出 | 最后更新 |
| --- | --- | --- | --- | --- | --- |
| 0 | git CLI 对象文件 `docs/research/sdk/git.md` | Fable | 完成 | PR #173：采用二进制、宿主 git、下限 2.39、不内嵌库；索引两处加行 | 2026-09-04 |
| 甲 | 仓库检查、现场锁、worktree 物化与核验 | Codex | 完成 | PR #174：`repo inspect`、`worktree materialize/verify`、现场锁；双审修正与回归用例已落实 | 2026-09-05 |
| 乙 | 封存、保全、拆除 | Grok | 待开始（等甲） | — | 2026-09-04 |
| 丙 | 本地集成：校验 → CAS → 回读 | Codex | 待双审（Fable / GLM） | `integrate`：快进 / 合并提交、Git ref 固定重试提交、CAS 与结果未知；分支 `codex/p1-toolbox-c` | 2026-09-05 |
| 丁 | 打包后三平台端到端、usage 与 README、状态板收口 | Grok | 待开始（等乙丙） | — | 2026-09-04 |

审核记录（每个 PR 两份评论，逐条维持 / 修正 / 推翻）：

| PR | Fable | GLM | 结果 |
| --- | --- | --- | --- |
| #174 | [9 项修正](https://github.com/yesme/hctl2/pull/174#issuecomment-5549351943) | [2 项修正、1 项备注](https://github.com/yesme/hctl2/pull/174#issuecomment-5549333306) | 修正已落实；身份来源采用显式标注提交与工作树；正文原语留位在 `content.rs` |

**延后与遗留**见 `01-plan.md` §六。

接手的会话：先读 `01-plan.md`，再看这张表，从第一个「待开始」或「进行中」的阶段接着做；每完成一个阶段改这张表并提交。

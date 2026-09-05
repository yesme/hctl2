# P1 收口 · 状态板

> 状态：已拍板 · 执行中（所有者 2026-09-04「你开始吧」）· 计划见 `01-plan.md`<br>
> 基线：main @ `033da13`（草案 v0.16.5）<br>
> 分工（所有者 2026-09-04）：Grok / Codex 主写，Claude（Fable）/ GLM 主审；写作与调研归 Fable

| 序 | 阶段 | 写 | 状态 | 产出 | 最后更新 |
| --- | --- | --- | --- | --- | --- |
| 0 | git CLI 对象文件 `docs/research/sdk/git.md` | Fable | 完成 | PR #173：采用二进制、宿主 git、下限 2.39、不内嵌库；索引两处加行 | 2026-09-04 |
| 甲 | 仓库检查、现场锁、worktree 物化与核验 | Codex | 完成 | PR #174：`repo inspect`、`worktree materialize/verify`、现场锁；双审修正与回归用例已落实 | 2026-09-05 |
| 乙 | 封存、保全、拆除 | Grok | 完成 | PR #177：`archive snapshot/remove`；保全覆盖已跟踪但被忽略的修改、磁盘上真实存在的嵌套仓库与已初始化子模块；丢弃确认绑当前树 sha | 2026-09-05 |
| 丙 | 本地集成：校验 → CAS → 回读 | Codex | 完成（复审通过） | PR #176：按 #178 修订任务书修正检出保护、祖先回读、两策略 no-op 与 reflog；重试开关的恢复路径已补进 help 与测试 | 2026-09-05 |
| 丁 | 打包后三平台端到端、usage 与 README、状态板收口 | Grok | 待开始（等乙丙） | — | 2026-09-04 |

审核记录（每个 PR 两份评论，逐条维持 / 修正 / 推翻）：

| PR | Fable | GLM | 结果 |
| --- | --- | --- | --- |
| #174 | [9 项修正](https://github.com/yesme/hctl2/pull/174#issuecomment-5549351943) | [2 项修正、1 项备注](https://github.com/yesme/hctl2/pull/174#issuecomment-5549333306) | 修正已落实；身份来源采用显式标注提交与工作树；正文原语留位在 `content.rs` |
| #176 | [4 项修正](https://github.com/yesme/hctl2/pull/176#issuecomment-5549705891) | [2 项非阻断备注](https://github.com/yesme/hctl2/pull/176#issuecomment-5549691090) | [Fable 复审](https://github.com/yesme/hctl2/pull/176#issuecomment-5549854613)与 [GLM 复审](https://github.com/yesme/hctl2/pull/176#issuecomment-5549857044)通过，六项关闭；锁诊断另列 `intent_digest`，预备 ref 清理归属见下 |
| #177 | [推翻 R1–R3 + 3 项修正](https://github.com/yesme/hctl2/pull/177#issuecomment-5549742219)；[复审两项修正](https://github.com/yesme/hctl2/pull/177#issuecomment-5549885751)；[再复审维持](https://github.com/yesme/hctl2/pull/177#issuecomment-5550215203) | [修正 A/B](https://github.com/yesme/hctl2/pull/177#issuecomment-5549727741)；[复审通过](https://github.com/yesme/hctl2/pull/177#issuecomment-5549883476) | 推翻撤销；两轮修正均落实；GLM 无保留 |

**延后与遗留**见 `01-plan.md` §六。

丙的重试缓存交接：P1 保留 `refs/hctl2/integrations/` 下的预备 ref，失败也不自动删。丁在 usage 说明这一点；P2 control 在意图结束、无需重试且结果仍有可达副本时负责显式清理。失败或结果未知的提交先保全再去掉最后一个 ref；删除缓存也会失去工具箱的键到结果定位，不能再依赖原键回查。P1 不加清理子命令。

接手的会话：先读 `01-plan.md`，再看这张表，从第一个「待开始」或「进行中」的阶段接着做；每完成一个阶段改这张表并提交。

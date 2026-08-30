# 设计文档清扫待办（v0.13.0 之后）

> 日期：2026-08-25<br>
> 状态：已核销 · 2026-08-31 所有者一次裁决：11 条语义放宽随 v0.15.2 落地（来时路 §32、PR #105），2 条已随组件退场核销。<br>
> 其余 11 条:措辞形态已随大修按白名单处理,语义改判见 parking-lot #3。<br>
> 来源：[grok-20260824a](./grok-20260824a.md) §4 与 [codex-design-review-20260825a](../review/20260825-v0.12.3/codex-design-review-20260825a.md) §三，经所有者裁决"记下来，先不改"。

## 1. 禁令清单清扫（正确道路只有一条，反例写不完）

v0.13.0 只在改到的段落写了正面陈述；其余仍有成串的"不是 / 不能 / 不得 / 拒绝"，例如 `spec/task.md` 末段的完成来源反例、六个 `CT-*` 契约测试族里的负例、`spec/system.md` 安全边界一节。清扫原则：把每条负例问一遍"正确道路写清了它还进得来吗"——进不来就删，进得来才说明正确道路没写清，改正面句而不是加禁令。整体一轮，单独提交。

## 2. codex 评审列出的过强断言（13 条，尚未拍板）

| 条款 | 位置 | 建议方向 |
| --- | --- | --- |
| agentd-only terminal | `spec/system.md` 端口一节、`spec/agent.md` 终端一节、`delivery.md` | 已过时,随大修核销 |
| discovery 绝不联网 | `spec/system.md` 扩展绑定 | 不静默安装/改配置；联网探测可配置 |
| 所有动作都必须 Preview | `spec/system.md` 场景端口、`spec/project.md` 场景合同 | 危险动作默认确认；普通命令可直接 submit |
| writer 不可证明静默就永久弃用 worktree/ChangeSet | `spec/agent.md` ChangeSet 一节 | 默认隔离并提示；允许用户确认后接管/采纳/封存/丢弃 |
| 清理前绝不允许丢弃残留 | `spec/agent.md` | 默认保全；用户确认后可丢弃 |
| 固定锁路径与多层 generation | `spec/system.md` 单写者 | 只保留一个逻辑 writer、已确认副作用不重复、旧结果不覆盖新结果 |
| 固定存储拓扑 / 禁止 Git refs | `spec/system.md` 存储 | 路径作为默认示例；实现选择 |
| Repo Instance 强绑定 common-dir 取证 | `spec/system.md` Repo 与执行现场 | stable repo ID 优先；缺失/冲突时展示证据让用户确认 |
| Project 归档前必须清空全部开放对象 | `spec/project.md` | 只阻止活跃写执行/租约/未决写副作用 |
| Scoped Room 只有成功回填才能归档 | `spec/project.md` | 允许以 abandoned / no-decision / superseded 归档 |
| Context 必须毫秒级、全本地、零模型 | `context.md`、`spec/project.md` | 默认本地规则；模型辅助可配置 |
| 活动 Run 时禁止采纳新 Task Revision | `spec/task.md` | 允许采纳；旧 Run 不得静默完成新 Revision |
| 打包与 tmux 拓扑写死 | `delivery.md` | 已过时,随大修核销 |

与第 1 项合并成一轮处理；每条若改合同，按转向立项。已在 v0.13.0 处理掉的不列（沙箱入场券、凭据网关、Git common-dir、在场证明、E2EE、Dagu 代次、P0 只验接缝、后端并发令牌与运行时排他）。

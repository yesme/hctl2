# C5 历史与档案施工卡(派给 GPT,C4 合入后)

> 状态:待派发 · Fable 依冻结总图预写;所有者在 C4 合入后原样派给 GPT<br>
> 基线:C4 合入后的 main<br>
> 依据:[`03-approved-plan.md`](./03-approved-plan.md) §3 C5 行与「拍板后修订」两行;[`02-target-map.md`](./02-target-map.md) §3.12(五张表)、§3.21、§3.24;拍板点 3、9、12;Grok 死文普查 A13–A18、A28–A31、C3/C4<br>
> 闸门:CI docs-check(自动)+ 所有者目检;无 Grok 环节

## 派发文本

```
你在 hctl2 仓库工作。先 git fetch 并把当前分支快进到 origin/main(C4 已合入),读 AGENTS.md、CONSTRAINTS.md,再读 .memo/design/doc-overhaul-20260830/ 下:施工图 README.md(§2 红线)、03-approved-plan.md(§3 C5 行、「拍板后修订」)、02-target-map.md(§3.12 五张表行、§3.21、§3.24)、01-inventory-dead-text.md(A13–A18、C3/C4)、08-c5-card.md。你是唯一写手,现在开 C5(历史与档案),这是最后一个施工簇。

范围:docs/design/references/decision-history.md、docs/design/spec/README.md(仅五张历史表)、全库版本戳、.memo 状态核销。不碰其他正文。

一、来时路折叠(拍板点 3;只折不删,§1–§33 编号不变):
- §6(Conductor 边界):正文缩为一段「当时为何、被 §18 取代」;顺带改正 :54 笔误「在 §19 换成 Dagu」→「在 §18 换成 Dagu」(Grok A13)。
- §13(P/B 双表与 P0 选型):双表部分留一句,选型段折叠(A14)。
- §18(Dagu):只折叠「B4 完成 API 代次隔离阻断」一段(已被 §23 撤销),选型理由与边界全部保留(A15)。
- §19(tmux):缩为一段,保留「当时因 tmux 官方二进制把 macOS 基线升到 15、后被 §29 取代;当前基线依据见 delivery 打包策略」的时间关系,不反改历史(A16 + GPT C2 保真修订)。
- §27(运行时 provider 三轮):缩为一段指向 §29;改正 :223 的 `agent.md#agency` 锚点为 `#agency-与-herdr`、「交付文档第 6 项」为「P0 第 2 项」(A17、C3)。
- §28(中间方案):缩为两句(A18)。
- §15 内 :155 锚点「#执行面已选依赖的运维与-footprint」改为「#已选外部服务的运维与资源占用」(C4)。
- 折叠格式统一:保留原章节标题与「(vX.Y.Z…)」后缀,正文一段 ≤5 行,以「细节见本仓库 Git 历史」收尾;不引入 archive 文件。

二、五张历史表迁移(拍板点 9;与本文件同一 PR,防台账锚点断链):
- 从 spec/README.md 整表剪切:v0.9.1 归并对照、v0.10.3 清扫、v0.11.1 词形收敛、v0.12.2 清扫、v0.13.0 收窄。
- 原样粘贴到 decision-history 对应章节尾:v0.9.1 → §11 尾;v0.10.3 → §12 尾;v0.11.1 → §14 尾;v0.12.2 → §20 尾;v0.13.0 → §22 尾。每张表前加一行「核销记录(自 spec/README 迁入)」。
- 小修订台账(§32)三行的详情链接改指新位置锚点;全库 grep 旧锚点(#v091-归并对照、#v0103-清扫、#v0111-词形收敛、#v0122-清扫、#v0130-收窄)改链。
- spec/README.md 迁出后应 ≤90 行(上限此时生效)。

三、小修订台账加行(拍板点 12):
| v0.15.1 | <合入日期> | 全库文档大修:门户收束、设计层与合同层同构合并、禁令按白名单三分、CT 矩阵拆出 contract-tests.md、来时路折叠与历史表迁入;不改合同语义 | [大修施工图](../../../.memo/design/doc-overhaul-20260830/README.md) |
(路径按仓库实际相对层级核对;若 .memo 链接不宜进正文,改写为「见 .memo/design/doc-overhaul-20260830/」纯文本。)

四、全库版本戳升 v0.15.1(拍板点 12):
- 所有带「草案 v0.15.0」头部的文件(根 README 基线行、design/README、vision、architecture、participant、context、spec 七件、glossary、decision-history 状态行「对应草案」)统一改 v0.15.1;日期改合入当日。
- participant.md / context.md 若仍是 v0.14.1(C2 未动戳)一并升。
- 检查 src/build/docs/version_stamps.allowlist 是否需要同步;全库一次改齐则检查应自然绿,不改 allowlist 白名单本身除非有正当滞后项。

五、.memo 状态核销(总图 §3.24;只改状态标头与表,不重写任何过程稿):
- .memo/design/hctl2-agentd-prd-20260826.md 文件头状态改「已废弃 · 组件已由 §29 退场;条款待按 Herdr / control / tool 三类重归」。
- .memo/notes/doc-cleanup-backlog-20260825.md:#1、#13 两行标「已过时,随大修核销」;文件头加一行「其余 11 条:措辞形态已随大修按白名单处理,语义改判见 parking-lot #3」。
- .memo/README.md 待拍板表:agentd-prd 行标废弃;「doc-overhaul 施工图」行改「已拍板 · 施工收口中」;control-storage、context-feeding §8、grok-ci-cadence 三行不动(仍待拍板,不猜删)。
- .memo/design/doc-overhaul-20260830/README.md(施工图)状态行改「已拍板 · C1–C5 施工完成,待 S4 收口」。

守则:来时路只折不删、不改写论证;历史表原样迁移一字不改;不新增概念与禁令;版本戳一次改齐;台账行是唯一新增正文。

验收:cd src && build/docs/materialize_repo_tree.sh && ./buck2 test root//build/docs/... 全绿(版本戳与链接检查是本簇的硬门);PR 描述三节 + 折叠前后行数(decision-history、spec/README)+ 改链清单 + 版本戳改动文件清单;"业界方案调研"不适用;开 auto-merge 但所有者目检前不合入。完成后回复:两文件行数、改链条数、版本戳改动数。
```

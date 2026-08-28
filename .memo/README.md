# .memo · 中间过程与备忘

> 这里放的是**还没成为产物**的东西。产物有两处家：规范在 `docs/design/`，证据审计在 `docs/research/`。写之前先按下面三个问题定位置；写完不用维护索引，靠目录和文件头。

## 三个问题定位置

1. **它写完要去哪？** 去设计文档/合同 → `design/`；去实现证据 → `docs/research/`（不在本目录）；哪都不去 → 下一问。
2. **它是针对某个基线版本的评审或提案吗？** 是 → `review/`；不是 → 下一问。
3. **它有一个能起名字的主题吗？** 有 → `notes/`；没有，就是那天聊了什么 → `log/`。

| 目录 | 命运 | 组织轴 | 会回头改吗 | 一句话判据 |
| --- | --- | --- | --- | --- |
| `design/` | 被合同消化：拍板后结论进 `docs/design/`，文件留作底稿 | 主题 | 改状态与去向 | 整份文件在等一个拍板 |
| `review/<日期>-<基线>/` | 随基线作废：一轮一个子目录，每个 harness 一份 | 轮 | 不改，只核销 | 作者是 harness、对象是某个 sha、基线一动就得重写 |
| `notes/` | 偶尔被翻 | 主题 | 偶尔 | 有名字，不等拍板（待办、原则记录、交接） |
| `log/` | 什么都不发生 | 日期 | 永不 | 只能用日期加"和谁聊"命名 |

`log/` 是唯一没有规则的地方：文件名 `YYYY-MM-DD-一两个词.md`，一天多次就多个文件，写完即冻结。某天从里面提炼出东西，就另起一份 `design/` 或 `notes/` 并在文件头写"来自 log/…"；原件不动。不确定该放哪的，先进 `log/`，有了去向再挪——分类发生在有去向的时候，不发生在写的时候。

## 文件头

`design/` 与 `review/` 的文件头固定三行，紧跟标题，其余目录按需：

```
> 状态：讨论中 | 待拍板 | 已拍板 | 已落地 | 已核销 | 已归档 | 已废弃
> 基线：main @ <sha>（草案 vX.Y.Z）
> 去向：docs/design/xxx.md + decision-history §N ｜ docs/research/xxx.md ｜ 无
```

状态值后可用「·」接一句限定（如 `已落地 · §8 待拍板`、`已拍板 · 待落地`）。`已拍板` 是所有者定了但合同还没写的中间态；`已归档` 用于评审轮被后续修订整体吸收、没有逐条核销记录的情况；`review/` 的去向写核销记录在哪。文件原有的「状态」行改名为「说明」保留。

文件名沿用 `<主题或 harness>-<日期>[a-z].md`；`review/` 子目录名 `<日期>-<基线版本或议题>`。

## 与 HCTL2 对象的对应

这套目录是在没有 HCTL2 的情况下手工模拟它：`log/` 是 Room 历史（content，本不该是文件）；`design/` 是一个改 docs 的 ChangeSet 加讨论它的 Scoped Room（飞行中，所以有生命周期）；`review/` 是一个多席位评审 Run，每份文件是一个 Attempt 的 Proposal，账本里的 Verdict 在 Git 只有影子；`notes/` 是 Memo；`docs/research/` 是 Task 交付的 Artifact。注意本目录名与 HCTL2 的 Memo 对象撞名——这里八成的东西不是 HCTL2 意义的 Memo。

## 待拍板

只列 `design/` 里状态不是"已落地"的项，拍了就删行：

| 文件 | 等什么 |
| --- | --- |
| `design/control-storage-20260821.md` | 五储对照总表是否成文进 `spec/system.md` |
| `design/context-feeding-20260826.md` §8 | 执行中产生的经验（lesson）是否作为 Result Proposal 可选输出项自动提案；晋升仍走 Memo/Skill |
| `design/hctl2-agentd-prd-20260826.md` | 讨论稿 → 合同或交付文档 |
| `design/grok-ci-cadence-20260828a.md` | C 已落地；发行整包是否继续挡 PR，以及 A/B/D 是否调整 workflow 与保护 |
| `notes/doc-cleanup-backlog-20260825.md` | 13 条过强断言逐条裁决（零散进行，不整份等） |

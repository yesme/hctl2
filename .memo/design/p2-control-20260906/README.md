# P2 · 接钥匙 · 状态板

> 状态：已拍板 · 2026-10-03 起按 [`07-demo-kickoff.md`](./07-demo-kickoff.md) 推进：九个包、单线串行、三次演示（所有者 2026-10-03「落」）；此前 P2.1 按 [`04-p21-kickoff.md`](./04-p21-kickoff.md)、P2.2 按 [`05-p22-kickoff.md`](./05-p22-kickoff.md) 开工<br>
> 基线：main @ `a075a66`（草案 v0.19.2）；`01-plan.md` 正文写于 v0.17.0，历史不改<br>
> 去向：`src/apps/*`（控制面与 CLI 目录按职责取名，对外二进制仍为 `hctl2-control` / `hctl2`）、`src/crates/*`、`docs/research/`；不改约束层

> 2026-09-20 命名更新：已有源码目录 `apps/hctl2-tool`、`crates/hctl2-facts`、`crates/hctl2-foundation`、`crates/hctl2-store` 分别改为 `apps/tool`、`crates/facts`、`crates/foundation`、`crates/store`（均相对 `src/`）；私有 package 与库目标同步用短名，对外命令仍为 `hctl2-tool`。本目录已拍板任务书的历史名称不回写，继续开工时按 [src/README 命名说明](../../../src/README.md)定位现行代码；尚未建立的 control / CLI 目录也按职责取名，不照抄上述历史路径。

| 文件 | 作者 | 内容 |
| --- | --- | --- |
| `01-plan.md` | Fable | P2 计划：定位与重述、起点核对、P2 的形状、工作包与分工、五项取舍的讨论与拍板、任务书要点、审核方式、研究层先行清单、延后与遗留、里程碑重切 |
| `02-research-brief.md` | Fable | 六份前置研究的任务书，Codex 写、GLM 审 |
| `03-repo-identity-discussion.md` | Fable | Repo 稳定身份的讨论底稿：Fable 的原理解（所有者判「理解问题很大」）与待聊的问题；已随 C 批撤题 |
| `04-p21-kickoff.md` | Fable | P2.1 开工书：v0.18.3 之后的约束变化落到哪个包、三包任务书（v0.18.11）、给 Codex 的第一包、待所有者定 |
| `05-p22-kickoff.md` | Fable | P2.2 开工书：落到戊己庚辛的裁决、四包任务书（v0.18.11）、顺序与前置、给 Codex 的第一包（指针版）、待所有者定 |
| `06-small-brain-engine-research.md` | Fable | 引擎调研任务书：持续建议、@ 时的缺口、前情提要起草三件活各用什么引擎（Jev、本地小模型、本地嵌入模型、机械规则），Grok 写，GLM 与 Fable 审 |
| `07-demo-kickoff.md` | Fable | 演示线开工书：三次演示、到「从聊天到合入」为止的九个包与席位、Agency 的定位、先不做的清单、头两包的开工提示词 |

| 级 | 工作包 | 状态 |
| --- | --- | --- |
| P2.1（旧 B0） | 前置研究三份 → 控制面存储与命令内核 → 进程与客户端边界 → 托管服务生命周期 | **已收口 2026-09-20**：开工书 `04` #271；存储与命令内核 Codex #276、进程与客户端边界 Grok #279、托管服务生命周期 Grok #282 已合（各有评审与作者说明）；打包压缩与 Gitea / tea 制品锁定 Codex #278 已合。修正：#282 把 `hctl2 start` 写成同时拉起 Gitea，所有者 09-21 指出混淆了纯本地仓库与外部平台 clone，改为按首次消费（Fable #285）。Gitea / tea 源码进源码伴随包 Codex #284 已合 |
| P2.2（旧 B1） | 研究复核 → Repo 注册 → 任务源与 Task（平台 issues 先）→ 聊天端口与 Room 树 → Project、名册、Request 与收口 | 开工书 `05` 已合 #273；chat 探针 #274；Repo 注册 Codex #287、任务源与 Task Codex #288、聊天端口与 Room 树 Codex #292 已合（#292 按 v0.19.0，含人工确认提要、未配置模型的机械草稿与恢复）。接口与 CT 对照见 [Repo 实现说明](../../../src/crates/repo/README.md)、[Task 实现说明](../../../src/crates/task/README.md)与 [Chat 实现说明](../../../src/crates/chat/README.md)。三件轻量判断与总结活的引擎调研 Grok #290 已合，结论是这版不在样本上跑模型、缺省用机械规则。**已收口 2026-10-03**：最后一包 Project、名册与 Request（`07` 第 1 包）Codex #307 已合；演示 1 由 Fable 验收通过，三处「人在 Cinny 里看不到」的缺口裁成约束缺省（v0.19.2）与补丁 1a（GLM 写，Grok、Qwen 审），另两处成小活 D、E，见 `07` §三、§四、§六 |
| P2.3 / P2.4（旧 B2） | 2a/2b 研究（已完成）→ `07` 的第 2 到第 9 包：Agency 服务骨架与端口、Agency 运行时与两家 harness、Context 组装、派工、变更与合入、完成 Task 与发布 Memo、命令行的人读输出、端到端收口；第 1 包的补丁 1a 与第 2 包并行 | 第 2 包评审中：Codex #317，`codex/agency-port`；Grok、DeepSeek 独立审。包 3 / 4 / 5 的代码任务说明分别在 `src/agency/README.md`、`src/crates/context/README.md`、`src/crates/participant/README.md`；顺序与演示见 `07` §三、§四 |
| P2.5（旧 B4/B5）与自举等级 A1–A4 | 入口见 `01` §四、§十 | 未开始 |

> 包名对照（所有者 2026-10-03：编号用数字，不用天干地支）：`07` 的第 1 包是旧任务书里的「辛」，第 2、3 包是「壬」拆成的两半，第 4 包是「子」，第 5 包是「癸」，第 6 包是「丑」，第 7 包是「寅」加发布 Memo，第 8 包是新增的命令行人读输出，第 9 包是「卯」。旧任务书（`01`、`04`、`05`）里的标题不回改。

> 2026-09-21 的遗留（Gitea 1.27.3 与 tea 0.15.1 的上游源码没进源码伴随包）已按所有者选的 a 由 Codex #284 补齐：`lock.json` 锁了两份源码归档，源码伴随包与 `sources.tsv` 里都有。

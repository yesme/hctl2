# S4 收口:Grok 全量删除安全审计

> 状态:讨论中 · S4 闸门报告(只列表,不改正文)<br>
> 对象:五簇累计 `git diff 192e7b6..origin/main -- docs/ README.md`(main @ `95cadc4`,草案 v0.15.1)<br>
> 对照:[`02-target-map.md`](./02-target-map.md) §3 全部处置与 §5、[`04-prohibition-whitelist.md`](./04-prohibition-whitelist.md)、[`01-inventory-dead-text.md`](./01-inventory-dead-text.md)<br>
> 去向:所有者一次裁决后由 GPT 合成修正 PR;本文件不进合同<br>
> 结论取值只能是:通过 / 回滚哪几句 / 进停车位

## ① 迁移表逐节核销;有无表外无记录删除

`192e7b6..main` 文件级删除只有两处,均在表内:`docs/design/references/implementation-evidence.md`(§3.22 delete);`docs/research/remote-control.md`(§3.23 split,正文在 `remote-control/codex-remote-feishu.md`,观察清单在 `remote-control/README.md`)。新增 `docs/design/contract-tests.md` 对应 §3.19 split。`docs/research/docs-lint.md` 是 S1 机械检查选型短记,不在删除面。

delete / fold-to-history / merge / split 对照现状:

| 表内处置 | 现状 |
| --- | --- |
| README 设计基线 / 六问 / 目标体验 merge 进 vision | 根 README 只留链接;权威在 `vision.md`「四个阶段」「目标体验」「设计原则」 |
| vision「从这里读下去」merge 进根 README 三条路径 | vision 无该节;根 README「阅读入口」三条仍在 |
| run.md「两种控制制度」merge 进 vision | 权威 `vision.md:91`;run 只留顺序特有句并引用 |
| spec/README 五张历史表 fold 进 decision-history | spec/README 已无五表;见 ③ |
| delivery 契约测试矩阵 split | delivery 留一句指向;十族在 `contract-tests.md` |
| implementation-evidence 整文件 delete | 文件不存在 |
| remote-control.md split | 根文件不存在;单案与清单在子目录,锚点 `e-l1-codex-remote-feishu` 仍在 |

标 keep 的承重节仍在:vision 五种失败 / 目标体验 / 不解决什么;四模块「为什么存在」与轻量路径 / 五种重试;delivery CLI / 明确不做 / P 表 / 切片 / 自举 / 选型 / 技术基线 / 打包「必须原生、不进 Docker」;spec 三类数据、两条终结来源、正常完成谓词、Room 与消息。角色表「不能做什么」列已按 N 整列删除,四表只留「可以做什么」。

未发现迁移表之外的无记录章节删除。C 类复述与 B 类负例的删改落在各簇 rewrite 行,不另开无记录项。

**结论:通过。**

## ② 必留 31 条与 A 类 15 处

白名单 §1 点名 A 类现均在权威位置(行号随 C3 重排,句子还在):

| 点名 | 现位置 |
| --- | --- |
| agent 三条底线 / ChangeSet 单写租约 | `spec/agent.md:34`、`:38` |
| run 正常完成谓词 / 失败隔离 | `spec/run.md:37`、`:39` |
| task Run claim / 完成 fail-closed / 两个终结来源 | `spec/task.md:64`、`:66`、`:68` |
| system 两类 actor / ACK 不盲重投 / 单写者 | `spec/system.md:101`(含「不能自报为 human」,C3 回滚已落地)、`:103`、`:167`/`:169` |
| connections 不能直接写目标 / 精确引用 / 旧代次只留审计 / 对账前不表现为已交接 | `spec/connections.md:10`、`:16`、`:115`、`:168` |

特别 tricky 6 条全留:`connections.md:115`/`:168`、`spec/system.md:103`、`spec/task.md:74` 不复活旧 Receipt、`spec/project.md:79` 不复活旧调用、`delivery.md:81` 结果未知不得签成功。

特别容易犯留下的对照句仍在(迁到 CT 的仍算留一句):`spec/agent.md:54` 模型自述已合并、`spec/project.md:97` mention 模糊匹配、`spec/run.md:35` Dagu 直接 mutation 标分歧、`docs/design/run.md:14` 引擎与 HCTL 不能互相冒充、`connections.md:147` 权限逐级缩小、`contract-tests.md:71` Herdr 归属只形成 drift、`:95` 换绑不冒充热切换、`:127` 聚合面不能改状态、`:88` actor 不能自报。转走 2 条:`agent.md` 关键规则已改为人话 + 引用 `spec/agent.md#写入合同`;`delivery.md:36` CLI 无隐藏权限句仍在(未删光)。

**结论:通过。**

## ③ 来时路折叠六节与五张表

编号 §1–§33 仍连续。六节对照:

| 节 | 表内要求 | 现状 |
| --- | --- | --- |
| §6 Conductor | 当时为何、被 §18 取代;Dagu 不误标 §19 | `:50` 标题写明后由 §18 取代;正文保留机械状态边界,选型交给 §18 |
| §13 P/B 与 P0 选型 | 双表留一句,选型段折叠 | `:139` 一段:当时建 P0–P6/B0–B6,选型由 §14/§18/§19/§29 与现行交付改判 |
| §18 Dagu | 选型理由留;只折 B4 阻断 | `:189` 判据与采用边界仍在;B4 缺口一句指向 §23 撤销 |
| §19 tmux | 升 macOS 15 的当时原因 + 被 §29 取代 | `:197` 写明因 Darwin 制品升 15,§29 取代 agentd+tmux,当前基线指向 delivery 打包策略 |
| §27 运行时 provider | 缩一段指 §29 | `:279` 当时同构化演进,实现由 §29 Herdr 取代,链到现行 agent/P0 |
| §28 中间方案 | 缩两句 | `:283` 标题已标被 §29 取代;正文只留 Agency 定名与 agentd 退场转折 |

五张核销表在 §11/§12/§14/§20/§22 尾,行数与 `192e7b6` spec/README 一致(12/4/3/3/7)。v0.9.1、v0.10.3、v0.11.1 表体逐字相同;v0.12.2 与 v0.13.0 各改一处相对链(`./connections.md` → `../spec/connections.md`),否则原样。台账 v0.15.1 大修行在 §32。

**结论:通过。**

## ④ §5 模块差异与五个易混边界

模块独有项仍在各自合同,未并进共享机制:Project 待确认注册与唯一 Repo Room、归档前置不隐式清场、Scoped Room 回填才能归档、Request 独占、mention 确定性;Task 契约惰性、HCTL-first/content-first、Run claim 双态、Done 信封、完成 fail-closed、不 reparent;Run Obligation 按观察序号铸造、五种重试、正常完成谓词、Gate 作者回避与 known/unknown、dynamic fork 有界模板;Agent ChangeSet 单写租约与不可证静默时换新 worktree+新 ChangeSet、`review_subject_digest` 与 `revision_digest` 分语义、终局结果契约与观测截断、终端五能力互不冒充。

五个易混边界主语仍分开:

1. 失败类 Run 不终结 Task(`spec/run.md:105`)vs Reopen/Deleted 只作来源事实(`spec/task.md:48`/`:68`)。
2. ChangeSet lease(`spec/agent.md:26`/`:38`)vs Task Run claim(`spec/task.md:64`)vs Agency owner lease(`spec/system.md:169`)。
3. chat server 不可用显示重同步中 vs 事后加密显示需要关注(`spec/connections.md:160`/`:161`)。
4. Dagu 直接 mutation 只标分歧(`spec/run.md:35`)vs Vikunja Done 可成完成请求(`spec/task.md:68` 与 system 动作分类)。
5. Harness 可读 common-dir/refs 与不获集成凭据同在 `spec/agent.md:34`。

Manifest 并集在 `spec/run.md:54–55`(含端口绑定、网络/secret、放置);usage 源码构建写明只用于更新托管 Tuwunel 制品(`docs/usage.md:231`)。system 两类 actor 仍在,只删了「不设在场证明」半句。打包策略整节仍在,只改 Tuwunel 与 macOS 依据。

**结论:通过。**

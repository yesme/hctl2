# 分层路由表：横向四层 + 纵向聚类

> 状态：已落地（路由表）· 待拍板（越层与归并处置）<br>
> 基线：main @ `b41ff6c`（草案 v0.15.4）<br>
> 去向：四层路由表已进 [`WRITING-GUIDE.md`](../../../WRITING-GUIDE.md) §0.4；本文余下的审计口径与纵向聚类进结构修订 PR

## 横向：四层颗粒度

HCTL2 现在是**两层**制度（设计层 / 合同层）。所有者要的是四层。重述后的差别只在两处细分：**愿景从架构里分出来**（说话方式不同：一个是说服，一个是讲清楚怎么组装），**交付与验证从合同里分出来**（目的不同：一个立约，一个核验）。其余不动。

| 层 | 回答什么 | 文风 | 允许词汇 | 规范性 | 文件 |
| --- | --- | --- | --- | --- | --- |
| ① 愿景 | 为什么存在、想给什么体验、按什么原则取舍 | 宣言（M1 叙事弧、M2 从约束推导、M6 密度与诚实） | 核心产品词 + 日常语言 | 规范性（原则层）；与合同冲突以合同为准 | `docs/design/vision.md`、根 `README.md` |
| ② 架构 | 由哪些模块与面组成、怎么组装达成目标、失败时怎么办 | 混合：抓大放小，先结论后论证 | 核心产品词 + 六个高频合同词（携中文对照） | 规范性（架构层） | `architecture.md`、`design/README.md`、四个模块设计正文 `project/task/run/agent.md`、横切 `participant.md`、`context.md` |
| ③ 机制 | 精确对象、状态机、写入者、不变量、互操作边界 | 技术（T2 约束优先于流程、T3 状态机五件套、T6 单源） | 合同层全部词汇 | 规范性合同；冲突时最终权威 | `spec/` 七个文件 |
| ④ 交付与验证 | 第一阶段做什么、怎么选型、怎么验收 | 技术（非规范） | 可引用合同词以指认被测合同，**不得重定义** | 非规范；不改变领域含义 | `delivery.md`、`contract-tests.md`、`docs/usage.md`、`research/README.md` |
| — 参考 | 对照与史料 | 表格 / 叙事 | 全部 | 非规范 | `references/glossary.md`、`references/decision-history.md` |

**每层读完应该能回答什么**（这层的验收标准，也是「替读者省力」这件事的可检验形式）：

- ① 读完能回答「为什么需要它、它不做什么」，**不需要知道任何对象名**。
- ② 读完能回答「有哪些模块和面、怎么组装、失败会怎样」，**不需要知道状态机与字段**。
- ③ 读完能实现内核或写一个 provider adapter，**不需要知道选了哪个产品**。
- ④ 读完能开工、能验收。

每层的失败判据同样明确：读者为了读懂 ① 必须翻 ③，就是 ① 写漏了；③ 里出现「为什么这样更好」的论证，就是该论证应该在 ① 或 ②。

**审计动作**：逐文件核对「这段实际的写法」和「这一层应有的写法」，产出越层清单，分两类——**下沉**（高层文件里出现了低层细节）与**上浮**（低层文件里出现了本该在高层的论证或愿景口号）。上一轮大修只查过下沉，没查过上浮。

**已知的层归属疑点**（需在审计中裁决，不预设答案）：

1. 四个模块设计正文（`project/task/run/agent.md`）与同名合同（`spec/*.md`）的边界是否稳定——设计正文里是否已经渗入状态机口径。
2. `design/README.md`（设计地图）同时承担索引、共同规则与文档纪律三件事，是否该拆。
3. `contract-tests.md` 归 ④ 还是 ③——它引用合同词但不立约，本表按 ④ 放置。
4. 根 `README.md` 是 ① 的浓缩还是独立门面层。

## 纵向：机制聚类

同一机制散在多个文件时，读者要跨文件拼图，也看不出两处讲的是不是同一件事。下表先列八个簇，不是全量，审计要补全并对每簇回答三个问题：**是同一机制吗？权威定义在哪？其余位置是引用还是复述？**

| 簇 | 成员（现有说法） | 主要落点 | 安全相关 |
| --- | --- | --- | --- |
| A 代次与 fence | `control_writer_generation`、`site_generation`、`engine_binding_generation`、`attempt_generation`、`runtime_generation`、Agency owner lease generation | `spec/system.md` §单写者、`spec/run.md`、`spec/agent.md` | ● |
| B 锁与租约 | `control.lock`、现场 OS 排他锁、Write Lease、Terminal Input Lease、Agency owner lease | `spec/system.md`、`spec/agent.md` | ● |
| C 冻结与摘要 | `revision_digest`、`review_subject_digest`、JCS 规范摘要、Snapshot 冻结、binding digest | `spec/system.md`、`spec/README.md` §三类数据、`spec/run.md` | ● |
| D 单写者与 CAS | 唯一 control writer、expected-version CAS、current pointer 推进 | `spec/system.md`、各模块写入合同 | ● |
| E 幂等与投递 | `idempotency_key`、outbox/inbox、`conflict_scope`、ACK 未知、重复命令返回原结果 | `spec/system.md` §命令与跨服务正确性、`spec/connections.md` | |
| F 失败与降级 | fail closed、安全暂停、待处理 / 需要关注、结果未知、分歧 | `architecture.md` §数据丢了怎么办、`spec/connections.md` §失败与恢复、`spec/system.md` 权威地图 | |
| G 恢复与对账 | 启动恢复七步、备份与恢复、drift、binding 分歧待对账 | `spec/system.md` §启动与恢复 | ● |
| H 权限与来源 | actor provenance 五值、`trust_level`、execution principal、service account、单用户模型、安全边界六条 | `spec/system.md` §命令 / §安全边界、`spec/agent.md` | ● |

A 是最强的归并候选：**六个代次，六处各自定义**，读者无法一眼看出它们是同族、层级关系是什么、哪些必须逐项携带。C 与 D 次之。

**归并候选必须分成两类交付**，不得混谈：

- **只改写法**（只改写法：抽一张共同语义表、其余位置改引用）→ 本轮语言/结构 PR 消化。
- **会改变含义**（合并会改变合同含义、状态数或写入者）→ **不在本轮落**，单独走设计变更路径（决策史转折 + CT 失败用例 + bump）。

判据：改完之后，任何一条 CT 用例的判定结果是否可能变化。可能变化的就归「会改变含义」这一类。

## 两种分割的嵌套

横向决定「这句话该不该出现在这个文件」，纵向决定「这件事该在哪个文件讲一次」。两者在文件内部继续嵌套：一个合同文件的节序按「概念 → 对象表 → 写入合同 → 失败与恢复 → 外部对齐」固定，相似机制相邻。审计到节粒度，不止文件粒度。

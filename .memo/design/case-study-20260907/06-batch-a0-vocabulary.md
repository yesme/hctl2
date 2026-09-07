# A0 词汇批的方案（讨论阶段）

> 状态：讨论中 · 按 `05-rewrite-process.md` 的规矩，方案先审后动手；本批不改语义<br>
> 基线：main @ `1d6ca5c`（草案 v0.17.1）<br>
> 去向：`README.md`、`docs/usage.md`、`WRITING-GUIDE.md`、`docs/design/**`（决策史除外）的 163 处"账本"；架构正文与词汇表的"前端"定义；`src/build/docs/dead_names.txt` 加"账本"；小修订台账一行；版本 v0.17.2

## 一、要改什么，来自哪里

所有者 2026-09-07 裁定"账本"太模糊，要按上下文写精确的标的物，已写进 CONSTRAINTS.md（#194）：控制面存储、治理记录、Git 正文或平台记录；决策史与 `.memo` 里的历史用词不改；机械检查随全库改口批一起落。同一轮定下 CLI 与 bench 统一叫前端，"执行面"只指内容系统与物理执行。本批就是把这两件事落到文本上，不动任何约束的含义。

范围内的 163 处按文件：`spec/system.md` 42、`spec/run.md` 13、`spec/project.md` 11、`architecture.md` 9、`participant.md` 9、`spec/connections.md` 9、`spec/repo.md` 9、`spec/task.md` 8、`context.md` 7、`contract-tests.md` 7、`delivery.md` 6、`vision.md` 6、`repo.md` 5、`spec/participant.md` 5、`spec/README.md` 4、`glossary.md` 4、`run.md` 3、`usage.md` 2、`WRITING-GUIDE.md` 2、根 `README.md` 1、`project.md` 1。研究文件与决策史不在范围内，`.memo` 不在范围内。

## 二、改法：按上下文分五类

"账本"在现文里指五种不同的东西，一类一个替换词；同一类里再按搭配定句式。全表见附录，这里给规则与例子。

| 类 | 现文的样子 | 改成 | 例子 |
| --- | --- | --- | --- |
| 甲 存储本身 | metadata 账本、用户级账本、控制面账本、control 账本、HCTL 账本、治理账本、权威账本、账本备份、账本快照、账本身份、账本写权、账本写入者、分叉账本、账本文件 | 控制面存储（用户级的写"用户级控制面存储"） | "用户级 metadata 账本与写锁"改"用户级控制面存储与写锁"；"不得合并两份分叉账本"改"不得合并两份分叉的控制面存储" |
| 乙 存储的事务 | 账本事务、同一账本事务、用户级账本事务 | 控制面事务 | "在同一账本事务里准入提案与版本"改"在同一控制面事务里准入提案与版本" |
| 丙 存储里的记录 | 账本事实、账本谓词、账本记录、进入账本、不进权威账本、Run 账本、五模块账本、源账本、同步账本 | 治理记录（任务后端那处"同步账本"改"同步记录"） | "Run 正常完成只由账本谓词决定"改"只由治理记录的谓词决定"；"纪要不进权威账本"改"纪要不进治理记录" |
| 丁 存储作为动作主语 | 账本接受、账本记 known、账本标 unknown、账本只保存、账本独占、账本给门 | 接受这类动作的主语改"控制面"，保存记录这类动作的主语改"控制面存储" | "账本接受这个版本，这是准入"改"控制面接受这个版本，这是准入"；"账本只保存引用与 digest"改"控制面存储只保存引用与 digest" |
| 戊 与 Git、平台对举 | 账本与 Git、权威在账本、第二本账、Git 里只有审计影子 | 对举时写"控制面存储"；"第二本账"改"第二个权威"；说 Git 那半时写"Git 正文"；说平台那半时写"平台记录" | "凭证的权威在账本，平台丢失后不得凭 Git 提交重建"改"凭证的权威在控制面存储，平台丢失后不得凭 Git 提交重建"；"它不是账本，也不是事实源"改"它不是控制面存储，也不是事实源" |

还有一个动词"记账"：愿景里"无法保持先记账再执行顺序"改"先记录再执行"；架构里"保存是工具箱的事，算数是账本的事"改"算数是控制面的事"。

**为什么按上下文而不一律替换**：所有者的原话是"用精确的标的物指代"，五家评审里 Codex 的意见是按上下文分别写，GLM 主张一律写"控制面存储"；一律替换会把一个模糊词换成另一个笼统词，"账本谓词"变成"控制面存储谓词"读不通，"进入账本"变成"进入控制面存储"把"记录"这层意思丢了。这一点所有者在 #196 之后已经认可，本批不重开。

**第 13 条之后"控制面存储"有两半**（`03-governance-text.md` §十）：治理记录在结构化存储，治理正文在材料仓库。本批不区分这两半，因为现文里的"账本"全指记录那一半；第 13 条落地时（C 批）再在需要的句子里区分。

## 三、"前端"的定义

- `architecture.md` §三个面 展示面那行："Workbench、CLI 与第三方场景客户端"改"前端（Workbench 与 CLI 的统称）与第三方场景客户端"。
- `glossary.md` 系统组件表加一行：前端，Workbench 与 CLI 的统称，展示面的实例，不拥有事实。
- 用例里的误用已改（#195）；设计文本里"执行面"现在都指内容系统与物理执行，不用动。
- 本批只给名字，不写"一个前端连多个控制面"，那是 A 批的语义。

## 四、机械检查

沿用 `src/build/docs/check_dead_names.sh`：它扫全部文档，排除决策史、`.memo`、`docs/research`，中文名按子串匹配，正好是 CONSTRAINTS 写的范围。做法：`dead_names.txt` 加一行"账本"，出处写 CONSTRAINTS 2026-09-07；词汇表留旧称对照那一行用 `dead_names.allowlist` 按子串豁免。不新增脚本，不改脚本。

## 五、版本、台账、契约测试

不改约束含义，按 v0.15.6 全库语言收口的先例：版本 v0.17.1 升 v0.17.2，23 处版本戳同步；小修订台账加一行"账本改口与前端定名，不改约束语义"，指向 CONSTRAINTS 与本方案；不加契约测试用例。

## 六、方案六样东西的对照

1. 要改哪些句子、来自哪里：第一节与附录。
2. 每处的改法：第二节的五类；附录逐行给了建议改法。
3. 好处坏处：按上下文改的好处是每处都指到实物，坏处是评审要逐行看 163 处；一律替换的好处是机械，坏处是换成另一个笼统词。
4. 推荐：按上下文改，附录的建议改法作为起点，评审逐行核。
5. 已拍板不重开：禁用"账本"；按上下文写精确标的物；决策史与 `.memo` 不改；机械检查随本批落。
6. 落点：第一节的文件清单、第三节的两处、第四节的两个文件、第五节的版本与台账。

参考用例 S1：本批不改任何行为，用例各步与失败路径不受影响。

## 七、请评审看的三件事

1. 五类的划分对不对，有没有第六类；附录里标错类的逐行指出。
2. 丁类"主语是控制面还是控制面存储"的判法能不能接受：接受、准入这类判断动作的主语是控制面这个单元，保存、记、标这类动作的主语是它的存储。
3. `WRITING-GUIDE.md` 里两处是写作指南举的例句，改口后例句仍成立吗。

## 附录：163 处逐行建议（原文片段 → 建议改法，评审逐行核）

| 位置 | 原文片段 | 建议改法 |
| --- | --- | --- |
| README.md:64 | architecture.md)；组件职责与账本的精确划分见[系统边界的组件表](./doc | architecture.md)；组件职责与控制面存储的精确划分见[系统边界的组件表](./doc |
| docs/usage.md:15 | t` 回读闭集外部事实。它不产生 HCTL 账本、Receipt 或 Verdict，也不做 | t` 回读闭集外部事实。它不产生 HCTL 的控制面存储、Receipt 或 Verdict，也不做 |
| docs/usage.md:193 | 期头、幂等键。工具箱不发明 ID，不读、不写账本。 | 期头、幂等键。工具箱不发明 ID，不读、不写控制面存储。 |
| WRITING-GUIDE.md:231 | 约束：存在活动写入者时，其代次**必须**是账本中的最大值。观察到活动写入者持有非最大代次， | 约束：存在活动写入者时，其代次**必须**是控制面存储中的最大值。观察到活动写入者持有非最大代次， |
| WRITING-GUIDE.md:449 | > SQLite 事务只保证账本内部一致，**而**事务提交与外部投递不在同 | > SQLite 事务只保证控制面存储内部一致，**而**事务提交与外部投递不在同 |
| docs/design/architecture.md:14 | ｜ 控制面 ｜ HCTL 自己的命令服务、账本与现场执行者（组件划分见[系统边界](./s | ｜ 控制面 ｜ HCTL 自己的命令服务、控制面存储与现场执行者（组件划分见[系统边界](./s |
| docs/design/architecture.md:58 | ｜ 场景 ｜ metadata（控制面账本） ｜ content（执行面系统） ｜ a | ｜ 场景 ｜ metadata（控制面存储） ｜ content（执行面系统） ｜ a |
| docs/design/architecture.md:68 | 文，身份、准入、当前指针与裁决仍独占在控制面账本。Git 里出现一份正文或副本，不等于它已被 | 文，身份、准入、当前指针与裁决仍独占在控制面存储。Git 里出现一份正文或副本，不等于它已被 |
| docs/design/architecture.md:68 | ，不等于它已被 HCTL 接纳。哪些记录属于账本、哪些属于 Git，精确划分见[系统边界的 | ，不等于它已被 HCTL 接纳。哪些记录属于控制面存储、哪些属于 Git，精确划分见[系统边界的 |
| docs/design/architecture.md:81 | 用 ｜ 承诺进入治理；批准施工图与开工是两件账本事实，可在一次预览里提交 ｜ | 用 ｜ 承诺进入治理；批准施工图与开工是两件治理记录，可在一次预览里提交 ｜ |
| docs/design/architecture.md:84 | → 版本准入 ｜ 保存是工具箱的事，算数是账本的事：先封存回读，再在同一账本事务里准入提案 | → 版本准入 ｜ 保存是工具箱的事，算数是控制面的事：先封存回读，再在同一控制面事务里准入提案 |
| docs/design/architecture.md:84 | 箱的事，算数是账本的事：先封存回读，再在同一账本事务里准入提案与版本；封存本身在事务之外，封 | 箱的事，算数是控制面的事：先封存回读，再在同一控制面事务里准入提案与版本；封存本身在事务之外，封 |
| docs/design/architecture.md:96 | 冻结的本地事实照旧存在。完成与评审都在控制面账本里判，引擎只报告执行走到了哪一步；引擎停报进 | 冻结的本地事实照旧存在。完成与评审都在控制面存储里判，引擎只报告执行走到了哪一步；引擎停报进 |
| docs/design/architecture.md:104 | - **metadata**：控制面账本是唯一不可再生的权威，必须有备份；判决的结晶 | - **metadata**：控制面存储是唯一不可再生的权威，必须有备份；判决的结晶 |
| docs/design/context.md:32 | Run 里产出者、评审者已结晶的产出与裁决（账本与 Git） ｜ 工作流引擎里的东西——引擎 | Run 里产出者、评审者已结晶的产出与裁决（控制面存储与 Git） ｜ 工作流引擎里的东西——引擎 |
| docs/design/context.md:45 | ｜ 聊天史、任务后端评论线、平台评审评论线、账本里的裁决与凭证 ｜ | ｜ 聊天史、任务后端评论线、平台评审评论线、控制面存储里的裁决与凭证 ｜ |
| docs/design/context.md:47 | Git 对象和 worktree 路径。指向账本、任务后端或代码协作平台评论的引用不是指针， | Git 对象和 worktree 路径。指向控制面存储、任务后端或代码协作平台评论的引用不是指针， |
| docs/design/context.md:77 | 指向纪要，只能指向精确事件；丢了就重建，不进账本，清单只记引用与指纹。它也不由房间里的模型 | 指向纪要，只能指向精确事件；丢了就重建，不进治理记录，清单只记引用与指纹。它也不由房间里的模型 |
| docs/design/context.md:83 | 史，也不是父子传承，而是横向接力；它的存储是账本和 Git，不是工作流引擎。引擎只报告执行进 | 史，也不是父子传承，而是横向接力；它的存储是控制面存储和 Git，不是工作流引擎。引擎只报告执行进 |
| docs/design/context.md:86 | t（裁决）正文小、必用，直接内联。它的权威在账本，Git 里只有结晶副本（公开仓库可能只剩摘 | t（裁决）正文小、必用，直接内联。它的权威在控制面存储，Git 里只有结晶副本（公开仓库可能只剩摘 |
| docs/design/context.md:86 | 结晶副本（公开仓库可能只剩摘要），所以物化以账本记录为准，副本只作指针。 | 结晶副本（公开仓库可能只剩摘要），所以物化以治理记录为准，副本只作指针。 |
| docs/design/contract-tests.md:63 | - 超时与候选切换只依据账本自己的 Obligation deadlin | - 超时与候选切换只依据控制面存储自己的 Obligation deadlin |
| docs/design/contract-tests.md:71 | - Run 正常完成只由账本谓词决定；引擎报告的进度与账本不一致时标为分 | - Run 正常完成只由治理记录的谓词决定；引擎报告的进度与控制面存储不一致时标为分 |
| docs/design/contract-tests.md:71 | 正常完成只由账本谓词决定；引擎报告的进度与账本不一致时标为分歧待对账，既不补足也不阻止谓词 | 正常完成只由治理记录的谓词决定；引擎报告的进度与控制面存储不一致时标为分歧待对账，既不补足也不阻止谓词 |
| docs/design/contract-tests.md:101 | 自带的接管/单写者/"会话有效"记录被当作账本事实或替代租约/代次时拒绝 | 自带的接管/单写者/"会话有效"记录被当作治理记录或替代租约/代次时拒绝 |
| docs/design/contract-tests.md:152 | ChangeSet Revision 在同一账本事务；工具箱封存回读先于准入，缺任一步不产生 | ChangeSet Revision 在同一控制面事务；工具箱封存回读先于准入，缺任一步不产生 |
| docs/design/contract-tests.md:163 | - 同一用户级账本只能有一个 control writer，第 | - 同一用户级控制面存储只能有一个 control writer，第 |
| docs/design/contract-tests.md:171 | - metadata 账本执行一致性 backup、restore p | - 控制面存储执行一致性 backup、restore p |
| docs/design/delivery.md:41 | CLI 没有隐藏权限，也不直接写治理账本、执行面 content 服务器或 Agen | CLI 没有隐藏权限，也不直接写控制面存储、执行面 content 服务器或 Agen |
| docs/design/delivery.md:78 | 已使用的 content 后端与平台连接后，账本、Git 工作树归属、integration | 已使用的 content 后端与平台连接后，控制面存储、Git 工作树归属、integration |
| docs/design/delivery.md:86 | 观察到 Engine 检查点进入等待态，在账本创建 Obligation/Seat/Att | 观察到 Engine 检查点进入等待态，在控制面存储创建 Obligation/Seat/Att |
| docs/design/delivery.md:115 | 保有平行 Project/Task/Run 账本。降级超过约定能力时回退到上一自举级别并留下 | 保有平行 Project/Task/Run 的治理记录。降级超过约定能力时回退到上一自举级别并留下 |
| docs/design/delivery.md:139 | ligation 身份与隔离仍由 HCTL 账本承担，结论与固定源码证据见 [Dagu 与候 | ligation 身份与隔离仍由 HCTL 的控制面存储承担，结论与固定源码证据见 [Dagu 与候 |
| docs/design/delivery.md:169 | 测试钉官方测试向量）、现场锁用标准库文件锁、账本备份用 SQLite Online Back | 测试钉官方测试向量）、现场锁用标准库文件锁、控制面存储备份用 SQLite Online Back |
| docs/design/participant.md:37 | 上三层归控制面账本，由用户建立和绑定；下四层的实物由 Agen | 上三层归控制面存储，由用户建立和绑定；下四层的实物由 Agen |
| docs/design/participant.md:37 | 与执行体](#agency-与执行体)），账本只冻结对它们的精确引用。调用前的 Trigg | 与执行体](#agency-与执行体)），控制面存储只冻结对它们的精确引用。调用前的 Trigg |
| docs/design/participant.md:80 | 对远程参与者，方法论版本只是对方的一句申报，账本标 unknown，第三件只能问到申报为止。 | 对远程参与者，方法论版本只是对方的一句申报，控制面存储标 unknown，第三件只能问到申报为止。 |
| docs/design/participant.md:82 | - **Skill 给方法，账本给门。** 方法论 Skill 决定怎么问、 | - **Skill 给方法，控制面存储给门。** 方法论 Skill 决定怎么问、 |
| docs/design/participant.md:82 | 些门放在 prompt 里必然失守，所以只在账本里。 | 些门放在 prompt 里必然失守，所以只在控制面存储里。 |
| docs/design/participant.md:84 | 行体；派出不转移参与者身份，“谁在工作”仍由账本的 Participant 层回答，派出方只 | 行体；派出不转移参与者身份，“谁在工作”仍由控制面存储的 Participant 层回答，派出方只 |
| docs/design/participant.md:90 | Agency 不制造参与者身份，身份由用户在账本里建；一般也不制造端点，远程数字员工是既存的 | Agency 不制造参与者身份，身份由用户在控制面存储里建；一般也不制造端点，远程数字员工是既存的 |
| docs/design/participant.md:92 | 执行。身份、授权、人设仍由 control 账本拥有。 | 执行。身份、授权、人设仍由 控制面存储拥有。 |
| docs/design/participant.md:93 | 、什么指纹），由执行体装载，由工具箱核验，由账本记 known 或 unknown。 | 、什么指纹），由执行体装载，由工具箱核验，由控制面存储记 known 或 unknown。 |
| docs/design/project.md:84 | o 模块激活仓库身份，Project 在同一账本事务里建 Repo Room；Project | o 模块激活仓库身份，Project 在同一控制面事务里建 Repo Room；Project |
| docs/design/repo.md:21 | *：逻辑仓库的稳定身份，写进 Git 也记在账本；一个仓库可以显式挂接多个仓库实例——某台机 | *：逻辑仓库的稳定身份，写进 Git 也记在控制面存储；一个仓库可以显式挂接多个仓库实例——某台机 |
| docs/design/repo.md:53 | 把内容封存成 Git 快照并回读，这是保存；账本接受这次结果和这个版本，这是准入。封存期间被 | 把内容封存成 Git 快照并回读，这是保存；控制面接受这次结果和这个版本，这是准入。封存期间被 |
| docs/design/repo.md:68 | 意图，执行与回读时对照；上游悄悄改规则不能让账本失序。 | 意图，执行与回读时对照；上游悄悄改规则不能让控制面存储失序。 |
| docs/design/repo.md:71 | 认的保持未知、继续占用冲突范围。凭证的权威在账本，平台丢失后不能凭 Git 提交重建凭证。 | 认的保持未知、继续占用冲突范围。凭证的权威在控制面存储，平台丢失后不能凭 Git 提交重建凭证。 |
| docs/design/repo.md:94 | 地平台，或显式不挂——Project 在同一账本事务里建 Repo Room；Project | 地平台，或显式不挂——Project 在同一控制面事务里建 Repo Room；Project |
| docs/design/run.md:30 | 为准绳，不随对象所有权走。判决的权威在控制面账本，结晶副本进 Git（私有仓库默认全文，公开 | 为准绳，不随对象所有权走。判决的权威在控制面存储，结晶副本进 Git（私有仓库默认全文，公开 |
| docs/design/run.md:52 | icipant 身份，不是随便一个进程；票是账本里附证据、可核的记录。别家买独立性的办法有三 | icipant 身份，不是随便一个进程；票是控制面存储里附证据、可核的记录。别家买独立性的办法有三 |
| docs/design/run.md:52 | 换厂商」可以是施工图里一条可声明的席位策略，账本记着每个 Participant 来自哪个 | 换厂商」可以是施工图里一条可声明的席位策略，控制面存储记着每个 Participant 来自哪个 |
| docs/design/vision.md:78 | un，授予有边界的自主权。批准施工图与开工是账本里的两件事实，复用现成施工图时可以在一次预览 | un，授予有边界的自主权。批准施工图与开工是控制面存储里的两件事实，复用现成施工图时可以在一次预览 |
| docs/design/vision.md:103 | 批准 Workflow 与启动 Run 是账本里的两件事实：前者确认施工图，后者才允许系统 | 批准 Workflow 与启动 Run 是控制面存储里的两件事实：前者确认施工图，后者才允许系统 |
| docs/design/vision.md:123 | ｜ 持久账本与对账 ｜ outbox/回读、崩溃恢复和投 | ｜ 持久的控制面存储与对账 ｜ outbox/回读、崩溃恢复和投 |
| docs/design/vision.md:134 | ta（治理元数据）**住在 HCTL 自己的账本，**content（场景内容）**住在各场 | ta（治理元数据）**住在 HCTL 自己的控制面存储，**content（场景内容）**住在各场 |
| docs/design/vision.md:134 | 此隔离，聊天服务器宕机不阻断治理与施工，治理账本也不因会话丢失而失忆。 | 此隔离，聊天服务器宕机不阻断治理与施工，控制面存储也不因会话丢失而失忆。 |
| docs/design/vision.md:159 | 原生客户端相同的模块约束。界面不能直接写治理账本，无法保持先记账再执行顺序的引擎修改不作为正 | 原生客户端相同的模块约束。界面不能直接写控制面存储，无法保持先记账再执行顺序的引擎修改不作为正 |
| docs/design/spec/README.md:76 | 谁批了什么、凭什么算数 ｜ HCTL 自己的账本（控制面） ｜ | 谁批了什么、凭什么算数 ｜ HCTL 自己的控制面存储（控制面） ｜ |
| docs/design/spec/README.md:104 | 对应”只是引入差异化语义的强信号，不是控制面账本的完整存储清单。事实是否进入账本仍取决于它是 | 对应”只是引入差异化语义的强信号，不是控制面存储的完整存储清单。事实是否进治理记录仍取决于它是 |
| docs/design/spec/README.md:104 | ，不是控制面账本的完整存储清单。事实是否进入账本仍取决于它是否有独立生命周期、恢复或权限边界 | ，不是控制面存储的完整存储清单。事实是否进治理记录仍取决于它是否有独立生命周期、恢复或权限边界 |
| docs/design/spec/README.md:104 | 生承载的可变 content 与内部拓扑不在账本复制；HCTL 自己的稳定身份、领域关系、授 | 生承载的可变 content 与内部拓扑不在控制面存储复制；HCTL 自己的稳定身份、领域关系、授 |
| docs/design/spec/connections.md:12 | trol 在同一本用户级 metadata 账本的一个事务中提交；跨 Project、Rep | trol 在同一本用户级控制面存储的一个事务中提交；跨 Project、Rep |
| docs/design/spec/connections.md:47 | est、Task Run 占用标记、Run 账本和引擎启动 outbox ｜ run ID | est、Task Run 占用标记、Run 的治理记录和引擎启动 outbox ｜ run ID |
| docs/design/spec/connections.md:52 | 属者状态、代次与租约；归属模块准入提案的同一账本事务里，Repo 模块准入 ChangeSe | 属者状态、代次与租约；归属模块准入提案的同一控制面事务里，Repo 模块准入 ChangeSe |
| docs/design/spec/connections.md:58 | ource event cursor，可从源账本重建 ｜ | ource event cursor，可从来源的治理记录重建 ｜ |
| docs/design/spec/connections.md:77 | control 在一个用户级账本事务中写 Run、Manifest、幂等结果 | control 在一个用户级控制面存储事务中写 Run、Manifest、幂等结果 |
| docs/design/spec/connections.md:118 | 3. control 在用户级账本事务中记录归属者到 Execution Ru | 3. control 在用户级控制面存储事务中记录归属者到 Execution Ru |
| docs/design/spec/connections.md:158 | ntrol 在同一用户级 metadata 账本事务中以比较并交换校验 Project Re | ntrol 在同一用户级控制面存储事务中以比较并交换校验 Project Re |
| docs/design/spec/connections.md:199 | 本地事实继续存在；Run 的完成与评审只依据账本推进，Run–Engine Binding | 本地事实继续存在；Run 的完成与评审只依据控制面存储推进，Run–Engine Binding |
| docs/design/spec/connections.md:205 | ｜ 场景投影丢失 ｜ 从五模块账本和 source event cursor | ｜ 场景投影丢失 ｜ 从五模块的治理记录和 source event cursor |
| docs/design/spec/participant.md:29 | 由 Agency 安装并申报，由执行体装载；账本只保存引用与 digest。 | 由 Agency 安装并申报，由执行体装载；控制面存储只保存引用与 digest。 |
| docs/design/spec/participant.md:45 | s 都使用窄执行主体。以下三条底线不可关闭；账本单写者另有自己的[三条底线](./syste | s 都使用窄执行主体。以下三条底线不可关闭；控制面存储单写者另有自己的[三条底线](./syste |
| docs/design/spec/participant.md:83 | 权、人设和 Seat 仍由 control 账本拥有。 | 权、人设和 Seat 仍由 控制面存储拥有。 |
| docs/design/spec/participant.md:85 | 格、审计与恢复等级裁决只在 control 账本。Agency 自带的接管、单写者或“会话有 | 格、审计与恢复等级裁决只在 控制面存储。Agency 自带的接管、单写者或“会话有 |
| docs/design/spec/participant.md:85 | ”记录只作执行协助与观测证据，不得写入或替代账本事实。 | ”记录只作执行协助与观测证据，不得写入或替代治理记录。 |
| docs/design/spec/project.md:27 | ｜ chat server 时间线与治理事件账本都只追加；Project Room 随 Pr | ｜ chat server 时间线与治理事件控制面存储都只追加；Project Room 随 Pr |
| docs/design/spec/project.md:37 | 唯一 Repo 身份，Project 在同一账本事务中创建其唯一 Repo Room；待确认 | 唯一 Repo 身份，Project 在同一控制面事务中创建其唯一 Repo Room；待确认 |
| docs/design/spec/project.md:39 | “创建 Project”命令在同一账本事务中创建该 Project 的唯一 Pro | “创建 Project”命令在同一控制面事务中创建该 Project 的唯一 Pro |
| docs/design/spec/project.md:39 | roject Room。Room 身份与治理账本在用户级控制面；任何已挂接现场打开的都是同一 | roject Room。Room 身份与控制面存储在用户级控制面；任何已挂接现场打开的都是同一 |
| docs/design/spec/project.md:64 | 只用于身份或展示。HCTL 治理事件在控制面账本只追加，以 Room–Server Bind | 只用于身份或展示。HCTL 治理事件在控制面存储只追加，以 Room–Server Bind |
| docs/design/spec/project.md:76 | 的 Git 对象或 worktree 路径；账本、任务后端内容和代码协作平台上的评审评论不得 | 的 Git 对象或 worktree 路径；控制面存储、任务后端内容和代码协作平台上的评审评论不得 |
| docs/design/spec/project.md:108 | ask Backend Snapshot 与账本增量维护，不进入权威账本；删除后可以完整重建 | ask Backend Snapshot 与控制面存储增量维护，不进入控制面存储；删除后可以完整重建 |
| docs/design/spec/project.md:108 | Snapshot 与账本增量维护，不进入权威账本；删除后可以完整重建。相关性门默认只以账本事 | Snapshot 与控制面存储增量维护，不进入控制面存储；删除后可以完整重建。相关性门默认只以控制面存储事 |
| docs/design/spec/project.md:108 | 威账本；删除后可以完整重建。相关性门默认只以账本事实——提及、认领、Request 关联和游 | 威控制面存储；删除后可以完整重建。相关性门默认只以治理记录——提及、认领、Request 关联和游 |
| docs/design/spec/project.md:118 | 不得指向纪要，只能指向精确事件；纪要不进权威账本，被使用时 Bundle 只记其引用与摘要， | 不得指向纪要，只能指向精确事件；纪要不进控制面存储，被使用时 Bundle 只记其引用与摘要， |
| docs/design/spec/project.md:173 | event）；HCTL 治理事件只在控制面账本追加，以事件 ID 精确引用消息，不占领域对 | event）；HCTL 治理事件只在控制面存储追加，以事件 ID 精确引用消息，不占领域对 |
| docs/design/spec/repo.md:47 | epo-注册与-project-归档)在同一账本事务中创建其唯一 Repo Room；待确认 | epo-注册与-project-归档)在同一控制面事务中创建其唯一 Repo Room；待确认 |
| docs/design/spec/repo.md:51 | it 身份，再由 control 预览并写入账本。相同 Git 公共目录的重试返回原现场；不 | it 身份，再由 control 预览并写入控制面存储。相同 Git 公共目录的重试返回原现场；不 |
| docs/design/spec/repo.md:73 | 树内容封存成 Git 字节并回读，这是保存；账本接受这个版本，这是准入。 | 树内容封存成 Git 字节并回读，这是保存；控制面接受这个版本，这是准入。 |
| docs/design/spec/repo.md:75 | 准入 Result Proposal 的同一账本事务里，本模块准入 ChangeSet Re | 准入 Result Proposal 的同一控制面事务里，本模块准入 ChangeSet Re |
| docs/design/spec/repo.md:75 | est，交给准入事务。封存的 Git 写入在账本事务之外（不同原子域）：封存意图以提案标识符 | est，交给准入事务。封存的 Git 写入在控制面事务之外（不同原子域）：封存意图以提案标识符 |
| docs/design/spec/repo.md:126 | ol 在归属者准入提案与本模块准入版本的同一账本事务里持久化发布意图与 outbox，act | ol 在归属者准入提案与本模块准入版本的同一控制面事务里持久化发布意图与 outbox，act |
| docs/design/spec/repo.md:161 | 系统事实权威地图)。Receipt 的权威在账本，Git 里只有审计影子；平台丢失后不得凭 | 系统事实权威地图)。Receipt 的权威在控制面存储，Git 里只有审计影子；平台丢失后不得凭 |
| docs/design/spec/repo.md:171 | rrit change ｜ 写入边界与租约在账本，分支只是载体 ｜ | rrit change ｜ 写入边界与租约在控制面存储，分支只是载体 ｜ |
| docs/design/spec/repo.md:174 | 凭证只在回读到合并提交与目标头后签发，权威在账本 ｜ | 凭证只在回读到合并提交与目标头后签发，权威在控制面存储 ｜ |
| docs/design/spec/run.md:27 | （只认当前观察，见“从节点到结果”），此后按账本内的 Attempt 结果与 Gate 策略 | （只认当前观察，见“从节点到结果”），此后按控制面存储内的 Attempt 结果与 Gate 策略 |
| docs/design/spec/run.md:31 | n 状态只由 control 根据获准命令和账本事实推进，workflow engine 回 | n 状态只由 control 根据获准命令和治理记录推进，workflow engine 回 |
| docs/design/spec/run.md:54 | 擎报告的进度只用于分歧检测；该进度不可读或与账本不一致时，control 只把 Run–En | 擎报告的进度只用于分歧检测；该进度不可读或与控制面存储不一致时，control 只把 Run–En |
| docs/design/spec/run.md:62 | ligation 的身份、截止时间与租约都是账本事实；引擎侧的 DAG run ID 与不可 | ligation 的身份、截止时间与租约都是治理记录；引擎侧的 DAG run ID 与不可 |
| docs/design/spec/run.md:62 | 联键，代次不在引擎。超时与备用候选准入只依据账本自己的 Obligation 截止时间。引擎 | 联键，代次不在引擎。超时与备用候选准入只依据控制面存储自己的 Obligation 截止时间。引擎 |
| docs/design/spec/run.md:64 | 干成了的证明”）：权威在 metadata 账本，结晶副本按[系统存储约束](./syste | 干成了的证明”）：权威在 控制面存储，结晶副本按[系统存储约束](./syste |
| docs/design/spec/run.md:68 | t。Git 保存不可变正文；control 账本独占身份、准入、摘要和批准/current | t。Git 保存不可变正文；控制面存储独占身份、准入、摘要和批准/current |
| docs/design/spec/run.md:82 | Task 的 Run 占用标记。在同一用户级账本事务中，control 创建 Run、不可变 | Task 的 Run 占用标记。在同一用户级控制面存储事务中，control 创建 Run、不可变 |
| docs/design/spec/run.md:108 | 确认回执未知时先回读再重投。引擎报告的进度与账本不一致时——例如检查点已被引擎自行推进、从界 | 确认回执未知时先回读再重投。引擎报告的进度与控制面存储不一致时——例如检查点已被引擎自行推进、从界 |
| docs/design/spec/run.md:122 | nter` 并附分片建议。Verdict 以账本记录物化，其 Git 结晶副本只作 `poi | nter` 并附分片建议。Verdict 以治理记录物化，其 Git 结晶副本只作 `poi |
| docs/design/spec/run.md:160 | 随后 Task 按同一个用户级账本中的当前 Revision、来源的新鲜度与分 | 随后 Task 按同一个用户级控制面存储中的当前 Revision、来源的新鲜度与分 |
| docs/design/spec/run.md:162 | 完成谓词只依据账本事实与外部证据的当前回读。引擎停报进度时，c | 完成谓词只依据治理记录与外部证据的当前回读。引擎停报进度时，c |
| docs/design/spec/run.md:173 | 观察其等待态后创建 Obligation，账本结果落定后再推进检查点；场景客户端不得直接操 | 观察其等待态后创建 Obligation，控制面存储结果落定后再推进检查点；场景客户端不得直接操 |
| docs/design/spec/system.md:11 | l` ｜ 唯一领域命令服务，负责路由、权限、账本、outbox 和对账；内含 Herdr 适 | l` ｜ 唯一领域命令服务，负责路由、权限、控制面存储、outbox 和对账；内含 Herdr 适 |
| docs/design/spec/system.md:142 | 储只有一本库：**用户级 metadata 账本**。它是全部 metadata 的唯一权威 | 储只有一本库：**用户级控制面存储**。它是全部 metadata 的唯一权威 |
| docs/design/spec/system.md:142 | dict/Receipt。一人多机连接同一本账本，账本必须备份。 | dict/Receipt。一人多机连接同一份控制面存储，控制面存储必须备份。 |
| docs/design/spec/system.md:142 | t/Receipt。一人多机连接同一本账本，账本必须备份。 | t/Receipt。一人多机连接同一份控制面存储，控制面存储必须备份。 |
| docs/design/spec/system.md:144 | OS 锁、跟踪记录与可丢弃缓存。它**不是账本，也不是事实源**。现场状态始终可以从 me | OS 锁、跟踪记录与可丢弃缓存。它**不是控制面存储，也不是事实源**。现场状态始终可以从 me |
| docs/design/spec/system.md:144 | **。现场状态始终可以从 metadata 账本、Git 与运行时观测对账重建；删除该目录不 | **。现场状态始终可以从 控制面存储、Git 与运行时观测对账重建；删除该目录不 |
| docs/design/spec/system.md:146 | 副作用的目标，不是另一份 metadata 账本：获准的不可变正文与判决审计影子经工具箱写入 | 副作用的目标，不是另一份 控制面存储：获准的不可变正文与判决审计影子经工具箱写入 |
| docs/design/spec/system.md:148 | 账本只保存 HCTL 自己的领域关系、授权与判决 | 控制面存储只保存 HCTL 自己的领域关系、授权与判决 |
| docs/design/spec/system.md:148 | napshot 对账受治理的那部分外部关系。账本与其余本地存储（锁、缓存、定义文件）的物理布 | napshot 对账受治理的那部分外部关系。控制面存储与其余本地存储（锁、缓存、定义文件）的物理布 |
| docs/design/spec/system.md:148 | 成对外 API，也不进 Git；“唯一用户级账本、权威归属和备份传承”由架构约束固定，独立于 | 成对外 API，也不进 Git；“唯一用户级控制面存储、权威归属和备份传承”由架构约束固定，独立于 |
| docs/design/spec/system.md:154 | .lock —— 用户级 metadata 账本与写锁 | .lock —— 用户级控制面存储与写锁 |
| docs/design/spec/system.md:166 | 不因此取得这些正文的准入权；control 账本独占稳定身份、准入决定、规范摘要、curre | 不因此取得这些正文的准入权；控制面存储独占稳定身份、准入决定、规范摘要、curre |
| docs/design/spec/system.md:166 | 中出现一份正文不表示已被 HCTL 准入，账本也不复制一份可漂移正文。 | 中出现一份正文不表示已被 HCTL 准入，控制面存储也不复制一份可漂移正文。 |
| docs/design/spec/system.md:167 | eipt 的权威在用户级 metadata 账本产生并保存；副本由工具箱写入 Git，用于审 | eipt 的权威在用户级控制面存储产生并保存；副本由工具箱写入 Git，用于审 |
| docs/design/spec/system.md:167 | 选，仍须显式恢复流程确认；未结晶的判决和现存账本保持原权威。副本粒度按仓库策略可配：私有仓库 | 选，仍须显式恢复流程确认；未结晶的判决和现存控制面存储保持原权威。副本粒度按仓库策略可配：私有仓库 |
| docs/design/spec/system.md:169 | 变”就自动成为 Git 正文；它们的权威只在账本。反过来，Git 正文的字节权威也不会因为账 | 变”就自动成为 Git 正文；它们的权威只在控制面存储。反过来，Git 正文的字节权威也不会因为账 |
| docs/design/spec/system.md:169 | 本。反过来，Git 正文的字节权威也不会因为账本保存了 digest 就转移到账本。 | 本。反过来，Git 正文的字节权威也不会因为控制面存储保存了 digest 就转移到控制面存储。 |
| docs/design/spec/system.md:169 | 也不会因为账本保存了 digest 就转移到账本。 | 也不会因为控制面存储保存了 digest 就转移到控制面存储。 |
| docs/design/spec/system.md:177 | eceipt ｜ 用户级 metadata 账本 + control；一人多机连同一控制面账 | eceipt ｜ 用户级控制面存储 + control；一人多机连同一控制面账 |
| docs/design/spec/system.md:177 | 本 + control；一人多机连同一控制面账本 ｜ 控制面不可用即系统不可写；客户端只读缓 | 本 + control；一人多机连同一控制面存储 ｜ 控制面不可用即系统不可写；客户端只读缓 |
| docs/design/spec/system.md:178 | ｜ 正文字节在 Git，由工具箱写入/回读；账本保存准入、digest、current/li | ｜ 正文字节在 Git，由工具箱写入/回读；控制面存储保存准入、digest、current/li |
| docs/design/spec/system.md:180 | ；本地只存 Snapshot、身份映射和同步账本 ｜ 看板显示待同步；不依赖当前放置位置、分 | ；本地只存 Snapshot、身份映射和同步记录 ｜ 看板显示待同步；不依赖当前放置位置、分 |
| docs/design/spec/system.md:180 | ision 正文存活于 Git，完成权威留在账本及其可验证审计影子；远端后端由 provid | ision 正文存活于 Git，完成权威留在控制面存储及其可验证审计影子；远端后端由 provid |
| docs/design/spec/system.md:181 | 本地事实继续存在；Run 的完成与评审只依据账本推进，引擎停报进度只让 Run–Engine | 本地事实继续存在；Run 的完成与评审只依据控制面存储推进，引擎停报进度只让 Run–Engine |
| docs/design/spec/system.md:181 | 账 ｜ 进度报告丢失不丢任何判决：Run 按账本继续结束或显式替代；凭证链权威在 metad | 账 ｜ 进度报告丢失不丢任何判决：Run 按控制面存储继续结束或显式替代；凭证链权威在 metad |
| docs/design/spec/system.md:181 | 或显式替代；凭证链权威在 metadata 账本，审计影子在 Git ｜ | 或显式替代；凭证链权威在 控制面存储，审计影子在 Git ｜ |
| docs/design/spec/system.md:183 | 历史、ChangeSet Revision、账本里的 Receipt 与 Verdict 存 | 历史、ChangeSet Revision、控制面存储里的 Receipt 与 Verdict 存 |
| docs/design/spec/system.md:187 | 用户级 metadata 账本「只允许唯一写者」的约束只有三条底线：同时只 | 用户级控制面存储「只允许唯一写者」的约束只有三条底线：同时只 |
| docs/design/spec/system.md:187 | writer 可以搬迁（换机器、上服务器），账本身份不变。不存在 Repo 级或 Proje | writer 可以搬迁（换机器、上服务器），控制面存储身份不变。不存在 Repo 级或 Proje |
| docs/design/spec/system.md:189 | l` 的 OS 锁保证，control 只在账本中以比较并交换推进该现场的 `site_ge | l` 的 OS 锁保证，control 只在控制面存储中以比较并交换推进该现场的 `site_ge |
| docs/design/spec/system.md:195 | SQLite 事务只保证账本内部一致，而事务提交与外部投递不在同一原子域 | SQLite 事务只保证控制面存储内部一致，而事务提交与外部投递不在同一原子域 |
| docs/design/spec/system.md:197 | 是三件事，不互相替代：单写者回答此刻谁有权写账本（本节）；expected-version | 是三件事，不互相替代：单写者回答此刻谁有权写控制面存储（本节）；expected-version |
| docs/design/spec/system.md:207 | iter_generation` ｜ 用户级账本此刻的逻辑写入者 ｜ 本节 ｜ 取得账本写权 | iter_generation` ｜ 用户级控制面存储此刻的逻辑写入者 ｜ 本节 ｜ 取得控制面存储写权 |
| docs/design/spec/system.md:207 | 户级账本此刻的逻辑写入者 ｜ 本节 ｜ 取得账本写权时 CAS 推进 ｜ | 户级控制面存储此刻的逻辑写入者 ｜ 本节 ｜ 取得控制面存储写权时 CAS 推进 ｜ |
| docs/design/spec/system.md:226 | 2. 打开权威账本、验证 schema，恢复 inbox/ou | 2. 打开控制面存储、验证 schema，恢复 inbox/ou |
| docs/design/spec/system.md:237 | 备份必须是由唯一写入者协调的一致备份集：完整账本快照，连同账本引用的精确用户级 Profil | 备份必须是由唯一写入者协调的一致备份集：完整控制面存储快照，连同控制面存储引用的精确用户级 Profil |
| docs/design/spec/system.md:237 | 一写入者协调的一致备份集：完整账本快照，连同账本引用的精确用户级 Profile/Skill | 一写入者协调的一致备份集：完整控制面存储快照，连同控制面存储引用的精确用户级 Profile/Skill |
| docs/design/spec/system.md:237 | ntime 不可变定义字节与摘要。后者存放在账本之外，单备份账本文件会漏掉它们。密钥值、可丢 | ntime 不可变定义字节与摘要。后者存放在控制面存储之外，单备份控制面存储文件会漏掉它们。密钥值、可丢 |
| docs/design/spec/system.md:237 | 变定义字节与摘要。后者存放在账本之外，单备份账本文件会漏掉它们。密钥值、可丢弃缓存、PTY | 变定义字节与摘要。后者存放在控制面存储之外，单备份控制面存储文件会漏掉它们。密钥值、可丢弃缓存、PTY |
| docs/design/spec/system.md:241 | 止且取得用户级排他锁后进行；不得合并两份分叉账本，也不得把备份恢复成新的账本身份。恢复保留原 | 止且取得用户级排他锁后进行；不得合并两份分叉的控制面存储，也不得把备份恢复成新的控制面存储身份。恢复保留原 |
| docs/design/spec/system.md:241 | 不得合并两份分叉账本，也不得把备份恢复成新的账本身份。恢复保留原账本身份，推进 contro | 不得合并两份分叉的控制面存储，也不得把备份恢复成新的控制面存储身份。恢复保留原控制面存储身份，推进 contro |
| docs/design/spec/system.md:241 | ，也不得把备份恢复成新的账本身份。恢复保留原账本身份，推进 control writer、s | ，也不得把备份恢复成新的控制面存储身份。恢复保留原控制面存储身份，推进 control writer、s |
| docs/design/spec/task.md:38 | 工步骤；其不可变正文与位置、摘要在 Git，账本保存稳定身份、准入与 current poi | 工步骤；其不可变正文与位置、摘要在 Git，控制面存储保存稳定身份、准入与 current poi |
| docs/design/spec/task.md:46 | 每个外部规范实体在用户级控制面账本内使用 `(provider, accoun | 每个外部规范实体在用户级控制面存储内使用 `(provider, accoun |
| docs/design/spec/task.md:52 | 1. HCTL-first：账本先固定 Task 身份并提交后端 outbo | 1. HCTL-first：控制面存储先固定 Task 身份并提交后端 outbo |
| docs/design/spec/task.md:55 | 执行并回读，不能把 Git 或后端写入伪装成账本事务的一部分。content-first 路 | 执行并回读，不能把 Git 或后端写入伪装成控制面事务的一部分。content-first 路 |
| docs/design/spec/task.md:95 | 每个 Task 在账本中至多有一个绑定 Run 的占用标记，状态为 | 每个 Task 在控制面存储中至多有一个绑定 Run 的占用标记，状态为 |
| docs/design/spec/task.md:95 | 创建 Run/Manifest 的同一用户级账本事务中，以比较并交换把空标记推进为 `act | 创建 Run/Manifest 的同一用户级控制面存储事务中，以比较并交换把空标记推进为 `act |
| docs/design/spec/task.md:111 | 节以 Git 为 home，control 账本独占身份准入、digest、current | 节以 Git 为 home，控制面存储独占身份准入、digest、current |
| docs/design/spec/task.md:111 | ompletion Receipt 的权威在账本，Git 只有审计影子。完整边界见[系统存储 | ompletion Receipt 的权威在控制面存储，Git 只有审计影子。完整边界见[系统存储 |
| docs/design/references/glossary.md:30 | 共享方法定义；由 Agency 安装并申报，账本只记引用、摘要与可核验性 ｜ [spec/p | 共享方法定义；由 Agency 安装并申报，控制面存储只记引用、摘要与可核验性 ｜ [spec/p |
| docs/design/references/glossary.md:69 | ｜ 身份、绑定、授权与判决，住在 HCTL 账本 ｜ | ｜ 身份、绑定、授权与判决，住在 HCTL 的控制面存储 ｜ |
| docs/design/references/glossary.md:139 | control 账本排他与 Repo 现场的 OS 锁不是 Le | 控制面存储排他与 Repo 现场的 OS 锁不是 Le |
| docs/design/references/glossary.md:141 | 全系统共用六种彼此独立的代次：账本写入者、仓库现场、Agency 绑定归属者、 | 全系统共用六种彼此独立的代次：控制面存储写入者、仓库现场、Agency 绑定归属者、 |

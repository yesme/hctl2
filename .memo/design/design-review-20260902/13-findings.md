# 发现清单（经对抗核验）

> 状态：合成中——A、I、W 三轴已合成；M 轴与调研回填条目待 R1 到后一并核验<br>
> 基线：main @ `6850f18`（草案 v0.16.0）；核验基线 分支 `claude/review-r1` @ `f8dcdf8`<br>
> 来源：[`11-findings-draft.md`](./11-findings-draft.md) × [`12-adversarial-A.md`](./12-adversarial-A.md)、[`12-adversarial-I.md`](./12-adversarial-I.md)、[`12-adversarial-W.md`](./12-adversarial-W.md)<br>
> 去向：「含义」类进 [`20-verdict-packet.md`](./20-verdict-packet.md)；「写法」类第三轮直接落

合成规则：推翻的删（列在文末供对照），修正的按修正后写，维持的保留。核验补出的漏项接续编号：A 轴 L-1…L-8 → A-49…A-56；I 轴核验补的四条 → I-14…I-17（草稿里 I-09…I-13 是调研回填，编号不动）；W 轴补的八条 → W-21…W-28。位置一律「文件 §节名」。

## 核验总账

| 轴 | 草稿 | 维持 | 修正 | 推翻 | 核验补出 | 合成后 |
| --- | --- | --- | --- | --- | --- | --- |
| A 模块与架构 | 45 | 18 | 24 | 3 | 8 | 50 |
| I 具体实现 | 8（另 5 条调研回填待核） | 2 | 6 | 0 | 4 | 12（+5 待核） |
| W 写作风格 | 19 | 6 | 12 | 1 | 8 | 26 |
| M 方法论 | 29（含回填） | 待 R1 | | | | |

核验对我的三个最重要的修正：（一）我在 A-04／A-14 里把所有者 #138 刚裁的一句当成问题重开了，且引文漏了限定词——推翻；（二）我对自己写的文件四处过严（A-23、A-24、A-26、A-27）、零处开脱；（三）I-08 说契约测试「全是失败用例」不成立，约十条是主题标签，正是「只有文字」藏身的地方。

## A · 模块与架构

| 编号 | 位置 | 发现（核验后） | 建议（核验后） | 类别 | 核验 |
| --- | --- | --- | --- | --- | --- |
| A-01 | `vision.md` §设计原则 第 5、6、10、11、12 条 | 五条原则只剩标题加链接；其中第 6、10 条链去的「共同规则」里并没有对应细则，是空链接 | 每条补成完整原则句加一句为什么；补写的论证第三轮让所有者过一眼 | 写法 | 维持 |
| A-02 | `vision.md` §产品原生核心与架构最小内核 表「持久账本与对账」 | 愿景层出现 `outbox`／回读 | 改日常语言：「先记账再执行，崩溃后能对账」 | 写法 | 维持 |
| A-03 | `vision.md` §产品原生核心与架构最小内核 末段 | 一句话并列四类事件处理，读者要数着读；分类是对最小内核的解释，文档纪律保护解释性复述，不删 | 拆两句或四项清单，末尾「精确分类见系统边界」 | 写法 | 修正（判据只留 L5） |
| A-05 | `vision.md` §一句话定位、§为什么需要 HCTL2 | `Harness` 与 `Coding Harness` 两名一物；README 已定「下称 Harness」 | 统一 `Harness`，首现对照一次 | 写法 | 维持 |
| A-06 | `vision.md` 全文 | 快、好、省散在三处（注意力分配、证据高于进度、Context 可解释），没有在设计原则里统一立成一条取舍原则 | 加一条原则：「取舍看三个尺度……冲突时先保好，再保快，最后保省」；顺序由所有者定 | 含义 | 修正（判据改为所有者原则） |
| A-07 | `README.md` §产品姿态、`vision.md` §一句话定位 | 四短语对应 Project、Room、Task、Run，`Participant` 不在姿态里 | 维持四短语；若所有者要加 Participant，两处同改 | 备注 | 修正（降为备注） |
| A-08 | `architecture.md` §三个面 执行面行、§场景与系统 Terminal 行 | 架构层两处点名 `Herdr`，与同文件「实现由交付文档决定」自相矛盾 | 改「本地 Agency 参考实现（运行时选型见交付文档）」 | 写法 | 修正（三处→两处） |
| A-09 | `architecture.md` §模块交接 表 | 四行覆盖八条边里五条，漏三条，其中「完成 Task」是差异化所在 | 加「完成」一行；合入并进「执行回程」 | 写法 | 修正 |
| A-10 | `architecture.md` §三个面 控制面行、`participant.md` §Agency 与执行体 | 「现场执行者」是 §33 给 hctl2-tool 的职责定界语，两处被当第三个名字用；词汇表已有「工具箱 ↔ hctl2-tool」 | 两处改「工具箱」或写「工具箱（现场执行者）」一次；词汇表 hctl2-tool 行补「职责定界见来时路 §33」 | 写法 | 修正（方向反转：收回 2 处，不改 48 处） |
| A-11 | `docs/design/README.md` 模块表 Participant 行两列、§场景客户端与受控端口 末段 | 设计地图三处点产品名（Herdr、Herdr API/TUI、Tuwunel/Vikunja/Dagu/Herdr） | 三处改角色名，选型指交付文档 | 写法 | 修正（补两处） |
| A-12 | `docs/design/README.md` 模块表 Project 行 | 「参与者」一词让读者以为身份归 Project | 改「参与者名册与角色绑定」 | 写法 | 维持 |
| A-13 | `docs/design/README.md` §共同规则 末段 | `CAS`、`outbox` 不在架构层允许的六词内 | 改「比较并交换、发件箱」或「并发与恢复的通用机制」 | 写法 | 维持 |
| A-15 | `project.md` §为什么存在 末段、§模块拥有什么 | 一处仍叫 `participant.md`「横切正文」；一处说 Project 拥有「Participant 与角色」 | 改「模块正文」；改「参与者名册与角色绑定」 | 写法 | 修正 |
| A-16 | `project.md` §Chat Room 场景 | widget／AppService 是 Matrix 规范词，总则放行外部标准原名 | 可选：删 widget/AppService 保留 Matrix | 可选写法 | 修正（不计违例） |
| A-17 | `task.md` §无 Run 的轻量路径、§关键规则 等 | 「第一阶段」限制散在架构层；权威在 `spec/run.md`／`spec/task.md`，不在 delivery.md | 架构层引用指向约束条款；「是否把全部第一阶段限制集中成一张表、放在哪层」进裁决包 | 含义 | 修正 |
| A-18 | `run.md` §Workflow 场景、§关键规则 | 「Dagu 控制台」点产品名 | 改「工作流引擎的原生管理界面」 | 写法 | 维持 |
| A-19 | `run.md` §模块拥有什么 | 「结晶归属以事实为准绳」把目的说清，A5 正面样本 | 维持 | 维持 | 维持 |
| A-20 | 架构层七文件 | 产品名共 23 处：`Herdr` 19、`Dagu` 2、`Vikunja` 1、`Tuwunel` 1，另 `xterm` 1 处待定 | 一次 sweep：产品名只留 delivery、usage、research | 写法 | 修正（数字） |
| A-21 | 架构层 | 约束词只有 4 处（README 63 行随 A-13；`context.md` 43 行「ref + digest」改「精确引用 + 摘要」；`run.md` 34 行已带对照） | 逐处改 | 写法 | 修正 |
| A-22 | `spec/README.md` §核心产品词、路由表 | 总则 23 个核心产品词一律可进愿景层（所有者 2026-09-02 裁定），但愿景第 2 条原则自己分了两档；是否把两档写进路由表属**复议** | 若分档：用户核心词至少含 Repo、Project、Room、Task、Run、Participant、Request、Workflow、Memo、Artifact 与四个场景名；治理词只留 Obligation、Seat、Attempt、Gate、Verdict、Receipt、ChangeSet、Evidence、Context、Skill | 含义（A 档，复议 09-02 裁定） | 修正 |
| A-23 | `participant.md` §Agency 与执行体 | `Herdr` 出现 14 次；接口表角色化后是架构层内容，不必搬 | 接口表留、主语改角色名；两句 Herdr 相关改「运行时选型与已知缺项见交付文档」；`Herdr` 架构层归零 | 写法 | 修正（作者对自己过重） |
| A-24 | `participant.md` §关键规则 | 20 条，约同层其他正文两倍 | 按「读者需不需要记住它」挑骨架到十条左右，保留已过 S1 第二关的人话一行句，其余「精确规则见约束附录」 | 写法 | 修正 |
| A-25 | `participant.md` §七件事分层 | 上三层归账本、下四层由 Agency 供给，一句说清 | 维持 | 维持 | 维持 |
| A-26 | `context.md` §为什么存在 | 「快、省、准」与「快、好、省」是两个对象上的尺子，只差一字易被读成矛盾 | 若 A-06 立为原则，`context.md` 加一句「快省准是快好省在上下文这件事上的具体形式：准就是这里的好」，名字不改 | 含义（随 A-06） | 修正 |
| A-28 | `spec/connections.md` §连接模型 首句 | 以全库仅此一见的名字 `Handoff` 做排除，读者要猜 | 删名字：「连接不是一份可独立漂移的共享状态，也不是独立的持久对象」 | 写法 | 修正 |
| A-31 | `spec/README.md` §词汇分类法 | 承诺的「语义名 ↔ 标识符」对照表只有技术词那张，字段级没有 | 补字段名对照表（可机械生成）或改掉承诺 | 含义 | 维持 |
| A-32 | `spec/README.md` §六族规则、§三类数据 | 族语义与三条法各定义一次，正面样本 | 维持 | 维持 | 维持 |
| A-33 | `spec/system.md` 11 处、`spec/participant.md` 10 处 | 约束层把 `Herdr` 当约束主语 | 主语改角色（Agency 适配代码、Agency 端点、Agency 运行时）；对齐表保留一行；「本地参考实现只在所选运行时外加技能目录、可用性申报与适配器」一句留 | 写法 | 修正（判据去 T6） |
| A-34 | `spec/system.md` §安全边界 第 2 条 | Tauri 2／Electron 配置项写进约束 | 约束只留三句；配置项搬 delivery.md 或研究条目 | 写法 | 维持 |
| A-35 | `spec/system.md` §单写者、§控制面自己的存储 | 两处把约束与实现当场分开说，范式 | 维持 | 维持 | 维持 |
| A-36 | `spec/run.md`、`spec/task.md` 多处 | `Dagu`、`Vikunja` 当约束主语 | 主语改「workflow engine」「引擎适配器」「任务后端 webhook」；Profile 三组规则留在 `spec/run.md`，只把「Dagu `human.task`」改「引擎的被动检查点原语」、「Dagu YAML」改「引擎定义」；产品名只留对齐表 | 写法 | 修正（不搬 Profile） |
| A-37 | `spec/run.md` Run 状态表、Attempt 状态表 | 两表无超时行；多数超时已定义在别处（Obligation 截止、Request 策略、副作用确认回执） | 加「超时」列或表下总纲指向来源（写法）；「Run 过渡态要不要墙钟超时」单列进裁决包 | 写法 + 含义（一条） | 修正 |
| A-38 | `spec/run.md`、`spec/task.md` 枚举值 | 生命周期状态用中文、其余枚举用英文标识符，两口径并存；「quorum-unreachable」「unsupported」是裸英文 | 裸英文两处改代码体或中文；代码体标识符登记进 A-31 对照表；Gate 票值、写入权模式是否改中文随 A-31 裁 | 写法 + 含义（随 A-31） | 修正 |
| A-39 | `spec/task.md` §启动 Run 的前置与排序令牌 第三段 | 一段里英文名词十来个 | 重写成中文（草稿给了改后句） | 写法 | 维持 |
| A-40 | `spec/task.md` §契约与来源 契约惰性 | M-02 的关键证据，两层一致 | 维持 | 维持 | 维持 |
| A-41 | `spec/project.md` §Context `small-brain` 四处 | 随 W-10 | 随 W-10 | 随 W-10 | 维持 |
| A-42 | `spec/project.md`、`spec/participant.md`、`spec/task.md` 对象表 | 票据、族成员与领域对象同列无标注，只有 `spec/run.md` 行内标了族 | 其余三表照 `spec/run.md` 行内括注类别或族 | 写法 | 修正 |
| A-43 | `delivery.md` §选型判据 首句 | 「实现名只出现在本文与实现证据」——规则在，执行没跟上 | 维持规则；A-08/11/20/23/33/36 按它清零；第三轮做成检查 | 维持 | 维持 |
| A-44 | `delivery.md` §明确不做 | 是「第一阶段不做」的集中处，但非规范复述，架构层引用不能只指这里 | 随 A-17；若做汇总表，每行指回约束条款 | 含义（随 A-17） | 修正 |
| A-46 | `doc-discipline.md` 第四条 | 首现对照两类词已裁并写明 | 维持；作为机械检查白名单 | 维持 | 维持 |
| A-47 | 来时路 | 驼峰与历史名允许 | 维持 | 维持 | 维持 |
| A-48 | better-harness 证据状态词表（R2） | known/unknown 两态 vs Observed/Missing/Unobserved/Not applicable 四态 | 评估把「不适用」从 unknown 分出 | 含义 | **待核**（调研回填） |
| A-49 | `participant.md` §Terminal 场景 恢复等级表；`spec/participant.md` §终端通道 首句 | 五个恢复等级的定义在架构层，约束层与 CT 反向引用——权威倒置；五个名字全英无对照 | 定义表搬到 `spec/participant.md`；架构层留一句加链接；词汇表登记五级中文对照 | 写法（搬家） | 核验补（L-1） |
| A-50 | 全库 12 处 `Trigger Preview` | 具名步骤无定义无词条；其实是 `Preview(command draft)` 用在调用命令上 | 词汇表登记「Trigger Preview｜调用预览；对『发起调用』命令做的 Preview，不是新命令」 | 写法 | 核验补（L-2） |
| A-51 | `vision.md` `control` 3 处、`actor` 2 处 | 与 A-02 同类越层词 | `control` 改「控制面」；`actor` 改「操作者」或「谁在做」 | 写法 | 核验补（L-3） |
| A-52 | `glossary.md` §核心产品词 表 vs `spec/README.md` | 词汇表 28 行比总则 23 个多了 Agent、Harness、Agency、worker、Chat Room；Chat Room 是四场景名里唯一不在核心词内的 | 词汇表拆出「系统角色名与泛称」小表（写法）；Chat Room 是否进核心产品词进裁决包 | 写法 + 含义（一条） | 核验补（L-4） |
| A-53 | `architecture.md` §模块交接 首句 | 「每次交接都交付冻结对象」被自家总表的事件回流边证伪 | 改「每次改变事实的交接都交付……；事件回流只是投影」，随 A-09 落 | 写法 | 核验补（L-5） |
| A-54 | `context.md` 4 处、`spec/project.md` 1 处、`spec/task.md` 1 处 | 「组装器」被当动作主体，组件表与词汇表都没登记 | 首现处写「组装器（control 内负责物化 Context Bundle 的部分，不是独立组件）」并进词汇表 | 写法 | 核验补（L-6） |
| A-55 | `spec/run.md` §外部概念对齐 「引擎外部检查点」行 | 与正文「Engine 检查点」一物两名 | 改「Engine 检查点」 | 写法 | 核验补（L-7） |
| A-56 | 全库 `control` 与 `hctl2-control` | 词汇表没有 control／hctl2-control／控制面 的一对一映射 | 词汇表加一行 | 写法 | 核验补（L-8） |

已推翻（不进后续）：A-04、A-14（现行原句已带「它的执行面」限定，且是所有者 #138 裁决原话；我未声明地重开了）；A-27（推理服务缓存是给排序补第二个理由的外部事实，路由表允许论证在①②层）。

## I · 具体实现

| 编号 | 位置 | 发现（核验后） | 建议（核验后） | 类别 | 核验 |
| --- | --- | --- | --- | --- | --- |
| I-01 | `spec/project.md` §根 Context Manifest 三句（萃取本地、相关性门只用账本事实、small-brain 须钉版本） | 能机械的先机械，模型判定可选且记账；CT 有对应用例 | 维持；引用时引约束层三句，不引架构层「必须快」 | 维持 | 维持 |
| I-02 | `spec/connections.md` §失败与恢复 表 | 十二个失败点每个都有机械结果；两行的取舍由绑定冻结的降级策略决定（定义在系统边界）；一行「等待物理执行静默」无超时 | 维持；无超时一行随 A-37 裁 | 维持 | 修正（数字与两处说明） |
| I-03 | `spec/*.md` 225 句规范句 | 治理规则没有一条依赖模型听话（Harness/模型/执行体主语的句子全由三条底线与 actor 来源赋值强制）。112 句无机制词里：2 句 small-brain 产物回源只有存在性检查，指针正确性留白；4 句不是系统规则（作者纪律、运维义务、用户选择、脚本误报）；2 句机制成立但措辞越过 control 能做的事（「关闭原生控制器」应为「不得声明 `managed_single_writer`」） | 约束层补一句「压缩与纪要的回源指针由组装器按来源分块赋予并随片段冻结，不由模型输出」；4 句移出规范句或改写；2 句改措辞；每节首行给一个 CT 族引用 | 含义（补一句）+ 写法 | 修正 |
| I-04 | 写作规则与 `src/build/docs/` 现有五项检查 | 路由表越层词、产品名越层、驼峰名、首现对照（白名单为文档纪律十个词）四项能机械化；「同层同表中英夹杂」只能窄版报告制；「需要」当规范词的卡定为「spec 内只许出现在『需要关注』」，先改掉 15 句描述句 | 每项一个 `sh_test` 进对应 profile；三个 profile 第一次真正分开 | 含义（B 档：强制方式） | 修正 |
| I-05 | `spec/system.md` 表 D 十三项通用机制 | I1 维持。I5 由部件矩阵回答：outbox、租约、代次是有理由的借鉴想法；JCS 摘要、现场锁、备份快照、密钥、全文索引五处应改 SDK 级 | 五处 SDK 选定写进交付文档技术基线；「同一个 SQLite 事务」点了产品名归 A 轴 | 含义（B 档） | 修正 |
| I-06 | `spec/run.md` §写入约束 完成谓词 | 四条账本谓词，模型自述与引擎进度都不能补足 | 维持 | 维持 | 维持 |
| I-07 | 五种复用决策用语、`delivery.md` §选型判据 | SDK 不是缺失而是并入了「采用为依赖」；缺的是把二进制与 SDK 分开、以及偏好顺序；适配协议与暂缓不在四级尺子上 | 采用为依赖拆成「采用二进制」「采用 SDK」；移植组件 ≈ 复制代码；仅参考行为 ≈ 借鉴想法；README 与选型判据写一句偏好顺序 | 含义（B 档） | 修正 |
| I-08 | `contract-tests.md` | 绝大多数是「X 时拒绝」形态；约十条是主题标签（「命令幂等」「schema migration、投影重建」等），CT-PRODUCT 三条是产品验收判据不是契约测试 | 主题标签展开成失败用例或移出；CT-PRODUCT 另立「产品验收」节并写明由人判 | 写法 | 修正 |
| I-09 | `src/build/tools/reindeer`（R3） | Reindeer 从源码编译，上游有官方二进制 | 换 DotSlash 官方二进制 | 含义（B 档） | **待核** |
| I-10 | `hctl2-services` 与 `runtime.sh` 系列（R3） | 675 行 shell 自写进程监督器；开发侧已用 Process Compose | 候选：control 经 Process Compose 托管，P2 前限时验证；R3 建议所有者拍板 | 含义（B 档，请务必看） | **待核** |
| I-11 | `hctl2-control` 六个 provider 客户端（R3） | 不该手写 HTTP 客户端；各 provider 有 schema 或类型库 | 技术基线写明「从 schema 生成或用类型库」 | 含义（B 档） | **待核** |
| I-12 | hctl2-control 十八条、hctl2-tool 五项（R3） | 自研非胶水的五条正好落在愿景「最小内核」五行上 | 维持 | 维持 | **待核** |
| I-13 | 本地 Agency 参考实现（R2） | 第十二族的 harness 侧钩子（PreToolUse deny、Stop 钩子核对终局清单）是账本门之外的另一层机械门 | 参考实现可带一份 harness 钩子作可声明加固；`ControlEvent v1` 适配协议候选、`gate-check.mjs` 移植候选 | 含义（B 档） | **待核** |
| I-14 | `CONSTRAINTS.md`、写作指南 L7 词汇黑名单 | 「张力」「合同」靠散文管，`dead_names.txt` 里没有 | 加两行词表、一行豁免 | 写法（加卡） | 核验补 |
| I-15 | 写作指南 T1「应当／不应 0 处」、L1 零修饰词表 | 拿实测数字当依据，没有卡会漂；今天 grep 均为 0 | 两个 `sh_test`：spec 内应当/不应为零；L1 词表为零 | 写法（加卡） | 核验补 |
| I-16 | `report_prohibition_density.sh` | 报告制，「禁令净减」没有基线 | 加基线文件，卡定为「不高于基线」 | 写法（加卡） | 核验补 |
| I-17 | `pr-contract.yml` 自建信号 | 只看新增脚本，改已有脚本不算；675 行监督器就是这样长出来的 | 触发条件加「已有脚本净增行数超过阈值」 | 含义（B 档：门的规则） | 核验补 |

## W · 写作风格

| 编号 | 位置 | 发现（核验后） | 建议（核验后） | 类别 | 核验 |
| --- | --- | --- | --- | --- | --- |
| W-01 | `vision.md` §产品原生核心 | 首现对照其实合规；括注里冒号后半句复述加粗短语（L5 填充） | 删复述半句 | 写法 | 修正 |
| W-02 | `README.md` §目标架构 第二段 | 一句三件事；Herdr 在首页不算越层（架构层与词汇表已用） | 只拆句，不删名 | 写法 | 修正 |
| W-03 | `architecture.md` §4×3 矩阵 表头 | 表头三列写法一致；真问题是括注内容「控制面账本」与首现对照形态相同、内容不同 | 表头去括注或用间隔号；§避免供应商锁定 两处中文裸用统一 | 写法 | 修正 |
| W-04 | 全库 | 活驼峰只有 `ChangeSet`（67 活文）与 `ReviewSubjectRef`（12）；其余全是外部原名。保留 ChangeSet 的理由是「合成才成一个概念、核心产品词」，不是 Git 通用说法 | `ChangeSet` 保留（或改 `Changeset` 求零例外，所有者拍板）；`ReviewSubjectRef` 散文写「评审对象引用」、字段位 `review_subject_ref`，同批改总则表例、词汇索引、`dead_names.txt` 注释 | 含义（B 档） | 修正 |
| W-05 | 写作指南 L4、文档纪律第四条 | 两条不冲突，所有者已裁；清点口径限十词 | 维持规则 | 维持 | 维持 |
| W-06 | 约束层 `CAS` 8 处 | 4 处机制名合规；`system.md` §单写者、§代次家族表、§备份与恢复 三处与 `connections.md` §失败与恢复 一处是动词用法 | 四处动词用法改「比较并交换」 | 写法 | 修正 |
| W-07 | `task.md` 两处、`participant.md` 一处 `adapter` | 架构层 adapter 4 处、适配器 15 处并用 | 架构层统一「适配器」 | 写法 | 维持（数字修） |
| W-08 | 架构层与愿景层 | 总则明文「写 human actor」，用它不违规，只缺词条；真正的同层两写是裸用「human」（README、project、task、vision）与「有权的人」并存 | 词汇表补「human actor｜有权的人」；裸用 human 清零：统一 human actor（从总则）或改「有权的人」（从 L4），所有者二选一 | 含义（B 档，对象换成 human 裸用） | 修正 |
| W-10 | `context.md`、`spec/project.md`、CT `small-brain` | 连字符自造复合词；同文件三个名字（专用小模型／专用压缩模型／small-brain）指一物；文中自述「不是新对象」 | 全文「专用小模型」，首现括注一次 | 写法 | 维持（判据 W3） |
| W-12 | `spec/connections.md` 总表 | human actor／human provenance 是总则原文；真夹杂只有「human scene」「human Kanban」；方向列其余全英文，改中文会撞 W2 | 方向列写「human actor / Run reducer → Participant」「… → Task」，删 scene/Kanban；「task-bound」→「绑定 Task 的」；reducer 随 W-23 | 写法 | 修正 |
| W-13 | `connections.md` §失败与恢复 表「CAS 拒绝」 | 与正文两写 | 改「比较并交换拒绝」 | 写法 | 维持 |
| W-14 | `system.md` 三处「CAS 推进」 | `expected-version CAS` 是机制名保留；三处动词改 | 三处改「比较并交换推进」 | 写法 | 修正 |
| W-15 | 全库 `ground truth` 9 处 | 3 处带括注、6 处裸用；架构层首现合规，约束层首现后裸用 | 约束层 5 处裸用改「事实源头」，或整层选定英文，按 0.9 扫 | 写法 | 修正（方向反转） |
| W-16 | 「Chat 端口绑定」活文 12 处 | 总则登记的形态就是它，L4 对已登记对象名让路；真病是它在 Binding 族（全英）与端口名（全中）两族里都是孤例；总则词汇索引 Binding 族不含它而词汇表含；字段组还是对象未定 | 改「聊天端口绑定」（词汇表已给）；若 A 轴裁它是 Binding 族对象则「Chat Port Binding（聊天端口绑定）」；字段组／对象之辨归 A 轴 | 写法 + 含义（一条） | 修正 |
| W-17 | `contract-tests.md` | 116 条里 108 条含拉丁字母，但大头合法；真夹杂约 15 词落在约 25 条（四分之一）：fail closed、adoption、divergence、credential、readback、lane、terminal intent、reviewer Seat、recipient、stage or health、human action、mutation、executor。drift、placement、provenance、regate 是约束层自己在用的词，不能单改交付层 | （丙）类改中文；（甲）外部原名加代码体；（乙）随约束层裁 | 写法 | 修正 |
| W-18 | 来时路 §36「四个对仗 Scene」 | §36 是现行结论不是史料 | 改「场景」 | 写法 | 维持 |
| W-19 | `docs/usage.md` 六处 Chatroom | 一处指场景该写 Chat Room；五处指 Tuwunel+Cinny 服务组合，改成场景名会混淆场景与系统 | `:3` 改 Chat Room；五处改「聊天服务（Tuwunel + Cinny）」 | 写法 | 修正 |
| W-20 | 写作指南 L3、L4 | 两条新规则没有明文；驼峰一条已有现成扫描可升为检查 | L3 加「不造驼峰拼接名」、L4 加「同层同表同一写法」；与 I-04 合并升检查 | 含义（B 档） | 维持 |
| W-21 | 愿景层与架构层九文件 | 十个自然译名词的首现对照几乎无一文件合规（architecture 0/5、context 0/7、participant 1/6……） | 一次 sweep；清点脚本限十词后直接产出这张表 | 写法 | 核验补 |
| W-22 | 总则「执行规格」vs 词汇表与 `architecture.md`「派发规格」 | 六个高频约束词之一的中文对照两写 | 以总则「执行规格」为准，改词汇表与 architecture 两处 | 写法 | 核验补 |
| W-23 | 约束层「Run reducer」与「归约器」 | 同层两写，`run.md` 同文件两写；架构层统一「归约器」 | 约束层统一「Run 归约器」，与 W-12 联动 | 写法 | 核验补 |
| W-24 | `participant.md` 九处、spec 与 CT `known/unknown` | 状态值用英文，违反总则「状态值用中文语义名」；A-38 只点了 spec/run | 「已核验／仅申报」 | 写法（接口细节可直接落） | 核验补 |
| W-25 | `context.md` §与相邻概念的分工 表 | 同列「Memo」（英）与「根上下文清单／消费上下文包」（中）并列——路由表六词白名单与 W2 的规则冲突 | 所有者裁：把 Context Manifest／Bundle 加进高频约束词，或接受表内对照混排 | 含义（B 档） | 核验补 |
| W-26 | `vision.md` 三处 | 「client-server」×2（architecture 已写中文）、「human 动作」、产品姿态英文加粗为主与首页主次相反 | 改中文；姿态改首页形态 | 写法 | 核验补 |
| W-27 | 词汇表 ChangeSet、Evidence 行 | 链接文本残留「spec/agent」 | 改「spec/participant」 | 写法 | 核验补 |
| W-28 | 「Share to Room」3 处 | 不合总则「命令用动宾语义名」 | 改「分享到 Room」 | 写法（低优先） | 核验补 |

已推翻：W-09（`participant.tui` 已在词汇表登记为标识符，L3 满足；降为代码体格式统一，不进裁决包）。

## M · 方法论

（待 R1 边界抽取到后，连同调研回填条目 M-28、M-29、A-48、I-09…I-13 一起做对抗核验，再合成到此处。）

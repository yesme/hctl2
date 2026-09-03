# I 轴对抗核验：具体实现

> 状态：已核验<br>
> 基线：分支 claude/review-r1 @ f8dcdf8<br>
> 去向：合成 13-findings.md

核验方法：默认怀疑，每条先找反例，找不到才写「维持」。判据只用[四轴原则](../../notes/review-four-axes-20260902.md)第三条与 [02-checklists.md](./02-checklists.md) 的 I1–I5。读过的原文：`docs/design/spec/` 七个文件全文、`delivery.md`、`contract-tests.md`、`context.md`、`participant.md`、`docs/research/README.md`、今天新落的 `docs/research/component-matrix-20260902.md`、`WRITING-GUIDE.md`、`AGENTS.md`、`.github/workflows/pr-contract.yml`、`src/build/docs/` 全部脚本与 BUCK。机械清点用 [10-inventory.md](./10-inventory.md) 的强制手段两节：225 句规范句、113 句含机制词、112 句不含——按文件加总核对无误（4+26+47+41+27+46+34；3+10+29+23+14+22+11）。

## 逐条结论

| 编号 | 结论 | 理由与原文引用 | 修正后的表述（若修正） |
| --- | --- | --- | --- |
| I-01 | 维持 | 引用的是架构层 `context.md` §萃取与压缩：「这一步必须快，缺省全本地、不花模型 token：先按显式引用与讨论窗口命中，再由本地检索兜住隐式关联，只有用户配了专用小模型时才引入模型判定」。「必须快」本身不可测，但 I1 要问的强制点在约束层，而且齐全：`spec/project.md` §根 Context Manifest「萃取与相关性判定默认全部在本地完成，不消耗大模型 token……相关性门默认只以账本事实——提及、认领、Request 关联和游标——作为判定输入」「用户配置专用的小模型（small-brain）后，相关性门才可以读取消息正文并使用模型辅助判定；该模型必须引用用户级定义机制中的精确 revision 和 digest」；`contract-tests.md` CT-PROJECT 有失败用例「相关性门判定缺可审计记录时无效，未配置 small-brain 时以消息正文做路由拒绝」。开关是配置存在性，判定留审计，模型是可选项。我试图推翻的点是：small-brain 一旦启用，它决定什么进 Context、结论对不对没人校验——但这正是第三轴允许的形态（能机械的先机械，模型可选且记账），不构成反例 | —（建议引用时改引约束层三句，不引架构层的「必须快」） |
| I-02 | 修正 | `spec/connections.md` §失败与恢复 表实有 **12** 行，不是 11：来源已变化、目标已提交未收到、身份可证但外部未知、身份不可证、取消或被替代、chat server 不可用、房间被加密、任务后端不可用、workflow engine 不可用、harness/Agency 不可用、其他适配器不可用、场景投影丢失。每行都给机械结果，这点成立。但两行把结果写成几选一——「harness / Agency 不可用｜执行安全暂停或按代次结束」「其他外部适配器不可用｜连接显示待启动/需要关注或安全暂停」——选哪个由 `spec/system.md` §端点与输入的信任边界「control 必须按该绑定冻结的降级策略暂停或终结活动执行」决定，机制在别处；一行「归属者取消或被替代｜……等待物理执行静默」没有超时，与 A-37 同病。试图推翻的点：「需要关注」是不是叮嘱？不是——它把决定交给人，不是让模型自觉；反例不成立 | 「十二个失败点每个都有机械结果，没有一处靠人或模型『注意』。其中两行在『暂停/结束』『待启动/需要关注/安全暂停』之间的取舍由绑定冻结的降级策略决定（定义在系统边界，本表只引），一行『等待物理执行静默』无超时（随 A-37 裁）。」 |
| I-03 | 修正 | 结论「约束层没有一条规则的成立依赖模型听话」对**治理规则**（谁能写账本、什么算结果、什么算完成）成立：我逐句过了 112 句，Harness／模型／执行体做主语的句子全部落在三条底线加入口校验上，见下节甲组。但「几乎全是由 control 代码强制」漏了两类：乙组 2 句是对 small-brain 产物的保真要求，约束层只写了存在性检查（「压缩条目缺少来源记录……必须拒绝交付」），指针指得对不对取决于压缩流水线怎么造，约束层没写——这是 112 句里唯一「机制留白、可能落到模型自觉」的地方；丙组 4 句根本不是对系统立的规则：1 句作者纪律、1 句运维义务、1 句对启动 Run 的人的劝告、1 句脚本误报。另有丁组 2 句机制成立但措辞越过了 control 能做的事。逐句见下节 | 「约束层的治理规则没有一条依赖模型听话：Harness、模型、执行体做主语的规范句全部由三条底线（不给凭据、只有提案通道、隔离工作树）与命令入口的 actor 来源赋值强制。112 句无机制词的句子里，2 句关于 small-brain 产物回源（压缩片段、滚动纪要）只有『有没有来源记录』的存在性检查兜底，指针正确性要由约束层补一句『回源指针由流水线按来源分块赋予，不由模型输出』才算机制；4 句不是系统规则（作者纪律、运维义务、用户选择、脚本误报），应移出规范句或改写。可选的写法改进不必逐句加『由 X 的契约测试判定』——CT 十族已按模块分家，每节首行给一个 CT 族引用即可。」 |
| I-04 | 修正 | 五项「全部可以机械检查」里，四项能、一项只能报告，而且「清点脚本已把前三项做出来了」说过了：`inventory_concepts.pl` 做的是概念×层计数、**按层**（不是按表）的中英两写、首现对照、驼峰词——没有实现路由表词汇的越层检查（计数不是准入），中英两写的粒度恰好是 W2 说「不是规则」的那个粒度（`Context` arch 11/27 跨四个文件）。逐项可行性见下节。另外漏了一项脚本已做、纪律已定的检查：首现对照（清点 §三，白名单在 `doc-discipline.md` 第四条的十个词，A-46 已指出它是白名单来源） | 「路由表越层词与产品名越层：能，做法同 `check_dead_names`（词表 + 按文件/按节豁免，spec 的外部概念对齐表整节豁免）。驼峰名：能，现存正文命中只有 `ChangeSet`、`ReviewSubjectRef` 与产品/协议名，豁免表十来行。首现对照：能，脚本已有，白名单是文档纪律第四条的十个词。同层同表中英夹杂：只能做窄版——按表检查词汇表配对且排除中文名是常用词的概念（`Run`/运行、`Task`/任务、`Request`/请求、`Context`/上下文），否则 Run 状态表里的『运行中』就会误报；W-12 那种『human scene』式短语不是配对问题，要另一条『表格单元格内非词汇表、非代码体的多词拉丁短语』启发式，两条都先报告制，一轮豁免后再升卡。『需要』当规范词：spec 现有 32 处，17 处是状态名『需要关注』，其余 15 处全是描述句（『需要输入时』『需要扩权时』），grep 分不出角色；可行的卡是『spec 内需要只允许出现在需要关注中』——先改掉那 15 句。每项一个 `sh_test`；顺带一提，现在 profile-design/spec/delivery 三个套件跑的是同一组四个测试，按层的检查会是它们第一次真正分开。」 |
| I-05 | 修正 | I1 部分维持：`spec/system.md` §代次家族 六行各有「权威定义」「何时产生或推进」两列，四条推导禁令是给实现者的设计规则，运行期由 `spec/run.md`「三组代次必须分别校验」与 CT-PARTICIPANT「旧 `runtime_generation` 的输入和结果必须拒绝」兜住。I5 部分不必再「待 R3」——今天落盘的 `docs/research/component-matrix-20260902.md` 表 D 已答：outbox 的三个 SQLite 库（effectum、apalis-sqlite、honker）「都不在『同一本账本的同一事务』里」，维持自研是有理由的借鉴想法；租约与代次的来源是 Kleppmann 的 fencing token，分布式锁服务是超集；JCS 摘要「由『可能手写』改 SDK：`serde_jcs` 或 `serde_json_canonicalizer`，CT 钉 RFC 8785 向量」；现场锁用 `fd-lock`、备份快照用 SQLite Online Backup API、密钥用 `keyring`、全文索引用 FTS5。矩阵总账：「hctl2-control 十八条里九条是胶水、两条是薄自研」 | 「维持 I1 判断。I5 已由部件矩阵表 D 回答：outbox、租约、代次是借鉴想法且有理由（同事务边界无库可借、锁服务是超集）；JCS 摘要、现场锁、备份快照、密钥存储、全文索引五处不该手写，应改成 SDK 级——这五处是约束层机制里唯一的『借用等级可以再上一级』的地方。实现层脚注：`spec/system.md`『control 必须在同一个 SQLite 事务中写入』在约束句里点了产品名 SQLite，与 A-33 同一模式，归 A 轴处理。」 |
| I-06 | 维持 | `spec/run.md` §写入约束：「Run 进入完成前，control 必须逐项证明：1. 所有必需 Obligation、Seat、Gate 和输出已达成；2. 所有 Attempt 已终态或已撤权；3. 没有影响必需输出的未决副作用；4. Manifest、Engine Execution Binding 和结果引用仍匹配。任何一项未知都不得完成 Run。Engine 检查点、进程退出、Harness 或模型自述和单个 Result Proposal 都不能补足上述谓词」。四条都是账本可核对的事实（状态、终态、outbox 计数、摘要相等）。我试图推翻的点：第 1 条的「已达成」建立在 Verdict 上，而 Verdict 正文是模型评审的判断——但谓词核对的是「票已记、已计数」这个账本事实，不是票的质量；谁强制仍是 control。CT-RUN 已有「Run 正常完成只由账本谓词决定；引擎报告的进度与账本不一致时标为分歧待对账，既不补足也不阻止谓词」。反例不成立 | — |
| I-07 | 修正 | `docs/research/README.md` §复用决策用语 原文：五种是「采用为依赖（Adopt dependency）、移植有边界的组件、适配协议、仅参考行为、暂缓」，问法对应「直接用它的 CLI/服务＝采用为依赖；借它的 schema/协议形状＝适配协议；抄它的代码＝移植有边界的组件；借它的思想/阶段/交互＝仅参考行为」。说「没有 SDK 一级」不准确：同一文件 §⑥ 写「Dagu、Tuwunel、Vikunja、Herdr、Cinny、Tauri 2 与 UI 通用库采用为依赖」——UI 通用库就是 SDK，已经被归进「采用为依赖」；§标准与通用库 也列了「Agent Client Protocol / Rust SDK」。所以 SDK 不是缺失，是与二进制**并成了一级**，抹掉了所有者四级尺子里前两级的区别（自己写多少代码）。主评审员建议的对照「适配协议与 SDK 分别对应协议形状与开发库」也不对：适配协议是借形状不借实现（Linear/GitHub、Termio/ATP 都是适配协议却没有拿它们的 SDK），不在四级尺子上。「没有写出偏好顺序」成立：README 与 `delivery.md` §选型判据 的四条判据都不是借用顺序。部件矩阵文件头已经按四级尺子写（「跨平台二进制 > 拿 SDK 自己开发 > 直接复制代码 > 借鉴想法，四级之外才是自研」，总账「SDK 级十二行」），是用语落后于实践 | 「五种复用决策里 SDK 不是缺失而是并入了『采用为依赖』（README §⑥ 已把 UI 通用库归为采用为依赖）；缺的是把二进制与 SDK 分开、以及偏好顺序。对照：采用为依赖 → 拆成『采用二进制』『采用 SDK』两级；移植有边界的组件 ≈ 直接复制代码；仅参考行为 ≈ 借鉴想法；适配协议与暂缓不在四级尺子上（前者借形状不借实现，后者是不借）。部件矩阵已按四级尺子写，README 的用语与 `delivery.md` §选型判据 应跟上，并把偏好顺序写成一句。仍进裁决包 B 档。」 |
| I-08 | 修正 | `contract-tests.md` 文件头自称「本文列出十族可观察行为的失败用例」，十族数对。但「全部是失败用例，每条都能写成机械测试」不成立：约十条是主题标签而不是失败用例——CT-SYSTEM「命令幂等」「schema migration、投影重建」「commit/确认回执各崩溃点回读」「打包后的整窗启动/退出/升级和安全边界」，CT-PARTICIPANT「冲突观测按来源证据仲裁」「attach/replay、IME/背压/慢客户端隔离」，CT-TASK「Project 分组映射（父任务/milestone/标签降级）有测试」（这句是『有测试』，不是测试）；CT-PRODUCT 三条是产品验收判据，无法机械判定——「用户十秒内能回答 Project 目标、Task 状态……」「正常成功保持安静」「HCTL2 仓库自举不使用隐藏的特例豁免或产品外补签事实」（后者只能靠代码评审）。其余确是「X 时拒绝／不得…」形态，正面样本成立。这一条对第三轴要紧：验证层自称全是失败用例，主题标签正是「只有文字」藏身的地方 | 「十族里绝大多数条目是『X 时拒绝／不得 Y』形态、能写成机械测试；约十条是主题标签（『命令幂等』『schema migration、投影重建』『attach/replay、IME/背压/慢客户端隔离』『Project 分组映射有测试』等），要展开成失败用例或移出；CT-PRODUCT 三条是产品验收判据，不是契约测试，应另立『产品验收』一节并写明由人判。写法类，含义不变。」 |

## I-03 逐句清单：112 句里不由 control 代码强制的句子

甲组先说清为什么主评审员的核心判断站得住。主语是 Harness、模型、执行体、Skill、Participant 的规范句，强制手段都能指到具体机制：

| 句子（出处） | 强制手段 |
| --- | --- |
| 「Harness、运行时钩子和模型只能提交 Result Proposal，不能提交治理命令」（`spec/participant.md` §不可关闭的三条底线） | 底线二「HCTL 不向 Harness 交付 control 客户端凭据……」；`spec/system.md` §命令与跨服务正确性「actor 来源只能由直接客户端连接、绑定中的账号映射或 control 内部归约器赋予」——来源是 control 赋的，不是提交者报的；CT-PARTICIPANT「Result Proposal 通道提交不了治理命令」 |
| 「Harness、模型和执行主体只能提交 Result Proposal，不能自报为 human」（`spec/system.md` §命令与跨服务正确性） | 同上；CT-CONNECTION「actor provenance 不能由 payload 自报」 |
| 「模型 Participant 的 Message、Result Proposal、总结及其正文中的 `@` 只能形成下一位 Participant/Role 与扇出建议，不能自行创建 Room Invocation、唤醒执行体或递归委派」（`spec/project.md` §场景约束） | `spec/project.md` 同节「chat server 里的普通消息本身不是入口」；CT-PROJECT「模型 Participant 的 `@`/建议不能创建 Invocation 或 fan-out」 |
| 「不得把 unknown 记为 known」（`spec/participant.md` §Skill 与申报） | 同段「工具箱能回读到同一 digest 的记 known，只有 Agency 申报的记 unknown」——由工具箱回读决定；CT-PARTICIPANT 有对应用例 |
| 「dynamic fork 的候选 Participant/Role、最大基数、预算、选择函数和权限上限都必须预先固定」（`spec/run.md` §启动与 Manifest） | 同段「模型输出不能新增接收者、扩大扇出或扩权，无法机械校验时整次 fork 必须拒绝」——自带机制词以外的机制 |
| 「受信任的 `in_process` Proposal 使用缩减头，而且不得提交 ChangeSet」「修正必须创建新 Proposal 和新的生产者序号，不得改写原项」（`spec/participant.md` §运行时与观测） | `spec/connections.md` §Participant → Project / Run「control inbox 先按提案标识符、producer sequence 和归属者去重。随后逐项校验……」 |

乙组：机制留白，可能落到 small-brain 自觉。两句都关于派生缓存，不进权威账本，影响面小，但正是第三轴要挑的形态。

| 句子（出处） | 现有兜底 | 留白 |
| --- | --- | --- |
| 「压缩产物的每个片段都必须能回到原文位置」（`spec/project.md` §根 Context Manifest） | 同段「压缩条目缺少来源记录，或压缩了证据类内容时，Bundle 必须拒绝交付」；CT-PROJECT「Bundle 压缩条目缺 compressor/原文 digest 记录……时拒绝交付」 | 检查的是「有没有来源记录」，不是「指得对不对」。指针若由 small-brain 在输出里给，正确性就靠模型；若由流水线按来源分块赋予，就是机械的。约束层没说是哪种 |
| 「纪要逐条携带消息事件回源指针」（`spec/project.md` §根 Context Manifest；`context.md` §前情提要「每条带消息事件回源指针……纪要是组装器机械触发、small-brain 计算的系统派生产品」） | 后半句「治理引用不得指向纪要，只能指向精确事件」有 CT-PROJECT「治理引用指向滚动纪要而非精确消息事件时拒绝」 | 同上：「逐条携带」的指针由谁赋、怎么验，约束层没写 |

补一句即可闭合：「压缩与纪要的回源指针由组装器按来源分块赋予并随片段冻结，不由模型输出。」

丙组：不是对系统立的规则。它们混在规范句里，会让「谁强制」这个问题没有答案。

| 句子（出处） | 实际对象 | 建议 |
| --- | --- | --- |
| 「`provider` 只是供应端的泛称，必须由具体模块说明它指哪一类供应端」（`spec/README.md` §核心产品词） | 文档作者 | 这是写作规则，归 I-04 的越层词检查；从规范句改成说明句 |
| 「一人多机连接同一本账本，账本必须备份」（`spec/system.md` §控制面自己的存储） | 运维的人 | 可测的部分已在 §备份与恢复「metadata 备份必须是由唯一写入者协调的一致备份集」；这句改成描述（「账本是唯一不可再生的权威，备份见下节」） |
| 「只覆盖局部研究、咨询或中间步骤的自动化必须使用无 Task Run 或 Room Invocation……不能绑定 Task 后再依靠 Prompt 声明『这次不算完整施工』」（`spec/run.md` §Workflow 与 Run 授权） | 启动 Run 的人。control 分不出「局部研究」与「完整施工」 | 机械后果已在：绑定 Task 的 Run 正常完成即提交「完成 Task」，由 Task「按当前 Revision、来源的新鲜度与分歧和逐项证据独立校验」。把句子改写成后果（「绑定 Task 的 Run 正常完成必触发完成校验；不想触发的自动化用无 Task Run」），或标 `> 理由：` |
| 「换掉全部界面与供应端之后内核必须保留什么，愿景文档已经回答」（`spec/system.md` §固定内核与受控端口） | 无——「必须」在名词从句里，脚本误报 | 不改文；清点脚本的规范句判定可排除「必须保留什么」这类疑问式 |

丁组：机制成立，措辞越过了 control 能做的事。

| 句子（出处） | 说明 |
| --- | --- |
| 「供应端不能统一拦截全部写入时，系统必须关闭原生控制器」（`spec/participant.md` §运行时与观测） | 第一阶段唯一的 Agency 运行时是 Herdr，`delivery.md` §开工前限时验证 已确认「API 与原生 controller 可交错写入」——control 关不掉 Herdr 的 TUI。实际机制是：绑定未声明拦截能力就选不了 `managed_single_writer`，CT-PARTICIPANT「`managed_single_writer` 下不得同时开放 Herdr API 写入与原生 controller 写入，尝试原生写入时执行不得继续声称策略成立」。把「关闭原生控制器」改成「不得声明 `managed_single_writer`」 |
| 「日志与 trace 使用稳定关联 ID，但不得包含密钥和完整敏感 payload」（`spec/system.md` §安全边界） | 脱敏是代码，但「不包含密钥」只能用已知密钥做反向测试，无法证明全集。可接受，标注即可 |

## I-04 各项能否机械化

| 规则 | 能不能 | 现状与依据 | 误报风险 |
| --- | --- | --- | --- |
| 路由表越层词（约束词进愿景/架构层） | 能 | 词表来源 `spec/README.md` §词汇索引 减去六个高频约束词；做法同 `check_dead_names.sh`（词边界 + 文件/子串豁免） | 低。中文形（发件箱、比较并交换、摘要）按 A-21 是架构层合法词，只查英文词形 |
| 产品名越层 | 能 | 当前 spec 七文件命中 Herdr/Dagu/Vikunja/Tuwunel/Cinny/Tauri/Electron 共 43 次（system 16、run 11、participant 10），架构层 23 次（participant.md 14）；A-33/A-36/A-23 清零后由检查保持 | 低。需按节豁免 spec 的「外部概念对齐」表与 `system.md` §组件 的 Herdr 行；`delivery.md`、research、references 整文件豁免 |
| 驼峰拼接名 | 能 | 清点 §四：正文命中除 decision-history 外只有 `ChangeSet`(76)、`ReviewSubjectRef`(14)、`GitHub`、`AppService`、`OpenCode`、`WebView`、`WezTerm`、`SemVer` | 低。豁免表十来行；正则漏 `ProjectV2` 这类含数字的，但那不是目标 |
| 首现对照（漏列） | 能 | `inventory_concepts.pl` §三 已产出；白名单是 `doc-discipline.md` 第四条的十个自然译名词 | 低。只查那十个词，不查释义式词 |
| 同层同表中英夹杂 | 只能窄版，先报告 | 按表检查词汇表配对同时出现；必须排除中文名是常用词的概念。W-12 那种「human scene」「task-bound Run reducer」不是配对问题，要另一条启发式：表格单元格内非词汇表、非代码体、非产品名的多词拉丁短语 | 中到高。Run 状态表含 `Run` 与「运行中」即误报；第二条启发式会命中所有未登记的英文专名 |
| 「需要」当规范词 | 能，但要先改文 | spec 内 32 处：17 处是状态名「需要关注」，其余 15 处全为描述句（`run.md`「Run 需要输入时」、`connections.md`「需要扩权时回到……」等）。grep 分不出角色 | 卡定为「spec 内『需要』只许出现在『需要关注』」即无误报；代价是先把 15 句改成「等待输入时」「要扩权时」 |

## 主评审员漏掉的 I 轴问题

查了什么：约束层七文件里每处指名第一方实现的地方（工具箱、control、编译器、索引、备份、锁、outbox），对照部件矩阵表 D 与 §二；写作规则里每条能落成 grep 的条目，对照 `src/build/docs/` 现有五项检查与 `dead_names.txt` 的内容；`pr-contract.yml` 的两条触发条件。结果：轮子无补充，散文管代码事补四条。

| 编号 | 位置 | 问题 | 判据 | 建议 |
| --- | --- | --- | --- | --- |
| I-09 | `CONSTRAINTS.md` §词汇与文档「禁用张力，一律写冲突」；`WRITING-GUIDE.md` L7 词汇黑名单（张力、合同） | 机制就在旁边却没用上：`check_dead_names.sh` 就是词表扫描，`dead_names.txt` 里没有这两个词。当前 `docs/` 命中为零（只剩 WRITING-GUIDE 自己的示例行），加两行词表、一行豁免就把这条从散文变成卡 | I2 | `dead_names.txt` 加「张力」「合同」，`dead_names.allowlist` 豁免 `WRITING-GUIDE.md` 的黑名单表与 L7 示例行；「中断」按 `dead_names.txt` 头注已裁为太常用不收，维持 |
| I-10 | `WRITING-GUIDE.md` T1「本语料不设 SHOULD 档……实测：应当／不应 0 处」；L1 零修饰词表 | T1 拿一个实测数字当依据，数字没有卡会漂。我今天 grep `docs/design` + README：应当/不应 0 处，L1 词表（革命性、无缝、robust、seamless……）0 处——两条卡现在加是免费的 | I2 | 两个 `sh_test`：spec 内「应当｜不应」为零；`docs/design` + README 内 L1 词表为零。都是纯词表 grep |
| I-11 | `src/build/docs/report_prohibition_density.sh` 与它服务的验收「大修后禁令总数必须净减」 | 脚本自注「Report-only: always exits 0」，「净减」由人读报告判。基线数字没有落盘，谁也说不出减了没减 | I2 | 报告加一个基线文件（每文件一个数），卡定为「不高于基线」；改基线要进 PR |
| I-12 | `.github/workflows/pr-contract.yml`「自建信号：新增的脚本或第一方工具。只看新增文件，改已有脚本不算」 | 部件矩阵找到的最大一块轮子——运行时服务生命周期 675 行 shell——不是一次新增出来的，是在已有脚本上长出来的。现有卡只在 `--diff-filter=A` 时触发，往 `runtime.sh` 再加三百行不会要求写调研节 | I3、I5 | 触发条件加「已有脚本净增行数超过阈值」（例如 +100 行）也算自建信号；阈值写在 workflow 里，改阈值要进 PR |

轮子：**无补充**。约束层里由第一方实现的机制（`hctl2-tool` 五项、control 的账本/outbox/代次/租约/摘要/索引/备份/锁/秘钥/适配器、Workflow 编译器）在 `component-matrix-20260902.md` 表 D 与 §二 里逐条对照过业界候选并钉了版本；矩阵自己已经指出的两处半（Reindeer 源码编译、运行时进程监督 shell、未来若手写 HTTP 客户端）都在构建与发行层，不在约束层文本里，而且已有建议动作。约束层文本里唯一点名要 HCTL 自己做的通用件只有「全文索引……可重建的派生投影」与「一致备份集」，矩阵都判为 SDK 级胶水（FTS5、SQLite Online Backup API）。

---

统计：核验 8 条——维持 2（I-01、I-06）、修正 6（I-02、I-03、I-04、I-05、I-07、I-08）、推翻 0；漏掉的问题补 4 条（I-09 至 I-12，全部是「靠散文管本该靠代码管」；「自己造轮子」类无补充）。

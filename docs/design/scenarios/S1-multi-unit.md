# 参考用例 S1：多单元协作

> 状态：验证文档 · 草案 v0.18.6<br>
> 定位：所有者 2026-09-06 写下、09-07 与 09-13 两次补写的多机用例的正式落点。它是验证器的第零级：拓扑、步骤、必然发生的情形、不变量与变体在这里编号，文档检查、将来的同机多单元测试与真机验证都引用它，不各自再写一份。行为的含义由相应的约束定义，本文只记录拓扑、步骤与预期结果并引用权威条款；已拍板但尚未落到约束的方向标「待落地」。用例引用[契约测试矩阵](../contract-tests.md)里用例的描述文本，不引用序号。S1 主线只保留所有者文本中的动作及其必然后果；看板多源、共享卡、原生 Done 与施工图 lint 是用例外，单列在末节，不冒充 S1 必然情形。

## 引言：所有者的世界观

所有者原话（2026-09-06）：「关键问题是——1 个 repo 的多个 HCTL instance（比如 mac、ubuntu）之间，是否应当协同工作、如何协同工作？mac 上做了一个 task，要不要 ubuntu 上接过来做完？我之前希望是这样，但现在不觉得了——因为以前的视角是 harness session，那么就会希望 ubuntu 上的 codex 可以接过来正在执行的 task。但现在 ① 有了 participant 概念后，引入了远程 agency 抽象；② 有了 UI-backend binding 概念后，引入了多 workbench：单 control 分界后；我的世界观有了变化。」结论一句：「新的架构已经做了大量的 dependency/deployment 分离，让各种配置分立远程化。核心的安排是 agency、ctl、cli/bench。」机器之间不接管会话，传的是结晶与契约。

## 一、拓扑

| 单元 | 位置 | 内容 |
| --- | --- | --- |
| 仓库（底座） | GitHub 上的 `gh-jssdk`；GitLab 上的 `gl-jstui` | 两个仓库各绑自己来源的平台 |
| cloud_agency | 云端主机 cloud | 装 cloud_codex、cloud_claude、cloud_glm 三个 harness；工种 cloud_tpl_sde（cloud_codex 加 Skill）、cloud_tpl_sdet（cloud_codex）、cloud_tpl_pm（cloud_claude）、cloud_tpl_sre（cloud_glm） |
| mac_agency | mac | 装 mac_gemini、mac_k3；工种 mac_tpl_sdet（mac_gemini）、mac_tpl_ops（mac_k3） |
| ubuntu_agency | ubuntu | 装 ubuntu_glm、ubuntu_grok；工种 ubuntu_tpl_sde（ubuntu_grok）、ubuntu_tpl_pm（ubuntu_glm） |
| mac_ctl | mac | 控制面，基于 gh-jssdk 开两个 Project：mac_jssdk_01、mac_jssdk_02 |
| cloud_ctl | cloud | 控制面，基于 gl-jstui 开 cloud_jstui_01、基于 gh-jssdk 开 cloud_jssdk_01 |
| 前端 | mac：mac_cli、mac_bench；cloud：只在建 Project 时用过 cloud_cli，没有常驻本机前端；ubuntu：ubuntu_bench，瘦客户端，没有控制面 | 一个控制面可以有多个前端；一个前端可以连多个控制面 |

工种是各机器的 Agency 定义的（[Participant 约束 §对象](../spec/participant.md#对象)）；席位选的是工种的实例，harness 名（mac_gemini 等）不是工种。

## 二、步骤

| 编号 | 步骤（所有者文本） | 系统在这一步要决定什么、依据 | 权威条款 |
| --- | --- | --- | --- |
| S1.1 | mac_ctl 启动，基于 gh-jssdk 建 mac_jssdk_01 与 mac_jssdk_02 | 人显式登记仓库、声明平台绑定、开两个 Project；不推断、不去重仓库身份 | [Repo 约束 §Repo 注册](../spec/repo.md#repo-注册)；[C 批拍板](https://github.com/yesme/hctl2/pull/224) |
| S1.2 | mac_jssdk_01 经 mac_agency 拿三个参与者：mac_ptcp_jssdk_01_01、_02 都基于 mac_tpl_sdet（各自新会话、新工作树检出，可共用一个对象库），_03 基于 mac_tpl_ops（可与前两者共用对象库、各自检出） | 三条选入记录、三次派工；同工种两个是两条记录；对象库共用与否归参与者机器（V1a/V1b） | [连接约束 §从授权到派工](../spec/connections.md#project--run--participant从授权到派工)；[系统边界 §Repo 与执行现场](../spec/system.md#repo-与执行现场) |
| S1.3 | mac_jssdk_01 经 cloud_agency（远程）拿 mac_ptcp_jssdk_01_04，基于 cloud_tpl_sde，检出在 cloud | mac_ctl 与 cloud_agency 配对认证一次、得到自己的租户；此后派工、观测、报告都经 cloud_agency | [Participant 约束 §派工与观测](../spec/participant.md#派工与观测)；[系统边界 §端点与输入的信任边界](../spec/system.md#端点与输入的信任边界) |
| S1.4 | mac_jssdk_02 经 mac_agency 拿 mac_ptcp_jssdk_02_01（mac_tpl_ops，检出在 mac，可与 mac_jssdk_01 的参与者共用对象库或另 clone）；经 ubuntu_agency 拿 _02（ubuntu_tpl_sde）、_03（ubuntu_tpl_pm），检出在 ubuntu、共用一个对象库 | 同上；跨 Project 共用对象库是允许的布局不是必须的布局 | 同上 |
| S1.5 | mac_cli 与 mac_bench 都连上 mac_ctl 的两个 Project | 同一控制面多个前端，前端不拥有事实 | [三面架构 §三个面](../architecture.md#三个面) |
| S1.6 | cloud_ctl 启动，基于 gl-jstui 建 cloud_jstui_01、基于 gh-jssdk 建 cloud_jssdk_01 | 同 S1.1；gh-jssdk 同时被两个控制面开 Project 是常态 | [三面架构 §单元与连接](../architecture.md#单元与连接) |
| S1.7 | cloud_jstui_01 经 ubuntu_agency 拿 cloud_ptcp_jstui_01_01（ubuntu_tpl_sde）、经 mac_agency 拿 _02（mac_tpl_ops） | 同 S1.3 | 同上 |
| S1.8 | cloud_jssdk_01 经 cloud_agency 拿 cloud_ptcp_jssdk_01_01（cloud_tpl_pm）、经 mac_agency 拿 _02（mac_tpl_ops） | mac_agency 同时给 mac_ctl 与 cloud_ctl 供人，各一个租户 | [系统边界 §单写者](../spec/system.md#单写者) |
| S1.9 | cloud 只在建 Project 时用 cloud_cli，之后没有常驻本机前端；合入预览、完成 Task 这类要人拍板的动作经 ubuntu_bench 远程进来 | 无前端不等于不用人决策；治理动作可经远程前端进入 | [三面架构 §三个面](../architecture.md#三个面) |
| S1.10 | ubuntu 是瘦客户端，没有 ubuntu_ctl | 参与者机器不需要控制面 | [三面架构 §单元与连接](../architecture.md#单元与连接) |
| S1.11 | ubuntu_bench 连上 mac_jssdk_02、cloud_jstui_01、cloud_jssdk_01 三个远程 Project | 一个前端连多个控制面，联合视图里每项事实与动作保留控制面来源 | 同上 |

## 三、必然发生的情形

| 编号 | 情形（所有者文本） | 走法 | 权威条款 |
| --- | --- | --- | --- |
| S1.N1 | 同一工种被两个控制面雇成不同的参与者实例：mac_tpl_ops 同时出现在 mac_jssdk_01 与 cloud_jssdk_01 里，是两个参与者 | 两条选入记录，两个租户各一次派工 | [Participant 约束 §对象](../spec/participant.md#对象) |
| S1.N2 | mac_ctl 与 cloud_ctl 同时对 gh-jssdk 开 PR、同时请求合入 main | 各按冻结的授权形态与目标保护判，平台仲裁；另一方的合法合入是外部事实，按事先采纳的契约核验，不自动成为自己的完成凭证 | [Repo 约束 §集成](../spec/repo.md#集成目标两个头与两种授权形态)；[系统边界 §命令与跨服务正确性](../spec/system.md#命令与跨服务正确性)多写通则 |
| S1.N3 | mac 上 gh-jssdk 的对象库被 mac_ctl 的参与者和 cloud_ptcp_jssdk_01_02 共用 | 共享存储不共享会话、凭据与结果；引用不相交与写租约 | [Repo 约束 §ChangeSet 与 Git 事实](../spec/repo.md#changeset-与-git-事实) |
| S1.N4 | ubuntu 的参与者要推 gh-jssdk 和 gl-jstui，凭据在哪台机器、没有时谁中转 | 凭据在哪动作在哪：持凭据单元交付；没有凭据按既定中转；推送权限不等于更新目标的权限 | [Repo 约束 §发布评审](../spec/repo.md#发布评审) |
| S1.N5 | gl-jstui 的完整走通要等第二个外部平台适配器 | 适配器未到不等于人撤掉了平台绑定；本地封存与精确版本交付单独核 | [交付文档 §当前范围](../delivery.md#当前范围) |
| S1.N6 | 同一个人在 mac_ctl 与 cloud_ctl 里是两个 human actor；多控制面不等于多用户，跨控制面的「同一人」归并不做 | 不归并；一方的授权不当另一方的授权 | [系统边界 §客户端动作与 provider 事件](../spec/system.md#客户端动作与-provider-事件) |
| S1.N7 | 跨机返工：cloud 上封存的版本被驳回，返工在 cloud 那台机器上做，用原工作树或重新物化，未封存的字节不搬去别的机器；旧版本的评审失效要让远端参与者看得到 | 有 Run：同一 Seat 的新 Attempt 沿冻结的选入记录派到 cloud_agency 的本控制面租户，开工包带被驳回的评审对象与必用的 Verdict 正文；无 Run：人发起新调用并明确选人；原树续做还是重建归 Agency 门后；重建只用封存并获准交付的版本；失效可见靠开工包、新的评审对象与 Change 投影 | [Run 约束 §从节点到结果](../spec/run.md#从节点到结果)；[Repo 约束 §ChangeSet 与 Git 事实](../spec/repo.md#changeset-与-git-事实)；[Project 约束 §Room Invocation](../spec/project.md#room-invocation) |
| S1.N8 | 参与者不响应而控制面还活着（Room 里 @ 了没反应、Run 里超时）：Agency 超时与 failover 的问题，Agency 是参与者的代理 | Agency 在冻结的执行规格内自愈，控制面无感；规格内干不了报无法履约；Agency 不可达只是联系不上，按截止与取消条件处理；Agency 挂了由人指定别的 Agency；回来的旧执行体在门后机械核图与席位 | [Participant 约束 §派工与观测](../spec/participant.md#派工与观测)；[连接约束 §失败与恢复](../spec/connections.md#失败与恢复) |

## 四、不变量

每条写：来自哪一步、限定、钉住哪句设计文本、对到哪条既有契约测试用例的描述文本（或「本批新写」）、在哪一级检查（1 文档关联与词汇；2 同机多单元行为测试；3 真机）。

| 编号 | 不变量与限定 | 来自 | 钉住 | 契约测试用例（描述文本） | 级别 |
| --- | --- | --- | --- | --- | --- |
| S1.I1 | 一个 Agency 同时给多个控制面供人；一次派工只属于一个租户。检查办法：一方重启或撤权，另一方在同一 Agency 上的执行仍可工作 | 拓扑、N1 | [系统边界 §单写者](../spec/system.md#单写者)租户段 | CT-CONNECTION「派工 outbox 携带旧 `control_writer_generation` 时租户拒绝；控制面甲读取、订阅、输入、取消或收取乙租户的派工时在结构上不可达」；CT-SYSTEM「恢复使别的租户失权……时失败」 | 1、2、3 |
| S1.I2 | 同一工种被两个控制面雇成两个参与者，身份按所属控制面解释；共享工种不等于共享执行或授权 | N1 | [Participant 约束 §对象](../spec/participant.md#对象) Participant 行 | CT-PARTICIPANT「同一工种在两个 Room 或两个 Run 里选出的是两条记录，不共享身份」 | 1、2 |
| S1.I3 | 一个仓库同时被多个控制面开 Project；Repo Room 的唯一性只在控制面之内 | S1.1、S1.6 | [三面架构 §单元与连接](../architecture.md#单元与连接)控制面条 | CT-PRODUCT「S1 的 mac_ctl 显式在 gh-jssdk 建两个 Project，另一控制面也用该仓库……」 | 1、2、3 |
| S1.I4 | 检出与对象库在参与者所在的机器上；同机参与者可共用对象库（V1a）或各自 clone（V1b）；共用不表示可同时写同一工作树 | S1.2、S1.4 | [系统边界 §Repo 与执行现场](../spec/system.md#repo-与执行现场)；[Repo 约束 §ChangeSet 与 Git 事实](../spec/repo.md#changeset-与-git-事实) | CT-SYSTEM「共用对象库的独立工作树可并行，不因同库整体互斥」；CT-REPO「两控制面共用对象库时独立本地引用或平台源分支撞名，在写入前拒绝」 | 1、2、3 |
| S1.I5 | 两个控制面对同一仓库同时请求合入：各按冻结的授权形态与目标保护条件处理，平台仲裁；另一方的合法合入是要被观察的外部事实，不自动补出本方的批准或凭证 | N2 | [Repo 约束 §集成](../spec/repo.md#集成目标两个头与两种授权形态) | CT-REPO「两控制面各有同平台目标的待决意图时不加跨控制面锁；平台确认别人合法合入，双方分别核冻结条件……」 | 1、2、3 |
| S1.I6 | 共用对象库上两个控制面的工作分支与私有保管引用不撞名；共同的集成目标可以是同一个 main | N3 | [Repo 约束 §ChangeSet 与 Git 事实](../spec/repo.md#changeset-与-git-事实) | CT-REPO「两控制面共用对象库时独立本地引用或平台源分支撞名，在写入前拒绝」（与 I4 同一条；CT-PRODUCT 的 S1 行只是端到端组合入口） | 2 |
| S1.I7 | 远程参与者要拿到开工包：每个消费者一份 Bundle，共同条目的摘要相同；必用材料执行前交付为现场可反复读的精确副本 | S1.3、S1.4 的必然后果 | [Project 约束 §根 Context Manifest](../spec/project.md#根-context-manifest) | CT-PROJECT「必需材料未送达、摘要不符或只给在线控制面定位符时拒绝派发」；CT-PRODUCT S1 行「异机开工包的必需正文可在本地重读」 | 2、3 |
| S1.I8 | 推送在持凭据的单元上做，没有凭据时按既定中转；参与者不持材料库凭据；推送权限不等于更新集成目标的权限 | N4 | [Repo 约束 §发布评审](../spec/repo.md#发布评审) | CT-REPO「获准 Git 交付不因执行地点不在控制面就拒绝；参与者无凭据但有获准中转时可交付……」 | 2、3 |
| S1.I9 | 控制面没有常驻本机前端照常工作；治理动作可经远程前端进入；一个前端连多个控制面时，每项事实和每个动作都保留控制面来源 | S1.9、S1.11 | [三面架构 §三个面](../architecture.md#三个面) | CT-WORKBENCH-IA「打开入口按 repo 选择并统一映射到控制面连接」只覆盖入口映射；「第二期有权用户从 ubuntu_bench 对目标控制面预览并提交前置满足的治理命令成功且只作用于目标控制面，无故拒绝、丢失控制面来源或被当作本地动作都失败」本批新写，按第二期跨机交付阶段验证 | 显示与来源：1、3；动作来源：2、3 |
| S1.I10 | 联系不上不是丢失：失联本身不撤权，也不使取消或截止失效；各种期限按已接受的规则判；恢复后核对身份与结果 | N8 | [Participant 约束 §派工与观测](../spec/participant.md#派工与观测) | CT-PARTICIPANT「Agency 不可达只记联系不上、不撤权、不延长授权……只有语义归属者或租约不能证明才进丢失」 | 2、3 |
| S1.I11 | 控制面睡着时，参与者在已接受的范围与截止内继续干到封存，结果由 Agency 保管到接收方确认保全；醒来按关联键幂等收取；睡眠不改变已冻结的截止 | 所有者 2026-09-07 回答 4（用例正文没有这句，作补充来源） | [系统边界 §控制面自己的存储](../spec/system.md#控制面自己的存储)末段；[连接约束 §失败与恢复](../spec/connections.md#失败与恢复) | CT-CONNECTION「结果接收方只收到摘要不能确认字节保全；发送方保管至同一精确结果确认已保全」 | 2、3 |
| S1.I13 | 控制面只留两条：复用会话不复用授权；进程被杀后会话可恢复不证明旧执行仍成立。待命、不新起会话、闲置超时冬眠是 Agency 的产品要求 | 所有者回答 2；#228 裁决 4 | [Participant 约束 §派工与观测](../spec/participant.md#派工与观测)；[交付文档 §本地 Agency 参考实现](../delivery.md#本地-agency-参考实现) | CT-PARTICIPANT「控制面记录待命参数或会话状态时失败；同一参与者再次派工复用上次的授权或租约时失败……会话文件可恢复被当作旧执行仍成立时失败」 | 2、3 |
| S1.I14 | 跨机返工：有 Run 沿冻结的选入记录回到同一家 Agency，无 Run 由人明确选人；未封存字节不搬机；重建只用封存并获准交付的版本；失效可见靠既有路径 | N7 | [Run 约束 §从节点到结果](../spec/run.md#从节点到结果)；[Repo 约束 §ChangeSet 与 Git 事实](../spec/repo.md#changeset-与-git-事实) | CT-RUN「返工的新 Attempt 未沿冻结的选入记录派工……时失败」；CT-REPO「返工或另一台机器上的重新物化来源不是已封存并获准交付的 ChangeSet Revision……时失败」；CT-PROJECT「无 Run 返工是人发起的新 Room Invocation……」 | 无 Run 返工：2、3；涉及 Run 与 Gate 的：待 P2.5 |
| S1.I15 | 治理正文缺省不写进被治理仓库，人显式发布除外；参与者读的是交付副本；私有材料副本不随 clone 授予读权 | N3 的必然后果（共用对象库的两个控制面各有私有治理材料）；C 批 #227 | [系统边界 §Git 的双重角色](../spec/system.md#git-的双重角色) | CT-SYSTEM「材料存储不要求注册成 Repo 或另建 Room……」；CT-REPO「普通 clone 不含私有审计副本，显式导出只交付获准范围」 | 1、2、3 |
| S1.I16 | 同一个人在两个控制面是两个 human actor，不自动归并；外部账号相同也不能把一方的授权当另一方的授权 | N6 | [系统边界 §客户端动作与 provider 事件](../spec/system.md#客户端动作与-provider-事件) | CT-SYSTEM「同一个人在两个控制面是两个 human actor：被归并、或一方的授权被当作另一方的授权时失败」 | 1、2 |
| S1.I17 | 参与者不响应而控制面还活着：Agency 内部自愈无感、无法履约报回、联系不上按截止、换 Agency 由人、回来的旧执行体门后机械核图 | N8 | [Participant 约束 §派工与观测](../spec/participant.md#派工与观测)；[连接约束 §失败与恢复](../spec/connections.md#失败与恢复) Agency 两行 | CT-PARTICIPANT「Agency 门后接替后要求控制面换派工、改规格或重新激活时失败……」 | 2、3 |

编号 S1.I12 保留给「两个控制面的 Task 绑同一张卡」，所有者定为不作必然情形，见末节。

## 五、变体

| 编号 | 变体 | 说明 |
| --- | --- | --- |
| S1.V1a | 同机参与者共用一个对象库（mac_ptcp_jssdk_01_01/02/03；mac_jssdk_02_01 与 mac_jssdk_01 的参与者） | 两个变体都是主线——用例原文「两种做法都成立」——同机多单元测试两种都跑，不以哪种为准；各引用 CT-PRODUCT「S1 的 mac_ctl 显式在 gh-jssdk 建两个 Project……」与 CT-SYSTEM「共用对象库的独立工作树可并行」 |
| S1.V1b | 同机参与者各自 clone | 同上；对象库不共用时 I6 的撞名检查退化为平台源分支不撞名 |

## 六、用例外补充验证

以下不是 S1 必然情形，来源逐项注明；它们的规则在相应约束里，验证用例在契约测试矩阵。

| 编号 | 情形 | 来源 | 走法 | 契约测试用例（描述文本） |
| --- | --- | --- | --- | --- |
| S1.X1 | 人给 gh-jssdk 启用看板 | 所有者 2026-09-06 原话（缺省任务源列选项让用户选，缺省给平台自带的 issues，必须显式同意） | 候选列表缺省建议 GitHub Issues，显式同意才绑；启用看板的 Project 确认源引用；之后仓库建议变化不搬既有卡 | CT-TASK「任务源按仓库绑零到多个：未显式同意就绑了缺省源或建了卡时失败……」 |
| S1.X2 | 两个控制面各在 GitHub Issues 上有 Task；同一张 issue 被两边各认领一张 | 01 §五 推论；所有者把「两控制面同改一张卡」归多写通则（原 S1.I12） | 各控制面各有自己的合并板投影；卡内容以 GitHub 为准，治理状态各自独立；写回评论带控制面与 Task 标识；人的原生 Done 按各自绑定成请求 | CT-TASK「另一控制面的自动写回触发本控制面完成……时失败」「合并板被当作权威表写入时失败」 |
| S1.X3 | 人在 Linear 里的一张相关卡要进 mac_jssdk_01 | 所有者原话（一边 GitHub 一边 Linear） | 人显式认领进该 Project，卡留在 Linear；Linear 里的分组与 Project 归属脱钩；Linear 只过身份与快照时不能作缺省源 | CT-TASK「认领分两路……」 |
| S1.X4 | Linear 绑定被停用 | 01 §五（Kimi） | Task 保留、标需要关注；重新接通同一实体即恢复；换卡不做 | CT-TASK「家所在的源停用后 Task 与认领保留……」 |
| S1.X5 | 施工图里一个两票 Gate、两席声明相同、未声明互异 | 所有者 2026-09-13 原话；#230 拍板乙 | 登记结果机械列出「这两席可能合成一票」与可选处理，登记通过；启动预览按实际候选再算；法定票数三、席位两个则登记拒绝 | CT-RUN「lint 票数检查……」 |
| S1.X6 | 显式不挂平台的仓库返工 | 决策史 §36、C 批（受限路径） | 封存版本只在对象库与获准副本里，重建从获准来源取得并确认送达，不要求公共平台；没有评审请求可写 | CT-REPO「返工或另一台机器上的重新物化来源不是已封存并获准交付的 ChangeSet Revision……」 |
| S1.X7 | gh-jssdk 上一张卡被另一张未关闭的 issue 挡住（blocked by），人要为它启动 Run | 所有者 2026-09-17 裁决（依赖是任务源内部语义；派工时人确认） | 依赖留在 GitHub，卡片与合并板照源投影；启动预览列出阻塞方，人显式确认后启动，不自动拒绝；HCTL 不另存依赖 | CT-TASK「Task 依赖归源……」、CT-RUN「绑定 Task 的卡在源上被未关闭的阻塞方挡住……」 |

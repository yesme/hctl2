# 参考用例 S3：Project 用户路径

> 状态：验证文档 · 草案 v0.18.8<br>
> 日期：2026-09-19<br>
> 定位：把所有者已确认的[用户流程](../../user-experience/02-user-journey.md)与[导航和组织结构](../../user-experience/04-project-navigation.md)落实为可失败的验收路径，不另写一份需求或增加执行机制。CT 引用[矩阵](../contract-tests.md)的描述文本；以下是验收要求，不是已经执行的测试报告。

## 一、新建与接入

Project A、B 使用同一 Control、同一 Repo；除注明外，都只操作 A。卡片与平台地址使用测试夹具，不在真实仓库做删除实验。多控制面的授权、三家 Agency 和十一位 Participant 仍按 [S1](./S1-multi-unit.md)覆盖；本页的双 Source 是所有者 Apollo 导航示例，不冒充 S1 的必然步骤。

| 编号 / 来源 | 输入与动作 | 预期；怎样算失败 | 权威与 CT 描述 |
| --- | --- | --- | --- |
| S3.P1 / P1、C2 | 选同一远端 Repo，分别明确创建 A、B；再重试创建 A 的原命令 | A、B 各一间主 Room、各自授权，重试返回 A；合并 A/B 或多建第三间即失败。这里只决定新建哪份工作，不推断仓库身份 | [Project 注册与归档](../spec/project.md#repo-注册与-project-归档)；CT-PROJECT「创建 Project 同时建立它唯一的主 Room」、CT-REPO「人显式登记仓库及平台绑定」 |
| S3.P2 / P1、Q2 | 本地目录有 remote，分别选接原 remote、另起本地独立工作；另测无 remote 的目录 | 接原 remote 不 detach；另起默认新副本与新本地 Repo，原目录和 remote 不变；纯本地走本地平台。自动迁移 Issues/PR、用原 Repo 身份换绑或报告原目录已 detach 即失败 | [Repo 注册](../spec/repo.md#repo-注册)；CT-REPO「本地目录入口区分有 remote 与纯本地」与「注册只在本地的仓库时缺省绑定本地平台」 |
| S3.P3 / P1 异常、Q2 | 路径只在客户端机器存在，或读取失败；另选原地切换但不确认后果 | 显示路径所属机器与读取/确认问题，不按远端 Control 同名目录操作，不当纯本地另建；原地切换未确认就改 remote 即失败 | [Repo 注册](../spec/repo.md#repo-注册)；CT-REPO「本地目录入口区分有 remote 与纯本地」 |
| S3.P4 / P2 | A 接入 SCM Issues 和另一个 Source；第二个绑定只支持读取；再注入读取中断 | 两个 Kanban 入口分别可见，读取不认领；只读入口不能声称建卡成功，失败/未读全不能显示为空板。无授权时显示对应连接设置入口 | [Task 契约与来源](../spec/task.md#契约与来源)；CT-TASK「任务源按仓库绑零到多个」「按源看板把两个已连接源强制合成一个入口」 |
| S3.P5 / C2、Project 独立 Namespace 确认 | A、B 都接同一 Source；先在 A 认领外部卡 T，再用只获准管理 B 的命令认领 T，在 A 重复认领。A 的契约已满足，提交其完成请求；B 的契约仍缺一项人验收 | A、B 各得自己的 Task，A 重试仍是原 Task，外部卡只有一张；A 完成不使 B 完成。要求先取消 A 的 Task 或取得其写权限、拒绝 B 认领、共用 Task 或复制外部卡即失败；B 的认领不改 A | [Task 契约与来源](../spec/task.md#契约与来源)；CT-TASK「同一控制面的两个 Project 分别认领同一源卡应得到各自的 Task」「另一 Project 或另一控制面的自动写回」 |
| S3.P6 / P3 | 创建 A 后未选人、未接 Source，打开 A；再为主 Room 选人 | 主 Room 可读，Rooms 初始为空；选人不启 Run，聊天不因无 Kanban 被拒，主 Room 不在 Rooms 重列 | [Project 注册与归档](../spec/project.md#repo-注册与-project-归档)、[跨场景入口](../spec/connections.md#场景与第三方适配器)；CT-PROJECT「创建 Project 同时建立它唯一的主 Room」、CT-WORKBENCH-IA「进入 Project 默认打开自己的 Project Room」 |
| S3.P7 / 共享 Source 变化 | A、B 已各自认领卡 T；先原生改标题，再由绑定认可的人将卡转 Done，两方绑定均允许自动提交且预览无需临场选择，A 证据齐全、B 缺人验收；重置夹具后换 HCTL 账号写回 Done | 标题变化双方均可观测且不改契约或 lifecycle；人的 Done 分别成为完成请求，只有 A 通过。自动写回 Done 只作观测；漏掉一方、共用验收或写回冒充人即失败 | [Task 多写通则](../spec/task.md#契约与来源)；CT-TASK「A、B 的 Task 同绑一张源卡：原生修改标题」 |

## 二、讨论、承诺与计划

| 编号 / 来源 | 输入与动作 | 预期；怎样算失败 | 权威与 CT 描述 |
| --- | --- | --- | --- |
| S3.T1 / T1 | 主 Room 讨论已有决定甲、理由乙、未决问题丙，系统建议开 Topic；先忽略，再接受并纠正提要 | 忽略不创建；接受预览保留甲乙与未决丙、精确来源，并采纳用户删减/去敏；把丙写成已定、仅给链接、复制全史或另建 Project 即失败 | [Room 与消息](../spec/project.md#room-与消息)及创建 Topic 段；CT-PROJECT「Topic Room 的前情提要缺正文或精确来源」、CT-WORKBENCH-IA「创建 Topic Room」 |
| S3.T2 / T1 的开场材料 | 另一台机器上的新 Participant 只拿 Topic 开场包；随后主 Room 新增决定丁 | 能重读确认的提要并从中说明话题缘起、甲乙与未决丙；不要求先翻完主 Room，丁不自动混入。提要不能代替实际代码/契约或原授权 | [Room 与消息](../spec/project.md#room-与消息)、[三种交付方式](../spec/project.md#三种交付方式)；CT-PROJECT「Topic Room 首次调用只给原聊天链接」、既有「必需材料未送达」 |
| S3.T3 / T1 降级 | 自动归纳未配置或本次失败；允许手动补提要后创建 | 明示自动能力不可用，手动路径可继续；只有新建按钮或人工提要却报告“持续建议已实现”即失败。触发策略与费用控制仍待实现设计，不在本例预定 | [Room 与消息](../spec/project.md#room-与消息)；CT-PROJECT「自动归纳未配置或失败却报告自动建议已完成」 |
| S3.T4 / T1、独立选人 | 主 Room 选甲，Topic 预填甲乙；未确认就尝试调用乙；另建 Run 时仍用这份预填 | 未选入者不执行，Room 与 Run 分别确认选人；预填自动继承名单、授权或整份会话即失败。相同工种可再次选入，不要求强制换人 | [Project 名册](../spec/project.md#room-名册)、[Run 启动](../spec/run.md#启动与-manifest)；CT-PARTICIPANT「同一工种在两个 Room 或两个 Run 里选出的是两条记录」 |
| S3.T5 / T2 | 在主 Room 与 Topic 分别预览建 Task；A 有两个 Source，目标未定或选只读 Source | 先明确有能力且获准的目标 Source，普通聊天不直接写入；两种 Room 均能提交合法预览，不因主 Room 类型拒绝获准写入 | [Task 契约与来源](../spec/task.md#契约与来源)、[Room Invocation](../spec/project.md#room-invocation)；CT-TASK「新建卡未确认目标源」、CT-PROJECT「主 Room 与 Topic Room 均可按本次授权发起只读或写入调用」 |
| S3.T6 / T2、固定归属 | 两个 Topic 引用同一 Task 和 Run；关闭其中一个；尝试把消息挪到另一 Room、把 Task 拖到另一 Source 或 B | 引用保留原归属，关闭不结束关联工作；普通 Topic 不要求结案理由，关联 Request 未经原动作仍待处理。搬消息、搬 Task 或凭同 Repo 越过 Project 即失败 | [Room 与消息](../spec/project.md#room-与消息)、[Task 契约与来源](../spec/task.md#契约与来源)；CT-PROJECT「普通 Topic Room」「Room 的 Project 归属」、CT-TASK「跨源的相对移动拒绝」 |
| S3.T7 / Q3 | 丢弃未提交草稿；A、B 各自的 Task 同绑卡 T，A 无活动 Run、B 有活动 Run。先取消并归档 A、尝试取消 B，再由有源删除权的人预览并确认删 T；分别注入响应未知与确认成功 | 草稿可丢弃，取消 A 保留历史、源卡和 B，B 因活动 Run 拒绝取消；删卡预览列出双方 Project、Task 状态与 B 的 Run，另确认共享后果与 Run 去向，但不授予取消 B 的权限。响应未知不报成功；确认后双方观测 tombstone，B 仍是原 Task、未完成未取消，Run 不被自动停止 | [Task 契约与来源](../spec/task.md#契约与来源)、[写入约束](../spec/task.md#写入约束)；CT-TASK「取消并归档未通过取消前置」「A、B 的 Task 同绑一张源卡且 B 有活动 Run」 |
| S3.T8 / T3 | 为 A 的 Task 填模板并登记 W，注入响应丢失后重试，切页再回来；编辑成 W2，尚未批准开工；另注入登记失败、Task 与 Project 不符，以及不带 Task 的合法登记 | 关联随 Run 模块登记准入，Task 可找回 W/W2；重试不重复、失败不留关联、跨 Project 拒绝，不带 Task 可登记。旧版不覆盖，保存不启 Run；无显式跳过声明时仍需默认读回，模板不免授权 | [Workflow 与 Run 授权](../spec/run.md#workflow-与-run-授权)；CT-RUN「保存计划只登记并关联 Workflow Revision」「施工图确无来源 Room 时」 |
| S3.T9 / T3 无 Run 路径 | 各例从开放文档 Task T、当前契约 R2 只需精确文档和人验收、无 Run 占用且无候选的初态独立开始；分别注入获准单次调用准入的 ChangeSet Revision、人工封存准入的 ChangeSet Revision、人工发布的 Artifact Revision，均明确关联 T/R2；另注入普通文件、未准入提案、无 Task 关联、只关联另一 Task 或 T/R1 的产出 | 三种有效交付均可从待处理面板回到 T，由有权用户按原完成命令验收；反例不计，人工交付不补造 Invocation，准入不自动完成。需要合入的另走原集成流程，不强行建 Run、不增加预览次数 | [Task 写入约束](../spec/task.md#写入约束)；CT-TASK「候选交付按准入记录判定」「验收契约未要求代码集成」、CT-PRODUCT「无 Run 路径、有契约的 Task、默认发布策略」 |
| S3.T10 / 既有 Request 升级路径 | A 的 Run 发出缺输入的 Request，主 Room 无相关消息；人以请求与冻结阻塞对象准备提要并创建 Topic，另试错误 Project/阻塞版本；以缺省配置将本房间与普通 Topic 都置为闲置 15 天（Request 截止晚于观察时点），再分别测试解决 Request 与关闭房间 | 合法创建不要求或伪造主 Room 消息，错来源拒绝；仅承接开放 Request 的活跃房间进需要关注，不另增待办；解决 Request 后无此提醒，关闭 Room 不解决 Request。普通 Topic 不被提醒或强制结案 | [Topic 创建与消息](../spec/project.md#room-与消息)、[Request](../spec/project.md#request)；CT-PROJECT「Run 的 Request 在主 Room 没有相关 Message」「两间活跃 Topic 同样闲置 15 天」 |

## 三、Run 观察与异常

| 编号 / 来源 | 输入与动作 | 预期；怎样算失败 | 权威与 CT 描述 |
| --- | --- | --- | --- |
| S3.R1 / R1–R2 | A 有执行中、等检查、等人、暂停的 Run；一个 Run 两节点并行；切换 DAG 与任务书 | 活动条目都能找到；默认 DAG、并行节点同时可见，侧栏保持本 Run 的 Worker 与步骤。不因无运行 Worker 隐藏等待项，不产生第二次 Run | [跨场景入口](../spec/connections.md#场景与第三方适配器)；CT-WORKBENCH-IA「Run 导航隐藏等待或暂停中的活动 Run」 |
| S3.R2 / 待你处理 | 同一待本人应答 Request 关联 Room/Task/Run；另有待本人确认的发布意图、契约变化待本人采纳的 Task、S3.T9 中本人有权确认完成的 Task 及其交付正反例、过渡态超时 Run。再注入他人待答 Request、本人无权确认完成的 Task、普通未读、Topic 建议、已打开的 Trigger Preview、自动等 CI，另测 Request 已承接同一 Task 验收动作；点击 Project 名与标记 | 名称进主 Room，标记进面板；四类实际来源均显示，各来源去重并回答四问；若 Request 已承接同一验收动作不再另计 Task。他人待答、无权处理、交付反例、消息、建议、预览本身与自动等待不计；任一漏项或多计即失败 | [Project Overview](../spec/project.md#repo-注册与-project-归档)；CT-PROJECT「待你处理按现有事项去重」「待处理来源分别注入」 |
| S3.R3 / 待处理的完成 | 先阅读/关闭面板，再提交一个失败动作；随后另一客户端成功处理 | 阅读或失败不销项；实际生效才退出列表，原处保留历史，旧面板再提交须重核。并发操作双重生效或原动作之外写投影状态即失败 | [Project Overview](../spec/project.md#repo-注册与-project-归档)、[跨场景入口](../spec/connections.md#场景与第三方适配器)；CT-PROJECT「待你处理按现有事项去重」、CT-WORKBENCH-IA「同一 Request ID 跨 Room/Task/Run 聚合去重」 |
| S3.R4 / R3 | 依次选有图形观察、终端观察、仅 Headless 的派工；另有同名 Worker；旧派工也有记录 | 按所选派工的能力与授权呈现，Headless 仍有进度/结果；观察不授输入权，旧画面不冒充当前，关闭面板不取消。转投同名 Worker 或伪装缺失能力即失败 | [Participant 端口与输入](../spec/participant.md#写入约束)、[跨场景入口](../spec/connections.md#场景与第三方适配器)；CT-PARTICIPANT「control 签发连接票据、Agency 校验，观察、输入、Attempt 控制与安全输入权限分离；票据不经 Agency 校验就生效、票据含主机或终端 ID、控制面或前端绕过 Agency 直连门后的进程、PTY 或 API、工具或参与者绕过 Agency 直接向控制面报告、控制面存储出现门后地址时失败」、「`native_interactive_allowed` 下经 Agency 的原生客户端输入是有效运行时输入，该输入不能直接产生领域结果，其中的文字与“完成”不准入；Agency 未声明逐次输入记录能力时，还必须标明逐次 provenance 和物理单写者保证不完整」、「`managed_single_writer` 下绕开有效输入授权的动作不得落到该派工；两个客户端同时持有同一目标的输入租约时失败，接管必须原子撤销旧租约；Agency 不能统一拦截全部写入时执行不得继续声称策略成立」、CT-WORKBENCH-IA「Run 导航隐藏等待或暂停中的活动 Run」 |
| S3.R5 / R4 | Worker 已结束、检查成功，但评审或集成未确认；另测 Run 完成而 Task 当前契约已升级 | 分别显示已知事实、对应版本与缺项；不把 Worker 退出、平台 merged 或旧版绿灯当成 Task 已验收。无代码交付的 Run 不被强加集成环节 | [Run 完成谓词](../spec/run.md#写入约束)、[Task 写入约束](../spec/task.md#写入约束)；CT-WORKBENCH-IA「进度分开投影执行、检查、评审、集成与 Task 验收」 |
| S3.R6 / R5 | 分别注入 S2 的契约升级、评后换版、施工者失联/回来、合入响应丢失 | UI 逐项符合 [S2 五个回答](./S2-rough-road.md#三每种情形的五个回答)，列明影响、系统正在做什么、是否需人动作；保留上下文与原入口。授权已失效却显示可继续写、合入未知却引导再合一次即失败 | S2 引用的各模块约束；CT-PRODUCT「S3 用户路径逐项按输入注入失败」 |
| S3.R7 / R5、S1.N8 | Agency 失联，执行可能继续；到另一前端连原 Control | 标最后观察时间与当前未知，不自动换 Agency 或重建工作；新前端仍看原 Project，不接管或搬迁 Harness Session。跨机动作按既定交付阶段验，不以文档存在算实现通过 | [派工与观测](../spec/participant.md#派工与观测)、[恢复](../spec/connections.md#失败与恢复)；S1.I9/I10/I17、CT-PARTICIPANT「Agency 不可达只记联系不上」 |

## 四、验证结果怎样记录

每行分别记录：文档依据已核、可执行测试是否已有、行为测试输入与结果、真机观察与缺口。只跑 Markdown、版本或链接检查时，后两项仍记未执行。提要是否说清已定与未定需要读回核对；摘要一致只能证明拿到相同字节，不能证明摘要语义正确。自动建议、只读 Source 的写入能力、图形观察和跨机访问各按实际配置与交付阶段验，替代入口不冒充完整能力。

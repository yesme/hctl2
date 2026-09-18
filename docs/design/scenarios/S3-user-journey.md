# 参考用例 S3：Project 用户路径

> 状态：验证文档 · 草案 v0.18.7<br>
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
| S3.P5 / C2、Project 独立 Namespace 确认 | A、B 都接同一 Source，分别认领外部卡 T；再在 A 重复认领。A 的契约已满足、B 的契约仍缺一项人验收 | A、B 各得自己的 Task，A 重试仍是原 Task，外部卡只有一张；A 完成不使 B 完成。拒绝 B 认领、共用 Task 或复制外部卡即失败；两方都观测源端变更，但分别准入人提交的完成请求 | [Task 契约与来源](../spec/task.md#契约与来源)；CT-TASK「同一控制面的两个 Project 分别认领同一源卡应得到各自的 Task」「另一 Project 或另一控制面的自动写回」 |
| S3.P6 / P3 | 创建 A 后未选人、未接 Source，打开 A；再为主 Room 选人 | 主 Room 可读，Rooms 初始为空；选人不启 Run，聊天不因无 Kanban 被拒，主 Room 不在 Rooms 重列 | [Project 注册与归档](../spec/project.md#repo-注册与-project-归档)、[跨场景入口](../spec/connections.md#场景与第三方适配器)；CT-PROJECT「创建 Project 同时建立它唯一的主 Room」、CT-WORKBENCH-IA「进入 Project 默认打开自己的 Project Room」 |

## 二、讨论、承诺与计划

| 编号 / 来源 | 输入与动作 | 预期；怎样算失败 | 权威与 CT 描述 |
| --- | --- | --- | --- |
| S3.T1 / T1 | 主 Room 讨论已有决定甲、理由乙、未决问题丙，系统建议开 Topic；先忽略，再接受并纠正提要 | 忽略不创建；接受预览保留甲乙与未决丙、精确来源，并采纳用户删减/去敏；把丙写成已定、仅给链接、复制全史或另建 Project 即失败 | [Room 与消息](../spec/project.md#room-与消息)及创建 Topic 段；CT-PROJECT「Topic Room 的前情提要缺正文或精确来源」、CT-WORKBENCH-IA「创建 Topic Room」 |
| S3.T2 / T1 的开场材料 | 另一台机器上的新 Participant 只拿 Topic 开场包；随后主 Room 新增决定丁 | 能重读确认的提要并从中说明话题缘起、甲乙与未决丙；不要求先翻完主 Room，丁不自动混入。提要不能代替实际代码/契约或原授权 | [Room 与消息](../spec/project.md#room-与消息)、[三种交付方式](../spec/project.md#三种交付方式)；CT-PROJECT「Topic Room 首次调用只给原聊天链接」、既有「必需材料未送达」 |
| S3.T3 / T1 降级 | 自动归纳未配置或本次失败；允许手动补提要后创建 | 明示自动能力不可用，手动路径可继续；只有新建按钮或人工提要却报告“持续建议已实现”即失败。触发策略与费用控制仍待实现设计，不在本例预定 | [Room 与消息](../spec/project.md#room-与消息)；CT-PROJECT「自动归纳未配置或失败却报告自动建议已完成」 |
| S3.T4 / T1、独立选人 | 主 Room 选甲，Topic 预填甲乙；未确认就尝试调用乙；另建 Run 时仍用这份预填 | 未选入者不执行，Room 与 Run 分别确认选人；预填自动继承名单、授权或整份会话即失败。相同工种可再次选入，不要求强制换人 | [Project 名册](../spec/project.md#room-名册)、[Run 启动](../spec/run.md#启动与-manifest)；CT-PARTICIPANT「同一工种在两个 Room 或两个 Run 里选出的是两条记录」 |
| S3.T5 / T2 | 在主 Room 与 Topic 分别预览建 Task；A 有两个 Source，目标未定或选只读 Source | 先明确有能力且获准的目标 Source，普通聊天不直接写入；两种 Room 均能提交合法预览，不因主 Room 类型拒绝获准写入 | [Task 契约与来源](../spec/task.md#契约与来源)、[Room Invocation](../spec/project.md#room-invocation)；CT-TASK「新建卡未确认目标源」、CT-PROJECT「主 Room 与 Topic Room 均可按本次授权发起只读或写入调用」 |
| S3.T6 / T2、固定归属 | 两个 Topic 引用同一 Task 和 Run；关闭其中一个；尝试把消息挪到另一 Room、把 Task 拖到另一 Source 或 B | 引用保留原归属，关闭不结束关联工作；普通 Topic 不要求结案理由，关联 Request 未经原动作仍待处理。搬消息、搬 Task 或凭同 Repo 越过 Project 即失败 | [Room 与消息](../spec/project.md#room-与消息)、[Task 契约与来源](../spec/task.md#契约与来源)；CT-PROJECT「普通 Topic Room」「Room 的 Project 归属」、CT-TASK「跨源的相对移动拒绝」 |
| S3.T7 / Q3 | 丢弃未提交草稿；对无活动 Run 的工作执行取消并归档；再对有活动 Run 的卡尝试删除 Source 卡片 | 草稿可丢弃，取消通过原前置后收起、保留历史与源卡；有活动 Run 不绕过取消前置，删除另确认目标、后果与 Run 去向。删除响应未知不报成功，也不声称 Run 已停止 | [Task 契约与来源](../spec/task.md#契约与来源)、[写入约束](../spec/task.md#写入约束)；CT-TASK「取消并归档未通过取消前置」 |
| S3.T8 / T3 | 为 Task 填模板并保存版本 W，切页再回来；编辑成 W2，尚未批准开工 | 从 Task 找回 W/W2，旧版不覆盖，任务书与 DAG 同源；计划不在活动 Run 中冒充执行。无显式跳过声明时仍需默认读回，模板不免授权 | [Workflow 与 Run 授权](../spec/run.md#workflow-与-run-授权)；CT-RUN「保存计划只登记并关联 Workflow Revision」 |
| S3.T9 / T3 无 Run 路径 | 文档 Task 的契约只需精确文档和人验收；人或获准单次调用交出它 | 按原完成命令验收，不强行建 Run；需要合入的另走原集成流程，不增加预览次数。需要人的验收可从待处理面板回到该 Task | [Task 写入约束](../spec/task.md#写入约束)；CT-TASK「验收契约未要求代码集成」、CT-PRODUCT「无 Run 路径、有契约的 Task、默认发布策略」 |

## 三、Run 观察与异常

| 编号 / 来源 | 输入与动作 | 预期；怎样算失败 | 权威与 CT 描述 |
| --- | --- | --- | --- |
| S3.R1 / R1–R2 | A 有执行中、等检查、等人、暂停的 Run；一个 Run 两节点并行；切换 DAG 与任务书 | 活动条目都能找到；默认 DAG、并行节点同时可见，侧栏保持本 Run 的 Worker 与步骤。不因无运行 Worker 隐藏等待项，不产生第二次 Run | [跨场景入口](../spec/connections.md#场景与第三方适配器)；CT-WORKBENCH-IA「Run 导航隐藏等待或暂停中的活动 Run」 |
| S3.R2 / 待你处理 | 同一 Request 同时关联 Room/Task/Run，另有普通未读、Topic 建议、自动等 CI；点击 Project 名与标记 | 名称进主 Room，标记进面板；真正待办计一次，四个问题与原处入口齐全，普通消息/建议/自动等待不计。把所有等待都报成人的待办即失败 | [Project Overview](../spec/project.md#repo-注册与-project-归档)；CT-PROJECT「待你处理按现有事项去重」 |
| S3.R3 / 待处理的完成 | 先阅读/关闭面板，再提交一个失败动作；随后另一客户端成功处理 | 阅读或失败不销项；实际生效才退出列表，原处保留历史，旧面板再提交须重核。并发操作双重生效或原动作之外写投影状态即失败 | [Project Overview](../spec/project.md#repo-注册与-project-归档)、[跨场景入口](../spec/connections.md#场景与第三方适配器)；CT-PROJECT「待你处理按现有事项去重」、CT-WORKBENCH-IA「同一 Request ID 跨 Room/Task/Run 聚合去重」 |
| S3.R4 / R3 | 依次选有图形观察、终端观察、仅 Headless 的派工；另有同名 Worker；旧派工也有记录 | 按所选派工的能力与授权呈现，Headless 仍有进度/结果；观察不授输入权，旧画面不冒充当前，关闭面板不取消。转投同名 Worker 或伪装缺失能力即失败 | [Participant 端口与输入](../spec/participant.md#写入约束)、[跨场景入口](../spec/connections.md#场景与第三方适配器)；CT-PARTICIPANT 的票据与输入用例、CT-WORKBENCH-IA「Run 导航隐藏等待或暂停中的活动 Run」 |
| S3.R5 / R4 | Worker 已结束、检查成功，但评审或集成未确认；另测 Run 完成而 Task 当前契约已升级 | 分别显示已知事实、对应版本与缺项；不把 Worker 退出、平台 merged 或旧版绿灯当成 Task 已验收。无代码交付的 Run 不被强加集成环节 | [Run 完成谓词](../spec/run.md#写入约束)、[Task 写入约束](../spec/task.md#写入约束)；CT-WORKBENCH-IA「进度分开投影执行、检查、评审、集成与 Task 验收」 |
| S3.R6 / R5 | 分别注入 S2 的契约升级、评后换版、施工者失联/回来、合入响应丢失 | UI 逐项符合 [S2 五个回答](./S2-rough-road.md#三每种情形的五个回答)，列明影响、系统正在做什么、是否需人动作；保留上下文与原入口。授权已失效却显示可继续写、合入未知却引导再合一次即失败 | S2 引用的各模块约束；CT-PRODUCT「S3 用户路径逐项按输入注入失败」 |
| S3.R7 / R5、S1.N8 | Agency 失联，执行可能继续；到另一前端连原 Control | 标最后观察时间与当前未知，不自动换 Agency 或重建工作；新前端仍看原 Project，不接管或搬迁 Harness Session。跨机动作按既定交付阶段验，不以文档存在算实现通过 | [派工与观测](../spec/participant.md#派工与观测)、[恢复](../spec/connections.md#失败与恢复)；S1.I9/I10/I17、CT-PARTICIPANT「Agency 不可达只记联系不上」 |

## 四、验证结果怎样记录

每行分别记录：文档依据已核、可执行测试是否已有、行为测试输入与结果、真机观察与缺口。只跑 Markdown、版本或链接检查时，后两项仍记未执行。提要是否说清已定与未定需要读回核对；摘要一致只能证明拿到相同字节，不能证明摘要语义正确。自动建议、只读 Source 的写入能力、图形观察和跨机访问各按实际配置与交付阶段验，替代入口不冒充完整能力。

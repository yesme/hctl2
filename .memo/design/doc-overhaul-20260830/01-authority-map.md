# 当前文档权威表

> 快照：`origin/main @ 061bd7e`（正文与施工图基线 `192e7b6` 相同）。
> 覆盖：根 `README.md`、`docs/usage.md` 与 `docs/design/**` 的 20 个 Markdown 文件，共 194 个二级章节；`implementation-evidence.md` 没有二级章节，另以文件级一行纳入。`重复`表示该处只可概括或引用，不能成为第二份定义。

## 根入口与使用说明

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `README.md#为什么需要它` | vision | 在仓库入口概括产品要解决的四个问题与五类失败。 | 重复 | duplicated | 愿景权威在 `docs/design/vision.md#为什么需要-hctl2`；本节内容逐项复述该节。 |
| `README.md#四个阶段与四个模块` | architecture | 在入口页概括意图、承诺、治理、运行与四模块对应关系。 | 重复 | duplicated | `vision.md#四个阶段的心智模型`给出愿景解释，四份模块合同定义事实所有权。 |
| `README.md#目标体验` | vision | 在入口页概括从 Room 塑形到 Task 验收的用户旅程。 | 重复 | duplicated | 同名权威叙述在 `vision.md#目标体验`；合同细节在 `spec/connections.md`。 |
| `README.md#设计基线` | design | 在入口页摘要客户端、provider 事件、版本证据与终结来源。 | 重复 | duplicated | 精确动作分类在 `spec/system.md#客户端动作与-provider-事件`，终结合同在 `spec/task.md#写入合同`。 |
| `README.md#目标架构` | architecture | 给入口读者展示客户端、控制面和执行面的总图。 | 重复 | duplicated | 三面与替换边界的权威在 `docs/design/architecture.md`，组件合同在 `spec/system.md`。 |
| `README.md#设计文档` | design | 提供当前规范、合同、交付、术语、历史和研究的阅读入口。 | 是 | current | 所列目标文件在 `061bd7e` 全部存在；设计地图也指向同一组文档。 |
| `README.md#许可证` | delivery | 声明仓库按 Apache License 2.0 发布。 | 是 | current | 根 `LICENSE` 为 Apache-2.0，当前发行包也收录该许可证。 |
| `docs/usage.md#当前入口一览` | delivery | 列出当前已实现与尚未实现的命令、库和产品入口。 | 是 | current | `src/` 当前有 `hctl2-tool`、protocol 与打包服务入口，尚无公共 `hctl2`、control 或 Workbench 实现。 |
| `docs/usage.md#安装当前离线包` | delivery | 说明三目标离线运行包、源码伴随包与安装器合同。 | 是 | current | `src/packaging/dependencies/lock.json` 与 release Buck 目标固定三 target、双归档和摘要。 |
| `docs/usage.md#使用-hctl2-services` | delivery | 说明五个受管组件的启停、状态、冒烟、端点和数据位置。 | 是 | current | packaging 生命周期实现与 smoke 测试覆盖 Tuwunel、Cinny、Vikunja、Dagu、Herdr。 |
| `docs/usage.md#使用-hctl2-tool` | delivery | 说明当前工具箱骨架仅支持帮助与版本。 | 是 | current | 当前二进制实现对其他参数返回 `HCTL2_TOOL_UNSUPPORTED_ARGUMENT`；设计中的 P1 能力尚未落地。 |
| `docs/usage.md#安装完整离线包` | delivery | 从最终用户角度重述完整运行包的安装与启动入口。 | 是 | current | release 打包测试验证离线安装、幂等重装、启动、smoke 与停止。 |
| `docs/usage.md#从源码制作外部子系统包` | delivery | 给发布开发者说明 Buck2 分 target 构建和验收命令。 | 是 | current | `root//packaging/dependencies:package` 与 `:package-test` 是当前 Buck2 目标；lock 是唯一版本与摘要来源。 |

## 设计地图、愿景与架构

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `docs/design/README.md#对象关系` | design | 用图索引 Repo、Project、Room、Task、Run 与不可变版本的主关系。 | 重复 | duplicated | 对象与基数分别由 `spec/project.md`、`spec/task.md`、`spec/run.md` 定义；本节自称设计地图。 |
| `docs/design/README.md#场景客户端与受控端口` | design | 概括无等级客户端、公共命令入口与模块专用端口。 | 重复 | duplicated | v0.15.0 决定记录在 decision-history §31，精确分类在 `spec/system.md`。 |
| `docs/design/README.md#共同规则` | design | 集中概括四模块共有的版本、命令、终结与单写者规则。 | 重复 | duplicated | 本节末尾明确把交接与共享机制权威交给 `spec/connections.md` 和 `spec/system.md`。 |
| `docs/design/README.md#文档纪律` | design | 定义设计、合同、交付、证据和历史各层的写作边界。 | 是 | current | `AGENTS.md` Repo 地图与各规范文件头采用同一分层；合同总则声明冲突时合同优先。 |
| `docs/design/README.md#支持文档` | design | 提供设计体系内部的职责索引与冲突解释顺序。 | 是 | current | 所列文件和锚点在当前树存在；末句给出模块、连接、系统、交付、证据的解释顺序。 |
| `docs/design/vision.md#一句话定位` | vision | 定义 HCTL2 的产品定位、四短语姿态与默认无界面体验。 | 是 | current | 根 README 第 3、7–8 行采用同一定位；后续 v0.15.0 未改产品目标。 |
| `docs/design/vision.md#为什么需要-hctl2` | vision | 定义产品面对的四个问题和五类失败模式。 | 是 | current | 根 README 只作入口摘要；四模块与合同分别承接这五类问题。 |
| `docs/design/vision.md#四个阶段的心智模型` | vision | 定义意图、承诺、治理、运行的解释模型及非流水线性质。 | 是 | current | 设计地图与四模块正文沿用该对应；`spec/connections.md`保留 Project 直达 Agent 的短路。 |
| `docs/design/vision.md#目标体验` | vision | 定义完整用户旅程、六个随时可回答的问题和注意力原则。 | 是 | current | delivery 的 CT-PRODUCT 将六问与安静成功写成验收。 |
| `docs/design/vision.md#两种控制制度` | vision | 区分人主导塑形与冻结边界内自动施工。 | 是 | current | `run.md#两种控制制度`与 `spec/run.md#workflow-与-run-授权`落实批准和开工分离。 |
| `docs/design/vision.md#产品原生核心与架构最小内核` | architecture | 定义项目连续性、固定内核与 local-first/client-server 的关系。 | 是 | current | `spec/system.md#固定内核与受控端口`和用户级账本合同承接该边界。 |
| `docs/design/vision.md#三类数据` | architecture | 用愿景语言解释 metadata、content、artifact 的分家价值。 | 重复 | duplicated | 三类数据的唯一精确定义在 `spec/README.md#三类数据`；本节明确链接该处。 |
| `docs/design/vision.md#设计原则` | vision | 提供当前合同未覆盖问题的产品取舍原则。 | 是 | current | 文件头明确本文用于裁决合同尚未覆盖的新问题；v0.15.0 客户端原则已写入第 14 条。 |
| `docs/design/vision.md#要解决什么不解决什么` | vision | 划定产品长期问题域与第一阶段不自研的类别。 | 是 | current | delivery 的范围与明确不做清单落地该方向；Herdr、Dagu 等实现遵循复用边界。 |
| `docs/design/vision.md#从这里读下去` | design | 为不同读者指向设计、合同、交接、系统、交付和历史。 | 是 | current | 所有目标路径和对应章节在当前树存在。 |
| `docs/design/architecture.md#三个面` | architecture | 定义展示面、控制面、执行面的组成、所有权与部署关系。 | 是 | current | `spec/system.md#组件`与用户级账本/执行面合同逐项支撑；根 README 架构图为摘要。 |
| `docs/design/architecture.md#场景与系统` | architecture | 定义四场景对应的四类 content/执行系统角色。 | 是 | current | `spec/README.md#核心产品词`列出五个系统角色名，四模块正文采用同一映射。 |
| `docs/design/architecture.md#避免供应商锁定` | architecture | 定义四模块各自的稳定端口、默认实现和替换范围。 | 是 | current | v0.14.1 decision-history §29 与 `spec/system.md#固定内核与受控端口`支撑。 |
| `docs/design/architecture.md#4×3-归属矩阵` | architecture | 定义四场景中三类数据的产品级归属图。 | 是 | current | 三类共同语义由 `spec/README.md#三类数据`定义；`spec/system.md#全系统事实权威地图`给出系统级落点。 |
| `docs/design/architecture.md#模块交接` | architecture | 用产品语言摘要四模块间的冻结交付物。 | 重复 | duplicated | 表头明确它是 `spec/connections.md#连接合同总表`的产品语言投影。 |
| `docs/design/architecture.md#数据丢了怎么办` | architecture | 用产品语言摘要不可用与永久丢失的恢复结果。 | 重复 | duplicated | 精确降级表在 `spec/connections.md#失败与恢复`，存储恢复在 `spec/system.md`。 |

## 四模块设计正文

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `docs/design/project.md#为什么存在` | design | 解释 Project 为何保存长期意图与 Room 连续性。 | 是 | current | `spec/project.md#对象`定义 Project、Room、Participant 等对象；无后续决定推翻。 |
| `docs/design/project.md#模块拥有什么` | design | 用产品语言说明 Project 的长期事实、content 与结晶归属。 | 是 | current | `spec/project.md`对象表和 `architecture.md#4×3-归属矩阵`支撑。 |
| `docs/design/project.md#关键规则` | design | 概括 Project 的命令、协作边、上下文、消息与未加密房间规则。 | 重复 | duplicated | E2EE 与命令规则又在本文件场景段及 `spec/project.md`多处完整定义；精确权威在合同。 |
| `docs/design/project.md#room-类型` | design | 解释 Repo Room、Project Room、Scoped Room 的用途和生命周期。 | 是 | current | `spec/project.md#repo-注册与-project-归档`及 `#room-与消息`提供精确合同。 |
| `docs/design/project.md#chat-room-场景` | design | 定义用户如何通过 Matrix、CLI 与 Workbench 使用 Project 场景。 | 重复 | duplicated | 场景路径有效，但 E2EE 准入与命令来源在本节和合同层重复展开；`spec/project.md#场景合同`为权威。 |
| `docs/design/project.md#模块交接` | design | 摘要 Project 与 Task、Run、Agent 的所有权方向。 | 重复 | duplicated | 本节明确把字段、事务和故障语义交给 `spec/connections.md`。 |
| `docs/design/task.md#为什么存在` | design | 解释 Task 为何把验收承诺从执行和外部终态中独立出来。 | 是 | current | `spec/task.md`写入合同将完成限制为同一 Task 命令的两类来源。 |
| `docs/design/task.md#模块拥有什么` | design | 说明 Task 的承诺尺度、契约、绑定、操作投影与结晶。 | 是 | current | `spec/task.md#对象`和 `#契约与来源`支撑；v0.10.1 的 Repo 级 Board 决定仍有效。 |
| `docs/design/task.md#关键规则` | design | 概括 lifecycle、Revision、provider Done、映射与终结来源。 | 重复 | duplicated | 精确 lifecycle、actor、Run claim 和 Receipt 已在 `spec/task.md#写入合同`完整定义。 |
| `docs/design/task.md#无-run-的轻量路径` | design | 定义简单承诺不经 Workflow 的用户路径与适用边界。 | 是 | current | delivery 纵向切片 A 与 `spec/connections.md#human-kanban--run-reducer--task--project验收与回流`支撑。 |
| `docs/design/task.md#kanban-场景` | design | 定义 Repo 级 Board、卡片交互、双重状态与原生客户端路径。 | 是 | current | v0.15.0 decision-history §31 与 `spec/task.md#契约与来源`支撑 Vikunja Done 路径。 |
| `docs/design/task.md#模块交接` | design | 摘要 Project 提炼、Run 授权、结果回流与 Agent 证据方向。 | 重复 | duplicated | 精确交接只在 `spec/connections.md`定义。 |
| `docs/design/run.md#为什么存在` | design | 解释通用引擎机械位置与 HCTL 语义治理的差异。 | 是 | current | v0.12.1 Dagu 改判保留该边界；`spec/run.md`让完成只由账本谓词决定。 |
| `docs/design/run.md#模块拥有什么` | design | 说明 Workflow Revision、Run、Obligation、Seat、Attempt、Gate 与 Receipt。 | 是 | current | `spec/run.md#对象`与 `#写入合同`逐项定义。 |
| `docs/design/run.md#两种控制制度` | design | 说明批准与开工分离及 Run 的反应式输入顺序。 | 是 | current | `spec/run.md`第 33 行与 `#workflow-与-run-授权`支撑先记账再推动 Dagu。 |
| `docs/design/run.md#关键规则` | design | 概括 Manifest 冻结、重试分类、独立评审、引擎边界和 Task 回流。 | 重复 | duplicated | 这些规则在 `spec/run.md`的写入合同、授权、结果、Request/Gate 与 Run→Task 中精确定义。 |
| `docs/design/run.md#workflow-场景` | design | 定义 Workbench、CLI 与 Dagu 管理界面的用户路径。 | 是 | current | v0.15.0 decision-history §31 及 `spec/system.md#客户端动作与-provider-事件`支撑。 |
| `docs/design/run.md#模块交接` | design | 摘要 Project/Task 到 Run、Run 到 Agent 与 Task 的方向。 | 重复 | duplicated | 精确事务和恢复在 `spec/connections.md`。 |
| `docs/design/agent.md#为什么存在` | design | 解释物理资源可替换而上层事实不可被终端反向定义。 | 是 | current | `spec/agent.md#对象`保留同一边界；v0.14.1 只更换 Agency 实现。 |
| `docs/design/agent.md#模块拥有什么` | design | 说明执行配置、Harness 目录、ChangeSet、运行时、终端与提案证据。 | 是 | current | `spec/agent.md#对象`逐项支撑。 |
| `docs/design/agent.md#关键规则` | design | 概括 worktree、写入、结果、观测、三条底线、代次与连接规则。 | 重复 | duplicated | 三条底线与精确写入/观测合同在 `spec/agent.md`、`spec/system.md`和 delivery CT-AGENT 重复定义。 |
| `docs/design/agent.md#terminal-场景` | design | 定义结构化检查、PTY 诊断、接管与两种输入保证。 | 是 | current | v0.15.0 decision-history §31 与 `spec/agent.md#终端通道连接与租约`支撑。 |
| `docs/design/agent.md#agency-与-herdr` | design | 说明 Agency 角色、Herdr 采用方式、三类接口与用户可见限制。 | 重复 | duplicated | v0.14.1 决定有效，但 v0.8.2 限制又在 `spec/agent.md`、system、connections、delivery 完整复述。 |
| `docs/design/agent.md#原生会话导入` | design | 定义原生会话导入为可选能力而非第一阶段前置。 | 是 | current | delivery 未决问题仍把范围与维护预算列为开放项，没有后续决定把它升为必做。 |
| `docs/design/agent.md#模块交接` | design | 摘要执行规格输入、结果提议输出与工具箱校验。 | 重复 | duplicated | 派发和结果准入的唯一精确合同在 `spec/connections.md`。 |

## 横切设计正文

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `docs/design/context.md#为什么存在` | design | 解释 Context 只管理开工包及其快、省、准目标。 | 是 | current | `spec/project.md#context-memo-artifact`落实根清单、消费包与交付字节。 |
| `docs/design/context.md#与相邻概念的分工` | design | 区分聊天、投影、清单、内容包、Memo、评论线、前序结果与会话内上下文。 | 是 | current | 对应合同分别落在 project/task/run/agent；未新增第五模块。 |
| `docs/design/context.md#投喂三档内联指针代取` | design | 定义执行体可达性驱动的三种上下文交付方式。 | 是 | current | v0.13.1 decision-history §26 与 `spec/project.md`第 61–63 行支撑。 |
| `docs/design/context.md#萃取与压缩中心设计` | design | 说明本地萃取阶梯、缺省不压缩与 small-brain 记录。 | 是 | current | v0.12.3 §21 与 Project 合同第 65–69 行支撑。 |
| `docs/design/context.md#前情提要房间的滚动上下文` | design | 说明权威骨架、出生来源链和滚动纪要的不同生命周期。 | 是 | current | Project 合同第 69 行把纪要定义为可重建派生缓存。 |
| `docs/design/context.md#同一-run-内的接力节点之间传什么` | design | 说明评审、返工和备用尝试只经结晶结果接力。 | 是 | current | v0.13.1 §26、`spec/run.md`第 81 行与 Project 合同第 61 行支撑。 |
| `docs/design/context.md#关键规则` | design | 汇总 Context 的可解释、冻结、传承、召回、晋升、成本和降级原则。 | 是 | current | `spec/project.md#context-memo-artifact`和 CT-PROJECT 覆盖这些可观察要求。 |
| `docs/design/context.md#场景` | design | 指出用户直接触摸 Context 的 Trigger Preview 与提升预览。 | 是 | current | `spec/project.md#场景合同`和 delivery CT-WORKBENCH-IA 支撑。 |
| `docs/design/context.md#模块交接与合同落点` | design | 为 Context 各部分指向 Project、Run、Task、Agent 与连接合同。 | 是 | current | 所列四个目标锚点在当前树存在；末段明确未入合同内容仍留 memo。 |
| `docs/design/participant.md#为什么存在` | design | 解释 Participant 身份为何必须独立于工具、模型和进程。 | 是 | current | `spec/project.md#repo-注册与-project-归档`第 46 行冻结 Participant revision 与角色绑定。 |
| `docs/design/participant.md#七件事分层` | design | 区分身份、角色、人设、模型、Skill、执行配置和物理执行。 | 是 | current | Project、Agent、Run 与连接合同分别拥有对应下层精确定义。 |
| `docs/design/participant.md#关键规则` | design | 定义 Participant、角色、Skill、权限与评审独立性的产品规则。 | 是 | current | `spec/run.md#request重试与-gate`与 `spec/system.md`Skill 合同支撑。 |
| `docs/design/participant.md#专业化-participant告别-byoa` | design | 说明带版本方法论 Skill 的专业化参与者与远程 Agency 关系。 | 是 | current | 现行合同支持 Participant/Skill/Worker Profile 分离；市场机制仍明确未设计。 |
| `docs/design/participant.md#模块交接与合同落点` | design | 为身份、配置、评审席位与冻结链指向四份合同。 | 是 | current | 所列 Project、Agent、Run、connections 章节均存在并承接对应事实。 |

## 交付与验证

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `docs/design/delivery.md#第一阶段范围` | delivery | 定义单用户单机第一阶段在 P2/P3 的模块与第三方交付范围。 | 是 | current | README 与 usage 仍声明早期实现；四默认依赖已在 lock 和离线包中。 |
| `docs/design/delivery.md#公共-cli` | delivery | 定义未来公共 `hctl2` CLI 的第一阶段命令面。 | 是 | current | `docs/usage.md`明确公共 CLI 尚未实现，因此本节仍是交付合同而非现状命令说明。 |
| `docs/design/delivery.md#明确不做` | delivery | 定义第一阶段及永久排除的交付项。 | 是 | current | vision 的问题域、v0.12.2 桥接决定与单用户合同支撑。 |
| `docs/design/delivery.md#实现阶段` | delivery | 定义 P0–P3 的施工顺序与出门条件。 | 是 | current | 当前代码处于外部依赖可运维与 `hctl2-tool` 骨架阶段，尚未达到 P2。 |
| `docs/design/delivery.md#纵向切片-a无-run-自举` | delivery | 定义 B2 无 Run 的第一次真实自举链路。 | 是 | current | `task.md#无-run-的轻量路径`及连接合同验收回流支撑。 |
| `docs/design/delivery.md#纵向切片-b完整治理` | delivery | 定义含 Dagu、Gate、返工、集成和 Task 回流的完整治理链路。 | 是 | current | `spec/run.md`、`spec/agent.md`与 `spec/connections.md`包含对应合同。 |
| `docs/design/delivery.md#kanban-content-后端切片` | delivery | 定义本地与远端任务后端的映射、采纳、写回和对账验收。 | 是 | current | `spec/task.md#契约与来源`及 CT-TASK 支撑。 |
| `docs/design/delivery.md#自举阶段` | delivery | 定义 B0–B6 何时可以切换 HCTL2 自身开发事实。 | 是 | current | P/B 双表由 decision-history §13、§15 记录，现行阶段表仍引用本节。 |
| `docs/design/delivery.md#契约测试矩阵` | delivery | 以 CT-PROJECT 至 CT-PRODUCT 定义第一阶段可观察验收。 | 是 | current | 各测试族分别引用现行合同；当前 Buck2 尚未实现这些产品级 CT 的全部目标。 |
| `docs/design/delivery.md#选型判据` | delivery | 定义外部系统的接口、运维、生命周期与限时验证准入。 | 是 | current | 四默认实现均有 research 证据与 lock；AGENTS 要求新增依赖先调研。 |
| `docs/design/delivery.md#开工前限时验证` | delivery | 固定 Dagu、Herdr、Tuwunel、Vikunja 版本、采用边界与 P0 事实。 | 是 | current | `lock.json`固定相同版本/commit；Tuwunel macOS 资产来自 HCTL2 Release，Herdr 来自官方 v0.8.2。 |
| `docs/design/delivery.md#打包策略选型判断首次消费时产品化` | delivery | 说明原生分发、平台矩阵、容器与 Windows 边界。 | 否 | contradictory | 第 270 行称 macOS Tuwunel 日常从源码构建，但第 262 行、`lock.json`和 packaging README 均使用 HCTL2 托管预编译资产，源码构建仅手动更新时启用。 |
| `docs/design/delivery.md#技术基线` | delivery | 汇总第一阶段语言、桌面壳、UI 库、存储与四外部系统选型。 | 是 | current | decision-history §30 采用 Tauri 2；lock 与 research 总表支撑四个外部系统。 |
| `docs/design/delivery.md#未决问题` | delivery | 保存仍未裁决的交付问题并标明已了结项的当前出处。 | 是 | current | 未划线条目在后续 decision-history 中没有裁决；划线项都附现行文档来源。 |

## 合同层总则

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `docs/design/spec/README.md#词汇分类法` | contract | 定义领域对象、票据记录、状态值、引用格式及命名门槛。 | 是 | current | 四模块合同按该分类书写；v0.9.1 后没有新增另一套分类法。 |
| `docs/design/spec/README.md#核心产品词` | contract | 维护 24 个核心产品词、5 个系统角色名和 6 个高频合同词白名单。 | 是 | current | 当前计数可复现为 24+5+6；设计正文采用该词集。 |
| `docs/design/spec/README.md#六族规则` | contract | 唯一定义 Revision、Binding、Receipt、Lease、命令与 Snapshot 的共同语义。 | 是 | current | 各模块合同只列成员并引用本节；glossary 明确不重复共同性质。 |
| `docs/design/spec/README.md#三类数据` | contract | 唯一定义 metadata、content、artifact 及三条共同规则。 | 是 | current | architecture 4×3 矩阵和 system 事实地图都引用本节。 |
| `docs/design/spec/README.md#词汇索引v091-归并后` | contract | 给当前合同词按六族、票据、引用和独立对象归类。 | 是 | current | 各模块对象表可逐项对上，v0.12.2 与 v0.13.0 只做后续清扫。 |
| `docs/design/spec/README.md#v091-归并对照` | history | 记录 v0.9.1 从旧复合名到当前合同词的核销。 | 否 | historical-only | decision-history §11 说明该轮归并；当前定义以词汇索引和模块合同为准。 |
| `docs/design/spec/README.md#v0103-清扫` | history | 记录四个未入册高频名降级为描述语或端口种类。 | 否 | historical-only | decision-history 小修订台账 v0.10.3 行指向本表。 |
| `docs/design/spec/README.md#v0111-词形收敛` | history | 记录驼峰专名、Intent 名与状态拼写的词形改写。 | 否 | historical-only | decision-history 小修订台账 v0.11.1 行指向本表。 |
| `docs/design/spec/README.md#v0122-清扫` | history | 记录 Room Event 除名、操作投影降级与“丢失”统一。 | 否 | historical-only | decision-history §20 与小修订台账 v0.12.2 行指向本表。 |
| `docs/design/spec/README.md#v0130-收窄` | history | 记录在场证明、沙箱、Engine 代次、E2EE 状态等改判核销。 | 否 | historical-only | decision-history §22–§25 记录相应转向；当前合同在模块和 system 文件。 |
| `docs/design/spec/README.md#外部对齐原则` | contract | 定义外部概念翻译、模块专用 adapter、binding 与迁移的共同原则。 | 是 | current | v0.14.1 §29 固定四模块各自替换边界；system 受控端口合同一致。 |
| `docs/design/spec/README.md#文件` | contract | 列出六份合同文件及各自唯一职责。 | 是 | current | 六个目标文件在当前树存在，职责与各文件头一致。 |

## Project 与 Task 合同

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `docs/design/spec/project.md#对象` | contract | 唯一定义 Project 模块的 Repo、Project、Participant、Room、Context、Request、Memo、Artifact 与 Invocation。 | 是 | current | 文件头声明对象/状态/写入者唯一权威；architecture 与 glossary 只作投影。 |
| `docs/design/spec/project.md#写入合同` | contract | 定义 Project 聚合的版本、lifecycle、命令写入者与不可变结果。 | 是 | current | connection 与 system 只引用这些端点，不另建 Project 状态机。 |
| `docs/design/spec/project.md#repo-注册与-project-归档` | contract | 定义 Repo 注册恢复、Project/Room 创建归档、Participant revision 与提升来源。 | 是 | current | `spec/system.md#repo-与执行现场`引用其 Repo 身份；backlog 对归档前置的改判尚未拍板。 |
| `docs/design/spec/project.md#room-与消息` | contract | 定义 Scoped Room、Matrix 消息顺序、冻结引用与未加密房间降级。 | 是 | current | v0.13.0 decision-history §24 与 connections 失败表支撑。 |
| `docs/design/spec/project.md#contextmemo-与-artifact` | contract | 定义 Context Manifest/Bundle、萃取、压缩、纪要、Memo 与 Artifact Revision。 | 是 | current | v0.12.3 §21、v0.13.1 §26 与 CT-PROJECT 支撑。 |
| `docs/design/spec/project.md#room-invocation` | contract | 定义单次调用生命周期、Retry、scope、lineage 与结果代次。 | 是 | current | connections 的 Project→Agent 派发与统一丢失规则引用本节。 |
| `docs/design/spec/project.md#request` | contract | 定义 Project 拥有的 Request 生命周期、去重、解决与应答面。 | 是 | current | `spec/connections.md#跨模块-request-回路`只定义跨模块字段与事务。 |
| `docs/design/spec/project.md#场景合同` | contract | 定义 Trigger Preview、human 执行边、mention 解析和 Chat provider 动作准入。 | 是 | current | v0.15.0 §31 与 system 客户端动作分类支撑；第一阶段 Matrix 结构化动作仍未交付。 |
| `docs/design/spec/project.md#外部概念对齐` | contract | 映射 Room、消息、mention、Scoped Room、Chat binding、Participant 与 Request 到外部体系。 | 是 | current | 总则外部对齐原则授权模块合同定义差异；不转移字段权威。 |
| `docs/design/spec/task.md#对象` | contract | 唯一定义 Task、Task Revision、Task Binding、Snapshot 与 Completion Receipt。 | 是 | current | 文件头声明唯一权威；Kanban 设计正文只作产品解释。 |
| `docs/design/spec/task.md#契约与来源` | contract | 定义 Repo 级任务 content、身份 claim、契约惰性、字段权威、provider Done 与对账。 | 是 | current | v0.15.0 §31 的 Vikunja 行与 CT-TASK 支撑；后端并发由 provider 自己处理。 |
| `docs/design/spec/task.md#写入合同` | contract | 定义 Task lifecycle、Run claim、完成命令两类来源与逐项 Receipt。 | 是 | current | `spec/run.md#run--task`和 connections 完成连接引用该准入。 |
| `docs/design/spec/task.md#启动-run-的前置与排序令牌` | contract | 定义 Start 前待采纳处理、fresh readback 与后端排序写回。 | 是 | current | CT-TASK 覆盖无契约、活动 Run、drift、移动与 Done 事件。 |
| `docs/design/spec/task.md#外部概念对齐` | contract | 对齐 Task、操作投影、placement、Snapshot 与外部 Issue/卡片概念。 | 是 | current | 总则授权模块对齐；Task 身份、契约与完成仍留在 HCTL。 |

## Run 与 Agent 合同

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `docs/design/spec/run.md#对象` | contract | 唯一定义 Workflow Revision、Deployment、Run、Engine binding、Obligation、Seat、Attempt、Verdict 与 Receipt。 | 是 | current | 文件头声明唯一权威；Dagu 私有对象不进入该模型。 |
| `docs/design/spec/run.md#写入合同` | contract | 定义 Run 聚合状态机、反应式输入、分歧处理、正常完成谓词与隔离。 | 是 | current | v0.13.0 decision-history §23 把代次和完成留在账本；CT-RUN 支撑。 |
| `docs/design/spec/run.md#workflow-与-run-授权` | contract | 定义 Workflow 正文/准入、批准与 Start、Manifest、Profile 和替代规则。 | 是 | current | Dagu 编译边界由 v0.12.1 §18 与 delivery P0 固定。 |
| `docs/design/spec/run.md#从节点到结果` | contract | 定义 Engine 等待点到 Obligation、Attempt、Proposal、Receipt 和路标推进的归约。 | 是 | current | connections 的 Run→Agent 与 Agent→Run 只负责跨模块信封。 |
| `docs/design/spec/run.md#request重试与-gate` | contract | 唯一定义 Run Request、五种重试身份、候选切换和独立 Gate。 | 是 | current | CT-RUN 覆盖 retry、quorum、regate、作者回避和迟到结果。 |
| `docs/design/spec/run.md#run--task` | contract | 定义正常 Run 如何进入 completion_pending 并请求 Task 独立验收。 | 是 | current | Task 写入合同与 connections 验收回流使用同一命令和幂等键。 |
| `docs/design/spec/run.md#外部概念对齐` | contract | 对齐 HCTL Workflow/Run 与 Dagu/BPMN 的定义、实例、检查点和机械 retry。 | 是 | current | v0.12.1 §18、v0.13.0 §23 确认 Dagu 只作路标。 |
| `docs/design/spec/agent.md#对象` | contract | 唯一定义 Worker Profile、Harness 目录、ChangeSet、租约、运行时、连接票据、Proposal 与 Evidence。 | 是 | current | v0.14.1 §29 移除 agentd 后仍保留这些 Agent 模块对象。 |
| `docs/design/spec/agent.md#写入合同` | contract | 定义 Agent 聚合写入者、生命周期、三条底线和可选执行加固。 | 是 | current | v0.13.0 §22 的改判及 system 命令/外部副作用合同支撑。 |
| `docs/design/spec/agent.md#changeset-与-git-事实` | contract | 定义 ChangeSet/worktree、Revision 摘要、集成意图、回读和孤本保全。 | 是 | current | CT-AGENT 与 `spec/system.md#外部权威副作用`支撑。 |
| `docs/design/spec/agent.md#运行时与观测` | contract | 定义代次分层、Agency/Herdr、输入策略、观测与 Result Proposal 信封。 | 是 | current | v0.14.1 §29、v0.15.0 §31 与 Herdr v0.8.2 research 验证支撑。 |
| `docs/design/spec/agent.md#终端通道连接与租约` | contract | 定义 attach、观察/输入权限、Input Lease、Execution Chat、Share 与恢复等级。 | 是 | current | Agent 设计 Terminal 场景和 CT-AGENT 以本节为精确合同。 |
| `docs/design/spec/agent.md#外部概念对齐` | contract | 对齐 Herdr terminal、Execution Runtime、PTY、ACP、resume、replay 与租约。 | 是 | current | 总则对齐原则与 Agency binding 保证不转移 HCTL 权威。 |

## 四模块连接合同

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `docs/design/spec/connections.md#连接模型` | contract | 唯一定义“目标命令+来源不可变引用”的跨模块连接形式。 | 是 | current | 四模块合同各自拥有端点状态；没有 Handoff 聚合或 clone 本地事务。 |
| `docs/design/spec/connections.md#连接图` | contract | 给出 Project、Task、Run、Agent 的合法方向与显式短路。 | 是 | current | `README.md`与 architecture 只作简化投影；Agent→Task 原始通道明确不存在。 |
| `docs/design/spec/connections.md#连接合同总表` | contract | 汇总每条跨模块连接的耐久输入、目标准入、提交与恢复依据。 | 是 | current | 本文件头声明连接合同唯一权威；各模块交接节都回链本表。 |
| `docs/design/spec/connections.md#project--task从讨论到承诺` | contract | 定义 Project 提案如何经 Task 命令、Git/后端 outbox 与回读成为承诺。 | 是 | current | Project 与 Task 合同分别拥有来源和目标状态；CT-CONNECTION 覆盖 handoff。 |
| `docs/design/spec/connections.md#project--task--run授权自动施工` | contract | 定义 Project/Task 到 Run Manifest、Task claim 与 Engine start 的原子连接。 | 是 | current | Run 授权合同与 Task Run claim 使用同一用户级账本事务。 |
| `docs/design/spec/connections.md#project--run--agent从授权到物理执行` | contract | 定义两类 owner 共用的 Execution Spec 和 Agency 预留/激活顺序。 | 是 | current | v0.14.1 Herdr 直接实现 Agency；Herdr 无 fence echo 的降级写在第 99 行。 |
| `docs/design/spec/connections.md#agent--project--run结果准入` | contract | 定义 Proposal 去重、逐输出代次校验与 owner 准入结果。 | 是 | current | Agent 合同定义字段，Project/Run 分别拥有语义结果；Task 不读原始状态。 |
| `docs/design/spec/connections.md#human-kanban--run-reducer--task--project验收与回流` | contract | 定义 human 与 Run reducer 如何提交同一 Task 命令并回流里程碑。 | 是 | current | v0.15.0 无等级客户端决定与 Task 独立验收合同支撑。 |
| `docs/design/spec/connections.md#跨模块-request-回路` | contract | 定义 Request 与来源 blocker 的字段、CAS、delivery、过期和恢复事务。 | 是 | current | Project 独占 Request lifecycle；其他模块只保存 request_id 与自身阻塞状态。 |
| `docs/design/spec/connections.md#版本权限与替代` | contract | 定义端到端版本链、上游漂移处理与逐级缩权。 | 是 | current | Run/Execution Spec/Proposal 的精确 digest 和代次字段逐层实现该链。 |
| `docs/design/spec/connections.md#失败与恢复` | contract | 唯一定义跨模块失败后可观察结果及执行身份丢失处理。 | 是 | current | 各模块引用本表，system 只定义共享恢复算法。 |
| `docs/design/spec/connections.md#场景与第三方适配器` | contract | 定义场景客户端只能走目标模块公共连接且不能建立平台捷径。 | 是 | current | v0.15.0 §31 与 system 动作分类支撑。 |

## 系统共享合同

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `docs/design/spec/system.md#组件` | contract | 定义 Workbench、control、tool、Herdr、Engine、chat server、task backend 与第三方平台职责。 | 是 | current | v0.14.1 §29 移除 agentd，当前 usage/packaging 采用相同组件集合。 |
| `docs/design/spec/system.md#固定内核与受控端口` | contract | 定义用户级 command service、模块专用端口、binding、扩展与 Skill 的共享机制。 | 是 | current | architecture 的供应端替换边界与 v0.14.1 §29 支撑；无跨模块通用 shim。 |
| `docs/design/spec/system.md#场景端口` | contract | 定义 Query、Preview、Submit、Subscribe 四类公共客户端操作与字段权威来源。 | 是 | current | Workbench、CLI 和 adapter 在 v0.15.0 后共用该合同。 |
| `docs/design/spec/system.md#客户端动作与-provider-事件` | contract | 唯一分类 content、human 请求、运行时输入、结果提议和不支持 mutation。 | 是 | current | decision-history §31 记录拍板；四模块合同逐项列出实际接纳路径。 |
| `docs/design/spec/system.md#命令与跨服务正确性` | contract | 定义 command envelope、actor provenance、幂等、outbox/readback 与规范摘要。 | 是 | current | v0.13.0 §22 撤销额外在场证明后保留两类 actor 来源；CT-SYSTEM 支撑。 |
| `docs/design/spec/system.md#外部权威副作用` | contract | 定义 tool/adapter 执行外部写的持久意图、conflict scope、凭据与对账。 | 是 | current | Agent 的集成合同和 Task provider 写回都引用该共享机制。 |
| `docs/design/spec/system.md#事实与存储` | contract | 定义 Repo Instance、用户级账本、Git 双重角色和全系统事实权威地图。 | 是 | current | v0.10.2 将 clone 本地账本移除；architecture 的三类数据与恢复摘要一致。 |
| `docs/design/spec/system.md#单写者` | contract | 定义 control、site 与 Agency binding 三层排他权和 generation。 | 是 | current | Agent/connection 的代次信封引用这些 generation；Herdr 无物理回显被明确降级。 |
| `docs/design/spec/system.md#启动与恢复` | contract | 定义取得锁、恢复账本/outbox、回读 provider、隔离旧代次和恢复租约的顺序。 | 是 | current | connections 失败表给可观察结果，本文拥有共享算法；无后续改判。 |
| `docs/design/spec/system.md#安全边界` | contract | 定义桌面壳、renderer、凭据、敏感输入与单用户威胁模型边界。 | 是 | current | decision-history §30 将 Tauri 2 与 Electron 安全网并列；CT-PACKAGING/INPUT 覆盖部分要求。 |

## 术语速查

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `docs/design/references/glossary.md#核心产品词` | research | 汇集四模块、四场景、Workbench、control、provider 和 adapter 的非规范短释。 | 否 | current | 文件头明确“速查入口，不是新的权威来源”；定义回链 vision、architecture 与 spec。 |
| `docs/design/references/glossary.md#三类数据数据类别不是对象` | research | 汇集 metadata、content、artifact 三类数据的非规范短释。 | 否 | current | architecture 的“三类数据”及各模块合同拥有规范定义。 |
| `docs/design/references/glossary.md#客户端动作分类不是对象` | research | 汇集 content mutation、human request、runtime input、result proposal 的非规范短释。 | 否 | current | `spec/system.md#客户端动作与-provider-事件` 是唯一分类权威。 |
| `docs/design/references/glossary.md#场景-系统对照` | research | 给出各场景客户端、content server 与 authority 的速查映射。 | 否 | current | architecture 与四模块合同分别定义场景映射和事实权威。 |
| `docs/design/references/glossary.md#revision-族不可变版本` | research | 汇集不可变版本类对象的字段与权威出处。 | 否 | current | Project、Task、Run、Agent 合同分别定义对应 Revision/Spec/Proposal。 |
| `docs/design/references/glossary.md#binding-族冻结的身份连接` | research | 汇集冻结身份连接类对象及其权威出处。 | 否 | current | 四模块合同与 connections 定义 binding 字段、生命周期和连接语义。 |
| `docs/design/references/glossary.md#receipt-族校验后的证明` | research | 汇集经回读校验的证明类对象及其权威出处。 | 否 | current | 各模块合同及 system 外部副作用合同拥有规范定义。 |
| `docs/design/references/glossary.md#lease-族单持有者独占权` | research | 汇集锁和租约类对象及其权威出处。 | 否 | current | `spec/system.md#单写者` 与各模块合同拥有规范定义。 |
| `docs/design/references/glossary.md#命令族持久命令与副作用` | research | 汇集 command、effect intent 与幂等记录类对象。 | 否 | current | `spec/system.md#命令与跨服务正确性` 和 `#外部权威副作用` 拥有规范定义。 |
| `docs/design/references/glossary.md#snapshot--观测族先观测后准入` | research | 汇集 provider 观测与快照类对象。 | 否 | current | 各模块合同定义观测字段，system 定义恢复时的 provider 回读。 |
| `docs/design/references/glossary.md#票据与规格步骤产物不是领域对象` | research | 汇集 Proposal、Manifest 与 Execution Spec 等步骤产物。 | 否 | current | Project、Run、Agent 合同分别拥有这些对象的规范定义。 |
| `docs/design/references/glossary.md#独立对象不属六族的领域对象` | research | 汇集不属于六个通用族的领域对象及其权威出处。 | 否 | current | 对应模块合同逐项拥有 Project Intent、Task、Request、Result 等定义。 |
| `docs/design/references/glossary.md#引用格式不是对象` | research | 说明稳定源引用的通用表示法。 | 否 | current | Project/Task 合同及 connections 的来源不可变引用要求支撑。 |

## 实现证据入口

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `docs/design/references/implementation-evidence.md`（无二级章节） | research | 仅把读者重定向到 `docs/research/` 的实现证据。 | 否 | superseded | 正文只有重定向；设计地图已直接列出 `docs/research/`，且基线正文没有对本文件完整路径的入链。 |

## 决策历史

| 来源（文件#章节） | 层 | 当前唯一职责（一句话） | 是否有权定义该事实 | 状态 | 证据（被哪个后续决定、代码或 lock 支撑/推翻） |
| --- | --- | --- | --- | --- | --- |
| `docs/design/references/decision-history.md#hctl1git-native-治理内核` | history | 记录 HCTL1 以 Git、Worktree 和 label 治理任务的起点。 | 否 | historical-only | HCTL2 当前四模块和用户级账本由 architecture/spec 定义；本节只保留谱系。 |
| `docs/design/references/decision-history.md#多-harness-终端是产品问题的起点` | history | 记录多 Harness 会话管理如何触发 HCTL2 设计。 | 否 | historical-only | 当前 Agent 模块由 Herdr/Agency 合同定义，不再由这一早期问题陈述定义。 |
| `docs/design/references/decision-history.md#从工作区直觉到四个权威模块` | history | 记录 Project、Task、Run、Agent 四模块的形成。 | 否 | historical-only | `architecture.md#四模块` 与四份模块合同是当前权威。 |
| `docs/design/references/decision-history.md#四个-scene-与模块对仗` | history | 记录 Chatroom、Kanban、Workflow、Terminal 与四模块的映射来源。 | 否 | historical-only | 当前映射由 architecture 和 design README 定义。 |
| `docs/design/references/decision-history.md#workbench第三方客户端与受控端口分离` | history | 记录早期 Workbench、原生客户端与 adapter 边界。 | 否 | historical-only | §31 进一步裁决客户端无等级；`spec/system.md#场景端口` 是当前合同。 |
| `docs/design/references/decision-history.md#当时的-conductor-只拥有机械状态后由-18-取代实现选型` | history | 记录曾选 Conductor 且限制其只拥有机械状态。 | 否 | historical-only | §18 已将实现改为 Dagu；Run 语义仍由 `spec/run.md` 拥有。 |
| `docs/design/references/decision-history.md#从-prompt-约束到机械终结权` | history | 记录将关键终结动作收归 human 与 reducer 的决定。 | 否 | historical-only | Task 与 system 合同中的 actor provenance、验收及命令准入是当前权威。 |
| `docs/design/references/decision-history.md#从开放-agent-mesh-到受控协作边` | history | 记录取消开放 Agent mesh、改用受控请求与提议的决定。 | 否 | historical-only | Agent、connections 与 Project Request 合同定义当前协作边。 |
| `docs/design/references/decision-history.md#延续治理合同同时更换对象与事实源` | history | 记录从 HCTL1 到 HCTL2 时保留治理原则并更换事实源。 | 否 | historical-only | 当前事实权威由 architecture 与 `spec/system.md#事实与存储` 定义。 |
| `docs/design/references/decision-history.md#参考实现各取所长` | history | 记录若干外部项目作为设计素材而非整体实现的用途。 | 否 | historical-only | 当前实现选择由后续 Dagu、Tuwunel、Herdr、Tauri 决策及 `docs/research/` 证据支撑。 |
| `docs/design/references/decision-history.md#过度精简的教训与愿景层的恢复` | history | 记录 v0.9.1 恢复愿景、场景与产品边界的原因。 | 否 | historical-only | `vision.md` 是当前愿景权威。 |
| `docs/design/references/decision-history.md#场景数据的三分metadatacontentartifact` | history | 记录 metadata、content、artifact 三分法的形成。 | 否 | historical-only | `architecture.md#三类数据` 是当前定义。 |
| `docs/design/references/decision-history.md#实现计划pb-双表与-p0-选型v0110` | history | 记录 v0.11.0 阶段的实现顺序和当时候选栈。 | 否 | historical-only | Conductor 与 Zellij 已分别被 §18、§19 取代，agentd 又被 §29 取代。 |
| `docs/design/references/decision-history.md#chat-server-定夺-tuwunelv0111` | history | 记录 Project 默认 homeserver 选定 Tuwunel。 | 否 | historical-only | 当前锁定版本与来源由 delivery、`src/third-party-sources.lock` 和 packaging 规则支撑。 |
| `docs/design/references/decision-history.md#四段施工序与组件正名v0120` | history | 记录 v0.12.0 的施工顺序与当时组件命名。 | 否 | historical-only | 其中 agentd/terminal backend 命名已由 §27—§29 的 Agency/Herdr 决定取代。 |
| `docs/design/references/decision-history.md#缺口审计转向传播与五个新机制v0120-审计` | history | 记录 v0.12.0 审计提出的转向传播与五项机制。 | 否 | historical-only | §22 撤销会话在场证明和可选底线，其余内容由现行 spec 吸收或约束。 |
| `docs/design/references/decision-history.md#适配器诚实合同终局结果观测截断与派生谱系v0120-补充` | history | 记录外部 provider 能力不足时的诚实降级规则。 | 否 | historical-only | Agent、system 与 connections 合同中的 outcome、观测和 lineage 字段为当前权威。 |
| `docs/design/references/decision-history.md#workflow-engine-从-conductor-改为-daguv0121` | history | 记录 workflow engine 改选 Dagu 的理由与边界。 | 否 | historical-only | §23 后续明确 generation 不由 Dagu 内部 ID 承载；delivery/Run 合同定义当前集成。 |
| `docs/design/references/decision-history.md#运行时后端从-zellij-改为-tmuxv0121` | history | 记录当时以 tmux 取代 Zellij。 | 否 | historical-only | §27—§29 随 Agency/Herdr 引入而取代 tmux 方案。 |
| `docs/design/references/decision-history.md#桥接退役结晶归位与概念清扫v0122` | history | 记录移除桥接阶段并把状态归还各模块的决定。 | 否 | historical-only | 当前 architecture、四模块合同与 connections 已吸收该决定。 |
| `docs/design/references/decision-history.md#context-合同裁决轮v0123` | history | 记录 Context 输入、所有权、快照与生命周期的裁决。 | 否 | historical-only | `context.md` 和各模块 Context 交接节是当前权威。 |
| `docs/design/references/decision-history.md#信任模型收窄三条底线不可关外层笼子可选cli-即人v0130` | history | 记录三条不可关闭底线及 human actor 的来源裁决。 | 否 | historical-only | vision、system、task 与 agent 合同保存当前规则。 |
| `docs/design/references/decision-history.md#代次不在-dagu完成与评审都在-hctldagu-只当路标v0130` | history | 记录 Dagu 只承载机械路线、代次与完成由 HCTL 控制。 | 否 | historical-only | Run、connections 和 system 的 generation/receipt 合同是当前权威。 |
| `docs/design/references/decision-history.md#room-对控制面明文可读不启用端到端加密v0130` | history | 记录默认不启用 Room 端到端加密以便 control 读取内容。 | 否 | historical-only | Project、delivery 与 usage 的当前 E2EE 说明支撑。 |
| `docs/design/references/decision-history.md#p0-只验证实际接口v0130` | history | 记录 P0 只验证 HCTL 实际依赖的外部接口这一方法。 | 否 | historical-only | 当时 tmux/agentd 例子已过时；现行实现证据位于 `docs/research/` 和 lock。 |
| `docs/design/references/decision-history.md#context-投喂三档与-run-内接力v0131` | history | 记录 Context 三档预算和同 Run 内接力规则。 | 否 | historical-only | `context.md`、Run 与 Agent 合同定义当前行为。 |
| `docs/design/references/decision-history.md#运行时-providerterminal-场景同构化v0132` | history | 记录把 Terminal 提升为 provider-backed Agency 的中间决策。 | 否 | historical-only | §28、§29 继续改名并最终确定 Herdr 直接实现 Agency。 |
| `docs/design/references/decision-history.md#中间方案agency-定名并拆分-agentdv0140已由-29-取代` | history | 记录 Agency 定名后仍保留 agentd 的中间方案。 | 否 | historical-only | 标题和正文均声明已由 §29 取代。 |
| `docs/design/references/decision-history.md#herdr-直接实现-agency并固定各模块的-provider-替换边界v0141` | history | 记录 Herdr 直接实现 Agency 以及四模块 provider 替换边界。 | 否 | historical-only | architecture、agent、system 与 delivery 是当前权威；源码/lock 采用 Herdr。 |
| `docs/design/references/decision-history.md#workbench-桌面壳改选-tauri-2v0142` | history | 记录 Workbench 桌面壳选择 Tauri 2。 | 否 | historical-only | delivery 与 `src/packaging/README.md` 保存当前发行事实。 |
| `docs/design/references/decision-history.md#客户端没有等级动作合同成为真正边界v0150` | history | 记录 Workbench、CLI 与原生客户端无特权等级的裁决。 | 否 | historical-only | vision、architecture 与 `spec/system.md#客户端动作与-provider-事件` 是当前权威。 |
| `docs/design/references/decision-history.md#小修订台账` | history | 记录不改变设计含义的小版本修订。 | 否 | historical-only | 当前基线版本由根 README 与 design README 声明；各条旧版本不定义现状。 |
| `docs/design/references/decision-history.md#当前设计` | history | 提供从历史页返回当前设计地图的导航。 | 否 | current | 该节只含指向 `docs/design/README.md` 的链接，不重复定义设计事实。 |

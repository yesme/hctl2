# S1 不一致账本：同一事实的跨层不同表述

> 状态：待拍板 · S1 产物之一，交所有者在 S1 末与死文清单、禁令盘点一并拍板（施工图 §12 拍板点 6）<br>
> 基线：main @ 37805fa（草案 v0.15.0）· 只读普查，未改任何正文<br>
> 去向：拍板后作为 S2 Fable 总图（`02-target-map.md` 迁移表）与 S3 GPT 施工的输入；「建议权威位置」是账本的事实判断，结构去留由总图定

读法：每条一行；类型 ∈ {矛盾 / 同义复述 / 限定条件不同的近似重复}。**⚠ 前缀 = 看似重复、其实限定条件不同，合并时会互相吃掉对方条目——最容易被误合并的一类**。行号以基线 37805fa 为准。共 16 条：矛盾 3、同义复述 9、限定条件不同 4。

## 一、施工图指定项（§8.1 / §8.2）

| # | 事实 | 位置 A | 位置 B（可多列） | 差异 | 类型 | 建议权威位置 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Tuwunel macOS 制品来源 | delivery.md:262（P0 第 3 项：「HCTL2 在自己的 GitHub Release 托管按 SHA-256 锁定的 macOS 包…源码、Rust 工具链与手动原生构建 workflow 不进入日常依赖图」） | delivery.md:270（打包策略：「Tuwunel 因上游没有 Darwin 二进制而从锁定源码原生构建」） | 前者：消费托管预编译制品；后者：每 target 源码构建。同文互斥，research 总表已记 2026-08-30 换用托管制品 | **矛盾** | delivery.md:262；:270 改为引用并注明源码构建仅用于更新托管制品 |
| 2 | Tuwunel 源码构建在打包流程中的角色 | delivery.md:262（源码构建不进日常依赖图） | usage.md:224（「不能只在 Apple Silicon 上交叉编译 Intel 包，因为 Tuwunel 原生构建…属于目标合同」；:216-233 整节把逐 target 源码构建写成常规打包路径） | usage 仍把逐 target Tuwunel 原生构建当作每个发行 target 的必做环节；delivery 已把它降级为例外（仅更新托管制品时） | ⚠️**限定条件不同** | delivery.md:262；usage.md 打包节改写为消费托管制品 + 源码构建仅限更新托管制品 |
| 3 | macOS 最低基线为 15 | delivery.md:270（「macOS 最低基线为 15」） | usage.md:20（「macOS 最低版本为 15」） | 两处口径一致，但该基线的论证出处是 tmux 官方二进制（decision-history §19），tmux 已于 v0.14.1 退场——复述一致、依据已失压（依据退场本身归死文账本 K-01 处理） | 同义复述（带论证失压注记） | delivery.md 打包策略一处定义；usage 引用 |
| 4 | agentd 是否仍是待装组件 | usage.md:212（「Herdr 由该服务命令管理，不安装成 HCTL2 自建的 `hctl2-agentd`」——以现在时提及已退场组件名） | spec/system.md:11-13（组件表：hctl2-control 内含 Herdr 适配代码，不实现终端会话服务；无 agentd 组件） | 设计/合同层已无 agentd；usage 用死名做对比对象 | **矛盾**（死名残留） | spec/system.md；usage.md 删该对比 |
| 5 | 设计文档版本戳 | participant.md:3、context.md:3（均为「草案 v0.14.1」） | docs/design/README.md:3、vision.md:3、spec/README.md:3 等（「草案 v0.15.0」）；另 vision.md:3 与 spec/README.md:3 标 v0.15.0 但日期 2026-08-29，architecture.md 为 v0.15.0 / 08-30 | 两份横切正文停在上一版；同版本内日期漂移 | **矛盾**（版本戳） | docs/design/README.md 基线行；全库统一 v0.15.0（大修收口时一次 v0.15.1） |
| 6 | Workbench 壳的安全负例形态 | delivery.md:224（CT-PACKAGING：「renderer Node/raw IPC/远程脚本」——Electron 时代用词） | delivery.md:276（技术基线：Tauri 2 主选 + Electron 安全网）；spec/system.md:210（已写成双形态合同：Tauri capability/permission + Electron 三开关并列） | 合同已双形态，CT 矩阵只覆盖 Electron 形态负例，Tauri 形态无对应用例（施工图 C4 已记「合同已改、测试未跟」） | ⚠️**限定条件不同** | spec/system.md:210；CT-PACKAGING 补 Tauri 壳中立用例（C4 执行） |

### §8.2 三组同构的全部复述处

| # | 事实 | 复述处（全部） | 差异 | 类型 | 建议权威位置 |
| --- | --- | --- | --- | --- | --- |
| 7 | Chat 房间不启用端到端加密的前置与降级 | project.md:25（关键规则一句）、project.md:66（场景段：原因+绑定检查+事后降级全文）、spec/project.md:14（对象表：绑定准入前置）、spec/project.md:29（写入合同：fresh 回读证明未加密）、spec/project.md:56（Room 与消息：第二合同前提+降级细则）、spec/project.md:107（对齐表）、spec/connections.md:161（失败表行）、architecture.md:33（场景表备注）、delivery.md:123（CT-PROJECT 三行测试）、glossary.md:87（Chat 端口绑定行）、usage.md:141（运行事实：Tuwunel 配置禁用房间加密） | 11 处。合同级定义散在 spec/project.md 四节；其余为设计层复述、测试用例与运行证据——后三者性质不同，不是重复权威 | 同义复述 | spec/project.md 一处定义（施工图 §8.2 已定）；project.md 留一句人话+指针；CT/usage 属测试与证据，保留 |
| 8 | 三条底线（工具不是人 / 合入钥匙不进工具 / 隔离工作树） | agent.md:27（设计正文人话版）、spec/agent.md:34（权威定义+细则）、spec/system.md:107（命令跨服务：工具不是人的命令面表述）、spec/system.md:123（外部权威副作用：钥匙不进工具+隔离树细则）、spec/system.md:214（安全边界：压缩承诺句）、spec/task.md:68（Task 终结两来源表述）、spec/connections.md:45（连接表行）、decision-history.md:189-195（§22 裁决源，合法历史）、delivery.md:97 + :174（B2 验收与 CT-AGENT）、README.md:99（设计基线 bullet） | 10 处。三句话以三种粒度（人话/细则/压缩句）散布；§22 是裁决出处非现行合同 | 同义复述 | spec/agent.md:34 一处定义（施工图 §8.2 已定）；system.md 改引用；设计正文与 README 留一句 |
| 9 | Herdr v0.8.2 的能力限制（栅栏回显 / 输入记录 / 事件游标 / 退出回读） | agent.md:71-76（四条限制清单+行为规则）、spec/agent.md:29（Terminal Input Lease 行：原生写入不受租约）、spec/agent.md:68（Agency 节：无栅栏回显→只在 HCTL 入口校验）、spec/agent.md:74（能力边界进入合同：完整四条）、spec/system.md:38（端口节：不能统一 writer gate）、spec/system.md:40（输入先经适配代码校验）、spec/system.md:181 + :183（单写者：不接收不回显 generation，两处）、spec/connections.md:99（启动顺序：声明 fence echo 的 Agency 拒绝旧代次；v0.8.2 不支持）、architecture.md:49（将来上游补齐再纳入 binding）、delivery.md:16（P3 出门：v0.8.2 不能声称 lease/provenance）、delivery.md:261（P0 第 2 项：已确认限制清单）、delivery.md:177-184（CT-AGENT 七行）、README.md:150（不能证明经过输入租约） | 13 处。版本号 `v0.8.2` 以「实现名+版本」形态进了合同层多处，与「实现名不进设计层、版本缺项下沉 binding/delivery」的既定方向相抵（拍板点 11 待裁：改写为能力条件句） | 同义复述（内含拍板点 11 的形态改写） | spec/agent.md 改能力条件句（有则如何、无则降级）；v0.8.2 缺项清单只在 delivery P0 与 binding 声明 |

## 二、本轮普查新发现

| # | 事实 | 位置 A | 位置 B（可多列） | 差异 | 类型 | 建议权威位置 |
| --- | --- | --- | --- | --- | --- | --- |
| 10 | Task 完成只有两个获准来源（有权 human / task-bound Run 正常完成的 reducer） | spec/task.md:68（权威定义） | README.md:99、docs/design/README.md:59、task.md:25、run.md:43、spec/run.md:105、spec/connections.md:45 + :119、spec/system.md:107、delivery.md:143（CT-TASK） | 10 处，是 §8.2 三组之外未点名的第四个同构群；各处表述一致但粒度三档（人话/合同/测试） | 同义复述 | spec/task.md；其余一句引用（并入 S2 同构合并映射） |
| 11 | Run Manifest 冻结字段的完整清单 | spec/run.md:49-55（「Run Manifest 至少冻结」清单） | spec/connections.md:61（「不可变 Run Manifest 固定…」散文完整罗列） | 两份清单**细目不一致**：connections 版含「端口绑定、网络/secret 范围」，run 版未列；run 版含「放置（placement）」，connections 版未提——同一对象两份近全清单各有对方没有的条目 | ⚠️**限定条件不同** | spec/run.md:49-55；connections.md:61 改引用（合并前须先并集核对，防互相吃掉条目） |
| 12 | Dagu `human.task` 的定位（被动检查点 / 机械暂停原语） | spec/run.md:67 + :85 + :116（生成物限制、机械暂停原语、对齐表） | delivery.md:260（P0 第 1 项：无进程 human.task=HCTL 外部执行检查点+不许 Dagu 自跑副作用） | 表述一致；delivery 版是 P0 验证口径，spec 版是合同定义，双份完整陈述 | 同义复述 | spec/run.md；delivery P0 留验证范围一句 |
| 13 | 一个 Repo 一个 Board | spec/task.md:24（权威：Board=任务 content 容器） | architecture.md:34 + :68、task.md:46、glossary.md:24 | 5 处复述，口径一致 | 同义复述 | spec/task.md；glossary/architecture 留指针 |
| 14 | 三类数据定义与统一律（artifact 是 content 的结晶） | spec/README.md:41-51（权威表+三条法） | vision.md:134-138（完整散文复述含统一律，仅尾部指向合同层） | vision 按纪律「不定义对象」，现状是完整复述+指针——按 §2 红线 4「权威去重只针对合同，不针对解释」，属解释性复述，删留需拍板 | 同义复述 | spec/README.md（权威不动）；vision 保留解释、删合同级措辞（C2 处理） |
| 15 | 目标体验八步 / 六个问题 / 设计基线 bullets | vision.md:69-99（目标体验、六问、设计原则） | README.md:69-99（几乎逐句同款的八步旅程、六问、设计基线 bullets） | 两份近全量同款叙事（施工图 §8.3 已列「README 收束为门户」候选） | 同义复述 | vision.md；README 留一句话+入口（C1 执行） |
| 16 | WezTerm 作为 Terminal 客户端的地位 | agent.md:53（角色表「场景客端：CLI / WezTerm」；另有错字「场景客端」） | delivery.md:16（「Herdr 官方 TUI 是原生 Terminal 客户端，WezTerm 可选」）；README.md:108（原生客户端只列 Matrix / Vikunja / Herdr，未列 WezTerm） | agent 角色表把 WezTerm 与 CLI 并列为一类场景客户端，delivery 定性为「可选」，README 未提——三处口径不一；且一等原生客户端已是 Herdr TUI，WezTerm 去留未裁 | ⚠️**限定条件不同** | agent.md Terminal 场景节；去留随死文普查拍板后三处统一 |

## 三、汇总与移交

- **条数**：16（矛盾 3 / 同义复述 9 / 限定条件不同 4）。
- **⚠ 误合并高危四条**：#2（源码构建是例外还是常规路径）、#6（CT 只测了安全网形态）、#11（两份 Manifest 清单各有独有条目）、#16（WezTerm 三处口径）——S2 总图做合并映射时必须逐条显式处置，不许笼统「合并同类项」。
- **移交**：#3 的论证失压与 #4 的死名、#16 的错字属死文/禁令账本（Grok/K3）接续；本账本只记表述差异。#9 含拍板点 11（Herdr 能力条件句化），#14 含红线 4（解释性复述删留），均已在施工图挂账。

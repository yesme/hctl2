# 结构审计：横向越层 + 纵向聚类

> 状态：待拍板 · Grok 结构审计（只报不改 `docs/`）
> 基线：main @ 2863632（草案 v0.15.4）
> 去向：Fable 汇总进 `20-verdict-packet.md`；只改写法的条目进本轮结构 PR，会改变含义的不在本轮落
> 范围：任务书所列 21 个文件。依据：`WRITING-GUIDE.md` §0.4 与 `## HCTL2 增补` S1–S3，本目录 `02-layer-routing.md`，所有者 2026-08-31 松紧基准。

口径先说清：

- **下沉**：高层文件里出现了低层才需要的细节（状态机口径、字段名、锁路径、阶段代号、实现名）。
- **上浮**：低层文件里出现了本该在高层的论证、口号或愿景表述。上一轮大修只查过下沉，本轮重点查上浮。
- 结构与用词分开报。口号该不该在这一层是结构问题；是不是人话归语言通读。已判「只列名字 + 链到约束层」的安排（五种重试路径）结构正确，不报为越层。「换人不换裁判」「掉线不丢身份」「谁点按钮不是关键」结构合格，不报。
- 不报篇幅。

---

## 一、横向越层清单

| # | 文件:行(节名) | 下沉/上浮 | 越到哪一层 | 原文摘要 | 建议：删 / 改成引用 / 搬到哪个文件 |
| --- | --- | --- | --- | --- | --- |
| 1 | `vision.md:104`（产品原生核心与架构最小内核） | 下沉 | ① → ③（兼 ②） | 最小内核表点名 Obligation / Seat / Attempt、outbox、预期版本、幂等；再给一条 `actor + 类型化命令 + 目标版本 + 证据 → control 与工具箱校验 → 领域事件 + outbox` 的状态转换 | ① 只保留「换掉界面和供应端之后，项目身份、授权和验收还必须在」这一句原则。对象表搬 `architecture.md`（组装）或改成引用 `spec/system.md` 固定内核。状态转换删，① 不需要命令信封 |
| 2 | `vision.md:91`（两种控制制度） | 下沉 | ① → ② | 塑形/施工对照表写到 control 自动推进、Run r1 冻结、Project Room 讨论 r2、引擎报告机械位置 | ① 设计原则里留「批准施工图和开工是两件事」。对照表搬 `architecture.md` 或 `run.md`「为什么存在」——那本来就是 ② 的组装问题 |
| 3 | `vision.md:136`（三类数据） | 下沉 | ① → ③ | 愿景正文写出 metadata / content / artifact 各住哪，再链总则「只定义一次」 | ① 只讲「记忆和裁决分家、故障隔离」。三类名字和权威所在留 `architecture.md` 4×3 与 `spec/README.md`。现在的写法等于在愿景里先上了一课对象 |
| 4 | `README.md:21`（目标架构） | 下沉 | 门面 → ② 和 ④ | 根 README 的图含 P2/P3、SQLite、`hctl2-control` / `hctl2-tool`、Dagu 直接 mutation、Matrix / Vikunja / Herdr 产品名 | 门面图只保留三面和四个模块。阶段代号、组件二进制、产品名、mutation 规则分别链 `delivery.md`、`spec/system.md`、`architecture.md`。见下文疑点 4 |
| 5 | `architecture.md:14`（三个面） | 下沉 | ② → ③ | 控制面一行写成约束层组件 `` `hctl2-control` / `hctl2-tool` `` | ② 写「命令服务与账本、现场执行者」。二进制名是 `spec/system.md` 组件表的事 |
| 6 | `architecture.md:36`（避免供应商锁定） | 下沉 | ② → ④ | 稳定边界表把 Tuwunel / Vikunja / Dagu / Herdr 写成第一阶段默认实现 | ② 只留系统角色和端口。默认实现名 `delivery.md` 已经有选型表，这里改成引用。可替换三档承诺留 ②——那是组装规则，不是选型 |
| 7 | `architecture.md:62`（4×3 归属矩阵） | 下沉 | ② → ③ | 结晶规律一段点名 current pointer、lifecycle、Receipt 以约束层标定 | ② 矩阵用产品语言（身份、授权、判决住账本；正文进 Git 不等于已被接纳）。`current pointer` 与 lifecycle 留 `spec/system.md` Git 双重角色 |
| 8 | `architecture.md:86`（数据丢了怎么办） | 下沉 | ② → ③ | 任务后端失联句点名 current binding、remote revision、来源 head、readback、fail closed；引擎失联句写「不铸新义务」 | ② 只写两类结果：已接纳的判决还在，依赖新鲜回读的入口停。字段名和「铸造义务」是 `spec/task.md` / `spec/run.md` / `connections.md` 失败表的事。结构安排（不可用 vs 永久丢失分列）正确，不要拆掉 |
| 9 | `project.md:39`（Room 类型） | 下沉 | ② → ③ | 「只有回填动作成功才能归档」 | ② 写「临时讨论空间先说清回填什么，结案后归档」。归档前置的合法情形（含显式 abandoned 等）只在 `spec/project.md` 写一次。② 复述状态机会和约束层日后放宽不同步 |
| 10 | `project.md:54`（Chat Room 场景） | 下沉 | ② → ④ | P2、Matrix widget/AppService、消息事件 ID、尚未实现的结构化命令适配器 | ② 场景表只回答谁读写消息、谁提交治理命令。P2 路径和适配器缺口搬 `delivery.md` 实现阶段 |
| 11 | `task.md:21`（关键规则） | 下沉 | ② → ③ | 「生命周期只有开放、完成、取消三个状态」加泳道/健康标注的派生公式 | ② 写「承诺只有未完成、完成、取消；看板上的泳道是投影」。状态枚举和正交标注留 `spec/task.md` 写入约束表 |
| 12 | `task.md:49`（Kanban 场景） | 下沉 | ② → ④ 兼 ③ | P2 CLI 路径；Done 事件要有操作者、版本和幂等依据才能转完成请求 | ② 写「原生界面改卡片是场景内容；拖进完成栏可以请求同一个完成命令，转成请求不等于通过」。信封字段和 P2 入口留约束层与交付文档 |
| 13 | `run.md:37`（关键规则） | 下沉 | ② → ④ | 「Dagu 界面的直接 mutation」 | ② 用系统角色：工作流引擎的管理界面直接改机械执行，只记分歧。产品名留 `delivery.md`。用词「mutation」归语言通读，本条只报层 |
| 14 | `agent.md:24`（关键规则） | 下沉 | ② → ③ | 终局结果契约、观测截断收尾、子执行体派生谱系 | ② 留「进程退出不等于交付了结果；观测要么完整，要么标明截断」。适配器必须承诺什么事件，只在 `spec/agent.md` 运行时与观测 |
| 15 | `agent.md:46` 与 `:61`（Terminal / Agency 与 Herdr） | 下沉 | ② → ④ 兼 ③ | P2 的 `hctl2 terminal`；`` `hctl2-control` `` 适配代码与 `` `hctl2-tool` `` 现场清理；Herdr terminal ID | ② 写「Workbench 就位前，CLI 和官方 TUI 都是终端客户端」。组件名和票据字段留 `spec/system.md` / `spec/agent.md`；Herdr 缺项链 `delivery.md` P0 |
| 16 | `context.md:48`（萃取与压缩） | 下沉 | ② → ③ | 三级阶梯（结构化引用 / 本地全文检索 / 小模型门）、前缀缓存、tokenizer、选材计量 | ② 留「快、省、准」和投喂三档。阶梯、缓存和计量字段是 `spec/project.md` Context 段的机制。前缀缓存还掺了提供商实现细节，更不应停在架构层 |
| 17 | `participant.md:45`（专业化 Participant） | 上浮 | ② ← ① | 「商业化延长线」：雇佣外部数字员工、劳务市场形状、发现不等于信任 | ② 横切正文停在「评审岗位 = 身份 + 方法论技能包 + 独立性」。市场和雇佣叙事搬 `vision.md` 不解决什么 / 未决，或继续留 memo。现在这段是愿景展望写进了架构 |
| 18 | `spec/system.md:23`（固定内核与受控端口） | 上浮 | ③ ← ① | 「即使把全部界面、聊天平台、Task 来源、工作流引擎和终端客户端都换掉……」几乎逐字复述愿景最小内核段 | 改成引用：内核必须保留的性质见愿景该节。③ 从下一句端口表开始立约束 |
| 19 | `spec/system.md:73`（客户端动作与 provider 事件） | 上浮 | ③ ← ① / ② | 「Workbench 不比四个原生客户端加在一起更高贵，CLI 也不更低。」 | 口号归 `vision.md` 原则 14 或 `architecture.md` 展示面。③ 直接从动作分类表开始——表本身是本层该有的约束 |
| 20 | `spec/agent.md:86`（终端通道） | 上浮 | ③ ← ② | 「可以并存但不能互相冒充」套在 exact attach 等能力上 | ③ 写可测试句：宣称 exact attach 时必须证明同一进程；证不出则只能标 semantic resume / replay / 丢失。口号已在 `agent.md` Terminal 场景，这里不要再喊一遍 |
| 21 | `delivery.md:41`（明确不做） | 上浮 | ④ ← ① | 用户级「总入口对话面」不做，并论证「这是显式设计决定，不是待补功能」 | ④ 只列范围。论证已在来时路 §12，愿景「不解决什么」也可以收一句。交付文档不要自己再立产品为什么 |

未报但读过、结构合格的几处，避免汇总时当成遗漏：

- `run.md:35` 五种重试路径只列名字并链 `spec/run.md`：架构层该有的写法，与校准 A8 的结构判断一致。
- `design/README.md` 共同规则「以上是概括」+ 链连接约束 / 系统边界：名字加链接，不是把状态机抄进地图。
- `spec/README.md` 三条法：权威定义在 ③、`architecture.md` 已改成引用，不是上浮。
- `contract-tests.md` 用约束词指认被测行为：④ 允许，且文件头写明不立约。
- `docs/usage.md`：当前入口和安装步骤，停在 ④。

---

## 二、纵向聚类矩阵

八个起点簇都还在。补了三个：I 两条「三条底线」撞名，J 路标停更，K 客户端动作分类。每一簇先答三个问题，再标归并候选。

判据：归并之后，`contract-tests.md` 里有没有任何一条用例的判定结果可能变化。可能变化 → 会改变含义，本轮不落。

### A 代次与 fence（优先做细）

**是同一机制吗？** 同一家族：单调递增、旧值失权、不能共用一个名叫 `generation` 的槽。不是同一个计数器。六个成员管六块资源，少带一层就会让旧进程从另一扇门写进来。

**权威定义在哪？** 没有一张总表。成员散落如下：

| 成员 | 管哪块资源 | 权威落点 | 何时产生 |
| --- | --- | --- | --- |
| `attempt_generation`（Room Invocation 侧对应 `invocation_version`，它不是 generation，但是同一层的语义 owner 版本） | 这一次逻辑执行是谁 | `spec/run.md` Attempt 写入约束；`spec/project.md` Room Invocation | 派发时已有，不得预填运行时身份 |
| `runtime_generation` | 这一次物理进程/PTY | `spec/agent.md` Execution Runtime | 激活映射时由 Agency 预留返回 |
| `control_writer_generation` | 用户级账本此刻的逻辑写入者 | `spec/system.md` 单写者 | 取得账本写权时 CAS 推进 |
| `site_generation` | 某个 Repo Instance 的 Git / worktree 现场 | `spec/system.md` 单写者 | 同一本账对本现场 CAS 推进；现场 OS 锁是它的物理伴生，不是它 |
| Agency owner generation（文中也叫 backend generation） | 某个 Agency 绑定范围（同一 server/socket/host namespace） | `spec/system.md` 单写者 | 新 owner 对账后推进；旧 generation 不再签发输入/停止/结果准入 |
| `engine_binding_generation` | 某次 Run 与工作流引擎 execution 的绑定 | `spec/run.md` Engine Execution Binding | 启动/关闭/标分歧时推进。**不在** Agent 派发元组里 |

`spec/connections.md:99` 规定 Agent 出站必须同时携带：owner 版本/代次、`runtime_generation`、`control_writer_generation`、`site_generation`、Agency owner generation。`spec/agent.md:64` 再声明三层含义不可混写，替代任一层不得改写别层身份。`glossary.md:107` 有一句家族说明，但没把六个名字排齐，也没写 `engine_binding_generation`。

层级：

```text
语义 owner     attempt_generation | invocation_version
                 ↓ 派发时只有这一层，禁止预填 runtime
物理执行       runtime_generation          （激活时才有）
基础设施 fence  ┌ control_writer_generation  （账本写入者）
               ├ site_generation            （Git 现场）
               ├ Agency owner generation    （派出方绑定）
               └ engine_binding_generation  （引擎路标绑定，走 Run↔引擎，不走 Agent 出站）
```

不是代次的东西（不要并进来）：Participant revision、Binding revision、producer sequence、content cursor、Write Lease / Terminal Input Lease 的 lease generation（那是 B 簇）。

**哪些必须逐项携带，哪些不能从别的推导：**

- Agent 出站五元组必须逐项携带。任一旧值都不能用另一层的新值顶替——`spec/connections.md:109` 写明不能把一个合格项的代次套给另一个旧项。
- `runtime_generation` 不能从 `attempt_generation` 推导：派发时还没有物理身份；同一逻辑尝试换现场必须换物理代次。
- `site_generation` 不能从 `control_writer_generation` 推导：写入者可以搬家，现场钉在仓库实例上。这正是 v0.15.3 control 可搬、现场不可合并的物理前提。
- Agency owner generation 不能从 `site_generation` 推导：Git 锁管不了另一台机器上的 PTY。
- `engine_binding_generation` 不能从 `attempt_generation` 推导：引擎重试是路标再次进入等待，同一绑定上铸造新义务；Attempt 换代不必换引擎绑定，引擎绑定换代也不等于换了裁判席。
- `in_process` 受信任同步调用可以省略 runtime / site / backend / lease，但仍须带 owner 与 `control_writer_generation`。这是显式缩减，不是推导。

**其余位置是引用还是各写一遍？** `spec/agent.md:64`、`spec/agent.md:80`、`spec/connections.md:99–103`、`spec/run.md:79` 各写了一遍「这三层不是一回事」。`glossary.md:107` 写了半句。没有一处给出六行对照。

**归并候选：**

- **只改写法**：在 `spec/system.md` 单写者（或约束层总则六族旁）抽一张六行表：成员、资源、产生时机、必须出现在哪条出站、不能从谁推导。`spec/agent.md:64`、`connections.md:101`、`glossary.md:107` 改成引用加本路径必带子集。不合并计数器。CT-AGENT「错误 owner/generation 输入拒绝」、CT-SYSTEM「旧 generation 与越权适配器拒绝」判定不变。
- **会改变含义**：把任两层合成一个 `generation`，或让 engine 绑定代次跟 Attempt 代次同进同退。CT-RUN「Engine retry 铸造新义务、旧票作废」、CT-AGENT「迟到结果只留审计」会变。不在本轮落。

### B 锁与租约

**是同一机制吗？** 同族（Lease：有期限、单持有者、旧代次失权），锁的对象不同。

**权威定义：** Write Lease、Terminal Input Lease 在 `spec/agent.md` 对象表与写入约束；control 排他与现场 OS 锁在 `spec/system.md:167–169`（文中已声明锁路径是实现细节）；Agency owner lease 与 A 簇 Agency owner generation 是同一块资源的两种说法。

**其余位置：** `delivery.md` P1 复述现场 OS 锁；`glossary.md` Lease 族只有 Write / Terminal Input 两行，没写 Agency owner，也没写「control 锁不是约束」。

**归并候选：**

- **只改写法**：Lease 族总表补三行（ChangeSet 写权、终端输入权、Agency owner、并注明 control/site 的 OS 锁是实现，约束是三条底线 + generation）。各模块引用。
- **会改变含义**：用一把 OS 锁兼管账本、Git 现场和 PTY；或把 Write Lease 与 Terminal Input Lease 合成一个。CT-AGENT「ChangeSet 单 writer」「一个目标最多一个活跃输入者」会变。

### C 冻结与摘要

**是同一机制吗？** 同一算法家族（RFC 8785 JCS + SHA-256），语义槽不同。

**权威定义：** 算法与「`revision_digest` ≠ `review_subject_digest`」在 `spec/system.md:107`。Snapshot 冻结是 `spec/README.md` 三条法第二条。binding digest 在 Resolved Port Binding。Context `bundle_digest` / spec digest 在 `spec/project.md` 与 `spec/connections.md` Execution Spec。

**其余位置：** 各模块写入约束复述「以 digest 精确引用」。算法没有第二份定义，语义槽有多处展开。

**归并候选：**

- **只改写法**：系统边界保留算法 +「每个 owner 自列字段、两种 digest 不可互换」。模块只列本对象进哪一种 digest。
- **会改变含义**：统一 `revision_digest` 与 `review_subject_digest`。CT-RUN「subject digest 不匹配的旧票不计数、必须完整重评」会变。

### D 单写者与 CAS

**是同一机制吗？** 相关，不是同一个。单写者：同时谁有权写账本。CAS：命令带着预期版本，对不上就拒绝。current pointer：界面读哪个不可变版本。ChangeSet 单 writer 是 Agent 模块的写权，不是 control writer。

**权威定义：** 单写者三条底线 `spec/system.md:167`。命令信封的 expected version `spec/system.md:90`。current pointer 推进在各模块写入约束。ChangeSet 单 writer `spec/agent.md` 写入约束。

**归并候选：**

- **只改写法**：在系统边界用三句话分清「谁在写账本 / 这条命令对哪个版本生效 / 界面在看哪一版」，各模块引用。不要把 ChangeSet 写权并进 control 单写者。
- **会改变含义**：全库共用一个 expected-version 槽，或让 Git 现场锁充当账本单写者。CT-SYSTEM 备份恢复与 CT-AGENT 单 writer 会变。

### E 幂等与投递

**是同一机制吗？** 是：同一键返回原结果，ACK 未知不盲重做，外部写占用 `conflict_scope`。

**权威定义：** `spec/system.md`「命令与跨服务正确性」和「外部权威副作用」。`spec/connections.md:151` 已写「算法只由系统边界定义，本节只规定可观察结果」——这是已经做对的单源。

**其余位置：** `delivery.md` 切片 B 第 10 步用产品语言复述 generation / outbox / readback，作为验收路径可以留，但不要再写一套算法。

**归并候选：**

- **只改写法**：交付切片改成引用连接约束失败表的可观察结果。算法不要第三份。
- **会改变含义**：ACK 未知改成自动重做，或 `conflict_scope` 按 operation 而不是按远端资源划分。CT-CONNECTION 回读与 CT-TASK 重复 webhook 会变。

### F 失败与降级

**是同一机制吗？** 同一组分了三层讲：不可用 vs 永久丢失（②），按事实权威怎么降级（③ 系统地图），连接上能看见什么（③ 连接表）。应当分层，不应当各写一套同义句。

**权威定义：** 产品叙述 `architecture.md`「数据丢了怎么办」；事实地图 `spec/system.md` 全系统事实权威地图；连接结果 `spec/connections.md` 失败与恢复。引擎失联「不铸新义务」的铸造条件在 `spec/run.md:73`。

**其余位置：** 三处都写了引擎失联 / 聊天失联 / 任务后端失联。② 还下沉了字段名（见清单 #8）。

**归并候选：**

- **只改写法**：② 只保留两类产品结果；③ 系统地图按事实行；连接表按失败点行。铸造条件只在 `spec/run.md`。现在的分工意图是对的，缺的是引用而不是复述。
- **会改变含义**：把「结果未知」并进 fail closed，或让引擎失联改成拦住已铸义务的判决。CT-RUN「路标停更不铸新义务、已铸照常判决」、CT-PROJECT 聊天不可用 fail closed 会变。

### G 恢复与对账

**是同一机制吗？** 启动恢复七步是 F 的开工实例；drift / binding 分歧是对账的两种观测分类。

**权威定义：** 七步 `spec/system.md:173`。对账完成前不得授予新租约，同节第 7 步。drift 在 Task / Agent 约束；binding 分歧在 Engine Execution Binding 与 connections 失败表。

**归并候选：**

- **只改写法**：七步留系统边界（顺序本身就是约束，校准 B4 结构方向）。各模块只引用「对账完成前不新授写权」。
- **会改变含义**：允许对账未完成时签发新 Write Lease / 输入租约。CT-SYSTEM 恢复后不重复副作用会变。

### H 权限与来源

**是同一机制吗？** 相关的一组，不是一条规则。actor provenance 五值、`trust_level`、execution principal、单用户模型、桌面壳安全边界、Harness 三条底线，各管各的门。

**权威定义：** provenance 与两类治理入口 `spec/system.md:72–101`。`trust_level` 同文件扩展绑定。Harness 三条底线 `spec/agent.md:34`。桌面壳六条 `spec/system.md` 安全边界。

**其余位置：** `vision.md` 原则 14 复述客户端无等级；`architecture.md` 展示面复述无特权；`delivery.md` CLI 节复述「没有隐藏权限」。

**归并候选：**

- **只改写法**：动作分类表只在 `spec/system.md` 一张。①/② 用一句话 + 链接。Harness 三条底线仍只在 Agent 写入约束（见 I 簇，不要和单写者三条底线并表）。
- **会改变含义**：把 provenance 五值收成 human / 非 human 两档，或让 execution principal 能提交「完成 Task」。CT-PROJECT「service/bridge bot 不能取得 human provenance」、CT-TASK 完成两来源会变。

### I（新增）两条「三条底线」撞名

**是同一机制吗？** 不是。单写者三条底线（一个逻辑 writer、已确认副作用不重复、旧结果不覆盖新结果）在 `spec/system.md:167`。Harness 三条底线（工具不是人、合入钥匙不进工具、隔离工作树）在 `spec/agent.md:34`。上一轮大修把后者钉在 Agent 写入约束，前者是 v0.15.2 放宽锁路径之后留下的控制面底线。

**权威定义：** 两处各一，不要并。

**归并候选：**

- **只改写法**：改名区分，例如「账本单写者底线」与「Harness 治理底线」。总则或 glossary 各给一行对照，避免后文「三条底线」无法跳转。
- **会改变含义**：并成一张六条或三张混表。CT-SYSTEM 与 CT-AGENT 各自的不可关闭项会对错号。

### J（新增）路标停更

**是同一机制吗？** 是：引擎只报告机械进度，不铸造义务、不签发凭证、不拦住已铸义务的判决。

**权威定义：** 铸造条件 `spec/run.md:73`（只认 fresh 观察；分歧待对账、缓存/迟到/旧 cursor 不铸新义务）。可观察失败结果 `spec/connections.md` 失败表「Workflow Engine 不可用」行。产品叙述 `architecture.md:87`。

**其余位置：** `spec/system.md` 权威地图「路标停更只让绑定待对账」；`run.md` 设计正文「引擎拥有机械位置……」——后一句的用词归语言通读，结构上 ② 声明分责是对的。

**归并候选：**

- **只改写法**：② 只说「引擎失联不拦已经作出的判决」。铸造与迟到观察只在 `spec/run.md` + connections 失败表。系统地图改引用。
- **会改变含义**：失联期间允许按旧 cursor 铸新义务，或反过来让失联拦住已铸义务的 Gate。CT-RUN 路标停更用例会变。本轮不落。

### K（新增）客户端动作分类

**是同一机制吗？** 是：不按客户端产品分权，按动作落点和信封分类。

**权威定义：** `spec/system.md`「客户端动作与 provider 事件」五类表。

**其余位置：** `vision.md` 原则 14、`architecture.md` 展示面、根 README 图注、`delivery.md` P3 段、四个模块设计正文的场景角色表，各写一遍「没有等级」。

**归并候选：**

- **只改写法**：五类表只在系统边界。① 原则 14 留一句体验。② 展示面留一句「动作按目标约束处理」并链接。模块场景表只列谁可以点什么，不重写分类。
- **会改变含义**：把 Herdr TUI 输入重新打成 drift，或把 Dagu 管理动作重新收成普通 Run 入口。CT-AGENT 原生输入、CT-RUN Dagu 直接改写会变。v0.15.0 刚裁过，本轮只去复述。

---

## 三、四个层归属疑点

### 1. 四个模块设计正文与同名约束的边界是否稳定？

**建议：不稳定，按「产品规则 vs 状态机」收，不要按文件名对仗各写一遍。**

`project.md` / `task.md` / `run.md` / `agent.md` 的「为什么存在」「模块拥有什么」「场景角色表」停在 ②，是合格的。渗入 ③/④ 的是「关键规则」和「场景」后半：状态枚举（Task 三态、Scoped Room 归档前置）、适配器终局契约、P2 CLI 子命令、Dagu/Herdr 产品名。结果是设计正文看起来像约束层的白话副本，约束一放宽（例如归档允许显式结案）② 就会过期。

收法：② 只写可观察的产品规则，一律链接约束层；状态机五件套、字段、阶段代号、实现名不进设计正文。这是只改写法。不要把设计正文降成目录页——「为什么存在」和场景角色表删掉会回到愿景层对象名不够用、架构层组装说不清。

### 2. `design/README.md` 是否该拆？

**建议：拆文档纪律，保留地图和共同规则。**

现在一页三件事：索引、共同规则、文档纪律。前两件是 ②。文档纪律是写作协议（谁定义什么、引入门槛、审计两轮），和「四个模块怎么组装」不是一个读者问题；它自己也把文风推给 `WRITING-GUIDE.md`，却仍在设计地图里立十一硬边界。新读者读地图会被写作流程挡住；实现者改约束时又不一定想到来这里看纪律。

收法：文档纪律整节搬 `WRITING-GUIDE.md`（或独立 meta，不进四层产品文档）。本页只留对象关系图、场景客户端、共同规则（名字 + 链接，现有「以上是概括」保留）。只改写法。共同规则不要再拆进四个模块——那是上一轮已经收过的单源。

### 3. `contract-tests.md` 归 ④ 还是 ③？

**建议：维持 ④。**

文件头已经立约：不描述状态机、不新增约束，约束变更须先改 spec 再加用例。它引用约束词是在指认被测行为，符合 0.4「可引用、禁止重定义」。若改归 ③，读者会把失败用例当成第二份约束正文，和「一个事实一个权威」冲突，也会诱使只改 CT 不改 spec。

不要把 CT 族写进模块约束末尾。④ 的位置让实现者按族验收，而不必在实现内核时先读完测试清单。

### 4. 根 `README.md` 是 ① 的浓缩还是独立门面？

**建议：独立门面，不要当 ① 改。**

它同时做四件事：愿景一句（①）、三面图（②）、早期实现与打包现状（④）、三条阅读路径。0.4 把它放在 ①，所以会出现清单 #4 那种下沉——门面图塞进 SQLite 和 Dagu mutation，新读者为了读懂首页必须先懂阶段代号和组件名。

但若按「① 浓缩」把实现状态和阅读路径删掉，仓库入口会变成只有口号、找不到代码现状。门面的职责是指路，不是说服。

收法：路由表加一行「门面」（根 README，允许按段混层，但每一段必须能指回本层权威）；或在 ① 注明根 README 不是愿景副本。图瘦身见清单 #4。只改写法。不要把根 README 并进 `vision.md`。

---

## 对指南的异议

1. **§0.4 验收标准与允许词汇打架。** ① 读完「不需要知道任何对象名」，同时允许词汇是核心产品词——而核心产品词就是 Project / Task / Run / Obligation / Seat 这张对象名单。审计时若按验收标准，愿景里出现 Task 就算下沉；若按允许词汇，这些名字是本层合法用词。请定一条：① 允许核心产品词，验收标准改成「不需要知道约束层对象（Obligation 的铸造条件、generation 字段、锁路径）」；或 ① 只许日常语言，核心产品词从 ② 起才出现。未裁定前，本报告按后一种更严的验收来报愿景下沉（清单 #1–#3），但把核心产品词出现在原则和心智模型里视为合法，没有逐条报。

2. **S3 与设计地图「文档纪律」第一条重复且更严。** 纪律写「设计正文只用核心产品词」；0.4 给 ② 另开六个高频约束词。本报告按 0.4（六个词可在 ② 携中文对照）。若纪律不改，结构 PR 会和地图自己的纪律冲突。建议纪律那条改成一句加链接，词汇归属留约束层总则，怎么写归指南——`01-style-posture.md` 已有这条，这里只是结构审计的回声，不另起方案。

# hctl-dag 的底座：可集成的 SDK 与引擎调研

> 状态：调研 · 日期：2026-10-22<br>
> 类别：⑥ 机械后端与基础设施（跨候选归纳） · 证据编号：E-L2-DAG-BACKEND<br>
> 对象：替换 Dagu 的 hctl-dag 的底座候选：Rust 与 C/C++ 的框架和组件、Go/Python/Node 的持久执行库、独立进程的引擎；本库 main `86fd3666` 的 control store crate 与依赖树<br>
> 定位：Informative 研究备忘录。所有者 2026-10-22 定去掉 Dagu、自建 hctl-dag（与 control 解耦，只做机械推进，裁决仍在 control），并要求「不到万不得已，我不想我们自己写代码」，所以先查可集成的 SDK。本文给推荐，不定案，路线由所有者定。按 extern C 嵌入的 workflow/DAG 引擎另做补充调研，结果追加在文末。发布后正文不改，只在文末追加复核记录。<br>
> 相关：[工作流引擎：Dagu 与七个候选对照](./workflow-engines.md)、[Dagu SDK 记录](./sdk/dagu.md)

## 一、结论

**没有哪个现成的 SDK 或引擎能直接当 hctl-dag 用。** 不管走哪条路，图语义这一层都得自己写：

- 就绪计算：全部前驱完成才开工，包括失败边、分支、有界循环、有界扇出；
- 完成时检查代次：过期的完成要拒绝；
- 不可变版本，以及运行中追加节点、继承已完成的节点；
- 全局单调游标。

现成的东西能替我们省的，只有底下那层管道：持久状态、持久定时器、外部事件投递。这层管道 control 自己的 store crate 已经有了：单调事件序号（`events.sequence AUTOINCREMENT`）、幂等键（`commands.idempotency_key`）、比较后写入（`src/crates/store/src/schema.rs`、`store.rs`）。

更根本的一点：**hctl-dag 要的是持久状态机，不是持久执行（durable execution）。**

- Temporal、Restate、DBOS、duroxide、durabletask 解决的是「一段代码在崩溃后靠重放接着跑」。hctl-dag 里没有长时间运行的代码，每次状态变化都是一次 API 调用或一个到期定时器触发的一笔数据库事务。用这一类，就得先把状态机写成一段代码，再靠重放把状态算回来，平白多出一层确定性约束，核心逻辑一行也省不掉。
- 以图为模型的引擎（Elsa、Camunda、Flowable、LittleHorse）范式是对的。但它们要带 .NET 或 JVM，有的还要 Kafka 或 Elasticsearch；节点和边级的增量接口、全局游标也照样缺。

**推荐路线甲**：用 control 已有的 store crate 和树里已有的依赖拼装，不加任何第三方依赖。DBOS Transact Go 作备选。见第四、五节。

## 二、需求与口径

| 编号 | 需求 |
|---|---|
| R1 | 单机持久状态，本地存储 |
| R2 | 被动节点：由外部调用方完成，完成要幂等 |
| R3 | 完成时带期望代次，过期的拒绝 |
| R4 | 节点和边级接口：逐个查询，逐条增加，加边时防环 |
| R5 | 运行中追加节点和边：出新的不可变版本，继承已完成的节点 |
| R6 | 版本不可变，最好按内容寻址 |
| R7 | 结构语义：全部前驱完成即开工、失败边、条件分支、有界循环、有界扇出、持久定时器 |
| R8 | 可观察：带游标轮询，或带单调序号的变更流 |
| R9 | 不需要执行任意命令 |
| R10 | 体积与形态 |
| R11 | 许可 |
| R12 | 活跃度：2026 年有发布 |

所有者在调研中途四次放宽口径（2026-10-22）：
1. **许可**：可以找对方付费，BSL、源码可得或商业许可都行，前提是对方确实卖能覆盖我们内嵌、分发的授权。
2. **存储**：不一定用 SQLite，RocksDB 这类内嵌存储引擎也行。外部数据库（Postgres、Kafka、Elasticsearch）仍算重。
3. **嵌入方式**：C/C++ 库经 FFI 嵌进 Rust 也算。
4. **语言**：「Go可以，甚至python sdk、nodejs sdk，都可以」。能内嵌进 Rust 只算加分，要报的是离线包里得多带什么运行时。

## 三、候选一览

四路并行调研，事实都取自一手来源（仓库、官方文档、某个 tag 下的源码）。下表只列结论，原样证据见各路回报，第六节汇总了未核实的点。

### 3.1 Rust 与 C/C++

| 候选 | 形态 / 存储 | 许可 · 最新版 | 能做的 | 缺的 |
|---|---|---|---|---|
| duroxide（Microsoft） | 进程内库；SQLite（sqlx）或自写 Provider | MIT · crates.io 0.1.32（2026-10-12），自标 Preview | 代码写的 orchestration，靠重放恢复；持久定时器；`raise_event`；`continue_as_new_versioned` | 外部事件没有去重键（R2 部分）；没有代次检查（R3）；没有节点和边级接口（R4）。[README](https://github.com/microsoft/duroxide/blob/585c89c86f03956f799913cf0b98186452221e75/README.md) |
| acts | 进程内库；KvStore 可插（SQLite、sled、自写） | Apache-2.0 · v0.25.0（2026-09-17） | `irq` 节点停住等外部 `complete`；重复完成报 already completed；有失败捕获、分支、循环、超时 | 模型是块结构，表达不了跨分支的任意汇合；同 id 重新部署会覆盖旧模型（R6 不过）；运行中只能往当前 step 里加 act；自带 QuickJS。[irq](https://github.com/yaojianpin/acts/blob/0c9b9a31f08f376b96d57f48803119275860b969/doc/src-en/model/act/irq.md) |
| dagrs 0.9（HarnessRun） | 进程内库；追加写的 JSONL | MIT OR Apache-2.0 · 0.9.0（2026-10-04） | checkpoint 比较后提交；`events_after` 游标 | 面向 agent 循环，interrupt 有 TTL；仓库一年迁了几次家，现在的仓库 0 星 |
| apalis 1.0 + apalis-workflow | 进程内库 | MIT · 1.0.0（2026-10-12） | 用代码建图 | 没有外部完成原语（R2 不过），是任务队列 |
| graphrun | 进程内库；redb | MIT · 0.1.4（2026-09-26） | 发布时算摘要，同摘要可重发、内容不同则冲突；信号按事件 id 幂等 | 项目才一个月、0 星，代码由 agent 生成；没有 R5 |
| C/C++：Taskflow、oneTBB flow graph、sogou/workflow | 只在内存里 | MIT / Apache-2.0 | 图结构 | 都不持久化，也没有可用的 Rust 绑定；没找到持久化的 C/C++ DAG 引擎 |

### 3.2 Go、Python、Node 的库

| 候选 | 形态 / 存储 | 许可 · 最新版 | 能做的 | 缺的 |
|---|---|---|---|---|
| **DBOS Transact Go** | 进程内库；纯 Go 的 SQLite 驱动（不需要 cgo），或 Postgres | MIT · v1.7.0（2026-10-15） | `Send` 带幂等键；`Recv` 的超时是持久的；流可按偏移读；运行带应用版本标签；`Patch`；`ForkWorkflow` 从某一步分叉 | 没有节点和边级接口，要写解释器 workflow；代次检查自己写；SQLite 路径 v0.16.0（2026-06-01）才有，它自己的 SQLite CI 只跑 Ubuntu。[tests-sqlite.yml](https://github.com/dbos-inc/dbos-transact-golang/blob/v1.7.0/.github/workflows/tests-sqlite.yml) |
| DBOS Transact Python / TS | Python 默认 SQLite；TS 只能用 Postgres | MIT · 3.3.0 / v5.3 | 接口同 Go 版 | 要带 CPython 或 Node；TS 版 R1 不过 |
| dapr/durabletask-go | 进程内库；SQLite 或 Postgres | Apache-2.0 · v0.15.1（2026-10-19） | `WaitForSingleEvent` / `RaiseEvent`；`ContinueAsNew` 可保留未处理的事件；带名字的版本 | 事件不去重；历史读取没有游标；主要为 Dapr 自己维护，fork 只有 17 星 |
| microsoft/durabletask-go | 只连 Azure 云服务 | Apache-2.0 · v1.0.0-beta | — | v1.0.0-beta.1 删掉了内嵌的 SQLite 和 Postgres 后端 |
| go-workflows | 进程内库；SQLite、MySQL、Postgres、Redis | MIT · v1.4.2（2026-06-05） | 历史读取带游标 | 作者明说不做版本（R6）；仍是预发布 |
| LangGraph、Prefect、Inngest、Hatchet、Vercel Workflow、Effect、Mastra、SpiffWorkflow、bpmn-engine | — | — | — | 各有挡住的地方：SQLite 只用于本地实验，必须 Postgres，服务端是 SSPL，脚本走 `eval`，或者模块标着 unstable |

### 3.3 独立进程的引擎

| 候选 | 运行时 / 存储 | 许可 · 最新版 | 能做的 | 缺的 |
|---|---|---|---|---|
| Elsa 3.9 | .NET；SQLite（EF Core） | MIT · 3.9.0（2026-10-04） | 唯一一个 R3、R5、R6 都原生具备的：User Task 完成时带 `expectedRevision` 和 `operationId`，revision 不符或同一 `operationId` 带了不同内容都返回 409；可以改运行中的实例（Migrate 到新版本、再调度新节点） | 定义只能整份提交（R4 部分）；只有按实例的日志（R8）；alteration 计划默认存在内存里；User Tasks 2026-10-04 才首发。[rest-api.md](https://github.com/elsa-workflows/elsa-core/blob/3.9.0/specs/013-user-tasks/contracts/rest-api.md) |
| Temporal | Go 静态二进制 156 MB | MIT · server v1.32.1，Rust SDK v1.1.0 | 最成熟；task token 带 attempt，过期的会被拒 | SQLite 官方「only for development and testing」；图只能当数据放进解释器；单个 run 有事件数上限 |
| Restate | Rust 静态二进制 248 MB，内嵌 RocksDB | BSL 1.1 加附加使用授权 · v1.8.1（2026-10-21） | 单个二进制；awakeable、Virtual Object 每个 key 只有一个写者 | 没有公开的自托管或 OEM 付费授权（on-prem 页写着「tbd.」）；默认内存预算在 GiB 级；没有全局序号 |
| Camunda 8 | JVM、RocksDB，加二级存储 | Camunda License 1.0（生产要买）· 8.10.5 | job lease 能拒过期的 token；有 OEM 计划 | 太重（c8run 895 MB，至少 4 GB 内存）；user task 完成不查版本 |
| Flowable 8 | JVM；H2 | Apache-2.0 | 能往运行中实例注入节点、迁移实例 | 没有 R3、R8；新节点接不进已有的 join |
| Conductor、LittleHorse、Hatchet、Inngest、Kestra、DolphinScheduler、Maestro、Argo、Windmill、Golem、Dapr、Obelisk、Resonate | — | — | — | 卡在其中至少一样：要 JVM 或 Kafka 或 Postgres 或 Kubernetes；许可没有付费出口；运行中不能加节点；成熟度不够（Resonate v0.9.8 之后没有新版本，main 分支在重写） |
| beads（任务图） | Go 加 Dolt | MIT · v1.3.1（2026-09-30） | HTTP 接口能逐个增删改节点和边，并算出能开工的；写操作带 `expected_version`，冲突返回 409；事件日志的 `seq` 无间隙、严格递增，可按 `?since=` 读 | 没有持久执行：不调度、不重试、没有定时器。[events-journal.md](https://github.com/gastownhall/beads/blob/v1.3.1/docs/reference/events-journal.md) |

### 3.4 拼装用的组件

- **petgraph 0.8.3**（已在树里，现在是 prost-build 间接带进来的）：`acyclic::Acyclic` 加边时维护拓扑序，`try_add_edge` 遇环返回 `Cycle`。[源码](https://github.com/petgraph/petgraph/blob/petgraph%40v0.8.3/src/acyclic.rs#L35-L91)
- **rusqlite 0.40.2 与 control 的 store crate**（已在树里）：事件表、幂等键、比较后写入都是现成的。SQLite 保证 `AUTOINCREMENT` 单调且不复用，WAL 模式同一时刻只有一个写者，所以 `WHERE seq > :cursor` 不会漏掉已提交的行。
- **内嵌存储引擎**（redb、fjall、RocksDB、LMDB、libmdbx）：在 R1、R5、R6、R8 上都不比 SQLite 省事，没有 SQL 就得自己手写索引，反而多出 300–700 行。
- **honker 0.5.0**（SQLite 上的队列加流）、**SurrealDB**（BUSL，带变更流）、**turso**（自带数据变更捕获，仍是 1.0 之前）：能省一点，但分别是 alpha、许可和依赖太重、不成熟。

## 四、四条路线

| 路线 | 底座 | 要我们写的 | 新增依赖 / 运行时 | 主要风险 |
|---|---|---|---|---|
| **甲 · 自家拼装（Rust）** | control 的 store crate、rusqlite、petgraph `Acyclic`、axum、tokio，都已在树里 | 图模型与版本、就绪计算、带代次的完成、定时器、追加与继承、事件游标、HTTP 接口。第三路估从零拼约 1,400–2,500 行，测试同量；其中事件日志、幂等、比较后写入 store 已有，能再省一些（没量过） | 无 | 全是自己的代码，正确性靠测试 |
| 乙 · DBOS Transact Go | DBOS：持久消息、持久超时、流、版本 | 解释器 workflow、定义表、代次检查、HTTP 接口，规模与甲相当 | Buck2 里要加 Go 工具链（现在没有，随包的 Go 程序都是下载的成品）；多一个 Go 二进制 | SQLite 路径才 5 个月；重放有确定性约束；范式不对（持久执行） |
| 丙 · Elsa 3.9 | 原生 user task 带 revision 与幂等，能改运行中的实例 | 映射层、全局游标、alteration 的持久化配置 | .NET 运行时 | User Tasks 首发才 3 周；定义只能整份交 |
| 丁 · 独立进程当底座（Restate / Temporal / Resonate） | 持久执行服务 | 解释器、代次检查、版本、游标 | 248 MB 的 BSL 二进制，或者 SQLite 只给开发用的 Temporal，或者还不成熟的 Resonate | 范式不对，还多一个进程 |

## 五、推荐与要写的部分

**推荐甲。** 理由：

1. 核心那一层每条路都要写，各路线的差别只在底下的管道；
2. 这层管道我们自己已经有，而且测过：control 的 store crate 已在线上用；
3. 范式对：hctl-dag 是持久状态机；
4. 不加依赖、不加运行时、不加工具链。

**要写的**，按第三路的拆法（行数是判断，不是量出来的）：

| 部分 | 行数 |
|---|---|
| 表与迁移：版本、版本里的节点和边、执行、节点状态、完成、定时器、事件 | 100–200 SQL |
| 版本模型与内容寻址：规范 JSON 取 SHA-256，没有才插入 | 80–150 |
| 图检查与防环加边 | 80–150 |
| 追加与继承：用父版本加新增部分出新版本，比较后切换执行所在的版本，节点状态按节点 id 保留 | 150–250 |
| 就绪计算：全部前驱完成、成功/失败/带标签的边、跳过没走的分支、循环轮次、扇出槽位、终态判定 | 300–550 |
| 带代次的完成与幂等：同一个请求返回已完成，内容不同返回 409 | 100–180 |
| 持久定时器循环 | 80–150 |
| 事件日志、游标接口、长轮询 | 80–150 |
| HTTP 接口，约 10–14 个端点 | 300–550 |
| 接线：单写者、WAL、重启后重新挂定时器、退出 | 100–200 |

**可以直接借的设计**（复用决策：仅参考行为）：
- beads：写操作带 `expected_version`，冲突返回 409；无间隙的 `seq` 加 `?since=` 游标；就绪集用递归 CTE 加一个存储的 `is_blocked` 列算。
- graphrun：发布时算摘要，同摘要幂等，内容不同则冲突。
- Elsa：完成契约里 `expectedRevision` 和 `operationId` 分开校验，冲突分成 revision-conflict 和 idempotency-conflict 两种。
- dagrs：checkpoint 比较后提交，加 `events_after` 游标。

**备选乙的适用条件**：所有者更想要一个第三方的持久执行内核，并且接受 Buck2 加 Go 工具链、接受重放的确定性约束。

## 六、实现层脚注与未核实

**SQLite 版本冲突，不影响路线选择。** 同一个二进制只能链一份 SQLite：C 符号会重名，Cargo 的 `links = "sqlite3"` 规则也只允许一个版本。sqlx 系的 SQLite 包要求 `libsqlite3-sys` 低于 0.38，我们树里是 0.38.2，所以它们不能和 control 链进同一个二进制。hctl-dag 是独立进程，可以单独解析依赖。在 Buck2 里，这只是多一套第三方依赖（自己的 `Cargo.lock`），算构建上的维护成本，不是阻断。

**还没核实、但会影响决定的几点：**
- DBOS 的 SQLite 路径在 macOS 上崩溃后能否恢复（它自己的 CI 只跑 Ubuntu）；
- duroxide 能否用于生产（自标 Preview）；
- Elsa 的 SQLite 能否用于生产，alteration 能否持久化；
- Restate 有没有付费的自托管或 OEM 授权，本地分发给最终用户算不算在允许范围里；
- 路线甲的行数是判断，不是原型量出来的。

各路回报里还有一批更细的未核实项（各引擎重复完成的语义、二进制体积、许可细节等），定了路线后，只核与所选路线有关的那几项。

## 复核记录

（空）

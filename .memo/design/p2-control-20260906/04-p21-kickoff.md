# P2.1 开工书：底座（甲 → 乙 → 丁）——按 v0.18.11 重切

> 状态：v2 · 按 Codex（修正后可合）与 GLM（可合）的轻审修订 · 所有者 2026-09-19「开」<br>
> 基线：main @ `2c6b2e4`（草案 v0.18.11；G 批六个动手 PR 与收口 #270 已合）；A4 十行 CT 展开随后合入 `8f1ef8d`（#272）<br>
> 去向：`src/apps/hctl2-control`、`src/apps/hctl2`、`src/crates/*`；本文只重切工作包与任务书，不改约束层；`docs/design/delivery.md` 只在发现缺口时改<br>
> 读法：先读 [`01-plan.md`](./01-plan.md) §三（P2 的形状）、§四 B0 三包、§六 任务书要点、§十二 补记，再读本文 §二 的变化映射与 §四 的任务书；[`README.md`](./README.md) 状态板已指向本文<br>
> v2 与 v1 的差别（听了谁的哪条见 PR #271 上的作者说明）：§二 甲的对象键改成「按既有作用域」、A2 的落点补甲与 P2.5、Overview 补落点、Room 三态改准、E 批与 Room Invocation 行的落点拆开、补 #257 接手清单七行与 A4 一行；§四 甲补材料裸库依据、inbox、四个副作用窗口、材料故障窗口、六字段逐缺；乙改为仅归属者可访问的本地 Unix socket；丁改 CT 族、Workbench 留 P3、服务数据备份归丁；§五 同步；§六 第 3 项改为引用 09-04 钥匙串裁决

## 一、定位与重述

P2.1 是底座（旧 B0）：控制面存储与命令内核（甲）、进程与客户端边界（乙）、托管服务生命周期（丁）；丙（Repo Instance）已随 C 批撤销（01 §十二）。DoD 只有一行，`docs/design/delivery.md` §自举阶段 B0：**干净 clone 可启动；重启不丢状态；脚本只管进程和恢复。**

01-plan 写在 v0.17.0，§十二 补记到 v0.18.3。之后又落了五轮约束——E 批（v0.18.5）、S 批（v0.18.6）、体验澄清 #257（v0.18.7）、G 批（v0.18.8–v0.18.11）、#263 三条原则（随 #265）。本文回答三件事：这些变化各落到哪个工作包；P2.1 三包的任务书按 v0.18.11 写细；第一包给谁、怎么验。P2.2 以后的包只登记影响，不在本文写细。

六份前置研究已经齐了（§三），P2.1 不再等任何研究或探针。

## 二、v0.18.3 之后的变化，落到哪个包

| 变化 | 出处 | 落到 | 任务书里怎么改 |
| --- | --- | --- | --- |
| 控制面管理自己登记的 Repo 及各 Project；Project 是独立 Namespace，外部实体到 Task 的映射唯一性带 `project_id`；同 Repo 可多个 Project；Room 只有每 Project 一间主 Room 加零到多间 Topic Room，没有仓库级 Room；引用保留对象既有作用域 | #257（v0.18.7）；G 批 D1、D2、O6、D4（`spec/system.md` §固定内核与受控端口、`spec/connections.md` §连接模型、`spec/task.md` §契约与来源、`spec/project.md` §Repo 注册与 Project 归档） | 甲（对象键）；乙（Query 以 Project 为范围）；己、辛、庚（P2.2） | 甲按对象既有作用域建键：Project 内的工作携带 Project；Repo 事实、共享定义与绑定保持原作用域；外部实体到 Task 的映射唯一性含 `project_id`，不给外部实体本身另加 Project 身份；Room 类型只有主 Room 与 Topic；不建 Repo 级 Room；Repo 记录只承载登记与平台绑定 |
| 治理记录与治理正文两半、材料保存 / 准入 / 交付三阶段、一致备份集；备份集清单以 `spec/system.md` §备份与恢复 为唯一清单：治理记录快照、承诺可取的治理正文与定位关系、已进入承诺保存范围的 Context Bundle 原文、控制面自己的 Profile 定义、已冻结 Skill 引用与摘要及可核验性、公开冻结配置；不含 Agency 安装定义、密钥值、缓存、PTY 流与 content | C 批（v0.18.1）；G 批 Codex-1 | 甲 | 备份集清单只引那一节，不另抄；CT-SYSTEM 一致备份行的两种输入进甲的失败用例 |
| 判决权威在治理记录、审计副本在治理材料；公开范围不随代码仓库隐私推定；治理正文以 Git 为后端——每个控制面一份私有本地裸库，供它管理的多个 Repo / Project 共用，不注册成业务仓库、不依赖 Gitea | G 批 A3、C7（`spec/run.md` §写入约束、`spec/system.md` §Git 的双重角色）；`delivery.md` §技术基线、`docs/research/sdk/git.md` | 甲（材料存储）；丑（P2.4 审计公开） | 甲不再有「私有仓库全文 / 公开仓库摘要」的分支，只提供材料库与公开范围字段；01 §九 第一条延后项随之作废 |
| 安全策略面七个策略点，当前生效取值可经公共查询面读到：端点与连接缺省本机回环、非本地须认证；凭据存放；客户端最小权限；租户隔离按控制面 | S 批（v0.18.6，`spec/system.md` §安全策略面） | 乙（查询面暴露取值；本地 Unix socket 仅归属者可访问）；甲（凭据只存引用）；丁（随包服务端口按回环缺省） | 乙加「策略面当前取值」查询；丁的 Process Compose 配置按回环缺省，不开非本地端口 |
| 「待你处理」投影：五种来源、按来源引用与原处理动作去重、处理后退出、别的客户端处理随之更新；Project Overview 保留为只读投影、不要求独立入口、不能替代主 Room 与待你处理；「需要关注」是对象标记，不等于「待你处理」 | #257；G 批 C4、E4、D8（`spec/project.md` §Repo 注册与 Project 归档 的两个锚点） | 辛（待处理投影与 Overview 只读投影）；乙（Query 合同预留）；P3（双入口与导航） | 乙的 Query 合同给按 Project 的待处理列表与 Overview 只读投影留位（接口细节 for agent），P2.2 实现；保留 Overview 不等于增加必选入口 |
| 「发布评审须人显式确认」开关的缺省归 Project 版本化设置，随本次授权冻结进 Execution Spec；不做双层覆盖 | G 批 A2、Codex-6（`spec/project.md` §Repo 注册与 Project 归档、§Room Invocation；`spec/repo.md` §发布评审；CT-REPO / CT-CONNECTION 配对） | 甲（Project 记录的版本化设置字段）；辛（更新设置）；癸（Invocation 冻结）；丑（消费策略）；P2.5（Run Manifest 与后续 Attempt 沿用） | 甲的 Project 记录含版本化设置字段，其中有这个开关的缺省；Repo 记录不建开关 |
| Topic Room 命令词「创建 / 关闭」，关闭即已归档；Project 归档拒绝清单收成一处；恢复后随归档转只读的开放对象恢复接收命令、已终态不复活、已关闭 Topic 不随恢复 | G 批 C1、C2、C3、Grok-3 | 辛（P2.2）；甲（状态字段） | Room 对外状态仍是 `spec/project.md` §写入约束 的三态活跃 / 只读 / 已归档：关闭 Topic 即已归档，随 Project 归档则只读，不加第四态；怎样持久化由甲决定；已关闭的 Topic 与随 Project 转只读的 Topic 在 Project 恢复后分别保持关闭、恢复可写 |
| 本地 detach 默认另建独立副本、不换绑原 Repo 身份（Q2）；已有工作默认「取消并归档」，删源卡是另一个确认、预览列本控制面全部绑定 Task（Q3） | #257 | 戊、庚、寅（P2.2、P2.4） | 只登记 |
| 依赖归源投影四字段、只有阻塞进启动预览；闲置提醒 14 天只对承接开放 Request 的 Topic；预算耗尽后不再派；Task 与 Run 一对多、Run 与评审请求一对一 | E 批（v0.18.5） | 庚（依赖投影）；辛（Topic 闲置）；P2.5（预算耗尽是 Run 调度规则，不落成无 Run 调用的自动调度）；丑 / P2.5（Run 与评审请求一对一） | 只登记 |
| 施工图可由讨论凝结或从模板填表而来；读回在确无来源 Room 时只豁免名册回避；计划关联由 Run 模块随登记保存 | G 批 A5；#257 | P2.5 | 只登记；模板或直接文本的形式不免检，仍按真实来源判 |
| Attempt：Agency 报无法履约后才按冻结规则重试，控制面不凭内部故障换 Attempt；Agency 只经公开端口返回派工引用 | G 批 Codex-3、Codex-2（CT-PARTICIPANT） | 壬（P2.3） | 只登记 |
| Room Invocation 采纳契约与 Run 绑 Task 都核同 Project | G 批 GLM-4、D3（`spec/connections.md` §Project → Task：从讨论到承诺、§Project / Task → Run） | 庚（Task 命令准入核目标归属）；癸（交付调用的冻结来源）；Run 侧 P2.5 | 只登记 |
| #263 三条原则：入口原则进愿景；组织层可观测（来源链视图）；「形成可执行承诺的成本」进自举总账 | #263、#265 | P3；卯 / A1 的计量 | 只登记；归纳机制单独出方案，它不代替接手清单的三项实现设计（见下） |
| #257 接手清单：内容归属与引用——固定 Room / Task / Run 归属、消息不换 Room、Task 不换 Source、引用不迁移授权 | `open-questions.md` §规范对齐清单 | 己、辛（Room）；庚（Task）；P2.5（Run）；通用引用字段与作用域由甲承接 | 甲只提供引用字段与作用域，不实现业务命令 |
| #257 接手清单：前情提要——两种创建来源（主 Room 消息；Request 及其阻塞对象）与人的确认；首轮必用材料交付；自动选材与生成方式仍是实现设计题，人工替代路径不能丢 | `04-project-navigation.md` §Rooms；`open-questions.md` §旧讨论怎样接手 | 己、辛（创建与确认）；子（材料交付） | 只登记 |
| #257 接手清单：主 Room 与 Topic Room 均按本次授权调用；每个 Room 独立选人、每个 Run 独立选人；Project 只持选人策略；推荐与预填不继承授权，TAMP 延长线未实现 | `open-questions.md` §规范对齐清单「主 Room 的调用范围」「Participant 选择」 | 癸（调用）；己、辛（Room 选人）；P2.5（Run 选人） | 只登记 |
| #257 接手清单：多 Source——Project 显式接入仓库已绑定的 Source、保存源引用、新建 Task 明确目标源、同源卡各 Project 独立观测与验收；按源导航在 P3；汇总投影可选 | `open-questions.md` §规范对齐清单「多 Source」 | 庚、辛（P2.2）；P3 | 只登记 |
| #257 接手清单：Run 关联——保存计划归 P2.5；引用不承担交付；执行 / 检查 / 评审 / 集成 / Task 验收分别投影是各模块职责；列表、DAG、任务书与 Worker 侧栏在 P3 | `open-questions.md` §规范对齐清单「Run 关联与导航」 | 庚、丑、寅、P2.5；P3 | 只登记，数据与命令行为不推到 P3 |
| #257 接手清单：持续建议触发与费用控制、提要的选材与生成、图形观察怎样交付——三项实现设计另案，非 P2.1；观察能力由壬承接、界面由 P3 承接 | `open-questions.md` §旧讨论怎样接手 | 另案；壬；P3 | 只登记 |
| #257 接手清单其余行：Q2、Q3、Topic 关闭、待人处理已有落点；C1 / C2 的四 Project 事实作为后续验收输入保留；愿景旅程、术语与验证、决策史与版本的文档同步已完成 | `open-questions.md` | — | 已同步；行为随对应模块及 P3 验，不为它们新建实现包 |
| A4：十行 CT 短行已展开（#272，已合入 `8f1ef8d`） | `contract-tests.md` CT-PROJECT / CT-TASK / CT-PARTICIPANT / CT-SYSTEM | 甲（命令幂等、commit / 确认回执各崩溃点回读、schema migration 与投影重建）；丁（一键启停；打包整窗行只取 control、CLI 与已消费服务的部分，Workbench 部分留 P3） | 只取各包适用的行，不把十行全当 P2.1 DoD |
| 公共 CLI 命令表本轮未改；G 批后需要的入口：`task cancel`（已有）、`project update` 的设置字段、待处理查询 | `delivery.md` §公共 CLI | 乙 | 接口细节 for agent，在乙的合同里加，不改交付文档 |
| P2.2 以后另登记（GLM 轻审补）：汇总投影口径（G 批 C11）、后端离线不放宽当前回读与版本比较前置（C9）、缺省任务源在注册仓库或该仓库第一个 Project 接入时选定（C10）、集成事实可接受本控制面另一 Project 完成（D6）、获准映射自动认领与同卡多 Project 各自认领（E2、Grok-2）、默认 Source 预填与历史列表折叠 | G 批方案 §三；`open-questions.md` §旧讨论怎样接手 | 戊、庚、辛、癸、寅（P2.2–P2.4） | 只登记，进 P2.2 开工书；D9「连接命令同事务」已在甲范围 |

01-plan §五第 5 项「评审发布策略的开关缺省关」仍成立，只是持有者从仓库改成 Project：我们自己的仓库上每个 Project 都用缺省关。

## 三、前置研究与探针状态

| 项 | 状态 | 决定 |
| --- | --- | --- |
| 0a Protobuf 接口生成链与本地传输（`docs/research/libs/protobuf-rpc.md`） | 09-06 落地，#188 复核 | prost / prost-build 0.14.4、tonic 0.14.6、pbjson 0.9.0；protoc 36.1 由 DotSlash 钉三平台摘要；传输 tonic gRPC over Unix socket，不用 TCP；Buck 原生 action 生成独立 crate；Git 领域正文仍 JCS |
| 0b SQLite schema 迁移（`docs/research/libs/sqlite-migrations.md`） | 09-06 落地 | rusqlite_migration 2.6.0（MSRV 1.95）；`user_version` 管序号；启动取单写者锁后先 Online Backup 再 `to_latest`，失败整体回退，身份不变 |
| 0c Process Compose 调用面（`docs/research/runtime/process-compose.md`） | 09-06 落地 | 维持 1.122.0；用 CLI 的 JSON 状态与组件动作；实例不存在才 detached 启动，control 重启后重连 |
| 治理正文的 Git 后端（`docs/research/sdk/git.md`；`delivery.md` §技术基线） | 已定 | 每个控制面一份私有本地裸库，沿用既有 Git 调用与校验，不另做存储协议 |
| 可复用机制（`docs/research/libs/sqlite-online-backup.md`、`fd-lock.md`、`serde-jcs.md`、`keyring.md`；`src/crates/hctl2-foundation`） | 已进 foundation | 一致备份、文件锁、JCS、密钥引用；Linux 密钥后端所有者 09-04 已裁（§六） |
| 1a Matrix 与 Vikunja 复核（`docs/research/sdk/matrix.md`、`sdk/vikunja.md` §复核记录） | 09-06 钉定源码核对（不是联调） | P2.2 前置已满足；Vikunja 加绑推到 B2 之后 |
| 2a 三家 harness 接入面（`docs/research/harness-adapters.md`） | 09-06 | 三家共用一个适配器骨架；双向审批路径未验收，不提前激活 |
| 2b `gh` 写侧（`docs/research/sdk/github.md` 09-17 复核记录） | 09-17 私有沙箱跑通 | P2.4 前置已满足 |
| Gitea 本地平台（`docs/research/gitea.md` 09-17 复核记录） | 09-17 本机跑通 | 结构化写走 `tea api`；`ALLOWED_HOST_LIST = loopback` |
| 探针（`delivery.md` §开工前限时验证） | chat 探针 B1 前、Agency 探针 B2 前、本地任务服务器加绑前、Dagu B4 前 | P2.1 不依赖任何探针；chat 探针在 P2.2 己开工前做，Agency 探针在 P2.3 壬开工前做 |

## 四、P2.1 三包任务书（v0.18.11）

分工沿 01 §四：Codex 与 Grok 主写，Fable 与 GLM 主审；每包一个 PR，两份独立评审到齐、修正项改完后由原作者合。甲先做必要的存储结构与键，不等于实现「注册 Repo / 创建 Room / 采纳 Task」，这些命令留 P2.2；甲的事务与恢复测试用库级夹具，乙的四类公共操作先用本阶段的运维入口串联，不为验底座提前补业务流程。crate 名、SQL 布局、内部接口不需要所有者逐项裁。

### 甲 · 控制面存储与命令内核（Codex）

- **目标**：一本用户级控制面存储加一套命令内核，让乙、丁和 P2.2 的业务命令都长在它上面。
- **范围**：schema 与迁移；命令信封六字段与幂等；领域事件、幂等结果、outbox 与 inbox 记录同一事务；投影可从事件重建；control writer 锁与代次；两半存储——治理记录 SQLite，治理正文与审计副本的材料库是每个控制面一份私有本地 Git 裸库，沿用现成 Git 调用，不注册为业务 Repo、不依赖 Gitea；一致备份集与恢复（清单只引 `spec/system.md` §备份与恢复，含已进入承诺保存范围的 Context Bundle 原文）；对象键按 §二 第一行（按既有作用域）；Room 三态与 Project 版本化设置字段。**不做**：任何适配器（inbox 的供应端收取与翻译留后续适配包）、Repo 注册、Room / Task 业务命令、gRPC 与守护进程（乙）、服务生命周期（丁）。
- **依据**：`spec/system.md` §命令与跨服务正确性、§单写者、§控制面自己的存储、§外部权威副作用、§启动与恢复、§备份与恢复、§安全策略面「凭据」；`spec/connections.md` §连接模型；`spec/project.md`、`spec/task.md` 的对象表与键；`delivery.md` §技术基线、§打包策略；研究 0b、`docs/research/sdk/git.md`、`libs/sqlite-online-backup.md`、`libs/fd-lock.md`、`libs/serde-jcs.md`、`libs/keyring.md`；foundation 已封装的锁、JCS、Online Backup、SecretStore（系统钥匙串优先，退用户目录 0600 文件）。
- **失败用例（必须有测试）**：第二个 writer 拒绝；恶意重放旧代次拒绝；命令信封六字段逐个缺失各拒绝，同一幂等键异载荷拒绝、同载荷返回原结果；领域结果与已确认副作用不重复、未确认动作按原意图回读不盲重做——四个窗口分别测：提交前崩溃无半条记录，提交后未投递不丢动作，外部成功但确认丢失保持未知并回读，重试只得一次效果（效果次数不是发送次数；用测试替身，不接适配器）；材料保存 / 准入的故障窗口：保存后未准入崩溃、准入后响应丢失、记录在而字节或定位丢失、保护期候选被回收；inbox 重复输入经同一记录键不形成第二份领域效果；删掉投影表重建后一致；迁移失败回退到快照且身份不变；旧 schema 被新版本打开先升级再服务、升级中返回类型化拒绝（乙呈现）；备份只缺 Agency 安装定义不判不完整、缺已承诺保存的 Bundle 字节或控制面自己的 Profile 定义判失败。对应 CT-SYSTEM 现行各行，含 #272 展开的「命令幂等」「commit/确认回执各崩溃点回读」「schema migration、投影重建」。
- **验收**：上列失败用例全部有测试；三平台 CI 绿；B0 DoD 的「重启不丢状态」由丁端到端验，甲提供恢复入口。

### 乙 · 进程与客户端边界（Grok）

- **目标**：`hctl2-control` 守护进程与公共 `hctl2` CLI 的骨架，四类操作各一条端到端。
- **范围**：`.proto` 合同与 Buck 生成链（0a 的决定）；tonic gRPC over 仅归属者可访问的本地 Unix socket，本阶段不开放非本地传输（这是安全策略面「端点与连接」的当前缺省，非本地须认证是后续取值，不是永久禁令）；版本协商（client 版本不匹配拒绝）；Query / Preview / Submit / Subscribe 四类入口，Subscribe 带序号、断线重连给重同步快照；危险动作未经 Preview 的直接 Submit 拒绝、普通命令直接 Submit 与经 Preview 一致；运维命令组 `hctl2 init/start/status/doctor/export/backup create|verify/restore preview|apply`；`status` / `doctor` 暴露安全策略面当前取值与密钥后端来源；Query 合同给按 Project 的待处理列表与 Overview 只读投影留位；升级中甲给出的类型化拒绝由乙原样呈现。**不做**：业务命令的实现（P2.2 起）。
- **依据**：`spec/system.md` §场景端口、§客户端动作与 provider 事件、§安全策略面；`delivery.md` §公共 CLI；研究 0a。
- **失败用例**：套接字被占；其他 OS 用户不能访问该 socket，旧 socket 不误连；client 版本不匹配；事件游标过期；未经 Preview 的危险 Submit。
- **依赖**：甲（对象表与命令内核）；0a 已定，`.proto` 合同可与甲并行起草，字段从甲的对象表来。

### 丁 · 托管服务生命周期（Grok）

- **目标**：control 经 Process Compose 拉起、探活、按首次消费启停随包服务；B0 端到端收口。
- **范围**：`hctl2 start` 带起 Tuwunel 与 Gitea（Vikunja 推到本地任务服务器加绑前，01 §十二）；就绪探针过了才算可用；服务死活不改治理事实；端口按回环缺省；Tuwunel 与 Gitea 的数据备份恢复随它们的一键生命周期在首次消费前由丁交付，不拿甲的控制面备份顶替（控制面备份不含 content）；尚未消费的服务不成为控制面存储可用的前置；B0 端到端：干净 clone → `hctl2 init/start` → 杀 control 与服务 → 重启 → 存储与投影一致。
- **依据**：`spec/system.md` §组件、§启动与恢复、§端点与输入的信任边界；`delivery.md` §实现阶段、§本地 Agency 参考实现、§打包策略、§技术基线；研究 0c、`gitea.md`。
- **失败用例**：CT-SYSTEM「一键启停下已消费服务器的启动顺序与健康检查」与「打包后的整窗启动 / 退出 / 升级和安全边界」（#272 展开版；本阶段只验 control、CLI 与已消费服务，Workbench 部分留 P3）；探针未过就报可用；服务死了治理记录被改。
- **依赖**：乙、0c。

### 顺序与并行

甲先；乙的 `.proto` 合同与甲并行起草、实现等甲；丁最后收口。A4 的十行 CT 展开已合（#272），甲与丁只取各自适用的行作验收依据。

## 五、第一包任务书（给 Codex，直接贴）

```
你在 yesme/hctl2 做 P2.1 第一包「甲 · 控制面存储与命令内核」。开工书：main 上 .memo/design/p2-control-20260906/04-p21-kickoff.md（先读 §二 变化映射、§三，再读 §四 甲）；计划背景 01-plan.md §三、§六 甲、§十二。约束权威：docs/design/spec/system.md §命令与跨服务正确性、§单写者、§控制面自己的存储、§外部权威副作用、§启动与恢复、§备份与恢复、§安全策略面「凭据」；spec/connections.md §连接模型；spec/project.md、spec/task.md 的对象表与键；docs/design/delivery.md §技术基线、§打包策略。研究：docs/research/libs/sqlite-migrations.md（rusqlite_migration 2.6.0、停机迁移）、docs/research/sdk/git.md（治理正文的私有本地裸库）、docs/research/libs/sqlite-online-backup.md、fd-lock.md、serde-jcs.md、keyring.md；src/crates/hctl2-foundation 已封装锁、JCS、Online Backup 与 SecretStore（系统钥匙串优先，探测不到退用户目录 0600 文件——所有者 2026-09-04 裁，见 keyring.md §复核记录）。基线 main @ 2c6b2e4（v0.18.11），CT 十行展开已合 8f1ef8d。分支 codex/p21-store-kernel，base main，一个 PR；两份独立评审（Fable、GLM）到齐、修正项改完后由你自己合，合前报所有者。
交付：src/crates 下的存储与命令内核 crate（名字自定，target 切细，消费方只依赖它）：schema 与迁移（user_version 管序号；启动取单写者锁后先 Online Backup 再 to_latest，失败整体回退、身份不变；升级中对外返回类型化拒绝）；命令信封六字段与幂等（六字段逐个缺失各拒绝；同键异载荷拒绝、同载荷返回原结果）；领域事件、幂等结果、outbox 与 inbox 记录同一事务（inbox 只做持久记录与去重边界，供应端收取与翻译留后续适配包）；投影可从事件重建；control writer 锁与代次；两半存储——治理记录 SQLite，治理正文与审计副本的材料库是每个控制面一份私有本地 Git 裸库，沿用现成 Git 调用，不注册为业务 Repo、不依赖 Gitea；材料的保存 / 准入 / 交付三阶段；一致备份集与恢复（清单只引 spec/system §备份与恢复现文，含已进入承诺保存范围的 Context Bundle 原文；恢复只在旧写入者停止且取得排他锁后进行，推进代次）；对象键按既有作用域——Project 内的工作携带 Project，Repo 事实、共享定义与绑定保持原作用域，外部实体到 Task 的映射唯一性含 project_id，不给外部实体本身另加 Project 身份；Room 对外三态活跃 / 只读 / 已归档（关闭 Topic 即已归档，随 Project 归档则只读），持久化方式你定；Project 记录含版本化设置字段，其中有「发布评审须人显式确认」的缺省。不做：任何适配器、Repo 注册、Room/Task 业务命令、gRPC 与守护进程（乙）、服务生命周期（丁）。
失败用例（必须有测试）：第二个 writer 拒绝；恶意重放旧代次拒绝；六字段逐个缺失各拒绝；同一幂等键异载荷拒绝、同载荷返回原结果；领域结果与已确认副作用不重复、未确认动作按原意图回读不盲重做——四个窗口分别测：提交前崩溃无半条记录，提交后未投递不丢动作，外部成功但确认丢失保持未知并回读，重试只得一次效果（效果次数不是发送次数；用测试替身）；材料保存 / 准入的故障窗口：保存后未准入崩溃、准入后响应丢失、记录在而字节或定位丢失、保护期候选被回收；inbox 重复输入经同一记录键不形成第二份领域效果；删掉投影表重建后一致；迁移失败回退到快照且身份不变；旧 schema 被新版本打开先升级再服务、升级中返回类型化拒绝；备份只缺 Agency 安装定义不判不完整、缺已承诺保存的 Bundle 字节或控制面自己的 Profile 定义判失败。对应 docs/design/contract-tests.md CT-SYSTEM 各行（含「命令幂等」「commit/确认回执各崩溃点回读」「schema migration、投影重建」的展开版）。
规矩：Buck2 原生目标与 actions，不自建脚本旁路；三平台同一 rustc 1.98.0（src/rust-toolchain.toml）；新依赖先有 docs/research 对象文件（rusqlite_migration 已有，其他新依赖先补研究文件再写代码）；禁用「张力」「账本」（代码注释与文档写「控制面存储」「治理记录」），执行身份不可证叫「丢失」；PR 描述三节——定位与重述、当前生效约束集、业界方案调研（本包无新脚本，调研节引 sqlite-migrations、git.md 与 foundation 的研究文件，不以「不适用」开头）；提交信息与 PR 描述不放会话链接。CI 三平台全绿再回报：PR 编号、分支、crate 与 target 清单、失败用例清单、没按开工书做的地方及原因。
```

## 六、待所有者定

1. 席位：甲 Codex、乙与丁 Grok，审 Fable 与 GLM——沿 01 §四，所有者可换。
2. 乙的 `.proto` 合同是否与甲并行起草（建议是：0a 已定，字段从甲的对象表来，两家对一次即可）。
3. Linux 无桌面会话的钥匙串来源**不待裁**：所有者 2026-09-04 已裁（`docs/research/libs/keyring.md` §复核记录）——系统钥匙串优先，探测不到退用户目录 0600 权限的文件，`hctl2 doctor` 报当前用的是哪一种，systemd 凭据只作可选加固；foundation 的 SecretStore 已有这两种后端。01 §九 的遗留状态按此作废。

## 七、轻审怎么审本文

三样：§二 的变化映射有没有漏（对照 G 批方案 §八 与 #257 接手清单，一条裁决一个落点）；§四 三包的范围与失败用例是否忠于约束原文、有没有把 P2.2 的活提前塞进 P2.1；§五 任务书能不能直接贴给 Codex（缺什么信息、哪句会被误读）。不审 P2 计划本身的对错——01 已拍板。

# P2.1 开工书：底座（甲 → 乙 → 丁）——按 v0.18.11 重切

> 状态：v1 · 待轻审（Codex、GLM）· 所有者 2026-09-19「开」<br>
> 基线：main @ `2c6b2e4`（草案 v0.18.11；G 批六个动手 PR 与收口 #270 已合）<br>
> 去向：`src/apps/hctl2-control`、`src/apps/hctl2`、`src/crates/*`；本文只重切工作包与任务书，不改约束层；`docs/design/delivery.md` 只在发现缺口时改<br>
> 读法：先读 [`01-plan.md`](./01-plan.md) §三（P2 的形状）、§四 B0 三包、§六 任务书要点、§十二 补记，再读本文 §二 的变化映射与 §四 的任务书；[`README.md`](./README.md) 状态板已指向本文

## 一、定位与重述

P2.1 是底座（旧 B0）：控制面存储与命令内核（甲）、进程与客户端边界（乙）、托管服务生命周期（丁）；丙（Repo Instance）已随 C 批撤销（01 §十二）。DoD 只有一行，`docs/design/delivery.md` §自举阶段 B0：**干净 clone 可启动；重启不丢状态；脚本只管进程和恢复。**

01-plan 写在 v0.17.0，§十二 补记到 v0.18.3。之后又落了五轮约束——E 批（v0.18.5）、S 批（v0.18.6）、体验澄清 #257（v0.18.7）、G 批（v0.18.8–v0.18.11）、#263 三条原则（随 #265）。本文回答三件事：这些变化各落到哪个工作包；P2.1 三包的任务书按 v0.18.11 写细；第一包给谁、怎么验。P2.2 以后的包只登记影响，不在本文写细。

六份前置研究已经齐了（§三），P2.1 不再等任何研究或探针。

## 二、v0.18.3 之后的变化，落到哪个包

| 变化 | 出处 | 落到 | 任务书里怎么改 |
| --- | --- | --- | --- |
| 控制面管理自己登记的 Repo 及各 Project；Project 是独立 Namespace，外部实体到 Task 的唯一性带 `project_id`；同 Repo 可多个 Project；Room 只有每 Project 一间主 Room 加零到多间 Topic Room，没有仓库级 Room | #257（v0.18.7）；G 批 D1、O6、D4（`spec/system.md` §固定内核与受控端口、`spec/task.md` §契约与来源、`spec/project.md` §Repo 注册与 Project 归档） | 甲（对象键）；乙（Query 以 Project 为范围）；己、辛、庚（P2.2） | 甲的对象表以 Project 为查询范围建键；Task 唯一键含 `project_id`；Room 类型只有主 Room 与 Topic；不建 Repo 级 Room；Repo 记录只承载登记与平台绑定 |
| 治理记录与治理正文两半、材料保存 / 准入 / 交付三阶段、一致备份集；备份集只含治理记录快照、承诺可取的治理正文与定位关系、控制面自己的 Profile 定义、已冻结 Skill 引用与摘要及可核验性、公开冻结配置，不含 Agency 安装定义、密钥值、缓存、PTY 流与 content | C 批（v0.18.1）；G 批 Codex-1（`spec/system.md` §备份与恢复） | 甲 | 备份集清单照 §备份与恢复现文；CT-SYSTEM 一致备份行的两种输入进甲的失败用例 |
| 判决权威在治理记录、审计副本在治理材料；公开范围不随代码仓库隐私推定 | G 批 A3、C7（`spec/run.md` §写入约束、`spec/system.md` §Git 的双重角色） | 甲（材料存储）；丑（P2.4 审计公开） | 甲不再有「私有仓库全文 / 公开仓库摘要」的分支，只提供材料库与公开范围字段；01 §九 第一条延后项随之作废 |
| 安全策略面七个策略点，当前生效取值可经公共查询面读到：端点与连接缺省本机回环、非本地须认证；凭据存放；客户端最小权限；租户隔离按控制面 | S 批（v0.18.6，`spec/system.md` §安全策略面） | 乙（查询面暴露取值；套接字只本机回环）；甲（凭据只存引用）；丁（随包服务端口按回环缺省） | 乙加「策略面当前取值」查询；丁的 Process Compose 配置按回环缺省，不开非本地端口 |
| 「待你处理」投影：五种来源、按来源引用与原处理动作去重、处理后退出、别的客户端处理随之更新；Project Overview 只读投影、不要求独立入口 | #257；G 批 C4、E4、D8（`spec/project.md` §Repo 注册与 Project 归档「待你处理」锚点） | 辛（P2.2 投影）；乙（Query 合同预留） | 乙的 Query 合同给按 Project 的待处理列表留位（`project show` 的一个视图或 `project pending`，接口细节 for agent），P2.2 实现 |
| 「发布评审须人显式确认」开关的缺省归 Project 版本化设置，随本次授权冻结进 Execution Spec；不做双层覆盖 | G 批 A2、Codex-6（`spec/project.md` §Repo 注册与 Project 归档、§Room Invocation；`spec/repo.md` §发布评审；CT-REPO / CT-CONNECTION 配对） | 辛（`project update` 字段）；癸、丑（P2.3、P2.4） | 甲的 Project 记录含版本化设置字段，其中有这个开关的缺省；Repo 记录不建开关 |
| Topic Room 命令词「创建 / 关闭」，关闭即已归档；Project 归档拒绝清单收成一处；恢复后随归档转只读的开放对象恢复接收命令、已终态不复活、已关闭 Topic 不随恢复 | G 批 C1、C2、C3、Grok-3 | 辛（P2.2） | 甲的 Room 状态只有活跃 / 已归档，「因 Project 归档转只读」是派生态，不加第四态 |
| 本地 detach 默认另建独立副本、不换绑原 Repo 身份（Q2）；已有工作默认「取消并归档」，删源卡是另一个确认、预览列本控制面全部绑定 Task（Q3） | #257 | 戊、庚、寅（P2.2、P2.4） | 只登记 |
| 依赖归源投影四字段、只有阻塞进启动预览；闲置提醒 14 天只对承接开放 Request 的 Topic；预算耗尽缺省；Task 与 Run 一对多、Run 与评审请求一对一 | E 批（v0.18.5） | 庚、辛、癸（P2.2、P2.3） | 只登记 |
| 施工图可由讨论凝结或从模板填表而来；读回在确无来源 Room 时只豁免名册回避；计划关联由 Run 模块随登记保存 | G 批 A5；#257 | P2.5 | 只登记 |
| Attempt：Agency 报无法履约后才按冻结规则重试，控制面不凭内部故障换 Attempt；Agency 只经公开端口返回派工引用 | G 批 Codex-3、Codex-2（CT-PARTICIPANT） | 壬（P2.3） | 只登记 |
| Room Invocation 采纳契约核同 Project；Run 绑 Task 核同 Project | G 批 GLM-4、D3 | 癸（P2.3）；P2.5 | 只登记 |
| #263 三条原则：入口原则进愿景；组织层可观测（来源链视图）；「形成可执行承诺的成本」进自举总账 | #263、#265 | P3；卯 / A1 的计量 | 只登记；归纳机制单独出方案 |
| 公共 CLI 命令表本轮未改；G 批后需要的入口：`task cancel`（已有）、`project update` 的设置字段、待处理查询 | `delivery.md` §公共 CLI | 乙 | 接口细节 for agent，在乙的合同里加，不改交付文档 |
| P2.2 以后另登记（GLM 轻审补）：汇总投影口径（G 批 C11）、后端离线不放宽当前回读与版本比较前置（C9）、缺省任务源在注册仓库或该仓库第一个 Project 接入时选定（C10）、集成事实可接受本控制面另一 Project 完成（D6）、获准映射自动认领与同卡多 Project 各自认领（E2、Grok-2）、接手清单三项实现设计与默认 Source 预填、历史列表折叠 | G 批方案 §三；`open-questions.md` §旧讨论怎样接手 | 戊、庚、辛、癸、寅（P2.2–P2.4） | 只登记，进 P2.2 开工书；D9「连接命令同事务」已在甲范围 |

01-plan §五第 5 项「评审发布策略的开关缺省关」仍成立，只是持有者从仓库改成 Project：我们自己的仓库上每个 Project 都用缺省关。

## 三、前置研究与探针状态

| 项 | 状态 | 决定 |
| --- | --- | --- |
| 0a Protobuf 接口生成链与本地传输（`docs/research/libs/protobuf-rpc.md`） | 09-06 落地，#188 复核 | prost / prost-build 0.14.4、tonic 0.14.6、pbjson 0.9.0；protoc 36.1 由 DotSlash 钉三平台摘要；传输 tonic gRPC over Unix socket；Buck 原生 action 生成独立 crate；Git 领域正文仍 JCS |
| 0b SQLite schema 迁移（`docs/research/libs/sqlite-migrations.md`） | 09-06 落地 | rusqlite_migration 2.6.0（MSRV 1.95）；`user_version` 管序号；启动取单写者锁后先 Online Backup 再 `to_latest`，失败整体回退，身份不变 |
| 0c Process Compose 调用面（`docs/research/runtime/process-compose.md`） | 09-06 落地 | 维持 1.122.0；用 CLI 的 JSON 状态与组件动作；实例不存在才 detached 启动，control 重启后重连 |
| 1a Matrix 与 Vikunja 复核（`docs/research/sdk/matrix.md`、`sdk/vikunja.md` §复核记录） | 09-06 钉定源码核对（不是联调） | P2.2 前置已满足；Vikunja 加绑推到 B2 之后 |
| 2a 三家 harness 接入面（`docs/research/harness-adapters.md`） | 09-06 | 三家共用一个适配器骨架；双向审批路径未验收，不提前激活 |
| 2b `gh` 写侧（`docs/research/sdk/github.md` 09-17 复核记录） | 09-17 私有沙箱跑通 | P2.4 前置已满足 |
| Gitea 本地平台（`docs/research/gitea.md` 09-17 复核记录） | 09-17 本机跑通 | 结构化写走 `tea api`；`ALLOWED_HOST_LIST = loopback` |
| 探针（`delivery.md` §开工前限时验证） | chat 探针 B1 前、Agency 探针 B2 前、本地任务服务器加绑前、Dagu B4 前 | P2.1 不依赖任何探针；chat 探针在 P2.2 己开工前做，Agency 探针在 P2.3 壬开工前做 |

## 四、P2.1 三包任务书（v0.18.11）

分工沿 01 §四：Codex 与 Grok 主写，Fable 与 GLM 主审；每包一个 PR，两份独立评审到齐、修正项改完后由原作者合。

### 甲 · 控制面存储与命令内核（Codex）

- **目标**：一本用户级控制面存储加一套命令内核，让乙、丁和 P2.2 的业务命令都长在它上面。
- **范围**：schema 与迁移；命令信封六字段与幂等；领域事件、幂等结果、outbox 同一事务；投影可从事件重建；control writer 锁与代次；两半存储（治理记录 SQLite；治理正文与审计副本的材料库）；一致备份集与恢复；对象键按 §二 第一行。**不做**：任何适配器、Repo 注册、Room / Task 业务命令、gRPC 与守护进程（乙）、服务生命周期（丁）。
- **依据**：`spec/system.md` §命令与跨服务正确性、§单写者、§控制面自己的存储、§备份与恢复；`spec/connections.md` §连接模型（引用字段）；`spec/project.md`、`spec/task.md` 的对象表与键；研究 0b、foundation 已封装的锁 / JCS / Online Backup / keyring。
- **失败用例（必须有测试）**：第二个 writer 拒绝；事务中途崩溃后 outbox 不重复投递；恶意重放旧代次拒绝；同一幂等键异载荷拒绝、同载荷返回原结果；删掉投影表重建后一致；迁移失败回退到快照且身份不变；旧 schema 被新版本打开先升级再服务、升级中返回类型化拒绝；备份只缺 Agency 安装定义不判不完整、缺已承诺保存的 Bundle 字节或控制面自己的 Profile 定义判失败。对应 CT-SYSTEM 现行各行，含 A4 展开的「命令幂等」「commit/确认回执各崩溃点回读」「schema migration、投影重建」。
- **验收**：上列失败用例全部有测试；三平台 CI 绿；B0 DoD 的「重启不丢状态」由丁端到端验，甲提供恢复入口。

### 乙 · 进程与客户端边界（Grok）

- **目标**：`hctl2-control` 守护进程与公共 `hctl2` CLI 的骨架，四类操作各一条端到端。
- **范围**：`.proto` 合同与 Buck 生成链（0a 的决定）；tonic gRPC over Unix socket，套接字只本机回环、非本地连接拒绝——这是安全策略面「端点与连接」的当前缺省，不是永久禁令，非本地须认证是后续取值；版本协商（client 版本不匹配拒绝）；Query / Preview / Submit / Subscribe 四类入口，Subscribe 带序号、断线重连给重同步快照；危险动作未经 Preview 的直接 Submit 拒绝、普通命令直接 Submit 与经 Preview 一致；运维命令组 `hctl2 init/start/status/doctor/export/backup create|verify/restore preview|apply`；`status` / `doctor` 暴露安全策略面当前取值；Query 合同给按 Project 的待处理列表留位。**不做**：业务命令的实现（P2.2 起）。
- **依据**：`spec/system.md` §场景端口、§客户端动作与 provider 事件、§安全策略面；`delivery.md` §公共 CLI；研究 0a。
- **失败用例**：套接字被占；client 版本不匹配；事件游标过期；非本机连接；未经 Preview 的危险 Submit。
- **依赖**：甲（对象表与命令内核）；0a 已定，`.proto` 合同可与甲并行起草，字段从甲的对象表来。

### 丁 · 托管服务生命周期（Grok）

- **目标**：control 经 Process Compose 拉起、探活、按首次消费启停随包服务；B0 端到端收口。
- **范围**：`hctl2 start` 带起 Tuwunel 与 Gitea（Vikunja 推到本地任务服务器加绑前，01 §十二）；就绪探针过了才算可用；服务死活不改治理事实；端口按回环缺省；B0 端到端：干净 clone → `hctl2 init/start` → 杀 control 与服务 → 重启 → 存储与投影一致。
- **依据**：`spec/system.md` §组件、§启动与恢复；`delivery.md` §本地 Agency 参考实现、§打包策略、§技术基线；研究 0c、`gitea.md`。
- **失败用例**：CT-SYSTEM「一键启停下已消费服务器的启动顺序与健康检查」、CT-PRODUCT「打包后的整窗启动 / 退出 / 升级和安全边界」（A4 展开版）；探针未过就报可用；服务死了治理记录被改。
- **依赖**：乙、0c。

### 顺序与并行

甲先；乙的 `.proto` 合同与甲并行起草、实现等甲；丁最后收口。A4 的十行 CT 展开（PR 待合）是甲与丁的验收依据，先于两包的 DoD 判定合入。

## 五、第一包任务书（给 Codex，直接贴）

```
你在 yesme/hctl2 做 P2.1 第一包「甲 · 控制面存储与命令内核」。开工书：main 上 .memo/design/p2-control-20260906/04-p21-kickoff.md（先读 §二 变化映射、§三，再读 §四 甲）；计划背景 01-plan.md §三、§六 甲、§十二。约束权威：docs/design/spec/system.md §命令与跨服务正确性、§单写者、§控制面自己的存储、§备份与恢复；spec/connections.md §连接模型；spec/project.md、spec/task.md 的对象表与键。研究：docs/research/libs/sqlite-migrations.md（rusqlite_migration 2.6.0、停机迁移）、docs/research/libs/ 下 foundation 已封装的锁、JCS、Online Backup、keyring。基线 main @ 2c6b2e4（v0.18.11）。分支 codex/p21-store-kernel，base main，一个 PR；两份独立评审（Fable、GLM）到齐、修正项改完后由你自己合，合前报所有者。
交付：src/crates 下的存储与命令内核 crate（名字自定，target 切细，消费方只依赖它）：schema 与迁移（user_version 管序号；启动取单写者锁后先 Online Backup 再 to_latest，失败整体回退、身份不变；升级中对外返回类型化拒绝）；命令信封六字段与幂等（同键异载荷拒绝、同载荷返回原结果）；领域事件、幂等结果、outbox 同一事务；投影可从事件重建；control writer 锁与代次；两半存储（治理记录 SQLite；治理正文与审计副本的材料库）；一致备份集与恢复（清单按 spec/system §备份与恢复现文：治理记录快照、承诺可取的治理正文与定位关系、控制面自己的 Profile 定义、已冻结 Skill 引用与摘要及可核验性、公开冻结配置；不含 Agency 安装定义、密钥值、缓存、PTY 流、content；恢复只在旧写入者停止且取得排他锁后进行，推进代次）；对象键以 Project 为查询范围（Task 唯一性带 project_id；Room 只有主 Room 与 Topic 两类；Project 记录含版本化设置字段，其中有「发布评审须人显式确认」的缺省）。不做：适配器、Repo 注册、Room/Task 业务命令、gRPC 与守护进程（乙）、服务生命周期（丁）。
失败用例（必须有测试）：第二个 writer 拒绝；事务中途崩溃后 outbox 不重复投递；恶意重放旧代次拒绝；同一幂等键异载荷拒绝；删掉投影表重建后一致；迁移失败回退到快照且身份不变；旧 schema 被新版本打开先升级再服务、升级中返回类型化拒绝；备份只缺 Agency 安装定义不判不完整、缺已承诺保存的 Bundle 字节或控制面自己的 Profile 定义判失败。对应 docs/design/contract-tests.md CT-SYSTEM 各行（含「命令幂等」「commit/确认回执各崩溃点回读」「schema migration、投影重建」的展开版，展开 PR 见 A4 小批）。
规矩：Buck2 原生目标与 actions，不自建脚本旁路；三平台同一 rustc 1.98.0（src/rust-toolchain.toml）；新依赖先有 docs/research 对象文件（rusqlite_migration 已有，其他新依赖先补研究文件再写代码）；禁用「张力」「账本」（代码注释与文档写「控制面存储」「治理记录」），执行身份不可证叫「丢失」；PR 描述三节——定位与重述、当前生效约束集、业界方案调研（本包无新脚本，调研节引 sqlite-migrations 与 foundation 的研究文件，不以「不适用」开头）；提交信息与 PR 描述不放会话链接。CI 三平台全绿再回报：PR 编号、分支、crate 与 target 清单、失败用例清单、没按开工书做的地方及原因。
```

## 六、待所有者定

1. 席位：甲 Codex、乙与丁 Grok，审 Fable 与 GLM——沿 01 §四，所有者可换。
2. 乙的 `.proto` 合同是否与甲并行起草（建议是：0a 已定，字段从甲的对象表来，两家对一次即可）。
3. Linux 无桌面会话的钥匙串来源（01 §九，`libs/keyring.md`）：甲首次写密钥引用时提出，本文不裁。

## 七、轻审怎么审本文

三样：§二 的变化映射有没有漏（对照 G 批方案 §八 与 #257 接手清单，一条裁决一个落点）；§四 三包的范围与失败用例是否忠于约束原文、有没有把 P2.2 的活提前塞进 P2.1；§五 任务书能不能直接贴给 Codex（缺什么信息、哪句会被误读）。不审 P2 计划本身的对错——01 已拍板。

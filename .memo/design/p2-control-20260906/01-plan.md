# P2 计划：接钥匙——control 与公共 CLI 从 B0 到 B2

> 状态：已拍板 · 所有者 2026-09-06 对 §五五项逐条表态（v2 已按此改）；细节 for agent<br>
> 基线：main @ `1d001ad`（草案 v0.17.0）<br>
> 去向：`src/apps/hctl2-control`、`src/apps/hctl2`（公共 CLI）、`src/crates/*` 的适配器与账本 crate、`docs/research/` 新增对象文件、`docs/design/delivery.md` 只在发现缺口时改；不改约束层

P1 收口（`.memo/design/p1-toolbox-20260904/`）与 v0.17.0 约束批（`.memo/design/scm-module-20260906/`，PR #186）之后，交付文档里的下一格是 P2。本文把 P2 拆成可派工的工作包：B0 与 B1 写细，B2 写到任务书级，B3 到 B5 只写入口与依赖，等 B2 真跑通再细化。读它之前先看 [`README.md`](./README.md) 状态板。

## 一、定位与重述

交付文档的施工顺序是 P0 探路 → P1 备装 → P2 接钥匙 → P3 装门面（`docs/design/delivery.md` §实现阶段）。P2 的定义是：`hctl2-control` 与覆盖 B0–B5 的公共 `hctl2` CLI 承载治理；各 content 系统在首次被使用时完成打包、备份恢复和一键生命周期；Matrix 与 Vikunja 的原生界面承担 content；Herdr TUI 按 Execution Spec 输入策略使用；Dagu 控制台只用于管理和诊断，到 B4 才是必需项。自举等级 B0 到 B5 全部发生在 P2 内部（§自举阶段）。

用一句人话说：P1 把"手"（工具箱）和"腿"（五个随包服务）备齐了，P2 要长出"脑子"——一本账、一套命令、一个常驻进程——然后用它管我们自己的开发。**第一次真正的自举是 B2**：从聊天室发起一次真实的代码改动，走完封存、发布评审、合入、完成凭证，人的预览只有两次。这份计划的靶心就是 B2；B0 和 B1 是到 B2 的必经路。

范围之外：Workbench（P3）、远程控制面与多主机（交付文档 §未决问题）、Windows、GitHub 之外的代码协作平台（研究层已定 GitHub 缺省、其余按需）、Dagu 与 Run 的实现（B4，只在本文留入口）。

三条硬边界从约束层来，本计划不重述、只引用：三条底线（`docs/design/spec/participant.md` §不可关闭的三条底线）；命令信封与单写者（`spec/system.md` §命令与跨服务正确性、§单写者）；五模块连接总表（`spec/connections.md`）。新立的 Repo 模块（`spec/repo.md`）是 B1 注册仓库与 B2 集成的归属者。

## 二、起点核对

代码树里已经有的（2026-09-06）：

| 东西 | 位置 | 状态 |
| --- | --- | --- |
| 工具箱六个子命令 + `wait` | `src/apps/hctl2-tool` | P1 收口，三平台打包后端到端绿；只经宿主 git，不读写账本 |
| 共享基础机制：标准库文件锁、JCS、SQLite Online Backup、keyring、FTS5 | `src/crates/hctl2-foundation` | 受测封装，尚无消费者；`rusqlite`（bundled + backup）、`serde_json_canonicalizer`、`keyring` 已在工作区依赖 |
| 闭集外部事实读取 | `src/crates/hctl2-facts` | 供 `wait` 与将来的准入检查共用；GitHub 事实经随包 `gh` |
| 五个随包服务的生命周期 | `src/packaging/dependencies`（Process Compose，`hctl2-services start/status/smoke/stop`） | 三平台验过拉起、就绪、重启、关停；Herdr 协议版本与 API 快照可回读 |
| 本地 Agency 参考实现 | `src/agency`（技能目录） | 只有技能目录；可用性申报与 control 适配器待建 |
| 供应端客户端层研究 | `docs/research/sdk/`：matrix（ruma，AppService 身份）、vikunja（progenitor 生成）、dagu（progenitor）、herdr（typify 生成）、github（gh）、git、linear | 决定已落，代码树里除 `gh` 外未接入任何一家 |
| 通用机制研究 | `docs/research/libs/`：六份 | 已进 foundation |
| control 接口与存储的过程稿 | `.memo/notes/control-api-schema-20260902.md`（RPC/schema 待 control 开工时选）、`.memo/design/control-storage-20260821.md`（五储对照，待拍板） | 本计划 §五第 1、2 项接住 |

代码树里没有的：`hctl2-control`、公共 `hctl2` CLI、账本 schema、任何一家适配器（Matrix、Vikunja、Herdr、harness、GitHub 写侧）、Context 组装器、Repo 注册的 Git 身份写入、长持现场锁与 `site_generation`。P1 备忘 §六 的延后清单里，B1 前要做的是不可变正文写入/回读与判决结晶副本、Repo 身份写入；B2 前要做的是 Skill 目录级摘要回读、Herdr 可用性申报、租约撤销保证写入者已停。

## 三、P2 的形状

**进程。** `hctl2-control` 是一个用户级常驻进程（`spec/system.md` §固定内核：用户级命令服务，对每个 Repo 保持独立语义范围）；公共 `hctl2` CLI 是它的第一个客户端，Workbench 是 P3 的第二个。control 不内嵌任何 content 系统（§三个面：执行面各系统是独立进程），只托管它们的生命周期。

**一本账。** `~/.hctl2/control.sqlite`（§控制面自己的存储）：唯一用户级 metadata 账本，单写者加 `control_writer_generation`，领域事件、幂等结果、outbox 在同一事务。`<repo>/.hctl2/` 是 Git 跟踪的不可变正文与判决审计影子；`<git-common-dir>/hctl2/` 只有锁与缓存。

**四类操作。** Query、Preview、Submit、Subscribe（§场景端口）。CLI 每条命令都落到这四类之一；Preview 与 Submit 分开是产品承诺（危险动作先预览），不是实现偏好。

**适配器都在 control 进程内。** 平台适配器、聊天端口、任务源端口、Agency 端口、harness 适配器都是 control 里的适配代码，各按 `docs/research/sdk/` 的决定接入，按所有者的四级顺序：随包官方命令行 > 官方 SDK > 从接口描述生成 > 手写。`gh`、Process Compose 走第一级；Matrix 走第二级（ruma）；Vikunja、Dagu、Herdr 走第三级（生成）。

**工具箱不变。** control 是它的调用方，意图字段从账本来，输出的 JSON 记录进账本作证据；P1 定下的接口不重塑，只在 B1 补两个原语（不可变正文写入/回读、结晶副本写入）、B2 补一个（Skill 目录级摘要）。

## 四、工作包、顺序与分工

分工沿用所有者 2026-09-04 的话：**Grok 与 Codex 主写代码，Fable 与 GLM 主审**；研究文件与写作归 Fable。每个工作包一个 PR，不自合，两份独立评审到齐、修正项改完后作者合；「推翻」回到本计划改任务书。

### B0 · 底座（干净 clone 可启动；重启不丢状态）

| 序 | 工作包 | 写 | 依赖 |
| --- | --- | --- | --- |
| 0a | 研究：Protobuf 作接口 schema 的生成链与本地传输（§五第 1 项的对象文件 `libs/protobuf-rpc.md`） | Fable | 无 |
| 0b | 研究：SQLite schema 迁移库（§五第 2 项，`libs/sqlite-migrations.md`） | Fable | 无 |
| 0c | 研究：Process Compose 作为 control 托管服务的调用面（CLI 与 REST 的结构化输出、健康、按组件启停） | Fable | 无 |
| 甲 | 账本与命令内核：schema 与迁移、命令信封与幂等、领域事件与投影、outbox/inbox、control writer 锁与代次、备份/恢复（用 foundation） | Codex | 0b |
| 乙 | 进程与客户端边界：`hctl2-control` 守护进程生命周期、`.proto` 合同与生成链进 Buck2、ProtoJSON 走本地套接字、Query/Preview/Submit/Subscribe 四类入口、`hctl2 init/start/status/doctor/export/backup/restore` | Grok | 0a、甲 |
| 丙 | Repo 模块的 B0 半边：Repo Instance 挂接（复用 `repo inspect`）、长持现场锁与 `site_generation`、`hctl2 repo instance attach/list/show/detach` | Codex | 甲 |
| 丁 | 托管服务生命周期：control 经 Process Compose 拉起、健康检查、按首次消费的组件启停；`hctl2 start` 带起 Tuwunel 与 Vikunja；B0 收口：干净 clone 启动、杀进程重启不丢状态的端到端 | Grok | 乙、0c |

甲是一切的地基，先出；乙丙并行都只依赖甲；丁收口。

### B1 · Project Room 与本地 Task 影子（重启可恢复；引用稳定；明确不切换事实）

| 序 | 工作包 | 写 | 依赖 |
| --- | --- | --- | --- |
| 1a | 研究复核：`sdk/matrix.md` 与 `sdk/vikunja.md` 按 B1 的实际调用面追加复核记录（AppService 注册方式、按事件 ID 读正文、房间加密回读；Vikunja 分组实体与条件写入） | Fable | 无 |
| 戊 | Repo 注册：「注册 Repo」命令、Git 身份写入 `<repo>/.hctl2/repo.toml` 并回读（§五第 3 项）、待确认 → 活跃、同一事务建 Repo Room 身份；工具箱补「不可变正文写入/回读」原语 | Codex | 丙 |
| 己 | 聊天端口与 Room：control 以 AppService 接 Tuwunel（ruma），建房、绑定、按事件 ID 读正文、加密状态回读；Room–Server Binding；`hctl2 room list/show`；「创建 Project」建 Project Room | Grok | 乙、1a |
| 庚 | 任务源端口与 Task 影子：Vikunja 生成客户端，一个 Repo 一个 Board、Project 是分组、Task 是卡片；Task 身份映射、Snapshot、content-first 认领、「采纳契约」写 Task Revision 正文进 Git（工具箱原语）；`hctl2 task create/update/adopt/move` | Codex | 戊、1a |
| 辛 | Project 与 Request：「创建/更新/归档/恢复 Project」、参与者授权、Request 创建/解决；`hctl2 project …`、`request …`；B1 收口：Room/Task/草稿重启可恢复、引用稳定、不切换事实的端到端 | Grok | 己、庚 |

### B2 · 无 Run 切片成为真实开发入口（第一次真正自举）

| 序 | 工作包 | 写 | 依赖 |
| --- | --- | --- | --- |
| 2a | 研究：三家 harness 适配器的接入面（Claude Code、Codex、Gemini CLI；钩子注入、结构化事件归一、`harness-hooks-20260903.md` 的钩子白名单入口落到工具箱子命令） | Fable | 无 |
| 2b | 研究复核：`sdk/github.md` 追加写侧调用面（`gh` 推分支、开/更新 PR、请求合并、写回评论；PR 描述三节；确认丢失后的回读） | Fable | 无 |
| 壬 | Agency 端口与 Herdr：typify 生成 Herdr 类型；名册与可用性申报（技能目录的 digest 由工具箱回读记 known）；按 Execution Spec 拉起执行体、预留 `runtime_generation`、Attach Descriptor、停止与退出回读；输入策略两种；harness 适配器骨架加 Claude Code、Codex、Gemini CLI 三家的钩子注入与事件归一 | Grok | 乙、2a |
| 癸 | Participant 与 Room Invocation：Participant/Worker Profile/Skill 申报对象；Trigger Preview（执行者、Context、权限、预算、评审发布策略）；「创建/取消/准入结果」调用命令；Execution Spec 冻结；Result Proposal 准入 | Codex | 辛、壬 |
| 子 | Context 组装器第一版：聊天史、任务后端评论线、平台评审评论线的萃取（无 small-brain，全本地）；三档投喂；根 Manifest 与 Bundle 冻结、交付摘要核对 | Codex | 己、庚 |
| 丑 | Repo 模块的 B2 半边：ChangeSet 与 Write Lease；封存准入与人的显式封存；集成意图两种授权形态、目标保护快照、`integrate` 的本地路径与 `gh` 的平台路径；发布评审（control 按冻结策略发出）；ChangeSet–Platform Binding；平台动作分类；`hctl2 changeset/review/integration …` | Grok | 戊、2b |
| 寅 | Task 完成与凭证：「完成 Task」命令逐项校验（mechanical 只认 Integration Receipt）、Task Completion Receipt、结晶副本写 Git；Vikunja Done 映射为同一命令的请求 | Codex | 庚、丑 |
| 卯 | B2 收口：CT-PRODUCT 两条路径先在试验仓库上各走通一次（纯本地目标；受保护的 GitHub `main`），至少两家 harness 各跑一遍，默认两次预览；harness 环境取不到 HCTL 交付的凭据；执行加固按声明生效；重启后账本、工作树、意图、凭证一致。拿 HCTL2 自己的仓库当真实开发入口是 B3 的起点，由所有者按质量决定何时开始 | Grok | 癸、子、丑、寅 |

B2 是所有者可以在 Trigger Preview 里第一次看到完整身份链的地方；卯的验收对照 `delivery.md` §自举阶段 B2 那一行，其中「第一次真正自举」按 §五第 4 项的解读：切片在真实仓库上走通算 B2，拿 HCTL2 自身当开发入口从 B3 起算。

### B3 到 B5 的入口

- **B3**（接管自身待办、并发 Invocation、Request、Receipt、冷启动恢复）：没有新组件，是 B2 的量与故障覆盖——连续至少 5 个真实变更、覆盖核心/界面/适配器、全程无手工改库。计划在 B2 收口后按暴露的缺口另写。
- **B4**（workflow engine、Run、Seat、独立 Gate）：Dagu 经 progenitor 生成客户端（`sdk/dagu.md`，GPL 规范文件的生成物是否算衍生作品留所有者裁）；Workflow Revision 编译、Run Manifest、Obligation/Seat/Attempt、Gate Receipt；Dagu 探针（`delivery.md` §开工前限时验证第 1 项）只须在 B4 前完成，不阻塞 B2。
- **B5**（候选切换、多票评审、regate、完整故障恢复）：完整治理切片（`delivery.md` §纵向切片 B）在真实变更上通过。

## 五、需要所有者一句话的取舍

按「方向、边界、取舍找 human；接口细节 for agent」列五项，每项附建议；不说就按建议走。

1. **CLI 与 control 之间用什么说话——所有者倾向直接上 Protobuf，省得将来再迁。** 我原建议 P2 先用 serde JSON、P3 再选 Protobuf；讨论后改为**现在就以 Protobuf 作 schema**，理由与边界如下。省下的不只是一次迁移：`.proto` 从第一天起就是唯一的接口合同，P3 的 TypeScript 一侧用官方生成器读同一份文件，不会出现「Rust 类型是权威、TS 手抄一份」的漂移；Protobuf 的字段编号纪律也逼着我们从 B0 起就按可演进的方式改接口。代价是多一条生成链（protoc 或纯 Rust 的 protox，加 prost/tonic 类 crate）要进 Buck2 action，这是 0a 研究要回答的第一件事。三条边界：其一，线上编码在 P2 用 ProtoJSON（Protobuf 官方定义的 JSON 映射）走本地 Unix 套接字，人能读、能用命令行工具调试，与 P3 Tauri 2 的 JSON IPC 同形；二进制编码与 gRPC/Connect 留到出现跨进程、多语言、跨版本兼容的需求时切换，切换不改 schema。其二，领域正文（Task Revision、Workflow Revision 等进 Git 的东西）继续用 RFC 8785 规范化 JSON 作事实源，`.proto` 只承载传输语义，不反向定义事实格式——这是 `.memo/notes/control-api-schema-20260902.md` 的原话。其三，不手写并行 DTO：Rust 类型只从 `.proto` 生成。对象文件 `docs/research/libs/protobuf-rpc.md` 先落，要回答：生成器选型（prost + pbjson 对 ProtoJSON；tonic/Connect 作将来选项）、protoc 二进制随 DotSlash 钉定还是用 protox 免二进制、Buck2 里的 proto 规则怎么接、TS 侧生成器（Protobuf-ES）与 Buf 的关系、许可证。
2. **账本 schema 迁移怎么做——所有者问业界最佳实践是什么。** 业界共识有四条：迁移只向前、按序编号、每条在一个事务里应用；当前版本记在库里而不是靠猜（SQLite 官方给的位置是 `PRAGMA user_version`，另有 `application_id` 标记这是谁的库；重量级框架则另建一张迁移历史表）；`ALTER TABLE` 在 SQLite 里能力有限，改列要按官方文档的「建新表、拷数据、删旧表、改名」十二步走；迁移前先做一致备份。Rust 生态里做这件事的库：`rusqlite_migration`（专为 rusqlite、用 `user_version`、迁移写成 SQL 字符串数组、带一致性自检）、`refinery`（多数据库、SQL 文件嵌入、历史表）、`sqlx migrate` 与 `diesel_migrations`（各绑自己的运行时或 ORM，我们不用）。**定**：按四级顺序取现成库，`rusqlite_migration` 首选——零额外运行时、与 foundation 里的 rusqlite 同栈；迁移在单写者锁内、Online Backup 一致快照之后应用。对象文件 `docs/research/libs/sqlite-migrations.md` 先落，钉版本与许可证，并核 `rusqlite_migration` 对 rusqlite 0.40 的兼容。
3. **Repo 的稳定身份写在 Git 哪里——所有者问「仓库的稳定身份文件」是什么。** 它是 HCTL 认出「这个克隆和那个克隆是同一个仓库」的依据。远端地址会换、目录名随便起、纯本地仓库没有远端、两个不同仓库的 HEAD 也可能碰巧相同，所以约束不让这些当身份，而是在注册时生成一个稳定的 Repo 身份，写进仓库内容里的一个受跟踪文件——存储拓扑定的位置是 `<repo>/.hctl2/repo.toml`，内容是 Repo 的稳定 ID 加共享配置的引用，账本记它的摘要。任何克隆都带着它，工具箱的 `repo inspect` 已经会读它、没有就报缺失（P1 甲）。P1 备忘 §六 留的题是：这个文件的第一次提交落在哪个分支、由谁提交。**定（所有者未表态，按建议）**：就是默认分支上的普通跟踪文件；注册命令让工具箱在一个 ChangeSet 工作树里写出它，人经正常路径把它合入（我们自己的仓库走 PR，纯本地仓库走 `integrate` 快进）；合入前 Repo 保持待确认，不接受 Project、Task、Run——与约束一致，也让第一次注册就是第一次走完集成路径。不另设 `refs/hctl2/identity` 之类的旁路 ref，因为普通克隆不会带上自定义 ref，身份就跟不走。
4. **B2 接哪些 harness——所有者：都行，你安排；可以接 Gemini；第一个接谁不代表自举，可以接好几家、写完再回头自举，刚做完的 HCTL2 未必拿得来写我们现在的内容。** 按这话改两处。其一，B2 的 harness 适配器不是「一家」而是「一组」：按 `docs/research/harness-hooks-20260903.md` §决定建议 的优先级，先接 Claude Code（钩子注入不落盘、退出码 2 硬拒、结构化事件与 Herdr 的 `--env` 透传都有文档依据），同批接 Codex（钩子要过信任关卡，自动化须预置信任或 `--dangerously-bypass-hook-trust`）与 Gemini CLI（系统 settings 路径变量，先实测）；三家共用一个适配器骨架，差别只在钩子注入方式与事件归一，2a 研究要把这三家的接入面一次写清。其二，**自举与 B2 的第一次走通解耦**：B2 的工作包先在一个试验仓库上把两条路径（纯本地目标、受保护的 GitHub `main`）走通并验收；拿 HCTL2 自己的仓库做真实开发入口是 B3「接管自身待办」的起点，什么时候开始由所有者按当时的质量判断，不由 B2 的时间表逼。`delivery.md` §自举阶段 B2 行写的「第一次真正自举」按这个解读，措辞是否要改留到 B2 收口时一并处理。
5. **评审发布策略的开关缺省值——所有者：按建议来。** 缺省关：默认路径就是两次预览，这是产品承诺；我们自己的仓库也用缺省值，让 B2 的验收覆盖的是产品默认而不是我们的特例。落点是 `delivery.md` §运行默认值 加一行，随丑（Repo 模块 B2）的 PR 一起改。

## 六、任务书要点

各工作包开工时由作者按 P1 备忘 §五的粒度写成任务书进 PR 描述；这里只钉每个包的边界与必须有的失败用例。共同约束沿用 P1：PR 三节按模板；新增依赖先有 `docs/research/` 对象文件；输出与错误码沿用工具箱的记录形状；「张力」写「冲突」；位置引用写「文件 §节名」。

- **甲（账本与命令内核）**：命令信封六个字段（`spec/system.md` §命令与跨服务正确性）缺一拒绝；同一幂等键异载荷拒绝、同载荷返回原结果；领域事件、幂等结果、outbox 同一事务；投影可从事件重建（删掉投影表重建后一致）；备份集含账本与引用的定义字节，恢复只在旧写入者停止且取得排他锁后进行、推进 `control_writer_generation`（§备份与恢复）。失败用例：第二个 writer 拒绝；事务中途崩溃后 outbox 不重复投递；恶意重放旧代次拒绝。
- **乙（进程与客户端边界）**：守护进程随 `hctl2 start` 拉起、`status/doctor` 可查；四类操作各一条端到端；Subscribe 带序号、断线重连给重同步快照；危险动作未经 Preview 的直接 Submit 拒绝、普通命令直接 Submit 与经 Preview 一致（CT-SYSTEM）。失败用例：套接字被占、client 版本不匹配、事件游标过期。
- **丙（Repo Instance 与现场锁）**：挂接经 `repo inspect` 无副作用读身份；相同公共目录重试返回原现场；长持锁与 `site_generation` 伴生，旧代次的工具箱动作被拒；移除只撤销新执行资格。失败用例：两个 control 动作争同一现场；锁文件不在本地文件系统。
- **丁（托管生命周期）**：control 经 Process Compose 按组件启停，就绪探针过了才算可用；服务死活不改治理事实（§组件）；B0 端到端：干净 clone → `hctl2 init/start` → 杀 control 与服务 → 重启 → 账本与投影一致。
- **戊（Repo 注册）**：待确认注册先记账再写 Git；结果未知按原身份与摘要恢复同一次注册；身份缺失或冲突要求用户处理；活跃前拒绝 Project/Task/Run。失败用例：两个 clone 同时注册同一身份；身份文件被手工改。
- **己（聊天端口）**：AppService 注册与虚拟用户；HCTL 自建房间不开加密、绑定前回读加密状态；治理引用只按事件 ID 冻结；chat server 不可用时依赖当前回读的命令拒绝。失败用例：房间事后被加密标需要关注；bridge bot 同形事件拒绝为 human 来源（CT-PROJECT）。
- **庚（任务源端口）**：唯一键 `(provider, account_stable_id, external_entity_kind, immutable_external_entity_id)`；Snapshot 先观测后采纳；content-first 只在卡片恰好归一个 Project 分组时认领；跨分组移动只标需要关注。失败用例：Done 事件缺 doer 映射只追加 Snapshot；同一实体并发命中只复用同一 Task。
- **辛（Project 与 Request）**：归档前置要能列出归 Repo 模块的活动租约与未决意图；Request 去重、过期不伪造答案。失败用例见 CT-PROJECT 相应行。
- **壬（Agency 端口）**：可用性申报的 Skill digest 由工具箱回读记 known、不一致不激活；退出与停止回读不足时只能报语义恢复或丢失；两种输入策略按声明能力如实记录。失败用例：Herdr 未声明栅栏回显却宣称逐次受租约管理（CT-PARTICIPANT）。
- **癸（Participant 与 Room Invocation）**：Trigger Preview 展示完整身份链与评审发布策略、写明授权的是发布不是合入；调用只有约束列出的合法边；执行身份无法证明进丢失、撤租约、提交停止与隔离 outbox。失败用例：模型 `@` 直接创建调用拒绝；迟到 Proposal 只留审计。
- **子（Context 组装器）**：三处讨论来源萃取全本地；指针只指 Git 对象与工作树路径；Bundle 记交付计量与 `bundle_digest`，派工前核对实际交付摘要；纪要与相关性门未配 small-brain 时不生成。失败用例：治理引用指向纪要拒绝；压缩条目缺回源指针拒绝交付。
- **丑（Repo 模块 B2）**：全部 CT-REPO 用例；两种授权形态成对用例；同目标待决互斥；人的显式封存；`gh` 写侧确认丢失后按关联键回读不重复建 PR；执行体工作树的环境不带 `GH_TOKEN`、`GIT_ASKPASS` 类变量且 worktree 配置不继承凭据助手（这是可选加固的第一项，按 Worker Profile 声明生效）。
- **寅（Task 完成）**：逐项核对判定者与校验等级；契约未要求集成的 Task 不要求 Integration Receipt；Receipt、生命周期事件、占用标记清除与写回 outbox 同一事务。
- **卯（B2 收口）**：两条路径都在试验仓库上跑真实的非文档代码改动，至少两家 harness；默认两次预览；重启一致；以 `delivery.md` §自举阶段 B2 行为验收依据（「自举」的解读见 §五第 4 项）。

## 七、审核方式与 DoD

- 每个 PR 由 Fable 与 GLM 各自独立审，结论以 PR 评论给出，逐条「维持 / 修正 / 推翻」，每条引用约束原文或本计划条款。审的重点：命令信封与幂等、单写者与代次、三条底线、账本语义不泄漏到适配器与工具箱、失败用例是否真的堵住、接口是否为下一级留对了位置。
- B0 的 DoD 是 `delivery.md` §自举阶段 B0 行：干净 clone 可启动；重启不丢状态；脚本只管进程和恢复。B1、B2 同理各取其行；不另造验收标准。
- 每一级收口由 Fable 更新本目录状态板、`src/README.md` 与研究层索引；决策史只在出现转向时加节（预计 B2 不加，除非 §五的取舍改变了约束）。

## 八、研究层先行清单（Fable）

按纪律三「新组件、新依赖先落 `docs/research/`」：0a Protobuf 生成链与本地传输、0b SQLite 迁移库、0c Process Compose 调用面是 B0 前置；1a Matrix 与 Vikunja 的 B1 调用面复核是 B1 前置；2a 三家 harness 适配器接入面、2b `gh` 写侧调用面是 B2 前置。已有研究结论有变的，逐份追加复核记录，不改正文。

## 九、延后与遗留

- 判决结晶副本写入 Git 的粒度（私有仓库全文、公开仓库可降为摘要）：B2 的寅首次消费时按仓库策略实现，本计划不定缺省。
- 重试缓存 ref 的清理（P1 备忘 §六）：丑实现意图终态后的显式清理。
- 参与者是否各有平台身份（`spec/repo.md` §平台动作与命令 留给交付文档）：B3 之前不做，共享账号映射不出多张票。
- 多 Task Run 的集成策略与 ChangeSet/PR 基数（`delivery.md` §未决问题）：B4 前定。
- Linux 无桌面会话的钥匙串持久来源（`libs/keyring.md` 待所有者拍板）：B0 甲首次写密钥引用时提出。
- 署名关卡：另开小 PR，任务书见 [`02-attribution-gate.md`](../scm-module-20260906/02-attribution-gate.md)。

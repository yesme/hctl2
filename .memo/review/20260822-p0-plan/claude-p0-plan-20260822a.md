# Claude 提案：P0 实施方案（2026-08-22）

> 状态：Informative 提案 · Claude Code / Fable 5 出品，供与其他 harness 的提案对比裁决，不是定案。<br>
> 输入：全库精读（delivery / evidence / system+architecture / decision-history+回顾 memo / project+task 合同 / agent+run 合同，六路并行）+ 四系统 2026-08 现状联网核实（Tuwunel / Vikunja / conductor-oss / Zellij）+ 六 harness 社区口碑核实。行号引用以 2026-08-22 的 main（ffb861a）为准。<br>
> 口径：P0 = 『探路』——限时、可丢弃的协议/分发/打包探针（delivery.md:53）。本方案只安排探针，不安排产品化；产品化按首次消费在 P2 落地。

## 〇、结论先行

- **六张探针卡**：四个系统各一张（A Zellij、B Tuwunel、C Vikunja、D Conductor）+ 打包/构建一张（E）+ **OS 沙箱与凭据网关一张（F，文档缺口，须先补进 delivery）**。
- **编队**：Grok 出全部测试计划与对抗清单 → Codex 攻 B/F（逻辑最密 + 沙箱本行）→ GLM 攻 A/E（系统动手向）→ Kimi 攻 C/D（定义清楚的 API/容器活）→ Antigravity 只做独立复跑与机械核对 → Claude 总协调、写证据与决策记录。**每张卡的『通过』不由实施者自报**：Antigravity 复跑 + Grok 对抗审查通过后，才由 Claude 落 evidence，人拍定案。
- **日历**：两周。第一周 A/B/C/E/F 并行开跑（D 可押后），第二周收尾、补证据、落文档勘误。每卡 timebox 3 个工作日（F 为 5），到时未过即按记录在案的降级方向重开选型，不延时硬磨（Conductor 三件套纪律，delivery.md:227）。
- **联网核实已经改变计划的四个事实**：见第二节。其中 Tuwunel 的 macOS 构建是 P0 最大的真实风险；Vikunja 的 darwin 包假设已过时（好消息）。
- **需要所有者拍板三件事**：见第七节。

## 一、P0 的硬约束（从文档抽回来的口径）

1. **产出只有三样**：实现证据（evidence 条目）、固定精确版本（release/commit/storage backend/build features）、产品化约束。探针脚本、数据、拼装环境全部可删除，不进产品生命周期（delivery.md:231）。
2. **通过不等于可运维**：不宣称一键生命周期、备份恢复或升级已具备——那些到各系统首次被纵向切片消费前（B1/B2/B4）才由 control 产品化（delivery.md:53, 231）。
3. **不是全局 barrier**：chat 与 task 探针在 B1 首次消费前完成即可，运行时探针在 B2 前，workflow engine 探针在 B4 前且不得阻塞 B2（delivery.md:231）。对 P1 的含义：施工顺序是『先探针、再物理执行原语』（delivery.md:49），P1 备装的唯一 P0 前置是 Zellij 探针的结论——**A 卡通过线一亮即可开工 P1，不必等其余五卡收口**。
4. **失败路径固定**：重开并修订对应选型决定与 decision-history；降级方向已记录在案——Zellij→tmux、Tuwunel→Continuwuity、Vikunja→git-bug（须显式接受『任务 content 也在 Git』例外）；不自研同类系统（delivery.md:227, 233-236）。
5. **合规门禁**：探针引入的任何外部源码固定已审阅 commit + 许可证核验 + attribution；密钥走 OS secret store；各系统管理端点只绑 loopback / owner-restricted socket（delivery.md:252; spec/system.md:36, 131）。

## 二、开工情报：2026-08 现状核实改变了计划的四处

**①（风险）Tuwunel 没有 darwin 支持路径，macOS 交叉构建假设存疑。**
最新 v1.9.0（2026-08-18）全部 34 个 release 资产均为 linux-gnu；`rust-toolchain.toml` 中 apple-darwin 目标被注释；源码构建依赖 liburing（Linux 独有）。delivery.md:244 假设『由我方 CI 为 pinned 版本交叉编译产出 darwin 构建』——这条假设没有任何官方支持背书，需要禁用 io_uring 相关 feature 自行摸索，官方无文档无 CI 覆盖。备选 Continuwuity 同样无 macOS 产物且文档明言无法交叉构建静态二进制——**换备选救不了这个问题**。因此探针 B 拆成两截：协议探针在 Linux 容器里跑（与宿主无关，先把协议假设的证据拿下），macOS 原生构建归探针 E 单独攻坚；E 失败的降级方向不是换 homeserver，而是 macOS 上 chat server 的交付形态例外（进 Linux VM/容器，与 Conductor 同舱）——这将是一个需要记入决策历史的打包例外。
另两条与探针直接相关：Synapse 兼容 admin API 是 2026-07（v1.8.1）才落地的新功能，端点按需实测；RocksDB 在线备份 v1.8.2 引入、v1.8.3 才修掉『重复恢复覆盖新数据』的 bug——备份恢复演练必须实测，不可只信文档。

**②（好消息）Vikunja 官方 darwin 包已经存在。**
v2.5.0（2026-08-04）release 资产中实测存在 darwin amd64/arm64 构建——delivery.md:244『Vikunja 官方不出 darwin 包』已过时，探针确认后应勘误，我方 CI 负担减半（剩 Gatekeeper/公证核验）。另三条改探针内容：v2 API（2.4.0 起）提供 **ETag 条件请求**——这正面回答选型判据的『条件写入』要求（delivery.md:224），探针 C 必须实测 If-Match 语义；webhook **无重试、投递仅一次**（≥400 或超时即丢）——观测机制必须验证『webhook + 轮询兜底』的组合而非单靠 webhook；position 为 float64、按 view 存储、间距 <0.01 触发服务端全量重排——排序探针必须验证『写后读回以服务端为准』。2026 年上半年该项目集中修了大量安全漏洞，pin 必须 ≥2.5.0 且管理端点严格 loopback。

**③（已解）Conductor postgres-only 有官方参考配置。**
repo 内 `docker/server/config/config-postgres.properties`（含 `conductor.elasticsearch.version=0`）与 `docker-compose-postgres.yaml` 一键起 server+PostgreSQL，无 ES 无 Redis——delivery.md:233『找到单机可运维的最小持久化组合』这个开放问题基本已有答案，探针 D 只需实测确认 + 量打包重量（镜像压缩约 432MB）+ 备份演练（全状态在一个 PG 库，pg_dump 即全量）。两个陷阱要写进探针：**不要用 conductoross/conductor-standalone 镜像**（停更于 3.15.0/2023-12，教程常引）；evidence 里的评估版本 v3.21.23 已旧，最新稳定 v3.32.1（2026-08-12），pin 时按 digest 固定。

**④（风险）Zellij headless 有已知未修 bug，0.45.0 刚发布两天。**
Issue #3733（open）：对未 attach 的后台会话连续多次 `zellij run` 只有第一条生效——直击我们的 headless 编排模型，必须首先复测。#5158 记录的转发机制（被转发型查询如 OSC 11 的响应不回传给 pane）**推断**在无 attach 客户端时同类查询将得不到应答——该推断尚无直接针对后台会话的 issue 报告，A 卡实测证实或证伪；delivery.md:234 点名的『headless 终端查询应答』正是冲这一点。kitty keyboard protocol 有多处边角缺陷（#4333 flags 栈不完整、#3592 NumLock 冲突）——各 harness 按键矩阵实测不可省。内存基线约 80MB、长会话病理场景可达 GB 级（#2104、#5056）——常驻内存实测记入证据即可，不设通过阈值但要给产品化约束（scrollback 限制策略）。版本 pin 建议：主探 0.44.3（2026-05-13，三个月成熟度），同套脚本在 0.45.0（2026-08-20）上复跑一遍记录差异，pin 判据=两版中通过项多且无阻断缺陷者。tmux 降级方向保持热备：社区共识其程序化控制仍更深，若 A 卡 headless 项失败，重开成本低。

## 三、探针卡片

通用格式：每卡产出 `REPORT.md`（原始实测记录）+ evidence 条目草稿（按 implementation-evidence.md 既有格式：锚点 + 标题 + 『固定版本 [`vX.Y.Z / 短commit`](URL)（日期，许可证）』+ 主要证据链接）+ 产品化约束清单。所有服务只绑 loopback；所有数据可删。

### P0-A · Zellij 运行时后端（deadline：B2 前；timebox 3 天；实施 GLM，复跑 Antigravity）

验证清单（合同依据：spec/agent.md 的 attach/租约/代次/观测仲裁条款）：
1. exact attach / detach：detach 与客户端退出均不停止底层进程；多观察者同时 attach 可行。
2. headless：`zellij attach --create-background` 建会话，无人 attach 期间以 `zellij action`（write-chars / dump-screen / list-*、subscribe NDJSON 流）驱动与观测；**首项复测 #3733**（连续多次 run/write 是否全部生效）。
3. 终端能力查询应答：后台会话内跑 fish 4.x 与依赖 OSC 11/XTGETTCAP 查询的 TUI（含各 harness 本体），观察挂起/超时行为——这是本卡的通过线主项。
4. 单输入者可实现性：验证 agentd 网关模型的物理基础——能否从外部对同一会话强制单一输入路径、接管后旧连接失效（后端原生不支持则验证网关代管可行）。
5. 断流诚实分级：杀 server / 杀 client / 重启后，能否机械判定『是否仍是同一进程』；证明不了时的降级路径（semantic resume / replay / 新建）。
6. resize、残留进程、EXITED 会话清理（#4641）、macOS/Linux 双平台全套。
7. kitty keyboard protocol 按键矩阵：六个 harness 逐一实测（矩阵由 Grok 生成，GLM 执行）。
8. 实测打包重量、空会话/工作会话常驻内存；确认 web client 默认关闭，`zellij-no-web` 预编译产物存在性。

通过线：1/2/3/4 全过；3 按 delivery.md:234 原判据执行——**无人 attach 期间 TUI 的能力查询须有应答**；5-8 记录实测即可。失败动作：headless 或单输入者不可行 → 以同套契约测试转测 tmux，重开运行时选型（decision-history 记录）；若实测结果是『查询无应答、但存在可靠超时降级』这一中间态，不得自行算通过——作为『放宽 delivery.md:234 判据』的语义变更提案交所有者裁决（放宽 vs 转 tmux 二选一）。

### P0-B · Tuwunel chat server（deadline：B1 前；timebox 3 天；实施 Codex，复跑 Antigravity）

宿主：Linux 容器（协议验证与宿主无关；macOS 原生构建归 E 卡）。pin 候选 v1.9.0（注意其 CA bundle 新要求）。
验证清单（合同依据：spec/project.md 的 Room/RoomEvent 写入合同）：
1. 本地分发一键起停；管理端点 loopback 绑定确认。
2. 程序化账号/房间管理 API（v1.8.1 起的 Synapse 兼容 admin API 按端点实测：建账号、建房、权限、purge）。
3. **事务 ID 幂等**：同一 txn_id 重发不产生第二个事件（合同前提，evidence 明言以验证结果为准）。
4. **单 homeserver 线性顺序**：并发写入回读顺序稳定、可作时间线权威。
5. 事件精确引用与冻结 digest：按事件 ID 回读消息、算 digest；编辑/撤回产生新事件而原事件仍可取回。
6. AppService 程序化注册（拍板理由之一）：admin 命令热注册 / 配置文件两路实测，exclusive namespace 行为确认。
7. 重同步：客户端断连后 cursor 续传不丢事件。
8. 备份恢复演练：RocksDB 在线备份（v1.8.2+）实测备份→毁库→恢复→数据完整；同时演练冷备（停服拷数据目录）。
9. 确认存储后端事实（RocksDB 系）与 build features——delivery.md:235 要求探针固定精确 release/commit、storage backend 与 build features，据实回填。

通过线：3/4/5/6/8 全过。失败动作：协议级失败 → Continuwuity 同套脚本对照（脚本本就可复用），重开 chat 选型。

### P0-C · Vikunja task backend（deadline：B1 前；timebox 3 天；实施 Kimi，复跑 Antigravity）

pin 候选 v2.5.0（安全底线，不得更低）。API 面：v1 为主（已冻结、文档全），v2（ETag）作条件写入验证面并评估直接采用 v2 的成熟度。
验证清单（合同依据：spec/task.md 的身份映射/快照/排序令牌条款）：
1. 单二进制 + SQLite 起停；管理端点 loopback；API token 按 scope 授权实测（注意 CVE-2026-40103 类 scope 越权已修的验证）。
2. 『一个 Repo 一个 Board』落地形态：board_scope_stable_id 可得且跨重启稳定。
3. Project 分组实体：Vikunja 里用什么承载（bucket 归属 view 的语义自 0.24.0 起变了——实测父任务/label 哪种能按 anchor 稳定回读归属），能力不足时降级为 label/filter 的判定路径。
4. 身份稳定性：task 的 immutable ID 在移动 bucket/view、重命名、跨 project 移动后不变；placement 身份与实体身份可分离取得。
5. **条件写入**：v2 API ETag/If-Match 实测——过期前置被拒、新鲜前置成功（选型判据硬要求，delivery.md:224）。
6. **排序令牌**：position 语义实测——float64、view 级、中点插入、<0.01 触发服务端全量重排、写后读回以服务端为准；结论回答『有无可条件写入的并发令牌』，无则记录降级为绝对移动+回读。附两条显式断言（侦察点名、未确认已修）：新建任务默认落点行为（community #4379 回归）、position 字段类型校验错误路径（issue #317：`positions.view_* must be int64` 致新任务静默丢弃）。
7. **泳道/bucket 语义**：bucket 自 0.24.0 起归属 view——bucket CRUD、done-bucket 副作用、bucket 增删改后投影锚点的稳定性实测（供 CT-TASK『lane 投影与外部状态分离』断言）；跨 bucket 移动走专用端点（任务更新传 bucket_id 无效）的行为确认。
8. 观测机制：webhook（HMAC 签名、事件清单、**无重试**）+ 轮询兜底的组合演练；断流后 gap 可检测。
9. 备份恢复演练：`vikunja dump` / `restore --preserve-config` 实测（注意 CLI dump 只能 CLI restore）。
10. AGPL 采用边界确认记录：独立进程 + REST，不 vendor 不链接。

通过线：2/4/5/8/9 全过且 3、7 有明确结论（哪怕是降级结论）。失败动作：身份或观测不可靠 → git-bug 对照探针（须显式接受模型例外），重开 task 选型。

### P0-D · conductor-oss workflow engine（deadline：B4 前，不阻塞 B2；timebox 3 天；实施 Kimi，复跑 Antigravity）

pin 候选 v3.32.1（按镜像 digest 固定；**禁用 standalone 旧镜像**）。宿主：Colima/Podman 容器。
验证清单（合同依据：spec/run.md 的 Engine 绑定/回读/幂等条款）：
1. postgres-only 参考配置实测起停（server+PG 两容器，无 ES 无 Redis）；量镜像体积、常驻内存（JVM heap 1-2GB 假设实测）。
2. versioned workflow definition 注册与回读：注册带版本定义、取回比对摘要（Engine Deployment 分歧检测基础）。
3. SIMPLE worker task poll/complete 全部经单一进程：长轮询领取、提交结果、未 ack 重新入队行为。
4. execution id + correlation key 恢复：起 execution 后杀掉客户端进程，凭 correlation 重查状态重建绑定。
5. 回读推进：不依赖回调，主动查 execution/task 状态与 success terminal。
6. retry 身份：引擎 retry 产生可区分的新 task execution identity（taskId/retryCount 实测对应）。
7. complete 幂等：重复 complete 行为确认；terminate/取消可收口，引擎失联后回读收口不永久阻塞。
8. Profile 节点子集：fork/join、switch、loop、dynamic fork、timer 可用；SUB_WORKFLOW/HUMAN/HTTP 类副作用节点可禁用或可不采用。
9. 备份演练：pg_dump 全量 → 毁 → 恢复 → execution 状态完整；归档/TTL 清理行为确认。
10. 升级路径：3.21→3.32 或 3.32.0→3.32.1 的一次实测升级。

通过线：1/3/4/5/7 全过。失败动作：重开 Engine 选型——修订选型决定 + decision-history 记录（不自研第二引擎；替代动量仅作风险记录）。

### P0-E · 打包与构建（横切；timebox 3 天；实施 GLM，复跑 Antigravity）

1. **Tuwunel macOS 原生构建攻坚**（本卡主项）：禁 io_uring 相关 feature、libclang 编 RocksDB，在 macOS 本机与 CI 各试一次；成败都记录精确步骤与产品化约束。失败 → 提交『macOS 上 chat server 走 Linux VM/容器舱』的打包例外提案（语义变更，owner 裁决）。
2. Vikunja darwin 官方包核验：下载、Gatekeeper/公证行为、跑通 C 卡冒烟；确认后勘误 delivery.md:244。
3. zellij-no-web 构建/预编译产物核验（A 卡的瘦身选项）。
4. Colima/Podman 免授权容器运行时在 macOS 的安装与资源开销实测（D 卡前置）。
5. 我方 CI 骨架示范：对一个 pinned 版本产出带校验和的构建物（合规门禁演练：commit 固定、license 核验、attribution）。

### P0-F · OS 沙箱与凭据网关（阻断性；timebox 5 天；实施 Codex，复跑 Grok+Antigravity）

**前置：本卡尚不在 delivery.md 开工前限时验证清单里**——decision-history §17 与回顾 memo 已把它定为 P0/B2 阻断性工程验证（『做不到则候选标 unsupported，不能把设计承诺降成 prompt』），但清单未同步。建议先补第 6 项（语义变更提交），再开卡。
验证清单（合同依据：spec/system.md:101,192 与 CT-AGENT 负例）：
1. macOS Seatbelt（sandbox-exec/libsandbox）与 Linux namespaces/Landlock 各建一个最小沙箱原型，把一个真实 harness（先 Codex CLI，再 Claude Code）关进去跑真实编码任务。
2. 负例逐条实测：读不到 OS secret store / SSH agent / control 凭据 / 未授权 provider 配置；写不了沙箱外文件与目标 Git ref；网络目的地白名单可强制。
3. 凭据网关原型：git push 经代理签名/代用凭据完成，harness 环境内无凭据可窃。
4. 每个候选 harness 出一行结论：可强制 / 带约束可强制 / unsupported。
5. 参考实现调研记入证据：Codex CLI 自身的 Seatbelt/Landlock 沙箱是现成先例，探针应先读其实现。

通过线：至少一个 harness 在 macOS 与 Linux 都达到『可强制』。失败动作：这是设计承诺的地基——任何 harness 都做不到则必须回到架构层重议（受治理执行的范围收缩），不是换个库能解决的事。

## 四、Harness 编队

| Harness | 角色 | 卡 | 依据（所有者评价 + 社区核实） |
| --- | --- | --- | --- |
| Grok Build + Grok-4.6 | 测试计划与对抗审查 | 全部卡的测试计划/对抗清单先行；结果的对抗性审查；kitty 按键矩阵生成 | 对抗分析与 testing plan 是其公认强项；长程可靠性未经独立验证 → 不当主攻手 |
| Codex + GPT-5.6-sol | 主攻手 ① | F（先）→ B（后） | 逻辑勾稽最密的两张卡；OS 级沙箱是 Codex CLI 本行（Seatbelt/Landlock 先例）；探针可丢弃，叠床架屋弱点无害 |
| Opencode + GLM-5.3 | 主攻手 ② | A → E | 系统动手向长清单机械执行；平价，量大管饱；GLM-5.3（2026-08-14）terminal/tool use 强化系官方自报、独立证据弱——开工前先过金标校准 |
| Kimi Code + K3 | 主攻手 ③ | C → D | 定义清楚的 REST/容器活；幻觉偏高 → 所有断言须复跑背书；自带沙箱弱 → 不碰 F 卡 |
| Antigravity + Gemini-3.7-flash | 机械复核员 | 全部卡的独立复跑；版本/许可证/下载物核对；起停脚本每日冒烟 | 能力最低但快；『复跑脚本核对输出』是其安全区；永不做原创作者 |
| Claude Code + Fable-5 | 总协调 | 编队指挥、evidence/decision-history/勘误写作、裁决建议汇总 | 架构与文档写作强项；本仓库文档纪律的既有执行者 |

**协作纪律（把 HCTL 的理念先手动过一遍）**：
1. 每卡顺序：Grok 测试计划 → 主攻手实施并写 REPORT → Antigravity 在干净环境复跑 → Grok 对抗审查（专挑『声称通过但证据不支持』）→ Claude 落 evidence 草稿 → **人拍定案**（修订选型决定与 decision-history，不另造 ADR catalog——三件套纪律）。producer/reviewer separation：任何 harness 不给自己的工作背书。
2. 模型自述不算数：REPORT 里每条结论必须附可复跑的命令与输出；复跑不一致按未通过处理。
3. 该机械做的绝不交给模型（施工纪律，decision-history §16）：起停脚本、校验和核对、许可证文本比对写成脚本，由脚本产出事实。

## 五、工程形态

- 落点：hctl2 repo 内新建 `probes/` 目录（`probes/README.md` 顶部声明：全目录可丢弃，不进产品生命周期，P0 收口后可整体删除或归档）；每卡一个子目录（`probes/zellij/` 等），内含安装/冒烟脚本、`REPORT.md`、测试计划。
- 长期产物只有三处：`docs/design/references/implementation-evidence.md` 新条目（按既有格式：锚点、固定版本 `vX.Y.Z / 短commit`、许可证链接、主要证据链接）、decision-history 的定案/翻案记录、delivery.md 的勘误与回填（每处独立提交，真语义变更单独立项——方法论纪律）。
- P0 期间已知要落的文档勘误（探针确认后逐条提交）：Vikunja darwin 包事实（delivery.md:244）；Tuwunel storage backend/features 回填（delivery.md:235）；Conductor 评估版本 v3.21.23 → 实际 pin 版本（evidence E-L2-CONDUCTOR）；Zellij/tmux 选型定案后补独立 evidence 条目（现仅 L1 表格一行）。

## 六、日历（建议，两个日历周）

- **第 1-3 天**：Grok 出六卡测试计划 + 给 GLM/Kimi 各出一个半天金标校准小任务（不合格即换防，备位：Codex 顶 A、Grok 顶 C）；GLM 开 A；Codex 开 F；Kimi 开 C；Antigravity 搭复跑环境 + 核对四系统版本/资产/许可证。
- **第 3-6 天**：Codex 转 B（Linux 容器）；GLM 转 E（macOS 构建攻坚）；Kimi 转 D；Antigravity 逐卡复跑。
- **第 6-10 天**：F 卡收口（最长 timebox）；Grok 对抗审查全部 REPORT；Claude 落 evidence 六条 + 勘误提交 + 定案草稿；到时未过的卡走失败动作。
- P1 备装（agentd + tool）不等 P0 全收口：A 卡通过线一亮即可开工。

## 七、需要所有者拍板

1. **F 卡入册**：把『OS 沙箱与凭据网关』补为 delivery.md 开工前限时验证第 6 项（语义变更 + decision-history 记录）。不补则 F 卡师出无名，而它是回顾 memo 点名的阻断性验证。
2. **探针落点**：repo 内 `probes/` 目录（我的建议，证据可引用、历史可追溯）vs 独立 scratch 仓库（更干净的可丢弃语义）。
3. **timebox 授权**：各卡 3 天（F 为 5 天）、总两周、到时即走失败动作——是否照此执行；以及六个 harness 的调度由谁排（建议：人只出现在定案与裁决点，日常由 Claude 排班）。

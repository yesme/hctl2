# Codex · 自建方案盘点：执行面与现场工具

> 状态：待拍板 · 独立报告；替换建议交主笔汇总，不是实施裁决
> 基线：origin/main @ f531ca195b4c2c96c4a5279acff516360dc64f05（草案 v0.19.2）
> 去向：核销与汇总见 `.memo/design/community-first-audit-20261005.md` §六；裁定后另开实施包

## 一、结论与范围

这一块不是整套执行引擎都自己重做了。RPC、数据库、密码算法、JSON 规范化、OS 文件锁、Git 操作已有社区基础。真正需要纠正的是：采用 Herdr 后，遇到接口不便就另外管理 Harness 进程；另有几处通用功能没有充分使用已经引入的库。

列七项。C1 是 **#335 尚未合入、所有者已叫停的实现**，不算 main 的缺陷。C2–C7 是 main 上的机会，其中 C4 要先验证部署代价，不能直接下替换命令。最明确的小改是 C5 的数据库迁移；C6、C7 可以继续利用现有组件，避免增加一套服务。

### 清点范围

| 范围 | 生产源码及内联测试规模 | 已核的功能 | 目前借用方式 |
| --- | --- | --- | --- |
| `src/agency` | 10 个 Rust 文件，约 2,737 行；另核 3 份 Skill | 配对与租户、派工协议、持久化、脚本执行体、Herdr 接入、OS 限制、启停 | Herdr 二进制；tonic、rusqlite、Landlock、标准库 SDK；少量自建通用机制 |
| `src/crates/agency-proto` | 5 个 Rust 文件、1 个 proto 文件，约 1,033 行 | RPC、执行规格、票据、结果头、Context 类型、摘要与编码 | tonic / prost 生成协议；RustCrypto、Base64、JCS SDK；HCTL 语义自己定义 |
| `src/crates/participant` | 3 个 Rust 文件，约 1,184 行 | 接受工种、选入记录、Worker Profile、候选校验 | 复用 Store 与规范化 SDK；治理规则自己定义 |
| `src/crates/context` | 5 个 Rust 文件，约 1,076 行 | 精确消费者选材、权限与预算、交付副本、冻结与读取 | 复用 Store、chat / task 接口及摘要 SDK；选材规则自己定义 |
| `src/apps/tool` | 8 个 Rust 文件，约 3,867 行 | 仓库检查、现场锁、worktree、封存、集成、平台事实等待 | 主机 Git 二进制、标准库文件锁；授权与核验逻辑自己定义 |

共 32 个生产源码文件，约 9,897 行；独立测试文件另约 7,856 行。计数含空行、注释、内联测试，不含生成代码；不是「重复造轮子的行数」。几项位置重叠，不能相加当作可删规模。

先读了[部件矩阵](../../../docs/research/component-matrix-20260902.md)、[Harness 适配](../../../docs/research/harness-adapters.md)、[Herdr SDK](../../../docs/research/sdk/herdr.md)、[运行时验证](../../../docs/research/runtime/agency-runtime-validation-20260829.md)、[Agency 端口](../../../docs/research/runtime/agency-port.md)、[Process Compose](../../../docs/research/runtime/process-compose.md)、[Git](../../../docs/research/sdk/git.md)、迁移、文件锁及 Context 的既有研究。没有读另两席本轮报告。

## 二、发现

### C1 · #335 另管真实 Harness 进程：回到官方 Herdr

- **位置**：未合入的 [#335](https://github.com/yesme/hctl2/pull/335)，审到 `1ec93e1faf183185df2407456ed932260b08558d`；`src/agency/src/launch.rs` 约 830 行，`harness/{mod,claude}.rs` 约 108 行，另涉及 Herdr、入口和测试。不是本报告 main 基线上的代码。
- **自建了什么**：Agency 自己启动真实 Harness、持有 `Child`、收管道输出与退出状态，Herdr 退成显示端。第 1 类；也增加了第 2 类进程管理代码。
- **它解决什么问题**：想取得不依赖屏幕回显的结束状态和结构化输出。
- **社区方案**：[Herdr 0.9.3](https://github.com/herdrdev/herdr/releases/tag/v0.9.3)，源码 `7b116c05bfda646af39d2524c54e70c751f57ee8`，Apache-2.0，09-29 发布且上游仍有提交。原生 `layout.apply` 可带 argv、目录和环境，由 Herdr 启动 PTY 中的程序；`pane.close` 管关闭。[官方接口文档](https://github.com/herdrdev/herdr/blob/7b116c05bfda646af39d2524c54e70c751f57ee8/docs/next/website/src/content/docs/socket-api.mdx)与[进程实现](https://github.com/herdrdev/herdr/blob/7b116c05bfda646af39d2524c54e70c751f57ee8/src/pane.rs)已核。main 已采用官方 0.8.2；原生 argv 启动并非 0.9.3 才有。
- **当初为什么自建**：#335 的 `e5e38b1` 接手修正，为取得真实退出状态选了旁路。所有者 10-05 否定这条路；[演示开工书](../../design/p2-control-20260906/07-demo-kickoff.md) §二第 18、19 条已记录坚持社区版、进程仍由 Herdr 管，以及完成判断的修正。
- **换过去要动什么**：撤掉真实 Harness 的独立 `Child` 路径，保留 Agency 的授权、归属、结果封存和准入适配；改 `launch`、`harness`、Herdr 客户端、入口与 live 测试。升级官方制品还需另核 lock、协议 20→22、schema 和发行测试。没有理由因此改派工对象。现有探针尚未解决「pane 不能写 Herdr 状态目录」；不能把启动成功当作四条验收全过。
- **建议**：**换，方向已定，#335 负责实施。** Harness 生命周期由自研提升到采用社区二进制；HCTL 治理适配仍自己写。明确结束接口、通用结构化结果通道的上游提案已被所有者暂缓，不在本报告重新要求。程序退出也不等于 Task 完成；Agent 表示做完、Sysone 评估与真实产物验收是另一件事。
- **把握**：**实测 + 查证**。本机 macOS arm64 用官方 0.9.3 原生 argv 启动已登录的 Claude Code 2.1.289，得到 `HCTL2_NATIVE_HERDR_OK`；关闭 `sleep 30` 的 pane 后进程退出。是原生接口探针，不是完整派工链。探针同时得到 `CREDENTIAL_READ_DENIED / STATE_WRITE_ALLOWED`，所以隔离验收仍未完成；Linux、macOS x86_64 未实跑。

### C2 · 交互 shell 加屏幕标记：别让它长成正式 Harness 协议

- **位置**：[Herdr 接入](../../../src/agency/src/herdr.rs) `run_command`，约 100 行；对应 `tests/herdr.rs`。全文件 503 行，不全属于此项。
- **自建了什么**：先建默认 shell pane，再发送调用方命令，以 `pane.wait_for_output` 的文字匹配收尾，另加标记、控制字符过滤。第 4 类。
- **它解决什么问题**：证明 Herdr 管道能启动程序、看到输出并收掉 pane。
- **社区方案**：C1 的官方 Herdr 0.9.3 / Apache-2.0 原生 argv 启动，已使用且在维护；能力来自同一[接口源码](https://github.com/herdrdev/herdr/tree/7b116c05bfda646af39d2524c54e70c751f57ee8/src/api/schema)。它可绕过交互 shell 的命令回显和历史替换，但 **0.9.3 的 `PaneExited` 仍不带数字退出码**，事件流也不是完整、持久的结果存储。
- **当初为什么自建**：[#328](https://github.com/yesme/hctl2/pull/328) 收窄为 Herdr 管道后使用这个验证助手；`330d8d6`、`86c523e`、`cb2e3ef` 逐次补过滤。评审已经指出 shell 历史替换可绕过输入过滤。正式 Harness 留给 3c，不把旧测试助手说成 main 已实现 Task 判定。
- **换过去要动什么**：正式路径直接传 argv / env / cwd；把测试改成核原生启动、观测和关闭，不靠屏幕上某一句话证明执行终态。旧助手若保留，只用于明示范围的烟雾测试；没有历史文件迁移。Agency 对结果语义的封存与验收仍需保留。
- **建议**：**换正式路径；有限保留测试助手，不再增加过滤。** 从自研 shell 调度提升为采用 Herdr 二进制。终态观察与业务结果判断分开；不要为了缺一个数字退出码又写一个平行进程管理器。
- **把握**：**查证 + 原生接口实测**。核了 main 函数、#328 修正史、上游 `PaneExited` 和 `child.wait` 实现；原生启动/关闭实测同 C1。本轮没有重新运行 shell 历史替换反例，也没有宣称事件退出码问题已由升级解决。

### C3 · 手写 Herdr wire 形状：兑现已有 schema 研究

- **位置**：[Herdr 客户端](../../../src/agency/src/herdr.rs) `Client` 约 90 行，加调用处的 `json!` / JSON Pointer；扩成 3c 后还有更多响应解释代码。
- **自建了什么**：手写方法名、参数与返回结构，用 `serde_json::Value` 逐字段解释 Herdr 私有协议。第 3 类；不是自创 Herdr 传输协议，而是手工维护其类型副本。
- **它解决什么问题**：从 Rust 调用官方 Unix socket API。
- **社区方案**：Herdr 二进制的 `api schema --json`，版本与许可同 C1。现行 [SDK 研究](../../../docs/research/sdk/herdr.md) 首选 [typify 0.7.0](https://github.com/oxidecomputer/typify/blob/v0.7.0/README.md) / Apache-2.0 生成类型；失败则复制 Herdr 同版本 `src/api/schema` 的 Apache-2.0 类型。typify 已有 0.7.0 发布和公开问题/PR，未核最近一次提交时间；我们尚未用它。Herdr 本身仍是 bin-only，不能声称有可直接链接的官方 Rust SDK。
- **当初为什么自建**：#328 只接几条方法，短实现能很快证明管道。没有找到正式撤销「先生成、失败再复制类型」建议的记录；研究中的「决定建议」也不能冒充所有者已经拍板的实现义务。
- **换过去要动什么**：先用钉定二进制导出 schema，验证生成代码可编译、请求与响应序列化能与真实 API 往返；成功才接 BUCK 原生生成目标，失败取同版官方类型并保留许可。只替换方法/参数/响应类型；小段 NDJSON socket 适配可留。无需数据库迁移，成本在生成器适配、类型体积和升级检查。
- **建议**：**换手写类型的来源，先做有停止条件的生成实验。** 自研 wire 类型→SDK 生成；失败→复制官方类型。若仅几条方法且两种替代的维护成本都更高，允许明示保留小子集，而不是为了「自动生成」再造大型生成框架。
- **把握**：**查证，未做生成实测**。亲读 Herdr schema 与 typify 的说明、许可、既有研究。Herdr 使用 draft 2020-12；typify 的兼容与复杂 `oneOf` 有已记录边界，不能把「有 schema」写成「生成已经可用」。

### C4 · Agency / Herdr 守护进程启停：先用现有监督器核代价

- **位置**：[Agency 入口](../../../src/agency/src/main.rs) 的 `start/status/stop`，以及 `herdr.rs` 的 `Server::start`、PID 文件、`ps` 校验、发信号、就绪轮询，合计约 300 行、2 个文件。这里讨论常驻服务，不讨论 Harness pane。
- **自建了什么**：后台启动、固定轮询、旧 PID 回收与进程组终止。第 2 类。目前还不是完整的崩溃重启监督器。
- **它解决什么问题**：独立安装的 Agency 和私有 Herdr 服务可启动、探活、清理。
- **社区方案**：[Process Compose 1.122.0](https://github.com/F1bonacc1/process-compose/releases/tag/v1.122.0)，Apache-2.0，08-18 发布且上游有后续提交；我们已在 services 包采用它。核了[源码 `up` 的 detached / 依赖选择](https://github.com/F1bonacc1/process-compose/blob/23b0acacc937d745279fb1551337f4031c4fc865/src/cmd/up.go)、[许可](https://github.com/F1bonacc1/process-compose/blob/23b0acacc937d745279fb1551337f4031c4fc865/LICENSE)，结合本库[研究](../../../docs/research/runtime/process-compose.md)的 readiness、restart、日志与私有 socket 验证。
- **当初为什么自建**：[#317](https://github.com/yesme/hctl2/pull/317) 确立 Agency 独立启动；#328 用有界启动和 RAII 保证失败不留 Herdr。没有找到明确比较 Process Compose 与 Agency 独立随包代价后选择手写的记录。
- **换过去要动什么**：核 Agency 是否随包带监督器、配置与启动命令能否保留独立生命周期；把 daemon 监督交给原生配置，Agency 留配对、授权、就绪语义。涉及 Agency 入口、Herdr Server、打包和生命周期测试；跨到 DeepSeek 的发行范围。先验证两 Agency 并存、重启、父进程退出、残留回收、私有状态目录、用户登录环境，不引入第二套跨 Control 服务。
- **建议**：**有条件换，不列为马上重写项。** 通用 daemon 监督可由自研提升为采用已选二进制；若部署依赖和身份环境代价明显大于这段小代码，保留有界启动/RAII，而不扩展成完整监督器。Process Compose 管服务，Herdr 管 Harness，不能拿前者绕过后者。
- **把握**：**查证，未实跑替代部署**。现有 Process Compose 验证不是本次 Agency 接线试验。它不会自动解决 macOS Aqua 登录状态或 pane 的写目录隔离；这两项不能写成采用监督器即可修好。

### C5 · Agency 数据库版本：复用已经采用的迁移库

- **位置**：[存储](../../../src/agency/src/storage.rs) `database`、[服务](../../../src/agency/src/service.rs) `Agency::open`、[租户](../../../src/agency/src/tenant.rs) `Tenant::open`，迁移相关约 40 行、3 个文件。
- **自建了什么**：手查 `user_version > 1`，每次打开都执行 `CREATE TABLE IF NOT EXISTS` 并写 `user_version=1`；没有使用版本化迁移序列。第 2 类。
- **它解决什么问题**：创建 registry / tenant schema，拒绝较新版本数据库。
- **社区方案**：[rusqlite_migration 2.6.0](https://github.com/cljoly/rusqlite_migration/releases)，Apache-2.0，05-28 发布；仍有发布维护记录，支持本库 rusqlite 0.40。已被 Store 采用。[上游源码](https://docs.rs/crate/rusqlite_migration/2.6.0/source/src/lib.rs)的 `goto_up` 在事务里执行迁移、更新 `user_version` 并提交；本次还亲读了本机下载的同版源文件与 Cargo 许可。
- **当初为什么自建**：#317 第一版只有 schema 1，用简短 DDL 初始化。既有[SQLite 迁移研究](../../../docs/research/libs/sqlite-migrations.md)已经推荐该库；没有找到 Agency 单独不用它的论证。
- **换过去要动什么**：Agency 的 Cargo / BUCK 加已有依赖，registry 与 tenant 各保留独立迁移列表；不依赖控制面 Store。处理版本 0 的半初始化数据库、已有版本 1、更新版本拒绝；补迁移失败回滚和重开测试。不需要改当前表结构，也不需要改租户隔离。
- **建议**：**换。** 自研迁移流程→采用现有 SDK。SQL 表定义仍归 Agency；小 SQL 不等于要引 ORM，也没有理由把两侧数据库合并。
- **把握**：**查证，未接线实测**。核了 main 两段 DDL、库事务实现、版本/许可和本库已有依赖。当前只有 schema 1；本项是避免后续迁移继续自建，不把未来扩表失败说成现存运行故障。

### C6 · 结果分页：先让 SQLite 筛选与限量

- **位置**：[租户](../../../src/agency/src/tenant.rs) `results`，约 100 行、1 个文件。
- **自建了什么**：从数据库取出一次派工的全部结果正文，再在 Rust 找游标、截页、反复序列化试算。第 2 类；其中游标筛选与行数限制未利用数据库原生能力，字节预算判断则是必要适配。
- **它解决什么问题**：按 Result Proposal 游标分页，限制响应大小，派工仍运行时不误报读取完成。
- **社区方案**：已经使用的 SQLite，经 rusqlite 0.40.2 调用；本地 bundled 源码版本 3.53.2。SQLite [公有领域许可](https://www.sqlite.org/copyright.html)，仍在维护。[原生查询](https://www.sqlite.org/lang_select.html)支持过滤、排序和 LIMIT；[游标窗口查询说明](https://www.sqlite.org/rowvalue.html#scrolling_window_queries)解释按排序键续取。无需新依赖或新分页服务。
- **当初为什么自建**：[#323](https://github.com/yesme/hctl2/pull/323) 先把每页最多 32 条与 4 MiB 文档限制落成；没有找到为何必须先读全量正文的论证。该 PR 已正确保留运行中 `complete=false`，不能替换时丢掉。
- **换过去要动什么**：先在同一派工里把现有 Proposal ID 游标解析成排序键，SQL 续取最多 `limit+1` 条，再做响应字节预算检查。保持未知游标拒绝、跨派工拒绝、空页、并发追加、保全标志和运行中完成语义；需要长期稳定顺序时核既有序列是否要单独存列。先用现有 schema，只有验证确需新列才配迁移。
- **建议**：**换全量装载与手工截行；留字节预算和业务完成条件。** 自研集合分页→采用数据库原生查询。不要引分页框架；当前限制一次返回多少条并没有限制一次读进内存多少条。
- **把握**：**代码查证，性能后果为推演**。亲核查询后 `.collect::<Vec<_>>()` 及截页循环；未做大结果集 RSS / 耗时基准，也未运行替代 SQL。不能用本项声称已经发生内存耗尽。

### C7 · 文件系统识别：不用解析 mount 的人类输出

- **位置**：[现场锁](../../../src/apps/tool/src/site_lock.rs) 的 Linux `stat` 调用、macOS `mount` 调用及路径解析，约 120 行、1 个文件。
- **自建了什么**：macOS 解析挂载表、转义路径和最长前缀以找文件系统类型；Linux 解析外部 `stat` 输出。第 4 类。
- **它解决什么问题**：现场锁拒绝远程 / FUSE 文件系统，避免把本机 OS 锁当作跨机锁。
- **社区方案**：已引入的 [rustix 1.1.5](https://docs.rs/crate/rustix/1.1.5/source/Cargo.toml)，许可 `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`，有现行 1.1.5 发布；未核最近提交时间。[`fs::statfs`](https://docs.rs/crate/rustix/1.1.5/source/src/fs/abs.rs)是原生 SDK；[Apple 后端](https://docs.rs/crate/rustix/1.1.5/source/src/backend/libc/fs/types.rs)返回 libc 的 `statfs`，含 `f_fstypename`，Linux 按 `f_type` 判别。亲读同版缓存源码和 Apple 结构定义，不只看 docs.rs 的 Linux 页面。
- **当初为什么自建**：P1 现场锁实施采用系统命令以保持 Rust 无 `unsafe`；[Git 研究](../../../docs/research/sdk/git.md)与[文件锁研究](../../../docs/research/libs/fd-lock.md)支持拒绝非本地现场，但没有找到必须解析 `mount` 文本的论证。rustix 已提供安全封装，原来的顾虑可重新核。
- **换过去要动什么**：打开现有 rustix 的 `fs` feature，tool 增加直接依赖与 BUCK 接线；保留非本地类型判据和现有错误码。补路径含空格、嵌套挂载、符号链接、查询失败及未知类型测试；三平台编译验证。SDK 只提供事实，不替 HCTL 决定哪些文件系统可用。
- **建议**：**换，低优先级。** 人类输出解析→采用已有 SDK；排他锁本身已经是标准库原生能力，不重做它。先证明确认类型所需字段在三个目标可取，不能借此放宽保守拒绝策略。
- **把握**：**查证，未接线编译或挂载实测**。核了当前解析代码、rustix 函数 cfg 与 Apple `StatFs` 类型；替代的跨平台代码尚未写，不能报告三平台已通过。

## 三、应该保留的自建与已经借用的部分

这些也在清点范围内。保留不表示所有细节已验收，而是它们解决的是 HCTL 自己的问题，或底层已采用社区实现。

| 部分 | 结论与理由 |
| --- | --- |
| Agency 配对、租户、归属者代次、派工四步、租约、票据、结果保全 | **留**。这是 Agency 对控制面承诺的语义；Herdr 的 workspace / pane 不代替授权与结果准入。持久化底座已用 SQLite，迁移按 C5 收束。 |
| `agency-proto` 的 RPC | **留**。`.proto` 由 tonic 0.14.6 / prost 0.14.4 生成，不是自建网络协议栈。HCTL 载荷保留精确 JCS 字节，是冻结摘要的需要，不等于重做 protobuf。 |
| 票据的 HMAC、SHA-256、Base64、规范 JSON | **留薄适配**。分别用 hmac 0.13.0、sha2 0.11.0、base64 0.22.1、serde_json_canonicalizer 0.3.2；没有自己写算法。HCTL claim 字段与验证条件仍属授权语义，不能因为存在 JWT 就无差别换格式。 |
| `ScriptRuntime` | **留为协议测试执行体**。所有者已允许第 5 包先用它走链；不冒充真 Harness，不扩成旁路真实执行引擎。它的观测/结果帧是测试协议，不要求换 ACP。 |
| `confine.rs` / `linux_confine.rs` | **留 OS 适配，隔离缺口单独验证**。macOS 调系统 sandbox，Linux 用 Landlock 0.4.7 SDK，不是自写隔离引擎。权限策略属于 HCTL；采用组件不自动证明策略正确。Landlock 源包许可为 MIT OR Apache-2.0，既有研究只写 MIT 不完整；本报告不改研究正文。 |
| Participant 的工种接受、选入记录、Worker Profile 校验 | **留**。这些保存冻结的选择与能力承诺，复用 Store；不是 Herdr pane 注册表，也没有第二个通用调度器。 |
| Context 的来源选择、预算、Manifest / Bundle 与交付核验 | **留**。当前明确不生成纪要、不压缩、不调用模型；未接 tokenizer 时计量为 `None`。这里实现精确材料与权限的冻结，不是重写通用检索或记忆系统。将来做检索、token 计量、压缩时再按对象研究选库，不能把尚未实现的能力记成现存轮子。 |
| tool 的仓库检查、worktree、封存与集成 | **留**。底层调用社区 Git 的 `worktree`、临时 index、`write-tree`、`commit-tree`、`merge-tree --write-tree`、`update-ref` 比较并交换；没有自己实现 Git 合并或对象库。HCTL 的树摘要核对、预期头、危险残留保护和幂等意图是必要适配。Herdr 的 worktree 便利入口不替代这些约束。 |
| 现场锁与 Agency writer 锁 | **留**。实际排他走 Rust 标准库 `File::try_lock`；诊断 JSON 不是锁算法。既有 fd-lock 研究已推荐标准库，不再引正在退出维护的同题库。仅类型识别按 C7 替换。 |
| `tool wait` | **留**。等待的是经 facts / 平台工具读到的事实，没有自建 Workflow Engine；超时与错误输出是命令适配。 |
| 3 份 Skill | **留**。沿用原生 `SKILL.md` 形式；shaping 已复用 Matt Pocock 方法并带 MIT 许可副本，review / readback 表达 HCTL 的过程规矩。没有发现另造 Skill 包格式或加载引擎。 |

保留项的社区证据：本库 [protobuf RPC](../../../docs/research/libs/protobuf-rpc.md)、[JCS](../../../docs/research/libs/serde-jcs.md)、[HMAC](../../../docs/research/libs/hmac.md)、[Base64](../../../docs/research/libs/base64.md)、[Landlock](../../../docs/research/libs/landlock.md)、[文件锁](../../../docs/research/libs/fd-lock.md)研究，以及本次亲读的同版 crate 源码/许可。Git 本机是 `2.50.1 (Apple Git-155)`，GPL-2.0；原生能力核对了[worktree](https://git-scm.com/docs/git-worktree)、[merge-tree](https://git-scm.com/docs/git-merge-tree)、[update-ref](https://git-scm.com/docs/git-update-ref)与[许可](https://github.com/git/git/blob/v2.50.1/COPYING)。这些 SDK / 二进制已经在用，不建议为抬高借用级别引入一个不满足冻结授权要求的完整 Agent 产品。

## 四、证据边界

| 证据 | 实际做了什么 | 没做什么 |
| --- | --- | --- |
| main 清点 | 冻结开工 SHA，检查五个范围的模块、依赖与通用机制调用路径，核相关测试、README、既有调研及提交史 | 不是逐个测试全重跑的代码正确性审查；不覆盖 Grok / DeepSeek 的专属范围 |
| 社区替代 | 亲读 Herdr 钉定源码/二进制接口、typify、Process Compose、迁移库、SQLite、rustix 与 Git 原生文档；许可从源码/分发信息核 | 没把既有研究的推荐当成运行验证；未对所有库核最新提交时间 |
| 3c 关联探针 | macOS arm64 的官方 Herdr 0.9.3 原生启动、关闭、Claude 实际响应、凭据与状态目录边界，结果也留在 #335 | 未完成 Agency→真实 Harness→结果准入全链；未实跑 Linux 或 macOS x86_64；未证明隔离验收全过 |
| 替代方案 | 确认候选提供相关能力，列出接线与失败测试 | 未写迁移接线、SQL 分页、schema 生成或 statfs 替换代码；未改 lock / spec |

报告交付检查：`git diff --check` 通过；`cd src && ./buck2 test root//build/docs/...` 为 14 项通过、0 失败。文档检查不核外部功能，也不等于候选接线已经可用。

Herdr 0.9.3 的原生事件没有数字退出码，订阅事件也可能丢失；这些事实值得保留，但不自动推出另写引擎。所有者已暂缓两项接口提案。如以后恢复，上游当前[贡献规矩](https://github.com/herdrdev/herdr/blob/e35f3937b0efe40ec0dab675709c68e1d8e8c9e6/CONTRIBUTING.md)要求未获准贡献者先走 issue / discussion，而不是直接提交实现 PR；本次没有对外提需求。

## 五、替换收益与代价前五项

排序考虑减少长期维护与取得社区能力，不按行数。C1 和 C2 同属 3c 的纠偏，可在同一包完成，不重复计算收益。

| 顺序 | 项 | 收益 | 代价与风险 | 推荐节奏 |
| --- | --- | --- | --- | --- |
| 1 | C1：真实 Harness 仍归 Herdr | 避免养第二个执行引擎，继续取得上游的 Harness 支持与终端控制 | 3c 需返工；结果观察与状态目录权限仍要诚实验证 | 当前 3c 内落实，方向已定 |
| 2 | C5：迁移复用既有 SDK | 低成本去掉后续数据库迁移自建路线，事务与版本检查复用成熟实现 | 旧数据库与半初始化兼容测试；不能耦合 control 数据库 | 汇总拍板后单独小包 |
| 3 | C3：Herdr 类型来自官方 schema | 减少升级时手写字段漂移，现有研究能变成可验实现 | 生成是否支持需先实验；全量类型体积可能不合算 | 与 0.9.3 接入核对，实验失败用官方类型 |
| 4 | C2：不以交互 shell 标记作正式协议 | 消掉回显/历史替换补过滤的反复工作，利用原生启动 | 测试改写；原生启动不解决所有结果语义 | 与 C1 同包，不再叠过滤 |
| 5 | C6：SQLite 先限量取结果 | 不增加依赖，避免每页都装载全量正文 | 续取顺序、并发追加、字节上限与 complete 的回归测试 | 有结果规模用例的小包，先量测 |

C7 是低成本平台适配收束，可随 tool 维护做；C4 潜在收益大，但独立 Agency 的随包与登录环境代价没有验证，不挤进立即替换前五项。

整体上，五个范围已经借用了主要基础设施的 SDK / 二进制，未发现自行实现 Git、RPC 栈、摘要算法、JSON 规范化或文件锁。最严重的偏离发生在还未合入的真实 Harness 路径，已经被所有者纠正。接下来应把已有社区组件用深：Herdr 管执行，SQLite 管查询，迁移库管版本；HCTL 只留授权、冻结、准入与必要适配。Participant / Context 的代码多不能单独证明重复造轮子，也不能借「它是治理」免查底下的通用机制。

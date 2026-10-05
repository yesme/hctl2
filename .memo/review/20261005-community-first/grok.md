# 控制面与命令行：自建方案盘点

> 状态：讨论中
> 基线：main @ f531ca195b4c2c96c4a5279acff516360dc64f05（草案 v0.19.2）
> 去向：.memo/design/community-first-audit-20261005.md §六

范围是任务书 §三 的 Grok 一行：`src/apps/control`（不含托管服务生命周期与密钥后端）、`src/apps/cli`，以及 `src/crates/` 下的 `store`、`project`、`task`、`chat`、`repo`、`facts`、`foundation`、`proto`。对照的设计是 `spec/system.md`、`project.md`、`task.md`、`repo.md`、`connections.md`。先读了 [部件矩阵](../../../docs/research/component-matrix-20260902.md)（2026-09-02，文末 2026-09-04 实现复核）、[`docs/research/libs/`](../../../docs/research/libs/README.md)、[`sdk/matrix.md`](../../../docs/research/sdk/matrix.md)、[`sdk/git.md`](../../../docs/research/sdk/git.md)、[`sdk/github.md`](../../../docs/research/sdk/github.md)、[`libs/protobuf-rpc.md`](../../../docs/research/libs/protobuf-rpc.md)、[`libs/sqlite-migrations.md`](../../../docs/research/libs/sqlite-migrations.md)、[`gitea.md`](../../../docs/research/gitea.md)。已经论证过的只核「当时的理由现在还成立吗」。社区链接、版本、许可和能力都在下面打开过；没打开的标印象。

这一块的通用机制大部分已经在尺子的上两级。下面每条是仍由第一方代码承担的通用部件。产品对象本身在文末短清单里。

## 子进程的时限、终止与输出上限

- **位置**：`src/crates/repo/src/git.rs` 的 `run`（约 85 行，文件 485 行），`src/apps/control/src/scm.rs` 与 `src/apps/control/src/tasks/github.rs` 调用它。`src/crates/store/src/materials.rs` 另有一份 `run`（约 30 行）和两处直接 `.output()`（`export` / `import`）。`src/crates/facts/src/lib.rs` 的 `gh` / `ps` 与 `src/apps/control/src/agency.rs` 的 `agency start` 也是无时限的 `Command::output`。
- **自建了什么**：自己轮询 `try_wait`、到点 `Child::kill`、用线程把标准输出截到 16 MiB，超时记成 `RESULT_UNKNOWN`。材料库那份先写完标准输入再 `wait_with_output`，没有时限，也没有字节上限。类 2；材料库与 `repo::git::run` 并排两份，也是类 1。
- **它解决什么问题**：卡住的 `git` / `gh` / `tea` 不能无限占住单写者；超时不能被当成「写入已经失败」。
- **社区方案**：[process_control 5.2.0](https://crates.io/crates/process_control)（2025-09-06 发布，此后无更新版本；MIT OR Apache-2.0；仓库 [dylni/process_control](https://github.com/dylni/process_control)；文档 [docs.rs 5.2.0](https://docs.rs/process_control/5.2.0/process_control/)）。`time_limit` + `terminate_for_timeout` 在超时后终止，并写明要避开进程号被系统复用后误杀；`wait` 超时返回 `Ok(None)`；`stdout_filter` / `stderr_filter` 可按块丢弃输出。`memory_limit` 只在 Android、Windows、Linux（gnu/musl）上提供，macOS 没有。对照：[wait-timeout 0.2.1](https://crates.io/crates/wait-timeout)（crates.io 记录 2025-02-03，MIT OR Apache-2.0）被 process_control 的文档写成不能自动终止、也没有边等边读输出的接口。[duct 1.1.2](https://docs.rs/duct/1.1.2/duct/struct.Expression.html)（2026-09-03，MIT）的 `Expression` 方法列表读到 `before_spawn`，没有时限。标准库 [Child（Rust 1.98.0）](https://doc.rust-lang.org/1.98.0/std/process/struct.Child.html) 有 `kill`、`try_wait`、`wait`、`wait_with_output`，没有时限。我们已在用标准库 `Child`，没有用 process_control。
- **当初为什么自建**：没有找到记录。`sdk/git.md` 定的是宿主 `git` 二进制，没有讨论子进程时限。
- **换过去要动什么**：`repo`、`store`、`facts` 加依赖；`repo::git::run`、`Materials::run`、材料库的 `export`/`import`、`facts` 的 `gh`/`ps`、`agency start` 收成同一个辅助函数。超时仍映射成现在的 `RESULT_UNKNOWN` 或材料不可读，不能当成写入失败。16 MiB 用 filter 计数后丢弃，不用 `memory_limit`。process_control 的 `wait` 会先关掉标准输入，材料库已经是「写完再等」，`github.rs` 的请求体也走这条；要保持这个顺序。风险：本报告没有在 macOS 或 Linux 上编译过这个 crate；它一年没有新版；filter 只丢弃字节，失控进程要靠时限结束。
- **建议**：换。现在是自研（标准库 `Child` 上的轮询和按进程号 `kill`）。换完到 SDK，结果分类和 16 MiB 计数仍留在我们这边。
- **把握**：查证（上面的 crates.io 与文档）。没有实测。

## 在官方 `gh` 旁边解析 HTTP 头、做条件请求和分页

- **位置**：`src/apps/control/src/tasks/github.rs`（157 行）解析 `gh api --include` 的状态行、`etag`、`retry-after`、`x-ratelimit-*`，并做内存条件请求缓存。`src/apps/control/src/tasks/provider.rs` 的 `list`（约 37 行）用 `page` / `per_page` / `limit` 自己翻页，重复页记 `SOURCE_INCOMPLETE`，最多 1000 页。`src/crates/facts/src/lib.rs` 的 CI 与引用读取用 `gh api --paginate --slurp`，拉取请求合并用 `gh pr view --json`。发行锁里的 `gh` 是 2.99.0（`src/packaging/dependencies/lock.json`）。
- **自建了什么**：官方二进制已经发出带头发的 HTTP 响应之后，自己切状态行和头，并自己翻页。类 1。2026-09-03 的 [`sdk/github.md`](../../../docs/research/sdk/github.md) 曾建议控制面用 octocrab，所以也是类 5。
- **它解决什么问题**：GitHub 的 304、404、429 和二级限额要分成「可重读」「平台拒绝」「结果未知」，写操作不能在限额下自动重发。翻页要在 GitHub 与 Gitea 上共用一个上限，重复页不能被当成看板已经读完。
- **社区方案**：官方二进制 [gh 2.102.0](https://github.com/cli/cli/releases/tag/v2.102.0)（2026-09-30，MIT；我们钉的是 2.99.0）。本次打开的 [gh api 手册](https://cli.github.com/manual/gh_api) 有 `--include`、`--paginate`、`--slurp`、`--cache <duration>`。`--cache` 是按时间的本地缓存，手册没有写它会代发 `If-None-Match` 或代读 `x-ratelimit-reset`。事实标准库 [octocrab 0.54.2](https://crates.io/crates/octocrab)（2026-09-14，MIT OR Apache-2.0）仍有 [etag 模块](https://docs.rs/octocrab/0.54.2/octocrab/etag/index.html)。[backoff 0.4.0](https://crates.io/crates/backoff) 停在 2021-12-14，不读 GitHub 的限额头。Gitea 侧已经是官方 [tea 0.15.1](https://gitea.com/gitea/tea/releases/tag/v0.15.1)（标签 `f34697c5ed`，2026-08-02；[v0.15.1 的 LICENSE](https://gitea.com/api/v1/repos/gitea/tea/contents/LICENSE?ref=v0.15.1) 是 MIT）。我们已在用 `gh` 和 `tea`，没有用 octocrab。
- **当初为什么自建**：[`sdk/github.md`](../../../docs/research/sdk/github.md) 把控制面定为 octocrab（GitHub App、ETag、类型化 webhook），把 `hctl2-tool wait` 定为 `gh`。落地后的控制面与事实读取都走钉定的 `gh`：`github.rs` 写明凭据留在 `gh` 自己的存储里。webhook 与 App 安装身份没有在本范围的代码里出现。二进制高于 SDK，这条落地比当时的 SDK 建议更靠上。`gh` 手册仍不代做条件请求和限额分类，所以头解析还在。
- **换过去要动什么**：改成 octocrab 要换掉 `github.rs`、`provider.rs` 的 GitHub 分支、`facts` 的三处读取，并自己保管令牌；`tea` 那一侧的翻页与「重复页即不完整」仍要留。只把翻页改成 `gh api --paginate` 会拆掉与 tea 共用的上限，也会把重复页收成一份看似完整的数组。风险：降到 SDK 之后，写操作的「限额下不自动重发」仍要自己写。
- **建议**：留。现在是跨平台二进制加胶水。换到 octocrab 会降到 SDK，策略层还在。`gh` 2.99.0 到 2.102.0 的安全修复见范围外，那是发行锁的版本，不是把这层胶水换掉。
- **把握**：查证（手册、crates.io、octocrab etag 模块、tea 发布页与 LICENSE API）。没有跑 `gh` 看 `--cache` 是否暗中做 ETag。

## Matrix 请求的传输

- **位置**：`src/apps/control/src/chat/matrix.rs`（864 行）里 `request` 约 80 行：ruma 的 `try_into_http_request_with_identity` 之后用 reqwest 发送，响应截到 8 MiB，再 `try_from_http_response`。`src/apps/control/src/chat/inbox.rs` 用 axum 接收 `PUT /_matrix/app/v1/transactions/{txn_id}`，正文交给 ruma 的 `push_events::v1::Request`。注册文件用 ruma 的 `Registration`，JSON 当作 YAML 的子集写出（`chat/config.rs`）。依赖钉在 `src/Cargo.toml`：ruma `=0.16.0`（`client-api-c`、`appservice-api-s`、`events`、`rand`），reqwest `=0.13.4`。
- **自建了什么**：类型库与 HTTP 库之间的那层发送、长度上限和本机地址检查。类 5。
- **它解决什么问题**：以 AppService 身份对回环上的 Tuwunel 发请求、收事务，并拒绝过大的响应。
- **社区方案**：[matrix-sdk 0.19.1](https://crates.io/crates/matrix-sdk)（2026-09-18，Apache-2.0）。本次打开的 [main 分支 `crates/`](https://github.com/matrix-org/matrix-rust-sdk/tree/main/crates) 仍是 base、common、crypto、sqlite、indexeddb、ui、search、qrcode、contentscanner、store-encryption 和主 crate，没有 appservice crate。2023 年移除 appservice 的记录仍是 [matrix-org/matrix-rust-sdk#2509](https://github.com/matrix-org/matrix-rust-sdk/pull/2509)。ruma 仍是类型层，不带 HTTP 客户端；这与 [`sdk/matrix.md`](../../../docs/research/sdk/matrix.md) 一致。我们已在用 ruma 与 reqwest。
- **当初为什么自建**：[`sdk/matrix.md`](../../../docs/research/sdk/matrix.md) 与部件矩阵的 provider 行：官方没有 AppService Rust SDK，类型用 ruma，发送用 reqwest，接收用 axum。
- **换过去要动什么**：换 matrix-sdk 要改 `matrix.rs`、`inbox.rs`、注册与虚拟用户路径，并带上它的存储与加密客户端。它仍然没有事务接收端。风险：调用面对不上，包变大。
- **建议**：留。现在是 SDK（ruma）加 HTTP 库。没有更上一级的 AppService 二进制或 SDK 可换。去上游提「恢复 appservice crate」的需求与当前调用面不成比例。
- **把握**：查证（crates.io 版本与 main 的 `crates/` 目录）。没有对照 0.19.1 标签再列一次目录；main 晚于该版本。

## 同一事务里的 outbox 与 inbox

- **位置**：`src/crates/store/src/command.rs`（624 行）与 `schema.rs`（213 行，schema 版本 4）。意图、幂等结果、inbox、outbox 在同一次 SQLite 事务里写。`src/crates/store/README.md` 写明 `begin_effect` 在发送前把状态记成 `unknown`，重启后只回读。
- **自建了什么**：一张 outbox / inbox，加上「结果未知不盲重发」。类 5。
- **它解决什么问题**：外部副作用先落在控制面存储里，和命令、事件同一事务；超时之后不能再发一次。
- **社区方案**：[effectum 0.7.0](https://crates.io/crates/effectum)（2024-07-23 后无新版，MIT OR Apache-2.0）。本次打开的 [README](https://github.com/dimfeld/effectum) 用 `Queue::new(路径)` 打开它自己的 SQLite 文件；路线图「Later」仍是「用 outbox 模式和队列通信的帮助函数」，还没有放进调用方的事务。[apalis-sqlite 1.0.0-rc.10](https://docs.rs/apalis-sqlite/1.0.0-rc.10/apalis_sqlite/)（2026-10-02，MIT）用 sqlx 的 `SqlitePool` 和 `SqliteStorage::setup` 建自己的表，仍是 rc，不能并进我们这份 rusqlite 事务。我们没有用这两个库。迁移库 `rusqlite_migration 2.6.0` 管的是表结构，不管投递。
- **当初为什么自建**：部件矩阵表 D 与 §二：候选都不在同一本控制面存储的同一事务里。
- **换过去要动什么**：命令提交要改成「先提交我们的事务，再往另一份 SQLite 入队」，崩溃窗口会落在两份库之间。`unknown` 只回读、冲突范围、取消尚未发送的动作都要重写。风险：把现在的单事务边界拆开。
- **建议**：留。当时的理由还在。现在是自研里的薄表（借鉴 transactional outbox 这个想法）。没有上一级可换。
- **把握**：查证（effectum README 与 apalis-sqlite 文档）。没有把它们链进本仓库编译。

## 宿主 git，而不是链进另一个 Git 实现

- **位置**：`src/crates/foundation/src/git.rs`（82 行）发现宿主 `git`、去掉 `GIT_*` 环境、要求 2.39 以上。`src/crates/repo/src/git.rs` 与 `src/crates/store/src/materials.rs`（336 行）用它做检查、裸库和 `refs/hctl2/materials/`。材料引用不带 Git 对象编号。
- **自建了什么**：对宿主 git 的环境清理和版本下限，加上材料库自己的 ref 布局。类 5。
- **它解决什么问题**：现场和 Harness、人、CI 用同一个 git；材料字节有耐久 ref，领域引用不暴露对象编号。
- **社区方案**：[gix 0.88.0](https://crates.io/crates/gix)（2026-09-25，MIT OR Apache-2.0）。本次打开的 [crate-status.md](https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md) 把 clone、fetch、commit、status、merge-base、低级 checkout 列在已有 plumbing；checkout、merge、rebase、push、钩子仍列在还缺的流程里。工作树一节同时写了打开带 worktree 的仓库，以及创建、移动、删除、修复链接工作树，正文没有给后三项打「未做」。[git2 0.21 与 libgit2](../../../docs/research/sdk/git.md) 的取舍在 2026-09-04 的调研里，本次没有重开 libgit2 发布页。我们已在用宿主 git。
- **当初为什么自建**：[`sdk/git.md`](../../../docs/research/sdk/git.md) 与部件矩阵：同一批文件上不能并排两个 Git 引擎；libgit2 不跑钩子、LFS 不内建；当时的 gix 还不能创建和删除链接工作树。
- **换过去要动什么**：`repo` 的 worktree、合并与回读，以及材料库的 `init --bare`、`hash-object`、`update-ref`、`clone --mirror`。风险：材料库即使用 gix 做得到裸库读写，现场检查仍会和宿主 git 分成两个引擎。
- **建议**：留。同一现场只用宿主 git 这条理由还在，不依赖「gix 能不能创建工作树」的细节。现在是跨平台二进制。换 gix 会降到 SDK，并在现场旁边加第二个引擎。
- **把握**：查证（crate-status.md 的总览与工作树节、crates.io 上的 gix 0.88.0）。「创建链接工作树是否已经能用」没有逐函数对源码，这一句是印象。

## 备份清单 `hctl2.control-backup.v1`

- **位置**：`src/crates/store/src/backup.rs`（251 行）。快照用 foundation 的 Online Backup，材料用 `git clone --mirror`，最后写 `manifest.json`（格式名、control 身份、schema、写者代次、事件序号、数据库 SHA-256、承诺材料数）。
- **自建了什么**：把一份 SQLite 快照和一份材料 Git 镜像收成一个目录的清单。类 3。快照本身已经是 SDK，这条只核清单。
- **它解决什么问题**：恢复前能核对身份、schema、字节摘要和承诺材料是否齐全。
- **社区方案**：SQLite Online Backup 已按 [`libs/sqlite-online-backup.md`](../../../docs/research/libs/sqlite-online-backup.md) 接上。本次打开的 [Litestream v0.5.17](https://github.com/benbjohnson/litestream/releases/tag/v0.5.17)（发布页标为 Latest；Apache-2.0 见部件矩阵，本次未重读许可证文件）仍是把 WAL 复制到对象存储的独立进程，并带实验性的只读 VFS。它不生成「控制面快照 + 材料镜像 + 身份核对」这种可搬走的目录。我们没有用 Litestream。
- **当初为什么自建**：部件矩阵表 D：快照用 SQLite 自己的 API，清单与校验是胶水；Litestream 是多机复制。
- **换过去要动什么**：备份与恢复的目录布局、`verify_backup`、已有备份的读取。风险：Litestream 解决的是另一件事，换过去会多一个进程，并丢掉材料镜像和身份核对。
- **建议**：留。现在是 SDK（Online Backup 与宿主 git）加一份本库清单。没有现成格式同时表达这两份字节和我们的身份核对。
- **把握**：查证（Litestream v0.5.17 发布说明、store 的 `backup.rs`）。没有安装 Litestream。

## 聊天观察缓存自己建表

- **位置**：`src/apps/control/src/chat/inbox.rs`（142 行）。`cache/chat-inbox.sqlite` 用 `CREATE TABLE IF NOT EXISTS` 建 `transactions`、`observations`、`health` 三张表，没有 `user_version`。
- **自建了什么**：在已经采用的 `rusqlite_migration` 旁边，给这份缓存写了无版本的建表语句。类 1。
- **它解决什么问题**：按 Matrix 事务 ID 去重，并记下房间是否读到过。文件头写明这里的观察不成为治理命令。
- **社区方案**：[rusqlite_migration 2.6.0](https://crates.io/crates/rusqlite_migration)（2026-05-28，仍是 crates.io 上的最新稳定版，Apache-2.0）已经用于控制面存储。[refinery 0.10.0](https://crates.io/crates/refinery)（2026-10-02，MIT）是另一套迁移库；[`libs/sqlite-migrations.md`](../../../docs/research/libs/sqlite-migrations.md) 当时因为 refinery 0.9.2 吃不了 rusqlite 0.40 而选了 rusqlite_migration。本次没有重读 refinery 0.10 的依赖范围，所以不把它当成可以换过去的理由。
- **当初为什么自建**：没有找到记录。控制面存储的迁移有调研；这份缓存没有单独的调研。
- **换过去要动什么**：给缓存加上 `user_version` 和迁移列表。旧文件可以删掉重建，因为去重状态可以从 homeserver 再投递。风险很小，收益也小。
- **建议**：留。缓存可以整文件丢掉再收事务；迁移库要守的是控制面存储那种不能丢的升级。现在控制面存储已经是 SDK，这份缓存是自研的三张表。
- **把握**：查证（`inbox.rs`、crates.io 上两个迁移库的版本）。refinery 0.10 与 rusqlite 0.40 是否已经相容：印象。

## 拉起本机的 control 与 agency

- **位置**：`src/apps/cli/src/main.rs` 的 `start_daemon` / `stop_daemon`（约 60 行）：启动同目录的 `hctl2-control`，轮询 `status`，最长约两分钟；停止时先 `services.stop`，再按 `control.pid` 调 `kill`。`src/apps/control/src/agency.rs` 的 `ensure_local`（约 40 行）在本机 agency 没有应答时执行 `agency start` 并等它退出。`src/apps/control/src/socket.rs`（36 行）把 `control.sock` 绑成 `0600`，活连接视为占用，残留文件删掉再绑。
- **自建了什么**：启动兄弟二进制并等待它的 RPC 或退出码。类 2。
- **它解决什么问题**：`hctl2 start` 要等控制面自己的存储可服务；本机 agency 要在配对前已经起来。套接字只给同一用户。
- **社区方案**：Process Compose 1.122.0 已经在发行包里管托管服务（部件矩阵与 `src/packaging/dependencies/lock.json`）。把它再用来管 control，会让监督者位于控制面之下。systemd 与 launchd 是两套系统单位，不是一个跨平台二进制。tonic 的 Unix 套接字接入已在 [`libs/protobuf-rpc.md`](../../../docs/research/libs/protobuf-rpc.md) 写明：权限与套接字生命周期留在我们这边。我们已用 tonic，没有用 Process Compose 来拉 control。
- **当初为什么自建**：P2.1 乙把 control 定成唯一命令进程，CLI 只是客户端。套接字模式在 protobuf 调研里留过。
- **换过去要动什么**：CLI 的启动停止、`control.pid`、agency 的 `ensure_local`，以及「存储未就绪时不要杀掉已经在应答的进程」这条行为。风险：和托管服务的监督器缠在一起；那一块不在本行。
- **建议**：留。现在是自研的薄启动。`agency start` 没有时限，若上一节的 process_control 辅助函数落地，这个调用可以走它；这不需要另请一个监督器。
- **把握**：查证（本树代码与 protobuf 调研）。Process Compose 能否把 control 当成它的一个进程：印象，没有读它现在的配置试验这条。

## 该自建

这些留在第一方，每条一个理由。

- 命令信封（幂等键、预期版本、actor 来源）：五类来源是我们的准入规则，通用幂等库表达不了「客户端不能自己声明身份」。
- 同事务 outbox / inbox：见上节，候选库都用自己的数据库文件。
- 六个代次槽：失权范围是按我们的资源划的，分布式租约服务会多一个网络服务。
- `project`、`task`、`chat`、`repo` 里的命令归约：Project、Task、Room、Repo 的状态规则就是这个产品要做的事。
- 恢复时核对未完成的外部动作：重启后只回读 `unknown`，这是命令正确性，不是通用任务队列的重试。
- 材料引用不带 Git 对象编号：引用要在换存储后端时还成立，git 只是现在的字节柜。
- Unix 对等用户变成 `local-owner:{uid}`（`src/apps/control/src/identity.rs`，14 行）：身份来自套接字对等凭据，不能由调用方填。
- JCS 之上的整数子集（`foundation` 的 `validate_canonical_numbers`，约 25 行）：[serde_json_canonicalizer 0.3.2](https://docs.rs/serde_json_canonicalizer/0.3.2/serde_json_canonicalizer/)（MIT，crates.io 上仍是 2026-02-03 的最新版）按 RFC 8785 把超出双精度的数收成 double。我们的摘要拒绝非整数和超出安全整数的数。库继续用，这层检查留着。
- `control.sock` 的 `0600` 与残留文件处理：tonic 负责 RPC，不负责这个文件的权限。

## 范围外

顺手看到、不在本行下结论的：

- `src/apps/control/src/services.rs`（约 1093 行）和 `scm.rs` 里 Gitea 引导的 `retry_bootstrap`（约 20 行）是托管服务生命周期，归 DeepSeek。
- `foundation` 的 `SecretStore` 与 `control.json` 的 `secret_backend` 是密钥后端，归 DeepSeek。本行只看到配置是一个 JSON 字段。
- 发行锁里的 `gh` 是 2.99.0，上游最新是 [2.102.0](https://github.com/cli/cli/releases/tag/v2.102.0)（2026-09-30），说明里是四个安全修复，涉及 `gh release download`、`gh run download`、`gh repo read-file`、`gh attestation` 和 `gh skill`。本行调用的是 `gh api` 与 `gh pr view`。升级归发行锁。
- `src/agency` 里在已经采用的 Herdr 旁边重做进程与输出，是任务书的起因，归 Codex。本行的 `control/src/agency.rs` 只是配对和拉起本机 `agency` 二进制。

## 前五（好处相对代价）

1. **子进程时限与终止，换 process_control。** 好处是单写者不再被无时限的 `git` 挂住，超时终止也不按可复用的进程号杀。代价是一个一年没发版的库、标准输入必须先写完、超时语义和 16 MiB 计数还要自己留，并且要在 macOS 与 Linux 上先测再合。这是本行唯一建议换的一条。
2. **GitHub 头解析与翻页，不换 octocrab。** 删掉约 150 行胶水的代价是从官方二进制降到 SDK，还要自己保管令牌；限额下不重发、重复页即不完整这些规则还在。
3. **聊天观察缓存，不改用迁移库。** 三张表没有版本。收益是以后改列时不用删文件；文件本来就可以删，homeserver 会再投递。
4. **备份清单，不换 Litestream。** Litestream 复制的是另一个库的 WAL，不包含材料 Git 镜像，也不核对 control 身份。
5. **control / agency 的启动，不交给 Process Compose。** 现在的轮询只等我们自己的 `status`。换监督器会碰到托管服务那一块，而且 `agency start` 缺的是时限，不是另一套监督器。

## 这一块借用到什么程度

控制面与命令行里，部件矩阵点过名的通用机制已经落在二进制或 SDK 上：RFC 8785 用 `serde_json_canonicalizer` 0.3.2，文件锁用标准库 `File::try_lock`，SQLite 快照用 Online Backup，schema 用 `rusqlite_migration` 2.6.0，全文用 FTS5，RPC 用 prost 0.14.4 / tonic 0.14.6 / pbjson 和钉定的 protoc，命令行用 clap 4.6.6，Matrix 用 ruma 0.16.0 加 reqwest 0.13.4，Git 用宿主 git，GitHub 用钉定的 gh 2.99.0，Gitea 用 gitea 1.27.3 与 tea 0.15.1，事实监听用 notify 8.2.0，摘要用 sha2。自研剩下的是命令内核、代次、治理归约、材料引用和套接字权限，加上上面那一层子进程时限。建议换的只有这层时限。

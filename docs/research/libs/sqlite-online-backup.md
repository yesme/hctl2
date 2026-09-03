# SQLite Online Backup API · 账本一致备份

> 状态：调研 · 日期：2026-09-03<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-LIB-SQLITE-BACKUP<br>
> 对象：SQLite Online Backup API（`sqlite3_backup_init / step / finish`），经 [`rusqlite` 0.40.2](https://crates.io/crates/rusqlite/0.40.2)（2026-08-08 发布）的 `backup` feature 使用；`bundled` feature 自带 SQLite 3.53.2<br>
> 许可证：SQLite 公有领域；rusqlite 与 libsqlite3-sys MIT

## 定位

我们用它做[系统边界 §备份与恢复](../../design/spec/system.md#备份与恢复)要求的「一致备份集」里的账本快照那一步：在 control 仍在写账本的情况下拿到一份事务一致的数据库副本。备份集的其余部分——定义字节、摘要、schema 可读性校验、清单——是我们自己的胶水，本文不管。它替代的手写代码是「停写、`cp` 数据库文件、恢复写」或者更糟的「不停写直接 `cp`」。

## 上游能力

- **API 语义**：`sqlite3_backup_init` 在源、目标两个连接之间建立备份句柄；`sqlite3_backup_step(N)` 拷 N 页（N = −1 全拷）；`finish` 释放。完成后目标是源库「开始拷贝那一刻」的逐位相同快照。每次 `step` 只在调用期间持有源库共享锁；首次 `step` 拿目标库排他锁，目标连接在 `init` 与 `finish` 之间不能被任何其他 API 使用；目标上已有读或写事务时 `init` 直接失败。
- **写入进行中的一致性**：分步备份期间，如果源库被**同一个连接**修改，目标库同步得到修改，备份继续；如果被**其他连接或其他进程**修改，下一次 `step` 自动从头重来。单次 `step(-1)` 在一次共享锁内拷完，不存在中途被改的窗口。返回 `SQLITE_LOCKED` 表示源连接正在写，`SQLITE_BUSY` 表示拿不到文件锁，两者都可重试。
- **rusqlite 封装**：`backup::Backup::new(&src, &mut dst)`、`step(n)`、`run_to_completion(pages_per_step, sleep, progress)`；便捷方法 `Connection::backup(name, path, progress)` 与 `Connection::restore(name, path, progress)`。`StepResult` 四个值：`Done / More / Busy / Locked`。文档同时指出 `VACUUM INTO` 是更简单的替代。
- 依赖树（`bundled` + `backup`，rustc 1.98.0）：三平台各 9 个 crate；bundled 编译期打开 FTS5、JSON1、RTREE、`SQLITE_THREADSAFE=1`、`SQLITE_USE_URI` 等。rusqlite 下载 1.01 亿，近 90 天 3,196 万。
- **VACUUM INTO**（SQLite 3.27.0，2019-02-07 加入）：把源库当前状态压实写到一个不存在或为空的文件里，输出是「源库的一致快照」；比备份 API 多耗 CPU，但输出更小、已删除内容被清除；默认不 `fsync`，源库 `PRAGMA synchronous` 为 NORMAL 或 FULL 时才同步到盘；目标可以是 URI 文件名。只需一个连接。
- **sqlite3_rsync**（3.47.0，2024-10-21 起随 SQLite 发布）：独立命令行工具，面向 SSH 场景复制活库。**Litestream** v0.5.17：独立进程，把 WAL 流式复制到对象存储。两者都是多机阶段的超集，本轮不用。

## 候选比较

| 候选 | 一致性保证 | 需要什么 | 写入进行中 | 输出 | 适合 |
| --- | --- | --- | --- | --- | --- |
| Online Backup API（rusqlite `backup`） | 逐位快照，事务一致 | 两个连接，目标连接独占 | 同连接改动被同步；他连接改动触发重来；`step(-1)` 无窗口 | 与源同页大小的原样副本 | 库内定时快照、进度回调 |
| `VACUUM INTO` | 一致快照 | 一个连接、一个空目标路径 | 读事务内完成 | 压实副本，默认不 fsync | 导出、瘦身副本 |
| `sqlite3_rsync` | 一致快照 | 独立二进制，通常配 SSH | 支持活库 | 远端副本 | 多机复制 |
| Litestream | WAL 连续复制 | 独立进程 + 对象存储 | 持续 | 远端 WAL 流 | 多机、异地 |
| 直接 `cp` / `tar` | 无 | — | 事务中拷贝得到损坏副本；WAL 库漏拷 `-wal` 会丢已提交事务 | — | 不适合 |

## 边界与取舍

- **不能直接拷文件**。SQLite 的 howtocorrupt §1.2 明确：后台备份程序在事务进行中拷数据库文件会得到损坏副本；上一次写事务失败时 `-journal` 或 `-wal` 必须与主文件一起拷。WAL 模式下已提交但未 checkpoint 的事务只在 `-wal` 里，漏拷即丢。备份 API 与 `VACUUM INTO` 都经由 SQLite 读路径，天然把 WAL 内容算进去，所以快照不手写文件复制。
- **WAL 模式下的注意点**：
  1. 目标库若已是 WAL 模式且页大小与源不同，`step` 返回 `SQLITE_READONLY`；目标为内存库时同样。做法：目标永远是新建的空文件，或先对齐 `PRAGMA page_size`。
  2. WAL 下读者不阻塞写者、写者不阻塞读者，所以 `step(-1)` 单次拷完既拿到一致快照，又不会让 control 的提交等待；回滚日志模式下同样的共享锁会挡住写者提交。这是账本选 WAL 的又一个理由。
  3. 备份进行中的长读事务会让 checkpoint 无法越过该读者的快照点，WAL 文件暂时增长；账本很小，可接受。
  4. 备份产物打开后的 journal 模式与 `-wal` 伴随文件的处理，上游文档未明确写出——在契约测试里显式 `PRAGMA journal_mode` 检查并按需设为 `DELETE`，让「一个文件就是完整快照」成为被测事实，而不是假设。
- **谁来发起备份**：control 是唯一写者。若备份用 control 自己的写连接，其间的写入会被同步进目标，备份不会重来但快照点变成「完成时刻」而不是「开始时刻」；若用独立只读连接加 `step(-1)`，快照点就是开始时刻且不受写入干扰。后者语义更干净，建议采用。
- **恢复**：不能把备份文件覆盖到活库上。恢复只在 control 停写、持有单写者锁、按约束重置 `control_writer_generation` 之后进行（`restore preview / apply` 两步，见约束层）。
- **备份集的一致性是我们的责任**：快照完成后用只读连接打开它，跑 `PRAGMA integrity_check`、读 `sqlite_schema`、算文件 SHA-256，连同定义字节与摘要写进清单——这些是胶水，不是库能替我们做的。

## 决定建议

**采用 SDK**：rusqlite 0.40.2 的 `backup` feature 调 SQLite 自带的 Online Backup API，用独立只读连接 + `step(-1)` 做账本快照；`VACUUM INTO` 作为等价备选，用于需要压实副本的导出场景。禁止直接拷文件。`sqlite3_rsync` 与 Litestream 仅参考行为，多机阶段再评估。契约测试至少钉三条：写入进行中备份产物 `integrity_check` 通过且包含备份开始前已提交的全部事务；WAL 库的备份产物不依赖 `-wal` 伴随文件；恢复只在停写状态下被接受。

## 复核记录：bundled 策略裁定（2026-09-04，issue #158）

**裁定：维持 `bundled` 为终态，关闭替代方案搜索；本节同时否决换用 RocksDB 的提议。**

替代候选逐条对照 #158 的五条退出条件（离线安装 / 四平台能力一致 / 来源摘要可锁 / 无需用户手工安装 / 实际降低构建维护成本）：

| 候选 | 判定 | 依据 |
| --- | --- | --- |
| SQLite 官方预编译链接库 | 不存在 | 官方下载页只发 CLI 工具与 Windows DLL；SQLite 的官方分发哲学是 amalgamation 自编译——编译期开关组合由使用方决定 |
| 第三方预编译 | 违反供应链纪律 | 无权威来源；编译开关组合无法验证，仍需自建能力探测；安全修复要等第三方重编；严格劣于 bundled |
| 系统库 | 三处硬伤 | 版本不可钉（macOS 随 OS、Linux 随发行版）；编译开关不可验证（FTS5 等各发行版不一致）；最小化系统不保证 `libsqlite3.so.0`，「无需手工安装」不成立 |
| RocksDB | 引擎与负载错配 | 见下段 |
| **bundled（amalgamation）** | **全过** | amalgamation 是官方校验和的源码形态（可锁性最高档）；三平台一个版本一套开关，能力一致由构造保证；链接进 control 二进制，用户零运行依赖；Windows 将来零新增供应链 |

**bundled 不是权宜之计**：amalgamation 就是 SQLite 的正规官方形态，Firefox/Chromium/iOS 生态与 rusqlite 自身的推荐默认都是自编译嵌入。深层理由与账本纪律同构——metadata 账本是「唯一不可再生的权威」，其存储引擎的版本与编译选项应视作一次**冻结绑定**：写库、CI、五年后 restore 的机器必须跑同一引擎版本；查询计划器与 FTS5 行为随版本漂移，等价于让部署环境定义账本行为。Tuwunel 用 RocksDB 是对的（homeserver 是流式高写入负载），control 是人尺度 OLTP + 完整性 + 审计，负载不同、选型不同。

**RocksDB 否决要点**：无查询层（每个访问模式手写索引与扫描）、无声明式跨实体事务、无 FTS5 等价物、备份产物是 SST 目录而非单文件、编译供应链问题原样保留且更糟（C++、更大、同样无官方预编译）、可审计性（doctor/export/取证拿 sqlite3 CLI 即读）全部丧失；单写者合同让它「可行」但不改变上述任何一条。

**残余成本（诚实清单）**：C/C++ 工具链进入三平台构建矩阵必经路径；SQLite 升级 = bump 版本 + 全量回归。issue 原列的两条缺点——「一次 C 编译」由 REAPI 远端缓存摊薄为一次；「fixup 维护面」不是 bundled 固有成本，是 #157（Buck2 build-script shim 需 Python ≥3.12）阻塞造成的手工复刻，由 #157 拆除。行动项全部转移至 [#157](https://github.com/yesme/hctl2/issues/157)。

## 证据

- SQLite 官方：[Online Backup API 使用指南](https://sqlite.org/backup.html)、[`sqlite3_backup_*` 参考（含 WAL 页大小与重来规则）](https://sqlite.org/c3ref/backup_finish.html)、[VACUUM INTO](https://sqlite.org/lang_vacuum.html#vacuuminto)、[3.27.0 发布记录](https://sqlite.org/releaselog/3_27_0.html)、[How To Corrupt An SQLite Database File §1.2、§1.3](https://sqlite.org/howtocorrupt.html)、[WAL 模式](https://sqlite.org/wal.html)、[sqlite3_rsync](https://sqlite.org/rsync.html)
- rusqlite：[crates.io 0.40.2](https://crates.io/crates/rusqlite/0.40.2)、[`backup` 模块源码（v0.40.2）](https://github.com/rusqlite/rusqlite/blob/v0.40.2/src/backup.rs)、[libsqlite3-sys `build.rs` 的 bundled 编译开关](https://github.com/rusqlite/rusqlite/blob/master/libsqlite3-sys/build.rs)、[v0.40.2 自带 sqlite3.h 版本号 3.53.2](https://github.com/rusqlite/rusqlite/blob/v0.40.2/libsqlite3-sys/sqlite3/sqlite3.h)
- Litestream：[v0.5.17](https://github.com/benbjohnson/litestream/releases/tag/v0.5.17)
- 本库：[系统边界 §备份与恢复](../../design/spec/system.md#备份与恢复)、[部件矩阵表 D 备份一行](../component-matrix-20260902.md#d--约束层里靠第一方代码实现的通用机制)、[契约测试 CT 备份条目](../../design/contract-tests.md)、[SQLite 官方下载页（分发形态：CLI 工具 + Windows DLL，无可链接库）](https://sqlite.org/download.html)

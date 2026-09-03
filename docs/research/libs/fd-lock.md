# fd-lock · 进程间文件锁

> 状态：调研 · 日期：2026-09-03<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-LIB-FD-LOCK<br>
> 对象：[`fd-lock` 4.0.4](https://crates.io/crates/fd-lock/4.0.4)（2025-03-10 发布）；同题对照 Rust 标准库 `std::fs::File::lock` 系列（Rust 1.89.0 起稳定）<br>
> 许可证：fd-lock MIT OR Apache-2.0；标准库 MIT OR Apache-2.0

## 定位

我们用它做两把锁：Git 工作树现场（Repo Instance）的 OS 排他锁，以及 control 账本的 `control.lock` 单写者锁——约束层要求同一现场同时只有一个执行者、同一账本同时只有一个写入者（见[系统边界 §单写者](../../design/spec/system.md#单写者)）。它替代的手写代码是「PID 文件 + `kill -0` 探活 + 陈旧锁清理」这一套：HCTL1 和当前 `src/packaging` 的 shell 里都有一份。OS 文件锁的好处只有一条，但是决定性的：持有者进程死亡（包括被 `SIGKILL`）时内核自动释放，不存在「陈旧锁文件」这个概念，也就不需要写清理逻辑。

## 上游能力

- **fd-lock 4.0.4**：`RwLock<T>` 包装任意实现 `AsFd`（Unix）或 `AsHandle`（Windows）的文件，`write() / try_write() / read() / try_read()` 返回 guard，guard 被 drop 时解锁。Unix 侧经 rustix 调 `flock`；Windows 侧调 `LockFileEx`，锁从偏移 0 起的 1 个字节，排他用 `LOCKFILE_EXCLUSIVE_LOCK`，非阻塞加 `LOCKFILE_FAIL_IMMEDIATELY`，`ERROR_LOCK_VIOLATION` 映射为 `WouldBlock`。
- 依赖树（`cargo tree`，rustc 1.98.0）：Linux x86_64 5 个 crate，macOS arm64 6 个，Windows x86_64 5 个；除 cfg-if 外全是 rustix 或 windows-sys 家族。
- 下载 6,066 万，近 90 天 1,315 万；仓库 89 star，最后提交 2025-04-23。
- 维护状态：作者在 issue #59（2025-08-14）记录「Rust 1.89 稳定了 `File::lock`，这个 crate 该怎么办」；2026-06-17 提交 PR #62「deprecation notice」准备弃用。截至本文 crates.io 尚未标记 deprecated，但方向已明。
- **标准库**：Rust 1.89.0（2025-08-07）稳定 `File::lock / lock_shared / try_lock / try_lock_shared / unlock`。Unix 用 `flock`（Solaris/illumos 用 `fcntl`），Windows 用 `LockFileEx`；锁的是整个文件；`try_lock` 返回 `TryLockError::{WouldBlock, Error(io::Error)}`；文件及其全部复制句柄关闭、显式 `unlock` 或进程终止时释放。HCTL2 钉的 rustc 1.98.0 已包含这组 API。

## 候选比较

| 候选 | 版本 / 发布日 | 许可证 | 机制 | 平台 | 依赖树 | 维护 |
| --- | --- | --- | --- | --- | --- | --- |
| `std::fs::File::lock` | Rust 1.89.0 / 2025-08-07 | MIT OR Apache-2.0 | flock / LockFileEx / fcntl（Solaris） | 三平台 + Windows | 0 | 标准库 |
| fd-lock | 4.0.4 / 2025-03-10 | MIT OR Apache-2.0 | flock（rustix）/ LockFileEx | 三平台 + Windows | 5–6 | 作者已提弃用 PR #62 |
| fs4 | 1.1.0 / 2026-04-28 | MIT OR Apache-2.0 | flock（rustix）/ LockFileEx，可选 async | 三平台 + Windows | 3–5 | fs2 的活跃分支，MSRV 1.75 |
| fs2 | 0.4.3 / 2018-01-06 | MIT/Apache-2.0 | flock（libc）/ LockFileEx | 三平台 + Windows | — | 2018 年后无发布 |
| file-lock | 2.1.11 / 2024-02-17 | MIT | fcntl POSIX 记录锁 | 仅 Unix | — | 语义不同：进程关闭该文件的任一 fd 即丢锁，fork 不继承 |
| rustix 直接 `flock` | 1.1.4 / 2026-02-22 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | flock | 仅 Unix | 4 | Windows 要另写 LockFileEx 分支 |

## 边界与取舍

三平台语义差异。fd-lock 与标准库用的是同一组系统调用，下表对两者都成立：

| 问题 | Linux / macOS（`flock`） | Windows（`LockFileEx`） |
| --- | --- | --- |
| 锁的性质 | 建议锁：不加锁的进程照样能读写该文件，只有都走 `flock` 的进程之间互斥 | 强制锁：排他锁期间其他句柄（包括本进程再 `open` 一次得到的句柄）读写都被拒 |
| 锁的归属 | open file description：`fork`/`dup` 出的 fd 共享同一把锁，`execve` 后仍持有；子进程显式解锁会连父进程一起解掉 | 句柄：被子进程继承的句柄不获得锁区访问权 |
| 持有者被 kill | 内核关闭全部 fd 时释放，`SIGKILL` 也一样 | 进程终止时由系统解锁，但微软文档写明「解锁耗时取决于系统资源」，建议退出前显式解锁 |
| 与 `fcntl` 锁 | 本地文件系统上互不干涉 | 不适用 |
| 同进程重复加锁 | 标准库文档：同一句柄或其副本已持锁时再加锁，行为未定义，可能死锁 | 同上 |

- **进程被 kill 后锁是否自动释放**：三平台都是。Linux/macOS 立即；Windows 可能滞后。这正是弃用 PID 文件的理由。契约测试写法：子进程 `try_lock` 成功后被 `SIGKILL`（Windows 用 `TerminateProcess`），父进程随即 `try_lock` 应成功——Unix 一次即成，Windows 允许有界重试。
- **NFS**：Linux 自 2.6.12 起在 NFS 客户端上把 `flock` 模拟成覆盖整个文件的 `fcntl` 字节范围锁，经锁管理协议走到服务器，因此能跨主机互斥，但要求文件以可写方式打开才能加排他锁，且与 `fcntl` 锁互相作用；`local_lock` 挂载选项（2.6.37 起）会把它退化为本机锁。CIFS 自 Linux 5.5 起用 SMB 字节范围锁模拟，并且变成强制锁。macOS 在 NFS 上的 `flock` 行为：Apple 的 flock(2) 手册只列了 `ENOTSUP`（描述符类型不对），未写 NFS 行为——未查到权威说明。结论：现场锁文件必须放本地文件系统，`doctor` 检测到工作树或账本目录在 NFS/CIFS 上时告警；不把跨主机互斥当作 OS 锁能提供的能力。
- **不要锁正在被别人读的文件**：Windows 是强制锁，锁住 SQLite 数据库文件会让 SQLite 自己的读写失败；SQLite 在 Unix 上也用自己的 `fcntl` 锁管理库文件。所以锁对象一律是专用的空 `.lock` 文件，不是账本、不是 Git 对象。
- **锁的寿命等于进程寿命**：打开锁文件的句柄在进程存活期间不能关闭；锁文件本身不删除、路径固定。
- fd-lock 相比标准库多出的只有 `RwLock<T>` 这层 guard 风格的封装；代价是一个即将被作者弃用的依赖。fs4 提供 async 变体，但 HCTL2 账本路径不引 async 运行时（见[部件矩阵表 D](../component-matrix-20260902.md#d--约束层里靠第一方代码实现的通用机制)），用不上。

## 决定建议

**采用 SDK**，且 SDK 首选标准库 `std::fs::File::try_lock`：零依赖，机制与 fd-lock 完全相同。fd-lock 4.0.4 只在需要 `RwLock<T>` guard 风格 API 时作为薄封装备选，并接受作者已启动弃用这一事实。评审裁决包里「现场锁用 fd-lock」这一句建议改读为「现场锁用 OS 文件锁 SDK：标准库 `File::lock`，fd-lock 为兼容备选」。不采用 file-lock（仅 Unix、`fcntl` 语义容易误用）和 fs2（停更）。

## 证据

- crates.io：[fd-lock 4.0.4](https://crates.io/crates/fd-lock/4.0.4)、[fs4 1.1.0](https://crates.io/crates/fs4/1.1.0)、[fs2 0.4.3](https://crates.io/crates/fs2/0.4.3)、[file-lock 2.1.11](https://crates.io/crates/file-lock/2.1.11)、[rustix 1.1.4](https://crates.io/crates/rustix/1.1.4)
- fd-lock 仓库：[Cargo.toml](https://github.com/yoshuawuyts/fd-lock/blob/main/Cargo.toml)、[Unix 实现](https://github.com/yoshuawuyts/fd-lock/blob/main/src/sys/unix/rw_lock.rs)、[Windows 实现](https://github.com/yoshuawuyts/fd-lock/blob/main/src/sys/windows/rw_lock.rs)、[issue #59 `File::lock` has been stabilized](https://github.com/yoshuawuyts/fd-lock/issues/59)、[PR #62 deprecation notice](https://github.com/yoshuawuyts/fd-lock/pull/62)
- 标准库：[`File::lock` 文档](https://doc.rust-lang.org/std/fs/struct.File.html#method.lock)、[Rust 1.89.0 发布公告](https://blog.rust-lang.org/2025/08/07/Rust-1.89.0/)
- 系统调用：[flock(2)（man7，含 NFS 与 CIFS 节）](https://man7.org/linux/man-pages/man2/flock.2.html)、[macOS flock(2)](https://keith.github.io/xcode-man-pages/flock.2.html)、[LockFileEx（Microsoft Learn）](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-lockfileex)
- 本库：[部件矩阵表 D 现场 OS 锁一行](../component-matrix-20260902.md#d--约束层里靠第一方代码实现的通用机制)

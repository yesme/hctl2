# rustix · 进程有效用户标识

> 状态：采用 · 复核：2026-09-28 · P2.2 己的完整包回归

## 定位与证据

[rustix 1.1.5 的 geteuid](https://docs.rs/rustix/1.1.5/rustix/process/fn.geteuid.html) 提供安全的原生有效用户 ID 查询；本地钉定源码 `src/process/id.rs` 与 `src/backend/libc/ugid/syscalls.rs` 已核，macOS 调 libc、Linux 使用对应系统调用。现有 Cargo.lock 已包含该版本，原有构建依赖继续共用。

## 候选与决定建议

采用并精确钉 **rustix =1.1.5**、`process` feature，替代 Supervisor 为拼 Process Compose socket 路径反复执行 `id -u`。不直接写 unsafe FFI，也不缓存可能读取失败而回退成 0 的 shell 结果。许可证 Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT。

本机完整包测试中，状态查询卡在 `Command::output(id -u)` 的读管道；进程采样与 lsof 显示，`id` 已退出，但并发启动的长驻 Process Compose / Tuwunel 持有该管道写端，读者收不到 EOF。原生查询不创建子进程或管道，消除这一具体阻塞点；不据此宣称已修复所有平台进程继承问题。验证用当前 UID 与 `id -u` 对照、完整包启停与服务回归。

## 2026-10-04 · Agency 本地 socket 属主复核

#317 使用同一 **rustix =1.1.5** 的安全 `geteuid()`，在发送凭据前核对 socket 及其父目录的属主、类型与模式。Tokio 的原生 `UnixStream::peer_cred()` 再核对已连接对端 UID，不自写系统调用；已核本机锁定 Tokio 源码 `src/net/unix/stream.rs`。这阻止别的用户抢占可预测路径，不把同一 OS 用户下的可信脚本声称为隔离沙箱。目录不合格时拒绝，而不是改权限后继续。

## 2026-10-05 · 现场锁的文件系统识别复核

`src/apps/tool/src/site_lock.rs` 改用本 crate 的 `fs` 功能（`rustix = { workspace = true, features = ["fs"] }`，`src/third-party/rust/BUCK` 由 Reindeer 再生）里的 `fs::statfs`，取代解析 `mount` 与 `stat` 的人类输出：macOS 读 `f_fstypename`（在 `forbid(unsafe_code)` 下按定长 C 串安全读取），Linux 把 `f_type` 映射回 `stat -f -c %T` 的拼写，未知魔数记 `unknown-<magic>`。保守拒绝清单与 `HCTL2_TOOL_FILESYSTEM_UNREADABLE`／`HCTL2_TOOL_FILESYSTEM_UNSUPPORTED` 错误码不变。

实证（macOS arm64）：把 HFS+ 磁盘映像挂到 `/Volumes/hctl2nested`，卷内仓库的现场锁回执报 `"filesystem": "hfs"`，宿主 APFS 上的仓库报 `"apfs"`——嵌套挂载由内核按路径作答，不再需要最长前缀匹配。Linux 侧由 `unit_test` 的魔数映射用例覆盖，两平台编译在 CI。

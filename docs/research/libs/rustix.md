# rustix · 进程有效用户标识

> 状态：采用 · 复核：2026-09-28 · P2.2 己的完整包回归

## 定位与证据

[rustix 1.1.5 的 geteuid](https://docs.rs/rustix/1.1.5/rustix/process/fn.geteuid.html) 提供安全的原生有效用户 ID 查询；本地钉定源码 `src/process/id.rs` 与 `src/backend/libc/ugid/syscalls.rs` 已核，macOS 调 libc、Linux 使用对应系统调用。现有 Cargo.lock 已包含该版本，原有构建依赖继续共用。

## 候选与决定建议

采用并精确钉 **rustix =1.1.5**、`process` feature，替代 Supervisor 为拼 Process Compose socket 路径反复执行 `id -u`。不直接写 unsafe FFI，也不缓存可能读取失败而回退成 0 的 shell 结果。许可证 Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT。

本机完整包测试中，状态查询卡在 `Command::output(id -u)` 的读管道；进程采样与 lsof 显示，`id` 已退出，但并发启动的长驻 Process Compose / Tuwunel 持有该管道写端，读者收不到 EOF。原生查询不创建子进程或管道，消除这一具体阻塞点；不据此宣称已修复所有平台进程继承问题。验证用当前 UID 与 `id -u` 对照、完整包启停与服务回归。

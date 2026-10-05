# process_control：外部命令的时限与超时终止

> 状态：调研 · 日期：2026-10-05<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-LIB-PROCESS-CONTROL<br>
> 对象：[process_control 5.2.0](https://crates.io/crates/process_control/5.2.0)（crates.io 创建时间 2025-09-06，此后无更高版本）<br>
> 许可证：MIT OR Apache-2.0（[LICENSE-MIT](https://docs.rs/crate/process_control/5.2.0/source/LICENSE-MIT)、[LICENSE-APACHE](https://docs.rs/crate/process_control/5.2.0/source/LICENSE-APACHE)）

## 定位

`git`、`gh`、`tea`、材料库、事实读取和拉起本机 `agency` 都要等一个子进程。标准库 [Child（Rust 1.98.0）](https://doc.rust-lang.org/1.98.0/std/process/struct.Child.html) 有 `kill`、`try_wait`、`wait`、`wait_with_output`，没有时限。自己轮询再 `Child::kill` 按的是进程号；进程已经退出、系统把同一个号分给别人之后，这一刀会砍错进程。本条目只覆盖这个时限和终止，不改命令超时在领域上的含义。

## 上游能力

核对的是 5.2.0 源码包（`process_control-5.2.0.crate`）和 [docs.rs 5.2.0](https://docs.rs/process_control/5.2.0/process_control/)。

- `time_limit` 加上 `terminate_for_timeout` 之后，`wait` 超时返回 `Ok(None)`，并终止该子进程。文档写明要避开进程号被复用后误杀。
- Unix 实现里，`unix_waitid` 的条件是「除了 espidf、horizon、openbsd、redox、tvos、vxworks」。macOS 与 Linux 落在这条里，走 `waitid(P_PID, …, WEXITED | WNOWAIT | WSTOPPED)`，先确认还是原来的那个进程，再终止。见源码包 `src/attr-aliases.txt` 与 `src/unix/wait/waitid.rs`。
- `stdout_filter` / `stderr_filter` 按读到的块决定要不要放进 `Output`。返回 `Ok(false)` 就丢掉这一块，不放进结果。我们用它计数，超过 16 MiB 就拒绝，不调用内存上限。
- `memory_limit` 的条件是 Android、Linux（gnu 或 musl）、Windows。macOS 没有这个方法。它限制的是虚拟地址空间（Linux 上是 `prlimit` / `RLIMIT_AS`），不是我们要的输出字节数。本库不调用它。
- `wait` 在等待前关掉标准输入。调用方要先把标准输入写完，再进入 `wait`。这与材料库和 `github.rs` 现在的顺序一致。

## 依赖

5.2.0 的 `Cargo.toml`：

| 依赖 | 范围 | 何时编进来 |
| --- | --- | --- |
| `attr_alias` | `0.1.0`（即 `^0.1.0`） | 始终。只为 `memory_limit` 等配置别名服务，本库不直接调用 |
| `libc` | `0.2.120` | Unix |
| `signal-hook` | `0.3` | 仅 espidf、horizon、openbsd、redox、tvos、vxworks |
| `parking_lot` | `0.12`，可选 | 与 `signal-hook` 同一组目标，且要开 feature；本库不开 |
| `windows-sys` | `0.61` | 仅 Windows |

macOS 与 Linux 上新的直接传递依赖是 `attr_alias`。`libc` 本仓库原先已有。`signal-hook` 与 `parking_lot` 不进这两个目标。

Buck 的 rustc 工作目录是仓库根，而 `attr_alias` 按相对路径打开 `src/attr-aliases.txt`（`attr_alias` 0.1.5 `aliases.rs` 的 `parse`）。Cargo 编译这个 crate 时工作目录是 crate 自己的目录，读的是包内那份。Buck 读不到包内路径，所以仓库 `src/attr-aliases.txt` 是 5.2.0 源码包里同名文件的原样副本。升级 `process_control` 时两份必须一起换。

`attr_alias` 在 2026-10-05 的 crates.io 最新稳定版是 [0.1.5](https://crates.io/crates/attr_alias)（2026-02-07，MIT OR Apache-2.0）。`src/Cargo.lock` 把 `process_control` 5.2.0 锁到 `attr_alias` 0.1.5，并记下 `libc`、`signal-hook`、`windows-sys` 0.61.2。Buck 按平台接线：macOS 与 Linux 只依赖 `libc` 加 `attr_alias`；Windows 只依赖 `windows-sys`。`signal-hook` 没有进这四个目标。

最低 Rust 版本：process_control 声明 1.83.0，低于本库 1.98.0。

## 为什么 2025-09-06 之后没有新版仍可接受

crates.io 在 2026-10-05 仍把 5.2.0 标为最新稳定版，没有更高版本，也没有 yank。我们调用的 `time_limit`、`terminate_for_timeout`、两个 filter 都在这一版里，而且 macOS / Linux 的终止路径是 `waitid`，不是按进程号 `kill`。标准库没有时限。[wait-timeout 0.2.1](https://crates.io/crates/wait-timeout) 被 process_control 自己的文档写成不能自动终止，也没有边等边收输出的接口。一年没有新版是维护节奏慢，不是缺我们要的能力。不使用 `memory_limit`，所以 macOS 上没有这个方法不影响这次采用。

## 候选比较

| 候选 | 版本 | 许可证 | 时限 | 超时后终止且避开进程号复用 | 收输出时能截断 |
| --- | --- | --- | --- | --- | --- |
| process_control | 5.2.0 / 2025-09-06 | MIT OR Apache-2.0 | 有 | macOS 与 Linux 用 `waitid` | filter 可丢弃后续块 |
| 标准库 `Child` | Rust 1.98.0 | 随编译器 | 无 | `kill` 按进程号 | 要自己读管道 |
| wait-timeout | 0.2.1 | MIT OR Apache-2.0 | 有 | 不终止 | 无配套收输出 |
| 继续手写轮询 | — | — | 有 | 按进程号 `kill` | 已有 16 MiB，但是另一份实现 |

## 决定建议

采用 SDK：钉 `process_control` `=5.2.0`，不开 `parking_lot`。一个辅助函数放在 `foundation::command::run_bounded`：先写完标准输入，再 `time_limit` + `terminate_for_timeout`，用 filter 计标准输出和标准错误，任一路超过 16 MiB 就拒绝。不调用 `memory_limit`。

各调用的时限与超时在领域上的含义由调用方保持：`git` / `gh` / `tea` 仍是结果未知，材料库仍是材料不可读，其余调用给出自己的错误码，不把超时当成成功。

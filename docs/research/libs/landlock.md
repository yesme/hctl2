# Landlock

> 对象：Linux 上挡住 Agency 凭据根的内核机制。

## 决定建议

macOS 用系统自带的 `sandbox-exec` 拒绝凭据根。Linux CI 的 runner 拒绝无特权 `unshare`（写 `/proc/self/uid_map` 得到 EPERM），所以不用用户命名空间。

Landlock 从 Linux 5.13 起可以由普通进程启用。允许名单里不放凭据根，子进程打开该路径会失败。工作区把 `unsafe_code` 设成 forbid，本仓库不直接写系统调用。采用 [`landlock` 0.4.7](https://docs.rs/landlock/0.4.7)（MIT），只使用 ABI V1。`restrict_self` 会设置 `no_new_privs`，这是内核要求的前提。

实现放在 `agency` 可执行文件的 `--confine` 入口，限制当前进程后 `exec` 目标程序。已打开的 stdin 不受这次限制影响。不把这件事声明成目录里的隔离效果。

内核不支持 Landlock，或 `restrict_self` 没有完全生效时，助手以非零状态退出，目标程序不会启动。限制失败不会变成不加限制地运行。

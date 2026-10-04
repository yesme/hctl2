# Landlock

> 对象：Linux 上挡住 Agency 凭据根的内核机制。

## 决定建议

macOS 用系统自带的 `sandbox-exec` 拒绝凭据根。Linux CI 的 runner 拒绝无特权 `unshare`（写 `/proc/self/uid_map` 得到 EPERM），所以不用用户命名空间。

Landlock 从 Linux 5.13 起可以由普通进程启用。允许名单里不放凭据根，子进程打开该路径会失败。工作区把 `unsafe_code` 设成 forbid，本仓库不直接写系统调用。采用 [`landlock` 0.4.7](https://docs.rs/landlock/0.4.7)（MIT），只使用 ABI V1。`restrict_self` 会设置 `no_new_privs`，这是内核要求的前提。

实现放在 `agency` 可执行文件的 `--confine` 入口，限制当前进程后 `exec` 目标程序。已打开的 stdin 不受这次限制影响。不把这件事声明成目录里的隔离效果。

内核不支持 Landlock，或 `restrict_self` 没有完全生效时，助手以非零状态退出，目标程序不会启动。限制失败不会变成不加限制地运行。

## 复核记录

2026-10-05：ABI V1 的 `from_read` 没有 `WriteFile`。锁定的 Herdr 0.8.2 在这个名单下 `workspace.create` 返回 `failed to openpty`（EACCES），开的是 `/dev/ptmx`。`/dev/pts` 是另一挂载的 devpts，父目录上的规则走不到这个挂载根。`/dev` 与 `/dev/pts` 因此加上 `WriteFile` 和 `MakeChar`，不放开删除和新建普通文件。凭据根落在这两处下面时仍拒绝启动。允许路径规范化失败就报错，不退回原始路径。

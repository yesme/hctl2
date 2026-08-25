# HCTL2 离线依赖包

这是按目标平台构建的 HCTL2 可离线安装开发包。它包含锁定版本的 Tuwunel、Vikunja、Dagu 和 tmux、所需的非系统动态库、许可证、对应上游源码，以及用于管理四个组件的 `hctl2-services`。

安装到当前用户：

```bash
./install.sh
~/.local/bin/hctl2-services start
~/.local/bin/hctl2-services smoke
```

安装器会验证完整内容，不联网、不编译，也不要求 Rust、Homebrew 或 Linux 构建工具。默认状态目录为 `${XDG_STATE_HOME:-$HOME/.local/state}/hctl2`；绝对路径 `HCTL2_STATE_ROOT` 可隔离并行环境。所有网络 listener 都只绑定 loopback。

未来 HCTL2 二进制和 Workbench 资源会进入同一个版本化 payload；当前命令详见[HCTL2 使用说明](./USAGE.md)。

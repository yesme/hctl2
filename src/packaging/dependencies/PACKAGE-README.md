# HCTL2 Linux 离线包

这是面向 Linux x86_64 的 HCTL2 可离线安装开发包。它包含锁定版本的 Tuwunel、Vikunja、Dagu 和 tmux 可执行文件、tmux 的非系统共享库、许可证材料、锁定的上游源码（包括 GPL/AGPL 组件的对应源码），以及用于运行四个依赖且纳入版本控制的生命周期脚本。

为当前用户安装：

```bash
./install.sh
~/.local/bin/hctl2-services start
~/.local/bin/hctl2-services smoke
```

默认持久状态目录为 `${XDG_STATE_HOME:-$HOME/.local/state}/hctl2`。设置绝对路径 `HCTL2_STATE_ROOT` 可以隔离一份安装的状态。所有服务 listener 都绑定 loopback。

这是第一段打包实现。未来 HCTL2 二进制和 Workbench 资源会进入同一个版本化 payload；公共 CLI 完成后由 `hctl2 start` 负责这套生命周期。当前所有命令的详细说明见[HCTL2 使用说明](./USAGE.md)。

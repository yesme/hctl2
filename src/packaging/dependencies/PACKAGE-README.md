# HCTL2 离线依赖包

这是按目标平台构建的 HCTL2 可离线安装开发包。它包含锁定版本的 Chatroom（Tuwunel 服务端与 Cinny 浏览器客户端）、Kanban（Vikunja）、Workflow（Dagu）和 Terminal（tmux）、为 Cinny 提供 loopback HTTP 的官方 Static Web Server 单二进制、所需的非系统动态库、许可证，以及统一管理它们的 `hctl2-services`。

安装到当前用户：

```bash
./install.sh
~/.local/bin/hctl2-services start
~/.local/bin/hctl2-services smoke
```

安装器会验证完整内容，不联网、不编译，也不要求 Rust、Python、Node.js、Homebrew 或 Linux 构建工具。默认状态目录为 `${XDG_STATE_HOME:-$HOME/.local/state}/hctl2`；绝对路径 `HCTL2_STATE_ROOT` 可隔离并行环境。所有网络 listener 都只绑定 loopback。

对应的锁定上游源码不占用本安装包；它们位于同一 Release 中的 `hctl2-<version>-<target>-sources.tar.gz`。普通安装不需要下载源码包，精确文件名见本目录的 `SOURCES.md`。

默认启动后可用的浏览器入口为：Chatroom `http://127.0.0.1:6168/`、Kanban `http://127.0.0.1:3456/`、Workflow `http://127.0.0.1:18080/`。Cinny 是随包的 Matrix 互操作与查看客户端，不是 HCTL2 Workbench。

未来 HCTL2 二进制和 Workbench 资源会进入同一个版本化 payload；当前命令详见[HCTL2 使用说明](./USAGE.md)。

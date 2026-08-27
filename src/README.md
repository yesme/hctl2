# HCTL2 产品代码工作区

这个目录存放全部 HCTL2 产品代码和产品测试。仓库根目录继续存放产品与设计文档。

最初的 P1 工作区只包含独立的机械组件：

- `hctl2-tool`：Git/SCM 与仓库机械操作；
- `hctl2-agentd`：Harness 发现、运行时所有权、PTY 与主机观测；
- `hctl2-protocol`：HCTL2 进程共享的传输 envelope。

`hctl2-control`、公共 `hctl2` CLI 与 Workbench 将在 P2/P3 进入代码树。两个 P1 可执行文件目前都不是治理命令入口。

`packaging/dependencies` 负责外部依赖供应链。它固定 Chatroom（Tuwunel 服务端与 Cinny 浏览器客户端）、Kanban（Vikunja）、Workflow（Dagu）和 Terminal（tmux）的版本，为三种目标平台分别构建运行安装包与源码伴随包，并交付离线安装器与纳入版本控制的生命周期脚本。下载的输入与生成的发行归档不提交到 Git。

Cinny 的静态内容由离线包内锁定的官方 `static-web-server` 单二进制提供，并由 `hctl2-services` 直接启停；HCTL2 不实现 HTTP 服务器，也不要求最终用户安装 Python 或 Node.js。`testing/cinny` 记录这个 Chatroom 浏览器客户端的人工验收边界；Cinny 不是 HCTL2 Workbench，也不是第五个执行面依赖。

面向人的 README、使用说明和其他产品文档使用中文；源码、配置和脚本使用英文；命令行 `--help` 内容使用英文。

在此目录运行工作区检查：

```bash
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo build --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
```

Buck2 构建环境、平台和工具链的验证入口见[构建环境说明](build/README.md)。Cargo 在 Buck2 迁移期间继续作为行为一致性检查；第一方日常构建入口将在四批迁移完成后统一切换。

当前各命令的构建与操作方法见[HCTL2 使用说明](../docs/usage.md)。

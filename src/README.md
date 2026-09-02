# HCTL2 产品代码工作区

这个目录存放全部 HCTL2 产品代码和产品测试。仓库根目录继续存放产品与设计文档。

P1 工作区只包含 HCTL2 自己需要实现的机械组件：

- `hctl2-tool`：Git/SCM 与仓库机械操作。

当前没有真实的 HCTL2 进程间 API，因此不预建公共 protocol crate。`hctl2-control` 开工并形成多客户端边界时，再根据实际兼容与多语言需求调研 schema-first RPC；协议生成代码不能反向成为 Git 领域正文的事实源。

Agent / Terminal 的进程、PTY 和终端会话直接交给外部运行服务 Herdr；本代码树不再实现或打包 `hctl2-agentd`。`hctl2-control`、公共 `hctl2` CLI 与 Workbench 将在 P2/P3 进入代码树。`hctl2-tool` 目前不是治理命令入口。

`packaging/dependencies` 负责外部依赖供应链。它固定 Chatroom（Tuwunel 服务端与 Cinny 浏览器客户端）、Kanban（Vikunja）、Workflow（Dagu）和 Terminal（Herdr）的版本，为三种目标平台分别构建运行安装包与源码伴随包，并交付离线安装器与纳入版本控制的生命周期脚本。除上游没有 Darwin 二进制的 Tuwunel 外，第三方运行内容都直接消费摘要锁定的官方发行物；下载输入与生成归档不提交 Git。

`packaging/release` 由 Buck2 导出第一方二进制和 manifest，校验并消费外部运行包与源码包，确定性地生成三平台完整用户安装包、checksums、SPDX SBOM 与 release manifest。它只在子系统边界组装，不改写外部项目的原生构建方式。

`agency` 是发布包自带的本地 Agency 参考实现：参与者供给方的第一方实现，目前只有 harness 原生格式的技能目录（`root//agency:skills`），可用性申报与 control 适配器待建；进程、PTY、终端会话与 TUI 由 Herdr 提供。设计见 [Participant 与 Terminal](../docs/design/participant.md#agency-与执行体)。

Cinny 的静态内容由离线包内锁定的官方 `static-web-server` 单二进制提供，并由 `hctl2-services` 直接启停；HCTL2 不实现 HTTP 服务器，也不要求最终用户安装 Python 或 Node.js。`testing/cinny` 记录这个 Chatroom 浏览器客户端的人工验收边界；Cinny 不是 HCTL2 Workbench，也不是第五个执行面依赖。

面向人的 README、使用说明和其他产品文档使用中文；源码、配置和脚本使用英文；命令行 `--help` 内容使用英文。

在 `src/` 产品工作区运行第一方 Buck2 检查：

```bash
./buck2 test --build-default-info \
  root//apps/... root//crates/... root//build/tests/... \
  root//:clippy root//packaging/release:first-party
```

迁移期间仍在 `src/` 目录运行 Cargo 一致性检查：

```bash
cargo fmt --all --check
cargo metadata --locked --no-deps --format-version 1
```

Buck2 构建环境、平台、BTD 影响范围选择和工具链的验证入口见[构建环境说明](build/README.md)，完整用户包的构建与验收见[发行组装说明](packaging/release/README.md)。Cargo 只保留格式与 locked manifest 检查，不再平行执行 Buck 已覆盖的编译、测试和 Clippy。

当前各命令的构建与操作方法见[HCTL2 使用说明](../docs/usage.md)。

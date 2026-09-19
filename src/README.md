# HCTL2 产品代码工作区

这个目录存放全部 HCTL2 产品代码和产品测试。仓库根目录继续存放产品与设计文档。Buck2 的项目根是仓库根：`src/` 是名为 `root` 的 cell，仓库根是 `repo` cell，文档与许可证以 `repo//...` 标签进入构建图；`./buck2` 启动器仍在本目录，从任何子目录运行都能找到项目根。

工作区包含 HCTL2 自己需要实现的机械组件与可复用基础机制：

- [`apps/tool`](apps/tool/)：Git/SCM 与仓库机械操作，对外命令为 `hctl2-tool`。P1 已具备仓库检查、现场锁、隔离 worktree 物化与核验、封存保全拆除、本地集成（快进或合并提交的比较并交换）以及 `wait` 回读闭集外部事实。独立运行只做普通本地操作，不产生治理记录或 Receipt。
- [`crates/facts`](crates/facts/)：供 `tool` 与 control 共用的事实读取代码。
- [`crates/foundation`](crates/foundation/)：标准库文件锁、JCS、SQLite Online Backup、keyring 与 FTS5 的受测封装。
- [`crates/store`](crates/store/README.md)：P2.1 的控制面存储与命令内核，含 schema 迁移、同事务记录、私有 Git 材料库和一致备份恢复；不包含业务命令或守护进程。
- [`crates/proto`](crates/proto/README.md)：Query / Preview / Submit / Subscribe 的 Protobuf 合同，由钉定 protoc 与 Buck 生成。
- [`apps/control`](apps/control/)：归属者 Unix socket 上的 control 守护进程，对外命令为 `hctl2-control`。
- [`apps/cli`](apps/cli/)：公共 CLI，经该 socket 说话，不直接写控制面存储；对外命令为 `hctl2`。

内部目录、私有 Cargo package、Rust 库与 Buck 库目标按职责取短名（如 `store`、`root//crates/store:store`），不重复产品前缀；package 均继承工作区的 `publish = false`。对外命令与发行物继续用产品名：`tool` 生成 `hctl2-tool`（`root//apps/tool:hctl2-tool`），`control` 生成 `hctl2-control`（`root//apps/control:hctl2-control`），`cli` 生成 `hctl2`（`root//apps/cli:hctl2`）。安装到 Harness 的 Skill 名、环境变量、持久化标识和系统临时目录仍处于仓库外的共享命名空间，保留 `hctl2` / `HCTL2` 前缀。

Git 领域正文仍是 JCS；Protobuf 只做进程间传输。

Agent / Terminal 的进程、PTY 和终端会话直接交给外部运行服务 Herdr；本代码树不再实现或打包 `hctl2-agentd`。Workbench 仍在 P3 进入代码树。`hctl2-tool` 目前不是治理命令入口。

`packaging/dependencies` 负责外部依赖供应链。它固定 Chatroom（Tuwunel 服务端与 Cinny 浏览器客户端）、Kanban（Vikunja）、Workflow（Dagu）和 Terminal（Herdr）的版本，为三种目标平台分别构建运行安装包与源码伴随包，并交付离线安装器。服务生命周期由随包 Process Compose 的声明式配置管理；GitHub 事实由随包 `gh` 读取。除上游没有 Darwin 二进制的 Tuwunel 外，第三方运行内容都直接消费摘要锁定的官方发行物；下载输入与生成归档不提交 Git。

`packaging/release` 由 Buck2 导出第一方二进制和 manifest，校验并消费外部运行包与源码包，确定性地生成三平台完整用户安装包、checksums、SPDX SBOM 与 release manifest。它只在子系统边界组装，不改写外部项目的原生构建方式。

`agency` 是发布包自带的本地 Agency 参考实现：参与者供给方的第一方实现，目前只有 harness 原生格式的技能目录（`root//agency:skills`），可用性申报与 control 适配器待建；进程、PTY、终端会话与 TUI 由 Herdr 提供。设计见 [Participant 与 Terminal](../docs/design/participant.md#agency-与执行体)。

Cinny 的静态内容由离线包内锁定的官方 `static-web-server` 单二进制提供，并由 Process Compose 启停；HCTL2 不实现 HTTP 服务器，也不要求最终用户安装 Python 或 Node.js。`testing/cinny` 记录这个 Chatroom 浏览器客户端的人工验收边界；Cinny 不是 HCTL2 Workbench，也不是第五个执行面依赖。

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

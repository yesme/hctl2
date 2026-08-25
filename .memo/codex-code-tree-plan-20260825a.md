# HCTL2 代码树与分阶段落地计划

> 日期：2026-08-25<br>
> 基线：`origin/main` @ `b1d22a8`（draft v0.12.3）<br>
> 状态：Informative · 开工布局记录，不修改设计合同<br>
> 所有者决定：产品代码统一放在仓库顶层 `src/`，不与 README、设计文档和 memo 混放。

## 一、施工主线

现有分步骤计划没有消失，权威版本是 [`docs/design/delivery.md`](../docs/design/delivery.md) 的 P0～P3：

1. **P0 · 探路**：先让 Tuwunel、Vikunja、tmux、Dagu 在本地独立跑起来，验证 HCTL 要使用的接缝。探针可丢弃，不进入产品代码树；留下的结论写入实现证据。
2. **P1 · 备装**：建立 Rust workspace，实现 `hctl2-tool`、`hctl2-agentd` 和必要的 typed protocol；先取得 Git/readback、Harness、tmux、PTY 等物理原语，此时没有治理账本。
3. **P2 · 接钥匙**：实现 `hctl2-control`、SQLite 账本和公共 `hctl2` CLI；Matrix、任务后端、runtime、workflow adapters 随相应纵向切片首次消费时接入，让各组件通过命令服务协同。B0～B5 都在本阶段逐级完成，Dagu 到 B4 才进入关键路径。
4. **P3 · 装门面**：实现 Electron + React 的 `hctl2-workbench`；Workbench 只消费与 CLI 相同的公共命令服务，不承担 B0～B5 的事实切换。

人话顺序就是：**外部组件先本地跑通 → tool/agentd 与执行接缝 → control 与账本 → CLI 串成纵向闭环 → Workbench。**

## 二、仓库边界

仓库根目录保存产品说明、设计、评审记录、许可与 Git/托管平台必需元数据；完整产品 workspace 只有一个入口：`src/`。

```text
hctl2/
├── README.md
├── docs/
├── .memo/
├── LICENSE
├── .gitignore
├── .github/                    # 托管平台元数据；构建 working-directory=src
└── src/                        # 全部产品代码、构建配置与产品测试
    ├── Cargo.toml              # virtual Cargo workspace
    ├── Cargo.lock
    ├── rust-toolchain.toml
    ├── rustfmt.toml
    ├── apps/
    ├── crates/
    └── tests/
```

仓库根目录不放第二份 `Cargo.toml`、`package.json`、产品 migration、fixture 或源代码。Rust 命令统一从 `src/` 运行：

```bash
cd src
cargo build --workspace
cargo test --workspace
```

P0 的临时数据、下载物、进程目录和一次性脚本放在 repo 外或明确 ignored 的临时目录；它们不成为产品目录。P0 的可保留产出只有固定版本、接缝结论、许可信息和实现证据。

## 三、目标代码树

代码树以部署组件为第一层，以领域模块为 `hctl2-control` 内部边界。Project、Task、Run、Agent 共享同一本 SQLite 账本和跨模块事务，第一阶段不拆成四个 crate。

```text
src/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── rustfmt.toml
│
├── apps/
│   ├── hctl2-cli/              # package=hctl2-cli；产出 binary hctl2
│   │   └── src/
│   │       ├── main.rs
│   │       ├── client.rs       # 只连接 control
│   │       ├── commands/
│   │       └── render.rs
│   │
│   ├── hctl2-control/          # 唯一领域 command service / metadata writer
│   │   ├── migrations/         # 一本账、一条有序 migration 链
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       ├── domain/
│   │       │   ├── primitives.rs
│   │       │   ├── project.rs
│   │       │   ├── task.rs
│   │       │   ├── run.rs
│   │       │   └── agent.rs
│   │       ├── application/
│   │       │   ├── command.rs
│   │       │   ├── query.rs
│   │       │   ├── event.rs
│   │       │   ├── projection.rs
│   │       │   ├── connections.rs
│   │       │   └── recovery.rs
│   │       ├── ports/          # control 作为消费者定义能力接口
│   │       ├── adapters/       # SQLite、Matrix、Vikunja、Dagu、tool、agentd
│   │       └── transport/      # 本地公共命令服务；不承载领域判断
│   │
│   ├── hctl2-tool/             # Git/SCM 机械操作；standalone 可用
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       ├── repo.rs
│   │       ├── changeset.rs
│   │       ├── integration.rs
│   │       ├── readback.rs
│   │       ├── digest.rs
│   │       ├── memo.rs
│   │       └── lint.rs
│   │
│   ├── hctl2-agentd/           # Harness、tmux、PTY 与物理观测
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       ├── discovery.rs
│   │       ├── harness.rs
│   │       ├── runtime/
│   │       │   └── tmux.rs
│   │       ├── terminal.rs
│   │       └── observation.rs
│   │
│   └── workbench/              # P3 才创建；Electron + React
│
├── crates/
│   └── hctl2-protocol/         # 纯 wire DTO，不放领域规则
│       └── src/
│           ├── public.rs       # Query / Preview / Submit / Subscribe
│           ├── tool.rs
│           ├── agentd.rs
│           ├── event.rs
│           └── error.rs
│
└── tests/
    ├── contract/               # test-only workspace package
    │   └── tests/
    │       ├── ct_project.rs
    │       ├── ct_task.rs
    │       ├── ct_run.rs
    │       ├── ct_agent.rs
    │       ├── ct_connection.rs
    │       └── ct_system.rs
    └── fixtures/
        ├── git/
        ├── jcs/
        ├── sqlite/
        └── providers/
```

这是目标形状，不是要求首个提交创建全部空目录。目录和文件随切片首次需要时落地；例如 P1 不创建 Matrix/Dagu adapter，P2 的 B0 也不创建 Workbench。

## 四、依赖与写入纪律

```text
hctl2 CLI ───────▶ hctl2-protocol ◀────── hctl2-control
                                               │
                                    domain ◀ application
                                               │
                                        ports ◀ adapters
                                               │
                              SQLite / tool / agentd / providers

hctl2-tool   ────▶ hctl2-protocol
hctl2-agentd ────▶ hctl2-protocol
```

- CLI 不链接 control 内核、SQLite、Git 或 agentd，只走公共命令服务。
- `domain/` 不依赖 SQLite、HTTP、tmux 或 provider SDK。
- 四模块各自决定自己的状态；跨模块动作只由 `application/connections.rs` 在同一 control 事务中编排。
- SQLite 只由 control 打开；tool/agentd 不读写治理账本。
- 端口 trait 放在消费者一侧，provider 类型在 adapter 边界转换，不能渗入领域模型。
- `hctl2-protocol` 只保存传输 DTO、错误与版本，不成为 `common`/`utils` 垃圾桶。
- 第一阶段不建动态 plugin system，不按 Matrix/Vikunja/Dagu/tmux 各造一套领域状态机。
- 测试辅助代码先就地保存；至少两个 package 真实复用后才抽共享 testkit。

## 五、过度设计禁区

本代码树落实 2026-08-25 设计评审和所有者裁决：

- 不建设隐藏 Git common-dir/refs 的 broker；`hctl2-tool` 正常使用 linked worktree、common-dir 和 refs。
- 不建设 nonce/challenge 型 human-presence 子系统；经认证的 Workbench/CLI 操作入口赋予 human provenance，execution/result 通道不能自报 human。
- managed HCTL Room 在 Matrix adapter 准入时强制非 E2EE，不实现第一阶段 Matrix crypto 设备体系。
- Context 实际 bytes 先作为 control 内部 retention/storage 选择，不新建领域级 blob service。
- Matrix Room 创建和其他外部写共用 outbox/readback，不另造 provisioning 聚合。
- generation、幂等键和 reducer 结构保持内部最小实现，只冻结“旧执行不能覆盖新执行”等产品不变量。

## 六、首批提交边界

第一批产品代码只建立能持续生长的地基：

1. `src/Cargo.toml`、toolchain pin、Cargo lock、workspace lint/format/test 配置；
2. `hctl2-protocol` 为 tool/agentd 提供的最小版本与错误 envelope；
3. `hctl2-tool`、`hctl2-agentd` 两个 package 的薄入口；
4. 两个 binary 的 `--version`、结构化启动错误和 smoke test；
5. CI 在 `src/` 执行 format、Clippy、build 和 test。

首批提交不创建 `hctl2-control`/CLI 空壳、数据库 schema，不接外部服务器，也不创建 Workbench 空壳。随后按 P1 先完成 tool/agentd 的真实纵向能力；进入 P2/B0 时才创建 control 与 CLI。

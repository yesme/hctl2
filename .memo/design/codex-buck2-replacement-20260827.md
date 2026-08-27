# Buck2 替换方案：第一方构建图与外部子系统边界

> 状态：已拍板 · 待落地（Accepted 为四批实现 PR 的施工依据；转向与 Buck2 commit 钉定尚未写入）<br>
> 基线：main @ c99114d（草案 v0.13.1）<br>
> 去向：decision-history 转向 + docs/research 钉 Buck2 commit + src/ 四批实现 PR<br>
> 日期：2026-08-27  
> 说明：Accepted · 作为后续四批实现 PR 的施工依据  
> 关联：[Grok 构建确定性备忘](./grok-20260827a.md)  
> 范围：HCTL2 第一方代码、工具链、CI/CD 与最终发行组装；不重写外部独立产品的原生构建图。

## 决定

HCTL2 采用 Buck2 管理自己的跨平台、多语言编译环境。选择理由不是替代 Cargo、pnpm、Vite 或上游项目的原生构建系统，而是把第一方源码、工具链、平台、生成动作、测试与产物纳入同一张可查询、可缓存、可验证的依赖图。

外部独立产品不做细粒度 Buck2 化。Tuwunel、Vikunja、Dagu、tmux、Cinny 与 Static Web Server 继续按各自上游形态获取或编译：有官方二进制时消费固定版本与摘要的官方制品；需要源码编译时继续使用上游 Cargo、`configure && make` 等原生入口。HCTL2 只在子系统边界接收并验证它们的产物合同，不维护其内部 crate、Go package 或 C target 图。

这条边界也修正了“所有我们交付的组件必须使用同一编译器版本”的过宽表述：HCTL2 自己编译的同一种语言默认共用一把钉死的工具链；外部子系统使用其上游工具链，版本与构建环境进入该子系统自己的记录。官方二进制视为摘要锁定的外部 blob，除非上游提供可验证 provenance，不推断它使用了哪一把编译器。

## 当前基线

第一方代码当前是 Rust 1.98.0 Cargo workspace，包含 `hctl2-agentd`、`hctl2-tool` 与 `hctl2-protocol`。Ubuntu 上复跑现有 CI 路径，格式、Clippy、workspace build 与 11 个测试全部通过。

`src/packaging/dependencies` 约有 2,500 行 shell。它已经固定远端 URL、版本、源码 commit 与 SHA-256，生成运行包、源码伴随包、许可证和 payload manifest，也具备安装、启停和 smoke 测试；但下载、编译、组包与宿主机工具链仍由 shell 顺序和人工 cache marker 串联。GitHub Actions 当前只检查 Rust workspace 和 shell 语法，没有构建外部子系统包，没有 Release workflow、签名、notarization、artifact provenance 或受保护的 `main` 必需检查。

## 四类构建对象

| 类型 | 例子 | 归属 |
| --- | --- | --- |
| HCTL2 第一方代码 | agentd、tool、protocol、未来 control、CLI | Buck2 原生目标 |
| 编进第一方产物的库 | Rust crates、未来 npm packages | Cargo/Reindeer 或 pnpm lock 是事实源；进入第一方 Buck 图 |
| 外部独立子系统 | Tuwunel、Vikunja、Dagu、tmux、Cinny、Static Web Server | 保持上游原生构建；以粗粒度产物合同接入 |
| 运行期资产 | config、安装与 `start/stop/status/smoke` 脚本、静态内容 | 作为声明输入被组包和测试，不误写成编译规则 |

这里的“第三方库”和“外部子系统”不能混为一谈。链接进 HCTL2 二进制的 crate 或 npm package 属于第一方编译闭包，必须受 HCTL2 工具链和锁文件约束；拥有独立进程、发布物与原生构建系统的上游产品是外部子系统，Buck2 不进入其内部图。

## 目标拓扑

```text
HCTL2 source
    │
    ├── pinned Rust / Node / pnpm / Tauri toolchains
    ├── code generation, lint and tests
    └── Buck2 first-party artifacts
                         │
                         ▼
                  release assembly
                         ▲
                         │
external subsystem builds
    ├── chatroom package
    ├── kanban package
    ├── workflow package
    └── terminal package
```

外部子系统向发行层只暴露固定合同：

- 目标平台身份；
- 运行归档与 SHA-256 sidecar；
- 许可证要求所需的源码伴随归档；
- dependencies、sources 与 licenses manifest；
- 可离线执行的安装和生命周期测试结果。

最终发行组装可以把这些归档作为不可透明修改的输入，但不得借此把上游内部构建逻辑复制进 HCTL2 Buck rules。

## 平台命名

不把 `macos` 全局改写成 `darwin`。业界成熟做法是按语义分层：产品和 Buck2 OS constraint 使用 `macos`，Rust target triple、Go/Node 平台与 `uname` 边界按各生态使用 `darwin`。

第一批 Buck platform 使用：

- `linux_x86_64_gnu`；
- `macos_x86_64`；
- `macos_arm64`。

平台定义分别记录 Buck OS/CPU constraint、Rust target triple、宿主检测值和上游 asset aliases。例如 `macos_arm64` 映射到 Rust 的 `aarch64-apple-darwin`、Go 的 `darwin/arm64` 与 `uname` 的 `Darwin/arm64`。目录 `platforms/macos` 保留产品平台语义，不使用混合生态字符串承担多种身份。

## 工具链纪律

- Buck2 本身使用仓库内 DotSlash launcher，钉定日期发行版及匹配的 Prelude commit，不使用移动的 `latest`。
- HCTL2 第一方 Rust target 共用 Rust 1.98.0；升级是一项显式、全平台回归的变更。
- Cargo manifests 和 lockfile 继续是 Rust 依赖事实源；Reindeer 生成第三方 crate 的 Buck targets，不手写两份依赖图。
- Workbench 落地后，pnpm lock 与 Cargo lock 继续各自拥有语言依赖事实；Buck2 钉 Node、pnpm、Rust、Tauri CLI，编排 Vite 与 Tauri2 的原生构建入口。第一版不使用 Buck2 JS Prelude 逐模块替代 pnpm/Vite。
- macOS 的 SDK、deployment target、架构、linker 与签名身份是 target-specific 输入。Linux 与 Apple Clang 不因版本字符串不同而被伪装成同一发行物。
- Buck2 本地 action 目前并不自动获得严格沙箱；“未声明输入不可见”的硬保证需要 Remote Execution 或受控构建环境。未接入之前，文档只承诺工具链钉定、动作输入身份和缓存失效，不夸大本地 hermeticity。

## 仓库形状

Buck2 启动所需的 `.buckconfig`、`.buckroot` 与 DotSlash launcher 位于仓库根；其余产品构建代码进入 `src/`：

```text
src/
├── build/
│   ├── platforms/
│   ├── toolchains/
│   └── rules/
├── apps/*/BUCK
├── crates/*/BUCK
├── third-party/rust/
└── packaging/dependencies/    # 外部子系统，保留原生构建
```

目录随首个真实使用场景创建，不提前铺 Workbench、Node 或未来语言的空壳。

## 四批实现

### PR 1 · Buck2 基础设施

- 钉 Buck2、DotSlash 与匹配 Prelude；
- 建立三个 platform configuration；
- 建立 Rust 1.98.0 toolchain；
- 提供 `./buck2 build //src/...` 与 `./buck2 test //src/...` 的稳定入口；
- 验证工具链或平台定义变化会让相关 action 失效。

### PR 2 · 第一方 Rust workspace

- 为 protocol、agentd 与 tool 建立 Buck targets；
- 钉 Reindeer，并从 Cargo 事实源生成第三方 crate 规则；
- 覆盖当前 build、unit test 与 smoke test；
- 保留 Cargo 路径作为 parity oracle，直到 Buck 产物与行为验证一致。

### PR 3 · CI 切换

- PR 主检查改为 Buck2 build/test；
- Linux、macOS Intel 与 macOS Apple Silicon 使用明确的平台和执行环境；
- GitHub Actions 引用钉完整 commit SHA；
- 消除 feature branch 同时触发 push 与 pull request 的重复运行；
- Buck 检查稳定后成为 `main` 必需检查；Cargo parity 暂时保留但不再代表主构建入口；
- 外部子系统 native build 与整包生命周期测试保持独立 job，不能用绿色第一方 job 冒充完整发行验证。

### PR 4 · 发行组装与旧入口收口

- Buck2 导出第一方二进制集合与 manifest；
- 外部子系统继续导出运行包、源码包、checksum 与 manifest；
- 确定性发行组装器消费两侧产物，生成三个目标平台的用户安装包；
- macOS 签名与 notarization 放在确定性构建后的 CD 阶段；
- Release workflow 上传运行包、源码包、checksums、SBOM 与 provenance；
- 通过三平台 parity 和安装生命周期验证后，删除只为第一方旧编译入口存在的脚本；保留 Cargo manifests、外部子系统 native build 与全部运行期脚本。

## 明确不做

- 不把 Tuwunel 的 Cargo graph 生成成 HCTL2 BUCK targets；
- 不把 tmux、libevent、ncurses 或 utf8proc 改写成细粒度 Buck C/C++ targets；
- 不要求外部官方二进制与 HCTL2 使用同一编译器；
- 不为了 Buck2 重写已经通过生命周期验证的外部子系统脚本；
- 不把运行期启停脚本包装成编译规则；
- 不在 parity 成立之前一次性删除 Cargo 或现有发行路径。

## 工作量表达方式

AI 化开发不再用传统人日作为主要估算单位。后续采用三项粗粒度指标：

1. **改动量级**：本次属于中等基础设施变更，不改业务模型，但触及构建入口、三平台工具链、CI 与发行边界；
2. **风险面**：第一方 Rust 规则低，Buck2 开源工具链与 macOS 双架构中，签名/notarization 和严格 hermeticity 高；
3. **可独立验收批次**：四个主 PR，每批都有可运行命令和退出条件，验证通过后才进入下一批。

估算保持大略，不为尚未遇到的问题虚构精确步骤或日历时间。

## 总体验收

- 一条 Buck2 命令构建和测试全部 HCTL2 第一方代码；
- 三个平台的工具链和 target identity 可查询，未使用自造字符串隐式推断；
- Cargo/pnpm 等语言锁文件仍是依赖事实源，不出现两套手写图；
- 外部子系统继续用原生入口构建，Buck2 不侵入其内部；
- CI 分清第一方、外部子系统与完整发行三种健康状态；
- 从空 runner 产生三个可安装发行包，完成 checksum、安装、启动、状态、smoke 与停止验证；
- `remote/main` 只接受通过相应检查的 PR，工作树与发布产物边界保持干净。

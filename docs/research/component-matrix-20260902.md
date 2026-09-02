# 部件矩阵：目的、现选择、业界最佳实践与借用等级

> 日期：2026-09-02<br>
> 状态：Informative 研究备忘录，不定义 HCTL2 语义；复用判断以[实现证据](./README.md#复用决策用语)的五种复用决策用语为准，借用等级用所有者立的四级尺子：跨平台二进制 > 拿 SDK 自己开发 > 直接复制代码 > 借鉴想法，四级之外才是自研。<br>
> 方法：部件清单取自[实现证据条目索引](./README.md#条目索引)、[交付文档](../design/delivery.md#第一阶段范围)、[三面架构](../design/architecture.md#场景与系统)、[系统边界的组件表](../design/spec/system.md#组件)与 `src/` 三份 README；每个部件问三件事——它为什么存在、业界最佳实践是什么、能不能直接借一个二进制；借来的是超集时再问多出的复杂度付得起吗、付了之后是不是就不用自己写了。已有研究条目直接引用，不重审；缺证据处用 `gh api`、crates.io API 与官方文档补，钉版本或 commit，宣传语不作数。<br>
> 证据钉：上游最新版本均为 2026-09-02 `gh api repos/<owner>/<repo>/releases/latest` 实测；crate 版本、许可与下载量取自 crates.io API 同日快照；引用 HCTL2 文档一律写「文件 §节名」。完整基线见文末[审计基线](#三审计基线来源与版本)。

## 结论先行

1. **四个 content 系统、随包客户端与桌面壳全部维持。** Tuwunel、Vikunja、Dagu、Herdr、Cinny、Static Web Server、Tauri 2 逐一对照至少两个候选后没有更好的二进制；Dagu（2.15.1 → 2.16.1）与 Vikunja（2.5.0 → 2.6.0）落后一个小版本，是常规升级，不是选型问题。
2. **建议换的两处都在构建与发行侧，而且都是「业界已有二进制、我们在自己编或自己写」。** 一是 Reindeer 启动器用 Cargo 从源码编译，而上游每个 release 都带八个官方二进制，应改成和 `buck2-bin` 一样的 DotSlash 清单；二是运行时服务生命周期（`hctl2-services` 加 `runtime.sh` 系列 675 行 shell 的 PID 文件、`kill -0`、`sleep` 轮询）是在重写一个进程监督器，而开发侧已经拍板用 Process Compose 干同样的事——后者是方向问题，需所有者拍板。
3. **约束层里靠第一方代码实现的十三项通用机制（表 D），没有一项能整块换成二进制或 SDK。** 最接近的超集是 durable execution 引擎：Restate 是 Rust 单二进制但服务端 BSL 1.1（非 OSI，按复用决策用语上限只能仅参考行为）；DBOS Transact 是 MIT 但没有 Rust；Temporal 要独立服务加数据库。语义部分全部维持自研，但其中六项应改成「拿 SDK 自己开发」而不是手写：JCS 用 `serde_jcs` 或 `serde_json_canonicalizer`，备份快照用 SQLite Online Backup API，全文索引用随 SQLite 一起进来的 FTS5，现场锁用 `fd-lock`，密钥用 `keyring`，provider 客户端从 OpenAPI/JSON Schema 生成。
4. **hctl2-control 的六个 provider 适配器应从 schema 生成或用类型库，不手写 HTTP 客户端。** Dagu 钉定 commit 带 OpenAPI 3.0.0（`api/v1/api.yaml`），Herdr 能导出 JSON Schema（`herdr api schema --json`），GitHub 有 `octocrab`，Linear 走 `graphql_client`，Matrix 用 `ruma` 类型（`matrix-sdk` 已不再提供 appservice crate）；Vikunja 的规格是 Swagger 2.0，要先转 OpenAPI 3 再生成。这是预防性建议——control 还没开工，现在定比写完再改便宜。
5. **我们在重造轮子的地方只有两处半：** Reindeer 源码编译、运行时进程监督脚本，以及「半处」——如果 control 开工后手写 reqwest 客户端。反过来，几处看起来像重造但查过业界后应维持的：账本加 outbox（SQLite 内嵌的 outbox 库没有一个在同一本账本的同一事务里）、文档检查器（[docs-lint.md](./docs-lint.md) 已审：五项里四项是本库私有语义）、macOS dylib 收集（[macdylibbundler.md](./build-tools/macdylibbundler.md) 已审）、发行组装 shell（没有工具把六个异源上游二进制加第一方产物组成离线包）。
6. **总账。** 矩阵四十行里：直接消费上游二进制十七行、SDK 级十二行、借鉴想法四行、自研七行（三个第一方组件加四块第一方 shell）。把自研件按约束层拆开：hctl2-tool 五项职责全是对 git CLI 的胶水；hctl2-control 十八条里九条是胶水、两条是薄自研、一条可改借用、一条待定，剩下五条——命令准入信封、六个代次、Verdict/Receipt 归约与法定票数、恢复对账、Context 选材策略——加上两条薄自研（outbox、绑定冻结），逐行落在[愿景文档 §产品原生核心与架构最小内核](../design/vision.md#产品原生核心与架构最小内核)那张五行表上。自研的部分是胶水吗？一半是，另一半就是产品本身。

## 一、部件矩阵

借用等级列写等级与取舍；建议列只有四种动作：维持、换成 X、由自研改借用、由借用改自研。「二进制」指直接消费上游跨平台官方发行物。

### A · 四个场景的 content 系统与随包客户端

| 部件 | 目的 | 现选择 | 业界最佳实践（候选钉来源） | 借用等级与取舍 | 建议 |
| --- | --- | --- | --- | --- | --- |
| chat server | 承载 Room 消息 content，让任何 Matrix 客户端直接进房间 | Tuwunel [`v1.9.0 / 5b366914`](https://github.com/matrix-construct/tuwunel/releases/tag/v1.9.0)，Apache-2.0；macOS 制品由 HCTL2 Release 自托管 | [Continuwuity `v26.7.2`](https://github.com/continuwuity/continuwuity/releases/tag/v26.7.2)（同 conduwuit 谱系，Rust 单二进制，Apache-2.0）；[Synapse `v1.159.0`](https://github.com/element-hq/synapse/releases/tag/v1.159.0)（参考实现，Python + PostgreSQL，AGPL-3.0） | 二进制。超集：联邦、E2EE、媒体仓库都不用，付的是约 60 MiB RSS、RocksDB/media 一致性备份和一条 macOS 自托管制品链；换来零聊天服务代码和整个 Matrix 客户端/桥接生态 | 维持；证据见 [matrix-homeserver.md](./matrix-homeserver.md#e-l4-matrix-homeserver) |
| Cinny | Workbench 到位前的浏览器聊天入口 | Cinny [`v4.12.6 / 33f4ba36`](https://github.com/cinnyapp/cinny/releases/tag/v4.12.6)，AGPL-3.0 | [Element Web `v1.12.27`](https://github.com/element-hq/element-web/releases/tag/v1.12.27)（AGPL-3.0，功能全、包更大） | 二进制（官方静态发行包，18.5 MiB） | 维持；见[运维与资源占用](./README.md#已选外部服务的运维与资源占用) |
| Static Web Server | 只给 Cinny 提供静态文件 | SWS [`v2.44.0 / 27aa3450`](https://github.com/static-web-server/static-web-server/releases/tag/v2.44.0)，MIT/Apache-2.0 | [Caddy `v2.11.4`](https://github.com/caddyserver/caddy/releases/tag/v2.11.4)（Apache-2.0，反向代理与自动 TLS，超集）；[miniserve `v0.35.0`](https://github.com/svenstaro/miniserve/releases/tag/v0.35.0)（MIT，Rust 单二进制） | 二进制。SWS 是三者里最窄的一个，正好只做这件事 | 维持 |
| task backend | 承载任务卡 content；API 完整、支持条件写入 | Vikunja [`v2.5.0 / ef2200e9`](https://github.com/go-vikunja/vikunja/releases/tag/v2.5.0)，AGPL-3.0；上游最新 [`v2.6.0`](https://github.com/go-vikunja/vikunja/releases/tag/v2.6.0)（2026-08-31） | [Planka `v2.2.1`](https://github.com/plankanban/planka/releases/tag/v2.2.1)（AGPL-3.0，Node.js + PostgreSQL 两进程）；[git-bug `v0.10.1`](https://github.com/git-bug/git-bug/releases/tag/v0.10.1)（GPL-3.0，零服务器，2025-05 后无发布）；Linear / GitHub 远端后端只适配协议 | 二进制。单进程 + SQLite 满足「运维简单」；AGPL 义务限于独立进程 | 维持；升级到 v2.6.0 走常规复核（changelog + Task 端口 CT）；证据见 [task-backends.md](./task-backends.md#e-l3-vikunja) |
| workflow engine | 令牌、重试、定时器等机械状态，以及无进程的等待检查点 | Dagu [`v2.15.1 / 532c5129`](https://github.com/dagucloud/dagu/releases/tag/v2.15.1)，GPL-3.0-or-later；上游最新 [`v2.16.1`](https://github.com/dagucloud/dagu/releases/tag/v2.16.1)（2026-08-31） | [Conductor `v3.32.1`](https://github.com/conductor-oss/conductor/releases/tag/v3.32.1)（Apache-2.0，JVM）；[Windmill `v1.801.0`](https://github.com/windmill-labs/windmill/releases/tag/v1.801.0)（AGPL + 商业版）；[Temporal `v1.31.2`](https://github.com/temporalio/temporal/releases/tag/v1.31.2)（MIT，需独立服务 + 数据库） | 二进制。超集：自执行 step、调度器、Web UI；用 Profile 只准依赖/条件/等待与 `human.task` 收窄，多出的部分不碰 | 维持；同上常规升级；证据见 [workflow-engines.md](./workflow-engines.md#e-l2-dagu) |
| Agency 运行时 | 按规格起 Harness，持有进程、PTY、终端会话；多观察者、单控制者、原生 TUI | Herdr [`v0.8.2 / 9eb5214`](https://github.com/herdrdev/herdr/releases/tag/v0.8.2)，Apache-2.0 | [tmux `3.7c`](https://github.com/tmux/tmux/releases/tag/3.7c)（ISC，无 harness 状态与 JSON API）；[Zellij `v0.45.1`](https://github.com/zellij-org/zellij/releases/tag/v0.45.1)（MIT）；Termio `termiod`、tty7、cmux、Pilotty 见 [agentd-runtime-candidates](./agentd-runtime-candidates-20260829.md) | 二进制。超集：workspace/tab、插件市场、Agent 状态检测；缺项（输入租约、事件序号、退出回读）按低信任降级而不补写终端服务 | 维持；证据见 [herdr.md](./runtime/herdr.md#e-l1-herdr) 与[验证记录](./runtime/agency-runtime-validation-20260829.md) |

### B · 第一方组件

| 部件 | 目的 | 现选择 | 业界最佳实践（候选钉来源） | 借用等级与取舍 | 建议 |
| --- | --- | --- | --- | --- | --- |
| `hctl2-control` | 唯一领域命令服务：准入、账本、outbox、对账、生命周期托管、provider 适配 | 第一方自研（Rust）；P2 开工，代码树尚无 | 能「整块借」的只有 durable execution 引擎：[Restate `v1.7.8`](https://github.com/restatedev/restate/releases/tag/v1.7.8)（Rust 单二进制，含幂等、journal、durable timer，但服务端 [BSL 1.1](https://github.com/restatedev/restate/blob/main/LICENSE)，非 OSI）；[DBOS Transact](https://github.com/dbos-inc/dbos-transact-py)（MIT，Postgres/SQLite 后端，Python/TS/Go/Java，无 Rust）；[Temporal `v1.31.2`](https://github.com/temporalio/temporal/releases/tag/v1.31.2)（MIT，独立服务 + 数据库） | 自研。三条理由：许可或语言不合；它们解决「步骤重放」，HCTL 的差异化是「谁有权、凭什么算完成」；再加一个服务进程与[选型判据](../design/delivery.md#选型判据)的「运维简单」冲突。内部按 SDK 拼装（表 D） | 维持自研；通用件全部改成 SDK 级（表 D） |
| `hctl2-tool` | 现场执行者五项：worktree/ChangeSet 物化与隔离、已持久化意图执行与回读、现场 OS 锁与 fence、封存保全、判决结晶副本写入 | 第一方自研 Rust，P1 骨架 135 行；进程级动作一律转调 git | git CLI 的 porcelain/plumbing 分层；[gix `0.87.1`](https://crates.io/crates/gix)（MIT/Apache-2.0，纯 Rust，push/merge/rebase 仍在建）；[git2 `0.21.0`](https://crates.io/crates/git2)（libgit2 绑定） | 自研胶水，零重实现（[decision-history §33](../design/references/decision-history.md#33-hctl2-tool-定界为现场执行者v0153)）。`git update-ref <ref> <new> <old>` 天然就是 expected-head 比较并交换 | 维持 |
| `hctl2` CLI | 公共命令入口，与 Workbench 共用命令服务 | 待建（P2） | [clap `4.6.6`](https://crates.io/crates/clap)（事实标准，MIT/Apache-2.0）；lexopt、argh（更小） | SDK | 维持计划：clap，不自写解析 |
| control API / RPC | CLI、Workbench 到 control 的 schema 与传输 | 未选；边界已定为 Query/Preview/Submit/Subscribe（`.memo/notes/control-api-schema-20260902.md`） | Protobuf + [tonic `0.14.6`](https://crates.io/crates/tonic) 或 Connect；[jsonrpsee `0.26.0`](https://crates.io/crates/jsonrpsee)；Tauri 2 IPC | SDK | 维持「首条真实调用链出现再选」，不预建 protocol crate |
| `hctl2-workbench` | 无特权组合客户端：四场景 provider 交互 + HCTL 命令入口 | Tauri [`2.11.5`](https://github.com/tauri-apps/tauri/releases/tag/tauri-v2.11.5) + React 19；[xterm.js `6.0.0`](https://github.com/xtermjs/xterm.js/releases/tag/6.0.0)、[Tiptap `v3.31.0`](https://github.com/ueberdosis/tiptap/releases/tag/v3.31.0)、[React Aria Components `1.20.0`](https://github.com/adobe/react-spectrum/releases/tag/react-aria-components%401.20.0)、React Flow + Dagre | [Electron `v44.1.1`](https://github.com/electron/electron/releases/tag/v44.1.1)（安全网）；GPUI（原生备选）；见 [workbench-shell.md](./workbench-shell.md#e-workbench-shell) 与[重开调研](./workbench-shell-reopen-20260826/README.md) | SDK / 框架级 | 维持 |
| 本地 Agency 参考实现 | 技能目录、可用性申报、与 control 对话的适配器 | `skills/` 已有（`hctl2-shaping` 改编自 mattpocock/skills，MIT）；申报与适配器待建；运行时为 Herdr | [Agent Skills 规范](https://agentskills.io/specification)；Herdr `v0.8.2` 文档树里有 `agent-skill.mdx`、`plugins.mdx`、`marketplace.mdx`（未审） | 复制代码（技能）+ 自研薄壳 | 维持；Herdr 自带的技能与插件机制列为观察项，不改方向 |

### C · 构建与发行工具

| 部件 | 目的 | 现选择 | 业界最佳实践（候选钉来源） | 借用等级与取舍 | 建议 |
| --- | --- | --- | --- | --- | --- |
| Buck2 | 第一方构建图、测试、Clippy、发行 action | [`2026-08-22`](https://github.com/facebook/buck2/releases/tag/2026-08-22)（DotSlash 清单）；上游周更，最新 `2026-09-01` | [Bazel `9.2.0`](https://github.com/bazelbuild/bazel/releases/tag/9.2.0)（+ rules_rust）；[Pants `2.33.1`](https://github.com/pantsbuild/pants/releases/tag/release_2.33.1)；Cargo-only | 二进制 | 维持；决定过程见 `.memo/design/codex-buck2-replacement-20260827.md` |
| bazel-remote | 本机 loopback REAPI action cache | [`v2.6.2`](https://github.com/buchgr/bazel-remote/releases/tag/v2.6.2)（当前最新），Apache-2.0 | [NativeLink `v1.6.6`](https://github.com/TraceMachina/nativelink/releases/tag/v1.6.6)（Rust，含远端执行，超集）；BuildBuddy | 二进制 | 维持；远端执行需求出现前不换 |
| Process Compose | 管 bazel-remote 的后台化、健康、重启、日志 | [`v1.122.0`](https://github.com/F1bonacc1/process-compose/releases/tag/v1.122.0)（当前最新），Apache-2.0 | [overmind `v2.5.1`](https://github.com/DarthSim/overmind/releases/tag/v2.5.1)（依赖 tmux）；systemd user units / launchd（OS 原生，两套模板）；[pueue `v4.0.4`](https://github.com/Nukesor/pueue/releases/tag/v4.0.4) | 二进制 | 维持；并建议同一二进制接管运行时服务生命周期（表 D 末行）；证据见 [process-compose.md](./build-tools/process-compose.md) |
| Syft | 扫描最终 payload 生成 SPDX 2.3 | [`v1.51.1`](https://github.com/anchore/syft/releases/tag/v1.51.1)（当前最新），Apache-2.0 | [Trivy `v0.74.0`](https://github.com/aquasecurity/trivy/releases/tag/v0.74.0)（Apache-2.0，含漏洞扫描超集）；[cargo-cyclonedx `0.5.9`](https://github.com/CycloneDX/cyclonedx-rust-cargo/releases/tag/cargo-cyclonedx-0.5.9)（只 Rust、只 CycloneDX） | 二进制 | 维持；证据见 [syft.md](./build-tools/syft.md) |
| DotSlash | 工具二进制清单与引导 | [`v0.5.9`](https://github.com/facebook/dotslash/releases/tag/v0.5.9)（当前最新），MIT | [mise `v2026.9.0`](https://github.com/jdx/mise/releases/tag/v2026.9.0)（`mise.lock` 带校验和，aqua 后端）；[aqua `v2.62.3`](https://github.com/aquaproj/aqua/releases/tag/v2.62.3)；[Hermit `v0.52.3`](https://github.com/cashapp/hermit/releases/tag/v0.52.3) | 二进制。DotSlash 无守护、无 shim、清单即文件，与 Buck2 同源 | 维持；证据见 [install-dotslash.md](./build-tools/install-dotslash.md) |
| BTD | 受影响 Buck target 计算 | [`2026-08-20 / 345497d`](https://github.com/facebookincubator/buck2-change-detector/releases/tag/2026-08-20)；上游最新 `2026-09-01` | [bazel-diff `v46.1.0`](https://github.com/Tinder/bazel-diff/releases/tag/v46.1.0)、target-determinator（Bazel 专用） | 二进制 | 维持；证据见 [buck2-change-detector.md](./build-tools/buck2-change-detector.md) |
| jq | 解析 BTD JSON Lines | [`jq-1.8.2`](https://github.com/jqlang/jq/releases/tag/jq-1.8.2)（当前最新），MIT | [gojq `v0.12.19`](https://github.com/itchyny/gojq/releases/tag/v0.12.19)；[jaq `v3.1.1`](https://github.com/01mf02/jaq/releases/tag/v3.1.1) | 二进制 | 维持；证据见 [jq.md](./build-tools/jq.md) |
| Reindeer | `Cargo.lock` → `third-party/rust/BUCK` | [`v2026.08.24.00 / 84035a20`](https://github.com/facebookincubator/reindeer/releases/tag/v2026.08.24.00)，MIT；[启动器](../../src/build/tools/reindeer)下载源码归档、校验 SHA-256，再用 Cargo 1.98 编进用户缓存 | 上游每个 release 带八个官方二进制（`aarch64/x86_64-apple-darwin`、linux musl/gnu，`.zst`，与 Buck2 发行格式相同）；Bazel 侧对应物 crate_universe（不适用） | 现状是「拿源码自己编」，介于 SDK 与二进制之间，还要求开发机有 Cargo；官方二进制可直接进 DotSlash 清单 | **换成 DotSlash 官方二进制**（钉 `v2026.08.24.00` 的 asset，或顺手升 `v2026.08.31.00`），删 66 行启动器；理由：同 `buck2-bin` 机制、零编译、去掉对宿主 Cargo 的隐含依赖 |
| actionlint / ShellCheck | workflow 与 shell 静态检查 | [`1.7.12`](https://github.com/rhysd/actionlint/releases/tag/v1.7.12) / [`0.11.0`](https://github.com/koalaman/shellcheck/releases/tag/v0.11.0)（均为当前最新） | [zizmor `v1.30.0`](https://github.com/woodruffw/zizmor/releases/tag/v1.30.0)（Actions 安全审计，是补充不是替代）；shfmt | 二进制 | 维持；zizmor 作追加观察项 |
| 文档检查器 | 链接与锚点、版本戳、死名、禁令密度、memo 基线，另有四轴清点 | 自研 bash + perl 617 行（`src/build/docs/`） | [lychee `v0.24.2`](https://github.com/lycheeverse/lychee/releases/tag/lychee-v0.24.2)（`--offline --include-fragments`）；[markdownlint-cli2 `v0.23.2`](https://github.com/DavidAnson/markdownlint-cli2/releases/tag/v0.23.2)；[vale `v3.19.0`](https://github.com/errata-ai/vale/releases/tag/v3.19.0) | 自研；[docs-lint.md](./docs-lint.md) 已审：五项里四项是本库私有语义（版本戳、退休词表、禁令词表、memo 目录约定），只有链接项有现成工具 | 维持；链接项的回退路径是 lychee 经 DotSlash 接入 |
| 发行组装 | 三平台完整包、checksums、SBOM、release manifest、离线安装与生命周期测试 | 自研 shell 在 Buck action 内（`packaging/release` 约 550 行 + `packaging/dependencies/common` 约 800 行） | [cargo-dist `v0.32.0`](https://github.com/axodotdev/cargo-dist/releases/tag/v0.32.0)（Rust 项目专用）；[nFPM `v2.47.0`](https://github.com/goreleaser/nfpm/releases/tag/v2.47.0)（deb/rpm/apk）；[GoReleaser `v2.18.0`](https://github.com/goreleaser/goreleaser/releases/tag/v2.18.0)（Go 中心）；rules_pkg（Bazel） | 自研胶水。没有工具把「六个异源上游二进制 + 第一方产物」组成一个离线包；候选都以单语言项目为中心 | 维持；重评触发点：需要 `.pkg`/`.deb`/`.dmg` 安装器时看 nFPM 与 cargo-dist |
| macOS dylib 收集 | 收集非系统 dylib、改写 install name、ad-hoc 签名 | 自研 helper 调 `otool`/`install_name_tool`/`codesign` | [macdylibbundler `63105a5`](https://github.com/auriamg/macdylibbundler/tree/63105a5571e0e9a83a8f2c37d0b91f2398b2031b)（不采用） | 借鉴想法；上游无发行制品、布局语义不合 | 维持；见 [macdylibbundler.md](./build-tools/macdylibbundler.md) |
| Rust 工具链 | 三平台同一 rustc | 1.98.0，Buck toolchains 声明官方归档 + SHA-256 | rustup | 二进制 | 维持 |
| DotSlash 引导、`buck2`、`btd`、`affected-targets` 启动器 | 引导与胶水 | 自研 shell 76 / 87 / 55 / 161 行 | `facebook/install-dotslash` Action（Linux CI 已用） | 胶水 | 维持 |

### D · 约束层里靠第一方代码实现的通用机制

| 机制 | 目的 | 现选择 | 业界最佳实践（候选钉来源） | 借用等级与取舍 | 建议 |
| --- | --- | --- | --- | --- | --- |
| 账本与比较并交换 | 唯一 metadata 权威；expected-version CAS；领域事件、幂等结果、outbox 同一事务 | 第一方自研于 SQLite（[系统边界 §控制面自己的存储](../design/spec/system.md#控制面自己的存储)） | SDK：[rusqlite `0.40.2`](https://crates.io/crates/rusqlite)（同步、bundled SQLite、MIT）vs [sqlx `0.9.0`](https://crates.io/crates/sqlx)（async、编译期校验）；事件溯源框架 [cqrs-es `0.5.0`](https://crates.io/crates/cqrs-es)、[disintegrate `4.0.0`](https://crates.io/crates/disintegrate)（Postgres 为主）；durable execution 见表 B | 自研 + rusqlite。CAS 是一条 `UPDATE … WHERE version = ?`，框架化反而带来第二套聚合模型；单写者与同步 rusqlite 贴合 | 维持自研；SDK 选 rusqlite，不把 async 运行时引进账本路径 |
| outbox / inbox | 意图先落账，适配器投递并回读；结果未知不重投 | 第一方自研 | 模式：[transactional outbox](https://microservices.io/patterns/data/transactional-outbox.html)；库：[effectum `0.7.0`](https://github.com/dimfeld/effectum)（自带独立 SQLite 文件，outbox 在 roadmap「later」，2024-07 后无发布）；[apalis-sqlite `1.0.0-rc.8`](https://crates.io/crates/apalis-sqlite)（独立存储，RC）；honker（Python） | 自研。三个候选都不在「同一本账本的同一事务」里，与[系统边界 §命令与跨服务正确性](../design/spec/system.md#命令与跨服务正确性)的事务边界冲突 | 维持自研（借鉴想法） |
| 租约与代次 | 单持有者、旧代次失权；六个代次槽各管一块资源 | 第一方自研（账本列 + CAS；[系统边界 §代次家族](../design/spec/system.md#代次家族)） | [Kleppmann《How to do distributed locking》](https://martin.kleppmann.com/2016/02/08/how-to-do-distributed-locking.html)的 fencing token；[etcd Lease API](https://etcd.io/docs/v3.5/learning/api/#lease-api)（TTL + keepalive）；Herdr 的 controller ownership（无代次、无 TTL，见 [herdr.md](./runtime/herdr.md#e-l1-herdr)） | 借鉴想法。分布式锁服务是超集，还要多一个网络服务 | 维持 |
| 现场 OS 锁 | `control.lock` 排他、Repo Instance 现场互斥 | 第一方 | [fd-lock `4.0.4`](https://crates.io/crates/fd-lock)（rustix flock 封装，MIT/Apache-2.0）；SQLite 自身文件锁 | SDK | 维持；用 fd-lock，不手写 flock FFI |
| 内容摘要 JCS + SHA-256 | 跨模块规范摘要 | 约束已定 RFC 8785 + SHA-256（[系统边界 §命令与跨服务正确性](../design/spec/system.md#命令与跨服务正确性)）；crate 未选 | [serde_jcs `0.2.0`](https://crates.io/crates/serde_jcs)（MIT/Apache-2.0，310 万下载，ryu-js 数字格式）；[serde_json_canonicalizer `0.3.2`](https://crates.io/crates/serde_json_canonicalizer)（MIT，640 万下载）；json-canon `0.1.3`（2023 后停更）；[sha2 `0.11.0`](https://crates.io/crates/sha2) | SDK。两者都几百行、无传递依赖负担；用 [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) 附录测试向量做 CT（[CT-SYSTEM](../design/contract-tests.md#ct-system--系统) 已列 JCS 一致性） | 由「可能手写」改 SDK：二选一，CT 钉 RFC 向量 |
| Context 组装与全文索引 | 萃取相关讨论、三档投喂、Manifest/Bundle 冻结；索引是可重建派生投影 | 第一方自研策略；索引选 SQLite FTS5（[交付文档 §技术基线](../design/delivery.md#技术基线)） | [FTS5](https://sqlite.org/fts5.html)（随 rusqlite bundled，BM25，同库同事务）；[tantivy `0.26.1`](https://crates.io/crates/tantivy)（MIT，独立索引目录，Lucene 级）；Meilisearch（独立服务）；生态四族见 [context-landscape](./context-landscape-20260824.md) | SDK（FTS5）+ 自研策略。tantivy 是超集，代价是第二份存储与进程内索引生命周期 | 维持；FTS5 |
| 终端票据与连接 | Attach Descriptor、Terminal Input Lease；观察、输入、接管、安全输入分权 | 第一方（账本行 + 短期票据）+ Herdr observe/control/takeover（[Participant 约束 §终端通道、连接与租约](../design/spec/participant.md#终端通道连接与租约)） | Herdr socket API（多观察者、单控制者、显式 takeover；无代次、无 TTL）；能力令牌 [biscuit-auth `6.0.0`](https://crates.io/crates/biscuit-auth)（Apache-2.0，可离线验证、可衰减） | 自研胶水 + 二进制。票据本地校验时不需要令牌库 | 维持；远程控制面阶段再评估 biscuit（「权限逐级缩小」与 attenuation 同构） |
| 备份 | 一致备份集：账本快照 + 定义字节 + 摘要 + schema 可读性校验 | 第一方自研（[系统边界 §备份与恢复](../design/spec/system.md#备份与恢复)） | [SQLite Online Backup API](https://sqlite.org/backup.html)（rusqlite `Connection::backup`）与 [`VACUUM INTO`](https://sqlite.org/lang_vacuum.html#vacuuminto)；[sqlite3_rsync](https://sqlite.org/rsync.html)（3.47 引入，3.50.0 去掉 WAL 与页大小限制，活库一致快照）；[Litestream `v0.5.17`](https://github.com/benbjohnson/litestream/releases/tag/v0.5.17)（Apache-2.0，WAL 流式复制到对象存储，独立进程） | SDK。快照用 SQLite 自带 API，备份集的清单与校验是胶水；Litestream 是多机阶段的超集 | 维持自研胶水；快照不手写文件复制 |
| Secret store | 密钥不进账本、Git、Room、Context | 约束定 OS secret store；crate 未选 | [keyring `4.2.0`](https://crates.io/crates/keyring)（MIT/Apache-2.0；macOS Keychain、Linux Secret Service/keyutils）；直接调 `security` / `secret-tool` CLI | SDK | 由「未选」定 SDK：keyring |
| Harness 接入 | 探测能力、派工、结果提案 | ACP + 原生服务端（[harness-access.md](./harness-access.md#e-l1-harness-access)） | [agent-client-protocol `2.0.0`](https://crates.io/crates/agent-client-protocol)（Apache-2.0，schema [`v1.21.0`](https://github.com/agentclientprotocol/agent-client-protocol/releases/tag/schema-v1.21.0)）；OpenCode OpenAPI 3.1 SDK；Pi JSONL RPC | SDK | 维持 |
| provider 适配客户端 | Matrix、Vikunja、Dagu、Herdr、GitHub、Linear 的类型与调用 | 待建（control P2） | Matrix：[ruma `0.16.0`](https://crates.io/crates/ruma) + [ruma-appservice-api `0.16.0`](https://crates.io/crates/ruma-appservice-api)（MIT）+ [reqwest `0.13.4`](https://crates.io/crates/reqwest)；[matrix-sdk `0.18.0`](https://crates.io/crates/matrix-sdk)（Apache-2.0，含 E2EE/store 超集，`crates/` 目录已无 appservice crate）。Vikunja：[`pkg/swagger/swagger.json`](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/swagger/swagger.json)（Swagger 2.0，425 KB）→ 先转 OpenAPI 3 再 [progenitor `0.14.0`](https://crates.io/crates/progenitor)（MPL-2.0）或 openapi-generator。Dagu：[`api/v1/api.yaml`](https://github.com/dagucloud/dagu/blob/532c512944b2e5eb8991b5bc7cbeafa74fd5b47a/api/v1/api.yaml)（OpenAPI 3.0.0）→ progenitor。Herdr：`herdr api schema --json` → [typify `v0.7.0`](https://github.com/oxidecomputer/typify/releases/tag/v0.7.0)。GitHub：[octocrab `0.54.1`](https://crates.io/crates/octocrab)。Linear：[graphql_client `0.16.0`](https://crates.io/crates/graphql_client) | SDK / 生成 | 预防性建议：不手写 HTTP 客户端，从 schema 生成或用类型库；Vikunja 的 2.0 → 3.0 转换进 Buck action |
| Workflow 编译与 lint | HCTL JSON → 受限 Dagu YAML；schema、引用、环 lint | 第一方自研 | [jsonschema `0.52.1`](https://crates.io/crates/jsonschema)（MIT）+ [petgraph `0.8.3`](https://crates.io/crates/petgraph)（环检测）+ Dagu `validate` | SDK + 胶水 | 维持 |
| 运行时服务生命周期 | 一键启停、顺序、健康检查、重启；由 control 托管 | 第一方 shell：[`hctl2-services`](../../src/packaging/dependencies/hctl2-services) + [`runtime.sh`](../../src/packaging/dependencies/common/runtime.sh)、`start.sh`、`stop.sh`、`status.sh`（PID 文件、`kill -0`、`sleep` 轮询，675 行）；P2 由 control 接管 | [Process Compose `v1.122.0`](https://github.com/F1bonacc1/process-compose/releases/tag/v1.122.0)（开发侧已采用；REST API + Unix socket、readiness/liveness、restart policy、detached）；systemd user units / launchd（OS 原生，两套模板）；supervisord（Python） | 现状是自研监督器；Process Compose 是二进制且已审。代价：用户包 +15–16 MB、多一个进程；收益：control 不再写 PID、健康、重启，只做顺序、备份、升级编排 | **由自研改借用（候选，需所有者拍板方向）**：control 经 Process Compose API 托管服务；P2 首次消费前限时验证 |

## 二、自研部分是不是胶水

判据：把 `hctl2-control` 按[系统边界 §组件](../design/spec/system.md#组件)与共享机制各节拆成条目，把 `hctl2-tool` 按 [decision-history §33](../design/references/decision-history.md#33-hctl2-tool-定界为现场执行者v0153) 的五项拆开，每条问「业界有没有现成的」。答案分三种：有二进制或 SDK 就直接借；只有模式就借想法；都没有才是差异化自研。**胶水**指借来的东西之间的翻译与编排；**非胶水**指我们自己定义语义的部分。

### `hctl2-control` 十八条

| 条目 | 约束出处 | 业界有没有现成的 | 判断 |
| --- | --- | --- | --- |
| 命令信封：幂等键、expected version、actor 来源五类、规范输入摘要 | [系统边界 §命令与跨服务正确性](../design/spec/system.md#命令与跨服务正确性) | 想法有：Idempotency-Key（Stripe、IETF httpapi 草案）、乐观并发；没有库能表达 `direct_client / provider_event / internal_reducer / execution_principal / unknown` 五类来源 | 维持自研，非胶水（产品核心） |
| 用户级 SQLite 账本 | [§控制面自己的存储](../design/spec/system.md#控制面自己的存储) | rusqlite | 胶水（SQL + 迁移） |
| outbox/inbox 与确认回读 | [§外部权威副作用](../design/spec/system.md#外部权威副作用) | 模式有，库不在同一事务（表 D） | 维持自研，薄：一张表加一个投递循环 |
| 单写者锁 + `control_writer_generation` | [§单写者](../design/spec/system.md#单写者) | fd-lock + CAS | 胶水 |
| 六个代次槽与推导禁令 | [§代次家族](../design/spec/system.md#代次家族) | fencing token 是想法来源 | 自研，非胶水（HCTL 语义） |
| Verdict / Receipt 归约、法定票数、regate | [Run 约束](../design/spec/run.md#request重试与-gate)；[HCTL1 谱系](./lineage/hctl1.md) | 没有——[方法论生态审计](./methodology-landscape-20260824.md)的结论是全生态无人把完成判定权收归有权人类或确定性归约 | 自研，差异化核心 |
| Extension Revision / Resolved Port Binding | [§固定内核与受控端口](../design/spec/system.md#固定内核与受控端口) | 想法：锁文件（`Cargo.lock`、`lock.json`）与 Nix 派生的「固定输入」 | 自研，薄 |
| 启动与恢复七步对账 | [§启动与恢复](../design/spec/system.md#启动与恢复) | 想法：durable execution 的 replay、Saga 补偿 | 自研，非胶水 |
| 一致备份集 | [§备份与恢复](../design/spec/system.md#备份与恢复) | SQLite Online Backup API | 胶水 |
| Context Manifest / Bundle、萃取、三档投喂、可选 small-brain | [Project 约束 §Context、Memo 与 Artifact](../design/spec/project.md#contextmemo-与-artifact)；[Context 设计正文](../design/context.md#萃取与压缩中心设计) | 索引：FTS5；策略：[生态四族](./context-landscape-20260824.md)仅参考行为 | 选材策略自研（非胶水），索引胶水 |
| Herdr 适配 | [Participant 约束 §终端通道、连接与租约](../design/spec/participant.md#终端通道连接与租约) | typify 从 `herdr api schema --json` 生成类型 | 胶水 |
| Matrix 适配：AppService 注册、事件读取、身份映射 | [Project 约束 §Room 与消息](../design/spec/project.md#room-与消息) | ruma 类型 + reqwest | 胶水 |
| 任务后端适配：Snapshot、条件写入、回读（Vikunja / GitHub / Linear） | [Task 约束 §外部概念对齐](../design/spec/task.md#外部概念对齐) | 生成客户端、octocrab、graphql_client | 胶水 |
| Dagu 适配：编译、注册、`human.task` 等待/完成/回读 | [Run 约束 §外部概念对齐](../design/spec/run.md#外部概念对齐) | progenitor 从 `api/v1/api.yaml` 生成 + Dagu `validate` | 胶水（编译器自研但薄） |
| 执行面服务生命周期托管 | [§组件](../design/spec/system.md#组件)（hctl2-control 托管一键启停） | Process Compose | **可改借用**（表 D 末行） |
| Secret store | [§控制面自己的存储](../design/spec/system.md#控制面自己的存储) | keyring | 胶水 |
| 派生投影与全文索引 | [Project 约束 §Context、Memo 与 Artifact](../design/spec/project.md#contextmemo-与-artifact) | FTS5 | 胶水 |
| RPC / API schema | `.memo/notes/control-api-schema-20260902.md` | tonic、jsonrpsee、Tauri IPC | 待定；届时是胶水 |

十八条里：胶水九条，薄自研两条（outbox、绑定冻结），可改借用一条，待定一条，非胶水五条。五条非胶水——命令信封、代次、Verdict/Receipt 归约、恢复对账、Context 选材——加两条薄自研，正好覆盖[愿景文档 §产品原生核心与架构最小内核](../design/vision.md#产品原生核心与架构最小内核)的五行最小内核。这不是巧合：最小内核就是「换掉一切之后什么必须仍然成立」，业界当然没有现成的。

### `hctl2-tool` 五项

| 条目 | 业界现成的 | 判断 |
| --- | --- | --- |
| worktree / ChangeSet 物化与隔离 | `git worktree add / remove / prune` | 胶水 |
| 已持久化意图的执行与回读（本地集成） | `git merge --ff-only`；`git update-ref <ref> <new> <old>` 就是原子的 expected-head 比较并交换；`git rev-parse` 回读 | 胶水 |
| 现场 OS 锁与 fence | fd-lock；`site_generation` 在账本推进 | 胶水 |
| 封存保全 | `git worktree remove`、`git bundle`、refs 归档 | 胶水 |
| 判决结晶副本写入 | `git hash-object` / `commit-tree` / `update-ref`，或 porcelain 提交 | 胶水 |
| （附）Repo Instance 身份读取 | `git rev-parse --git-common-dir`、`git config` | 胶水 |

`hctl2-tool` 是纯胶水，这与 §33 的「进程级动作一律转调业界工具，零重实现」一致。gix 或 git2 只在子进程开销成为瓶颈时再评估；现阶段没有理由链接一个 Git 实现进二进制。

### 第一方 shell（约 3,700 行）分类

| 块 | 行数 | 判断 |
| --- | --- | --- |
| 发行与依赖打包（`packaging/release`、`packaging/dependencies/common` 的组包与测试、安装器） | 1,279；另有 `platforms/` 下 691 行 OS 专属脚本 | 胶水，最大一块；维持（表 C「发行组装」） |
| 运行时服务生命周期（`hctl2-services`、`common/runtime.sh`、`start.sh`、`stop.sh`、`status.sh`） | 675 | 重写监督器；建议改 Process Compose（表 D 末行） |
| 文档检查与四轴清点（`build/docs`） | 617 | 已审，维持（[docs-lint.md](./docs-lint.md)） |
| 启动器与引导（`buck2`、`btd`、`reindeer`、`install-dotslash`、`affected-targets`） | 445 | 胶水；`reindeer` 一处改官方二进制（表 C） |

<a id="三审计基线来源与版本"></a>
## 三、审计基线（来源与版本）

复核方式：上游最新以 2026-09-02 `gh api repos/<owner>/<repo>/releases/latest` 为准（无 release 的取最新 tag）；crate 数据取 crates.io API 同日快照；文件级事实取钉定 commit 的 `contents` API。

| 对象 | HCTL2 钉定 | 上游最新（2026-09-02） | 来源 |
| --- | --- | --- | --- |
| Tuwunel | `v1.9.0 / 5b366914` | `v1.9.0` | `src/packaging/dependencies/lock.json` |
| Cinny | `v4.12.6 / 33f4ba36` | `v4.12.6` | 同上 |
| Static Web Server | `v2.44.0 / 27aa3450` | `v2.44.0` | 同上 |
| Vikunja | `v2.5.0 / ef2200e9` | `v2.6.0`（2026-08-31） | 同上 |
| Dagu | `v2.15.1 / 532c5129` | `v2.16.1`（2026-08-31） | 同上；`api/` 目录在钉定 commit 只有 `v1`，`api/v1/api.yaml` 首行 `openapi: "3.0.0"` |
| Herdr | `v0.8.2 / 9eb5214` | `v0.8.2` | 同上；文档树含 `socket-api.mdx`（`herdr api schema --json`） |
| Buck2 | `2026-08-22` | `2026-09-01` | `src/build/tools/buck2-bin` |
| bazel-remote | `v2.6.2` | `v2.6.2` | `src/build/tools/bazel-remote-bin` |
| Process Compose | `v1.122.0` | `v1.122.0` | `src/build/tools/process-compose-bin` |
| Syft | `v1.51.1 / 91a0032` | `v1.51.1` | `src/packaging/release`（Buck `http_archive`） |
| DotSlash | `v0.5.9` | `v0.5.9` | `src/build/tools/dotslash.env` |
| BTD | `2026-08-20 / 345497d` | `2026-09-01` | `src/build/tools/btd` |
| jq | `jq-1.8.2` | `jq-1.8.2` | `src/build/tools/jq-bin` |
| Reindeer | `v2026.08.24.00 / 84035a20`（源码编译） | `v2026.08.31.00`；两个版本的 release 均带 8 个二进制 asset | `src/build/tools/reindeer.env`；`gh api …/releases` assets |
| actionlint / ShellCheck | `1.7.12` / `0.11.0` | 同 | `src/build/tools/*-bin` |
| Rust | `1.98.0` | — | `src/rust-toolchain.toml` |
| Restate | — | `v1.7.8`；服务端 LICENSE 为 Business Source License 1.1；`restate-sdk` crate `0.12.0` MIT | `gh api …/contents/LICENSE` |
| DBOS Transact | — | `dbos-transact-py 2.31.0`，MIT；无 Rust 实现 | GitHub |
| Temporal | — | `v1.31.2`，MIT | GitHub |
| matrix-rust-sdk | — | `matrix-sdk 0.18.0`；`crates/` 目录无 appservice crate；`ruma-appservice-api 0.16.0` 独立存在 | `gh api …/contents/crates`；crates.io |
| Vikunja API 规格 | — | `pkg/swagger/swagger.json` 首字段 `"swagger": "2.0"`，425,628 字节 | 钉定 commit `contents` API |
| effectum | — | `0.7.0`（2024-07-23），自带独立 SQLite 文件，outbox 在 roadmap「later」 | GitHub README |
| serde_jcs / serde_json_canonicalizer / json-canon | — | `0.2.0` / `0.3.2` / `0.1.3`（后者 2023-05 后停更） | crates.io |
| rusqlite / sqlx / keyring / fd-lock / tantivy / clap | — | `0.40.2` / `0.9.0` / `4.2.0` / `4.0.4` / `0.26.1` / `4.6.6` | crates.io |
| gix / git2 / octocrab / graphql_client / jsonschema / petgraph / progenitor / typify | — | `0.87.1` / `0.21.0` / `0.54.1` / `0.16.0` / `0.52.1` / `0.8.3` / `0.14.0` / `v0.7.0` | crates.io、GitHub |
| agent-client-protocol | — | crate `2.0.0`；schema `v1.21.0` | crates.io、GitHub |
| sqlite3_rsync | — | 3.50.0 去掉 WAL 与页大小限制；活库一致快照 | sqlite.org/rsync.html |
| Litestream | — | `v0.5.17`，Apache-2.0 | GitHub |
| 其余候选 | — | Continuwuity `v26.7.2`、Synapse `v1.159.0`、Element Web `v1.12.27`、Caddy `v2.11.4`、miniserve `v0.35.0`、Planka `v2.2.1`、git-bug `v0.10.1`、Conductor `v3.32.1`、Windmill `v1.801.0`、tmux `3.7c`、Zellij `v0.45.1`、Bazel `9.2.0`、Pants `2.33.1`、NativeLink `v1.6.6`、overmind `v2.5.1`、pueue `v4.0.4`、Trivy `v0.74.0`、cargo-cyclonedx `0.5.9`、mise `v2026.9.0`、aqua `v2.62.3`、Hermit `v0.52.3`、bazel-diff `v46.1.0`、gojq `v0.12.19`、jaq `v3.1.1`、zizmor `v1.30.0`、lychee `v0.24.2`、markdownlint-cli2 `v0.23.2`、vale `v3.19.0`、cargo-dist `v0.32.0`、nFPM `v2.47.0`、GoReleaser `v2.18.0`、Tauri `2.11.5`、Electron `v44.1.1`、xterm.js `6.0.0`、Tiptap `v3.31.0`、React Aria Components `1.20.0`、biscuit-auth `6.0.0`、tonic `0.14.6`、jsonrpsee `0.26.0` | GitHub releases / crates.io |

## 复核记录

- 2026-09-02 首发。

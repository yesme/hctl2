# 自建方案盘点 · 构建、发行与运维（DeepSeek 席）

> 状态：已落地 · 待 Claude 汇总
> 基线：main @ f531ca1（草案 v0.19.2）
> 去向：`.memo/design/community-first-audit-20261005.md` §六（汇总与待裁清单）

## 范围与方法

范围按任务书 §三「DeepSeek」行：`src/build`、`src/packaging`、`.github/workflows`、仓库根 `run` 与各 `BUCK`/脚本、控制面里的托管服务生命周期与密钥后端、文档检查器。对应的设计文档（`delivery.md`、`docs/research/build-tools/`、`docs-lint.md`、`gitea.md`、`matrix-homeserver.md`）一并读了。

方法：先读既有调研（2026-09-02 部件矩阵、`docs/research/build-tools/` 十项、`docs-lint.md`、`libs/keyring.md`），已论证过的不重新论证；社区方案的**版本、许可、维护状态与「到底有没有那个能力」于 2026-10-05 逐个打开官方仓库/release 页核对**；没核过的点单独标「印象」。范围外顺手看到的标「范围外」。

## 一、复核既有审计（第 5 类：当初的理由现在还成立吗）

### 1. 随包服务生命周期与进程监督（类 1 + 2）

- **位置**：`src/packaging/dependencies/hctl2-services`（191 行 bash）、`src/packaging/dependencies/process-compose/*.yaml`（7 个清单共 200 行）、`src/apps/control/src/services.rs`（1,093 行）、`platforms/*/runtime.sh`（71 行）。
- **自建了什么**：一层「HCTL2 语义」的监督胶水——state root、consumed 记录、start/stop/restart/status/smoke 五个动词、健康快照（running/ready）、备份与恢复。进程本体已交给 Process Compose。
- **它解决什么问题**：让 control 能按需拉起/停掉随包服务（Tuwunel、Gitea…），并在 `status` 里给出可核对的 `running`/`ready`。
- **社区方案**：Process Compose `v1.122.0`（Apache-2.0；2026-08-18 发布，2.8k star，活跃；已核对）。要的几件事——进程编排与依赖顺序、恢复/重启策略、健康检查（liveness+readiness）、CLI+REST API、命名空间——都在它手里；我们**已经采用**（部件矩阵 I-10 已落地）。
- **当初为什么自建**：部件矩阵第 2 条判「675 行 shell 的 PID 文件、`kill -0`、`sleep` 轮询是在重写一个进程监督器」，应改 Process Compose；I-10 已落地。剩下的自建是 consumed/backup/restore 这类产品语义，不在 Process Compose 的范围里。
- **换过去要动什么**：没有可再换的部分。若要把 191 行 `hctl2-services` 也去掉，需要把它的 5 个动词与 state-root 处理并进 control，涉及 30+ 调用点（`test-package.sh`、演示手册、各处测试）。
- **建议**：**维持**（早已换成社区监督器；余下是最小语义层）。现在在「借鉴想法」，没有更上一级可挪。
- **把握**：查证（Process Compose release 页已打开；仓内文件与调用点已读）。

### 2. 发行组装（类 5）

- **位置**：`src/packaging/release/`（`assemble.sh` 305、`defs.bzl` 175、`test-package.sh` 276、`test-toolbox.sh` 284、`install.sh` 119、`export-first-party.sh` 57 行，共 1,216 行）＋ `src/packaging/dependencies/`（`common/` 1,230、`platforms/` 780、`install-package.sh` 107 等，共约 2.6k）。合计约 3.8k 行。
- **自建了什么**：把六个异源上游制品（Tuwunel、Cinny、Vikunja、Dagu、Herdr、Gitea）＋ 第一方产物组成三平台离线包，生成 checksums、SBOM、release manifest，并跑离线安装与生命周期测试。
- **它解决什么问题**：`delivery.md` 的交付形状——一个自带全部依赖的离线包，装上就能跑。
- **社区方案**（2026-10-05 逐个打开 release 页核对）：
  - cargo-dist `v0.33.0`（Apache-2.0/MIT，2026-09-10）——面向**本项目**产物出 tarball/installer + manifest + checksums；无多源聚合，无 SBOM，无离线生命周期测试。
  - nFPM `v2.47.0`（MIT，2026-06-20）——只做 deb/rpm/apk。
  - GoReleaser `v2.18.2`（MIT，2026-09-17，16k star）——**最接近**：归档、checksums、SBOM（syft 管道）、签名、多通道发布、manifest 都有；但同样以「构建本项目产物」为中心，不覆盖「六个异源上游二进制聚合」与离线安装生命周期测试。
  - rules_pkg `1.3`（Apache-2.0，2026-05-05）——Bazel 规则集，需整体迁入 Bazel 才有意义；同样缺聚合/SBOM/生命周期。
- **当初为什么自建**：部件矩阵表 C（2026-09-02）：「没有工具把『六个异源上游二进制 + 第一方产物』组成一个离线包；候选都以单语言项目为中心」。
- **换过去要动什么**：若换 GoReleaser，要把 Buck2 的产物喂给它并将 manifest/SBOM/测试重写一遍，收益主要是 checksums/manifest 那几百行；风险是整条发行链路（三平台 + 离线测试）重构。其余候选连一半都覆盖不到。
- **建议**：**维持**——第 5 类复核后理由仍成立。重评触发点：需要 `.pkg`/`.deb`/`.dmg`，或 GoReleaser 支持外部制品聚合（可**去上游提需求**）。现在在「自研」；换完也只到「SDK/二进制」，因为聚合与生命周期测试仍得自己写。
- **把握**：查证（四个候选的 release 页均已打开；仓内规模已数）。

### 3. macOS dylib 收集（类 5）

- **位置**：`src/packaging/dependencies/platforms/macos/`（`common.sh` 139、`bootstrap.sh` 117、`package.sh` 104、`tuwunel.sh` 135、`tuwunel-prebuilt.sh` 45 行，共约 540 行）。
- **自建了什么**：收集非系统 dylib、改写 install name、ad-hoc 签名（调 `otool`/`install_name_tool`/`codesign`）。
- **它解决什么问题**：让 macOS 产物在没有开发机的环境里也能加载它依赖的第三方 dylib。
- **社区方案**：macdylibbundler `1.0.5`（MIT）——**能力其实覆盖**：README 载明用 `otool -L` 遍历依赖、拷 dylib 进包、把 install name 改成 `@executable_path` 相对路径、ad-hoc codesign 可开关，2022-01 起还支持 re-exported dylib。**但它停滞**：1.0.5 的 tag 与最后一次提交都是 2022-12-05，此后无活动，也没有上游发行制品。
- **当初为什么自建**：`build-tools/macdylibbundler.md`（2026-08-31）判「不采用：布局与冲突语义不符，且无上游发行制品」。
- **换过去要动什么**：把三步换成调 dylibbundler；风险是把一个近四年未维护的工具塞进发行链，而现链路已能跑通。
- **建议**：**维持**（矩阵结论仍成立），但要**补记一条事实**：它缺的不是能力而是维护与制品；它若恢复维护，值得重评（那时可**去上游提需求**要发行制品）。现在在「自研」；换完到「自研（调外部二进制）」。
- **把握**：查证（仓库首页与提交页已打开）。

### 4. 文档检查器（类 5）

- **位置**：`src/build/docs/`（10 个 Buck2 `sh_test` 目标；脚本 802 行＋4 个 allowlist＋3 个词表）＋ `src/build/tests/` 里的夹具测试（1,704 行中的一部分）。
- **自建了什么**：常驻 CI 的五项机械检查——链接与 `#锚点` 可达、版本戳一致、死名扫描、禁令密度（报告制）、`.memo/review` 目录与文件头基线；后四项是本库私有语义。
- **它解决什么问题**：文档大修后不让链接、戳、退休词回潮。
- **社区方案**：lychee（MIT/Apache-2.0；最新 `v0.24.2`，4k star，仍在维护；已核对）覆盖「链接与锚点」这一项；但按原审计，中文标题的 slug 需与 GitHub 逐字节一致，其 Markdown 锚点实现与本库链接形态（含 `<a id>`）的兼容性要逐案验证，且它默认是网络姿态，与「离线、确定性」的 gate 要求冲突。
- **当初为什么自建**：`docs-lint.md`（2026-08-31）——五项里四项无现成覆盖，引入第二个工具族只为剩下一项不划算；回退路径已写明（日后要用 lychee，走既有的 DotSlash 固定工具机制）。
- **换过去要动什么**：只剩外链检查，而那是本库**明确不做**的；无可换。
- **建议**：**维持**。复核结论：lychee 仍活跃，但原理由（四项私有语义＋离线确定性）未变；**未复测**的是「lychee 对中文 slug 的行为」这一条（标印象）。
- **把握**：查证（lychee release 页已打开）＋ 一处印象（中文 slug 行为未复测）。

### 5. CI 路径过滤与增量验证（类 4 + 5）

- **位置**：`.github/workflows/*.yml`（共 1,140 行，其中自写 shell 步骤 17 处）、`src/build/ci/affected-targets`（163 行，内包官方 btd）、`src/build/ci/clippy-coverage`、`src/build/tests/check_validation_range.sh` 等夹具。
- **自建了什么**：①「PR 只验增量、旧 head 已绿可跳过」的选择逻辑（读历史 run、判 before/head 关系、输出 mode）；②粗过滤：按改动路径决定跑哪些 job。
- **它解决什么问题**：在 required check 语境下，既不让全量跑，也不让增量漏。
- **社区方案**（2026-10-05 打开核对）：
  - `dorny/paths-filter v4.0.3`（MIT，2026-08-05，活跃）——按 glob 过滤变更文件并输出 `true/false`、count 与文件列表；只覆盖上面 ②。
  - `tj-actions/changed-files v47.0.6`（MIT，2026-04-18；此后 main 有 26 个提交无新 release，单人维护）——同 ② 且更重。
  - GitHub 原生 `on.pull_request.paths`——最粗的一层，同样只覆盖 ②。
  没有候选覆盖 ①：那正是我们自写的东西。
- **当初为什么自建**：`build-tools/github-actions-incremental-validation.md`——复用旧 head 的成功 workflow 证据，快进增量验证，历史改写时全量回退。
- **换过去要动什么**：把 ② 换 dorny/paths-filter 可减掉一部分 shell，但要在 10 处左右重连 `if:`/outputs，收益小；① 无候选可换。
- **建议**：**维持**（① 无现成；② 换的收益不足以动 CI 骨架）。留意：若将来只需要 ②，直接用原生 `paths:` 或 paths-filter 即可。
- **把握**：查证（两个 action 的 release 页已打开；workflow 与脚本已读）。

### 6. 密钥后端（类 2）

- **位置**：`src/crates/foundation` 的 `SecretStore`（约 200 行）＋ `src/apps/control/src/config.rs`（132 行）。
- **自建了什么**：后端选择（探测缺省／显式 `system-keyring`／`user-file`）与 0600 文件的 `user-file` 回退。
- **它解决什么问题**：让 control 在无屏会话（CI、纯终端）里也能拿到 Gitea/Matrix 的令牌，同时不把密钥写进账本。
- **社区方案**：`keyring` `4.2.0`（MIT OR Apache-2.0；2026-08-29，活跃，已迁至 open-source-cooperative；`keyring-core` 1.0.0 拆分为多 store crate）——**我们已在用**它的原生库访问。**user-file 回退在上游没有等价物**（上游 store 都是 OS 原生库或 Secret Service）。
- **当初为什么自建**：所有者 2026-09-04 裁定「探测到钥匙串就用钥匙串，探测不到退 0600 文件」；`libs/keyring.md` 记了这条与四级尺子。
- **换过去要动什么**：无可换。若将来要更好的文件后端（`age`/`sops` 之类），是另一次选型。
- **建议**：**维持**（回退属策略要求，不是可用库能替的）。原生库那半停在「SDK」，回退那半停在「自研」，都没有可挪的一级。
- **把握**：查证（keyring 页面已打开；仓内实现已读）。

## 二、新发现（既有调研未覆盖的）

### 7. `run`：十家 harness 的启动器（类 4）

- **位置**：仓库根 `run`（319 行 bash）。
- **自建了什么**：每家 harness 一个 worktree、会话 id 表、启动前 fetch 与快进家分支、`--print`/`--session-id`/`--no-sync` 三种入口。
- **它解决什么问题**：多 harness 并行时各自工作树与会话不串味。
- **社区方案**：无。mise/just/devbox 这类工具管环境与任务，不涉及 agent 会话与 worktree 约定；会话 id 是各家 CLI 的私有状态。
- **当初为什么自建**：没有找到记录；约定见 `.memo/notes/ubuntu-multi-harness-worktree-setup-20260822a.md`。
- **换过去要动什么**：不适用。
- **建议**：**该自建**（保留）。
- **把握**：查证（脚本已读）。

### 8. `src/buck2` 包装的主机工具解析（类 4）

- **位置**：`src/buck2`（145 行 bash）。
- **自建了什么**：在启动器里解析宿主 python3、clang/clang++/ar（macOS 优先 `/usr/bin` 的 xcrun 垫片），并以 `--config hctl2.*` 注入。
- **它解决什么问题**：Buck2 的 prelude 在无 PATH 的 hermetic 环境里跑：`python3` 会解析到系统 3.9，C/C++ shim 用 `os.execl` 不搜 PATH。
- **社区方案**：Buck2 自身的工具链机制——`build/toolchains/BUCK` 已把这些 config 接进 toolchain（这部分是原生的）；但「探测宿主工具绝对路径」这件事 Buck2 不提供。
- **当初为什么自建**：脚本注释写明了原因（同上一格的引文）。
- **换过去要动什么**：不适用；可**去上游提需求**（给宿主工具链探测/exec-path 注入一个原生入口）。
- **建议**：**维持 + 去上游提需求**。现在在「自研」；上游若提供，可到「二进制/SDK」。
- **把握**：查证（脚本已读）。

### 9. macOS Tuwunel 自建制品（类 5）

- **位置**：`.github/workflows/tuwunel-macos.yml`（76 行）＋ `platforms/macos/tuwunel.sh` 与 `tuwunel-prebuilt.sh`（180 行）。
- **自建了什么**：为 macOS arm64/x86_64 原生编 Tuwunel 制品（Buck2 构建、缓存关闭、attest 出处）。
- **它解决什么问题**：Tuwunel 上游不发 macOS 制品，而演示要在 macOS 上跑。
- **社区方案**：上游 `matrix-construct/tuwunel v1.9.3`（Apache-2.0；2026-09-25；非常活跃，2.6k star）——**34 个资产全部是 `*-linux-gnu`（.zst/.deb/.rpm/.nix/.oci/docker）＋ 源码，没有任何 apple-darwin**；v1.8.0 同样只有 linux。**不能替代**。
- **当初为什么自建**：上游不发 macOS 制品（部件矩阵亦记「HCTL2 托管的 macOS Tuwunel 预编译制品」）。
- **换过去要动什么**：上游一旦提供 macOS 制品，删掉该工作流与两个脚本约 250 行，改为按 I-09 的方式（DotSlash/lock 清单）消费官方制品。
- **建议**：现状**维持**＋**去上游提需求**（增加 apple-darwin 目标/制品）。第 5 类复核：理由仍成立。现在在「自研」；上游若发制品，直接到「二进制」。
- **把握**：查证（上游 release 的两个版本资产清单已打开）。

### 10. 对照：已经被社区接管的点（不算自建，作为借用程度的证据）

`reindeer`（已是 DotSlash 官方二进制 `v2026.08.24.00`，I-09 已落地）、`dotslash`、`btd`（Buck2 官方 Change Detector）、`jq`、`syft`（SBOM）、`actionlint`、`shellcheck`、`bazel-remote`（CI 缓存本体，`v2.6.2`，2026-07-23，活跃）、`process-compose`、`actions/{checkout,cache,attest-build-provenance,upload-artifact}`。另：`packaging/dependencies/fixups/` 是 reindeer 的原生机制，不算自建。

## 三、应该自己写的（短清单）

1. `run`：agent 会话与工作树编排，无社区对应物。
2. 控制面的服务语义层（consume/健康快照/备份恢复/status）：产品语义，Process Compose 不管这些。
3. `hctl2-services` 的 state-root/`--no-wait`/`smoke`：随包环境的私有约定；去掉要改 30+ 调用点。
4. `user-file` 密钥回退：策略要求（所有者 2026-09-04 裁定），上游无等价物。
5. 四项私有语义的文档检查（版本戳、死名、禁令密度、memo 基线）：通用工具不覆盖。
6. 发行组装的异源聚合与离线生命周期测试：候选工具都以单语言项目为中心。
7. macOS Tuwunel 制品：上游无 macOS 资产，属临时自建（去上游提需求后可退役）。

## 四、按「换过去的好处与代价」排（前五）

| # | 换什么 | 好处 | 代价 | 结论 |
| --- | --- | --- | --- | --- |
| 1 | 发行组装 → GoReleaser `v2.18.2` | 白拿 checksums/SBOM/manifest/多通道发布（约几百行） | 整条发行链路重构；异源聚合与离线生命周期测试仍要自写；三平台要重验 | 维持，等它支持外部制品聚合 |
| 2 | CI 粗过滤 → `dorny/paths-filter v4.0.3` | 减掉一部分 workflow shell | 10 处左右 `if:`/outputs 重连；增量验证那半仍需自写 | 维持（收益小） |
| 3 | macOS dylib → macdylibbundler | 少 540 行胶水 | 工具 2022-12 起停更；现链路已通 | 维持，等上游恢复维护再评 |
| 4 | macOS Tuwunel → 上游制品 | 删约 250 行；制品随上游安全更新 | 需要上游先支持 apple-darwin | 去上游提需求 |
| 5 | `hctl2-services` → 并进 control | 少一个随包脚本 | 改 30+ 调用点；换不到社区组件 | 维持 |

## 五、这一块整体借用到了什么程度

**工具层几乎全借**：构建用 Buck2（官方二进制，DotSlash 锁定）、依赖生成用 Reindeer（官方二进制，I-09 已落地）、变更选择用官方 btd、缓存用 bazel-remote、进程监督用 Process Compose、SBOM 用 syft、CI 用官方 Actions（含 attest）、脚本质量用 actionlint/shellcheck、解压用上游 xz/zstd——都消费官方制品，版本与摘要进锁文件。**自研集中在胶水与私有语义**：发行组装约 3.8k 行、CI 编排约 1.1k、文档检查约 0.8k、两个启动器约 0.5k、服务语义层约 1.3k。本轮**没有发现「在已采用组件旁边重做其能力」的新案例**：唯一的历史案例（进程监督）已由 Process Compose 接管（矩阵 I-10）；剩下的可换点都是「局部环节」而非「整条链路」，且换过去后仍需自写核心部分（异源聚合、离线生命周期测试、增量验证选择）。**范围外**顺手看到一处：`docs/research/component-matrix-20260902.md` 的基线是 2026-09-02，其中「Reindeer 源码编译」已在本轮复核中确认退役，汇总时请以本报告为准。

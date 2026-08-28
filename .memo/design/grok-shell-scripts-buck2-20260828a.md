# 仓库 shell 脚本盘点：为何存在、哪些该进 Buck2

> 状态：已落地 · shell-as-Make 已核销<br>
> 基线：main @ f6c025e<br>
> 去向：`src/packaging/` Buck 原生输出与测试目标、删除 uname 分发器；运行期脚本等 P2 CLI 另议。<br>
> 日期：2026-08-28<br>
> 说明：Grok 按职责自分类的施工备忘，不改规范。边界与 [Buck2 替换方案](./codex-buck2-replacement-20260827.md) 一致：不进入上游内部构建图。确定性诉求见 [构建确定性备忘](./grok-20260827a.md)。<br>
> 范围：仓库内带 shebang 的脚本与 DotSlash 入口；不含 GitHub Actions YAML 里的 inline shell。

## 怎么来的

第一方进入 Buck2 图后，盘点时 `src/` 仍有约 3070 行 shell 与可执行入口。所有者问：每份为什么存在，能不能用 Buck2 native 换掉。本文按**职责**分成六类，不按目录堆文件名。

## 总判

| 类 | 体积（约） | Buck2 native？ |
| --- | --- | --- |
| A 构建引导 | 160 行 | 鸡生蛋，留薄引导；`merge_sysroot.sh` 已是 genrule |
| B `uname` 分发器 | 220 行 | **已删除**，换成 `--target-platforms` / `select` |
| C 锁、下载、组包（shell 当 Make） | 2000 行 | metadata、下载、准备与组包均为声明输入/输出；shell 只留作 action body |
| D 装到用户机器上的启停 | 800 行 | **不该当构建问题**；`export_file` 打进包，消灭要等 P2 CLI |
| E 发行组装 | 600 行 | `complete` 目录输出与 `complete-test` 已进图；shell 只留作 action/test body |
| F 与产品构建无关 | `run` 222 行 | 不进 `src/` 图 |

该用 Buck2 换掉的是「shell 当 Make」。不该换的是用户机器上的启停，以及 Tuwunel/Vikunja 自己怎么编。

## 2026-08-28 实施更新

`5d4297f` 完成第一步：`lock.json` 成为 Buck 的下载事实源，官方 blob 和 macOS Tuwunel Rust 组件由 `http_file`/`http_archive` 获取与校验；平台 `select` 选择对应 action；平行 fingerprint 和最终依赖包 cache 被删除。后续又把单一 `prepared` 拆成六个组件 action，组件与配置变化不再相互失效，更不会连带触发 Tuwunel Cargo，并删除不可变 action 内无效的 `.hctl2-*-sha256` marker。

第二步完成全部收口：`lock.json` 还成为版本、target identity、运行端口和确定性时间的唯一 metadata 事实源；Buck 生成每个平台的 `build-metadata.sh`，六个组件 action 分别调用选定 platform 的上游 bootstrap action body，`package` 和 `complete` 分别声明依赖包与最终发行目录，两个 `sh_test` 验证中间包和最终用户包。workflow 不再调用组包脚本，所有 `uname`/target wrapper 和重复 versions 文件均已删除。

## A · 构建引导

没有 buck2 之前不能用 buck2 装 buck2。

| 文件 | 为何存在 | Native 替代 |
| --- | --- | --- |
| `src/buck2` + `build/tools/buck2-bin` | 薄启动器自动确保共享 REAPI cache 存活；DotSlash 清单按平台拉钉死的 buck2 | 留作构建引导；不承担 target 分发或组包 |
| `build/tools/bazel-remote-bin` + `buck2-cache` | 钉定单二进制 REAPI cache，并以 loopback 服务让多 worktree 共用 Buck action results | 标准 Buck cache 边界；不是自制 fingerprint/Cargo cache |
| `src/build/tools/install-dotslash` + `dotslash.env` | 开发机还没有 DotSlash 时按 SHA 装一份 | 留作一次性引导 |
| `src/build/tools/reindeer` + `reindeer.env` | 下载钉死 Reindeer，从 Cargo.lock 生成 `third-party/rust/BUCK` | 生成物已进 Git。日常 `buck2 build` 不跑它；更新 lock 时用人跑 |
| `src/build/toolchains/rust/merge_sysroot.sh` | 拼 rustc 归档 + std 归档 | **已是 Buck genrule 动作**，范本 |

## B · `uname` 分发器

shell 只能看本机 `uname`，所以每个 target 复制一份 13–17 行入口。

| 文件 |
| --- |
| `packaging/dependencies/bootstrap.sh`、`build-package.sh`、旧 `test-package.sh` 分发器 |
| `bootstrap-<target>.sh`、`build-package-<target>.sh`、`test-package-<target>.sh`（各三平台） |
| `versions.sh`、`lib.sh` |

第一方发行、外部准备、组包和测试统一使用 `--target-platforms root//build/platforms:…`。本类已全部删除；同名的现 `test-package.sh` 是单一 `sh_test` 测试体，不再做宿主或 target 分发。

## C · 锁、下载、组包（shell 当 Make）

体积最大。哈希纪律已有，缺的是「没声明的工具链不能参加」。

| 文件 | 为何存在 |
| --- | --- |
| `common/versions.sh`、`targets/*.sh`、`platforms/macos/versions.sh` | 版本 / URL / SHA 写成 bash `readonly` |
| `common/build.sh` | `curl` + 校验 + `~/.cache` 手搓 |
| `platforms/*/bootstrap.sh` | 拿官方包；Darwin 上仍用上游 `cargo --locked` 编 Tuwunel |
| `platforms/*/package.sh`、`common/package.sh` | 组 tar、许可证、manifest |
| `common/test-package.sh` | 离线安装再启停 smoke |

**第一步已完成：**

- 官方 blob（Vikunja、Dagu、Linux Tuwunel、官方 tmux、Cinny、Static Web Server）已由 Starlark `http_file` / `http_archive` 以 URL + SHA 声明；
- Darwin Tuwunel 由 Buck 声明 Rust 组件和输入，再用粗粒度 action 调上游 Cargo。

**本轮收口：**

- 组 payload、源码伴随包与 SHA 清单 → Buck 声明输出；
- 生命周期测试 → `buck2 test` 调现有 smoke，测试体仍可用 shell；
- 删除非 Buck 兼容路径中的自制下载/cache 机制，不维护第二套输入身份。

**不该变成** Tuwunel 的 `rust_library` 图。Darwin 现场 cargo 最多外套一层 `genrule`，内部 crate 不进 HCTL2 图——替换方案备忘已立法。

## D · 装到用户机器上的启停

用户没有 Buck2，产品还没有 `hctl2 start`。

| 文件 | 为何存在 |
| --- | --- |
| `hctl2-services` | 安装后总入口 |
| `start.sh` / `stop.sh` / `status.sh` / `smoke.sh` | loopback 服务生命周期 |
| `common/runtime.sh`、`platforms/*/runtime.sh` | 端口、PID、socket |
| `install-package.sh`、`packaging/release/install.sh` | 离线安装器 |

Buck2 只 `export_file` 打进包。消灭这些是 P2 公共 CLI，不是 Starlark。

## E · 发行组装

| 文件 | 现状 |
| --- | --- |
| `export-first-party.sh` | 已被 `packaging/release/defs.bzl` 的 `genrule` 调用 |
| `assemble.sh`（~342 行） | `complete` genrule 的 action body，校验 sidecar 并拼完整发行目录 |
| `release/test-package.sh` | `complete-test` 的测试体，验证最终包与服务生命周期 |

`assemble.sh` 的输入已改为 `:first-party` + `dependencies:package`，输出由 Buck 声明。人和 workflow 不再先 `build-package.sh` 再 `assemble.sh`。

## F · 与产品构建无关

`run`：本设计仓库开 Claude / Codex / Grok 等 session。不进 `src/` 的 Buck 图。

## 落地顺序

1. ~~官方 blob → `http_file`；Darwin Tuwunel 外套粗粒度 action。~~ 已完成。
2. ~~组 payload 与源码伴随包 → Buck 输出，消除 workflow 对 `build-package-<target>.sh` 的调用。~~ 已完成。
3. ~~`assemble` → Buck 发行目标，workflow 只上传和发布声明输出。~~ 已完成。
4. ~~Buck 测试目标运行现有合同与生命周期测试体。~~ 已完成。
5. ~~删除 B 类分发器与不再被 action/测试/运行包引用的构建 helper。~~ 已完成。
6. D 类等 P2 CLI，不在本轮为追求零 shell 而重写。

## 拍板结果

1. 六类划分按“删分发器、构建编排进图、保留引导/上游原生入口/运行期代码”施工。
2. 所有官方 blob 已一次性进入 Buck 下载规则，不分批维护两套下载事实。
3. Darwin Tuwunel 保持上游 Rust 1.95.0 的粗粒度 Cargo action；Linux 官方包是摘要锁定 blob，不声称两者来自同一构建。

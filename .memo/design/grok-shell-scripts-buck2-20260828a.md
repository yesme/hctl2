# 仓库 shell 脚本盘点：为何存在、哪些该进 Buck2

> 状态：待拍板<br>
> 基线：main @ ad90c01<br>
> 去向：`src/packaging/` Buck 目标与删除 uname 分发器；不进合同层。运行期脚本等 P2 CLI 另议。<br>
> 日期：2026-08-28<br>
> 说明：Grok 按职责自分类的施工备忘，不改规范。边界与 [Buck2 替换方案](./codex-buck2-replacement-20260827.md) 一致：不进入上游内部构建图。确定性诉求见 [构建确定性备忘](./grok-20260827a.md)。<br>
> 范围：仓库内带 shebang 的脚本与 DotSlash 入口；不含 GitHub Actions YAML 里的 inline shell。

## 怎么来的

第一方已进 Buck2 图。仓库里仍有约 3300 行 shell。所有者问：每份为什么存在，能不能用 Buck2 native 换掉。本文按**职责**分成六类，不按目录堆文件名。

## 总判

| 类 | 体积（约） | Buck2 native？ |
| --- | --- | --- |
| A 构建引导 | 160 行 | 鸡生蛋，留薄引导；`merge_sysroot.sh` 已是 genrule |
| B `uname` 分发器 | 220 行 | **该删**，换成 `--target-platforms` / `select` |
| C 锁、下载、组包（shell 当 Make） | 2000 行 | **该进图**：`http_file` / `http_archive` + genrule；上游 cargo 不拆 |
| D 装到用户机器上的启停 | 800 行 | **不该当构建问题**；`export_file` 打进包，消灭要等 P2 CLI |
| E 发行组装 | 600 行 | 半截已是 genrule；`assemble.sh` 是下一刀 |
| F 与产品构建无关 | `run` 222 行 | 不进 `src/` 图 |

该用 Buck2 换掉的是「shell 当 Make」。不该换的是用户机器上的启停，以及 Tuwunel/Vikunja 自己怎么编。

## A · 构建引导

没有 buck2 之前不能用 buck2 装 buck2。

| 文件 | 为何存在 | Native 替代 |
| --- | --- | --- |
| `src/buck2` | DotSlash 清单，按平台拉钉死的 buck2 | **已经是 native**（`#!/usr/bin/env dotslash`，不是 shell） |
| `src/build/tools/install-dotslash` + `dotslash.env` | 开发机还没有 DotSlash 时按 SHA 装一份 | 留作一次性引导 |
| `src/build/tools/reindeer` + `reindeer.env` | 下载钉死 Reindeer，从 Cargo.lock 生成 `third-party/rust/BUCK` | 生成物已进 Git。日常 `buck2 build` 不跑它；更新 lock 时用人跑 |
| `src/build/toolchains/rust/merge_sysroot.sh` | 拼 rustc 归档 + std 归档 | **已是 Buck genrule 动作**，范本 |

## B · `uname` 分发器

shell 只能看本机 `uname`，所以每个 target 复制一份 13–17 行入口。

| 文件 |
| --- |
| `packaging/dependencies/bootstrap.sh`、`build-package.sh`、`test-package.sh` |
| `bootstrap-<target>.sh`、`build-package-<target>.sh`、`test-package-<target>.sh`（各三平台） |
| `versions.sh`、`lib.sh` |

第一方发行已经用 `--target-platforms root//build/platforms:…`（见 `packaging/release:first-party`）。外部组包一旦变成 Buck 目标，本类全部可删。

## C · 锁、下载、组包（shell 当 Make）

体积最大。哈希纪律已有，缺的是「没声明的工具链不能参加」。

| 文件 | 为何存在 |
| --- | --- |
| `common/versions.sh`、`targets/*.sh`、`platforms/macos/versions.sh` | 版本 / URL / SHA 写成 bash `readonly` |
| `common/build.sh` | `curl` + 校验 + `~/.cache` 手搓 |
| `platforms/*/bootstrap.sh` | 拿官方包；Darwin 上仍用上游 `cargo --locked` 编 Tuwunel |
| `platforms/*/package.sh`、`common/package.sh` | 组 tar、许可证、manifest |
| `common/test-package.sh` | 离线安装再启停 smoke |

**该变成：**

- 官方 blob（Vikunja、Dagu、Linux Tuwunel、官方 tmux、Cinny、Static Web Server）→ Starlark 里的 `http_file` / `http_archive`（URL + SHA）
- 组 payload、写 SHA 清单 → `genrule`
- 生命周期测试 → `buck2 test` 调现有 smoke（测试体仍是 shell）

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
| `assemble.sh`（~338 行） | 校验 sidecar、拼完整发行 tar，仍手跑 |
| `release/test-package.sh` | 测完整包 |

下一刀：`assemble.sh` 的输入改为 `:first-party` + C 类外部归档，输出带 SHA 的发行 tar。人不再先 `build-package.sh` 再 `assemble.sh`。

## F · 与产品构建无关

`run`：本设计仓库开 Claude / Codex / Grok 等 session。不进 `src/` 的 Buck 图。

## 建议落地顺序

1. 官方 blob → `http_file`，删 `download_verified` 和本机手搓 cache。
2. 组包 + `assemble` → Buck 目标，`select` 换掉 B 类分发器。
3. `sh_test` 跑 D 类 smoke；安装器仍打进包。
4. Darwin Tuwunel 继续上游 cargo，外套 `genrule`。
5. D 类等 P2 CLI。

## 待拍板

1. 六类划分与上表「该删 / 该进图 / 留引导 / 留运行期」是否按此施工。
2. C 类官方 blob 是否第一批就全部 `http_file`，还是先 Tuwunel/tmux 以外、已有官方包的那几个。
3. Darwin Tuwunel 的 `genrule` 是否与「跨平台同一 rustc 版本」（确定性备忘第 3 条）绑在同一批：Linux 若仍吃官方 deb，则不是「我们编的两个 target」。

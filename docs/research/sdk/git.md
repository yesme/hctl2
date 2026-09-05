# Git 现场引擎：宿主 git 二进制、libgit2 与 gitoxide

> 状态：已拍板（所有者 2026-09-04 同意依赖 git CLI）· 日期：2026-09-04<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-SDK-GIT<br>
> 对象：宿主 git（本机 `2.50.1 (Apple Git-155)`）· [git2 `0.21.0`](https://crates.io/crates/git2)（2026-05-18，捆绑 libgit2 1.9 系源码；libgit2 最新 [v1.9.7](https://github.com/libgit2/libgit2/releases/tag/v1.9.7) 2026-08-13）· [gix `0.87.1`](https://crates.io/crates/gix)（2026-08-24，MSRV 1.85）· 随包对照 [dugite-native v2.53.0-4](https://github.com/desktop/dugite-native/releases/tag/v2.53.0-4)（2026-08-11）<br>
> 许可证：git GPL-2.0-only（子进程调用，不链接）；libgit2 GPL-2.0 带链接例外；git2 与 gix MIT OR Apache-2.0；dugite-native 的构建脚本 MIT，产物里的 git 仍是 GPL-2.0

## 定位

`hctl2-tool` 的五项现场职责（[决策史 §33](../../design/references/decision-history.md#33-hctl2-tool-定界为现场执行者v0153)）全部落在 Git 上：物化与隔离 worktree、封存保全、现场校验与回读、本地集成、不可变正文与判决结晶副本写入。问题只有一个：这些操作经宿主机上的 `git` 二进制做，还是把 libgit2 或 gitoxide 链进工具箱进程里做。

判断的出发点不是性能，是**谁还在碰同一个现场**。工具箱物化出来的 worktree 是给 Harness 和人用的，Harness（Claude Code、Codex 等）与人在里面用的是宿主 git，CI 也是。工具箱如果带自己的 Git 实现，就是同一批文件上并排两个引擎。§33 把工具箱锚在 git 的 porcelain / plumbing 分层上——上层只编排底层，不重实现——这份文件把它落实到具体命令。

五项职责各用哪些 plumbing 命令：

| 职责 | 命令 |
| --- | --- |
| 仓库检查 | `rev-parse --path-format=absolute --git-common-dir`、`worktree list --porcelain -z`、`for-each-ref --format`、`config --get` |
| 物化与核验 | `worktree add`、`worktree remove`、`worktree prune`、`merge-base --is-ancestor`、`status --porcelain=v2 -z` |
| 封存 | `GIT_INDEX_FILE=<临时索引>` 下 `add -A` + `write-tree`，`commit-tree`，`update-ref` 挂到工具箱自己的 ref 命名空间 |
| 本地集成 | `merge-tree --write-tree`（无工作树合并）、`commit-tree`、`update-ref <ref> <new> <old>`（比较并交换） |
| 正文写入与回读（B1 前） | `hash-object -w`、`update-index`、`write-tree`、`commit-tree`、`cat-file` |

## 上游能力

### 宿主 git

各目标平台自带的版本（2026-09-04 查）：

| 平台 | 版本 | 来源 |
| --- | --- | --- |
| Ubuntu 22.04 LTS | 2.34.1 | packages.ubuntu.com jammy |
| Ubuntu 24.04 LTS | 2.43.0 | packages.ubuntu.com noble |
| Ubuntu 26.04 LTS | 2.53.0 | packages.ubuntu.com resolute |
| Debian 12 | 2.39.5 | packages.debian.org bookworm |
| Debian 13 | 2.47.3 | packages.debian.org trixie |
| macOS，Xcode 15.6 / 16 的命令行工具 | 2.39.5（Apple Git-154） | Apple 开发者论坛与安装指南 |
| macOS 26，Xcode 26 的命令行工具 | 2.50.1（Apple Git-155） | 本机 `git --version` |
| GitHub `ubuntu-24.04` runner 镜像 20260823 | 2.55.0（另有 Git LFS 3.7.1） | actions/runner-images |
| GitHub `macos-15` arm64 runner 镜像 20260727 | 2.55.0 | actions/runner-images |

工具箱用到的命令各自出现的版本，按发布说明核对：

| 命令或开关 | 最低版本 | 发布说明原文要点 |
| --- | --- | --- |
| `worktree add`、`rev-parse --git-common-dir` | 2.5 | 链接工作树机制引入 |
| `status --porcelain=v2` | 2.11 | 机器可读的第二版状态格式 |
| `--no-optional-locks` / `GIT_OPTIONAL_LOCKS=0` | 2.15 | 「`git status` 这类只读操作会顺手更新索引，新开关可以禁掉」 |
| `worktree remove` | 2.17 | 删除链接工作树的正式子命令 |
| `rev-parse --path-format=(absolute\|relative)` | 2.31 | 「可以显式要求输出绝对或相对路径」 |
| `worktree list --porcelain -z` | 2.36 | 「porcelain 输出没有正确 c-quote 路径与锁原因，引入 NUL 结尾的 `-z`」 |
| `merge-tree --write-tree` | 2.38 | 「取两个提交，算出合并后的树，不需要工作树」 |
| `update-ref <ref> <new> <old>`、`commit-tree`、`write-tree`、`merge-base --is-ancestor` | 1.x / 1.8 | 早于所有目标平台 |

本机 2.50.1 实测（临时仓库）：`merge-tree --write-tree main side` 输出结果树 sha；`rev-parse --path-format=absolute --git-common-dir` 给绝对路径；`worktree list --porcelain -z` 以 NUL 分隔；`GIT_OPTIONAL_LOCKS=0 git status --porcelain=v2 --branch` 不写索引；`update-ref refs/heads/main <new> <old>` 旧值匹配时成功，旧值给全零时退出码 128、stderr `cannot lock ref … reference already exists`——这就是比较并交换失败的形态。

### libgit2 与 git2

- git2 `0.21.0`（2026-05-18），MIT OR Apache-2.0；`libgit2-sys` 捆绑 libgit2 源码由 `cc` crate 编译，网络传输（libssh2、OpenSSL）是可选 feature，工具箱不做远端操作可以全关。
- libgit2 `v1.9.7`（2026-08-13）是安全版本：修 libssh2 传输里远端路径的命令注入（CVE-2026-5917）。远端传输不在工具箱职责内，但说明这条依赖的补丁节奏要跟。
- 有：链接工作树 API、进程内合并（自家实现，不是 git 的 ort）、带预期旧值的引用创建（比较并交换）、内存索引。
- 没有：不跑钩子；过滤器要自己注册，LFS 不内建；SHA-256 仓库仍是实验特性。

### gitoxide 与 gix

- gix `0.87.1`（2026-08-24），MIT OR Apache-2.0，MSRV 1.85，2026 年 7 月到 8 月三个版本，0.x 期破坏性升级频繁。
- 仓库 `crate-status.md`（2026-09-04 读 main）：工作树「打开带 worktree 的仓库」已做，「创建、移动、删除、修复」未做；合并的 blob 三方合并已做，树合并只到「树 diff 启发式与 git 测试用例一致」和「生成带 stage 的索引」，作者自注「要重写成能证明正确的逻辑，现在状态太多没测全」，没有宣称与 ort 一致；`gix-status` 未跟踪文件已做；`gix-filter` / `gix-lfs` 大部分待做；`gix-hook` 的发现与执行都未做。

### 随包 portable git 的对照

GitHub Desktop 用的 dugite-native 每个 git 版本出一套可搬移的 git 目录：`v2.53.0-4`（2026-08-11）含 git 2.53.0、Git LFS 3.7.1 与 Git Credential Manager；体积 macOS arm64 62 MB、macOS x64 66 MB、Linux x64 65 MB、Linux arm64 23 MB。这是「宿主版本不够时随包一份」的现成来源，仍属采用二进制，不是换引擎。

## 候选比较

| 维度 | 宿主 git 二进制 | libgit2（git2） | gitoxide（gix） |
| --- | --- | --- | --- |
| 与 Harness、人、CI 同一引擎 | 是 | 否 | 否 |
| checkout 保真：LFS、过滤器、`.gitattributes`、sparse-checkout、`post-checkout` 钩子、conditional include | 全部按用户配置生效 | 过滤器要自注册，LFS 不内建，不跑钩子 | 过滤器部分，LFS 与钩子未做 |
| 合并语义 | `merge-tree --write-tree` 与 `git merge` 同一实现 | 自家合并实现，重命名检测与冲突边界有差异 | 树合并未完成 |
| 链接工作树增删 | 全 | 有 API | 未做 |
| 引用比较并交换 | `update-ref` 带旧值 | 带预期旧值的创建 | 引用事务带预期旧值 |
| 版本来源 | 宿主，需下限 | 随 crate 钉死 | 随 crate 钉死 |
| 构建供应链 | 无新增 | C 编译走 Buck 的 build-script shim，再来一轮 reindeer fixup（#165 刚为 SQLite 走过一遍） | 纯 Rust，几十个 `gix-*` crate |
| 错误形态 | 退出码加 stderr 文本；porcelain 格式由 git 承诺稳定 | 类型化 | 类型化 |
| 许可证 | GPL-2.0，子进程不链接 | GPL-2.0 带链接例外 | MIT OR Apache-2.0 |

## 边界与取舍

- **同一引擎是决定性理由。** 工具箱物化的树如果和人手工 `git worktree add` 出来的不一样（LFS 指针文件没被替换、过滤器没跑、钩子没触发），分歧最难查，而且直接落在 Harness 要工作的文件上。
- **合并结果要和人看到的一致。** 集成回读得到的结果树必须等于人在本地 `git merge` 会得到的树，Integration Receipt 才说得清。`merge-tree --write-tree` 就是 `git merge` 的引擎；libgit2 是另一套实现，gitoxide 的树合并还没完成。
- **plumbing 输出就是稳定接口。** `status --porcelain=v2`、`worktree list --porcelain -z`、`for-each-ref --format`、`update-ref` 的比较并交换，都是 git 承诺不变的格式。业界同类工具都走这条路：GitHub Desktop（dugite 包一份 git 再转调）、VS Code 的 git 扩展、gh、lazygit、Gitea（默认 git 命令行，go-git 只在实验性 build tag 下）。jj 自己内置 gitoxide，仍在 0.27（2025-03-05）把 fetch / push 默认改为转调 git 子进程，0.30（2025-06-04）把 libgit2 路径整个删除。
- **内置库的四个好处在我们这里不构成代价。** 类型化错误、不解析文本：工具箱一次命令只调几次 git，解析 porcelain 几十行。不依赖宿主版本：正解是版本下限加必要时随包 portable git，仍是采用二进制。不跑钩子、不读会执行命令的配置项（`core.fsmonitor`、`core.hooksPath`）：这是单用户本机工具，仓库本地配置是用户自己的，Harness 反正在同一仓库跑 git；工具箱自己的调用能用 `-c` 关的就关。性能：无关。
- **版本下限定 2.39。** 唯一高要求是 `merge-tree --write-tree` 的 2.38；定 2.39 让 Debian 12 与 Xcode 15.6 起的命令行工具都在线内。Ubuntu 22.04 的 2.34 不满足，用户走 git-core PPA 或 Homebrew；22.04 标准支持到 2027-04。工具箱启动时读 `git --version`，低于下限拒绝并给稳定错误码，每条记录带 git 路径与版本。
- **调用卫生**（工具箱自己的每次调用）：启动时解析一次 git 绝对路径（`HCTL2_GIT` 可覆盖，与 `HCTL2_GH` 同款），之后不再查 `PATH`；清掉继承环境里的 `GIT_DIR`、`GIT_COMMON_DIR`、`GIT_WORK_TREE`、`GIT_INDEX_FILE`、`GIT_OBJECT_DIRECTORY`、`GIT_ALTERNATE_OBJECT_DIRECTORIES`、`GIT_NAMESPACE`与配置注入变量 `GIT_CONFIG_COUNT`、`GIT_CONFIG_KEY_*`、`GIT_CONFIG_VALUE_*`、`GIT_CONFIG_PARAMETERS`，保留选择配置文件的 `GIT_CONFIG_GLOBAL`、`GIT_CONFIG_SYSTEM`、`GIT_CONFIG_NOSYSTEM`（#174 Fable 复核修正）；需要的显式设置（封存用临时 `GIT_INDEX_FILE`）；仓库与工作树用 `-C <path>` 显式给；`GIT_TERMINAL_PROMPT=0`；在 Harness 正在用的 worktree 里读状态加 `GIT_OPTIONAL_LOCKS=0`，不碰它的索引；`LC_ALL=C` 让 stderr 可匹配；只用 `-z` 与 porcelain 格式。用户配置照常生效——这正是选宿主 git 的理由；`worktree add` 触发 `post-checkout`、`update-ref` 触发 `reference-transaction`，与人手工操作一致，不特意关；工具箱用 `commit-tree` 生成封存与合并提交，不经 `pre-commit`，Harness 自己的提交照常经过。
- **什么情况会翻转**：要在没装 git 的机器上工作；要每秒成千上万次对象级操作；要自己的对象模型（jj、GitButler 那种自己就是 VCS 的情形）。三条都不是 HCTL 的情形——装了 Coding Harness 的机器必然有 git。

## 决定

- **采用二进制**：宿主 git，下限 2.39，不随包。`hctl2-tool` 启动读版本，低于下限拒绝；`doctor` 出现后报版本与路径。
- 不采用 SDK：git2 / libgit2 与 gix / gitoxide 都不进依赖树。
- 保留选项：宿主下限把主要平台挡在外面时，按 dugite-native 的做法随包一份 portable git，走 DotSlash 钉摘要，仍是采用二进制。
- 开工前核对（甲 PR）：三平台 CI 的 git 版本记进测试矩阵（当前 runner 都是 2.55.0，远高于下限）；下限回归可在 CI 用 Debian 12 容器（2.39.5）跑一次工具箱测试，由丁 PR 决定要不要加。

## 证据

- 宿主 git 版本：[packages.ubuntu.com git](https://packages.ubuntu.com/search?keywords=git&searchon=names&exact=1&suite=all&section=all)（jammy 2.34.1、noble 2.43.0、resolute 2.53.0）· [packages.debian.org git](https://packages.debian.org/search?keywords=git&searchon=names&exact=1&suite=all&section=all)（bookworm 2.39.5、trixie 2.47.3）· [Apple 论坛：命令行工具的 git 版本](https://developer.apple.com/forums/thread/795617)与 [mac.install.guide](https://mac.install.guide/git/)（Xcode 15.6 为 2.39.5 Apple Git-154）· [runner-images ubuntu-24.04](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md) · [runner-images macos-15 arm64](https://github.com/actions/runner-images/blob/main/images/macos/macos-15-arm64-Readme.md)
- git 发布说明：[2.38](https://github.com/git/git/blob/master/Documentation/RelNotes/2.38.0.adoc)（`merge-tree --write-tree`）· [2.36](https://github.com/git/git/blob/master/Documentation/RelNotes/2.36.0.adoc)（`worktree list -z`）· [2.31](https://github.com/git/git/blob/master/Documentation/RelNotes/2.31.0.adoc)（`--path-format`）· [2.15](https://github.com/git/git/blob/master/Documentation/RelNotes/2.15.0.adoc)（`--no-optional-locks`）
- git 手册：[git-worktree](https://git-scm.com/docs/git-worktree) · [git-merge-tree](https://git-scm.com/docs/git-merge-tree) · [git-update-ref](https://git-scm.com/docs/git-update-ref) · [git-status（porcelain v2）](https://git-scm.com/docs/git-status#_porcelain_format_version_2) · [git（环境变量）](https://git-scm.com/docs/git#_environment_variables)
- libgit2 / git2：[git2 crates.io](https://crates.io/crates/git2)（0.21.0，2026-05-18）· [libgit2 v1.9.7](https://github.com/libgit2/libgit2/releases/tag/v1.9.7)（2026-08-13，CVE-2026-5917）· [libgit2 README（许可证：GPL-2.0 带链接例外）](https://github.com/libgit2/libgit2#license)
- gitoxide：[gix crates.io](https://crates.io/crates/gix)（0.87.1，2026-08-24，MSRV 1.85）· [crate-status.md](https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md)（工作树创建、树合并、过滤器、钩子的完成状态）
- 业界做法：[dugite-native v2.53.0-4](https://github.com/desktop/dugite-native/releases/tag/v2.53.0-4) · [jj CHANGELOG](https://github.com/jj-vcs/jj/blob/main/CHANGELOG.md)（0.27.0 子进程默认、0.30.0 删 libgit2）· [Gitea 后端指南](https://docs.gitea.com/contributing/guidelines-backend)（默认 git 命令行，`gogit` 为实验 build tag）
- 本仓库：[决策史 §33](../../design/references/decision-history.md#33-hctl2-tool-定界为现场执行者v0153) · [Participant 约束 §ChangeSet 与 Git 事实](../../design/spec/participant.md#changeset-与-git-事实) · [系统边界 §单写者](../../design/spec/system.md#单写者) · [P1 收口计划](../../../.memo/design/p1-toolbox-20260904/01-plan.md) · [gh 二进制的同款做法](./github.md)

## 复核记录

- 2026-09-05 乙实现复核：封存按本文「五项职责」表执行——在 `<git-common-dir>/hctl2/` 下放临时 `GIT_INDEX_FILE`（现场锁串行化，路径按 ChangeSet 引用固定，重跑先删残留），`add --all` 不加 `-f`，再 `write-tree`、`commit-tree`、`update-ref` 挂到 `refs/hctl2/changesets/<ref>/trees/<tree_sha>`。同一树 sha 复用已有快照提交。被忽略文件不进树，拆除时用 `ls-files -o -i --exclude-standard` 列残留（路径与大小，封顶 64）。拆除走 `worktree remove --force -- <path>`，不调用 `worktree prune --expire now`，避免误清其他陈旧 worktree。`commit-tree` 使用工具箱自己的 `GIT_AUTHOR_*` / `GIT_COMMITTER_*`，保全不依赖 `user.name`。无新依赖，仍是宿主 git、下限 2.39。

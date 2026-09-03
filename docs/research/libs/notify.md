# notify · 跨平台文件系统监听

> 状态：调研 · 日期：2026-09-03<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-LIB-NOTIFY<br>
> 对象：[`notify` 8.2.0](https://crates.io/crates/notify/8.2.0)（2025-08-03 发布）；配套 [`notify-debouncer-full` 0.7.0](https://crates.io/crates/notify-debouncer-full/0.7.0)（2026-01-23）、[`notify-debouncer-mini` 0.7.0](https://crates.io/crates/notify-debouncer-mini/0.7.0)（2025-08-03）；`notify` 9.0.0-rc.5（2026-08-30）仍是预发布，不钉<br>
> 许可证：notify CC0-1.0；notify-types、notify-debouncer-mini、notify-debouncer-full、file-id 均为 MIT OR Apache-2.0

## 定位

我们用它做 hctl2-tool `wait` 命令里「等某路径出现且摘要匹配」这一种事实的**唤醒源**：不是靶子出现了就算，而是靶子出现且 SHA-256 等于预期才算。事件只是提醒「该再查一次了」，事实由 `wait` 自己 stat + 哈希确认。它替代的手写代码是三平台各一套 inotify / FSEvents / ReadDirectoryChangesW 绑定，以及不得不写的轮询循环。`wait` 要等的另一类事实——CI 状态、PR 是否合并——不在文件系统里，靠 GitHub API 而不是它（见下）。

## 上游能力

- **后端**：Linux/Android inotify；macOS FSEvents（默认 feature `macos_fsevent`）或 kqueue（feature `macos_kqueue`）；Windows ReadDirectoryChangesW；BSD 家族 kqueue；所有平台 `PollWatcher`。`recommended_watcher()` 按平台选实现。
- **API**：`Watcher::watch(path, RecursiveMode)`；事件 `Event { kind, paths, attrs }`，`attrs` 里有 `Flag::Rescan` 表示「后端丢了事件，请全量复查」；错误 `ErrorKind::MaxFilesWatch` 表示 watch 数量超限。`Config::with_poll_interval`（PollWatcher，默认 30 秒）、`with_compare_contents`（对文件内容哈希比对，供 `/proc` `/sys` 这类不更新 mtime 的伪文件系统用，代价是每轮读全部文件）、`with_follow_symlinks`。
- **debouncer**：`notify-debouncer-full` 把 Rename 的 From/To 配对成一个事件、用 file-id 跟踪 inode/文件 ID 以在 FSEvents 与 Windows 上缝合改名、去重 Create/Modify、目录删除只发一个 Remove；`notify-debouncer-mini` 只做最简合并。
- **规模与维护**：notify 下载 1.48 亿，近 90 天 3,625 万；仓库 3,446 star，最后提交 2026-09-02；rust-analyzer、zed、deno、cargo-watch、watchexec 都在用。8.2.0 MSRV 1.77；9.0 线 MSRV 1.88，自 2026-01 起出了五个 rc。
- **依赖树**（`cargo tree`，rustc 1.98.0）：notify 8.2.0 Linux 10 个 crate、macOS 8、Windows 13；加 debouncer-full 后 12 / 10 / 15；debouncer-mini 反而更多（18 / 16 / 19，默认拉 crossbeam-channel）。

## 候选比较

| 候选 | 版本 / 发布 | 许可证 | 覆盖 | 依赖树 | 备注 |
| --- | --- | --- | --- | --- | --- |
| notify | 8.2.0 / 2025-08-03 | CC0-1.0 | 三平台 + Windows + 轮询 | 8–13 | 生态默认选择 |
| notify | 9.0.0-rc.5 / 2026-08-30 | CC0-1.0 | 同上 | — | 预发布；有 v8→v9 升级指南 |
| notify + debouncer-full | 0.7.0 / 2026-01-23 | MIT OR Apache-2.0 | 同上 | 10–15 | 改名缝合、去重 |
| 只用 `PollWatcher` | notify 内含 | — | 任何文件系统 | 同 notify | 30 秒默认间隔可调；大树昂贵 |
| watchexec | 二进制 | Apache-2.0 | 三平台 | 外部进程 | 面向「文件变了就跑命令」，超出 `wait` 需要 |
| 直接绑定 inotify / objc2 FSEvents / windows-sys | — | — | 各一套 | 各自 | 等于重写 notify 的后端层 |
| 纯轮询自写 | — | — | 任何 | 0 | 延迟与 CPU 取舍全靠自己 |

## 边界与取舍

各平台的丢事件与轮询回退（依据 notify 8.2.0 文档「Known Problems」与源码）：

| 平台 | 后端 | 会怎么丢 | notify 8.2.0 的表现 | 回退 |
| --- | --- | --- | --- | --- |
| Linux | inotify | 事件队列溢出 `IN_Q_OVERFLOW`；watch 数超过 `fs.inotify.max_user_watches`（`ENOSPC`） | 溢出 → 发一个 `EventKind::Other` 带 `Flag::Rescan`；`ENOSPC` → `ErrorKind::MaxFilesWatch` | 收到 Rescan 全量复查；PollWatcher 不受 watch 上限约束 |
| macOS | FSEvents | 内核或用户态丢弃（`KernelDropped` / `UserDropped`）、`MustScanSubDirs`；事件按目录合并、带延迟；不属于本用户的文件受安全模型限制 | 三种标志都映射为 Rescan 事件并在 `info` 里注明原因 | 同上；kqueue 每文件一个 fd，不适合大树 |
| Windows | ReadDirectoryChangesW | 固定缓冲区溢出（`ERROR_NOTIFY_ENUM_DIR`） | 8.2.0 源码里未见对该错误码的显式处理（grep 未命中，实际表现未核实） | PollWatcher |
| 网络与伪文件系统 | NFS、SMB、WSL 看 Windows 路径、Docker on Apple Silicon、`/proc` `/sys` | 根本不发事件，或后端不可用 | 文档明示改用 PollWatcher，伪文件系统加 `compare_contents` | 轮询 |

通用坑：编辑器保存方式不一（截断重写或新建后替换），事件模式不可预期；监听单个文件要监听其父目录，要收到目录删除事件要监听祖父目录；大目录树上「linux 后端本身就不保证 100% 可靠」。

`wait` 的正确形状，因此不是「订阅事件」而是「先查后等、事件当提醒、定时兜底」：

1. 启动先查一次：路径存在且摘要匹配，立即返回，根本不建 watcher。
2. 否则在**最近的已存在祖先目录**上建非递归 watcher（靶子路径可能尚不存在）；任何事件、任何 Rescan、任何后端错误都只做一件事——再查一次。
3. 不管后端是不是原生，都保留一个定时兜底复查（几秒级，可配置），代价只是一次 stat 加一次可能的哈希。
4. 摘要不匹配只说明「还没写完或不是这份」，继续等；写方约定先写临时文件再改名，读方就不会读到半截。
5. 超时与取消要给出有类型的结果：匹配 / 超时 / 后端不可用已退回轮询。
6. 检测到靶子在网络文件系统上时直接用 PollWatcher，并在结果里注明。
7. debouncer-full 能压住事件风暴，但引入 timeout 延迟；单靶子场景用原生 notify 加自己的复查节流即可，需要跟踪改名时再上 debouncer。

CI 状态与 PR 合并这类事实不靠文件监听，靠 GitHub REST：

- 检查状态：`GET /repos/{owner}/{repo}/commits/{ref}/check-runs`，每个 check run 有 `status`（`queued / in_progress / completed`）与 `conclusion`（`success / failure / neutral / cancelled / skipped / timed_out / action_required / stale`），`filter=latest` 只看最新一轮；只覆盖 Checks API 产生的 check run，旧式 commit status 走 combined status 端点；fork 分支上的推送不被检测。
- 合并事实：`GET /repos/{owner}/{repo}/pulls/{n}` 的 `merged`（布尔）、`merged_at`、`merge_commit_sha`、`state`；`mergeable` 为 `null` 表示 GitHub 还在后台算，稍后重试。
- 轮询礼仪：用 ETag / `If-None-Match` 做条件请求，命中 `304` 不计入主速率限制；遵守 `x-poll-interval`、`retry-after`、`x-ratelimit-remaining/reset`；请求串行，避开次级限制。额度：个人令牌 5,000 次/小时，Actions 的 `GITHUB_TOKEN` 每仓库 1,000 次/小时，未认证 60 次/小时。本机工具收不到 webhook，轮询是唯一形态。
- 实现：SDK 用 [octocrab 0.54.1](https://crates.io/crates/octocrab/0.54.1)（2026-07-24，MIT OR Apache-2.0，部件矩阵已列）；二进制用 `gh pr checks --watch --interval N --fail-fast --json …`（未完成时退出码 8）。两者都是同一组 REST 端点的封装。

许可证提示：notify 本体是 CC0-1.0 公有领域奉献，被 Apache-2.0 项目使用无障碍；CC0 不含专利授权条款，这里只是记录，不构成风险判断。

## 决定建议

**采用 SDK**：notify 8.2.0（钉 8.x，9.0 稳定后再升），debouncer-full 0.7.0 按需；`wait` 的文件事实一律「先查、事件当提醒、定时兜底轮询」，Rescan 与后端错误统一退化为复查，网络文件系统自动走 PollWatcher。CI 与 PR 合并事实走 GitHub REST 加 ETag 条件请求——control 内用 octocrab SDK，hctl2-tool 现场若已有 `gh` 也可直接用其二进制；两者不靠文件监听。不自写平台绑定，不用 watchexec。

## 证据

- notify：[crates.io 8.2.0](https://crates.io/crates/notify/8.2.0)、[docs.rs 8.2.0（平台、Known Problems、Config）](https://docs.rs/notify/8.2.0/notify/)、[仓库 README（平台矩阵、MSRV 政策、许可）](https://github.com/notify-rs/notify)、[8.2.0 inotify 溢出处理源码](https://github.com/notify-rs/notify/blob/notify-8.2.0/notify/src/inotify.rs)、[8.2.0 FSEvents 丢弃标志源码](https://github.com/notify-rs/notify/blob/notify-8.2.0/notify/src/fsevent.rs)、[8.2.0 Config 源码](https://github.com/notify-rs/notify/blob/notify-8.2.0/notify/src/config.rs)、[debouncer-full README](https://github.com/notify-rs/notify/blob/notify-8.2.0/notify-debouncer-full/README.md)、[v8→v9 升级指南](https://github.com/notify-rs/notify/blob/main/docs/UPGRADING_V8_TO_V9.md)
- debouncer：[notify-debouncer-full 0.7.0](https://crates.io/crates/notify-debouncer-full/0.7.0)、[notify-debouncer-mini 0.7.0](https://crates.io/crates/notify-debouncer-mini/0.7.0)、[file-id 0.2.3](https://crates.io/crates/file-id/0.2.3)
- GitHub REST：[List check runs for a Git reference](https://docs.github.com/en/rest/checks/runs?apiVersion=2022-11-28#list-check-runs-for-a-git-reference)、[Get a pull request](https://docs.github.com/en/rest/pulls/pulls?apiVersion=2022-11-28#get-a-pull-request)、[Best practices（条件请求、轮询、次级限制）](https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api?apiVersion=2022-11-28)、[Rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api?apiVersion=2022-11-28)、[gh pr checks](https://cli.github.com/manual/gh_pr_checks)、[octocrab 0.54.1](https://crates.io/crates/octocrab/0.54.1)
- 本库：[部件矩阵（hctl2-tool 五项与 provider 客户端一行）](../component-matrix-20260902.md)；R3 剩余简报（`.memo/design/design-review-20260902/30-r3-remaining-brief.md`）列出 wait 命令所需库

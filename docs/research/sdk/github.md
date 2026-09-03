# GitHub 客户端层：octocrab 与 gh 二进制

> 状态：调研 · 日期：2026-09-03<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-SDK-GITHUB<br>
> 对象：GitHub REST / GraphQL API（github.com）· [octocrab `0.54.1`](https://crates.io/crates/octocrab)（2026-07-24）· [gh CLI `v2.99.0 / d528f20f`](https://github.com/cli/cli/releases/tag/v2.99.0)（2026-09-01）<br>
> 许可证：octocrab MIT OR Apache-2.0；gh MIT；GitHub API 的使用受 GitHub 服务条款约束（不是开源许可）

## 定位

远端 SCM 与 CI 的**事实来源**，不是 HCTL 的 Task 模型。两个调用方，形态不同：

| 调用方 | 形态 | 要读的事实 | 要写的东西 |
| --- | --- | --- | --- |
| `hctl2-control` | 常驻服务；以 GitHub App 安装身份工作；接 webhook | PR 是否合并、某提交的 checks、引用推进 | 远端 push / PR / merge 属"外部副作用命令"，走适配器（[Participant 约束](../../design/spec/participant.md)） |
| `hctl-tool wait` | 开发机上的一次性命令（工具箱） | 同上三类，作为 `toolbox_readback` 证据通道（[Participant 约束 §证据通道](../../design/spec/participant.md)） | 不写 |

对应端点（REST，API 版本 `2022-11-28`）：

- PR 合并：`GET /repos/{o}/{r}/pulls/{n}` 的 `merged` / `merged_at` / `merge_commit_sha`；`GET …/pulls/{n}/merge` 回 204 / 404。
- 提交 checks：`GET /repos/{o}/{r}/commits/{ref}/check-runs`（Checks API）、`…/check-suites`；旧式 status 走 `GET …/commits/{ref}/status`（combined）。
- 引用推进：`GET /repos/{o}/{r}/git/ref/heads/{branch}` 的 `object.sha`；必要时 `compare`。

## 上游能力

**官方 SDK：Rust 没有。** GitHub 官方维护的客户端是 Octokit 系列（JS/TS、.NET、Ruby 等）和 `gh` CLI；crates.io 上没有 GitHub 官方发布的 Rust crate（官方"第三方库清单"页面本次未能抓到文本，未逐项核对）。**官方客户端只有一个是我们能直接用的：`gh` 二进制。**

**事实标准 Rust 库：octocrab。**

- `0.54.1`（2026-07-24），MIT OR Apache-2.0，`rust-version = 1.85.0`，edition 2018；2026 年 6 月起几乎每月一个版本（0.52.0 → 0.54.1 共五个）；1,437 星、105 个开放 issue；维护者 XAMPPRocky。
- 源码里看到覆盖 HCTL 需要的面：`checks()`（按 ref 列 check runs / check suites，含 `check_name` / `status` 过滤）、`commits().associated_check_runs()`、`repos().get_ref()` / `combined_status_for_ref()` / `list_statuses()`、`pulls().get()` / `is_merged()`、`ratelimit()`、`graphql()`；鉴权有 `personal_token`、`app(app_id, key)` + `installation(id)`（GitHub App JWT → 安装令牌）、`user_access_token`、`oauth`；`etag` 模块提供 `Etagged<T>` 与 `EntityTag`（条件请求）；`models::webhook_events` 有类型化的 webhook payload；默认 feature 带 rustls、retry、timeout、follow-redirect。
- 手写维护，不是从 OpenAPI 生成——字段落后于 GitHub 时靴子落在"未知字段被 serde 忽略"上。

**接口描述：有，但没必要用。** GitHub 在 [`github/rest-api-description`](https://github.com/github/rest-api-description) 发布完整 OpenAPI（3.0 与 3.1 两套，几十 MB），GraphQL schema 也公开。Oxide 曾从它生成过 [`octorust`](https://crates.io/crates/octorust)（最新 `0.10.0`，2025-03-12，MIT），此后 18 个月没有发布——从整份 GitHub OpenAPI 生成的客户端体量大、维护冷，不如手写维护中的 octocrab。

**`gh` 二进制。** `v2.99.0`（2026-09-01），MIT，发行物覆盖 linux（deb/rpm/tar.gz）、macOS（amd64 / arm64 zip、universal pkg）、**windows（386 / amd64 / arm64 的 zip 与 msi）**。与 HCTL 有关的能力（官方手册）：

- `gh pr view --json <fields>`：字段含 `mergedAt`、`mergeCommit`、`state`、`mergeStateStatus`、`statusCheckRollup`、`headRefOid`、`baseRefName`、`number`、`url` 等。
- `gh pr checks --json <fields>`：字段 `bucket`（把 state 归为 pass / fail / pending / skipping / cancel）、`completedAt`、`description`、`event`、`link`、`name`、`startedAt`、`state`、`workflow`；`--watch --interval --fail-fast`；**checks 未完成时退出码 8**。
- `gh api <endpoint>`：`--jq`、`--template`、`--paginate`、`--slurp`、`--cache <duration>`、`--header`、`--method`、`--field` / `--raw-field`、`--input`、`--include`；`{owner}` / `{repo}` / `{branch}` 占位符从当前仓库或 `GH_REPO` 填。
- 鉴权：`GH_TOKEN` / `GITHUB_TOKEN`（前者优先）**覆盖已存的登录凭据**；否则用 `gh auth login` 存下的 OAuth 令牌；`GH_HOST`、`GH_ENTERPRISE_TOKEN` 选主机；`GH_PROMPT_DISABLED` 关交互；`gh auth token` 可把当前令牌吐出来给别的程序。

## 候选比较

| 维度 | 进程内 SDK（octocrab） | 调用 `gh` 二进制 | 从 OpenAPI 生成 |
| --- | --- | --- | --- |
| 鉴权复用 | HCTL 自己保管凭据：PAT，或 GitHub App 私钥 → 安装令牌（内建） | **复用用户已有的 `gh auth login`**；CI 用 `GH_TOKEN`；不能自己做 App JWT，只能接预先换好的安装令牌 | 同 SDK |
| 速率限制 | 同一套 GitHub 限额（用户令牌 5,000/时；App 安装 5,000 起按仓库 / 成员数加到 12,500，Enterprise Cloud 15,000；`GITHUB_TOKEN` 1,000/时/仓库）；`etag` 条件请求命中 304 **不计入**主限额；`ratelimit()` 读余量；`retry` feature | 限额相同；`gh api --cache` 是本地缓存不是 ETag；无内建余量读取（可 `gh api rate_limit`） | 同 SDK |
| 结构化输出 | 类型化 model；字段落后时靠忽略未知字段 | `--json` 字段白名单 + `--jq`；字段由 gh 版本决定，钉版本即稳定；`gh pr checks` 的 `bucket` 归类与退出码 8 直接就是 `wait` 语义 | 类型全、体量大 |
| Windows | 纯 Rust（rustls 默认），无问题 | 官方 msi / zip；DotSlash 可钉三平台 | 同 SDK |
| 进程模型 | 常驻服务友好；webhook payload 类型现成 | 每次调用 spawn 进程，几十到几百毫秒；没有 webhook 接收能力（`gh webhook forward` 只是开发用扩展） | 同 SDK |
| 版本钉定 | `Cargo.lock` | DotSlash 清单钉 URL + SHA-256（与 [jq](../build-tools/jq.md)、[btd](../build-tools/buck2-change-detector.md) 同一套做法） | 同 SDK |
| 维护 | 月更，1.4k 星，手写 | GitHub 官方，周更 | octorust 18 个月未发布 |

## 边界与取舍

- **鉴权**：control 应以 GitHub App 安装身份运行（限额更高、权限按仓库收窄、webhook 与身份一体），这只有 SDK 路径顺手。`hctl-tool wait` 跑在开发机、只读，复用用户的 `gh` 登录最省事，也符合"合入钥匙不进工具"——工具不保管任何 HCTL 凭据。
- **速率限制**：GitHub 官方最佳实践明说"订阅 webhook 而不是轮询"，轮询必须用条件请求（304 免费）并固定节奏。control 走 webhook（`check_run`、`check_suite`、`status`、`pull_request`、`push`）+ 带 ETag 的确认回读；`wait` 命令是人等结果，轮询间隔由 `gh pr checks --watch --interval` 或我们自己控制。二级限额：并发 ≤ 100、REST 每分钟 ≤ 900 点、内容创建每分钟 ≤ 80。
- **事件与 webhook**：只有 control 需要接；octocrab 的 `models::webhook_events` 提供类型。签名校验（`X-Hub-Signature-256`）自己做。
- **Windows**：两条路径都可用。
- **两套映射的代价**：control 用 octocrab、tool 用 gh，意味着"PR 合并 / checks / ref"三类事实有两份字段映射。缓解：把"事实的领域形状"定义在共享 crate，两条路径各自填充；或让 tool 也用 octocrab、只用 `gh auth token` 借令牌（一次进程调用换鉴权，其余进程内）。后者是保留选项，先不做。

## 决定建议

- 三级判定：**第一级——但按调用方拆开**。官方 Rust SDK 不存在；官方**二进制** `gh` 存在；事实标准 Rust 库 octocrab 存在。不从 OpenAPI 生成。
- `hctl2-control`：**采用 SDK**——octocrab `0.54.1`，GitHub App 鉴权、ETag 条件请求、类型化 webhook payload。
- `hctl-tool wait`：**采用二进制**——`gh v2.99.0`，DotSlash 钉三平台 URL 与 SHA-256；用 `gh pr view --json` / `gh pr checks --json` / `gh api --jq`，只解析白名单字段；`GH_PROMPT_DISABLED=1`、`NO_COLOR=1`；把 `gh pr checks` 退出码 8 映射为"仍在等"。
- 开工前核对：octocrab 0.54.1 的 `checks()` 列表 API 是否覆盖 `check-suites` 的 `app_id` 过滤（源码里看到有）；`gh` 在没有本地 git 仓库时用 `-R owner/repo` 显式指定，避免占位符猜错。

## 证据

- octocrab：[crates.io](https://crates.io/crates/octocrab)（0.54.1，2026-07-24，MIT OR Apache-2.0，rust-version 1.85.0）· [Releases](https://github.com/XAMPPRocky/octocrab/releases) · [`src/api/checks.rs`](https://github.com/XAMPPRocky/octocrab/blob/v0.54.1/src/api/checks.rs) · [`src/api/repos.rs`](https://github.com/XAMPPRocky/octocrab/blob/v0.54.1/src/api/repos.rs)（`get_ref`、`combined_status_for_ref`）· [`src/api/pulls.rs`](https://github.com/XAMPPRocky/octocrab/blob/v0.54.1/src/api/pulls.rs)（`get`、`is_merged`）· [`src/lib.rs`](https://github.com/XAMPPRocky/octocrab/blob/v0.54.1/src/lib.rs)（`app`、`installation`、`personal_token`、`etag`、`ratelimit`、`graphql`）· [`src/etag.rs`](https://github.com/XAMPPRocky/octocrab/blob/v0.54.1/src/etag.rs) · [`src/models/webhook_events.rs`](https://github.com/XAMPPRocky/octocrab/blob/v0.54.1/src/models/webhook_events.rs)
- gh：[Release v2.99.0](https://github.com/cli/cli/releases/tag/v2.99.0)（资产含 windows 386 / amd64 / arm64）· [manual gh pr checks](https://cli.github.com/manual/gh_pr_checks) · [manual gh pr view](https://cli.github.com/manual/gh_pr_view) · [manual gh api](https://cli.github.com/manual/gh_api) · [manual gh help environment](https://cli.github.com/manual/gh_help_environment)（`GH_TOKEN` 优先于已存凭据）
- GitHub 文档：[REST 限额](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api) · [REST 最佳实践](https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api)（304 不计限额、用 webhook 不轮询）· [Checks runs](https://docs.github.com/en/rest/checks/runs) · [Commit statuses](https://docs.github.com/en/rest/commits/statuses) · [Pulls](https://docs.github.com/en/rest/pulls/pulls) · [Git refs](https://docs.github.com/en/rest/git/refs) · [Webhook 事件](https://docs.github.com/en/webhooks/webhook-events-and-payloads) · [OpenAPI 描述仓库](https://github.com/github/rest-api-description)
- 生成路径对照：[octorust crates.io](https://crates.io/crates/octorust)（0.10.0，2025-03-12）
- 本仓库：[Participant 约束](../../design/spec/participant.md) · [jq 的 DotSlash 钉定先例](../build-tools/jq.md) · [任务后端条目（GitHub 外部来源）](../task-backends.md) · [部件矩阵](../component-matrix-20260902.md)

## 复核记录

- 2026-09-04：`hctl2-tool wait` 已采用 `gh v2.99.0`，通过 DotSlash 固定 Linux x86_64、macOS arm64/x86_64、Windows x86_64 官方制品，并把当前三个发行平台的 `gh` 放入离线包。GitHub 事实的字段映射位于共享 `hctl2-facts` crate，未来 control 可以复用同一事实形状；control 自身的 GitHub provider 尚未开工，因此不提前加入未被调用的 octocrab 依赖。

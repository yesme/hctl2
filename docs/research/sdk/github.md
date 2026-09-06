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

- 2026-09-06 第五模块（Repo）方案审定后的复核：① 所有者定了操作外部系统的四级顺序（随包命令行 > 官方 SDK > 按接口描述生成 > 手写），控制面一侧原定的 octocrab 退为第二选择，先核 gh 的调用面能不能覆盖发布评审（推分支、开或更新 PR）、读评审线程与正式评审、读分支保护条件、请求合入；gh 的短处（每次调用起进程、不能接收 webhook、GitHub App 身份只能拿预先换好的安装令牌）到 B2 真用时权衡。② 两个「头」：合并接口的 `sha` 与 `gh pr merge --match-head-commit` 校验的都是源分支的头，GitHub 没有「预期目标头不符就拒绝」的参数；分支保护「要求分支与目标同步」只保证候选包含当前主干，合并队列在最新主干上重新验证后排队合入，语义是「接受目标前移、重新验证」，不是比较并交换。因此 GitHub 的绑定声明「不能保证预期目标头」，集成意图须事前选择授权形态（`docs/research/scm-platforms.md`）。③ `hctl2-tool wait` 现有的三类 GitHub 事实（提交的检查、PR 是否合并、引用是否推进）不含评审线程是否解决、正式评审状态与目标保护条件，B2 前要补。

### 2026-09-06 · P2.4 写侧调用与恢复复核

> 对象：随包 `gh v2.99.0 / d528f20f2ee02f6703773e9f56c90e3c3f5d46b0`，宿主 Git ≥2.39；GitHub REST 与 GraphQL<br>
> 许可证：gh MIT，Git GPL-2.0；GitHub 平台按服务条款使用<br>
> 定位：Repo 模块平台适配器执行获准的发布 / 合入并回读，不把 GitHub 评论或按钮变成新的 HCTL 授权。

#### 上游能力

结构化输出指不靠终端提示措辞解析。表中「身份」是上游实际记下的操作者，由所用凭据决定；不是可任填的 author 字符串。「条件写」指服务端在写入时检查预期旧值，不是先读再写。

| 操作 | 调用及结构化输出 | 身份 | 条件写 / 恢复限制 | 是否需 SDK 降级 |
| --- | --- | --- | --- | --- |
| 推源分支 | 宿主 `git push --porcelain <remote> <精确提交>:refs/heads/<分支>`；porcelain 是稳定的制表符格式，**不是 JSON** | HTTPS credential helper 可调用 `gh auth git-credential`，平台记用户或 App；Git commit 的作者 / 共同作者独立保存 | 更新已授权的源 ref 时用显式 `--force-with-lease=<ref>:<已知旧SHA>` 作旧值校验；新 ref 以空旧值要求不存在。此标志只保证比较，不代替重写授权或分支保护 | 无；gh 本来没有 push 子命令，现成 Git 就是调用方 |
| 创建 PR | `gh pr create -R … --head … --base … --title … --body-file …` 回 URL；需要 JSON 可用同二进制 `gh api --method POST repos/…/pulls --input …` | 令牌对应用户 / App | 没有服务端创建幂等键，也无冻结源 SHA 参数；先推精确提交再创建，创建后核 head、base、描述与关联信息 | 无；切 `gh api` 仍是采用二进制 |
| 更新 PR 描述 | `gh pr edit <编号> -R … --body-file …`；或 `gh api --method PATCH repos/…/pulls/<编号>` 回 JSON | 同上；不能替别人发言 | 公开更新接口无预期 head / body 版本的原子比较；写前及写后摘要只作漂移检查，不宣称能排除原生客户端的并发修改 | 无；SDK 无法补服务端比较 |
| 请求合入 | `gh pr merge <编号> -R … --merge --match-head-commit <源SHA> --subject <PR标题> --body-file <三节描述文件>`；高层命令无 JSON，随后 `pr view --json` / `api` 回读 | 令牌对应用户 / App，另可指定 Git 合并提交署名字段，二者不是同一种身份 | `expectedHeadOid` 只比源头。无目标头比较并交换；成功退出可能仅已接受 / 已排队 | 无；需要单次 JSON 响应可用 `gh api`，不改授权形态 |
| 写普通评论 | `gh pr comment <编号> -R … --body-file …` 回 URL；`gh api --method POST repos/…/issues/<编号>/comments` 回含 ID 的 JSON | 评论的实际作者是令牌账号 | 创建无服务端幂等键；保存 comment ID，确认丢失按操作关联与正文摘要查重；`--edit-last` 不能证明编辑的是本次操作那条 | 无 |
| 读评审线程 | `gh api graphql` 查询 `reviewThreads { nodes { id isResolved isOutdated } pageInfo { … } }` | 读权限决定可见范围，不生成作者身份 | 分页取全；`isOutdated` 不等于 `isResolved`，查询失败 / 部分页不等于无未解决线程 | 无；`pr view --json` 没列出的字段用原生 API |
| 读正式评审 | REST `/pulls/<编号>/reviews`，或 GraphQL `reviews/latestReviews`、`reviewDecision` | 保留评审账号稳定 ID、state、commit ID 与提交时间 | 普通 issue comment 不是 APPROVED / CHANGES_REQUESTED。当前资格还受旧评审失效、最后推送、CODEOWNERS 等规则约束；不数评论文字里的「通过」 | 无 |
| 读保护与合并资格 | REST `/branches/<目标>/protection`、`/rules/branches/<目标>` 及仓库合并设置；`pr view --json mergeable,mergeStateStatus,reviewDecision,headRefOid,baseRefOid,statusCheckRollup` | user / App 均可按权限读；保护端点可能需要 Administration:read | 旧式 branch protection 与 rulesets（另一套仓库规则）一起核。`UNKNOWN`、403、不可解释的 404 与 GraphQL 部分错误不算通过；快照冻结后仍在执行与回读时对照 | 无 |

`gh pr create` 源码明确：给 `--head` 后不代推分支。这样可以分清「推送已成功」与「创建 PR 未确认」，避免自动 fork / 自动推当前 HEAD。描述三节由 control 按冻结发布策略组装，模型产出只进入获准的文本部分；评论写回仍是 content，不变成正式评审或 Gate 的一票。

`gh auth setup-git` 不是无副作用探测：钉定源码会写 **global** credential helper，先清空对应 host 的 helper 链，再加自己的命令。人工接入可显式运行；control 正常写操作复用已配置凭据，需局部注入时用 Git 原生进程级配置限定到受信任的 Git 子进程，不每次重写用户全局配置。GitHub 令牌及 helper 不下发到执行体；可选加固还需落实工作树不继承凭据的检查，本次没有运行 setup-git 或改用户配置。

#### 源头、目标头与合并设置

gh `merge.go` 把 `--match-head-commit` 放进 payload，`http.go` 传成 GraphQL `expectedHeadOid`。REST 合入接口的 `sha` 同样检查源头。这两条都没有预期目标头参数，因此维持[Repo §集成](../../design/spec/repo.md#集成目标两个头与两种授权形态)的 GitHub 能力声明：只在有权 actor 事前选择「接受目标前移」后执行，不到写入时临时放宽。

目标要求合并队列时，gh 即使没给 `--auto` 也会走启用自动合并 / 入队路径，方式由队列决定；这时 `--merge` 不是覆盖队列策略的指令。P2.4 要核队列配置与冻结方式相符，回读已合并状态、合并提交及实际目标头后才给成功证据。`--admin` 不能作为满足保护条件的办法。

本次还通过 GraphQL 自省核到 `EnablePullRequestAutoMergeInput.expectedHeadOid`，但它的字段说明不足以证明「源头在入队后变化，整次请求必取消而不会跟随新头」。本批优先直接合入，不主动开启 auto-merge；队列 / 自动合并在完成这项竞争实验前不声明能保持冻结源版本。发现源头变化才事后取消，不能补成已验证的原子保证。队列还会忽略传入的 merge method、commit headline / body，需以队列实际策略核验。

所有者在 `.memo/design/scm-module-20260906/02-attribution-gate.md` §二第 5 项已定：本库只留 merge commit，合并提交带 PR 标题与描述。这是**决定，不是已应用的设置**。2026-09-06 只读实查 `yesme/hctl2`：

| 事实来源 | 回读值 | 对 P2 的含义 |
| --- | --- | --- |
| 仓库设置 | `allow_merge_commit=true`、`allow_squash_merge=true`、`allow_rebase_merge=true`；`merge_commit_title=MERGE_MESSAGE`、`merge_commit_message=PR_TITLE` | 尚未只留 merge，也不是 title + description。实现要检测并提示，不能写「仓库现已如此」；本研究不改远端设置 |
| main 的旧式保护 | strict 同步；必需检查 CI gate、Release gate、PR contract；要求线程解决；批准数为 0；不要求线性历史；管理员也受保护 | 三个检查变绿不代表 GLM 已审；人工评审要求仍由本轮工作约定保障 |
| main 的有效 rulesets | `deletion`、`non_fast_forward` | 补充禁止删除 / 非快进改 ref 的规则，不等于选择快进合并策略，也不等于预期目标头比较 |
| PR #186 的 GraphQL 回读 | `reviewThreads`、`latestReviews` 为空，`reviewDecision=null` | 这个 PR 的普通评审评论不能被当成平台的正式批准；空列表只说明该查询的对象，不证明 HCTL 评审通过 |

按[署名任务书 §二第 5、6 项](../../../.memo/design/scm-module-20260906/02-attribution-gate.md#二hctl2-要做的事)，本批直接合入用 `--merge`，并显式传 `--subject <PR标题>` / `--body-file <三节描述文件>`；描述由 control 组装，不依赖仓库缺省消息，也不另走 squash。描述会落进 Git，因此 control 仍需排除密钥；队列会忽略这些消息参数的限制仍保留，不能由直接合入的结论推定队列同样可用。

#### 确认丢失后的关联

分支名是查询条件，不是完整关联键。发布意图先固定平台仓库 ID、head 仓库身份与分支、base ref、精确 ChangeSet Revision 和此次发布的描述摘要；结果里保存 PR 的稳定 ID / 编号、实际提交。后续原生改名或关闭不改写这份发布证据。

推送成功但创建确认丢失时，用 `gh api --method GET repos/…/pulls`，显式给 `head=<owner>:<branch>`、`base=<ref>`、`state=all` 并取全分页，再核候选的 head 仓库 ID、源 SHA、目标与该次发布关联。已关闭或已合并的也可能正是原结果，不能只查 open。可把已有意图 ID / Revision 引用写进 control 拥有的描述关联区，但它只是匹配线索，不是授权；命中多个、被编辑或仍查不到时保留结果未知，不按标题选第一个，也不直接再建一条。

更新确认丢失后，回读原 PR / comment ID 与描述摘要；有新 Revision 或不同正文就报告冲突，不让旧重试覆盖。合并确认丢失后，回读 PR 的 `merged` / `mergedAt`、`merge_commit_sha` / `mergeCommit`，再核目标历史和固定源版本；不能仅凭目标 ref 前移就算本次完成。平台 2026 年还提供异步合并请求 ID，但它并不增加目标头比较能力，本次不切换调用路线。

#### 身份、限流与候选比较

| 凭据 | 实际平台身份 | 代价与选择 |
| --- | --- | --- |
| 用户 OAuth / PAT（个人访问令牌） | 操作归这个用户 | P2 首选：复用已登录的 gh，或由现有密钥存储向受信任子进程提供令牌；绑定记录实查账号，不从 Git author 猜账号 |
| GitHub App 安装令牌 | 操作归 App bot，不归某位人 | gh 可接预先换好的 `GH_TOKEN`，但不负责生成 App JWT、换取和续期安装令牌；安装令牌通常 1 小时，权限按安装范围收窄 |
| GitHub App 用户访问令牌 | 用户身份并带 App 来源 | 不是安装令牌的「改名字」功能，需要该用户授权；本批不为各 Participant 新建账号或 OAuth 系统 |

读保护条件、写 PR / 评论、推送 / 合入分别需要相应的 Administration:read、Pull requests:write、Contents:write 等权限；缺权限是绑定能力不足，不是换库即好。平台 actor、Git 提交作者、HCTL human 授权者与 Participant 可以不同，分别记录；共享账号的多条评论不能形成多张席位票。

限流维持同一套 GitHub 配额：一般用户 REST 为 5,000 次/小时；安装令牌从 5,000 起，按安装规模可到 12,500，Enterprise 条件另算。GraphQL 另计点数，共享二级并发限制；依据响应的剩余额度、reset、Retry-After 及 403/429 退避，不把所有 403 当限流。当前回读不使用 `gh api --cache` 的旧数据充数。ETag 适合 GET 轮询；请求批次与分页都保留失败状态，webhook 只触发复查，未收到不能证明未发生。

| 候选 | 版本 / 许可证 | MSRV / 宿主依赖 | 本次判定 |
| --- | --- | --- | --- |
| gh + 宿主 Git | gh 2.99.0 / MIT；Git ≥2.39 / GPL-2.0 | 调二进制，MSRV 不适用；沿用三平台随包 gh 与宿主 Git 检查 | **采用二进制**；所有本批操作有原生 CLI 或 `gh api` 路径 |
| octocrab | 0.54.1 / MIT OR Apache-2.0 | MSRV 1.85，符合本库 1.98 | 暂缓；将来若必须在 control 内管理 App 换票再核，不为补 target CAS 或 JSON 输出引入 |
| 整份 GitHub OpenAPI 生成 | 本次未选生成器版本 | 未引入 | 暂缓；不是 CLI 已覆盖调用的替代要求 |

#### 边界与取舍

gh 固定的是客户端，不是远端 API 行为。2.99.0 源码的 REST 默认版本头仍为 `2022-11-28`，本库 `hctl2-facts` 也用它；截至复核日官方支持到 2028-03-10，最新版本是 `2026-03-10`。P2 为复用现有事实读取先维持旧版本头，升级时单独验证；GraphQL 字段也需固定查询与样本，新增未知值按未知处理。

本次对 gh 的创建 / 更新 / 合入 / 评论参数与 payload 做了钉定源码核对，对本库设置、保护和 PR 线程 / 评审作了只读 API 实查；没有拿用户 PR 做合并反例，没有伪造批准或修改仓库规则。App 安装身份的实际写入与过期、源头竞争、断网丢确认等仍待 P2.4 在专门试验仓库验证。

#### 决定建议

**维持采用二进制：control 与工具箱都用已随包的 `gh 2.99.0 / d528f20f`，推分支复用宿主 Git ≥2.39；P2 先复用用户登录，octocrab `0.54.1` 暂缓。** 高层命令缺 JSON 时改调同一 gh 的原生 API，写侧、线程、评审和保护读取均不需要降级到 SDK。缺的是 GitHub 的目标头条件写与部分写入幂等保证，不是 CLI：绑定如实声明，集成只按事前选定的「接受目标前移」，确认丢失按精确发布关联回读。本文不把旧文的「control 必须用 App」当已定前提，也不把仓库尚未应用的 merge-only 设置写成现状。

本批直接合入固定 `--merge --match-head-commit`，并用 `--subject` / `--body-file` 显式携带 PR 标题与 control 组装的三节描述，保留提交与评审修正的来历；队列 / 自动合并仍待冻结源版本的竞争实验。

#### 证据

- gh 钉定源码：[create 的显式 head 与发布](https://github.com/cli/cli/blob/d528f20f2ee02f6703773e9f56c90e3c3f5d46b0/pkg/cmd/pr/create/create.go)、[merge 策略与队列](https://github.com/cli/cli/blob/d528f20f2ee02f6703773e9f56c90e3c3f5d46b0/pkg/cmd/pr/merge/merge.go)、[expectedHeadOid 请求](https://github.com/cli/cli/blob/d528f20f2ee02f6703773e9f56c90e3c3f5d46b0/pkg/cmd/pr/merge/http.go)、[setup-git 的 global helper 写入](https://github.com/cli/cli/blob/d528f20f2ee02f6703773e9f56c90e3c3f5d46b0/pkg/cmd/auth/shared/gitcredentials/helper_config.go)、[默认 API 版本](https://github.com/cli/cli/blob/d528f20f2ee02f6703773e9f56c90e3c3f5d46b0/api/client.go)。
- 写入接口：[gh pr create](https://cli.github.com/manual/gh_pr_create)、[edit](https://cli.github.com/manual/gh_pr_edit)、[merge](https://cli.github.com/manual/gh_pr_merge)、[comment](https://cli.github.com/manual/gh_pr_comment)、[gh api](https://cli.github.com/manual/gh_api)、[Git push porcelain / force-with-lease](https://git-scm.com/docs/git-push)、[GitHub PR REST](https://docs.github.com/en/rest/pulls/pulls)、[评论 REST](https://docs.github.com/en/rest/issues/comments)。
- 读取与身份：[正式评审](https://docs.github.com/en/rest/pulls/reviews)、[分支保护](https://docs.github.com/en/rest/branches/branch-protection)、[有效 rulesets](https://docs.github.com/en/rest/repos/rules)、[App 安装身份](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/authenticating-as-a-github-app-installation)、[App 用户身份](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/about-authentication-with-a-github-app)、[API 版本支持期](https://docs.github.com/en/rest/about-the-rest-api/api-versions)、[限流](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)。GraphQL 的线程与正式评审字段已通过本库 PR #186 查询验证，未以 SDK 字段表代替实际接口。
- 本库：[平台能力对照及快进反例](../scm-platforms.md#复核记录)、[Repo §平台绑定与能力声明](../../design/spec/repo.md#平台绑定与能力声明)、[Repo §发布评审](../../design/spec/repo.md#发布评审)、[Git 现场引擎](./git.md)。现有 `src/crates/hctl2-facts/src/lib.rs` 的平台侧事实只有 checks、PR merged、ref advanced 三类，另有非平台侧的 path-digest、process-exited；P2.4 仍需补线程 / 正式评审 / 保护回读，不能写作已经实现。

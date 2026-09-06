# 本地代码协作平台：Gitea（限时验证）

> 类别：⑥ 机械后端与基础设施 · 证据编号：E-SCM-GITEA<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 上级调研：[代码协作平台市场调研](./scm-platforms.md)（各家合入调用面的横向对照）；总览、引用准入与复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-scm-gitea"></a>
## E-SCM-GITEA · 本地代码协作平台选型（限时验证）

决策史 §36（v0.17.1）：只在本地的仓库缺省绑定随包的本地代码协作平台，评审请求、评审线程、保护规则与合入都在它上面走，与外部平台共用同一个平台端口。所有者点名 Gitea：小、单二进制、功能全。本文核对它在 HCTL 调用面上的每一项，给绑定的能力声明提供实测依据；不评估 Gitea 自身的其他功能。

## 审计基线

- 版本：[Gitea v1.27.3](https://github.com/go-gitea/gitea/releases/tag/v1.27.3)，2026-08-29 发布；上游按月出补丁版，1.27.0 发布于 2026-07-13。
- 许可：MIT（Copyright 2016 The Gitea Authors、2015 The Gogs Authors）。
- 发布物：每个平台一个单文件可执行，内嵌静态资源，自带 SQLite、MySQL、PostgreSQL 驱动。HCTL 消费的四个：`gitea-1.27.3-linux-amd64`（120 MiB）、`gitea-1.27.3-linux-arm64`（111 MiB）、`gitea-1.27.3-darwin-10.12-amd64`（120 MiB）、`gitea-1.27.3-darwin-10.12-arm64`（112 MiB）。文件名里的 10.12 是历史命名，实际最低 macOS 由 Go 工具链决定，低于 HCTL 的 macOS 15 基线。下载包是 xz 压缩，darwin arm64 37 MiB、linux amd64 41 MiB。每个发布物附 `.sha256`、GPG `.asc` 与 Sigstore `.sigstore.json`，打包时按 SHA-256 锁定。
- 官方命令行：[tea v0.15.1](https://gitea.com/gitea/tea/releases/tag/v0.15.1)，2026-08-02 发布，darwin/linux × amd64/arm64 单二进制；`--output json` 逐命令可用。
- 宿主依赖：Gitea 调用宿主 git（文档要求 2.0 以上），与工具箱用的是同一个宿主 git（HCTL 下限 2.39，见 [sdk/git.md](./sdk/git.md)）；不需要数据库服务，SQLite 文件默认在 `data/gitea.db`。

## 为什么是它

所有者给的三条理由都核实了。单二进制：见上。小：一个进程、内嵌 SQLite，不需要外部数据库或消息队列。功能全：评审请求、正式评审、评论线程与解决状态、分支保护、提交状态、webhook 都有 REST 接口，对象形状与 GitHub 同构（pull request、review、commit status、branch protection），平台适配器可以沿用为 GitHub 设计的调用面，只换后端。此外它有官方命令行，符合所有者定的四级顺序第一级。

## 调用面核对

按[市场调研的十个维度](./scm-platforms.md#能力声明的维度)逐项核对，接口以 v1.27.3 的源码与 REST 路由为准，路径前缀 `/api/v1`：

| 维度 | Gitea 的对应 | 能力声明 | 依据 |
| --- | --- | --- | --- |
| 评审单位与身份 | Pull request，仓库内的 `index` | 有 | `/repos/{owner}/{repo}/pulls/{index}` |
| 源头校验 | 合并表单的 `head_commit_id` | 有 | `MergePullRequestForm`，见市场调研 |
| 预期目标头保证 | 无；`fast-forward-only` 只保证祖先关系 | 无 | 市场调研 2026-09-06 复核记录 |
| 目标前移后的行为 | 按合并策略：非快进策略直接合入，`fast-forward-only` 拒绝非快进 | 记为保护条件 | 同上 |
| 评审状态形态 | 正式评审：`APPROVED` / `REQUEST_CHANGES` / `COMMENT`（另有 `PENDING`、`REQUEST_REVIEW`），带 `official`、`stale`、`dismissed` | 有 | `modules/structs/pull_review.go` |
| 线程解决状态 | 评审评论带 `resolver`（解决者）；分支保护没有「线程必须解决」这一条件 | 能读；不能作平台侧合入前置，只能作 HCTL 预览里的契约项 | 同上；`modules/structs/repo_branch.go` |
| 检查结果来源 | 提交状态 `POST /repos/{owner}/{repo}/statuses/{sha}`、`GET /repos/{owner}/{repo}/commits/{ref}/status`；平台自带流水线（Actions）要另装 Gitea Runner，本批不随包 | 外部状态写回 | `routers/api/v1/api.go`；Actions 概览 |
| 目标保护条件回读 | `GET / POST / PATCH /repos/{owner}/{repo}/branch_protections`：`enable_status_check` 与 `status_check_contexts`、`required_approvals`、`block_on_rejected_reviews`、`block_on_official_review_requests`、`block_on_outdated_branch`、`dismiss_stale_approvals`、`enable_push`、`block_admin_merge_override`，按 `rule_name` 通配 | 有，整份可读可写 | `modules/structs/repo_branch.go` |
| 身份映射 | 本地账号；`gitea admin user create` 建账号，`gitea admin user generate-access-token --scopes` 出令牌 | 有 | 命令行文档 |
| 官方命令行 | tea：`pulls create / review / approve / reject / merge / resolve / unresolve / review-comments`、`branches protect / unprotect`、`repos create / migrate`、`api`（任意 REST 调用）；`pulls merge --style` 只有 merge / rebase / squash / rebase-merge，没有 `fast-forward-only`，也没有 `head_commit_id` | 有，部分 | tea CLI 清单 |

两处要在适配器里补：合并要带源头校验时走 `tea api -X POST` 调 `/repos/{owner}/{repo}/pulls/{index}/merge` 传 `head_commit_id`；提交状态写回与分支保护的字段级读写也走 `tea api`。`tea api` 仍是随包的官方命令行，不算降级。

## 运行形态

- 监听：`[server] PROTOCOL` 支持 `http`、`https`、`http+unix`；`http+unix` 时 `HTTP_ADDR` 是套接字路径（`UNIX_SOCKET_PERMISSION` 默认 666）。tea 的 `login add --url` 是 HTTP 地址，没有套接字选项，所以本地平台先监听回环地址上的一个端口，套接字形态待核。
- 锁定：`[security] INSTALL_LOCK = true` 关闭安装页；`[service] DISABLE_REGISTRATION = true` 只允许管理员建账号；`REQUIRE_SIGNIN_VIEW = true` 未登录不能读任何页面或 API；`[server] DISABLE_SSH = true`，本机只走 HTTP 推送。
- 数据：`[database] DB_TYPE = sqlite3`，`PATH` 默认 `data/gitea.db`；`APP_DATA_PATH` 与 `[repository] ROOT`（默认 `{APP_DATA_PATH}/gitea-repositories`）都放到 HCTL 的用户级数据目录下。`gitea dump` 与 `gitea restore-repo` 是官方备份与恢复入口，`gitea migrate` 在升级后跑库迁移，`gitea doctor check` 做自检。
- 启动：`gitea web --config <app.ini>`，随包由 control 经 Process Compose 托管（见 [process-compose.md](./runtime/process-compose.md)），与 Tuwunel、Vikunja 同列。
- 账号：一个管理员账号给 control，适配器持它的令牌；有权的人各自一个普通账号；平台账号到人的映射写在绑定里。

## 不采用与边界

- **不随包 Actions 执行器。** Gitea Actions 要另装 Gitea Runner（独立程序）。本地平台上的检查只有外部写回一种来源，来源是工具箱回读的本地测试事实。
- **不为外部平台的克隆建镜像。** 来自 GitHub、GitLab 的仓库，评审与合入在来源平台上走；Gitea 的 `repos migrate` 能做镜像，HCTL 不用它。
- **没有预期目标头保证。** 与 GitHub 相同，集成意图只能选「接受目标前移」形态。
- **备选 Forgejo。** Gitea 的社区分支，自 v9 起 GPL-3.0-or-later，API 高度兼容；tea 对它的兼容程度待核。

## 候选对照：Gogs（所有者 2026-09-07 提出）

所有者看到 Gitea 单二进制一百多 MiB，要求把 Gogs（Gitea 2016 年从它分叉出来）加入候选，能选就选它。核对结果：**落选**，两条理由都站得住。

**接口承载不了 PR 过程。** Gogs v0.14.3（2026-06-07 发布，MIT）的 REST 路由表（`internal/route/api/v1/api.go`）里，仓库一级只有：仓库的建、迁、删，webhook，协作者，文件内容与原始文件，git 树与 blob，fork、tag、分支列表与单个分支，提交，部署密钥，issue 与评论，release。**没有**评审请求（pull request）的任何接口，没有提交状态，没有分支保护，没有正式评审。Web 界面里有 pull request，但只有评论，没有批准或请求修改这类评审状态；分支保护只有三个开关——保护、要求经 pull request、推送白名单（`internal/database/repo_branch.go` 的 `ProtectBranch`），没有必需检查和批准数。对照十个维度：评审单位（接口无）、源头校验（无）、评审状态（无）、线程解决（无）、检查（无）、保护条件回读（无）、官方命令行（无）。适配器要驱动它的 PR 只能抓网页，四级顺序里没有这一级。

**也不算小。** 解压后的单二进制（darwin arm64）：Gogs 87 MiB，Gitea 112 MiB，只差四分之一；下载包 Gogs zip 39 MiB、Gitea xz 37 MiB，反而 Gitea 更小。放到我们已选的随包服务里看，Gitea 是同一量级：

| 随包服务（darwin arm64，解压后） | 单二进制 |
| --- | --- |
| Dagu 2.16.2 | 149 MiB |
| Gitea 1.27.3 | 112 MiB |
| Vikunja 2.6.0 | 109 MiB |
| Gogs 0.14.3（未选） | 87 MiB |
| Tuwunel 1.9.0 | 下载包 31 MiB（zst） |
| Herdr 0.8.2（linux x86_64） | 22 MiB |

Gitea 大在内嵌的前端资源、模板与三种数据库驱动，和 Vikunja、Dagu 大的原因相同。维护节奏也在 Gitea 这边：Gogs 一年两三个补丁版、一千余个未关 issue，Gitea 按月出补丁。

结论：Gogs 记为**暂缓**，只作对照，不进依赖。要真正更小的本地平台，得等一个既小又有完整评审请求接口的实现出现，目前没有。

## 决定建议

- **采用二进制**：Gitea v1.27.3 作为随包的本地代码协作平台，四个官方单二进制按 SHA-256 锁定；Forgejo 暂缓、记为备选。
- **适配器第一级用 tea**：`pulls`、`branches`、`repos` 子命令覆盖建仓、发布评审、评审与合并的主路径；源头校验的合并、提交状态、分支保护字段用 `tea api` 直调 REST。不引入 Go SDK，Rust 侧没有官方 SDK。
- **待核项**：`http+unix` 与 tea 的配合；Forgejo 的 tea 兼容性；`REQUIRE_SIGNIN_VIEW` 打开时 webhook 的行为。

## 依据

- 发布：[v1.27.3](https://github.com/go-gitea/gitea/releases/tag/v1.27.3) · [tea v0.15.1](https://gitea.com/gitea/tea/releases/tag/v0.15.1) · [LICENSE](https://github.com/go-gitea/gitea/blob/main/LICENSE)
- 文档：[二进制安装](https://docs.gitea.com/installation/install-from-binary) · [配置速查](https://docs.gitea.com/administration/config-cheat-sheet) · [命令行](https://docs.gitea.com/administration/command-line) · [Actions 概览](https://docs.gitea.com/usage/actions/overview)
- Gogs 对照：[v0.14.3 发布页](https://github.com/gogs/gogs/releases/tag/v0.14.3) · [`internal/route/api/v1/api.go`](https://github.com/gogs/gogs/blob/main/internal/route/api/v1/api.go) · [`internal/database/repo_branch.go`](https://github.com/gogs/gogs/blob/main/internal/database/repo_branch.go)
- 源码：[`routers/api/v1/api.go`](https://github.com/go-gitea/gitea/blob/main/routers/api/v1/api.go) · [`modules/structs/repo_branch.go`](https://github.com/go-gitea/gitea/blob/main/modules/structs/repo_branch.go) · [`modules/structs/pull_review.go`](https://github.com/go-gitea/gitea/blob/main/modules/structs/pull_review.go) · [tea CLI 清单](https://gitea.com/gitea/tea/src/branch/main/docs/CLI.md)

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
- 发布物：每个平台一个单文件可执行，内嵌静态资源，自带 SQLite、MySQL、PostgreSQL 驱动。HCTL 消费的四个：`gitea-1.27.3-linux-amd64`（120 MiB）、`gitea-1.27.3-linux-arm64`（111 MiB）、`gitea-1.27.3-darwin-10.12-amd64`（120 MiB）、`gitea-1.27.3-darwin-10.12-arm64`（112 MiB）。文件名里的 10.12 是历史命名，实际最低 macOS 由 Go 工具链决定，低于 HCTL 的 macOS 15 基线。下载包是 xz 压缩，darwin arm64 37 MiB、linux amd64 41 MiB；打包时锁定 xz 制品下载、解压后原样装入。HCTL 安装包整包用 xz `-9 -T0`：同一个 darwin arm64 二进制实测 gzip 41 MiB、上游 xz 37 MiB、zstd 最高档 34 MiB、xz `-9 -T0` 30 MiB（交付文档用十进制 MB 记同一组数），`-9e` 只再省 0.3% 且慢两成；多线程模式的输出与核数无关（xz 5.4 起），可复现。不用 UPX：UPX 自 4.2.0 起禁用 macOS 支持，且会改写上游制品。每个发布物附 `.sha256`、GPG `.asc` 与 Sigstore `.sigstore.json`，打包时按 SHA-256 锁定。
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
| 身份映射 | 本地账号；`gitea admin user create` 建账号，`gitea admin user generate-access-token --scopes` 出令牌。人的普通账号走同一条命令：`--random-password` 把生成的口令打到 stdout，且打印在建用户之前（`cmd/admin_user_create.go:155` 对 `:228`）；不显式传 `--must-change-password` 时个人账号默认须改口令，库里已有用户就恒为真（`:161-179`），所以那是初始口令而不是长期密钥；建出的账号 `IsActive` 为真（`:202`）、不带 `--admin` 就不是管理员。协作权走 `PUT /repos/{owner}/{repo}/collaborators/{username}`，body `{"permission":…}`，未识别的值回落到 `read` 而不是升级（`routers/api/v1/repo/collaborators.go:183-185`），成功回 204，账号不存在是 422 而不是 404（`:170`）；同一 URL 的 GET 是协作者回 204 空正文、不是回 404（`:121-125`），所以「先读后写」要能接受空正文的成功——但空正文里**没有权限级别**，读不出平台当前持的是 `read` 还是 `write`；因此回读不能当作「级别已是请求值」的证据去跳过 PUT，只能用来确认人在名单上，PUT 对同一级别幂等、对另一级别改到请求值。`gitea admin user list` 在账号行之上还有一行表头，第二列正是字面量 `Username`（`cmd/admin_user_list.go:44`；数据行首列是 `%d` 数字 ID，`:47`；`:41` 的 tabwriter 以空格补齐，所以列间是 tab 加空格），按第二列判「账号已存在」必须先排掉表头，否则一个真叫 `Username` 的人会被当成已存在、跳过建号，随后的协作者 PUT 撞 422 | 有 | `cmd/admin_user_create.go`、`cmd/admin_user_list.go`、`routers/api/v1/repo/collaborators.go`、`routers/api/v1/api.go:1285-1292` |
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
- **tea 随包、适配器第一级用它**：四平台单二进制按 SHA-256 锁定；`pulls`、`branches`、`repos` 子命令覆盖建仓、发布评审、评审与合并的主路径；源头校验的合并、提交状态、分支保护字段用 `tea api` 直调 REST。不引入 Go SDK，Rust 侧没有官方 SDK。
- **待核项**：`http+unix` 与 tea 的配合；Forgejo 的 tea 兼容性；`REQUIRE_SIGNIN_VIEW` 打开时 webhook 的行为。

## 依据

- 发布：[v1.27.3](https://github.com/go-gitea/gitea/releases/tag/v1.27.3) · [tea v0.15.1](https://gitea.com/gitea/tea/releases/tag/v0.15.1) · [LICENSE](https://github.com/go-gitea/gitea/blob/main/LICENSE)
- 压缩：[UPX NEWS](https://github.com/upx/upx/blob/devel/NEWS)（4.2.0「disable macOS support until we fix compatibility with macOS 13+」）· xz 5.8.3 手册 `--threads`（多线程压缩器与核数无关）
- 文档：[二进制安装](https://docs.gitea.com/installation/install-from-binary) · [配置速查](https://docs.gitea.com/administration/config-cheat-sheet) · [命令行](https://docs.gitea.com/administration/command-line) · [Actions 概览](https://docs.gitea.com/usage/actions/overview)
- Gogs 对照：[v0.14.3 发布页](https://github.com/gogs/gogs/releases/tag/v0.14.3) · [`internal/route/api/v1/api.go`](https://github.com/gogs/gogs/blob/main/internal/route/api/v1/api.go) · [`internal/database/repo_branch.go`](https://github.com/gogs/gogs/blob/main/internal/database/repo_branch.go)
- 源码：[`routers/api/v1/api.go`](https://github.com/go-gitea/gitea/blob/main/routers/api/v1/api.go) · [`modules/structs/repo_branch.go`](https://github.com/go-gitea/gitea/blob/main/modules/structs/repo_branch.go) · [`modules/structs/pull_review.go`](https://github.com/go-gitea/gitea/blob/main/modules/structs/pull_review.go) · [tea CLI 清单](https://gitea.com/gitea/tea/src/branch/main/docs/CLI.md)

## 2026-09-15 · D 批 issues 作任务源的调用面复核

> 对象：Gitea v1.27 源码（`modules/structs/issue.go`、`routers/api/v1/api.go`、`modules/webhook/type.go`，release/v1.27 分支）；tea 命令行文档（`docs/CLI.md`，main 分支；随包 tea v0.15.1 的逐命令核对留给运行验证）<br>
> 定位：只在本地的仓库缺省绑定本地平台，它的 issues 是缺省任务源（D 批 #230 拍板甲）。

| 操作 | 调用及结构化输出 | 条件写 / 限制 | 备注 |
| --- | --- | --- | --- |
| 列 / 读卡 | REST `GET /repos/{owner}/{repo}/issues`、`GET …/issues/{index}`；`tea issues list -o json --fields …`，`--state all\|open\|closed`、`--milestone`、`--labels` | 只读 | 实体键 `id`（int64）；`number` 是仓库内 index |
| 建卡 | `POST …/issues`（`CreateIssueOption`：title、body、ref、assignees、due_date、milestone、labels、projects、closed）；`tea issues create` | 无幂等键，创建后查重 | — |
| 编辑 | `PATCH …/issues/{index}`（`EditIssueOption`：title、body、ref、assignees、milestone、projects、state、due_date、unset_due_date、`content_version`）；`tea issues edit --title / --description`，`tea issues close / reopen`；字段级走 `tea api -X PATCH … --data` | `content_version` 的源码注释是「用于编辑时检测冲突」：条件写入声明为「有」；tea 子命令是否传它待核，不传时用 `tea api` 显式带 | — |
| 评论 | `POST …/issues/{index}/comments`；`GET / PATCH / DELETE …/issues/comments/{id}` | 回含 ID | 写回评论带控制面与 Task 标识 |
| milestone / 标签 | `…/milestones`、`…/labels` 的增删改查；`tea milestones`、`tea labels` | — | 源内分组锚点用 milestone 或标签 |
| 看板位置 | v1.27 的 API 路由表里没有 projects（看板）路由 | 看板位置声明为「无」，位置由 state 加 milestone、标签派生 | 平台自带的项目看板只能在界面里动，HCTL 不读它 |
| 观测 | webhook 事件 `issues`、`issue_assign`、`issue_label`、`issue_milestone`、`issue_comment` | 只唤醒，接纳前回读 | 仓库 hooks：`/repos/{owner}/{repo}/hooks` |

决定：采用随包 tea 加 `tea api`，不新增 SDK；绑定能力声明：建卡与字段写回「有」、条件写入「有（`content_version`）」、看板位置「无」。运行验证（P2.2 使用前，研究记录不代替）：tea v0.15.1 各子命令 `-o json` 的字段、`content_version` 冲突时的实际响应、`http+unix` 形态下的 tea。

## 2026-09-17 · 本机运行验证（Gitea 1.27.3 + tea 0.15.1，darwin-arm64）

> 对象：`gitea-1.27.3-darwin-10.12-arm64`、`tea-0.15.1-darwin-arm64`（dl.gitea.com，SHA-256 与发布页一致）；回环端口、sqlite、`INSTALL_LOCK`、`DISABLE_REGISTRATION`、`REQUIRE_SIGNIN_VIEW = true`；webhook 指向本机监听器。全程在本机完成，无外网请求。<br>
> 结论：09-15 调用面复核的每一格都跑通；三个待核项里 `http+unix` 与 `REQUIRE_SIGNIN_VIEW` 下的 webhook 有答案，Forgejo 只做了文档对照——它没有 darwin 制品，本机跑不了。

| 待核 / 复核项 | 观察 | 对设计的意思 |
| --- | --- | --- |
| `content_version` | issue JSON（建、读、列）直接带 `content_version`，从 0 起；`PATCH` 带旧值回 `409 {"message":"the issue is already changed"}`；不带该字段则无条件覆盖、版本号照样递增；title 与 body 都受它保护；评论的 `PATCH` 没有版本号 | 条件写入「有」成立，范围只是 issue 编辑（title、body 等）；写回 issue 一律带上读到的 `content_version`，不带等于放弃锁；评论接口没有版本锁，只追加不改写 |
| 列与轮询 | `GET …/issues?state=all&type=issues&since=…&limit=…&page=…` 生效，`since` 按 `updated_at` 过滤，回 `X-Total-Count` 与 `Link`（rel=next / last）；跨仓库 `GET /repos/issues/search?since=…` 可作控制面一次轮询的入口 | 轮询对账用 `since` 加分页，不逐卡读；这不取消启动 Run 前置所要求的当前回读 |
| webhook 与 `REQUIRE_SIGNIN_VIEW` | 打开时未登录访问 API 一律 `403 "Only signed in user is allowed to call APIs."`（含 `/version`），网页 303 到登录页；出站 webhook 不受影响：`issues`（opened / edited / closed / reopened / assigned / milestoned / label_updated）与 `issue_comment`（created / edited）都投递到本机监听器，每条带 `X-Gitea-Delivery`、`X-Gitea-Event`、`X-Gitea-Signature` 与 `X-Hub-Signature-256` | 待核项关闭：签入才可见不影响唤醒；控制面校验 `X-Hub-Signature-256` 即可，与 GitHub 同一套 |
| webhook 目标在本机 | `[webhook] ALLOWED_HOST_LIST` 缺省 `external`，回环地址会被拒；设成 `loopback` 后才投递。建 hook 时不校验目标（指向不可达地址也回 201），失败只在投递时发生；REST 没有投递记录接口（`…/hooks/{id}/deliveries` 404），只有 `POST …/hooks/{id}/tests` | 随包 `app.ini` 加 `ALLOWED_HOST_LIST = loopback`；投递失败靠轮询兜底，不靠回查 |
| `http+unix` | `PROTOCOL = http+unix` 加 `HTTP_ADDR = <套接字路径>` 可用：curl `--unix-socket` 读写 issues、409 冲突、webhook 投递都与 TCP 一致；套接字路径受 macOS 104 字节上限，超长路径下进程起来但套接字不出现、日志无错。tea 0.15.1 只认 `http(s)://`：`http+unix://` 报 `unsupported protocol scheme`，`unix://` 与 `http://unix:…` 被当主机名解析失败 | 待核项关闭：本地平台监听回环端口（09-15 已定）；套接字形态只对不经 tea 的第一方调用可选，且路径要短 |
| tea 子命令 | `issues list -o json --fields …` 可用，字段全集 `index,state,kind,author,author-id,url,title,body,created,updated,deadline,assignees,milestone,labels,comments,owner,repo`（`updated` 为 UTC，`labels` 与 `assignees` 是逗号串）；`issues create / edit / close / reopen` 与 `comment` 不理会 `-o json`，只出人读文本；`milestones list`、`labels list` 的 `-o json` 可用；`tea api -X POST/PATCH -d '<json>' <path>` 原样透传，回原始 JSON，409 也原样回 | 建卡、改卡、评论的结构化通路只走 `tea api`；`issues list` 的 JSON 里没有实体键 `id` 与 `content_version`，只够做投影，身份与锁仍要 `tea api` 读 |
| 看板与依赖 | `…/projects`、`/user/projects` 404，与源码结论一致；`…/issues/{index}/dependencies` 增删查可用 | 看板位置「无」成立；Task 依赖若要投影，源原生依赖可读 |
| Forgejo（只对照 swagger，未运行） | 发布页（v16.0.4、v15.0.8）的预编译服务器二进制只有 linux，另有源码包；forgejo 分支 swagger（提交 `3b7f449d`）的 `EditIssueOption` 与 `Issue` 都没有 `content_version`，`EditIssueOption` 与 `EditIssueCommentOption` 各多一个 `updated_at`——那是设置更新时间的字段，不是预期版本比较；issues 列表多 `sort` 参数；projects 与 hook deliveries 同样没有；依赖与 `/repos/issues/search` 有 | 备选不是零成本：Forgejo 未验证服务端条件写入，绑定能力声明按平台探测，条件写入声明「无」；时间戳只作漂移检查、以回读为准，不宣称并发保证；真要验须在 linux 上跑 |

决定不变：Gitea 1.27.3 与 tea 0.15.1 随包，结构化写走 `tea api`。随包配置补两条：`ALLOWED_HOST_LIST = loopback`；不用套接字形态。GitHub Issues 侧的写侧验证同日在私有沙箱完成，见 `sdk/github.md` 复核记录；GitHub 的 issues 权限与 Projects V2 权限分开，`project` scope 只管后者。

## 2026-09-21 · 源码伴随包锁定复核

决定建议：维持 Gitea 1.27.3 与 tea 0.15.1 的官方二进制；补齐同版本上游源码归档，沿用 Buck `http_file` 的 SHA-256 校验和现有源码伴随包，不自行重打上游归档。两项都标 `reproducibility`，表示保留源码供复核，不表示已经验证源码能逐字节重建随包二进制。

| 组件 | 锁定归档 | Release tag 对应 commit | 实测 SHA-256 |
| --- | --- | --- | --- |
| Gitea 1.27.3 | [gitea-src-1.27.3.tar.gz](https://dl.gitea.com/gitea/1.27.3/gitea-src-1.27.3.tar.gz) | `146cc3eec57174711eac0e0a0c7b38670c6e3922` | `3283ae40dd1f7b09450bb5a56455e78106fe17f4211d254c7c0179b8927bf382` |
| tea 0.15.1 | [v0.15.1.tar.gz](https://gitea.com/gitea/tea/archive/v0.15.1.tar.gz)，包内命名 `tea-0.15.1-source.tar.gz` | `f34697c5ed65928e265d6f48e16928819ce0f332` | `e242dd3589c31a36320d75e0de9eefa3fa429bd9b0af89d35af8585c7f514b9c` |

Gitea 归档为 [v1.27.3 Release](https://github.com/go-gitea/gitea/releases/tag/v1.27.3) 的源码资产；下载摘要与[上游 `.sha256`](https://dl.gitea.com/gitea/1.27.3/gitea-src-1.27.3.tar.gz.sha256) 一致，归档内 `VERSION` 为 `1.27.3`。tea 使用 [Release API](https://gitea.com/api/v1/repos/gitea/tea/releases/tags/v0.15.1) 的 `tarball_url`；其[发布校验清单](https://dl.gitea.com/tea/0.15.1/checksums.txt)未列 tag 源码归档，因此上表是本次下载实算的摘要，不冒充上游签名校验。commit 分别核自 [Gitea tag 的 commit](https://api.github.com/repos/go-gitea/gitea/commits/v1.27.3) 与 [tea tag API](https://gitea.com/api/v1/repos/gitea/tea/tags/v0.15.1)。两份归档的根 `LICENSE` 都是 MIT；原文随归档保留。

落地范围：`lock.json` 的 `common.gitea_source` / `common.tea_source` 与组件源码元数据；`sources.tsv`、源码伴随包及 `dependencies.tsv` 的源码列。运行二进制、下载格式、服务配置与启停行为不变。

## 2026-09-21 · Repo 注册调用复核

决定建议：仍用 Gitea 1.27.3、tea 0.15.1 的锁定二进制与原生管理 CLI，不引入 HTTP SDK。控制面只在注册消费本地平台时启动 Gitea、等就绪、物化账号与令牌；外部 GitHub clone 不走这条路径。

本次对照上节锁定源码归档：tea 的 `cmd/api.go`、`modules/context/context_login.go`、`modules/context/context.go`；Gitea 的 `cmd/admin_user_generate_access_token.go`、`routers/api/v1/api.go`。原生管理命令用法另核[官方 CLI 文档](https://docs.gitea.com/administration/command-line/)。

| 观察 | 实现决定 |
| --- | --- |
| tea `api --include` 的 JSON 正文在 stdout、HTTP 状态与响应头在 stderr；HTTP 403 仍退出 0，本机实际建仓已遇到 | 校验 HTTP 2xx，再解析平台身份与关联；进程退出 0 不等于成功 |
| `GITEA_INSTANCE_URL`、`GITEA_TOKEN` 支持每次调用的登录；无需写 tea 全局配置 | 令牌来自既有 SecretStore，经子进程环境传入，不放 argv、URL 或治理材料 |
| Gitea `/user/repos` 同时经过 user 与 repository scope 检查；只有 `read:user` 时 POST 返回 403，改成 `write:user` 后真实建仓通过 | 管理账号的本包令牌用 `write:user,write:repository,write:issue`；不误认为 repository scope 单独足够 |
| 原生 `generate-access-token --raw` 可读出令牌；同名 token 再建被拒绝，旧明文不能重新取回 | 固定 token 名；持久化失败后报凭据不可用，由人恢复或明确撤销失落 token，不静默制造多份 |
| 实际 macOS arm64 完整包：start 仅带 Tuwunel，注册本地仓库后 Gitea 可用、建仓并只推 main、稳定 ID 确认后激活；stop/start 后重复命令仍返回同一 Repo | 接入已有 Supervisor；注册与初始交付分阶段回读，pending 不冒充 active |

未知建仓结果的适配器失败注入使用子进程 fixture：模拟 POST 已生效但返回 HTTP 503；重试按原名称及注册关联读取，POST 计数仍为一次。它检验恢复分支，不冒充真实网络故障实验。没有重做 Gitea 服务、账户系统或 Git 协议。

## 2026-09-21 · 任务源写入条件的范围复核

决定建议：仍采用 Gitea 1.27.3 / tea 0.15.1；修正 09-17「title 与 body 都受它保护」的宽泛说法。`content_version` 的数据库比较并更新只覆盖正文；标题、状态等字段有入口的旧版本检查，不具备整次 PATCH 的原子条件写入保证。任务源能力声明同时写出这个范围，每次 issue 编辑仍带已读到的版本，并回读目标字段；未知不报成功。

钉定源码：[issue API](https://github.com/go-gitea/gitea/blob/146cc3eec57174711eac0e0a0c7b38670c6e3922/routers/api/v1/repo/issue.go) 的 `EditIssue` 先检查 `ContentVersion`，随后分别调用 `ChangeTitle`、`ChangeContent`；源码明确留有将全部修改包进事务的待办。不能据入口检查推导多字段事务或标题并发保证。

本包原生 Buck 测试用锁定的官方二进制在私有回环仓库验证：标题写入后 `content_version` 不变；正文写入才推进版本；再携带旧值修改标题返回 409。依赖投影分别读 `dependencies`（阻塞本卡）与 `blocks`（本卡阻塞的卡）；同版本 API 没有父子接口，两个父子字段为空，不从现有阻塞关系编造父子。09-17 的任务后端总览「四家都有父子」不能作为 Gitea 父子能力声明的依据。

评论读取也不套通用分页：[`ListIssueComments`](https://github.com/go-gitea/gitea/blob/146cc3eec57174711eac0e0a0c7b38670c6e3922/routers/api/v1/repo/issue_comment.go) 返回该 Issue 的全部评论，只有 since/before 过滤，没有 page/limit；通用的「读下一页直到空」会重复读同一批。适配器按此接口单次读取，并保留进程输出大小上限；其他分页接口重复返回同一页时报告不完整，不报告空板。

## 2026-09-28 · #288 评审后的轮询与拒绝结果复核

决定建议：版本仍钉 Gitea 1.27.3 / tea 0.15.1；普通对账用 `since` 加分页，只补读变化卡片的依赖与评论，另做周期完整核对，处理删除及不保证推进更新时间的关系变化。具体卡片的命令只回读该卡，显式刷新保留完整读取。后台连接复用已保存的地址与凭据，不消费服务或引导账号。

本机原生 Buck 测试中，间隔超过一秒后为卡添加阻塞依赖，该卡 `updated_at` 从 `2026-09-28T02:48:55+08:00` 变为 `2026-09-28T02:48:57+08:00`；不能假定 API 时间都以 Z 结尾。测试同时验证带时区的 `since`、评论回读与 409 状态码。这只证明所测添加动作，不推导全部依赖编辑、删除都更新两端时间；周期完整核对仍保留。

拒绝结果不按「所有 4xx」归类：401/403/404 的拒绝响应及单独正文条件更新的 409 可保留原意图、发送前精确回读与平台响应，结束为失败；多字段 PATCH 的 409 仍可能已改过标题，不证明整体未生效。超时、5xx 与未核实的错误只按原目标回读。子进程夹具覆盖明确拒绝、可能部分写入与确认丢失，不能把夹具结果说成真实并发故障实验。

## 复核记录

### 2026-10-07 · 合并与保护条件回读的实测（第 6 包集成一半）

对着随包的 Gitea 1.27.3（演示 2 的私有实例）用随包 `tea api` 实测，结果进了绑定的能力声明（`remote_merge`、`protection_readback` 对本地平台声明为真）与 control 的适配器：

- `GET repos/{o}/{r}/branches/main` → 200，`commit.id` 是分支头；`GET repos/{o}/{r}/branch_protections/main` 在未保护时 404（适配器记为「未保护」快照），保护字段按 1.27 的 `BranchProtection` 读：`enable_push`、`enable_status_check` + `status_check_contexts`、`required_approvals`、`block_on_outdated_branch`。
- `POST repos/{o}/{r}/pulls/{index}/merge` 带 `head_commit_id`：头不符 → **409 `head out of date`**（源头匹配生效，什么都没合）；刚建的评审请求在可合性检查完成前 → **405 `Please try again later`**（不是终态，稍后同一请求可再试）；检查完成后 `Do: fast-forward-only` → **200**，回读 `merged: true`、`merge_commit_sha` 等于候选提交、分支头等于候选提交；再合一次 → 405 `The PR is already merged`。
- **REST 接口不给树 ID**：`GET git/commits/{sha}` 的 `commit.tree.sha` 与 `GET git/trees/{sha}` 的 `sha` 都回传提交 ID。所以平台目标的 Receipt 只在快进（合并提交就是候选）时记准入的结果树，合并提交的树记为未读。
- 评审线程、正式评审、评论正文回读仍未实测，声明照旧为假。

**同日补（#390 评审后）· 保护条件要按分支读生效规则，不按分支名查规则。** 对同一实例再测（受保护分支的证据，上面只测了未保护的 404）：
- 未保护时 `GET branches/main` → `protected: false`、`effective_branch_protection_name: ""`。
- `POST branch_protections` 建一条通配规则 `rule_name: "ma*"`（`enable_push: false`、`enable_status_check: true` + `["canary"]`、`required_approvals: 1`、`block_on_outdated_branch: true`、`block_admin_merge_override: true`）→ 201，返回的规则共 36 个字段（含 `approvals_whitelist_*`、`merge_whitelist_*`、`bypass_allowlist_*`、`enable_force_push*`、`protected_file_patterns`、`priority`、`created_at`、`updated_at` 等）。
- 规则生效后 `GET branches/main` → `protected: true`、`effective_branch_protection_name: "ma*"`、`required_approvals: 1`、`status_check_contexts: ["canary"]`；**`GET branch_protections/main` → 404**（按名查规则，没有叫 `main` 的规则）；`GET branch_protections/ma%2A` → 200，`rule_name: "ma*"`、`branch_name: ""`、`priority: 1`。
- `DELETE branch_protections/ma%2A` → 204，`GET branches/main` 回到 `protected: false`。
- 结论进适配器：先读分支记录，受保护就按 `effective_branch_protection_name`（一段路径、百分号编码）读规则；受保护却读不到规则记 `PROTECTION_UNREAD`，不记未保护。快照冻结整条规则（除两个时间戳），不只挑几个字段。
- 这次实测没有再跑合并：`remote_merge` 的依据仍是上面那次快进合并；合并提交的树与「目标头是否承载合并提交」改由 `hctl2-tool readback` 从 Git 读，不再依赖 REST。

### 2026-10-08 · 关闭者归属与依赖门槛实测（第 7 包完成 Task）

**决定建议：** Gitea 上「谁把卡片推进终态」的来源改为 issue 时间线，不用 issue 载荷的 `closed_by`——该字段在 Gitea 不产出。适配器读卡时，卡片已关闭且载荷缺 `closed_by` 才补读一次时间线，只补这一个字段；供应端 Done 的归属判定（`closed_by` 与绑定声明的 `human_account` 比对）与能力声明不变。

实测（原生 Buck 测试，锁定的 Gitea 1.27.3 + tea 0.15.1，私有回环仓库）：

| 观察 | 对设计的意思 |
| --- | --- |
| `GET /repos/{o}/{r}/issues/{n}` 的载荷没有 `closed_by`；完整键集为 assets、assignee(s)、body、closed_at、comments、content_version、created_at、due_date、html_url、id、is_locked、labels、milestone、number、original_author(_id)、pin_order、projects、pull_request、ref、repository、state、time_estimate、title、updated_at、url、user，唯一的人字段 `user` 是作者 | 不能假定平台带得回关闭者；归属要另找来源 |
| 钉定 commit 的 `modules/convert/issue.go` 通篇没有 `ClosedBy` | 上述缺失是转换层行为，不是本次实例配置所致 |
| `GET /repos/{o}/{r}/issues/{n}/timeline` 的 `{"type":"close"}` 条目带 `user.login`；同一次「人用自己账号关闭」取回的就是该人 | 关闭者从时间线补读；只对已关闭且缺字段的卡发起，仍是条件读取 |
| 关闭仍有未关闭依赖的卡被拒，HTTP 412（`CloseIssue` 分支，`routers/api/v1/repo/issue.go`） | 412 与 404/409 语义不可互推；测试与运维要先解除阻塞依赖 |
| `PATCH …/issues/{n}` 只带 `{"state":"closed"}`、不带 `content_version`，在无未关闭依赖的卡上成功 | 与 09-17「不带该字段则无条件覆盖」一致；状态编辑不需要版本锁 |
| 以另一个账号的凭据走适配器写回被拒（`PLATFORM_CHANGED`，冻结绑定与客户端身份不一致） | 「人在平台上动手」只能用与该人凭据绑定的外带调用，不能用控制面账号的适配器代写 |
| 仅设 `GITEA_INSTANCE_URL`/`GITEA_TOKEN`、没有 tea 登录配置与本地远端时，`tea issues close` 不可用（`-repo` 不是其旗标） | 外带写回统一走 `tea api` 形态，与适配器同一条通路 |

以上均为所测动作的观察，不外推到其它实例配置或其它账号组合。

### 2026-10-11 · 平台评审评论进入真实返工 Context

决定建议：仍用 Gitea 1.27.3 与 tea 0.15.1 的原生 API，不引入 SDK。新建本地平台绑定可声明 `review_text_readback=true`；评审线程解决状态与正式批准的机械判定仍未在这次验证，不扩大那两项能力。已有冻结绑定不自动改写。

原生目标 `root//packaging/release:room-cli-test` 的 `review_comments_real_cli_freezes_gitea_content_without_authorizing_integration` 使用实际安装包、CLI、control 与脚本 Agency。首次写入调用经封存准入、发布 worker 推送与建请求产生真实 `changeset_platform_binding`，不用映射测试缝。随后用随包 tea 在同一请求创建一般评论、`COMMENT` 评审及一条行内评论，再从真实 `invocation preview | start | show` 走返工。

| 观察 | 对设计的意思 |
| --- | --- |
| `GET issues/{index}/comments`、`GET pulls/{index}/reviews`、`GET pulls/{index}/reviews/{review_id}/comments` 读回原生 ID、正文与提交关联；创建评审用 `POST pulls/{index}/reviews`，带 `event: COMMENT`、`commit_id` 与 `comments` | 普通评论、评审正文和行内评论都能作为精确 Context 来源；没有验证平台批准或线程解决资格 |
| Bundle 中的规范正文保留精确 Revision、平台映射、请求编号、平台提交及各评论 ID；来源摘要与实际交付字节一致 | 继续使用第 4 包的 Manifest / Bundle，不另造评论存储或客户端 |
| 预览后修改平台评论，启动仍接受原预览，`context show` 回读原 Bundle | 平台当前内容不覆盖已冻结的派工上下文 |
| 评论正文含「合入吧」，下一次脚本调用正常交回，控制面没有集成意图 | 评论是 content，不是授权 |

首次全组运行 Build ID `fb6a43d9-884b-48f1-97e2-e6de2587488f`：本条用例及另外九条通过；两条旧用例分别报 SQLite I/O 与 Git 材料不可用，全组不是绿。本条只记录上述实际通过的评论链，不用它覆盖两条旧用例的失败。GitHub 读取沿既有 gh API 适配，本次未对在线 GitHub 评论线实跑。

随后以 `--test-threads=1` 单列本条与两条失败用例补跑，Build ID `ec73ecef-749a-4eee-ba0b-a4338a034e0a`：三条通过。本条再次从真实发布走到返工 Context；两个旧用例的首次失败原因仍未确定，补跑通过不等于已解释或修复它们。

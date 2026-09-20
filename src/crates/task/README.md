# 任务源端口与 Task 影子（P2.2 庚）

实现依据：[开工书 §庚](../../../.memo/design/p2-control-20260906/05-p22-kickoff.md#庚--任务源端口与-task-影子codex)、[Task 约束](../../../docs/design/spec/task.md)、[CT-TASK](../../../docs/design/contract-tests.md#ct-task--task--kanban)。本文说明实现与验收边界，不定义新约束。

## 模块与接入

| 位置 | 职责 |
| --- | --- |
| `crates/task/src/model.rs` | 任务源、Project 源引用、Snapshot、Task、契约与命令输入 |
| `planning.rs` | 纯读取预览、作用范围与版本检查、冻结外部写入意图 |
| `commands.rs` | 复用 Store 命令内核；契约先保存材料，再同事务准入 Revision、记录与 outbox |
| `observations.rs` | 观测、获准映射的自动认领、写前复核、回读确认与按源看板 |
| `apps/control/src/tasks/` | Query / Preview / Submit、定期对账、gh / tea 原生调用；网络不占 Store 事务 |
| `apps/control/src/scm.rs` | 与 Repo 共用已注册平台身份、托管 Gitea 与凭据连接 |
| `apps/cli/src/task.rs` | 公共 `hctl2 task` 客户端，写命令先预览再确认 |

源候选及注册时的显式缺省选择直接读取 `repo::SourceCandidate` 与注册记录。`connect` 建立 `port_kind=task_source` 的绑定，`attach` 才把它接入指定 Project；仅查看源卡不算认领。每个 Project 可接多份源引用，同卡在不同 Project 得到独立 Task；规范实体与 Project 的唯一映射由 Store 现有索引守住。Task 的身份记录与状态记录同事务更新，没有新增数据库表或存储引擎。

本批只把已注册平台的 issues 候选变成绑定，注册目前每种平台返回一个候选；第二源引用的准入已可用并有领域测试，端到端接第二家任务服务器仍等 Vikunja / Linear 后续包。辛负责创建 Project：本包拒绝不存在、未激活 Repo 下或已归档的 Project。测试预置 Project，不为产品增加绕过辛的命令。

辛可读取 `task_state.pending_contract` 与 `needs_attention` 组装自己的投影；这里不实现 Request。P2.3 接 Run 时使用同 Project 的 `run` 记录（当前检查 `task_id`、`lifecycle`）及 Task 的 `run_occupancy`；未知 Run 形状按非终态处理。契约采纳不改 Run，取消检查活动 Run，删源卡只确认影响而不取消任一 Run。

## CLI

命令输入形状以 [`Action`](src/model.rs) 为准。JSON 不带 `kind` 也可，CLI 由子命令补入；多余字段拒绝。以下文件内容是示例，不是自动创建 Project 的入口。

```json
{"repo_id":"REPO_ID","candidate_id":"github_issues","consent":true,"make_default":true}
```

```sh
hctl2 task connect --input source.json --key source-confirmation
hctl2 task connect --input source.json --key source-confirmation --preview-token TOKEN
hctl2 task sources
```

已有 Project 接入源（`approved_scope` 取源的 `board_scope_stable_id`；没有原生分组就省略 `group`）：

```json
{"project_id":"PROJECT_ID","project_version":1,"source_id":"SOURCE_ID","approved_scope":"PLATFORM_REPO_ID","consent":true}
```

```sh
hctl2 task attach --input attach.json --key project-source
hctl2 task attach --input attach.json --key project-source --preview-token TOKEN
hctl2 task board PROJECT_ID SOURCE_ID
hctl2 task list --project-id PROJECT_ID
hctl2 task show PROJECT_ID TASK_ID
```

`claim` 接受卡的规范 `entity_id`；不要求原生分组。`create` 显式给 `project_id`、`project_version`、`source_id`、`title`、`body`，可带 `adoption`。不带契约也能建 Task，卡的 closed 不代表 Task 完成。`adopt` 保存带校验等级的契约正文及精确来源；本地来源不伪造 Backend Binding，后端来源检查 Snapshot 引用与 `state_version`。

`update` 支持标题、正文或追加评论；评论与其他字段分次提交。`move` 支持 issues 的 open / closed 阶段；两家此绑定均未声明看板排序能力，带 rank 或跨源相对位置返回类型化拒绝，不模拟本地排序。`cancel` 只取消并归档本 Task。`delete-card` 是另一次预览确认：列出本控制面全部同卡 Task、各自 Project / 生命周期 / 活动 Run；`active_run_choices` 表示逐一确认这些 Run 继续运行的后果，不授予停止它们的权限。

`refresh` 显式读取源；控制面每 60 秒对已接入的活跃源对账，不依赖公网 webhook。`set-active` 带源记录 `version` 停用或重新启用；保留既有实体映射。`resume` 带 `effect_id` 恢复原外部意图，不重新选择目标。写命令均沿用上述预览、相同输入加 token 的两步形状；重复原 key 返回原领域结果。

查询和观察型写入结果都是 stdout JSON；外部步骤错误保留已准入的 Task / effect，附 `error.code`、`recovery_action` 与实际 `effect_state`，CLI 非零退出。未就绪、拒绝、结果未知不报成功。

## 写回与恢复边界

GitHub 使用现有 gh 登录；Gitea 1.27.3 经随包 tea 0.15.1 的 `api`，复用 Repo 的托管实例与凭据。删除能力在接源时回读管理员权限；删除确认再次核当前仓库身份与权限，单个 404 不证明已删除。

两家写前回读、写后核目标和字段。Gitea 的 `content_version` 数据库条件更新只覆盖正文；标题 / 状态与正文混合 PATCH 不是原子事务，因此能力声明另列 `conditional_fields=["body"]`，不会把整次 PATCH 说成原子比较并更新。GitHub 不声明条件更新。具体证据见 [Gitea 复核](../../../docs/research/gitea.md#2026-09-21--任务源写入条件的范围复核)。

领域记录与 outbox 先同事务持久化，再执行外部步骤。创建与评论携带控制面、Task 和输入摘要的关联标记；结果未知只按原标记回读，不再次 POST。未知写入继续占冲突范围；卡片已被人改到不能证明原结果时，返回未知，不凭相似内容猜成功。调用者可显式重试原命令或 `resume`，后台轮询不重发写入。

创建已发出但尚未确认时，对账及人工认领都保留原 Task 的位置，不再生成第二个 Task。取消会撤销尚未发送的意图；未知意图保留以待回读。删除在准入和未发送意图恢复时重核受影响集合，新增绑定 Task / Run 会拒绝旧确认。

后端不可用时保留最后观测并标不完整，按源看板不会显示成空板；依赖当前回读的动作拒绝。源 stage、标题、分组或依赖变化不改 Task 归属、契约或生命周期；契约相关字段变化产生待采纳提示。停用源不阻止独立的本地契约采纳，后端来源采纳仍要求当前回读。依赖只存在 Snapshot 的 parent / children / blocked_by / blocking 四字段；Gitea 没有核实的父子接口时前两项为空，不另建依赖对象。

Task 外部调用与 Repo、服务维护、恢复共用现有运维锁；Store 仅在短事务期间占用，查询不等待网络。客户端断开后阻塞工作仍持锁。此锁不声称排除平台原生客户端或另一个控制面的写入。

## 验证与 CT 对照

| CT-TASK 现行条目（按内容对应，含本批只覆盖一半的行） | 失败输入 / 本批验证 |
| --- | --- |
| stage / health 不改 lifecycle 与 Revision；无契约终态仅投影 | `domain_test`：外部 closed、标题改变、分组漂移后仍开放且保留原契约；本批没有完成命令 |
| 非法 move；local state 与 remote revision 分开 | `domain_test`：过期版本、额外 lifecycle 字段、跨源相对位置、离线 / 过期 Snapshot 拒绝；原生 Gitea 测试：旧 content_version 返回 409 |
| 依赖四字段、源无能力不阻拦 | `domain_test`：四字段观测不生成依赖对象；`task_native_test`：真实 Gitea blocked_by / blocking 两向；GitHub 端点沿已完成写侧研究，未在个人远端重跑 |
| 本地 adoption 与外部 Binding / Snapshot 版本分开 | `domain_test`：本地来源无 Backend Binding、错 Project / 摘要 / Snapshot 版本拒绝、源停用后的本地采纳可通过 |
| 禁用绑定不释放映射、同 Project 重复认领不新增 | `domain_test`：停用保留 Task 并标关注，重新接通、重复认领仍同一 Task |
| 零到多源、显式同意、候选 / 缺省与接源分开 | `domain_test`：未同意与错误范围拒绝、第二个已有源引用可接；`task_cli_test`：复用 Repo 缺省候选、未创建 Project 不暗造对象。新的第二家 provider 不在本批 |
| 两路认领、只查看不认领、稳定获准分组与漂移 | `domain_test`：无 group 不自动认领、显式认领无 group 通过；稳定标签自动认领；分组消失 / 移动保留原 Task 与 Project，不写源上分组 |
| 按源看板、未认领卡可见、失败不是空板 | `domain_test`：未认领卡可见、失败 Snapshot 留最后卡片并标不完整，不强制汇总 |
| 同卡两 Project 独立认领与契约；A/B 观测写回 | `domain_test`：同卡两个 Task、A 采纳不改 B、同一标题观测进两边、不推进 lifecycle；provider 写回回读后刷新全部绑定。原生 Done 触发完成留 P2.4 |
| 取消与单独删卡、A/B 活动 Run 后果 | `domain_test`：未确认删除拒绝、漏 Run 处理选择拒绝、A 取消不删卡也不取消 B、B 活动 Run 拒绝取消；新绑定使旧删卡预览失效；原生测试：删除回读 tombstone |
| 验收项带等级 | `domain_test`：缺 grade 反序列化拒绝，空验收项拒绝；等级证据与凭证验收留 P2.4 |
| 正文保存 / 准入崩溃与建卡确认丢失 | `domain_test`：保存材料后重开 Store 同命令仅一 Revision；待确认创建不被重复认领、Pending 取消、Unknown 仅回读；`task_cli_test`：真实 daemon 重启后重复命令仅一次 POST |
| 后端无条件写入未确认不报成功 | `unit_test`：tea 退出码 0 但 HTTP 503、响应丢失后按精确标记回读只 POST 一次；原卡被转移至不同实体拒绝。`task_native_test`：建卡、条件冲突、标题与正文版本差异、评论幂等与删除 |
| Start / Complete、Run reducer、Vikunja Done、证据等级、候选交付与完成凭证各行 | 按开工书留 P2.3 / P2.4，不把本批领域测试记成这些行为已经交付 |

Buck 目标：`root//crates/task:{task,domain_test,clippy}`；`root//apps/control:{unit_test,task_native_test,boundary_test,services_test,clippy}`；`root//apps/cli:{task_cli_test,cli_test,clippy}`。根 `root//:clippy` 包含 Task。原生 Gitea 测试消费 lock.json 已有四平台制品，在私有临时目录运行回环服务；不下载另一套版本，不启动开发者已有实例。

CLI 测试运行真实 CLI 与 daemon，只有 gh 是子进程夹具；原生 Gitea 测试不使用该夹具。已有 `root//packaging/release:complete-test` 继续验证完整安装包、原生服务生命周期及 Repo 注册。Project 创建与全 B1 的两个主 Room 恢复验收由辛串起，不在这里假报完成。

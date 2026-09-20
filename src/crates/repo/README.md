# Repo 注册（P2.2 戊）

实现依据：[开工书 §戊](../../../.memo/design/p2-control-20260906/05-p22-kickoff.md#戊--repo-注册codex)、[Repo 约束](../../../docs/design/spec/repo.md#repo-注册)。这里说明当前实现，不重定义约束。

## 代码与后续接入

| 位置 | 职责 |
| --- | --- |
| `crates/repo/src/model.rs` | 输入、冻结预览、注册状态、与庚共用的 `SourceCandidate`；候选选择不等于已创建 Task 绑定 |
| `crates/repo/src/registry.rs` | 复用 `store` 的命令、版本比较与 outbox；`require_active` 给后续 Project / Task / Run 准入调用 |
| `crates/repo/src/git.rs` | 宿主 Git 的只读检查、独立副本、选定 ref 的首次交付与回读 |
| `apps/control/src/repositories/` | RPC 编排、随包 gh / tea / Gitea CLI；事务外运行外部步骤 |
| `apps/cli/src/repo.rs` | `hctl2 repo register / list / show`，经控制面提交，不直接改存储 |

己、庚、辛分别建自己的模块，不往 `repositories/` 加 Room、Task、Project 命令。庚读取已确认的候选、显式选择与平台 Binding 后建立 `task_source` 绑定；辛调用 `require_active` 后关联 Repo。这里没有实现它们的业务命令或平台换绑。

注册 ID 来自控制面身份和命令幂等键，不从目录、URL 或提交内容算身份。同一命令只产生同一登记；不同的显式登记不按内容去重。

## CLI

输入是 JSON。下例在运行控制面的机器上检查已有目录；`machine: "control"` 是明确选择该机器，不表示路径可跨机器访问。其他机器目前返回 `MACHINE_UNREACHABLE`，不当纯本地仓库。

```json
{
  "name": "Apollo",
  "origin": "local",
  "platform_path": "apollo",
  "local": {"machine": "control", "path": "/absolute/path/to/apollo"},
  "default_source": "gitea_issues"
}
```

```sh
hctl2 repo register --input register.json --key apollo-registration
# 阅读 effect_summary；用刚返回的 preview_token 提交相同输入。
hctl2 repo register --input register.json --key apollo-registration --preview-token TOKEN
hctl2 repo list
hctl2 repo show REPO_ID
```

无 remote 的已读目录省略 `platform` 时选本地 Gitea；有 remote 时需要显式选外部来源，或 `origin: "independent", platform: "local"`。独立工作默认在控制面私有目录建立新副本，原目录及 remote 不变；`local.in_place: true` 明确选择原地改 remote，预览冻结原 remote，本实现只接受恰好一个 remote。

外部 GitHub 输入用 `origin: "external", platform: "github", instance: "github.com", platform_repo_id: "数字 ID", platform_path: "owner/name"`。URL 放 `remote_evidence`，只是辅助证据；冲突进预览。gh 回读平台稳定 ID 和账号；即使输入含本机 clone，也不消费 Gitea。gh 使用已有登录，不复制令牌。

显式 `platform: "none"` 不建平台、不因 URL 改判；不附 `instance`、`platform_repo_id`、`platform_path`。本地任务服务器当前未接入，候选标未可用，不假装已绑定。未选 `default_source` 就只返回候选；GitHub / Gitea 的实际 issues 能力还要经平台回读才能确认。

本地 Gitea 的稳定 ID 由建仓分配，所以第一次提交先待确认。建仓与初始交付回读后，读取返回的 `version`、`observed.stable_id`，人再确认该身份：

```sh
hctl2 repo register --confirm REPO_ID --version VERSION --platform-repo-id ID --key apollo-confirm
hctl2 repo register --confirm REPO_ID --version VERSION --platform-repo-id ID --key apollo-confirm --preview-token TOKEN
# 中途失败：同一注册输入、同一 key 重试，或预览后继续原登记。
hctl2 repo register --resume REPO_ID --key apollo-resume
# 放弃：同样先预览，再带 token 提交；平台残留不自动删除。
hctl2 repo register --abandon REPO_ID --version VERSION --key apollo-abandon
```

所有 Repo 写命令均先预览。预览冻结输入目录的已提交 HEAD、选定 refs、remote 和可达治理路径；提交前变化则重做预览。初始交付不包含未提交内容，默认只推 HEAD 分支；detached HEAD 使用新平台默认分支 `main`。其他 refs 用 `local.extra_refs` 显式列出。私有 refs 不推；选中历史包含 `.memo` / `.hctl2` 时拒绝，只有人明确将该历史公开（`local.publish_governance: true`，路径在预览中列出）才继续，不自动改写历史。此路径检查不声称能识别任意文件里的敏感正文。

## 失败与恢复

先持久化注册、材料与平台意图，再调用外部平台。Gitea 通过 `Supervisor::consume` 按需启动并等就绪；原生管理 CLI 建控制面账号和令牌，令牌进既有 SecretStore。固定令牌名已存在而 SecretStore 丢失时拒绝继续：恢复原令牌，或由人明确撤销失落令牌后重试，不自动累加令牌。

建仓用冻结的账号、仓库名和注册关联标记；发送前先读、发送后再读。响应丢失后保持原 outbox 未知，重试只查原目标；同名但关联不符拒绝。Git 交付仅创建不存在的选定 refs，空旧值的 `--force-with-lease` 做创建比较，已有不同值拒绝；回读完整集合才能确认。已交付而确认丢失可只回读，不要求原输入目录仍在。

当前由显式重试驱动恢复，不在后台静默重新授权。放弃保留原目标和已读到的平台仓库，未知 outbox 不伪装已完成。原地 remote 更改失败也不回滚已确认的平台交付。

Repo 外部步骤与存储恢复、服务维护串行；SQLite 只在短事务期间占用。客户端断开不提前释放执行锁；Query 与普通命令不等外部步骤。这是当前单机编排方式，不是新的跨控制面锁。

## 验证与 CT 对照

| CT-REPO 现行条目 / 相邻准入条目 | 本批失败输入与验证落点 |
| --- | --- |
| 同一注册命令重投；不同显式登记不按内容合并 | `registration_test`：重投、重启、同 key 改参数拒绝；`boundary_test`：并发两次只得一个 Repo |
| 平台稳定标识缺失，URL 不替代，证据冲突供人确认 | `registration_test`：缺 ID、ID 不匹配、冲突预览；`cli_test`：错误 GitHub ID 不激活 |
| 外部仓库不绑定本地镜像；显式不挂不改判 | `registration_test`：external + local 拒绝、none 保留；`cli_test`：GitHub 和 none 不消费 Gitea。平台换绑命令留后续包 |
| 指定机器与本地读取；remote 必须选；独立副本不改输入 | `git_test`：不可读、错机器、有 remote 未选、原地前置、remote 漂移、原目录与未跟踪文件保留 |
| 纯本地缺省；HEAD / 空仓 / 其他 refs / 治理材料 | `git_test`：空仓、detached HEAD、未选分支与私有 refs、附注标签、历史已删治理文件；完整安装包测试：实际 Gitea 建仓并只推 main，无需配置输入 remote |
| 同名分支不覆盖、未知只回读、不可用不降级、放弃留残留 | `git_test`：不同目标 SHA 拒绝与重复交付；`registration_test`：未知意图、放弃残留、提前确认拒绝；`boundary_test`：不可用重试同一 pending；平台适配器测试核 HTTP 失败和关联回读 |
| 待确认不接受 Project / Task / Run；注册不建 Room | `registration_test`：`require_active` 拒绝、无 Room 记录。后续业务入口尚不存在，不把此守卫测试冒充完整业务验收 |
| 本地平台检查只声明外部状态写回 | `registration_test` 核 Binding 的 `external_status_only`，不冒充机械证据 |

Buck 目标：`root//crates/repo:repo`、`:registration_test`、`:git_test`、`:clippy`；复用 `root//apps/control:{unit_test,boundary_test,services_test}`、`root//apps/cli:cli_test`、`root//packaging/release:complete-test`。其余 CT-REPO（写租约、ChangeSet、发布评审、集成、审计公开）不在戊的交付范围。

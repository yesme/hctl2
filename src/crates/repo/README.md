# Repo 注册（P2.2 戊）

实现依据：[开工书 §戊](../../../.memo/design/p2-control-20260906/05-p22-kickoff.md#戊--repo-注册codex)、[Repo 约束](../../../docs/design/spec/repo.md#repo-注册)。这里说明当前实现，不重定义约束。

## 代码与后续接入

| 位置 | 职责 |
| --- | --- |
| `crates/repo/src/model.rs` | 输入、冻结预览、注册状态、与庚共用的 `SourceCandidate`；候选选择不等于已创建 Task 绑定 |
| `crates/repo/src/registry.rs` | 复用 `store` 的命令、版本比较与 outbox；`require_active` 给后续 Project / Task / Run 准入调用 |
| `crates/repo/src/git.rs` | 宿主 Git 的只读检查、独立副本、选定 ref 的首次交付与回读 |
| `apps/control/src/repositories/` | RPC 编排、随包 gh / tea / Gitea CLI；事务外运行外部步骤 |
| `apps/control/src/scm.rs` | Repo 与 Task 共用的平台连接、GitHub 身份回读及 Gitea 凭据；本地平台上人的普通账号与协作者授权；不承载 Task 命令 |
| `apps/cli/src/repo.rs` | `hctl2 repo register / grant / list / show`，经控制面提交，不直接改存储 |

己、庚、辛分别建自己的模块，不往 `repositories/` 加 Room、Task、Project 命令。庚读取已确认的候选、显式选择与平台 Binding 后建立 `task_source` 绑定；辛调用 `require_active` 后关联 Repo。这里没有实现它们的业务命令或平台换绑。`prepared.sources` 是冻结预览，`sources` 是当前观测；后者随 Issues 能力变化刷新，不改冻结输入。目前每种平台只返回一个候选，候选 ID 暂取 provider 名，`recommended` 为真；庚增加多源选择时可扩展，推荐本身不等于已绑定。`repo list` 跳过 P2.1 尚无注册内容的占位记录。

庚的接入与失败用例见 [Task 实现说明](../task/README.md)；复用本包的 `SourceCandidate` 和已确认的 `default_source`，没有另一份候选协议。

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
hctl2 repo register --resume REPO_ID --key apollo-resume --preview-token TOKEN
# 放弃：同样先预览，再带 token 提交；平台残留不自动删除。
hctl2 repo register --abandon REPO_ID --version VERSION --key apollo-abandon
```

本地平台上有权的人各有自己的普通账号，由 control 建；管理员账号只给 control，不冒充人的身份：

```sh
hctl2 repo grant --repo-id REPO_ID --username alice --key alice-grant
# 阅读 effect_summary；用刚返回的 preview_token 提交相同输入。
hctl2 repo grant --repo-id REPO_ID --username alice --key alice-grant --preview-token TOKEN
```

`--permission` 取 `read`（缺省）或 `write`，`read` 已足够让人在平台上开 Issue；`admin` 一类的级别不接受。账号已存在就复用、不重置其口令；协作权每次都写请求的那个级别再回读确认，因为平台的协作者回读是 204 空正文、读不出当前级别。返回 `{repo_id, full_name, username, account_created, initial_password?, permission}`：`initial_password` 只在本次确实建了账号时出现一次，控制面不存它、不写日志、不进事件流，人自己保管；Gitea 的个人账号默认须改口令，所以它是初始口令而不是长期密钥。只对已激活（`lifecycle: active`）的本地平台注册有效：外部 GitHub 注册拒绝（`INVALID_INPUT`，账号归 GitHub 自己管），待确认注册拒绝（`REPO_PENDING`）。用户名会被拼进平台 API 路径，所以先在控制面按字符集与 `.` 规则校验，不靠平台事后拒绝。

所有 Repo 写命令均先预览。预览冻结输入目录的已提交 HEAD、选定 refs、remote 和可达治理路径；提交前变化则重做预览。初始交付不包含未提交内容，默认只推 HEAD 分支；detached HEAD 使用新平台默认分支 `main`，但若输入已有不同 SHA 的 `main`，拒绝猜测：先检出目标分支，或用 `local.extra_refs` 明确选择原 `refs/heads/main`。其他 refs 用 `local.extra_refs` 显式列出。私有 refs 不推；选中历史包含 `.memo` / `.hctl2` 时拒绝，只有人明确将该历史公开（`local.publish_governance: true`，路径在预览中列出）才继续，不自动改写历史。此路径检查不声称能识别任意文件里的敏感正文。

`register` / `resume` 返回 `{registration, error?}`：记录已准入后，外部步骤失败也返回该记录及结构化错误，CLI 以非零退出；准入或预览被拒则只有错误、没有新登记。`confirm` / `abandon` 与 `show` 直接返回注册记录，`list` 返回 `{items}`。各形状沿既有命令区分，不把外部错误藏成成功。

## 失败与恢复

先持久化注册、材料与平台意图，再调用外部平台。Gitea 通过 `Supervisor::consume` 按需启动并等就绪；原生管理 CLI 建控制面账号和令牌，令牌进既有 SecretStore。固定令牌名已存在而 SecretStore 丢失时拒绝继续：恢复原令牌，或由人明确撤销失落令牌后重试，不自动累加令牌。

建仓用冻结的账号、仓库名和注册关联标记；发送前先读、发送后再读。响应丢失后保持原 outbox 未知，重试只查原目标、不再次 POST；同名但关联不符拒绝。Git 交付仅创建不存在的选定 refs，空旧值的 `--force-with-lease` 做创建比较，已有不同值拒绝；回读完整集合才能确认。已交付而确认丢失可只回读，不要求原输入目录仍在。所有带平台凭据的推送都从控制面私有副本执行；不使用输入仓库的配置、hooks 或模板，禁用继承的全局配置与 Git trace。原地选项只在成功后无凭据地改原 remote。

当前由显式重试驱动恢复，不在后台静默重新授权；每次恢复核当前调用者、原授权及冻结材料。平台服务实例、稳定 ID、仓库全名、clone URL 用于核身份，Issues 能力与凭据观测变化不视为换仓。Gitea 尚未验证的评审、合并、保护回读能力声明为不支持；两家均未建立 human 到平台账号的映射，管理员凭据账号不冒充该映射。`repo grant` 建了人的普通账号与协作权，但约束要求的「账号映射照常写进绑定」还没做：绑定里没有这条映射的持久记录，命令重跑的幂等来自平台侧——账号已在就复用，协作权写请求的级别（对同一级别幂等）再回读。

`repo grant` 的外部步骤：账号已在原生 `admin user list` 里就复用（表头行不算账号），协作权**总是** PUT 请求的那个级别、再 GET 回读；PUT 之后回读不到授权按 `PLATFORM_READBACK` 拒绝并保留结果未知，不当成功。回读不能替代写、也不能用来跳过写：平台的 GET 对协作者回 204 空正文，正文里没有权限级别，「人在名单上」推不出「级别已是请求值」；写对同一级别幂等，对另一级别改到请求值。建账号的原生 CLI 自身失败按 `PLATFORM_BOOTSTRAP` 拒绝——Gitea 先打印生成的口令、再建用户，所以那行口令只在命令整体成功后才可信，失败时既不返回它也不把它写进错误正文。账号建成而授权失败时账号留下来、刚打印的初始口令被丢弃而不进错误正文，错误正文说明这一点；恢复是用原生管理 CLI 重置口令后重跑同一命令，账号已存在即复用。这条命令不写控制面存储：没有新的持久记录，也没有 outbox 阶段。

放弃保留原目标和已读到的平台仓库；尚未发送的 outbox 同事务标 `cancelled`、释放目标，不伪装外部成功。结果未知仍占用冲突范围。放弃后 `resume` 只回读：确认原仓库残留（交付未知还要核原 refs）后记录事实并释放已确认动作的范围，不建仓、不推送、不激活；查不到不能证明旧请求不会晚到。新登记遇冲突返回 `REGISTRATION_TARGET_BUSY` 和原登记 ID；已存在的残留仍须人清理，不会被新登记接管。原地 remote 更改失败也不回滚已确认的平台交付。

Repo 外部步骤与 `restore.apply`、服务维护串行；Gitea 就绪等待最多 30 秒，期间占此运维锁；SQLite 只在短事务期间占用。客户端断开不提前释放执行锁；Query、普通命令与治理备份不等外部步骤。这是当前单机编排方式，不是新的跨控制面锁。服务数据备份仍按丁的停服流程，治理备份不含平台数据。

## 验证与 CT 对照

| CT-REPO 现行条目 / 相邻准入条目 | 本批失败输入与验证落点 |
| --- | --- |
| 同一注册命令重投；不同显式登记不按内容合并 | `registration_test`：重投、重启、同 key 改参数拒绝；`boundary_test`：并发两次只得一个 Repo |
| 平台稳定标识缺失，URL 不替代，证据冲突供人确认 | `registration_test`：缺 ID、ID 不匹配、冲突预览；`cli_test`：错误 GitHub ID 不激活 |
| 外部仓库不绑定本地镜像；显式不挂不改判 | `registration_test`：external + local 拒绝、none 保留；`cli_test`：GitHub 和 none 不消费 Gitea。平台换绑命令留后续包 |
| 指定机器与本地读取；remote 必须选；独立副本不改输入 | `git_test`：不可读、错机器、有 remote 未选、原地前置、remote 漂移、原目录与未跟踪文件保留；恶意 hooks、URL rewrite、全局配置与 trace 不进入凭据调用 |
| 纯本地缺省；HEAD / 空仓 / 其他 refs / 治理材料 | `git_test`：空仓、detached HEAD 歧义与显式选择、未选分支与私有 refs、附注标签、历史已删治理文件；完整安装包测试：实际 Gitea 有提交 / 空仓建仓，不跑用户 hook、只推 main，无需配置输入 remote |
| 同名分支不覆盖、未知只回读、不可用不降级、放弃留残留 | `git_test`：不同目标 SHA 拒绝与重复交付；`registration_test`：未知意图、放弃后回读残留、取消未发送意图后重新登记、能力变而身份不变、恢复时授权与输入重核；`boundary_test`：不可用重试同一 pending；`cli_test`：放弃后 resume 不重启外部步骤、新 key 复用目标；平台适配器测试核 HTTP 失败和关联回读 |
| 待确认不接受 Project / Task / Run；注册不建 Room | `registration_test`：`require_active` 拒绝、无 Room 记录。后续业务入口尚不存在，不把此守卫测试冒充完整业务验收 |
| 本地平台检查只声明外部状态写回 | `registration_test` 核 Binding 的 `external_status_only`，不冒充机械证据 |
| 本地平台账号：管理员给 control，有权的人各一个普通账号（[Repo 约束 §平台绑定与能力声明](../../../docs/design/spec/repo.md#平台绑定与能力声明)） | 平台适配器测试：建账号失败不把已打印的口令当账号、已存在的账号复用且不再建、`admin user list` 的表头行不当成账号、协作者授权每次都写请求的级别并由回读确认（含已是协作者时改级别）、响应丢失按结果未知；`unit_test`：会改写平台路径的用户名与越权 permission 在接触平台前拒绝；`cli_test`：外部注册与待确认注册都拿不到 preview_token。绑定里的账号映射未实现，见「失败与恢复」 |

Buck 目标：`root//crates/repo:repo`、`:registration_test`、`:git_test`、`:clippy`；复用 `root//apps/control:{unit_test,boundary_test,services_test}`、`root//apps/cli:cli_test`、`root//packaging/release:complete-test`。其余 CT-REPO（写租约、ChangeSet、发布评审、集成、审计公开）不在戊的交付范围。

## 第 6 包 · 集成一半（意图、两种授权形态、Receipt）

实现依据：[开工书 §四 第 6 包](../../../.memo/design/p2-control-20260906/07-demo-kickoff.md) 验收第 5–7、9（集成半边）、11、12 条；[Repo 约束 §集成](../../../docs/design/spec/repo.md#集成目标两个头与两种授权形态)、§恢复。代码在 `src/integration.rs`；control 的编排在 `apps/control/src/integration.rs`；CLI 是 `hctl2 integration preview | submit | show | list`。

**对象。** `integration_intent`（Repo 范围）记一次「合入 ChangeSet Revision」的持久授权：冻结的预览（源版本的五个身份字段、目标、所选形态、策略、预期目标头或预览时看到的头、平台目标的保护快照、绑定版本）、状态（`pending` 未尝试 / `unknown` 已尝试未确认 / `succeeded` / `failed`）、尝试次数、留给人的 `attention`、终态原因 `failure`、`receipt_id`。`integration_receipt` 是唯一凭证：源、目标、形态、策略、执行前后的目标头、集成提交与树、回读的证据通道（本地目标是 `hctl2-tool`）与回读原文；它只在回读到结果之后、与效果确认和终态同一事务写入。两种记录都是 `RecordData::Value`，每个意图一条 outbox 效果，冲突范围是目标本身（`kind:provider_ref:ref`），所以同一目标同时至多一个待决意图（CT-REPO 第 13 行），由 Store 的 `EFFECT_CONFLICT` 保证、以 `TARGET_BUSY` 报出。

**源版本。** 集成只引用已准入的 ChangeSet Revision（记录 `changeset_revision`，另一半 `changeset.rs` 写入；本模块只读 `AdmittedRevision`：五个身份字段加 `producer_ref`、`review_subject_digest`）。版本里没有提交对象：执行时在目标仓库里找一个树正好是 `result_tree_sha`、父提交含 `base_commit_sha` 的提交（执行体自己的提交），找不到就用固定身份 `commit-tree` 包一个，重试得到同一个对象。`result_commit_sha` 只出现在 Receipt 与平台证据里，和约束一致。另一半合入之前，用例用 `admit_revision_seam` 写同一种记录，它不是领域准入；#384 合入后改走 `changeset::admit` 并删掉。

**形态。** `expected_head` 冻结预览时的目标头，执行时不等就 `failed`、不重试；本地目标总能选它，平台目标只有绑定声明 `expected_target_head` 为真才能选，否则 `EXPECTED_HEAD_UNSUPPORTED`，不在执行时降级（CT-REPO 第 17 行）。`accept_advance` 冻结源、策略与保护快照，接受目标前移，Receipt 记实际目标头。平台目标还要求绑定声明 `remote_merge` 与 `protection_readback`，缺一条 `CAPABILITY_MISSING`；随包 Gitea 这两项现在声明为未验证，所以本批平台目标还进不了预览，验收第 5、7 条的 Gitea 路径在下一个 PR 连同能力声明的实测一起落。

**目标连续性。** 预览冻结目标的连续性证据：本地目标是 Git 公共目录的文件系统身份（设备号与 inode；同路径的新 clone 是另一个 inode，整个目录搬走再搬回还是同一个），平台目标是平台实例与仓库稳定 ID。执行前重读对照，不一致就什么都不写、留 `TARGET_IDENTITY_MISMATCH` 给人（原仓库回来之后同一意图自己续上；CT-REPO 第 36 行「同路径新 clone 不证明目标连续」）。

**每次尝试的输入先冻结。** 执行前把这次给执行体的精确输入记进意图（`attempt`：序号、执行体侧重试键、候选提交、预期头），再执行；重试原样复用，所以「工具已写、control 没来得及确认」之后重启，工具按同一输入与同一重试键认出自己的结果（`already_applied`），不会拿已经变化的目标头重新规划。只有终态回读、或工具证明什么都没写（`accept_advance` 下的 `HEAD_DRIFT` / `CAS_REJECTED`）才清掉它换新序号重规划；`KEY_REUSED` 一律当结果未知。

**执行与回读（本地目标）。** `control` 的后台 worker 每秒看一遍开着的意图：`pending` 立刻执行；`unknown` 每 10 秒再试一次。执行就是 P1 的 `hctl2-tool integrate`（库内调用，同一份代码）：源提交、基线、结果树、目标 ref、预期头（`accept_advance` 下取执行前刚读到的头）、策略、以意图 ID 为重试键。工具的 JSON 记录是回读：退出码 0 → `succeeded`，Receipt 记 `before_head / after_head / new_head / integrated_tree_sha`；`RESULT_UNKNOWN` → `unknown`，下次只回读；`TARGET_CHECKED_OUT` → `unknown` 加 `attention`（精确工作树路径与「切离后重试同一意图」），人切离后同一意图自己续上，不新提交；`HEAD_DRIFT` / `CAS_REJECTED` 在 `expected_head` 下 → `failed`（`TARGET_HEAD_MISMATCH`），在 `accept_advance` 下 → 重读头再试；其余拒绝 → `failed`。`fast_forward` 的回读树必须等于准入的结果树，`merge_commit` 的树是合并树。工具的快进条件本批放宽为「目标头是候选的祖先」（目标沿候选这条线前移也能快进，候选已可达则 `already_applied`），分叉仍拒绝，P1 的用例不变。工具的合并提交用仓库配置的 Git 身份；没有身份的仓库合并会以 `COMMIT_FAILED` 终态失败，这是现状，由谁给身份另议。

**同一 key 重投。** 已准入的 key 再预览返回冻结的预览、不再读目标，再提交回到同一个意图；同 key 换输入是 `IDEMPOTENCY_CONFLICT`。

**给第 7 包的框架（验收第 12 条的 Receipt 部分）。** `integration.show {repo_id, intent_id}` 返回 `{intent, receipt, effect_state}`；`integration.list {repo_id}` 列该 Repo 的意图。「完成 Task」的机械项拿 `receipt`：核 `receipt.source.change_set_revision_id`、`receipt.target`、`receipt.target_head_after`、`receipt.evidence_level`，并以 `integration_receipt` 记录的 `sources` 指回意图。错误码：`REVISION_NOT_ADMITTED`、`PLATFORM_NOT_BOUND`、`LOCAL_TARGET_NOT_ALLOWED`、`EXPECTED_HEAD_UNSUPPORTED`、`CAPABILITY_MISSING`、`TARGET_HEAD_UNKNOWN`、`TARGET_BUSY`、`VERSION_CONFLICT`、`IDEMPOTENCY_CONFLICT`、`INTENT_NOT_FOUND`、`INTENT_TERMINAL`、`READBACK_MISMATCH`，都带 `recovery_action`。

**验证。** `root//crates/repo:integration_test`（领域：预览冻结与拒绝、同 key 幂等、同目标互斥与终态释放、只有确认回读写 Receipt、回读与冻结不符拒绝、未知占用）；`root//apps/control:integration_test`（真实控制套接字 + 真实 Git 仓库 + 工具：被检出的目标等人、切离后同一意图续上并签唯一 Receipt、第二个意图 `TARGET_BUSY`、终态后新授权、预期头漂移 `failed` 不重试、`accept_advance` 合并提交记实际头并在执行体提交被删时包提交、同 key 重预览回放冻结预览）。新增：同路径新 clone 不被当成原目标、原仓库回来后续上；工具已写而确认丢失后按冻结的尝试恢复、只签一张 Receipt；`accept_advance` + `fast_forward` 的目标已到候选时 `already_applied`。还没接的：平台目标的执行与保护快照回读（Gitea、GitHub）、重启整个 control 的用例、真实 CLI 到处理函数的用例（等 #384 合入后用 `changeset::admit` 造源版本）。

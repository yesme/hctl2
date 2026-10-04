# Participant 控制面半边

第 2 包实现 Agency Binding、工种的显式接受、派工意图与映射、观测记录和 Proposal 字节保全。它依赖 Store 与共享合同，不依赖 Agency 服务、Herdr 或执行进程。`apps/control/src/agency.rs` 处理本地消费、RPC 与私有配对凭据。

## 已有接法

`accept_binding` 固定公开目录，`accept_profession` 接受精确名册项。`prepare_dispatch` 要求已保存的授权归属者与接受过的工种；它在 Store 事务里保存规格与准备 outbox。`record_dispatch` 同事务保存映射、确认准备与激活 outbox。控制面端口发送外部动作前调用 `begin_effect`；响应未知先回读，不盲目重发。

Pending 的 prepare / activate 先通过本租户的幂等 `fence` 同步控制面写者，再重核原授权并调用 `begin_effect`。Agency 联系不上或代次同步失败时，业务动作仍是未尝试的 Pending，不把服务恢复后的合法派工判为终局拒绝。已是 Unknown 的动作仍只回读，不借这条路径重发。

`preserve_proposal` 保存并回读精确材料，`proposal_inbox` 只是接收与审计，不是 Project / Run 准入。观测、联系不上和无法履约不能自行完成 Task 或 Invocation。Buck：`root//crates/participant:participant`、`:clippy`；完整端口链的测试在 `root//agency:control_port_test`。

## 第 5 包主体 · 选入校验与 Worker Profile

主体分两份 PR 串行交付：本批先提供选入校验与 Profile；下一批接 Invocation 状态机、Context、Agency 四步启动、结果准入、Room 投影与连接票据。本批不创建 Invocation，也不宣称派工或演示 2 已完成。

`profiles::prepare_profile / admit_profile` 沿用 Store 的命令、幂等结果与事务。创建和更新只移动 `worker_profile` 指针；`worker_profile_revision` 用规范内容摘要定位，不原地修改。选入记录只引用精确 Revision，不引用 current。重投须保持 actor 与输入，修改预览内容或提交旧指针版本均拒绝。`profile_at` 读取并复核精确版本与内容摘要。

初版只实现只读配置：`mode = read_only`；权限为 `context.read`、`git.read`、`terminal.observe` 的不重复子集。Profile 不授予治理命令、Task 完成、派工或集成权。`max_context_bytes` 是字节预算，不是 token 估算。`environment` 描述要求的环境属性，不保存 Agency 门后的目录、主机或进程。写入型配置随第 6 包补，以上是本批实现范围，不是新增约束。

`selection::validate_roster` 核接受过的 Binding、工种与条款摘要，Profile 的 Harness / 模型、能力承诺、权限和预算，再核 Skill 的精确内容与核验报告。Skill 引用沿用现有 `Reference` 的 `Revision(content_digest)`；provider revision 由接受目录中的唯一精确项固定，缺项或歧义拒绝。unknown 不升为 known，回读不一致拒绝，optional 缺失返回逐候选的 `optional_skill_degradations`。Skill 正文仍由 Agency 保存。

Project 的 `selection_policy` 初版接受 `allowed_agencies`（Binding ID）、`allowed_professions`（目录项稳定 ID）、`permissions`、`max_context_bytes`。省略表示该维度未另收窄，显式空数组表示不允许任何项；未知字段拒绝。权限格式是 `{"allow":["context.read"]}`，预算格式是 `{"max_bytes":65536}`。Profile 的要求不得超出选入上限。这里只核 Room 选人，不实施 Run 席位多样性或 Gate 计票规则。

校验返回依赖记录，由 Project 的名册事务核精确版本；名册的范围与写入仍归 Project。`resolve_room_candidate(store, project, room, target)` 只在当前 Room 名册里按选入记录 ID 或职责精确解析；零个或多个候选都返回类型化错误，显示名不是路由键。它不创建调用、不授予权限；Invocation 入口另核 human 来源、Project / Room 状态与派工预览。

| 本批覆盖的 CT 内容 | 会失败的输入与目标 | 留给主体下一批 |
| --- | --- | --- |
| 工种、Profile 与选人策略 | P：虚构引用、同目录工种从 A 家接受却选 B 家、条款摘要错、Profile 模型不符、能力缺项、超出策略或选入上限、预览后政策更新 | 实际派工配置和终局状态；目录成员检查在正常接受与不可变绑定路径下未单独测到 |
| Skill 申报与核验 | P：required 缺失、回读摘要不符、unknown 或转述伪装 known；optional 缺失的降级输出 | Execution Spec 的 activated 状态与实际装载 |
| 名册独立、精确提及 | P：别的 Room 的记录、模糊显示名、重复职责；同 Harness 两条记录仍独立 | human 批准建议、Run 席位与模型不得发起调用 |
| 精确版本与重放 | W：旧预览、篡改预览、不同 actor / 输入重投；P：Profile 指针更新不回写旧记录 | Invocation / Attempt 的冻结与替代 |
| 真实 CLI 接线 | C：`project select` 对不存在候选拒绝、预览不写、确认后写一份名册 | 最少的 Invocation preview / start / show 全链 |

W = `root//crates/participant:profiles_test`；P = `root//crates/project:domain_test`；C = `root//apps/cli:cli_test`。本批没有新三方依赖、脚本或 Agency 运行时。

## 第 5 包任务说明

主体下一批（5b）先接 Profile 创建的 control Preview / Submit 与最少 CLI，复用 `prepare_profile / admit_profile`。真实主链按「配对 Agency → 接受工种 → 人确认创建只读 Profile → 选入名册 → Invocation 预览 / 启动 / 查看」起步，不靠测试直接写 Store，也不等后半段。Profile 更新 CLI 和只读查询留后半段。

5b 同时接两处选入补齐：Project 创建 / 更新时按 `SelectionPolicy` 解析策略，让未知字段在保存前失败；新选入记录的 `sources` 带上精确 Worker Profile Revision，方便沿来源链读取，既有不可变选入记录不回写。分别在 Project 的 `domain_test` 补失败输入与来源引用断言。

当前端口只核授权归属者的精确记录版本；完整领域授权仍未接线。第 5 包在恢复 Pending 前传入真实的授权判定，不沿用端口中的 `still_authorized=true`；Unknown / Confirmed 的回读和字节保全不发新授权，但后续激活、输入与准入另核当前语义归属。当前逐份成果保全在首个错误处返回；第 5 包接多成果时改为逐份报告，不让一个坏成果挡住其余保全。

依据：[演示线开工书第 5 包](../../../.memo/design/p2-control-20260906/07-demo-kickoff.md#第-5-包--派工)、[从授权到派工](../../../docs/design/spec/connections.md#project--run--participant从授权到派工)、[结果准入](../../../docs/design/spec/connections.md#participant--project--run结果准入)。

| 文件 | 要补什么 |
| --- | --- |
| `participant/src/selection.rs`、`profiles.rs` | Room 选入校验与 Profile 已在主体首批接线；Run 席位由 Run 后续消费同一精确引用形状 |
| `participant/src/tickets.rs` | 按当前归属者、冻结规格、权限与控制面写者签发短期票据；连接不恢复领域授权 |
| `project/src/invocation.rs` | Room Invocation 的预览、状态机、语义版本、取消与重试、结果准入和 Room 投影；不要放到本 crate |
| `apps/control/src/dispatch.rs` | 调用 Context 组装器和本包的准备 / 映射接口，再沿 Agency 端口激活；恢复原 outbox |
| `apps/cli/src/invocation.rs`、`terminal.rs` | Invocation 与 Terminal 命令；端口的观察和输入对手方是 Agency |

先走只读调用。没有唯一、获准的本 Room 候选时拒绝；模型提及与建议不创建调用。预览冻结执行者、Context、权限、预算与评审发布策略。四步启动沿现有接口，不在 RPC 成功后补写授权。当前测试中的授权归属者是测试预置记录，不表示 Invocation 业务已交付。

每份结果保全后还要校验归属者状态与语义版本、绑定、Spec / Bundle、逐项输出范围、权限和证据，再由 Project 准入；迟到或失权的结果只留审计。包 6 才补 Write Lease、封存与 ChangeSet 准入。派工票据签发、真实 Terminal 连接、丢失判定与停止隔离报告也在后续业务接线，不能借当前配对凭据直接给模型命令权。

失败用例：未选入名册、跨 Room 授权、旧语义版本、旧预览、错误输出授权、取消期间返回结果、响应丢失与控制面重启、未知输入投递、停止报告缺失。对照 CT-PROJECT、CT-PARTICIPANT、CT-CONNECTION 的现行行逐条列已做与未做，不把端口测试当领域准入测试。

## 第 5 包后半段任务说明

后半段由 Qwen 或 DeepSeek 在主体全部合入后接，不在本批增加 CLI 或只读查询。以下是本批已经可用的库入口；Invocation 与票据入口由主体下一批补齐，届时同步写出精确输入输出。

| 后半段命令 | 可复用接口与输入输出 | CT 对照 |
| --- | --- | --- |
| `profession` | 复用既有接受目录、`accept_profession` 与 `agency.profession.list`；输入 Binding ID 和精确 Profession，输出接受记录；不得靠显示名重建身份 | CT-PARTICIPANT 工种冻结与名册独立 |
| `room roster` | 复用 Project `prepare / admit` 的 `Action::Select` 和 `project.roster`；输入 Project / Room 与预期版本、`Selection[]`，输出不可变引用与 optional 降级项 | CT-PROJECT 选人策略、CT-PARTICIPANT Room / Run 独立身份 |
| Profile 更新 CLI 与只读查询 | 创建的最少 CLI / control 接线由 5b 先做；更新复用 `ProfileInput {key, action: update}` → `ProfilePlan` → 确认后 `admit_profile`，输出指针版本与精确 Revision。读取复用 `profile_at`；可信 actor 从已认证入口取得，不从 JSON 接受 | CT-CONNECTION 共享定义的原作用域、精确版本、权限逐级收窄 |
| Invocation list / cancel / retry、Terminal inspect / attach / replay | 本批不虚设 API；下一份主体 PR 定义状态机、取消 / 重试与票据后，在对应 README 补接线表。后半段只作命令与只读展示，不复制 reducer | CT-PROJECT Invocation 合法边；CT-PARTICIPANT 终端权限；CT-CONNECTION 失败恢复 |

# store · 控制面存储与命令内核

P2.1 甲的库级交付；任务书见 [04-p21-kickoff §五](../../../.memo/design/p2-control-20260906/04-p21-kickoff.md#五第一包任务书给-codex直接贴)。消费方只依赖 `root//crates/store:store`。本库没有守护进程、RPC、供应端适配器或 Repo / Room / Task 业务命令。

## 模块与接入边界

| 文件 | 职责 |
| --- | --- |
| `model.rs` | 按既有作用域的对象键、版本引用、命令信封、Room 三态与 Project 设置字段 |
| `schema.rs` | rusqlite_migration 迁移、schema 检查、SQLite 快照与回退 |
| `store.rs` | writer 锁与代次、启动状态、事务入口、读取、恢复后待投递动作核验 |
| `command.rs` | 同事务记录事件、幂等结果、inbox / outbox、材料准入与交付确认；inbox 按绑定对象身份、消费目标对象和消息键去重 |
| `materials.rs` | 私有本地 Git 裸库；候选保存、精确读取、备份复制；不注册业务 Repo |
| `backup.rs` | 治理记录与已承诺材料的一致备份集、验证与排他恢复 |
| `error.rs` | `code`、`message`、`recovery_action`，由乙呈现给客户端 |

`Store::open_with_status` 取得 foundation 的 OS 锁后才开写连接。schema 1 只含身份，schema 2 加命令内核；`user_version` 由 rusqlite_migration 管理。升级前用 Online Backup 保存可核验快照；升级失败恢复原 schema、身份和代次。升级期间 `StartupStatus::require_ready` 返回 `UPGRADE_IN_PROGRESS`，尚未服务返回 `STORE_NOT_READY`；乙负责并发呈现这些状态。

`Store::submit` 接受当前 `WriterGeneration`、可信来源 `TrustedActor`、完整 `Command` 和 reducer 回调。`TrustedActor` 没有反序列化入口，由乙的连接身份或后续可信适配器构造；不能拿客户端填写的 actor 自证身份。reducer 在同一事务中校验业务前置、写记录、准入材料、登记副作用，任何写操作出错都会使整条命令回退，即使回调误吞了错误。外部调用留在事务之外。

本库核作用域、目标预期版本、记录版本递增、基本身份不变与唯一索引；具体业务状态转换、引用是否授权、领域 Revision 的计算与 binding 当前有效性由各模块 reducer 校验。`get`、`versions` 和副作用读取是控制面内部 API，不是可直接转发给任意客户端的公共 Query。乙的 `.proto`、服务进程文件归乙，丁的服务生命周期文件归丁。

inbox 的绑定版本单独保存供审计，不参与消息去重；同一绑定对象升版后的重复消息仍返回原结果。同一消息送到不同 Project 的目标可分别消费，不能按全局消息键去重。`message_key` 由适配器提供，应包含实际来源的稳定命名空间，不把不同来源碰巧同名的消息混为一条；具体供应端的取键由 P2.2 适配器实现。

Repo 事实保持 Repo 作用域；共享定义和绑定可保持控制面或 Repo 作用域。Project 内的 Room / Task / Run 键携带 Project；同 Repo 的两个 Project 可各有主 Room、各映射同一个外部实体为自己的 Task。外部实体仍是四元组，不加 Project 字段。Room 状态与 Project 设置可保存历史版本；关闭、归档和更新设置的业务行为由 P2.2 实现。

## 保存、准入与外部效果

材料先经 `save_material` 在 SQLite 事务之外保存，返回带控制面、作用域、材料版本和精确字节 SHA-256 的 `MaterialRef`；Git 对象编号不进入领域引用。同一命令、槽位和字节得到同一候选；候选在 `admit_material` 成功前不是治理权威。每个版本有持久 Git ref，普通 `git gc` 不会回收它。本阶段保守保留候选和历史版本，不启用应用级自动清理，代价是材料与迁移快照占用会增长。

交付通过 `DeliveryGrant` 冻结接收者、用途与精确材料集合。发送字节不等于对方收到；可信交付适配器用接收者回读的实际字节确认，摘要不同拒绝。未来调用包的身份认证和传输由对应包完成，不由本库猜测。

outbox 保存原意图、绑定版本、输入摘要、权限范围、幂等键与外部资源的冲突范围。`begin_effect` 在发送前持久化 `unknown`；重启发现未知状态只回读，不能盲重发。已确认结果不可改；同事务可写模块的确认凭证与结果。尚未发送的旧代次动作由可信调用方核对原冻结授权后显式恢复；`resume_pending_effect` 的布尔入参是这项内部核验结果，不是客户端授权字段。未知动作仍占用冲突范围；真实平台的回读与幂等能力留给适配包，本包用 SQLite 测试替身验证效果次数。

凭据表只存 SecretStore 引用。`require_secret` 仅供可信适配器使用；引用或值缺失、空值都报 `CREDENTIAL_UNAVAILABLE`，不会作为“无凭据”继续写外部系统。密钥值不进治理记录或备份。

## 备份与恢复

清单权威是 [spec/system §备份与恢复](../../../docs/design/spec/system.md#备份与恢复)。模块把承诺保存的正文、Context Bundle 原文、自己的 Profile 定义等登记为 `Record.materials`；冻结配置、Skill 引用与摘要随记录保存。备份核对全部准入材料及历史事件的材料引用，缺字节或定位即失败。Agency 安装定义、内容服务器数据与密钥值不在本备份内。

`backup` 独占借用 writer，复制 SQLite 快照和独立的 Git mirror（无硬链接），最后才写完成清单。`verify_backup` 验身份、schema、快照摘要、事件边界和承诺材料集合，也接受可升级的旧 schema。`restore` 先取得排他锁；不同控制面身份拒绝覆盖。先补齐材料，在暂存副本上复用启动的迁移与校验，再用 SQLite Backup API 原子恢复已推进代次的记录快照，代次高于备份与当前存储两者。迁移失败不动原备份和现存记录；原地恢复保留已有较新材料，不用跨目录重命名拼事务。恢复仅支持本库已知格式/schema，不猜读未来版本。

## 错误码与乙的呈现

乙原样保留 `code`、`message`、`recovery_action`；下表不另定义 RPC 枚举。一个错误码可有不同恢复提示，以实际返回值为准。表示未知结果或冲突的错误不授权重新发送外部动作。

| code | 含义 | recovery_action | 乙的呈现 |
| --- | --- | --- | --- |
| `INVALID_INPUT` | 空值、非法结构或摘要、身份不变量被违反 | `correct_input` | 输入拒绝，显示具体原因 |
| `INVALID_JSON` | JSON 无法解码 | `correct_input` | 显示解析错误 |
| `ACTOR_MISMATCH` | 信封 actor 不等于可信来源 | `reauthenticate` | 要求重新认证 |
| `PERMISSION_DENIED` | 作用域、交付用途或原授权不再允许 | `request_authorization` / `reconcile_authorization` | 拒绝操作，显示对应授权提示 |
| `WRITER_BUSY` | 已有 writer 持锁 | `stop_previous_writer` | 说明当前不能接管 |
| `STALE_WRITER` | writer 代次已被替代 | `reconnect_control` | 重新连接，不沿用旧写入者 |
| `STORE_NOT_READY` | 尚未服务 | `check_status` | 显示启动状态 |
| `UPGRADE_IN_PROGRESS` | 正在迁移 | `retry_after_upgrade` | 等升级完成再提交 |
| `SCHEMA_UNSUPPORTED` | 不是本应用的库、残缺或未来 schema | `use_compatible_version` | 拒绝开库，选择兼容软件 |
| `MIGRATION_FAILED` | 迁移或升级后校验失败 | `inspect_migration_snapshot` | 显示失败及保留快照的位置上下文 |
| `STORAGE_CORRUPT` | 完整性或外键检查失败 | `restore_backup` | 提示显式恢复 |
| `STORAGE_IO` / `STORAGE_SQLITE` / `FOUNDATION` | 存储或基础库故障 | `inspect_storage` | 故障诊断，不解释成领域冲突 |
| `IDEMPOTENCY_CONFLICT` | 同一命令身份用了不同内容或授权 | `use_original_command` | 展示冲突，不能当成功重放 |
| `INBOX_CONFLICT` | 同一来源消息键对应不同摘要 | `reconcile_source` | 对账供应端记录 |
| `COMMAND_ABORTED` | reducer 吞掉写入错误，整条命令已回退 | `fix_reducer` | 实现故障，不展示半成功 |
| `VERSION_CONFLICT` | 目标已变或记录版本不连续 | `preview_current_version` | 重新预览 |
| `UNIQUENESS_CONFLICT` | 同 Project 已有主 Room 或该实体的 Task | `preview_existing_object` | 展示既有对象，不要求检查存储 |
| `EFFECT_CONFLICT` | 未决动作占着同一外部资源 | `read_back_original_intent` | 保留未知状态并回读 |
| `READBACK_REQUIRED` | 动作未知、已完成或属于旧 writer | `read_back_original_intent` | 查原意图，不盲重发 |
| `READBACK_MISMATCH` | 回读对象不符或试图改已确认结果 | `read_back_original_intent` / `inspect_readback` | 拒绝确认并显示差异 |
| `MATERIAL_NOT_ADMITTED` | 缺准入定位 | `admit_material` / `restore_or_admit_material` | 提示按原命令准入或显式恢复 |
| `MATERIAL_UNAVAILABLE` | 材料库、定位或字节不可读 | `restore_material` | 暂停依赖材料的操作 |
| `MATERIAL_DIGEST_MISMATCH` | 定位、原字节或接收字节不匹配 | `restore_material` / `retry_delivery` | 按返回提示恢复材料或重新交付 |
| `CREDENTIAL_UNAVAILABLE` | 凭据引用或值缺失、值为空 | `restore_secret_store` | 阻止依赖动作，不输出密钥 |
| `GIT_UNAVAILABLE` / `GIT_VERSION` | 找不到 Git 或低于 2.39 | `install_git` | 提示安装所需 Git |
| `BACKUP_INCOMPLETE` | 清单、边界或承诺材料不完整 | `select_complete_backup` / `restore_material` | 拒绝恢复并指明缺项 |
| `BACKUP_CHANGED` | 恢复准备期间备份变了 | `verify_backup_again` | 重新验证原备份 |
| `RESTORE_IDENTITY_CONFLICT` | 现存控制面与备份身份不同 | `select_matching_backup` | 拒绝覆盖另一控制面 |
| `RESTORE_INCOMPLETE` | SQLite 恢复未完成 | `retry_restore` | 保留故障，显式重试恢复 |

## 验证入口与失败用例

在 `src/` 运行 `./buck2 test root//crates/store/...`。以下五组都是独立 native `rust_test`；`clippy` 汇集库与各组的 Buck Clippy 诊断。没有 Cargo build/test 旁路或新增脚本。

| target（均在 `root//crates/store:`） | 可失败的情形 |
| --- | --- |
| `unit_test` | 旧 schema 未升级就服务；升级中未拒绝；迁移 SQL 失败、进程中途退出、成功 SQL 改坏身份；未来或残缺 schema 被误认 |
| `command_test` | 第二 writer；旧代次重放；信封各组缺失、空值或错摘要；幂等键异载荷；同命令或 inbox 升版重投形成第二次效果；不同 Project 的同源事件被误去重；跨模块半事务；回调吞掉写入错误 |
| `command_test` | 真子进程退出：commit 前、commit 后未发送、外部成功确认丢失、确认凭证 commit 前；未知动作被重发、冲突范围提前释放、错目标回读被接受；幂等供应端替身实际收到两次重复投递、效果计数须为一（不代表所有供应端都有此能力） |
| `materials_test` | 真子进程在保存后或准入后退出；响应丢失形成第二份准入；保护候选被 Git GC 清掉；准入时权限/版本/代次已变；定位或字节丢失；交付范围或接收字节不匹配 |
| `recovery_test` | 删除投影表后重建不一致；备份依赖原始对象文件；活 writer 下恢复；恢复旧备份未超过当前代次或未升级就服务；不同身份覆盖；缺 Bundle / Profile 定义未报错；缺 Agency 定义被误报；凭据缺失或空值未拒绝、密钥值进入备份 |
| `model_test` | 同 Repo 的 Project 互相吞 Task；同一 Project 同实体出现两个 Task；实体换绑；同 Project 两个主 Room；Repo 级 Room；Room 三态或 Project 设置历史丢失 |

并跑 `root//crates/foundation/...` 与 `root//apps/tool/...`，覆盖复用 Git helper、文件锁和 Online Backup 的回归。三平台结果以 PR 的 Code CI 为准；B0 的进程和服务端到端恢复由丁验收。

# control · 本地控制守护进程

## Agency 端口（演示线第 2 包）

`src/agency.rs` 只消费共享合同，独立服务代码在 `src/agency`。`hctl2 agency pair|bindings|catalog|accept` 提供本地配对与冻结工种入口；派工业务命令留第 5 包。首次配对拉起随包的 `agency`，以后控制面重启只恢复自己的消费和写者栅栏，不停止共用 Agency。配对端口与密钥放 SecretStore 的私有文件后端，不进入治理记录或备份。

准备 / 激活 / 保全 RPC 由已有 Store outbox / inbox 接线，实际调用与失败语义见 [Participant](../../crates/participant/README.md)。端口观测不产生领域结果，联系不上不撤销授权。Agency 自己的 journal 与成果由自己恢复；控制面备份不替代它。运行时、Context 与 Invocation 三包各占自己的文件。

## Worker Profile 创建入口（第 5b 包首段）

`profiles.rs` 把 `profile.create` 接到既有 Preview / Submit 和 Participant 的 `prepare_profile / admit_profile`。预览 token 绑定原输入和方案；提交校验 `command_id = profile:KEY` 与 `idempotency_key = KEY`，actor 来自连接。没有新 RPC、执行服务或配对凭据通道。这里只建定义，不调用 Agency，也不创建 Invocation。

最少 CLI：`hctl2 profile create --input profile.json --key KEY` 先预览，同一命令加 `--preview-token TOKEN` 确认。文件是 `{"id":"research","profile":{…}}`，其中 Profile 的字段见 [Participant README](../../crates/participant/README.md#第-5-包主体--选入校验与-worker-profile)。创建后精确引用在 `revision`，选人时填入 `worker_profiles`。更新命令和只读查询见下面的[后半段](#第-5-包后半段--命令入口)；下一段主体接 Invocation 与四步启动，不由本入口代替。

## 第 5b 包 · 只读派工主链

`dispatch.rs` 接最少的 `invocation.start` Preview / Submit 和 `invocation.show` Query，actor 仍由本机连接取得。CLI 为 `hctl2 invocation preview --input invocation.json --key KEY`、`start --input invocation.json --key KEY --preview-token TOKEN`、`show PROJECT INVOCATION`。输入是 Project 的 `invocation::Input`，不接受调用者写好的 Execution Spec 或 Bundle。预览返回真实选入记录、Profile、权限、预算、必需 Skill、发布确认缺省和冻结 Context；token 数未实现，仍为 `null`。

`dispatch/context.rs` 用第 4 包的选材和组装器：请求正文、Topic 的已确认提要及来源、显式 Task 的精确评论、该 Room 最新一页的服务器顺序窗口。只授 `context.read` 内所选来源，不授整 Project 的资料读取；超预算走既有 Pointer 字节副本。提交前重核选人、策略、Room 与存储来源版本。当前原生 Agency 不申报 required Skill；带必需 Skill 时因没有取回原文字节的端口返回 `SKILL_DELIVERY_UNAVAILABLE`，不冒充已装载。额外 Memo / Artifact 选材和平台评审评论线仍未接。

人确认后，后台沿 Store 意图准备、持久化映射、激活；仅原 Invocation reducer 在确认第 4 步后提交运行状态。Unknown 只回读原请求。每份成果先保存和回读字节再确认；单项冲突按份报告，不挡后面的保全。Project 校验逐项身份、权限、冻结引用、证据与 schema，在同一事务里准入只读回答、结束 Invocation 和登记 Room 投影。它不判 Task 完成，不把一轮返回当成任务验收。

`dispatch/recovery.rs` 只清理原派工，撤权后不走重新授权。停止请求不证明隔离，停止报告明确记 `isolation_confirmed=false`。完成后的投影独立于 Agency 在线状态，以原 Matrix PUT 事务键恢复；已准入记录与待投影意图可从 `invocation.show` 查看。同一启动键重投取原冻结包并走 Store 原命令校验，重启不重取当前窗口、不新派工。联系不上保持观测错误；已有映射被 Agency 明确报不存在时才进入丢失，停止依据仍保留。

本机 `root//packaging/release:room-cli-test` 用真实 CLI、控制守护进程、独立脚本 Agency 和锁定的 Tuwunel / Gitea 走「配对 → 接受工种 → 创建 Profile → 选人 → 预览 → 启动 → 查看 → Room」。另测重启重投，以及 Matrix 停止后准入回答、控制面重启且 Agency 已停时恢复投影。它不运行真实 Claude，不冒充演示 2 验收。waiting_input 的 Request 收发、受管输入（终端接管）和写入型调用尚未接，本批不把合法边表测试报成这些能力。

## 第 5 包后半段 · 命令入口

`dispatch.rs` 增 `invocation.list` Query，以及 `invocation.cancel` 的 Preview / Submit：预览是领域侧 `cancel_preview` 的只读投影，确认后 Submit 把原 `End` 交给既有 `invocation::end`，不另开状态路径。`profiles.rs` 增 `profile.show`。三者与 Terminal 一样从连接的 `TrustedActor` 取身份、先过 `chat::owner` 的 Project 范围门再读记录，payload 不能自报 actor；错误 JSON 与主链一致（`error.code / message / recovery_action` 加非零退出）。

`terminal.inspect | replay | attach` 挂在 Query 上，不占用 Submit 的幂等身份，而且三条都是读：`dispatch_observation` 与 `dispatch_contact` 只由对账循环写，`inspect` 走 `agency::inspect_dispatch`（`observe_dispatch` 拆出来的只读一半），人看过不留记录——「谁看过」是访问日志的策略点，不进治理记录。因为 `inspect` 要等 Agency，这是 Query 处理里唯一的异步分支，它用 `shared.lock().await` 取存储，不用 `access()` 的阻塞锁。`attach` 直接返回类型化拒绝，不签发票据、不联系 Agency。

CLI 增 `profession` 与 `terminal` 两棵子命令树，以及 `room roster show | select`、`profile update | show`、`invocation list | cancel | retry`。写命令仍是「不带 `--preview-token` 就预览、带 token 就提交」，`profession accept` 除外：它不在 `is_dangerous` 里，走 `keyed_submit` 直接提交、没有预览闸门，与既有 `agency accept` 相同。命名按领域对象正名——`profession accept`、`room roster show | select` 是正名，既有的 `agency accept`、`project roster | select` 降为文档里写明的别名；本批两套都留，收敛成一套放第 8 包（所有者 2026-10-06 裁定）。`room-cli-test` 增一条端到端用例，从真实命令行走完这 12 条命令并逐条验拒绝；受管输入与写入型调用不在其中。

## 第 6 包 · 拆分 2：写入预览、租约与封存准入

原 `invocation preview | start | show` 接受 Project 新增的可选 `write` 边界，不新建 RPC。预览调用 `repo::review::freeze_policy` 保存不可改写的策略定义，再由领域读取；没有租约 grant 或发布意图。提交用同一授权事务激活租约、冻结 Spec 与 prepare outbox，后续沿既有 Agency 四步启动。只读输入的编码与路径保留。

`dispatch/context.rs` 把待启动 ChangeSet、基线、租约、完整目标正文、完整策略正文与注册副本路径的规范字节作为必需材料送到执行体。路径标为控制面机器所有，不是远程执行目录。可选 `review_change_set_revision` 为本 Repo 的精确 `store::Reference`；读取钩子已留，第 10 条接平台评论前会明确拒绝，不默默删掉该来源。Context 的其他来源仍沿第 4 包接口。

取消后，`dispatch/recovery.rs` 跨页读取原派工的停止报告：写入型不在逻辑取消或一轮结束时提前结束回读，要看到脚本退出或确认从未激活。保存报告和租约撤销同事务；缺证明继续占用旧租约。停止报告仍不代表已实施系统级隔离。

`dispatch/write.rs` 读取原提案保存的字节，在控制面存储事务外调用现场工具封存 Git。准入时重新核调用、租约、Spec、输出与原材料，ChangeSet Revision、结果、调用终态、租约撤销中、Room 投影和发布意图共用一个事务。策略类型、冻结与发布入队复用 `repo::review::{Policy, freeze_policy, Publication, enqueue}`；不在提案事务里嵌套调用外层 `admit_with_publication` 的 Store 提交。推送、建评审请求和平台映射仍归发布 worker，本包不复制它。

人的封存经 `changeset seal` 预览与确认，使用独立命令，不借用 Invocation 租约。`changeset show | diff` 读取已准入版本；路径只作为此次 Git 操作输入，不新增工作区注册对象。本段暂限控制面可读取的本地 Git；跨机 Git 交付、残留接管 / 丢弃、评审评论读取另交后续段。

`root//packaging/release:room-cli-test` 的 `write_dispatch_real_cli_freezes_policy_grants_once_and_requires_exit_before_regrant` 从真实 CLI、控制进程、随包 Gitea 和脚本 Agency 走预览、启动、查看与取消：检查第二写入者被拒、Spec 不随 Project 确认缺省变化、退出报告后才预览下一代租约。取消暴露的 shell 子进程持有管道问题在 Agency 的脚本执行体用原生进程组修正，未新增控制面进程管理。

同一目标的 `write_dispatch_real_cli_admits_and_publishes_to_packaged_gitea` 让脚本在隔离执行目录生成真实提交，经封存准入与同事务入队，由发布 worker 推送、建请求、写 `changeset_platform_binding`。用随包 `tea` 独立回读请求与分支头，再核精确结果树及原工作副本的 HEAD 未移动。`write_dispatch_real_cli_persists_human_gate_and_publishes_after_restart` 另走确认开关打开的路径：准入后平台无请求，控制面重启后人经 `review publish` 放行；再次重启、重投仍只有一份版本、结果、意图与映射，一条平台请求。两个用例不使用准入或平台映射的测试缝。

`write_dispatch_real_cli_accepts_verified_no_changes_without_publishing` 交回未改动的实际工作目录，现场工具回读两棵 Git 树相同后接受空结果。它核对目标正文与本地路径、重启后的单份结果、没有 Revision / 发布意图 / 平台请求。脚本保留自己的交付目录直到受管停止；不在封存前退出并清掉未交付对象。

第 8 条的残留接管 / 采用 / 丢弃命令与第 10 条的平台评论线，统一留到 #405 合入后的「第 6 包收尾」PR；本段只提供后者需要的精确版本引用与读取钩子，不把未接读取说成可用。

## 第 6 包 · 集成一半

`integration.rs` 接 `integration.submit` 的 Preview / Submit 和 `integration.show | list` 的 Query，领域在 [Repo crate](../../crates/repo/README.md#第-6-包--集成一半意图两种授权形态receipt)。预览时读目标：本地目标用库内的 `hctl2-tool repo inspect` 取目标 ref 的头；已准入的 key 回放冻结预览、不再读。提交只持久化意图与效果；后台 worker（`reconcile`，与派工的 worker 并列）执行并回读，本地目标调库内的 `hctl2-tool integrate`，`accept_advance` 下执行前再读一次头。工具的拒绝按码分流：目标被检出留给人、结果未知只回读、预期头不符终态失败。随包 Gitea 目标：预览读分支头与生效的保护规则（`integration/gitea.rs`），执行时适配器只做一件写入——请求合并发布的评审请求（源头钉死）——发出前把尝试持久标为已发出，明确拒绝才撤回，响应丢了只回读不重发；回读先读平台的请求与分支（保护再对照一次），再调库内的 `hctl2-tool readback` 从平台的 Git 拉目标 ref 到 `<root>/integration/readback/<repo_id>.git` 核合并提交是否被目标承载、树与父提交，Receipt 的证据通道是 `hctl2-tool`。平台连接可注入（`drive_with` 收 `Connection`），这套规矩写在 `integration/target.rs` 的 `PlatformTarget` 上，Gitea（`integration/gitea.rs`）与 GitHub（`integration/github.rs`，随包 `gh`、经典保护 + 规则集生效规则、`PUT merge` 带 `sha`、快进在预览就拒绝）各只换适配器；用例用脚本化的 `tea` / `gh` 与一个真实裸仓库扮演平台，另有对公开沙箱 `yesme/hctl2-canary` 的实跑用例（`HCTL2_GITHUB_LIVE=1`）。绑定声明缺能力时预览就拒绝。发布评审（`review.rs`，验收第 4 条与第 9 条发布半边）是同一条链的前一端：意图由版本准入的事务写入（`repo::changeset::admit_with_publication`），后台 worker `review::reconcile` 分两段执行——持 Git 凭据推冻结的提交到策略分支（`--force-with-lease` 钉上次确认的头，推完 `ls-remote` 回读）、再经同一套 `PlatformTarget` 接口找或建评审请求——每段回读后各自确认，映射证据写成集成一半读的 `changeset_platform_binding`；`review.publish` 是人放行「须人显式确认」意图的两步命令，`review.show|list` 查询。领域与状态机见 [Repo crate](../../crates/repo/README.md#第-6-包--发布评审意图两段映射证据)。

## 控制服务

P2.1 乙的进程边界。目录与私有 crate 名是 `control`；对外二进制仍是 `hctl2-control`。监听控制面数据目录下仅归属者可访问的 Unix socket（`control.sock`，模式 0600），对外提供 `hctl2.control.v1` 的 Query / Preview / Submit / Subscribe。存储打开在工作线程上，与 RPC 并发；`STORE_NOT_READY` 与 `UPGRADE_IN_PROGRESS` 把 `store` 的 `code` / `message` / `recovery_action` 原样放到错误对象里。

`TrustedActor` 由 Unix 连接的 `peer_cred.uid` 构造（`local-owner:<uid>` / DirectClient / Control），并与 socket 文件所有者比对，不从客户端字段反序列化。运维入口是 `status`、`doctor`、`export`、`backup`、`restore`。危险动作 `restore.apply` 必须先 Preview。P2.2 的 Repo、Task、Room 与 [Project / Request](../../crates/project/README.md)业务命令均沿此边界接入；Project 预览冻结输入、来源与版本，提交复用 Store 事务，网络读取和写入在事务外进行。

Subscribe 本轮只在内存里保留 32 条事件；游标过期时的重同步快照只带 `event_seq` 占位。P2.2 起序号改从 store 事件表出，快照要载投影。开库失败时进程继续服务，`status` / `doctor` 以类型化 `startup_error` 呈现，不自行退出。

随包 Tuwunel 与 Gitea 由 Process Compose 按首次消费拉起：Tuwunel 是基线（每个 Project 都有主 Room），存储打开成功之后才 `hctl2-services start --no-wait`，开库失败不碰服务；Gitea 由 `Supervisor::consume` 拉起并记进 `<root>/hosted-consumed.json`，之后随 start 一起起；戊在注册纯本地仓库（或显式选本地平台）时调用它，本包只提供入口。GitHub / GitLab 克隆不消费它。`services.consume` 是对应的运维 Submit。记录随 `services.backup` 进备份、随 `services.restore` 写回；记录损坏时只起基线并在 `last_error` 报出，不当成从未消费。`services.consume` 成功只表示「已记为消费、已请求启动」，就绪看 `services` 快照的 `available`。消费、启动、停止、备份、恢复在 Supervisor 内走同一把串行锁；备份与恢复在停掉已消费组件后若发现服务状态根里仍有进程在跑（例如人用 `hctl2-services start gitea` 起了未消费的 Gitea），以 `SERVICES_FAILED` 拒绝，不复制、不覆盖运行中的数据。备份里没有消费记录（本功能之前做的备份）时，恢复不改目标根的记录；备份里的记录损坏则恢复整体失败，不假装恢复完整。不等所有探针通过才接受命令。探针未过的组件 `available=false`。服务死活只出现在 `status` / `query services` 的观察字段（含 `source`、`observed_at`、`event_seq`），不写入治理记录。Vikunja 仍不随 `hctl2 start` 拉起。Gitea 管理员账号与访问令牌不在本包物化，交给 P2.2 戊接本地平台时处理。

缺省 `--root` 时与 `hctl2-services` 共用 `~/.local/state/hctl2`（或 `XDG_STATE_HOME/hctl2`）。显式 `--root` 才把服务状态嵌在该目录的 `services/` 下。没有其他组件在跑时，`hctl2 stop` 会 `down` 掉 Process Compose 项目；有 Cinny/Vikunja 等仍在跑则只停 Tuwunel 与 Gitea，本体可按 `--keep-project` 留下。

## 密钥后端

控制面保管的密钥（Gitea 管理员令牌、Matrix 应用服务令牌等）走 `foundation::SecretStore`。缺省是探测式：这台机器有可用的系统钥匙串就用钥匙串，没有就退到 `<root>/secrets/` 下的 0600 文件——所有者 2026-09-04 的裁定，两种都是获准后端。

要显式钉一种，用配置键 `secret_backend`，写在 `<root>/control.json`：

```json
{ "secret_backend": "user-file" }
```

`hctl2 init --secret-backend <system-keyring|user-file>` 与 `hctl2 start --secret-backend <…>` 写这个文件（保留文件里的其他键），未知取值由命令行直接拒绝，不静默退回缺省。缺省（文件不存在或不写该键）不变，仍是探测式。

行为：

- `user-file`：只用私有文件后端，完全不碰系统钥匙串。无屏幕会话（CI、纯终端）与隔离测试用这一档。
- `system-keyring`：只用系统钥匙串；这台机器没有可用钥匙串时**启动失败**（`hctl2 start` 报「control exited before ready」），不静默改道到文件——显式选择不该被悄悄换掉。
- 两个后端互相独立：换设置不会搬运已有密钥，原后端里的条目要人自己迁移或撤销。
- `status` 与 `doctor` 的 `policy.credential_storage` / `secret_backend` 报告**实际在用**的那一种（`system-keyring` 或 `user-file`）；配置本身读不出来时报 `CREDENTIAL_STORAGE_INVALID`。

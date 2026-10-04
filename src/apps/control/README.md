# control · 本地控制守护进程

## Agency 端口（演示线第 2 包）

`src/agency.rs` 只消费共享合同，独立服务代码在 `src/agency`。`hctl2 agency pair|bindings|catalog|accept` 提供本地配对与冻结工种入口；派工业务命令留第 5 包。首次配对拉起随包的 `agency`，以后控制面重启只恢复自己的消费和写者栅栏，不停止共用 Agency。配对端口与密钥放 SecretStore 的私有文件后端，不进入治理记录或备份。

准备 / 激活 / 保全 RPC 由已有 Store outbox / inbox 接线，实际调用与失败语义见 [Participant](../../crates/participant/README.md)。端口观测不产生领域结果，联系不上不撤销授权。Agency 自己的 journal 与成果由自己恢复；控制面备份不替代它。运行时、Context 与 Invocation 三包各占自己的文件。

## Worker Profile 创建入口（第 5b 包首段）

`profiles.rs` 把 `profile.create` 接到既有 Preview / Submit 和 Participant 的 `prepare_profile / admit_profile`。预览 token 绑定原输入和方案；提交校验 `command_id = profile:KEY` 与 `idempotency_key = KEY`，actor 来自连接。没有新 RPC、执行服务或配对凭据通道。这里只建定义，不调用 Agency，也不创建 Invocation。

最少 CLI：`hctl2 profile create --input profile.json --key KEY` 先预览，同一命令加 `--preview-token TOKEN` 确认。文件是 `{"id":"research","profile":{…}}`，其中 Profile 的字段见 [Participant README](../../crates/participant/README.md#第-5-包主体--选入校验与-worker-profile)。创建后精确引用在 `revision`，选人时填入 `worker_profiles`。更新命令和只读查询留第 5 包后半段；下一段主体接 Invocation 与四步启动，不由本入口代替。

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

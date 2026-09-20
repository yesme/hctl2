# control · 本地控制守护进程

P2.1 乙的进程边界。目录与私有 crate 名是 `control`；对外二进制仍是 `hctl2-control`。监听控制面数据目录下仅归属者可访问的 Unix socket（`control.sock`，模式 0600），对外提供 `hctl2.control.v1` 的 Query / Preview / Submit / Subscribe。存储打开在工作线程上，与 RPC 并发；`STORE_NOT_READY` 与 `UPGRADE_IN_PROGRESS` 把 `store` 的 `code` / `message` / `recovery_action` 原样放到错误对象里。

`TrustedActor` 由 Unix 连接的 `peer_cred.uid` 构造（`local-owner:<uid>` / DirectClient / Control），并与 socket 文件所有者比对，不从客户端字段反序列化。业务命令留 P2.2；本阶段运维入口是 `status`、`doctor`、`export`、`backup`、`restore`。危险动作 `restore.apply` 必须先 Preview。

Subscribe 本轮只在内存里保留 32 条事件；游标过期时的重同步快照只带 `event_seq` 占位。P2.2 起序号改从 store 事件表出，快照要载投影。开库失败时进程继续服务，`status` / `doctor` 以类型化 `startup_error` 呈现，不自行退出。

随包 Tuwunel 与 Gitea 由 Process Compose 按首次消费拉起：control 存储就绪后才 `up --detached` / `hctl2-services start --no-wait`，不等所有探针通过才接受命令。探针未过的组件 `available=false`。服务死活只出现在 `status` / `query services` 的观察字段，不写入治理记录。Vikunja 仍不随 `hctl2 start` 拉起。

# 通用机制的现成库

本目录回答「五处不该手写的通用机制各用什么库」（所有者 2026-09-03 裁决 I-05），加上 `hctl-tool wait` 命令要用的文件监听库。借用等级按研究层的六种复用决策记，偏好顺序是跨平台二进制 > SDK > 复制代码 > 借鉴想法；这六处都是库，等级为「采用 SDK」。outbox、租约与代次维持自研，不在此列。

| 文件 | 对象 | 决定 |
| --- | --- | --- |
| [`serde-jcs.md`](./serde-jcs.md) | RFC 8785 JCS 规范化 | 采用 SDK：`serde_json_canonicalizer` 首选（官方测试数据随测试），`serde_jcs` 可互换；契约测试钉官方向量；嵌套 NaN 与大整数键的缺陷由规范对象定义兜住 |
| [`fd-lock.md`](./fd-lock.md) | 进程间文件锁 | 采用 SDK，首选标准库 `File::try_lock`（零依赖，同样的 flock / LockFileEx）；`fd-lock` 作备选，其作者已提弃用；锁文件须在本地文件系统 |
| [`sqlite-online-backup.md`](./sqlite-online-backup.md) | 账本一致备份 | 采用 SDK：rusqlite `backup` feature 走 Online Backup API，WAL 下一致快照且不挡写者；禁止直接拷文件 |
| [`keyring.md`](./keyring.md) | 系统钥匙串 | 采用 SDK：keyring 经逐平台 store；无桌面会话的 Linux 持久来源（systemd 凭据或 0600 文件）需所有者拍板 |
| [`sqlite-fts5.md`](./sqlite-fts5.md) | 全文索引 | 采用 SDK：随 bundled SQLite 零新增依赖；中文靠应用层预分词加 unicode61，trigram 表补子串检索；索引独立文件、可重建、不进备份集 |
| [`notify.md`](./notify.md) | 文件系统监听 | 采用 SDK：notify 加 debouncer；`wait` 先查后等、事件只触发复查、定时兜底；CI / PR 事实不靠它，走 GitHub API |

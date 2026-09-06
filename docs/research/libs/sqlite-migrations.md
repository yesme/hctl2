# SQLite 账本 schema 迁移

> 对象：`rusqlite_migration 2.6.0`、现用 `rusqlite 0.40.2`；备选 `refinery / refinery-core 0.9.2`（2026-09-06）<br>
> 许可证：rusqlite_migration Apache-2.0；rusqlite 与 refinery MIT OR Apache-2.0；SQLite 公有领域<br>
> 定位：P2.1 启动时的停机升级，研究证据 E-LIB-SQLITE-MIGRATIONS；沿用 foundation 一致备份，不另造迁移器。

## 上游能力

`rusqlite_migration::Migrations::to_latest` 用 `PRAGMA user_version` 记已执行的迁移序号。2.6.0 源码 `goto_up` 在循环外创建一个事务，执行全部待升级 SQL、更新版本号，最后一起提交；`M::foreign_key_check()` 可在提交前验证外键。调用方不再套一层 `BEGIN`。

本机 macOS arm64 的隔离工程用 Rust 1.98、Reindeer 与 Buck2 编译检查 2.6.0 + rusqlite 0.40.2，通过（Build ID `49358cc6-aa6b-4b77-a01a-b1b41abaa5ef`）。接入注意：该库 build.rs 会从 README 生成 rustdoc，Reindeer 要启用原生 `[buildscript.run]` 并传 `CARGO_PKG_README="README.md"`；这不是需要自建脚本的数据库迁移步骤。本批只记接法，不把实验 fixup 加进产品依赖。

## 候选比较

| 候选 | 版本 | 许可证 | MSRV | 与现状相容性 |
| --- | --- | --- | --- | --- |
| rusqlite_migration | 2.6.0，查询时最新稳定 | Apache-2.0 | 发布包声明 1.95 | `rusqlite ^0.40.0`，与本库 0.40.2 同一 Connection 类型；只新增迁移库与 log，适合停机迁移 |
| refinery / refinery-core | 0.9.2，查询时最新稳定 | MIT OR Apache-2.0 | 未声明 | rusqlite feature 要求 `>=0.23, <=0.39`，不能直接消费现用 0.40.2；多后端与迁移历史表不是此处需要的能力 |
| 自写版本表与 SQL 循环 | 不适用 | 不适用 | 不适用 | 重做已有事务、版本核验与测试入口，不采用 |

## 边界与取舍

`user_version` 是本应用独占的 schema 版本，初值 0；`application_id` 是 SQLite 文件头内的应用格式标记，用来识别文件种类，不是 schema 版本，也不是账本身份。遇到错误应用标记或比本程序更新的 schema，报错保留原库；不能把不认识的文件改标记后当新库。

启动顺序沿用[已定计划 §五第 2 项](../../../.memo/design/p2-control-20260906/01-plan.md#五需要所有者一句话的取舍)：取得 control 单写者锁、暂不接业务命令 → 校验库身份 / 版本 → foundation Online Backup 生成并验证一致快照 → 迁移事务 → 校验、重建投影 → 对外服务。SQL 迁移只覆盖事件、幂等结果、outbox、租约与代次等事实源表；投影从事件重建，旧事件载荷在重放时升格，不改写历史载荷。

SQLite 官方十二步中的事务边界不能照“十二步全放事务”理解：若需要关外键，`foreign_keys=OFF` 在事务前执行，事务中设置无效；先建新表、复制、删旧表、最后改新表名，避免先改旧表名让引用跟着漂移；原索引、触发器、受影响视图要恢复；提交前运行 `foreign_key_check`，提交后重新启用外键。`journal_mode` 等连接设置同样放事务外，SQL 脚本不手写事务命令。

迁移失败先由库回滚，control 保持未服务状态并关闭全部连接；按[一致备份研究](./sqlite-online-backup.md)的恢复方式，用已验证快照恢复或替换完整数据库，避免旧 WAL 混入，重新打开后核对原版本、账本身份和完整性。保留失败库供诊断，不能以自动重试掩盖每次启动都失败的迁移。若事务已提交而投影重建失败，也不开始接受业务写入；可重建投影或按同一快照回退，只有恢复核验通过才报告恢复成功。

## 决定建议

**维持首选，采用 SDK：钉 `rusqlite_migration 2.6.0`，复用 `rusqlite 0.40.2`，Rust 1.98 满足其声明 MSRV 1.95。** 库用 `user_version` 管迁移序号，`application_id` 仅识别文件种类；启动取得单写者锁后先做 Online Backup，再由一次 `to_latest` 在一个事务内升级。允许停机已免去在线双写成本，refinery 既无必要又与当前 rusqlite 版本范围不合。本批不改 SQLite bundled 决定或依赖文件。

## 证据

- [rusqlite_migration 2.6.0 manifest](https://docs.rs/crate/rusqlite_migration/2.6.0/source/Cargo.toml)、[`goto_up` / `foreign_key_check` / `user_version` 实现](https://docs.rs/crate/rusqlite_migration/2.6.0/source/src/lib.rs)；源码核对，不以 README 的“原子更新”一句替代事务检查。
- [refinery-core 0.9.2 依赖范围](https://docs.rs/crate/refinery-core/0.9.2/source/Cargo.toml)、[SQLite 改表十二步](https://sqlite.org/lang_altertable.html#otheralter)、[user_version](https://sqlite.org/pragma.html#pragma_user_version)、[application_id](https://sqlite.org/pragma.html#pragma_application_id)。
- [现用依赖](../../../src/Cargo.toml)、[foundation 实现](../../../src/crates/hctl2-foundation/src/lib.rs)、[系统边界 §控制面自己的存储](../../design/spec/system.md#控制面自己的存储)。

# tempfile · 独占临时目录

> 状态：实现证据 · 日期：2026-10-06<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-LIB-TEMPFILE<br>
> 对象：tempfile 3.27.0 · 许可证：MIT OR Apache-2.0

## 定位与决定建议

采用 SDK：Agency 的安装探针用 `tempfile` 3.27.0 的 `Builder::tempdir()`，替代进程号加时间戳命名、`create_dir_all` 和手动清理。它已作为间接依赖锁在 `src/Cargo.lock`，本次只增加 Agency 的直接依赖，不升级版本。

## 源码核对

核对 Cargo 缓存中的上游 3.27.0 源码及同版本文档：

- `src/util.rs::create_helper` 生成随机候选名称；已占用的名称不返回给调用者。这里只重新分配名称，不重新启动 Herdr，也不延长执行预算。
- `src/dir/imp/unix.rs::create` 使用 `DirBuilder::create`，不是 `create_dir_all`。目录已存在时创建失败，不会共享已有目录。
- `src/dir/mod.rs::Drop` 清理所持有的目录；提前返回错误也释放目录。探针的 Herdr 与 Launch 先销毁，最后释放目录，避免服务还在用文件时先清理文件。
- `Builder::permissions` 可在创建时指定 Unix 权限。本次用 `0700`，不依赖开发机的 umask。
- 上游 `Cargo.toml` 声明 MSRV 1.63、MIT OR Apache-2.0；本库 rustc 1.98.0 满足。

锁定源码摘要：`32497e9a4c7b38532efcdebeef879707aa9f794296a4f0244f6f69e9bc8574bd`。Cargo 与 Reindeer 仍按仓库原流程生成锁与 Buck 图，不手写第三方目标。

## 候选与边界

| 选法 | 好处 | 代价与边界 |
| --- | --- | --- |
| `tempfile::TempDir` | 独占创建、随机名称、自动清理由社区库实现 | 新增一条直接依赖；版本已在依赖图中 |
| 复用 `storage::nonce` 加标准库目录创建 | 无新直接依赖 | 仍要自己处理独占创建、失败清理与所有权，不采用 |
| 给时间戳加锁或序号 | 可减少同进程碰撞 | 继续自建命名机制；也不能借 `create_dir_all` 证明独占，不采用 |
| 串行测试或加等待 | 改动小 | 不修生产探针的共享目录问题，不采用 |

它不负责会话、派工重试或结果准入。进程被强制杀死时析构不保证发生，遗留临时目录仍是临时数据；不据此声称具备崩溃恢复。

## 证据

- [Builder 3.27.0 文档](https://docs.rs/tempfile/3.27.0/tempfile/struct.Builder.html)：`tempdir`、权限与清理。
- [3.27.0 源码](https://docs.rs/crate/tempfile/3.27.0/source/)：`Cargo.toml`、`src/util.rs`、`src/dir/imp/unix.rs`、`src/dir/mod.rs`。
- [本库 Rust 依赖更新流程](../../../src/third-party/rust/README.md)。

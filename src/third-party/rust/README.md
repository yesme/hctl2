# Rust 第三方依赖

Cargo manifests 和 `Cargo.lock` 是 Rust 依赖的唯一事实源。Reindeer 只读取这份事实并重新生成同目录的 `BUCK`，不得手工维护第二套 crate 版本或依赖关系。

增加外部 crate 并更新 `Cargo.lock` 后，在仓库根目录执行：

```bash
src/build/tools/reindeer buckify
git diff -- src/Cargo.lock src/third-party/rust/BUCK
```

仓库内 Reindeer 启动器通过 DotSlash 固定上游 `v2026.08.24.00` 官方二进制，不在本机编译 Reindeer。crate 的例外构建参数只放在 `fixups/`；`BUCK` 始终由 Reindeer 生成，不手改版本或依赖关系。

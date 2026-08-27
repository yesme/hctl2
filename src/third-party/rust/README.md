# Rust 第三方依赖

Cargo manifests 和 `Cargo.lock` 是 Rust 依赖的唯一事实源。Reindeer 只读取这份事实并重新生成同目录的 `BUCK`，不得手工维护第二套 crate 版本或依赖关系。

当前 workspace 只有本地 crate，因此生成文件还没有第三方目标。增加外部 crate 并更新 `Cargo.lock` 后，在仓库根目录执行：

```bash
src/build/tools/reindeer buckify
git diff -- src/Cargo.lock src/third-party/rust/BUCK
```

仓库内 Reindeer 启动器固定上游 commit、源码归档 SHA-256 和 Rust 1.98.0。首次运行会在用户缓存目录编译这个工具，不把工具二进制或 crate 源码写进 Git 工作树。

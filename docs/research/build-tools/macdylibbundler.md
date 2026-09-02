# macdylibbundler 适用性审计

> 状态：不采用 · 2026-09-02
> 核对版本：[`63105a5`](https://github.com/auriamg/macdylibbundler/tree/63105a5571e0e9a83a8f2c37d0b91f2398b2031b) · MIT

## 定位

HCTL2 的 macOS 外层打包需要从已经构建的 Tuwunel 等可执行文件收集非系统 dylib，复制到发行目录，改写 Mach-O install name 并做 ad-hoc 签名。这是发行产物后处理，不应修改 Tuwunel 或其他 upstream 的源码、Cargo 配方和 feature 定义。macdylibbundler 正是面向该问题的现成候选，因此在保留手写逻辑前必须核对它能否满足当前合同。

## 源码核对

[macdylibbundler](https://github.com/auriamg/macdylibbundler) 递归调用 Apple `otool` 和 `install_name_tool`，复制依赖并修正 install name，默认 ad-hoc codesign；上游 README 也明确把“避免每个移植者各写一套 home solution”作为目标。但当前主线最后提交停在 2022-12-05，没有 GitHub Release 或官方预编译制品，只提供 `make`、Homebrew 和 MacPorts 安装方式。

它与 HCTL2 现有发行合同还有三项不兼容：

1. 一个 `--install-path` 同时用于可执行文件和随包 dylib；HCTL2 当前布局要求前者从 `libexec/hctl2` 指向 `lib/hctl2/vendor`，后者从 vendor 目录指向同目录，二者的 `@loader_path` 不同。
2. `Dependency::mergeIfSameAs` 按原始文件名合并依赖，没有比较内容摘要；HCTL2 必须在两个不同 dylib 使用同一 basename 时失败，而不能静默任选一个。
3. 缺失依赖时工具可能进入交互式路径询问，并以搜索目录补全；Buck action 必须只消费声明输入，不能从宿主机临时寻找未声明库。

## 决定

- 不采用 macdylibbundler，也不 fork/patch 它来适应 HCTL2；为替换少量外层代码维护上游分叉不合算，并违反“默认不修改 upstream”的边界。
- 继续调用 macOS 原生 `otool`、`install_name_tool`、`lipo`、`vtool` 和 `codesign`，但把递归闭包发现、basename 冲突检测和收敛检查收进一份共享 helper，删除原生构建与最终组包之间的重复实现。
- helper 只处理已构建产物。外部项目的源码、锁文件、Cargo/Go/npm 构建图保持原样；如果未来成熟工具同时提供官方制品、非交互声明输入和所需布局语义，再重新评估替换。

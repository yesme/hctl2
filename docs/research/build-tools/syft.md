# Syft SBOM 生成器审计

> 状态：采用为构建依赖 · 2026-09-02
> 固定版本：[`v1.51.1 / 91a0032`](https://github.com/anchore/syft/tree/91a0032987d91b7411b52f6f5c185c5e7f775495) · Apache-2.0

## 定位

Syft 只在 HCTL2 完整发行目标中扫描已经组装的 payload，生成 SPDX 2.3 SBOM；它不进入最终用户安装包，不代替 Buck2 的依赖图，也不要求外部组件改变原生构建。Buck2 直接把固定版本的官方 Linux x86_64、macOS x86_64 和 macOS arm64 归档声明为工具输入，以 SHA-256 校验后执行；action 运行时不再下载工具。

## 上游能力

[Syft](https://github.com/anchore/syft) 原生扫描目录、文件系统和归档，识别 Go、Rust、JavaScript 等多种生态，并直接输出 SPDX、CycloneDX 与 Syft JSON。官方文档把 `spdx-tag-value` 定义为 SPDX 2.3 输出，并允许 `file.metadata.selection=all` 把未归属到包管理器的随包文件及其摘要写入 SBOM。`v1.51.1` Release 提供三种 HCTL2 构建平台的官方压缩包、checksums、签名和工具自身 SBOM；可执行文件没有额外运行时依赖。

| 平台 | 官方制品 | SHA-256 |
| --- | --- | --- |
| Linux x86_64 | `syft_1.51.1_linux_amd64.tar.gz` | `8fcb33017a0dc1058298c923c436d19dfa68ae93968e0b423248542e3afb9fc3` |
| macOS x86_64 | `syft_1.51.1_darwin_amd64.tar.gz` | `0e186ce1d4351ec276126851ca3ff258ed070e93e73574ed64858d4fc2339867` |
| macOS arm64 | `syft_1.51.1_darwin_arm64.tar.gz` | `ac063af3b9874769deb7ea1e6d76841e68f9e3bb50cd654226fc977de65532c1` |

## 可复现性边界

对同一目录连续执行 Syft 时，包与文件发现结果稳定，但 SPDX `Created` 使用当前时间，`DocumentNamespace` 带随机 UUID。HCTL2 因此只在生成后规范化这两个文档级字段：时间取已有 `SOURCE_DATE_EPOCH`，namespace 取版本与目标共同决定的发行地址。组件发现、SPDX 序列化、文件摘要和 relationship 全部仍由 Syft 产生；仓库不再实现 SPDX writer 或许可证映射表。

## 决定

- 删除发行脚本中的手写 SPDX package、ID、license 和 relationship 生成代码。
- 以 Buck2 `http_archive` 声明 Syft，并把当前平台的 `syft` 可执行文件显式传入完整发行 action。
- 扫描最终 payload，启用全部文件元数据；测试验证生成器身份、根包和关键随包文件，而不依赖仓库自造的顶层组件记录。
- Syft 升级是构建供应链变更：复核官方摘要、许可、输出 schema、可复现字段和三平台测试。

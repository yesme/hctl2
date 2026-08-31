# HCTL2 完整发行组装

这个目录是第一方 Buck2 构建与外部子系统原生构建之间唯一的发行边界。`root//packaging/release:complete` 在 Buck action graph 中同时依赖第一方产物和粗粒度外部子系统包；它不进入 Tuwunel、Vikunja、Dagu、Herdr、Cinny 或 Static Web Server 的内部构建图，只消费两侧已经声明并校验的产物约束。

## 产物

每个目标平台生成：

```text
hctl2-<version>-<target>.tar.gz
hctl2-<version>-<target>.tar.gz.sha256
hctl2-<version>-<target>-sources.tar.gz
hctl2-<version>-<target>-sources.tar.gz.sha256
hctl2-<version>-<target>.spdx
hctl2-<version>-<target>.release.tsv
hctl2-<version>-<target>.SHA256SUMS
```

运行包是给最终用户的单一离线安装包；源码伴随包保持外部许可证与可复现性所需的对应源码；SPDX、release manifest、sidecar 和平台命名的 `SHA256SUMS` 记录实际发行内容。GitHub tag 流水线还为全部文件生成 artifact attestation，并在三平台完成后统一发布。

## 本地构建

在 `src/` 用一个目标产出完整发行目录：

```bash
./buck2 build root//packaging/release:complete \
  --target-platforms root//build/platforms:linux_x86_64_gnu \
  --out /absolute/path/release
```

完整用户包验收也是 Buck 测试目标：

```bash
./buck2 test root//packaging/release:complete-test \
  --target-platforms root//build/platforms:linux_x86_64_gnu
```

`complete` 声明第一方导出、外部运行/源码包、`assets/` 中的许可证与用户文档快照、组装器为输入；`complete-test` 声明最终目录、生成的目标 metadata 和生命周期测试体为输入。仓库级 `LICENSE` 与 `docs/usage.md` 是文档事实源，CI 机械校验发行快照与它们一致；Buck workspace 因而保持在 `src/` 内。workflow 只负责选择平台、调用这两个目标、上传、证明和发布，不再编码组包步骤。macOS Intel 与 Apple Silicon 必须在对应架构的 GitHub-hosted Mac 上分别执行同一流程。组装阶段对自建 Mach-O 做 ad-hoc 签名；公开发布所需的 Developer ID 签名和 notarization 属于凭证受控的 CD 阶段，不混入确定性构建输入。

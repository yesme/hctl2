# HCTL2 完整发行组装

这个目录是第一方 Buck2 构建与外部子系统原生构建之间唯一的发行边界。`root//packaging/release:inputs` 在 Buck action graph 中同时依赖第一方产物和粗粒度外部子系统产物；它不进入 Tuwunel、Vikunja、Dagu、tmux、Cinny 或 Static Web Server 的内部构建图，只消费两侧已经声明并校验的产物合同。

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

先在 `src/` 请求完整发行输入。这个目标建立“发行依赖外部构建”的真实图边，不产生另一份 fingerprint 或复制产物：

```bash
./buck2 build root//packaging/release:inputs \
  --target-platforms root//build/platforms:linux_x86_64_gnu
```

然后把已经构建的两侧产物导出到组装工作目录：

```bash
./buck2 build root//packaging/release:first-party \
  --target-platforms root//build/platforms:linux_x86_64_gnu \
  --out /absolute/path/first-party

./buck2 build root//packaging/dependencies:prepared \
  --target-platforms root//build/platforms:linux_x86_64_gnu \
  --out /absolute/path/hctl2-build-cache
```

随后只做外部运行包与源码包的确定性组装，不再重复 bootstrap，再组装完整发行：

```bash
HCTL2_BUILD_CACHE=/absolute/path/hctl2-build-cache \
HCTL2_SKIP_BOOTSTRAP=1 \
HCTL2_DIST_DIR=/absolute/path/dependencies \
  packaging/dependencies/build-package-linux-x86_64.sh

packaging/release/assemble.sh \
  --first-party /absolute/path/first-party \
  --dependencies /absolute/path/dependencies/hctl2-<version>-linux-x86_64.tar.gz \
  --sources /absolute/path/dependencies/hctl2-<version>-linux-x86_64-sources.tar.gz \
  --output /absolute/path/release
```

最终验收必须针对完整用户包，而不是中间依赖包：

```bash
packaging/release/test-package.sh \
  /absolute/path/release/hctl2-<version>-linux-x86_64.tar.gz \
  /absolute/path/release/hctl2-<version>-linux-x86_64-sources.tar.gz
```

macOS Intel 与 Apple Silicon 必须在对应架构的 GitHub-hosted Mac 上分别执行同一流程。组装阶段对自建 Mach-O 做 ad-hoc 签名；公开发布所需的 Developer ID 签名和 notarization 属于凭证受控的 CD 阶段，不混入确定性构建输入。

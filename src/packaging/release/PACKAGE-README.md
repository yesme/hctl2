# HCTL2 离线安装包

这个归档是面向最终用户的完整 HCTL2 安装包，包含 Buck2 构建的第一方命令、四个执行面依赖、Cinny 浏览器客户端、Static Web Server、运行脚本、许可证、校验清单和 SPDX SBOM。

先校验下载目录中的 SHA-256 sidecar，再解压并安装：

```bash
tar -xzf hctl2-<version>-<target>.tar.gz
cd hctl2-<version>-<target>
./install.sh
```

默认安装到 `~/.local`；可以用 `./install.sh --prefix /absolute/path` 指定其他绝对路径。安装器会先验证完整 payload，再以版本目录原子落盘，并只维护 `hctl2-agentd`、`hctl2-tool` 与 `hctl2-services` 三个命令链接。

`USAGE.md` 是完整中文使用说明；`SOURCES.md` 指向同版本、同目标平台的源码伴随包。`payload/share/hctl2/SBOM.spdx`、`first-party.tsv`、`dependencies.tsv` 与 `PAYLOAD.sha256` 可用于审计实际交付内容。

macOS 自动构建产物使用 ad-hoc 签名保证本地内容闭包可执行；面向公开下载的 Developer ID 签名与 notarization 必须在确定性组装完成后的发布环境执行。

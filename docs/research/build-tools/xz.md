# XZ Utils（发行包压缩）

## 决定建议

采用 **XZ Utils 5.8.4**，仅作构建工具。Buck2 下载并校验 pkgx 的 Linux x86_64、macOS x86_64 / arm64 预编译包；HCTL2 不编译 xz，也不要求用户安装 pkgx。安装包和源码伴随包用 `xz -9 -T0`，文件名为 `.tar.xz`，不用 UPX。第三方下载制品的格式不变，包括 HCTL2 已托管的 Tuwunel `.tar.gz`。

依据是 [P2 计划](../../../.memo/design/p2-control-20260906/01-plan.md) §十一的打包压缩裁定。压缩工具、参数、摘要进入 Buck action 输入；不以构建机 PATH 上的 xz 决定输出。

## 审计基线

- 核验日期：2026-09-20。
- 上游：[tukaani-project/xz v5.8.4](https://github.com/tukaani-project/xz/releases/tag/v5.8.4)，2026-09-09 发布。该版本修复了此前版本的内存错误。
- 二进制供应者：**pkgx，不是 XZ 上游官方二进制**。[构建配方](https://github.com/pkgxdev/pantry/blob/b27c0c4748e071ee2bdfd4f9360cfda94faeeed1/projects/tukaani.org/xz/package.yml) 从上游同版本源码运行 configure / make install，关闭 debug 和文档。
- 工具许可见[上游 COPYING](https://github.com/tukaani-project/xz/blob/v5.8.4/COPYING)：xz 与 liblzma 为 0BSD；若编入兼容 getopt_long 则该部分为 LGPL-2.1-or-later，随包 grep/diff/view 脚本为 GPL-2.0-or-later。工具及库不进入 HCTL2 用户安装包。

制品根地址为 `https://dist.pkgx.dev/tukaani.org/xz/`。下表摘要已与供应者同 URL 后缀 `.sha256sum` 及实际下载字节交叉核对。

| 目标 | 根地址下的路径 | SHA-256 |
| --- | --- | --- |
| Linux x86_64 | `linux/x86-64/v5.8.4.tar.xz` | `9a6341f4993aeef3365b700858983fe504d5ac9a0b94126d76d71084f8b7d720` |
| macOS x86_64 | `darwin/x86-64/v5.8.4.tar.xz` | `d93afe9dbb1b19e4e60c336d6f0893e90be3815c025d482a42e6bdd72bb4b63f` |
| macOS arm64 | `darwin/aarch64/v5.8.4.tar.xz` | `45bac764305accc818c9cea5340361a120464a715d8022bd6293ef44f5d71120` |

## 来源与运行检查

macOS 两份制品已在本机执行 `--version`（Intel 版经 Rosetta），均报告 xz 与 liblzma 5.8.4。两份 Mach-O 最低系统均为 macOS 11.0；`otool -L` 显示加载包内相对路径的 liblzma 和系统 libSystem，不依赖 Homebrew。保留 `v5.8.4/bin` 与 `v5.8.4/lib` 层次即可运行。

Linux ELF 依赖 glibc（最高要求 GLIBC_2.17）和 liblzma；制品的 RPATH 残留 `v5.8.4+brewing/lib`，实际目录是 `v5.8.4/lib`。调用 xz 时单独设置 `LD_LIBRARY_PATH` 指向同包的 lib，不修改 ELF、不污染其他构建命令；打包前同时核验工具与库的版本，错版即失败。Linux 原生运行结果由三平台 CI 验证，不以 macOS 的静态检查冒充运行验证。

其他来源的取舍：

- XZ 上游此次发行只有源码和 Windows 二进制，不能直接覆盖当前构建矩阵。
- [Homebrew xz](https://formulae.brew.sh/formula/xz) 5.8.4 未提供 macOS Intel Bottle，且 Bottle 的 Cellar 路径需安装器重定位。
- [conda-forge xz-tools](https://api.anaconda.org/package/conda-forge/xz-tools) 当前可取 5.8.3，但工具与 liblzma 分包，`.conda` 解包还引入 zstd；不为一个压缩器增加另一条安装链。
- pkgx 三包约 416–542 KiB，自带所需 liblzma，Buck 原生 `http_archive` 即可取得。代价是信任第三方构建者，且需处理上述 Linux 库路径；固定摘要不能替代来源审计。

## 参数与验证

[xz 手册](https://tukaani.org/xz/man/xz.1.html) 说明：从 5.4 起，`-T0` 在单核机器仍用多线程编码格式；多线程工作数可以减少而不改变输出，单线程编码格式则不同。因此钉版本并保持 `-9 -T0`，清空 `XZ_DEFAULTS` / `XZ_OPT`；加 `--no-adjust`，避免内存紧张时静默换字典或编码模式。多线程会增加构建内存与时间；`-9` 解压约需 65 MiB。

复用 tar 的既有排序、所有者与时间归一化，不另造归档器。共享函数只负责调用已声明的 xz 和检查版本，下载、SHA-256 与缓存仍由 Buck 原生规则承担。回归测试覆盖版本错配、环境变量干扰、不同多线程工作数的输出一致、往返解压及损坏流拒绝；完整发行测试继续核校验旁文件、安装、生命周期与工具箱。

用户通过系统 `tar -xJf` 解包，不需要构建工具路径、Rust 或 pkgx。macOS 的系统 tar 自带 xz 解码；GNU tar 环境需要运行端 xz 解码器（通常由 `xz-utils` 提供），不要求它与构建端同版本。

## 复核记录

### 2026-09-20 · PR #278 评审补记

- 构建启动也有解码前提：本库所用 Buck Prelude 的 `prelude//http_archive/unarchive.bzl` 中，`_TAR_FLAGS` 为 `tar.xz` 选择 `-J`，`_unarchive_cmd` 调用构建机的 `tar`，先解开 pkgx 工具包。macOS 系统 tar 自带解码能力；GNU tar 环境需可调用的 xz 解码器。这个启动用解码器不决定安装归档的压缩字节；后续压缩仍只用已钉定、同时核验工具与 liblzma 的 5.8.4，满足裁定的 ≥ 5.4，不要求启动用解码器同版。
- 多线程一致性回归比较同一份 3 MiB 输入、同一 1 MiB 块大小下的 `-T+1` 与 `-T2`；它不是在不同核数机器上逐字节比较完整安装包。三平台完整安装测试验证的是归档完整性与安装、服务生命周期，不应将它报告为完整包的跨机字节一致性证明。

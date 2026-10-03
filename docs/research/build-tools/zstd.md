# Zstandard（发行包压缩）

## 决定建议

采用 **Zstandard 1.5.7**，仅作构建工具。发行包与源码伴随包用 `zstd --ultra -22 --long=27 -T0`，文件名为 `.tar.zst`；pull request 的验证构建用 `-12`（`HCTL2_ZSTD_PRESET=fast`），与 `release` 一样只安装、测试、不发布。

理由是压缩只发生一次、解压发生在每次安装：用编码时间和体积换解码时间。所有者 2026-10-03 判：「我觉得还是换一下吧，zstd 最高档位。毕竟压缩就一次，但解压会有很多次。」实测代价与收益见下。

工具在打包 action 里从上游源码现编（`root//build/tools:zstd-bin`），不随包分发、不进用户运行环境；钉定的 **xz 5.8.4 保留**，只用于解开 Gitea 上游的 `.xz` 下载制品（上游只发 `.xz`）。上游下载格式与托管的 Tuwunel `.tar.gz` 不变。

## 审计基线

- 核验日期：2026-10-03。
- 上游：[facebook/zstd v1.5.7](https://github.com/facebook/zstd/releases/tag/v1.5.7)，官方发布资产 `zstd-1.5.7.tar.gz`。
- 摘要：`eb33e51f49a15e023950cd7825ca74a4a2b43db8354825ac24fc1b7ee09e6fa3`，与上游同 Release 的 `.sha256` 旁文件交叉核对一致。
- 许可：[LICENSE](https://github.com/facebook/zstd/blob/v1.5.7/LICENSE)（BSD-3-Clause）或 [COPYING](https://github.com/facebook/zstd/blob/v1.5.7/COPYING)（GPLv2）双许可；工具不进入 HCTL2 用户安装包，构建产物里随附两份许可原文。

**为什么从源码编，而不是钉预编译制品。** 上游不发 macOS / Linux 二进制（Release 里只有源码与 Windows 制品）。pkgx 的 zstd 可直接下载，但 `otool -L` 显示它依赖三个同门包（`@rpath/zlib.net/v1/lib/libz.dylib`、`@rpath/tukaani.org/xz/v5/lib/liblzma.dylib`、`@rpath/lz4.org/v1/lib/liblz4.dylib`），独立使用时库解析失败；为它钉四包 × 三平台会引入脆弱布局，且把 xz 又拉回依赖链。源码路线只有一个上游制品、一个摘要，构建 4 秒（本机 10 核，`make -j zstd`），产出的 CLI 只链接系统 `libz` 与 libc。

代价：打包 action 需要构建机上有 `make` 与 C 编译器。三种发布宿主（Linux x86_64、macOS arm64 / x86_64）都具备；`src/packaging/dependencies/README.md` 里「Linux 构建不需要 C toolchain」一句已按此收窄（只对解压与组装成立，压缩工具要现场编）。

## 来源与运行检查

- 本机（macOS arm64）由 `root//build/tools:zstd-bin` 构建并运行，`zstd --version` 报告 `*** Zstandard CLI (64-bit) v1.5.7, by Yann Collet ***`；`require_pinned_zstd` 按这条完整字符串核对版本，错版即失败。
- 沙箱与 buck2 缓存：目标 `cacheable`，两次构建命中缓存、输出目录含 `bin/zstd` 与 `LICENSE` / `COPYING`。
- Linux 与 macOS x86_64 由三平台 CI 验证；工具字节不必跨机一致（它只在构建机上运行），但同一版本的编码结果跨平台一致。

## 参数与验证

- **档位、窗口与 LDM**：`--ultra -22 --long=27`。`--long` 不是冗余写法：它打开 long distance matching（上游 `programs/zstdcli.c` 的 `--long` 分支置 `ldmFlag`）并声明 27 位窗口，有没有它、以及走不走多线程路径，都会改变输出字节——898 KB 的 tar 上，无 `--long` 为 119,798 B、`--long=27` 为 119,729 B，`--single-thread` 又与 `-T1` 不同。删掉它等于换一套发行字节，所以它属于钉定的一部分。少写 `--ultra` 时 zstd 直接报错而不是降级，这是刻意的。
- **worker 数不进字节**：分段（job）大小由 `ZSTDMT_computeTargetJobLog`（`lib/compress/zstdmt_compress.c`）决定，与 worker 数无关；分两支——开了 `--long` 走 `MAX(21, cycleLog(chainLog, strategy) + 3)`，否则走 `MAX(20, windowLog + 2)`，再截到 `ZSTDMT_JOBLOG_MAX`（64 位下 30）。我们两组参数都带 `--long`，所以永远走前一支；`cycleLog = chainLog - (strategy ≥ btlazy2 ? 1 : 0)`，而 `chainLog` 随输入大小自适应（同一命令 `-22 --long=27`：1 MiB → clog 21、24 MiB 及以上 → clog 26、48 MiB 及以上 → clog 27），所以段大小是「参数 × 载荷」的函数：发行档 163–172 MB 落在 512 MiB 段，必然是单段，这一档上多 worker 只影响速度（665 MB payload 实测：`-T1` 214 s、`-T0` 159 s）。要真正观测「worker 数不进字节」，得在能跨段的低档上验——回归测试用 `-3 --long=21`（2 MiB 段）压 16 MB 文本：`-T1` / `-T3` / `-T0` 字节相同。同参数的 `--single-thread` 字节不同，只说明 MT 与 ST 是不同代码路径，不是跨段的证据（单段下它照样不同）。
- **多线程必须在构建里**：`zstd-build.sh` 记录 make 日志，出现 `without multithreading support`（pthread 探测失败时的静默降级）即失败。降级后版本串不变、发行字节却会变，`require_pinned_zstd` 覆盖不到这一点。
- **环境隔离**：`run_zstd` 用 `env -u ZSTD_CLEVEL -u ZSTD_NBTHREADS` 取消调用方环境，`check_zstd.sh` 用「毒化 PATH + 设置这两个变量」断言输出字节不变。
- **回归测试**（`src/build/tests/check_zstd.sh`，`root//build/tests:zstd_test`）：缺省预设与显式 `--ultra -22 --long=27` 字节相同；`fast` 与显式 `-12` 字节相同、且在可压缩样本上与 `release` 不同；未知与空预设被拒；往返解压一致；截断流被拒；版本错配与缺工具均失败。
- **用户侧解包**：macOS 系统 tar（libarchive 3.7.4）内置 zstd 解码，`tar --zstd -xf` 与自动识别都可用（已实测）；GNU tar 环境需要 `zstd` 命令，与换用前需要 `xz` 是同一类要求。

## 实测对照（2026-10-03）

样本一：macOS arm64 运行包的全部 10 个锁定制品（`lock.json` 逐个校验 SHA-256），按 `platforms/macos/package.sh` 的方式打成 665 MB 的 tar。

| 方案 | 压缩后 | 相对 xz | 编码 | 解码（整包解包） |
| --- | --- | --- | --- | --- |
| xz -9 -T0（换用前） | 163.8 MB | — | 64.5 s | 5.9 s |
| zstd --ultra -22 --long=27 | 172.3 MB | **+5.2%** | 158–161 s | 0.7 s |
| zstd -19 -T0 | 189.6 MB | +15.8% | 37.7 s | — |

样本二：Silesia 语料（212 MB tar，单线程）：xz -9 = 48.8 MB，zstd -22 = 52.3 MB（+7.2%）。
样本三：源包近似体（117 MB，内容全是上游 source `.tar.gz`）：两者几乎相同（115.1 vs 115.2 MB），zstd 编码快一倍（38.8 s → 17.2 s）。

读法：体积代价落在下载侧（+5.2%，约 8.5 MB），收益落在每次安装的解包（约 5 秒）与 CI 之外的解码；编码变慢落在 CI（xz -T0 吃满 10 核，zstd -22 因窗口大、job 少而几乎不并行）。这是所有者按「一次编码、多次解码」拍板的取舍，数字在这里留档。

## 复核记录

### 2026-10-03 · 换用 zstd（所有者裁定）

- 所有者读实测对照后裁定换用最高档位；本文件与 [`xz.md`](./xz.md) 的复核记录、`README.md` 索引行、打包 README 与使用文档同批更新。
- 同批改动：`src/build/tools/zstd.bzl`（源码钉定）、`zstd-build.sh`（构建脚本，含 make 日志的线程断言）、`BUCK`（`zstd-bin` 目标）、`common/action.sh`（`run_zstd` / `require_pinned_zstd` / `zstd_preset_flags` / `compress_archive`）、两个 `defs.bzl`（工具输入与预设配置 `hctl2.zstd_preset`）、归档名与解包路径（`package.sh`、`assemble.sh`、三个 `test-package.sh`）、`src/apps/cli/tests/room.rs` 与 `src/packaging/release/BUCK`（release 测试按 `.tar.zst` 找包、解包走钉定 zstd）、`docs/usage.md`、打包 README ×4、`src/crates/{chat,project}/README.md` 的预设开关与 `release.yml`。
- 归档名由 `.tar.xz` 改为 `.tar.zst`，`.sha256` 旁文件同批改名；解包一律经钉定 zstd（`run_zstd -dc … | tar -xf -`），不依赖宿主 PATH 上的 tar 解码能力。
- 三平台 CI（`Buck2` 与 `Complete package` 四个 job）已跑通：Linux x86_64 与 macOS arm64 上现编工具、打整包（release 预设）并跑完离线安装与服务生命周期；macOS x86_64 的发布构建按既定排期在 tag 流水线验证。

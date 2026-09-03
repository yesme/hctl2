# python-build-standalone · Buck2 宿主 Python 钉定

> 状态：采用为开发依赖 · 2026-09-04
> 固定制品：[cpython-3.12.14+20260901](https://github.com/astral-sh/python-build-standalone/releases/tag/20260901) `install_only_stripped` 变体 · 制品含 CPython（PSF 许可）与随包工具（MIT 等）

## 定位

它只解决一件事：给 Buck2 的宿主 Python 动作提供一个钉死版本的解释器，不依赖宿主机 `/usr/bin/python3`。直接动因是 issue #157：Buck2 `2026-08-22` prelude 的 Cargo build-script C/C++ shim（`from_any_dir.py`）使用 `pathlib.Path.relative_to(..., walk_up=True)`，该参数要求 Python 3.12；macOS 系统 Python 停在 3.9.6，`libsqlite3-sys` 的上游 build script 因此无法经 shim 编译 C，仓库被迫手工复刻编译开关（该复刻由本对象一并拆除）。它是开发/CI 构建工具，不进入用户发行包，最终用户运行环境继续不依赖 Python。

## 上游能力

[python-build-standalone](https://github.com/astral-sh/python-build-standalone)（Astral 维护，前 indygreg 项目）发布预编译 CPython 发行版，是 uv、rye、maturin 等工具链获取 Python 的事实标准来源。`install_only_stripped` 变体解压即用（无安装步骤、已去符号）：归档根为 `python/`，入口 `python/bin/python3.12`，标准库以相对路径定位，可整体搬移。每个 Release 附全部制品的摘要。

本仓库三个构建平台的制品与 SHA-256：

| 平台 | 制品 | 大小 | SHA-256 |
| --- | --- | --- | --- |
| Linux x86_64 | `cpython-3.12.14+20260901-x86_64-unknown-linux-gnu-install_only_stripped.tar.gz` | 34,143,368 | `72748da13197c1fb161e3afeef20a6a385ff24f2165e6e2758e47008e7faba4c` |
| macOS arm64 | `cpython-3.12.14+20260901-aarch64-apple-darwin-install_only_stripped.tar.gz` | 24,981,445 | `81a359f1cfadd4da11766534c5913791cea55f26e1bb902cacd2a531bb1e4b2b` |
| macOS x86_64 | `cpython-3.12.14+20260901-x86_64-apple-darwin-install_only_stripped.tar.gz` | 24,686,153 | `65b195c9cedc1fef6767f044f9822069adbd1bd9204d424ece4628776fdc04bb` |

选择 3.12 而非更新大版本：prelude 的硬要求是 ≥3.12；3.12 是满足要求的最保守版本，生态兼容面最宽。实测（2026-09-04，macOS arm64）：归档解压后 `python/bin/python3.12 --version` 输出 `Python 3.12.14`，curl 下载的制品无 quarantine 属性，Gatekeeper 不拦截。

## 接线机制（实测定稿，含两个被证伪的中间方案）

Buck2 action 环境是封闭的：**不含 PATH**，prelude 默认 `python_bootstrap` toolchain 的裸名 `python3` 经 execvp 默认搜索落在 `/usr/bin/python3`（3.9.6）——这正是 #157 的故障形态。因此：

- ~~方案一（证伪）：启动器把 DotSlash 目录前置到 PATH~~——PATH 不进 action 环境，无效。
- ~~方案二（证伪）：`xcrun --find clang` 的工具链裸二进制作为 C 编译器~~——它不像 `/usr/bin/clang` 那样自动注入活跃 SDK，`stdio.h` 找不到。
- **定稿**：`src/buck2` 启动器每次以 `host-bin/python3 -c 'import sys; print(sys.executable)'` 解析出钉定解释器的**真实二进制绝对路径**（dotslash 缓存内），以 `--config hctl2.python=<abs>` 注入；`src/build/toolchains/BUCK` 的 `system_python_bootstrap_toolchain` 读该 config 作为 interpreter。无 config 时回退裸名 `python3`（保持旧行为）。

同一趟修复还拆掉了同一 shim 链路上的另外三个阻塞（缺一仍不可用）：

1. **`os.execl` 不搜 PATH**：`from_any_dir.py` 以 `os.execl(cc[0], ...)` 重入 C 编译器，而 system 工具链给的是裸名 `clang`——对 execl 永远 ENOENT。修法：启动器解析绝对路径注入 `hctl2.cc/cxx/ar`，toolchain 读 config 覆盖。macOS 上取 `/usr/bin/clang`（xcrun shim，自动注 SDK；Xcode.app 内的裸二进制不带 `-isysroot` 找不到系统头），Linux 用 `command -v`。
2. **`env!("CARGO_MANIFEST_DIR")` 编译期展开失败**：真 Cargo 编译 build script 时设置该变量，Buck 不设。修法：fixup `cargo_env = ["CARGO_MANIFEST_DIR"]`（与 proc-macro2 fixup 同机制）；该值只喂 `cargo:include` 打印，对 `bundled_bindings` 惰性。
3. **cc-rs 运行期要求 `OPT_LEVEL`**：prelude 仅在工具链带字面 `-Copt-level=N` 时注入（本仓库未设）。修法：fixup `[buildscript.run] env = { OPT_LEVEL = "2" }`（2 为 SQLite 上游推荐档）。注意 TOML 约束：`buildscript.run = true` 布尔速记与 `[buildscript.run]` 表二选一，表形态隐含 run=true。
4. **链接缺 `sqlite3_*` 符号**：build.rs 的 `cargo:rustc-link-lib/-search` 输出默认不被翻译。修法：fixup `[buildscript.run] rustc_link_lib = true`、`rustc_link_search = true`。

## 决定

- `src/build/tools/host-bin/python3`：三平台 DotSlash 清单，摘要锁定官方制品；`src/buck2` 注入解析出的绝对路径与 `hctl2.python` config。
- `libsqlite3-sys` fixup 缩为一小块声明（run=true + `cargo_env` + `OPT_LEVEL` + `rustc_link_lib/search`），删除手工 `cxx_library`（20 个转录编译宏）与 bindings 拷贝 genrule——编译开关回归 upstream build.rs，随版本升级不再漂移。
- 回归锚点已验：`hctl2-foundation` 的 JCS 官方向量、FTS5、Online Backup 测试在翻转后保持绿（本地 macOS arm64：first-party 全套 139 命令、docs 13 项、clippy、ShellCheck `--severity=error` 全绿；Linux 由 CI 矩阵验证——ubuntu-24.04 runner 预装 clang）。
- **已知代价**：注入的解释器绝对路径含 dotslash 缓存哈希，逐机器不同——buildscript 相关 action 的 key 随机器变化，跨机 REAPI 缓存对这些（少数）动作不命中；C 编译/链接动作的 key 仍稳定（编译器路径 `/usr/bin/clang` 逐机一致）。
- **上游报告项**（待提交 issue，不阻塞）：① prelude `from_any_dir.py` 使用 3.12 API 但未声明最低 Python 版本；② `os.execl` 不搜 PATH，与 system_cxx_toolchain 的裸名编译器组合必然失败。修复前本仓库以 config 注入绝对路径为既定机制。
- 升级路径：换版本 = 更新 DotSlash 清单中的版本号、大小与摘要，重跑三平台测试；属构建供应链变更，按仓库纪律整体审阅。

## 证据

- 上游：[Release 20260901 资产列表](https://github.com/astral-sh/python-build-standalone/releases/tag/20260901)（摘要取自各资产的官方 digest）、[仓库 README（变体说明：install_only 解压即用）](https://github.com/astral-sh/python-build-standalone/blob/main/docs/installing.md)
- Buck2：[`prelude/toolchains/python.bzl` @ 2026-08-22](https://raw.githubusercontent.com/facebook/buck2/2026-08-22/prelude/toolchains/python.bzl)（`system_python_bootstrap_toolchain` 的 interpreter 默认 `python3`）、[`prelude/rust/cargo_buildscript.bzl` @ 2026-08-22](https://raw.githubusercontent.com/facebook/buck2/2026-08-22/prelude/rust/cargo_buildscript.bzl)（CC/LD/AR shim 经 `from_any_dir` 包装、OPT_LEVEL 仅从 `-Copt-level=` 推导）、[`prelude/rust/tools/from_any_dir.py` @ 2026-08-22](https://raw.githubusercontent.com/facebook/buck2/2026-08-22/prelude/rust/tools/from_any_dir.py)（`walk_up` 与 `os.execl`，2026-09-04 检查上游 main 仍未修）
- Reindeer：[MANUAL（fixup 语法：cargo_env、[buildscript.run] env/rustc_link_lib）](https://github.com/facebookincubator/reindeer/blob/v2026.08.24.00/docs/MANUAL.md)、libsqlite3-sys 0.38.2 `build.rs`（bundled 分支经 cc 编译 + `cargo:rustc-link-lib` 输出）
- 本库：[issue #157](https://github.com/yesme/hctl2/issues/157)、[SQLite bundled 复核裁定](../libs/sqlite-online-backup.md#复核记录bundled-策略裁定2026-09-04issue-158)、[src/build/README.md DotSlash 工具段](../../../src/build/README.md)

## 复核记录

- 2026-09-04 上游报告项已提交（以所有者身份，发前核对 `main` 两处代码未动、无既有 issue）：① `walk_up` 未声明最低 Python 版本 → [facebook/buck2#1490](https://github.com/facebook/buck2/issues/1490)，建议改用 `os.path.relpath` 一行去掉 3.12 依赖；② `os.execl` 不搜 PATH 与 `path_clang_tools` 裸名错配 → [facebook/buck2#1491](https://github.com/facebook/buck2/issues/1491)，建议 `shutil.which` 或 `os.execvp`。两条互相引用，都表示可提 PR。上游修复合入并升级 prelude 之后，再评估是否撤掉启动器的绝对路径注入（撤掉即消除「buildscript action 跨机缓存不命中」这项已知代价）。

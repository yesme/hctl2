# Buck2 项目根：仓库根还是 `src/`

> 日期：2026-09-02<br>
> 状态：Informative 研究备忘录；结论已由所有者裁决并落地（本文即落地 PR 的一部分）<br>
> 对象：Buck2 项目根（project root）与 cell 布局；对照 Bazel `rules_pkg`、GoReleaser 的文档打包做法<br>
> 证据钉：Buck2 二进制以 `src/build/tools/buck2-bin` 的 DotSlash 清单锁定（`buck2 --version` 报 `02853c21…`）；官方文档 [.buckconfig](https://buck2.build/docs/concepts/buckconfig/)、[Key Concepts](https://buck2.build/docs/concepts/key_concepts/)；[rules_pkg 文档](https://github.com/bazelbuild/rules_pkg/blob/main/docs/latest.md)；[GoReleaser archive 文档](https://goreleaser.com/customization/archive/)。

## 结论先行

1. **业界的做法是「文档留在原地，打包规则把它当输入」，没有人提交派生副本。** Bazel/Buck2 单体仓库的构建根就是仓库根，`docs/` 只是一个普通 package；`rules_pkg` 的 `pkg_files` 直接把 `README.md`、`LICENSE` 当 `srcs` 装进包；GoReleaser 默认把仓库根的 `README*`、`LICENSE*`、`CHANGELOG*` 塞进归档，要带 `docs/*` 就在 `archives.files` 列一行。「提交快照 + CI 校验新鲜度」是给必须提交生成物的场合用的兜底（`go generate` 加 `git diff --exit-code`），用在一份 markdown 上是拿兜底当主路。
2. **HCTL2 之前奇怪的不是「文档进了代码」，是「构建根在 `src/` 而文档在它上面」。** Buck2 的项目根就是放 `.buckconfig` 的目录，cell 路径都相对它写；官方文档没有明说 `..` 不行，但本仓库 2026-08-29 实测写 `..` 报「expected a normalized path」（记录在已删除的 `materialize_repo_tree.sh` 头注里）。于是根目录文档对 Buck 不可见，只能靠两座桥：`src/packaging/release/assets/` 的字节级快照加 CI `cmp`，以及 `materialize_repo_tree.sh` 把文档复制进 cell 供检查器读。
3. **裁决：项目根挪到仓库根，`src/` 继续叫 `root` cell，所有既有标签不变。** Buck2 允许 cell 是子目录（官方例子就是 `./third-party/...`），也不要求项目根所在的 cell 叫 `root`。`.buckconfig` 里 `repo = .`、`root = src`，加一行 `target:repo//...->prelude//platforms:default` 让新 cell 的目标有默认平台。实测：`repo//:usage`、`repo//:LICENSE`、`repo//:docs_tree`（182 个文件，含 `.memo/**`）与全部 `root//...` 目标在同一图里解析，`buck2 audit cell` 列出五个 cell。
4. **两座桥同时退役。** 发行的许可证与用户文档改由 `repo//:LICENSE`、`repo//:usage` 导出，`src/packaging/release/assets/` 与三个 workflow 里的 `cmp` 步骤删除；文档检查改吃 `repo//:docs_tree`，五个检查器的参数从 cwd 相对路径改成 `$(location ...)` 宏，`materialize_repo_tree.sh` 删除。文档链进 `src/` 的两处（`src/testing/tmux/*.py`）由链接检查器对工作树核对，不再靠 `git ls-files` 生成的清单。
5. **代价与注意事项。** 项目根变大，Buck 的文件监视范围覆盖整个仓库，`[project] ignore = .git` 把版本库排除；`buck-out/` 从 `src/` 挪到仓库根（`.gitignore` 已覆盖）；`.buckconfig.local`（远端 cache 配置）也挪到仓库根；`src/build/ci/affected-targets` 给 BTD 的变更路径改为仓库相对路径。2026-08-29 的「Buck workspace 严格位于 `src/`」判断被本裁决替代，理由是它以边界整洁换来了两份需要维护一致的副本，且与 Bazel/Buck2 的标准布局相反。

## 机制对照

| 方案 | 文档进包 | 文档检查读文档 | 代价 |
| --- | --- | --- | --- |
| 构建根在 `src/`（2026-08-29 至 09-02） | 提交 `assets/USAGE.md`、`assets/LICENSE` 快照，CI `cmp` 校验 | `materialize_repo_tree.sh` 复制进 cell（gitignored） | 两份副本；改 `docs/usage.md` 忘改快照即红（PR #138 踩过）；一个 shell 旁路 |
| 构建根在仓库根，`src/` 为 `root` cell（本裁决） | `repo//:usage`、`repo//:LICENSE` 直接导出 | `repo//:docs_tree` 文件组 | 监视范围变大（`ignore = .git`）；`buck-out` 与 `.buckconfig.local` 位置变动 |
| 把源挪进 cell（未采纳） | 单源在 `src/packaging/release/assets/` | 仍需复制 | 用户文档住到打包目录下，写作指南路由表要改 |

## 复核记录

- 2026-09-02 首发，随落地 PR 一并提交。
- 2026-09-20 内部命名复核：Buck 的 cell/package 路径已区分内部目标；Cargo 的[目标命名](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-name-field)允许 package、库与 Binary 分别命名，[Buck `rust_binary`](https://buck2.build/docs/prelude/rules/rust/rust_binary/)也独立声明 Binary 目标。本仓库四个第一方 package 均继承 `publish = false`，因此目录、package 与库目标采用 `tool`、`facts`、`foundation`、`store`；`tool` 用显式 `[[bin]]` 保留对外的 `hctl2-tool`。不新增命名转换脚本或依赖，Cargo.lock 只调整第一方名称，三方版本与摘要不变。构建根与 cell 布局不变，当前路径见 [src/README](../../../src/README.md)；本条更新内部目标名称，不回写上文项目根迁移时的历史标签。

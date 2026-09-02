# Buck2 Change Detector 源码审计

> 类别：⑥ 机械后端与基础设施 · 证据编号：E-TOOL-BTD<br>
> 状态：采用为依赖 · 2026-08-31<br>
> 固定版本：[`2026-08-20 / 345497d`](https://github.com/facebookincubator/buck2-change-detector/tree/345497d774083ae25421fed1dedb19edf4d4efd2) · MIT OR Apache-2.0

## 结论

采用 Meta 的 Buck2 Change Detector（BTD）做 `src/` target 影响范围计算，不自行实现 `owner + rdeps`。BTD 只回答哪些 target 可能受影响；测试关系、验证类别、目标平台与 required gate 仍由本仓库的 Buck 图和 CI 策略定义。

上游仓库包含 `targets`、`btd` 和总入口 `supertd` 三个二进制，但日期 release 只发布 `btd`。HCTL2 因此直接消费上游官方 `btd` DotSlash 制品，由 Buck2 原生 `targets --streaming --json-lines` 导出修改前、修改后两份图；不为取得 `supertd` 自行编译或维护另一条供应链。

## 源码核对

| 关注点 | 固定 commit 中的实现 | 对 HCTL2 的含义 |
| --- | --- | --- |
| Git 输入 | [`status.rs`](https://github.com/facebookincubator/buck2-change-detector/blob/345497d774083ae25421fed1dedb19edf4d4efd2/btd/src/sapling/status.rs) 解析 `git diff --name-status`；rename 展开为旧路径删除和新路径新增，copy 记新路径新增，冲突状态直接报错 | 不需要另写 rename/delete 修补逻辑；Git 解析失败进入全量回退 |
| 图的局部重算 | [`rerun.rs`](https://github.com/facebookincubator/buck2-change-detector/blob/345497d774083ae25421fed1dedb19edf4d4efd2/btd/src/rerun.rs) 处理 BUCK 文件增删、传递 `.bzl` import、PACKAGE、glob 与 buckconfig/deployment 变化 | 覆盖本库最容易由路径正则漏掉的图变化 |
| 直接与递归影响 | [`diff.rs`](https://github.com/facebookincubator/buck2-change-detector/blob/345497d774083ae25421fed1dedb19edf4d4efd2/btd/src/diff.rs) 比较 input、target hash、CI labels、rule import 和 package value，再沿反向依赖传播 | 输出是保守候选集；CI 再按 `ci:fast`、`ci:platform` 等本库 labels 选择验证目标 |
| 图错误 | [`check.rs`](https://github.com/facebookincubator/buck2-change-detector/blob/345497d774083ae25421fed1dedb19edf4d4efd2/btd/src/check.rs) 检查新 package error、删除后仍被引用的 target，以及新增 dangling `deps` / `tests` | selector 同时提供图完整性检查；失败不能当作“无目标” |
| Buck 导图参数 | [`run.rs`](https://github.com/facebookincubator/buck2-change-detector/blob/345497d774083ae25421fed1dedb19edf4d4efd2/td_util/src/buck/run.rs) 固定 streaming、keep-going、unconfigured hash、JSON lines、imports、labels 与 tests 属性 | HCTL2 使用与固定版本相同的 Buck 原生命令；升级 BTD 时一并复核参数 |
| 职责边界 | [仓库总 README](https://github.com/facebookincubator/buck2-change-detector/blob/345497d774083ae25421fed1dedb19edf4d4efd2/README.md) 明确不负责后续 build/test，Meta 内部另由 Citadel 解释 labels | 不把 BTD 当成 CI policy engine，也不复制未开源的 Citadel |

代码入口使用 `#![forbid(unsafe_code)]`。BTD 的输出由输入图与变更列表确定；显式提供 `--base` 和 `--diff` 时不会自行读取其他 target graph。HCTL2 同时启用 `--check-dangling` 与 `--buckconfig-select-all`，对配置修改采用全图保守选择。

## 制品与接入

上游 [`2026-08-20` release](https://github.com/facebookincubator/buck2-change-detector/releases/tag/2026-08-20) 提供官方 DotSlash 清单：Linux x86_64 musl 压缩包 2,179,317 bytes，macOS arm64 1,666,871 bytes，macOS x86_64 1,841,435 bytes。HCTL2 原样固定该清单的 BLAKE3 digest；BTD 是开发/CI 工具，不进入最终用户安装包。

一次 PR 选择按以下顺序进行：

1. Git 输出 base 到 head 的 `--name-status`；
2. base worktree 与当前 worktree 各由 Buck2 导出一份 unconfigured target graph；
3. BTD 输出受影响 target 及其 labels；
4. CI 只把属于当前验证类别的 target 交给 `buck2 test --build-default-info`；
5. 任一步失败，选择结果改为本批次定义的全量 first-party targets，不允许空结果放行。

2026-08-31 本地实测：`main` 图为 581 行、156,788 bytes，本次增加测试关系和 labels 后为 590 行、166,387 bytes；BTD 处理约 0.32 秒。对本次 BUCK/BZL 变更，39 个受影响 target 中只有 12 个进入 first-party 验证；合成的单个 `hctl2-tool` Rust 文件变更影响 9 个 target，其中 6 个进入验证（本模块 build/test/Clippy 加 first-party release）。空 diff 正确输出零 target；删除、改名与 selector 失败回退继续由 CI 约束覆盖。

## 不采用与升级边界

- 不采用 `supertd`：它是同仓库的便利总入口，但上游没有发布对应官方二进制；为它自行编译会增加 Rust 供应链和维护面，且 `btd` 已覆盖本库需要的判定功能。
- 不用 BTD 选择根目录文档：Buck cell 位于 `src/`，文档类别继续由 Git attribute 判定。（2026-09-02 复核：项目根已挪到仓库根，文档成为 `repo//:docs_tree` 的输入；文档 profile 的选择仍由 `.gitattributes` 判定，未改用 BTD。见[Buck2 项目根](./buck2-project-root.md)。）
- 不按 `depth` 截断反向依赖：当前图很小，截断会用正确性换时间而没有实际收益。
- 升级日期 tag 时必须复核导图参数、Git parser、buckconfig 行为、官方 DotSlash digest，并重跑本库的 selector 失败回退约束。

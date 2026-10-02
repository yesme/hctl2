# GitHub Actions 增量重验证

> 类别：⑥ 机械后端与基础设施 · 证据编号：E-TOOL-GHA-REVALIDATION<br>
> 状态：采用平台原生机制 · 2026-08-31

## 结论

保留 `main` 的 strict branch protection，但不把分支前移后的每个 SHA 都当成第一次验证。PR head 以快进方式增加提交时，如果紧邻的旧 head 已有同一 workflow 的成功结果，Code 与 Release 只验证旧 head 到当前 test merge 的增量；Buck target 影响范围继续由 Buck2 Change Detector（BTD）计算。旧结果缺失、查询失败或提交历史被重写时，回退到完整 PR diff。

这不是另一套构建缓存，也不产生平行 fingerprint。复用证据只有 Git commit SHA、GitHub 已保存的 workflow 结论和 Buck 图；required check 仍在当前 head 上重新产生。

## 平台机制

| 机制 | GitHub 原生能力 | HCTL2 用法 |
| --- | --- | --- |
| strict required checks | [strict 模式要求 PR 在合入前包含最新 base](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches) | 保留 strict；优化更新后的重验证内容，不放松合入条件 |
| 精确更新区间 | [`github.event` 是触发 workflow 的完整 webhook payload](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#github-context)，`pull_request/synchronize` 提供更新前后的 head | 用 `before` 和 `pull_request.head.sha` 证明 PR 提交链；验证上界仍是 GitHub 检出的当前 test merge，覆盖最新 base 的兼容性 |
| 旧验证证据 | [List workflow runs for a workflow](https://docs.github.com/en/rest/actions/workflow-runs#list-workflow-runs-for-a-workflow) 可按 workflow 文件、`head_sha`、event 和状态查询 | 分别确认旧 head 上 Code 或 Release workflow 存在成功 run；只授予 `actions: read` |
| PR 分支前移 | [Update a pull request branch](https://docs.github.com/en/rest/pulls/pulls#update-a-pull-request-branch) 把最新 base merge 进 PR head | 使用 merge 更新，保留旧 head 为新 head 的祖先；rebase/强推不能继承旧结果 |
| 影响范围 | BTD 比较两份 Buck 图并沿反向依赖传播，见 [`buck2-change-detector.md`](./buck2-change-detector.md) | 增量模式传入 `before...after`；selector 失败仍由现有全量目标回退接管 |

GitHub 官方也给出了以 `GH_TOKEN` 调用 `gh` 的 workflow 示例，并建议按最小权限配置 `GITHUB_TOKEN`；HCTL2 因此直接使用 runner 已有的 GitHub CLI 和仓库固定的 jq，不引入新的 Action 或常驻服务。参考：[在 workflow 中使用 `GITHUB_TOKEN`](https://docs.github.com/en/actions/tutorials/authenticate-with-github_token)、[workflow permissions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#permissions)。

## 判定顺序

一次 `synchronize` 只在以下条件全部成立时收窄验证区间：

1. 事件给出旧 head 与当前 head；
2. Git 证明旧 head 是当前 head 的祖先；
3. 同一 workflow 在旧 head 上至少有一次成功的 `pull_request` run。

成立后，Code 的路径分类、Cargo hygiene、文档检查和 BTD 都读取旧 head 到当前 test merge 的增量；Release 的完整包路径分类也读取同一区间。新增提交影响某个既有 target 时，BTD 会把它及受影响的反向依赖重新选出；没有进入相应依赖闭包的变化不启动三平台重构建。Code/Release workflow 自身或 target selector 变化仍命中现有的全量策略。

以下情况不继承：PR 第一次打开或重新打开、旧 workflow 未成功、API 不可用、旧对象不可达、rebase/强推造成非快进历史。它们全部回到 base 到当前 test merge 的完整 PR diff。这个回退允许外部 CI 继续独立工作，也避免把 GitHub 可用性错误解释成“没有影响”。

## 仓库实测

GitHub API 对本仓库 PR #122 的连续运行返回真实 PR head，而不是临时测试 merge SHA；旧 head `ffacc37` 可分别查到成功的 Code 和 Release run。该 PR 后续的 GitHub 原生 merge 更新产生 `20c53de`，第一父提交是 `ffacc37`，所以祖先关系能够机械证明。相反，PR #120 的一次 rebase 把 `60053e5` 改写为不相干的 `e29f7de`；这种历史没有安全的增量继承链，必须完整重验。

GitHub 的临时测试 merge 可能已经包含稍后才进入 PR 分支的 base 提交，但历史 workflow API 不保存那次临时 merge 的 base SHA。本方案不解析旧日志或另存证据清单，而选择保守重验这一小类竞态；避免为减少一次边缘重复构建而维护第二套状态。

## 复核记录

- 2026-09-19 · 增量路径从未生效：path-filter job 在安装 DotSlash 之前运行「Resolve validation range」，那一步却用仓库固定的 `src/build/tools/jq-bin`（DotSlash 清单）判断上一 head 有没有成功 run；`/usr/bin/env: 'dotslash': No such file or directory` 使管道退出非零，被 `if` 当成「没有成功 run」，每次 `synchronize` 都回退到完整 PR diff（#257 的 Code run 35392813862、Release run 35392813857，以及更早的 35388630244 都是这样）。修法沿用本文已采用的机制：runner 自带的 GitHub CLI 内置 jq，`gh api --jq` 直接求值（jq 变量用 `$ENV.PREVIOUS_HEAD` 读环境），不引入新工具；三种结果分开——成功 run 计数为 0 才是「没有成功记录」，`gh api` 非零退出且 stderr 带 `gh:` 前缀或连接错误是查询失败，其余是答案无法求值——后两种回退完整检查并在日志里引用 stderr 原文。历史改写、非 PR 运行、缺上一 head 的判定不变。回归用例 `src/build/tests/check_validation_range.sh` 从两份 workflow 抽出步骤正文，在没有 DotSlash 的 PATH 下用替身 `gh`（以固定 jq 求值同一条 `--jq` 过滤式）跑十几种输入，含「上一提交只改 Skill、本次只改文档」时 Release 路径过滤为 false。
- 2026-09-19 · Codex SWE 评审（#259）后的两处收紧：对 `gh api` 非零退出的分类改为三段——`gh:` 前缀或连接错误记查询失败，裸的 jq 消息（`cannot iterate over`、`failed to parse jq expression`）记求值失败，其余如实报「无法判定查询还是求值失败」；三段都回退完整 PR diff 并引用 stderr 原文，分类只是诊断用的启发式，不是完备分类。回归用例的替身 `gh` 改为核对请求本身（`api`、`--method GET`、本 workflow 的 runs 路径、`head_sha` 等于上一 head、`event=pull_request`、`status=completed`），并按 workflow 文件分别给夹具，用「Release 成功但 Code 没成功」及其反向配对证明两份历史不能互借；本地变异核对：把 code.yml 的查询改指 release.yml 时 3 项失败，把计数 0 当成功时 6 项失败。替身用固定的 jq 1.8.2 求值同一条过滤式，生产用 gh 内置的 gojq；替身的错误用例是复现 CLI 文案，不是真实 GitHub API 故障实测。
- 2026-09-19 · Codex 复审指出 `{"workflow_runs":{}}` 这种结构异常在原过滤式下得 0、会被记成「没有成功记录」；过滤式改为先用 jq 原生的 `type` 核 `workflow_runs` 是数组，不是就 `error("unexpected workflow_runs type: …")`，gh 内置 jq 把它打成 `error: …`（本机核过），分类规则据此把 `error: ` 前缀记为求值失败；回归用例加对象夹具与 `error: ` 文案两条。
- 2026-09-21 · 所有者裁定按事件分层并接缓存。实测（#284 / #285 的 Code 与 Release run）：排队只有秒级到三分钟；Code 的 Intel macOS 作业 8 分钟、另两平台 3 分钟；完整包作业 13–17 分钟，其中装依赖包约 8 分钟（下载全部上游制品并 `xz -9`）、装完整包约 5 分钟（再 `xz -9`）、安装与生命周期测试本身 1 分钟；两份 workflow 都以 `HCTL2_BUCK2_CACHE=0` 运行，日志「Cache hits: 0%」，771 个动作全部本地重做。三项改动：一、两份 workflow 的平台矩阵都由 path-filter job 按事件输出（`fromJSON`）——pull_request 只跑 Linux x86_64 与 macOS arm64（Code 与完整包同规则），`push` 到 main、tag、每周定时与手动跑三平台；两份 workflow 新增 `push: branches: [main]`，Code 另加 `push: tags: v*`，合入后的 main 与每个 tag 始终有三平台 Code 与三平台完整包的结论，缓存也只在这些非 PR 运行上保存；`check_validation_range.sh` 抽出两份 workflow 的选平台步骤，机械断言 PR 两平台、其他事件三平台。二、CI 改用仓库自带的 bazel-remote REAPI 缓存（`src/buck2` 的 `local` 模式，进程由 Process Compose 托管），数据目录放 `$RUNNER_TEMP`，用 GitHub 原生 `actions/cache/restore` / `actions/cache/save` 按「平台 + runner 镜像（`ImageOS`-`ImageVersion`）」键持久化（`buck2-reapi-<平台>-<镜像>-<run>`，前缀只恢复同镜像的最近一份——宿主 C 工具链不是 Buck 声明的输入，不同镜像的动作结果不能互用；Code 的 Ubuntu 24.04 与完整包的 Ubuntu 26.04 因此各存各的）；`HCTL2_BUCK2_CACHE_MAX_GIB=2` 只限制单次快照的来源目录规模，仓库 10 GB 配额下的总占用与驱逐节奏待观测；作业结束先 `process-compose down`，停成功才保存，停失败跳过保存。bazel-remote 的磁盘目录本就是为跨进程持久化设计的，`actions/cache` 是平台原生机制，两者都不是自建。三、pull_request 的完整包测试传 `--config hctl2.xz_preset=fast`（`xz -1`），main / tag / 定时 / 手动保持 `release`（`xz -9`，所有者 2026-09-07 裁定的发布制品参数不变）；预设经 Buck 配置进 genrule 命令，两种预设的动作摘要不同，缓存条目互不冒充。预期：源码-only 的 PR 不触发完整包，Code 约 1–3 分钟；改依赖或打包的 PR 约 3–6 分钟。未验证：GitHub 缓存条目的驱逐节奏与真实命中率，以合入后的 run 日志「Cache hits」为准。

- 2026-10-03 · 整体超时与 Clippy 诊断基线（小活 A）。五份 workflow 的 13 个 job 此前都没有 `timeout-minutes`；取值按近十次 run 的 job 级最大耗时：Code 的 Path filter 20 s、CI gate 4 s、Cargo hygiene 17 s、Docs checks 9 s 分别取 10、5、15、15 分钟；Buck2 矩阵实测 Linux 216 s、macOS arm64 287 s、macOS x86_64 919 s，最慢一项取 45 分钟（约 3 倍余量，且远低于 #287 那次要手工取消的 40 多分钟挂起）；Release 的 Complete package 矩阵实测 908、1195、1380 s，取 60 分钟（末尾是 `xz -9`，耗时随仓库增长）；PR contract 16 s 与 Static contract 6 s 取 5；Package lifecycle 110 s 取 30；Publish GitHub release 与 Tuwunel 的 build 近期无任何 run，按各自步骤性质取 30 与 60。Clippy 侧：`root//:clippy` 的 clippy 动作只把诊断写进各目标的 `clippy.txt`、不看内容，所以构建成功与零诊断无关——在 `main @ c8fe912` 上构建该目标得到 36 个产物、10 个非空，共 34 条诊断、8 个 lint、10 个 target（`proto` 10 条在 prost/serde 生成的 `hctl2.control.v1.serde.rs`；`chat` 1 条是 `src/crates/chat/src/model.rs:92` 的 `Action` 枚举 `large_enum_variant`；`control` 12 条、`tool` 2 条、`chat` 的 `tests/domain.rs` 4 条为机械改写）。所有者当日裁定本包只落超时，「诊断非空即失败」的关卡等基线清零后另包做。已验证：`actionlint`（含 `shellcheck --severity=error`）与 `root//build/tests:validation_range_test` 在本改动上通过。未验证：13 个超时值未经真实触发——要等某个 job 真挂或真跑长才有证据；`tuwunel-macos.yml` 与 Publish GitHub release 从未运行过，取值没有实测支撑。
- 2026-10-03 · 所有者裁定生成代码不计诊断，第一方基线随之收窄。`crates/proto/src/lib.rs` 第 3 行已有 `#![allow(clippy::doc_markdown, clippy::must_use_candidate, clippy::pedantic)]`，它经 `include!(env!("HCTL2_PROTO_OUT"))` 作用在 `genrule :generated`（`protoc` 加自写的 `codegen/main.rs`，产物不在版本控制里）产出的全部生成代码上——生成代码开口子这件事边界已经在了，只是口子窄：`useless_borrows_in_formatting` 不在 `pedantic` 组里，10 条因此漏网。根子是 lint 与 `-Dwarnings` 的作用域是整个 crate 编译到的每个 token，第一方源码只是其子集；Cargo 对依赖用 `--cap-lints allow` 处理同一件事，本仓的等价物就是这一行 allow，边界该落在 codegen 边界上而不是每道 lint 各补一次。裁定：Clippy 关卡只对第一方源码生效，生成代码不计。第一方基线因此是 24 条诊断、9 个 target（`control` 3、`chat_native_test` 5、`task_native_test` 4、`control` 的 `unit_test` 4、`tool` 1、`tool` 的 `unit_test` 1、`git_site_test` 1、`chat` 1 条 `large_enum_variant`、`chat` 的 `domain_test` 4），其中 23 条机械改写、1 条要改模型。上一条里把 proto 那 10 条算进 34 的口径按此更正。已核：生成文件不在版本控制（`git ls-files` 零命中）；`lib.rs` 的 allow 是 crate 级、经 `include!` 覆盖生成代码。未核：把 allow 开到 `clippy::all` 是否会连带盖掉 `lib.rs` 自身将来的手写代码（该文件目前只有文档注释与 `include!`），以及把 `include!` 包进 `mod` 以单独圈定生成代码会不会改掉既有路径——两者都留给做关卡那一包时定。
- 2026-10-03 · Clippy 关卡与基线清理（同一 PR 的后半）。新增 `root//build/tests:clippy_clean_test`（`src/build/tests/check_clippy_clean.sh`）：`root//:clippy` 的 clippy 动作无论报告里有什么都成功，所以这是唯一的读者，一份非空报告就让构建失败并把内容打到日志。报告经 `$(location root//:clippy)` 以 env 传入，路径保留 filegroup 的 `<crate>/<key>` 结构（`store/unit`、`tool/unit`、`control/unit` 互不覆盖）；`unit_test` 在 control、tool、store 三个包里各有一个，沙箱按 basename 扁平拷贝会让干净的那份盖掉脏的那份，目录输出是这里唯一不漏的走法。沙箱里一份报告都找不到时关卡直接失败（不是放行），免得目标选择一变就静默失效。24 条第一方诊断清零，落在 12 个点：`needless_question_mark` 3 处（drafting.rs、chat/mod.rs、native_tests.rs）、`collapsible_if` 1 处（tree.rs，改成 edition 2024 的 let-chain）、`items_after_test_module` 1 处（services.rs 的 `uid_tests` 整块移到文件末）、`while_let_loop` 1 处（integration.rs）、`permissions_set_readonly_false` 1 处（git_site.rs，改成先存原权限再恢复——原写法 `set_readonly(false)` 在 Unix 上会把目录置成 world writable，lint 指的是这个而非空操作）、`cloned_ref_to_slice_refs` 4 处（domain.rs，改用 `std::slice::from_ref` 借用；`&[source]` 是 move 进临时数组，Clippy 给的建议在 `source` 后面还要用时编不过）、`large_enum_variant` 1 处（model.rs 的 `Action::CreateTopic` 箱掉 `origin` 与 `brief`，serde 对 `Box` 透明所以持久化 JSON 形状不变、治理记录不用迁移）。已验：`root//:clippy` 36 份报告全空；关卡在 36 份全空时通过；向 `crates/facts` 注入一个 `collapsible_if` 后关卡失败并打印 lint 名、位置与建议，随后撤回；第一方全量目标 30 项测试全过；`cargo fmt --all --check` 与 `cargo metadata --locked` 退出码 0；`profile-all` 14 项；`actionlint` 与 `shellcheck` 退出码 0。未验：关卡在 Linux 与两个 macOS 平台上的实际运行（本地只跑了 macOS arm64，待 CI 三平台）；`large_enum_variant` 装箱后各变体的实际字节数未实测，按字段推算是从 329 降到与 `Send` 持平。

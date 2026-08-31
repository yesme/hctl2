# 构建系统优化：三批实施方法

> 状态：已拍板 · 分三批落地中
> 基线：main @ 4bf8e81（草案 v0.15.4）
> 去向：`.github/workflows/**`、`src/**/BUCK` 与 `src/build/**`

## 定位与目标

这项工作位于构建与交付层。目标不是改产品架构，也不是另造一套 CI，而是让一次变更只执行与其影响范围相称的检查，同时让同一套检查可以在开发机、GitHub-hosted runner 或其他 CI 上执行。

检查定义归 Buck2，变更识别归 Git，合入策略归代码托管平台的 branch protection。GitHub Actions 只负责触发、选择执行器、调用 Buck target 和报告结果，不保存另一份编译逻辑。

硬约束：

- 构建、测试、Clippy 与打包检查继续使用 Buck2 原生 target/action；
- Linux x86_64、macOS x86_64、macOS arm64 的平台证据不可互相替代；
- required check 必须稳定出现，不因 workflow path filter 留在 Pending；
- selector 失败时退回更大的检查范围，不能静默少测；
- tag 发行始终执行三平台完整包、生命周期、制品证明和发布；
- 检查命令可脱离 GitHub Actions 在本机或其他 CI 运行。

可调整的是普通 PR 与合入后的检查频率、完整包的触发范围、执行器来源以及共享 cache 的部署方式。

## 现状判断

慢的首因是调度重复，不是 Buck2 action 本身：

- 纯文档或 memo PR 的 Code workflow 中位数约 30 秒；同类变更合入 `main` 后却无条件执行三平台 Buck、Cargo 与 docs，30 个样本中位数 84 秒；
- Release 将整个 `src/` 视为发行输入，`src/build/docs/**` 的修改也会触发三平台完整包，实测 205–240 秒；
- 完整 Release 中位数约 217 秒，普通 PR 还会上传约 630 MiB、没有后续消费者的三平台制品；
- dependency package 生命周期已被 complete release test 覆盖，相关 PR 合入后又在 `main` 三平台复验；
- 每个平台依次执行 `cquery`、build、test、Clippy；`cquery` 没有独立断言，Buck2 原生 `test --build-default-info` 可以合并 build 与 test 请求。

本地 Buck cache 已跨 worktree 共享，六个外部组件也已有独立 action。这里所说的“模块独立 cache”是让 target、声明输入与 action 足够细，使改动只使本模块及其反向依赖失效；不是部署多套物理 cache。

## 统一结构

```text
Git base/head 与文档 profile
            ↓
Buck2 Change Detector / Buck target graph
            ↓
Buck targets、tests、labels 与 action cache
            ↓
本机 / GitHub-hosted / 其他受信 CI executor
            ↓
commit status → branch protection
```

`src/` 的精确 target 选择采用 Meta 的 [Buck2 Change Detector](https://github.com/facebookincubator/buck2-change-detector)，不自写 `owner+rdeps` 替代品。BTD 比较 base/diff 两份 Buck 图，并处理删除、改名、BUCK/BZL imports、glob 与 buckconfig 变化；它只决定受影响 target，验证强度仍由仓库里的 Buck tests、target labels 和 gate policy 决定。引入前按仓库纪律补 `docs/research/` 源码审计并通过 DotSlash 锁定官方二进制。

根目录文档位于 `src` Buck cell 之外，由 Git 识别变更类别。可用版本控制内的自定义 `.gitattributes` 作为 profile 的单一来源，再由 `git check-attr` 在本机和 CI 读取，避免在多个 workflow 与 hook 中复制路径正则。

文档语气、架构是否重复、术语是否必要不伪装成确定性 lint。可机械判定的规则进 Buck；语义审阅使用仓库内、与具体模型无关的 rubric，由 LLM 或 human 审阅。Memo 不执行设计正文级别的审阅。

## 第一批：先删除重复工作

第一批不引入新组件，不改变产品或合同语义，也不做 target-level selector。只修改现有 CI 调度和 Buck 调用：

1. memo-only 变更不启动 docs/Buck runner；`.memo/review` 的既有基线检查仍保留；
2. PR body `edited` 只重跑 PR contract，不重新触发编译与文档检查；
3. 普通 PR 已在 strict branch protection 下通过 required checks 后，合入 `main` 不再无条件重跑三平台 Code；保留手动入口，并增加定期全量基线；
4. Release 精确区分产品/打包输入，至少排除 `src/build/docs/**` 等不进入发行物的工具；
5. PR 只验证完整包，不上传发行制品；tag 与手动发行保留 `--out`、artifact、attestation 和 publish；
6. 删除没有独立判定力的 `cquery`，用 `buck2 test --build-default-info` 合并 first-party build/test/Clippy 请求；
7. 将 dependency static contract 纳入 required gate 后，删除合入 `main` 的三平台 dependency lifecycle 子集复验；
8. `main`、tag、手动和 PR 的 gate 名称保持稳定；required workflow 不使用会使状态缺失的顶层 path filter。

第一批验收重点是：纯 memo 不启动编译，`src/build/docs/**` 不启动完整 Release，PR 编辑描述不重跑编译，普通合入不再重复同一棵树，tag 发行行为不变。

## 第二批：让 Buck 图承担细粒度

1. 给 production targets 补 Buck 原生 `tests=[...]` 关系；
2. 将全库 `//:clippy` 收细为模块级聚合 target；
3. 以 target labels 表达 fast、platform、integration、release 等验证属性；
4. 调研并锁定 Buck2 Change Detector，以 base/diff 图选择受影响 targets；BTD 或图导出失败时全量执行；
5. 按 `design / spec / delivery / research / memo` 区分文档 profile：
   - memo notes/log 不执行产品文档语义审阅；
   - research 检查离线链接与证据元数据；
   - architecture/vision 检查中文表达、重复内容与层级；
   - spec/contract/delivery 检查版本、CT 配套、术语和合同一致性；
6. Cargo parity 先保留其独有的 lint/manifest 价值；Buck 等价覆盖后缩为工具链或定期基线检查，不并行维护第二份构建定义。

BTD 输出的是“受影响范围”，不是“必须运行所有下游昂贵目标”。普通 Rust 变更默认三平台验证受影响的第一方 target、test、Clippy 与 first-party release；完整三平台生命周期只在 packaging、dependency、toolchain 或 tag 变化时运行。是否为普通 Rust 变更额外保留一次 Linux complete-package smoke，可在第二批实施前用实际失败样本再定。

## 第三批：执行器与共享 cache

1. 提供可选 Git hook：
   - `pre-commit` 只做亚秒级 index 检查；
   - `pre-push` 对待推送 SHA 运行当前宿主平台的受影响 Buck gate；
   - 不在 `post-commit` 或 `post-merge` 隐藏启动编译；
2. 评估标准 REAPI shared action cache，而不是缓存 `buck-out`：
   - 受信开发机、main、tag 可写；
   - PR 与 fork 默认只读或无凭证冷构建；
   - 保持 `remote_enabled=false`，暂不引入 remote execution；
3. GitHub-hosted 只是默认执行器之一；其他 CI 可运行相同 Buck targets，再通过 GitHub App/commit status 报告给 branch protection；
4. 个人日用 Mac 不直接暴露给 public repo 的自动 PR。确需替代 hosted macOS arm64 时，使用专用、隔离、最好一次性的受信 runner；macOS x86_64 继续由对应平台证据提供；
5. 共享 cache 先用 `remote_cache_hits`、传输字节和 wall time 做试验，只有稳定快于冷构建才长期启用。

## 时间目标

- memo-only PR：不启动编译，目标 20 秒量级；
- design/spec PR：机械检查维持 30 秒量级，语义审阅只看变更文件和章节；
- 普通 first-party PR：目标 1–2 分钟；
- 三平台完整包：只在相关变更或 tag 运行，维持 3–4 分钟；
- 合入 `main`：不重复 PR 已验证的同一棵树；定期全量负责发现 runner、托管制品和工具链随时间变化的问题。

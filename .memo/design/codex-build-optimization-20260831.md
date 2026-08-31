# 构建系统优化：三批实施方法

> 状态：已拍板 · 三批仓库内实施均已落地；持久远端端点未采购或部署
> 基线：main @ 4efe0d3（草案 v0.15.4）
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

第二批落地时没有增加 Linux complete-package smoke：first-party release 已进入普通 Rust 变更的三平台受影响目标集，完整生命周期继续由 packaging、dependency、toolchain、构建规则与 tag 变化触发。production target 的 `tests`、模块级 Clippy 和验证类别均在 Buck 图中声明；BTD 使用官方制品并在失败时回退全量 first-party 集合；文档 profile 由 `.gitattributes` 分类；Cargo 只保留格式与 locked metadata。

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

第三批落地结果：

- `src/build/hooks/` 提供可选 `pre-commit` 与 `pre-push`，由 `src/build/tools/git-hooks` 显式安装或移除；不在克隆、提交或合并后自动开启。前者只调用 `git diff --cached --check`，后者验证 Git 即将推送的精确提交。
- `src/build/ci/affected-targets` 把原先写在 GitHub Actions 里的 Buck 导图、BTD 与标签筛选提成可移植入口；`verify-affected` 在当前宿主平台执行结果，选择失败时回退到全量检查。GitHub Actions 与其他 CI 不再需要各写一份选择算法。
- BTD JSON Lines 由摘要锁定的 jq 官方单文件制品解析，不要求开发者预装 jq，也不以 `grep`/`sed` 解析 JSON。
- Linux CI 的 DotSlash 引导改用锁定到不可变提交的 Meta 官方 GitHub Action，并在安装后核对仓库固定的 `v0.5.9`；macOS CI 与开发机继续用同一版本和三平台 SHA-256 的最小安装器。官方 Action 当前只支持最新 Release，且在 macOS 15 会误用 BSD `sha256sum --check`，所以版本核对采用失败关闭、macOS 保留已验证的摘要安装路径，避免工具静默漂移或为套用 Action 而降级校验。
- `src/buck2` 增加 `local / 0 / remote` 三种缓存模式；`remote` 模式只消费 Buck 原生 `.buckconfig.local` 覆盖和 mTLS 证书路径，不把端点、私钥或另一份指纹写入动作图。Buck2 上游 #1445 尚未解决 `http_headers` 可能进入事件日志的问题，因而模板明确不用持有者令牌。来自分叉仓库的 PR 当前不配置凭据，也不会取得写证书。
- `buck2-cache benchmark` 在相同隔离目录依次执行冷构建、写入缓存和缓存复用。每轮之间由 Buck 原生 `clean` 删除隔离 `buck-out`，再由 Buck 事件日志报告命中、传输与时间。本机 Ubuntu 26.04 的 Rust 工具链探针实测依次为 **3:09.2**、**3:10.3** 和 **1.6 秒**；复用轮的 **5 个动作全部命中缓存**，HTTP 下载从 **105 MiB** 降为 **0**，RE 下载 **1.9 MiB**。这证明现有 REAPI 动作缓存能脱离 Buck 守护进程与 `buck-out` 复用；第一次写入仍要下载工具链并上传 CAS，不能伪装成免费加速。
- 本轮还修正了 `bazel-remote` 第一次下载时的启动竞态：启动器现在等 HTTP 健康检查端点可用后才让 Buck 连接。

仓库没有凭空部署“共享远端主机”。目前跨工作树的回环缓存已经验证；跨机器缓存的代码接口、凭据边界和测量入口也已齐备。真正启用还要提供持久 REAPI 端点、CA 和受信客户端证书，再通过真实网络运行同一基准。如果缓存复用不能稳定快于冷构建，就维持本机模式。额外执行器同理：任意受信执行器都可运行 `verify-affected` 并回报提交状态，但个人日用 Mac 不注册为公开仓库的自动执行器。

## 时间目标

- memo-only PR：不启动编译，目标 20 秒量级；
- design/spec PR：机械检查维持 30 秒量级，语义审阅只看变更文件和章节；
- 普通 first-party PR：目标 1–2 分钟；
- 三平台完整包：只在相关变更或 tag 运行，维持 3–4 分钟；
- 合入 `main`：不重复 PR 已验证的同一棵树；定期全量负责发现 runner、托管制品和工具链随时间变化的问题。

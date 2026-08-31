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

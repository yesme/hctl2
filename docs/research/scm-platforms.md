# 代码协作平台市场调研：GitHub 缺省，其余后端按需

> 状态：调研 · 日期：2026-09-06<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-SCM-PLATFORMS<br>
> 对象：GitHub（缺省实现）、GitLab（gitlab.com 与自建）、Gitea、Forgejo、Gerrit、Bitbucket Cloud、Azure DevOps Repos，以及各家的官方命令行工具<br>
> 许可证：GitHub、gitlab.com、Bitbucket Cloud、Azure DevOps 是托管服务，受各自服务条款约束；GitLab 自建版社区版 MIT（企业版专有）；Gitea MIT；Forgejo 自 v9 起 GPL-3.0-or-later；Gerrit Apache-2.0

## 定位

第五模块 Repo（仓库）的远端一侧接的是「代码协作平台」：承载 PR 或 MR 线程、平台评审状态、检查结果、远端合入的外部系统（备忘 `.memo/design/scm-module-20260906/01-scm-module.md` 第三节）。所有者 2026-09-06 拍板：GitHub 是缺省实现，本批只做它的适配器；未来支持哪些后端、先后如何，以这份调研为依据，不在设计正文里点名「第二个」。本文只回答三件事：各平台在我们的调用面上能给什么；哪些差别要写进绑定的能力声明；各家有没有能走「随包命令行工具优先」这条路的官方工具。本文不比较平台的产品优劣，也不替任何仓库选平台。

## 我们的调用面

B2 的无 Run 切片需要的操作（备忘第六节），按读、写、映射三类：

| 类 | 操作 |
| --- | --- |
| 读 | 仓库身份、目标 ref 当前头；PR 或 MR 的当前头、必需检查逐项状态、评审线程是否解决、正式评审状态、合并资格；目标分支的保护条件；评审评论正文（供代取） |
| 写 | 推送变更集分支；创建或更新 PR 或 MR（含描述）；把 Gate 结论或集成结果写回；请求合入 |
| 映射 | 平台账号到账本里的人 |

其中最要紧的一条是合入时的两个「头」：源头是待合入的提交，目标头是预览时主干所在的提交。约束要求集成命令固定预期目标头；本地路径用 `git update-ref` 的旧值比较做到了；平台能不能做到，是下面逐家对照的重点。

## 逐家对照

| 平台 | 形态 | 评审单位 | 合入接口对源头的校验 | 目标头有没有保证 | 检查与评审的回读 | 官方命令行 |
| --- | --- | --- | --- | --- | --- | --- |
| GitHub | 托管 | Pull request | 合并接口 `sha` 参数与 `gh pr merge --match-head-commit` 都校验源头；源头变了拒绝 | 没有「预期目标头不符就拒绝」的参数。分支保护「要求分支与目标同步」只保证候选包含当前主干；合并队列在最新主干上重新验证并排队合入，语义是「接受目标前移、重新验证」 | REST 与 GraphQL：检查（checks、statuses）、正式评审（reviews）、评论线程（review threads 的 resolved 状态）、分支保护条件均可读 | gh，官方，`--json` 白名单字段，随包已钉 v2.99.0 |
| GitLab | 托管（gitlab.com）与自建（社区版、企业版） | Merge request | 合并接口与 `glab mr merge --sha` 校验源头：「只在源分支的 HEAD 等于此 SHA 时合并」 | 没有预期目标头参数。合并列车（merge trains）把排队的 MR 与前面的 MR 一起在目标分支上跑流水线、依次合入，前提是启用「合并结果流水线」与「流水线必须成功」；语义同样是「接受目标前移、重新验证」 | REST：批准（approvals）、流水线、讨论线程（discussions 的 resolved）、合并请求设置可读 | glab，GitLab 官方维护，`glab mr view -F json` 与 `--jq`；`glab mr merge` 的文档未列结构化输出，待核 |
| Gitea | 自建 | Pull request | 合并接口的表单字段 `head_commit_id`（源码 `services/forms/repo_form.go` 的 `MergePullRequestForm`）；`do` 可选 `merge`、`rebase`、`rebase-merge`、`squash`、`fast-forward-only`、`manually-merged`；另有 `merge_when_checks_succeed`、`force_merge` | 没有预期目标头参数；`fast-forward-only` 要求目标能快进到源头，目标前移后快进失败即拒绝——这是一种间接的目标头保证，但只对快进策略成立 | REST：评审（reviews）、提交状态（statuses）、分支保护可读 | tea，Gitea 官方，`pulls create / checkout / review / approve / reject / merge / close / reopen`，全部支持 `-o json` |
| Forgejo | 自建（Gitea 分支，API 与 Gitea 高度兼容） | Pull request | 同 Gitea 的接口形状，逐版本核对 | 同 Gitea | 同 Gitea | 官方没有自己的命令行；社区的 forgejo-cli（fj）支持创建与合并 PR；tea 是否完全兼容待核 |
| Gerrit | 自建 | Change（每个 Change 有多个 patch set） | 提交（submit）作用于 Change 的当前 patch set，没有 SHA 参数；改了 patch set 就是新的评审对象 | **有**：项目的提交类型（submit type）设为 Fast Forward Only 时，「只有在提交时目标分支能快进到当前 patch set 时才能提交」，目标一动其他 Change 就不可提交，必须变基。这是本次调研里唯一的精确目标头保证；代价是文档自己也不推荐，只适合改动少、稳定性要求高的项目。其余提交类型（Merge If Necessary、Rebase If Necessary 等）是「接受目标前移」 | REST：labels 与 votes（Code-Review 等）、`mergeable`、`submittable`、评论均可读；不可提交时 submit 返回 409 | 没有 gh 那样的官方命令行；官方 SSH 命令 `gerrit query --format=JSON` 与 REST 可用；OpenDev 维护的 git-review 只管推送评审 |
| Bitbucket Cloud | 托管 | Pull request | 合并接口有 `merge_strategy`（merge_commit、squash、fast_forward）与 `close_source_branch`；有没有源头校验参数，本次未能取到接口文档正文，待核 | 待核 | REST：参与者批准（participants）、提交状态（commit statuses）可读 | 没有官方命令行（Atlassian 的 ACLI 是否覆盖 Bitbucket，待核） |
| Azure DevOps Repos | 托管 | Pull request | 完成 PR 用更新接口把 `status` 设为 `completed`，请求体带 `lastMergeSourceCommit`（上次合并预览时源分支的头）；`completionOptions.mergeStrategy` 可选 noFastForward、squash、rebase、rebaseMerge | 没有预期目标头参数；`lastMergeTargetCommit` 只是回读字段。分支策略里的构建验证可设「目标分支更新时立即使构建状态失效并重排」或「更新后超过 n 小时失效」，语义是「目标前移则重新验证」 | REST：评审者投票（approve、approve with suggestions、wait for author、reject）、状态、策略评估可读；有「评论必须解决」策略 | az repos pr（azure-devops 扩展），`create / show / list / update / set-vote / reviewer / policy`，默认 JSON 输出 |

## 三个发现

**源头校验普遍，目标头保证几乎没有。** GitHub、GitLab、Gitea、Azure DevOps 都提供「源分支的头必须等于某个 SHA」的校验，防的是「合入了没审过的提交」；只有 Gerrit 的 Fast Forward Only 提交类型和 Gitea 的 `fast-forward-only` 策略能在目标前移时拒绝，而且都以快进为前提。所以备忘第六节的结论成立：适配器要在绑定里声明「不能保证预期目标头」（GitHub 一律如此），集成意图要事前选择授权形态，不能假装平台做了比较并交换。

**平台的答案是队列与重新验证，不是拒绝。** GitHub 的合并队列、GitLab 的合并列车、Azure DevOps 的构建过期策略，都是在目标前移之后拿最新主干重新验证候选再合入。这正是备忘第十五节第 5 项所说的第二种授权形态「接受期间目标前移，按保护要求重新验证，以回读为准」的平台原生实现。适配器用这些机制时，凭证仍只能在回读到合并提交与实际目标头之后签。

**官方命令行工具的覆盖参差。** GitHub、GitLab、Gitea、Azure DevOps 四家有官方命令行且有结构化输出，按所有者定的四级顺序（随包命令行 > 官方 SDK > 按接口描述生成 > 手写）可以先走第一级，但每个操作仍要逐项核对能不能给我们要的字段；Gerrit 只有 SSH 命令与 REST；Forgejo 靠社区工具或 Gitea 兼容；Bitbucket 没有官方命令行。

## 能力声明的维度

从上表抽出来、要写进 Repo 模块约束里「平台绑定的能力声明」的维度：

1. 评审单位与它的身份：PR / MR 编号，或 Gerrit 的 Change 与 patch set。
2. 源头校验：有没有「源头必须等于某 SHA」的合入参数。
3. 预期目标头保证：有（Gerrit Fast Forward Only、Gitea fast-forward-only）、无（其余）。
4. 目标前移后的平台行为：拒绝、重新验证后合入（队列）、直接合入。
5. 评审状态的形态：正式评审（GitHub reviews、Gitea reviews、Azure votes、Gerrit labels）、批准（GitLab approvals、Bitbucket participants）、只有评论。
6. 线程解决状态能不能读、能不能作为合入前置。
7. 检查结果的来源：平台自带流水线、外部状态回写，或两者。
8. 目标分支保护条件能不能整份回读（必需检查清单、同步要求、线程解决、批准数）。
9. 身份映射：平台账号到人。
10. 官方命令行与结构化输出：有、无、部分。

## 决定建议

- **采用二进制、GitHub 缺省**：与 `docs/research/sdk/github.md` 一致，GitHub 的适配器按四级顺序先用随包的 gh；控制面一侧原定的 octocrab 退为第二选择，见该文件的复核记录。
- **其余平台暂缓**（研究层的六种复用决策之一）：不点名第二个；哪个仓库真在哪家平台上，再按本文的维度写它的能力声明、按四级顺序核它的命令行，补对象文件 `docs/research/sdk/<平台>.md`。
- **约束层的写法**：Repo 模块约束里，集成意图的授权形态分两种——「精确目标头」（本地路径与 Gerrit Fast Forward Only 这类能保证的目标）与「接受目标前移、按保护要求重新验证」（GitHub 等）；绑定不能保证前者时默认拒绝前者，后者须人事前显式选择。这是备忘第十五节第 5 项已拍板的内容，本文提供的是各平台能做到哪一种的证据。
- **待核项**：Bitbucket Cloud 合并接口的字段与 Atlassian 命令行的覆盖；`glab mr merge` 的结构化输出；tea 对 Forgejo 的兼容程度。真有仓库在这些平台上时再核，不为本批补。

## 证据

- GitHub：[合并 PR 接口（`sha` 参数）](https://docs.github.com/en/rest/pulls/pulls#merge-a-pull-request) · [gh pr merge（`--match-head-commit`）](https://cli.github.com/manual/gh_pr_merge) · [分支保护：要求分支与目标同步](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches#require-status-checks-before-merging) · [合并队列](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue)（2026-09-06 读：「与『要求分支与目标同步』提供同样的好处，但不要求作者更新分支」；检查失败或与目标冲突的 PR 被移出队列）
- GitLab：[glab mr merge](https://gitlab.com/gitlab-org/cli/-/blob/main/docs/source/mr/merge.md)（`--sha`：「只在源分支的 HEAD 等于此 SHA 时合并，用来确保只合入已评审的提交」）· [glab mr view（`-F json`、`--jq`）](https://gitlab.com/gitlab-org/cli/-/blob/main/docs/source/mr/view.md) · [合并列车](https://docs.gitlab.com/ci/pipelines/merge_trains/)（「针对队列中每个合并请求前面所有合并请求的合并结果测试」，前提是合并结果流水线与流水线必须成功）· [合并请求 API](https://docs.gitlab.com/api/merge_requests/)
- Gitea：[`services/forms/repo_form.go` 的 `MergePullRequestForm`](https://github.com/go-gitea/gitea/blob/main/services/forms/repo_form.go)（`do` 取值、`head_commit_id`、`merge_when_checks_succeed`、`force_merge`）· [tea 命令清单](https://gitea.com/gitea/tea/src/branch/main/docs/CLI.md)（`pulls` 子命令与 `-o json`）
- Forgejo：[forgejo-cli](https://codeberg.org/Cyborus/forgejo-cli)（社区工具，创建与合并 PR）
- Gerrit：[项目配置：提交类型](https://gerrit-review.googlesource.com/Documentation/config-project-config.html)（Fast Forward Only：「只有在提交时目标分支能快进到当前 patch set 时才能提交」）· [Changes REST：Submit Change 与 labels、submittable、mergeable](https://gerrit-review.googlesource.com/Documentation/rest-api-changes.html)
- Bitbucket Cloud：[Pull requests API](https://developer.atlassian.com/cloud/bitbucket/rest/api-group-pullrequests/)（本次抓取被截断，合并接口字段待核）
- Azure DevOps：[Pull Requests - Update（`lastMergeSourceCommit`、`completionOptions`、`mergeStrategy`）](https://learn.microsoft.com/en-us/rest/api/azure/devops/git/pull-requests/update?view=azure-devops-rest-7.1) · [分支策略（构建验证的过期选项、评论必须解决、限制合并类型）](https://learn.microsoft.com/en-us/azure/devops/repos/git/branch-policies?view=azure-devops) · [az repos pr](https://learn.microsoft.com/en-us/cli/azure/repos/pr?view=azure-cli-latest)（默认 JSON 输出）
- 本仓库：[第五模块方案](../../.memo/design/scm-module-20260906/01-scm-module.md) · [Git 现场引擎](./sdk/git.md) · [GitHub 客户端层](./sdk/github.md) · [Participant 约束 §ChangeSet 与 Git 事实](../design/spec/participant.md#changeset-与-git-事实)

## 复核记录

- **2026-09-06，PR #186 评审（Codex/GPT 推翻一条结论）。** 正文把 Gerrit 的 Fast Forward Only 提交类型与 Gitea 的 `fast-forward-only` 合并策略列为「预期目标头保证」（逐家对照表、三个发现第一条、能力声明维度第 3 条、决定建议第三条），这条推论不成立。快进只要求当前目标是源提交的祖先，不比较当前目标与人预览时记录的提交是否相等：提交链 A → B → H，预览时目标为 A、源固定为 H，别人先把目标推进到 B，B 仍能快进到 H，源没变，「目标必须还是 A」却已不满足；Gerrit 官方定义也明确给后继变更留了可提交路径。所以在已核对接口的平台里**没有发现**预期目标头的比较并交换：能力声明维度第 3 条对 GitHub、GitLab、Gitea、Forgejo、Gerrit、Azure DevOps 记「无」，Bitbucket Cloud 的合并接口字段仍待核、暂不记；本批没有任何经过验证、可声明该能力的平台绑定，决定建议里「Gerrit Fast Forward Only 这类能保证的目标」一句作废，「精确目标头」形态目前只有本地路径（宿主 git 的 `update-ref` 旧值比较）能采用；快进策略的正确归类是「目标前移后的平台行为 = 拒绝非快进、接受快进」，属于第二种形态下的一种保护条件。GitHub 合并接口 `sha` 只校验源头的判断维持。依据：[Gerrit 提交类型](https://gerrit-review.googlesource.com/Documentation/config-project-config.html)、[Git 快进合并](https://git-scm.com/docs/git-merge#_fast_forward_merge)。约束层已按此写：`docs/design/spec/repo.md` §集成 的第一种形态对远端目标以绑定声明为前提，本批没有任何平台声明它。

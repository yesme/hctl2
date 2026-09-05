# GitHub 依赖与代码协作平台边界

> 状态：已拍板 · 记录 2026-09-05 讨论结论；事实核查与责任划分继续有效，「不意味着增加第五个业务模块」一句被所有者 2026-09-06 的方向覆盖（SCM 立为第五模块），见同目录 `01-scm-module.md`
> 基线：main @ 0456ac2（草案 v0.16.5）
> 去向：docs/design/spec/system.md + docs/design/spec/participant.md + docs/design/delivery.md

## 问题与结论

当前开发已经反复使用这条路径：作者开 PR → 多个评审者在评论中给意见 → 作者修正 → 再评审 → 合入。所有者据此问：我们是否已经依赖 GitHub？要保持无依赖，还是显式引入可替换的 GitProvider？所有者对两者没有先验偏好，要求根据实际需求判断。

本轮结论：**Git 是基础依赖，GitHub 不必成为整个产品的必需依赖；使用 PR 协作的路径则显式依赖一个代码协作平台。** 以 GitHub 为首个实现，保留既有本地集成路径；以后有 GitLab 等需求时再适配。依赖清楚、可以替换，比名义上没有依赖而实现里处处调用 GitHub 更诚实。

这里称为 **SCM provider（代码协作平台适配器）**，比 GitProvider 更准确：Git 本身不定义 PR/MR、评审评论或 CI 检查。这个名称用于说明既有适配器的职责，不意味着增加第五个业务模块、独立服务或一套通用对象模型。本轮只保存结论，没有修改当前合同。

## 现状核对

「设计没有规定任何 GitHub 依赖」不完全准确；「设计要求所有用户依赖 GitHub」也不成立。

| 问题 | 当前事实与出处 |
| --- | --- |
| Git 是否已是设计的一部分？ | 是。[Git 的双重角色](../../../docs/design/spec/system.md#git-的双重角色)区分不可变正文与账本判决的审计副本；本地工作树、提交、集成也已进入合同。 |
| 远端 PR/merge 是否完全没设计？ | 已有。[ChangeSet 与 Git 事实](../../../docs/design/spec/participant.md#changeset-与-git-事实)明确远端 push、PR、merge 走适配器；[外部权威副作用](../../../docs/design/spec/system.md#外部权威副作用)已有持久意图、绑定、投递与回读规则。 |
| 本地集成必须经过 GitHub 吗？ | 不必。[交付文档](../../../docs/design/delivery.md)的「纵向切片 A」第 5 步走本地工具箱，「纵向切片 B」第 8 步区分本地工具箱与远端适配器。 |
| 当前代码是否已有 GitHub 专用依赖？ | 有。[使用说明](../../../docs/usage.md#等待外部事实)中的三类 GitHub 事实查询使用随包固定版本的 `gh`；[GitHub SDK 调研](../../../docs/research/sdk/github.md#复核记录)明确只读等待已落地，control 的 GitHub provider 尚未实现。 |
| 设计真正缺什么？ | [固定内核与受控端口](../../../docs/design/spec/system.md#固定内核与受控端口)的可替换端点表列出四场景，后文另有 SCM 副作用；[事实权威地图](../../../docs/design/spec/system.md#全系统事实权威地图)尚未把 PR 讨论、平台评审与控制面判决的关系集中说明。缺的是既有边界的完整说明，不是从零发明适配层。 |

## 谁负责什么

GitHub 在这条路径里不只是 repo storage：它还提供 diff、讨论、评审、CI 状态与远端合入。把这些一起叫作「Git 存储」会掩盖实际依赖。

| 责任 | 归属 |
| --- | --- |
| 文件、commit、tree、ref、本地工作树 | Git；本地检查与集成复用已有工具箱。 |
| PR/MR、行内讨论、平台评审状态、检查结果、远端合入 | 代码协作平台；适配器使用其原生能力并回读事实。 |
| HCTL 特有的席位、精确评审版本、返工规则、任务完成判定与凭证 | HCTL 控制面；外部事实作为证据进入已有准入流程。 |

原生客户端、Workbench 集成客户端与 CLI 都可以表达用户意图；客户端没有高低之分。平台上的评论、批准、合入是各自的外部事实，是否满足某个 HCTL 契约，仍按该契约判断。

这不等于每次平台评审之后再加一轮 HCTL Gate。[Task 的轻量路径](../../../docs/design/task.md#无-run-的轻量路径)已经允许契约接受可回读的精确外部 SCM 评审证据。只有任务确实要求 HCTL 内部独立评审、多席位或返工治理时，才使用相应 Run；平台已完成的工作直接复用。

## 采用范围与代价

优先采用 GitHub 已有的 PR、评论、检查和合入能力，不另造评审网站或本地 GitHub 替身。适配继续沿用现有 Port–Provider Binding 与外部副作用规则，只覆盖我们的实际使用子集；GitLab 的差异留到有真实需求时处理。

选择远端 PR 路径，就接受该平台的账号、权限、可用性及 API 依赖；保留本地路径，则不把这些成本强加给所有用户。平台不可用时，依赖它的步骤等待或报告无法确认，不把本地事实当作远端已完成。

**可替换接口不等于可迁移全部历史。** 普通 Git clone 保存代码与 Git 历史，不保存整套 PR、评论和平台状态；更换平台时，这些资料仍需要平台导出或迁移能力。GitHub 的[备份说明](https://docs.github.com/en/repositories/archiving-a-github-repository/backing-up-a-repository)也区分 Git 镜像与附加元数据备份。本轮不实现历史迁移工具。

## 实现层脚注：当前评论流程还不是自动 Gate

2026-09-06 回读 [PR #176](https://github.com/yesme/hctl2/pull/176)：评审意见是普通 PR 评论，返回的 formal reviews 列表为空；[Fable 复审](https://github.com/yesme/hctl2/pull/176#issuecomment-5549854613)与 [GLM 复审](https://github.com/yesme/hctl2/pull/176#issuecomment-5549857044)均通过同一 GitHub 用户 `yesme` 发布。这不否定评审价值，只说明目前是人和 harness 解释评论、推进流程，不是 GitHub 在执行 HCTL 的席位规则。

GitHub 的[普通评论](https://docs.github.com/en/rest/issues/comments)与[正式 PR Review](https://docs.github.com/en/rest/pulls/reviews)是不同能力。把现有流程自动化时，需要能确定「谁审了哪个版本、结论是什么」；共享 GitHub 登录名或评论中的署名，单独都不足以证明 HCTL 席位与评审版本。继续复用既有执行身份与证据规则即可，不由此要求每个模型单独开 GitHub 账号或另建复杂认证系统。

## 后续设计工作

后续单独任务把代码协作平台的可选性、事实归属及与已有 SCM 适配器的关系补进权威文档；交付文档说明 GitHub 首先支持哪些操作、本地路径覆盖哪些场景。证据继续引用现有 [Git 调研](../../../docs/research/sdk/git.md)与 [GitHub 调研](../../../docs/research/sdk/github.md)，不在本备忘里另写接口字段或实现计划。

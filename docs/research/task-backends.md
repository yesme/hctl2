# 任务后端：Linear / GitHub 外部来源、Vikunja 与 git-bug

> 类别：⑥ 机械后端与基础设施 · 证据编号：E-L3-VIKUNJA、E-L3-GIT-BUG<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

## L3 外部系统与观察清单

Linear 和 GitHub 提供外部字段的写入权威，也是没有 Workbench 时的外部系统原生降级方案；它们不是 HCTL 的 Task 模型：

- Linear：[GraphQL](https://linear.app/developers/graphql)、[Webhook](https://linear.app/developers/webhooks)、[速率限制](https://linear.app/developers/rate-limiting)、[分页](https://linear.app/developers/pagination)
- GitHub：[Projects API 指南](https://docs.github.com/en/issues/planning-and-tracking-with-projects/automating-your-project/using-the-api-to-manage-projects)、[GraphQL 参考](https://docs.github.com/en/graphql/reference/projects)、[Webhook](https://docs.github.com/en/webhooks/webhook-events-and-payloads)、[REST 最佳实践](https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api)
- React Aria：[Kanban 示例](https://react-aria.adobe.com/examples/kanban)、[拖放](https://react-aria.adobe.com/dnd)——只作为 UI 基础组件。

<a id="e-l3-vikunja"></a>
## E-L3-VIKUNJA · 本地任务服务器选型（限时验证）

[Vikunja `v2.5.0 / ef2200e9`](https://github.com/go-vikunja/vikunja/tree/ef2200e9429c5cc42f5c1811433418bfcc72b3aa)：Go 单二进制、默认 SQLite、REST API（v1/v2）与 webhooks，看板/列表/甘特多视图；AGPL-3.0-or-later（desktop 组件 GPL-3.0-or-later）。该发布已有官方 macOS arm64/amd64 full zip，旧记录“无 Darwin 包”已作废。

角色：Kanban 场景本地 content 后端的已选实现。采用边界：独立进程托管、经任务源受控端口访问，不 vendor、不链接其源码——AGPL 义务因此限于该服务自身。验证要点见[交付文档](../design/delivery.md#开工前限时验证)。

<a id="e-l3-git-bug"></a>
## E-L3-GIT-BUG · 零服务器任务后端对照（限时验证）

[git-bug](https://github.com/git-bug/git-bug)：分布式、离线优先的任务追踪，任务以 git 对象存于 refs、随 push/pull 同步；Go 实现，CLI/TUI/Web UI 与 GraphQL API，带 GitHub/GitLab/Jira 桥接；GPL-3.0-or-later。

角色：与 Vikunja 并列的对照候选，代表“零服务器、随仓库分布式”的另一条路。已知冲突：看板语义弱（排序/泳道需另行承载）、无 webhook（观测靠 refs 轮询）、任务 content 进 git refs 与“content 归第三方服务器”的统一律相悖。若验证胜出，须显式接受模型例外并记入[来时路](../design/references/decision-history.md)。采用边界同上：独立进程/CLI 调用，不 vendor（GPL 义务隔离）。

## 2026-08-30 provider 动作复核

本次为 v0.15.0 的客户端动作合同复核固定 Vikunja 实际行为，不从 README 推断：

- v2.5.0 的 task-bucket API 明确说明，移入 Done bucket 会把 task 标为 done，移出会取消 done；重复移到同一 bucket 是 no-op（[`task_bucket.go`](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/routes/api/v2/task_bucket.go#L38-L46)）。
- 实际 model 代码在 bucket move 事务中比较旧/新 bucket，修改 `task.Done`/`DoneAt`，随后发出 `TaskUpdatedEvent`（[`kanban_task_bucket.go`](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/kanban_task_bucket.go#L87-L231)、[事件派发](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/kanban_task_bucket.go#L248-L259)）。这证明“拖入 Done”不是纯 UI 排序，而是 provider 内一项真实状态变化。
- `TaskUpdatedEvent` 被注册为 webhook event，payload 重载完整 task 与 doer；更新事件的 doer 来自实际认证主体（[listener 注册](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/listeners.go#L60-L69)、[doer 派发](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/tasks.go#L1599-L1613)、[webhook 数据重载](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/listeners.go#L1402-L1430)）。
- outbound webhook 只有 payload HMAC、可选 Basic Auth、User-Agent 与 Content-Type，没有单独 delivery ID header（[`webhooks.go`](https://github.com/go-vikunja/vikunja/blob/ef2200e9429c5cc42f5c1811433418bfcc72b3aa/pkg/models/webhooks.go#L345-L371)）。因此 HCTL 不能用“收到第几次 HTTP POST”作动作身份；幂等依据应由 binding revision、external task ID、updated/remote revision、Done transition 和映射后的 actor 组成，并在接纳前回读 current task。

结论：Vikunja 原生 Done 可以表达 owner human 的完成请求，但不能直接证明 HCTL Task 完成。缺 doer、明确前后变化、版本或 fresh readback 时只作 Snapshot；满足条件时归一到与 CLI/Workbench 相同的「完成 Task」命令，仍由 Task reducer 独立验收。普通创建、编辑和非终态移动继续按 backend-authoritative content 处理。

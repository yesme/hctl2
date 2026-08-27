# Rakazo

> 类别：③ 独立 Agent 产品 · 证据编号：E-RAKAZO<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-rakazo"></a>
## E-RAKAZO · Rakazo

### 核心价值与跨层画像

Rakazo 定位为 [Grok Bot](./grok-bot.md#e-grok-bot) 的自托管开源替代:每个 bot 拥有自己的聊天线程、Markdown 记忆、cron 例程、示范教学技能和一台带图形桌面的沙箱"计算机";模型自选(BYOK 或订阅 OAuth),agent loop 用 Pi 内嵌在自家 API/worker 进程中自建,不适配 Claude Code/Codex 等外部 Harness。仓库在审计时只有 9 天历史、单人主导、迭代极快;但它最值得收录的不是产品形态,而是一个反差事实:**这个 early beta 在运行治理上的工程严谨度远超其产品成熟度**,并有属性测试、Postgres 集成测试、崩溃恢复拓扑测试和真模型 canary 背书。三块经源码验证的亮点:

1. **三层带隔离栅栏的租约加幂等效果账本(L2)**。Run 租约以 CAS 认领、`leaseFence` 递增、60 秒心跳续租、续租失败即中止,每次执行留 Attempt 行;计算机执行租约与屏幕租约(`runId:fence`)各自独立,防止旧执行回抢;恢复语义是双保险——事件驱动入队之外,一个用 Postgres advisory lock 选主的对账器每 30 秒兜底扫描过期租约与到期例程。所有非只读工具调用前先写 `ExternalEffect` 意向行(幂等键=工具调用 ID),完成后置 completed;重放时已完成的直接返回旧结果,**状态不确定的非幂等工具直接拒绝重复执行**。状态机显式断言合法迁移(`failed→queued` 可重试,`completed/cancelled` 终态)。
2. **等人状态与人/机双租约接管(L2/L1)**。`waiting_input`(等回答)与 `waiting_takeover`(等上屏)是 Run 的一等状态,挂起前强制把工作区 checkpoint 到持久存储;接管期间执行租约转 24 小时保持、控制权交给人,人的控制租约限时、到期由后台任务自动回收,释放时自动找回等待中的 Run 重新入队恢复。
3. **供应商中立的可移植工作区(L1)**。沙箱经统一 `SandboxProvider` 契约支持 Docker/E2B/Daytona/本机等后端并逐后端探测能力降级(非图形后端过滤图形工具、分不出屏时显式报错而非静默排队);文档明确 `providerRef` 只是"加速路径而非持久数据",机器消失即从 checkpoint 重建——与 HCTL2"领域对象不被进程反向定义"同源。Docker 路径有特权分离:API/worker 进程不持有 Docker socket,由独立 supervisor 服务代管。另有值得单记的 L4 细节:Markdown 记忆文档带整数修订号与全量修订表,**每条修订携带 `sourceRunId`/`sourceThreadId` 出处**,可导出导入;这是"知识修订可溯源到产生它的 Run"的野生同构实现。

反面证据同样直接。官网称"给 bot 演示一次工作流,它存成你能读、能改、能提交的纯 Markdown",实现里并不存在可提交的 Markdown 工作流文件:例程是无版本号的数据库行(name+prompt+cron),触发时读**当前** prompt;示范教学的产物是 JSON playbook(由确定性代码而非 LLM 从录屏编译,这一点反而诚实),可编辑但无修订历史,Run 不记录执行时用的是哪个版本。官网的"Approvals that hold"经全库检索只是 playbook 文本约定加 ask 机制,没有任何策略引擎;防越权靠提示词自律("Prefer tools over claiming you already did the work")。凭证边界的"credentials never leave your environment"只在纯 Docker 自托管路径严格成立——选云沙箱时浏览器登录态在第三方 VM 上,长历史压缩外包给 Supermemory SaaS。

### 审计基线

| 基线 | 状态 | 可支持的结论 |
| --- | --- | --- |
| [`v0.1.0-beta / 53b119a6`](https://github.com/elie222/rakazo/releases/tag/v0.1.0-beta) · 2026-08-13 | 已发布 prerelease | 产品骨架与自托管路径 |
| [`main@90572cb2`](https://github.com/elie222/rakazo/tree/90572cb2bcab4458aebbe1994b3ffbc9ddfac339) · 2026-08-21 | 审计快照(本地克隆核验 HEAD 与许可证) | 上述租约/效果账本/接管/工作区机制全部在此基线经源码验证 |

许可证为 [Apache-2.0](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/LICENSE)。仓库创建于 2026-08-13,单一维护者贡献约 92% 提交,处于极速演进期;能力判断以固定源码为准,官网叙事必须逐条对照实现甄别。

### 采用与边界

HCTL 对照 L2 采用:三层租约的隔离栅栏组合、意向-完成两段式幂等效果账本(含"不确定态拒绝重试非幂等操作")、advisory lock 选主的兜底对账器、挂起前强制 checkpoint 的等人状态,以及 `failed→queued` 显式可重试的状态机断言。对照 L1 采用:供应商中立契约与"可移植工作区为唯一持久边界、供应商引用仅缓存"的纪律、逐后端能力探测与显式降级、执行进程与容器特权的分离,以及人控制租约与机执行租约分开计时的接管模型。对照 L4 采用:记忆修订携带 Run 级出处的最小可行样板。

明确不采用:无版本冻结的例程与 playbook(Run 必须绑定冻结的 Workflow Revision,这正是 Rakazo 反向验证的差异化空间);提示词自律代替确定性 Gate 与策略引擎(与"证据高于自述"相反,是现成反例);模型 loop 与业务同进程(HCTL 的 Harness 边界要求进程级隔离与无隐藏写权限);把长期记忆压缩外包给第三方 SaaS;一 bot 一线程的二元对话不映射为 Room。Rakazo 没有 L3:Task 仅是 prompt+status,无契约、无验收、无看板。

主要证据(固定到 `90572cb2`):

- [仓库](https://github.com/elie222/rakazo)、[v0.1.0-beta](https://github.com/elie222/rakazo/releases/tag/v0.1.0-beta)、[官网](https://rakazo.com/)与[许可证](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/LICENSE)
- 领域对象与治理:[Prisma schema(Run 租约 L342-376、ExternalEffect L392-409、Routine L411-432、MemoryRevision L514-526)](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/db/prisma/schema.prisma)、[Run 状态机断言](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/core/src/run-state.ts)、[执行器(租约/效果账本/工具分发)](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/adapters/src/executor.ts)与[选主对账器](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/adapters/src/job-reconciler.ts)
- 等人与接管:[事务性线程事件](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/db/src/events.ts)、[API 路由(ask/takeover/release)](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/apps/api/src/router.ts)、[屏幕租约](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/core/src/screen-lease.ts)与[签名屏幕代理](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/apps/api/src/screen-proxy.ts)
- 沙箱与工作区:[计算机生命周期](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/adapters/src/computer-lifecycle.ts)、[工作区 checkpoint](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/adapters/src/computer-workspace.ts)、[supervisor 特权分离](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/infra/sandboxes/supervisor/src/supervisor-logic.ts)、[适配器契约](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/adapter-kit/src/interfaces.ts)与[计算机运行时文档](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/docs/computer-runtime.md)
- 记忆与教学:[Markdown 记忆(修订+出处)](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/memory/src/index.ts)、[确定性 playbook 编译](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/core/src/teach-playbook.ts)与[官网 Markdown 声称处](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/apps/www/src/pages/index.astro#L107)
- 加密与凭证:[secret 加密存储](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/packages/adapters/src/secrets.ts)与[自托管文档](https://github.com/elie222/rakazo/blob/90572cb2bcab4458aebbe1994b3ffbc9ddfac339/docs/self-host.md)

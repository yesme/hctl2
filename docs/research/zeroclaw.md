# ZeroClaw SOP

> 类别：③ 独立 Agent 产品 · 证据编号：E-L2-ZEROCLAW<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-l2-zeroclaw"></a>
## E-L2-ZEROCLAW · ZeroClaw SOP

ZeroClaw 不能直接提供 HCTL Workflow，但它的 SOP 引擎是少见的 L2 邻近实现证据：每个 SOP 的准入策略支持 `parallel`、`hold`、`coalesce` 和 `drop`，Run 可以持久化并在重启后恢复；人工介入/检查点、经过身份认证的审批组与法定票数、只追加审批审计、拒绝作用域版本已经过期的提示、修改/修订、步骤级工具范围，以及重试/跳转，共同形成了可复用的 Gate 与准入失败用例库。

HCTL 只借鉴绑定版本的人类决策、准入与背压，以及默认拒绝的策略测试；不把 SOP 当作 Workflow Revision，不把事件触发当作 Start 授权，不把 Agent `sop_advance` 当作 Verdict，不把工具收据当作 HCTL Receipt，也不把 ZeroClaw Run 数据库当作领域权威事实。固定版本为 [`v0.8.4 / a56c345d`](https://github.com/zeroclaw-labs/zeroclaw/tree/a56c345d51dd8ab562e9351e0d4ab83f6a741db9)（MIT 或 Apache-2.0）。[语法说明](https://github.com/zeroclaw-labs/zeroclaw/blob/a56c345d51dd8ab562e9351e0d4ab83f6a741db9/docs/book/src/sop/syntax.md)与[运行时契约](https://github.com/zeroclaw-labs/zeroclaw/blob/a56c345d51dd8ab562e9351e0d4ab83f6a741db9/docs/book/src/sop/how-it-works.md)对默认持久化行为仍有冲突，初始化失败时还会降级到进程内内存，因此不能承担 HCTL 的权威事实。

## 复核记录

- **2026-08-24**：SOP 子系统占其全史提交路径不足 1%，是该个人助理运行时的边缘外围；按"相邻实现证据"降权引用是正确的。

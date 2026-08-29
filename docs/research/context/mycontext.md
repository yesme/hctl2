# MyContext

> 类别：④ Context 管理 · 证据编号：E-MYCONTEXT<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-mycontext"></a>
## E-MYCONTEXT · MyContext

MyContext（openTrinity/mycontext）是"个人工作上下文"管线：把 IM、日历、会议、邮件等个人工作源做增量采集、规范化/派生、检索与图查询；AI 只是受控消费者，故障显式降级。它对 HCTL2 的独特价值是 Context 成本纪律的对照样本，以下机制均经源码复审：双层轮询探针 + 单事务 outbox 采集；CJK bigram 全文检索作为常驻零费用检索层；kl-graph 的 RRF 多路机械融合与逐 chunk 抽取缓存（无本地相关性分类器，外部 rerank 挂点默认关闭）；三级可见降级（agent → 有来源的本地检索列表 → 建索引提示）；本地向量索引已实现但因 embedding 费用不接线；LLM 画像抽取管道建成后被实测下线，换成零模型的确定性测量——"能用规则就不用模型"在它那里是执行过的决定，不是口号。

采用边界：**仅参考行为**。采纳"多来源、增量采集、规范化/派生、检索与图、AI 只是受控消费者、故障显式降级"的分层思想与成本纪律；不照搬其个人数字分身产品边界，也不把 SQLite vault 当成 HCTL2 Context 的定义。项目为开发者预览，许可证 Elastic License 2.0（源码可见、非 OSI），只作设计研究、不复制实现。固定基线 [`81b3c7ac`](https://github.com/openTrinity/mycontext/tree/81b3c7ac178dbf141ca97cbe6b6682f73e3d3199)（2026-08-22 源码复审确认 HEAD 未变）；详细审计与 §7.3/§8 成本纪律对照见 [Context 设计备忘录](../../../.memo/design/context-design-20260819.md)。

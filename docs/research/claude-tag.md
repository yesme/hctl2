# Claude Tag

> 类别：② Agent 协作平台 · 证据编号：E-L4-CLAUDE-TAG<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-l4-claude-tag"></a>
## E-L4-CLAUDE-TAG · Claude Tag

Claude Tag 为 L4 提供产品行为证据。它最独特的设计是：一条 Slack 讨论串就是多人可见的工作会话，频道成员可以继续会话，也可以中途调整方向；讨论串及其上下文持久保存，托管沙箱则可以回收后重建。Agent 使用限定在频道范围内的服务身份、访问权限和记忆范围，不会冒充发起人。Checklist、定时 Routine、频道监听和代码仓事件还展示了低噪声的异步协作投影。

HCTL 借鉴持久 Room 与临时运行环境分离、多人共同引导、Agent 独立身份和受作用域约束的访问控制，并且只把 Checklist/Routine 当作投影或触发器。Slack 频道或讨论串不映射为 Project/Room，Checklist、Memory 或 Routine 不成为 Task、Run 或知识的权威事实，频道成员身份也不能绕过 HCTL 的权限与 Gate。Claude Tag 是闭源的公开测试产品，只能作为行为证据，不能移植源码，也不承担 L1 运行环境方案。

基线按公开资料日期固定为 2026-06-23 Public Beta：[发布公告](https://www.anthropic.com/news/introducing-claude-tag)、[工作原理](https://claude.com/docs/claude-tag/concepts/how-it-works)、[Agent 身份](https://claude.com/docs/claude-tag/concepts/agent-identity)、[Routines](https://claude.com/docs/claude-tag/users/proactivity)、[Memory](https://claude.com/docs/claude-tag/users/memory)。

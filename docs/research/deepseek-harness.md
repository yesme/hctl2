# DeepSeek Harness / Cordis

> 类别：① Coding Harness · 证据编号：E-L1-DEEPSEEK-HARNESS<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-l1-deepseek-harness"></a>
## E-L1-DEEPSEEK-HARNESS · DeepSeek Harness / Cordis

### 设计亮点

DeepSeek Harness 在 L1 和跨层架构上都有独特价值。它没有把模型适配器、工具注册、Session 日志和 Agent Loop 写死为固定模块，而是把它们都实现为 Cordis 插件：共享 Context 提供服务、类型化事件、依赖注入和可撤销注册；Profile、Bundle 与 Patch 共同组合出实际运行的插件树。Service Definition、Provider、Consumer 三段式能力边界，让使用者依赖抽象能力，而不是具体插件名称。

它的 Session 设计同样扎实：所有对模型可见的内容都必须能从只追加事件日志重建；运行中崩溃时，系统追加“本轮被中断”的结束事件，而不是截断历史；遇到未知的必需事件或数据结构版本时拒绝恢复。HCTL 可以采用这些能力边界、插件生命周期测试和可重建日志，并把最终解析出的 Profile、插件集合与配置摘要冻结进 Attempt/Run Manifest。

### Cordis 论文实际证明了什么

[Cordis 论文固定版本](https://github.com/cordiverse/paper/tree/948a07b369c62adb3b12e102458be5c18dfb69b9)（Draft v8，2026-08-13）把进程内修改建模为带逆操作（inverse）的可逆 effect，把依赖上下文建模为会随服务出现或消失而重新解析的 reactive coeffect。这个模型解释了为什么注册监听器、提供服务、挂载子插件和热重载可以使用统一的生命周期管理。

论文也明确给出了保证范围：

- 可逆性只适用于系统边界内、可以独占修改并恢复的具体位置；边界不是按“文件或网络”这种介质一刀切。系统独占且可恢复的私有文件可以在边界内，向其他主体可见的输出则已经越界。论文还区分资源获取与对外输出；越界输出仍需延迟提交或应用级补偿；
- 依赖声明和服务注入不是安全沙箱。不可信代码需要语言运行时沙箱、操作系统强制隔离、容器或虚拟机；同一用户下的普通独立进程只能隔离崩溃，不能阻止它访问仓库、网络或凭据；
- 细粒度拆分可以消除依赖环，但也可能带来大量集成组件、命名、配置和认知负担；
- 接口漂移、键冲突、行为契约和多版本解析仍是开放问题；
- 经验材料来自单一语言和单一生态，没有受控对照，也没有性能或生产率的量化结论。

论文仓库没有声明许可证，因此这里只概括其观点，不复用论文文本或图表。

### kxn 的批评如何使用

[kxn 的评论](https://mp.weixin.qq.com/s/O3A4RpQM4jZz_XkDFvORyQ)指出：插件即使声明了依赖，仍可能因为钩子顺序、优先级和共享行为而互相干扰；第三方插件再叠加版本冲突，维护复杂度会迅速上升。这个批评与论文自身列出的限制基本一致，提醒我们“可组合、可卸载”不等于“行为没有冲突、生态自然可治理”。

评论作者也明确说明没有真实运行项目，主要依据源码分析。因此本文只把它当成二级审查问题，不用它证明 DSH 的实现、性能或成熟度。

### HCTL 的取舍

HCTL 不采用“Everything is a Plugin”，而采用[固定内核与受控端口](../design/spec/system.md#固定内核与受控端口)：

- Repo/Project/Task/Run 身份、命令准入、权限、版本与证据、领域归约器、持久账本、隔离栅栏和 Receipt 固定在内核中；
- harness、运行时后端、任务源、workflow engine、Chat 端口和渲染组件通过类型化端口进行替换；
- 多个提供方可以声明同一个带命名空间和版本的能力，唯一的是一次已经选定的权威绑定；插件加载顺序和钩子优先级不能决定权限或语义结果；
- `Extension Revision` 与 `Resolved Port Binding` 固定代码、接口、数据结构、配置、依赖图和信任级别；Run、Attempt、Invocation、Task Source 与外部聊天渠道在各自正确粒度冻结绑定；
- 响应式依赖只用于准入前发现或纯展示/遥测；提供方在活动执行中消失时安全暂停或失败，不能在原执行内自动改绑；
- 进程内注销器（disposer）只能撤销注册，不能声称已经回滚越过系统边界的输出；
- 进程内扩展等同受信任代码；不可信代码必须使用操作系统强制隔离和能力削减后的代理接口；
- 第一阶段只允许第一方或经审计的进程内扩展，不建设任意第三方插件市场。

DSH 由模型生成的 Workflow 仍缺少冻结版本、持久恢复、Gate 和语义 Receipt，只能作为 L2 边界证据，不能承担 HCTL 的 Workflow 治理。

固定实现基线为 [`master@47f94385`](https://github.com/deepseek-ai/deepseek-harness/tree/47f943859bef60e4160492346772ded9b24f765a)（2026-08-13，`0.1.0-rc.5`，[MIT](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/LICENSE)）。官方将其标为开发者预览版，允许破坏性变化；[`BENCHMARK.md`](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/BENCHMARK.md)只给出运行方法，没有公开结果。

主要源码与文档：[架构](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/architecture.md)、[Cordis 入门](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/cordis-primer.md)、[能力边界](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/capability-seams.md)、[Session](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/subsystems/session.md)、[持久化](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/subsystems/persistence.md)和[Workflow 边界](https://github.com/deepseek-ai/deepseek-harness/blob/47f943859bef60e4160492346772ded9b24f765a/docs/subsystems/workflow.md)。

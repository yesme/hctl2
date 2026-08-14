# 基础、目标与原则

> Status: Normative · Draft v0.7.0<br>
> Parent: [HCTL2 设计规范](./README.md)

## 一句话定位

> HCTL2 是把人主导的目标塑形与机器驱动的可验证施工连接起来的 Repo-local、多 Harness 项目协作系统。

HCTL2 以 Git Repo 为边界，以 Project 组织目标，以 Room 承载意图与协作，以 Task 追踪承诺，以 Run 治理经过授权的自动施工。正常执行默认 headless；terminal 是观察、诊断和接管路径，而不是用户理解项目的主入口。

产品姿态可以概括为：

> **Project-scoped、Room-mediated shaping、Task-tracked、Run-executed。**

## 为什么需要 HCTL2

软件开发正在从“一个人操作一个 IDE 或终端”转向“一个人同时管理多个不同能力、不同上下文和不同权限的 Coding Harness”。现有工具通常把其中一个局部做得很好：聊天与上下文、异步 Task、worktree/terminal，或通用 Workflow Engine；但它们没有共同回答项目生命周期中的四个问题：

1. 我们真正要解决什么，讨论依据是什么？
2. 哪些内容已成为可排程、可验收的承诺？
3. 哪些自动施工已获授权，凭什么可以继续或通过 Gate？
4. 哪个具体执行实例在何处运行，如何观察、恢复或接管？

缺少这组分层后，会出现以下失败模式。

### 多 Harness 被压缩成多个终端

terminal multiplexer 能保存进程，却无法稳定表达 Participant 的角色、TaskRevision、审批证据、授权范围和下一步动作。Tab、pane 和 worktree 名称也不能承担长期项目身份。

### 人成为机械消息总线

人在 author、reviewer、tester 和 security reviewer 之间复制上下文、等待完成再转发 feedback。这是高出错的机械劳动，不是人的意图与判断优势。

### “Agent 完成”被误当成“项目完成”

Harness 完成一轮、进程退出、代码提交、review accept、CI 通过、provider Issue Closed 和 HCTL Task 验收是不同事实。把任意一个状态当作完成，会在 revision 变化、重试、迟到结果和外部同步失败时失去正确性。

### Context 无来源、无版本、不可复现

把整段聊天或一个 lead Agent 的自由总结发给 worker，无法回答它当时看到了什么、遗漏了什么，也无法在 failover 后重现同一个 obligation。

### UI、领域与 runtime 相互污染

如果 Room、Project 或 Task 直接等同于 Conversation、session、worktree 或 terminal，纯讨论、多 Run、权限隔离、retry 和 crash recovery 都会破坏映射。

## 目标体验

理想的完整旅程是：

1. 用户进入 Repo Room，引用代码、Artifact、Commit 或 Memo，与多个 Participant 做 bounded research。
2. 话题成型后，用户把相关来源和 Context 提升为具名 Project。
3. 用户在 Project Room 中塑形目标、范围和 acceptance，并把承诺提炼成 TaskRevision。
4. 简单工作可由人或一次 bounded RoomInvocation 完成；它不需要 Run。
5. 需要持久自动施工时，用户批准 WorkflowRevision，再显式 Start Run 授予 bounded autonomy。
6. Run headless 推进；需要澄清、决定或授权时，系统创建 Request 并投影回 Project。
7. 只有需要观察或接管精确 Attempt 时，用户才打开 Execution Chat Projection 或 terminal attach。
8. 执行 result 与 evidence 逐层回流；只有通过 revision、policy、acceptance 和 Receipt 校验后，Task 才能语义完成。

用户应始终能回答：

- Project 要交付什么；
- 哪些 Task 正在进行、待 review 或被阻塞；
- 哪个 Run/Node 在等待，为什么；
- 当前需要谁提供什么；
- 哪个 Harness、Skill、Context 和 permission 正在执行；
- 结论绑定哪个 revision 与 evidence。

## HCTL2 要解决什么

- Repo 级 Harness catalog、自发现、能力探测和偏好排序；
- Repo/Project Room 中的多参与者结构化协作；
- 可追踪、可排序、可验收的 Task；
- 冻结 Workflow 的持久 Run；
- candidate fallback、quorum、regate 与 revision fencing；
- Git/worktree/PR/Receipt 的确定性验证；
- 低噪声 Attention、Request 与按需商议；
- Harness 结构化事件与 PTY/TUI 逃生通道；
- Workbench、control、Engine、agentd 或 runtime 重启后的对账恢复。

## HCTL2 不解决什么

- 不重新实现 LLM、Coding Harness、通用 Workflow Engine、terminal emulator 或 multiplexer；
- 不用自然语言 prompt 替代 branch protection、Receipt、authority 或 policy；
- 不把不同 Harness 的能力伪装成完全相同；
- 不让每项工作都必须聊天或必须创建 DAG；
- Phase 1 不建立多人组织、云队列、browser/mobile 客户端、多 host 或 Conductor HA。

## 产品原生核心与架构最小内核

HCTL2 不是四个现成 UI 的拼装器。其产品原生核心是一个 **repo-scoped project semantic control plane**：

1. **Repo–Project lifecycle**：Repo 注册、Project create/update/archive/restore，以及不随外部 provider 改变的 stable identity；
2. **Project continuity**：Room、Task、Run、Artifact、Request 和 evidence 始终回到同一个 Project，替换 Harness、session、terminal 或 SaaS 后仍可继续；
3. **Project-driven control**：系统根据 Project context、role、Expertise、revision、capability 和 evidence 判断下一步允许发生什么。

即使更换全部 UI、Chat surface、Task provider、Workflow Engine 和 terminal client，下列最小内核也必须保留：

| 最小内核能力 | 必须保持的性质 |
| --- | --- |
| Stable identity 与 binding | Repo、Project、Task、Room、Run 及外部对象的身份不随客户端/provider 漂移 |
| Command admission 与 authority | 每个动作验证 actor、scope、capability、expected revision、policy 与 idempotency |
| Revision、evidence 与 semantic validation | Task/Workflow/Context freeze、Verdict、Receipt、acceptance 与 stale-result fence 可验证 |
| Execution governance | Run Manifest、Obligation/Seat/Attempt、candidate fallback、quorum 与 regate 由 HCTL 语义控制面协调 |
| Durable ledger 与 reconciliation | inbox/outbox、provider read-back、crash recovery 和 projection rebuild 保持跨组件一致 |

最小状态转换固定为：

```text
actor + typed command + target revision + evidence
  → hctl2-control/core validate
  → committed domain event + durable outbox intent
```

外部 provider 的事件先成为 observation 或 proposal；只有通过同一 admission boundary，才可能改变 HCTL 语义。

## 设计原则

1. **四层分责。** Intent、Commitment、Governance 和 Runtime 通过 typed seam 连接，不能隐藏跨层副作用。
2. **领域对象少而稳定。** Repo、Project、Task、Room、Run 是用户核心对象；Obligation、Seat、Attempt 只在需要时渐进披露。
3. **协作拓扑与控制拓扑正交。** Room 回答在哪里交流；Workflow/Run 回答谁有权自动推进。
4. **人的角色是意图与授权中心。** Planning 可以并行研究和建议，但不能替用户决定目标；Run 只能在批准的 envelope 内自动推进。
5. **机械动作必须确定性。** mention、spawn、retry、fallback、gate 和 merge 不依赖模型是否记得调用工具。
6. **Chat 不是数据库。** Message 可以形成 Proposal；正式变化必须走 typed command、revision check 和 authority check。
7. **Context 必须可解释。** 每次调用冻结 ContextManifest、来源、digest、Skill 和 capability/permission snapshot。
8. **Evidence over progress。** Harness progress、自述 verdict、屏幕状态和 provider Closed 都不能越过 acceptance 与 Receipt。
9. **Headless by default。** 高层 status、diff、trace 和 Request 优先；terminal 是诊断与 takeover 路径。
10. **At-least-once correctness。** 外部副作用使用 idempotency、durable inbox/outbox、fencing 和 reconciliation。
11. **Runtime identity 可替换。** provider conversation、process、worktree、mux target 和 terminal connection 都不是 Project/Task/Room identity。
12. **外部事实按字段授权。** Linear/GitHub 可拥有配置的 operational fields，但不接管 HCTL `task_id`、adopted TaskRevision、acceptance、Run binding 或 semantic completion。
13. **复用实现，不继承 donor 领域。** 外部项目只贡献被验证的切片；其对象名称和数据库 schema 不进入 HCTL 公共模型。
14. **Workbench 无隐藏特权。** Workbench、CLI 和未来客户端通过同一 command/query service；任何界面都不能直接写 SQLite、provider mirror 或 Conductor task。
15. **尽早 dogfood。** 稳定版本 N 通过正式产品 seam 治理隔离环境中的 N+1；bootstrap/recovery 脚本不成为第二事实源。

## Planning 与 Build 的授权边界

Planning/Shaping 与 Run/Automated Build 是两种控制制度，而不是必须存在的两种 Room：

| 制度 | 推进权 | 主要产物 |
| --- | --- | --- |
| Planning / Shaping | 人是意图和授权中心；系统可研究、建议、汇总 | spec、ADR、Task、Artifact、Workflow proposal 或普通 Git 文件 |
| Run / Automated Build | control 在冻结 Workflow 与 policy 内自动推进；人处理例外 | code/doc、Verdict、Receipt、PR、Run history |

Approve Workflow 与 Start Run 是两个动作：前者确认施工图，后者允许系统消耗资源并产生副作用。Start Run 至少冻结 WorkflowRevision、TaskRevision bindings、repo base、roles、authorized candidates、fallback/capability/permission/budget policy，并生成可预览的 Run Manifest。

Run r1 执行冻结 revision 时，Project Room 可以继续讨论 r2。除明确标为 runtime-mutable 且带 Receipt 的 placement 参数外，candidate set、budget、permissions、quorum、scope 或 acceptance 的变化都必须产生 replacement Run 或新 TaskRevision，不得原地漂移。

## Phase 1 设计姿态

Phase 1 交付 HCTL-native Workbench，但所有领域状态和后台执行都不依赖窗口存活。技术基线为 Rust control/core/agentd、Electron + React Workbench、Repo-local SQLite、Git、Conductor、Tiptap、React Aria、React Flow、xterm.js，以及通过 contract bench 选定的一个 RuntimeBackend。

具体交付范围、排除项和 B0–B6 自举路线见[Phase 1 与分级自举](./delivery/phase-1-and-dogfooding.md)。

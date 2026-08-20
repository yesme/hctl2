# HCTL2

HCTL2 是把**人主导的目标塑形**与**机器驱动的可验证施工**连接起来的项目协作系统。它的交付物与承诺以 Git 仓库为边界，协作与治理随用户走，默认部署在本地；面向同时使用多个 Coding Harness（Codex、Claude Code、OpenCode 这类编码代理工具，下称 Harness）的开发者。

产品姿态一句话：

> 目标以 **Project** 为界，塑形在 **Room** 中发生，承诺由 **Task** 追踪，施工交给 **Run** 治理。<br>
> （Project-scoped · Room-mediated shaping · Task-tracked · Run-executed）

> [!IMPORTANT]
> HCTL2 当前处于设计阶段。权威基线是 **草案 v0.10.2**；仓库里还没有可安装应用、CLI、构建脚本或测试套件。

## 为什么需要它

软件开发正在从“一个人操作一个 IDE 或终端”转向“一个人同时管理多个能力、上下文和权限各不相同的 Harness”。现有工具往往把某个局部做得很好——聊天与上下文、异步任务、worktree（Git 工作树）与终端、通用工作流引擎——但它们没有共同回答项目生命周期里的四个问题：

1. 我们真正要解决什么，讨论依据是什么？
2. 哪些内容已经成为可排期、可验收的承诺？
3. 哪些自动施工已获授权，凭什么可以继续或通过评审？
4. 哪个执行实例正在哪里运行，如何观察、恢复或接管？

缺了这组回答，就会反复出现五种失败：

- **多 Harness 被压缩成多个终端。** 终端复用器能保住进程，却说不清参与者的角色、任务契约、审批证据和下一步动作；标签页和 worktree 的名字扛不起长期的项目身份。
- **人退化成消息总线。** 在作者、评审者、测试者之间复制上下文，盯着一个完成再转发给下一个——这是高出错的机械劳动，不是人的意图与判断优势。
- **“Harness 说完成了”被当成“项目完成了”。** Harness 跑完一轮、进程退出、代码提交、评审通过、CI 绿了、外部 Issue 关了、任务验收通过，是七件不同的事实；混为一谈，就会在版本变化、重试、迟到结果和外部同步失败时丢掉正确性。
- **上下文无来源、无版本、不可复现。** 把整段聊天或某个模型的自由总结丢给下一个执行者，事后无法回答它当时看到了什么、漏了什么，也无法在故障切换后重现同一份交付义务。
- **界面、领域与运行时互相污染。** 如果 Room（协作聊天室）、Project、Task 直接等同于会话、进程或 worktree，那么纯讨论、多次施工、权限隔离、重试和崩溃恢复都会破坏这层映射。

## 四个阶段与四个模块

HCTL2 的心智模型是一条从意图到运行的链，四个领域模块各自拥有其中一段的事实：

```text
意图 Intent      ──提炼──▶  承诺 Commitment  ──授权──▶  治理 Governance  ──分派──▶  运行 Runtime
Project · Chat Room        Task · Kanban              Run · Workflow             Agent · Terminal
```

结果沿相反方向回流：运行层交出提案与证据；治理层校验后形成 Verdict（裁决）与 Receipt（凭证）；承诺层据此独立验收；意图层沉淀里程碑与长期经验。

这条链是心智模型，不是流水线：**一件事不必完整经历四个阶段。** 简单 Task 可以不创建 Run；Project 可以直接发起一次边界明确的 Harness 调用；纯研究或文档的 Project 可以从未施工。四个模块是语义分责，不是界面菜单；部署由正交的[三面架构](./docs/design/architecture.md)回答。从 Project 到 Agent，越靠前越接近人的意图，越靠后越是可替换的执行资源。

| 领域模块 | 操作场景 | 回答的问题 |
| --- | --- | --- |
| Project | Chat Room | 我们要解决什么，依据是什么？哪些讨论已经足够稳定，可以成为承诺？ |
| Task | Kanban | 已承诺交付什么，现在进行到哪里？ |
| Run | Workflow | 哪些自动施工已获授权，凭什么继续或通过？ |
| Agent | Terminal | 哪个执行实例在哪里，怎样观察、恢复或接管？ |

```mermaid
flowchart LR
    P["Project\nChat Room"] -->|提炼承诺| T["Task\nKanban"]
    T -->|批准自动施工| R["Run\nWorkflow"]
    P -->|批准无 Task Run| R
    P -->|一次有边界的调用| H["Agent\nTerminal"]
    R -->|分派执行| H
    H -->|提案与证据| P
    H -->|结果与证据| R
    R -->|Verdict / Receipt；正常完成可提交 Task 命令| T
    T -->|已验证里程碑| P
```

图中的短路边（Project 直达 Run 或 Harness）是显式设计，不是例外；它们不改变四个模块的事实所有权。

## 目标体验

理想的完整旅程是：

1. 用户进入 Repo Room，引用代码、Artifact（登记过的交付物）、Commit 或 Memo（沉淀的备忘），邀请多个 Participant（参与者）做有边界的研究。
2. 话题成型后，把相关来源和上下文提升为一个具名 Project。
3. 在 Project Room 中塑形目标、范围和验收标准，把承诺提炼成 TaskRevision（任务契约版本）。
4. 简单工作由人完成，或通过一次有边界的 RoomInvocation（从聊天室发起的单次调用）完成；它不需要 Run。
5. 需要持久自动施工时，先批准 Workflow（施工图），再显式启动 Run，授予有边界的自主权。
6. Run 默认在后台无界面推进；需要澄清、决定或授权时，系统创建 Request（请求卡）并投影回 Project。
7. 只有需要观察或接管某次精确执行时，用户才打开结构化执行投影或终端。
8. 执行结果与证据逐层回流；只有通过版本、规则、验收和 Receipt 校验，Task 才算语义完成。

无论走到哪一步，用户都应该随时能回答六个问题：

- Project 要交付什么；
- 哪些 Task 正在进行、待评审或被阻塞；
- 哪个 Run 在等待，为什么；
- 当前需要谁提供什么；
- 哪个 Harness 带着哪份上下文和权限在执行；
- 结论绑定哪个版本与哪些证据。

## 设计基线

- Project、Task、Run 和 Agent 模块的对象都有稳定身份，不能由聊天串、外部 Issue、工作流任务、worktree 或终端面板反向定义；
- Chat Room、Kanban、Workflow 和 Terminal 是操作场景，不是第二份领域事实；
- Workbench 是四个场景的集成客户端；适配后的第三方平台可以按场景替代或补充其中一个面板，也可以提供受控端口，但不会整体替代 Workbench 或领域模块；
- 所有适配器都使用同一命令、查询、事件和能力边界，没有隐藏写权限；
- Revision、Verdict、Receipt 和可核验证据高于进度、自述、屏幕状态和外部 Closed；
- 模型只能建议结果与下一位协作者。Task Completed 只接受有权人类命令，或绑定该 Task 的 Run 正常完成后的确定性归约命令；Task Cancelled 只接受有权人类命令。普通 Room 的临场执行边只由人类创建，Workflow 的归约器只能实例化冻结图中的边。

## 目标架构

```mermaid
flowchart LR
    subgraph Clients["可并存的场景客户端"]
        Bench["hctl2-workbench\nRoom · Kanban · Workflow · Terminal"]
        ChatClient["Feishu / Slack / Discord\nChat Room 客户端"]
        TaskClient["Linear / GitHub\nKanban 客户端"]
        WorkflowClient["第三方 Workflow UI"]
        TerminalClient["WezTerm / CLI\nTerminal 客户端"]
    end

    subgraph Control["hctl2-control · 四个领域模块"]
        P["Project\nChat Room"]
        T["Task\nKanban"]
        R["Run\nWorkflow"]
        H["Agent\nTerminal"]
    end

    Bench --> P
    Bench --> T
    Bench --> R
    Bench --> H
    ChatClient --> P
    TaskClient --> T
    WorkflowClient --> R
    TerminalClient --> H

    P --> ChatPort["Chat 受控端口"]
    ChatPort --> ChatSrv["chat server（Matrix 协议）"]
    T --> TaskSource["TaskSource 受控端口"]
    TaskSource --> TaskBackend["任务后端（本地任务服务器 / Linear、GitHub）"]
    R --> Engine["WorkflowEngine 端口"]
    H --> Agentd["agentd"]
    Agentd --> Runtime["Harness / RuntimeBackend"]

    Control --> DB["用户级 metadata 账本（SQLite）"]
    Control --> Core["hctl2-core · Git/SCM"]
```

部署视角上，系统分三个面：展示面（Workbench 与第三方客户端）、控制面（`hctl2-control`/`hctl2-core` 与治理账本）、执行面（各场景的 content 系统与 agentd 物理执行），详见[三面架构](./docs/design/architecture.md)。

HCTL2 自建 Workbench，不是为了重写通用 UI，而是因为 Repo/Project/Room/Task/Run 的导航无法无损套入任何现成工具的会话、终端或工作树主导航。Workbench 是组合式场景客户端；第三方客户端只替代或补充对应场景，例如 Feishu/Slack/Discord 操作 Chat Room，WezTerm 操作 Terminal。它们都使用相应模块的 Query/Preview/Submit/Subscribe，不获得跨模块捷径。

图右侧的受控端口提供底层能力，不等于场景客户端。同一平台可以兼任两者，但 client binding 与 authority binding 必须分开。`hctl2-control` 拥有领域命令与本地账本，`hctl2-core` 校验 Git/SCM 事实，agentd 拥有物理运行时观测，外部 Workflow Engine 只维护机械执行位置。即使把全部界面、聊天平台、Task 来源、工作流引擎和终端客户端都换掉，这套身份、权限、版本证据与恢复边界也必须原样保留——项目不随工具更换而丢失。

## 设计文档

设计文档分两层：设计层用产品语言回答为什么与怎么用；合同层（`docs/design/spec/`）承载精确的对象、状态机与写入合同，两层冲突时以合同层为准。建议先看愿景，再看设计地图和四个模块的设计正文，需要精确定义时下钻合同层。

- [愿景与设计原则](./docs/design/vision.md)
- [设计地图与文档纪律](./docs/design/README.md)
- [三面架构](./docs/design/architecture.md)
- [Project 与 Chat Room](./docs/design/project.md)
- [Task 与 Kanban](./docs/design/task.md)
- [Run 与 Workflow](./docs/design/run.md)
- [Agent 与 Terminal](./docs/design/agent.md)
- [第一阶段、验证与自举](./docs/design/delivery.md)
- [合同层总则](./docs/design/spec/README.md)（词汇分类、六族规则、归并对照）
- [四模块连接与端到端闭环](./docs/design/spec/connections.md)
- [系统边界与适配器合同](./docs/design/spec/system.md)
- [术语对照表](./docs/design/references/glossary.md)
- [从 HCTL 到 HCTL2 的来时路](./docs/design/references/decision-history.md)
- [非规范实现证据](./docs/design/references/implementation-evidence.md)

第一阶段面向单用户和 macOS/Linux。具体范围、技术栈、CLI、验证切片和未决问题统一记录在[交付文档](./docs/design/delivery.md)，不在各模块重复。

## 许可证

HCTL2 使用 [Apache License 2.0](./LICENSE) 发布。

# 系统边界与适配器合同

> 状态：规范性合同 · 草案 v0.9.1<br>
> 本文只定义四个模块共享的运行机制，不拥有 Project、Task、Run 或 Agent 的领域状态。

## 组件

| 组件 | 职责 |
| --- | --- |
| `hctl2-workbench` | 四个场景的集成客户端；提交命令、查询投影、显示事件 |
| `hctl2-control` | 唯一领域 command service、路由、权限、账本、outbox 和对账 |
| `hctl2-core` | Git/SCM、Revision、digest、Receipt 和合并事实校验 |
| agentd | Harness 发现、物理运行时、PTY、终端网关和主机观测 |
| Workflow Engine | 通过适配器保存 Run 的机械 token、task、timer、retry 和历史 |
| 第三方场景平台 | 提供部分场景客户端、受控端口或两者；两种 binding 与权威分离 |

Workbench 不是特殊内核。Workbench、CLI 和外部 UI 是场景客户端，使用 Query/Preview/Submit/Subscribe；外部 Chat、TaskSource、WorkflowEngine、Harness 和 RuntimeBackend 是由内核调用的受控端口。同一产品可以同时提供客户端与端口，但两者的 binding、权限和事实权威必须分开。agentd 是组件实现名（Agent 模块的本机执行守护进程），不是 Agent 模块本身。

## 固定内核与受控端口

固定内核是一个以仓库为边界的项目语义控制面（repo-scoped project semantic control plane）。固定内核实现四模块定义的稳定身份、Revision、权限、字段权威、领域归约与 Receipt；本文件只拥有共享 command envelope、扩展绑定、outbox/inbox、单写者和恢复机制。

即使把全部界面、聊天平台、Task 来源、工作流引擎和终端客户端都换掉，内核所守的身份、权限、版本证据、治理与恢复边界也必须原样保留——这是[愿景文档](../vision.md#产品原生核心与架构最小内核)中“产品原生核心与架构最小内核”在系统层的落点。

可以替换的端口包括：

| 场景 | 端口 |
| --- | --- |
| Chat Room | 消息/附件/身份转换与外部投递 |
| Kanban | TaskSource 读取、字段写回与快照 |
| Workflow | Workflow Engine 编译、注册、执行和回读 |
| Terminal | Harness、RuntimeBackend、终端网关与 attach provider |

第一阶段 Conductor 的管理/API 端点只绑定 loopback 或 owner-restricted local socket。未来非本地 transport 必须认证客户端，且仍只能由 control 经 WorkflowEngine 端口适配器发起变更，不能把 Engine mutation 暴露给场景客户端或其他本地进程。

每个扩展绑定都冻结代码版本、接口/schema、配置摘要、依赖、能力和信任级别。运行中不得因“发现更好的插件”而响应式改绑；提供方消失时安全暂停、失败或创建替代执行。

`trust_level` 只能由 control policy 根据允许的 trusted source 与精确 artifact digest 授予，扩展或 registry 的自我声明不能授信。discovery 只读取已配置 definition 和本地安装并执行无副作用探测，不得联网、安装/升级或修改 Harness/adapter 配置；install/upgrade 必须是用户显式提交的类型化动作，产生新 ExtensionRevision，后续解析再产生新 ResolvedPortBinding，不能改写活动绑定。

跨模块可引用的扩展信息只有两个最小概念：

- `ExtensionRevision`：扩展稳定身份的一份不可变版本，固定代码/制品摘要、接口与 schema、依赖、声明能力和信任级别；
- `ResolvedPortBinding`：一次已解析端口选择，固定 `binding_revision_id`、ExtensionRevision、provider/installation、配置摘要、credential reference、实测能力、权限作用域、health 和降级策略。

同一 `(port_kind, scope_id)` 的一次准入只能解析出一个 binding revision；提供方加载顺序、hook 优先级或 UI 选择顺序不得决定事实。Room 的 Chat 端口绑定、TaskBinding、EngineDeployment、EngineExecutionBinding 和 ExecutionSpec（含其接入方式字段组）都引用精确 ResolvedPortBinding；历史执行继续使用原 binding。credential reference 只定位 secret store 条目，不包含密钥。

跨 Project 使用的 Skill 是带稳定 ID、revision 和 digest 的共享定义，至少固定 manifest/instructions/assets/scripts、来源/license、兼容能力与依赖；更新创建新 revision，current pointer 只用于选择，ExecutionSpec 与 Run Manifest 必须冻结精确 ref+digest。Skill 可以提供方法并请求能力，不能授予权限、票权、委派或 Task 完成权。

进程内扩展等同受信任代码。普通独立进程只隔离崩溃；不可信扩展需要操作系统强制隔离和能力削减的代理接口。

## 场景端口

所有场景客户端共享四类操作：

```text
Query(filters, cursor) -> Projection
Preview(command draft) -> Effect summary + preconditions
Submit(typed command) -> accepted result or typed rejection
Subscribe(cursor) -> ordered events or resync snapshot
```

场景客户端只声明交互能力与降级行为；受控端口报告 provider 支持的读写能力，实际字段权威只能由对应模块的 authority binding 授予。外部平台可以拥有其场景 content 的 ground truth，以及明确授权的字段；但它不拥有治理——其数据库、thread、Issue、workflow task、Session 或 pane 不成为 HCTL 的身份、授权或判决来源。

渲染器、拖放、按钮和终端输入都只是 command client。未能提供等价预览、版本或权限信息时，动作必须禁用或安全暂停。

## 命令与跨服务正确性

改变事实的命令至少携带：

```text
command_id / idempotency_key
authenticated actor principal + actor kind/provenance + permission scope
target stable ID
expected revision or state version
frozen adapter binding
canonical input digest
```

actor kind/provenance 由认证场景入口或 control 内部 reducer 赋予，调用 payload、Room 消息、Harness 进程和 adapter 都不能自报为 human 或 workflow reducer。execution principal 只获得 Invocation/Attempt 冻结的窄能力。Task Completed 只接受有权 human actor 或 task-bound Workflow 的正常完成 reducer，Task Cancelled 只接受有权 human actor；普通 Room 临场 fan-out 只接受有权 human actor，Workflow reducer 只能实例化 WorkflowRevision 已冻结的边。

control 在一个 SQLite 事务中写领域事件、幂等结果和 outbox。外部适配器按同一 key 投递并回读；超时或 ACK 丢失保持“结果未知”，不能盲目重做。重复命令返回原结果，异载荷复用同一 key 被拒绝。

Receipt 证明的是已经校验的结果，不是另一个 writer。投影可以从事件重建，缓存或界面状态不能反向成为事实。

跨模块引用的规范摘要统一使用 RFC 8785 JCS 规范对象的 SHA-256，摘要字段自身不参与计算；每个领域 owner 只定义自己规范对象包含哪些字段。完整 Revision 的 `revision_digest` 与为评审选取字段生成的 `review_subject_digest` 是不同语义，即使某次字节恰好相同也不能互换。

## 外部权威副作用

包括远端 SCM 在内、会改变第三方权威事实的动作统一写成持久 EffectIntent/outbox 记录（executor = adapter），固定 owner ref、ResolvedPortBinding、operation、target、adapter 声明的 conflict scope、权限、规范输入摘要和幂等键。`conflict_scope` 表示同一远端资源的互斥域，不能仅因 close/reopen/update 等 operation 不同而拆开。本地 Git 变更是同族 EffectIntent（executor = core）：先由 control 持久化 intent/outbox，再由 core 执行和回读；Harness/model 不直接取得集成权。

adapter 只投递并回读；只有在它确认目标、版本和结果后，control/core 的校验事务才能写成功 Receipt。投递超时或 ACK 丢失保持结果未知，并占用 conflict scope，阻止同一资源上的重叠写。Harness 第一阶段不直接持有可绕过该端口的外部写凭据。

第一阶段不承诺自动发现或补偿任意带外写：Harness 不获得可绕过受控端口的外部写凭据；provider 被人在 HCTL 外修改时，只由对应端口回读为 Snapshot/drift，并阻止依赖旧版本的命令，直到用户通过该模块既有的采纳或对账动作处理。带外观测不能成为 ResultProposal、Artifact、Verdict 或 Receipt。

## 事实与存储

| 事实 | 权威来源 |
| --- | --- |
| 四模块的 metadata：领域账本、Room/Request 身份与绑定、Participant 名册、授权、租约记账与判决 | 用户级 metadata 账本 + control；一人多机连同一控制面账本 |
| RepoInstance 物理事实：worktree 与 ChangeSet 的本地归属、运行现场、单写锁与本地投影缓存 | RepoInstance SQLite + control |
| 共享 Project/Workflow 配置 Revision、Memo、Artifact/ChangeSet 不可变内容 | Git + core；control 账本保存本 RepoInstance 的 admission、current pointer 和 lifecycle 投影 |
| Workflow 机械位置 | 通过绑定访问的 Workflow Engine |
| Harness 进程、PTY、容器、主机与原始流 | agentd / RuntimeBackend 仅提供物理观测；绑定、租约和 lifecycle 仍由 control 记账 |
| Linear/GitHub 等外部字段 | 对应 provider；本地只存 Snapshot 和同步账本 |

存储拓扑固定为：

```text
~/.hctl2/                      # 用户级配置、Harness/Profile/Skill/Runtime 定义
                               # control.sqlite、control.lock —— 用户级 metadata 账本与写锁
<repo>/.hctl2/                # Git tracked · repo.toml、projects/、workflows/
                               # memos/、policies/、skills/、schemas/
<git-common-dir>/hctl2/       # untracked · state.sqlite（RepoInstance 物理事实账本）、lock、traces/、cache/
```

`<git-common-dir>/hctl2/` 是当前 RepoInstance 及其 linked worktree 的共享运行目录。HCTL 不写 Git 内部命名空间；密钥使用系统 secret store，不进入 Git、Room 或 Context。

用户级 Profile/Skill/Runtime 定义也以不可变 revision/digest 被引用；更新 current pointer 必须取得用户配置存储的排他锁并做 expected-version CAS。某个 RepoInstance 的活动执行只读已冻结 revision，不因另一个实例更新用户级 current pointer 而漂移。

## 单写者

用户级 metadata 账本同时只有一个 control writer：先取得 `~/.hctl2/control.lock` 排他锁，再以 CAS 推进其 writer generation；writer 可以搬迁（换机器、上服务器），账本身份不变。第一阶段单机部署时，它与 RepoInstance 服务同进程，行为不变。

每个 RepoInstance 同时只有一个 control writer：先取得 `<git-common-dir>/hctl2/control.lock` 排他锁，再以 CAS 推进 `control_writer_generation`。失败的第二实例只能连接现有服务或只读诊断。所有改变事实的下游 envelope 携带当前 generation，旧 generation 被拒绝。

每个 RuntimeBackend ownership scope 同时只有一个 agentd owner lease 和单调 generation；scope 至少覆盖相同资源 broker/socket/host namespace。agentd 必须先取得该资源侧的 OS lock、broker token 或等价排他原语才可执行输入、停止和接管。新 owner 必须先对账，旧 generation 的输入、停止、接管和结果一律失权；不能强制排他的 backend 只可观察，不开放这些写能力。

SQLite 锁不是外部副作用隔离。幂等键、generation、租约、outbox 和 readback 必须共同工作。

## 启动与恢复

恢复顺序固定为：

1. 取得 control/backend 的 OS/资源侧排他权，禁止旧 owner 继续执行动作；
2. 打开权威账本、验证 schema，恢复 inbox/outbox/租约，并 CAS 推进 writer/backend generation；
3. 回读外部 Task Source 和未确认副作用；
4. 查询 Workflow Engine、agentd runtime 和 core Git/SCM；
5. 将观测分类为运行、等待、丢失、被替代、孤儿或结果未知；
6. 隔离旧 generation，只重放可证明幂等且仍获准的动作；
7. 对账完成后才授予新的写入或输入租约。

UI 重载只重建投影。无法证明同一执行身份时，宁可标记 Lost/Interrupted 或要求人工对账，也不能自动接管或伪造成功。

## 安全边界

- Electron renderer、Web 内容、终端转义序列和外部消息都视为不可信输入。
- 打包后的 Electron 固定 `nodeIntegration=false`、`contextIsolation=true`、`sandbox=true`；narrow preload 只暴露具名 typed command，不暴露 raw `ipcRenderer`。禁止 remote runtime script/CDN，CSP 拒绝远程或未声明的可执行来源。
- 文件、Git、网络、凭据和进程能力由 control/core/agentd 授权，不交给渲染器。
- 敏感输入不进入 Room、日志、Context 或终端回放。
- 日志与 trace 使用稳定关联 ID，但不得包含密钥和完整敏感 payload。
- 第一阶段是单用户信任模型，不声称隔离同一 OS 用户的恶意进程。

# 当前架构一页摘要

> 事实快照：`origin/main @ 061bd7e`（草案 v0.15.0）。本页只归纳当前设计；精确对象、状态与写入者以 `docs/design/spec/` 为准。

## 四个模块

| 模块 | 生命周期位置 | 对应场景 | 当前职责 |
| --- | --- | --- | --- |
| Project | 意图 | Chat Room | 保存目标、范围、协作身份、来源、Participant、Context、Request、Memo、Artifact 与单次调用；消息正文由 chat server 承载。 |
| Task | 承诺 | Kanban | 保存可排序、可指派、可验收的承诺、冻结契约、来源绑定、字段权威和完成凭证；任务卡与流转由 task backend 承载。 |
| Run | 治理 | Workflow | 保存获准施工图、Run 授权、Obligation、Seat、Attempt、Gate、Verdict 与 Receipt；workflow engine 只保存机械位置。 |
| Agent | 运行 | Terminal | 把获准执行变成可观察、可隔离、可恢复的 Harness 进程，管理 ChangeSet、写租约、运行时、终端通道、结果提议与证据。 |

这四段是事实所有权，不是强制流水线：Task 可以没有 Run，Project 可以发起一次有边界的 Harness 调用；结果从 Agent 经 Run/Project 校验后回流，Task 始终独立验收。

## 三个面

| 面 | 当前组成 | 拥有的事实 |
| --- | --- | --- |
| 展示面 | Workbench、CLI、第三方场景客户端 | 不因客户端名称获得事实或特权；按动作目标调用 HCTL，或读写 provider content 与精确运行时。 |
| 控制面 | `hctl2-control`、`hctl2-tool`、用户级 metadata 账本 | 稳定身份、绑定、授权、判决、命令准入、外部副作用意图与恢复。 |
| 执行面 | chat server、task backend、workflow engine、Agency/Harness | 场景 content、机械状态与物理执行；接受控制面按先记账后执行顺序发出的副作用。 |

## 三类数据

| 数据 | 当前权威所在 | 内容 |
| --- | --- | --- |
| metadata（治理元数据） | HCTL 用户级账本 | 身份、绑定、授权、租约、代次、判决与 Receipt。 |
| content（场景内容） | 各场景专职系统 | 消息、任务卡与流转、机械执行历史、会话转录。 |
| artifact（结晶） | Git | 从 content 提炼出的不可变决议、Memo、冻结契约与施工图、凭证链、代码变更。 |

## 四个第一阶段默认实现

| 场景 | 默认实现 | 稳定替换边界 |
| --- | --- | --- |
| Chat Room | Tuwunel（Matrix homeserver；Cinny 是随包客户端） | Matrix 协议与 Chat 端口绑定。 |
| Kanban | Vikunja | task backend 端口、Task Binding 与字段权威合同。 |
| Workflow | Dagu | Workflow Revision 中间表示、编译与引擎回读端口。 |
| Terminal | Herdr | Agency 端口与客户端侧终端 transport adapter。 |

## 三条执行底线

1. **工具不是人。** Harness、模型和运行时钩子只能提交 Result Proposal；治理命令来自可归属到 owner human 的动作，Task 另允许绑定 Run 正常完成后的确定性归约命令。
2. **合入钥匙不进 Harness。** 集成与外部写凭据只由当前工具箱或 adapter 网关代用；合入必须经过持久意图、执行、回读和 Integration Receipt。
3. **隔离工作树。** 每个 ChangeSet 使用独立 worktree 与单一有效 Write Lease；旧写入者无法证明失权时保全旧现场并换新 ChangeSet/worktree。

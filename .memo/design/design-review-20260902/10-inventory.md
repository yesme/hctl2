# 机械清点（第一轮 1.2）

> 状态：已生成<br>
> 基线：main @ `6850f18`（草案 v0.16.0）<br>
> 生成：`cd src && ./buck2 build root//build/docs:inventory --show-simple-output`，脚本在 `src/build/docs/inventory_*.pl`，只提取与计数，不判断。四份产出原样并入，判断见 `11-findings-draft.md`。

---

## 概念清点（脚本产出，不含判断）

概念宇宙：词汇表中带「中文对照」列的全部表格，共 65 条。层按路径划分：vision / arch / spec / delivery / reference / guide。计数规则：英文名按词边界匹配，中文名按子串匹配，`英文（中文）` 形式的对照不计入中文次数。

### 一、概念 × 层：英文次数 / 中文次数

| 概念（英） | 中文 | vision | arch | spec | delivery | reference | guide |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `Agent` | 编码代理 | — | — | 2 / 1 | — | 32 / 4 | — |
| `Harness` | 编码代理工具 | 16 / 2 | 18 / 0 | 37 / 1 | 11 / 0 | 38 / 2 | — |
| `Agency` | 派出方 | 1 / 0 | 30 / 1 | 58 / 2 | 19 / 0 | 25 / 2 | — |
| `worker` | 执行体 | — | 2 / 43 | 2 / 24 | 1 / 0 | 3 / 22 | 0 / 1 |
| `Repo` | 仓库 | 6 / 6 | 14 / 17 | 54 / 16 | 11 / 8 | 19 / 18 | 0 / 3 |
| `Project` | 项目 | 23 / 11 | 69 / 4 | 148 / 1 | 29 / 0 | 31 / 1 | 1 / 0 |
| `Room` | 协作聊天室 | 15 / 0 | 53 / 0 | 107 / 0 | 27 / 0 | 35 / 1 | — |
| `Chat Room` | 聊天室场景 | 1 / 0 | 11 / 0 | 7 / 0 | 3 / 0 | 6 / 1 | — |
| `Participant` | 参与者 | 4 / 3 | 40 / 23 | 74 / 6 | 7 / 2 | 19 / 9 | — |
| `Request` | 请求卡 | 4 / 0 | 11 / 0 | 47 / 0 | 9 / 0 | 3 / 1 | — |
| `Memo` | 备忘 | 1 / 1 | 17 / 1 | 16 / 0 | 4 / 1 | 4 / 1 | — |
| `Artifact` | 工件 | 3 / 0 | 11 / 2 | 28 / 0 | 1 / 1 | 6 / 1 | — |
| `Context` | 上下文 | 2 / 8 | 11 / 27 | 34 / 2 | 11 / 1 | 15 / 8 | 0 / 3 |
| `Skill` | 技能包 | 1 / 0 | 20 / 3 | 30 / 0 | 5 / 1 | 9 / 1 | — |
| `Task` | 任务承诺 | 17 / 0 | 56 / 0 | 250 / 0 | 36 / 0 | 52 / 2 | 1 / 0 |
| `Kanban` | 看板 | 2 / 1 | 10 / 5 | 13 / 8 | 8 / 3 | 7 / 4 | — |
| `Run` | 一次受治理施工 | 22 / 0 | 82 / 0 | 183 / 0 | 37 / 0 | 40 / 1 | 1 / 0 |
| `Workflow` | 施工图 | 6 / 4 | 15 / 17 | 31 / 6 | 9 / 1 | 15 / 12 | — |
| `Obligation` | 交付义务 | 2 / 1 | 3 / 6 | 31 / 0 | 6 / 0 | 13 / 2 | — |
| `Seat` | 执行席位 | 2 / 0 | 3 / 0 | 48 / 0 | 7 / 0 | 9 / 1 | — |
| `Attempt` | 执行尝试 | 3 / 0 | 1 / 0 | 50 / 0 | 4 / 0 | 10 / 1 | — |
| `Gate` | 评审关卡 | — | 7 / 3 | 21 / 0 | 5 / 0 | 8 / 2 | — |
| `Verdict` | 裁决 | 3 / 3 | 4 / 23 | 26 / 4 | 1 / 2 | 6 / 21 | — |
| `Receipt` | 凭证 | 7 / 0 | 2 / 18 | 60 / 7 | 17 / 1 | 21 / 3 | 0 / 1 |
| `Terminal` | 终端场景 | 1 / 0 | 16 / 0 | 17 / 0 | 6 / 0 | 17 / 1 | — |
| `ChangeSet` | 变更集 | — | 3 / 7 | 50 / 0 | 10 / 0 | 13 / 2 | 0 / 1 |
| `Evidence` | 证据 | 0 / 10 | 0 / 19 | 7 / 38 | 0 / 25 | 2 / 27 | 0 / 2 |
| `Workbench` | 工作台 | 3 / 0 | 31 / 0 | 16 / 0 | 30 / 0 | 21 / 2 | — |
| `hctl2-tool` | 工具箱 | — | 0 / 5 | 2 / 37 | 10 / 9 | 5 / 7 | 1 / 2 |
| `owner` | 归属者 | — | — | 5 / 50 | 0 / 2 | 8 / 7 | 1 / 0 |
| `fence` | 代次栅栏 | — | — | 0 / 18 | 0 / 3 | 7 / 1 | 2 / 0 |
| `worktree` | Git 工作树 | 3 / 1 | 6 / 1 | 1 / 17 | 1 / 5 | 9 / 1 | — |
| `ID` | 标识符 | — | 5 / 0 | 30 / 4 | 8 / 1 | 9 / 3 | 0 / 1 |
| `claim` | 认领 | — | — | 0 / 7 | — | 5 / 1 | — |
| `CAS` | 比较并交换 | — | 1 / 0 | 8 / 14 | — | 4 / 1 | — |
| `fresh readback` | 当前回读 | — | — | 0 / 8 | 0 / 5 | 4 / 1 | 1 / 0 |
| `ACK` | 确认回执 | — | — | 0 / 13 | 0 / 4 | 2 / 1 | — |
| `metadata` | 治理元数据 | 2 / 0 | 5 / 1 | 21 / 1 | 3 / 0 | 7 / 2 | — |
| `content` | 场景内容 | 8 / 0 | 25 / 6 | 60 / 2 | 17 / 0 | 28 / 3 | — |
| `artifact` | 结晶 | 1 / 0 | 8 / 24 | 6 / 16 | 1 / 2 | 5 / 12 | 0 / 2 |
| `Task Revision` | 任务契约版本 | 1 / 1 | 3 / 10 | 41 / 0 | 5 / 0 | 6 / 1 | — |
| `Workflow Revision` | 施工图版本 | — | 2 / 0 | 16 / 0 | 3 / 0 | 5 / 1 | — |
| `ChangeSet Revision` | 变更集快照 | — | 0 / 5 | 17 / 0 | 4 / 0 | 2 / 1 | — |
| `Artifact Revision` | 工件版本 | — | — | 7 / 0 | — | 1 / 1 | — |
| `Extension Revision` | 扩展版本 | — | — | 4 / 0 | — | 1 / 1 | — |
| `Engine Deployment` | 引擎部署版本 | — | — | 8 / 0 | 1 / 0 | 2 / 1 | — |
| `Resolved Port Binding` | 端口解析绑定 | — | — | 11 / 0 | 1 / 0 | 3 / 1 | — |
| `Chat 端口绑定` | 聊天端口绑定 | — | 1 / 0 | 6 / 0 | 1 / 0 | 5 / 1 | — |
| `Task Binding` | 任务来源绑定 | — | — | 16 / 0 | 2 / 0 | 3 / 1 | — |
| `Project Role Binding` | 角色绑定 | — | 0 / 5 | 9 / 3 | — | 1 / 1 | — |
| `Engine Execution Binding` | 引擎执行绑定 | — | — | 14 / 0 | 1 / 0 | 3 / 1 | — |
| `Write Lease` | 写入租约 | — | — | 7 / 1 | — | 3 / 1 | — |
| `Terminal Input Lease` | 终端输入租约 | — | — | 8 / 0 | — | 1 / 1 | — |
| `Agency binding owner lease` | 派出方绑定的归属者租约 | — | — | — | — | 1 / 1 | — |
| `Task Source Snapshot` | 来源快照 | — | 0 / 1 | 7 / 1 | — | 1 / 1 | — |
| `Result Proposal` | 结果提案 | — | 2 / 5 | 26 / 1 | 3 / 0 | 5 / 2 | — |
| `Execution Spec` | 派发规格 | — | 3 / 2 | 49 / 0 | 5 / 0 | 7 / 2 | — |
| `Run Manifest` | 施工清单 | — | 1 / 5 | 14 / 0 | — | 2 / 2 | — |
| `Attach Descriptor` | 连接票据 | — | 0 / 1 | 6 / 4 | 0 / 2 | 1 / 2 | — |
| `Context Manifest` | 根上下文清单 | 0 / 1 | 0 / 4 | 11 / 1 | 1 / 0 | 1 / 2 | — |
| `Context Bundle` | 消费上下文包 | — | 0 / 4 | 11 / 1 | 1 / 0 | 1 / 1 | — |
| `Repo Instance` | 仓库实例 | — | — | 12 / 1 | 3 / 0 | 7 / 1 | — |
| `Room Invocation` | 单次调用 | 1 / 1 | 3 / 2 | 34 / 0 | 3 / 0 | 5 / 1 | — |
| `Execution Runtime` | 执行运行时 | — | — | 15 / 0 | 2 / 0 | 5 / 1 | — |
| `Worker Profile` | 执行者配置 | — | 0 / 5 | 14 / 0 | 2 / 0 | 3 / 2 | — |

### 二、同一层里中英两写的概念（两种写法都出现，且不是对照形式）

| 概念 | 层 | 英 / 中 | 涉及文件 |
| --- | --- | --- | --- |
| `ACK` | reference | 2 / 1 | docs/design/references/glossary.md |
| `Agency` | arch | 30 / 1 | docs/design/participant.md |
| `Agency` | spec | 58 / 2 | docs/design/spec/README.md、docs/design/spec/participant.md |
| `Agency` | reference | 25 / 2 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Agency binding owner lease` | reference | 1 / 1 | docs/design/references/glossary.md |
| `Agent` | spec | 2 / 1 | docs/design/spec/README.md |
| `Agent` | reference | 32 / 4 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Artifact` | arch | 11 / 2 | docs/design/README.md |
| `Artifact` | delivery | 1 / 1 | （分散在不同文件） |
| `Artifact` | reference | 6 / 1 | docs/design/references/glossary.md |
| `Artifact Revision` | reference | 1 / 1 | docs/design/references/glossary.md |
| `Attach Descriptor` | spec | 6 / 4 | docs/design/spec/participant.md、docs/design/spec/system.md |
| `Attach Descriptor` | reference | 1 / 2 | docs/design/references/glossary.md |
| `Attempt` | reference | 10 / 1 | docs/design/references/glossary.md |
| `CAS` | spec | 8 / 14 | docs/design/spec/connections.md、docs/design/spec/system.md、docs/design/spec/task.md |
| `CAS` | reference | 4 / 1 | docs/design/references/glossary.md |
| `ChangeSet` | arch | 3 / 7 | docs/design/participant.md |
| `ChangeSet` | reference | 13 / 2 | docs/design/references/glossary.md |
| `ChangeSet Revision` | reference | 2 / 1 | docs/design/references/glossary.md |
| `Chat Room` | reference | 6 / 1 | docs/design/references/glossary.md |
| `Chat 端口绑定` | reference | 5 / 1 | docs/design/references/glossary.md |
| `Context` | vision | 2 / 8 | docs/design/vision.md |
| `Context` | arch | 11 / 27 | docs/design/README.md、docs/design/context.md、docs/design/participant.md、docs/design/project.md |
| `Context` | spec | 34 / 2 | docs/design/spec/participant.md、docs/design/spec/project.md |
| `Context` | delivery | 11 / 1 | （分散在不同文件） |
| `Context` | reference | 15 / 8 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Context Bundle` | spec | 11 / 1 | docs/design/spec/project.md |
| `Context Bundle` | reference | 1 / 1 | docs/design/references/glossary.md |
| `Context Manifest` | spec | 11 / 1 | docs/design/spec/project.md |
| `Context Manifest` | reference | 1 / 2 | docs/design/references/glossary.md |
| `Engine Deployment` | reference | 2 / 1 | docs/design/references/glossary.md |
| `Engine Execution Binding` | reference | 3 / 1 | docs/design/references/glossary.md |
| `Evidence` | spec | 7 / 38 | docs/design/spec/connections.md、docs/design/spec/participant.md、docs/design/spec/task.md |
| `Evidence` | reference | 2 / 27 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Execution Runtime` | reference | 5 / 1 | docs/design/references/glossary.md |
| `Execution Spec` | arch | 3 / 2 | docs/design/architecture.md |
| `Execution Spec` | reference | 7 / 2 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Extension Revision` | reference | 1 / 1 | docs/design/references/glossary.md |
| `Gate` | arch | 7 / 3 | docs/design/run.md |
| `Gate` | reference | 8 / 2 | docs/design/references/glossary.md |
| `Harness` | vision | 16 / 2 | README.md、docs/design/vision.md |
| `Harness` | spec | 37 / 1 | （分散在不同文件） |
| `Harness` | reference | 38 / 2 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `ID` | spec | 30 / 4 | docs/design/spec/connections.md |
| `ID` | delivery | 8 / 1 | docs/design/contract-tests.md |
| `ID` | reference | 9 / 3 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Kanban` | vision | 2 / 1 | （分散在不同文件） |
| `Kanban` | arch | 10 / 5 | docs/design/architecture.md、docs/design/task.md |
| `Kanban` | spec | 13 / 8 | docs/design/spec/connections.md、docs/design/spec/system.md、docs/design/spec/task.md |
| `Kanban` | delivery | 8 / 3 | docs/design/contract-tests.md、docs/design/delivery.md |
| `Kanban` | reference | 7 / 4 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Memo` | vision | 1 / 1 | docs/design/vision.md |
| `Memo` | arch | 17 / 1 | （分散在不同文件） |
| `Memo` | delivery | 4 / 1 | （分散在不同文件） |
| `Memo` | reference | 4 / 1 | docs/design/references/glossary.md |
| `Obligation` | vision | 2 / 1 | docs/design/vision.md |
| `Obligation` | arch | 3 / 6 | docs/design/run.md |
| `Obligation` | reference | 13 / 2 | docs/design/references/glossary.md |
| `Participant` | vision | 4 / 3 | README.md、docs/design/vision.md |
| `Participant` | arch | 40 / 23 | docs/design/README.md、docs/design/participant.md |
| `Participant` | spec | 74 / 6 | docs/design/spec/README.md、docs/design/spec/participant.md |
| `Participant` | delivery | 7 / 2 | docs/design/contract-tests.md |
| `Participant` | reference | 19 / 9 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Project` | vision | 23 / 11 | README.md、docs/design/vision.md |
| `Project` | arch | 69 / 4 | docs/design/participant.md、docs/design/run.md |
| `Project` | spec | 148 / 1 | docs/design/spec/task.md |
| `Project` | reference | 31 / 1 | docs/design/references/glossary.md |
| `Project Role Binding` | spec | 9 / 3 | docs/design/spec/participant.md、docs/design/spec/project.md |
| `Project Role Binding` | reference | 1 / 1 | docs/design/references/glossary.md |
| `Receipt` | arch | 2 / 18 | docs/design/participant.md、docs/design/run.md |
| `Receipt` | spec | 60 / 7 | docs/design/spec/README.md、docs/design/spec/run.md、docs/design/spec/system.md、docs/design/spec/task.md |
| `Receipt` | delivery | 17 / 1 | （分散在不同文件） |
| `Receipt` | reference | 21 / 3 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Repo` | vision | 6 / 6 | docs/design/vision.md |
| `Repo` | arch | 14 / 17 | docs/design/architecture.md、docs/design/context.md、docs/design/project.md、docs/design/task.md |
| `Repo` | spec | 54 / 16 | docs/design/spec/README.md、docs/design/spec/participant.md、docs/design/spec/project.md、docs/design/spec/system.md、docs/design/spec/task.md |
| `Repo` | delivery | 11 / 8 | docs/design/contract-tests.md、docs/design/delivery.md |
| `Repo` | reference | 19 / 18 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Repo Instance` | spec | 12 / 1 | docs/design/spec/system.md |
| `Repo Instance` | reference | 7 / 1 | docs/design/references/glossary.md |
| `Request` | reference | 3 / 1 | docs/design/references/glossary.md |
| `Resolved Port Binding` | reference | 3 / 1 | docs/design/references/glossary.md |
| `Result Proposal` | arch | 2 / 5 | docs/design/architecture.md、docs/design/participant.md |
| `Result Proposal` | spec | 26 / 1 | docs/design/spec/system.md |
| `Result Proposal` | reference | 5 / 2 | docs/design/references/glossary.md |
| `Room` | reference | 35 / 1 | docs/design/references/glossary.md |
| `Room Invocation` | vision | 1 / 1 | docs/design/vision.md |
| `Room Invocation` | arch | 3 / 2 | docs/design/project.md |
| `Room Invocation` | reference | 5 / 1 | docs/design/references/glossary.md |
| `Run` | reference | 40 / 1 | docs/design/references/glossary.md |
| `Run Manifest` | arch | 1 / 5 | docs/design/architecture.md |
| `Run Manifest` | reference | 2 / 2 | docs/design/references/glossary.md |
| `Seat` | reference | 9 / 1 | docs/design/references/glossary.md |
| `Skill` | arch | 20 / 3 | docs/design/participant.md |
| `Skill` | delivery | 5 / 1 | （分散在不同文件） |
| `Skill` | reference | 9 / 1 | docs/design/references/glossary.md |
| `Task` | reference | 52 / 2 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Task Binding` | reference | 3 / 1 | docs/design/references/glossary.md |
| `Task Revision` | vision | 1 / 1 | docs/design/vision.md |
| `Task Revision` | arch | 3 / 10 | docs/design/architecture.md、docs/design/task.md |
| `Task Revision` | reference | 6 / 1 | docs/design/references/glossary.md |
| `Task Source Snapshot` | spec | 7 / 1 | docs/design/spec/task.md |
| `Task Source Snapshot` | reference | 1 / 1 | docs/design/references/glossary.md |
| `Terminal` | reference | 17 / 1 | docs/design/references/glossary.md |
| `Terminal Input Lease` | reference | 1 / 1 | docs/design/references/glossary.md |
| `Verdict` | vision | 3 / 3 | docs/design/vision.md |
| `Verdict` | arch | 4 / 23 | docs/design/context.md、docs/design/participant.md、docs/design/run.md |
| `Verdict` | spec | 26 / 4 | docs/design/spec/README.md、docs/design/spec/participant.md、docs/design/spec/run.md |
| `Verdict` | delivery | 1 / 2 | （分散在不同文件） |
| `Verdict` | reference | 6 / 21 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Workbench` | reference | 21 / 2 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Worker Profile` | reference | 3 / 2 | docs/design/references/glossary.md |
| `Workflow` | vision | 6 / 4 | docs/design/vision.md |
| `Workflow` | arch | 15 / 17 | docs/design/README.md、docs/design/architecture.md、docs/design/project.md、docs/design/run.md、docs/design/task.md |
| `Workflow` | spec | 31 / 6 | docs/design/spec/README.md、docs/design/spec/connections.md、docs/design/spec/run.md、docs/design/spec/task.md |
| `Workflow` | delivery | 9 / 1 | （分散在不同文件） |
| `Workflow` | reference | 15 / 12 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `Workflow Revision` | reference | 5 / 1 | docs/design/references/glossary.md |
| `Write Lease` | spec | 7 / 1 | docs/design/spec/participant.md |
| `Write Lease` | reference | 3 / 1 | docs/design/references/glossary.md |
| `artifact` | arch | 8 / 24 | docs/design/architecture.md、docs/design/context.md |
| `artifact` | spec | 6 / 16 | docs/design/spec/README.md、docs/design/spec/project.md、docs/design/spec/run.md、docs/design/spec/task.md |
| `artifact` | delivery | 1 / 2 | （分散在不同文件） |
| `artifact` | reference | 5 / 12 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `claim` | reference | 5 / 1 | docs/design/references/glossary.md |
| `content` | arch | 25 / 6 | docs/design/architecture.md、docs/design/context.md、docs/design/project.md、docs/design/task.md |
| `content` | spec | 60 / 2 | docs/design/spec/README.md |
| `content` | reference | 28 / 3 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `fence` | reference | 7 / 1 | docs/design/references/glossary.md |
| `fresh readback` | reference | 4 / 1 | docs/design/references/glossary.md |
| `hctl2-tool` | spec | 2 / 37 | docs/design/spec/system.md |
| `hctl2-tool` | delivery | 10 / 9 | docs/design/delivery.md |
| `hctl2-tool` | reference | 5 / 7 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `hctl2-tool` | guide | 1 / 2 | WRITING-GUIDE.md |
| `metadata` | arch | 5 / 1 | docs/design/architecture.md |
| `metadata` | spec | 21 / 1 | docs/design/spec/README.md |
| `metadata` | reference | 7 / 2 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `owner` | spec | 5 / 50 | docs/design/spec/connections.md、docs/design/spec/system.md |
| `owner` | reference | 8 / 7 | docs/design/references/glossary.md |
| `worker` | arch | 2 / 43 | docs/design/architecture.md、docs/design/participant.md |
| `worker` | spec | 2 / 24 | docs/design/spec/README.md、docs/design/spec/project.md |
| `worker` | reference | 3 / 22 | docs/design/references/decision-history.md、docs/design/references/glossary.md |
| `worktree` | vision | 3 / 1 | docs/design/vision.md |
| `worktree` | arch | 6 / 1 | docs/design/project.md |
| `worktree` | spec | 1 / 17 | （分散在不同文件） |
| `worktree` | delivery | 1 / 5 | docs/design/contract-tests.md |
| `worktree` | reference | 9 / 1 | docs/design/references/glossary.md |

### 三、英文名首现处未带中文对照（按文件）

| 文件 | 概念 |
| --- | --- |
| README.md | `Agency`、`Chat Room`、`Context`、`Kanban`、`Participant`、`Project`、`Room`、`Run`、`Task`、`Terminal`、`Workbench`、`Workflow`、`content` |
| WRITING-GUIDE.md | `Project`、`Run`、`Task`、`fence`、`fresh readback`、`hctl2-tool`、`owner` |
| docs/design/README.md | `Artifact`、`CAS`、`Chat Room`、`Context`、`Harness`、`ID`、`Kanban`、`Participant`、`Project`、`Repo`、`Request`、`Room`、`Run`、`Skill`、`Task`、`Terminal`、`Workbench`、`Workflow`、`content` |
| docs/design/architecture.md | `Agency`、`ChangeSet`、`Chat Room`、`Chat 端口绑定`、`Execution Spec`、`Gate`、`Kanban`、`Memo`、`Obligation`、`Participant`、`Project`、`Repo`、`Result Proposal`、`Room`、`Run`、`Run Manifest`、`Task`、`Task Revision`、`Terminal`、`Workbench`、`Workflow`、`Workflow Revision` |
| docs/design/context.md | `Artifact`、`Context`、`Memo`、`Participant`、`Project`、`Repo`、`Room`、`Run`、`Skill`、`Task`、`Task Revision`、`Terminal`、`Verdict`、`artifact`、`content`、`worktree` |
| docs/design/contract-tests.md | `Agency`、`Attempt`、`ChangeSet`、`Chat Room`、`Chat 端口绑定`、`Context`、`Context Bundle`、`Context Manifest`、`Engine Execution Binding`、`Execution Runtime`、`Execution Spec`、`Gate`、`Harness`、`ID`、`Kanban`、`Memo`、`Obligation`、`Participant`、`Project`、`Receipt`、`Repo`、`Repo Instance`、`Request`、`Resolved Port Binding`、`Result Proposal`、`Room`、`Run`、`Seat`、`Skill`、`Task`、`Task Binding`、`Task Revision`、`Terminal`、`Verdict`、`Workbench`、`Worker Profile`、`Workflow`、`Workflow Revision`、`content`、`metadata`、`worktree` |
| docs/design/delivery.md | `Agency`、`Artifact`、`Attempt`、`ChangeSet`、`ChangeSet Revision`、`Chat Room`、`Context`、`Engine Deployment`、`Execution Spec`、`Gate`、`Harness`、`ID`、`Kanban`、`Memo`、`Obligation`、`Participant`、`Project`、`Receipt`、`Repo`、`Repo Instance`、`Request`、`Room`、`Room Invocation`、`Run`、`Seat`、`Task`、`Task Revision`、`Terminal`、`Workbench`、`Workflow`、`Workflow Revision`、`artifact`、`content`、`hctl2-tool`、`metadata`、`worker` |
| docs/design/doc-discipline.md | `Obligation`、`Run`、`Seat`、`Task`、`Workbench` |
| docs/design/participant.md | `Agency`、`Context`、`Execution Spec`、`Gate`、`Harness`、`ID`、`Memo`、`Participant`、`Project`、`Receipt`、`Result Proposal`、`Room`、`Run`、`Seat`、`Task`、`Terminal`、`Verdict`、`Workbench`、`worker` |
| docs/design/project.md | `Chat Room`、`Context`、`Harness`、`ID`、`Kanban`、`Project`、`Repo`、`Room`、`Room Invocation`、`Run`、`Skill`、`Task`、`Terminal`、`Workbench`、`Workflow`、`content` |
| docs/design/references/decision-history.md | `Agency`、`Agent`、`Artifact`、`Attempt`、`CAS`、`ChangeSet`、`ChangeSet Revision`、`Chat Room`、`Chat 端口绑定`、`Context`、`Engine Deployment`、`Engine Execution Binding`、`Evidence`、`Execution Runtime`、`Execution Spec`、`Gate`、`Harness`、`ID`、`Kanban`、`Memo`、`Obligation`、`Participant`、`Project`、`Receipt`、`Repo`、`Repo Instance`、`Request`、`Resolved Port Binding`、`Result Proposal`、`Room`、`Room Invocation`、`Run`、`Run Manifest`、`Seat`、`Skill`、`Task`、`Task Binding`、`Task Revision`、`Terminal`、`Verdict`、`Workbench`、`Worker Profile`、`Workflow`、`Workflow Revision`、`Write Lease`、`artifact`、`claim`、`content`、`fence`、`fresh readback`、`hctl2-tool`、`metadata`、`owner`、`worker`、`worktree` |
| docs/design/references/glossary.md | `ACK`、`Agency`、`Agency binding owner lease`、`Agent`、`Artifact`、`Artifact Revision`、`Attach Descriptor`、`Attempt`、`CAS`、`ChangeSet`、`ChangeSet Revision`、`Chat Room`、`Chat 端口绑定`、`Context`、`Context Bundle`、`Context Manifest`、`Engine Deployment`、`Engine Execution Binding`、`Evidence`、`Execution Runtime`、`Execution Spec`、`Extension Revision`、`Gate`、`Harness`、`ID`、`Kanban`、`Memo`、`Obligation`、`Participant`、`Project`、`Project Role Binding`、`Receipt`、`Repo`、`Repo Instance`、`Request`、`Resolved Port Binding`、`Result Proposal`、`Room`、`Room Invocation`、`Run`、`Run Manifest`、`Seat`、`Skill`、`Task`、`Task Binding`、`Task Revision`、`Task Source Snapshot`、`Terminal`、`Terminal Input Lease`、`Verdict`、`Workbench`、`Worker Profile`、`Workflow`、`Workflow Revision`、`Write Lease`、`artifact`、`claim`、`content`、`fence`、`fresh readback`、`hctl2-tool`、`metadata`、`owner`、`worker`、`worktree` |
| docs/design/run.md | `Chat Room`、`Execution Spec`、`Harness`、`Obligation`、`Participant`、`Project`、`Request`、`Room`、`Run`、`Seat`、`Task`、`Verdict`、`Workbench`、`Workflow`、`Workflow Revision` |
| docs/design/spec/README.md | `Agent`、`Artifact`、`Artifact Revision`、`Attach Descriptor`、`Attempt`、`ChangeSet`、`ChangeSet Revision`、`Chat Room`、`Context`、`Context Bundle`、`Context Manifest`、`Engine Deployment`、`Engine Execution Binding`、`Evidence`、`Execution Runtime`、`Execution Spec`、`Extension Revision`、`Gate`、`Kanban`、`Memo`、`Obligation`、`Participant`、`Project`、`Project Role Binding`、`Receipt`、`Repo`、`Repo Instance`、`Request`、`Resolved Port Binding`、`Room`、`Run`、`Seat`、`Skill`、`Task`、`Task Binding`、`Task Source Snapshot`、`Terminal`、`Terminal Input Lease`、`Verdict`、`Workbench`、`Worker Profile`、`Workflow`、`Write Lease`、`artifact`、`content`、`metadata` |
| docs/design/spec/connections.md | `Agency`、`Artifact`、`Artifact Revision`、`Attempt`、`CAS`、`ChangeSet`、`ChangeSet Revision`、`Chat Room`、`Context`、`Context Bundle`、`Context Manifest`、`Engine Execution Binding`、`Evidence`、`Execution Runtime`、`Execution Spec`、`Gate`、`Harness`、`ID`、`Kanban`、`Memo`、`Obligation`、`Participant`、`Project`、`Project Role Binding`、`Receipt`、`Repo`、`Repo Instance`、`Request`、`Resolved Port Binding`、`Result Proposal`、`Room`、`Room Invocation`、`Run`、`Run Manifest`、`Seat`、`Skill`、`Task`、`Task Revision`、`Terminal`、`Verdict`、`Workbench`、`Worker Profile`、`Workflow`、`Write Lease`、`content`、`metadata`、`owner` |
| docs/design/spec/participant.md | `Agency`、`Agent`、`Attach Descriptor`、`Attempt`、`ChangeSet`、`ChangeSet Revision`、`Context`、`Context Bundle`、`Evidence`、`Execution Runtime`、`Execution Spec`、`Harness`、`ID`、`Participant`、`Project`、`Project Role Binding`、`Receipt`、`Repo`、`Repo Instance`、`Result Proposal`、`Room`、`Room Invocation`、`Run`、`Run Manifest`、`Seat`、`Skill`、`Task`、`Terminal`、`Terminal Input Lease`、`Verdict`、`Workbench`、`Worker Profile`、`Write Lease`、`content` |
| docs/design/spec/project.md | `Agency`、`Artifact`、`Artifact Revision`、`Attempt`、`ChangeSet`、`ChangeSet Revision`、`Chat Room`、`Chat 端口绑定`、`Context`、`Context Bundle`、`Context Manifest`、`Execution Spec`、`Gate`、`Harness`、`ID`、`Memo`、`Participant`、`Project`、`Project Role Binding`、`Receipt`、`Repo`、`Repo Instance`、`Request`、`Resolved Port Binding`、`Result Proposal`、`Room`、`Room Invocation`、`Run`、`Seat`、`Skill`、`Task`、`Task Source Snapshot`、`Verdict`、`Workbench`、`Worker Profile`、`artifact`、`content`、`metadata`、`worker`、`worktree` |
| docs/design/spec/run.md | `Agency`、`Artifact`、`Artifact Revision`、`Attempt`、`ChangeSet`、`ChangeSet Revision`、`Context`、`Context Bundle`、`Context Manifest`、`Engine Deployment`、`Engine Execution Binding`、`Execution Spec`、`Gate`、`Harness`、`ID`、`Obligation`、`Participant`、`Project`、`Project Role Binding`、`Receipt`、`Request`、`Resolved Port Binding`、`Result Proposal`、`Room`、`Room Invocation`、`Run`、`Run Manifest`、`Seat`、`Skill`、`Task`、`Task Revision`、`Verdict`、`Workbench`、`Worker Profile`、`Workflow`、`Workflow Revision`、`artifact`、`metadata` |
| docs/design/spec/system.md | `Agency`、`Artifact`、`Attach Descriptor`、`CAS`、`ChangeSet`、`ChangeSet Revision`、`Chat Room`、`Context`、`Engine Execution Binding`、`Execution Runtime`、`Execution Spec`、`Extension Revision`、`Harness`、`ID`、`Kanban`、`Memo`、`Participant`、`Project`、`Receipt`、`Repo`、`Repo Instance`、`Request`、`Resolved Port Binding`、`Result Proposal`、`Room`、`Run`、`Run Manifest`、`Skill`、`Task`、`Task Revision`、`Terminal`、`Terminal Input Lease`、`Verdict`、`Workbench`、`Workflow`、`Workflow Revision`、`content`、`hctl2-tool`、`metadata`、`owner` |
| docs/design/spec/task.md | `CAS`、`Chat Room`、`Context`、`Context Manifest`、`Evidence`、`Execution Spec`、`Harness`、`ID`、`Kanban`、`Project`、`Receipt`、`Repo`、`Request`、`Resolved Port Binding`、`Room`、`Run`、`Run Manifest`、`Task`、`Task Binding`、`Task Revision`、`Task Source Snapshot`、`Verdict`、`Workbench`、`Workflow`、`Workflow Revision`、`artifact`、`content` |
| docs/design/task.md | `Artifact`、`Kanban`、`Participant`、`Project`、`Repo`、`Run`、`Task`、`Task Revision`、`Workbench`、`Workflow`、`content` |
| docs/design/vision.md | `Artifact`、`Attempt`、`Context`、`Project`、`Repo`、`Room`、`Run`、`Skill`、`Task`、`Workbench`、`content` |
| docs/usage.md | `Agency`、`ID`、`Kanban`、`Participant`、`Room`、`Terminal`、`Workbench`、`Workflow`、`hctl2-tool` |

### 四、驼峰拼接词（正文，代码块已排除）

| 词 | 总次数 | 文件 |
| --- | --- | --- |
| `ChangeSet` | 76 | docs/design/architecture.md (1)、docs/design/contract-tests.md (3)、docs/design/delivery.md (7)、docs/design/participant.md (2)、docs/design/references/decision-history.md (9)、docs/design/references/glossary.md (4)、docs/design/spec/README.md (3)、docs/design/spec/connections.md (5)、docs/design/spec/participant.md (23)、docs/design/spec/project.md (3)、docs/design/spec/run.md (8)、docs/design/spec/system.md (8) |
| `GitHub` | 22 | WRITING-GUIDE.md (2)、docs/design/README.md (1)、docs/design/architecture.md (2)、docs/design/delivery.md (4)、docs/design/references/decision-history.md (5)、docs/design/spec/README.md (1)、docs/design/spec/system.md (1)、docs/design/spec/task.md (4)、docs/design/task.md (2) |
| `ReviewSubjectRef` | 14 | docs/design/delivery.md (2)、docs/design/references/decision-history.md (2)、docs/design/references/glossary.md (1)、docs/design/spec/README.md (2)、docs/design/spec/connections.md (1)、docs/design/spec/project.md (1)、docs/design/spec/run.md (5) |
| `AppService` | 6 | docs/design/delivery.md (1)、docs/design/project.md (1)、docs/design/references/decision-history.md (3)、docs/design/spec/project.md (1) |
| `OpenCode` | 4 | README.md (1)、docs/design/delivery.md (1)、docs/design/references/glossary.md (1)、docs/design/vision.md (1) |
| `HarnessAdapter` | 2 | docs/design/references/decision-history.md (2) |
| `RuntimeBackend` | 2 | docs/design/references/decision-history.md (2) |
| `TaskSource` | 2 | docs/design/references/decision-history.md (2) |
| `WebView` | 2 | docs/design/spec/system.md (2) |
| `WezTerm` | 2 | docs/design/delivery.md (1)、docs/design/references/decision-history.md (1) |
| `WorkflowEngine` | 2 | docs/design/references/decision-history.md (2) |
| `AttemptSpec` | 1 | docs/design/references/decision-history.md (1) |
| `BindingRevision` | 1 | docs/design/references/decision-history.md (1) |
| `ChangeSetWriteLease` | 1 | docs/design/references/decision-history.md (1) |
| `ChatSurfaceBindingRevision` | 1 | docs/design/references/decision-history.md (1) |
| `DeepSeek` | 1 | docs/design/references/decision-history.md (1) |
| `EngineDeploymentRevision` | 1 | docs/design/references/decision-history.md (1) |
| `ExternalEffectIntent` | 1 | docs/design/references/decision-history.md (1) |
| `HarnessAdapterBinding` | 1 | docs/design/references/decision-history.md (1) |
| `HarnessDefinition` | 1 | docs/design/references/decision-history.md (1) |
| `IntegrationIntent` | 1 | docs/design/references/decision-history.md (1) |
| `InvocationBinding` | 1 | docs/design/references/decision-history.md (1) |
| `InvocationRuntime` | 1 | docs/design/references/decision-history.md (1) |
| `LobeHub` | 1 | docs/design/references/decision-history.md (1) |
| `OpenClaw` | 1 | docs/design/references/decision-history.md (1) |
| `RuntimeShard` | 1 | docs/design/references/decision-history.md (1) |
| `SemVer` | 1 | WRITING-GUIDE.md (1) |
| `TaskRevision` | 1 | docs/design/references/decision-history.md (1) |
| `TaskSourceBindingRevision` | 1 | docs/design/references/decision-history.md (1) |
| `TaskSourceConnection` | 1 | docs/design/references/decision-history.md (1) |
| `TaskSourceConnectionRevision` | 1 | docs/design/references/decision-history.md (1) |
| `TerminalBundle` | 1 | docs/design/references/decision-history.md (1) |
| `TerminalGateway` | 1 | docs/design/references/decision-history.md (1) |
| `TypeScript` | 1 | docs/design/references/decision-history.md (1) |
| `WorkflowEngineAdapter` | 1 | docs/design/references/decision-history.md (1) |
| `ZeroClaw` | 1 | docs/design/references/decision-history.md (1) |

---

## 边界清点（脚本产出，不含判断）

来源：`docs/design/spec/connections.md` §连接约束总表。四格按原表列名映射：交付物 ← 耐久输入；写入者与准入 ← 目标准入与提交；恢复 ← 恢复依据；「失败处理」原表无独立列，标 —，由通读补。

| # | 边（方向） | 交付物 | 写入者与准入 | 恢复依据 | 失败处理 |
| --- | --- | --- | --- | --- | --- |
| 1 | Project → Task | Project/version、来源引用、可选 Task 契约及摘要、Repo Board/Project 分组锚点 | “创建 Task”命令固定不可变 `project_id` 并持久化后端创建 outbox；只有携带初始契约时才写正文 outbox，后续由“采纳契约”准入 Task Revision | 命令、幂等与关联键 → 同一 Task、外部卡和可选 Task Revision 引用 | — |
| 2 | Project / Task → Run | Project/version、可选精确 Task Revision、Workflow/Deployment refs、repo baseline、根 Context Manifest、Participant/Role/Skill、候选、权限、预算和 Gate | Run 命令原子写 Run Manifest、Task Run 占用标记、Run 账本和引擎启动 outbox | run ID + manifest digest → Engine Execution Binding/readback | — |
| 3 | Project → Participant | Room Invocation + Execution Spec | Project 先持久化调用授权，Participant 模块再预留、绑定和激活运行时 | invocation id + invocation_version + Execution Spec digest | — |
| 4 | Run → Participant | Attempt + Execution Spec | Run 先持久化派发授权，Participant 模块再预留、绑定和激活运行时 | attempt id + attempt_generation + Execution Spec digest | — |
| 5 | Participant → Project/Run | Result Proposal、逐输出的归属者/运行时/现场/Agency 绑定代次、Revision/Evidence 引用 | 归属模块去重并逐项校验身份、代次、Context Bundle、权限、写租约和输出 schema 后准入 | 提案标识符 + producer sequence + 归属者/spec digest；迟到结果只留历史 | — |
| 6 | human scene / Run reducer → Participant | 「合入 ChangeSet」命令、精确 ChangeSet Revision/target/证据引用 | Participant 模块准入授权并持久化 intent/outbox，工具箱执行与回读；Integration Receipt 返回发起模块作证据 | intent id + expected target head → 唯一 Receipt；结果未知不重投 | — |
| 7 | human Kanban / Run reducer → Task | human provenance，或正常完成 Run ref；被冻结的 Task Revision ref、Revision/Evidence/Verdict/Receipt refs | human actor 或 task-bound Run reducer 提交同一个「完成 Task」命令；Task 按当前验收约束独立校验 | 「完成 Task」命令 id → Task Completion Receipt；Harness 只提供证据 | — |
| 8 | Task/Run/Participant → Project | source ref、event id/sequence、版本、敏感级别 | Project 只建低噪声投影；Memo/Artifact 仍需 Project 命令发布 | source event cursor，可从源账本重建 | — |

共 8 条边。

---

## 强制手段清点（脚本产出，不含判断）

范围：`docs/design/spec/*.md` 全部含「必须／不得／只能」的句子。机制词表：比较并交换、CAS、摘要、digest、租约、Lease、代次、generation、归约、reducer、回读、readback、outbox、幂等、idempot、事务、栅栏、fence、Receipt、凭证、Snapshot、快照、revision、Revision、CT-、契约测试、校验、拒绝、Preview、预览。无机制词的句子单列在第二节，是 I1/I2 的候选集；有机制词不等于已被机制强制，需通读判断。

### 一、按文件统计

| 文件 | 规范句 | 含机制词 | 无机制词 |
| --- | --- | --- | --- |
| docs/design/spec/README.md | 4 | 1 | 3 |
| docs/design/spec/connections.md | 26 | 16 | 10 |
| docs/design/spec/participant.md | 47 | 18 | 29 |
| docs/design/spec/project.md | 41 | 18 | 23 |
| docs/design/spec/run.md | 27 | 13 | 14 |
| docs/design/spec/system.md | 46 | 24 | 22 |
| docs/design/spec/task.md | 34 | 23 | 11 |

### 二、无机制词的规范句（I1/I2 候选）

| 文件 | 节 | 句子（截 90 字） |
| --- | --- | --- |
| docs/design/spec/README.md | 核心产品词 | `provider` 只是供应端的泛称，必须由具体模块说明它指哪一类供应端。 |
| docs/design/spec/README.md | 三类数据 | human 请求可以来自 Workbench/CLI，也可以来自模块绑定明确接纳的 provider 动作，但必须归一到同一命令。 |
| docs/design/spec/README.md | 三类数据 | 结果可以作为记录写回 content 系统，回写本身不得再取得 human provenance。 |
| docs/design/spec/connections.md | 连接模型 | 引用还必须携带所属 Repo/Project、生产者和适用绑定版本。 |
| docs/design/spec/connections.md | Project → Task：从讨论到承诺 | 确认时，「创建 Task」命令或「采纳契约」命令必须冻结： |
| docs/design/spec/connections.md | Project / Task → Run：授权自动施工 | 提交后发生的上游更新不改写活动 Run，只能影响新 Run 或触发显式替代。 |
| docs/design/spec/connections.md | Project / Run → Participant：从授权到物理执行 | Execution Spec 必须分别引用它们，任何一个都不能代替另一个。 |
| docs/design/spec/connections.md | Project / Run → Participant：从授权到物理执行 | 除此之外不得省略物理字段组。 |
| docs/design/spec/connections.md | Participant → Project / Run：结果准入 | 每个输出都必须携带自己的生产者字段组； |
| docs/design/spec/connections.md | 跨模块 Request 回路 | 任何分支都不得投给替代执行，也不得留下活动 Seat 或 Attempt。 |
| docs/design/spec/connections.md | 版本、权限与替代 | 范围、权限、候选或验收含义变化时必须显式替代，而不是原地修补； |
| docs/design/spec/connections.md | 版本、权限与替代 | 权限只能逐级缩小：actor/Project role → Run Manifest（有 Run 时）→ Execution Spec → Agency/adapter envel… |
| docs/design/spec/connections.md | 失败与恢复 | 系统对账完成前，各模块都不得表现为已完成交接。 |
| docs/design/spec/participant.md | Skill 与申报 | 不得把 unknown 记为 known。 |
| docs/design/spec/participant.md | 写入约束 | 每次绑定都必须从实际探测结果中选择精确端口和降级方式，并冻结版本、配置、能力、信任级别和权限。 |
| docs/design/spec/participant.md | 不可关闭的三条底线 | ** Harness、运行时钩子和模型只能提交 Result Proposal，不能提交治理命令。 |
| docs/design/spec/participant.md | 可选执行加固 | Execution Spec 必须冻结已声明项。 |
| docs/design/spec/participant.md | 可选执行加固 | 未声明时，control 不施加这些加固，也不得记录为已生效； |
| docs/design/spec/participant.md | ChangeSet 与 Git 事实 | 候选切换、接管或取消必须先让旧写入者失权。 |
| docs/design/spec/participant.md | ChangeSet 与 Git 事实 | 只有自动恢复必须从获准基线创建新的 Git 工作树和 ChangeSet。 |
| docs/design/spec/participant.md | 运行时与观测 | 替代任一层只使引用该层旧值的 HCTL 动作失效，不得顺带改写其他层的身份； |
| docs/design/spec/participant.md | 运行时与观测 | 派出交付物必须按冻结规格逐项核验后方可激活； |
| docs/design/spec/participant.md | 运行时与观测 | Agency 自带的接管、单写者或“会话有效”记录只作执行协助与观测证据，不得写入或替代账本事实。 |
| docs/design/spec/participant.md | 运行时与观测 | Execution Spec 必须冻结终端输入策略。 |
| docs/design/spec/participant.md | 运行时与观测 | 供应端不能统一拦截全部写入时，系统必须关闭原生控制器。 |
| docs/design/spec/participant.md | 运行时与观测 | 执行记录必须标明输入来源不完整； |
| docs/design/spec/participant.md | 运行时与观测 | 不得声称物理单写者、完整回放，或由该输入产生 HCTL 命令或结果。 |
| docs/design/spec/participant.md | 运行时与观测 | 切换策略必须创建新 Execution Spec 或替代执行，不能在活动执行背后静默放宽。 |
| docs/design/spec/participant.md | 运行时与观测 | 未声明时不得声称来源完整。 |
| docs/design/spec/participant.md | 运行时与观测 | Agency 声明事件游标时，必须报告序号和缺口； |
| docs/design/spec/participant.md | 运行时与观测 | 未声明时，事件流只能作为有界观测。 |
| docs/design/spec/participant.md | 运行时与观测 | 证据不足时只能报告语义恢复、回放或丢失。 |
| docs/design/spec/participant.md | 运行时与观测 | 未知事件保留原文并安全降级，不得凭渲染器猜测完成。 |
| docs/design/spec/participant.md | 运行时与观测 | 每个 harness 适配器必须为其接入端口声明终局结果清单，并逐项核对： |
| docs/design/spec/participant.md | 运行时与观测 | - 执行体进程正常退出但缺少清单要求的终局结果事件时，适配器必须合成类型化协议错误，不得默认成功。 |
| docs/design/spec/participant.md | 运行时与观测 | - 由 control 主动取消导致的退出必须归因为取消，不得上报为执行失败。 |
| docs/design/spec/participant.md | 运行时与观测 | - 观测上报通道失败时，只能显式标记该执行的观测截断并终结事件流，不得交付有缺口的事件流冒充完整历史。 |
| docs/design/spec/participant.md | 运行时与观测 | - harness 内部派生的子执行体事件必须携带稳定的派生谱系引用，不得摊平进主执行流。 |
| docs/design/spec/participant.md | 运行时与观测 | 受信任的 `in_process` Proposal 使用缩减头，而且不得提交 ChangeSet。 |
| docs/design/spec/participant.md | 运行时与观测 | 修正必须创建新 Proposal 和新的生产者序号，不得改写原项。 |
| docs/design/spec/participant.md | 终端通道、连接与租约 | 能力不足时准确降级为 structured inspect 或 terminal，不得改投另一个会话。 |
| docs/design/spec/participant.md | 终端通道、连接与租约 | 无法证明是同一进程时只能 semantic resume、replay 或新建执行，不能声称 exact attach。 |
| docs/design/spec/project.md | Repo 注册与 Project 归档 | 缺失或冲突必须要求用户处理，不得静默合并。 |
| docs/design/spec/project.md | Repo 注册与 Project 归档 | repo_scope Room Invocation 改为冻结 Repo Instance/repo/base 且只能只读。 |
| docs/design/spec/project.md | Repo 注册与 Project 归档 | Execution Spec 必须分别冻结四者的精确引用。 |
| docs/design/spec/project.md | Repo 注册与 Project 归档 | 不得复制整段 Room、把隐式聊天窗口当作来源，或让后续 Room 消息改变既有 Project。 |
| docs/design/spec/project.md | Room 与消息 | 创建 Scoped Room 时必须冻结 parent Room、讨论目标、完成条件和回填动作。 |
| docs/design/spec/project.md | Room 与消息 | 回填失败时，Room 和目标引用必须保留为可恢复状态。 |
| docs/design/spec/project.md | 三种交付方式 | 必用内容超出预算时，必须降为 `pointer` 并附分片建议，不得静默丢弃。 |
| docs/design/spec/project.md | 三种交付方式 | 它只能指向执行体在获准范围内可自行打开的 Git 对象或 worktree 路径； |
| docs/design/spec/project.md | 三种交付方式 | 账本和任务后端内容不得作为 `pointer`。 |
| docs/design/spec/project.md | 根 Context Manifest | 每次顶层授权必须冻结一个根 Manifest，并包含： |
| docs/design/spec/project.md | 根 Context Manifest | Repo Room、Project Room 和 Run 之间只能通过这些 parent/source 引用传承 Context。 |
| docs/design/spec/project.md | 根 Context Manifest | 无论采用哪种方式，每次判定都必须把输入事实引用与结论记为可审计观测； |
| docs/design/spec/project.md | 根 Context Manifest | 压缩产物的每个片段都必须能回到原文位置。 |
| docs/design/spec/project.md | 根 Context Manifest | 它不是权威：治理引用不得指向纪要，只能指向精确事件； |
| docs/design/spec/project.md | Room Invocation | 来源建议必须精确引用 chat server 事件 ID 或 Result Proposal； |
| docs/design/spec/project.md | Room Invocation | 父执行必须精确引用 Room Invocation 或 Attempt。 |
| docs/design/spec/project.md | Room Invocation | 写入、Project Artifact 或 Project 范围权限必须选择精确 Project 与版本。 |
| docs/design/spec/project.md | Request | 上述阻塞身份相同的重复创建必须去重到现有活动 Request，可以追加提醒事件。 |
| docs/design/spec/project.md | Request | 任一归属者、版本、范围或所需动作变化时，control 必须创建新 Request 并取代旧 Request； |
| docs/design/spec/project.md | Request | 旧解决结果不得推进新阻塞项。 |
| docs/design/spec/project.md | 场景约束 | 模型 Participant 的 Message、Result Proposal、总结及其正文中的 `@` 只能形成下一位 Participant/Role 与扇出建议，不能自行创… |
| docs/design/spec/project.md | 场景约束 | mention 的解析必须确定性：`@` 目标只按获准的 Participant/Role 绑定精确解析； |
| docs/design/spec/project.md | 场景约束 | 无唯一授权候选时必须明确失败或要求人选择，不得按显示名模糊匹配、静默换人或把 mention 字符串交给模型猜测路由。 |
| docs/design/spec/run.md | 写入约束 | 启动中、暂停中和取消中都必须能通过取消、失败或替代进入终态，不能因 workflow engine 失联永久阻塞绑定 Task。 |
| docs/design/spec/run.md | 写入约束 | Run 进入完成前，control 必须逐项证明： |
| docs/design/spec/run.md | 写入约束 | 任何一项未知都不得完成 Run。 |
| docs/design/spec/run.md | 写入约束 | 若只能撤销逻辑权威而无法证明旧进程已静默，则隔离旧 Git 工作树和 ChangeSet； |
| docs/design/spec/run.md | Workflow 与 Run 授权 | 只覆盖局部研究、咨询或中间步骤的自动化必须使用无 Task Run 或 Room Invocation，并以稳定引用把结果交回 Task； |
| docs/design/spec/run.md | 启动与 Manifest | 旧写入未能在物理上证明静默时，还必须按 [Participant 约束](./participant.md#changeset-与-git-事实)使用新的 ChangeSet 和 … |
| docs/design/spec/run.md | 启动与 Manifest | 范围、验收、候选、权限、Gate 或超出获准边界的放置变化必须创建替代 Run，不能原地漂移。 |
| docs/design/spec/run.md | 启动与 Manifest | 3. dynamic fork 只能实例化 Manifest 中已冻结的有界 Seat 模板； |
| docs/design/spec/run.md | 启动与 Manifest | dynamic fork 的候选 Participant/Role、最大基数、预算、选择函数和权限上限都必须预先固定； |
| docs/design/spec/run.md | 从节点到结果 | Execution Spec 必须固定 Attempt、Seat、Run、Participant、Role Binding、Worker Profile、Agency 绑定、Con… |
| docs/design/spec/run.md | 从节点到结果 | 修正或重新施工必须创建新的 Attempt 和 Proposal，不能复活旧 Attempt。 |
| docs/design/spec/run.md | Request、重试与 Gate | 备用 Attempt 必须继承原 Seat 的逻辑身份和全部评审依据，不能借更换 Worker Profile 改变 Context、Skill、权限、票位或绕过分离。 |
| docs/design/spec/run.md | Request、重试与 Gate | 受控端口能认证的供应端、模型和操作者信息必须按 `known/unknown` 展示； |
| docs/design/spec/run.md | Request、重试与 Gate | 策略要求物理或组织独立、但当前端口无法认证时，Gate 必须返回 unsupported。 |
| docs/design/spec/system.md | 固定内核与受控端口 | 换掉全部界面与供应端之后内核必须保留什么，[愿景文档](../vision.md#产品原生核心与架构最小内核)已经回答； |
| docs/design/spec/system.md | 客户端使用的公开面 | 未来的远程 Agency 必须直接实现 Agency 约束，或通过专用适配器接入； |
| docs/design/spec/system.md | 端点与输入的信任边界 | 1. 第一阶段的 chat server、本地任务服务器、workflow engine 和 Herdr 管理/API 端点只能绑定本机回环地址或仅归属者可访问的本地套接字。 |
| docs/design/spec/system.md | 端点与输入的信任边界 | 未来的非本地传输必须认证客户端。 |
| docs/design/spec/system.md | 端点与输入的信任边界 | 绑定未声明这些能力时，原生交互只能按来源不完整的运行时输入记录。 |
| docs/design/spec/system.md | 端点与输入的信任边界 | 无论采用哪种输入模式，HCTL 结果都只能从 Result Proposal 准入。 |
| docs/design/spec/system.md | 端点与输入的信任边界 | 绑定的供应端不可用时，control 必须按该绑定冻结的降级策略暂停或终结活动执行。 |
| docs/design/spec/system.md | 端点与输入的信任边界 | 联网探测只能由用户显式启用。 |
| docs/design/spec/system.md | 端点与输入的信任边界 | 同一端口种类和作用域在一次准入中只能解析出一个绑定版本，结果不得受加载顺序或界面选择顺序影响。 |
| docs/design/spec/system.md | 端点与输入的信任边界 | Room、Task、Run 和 Execution Spec 都必须引用精确绑定版本。 |
| docs/design/spec/system.md | 端点与输入的信任边界 | 凭据引用只指向 secret store 条目，不得包含密钥。 |
| docs/design/spec/system.md | 端点与输入的信任边界 | 不可信扩展必须使用操作系统强制隔离和能力削减的代理接口。 |
| docs/design/spec/system.md | 场景端口 | 客户端只能声明自己的交互能力和降级方式，受控端口只能报告供应端能力。 |
| docs/design/spec/system.md | 客户端动作与 provider 事件 | 重复、迟到或乱序投递必须得到相同结果。 |
| docs/design/spec/system.md | 命令与跨服务正确性 | Harness、模型和执行主体只能提交 Result Proposal，不能自报为 human。 |
| docs/design/spec/system.md | 命令与跨服务正确性 | 结果未知时不得盲目重做。 |
| docs/design/spec/system.md | Repo 与执行现场 | 身份缺失、分支来源语义不明、一个公共目录已归属另一 Repo，或证据互相冲突时，系统不得静默挂接。 |
| docs/design/spec/system.md | 控制面自己的存储 | 一人多机连接同一本账本，账本必须备份。 |
| docs/design/spec/system.md | 备份与恢复 | 恢复只能在旧写入者已经停止且取得用户级排他锁后进行； |
| docs/design/spec/system.md | 备份与恢复 | 不得合并两份分叉账本，也不得把备份恢复成新的账本身份。 |
| docs/design/spec/system.md | 备份与恢复 | 系统不得把空值当凭据或静默降权。 |
| docs/design/spec/system.md | 安全边界 | - 日志与 trace 使用稳定关联 ID，但不得包含密钥和完整敏感 payload。 |
| docs/design/spec/task.md | 契约与来源 | 做不到时，该后端只能显示过滤视图，不能声称支持 Task 身份导入或跨组移动。 |
| docs/design/spec/task.md | 契约与来源 | 系统不得猜测 Project 或先创建 Task。 |
| docs/design/spec/task.md | 契约与来源 | 无契约的“启动 Run”或“完成 Task”必须先要求该独立动作。 |
| docs/design/spec/task.md | 契约与来源 | 并发命中同一实体时只能复用同一 Task 或返回类型化冲突。 |
| docs/design/spec/task.md | 契约与来源 | 在恢复原位置或建立新 Task 前，系统必须阻止采纳、启动、完成和后端操作字段写入。 |
| docs/design/spec/task.md | 契约与来源 | “移动 Task”只能在原 Project 锚点内改变阶段和排序； |
| docs/design/spec/task.md | 契约与来源 | 重开、取消、跨 Project 移动和契约采纳第一阶段没有供应端动作映射，必须使用公共命令入口。 |
| docs/design/spec/task.md | 写入约束 | 替代只能走 [Run 约束](./run.md#启动与-manifest)规定的原子撤权和换代路径，不能先清空标记再留下两个可写执行。 |
| docs/design/spec/task.md | 写入约束 | 用户必须先显式结束该 Run 并等待旧执行撤权、隔离； |
| docs/design/spec/task.md | 写入约束 | 「重开 Task」命令只接受有权 human actor，必须以预期 task_lifecycle_version 把完成/已取消 → 开放并推进版本； |
| docs/design/spec/task.md | 启动 Run 的前置与排序令牌 | 存在未处理的待采纳时不得启动 Run，control 也不得自动采纳或静默越过。 |

---

## 部件清点（脚本产出，不含判断）

来源：`docs/research/README.md` §条目索引。后三列空着，由部件矩阵调研填。

| 对象 | 类别 | 现有复用决策 | 业界最佳 | 借用等级 | 取舍 |
| --- | --- | --- | --- | --- | --- |
| OpenCode、Pi 与 Kimi Code | ① Coding Harness | 适配协议 | — | — | — |
| DeepSeek Harness / Cordis | ① Coding Harness | 仅参考行为 | — | — | — |
| First Tree | ② Agent 协作平台 | 移植有边界的组件 | — | — | — |
| Claude Tag | ② Agent 协作平台 | 仅参考行为 | — | — | — |
| Grok Bot 与 Grok Build | ② Agent 协作平台 | 仅参考行为 | — | — | — |
| Cumora | ② Agent 协作平台 | 移植有边界的组件 | — | — | — |
| LobeHub | ② Agent 协作平台 | 仅参考行为 | — | — | — |
| Multica | ② Agent 协作平台 | 仅参考行为 | — | — | — |
| Helio | ② Agent 协作平台 | 核心仅参考行为；开源外围有边界移植并适配协议 | — | — | — |
| Codeg | ② Agent 协作平台 | 仅参考行为为主，可按需移植 | — | — | — |
| Stably Orca | ② Agent 协作平台 | 仅参考行为为主，可按需移植 | — | — | — |
| Superset | ② Agent 协作平台 | 仅参考行为 | — | — | — |
| OpenClaw | ③ 独立 Agent 产品 | 仅参考行为并适配协议 | — | — | — |
| Hermes Agent | ③ 独立 Agent 产品 | 仅参考行为 | — | — | — |
| Rakazo | ③ 独立 Agent 产品 | 仅参考行为为主，可按需移植 | — | — | — |
| ZeroClaw SOP | ③ 独立 Agent 产品 | 仅参考行为 | — | — | — |
| MyContext | ④ Context 管理 | 仅参考行为 | — | — | — |
| Codex Remote Feishu | ⑤ 远程操控与会话同步 | 仅参考行为 | — | — | — |
| MindFS | ⑤ 远程操控与会话同步 | 仅参考行为 | — | — | — |
| Paseo | ⑤ 远程操控与会话同步 | 适配协议 | — | — | — |
| HAPI | ⑤ 远程操控与会话同步 | 仅参考行为 | — | — | — |
| Happy | ⑤ 远程操控与会话同步 | 仅参考行为 | — | — | — |
| Remux | ⑤ 远程操控与会话同步 | 仅参考行为 | — | — | — |
| Moshi | ⑤ 远程操控与会话同步 | 仅参考行为 | — | — | — |
| ServerCC | ⑤ 远程操控与会话同步 | 仅参考行为 | — | — | — |
| QuickTUI | ⑤ 远程操控与会话同步 | 仅参考行为 | — | — | — |
| Redock | ⑤ 远程操控与会话同步 | 仅参考行为 | — | — | — |
| Herdr | ⑥ 机械后端与基础设施 | 采用为依赖 | — | — | — |
| Dagu 机械状态后端与 workflow 候选复审 | ⑥ 机械后端与基础设施 | 采用 Dagu 为依赖，其余候选暂缓 | — | — | — |
| chat server 选型（限时验证） | ⑥ 机械后端与基础设施 | 采用 Tuwunel 为依赖，Continuwuity 暂缓 | — | — | — |
| L3 外部系统与观察清单 | ⑥ 机械后端与基础设施 | 采用 Vikunja 为依赖，git-bug 暂缓，Linear/GitHub 适配协议 | — | — | — |
| 运行时后端复审 | ⑥ 机械后端与基础设施 | 历史选型，不再采用 | — | — | — |
| Herdr 作为 Agent / Terminal 运行服务的验证清单、源码核对与 macOS 实测 | ⑥ 外部运行服务与基础设施 · 补充审计 | 采用 Herdr 的验证证据 | — | — | — |
| Agent 运行服务候选的源码复审：Termio、tty7、cmux、Pilotty 及相邻候选 | ⑥ 外部运行服务与基础设施 · 补充审计 | 旧结论废止，源码与实测证据保留 | — | — | — |
| Workbench 桌面壳：Electron 与 Tauri 2 | ⑥ 机械后端与基础设施 | 采用 Tauri 2，Electron 为安全网 | — | — | — |
| HCTL1 / yesme/hctl | ⑦ 直接谱系 | 仅参考行为（直接谱系证据） | — | — | — |
| 方法论工具十二族与完成判定权横评（11 个仓库各钉 commit） | 方法论生态 | 逐项适配协议、有边界移植或仅参考行为 | — | — | — |
| mattpocock/skills（Skills for Real Engineers）：wayfinder 与 grill 系逐源码审计、完成判定权专项、方法 / 对象 / 机制的分界原则 | 方法论生态 · 单对象补充审计 | 仅参考行为；采用为依赖、适配协议、移植组件均为零 | — | — | — |
| Context 处理生态四族与快省准横评（链接级） | ④ Context 管理 | 仅参考行为 | — | — | — |
| Grok Bot 0.18 客户端重建源码审计（`a9f633e`），[grok-bot.md](./workbench/grok-bot.md) 的补充证据 | ② Agent 协作平台 | 仅参考行为的补充证据 | — | — | — |
| Workbench 桌面壳重开调研：GPUI / Iced / Flutter / Web 壳，含 7 份附录 | ⑥ 机械后端与基础设施 | 采用 Tauri 2 的选型证据 | — | — | — |
| DotSlash 官方 GitHub Action 与本地引导安装审计 | ⑥ 机械后端与基础设施 | Linux CI 锁 Action 提交；macOS CI 与开发机使用摘要锁定安装器 | — | — | — |
| Buck2 Change Detector 的源码、官方制品与失败回退审计 | ⑥ 机械后端与基础设施 | 采用官方 `btd` 二进制；不自行构建 `supertd` | — | — | — |
| BTD JSON Lines 解析工具与官方制品审计 | ⑥ 机械后端与基础设施 | 采用摘要锁定的官方单文件制品；不依赖宿主预装 jq | — | — | — |
| GitHub Actions 增量重验证 | ⑥ 机械后端与基础设施 | 采用平台原生 workflow 证据；快进更新增量验证，失败时全量回退 | — | — | — |

---


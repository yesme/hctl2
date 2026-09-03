# Fixture vocabulary

This file is a parser-stable subset of `docs/design/spec/README.md`. It exists so
checker fixtures do not inherit live-document product names.

## 核心产品词

| 词 | 中文对照 | 类别 | 可见性 |
| --- | --- | --- | --- |
| Repo | 仓库 | 对象 | 用户可见 |
| Project | 项目 | 对象 | 用户可见 |
| Room | 聊天室 | 对象，也是 Project 模块的场景名 | 用户可见 |
| Participant | 参与者 | 对象 | 用户可见 |
| Request | 请求卡 | 对象 | 用户可见 |
| Memo | 备忘 | 对象 | 用户可见 |
| Artifact | 工件 | 对象 | 用户可见 |
| Context | 上下文 | 横切概念；其清单与包是票据 | 用户可见 |
| Skill | 技能包 | 对象 | 用户可见 |
| Task | 任务 | 对象 | 用户可见 |
| Kanban | 看板 | Task 模块的场景名 | 用户可见 |
| Run | 一次受治理施工 | 对象 | 用户可见 |
| Workflow | 施工图 | 对象，也是 Run 模块的场景名 | 用户可见 |
| Terminal | 终端 | Participant 模块的场景名 | 用户可见 |
| Workbench | 工作台 | 客户端产品 | 用户可见 |
| Receipt | 凭证 | 票据 | 用户可见——「完成不能自述」靠它，愿景层要讲 |
| Gate | 评审关卡 | 节点类型 | 治理内部；愿景层只以中文「评审关卡」出现 |
| Obligation | 交付义务 | 对象 | 治理内部 |
| Seat | 席位 | 对象 | 治理内部 |
| Attempt | 尝试 | 对象 | 治理内部 |
| ChangeSet | 变更集 | 对象 | 治理内部 |
| Verdict | 裁决 | 票据 | 治理内部 |
| Evidence | 证据 | 票据 | 治理内部 |

另有八个高频约束词可在设计正文携中文对照使用：Task Revision（任务契约版本）、Workflow Revision（施工图版本）、Room Invocation（单次调用）、Execution Spec（执行规格）、Result Proposal（结果提案）、Run Manifest（施工清单）、Context Manifest（根上下文清单）、Context Bundle（消费上下文包）。

## 词汇索引

- **Revision 族**：Task Revision、Workflow Revision、ChangeSet Revision、Artifact Revision、Extension Revision、Engine Deployment
- **Binding 族**：Port–Provider Binding、Room–Server Binding、Task–Backend Binding、Run–Engine Binding、Participant–Agency Binding
- **Receipt 族**：Gate Receipt、Task Completion Receipt、Integration Receipt
- **Lease 族**：Write Lease、Terminal Input Lease
- **Snapshot/观测族**：Task Backend Snapshot、Result Proposal
- **票据与规格**：Execution Spec、Run Manifest、Attach Descriptor、Context Manifest、Context Bundle
- **引用格式**：ReviewSubjectRef
- **独立对象**：Repo Instance、Room Invocation、Execution Runtime、Worker Profile

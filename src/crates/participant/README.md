# Participant 控制面半边

第 2 包实现 Agency Binding、工种的显式接受、派工意图与映射、观测记录和 Proposal 字节保全。它依赖 Store 与共享合同，不依赖 Agency 服务、Herdr 或执行进程。`apps/control/src/agency.rs` 处理本地消费、RPC 与私有配对凭据。

## 已有接法

`accept_binding` 固定公开目录，`accept_profession` 接受精确名册项。`prepare_dispatch` 要求已保存的授权归属者与接受过的工种；它在 Store 事务里保存规格与准备 outbox。`record_dispatch` 同事务保存映射、确认准备与激活 outbox。控制面端口发送外部动作前调用 `begin_effect`；响应未知先回读，不盲目重发。

`preserve_proposal` 保存并回读精确材料，`proposal_inbox` 只是接收与审计，不是 Project / Run 准入。观测、联系不上和无法履约不能自行完成 Task 或 Invocation。Buck：`root//crates/participant:participant`、`:clippy`；完整端口链的测试在 `root//agency:control_port_test`。

## 第 5 包任务说明

当前端口只核授权归属者的精确记录版本；完整领域授权仍未接线。第 5 包在恢复 Pending 前传入真实的授权判定，不沿用端口中的 `still_authorized=true`；Unknown / Confirmed 的回读和字节保全不发新授权，但后续激活、输入与准入另核当前语义归属。当前逐份成果保全在首个错误处返回；第 5 包接多成果时改为逐份报告，不让一个坏成果挡住其余保全。

依据：[演示线开工书第 5 包](../../../.memo/design/p2-control-20260906/07-demo-kickoff.md#第-5-包--派工)、[从授权到派工](../../../docs/design/spec/connections.md#project--run--participant从授权到派工)、[结果准入](../../../docs/design/spec/connections.md#participant--project--run结果准入)。

| 文件 | 要补什么 |
| --- | --- |
| `participant/src/selection.rs` | Room / Run 选入记录、候选校验、Worker Profile；当前 `project` 的名册接线转为真实工种引用 |
| `participant/src/tickets.rs` | 按当前归属者、冻结规格、权限与控制面写者签发短期票据；连接不恢复领域授权 |
| `project/src/invocation.rs` | Room Invocation 的预览、状态机、语义版本、取消与重试、结果准入和 Room 投影；不要放到本 crate |
| `apps/control/src/dispatch.rs` | 调用 Context 组装器和本包的准备 / 映射接口，再沿 Agency 端口激活；恢复原 outbox |
| `apps/cli/src/invocation.rs`、`terminal.rs` | Invocation 与 Terminal 命令；端口的观察和输入对手方是 Agency |

先走只读调用。没有唯一、获准的本 Room 候选时拒绝；模型提及与建议不创建调用。预览冻结执行者、Context、权限、预算与评审发布策略。四步启动沿现有接口，不在 RPC 成功后补写授权。当前测试中的授权归属者是测试预置记录，不表示 Invocation 业务已交付。

每份结果保全后还要校验归属者状态与语义版本、绑定、Spec / Bundle、逐项输出范围、权限和证据，再由 Project 准入；迟到或失权的结果只留审计。包 6 才补 Write Lease、封存与 ChangeSet 准入。派工票据签发、真实 Terminal 连接、丢失判定与停止隔离报告也在后续业务接线，不能借当前配对凭据直接给模型命令权。

失败用例：未选入名册、跨 Room 授权、旧语义版本、旧预览、错误输出授权、取消期间返回结果、响应丢失与控制面重启、未知输入投递、停止报告缺失。对照 CT-PROJECT、CT-PARTICIPANT、CT-CONNECTION 的现行行逐条列已做与未做，不把端口测试当领域准入测试。

# 场景数据模型：4 场景 × 3 类数据（讨论备忘）

> 状态：已落地（v0.10.0 全量写入）<br>
> 基线：main @ 3aa2950（草案 v0.9.1）<br>
> 去向：docs/design/architecture.md + spec/system.md 4×3 归属与丢失恢复合同 + decision-history §12<br>
> 日期：2026-08-20<br>
> 主题：metadata / content / artifact 三分与四场景的完整矩阵；由此确立的架构与术语方向<br>
> 说明：已落地——2026-08-21 随草案 v0.10.0 全量写入设计文档（术语正名 Agent 模块、三面架构 architecture.md、4×3 归属与丢失恢复合同、选型判据与五项限时验证）；决策记录见 decision-history §12。关联：[room-ground-truth-20260819](./room-ground-truth-20260819.md)、[design-doc-method-20260819](../notes/design-doc-method-20260819.md)。

## 出发点（所有者论断）

hctl-bench 看上去是本机一体的 IDE/ADE，实质是瘦客户端——chat-room、kanban、workflow、harness session 本质上都跑在“服务器”上；bench 是它们的“象”，服务器是“实”。把客户端和服务器装在同一台机器上不改变 client-server 实质。因此“单人假设”要拆开：单用户（单租户授权）仍成立；单机（部署共址）只是默认部署形态。已写入 vision.md：**架构是 client-server 的，local-first 是默认部署形态而不是架构假设。**

## 核心：4 场景 × 3 类数据

| 场景 | **metadata**（HCTL 自己的库） | **content**（第三方 ground truth） | **artifact**（Git） |
| --- | --- | --- | --- |
| Chat Room | Room 身份、归属 Project、Participant 名册与角色绑定、桥接配置、“哪条消息升格成了什么”的记录 | 聊天记录、调用过程与结果卡（Matrix homeserver：Synapse / Continuwuity） | 决议、Memo |
| Kanban | Task 身份映射、字段权威绑定、冻结契约（或其 digest）、完成凭证 | 任务卡、流转、排序、评论（Plane / Linear / GitHub） | 冻结的任务契约、施工图 DAG |
| Workflow | Run 授权、引擎绑定、代次、Gate 规则、裁决 | 令牌位置、重试、定时器、机械执行历史（Conductor） | 凭证链（Receipt/Verdict 及其精确引用） |
| Terminal | 执行授权（派发规格）、写租约、输入租约、代次、观测账 | 会话转录、PTY 流（harness session / RuntimeBackend） | ChangeSet → 合并的代码（全系统最重要的产出） |

现行设计里 Workflow 与 Terminal 两行本来就这么运作（Conductor 拥有机械历史、harness session 拥有转录，HCTL 只留绑定与治理）；本提案是把这条规则统一推广到 Chat 与 Kanban，补掉现行设计的不对称。

## 统一律与 metadata 的准确定义

> **每个场景的 artifact = 该场景 content 的结晶。**
> 讨论 → 决议/Memo；任务流转 → 冻结契约/施工图；机械执行 → 凭证链；会话字节流 → 代码变更。

**metadata 是贯穿三者的骨架：身份、绑定、授权、判决。** 它不含记忆（记忆在 content），不含成果（成果在 Git），只含“谁是谁、谁连着谁、谁批了什么、凭什么算数”。

此模型解开了此前“台账 vs Git 分不清”的结：旧“语义账本”混装了记忆（content）与判决（governance）；三分法把记忆交给第三方 ground truth，判决留在 HCTL 库，结晶归 Git。

## 三条必须显式立的法

1. **能承载 ≠ 能裁决。** 平台拥有 content 的 ground truth，永远不拥有治理：Matrix 消息不能触发派发，Plane 拖卡不能完成 Task，Conductor 的 COMPLETED 不能签凭证。判决只在 metadata 层产生。（现行“外部平台不能成为第五事实源”的精确化：可以拥有 content，不能拥有 governance。）
2. **冻结摘要是两个世界之间的防火墙。** content 世界可变，治理世界不可变；现有 snapshot/adoption/digest 机制原封不动地成为这道墙——Run 开工前把依赖的 content 冻结为带摘要的引用，此后 content 漂移改不了已授权的施工。连接合同层因此几乎不用动。
3. **命令走 HCTL，记录落平台。** Trigger Preview、类型化命令、准入校验在 metadata 层执行；结果可作为自定义事件写回 content 平台（Matrix 支持自定义事件类型），但平台里的记录只是记录，不是命令。

## 代价与倾向

- **主要价格是本机部署重量**：control + agentd + Conductor 之外再加 Matrix homeserver 与任务服务器。所有者修正：**轻不等于嵌入式**——Matrix homeserver、Conductor 和 Harness（Codex/Claude Code 这类独立进程）都不是嵌入式；“轻”指轻量实现选择（如 Continuwuity 之于 Synapse）、低资源占用，以及随 HCTL 一键启停的生命周期托管，而不是把服务塞进 control 进程。因此任务场景的方向也是找轻量的独立任务服务器（保持四场景全真服务器、模型无例外），而非嵌入式默认；Plane 偏重，作为候选之一进入选型评估。各项均需 Conductor 式的开工前限时验证。
- **次要价格**：每个 content 平台的幂等/顺序/恢复合同逐场景核验（Matrix 事务 ID 幂等、单 homeserver 线性顺序，过关；Plane 类待验）；平台宕机降级合同——故障隔离反而更好（聊天服务器挂了，治理与施工照常）。

## 顺手了结的存量问题

- **Room ground truth**：Matrix 类服务器是 content 的家；此前对“平台当权威”的反对在三分下失效（平台只承载 content，不承载治理）。
- **hub 问题缩小**：需要随所有者走的只剩 metadata——小、私密、低频；多设备的大头（聊天同步）Matrix 原生解决。
- **多用户路径**：Matrix 成员制 + 任务服务器工作区权限天然就绪；metadata 层加用户系统即可。

## 待下一轮讨论的三个刀口

1. 冻结契约与凭证放 metadata 库还是升格进 Git（审计与随仓库 vs 仓库可能公开的隐私）；可能按仓库策略可配，需定默认值。
2. 施工图 DAG 归 Kanban 还是 Workflow 的产出。当前倾向所有者的分法（Kanban=规划场景，结晶“干什么的计划”；Workflow 结晶“干成了的证明”）。
3. 本机默认栈：四场景全真服务器为方向；待选型的是各场景的轻量实现（chat 已有 Continuwuity 候选，task 服务器候选待评估）与统一的生命周期托管方式。

## 落入 design doc 的组织方向（所有者定调，待展开）

下一轮 design doc 改造按**术语、架构、选型**三类进行，且这个分类要体现在文件命名与组织上：

- **术语**：场景与系统分开命名——Terminal 是场景、Harness 是系统；Workflow 是场景、Conductor 是系统；Chat Room 是场景、Matrix homeserver 是系统；Kanban 是场景、任务服务器是系统。现行文档用 Harness 兼指场景与系统，需要正名。
- **架构**：在四模块之上进一步切出三个面——**展示面**（hctl-bench 及第三方客户端）、**控制面**（hctl-control：metadata 与治理）、**执行面**（chat server、task server、workflow server、harness——即各场景 content 的承载系统与物理执行）。
- **选型**：如何让本机部署尽量轻。轻不等于嵌入式：Matrix homeserver、Conductor、Harness 都是独立进程/服务器；轻指轻量实现选择、低资源占用与随 HCTL 一键启停的生命周期托管，限时验证兜底。

预计改动不小；先讨论清楚再动手。

# 设计演进记录

> Status: Informative · Draft v0.7.0<br>
> 本文解释设计如何收敛，不重新打开已定案 contract。

1. HCTL1 / yesme/hctl 先把 Git-native Seat/claim/fencing、exact Verdict/quorum、Receipt 与 fail-closed reconcile 做成规范、Go kernel 和 executable corpus；HCTL2 从这条直接谱系出发，确认 core 必须保留 deterministic SCM facts，但不能把 Git-only operational truth 或旧 Seat/Obligation 语义原样放大。
2. 初期把产品理解为 multi-Harness terminal/worktree manager，研究 Zellij、WezTerm、cmux、Herdr。
3. 发现 Project 才是 user logical unit，terminal 应退到 execution/debug path。
4. 引入 Conductor，将 mechanical workflow progression 移出 LLM 和 terminal。
5. 借 Multica/Linear/Jira，明确 persistent Task、Attempt、Attention 与 Board 的差异。
6. 借 Claude Tag，认识到 shared Room 应独立于常驻“包工头进程”。
7. 区分 Repo Room、Project Room 与 on-demand Scoped Room；Room topology 与 Plan/Build control topology 正交。
8. 明确 Planning 可以只产出 Git Artifact；只有 durable automated construction 才需要 Run/Workflow。
9. 固定 Project Overview + HCTL Task Kanban，删除公共 Workspace/Work Item 和 lossy external-name mapping。
10. 对照 BPMN、Dagu、Duroxide 后，确认 Engine 应独立、被动；HCTL 保留 effect/semantic governance，采用 Conductor external-worker seam。
11. 研究 Agent Skills，明确 Skill 是 guidance/expertise，不是 gate；Receipt/core 才能改变 formal state。
12. 深入 Codeg，验证 ACP/Composer/Skills/event cards 与 async Task UX；同时拒绝 lead-agent routing、Conversation=Room、fixed To-do pipeline=Workflow。
13. 扫描 MindFS/Paseo/HAPI/Happy，确认 provider daemon、session sync、remote control 已有大量实现；HCTL 不必重造整套 remote platform。
14. 扫描 Redock/Remux/ServerCC/QuickTUI/Moshi，确认 mobile exact terminal、handoff、structured inspect、resume 和 replay 必须拆成 capability taxonomy。
15. 研究 Termio，确认 Harness manifest、status authority 与 session-control 有成熟 contract；采用 protocol/fixture thinking，而不把 ATP 改名为 HCTL standard。
16. 深入 Stably Orca，确认 daemon-owned PTY、worktree/diff/remote、generation/fencing/reconnect 是 L1 strongest evidence；其实验 Run/Task/Dispatch 只作 adjacent radar，不成为 second workflow truth。
17. 对照 Codeg 与 Stably Orca 后，确定它们没有放反：Codeg 的 highest information gain 是 independent Task/review lifecycle（L3），Stably Orca 是 execution/runtime continuity（L1）。
18. 深入 First Tree，修正“所有相邻产品都 session-first”的过度概括：persistent Chat + Context Tree + Human Request 可以形成完整 L4 product axis；同时确认它没有 first-class Task、Workflow governance 或 public exact terminal attach。
19. 将 First Tree Context/Need You/Inbox/Feishu audit 转成 Memo promotion、Request reducer、Room bridge 的 targeted contract；不移植 Team/Agent/Chat/PostgreSQL domain。
20. 审计 Codex Remote Feishu 后纠正其层级：飞书只是 provider managed session 的 remote control/projection client，highest information gain 是 L1 attach/route、queue/steer、Request 与 recovery 状态机；它不定义 canonical Room，也因未声明 repository license 只作 behavior evidence。
21. 重访 external Task，决定 HCTL UI 保留稳定 Task identity/acceptance/verification，同时允许 Linear/GitHub 按 binding revision 拥有 operational fields；用 snapshot/adoption、dual completion、outbox/read-back 保持双方真实。
22. 重访 Room UI，固定 RoomStore/Projector/ProjectionStore、concurrent Invocation 与 timeline order；`virtua`、assistant-ui scoped renderer、Tiptap 和 mature chat tests 只解决 bounded primitives。
23. Terminal 进入同一 Workbench 后，选择 xterm.js 作为 narrow embedded client、WezTerm 作为 external escape path，PTY/runtime truth 留在 agentd/Backend。
24. 最终将全部探索收敛为四层：L4 First Tree、L3 Codeg、L2 HCTL lineage + HCTL2-native + Conductor、L1 Stably Orca；其他二十多个 efforts 只在最擅长的一个层保留 targeted evidence，平庸 overlap 不再重复引用。
25. 扩展 Harness evidence bench：Claude Tag 只强化 L4 shared steerable collaboration；OpenCode、Pi、Kimi Code 与 DeepSeek Harness/Cordis 只强化 L1 access/capability/composability contract，均不获得 HCTL 上层对象定义权，也不自动扩大 Phase 1 scope。
26. 审计 always-on agent products 后，只保留各自最深切片：OpenClaw 的 multi-channel identity/routing 进 L4，Hermes 的 durable Task/attempt recovery 进 L3，ZeroClaw 的 revision-scoped SOP admission/approval 进 L2 adjacent evidence；三者都不成为新的 layer anchor。

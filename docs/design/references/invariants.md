# 规范性不变量

> Status: Normative · Draft v0.7.0<br>
> Parent: [HCTL2 设计规范](../README.md)<br>
> Rule: 这些规则优先于实现 evidence、UI convenience 与 donor semantics。

## Cross-layer

1. **X-ID-01** Repo、Project、Task、Room、Run 的 stable identity 不依赖 Workbench、provider、Harness、worktree、session、PTY 或 terminal。
2. **X-SEAM-01** 跨层变化只能通过 typed command/proposal/evidence seam；共享显示或自然语言暗示不能产生 hidden mutation。
3. **X-AUTH-01** 每个 mutation 校验 actor、scope、capability、authority policy、expected revision 与 idempotency。
4. **X-AUTH-02** Workbench、CLI 与 future client 使用同一 admission；HCTL-native UI 没有绕过 SQLite/Engine/provider 的 hidden privilege。
5. **X-REV-01** 每个 Verdict/Receipt 绑定 subject revision 与 policy digest；stale/unauthorized/duplicate evidence 无效。
6. **X-EFFECT-01** Cross-service command 使用 durable inbox/outbox、idempotency、fence、result journal 与 reconcile；local intent 先于 external effect。
7. **X-PROJ-01** Board、Graph、RoomProjectionStore、assistant-ui、xterm buffer、React store 与 external status card 都是 projection/client state。
8. **X-REUSE-01** Donor object name、DB/store 或 public schema 不反向定义 HCTL；移植必须 pin source、verify license、retain attribution/change record 并有 contract tests。
9. **X-DOGFOOD-01** Dogfood 使用普通 public product seam；manual SQLite、manual Engine complete、hidden prompt/context 或 special HCTL-repo path 使验收失败。
10. **X-DOGFOOD-02** Stable N 治理 isolated N+1；bootstrap/recovery script 不成为 Room/Task/Run/Workflow/Receipt truth。

## L4 · Intent & Collaboration

1. **L4-ROOM-01** Room 不等于 assistant Thread、Harness conversation、provider session、runtime container 或 terminal transcript。
2. **L4-ROOM-02** Room message、Invocation result、Proposal 与 Artifact Diff 不直接改变 Task、Workflow、Run、permission 或 Git。
3. **L4-ROOM-03** Raw message 不自动进入 Git；Memo 必须 proposal → preview → publish，并保留 source provenance。
4. **L4-ROOM-04** Room source event 与 monotonic `room_sequence` 同一 transaction 提交；timestamp、DOM/virtualizer index 和 completion order 不是 identity/order authority。
5. **L4-ROOM-05** Provider transcript、terminal scrollback 和 raw token delta 不自动成为 Room history；只能显式 share normalized summary/ref。
6. **L4-INV-01** Room 没有 global running/cancel/queue；status/control 精确绑定 Invocation/Attempt。
7. **L4-INV-02** Retry 创建新 RoomInvocationRecord 并保留旧 provenance；unknown non-Run invocation 不自动 replay。
8. **L4-MENTION-01** `@participant`/`@role` 在 prompt 前解析 stable identity/authorized candidates；display label 或 free text 不 routing。
9. **L4-RECIPE-01** `/compare`/cross-review 是 Recipe，不是 Participant；durable join/fallback/gate 必须提升 L2 Run。
10. **L4-CONTEXT-01** ContextBundle 由 control 生成并带 digest/provenance；worker 不直接读取 Room DB。
11. **L4-SKILL-01** Skill availability、Project role、Expertise profile 与 actual Invocation selection 是四种事实；Skill 不能扩大 authority 或替代 Gate。
12. **L4-REQUEST-01** Request 是 first-class object；普通 reply/clarification/reaction 不 resolve，只有 authorized typed resolution 可关闭。
13. **L4-REQUEST-02** Room membership 不等于 contributor、required actor、authority holder、facilitator 或 executor capability。
14. **L4-RENDER-01** Message renderer/action 只发 typed intent，不启动 Harness、完成 Request、merge Artifact 或签 Receipt。
15. **L4-BRIDGE-01** External Chat ingress/egress 进入同一 canonical Room，使用 stable principal、dedupe、outbox、fence 和 receipt；edit/delete 不抹掉已引用历史。

## L3 · Commitment & Tracking

1. **L3-TASK-01** HCTL Task ≠ Workflow Node ≠ Conductor task execution ≠ Obligation ≠ Seat ≠ Attempt。
2. **L3-BOARD-01** Board 只投影 HCTL Task；Project、Run、Request、Artifact、worktree 不冒充 Task card。
3. **L3-REV-01** TaskRevision 与 TaskOperationalState 分离；move/rank/assignee 不制造 contract revision。
4. **L3-REV-02** Active Run frozen TaskRevision 不得原地修改、取消或被 provider latest text 覆盖。
5. **L3-SOURCE-01** 每个 field 在 BindingRevision 中同时最多一个 writer authority；client 不能授予/覆盖 authority。
6. **L3-SOURCE-02** Local field 使用 `state_version`；provider field 使用 source revision/digest + outbox/read-back；两者不能混用为 CAS。
7. **L3-SOURCE-03** Provider contract change 只 append Snapshot/Proposal，adopt 后才创建 TaskRevision；pending snapshot 不自动进入 worker Context。
8. **L3-SOURCE-04** Provider Done/Closed/Reopen/Cancelled/Deleted 不等于 HCTL Complete/Reopen/Cancel，也不自动 stop Run。
9. **L3-SOURCE-05** Provider assignee 不自动成为 Participant；external comment 不自动成为 Room message。
10. **L3-SOURCE-06** Provider delete 只 tombstone；Task、Run、Revision、Receipt/history 不删除。
11. **L3-SOURCE-07** Immutable entity identity 与 board placement identity 分离；GitHub Issue 与 ProjectV2Item 不共用 ID。
12. **L3-SYNC-01** SQLite external snapshot/mirror 不是 provider 第二 truth；unknown effect 在 read-back 前不显示 committed 或 blind retry。
13. **L3-LANE-01** source workflow state、HCTL lifecycle 与 derived board lane 是三种事实；Done/Reopen/Cancel 使用独立 typed intent。
14. **L3-LANE-02** Provider terminal/HCTL Open task 不参与普通 rank/reorder；cross provider/ordering scope `before_task_id` 被拒绝。
15. **L3-COMPLETE-01** Run completed、Agent self-report、Git commit 或 provider Closed 不自动产生 semantic completion。
16. **L3-COMPLETE-02** Completion 满足 current TaskRevision acceptance 与 evidence，CompletionReceipt/lifecycle/projection/provider outbox 原子提交；provider read-back 只确认 sync。
17. **L3-RUN-01** Drag to In Progress 不 Start Run；Task 可以无 Run，一个 Run 可以绑定 0..N TaskRevision。

## L2 · Governance & Orchestration

1. **L2-WF-01** Run 固定 WorkflowRevision；running graph 不静默漂移。
2. **L2-WF-02** Canonical executable JSON 由 compiler 构造并 schema/profile/semantic validate；model free-text JSON 不部署。
3. **L2-ENGINE-01** Conductor 拥有 token/node/timer/retry/history；只有 hctl2-control 可 poll/complete/fail/signal HCTL external task。
4. **L2-ENGINE-02** Workbench、CLI、external view 和 Conductor UI 不直接 mutate Engine；Run control 是 HCTL typed command。
5. **L2-EFFECT-01** Engine READY 不产生 effect；control 创建 Obligation/Seat/Attempt 并授权 core/agentd 执行。
6. **L2-OBL-01** 每个 polled HCTL external task execution 恰对应一个 Obligation；Engine system/control task 不对应 Obligation。
7. **L2-SEAT-01** Obligation → 1..N Seat；Seat → 0..N Attempt；candidate fallback 只在同 Seat 新 Attempt。
8. **L2-RETRY-01** Technical fallback、Engine retry、transport retry、semantic rework 和 replan 使用不同 identity/owner；不能互相伪装。
9. **L2-RETRY-02** Semantic reject 不换候选裁判；只有 typed technical failure 进入 fallback policy。
10. **L2-FENCE-01** Old generation/lease/revision 失权；late result 留 history 但不能 vote、complete 或 write。
11. **L2-GATE-01** Same Seat backup Attempt 不增加票数；duplicate/stale/unauthorized vote 不计数。
12. **L2-GATE-02** New subject revision 使旧 required Verdict stale；默认完整 regate；implementation rework 通常不创建 TaskRevision。
13. **L2-RECEIPT-01** Harness progress/complete/result 是 proposal；只有 control/core verification 可签 Verdict/Receipt 或 complete Engine task。
14. **L2-REQUEST-01** Request 只阻塞声明 scope；普通 message 不 signal Engine，authorized typed resolution 才推进。
15. **L2-RESULT-01** Control 先 durable commit domain result/outbox，再 complete Engine；restart 重放 complete 而非重做 effect。

## L1 · Execution & Runtime

1. **L1-MAP-01** Repo、Project、Task、Room、Participant、Obligation、Seat 不拥有 Backend/mux/PTY/terminal identity。
2. **L1-MAP-02** Run → 0..N RuntimeShard；RoomInvocation → 0..1 InvocationRuntime；Attempt/InvocationRuntime → 0..1 TerminalBundle。
3. **L1-PART-01** Participant/role/Seat 不等于 Harness process、tab、pane 或 provider session。
4. **L1-BIND-01** HarnessAdapterBinding 冻结 adapter/capability/session identity/degradation；provider name 不隐含 capability。
5. **L1-OBS-01** Runtime/hook/transcript/screen 都是 observed state；screen/title 只能 advisory，不能完成 Seat/Task。
6. **L1-WORKTREE-01** Worktree 由 ChangeSet/write boundary 懒创建，不永久绑定 Task/Room/Participant；cleanup 不删除 domain history。
7. **L1-WRITE-01** 一个 ChangeSet 同时只有一个 ChangeSetWriteLease holder；fallback/takeover fence old writer。
8. **L1-GIT-01** Agent merge claim 不可信；core 验证 base/HEAD/ancestry/PR/checks/reviews/fence/target head。
9. **L1-ATTACH-01** Exact PTY、native handoff、structured live、semantic resume、replay 是非互斥不同 capability，UI 使用准确 verb。
10. **L1-ATTACH-02** Session resume、structured reattach 与 exact PTY attach 都不能替代 ChangeSet、verification evidence 与 Receipt。
11. **L1-DESC-01** Terminal client 只消费短期 descriptor/opaque connection；renderer 不提交 arbitrary argv/cwd/pane ID 或直接访问 Backend。
12. **L1-LEASE-01** 一个 terminal target 可有多个 observer，但默认最多一个 TerminalInputLease；takeover 原子撤销 old lease。
13. **L1-GEN-01** Descriptor、snapshot、output sequence 与 input 绑定 runtime generation；cross-generation full resync and reauthorization。
14. **L1-CLIENT-01** xterm.js/WezTerm 是 presenter/control clients，不拥有 PTY/process/session truth；detach 不 stop Attempt。
15. **L1-PROJ-01** Execution Chat Projection 绑定单一 Attempt/InvocationRuntime，不是 Room，没有独立 conversation identity。
16. **L1-SECRET-01** Terminal output/input、secret 和 raw trace 不默认进入 Room、Memo、React global store 或 telemetry。
17. **L1-BACKEND-01** 外部 runtime/orchestrator 只能作为 effect provider 或唯一 chosen implementation；不得与 HCTL/Conductor 并行维护 second Run truth。

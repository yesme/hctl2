# 验证、ADR 与未决事项

> Status: Normative validation catalog · Draft v0.7.0<br>
> Parent: [HCTL2 设计规范](../README.md)

## ADR catalog

ADR 记录为什么作出决定、适用边界和替换条件，不是开放问题列表。Phase 1 至少记录：

1. 四层模型与 Repo/Project/Task/Room/Run 公共对象；
2. Project Room 默认导航与 typed L4→L3→L2→L1 seams；
3. Repo-local SQLite、Git、Conductor、control、core、agentd 的事实边界；
4. Conductor external passive engine、HCTL Profile 与 compiler；
5. capability-first Harness catalog、HarnessAdapterBinding 与 observed-state authority；
6. deterministic `@ / $ #`、ProjectRoleBinding、Expertise 与 ContextManifest；
7. HCTL Room event/timeline、concurrent Invocation、RoomProjector；
8. Tiptap Composer、HCTL ReferenceNode 与 ComposerEnvelope；
9. `virtua` timeline 与 assistant-ui renderer-only boundary；
10. React Aria Task Kanban、provider dual-state 与 React Flow read-only Run graph；
11. ChangeSet/worktree/write lease、Receipt/quorum/regate；
12. RuntimeBackend contract 与 Phase 1 default backend；
13. xterm.js embedded client、binary TerminalTransport、WezTerm fallback 与 attach taxonomy；
14. TaskSource field authority、stable identity、snapshot adoption 与 outbox/read-back；
15. Provider lifecycle 与 HCTL semantic completion 分离；
16. External code license/provenance/upstream isolation；
17. B0–B6 self-hosting 与 stable N governing isolated N+1；
18. Selected-reference admission rule：每个 effort 只在最高信息增益层进入正文。

## 开工前的三项 time-boxed validation

这些结果依赖实际 packaging/workload/provider configuration，不能只读源码定案：

### V-SPK-01 · Conductor local distribution

验证 fixed version、SQLite configuration、JRE/JAR size、cold start/RSS、loopback security、upgrade/backup 和 destructive restart。失败时重开 Workflow backend ADR，不自研 Engine。

### V-SPK-02 · RuntimeBackend

Zellij 与 tmux 使用同一 contract bench：exact target、headless lifecycle、process persistence、crash/adopt/reconcile、generation/fence、macOS/Linux、terminal snapshot/resync。Phase 1 只交付胜出的一个。

### V-SPK-03 · First external-authoritative Task Source

Linear 与 GitHub 使用同一 fixture 验证 stable identity、field authority、rank/move、outbox/read-back、rate limit、uncertain create、conflict/tombstone 与 no-Workbench native surface。Phase 1 只要求一方达到完整 read/write；另一方仍需 identity/mapping/snapshot fixture。

## Contract suites

### L4 · Room

- `V-L4-01` Tiptap CJK IME、atomic reference、draft/wire round-trip、paste/drop/undo/focus；
- `V-L4-02` stable ID mention、role binding、membership/authority、label collision；
- `V-L4-03` multiple interleaved streams、independent cancel/retry、epoch/sequence stale isolation；
- `V-L4-04` cursor pagination、prepend/height anchor、unread/follow、around-message、10k items；
- `V-L4-05` Request claim/FIFO/draft/attachment/explicit resolution/double-resolution/supersede；
- `V-L4-06` external bridge idempotency、double dedupe、echo、lease/fence、outbox/DeliveryReceipt/reconnect；
- `V-L4-07` renderer/virtualizer replacement leaves Room truth/order/commands unchanged。

### L3 · Task

- `V-L3-01` Local Task CRUD/revision/rank/lifecycle/acceptance；
- `V-L3-02` keyboard/screen-reader Kanban and explicit non-drag actions；
- `V-L3-03` invalid move/Done/active-Run mutation/stale binding rejection；
- `V-L3-04` provider create/update/reorder/close saga、timeout/duplicate/rate limit/read-back；
- `V-L3-05` contract divergence during active Run、adoption、tombstone/relink；
- `V-L3-06` provider Closed/HCTL unverified and HCTL verified/provider pending dual state；
- `V-L3-07` no-Run Task completes a real Artifact via evidence/Receipt without hidden Run。

### L2 · Governance

- `V-L2-01` compiler/schema/profile/semantic validation and forbidden effect rejection；
- `V-L2-02` start/poll/complete/signal idempotency、unknown outcome/restart reconcile；
- `V-L2-03` technical candidate fallback vs semantic reject separation；
- `V-L2-04` engine retry creates new Obligation and fences old generation；
- `V-L2-05` 2-of-3 quorum、veto、same-seat backup no extra vote、quorum impossible；
- `V-L2-06` reject → new subject revision → all required regate；
- `V-L2-07` Request blocks exact scope and only typed authorized resolution signals Engine；
- `V-L2-08` direct Conductor mutation unavailable/detected as divergence。

### L1 · Runtime

- `V-L1-01` ACP/custom agent/preflight and initial Harness degradation matrix；
- `V-L1-02` process/backend liveness vs structured semantic observation authority；
- `V-L1-03` ChangeSet single writer、retry reuse、fallback fence、late write rejection；
- `V-L1-04` exact PTY / handoff / structured live / resume / replay accurate verbs；
- `V-L1-05` descriptor expiry、generation fence、observe/input/takeover、old lease revoke；
- `V-L1-06` xterm alternate screen/wide glyph/IME/mouse/paste/resize/reconnect；
- `V-L1-07` binary backpressure/bounded buffer/snapshot-resync under load；
- `V-L1-08` Backend crash/adopt/reconcile and terminal client exit without stopping Attempt；
- `V-L1-09` Git merge interruption, HEAD/index/PR reconcile and typed recovery。

### Cross-layer

- `V-X-01` packaged Electron full-window focus/wheel/drop/portal integration；
- `V-X-02` Workbench/control/Conductor/agentd restart at arbitrary slice step without duplicate effect；
- `V-X-03` SQLite migration/backup/restore and projection rebuild；
- `V-X-04` secret redaction、renderer sandbox、typed IPC、terminal trust boundary；
- `V-X-05` provider/Engine/runtime unavailable while frozen Project/Task/Run remains explainable；
- `V-X-06` B2、B4、B5 each execute real HCTL code changes; fixture does not replace dogfood gate。

## Product acceptance

用户应在十秒内回答：Project goal、active/review Tasks、waiting Run/node reason、required human action、current Harness/Skills，以及 result revision/evidence。

产品行为指标：

- Project 默认进入 Room，不需要寻找 Harness tab/session；
- Normal Run green events quiet，action-needed 才新通知；
- no-Run Task path 足够轻；
- invalid command 明确 explain/rollback；
- Harness fallback 后用户仍在同一 Project/Task/Seat continuity；
- Room 不因 runtime crash 丢失；
- concurrent Invocation 可以独立 cancel/retry，Room switch 不串 stream；
- long timeline 不跳 anchor，离底后不抢 scroll；
- terminal 不成为 normal status query；client close 不 stop Attempt；
- ContextBundle 可解释 worker 当时看到什么；
- mixed-source Board 清楚区分 provider lifecycle、HCTL verification、Pending Sync、SourceChanged、Conflict；
- provider unknown write 不显示假成功，Closed-unverified 不进入 HCTL Done；
- Workbench unavailable 时每层按 documented degradation 行为继续或安全暂停。

## 未闭合问题

1. Task/ChangeSet/PR 默认基数与 multi-Task Run integration branch policy；
2. Conductor fixed version、distribution、upgrade 与 backup；
3. Repo-local Room export、cross-clone migration、privacy/retention；
4. Project split/merge 与 Task dependency product expression；
5. Scoped Room maximum active lifecycle/auto-archive policy；
6. Native session import first providers and maintenance budget；
7. Multi-host/remote agentd transport；
8. Windows Phase 2 ConPTY/path/IME/backend；
9. ELK default distribution decision；
10. Multi-user claim、authority chain、notification；
11. Cost/budget hard-limit UX and mid-run exhaustion；
12. Zellij vs tmux bench result；
13. Phase 2 remote/multi-device uses Happy/Paseo-like layer or own relay；
14. Linear vs GitHub as first full external-authoritative adapter；
15. When desktop needs webhook relay; refresh/reconcile freshness and rate budget before then；
16. Which external title/body changes may auto-propose vs require explicit adoption；
17. Multi-user provider ACL/HCTL authority/external direct-edit conflict；
18. Which external Chat bridge, if any, becomes first L4 fallback after Phase 1；
19. Stable First Tree main research pin policy as its unreleased bridge/runtime evolves。

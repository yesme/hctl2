# Phase 1 与分级自举

> Status: Normative scope · Draft v0.7.0<br>
> Parent: [HCTL2 设计规范](../README.md)

## Phase 1 目标

Phase 1 面向单用户、单机、macOS/Linux，架构保留 Windows 原生移植路径。HCTL 自有领域 schema、四层导航和 command admission；成熟 library、选择性移植、sidecar 与 adapter 只实现通用切片，不引入第二事实源。

## 必须交付

### Foundation / Cross-layer

- 单 RepoInstance、多 Project；
- Repo-local SQLite + WAL/FTS5、Git shared artifacts、migration/backup；
- transport-neutral command/query/event seam 与 local Electron IPC/CLI required subset；
- stable IDs、typed command admission、revision/evidence/fence、inbox/outbox/reconcile；
- macOS/Linux packaged Workbench/control/agentd/Conductor lifecycle。

### L4 · Project Room

- Repo Room、Project Room、Scoped Room；
- HCTL-owned Room event/timeline schema、RoomProjector/ProjectionStore；
- `virtua` dynamic timeline、cursor/anchor/unread、concurrent stream isolation；
- Tiptap Semantic Composer 与 deterministic `@ / $ #`；
- 同一 Room 至少两个 independent stream/cancel/retry RoomInvocations；
- ContextManifest/Bundle、standard Agent Skills binding、Memo proposal/publish；
- Request card、Needs Attention 与 Scoped Room；
- assistant-ui 仅为 replaceable scoped message/part/action renderer。

### L3 · Task / Kanban

- Project Overview；
- Task CRUD、TaskRevision、operational state、lifecycle、rank、acceptance；
- React Aria HCTL Task Kanban；
- Local production TaskSourceAdapter；
- Linear 与 GitHub 都完成 identity/mapping/snapshot fixtures；
- Phase 1 exit 前至少一个 external-authoritative adapter 通过完整 read/write/reconcile acceptance；
- explicit refresh/periodic reconcile，不依赖 public webhook relay；
- provider lifecycle / HCTL verification dual state 与 TaskCompletionReceipt。

### L2 · Workflow governance

- typed WorkflowModel → canonical Conductor JSON compiler/validation/profile；
- WorkflowRevision approve、Run Manifest preview、Start/Pause/Cancel typed command；
- Conductor external task → Obligation → Seat → Attempt；
- candidate timeout/429 fallback；
- 2-of-3 review reducer；
- reject → new subject revision → all required regate；
- revision/fence/Receipt、crash/restart reconciliation；
- React Flow read-only Run graph 与 progressive Inspector。

### L1 · Execution runtime

- Codex、Claude Code、OpenCode 的 Harness definitions/bindings/capability snapshots；
- structured protocol 与 PTY/hook degradation paths；
- agentd process/Attempt lifecycle、ChangeSet/worktree/write lease、Git/merge verification；
- Zellij/tmux 同一 contract bench 后选择一个默认 RuntimeBackend；
- `@xterm/xterm` embedded terminal + optional WezTerm external path；
- 至少一个 PTY-backed Harness 支持 exact Attempt attach；
- normalized Execution Chat Projection；
- exact PTY、structured live、handoff、resume、replay 的准确标示与权限。

## 明确不做

- multi-user organization/RBAC、remote multi-host placement；
- Windows release、browser/mobile client、remote relay/E2EE sync；
- full external Chat bridge as Phase 1 product surface；
- arbitrary user-defined Task lifecycle、Project Kanban、Task Room/thread；
- cross-provider move/copy、external comment automatic Room sync、complex bidirectional contract merge；
- 同时把 Linear/GitHub 两套 full bidirectional adapter 作为首个治理切片前置；
- Board virtualization、semantic vector search、auto import all provider histories；
- assistant-ui ThreadRuntime/Cloud/queue 或其他 Chat backend；
- implicit regenerate/branch/edit、presence/reaction/social read receipts；
- 同时交付 Zellij 与 tmux 两个 backends；
- custom terminal emulator/multiplexer；
- donor IDE/remote/mobile full surface；
- generic visual Workflow editor、LLM modify running graph、Conductor HA。

## 四层纵向治理切片

目标：一个 author、三个 logical gater Seats（B/C/D）、B 的 backup candidate、2-of-3 gate 和一次 aggregate reject/rework。

1. Repo 初始化发现至少两个 Harness。
2. Project Room structured mention 精确到指定 Participant/profile。
3. ContextBundle digest/provenance 可查看。
4. Room proposal 提炼为 adopted TaskRevision。
5. Workflow JSON 程序生成并 validate，用户 approve/start。
6. READY external node 创建 Obligation；gate policy 创建 B/C/D Seats。
7. Author Attempt 冻结 adapter/capability/runtime/generation，产生 ChangeSetRevision 1。
8. 三个 gater Seat 并行 review 同一 subject。
9. B 需要开放澄清，创建 Request 并升级 Scoped Room；普通 reply 不 resolve。
10. Authorized user 应用 proposal 后继续。
11. B reject、C accept、D changes_requested；reducer 得出 aggregate changes_requested。
12. Author 产生 ChangeSetRevision 2；TaskRevision 不变，旧 B/C/D Verdict 全 stale。
13. 新 review round 创建完整 B/C/D Seats。
14. B primary 因 429/timeout 技术失败，在同 Seat 切 backup；Conductor task 和 vote identity 不变。
15. Old Attempt late result 被 fence。
16. B backup 与 C accept 达到 2-of-3；D outstanding Attempt 被 fence；backup 不增加额外票。
17. Core Receipt 解锁 merge；L3 独立检查 acceptance 后 Complete Task。
18. 任意步骤间重启 Workbench/control/Conductor/agentd，恢复且无 duplicate effect。
19. Happy path 不 attach terminal；attach 时必须指向 exact Attempt 并准确标 capability。

## External Task Source 纵向切片

该切片与治理切片独立，避免 provider 集成阻塞 Seat/quorum/regate：

1. 连接 Linear 或 GitHub scope，preview stable identity、mapping、authority 与 capability。
2. Import/bind external item，append snapshot，补齐并 adopt TaskRevision。
3. Kanban move 经 durable outbox → provider mutation → read-back → confirmed。
4. Simulate timeout、duplicate delivery、rate limit；ambiguous create 先 query，仍 uncertain 则 Request。
5. Active Run 中 provider contract change 不改变 digest，显示 SourceChanged/PendingAdoption。
6. Provider 先 Closed 时 HCTL 保持 non-terminal + unverified；evidence 满足后仍需 Complete Task。
7. Delete/mapping drift 保留 history 并 Needs Attention。
8. Restart 后 outbox、cursor、snapshot、conflict 可 reconcile。

## Bootstrap 与 dogfooding 原则

HCTL 不等 Phase 1 完整后才用于开发自己。自举按能力而不是“前/后”二分，每一级都通过普通 Workbench/control/agentd seam，并包含真实负路径。能打开自己的 Repo 或生成一次代码不算完整 self-hosting。

| Level | 最小能力 | Truth/cutover | 晋级验收 |
| --- | --- | --- | --- |
| **B0 · substrate** | Domain IDs、migration、command/query/event、`init/start/status/doctor/export`、supervisor | 旧工具仍负责开发 | Clean clone 可启动；restart 不丢 state；script 只管理 process/recovery |
| **B1 · shadow** | Repo/Project、Project Room、Local Task、Room projection、thin Workbench/Tiptap | `HCTL Bootstrap` Project 只 shadow | Room/Task/draft restart recovery；stable refs；明确不是 truth cutover |
| **B2 · assisted self-bootstrap** | B1 + deterministic mention、一个 adapter、ContextBundle、agentd、RoomInvocation、ChangeSet/write lease、diff/test evidence | HCTL Project Room/Task 成为权威入口；旧工具只是 worker/escape | HCTL N 从 Project Room 在隔离 worktree 完成 N+1 的真实非文档代码+tests；越界 write/old lease rejected。**第一次真正自举** |
| **B3 · operational** | B2 + TaskRevision/acceptance、typed Receipt/merge、Request、concurrent Invocations、terminal inspect、cold reconcile | HCTL 接管自身 backlog/collaboration | 连续至少 5 个真实变更，覆盖 core/UI/adapter、failure/restart，无 manual DB/prompt relay |
| **B4 · governed** | B3 + Conductor/compiler、WorkflowRevision/Run、Obligation/Seat/Attempt、independent gater、reject/regate | 正常变更进入 Run | 一个真实 change 走 reject→rework→regate→merge，并 restart component；无 manual Engine complete/bypass Receipt |
| **B5 · full semantic** | B4 + ordered candidates、429 backup、2-of-3、late fence、Scoped Room、full reconcile | 外部 Workbench 只作 worker/emergency | 完整四层治理切片在 HCTL 自身真实 change 上通过，不只是 fixture |
| **B6 · self-release** | B5 + packaging、migration、compatibility、clean-install smoke、rollback | Stable N 治理 isolated N+1 | N 驱动 N+1 build/test/package/upgrade/rollback；被测 process 不覆盖 governing control/DB |

Phase 1 self-hosting maturity target 是 B5；B6 是 release/upgrade gate，但 Phase 1 exit 仍要同时满足全部 product scope 和 external TaskSource slice。

## 第一自举点与实现顺序

第一自举点固定为 B2，不等待 Conductor、quorum、external Task Source、完整 Run Graph 或全部 terminal/runtime 能力：

```text
Project Room + Local Task
  → ContextBundle
  → single Harness RoomInvocation
  → isolated worktree + ChangeSetWriteLease
  → diff/test evidence
  → human review/merge
```

B2 前实现优先级：durable domain/storage seam → thin Project Room/Local Task → Tiptap/timeline → one HarnessAdapter/agentd → worktree/evidence。之后尽量用 HCTL 开发 Conductor/Run、Seat/quorum/regate、external source 和 packaging。

## Cutover 与 escape hatch

- B0 前：Codeg、direct Harness、scripts 是正常工具；
- B1：只 shadow，不允许两个系统同时自称 Room/Task truth；
- B2：对 `HCTL Bootstrap` Project 做显式 cutover；Room/Task 只在 HCTL 写；
- B3：direct Codeg/CLI 属 escape hatch，必须补录 reason/input/result/evidence；
- B4：scripts 不得实现 retry、gate、Task completion、Run mutation 或 Receipt，只保留 install/start/doctor/migrate/recover；
- B5：外部 Workbench 可以保留，但不再成为 project shell 或 fact owner。

防止伪自举：

1. HCTL Repo 作为普通 Repo 接入，不写特殊 `if repo == hctl` path；
2. Stable N 管理 isolated worktree/data profile 的 N+1；
3. 只走 public product seam，test direct internal function 不算 dogfood；
4. 首次 self-bootstrap 必须有真实 code + automated tests + inspectable diff；
5. Manual SQLite、manual Engine complete、hidden prompt/context 都使 gate 失败；
6. 每级至少包含一个 negative path 和 restart recovery；
7. External Task Source 不是 B1–B4 prerequisite；
8. 持续记录 HCTL 内完成比例、escape 原因、untracked semantic operation 和 recovery success。

## 目标 repository layout

```text
README.md
DESIGN_DOC.md
docs/design/                    # 当前权威设计规范
.hctl2/                         # 未来 Git-tracked product configuration
  repo.toml
  projects/
  workflows/
  memory/
  policies/
  skills/
  schemas/
schemas/
examples/
crates/
apps/
```

`<git-common-dir>/hctl2/` 是 operational state，不出现在 repo tree，也不提交 Git。

# 术语表

> Status: Normative terminology index · Draft v0.7.0<br>
> 完整语义以[权威领域模型](../domain-model.md)为准。

## Foundation

| Term | Definition |
| --- | --- |
| Repo | Git repository 的逻辑 identity 与 shared config/Artifact boundary |
| RepoInstance | 一个 local clone/git-common-dir；linked worktree 只拥有 checkout/ChangeSet identity |
| Project | 具名 goal、Room、Task、Artifact、Run 的长期聚合边界 |
| Product-native Core | Repo–Project lifecycle、Project continuity 与 project-driven control；说明用户为何使用 HCTL |
| Architecture-minimum Kernel | 更换 UI/provider/Engine/runtime 后仍保留的 identity、authority、revision/evidence、governance、reconcile |
| Typed seam | 四层之间传递 proposal/command/frozen contract/evidence 的显式协议边界 |

## L4 · Intent & Collaboration

| Term | Definition |
| --- | --- |
| Room | 多 Participant 的持久协作空间；不是 Harness session 或 terminal transcript |
| Repo Room | RepoInstance 的开放研究入口 |
| Project Room | Project 的默认 shaping/continuity surface |
| Scoped Room | 从 Request/incident 派生、活跃期临时但记录持久的商议空间 |
| RoomStore | SQLite 中 Room/message/Invocation/source refs 的 durable store |
| RoomProjector | 把 typed source event 投影为 stable ordered timeline items 的 query component |
| RoomProjectionStore | Workbench 内可重建的 normalized cache，不是 fact owner |
| RoomTimelineItem | 带 stable ID、room sequence、actor、provenance、versioned blocks/actions 的 view item |
| RoomInvocationRecord | 无 Run 的 bounded Harness call；无 automatic retry/fallback/successor |
| ComposerDocument | Tiptap versioned editor draft；不是 RoomMessage 或 prompt |
| ComposerEnvelope | Send 时生成的 data-only text/reference/attachment/command wire object |
| ContextManifest | 本次 invocation 选择了哪些 source、为什么、如何 budget/render 的 provenance |
| ContextBundle | Adapter/Harness 实际获得的可复现 context |
| ExpertiseProfile | Skills、instruction、tool/context policy collection |
| Request | 向指定 human/role 索取 input/authority/decision 的 first-class object |
| Memo | 从 Room/source 明确提炼、preview 并 publish 到 Git 的长期知识 |

## L3 · Commitment & Tracking

| Term | Definition |
| --- | --- |
| Task | 可独立排序、验收和完成的 user commitment |
| TaskRevision | Immutable adopted Task contract |
| TaskOperationalState | Stage/rank/priority/owner/blocker/sync 与 HCTL lifecycle 的高频 state；lane/health 为 projection |
| TaskSourcePolicy | Project 对 Local/linked-readonly/external-authoritative 的 default policy |
| TaskSourceAdapter | Local、Linear、GitHub query/mutation/reconcile implementation seam |
| TaskSourceBinding | `task_id` 与 external entity/board placement/field authority 的 versioned binding |
| TaskSourceSnapshot | Provider raw/normalized append-only observation；contract projection 需 adopt |
| Source Divergence | Provider current contract 与 adopted TaskRevision 不一致 |
| Semantic Completion | Acceptance/evidence/Receipt 校验后的 HCTL completion fact |
| TaskCompletionReceipt | 绑定 TaskRevision、acceptance policy、source snapshot/head、evidence 与 actor 的 completion proof |

## L2 · Governance & Orchestration

| Term | Definition |
| --- | --- |
| WorkflowRevision | Immutable executable control graph definition |
| Workflow Node | Graph mechanical step；不是 Task |
| Run | 执行 frozen WorkflowRevision + Manifest 的 bounded automation instance |
| Run Manifest | Run 的 Task/repo/role/candidate/capability/permission/budget/policy freeze |
| Conductor Task Execution | Engine 中某 node 的 execution instance |
| Obligation | Control 对一个 HCTL external Engine task 欠下的 logical result |
| Seat | Obligation 中 stable logical executor/voter position 与 candidate/lease policy |
| Attempt | 一个 concrete WorkerProfile/Harness 对 Seat 的 execution try |
| Verdict | 针对 immutable subject revision 的 accept/reject/changes-requested semantic result |
| Receipt | Core/control 校验 actor/revision/policy/evidence 后的 formal proof |
| Regate | Subject revision 变化后重新完成全部 required review/gate |
| Candidate fallback | Typed technical failure 后，在同 Seat 下更换 concrete worker 并创建 Attempt |

## L1 · Execution & Runtime

| Term | Definition |
| --- | --- |
| HarnessDefinition | Harness/ACP agent 的 distribution、launch 与 probe definition |
| HarnessInstallation | Host 上 path/version/auth/health instance |
| HarnessCapability | ACP/MCP/resume/Skills/PTY/streaming 等 measured capability |
| HarnessAdapterBinding | Invocation/Attempt frozen ACP/app-server/SDK/PTY/hook binding 与 capability snapshot |
| ParticipantProfile | Room 中可被 `@` 的 stable logical identity |
| WorkerProfile | Reusable Harness/model/mode/permission/environment configuration |
| ProjectRoleBinding | Project role 到 logical Participant 与 candidate WorkerProfiles 的 binding |
| InvocationBinding | Concrete call 的 participant/worker/Harness/Skills/Context/Capability freeze |
| ChangeSet | 一次 authorized write 与 SCM evidence 的 logical boundary |
| ChangeSetWriteLease | ChangeSet/worktree 的 exclusive writer lease |
| RuntimeBackend | Process/PTY/vendor runtime 的 replaceable host implementation seam |
| RuntimeShard | Run 在 host/isolation/generation 上的 physical partition |
| InvocationRuntime | Non-Run RoomInvocation 的 host/isolation/generation boundary |
| TerminalBundle | Attempt 或 InvocationRuntime 的 terminal channels |
| TerminalGateway | Agentd 中解析 descriptor、authorization 与 terminal byte stream 的 trusted gateway |
| AttachCapability | exact PTY、handoff、structured live、semantic resume、replay capability set |
| AttachDescriptor | Agentd signed short-lived target/generation/capability/permission/expiry descriptor |
| TerminalInputLease | Exact terminal target 的 exclusive input/resize lease |
| TerminalClientAdapter | Embedded/external terminal presenter/control client interface |
| TerminalTransport | Sequence/backpressure/input/resize byte connection；不是 runtime truth |
| Execution Chat Projection | 绑定一个 Attempt/InvocationRuntime 的 structured transcript/event/control view |

## Reliability / Sources of truth

| Term | Definition |
| --- | --- |
| Durable inbox | External event/call 的 deduplicated committed intake journal |
| Durable outbox | 与 domain event 原子写入、可 retry/reconcile 的 external effect intent |
| Fence / generation | 使 stale writer/result/connection 失权的 monotonic execution version |
| Projection | 从 canonical facts 重建的 query/UI state |
| Source authority | 某 field/object 当前被授权的唯一 writer/owner |
| Read-back | Mutation 后重新读取 external current state，确认或发现 uncertain/conflict |
| Reconciliation | 比较 desired/canonical 与 external observed state，修复或显式标记 divergence |

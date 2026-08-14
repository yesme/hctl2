# Workbench 与交互集成

> Status: Normative · Draft v0.7.0<br>
> Parent: [HCTL2 设计规范](../README.md)

## Workbench 的位置

HCTL-native Workbench 是四层语义的统一原生客户端，不是 source of truth。建设自己的 Shell 不是为了重写通用 UI，而是因为 Repo/Project/Room/Task/Run 的导航无法无损套入 donor 的 Session、Conversation、Terminal 或 Worktree 主导航。

Workbench 只做 projection、command entry、preview、confirmation 与 local view state。关闭、reload、替换 UI 或通过 CLI 操作，都不能改变领域 identity 和 authority。

## 一级 surface

| Surface | 用户主要回答的问题 | 所属主层 |
| --- | --- | --- |
| Project Room | Project 正在形成什么、谁参与、需要澄清什么？ | L4 |
| Projects Overview | 哪些目标在推进、整体健康如何？ | L3 aggregation |
| Task Kanban | 下一项承诺是什么、哪些待 review/attention？ | L3 |
| Run View | 自动施工走到哪、哪个 obligation 在等待？ | L2 |
| Inspector / Request detail | 当前 revision、evidence、authority 和动作是什么？ | Cross-layer |
| Execution projection / Terminal | 精确 Attempt 内部发生什么，是否需要观察或接管？ | L1 |

进入 Project 默认打开 Project Room，Task/Run/Artifact 是相邻导航。Deep link 可以直接落到 Task/Run/Attempt，但 breadcrumb 必须回到同一 Project；Conversation tab 或 terminal pane 不能成为 Project home。

## 可内外访问的四个操作面

“可操作”表示向事实 owner 提交 typed intent 并等待校验，不表示直接写 database、mirror 或 state machine。

| Layer surface | Workbench | External/fallback | Authority boundary |
| --- | --- | --- | --- |
| L4 Room | Project Room + Composer | future CLI/Chat bridge；Phase 1 不保证完整 bridge | canonical event 在 control/SQLite；external edit/delete 不能抹历史 |
| L3 Task | HCTL Kanban | Linear/GitHub native UI、typed CLI | 每 field 一个 owner；provider Closed ≠ semantic completion |
| L2 Workflow | Run View | CLI/API/status card；Conductor UI read-only | Conductor mechanical truth；control effect authority |
| L1 Runtime | xterm.js/Execution Inspector | WezTerm、typed terminal CLI、inspect/resume/replay | process/PTY truth 在 agentd/Backend；input lease 唯一 |

外部 surface 不是 Phase 1 自动承诺。各层的 guaranteed/future degradation 以 layer 文档为准。

## Phase 1 UI 技术栈

| Area | Choice | Boundary |
| --- | --- | --- |
| Backend/control/agentd | Rust | domain/effect/recovery |
| Desktop shell | Electron + Vite + TypeScript + React 19 | projection/client only |
| Styling/overlay | Tailwind CSS 4 + shadcn Base UI flavor | one focus/overlay primitive family |
| Room timeline | HCTL RoomProjector/ProjectionStore + `virtua` | virtualizer 不拥有 Room truth |
| Message renderer | assistant-ui scoped primitives | one item/allowlisted parts only |
| Composer | Tiptap/ProseMirror | one editor engine; HCTL wire/routing |
| Task Kanban | React Aria Components | collection/DnD/a11y only |
| Run graph | React Flow + Dagre | read-only projection/layout |
| Embedded terminal | `@xterm/xterm` | renderer/client only |
| External terminal | WezTerm | optional attach client |

技术选择受 contract tests 约束；如果 library 不能维持 identity、authority、revision、evidence、focus 或 recovery boundary，应替换实现而不是削弱合同。

## L4 Room integration

### Timeline

HCTL RoomProjector/ProjectionStore 拥有 cursor window、active streams、unread 和 stable provenance；`virtua` 只负责 dynamic viewport。Scroll controller 负责：

- prepend history 时保持 first visible item + pixel offset；
- 离底后不强制 follow，只显示 New activity；
- Room switch 保存 `{anchor_item_id, offset, at_end}`；
- focus/selection item 不因 virtualization 失踪；
- image/diff/tool card late height change 后校正 anchor；
- `aria-live` 只播 milestone/complete short message，不逐 token。

Assistant-ui 只在 scoped provider 后渲染 text/image/file/tool/data part/action。禁用其 ThreadList、cloud/store/queue、global `isRunning`、global cancel、branch/edit 和 adjacent assistant-message join。Actor/Participant/Invocation identity 保留在 HCTL view model。

### Composer

Phase 1 只用 Tiptap/ProseMirror。它必须支持 atomic stable refs、CJK IME guard、cancelable search、keyboard/ARIA、undo/redo、paste/drop、versioned draft JSON 与 separate wire serialization。

HCTL 自己实现 `@ / $ #` extensions、Reference schema、ComposerEnvelope、routing 与 permission。Suggestion popup 经 Base UI/`#overlay-root` adapter，不引入 Radix/body portal 或第二 focus trap。

## L3 Kanban integration

React Aria GridList/useDragAndDrop 提供 pointer/touch/keyboard/screen reader parity。每 lane 一个 GridList、stable key = `task_id`、显式 drag handle，并提供“移动到…”菜单。

Frontend 只显示 command projection：pending external mutation、uncertain、rollback、conflict 和 provider/HCTL dual state 必须可见。MVP 不做 Board virtualization；实测需要后再评估 React Aria Virtualizer。

## L2 Run graph integration

```text
WorkflowRevision immutable topology
  + LayoutCache pure UI
  + RunOverlay dynamic projection
  = Run View
```

Node status update 只 patch overlay，不 re-layout；retry/candidate/votes 进 node Inspector，不把每个 Attempt 画成 graph node。Graph `nodesDraggable=false`、`nodesConnectable=false`，允许 focus/select/open Inspector。Workflow editing 发生在 typed proposal/form/diff，不直接拖动运行图。

## L1 terminal integration

Embedded xterm.js 通过 trusted preload 的 transferable binary MessagePort 连接 agentd TerminalGateway。React 只包 thin lifecycle adapter，不采用 third-party React terminal wrapper；steady output 不走 JSON RPC/global store。

Focus 后 terminal 获取大多数 keyboard event，用户通过 configurable escape chord 把 focus 还给 Workbench。Panel close/reload 只 detach；reopen 重新解析 logical target 和 descriptor。

## Focus、keyboard、portal

输入优先级：

1. IME composition；
2. focused embedded terminal；
3. Modal/Popover/Composer；
4. Board/Graph local surface；
5. Workbench global shortcuts。

HotkeyRouter 检查 `defaultPrevented`、`isComposing`、editable target、terminal focus 和 local scope。Electron `before-input-event` 只处理少数 native commands。

全局只有一个 `#overlay-root` 与 z-index token set。Base UI 负责 Shell overlay；不混用 React Aria Modal、Radix Modal 和 manual body portal。每个 popup ID 唯一并支持 split pane、Drawer 与 scroll container。

## Semantic cards 与 action discipline

Plan、Tool call/result、Permission、Question、Delegation、Progress、Diff、Result/Error、Usage 等都是带 source event ID/trace link 的 semantic projection。Card/action 只能提交 typed command intent：

- Permission card 不直接修改 Harness process；
- Request card 不通过普通 reply resolve；
- Diff card 不直接 merge；
- Result card 不 Complete Task；
- terminal status card 不改变 Attempt state。

## Full-window acceptance

同一 packaged Electron window 同时运行 React Aria Board、Room + `virtua` + scoped message renderer、Tiptap Composer、Base UI Drawer、React Flow graph 和 xterm terminal 时，必须通过：

- single React/Tailwind runtime、strict CSP 与 packaging；
- CJK IME Enter 不误 submit/select，也不错误发送 PTY input；
- `@ / $ #`、undo/redo、paste、draft reload、stable-ID round trip；
- popup 在 drawer/split 中定位和 focus restore；
- Board DnD、Composer drop、Graph pan/zoom、terminal wheel/mouse 不抢事件；
- Room stale async/stream isolation、10k items、prepend/height anchor；
- multi-Invocation independent cancel/retry；
- graph status patch 不 relayout；
- terminal alternate screen、resize、wide glyph、IME、backpressure、reconnect；
- renderer reload/window close 只 detach，fresh descriptor 恢复 target；
- agentd 侧强制 observe/input/takeover、expiry 和 generation fence；
- screen reader 可读 actor、item kind、Task move、Request action 与 Run node。

## 无 Workbench 的共同原则

Workbench unavailable 时，各层分别按自己的合同继续或安全暂停；共同规则只有：

- canonical owner 继续存在，client cache 可以丢弃；
- alternate client 必须使用同一 typed command/query/descriptor seam；
- 没有等价 preview/authority 的 mutation 不降级为 raw API/SQL/shell；
- UI 恢复从 canonical snapshot/event/projection revision 重建；
- graceful degradation 不等于 feature parity。

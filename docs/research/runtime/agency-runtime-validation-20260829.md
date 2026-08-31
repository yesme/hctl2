# Herdr 运行服务验证记录

> 类别：⑥ 外部运行服务与基础设施 · 补充审计<br>
> 状态：2026-08-29 证据快照；Herdr 已确定为 Agent 模块 / Terminal 场景的运行服务<br>
> 基线：HCTL `origin/main@d0e33b2`（研究起点；当前设计 v0.14.1）；Herdr [`v0.8.2 / 9eb5214`](https://github.com/herdrdev/herdr/tree/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c)；tmux `3.7c`

## 设计位置

PR #36—#42 重新划分了约束与职责：

- Herdr 对应 Agent 模块 / Terminal 场景，关系类似 Vikunja 对应 Task / Kanban、Dagu 对应 Run / Workflow、Tuwunel 对应 Project / Chat Room。
- HCTL 通过 Herdr 创建并持有 Harness、PTY 和终端会话；Herdr 自带的 TUI 是这个场景的原生客户端。
- HCTL 自己只保留 Worker Profile 与 Execution Spec 解析、租约和代次、权限、审计、结果验收、恢复等级判断，以及把这些要求翻译成 Herdr API 调用的适配器。
- `hctl2-agentd + tmux` 不再作为并行的正式方案。旧实现中的 HCTL 治理代码可以迁到 control/tool，tmux 会话管理、输出转发和终端重放代码不再继续扩写。

过去对 tmux、Termio、tty7、cmux、Pilotty 等项目的验证仍然有价值：它们提供资源占用、输出保留、慢客户端、退出和恢复行为的对照，但不再代表当前产品还要同时维护这些实现。更广的源码、发行物、RSS 与测试记录见 [运行服务候选全量复审](../agentd-runtime-candidates-20260829.md)。

## Herdr 上线前的验证清单

Herdr 不需要实现 HCTL 的领域治理，但它提供的功能必须如实验证。不支持的功能可以明确标为不支持，不能用相近能力冒充。

| 能力 | 验证方法 | 暂不支持时的处理 |
| --- | --- | --- |
| 派工与核验 | 数组 argv、cwd、显式 env、尺寸与冻结 spec digest 能逐项核对；返回实际进程、Harness/版本与已施加加固项 | 缺项则不激活；不能只信功能声明 |
| 精确身份 | runtime ID 与物理进程/PTY/session 一一对应；替代后旧引用不能命中新现场 | 无法证明同一进程时只报 semantic resume、replay 或丢失 |
| 观察完整性 | 原始输出或规范化事件带来源 sequence；重连、缓冲溢出、慢观察者必须无缝续接、重新锚定或显式 gap | 无 sequence/gap 的流只能作带外诊断，不能冒充完整 trace |
| 多观察者 | 慢观察者不阻塞 Harness 和快观察者；各自队列有界 | 未声明扇出时只经 control 网关观察 |
| 单输入者 | 合规输入、Agency 原生客户端和 API/CLI 直写都经过同一个 writer gate；接管使旧 writer 的迟到输入失效 | 无栅栏通道的输入全部按带外输入处理 |
| 带外输入入账 | 原生客户端每次写输入至少产生不可混淆的 source、target、observed_at 事件；无回显输入也能被发现 | 发现不了带外输入时，不能同时开放原生写入口并宣称 governed execution 未受污染 |
| 栅栏回显 | 请求携带并回显 runtime/site/Agency owner generation 与 input lease；执行点拒绝旧值 | 不支持则整通道为低信任，不能承载要求栅栏的动作 |
| 退出与停止 | 区分 Harness 退出、pane/shell 退出和 control 主动取消；回读退出码；HUP/TERM/KILL 后回读残留进程 | 无退出码或残留证明时只能上报观测不完整/停止未收敛 |
| 恢复 | 分别测试 client 断开、Agency 正常重启、crash、受控 live handoff 与 Harness 原生 resume | 只按 exact attach / semantic resume / replay / 丢失四级报告 |
| footprint | 同一目标、同一终端数、同一约 200 KB/terminal 输出量；同时报告 server-only 与子进程是否计入 | 不同拓扑或保留策略不得直接比较 |

## Herdr v0.8.2 源码核对

稳定版以 tag peeled commit `9eb5214` 为准；另对照了 2026-08-29 upstream HEAD `c2637dc`。与下表相关的 request envelope、event ring、pane API 与退出 event 没有出现能关闭这些缺口的新约束，因此不拿未发布 HEAD 冒充稳定版能力。

### 已经可以直接复用的能力

- 官方提供 macOS/Linux × arm64/x86_64 单二进制；Apache-2.0，API 有 protocol 版本协商与 JSON schema。
- server 持有 PTY、parser、检测任务与稳定 terminal ID；client 退出不杀 pane。正常 server 存活时可以 exact attach。
- terminal observe 与 writable control 分开；多个 observer 可并存，controller takeover 会断开旧 controller。每个客户端的 render slot 容量为一，慢客户端只推迟自己的最新完整 frame，不阻塞 PTY。
- workspace/layout API 能传 cwd、env 与数组 command；`agent.start` 能按已知 integration 启动多种 Harness 并回报检测状态。
- pane shutdown 会枚举 session process，依次 HUP、TERM、KILL，每阶段等待 250 ms；普通 session snapshot、Harness 原生 session resume 与实验性 live handoff 分开表述。
- 状态来源仲裁、过期 hook sequence 与“`agent prompt --wait` 不等于某一轮完成”的边界，适合作为低层观测来源。

### 当前需要适配或补充的能力

| 能力 | v0.8.2 源码事实 | 处理方式 |
| --- | --- | --- |
| 精确派工 | `agent.start` 先找到 pane shell，再把拼好的命令作为终端输入发送；响应只返回期望 argv。layout/pane 路径可直接启动数组 command，但没有冻结 spec、实际 env/版本/加固项的交付回执 | adapter 可组合现有 API，但完整核验需要新增 launch receipt；不能把 `AgentStarted` 当作已验证交付 |
| 栅栏回显 | API `Request` 只有 `id + method/params`；pane input/close 等方法不携带 generation/lease，也不保存绑定 fence | 当前只能低信任接入；若要承载 governed input，给请求、runtime binding 与响应增加 opaque fence envelope，并在执行点拒绝旧值 |
| 统一 writer gate | direct attach/controller 的 owner 是进程内 `HashMap<terminal_id, client_id>`；API `pane.send_text/send_keys/send_input` 直接写 `PaneRuntime`，不检查该 owner | native client 与 API 可以物理交错输入；必须把所有写路径并入同一 writer claim，或 governed runtime 期间禁用 native 写入口 |
| 带外输入 provenance | native client 输入处理只写 debug 日志中的 `client_id` 与 payload 长度；API/event stream 没有“谁向哪个 pane 输入”的事件 | control 无法可靠入账，尤其是 no-echo/安全输入；需要可订阅的 input provenance 事件。未补前不能以原生客户端写入作为正式互操作能力 |
| 完整观察与 gap | `EventHub` 只保留内存中最近 512 个事件；内部 sequence 不进入 `EventEnvelope`，`events_after` 在溢出后静默返回仍保留的后缀。直接 observer 收到的是可丢弃并重新锚定的 render frame，不是带持久 cursor 的 raw stream | 可作 UI 和诊断观察；完整 trace 需要 Herdr 增加 output sequence、最早可读位置、gap 事件和重连参数。结构化 Harness 事件可由 Harness adapter 另行保存 |
| 退出事实 | `PaneExited` 和 `PaneInfo` 没有 exit code；`agent.start` 启动的 Harness 通常退出回到 pane shell，pane 本身仍活着 | 结构化 Harness adapter 可另交终局结果；通用 PTY 路径仍需进程 incarnation 与退出码回读 |
| 停止结果 | 内部会 HUP/TERM/KILL 并在仍有残留时记 warning，但 `pane.close` 仍返回成功，API 不返回残留 PID 或 stop receipt | 返回“已停止 / 仍有残留进程”及其 PID，供 control 准确记录 |
| 重启恢复 | 普通 server stop/restart 会丢失原进程，只恢复 workspace/tab/pane/cwd/layout/focus；history 是 replay，Harness session restore 是 semantic resume。只有实验性、主动发起的 Unix live handoff 尝试保留 PTY/进程 | 分级本身正确；不能把普通“persistent session”宣传成 exact attach。live handoff 可独立验证，失败必须安全降级 |
| 持久治理 | controller owner、事件 ring 和 runtime registry 都在内存；没有 TTL、lease generation、持久确认 cursor | 这不是要求 Herdr拥有治理权威；但要由 control 补齐，且物理执行点不支持 fence 时只能低信任 |

这里的缺口不是“Herdr 多做了插件、worktree、TUI，所以不能用”。额外功能的体积与 RSS 尚可接受。writer bypass、原生客户端输入无记录、无 gap/sequence、无 fence echo 和退出/停止回读不足，会限制 HCTL 现在能对外承诺的功能。这些问题可以由 HCTL 适配、在 Herdr 上游修改，或暂时明确为不支持；不需要因此再做一套终端服务。

## footprint 复现

### Apple Silicon macOS

2026-08-29 使用官方 `herdr-macos-aarch64` v0.8.2；`XDG_CONFIG_HOME`、`XDG_STATE_HOME` 与 `HERDR_SOCKET_PATH` 指向 `mktemp` 目录。每档对 server RSS 连续采样十次、间隔 200 ms；不计被托管 shell。有效探针目录已 recoverably 移到 `~/.Trash/hctl2-herdr-macos-load.WgSs4z`。

| 负载 | server RSS 平均 | 峰值 | 备注 |
| --- | ---: | ---: | --- |
| 空 headless server | 17.97 MiB | 18.39 MiB | 0 workspace |
| 10 workspace / 10 idle shell | 26.20 MiB | 26.28 MiB | shell 子进程另占 57.52 MiB，不进入与 tmux server-only 的对照 |
| 每 pane `/usr/bin/seq 1 35000`（约 199 KB 输出） | 119.98 MiB | 121.20 MiB | 默认每 pane 10 MB scrollback 上限；说明小量重输出已有明显解析后表示放大 |

二进制为 18,969,952 bytes（18.97 MB / 18.09 MiB）。这一结果与 PR #36 的 Linux x86_64 探针方向一致：13.7 → 19.9 → 112 MiB；同 Linux 负载下 tmux 3.7c 为 1.95 MiB（共享 server）、19.3 MiB（10 个隔离 server），全保留输出后 21.9 MiB。Herdr 的空闲开销在十个 Harness 场景可接受，重输出约为 tmux 的五倍是需要进入容量预算和长期压力测试的真实差异。

2026-08-30 再次核对 v0.8.2 的 GitHub Release API：HCTL 当前三个发行目标对应的官方原始二进制均存在，macOS arm64/x86_64 分别为 18,969,952 / 20,551,504 bytes，Linux x86_64 为 22,733,040 bytes；Release 还提供 Windows x86_64 zip。Windows 仍不在第一阶段支持范围，原因是 HCTL 完整包和生命周期尚未进入 Windows 验证矩阵，而不是 Herdr 缺少发行物。

### tmux 对照数据

tmux 3.7c 的三目标 P0 已验证 owner-only socket、唯一可写 control client、输入/resize、稳定 session/window/pane ID 与 PID、断开重连、退出码、残留清理、无人 attach 时的 DSR/DA/DECRQM 应答，以及快慢观察者隔离；详见 [tmux P0 接口验证](../tmux-runtime.md#2026-08-29-p0-接口验证)。这些数据作为 Herdr 的行为和资源占用对照保留，不表示产品还要分发或维护 tmux。

## 当前结论

- **Herdr 是已经选定的运行服务，不再是与 tmux 并列的候选。**
- 上表的缺口不导出一套新的 HCTL 终端服务。能由适配代码完成的留在 HCTL；Herdr 暂不支持的功能明确标为不支持；确实需要运行服务配合的功能优先推动 Herdr 上游补充。
- 在对外声称“所有输入都受 HCTL 租约控制”之前，必须先统一输入权并记录原生客户端输入。否则可以先使用 Herdr，但要明确说明原生写入不受 HCTL 输入租约约束。output sequence/gap、fence echo、exit/stop receipt 按产品需求逐项完成，文档不能提前声称已经具备。
- tmux、Termio、tty7、cmux 和 Pilotty 的记录用于回归测试设计、容量预算及 Herdr 将来不够用时的重新评估，不构成并行实现计划。

## 源码定位

- Herdr API envelope 与方法表：[`src/api/schema.rs`](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/src/api/schema.rs)
- 512-event 内存 ring：[`src/api/event_hub.rs`](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/src/api/event_hub.rs)
- controller owner 与 native input：[`src/server/headless.rs`](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/src/server/headless.rs)
- API 直写 pane：[`src/app/api/panes.rs`](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/src/app/api/panes.rs)
- `agent.start` 的 shell-input 启动路径：[`src/app/agents.rs`](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/src/app/agents.rs)
- PTY/runtime 与 stop policy：[`src/pane.rs`](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/src/pane.rs)
- 恢复等级说明：[`session-state.mdx`](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/docs/next/website/src/content/docs/session-state.mdx)

## 2026-08-30 原生客户端定位复核

v0.15.0 改变的是产品解释，不是上面的源码事实。旧结论把“不能证明所有输入经过 HCTL lease”推得过远，容易把 Herdr TUI 只当诊断工具；真实用户路径里，人在精确终端继续对 Harness 输入，本来就是 Terminal 场景的正常交互。Workbench 的 xterm 若直连 Herdr transport，也不会因为装在 HCTL 窗口里就自动获得更强保证。

因此 Execution Spec 分两种要求：

- `managed_single_writer`：需要证明所有输入经过 descriptor/generation/Terminal Input Lease。v0.8.2 必须关闭原生 controller 写入，只开放 Herdr adapter 可校验的路径；writer bypass、无 fence echo 和无 input event 仍是阻断事实。
- `native_interactive_allowed`：允许 Workbench 直连、Herdr TUI 等原生客户端向已映射的精确 terminal 输入。输入立即影响运行时，属于有效的用户运行时输入；但逐次 actor、generation、单写者和完整 replay 无法证明，必须如实标注。终端文字不因此成为 Result Proposal、Task 完成或 Run 裁决，后续结构化结果和 Git/SCM/Test evidence 仍各自按原约束验收。

这不是为 Herdr 缺口找借口，也不要求 HCTL 自建 writer proxy：一项执行选择自己需要的保证，当前 provider 做不到的强保证就关闭对应入口；普通交互所需的较弱保证则可以直接复用 Herdr 已有 TUI/transport。将来 Herdr 增加统一 writer gate、输入 provenance 事件和 fence echo 后，两种路径可以在新 binding revision 下使用同一物理入口。

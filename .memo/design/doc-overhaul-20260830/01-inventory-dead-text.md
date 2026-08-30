# S1 死文普查

> 状态：待拍板 · S1 产物<br>
> 基线：main @ 37805fa（草案 v0.15.0）<br>
> 去向：S1 末拍板点 6 的删/转/留清单；不进合同<br>
> 施工员：Grok · 只读普查，不改正文<br>
> 判据：[施工图 §7.1](./README.md#71-死文四判据命中任一)：1 被转向取代 / 2 无权威对应 / 3 重复权威 / 4 无入口且非历史<br>
> 处置：删 / 转历史 / 改写为当前事实。Git 历史即 archive。无依据写「待核」，不猜。

对照 `docs/design/references/decision-history.md` 33 节与现行文档。从施工图 §8.1 起，并补扫现在时退场机制、断链、「第一阶段不设 X」负述。`decision-history` 已取代章节只标折叠，不建议删。

判据 3 的跨域复述（§8.2 三组）**不建议删合同定义**，只标重复、供 S2 收束。误删会丢承重规则，见文末「最危险三条」。

## A. §8.1 种子（逐条核实）

| # | 文件:行号 | 原句或段落摘要 | 判据 | 处置 | 核销依据 |
| --- | --- | --- | --- | --- | --- |
| A1 | `docs/design/references/implementation-evidence.md`:1–3 | 3 行转发 stub，指向 `docs/research/` | 4 | 删 | 设计地图已改链到 `docs/research/README.md`（`docs/design/README.md`:98）；`docs/design/**` 内零引用本 stub。research 仍有旧路径，见 C3 |
| A2 | `docs/usage.md`:212 | 「Herdr 由该服务命令管理，不安装成 HCTL2 自建的 `hctl2-agentd`」 | 1 | 改写为当前事实 | §29 退场 agentd；负述已撤销机制。现行句只需「Herdr 由 `hctl2-services` 管理」 |
| A3 | `docs/design/delivery.md`:270 | 「Tuwunel 因上游没有 Darwin 二进制而从锁定源码原生构建」 | 1 | 改写为当前事实 | 同文件 `:262`「HCTL2 在自己的 GitHub Release 托管按 SHA-256 锁定的 macOS 包」；research 总表 2026-08-30 换用托管制品。以后者为准 |
| A4 | `docs/design/delivery.md`:270 | 「macOS 最低基线为 15」 | 1 | 改写为当前事实 | 出处 §19 tmux 官方 Darwin 二进制；tmux 已由 §29 退场。是否仍为 15 **待核** Tuwunel/Herdr/Tauri 随包 Mach-O（施工图 §8.1；停车位未单列此项，属事实修正） |
| A5 | `docs/design/spec/system.md`:109 | 「第一阶段不设额外的用户在场证明」 | 1 | 改写为当前事实 | §22 撤销在场证明。`spec/README.md`:121 清扫表同行是核销记录，合法。同句「两类 actor 来源」是承重规则，**不可整段删** |
| A6 | `docs/design/agent.md`:53 | 角色表「场景客端：CLI / WezTerm」 | 1 | 改写为当前事实 | §31 一等原生 Terminal 客户端是 Herdr TUI。WezTerm 未在转向中除名，保留与否则 **待核**（施工图：普查裁决）；若留须降为「可选外部终端」，勿与 CLI 并列成一等客端 |
| A7 | `docs/design/agent.md`:53 | 「场景客端」 | — | 改写为当前事实 | 笔误。同表上一行写「场景客户端」。无转向编号 |
| A8 | `docs/design/delivery.md`:16 | Agent 适配列「Herdr 官方 TUI 是原生 Terminal 客户端，WezTerm 可选」 | 1 | 改写为当前事实 | 与 A6 同条；「可选」已比 agent.md 角色表诚实，是否删除 WezTerm **待核** |
| A9 | `docs/design/participant.md`:3 | 头部「草案 v0.14.1」 | 1 | 改写为当前事实 | 现行基线 v0.15.0（§31）；横切正文未随 stamp |
| A10 | `docs/design/context.md`:3 | 头部「草案 v0.14.1」 | 1 | 改写为当前事实 | 同 A9 |
| A11 | `docs/design/vision.md`:4 | 日期 2026-08-29；状态已是 v0.15.0 | — | 改写为当前事实 | 种子指出与设计地图 `:4`（08-30）不一致。无转向编号；是否必须对齐 **待核** |
| A12 | `docs/design/spec/README.md`:4 | 日期 2026-08-29；状态已是 v0.15.0 | — | 改写为当前事实 | 同 A11 |
| A13 | `docs/design/references/decision-history.md`:50–54 | §6 标题已写「后由 §18 取代实现选型」；正文仍展开当时 Conductor 边界 | 1 | 转历史 | §18 取代 Conductor 选型。折叠为「当时为何、被谁取代」，不删。正文 `:54`「在 §19 换成 Dagu」为笔误（Dagu 是 §18，§19 是 tmux），折叠时改写 |
| A14 | `docs/design/references/decision-history.md`:111–113 | §13 P0–P6、conductor-oss、Zellij、延至 P4 | 1 | 转历史 | §15 收成 P0–P3；§18/§19/§29 改选型；现行 `delivery.md`:284 远端后端在 P2 之后按需。折叠不删 |
| A15 | `docs/design/references/decision-history.md`:147–155 | §18 Conductor→Dagu，并留下 B4 完成 API 代次隔离阻断 | 1 | 转历史 | §23 撤 B4 阻断、代次不在 Dagu。选型本身仍有效，折叠「阻断项已被 §23 取代」段，不删整节 |
| A16 | `docs/design/references/decision-history.md`:157–165 | §19 第一阶段运行时后端改为 tmux / agentd 持有 socket | 1 | 转历史 | §29 取消 `hctl2-agentd + tmux`。折叠不删。macOS 15 论证见 A4 |
| A17 | `docs/design/references/decision-history.md`:219–229 | §27 运行时 provider；agentd 为治理桥；tmux 内置最简 provider；herdr 为第二候选；落点「delivery 第 6 项」 | 1 | 转历史 | §28 定名 Agency；§29 Herdr 直接实现。折叠不删。`:223` 两处断链见 C1–C2 |
| A18 | `docs/design/references/decision-history.md`:231–239 | §28 标题已写「已由 §29 取代」；仍写建设 `hctl2-agency` | 1 | 转历史 | §29 取消 `hctl2-agency`。折叠不删 |
| A19 | `docs/design/spec/README.md`:71–87 | 「v0.9.1 归并对照」整表 | 3 | 转历史 | 核销记录不是当前合同。去向由拍板点 9（并入来时路折叠 / 留总则末标核销记录） |
| A20 | `docs/design/spec/README.md`:88–95 | 「v0.10.3 清扫」整表 | 3 | 转历史 | 同 A19；台账已链到本表（decision-history §32） |
| A21 | `docs/design/spec/README.md`:97–105 | 「v0.11.1 词形收敛」整表 | 3 | 转历史 | 同 A19 |
| A22 | `docs/design/spec/README.md`:107–115 | 「v0.12.2 清扫」整表 | 3 | 转历史 | 同 A19 |
| A23 | `docs/design/spec/README.md`:117–127 | 「v0.13.0 收窄」整表 | 3 | 转历史 | 同 A19。表内「用户在场证明 / OS 沙箱入场券」作为核销记录合法，与 A5 清扫表同行同类 |
| A24 | `docs/research/README.md`:3 | 状态行四个重组日期括号（08-24/26/27/29/30） | 4 | 改写为当前事实 | 种子：历史括号应收成一句。无转向编号。索引职责仍在，非删文件 |
| A25 | `docs/research/tmux-runtime.md`:7–8 | 文首已写「Herdr 已取代 tmux…下文保留对照」 | 1 | 转历史 | §29。**已有标注，不删文件**（施工图：证据不删） |
| A26 | `docs/research/agentd-runtime-candidates-20260829.md`:7–11 | 文首已写 v0.14.1 重解释，旧结论「继续 tmux」已取代 | 1 | 转历史 | §29。**已有标注，不删文件** |
| A27 | `docs/research/workbench-shell.md`:7 | 文首已写 v0.14.2 重解释：主选 Tauri 2，Electron 降为安全网 | 1 | 转历史 | §30。文件保留。但「当前决定」正文仍以现在时写 Electron，见 C5 |
| A28 | `.memo/design/hctl2-agentd-prd-20260826.md`:3–9 | 状态仍「讨论中」；去向仍写 agentd 交付合同；追记承认退场但正文仍以现在时定义 agentd | 1 | 转历史 | §28/§29。标头改为已废弃/待按 Herdr·control·tool 三类重归；不在本轮重写 83 条 AGD |
| A29 | `.memo/notes/doc-cleanup-backlog-20260825.md`:15 | 「agentd-only terminal」仍待拍板 | 1 | 转历史 | §29 已取消 agentd-only。并入禁令盘点时标过时。语义其余 11 条进停车位第 3 项，本普查不逐条改合同 |
| A30 | `.memo/notes/doc-cleanup-backlog-20260825.md`:27 | 「打包与 tmux 拓扑写死」仍待拍板 | 1 | 转历史 | §29 tmux 退场。同 A29 |
| A31 | `.memo/README.md`:43–48 | 待拍板表仍列 control-storage、context-feeding §8、agentd-prd、grok-ci-cadence、doc-cleanup-backlog、本施工图 | — | 改写为当前事实 | 种子：大修时核销或归档。agentd-prd 见 A28。其余是否仍待拍板 **待核**（非决策史转向）；本轮只建议刷新表，不猜删行 |

## B. §8.2 判据 3 复述（供 S2 收束，禁止当死文整段删）

权威定义必须留在合同层一处；其余改一句引用。下面「处置」是收束，不是删除该规则。

| # | 文件:行号 | 原句或段落摘要 | 判据 | 处置 | 核销依据 |
| --- | --- | --- | --- | --- | --- |
| B1 | `docs/design/spec/project.md` 等多处 | Chat 房间不开 E2EE 的前置与降级，设计/合同/连接/架构/CT 约七处完整复述 | 3 | 改写为当前事实 | §24 权威在 spec/project.md「Room 与消息」。其余一句引用。施工图 §8.2 |
| B2 | `docs/design/spec/agent.md` 写入合同等 | 三条底线在 agent 设计、agent 合同、system 多节、§22、CT-AGENT/B2 复述 | 3 | 改写为当前事实 | §22 权威在 spec/agent.md 写入合同。system.md 引用；设计层一句人话。**不可删 spec/agent 定义** |
| B3 | `docs/design/spec/agent.md` 运行时与观测等 | Herdr v0.8.2 能力限制在 agent 设计/合同、system、connections、delivery P0、CT-AGENT 约七处 | 3 | 改写为当前事实 | §29/§31：合同留能力条件句；v0.8.2 缺项清单下沉 delivery P0 与 binding（拍板点 11）。**不可把合同能力句当成 P0 脚注删掉** |

## C. 种子外补扫

| # | 文件:行号 | 原句或段落摘要 | 判据 | 处置 | 核销依据 |
| --- | --- | --- | --- | --- | --- |
| C1 | `docs/design/architecture.md`:21 | 「远程连接的认证与传输见 spec/system.md 的端点约束与未决问题」 | 2 | 改写为当前事实 | `spec/system.md` 无「端点约束」「未决问题」节（现有节：组件/端口/动作/命令/副作用/存储/单写者/启动与恢复/安全边界）。未决清单在 `delivery.md`:280–290 |
| C2 | `docs/design/participant.md`:45 | 链到 `agent.md#agency` | 2 | 改写为当前事实 | 现行标题是「Agency 与 Herdr」，锚点 `#agency-与-herdr`（`agent.md`:57）。`#agency` 不存在 |
| C3 | `docs/design/references/decision-history.md`:223 | 落点 `agent.md#agency`；「交付文档第 6 项」 | 2 | 改写为当前事实 | 同 C2；现行 P0 只有 5 项（`delivery.md`:260–284），Herdr 是第 2 项。属折叠 §27 时一并改指针，不单删 |
| C4 | `docs/design/references/decision-history.md`:155 | 链到 `docs/research/README.md#执行面已选依赖的运维与-footprint` | 2 | 改写为当前事实 | 现行标题「已选外部服务的运维与资源占用」（`docs/research/README.md`:239）。delivery.md:276 已用正确锚点 |
| C5 | `docs/research/workbench-shell.md`:12–14 | 「### 当前决定」「第一阶段保持 Electron + React 19，不改用 Tauri 2」 | 1 | 改写为当前事实 | §30；与同文件 `:7` 文首重解释直接冲突。文件不删，此小节须改为历史快照口吻或改写为 Tauri 2 主选 |
| C6 | `docs/research/workbench-shell-reopen-20260826/README.md`:4 | 「现行决定见 implementation-evidence.md#e-workbench-shell (Electron + React 19…)」 | 1 | 改写为当前事实 | §30；证据入口已迁 `docs/research/workbench-shell.md#e-workbench-shell`。implementation-evidence stub 无该锚点 |
| C7 | `docs/research/workbench-shell-reopen-20260826/workbench-shell-reopen-20260826-a6-local-probes.md`:3 | Electron 基线取自 `implementation-evidence.md#e-workbench-shell` | 1 | 改写为当前事实 | 同 C6，改链到 research 条目 |
| C8 | `docs/research/workbench-shell-reopen-20260826/workbench-shell-reopen-20260826-a3-linux-web-routes.md`:87 | Helio「documented separately in implementation-evidence.md E-HELIO」 | 4 | 改写为当前事实 | stub 无 E-HELIO；Helio 条目应在 `docs/research/workbench/` **待核**精确文件名 |
| C9 | `docs/research/methodology-landscape-20260824.md`:58 | 工作方式仍写「证据纪律（implementation-evidence 全部钉 commit）」 | 1 | 改写为当前事实 | 证据库已迁 `docs/research/`。属口吻过时，非删该调研 |
| C10 | `docs/design/spec/README.md`:121 | 清扫表「用户在场证明 / OS 沙箱入场券」 | 3 | 转历史 | 与 A23 同表。核销记录合法；若拍板点 9 整表外移则随表走，不要单独当现行禁令 |

未列入：现行规范层（`docs/design/**` 除 decision-history）已无现在时 `agentd` / `hctl2-agency` / `Zellij` 作为第一阶段实现。Electron 作为「安全网」出现在 `delivery.md`:276 与 `spec/system.md`:210，与 §30 一致，**不是死文**。`delivery.md`:287 `#原生会话导入` 锚点存在（`agent.md`:80）。

## D. 计数

| | 条数 |
| --- | --- |
| 合计（A+B+C，A13–A18 按节计、不含 DH 逐句） | 44 |
| 判据 1 被转向取代 | 24 |
| 判据 2 无权威对应 / 断链 | 4 |
| 判据 3 重复权威 | 9 |
| 判据 4 无入口且非历史 | 3 |
| 未归入四判据（戳/笔误/待拍板表） | 4 |
| 其中含「待核」 | 7 |

| 处置 | 条数 |
| --- | --- |
| 删 | 1（A1 stub） |
| 转历史（折叠/标核销，不删文件） | 18 |
| 改写为当前事实 | 25 |

§8.1 种子均已落行。decision-history §6/§13/§18/§19/§27/§28 建议折叠不删。

## E. 最危险的三条（若按「死文」整段删会丢承重规则）

1. **A5 `spec/system.md`:109。** 「不设在场证明」是负述，该删的是这半句。同段规定治理命令只有两类 actor（可映射 human 的动作 + task-bound Run reducer）。这是 §22 入口规则。整段当死文删，会把「工具不是人」的合同入口一起拆掉。

2. **B2 三条底线的合同定义（`spec/agent.md` 写入合同）。** 判据 3 会引诱人「删重复、留一处人话」。权威必须留在合同层。只留 vision/agent 设计句或只留 CT 负例，实现者会把底线读成原则而不是写入合同。

3. **B3 / A3–A4 打包与 Herdr 能力。** 把 `delivery.md`:268–272 整节当「tmux 时代残留」删，会丢掉仍生效的「必须原生、不进 Docker、按 target 分发」。把 spec/agent 里 Herdr「无则降级」的能力条件句当成与 P0 重复而删，会把 §29「按实测降级、禁止另写终端服务」从合同里拿掉，只剩 delivery 探针散文。

这三条的正确处置都是 **改写/收束引用，不是删规则**。

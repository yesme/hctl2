# HCTL2 draft v0.12.3 开工前设计评审

> 日期：2026-08-25<br>
> 对象：`origin/main` @ `79da0e3`（draft v0.12.3）<br>
> 范围：README、设计层、四模块合同、连接/系统合同、delivery、decision history 与当前实现证据<br>
> 状态：Informative · 评审记录，不直接修改规范<br>
> 说明：本文同时保留初审发现、复核后的降级判断和所有者逐项裁决，避免把一次评审意见误写成新产品合同。

## 一、结论

架构主线已经足够稳定，可以开始 coding。

设计最扎实的部分是：四模块事实归属、metadata/content/artifact 三分、不可变 revision/digest、唯一治理账本、类型化命令、外部副作用先持久化再 readback，以及 Task/Run/Agent 之间“不让进程退出、模型自述或外部关闭冒充完成”的边界。这些没有发现需要推倒重来的矛盾。

初审曾给出较保守的判断：“先做 P0、脚手架和 P1，暂缓 B1/P2 核心 schema/reducer”。经所有者追问和第二轮复核，这个判断需要修正：**第 4–7 项大多是实现选择或 delivery bookkeeping，不是 design blocker；第 1–2 项反而暴露了当前文档自身的过度设计；第 3 项已经由所有者拍板。**

因此最终建议是：立即建立工程脚手架、跑 P0、实现 P1/B0；进入相应切片时再做最小合同清理，不要先实现本文点出的过强机制。

## 二、逐项评审与裁决

### R1 · Harness、Git common-dir 与 target refs

初审质疑：合同要求 Harness 看不到 target ref/common-dir，而普通 linked worktree 天然依赖 common-dir，这会迫使实现引入独立 clone 或 Git broker。

精确来源：

- [`docs/design/spec/agent.md`](../docs/design/spec/agent.md) 第 34 行：“不得读取目标 ref/其他 ChangeSet”；
- [`docs/design/delivery.md`](../docs/design/delivery.md) 第 169 行：把 `target Git ref/common-dir` 访问列为必须失败的负例；
- [`docs/design/spec/system.md`](../docs/design/spec/system.md) 第 101、192 行把 OS sandbox、credential gateway 和拒绝启动继续强化。

来源追溯：这些条款由 `454d800608b4d9a1f4ecff05162675c5645403e2`（`docs: audit and close v0.12 architecture gaps`）引入。Git author/committer 为 Yesme，提交 trailer 为 `GPT-5.6 Codex xhigh`。它不是更早产品愿景，而是 2026-08-21 architecture-gap audit 新增的机制；[`decision-history.md`](../docs/design/references/decision-history.md) 第 131–137 行也承认当时五个新机制一次合入，回滚粒度过大。

复核纠正：文档没有逐字说“common-dir 不可读”；这是初审把“target ref 不可读”和 delivery 的 common-dir 负例揉在一起后的外推。但 delivery 的测试条款确实明确要求 common-dir 访问失败，所以实质问题仍在。

所有者裁决：**worktree 可以看到 common-dir 和 refs，这没有问题。** HCTL 真正要守的是结果如何准入、何时可以签 Integration Receipt，而不是向 Harness 隐藏普通 Git 元数据。

结论：这不是 coding blocker；相反，后续应删除 common-dir/refs 的负例，不要为它建设 Git broker 或独立 clone 体系。

### R2 · human-presence proof

初审质疑：[`spec/system.md`](../docs/design/spec/system.md) 第 87 行要求一次性、绑定规范命令摘要且不能由 CLI payload/env 代填的用户在场证明，但没有 challenge、TTL、防重放或 CLI handshake，无法实现。

关联传播：

- [`spec/agent.md`](../docs/design/spec/agent.md) 第 74、78 行；
- [`delivery.md`](../docs/design/delivery.md) 第 97、168 行；
- [`docs/design/README.md`](../docs/design/README.md) 第 48 行；
- [`docs/design/project.md`](../docs/design/project.md) 第 53–55 行；
- [`decision-history.md`](../docs/design/references/decision-history.md) 第 131、173 行。

所有者裁决：**用户只要正在操作 HCTL Workbench 或 `hctl2` CLI，就算“人在”。** 第一阶段不需要 nonce、challenge、command-digest proof，也不需要判断 CLI 是被人还是子进程启动。

最小边界应是入口分类：Workbench/CLI 命令按 human 入口审计，control reducer 是 system，harness/adapter 的 Result Proposal 通道是 execution。结果通道不能直接冒充治理命令；无需再做进程来源证明。

结论：当前 presence-proof 机制属于过度设计，不应进入实现关键路径。

### R3 · Matrix E2EE

初审发现仍成立：当前 Context、FTS、消息 digest 和恢复合同要求 HCTL 能读取 managed Room 的消息正文，但设计没有说明 E2EE 策略。若 managed Room 被客户端加密，现有明文提取和恢复路径不成立。

所有者裁决：**第一阶段 HCTL Room 强制非 E2EE。**

建议进入 chat/Tuwunel 切片时落实三个最小行为：创建 managed Room 时不启用 E2EE；发现 `m.room.encryption` 时拒绝继续作为 managed Room 使用；P0 覆盖加密尝试、redaction 后引用和 Context 降级。无需为第一阶段实现完整 Matrix crypto device、设备验证或密钥备份。

### R4 · Context Bundle 实际 bytes 放在哪里

初审说法过重：[`spec/project.md`](../docs/design/spec/project.md) 第 62 行要求实际交付内容至少保留到 owner 终态和 Result Proposal 准入窗口关闭，而系统存储图只明确列出 metadata ledger、Git、content server 和 cache，因此初审把“缺少 durable blob home”列成 blocker。

人话解释：HCTL 拼好一份 prompt 发给 Harness。文档说这份实际内容短期内要留着，以便崩溃后查看“当时到底发了什么”。问题只是这些 bytes 放 SQLite、普通文件还是缓存目录。

复核结论：

- “是否承诺执行收口前可查看实际交付内容”是产品行为；
- SQLite BLOB、文件目录、压缩格式、加密方式和清理任务都是 implementation；
- 不需要新增 blob store 领域对象，也不应阻塞 schema/scaffold；
- 如果首版不承诺长期逐字 replay，删除正文后保留来源 refs、digest 和“不可 replay”标记即可。

### R5 · Matrix Room 创建、归档和外部副作用

初审要求为 Matrix room create/archive 增加 `pending/active/result_unknown/unavailable` provisioning 状态机。这个要求过头了。

[`spec/system.md`](../docs/design/spec/system.md) 已经有通用 outbox/readback/结果未知合同；Room binding 也已有 health 投影。Matrix 创建超时、ACK 丢失和重试属于 adapter implementation，不需要再造一组 Project/Room 领域状态。

唯一产品问题是：Project 归档后，“Project Room 只读”究竟只指 HCTL 不再接受治理命令，还是还要修改 Matrix power level 禁止继续发消息。这个选择可以在实现归档切片时补一句，不阻塞当前 coding。

### R6 · Run pause、generation、reducer 与 `completion_pending`

初审把以下问题合并成核心 reducer blocker：

- pause/resume 没说明对已运行 Attempt、runtime、lease 和 clocks 的影响；
- attempt/runtime/control/site/backend generation 分层很重；
- Result Proposal → Attempt → Seat → Obligation/Gate 的原子归约未完全展开；
- Task 的 `completion_pending` 与 current Revision/Adopt 存在时序问题。

复核后应分层：

- **产品 design**：pause 对用户意味着什么。这个决定在进入 Run/B4 前需要拍板，但不挡 P0/P1/B0；
- **必须保持的语义**：旧执行的迟到结果不能覆盖新执行；一个 Seat 的备用 Attempt 不增加票数；Proposal 不是 Verdict/Receipt；同一 Task 不能同时有两个 task-bound Run；
- **implementation**：generation 拆几个字段、token 如何分配、哪一步同事务、reducer 代码结构、内部 claim 是否叫 `completion_pending`。

结论：初审把实现细节抬成了 design blocker。代码应先用最小 execution token/幂等/readback 模型落地，再由测试驱动细化，不必在设计文档中预先冻结数据库和 reducer 形状。

### R7 · P2 与 CLI

P0/P1/P2/P3 是施工阶段：P2 的意思是 control 已经可用、Workbench 还没有完成，因此当时靠 CLI 和 content 系统原生界面工作。

初审发现 [`delivery.md`](../docs/design/delivery.md) 的 P2 scope 表列出 Scoped Room、Memo/Artifact、完整 Task 管理等能力，但 CLI 表没有逐项列出对应子命令。这个发现本身成立，但性质只是交付清单 bookkeeping，不是架构 blocker。

不需要新增 generic escape hatch；[`spec/system.md`](../docs/design/spec/system.md) 已定义 Query/Preview/Submit/Subscribe。更合理的原则是：Workbench 不得绕过 public control API；CLI 覆盖自举、自动化和当前切片所需动作，并随切片增长。没有必要要求每个 GUI 动作都预先拥有一条专用 CLI 子命令。

## 三、同类过强断言清单

以下是沿 R1/R2 全库复查时发现的同类条款。它们不是本 memo 新拍板的产品行为，而是后续文档清理候选；实现者不应因为看到一个 MUST 就立刻建设额外子系统。

| 类别 | 当前位置 | 为什么偏重 | 建议最小合同 |
| --- | --- | --- | --- |
| 强制 OS sandbox 入场券 | `spec/agent.md:34`、`spec/system.md:101,192`、`delivery.md:97,169` | 把可选加固变成所有 Harness/B2 的硬门 | Worker Profile 声明普通本机/sandbox/container；如实显示，不把隔离强度冒充为治理权 |
| 全面 credential/network gateway | `spec/agent.md:34`、`spec/system.md:101`、`delivery.md:169` | 禁 SSH agent/provider config/network 会让现实 Harness/Git 难以工作 | 不复制、不持久化、不记录 HCTL 自身 secret；继承能力由用户 Profile 决定 |
| agentd-only terminal | `spec/system.md:36`、`spec/agent.md:78`、`delivery.md:20,240` | 把 tmux 拓扑和 direct attach 提升为产品正确性 | agentd 提供受管 attach；direct attach 的代码可重新扫描、封存、采纳 |
| discovery 绝不联网 | `spec/system.md:42` | 把无副作用与不联网混为一谈 | 不静默安装/改配置；联网探测可配置，install/upgrade 需显式确认 |
| 所有动作都必须 Preview | `spec/system.md:70`、`project.md:55`、`spec/project.md:92-96` | 日常命令会被强制增加交互层 | 危险动作默认确认；普通命令可直接 submit/`--yes`，仍做版本/权限检查 |
| writer 不可证明静默就永久弃用 worktree/ChangeSet | `spec/agent.md:38`、`delivery.md:164` | 默认保护被写成唯一恢复路径 | 默认隔离并提示；允许 user-confirmed takeover/adopt/seal/discard |
| 清理前绝不允许丢弃残留 | `spec/agent.md:56` | 用户无法显式放弃垃圾现场 | 默认保全；用户确认后可 discard |
| backend 无强排他即只读 | `spec/system.md:161` | 单用户环境被多租户级保证阻断 | 优先排他；做不到时 best-effort + readback + conflict warning |
| 固定锁路径和多层 generation | `spec/system.md:157-175` | 实现算法写进产品合同 | 只保留一个逻辑 writer、已确认副作用不重复、旧结果不覆盖新结果 |
| 固定存储拓扑/禁止 Git refs | `spec/system.md:121-131` | 与“物理布局是 control 私事”自相矛盾 | 路径作为默认示例；表、文件、Git refs 由实现选择 |
| Repo Instance 强绑定 common-dir 取证 | `spec/system.md:109-111` | remote/path/用户确认全部被降为不足 | stable repo ID 优先；缺失/冲突时展示证据并让用户确认 |
| Project 归档前必须清空全部开放对象 | `spec/project.md:42` | 归档变成逐项清场工程 | 只阻止活跃写执行/租约/未决写副作用；开放事项可作为历史归档 |
| Scoped Room 只有成功回填才能归档 | `spec/project.md:52` | 无结论讨论无法正常结束 | 允许 human 以 abandoned/no-decision/superseded 归档 |
| Context 必须毫秒级、全本地、零模型 | `context.md:17,34-38`、`spec/project.md:64-66` | 优化目标被写成准入硬门，字段和缓存机制随之膨胀 | 默认先用本地规则；模型辅助可配置；只冻结来源、digest 与已知缺口 |
| 活动 Run 时禁止采纳新 Task Revision | `spec/task.md:30` | Run 已冻结旧 Revision，没必要锁死 current | 允许采纳；旧 Run 不得静默完成新 Revision |
| 打包与 tmux 拓扑写死 | `delivery.md:240,249-250` | P0 本应验证实现，却预先决定每 runtime server/socket 和“必须原生” | 默认方案 + 能力矩阵；最终形态由 P0 证据决定 |

不建议松动的底线：secret 不进入日志/Room/Context；Electron renderer 不开放 Node/raw IPC；Receipt 只能在目标事实 readback 后签发；旧 Attempt/旧 execution token 的迟到结果不能自动覆盖当前执行；Harness 的 Proposal 不能直接冒充 Task/Run 完成。

## 四、开工准备

### 可以立即做

1. 工程脚手架：Rust workspace、toolchain pin、Cargo lock、基础 crate、格式化/lint/test/CI；
2. public command DTO、最小 SQLite migration、ID/digest/幂等 fixture 和 fake ports；
3. P0：Tuwunel（含 managed Room 非 E2EE）、Vikunja、tmux；Dagu 只需在 B4 前完成；
4. P1：`hctl2-tool` 的 Git/readback 原语与 `hctl2-agentd` 的 runtime/PTY 最小能力；
5. B0：单 writer、command/query/event、outbox/readback 和 CLI 生命周期。

### 不要先做

- 不要为隐藏 Git common-dir/refs 建 Git broker；
- 不要实现 nonce/challenge 型 human-presence protocol；
- 不要为 Context 新建领域级 blob store；
- 不要为 Matrix Room 另造一套 provisioning 聚合；
- 不要先冻结五层 generation、tmux socket 拓扑或 reducer 表结构；
- 不要为了补齐 P2 表而一次性生成完整 CLI 命令宇宙。

### 工程现场

评审时仓库仍是纯文档仓库，没有 Cargo workspace、`rust-toolchain.toml`、`Cargo.toml`、Node/package-manager pin 或 CI。Ubuntu 侧现有 GCC、Make、pkg-config、SQLite/FTS5、OpenSSL、Node/npm；本轮之后已安装 Rust/Cargo 1.98.0、rustfmt 与 Clippy，并完成临时 crate 编译验证，但这些是本机环境，不是仓库可复现基线。

另外，`.memo/*-p0-plan-20260822a.md` 多份旧计划仍写 Zellij/Conductor，而当前权威选型已是 tmux/Dagu。开六个 Harness 编码前应明确这些计划已过时，避免执行者照旧 memo 实现被取代的技术。

## 五、最终 Go / No-go

- **Go**：脚手架、P0、P1、B0，以及不依赖上述过强机制的纵向代码。
- **Go with local decision**：Chat 切片按“managed Room 非 E2EE”施工；Run 切片开始前再拍 pause 的用户语义；Project archive 切片开始前再拍 Matrix 是否随归档禁言。
- **No-go**：不是某个模块，而是三类不该写的复杂度——Git 隐藏/broker、密码学式 presence proof、把内部 generation/reducer/存储拓扑升格成领域合同。

最终判断：**没有理由继续停留在纯设计阶段；开始 coding，同时让代码和测试帮助文档从“预先证明一切”回到“只冻结真正的产品不变量”。**

# HCTL2 v0.7–v0.12 retrospective 与架构审计

> 日期：2026-08-21<br>
> 状态：Informative；记录历史理解、架构评价、本轮问题与修复，不覆盖 `docs/design/` 的规范。<br>
> 审计范围：从 `729238e`（v0.7 重组）到本轮修订前 `367ac3f`，同时回看 `f536e99`（v0.6.1）及这一段的全部提交、现存设计文档和 `.memo` 评审记录。

## 结论先行

当前架构不需要推倒重来。四个权威模块与四个操作场景的对仗是成立的：

| 权威模块 | 操作场景 | 核心职责 |
| --- | --- | --- |
| Project | Chat Room | 意图、协作、Context、Request、长期知识与交付物 |
| Task | Kanban | 承诺、来源绑定、操作投影与独立验收 |
| Run | Workflow | 冻结施工图、受治理执行、Gate、Verdict 与 Receipt |
| Agent | Terminal | Participant 的物理实现、ChangeSet、运行时、终端与 Evidence |

这四个模块不是强制流水线。Project 可以直达 Agent，Project 可以启动没有 Task 的 Run，Task 也可以不创建 Run。真正需要精心设计且不能因“精简”删除的，是模块之间的连接：谁交出什么不可变引用、目标模块由哪条命令准入、外部动作在哪个持久边界之后发生、崩溃后依据什么回读。

本轮发现的问题不是“领域对象太少”，而是 v0.10–v0.12 的数次横切转向没有完整传播：用户级 control 取代 per-clone control、Repo 级 Board 取代 Project 级 Board、三类数据重新划分事实源、Harness 模块改名 Agent、施工阶段重排。这些改动各自合理，但旧摘要、连接和验收仍残留前一版假设。

修复策略因此是：恢复缺失的正常 design，消除同一事实的两种解释，不恢复 v0.7 的对象爆炸，也不再为每个 crash point 发明一个新聚合。

## 审计方法与防止过度修复

本轮采用四条约束：

1. 先按提交历史识别 decision pivot，再检查当前端到端闭环；不从某个字段缺失直接推导新对象。
2. 同根问题跨文件归并成一次修复。例如“唯一用户级 writer”同时修正 system、connections 和设计地图，不再为每个模块各写一套 writer 状态机。
3. 只有稳定身份、独立 lifecycle、权限边界或恢复边界确实不同，才允许新增领域名；字段组、投影、一次动作和引用尽量留在既有对象中。
4. 只做一轮根因修复和一轮只读回归。若回归主要发现本轮新增概念造成的问题，应回滚该概念，而不是继续补丁摞补丁。

文档体量用于发现异常，不用于裁决设计价值。按 Git 中 `docs/design/**/*.md` 的可复核口径：v0.7 为 16 文件、3037 行、249 个一至三级标题；v0.8 收敛后为 10 文件、1470 行、139 个标题。若排除一直作为非规范研究记录的 implementation evidence 与 decision history，规范/设计主体从 2696 行降到 876 行，约减少 67.5%。删减比例本身不是错误；错误在于删除粒度同时跨过了“重复权威”和“唯一解释/恢复合同”之间的边界。

## v0.7 → v0.8 为什么过度精简

### v0.7 解决了什么

`729238e` 把 v0.6.1 的大文档拆成四层、领域模型、跨层生命周期、系统架构、Workbench、验证、术语和不变量等 16 份文档。它第一次把大量实现问题说完整，但也形成了三个严重副作用：

- 同一个状态或 CAS 规则在 domain、layer、lifecycle、invariants、validation 中重复定义；
- 四个实现层与四个领域所有权混在一起，读者容易把流程图当对象归属；
- 每发现一个 crash point 或 fence 条件，就新增对象名、Receipt 名或专章，形成补丁链。

所以 v0.8 的方向——用 Project、Task、Run、Harness 四个 owner 重画事实边界，并把连接与系统机制集中——是正确的。

### v0.8 删错了什么

`ff73679` 的压缩把“去掉第二权威”错误地推广成“去掉第二次解释”。它正确删除了重复状态机、重复字段表、实现级 schema 和 donor prose，却一并删除了以下正常设计：

- 一句话定位、失败模式、目标体验与各模块为什么存在；
- 四模块之间的完整 handoff、恢复顺序和合法 producer；
- Workbench 信息架构及没有 Workbench 时的可达操作面；
- 稳定契约测试 ID、失败矩阵和 decision 重新打开的机制；
- 术语之间“不是同一个东西”的消歧；
- Repo/clone、Context 传承、Engine retry、Gate 身份、ChangeSet 保全等实现者必须做选择的合同。

`ecaf4ec` 随即恢复 essential contracts，说明问题不是后来才产生：第一次简化完成时，可实现性审计已经发现它删掉了合法 writer、终态路径和恢复前置。`36db492` 又补回场景客户端边界。换言之，v0.8 的问题不是“四模块太少”，而是把**解释、连接和验证**误判成了平行权威。

### 应保留的教训

精简应该删除：

- 同一对象在多个文件中的完整定义；
- 只有一个步骤却被命名成聚合的对象；
- 每层重复的 CAS/outbox/fence/crash 算法；
- 已由外部标准完整表达、HCTL 没有差异化语义的自造词；
- UI、表结构和当前实现选型中的非合同细节。

精简不应该删除：

- 为什么这样划分 owner；
- 模块连接、事务边界和恢复 oracle；
- 威胁模型与权限边界；
- 用户如何操作、降级和恢复；
- 可独立验证的失败用例；
- reference implementation 的采用/拒绝证据。

## CamelName 为什么可以大量消除

CamelName 的大量减少不等于领域语义的大量减少。v0.9.1 与 v0.11.1 实际用了三种不同动作：

1. **删除伪概念**：把某个对象里的字段组、一个中间步骤或 UI 投影还原为普通语言，不再把它伪装成独立 aggregate。
2. **归并同义概念**：InvocationBinding / AttemptSpec 归入同一 Execution Spec，多个副作用 Intent 归入同一命令族，共同语义由 Revision、Binding、Receipt、Lease、Command、Snapshot 六族承载。
3. **只改词形**：`TaskRevision` 变成 Task Revision，`CompleteTaskIntent` 变成「完成 Task」命令。身份、writer、状态机和恢复边界完全未删，只是不在设计仓库提前冻结实现标识符。

这样做合理，因为代码拼写不是 design semantics。设计文档必须冻结的是：

- 这个名词是否有稳定身份；
- 谁能写、何时不可变；
- 它与相邻概念有什么不可互换的语义；
- 实现必须携带哪些精确版本、摘要和 authority。

仍应保留原形的例外也很明确：

- 已成为产品/行业词的 ChangeSet、Run Manifest、Gate Receipt；
- 合同必须逐字指认的 schema/wire 字段和序列化格式；
- Matrix、ACP、JCS、PTY、外部 API 等标准或源码原名。

v0.11.1 一度把纪律写成“设计仓库不含任何代码标识符”，这过于绝对。本轮把它收窄为“自造语义名不冻结代码词形；精确协议、schema 和格式可以保留原名”。这既避免伪类爆炸，也不牺牲可实现性。

## 各次大迭代解决的问题

| 版本 / 提交 | 主要问题 | 有效解法 | 留下的债务 |
| --- | --- | --- | --- |
| v0.7 · `729238e` | v0.6.1 单体文档难导航，执行/治理混杂 | 四层研究、领域模型、跨层生命周期、验证与不变量展开 | 多份平行权威、对象与 CamelName 爆炸 |
| v0.8 · `ff73679`–`36db492` | 平行权威与补丁链 | 四模块 owner、connections/system 集中、场景客户端分离 | 删除粒度过大，愿景/连接/验证被过压缩 |
| v0.9 · `d3ccac4` | 规则有了但不知道为什么 | 恢复 vision、失败模式、目标体验和模块 rationale | 仍有概念平铺 |
| v0.9.1 · `9e7c86f` | 产品语言与精确合同混在一起；75+ 具名概念 | design/spec 分层、六族归并、外部标准对齐 | 少数规则后来被过度泛化 |
| v0.10 · `98de4aa` | Room/Task/Run/Terminal 的 content 与治理都塞在本地账本 | 四场景 × metadata/content/artifact；三面架构；Harness 模块正名 Agent；用户级 control | 多项横切转向同时发生，旧连接未全部迁移 |
| v0.10.1 · `4e7eb8b` | Project 级 Board 无法自然对齐 repo 级 Issue | 一个 Repo 一个 Board，Project 是分组 | Project group anchor、跨组漂移和 identity claim 没闭合 |
| v0.10.2 · `f9d0024` | 每 clone 一本 ledger 破坏多设备连续性 | 一份用户级 metadata 账本 + Git；clone 只留现场锁/cache | 旧 per-Repo control writer 文案残留 |
| v0.10.3 · `233f884` / `8746dfc` | 未入册的高频实现名和存储泄漏 | 端口化、存储私有、无 counterpart 作为差异信号 | “账本只有跨系统边”“存储位置不构成合同”被说得过强 |
| v0.11 · `fb98792` | 不知道先建什么、何时敢自举 | P 阶段与 B 自举梯度、P0 限时验证、实现选型 | 最初把 probe 与产品化混为一谈 |
| v0.11.1 · `30b6e6d` | 代码词形继续制造伪类感 | CamelName/Intent/state 词形收敛，Tuwunel 定夺 | “不含任何标识符”过度绝对 |
| v0.12 · `9c07a13` | 施工序过长、core 名称模糊 | P0–P3 四段、`hctl2-tool` 正名、机械动作不交给模型 | P0 没有 lifecycle owner、P2 没公开治理入口、B2 缺 integration 闭环 |

## 当前架构评价

### 合理且应保留

1. **四模块 / 四场景**：owner 与 UX 视角对仗清楚，同时允许短路，不强迫线性流水线。
2. **三面架构**：展示面无事实，控制面拥有 metadata，执行面拥有 content/physical runtime。Workbench 是集成客户端，不是特殊内核；Feishu/Slack/Discord 可以替代 Chat Room 模块的界面，WezTerm 可以替代 Terminal 模块的界面，而不是替换整个 Workbench。
3. **固定内核 + 受控端口**：DeepSeek Harness/Cordis 启发的微内核思想已经进入 system contract：身份、authority、reducer、Receipt 与恢复留在固定内核；Chat、Task source、Workflow Engine、Harness、runtime 是可换端口。端口报告能力，binding 才授予 authority。
4. **三类数据**：metadata、content、artifact 的区分解决了“平台能否成为事实源”的伪冲突。平台可以拥有 content，不能拥有治理；Git 可以拥有不可变正文，不能自行证明 admission 或 Verdict。
5. **typed handoff + readback**：模块不直接改写彼此，外部动作先持久意图再执行，ACK 未知时按稳定关联键回读。
6. **Task 终结权收窄**：只有有权 human 从 Kanban 提交，或 task-bound Run 正常完成后 reducer 机械提交同一命令。Harness/LLM 只能提案和供证，不能靠 prompt 自报成功。
7. **Chat 中 human-in-the-middle**：普通 Room 中人天然处在讨论 focus；模型可以建议下一位 Participant，但只有人能批准临场 fan-out。自动协作边只来自冻结 Workflow。

### 大流程是否闭合

当前主链是合理的：

```text
Repo 注册 / 现场挂接
  → Project / Room / Context
  → Task Revision（可选）
  → Run Manifest（可选 Run）
  → Execution Spec
  → Agent runtime / ChangeSet
  → Result Proposal + Evidence
  → owner admission / Gate / Verdict
  → integration intent → tool/adapter → readback → Integration Receipt
  → Task Completion Receipt
```

两条短路同样闭合：Project → Agent 适合一次性研究/写入；无 Run Task 由 human 在精确证据与 Integration Receipt 上完成。Run 不再把 Engine terminal、Harness 退出或模型自述误当语义成功。

### 仍是实施风险而非新的领域模块

- macOS/Linux 上对通用 coding Harness 的 OS sandbox、Git broker、credential gateway 是否真能强制，是 P0/B2 的阻断性工程验证；若做不到，候选必须标 unsupported，不能把设计承诺降成 prompt。
- 用户级 control 的远程认证/传输、多主机现场编排和 split-brain 仍在第一阶段之外。
- Participant 商业化与 Context plane 的详细方案已分别留在 `.memo/participant-design-20260819.md` 和 `.memo/context-design-20260819.md`；进入规范前应逐条裁决，不应整篇升格成第五模块。
- Project 跨组 reparent、多 Task Run、全局检测/自动补偿任意带外写仍是明确后置范围。

## 本轮发现与修复

### 根因级修复

| 根因 | 修复 |
| --- | --- |
| managed Harness 与“同 OS 用户不隔离”的威胁模型互相否定 | 第一阶段受治理 Harness 必须进入 OS 强制 sandbox；control/human credential、SSH agent、target ref 和其他 ChangeSet 不可见；human 高风险命令要求 execution principal 拿不到的一次性 user-presence proof；agentd/tool 是终端、Git 与 credential gateway |
| 用户级 control 与 per-Repo control writer 并存 | 只保留一个用户级 control writer；Repo Instance 只有 tool/agentd 的 site fence 和资源锁，不是第二 control service |
| Repo/clone 身份链缺失 | 在既有 Repo 上补「注册 Repo」，在 system 补显式「挂接 Repo Instance」；remote URL/目录名/相同 HEAD 不足以证明身份 |
| Run 可以无条件进入完成并触发 Task | 加机械 normal-completion predicate；required Obligation/Seat/Gate/output、Receipt、所有 Attempt 撤权与 external-effect settlement 均须满足；Task Run claim 用 `active` / `completion_pending` 消除 Run→Task 交接竞态 |
| Context/Participant/Skill 与各种 generation 混写 | Run Manifest 冻结根 Context Manifest、精确 Participant/Role/Skill；Execution Spec 冻结 consumer-specific Context Bundle；owner version、runtime generation、control/site/backend fence 分层携带并逐项回传 |
| Git 与 ledger 谁拥有 immutable Revision 含混 | Task/Workflow/Memo/Artifact/ChangeSet 的不可变正文在 Git；ledger 独占 identity、admission、digest、current/lifecycle；Run Manifest、Execution Spec、Verdict/Receipt 权威在 ledger，Git Verdict/Receipt 只是审计影子 |
| P2 在 Workbench 之前却没有治理入口 | 公共 `hctl2` CLI 补到覆盖 B0–B5；provider 原生 UI 明确只是 content/diagnostic/break-glass，不冒充 HCTL command client |
| Repo Board 转向缺 Project 分组与 Task identity 政策 | 既有 task-source metadata 保存稳定 group anchor；anchor 永不 claim Task；只有唯一 Project group 下的卡可 claim；第一阶段 Task.project 不变，跨组/多组/未分组只形成 drift 并 fail closed |
| Project archive 可能冻住活动 child | Archive 定义为 quiescent transition，开放 Task/Request、非终态 Run/Invocation、Scoped Room、lease 或结果未知 effect 均阻止；Restore 不复活旧执行 |
| “后端离线不阻断治理”被读成所有命令照常 | 只保证已接纳事实不消失；任何要求 fresh head/revision/cursor/lease/readback 的 Start/Adopt/Complete/Move 均 fail closed |

### 精细修复

- immutable binding 不再包含 health、cursor、成员现状等动态观测；它们成为带版本投影。
- 无法证明旧 writer 已 fence 时，新执行必须使用新 worktree **和**新 ChangeSet；旧未封存/未跟踪字节必须保全。
- 本地/远端合入统一为持久 integration intent → tool/adapter → readback → Integration Receipt；纵向切片不再漏掉“代码真正进入目标 ref”。
- Task HCTL-first 创建把 ledger transaction、必需的后端卡片 outbox 与“仅初始契约分支需要”的 Git 正文 outbox 分开，不再声称跨系统写能塞进 SQLite 事务。
- Gate 第一期只宣称 distinct logical Participant revision 与 producer/reviewer separation；operator、公司、模型/provider 独立性若不能认证就显示 unknown/unsupported。
- Repo/Project/Run/Agent 的 Request deadline 分别回到精确 owner 收口，永不暗中终结 Task。
- 备份定义为唯一 writer 协调的一致 ledger snapshot + 精确用户定义 refs；恢复推进全部适用 generation，旧 descriptor/lease/outbox 权限失效。
- 交付验证恢复稳定 `CT-*` family ID；新增合同必须增加失败用例，不恢复每层一份不变量副本。
- P0 改成按首次消费推进的可丢弃 probe；P1 只是工具辅助，B2 才是第一次真正自举；Conductor 不再阻塞无 Run 的 B2。
- Tuwunel/Continuwuity 的嵌入式存储事实修正为 RocksDB 系，精确 release/commit/backend/features 留给 P0 固定。

## 本轮刻意没有做的事

- 没有恢复 v0.7 的 domain model、四层专章、lifecycle、invariants 与 validation 五份平行合同。
- 没有为 Context recall、delivery ACK、scope claim、backup、Project group 或 Task Run claim各造一个新顶层 aggregate。
- 没有把 Participant/Context 两份专题 memo 整篇升格进规范。
- 没有把第三方 provider 的内部数据库结构复制进 HCTL ledger。
- 没有把未来 marketplace、结算、多租户、multi-host 或跨 Project reparent 提前设计成第一阶段状态机。

这些不是遗漏，而是本轮用来避免 fix → 新问题 → 再 fix 循环的边界。

## 最终判断

从 design-doc 角度，当前结构已经回到一个健康尺度：

- design 层保留愿景、模块 rationale、三面架构和产品体验；
- spec 层只有四模块、connections、system 三类 owner；
- delivery 只写范围、施工序和可观察验证；
- decision-history 解释 pivot；implementation-evidence 保留参考实现研究；
- `.memo` 承载尚未升格的专题与审计。

这比 v0.7 少很多名字和重复规则，也比 v0.8 多回了必要的解释、连接、恢复与测试。后续若再发现问题，优先问“哪一个现有 owner 的合同缺了一条”，而不是“还需要什么新对象”。

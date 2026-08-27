# 方法论生态审计：家族地图、HCTL2 的定位与借鉴决策

> 日期：2026-08-24<br>
> 状态：Informative 研究备忘录，不定义 HCTL2 语义；复用判断以 [实现证据](../design/references/implementation-evidence.md) 的五种复用决策用语为准。<br>
> 方法：11 个方法论工具逐仓库克隆审计（源码 + 完整提交历史 + 许可证 + 完成判定权专项核对），另做一轮生态扫尾（gh api / 搜索，约 25 个候选仓库验证）。所有结论钉在具体 commit；宣传语一律不作数。

## 结论先行

1. **家族不止五个。** 用户已知的五族（spec 驱动、任务图驱动、流程/角色模拟、轻量纪律、编排器）之外，扫尾确认了七个驱动机制独立的家族：Ralph 式暴力循环、上下文工程/记忆纪律、Harness 技能包当方法论、会话多路复用（人当调度器）、系统记录寄生（Issue→PR 委派）、Swarm 自治群体、TDD/eval 驱动（弱家族）。没有找到第十三类。
2. **HCTL2 不属于任何单一家族——它是把四个家族各自最强的一段接成一条链，并补上全生态都缺的那一环。** Project/Room ≈ spec 驱动的塑形段 + 上下文纪律；Task/Kanban ≈ 任务图驱动 + 谁都没有的契约冻结；Run/Workflow ≈ 编排器的治理段 + TDD/eval 家族的确定性归约；Agent/Terminal ≈ 会话多路复用家族。全生态缺的那一环是**完成判定权**：11 个被审计工具里没有一个把"完成"完整地收归有权人类或确定性归约（详见下文横评表）——这正是 HCTL2 的差异化空间，也是它值得存在的最硬证据。
3. **借鉴方式的总答案：不引 CLI，重借 schema，少量抄件，大量借阶段。** 11 个工具全部得出"不采用为依赖"的结论（身份模型冲突、完成判定权冲突、供应链不稳三个原因至少中一个）；但它们的**协议形状**（delta 规格、EARS 验收语法、依赖边词表、归一化 transcript）是被几千 commit 实战打磨过的现成 schema；可整块搬走的代码只有几个边界清晰的小件；而**阶段划分与治理行为**（阶段门、只读审计、机械关单、无证据即重置）是最大的一笔可借资产。

## 一、家族地图（已知五族的校准 + 新七族）

### 已知五族——四个代表的归属需要校准

| 家族 | 代表 | 审计后的校准 |
| --- | --- | --- |
| spec 驱动 | **Kiro**（定名者，闭源）、**spec-kit**（GitHub 官方）、**OpenSpec** | 成立。三者共同点：意图先冻结为规格工件再施工。分野在归约端：OpenSpec 的 archive 是确定性归并 + 人类确认（全场最接近 HCTL2 立场的一步），Kiro/spec-kit 的任务完成仍是模型自勾 |
| 任务图驱动 | **beads**、**Taskmaster** | 成立。**vibe-kanban 要移出此族**：它无依赖图、无规格工件，看板只是 UI，内核是多 harness 执行编排（10 个 harness 的统一 spawn/续跑/日志归一/审批桥）——归编排器 |
| 流程/角色模拟 | **MetaGPT**（鼻祖）、**BMAD** | 方向成立但两者都在漂移：MetaGPT 默认路径已弃用论文里的 SOP 瀑布（Engineer 评审/QA 的 hire 代码被注释掉，`software_company.py:56-62`），改为 TeamLeader 单点 LLM 路由，团队重心迁往闭源 MGX SaaS；BMAD v6 里角色只是"可选交互皮肤"，真正骨架是四阶段文档链 + 少数确定性 Python 脚本 |
| 轻量纪律 | **GSD** | **名不副实**：GSD 现已是 67 个斜杠命令 + 33 个子代理 + TypeScript SDK 的重型 spec 驱动系统（含预计算任务图与对抗式验证者）。真正收缩成轻量纪律的反而是 **agent-os v3**——官方 CHANGELOG 明言"Claude Code 的 plan mode 现在做得更好，不再重造"，砍掉整条 spec/tasks/实现流水线，只剩标准挖掘/索引/注入三件套。这个收缩是重要行业信号：**方法论工具的流水线功能正在被 harness 原生能力吞噬，活下来的是 harness 之外的资产（标准、账本、治理）**——恰好是 HCTL2 选的位置 |
| 编排器 | **Gas Town**（+beads 配套） | 成立，且是全场唯一真编排器（spawn/监控/回收 tmux 里的真进程）。vibe-kanban 并入此族 |

### 新七族（扫尾发现，代表项目均经 gh api 验证存在）

| 家族 | 驱动机制 | 代表 | 与 HCTL2 的关系 |
| --- | --- | --- | --- |
| Ralph 式暴力循环 | 同一 prompt 循环喂无状态 agent，跨轮状态全在仓库文件，核心工程问题是退出检测 | snarktank/ralph（21.6k★）、frankbria/ralph-claude-code、Anthropic 官方 ralph-wiggum plugin | 全家族都在解"agent 自宣完成不可信"——与 Run 的确定性归约同构，仅参考行为 |
| 上下文工程/记忆纪律 | 工件是记忆/世界观而非承诺，方法论=策展纪律 | agents.md（23.8k★）、claude-mem、context-engineering-intro；PRP 是本族与 spec 族的杂交 | Project 模块是这类工件的领域宿主；AGENTS.md 是向多 harness 下发上下文的事实标准注入面（适配协议首选） |
| Harness 技能包当方法论 | 方法论本体就是 harness 扩展，随会话加载随会话消失 | obra/superpowers、EveryInc/compound-engineering-plugin、SuperClaude | 证明技能层是方法论的主流分发格式，但技能层天然给不出"证据高于自述"；HCTL2 的治理必须留在 harness 之外，技能包只当客户端注入通道 |
| 会话多路复用/并行 worktree | 人保留全部分派与裁决权，工具只管 N 个并行会话的生命周期 | claude-squad（8.4k★）、crystal、container-use | 编排器家族的对偶（调度者是人）。与 Agent/Terminal 模块同一问题域，既是"移植有边界的组件"候选清单，又是"身份绑在会话/worktree 上"的反例库 |
| 系统记录寄生（Issue→PR 委派） | 刻意不自建任务库，寄生在 GitHub Issues/PR 上，完成判定外包给 PR review/CI | automazeio/ccpm（8.3k★）、Multica（已在实现证据内深审） | 任务图家族的姊妹分支；Task/Run 与 GitHub 互操作时 CCPM 的 epic/issue/worktree/PR 映射是现成适配协议候选 |
| Swarm 自治群体 | 把并发拓扑本身（hive-mind/共享内存/consensus 投票）当方法论，人基本不在回路 | ruvnet/ruflo（原 claude-flow，69.2k★）、oh-my-claudecode | **主要作反面参照并建议明确暂缓**：多个模型互相投票仍是自述，不是证据 |
| TDD/eval 驱动（弱家族） | 下一步由失败测试决定，hook 强制红-绿次序，测试套件同时充当规格与完成判定 | nizos/tdd-guard（2.3k★，独立项目仅此一个过千星） | 机制上与 HCTL2 最同构的一族："测试通过"就是绑定 Run 的确定性归约的最小实例；作为独立社区未成形，纪律正被技能包吸收 |

未单列的：Tessl（spec 注册表商业化，公开仓库仅 70★，spec 族极端）；异步云 agent PR 交付（Devin/Codex cloud/Jules）机制已被"系统记录寄生"覆盖；代码知识图谱、模型路由网关、awesome 清单——是基础设施或教学材料，不是方法论。

> **2026-08-26 补记：编排器、群体自治、流程/角色模拟是不是一族。** 三家都把"多个 agent 之间的形状"当成方法论本身，所以在四场景里都落 Workflow；画场景地图时可以合成一个"多 agent 拓扑家族"标三个亚种。做借鉴决策时必须分开，因为它们在三个轴上分得很开：
>
> | | 编排器（Gas Town、vibe-kanban） | 群体自治（ruflo/claude-flow） | 流程/角色模拟（MetaGPT、BMAD） |
> | --- | --- | --- | --- |
> | 拓扑由谁持有 | 代码写死，运行时不变 | 模型临场生成（hive-mind、自组织） | 名义上是角色 SOP，实际骨架是阶段文档链 |
> | "agent"是什么 | 真进程：spawn/监控/回收 tmux 里的外部 harness | 同一运行时内的一批 LLM 调用，共享内存 | 提示词角色（BMAD v6 明说角色是"可选交互皮肤"） |
> | 完成怎么判、人在哪 | 机械关单：gates 过 + 推送验证后由 Go 代码带 commit SHA 关闭；人在门上 | consensus 投票，仍是自述；人不在回路 | 人批阶段门 + 模型自勾 done；MetaGPT 没有人类否决位 |
>
> 角色模拟这一族正在塌缩：MetaGPT 默认路径把 SOP 瀑布换成 TeamLeader 单点 LLM 路由（往 swarm 塌），BMAD 砍到只剩四阶段文档链加确定性脚本（往 spec 驱动塌）。稳得住的只有两头——拓扑在代码里、进程是真的（编排器），和拓扑在模型里、进程是假的（swarm）。对 HCTL2 的用法因此不同：编排器是 Run/Workflow + agentd + tmux 的同问题域亲戚，机械关单、convoy 归约、Witness 是正面证据；swarm 暂缓；角色模拟不是拓扑问题而是 Worker Profile 问题——角色是提示词层的东西，HCTL2 托管它但不拥有它，其阶段门可借给 Workflow Revision 当 Gate 形状。产品侧的来时路归类见[实现证据 ⑧ 来时路与场景落点](../design/references/implementation-evidence.md#lineage-scene-map)。

## 二、我们属于哪种实践

**产品层面**（HCTL2 是什么）：四段链各对应一个家族的强项，外加一个全场缺失的差异化环节——

- Project/Chat Room ≈ spec 驱动的塑形段（Kiro 的阶段门、spec-kit 的 constitution、agent-os 的标准挖掘）+ 上下文工程家族（依据的策展与注入）；
- Task/Kanban ≈ 任务图驱动（beads 的依赖边与 readiness 归约、Taskmaster 的任务 schema），**加上全生态都没有的契约冻结**——被审计工具的"任务"全部是可随时改写的活文件/活行，没有一家有不可变 Task Revision；
- Run/Workflow ≈ 编排器的治理段（Gas Town 的机械关单与 convoy 归约）+ TDD/eval 家族的确定性归约立场；
- Agent/Terminal ≈ 会话多路复用家族（claude-squad/crystal 的问题域，HCTL2 用 agentd+tmux 自answer）。

**研发实践层面**（HCTL2 项目怎么开发自己）：spec 驱动（设计层+合同层双层文档为权威，草案版本化，决策进 decision-history）+ 证据纪律（implementation-evidence 全部钉 commit）+ 多路评审与对抗核验（多个 harness worktree 并行出评审/P0 提案，人裁决）+ 限时验证（选型先跑可丢弃探针）。我们自己的工作方式恰是我们要造的产品的手工版。

## 三、完成判定权横评（本次审计最硬的一张表）

HCTL2 立场：Task 完成只接受有权人类命令，或绑定 Task 的 Run 正常完成后的确定性归约。11 个工具逐一核对源码后的实况：

| 工具 | 完成判定实况 | 与 HCTL2 立场 |
| --- | --- | --- |
| OpenSpec | 任务=模型自勾 checkbox（CLI 只数不验）；但 **archive 归约=确定性归并+人类确认，JSON 模式把确认点做成具名机读阻塞态**（`src/core/archive.ts:1289-1500`） | 中间层违反，最重一步同向（全场最佳单点） |
| spec-kit | 模型自勾 `[X]` + 自查 Done When；workflow gate 有人批但可被 `verdict_input` 非交互绕过 | 违反 |
| Kiro | agent 施工完自动改 `[x]`，漂移补救（Sync Files）仍是让模型再自查——用自述修自述；Correctness 的属性测试是可选项不做裁决 | 违反（且有 24 任务多会话即漂移的用户实测，issue #6826） |
| beads | 任何 actor（含模型）可 `bd close`，验收标准只是文本不校验；但 **gate 机制证明"确定性归约+人工放行"可表达为图节点**（gh:pr/gh:run/timer/human 四型） | close 违反，gate 同向 |
| Taskmaster | `set_task_status` 无权限模型且放进 MCP core-7——模型可直接把自己标 done；autopilot 的"测试证据"是 agent 用 `--results` JSON 自报的数字，系统从不亲自跑测试 | 违反（自述冒充证据的教科书反例） |
| vibe-kanban | 看板列由 **git/PR 事实确定性归约**（PR 开→In review，全部合并→Done；进程结束只通知不动卡）——生产系统里最完整的同向实证；但 MCP `update_issue` 给模型留了直接改 Done 的后门 | 归约同向，后门违反 |
| BMAD | spec 意图段人批准后冻结（仅人可改）；story 的 done 由模型按散文规则落笔；retro 验收最同向（机器裁决基于 git 证据、人可否决、无人决定的失败记 not accepted） | 混合，比 HCTL2 宽松一档 |
| MetaGPT | LLM 自发 `finish_current_task` 翻转布尔，人类无否决位；论文宣传的评审/QA 闸门在默认路径被自己人注释停用 | 违反（且质量闸门工程上名存实亡） |
| agent-os | v3 让渡给 harness plan mode；v2 是 implementer 自勾+verifier 抽查后补勾+自写 ✅ 报告 | v2 违反，v3 弃权 |
| GSD | 验证者是对抗式模型裁决但立场证据主义（"SUMMARY 声明不是证据"），默认八个人类门全开；`--auto` 链下模型 verdict 可直接推进 | 默认同向，逃生门违反 |
| Gas Town | 合并路径**机械关单**（gates 通过+推送验证后由 Go 代码带 commit SHA 关闭）、convoy 纯确定性落地、Witness"无证据即重置"；但模型对子 issue 有直接 `bd close` 权，治理规则靠角色提示词 | 主路径同向（已运行八个月的实证），侧门违反 |

读法：**同向的碎片散落在各产品里（OpenSpec 的归约门、vibe-kanban 的 PR 归约、Gas Town 的机械关单、GSD 的对抗验证、beads 的 gate），但没有一家把它做成不可绕过的系统边界**——每家都留了模型自报的通道（checkbox、MCP 后门、`--auto`、`bd close`）。HCTL2 要做的正是把这些碎片接成没有侧门的一条线。

## 四、借鉴决策（按五种复用决策用语）

对应用户的四问：直接用 CLI＝采用为依赖；借 schema＝适配协议；抄代码＝移植有边界的组件；借思想/阶段＝仅参考行为。

### 采用为依赖：零

11 个工具无一入选。三类否决理由（每个工具至少中一条）：
- **完成判定权模型冲突**：Taskmaster/beads/Kiro/MetaGPT 的模型可自标完成；
- **身份模型冲突**：spec-kit 用仓库路径/分支/feature.json 反向推导身份，GSD 的状态被 worktree 反向定义，Gas Town 用 cwd 反推 agent 身份，agent-os 用时间戳当目录身份——全是 HCTL2 明令反对的"对象被容器反向定义"；
- **供应链不稳**：Taskmaster OSS 停更导流闭源 SaaS（2026-02 起月提交个位数）、vibe-kanban 官宣 sunset（2026-04-24 终版）、GSD 九个月三易组织与 npm 包名、beads 十个月换过一次存储底座（SQLite→Dolt）、MetaGPT 维护态、agent-os 十个月三次架构重写。

### 适配协议（借 schema，最富的一层）

| 来源 | 借什么 | 证据锚点 |
| --- | --- | --- |
| OpenSpec | delta 规格文本协议：ADDED/MODIFIED/REMOVED/RENAMED 需求块 + Requirement/Scenario（SHALL + WHEN/THEN）+ 确定性归并语义 → Task Revision 的"相对现状变更"表达 | `schemas/spec-driven/schema.yaml`、`src/core/specs-apply.ts` @ f1b521df |
| spec-kit | ① 多 harness 命令/事件适配矩阵（单一规范命令文档 + 占位符重写 + 每 harness 一份 registrar_config + canonical→native 事件映射 + 各家非交互执行参数），38 个集成 parity 测试验证过；② spec 模板的 FR-###/SC-### 稳定键 + 独立可测 user story | `src/specify_cli/integrations/base.py`、`templates/spec-template.md` @ 27f50f7e |
| Kiro | ① EARS 验收语法（WHEN/IF/WHILE/WHERE…THE SYSTEM SHALL…，bugfix 用 SHALL CONTINUE TO 锁不变行为）——把验收写成可逐条判真的原子命题，正是确定性归约需要的输入形状；② 任务行尾 `_Requirements: 1.1, 2.3_` 回链格式 | kiro.dev/docs/specs/（格式公开，源自 Rolls-Royce 公开方法论） |
| beads | 依赖边类型词表（阻塞类 blocks/parent-child/conditional-blocks/waits-for vs 注解类 related/discovered-from/…）+ "边类型决定是否参与 readiness 归约"的分层 | `internal/types/types.go:1214-1308` @ 8d86c06b |
| Taskmaster | tasks.json 的 tagged 任务图 schema 作导入互操作格式（装机量最大的存量）；testStrategy 字段进 Task Revision 字段设计 | `packages/tm-core/src/common/types/index.ts:131-166` @ c0c98d36 |
| vibe-kanban | ① NormalizedConversation/NormalizedEntry 统一 transcript schema（9 个 harness 的 normalize_logs 验证过覆盖面）→ Run 证据流的规范化事件形状；② ExecutorAction 链（setup→agent→cleanup 作为可持久化数据） | `crates/executors/src/logs/mod.rs`、`actions/mod.rs` @ 4deb7eca |
| BMAD | ① spec 契约形状：`<frozen-after-approval>` 人有意图段 + 可变执行段 + 追加式变更日志 + Verification 命令段；② build-auto 无人值守终态协议（done/blocked+具名原因，blocked 永久直至人显式重试，回环 >5 强制 HALT） | `src/bmm-skills/ship/bmad-build/spec-template.md`、`docs/reference/build-auto.md` @ 1479a58b |
| GSD | ① must_haves 目标反推验收 schema（truths/artifacts/key_links 三层，核验三级递进 exists→substantive→wired）；② checkpoint 协议（human-verify/decision/human-action 三型 + gate=blocking + 计划级 autonomous 布尔） | `templates/phase-prompt.md L547-611`、`references/checkpoints.md` @ bdcaab2c |
| Gas Town | 多 harness 上下文注入矩阵（Claude/Gemini settings 钩子、Copilot .github/hooks、OpenCode JS 插件、Codex 启动 nudge；base→role→rig 三层覆盖合并）——踩坑八个月的成品 | `docs/HOOKS.md`、`internal/config/agents.go` @ 649b832b |
| agents.md（生态） | AGENTS.md 开放约定作为跨 harness 上下文注入面 | github.com/agentsmd/agents.md |

### 移植有边界的组件（抄代码，少而精）

| 来源 | 组件 | 理由 |
| --- | --- | --- |
| vibe-kanban | **executors crate 整体**（10 个 harness 的 spawn/续跑 session-id 语义/日志流解析/审批桥） | 全场最有价值且边界最清晰的资产；项目已 sunset 不能当依赖，Apache-2.0 移植安全——Agent/Run 模块正需要这层 |
| Gas Town | `internal/tmux` Go 封装（NewSessionWithCommand 避 send-keys 竞态、WaitForRuntimeReady 按 runtime 探测就绪、每镇独立 socket） | HCTL2 已拍板 tmux 运行时后端，这是被 98 个文件实战锤过的封装（MIT）；若不用 Go 则降级为仅参考行为 |
| BMAD | `git_evidence.py`（只度量不裁判，JSON-only，stdlib 无依赖带单测）、`memlog.py`（追加式盲写原子决策日志） | 与"证据/裁决分离"立场完全一致的最小确定性件，可直接抄 |
| tdd-guard | hook 拦截实现（在 harness 侧强制外部纪律、采集违规证据） | Run 的 Verdict/证据链参考，候选 |

### 仅参考行为（借思想/阶段，最大的一笔）

- **阶段划分**：Kiro/spec-kit 的 意图→需求→设计→任务 逐阶段人批门（Quick Spec 反证了门的存在）；spec-kit 的 analyze 严格只读、converge 把缺口归约为新任务而不改历史、constitution 修宪走独立流程、gate 无人时挂起（PAUSED）而非降级为自动通过。
- **治理机制**：Gas Town 机械关单（close reason 内嵌 MR id/commit SHA 作回执）+ convoy 确定性落地 + Witness "无证据即重置"；GSD 四型门分类（Pre-flight/Revision 有界回环/Escalation/Abort）+ 逐任务原子 commit 回执链 + safe_resume_gate 按 git 事实对账孤儿状态；BMAD retro 裁决三硬规则（机器裁决基于证据、人永远可覆盖、无人决定的失败记 not accepted）；beads gate 图节点与"跨节点接管必须显式"的租约语义；Taskmaster 复杂度报告作为人可先审的独立中间工件 + findNextTask 纯函数归约。
- **上下文纪律**：agent-os 的标准 挖掘→索引→按需注入 回路与逐条问 why；GSD 的"计划即给全新上下文子代理的 prompt"+ 调度决策冻进契约工件（wave/depends_on 计划期预算）；"Task 1 永远是保存 spec 文档"（施工前塑形产物必须已落盘）。
- **反面教材（写进设计文档的否决理由）**：MetaGPT 单点 LLM 路由与被注释掉的质量闸门；Kiro 用自述修自述的 Sync Files；Taskmaster 自报测试数字；Swarm 家族 consensus 即自述；prompt 文字当强制层（spec-kit/BMAD/GSD 通病——BMAD 自认"prompt-level behavior contracts, not enforceable checks"）。

### 暂缓

beads 整体（等 HCTL2 Task schema 定形后重估，届时做单向 issues.jsonl 导出适配成本很低）；Swarm 家族；Dolt 存储底座；spec-kit 的 extension/preset/bundle 市场机制；Gas Town 的 Wasteland 跨镇联邦。

## 五、审计基线一览

| 工具 | 仓库 | 钉定 | 许可 | 历史画像 |
| --- | --- | --- | --- | --- |
| OpenSpec | Fission-AI/OpenSpec | f1b521df | MIT | 2025-08 起 796c/45 tag，实质双人 |
| spec-kit | github/spec-kit | 27f50f7e (v1.0.1+) | MIT | 2025-08 起 1854c/217 tag，GitHub 官方 |
| Kiro | 闭源（kiro.dev） | 文档快照 2026-08-24 | 专有 | 2025-07 预览，~2025-11 GA，Amazon Q 官方继任 |
| beads | gastownhall/beads | 8d86c06b (v1.2.2) | MIT | 2025-10 起 10716c，Yegge 44%+agent 蜂群，2026-01 换存储底座 |
| Taskmaster | eyaltoledano/claude-task-master | c0c98d36 (0.43.1) | MIT+Commons Clause | 2025-03 起 1216c，2026-02 起停更导流 SaaS |
| vibe-kanban | BloopAI/vibe-kanban | 4deb7eca (v0.1.44 终版) | Apache-2.0 | 2025-06 起 2070c/441 tag，2026-04-24 官宣 sunset |
| BMAD | bmad-code-org/BMAD-METHOD | 1479a58b (v6.11.0) | MIT+商标声明 | 2025-04 起 2060c，创始人 52%，v4→v6 断代重构 |
| MetaGPT | FoundationAgents/MetaGPT | 11cdf466 | MIT | 2023-06 起 6367c，2025 起维护态（165c/年），重心迁闭源 MGX |
| agent-os | buildermethods/agent-os | cae8e664 (v3.0.0+6) | MIT | 2025-07 起仅 130c，单人 83%，十个月三次重写 |
| GSD | gsd-build/get-shit-done（已归档）；后继 open-gsd/gsd-core | bdcaab2c；后继 314ea20f | MIT | 2025-12 起 2928c/229 tag，64.6k★，九个月三易其主 |
| Gas Town | steveyegge/gastown | 649b832b (v1.2.1 后) | MIT | 2025-12 起 7770c，1.5 个人类 + 自家 agent 蜂群，17.7k★ |

生态扫尾的搜索方法、重定向记录（PRPs-agentic-eng→Wirasm/prp、claude-flow→ruvnet/ruflo、steveyegge/beads→gastownhall/beads）与"判定为非方法论"的高星现象清单，见本轮工作流产物（星数为 2026-08-24 gh api 实测）。

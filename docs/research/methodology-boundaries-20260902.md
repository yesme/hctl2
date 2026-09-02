# 方法论工具的阶段边界审计：19 家逐边抽表，对照 HCTL2 四模块交接

> 日期：2026-09-02<br>
> 状态：Informative 研究备忘录，不定义 HCTL2 语义；发布后正文不改，只在文末追加复核记录。复用判断沿用 [docs/research/README.md](./README.md) 的五种复用决策用语；本文的裁决词是另一套——遗漏 / 过早 / 改写 / 维持——只对「边」说话。<br>
> 方法：对[方法论生态审计](./methodology-landscape-20260824.md)审计基线表的 11 个工具加七个新家族的代表（snarktank/ralph、thedotmack/claude-mem、obra/superpowers、smtg-ai/claude-squad、automazeio/ccpm、ruvnet/ruflo、nizos/tdd-guard），再加 mattpocock/skills（引用[既有审计](./methodology-mattpocock-skills-20260902.md)，不重读），每家抽同一张「阶段边界表」：阶段、阶段产物、跨越条件（原文）、谁判、机械还是散文、对应 HCTL2 的哪条交接。18 个开源仓库浅克隆 HEAD 逐文件读源码、模板与 prompt 文本；Kiro 闭源，读 kiro.dev 公开文档快照与三个 GitHub issue。然后把 HCTL2 的交接总表逐行列出并排，分三类归纳，每条四选一。宣传语一律不作数。<br>
> 证据钉：外部对象钉 commit 见文末[审计基线一览](#五审计基线一览)，行号钉在该 commit；HCTL2 侧核对于 main @ `6850f18`（草案 v0.16.0，2026-09-02），引用用「文件 §节名」。本轮的抽取稿（每仓库 8–15 行、含全部原文引用）留在工作流产物中，正文只保留能改变判断的边。

## 结论先行

1. **19 家的边有一个共同形状：门后没有冻结。** 阶段产物里不可变的只有 git commit 和 GitHub issue 号；`requirements.md`、`tasks.md`、`tasks.json`、`prd.json`、`PLAN.md`、`STATE.md`、`sprint-status.yaml`、`.claude-flow/*.json` 全是可就地改写的文件。HCTL2 八条连接的交付物全部带 digest 或代次。这不是「我们更严」，是递的东西不同：别家递的是当前状态的一份副本，HCTL2 递的是对精确版本的引用。用户已经在替我们要这条边——Kiro #5019 "Treat Completed Task as Immutable" 被以无跟进关闭，#6826 说 `tasks.md` "drifting from actual implementation state"。
2. **别家的散文门集中在计划段、机械门集中在集成段；HCTL2 反过来。** 需求 → 设计 → 任务的门几乎全是 prompt 或人点按钮（Kiro 三道 "Happy?"、spec-kit 的 Constitution Check、OpenSpec "enablers, not gates"）；PR merged、tests pass、exit code 才是代码判。HCTL2 计划段只有一道门（H1 采纳）但它是机械准入，集成段两道（H6 合入、H7 完成）也是机械且逐项绑定证据。**计划段少门、集成段重门，是 HCTL2 这一票的形状**，19 家里没有一家这么投。
3. **别家有、HCTL2 没有的边六条：五条维持，一条改写。** 规格链内部的门、任务间 readiness 归约、会话间记忆交接、完成后的归并 / 回顾、编辑级 TDD 门——换一种方法论就不需要，归 Skill，HCTL2 托管不拥有。改写的一条是「等待外部机械事实」的节点：beads 把 gate 做成一等 issue 五型（human / timer / gh:run / gh:pr / bead），vibe-kanban 用 PR 事实驱动看板，Gas Town 用 merge proof 关单；HCTL2 的 Workflow Profile 里这种节点没有名字，要在 Run 里「等 CI 绿」只能派一个 Participant 去查，把机械事实伪装成执行体提案。改法是给外部执行节点补一种 `executor = tool` 的 Obligation，产出外部事实的 Evidence，不占席位不投票。
4. **只有 HCTL2 有的边九条：找不到一条「过早」。** 每条都能在别家找到用 prompt 写出来的同向碎片（"SUMMARY.md claims are not evidence"、"Do Not Trust the Report"、"Judge against the diff, not against the implementation subagent's report"、"not the LLM's self-report"）、自己长出来的半成品（四家记 baseline sha、两家长出持久状态机、vibe-kanban 的专用 approve 端点、Gas Town 的 `commit_sha` 回执链）、或用户替我们要它的事故记录。「市场没提出需求、我们先钉死」这种情形没有出现。
5. **别家定义得比我们干净的三处，全是「我们有对象、缺形状」，三条都改写、都不新增对象。** BMAD 用 `intent_gap` / `bad_spec` 让评审者按根因落在冻结块内外分流回环——HCTL2 有语义返工和替代执行两条路，但谁判走哪条没写。GSD `must_haves` 把验收项分成真值 / 工件 / 连线三类、核验分 exists → substantive → wired 三级、前两级代码判后两级模型判——HCTL2 说了 Receipt 绑什么，没说一条验收项长什么样。tdd-guard 的 `test.json` 由 reporter 在测试进程内写、不经模型转述——HCTL2 的 Evidence 没有生产者字段区分「工具箱直接产生」和「harness 事件转述」。
6. **「谁判」列的总账，比 landscape 的横评更细一档。** 15 家有任务完成这条边，14 家让模型在主路径或侧门上自己标完成；唯一例外 ccpm 靠人宣告，但也没有任何机制阻止子代理运行 `gh issue close`。全场由代码判完成且输入不由施工模型写的只有四处——Gas Town `gt mq post-merge`（merge proof）、vibe-kanban PR merged → Done（MCP 后门除外）、BMAD retro `pending_stories`、GSD `phase.complete`（但输入可被工作流改写）——且四处都是「合并即完成」或「文件齐即完成」，没有一处按验收约束逐项校验。对 landscape 的两处校准：Gas Town 的「机械关单」只覆盖关单那一步，合并决策是 Claude 按清单手打 `git merge`，Go 合并管道在 HEAD 无调用方；Witness 是 restart-first 不是 reset-first。

## 一、读表方法

本文只审一件事：**工作从一种状态进入另一种状态的点**——谁递什么、凭什么过、谁判、判据是代码还是一句 prompt。它不评产品、不评完成判定权的总账（那在[方法论生态审计](./methodology-landscape-20260824.md)第三节），只把每家的边一条条摆出来，和 HCTL2 的边并排。

五个列的口径：

- **阶段产物**：过边时递过去的东西。写清载体（文件、issue、数据库行、git commit、stdout 一句话）与可变性（能不能事后改、有没有版本或摘要）。
- **跨越条件**：原文照抄，配位置。外部仓库钉 commit 用 `文件:行`；Kiro 闭源用 `URL § 节`；HCTL2 用 `文件 §节名`。
- **谁判**：人、模型、代码三选一，混合的写清哪一步谁判。
- **机械还是散文**：条件由代码判定（退出码、文件存在、checkbox 解析、状态字段比较、PR 状态）是机械；条件是一句 prompt 让模型自己判断是散文。「机械载体 + 散文判断」是最常见的混合——checkbox 由代码数，但勾不勾由模型决定。
- **对应 HCTL2 交接**：用下面的编号。

HCTL2 的边（编号沿用全文）：

| 编号 | 边 | 一句话 |
| --- | --- | --- |
| H1 | Project → Task | 讨论升格为承诺：「创建 Task」/「采纳契约」命令冻结不可变 Task Revision |
| H2 | Project / Task → Run | 承诺进入治理：批准 Workflow 与「启动 Run」是两个动作，Run Manifest 冻结引用 |
| H3 | Project → Participant | 无 Run 的短路：Room Invocation + Execution Spec |
| H4 | Run → Participant | Attempt + Execution Spec 派发给执行体 |
| H5 | Participant → Project / Run | 执行体交回 Result Proposal，归属模块校验后归约为 Verdict / Receipt；自述、进程退出、终端屏幕都不算 |
| H6 | → Participant「合入 ChangeSet」 | 人或 Run reducer 下令，工具箱执行并回读，出 Integration Receipt |
| H7 | → Task「完成 Task」 | 只接受有权 human 或 task-bound Run 正常完成后的归约；Task 独立校验，出 Task Completion Receipt |
| H8 | Task / Run / Participant → Project | 里程碑低噪声投影；Memo / Artifact 沉淀需 Project 自己的命令 |
| H9 | Request 回路 | 需要人澄清 / 决定 / 授权时创建 Request 阻塞对应对象；「解决 Request」由所需 actor 提交，普通聊天回复不算 |

模块内部还有几道门，别家常把它们当阶段，对照时一并列出：G1 Repo Room → Project 提升、G2 Workflow 登记 / 编译 / 批准、G3 Run 内 Gate、G5 Scoped Room 结案、G6 Context 接力。精确定义见第三节。

## 二、逐工具阶段边界表

表头统一为：阶段（从→到）· 阶段产物（载体、可变性）· 跨越条件（原文 + 位置）· 谁判 · 机械/散文 · 对应 HCTL2 交接。「对应」列用上节的 H1–H9 编号；写「无对应」时给一句原因。位置引用是 `文件:行`，行号钉在本节标题里的 commit；Kiro 用 `URL § 节`。每个工具只保留能改变判断的边，完整抽取稿（含全部原文）在本轮工作流产物中。

### 2.1 spec 驱动

#### OpenSpec（Fission-AI/OpenSpec @ d0071d73，2026-09-01，MIT）

一条 change 是一个目录，状态就是目录里有哪些文件：`schema.yaml` 声明 proposal → specs/design → tasks → apply 的依赖图，CLI 用「文件是否存在」判 artifact 完成（`src/core/artifact-graph/state.ts:14-29`），用正则数 `- [x]` 判施工进度（`src/utils/task-progress.ts:23,48`）。唯一有机械门的是 `openspec archive`。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | proposal 写完 → specs/design 解锁 | `proposal.md`（可变，无版本） | 依赖图 `requires: - proposal`（`schemas/spec-driven/schema.yaml:134-135,169-170`）；完成 = 文件存在（`state.ts:22-26`）；未满足只给 `<warning>` "Complete them first or proceed with caution"（`src/commands/workflow/instructions.ts:216-223`） | 代码给状态字段，模型决定是否照做 | 机械字段 + 散文遵守 | 无对应：HCTL2 不在 Task 内部细分规格文档阶段 |
| 2 | specs+design → tasks | `tasks.md`（checkbox，可变） | `requires: - specs - design`（`schema.yaml:211-213`）；"Dependencies are enablers, not gates"（`src/core/templates/workflows/propose.ts:113`） | 模型 | 散文 | H2 弱对应：施工图写完即算通过，没有批准动作 |
| 3 | 计划完成 → 开始施工 | `instructions apply --json` 的 `state: blocked \| ready`（瞬时，不落盘） | 机械侧：缺 artifact / 缺 tasks.md / 无可做任务 → `blocked`（`instructions.ts:438-461`）。人闸只在 prompt："After the planning artifacts are complete, stop… Wait for a new user request"（`propose.ts:16`）；apply "Can be invoked anytime"（`apply-change.ts:194`）；模板自认 "prompt-level behavior contracts, not enforceable checks"（`apply-change.ts:74-78`） | 代码 + 人（下一条消息）+ 模型 | 机械字段 + 散文人闸 | H2「启动 Run」：人发新请求即启动，无 Run Manifest |
| 4 | 一个任务完成 → 下一个 | `tasks.md` 该行 `[ ]`→`[x]`（模型手改） | "Only mark a task `- [x]` when its specified behavior is fully implemented"（`apply-change.ts:182`） | 模型 | 散文勾选，机械计数 | H5 无对应：勾选即执行体自述 |
| 5 | 施工遇阻 → 问人 | 对话内 "Implementation Paused" 文本块（`apply-change.ts:153-171`） | "Pause if: Task is unclear → ask for clarification…"（`:107-112`） | 模型 | 散文 | H9 无对应：没有 Request 对象，普通回复即解决 |
| 6 | 全部勾完 → 可归档 | `state: all_done`（瞬时） | `remaining === 0 && total > 0`（`instructions.ts:452-454`） | 代码 | 机械（数 checkbox） | H5 弱对应：只计数，无证据引用 |
| 7 | 归档 · 校验 + 未完成任务确认 | 无产物 | delta spec 校验失败即停（`src/core/archive.ts:1268-1295`）；未完成任务 → "Continue?" 默认 N（`:1352-1356`）；JSON 模式无 `--yes` 抛 `archive_tasks_incomplete`（`:1341-1350`）；`--yes` 直接过（`:1369`） | 代码 + 人；`--yes` 让模型可绕 | 机械计数 + 可移除的人闸 | H7 最接近「有权人类命令」的一点，但 `--yes` 把它降为可选 |
| 8 | 归档 · delta → 主 spec 确定性合并 | 主 spec 重写；结果 JSON 只打到 stdout（`archive.ts:2047-2054`） | 预演 → 确认 "Proceed with spec updates?" → 指纹比对（输入变了拒写，`:1540-1548`）→ 全部过 validator 再写（`:1589-1599`）；JSON 无 `--yes` 抛 `archive_confirmation_required`（`:1488-1494`） | 代码 + 人一次确认 | 机械 | H6 部分对应：确定性合并 + 前后指纹 ≈ 合入 ChangeSet，但回执不持久 |
| 9 | 归档 · 目录搬迁 → 事后 lint | `archive/YYYY-MM-DD-<name>/`；`validate --archived` ERROR 列表 | 先占位再搬（`archive.ts:2030-2040`）；`validate --archived` 数未完成任务报 ERROR（`src/commands/validate.ts:482-489`） | 代码 | 机械 | H7 的落地动作 + 事后补丁：承认 `--yes` 能把未完成 change 归档，靠 CI 兜底 |
| 10 | 会话 → 会话 | 目录内文件；无记忆条目 | continue 模板 "STOP after creating ONE artifact"，靠 `openspec status --json` 重算（`continue-change.ts:39-47,82`） | 代码重算 | 机械 | 无对应：文件系统当唯一状态源 |

判点：同一条归档边在同一仓库里有一个机械版（CLI）和一个散文版（skill `openspec-archive-change` 让模型自己 `mv`，`archive-change.ts:145`，且 "Don't block archive on warnings"，`:173`）。JSON 模式的每个阻塞错误都附带 `--yes` 重跑命令（`archive.ts:282-303`），等于把绕过方法直接教给调用的 agent。零 delta 缺口由源码自述（`archive.ts:1236-1238`）。

#### spec-kit（github/spec-kit @ 0053c3a3，2026-09-01，MIT）

每个 slash command 是一份 Markdown 模板，frontmatter 挂一个脚本做机械准入，正文全是给模型的散文步骤。机械准入只有四件「文件存在」：feature 目录、plan.md、spec.md、tasks.md（`scripts/bash/check-prerequisites.sh:139-163`）。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 治理原则 → constitution.md | `.specify/memory/constitution.md`（整文件覆写；semver 由模型算） | "Write the completed constitution back… (overwrite)"（`templates/commands/constitution.md:125`） | 模型 | 散文 | 无对应：接近 Project 级 Memo |
| 2 | 需求口述 → spec.md | `specs/<NNN>/spec.md`（可变）+ `.specify/feature.json` 指针 | 目录由模型 `mkdir`（`templates/commands/specify.md:94-106`）；NEEDS CLARIFICATION 上限 3 个并等人答（`:128,230`） | 模型 + 人 | 散文 + 机械指针 | H1 弱对应：spec 是 Task 草稿，无 digest |
| 3 | spec → clarify → plan | spec.md 内追加 `## Clarifications`；`plan.md` 等 | clarify 门是软的："If the user explicitly states they are skipping clarification… you may proceed, but must warn"（`clarify.md:60`）；`setup-plan.sh` **不检查 spec.md 存在**（`scripts/bash/setup-plan.sh:30-65`）；Constitution Check "GATE: Must pass before Phase 0"（`templates/plan-template.md:39-41`）由模型自判 | 模型 | 散文（门）+ 机械（feature 上下文） | H9 散文版；H2 弱对应 |
| 4 | plan → tasks.md | `tasks.md`（严格 checkbox 格式） | `if [[ ! -f "$IMPL_PLAN" ]]… exit 1`（`setup-tasks.sh:31-41`） | 代码 | 机械准入 | H2：施工图分解，仍无冻结 |
| 5 | tasks → analyze（只读）→ implement | 对话内报告；checklist 状态 | analyze "STRICTLY READ-ONLY"（`analyze.md:58`），结论只 "Recommend"（`:196`）；implement 前 "If any checklist has unchecked items: STOP and ask… (yes/no)"，全勾则 "Automatically proceed"（`implement.md:79-88`）；checklist 的 `[x]` "does NOT mean implementation work is complete"（`:59`） | 代码（tasks.md 存在）+ 模型（数框）+ 人（仅在有未勾项时） | 机械 + 散文 | H2「启动 Run」：人确认只在清单不干净时出现 |
| 6 | 一个任务完成 → 下一个 → 收尾 | `tasks.md` `[X]`（模型手改）；"Completion Report" | "make sure to mark the task off as [X]"（`implement.md:169`）；Done When "All tasks… marked `[X]`"（`:219`）；仓库内无任何脚本读取 `[X]` | 模型 | 散文 | H5/H7 无对应：勾选即自述，无 Receipt |
| 7 | implement 后 → converge | `tasks.md` 末尾追加 `## Phase N: Convergence`（append-only） | "APPEND-ONLY, NEVER REWRITE"（`converge.md:73-74`）；无差距则 "byte-for-byte unchanged"（`:82-83`）；开 PR 只是建议（`:233`） | 模型 | 散文 | H5 散文替身；H6 无对应 |
| 8 | workflow 引擎：specify → gate → plan → gate → tasks → implement | `.specify/workflows/runs/{id}/state.json`（gate 的 `output.choice` 落盘） | review-spec / review-plan 是 `type: gate`（`workflows/speckit/workflow.yml:50-54,62-66`）；非 TTY → `PAUSED` 落盘而不是静默通过（`src/specify_cli/workflows/steps/gate/__init__.py:174-175`）；`specify workflow resume` 恢复；tasks → implement **无 gate**（`workflow.yml:68-78`） | 人 + 代码状态机 | 机械 | H2「批准 Workflow」的机械版；review-spec ≈ H1 采纳 |
| 9 | gate 的非交互绕过 | 同 state.json | `verdict_input` 可指定一个 workflow input 作为选择（`gate/__init__.py:29-30,143-159`）；必须在 YAML 显式声明，内置流未开 | 代码 | 机械（等价 `--yes`） | 与 HCTL2「所需 actor 提交」冲突 |

判点：spec-kit 是全场唯一把「谁有权勾哪个框」写清楚的：tasks.md 的 `[X]` = 工作完成（模型勾），checklists 的 `[x]` = 评审者认可需求质量（`templates/checklist-template.md:8-9`；implement "must not modify markers"，`:41`）。但两者都没有代码校验。gate 步的 PAUSED 落盘是全场最干净的「无人时挂起而不降级为自动通过」。

#### Kiro（kiro.dev 文档快照 2026-09-02，闭源专有）

流水线是 `.kiro/specs/<name>/{requirements,design,tasks}.md` 三份可手改 Markdown；进度唯一载体是 `tasks.md` 的 `[ ]/[x]`，无版本、摘要、回执。门都是 IDE 里「人点按钮」。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | requirements.md → design | `requirements.md`（EARS 文风，可手改、可被 Refine 级联改写） | 流程图 `C{Happy?} -->\|yes\| E["design.md"]`（kiro.dev/docs/specs/feature-specs/ § Requirements-First）；IDE 按钮 **Proceed to Design**（…/specs/analyze-requirements/ § How to invoke it）；CLI "At each phase checkpoint, press Ctrl+X…"（…/specs/ § Getting Started） | 人 | 机械触发 + 散文内容（全站无 EARS lint） | 近 H1，但无冻结、无 digest、无回链 |
| 2 | design.md → tasks | `design.md`（可手改） | `F{Happy?} -->\|yes\| H["Implementation"]`（同上）；"structured phases with approval gates between each one"（…/custom-agents/built-in/ § Spec agents） | 人 | 机械触发 + 散文内容 | 近 H2 批准施工图，但无 Run Manifest |
| 3 | tasks.md → 开工 | `tasks.md` checkbox 清单 | 点单个任务 / **Run all Tasks**（"only runs incomplete tasks that are marked as required"，…/specs/best-practices/ § Can I execute all…）；CLI `/spec run <name>  # executes all tasks autonomously`；`PreTaskExec` hook 可 exit 2 阻断（IDE only，…/hooks/ § Available triggers） | 人 + 代码前置门 | 机械 | 近 H2「启动 Run」+ H4 派发；任务文本即全部边界 |
| 4 | 任务施工中 → `[x]` | 该行 checkbox；IDE 内部 in_progress/completed 状态机 | "Tasks are updated as in-progress or completed"（…/specs/ § Task Execution）；`PostTaskExec` "Can block? No"（…/hooks/ § Available triggers）；CLI 实测 "the [ ] checkboxes are never automatically marked [x]"（github.com/kirodotdev/Kiro/issues/6826）。**completed 由谁触发文档未写明** | 模型 | 散文判 + 机械载体 | H5 无对应：自述即结果 |
| 5 | 任务 → 下一 wave | 剩余 `[ ]` 且 required 的任务；依赖图不落盘 | "Kiro builds a dependency graph… groups independent tasks into waves"（…/specs/ § Running tasks in parallel） | 代码过滤 + 模型分析依赖 | 机械 + 散文 | 近 H4 逐个派发，无 Execution Spec |
| 6 | 施工完 → 评审/合入 | Web：feature branch + PR；IDE/CLI：直接落工作区 | Web "the agent opens a pull request"（…/specs/ § Task Execution，Web 页签）；IDE 默认 Autopilot "without asking for approval at each step"（…/ide/chat/autopilot/） | 人审 PR；Autopilot 下无人门 | 机械（PR 事实） | Web 的 PR 近 H6；IDE/CLI 无此边 |
| 7 | 需求变更 → 漂移修复 | 就地改写 `design.md`/`tasks.md` | **Sync Files** "Kiro will automatically mark completed tasks"——模型扫代码库自查（…/specs/best-practices/ § What if some tasks are already implemented?）；已完成任务无保护，steering 拦不住（issues/5019） | 模型 | 散文 | 无对应：HCTL2 用新 Task Revision，Kiro 就地覆写 |
| 8 | Quick Spec：意图 → tasks.md | 三文件一次生成 | "with no approval gates between phases"（…/specs/quick-spec/ § How it works）；不自动开工 | 人（前置问答）+ 模型 | 散文 | H1+H2 压成一次无冻结的「采纳」 |
| 9 | 会话 → 会话 | 三份 spec 文件；compaction 摘要 | "Kiro automatically includes all spec files… in the conversation context"（…/specs/best-practices/ § How do I reference a spec）；用户实测 "tasks.md drifting from actual implementation state"（issues/6826） | 无门 | 散文 | 无对应：HCTL2 不代管会话内上下文 |

判点：绕过开关全是产品一等公民——Quick Spec、Autopilot（默认）、`/spec run`、Run all Tasks、Web Autonomous；硬编码 always-ask 名单保护 `.git/**`、`.kiro/hooks/**`，**不保护 `.kiro/specs/**`**（…/permissions/ § Default behavior）。同站另一条产品线 Kiro Crew Task Runner 是反例：tests pass → commit；"Independent reviewer session reads the actual git diff (not the LLM's self-report)"；失败 `git reset --hard HEAD~1`；`force_approval` 即便 YOLO 也阻塞（…/crew/features/task-runner/ § The loop / § Force approval gates）；但留了 `--no-test` 与 "Review exceptions are non-fatal (returns 'passed' to avoid blocking)" 两个软点。

### 2.2 任务图驱动

#### beads（gastownhall/beads @ 40b32324，2026-09-01，MIT）

Dolt 后端的 issue 图数据库 CLI（Go）。`bd create` → `bd dep add`（19 种边，仅 4 种参与就绪归约）→ `bd ready` 用 SQL 归约出可领集合 → `bd update --claim`（行级 CAS + 5 分钟 node-local 租约）→ 施工 → `bd close`。gate 是一类 `type=gate` 的 issue，通过 `blocks` 边阻塞步骤。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | open → ready（可领） | `issues.is_blocked` 派生列 | `status IN ('open','in_progress') AND is_blocked = 0`（`internal/storage/sqlbuild/ready.go:105-110`）；参与词表 `AffectsReadyWork(): DepBlocks \|\| DepParentChild \|\| DepConditionalBlocks \|\| DepWaitsFor`（`internal/types/types.go:1300-1302`）；`waits-for` "DELIBERATELY OUTSIDE" 环检测集（`:1337-1361`） | 代码 | 机械 | 无对应：HCTL2 没有 Task 间依赖归约 |
| 2 | ready → in_progress（claim） | `assignee/status/started_at` + `row_lock` 代次 token（equality-only，`types.go:66-90`）+ node-local `leases` 行（不入 Dolt 历史，`:56-64`） | `UPDATE … WHERE id = ? AND row_lock = ? AND status IN (…)`（`internal/storage/issueops/claim.go:114-118`）；同 actor 重领幂等（`:147-148`）；`DefaultLeaseTTL = 5 * time.Minute`（`issueops/lease.go:26`） | 代码 | 机械 | H4 的反向：执行体 pull 式自领，无 Run 下达的 Execution Spec；行级 CAS + 代次 token 与 HCTL2 代次校验同构 |
| 3 | in_progress → 丢失 → open（租约回收） | 字段清空 + `ReclaimedLease` 事件 | `bd heartbeat` "Only the current owner may heartbeat. If the lease has already been reclaimed or the issue closed, heartbeat fails so the worker learns to stop."（`cmd/bd/heartbeat.go:24-25`）；`bd reclaim` "SKIPS a lease another replica granted"，跨节点须显式 `--any-replica`（`reclaim.go:45-46,166-167`） | 代码 | 机械 | H5 否定分支：租约过期 = 执行身份丢失、Task 回到可派发；跨节点接管必须显式 |
| 4 | → closed（关单） | `status=closed`、`close_reason`（自由文本，默认 `"Closed"`，`cmd/bd/close.go:449`）、`closed_by_session`；**不含 commit / PR 引用** | CLI 层 `NotTemplate() / NotPinned(force) / AssigneeMatches(actor, force)`（`cmd/bd/show_unit_helpers.go:30-34`）；"Authority is identity-by-string… bd has no identity layer"（`internal/validation/issue.go:154-158`）；未分配 issue 任何 actor 可关（`:170`）；引擎内 open-children / live-blocker 守卫，"force bypasses the child and blocker policies, never the version check"（`issueops/close.go:60-61,150-160`）；**`acceptance_criteria` 字段在关单路径未被读取**（grep 无命中） | 代码判形式 | 机械（形式） | H7 弱化版：只校验作者身份与图结构，不校验验收约束；`close_reason` 是自述 |
| 5 | 步骤阻塞 → gate 清除 → 步骤 ready | `type=gate` issue：`AwaitType ∈ {human, timer, gh:run, gh:pr, bead}`、`AwaitID`、`Timeout`（`types.go:158-161`） | 解决条件 "gh:run: status=completed AND conclusion=success / gh:pr: state=MERGED / timer: current time > created_at + timeout / bead: target bead status=closed"（`cmd/bd/gate.go:608-616`）；实现 `gh run view --json status,conclusion`（`:1066`）、`gh pr view --json state`（`:1127`）；human gate 无条件，`bd gate resolve` 直接 `CloseIssue` 不校验谁（`:564-569`）；`bd close <gate>` 对机器可检 gate 先 `checkGateSatisfaction`，但 "If we can't check the condition, allow close with a warning"（`close.go:575-581`） | timer / gh / bead：代码；human：任何 actor | 机械（四型）+ 无条件（human） | `gh:pr` ≈ H6 合入事实作门；`human` ≈ H9 但无「所需 actor」；**`gh:run` / timer 在 HCTL2 没有节点名字**（见 4.1 第 6 条） |
| 6 | formula → proto → molecule / wisp | proto = `IsTemplate` 只读 epic；molecule = parent-child 树 + `needs` → `blocks` 边（`cmd/bd/cook.go:643-648`） | `bd cook` + `bd mol pour --var`；默认不物化步骤 "only the root issue is created; steps are read inline at prime time"（`internal/formula/types.go:111-116`）；无人批准环节 | 代码（结构变换） | 机械 | ≈ G2 + H2 但无冻结 manifest：proto 是可编辑 issue 集合 |
| 7 | 步 n 关闭 → 步 n+1 领取 | 下一步 `status/assignee` | `bd close --continue` → `AdvanceToNextStep` 抢到即止（`cmd/bd/mol_current.go:633-656`）；"Ordinary epics remain open when all children finish so they can become explicitly close-eligible"（`close.go:596-598`） | 代码 | 机械 | ≈ Run reducer 派下一 Attempt；「根不自动关」与「Task 完成须独立命令」同向 |
| 8 | 会话 → 会话 | 无交接物；`bd prime` 输出散文清单 | `# 🚨 SESSION CLOSE PROTOCOL 🚨 … [ ] 1. bd close <id1> <id2> … [ ] 4. report handoff`（`cmd/bd/prime.go:719,787-790`） | 模型 | 散文 | 无对应：beads 的答案是「没有会话态，一切在 issue 里」 |

判点：就绪归约是唯一硬门，且刻意窄——19 种边只有 4 种影响 ready，`parent-child` 影响 ready 但不算 hard blocker，这是「结构 vs 阻塞」的干净切分。gate 是一等 issue，所以门与任务共用状态机和权限模型；代价是 human gate 没有「谁有权 resolve」。`bd gate check` 在 `gh` 不可用时降级放行——HCTL2 对同一情形的回答是类型化拒绝加需要关注（connections.md §失败与恢复）。

#### Taskmaster（eyaltoledano/claude-task-master @ c0c98d36，2026-04-23，MIT + Commons Clause）

PRD → `parse-prd` 生成 `tasks.json` → `analyze-complexity` → `expand` → `next` 机械选任务 → agent 实现 → `set-status done`。状态全部是 `tasks.json` 里的 `status` 字符串。另有 autopilot（TDD 状态机，状态在 `workflow-state.json`）与 loop（靠 stdout 哨兵退出）两条自动化回路。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | PRD → 任务清单 | `tasks.json` 目标 tag 整节覆写（`scripts/modules/task-manager/parse-prd/parse-prd-helpers.js:241-257`） | 唯一的门是 tag 非空："Use --force to overwrite or --append"（`:105`） | 人下命令；模型拆 | 机械门 + 散文内容 | H1 弱化：无冻结，`--force` 整节重写 |
| 2 | 任务清单 → 复杂度报告 → 子任务 | `task-complexity-report.json`（可再生）；`task.subtasks[]` | 无门；报告只是 `expand` 的输入（`expand-task.js:160,196`） | 模型 | 散文 | 无对应：施工图「建议稿」，无批准动作 |
| 3 | 选下一个任务 | 返回一个任务对象 | `depsSatisfied = fullDeps.every(depId => completedIds.has(…))`，排序 priority → 依赖数 → id（`find-next-task.js:55-67,109-128`） | 代码 | 机械 | 无对应：HCTL2 不做排程 |
| 4 | 任务施工完 → done | `task.status = newStatus`（`update-single-task-status.js:100-101`）；父任务 done 级联子任务 done（`:108-128`） | 代码只校验枚举（`set-task-status.js:35-39`）；`validateTaskDependencies` 调用后**结果丢弃**（`:125-127`）；MCP `set_task_status` 无身份/权限判断（`mcp-server/src/tools/set-task-status.js:29-57`）；验证全在 prompt："After verifying the implementation… mark the subtask as completed"（`assets/rules/dev_workflow.mdc:404`） | 模型自判 | 散文（代码只判枚举合法） | H7 缺失：执行体自述决定完成，无 Receipt |
| 5 | autopilot 启动 | `workflow-state.json`（原子写 + 备份，`workflow-state-manager.ts:129-187`）；分支 `tm/<ns>/task-<id>` | `ensureCleanWorkingTree()`（`workflow.service.ts:179`）；"Task has no subtasks. Expand task first."（`start.command.ts:89-96`）；`PreflightChecker` 存在但 autopilot start 不调用 | 代码 | 机械 | H2 近似：Manifest 极简化为 {taskId, subtasks, branch, tag} |
| 6 | RED → GREEN → COMMIT | `context.lastTestResults = {total,passed,failed,skipped}`（`workflow/types.ts:34-40`） | RED："Test results required"（`workflow-orchestrator.ts:164-166`）；`failed === 0` 视为 "feature already implemented" 直接标 completed（`:171-207`）；GREEN："must have zero failures"（`:227-230`）。数字来自 CLI `--results <json>` / MCP `testResults` 参数——**agent 自报**；`TestResultValidator` 有 setter 但 `handleTDDPhaseTransition` 从不调用（`:658-661` vs `:152-291`） | 代码判 JSON 字段；数字由模型自报 | 形式机械、实质散文 | H5 反例：自述被直接当 Evidence |
| 7 | FINALIZE → COMPLETE | 任务 `done`；删除 `workflow-state.json`（`workflow.service.ts:537-540`） | "Complete all subtasks first"（`:507-511`）；"working tree has uncommitted changes"（`:517-524`） | 代码 | 机械 | H7 最接近的一条：校验仅「工作树干净」，不按验收约束；不留 Run 记录 |
| 8 | loop 一轮 → 下一轮 / 退出 | 进度文件；`LoopIteration.status` | `<loop-complete>` / `<loop-blocked>` 哨兵正则（`loop.service.ts:289-305`）；prompt "Complete ONLY ONE task per iteration"（`loop/presets/default.ts:21`） | 代码判哨兵，模型输出哨兵 | 机械 + 散文 | 无对应；`<loop-blocked>` 接近 H9 触发点但只是退出 |

判点：独有的边是「测试必须先失败」作 RED 门（0 失败被解释为「已实现」并自动跳过）——但门的输入是自报数字，写好的校验器没接线。`validate-dependencies` 命令源码保留注释 `// process.exit(1); // Uncomment if validation failure should stop the process`（`scripts/modules/dependency-manager.js:641-642`）。

### 2.3 编排器

#### vibe-kanban（BloopAI/vibe-kanban @ 4deb7eca，2026-04-24 终版，Apache-2.0）

本地 Rust 服务管 Workspace（= 一次 attempt：worktree + 分支）、ExecutionProcess（setup → agent → cleanup 链）、PullRequest；看板卡与状态列在 remote 服务。卡片状态由三种事实归约：首个 workspace → In progress；PR open → In review；全部 PR merged（或直接 merge）→ Done。进程退出只发通知，不动卡。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Issue → Workspace + 首个 agent | `workspaces` 行 + worktree + `execution_processes` 行（`executor_action` JSON，`crates/db/src/models/execution_process.rs:62-77`）；每次执行存 `before_head_commit`/`after_head_commit`（`services/…/container.rs:1157-1166`） | 无前置门；"If this is the first workspace and the issue is in Backlog or To do, moves to In progress"（`crates/remote/src/db/issues.rs:638-648`）；prompt 默认 = issue 标题+描述（`crates/mcp/src/task_server/tools/task_attempts.rs:28`） | 人（UI）或模型（MCP `start_workspace`） | 机械 | H2+H4 合并：Execution Spec 就是 prompt 字符串；baseline sha 有记录，无批准 |
| 2 | setup → agent → cleanup | `ExecutorAction { typ, next_action }`（`crates/executors/src/actions/mod.rs:36-39`） | `status == Completed && exit_code == Some(0)`（`crates/local-deployment/src/container.rs:555-558`）；agent 无提交则跳过 cleanup（`:581-610`）；**宿主替 agent 提交未提交改动** `try_commit_changes`（`:570-580,1554-1578`） | 代码 | 机械（退出码 + git 状态） | 无对应：Run 内部 step 编排 |
| 3 | agent 进程结束 → 通知 | 桌面通知；无状态写入 | `finalize_task` 只 `notify`，"Skip notification if process was intentionally killed"（`services/…/container.rs:238-270`） | 代码 | 机械 | 与 H5 同向的「刻意不画边」：进程退出不构成结果 |
| 4 | 工具审批 / 提问 → 人答 | 内存 `PendingApproval { timeout_at }`（`services/approvals.rs:21-26`） | "Blocks until approved/denied/timed out"（`crates/executors/src/approvals.rs:41-46`）；超时默认 `TimedOut`（`:92`） | 人 | 机械（专用端点 + 超时） | H9：专用 approve 端点体现「普通聊天不算解决」 |
| 5 | 分支 → PR → In review | `pull_requests` 行 `{pr_status, merged_at, merge_commit_sha, synced_at}`（`pull_request.rs:10-23`） | 目标分支须在远端存在、push 成功、host `create_pr` 成功后才建行（`routes/workspaces/pr.rs:235-319`）；remote `ReviewStarted => "In review"`（`issues.rs:521,533`）；MCP 34 个工具中**无 PR/merge 工具** | 人触发；代码判 git/host 事实 | 机械 | In review 由集成事实投影，HCTL2 进入评审是 Gate |
| 6 | PR merged → Done + 归档 workspace | remote issue `status_id`；本地 `update_status(merged_at, merge_commit_sha)` | 60 秒轮询 git host（`pr_monitor.rs:68`）；"if all linked PRs are merged, move issue to Done"，`prs.iter().all(\|pr\| pr.status == Merged)`（`issues.rs:522,534-543`）；找不到名为 "Done" 的列则静默不动（`:546-550`） | 代码 | 机械 | H6（PR merged ≈ Integration Receipt）→ H7（Done 由归约得出）；但无「按验收约束独立校验」 |
| 7 | 后门：任意状态 ↔ 任意状态 | `issues.status_id` 直接改 | MCP `update_issue` 参数 `status: "New status name…"`（`remote_issues.rs:209-210`）；服务端只 `ensure_project_access`，无迁移校验（`routes/issues.rs:357-378`） | 模型或人 | 机械（无约束） | 违反 H7：模型可自标 Done，且覆盖 #6 的归约结果 |

判点：这是 19 家里最接近「进程退出不算结果、合入事实才算」的生产实现，PR 行的 `merged_at + merge_commit_sha` 是最小的 Integration Receipt。归约按状态列**名字**查找（`find_by_name(…, "Done")`），列名对不上就静默跳过——干净但脆弱。

#### Gas Town（steveyegge/gastown @ 649b832b，2026-07-23，MIT）

`bd` 之上的多 agent 编排层（Go CLI `gt` + 角色提示词 + TOML 巡逻 formula）。Mayor（Claude）`bd create` → `gt sling`（Go：spawn polecat、`bd update --status=hooked`）→ polecat（Claude）施工 → `gt done`（Go：校验身份、push、验证远端 tip == 本地 HEAD、建 MR bead 记 `commit_sha`）→ Refinery（Claude，按清单 rebase / 测试 / merge / push）→ `gt mq post-merge`（Go：merge proof 后关 MR 与源 issue）→ daemon 判 convoy 落地。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 用户请求 → work bead | `bd create` 的 issue（可变） | Mayor "1. Is it coordination? → Do it yourself. 2. Is it a code change? → File a bead, sling it. … If you're unsure, sling it."（`internal/templates/roles/mayor.md.tmpl:55-63`） | 模型 | 散文 | ≈ H1，无不可变 Revision |
| 2 | 一批 bead → staged → open convoy | convoy bead，状态 `staged_ready / staged_warnings / open / closed`；`tracks` 边（非阻塞） | `chooseStatus`：有 error 不建、有 warning → `staged_warnings`（`internal/cmd/convoy_stage.go:1320-1328`）；`gt convoy launch` 对 `staged_warnings` 须 `--force`（`convoy_launch.go:60-88`）；staged 期间 daemon 不 feed（`internal/convoy/operations.go:71-74`） | 代码（DAG 分析）+ 人 / 模型（`--force`） | 机械 | ≈ H2：stage = 批准施工图，launch = 启动 Run；只记 tracks 边，不冻结 baseline / 预算 / 参与者 |
| 3 | open bead → hooked（派发） | work bead `status=hooked`、`assignee=<rig>/polecats/<name>`；agent bead `working` | `BdCmd("update", beadID, "--status=hooked", "--assignee="+targetAgent)` 并回读（`internal/cmd/sling_helpers.go:1278,1304`）；"The hook is the 'durability primitive' - work on your hook survives session restarts, context compaction, and handoffs"（`hook.go:33-36`）；"polecats cannot hook work (use gt done for handoff)"（`:238`） | 代码 | 机械 | ≈ H4：hook bead ≈ Execution Spec 载体；push 式派发，比 beads 更接近 HCTL2 |
| 4 | 施工中步间推进 | 无步骤 bead；git 提交是唯一持久化产物 | "Exit criteria (HARD GATE): Implementation complete AND code committed to git"（`internal/formula/formulas/mol-polecat-work.formula.toml:291-294`）；"Scope is a contract… mail the mayor BEFORE implementing it… Wait for a response"（`:223-229`） | 模型自判 | 散文 | 无对应；「scope is a contract → mail mayor」≈ H9 但回复是普通邮件 |
| 5 | 施工完 → 交工（`gt done`） | MR bead：描述为 `key: value` 文本，含 `branch / target / source_issue / commit_sha / worker / …[/skip_verify]`（`internal/cmd/done.go:1721-1759`）；agent bead completion metadata | 身份校验 "gt done identity mismatch: BD_ACTOR=… but GT_ROLE/GT_RIG/GT_POLECAT resolve to…"（`done.go:160-170`）；"Verify the pushed branch tip is the exact local commit before creating any MR bead. Branch-exists checks are insufficient"（`:1438-1453`）；`--skip-verify` 仍建 MR，只写 `skip_verify: true`（`:1444-1445,1727-1729`）；提示词 "There is no approval step. There is no confirmation."（`polecat.md.tmpl:24-27`） | 何时 done：模型；能否 done：代码 | 机械 + 散文 | ≈ H5：MR bead ≈ Result Proposal（含 `commit_sha` 证据引用）；`--skip-verify` 把证据校验降为自述 |
| 6 | MR 排队 → 合并（Refinery） | target 分支 merge commit | **两套实现并存**：Go `Engineer.doMerge`（gates → `MergeNoFF` → push → `VerifyPushedCommit`，`internal/refinery/engineer.go:504-767`）但 `ProcessBatch` **无调用方**（仅 `internal/refinery/batch.go:202,211`）；生产路径是 Claude："**You are the decision maker.** All merge/conflict decisions are made by you, not Go code."（`refinery.md.tmpl:86`）；清单 "GATE REQUIREMENT: You CANNOT proceed to merge-push without: All quality checks and tests passing, OR Bead filed… for the pre-existing failure"（`mol-refinery-patrol.formula.toml:600-602`）；肉眼比对 SHA "If SHAs differ: STOP… DO NOT close MR bead"（`:673-678`） | 模型（按清单）；「测试红是否 pre-existing」由模型判 | 散文为主；Go 版机械但未接线 | ≈ H6 合入命令：由 Refinery agent 自行决定何时合入 |
| 7 | 已合并 → 机械关单（`gt mq post-merge`） | MR bead `closed`、`merge_commit`；源 bead `closed`，reason 含 `target_branch / commit_sha`；远端分支删除 | "1. Verify the target branch contains the submitted source head 2. Close the MR bead 3. Close the source issue 4. Delete the remote polecat branch at the submitted head"（`internal/cmd/mq.go:177-180`）；"merge proof failed for MR %s: target %s does not contain submitted head %s"（`:601-620`）；缺 `commit_sha` 直接失败（`:612-615`）；`ForceCloseWithReason` 绕过 bd 自身 open-children / blocker 策略（`internal/refinery/work_bead_close.go:82`）；"send MERGED only after gt mq post-merge succeeds"、"never manually close"（`mol-refinery-patrol.formula.toml:615,899`） | 代码 | 机械 | ≈ H6 Integration Receipt（合入事实 + commit SHA）与 H7 合为一步：合并即关 Task，无独立验收 |
| 8 | tracked 全部关闭 → convoy landed；否则 feed 下一 issue | convoy bead `closed`；通知 mail；下一 issue 被 `gt sling` | daemon 5 秒事件轮询（`internal/daemon/convoy_manager.go:348-386`）；"A 0/0 result means cross-rig tracking resolution failed — not that all issues are done"（`internal/cmd/convoy.go:1017-1022`）；`unknown` 保持 open（`:1027-1040`）；"Only one issue is dispatched per call."（`internal/convoy/operations.go:282`）；"Manual overrides (close --force, land) bypass the check entirely"（`docs/design/convoy/convoy-lifecycle.md:62`） | 代码 | 机械 | ≈ H7 归约 + H8 投影（"🚚 Convoy landed"）；feed-next ≈ Run reducer 派下一 Attempt |
| 9 | 执行体失联 → 重置或关（Witness） | work bead `open + assignee=""` 或 `closed`；`PatrolReceipt{Verdict: stale \| orphan, Evidence{…}}`（`internal/witness/patrol_receipts.go:23-29`） | orphan 路径 "Polecat is truly gone (no session, no directory). Reset the bead."（`internal/witness/handlers.go:2905-2990`）；重置前先 `verifyCommitOnMain`——HEAD 已是 main 祖先则**关单** "Work already on main (verified by witness…)"（`:2771-2779`）；该 Go 函数在 HEAD 只由 `gt up` 调用（`internal/cmd/up.go:1030-1046`），巡逻期间由 Witness agent 按清单手做同样的 `bd update --status=open --assignee=`（`mol-witness-patrol.formula.toml:40`）；zombie 路径 restart-first：有 pending MR 不动、分支已并入则 nuke、否则 `RestartPolecatSession`（`handlers.go:2001-2200`） | 检测：代码；处置：代码（orphan）或模型（巡逻清单） | 机械 + 散文 | ≈ H5 失败分支（丢失 → Attempt 作废）；「先查 main 再决定关或重置」= 用集成事实反推完成，HCTL2 不允许（Harness 改写目标引用不产生 Receipt） |
| 10 | 会话 → 会话（`gt handoff` / `gt prime` / `gt seance`） | handoff mail bead（hooked 给自己，`internal/cmd/handoff.go:1306-1343`）；`.runtime/handoff-marker` | "handoff mail failed to persist (Dolt may be down)" 则不重启（`:298-308`）；继任者 `gt prime --hook` 三段判态（`prime_session.go:284-361`）；`gt seance --talk` → `claude --fork-session --resume <id>`（`seance.go:57-58`） | 何时切：模型；搬运与恢复：代码 | 散文 + 机械 | 无对应；但 hook bead「工作挂在 Task 上、会话可死」与 HCTL2「结果归 Task 不归会话」同向 |
| 11 | 角色间交接（nudge / mail） | nudge（tmux 注入，零存储）、mail bead（Dolt） | "If the recipient dies and restarts, do they need this message? If yes → mail. If no → nudge."（`witness.md.tmpl:209`）；Deacon 重派发限流 `DefaultMaxRedispatches = 3`、cooldown 5 分钟（`internal/deacon/redispatch.go:20-28`） | 发信号：代码；处置：模型 | 机械信号 + 散文处置 | ≈ H8 + H9（escalation ≈ Request，解决是普通邮件） |

判点：Gas Town 是全场唯一一处「交付物 = 可独立复核的事实引用」：`commit_sha` 从 `gt done` 写进 MR bead，`gt mq post-merge` 用它做 merge proof，关单 reason 再写回 `target_branch / commit_sha`，分支删除也钉在 `expectedHead`。两处要校准 landscape 的说法：（一）「机械关单」只覆盖关单那一步——合并决策本身是 Claude 按清单手打 `git merge --no-ff`，Go 合并管道 `ProcessBatch` 在 HEAD 无调用方，「测试红是分支引起还是 pre-existing」这道最关键的门是散文门；（二）Witness 在 HEAD 是 restart-first 而非 reset-first，重置只发生在会话与目录都没了的 orphan 路径，且先查 main、有证据就关单。绕过口：`gt done --skip-verify`、polecat 直接 `bd close`（只有提示词禁止）、`gt convoy close --force`。

### 2.4 轻量纪律与重型 spec 驱动

#### agent-os（buildermethods/agent-os @ 475b0cac，2026-08-29，MIT）

v3 只剩五个 slash command；工作流阶段（写 spec、拆任务、实现、编排、验证）全部退出，CHANGELOG 明说让渡给 harness："Spec writing — Now best handled using Plan mode / Task breakdown — Tools like Claude Code automatically create and track todo lists / Implementation orchestration — Frontier models manage task delegation on their own"（`CHANGELOG.md:21-23`）。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 代码库 → 标准文件 | `agent-os/standards/<folder>/<name>.md` | 每条走完整回路 "Ask 1-2 clarifying questions about the 'why'… Confirm with user before creating the file"（`commands/agent-os/discover-standards.md:72-78`） | 人逐条批准 | 散文 | H8 Memo 沉淀的散文版 |
| 2 | 标准文件 → 索引 | `index.yml`（`folder: file: description:`，无版本） | 新条目描述要人认；陈旧条目 "Remove them from the index automatically (no confirmation needed)"（`index-standards.md:57-60`） | 人 + 模型 | 散文 | 无对应：Memo 的检索层 |
| 3 | 索引 → 注入上下文 | 对话内全文块 / `@` 引用行 / 复制进 skill | 匹配后仍要人认 "Inject these standards? (yes / just 1 and 3 / …)"（`inject-standards.md:80-88`） | 模型匹配 + 人确认 | 散文 | Context Manifest 的散文版：不冻结、不记录 |
| 4 | shape-spec → 计划提交 harness 审批 | `agent-os/specs/<ts-slug>/{plan,shape,standards,references}.md`——**全部由执行阶段的 Task 1 写入** | "must be run in plan mode… If NOT in plan mode, stop immediately"（`shape-spec.md:13-23`，是否在 plan mode 由模型自报）；"Task 1 always being 'Save spec documentation'"（`:127`）；批准键是 harness plan mode 的 approve，仓库无代码看到它 | 人（在 harness 里 approve） | 散文 | H1→H2 合并成 harness 的一次 approve；被批准的东西在批准时不存在于磁盘 |
| 5 | 计划批准 → 实现 → 完成 | （无）v3 不提供 | "Implementation/orchestration phases retired—frontier models handle this well on their own now"（`CHANGELOG.md:41`） | harness | — | H4/H5/H6/H7 全部让渡 |

判点：v2.1 → v3 砍掉 write-spec / create-tasks / implement-tasks / orchestrate-tasks 以及 v2.1 还保留的 "final overall verification step"（`CHANGELOG.md:72-78,133`）。仓库内不再有任何「完成」概念。这是行业信号：流水线段被 harness 吞掉，活下来的是 harness 之外的资产（标准）。

#### GSD（open-gsd/gsd-core @ fa107c04，2026-09-02，MIT；旧 gsd-build/get-shit-done @ bdcaab2c 已归档）

`new-project` → 每阶段 `discuss-phase`（CONTEXT.md）→ `plan-phase`（PLAN.md，含 `must_haves`）→ `execute-phase`（每 plan 一个 executor 子代理，逐任务 commit，产 SUMMARY.md；末尾 verifier 产 VERIFICATION.md）→ `verify-work`（UAT）→ `phase.complete` → `transition`。状态全在 `.planning/` 的 Markdown frontmatter + git 提交；编排文本是 prompt，`gsd-tools.cjs` 提供确定性查询/写入。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 无项目 → 已定义 | PROJECT.md / REQUIREMENTS.md / ROADMAP.md（每阶段 Success Criteria）/ STATE.md，每步一 commit | roadmap 门 "Ask for approval before committing… Loop until user approves"（`workflows/new-project.md:1231-1270`）；auto："Skip approval gate — auto-approve and commit directly"（`:1229`） | 人 / 模型（auto） | 散文 | H1 部分对应：阶段 = Task 草案（目标 + Success Criteria），可变、无冻结 |
| 2 | 上下文锁定 → 计划就绪 | `PLAN.md` frontmatter `must_haves: truths/artifacts/key_links`（`templates/phase-prompt.md:556-602`）；任务 `<verify>/<done>` | plan-checker 修订环 "Max 3 Iterations"（`plan-phase.md:1197`）；REQ 覆盖门用 grep 对比（`:1344-1388`）；已 Complete 阶段重规划要 `--force`（`:109-111`）；"an entry whose severity is missing or unrecognized counts as a BLOCKER (fail closed)"（`:1205`） | 模型（planner 写、checker 判）；人只在上限时三选一 | 机械（REQ grep、exit 1）+ 散文（质量判定） | H2「批准 Workflow」：批准者是模型，PLAN 可再编辑，退役只靠 `status: superseded` |
| 3 | 计划就绪 → 派发执行体 | 执行体 prompt；worktree 分支 | safe_resume_gate："derive CURRENT_PLAN_ID… git log --grep… If production commits exist and SUMMARY.md is missing… stop before spawning"（`execute-phase.md:181-195`）；isolation hook "HARD-BLOCKING… executor runs and commits directly in the user's PRIMARY checkout"（`hooks/gsd-agent-isolation-guard.js:12-24`） | 代码（hook）+ 模型执行 bash 对账 + 人三选一 | 机械 + 散文 | H4：PLAN + prompt ≈ Execution Spec；safe_resume_gate ≈ 派发前按 git 对账 |
| 4 | 任务 → 下一任务（执行体内） | 每任务一 commit `{type}({phase}-{plan}): …`（`agents/gsd-executor.md:544`） | "After each task completes (verification passed, done criteria met), commit immediately"（`:446`）；前置未满足 "STOP — return a checkpoint… NEVER auto-approved, even under AUTO_CFG=true"（`:151`） | 模型 | 散文；commit 是机械载体 | 无对应：Attempt 内部步骤 |
| 5 | 执行体 → 人（checkpoint 三型） | `## CHECKPOINT REACHED` 块；恢复用 fresh continuation agent | 三型均 `gate="blocking"`；auto："human-verify auto-approves, decision auto-selects first option, human-action still stops"（`references/checkpoints.md:11`）；`gate="blocking-human"` "never auto-approved"（`:12`，旧仓库无此型） | 人 / 模型（auto 自批） | 散文 | H9：checkpoint ≈ Request，但聊天回复 "approved" 即算解决，auto 下模型自解 |
| 6 | 计划完成 → 已记录 | `SUMMARY.md`（`status: complete`）+ `## Self-Check: PASSED` | "SUMMARY.md claims" 由编排器抽查：文件存在 + `git log --grep` ≥1（`execute-phase.md:1070-1075`）；`verify-summary` CLI 自认 "unacceptable as a gate"（`src/verify.cts:103-106`） | 模型自检 + 模型跑 bash 抽查 | 机械 + 散文 | H5：SUMMARY ≈ Result Proposal；抽查强度只到「存在 + grep」 |
| 7 | 所有计划完成 → 阶段目标已验证 | `VERIFICATION.md` frontmatter `status: passed \| gaps_found \| human_needed`（`templates/verification-report.md:10-15`） | "SUMMARY.md claims are not evidence"（`agents/gsd-verifier.md:15`）；"Assume the phase goal was not achieved until codebase evidence proves it"（`:26`）；三级 exists → substantive → wired（`:243-288`）+ data-flow；`verify.artifacts`/`verify.key-links` 由代码给 exists/pattern；"passed is ONLY valid when the human verification section is empty"（`:553`） | 模型（verifier）判 verdict；代码给 L1-L2 结果 | 机械（L1-L2、路由三值）+ 散文（L3-L4、truths） | H5 归约校验一侧 + H7 前置：校验者本身是模型 |
| 8 | 已验证 → 阶段完成 | ROADMAP `[x]`+日期；`phase.complete` 返回 JSON | 代码门 1："refuse completion when any plan lacks a matching *-SUMMARY.md, UNLESS… status: superseded"（`src/phase.cts:3311-3320`）；门 2：`if (verificationStatus.status !== 'passed') return`（`:3533`）；stale 检测（SUMMARY 比 VERIFICATION 新，`src/verification.cts:18-23`）；`next --force` 绕过 `next` 的门但绕不过 `phase.complete` | 代码 | 机械 | H7：全场最接近「独立校验后出回执」的代码动作 |
| 9 | human_needed → UAT → 完成 | `UAT.md`（每条 `result: pass/skipped/blocked/issue`） | 人应答词匹配 "Empty response, 'yes', 'y', 'ok', 'pass'…"（`verify-work.md:364-365`）；零 issue 时工作流**改写** VERIFICATION frontmatter `status → passed`（`:621-623`）再过 `uat-passed` 谓词（`:639`） | 人 → 模型改写 → 代码谓词 | 机械 + 散文 | H9 + H7：完成回执的输入被模型改写 |
| 10 | 会话 → 会话 | `HANDOFF.json`（含每任务 commit、`uncommitted_files`）+ `.continue-here.md`；"Consumed exactly once — on resume"（`docs/reference/planning-artifacts.md:118-126`） | 恢复对账 "Validate uncommitted_files against git status"（`resume-project.md:109`）；`next` Gate 1 硬停直到删除 `.continue-here.md`（`next.md:45-60`） | 模型 + 代码（文件存在） | 机械 + 散文 | 无对应：HANDOFF 可变、消费一次即删，与 Context Manifest 相反 |

判点：唯一的代码硬门是 `phase.complete`，但它的两个输入都可被模型改写（verify-work 直接 `frontmatter.set … passed`；`status: superseded` 是「committable, review-time-trusted bypass」，`phase.cts:3370-3371`）。`--auto` 链下从 discuss 到 transition 人只剩四个位置：`blocking-human` 门、`human-action` 认证门、verifier 给 `human_needed`、`gaps_found` 停链。真正注册的硬拦 hook 只有两个（写保护、执行体隔离，`hooks/hooks.json`）；phase-boundary 与 validate-commit 是 opt-in 且未注册。

### 2.5 流程 / 角色模拟与群体自治

#### BMAD（bmad-code-org/BMAD-METHOD @ 891c0abb，2026-09-02，MIT）

四阶段文档链（分析 → 规划 → 方案 → 实施）全部由 Markdown / YAML 承载：规划文档 frontmatter `status: draft|final` + append-only `.memlog.md`；实施用 `epics.md` → `sprint-status.yaml`（`backlog/ready-for-dev/in-progress/review/done`）→ 每个 story 一个 spec 文件。人介入点全是聊天菜单。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 规划文档 draft → final | `prd.md` / `ARCHITECTURE-SPINE.md` frontmatter `status`；`.memlog.md` append-only | 架构先跑确定性 lint "`lint_spine.py` … settles the mechanical misses (placeholders, duplicate AD IDs, missing Binds/Prevents/Rule, unpinned Stack versions)"（`src/bmm-skills/plan/bmad-architecture/references/reviewer-gate.md:5`），再 subagent 评审，"Headless never skips the gate"（:7）；headless 歧义 → `status: "blocked"`（`plan/bmad-prd/references/headless.md:29`） | 模型 + 人逐项拍板 | 机械 lint + 散文评审 | H1 弱对应：终稿无不可变 Revision；memlog 是台账不是冻结 |
| 2 | PRD → `epics.md` | 可变 Markdown | "FORBIDDEN to load next step until user approves epics_list"（`plan/bmad-create-epics-and-stories/steps/step-02-design-epics.md:38`）；每个 story "Ask: 'Does this story capture the requirement correctly?'"（`step-03-create-stories.md:159-168`） | 人（聊天回 C） | 散文 | H1 近似（story = 候选 Task），无冻结 |
| 3 | `SPEC.md` → `stories.yaml` | 字段 `spec_checkpoint / done_checkpoint / invoke_dev_with`；"No `status` field, ever."（`plan/bmad-spec/assets/stories-schema.md:20`） | "ask the user for `spec_checkpoint`, `done_checkpoint`… rather than defaulting them silently; capturing that human judgment is what the fields are for"（`bmad-spec/SKILL.md:136`）；build-auto "never read the checkpoint fields"（`ship/bmad-build-auto/step-01-clarify-and-route.md:29`） | 人 + 模型 | 散文 | H2 部分：「此处必须有人」的显式布尔，由外部编排器读 |
| 4 | `epics.md` → `sprint-status.yaml` | 脚本原子写；状态单调 "never downgrade"（`references/generate-tracking.md:16`） | 就绪门 "could a developer implement these epics without inventing decisions nothing records?"（`references/readiness-gate.md:7`）PASS / CONCERNS / FAIL | 门 = 模型；生成 = 代码 | 门散文，生成机械 | Task 登记簿（H1-lite）：一行一 story，不回退 |
| 5 | story 意图 → spec `draft` → `ready-for-dev`（CHECKPOINT 1） | `spec-{slug}.md`：`<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">` 包住 Intent / 边界 / IO 矩阵（`ship/bmad-build/spec-template.md:16-44`）；其余段可变；`## Spec Change Log` append-only | "The spec cannot leave `draft` while any entry remains [in Open Questions]"（`spec-template.md:50-51`）；"HALT for the human's answers. Write each answer into the `<frozen-after-approval>` block"（`step-02-plan.md:34`）；"Set status `ready-for-dev`; everything inside `<frozen-after-approval>` is then locked and only the human can change it"（:36-58）。**但**路由门 in-session 路径由模型自判 "If there are no intent gaps, nothing irreversible, and the change is small… Set `route: 'in-session'` and `status: 'in-progress'`… EARLY EXIT"（`step-02-plan.md:15-20`）——不经人批准直接施工 | 人批准；in-session 路由模型自判 | 散文（冻结只靠 "read-only. Do not modify"，`step-03-implement.md:11`，无 hash） | H1+H2 合一：frozen 块 ≈ Task Revision，其余段 ≈ 施工步骤；无 digest、无版本号 |
| 6 | `ready-for-dev` → `in-progress` → 派发 → 收回 diff | spec frontmatter `baseline_commit`；`{diff_file}` | "Capture `baseline_commit` (current HEAD…) before making any changes… If the frontmatter already contains `baseline_commit` (resumed run), preserve the existing value — never overwrite it"（`step-03-implement.md:21`）；子代理 "Read {spec_file} fully and implement it — the spec is the sole source of truth"（`customize.toml:77-80`）；"Judge against the diff, not against the implementation subagent's report"（`step-03-implement.md:41`）；"A covering test that exists but did not run… counts as missing. If a test disagrees with the matrix, never edit the expectation to match the code"（:47） | 模型（父会话审 diff） | 机械（baseline、diff）+ 散文（对照 AC） | H4 + H5 原则一致（看 diff 不看自述），归约仍是模型 |
| 7 | `in-progress` → `in-review` → 回环 | `## Review Triage Log`；`review_loop_iteration` 自增 | 三层评审 subagent；路由 `intent_gap`（根因在冻结块内 → 回人）/ `bad_spec`（根因在块外 → 模型改 spec 重推导）/ `patch` / `defer`（`step-04-review.md:57-65`）；"If it exceeds 5, HALT and escalate to the human"（:62） | 模型；人只在 intent_gap 与 >5 次时出场 | 散文 verdict + 机械计数 | H5 归约角色由模型担；intent_gap 回人 ≈ H9（聊天 HALT，无阻塞对象） |
| 8 | `in-review` → spec `done` → 本地 commit | spec `status: done`；sprint `review` | "Change `{spec_file}` status to `done`"（`step-05-present.md:53`）；"NEVER auto-push."（:9）；"Offer to push and/or create a pull request"（:72） | 模型写 done；人决定 push | 机械 commit + 散文 | H6 缺位：本地 commit 不是 Integration Receipt |
| 9 | sprint `review` → `done`（code-review 独立会话） | story 文件 `### Review Findings`；sprint-status 手改 | "If all `decision-needed` and `patch` findings were resolved… set `new_status` = `done`"（`ship/bmad-code-review/steps/step-04-present.md:91-92`）；"HALT — I am waiting for your numbered choice"（:51） | 人决定 decision-needed；模型算 new_status 并手改 YAML | 散文 | H7 弱对应：推 `done` 的是评审会话，不按 story 验收约束独立校验 |
| 10 | 无人值守一轮（build-auto） | 同一 spec；`<intent-contract>` 取代 frozen 块；`## Auto Run Result`；intent_gap 时保存 patch 文件 | "If intent gaps exist, do not fantasize and do not leave open questions… HALT with status `blocked`"（`bmad-build-auto/step-02-plan.md:14`）；"write `status: done`… 1. Commit… Do not push. 2. Verify the version-controlled working copy is clean. Otherwise HALT with status `blocked`"（`step-04-review.md:111-116`）；编排器 "Read `status`, `blocking condition`… rather than inferring success from chat output alone"（`docs/build/autonomous-development-loops.md:277`）；"A `blocked` story file is permanent… To retry, delete the story file"（:78） | 模型全程自判；编排器只读 frontmatter | 机械枚举 + 散文 verdict | H5：frontmatter ≈ Result Proposal，`done` 仍是自述；`blocked` + patch 文件 ≈ Request 的证据包 |
| 11 | epic 全部 `done` → retro 验收 | `epic-{N}-retro-{date}.md` frontmatter `verdict: accepted \| accepted-with-open-items \| rejected` | 三条硬规则 "1. A human decision always overrides the machine verdict. 2. An epic that fails its criteria with no human decision is recorded as not accepted — never as silently accepted. 3. A non-empty `pending_stories` list makes the machine verdict rejected, including in headless mode."（`ship/bmad-retrospective/references/acceptance-verdict.md:49-53`）；`pending_stories` 由脚本算（`scripts/sprint_status.py:309`）；"The script writes no verdict of any kind into `sprint-status.yaml`"（`retro-document.md:23`） | 代码（pending_stories）+ 模型（criteria）；人可覆盖 | 机械 + 散文 | H7（epic 粒度 Completion Receipt = retro 文档 frontmatter）；人类否决位显式 |

判点：BMAD 是唯一把「人拥有的意图」和「模型拥有的施工段」写进同一文件并用标签隔开的，且用 `intent_gap` / `bad_spec` 区分回环去向——根因落在冻结块内就回人，落在块外就模型自改。但冻结靠 prompt，没有 hash；「是否需要人批准」本身由模型的路由门判（in-session 路径）。`sprint-status.yaml` 被四个技能写：脚本生成、build 手改、code-review 手改、retro 脚本——retro 明令 "Do not hand-edit"（`retro-document.md:40`），build / code-review 却靠 prompt 手写。英文 `docs/reference/build-auto.md` 在 HEAD 不存在，现址 `docs/build/autonomous-development-loops.md`。

#### MetaGPT（FoundationAgents/MetaGPT @ 11cdf466，2026-01-21，MIT）

默认 CLI 组建 `Team(MGXEnv)`，雇 TeamLeader / ProductManager / Architect / Engineer2 / DataAnalyst；`ProjectManager` 未雇，`Engineer` / `QaEngineer` 的雇佣被注释（`metagpt/software_company.py:45-62`）。论文的 SOP 瀑布在默认路径是休眠的：`use_fixed_sop: bool = False`（`metagpt/roles/di/role_zero.py:98`）。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | idea → TL 建 Plan | 内存 `Plan{tasks, current_task_id}`（`metagpt/schema.py:496-503`），可 `append/reset/replace` | "The standard software development process has four steps… You may choose to execute any of these steps."（`metagpt/prompts/di/team_leader.py:17`）；"For XS and S requirements… directly ask Engineer to write the code"（:25） | 模型 | 散文 | H1 弱对应：Plan 可被后续命令改写 |
| 2 | 当前任务 → 派发成员 | `UserMessage(send_to=<name>)` 纯文本 | "DONT omit any necessary info… because you are their sole info source"（`team_leader.py:77-78`） | 模型 | 散文 | H4 退化为一条自由文本消息 |
| 3 | 成员收到 → 反应（watch/publish） | `rc.msg_buffer` 过滤 | `n.cause_by in self.rc.watch or self.name in n.send_to`（`metagpt/roles/role.py:411`）；SOP `_watch` 前注释 "will only be effective when self.use_fixed_sop is changed to True"（`metagpt/roles/architect.py:46`） | 代码（地址匹配） | 机械寻址 | 无对应：默认路径下 watch/publish 不是阶段门 |
| 4 | 成员干完 → 自述 → TL 收 | `AIMessage("I have finished the task, please mark my task as finished…")`（`role_zero.py:297-301`） | TL "If a team member has finished a task… mark the current task as completed."（`team_leader.py:14`） | 模型信模型 | 散文 | H5 反例：纯自述，无 Evidence |
| 5 | 当前 Task → 下一 Task | `Task.is_finished=True`（`schema.py:662-666`） | 只在 LLM 输出 `Plan.finish_current_task` 命令时执行（`role_zero.py:424-426`）；三次重复输出被代码强制转 `end`（`metagpt/utils/role_zero_utils.py:79-83`）；PLAN_AND_ACT 模式 "if auto mode, then the code run has to succeed for the task to be considered completed"（`metagpt/strategy/planner.py:128-129`） | 模型决定、代码记录 | 散文决策 + 机械翻转 | H7-lite：完成 = LLM 说完成 |
| 6 | 人类介入位 | stdin 文本 | `ask_human` 只在模型决定问时（`role_zero.py:461-462`）；`AskReview` 只在 `auto_run=False`，而 RoleZero 强制 `Planner(…, auto_run=True)`（`role_zero.py:114`）；否决词只有 `exit` / `stop`（`metagpt/actions/di/ask_review.py:15,55`；`role_zero.py:435`） | 人（仅当模型决定问） | 散文 | H9 弱对应；默认路径**无人类否决位** |
| 7 | 轮次 / 预算耗尽 → 停 | `Team.serialize` 全量 dump | `while n_round > 0`、`NoMoneyException`（`metagpt/team.py:99-134`）；默认 `n_round=5`（`software_company.py:16`） | 代码 | 机械 | 无对应：硬停无回执 |
| 8 | 代码 → 评审 / 测试 / PR | Engineer2 工具 `CodeReview`、`git_create_pull` | `--code-review` / `--run-tests` 只在被注释的雇佣分支使用（`software_company.py:56-62`） | 模型 | 散文 | H6 缺位 |

判点：`finish_current_task` 的布尔由代码翻，触发权在 LLM，TL 依据成员一句 "I have finished the task" 就翻别人的任务。唯一的机械门是 `n_round` / 预算、git `changed_files` 增量检测、重复响应检测——全是对模型「卡住」的兜底，不是验收。

#### ruflo（ruvnet/ruflo @ 4d0134e5，2026-09-01，MIT，v3.38.20）

两套互不共享状态的实现：TS 库 `v3/@claude-flow/swarm`（内存态 `TaskOrchestrator` 状态机、`ConsensusEngine`）与 Claude Code 实际调用的 MCP 工具（把状态写在 `.claude-flow/*.json`）。Claude Code 只碰 MCP 层。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 任务状态机（TS 库） | 内存 `Map<TaskId, TaskDefinition>`，进程内不持久 | `completeTask` "if (task.status !== 'in-progress') throw"（`v3/@claude-flow/swarm/src/coordination/task-orchestrator.ts:221`）；触发源是 `agent:task-completed` 事件，`success: true` 硬编码（:555-559） | 代码状态机；输入是 agent 事件 | 机械转移、零内容校验 | H5 反例：agent 事件即自述 |
| 2 | 任务创建 → 完成（MCP，实际运行时） | `.claude-flow/tasks/store.json` | `task_complete` "task.status = 'completed'; task.progress = 100; … task.result = input.result \|\| {}"（`v3/@claude-flow/cli/src/mcp-tools/task-tools.ts:252-257`）——无前置、无调用者校验；CLI `task` 子命令**没有 complete**（`cli/src/commands/task.ts:69-706`） | 模型 | 机械写入 | H7：完成 = 模型写 JSON |
| 3 | 提案 → 投票（TS 库 consensus） | 内存 `ConsensusProposal` | raft 默认阈值 0.66（`swarm/src/types.ts:433`；`consensus/raft.ts:460-467`）；leader 自投 `approve: true, confidence: 1.0`（:208-213）；`coordinateConsensus` 在非测试代码里**无调用点**（`queen-coordinator.ts:1698`） | 代码计票 | 机械 | 无对应；consensus 不在任务生命周期路径上 |
| 4 | 提案 → 投票（MCP `hive-mind_consensus`） | `.claude-flow/hive-mind/state.json` | raft `floor(n/2)+1`、bft `floor(2n/3)+1`（`hive-mind-tools.ts:82-97`）；`totalNodes = state.workers.length \|\| 1`（:552）——无 worker 时一票即过；`voterId` 是自由字串入参（:536）；CLI `--require-consensus` 只写成 tag `consensus:required`，`cli/src` 无消费者（`cli/src/commands/hive-mind.ts:1021-1047`） | 模型以任意 voterId 投；代码计票 | 机械计票 + 散文投票 | 无对应；consensus 与任务完成之间没有连线 |
| 5 | SPARC 阶段 N → N+1 | `memory_store` KV（`sparc-phases/spec-{slug}` 等），默认 upsert "writing an existing key updates it"（`memory-tools.ts:313,347`） | 门是 prompt："Phase 1 gate: Verify spec has >= 3 acceptance criteria…"（`plugins/ruflo-sparc/commands/ruflo-sparc.md:35-39`）；"`sparc phase <phase-name>` Jump to a specific phase… Store warning if jumping forward (skipping gates)"（:50-55） | 模型自评 | 散文 | H1/H2 形似，载体可 upsert，无冻结 |
| 6 | 会话结束 → 下次 | `.claude-flow/sessions/current.json`；AgentDB | `hooks_session-restore` 返回计数而非恢复对象 `tasksRestored: … Math.min(taskEntries, 10)`（`hooks-tools.ts:2497`） | 代码记录 | 机械记录 | 无对应 |
| 7 | pre/post-task、post-edit 钩子 | AgentDB 学习记录 | `hooks_post-task` "quality = (params.quality) \|\| (success ? 0.85 : 0.3)"（`hooks-tools.ts:1543-1545`）；hooks "always exits 0 so a CLI/install failure never surfaces an error… or blocks a turn"（`.claude-plugin/hooks/hooks.json:2`） | 代码 | 机械**记录**，无一处阻塞 | 无对应（不是门） |
| 8 | Stop / 权限 | hook JSON | Stop 是 prompt 型："Evaluate if the current task has been completed successfully… Respond with {"decision": "stop"} if complete"（`plugin/hooks/hooks.json:166-167`）；`PermissionRequest` matcher `^mcp__claude-flow__.*$` → `{"decision": "allow"}`（:198-203） | 模型自评；代码自动放行 | 散文 + 机械放行 | 把 H9 的人类授权位自动化掉了 |

判点：库层 consensus 有 2439 行实现但没有非测试调用方；MCP 层 `--require-consensus` 留了个没人读的 tag。结论：**consensus 不构成任何阶段边界**。人的位置：没有——MCP 工具权限被钩子自动 allow，所有 hook 永远 exit 0。landscape 的暂缓判断维持。

### 2.6 新七族代表

#### ralph（snarktank/ralph @ 6c53cb0b，2026-02-01，MIT）

`ralph.sh` 是 `for i in $(seq 1 $MAX_ITERATIONS)` 的 bash 循环（`ralph.sh:84`），每轮用干净上下文起 `claude --dangerously-skip-permissions --print`（`:95`），把 `prompt.md` 整篇喂进 stdin。状态落在 `prd.json`（每条 story 的 `passes` 布尔）、`progress.txt`（只追加）、git 提交。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | PRD → `prd.json` | JSON，`passes:false`，可变 | "Each story must be completable in ONE Ralph iteration (one context window)"（`skills/ralph/SKILL.md:48`） | 人调 skill，模型转换 | 散文 | H1：story = Task，无冻结、无 digest |
| 2 | 一轮开始 → 选 story | 无产物 | "Pick the highest priority user story where `passes: false`"（`prompt.md:10`）；"Work on ONE story per iteration"（`:105`） | 模型 | 散文 | H4：每轮 = 干净上下文的 Attempt，Execution Spec = prompt.md 全文；无 Attempt 记录 |
| 3 | 施工 → 检查 → commit | 本地 commit，**未推送** | "Run quality checks… If checks pass, commit ALL changes"（`prompt.md:12-14`）；仓库内无 push/PR 步骤 | 模型自己跑并解读 | 散文 | H5：commit 是自述结果；H6 完全在工具之外 |
| 4 | story 完成 → `passes:true` + progress | `prd.json` 就地改写；`progress.txt` 追加 learnings | "Update the PRD to set `passes: true`"（`prompt.md:15`）；"Append your progress"（`:16`） | 模型自标 | 散文 | H7：执行体自述完成——HCTL2 明文排除的形态；H8：learnings 模型自写自读 |
| 5 | 一轮结束 → 下一轮 / 退出 | 模型 stdout 的一句话 | 模型侧 "If ALL stories are complete and passing, reply with: <promise>COMPLETE</promise>"（`prompt.md:96-99`）；bash 侧 `grep -q "<promise>COMPLETE</promise>"` 则 `exit 0`（`ralph.sh:99-103`）；跑满 `exit 1`（`:110-113`）；**bash 从不读 prd.json** | 模型决定说不说；bash 检测 | 机械 token 检测 + 散文判断 | Run 完成信号 = 模型一句话，非归约 |
| 6 | 上一轮 → 下一轮（记忆） | `progress.txt`、`prd.json`、git log | "Read the progress log… (check Codebase Patterns section first)"（`prompt.md:8`）；"Each iteration spawns a fresh AI instance… with clean context"（`AGENTS.md:44`） | 代码只把文件留在原地 | 机械存在 + 散文使用 | Context Manifest 的替身：无清单，靠约定文件名 |

判点：施工模型同时是实现者、验收者（`passes:true`）和退出裁判（`COMPLETE`）。两种失配都无人检查：模型说 COMPLETE 但仍有 `passes:false`（直接 `exit 0`）；全部 `passes:true` 但没说这句话（跑到上限 `exit 1`）。工具崩溃被 `|| true` 吞掉继续循环（`ralph.sh:92,95`）。绕过开关：不需要——本来就没有门。

#### claude-mem（thedotmack/claude-mem @ e5b6719f，2026-09-02，Apache-2.0）

Claude Code 插件，五类 hook：`SessionStart` 注入上下文、`UserPromptSubmit` 建会话行、`PostToolUse *` 异步上报工具调用、`PreToolUse Read` 注入文件历史、`Stop` 请求进度小结（`plugin/hooks/hooks.json:17-87`）。工具事件由**另一个旁观模型**压缩成 XML `<observation>`，代码解析后写 SQLite；下次 `SessionStart` 查最近 50 条观察 + 10 条小结注入。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 会话开始 / compact → 注入 | 渲染文本，每次即时查询生成，不落盘 | matcher `startup\|clear\|compact`（`hooks.json:19`）；`ORDER BY created_at_epoch DESC LIMIT 50`（`ObservationCompiler.ts:60-61`；默认值 `SettingsDefaultsManager.ts:136,166`） | 代码 | 机械（无条件） | 形似 Context Manifest 但是查询结果，不冻结 |
| 2 | 事件批 → `observations` 行 | SQLite 行，可删（`DataRoutes.ts:93`），无版本链 | 旁观模型 "You are a Claude-Mem, a specialized observer tool…"（`plugin/modes/code.json` `system_identity`）；代码解析 XML（`ResponseProcessor.ts:311`）、`ON CONFLICT(memory_session_id, content_hash) DO NOTHING`（`SessionStore.ts:2663-2668`）、`files_modified` 用工具输入证据**覆盖**模型输出（`ResponseProcessor.ts:253-257`） | 模型定内容，代码定结构 | 机械结构 + 散文内容 | H8 Memo——由旁观模型自动产生，不是 Project 命令 |
| 3 | 每轮 Stop → 小结行 | `session_summaries` 行（每次 Stop 追加） | "This is a checkpoint to capture progress so far. The session is ongoing"（`code.json` `summary_instruction`） | 代码触发，模型写 | 机械触发 + 散文内容 | H8 里程碑投影——自动、无命令、无回执 |
| 4 | 任意阶段门 | — | 所有 handler 返回 `{ continue: true, suppressOutput: true }`（`hook-response.ts:1-4`）；src 内 grep `decision: 'block'` 无命中；worker 不可达 → `exit 0` 静默跳过（`hook-command.ts:139-150`） | — | — | **一条门都没有** |

判点：它刻意不画任何边——旁观模型被要求 "SILENT BY DESIGN… Never contact… any other agent or session"（`code.json` `observer_role`）。「会话结束」在它的模型里不存在，交接边其实是「每轮 Stop → 小结行」加「下次 SessionStart → 查询」。产物是「字段可信、内容不可信」：结构由代码保证，内容由第二个模型判断。MCP 工具 `observation_add`（`servers/mcp-server.ts:585`）允许主模型直接写记忆，绕过旁观者。

#### superpowers（obra/superpowers @ b36e0829，2026-08-12，MIT，v6.3.0）

14 个 SKILL.md + 3 个只生成文件的 bash 脚本；唯一 hook 是 SessionStart 注入 `using-superpowers` 全文（`hooks/hooks.json:3-14`），`exit 0` 无阻断路径。链：brainstorming → writing-plans → subagent-driven-development（SDD）或 executing-plans → TDD / verification-before-completion 横切 → requesting-code-review → finishing-a-development-branch。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 想法 → 设计已批准 | spike 口头 / bounded 聊天里的短设计 / architectural `docs/superpowers/specs/*.md` 并 commit | "Do NOT invoke any implementation skill… until you have told your human partner what you intend and they have approved it… the approval gate never does [scale down]"（`skills/brainstorming/SKILL.md:14-20`）；"The ratchet is one-way"（`:50-52`） | 人 | 散文 | H1：spec + commit ≈ Task Revision 来源；批准是聊天里的 "yes"，无 digest |
| 2 | 设计已批准 → 计划 | `docs/superpowers/plans/*.md`，checkbox 步骤，`**Spec:**` 回链 | 无占位符门 "never write them: 'TBD', 'TODO'…"（`skills/writing-plans/SKILL.md:131-139`）；交接 "Which approach?"（`:157-163`） | 模型写与自审；人选执行方式 | 散文 | H2 前半：批准退化为人选 SDD/inline；plan 可变 |
| 3 | 任务 → 派发实现子代理 | `BASE=$(git rev-parse HEAD)`（`skills/subagent-driven-development/SKILL.md:248-249`）；brief 文件 `task-<N>-brief.md`（`scripts/task-brief`）；ledger `.superpowers/sdd/<plan>/progress.md`（gitignored） | "Never make a subagent read the whole plan file"（`:262`）；"Never dispatch multiple implementation subagents in parallel"（`:282`）；"After compaction, trust the ledger and git log over your own recollection"（`:150-152`） | 模型（控制器） | 机械切分 + 散文决策 | H4：brief = Execution Spec（"it is your requirements, with the exact values to use verbatim"，`:256-257`）；BASE sha = 基线 |
| 4 | 实现子代理 → 控制器 | 报告文件（TDD Evidence: RED/GREEN 命令与输出）+ 四态 `DONE \| DONE_WITH_CONCERNS \| BLOCKED \| NEEDS_CONTEXT`（`implementer-prompt.md:130-146`）；`review-package` 从 git 生成 diff 文件 | "never spawn a reviewer to check your work… its approval counts for nothing"（`implementer-prompt.md:50-60`）；"BASE is the commit you recorded before dispatching the implementer — never HEAD~1"（`SKILL.md:290`） | 模型 | 散文；review-package 是机械证据载体 | H5 前半：报告 = 自述；控制器不当结果，交给评审 |
| 5 | 实现 → 评审通过（≤5 轮修复） | 双 verdict "Spec Compliance ✅/❌/⚠️ … Task quality: Approved \| Needs fixes"（`task-reviewer-prompt.md:163-187`）；ledger 行 `Task <N>: complete (commits <base7>..<head7>, review clean)`（`SKILL.md:437`） | "Do Not Trust the Report. Treat the implementer's report as unverified claims"（`task-reviewer-prompt.md:64-71`）；"Five rounds maximum per task"（`SKILL.md:373`）；上限后控制器自行裁定 `Task <N>: parked — <finding> — Ruling: <why the code stands>`（`:416`，"a silent discard is forbidden"，`:427-429`）；"Do not pause to check in with your human partner between tasks"（`:17`），只有四条停机条件（`:27-31`） | 模型（评审子代理 + 控制器裁定）；人不在环 | 散文 | H5 后半：评审 = 归约模块；但控制器（模型）5 轮后可停放 Critical 发现继续推进 |
| 6 | 评审通过 → 合入决定 | 全套测试 → 三选菜单 → merge / `git push -u origin` + PR URL / 保留 | "If tests fail, report the failures and stop — the menu comes after a green suite"（`skills/finishing-a-development-branch/SKILL.md:16-24`）；"Wait for their answer; the integration decision is theirs"（`:78-82`）；丢弃要 "Type 'discard' to confirm"（`:143-146`） | 人 | 机械（测试退出码）+ 散文 | H6：merge/push 事实 + 报 URL ≈ 回执，无结构化对象 |
| 7 | 任何「完成」声明 | 无产物 | "NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE… If you haven't run the verification command in this message, you cannot claim it passes"（`skills/verification-before-completion/SKILL.md:16-20`）；"Agent reports 'success'" 列为不算证据（`:47`） | 模型自律 | 散文 | H5 原则的 prompt 版，无机制承载 |

判点：机械强制为零，每条 MUST/Never 都取决于模型是否遵守。分层清晰：设计阶段每步问人，施工阶段一次都不问，集成阶段再问一次。「证据高于自述」出现在三个角色的措辞里（对自己、对实现者报告、对修复报告），但评审只看实现者报告里的命令输出加 diff，被禁止重跑全套测试（`task-reviewer-prompt.md:75-82`）。交接物是唯一命名的文件（brief、`review-<base7>..<head7>.diff`）而非粘贴文本，BASE 明确记录——这是技能包家族里最干净的派发形状；但 ledger 完成即 `rm -rf`（`SKILL.md:482-485`），verdict 不落盘。

#### claude-squad（smtg-ai/claude-squad @ ce1ffb43，2026-08-20，AGPL-3.0）

Go TUI。每个 instance = 一个 tmux 会话 + 一个 git worktree（`session/git/worktree.go:73-91`）。状态存 `~/.claude-squad/state.json`（含 `base_commit_sha` 与完整 diff 文本，`session/storage.go:10-42`）。人是调度器：`n` 新建、`c` checkout、`r` 恢复、`p` 推送、`D` 杀（`keys.go:38-58`）。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 无 → 新 instance | worktree + 新分支；`GitWorktreeData{BaseCommitSHA,…}`；tmux 会话 | 标题非空、≤32 宽、实例上限（`app.go:416-452,642-645`）；无初始提交则拒绝（`worktree_ops.go:97`）；`worktree add -b`（`:108`） | 人触发；代码检查 | 机械 | H4：worktree = baseline，`N` 的 prompt = Execution Spec；无 Task、无 Attempt 记录 |
| 2 | Running ↔ Ready | `Status` 字段 | `updated` = 截屏哈希变化（`tmux.go:262-266`）；`hasPrompt = strings.Contains(content, "No, and tell Claude what to do differently")`（`:255`） | 代码 | 机械（屏幕启发式） | 无对应——HCTL2 明言终端屏幕不算结果；这是它唯一的自动判定，且只判忙/闲 |
| 3 | 权限提示 → AutoYes | 无 | daemon `if hasPrompt { instance.TapEnter() }`（`daemon.go:51-52`）；开关 `-y/--autoyes`（`main.go:151`） | 代码替人 | 机械 | H9 被短路：授权请求由代码自动放行，不区分内容 |
| 4 | Running → Paused（`c`） | 脏则本地 commit（`--no-verify`，`worktree_git.go:144`）；worktree 删除、分支保留 | `git status --porcelain` 非空即 commit（`:153-160`） | 人触发 | 机械 | H6 前置：ChangeSet 固化为 commit，无 receipt |
| 5 | 推送（`p`） | 远端分支；不建 PR | 确认框 "[!] Push changes from session '%s'?"（`app.go:737`）；`gh repo sync` 失败则 `git push -u`（`worktree_git.go:97-107`） | 人确认 | 机械 | H6 部分：推送事实，无 Integration Receipt，合并留给 GitHub |
| 6 | 杀实例（`D`） | worktree 与分支删除 | 确认框；`checked out` 则拒绝（`app.go:696-714`） | 人 | 机械 | 无对应；未推送的工作随分支一起删 |

判点：`Status` 枚举只有 Running/Ready/Loading/Paused（`instance.go:18-29`），没有「完成」——工具从不宣称任务完成，也不让模型宣称。每条边都是人按键 + 确认框，只有 AutoYes 例外。Participant 生命周期做得机械且干净，但 Task/Run/Verdict 层完全缺席：「人当调度器」即「人当 reducer」，只是不留裁决记录。

#### ccpm（automazeio/ccpm @ 7d7e4623，2026-03-18，MIT）

此 HEAD 已改为 Agent Skill 布局（`README.md:22`）：`skill/ccpm/SKILL.md` 路由到 `references/*.md`（全是 prompt）+ 14 个只读查询脚本。PRD → epic → 任务文件（frontmatter `status/depends_on/parallel`）→ `gh issue create` 后文件改名为 issue 号 → worktree → 子代理按 stream 施工 → 人说「完成」时关 issue → epic-merge。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | PRD → epic → 任务文件 | `epic.md`（`prd:` 回链）；`001.md…`（Acceptance Criteria 复选框） | "Verify `.claude/prds/<name>.md` exists… confirm overwrite"（`references/plan.md:62-64`）；"Circular dependencies are an error"（`structure.md:106`，模型自查） | 模型 | 散文 | H1 近似（有回链，可覆写）；H2 无批准动作 |
| 2 | 任务文件 → GitHub issues | epic issue + task issues；文件改名为 `<issue>.md`；`github:` 回链；worktree | 模板仓库保护 `exit 1`（`sync.md:11-18`）；"No tasks to sync. Decompose the epic first"（`:29`） | 模型执行 gh | 机械（gh 退出）+ 散文步骤 | H1 最接近：issue 号成为身份；body 与文件仍可编辑 |
| 3 | issue → 开工 | `stream-<X>.md` `status: in_progress`；子代理 prompt "Work ONLY in your assigned files"（`execute.md:128`） | Preflight：`gh issue view --json state`、worktree 存在、`git status --porcelain` 干净（`execute.md:83-87,176-179`）——全由模型逐条执行 | 人触发；模型执行 | 形式机械、无脚本 | H2+H4：子代理 prompt = Execution Spec，无 Manifest |
| 4 | stream 施工完 → 标完成 | `stream-<X>.md` `status: completed` | "Complete your stream's work and mark status: completed when done"（`execute.md:134`） | 子代理自判 | 散文 | H5 反例：自述即结果 |
| 5 | 任务完成 → 关 issue | frontmatter `status: closed`；`gh issue comment` 固定文案 "✅ Task completed — all acceptance criteria met." + `gh issue close`（`sync.md:166-170`） | "Trigger: User marks a task complete"（`:161`）——无任何前置检查；Acceptance Criteria 复选框从不被解析 | 人宣告；模型执行 | 机械命令、散文条件 | H7：有「人类命令」一面，无独立校验；"all acceptance criteria met" 是硬编码模板 |
| 6 | 全部关闭 → 合入 main | `git merge --no-ff` + `push`；关 epic issue；目录归档 | "Check for uncommitted changes… block if dirty / Warn if any task issues are still open"（`sync.md:186-189`）；测试 "if detectable"（`:194-196`） | 人触发；模型执行 | 机械（dirty、退出码）+ 散文 | H6 + epic 级 H7；无 Integration Receipt |
| 7 | 查询：next / blocked | 无产物 | `next.sh` 只当 `depends_on` 为空才算 ready（`references/scripts/next.sh:36`），不检查依赖是否已关闭，与 `execute.md:186` 的散文定义不一致 | 代码 | 机械 | 无对应：只读投影 |

判点：只有「读」是脚本（"Script-First Rule"，`SKILL.md:47-66`），所有「写」都是让模型手工执行的 bash 片段。状态双写在 frontmatter 与 GitHub issue 两处，`validate.sh` 只校验本地文件不比对 GitHub。子代理未被要求关 issue，但也没有任何机制阻止它运行 `gh issue close`。

#### tdd-guard（nizos/tdd-guard @ ccd71b49，2026-08-16，MIT）

Claude Code `PreToolUse` hook，matcher `Write|Edit|MultiEdit|TodoWrite`（`plugin/hooks/hooks.json:5`）。三层判：代码快速路径（忽略文件、guard 关闭、恰好新增一个测试 → 放行）；代码 lint 门；模型 validator（把 modifications + `test.json` + todos + lint + instructions 拼成 prompt，交给无工具的另一个 Claude 回 `{"decision": "block" | null}`，代码解析）。测试结果由各框架 reporter 在测试进程内写 `.claude/tdd-guard/data/test.json`。

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 会话开始 → 干净状态 | 删 `test/todos/modifications/lint.json`（`Storage.ts:1`） | `SessionStart` → `processSessionStart` → allow（`processHookData.ts:103-106`） | 代码 | 机械 | 无对应；效果是**清空证据**，与 Context Manifest 相反 |
| 2 | Red：新增恰好一个测试 → 免审 | `modifications.json` | `isTestFile && countAddedTests(...) === 1`（`processHookData.ts:173-180`；ast-grep 计数 `testCounter.ts:151-170`） | 代码 | 机械（AST） | 无对应 |
| 3 | Red → Green：改实现须有失败测试证据 | `test.json`（reporter 写，整体覆盖，无签名） | "This test output is from PREVIOUS work… The modification has NOT been executed yet"（`prompts/tools/test-output.ts:3-5`）；`decision === 'block'` → block，`null` → allow，其他抛错（`validator.ts:150-158`）；模型无响应 → `block('No response from model, try again')`（`:21`） | 代码供证据、模型裁决、代码解析 | 机械载体 + 散文裁决 | 结构上是 H5 的微缩：test.json = Evidence，validator = reducer；只是 reducer 是 LLM |
| 4 | Green → Refactor | 同上 + `lint.json` | `isTestPassing`（全部 `state !== 'failed'`）且有 lint 问题未提醒 → block（`processHookData.ts:277-288`） | lint 门代码；重构许可模型 | 机械 + 散文 | 无对应 |
| 5 | 违规 → 阻断送达 | stdout `{decision:'block', reason}` + `exit 0`（`tdd-guard.ts:42-48`） | 用 hook JSON 阻断而非退出码；理由要 "helpful directions so that they do not get stuck"（`prompts/rules.ts:58`） | 代码 | 机械 | H9 反向：不向人发 Request，把 reason 喂回模型 |
| 6 | 关/开 guard | `config.json` `guardEnabled`，跨会话保留 | `UserPromptSubmit` 恰为 `tdd-guard off`（`userPromptHandler.ts:7-10,26-36`） | 人 | 机械 | 人的显式命令切换治理状态；无回执 |

判点：证据载体独立于施工模型的自述——`test.json` 由 reporter 在测试进程里写（`VitestReporter.ts:51`），不是让模型转述。这是四个小仓库里唯一一处「结果不由执行体自述」。但它是可覆盖的普通文件，模型可用 Bash 改写，文档因此建议 `deny Read(.claude/tdd-guard/**)` 并禁 `sed/awk/perl`（`docs/enforcement.md:9-35`）。fail-closed 与 fail-open 不对称：validator 异常 → block；hook JSON 不合 schema → allow。README 声明项目已被 Probity 取代（`README.md:9-13`）。

### 2.7 mattpocock/skills（@ 6654f6b6，2026-08-24，MIT）——引用既有审计

不重读，直接引用 [methodology-mattpocock-skills-20260902.md](./methodology-mattpocock-skills-20260902.md) §完成判定权专项的逐点核对表。按本文口径折叠：

| # | 阶段 | 阶段产物 | 跨越条件 | 谁判 | 机械/散文 | 对应 HCTL2 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | grilling → 讨论完成 | 对话共识；grill-with-docs 留 CONTEXT.md/ADR | 前沿为空 + 人确认 shared understanding（grilling/SKILL.md:28） | 人 | 散文 | H1 前半；docs 自认弱模型冲门 |
| 2 | to-spec / to-tickets → 就绪 | issue（spec）、子 issue（票，带 tracker 原生阻塞边） | `ready-for-agent` 标签**由模型自贴**，"no need for additional triage"（to-spec/SKILL.md:19；to-tickets:63） | 模型 | 散文 | H1 采纳：状态判定权在模型；AFK agent 把整份 spec 当票施工是 "the most-reported rough edge" |
| 3 | wayfinder 决策票 → 解决 | 评论落答案 + close + 地图追加一行（wayfinder/SKILL.md:125） | HITL 票 "the agent never stands in for the human's side"（:75）——但 close 动作是 agent 的 | 人在回路，agent 落锤 | 散文 | H9 / Scoped Room：三次失守（自答 e5932a7、Notes 自豁免、prototype 自关） |
| 4 | implement 施工票 → 完成 | commit；技能**没有完成步骤** | "It ends at the commit and never touches the work item… Close the ticket and reconcile the criteria yourself"（docs/engineering/implement.md:51-53） | 关单留给人 | 散文 | H7 同向但靠省略：无任何东西阻止 agent 去关 |
| 5 | TDD / code-review 证据 | 模型自跑自报；评审只看 `git diff <fixed-point>...HEAD`，commit 前调用常评空 diff（docs/engineering/implement.md:65） | "an agent reviewing the code it just wrote is biased toward its own solution"（:67） | 模型 | 散文 | H5 反例：自述证据可能是空集 |
| 6 | implement-spec → 整 spec 完成 | PR 标记 closing 全部票 | 人 merge 时机械关单（implement-spec/SKILL.md:23,31,33） | 人在 merge 门 | 机械（平台副作用） | 系统记录寄生标准形状（同 ccpm）：无逐票验收 |

判点（引用既有结论）：该仓库自己画了一条机制与方法论的分界线，线画在阶段切换上——人触发的 skill 永远不能调用另一个人触发的 skill（`.agents/invocation.md`）；「从一个阶段跨到下一个阶段只有人能按，阶段内部怎么干模型自己挑」正是本文要审的那条线。

## 三、HCTL2 交接对照

HCTL2 侧核对于 main @ `6850f18`（草案 v0.16.0，2026-09-02）。下表逐行列出 [spec/connections.md §连接约束总表](../design/spec/connections.md) 的八条连接，加上模块内部被别家当作阶段的几道门。「谁判 / 机械还是散文」两列按本文口径填——这是为了和第二节并排，不是对约束的重述。

### 3.1 跨模块连接（八条）

| 编号 | 方向 | 交付物 | 准入（目标怎么判） | 恢复依据 | 谁判 | 机械/散文 |
| --- | --- | --- | --- | --- | --- | --- |
| H1 | Project → Task | Project/version、来源引用（Message / Artifact / Memo / Request 的精确引用）、契约（标题、预期结果、验收约束、角色/能力）、proposal digest | 「创建 Task」/「采纳契约」命令；CAS 校验 Project version 与当前 Task Revision；工具箱回读 Git 正文后才准入不可变 Task Revision；普通消息、总结、拖放都不能创建 Task（connections.md §Project → Task；spec/task.md §契约与来源） | 命令幂等键 + 关联键 → 同一 Task / 卡 / Revision | 人（采纳）+ 代码（CAS、digest） | 机械准入；契约内容由人写 |
| H2 | Project / Task → Run | Project/version、0..1 Task Revision、Workflow Revision + Engine Deployment、repo baseline、根 Context Manifest、Participant / Role / Skill、候选、权限、预算、Gate → 冻结为 Run Manifest | 「启动 Run」命令；批准 Workflow 是另一个动作；CAS 校验 Project / Task / Workflow 版本、Task 有契约且无占用标记、无未处理待采纳（spec/run.md §启动与 Manifest；spec/task.md §启动 Run 的前置与排序令牌） | run_id + manifest_digest → Engine Execution Binding 回读 | 人（启动）+ 代码（CAS、lint、编译） | 机械 |
| H3 | Project → Participant | Room Invocation + Execution Spec（`repo_scope` 只读 / `project_scope` 可写） | Trigger Preview 后由可归属 human 动作提交；模型 Participant 的 `@` 只形成建议（spec/project.md §Room Invocation、§场景约束） | invocation id + invocation_version + spec digest | 人 + 代码 | 机械 |
| H4 | Run → Participant | Attempt + Execution Spec（冻结 Context Manifest / Bundle digest、Participant revision、Role Binding、Skill digest、Worker Profile、端口绑定、权限、预算、租约规则、三层代次） | Run 先持久化派发授权；Agency 交付执行体端点；实际能力缺任一已声明加固项则拒绝激活（connections.md §Project / Run → Participant） | attempt id + attempt_generation + spec digest | 代码 | 机械 |
| H5 | Participant → Project / Run | Result Proposal（proposal id、归属者、运行时、代次、spec / bundle digest、producer sequence；逐输出 schema key + content digest） | inbox 去重；逐项校验归属者状态、代次栅栏、digest、租约、ChangeSet、输出范围、证据、权限；Attempt 结果由 Run 归约为 Seat 结果 / Verdict / Receipt；Task 不消费进程状态、自述、终端屏幕（connections.md §Participant → Project / Run） | 提案标识符 + producer sequence；迟到结果只留历史 | 代码（归约器）；Gate 席位由 Participant 投票、代码计票 | 机械 |
| H6 | human scene / Run reducer → Participant「合入 ChangeSet」 | 精确 ChangeSet Revision、来源与基线、目标引用、预期目标头、策略、适用 Verdict 与证据 | 先持久化意图与 outbox，工具箱执行 Git 集成并回读基线、HEAD、tree、祖先、PR、检查、评审、目标分支头；成功回读才写唯一 Integration Receipt；"模型自述不能证明集成成功"（spec/participant.md §ChangeSet 与 Git 事实） | intent id + expected target head → 唯一 Receipt；结果未知不重投 | 人（下令）+ 代码（回读） | 机械 |
| H7 | human Kanban / Run reducer → Task「完成 Task」 | human provenance 或正常完成 Run ref；冻结的 Task Revision ref；Revision / Evidence / Verdict / Receipt refs | 只有两个获准来源；Task 按当前 Revision、验收规则、候选、全部必需证据独立校验；Receipt 逐条验收项分别固定通过 / 失败与 Evidence 引用，"不能用一个总括的'测试通过'替代逐项绑定"；Run 完成而 Task 校验失败时 Run 保持完成、Task 保持开放并需要关注（spec/task.md §写入约束） | 命令 id → Task Completion Receipt | 人，或代码（归约器）；Task 侧代码校验 | 机械 |
| H8 | Task / Run / Participant → Project | source ref、event id / sequence、版本、敏感级别 | Project 只建低噪声投影；"Memo 只由用户明确发布"；"普通 Git 文件在登记前不是 Artifact"（spec/project.md §Context、Memo 与 Artifact） | source event cursor，可从源账本重建 | 代码（投影）；人（Memo / Artifact 发布） | 机械投影；沉淀由人 |

### 3.2 模块内部的门

| 编号 | 门 | 交付物 | 跨越条件 | 谁判 |
| --- | --- | --- | --- | --- |
| G1 | Repo Room → Project 提升 | 显式选中的来源 | "从 Repo Room 提升 Project 时只带显式选中的来源……之后的聊天不会偷偷改变既有 Project"（design/project.md §关键规则） | 人 |
| G2 | Workflow 登记 / 编译 / 批准 | Workflow Revision（规范化 JSON，Git 正文）+ Engine Deployment | schema / 引用 / Profile / 图结构 lint → 固定编译器生成 Dagu YAML → 人批准；"Approve Workflow 只确认施工图"（spec/run.md §Workflow 与 Run 授权） | 代码 lint + 人批准 |
| G3 | Run 内 Gate | ReviewSubjectRef → 各席位 Verdict → 汇总 Verdict / Receipt | 法定票数；作者不占必需评审席位；备用 Attempt 不增票；返工产生新 Revision 后旧票作废、完整重评；quorum-unreachable 沿失败边推进（spec/run.md §Request、重试与 Gate） | Participant 投票 + 代码计票 |
| G4 | Request 回路 | Request（归属者、受影响 revision、阻塞范围、所需 actor / role、截止策略） | 「解决 Request」由所需 actor 提交；"普通 Room 回复不能解决 Request"；过期按 fail / cancel 冻结策略结束，"不伪造答案"（connections.md §跨模块 Request 回路） | 人（或所需 role）+ 代码 CAS |
| G5 | Scoped Room 结案 | 讨论目标、完成条件、回填动作 | "归档只允许两条路径：回填动作成功，或有权 human actor 显式以 abandoned、no-decision 或 superseded 结案"；"达到完成条件不会自动修改目标"（spec/project.md §Room 与消息） | 人 |
| G6 | Context 接力（Room → Run → Attempt） | 根 Context Manifest（来源 ref + digest、freshness、gaps）→ 每消费者 Context Bundle（digest、压缩记录） | Execution Spec 冻结 Manifest 与 Bundle digest；"Context Bundle 是调用开工时交付给执行体的输入包，不代管执行体在会话中自行组织的工作上下文"（spec/project.md §Context、Memo 与 Artifact）；"没提交的对继任者不存在"（design/run.md §关键规则） | 代码 |
| G7 | 重开 Task | 预期 task_lifecycle_version | 只接受有权 human；不复活旧 Receipt；drift 须先采纳或显式冻结（spec/task.md §写入约束） | 人 + 代码 |

### 3.3 一眼看出的三个形状差异

把第二节 19 张表和上面两张并排，不需要逐条归纳就能看到三件事：

1. **别家的边后面没有冻结。** 19 家的阶段产物里，不可变的只有 git commit 和 GitHub issue 号；spec / plan / tasks / PRD.json / PLAN.md / STATE.md 全是可就地改写的文件。HCTL2 八条连接的交付物全部带 digest 或 generation。这不是"我们更严"，而是两种不同的东西：别家递的是「当前状态的一份副本」，HCTL2 递的是「对精确版本的引用」。
2. **别家的「谁判」列里，模型出现在完成边上；HCTL2 的模型只出现在投票边上。** 19 家里 15 家有任务完成这条边，其中 14 家让施工模型在主路径或侧门上自己把任务标完成（checkbox、`passes:true`、`set_task_status`、`status: completed`、`task_complete`、`finish_current_task`、`bd close`）；HCTL2 的模型 Participant 只能在 Gate 席位投票（G3）或交提案（H5），完成命令（H7）的两个来源里没有它。
3. **别家的散文门集中在计划段，机械门集中在集成段。** 需求 → 设计 → 任务的门几乎全是 prompt 或人点按钮；PR merged、tests pass、exit code 才是代码判。HCTL2 反过来：计划段只有一道门（H1 采纳），但它是机械准入；集成段（H6 / H7）也是机械，且比别家多了逐项绑定。计划段少门、集成段重门，是 HCTL2 这一票的形状。

## 四、三类归纳

每条归纳给「遗漏 / 过早 / 改写 / 维持」四选一。裁决用一把尺子：**换一种方法论，这条边还需不需要**（[mattpocock 审计 §分界原则](./methodology-mattpocock-skills-20260902.md)）。换了就不需要的归 Skill，HCTL2 托管不拥有；换了仍需要的归对象或机制。「过早」只给一种情形：别家没有提出这条边的需求、也没有用户替我们要它、而我们已经钉死。

### 4.1 别家有、HCTL2 没有的边

| # | 边 | 哪几家 | 递什么 | 我们为什么没有它的位置 | 裁决 |
| --- | --- | --- | --- | --- | --- |
| 1 | 规格链内部的门：需求 → 设计 → 任务 | Kiro（三道 "Happy?" 门）、OpenSpec（proposal → specs/design → tasks 依赖图）、spec-kit（specify → clarify → plan → tasks）、BMAD（四阶段文档链）、GSD（discuss → plan → plan-checker）、ccpm（prd → epic → tasks）、superpowers（brainstorm → plan）、ruflo SPARC（五阶段） | 可变的 Markdown / JSON / KV：`requirements.md`、`plan.md`、`PLAN.md`、`epics.md`、`sparc-phases/*` | HCTL2 在「意图」段内部不设系统门，只在意图 → 承诺处画一条边（H1）。中间文档是 Artifact Revision，可登记、可被 Gate 评审，但过不过由塑形的人决定 | **维持**。判据：19 家里这些门后面全没有冻结——Kiro 三文件随时可改、spec-kit 无 digest、ruflo 的 KV 默认 upsert。门后无冻结，说明它们是「人看一眼」而不是交接；换一种方法论，门的数目和位置都变（Kiro 3 道、spec-kit 2 道 gate、OpenSpec 0 道），正是「方法归 Skill」的东西。HCTL2 让塑形 Participant 装载的 Skill 自带这些门，Artifact Revision 给它们落点 |
| 2 | 任务 → 任务的 readiness 归约（下一个能做什么） | beads `bd ready`（`sqlbuild/ready.go:105-110`，19 种边只有 4 种参与）、Taskmaster `next`（`find-next-task.js:55-128`）、GSD wave / `blocked_by`、Kiro waves、ccpm `next.sh`、mattpocock frontier、OpenSpec artifact 依赖图 | 「下一个可做的任务」这个选择结果 | Task 是承诺尺度不是工作项尺度（design/task.md §模块拥有什么）；依赖归后端原生字段（操作投影），Blocked 是 health；HCTL2 不排程 | **维持**。判据：readiness 归约的消费者是 AFK 自领循环——ralph、Taskmaster loop、Gas Town daemon `feedNextReadyIssue`（`internal/convoy/operations.go:307-346`）、beads——而 HCTL2 明文拒绝模型自领（"普通 Room 的临场执行边由 human 提交"，design/README.md §共同规则）；人在看板上看「哪个能做」，后端的阻塞字段投影就够。ccpm `next.sh` 的缺陷（`depends_on` 非空即不 ready，与自家散文定义不一致）顺带说明自建 readiness 容易写错。第一阶段一个 Run 至多绑一个 Task；放开时这条要重估 |
| 3 | 会话 → 会话的记忆交接 | ralph `progress.txt`、claude-mem `observations`、GSD `HANDOFF.json` / `STATE.md`、Kiro 三文件、Taskmaster `update-subtask`、superpowers ledger、ruflo `session-end`、BMAD 上一 story 的 Code Map | 全部由模型写、模型读；claude-mem 可删无版本，GSD HANDOFF "Consumed exactly once"，superpowers ledger 完成即 `rm -rf` | HCTL2 明文不代管执行体会话内的工作上下文（spec/project.md §Context、Memo 与 Artifact），接力只经结晶：Memo 人发布、Context Manifest 冻结、ChangeSet pointer、Verdict 正文 | **维持**。判据：19 家在这条边上无一有门；claude-mem 的产物是「字段可信、内容不可信」（结构由代码保证、内容由第二个模型判），正好说明记忆载体天然给不出证据。代价：Run 内学到的东西要靠人发布 Memo——这是"原始消息、执行日志和自动总结不会自动进入长期知识"的既定立场，滚动纪要作为派生缓存已有位置 |
| 4 | 完成后的归并 / 归档 / 回顾边 | OpenSpec `archive`（delta → 主 spec 确定性合并）、GSD `complete-milestone`、BMAD retro（`verdict` 三硬规则）、ccpm epic 归档、spec-kit `converge` | 「现状规格的新版本」或「里程碑归档 + 统计」 | Artifact 发布新版本只移动 current pointer（人）；Project 归档；没有自动归并 | **维持**。判据：知识回流需人显式发布是有意的；里程碑 / epic 层在 HCTL2 由后端原生父子任务承载，而各家 milestone close 都是人手动加统计，不是治理事实。OpenSpec 归并的形状（预演 → 确认 → 指纹比对 → 校验 → 写）已在 landscape 列为适配协议，是 Artifact 发布命令的行为参考 |
| 5 | 编辑级 step gate（红 → 绿 → 重构） | tdd-guard（PreToolUse hook）、GSD TDD RED-commit 门、Taskmaster autopilot RED/GREEN、superpowers TDD | 测试结果 JSON、RED/GREEN commit | Attempt 内部纪律归 Skill 与 Worker Profile；证据进 Result Proposal | **维持**。判据：方法归 Skill。但 tdd-guard 有一处值得单独记：`test.json` 由 reporter 在测试进程内写（`VitestReporter.ts:51`），不由模型转述——见 4.3 第 3 条 |
| 6 | 等待外部机械事实的节点（CI 跑绿、PR 合并、定时器） | beads gate 五型 human / timer / gh:run / gh:pr / bead（`cmd/bd/gate.go:30-35,608-616`）、vibe-kanban PR monitor 60 秒轮询 → Done（`pr_monitor.rs:68`）、Gas Town `gt mq post-merge` 的 merge proof（`internal/cmd/mq.go:601-620`） | 「外部系统状态变化」这一事实 | Workflow Profile 允许 timer wait 与 Dagu `human.task` 被动检查点；「等 CI 绿 / 等 PR 合并」只在 H6 合入时由工具箱校验，没有作为 Workflow 节点的名字；`executor = tool` 只出现在合入命令（spec/participant.md §对象） | **改写**。判据：HCTL2 现在要在 Run 里「等 CI」，只能派一个 Participant 去查，把机械事实伪装成执行体提案——这违反自己的"能承载不等于能裁决"。vibe-kanban 用 PR 事实驱动看板是生产实证，beads 把它做成一等 issue 的五型 `AwaitType` 是最干净的词表，且解决条件逐型写成谓词（"gh:run: status=completed AND conclusion=success / gh:pr: state=MERGED / timer: current time > created_at + timeout"）。顺带一条对照：beads `bd gate check` 在 `gh` 不可用时 "allow close with a warning"（`close.go:579-581`），HCTL2 对同一情形已写死为类型化拒绝加需要关注——新节点类型要沿用这条。改法：Workflow Profile 的外部执行节点补一种 `executor = tool` 的 Obligation，产出外部事实的 Evidence（PR 状态、check run 结论），不占 Participant 席位、不投票。不新增模块 |

### 4.2 只有 HCTL2 有的边

| # | 边 | 别家怎么不用它也把事办了 | 市场未收敛、我们先钉死？ | 裁决 |
| --- | --- | --- | --- | --- |
| 1 | H1 采纳契约：不可变 Task Revision + digest | 把承诺当活文件，漂移靠事后补：Kiro Sync Files（模型扫代码库自查）、spec-kit converge、GSD `--gaps`、OpenSpec verify。BMAD `<frozen-after-approval>` 是唯一在文件里标出冻结段的，但靠 prompt "read-only. Do not modify" | 不是。用户在替我们要这条边：Kiro #5019 "Treat Completed Task as Immutable"（被以无跟进关闭）、#6826 "tasks.md drifting… unreliable as a progress tracker and handoff document"、mattpocock `ready-for-agent` 标签事故（"the most-reported rough edge"） | **维持**。这是完成判定权的前提：没有冻结的验收约束，就没有东西可以独立校验 |
| 2 | H2 Run Manifest：批准 Workflow ≠ 启动 Run，且冻结 baseline / 参与者 / 权限 / 预算 / Gate | 两步本身不独有——GSD plan → execute、Kiro design "Happy?" → Start task、spec-kit review-plan gate → implement 都是两步。独有的是 Manifest 冻结。别家最接近：vibe-kanban `before_head_commit / after_head_commit`、superpowers `BASE=$(git rev-parse HEAD)`、BMAD `baseline_commit`（"never overwrite it"）、Taskmaster `workflow-state.json`（finalize 即删） | 不是。四家都自己长出了 baseline 记录，只是没长出预算 / 权限 / Gate | **维持**。实现层脚注（不构成对边的裁决）：19 家无一用外部 DAG 引擎，但 spec-kit 与 Taskmaster 都自己长出了持久状态机（PAUSED 落盘 + resume；`workflow-state.json` 原子写）——「需要持久状态机」被验证，「需要外部引擎」没有。这是交付选型问题，记下不裁 |
| 3 | H5 Result Proposal 准入：执行体只能提议 | 15 家有任务完成这条边，其中 14 家让模型在主路径或侧门上自己标完成（checkbox、`passes:true`、`set_task_status`、`status: completed`、`task_complete`、`finish_current_task`、`bd close`——未分配的 issue 任何 actor 可关，已分配的靠字符串比较，`--force` 全跳，`internal/validation/issue.go:154-173`）；唯一例外 ccpm 靠人宣告，但也没有任何机制阻止子代理运行 `gh issue close`。同向碎片全在 prompt 里：GSD "SUMMARY.md claims are not evidence"、superpowers "Do Not Trust the Report"、BMAD "Judge against the diff, not against the implementation subagent's report"、Kiro Crew "not the LLM's self-report"、Gas Town Witness 的 `verifyCommitOnMain`（`internal/witness/handlers.go:1402-1462`） | 不是。五家把同一句话写进了 prompt，说明需求已提出；没有一家做成系统边界，说明工具层做不到 | **维持**。landscape 已判的差异化环节，本轮 19 家逐边核对后不变 |
| 4 | H7 Task Completion Receipt：两个获准来源 + 逐项绑定证据 | 最接近的四个：GSD `phase.complete`（代码门，但输入可被工作流改写 `frontmatter.set … passed`，且 `status: superseded` 是 "committable, review-time-trusted bypass"）、BMAD retro `verdict`（epic 粒度，`pending_stories` 代码算）、vibe-kanban `merged_at + merge_commit_sha`（Integration 级不是 Completion 级）、Gas Town `gt mq post-merge`（merge proof 后关单，reason 含 `target_branch / commit_sha`，`work_bead_close.go:77-80`）——但合并即完成，无独立验收，且 `ForceCloseWithReason` 绕过 bd 自身策略 | 不是。GSD 与 BMAD 都走到了「代码判完成」门口，卡在输入由模型写 | **维持** |
| 5 | H3 Room Invocation：无 Run 的短路也有身份票据 | 短路 = 直接开会话：claude-squad instance、Kiro plan mode（"Plan lives in conversation context"）、superpowers spike、BMAD in-session 路由 | 不是。claude-squad 证明「人当调度器」时也要 instance 记录（`state.json` 带 `base_commit_sha` 与完整 diff），只是不留裁决 | **维持**。「必须比开终端更轻」的标尺由 Trigger Preview 承担，Execution Spec 是 agent 侧的 |
| 6 | H9 / G4 Request 对象 + "普通 Room 回复不能解决 Request" | GSD checkpoint（聊天 "approved" 即算；auto 模式 human-verify 自批、decision 自选第一项）、claude-squad AutoYes（字符串匹配替人回车）、ruflo `PermissionRequest` 自动 allow、mattpocock HITL 自答事故、BMAD intent_gap HALT（聊天） | 不是。vibe-kanban 的 `PendingApproval` 专用端点 + 超时默认 `TimedOut` 是同向实证 | **维持**。反例清单本身就是理由：凡把授权请求放在聊天里的，都出现了代码或模型替人回答 |
| 7 | H4 Execution Spec + 三层代次 + 丢失处理 | 派发 = spawn 进程 + prompt 字符串：vibe-kanban `executor_action` JSON（唯一持久化的派发记录）、superpowers brief 文件、GSD executor prompt + `safe_resume_gate` 按 git log 对账 | 不属于方法论投票——这是执行身份正确性，不是流程形状 | **维持**，并标注不在本轮审的投票范畴内 |
| 8 | G3 Gate 法定票数、作者回避、备用 Attempt 不增票 | 评审全是单评审者：superpowers task-reviewer、GSD verifier、BMAD 三层 subagent 一个会话汇总、Kiro Crew reviewer。唯一多票是 ruflo consensus——不在任务路径上，且 `voterId` 是自由字串 | 多票评审：市场确实未收敛。作者回避：不是——superpowers "never spawn a reviewer to check your work… its approval counts for nothing"、BMAD code-review "fresh context, different LLM recommended"、Kiro Crew "Independent reviewer session" 三家都在要 | **维持**。法定票数是策略可声明的，单席位 = 法定票数 1，边不强迫任何人多投票；作者回避是三家在 prompt 里要、HCTL2 放进账本的东西 |
| 9 | Task 占用标记：同一 Task 至多一个活动 Run | 一个任务多个并行 attempt：vibe-kanban 允许一个 issue 开多个 workspace（`crates/remote/src/db/issues.rs:638-648` 只在首个 workspace 时推状态）；ccpm 把一个 issue 拆成并行 stream；Gas Town convoy 与 Kiro waves 是多任务并行不是多 attempt，不算 | 不是未收敛，是方向相反：别家并行在 Task 之下、Run 之外；HCTL2 并行在 Run 之内（Seat / 候选 / dynamic fork） | **维持**。占用标记管的是「只有一个 Run 能提交完成」；「让三个 Harness 各试同一任务、人挑一个」在 HCTL2 是一个带 dynamic fork 的 Run，或三个 Room Invocation。交付侧要把前者做成现成 Workflow 模板，否则用户会绕 |

读法：只有 HCTL2 有的九条边，每一条都能在别家找到用 prompt 写出来的同向碎片、自己长出来的半成品（baseline 记录、持久状态机、专用 approve 端点）、或用户替我们要它的事故记录。**找不到一条是市场没提出需求、我们先钉死的**——所以这一节没有「过早」。

### 4.3 别家交付物或跨越条件定义得比我们干净的

| # | 边 | 别家原文 | HCTL2 原文 | 差在哪 | 裁决 |
| --- | --- | --- | --- | --- | --- |
| 1 | 评审发现 → 回环去向 | BMAD：`intent_gap` "Root cause is inside `<frozen-after-approval>`. Revert code changes. Loop back to the human to resolve." / `bad_spec` "Revert code changes… Append a new change-log entry… re-derive the code"（`ship/bmad-build/step-04-review.md:63-64`） | "范围、权限、候选或验收含义变化时必须显式替代，而不是原地修补"（spec/connections.md §版本、权限与替代）；语义返工 = "changes_requested 汇总 → 新 ChangeSet Revision… 旧票失效并完整 regate"（spec/run.md §Request、重试与 Gate） | HCTL2 有两条路（语义返工 / 替代执行），但**谁判走哪条**没写——BMAD 让评审者按「根因落在冻结块内还是块外」分类，机械决定回人还是回模型 | **改写**。Verdict 的 `changes_requested` 加一个分歧落点字段：落在 Task Revision 验收约束内 → Task 显示需要关注并建议采纳新 Revision；落在约束外 → 走语义返工。不新增对象，只给已有的两条路一个机械分流 |
| 2 | 验收约束的形状 | GSD `must_haves`："truths / artifacts (path, min_lines, exports, contains) / key_links (from, to, via, pattern)"（`templates/phase-prompt.md:556-602`）；核验三级 "Exists / Substantive / Wired"（`agents/gsd-verifier.md:243-288`）；"Presence is not behavior… Never let symbol presence alone produce a VERIFIED on a behavior-dependent truth"（`:863`） | "Task Revision 冻结验收契约，不冻结施工步骤"（spec/task.md §契约与来源）；Receipt "每一条验收项还要分别固定通过或失败、Evidence/Verdict/Receipt 引用与摘要"（§写入约束） | HCTL2 说了 Receipt 绑什么，没说一条验收项长什么样、哪些等级代码能判、哪些要评审。GSD 把「文件存在 / 内容非空 / 被接线」分开，前两级代码判、后两级模型判，且禁止用存在冒充行为 | **改写**。Task Revision 验收约束的写作形状采用三类 + 每项声明校验等级（机械可判 / 需评审），Receipt 逐项绑定才有明确的 schema。landscape 已把 must_haves 列为适配协议，本轮把它落到验收约束的写作指引 |
| 3 | 测试证据由谁产生 | tdd-guard：`test.json` 由 reporter 在测试进程内写（`VitestReporter.ts:39-52`；`pytest_reporter.py:106-108`）；validator 看到的永远是 "the output from the most recent test run BEFORE this modification"（`prompts/tools/test-output.ts:3-5`） | "结构化事件统一归一为生命周期提示、工具调用、权限请求、文件变化、测试、用量和原始输出"（spec/participant.md §运行时与观测）；工具箱校验 "Git 基线、HEAD、tree、祖先关系、PR、检查、评审和目标分支头"（§ChangeSet 与 Git 事实） | HCTL2 的本地测试证据走 harness 适配器归一的「测试」事件——是执行体上报；PR 上的检查由工具箱回读——是外部事实。两者在 Evidence 里没有生产者区分 | **改写**。Evidence 加生产者字段：reporter / 工具箱直接产生 vs harness 事件转述；Task 验收策略可以要求前者。不新增对象 |
| 4 | 归档 / 完成的阻塞态命名 | OpenSpec JSON 模式：`archive_validation_failed` / `archive_tasks_incomplete` / `archive_confirmation_required`，每个阻塞点抛具名错误（`archive.ts:1286-1290,1341-1350,1488-1494`） | "命令必须拒绝"、"类型化拒绝"（spec/task.md、spec/run.md 多处） | 同一件事，HCTL2 叫「类型化拒绝」但约束层没列举类型词表；OpenSpec 列了 | **维持**。类型词表是接口细节，for agent，自己设计自己用，不需要进约束 |
| 5 | 无人时门的行为 | spec-kit gate：非 TTY → `PAUSED` 落盘，`specify workflow resume` 恢复（`gate/__init__.py:174-175`）；Kiro Crew `force_approval` "block execution even in YOLO mode" | Request "冻结截止时间与 `fail \| cancel` 默认策略… 过期不能猜测答案"（spec/run.md §Request、重试与 Gate） | HCTL2 比两家多了截止语义；spec-kit 永远等 | **维持**。HCTL2 的定义更完整 |
| 6 | 合入 → 关单的回执 | Gas Town："1. Verify the target branch contains the submitted source head 2. Close the MR bead (status: merged) 3. Close the source issue 4. Delete the remote polecat branch at the submitted head"（`internal/cmd/mq.go:177-180`）；关单 reason `Merged in <MR>` + `target_branch:` + `commit_sha:`（`internal/refinery/work_bead_close.go:77-80`） | "命令必须固定 ChangeSet Revision、来源与基线、目标引用、预期目标头、策略、适用 Verdict 和证据、actor 与权限、绑定和幂等键。成功回读后，control 才写唯一 Integration Receipt"；"工具箱校验 Git 基线、HEAD、tree、祖先关系、PR、检查、评审和目标分支头"（spec/participant.md §ChangeSet 与 Git 事实） | 同一件事：都用「目标包含提交头」做证明。Gas Town 的载体是描述里的 `key: value` 文本，HCTL2 是结构化 Receipt；Gas Town 把合入与关 Task 合为一步，HCTL2 分开（H6 / H7） | **维持**。HCTL2 的定义更完整；Gas Town 值得记的一点是 merge proof 必须用提交头而不是「分支存在」（"Branch-exists checks are insufficient"，`done.go:1438-1453`），HCTL2 已有（预期目标头 + 祖先关系） |

## 五、审计基线一览

本轮全部钉 HEAD（2026-09-02 浅克隆）。与 landscape 前次钉定不同的，两列并列；行号一律指本次钉定。

| 工具 | 仓库 | 本次钉定 | landscape 前次钉定 | 许可 | 本轮读了什么 |
| --- | --- | --- | --- | --- | --- |
| OpenSpec | Fission-AI/OpenSpec | d0071d73（2026-09-01） | f1b521df | MIT | `schemas/spec-driven/schema.yaml`、`src/core/archive.ts`、`src/core/artifact-graph/*`、`src/core/templates/workflows/*.ts`、`src/commands/*` |
| spec-kit | github/spec-kit | 0053c3a3（2026-09-01） | 27f50f7e | MIT | `templates/commands/*.md`、`scripts/bash/*.sh`、`workflows/speckit/workflow.yml`、`src/specify_cli/workflows/steps/gate/`、`engine.py` |
| Kiro | 闭源（kiro.dev） | 文档快照 2026-09-02，42 页 + issues #5011 / #5019 / #6826 | 快照 2026-08-24 | 专有 | specs / hooks / steering / permissions / autopilot / plan / quick-spec / correctness / crew task-runner |
| beads | gastownhall/beads | 40b32324（2026-09-01） | 8d86c06b | MIT | `internal/types/types.go`、`internal/storage/sqlbuild/ready.go`、`issueops/{claim,close,blocked_state,lease}.go`、`cmd/bd/{close,gate,reclaim,heartbeat,cook,mol_current,prime}.go`、`internal/validation/issue.go` |
| Taskmaster | eyaltoledano/claude-task-master | c0c98d36（2026-04-23） | 同 | MIT + Commons Clause | `scripts/modules/task-manager/*`、`mcp-server/src/tools/set-task-status.js`、`packages/tm-core/src/modules/{workflow,loop,tasks}/*`、`apps/cli/src/commands/autopilot/*` |
| vibe-kanban | BloopAI/vibe-kanban | 4deb7eca（2026-04-24 终版） | 同 | Apache-2.0 | `crates/{remote/src/db/issues.rs, services/src/services/{container,pr_monitor,approvals}.rs, local-deployment/src/container.rs, mcp/src/task_server/tools/*, executors/src/actions/*, server/src/routes/*}` |
| BMAD | bmad-code-org/BMAD-METHOD | 891c0abb（2026-09-02） | 1479a58b | MIT | `src/bmm-skills/{plan,ship}/**`（bmad-spec、bmad-build、bmad-build-auto、bmad-code-review、bmad-retrospective 全部 steps / references / templates）、`scripts/{sprint_status,sprint_plan,git_evidence}.py`、`docs/build/autonomous-development-loops.md` |
| MetaGPT | FoundationAgents/MetaGPT | 11cdf466（2026-01-21） | 同 | MIT | `metagpt/software_company.py`、`team.py`、`roles/di/{role_zero,team_leader,engineer2}.py`、`strategy/planner.py`、`schema.py`、`prompts/di/*`、`environment/mgx/mgx_env.py` |
| agent-os | buildermethods/agent-os | 475b0cac（2026-08-29） | cae8e664 | MIT | `commands/agent-os/*.md`、`CHANGELOG.md`、`scripts/{project-install,sync-to-profile}.sh` |
| GSD | open-gsd/gsd-core；gsd-build/get-shit-done（已归档） | fa107c04（2026-09-02）；bdcaab2c | 314ea20f；bdcaab2c | MIT | `workflows/*.md`、`agents/{gsd-executor,gsd-verifier,gsd-planner}.md`、`references/{checkpoints,gates,verifier-evidence-gate}.md`、`templates/*`、`src/{phase,verification,verify}.cts`、`hooks/*` |
| Gas Town | steveyegge/gastown | 649b832b（2026-07-23） | 同 | MIT | `internal/cmd/{done,mq,sling_*,convoy_*,hook,handoff,prime_session,seance,up}.go`、`internal/refinery/{engineer,batch,work_bead_close}.go`、`internal/witness/handlers.go`、`internal/convoy/operations.go`、`internal/daemon/convoy_manager.go`、`internal/templates/roles/*.tmpl`、`internal/formula/formulas/*.toml` |
| ralph | snarktank/ralph | 6c53cb0b（2026-02-01） | —（新增） | MIT | `ralph.sh`、`prompt.md`、`CLAUDE.md`、`AGENTS.md`、`skills/{prd,ralph}/SKILL.md` |
| claude-mem | thedotmack/claude-mem | e5b6719f（2026-09-02） | —（新增；本族另一代表 agents.md 未取） | Apache-2.0 | `plugin/hooks/hooks.json`、`plugin/modes/code.json`、`src/cli/handlers/*`、`src/services/sqlite/SessionStore.ts`、`src/sdk/parser.ts`、`ResponseProcessor.ts`、`servers/mcp-server.ts` |
| superpowers | obra/superpowers | b36e0829（2026-08-12，v6.3.0） | —（新增） | MIT | 全部 14 个 `skills/*/SKILL.md` 与附带 prompt 模板、`skills/subagent-driven-development/scripts/*`、`hooks/*` |
| claude-squad | smtg-ai/claude-squad | ce1ffb43（2026-08-20） | —（新增） | AGPL-3.0 | `app/app.go`、`session/{instance,storage}.go`、`session/git/*`、`session/tmux/tmux.go`、`daemon/daemon.go`、`keys/keys.go` |
| ccpm | automazeio/ccpm | 7d7e4623（2026-03-18） | —（新增） | MIT | `skill/ccpm/SKILL.md`、`references/{plan,structure,sync,execute,track,conventions}.md`、`references/scripts/*.sh`（此 HEAD 已是 Agent Skill 布局，`.claude/commands/pm/*` 不存在） |
| ruflo | ruvnet/ruflo（原 claude-flow） | 4d0134e5（2026-09-01，v3.38.20） | —（新增） | MIT | `v3/@claude-flow/swarm/src/{coordination/task-orchestrator.ts, consensus/*, queen-coordinator.ts, types.ts}`、`v3/@claude-flow/cli/src/mcp-tools/{task,hive-mind,memory,hooks,claims}-tools.ts`、`cli/src/commands/{task,hive-mind}.ts`、`plugins/ruflo-sparc/**`、`plugin/hooks/hooks.json`、`.claude-plugin/hooks/hooks.json` |
| tdd-guard | nizos/tdd-guard | ccd71b49（2026-08-16） | —（新增） | MIT | `src/cli/tdd-guard.ts`、`src/hooks/*`、`src/validation/*`、`src/guard/GuardManager.ts`、`reporters/{vitest,pytest}/*`、`plugin/hooks/hooks.json`、`docs/enforcement.md` |
| mattpocock/skills | mattpocock/skills | 6654f6b6（既有审计钉定，未重读） | — | MIT | 引用 [methodology-mattpocock-skills-20260902.md](./methodology-mattpocock-skills-20260902.md) |

对 landscape 结论的两处校准（不改其正文，记在这里供复核记录引用）：

- landscape 第三节写 Gas Town「合并路径机械关单（gates 通过 + 推送验证后由 Go 代码带 commit SHA 关闭）」。本轮核对：**关单**那一步（`gt mq post-merge`）确是 Go 代码带 merge proof；但**合并**本身在 HEAD 由 Refinery agent 按 `mol-refinery-patrol` 清单手打 `git merge --no-ff` / `git push` 并肉眼比对 SHA，Go 的 `Engineer.doMerge` 管道完整存在但 `ProcessBatch` 无调用方。「测试红是分支引起还是 pre-existing」这道门是散文门。
- landscape 写 Witness「无证据即重置」。本轮核对：HEAD 的 zombie 路径是 restart-first（有 pending MR 不动、分支已并入则 nuke、否则重启会话）；重置只发生在会话与目录都没了的 orphan 路径，且先 `verifyCommitOnMain`，HEAD 已是 main 祖先就**关单**而不是重置。定性「有证据即关、无证据即重置」更准。

## 复核记录

（发布后追加。）

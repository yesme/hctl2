# P1 收口计划：hctl2-tool 的现场 Git 职责

> 状态：待拍板 · §四的三项取舍等所有者一句话；其余按所有者 2026-09-04「出计划即开工」执行，分工 Grok / Codex 主写、Claude / GLM 主审<br>
> 基线：main @ `033da13`（草案 v0.16.5）<br>
> 去向：`src/apps/hctl2-tool`、`docs/research/sdk/git.md`、`docs/usage.md`、`src/README.md`；不改约束层

起点是所有者问 Codex「下一步是什么」得到的提案（2026-09-04），本文由 Fable 复核后改写成可派工的计划。读它之前先看 [`README.md`](./README.md) 状态板；接手的会话从状态板第一个未完成阶段接着做。

## 一、定位与重述

交付文档把施工顺序定为 P0 探路 → P1 备装 → P2 接钥匙 → P3 装门面（`docs/design/delivery.md` §实现阶段）。P1 有两半：打包本地 Agency 参考实现（运行时为 Herdr），以及实现 `hctl2-tool` 的现场 Git 职责。前一半已经就位——#159 把 Herdr 连同其余四个服务交给 Process Compose，#162 在三平台验过拉起、就绪、重启、关停，`smoke.sh` 回读 Herdr 协议版本与 API 快照；可用性申报要向 control 报名册，等 P2 有 control 再建。后一半只做了 `wait`（回读外部机械事实），工具箱最核心的五项职责一项没动。

五项职责是所有者 2026-08-31 定的界（`docs/design/references/decision-history.md` §33）：worktree/ChangeSet 物化与隔离、已持久化意图的执行与回读、现场 OS 锁与 fence、封存保全、判决结晶副本写入；进程级动作一律转调业界工具，零重实现。P1 的硬边界（`delivery.md` §实现阶段 P1 行）：不产生 HCTL metadata 或 Receipt，不得称为自举。

所以这份计划要解决的问题只有一个：**把 `hctl2-tool` 从「只会 wait」补到「五项现场职责在三平台打包后可用」**，同时把 P2 需要的接口形状一次定对，让 control 到时候只是换一个调用方，不重塑工具箱。不在范围内的：control、公共 CLI、Workbench、五家 SDK 接入、构建系统再优化。

## 二、对 Codex 提案的复核

提案的骨架成立：现状判断五条全部核实（三平台构建与离线包可用；`hctl2-tool` 只有 `wait`；control / CLI / Workbench 不存在；远端无开放 issue 与 PR；顺序 P1 → P2 → P3），「第一份 PR 只做仓库检查 + 现场锁 + 隔离 worktree 物化，封存与集成留给下一份」这个切法也对——破坏性 Git 动作单独审。逐条裁决如下，「修正」与「补」是与提案字面不同的地方：

| 提案条目 | 裁决 | 依据与改法 |
| --- | --- | --- |
| 下一步是 P1 剩余部分，不继续优化构建、不提前开 Workbench | 维持 | `delivery.md` §实现阶段 |
| 第 1 步：调用官方 git CLI，不链接或重写 Git | 维持，补前置 | 与决策史 §33「转调业界工具，零重实现」一致。但 git 二进制是工具箱的新依赖，按纪律先落 `docs/research/` 对象文件（版本下限、宿主 git 还是随包、gitoxide / libgit2 为何不用），再写代码 |
| 第 1 步：读取 common-dir、HEAD、ref 与仓库身份 | 修正 | 「仓库身份」要拆成三个不同的东西：Git 里的稳定 Repo 身份（跟踪文件，P2 注册命令写入）、Git 公共目录身份（本现场，同一公共目录重试返回原现场）、辅助证据（远端 URL、目录名、HEAD）。P1 只做无副作用读取，三者在输出里分开；身份写入随 P2 的注册命令 |
| 第 1 步：用 OS 文件锁保护现场 | 维持，补边界 | 锁文件在 `<git-common-dir>/hctl2/lock`，复用 `hctl2-foundation` 已有的标准库排他锁，不写 PID 文件；锁文件须在本地文件系统。P1 只需「一次命令一把锁」的互斥；「整个执行期间占住现场、与 `site_generation` 伴生」的长持锁要有 control 才有意义，P2 再做，但 P1 的锁模块要为它留位置 |
| 第 1 步：创建和核验隔离 worktree/ChangeSet | 修正措辞 | 工具箱不「创建 ChangeSet」——ChangeSet 是账本对象，ID 由调用方给。工具箱做的是按给定 ChangeSet 引用与基线**物化**一个独立 worktree 与分支，并能回读核验它还在、还对。checkout 不能放进 `<git-common-dir>/hctl2/`（那是可丢弃缓存目录，而未提交修改可能是孤本） |
| 第 1 步：结构化 JSON 输出、不产生 metadata / Verdict / Receipt | 维持 | 沿用 `wait` 的记录形状（schema 名、`evidence_level: toolbox_readback`、语义退出码、稳定错误码） |
| 第 2 步：封存 tracked/untracked、清理不丢孤本、固化 ChangeSet Revision | 维持，补边界 | 约束原文在 `spec/participant.md` §ChangeSet 与 Git 事实。工具箱只产出 Git 事实（`base_commit_sha`、`result_tree_sha`、快照提交），Revision 的 ID、`producer_ref`、`revision_digest` 归账本。封存不能动 Harness 的索引与 HEAD，结果树要从某个 ref 可达（防 GC）。拆除默认保全，丢弃只在调用方显式确认时发生；保全失败必须留下精确路径、Git 状态和类型化恢复动作 |
| 第 2 步：`git update-ref <ref> <new> <old>` 做比较并交换，再回读 | 维持，补策略与幂等 | 除 CAS 外还要校验：待集成提交的树等于 `result_tree_sha`、基线是预期目标头的祖先、预期目标头等于当前目标头。策略在 P1 只做快进与合并提交两种（见 §四）；重跑同一意图必须幂等（目标已等于新值 → 报「已生效」，不是失败）；中断按结果未知返回恢复动作 |
| 第 2 步：只执行决定，不判断该不该合入 | 维持 | 三条底线之一「合入钥匙不进工具」的工具箱侧 |
| 第 3 步：P1 端到端验收含 Herdr 启动 / 观察 / 停止 | 修正 | 这半已由 #159 / #162 覆盖，不重做；P1 收口验收只补工具箱在三平台**打包后**的端到端用例 |
| 第 3 步：锁竞争、崩溃、脏树、重复调用的失败测试 | 维持，改分工 | 失败用例是主审的核验对象；写在各 PR 里，由审方逐条对抗 |
| 第 4 步：开始 P2/B0（账本、唯一 writer、代次、RPC 选型） | 维持，移出本计划 | 这是下一份计划的入口。Fable 可先做 P2 前置调研（RPC / schema 按 `.memo/notes/control-api-schema-20260902.md` 的触发点；SQLite 迁移方式），等所有者说开 |
| 第 5、6 步：P0 探针按首次消费前完成；B1 → B5 → P3 | 维持 | 与 `delivery.md` §开工前限时验证、§自举阶段逐字一致 |
| 提案漏掉的两项职责 | 补 | §33 的第五项「判决结晶副本写入」与「不可变正文写入/回读」（Task / Workflow Revision、Memo、Artifact 正文进 `<repo>/.hctl2/`）提案没提。它们的格式由 control 定，按「首次消费时完成」延到 B1 前；但 P1 要把工具箱的通用原语接口（把一段字节写到某 ref 的某路径并回读定位符与摘要）留出来，不能等 B1 再重塑命令面 |
| 提案漏掉的一项回读 | 补 | Skill digest 回读（本地 Agency 申报的 Skill 由工具箱回读记 known / unknown）。目录级摘要 `wait path-digest` 不覆盖，B2 前补，不在 P1 |

## 三、计划

### PR 序列与分工

分工按所有者 2026-09-04 的话：**Grok 与 Codex 主写，Claude（Fable）与 GLM 主审**。写作与调研仍归 Fable。

| 序 | 内容 | 写 | 依赖 |
| --- | --- | --- | --- |
| 0 | `docs/research/sdk/git.md`：git CLI 作现场引擎的对象文件 | Fable | 无；半天内落，与甲并行 |
| 甲 | 仓库检查（三种身份分开读）、现场锁、隔离 worktree 物化与核验 | Codex | 0（PR 调研节引用它） |
| 乙 | 封存、保全、拆除 | Grok | 甲 |
| 丙 | 本地集成：校验 → CAS → 回读，幂等与结果未知 | Codex | 甲；与乙并行 |
| 丁 | P1 收口：打包后三平台端到端用例、`docs/usage.md` 与 `src/README.md`、状态板 | Grok | 乙、丙 |

乙与丙都只依赖甲，分别是两个子命令，可以并行；甲要把模块布局定下来，让乙丙各占各的文件，少打架。

### 审核方式

- 每个 PR 由 Fable 与 GLM **各自独立**审，结论以 PR 评论给出，逐条「维持 / 修正 / 推翻」，每条引用约束原文或本计划的条款。审的重点：三条底线的工具箱侧、保全默认、回读与幂等、不产生账本语义、接口是否为 P2 留对了位置、失败用例是否真的堵住。
- 作者开 PR **不自合**；两份评论到齐、「修正」项在同一 PR 改完后作者合。「推翻」回到本计划改任务书，不在 PR 里争。
- 审方不写产品代码；审出的缺测试用例写成要求派回作者。

### P1 收口的验收（DoD）

1. `hctl2-tool` 具备：仓库检查、现场锁、worktree 物化与核验、封存、保全拆除、本地集成六个子命令，全部只经 `git` 二进制操作仓库，每次调用一条 JSON 记录。
2. 三平台 CI 在**打包后**的发行物上跑通一条链：临时仓库 → 物化 → 在 worktree 里改文件并提交 → 封存 → 集成到目标 ref → 拆除；并覆盖锁竞争、中途被杀、脏树、重复调用、预期目标头不匹配五类失败。
3. 没有账本、没有 Receipt、没有远端副作用；`docs/usage.md` 明说工具箱独立运行只提供普通本地操作。
4. 状态板全绿后由 Fable 收口：`src/README.md` 的 P1 描述、研究层索引、本目录状态板；决策史不加行（P1 就位是里程碑不是转折）。

## 四、需要所有者一句话的取舍

按「方向、边界、取舍找 human；接口细节 for agent」的纪律，只列三项，每项附我的建议，不说就按建议走：

1. **集成策略的 P1 集合**：`delivery.md` §未决问题 里「ChangeSet/PR 默认基数与集成策略」尚未定。建议 P1 只实现两种——快进（目标头必须是基线且待集成提交以它为祖先）与合并提交（用 `git merge-tree --write-tree` 加 `commit-tree` 在不碰任何工作树的情况下生成，冲突即拒绝）；rebase、squash 不做。策略是调用方输入，工具箱不选。
2. **git 来源**：建议用宿主 git，不随包（与 `ps` 同类，与随包的 `gh` 不同——gh 是为了复用用户登录并钉 `--json` 输出形状）。版本下限定在 Ubuntu 24.04 LTS 自带的 2.43，`merge-tree --write-tree` 需要 2.38 起；本机 macOS 26 的 Xcode git 是 2.50。工具箱启动时读版本，低于下限就拒绝并说明。
3. **被忽略文件（`.gitignore` 命中）算不算修改**：约束写的是「已跟踪、未跟踪且尚未封存的修改」。建议不算：封存不收它们，拆除时把它们列进残留记录（路径与大小，封顶条数）后随 worktree 一起删；调用方可以要求「有被忽略文件即拒绝拆除」。否则每个 worktree 都有 `buck-out`，拆除永远要人确认。

## 五、任务书

### 共同约束（甲乙丙丁都适用）

- PR 描述三节按模板；甲的调研节必须引用 `docs/research/sdk/git.md`；新增脚本要论证。
- 只经 `git` 二进制操作仓库，`HCTL2_GIT` 可覆盖路径（与 `HCTL2_GH` 同款）；调用时显式指定仓库与工作树，不依赖当前目录与 `GIT_DIR` 类环境变量；不链接 libgit2，不用 gitoxide。
- 输入就是意图的字段：ChangeSet 引用、基线、目标 ref、预期头、幂等键都由调用方给；工具箱不发明 ID，不读账本，不写账本。
- 输出沿用 `wait`：一次调用一条 JSON 记录，`schema` 带版本，`evidence_level: toolbox_readback`，语义退出码，稳定错误码 `HCTL2_TOOL_*`。
- 不做远端副作用（push、PR、merge 归 adapter），不做 lint 与代码检查；碰到就拒绝并给错误码。
- 每个子命令自带失败用例；三平台 Buck 测试绿；`--help` 英文，文档中文。
- 「张力」写「冲突」；位置引用写「文件 §节名」。

### 0 · Fable：`docs/research/sdk/git.md`

对象：git CLI 作为工具箱的现场引擎。回答：为何采用二进制而非 gitoxide / git2（决策史 §33 的分层原则、跨平台构建、功能覆盖）；版本下限与依据（`worktree`、`rev-parse --git-common-dir`、`update-ref` 的旧值比较、`merge-tree --write-tree`、`status --porcelain=v2`、`worktree list --porcelain -z` 各自的最低版本）；三平台宿主 git 的实际版本；哪些用户配置与 hook 会影响工具箱的结果（`core.hooksPath`、`reference-transaction`、`post-checkout`、`core.autocrlf`、`merge.conflictstyle`），工具箱怎么隔离；宿主 git 与随包的取舍。决定写成「采用二进制」。

### 甲 · Codex：仓库检查、现场锁、worktree 物化

1. **仓库检查**（无副作用）：给一个路径，输出 Git 公共目录（规范化绝对路径）、当前 worktree 列表、HEAD 与指定 ref 的值、Git 里的稳定 Repo 身份（若 `<repo>/.hctl2/repo.toml` 存在则读，不存在就报缺失，不造）、辅助证据（远端 URL）。三者在记录里分三个字段组，不合并。裸仓库、子模块、非仓库路径各有明确错误码。
2. **现场锁**：`<git-common-dir>/hctl2/lock`，复用 `hctl2-foundation` 的排他锁；每个会改现场的子命令进入前取锁、退出释放；拿不到锁立即返回「被占用」并带持有信息（能读到多少给多少），不等待。检测锁文件所在文件系统不是本地（NFS / CIFS）时拒绝并说明。模块设计给 P2 的长持锁留接口，本 PR 不实现长持。
3. **worktree 物化**：输入 ChangeSet 引用与基线提交；在调用方给定的根目录下建独立 worktree 与分支（分支命名由你定，要能从名字认出 ChangeSet），checkout 不进 `<git-common-dir>/hctl2/`。重复调用同一 ChangeSet 引用返回同一 worktree（幂等），基线不同则拒绝。Harness 在里面要能正常 `git log / fetch / diff / commit`。
4. **核验**：给 ChangeSet 引用，回读 worktree 是否存在、分支是否仍指向它、HEAD 是否以基线为祖先、工作树干净还是脏（tracked / untracked 分开计数）。
5. 失败用例：非仓库路径、锁被另一进程持有、根目录不可写、基线提交不存在、同一 ChangeSet 用不同基线重复物化。

### 乙 · Grok：封存、保全、拆除

1. **封存**：在有效 worktree 上，把已跟踪与未跟踪（不含被忽略）的当前内容写成一棵树，用临时索引，不动 Harness 的索引与 HEAD；以 HEAD 为父生成快照提交并挂到工具箱自己的 ref 命名空间下（防 GC）；输出 `base_commit_sha`、`result_tree_sha`、快照提交 sha。同一内容重复封存得到同一树 sha（幂等）。
2. **保全确认**：拆除前先封存，再比对工作树状态，证明所有未封存修改已有可达副本；证明不了就拒绝拆除，记录里给精确路径、`git status` 摘要与恢复动作（比如「先封存」「先提交」）。
3. **拆除**：默认保全路径；`--discard`（名字你定）只在调用方显式给出确认时走，不留副本直接拆。被忽略文件按 §四第 3 项处理。拆除后 `git worktree prune` 只清本次这一个，不碰其他 worktree。
4. 失败用例：封存中途被杀（重跑要能收敛）、拆除时有未封存修改且无确认、worktree 目录被手工删了一半、分支被 Harness 改到别处、锁被占。

### 丙 · Codex：本地集成

1. 输入：待集成提交、`base_commit_sha`、`result_tree_sha`、目标 ref、预期目标头、策略（快进 | 合并提交）、幂等键。
2. 校验顺序：提交的树等于 `result_tree_sha` → 基线是预期头的祖先或等于它 → 当前目标头等于预期头；任一不成立返回类型化结果（树不匹配 / 基线不是祖先 / 预期头不匹配即漂移），不改任何 ref。
3. 执行：快进直接 CAS；合并提交用 `merge-tree --write-tree` 得到新树，冲突即拒绝并列冲突路径，再 `commit-tree` 生成合并提交；最后 `update-ref <ref> <new> <old>` 做 CAS。
4. 回读：CAS 后重新读目标 ref，记录前后值；重跑同一幂等键且目标已等于新值 → 报「已生效」；进程在 CAS 后、回读前被杀，下次调用按结果未知先回读再判定。
5. 失败用例：预期头被别人推进、树不匹配、合并冲突、目标 ref 不存在、CAS 竞争（两个进程同时集成同一目标）。

### 丁 · Grok：P1 收口

1. 打包后三平台端到端：在 `src/packaging/release/test-package.sh`（或同级 Buck 测试）里，用发行物内的 `hctl2-tool` 对临时仓库跑通 DoD 第 2 条整条链与五类失败。
2. `docs/usage.md` 的 `hctl2-tool` 一节改写成六个子命令的用法；`src/README.md` 的 P1 描述更新；研究层索引加 `sdk/git.md` 一行。
3. 状态板收口交 Fable。

### 审 · Fable 与 GLM

每个 PR 一份独立评论。逐条对抗核验的清单：

- 三条底线：Harness 不需要任何 HCTL 凭据就能在 worktree 里工作；工具箱没有任何入口能凭调用方一句话改目标 ref（除集成子命令的 CAS）；worktree 与分支确实独立。
- 保全默认：找一条能删孤本的路径，找到即「推翻」。
- 回读与幂等：每个改现场的子命令重跑一次结果一致；中途被杀后能收敛。
- 账本语义泄漏：输出里出现了工具箱发明的 ID、Receipt、Verdict 类字段即「修正」。
- P2 接口位：意图字段是否都能从命令行给全；长持锁与不可变正文写入的位置是否留了。
- 失败用例：逐条看是否真的堵住，而不是测了「返回非零」。

## 六、延后与遗留

- 不可变正文写入/回读与判决结晶副本：B1 前，随第一个消费者（Task Revision 正文）做；本计划只要求接口留位。
- Skill digest 目录级回读：B2 前。
- 长持现场锁与 `site_generation` 伴生：P2 B0，control 出现时。
- 稳定 Repo 身份写入 Git（跟踪文件 `<repo>/.hctl2/repo.toml` 需要一次提交——落在哪个 ref、由谁提交）：P2 注册命令的设计题，P1 不碰。
- Herdr 可用性申报（本地 Agency 向 control 报名册）：P2 B2 前。
- Grok 在 #159 留的脚注：P2 写准入时必须继续拒绝模型转述。
- P2 前置调研（RPC / schema、SQLite 迁移方式）：Fable，等所有者说开。

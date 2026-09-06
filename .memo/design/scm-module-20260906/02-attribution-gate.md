# 提交署名的机械关卡：任务书

> 状态：已拍板 · 做法照 abacistopia（所有者 2026-09-06：「所有的 harness 怎么写邮箱，都看 abacistopia 那边」）；仓库设置一项等所有者动手<br>
> 基线：main @ `1d001ad`（草案 v0.17.0）<br>
> 去向：`src/agency/attribution/`（从 abacistopia 移植 `coauthor.py` 与 `hooks/commit-msg`）、`.github/workflows/pr-contract.yml` 加一步、`AGENTS.md` 加「提交署名」一节、仓库设置只留 merge commit 并让合并提交带 PR 描述；不改约束层

所有者 2026-09-06 提出：coauthor 要有纪律，并且要机械执行。`01-scm-module.md` 附录二记了审计结论，四家评审都维持「另开小 PR、与第五模块方案分开」。本文是那份小 PR 的任务书；v2 按所有者的指向改成照搬 abacistopia 的做法，不再自己发明白名单。

## 一、abacistopia 那边怎么做（2026-09-06 核对）

- **一种尾注形状，六家 harness 各一个 GitHub App 机器人邮箱**：`Co-authored-by: {模型} ({effort} effort) <机器人邮箱>`。邮箱是固定的，模型与 effort 从当前会话的证据里读出来，不硬编码——模型切换或恢复会话后配置文件会漂，只有会话记录不会。

  | 家 | 机器人邮箱 | 尾注例子 |
  | --- | --- | --- |
  | Claude（Claude Code） | `281844019+a-claude-code-bot[bot]@users.noreply.github.com` | `Claude Fable 5 (max effort)` |
  | Codex | `281847692+a-chatgpt-codex-bot[bot]@users.noreply.github.com` | `Codex GPT-5.6 Sol (max effort)` |
  | GLM（经 OpenCode） | `281846436+a-glm-code-bot[bot]@users.noreply.github.com` | `GLM 5.2 (max effort)` |
  | Grok（Grok Build） | `302482056+a-grok-build-bot[bot]@users.noreply.github.com` | `Grok 4.5 (high effort)` |
  | Gemini（Antigravity） | `295901900+an-antigravity-cli-bot[bot]@users.noreply.github.com` | `Gemini 3.6 Flash (high effort)` |
  | Kimi | `281852327+a-kimi-code-bot[bot]@users.noreply.github.com` | `Kimi K3 (max effort)` |

- **提交的作者是人，harness 是共同作者**：共享 `.git/config` 里 `user.email` 设为所有者的 `jacky.chao.wang@gmail.com`，所以每个提交的作者都是 Yesme，机器身份只出现在尾注里。hctl2 这边没设，所以我们的提交作者是这台 Mac 的默认身份 `Haibara AI <haibara@JackydeMac-mini.local>`，GitHub 在 squash 时还会把它当成一个不认识的共同作者补进来——纯噪音。
- **一个生成器，一个钩子**：`scripts/coauthor.py <harness>` 读各家会话证据生成尾注（Claude 读会话记录里最近一条 `message.model` 加 `$CLAUDE_EFFORT`；Codex 读 rollout 的 `turn_context`；OpenCode 读会话库的 `modelID` 与 `variant`；Kimi 读会话日志；Antigravity 读会话日志的模型覆写），任一字段读不到就报错、不猜。`scripts/hooks/commit-msg` 在 `claude/*`、`codex/*`、`glm/*`、`grok/*`、`kimi/*`、`agy/*` 分支上核对提交消息里必须有与当前会话完全一致的那条尾注；钩子经共享 `.git` 的 `core.hooksPath` 装一次覆盖全机 worktree。
- **合并只走 squash**（`gh pr merge --squash`），禁普通 merge；abacistopia 是免费私有仓、开不了分支保护，只能靶后审计；hctl2 是公开仓、分支保护齐全，可以在 PR contract 里真拦。
- 明写的一条纪律：**给别家署 co-author 时只用它已验证的真实尾注（取自它的历史提交），不猜**；猜错等于错误归因。
- 顺带发现：Fable 在 hctl2 现用的 `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>` 正是 abacistopia 的 CLAUDE.md 明令不用的形式（缺 App 机器人邮箱、缺 effort）。本 PR 落地后改用上表的形式。

## 二、hctl2 要做的事

1. **移植生成器与钩子，放在本地 Agency 参考实现之下，不进 HCTL 主体**（所有者 2026-09-06：署名能力本质上是「本地 Agency」对「本地 Participant」的一个实现，逻辑上属于本地 Agency，不该泄露到 HCTL 主体）。落点是 `src/agency/attribution/`（与技能目录 `src/agency/skills/` 并列）：`coauthor.py` 从 abacistopia 复制（保留出处注释），只保留 hctl2 会用到的家（Claude、Codex、GLM、Grok、Antigravity），去掉 abacistopia 专有的 `./run --session-id` 与 `COAGNET_*` 环境变量路径，只认各家原生会话证据；`hooks/commit-msg` 原样移植，分支前缀表加 `fable/*` 映射到 claude。为什么这归 Agency：知道「Claude 的会话记录在哪、Codex 的 rollout 长什么样、OpenCode 的会话库怎么读」是派出方对自己派出的执行体的了解，HCTL 主体不该有这种各家私货。到了产品里这件事反而变简单：执行体在 ChangeSet 工作树里的提交，其模型与 effort 由 Execution Spec 冻结、Agency 交付时申报，尾注从冻结值生成，不用去翻会话文件——现在这套脚本是 HCTL 还没有的时候的过渡品。这两个是新增脚本，PR 的调研节要写清：不用现成的 DCO / commitlint 类 GitHub Action，因为它们检查的是 `Signed-off-by` 或提交格式，不认厂商机器人邮箱，也读不到会话证据。
2. **作者身份**：已设（2026-09-06，Fable 代做）：`/Users/haibara/workspace/hctl2/.git/config` 的 `user.name`/`user.email` 为 Yesme / `jacky.chao.wang@gmail.com`，共享 `.git` 下的全部 worktree 生效；此后 hctl2 的提交作者是 Yesme，共同作者是尾注里的机器人。`core.hooksPath` 等钩子移植进仓库后再设，每台机器一次。
3. **PR contract 加一步**（在现有那一步里加十几行，不新增脚本文件）：对 `origin/<base>...HEAD` 逐个提交，头分支前缀在上表的 harness 映射里时，每个提交必须至少有一条 `Co-authored-by:` 尾注，值匹配 `^.+ \(.+ effort\) <[0-9]+\+an?-[a-z-]+-bot\[bot\]@users\.noreply\.github\.com>$`，且邮箱属于该分支前缀对应的那一家；不匹配就失败，错误信息指出哪个提交、缺哪家的尾注。头分支不在映射里（所有者手工开的分支）不检查——责任仍由 PR 发起者、合并者与 squash 提交记录。这是护栏不是安防，和 abacistopia 的定位一样。
4. **`AGENTS.md` 加「提交署名」一节**，四句：每个 harness 产出的提交末尾带本家的 `Co-authored-by` 尾注，用 `python3 src/agency/attribution/coauthor.py <harness>` 生成、不手写；合并只用 squash；给别家署名只用它已验证的真实尾注；提交与 PR 描述不带 harness 专有的会话链接，留痕写 HCTL 层面的引用。Fable 的 CLAUDE.md 侧默认尾注与会话链接规则由本节覆盖。
5. **仓库设置**（只有所有者能改，GitHub 网页：Settings → General → Pull Requests）——**改为只留 merge commit**（所有者 2026-09-06：不要把 commit 只留在 PR 里，将来做复盘未必看得到 PR，但一定看得到 commit）。取消勾选「Allow squash merging」与「Allow rebase merging」，只留「Allow merge commits」；把「Default commit message」设为「Pull request title and description」，这样 PR 描述的三节也进合并提交，复盘只看 git 就够，不用把整个 PR 历史用几百次 API 调用捞回来。「Require linear history」不开也不能开：merge commit 本来就不线性，主线按第一父提交读（`git log --first-parent`）就是 PR 粒度的故事，展开第二父链就是评审轮次里逐次的修改。abacistopia 那边是 squash，这里有意不同：那边的 PR 多是单人单提交，这里的 PR 是多家审、多轮改，过程本身是证据。
6. **提交说明的纪律**（配合上一条）：PR 里每个提交都要写清改了什么、为什么；回应评审意见的提交写明回应谁的哪一条（例如「按 GPT 第二轮第 1 条：集成意图的唯一性按意图、互斥按目标」），让复盘时光看 `git log` 就能重建审、改、审、改的链条。不推 `wip` 一类空壳提交：推之前在本地整理成有意义的单元；一旦推上去、被审过，就不再改写（主线只进不改，PR 分支同理）。每个提交照旧带本家的 `Co-authored-by` 尾注，因为提交会原样进主线，尾注是逐个提交的归因。

## 三、所有者问的三件

- **要求「线性」对不对，行业推荐是什么。** 行业没有唯一推荐，有三种合并方式各配一种历史观：merge commit 保留分支拓扑与每个中间提交，Linux 内核与 Git 自己这么干，代价是 `git log` 与 `git bisect` 要在网状历史里走；rebase 合并让主线线性且保留每个中间提交，前提是作者把提交整理干净；squash 把一个 PR 压成主线上的一个提交，Google 式的「一次评审一个提交」，代价是丢掉 PR 内部的提交粒度。GitHub 的官方建议只有一条：一个仓库选定一种、用设置限制住，别混用。「Require linear history」这个开关是给选了 squash 或 rebase 的仓库防止有人推 merge commit 进主线用的——它是结果不是目标。本文 v3 曾建议 squash；所有者 2026-09-06 裁定改为 merge commit（理由见第二节第 5 条：复盘要看得到过程提交），于是「要求线性」这个问题自然消失——主线不线性是有意的，开关不开。顺带一提，产品里合并方式不是全局设置而是每个目标的策略：集成意图固定「合并方式」，平台绑定声明允许哪几种（`spec/repo.md` §集成）；P1 的本地 `integrate` 做了快进与合并提交两种，squash 由 GitHub 路径提供。
- **审-改-审-改的过程本身有没有价值、代价多少（所有者 2026-09-06 追问）。** 有价值，而且是设计里本来就要记的东西：Run 模块的 Verdict（席位对精确版本的裁决）、Evidence 与返工次数，正是「参与者在各维度上的能力」的事实来源，做复盘或评估参与者时查的就是它们。现在没有 control，这些事实住在 GitHub 的 PR 线程里；只留 squash 不会丢掉它们——GitHub 在合并、删分支之后仍保留 PR 的每个提交与全部评论，评审轮次、谁发现了什么、作者怎么处置都还在。丢的只是这些东西在克隆里的副本，「可替换的接口不等于可迁移的历史」。所以代价分两层：留在平台上零成本；要它跟着仓库走，就在每轮收口时用 `gh` 把 PR 线程导出成 markdown 存进 `.memo/review/<日期>-<议题>/`（这个目录的定义本来就是「一轮一个子目录、每家一份」），按里程碑做、不必每个 PR 做。至于评估参与者能力，等 Run 模块有了 Verdict 与 Receipt，它就是账本上的一个查询（哪位的裁决后来被推翻过几次、哪位的返工几轮过关），已列进 P2 计划的延后清单。所有者 2026-09-06 的裁定正是这一条：改用 merge commit，让 PR 内部的逐次提交连同「为什么改」留在克隆里，复盘只看 git；导出 PR 线程降为可选，只在需要评审者原话时做。
- **会话链接是什么、要不要留。** Fable 的提交末尾除署名外还有一行 `Claude-Session: https://claude.ai/code/session_…`，PR 描述末尾也有同一个链接，指回产生这次改动的 Claude Code 会话。它不是公开可访问的：只有登录了那个账号的人能打开，别人点开是登录页。所以在公开仓库里它对读者没有信息量，还顺带暴露了我们的内部工具形态；所有者的判断对。在 HCTL 的语境下，可追溯应该回到 HCTL 自己的对象：产品里 ChangeSet Revision 的 `producer_ref` 指向精确的 Invocation 或 Attempt，账本再从那里连到 Execution Runtime 与会话，这条链不靠提交消息里的链接。**定（所有者：链接该留，但该留的是 HCTL 生态内部的）**：不检查 harness 会话链接，hctl2 的提交与 PR 描述不再带它；留痕写 HCTL 生态内的引用——现在是 PR 编号与备忘路径，有了 control 之后是 Task、ChangeSet 或 Invocation 的稳定 ID，指回账本里的对象。`AGENTS.md` 那一节加这一句；Fable 从本 PR 落地后的下一次提交起去掉 `Claude-Session:` 行。
- **「我去 GitHub 里改？」** 是，只有第二节第 5 项要在 GitHub 网页上动手；第 2 项已由 Fable 代做。

## 四、分工

`src/agency/attribution/` 两个文件的移植、`pr-contract.yml` 一步、`AGENTS.md` 一节由 Codex 写（代码归 Grok/Codex 的分工），Fable 审；PR 的调研节引用本文第一节与 abacistopia 的 `GIT-PROTOCOL.md` §1、§2、§5。落地后 Fable 与各家在下一次提交起改用新尾注、去掉会话链接；旧提交不改写（主线只进不改）。

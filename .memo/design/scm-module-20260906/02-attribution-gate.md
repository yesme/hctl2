# 提交署名的机械关卡：任务书

> 状态：已拍板 · 做法照 abacistopia（所有者 2026-09-06：「所有的 harness 怎么写邮箱，都看 abacistopia 那边」）；仓库设置一项等所有者动手<br>
> 基线：main @ `1d001ad`（草案 v0.17.0）<br>
> 去向：`scripts/coauthor.py` 与 `scripts/hooks/commit-msg` 从 abacistopia 移植、`.github/workflows/pr-contract.yml` 加一步、`AGENTS.md` 加「提交署名」一节、仓库设置只留 squash；不改约束层

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

1. **移植生成器与钩子**（复制代码，来源是所有者自己的仓库，保留出处注释）：`scripts/coauthor.py` 只保留 hctl2 会用到的家（Claude、Codex、GLM、Grok、Antigravity；Kimi 先留着不删，无害），去掉 abacistopia 专有的 `./run --session-id` 与 `COAGNET_*` 环境变量路径，改为只认各家原生会话证据；`scripts/hooks/commit-msg` 原样移植，分支前缀表加 `fable/*` 映射到 claude（Fable 现有 PR 分支的前缀）。这两个是新增脚本，PR 的调研节要写清：不用现成的 DCO / commitlint 类 GitHub Action，因为它们检查的是 `Signed-off-by` 或提交格式，不认厂商机器人邮箱，也读不到会话证据。
2. **作者身份**：所有者在这台 Mac 上给 `/Users/haibara/workspace/hctl2/.git/config` 设 `user.name`/`user.email` 为 Yesme / `jacky.chao.wang@gmail.com`，并 `git config core.hooksPath "$(git rev-parse --show-toplevel)/scripts/hooks"`（与 abacistopia 同一句）；每台机器一次。此后 hctl2 的提交作者是 Yesme，共同作者是尾注里的机器人。
3. **PR contract 加一步**（在现有那一步里加十几行，不新增脚本文件）：对 `origin/<base>...HEAD` 逐个提交，头分支前缀在上表的 harness 映射里时，每个提交必须至少有一条 `Co-authored-by:` 尾注，值匹配 `^.+ \(.+ effort\) <[0-9]+\+an?-[a-z-]+-bot\[bot\]@users\.noreply\.github\.com>$`，且邮箱属于该分支前缀对应的那一家；不匹配就失败，错误信息指出哪个提交、缺哪家的尾注。头分支不在映射里（所有者手工开的分支）不检查——责任仍由 PR 发起者、合并者与 squash 提交记录。这是护栏不是安防，和 abacistopia 的定位一样。
4. **`AGENTS.md` 加「提交署名」一节**，三句：每个 harness 产出的提交末尾带本家的 `Co-authored-by` 尾注，用 `python3 scripts/coauthor.py <harness>` 生成、不手写；合并只用 squash；给别家署名只用它已验证的真实尾注。Fable 的 CLAUDE.md 侧默认尾注规则由本节覆盖。
5. **仓库设置**（只有所有者能改，GitHub 网页：Settings → General → Pull Requests）：取消勾选「Allow merge commits」与「Allow rebase merging」，只留「Allow squash merging」；squash 提交消息保持现在的「commit messages」选项，这样各提交里的尾注会进主线的那一个提交。**「Require linear history」不用开**：所有者担心它限制太多，而只留 squash 之后主线本来就是线性的——线性历史开关只多防一件事：有推送权的人绕过 PR 直接推一个 merge 提交到 main，而 main 的分支保护已经要求走 PR、管理员也受约束，这条路已经堵死。

## 三、所有者问的两件

- **「会话链接」是什么**：Fable 的提交末尾除署名外还有一行 `Claude-Session: https://claude.ai/code/session_…`，PR 描述末尾也有同一个链接，指回产生这次改动的 Claude Code 会话，是 Claude Code 自带的留痕，别家没有对应物。**不列入检查**，它是自愿留痕不是纪律。
- **「我去 GitHub 里改？」**：是，只有第二节第 5 项要你在 GitHub 网页上动手；第 2 项是在这台 Mac 上敲两句 git config，可以由我代做，你说一声即可。其余由 Codex 写、Fable 审。

## 四、分工

`scripts/` 两个文件的移植、`pr-contract.yml` 一步、`AGENTS.md` 一节由 Codex 写（代码归 Grok/Codex 的分工），Fable 审；PR 的调研节引用本文第一节与 abacistopia 的 `GIT-PROTOCOL.md` §1、§2、§5。落地后 Fable 与各家在下一次提交起改用新尾注；旧提交不改写（主线只进不改）。

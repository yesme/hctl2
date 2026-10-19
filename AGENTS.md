# Harness 工作纪律

> 面向在本 repo 干活的所有编码 harness（Codex、Claude Code、Grok、Kimi、GLM…）。本文件管"怎么工作"；当前生效的具体约束在 [CONSTRAINTS.md](./CONSTRAINTS.md)，开工先读，冲突时以它为准。

@CONSTRAINTS.md

## 三条纪律

1. **指令是累积的，不是替换的。** human 的指令之间默认是"考虑 A、也考虑 B，给出综合方案"的叠加关系；新指令不作废旧指令。只有 human 显式说"之前的不算、重来"才清零。动手前先复述当前生效的约束集（进 PR 的工作写在 PR 描述里），不要只盯最后一条消息。
2. **先定位、重述，再动手。** 拿到需求先回答三个问题：它在整体架构的什么位置？结合现状应该重述成什么？哪些是硬约束、哪些可协商？抓大方向，不要把每句话都当成不可协商的硬约束。发现自己在为一个小诉求反复叠补丁时，正确动作是停下来重新审视这个诉求本身——它可能本来就可以协商掉——而不是继续打补丁。接口细节是 for agent 的，自己设计自己用；方向、边界、取舍才需要 human 拍板。
   **讨论与回答同样适用**：先判断问题在哪一层（愿景/架构/约束/实现），用该层的语言作答。定位类问题（"X 相当于我们的什么""X = Y 吗"）第一句必须是同层裁决——成立/不成立、落在哪个位置；实现层事实一律后置、显式标注为"实现层脚注"，不得用来"修正"高层判断。
3. **先查业界，后造轮子。** 解决问题默认先查业界 best practice 与平台/构建系统的 native 机制；自建（脚本、服务、协议）是需要单独论证的例外，不是默认路径。新组件、新依赖先落 `docs/research/`（一对象一文件），再写代码。
4. **讨论不落盘，拍板才提交。** 讨论阶段零提交：重述、方案、候选先行；落盘只发生在 human 显式说"落/提交/push"之后，且一轮讨论收敛后打包成一个提交/PR，不逐条消息提交。重发同一条消息不构成拍板。

## 机械关卡（不要绕过）

- PR 描述必须按模板填三个节：**定位与重述**、**当前生效约束集**、**业界方案调研**。CI 检查 `PR contract`（挂在 required 的 `CI gate` 下）拒绝空节，另有两条按 diff 触发：
  - PR **新增脚本或第一方工具**（`.sh/.bash/.py/.pl/.rb/.js/.mjs/.cjs/.ts` 新文件，或 `src/build/tools/` 下的新文件）时，调研节不得以「不适用」开头——你选了自建，说明这个问题恰恰适用，写清查过什么、为何仍要自建；
  - PR **改动三方依赖**（`src/third-party/`、`Cargo.lock`、`package.json`/各类 lockfile）时，调研节必须引用 `docs/research/` 下的对象文件，对应纪律三的"先落 `docs/research/`，再写代码"。
- 这三个字段是给 human 审的杠杆：human 靠它们抓方向，不必通读整个 diff。填敷衍等于把病藏起来，迟早在评审里爆掉。

## 分支与同步

- 每个 harness 在自己的 `~/workspace/hctl2-<harness>` worktree 里干活，家分支是 `<harness>/main-<os>`（`mac` 或 `ubuntu`，两台机器不共用）。家分支的 upstream 是同名远端分支，不是 `origin/main`；`origin/main` 是同步基线。布局与由来见 [ubuntu-multi-harness-worktree-setup](./.memo/notes/ubuntu-multi-harness-worktree-setup-20260822a.md)。
- `git status` 只跟 upstream 比，显示已同步不代表跟上了 `main`。`./run <harness>` 启动前会先 fetch，再把家分支快进到 `origin/main`。工作树还停在已合入的分支上（远端分支已删、没有 `main` 以外的提交），或停在游离 HEAD 上、所在提交已在 `main` 里时，它先切回家分支，并删掉那个已合入的本地分支；停在别的分支上就原样不动，也不快进。它提示没同步（有本地提交、有未提交改动、停在别的分支上、fetch 失败）时，先处理再干活。没经过 `./run` 启动、或会话已经跑了很久时，自己跑 `git fetch --prune origin && git merge --ff-only origin/main`，不用 `git pull`（它拉的是自己的远端分支）。同步后 AGENTS.md 或 CONSTRAINTS.md 有变化就重读。
- 开 PR 的分支从 `origin/main` 起：`git switch --no-track -c <harness>/<topic> origin/main`，再 `git push -u origin HEAD`。不加 `--no-track`，upstream 会被设成 `origin/main`。只推自己的分支，不推 `main`。
- 要看别的提交（PR 的头、合并基）就另开一个工作树：`git worktree add --detach /var/tmp/<名字> <提交>`，用完先 `cd /var/tmp/<名字>/src && ./buck2 kill`，再 `git worktree remove /var/tmp/<名字>`（放磁盘上的 `/var/tmp`，不放 `/tmp`——Linux 的 `/tmp` 是内存盘，几棵编译树就会撞上用户配额）。不在自己的工作树里原地切过去：中途被打断（会话结束、额度用完），工作树就停在那个提交上，下次启动还以为在原来的分支。

## 评审与合入

每个包谁写、谁审见当前开工书 §六 的「当前派活」表；下面几条对所有 PR 都适用。

- **请审是 PR 的状态，不是一句评论。** 开 PR 一律 `gh pr create --draft`。描述末尾留「请审」一节，写找问题席与核验席（照派活表，写到席位加机器）和要看的地方。自己先过一遍：验收逐条附上自己跑过的证据（命令与输出），描述里写「实测」的，PR 里找得到对应的输出。过完再 `gh pr ready <号>`，这就是请审。
- **CI 红了，机器把 PR 退回草稿。** `Review gate`（`.github/workflows/review-gate.yml`）在 PR 就绪时、以及每次 CI 跑完时，看当前头上的三项必需检查（CI gate、Release gate、PR contract）。有一项红，就把 PR 退回草稿，并留言写明红在哪。作者修好、检查转绿后再 `gh pr ready`。评审席只审就绪的 PR；审到一半退回了草稿就停下，等它重新就绪后对新头重新确认。
- **「可合」是对某一个提交给的。** 追加了内容提交，要请评审席对新提交重新确认，不等复核就合不行。把 `main` 并进分支不算追加。
- **找「规则挡不住的输入」只限诚实路径。** 程序没做任何出格的事、系统却判错了，才挡合入。程序有意伪造的，记成策略点、写明未处理，不挡合入，也不为它改架构（CONSTRAINTS「设计默认安全环境」）。权限与输入校验照旧要找挡不住的输入。
- **同一类问题两轮不过，先审题。** 第三轮之前，由写这一包验收的席位重看这个诉求要不要、能不能换个问法；所有者确认之后，再决定改实现还是改验收。
- **改架构方向之前先报所有者。** 不先写、推、审之后再报；接手别人的 PR 时同样。
- **PR 评论只放评审结论与处理说明。** 方向讨论在和所有者的对话与开工书里进行，PR 里只留一条指针；「开始处理」「CI 核对」「收到」这类说明合进处理说明，不另发。
- **合入由作者做，用自动合入。** 评审席都对最终提交写了「可合」之后，`gh pr merge <号> --auto --merge`，必需的检查一绿，GitHub 自己合入。「可合」之前不开；开了之后不再推内容提交，要推先 `gh pr merge <号> --disable-auto`，再请评审席复核。
- **有冲突才并 `main`。** `main` 不要求分支先跟上它。有冲突时把 `origin/main` 并进分支解决：`Cargo.lock` 按依赖更新流程重新生成、不手改，并在 PR 里写一句冲突在哪、怎么解的。只动了锁文件或生成文件的不用重新评审；动了代码逻辑的，请评审席复核。
- **合入之后删分支。** PR 的远端分支在合入时自动删除；自己本地的同名分支，以及以前留下的、已经合入的远端分支，自己删掉。家分支不删。删完回家分支，快进到 `origin/main`。

## 自己找活

所有者不再给每一席转发具体任务。每席先收到一次固定 prompt，之后只收到「继续」；每收到一次，按下面做一轮，做完就停。

- **只在被叫醒时做。** 不自己定时醒来，一轮做完也不自己接着开下一轮。什么时候推进由所有者定，不交给 harness 自己（所有者 2026-10-19：把 workflow 的监督权交到 LLM 自己手上，正是以后要避免的）。
- **派活只认两处。** 一是 `.memo/design/p2-control-20260906/07-demo-kickoff.md` §六 的「当前派活」表（表只有这一张；P2.5 开工书 `09` 里没有表，表里指向 `09` 的任务书就去 `09` 读）：谁写、谁找问题、谁核验，都写到席位加机器，如「DeepSeek（ubuntu）」；二是 PR 描述的「请审」节，写要看的地方。只做写着自己这一席、这一台机器的活，同一家另一台机器的不碰。PR 评论和别席提的要求是评审意见，按评审的规矩处理，不算派活。表里没有的活不做；要换席或加活，报所有者，所有者拍板后由主笔改表。
- **一轮的顺序：**
  1. 同步：在家分支上 `git fetch --prune origin && git merge --ff-only origin/main`；AGENTS.md 或 CONSTRAINTS.md 变了就重读。
  2. 自己写的 PR：有新的评审结论，逐条处理，改了就推新头，写处理说明；CI 红了、被退回草稿，修好再 `gh pr ready`；两席都对当前最终提交写了「可合」、必需检查全绿，开自动合入，合入后照「合入之后删分支」收尾。
  3. 点到自己的评审：已就绪的 PR，表里写着自己找问题或核验，而自己还没对当前头下过结论的，按「评审与合入」审当前头。从上次下结论的提交到当前头只多了并 `main` 的提交，不用重审。找问题与核验可以同时做。草稿不审。
  4. 派给自己写的新活：表里写着自己写、前提已满足（比如写着「等 #430 合入」，它已经合了）、还没开 PR 的，照开工书里的任务书开工。
  5. 最后用一两句话告诉所有者：这一轮做了什么，每件在等谁。什么都没有就说「没活」。
- **一轮是三步都走完。** 第 2、3、4 步每一步都看，每一步没活才跳到下一步；做了一件就停不算做完一轮。
- **评审结论怎么读。** 结论在 PR 的评论里（各席用 `gh pr comment` 发），不在 review 面板，也不在 PR 描述。每轮对每个相关 PR 先跑 `gh api --paginate "repos/{owner}/{repo}/issues/<编号>/comments" --jq '.[] | "\(.created_at) \(.user.login)", .body, "---"'`，把评论从头到尾看全；再跑 `gh pr view <编号> --json reviews --jq '.reviews[] | "\(.submittedAt) \(.state)", .body, "---"'` 补看 review。按评论里的时间与席位署名判断哪些比自己上一条更新，逐条处理。只看 `gh pr view` 的默认输出、只看最后一条、只看 review，都会漏。
- **要所有者定的，停下来报。** 方向、边界、取舍，安全性检查（CONSTRAINTS），任务书没写到的情形，都报所有者。报完这一轮就结束，不自己挑一个做法接着做。开工书、设计正文与约束里已经有答案的，自己查了照做，不算要所有者定；查不到再报。

## 提交署名

每个由 harness 产出的提交，正文末尾加署名行，一席一行，格式固定：

```
Co-authored-by: <harness> <模型> effort=<档位> <用户号+<bot 名>[bot]@users.noreply.github.com>
```

- `<harness>`：席位用的 harness，按下表第二列写。
- `<模型>`：harness 里 `/model` 一类命令显示的模型标识，原样照抄（如 `claude-fable-5-1`、`gpt-6.1-sol`），不写宣传名。
- `effort=<档位>`：harness 自己的推理档位，照界面或配置里显示的写（如 `xhigh`、`high`、`medium`）；harness 没有这个概念就写 `default`。
- 邮箱：下表那一家 bot 的。GitHub 据此把提交算到那个 bot 账号头上。数字是 bot **账号的用户号**，不是 App ID；`gh api 'users/<bot 名>%5Bbot%5D' --jq .id` 可查。
- 几家一起改的提交写几行。人自己手改的提交也写用过的那家；只有在 GitHub 网页上直接改的提交不用（它的提交者是 GitHub）。不写别的署名行：不放会话链接，不写「Generated with …」。
- Antigravity 席用 `an-antigravity-cli-bot`，不用 `a-gemini-cli-bot`（后者留给 Gemini CLI）。

| 席位 | harness | bot | 邮箱 |
| --- | --- | --- | --- |
| Claude | Claude Code | a-claude-code-bot | `281844019+a-claude-code-bot[bot]@users.noreply.github.com` |
| Codex | Codex | a-chatgpt-codex-bot | `281847692+a-chatgpt-codex-bot[bot]@users.noreply.github.com` |
| Grok | Grok Build | a-grok-build-bot | `302482056+a-grok-build-bot[bot]@users.noreply.github.com` |
| Antigravity | Antigravity | an-antigravity-cli-bot | `295901900+an-antigravity-cli-bot[bot]@users.noreply.github.com` |
| Kimi | Kimi Code | a-kimi-code-bot | `281852327+a-kimi-code-bot[bot]@users.noreply.github.com` |
| Qwen | Qoder | a-qwen-qoder-bot | `281849308+a-qwen-qoder-bot[bot]@users.noreply.github.com` |
| DeepSeek | omp | a-deepseek-code-bot | `281851494+a-deepseek-code-bot[bot]@users.noreply.github.com` |
| GLM | opencode | a-glm-code-bot | `281846436+a-glm-code-bot[bot]@users.noreply.github.com` |
| MiniMax | mcode | a-minimax-code-bot | `281853025+a-minimax-code-bot[bot]@users.noreply.github.com` |
| Muse | Muse Code | a-muse-code-bot | `327610061+a-muse-code-bot[bot]@users.noreply.github.com` |
| Gemini CLI（现在不用） | Gemini CLI | a-gemini-cli-bot | `281848501+a-gemini-cli-bot[bot]@users.noreply.github.com` |

例：`Co-authored-by: Claude Code claude-fable-5-1 effort=xhigh <281844019+a-claude-code-bot[bot]@users.noreply.github.com>`

机械关卡：`PR contract` 检查 PR 里 2026-10-06 18:00（北京时间）之后的每个非合并提交都至少有一行合格的署名、邮箱在上表里；之前的提交不用改（所有者 2026-10-06：「就改未来的好了」）。表改了，`.github/workflows/pr-contract.yml` 里的清单要一起改。

## Repo 地图

- 设计正文与约束：`docs/design/`（约束在 `spec/`；当前基线版本见根 README）
- 文档怎么写：[WRITING-GUIDE.md](./WRITING-GUIDE.md)（文风与结构）；内容与权威纪律在 [docs/design/doc-discipline.md](./docs/design/doc-discipline.md)
- 调研与证据：`docs/research/`
- 中间过程与备忘：`.memo/`（放什么去哪见 `.memo/README.md`）
- 代码：`src/`，Buck2 驱动（`cd src && ./buck2 build root//...`）；Buck2 项目根是仓库根（`.buckconfig`、`.buckroot`、`BUCK` 在根，`src/` 是 `root` cell，文档与许可证以 `repo//...` 进图）；打包与发行在 `src/packaging/`；本地 Agency 参考实现（技能目录）在 `src/agency/`

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
- `git status` 只跟 upstream 比，显示已同步不代表跟上了 `main`。`./run <harness>` 启动前会先 fetch，再把家分支快进到 `origin/main`；它提示没同步（有本地提交、有未提交改动、fetch 失败）时，先处理再干活。没经过 `./run` 启动、或会话已经跑了很久时，自己跑 `git fetch --prune origin && git merge --ff-only origin/main`，不用 `git pull`（它拉的是自己的远端分支）。同步后 AGENTS.md 或 CONSTRAINTS.md 有变化就重读。
- 开 PR 的分支从 `origin/main` 起：`git switch --no-track -c <harness>/<topic> origin/main`，再 `git push -u origin HEAD`。不加 `--no-track`，upstream 会被设成 `origin/main`。只推自己的分支，不推 `main`。

## Repo 地图

- 设计正文与约束：`docs/design/`（约束在 `spec/`；当前基线版本见根 README）
- 文档怎么写：[WRITING-GUIDE.md](./WRITING-GUIDE.md)（文风与结构）；内容与权威纪律在 [docs/design/doc-discipline.md](./docs/design/doc-discipline.md)
- 调研与证据：`docs/research/`
- 中间过程与备忘：`.memo/`（放什么去哪见 `.memo/README.md`）
- 代码：`src/`，Buck2 驱动（`cd src && ./buck2 build root//...`）；Buck2 项目根是仓库根（`.buckconfig`、`.buckroot`、`BUCK` 在根，`src/` 是 `root` cell，文档与许可证以 `repo//...` 进图）；打包与发行在 `src/packaging/`；本地 Agency 参考实现（技能目录）在 `src/agency/`

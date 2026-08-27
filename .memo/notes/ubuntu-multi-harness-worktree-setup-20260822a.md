# Ubuntu 多 harness worktree 环境搭建任务

- 日期：2026-08-22
- 交接者：Codex
- Ubuntu 状态：`PENDING`
- 目标仓库：`hctl2`
- 依据：复用 abacistopia 已运行的“一份 canonical clone + 每 harness 一个 sibling worktree + 分支带机器后缀”模式

## 1. 任务结果

在 Ubuntu 上保留一份 canonical checkout：

```text
~/workspace/hctl2                 branch: main
```

并从同一个 `origin/main` commit 创建以下六个 sibling worktree：

| Harness | Worktree | Branch |
| --- | --- | --- |
| Codex | `~/workspace/hctl2-codex` | `codex/main-ubuntu` |
| Claude Code | `~/workspace/hctl2-claude` | `claude/main-ubuntu` |
| Kimi Code | `~/workspace/hctl2-kimi` | `kimi/main-ubuntu` |
| OpenCode / GLM | `~/workspace/hctl2-glm` | `glm/main-ubuntu` |
| Grok Build | `~/workspace/hctl2-grok` | `grok/main-ubuntu` |
| Antigravity | `~/workspace/hctl2-agy` | `agy/main-ubuntu` |

必须使用 `agy`，不要另建 `gemini`。Mac 和 Ubuntu 目录名相同，但 branch 必须以 `main-mac` / `main-ubuntu` 隔离，不能跨机器共用同一 harness branch。

## 2. 边界

本任务只建立 Git worktree/branch/upstream 环境，不做以下事项：

- 不为六个 harness 分别 clone 仓库；
- 不复制 Mac 的认证、session、token、SSH key 或 harness 配置；
- 不安装、升级或登录六个 harness CLI；
- 不复制 abacistopia 的 hooks、launcher、round protocol 或仓库专属规则；
- 不删除、reset、覆盖或自动接管任何碰撞的目录/branch/worktree；
- 不修改 hctl2 产品设计文件。

搭建完成后，canonical `hctl2` 只承载共享 `.git`、读取和 fast-forward 对齐；日常编辑只发生在各自的 `hctl2-<harness>` 中。

## 3. 全量 preflight

先读完本 memo，再进行任何 mutation。Ubuntu 执行者应从 canonical checkout 运行检查。

### 3.1 主机与 canonical

确认：

- `uname -s` 返回 `Linux`；
- `git rev-parse --show-toplevel` 指向 Ubuntu 的 canonical `hctl2`；
- canonical 当前 branch 是 `main`；
- `git status --porcelain` 为空；
- `origin` 是预期的 `yesme/hctl2` 仓库；
- SSH 访问 origin 正常。

然后：

```bash
git fetch --prune origin
git merge --ff-only origin/main
git status --short --branch
```

fast-forward 或 fetch 失败时停止，不使用默认 `git pull`、merge commit、reset 或 force 来“修好”现场。

### 3.2 一次性钉住基线

在 fetch/FF 后只解析一次基线：

```bash
HCTL2_BASE_SHA="$(git rev-parse refs/remotes/origin/main)"
git cat-file -e "$HCTL2_BASE_SHA^{commit}"
```

六个 branch 都必须从这个相同 SHA 创建。不要在创建循环中反复读取一个可能变化的 `origin/main`。

### 3.3 碰撞检查

确认下面六个 path 全部不存在；既检查普通 path，也检查 dangling symlink：

```text
../hctl2-codex
../hctl2-claude
../hctl2-kimi
../hctl2-glm
../hctl2-grok
../hctl2-agy
```

确认以下 ref 在 fetch 后本地和 origin 都不存在：

```text
codex/main-ubuntu
claude/main-ubuntu
kimi/main-ubuntu
glm/main-ubuntu
grok/main-ubuntu
agy/main-ubuntu
```

同时检查：

```bash
git worktree list --porcelain
git worktree prune --dry-run
git for-each-ref --format='%(refname)' refs/heads refs/remotes/origin
```

规则：

- 任一目标 path/ref 已存在，先整体停止并报告；不要猜它能否复用；
- `prune --dry-run` 若报告 stale registration，停止并报告；不要自动 prune；
- 不得使用 `-B`、`--force`、`reset --hard`、`rm -rf` 或删除 branch 来消除碰撞；
- 只有六个目标全部无碰撞，才开始创建第一个 worktree。

## 4. 创建 worktree

从 canonical 串行执行，不能并行写共享 `.git`：

```bash
git worktree add -b codex/main-ubuntu ../hctl2-codex "$HCTL2_BASE_SHA"
git worktree add -b claude/main-ubuntu ../hctl2-claude "$HCTL2_BASE_SHA"
git worktree add -b kimi/main-ubuntu ../hctl2-kimi "$HCTL2_BASE_SHA"
git worktree add -b glm/main-ubuntu ../hctl2-glm "$HCTL2_BASE_SHA"
git worktree add -b grok/main-ubuntu ../hctl2-grok "$HCTL2_BASE_SHA"
git worktree add -b agy/main-ubuntu ../hctl2-agy "$HCTL2_BASE_SHA"
```

若中途失败：

1. 立即停止后续 mutation；
2. 保留已经创建的 clean worktree 和 branch；
3. 输出 `git worktree list --porcelain` 以及每个已创建目录的 branch、HEAD、status；
4. 把现场分类为 partial/mismatch 并报告；
5. 不自动回滚或清理，因为失败后可能已有 harness 进入目录。

## 5. 本地验证

每个目标都必须满足：

- `git branch --show-current` 精确等于表中 branch；
- `git rev-parse HEAD` 精确等于 `HCTL2_BASE_SHA`；
- `git status --porcelain` 为空；
- 顶层 `.git` 是 worktree pointer file，不是独立 clone 的 `.git/` 目录；
- `git rev-parse --git-common-dir` 最终解析到 canonical `hctl2/.git`；
- canonical 仍在 `main` 且保持 clean。

建议逐个执行显式检查，避免通配符把别的目录混进结果：

```bash
git -C ../hctl2-codex branch --show-current
git -C ../hctl2-claude branch --show-current
git -C ../hctl2-kimi branch --show-current
git -C ../hctl2-glm branch --show-current
git -C ../hctl2-grok branch --show-current
git -C ../hctl2-agy branch --show-current

git worktree list --porcelain
```

仓库可能已有其他合法 worktree，所以验收条件是“canonical + 这六个目标都存在且映射正确”，不是机械断言总数永远等于七。

## 6. 原子发布远端 branch

本地验证全部通过后，从 canonical 一次发布六条 branch，并设置各自 upstream：

```bash
git push --atomic --set-upstream origin +  codex/main-ubuntu +  claude/main-ubuntu +  kimi/main-ubuntu +  glm/main-ubuntu +  grok/main-ubuntu +  agy/main-ubuntu
```

`--atomic` 失败时，远端六条 ref 应保持全不发布；保留本地布局并报告，不改用六次无验证 push，也不 force push。

发布成功后逐一确认 upstream：

| Local branch | Required upstream |
| --- | --- |
| `codex/main-ubuntu` | `origin/codex/main-ubuntu` |
| `claude/main-ubuntu` | `origin/claude/main-ubuntu` |
| `kimi/main-ubuntu` | `origin/kimi/main-ubuntu` |
| `glm/main-ubuntu` | `origin/glm/main-ubuntu` |
| `grok/main-ubuntu` | `origin/grok/main-ubuntu` |
| `agy/main-ubuntu` | `origin/agy/main-ubuntu` |

任何 harness branch 的 upstream 指向 `origin/main` 都是配置错误，必须在开始开发前报告和修正。

## 7. 完成后的使用规则

- canonical `hctl2` 不作为开发目录；只允许读取、fetch 和 `merge --ff-only origin/main`；
- 每个 harness 只在自己的 sibling worktree 编辑、stage、commit 和 push；
- stage 使用显式 pathspec，不用 `git add -A` 或 `git add .`；
- session 开始先 fetch；没有本地 commit 时 FF 到 `origin/main`，有本地 commit 时先审查后 rebase；
- 只 push 自己的 `<harness>/main-ubuntu`，不直接 push `main`；
- Mac 与 Ubuntu 的同名 harness 通过 origin/main 或明确的 WIP commit/cherry-pick 交接，不共用 machine branch；
- 任何 force 操作都不属于本次 setup；绝不 force push `main`；
- merge/PR、co-author trailer、hooks 和 round 协作策略由 hctl2 后续单独制定，本 memo 不擅自把 abacistopia 的仓库专属机制复制过来。

## 8. Ubuntu 完成回报

执行者完成后应记录：

- canonical path、`origin/main` 基线 SHA；
- 六个 `path → branch → upstream → HEAD` 映射；
- 六个 status 均 clean；
- 六个 worktree 的 common dir 都指向 canonical `.git`；
- 原子 push 结果；
- 任何与本 memo 不同的平台差异或碰撞。

如果现场已经完整存在且所有映射都精确匹配，只做验证并报告幂等成功；如果是 partial 或任意 mismatch，停止并等待用户决定，不擅自修复。

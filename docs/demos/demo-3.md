# 演示 3「真实交付」操作手册

> **核验状态总览**：本手册分节标注实跑状态。截至当前 `main`（`631e47c`），**第一节（构建、离线安装、启动控制面）、第二节（A 链全部 10 步，含从离线安装包拉起真 Claude Code 会话完成写入与封存、自动发布、合入预览、完成预览及十秒回答）、第三节（B 链全部 8 步，含受保护 GitHub `main` 上拉起真 Codex 会话完成写入与封存、自动发布 GitHub PR、`hctl2-tool wait commit-ci` 等待必需检查 `canary`、合入预览、完成预览、十秒回答及远端遗留清理）、第四节（C 链全部 5 步，含真 Claude Code 会话写入与发布、合入、完成三次人工预览）、第五节（重启恢复、凭据隔离与三条逃生口全部小节）、第六节（清理环境）**均已在 Ubuntu（`Linux 7.0.0-34-generic x86_64`）上从零实跑通过（原始终端输出日志见仓内 `.memo/log/2026-10-19-demo3-manual-run.md` 与 `.memo/log/2026-10-20-演示3验收-ubuntu.md`）。

本文记录演示 3「真实交付」从零构建、离线安装到端到端走查的完整命令序列。流程覆盖：本地 Gitea 仓库注册与 Claude Code 真会话写入交付链（A 链，人类仅在合入与完成两处预览确认）、受保护 GitHub `main` 与 Codex 真会话交付链（B 链）、开启「发布评审须人显式确认」开关的三次预览链（C 链），以及重启一致性、凭据隔离与逃生口的核对方法。

所有命令均显式指定独立状态根 `--root "$ROOT"` 与独立 Agency 状态根 `--agency-root "$ROOT/agency"`，不触碰用户默认状态目录 `~/.local/state/hctl2` 与 `~/.hctl2/agency`。

---

## 一、环境构建与离线安装

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

### 1. 构建完整发布包

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，Buck2 Build ID `b74d800c-1f90-4034-9b92-c41c04f838b2`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

在代码仓库根目录下进入 `src/`，构建当前平台的完整离线运行包：

```bash
export REPO_ROOT="$(pwd)"
cd "$REPO_ROOT/src"
./buck2 build --config hctl2.zstd_preset=fast root//packaging/release:complete
cd "$REPO_ROOT"
```

构建完成后，运行安装包位于 `buck-out/v2/art/root/<hash>/packaging/release/__complete__/out/release/hctl2-0.0.0-<target>.tar.zst`。

### 2. 离线安装、前置条件检查与工作区变量设置

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

新建独立演示工作目录（建议使用磁盘上的短路径如 `/var/tmp/d3`，既避免 Unix domain socket 路径超出系统 `SUN_LEN` 限制，也避免在内存盘 `/tmp` 配额较小的机器上解压 635 MB 载荷时触发磁盘配额不足），解压并运行离线安装器 `./install.sh`：

```bash
export WS="${WS:-/var/tmp/d3}"
mkdir -p "$WS/pkg" "$WS/prefix" "$WS/state" "$WS/work"

ARCHIVE="$(find "$REPO_ROOT/buck-out" -path '*packaging/release/__complete__*' -name 'hctl2-0.0.0-*.tar.zst' ! -name '*-sources.tar.zst' | head -n 1)"
[ -f "$ARCHIVE" ] || { echo "未找到完整运行包: $ARCHIVE"; return 1 2>/dev/null || exit 1; }

tar --zstd -xf "$ARCHIVE" -C "$WS/pkg"
PKG_DIR="$(find "$WS/pkg" -mindepth 1 -maxdepth 1 -type d -name 'hctl2-0.0.0-*' | head -n 1)"
"$PKG_DIR/install.sh" --prefix "$WS/prefix"

export PATH="$WS/prefix/bin:$PATH"
export INSTALL_ROOT="$(find "$WS/prefix/lib/hctl2" -mindepth 1 -maxdepth 1 -type d -name 'hctl2-0.0.0-*' | head -n 1)"
export CLI="$WS/prefix/bin/hctl2"
export ROOT="$WS/state"

# 校验安装产物（通过 install.sh 安装后，hctl2 与 agency 会由自身二进制路径自动解析共享安装根）
[ -x "$CLI" ] || { echo "CLI 未就绪: $CLI"; return 1 2>/dev/null || exit 1; }
[ -x "$WS/prefix/bin/agency" ] || { echo "agency 未就绪"; return 1 2>/dev/null || exit 1; }
[ -x "$WS/prefix/bin/hctl2-tool" ] || { echo "hctl2-tool 未就绪"; return 1 2>/dev/null || exit 1; }
[ "$(jq -r .hasCompletedOnboarding ~/.claude.json 2>/dev/null)" = "true" ] || { echo "请先在终端运行一次 claude 完成登录与初始主题选择引导"; return 1 2>/dev/null || exit 1; }
```

> [!IMPORTANT]
> - **状态隔离**：全程所有控制面命令均须携带 `--root "$ROOT"`，配对命令携带 `--agency-root "$ROOT/agency"`，确保状态与托管服务全部落在独立目录 `$WS/state` 下，不污染 `~/.local/state/hctl2` 与 `~/.hctl2/agency`。
> - **版本门槛与首次引导前置（以代码 `src/agency/src/harness/mod.rs` 为准）**：执行 A 链与 C 链前请确认本机已安装并登录 `claude`（代码准入门槛 **Claude Code >= 2.1.287**，`CLAUDE_MINIMUM`；Ubuntu 实测通过版本为 `2.1.302` / `2.1.303`），且已由人在终端运行过一次 `claude` 完成初始登录与主题选择引导（`~/.claude.json` 中 `hasCompletedOnboarding` 为 `true`；Agency 不替人按键，未完成引导会停在 `Choose the text style` 界面导致 `STANDBY_START_TIMEOUT`）。此外，Linux 下 `Herdr` 启动时通过 Landlock `PathFd` 按文件 inode 放行 `~/.claude.json`（`src/agency/linux_confine.rs`）；若 A 链与 C 链之间宿主机上有其他 `claude` 进程原子重写过 `~/.claude.json`（改变 inode），旧 `Herdr` 进程将因无权读取新 inode 而使受管 `claude` 回退到首次引导界面——第四.2 节在启动 C 链派工前会重启一次本实例的 `agency` 以刷新 `Herdr` 的 Landlock 句柄。执行 B 链前请确认本机已安装并登录 `codex`（代码准入门槛 **Codex CLI >= 0.153.4**，`CODEX_MINIMUM`）且 `gh auth status` 已登录具备 `yesme/hctl2-canary` 权限的账号。
> - **已知平台限制（macOS 下 A 链不可用）**：在 macOS 上，Claude Code 仅将登录凭据存放在系统钥匙串（`Claude Code-credentials`），而 Agency 的 Seatbelt 凭据隔离边界显式封禁了 `~/Library/Keychains`，导致受管执行环境内 Claude Code 报错 `Not logged in`（`HARNESS_TURN_ERROR`）。因此 **A 链（真 Claude Code 会话）目前只能在 Linux 上运行**。
> - **Claude Code 工作区信任与启动自检**：本地 Agency 在 `/tmp/hctl2-exec-*` 下为派工建执行目录（写入型派工开在 git worktree 里）；让不让 Claude 在执行目录里工作由 Agency 所有者决定，Agency 起的每个 Claude 会话都带 `CLAUDE_CODE_SANDBOXED=1` 启动（不修改用户的 `~/.claude.json`），用户无需手工干预。Agency 启动时会在 `/tmp` 下的新目录里开一次 Claude 会话自检这一条（绝不替人按键应答）；若将来换成了不认该变量而仍弹出信任框的版本，`<agency-root>/serve.err` 会写明 `CLAUDE_WORKSPACE_UNTRUSTED`，且 `agency catalog` 不会公布 `claude-code` 工种（详见 `docs/usage.md`「本地 Agency 与 Claude Code 的工作区信任」）。

### 3. 启动控制面并核对初始状态

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

确保随包服务端口（Tuwunel `6167`、Cinny `6168`、Gitea `3001`）空闲后，以私有文件密钥后端启动控制面：

```bash
$CLI --root "$ROOT" start --secret-backend user-file

for i in $(seq 1 30); do
  [ "$($CLI --json --root "$ROOT" status 2>/dev/null | jq -r '.services.hosted[] | select(.name=="tuwunel") | .ready')" = "true" ] && break
  sleep 1
done

$CLI --root "$ROOT" status
$CLI --json --root "$ROOT" status
```

**核对字段**：
- `$CLI --root "$ROOT" status` 当前在不加 `--json` 时同样直接输出缩进的 JSON 对象（与 `--json` 结构一致，约 45 行，包含控制面 socket 路径、密钥存储策略与三项随包托管服务的 `consumed` / `ready` / `pid` / `endpoint` 等字段）。
- 输出中 `ready` 为 `true`，`policy.credential_storage` 为 `"user-file"`，`services.hosted` 中 `tuwunel` 的 `ready: true`、`gitea` 的 `consumed: false`（首次注册本地仓库时按需拉起）。

---

## 二、A 链：本地 Gitea + 真 Claude Code 写入交付（两次人工预览）

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

### 1. 初始化带 `.gitignore` 的本地试验仓库

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

在 `$WS/write-site` 下创建一个包含真实非文档代码缺陷的 Python 仓库，并配置 `.gitignore` 忽略测试运行生成的字节码缓存（避免 `__pycache__` 混入封存版本）：

```bash
mkdir -p "$WS/write-site"
cd "$WS/write-site"
git init -q --initial-branch=main
git config user.name "Demo3 Walkthrough"
git config user.email "demo3@example.test"

cat << 'EOF' > .gitignore
__pycache__/
*.py[cod]
EOF

cat << 'EOF' > calculator.py
def double(value):
    """Return twice the argument."""
    return value  # bug: returns the argument unchanged
EOF

cat << 'EOF' > test_calculator.py
import unittest
from calculator import double


class DoubleTests(unittest.TestCase):
    def test_positive(self):
        self.assertEqual(double(3), 6)

    def test_negative(self):
        self.assertEqual(double(-4), -8)

    def test_zero(self):
        self.assertEqual(double(0), 0)


if __name__ == "__main__":
    unittest.main()
EOF

git add .gitignore calculator.py test_calculator.py
git commit -q -m "fixture baseline with defective double()"
export BASELINE_SHA="$(git rev-parse HEAD)"
cd "$WS"
```

**核对字段**：
- `BASELINE_SHA` 为 40 位十六进制提交摘要。
- 在基线上运行 `/usr/bin/python3 -B -m unittest -v test_calculator` 应有 2 条用例失败（`test_positive`、`test_negative`）。

### 2. 注册并激活本地 Repo（两步确认）

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

编写本地仓库注册输入 `$WS/work/a-repo.json`（不指定外部远端，缺省绑定随包本地 Gitea 平台与 `gitea_issues` 任务源）：

```bash
cat << EOF > "$WS/work/a-repo.json"
{
  "name": "demo3-walkthrough",
  "origin": "local",
  "platform": "local",
  "platform_path": "demo3-walkthrough",
  "default_source": "gitea_issues",
  "local": {
    "machine": "control",
    "path": "$WS/write-site",
    "in_place": false,
    "publish_governance": false,
    "extra_refs": []
  }
}
EOF

PREV_REG="$($CLI --json --root "$ROOT" repo register --input "$WS/work/a-repo.json" --key a-register)"
TOK_REG="$(echo "$PREV_REG" | jq -r .preview_token)"
SUB_REG="$($CLI --json --root "$ROOT" repo register --input "$WS/work/a-repo.json" --key a-register --preview-token "$TOK_REG")"
export REPO_ID="$(echo "$SUB_REG" | jq -r .registration.repo_id)"
export STABLE_ID="$(echo "$SUB_REG" | jq -r .registration.observed.stable_id)"
export REG_VERSION="$(echo "$SUB_REG" | jq -r .registration.version)"
```

> [!NOTE]
> 首次使用本地 Gitea 时，控制面会自动消费并拉起随包 Gitea 服务。若 Gitea 尚在初始化导致首次提交返回 `PLATFORM_NOT_READY`（`recovery_action: "retry_registration"`），等待数秒后先由 `$CLI --json --root "$ROOT" repo list | jq -r '.items[0].repo_id'` 取出待恢复的 `REPO_ID`，再按恢复路径执行 `repo register --resume "$REPO_ID" --key a-register-resume` 预览并提交即可。

**核对字段**：
- `registration.lifecycle`: `"pending"`。
- `registration.version`: `3`。
- `registration.observed.instance`: `"http://127.0.0.1:3001"`，`stable_id`: `"1"`，`clone_url` 已建立。

确认平台身份以激活 Repo（两步确认）：

```bash
PREV_CONF="$($CLI --json --root "$ROOT" repo register --confirm "$REPO_ID" --version "$REG_VERSION" --platform-repo-id "$STABLE_ID" --key a-confirm)"
TOK_CONF="$(echo "$PREV_CONF" | jq -r .preview_token)"
$CLI --json --root "$ROOT" repo register --confirm "$REPO_ID" --version "$REG_VERSION" --platform-repo-id "$STABLE_ID" --key a-confirm --preview-token "$TOK_CONF"
```

**核对字段**：
- `lifecycle`: `"active"`。
- `version`: `4`。

### 3. 配对 Agency、接受 `claude-code` 工种并创建写入型 Profile

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

将控制面与随包原生 Agency 配对（显式指定 `--agency-root "$ROOT/agency"`），并从名册中提取 `claude-code` 工种引用并接受：

```bash
PAIR_OUT="$($CLI --json --root "$ROOT" agency pair --binding-id native --agency-root "$ROOT/agency" --key a-pair)"
echo "$PAIR_OUT" | jq '{paired: .paired, professions: [.binding.data.value.catalog.professions[].reference]}'

CATALOG_OUT="$($CLI --json --root "$ROOT" agency catalog native)"
echo "$CATALOG_OUT" | jq -c '.professions[] | select(.reference.id=="claude-code")' > "$WS/work/claude-profession.json"
jq -c '.reference' "$WS/work/claude-profession.json" > "$WS/work/claude-ref.json"

ACCEPT_CLAUDE="$($CLI --json --root "$ROOT" agency accept --binding-id native --reference "$WS/work/claude-ref.json" --key a-accept-claude)"
export CLAUDE_PROF_KEY="$(echo "$ACCEPT_CLAUDE" | jq -c .profession.key)"
export CLAUDE_PROF_VER="$(echo "$ACCEPT_CLAUDE" | jq -r .profession.version)"
export CLAUDE_PROF_DIGEST="$(jq -r '.digest' "$WS/work/claude-ref.json")"
```

**核对字段**：
- `paired`: `true`。
- `professions` 中包含 `id: "claude-code"`（`terms` 写明在选入绑定的原生 Claude Herdr 会话中执行、租约 ChangeSet 工作树与工具封存交回）。
- `ACCEPT_CLAUDE` 返回已接受的 `profession.key` 与 `profession.version: 1`。

根据该工种申报的 `harness` 与 `model` 创建写入型 Worker Profile（两步确认）：

```bash
CLAUDE_HARNESS="$(jq -c '.harness' "$WS/work/claude-profession.json")"
CLAUDE_MODEL="$(jq -c '.model' "$WS/work/claude-profession.json")"

cat << EOF > "$WS/work/a-profile.json"
{
  "id": "demo3-claude-write",
  "profile": {
    "harness": $CLAUDE_HARNESS,
    "model": $CLAUDE_MODEL,
    "mode": "write",
    "permissions": [
      "context.read",
      "git.read",
      "git.write"
    ],
    "environment": [],
    "required_capabilities": {
      "event_cursor": false,
      "exact_attach": false,
      "input": false,
      "input_provenance": false,
      "isolation_effects": [],
      "managed_single_writer": false,
      "secure_input": false,
      "stop": true,
      "tool_execution_unmediated": false
    },
    "max_context_bytes": 262144
  }
}
EOF

PREV_PROF="$($CLI --json --root "$ROOT" profile create --input "$WS/work/a-profile.json" --key a-profile)"
TOK_PROF="$(echo "$PREV_PROF" | jq -r .preview_token)"
SUB_PROF="$($CLI --json --root "$ROOT" profile create --input "$WS/work/a-profile.json" --key a-profile --preview-token "$TOK_PROF")"
export WRITE_PROFILE_REV="$(echo "$SUB_PROF" | jq -c .revision)"
```

**核对字段**：
- `WRITE_PROFILE_REV` 包含 `worker_profile_revision` 的 `key` 与 `version.state: 1`。

### 4. 创建 Project 并将写入参与者选入主 Room

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

创建缺省发布策略（`publish_review_requires_confirmation: false`）的 Project（两步确认）：

```bash
cat << EOF > "$WS/work/a-project.json"
{
  "repo_id": "$REPO_ID",
  "definition": {
    "name": "demo3 walkthrough",
    "goal": "deliver tested code through a local review",
    "scope": "demo3-walkthrough",
    "roles": [],
    "role_members": {},
    "defaults": {},
    "settings": {
      "selection_policy": {},
      "publish_review_requires_confirmation": false
    }
  }
}
EOF

PREV_PROJ="$($CLI --json --root "$ROOT" project create --input "$WS/work/a-project.json" --key a-project)"
TOK_PROJ="$(echo "$PREV_PROJ" | jq -r .preview_token)"
SUB_PROJ="$($CLI --json --root "$ROOT" project create --input "$WS/work/a-project.json" --key a-project --preview-token "$TOK_PROJ")"
export PROJECT_ID="$(echo "$SUB_PROJ" | jq -r .project_id)"
export MAIN_ROOM_ID="$(echo "$SUB_PROJ" | jq -r .main_room_id)"
```

先从 `project show` 读取当前 Project 版本，再将 `claude-code` 写入参与者选入主 Room 名册（两步确认）：

```bash
PROJ_VER="$($CLI --json --root "$ROOT" project show "$PROJECT_ID" | jq -r .project.version)"

cat << EOF > "$WS/work/a-select.json"
{
  "project_id": "$PROJECT_ID",
  "project_version": $PROJ_VER,
  "room_id": "$MAIN_ROOM_ID",
  "topic_command_key": null,
  "roster_version": null,
  "selections": [
    {
      "room_id": "$MAIN_ROOM_ID",
      "selected_item": {
        "key": $CLAUDE_PROF_KEY,
        "version": { "state": $CLAUDE_PROF_VER }
      },
      "profession": {
        "key": $CLAUDE_PROF_KEY,
        "version": { "state": $CLAUDE_PROF_VER }
      },
      "profession_digest": "$CLAUDE_PROF_DIGEST",
      "agency": {
        "key": {
          "id": "native",
          "kind": "agency_binding",
          "scope": { "kind": "control" }
        },
        "version": { "state": 1 }
      },
      "required_skills": [],
      "optional_skills": [],
      "worker_profiles": [
        $WRITE_PROFILE_REV
      ],
      "responsibility": "research",
      "permission": {
        "allow": [
          "context.read",
          "git.read",
          "git.write"
        ]
      },
      "budget": {
        "max_bytes": 262144
      },
      "display_name": "Claude Code writer",
      "persona_tags": []
    }
  ]
}
EOF

PREV_SEL="$($CLI --json --root "$ROOT" project select --input "$WS/work/a-select.json" --key a-select)"
TOK_SEL="$(echo "$PREV_SEL" | jq -r .preview_token)"
$CLI --json --root "$ROOT" project select --input "$WS/work/a-select.json" --key a-select --preview-token "$TOK_SEL"
```

**核对字段**：
- `roster_version`: `1`。

### 5. 连接 Gitea 任务源、创建 Task 并采纳契约

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

连接并挂接随包 Gitea 的 issues 任务源（各两步确认，每次写 Project 作用域命令前从 `project show` 读取当前 `project_version`）：

```bash
cat << EOF > "$WS/work/a-source.json"
{
  "repo_id": "$REPO_ID",
  "candidate_id": "gitea_issues",
  "consent": true,
  "make_default": false
}
EOF

PREV_SRC="$($CLI --json --root "$ROOT" task connect --input "$WS/work/a-source.json" --key a-connect)"
TOK_SRC="$(echo "$PREV_SRC" | jq -r .preview_token)"
SUB_SRC="$($CLI --json --root "$ROOT" task connect --input "$WS/work/a-source.json" --key a-connect --preview-token "$TOK_SRC")"
export SOURCE_ID="$(echo "$SUB_SRC" | jq -r .source_id)"

PROJ_VER="$($CLI --json --root "$ROOT" project show "$PROJECT_ID" | jq -r .project.version)"

cat << EOF > "$WS/work/a-attach.json"
{
  "project_id": "$PROJECT_ID",
  "project_version": $PROJ_VER,
  "source_id": "$SOURCE_ID",
  "approved_scope": "$STABLE_ID",
  "consent": true
}
EOF

PREV_ATT="$($CLI --json --root "$ROOT" task attach --input "$WS/work/a-attach.json" --key a-attach)"
TOK_ATT="$(echo "$PREV_ATT" | jq -r .preview_token)"
$CLI --json --root "$ROOT" task attach --input "$WS/work/a-attach.json" --key a-attach --preview-token "$TOK_ATT"
```

在随包 Gitea issues 上创建 Task（两步确认）：

```bash
PROJ_VER="$($CLI --json --root "$ROOT" project show "$PROJECT_ID" | jq -r .project.version)"

cat << EOF > "$WS/work/a-task.json"
{
  "project_id": "$PROJECT_ID",
  "project_version": $PROJ_VER,
  "source_id": "$SOURCE_ID",
  "title": "Fix double() in calculator.py",
  "body": "Fix the bug in calculator.py: double(value) must return twice its argument. Edit the non-documentation Python code, run /usr/bin/python3 -B -m unittest -v test_calculator, and report the command and its output. Leave all changes uncommitted for Agency sealing. Do not push, publish, obtain credentials, or edit harness global settings."
}
EOF

PREV_TASK="$($CLI --json --root "$ROOT" task create --input "$WS/work/a-task.json" --key a-task-create)"
TOK_TASK="$(echo "$PREV_TASK" | jq -r .preview_token)"
SUB_TASK="$($CLI --json --root "$ROOT" task create --input "$WS/work/a-task.json" --key a-task-create --preview-token "$TOK_TASK")"
export TASK_ID="$(echo "$SUB_TASK" | jq -r .task_id)"
export TASK_VER="$(echo "$SUB_TASK" | jq -r .task.version)"
```

**核对字段**：
- `effect_state`: `"confirmed"`。
- `task.data.entity.provider`: `"gitea_issues@http://127.0.0.1:3001"`，`immutable_external_entity_id`: `"1"`（卡建在随包 Gitea issues 上）。

为该 Task 采纳契约（含 1 项要求 `Integration Receipt` 的机械验收项与 1 项人工验收项；`proposal_digest` 为 `adoption.contract` 经 `jq -cS .` 规范化后的无换行 JSON SHA-256 摘要，当读者改动契约内容时会自动重算，两步确认）：

```bash
PROJ_VER="$($CLI --json --root "$ROOT" project show "$PROJECT_ID" | jq -r .project.version)"

CONTRACT_JSON="$(cat << 'EOF'
{
  "scope": "demo3-walkthrough",
  "expected_outcome": "double() fixed and merged through review",
  "acceptance": [
    {
      "text": "the Task change is integrated",
      "grade": "mechanical",
      "evidence": {
        "accept": "integration_receipt",
        "min_channel": "unmediated"
      }
    },
    {
      "text": "a human verified the fix and tests",
      "grade": "human"
    }
  ],
  "roles": [],
  "capabilities": []
}
EOF
)"
PROPOSAL_DIGEST="$(printf '%s' "$CONTRACT_JSON" | jq -cS . | tr -d '\n' | sha256sum | awk '{print $1}')"

cat << EOF > "$WS/work/a-adopt.json"
{
  "project_id": "$PROJECT_ID",
  "project_version": $PROJ_VER,
  "task_id": "$TASK_ID",
  "version": $TASK_VER,
  "adoption": {
    "contract": $CONTRACT_JSON,
    "origin": {
      "kind": "local",
      "reference": {
        "key": {
          "scope": { "kind": "project", "id": "$PROJECT_ID" },
          "kind": "project",
          "id": "$PROJECT_ID"
        },
        "version": { "state": $PROJ_VER }
      },
      "proposal_digest": "$PROPOSAL_DIGEST"
    }
  }
}
EOF

PREV_ADOPT="$($CLI --json --root "$ROOT" task adopt --input "$WS/work/a-adopt.json" --key a-task-adopt)"
TOK_ADOPT="$(echo "$PREV_ADOPT" | jq -r .preview_token)"
SUB_ADOPT="$($CLI --json --root "$ROOT" task adopt --input "$WS/work/a-adopt.json" --key a-task-adopt --preview-token "$TOK_ADOPT")"
export TASK_VER_ADOPTED="$(echo "$SUB_ADOPT" | jq -r .task.version)"
```

**核对字段**：
- `task.data.revision.number`: `1`。
- `task.version`: `4`。

### 6. 从 Room 发起写入型调用（真 Claude Code 会话改代码并封存）

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

编写写入型调用输入 `$WS/work/a-invoke.json` 并预览：

```bash
DEADLINE_MS="$(( ($(date +%s) + 900) * 1000 ))"

cat << EOF > "$WS/work/a-invoke.json"
{
  "project_id": "$PROJECT_ID",
  "room_id": "$MAIN_ROOM_ID",
  "task_id": "$TASK_ID",
  "target": "research",
  "profile": $WRITE_PROFILE_REV,
  "request": "Work on the adopted Task: fix double() in calculator.py and run the tests as the Task body says.",
  "budget": 262144,
  "deadline_ms": $DEADLINE_MS,
  "retry_of": null,
  "write": {
    "change_set_id": null,
    "baseline_commit": "$BASELINE_SHA",
    "target_branch": "main",
    "allow_update": true
  }
}
EOF

# 查看人读预览输出（对象、原因与后果三段）
$CLI --root "$ROOT" invocation preview --input "$WS/work/a-invoke.json" --key a-invoke

# 获取机器 JSON 预览与 preview_token
PREV_INV="$($CLI --json --root "$ROOT" invocation preview --input "$WS/work/a-invoke.json" --key a-invoke)"
TOK_INV="$(echo "$PREV_INV" | jq -r .preview_token)"
export CHANGESET_ID="$(echo "$PREV_INV" | jq -r .effect_summary.preview.write.lease.pending.change_set_id)"
```

**核对字段**：
- `effect_summary.preview.write.authorization`: `"publish_for_review_not_integration"`（发布授权包含在派工预览中，写明授权的是发布去评审、不是合入）。
- `effect_summary.preview.write.lease.pending`: 包含分配的 `change_set_id`（`cs-...`）、`baseline_commit` 与状态为 `"pending"` 的 Write Lease。
- `effect_summary.preview.write.publication_target`: 冻结的评审发布策略（`target_branch: "main"`，`branch_rule: "hctl2/{change_set}"`，`requires_human_confirmation: false`）。

确认启动写入型调用，并轮询等待真 Claude Code 会话完成修改与 Agency 封存交回（轮询上限与 15 分钟 `DEADLINE_MS` 对齐，并在循环后断言调用进入 `completed`）：

```bash
SUB_INV="$($CLI --json --root "$ROOT" invocation start --input "$WS/work/a-invoke.json" --key a-invoke --preview-token "$TOK_INV")"
export INVOCATION_ID="$(echo "$SUB_INV" | jq -r .invocation_id)"

for i in $(seq 1 450); do
  INV_SHOW="$($CLI --json --root "$ROOT" invocation show "$PROJECT_ID" "$INVOCATION_ID")"
  STATE="$(echo "$INV_SHOW" | jq -r .state)"
  case "$STATE" in
    completed)
      echo "Invocation completed: $(echo "$INV_SHOW" | jq -r .reason)"
      break
      ;;
    failed|cancelled|lost)
      echo "Invocation ended in $STATE: $(echo "$INV_SHOW" | jq -r .reason)"
      break
      ;;
  esac
  sleep 2
done
[ "$STATE" = "completed" ] || { echo "调用未完成: state=$STATE"; return 1 2>/dev/null || exit 1; }
```

**核对字段**：
- `state`: `"completed"`。
- `reason`: `"Git version admitted; not Task acceptance"`。

### 7. 核对封存版本、自动发布与 Room 安静投影

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

查看准入的 ChangeSet Revision 与精确代码 diff：

```bash
CS_SHOW="$($CLI --json --root "$ROOT" changeset show "$REPO_ID" "$CHANGESET_ID")"
export REVISION_ID="$(echo "$CS_SHOW" | jq -r '.revisions[0].change_set_revision_id')"
echo "$CS_SHOW" | jq '.revisions[0]'

$CLI --json --root "$ROOT" changeset diff "$REPO_ID" "$CHANGESET_ID" "$REVISION_ID" | jq -r .diff
```

**核对字段**：
- `revisions[0]`: 包含五个身份字段（`change_set_id`、`change_set_revision_id`、`base_commit_sha`、`result_tree_sha`、`review_subject_digest`）以及指向本次调用的 `producer_ref.invocation_id`。
- `diff`: 精确显示 `calculator.py` 中 `return value` 改为 `return value * 2`（不含 `__pycache__` 二进制文件）。

等待后台自动发布完成，并查看 `review list` 与 `review show`：

```bash
for i in $(seq 1 30); do
  REV_LIST="$($CLI --json --root "$ROOT" review list "$REPO_ID")"
  PUB_STATE="$(echo "$REV_LIST" | jq -r --arg cs "$CHANGESET_ID" '.items[] | select(.change_set_id==$cs) | .state')"
  [ "$PUB_STATE" = "published" ] && break
  sleep 1
done

export PUBLISH_INTENT_ID="$(echo "$REV_LIST" | jq -r --arg cs "$CHANGESET_ID" '.items[] | select(.change_set_id==$cs) | .intent_id')"
$CLI --json --root "$ROOT" review show "$REPO_ID" "$PUBLISH_INTENT_ID" | jq '{state: .intent.state, stages: .stages, mapping: .mappings[0]}'
```

**核对字段**：
- `state`: `"published"`。
- `stages.push.confirmed`: `true`，`stages.review_request.confirmed`: `true`（推送与建评审请求两段各自确认）。
- `mapping`: 记录 `platform_commit_sha` 与 Gitea 上的 `review_request.index`。

查看主 Room 时间线，核对「正常成功保持安静」：

```bash
$CLI --json --root "$ROOT" room timeline "$PROJECT_ID" "$MAIN_ROOM_ID" | jq '[.events[] | select(.type=="m.room.message") | .content.body]'
```

**核对字段**：
- 全程仅有唯一一条消息投影：`Write admitted as version csr-... of ChangeSet cs-... (baseline ..., result tree ...). Publishing for review is queued: push to hctl2/cs-... and open or update the review request, each read back.`，无内部 JSON 泄漏或多余刷屏。

### 8. 第一次人工预览：合入（Integration Receipt）

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

编写合入输入 `$WS/work/a-integration.json`，执行第一次人工预览与提交确认：

```bash
cat << EOF > "$WS/work/a-integration.json"
{
  "repo_id": "$REPO_ID",
  "change_set_revision_id": "$REVISION_ID",
  "target_kind": "platform",
  "target_ref": "refs/heads/main",
  "form": "accept_advance",
  "strategy": "fast_forward"
}
EOF

# 查看人读合入预览
$CLI --root "$ROOT" integration preview --input "$WS/work/a-integration.json" --key a-merge

PREV_INT="$($CLI --json --root "$ROOT" integration preview --input "$WS/work/a-integration.json" --key a-merge)"
echo "$PREV_INT" | jq -c '.effect_summary.protection'
TOK_INT="$(echo "$PREV_INT" | jq -r .preview_token)"
SUB_INT="$($CLI --json --root "$ROOT" integration submit --input "$WS/work/a-integration.json" --key a-merge --preview-token "$TOK_INT")"
export INTEGRATION_INTENT_ID="$(echo "$SUB_INT" | jq -r .intent_id)"

for i in $(seq 1 30); do
  INT_SHOW="$($CLI --json --root "$ROOT" integration show "$REPO_ID" "$INTEGRATION_INTENT_ID")"
  [ "$(echo "$INT_SHOW" | jq -r .intent.state)" = "succeeded" ] && break
  sleep 1
done

export RECEIPT_ID="$(echo "$INT_SHOW" | jq -r .receipt.receipt_id)"
export MERGED_COMMIT="$(echo "$INT_SHOW" | jq -r .receipt.target_head_after)"
echo "$INT_SHOW" | jq '{state: .intent.state, receipt_id: .receipt.receipt_id, evidence_level: .receipt.evidence_level, git_readback_level: .receipt.readback.git.evidence_level, target_head_after: .receipt.target_head_after}'
```

**核对字段**：
- 人读预览的 `Object` 段列出 `ChangeSet Revision`、`repo`、`target ref`、`authorization form: accept_advance`（本地 Gitea 不提供原子预期目标头合并，只能选接受目标前移）、`strategy` 与 `target head at preview`，末尾 `Confirm with` 给出可直接按原词执行的 `integration submit --input ... --key a-merge --preview-token ...` 命令；冻结的目标保护快照在 `--json` 预览的 `effect_summary.protection` 中查看。
- 提交回读后：`intent.state: "succeeded"`，签出唯一一张 `Integration Receipt`（`receipt_id: "receipt-..."`，`evidence_level: "hctl2-tool"`，`git_readback_level: "unmediated"`）。

### 9. 第二次人工预览：完成 Task（Task Completion Receipt）

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

编写完成输入 `$WS/work/a-complete.json`（引用刚才签出的 `$RECEIPT_ID` 作为第 0 项机械验收项的证据），执行第二次人工预览与确认：

```bash
TASK_CUR_VER="$($CLI --json --root "$ROOT" task show "$PROJECT_ID" "$TASK_ID" | jq -r .version)"

cat << EOF > "$WS/work/a-complete.json"
{
  "project_id": "$PROJECT_ID",
  "task_id": "$TASK_ID",
  "version": $TASK_CUR_VER,
  "lifecycle_version": 1,
  "revision_number": 1,
  "acceptance": [
    {
      "item": 0,
      "channel": "unmediated",
      "generation": 1,
      "judge": { "kind": "hctl2_tool" },
      "producer": "hctl2-tool",
      "references": [
        {
          "key": {
            "id": "$RECEIPT_ID",
            "kind": "integration_receipt",
            "scope": { "id": "$REPO_ID", "kind": "repo" }
          },
          "version": { "state": 1 }
        }
      ]
    },
    {
      "item": 1,
      "channel": "narrated",
      "generation": null,
      "judge": { "kind": "human", "actor": "local-owner:$(id -u)" },
      "producer": null,
      "references": []
    }
  ]
}
EOF

# 查看人读完成预览
$CLI --root "$ROOT" task complete --input "$WS/work/a-complete.json" --key a-complete

PREV_COMP="$($CLI --json --root "$ROOT" task complete --input "$WS/work/a-complete.json" --key a-complete)"
TOK_COMP="$(echo "$PREV_COMP" | jq -r .preview_token)"
SUB_COMP="$($CLI --json --root "$ROOT" task complete --input "$WS/work/a-complete.json" --key a-complete --preview-token "$TOK_COMP")"
echo "$SUB_COMP" | jq '{lifecycle: .lifecycle, lifecycle_version: .lifecycle_version, receipt_id: .receipt_id, items: .items}'
```

**核对字段**：
- `lifecycle`: `"completed"`，`lifecycle_version`: `2`。
- `receipt_id`: `"<TASK_ID>:2"`（Task Completion Receipt 与状态更新在同一事务写入）。
- `items[0]`: `grade: "mechanical"`，`outcome: "passed"`，`validation_level: "unmediated"`，引用 `$RECEIPT_ID`。

### 10. 核对平台回读状态与 `task show` 十秒回答

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

通过产品自身的 `review show` 与 `integration show` 核对已回读确认的平台事实与无中介 Git 回读（不直接读取 `$ROOT/secrets` 下的控制面内部凭据文件）：

```bash
$CLI --json --root "$ROOT" review show "$REPO_ID" "$PUBLISH_INTENT_ID" \
  | jq '{state: .intent.state, push_confirmed: .stages.push.confirmed, pr_confirmed: .stages.review_request.confirmed, platform_commit_sha: .mappings[0].platform_commit_sha, review_request: .mappings[0].review_request}'

$CLI --json --root "$ROOT" integration show "$REPO_ID" "$INTEGRATION_INTENT_ID" \
  | jq '{state: .intent.state, receipt_id: .receipt.receipt_id, evidence_level: .receipt.evidence_level, integrated_commit: .receipt.integrated_commit, target_head_after: .receipt.target_head_after, git_readback: .receipt.readback.git, platform_readback: .receipt.readback.platform}'
```

> [!TIP]
> 若人类操作者希望使用自己的账号通过浏览器或 `tea` 命令行直接查看随包 Gitea 上的仓库与 PR，可通过 `$CLI --root "$ROOT" repo grant` 为人类自己的 Gitea 账号授予仓库访问权限，再用自己的 `tea login` 登录查看，无需触碰 `$ROOT/secrets` 中的控制面凭据。

**核对字段**：
- `review show` 中 `push_confirmed: true`、`pr_confirmed: true`，`review_request` 记录了 Gitea 上的 PR 编号与链接。
- `integration show` 中 `evidence_level`（`receipt.evidence_level`）为 `"hctl2-tool"`，`git_readback.evidence_level` 为 `"unmediated"`，且 `integrated_commit` 与 `target_head_after`（`$MERGED_COMMIT`）完全一致。

运行人读输出 `task show`，核对「十秒回答」：

```bash
$CLI --root "$ROOT" task show "$PROJECT_ID" "$TASK_ID"
```

**核对字段**：
- 输出直接包含六行核心状态：
  - `goal`: `Fix double() in calculator.py`
  - `state`: `completed; contract revision 1`
  - `blocking`: `nothing`
  - `harness`: `claude-code (via herdr, model none) — invocation invocation-... completed`（按工种明确显示当前是谁在回答）
  - `evidence`: 同时列出准入版本 `csr-...`、已发布评审请求、Integration Receipt 与 Completion Receipt
  - `next`: `nothing — the Task is completed`

---

## 三、B 链：受保护 GitHub `main` + 真 Codex 写入交付

**实跑状态**：实跑过（提交 `009cfab`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-20-演示3验收-ubuntu.md`；其中 `hctl2-tool wait commit-ci` 亦在提交 `631e47c` 上对发布提交 `f8e4a451fd6db10aff5f8e33adbd06b287602db0` 实跑核验）。

> [!NOTE]
> **GitHub 登录与凭据说明**：现行做法复用人这台机器上已登录的 `gh`（随包的 `$WS/prefix/libexec/hctl2/gh` 与 `hctl2-tool` 读取宿主机当前用户的 `gh` 登录态，操作前确认 `gh auth status` 已登录并对 `yesme/hctl2-canary` 有写权限）。小活 AB 合入后改成 control 自己的凭据。

### 1. 前置检查与克隆 `yesme/hctl2-canary`

**实跑状态**：实跑过（提交 `009cfab`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-20-演示3验收-ubuntu.md`）。

确认本机 `gh` 已登录（滤除令牌行）、`codex --version` 满足最低版本要求（`>= 0.153.4`），核对公开沙盒仓库 `yesme/hctl2-canary` 的 `main` 分支保护规则，并将 `yesme/hctl2-canary` 克隆到 `$WS/canary-source`：

```bash
gh auth status 2>&1 | grep -v -i 'token'
codex --version
gh api repos/yesme/hctl2-canary --jq '{id, full_name, default_branch}'
gh api repos/yesme/hctl2-canary/branches/main/protection \
  --jq '{required_checks: .required_status_checks.contexts, enforce_admins: .enforce_admins.enabled, required_approvals: .required_pull_request_reviews.required_approving_review_count}'

rm -rf "$WS/canary-source"
git clone --quiet --branch main --single-branch https://github.com/yesme/hctl2-canary.git "$WS/canary-source"
git -C "$WS/canary-source" config user.name "Demo3 Acceptance"
git -C "$WS/canary-source" config user.email "demo3@example.test"
export B_BASELINE_SHA="$(git -C "$WS/canary-source" rev-parse HEAD)"
echo "B_BASELINE_SHA=$B_BASELINE_SHA"
```

**核对字段**：
- `yesme/hctl2-canary` 的仓库数字 ID 为 `1407796416`，`default_branch` 为 `"main"`。
- `main` 分支保护规则：`required_checks: ["canary"]`、`enforce_admins: true`、`required_approvals: 0`。

### 2. 注册并激活外部 GitHub Repo（`repo register`）

**实跑状态**：实跑过（提交 `009cfab`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-20-演示3验收-ubuntu.md`）。

注册输入照 `src/packaging/release/demo3-github.sh`（`origin: "external"`、`platform: "github"`、`instance: "github.com"`、`platform_repo_id: "1407796416"`、`platform_path: "yesme/hctl2-canary"`、`default_source: "github_issues"`），预览并提交确认：

```bash
cat << EOF > "$WS/work/b-repo.json"
{
  "name": "hctl2-canary",
  "origin": "external",
  "platform": "github",
  "instance": "github.com",
  "platform_repo_id": "1407796416",
  "platform_path": "yesme/hctl2-canary",
  "default_source": "github_issues",
  "local": { "machine": "control", "path": "$WS/canary-source" }
}
EOF

PREV_BREG="$($CLI --json --root "$ROOT" repo register --input "$WS/work/b-repo.json" --key b-register)"
TOK_BREG="$(echo "$PREV_BREG" | jq -r .preview_token)"
SUB_BREG="$($CLI --json --root "$ROOT" repo register --input "$WS/work/b-repo.json" --key b-register --preview-token "$TOK_BREG")"
echo "$SUB_BREG" | jq -c '{lifecycle: .registration.lifecycle, version: .registration.version, delivered: .registration.delivered, repo_id: .registration.repo_id, full_name: .registration.observed.full_name, stable_id: .registration.observed.stable_id, error: .error}'
export B_REPO_ID="$(echo "$SUB_BREG" | jq -r .registration.repo_id)"
export B_STABLE_ID="$(echo "$SUB_BREG" | jq -r .registration.observed.stable_id)"
```

**核对字段**：
- `registration.lifecycle`: `"active"`，`registration.delivered`: `true`（外部 GitHub 仓库直接激活，不触发本地 Gitea 镜像创建）。
- `registration.observed.full_name`: `"yesme/hctl2-canary"`，`registration.observed.stable_id`: `"1407796416"`。

### 3. 接受 `codex-cli` 工种、创建写入型 Profile、新建 Project 并选入主 Room 名册

**实跑状态**：实跑过（提交 `009cfab`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-20-演示3验收-ubuntu.md`）。

从已配对的原生 Agency 目录中提取 `codex-cli` 工种引用并执行 `agency accept`，创建对应 `codex-cli` 的写入型 Worker Profile，新建绑定 `$B_REPO_ID` 的 Project 并将 `codex-cli` 选入主 Room 名册：

```bash
CATALOG_OUT="$($CLI --json --root "$ROOT" agency catalog native)"
echo "$CATALOG_OUT" | jq -c '.professions[] | select(.reference.id=="codex-cli")' > "$WS/work/codex-profession.json"
jq -c '.reference' "$WS/work/codex-profession.json" > "$WS/work/codex-ref.json"
jq -c '{id: .reference.id, revision: .reference.revision, harness, model, terms}' "$WS/work/codex-profession.json"

ACCEPT_CODEX="$($CLI --json --root "$ROOT" agency accept --binding-id native --reference "$WS/work/codex-ref.json" --key b-accept-codex)"
export B_PROF_KEY="$(echo "$ACCEPT_CODEX" | jq -c .profession.key)"
export B_PROF_VER="$(echo "$ACCEPT_CODEX" | jq -r .profession.version)"
export B_PROF_DIGEST="$(jq -r '.digest' "$WS/work/codex-ref.json")"

B_HARNESS="$(jq -c '.harness' "$WS/work/codex-profession.json")"
B_MODEL="$(jq -c '.model' "$WS/work/codex-profession.json")"

cat << EOF > "$WS/work/b-profile.json"
{
  "id": "demo3-codex-write",
  "profile": {
    "harness": $B_HARNESS,
    "model": $B_MODEL,
    "mode": "write",
    "permissions": ["context.read", "git.read", "git.write"],
    "environment": [],
    "required_capabilities": {
      "event_cursor": false,
      "exact_attach": false,
      "input": false,
      "input_provenance": false,
      "isolation_effects": [],
      "managed_single_writer": false,
      "secure_input": false,
      "stop": true,
      "tool_execution_unmediated": false
    },
    "max_context_bytes": 262144
  }
}
EOF

PREV_BPROF="$($CLI --json --root "$ROOT" profile create --input "$WS/work/b-profile.json" --key b-profile)"
TOK_BPROF="$(echo "$PREV_BPROF" | jq -r .preview_token)"
SUB_BPROF="$($CLI --json --root "$ROOT" profile create --input "$WS/work/b-profile.json" --key b-profile --preview-token "$TOK_BPROF")"
export B_PROFILE_REV="$(echo "$SUB_BPROF" | jq -c .revision)"

cat << EOF > "$WS/work/b-project.json"
{
  "repo_id": "$B_REPO_ID",
  "definition": {
    "name": "demo3 github acceptance",
    "goal": "deliver tested code through protected GitHub main",
    "scope": "yesme/hctl2-canary",
    "roles": [],
    "role_members": {},
    "defaults": {},
    "settings": {
      "selection_policy": {},
      "publish_review_requires_confirmation": false
    }
  }
}
EOF

PREV_BPROJ="$($CLI --json --root "$ROOT" project create --input "$WS/work/b-project.json" --key b-project)"
TOK_BPROJ="$(echo "$PREV_BPROJ" | jq -r .preview_token)"
SUB_BPROJ="$($CLI --json --root "$ROOT" project create --input "$WS/work/b-project.json" --key b-project --preview-token "$TOK_BPROJ")"
export B_PROJECT_ID="$(echo "$SUB_BPROJ" | jq -r .project_id)"
export B_ROOM_ID="$(echo "$SUB_BPROJ" | jq -r .main_room_id)"

B_PROJ_VER="$($CLI --json --root "$ROOT" project show "$B_PROJECT_ID" | jq -r .project.version)"

cat << EOF > "$WS/work/b-select.json"
{
  "project_id": "$B_PROJECT_ID",
  "project_version": $B_PROJ_VER,
  "room_id": "$B_ROOM_ID",
  "topic_command_key": null,
  "roster_version": null,
  "selections": [
    {
      "room_id": "$B_ROOM_ID",
      "selected_item": { "key": $B_PROF_KEY, "version": { "state": $B_PROF_VER } },
      "profession": { "key": $B_PROF_KEY, "version": { "state": $B_PROF_VER } },
      "profession_digest": "$B_PROF_DIGEST",
      "agency": {
        "key": { "id": "native", "kind": "agency_binding", "scope": { "kind": "control" } },
        "version": { "state": 1 }
      },
      "required_skills": [],
      "optional_skills": [],
      "worker_profiles": [ $B_PROFILE_REV ],
      "responsibility": "research",
      "permission": { "allow": ["context.read", "git.read", "git.write"] },
      "budget": { "max_bytes": 262144 },
      "display_name": "Codex writer",
      "persona_tags": []
    }
  ]
}
EOF

PREV_BSEL="$($CLI --json --root "$ROOT" project select --input "$WS/work/b-select.json" --key b-select)"
TOK_BSEL="$(echo "$PREV_BSEL" | jq -r .preview_token)"
$CLI --json --root "$ROOT" project select --input "$WS/work/b-select.json" --key b-select --preview-token "$TOK_BSEL" \
  | jq -c '{roster_version, candidate_validation, error}'
```

**核对字段**：
- `candidate_validation`: `"accepted_catalog_and_project_policy"`，`roster_version`: `1`，`error`: `null`。

### 4. 连接 `github_issues` 任务源、创建 Task 并采纳契约

**实跑状态**：实跑过（提交 `009cfab`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-20-演示3验收-ubuntu.md`）。

> [!IMPORTANT]
> **耗时提示**：从 `task connect` 到 `task create` 完成要等几分钟（本次实跑共约 4 分 50 秒，其中 `task create` 从预览到 GitHub issue 建出约 2 分 30 秒）。原因是 `task attach` 与 `task create` 在预览和提交前会触发对 `github_issues` 源的对账快照读取，随包的 `$WS/prefix/libexec/hctl2/gh` 会逐个读取 `yesme/hctl2-canary` 上已有 issue 的依赖与子 issue（`parent`、`sub_issues`、`dependencies/blocked_by`、`dependencies/blocking`），执行期间请耐心等待返回，不要误判为挂起而中断。

连接 `github_issues` 任务源并挂接到 `$B_PROJECT_ID`，创建要求在独立子目录 `canary_cases/acc_$B_TS/` 下实现 `digit_sum.py` 与单元测试 `test_digit_sum.py` 的 Task，随后执行 `task adopt` 采纳契约：

```bash
cat << EOF > "$WS/work/b-source.json"
{
  "repo_id": "$B_REPO_ID",
  "candidate_id": "github_issues",
  "consent": true,
  "make_default": false
}
EOF

PREV_BSRC="$($CLI --json --root "$ROOT" task connect --input "$WS/work/b-source.json" --key b-connect)"
TOK_BSRC="$(echo "$PREV_BSRC" | jq -r .preview_token)"
SUB_BSRC="$($CLI --json --root "$ROOT" task connect --input "$WS/work/b-source.json" --key b-connect --preview-token "$TOK_BSRC")"
export B_SOURCE_ID="$(echo "$SUB_BSRC" | jq -r .source_id)"

B_PROJ_VER="$($CLI --json --root "$ROOT" project show "$B_PROJECT_ID" | jq -r .project.version)"

cat << EOF > "$WS/work/b-attach.json"
{
  "project_id": "$B_PROJECT_ID",
  "project_version": $B_PROJ_VER,
  "source_id": "$B_SOURCE_ID",
  "approved_scope": "$B_STABLE_ID",
  "consent": true
}
EOF

PREV_BATT="$($CLI --json --root "$ROOT" task attach --input "$WS/work/b-attach.json" --key b-attach)"
TOK_BATT="$(echo "$PREV_BATT" | jq -r .preview_token)"
$CLI --json --root "$ROOT" task attach --input "$WS/work/b-attach.json" --key b-attach --preview-token "$TOK_BATT" \
  | jq -c '{source_id, error}'

export B_TS="$(date +%s)"
B_PROJ_VER="$($CLI --json --root "$ROOT" project show "$B_PROJECT_ID" | jq -r .project.version)"

cat << EOF > "$WS/work/b-task.json"
{
  "project_id": "$B_PROJECT_ID",
  "project_version": $B_PROJ_VER,
  "source_id": "$B_SOURCE_ID",
  "title": "Demo 3 acceptance B chain $B_TS",
  "body": "In a new directory canary_cases/acc_$B_TS/, add digit_sum.py defining digit_sum(n), which returns the sum of the decimal digits of abs(n) (digit_sum(0) == 0, digit_sum(-907) == 16), and test_digit_sum.py with unittest cases for zero, a positive and a negative number. Run /usr/bin/python3 -B -m unittest -v test_digit_sum inside that directory and report the command and its output. Leave all changes uncommitted for Agency sealing. Do not push, publish, obtain credentials, or edit harness global settings."
}
EOF

PREV_BTASK="$($CLI --json --root "$ROOT" task create --input "$WS/work/b-task.json" --key b-task-create)"
TOK_BTASK="$(echo "$PREV_BTASK" | jq -r .preview_token)"
SUB_BTASK="$($CLI --json --root "$ROOT" task create --input "$WS/work/b-task.json" --key b-task-create --preview-token "$TOK_BTASK")"
export B_TASK_ID="$(echo "$SUB_BTASK" | jq -r .task_id)"
export B_TASK_VER="$(echo "$SUB_BTASK" | jq -r .task.version)"
echo "$SUB_BTASK" | jq -c '{effect_state, task_id, version: .task.version, number: .task.data.number, entity: .task.data.entity, error}'

B_PROJ_VER="$($CLI --json --root "$ROOT" project show "$B_PROJECT_ID" | jq -r .project.version)"

B_CONTRACT_JSON="$(cat << EOF
{
  "scope": "canary_cases/acc_$B_TS",
  "expected_outcome": "tested digit_sum merged through protected main",
  "acceptance": [
    {
      "text": "the Task change is integrated",
      "grade": "mechanical",
      "evidence": {
        "accept": "integration_receipt",
        "min_channel": "unmediated"
      }
    },
    {
      "text": "a human verified the change and its tests",
      "grade": "human"
    }
  ],
  "roles": [],
  "capabilities": []
}
EOF
)"
B_PROPOSAL_DIGEST="$(printf '%s' "$B_CONTRACT_JSON" | jq -cS . | tr -d '\n' | sha256sum | awk '{print $1}')"

cat << EOF > "$WS/work/b-adopt.json"
{
  "project_id": "$B_PROJECT_ID",
  "project_version": $B_PROJ_VER,
  "task_id": "$B_TASK_ID",
  "version": $B_TASK_VER,
  "adoption": {
    "contract": $B_CONTRACT_JSON,
    "origin": {
      "kind": "local",
      "reference": {
        "key": {
          "scope": { "kind": "project", "id": "$B_PROJECT_ID" },
          "kind": "project",
          "id": "$B_PROJECT_ID"
        },
        "version": { "state": $B_PROJ_VER }
      },
      "proposal_digest": "$B_PROPOSAL_DIGEST"
    }
  }
}
EOF

PREV_BADOPT="$($CLI --json --root "$ROOT" task adopt --input "$WS/work/b-adopt.json" --key b-task-adopt)"
TOK_BADOPT="$(echo "$PREV_BADOPT" | jq -r .preview_token)"
SUB_BADOPT="$($CLI --json --root "$ROOT" task adopt --input "$WS/work/b-adopt.json" --key b-task-adopt --preview-token "$TOK_BADOPT")"
echo "$SUB_BADOPT" | jq -c '{revision_number: .task.data.revision.number, version: .task.version, error}'
```

**核对字段**：
- `task create` 返回 `effect_state: "confirmed"`，`entity.provider` 为 `"github_issues@github.com"`，`number` 为 `yesme/hctl2-canary` 上新建的 GitHub issue 编号。
- `task adopt` 返回 `revision_number: 1`、`error: null`。

### 5. 带 `task_id` 发起写入型调用（真 Codex 会话改代码并封存）

**实跑状态**：实跑过（提交 `009cfab`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-20-演示3验收-ubuntu.md`）。

带 `task_id` 发起写入型调用，`request` 只写一句 `"Work on the adopted Task as its contract specifies."`（不重复实现规格，由开工包内联交付的 `task_contract` 与 `task_card` 驱动真 Codex 会话完成代码编写与测试，并由 Agency 调用 `hctl2-tool` 封存交回）：

```bash
B_DEADLINE_MS="$(( ($(date +%s) + 900) * 1000 ))"

cat << EOF > "$WS/work/b-invoke.json"
{
  "project_id": "$B_PROJECT_ID",
  "room_id": "$B_ROOM_ID",
  "task_id": "$B_TASK_ID",
  "target": "research",
  "profile": $B_PROFILE_REV,
  "request": "Work on the adopted Task as its contract specifies.",
  "budget": 262144,
  "deadline_ms": $B_DEADLINE_MS,
  "retry_of": null,
  "write": {
    "change_set_id": null,
    "baseline_commit": "$B_BASELINE_SHA",
    "target_branch": "main",
    "allow_update": true
  }
}
EOF

# 查看人读派工预览
$CLI --root "$ROOT" invocation preview --input "$WS/work/b-invoke.json" --key b-invoke

PREV_BINV="$($CLI --json --root "$ROOT" invocation preview --input "$WS/work/b-invoke.json" --key b-invoke)"
TOK_BINV="$(echo "$PREV_BINV" | jq -r .preview_token)"
export B_CHANGESET_ID="$(echo "$PREV_BINV" | jq -r .effect_summary.preview.write.lease.pending.change_set_id)"
echo "$PREV_BINV" | jq -c '{authorization: .effect_summary.preview.write.authorization, change_set_id: .effect_summary.preview.write.lease.pending.change_set_id, baseline: .effect_summary.preview.write.lease.pending.baseline_commit, publication_target: .effect_summary.preview.write.publication_target}'

SUB_BINV="$($CLI --json --root "$ROOT" invocation start --input "$WS/work/b-invoke.json" --key b-invoke --preview-token "$TOK_BINV")"
export B_INVOCATION_ID="$(echo "$SUB_BINV" | jq -r .invocation_id)"

for i in $(seq 1 450); do
  B_SHOW="$($CLI --json --root "$ROOT" invocation show "$B_PROJECT_ID" "$B_INVOCATION_ID")"
  B_STATE="$(echo "$B_SHOW" | jq -r .state)"
  case "$B_STATE" in
    completed)
      echo "B invocation completed: $(echo "$B_SHOW" | jq -r .reason)"
      break
      ;;
    failed|cancelled|lost)
      echo "B invocation ended in $B_STATE: $(echo "$B_SHOW" | jq -r .reason)"
      break
      ;;
  esac
  sleep 2
done
[ "$B_STATE" = "completed" ] || { echo "B 链调用未完成: state=$B_STATE"; return 1 2>/dev/null || exit 1; }

# 核对开工包条目（契约 task_contract 与卡正文 task_card 已内联封入 Context Bundle）
echo "$B_SHOW" | jq -c '.dispatch_intent.data.value.bundle.document.entries[] | {source: .source.id, mode: .delivery.mode, bytes: (.delivery.bytes | @base64d | length), bytes_digest}'
```

**核对字段**：
- `invocation show` 返回 `state: "completed"`，`reason: "Git version admitted; not Task acceptance"`。
- `bundle.document.entries` 中包含内联的 `task_contract/<B_TASK_ID>` 与 `task_card/<B_TASK_ID>`。

### 6. 自动发布 PR、用 `hctl2-tool wait commit-ci` 等待 `canary` 通过并预览提交合入

**实跑状态**：实跑过（提交 `009cfab`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-20-演示3验收-ubuntu.md`；`hctl2-tool wait commit-ci` 亦在提交 `631e47c` 上对发布提交 `f8e4a451fd6db10aff5f8e33adbd06b287602db0` 实跑核验）。

核对封存的 `ChangeSet Revision` 与 diff，等待 `review show` 中两段发布（推送分支与创建 GitHub PR）各自回读确认，随后用产品自己的 `$WS/prefix/bin/hctl2-tool wait commit-ci`（见 `docs/usage.md`「事实等待」，不用人的 `gh api`）等待受保护 `main` 的必需检查 `canary` 通过，再执行合入预览与提交：

```bash
B_CS_SHOW="$($CLI --json --root "$ROOT" changeset show "$B_REPO_ID" "$B_CHANGESET_ID")"
export B_REVISION_ID="$(echo "$B_CS_SHOW" | jq -r '.revisions[0].change_set_revision_id')"
echo "$B_CS_SHOW" | jq '.revisions[0]'
$CLI --json --root "$ROOT" changeset diff "$B_REPO_ID" "$B_CHANGESET_ID" "$B_REVISION_ID" | jq -r .diff

for i in $(seq 1 60); do
  B_REV_LIST="$($CLI --json --root "$ROOT" review list "$B_REPO_ID")"
  B_PUB_STATE="$(echo "$B_REV_LIST" | jq -r --arg cs "$B_CHANGESET_ID" '.items[] | select(.change_set_id==$cs) | .state')"
  [ "$B_PUB_STATE" = "published" ] && break
  sleep 2
done
export B_PUBLISH_INTENT_ID="$(echo "$B_REV_LIST" | jq -r --arg cs "$B_CHANGESET_ID" '.items[] | select(.change_set_id==$cs) | .intent_id')"
B_PUB_SHOW="$($CLI --json --root "$ROOT" review show "$B_REPO_ID" "$B_PUBLISH_INTENT_ID")"
echo "$B_PUB_SHOW" | jq '{state: .intent.state, stages: .stages, mapping: .mappings[0]}'
export B_PUBLISH_COMMIT="$(echo "$B_PUB_SHOW" | jq -r '.mappings[0].platform_commit_sha')"
export B_PR_INDEX="$(echo "$B_PUB_SHOW" | jq -r '.mappings[0].review_request.index')"
export B_HEAD_BRANCH="$(echo "$B_PUB_SHOW" | jq -r '.mappings[0].review_request.head_branch')"

# 用产品自己的 hctl2-tool wait commit-ci 等待必需检查 canary 在发布提交上通过（不用人的 gh api）
"$WS/prefix/bin/hctl2-tool" wait --deadline "$(( $(date +%s) + 600 ))" \
  commit-ci --repo yesme/hctl2-canary --commit "$B_PUBLISH_COMMIT" \
  | jq '{schema, outcome, evidence_level, checks: [.observation.check_runs[] | {name, status, conclusion}]}'

cat << EOF > "$WS/work/b-integration.json"
{
  "repo_id": "$B_REPO_ID",
  "change_set_revision_id": "$B_REVISION_ID",
  "target_kind": "platform",
  "target_ref": "refs/heads/main",
  "form": "accept_advance",
  "strategy": "merge_commit"
}
EOF

# 查看人读合入预览（末尾 Confirm with 给出可直接按原词执行的 integration submit 命令）
$CLI --root "$ROOT" integration preview --input "$WS/work/b-integration.json" --key b-merge

PREV_BINT="$($CLI --json --root "$ROOT" integration preview --input "$WS/work/b-integration.json" --key b-merge)"
echo "$PREV_BINT" | jq -c '.effect_summary.protection'
TOK_BINT="$(echo "$PREV_BINT" | jq -r .preview_token)"
SUB_BINT="$($CLI --json --root "$ROOT" integration submit --input "$WS/work/b-integration.json" --key b-merge --preview-token "$TOK_BINT")"
export B_INTEGRATION_INTENT_ID="$(echo "$SUB_BINT" | jq -r .intent_id)"

for i in $(seq 1 90); do
  B_INT_SHOW="$($CLI --json --root "$ROOT" integration show "$B_REPO_ID" "$B_INTEGRATION_INTENT_ID")"
  B_INT_STATE="$(echo "$B_INT_SHOW" | jq -r .intent.state)"
  [ "$B_INT_STATE" = "succeeded" ] || [ "$B_INT_STATE" = "failed" ] && break
  sleep 1
done

export B_RECEIPT_ID="$(echo "$B_INT_SHOW" | jq -r .receipt.receipt_id)"
export B_MERGED_COMMIT="$(echo "$B_INT_SHOW" | jq -r .receipt.integrated_commit)"
echo "$B_INT_SHOW" | jq '{state: .intent.state, receipt_id: .receipt.receipt_id, evidence_level: .receipt.evidence_level, git_readback_level: .receipt.readback.git.evidence_level, integrated_commit: .receipt.integrated_commit, target_head_after: .receipt.target_head_after, platform_readback: .receipt.readback.platform}'
```

**核对字段**：
- `review show` 中 `state: "published"`，`stages.push.confirmed` 与 `stages.review_request.confirmed` 均为 `true`。
- `hctl2-tool wait commit-ci` 返回退出码 `0`，输出 `schema: "hctl2.external-fact.v1"`、`outcome: "established"`、`evidence_level: "unmediated"`，且 `checks` 中 `canary` 的 `status: "completed"`、`conclusion: "success"`。
- `--json` 合入预览的 `effect_summary.protection` 给出冻结的 GitHub 分支保护快照（`required_checks: ["canary"]`、`requires_review_request: true`、`enforce_admins.enabled: true`）；人读合入预览末尾 `Confirm with` 给出 `integration submit --input ... --key b-merge --preview-token ...`。
- `integration show` 返回 `state: "succeeded"`，签出 `Integration Receipt`（`evidence_level: "hctl2-tool"`、`git_readback_level: "unmediated"`，`integrated_commit` 与 `target_head_after` 一致）。

### 7. 完成 Task（`task complete`）与十秒回答（`task show`）

**实跑状态**：实跑过（提交 `009cfab`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-20-演示3验收-ubuntu.md`）。

引用 `$B_RECEIPT_ID` 执行 `task complete` 的人读预览、`--json` 预览与确认提交，并运行 `task show` 核对十秒回答：

```bash
B_TASK_CUR_VER="$($CLI --json --root "$ROOT" task show "$B_PROJECT_ID" "$B_TASK_ID" | jq -r .version)"

cat << EOF > "$WS/work/b-complete.json"
{
  "project_id": "$B_PROJECT_ID",
  "task_id": "$B_TASK_ID",
  "version": $B_TASK_CUR_VER,
  "lifecycle_version": 1,
  "revision_number": 1,
  "acceptance": [
    {
      "item": 0,
      "channel": "unmediated",
      "generation": 1,
      "judge": { "kind": "hctl2_tool" },
      "producer": "hctl2-tool",
      "references": [
        {
          "key": {
            "id": "$B_RECEIPT_ID",
            "kind": "integration_receipt",
            "scope": { "id": "$B_REPO_ID", "kind": "repo" }
          },
          "version": { "state": 1 }
        }
      ]
    },
    {
      "item": 1,
      "channel": "narrated",
      "generation": null,
      "judge": { "kind": "human", "actor": "local-owner:$(id -u)" },
      "producer": null,
      "references": []
    }
  ]
}
EOF

# 查看人读完成预览
$CLI --root "$ROOT" task complete --input "$WS/work/b-complete.json" --key b-complete

PREV_BCOMP="$($CLI --json --root "$ROOT" task complete --input "$WS/work/b-complete.json" --key b-complete)"
TOK_BCOMP="$(echo "$PREV_BCOMP" | jq -r .preview_token)"
SUB_BCOMP="$($CLI --json --root "$ROOT" task complete --input "$WS/work/b-complete.json" --key b-complete --preview-token "$TOK_BCOMP")"
echo "$SUB_BCOMP" | jq '{lifecycle: .lifecycle, lifecycle_version: .lifecycle_version, receipt_id: .receipt_id, items: [.items[] | {item, grade, outcome, validation_level, references: [.references[].key.id]}]}'

$CLI --root "$ROOT" task show "$B_PROJECT_ID" "$B_TASK_ID"
```

**核对字段**：
- `task complete` 返回 `lifecycle: "completed"`、`lifecycle_version: 2`、`receipt_id: "<B_TASK_ID>:2"`。
- `task show` 单屏输出六行十秒回答：`goal`、`state: completed; contract revision 1`、`blocking: nothing`、`harness: codex-cli (via herdr, model none) — invocation ... completed`、`evidence`（同时列出 `version csr-...`、`review request #<B_PR_INDEX> (published)`、`Integration Receipt` 与 `Completion Receipt`）与 `next: nothing — the Task is completed`。
- **Task 完成后外部卡写回现状**：`src/crates/task/src/planning.rs` 中的 `Action::Complete` 在同一 Store 事务内写入 `Task Completion Receipt`（`task_completion_receipt`）并将 `Task.lifecycle` 推进为 `"completed"`，但 `plan.effects` 为空，未向任务源 outbox 入队关闭外部 issue 或追加完成评论的写回意图（GitHub `github_issues` 与本地 Gitea `gitea_issues` 均如此）。因此 `task complete` 完成后，GitHub issue（如本次实跑的 `#41`）与本地 Gitea issue（A 链 `#1`、C 链 `#2`）在平台侧仍保持 `open`。GitHub 侧留下的 issue 与 PR 源分支按下一小节由人用 `gh` 清理。

### 8. 收尾清理 `yesme/hctl2-canary` 上留下的远端对象

**实跑状态**：照用例或代码写的，未实跑（供人类操作者在完成核验后用自己的 `gh` 收尾清理远端沙盒遗留物）。

B 链跑完后，`yesme/hctl2-canary` 上会留下两项未关闭/未删除的远端对象：一是第 4 步新建的 GitHub issue（`task complete` 未自动写回关闭，仍为 `open`），二是第 6 步已合入 PR 的远端源分支 `refs/heads/$B_HEAD_BRANCH`（即 `hctl2/cs-<B_CHANGESET_ID>`）。由人用自己的 `gh` 执行以下命令收尾：

```bash
export B_ISSUE_NUMBER="$($CLI --json --root "$ROOT" task show "$B_PROJECT_ID" "$B_TASK_ID" | jq -r .data.number)"
export B_HEAD_BRANCH="$(echo "$B_PUB_SHOW" | jq -r '.mappings[0].review_request.head_branch')"

# 1. 关闭本次 B 链在 yesme/hctl2-canary 上创建的 issue
gh issue close "$B_ISSUE_NUMBER" --repo yesme/hctl2-canary --comment "Closed after demo-3 B chain walkthrough."

# 2. 删除已合入 PR（#$B_PR_INDEX）留下的远端源分支
gh api -X DELETE "repos/yesme/hctl2-canary/git/refs/heads/$B_HEAD_BRANCH"

# 3. 核对 issue 已关闭、远端源分支已不存在
gh api "repos/yesme/hctl2-canary/issues/$B_ISSUE_NUMBER" --jq '{number, state, title}'
gh api "repos/yesme/hctl2-canary/pulls/$B_PR_INDEX" --jq '{number, state, merged, head_ref: .head.ref}'
```

若要直接清理 `2026-10-20` 验收实跑（`.memo/log/2026-10-20-演示3验收-ubuntu.md`）在 `yesme/hctl2-canary` 上留下的 issue `#41` 与 PR `#42` 源分支 `hctl2/cs-9bc595043cf485f07bcf90479ac85dc56518ba2ece0b262aae0dda5b9929a399`，可直接执行：

```bash
gh issue close 41 --repo yesme/hctl2-canary --comment "Closed after 2026-10-20 demo-3 B chain acceptance."
gh api -X DELETE "repos/yesme/hctl2-canary/git/refs/heads/hctl2/cs-9bc595043cf485f07bcf90479ac85dc56518ba2ece0b262aae0dda5b9929a399"
```

---

## 四、C 链：开启「须人显式确认」开关的三次预览链

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

在已激活的本地 Gitea Repo（`$REPO_ID`）上新建一个开启 `publish_review_requires_confirmation: true` 的独立 Project，挂接同一 Gitea 任务源并创建采纳契约的 Task，以 A 链合入后的最新 `main` 头（`$MERGED_COMMIT`）为 `baseline_commit` 走完发布（`review publish`）、合入（`integration preview` / `integration submit`）与完成（`task complete`）三次人工预览闭环。

### 1. 创建开启发布确认开关的 Project、选入写入参与者并创建采纳 Task

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

```bash
cat << EOF > "$WS/work/c-project.json"
{
  "repo_id": "$REPO_ID",
  "definition": {
    "name": "demo3 confirm walkthrough",
    "goal": "publish needs a human",
    "scope": "demo3-walkthrough",
    "roles": [],
    "role_members": {},
    "defaults": {},
    "settings": {
      "selection_policy": {},
      "publish_review_requires_confirmation": true
    }
  }
}
EOF

PREV_CPROJ="$($CLI --json --root "$ROOT" project create --input "$WS/work/c-project.json" --key c-project)"
TOK_CPROJ="$(echo "$PREV_CPROJ" | jq -r .preview_token)"
SUB_CPROJ="$($CLI --json --root "$ROOT" project create --input "$WS/work/c-project.json" --key c-project --preview-token "$TOK_CPROJ")"
export C_PROJECT_ID="$(echo "$SUB_CPROJ" | jq -r .project_id)"
export C_ROOM_ID="$(echo "$SUB_CPROJ" | jq -r .main_room_id)"

C_PROJ_VER="$($CLI --json --root "$ROOT" project show "$C_PROJECT_ID" | jq -r .project.version)"

cat << EOF > "$WS/work/c-select.json"
{
  "project_id": "$C_PROJECT_ID",
  "project_version": $C_PROJ_VER,
  "room_id": "$C_ROOM_ID",
  "topic_command_key": null,
  "roster_version": null,
  "selections": [
    {
      "room_id": "$C_ROOM_ID",
      "selected_item": {
        "key": $CLAUDE_PROF_KEY,
        "version": { "state": $CLAUDE_PROF_VER }
      },
      "profession": {
        "key": $CLAUDE_PROF_KEY,
        "version": { "state": $CLAUDE_PROF_VER }
      },
      "profession_digest": "$CLAUDE_PROF_DIGEST",
      "agency": {
        "key": {
          "id": "native",
          "kind": "agency_binding",
          "scope": { "kind": "control" }
        },
        "version": { "state": 1 }
      },
      "required_skills": [],
      "optional_skills": [],
      "worker_profiles": [
        $WRITE_PROFILE_REV
      ],
      "responsibility": "research",
      "permission": {
        "allow": [
          "context.read",
          "git.read",
          "git.write"
        ]
      },
      "budget": {
        "max_bytes": 262144
      },
      "display_name": "Claude Code writer",
      "persona_tags": []
    }
  ]
}
EOF

PREV_CSEL="$($CLI --json --root "$ROOT" project select --input "$WS/work/c-select.json" --key c-select)"
TOK_CSEL="$(echo "$PREV_CSEL" | jq -r .preview_token)"
$CLI --json --root "$ROOT" project select --input "$WS/work/c-select.json" --key c-select --preview-token "$TOK_CSEL"

# 将已连接的 Gitea 任务源挂接到 C 链 Project，并创建、采纳 C 链 Task
C_PROJ_VER="$($CLI --json --root "$ROOT" project show "$C_PROJECT_ID" | jq -r .project.version)"

cat << EOF > "$WS/work/c-attach.json"
{
  "project_id": "$C_PROJECT_ID",
  "project_version": $C_PROJ_VER,
  "source_id": "$SOURCE_ID",
  "approved_scope": "$STABLE_ID",
  "consent": true
}
EOF

PREV_CATT="$($CLI --json --root "$ROOT" task attach --input "$WS/work/c-attach.json" --key c-attach)"
TOK_CATT="$(echo "$PREV_CATT" | jq -r .preview_token)"
$CLI --json --root "$ROOT" task attach --input "$WS/work/c-attach.json" --key c-attach --preview-token "$TOK_CATT"

C_PROJ_VER="$($CLI --json --root "$ROOT" project show "$C_PROJECT_ID" | jq -r .project.version)"

cat << EOF > "$WS/work/c-task.json"
{
  "project_id": "$C_PROJECT_ID",
  "project_version": $C_PROJ_VER,
  "source_id": "$SOURCE_ID",
  "title": "Add trailing comment in calculator.py (C chain)",
  "body": "Add a trailing comment line '# demo3 C' to calculator.py and leave it uncommitted for Agency sealing. Do not push or obtain credentials."
}
EOF

PREV_CTASK="$($CLI --json --root "$ROOT" task create --input "$WS/work/c-task.json" --key c-task-create)"
TOK_CTASK="$(echo "$PREV_CTASK" | jq -r .preview_token)"
SUB_CTASK="$($CLI --json --root "$ROOT" task create --input "$WS/work/c-task.json" --key c-task-create --preview-token "$TOK_CTASK")"
export C_TASK_ID="$(echo "$SUB_CTASK" | jq -r .task_id)"
export C_TASK_VER="$(echo "$SUB_CTASK" | jq -r .task.version)"

C_PROJ_VER="$($CLI --json --root "$ROOT" project show "$C_PROJECT_ID" | jq -r .project.version)"

C_CONTRACT_JSON="$(cat << 'EOF'
{
  "scope": "demo3-walkthrough",
  "expected_outcome": "trailing comment added and merged after explicit review publish confirmation",
  "acceptance": [
    {
      "text": "the C chain change is integrated",
      "grade": "mechanical",
      "evidence": {
        "accept": "integration_receipt",
        "min_channel": "unmediated"
      }
    },
    {
      "text": "a human verified all three previews",
      "grade": "human"
    }
  ],
  "roles": [],
  "capabilities": []
}
EOF
)"
C_PROPOSAL_DIGEST="$(printf '%s' "$C_CONTRACT_JSON" | jq -cS . | tr -d '\n' | sha256sum | awk '{print $1}')"

cat << EOF > "$WS/work/c-adopt.json"
{
  "project_id": "$C_PROJECT_ID",
  "project_version": $C_PROJ_VER,
  "task_id": "$C_TASK_ID",
  "version": $C_TASK_VER,
  "adoption": {
    "contract": $C_CONTRACT_JSON,
    "origin": {
      "kind": "local",
      "reference": {
        "key": {
          "scope": { "kind": "project", "id": "$C_PROJECT_ID" },
          "kind": "project",
          "id": "$C_PROJECT_ID"
        },
        "version": { "state": $C_PROJ_VER }
      },
      "proposal_digest": "$C_PROPOSAL_DIGEST"
    }
  }
}
EOF

PREV_CADOPT="$($CLI --json --root "$ROOT" task adopt --input "$WS/work/c-adopt.json" --key c-task-adopt)"
TOK_CADOPT="$(echo "$PREV_CADOPT" | jq -r .preview_token)"
$CLI --json --root "$ROOT" task adopt --input "$WS/work/c-adopt.json" --key c-task-adopt --preview-token "$TOK_CADOPT"
```

### 2. 发起写入型调用并观察发布意图停在 `pending_human`

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

以 A 链合入后的最新 `main` 头 `$MERGED_COMMIT` 作为 `baseline_commit`（以便后续合入可直接 fast-forward 推进 `refs/heads/main`）：

```bash
C_DEADLINE_MS="$(( ($(date +%s) + 900) * 1000 ))"

cat << EOF > "$WS/work/c-invoke.json"
{
  "project_id": "$C_PROJECT_ID",
  "room_id": "$C_ROOM_ID",
  "task_id": "$C_TASK_ID",
  "target": "research",
  "profile": $WRITE_PROFILE_REV,
  "request": "Add a trailing comment line '# demo3 C' to calculator.py and leave it uncommitted. Do not run tests. Do not push or obtain credentials.",
  "budget": 262144,
  "deadline_ms": $C_DEADLINE_MS,
  "retry_of": null,
  "write": {
    "change_set_id": null,
    "baseline_commit": "$MERGED_COMMIT",
    "target_branch": "main",
    "allow_update": true
  }
}
EOF

PREV_CINV="$($CLI --json --root "$ROOT" invocation preview --input "$WS/work/c-invoke.json" --key c-invoke)"
TOK_CINV="$(echo "$PREV_CINV" | jq -r .preview_token)"
export C_CHANGESET_ID="$(echo "$PREV_CINV" | jq -r .effect_summary.preview.write.lease.pending.change_set_id)"
echo "$PREV_CINV" | jq '.effect_summary.preview.write.publication_target'

# 若 A 链与 C 链之间宿主机上有其他 claude 进程重写过 ~/.claude.json（改变 inode），
# 重启一次本实例的 agency 以刷新 Herdr 对 ~/.claude.json 的 Landlock PathFd 句柄（不改配对与状态）：
"$WS/prefix/bin/agency" --root "$ROOT/agency" stop >/dev/null 2>&1 || true
HCTL2_INSTALL_ROOT="$INSTALL_ROOT" "$WS/prefix/bin/agency" --root "$ROOT/agency" start >/dev/null

SUB_CINV="$($CLI --json --root "$ROOT" invocation start --input "$WS/work/c-invoke.json" --key c-invoke --preview-token "$TOK_CINV")"
export C_INVOCATION_ID="$(echo "$SUB_CINV" | jq -r .invocation_id)"

for i in $(seq 1 450); do
  C_SHOW="$($CLI --json --root "$ROOT" invocation show "$C_PROJECT_ID" "$C_INVOCATION_ID")"
  C_STATE="$(echo "$C_SHOW" | jq -r .state)"
  case "$C_STATE" in
    completed)
      echo "C invocation completed: $(echo "$C_SHOW" | jq -r .reason)"
      break
      ;;
    failed|cancelled|lost)
      echo "C invocation ended in $C_STATE: $(echo "$C_SHOW" | jq -r .reason)"
      break
      ;;
  esac
  sleep 2
done
[ "$C_STATE" = "completed" ] || { echo "C 链调用未完成: state=$C_STATE"; return 1 2>/dev/null || exit 1; }

C_CS_SHOW="$($CLI --json --root "$ROOT" changeset show "$REPO_ID" "$C_CHANGESET_ID")"
export C_REVISION_ID="$(echo "$C_CS_SHOW" | jq -r '.revisions[0].change_set_revision_id')"

C_REV_LIST="$($CLI --json --root "$ROOT" review list "$REPO_ID")"
export C_PUBLISH_INTENT_ID="$(echo "$C_REV_LIST" | jq -r --arg cs "$C_CHANGESET_ID" '.items[] | select(.change_set_id==$cs) | .intent_id')"
$CLI --json --root "$ROOT" review show "$REPO_ID" "$C_PUBLISH_INTENT_ID" \
  | jq '{intent_id: .intent.intent_id, state: .intent.state, requires_human_confirmation: .intent.policy.policy.requires_human_confirmation, push_dispatched: .stages.push.dispatched}'
```

**核对字段**：
- 派工预览中 `publication_target.requires_human_confirmation` 为 `true`。
- 调用完成后，`review show` 中该 ChangeSet 的发布意图状态停在 `state: "pending_human"`，`requires_human_confirmation: true`，`push_dispatched: false`，尚未向平台推送或建 PR。

### 3. 第一次人工预览：显式确认发布（`review publish`）

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

```bash
# 查看人读发布预览（明确写出推到哪个分支、建到哪个目标、不是合入）
$CLI --root "$ROOT" review publish "$REPO_ID" "$C_PUBLISH_INTENT_ID"

PREV_CPUB="$($CLI --json --root "$ROOT" review publish "$REPO_ID" "$C_PUBLISH_INTENT_ID")"
TOK_CPUB="$(echo "$PREV_CPUB" | jq -r .preview_token)"
echo "$PREV_CPUB" | jq '.effect_summary | {branch: .branch, target_branch: .target_branch, authorizes: .authorizes}'

$CLI --json --root "$ROOT" review publish "$REPO_ID" "$C_PUBLISH_INTENT_ID" --preview-token "$TOK_CPUB"

for i in $(seq 1 30); do
  C_PUB_SHOW="$($CLI --json --root "$ROOT" review show "$REPO_ID" "$C_PUBLISH_INTENT_ID")"
  [ "$(echo "$C_PUB_SHOW" | jq -r .intent.state)" = "published" ] && break
  sleep 1
done

echo "$C_PUB_SHOW" | jq '{state: .intent.state, stages: .stages, mapping: .mappings[0]}'
```

**核对字段**：
- `effect_summary.branch`: `"hctl2/cs-..."`（推送的目标分支）。
- `effect_summary.target_branch`: `"main"`（评审请求的目标分支）。
- `effect_summary.authorizes`: `"publishing this revision for review on the platform: push to the branch, then create or update the review request. Not a merge."`（明确声明不是合入）。
- 提交确认后，意图状态转为 `"published"`，`push` 与 `review_request` 两段均 `confirmed: true`。

### 4. 第二次人工预览：合入（`integration preview` / `integration submit`）

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

```bash
cat << EOF > "$WS/work/c-integration.json"
{
  "repo_id": "$REPO_ID",
  "change_set_revision_id": "$C_REVISION_ID",
  "target_kind": "platform",
  "target_ref": "refs/heads/main",
  "form": "accept_advance",
  "strategy": "fast_forward"
}
EOF

# 查看人读合入预览
$CLI --root "$ROOT" integration preview --input "$WS/work/c-integration.json" --key c-merge

PREV_CINT="$($CLI --json --root "$ROOT" integration preview --input "$WS/work/c-integration.json" --key c-merge)"
TOK_CINT="$(echo "$PREV_CINT" | jq -r .preview_token)"
SUB_CINT="$($CLI --json --root "$ROOT" integration submit --input "$WS/work/c-integration.json" --key c-merge --preview-token "$TOK_CINT")"
export C_INTEGRATION_INTENT_ID="$(echo "$SUB_CINT" | jq -r .intent_id)"

for i in $(seq 1 30); do
  C_INT_SHOW="$($CLI --json --root "$ROOT" integration show "$REPO_ID" "$C_INTEGRATION_INTENT_ID")"
  [ "$(echo "$C_INT_SHOW" | jq -r .intent.state)" = "succeeded" ] && break
  sleep 1
done

export C_RECEIPT_ID="$(echo "$C_INT_SHOW" | jq -r .receipt.receipt_id)"
echo "$C_INT_SHOW" | jq '{state: .intent.state, receipt_id: .receipt.receipt_id, evidence_level: .receipt.evidence_level, git_readback_level: .receipt.readback.git.evidence_level, target_head_after: .receipt.target_head_after}'
```

**核对字段**：
- `intent.state`: `"succeeded"`，签出 C 链对应的 `Integration Receipt`（`$C_RECEIPT_ID`）。

### 5. 第三次人工预览：完成 Task（`task complete`）与十秒回答

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

```bash
C_TASK_CUR_VER="$($CLI --json --root "$ROOT" task show "$C_PROJECT_ID" "$C_TASK_ID" | jq -r .version)"

cat << EOF > "$WS/work/c-complete.json"
{
  "project_id": "$C_PROJECT_ID",
  "task_id": "$C_TASK_ID",
  "version": $C_TASK_CUR_VER,
  "lifecycle_version": 1,
  "revision_number": 1,
  "acceptance": [
    {
      "item": 0,
      "channel": "unmediated",
      "generation": 1,
      "judge": { "kind": "hctl2_tool" },
      "producer": "hctl2-tool",
      "references": [
        {
          "key": {
            "id": "$C_RECEIPT_ID",
            "kind": "integration_receipt",
            "scope": { "id": "$REPO_ID", "kind": "repo" }
          },
          "version": { "state": 1 }
        }
      ]
    },
    {
      "item": 1,
      "channel": "narrated",
      "generation": null,
      "judge": { "kind": "human", "actor": "local-owner:$(id -u)" },
      "producer": null,
      "references": []
    }
  ]
}
EOF

# 查看人读完成预览
$CLI --root "$ROOT" task complete --input "$WS/work/c-complete.json" --key c-complete

PREV_CCOMP="$($CLI --json --root "$ROOT" task complete --input "$WS/work/c-complete.json" --key c-complete)"
TOK_CCOMP="$(echo "$PREV_CCOMP" | jq -r .preview_token)"
SUB_CCOMP="$($CLI --json --root "$ROOT" task complete --input "$WS/work/c-complete.json" --key c-complete --preview-token "$TOK_CCOMP")"
echo "$SUB_CCOMP" | jq '{lifecycle: .lifecycle, lifecycle_version: .lifecycle_version, receipt_id: .receipt_id}'

$CLI --root "$ROOT" task show "$C_PROJECT_ID" "$C_TASK_ID"
```

**核对字段**：
- `lifecycle`: `"completed"`，`receipt_id`: `"<C_TASK_ID>:2"`，`task show` 同样给出完整六行十秒回答。开启确认开关的 Project 全程走完发布（`review publish`）、合入（`integration preview` / `integration submit`）、完成（`task complete`）三次人工预览。

---

## 五、重启恢复、凭据隔离与逃生口怎么看

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

### 1. 重启恢复与不重复外部副作用怎么看

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`；常驻回归用例 Buck2 Build ID `8e13b650-e704-4c70-b58d-d16accbdac9a`）。

精确卡在特定阶段边界的测试注入缝由运行时环境变量 `HCTL2_TEST_SEAM_ROOT` 驱动（`src/apps/control/src/test_seams.rs`），日常手动运行不设置该环境变量，也不通过手动猜测时机强杀进程，而是从以下两处核对重启一致性与外部副作用幂等：

1. **在当前状态根上查看正常启停后的记录与平台幂等**（实跑过）：
   正常停止并重启控制面、Agency 与托管服务后，回读 A 链与 C 链的各项记录：
   ```bash
   $CLI --root "$ROOT" stop
   "$WS/prefix/bin/agency" --root "$ROOT/agency" stop
   $CLI --root "$ROOT" start --secret-backend user-file

   for i in $(seq 1 30); do
     G_READY="$($CLI --json --root "$ROOT" status 2>/dev/null | jq -r '.services.hosted[] | select(.name=="gitea") | .ready')"
     [ "$G_READY" = "true" ] && break
     sleep 1
   done

   $CLI --json --root "$ROOT" changeset show "$REPO_ID" "$CHANGESET_ID" | jq '{version: .change_set.version, revisions: (.revisions | length)}'
   $CLI --json --root "$ROOT" review show "$REPO_ID" "$PUBLISH_INTENT_ID" | jq '{state: .intent.state, index: .mappings[0].review_request.index}'
   $CLI --json --root "$ROOT" integration show "$REPO_ID" "$INTEGRATION_INTENT_ID" | jq '{state: .intent.state, receipt_id: .receipt.receipt_id}'
   $CLI --root "$ROOT" task show "$PROJECT_ID" "$TASK_ID"
   ```
   **怎么看**：重启后 `gitea` 与 `tuwunel` 按已消费集合自动恢复；`changeset show`、`review show`、`integration show` 与 `task show` 的输出与重启前一致，`Integration Receipt` 与 `Task Completion Receipt` 各只有一张，Gitea 平台上不会多出重复的评审请求或合并提交。

2. **查看三处精确杀点的常驻回归用例**（实跑过，Build ID `8e13b650-e704-4c70-b58d-d16accbdac9a`）：
   「发布第一段推送后未建请求」「集成已提交未回读」「完成已预览未提交」三处精确状态的进程被杀后重启恢复，由 `room-cli-test` 中的常驻用例通过测试缝精确卡位并各自重启一次 `control` + `Agency` + 随包服务进行验证：
   ```bash
   cd "$REPO_ROOT/src"
   ./buck2 test root//packaging/release:room-cli-test -- \
     --test-arg=demo3_gitea_chain_restarts_do_not_repeat_side_effects \
     --test-arg=--nocapture
   cd "$WS"
   ```
   **怎么看**：输出中打印 `LIVE CLI demo3 restart: three kill points (publish after push, integration before readback, completion after preview) survived control + Agency + packaged service restarts with one review request, one merge and one receipt each`，且用例结果为 `test project::demo3_gitea_chain_restarts_do_not_repeat_side_effects ... ok`。

### 2. 凭据隔离与执行加固怎么看

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`；常驻测试 Buck2 Build ID `927e8eb2-fc6d-41a1-86db-9f30c7de3cc6`）。

各家 harness 的执行环境读不到控制面凭据根、系统钥匙串与 `gh auth` 凭据，且写入工作树与状态根物理隔离，可通过以下两处查看：

1. **在当前状态根上查看执行目录边界与凭据文件权限**（实跑过）：
   ```bash
   # 查看控制面凭据存储目录与文件权限（位于 $ROOT/secrets，文件权限为 0600）
   ls -la "$ROOT/secrets"

   # 查看写入型调用的工作树落点（位于 $ROOT 之外的 /tmp/hctl2-exec-<20位hex>/write-worktrees/...）
   $CLI --json --root "$ROOT" changeset show "$REPO_ID" "$CHANGESET_ID" \
     | jq '.residuals[] | {status: .status, worktree_path: .worktree_path}'
   ```
   **怎么看**：控制面凭据保存在 `$ROOT/secrets` 下，而 Agency 为写入型调用物化的隔离工作树位于 `/tmp/hctl2-exec-.../write-worktrees/...`。Linux 下 Landlock（以及 macOS 下 Seatbelt）沙箱放行名单仅包含工作树与必要的运行时只读路径，不含控制面状态根 `$ROOT`、系统钥匙串目录（`~/.local/share/keyrings` 或 `~/Library/Keychains`）与 GitHub CLI 凭据目录（`~/.config/gh`）。

2. **查看凭据隔离与写入封存常驻测试**（实跑过，Build ID `927e8eb2-fc6d-41a1-86db-9f30c7de3cc6`）：
   ```bash
   cd "$REPO_ROOT/src"
   ./buck2 test root//agency:write_session_test -- --test-arg=--nocapture
   cd "$WS"
   ```
   **怎么看**：缺省运行确定性常驻用例（含各家只读/写入沙箱下读取控制面凭据根、系统钥匙串与 `gh auth` 均被拒绝的校验，以及 fixture 写入封存、越界拒绝与租约校验用例）；另有带 `#[ignore]` 的真 harness 写入用例，设置 `HCTL2_HARNESS_LIVE=1` 并传入 `-- --ignored` 时会拉起本机真实 harness 会话在租约工作树内修改非文档代码并运行测试。

### 3. 三条逃生口怎么看

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

1. **逃生口一（无契约卡只映射不签凭证）**（实跑过）：
   先从 `project show` 取出当前 Project 版本，创建一个不执行 `task adopt` 的裸 Task（bare card），直接尝试预览 `task complete`：
   ```bash
   PROJ_VER="$($CLI --json --root "$ROOT" project show "$PROJECT_ID" | jq -r .project.version)"

   cat << EOF > "$WS/work/bare-task.json"
   {
     "project_id": "$PROJECT_ID",
     "project_version": $PROJ_VER,
     "source_id": "$SOURCE_ID",
     "title": "Bare card without contract",
     "body": "Mapped from issue only, no contract adopted."
   }
   EOF

   PREV_BARE="$($CLI --json --root "$ROOT" task create --input "$WS/work/bare-task.json" --key bare-task-create)"
   TOK_BARE="$(echo "$PREV_BARE" | jq -r .preview_token)"
   SUB_BARE="$($CLI --json --root "$ROOT" task create --input "$WS/work/bare-task.json" --key bare-task-create --preview-token "$TOK_BARE")"
   export BARE_TASK_ID="$(echo "$SUB_BARE" | jq -r .task_id)"
   export BARE_TASK_VER="$(echo "$SUB_BARE" | jq -r .task.version)"

   cat << EOF > "$WS/work/bare-complete.json"
   {
     "project_id": "$PROJECT_ID",
     "task_id": "$BARE_TASK_ID",
     "version": $BARE_TASK_VER,
     "lifecycle_version": 1,
     "revision_number": 1,
     "acceptance": []
   }
   EOF

   $CLI --json --root "$ROOT" task complete --input "$WS/work/bare-complete.json" --key bare-task-complete || true
   ```
   **怎么看**：命令返回非零退出码并输出 `{"error":{"code":"CONTRACT_REQUIRED","message":"adopt a contract before completing this Task","recovery_action":"adopt_contract"}}`，拒绝签发 Completion Receipt，而 `$BARE_TASK_ID` 保持 `open` 且拥有完整的平台 issue 映射。

2. **逃生口二（采纳契约后完成有 Receipt）**（实跑过）：
   见第二节第 5、9 步与第四节第 1、5 步：对已创建或认领的 Task 执行 `task adopt` 采纳契约后，即可凭 `Integration Receipt` 通过 `task complete` 签出 `Task Completion Receipt`。

3. **逃生口三（无 Run 路径有可回溯对象）**（实跑过）：
   见第二节第 6–10 步与第四节第 2–5 步：全程未创建 Run，但从 `invocation show`、`changeset show`、`review show`、`integration show` 到 `task show` 每一步都有带精确摘要和版本的可回溯治理对象与两张 Receipt。

---

## 六、清理环境

**实跑状态**：实跑过（提交 `0566e83`，机器 Ubuntu `Linux 7.0.0-34-generic x86_64`，日志 `.memo/log/2026-10-19-demo3-manual-run.md`）。

验证完成后，先在停止控制面之前从 `changeset show` 取出本次 Agency 实例专属的 `/tmp/hctl2-exec-<20位hex>` 执行根目录，再停止控制面、Agency、Herdr 与随包服务，并仅清理本次走查的工作区 `${WS:?}` 与专属执行目录 `$EXEC_ROOT`（绝不通配删除 `/tmp/hctl2-exec-*`，以免误删同机其他正在运行的 Agency 实例的工作树）：

```bash
EXEC_ROOT="$($CLI --json --root "$ROOT" changeset show "$REPO_ID" "$CHANGESET_ID" 2>/dev/null | jq -r '.residuals[0].worktree_path // empty | capture("^(?<root>/tmp/hctl2-exec-[0-9a-f]{20})").root // empty')"
$CLI --root "$ROOT" stop
"$WS/prefix/bin/agency" --root "$ROOT/agency" stop || true
HCTL2_STATE_ROOT="$ROOT/services" "$INSTALL_ROOT/bin/hctl2-services" stop || true
pkill -f "$WS/prefix/.*herdr server" || true
case "$EXEC_ROOT" in
  /tmp/hctl2-exec-[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f])
    rm -rf "$EXEC_ROOT"
    ;;
esac
rm -rf "${WS:?}"
```

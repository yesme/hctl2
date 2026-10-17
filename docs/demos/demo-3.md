# 演示 3「真实交付」操作手册

本文记录演示 3「真实交付」从零构建、离线安装到端到端走查的完整命令序列。流程覆盖：本地 Gitea 仓库注册与 Claude Code 真会话写入交付链（A 链，人类仅在合入与完成两处预览确认）、受保护 GitHub `main` 与 Codex 真会话交付链（B 链）、开启「发布评审须人显式确认」开关的三次预览链（C 链），以及重启一致性、凭据隔离与逃生口的核对方法。

本手册依据走查清单与本地 Ubuntu 实跑记录整理，所有命令均显式指定独立状态根 `--root "$ROOT"`，不触碰用户默认状态目录 `~/.local/state/hctl2`。

---

## 一、环境构建与离线安装

### 1. 构建完整发布包

在代码仓库根目录下进入 `src/`，构建当前平台的完整离线运行包：

```bash
export REPO_ROOT="$(pwd)"
cd "$REPO_ROOT/src"
./buck2 build --config hctl2.zstd_preset=fast root//packaging/release:complete
cd "$REPO_ROOT"
```

构建完成后，运行安装包位于 `buck-out/v2/art/root/<hash>/packaging/release/__complete__/out/hctl2-0.0.0-<target>.tar.zst`。

### 2. 离线安装与工作区变量设置

新建独立演示工作目录（建议使用短路径如 `/tmp/d3`，避免 Unix domain socket 路径超出系统 `SUN_LEN` 限制），解压并运行离线安装器 `./install.sh`：

```bash
export WS="${WS:-/tmp/d3}"
mkdir -p "$WS/pkg" "$WS/prefix" "$WS/state" "$WS/work"

ARCHIVE="$(find "$REPO_ROOT/buck-out" -path '*packaging/release/__complete__*' -name 'hctl2-0.0.0-*.tar.zst' ! -name '*-sources.tar.zst' | head -n 1)"
[ -f "$ARCHIVE" ] || { echo "未找到完整运行包: $ARCHIVE"; return 1 2>/dev/null || exit 1; }

tar --zstd -xf "$ARCHIVE" -C "$WS/pkg"
PKG_DIR="$(find "$WS/pkg" -mindepth 1 -maxdepth 1 -type d -name 'hctl2-0.0.0-*' | head -n 1)"
"$PKG_DIR/install.sh" --prefix "$WS/prefix"

export PATH="$WS/prefix/bin:$PATH"
export INSTALL_ROOT="$(find "$WS/prefix/lib/hctl2" -mindepth 1 -maxdepth 1 -type d -name 'hctl2-0.0.0-*' | head -n 1)"
export HCTL2_INSTALL_ROOT="$INSTALL_ROOT"
export CLI="$WS/prefix/bin/hctl2"
export ROOT="$WS/state"

# 校验安装产物
[ -x "$CLI" ] || { echo "CLI 未就绪: $CLI"; return 1 2>/dev/null || exit 1; }
[ -x "$WS/prefix/bin/agency" ] || { echo "agency 未就绪"; return 1 2>/dev/null || exit 1; }
[ -x "$WS/prefix/bin/hctl2-tool" ] || { echo "hctl2-tool 未就绪"; return 1 2>/dev/null || exit 1; }
```

> [!IMPORTANT]
> - 全程所有控制面命令均须携带 `--root "$ROOT"`，确保状态与托管服务全部落在独立目录 `$WS/state` 下，不污染 `~/.local/state/hctl2`。
> - 执行 A 链前请确认本机已安装并登录 `claude`（Claude Code >= 2.1.263）；执行 B 链前请确认本机已安装并登录 `codex`（Codex CLI >= 0.153.4）且 `gh auth status` 已登录具备 `yesme/hctl2-canary` 权限的账号。

### 3. 启动控制面并核对初始状态

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
- 人读输出 `$CLI --root "$ROOT" status` 十秒内直接给出控制面就绪状态、密钥存储策略与各托管服务状态。
- JSON 输出中 `ready` 为 `true`，`policy.credential_storage` 为 `"user-file"`，`services.hosted` 中 `tuwunel` 的 `ready: true`、`gitea` 的 `consumed: false`（首次注册本地仓库时按需拉起）。

---

## 二、A 链：本地 Gitea + 真 Claude Code 写入交付（两次人工预览）

### 1. 初始化带 `.gitignore` 的本地试验仓库

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
> 首次使用本地 Gitea 时，控制面会自动消费并拉起随包 Gitea 服务。若 Gitea 尚在初始化导致首次提交返回 `PLATFORM_NOT_READY`（`recovery_action: "retry_registration"`），等待数秒后按恢复路径执行 `repo register --resume "$REPO_ID" --key a-register-resume` 预览并提交即可。

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

将控制面与随包原生 Agency 配对，并从名册中提取 `claude-code` 工种引用并接受：

```bash
PAIR_OUT="$($CLI --json --root "$ROOT" agency pair --binding-id native --key a-pair)"
echo "$PAIR_OUT" | jq '{paired: .paired, professions: [.binding.data.value.catalog.professions[].reference]}'

CATALOG_OUT="$($CLI --json --root "$ROOT" agency catalog native)"
echo "$CATALOG_OUT" | jq -c '.professions[] | select(.reference.id=="claude-code")' > "$WS/work/claude-profession.json"
jq -c '.reference' "$WS/work/claude-profession.json" > "$WS/work/claude-ref.json"

ACCEPT_CLAUDE="$($CLI --json --root "$ROOT" agency accept --binding-id native --reference "$WS/work/claude-ref.json" --key a-accept-claude)"
export CLAUDE_PROF_KEY="$(echo "$ACCEPT_CLAUDE" | jq -c .profession.key)"
export CLAUDE_PROF_VER="$(echo "$ACCEPT_CLAUDE" | jq -r .profession.version)"
export CLAUDE_PROF_DIGEST="$(jq -r '.reference.digest' "$WS/work/claude-ref.json")"
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

将 `claude-code` 写入参与者选入主 Room 名册（两步确认）：

```bash
cat << EOF > "$WS/work/a-select.json"
{
  "project_id": "$PROJECT_ID",
  "project_version": 1,
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

连接并挂接随包 Gitea 的 issues 任务源（各两步确认）：

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

cat << EOF > "$WS/work/a-attach.json"
{
  "project_id": "$PROJECT_ID",
  "project_version": 1,
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
cat << EOF > "$WS/work/a-task.json"
{
  "project_id": "$PROJECT_ID",
  "project_version": 1,
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

为该 Task 采纳契约（含 1 项要求 `Integration Receipt` 的机械验收项与 1 项人工验收项，两步确认）：

```bash
cat << EOF > "$WS/work/a-adopt.json"
{
  "project_id": "$PROJECT_ID",
  "project_version": 1,
  "task_id": "$TASK_ID",
  "version": $TASK_VER,
  "adoption": {
    "contract": {
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
    },
    "origin": {
      "kind": "local",
      "reference": {
        "key": {
          "scope": { "kind": "project", "id": "$PROJECT_ID" },
          "kind": "project",
          "id": "$PROJECT_ID"
        },
        "version": { "state": 1 }
      },
      "proposal_digest": "8544bbfeba73843dd82cfb34ea22fb3aaf98791a509474b02e8b2916f44572b7"
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

确认启动写入型调用，并轮询等待真 Claude Code 会话完成修改与 Agency 封存交回：

```bash
SUB_INV="$($CLI --json --root "$ROOT" invocation start --input "$WS/work/a-invoke.json" --key a-invoke --preview-token "$TOK_INV")"
export INVOCATION_ID="$(echo "$SUB_INV" | jq -r .invocation_id)"

for i in $(seq 1 180); do
  INV_SHOW="$($CLI --json --root "$ROOT" invocation show "$PROJECT_ID" "$INVOCATION_ID")"
  STATE="$(echo "$INV_SHOW" | jq -r .state)"
  case "$STATE" in
    completed)
      echo "Invocation completed"
      break
      ;;
    failed|cancelled|lost)
      echo "Invocation ended in $STATE: $(echo "$INV_SHOW" | jq -r .reason)"
      break
      ;;
  esac
  sleep 2
done
```

**核对字段**：
- `state`: `"completed"`。
- `reason`: `"Git version admitted; not Task acceptance"`。

### 7. 核对封存版本、自动发布与 Room 安静投影

查看准入的 ChangeSet Revision 与精确代码 diff：

```bash
CS_SHOW="$($CLI --json --root "$ROOT" changeset show "$REPO_ID" "$CHANGESET_ID")"
export REVISION_ID="$(echo "$CS_SHOW" | jq -r '.revisions[0].change_set_revision_id')"
echo "$CS_SHOW" | jq '.revisions[0]'

$CLI --json --root "$ROOT" changeset diff "$REPO_ID" "$REVISION_ID" | jq -r .diff
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
- 全程仅有唯一一条消息投影：`Write admitted as version csr-... of ChangeSet cs-... Publishing for review is queued: push to hctl2/cs-... and open or update the review request, each read back.`，无内部 JSON 泄漏或多余刷屏。

### 8. 第一次人工预览：合入（Integration Receipt）

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
- 预览输出写明 `form: "accept_advance"`（本地 Gitea 不提供原子预期目标头合并，只能选接受目标前移）以及冻结的目标保护快照。
- 提交回读后：`intent.state: "succeeded"`，签出唯一一张 `Integration Receipt`（`receipt_id: "receipt-..."`，`evidence_level: "hctl2-tool"`，`git_readback_level: "unmediated"`）。

### 9. 第二次人工预览：完成 Task（Task Completion Receipt）

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

### 10. 核对平台真实状态与 `task show` 十秒回答

用随包 `tea` 独立回读本地 Gitea 平台上的 PR 与 `main` 分支头（直接从状态根读取控制面管理员 token，不在终端打印凭据）：

```bash
REPO_SHOW="$($CLI --json --root "$ROOT" repo show "$REPO_ID")"
FULL_NAME="$(echo "$REPO_SHOW" | jq -r .observed.full_name)"
GITEA_URL="$(echo "$REPO_SHOW" | jq -r .observed.instance)"
SECRET_FILE="$(find "$ROOT/secrets" -type f | head -n 1)"

GITEA_INSTANCE_URL="$GITEA_URL" GITEA_TOKEN="$(cat "$SECRET_FILE")" NO_COLOR=1 \
  "$INSTALL_ROOT/libexec/hctl2/tea" api -X GET "/api/v1/repos/$FULL_NAME/pulls?state=all" \
  | jq '[.[] | {number: .number, state: .state, merged: .merged, head_sha: .head.sha, merge_commit_sha: .merge_commit_sha}]'

GITEA_INSTANCE_URL="$GITEA_URL" GITEA_TOKEN="$(cat "$SECRET_FILE")" NO_COLOR=1 \
  "$INSTALL_ROOT/libexec/hctl2/tea" api -X GET "/api/v1/repos/$FULL_NAME/branches/main" \
  | jq '{name: .name, commit_id: .commit.id}'
```

**核对字段**：
- Gitea 上该 ChangeSet 对应的评审请求恰有 1 条，`merged: true`，`head_sha` 与 `review show` 的 `platform_commit_sha` 一致。
- `main` 分支的 `commit_id` 与 `Integration Receipt` 的 `target_head_after`（`$MERGED_COMMIT`）完全一致。

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

> [!NOTE]
> B 链完整可复制命令序列将在 `#431` 第二轮走查实录走通后补入本节；以下为完整步骤骨架。

### 1. 前置检查：核对 `yesme/hctl2-canary` 分支保护与本地克隆

- 确认本机 `gh auth status` 已登录，且 `codex --version` 满足最低版本要求。
- 核对公开沙盒仓库 `yesme/hctl2-canary` 的 `main` 分支保护快照（要求经 PR 合入、必需检查 `canary`、对管理员生效），并在 `$WS/canary-source` 准备本地克隆。

### 2. 注册并激活外部 GitHub Repo

- 以 `origin: "external"`、`platform: "github"`、`platform_path: "yesme/hctl2-canary"`、`platform_repo_id: "1407796416"`、`default_source: "github_issues"` 预览并提交 `repo register`（外部平台直接进入 `active`，不在本地 Gitea 建镜像仓）。

### 3. 接受 `codex-cli` 工种、创建写入型 Profile 并选入名册

- 从已配对的原生 Agency 目录中提取 `codex-cli` 工种引用并执行 `agency accept`。
- 创建对应 `codex-cli` 的写入型 Worker Profile，新建绑定该 GitHub Repo 的 Project，并将 `codex-cli` 选入主 Room 名册。

### 4. 连接 `github_issues` 任务源、创建 Task 并采纳契约

- 连接 `github_issues` 任务源并挂接至该 Project（`approved_scope: "1407796416"`）。
- 创建要求在独立子目录下编写非文档代码与单元测试的 Task，确认 issue 在 `yesme/hctl2-canary` 上建出且绑定生效，随后执行 `task adopt` 采纳契约。

### 5. 发起写入型调用（真 Codex 会话改代码并封存）

- 执行 `invocation preview` 与 `invocation start`，由 Agency 经 app-server `turn/start` 与 Herdr pane 中的 `codex resume --remote` 在隔离工作树中完成代码编写与测试，并由 `hctl2-tool` 封存交回。

### 6. 自动发布 PR、等待 `canary` 检查通过并预览合入

- 核对 `review show` 中分支推送与 GitHub PR 创建两段各自确认。
- 等待 GitHub Actions 必需检查 `canary` 通过后，执行 `integration preview`（核对冻结的保护快照且只允许 `accept_advance`）与 `integration submit`，回读合并提交并签出 `Integration Receipt`。

### 7. 完成 Task 与平台事实核对

- 引用该 `Integration Receipt` 执行 `task complete` 预览与确认，签出 `Task Completion Receipt`。
- 核对 `task show` 中 `harness` 行显示 `codex-cli (via herdr, model none)`，并与 GitHub 远端 PR 状态及 `main` 头比对一致。

---

## 四、C 链：开启「须人显式确认」开关的三次预览链

在已激活的本地 Gitea Repo（`$REPO_ID`）上新建一个开启 `publish_review_requires_confirmation: true` 的独立 Project，验证发布意图停在 `pending_human`，需经人类显式执行 `review publish` 预览与确认后才推送并创建评审请求。

### 1. 创建开启发布确认开关的 Project 并选入写入参与者

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

cat << EOF > "$WS/work/c-select.json"
{
  "project_id": "$C_PROJECT_ID",
  "project_version": 1,
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
```

### 2. 发起写入型调用并观察发布意图停在 `pending_human`

```bash
C_DEADLINE_MS="$(( ($(date +%s) + 900) * 1000 ))"

cat << EOF > "$WS/work/c-invoke.json"
{
  "project_id": "$C_PROJECT_ID",
  "room_id": "$C_ROOM_ID",
  "target": "research",
  "profile": $WRITE_PROFILE_REV,
  "request": "Add a trailing comment line '# demo3 C' to calculator.py and leave it uncommitted. Do not run tests. Do not push or obtain credentials.",
  "budget": 262144,
  "deadline_ms": $C_DEADLINE_MS,
  "retry_of": null,
  "write": {
    "change_set_id": null,
    "baseline_commit": "$BASELINE_SHA",
    "target_branch": "main",
    "allow_update": true
  }
}
EOF

PREV_CINV="$($CLI --json --root "$ROOT" invocation preview --input "$WS/work/c-invoke.json" --key c-invoke)"
TOK_CINV="$(echo "$PREV_CINV" | jq -r .preview_token)"
export C_CHANGESET_ID="$(echo "$PREV_CINV" | jq -r .effect_summary.preview.write.lease.pending.change_set_id)"

SUB_CINV="$($CLI --json --root "$ROOT" invocation start --input "$WS/work/c-invoke.json" --key c-invoke --preview-token "$TOK_CINV")"
export C_INVOCATION_ID="$(echo "$SUB_CINV" | jq -r .invocation_id)"

for i in $(seq 1 180); do
  C_SHOW="$($CLI --json --root "$ROOT" invocation show "$C_PROJECT_ID" "$C_INVOCATION_ID")"
  C_STATE="$(echo "$C_SHOW" | jq -r .state)"
  case "$C_STATE" in
    completed|failed|cancelled|lost) break ;;
  esac
  sleep 2
done

C_REV_LIST="$($CLI --json --root "$ROOT" review list "$REPO_ID")"
export C_PUBLISH_INTENT_ID="$(echo "$C_REV_LIST" | jq -r --arg cs "$C_CHANGESET_ID" '.items[] | select(.change_set_id==$cs) | .intent_id')"
echo "$C_REV_LIST" | jq --arg cs "$C_CHANGESET_ID" '.items[] | select(.change_set_id==$cs) | {intent_id: .intent_id, state: .state, requires_human_confirmation: .policy.policy.requires_human_confirmation}'
```

**核对字段**：
- 派工预览中 `publication_target.requires_human_confirmation` 为 `true`。
- 调用完成后，`review list` 中该 ChangeSet 的发布意图状态停在 `state: "pending_human"`，`push.dispatched: false`，尚未向平台推送或建 PR。

### 3. 人工显式预览并确认发布（`review publish`）

```bash
# 查看人读发布预览（明确写出推到哪个分支、建到哪个目标、不是合入）
$CLI --root "$ROOT" review publish --repo-id "$REPO_ID" --intent-id "$C_PUBLISH_INTENT_ID" --key c-publish

PREV_CPUB="$($CLI --json --root "$ROOT" review publish --repo-id "$REPO_ID" --intent-id "$C_PUBLISH_INTENT_ID" --key c-publish)"
TOK_CPUB="$(echo "$PREV_CPUB" | jq -r .preview_token)"
echo "$PREV_CPUB" | jq '.effect_summary | {branch: .branch, target_branch: .target_branch, authorizes: .authorizes}'

$CLI --json --root "$ROOT" review publish --repo-id "$REPO_ID" --intent-id "$C_PUBLISH_INTENT_ID" --key c-publish --preview-token "$TOK_CPUB"

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
- 提交确认后，意图状态转为 `"published"`，`push` 与 `review_request` 两段均 `confirmed: true`。连同后续的合入预览与完成预览，开启开关的 Project 全程共需三次人工预览。

---

## 五、重启恢复、凭据隔离与逃生口怎么看

### 1. 重启恢复与不重复外部副作用怎么看

生产构建的二进制不开启测试注入缝（`test_seams` 在生产构建中恒为 `false`），演示时不通过手动猜测时机强杀进程，而是从以下两处核对重启一致性与外部副作用幂等：

1. **在当前状态根上查看正常启停后的记录与平台幂等**：
   正常停止并重启控制面与托管服务后，回读 A 链与 C 链的各项记录：
   ```bash
   $CLI --root "$ROOT" stop
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

2. **查看三处精确杀点的常驻回归用例**：
   「发布第一段推送后未建请求」「集成已提交未回读」「完成已预览未提交」三处精确状态的进程中断恢复，由 `room-cli-test` 中的常驻用例通过测试缝精确卡位并各自重启一次 `control` + `Agency` + 随包服务进行验证：
   ```bash
   cd "$REPO_ROOT/src"
   ./buck2 test root//packaging/release:room-cli-test -- \
     --test-arg=demo3_gitea_chain_restarts_do_not_repeat_side_effects \
     --test-arg=--exact --test-arg=--nocapture
   cd "$WS"
   ```
   **怎么看**：输出中打印 `LIVE CLI demo3 restart: three kill points (publish after push, integration before readback, completion after preview) survived control + Agency + packaged service restarts with one review request, one merge and one receipt each`，且用例结果为 `ok`。

### 2. 凭据隔离与执行加固怎么看

两家 harness 的执行环境读不到控制面凭据根、系统钥匙串与 `gh auth` 凭据，且写入工作树与状态根物理隔离，可通过以下两处查看：

1. **在当前状态根上查看执行目录边界与凭据文件权限**：
   ```bash
   # 查看控制面凭据存储目录与文件权限（位于 $ROOT/secrets，权限为 0600）
   ls -la "$ROOT/secrets"

   # 查看写入型调用的工作树落点（位于 $ROOT 之外的 /tmp/hctl2-exec-*/write-worktrees/...）
   $CLI --json --root "$ROOT" changeset show "$REPO_ID" "$CHANGESET_ID" \
     | jq '.residuals[] | {status: .status, worktree_path: .worktree_path}'
   ```
   **怎么看**：控制面凭据保存在 `$ROOT/secrets` 下，而 Agency 为写入型调用物化的隔离工作树位于 `/tmp/hctl2-exec-.../write-worktrees/...`。Linux 下 Landlock（以及 macOS 下 Seatbelt）沙箱放行名单仅包含工作树与必要的运行时只读路径，不含控制面状态根 `$ROOT`、系统钥匙串目录（`~/.local/share/keyrings` 或 `~/Library/Keychains`）与 GitHub CLI 凭据目录（`~/.config/gh`）。

2. **查看双家 harness 的凭据与加固常驻测试**：
   ```bash
   cd "$REPO_ROOT/src"
   ./buck2 test root//agency:write_session_test -- --test-arg=--nocapture
   cd "$WS"
   ```
   **怎么看**：测试逐项验证在受限执行环境中读取控制面凭据根、钥匙串与 `gh auth` 均被拒绝，同时写入型会话在租约工作树内可正常修改代码与执行测试。

### 3. 三条逃生口怎么看

1. **逃生口一（无契约卡只映射不签凭证）**：
   创建一个不执行 `task adopt` 的裸 Task（bare card），直接尝试预览 `task complete`：
   ```bash
   cat << EOF > "$WS/work/bare-task.json"
   {
     "project_id": "$PROJECT_ID",
     "project_version": 1,
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
   **怎么看**：命令返回非零退出码并拒绝签发 Completion Receipt（提示须先采纳契约），`$BARE_TASK_ID` 保持 `open` 且拥有完整的平台 issue 映射。

2. **逃生口二（采纳契约后完成有 Receipt）**：
   见第二节第 5、9 步：对已创建或认领的 Task 执行 `task adopt` 采纳契约后，即可凭 `Integration Receipt` 通过 `task complete` 签出 `Task Completion Receipt`。

3. **逃生口三（无 Run 路径有可回溯对象）**：
   见第二节第 6–10 步：A 链全程未创建 Run，但从 `invocation show`、`changeset show`、`review show`、`integration show` 到 `task show` 每一步都有带精确摘要和版本的可回溯治理对象与两张 Receipt。

---

## 六、清理环境

验证完成后停止控制面与随包服务，并清理临时工作区：

```bash
$CLI --root "$ROOT" stop
HCTL2_STATE_ROOT="$ROOT/services" "$INSTALL_ROOT/bin/hctl2-services" stop || true
rm -rf "$WS"
```

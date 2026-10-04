# 演示 1「协作现场」操作手册

本文记录演示 1「协作现场」的完整操作流程。流程包括环境构建、依赖解包、服务启动、人类账号注册、四步核心业务操作（建项目、进聊天室、开 Topic、认领平台 issue 为 Task），以及停机重启后的数据完整性核对。

本手册记录本地实跑验证的操作全流程，所有命令与核对字段均已按仓库实际布局（产物在仓库根 `buck-out/`）与步骤核对验证。

---

## 一、环境构建与变量设置

### 1. 构建依赖包与可执行文件

在代码仓库根目录下进入 `src/`，构建外部依赖整合包、控制面守护进程与命令行工具：

```bash
cd src
./buck2 build root//packaging/dependencies:package \
  root//apps/cli:hctl2 \
  root//apps/control:hctl2-control \
  --config hctl2.zstd_preset=fast
cd ..
```

构建完成后产物位置：
- 离线安装包：`buck-out/v2/art/root/<hash>/packaging/dependencies/__package__/out/dependency-packages/hctl2-0.0.0-<target>.tar.zst`
- CLI：`buck-out/v2/art/root/<hash>/apps/cli/__hctl2__/hctl2`
- 控制面守护进程：`buck-out/v2/art/root/<hash>/apps/control/__hctl2-control__/hctl2_control`

### 2. 解包安装与设置环境变量

新建测试工作目录（建议使用短路径如 `/tmp/d1` 或在仓库根目录下建 `work`），并在其下解压依赖包 payload：

```bash
# REPO_ROOT 指向代码仓库根目录（产物位于 $REPO_ROOT/buck-out）
export REPO_ROOT="$(pwd)"

# WS 为演示工作区根目录。为避免 SUN_LEN 超长及在仓库内产生未跟踪文件，建议使用短路径（如 /tmp/d1 或当前目录）
export WS="$(pwd)"
cd "$WS"
mkdir -p work/pkg work/root

# 解压依赖包（排除 -sources 伴随包）
tar --zstd -xf $(find "$REPO_ROOT/buck-out" -name "hctl2-0.0.0-*.tar.zst" ! -name '*-sources.tar.zst' | head -n 1) -C work/pkg

export INSTALL_ROOT="$(find "$WS/work/pkg" -name payload -type d | head -n 1)"
export HCTL2_INSTALL_ROOT="$INSTALL_ROOT"
export HCTL2_CONTROL_BIN="$(find "$REPO_ROOT/buck-out" -name hctl2_control -type f | head -n 1)"
export CLI="$(find "$REPO_ROOT/buck-out" -name hctl2 -type f -perm +111 | head -n 1)"
export ROOT="$WS/work/root"

# 显式校验关键产物，避免因路径为空导致后续步骤静默失败
[ -d "$INSTALL_ROOT" ] || { echo "INSTALL_ROOT 未找到: $INSTALL_ROOT"; return 1; }
[ -x "$HCTL2_CONTROL_BIN" ] || { echo "HCTL2_CONTROL_BIN 未找到或不可执行: $HCTL2_CONTROL_BIN"; return 1; }
[ -x "$CLI" ] || { echo "CLI 未找到或不可执行: $CLI"; return 1; }
```

> [!IMPORTANT]
> macOS 上 Unix domain socket 路径长度受限于系统 `SUN_LEN`（约 104 字节）。`$ROOT` 的绝对路径必须保持短小，否则控制面启动或连接时会报错 `path must be shorter than SUN_LEN`。

---

## 二、起服务与注册人类账号

### 1. 启动控制面与后台服务

> [!TIP]
> 随包服务端口（Tuwunel 6167 / Cinny 6168 / Gitea 3001）为固定端口。若同机已有正在运行的实例占用端口，`start` 虽返回 0 且 `ready: true`，但被占用的服务其 `available` 与 `running` 将为 `false`（静默失败）。启动前可先确认端口未被占用：
> ```bash
> lsof -i :6167 -i :6168 -i :3001
> ```

通过 CLI 启动守护进程：

```bash
$CLI --root "$ROOT" start
```

检查控制面状态：

```bash
$CLI --json --root "$ROOT" status
```

**核对字段**：
- `ready`: 应为 `true`。
- `services.hosted`: 列表中 `tuwunel` 的 `available: true`、`ready: true`、`running: true`。此时 `gitea` 的 `consumed` 应为 `false`（首次使用时才拉起）。

### 2. 启动 Cinny 客户端【目前要手工做】

`hctl2 start` 默认拉起 Tuwunel 聊天服务，但不会自动拉起 Cinny Web 服务。需单独拉起：

```bash
HCTL2_STATE_ROOT="$ROOT/services" "$INSTALL_ROOT/bin/hctl2-services" start cinny
```

检查 Cinny 服务：

```bash
curl -sI http://127.0.0.1:6168 | head -n 1
```

**核对字段**：HTTP 状态码为 `200 OK`。

### 3. 注册人类账号

获取 Tuwunel 注册令牌：

```bash
cat "$ROOT/services/config/tuwunel-registration-token"
```

人类账号注册有两种方式：

- **方式 A（浏览器界面）**：浏览器打开 `http://127.0.0.1:6168/`，进入 Cinny 注册页面，输入用户名（如 `yesme`）、密码与上述注册令牌完成注册。首个注册用户自动成为 Tuwunel 服务器管理员。
- **方式 B（Matrix API 命令行）**：

通过 Matrix 用户交互认证（UIA）接口注册：

```bash
REG_TOKEN="$(cat "$ROOT/services/config/tuwunel-registration-token")"
SESSION="$(curl -s -X POST http://127.0.0.1:6167/_matrix/client/v3/register \
  -H "Content-Type: application/json" \
  -d '{"username":"yesme","password":"password123"}' | jq -r .session)"

REGISTER_RESP="$(curl -s -X POST http://127.0.0.1:6167/_matrix/client/v3/register \
  -H "Content-Type: application/json" \
  -d "{\"auth\":{\"type\":\"m.login.registration_token\",\"token\":\"$REG_TOKEN\",\"session\":\"$SESSION\"},\"username\":\"yesme\",\"password\":\"password123\"}")"

export USER_TOKEN="$(echo "$REGISTER_RESP" | jq -r .access_token)"
```

**核对字段**：返回 JSON 中包含 `user_id` 为 `@yesme:hctl2.localhost` 与非空 `access_token`。

---

## 三、步骤 1：用命令建 Project

### 1. 本地初始化 Git 仓库 `apollo`

```bash
mkdir -p work/apollo && cd work/apollo
git init
git config user.name "Yesme"
git config user.email "you@example.com"  # 填入你自己的邮箱
git commit --allow-empty -m "initial commit"
cd "$WS"
```

### 2. 登记 Repo（两步确认）

编写 `work/register.json`：

```bash
cat << EOF > work/register.json
{
  "name": "Apollo",
  "origin": "local",
  "platform_path": "apollo",
  "local": {
    "machine": "control",
    "path": "$WS/work/apollo"
  },
  "default_source": "gitea_issues"
}
EOF
```

预览登记：

```bash
PREVIEW_REG="$($CLI --json --root "$ROOT" repo register --input work/register.json --key register-apollo)"
TOKEN_REG="$(echo "$PREVIEW_REG" | jq -r .preview_token)"
```

**核对字段**：`preview_token` 为非空字符串。

提交登记：

```bash
SUBMIT_REG="$($CLI --json --root "$ROOT" repo register --input work/register.json --key register-apollo --preview-token "$TOKEN_REG")"
export REPO_ID="$(echo "$SUBMIT_REG" | jq -r .registration.repo_id)"
```

**核对字段**：
- 控制面首次使用 Gitea，自动拉起 Gitea 托管服务。
- `lifecycle`: 应为 `"pending"`。
- `version`: 应为 `3`。
- `observed.stable_id`: 应为 `"1"`。
- `repo_id`: 记下该 Repo ID。

### 3. 确认激活 Repo（两步确认）

预览确认：

```bash
PREVIEW_CONFIRM="$($CLI --json --root "$ROOT" repo register --confirm "$REPO_ID" --version 3 --platform-repo-id 1 --key confirm-apollo)"
TOKEN_CONFIRM="$(echo "$PREVIEW_CONFIRM" | jq -r .preview_token)"
```

**核对字段**：`preview_token` 为非空字符串。

提交确认：

```bash
$CLI --json --root "$ROOT" repo register --confirm "$REPO_ID" --version 3 --platform-repo-id 1 --key confirm-apollo --preview-token "$TOKEN_CONFIRM"
```

**核对字段**：
- `lifecycle`: 应为 `"active"`。
- `version`: 应为 `4`。

### 4. 创建 Project（两步确认）

编写 `work/project.json`：

```bash
cat << EOF > work/project.json
{
  "repo_id": "$REPO_ID",
  "definition": {
    "name": "Apollo",
    "goal": "交付目标",
    "scope": "工作范围",
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
```

预览创建：

```bash
PREVIEW_PROJ="$($CLI --json --root "$ROOT" project create --input work/project.json --key create-apollo)"
TOKEN_PROJ="$(echo "$PREVIEW_PROJ" | jq -r .preview_token)"
```

**核对字段**：`preview_token` 为非空字符串。

提交创建：

```bash
SUBMIT_PROJ="$($CLI --json --root "$ROOT" project create --input work/project.json --key create-apollo --preview-token "$TOKEN_PROJ")"
export PROJECT_ID="$(echo "$SUBMIT_PROJ" | jq -r .project_id)"
export MAIN_ROOM_ID="$(echo "$SUBMIT_PROJ" | jq -r .main_room_id)"
export MATRIX_MAIN_ROOM="$(echo "$SUBMIT_PROJ" | jq -r .receipt.matrix_room_id)"
```

**核对字段**：
- `delivery`: 应为 `"confirmed"`。
- `project_id`: 成功分配 `project-...`。
- `main_room_id`: 主 Room 编号 `main-...`。
- `receipt.matrix_room_id`: 在 Tuwunel 上创建的 Matrix 房间 ID `!...`。

**幂等性检查**：带相同 key 重新提交预览与确认，返回完全相同的 `project_id`。

---

## 四、步骤 2：在 Cinny 里看到主 Room

### 1. 邀请人类账号进入主 Room（两步确认）

先读取主 Room 的 Binding 信息：

```bash
ROOM_SHOW="$($CLI --json --root "$ROOT" room show "$PROJECT_ID" "$MAIN_ROOM_ID")"
BINDING_KEY="$(echo "$ROOM_SHOW" | jq -c .binding.key)"
BINDING_VERSION="$(echo "$ROOM_SHOW" | jq -r .binding.version)"
```

编写 `work/members.json`：

```bash
cat << EOF > work/members.json
{
  "project_id": "$PROJECT_ID",
  "project_version": 1,
  "rooms": [
    {
      "key": $BINDING_KEY,
      "version": {
        "state": $BINDING_VERSION
      }
    }
  ],
  "users": [
    "@yesme:hctl2.localhost"
  ],
  "invite": true
}
EOF
```

预览成员邀请：

```bash
PREVIEW_MEM="$($CLI --json --root "$ROOT" project members --input work/members.json --key invite-yesme)"
TOKEN_MEM="$(echo "$PREVIEW_MEM" | jq -r .preview_token)"
```

**核对字段**：`preview_token` 为非空字符串。

提交成员邀请：

```bash
$CLI --json --root "$ROOT" project members --input work/members.json --key invite-yesme --preview-token "$TOKEN_MEM"
```

**核对字段**：
- `delivery`: 应为 `"confirmed"`。
- `rooms[0].receipt.members`: 包含 `[{"membership":"invite","user_id":"@yesme:hctl2.localhost"}]`。

### 2. 人类账号接受邀请并发送两条消息

人类可以在 Cinny 界面中点击接受并打字，也可以通过 Matrix 客户端 API 模拟：

接受邀请：

```bash
curl -s -X POST -H "Authorization: Bearer $USER_TOKEN" \
  "http://127.0.0.1:6167/_matrix/client/v3/rooms/$MATRIX_MAIN_ROOM/join"
```

在主 Room 中发送两条消息：

```bash
EVT1_JSON="$(curl -s -X PUT -H "Authorization: Bearer $USER_TOKEN" -H "Content-Type: application/json" \
  -d '{"msgtype":"m.text","body":"第一条消息：讨论项目架构"}' \
  "http://127.0.0.1:6167/_matrix/client/v3/rooms/$MATRIX_MAIN_ROOM/send/m.room.message/m1")"

EVT2_JSON="$(curl -s -X PUT -H "Authorization: Bearer $USER_TOKEN" -H "Content-Type: application/json" \
  -d '{"msgtype":"m.text","body":"第二条消息：确定先做 README"}' \
  "http://127.0.0.1:6167/_matrix/client/v3/rooms/$MATRIX_MAIN_ROOM/send/m.room.message/m2")"

export EVT1="$(echo "$EVT1_JSON" | jq -r .event_id)"
export EVT2="$(echo "$EVT2_JSON" | jq -r .event_id)"
```

**核对字段**：`EVT1` 与 `EVT2` 均为形如 `$...` 的有效 Matrix 事件 ID。

---

## 五、步骤 3：开 Topic、带前情提要

### 1. 机械选材起草提要

编写 `work/draft.json`：

```bash
cat << EOF > work/draft.json
{
  "project_id": "$PROJECT_ID",
  "project_version": 1,
  "origin": {
    "kind": "room",
    "room_id": "$MAIN_ROOM_ID",
    "binding_version": 2
  },
  "selection": {
    "kind": "events",
    "event_ids": [
      "$EVT1",
      "$EVT2"
    ]
  }
}
EOF
```

执行起草命令：

```bash
DRAFT_OUT="$($CLI --json --root "$ROOT" room draft --input work/draft.json)"
```

**核对字段**：
- `automatic_summary`: 应为 `"not_configured"`。
- `rule_reference`: 应为 `"hctl2.brief.verbatim.v1"`。
- `fragments`: 包含两段逐字原文，`excerpt` 分别为 `"第一条消息：讨论项目架构"` 与 `"第二条消息：确定先做 README"`。

### 2. 创建 Topic Room（两步确认）

编写 `work/topic.json`（人类可以编辑正文各节，`brief.sources` 直接引用草稿中的 `fragments[].source`）：

```bash
SRC1="$(echo "$DRAFT_OUT" | jq -c '.fragments[0].source')"
SRC2="$(echo "$DRAFT_OUT" | jq -c '.fragments[1].source')"

cat << EOF > work/topic.json
{
  "project_id": "$PROJECT_ID",
  "project_version": 1,
  "name": "README 起草",
  "origin": {
    "kind": "room",
    "room_id": "$MAIN_ROOM_ID",
    "binding_version": 2
  },
  "brief": {
    "context_and_goal": "起草项目说明文档",
    "settled_facts_and_reasons": ["已确定先做 README"],
    "disagreements_and_questions": [],
    "constraints_and_materials": [],
    "sources": [
      $SRC1,
      $SRC2
    ]
  },
  "participants": [],
  "roster_confirmed": true
}
EOF
```

预览创建 Topic：

```bash
PREVIEW_TOPIC="$($CLI --json --root "$ROOT" room create-topic --input work/topic.json --key topic-readme)"
TOKEN_TOPIC="$(echo "$PREVIEW_TOPIC" | jq -r .preview_token)"
```

**核对字段**：`preview_token` 为非空字符串。

提交创建 Topic：

```bash
SUBMIT_TOPIC="$($CLI --json --root "$ROOT" room create-topic --input work/topic.json --key topic-readme --preview-token "$TOKEN_TOPIC")"
export TOPIC_ROOM_ID="$(echo "$SUBMIT_TOPIC" | jq -r .room_id)"
export MATRIX_TOPIC_ROOM="$(echo "$SUBMIT_TOPIC" | jq -r .receipt.matrix_room_id)"
```

**核对字段**：
- `state`: 应为 `"confirmed"`。
- `room_id`: 分配出 Topic Room ID `topic-...`。
- `receipt.matrix_room_id`: 在 Tuwunel 上对应创建的 Matrix 房间 ID。

### 3. 查看 Room 层级树

```bash
$CLI --json --root "$ROOT" room hierarchy "$PROJECT_ID" "$MAIN_ROOM_ID"
```

**核对字段**：
- `carrier_space_id`: 主 Room 的承载 Space ID。
- `children`: 包含刚建出的 `$TOPIC_ROOM_ID`（注意 `children` 为 Topic Room ID 字符串数组如 `["topic-..."]`，非对象数组）。

### 4. 查看 Topic 详情与成员邀请

```bash
$CLI --json --root "$ROOT" room show "$PROJECT_ID" "$TOPIC_ROOM_ID"
```

**核对字段**：
- `room.brief`: 包含 `material_id` 与 `byte_digest`，指向材料记录（顶层 `.brief` 则包含 5 节提要内容文本）。
- `hierarchy.parents`: 列表中包含父节点主 Room，`canonical: true`。
- 时间线只有建房与状态事件（共 11 个状态事件，无 `m.room.message` 业务消息）；`m.room.member` 只有一条且 `state_key` 为 `@hctl2_control`（人类尚未加入）。

将人类账号邀入 Topic Room：

```bash
TOPIC_BINDING_KEY="$($CLI --json --root "$ROOT" room show "$PROJECT_ID" "$TOPIC_ROOM_ID" | jq -c .binding.key)"
cat << EOF > work/members-topic.json
{
  "project_id": "$PROJECT_ID",
  "project_version": 1,
  "rooms": [
    {
      "key": $TOPIC_BINDING_KEY,
      "version": {
        "state": 2
      }
    }
  ],
  "users": [
    "@yesme:hctl2.localhost"
  ],
  "invite": true
}
EOF

PREVIEW_T_MEM="$($CLI --json --root "$ROOT" project members --input work/members-topic.json --key invite-yesme-topic)"
TOKEN_T_MEM="$(echo "$PREVIEW_T_MEM" | jq -r .preview_token)"
$CLI --json --root "$ROOT" project members --input work/members-topic.json --key invite-yesme-topic --preview-token "$TOKEN_T_MEM"
```

**核对字段**：`delivery: "confirmed"`。

人类账号接受 Topic Room 邀请（可在 Cinny 界面点击接受，或使用 Matrix 客户端 API）：

```bash
# 获取 Topic Room 对应的 Matrix 房间 ID
MATRIX_TOPIC_ROOM="$($CLI --json --root "$ROOT" room show "$PROJECT_ID" "$TOPIC_ROOM_ID" | jq -r .room.native_id)"

# 人类账号接受邀请并加入 Topic 房间
curl -s -X POST -H "Authorization: Bearer $USER_TOKEN" \
  "http://127.0.0.1:6167/_matrix/client/v3/rooms/$MATRIX_TOPIC_ROOM/join"
```

**核对字段**：返回 `{"room_id":"!..."}`。人类账号在 Cinny 中可见并已加入该 Topic Room。

---

## 六、步骤 4：平台上的 issue 认领成 Task

### 1. 创建 Gitea 人类账号并添加为协作者【目前要手工做】

控制面以自身生成的管理员账号创建私有仓库。目前 `hctl2` 尚无为人开通本地 Gitea 账号及协作者权限的命令（规划于小活 E 交付）。需手工执行：

#### (1) 创建人类账号

```bash
"$INSTALL_ROOT/libexec/hctl2/gitea" \
  --config "$ROOT/services/config/gitea/app.ini" \
  --work-path "$ROOT/services/data/gitea" \
  admin user create \
  --username yesme \
  --password password123 \
  --email yesme@hctl2.localhost \
  --admin=false \
  --must-change-password=false
```

> [!NOTE]
> 此处密码为本地演示环境设置的示例口令；在生产或严谨环境中敏感输入应通过环境变量或交互输入传递，避免明文暴露在命令行参数中。

> [!WARNING]
> 创建用户必须显式带上 `--must-change-password=false`。Gitea 默认要求非首个用户在首次登录时修改密码，若不带该标志，后续通过 API 开 issue 会被拦截并报错 `HTTP 403 Forbidden: You must change your password`。

#### (2) 获取 Gitea 管理员令牌

- **方式一（交互式桌面环境）**：从 macOS 钥匙串读取控制面存放的令牌（本方式假设密钥后端为系统钥匙串 `system-keyring`，即默认配置；若控制面启动时显式指定了 `--secret-backend user-file`，凭据存放于 `$ROOT/secrets` 下，钥匙串中无此项，请使用方式二；参见 [docs/usage.md](../usage.md#安装完整离线包)）：
  ```bash
  CONTROL_ID="$($CLI --root "$ROOT" status | jq -r .control_id)"
  ADMIN_TOKEN="$(security find-generic-password -s hctl2 -a "gitea:${CONTROL_ID}:admin" -w)"
  ```
- **方式二（无头/终端环境）**：为避免钥匙串权限弹窗在无屏幕会话中挂起，可直接通过 Gitea admin 命令生成管理令牌：
  ```bash
  CONTROL_ID="$($CLI --root "$ROOT" status | jq -r .control_id)"
  ADMIN_USER="hctl-${CONTROL_ID:0:16}"
  ADMIN_TOKEN="$("$INSTALL_ROOT/libexec/hctl2/gitea" \
    --config "$ROOT/services/config/gitea/app.ini" \
    --work-path "$ROOT/services/data/gitea" \
    admin user generate-access-token \
    --username "$ADMIN_USER" \
    --token-name "manual-admin" \
    --raw)"
  ```

#### (3) 将人类账号加为仓库协作者

```bash
curl -s -o /dev/null -w "%{http_code}\n" -X PUT \
  -H "Authorization: token $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"permission":"write"}' \
  "http://127.0.0.1:3001/api/v1/repos/$ADMIN_USER/apollo/collaborators/yesme"
```

**核对字段**：HTTP 状态码返回 `204`。

### 2. 人类以自身账号在 Gitea 开 issue #1

通过 Gitea API 提交 issue：

```bash
curl -s -X POST -u yesme:password123 \
  "http://127.0.0.1:3001/api/v1/repos/$ADMIN_USER/apollo/issues" \
  -H "Content-Type: application/json" \
  -d '{"title":"编写项目说明与规范","body":"请为 Apollo 项目撰写 README.md 和初始规划。"}' | jq '{number: .number, title: .title, state: .state}'
```

**核对字段**：
- `number`: 应为 `1`。
- `state`: 应为 `"open"`。
- `title`: 应为 `"编写项目说明与规范"`。

### 3. 连接任务源并认领 Task（各命令两步确认）

#### (1) 连接任务源

编写 `work/task-source.json`：

```bash
cat << EOF > work/task-source.json
{
  "repo_id": "$REPO_ID",
  "candidate_id": "gitea_issues",
  "consent": true,
  "make_default": true
}
EOF

PREVIEW_T_CONN="$($CLI --json --root "$ROOT" task connect --input work/task-source.json --key connect-apollo)"
TOKEN_T_CONN="$(echo "$PREVIEW_T_CONN" | jq -r .preview_token)"
SUBMIT_T_CONN="$($CLI --json --root "$ROOT" task connect --input work/task-source.json --key connect-apollo --preview-token "$TOKEN_T_CONN")"
export SOURCE_ID="$(echo "$SUBMIT_T_CONN" | jq -r .source_id)"
```

**核对字段**：`source_id` 为非空字符串。

#### (2) 将任务源挂接至 Project

编写 `work/task-attach.json`：

```bash
cat << EOF > work/task-attach.json
{
  "project_id": "$PROJECT_ID",
  "project_version": 1,
  "source_id": "$SOURCE_ID",
  "approved_scope": "1",
  "consent": true
}
EOF

PREVIEW_T_ATT="$($CLI --json --root "$ROOT" task attach --input work/task-attach.json --key attach-apollo)"
TOKEN_T_ATT="$(echo "$PREVIEW_T_ATT" | jq -r .preview_token)"
$CLI --json --root "$ROOT" task attach --input work/task-attach.json --key attach-apollo --preview-token "$TOKEN_T_ATT"
```

**核对字段**：返回 JSON 中包含 `project_id` 与 `source_id`。

#### (3) 刷新任务源

编写 `work/task-refresh.json`：

```bash
cat << EOF > work/task-refresh.json
{
  "repo_id": "$REPO_ID",
  "source_id": "$SOURCE_ID"
}
EOF

PREVIEW_T_REF="$($CLI --json --root "$ROOT" task refresh --input work/task-refresh.json --key refresh-apollo)"
TOKEN_T_REF="$(echo "$PREVIEW_T_REF" | jq -r .preview_token)"
$CLI --json --root "$ROOT" task refresh --input work/task-refresh.json --key refresh-apollo --preview-token "$TOKEN_T_REF"
```

**核对字段**：返回成功回执。

#### (4) 查看看板

```bash
$CLI --json --root "$ROOT" task board "$PROJECT_ID" "$SOURCE_ID"
```

**核对字段**：
- `cards[0].card.number`: 应为 `1`。
- `cards[0].card.title`: 应为 `"编写项目说明与规范"`。
- `cards[0].claimed`: 应为 `false`。

#### (5) 认领 Task

编写 `work/task-claim.json`：

```bash
cat << EOF > work/task-claim.json
{
  "project_id": "$PROJECT_ID",
  "project_version": 1,
  "source_id": "$SOURCE_ID",
  "entity_id": "1"
}
EOF

PREVIEW_T_CLAIM="$($CLI --json --root "$ROOT" task claim --input work/task-claim.json --key claim-apollo-1)"
TOKEN_T_CLAIM="$(echo "$PREVIEW_T_CLAIM" | jq -r .preview_token)"
SUBMIT_T_CLAIM="$($CLI --json --root "$ROOT" task claim --input work/task-claim.json --key claim-apollo-1 --preview-token "$TOKEN_T_CLAIM")"
export TASK_ID="$(echo "$SUBMIT_T_CLAIM" | jq -r .task_id)"
```

**核对字段**：
- `task_id`: 成功分配（64 位十六进制裸摘要，无 `task-` 前缀，与 `repo_id`、`source_id` 同形）。
- `task.data.lifecycle`: 应为 `"open"`。

### 4. 查看 Task 详情

```bash
$CLI --json --root "$ROOT" task show "$PROJECT_ID" "$TASK_ID"
```

**核对字段**：
- `data.lifecycle`: 应为 `"open"`。
- `data.entity.immutable_external_entity_id`: 应为 `"1"`。
- `data.title`: 应为 `"编写项目说明与规范"`。

---

## 七、步骤 5：重启后核对

### 1. 保存未发送草稿

编写 `work/view-state.json`：

```bash
cat << EOF > work/view-state.json
{
  "project_id": "$PROJECT_ID",
  "room_id": "$MAIN_ROOM_ID",
  "binding_version": 2,
  "client_id": "cinny-web",
  "draft": "未发送草稿：准备开始开发",
  "read_cursor": null
}
EOF

$CLI --json --root "$ROOT" room save-view-state --input work/view-state.json
```

**核对字段**：`draft` 包含 `"未发送草稿：准备开始开发"`。

### 2. 存下重启前 10 个查询输出基线

```bash
mkdir -p work/pre
$CLI --json --root "$ROOT" project show "$PROJECT_ID" > work/pre/1_project_show.json
$CLI --json --root "$ROOT" project overview "$PROJECT_ID" > work/pre/2_project_overview.json
$CLI --json --root "$ROOT" project list > work/pre/3_project_list.json
$CLI --json --root "$ROOT" room show "$PROJECT_ID" "$MAIN_ROOM_ID" > work/pre/4_room_show_main.json
$CLI --json --root "$ROOT" room show "$PROJECT_ID" "$TOPIC_ROOM_ID" > work/pre/5_room_show_topic.json
$CLI --json --root "$ROOT" room hierarchy "$PROJECT_ID" "$MAIN_ROOM_ID" > work/pre/6_room_hierarchy.json
$CLI --json --root "$ROOT" room timeline "$PROJECT_ID" "$MAIN_ROOM_ID" > work/pre/7_room_timeline.json
$CLI --json --root "$ROOT" room view-state "$PROJECT_ID" "$MAIN_ROOM_ID" "cinny-web" > work/pre/8_room_view_state.json
$CLI --json --root "$ROOT" task show "$PROJECT_ID" "$TASK_ID" > work/pre/9_task_show.json
$CLI --json --root "$ROOT" repo show "$REPO_ID" > work/pre/10_repo_show.json
```

### 3. 停止服务与守护进程

```bash
kill -KILL $(cat "$ROOT/control.pid")
$CLI --root "$ROOT" status || echo "control 已断开"
HCTL2_STATE_ROOT="$ROOT/services" "$INSTALL_ROOT/bin/hctl2-services" stop
HCTL2_STATE_ROOT="$ROOT/services" "$INSTALL_ROOT/bin/hctl2-services" status || true
```

**核对字段**：
- `$CLI --root "$ROOT" status` 报告 `transport error`。
- `hctl2-services status` 报告 `HCTL2 services are not running.`。

### 4. 重新启动服务

```bash
$CLI --root "$ROOT" start
```

约 1 秒后检查状态：

```bash
$CLI --json --root "$ROOT" status
```

**核对字段**：
- `ready`: 应为 `true`。
- `services.hosted`: `tuwunel` 与 `gitea` 均自动恢复运行（`available: true`、`ready: true`、`running: true`）。

重新拉起 Cinny【目前要手工做】：

```bash
HCTL2_STATE_ROOT="$ROOT/services" "$INSTALL_ROOT/bin/hctl2-services" start cinny
```

### 5. 重启后完整核对

#### (1) 重新获取 10 项查询并逐项对比

```bash
mkdir -p work/post
$CLI --json --root "$ROOT" project show "$PROJECT_ID" > work/post/1_project_show.json
$CLI --json --root "$ROOT" project overview "$PROJECT_ID" > work/post/2_project_overview.json
$CLI --json --root "$ROOT" project list > work/post/3_project_list.json
$CLI --json --root "$ROOT" room show "$PROJECT_ID" "$MAIN_ROOM_ID" > work/post/4_room_show_main.json
$CLI --json --root "$ROOT" room show "$PROJECT_ID" "$TOPIC_ROOM_ID" > work/post/5_room_show_topic.json
$CLI --json --root "$ROOT" room hierarchy "$PROJECT_ID" "$MAIN_ROOM_ID" > work/post/6_room_hierarchy.json
$CLI --json --root "$ROOT" room timeline "$PROJECT_ID" "$MAIN_ROOM_ID" > work/post/7_room_timeline.json
$CLI --json --root "$ROOT" room view-state "$PROJECT_ID" "$MAIN_ROOM_ID" "cinny-web" > work/post/8_room_view_state.json
$CLI --json --root "$ROOT" task show "$PROJECT_ID" "$TASK_ID" > work/post/9_task_show.json
$CLI --json --root "$ROOT" repo show "$REPO_ID" > work/post/10_repo_show.json

for f in $(ls work/pre); do
  echo "--- diff $f ---"
  diff -u "work/pre/$f" "work/post/$f" || true
done
```

**核对结果**：
- 除 `7_room_timeline.json` 中的 `unsigned.age` 因时间流逝不同外，其余 9 个查询结果逐字节完全一致。

#### (2) 人类账号 `/sync` 回读

```bash
curl -s -H "Authorization: Bearer $USER_TOKEN" "http://127.0.0.1:6167/_matrix/client/v3/sync" | jq '.rooms.join | keys'
```

**核对字段**：主 Room 与 Topic Room 均在列表中（由于首个注册用户自动成为 Tuwunel 服务器管理员，列表中还包含 Tuwunel 的 Admin Room，共 3 个房间）。

#### (3) Gitea issue #1 回读

```bash
curl -s -u yesme:password123 "http://127.0.0.1:3001/api/v1/repos/$ADMIN_USER/apollo/issues/1" | jq '{number: .number, title: .title, state: .state}'
```

**核对字段**：`number: 1`，`state: "open"`，`title: "编写项目说明与规范"`。

#### (4) 重启后刷新任务源

```bash
PREVIEW_POST_REF="$($CLI --json --root "$ROOT" task refresh --input work/task-refresh.json --key refresh-apollo-after-restart)"
TOKEN_POST_REF="$(echo "$PREVIEW_POST_REF" | jq -r .preview_token)"
$CLI --json --root "$ROOT" task refresh --input work/task-refresh.json --key refresh-apollo-after-restart --preview-token "$TOKEN_POST_REF"
```

**核对字段**：与 Gitea 通信正常，刷新成功（退出码 0）。

---

## 八、清理环境

验证完成后停止服务并清理：

```bash
$CLI --root "$ROOT" stop
HCTL2_STATE_ROOT="$ROOT/services" "$INSTALL_ROOT/bin/hctl2-services" stop
rm -rf work
```

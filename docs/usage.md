# HCTL2 使用说明

本文说明当前代码树里每个 `hctl2-*` 入口的实际用途。HCTL2 仍处于早期实现阶段：现在可以运行 Chatroom、本地 Gitea、Kanban、Workflow、Terminal 五类打包依赖，用 `hctl2` / `hctl2-control` 起控制面与已消费服务，并用 `hctl2-tool` 做本地 Git 现场操作与闭集外部事实回读。Workbench 尚未实现。

## 当前入口一览

| 名称 | 当前状态 | 面向谁 | 现在能做什么 |
| --- | --- | --- | --- |
| `hctl2-services` | 可用 | 安装包用户、开发者 | 通过 Process Compose 启停并检查 Chatroom（Tuwunel + Cinny）、Gitea、Vikunja、Dagu 和 Herdr |
| `hctl2-tool` | P1 可用 | HCTL2 开发者、Harness、安装包用户 | 仓库检查、现场锁、worktree 物化与核验、封存保全拆除、本地集成，以及 `wait` 回读闭集外部事实 |
| `hctl2` | P2.1 可用 | 最终用户 | 公共 CLI：`init/start/stop/status/doctor/export/backup/restore`，经本地 Unix socket 与 control 说话 |
| `hctl2-control` | P2.1 可用 | HCTL2 内部组件 | 控制面守护进程；`hctl2 start` 拉起它，并按首次消费经 Process Compose 拉起随包服务：Tuwunel 随 start 拉起；Gitea 由 `hctl2 services consume gitea` 拉起并记为已消费，P2.2 戊在注册纯本地仓库（或显式选本地平台）时调用它 |
| `hctl2-workbench` | 尚未实现 | 最终用户 | 未来的图形客户端 |

`hctl2-tool` 不是后台服务，也不是治理命令入口。独立运行只提供普通本地操作：经宿主 `git` 读写本机仓库，并用 `wait` 回读闭集外部事实。它不产生 HCTL 治理记录，也不签发 Receipt 或 Verdict，也不做 push、PR、merge 等远端副作用——远端动作归控制面里的平台适配器，见[Repo 模块约束](./design/spec/repo.md)。Herdr 是随包提供的外部运行服务，不是 HCTL2 自建命令。

## 安装当前离线包

当前代码树为 Linux x86_64、macOS arm64 和 macOS x86_64 分别定义离线包；macOS 系统要求以[交付文档的打包策略](./design/delivery.md#打包策略选型判断首次消费时产品化)为准。

运行安装包内含固定版本的 Tuwunel、Cinny、Gitea、tea、Vikunja、Dagu、Herdr、Static Web Server、Process Compose、供 `hctl2-tool` 使用的 GitHub CLI、许可证、`hctl2`、`hctl2-control`、`hctl2-services` 与 `hctl2-tool`。锁定的上游源码位于同一 Release 中单独发布的源码伴随包。

安装过程不联网，也不在用户机器上编译，不依赖 Rust、Python、Node.js、Homebrew 或 Linux 构建工具。

每个 target 同时发布两份归档：

| 文件 | 用途 | 是否需要安装 |
| --- | --- | --- |
| `hctl2-0.0.0-<target>.tar.zst` | 运行安装包 | 是 |
| `hctl2-0.0.0-<target>-sources.tar.zst` | GPL/AGPL 对应源码与其余构建审计源码 | 否 |

两份归档各有独立的 `.sha256` 文件。源码包必须和运行包保存在同一 Release 下载位置，但普通用户安装和运行 HCTL2 时不需要下载它。

解压并按默认位置安装（macOS 系统 tar 自带 zstd 解码；GNU tar 环境需有 tar 与 `zstd` 命令，通常由 `zstd` 包提供）：

```bash
tar --zstd -xf hctl2-0.0.0-<target>.tar.zst
cd hctl2-0.0.0-<target>
./install.sh
```

默认安装前缀是 `$HOME/.local`。如果 `$HOME/.local/bin` 尚未在 `PATH` 中，可以为当前 shell 加入：

```bash
export PATH="$HOME/.local/bin:$PATH"
```

也可以安装到另一个绝对路径：

```bash
./install.sh --prefix /absolute/path/to/hctl2
```

安装器会校验整个运行归档及 payload 的 SHA-256，随后创建 `$PREFIX/bin/hctl2`、`$PREFIX/bin/hctl2-control`、`$PREFIX/bin/hctl2-services` 与 `$PREFIX/bin/hctl2-tool` 符号链接。重复安装同一个完整包是安全的；安装不会自动启动任何进程。运行包根目录的 `SOURCES.md` 会明确指出与它对应的源码伴随包名。

查看安装器的英文命令帮助：

```bash
./install.sh --help
```

## 使用 `hctl2-services`

先查看命令自身的英文帮助：

```bash
hctl2-services --help
```

### 启动

启动全部四类依赖：

```bash
hctl2-services start
```

只启动一个或多个指定组件：

```bash
hctl2-services start tuwunel
hctl2-services start vikunja dagu
hctl2-services start cinny
```

可用组件名固定为 `tuwunel`、`cinny`、`vikunja`、`dagu` 和 `herdr`。其中 Tuwunel 与 Cinny 共同构成 Chatroom，后者不是第五类执行依赖。不指定组件时由 Process Compose 启动全部五个组件；单独启动 `cinny` 也会按声明依赖启动 Tuwunel。命令在所选组件的声明式就绪探针通过后返回；重复执行 `start` 不会产生第二组进程。

### 查看状态

```bash
hctl2-services status
```

`status` 显示 Process Compose 对五个受管组件的进程状态、就绪状态、PID、运行时长、重启次数和退出码。

只有五个受管组件全部就绪时，`status` 才返回退出码 `0`；任一组件未就绪都会返回非零退出码。因此它可以直接用于脚本、健康检查和 CI，但在启用了 `set -e` 的 shell 中也会使脚本立即退出。

### 运行冒烟检查

```bash
hctl2-services smoke
```

`smoke` 要求五个 Process Compose 就绪探针已经通过，再检查 Cinny 是否只指向随包 Tuwunel 并启用 hash router、Tuwunel 的非加密房间策略，以及 Herdr 的协议回读、API snapshot 和 socket 权限。全部检查通过时返回退出码 `0`。

### 重启或停止

重启全部组件：

```bash
hctl2-services restart
```

只重启指定组件：

```bash
hctl2-services restart dagu
```

停止全部组件：

```bash
hctl2-services stop
```

只停止指定组件：

```bash
hctl2-services stop herdr
```

不指定组件时，Process Compose 按声明的依赖反序停止并退出；停止未运行的受管组件是安全的。进程身份、重启、就绪与关停全部由 Process Compose 管理，HCTL2 不再维护 PID 文件或自行发送信号。

### 本地端点

| 组件 | 用途 | 本地位置 |
| --- | --- | --- |
| Tuwunel | HCTL Room 的 Matrix homeserver | `http://127.0.0.1:6167` |
| Cinny | Chatroom 随包浏览器客户端 | `http://127.0.0.1:6168/` |
| Vikunja | Kanban 浏览器客户端与本地任务后端 | `http://127.0.0.1:3456/` |
| Dagu | Workflow 浏览器客户端与本地工作流引擎 | `http://127.0.0.1:18080/` |
| Herdr | 本地 Agency 参考实现的运行服务（Participant / Terminal） | 仅归属者可访问的 Unix socket；Linux 位于状态目录，macOS 位于短 `/tmp/hctl2-herdr-<uid>/` 目录 |

这些网络服务只监听本机回环地址，不对局域网或公网开放。Cinny 是官方 Web 发行包的静态内容，由随包的官方 `static-web-server` 单二进制提供；它的 homeserver 固定为 `http://127.0.0.1:6167`，不能改连任意服务器。Cinny 主要用于 Matrix 互操作和人工查看，不是 HCTL2 Workbench，也没有 HCTL2 治理权限。

Dagu 还会占用内部端口 `18090`、`15055` 和 `18091`。当前 Tuwunel 配置禁用联邦互通和房间加密，以便 HCTL2 控制面将来可以按消息 ID 读取 HCTL Room 正文；Dagu 仅在本机回环地址上关闭认证；Vikunja 首次启动时生成随机本地密钥。

### 状态、日志和数据

默认状态根目录按以下优先级确定：

1. 绝对路径环境变量 `HCTL2_STATE_ROOT`；
2. `$XDG_STATE_HOME/hctl2`；
3. `$HOME/.local/state/hctl2`。

例如，为一次开发测试隔离全部状态：

```bash
export HCTL2_STATE_ROOT=/absolute/path/to/hctl2-state
hctl2-services start
```

请对同一组 `start`、`status`、`smoke`、`restart` 和 `stop` 命令使用相同的 `HCTL2_STATE_ROOT`。状态根目录主要包含：

| 路径 | 内容 |
| --- | --- |
| `config/` | 自动生成的配置与本地 secret |
| `data/` | Tuwunel、Vikunja、Dagu 与 Herdr 的持久数据 |
| `logs/` | 各服务日志与 `process-compose.log` |
| `runtime/` | Linux 的 Herdr socket 等运行时文件；macOS Herdr 与 Process Compose socket 因路径上限放在 owner-only 的短 `/tmp` 目录，以状态根哈希命名 |

状态目录与安装目录相互独立，升级或重装同一发行包不会主动删除用户数据。

### 常见故障

- 如果组件未能就绪，先看 `hctl2-services status`，再查看状态根目录下 `logs/<component>.log` 与 `logs/process-compose.log`。
- 如果另一组进程占用了固定端口，Process Compose 会把对应组件标成失败；停止或重新配置冲突的实例后再重启。
- 如果命令找不到 Process Compose 实例，先确认当前使用的 `HCTL2_STATE_ROOT` 是否与启动时一致；不同状态根使用不同的控制 socket。
- 如果只启动了部分组件，`status` 和 `smoke` 返回非零是预期行为，因为这两个命令检查的是完整依赖集合。
- 如果 Chatroom 页面可打开但无法连接，先检查 Tuwunel 与 `cinny` 两行状态；客户端配置固定指向 `http://127.0.0.1:6167`，不接受任意 homeserver URL。

## 写入型派工与 ChangeSet

`invocation preview` 的 JSON 输入可以带 `write`。Worker Profile 也须是 `write` 模式并允许 `git.write`。基线填 Git 回读的提交 SHA；目标分支和能否更新评审请求由人明确选择：

```json
{
  "project_id": "P",
  "room_id": "R",
  "target": "worker-name",
  "profile": {"key": {"scope": {"kind": "control"}, "kind": "worker_profile_revision", "id": "<revision>"}, "version": {"state": 1}},
  "request": "修改代码并交回测试结果",
  "budget": 65536,
  "deadline_ms": 1799999999999,
  "write": {
    "change_set_id": null,
    "baseline_commit": "<完整的 40 位提交 SHA>",
    "target_branch": "main",
    "allow_update": true
  }
}
```

`profile` 使用 `profile create/show` 回读的精确 Revision 引用，不自行拼引用。`change_set_id: null` 请求新 ChangeSet；复用既有 ChangeSet 时填它的 ID，旧写入者的停止证据仍要成立。预览本身不授租约：

```bash
hctl2 invocation preview --input /absolute/write.json --key write-1
hctl2 invocation start --input /absolute/write.json --key write-1 --preview-token <t>
hctl2 invocation show <project_id> <invocation_id>
hctl2 changeset show <repo_id> <change_set_id>
hctl2 changeset diff <repo_id> <change_set_id> <revision_id>
```

人也能独立封存，不借执行者租约。`seal` 的输入含 `repo_id`、`base_commit_sha`、可选的 `change_set_id / parent_revision_id`，以及 `location`：提交用 `{"kind":"commit","repo_path":"/absolute/repo","commit_sha":"<sha>"}`；未提交修改用 P1 工作树的 `{"kind":"worktree","repo_path":"/absolute/sites/<ChangeSet>"}`。

```bash
hctl2 changeset seal --input /absolute/seal.json --key human-1
hctl2 changeset seal --input /absolute/seal.json --key human-1 --preview-token <t>
```

### 处理失权残留

`takeover / adopt / discard` 都先预览、再用同一输入和预览票确认。输入含 `repo_id`、来源 `change_set_id` 和精确的绝对 `repo_path`。`takeover` 把预览的快照作为人的独立版本接受到原 ChangeSet；`adopt` 接受到另一 ChangeSet，省略 `target_change_set_id` 时新建；目标已有版本时可显式给 `parent_revision_id`。这两条命令不启动执行，也不把旧租约改成已停止。

```bash
hctl2 changeset takeover --input /absolute/residual.json --key recover-1
hctl2 changeset takeover --input /absolute/residual.json --key recover-1 --preview-token <t>
hctl2 changeset adopt --input /absolute/residual.json --key adopt-1
hctl2 changeset adopt --input /absolute/residual.json --key adopt-1 --preview-token <t>
hctl2 changeset discard --input /absolute/residual.json --key discard-1
hctl2 changeset discard --input /absolute/residual.json --key discard-1 --preview-token <t>
```

丢弃确认绑定预览的树与路径；确认前又有非忽略修改或目录迁移时拒绝删除。确认删除的范围是整个该工作树，包括 Git 忽略的文件；预览会列出 Git 状态，忽略文件不在封存快照里。`changeset show` 返回保存的 Git 观测和残留处理记录，含工作树路径。目录在另一台机器上时，这些命令不能假装能访问它；先在资源所在机器保全，不能把未封存字节自动搬过来。没有旧写入者停止证明时仍不授新租约。

### 把平台评审评论带入返工

下一次 Invocation 的输入加 `review_change_set_revision`，值为所选 ChangeSet Revision 的精确 Store 引用。预览通过该版本的 `changeset_platform_binding` 读取评审请求、一般评论、评审及该提交的行内评论；完整标识和原文冻进 Context Bundle。预览后平台改了评论，也不替换本次已确认的字节。读不到来源会报错，不静默漏材料。评论里的「合入吧」只是 content，不替代 `integration` 的授权。

## 发布评审与合入（`hctl2 review`、`hctl2 integration`）

写入型调用封存的版本按冻结的评审发布策略由 control 自动发布去评审；策略开了「须人显式确认」时意图停在 `pending_human`，由人放行：

```bash
hctl2 review list <repo_id>
hctl2 review show <repo_id> <intent_id>
hctl2 review publish <repo_id> <intent_id>                       # 预览：推到哪个分支、建到哪个目标分支
hctl2 review publish <repo_id> <intent_id> --preview-token <t>   # 放行；推送与建请求在后台跑并回读
```

`show` 给出两段各自的确认（推送到的提交、评审请求编号）和这个 ChangeSet 各版本的映射。合入另走 `hctl2 integration preview|submit|show|list`。

## 使用 `hctl2-tool`

安装离线包后，`PATH` 里的 `hctl2-tool` 就是发行物里的那一份。从源码构建：

```bash
cd src
./buck2 build root//apps/tool:hctl2-tool
```

`--help` 与 `--version` 为英文。无参数调用等同于 `--help`。Git 现场命令要求宿主 `git` ≥ 2.39；可用 `HCTL2_GIT` 覆盖可执行文件路径，与 `HCTL2_GH` 同款。每次调用在标准输出写一条 JSON 记录，`evidence_level` 为 `unmediated`（直报，2026-09-14 前为 `toolbox_readback`）。`outcome` 为 `established`（成立）、`not_established`（已确定不成立）、`unreadable`（读不到）或 `timeout`（仅 `wait`）；对应退出码 `0`、`3`、`4`、`5`。参数或启动错误返回 `1` 并写到标准错误。观察类失败（含仓库状态不成立）走标准输出 JSON，带 `error.code` 与 `error.recovery_action`。

意图字段由调用方给出：ChangeSet 引用、基线、目标 ref、预期头、幂等键。`hctl2-tool` 不发明 ID，不读、不写控制面存储。

### 仓库检查

```bash
hctl2-tool repo inspect --path /path/to/repo
hctl2-tool repo inspect --path /path/to/repo --ref refs/heads/main
```

输出把 Git 公共目录身份、稳定 Repo 身份（`<repo>/.hctl2/repo.toml` 缺失就报 missing，不造）和辅助证据分成三个字段组。

### worktree 物化与核验

```bash
hctl2-tool worktree materialize --repo /path/to/repo \
  --root /path/to/worktree-root --change-set-ref CS-1 --baseline <commit-sha>
hctl2-tool worktree verify --repo /path/to/repo --change-set-ref CS-1
```

同一 ChangeSet 引用重复物化返回同一工作树。checkout 不进 Git 内部目录。核验分开计数已跟踪与未跟踪修改。

### 封存、保全、拆除

```bash
hctl2-tool archive snapshot --repo /path/to/repo --change-set-ref CS-1
hctl2-tool archive remove --repo /path/to/repo --change-set-ref CS-1
hctl2-tool archive remove --repo /path/to/repo --change-set-ref CS-1 \
  --discard-unarchived --confirm-discard <current-tree-sha>
```

默认拆除先封存再证明可达副本，然后只拆本次 worktree。`--confirm-discard` 必须等于当前工作树的树 sha（可用一次 snapshot 或确认不匹配记录里的 `current_tree_sha`）；缺一对是用法错误。被忽略的未跟踪文件默认随工作树删除，并列入残留（路径、大小、总数）；`--reject-ignored` 在有残留时拒绝拆除。磁盘上真实存在的嵌套仓库或已初始化子模块拒绝保全拆除。

### 本地集成

```bash
hctl2-tool integrate --repo /path/to/repo \
  --commit <candidate-sha> --base-commit-sha <baseline-sha> \
  --result-tree-sha <tree-sha> --target-ref refs/heads/main \
  --expected-head <current-main-sha> --strategy fast-forward \
  --idempotency-key <caller-key>
```

`--strategy` 为 `fast-forward` 或 `merge-commit`。`fast-forward` 要求目标头是候选提交的祖先（目标可以已经沿候选这条线前移；分叉的目标拒绝），候选已经可从目标头到达时不写、回 `already_applied`。目标 ref 正被任一工作树检出时默认拒绝；`--allow-checked-out-target` 才放行，且该开关绑在幂等键上。成功回读后 `status` 为 `applied` 或 `already_applied`。`hctl2-tool` 把预备提交钉在 `refs/hctl2/integrations/` 下作重试缓存，失败也不自动删；P1 不加清理子命令，P2 control 在意图结束且结果仍有可达副本时负责显式清理。

### 从平台的 Git 回读目标 ref

```bash
HCTL2_GIT_USER=<账号> HCTL2_GIT_TOKEN=<令牌> hctl2-tool readback \
  --path /path/to/mirror.git --remote http://127.0.0.1:3001/owner/name.git \
  --ref refs/heads/main --commit <merge-sha>
```

`--path` 是调用方自己的仓库（裸仓库即可），`--ref` 被原名拉进去（强制更新，带 `--no-tags`）。记录 `hctl2.readback.v1` 报远端与本地读到的头、头的树、`--commit` 是否存在、它的树与父提交、以及目标头的历史是否包含它（`contains`）；远端没有这个 ref 时头为 `null`。只报事实，不判断集成是否成功。凭据只从 `HCTL2_GIT_USER` / `HCTL2_GIT_TOKEN` 两个环境变量走 Git 自己的每进程 credential helper，不写配置、不进 URL；没有这两个变量就按匿名拉取。control 对平台目标签 Integration Receipt 之前就是用它核合并提交与目标头。

### 等待外部事实

`wait` 接受绝对 Unix 秒截止时间与一个事实：

```bash
hctl2-tool wait --deadline 1788451200 commit-ci \
  --repo yesme/hctl2 --commit <commit-sha>
hctl2-tool wait --deadline 1788451200 pr-merged \
  --repo yesme/hctl2 --number 82
hctl2-tool wait --deadline 1788451200 ref-advanced \
  --repo yesme/hctl2 --ref heads/main --from <old-sha>
hctl2-tool wait --deadline 1788451200 path-digest \
  --path /absolute/path/to/artifact --sha256 <lowercase-sha256>
hctl2-tool wait --deadline 1788451200 process-exited --pid 12345
```

GitHub 三类事实调用随包固定版本的 `gh` 并复用用户已有登录；它不会发起交互登录，使用前可由用户运行 `gh auth login` 建立凭据，或按 GitHub CLI 支持的环境变量提供令牌。

## 安装完整离线包

完整离线包的下载、校验、解压和安装步骤见[安装当前离线包](#安装当前离线包)。最终用户只需下载同一版本和目标平台的运行包及其 `.sha256` 文件；源码伴随包与它的校验文件在同一 Release 提供，供源码与供应链审计按需下载，不参与安装。

安装后提供 `hctl2`、`hctl2-control`、`hctl2-tool` 与 `hctl2-services`。`hctl2 start` 拉起控制面，并按首次消费经 Process Compose 拉起随包服务：Tuwunel 是每个 Project 主 Room 的落点，随 start 拉起；Gitea 不随 start 拉起：`hctl2 services consume gitea` 把它记为已消费并拉起，之后随 start 一起起；`hctl2 repo register` 注册纯本地仓库（或显式选本地平台）时已接入该入口，并在就绪后建仓与交付初始代码，身份确认后激活 Repo。`hctl2 repo grant` 给本机的人在同一个本地 Gitea 上建普通账号（已存在就复用）并授予已激活 Repo 的协作权，同样按需拉起 Gitea；它打印的初始口令只出现一次，控制面不保存。命令、输入与恢复例子见 [Repo 注册说明](../src/crates/repo/README.md#cli)。GitHub 等外部平台的克隆绑来源平台，不会拉起 Gitea。已消费集合记在控制面数据目录的 `hosted-consumed.json`，随 `hctl2 services backup` 一起备份、随 `restore` 写回；`hctl2 services status` 列出每个托管组件的 `consumed` 与健康。control 不等所有服务探针通过才接受控制面命令。缺省数据目录与 `hctl2-services` 相同（`~/.local/state/hctl2` 或 `$XDG_STATE_HOME/hctl2`）；只有 `hctl2 --root DIR` 才把服务状态放到 `DIR/services`。运行 `hctl2-services start` 仍会启动全部随包组件（Tuwunel、Cinny、Gitea、Vikunja、Dagu、Herdr），请与 `hctl2 start` 共用同一状态根，避免抢端口。`hctl2 stop` 停掉本控制面拉起的 Tuwunel 与 Gitea；若没有别的组件在跑，会把 Process Compose 项目 `down` 掉，否则本体可按 `--keep-project` 留下。Tuwunel 与 Cinny 共同组成 Chatroom。Vikunja 不随 `hctl2 start` 拉起。

Room 端口的当前命令和确认流程见 [Chat 实现说明](../src/crates/chat/README.md#cli-示例与提要)：`hctl2 room list|show` 读取已建立的 Room，`draft` 机械选入原文，`create-topic|close|rebind|send|freeze|resume` 经预览确认执行。本阶段尚未提供 Project 创建命令，主 Room 的业务入口由 P2.2 辛接线；不要把端口已实现当作完整 Project 使用路径已交付。

密钥后端缺省沿用「有钥匙串就用钥匙串、没有就退到用户目录下的 0600 文件」。要显式选一种，在 `hctl2 init` 或 `hctl2 start` 上加 `--secret-backend system-keyring|user-file`：它记进控制面数据目录的 `control.json`，之后每次 start 都按它执行；`hctl2 status` 的 `policy.credential_storage` 报告实际在用的那一种。无屏幕会话（CI、纯终端）建议 `user-file`：`system-keyring` 在 macOS 上每次读都要求授权，弹不出来就直接失败。选项与配置形状见 [control 说明](../src/apps/control/README.md#密钥后端)。

完整的端到端操作步骤见[演示 1「协作现场」操作手册](./demos/demo-1.md)。

## 制作外部子系统包

这一节面向发布与打包开发者，不是最终用户安装步骤。日常组包消费上游官方制品和 HCTL2 托管的 macOS Tuwunel 预编译制品；版本、URL、SHA-256 和 target identity 统一由 `packaging/dependencies/lock.json` 锁定。进入 `src/`，显式选择平台并运行 Buck：

```bash
./buck2 build root//packaging/dependencies:package \
  --target-platforms root//build/platforms:macos_arm64 \
  --out /absolute/path/dependency-packages
```

完整验证两份归档的内容与校验清单，以及运行包的离线安装、幂等重装、启动、冒烟检查和停止：

```bash
./buck2 test root//packaging/dependencies:package-test \
  --target-platforms root//build/platforms:macos_arm64
```

源码构建只用于更新 HCTL2 托管的 macOS Tuwunel 预编译制品，不进入日常组包依赖。更新时由 `Tuwunel macOS assets` workflow 在对应架构的原生 macOS runner 上构建并测试 arm64 与 x86_64 制品，再把发布地址和摘要写回 lock；普通安装、组包和完整包验证继续消费锁定的预编译制品。

外部运行包、源码伴随包及各自的 `.sha256` 位于导出的 Buck 目录，不会提交到 Git。

在源码仓库中，更详细的供应链、版本锁定与平台范围记录在 `src/packaging/dependencies/README.md`；Buck2 第一方导出、确定性组装和完整包验收记录在 `src/packaging/release/README.md`。

# ubuntu 上给 Agency 导入专用登录：带路文本

> 给 ubuntu 上帮所有者带路的 agent 读的（所有者 2026-10-21：「能不能放个文本，我找ubuntu上的minimax指导我做」）。依据：`07` §二 第 38 条（凭据按归属分三类）、第 55 条（真会话测试一律用专用登录，不指向所有者个人的）、第 58 条（ubuntu 专用登录的位置）。做完以后，ubuntu 上的真会话测试与核验都用这里导入的登录。

## 带路的 agent 先读这几条

- **登录命令由所有者自己执行。** 你只说下一步敲什么、解释输出，自己不执行任何登录、导入、写 token 文件的命令（CONSTRAINTS：遇到凭据一类，不替用户做）。
- **不读、不打印、不经手 token。** 不 `cat` token 文件，不让所有者把 token 贴进对话，不把 token 写进命令参数。
- **你可以自己做的只有只读检查**：`ls -ld <路径>`、`<agency> --root ... login check ...`、`<命令> --help`、`git log` 一类。
- **不碰这些**：`~/.local/state/hctl2`（所有者自己的实例）；所有者个人的 `~/.claude`、`~/.claude.json`、`~/.codex`、`~/.omp`（不读、不复制）；别的席位的 `/var/tmp` 目录与进程。
- 每一步的检查没过，就停下告诉所有者卡在哪一步、输出是什么，不自己想办法绕。
- 四件事互相独立：第 4 件（omp）最急，H3（#464）合入前要用；Claude、Codex 两件可以晚点做。所有者说先做哪件就做哪件。

## 0. 准备：找到带 `login` 子命令的 agency

```bash
A=/var/tmp/hctl2-466-bin/agency
$A login --help
```

- 能看到 `prepare`、`check`、`claude-token` 三个子命令，就用它。
- 文件不存在或没有 `login`：说明 #466 已合入、这份临时编译被清掉了。那就在你自己的工作树同步 main 后构建：`git fetch --prune origin && git merge --ff-only origin/main && cd src && ./buck2 build root//agency:agency --show-full-output`，把输出的路径赋给 `A`。

```bash
ROOT=/var/tmp/hctl2-managed-login/agency
$A --root "$ROOT" login prepare
```

输出是一行 JSON，记下其中三个路径（只是路径，不是密钥）：`home`、`claude_config_dir`、`codex_home`；`harness_home` 也记下，核验席跑测试时要用。

## 1. Claude Code：导入所有者已有的 token

所有者已经有一个 `claude setup-token` 生成的 token（在 mac 上生成过），同一个 token 在 ubuntu 上照样能用，不用重新生成。

```bash
umask 077
${EDITOR:-nano} /var/tmp/claude-token.txt     # 所有者把 token 粘进去，存盘；只有这一行
$A --root "$ROOT" login claude-token --expires-at "$(date -d '2027-09-15' +%s)" < /var/tmp/claude-token.txt
$A --root "$ROOT" login check claude-code
```

- `--expires-at` 是所有者自己声明的截止时间。setup-token 一年有效；上面写的 2027-09-15 留了余量。所有者记得 mac 上导入时写的日期，就用同一个。
- 检查要看到 `"local_preconditions_met":true`。导入命令拒绝从终端直接读 token，必须用文件，这是有意的。
- **先别删 `/var/tmp/claude-token.txt`**，第 2 步还要用一次。

## 2. Claude Code：把第一次启动的几屏点完

托管目录里的 Claude 是全新的，第一次启动会停在欢迎、主题、文字样式、信任这几屏，不点完 Agency 起不来会话。所有者在 mac 上做过同样一步。

```bash
mkdir -p /var/tmp/claude-firstrun && cd /var/tmp/claude-firstrun
HOME=<home> CLAUDE_CONFIG_DIR=<claude_config_dir> CLAUDE_CODE_OAUTH_TOKEN="$(cat /var/tmp/claude-token.txt)" claude
```

- `<home>`、`<claude_config_dir>` 换成第 0 步记下的路径。token 是命令执行时从文件读的，不会留在 shell 历史里；你（带路的 agent）不要自己执行这一条。
- 一屏一屏点完，看到输入框后输入 `/exit` 退出。
- 退出后删掉 token 文件：`shred -u /var/tmp/claude-token.txt`。

## 3. Codex：登录订阅账号到托管目录

```bash
codex login --help        # 带路的 agent 可以先看：有没有不开浏览器的登录方式（设备码一类）
CODEX_HOME=<codex_home> HOME=<home> codex -c 'cli_auth_credentials_store="file"' login
$A --root "$ROOT" login check codex-cli
```

- 登录会打开浏览器，用所有者的 Codex 订阅账号授权。ubuntu 上开不了浏览器：先看 `--help` 里有没有设备码一类的选项，有就用；没有就停下，告诉所有者「需要在有桌面的会话里做」。
- 检查要看到 `"local_preconditions_met":true`。

## 4. omp：H3（#464）用的专用登录

```bash
export OMP_MANAGED_LOGIN_HOME=/var/tmp/hctl2-omp-managed
install -d -m 700 "$OMP_MANAGED_LOGIN_HOME"
HOME="$OMP_MANAGED_LOGIN_HOME" omp auth-broker login deepseek
cat > "$OMP_MANAGED_LOGIN_HOME/.omp/agent/config.yml" <<'CFG'
modelRoles:
  default: deepseek/deepseek-flash
CFG
ls -l "$OMP_MANAGED_LOGIN_HOME/.omp/agent/"
```

- 第三条是交互登录，按提示由所有者完成；产物只落在 `/var/tmp/hctl2-omp-managed/.omp/agent` 下，不碰所有者个人的 `~/.omp`。
- 最后一条只看目录里有没有登录产物（如 `agent.db`），不打开文件内容。

## 做完以后

告诉所有者哪几件做完了，所有者转告主笔 Claude（mac）。核验席以后在 ubuntu 上跑真会话，用这些位置：

| 用途 | 设置 |
| --- | --- |
| Claude Code、Codex 真会话 | `HCTL2_LIVE_AGENCY_ROOT=/var/tmp/hctl2-managed-login/agency`；`HCTL2_AGENCY_HARNESS_HOME=<第 0 步的 harness_home>`；`HCTL2_HARNESS_LIVE=1` |
| omp 真会话（#464） | 照 #464 里 DeepSeek（ubuntu）2026-10-20 18:56 的评论，指向 `/var/tmp/hctl2-omp-managed` |

`/var/tmp` 在重启后一般还在；被清掉了就照本文重做一遍。

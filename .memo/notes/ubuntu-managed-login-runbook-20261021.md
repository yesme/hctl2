# ubuntu 上给 Agency 导入专用登录：带路文本

> 给 ubuntu 上帮所有者带路的 agent 读的（所有者 2026-10-21：「能不能放个文本，我找ubuntu上的minimax指导我做」）。依据：`07` §二 第 38 条（凭据按归属分三类）、第 55 条（真会话测试一律用专用登录，不指向所有者个人的）、第 58 条（ubuntu 专用登录的位置）。做完以后，ubuntu 上的真会话测试与核验都用这里导入的登录。

## 带路的 agent 先读这几条

- **登录命令由所有者自己执行。** 你只说下一步敲什么、解释输出，自己不执行任何登录、导入、写 token 文件的命令（CONSTRAINTS：遇到凭据一类，不替用户做）。
- **不读、不打印、不经手 token。** 不 `cat` token 文件，不让所有者把 token 贴进对话，不把 token 写进命令参数。
- **你可以自己做的只有只读检查**：`ls -ld <路径>`、`<agency> --root ... login check ...`、`<命令> --help`、`git log` 一类。
- **不碰这些**：`~/.local/state/hctl2`（所有者自己的实例）；所有者个人的 `~/.claude`、`~/.claude.json`、`~/.codex`、`~/.omp`（不读、不复制）；别的席位的 `/var/tmp` 目录与进程。
- 每一步的检查没过，就停下告诉所有者卡在哪一步、输出是什么，不自己想办法绕。
- 三件事互相独立：第 3 件（omp）最急，H3（#464）合入前要用；Claude、Codex 两件可以晚点做。所有者说先做哪件就做哪件。

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

目录不用先建，`login prepare` 自己建（0700）。输出是一行 JSON，记下其中三个路径（只是路径，不是密钥）：`home`、`claude_config_dir`、`codex_home`；`harness_home`（缺省是 `/var/tmp/hctl2-managed-login/agency.harness-login`）也记下，核验席跑测试时要用。

## 1. Claude Code：导入所有者已有的 token，顺手把第一次启动的几屏点完

所有者已经有一个 `claude setup-token` 生成的 token（在 mac 上生成过），同一个 token 在 ubuntu 上照样能用，不用重新生成。下面几条在**同一个 shell** 里连着敲：

```bash
read -rs -p '粘贴 Claude token，回车（屏幕上不显示）: ' CLAUDE_TOKEN; echo
printf '%s' "$CLAUDE_TOKEN" | $A --root "$ROOT" login claude-token --expires-at "$(date -d '2027-09-15' +%s)"
$A --root "$ROOT" login check claude-code
```

- `read -s` 不回显、不进 shell 历史；`printf` 是 bash 内建命令，不另起进程，`ps` 里看不到 token。导入命令拒绝直接从终端读，从管道读是允许的。
- `--expires-at` 是所有者自己声明的截止时间。setup-token 一年有效；上面写的 2027-09-15 留了余量。所有者记得 mac 上导入时写的日期，就用同一个。
- 检查要看到 `"local_preconditions_met":true`。

接着在同一个 shell 里点完第一次启动的几屏。托管目录里的 Claude 是全新的，会停在欢迎、主题、文字样式、信任这几屏，不点完 Agency 起不来会话；所有者在 mac 上做过同样一步：

```bash
mkdir -p /var/tmp/claude-firstrun && cd /var/tmp/claude-firstrun
HOME=<home> CLAUDE_CONFIG_DIR=<claude_config_dir> CLAUDE_CODE_OAUTH_TOKEN="$CLAUDE_TOKEN" claude
# 一屏一屏点完，看到输入框后输入 /exit
unset CLAUDE_TOKEN
```

- `<home>`、`<claude_config_dir>` 换成第 0 步记下的路径。Agency 起 Claude 会话用的正是这两个目录（`src/agency/src/standby.rs` 设 `HOME`、`CLAUDE_CONFIG_DIR`），所以这里点完的几屏，Agency 起会话时就不会再出现。你（带路的 agent）不要自己执行这几条。

## 2. Codex：登录订阅账号到托管目录

```bash
CODEX_HOME=<codex_home> HOME=<home> codex -c 'cli_auth_credentials_store="file"' login --device-auth
$A --root "$ROOT" login check codex-cli
```

- `--device-auth` 是设备码登录：终端打印一个网址和一串码，所有者在任何一台有浏览器的设备上打开网址、输入码、用 Codex 订阅账号授权。ubuntu 上没有桌面也能做。（Codex 0.163.0 的 `codex login --help` 里有这个选项；ubuntu 上版本不同，先 `codex login --help` 看一眼。）
- `cli_auth_credentials_store="file"` 让登录落成托管目录里的 `auth.json`，不进系统钥匙串；Agency 起会话时钉同一个后端。
- 检查要看到 `"local_preconditions_met":true`。

## 3. omp：H3（#464）用的专用登录

omp 用 DeepSeek 的 API key，不是浏览器登录。所有者先在 DeepSeek 后台（https://platform.deepseek.com/api_keys）为 Agency 新建一个专用 key，不复用平时用的那个：用量单独看，出事单独吊销。然后敲下面几条；第三条 `omp auth-broker login deepseek` 会打印同一个网址，提示粘贴 key，所有者粘贴、回车即可，key 存进专用 HOME 下的 `.omp/agent/agent.db`。

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

- 第三条由所有者粘贴 key（粘贴时屏幕上回不回显没查到；回显了也只在所有者自己的终端里，不进命令参数和历史）；产物只落在 `/var/tmp/hctl2-omp-managed/.omp/agent` 下，不碰所有者个人的 `~/.omp`。
- 最后一条只看目录里有没有 `agent.db` 和 `config.yml`，不打开文件内容。`config.yml` 的模型名照 #464 作者 DeepSeek（ubuntu）2026-10-20 18:56 评论里实跑过的写法。

## 做完以后

告诉所有者哪几件做完了，所有者转告主笔 Claude（mac）。核验席以后在 ubuntu 上跑真会话，用这些位置：

| 用途 | 设置 |
| --- | --- |
| Claude Code、Codex 真会话 | `HCTL2_LIVE_AGENCY_ROOT=/var/tmp/hctl2-managed-login/agency`；`HCTL2_AGENCY_HARNESS_HOME=<第 0 步的 harness_home>`；`HCTL2_HARNESS_LIVE=1` |
| omp 真会话（#464） | 照 #464 里 DeepSeek（ubuntu）2026-10-20 18:56 的评论，指向 `/var/tmp/hctl2-omp-managed` |

`/var/tmp` 在重启后一般还在；被清掉了就照本文重做一遍。

## 本文命令查证到哪一步

主笔 Claude（mac）2026-10-21 核过：`agency login` 的三个子命令、`prepare` 输出的字段名、`check` 输出的 `local_preconditions_met`、导入命令拒绝终端但收管道（#466 头 `a7c2885a` 的 `src/agency/src/main.rs`、`managed_login.rs`）；Agency 起 Claude 用的 `HOME`、`CLAUDE_CONFIG_DIR`（同头 `standby.rs`）；`codex login --device-auth`（mac 上 Codex 0.163.0 的帮助）；`omp auth-broker login deepseek` 打印 DeepSeek 后台网址、提示粘贴 key、落 `~/.omp/agent/agent.db`（mac 上在临时 HOME 里实看，没有粘贴真 key）。没核到的：ubuntu 上三家的版本与帮助、omp 粘贴时是否回显；遇到和本文不符的，停下报。

# ubuntu 上给 Agency 导入专用登录

> 所有者 2026-10-21 要的（「能不能放个文本，我找ubuntu上的minimax指导我做」），同日照第一次实操改写：每件事一段，所有者整段复制、粘贴一次，要输入时它自己提示（所有者：「codex直接把zsh的()给包好，我一运行，输入，就好了」）。依据：`07` §二 第 38 条（凭据按归属分三类）、第 55 条（真会话测试一律用专用登录，不指向所有者个人的）、第 58 条（ubuntu 专用登录的位置）。

## 怎么用

三件互相独立，要哪件做哪件。每段整段复制，粘贴进 ubuntu 的终端，回车。

- 每段包在 `( … )` 里，在子 shell 里跑：变量、`HOME` 的改动和 token 随它结束一起消失，不留在所有者的 shell 里；中间一步出错就停在那一步，不往下走（`set -e`）。
- zsh 和 bash 都能跑；没有 heredoc、没有行内注释，从终端界面复制时每行前面多出来的缩进不影响。
- token、key 只在提示时粘贴，屏幕上不显示（omp 那件由 omp 自己提示），不进命令参数和 shell 历史。

## 1. omp（H3 #464 用，2026-10-21 已做）

先在 DeepSeek 后台（https://platform.deepseek.com/api_keys）为 Agency 新建一个专用 key，不复用平时那个：用量单独看，出事单独吊销。然后粘贴这段，omp 提示时粘贴 key：

```
(
set -e
umask 077
H=/var/tmp/hctl2-omp-managed
mkdir -p "$H"
HOME="$H" omp auth-broker login deepseek
C="$H/.omp/agent/config.yml"
printf 'modelRoles:\n  default: deepseek/deepseek-flash\n' > "$C"
chmod -R go-rwx "$H"
ls -l "$H/.omp/agent"
)
```

最后列出的 `.omp/agent` 里有 `agent.db` 和 `config.yml` 就成了。`config.yml` 的模型名照 #464 作者 DeepSeek（ubuntu）2026-10-20 18:56 评论里实跑过的写法。omp 没有 `agency login check`，看这两个文件在不在就是检查。

## 2. Claude Code

所有者已有 `claude setup-token` 生成的 token（mac 上生成过），同一个 token 在 ubuntu 上照样能用。粘贴这段：先提示粘贴 token，导入并检查，然后进入一次 Claude，把第一次启动的欢迎、主题、信任几屏点完，看到输入框后输入 `/exit`。

```
(
set -e
umask 077
A=/var/tmp/hctl2-466-bin/agency
R=/var/tmp/hctl2-managed-login/agency
H=/var/tmp/hctl2-managed-login/agency.harness-login
unset HCTL2_AGENCY_HARNESS_HOME
"$A" --root "$R" login prepare >/dev/null
printf '粘贴 Claude token，回车（屏幕上不显示）: '
read -rs T
echo
E=$(date -d 2027-09-15 +%s)
printf '%s' "$T" | "$A" --root "$R" login claude-token --expires-at "$E"
"$A" --root "$R" login check claude-code
mkdir -p /var/tmp/claude-firstrun
cd /var/tmp/claude-firstrun
echo '进入 Claude：点完几屏，见到输入框后输入 /exit'
export HOME="$H/home"
export CLAUDE_CONFIG_DIR="$H/claude"
export CLAUDE_CODE_OAUTH_TOKEN="$T"
claude
)
```

- 检查那一行要输出 `"local_preconditions_met":true`。
- 截止时间写的是 2027-09-15（setup-token 一年有效，留了余量）；所有者记得 mac 上导入时写的日期，就把段里的日期换成那个。
- 进入 Claude 用的 `HOME`、`CLAUDE_CONFIG_DIR` 和 Agency 起会话时一样（`src/agency/src/standby.rs`），这里点完，Agency 起会话时就不会再停在这几屏。

## 3. Codex

粘贴这段。它打印一个网址和一串码，在任何有浏览器的设备上打开网址、输入码、用 Codex 订阅账号授权；回到终端，最后一行检查要输出 `"local_preconditions_met":true`。

```
(
set -e
umask 077
A=/var/tmp/hctl2-466-bin/agency
R=/var/tmp/hctl2-managed-login/agency
H=/var/tmp/hctl2-managed-login/agency.harness-login
unset HCTL2_AGENCY_HARNESS_HOME
"$A" --root "$R" login prepare >/dev/null
export CODEX_HOME="$H/codex"
export HOME="$H/home"
codex -c 'cli_auth_credentials_store="file"' login --device-auth
"$A" --root "$R" login check codex-cli
)
```

`cli_auth_credentials_store="file"` 让登录落成托管目录里的 `auth.json`，不进系统钥匙串；Agency 起会话时钉同一个后端。

## 段里写死的位置

- agency 程序：`/var/tmp/hctl2-466-bin/agency`，是 Antigravity（ubuntu）预先编好的 #466 版本。段里第一次调用它就报「没有这个文件」，说明 #466 已合入、这份临时编译被清掉了：在任一工作树同步 main 后 `cd src && ./buck2 build root//agency:agency --show-full-output`，把段里那一行换成输出的路径。
- Claude、Codex：Agency 根 `/var/tmp/hctl2-managed-login/agency`，托管目录是它的同级 `/var/tmp/hctl2-managed-login/agency.harness-login`（`login prepare` 自己建，0700；下面有 `home`、`claude`、`codex`）。段里先 `unset HCTL2_AGENCY_HARNESS_HOME`，保证落在这个缺省位置。
- omp：`/var/tmp/hctl2-omp-managed`。

`/var/tmp` 重启后一般还在；被清掉了就重新粘贴对应那段。

## 核验席怎么用

| 用途 | 设置 |
| --- | --- |
| Claude Code、Codex 真会话 | `HCTL2_LIVE_AGENCY_ROOT=/var/tmp/hctl2-managed-login/agency`；`HCTL2_AGENCY_HARNESS_HOME=/var/tmp/hctl2-managed-login/agency.harness-login`；`HCTL2_HARNESS_LIVE=1` |
| omp 真会话（#464） | `HCTL2_HARNESS_LIVE=1`、`HCTL2_OMP_REAL=1`、`HCTL2_OMP_AGENT_DB=/var/tmp/hctl2-omp-managed/.omp/agent/agent.db`、`HCTL2_OMP_CONFIG=/var/tmp/hctl2-omp-managed/.omp/agent/config.yml` |

## 查证到哪一步

主笔 Claude（mac）2026-10-21：三段在 mac 上用 zsh（交互模式）和 bash 各跑一遍，每行前加两格缩进模拟复制，agency、claude、codex、omp、date 用桩代替——token 经管道交到导入命令、长度对、`HOME` 与托管目录指对、`config.yml` 落成 0600。命令与字段对过源码：#466 头 `a7c2885a` 的 `src/agency/src/main.rs`、`managed_login.rs`（`prepare` 建目录、同级 `.harness-login`、`home`/`claude`/`codex` 三个子目录、导入拒绝终端收管道）、`standby.rs`；`codex login --device-auth`（Codex 0.163.0 帮助）；`omp auth-broker login deepseek` 打印后台网址、提示粘贴、落 `~/.omp/agent/agent.db`（mac 上临时 HOME 里实看）。omp 那段 2026-10-21 由所有者在 ubuntu 上做成（当时还是逐条写法，heredoc 在 zsh 里因复制带缩进失败，改成本段的写法）。没核到的：ubuntu 上 Claude、Codex 两段没有真跑过；遇到和本文不符的，停下报。

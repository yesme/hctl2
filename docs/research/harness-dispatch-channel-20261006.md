# 各家 harness 怎么交一轮正文

> 状态：调研 · 日期：2026-10-06<br>
> 类别：① Coding Harness · 证据编号：E-HARNESS-DISPATCH-CHANNEL<br>
> 定位：只回答「一轮正文怎么送进活着的会话、结束和回答从哪取、续接与 pane 是否还在」。不定义 HCTL2 的派工或完成。复用判断沿用 [docs/research/README.md](./README.md)。证据三档：**官方文档说**、**源码里看到**、**实测**。没查到的写「未查到」，不猜。

## 定位

第 3d 包把 Claude Code 的正文改成：输入框里只交标记，会话插件在 `prompt.submit` 把正文换进去。所有者接着问别家有没有同样的钩子。这份文件按同一组五个问题看本机现役的各家。演示 3 的第二家是 Codex，决定建议只针对它。建议归建议，不替所有者定。

Claude Code 的交法本次不重测，只作对照，依据是已有复核 [Herdr 3d](./sdk/herdr.md#2026-10-06--3d-常驻会话前置核验) 与 [钩子复核](./harness-hooks-20260903.md#2026-10-06--claude-常驻会话的一轮结束)。

五个问题：

1. 正文怎么不经输入框送进活着的会话。
2. 一轮的结束与回答从哪里取；子代理怎么区分。
3. 能不能只对一次启动生效、不动全局配置；要装的东西装在哪。
4. 怎么续接上一次的对话。
5. 走结构化接口时，pane 里还是不是平常的交互界面；Herdr 认不认得它是 agent。

Herdr 认不认得，统一对着官方 Herdr 0.9.3 源码 `7b116c05bfda646af39d2524c54e70c751f57ee8` 的 `src/detect/mod.rs`：`identify_agent` 按进程名匹配。该提交认 `codex`、`grok` / `grok-build`、`agy` / `antigravity`、`kimi`、`qodercli` / `qoderclicn`、`omp`、`opencode`、`pi`。**未查到** `zcode` 或 `mcode`。交互界面被认成 agent，不等于结构化接口也占着那个 pane。

## 总表

| Harness（本机或文档版本） | 不经输入框交正文 | 改写提交文字的钩子 | 一轮结束与回答 | 只对一次启动、不动全局配置 | 续接 | 结构化接口还有没有交互 pane | Herdr 进程名 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Codex CLI 0.160.1 | 有。`codex exec` 与 app-server `turn/start` 把正文当参数 | 没有替换字段。`UserPromptSubmit` 只能追加 developer context 或阻断 | `turn.completed`；回答在 `item.completed` 的 `agent_message`，也在会话 jsonl 的 assistant `output_text` | 单次 `-c` 覆盖配置、不写文件。钩子在 `~/.codex` 或项目 `.codex`，要信任 | `thread/resume`、`codex exec resume`、`codex resume`、`codex resume --remote` | exec 没有 pane。`codex resume --remote` 连到同一个 app-server 时，界面会画出另一个客户端的 `turn/start`。不带 `--remote` 的 TUI 仍是输入框 | `codex` |
| Grok Build 1.0.46 | 有。`grok -p` 与 `grok agent stdio` 的 `session/prompt` | 未查到能替换正文的输出字段。钩子能追加上下文、拒绝工具、拦住 Stop | headless 的 json / streaming-json；ACP 的 `session/update` | 单次 `--allow` / `--deny` / `--cwd`。钩子目录是 `~/.grok/hooks` 与项目 `.grok/hooks` | `--resume`、`-c`、ACP `session/load` | `grok agent stdio` 官方文档说是没有交互界面的 ACP。TUI 另开 | `grok`、`grok-build` |
| Antigravity CLI `agy` 1.2.17 | 有。`--print` / `--prompt`，stdin 可作 `stream-json` | 未查到 | `--output-format json` 或 `stream-json` | 单次 `--model`、`--mode`、`--conversation`。配置根未查到 | `--continue`、`--conversation <id>` | `--print` 的帮助写明非交互。交互 TUI 是默认路径 | `agy`、`antigravity` |
| Kimi Code 2.1.1 | 有。`kimi -p` 与 `kimi acp` | 未查到替换字段。`UserPromptSubmit` 官方文档说返回文本附加到上下文，或阻断本轮 | `Stop`；`--output-format stream-json` 的 Assistant 消息；另有 `TurnStarted` | `KIMI_CODE_HOME` 可整份挪走。钩子默认在用户配置。单次 `--skills-dir`、`--model` | `--session`、`-c` | `-p` 与 `acp` 的帮助都写明不开 TUI | `kimi` |
| Qoder CLI CN `qoderclicn` 1.1.65 | 有。`-p` / `--print` | 未查到。本机有 `hooks` 子命令，只看到从 Claude Code 迁移 | `-o` / `--output-format`。事件名未查到 | `--config-dir` 换这一次的用户配置根。`--disallowed` 一类限制未在帮助里写成不落盘 | `--resume`、`-c`、`--session-id` | `--print` 写明非交互 | `qoderclicn`、`qodercli` |
| omp 18.6.1 | 有。`-p` 与 `--mode rpc` | 未查到替换语义。`--hook` 可按次加载文件 | `--mode json` 或 `rpc`。子代理事件未查到 | `--profile` 隔离一份状态；`--config` 只叠这一次；`--hook` 按次加载 | `-r` / `--resume`、`-c` | `--mode rpc --no-ui` 写明无界面。默认是 TUI | `omp` |
| OpenCode 1.18.34 | 有。`opencode serve` 的 `POST /session/:id/message`，以及 `opencode run`、`opencode acp` | 插件能在工具执行前抛错。未查到改写用户正文的钩子 | HTTP 响应与会话消息。`session.idle` 是插件事件，不是正文本身 | `OPENCODE_CONFIG_CONTENT` 或 `OPENCODE_CONFIG_DIR` 可指到一次启动的目录 | 会话 id、`POST /session/:id/fork`、`opencode import` / `export` | `serve` 与 `acp` 无 TUI。`opencode` 默认是 TUI，另有 `/tui/submit-prompt` 走输入框 | `opencode` |
| ZCode CLI 0.16.9 | 有。`zcode app-server` 与 `-p` / `--prompt` | `UserPromptSubmit` 能阻断或加上下文。官方 README 没有替换正文的字段 | app-server 的 `turn.completed` / `turn.failed`；`--surface` 可选 terminal 或 desktop | `--disallowed-tools` 帮助写明只对这一次，不改已存设置。钩子在 `~/.zcode/cli/config.json` | 官方 README 有 resume；社区 ACP 桥描述 `session/resume` | `app-server` 是 stdio 协议。`--surface terminal` 才把无 TUI 的提示画到终端 | 未查到 |
| MiniMax Code `mcode` 0.6.3 | 有。`mcode exec` 与 `mcode acp` | 未查到 | exec 的 stdout。ACP 事件名未查到 | `--config`、`--system-prompt`、`--model` 都写明只对这一进程 | `--session`、`--continue`；exec 也可用 `--session` | `exec` 与 `acp` 写明不开 TUI。默认命令开 TUI | 未查到 |
| Pi | 本机未安装 | 未查到 | 未查到 | 未查到 | 未查到 | 未查到 | `pi` |

## 逐家

### Codex CLI

本机 `codex-cli 0.160.1`。实测只跑了一轮，工作目录 `/tmp/hctl-dispatch-probe`，没有改 `~/.codex`。

命令是 `codex exec --skip-git-repo-check --sandbox read-only --json`，参数正文为 `Reply with exactly HCTL_DISPATCH_OK and nothing else.`。进程退出码 0。stdout 四条 JSON：`thread.started`、`turn.started`、`item.completed`（`item.type` 为 `agent_message`，`text` 与 `HCTL_DISPATCH_OK` 逐字相同）、`turn.completed`。会话记录写在 `~/.codex/sessions/2026/10/06/` 下当天新的 `rollout-*.jsonl`。其中任务那条 `response_item` / `message` / `user` 的 `input_text` 与命令行参数逐字相同；另有一条更早的 user 消息是 `<environment_context>`，不是任务正文。assistant 的 `output_text` 是 `HCTL_DISPATCH_OK`。记录里还有 `event_msg` / `task_complete`。这次没有子代理，子代理字段未在这条记录里出现。

这次 exec 的标准输入不是 TTY，CLI 打过一行 `Reading additional input from stdin...`。落盘的任务 `input_text` 没有多出字符。回答不拿模型自述当证据，拿的是上述 `output_text` 与 stdout 的 `agent_message`。

1. **交法。** 实测：正文是 exec 的参数，不是在 TUI 输入框里敲的。官方文档说 app-server 用 `thread/start` 打开线程，再用 `turn/start` 提交 `user input`。源码里看到 `UserPromptSubmit` 的命令输出 schema 只有 `additionalContext`、阻断用的 `decision: block` 和通用的 `continue`；没有替换 `prompt` 的字段。官方文档说纯文本 stdout 会变成额外的 developer context。所以 Codex 做不到 Claude 那种「提交前把标记换成正文」。
2. **结束与回答。** 实测见上。子代理：官方文档有 `SubagentStart` / `SubagentStop`，matcher 看 `agent_type`。这次记录里没有子代理事件。
3. **一次启动。** 本机帮助：`-c` 覆盖本会从 `~/.codex/config.toml` 读到的值，不写回文件。钩子文件在用户目录或项目 `.codex`，非托管命令钩子要先信任。本机 `codex plugin --help`（0.160.1）的子命令是 `add`、`list`、`marketplace`、`remove`：从 marketplace 安装，或卸载并删掉本地缓存。没有看到只给这一次加载、又能改写已提交正文的子命令。
4. **续接。** 官方文档说 `thread/resume` 按线程 id 接着写。本机帮助有 `codex exec resume` 与 `codex resume`。`codex resume` 也接受 `--remote`。exec 那一轮没有再开第二次对话；`--remote` 的续接见下面。
5. **pane。** 实测：exec 写完 JSON 就退出，没有交互界面。不带 `--remote` 的 TUI 仍把人敲的字收进输入框；Herdr 的 `agent.prompt` 走的是这条。把 TUI 连到 app-server 之后，界面和 `turn/start` 可以是同一条线程，见下面。

**`--remote` 与 daemon。** 本机帮助（0.160.1）：`codex --remote <ADDR>` 把 TUI 连到 app-server，地址可以是 `ws://`、`wss://`、`unix://`、`unix://PATH`。`codex agents` 的帮助写着浏览共享的本地 app-server daemon 上的会话，它也接受 `--remote`。这次没有另开 `codex agents` 的界面。`codex app-server daemon` 的子命令有 `bootstrap`、`start`、`restart`、`update`、`enable-remote-control`、`disable-remote-control`、`stop`、`version`。`bootstrap` 的帮助写着为 SSH 场景安装持久的本地管理，可选 `--remote-control`。`start` 的帮助写着若尚未运行则启动。官方文档把 `codex app-server --listen` 配 `codex --remote` 写成连上 TUI 的做法，并写明 app-server 主要用于开发和调试，可能不经通知就变。

1. **另一个客户端 `turn/start`，界面显示不显示。** 实测：显示。一次性 `codex app-server --listen ws://127.0.0.1:<port>`，`CODEX_HOME` 指到 `/tmp` 下的临时目录，里面只有复制来的 `auth.json` 和 `config.toml`，测完删除。先 `thread/start`（`sandbox` 为 `read-only`，`approvalPolicy` 为 `never`）。这一刻还没有 rollout：`codex resume <threadId> --remote` 退出，错误是 `thread/resume failed: no rollout found`（JSON-RPC -32600）。再由第一个客户端 `turn/start`，正文要求只回 `HCTL_REMOTE_A`。这一轮的 RPC 收到 `turn/completed`，临时目录里出现一份 rollout。然后 `codex resume <threadId> --remote ws://127.0.0.1:<port>` 留在界面上，屏幕上有第一轮的用户正文和 `HCTL_REMOTE_A`。另一个 WebSocket 客户端对同一 `threadId` `turn/start`，正文要求只回 `HCTL_REMOTE_B`。界面上出现了这句用户正文，状态行是 Working，也出现了 `HCTL_REMOTE_B`；窗口标题一度是 `Return HCTL_REMOTE_B`。第二客户端这条连接上没有读到 `turn/completed`。空的 `codex --remote`（不 resume）能停住，当时 `thread/list` 是 0 条，没有对那个空界面发 `turn/start`。`codex resume --remote` 再加 `--sandbox` 会被拒绝：`Permission overrides are not supported when resuming a remote task.` 沙箱写在 `thread/start` 和 `turn/start` 上。线程 id 是 `01a10e6f-27fb-7232-a382-1890d21314c4`。
2. **Herdr 认不认 `--remote`。** 实测：认进程名。resume 那次的 `comm` 是 `codex`，参数是 `codex resume <threadId> --remote ws://127.0.0.1:<port>`。另一次空的 `codex --remote ws://127.0.0.1:<port>` 也是 `comm` 为 `codex`，并且一直活着。源码里看到 Herdr 0.9.3（`7b116c05`）的 `lookup_agent` 只认 basename `codex`，没有按 `--remote` 排除。没有把这个进程放进 Herdr pane，屏幕状态规则未实测。
3. **daemon 怎么起、装在哪、动不动全局配置。** 本机已经有一个托管 daemon。这次没有 `start`、`stop`、`bootstrap`，也没有 `enable-remote-control`。`codex app-server daemon version` 打印 `status` 为 running，`backend` 为 `pid`，`managedCodexPath` 为 `~/.codex/packages/app-server-daemon/current/bin/codex`，版本 0.160.1，socket 为 `~/.codex/app-server-control/app-server-control.sock`。进程命令行是该包 `releases/0.160.1-aarch64-apple-darwin/bin/codex app-server --listen unix:// --managed-daemon`，同目录还有 `pid-update-loop` 和 `codex-code-mode-host`。`~/Library/LaunchAgents` 里没有名字带 codex 或 openai 的 plist。界面那一轮连的不是这个托管进程，而是临时 `CODEX_HOME` 上的一次性 `--listen`。跑完后真实 `~/.codex` 的 `config.toml`、`auth.json`、`hooks.json`、`.codex-global-state.json` 哈希未变，`sessions/2026/10/06` 没有新的 rollout，托管进程的 pid 未变。`bootstrap` 会不会改全局配置：未实测。帮助只说明它要安装持久管理。

### Grok Build

本机 `grok 1.0.46`。没有开新会话。

1. 官方文档说 `grok -p` 把一条提示送进无界面运行；`grok agent stdio` 是 ACP，`session/prompt` 送用户消息，正文在协议参数里。本机帮助有 `agent` 子命令，写明不带交互界面。
2. 官方文档说 `--output-format json` 结束时给一个对象，`streaming-json` 按行给 ACP `session/update`。源码文档列出 `Stop`：可以 `decision: block` 把理由喂回模型，或 `continue: false` 结束。`SubagentStop` 单独列出。`--no-subagents` 在本机帮助里。dashboard 帮助写明子代理会话分开列。
3. 钩子从 `~/.grok/hooks/*.json` 与项目 `.grok/hooks/` 合并，项目钩子要信任。未查到一次启动就换掉提交正文的开关。单次权限用 `--allow` / `--deny`。
4. 官方文档与本机帮助：`--resume`、`-c`、ACP `session/load`。
5. `grok agent stdio` 没有 TUI。普通 `grok` 是 TUI。Herdr 认 `grok` 与 `grok-build`。

### Antigravity CLI

本机 `agy 1.2.17`。没有开新会话。

1. 本机帮助：`--print` / `--prompt` 非交互；`--input-format stream-json` 从 stdin 按行读 NDJSON，每一行一轮，且必须配 `--output-format stream-json`。这是把正文当数据。未查到改写提交文字的钩子。
2. `--output-format json` 或 `stream-json`。子代理事件未查到。
3. `--model`、`--mode`、`--agent`、`--conversation` 都写在「当前 CLI session」。配置文件位置未查到。
4. `--continue` 与 `--conversation <id>`。
5. `--print` 不开交互界面。默认无子命令时帮助没有写成协议服务器。Herdr 认 `agy` 与 `antigravity`。

### Kimi Code

本机 `kimi 2.1.1`。没有开新会话。官方页面日期记为 2026-09-24 读到的文档。

1. 本机帮助与官方文档：`kimi -p` 不开 TUI，正文是参数。`kimi acp` 是 stdio 上的 ACP。`UserPromptSubmit` 官方文档说：返回的文本附加到上下文，阻断则本轮不调用模型。未查到替换 `prompt` 的字段。
2. `Stop` 在模型准备结束本轮时触发，阻断可以再补一条让它继续。`-p --output-format stream-json` 每行一个 JSON，普通回复是 Assistant 消息。官方事件表另有 `TurnStarted`（含 `turn_id` 与 `prompt`）和 `SubagentStart` / `SubagentStop`（带 `agent_name`）。
3. 既有调研记过 `KIMI_CODE_HOME` 可整份挪走。本次没有改那个目录。单次有 `--skills-dir`、`--model`、`--agent-file`。
4. `--session [id]` 与 `-c`。
5. `-p` 与 `acp` 都不是 TUI。Herdr 认 `kimi`。

### Qoder CLI CN

本机 `qoderclicn 1.1.65`。没有开新会话。

1. 帮助写明默认交互，`-p` / `--print` 非交互，初始 query 是参数。
2. `--output-format` 存在。具体事件名未查到。子代理未查到。
3. `--config-dir` 换这一次的用户配置根。`hooks` 子命令只看到 `migrate`（从 Claude Code 迁钩子）。钩子能不能改写正文未查到。
4. `-r` / `--resume`、`-c`、`--session-id`、`--fork-session`。
5. `--print` 非交互。Herdr 把 `qoderclicn` 认成 `qodercli`。

### omp

本机 `omp 18.6.1`。帮助里有 `PI_SMOL_MODEL` 一类环境变量，和 Pi 同一族。没有开新会话。

1. `-p` 非交互。`--mode rpc` 或 `rpc-ui`，可加 `--no-ui`。位置参数是要发送的消息。
2. `--mode json` 或 `rpc`。子代理事件未查到。
3. `--profile` 隔离 auth、会话、设置和缓存。`--config` 只为这一次叠加。`--hook` 可重复，按次加载文件，不要求先写入全局目录。钩子能不能改写正文未查到。
4. `-r` / `--resume`、`-c`。另有 `--from-claude` 导入，不是 omp 自己的续接。
5. 默认是交互。`rpc --no-ui` 无界面。Herdr 认 `omp`，文档写状态要装集成。

### OpenCode

本机 `opencode 1.18.34`。没有开新会话。服务器页面为 2026-10-03 的官方文档。

1. `opencode serve` 后 `POST /session/:id/message`，body 的 `parts` 是正文。`opencode run [message..]` 与 `opencode acp` 也是参数或协议，不是 TUI 输入框。`POST /tui/append-prompt` 与 `/tui/submit-prompt` 才碰输入框。插件钩子 `tool.execute.before` 能中止工具。未查到改写用户正文的钩子。
2. 发送消息的 HTTP 调用等到回复；会话消息可再 `GET`。插件 `event` 能收到 `session.idle`。子会话有 `GET /session/:id/children`。子代理的结束事件名未查到。
3. 既有调研记过 `OPENCODE_CONFIG_CONTENT` 与 `OPENCODE_CONFIG_DIR`。本次没有写用户配置。
4. 同一 session id 再发消息即续接。另有 fork、export、import。
5. `serve` 与 `acp` 无 TUI。`opencode` 默认是 TUI。Herdr 认 `opencode`。

### ZCode

本机 `zcode 0.16.9`。没有开新会话。钩子段落来自 `zai-org/ZCode` 仓库 `apps/zcode-cli/README.md`（2026-10-06 读到的 main）。

1. `zcode app-server` 是 stdio 协议。`-p` / `--prompt` 不开 TUI。社区桥把 ACP `session/prompt` 译成 app-server 的发送。这是数据，不是输入框。
2. 同一 README 与桥的说明：`turn.completed`、`turn.failed`，流里还有 `model.streaming` 与 `tool.updated`。`UserPromptSubmit` 在正文写入历史之前运行，可以 `continue: false` 阻断，或加上下文。没有替换正文的字段。子代理未查到与主轮事件的区分字段。
3. `--disallowed-tools` 帮助写明只对这一次提示或 TUI，不改已存设置。钩子在 `~/.zcode/cli/config.json`，默认关闭。
4. README 写了 resume。社区桥写 `session/resume`。精确 CLI 旗标未在本次帮助摘要里单列，记为官方 README 说有，本机帮助摘要未再展开。
5. `app-server` 不是 TUI。`--surface terminal` 或 `desktop` 只影响无 TUI 提示的呈现。Herdr 0.9.3 的进程名表里未查到 `zcode`。

### MiniMax Code

本机 `mcode 0.6.3`。没有开新会话。

1. `mcode exec` 不开 TUI，提示是参数或 `--input -`。`mcode acp` 是 stdio ACP。未查到改写提交文字的钩子。
2. exec 把结果打到 stdout。ACP 的具体通知名未查到。子代理未查到。
3. `--config`、`--system-prompt`、`--model`、`--effort` 都写明只对这一进程或这一 Run。
4. `--session`、`--continue`。exec 也可 `--session <id>` 接到已有会话。
5. exec 与 acp 不开 TUI。无子命令时开 TUI。Herdr 0.9.3 进程名表里未查到 `mcode`。

### Pi

本机 `PATH` 上没有 `pi`。五个问题里交法、结束、一次启动、续接都未查到。Herdr 0.9.3 认进程名 `pi`，集成安装名也是 `pi`。omp 的帮助带 Pi 的环境变量，不能把 omp 的旗标写成 Pi 本体已核实。

## 决定建议

第二家 harness（Codex）不要照搬 Claude 的「输入框里交标记，再用钩子换成正文」。Codex 的 `UserPromptSubmit` 换不了正文，只能追加 developer context 或阻断。正文走 app-server 的 `turn/start`，或等价的 `codex exec`：参数就是正文。exec 那一轮里，会话记录的 `input_text` 与参数逐字相同，结束看 `turn.completed` 和 assistant 的 `output_text`。

交互 pane 可以接到同一条线程上，不必再做一条安装命令。本机实测：一次性 `codex app-server --listen` 上，第一轮 `turn/start` 把 rollout 写出来之后，`codex resume <threadId> --remote` 的界面画出了另一个客户端随后的 `turn/start`。进程名仍是 `codex`，Herdr 0.9.3 按这个名字认。建议 pane 里跑连着这个 app-server 的 `codex --remote`（要看已经打开的线程就用 `codex resume <id> --remote`），正文由另一个客户端 `turn/start`。`herdr integration install codex` 仍只负责会话身份和恢复。沙箱写在 `thread/start` 和 `turn/start` 上；`resume --remote` 不接受 `--sandbox`。线程要等第一次 `turn/start` 写出 rollout，之前 `resume --remote` 会因找不到 rollout 退出。

默认启动仍是分开的。`herdr agent start --kind codex` 拉起的是不带 `--remote` 的 TUI，`agent.prompt` 仍进输入框。这次没有测那个不带 `--remote` 的界面会不会看到 app-server 上的轮次。也不必为了交正文去跑 `daemon bootstrap`：一次性 `--listen` 就够，而且没有改真实的 `~/.codex` 配置。本机那个已经在跑的托管 daemon 装在 `~/.codex/packages/app-server-daemon`，这次没有往它里面交 turn。

这是建议，不代替所有者决定开哪一包、谁写。

## 证据与版本

| 对象 | 版本 | 档 | 位置 |
| --- | --- | --- | --- |
| Codex CLI | 本机 0.160.1 | 实测 | 2026-10-06，一轮 `codex exec --json`；会话 `rollout-2026-10-06T06-07-45-*.jsonl` 的 user `input_text` 与参数逐字相同 |
| Codex app-server / hooks | 文档页 2026-10-06 读取；schema 为仓库 main 上的 `user-prompt-submit.command.output.schema.json` | 官方文档说；源码里看到 | [App Server](https://developers.openai.com/codex/app-server)、[Hooks](https://developers.openai.com/codex/hooks)、[schema](https://github.com/openai/codex/blob/main/codex-rs/hooks/schema/generated/user-prompt-submit.command.output.schema.json) |
| Codex `--remote` / daemon | 本机 0.160.1；Herdr 源码 `7b116c05` | 实测；源码里看到；官方文档说 | 2026-10-06。临时 `CODEX_HOME` 上一次性 `--listen`。线程 `01a10e6f-27fb-7232-a382-1890d21314c4`：第一轮 `turn/start` 之后 `codex resume --remote` 的界面画出第二客户端的 `HCTL_REMOTE_B`。托管 daemon 只读了 `daemon version` 和进程命令行，没有 bootstrap。`codex plugin --help` 见 Codex 一节第 3 问 |
| Grok Build | 本机 1.0.46；文档为 `xai-org/grok-build` main 与 docs.x.ai | 本机帮助；官方文档说 | [headless](https://docs.x.ai/build/cli/headless-scripting.md)、[hooks 指南](https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/custom-hooks.md) |
| Antigravity CLI | 本机 `agy` 1.2.17 | 本机帮助 | 钩子与配置根未查到 |
| Kimi Code | 本机 2.1.1；文档页标 2026-09-24 | 本机帮助；官方文档说 | [Hooks](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/hooks.html) |
| Qoder CLI CN | 本机 `qoderclicn` 1.1.65 | 本机帮助 | 钩子语义未查到 |
| omp | 本机 18.6.1 | 本机帮助 | 钩子替换语义未查到 |
| OpenCode | 本机 1.18.34；文档 2026-10-03 | 本机帮助；官方文档说 | [Server](https://opencode.ai/docs/server/) |
| ZCode | 本机 0.16.9 | 本机帮助；源码仓库 README | [ZCode CLI README](https://github.com/zai-org/ZCode/blob/main/apps/zcode-cli/README.md) |
| MiniMax Code | 本机 `mcode` 0.6.3 | 本机帮助 | ACP 事件名未查到 |
| Pi | 未安装 | 未查到 | 只确认 Herdr 认进程名 |
| Herdr | 0.9.3，`7b116c05bfda646af39d2524c54e70c751f57ee8` | 源码里看到 | `src/detect/mod.rs` 的 `Agent` 与 `lookup_agent` |

实测没有改任何一家的全局配置，没有打印凭据，没有碰 `~/.local/state/hctl2`。Codex 的 `--remote` 一轮用的是临时 `CODEX_HOME`；真实 `~/.codex` 的 `config.toml`、`auth.json`、`hooks.json`、`.codex-global-state.json` 哈希未变。Codex 以外没有开登录会话。

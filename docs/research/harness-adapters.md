# 三家 Harness 的无界面接入与观测

> 状态：调研 · 日期：2026-09-06<br>
> 类别：① Coding Harness · 证据编号：E-L1-HARNESS-ADAPTERS<br>
> 对象：Claude Code `2.1.263`、Codex CLI `0.153.4`、Gemini CLI `0.58.0`；运行时仍为 Herdr `v0.8.2 / 9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c`<br>
> 许可证：Claude Code 按 Anthropic 条款，npm 标 `SEE LICENSE IN README.md`；Codex CLI、Gemini CLI、Herdr 为 Apache-2.0

## 定位

本地 Agency（派出方）交付执行体时，用 Harness（编码代理程序）适配器核对启动规格、会话身份与观测。三家同时接入已定，本文不再选一家淘汰另外两家；只补[工具调用前钩子研究](./harness-hooks-20260903.md)没有回答的无界面命令、事件、终局和恢复。

共用的是启动核验、身份关联、权限处理、观测归一和终局检查；不是强行让三个私有协议长成一样。进程与终端仍交 Herdr，领域结果仍按[Participant §运行时与观测](../design/spec/participant.md#运行时与观测)准入。

## 上游能力

### 启动参数与协议

JSONL 表示一行一个 JSON 对象；这里的流是机器接口，不是终端屏幕抓取。下表命令展示已核对的参数，模型、工作目录、配置文件与会话 ID 从冻结规格填入，提示正文可通过 stdin 传入，不写进 shell 拼接串。

| 家 | 无界面观察入口 | 需要来回处理审批时 | 稳定性与版本核对 |
| --- | --- | --- | --- |
| Claude Code 2.1.263 | `claude -p --output-format stream-json --verbose --permission-mode manual`；增量片段另加 `--include-partial-messages` | 二进制已有 `--permission-prompt-tool` 指向 MCP 审批工具；`--permission-prompts host` 交 SDK host / 该工具处理。SDK 的 `canUseTool` 回调是另一条现成路径 | CLI 与 SDK 有公开类型，但流没有独立协商的协议版本；钉 CLI 版本及消息样本。2.1.263 的 help 列 `manual`，官方说明它是 `default` 的别名，二者都接受 |
| Codex CLI 0.153.4 | `codex exec --json --color never --sandbox workspace-write -C <工作树>`；只读调用换成 `read-only` | 同一二进制的 `codex app-server --listen stdio://`，或本地 Unix socket；协议的 `item/commandExecution/requestApproval`、`item/fileChange/requestApproval` 等可以请求并接收回答 | `exec` 的 JSONL 与 app-server 的 JSON-RPC 是两种入口；后者能由钉定二进制导出 schema。CLI help 仍标 app-server experimental，不能据此承诺跨版本不变 |
| Gemini CLI 0.58.0 | `gemini -p <提示> --output-format stream-json --approval-mode default` | 同一程序的 `gemini --acp --approval-mode default`；ACP（Agent Client Protocol，代理与客户端的双向协议）有 `session/request_permission` | 0.58.0 的 `--experimental-acp` 已是弃用别名，正式参数为 `--acp`。JSONL 的六种事件在源码枚举，不假定与 ACP 通知同形 |

无界面不意味着无人可以回答，但**单向 JSONL 不自动提供审批回复通道**。Claude 的 `--permission-prompts none` 是自动拒绝会询问的操作，不是逐项询问；Codex 的 `exec` 不提供 app-server 那套请求/回复；Gemini 的非交互路径把 `ask_user` 判定转成拒绝。遇到这种能力缺口，不以全放行参数绕过。

### 七类事件逐项对照

下表是上述 JSONL 观察入口的形状；审批另注明双向入口。可以落进同一套七类记录，不表示每家每一类都有完整的原生证据。

| 观测类 | Claude Code | Codex CLI | Gemini CLI | 归一边界 |
| --- | --- | --- | --- | --- |
| 生命周期提示 | `system/init`、`result`；子执行另有任务通知 | `thread.started`、`turn.started/completed/failed` | `init`、`result`、`error` | 记录来源、时间、会话及轮次；不直接推进 Task / Run |
| 工具调用 | `assistant.message.content` 的 `tool_use`，`user` 中相应 `tool_result` | `item.*` 中 `command_execution`、`mcp_tool_call` 等 | `tool_use` 的 `tool_id/tool_name/parameters`，`tool_result` 的状态与输出 | 按工具调用 ID 配对，增量与完整消息去重 |
| 权限请求 | 通过审批工具 / `canUseTool`；`system/permission_denied` 是已拒绝通知，不是等回答的请求 | app-server 的 `…/requestApproval`；不把 `exec` 的错误当请求 | ACP 请求；JSONL 六类中没有独立权限请求 | 请求与授权答复相关联；没有此通道就声明不支持相应交互 |
| 文件变化 | 从 Edit / Write 等工具参数、结果观察 | 原生 `file_change`，含文件与变更信息 | 从 `replace` / `write_file` 等工具参数、结果观察；ACP 可带 locations | shell、副执行及外部程序也会改文件，这些事件均不能代替工具箱 Git 树回读 |
| 测试 | 工具执行输出 | 命令执行输出及退出状态 | 工具执行输出 | 三家均非独立测试证据服务；只能标测试观察或原始输出，正式证据由工具箱回读 |
| 用量 | result 的 `usage`、`modelUsage`、费用等 | `turn.completed.usage`；app-server 另有 token usage 通知 | `result.stats` 的 token、时长、tool_calls 与按模型统计 | 字段缺失记未知，缓存 token 与总量不重复累加；不混称统一计费金额 |
| 原始输出 | 完整消息、可选 `stream_event` 增量、诊断 stderr | item 内容、错误和诊断 stderr | `message` 的 `role/content/delta`、`error` | 未知类型保留原件后降级，不丢掉子执行归属 |

Claude 子代理消息可带 `parent_tool_use_id`，Codex 的多执行体信息、Gemini 的子会话也要保留上游提供的关联；JSONL 没给完整谱系时记观测缺项，不摊平进主流伪称完整。

### 终局清单

| 入口 | 必须等到的本轮终局 | 不足以判成功的信号 |
| --- | --- | --- |
| Claude print | 同一 `session_id` 的 `type=result`，核 `subtype` 与 `is_error`；成功、限额、执行错误分开 | 一条 assistant 文本、Stop 提示、退出码 0；result 之后仍可有尾随系统消息，继续收完 |
| Codex exec | `turn.completed` 或 `turn.failed`；前者仍只表示这轮执行完成 | `item.completed` 只结束一个 item，`error` 不一定是完整终局；最后消息文件不代替事件 |
| Codex app-server | 对应 thread / turn 的 `turn/completed`，核 turn.status 与 error | RPC 接受 `turn/start`、某个 item 完成 |
| Gemini print | `type=result`，核 `status=success/error`；正文通常在前面的 `message`，不要求 result 里有回答文本 | `error` 可能只是 warning；进程正常退出或一条 assistant message |
| Gemini ACP | 同一次 `session/prompt` 的响应及 `stopReason`，如 `end_turn` / `cancelled` / 限额 | `session/update` 的文字通知 |

正常退出却缺必需终局事件，合成类型化协议错误；主动取消归为取消；观察通道断掉则标截断。上述都是适配器的执行结果，不是领域成功凭证。Herdr 的 pane 退出信息不含退出码，且 Harness 退出后 shell 可能仍活着，不能把 pane 状态补成退出证明。

### 会话、转录与环境

| 家 | 会话关联与恢复 | 转录位置与注入方式 |
| --- | --- | --- |
| Claude | `system/init`、result 给 `session_id`；可先指定 `--session-id <UUID>`，恢复用 `--resume <ID>` | 默认 `~/.claude/projects/<encoded-cwd>/<ID>.jsonl`，长目录名有截断与 hash；优先用 hook 的 `transcript_path`，不自写路径猜测。`CLAUDE_CONFIG_DIR` 会连同配置、认证、插件和历史一起换根；`--settings` 更适合单次追加配置 |
| Codex | `thread.started.thread_id`；`codex exec resume <ID>` 或 app-server `thread/resume` | 常规文件在 `CODEX_HOME/sessions/` 的日期子目录；app-server `thread/read` 可回 `path`，schema 明标该路径不稳定且可空。用 thread ID 对照记录；启用 `--ephemeral` 后不具备这条文件恢复路径 |
| Gemini | JSONL `init.session_id`；ACP `session/new/load` 管会话；恢复按精确会话选取，不用 `latest` 猜 | 0.58.0 的 `chatRecordingService` 写 **JSONL**：`<GEMINI_CLI_HOME 或用户目录>/.gemini/tmp/<project-identifier>/chats/session-<时间>-<ID前8位>.jsonl`；子执行另按完整父会话 ID 分目录。旧 `.json` 有兼容读取，不能沿用旧版固定文件名假设 |

Herdr 0.8.2 的 `workspace.create`、`tab.create`、`pane.split` 都有 `env: HashMap<String,String>`，CLI 对应重复 `--env KEY=VALUE`；可传各家的配置路径与 HCTL 的精确运行时关联。只对**新建**进程生效，不改已活跃会话的权限。`CODEX_HOME` / `CLAUDE_CONFIG_DIR` 换根会同时搬走认证与 Herdr 已装钩子，不能只复制一份设置就宣称集成保留；优先单次设置或在交付前核验有效配置与 hook 信任。

Gemini 的单次配置用 `GEMINI_CLI_SYSTEM_SETTINGS_PATH` 指向 control 生成的设置文件；需要策略文件时用上游 `--policy` / `--admin-policy` 机制，核实际优先级，不放一套平行权限系统。`--env` 只是传值，不证明策略已加载。JSONL / ACP / app-server 字节需由其原生通道接收，Herdr 的 `pane.read` 是有界屏幕观察，不能代替完整协议流。

### 「逐项询问」的实际限制

| 家 | 可钉的基础配置 | 还需核验的部分 |
| --- | --- | --- |
| Claude | `--permission-mode manual`，显式 ask 规则覆盖受审工具；审批回到 host / MCP 工具，每次仅回本次决定 | Manual 仍自动放行部分只读工具；既有 allow、hook 与托管策略参与判定。`canUseTool` 不覆盖已自动批准的操作；`--bare` 会跳过需要的钩子 |
| Codex | app-server `approvalPolicy="untrusted"`、`approvalsReviewer="user"`，按当前 request ID 回一次性决定；同版本的规则与已信任钩子负责细化 | untrusted 不是所有工具都询问，on-request 更不是；托管工具不全受 PreToolUse 覆盖。未获信任的 hook 会被跳过，不能以「文件存在」算生效 |
| Gemini | `--acp --approval-mode default`，受审工具选 `ask_user`，每次选 `allow_once` 对应的上游选项而非永久授权 | print 模式遇 ask_user 会拒绝；默认只读允许，源码还对部分安全 shell 命令作放行判断，笼统的一条 ask 规则不足以证明逐次拦截；应按实际工具与参数复核 |

**推翻「三家只钉一个权限模式参数，就都能逐项询问」的假设；不推翻三家接入。** 若「逐项」指每个工具调用都经人确认，本次没有证据能为三家作这个统一承诺。需要审批的具体动作及其实际覆盖必须随冻结规格核验，不能自行把它缩成「危险命令才问」，也不能用自动批准或绕过沙盒替代缺失的请求通道。这里登记能力限制，不改约束层。

## 候选比较

三家是同批接入对象；下表比较借用的程序与接口，不另建代理执行引擎。MSRV 是最低 Rust 编译器版本；我们调用上游程序，不把其源码编进 control，因此这里均不适用，第一方适配器仍用本库 1.98.0。

| 对象 | 钉定版本 / 许可证 | 发布与宿主依赖 | 取舍 |
| --- | --- | --- | --- |
| Claude Code | 2.1.263 / Anthropic 条款 | 有官方原生程序；npm 启动包声明 Node ≥22 | 适配已获授权安装的 CLI / 公开接口，不移植闭源实现；不因可下载就推定能随包再分发 |
| Codex CLI | 0.153.4 / Apache-2.0 | 原生 Rust 程序；npm 启动包 Node ≥16 不等于原生程序运行要 Node | 同一程序已带观察和双向协议，无须另造代理服务 |
| Gemini CLI | 0.58.0 / Apache-2.0 | npm / 官方 JS bundle 用 Node ≥20；该 release 另有 macOS arm64/x64 **unsigned** 程序，没有同批 Linux 原生 asset | 适配 CLI / ACP；不把未签名 Mac 资产当成三平台已可交付的二进制方案 |
| Herdr | 0.8.2 / Apache-2.0，协议 20 | 已随包的官方程序 | 维持运行时角色；它的 Agent 状态检测不代办 Harness 终局协议 |

## 边界与取舍

版本是本次复核基线，不自动升级用户已装的 harness，也不改变发布 lock。本次做了以下无模型调用的检查，未触发真实编码会话或产生模型费用：

| 本机检查（macOS arm64，2026-09-06） | 结果 |
| --- | --- |
| `claude --version/--help`、`codex --version`、`codex exec/app-server --help` | 分别为 2.1.263、0.153.4；参数以本机程序和官方文档交叉核对 |
| `codex app-server generate-json-schema --out <临时目录>` | 成功；导出 schema 确有三类 `…/requestApproval`、approvalPolicy、approvalsReviewer 及 thread.path；没有启动模型 |
| Gemini 0.58.0 官方 unsigned arm64 程序 `--version` | 退出 137，无版本输出；`codesign --verify` 报未签名。没有改签或放宽宿主校验，不能写成原生程序通过 |
| 同版本官方 JS bundle，经 Node 26.8.1 执行 `--version/--help` | 通过，输出 0.58.0 与 `--acp` 等参数；未使用机器原先安装的 0.46.0 冒充新版本 |
| 同 bundle，独立 `GEMINI_CLI_HOME`、`--list-sessions`，系统设置路径指向故意截断的 JSON | 退出 52，错误明确指向指定文件；改指向有效 `{}` 后越过解析，退出 41，提示独立 home 尚未配置认证。**只证明换路径及读取有效，不是登录或 hook 执行通过** |

Gemini 实验输入是 v0.58.0 的 `gemini-cli-bundle.zip`，SHA-256 `c4e5914239a247091bd0ce2af782f4524a7d69f733e8a799b2b7a126272837ab`；未签名 arm64 zip 为 `2f3708ab95187215db759440eb010ea20162a4268f35b0230e7ce3ccdc45fb86`。其余事件映射基于官方协议文档与钉定源码。每种接入在 P2.3 激活前仍需真实会话验证：审批被拒、取消、缺终局、通道断开及精确会话恢复；没有以源代码核对代替这些实验。

会话标识不等于模型标识。署名读取这次实际会话的模型与 effort 证据，并与冻结规格核对，不从全局默认值或文件修改时间猜。转录能恢复谈话，不证明仍是同一个进程、同一物理代次；缺退出证据照约束报告恢复等级或丢失。

## 决定建议

**维持三家共用一个适配器骨架，借用等级为适配协议；复核版本钉 Claude Code `2.1.263`、Codex CLI `0.153.4`、Gemini CLI `0.58.0`，运行时仍用 Herdr `0.8.2`。** 共用身份核验、七类观测、终局与恢复处理，各家的启动、消息类型和权限交互按上表适配，省掉重复执行引擎。

观察入口选三家已有 JSONL；确需无界面来回审批时，优先核 Claude 二进制的审批工具接口、Codex 同二进制 app-server、Gemini 同程序 ACP，不手写另一套审批协议。**不承诺只靠 print / exec + 一个模式开关就支持逐项询问**：Gemini print 明确拒绝这种交互，另外两家也有自动批准与覆盖边界。三条双向路径尚未完成真实会话验收，不能提前激活对应能力。Gemini 配置路径已做解析反例验证；未签名 Mac 程序未运行成功，P2 接入不据此扩大随包范围。

## 证据

- Claude：[CLI 参数](https://code.claude.com/docs/en/cli-reference)、[无界面方式](https://code.claude.com/docs/en/headless)、[消息类型](https://code.claude.com/docs/en/agent-sdk/typescript)、[终局与循环](https://code.claude.com/docs/en/agent-sdk/agent-loop)、[审批回调](https://code.claude.com/docs/en/agent-sdk/user-input)、[权限规则](https://code.claude.com/docs/en/permissions)、[会话文件与恢复](https://code.claude.com/docs/en/agent-sdk/sessions)、[条款适用说明](https://code.claude.com/docs/en/data-usage)。版本 / npm 运行要求来自[2.1.263 发布元数据](https://registry.npmjs.org/@anthropic-ai/claude-code/2.1.263)。
- Codex：[官方非交互文档](https://learn.chatgpt.com/docs/non-interactive-mode)、[官方 app-server 文档](https://learn.chatgpt.com/docs/app-server)、[权限与安全](https://learn.chatgpt.com/docs/agent-approvals-security)、[官方 CLI 入口](https://learn.chatgpt.com/docs/codex/cli)。本次按 OpenAI 官方文档核对，再用本机 0.153.4 的 help 与导出 schema 复核，不从当前会话行为反推接口保证；版本与许可同时核对安装分发信息。
- Gemini 钉定源码：[CLI 参数及非交互限制](https://github.com/google-gemini/gemini-cli/blob/v0.58.0/packages/cli/src/config/config.ts)、[JSONL 类型](https://github.com/google-gemini/gemini-cli/blob/v0.58.0/packages/core/src/output/types.ts)、[策略判定](https://github.com/google-gemini/gemini-cli/blob/v0.58.0/packages/core/src/policy/policy-engine.ts)、[ACP 请求与终局](https://github.com/google-gemini/gemini-cli/blob/v0.58.0/packages/cli/src/acp/acpSession.ts)、[转录写入](https://github.com/google-gemini/gemini-cli/blob/v0.58.0/packages/core/src/services/chatRecordingService.ts)、[存储路径](https://github.com/google-gemini/gemini-cli/blob/v0.58.0/packages/core/src/config/storage.ts)、[设置读取](https://github.com/google-gemini/gemini-cli/blob/v0.58.0/packages/cli/src/config/settings.ts)。[官方 v0.58.0 资产](https://github.com/google-gemini/gemini-cli/releases/tag/v0.58.0)、[无界面文档](https://geminicli.com/docs/cli/headless/)、[策略文档](https://geminicli.com/docs/reference/policy-engine/)。
- Herdr：[workspace 环境字段](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/src/api/schema/workspaces.rs)、[pane 环境字段](https://github.com/herdrdev/herdr/blob/9eb521456ac0d19d3ab3d9d7cea3cca10baa8a4c/src/api/schema/panes.rs)、[本库运行时验证](./runtime/agency-runtime-validation-20260829.md)、[Herdr SDK 条目](./sdk/herdr.md)。

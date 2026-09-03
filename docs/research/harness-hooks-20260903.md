# 编码 Harness 的「工具调用前」钩子：八家 CLI 与 ACP 权限请求的可用性盘点

> 状态：调研 · 日期：2026-09-03<br>
> 类别：① Coding Harness · 证据编号：E-L1-HARNESS-HOOKS<br>
> 定位：Informative 研究备忘录，只回答「能不能在模型调工具之前由我们的检查程序硬拒」这一件事，不定义 HCTL2 语义；发布后正文不改，只在文末追加复核记录。复用判断沿用 [docs/research/README.md](./README.md) 的五种复用决策用语。证据分三档标注：**官方文档说**（当天读到的官方页面）、**源码里看到**（钉在具体 commit 的仓库文件）、**第三方文章称**（不作数，只备注）。版本与链接见文末[证据](#证据)。

## 定位

HCTL2 要给本地 Agency 参考实现（`src/agency/`）加一份可选的执行加固——**工具接口白名单**：模型每次调工具之前，先由我们的检查程序（`hctl-tool` 的一个子命令）按冻结规格里声明的白名单判一次，越界就拒，不给模型执行机会。白名单的格式和检查入口是我们自己的，与 harness 无关；需要 harness 配合的只有一件事：在「工具调用前」把控制权交给我们的程序，并且服从它的拒绝。

实现候选有两条路：PTY 交互模式下用各 harness 自带的钩子机制；结构化接入模式下用 ACP（Agent Client Protocol）的 `session/request_permission`。Herdr 只负责在 PTY 里拉起 harness，本身没有钩子；钩子活在 harness 自己的配置里，由参考实现在拉起前注入。所以每家 harness 都要回答同样七个问题：有没有调用前钩子；配置放在哪；能不能硬拒、决策怎么返回；钩子拿到什么输入；能不能按一次会话配置而**不往仓库工作树里写文件**（写进去会污染 ChangeSet 的 diff）；有没有 Stop 类钩子（只记录，不当门）；文档是否稳定。

## 逐 harness

先给总表，再逐家展开。总表里「按会话不动仓库」一列区分两个层次：**每会话**指配置只对这一次拉起生效、不留痕；**用户级**指写在用户 home 下、不进工作树、但对这台机器上该用户的所有会话都生效。

| Harness（钉定版本） | 调用前钩子 | 硬拒与返回方式 | 钩子输入（关键字段） | 按会话不动仓库 | Stop 类钩子 | 文档状态 |
| --- | --- | --- | --- | --- | --- | --- |
| Claude Code 2.1.259 | `PreToolUse` | 是。退出码 2（stderr 给模型）或 stdout JSON `hookSpecificOutput.permissionDecision: "deny"` + `permissionDecisionReason` | `session_id`、`tool_name`、`tool_input`、`tool_use_id`、`cwd`、`permission_mode` | **每会话**：`--settings '<内联 JSON>'`，官方文档说「只持续一个会话，不写任何文件」；`CLAUDE_CONFIG_DIR` 可整体改 home | `Stop`、`SessionEnd` | 稳定；只有 `agent` 型钩子标实验 |
| Codex CLI rust-v0.153.0 | `PreToolUse`（Bash、`apply_patch`、MCP、本地函数工具；托管工具如 WebSearch 不触发） | 是。退出码 2 或 `hookSpecificOutput.permissionDecision: "deny"` | `session_id`、`turn_id`、`tool_name`、`tool_input`、`tool_use_id`、`cwd`、`permission_mode` | **用户级**（`~/.codex/hooks.json` 或 `config.toml`），或 `CODEX_HOME` 整体重定位；**非托管钩子需先审核信任**，自动化要带 `--dangerously-bypass-hook-trust` | `Stop`、`SessionEnd`、`Interrupt` | 现行文档不标实验；有信任关卡 |
| Gemini CLI v0.58.0 | `BeforeTool` | 是。退出码 2（stderr 作为工具错误回给模型）或 stdout JSON `{"decision":"deny","reason":…}` | `session_id`、`tool_name`、`tool_input`、`cwd`、`transcript_path`、`timestamp` | **每会话可行但未实测**：`GEMINI_CLI_SYSTEM_SETTINGS_PATH` 指向一份每会话 settings；或 `GEMINI_CLI_HOME` 整体重定位 | `AfterAgent`、`SessionEnd`（尽力执行，不等待） | 稳定；项目级钩子有指纹告警 |
| OpenCode v1.18.27 | 插件钩子 `tool.execute.before`（进程内 JS/TS） | 是。钩子里 `throw` 即中止该次工具执行（源码里看到） | `tool`、`sessionID`、`callID`、`args` | **每会话**：`OPENCODE_CONFIG_CONTENT` 内联 JSON 或 `OPENCODE_CONFIG_DIR` 指向每会话目录；`plugin` 条目可以是文件路径（源码里看到，文档只写 npm） | 无专门 Stop；`event` 钩子收 `session.idle` | 插件 API 有类型定义与文档；文件路径规格未写进文档 |
| Cursor CLI（`agent`，CLI changelog 至 2026-08-26） | `preToolUse`、`beforeShellExecution`、`beforeMCPExecution` | 是。stdout JSON `permission: "deny"`（可附 `agent_message`）或退出码 2；默认 fail-open，`failClosed: true` 改为失败即拒 | `tool_name`、`tool_input`、`tool_use_id`、`cwd`、`conversation_id`（无 `session_id`） | **用户级**（`~/.cursor/hooks.json`）；每会话方式**未查到** | `stop`、`sessionEnd` | 文档完整；项目级钩子只在受信任工作区生效 |
| Kimi Code 0.40.1 | `PreToolUse`（`[[hooks]]` 表） | 是。退出码 2（stderr 为原因）或 `hookSpecificOutput.permissionDecision: "deny"`；其他非零退出码**默认放行** | `session_id`、`session_title`、`client_type`、`cwd`、`tool_name`、`tool_input` | **用户级**（`~/.kimi-code/config.toml`），或 `KIMI_CODE_HOME` 整体重定位；项目级 `local.toml` 能否放钩子**未查到** | `Stop`、`SessionEnd` | 官方页面不标 beta；镜像文档称 beta（第三方，不作数） |
| Qwen Code v0.23.0 | `PreToolUse` | 是。退出码 2（stderr 给模型）或 `hookSpecificOutput.permissionDecision: "deny"` | `session_id`、`tool_name`、`tool_input`、`tool_use_id`、`tool_call_id`、`cwd`、`permission_mode` | **每会话可行但未实测**：`QWEN_CODE_SYSTEM_SETTINGS_PATH`；或 `QWEN_CODE_HOME` | `Stop`、`SessionEnd`、`SessionDelete` | 稳定；项目级钩子要求受信任目录 |
| GitHub Copilot CLI v1.0.82 | `preToolUse` | 是。stdout JSON `permissionDecision: "deny"` + `permissionDecisionReason`（deny 时必填）；退出码 2 或任何非零退出码都算拒；**超时 fail-open** | `sessionId`、`toolName`、`toolArgs`、`cwd`、`timestamp` | **用户级**（`~/.copilot/hooks/*.json`），或 `COPILOT_HOME` 整体重定位 | `agentStop`、`sessionEnd` | 文档不标预览；插件钩子不触发的 issue 仍开着 |
| ACP（protocolVersion 1，schema-v1.21.0） | `session/request_permission`（Agent **可以**在执行工具前调，不是必须） | 是。Client 回 `outcome.selected.optionId` 选 `reject_once` / `reject_always` | `sessionId`、`toolCall.toolCallId`、`title`、`kind`、`rawInput`（可选） | 天然每会话：判决在 Client 进程里，不落任何文件 | 无；`session/update` 只是通知 | 协议 v1 稳定，只在破坏性变更时升版 |

### Claude Code

- **钩子**：`PreToolUse`，`matcher` 按工具名匹配，字母数字字符串按精确或 `|` 分隔列表匹配，含其他字符则当正则；MCP 工具名形如 `mcp__<server>__<tool>`。官方文档说所有匹配到的钩子**并行**执行，同一处理器在多个 settings 文件里重复定义只跑一次。
- **配置位置**：用户 `~/.claude/settings.json`、项目 `.claude/settings.json`、项目本地 `.claude/settings.local.json`、托管策略、插件 `hooks/hooks.json`、Skill 与子代理 frontmatter。`CLAUDE_CONFIG_DIR` 把 home 下的 settings、会话历史、插件整体挪走。命令行 `--settings <文件或内联 JSON>` 位于用户/项目/本地之上、托管之下，「能设用户 settings 能设的任何键」，「只持续一个会话，不写任何文件」；`--setting-sources user,project,local` 还能裁掉来源。
- **硬拒**：退出码 2 一定阻止，stderr 文本作为拒绝原因给模型，且「JSON 说 allow 也覆盖不了退出码 2」；退出码 0 时解析 stdout JSON，`hookSpecificOutput.permissionDecision` 取 `allow`/`deny`/`ask`，`permissionDecisionReason` 是原因；旧字段 `decision: approve/block` 仍兼容。
- **输入**：`session_id`、`prompt_id`、`transcript_path`、`cwd`、`permission_mode`、`hook_event_name`、`tool_name`、`tool_input`、`tool_use_id`。
- **多来源合并**：官方文档说「钩子条目跨 settings 层级是合并、不是替换」。这意味着我们用 `--settings` 注入的 `PreToolUse` 不会盖掉用户已有的钩子（含 Herdr 装的 `SessionStart`）。
- **stderr 去向**：退出码 0 时 stderr「只进调试日志，不进 transcript，Claude 看不到」；退出码 2 时给模型；其他非零退出码时「动作继续，transcript 显示一条 `<hook name> hook error` 通知和 stderr 的第一行」——这一行会画到屏幕上。
- **稳定性**：整套钩子不标实验，只有 `type: "agent"` 一种「实验性、可能变化」。文档没写 `.claude/settings.json` 里的钩子需要先过工作区信任对话框（只对子代理 frontmatter 钩子这么写）。

### OpenAI Codex CLI

- **钩子**：`PreToolUse`（还有 `PermissionRequest`）。官方文档说 `PreToolUse`/`PostToolUse` 覆盖 shell 命令（匹配名 `Bash`）、`apply_patch` 文件编辑（匹配 `apply_patch`、`Edit`、`Write`）、MCP 工具（`mcp__<server>__<tool>`）和本地函数工具；托管工具如 `WebSearch` 不触发。第三方文章称钩子于 v0.114（2026-03）首发时为实验特性、只拦 Bash——与当天官方页面不一致，以官方为准。
- **配置位置**：`~/.codex/hooks.json` 或 `~/.codex/config.toml` 内联 `[[hooks.PreToolUse]]`（用户层）、`<repo>/.codex/hooks.json` 或 `.codex/config.toml`（项目层，**只在项目受信任时加载**）、插件、`requirements.toml` 托管层。`CODEX_HOME` 默认 `~/.codex`，存放配置、鉴权、历史、日志。`-c key=value` 能按 TOML 字面量覆盖（文档示例含数组），但**能否用 `-c` 注入钩子表未查到**。`[features] hooks` 默认开，旧名 `codex_hooks` 已废弃。
- **硬拒**：退出码 2 且 stderr 为原因，或 stdout JSON `hookSpecificOutput.permissionDecision: "deny"` + `permissionDecisionReason`；还支持 `updatedInput` 改写参数。
- **输入**：`session_id`、`cwd`、`hook_event_name`、`transcript_path`、`model`、`permission_mode`、`turn_id`、`tool_name`、`tool_input`、`tool_use_id`。
- **信任关卡**（对我们最要紧）：官方文档说「非托管钩子运行前，Codex 要求你审核并信任这条钩子的精确定义」，信任「记在钩子当前哈希上，新的或改过的钩子标记为待审并跳过直到被信任」；用户层 `~/.codex/config.toml` 里的钩子同样要过这一关；自动化可以对单次调用加 `--dangerously-bypass-hook-trust`。信任记录存在哪个文件**未查到**。
- **稳定性**：现行官方页面不标实验、不标平台限制；最新发布说明里仍有钩子相关改动（把内置清理钩子当作 built-in、TUI 钩子活动渲染）。

### Gemini CLI

- **钩子**：`BeforeTool`，`matcher` 是正则，按工具名匹配（内置如 `run_shell_command`，MCP 形如 `mcp_<server>_<tool>`）。类型目前只有 `command`。
- **配置位置**：`settings.json` 的 `hooks` 键，四层：系统默认、用户 `~/.gemini/settings.json`、项目 `.gemini/settings.json`、系统覆盖 `/etc/gemini-cli/settings.json`，系统层最高。环境变量：`GEMINI_CLI_HOME` 是「用户级配置与存储的根目录」，`GEMINI_CLI_SYSTEM_SETTINGS_PATH`、`GEMINI_CLI_SYSTEM_DEFAULTS_PATH` 改两份系统文件的位置。没有查到加载指定 settings 文件的命令行参数。`hooksConfig.enabled` 是总开关。
- **硬拒**：退出码 2「阻止执行，stderr 成为发给代理的工具错误，轮次继续」；或退出码 0 并输出 `{"decision":"deny","reason":"…"}`。文档要求 stdout 除最终 JSON 外**一个字都不能多**。
- **输入**：`session_id`、`transcript_path`、`cwd`、`hook_event_name`、`timestamp`、`tool_name`、`tool_input`、可选 `mcp_context`。超时单位毫秒，默认 60000。
- **信任**：文档说「项目级钩子在打开不受信任项目时尤其危险」，Gemini CLI 给项目钩子做指纹，名字或命令变了会在执行前告警；`/hooks` 命令可以逐条启停。
- **稳定性**：主页不标实验；最新发布说明未提钩子。

### OpenCode

- **钩子**：不是 shell 钩子，是进程内插件（JS/TS）。源码里看到 `packages/plugin/src/index.ts` 的签名：`"tool.execute.before"?: (input: { tool: string; sessionID: string; callID: string }, output: { args: any }) => Promise<void>`；另有 `"permission.ask"?: (input: Permission, output: { status: "ask" | "deny" | "allow" })`。官方文档的例子用 `throw new Error("Do not read .env files")` 阻止读取。源码里看到 `packages/opencode/src/session/tools.ts` 在 `item.execute(args, ctx)` 之前 `trigger("tool.execute.before", …)`，所以抛出即工具不执行；模型看到什么，文档没写。
- **配置位置**：项目 `.opencode/plugins/`、全局 `~/.config/opencode/plugins/`、`opencode.json` 的 `plugin` 数组。环境变量：`OPENCODE_CONFIG`（指定配置文件）、`OPENCODE_CONFIG_CONTENT`（内联 JSON，优先级接近最高）、`OPENCODE_CONFIG_DIR`（「像标准 `.opencode` 目录一样搜索 agents、commands、modes、plugins」，是**追加**不是替换全局目录）。官方文档只说 `plugin` 数组支持 npm 包名；源码里看到 `packages/opencode/src/config/plugin.ts` 的 `resolvePluginSpec` 会把 `./x.ts`、绝对路径、`file://` 规格归一成 file URL——文件路径可用，但未写进文档。
- **一处副作用**（源码里看到）：`config.ts` 对每个配置目录都调用 `ensureGitignore` 并后台 `npm install @opencode-ai/plugin`。若把 `OPENCODE_CONFIG_DIR` 指向每会话目录，写入落在那里，不碰工作树；若用项目 `.opencode/`，会在工作树里生成 `.gitignore`、`node_modules` 等。
- **Stop 类**：没有专门的 Stop 钩子，`event` 钩子能收 `session.idle`。
- **稳定性**：插件 API 有 TypeScript 类型与文档，`experimental.*` 命名空间明确标实验，`tool.execute.before` 不在其中。

### Cursor CLI

- **钩子**：通用 `preToolUse`（`tool_name` 取 `Shell`/`Read`/`Write`/`Task`/`MCP:*`）与专用 `beforeShellExecution`（`command`、`cwd`、`sandbox`）、`beforeMCPExecution`（`tool_name`、`tool_input`、`mcp_server_name`）。CLI changelog 说 2026-01 起 CLI 读钩子，「Claude Code settings.json 的钩子会被读取并合并」，2026-04 起「接受 Claude Code 格式的钩子响应」，2026-05-20 起 payload 走 stdin。
- **配置位置**：`hooks.json`，四层：企业 `/etc/cursor/hooks.json`、团队（云端下发）、项目 `<project>/.cursor/hooks.json`、用户 `~/.cursor/hooks.json`。CLI 自己的 `cli-config.json` 可用 `CURSOR_CONFIG_DIR` 重定位，**`hooks.json` 是否跟着走未查到**。没有查到每会话注入钩子的参数。
- **硬拒**：stdout JSON `permission: "allow" | "deny" | "ask"`，可附 `user_message`、`agent_message`；退出码 2 等价 deny。**默认 fail-open**（其他失败放行），钩子定义上 `failClosed: true` 改为失败即拒——文档建议安全类钩子打开。
- **输入**：公共字段 `conversation_id`、`generation_id`、`hook_event_name`、`workspace_roots`、`cursor_version`、`model`；`preToolUse` 另有 `tool_name`、`tool_input`、`tool_use_id`、`cwd`。没有 `session_id`。
- **稳定性**：文档完整；「项目钩子只在受信任工作区生效」；`prompt` 型钩子需登录。

### Kimi Code

- **钩子**：`PreToolUse`（还有 `PermissionRequest`、`PermissionResult`）。配置是 `config.toml` 里的 `[[hooks]]` 数组，每条 `event`（必填）、`command`（必填）、`matcher`（可选正则）、`timeout`（1–600 秒，默认 30）。
- **配置位置**：官方文档说「所有钩子规则都写在 `~/.kimi-code/config.toml` 的 `[[hooks]]` 数组里」；`KIMI_CODE_HOME` 可以把整个目录挪走（文件名固定 `config.toml`）。项目级 `<project>/.kimi-code/local.toml` 存在，但**能否放钩子未查到**。
- **硬拒**：退出码 2「停止当前操作，stderr 内容作为阻止原因」；或 stdout JSON `hookSpecificOutput.permissionDecision: "deny"` + `permissionDecisionReason`。官方文档说「阻止后 Kimi Code CLI 把阻止原因写回上下文，模型据此选更安全的替代」。**其他非零退出码默认放行（fail-open）**。
- **输入**：`hook_event_name`、`session_id`、`session_title`、`client_type`、`cwd`；`PreToolUse` 另有 `tool_name`、`tool_input`（示例读 `tool_input.command`），完整 schema 文档没列全。
- **稳定性**：当天读到的官方页面没有 beta 字样；搜索到的第三方镜像文档写「beta 特性」，不作数。Herdr 的 Kimi 集成往同一个 `config.toml` 追加 `[[hooks]]`，说明该文件被第三方工具当作可编辑目标。

### Qwen Code

- **钩子**：`PreToolUse`，matcher 正则按工具 id 匹配；类型有 `command`、`http`、`function`（仅会话级）、`prompt`。
- **配置位置**：`.qwen/settings.json`（项目，**要求受信任目录**）、`~/.qwen/settings.json`、系统 `/etc/qwen-code/settings.json`；环境变量 `QWEN_CODE_HOME`、`QWEN_CODE_SYSTEM_SETTINGS_PATH`、`QWEN_CODE_SYSTEM_DEFAULTS_PATH`，与 Gemini CLI 同构。`disableAllHooks: true` 顶层总开关。
- **硬拒**：退出码 2「忽略 stdout，把 stderr 作为错误反馈给模型」；或 `hookSpecificOutput.permissionDecision: "deny"` + `permissionDecisionReason`；`decision` 字段也接受 `allow|deny|ask|block`。退出码 0 或其他非零时 stderr 只在 debug 模式显示。
- **输入**：`session_id`、`transcript_path`、`cwd`、`hook_event_name`、`timestamp`、`permission_mode`、`tool_name`、`tool_input`、`tool_use_id`、`tool_call_id`。
- **稳定性**：不标实验；最新发布说明里有三条钩子相关修补（HTTP 钩子禁重定向、fire-and-forget 钩子善后、`PreToolUse` 返回 ask 时显示 diff），说明还在活跃演进。

### GitHub Copilot CLI

- **钩子**：`preToolUse`，官方文档称它「最有力，能批准或拒绝工具执行」。
- **配置位置**：按「策略 → 用户 → 项目 → 插件」顺序加载并合并：策略 `/etc/github-copilot/policy.d/*.json`；用户 `~/.copilot/hooks/*.json`，设了 `COPILOT_HOME` 则是 `$COPILOT_HOME/hooks/`；仓库 `.github/hooks/*.json`；内联于 `.github/copilot/settings.json`、`.github/copilot/settings.local.json`、`~/.copilot/settings.json`；插件目录。没有查到命令行参数或其他环境变量能指定钩子文件。
- **硬拒**：stdout JSON `permissionDecision: "allow" | "deny" | "ask"`，`permissionDecisionReason`「显示给代理，deny 时必填」，`modifiedArgs` 可改参数。退出码规则对 `preToolUse` 是 fail-closed：「退出码 2、崩溃和其他非零退出都拒绝」，但**「超时对所有事件都 fail-open，包括 `preToolUse` 和管理员下发的策略钩子」**——这条要写进我们的超时预算。
- **输入**：`sessionId`、`timestamp`、`cwd`、`toolName`、`toolArgs`（驼峰命名，与其他家不同）。
- **稳定性**：文档不标预览。GitHub issue #2540（2026-04 开，仍 open）报告插件 `hooks.json` 里的 `preToolUse` 不触发，并提到另一个 bug：配置里的钩子在主会话触发、在子代理不触发。我们走用户级 `hooks/*.json`，不走插件，但子代理那条要实测。

## ACP 权限请求

ACP（agentclientprotocol.com，protocolVersion 1）里 Agent 与 Client 分进程；我们的适配器是 Client。与钩子等价的入口是 Agent 调 Client 的 `session/request_permission`：

- **请求**：`sessionId`、`toolCall`（`ToolCallUpdate`：`toolCallId` 必填，`title`、`kind`、`status`、`content`、`locations`、`rawInput`、`rawOutput` 可选）、`options[]`（每项 `optionId`、`name`、`kind` ∈ `allow_once` / `allow_always` / `reject_once` / `reject_always`）。
- **响应**：`outcome` 为 `{"outcome":"cancelled"}` 或 `{"outcome":"selected","optionId":…}`。Client 选 `reject_once` 就是硬拒，判决在我们进程里做，不落任何文件，天然每会话。
- **两条边界**：其一，规范原文是 Agent「**MAY** request permission from the user before executing a tool call」——问不问由 Agent 决定，Agent 若在自动批准模式下运行，一次都不会问；所以 ACP 模式下适配器必须把 Agent 拉在「每次都请求」的审批模式，这是 Agent 各自的开关，不是协议的。其二，`rawInput` 是可选字段，白名单要按参数判时得先确认目标 Agent 填它；只填 `title` 与 `kind`（`read`/`edit`/`delete`/`move`/`search`/`execute`/`fetch`/…）时只能按类别粗判。`session/update` 里的 `tool_call` 通知只能观察，不能阻止。
- **谁说 ACP**：ACP 官网列表里 Gemini CLI、Kimi CLI、OpenCode、Qwen Code、Cursor、GitHub Copilot 为原生实现；Claude Code 经 Zed 的 `claude-agent-acp` 适配器，Codex 经 `codex-acp` 适配器。适配器版本与它们把哪些工具映射成 `request_permission`，本轮未查。

## 边界与取舍

四条判据：① 执行前硬拒；② 按会话配置且不动仓库；③ 拿到工具名与参数；④ 有文档且稳定。

| Harness | ① 硬拒 | ② 每会话不动仓库 | ③ 工具名+参数 | ④ 文档稳定 | 裁决 |
| --- | --- | --- | --- | --- | --- |
| Claude Code | 满足 | **满足**（`--settings` 内联） | 满足 | 满足 | **支持**，首选样板 |
| Codex CLI | 满足 | 部分：用户级或 `CODEX_HOME` 重定位；且要过信任审核或带 `--dangerously-bypass-hook-trust` | 满足 | 满足 | **支持，有信任关卡**；重定位 `CODEX_HOME` 时要把鉴权一起种进去 |
| Gemini CLI | 满足 | 可行未实测：`GEMINI_CLI_SYSTEM_SETTINGS_PATH` 指向每会话文件 | 满足 | 满足 | **支持**，需实测环境变量路径 |
| OpenCode | 满足 | **满足**（`OPENCODE_CONFIG_CONTENT` + 文件路径插件） | 满足 | 部分：文件路径规格只在源码 | **支持**；插件是 JS，要在里面 spawn `hctl-tool` |
| Cursor CLI | 满足（须 `failClosed: true`） | 不满足：只有用户级 `~/.cursor/hooks.json`，每会话方式未查到 | 满足 | 满足 | **部分支持**：只能用户级全局挂，不能按会话 |
| Kimi Code | 满足（其他非零 fail-open） | 部分：用户级或 `KIMI_CODE_HOME` 重定位 | 满足 | 满足 | **支持**，同 Codex 的重定位代价；重定位会丢 Herdr 的 Kimi 生命周期钩子，要一起种 |
| Qwen Code | 满足 | 可行未实测：`QWEN_CODE_SYSTEM_SETTINGS_PATH` | 满足 | 满足 | **支持**，与 Gemini 同构 |
| Copilot CLI | 满足（超时 fail-open） | 部分：用户级或 `COPILOT_HOME` 重定位 | 满足 | 部分：钩子相关 issue 未关 | **支持，有两处漏水**：超时放行、子代理触发待实测 |
| ACP | 满足 | 满足 | 部分：`rawInput` 可选 | 满足 | **支持**，前提是把 Agent 拉在每次都问的审批模式 |

几点取舍：

- **「用户级」不等于「每会话」。** 写到 `~/.codex`、`~/.cursor`、`~/.kimi-code`、`~/.copilot` 不污染工作树，但对这台机器上该用户所有会话生效，包括人自己开的会话。要真正按会话隔离，只有三条路：命令行内联（Claude Code）、内联环境变量（OpenCode）、把整个 home 用环境变量挪到每会话目录（Codex、Gemini、Kimi、Qwen、Copilot、Claude Code 都有对应变量）。挪 home 的代价是鉴权、历史、Herdr 集成钩子都要一起种进去，否则 harness 会当自己是首次运行。Gemini 与 Qwen 的「系统 settings 路径」变量是个例外：只挪一份文件、优先级最高、不碰 home——最省事，但当天没有实测。
- **fail-open 的家要补一道自己的兜底。** Kimi（其他非零退出码放行）、Cursor（默认放行，需 `failClosed`）、Copilot（超时放行）三家里，我们的检查程序自身出错时 harness 会放行。对策是检查程序把「自己出错」也映射成显式 deny 输出，而不是靠退出码；超时预算留够。
- **Cursor CLI 标「部分支持」不是因为钩子弱，是因为没找到按会话挂的口。** 若后续查到 `CURSOR_CONFIG_DIR` 也重定位 `hooks.json`，可升级。
- **ACP 模式的门比钩子薄一层。** 钩子在 harness 决定执行之后、执行之前必经；`request_permission` 只在 Agent 觉得该问时才来。它足够做白名单，但前提写进冻结规格：Agent 的审批模式必须是「逐项询问」。

## 与 Herdr 的兼容

Herdr（v0.8.2，master @ `0f8ad12`）只在 PTY 里拉起 harness，钩子由参考实现在拉起前注入。要验证三件事。

**其一，环境变量与配置目录能否透传进 Herdr 拉起的进程。** 官方 CLI 参考说 workspace/tab/pane 创建都接受 `--cwd PATH` 与 `--env KEY=VALUE`；窗格里的进程继承 `HERDR_ENV`、`HERDR_PANE_ID`、`HERDR_BIN_PATH`、`HERDR_SOCKET_PATH` 以及调用方给的 `--env`，「Herdr 管理的变量与调用方提供的冲突时以 Herdr 为准」。`herdr agent start <name> --kind claude --pane ID -- <agent-args...>` 把 `--` 之后的参数原样交给 harness，所以 `--settings '<JSON>'` 这类命令行注入也能到达。结论：`CLAUDE_CONFIG_DIR`、`CODEX_HOME`、`GEMINI_CLI_SYSTEM_SETTINGS_PATH`、`OPENCODE_CONFIG_CONTENT` 等都可以在建窗格时用 `--env` 设进去，命令行参数走 `agent start -- …`。Herdr 自己的 Claude 集成也「默认用 `~/.claude`，设了 `CLAUDE_CONFIG_DIR` 则用它」，说明它把这个变量当作正当接口。

**其二，钩子回调能否找到 `hctl-tool`。** 钩子进程继承 harness 的环境与工作目录，harness 又继承窗格 shell 的环境，PATH 沿链传下去。但 Herdr 自己的经验是不靠 PATH：v0.8.2 发布说明写「Agent 钩子现在调用正在运行的 Herdr 二进制，而不是 PATH 上先出现的那个」，源码里看到 Claude 集成的 PowerShell 脚本用 `HERDR_BIN_PATH` 定位二进制，sh 脚本则直接经 `HERDR_SOCKET_PATH` 走 socket。我们照做：钩子命令里写 `hctl-tool` 的绝对路径，或用 `--env HCTL_TOOL_PATH=…` 传进去再在钩子里引用，不赌 PATH。

**其三，钩子往 stderr 打的字会不会干扰 Herdr 判断 agent 状态。** Herdr 的状态检测分两种权威：装了「完整生命周期钩子」的 agent（Pi、OMP、Kimi Code、OpenCode、Kilo、MastraCode）以集成的报告为准，不再看屏幕；其余 agent——**包括 Claude Code、Codex、Gemini、Cursor、Copilot、Qwen**——Herdr 集成只报告会话身份，状态「仍来自屏幕清单检测」：读窗格底部实时快照，用 TOML 清单里的正则判 `idle`/`working`/`blocked`，也参考终端标题与 OSC 进度序列。源码里看到 `src/detect/manifests/claude.toml` 的规则匹配的是「esc to interrupt」、忙碌转轮字形、「do you want to proceed?」这类界面文本，blocked 判定「刻意从严，只在底部快照匹配到已知的批准、提问或权限界面时才标」，没匹配到就回落 `idle`。

钩子本身的 stdin/stdout/stderr 是 harness 用管道接走的，不直接进 PTY；会画到屏幕的只有 harness 决定显示的部分。逐家看：Claude Code 退出码 0 时 stderr 只进调试日志，退出码 2 时给模型，**其他非零退出码时在 transcript 里显示 `hook error` 通知和 stderr 第一行**；Qwen 退出码 0 或其他非零时 stderr 只在 debug 模式显示；Gemini 与 Kimi 退出码 2 时 stderr 作为原因进上下文；Copilot 退出码 2 在非 `preToolUse` 事件上「stderr 显示给用户」。结论：只要我们的钩子在放行时不往 stderr 写、拒绝时走 stdout JSON 而不是「退出码 2 + stderr」，就没有任何文字额外进入屏幕，Herdr 的屏幕清单看到的仍是 harness 自己的界面。拒绝原因进模型上下文后 harness 会把它渲染成一段工具错误，这与任何工具失败无异，不构成新的屏幕形状。Herdr 自己的钩子脚本也是这个纪律：源码里看到 `herdr-agent-state.sh` 把 stdin 落到临时文件、全部输出重定向、任何异常都 `exit 0`。

一个附带的兼容点：Herdr 的 Claude 集成把 `SessionStart` 钩子写进 `~/.claude/settings.json`；Claude Code 钩子跨层合并，我们用 `--settings` 注入不会顶掉它。但若走 `CLAUDE_CONFIG_DIR` 每会话重定位，新目录里没有 Herdr 那条钩子，会话身份上报就没了——同理适用于 Kimi（`~/.kimi-code/config.toml` 的 `[[hooks]]`，而且 Kimi 那条是生命周期权威）与 OpenCode（`~/.config/opencode/plugins/herdr-agent-state.js`，`OPENCODE_CONFIG_DIR` 是追加不受影响）。重定位 home 的方案要把 Herdr 集成一起种进去。

## 决定建议

按接入模式分两条，白名单格式与检查入口一条：

1. **PTY 交互模式用原生钩子。** 优先级：Claude Code（`--settings` 内联 `PreToolUse`）、OpenCode（`OPENCODE_CONFIG_CONTENT` 加文件路径插件）、Gemini 与 Qwen（系统 settings 路径变量，先实测）、Codex 与 Kimi 与 Copilot（home 重定位 + 种入鉴权与 Herdr 集成；Codex 另加 `--dangerously-bypass-hook-trust` 或预置信任）；Cursor CLI 只能用户级挂，标「部分支持」，不进第一批。八家的 `PreToolUse` 类事件都把工具名与参数交到 stdin JSON（OpenCode 是函数参数），所以检查程序可以统一。
2. **ACP 结构化模式用协议权限请求。** 适配器作为 Client 在 `session/request_permission` 里判，选 `reject_once` 硬拒，不落文件。冻结规格里必须写明 Agent 的审批模式为逐项询问，并按 Agent 记录 `rawInput` 是否可用；不可用时只能按 `kind` 粗判，要在可核验性上标出来。
3. **白名单格式与检查入口和 harness 无关。** 白名单随冻结规格走；检查入口是 `hctl-tool` 的一个子命令，读 stdin JSON，按 `--harness <name>` 把各家字段（`tool_name`/`tool_input`、`toolName`/`toolArgs`、`tool`/`args`、Cursor 的 `command`）归一成同一个内部形状再判，按各家格式输出 deny（Claude/Codex/Kimi/Qwen 的 `hookSpecificOutput.permissionDecision`、Gemini 的 `decision`、Cursor 的 `permission`、Copilot 的 `permissionDecision`），永远以退出码 0 + stdout JSON 表达拒绝，放行时不写 stderr；自身出错映射成显式 deny，避开 Kimi/Cursor/Copilot 的 fail-open。ACP 模式复用同一判定核心，只是输入来自 `toolCall`，输出是 `optionId`。
4. **复用决策**：钩子机制全部**适配协议**——借各家的事件名、字段名与判决格式，不移植代码；Herdr 的 `HERDR_BIN_PATH` 定位与「钩子静默」纪律**仅参考行为**。不新增依赖。

## 证据

查阅日均为 2026-09-03；GitHub 发布信息取自各仓库 `releases/latest`。

| 对象 | 钉定 | 链接 |
| --- | --- | --- |
| Claude Code | CHANGELOG 最新版本 2.1.259；仓库 main @ `f173a69`（2026-09-02） | [hooks](https://code.claude.com/docs/en/hooks) · [settings](https://code.claude.com/docs/en/settings) · [CLI reference](https://code.claude.com/docs/en/cli-reference) · [CHANGELOG](https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md) |
| Codex CLI | rust-v0.153.0（2026-09-03） | [hooks](https://learn.chatgpt.com/docs/hooks)（`developers.openai.com/codex/hooks` 重定向至此） · [config-advanced](https://learn.chatgpt.com/docs/config-file/config-advanced) · [config-reference](https://learn.chatgpt.com/docs/config-file/config-reference) · [releases](https://github.com/openai/codex/releases/tag/rust-v0.153.0) |
| Gemini CLI | v0.58.0（2026-09-01） | [hooks](https://geminicli.com/docs/hooks/) · [hooks reference](https://geminicli.com/docs/hooks/reference/) · [writing hooks](https://geminicli.com/docs/hooks/writing-hooks/) · [configuration](https://geminicli.com/docs/reference/configuration/) · [releases](https://github.com/google-gemini/gemini-cli/releases/tag/v0.58.0) |
| OpenCode | v1.18.27（2026-09-02）；源码 dev @ `79d5031`（2026-09-03）：`packages/plugin/src/index.ts`、`packages/opencode/src/config/config.ts`、`packages/opencode/src/config/plugin.ts`、`packages/opencode/src/session/tools.ts` | [plugins](https://opencode.ai/docs/plugins/) · [config](https://opencode.ai/docs/config/) · [仓库](https://github.com/anomalyco/opencode/tree/79d503150ca22f151afe4ea543fac8a8eb8aef53) |
| Cursor CLI | CLI changelog 最新条目 2026-08-26（无版本号） | [hooks](https://cursor.com/docs/hooks) · [CLI configuration](https://cursor.com/docs/cli/reference/configuration) · [CLI changelog](https://cursor.com/docs/cli/changelog) |
| Kimi Code | @moonshot-ai/kimi-code@0.40.1（2026-09-02） | [hooks](https://moonshotai.github.io/kimi-code/en/customization/hooks) · [config files](https://moonshotai.github.io/kimi-code/en/configuration/config-files) · [仓库](https://github.com/MoonshotAI/kimi-code) |
| Qwen Code | v0.23.0（2026-09-03） | [hooks](https://qwenlm.github.io/qwen-code-docs/en/users/features/hooks/) · [settings](https://qwenlm.github.io/qwen-code-docs/en/users/configuration/settings/) · [releases](https://github.com/QwenLM/qwen-code/releases/tag/v0.23.0) |
| GitHub Copilot CLI | v1.0.82（2026-08-29） | [hooks reference](https://docs.github.com/en/copilot/reference/hooks-reference) · [use hooks (CLI)](https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/use-hooks) · [about hooks](https://docs.github.com/en/copilot/concepts/agents/hooks) · [issue #2540](https://github.com/github/copilot-cli/issues/2540) |
| ACP | protocolVersion 1；schema-v1.21.0（2026-08-20） | [tool calls / request_permission](https://agentclientprotocol.com/protocol/tool-calls) · [schema](https://agentclientprotocol.com/protocol/schema) · [initialization](https://agentclientprotocol.com/protocol/initialization) · [agents](https://agentclientprotocol.com/overview/agents) |
| Herdr | v0.8.2（2026-08-19）；源码 master @ `0f8ad12`（2026-09-03）：`src/integration/targets.rs`、`src/integration/claude_settings.rs`、`src/integration/assets/claude/herdr-agent-state.sh`、`src/detect/manifests/claude.toml`、`docs/next/website/src/content/docs/{agents,integrations}.mdx` | [agents](https://herdr.dev/docs/agents/) · [integrations](https://herdr.dev/docs/integrations/) · [cli-reference](https://herdr.dev/docs/cli-reference/) · [configuration](https://herdr.dev/docs/configuration/) · [仓库](https://github.com/herdrdev/herdr/tree/0f8ad128b65869d683253753a477c0ccafca9bf2) · 本仓既有条目 [runtime/herdr.md](./runtime/herdr.md) |

未查到、留待实测的项，正文已就地标注：Codex `-c` 能否注入钩子表、Codex 信任记录存放位置；Cursor `CURSOR_CONFIG_DIR` 是否重定位 `hooks.json`；Kimi 项目级 `local.toml` 能否放钩子；Gemini/Qwen 系统 settings 路径变量用于每会话注入的实际行为；Copilot 子代理是否触发 `preToolUse`；各 ACP Agent 是否填 `rawInput`。

## 复核记录

（发布后追加。）

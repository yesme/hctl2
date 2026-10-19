# Harness 结构化控制、观测窗格与终端复用器底座取舍

> 状态：已拍板 · 所有者 2026-10-20「按照你裁定的规则来。但是，我已确定使用tmux (因为herdr存在的意义已经消失)，放在后边来做仅仅是为了开发上的便利而已。另外，对于各个harness的适配其实是agency内部的实现过程（每加一个实现增加一个文件/文件目录），是可扩展的。对于control来说，只暴露一个可观测接口（例如tmux渲染出来的readonly窗格）。」裁定记在 [`07` §二 第 45 条](./p2-control-20260906/07-demo-kickoff.md)，设计正文 v0.19.5（决策史 §41），任务书在 [`09` §四 H0、第 10 包](./p2-control-20260906/09-p25-kickoff.md)；下文保留讨论时的原样，主笔核出与库不符的几处列在「勘误」
> 基线：main @ `c4c553f9`（草案 v0.19.4）
> 去向：[`09-p25-kickoff.md`](./p2-control-20260906/09-p25-kickoff.md) + [`docs/design/participant.md`](../../docs/design/participant.md) + [`docs/research/README.md`](../../docs/research/README.md) + `src/agency/`

## 勘误（主笔 2026-10-20 核对；下文正文保留讨论时的原样）

- §一 引的验收句「单会话与两会话并跑在 herdr 里起得来……」不在 `09` 里。`09` §四 H1–H3 验收第 1 条第五问是「走结构化接口时 pane 里还是不是平常的交互界面、Herdr 认不认得」，第 2 条写的是「结构化接口没有交互界面的，写明 pane 里看到的是什么，交所有者定接受不接受」——这道题本来就留给了所有者，不是写席与审席的理解分歧。
- §一「评审席（Claude 对 #461/#464、Grok 对 #461）指出」不实：当时 #461 上没有任何评论，#464 的两席评审也没提这件；是三份 PR 描述自己把「没有 pane 接受不接受」列给了所有者。
- §一、§四「小活 Y（10 家 Harness 无交互探活命令表）」库里没有这件活；`07` 的小活 Y 是 Linux 上 `Text file busy`。所要的内容并进钥匙串专项（`07` §二 第 45 条 ⑤）。
- §2.3 类别 ① 的 `opencode attach`、`qodercli --acp` 在 `harness-dispatch-channel-20261006.md` 里没有依据（OpenCode 行只有 serve / acp，Qoder 行是 `-p` / `--print`、没有 ACP），写备忘的 mac 上也没装这两家，按未核实处理。
- §4.1 表里 H1「每轮起一个 `agy -p`」不对：`agy.rs`（#467）是一个进程常驻、`--input-format stream-json` 从 stdin 一轮轮喂，`--conversation <id>` 只在续接时用，合 `07` §二 第 22 条。
- §三、§四 的 H1–H6：`09` 只排了 H1–H3（所有者「这次接 3 家」）；Kimi、Qwen 等额度，MiniMax 不排，Muse 不可用。公共底座等下一家真排上再抽。

## 一、触发背景与三项待裁定问题总览

在推进 P2.5 的三家新 Harness 接入包——H1（Antigravity CLI `antigravity-cli`，PR #467）、H2（Grok Build CLI `grok`，PR #461）、H3（DeepSeek `omp`，PR #464）——时，评审席（Claude 对 #461/#464、Grok 对 #461）指出开工书 [`09-p25-kickoff.md`](./p2-control-20260906/09-p25-kickoff.md) §四 H1–H3 的第 3 条验收写着：

> 「单会话与两会话并跑在 herdr 里起得来、不断连；同一会话发两轮提示词，`turn.complete` 都能对上。」

而 H1、H2、H3 的实际实现均参考了 3e（Codex `app-server`）的控制路径，由 Agency 直接作为 OS 父进程拉起对应 CLI 的原生结构化管道模式（`agy --mode agent -p --output-format stream-json`、`grok acp`、`omp --mode rpc`），通过 stdio JSONL / JSON-RPC 收发提示词并判定 `turn.complete`，**完全没有经过 `herdr`**。

围绕这一现状，我们系统梳理了需要所有者裁定的三个问题，并就**裁定点 1** 展开了两轮深度讨论与调研：

1. **裁定点 1（已展开讨论并形成倾向）**：
   - **上篇（控制 vs 观测）**：到底是用 Harness 的原生结构化 CLI/RPC 模式以获得更稳定、可编程（programmable）的交互，还是用终端容器（Herdr/tmux）交互 TUI 以获得更好的可观测性？除 `agy` 和 `codex` 外，其余 8 家 Harness 是否也面临同一抉择？业界（Orca、Grokbot、Codex / ACP 生态等）如何取舍？
   - **下篇（终端容器底座选型）**：在所有者倾向**选项 B（控制走各家原生结构化接口 + 终端窗格挂只读观测 TUI）**的前提下，原本作为「控制 Agent」引入的 Herdr 已经退化为纯 TUI 容器；如果只是做 TUI 与观测容器，回到 `tmux` 是否最省内存？
2. **裁定点 2（待拍板）**：H1–H6 六家结构化管道 Harness 的子进程收发、落盘目录与 `state.json` / `events.jsonl` schema 是否抽出共用底座？
3. **裁定点 3（待拍板）**：跨平台（macOS 系统 Keychain vs Ubuntu 配置文件）凭证环境检查如何与 [`09-p25-kickoff.md`](./p2-control-20260906/09-p25-kickoff.md) 的小活 Y（10 家 Harness 无交互探活命令表）对齐？

---

## 二、裁定点 1（上）：程序化结构化控制 vs TUI 交互观测

### 2.1 架构层结论：控制面与观测面解耦

**「程序化稳定控制（CLI/RPC/ACP）」与「TUI 现场可观测性」不是非此即彼的二选一；正确的架构切分是「控制面与观测面解耦（Control Plane ≠ Observation Plane）」：**

- **控制面（发提示词、续接会话、判 `turn.start` / `turn.complete`、拿报错与退出码）**：必须走各家 Harness 的**原生结构化协议**（JSON-RPC / ACP / `stream-json`），不走终端模拟按键；
- **观测面（人类实时看屏幕、看流式思考与工具调用、排查卡死）**：走**终端复用器窗格**——支持原生 `--remote` / `attach` 的直接挂原生 TUI 旁观器；不支持原生附着的，挂一个实时渲染该会话结构化事件流的**只读观测窗格（Observation Pane）**。

### 2.2 为什么不能用终端模拟按键（`herdr agent prompt`）充当控制面

1. **输入投递脆弱（转义、截断、误触快捷键）**：
   通过 PTY 模拟键盘往交互式 TUI 注入多行代码、大体积上下文（几 KB 到几十 KB）或首字符带 `!` / `/` 的文本时，极易发生丢字、粘贴分块截断或误触 TUI 内置命令。我们在 3d 接 Claude Code 时已经踩过一次（#362）：`herdr agent prompt` 喂多行和大文本直接失真，最后不得不改成**用自建 `--plugin-dir` 插件在 `prompt.submit` 钩子里把一行短 marker 替换成真正的长提示词**，才绕开 TUI 输入框。
2. **完成判定与遥测失真**：
   TUI 屏幕输出是给人眼看的 ANSI 富文本，没有机器契约。`herdr agent prompt --wait` 只能粗粒度等终端空闲，既无法精确对齐单轮 `turn.complete`，也拿不到结构化的 `session_id`、token 用量、工具调用轨迹与进程退出码（`herdr` 的 `pane.exited` 事件甚至不携带退出码）。

### 2.3 全量 10 家 Harness 的接口形态盘点

除 `agy` 和 `codex` 外，**我们席位表里的全部 10 家 Harness 都面临完全相同的抉择**。按上游 CLI 原生提供的接口形态，10 家可归为三类（依据 [`docs/research/harness-dispatch-channel-20261006.md`](../../docs/research/harness-dispatch-channel-20261006.md)、[`docs/research/harness-adapters.md`](../../docs/research/harness-adapters.md) 及各家 CLI 实测）：

| 类别 | 涵盖的 Harness（共 10 家） | 原生结构化控制接口 | 原生 TUI 与结构化控制的关系 |
| --- | --- | --- | --- |
| **① 原生支持 Server + Remote TUI 分离（3 家）** | **Codex**（`codex`）<br>**GLM**（`opencode`）<br>**Qwen**（`qodercli`） | `codex app-server --listen ws://...`<br>`opencode serve` / `opencode acp`<br>`qodercli --acp` / `--output-format stream-json` | **天然解耦**：后台跑结构化服务（Agency 经 WebSocket / HTTP / stdio 控制），同时终端窗格可跑 `codex resume <id> --remote ws://...` 或 `opencode attach http://...` 实时旁观同一会话（Codex 在 3e 已按此落地） |
| **② 原生支持结构化管道，但无内置 Remote TUI 附着（6 家）** | **Antigravity**（`agy`）<br>**Grok**（`grok`）<br>**DeepSeek**（`omp`）<br>**Kimi**（`kimi`）<br>**MiniMax**（`mcode`）<br>**Muse**（`muse`） | `agy --mode agent -p --output-format stream-json [--resume <id>]`<br>`grok acp`（JSON-RPC 2.0 over stdio）<br>`omp --mode rpc`（JSON-lines over stdio）<br>`kimi acp` / `--print --output-format stream-json`<br>`mcode acp` / `mcode serve`<br>`muse acp` / `muse serve` | **进程级互斥**：不带参数启动时进全屏交互 TUI（或 `agy` 拉起 Electron GUI）；带 `acp` / `rpc` / `stream-json` 启动时走纯 stdio 结构化管道，不再渲染 TUI，也没有内置 `--remote` 旁观子命令 |
| **③ 交互 TUI + Hook 插件回传（1 家特例）** | **Claude Code**（`claude`） | 虽有 `claude -p --output-format stream-json`，但 3d 选了常驻 TUI + `--plugin-dir` 插件经 Agency UDS 回传 `turn.start` / `turn.complete` | 跑在终端窗格里，但真正的提示词正文投递和完成判定已经通过插件抽离到了 Agency 自己的 Unix Socket 上，终端窗格只负责敲一行短 marker |

### 2.4 业界同类系统的抉择演进

| 系统 / 阵营 | 做法 | 实战得失 |
| --- | --- | --- |
| **早期纯 TUI 终端派**<br>（**Grokbot**、**Overstory**、**Claude Squad**、**Agent Deck**，见 [`.memo/notes/grokbot-compare/`](../notes/grokbot-compare/)） | 全部跑在 `tmux` 窗格里，靠 `tmux send-keys` / `load-buffer` + `paste-buffer` 塞提示词，靠 `capture-pane` 抓屏幕或正则匹配提示符判完成 | **得**：零协议适配，人随时 `tmux attach` 看现场。<br>**失**：多行/特殊字符粘贴极易翻车；CLI 升一版改个 UI 字符就挂；拿不到精确完成事件和 token 统计。Overstory 等后期都在往外挂 hook 或本地 RPC 补丁迁移 |
| **现代「结构化控制 + 终端观测解耦」派**<br>（**Codex `app-server` 架构**、**OpenCode `serve + attach`**、**Zed / JetBrains ACP 生态**、**Orca 类编排器**） | 编排器一律走 stdio JSON-RPC（ACP / `app-server`）或 `stream-json` 管道拿确定性事件流；终端窗格只做**附着旁观（Attach / Mirror）**，不反向承担控制职责 | **得**：控制面 100% 确定性（无转义污染、精确 `turn.complete`、结构化 tool call 与报错），同时保留终端窗格供人旁观 |

### 2.5 三种候选方案与所有者倾向

| 方案 | 做法 | 取舍评价 |
| --- | --- | --- |
| **选项 A：纯结构化管道（无 `--remote` 的不开窗格）** | 第 ② 类 6 家只跑 stdio 结构化管道并落盘 `events.jsonl`，不开终端窗格 | 控制最简、零额外内存，但人类无法在终端复用器里统一切窗格看这 6 家的实时输出 |
| **选项 B：结构化管道控制 + 终端只读观测窗格（所有者倾向）** | 控制面 100% 走原生结构化管道（`stream-json` / `acp` / `rpc`）；观测面由 Agency 把每轮结构化事件/回复实时格式化追加到 `<session_dir>/view.log`，并在终端复用器里为该会话挂一个只读观测窗格（`tail -f <session_dir>/view.log`），把 `pane_id` 记入会话状态 | **兼顾两端**：既不牺牲分毫程序化控制可靠性，又保住所有席位在终端复用器里「一席一窗格、随时可旁观、契约测试可断言窗格存活与内容」的统一体验 |
| **选项 C：退回用终端容器敲键盘控制 TUI** | 放弃 `acp` / `rpc` / `stream-json`，改用 `herdr agent prompt` 往各家 TUI 敲字 | 重踩 #362 的多行失真与完成判定黑盒坑，且 `agy` 无参数直接拉起 Electron 桌面窗而非终端 TUI，已排除 |

**阶段结论**：所有者明确倾向 **选项 B**。

---

## 三、裁定点 1（下）：在选项 B 下，Herdr 是否退化？是否回到 `tmux` 最省内存？

### 3.1 所有者的核心追问与架构层裁决

所有者追问：

> 「我倾向于选项 B。在这样的选择下，herdr 还是最好的选择么？因为 herdr 本来我们是作为『控制 agent』来使用的，而现在似乎退化为一个 TUI 了——控制由各家的原生接口来执行了。如果只是 TUI，是不是回到 tmux 最省 Mem？」

**架构层裁决：完全成立。**

当初在 v0.14.1（见 [`docs/research/runtime/herdr.md`](../../docs/research/runtime/herdr.md) 与 [`docs/research/sdk/herdr.md`](../../docs/research/sdk/herdr.md)）弃 `tmux` 选 Herdr，核心假设是：**靠 Herdr 的 `agent.start` + `agent.prompt --wait` + `pane.agent_status_changed` 充当统一的跨 Harness 语义控制器**，省掉自建 `agentd`。

但经过 3c → 3d → 3e → H1/H2/H3 的实战演进，Herdr 的「控制面」事实上已经被完全架空：

- **9 家走结构化协议的 Harness**（Codex、Antigravity、Grok、DeepSeek `omp`、GLM `opencode`、Kimi、Qwen、MiniMax、Muse）：控制与完成判定 100% 走 Agency 直连的 WebSocket JSON-RPC 或 stdio JSONL/ACP 管道；终端窗格只跑 `codex resume --remote` 或选项 B 的只读观测流，**对 Herdr `agent.*` 控制接口的调用量为 0**；
- **唯一跑交互式 TUI 的 Claude Code**（[`src/agency/src/standby.rs`](../../src/agency/src/standby.rs)）：正文投递与 `turn.start` / `turn.complete` 判定已全部迁到自建 `--plugin-dir` 插件 + Agency UDS，Herdr 只剩**往 PTY 敲一行短 marker + 启停窗格**（用 `tmux send-keys -l` + `Enter` 完全等价）。

换言之：**Herdr 作为「Agent 控制器」的立身之本已经消失，实际只剩「TUI / 观测窗格容器」这一层职责。**

### 3.2 实现层脚注：`tmux 3.7c` vs `Herdr 0.9.3` 的实测内存与体积对比

我们在 [`docs/research/tmux-runtime.md`](../../docs/research/tmux-runtime.md) 和 [`docs/research/runtime/herdr.md`](../../docs/research/runtime/herdr.md) 中用同一套口径（**1 个守护进程挂 10 个窗格，扣除子进程本身**）实测过两者的数据：

| 指标（单守护进程 + 10 窗格） | `tmux 3.7c`（官方 `tmux-builds` 静态包） | `Herdr 0.9.3` | 对比差距 |
| --- | --- | --- | --- |
| **二进制体积** | **1.62 MiB**（macOS）/ **2.2 MiB**（Linux），tar.gz 仅 **0.62 MiB** | **~18–21.7 MiB** | **`tmux` 小 ~11×** |
| **空载 10 窗格常驻 RSS** | **1.95 MiB**（Linux）/ **3.7 MiB**（macOS） | **19.9 MiB**（Linux）/ **26.2 MiB**（macOS） | **`tmux` 省 ~7–10×** |
| **输出负载下 RSS**（10 窗格各吐 200 KB，共 2 MB 原始输出） | **8.6 MiB**（默认 `history-limit 2000`）/ **21.9 MiB**（保留全部历史） | **112 MiB**（内存富终端 cell 结构对原始字节有 **~45× 放大**） | **`tmux` 省 ~5–13×** |
| **人类旁观体验** | 原生支持 `tmux -S <sock> attach -r`（只读旁观，不怕手滑敲进按键）、标准滚屏搜索、`capture-pane` | 需通过 `herdr tui` 或 `herdr pane read` JSON 接口 | `tmux` 原生支持只读 attach 且为终端工业标准 |
| **三平台静态分发与 P0 验证** | Linux x86_64、macOS arm64、macOS x86_64 零系统依赖静态运行、`-S <sock>` 隔离、`pipe-pane`、`remain-on-exit` **已在 [`tmux-runtime.md`](../../docs/research/tmux-runtime.md) 全部验通** | 已集成在 `src/packaging/dependencies/` | 两者均满足当前支持矩阵（Ubuntu 24.04 x86_64 + macOS 15 arm64） |

### 3.3 分步落地路径（接口先收窄、底座单独换）

为避免把 `Herdr → tmux` 的底座替换塞进当前正在并发推进的 H1–H6 PR 里引发大面积冲突（因为 [`herdr.rs`](../../src/agency/src/herdr.rs)、Claude [`standby.rs`](../../src/agency/src/standby.rs)、Codex [`codex.rs`](../../src/agency/src/codex.rs) 和打包脚本是共用的），建议按两步拆解：

1. **第一步（在 H1–H6 立即执行选项 B，但收窄容器调用面）**：
   - 各家结构化管道 Harness（`agy`、`grok`、`omp` 等）将每轮结构化事件与回复实时格式化追加到 `<session_dir>/view.log`，只调用一个极薄的**「只读观测窗格（Observation Pane）」**原语（开普通命令窗格跑 `tail -f <session_dir>/view.log`、读屏、关窗格），把 `pane_id` 记入会话状态；
   - **严禁在 H1–H6 里新绑任何 Herdr 专属的 `agent.start` / `agent.prompt` / `agent.wait` 接口**。
2. **第二步（单开一个终端容器底座替换包：Herdr → `tmux 3.7c`）**：
   - 单独排一包：将 `src/packaging/dependencies/` 中的 Herdr 替换为 `tmux 3.7c` 官方静态二进制；将 `src/agency/` 的窗格原语切到 `tmux -S <agency_sock>`（同一 Agency root 下所有会话共用 1 个 `tmux` server，不同会话对应不同 window/pane）；将 Claude [`standby.rs`](../../src/agency/src/standby.rs) 的短 marker 注入改为 `tmux send-keys -l` + `Enter`；
   - 因为第一步已将 H1–H6 收窄到纯窗格原语，第二步替换底座时 H1–H6 业务代码无需返工。

---

## 四、一并存盘的其余两项待裁定问题（裁定点 2、裁定点 3）

### 4.1 裁定点 2：H1–H6 的代码结构与落盘 schema 是否统一

当前 H1（#467 `agy.rs`）、H2（#461 `grok.rs`）、H3（#464 `omp.rs`）由三席并行编写，虽然都在做「拉起 JSON-lines 子进程 → 发 prompt → 扫事件流判 `turn.complete` → 落盘会话目录」，但各自独立写了一套，落盘结构与代码重复度如下：

| 维度 | H1 Antigravity（#467 `agy.rs`，882 行） | H2 Grok（#461 `grok.rs`，967 行） | H3 DeepSeek `omp`（#464 `omp.rs`，834 行） |
| --- | --- | --- | --- |
| **调用形态** | 每轮 `agy --mode agent -p --output-format stream-json [--resume <id>] <prompt>` | 进程常驻或按轮 `grok acp`（JSON-RPC 2.0 `initialize` → `session/new` / `session/load` → `session/prompt`） | 进程常驻 `omp --mode rpc`（JSON-lines `prompt` → `get_state` → `agent_end`） |
| **落盘目录与文件** | `sessions/<session_id>/`：`state.json` + `turn-XXXX.jsonl` | `grok/<session_id>/`：`session.json` + `events.jsonl` | `omp/<session_id>/`：`state.json` + `events.jsonl` |
| **重复的样板逻辑** | 子进程进程组拉起与超时强杀、stdout/stderr 行缓冲读取、凭据环境检查、错误分类 | 同左（且后续 H5 Kimi、H6 Qwen 也走 ACP / `stream-json`，与 H1/H2 高度同构） | 同左 |

**待拍板选项**：

- **选项 2A（推荐）**：在补齐选项 B（只读观测窗格）时，顺手把**会话落盘路径与基础元数据名对齐**（统一为 `<harness>/<session_id>/state.json` + `events.jsonl` + `view.log`）；等 H1–H3 合入后、H4–H6 开工前，抽一个 ~200 行的公共 JSON-lines / ACP 会话底座模块供各家共用。
- **选项 2B**：在 H1–H3 合入前就先抽公共模块，三席基于公共模块重写后再合。
- **选项 2C**：保持各家完全独立，不强求统一目录名与公共底座。

### 4.2 裁定点 3：跨平台凭证检查与小活 Y（无交互探活）的边界

各家 Harness 在 macOS 与 Ubuntu 上的登录态/凭据存储机制存在跨平台差异（见 [`docs/research/macos-keychain-and-harness-credentials.md`](../../docs/research/macos-keychain-and-harness-credentials.md)）：

- **Antigravity（`agy`）**：macOS 上默认将 OAuth 状态保存在 `~/Library/Application Support/Antigravity/User/globalStorage/state.vscdb`，加密密钥保存在 **macOS 系统 Keychain**（服务名 `antigravity Safe Storage`）；在无 GUI Keychain 的 Linux（Ubuntu）或被清洗了环境的进程里，则依赖环境变量或凭据根配置；
- **Grok（`grok`）**：读 `~/.config/grok/config.toml` 或 `GROK_API_KEY` / `XAI_API_KEY`；
- **DeepSeek（`omp`）**：读 `~/.omp/agent/agent.db`（SQLite `auth_storage` 表）或对应 API key 环境变量。

同时，[`09-p25-kickoff.md`](./p2-control-20260906/09-p25-kickoff.md) §六 排有 **小活 Y（10 家 Harness 的无交互探活命令表）**。

**待拍板选项**：

- **选项 3A（推荐）**：H1–H3 严格遵守 [`CONSTRAINTS.md`](../../CONSTRAINTS.md)「产品只测试前提是否满足，不满足就诚实报错并告诉用户怎么做，不替用户做 workaround」；至于 10 家 Harness 在 macOS 与 Ubuntu 双端到底传哪些最小环境变量、跑哪条无交互命令探活，统一收口到 **小活 Y** 一次性实测建表。
- **选项 3B**：不等小活 Y，要求 H1–H3 各自在自己的 PR 里先把 mac + ubuntu 双端的跨机器凭据搬运或免 Keychain 方案跑通。

---

## 五、需与设计文档主笔（Claude）确认的落点清单

1. **开工书 [`09-p25-kickoff.md`](./p2-control-20260906/09-p25-kickoff.md) §四 H1–H6 的任务书与验收第 3 条**：
   - 明确写清「控制面走各家原生结构化协议（`stream-json` / `acp` / `rpc`），观测面挂终端只读观测窗格（选项 B）」，消除写席与审席对「在容器里起得来、发两轮提示词 `turn.complete` 对得上」到底是「用容器敲键盘控制」还是「原生协议控制 + 容器窗格观测」的理解分歧。
2. **终端容器底座替换（Herdr → `tmux 3.7c`）的排期与文档镜像面**：
   - 在开工书排入独立的「终端容器底座替换包（Herdr → `tmux 3.7c`）」，明确与 H1–H6 的接口边界（H1–H6 只调窄窗格原语）；
   - 确定何时同步更新 [`docs/design/participant.md`](../../docs/design/participant.md)、[`docs/research/README.md`](../../docs/research/README.md)（将终端运行时决议从 Herdr 改回 `tmux 3.7c`）与 [`docs/design/references/decision-history.md`](../../docs/design/references/decision-history.md)。
3. **裁定点 2（落盘 schema 与公共层时机）与裁定点 3（与小活 Y 的边界）**：
   - 确认是否采纳 **2A + 3A** 写进开工书。

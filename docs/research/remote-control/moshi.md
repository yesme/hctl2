# Moshi

> 类别:⑤ 远程操控与会话同步 · 证据编号:E-L1-MOSHI<br>
> 状态:证据审计(闭源,行为口径) · 钉定版本与公开资料快照见文内「审计基线」;发布后正文不改,只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-l1-moshi"></a>
## E-L1-MOSHI · Moshi

### 它是什么(产品形态与运营状态)

Moshi(Moshi Tech Limited,开发者 GitHub 账号 [rjyo/Joel](https://github.com/rjyo))是把本机编码代理会话搬到手机上的移动终端:iOS 首发 2026-01-21,Android 2026-05 上架,另有 Moshi Desktop 与 moshi-hook 内嵌的本机 Web 客户端(127.0.0.1:24544)。App Store 应用名直接叫「Moshi: herdr/tmux on SSH/MOSH」——把 Herdr 当一等多路复用器,且官方 Skill 明说「Herdr 优先于 tmux」。运营极活跃:App 近乎每周发版(2026-08-28 出 3.13.0),moshi-hook 几乎每日发版(2026-08-29 出 v0.3.12)。定价:免费层(SSH、无限会话、push、用量追踪)+ Pro $7.99/月、$69.99/年、$199 买断([pricing](https://getmoshi.app/pricing));Mosh/多路复用器自动重挂/图片粘贴/Diff 是 Pro 项。

### 行为证据(远程操控与会话同步视角,含公开资料与公开代码链接)

- **拓扑**:手机 ⇄ 主机走 SSH/Mosh/Eternal Terminal 直连;主机上跑闭源 Go 单二进制 daemon `moshi-hook`(CDN 分发,brew tap 安装),开三个口:agent hook 子进程用的 Unix socket、仅回环的 gateway `127.0.0.1:24543`(手机经 SSH 本地转发访问,无鉴权)、到 `api.getmoshi.app` 的长连 WebSocket(bearer hostSecret)。五个接口面的完整线协议**公开成文**:[docs/api.md@a1b7a55](https://github.com/rjyo/homebrew-moshi/blob/a1b7a5502af774fa0f182eb5fb3c14f670371c8f/docs/api.md)。
- **数据分界(声明 vs 可验证)**:官方声称 transcript、diff、文件只走 SSH 转发的本地 gateway「never passes through the Moshi backend」;经厂商服务器的只有通知摘要(prompt ≤200 字符、回复 ≤80、审批命令 ≤256 + 项目/会话/模型元数据)([docs/hooks](https://getmoshi.app/docs/hooks))。协议文档与分界描述公开可查,但 server 与二进制闭源,无 E2E 加密声明,分界只能按「声明+协议文档」采信。
- **观察通路**:Chat View 不是终端字节投影——gateway 直接读各 agent 的**本地 transcript 文件**(Claude JSONL、Cursor SQLite、Hermes state.db 等,逐 agent 枚举了路径解析规则)流式推给手机,结构化成消息/工具卡/计划/问题;终端本体仍是权威。working 状态另以 250ms 轮询终端文本产生 ephemeral 预览行。支持矩阵分三档:Tier A 原生 Chat View(Claude Code/Codex/OpenCode/Cursor/Kimi/Grok 等 9 个)、Tier B 仅生命周期事件、Tier C 纯终端。
- **输入通路(对 HCTL2 输入租约最有参考价值)**:结构化回答走「verified TUI bridge」——`/v1/questions/answer`、`/v1/plans/answer`、`/v1/approvals/answer` 在注入按键前**重新核对屏幕上可见的问题文本、选项标签、plan 指纹**,不匹配返回 409 拒绝注入;自由文本 `/v1/prompt` 明确写明无屏幕校验;`/v1/keys` 只放行固定白名单键、每次 ≤8 个。审批的云通路是阻塞式:hook 子进程 → daemon → WSS → 厂商 server → push → 手机决定 POST 回 server → WSS 回 daemon → hook stdout 放行 agent。
- **注意力提醒**:hook 事件归一成六类(approval_required/task_complete/session_started/session_ended/tool_running/tool_finished),每会话只保一行、工具事件按 5s/会话节流;「事件过滤准则」成文(丢弃流式碎片/子代理事件等)([docs/hooks.md@a1b7a55](https://github.com/rjyo/homebrew-moshi/blob/a1b7a5502af774fa0f182eb5fb3c14f670371c8f/docs/hooks.md))。两个在场感知开关:`suppress-push-while-unlocked`(Mac 本地解锁时推送转静默)、`suppress-nested-agent-push`(agent 套 agent 的事件可屏蔽,并明示副作用是嵌套审批也不可见)。iOS Live Activity 显示会话+最新事件,task_complete 只更新不弹横幅;Apple Watch 可腕上审批。另有自定义 webhook:`POST https://api.getmoshi.app/api/webhook {token,title,message}`。
- **hook 安装机制**:`moshi-hook install` 写各 agent 配置(Claude `~/.claude/settings.json`、Codex `hooks.json`、OpenCode/Pi/OMP 装 TS 插件、Hermes 装 Python 插件,逐 agent 列表见 [docs/usage.md@a1b7a55](https://github.com/rjyo/homebrew-moshi/blob/a1b7a5502af774fa0f182eb5fb3c14f670371c8f/docs/usage.md));默认**不装** PreToolUse/PostToolUse(太吵),用户手动加。Herdr 上没有屏幕抓取回退,hooks 是唯一 prompt 来源(与 tmux 不同)。
- **会话连续性**:两层——传输层 Mosh/ET 抗切网,进程层 tmux/Herdr 保命;Pro 自动重挂;会话可导出 `MOSHI_CLIENT=1` 供 shell 侧探测。配对走 Easy Pair:主机 `moshi host setup` 出 QR,手机扫码换 SSH 公钥安装 + hostSecret(文档明示扫码即得 SSH 访问的边界风险)。

### 审计基线(版本历史时间线 + 公开代码钉定 + 资料快照日期)

- App(iOS)版本历史(App Store 快照 2026-08-30,覆盖 3.6.1→3.13.0):3.6.1(06-21)Chat View 支持 Codex;3.7.0(07-06)图片经 hook 投递;3.8.0(07-12)YubiKey PIV、tmux/Herdr 深链;3.9.x(07-17~22)Chat View 扩到 Pi/OMP/Kimi、深链可切窗口;3.10.x(07-27~30)会话可直接开在 Chat View、Grok 支持;3.11~3.12(07-31~08-21)Live Activity 显示当前 prompt、通知点按回到对应 Terminal/Chat View、Cursor 支持、Chat View 默认开启;3.13.0(08-28)Windows/WSL hooks、BYOK 听写、Chat View 里改模型与 reasoning effort、`/`` @`` $` 自动补全。首发 2026-01-21。
- moshi-hook 版本线(tap 提交史):v0.1.0 2026-04-28 → v0.3.12 2026-08-29,近乎每日一版。
- 公开代码钉定(均为开发者个人账号,无组织):
  - [rjyo/homebrew-moshi@a1b7a55](https://github.com/rjyo/homebrew-moshi/tree/a1b7a5502af774fa0f182eb5fb3c14f670371c8f)(tap + api/usage/hooks/windows 四份协议文档;二进制闭源、tap 无 LICENSE)
  - [rjyo/moshi-hooks@d67fb0d](https://github.com/rjyo/moshi-hooks/tree/d67fb0d95a1d945580cc38414aaa2cba2ce2ee6e)(MIT,已废弃的第一代 TS hook 适配器,npm `moshi-hooks` 1.1.1;可读到早期「hook stdin → Moshi API → APNs → Live Activity」全链路源码)
  - [rjyo/moshi-skill@d094357](https://github.com/rjyo/moshi-skill/tree/d09435760329125c30920210ccb1eb1e482f0700)(agent 读的主机就绪/Herdr 优先 Skill,`npx skills add rjyo/moshi-skill`)
  - [rjyo/herdr-window-title-sync@b07f114](https://github.com/rjyo/herdr-window-title-sync/tree/b07f1140b7308d66487b2f4be546c0c7db065569)(MIT,Herdr 插件示例:manifest 声明订阅 `pane.agent_status_changed` 等事件)
  - [rjyo/claude-code-moshi-notify@50db614](https://github.com/rjyo/claude-code-moshi-notify/tree/50db6140c977dee8f260bbe9d7aae7da9839bd9c)(fork 自 maplefukku 同名仓库;Stop hook 读 transcript 尾部发 webhook 的脚本示例)
- 官方文档快照:https://getmoshi.app/docs/*(introduction/hooks/chat-view/security-sync/herdr/notifications/subscription 等约 40 页,2026-08-30 读取)。

### 采用与边界(复用决策:仅参考行为)

**结论:仅参考行为**(app、daemon、server 均闭源),但协议文档与两个 MIT 仓库可作行为佐证引用。值得借的具体行为:
1. **注入前屏幕校验**(verified TUI bridge):远程结构化回答在落键前核对现场可见状态,不匹配即 409——与 HCTL2「输入必须经输入租约」互补,是「租约有效 ≠ 现场未变」的防陈旧答案机制;且诚实区分「有校验的回答通道」与「无校验的自由文本通道」。
2. **通知摘要与全量数据分通道**:云端只过 200/80/256 字符级摘要,transcript/diff 走 SSH 隧道内本地 gateway——控制面/数据面分离的现成口径,可对照 HCTL2 票据抵达执行现场的设计。
3. **在场感知的提醒抑制**:本地控制台解锁时静默推送、嵌套 agent 事件抑制(并明示盲区)、每会话单行 + 节流的六类事件模型——注意力提醒的完整参考实现口径。
4. **能力协商而非版本比对**:gateway 首帧发布 `capabilities` 数组,客户端按能力选路径——多版本共存的执行面接口演化方式。

**对既有观察清单行的修正**:原「移动终端、钩子与注意力提醒、TUI Chat 投影;闭源;只参考用户体验和互操作行为」方向正确,两处需改:①「TUI Chat 投影」实为**读 agent 本地 transcript 文件的结构化投影**(终端字节视图另存),不是 TUI 解析;②2026-08-29 复审「公开资料只能验证移动/浏览器控制行为」已过时——moshi-hook 五个接口面的线协议、hook 事件模型、逐 agent 安装文件清单均已公开成文,互操作机制可验证到 changelog 级。另需登记:Moshi 与 HCTL2 已选定的 Herdr 深度耦合(Herdr 优先、hooks 是 Herdr 上唯一 prompt 来源),其 Herdr 集成面(`herdr pane send-keys`/`pane read`/插件事件)对 HCTL2 是直接可对照的现成用法。

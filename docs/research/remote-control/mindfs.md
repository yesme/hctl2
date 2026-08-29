# MindFS

> 类别:⑤ 远程操控与会话同步 · 证据编号:E-L1-MINDFS<br>
> 状态:证据审计 · 钉定版本与许可见文内「审计基线」;发布后正文不改,只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-l1-mindfs"></a>
## E-L1-MINDFS · MindFS

### 它是什么(产品形态与投入口径)

MindFS 自称「AI Agent 远程访问网关」:单个 Go 二进制跑在工作机上,把本机已装的编码代理 CLI(Claude Code、Codex 及十余个 ACP 代理)包装成可从浏览器/手机访问的多会话服务。展示端是内嵌 Web PWA(渐进式网页应用)加 Android(Capacitor WebView 壳)与 HarmonyOS 壳。

- 远程访问走「反向隧道」:本机主动向厂商中继 relay.a9gent.com 发起出站 WebSocket,用 yamux 复用回传公网请求([relay/service.go#L331-L350](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/relay/service.go#L331-L350));也可关掉中继走私网直连。
- 代理一律经 SDK/ACP 结构化驱动,三种协议:claude-sdk、codex-sdk、其余全部 ACP([protocol.go#L16-L25](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/agent/protocol.go#L16-L25))。
- `creack/pty` 只用于每会话附带的长驻命令 shell,不承载代理主会话([commandexec/process_unix.go#L21-L42](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/commandexec/process_unix.go#L21-L42))。
- 投入口径:2026-02-06 首提交;近 90 天 220 次提交,最近提交 2026-08-28,但其中 212 次出自单一作者 yandc,实质是高强度个人项目。

### 设计亮点(远程操控与会话同步视角,含代码证据链接)

**外部会话导入与双向续接(独特点)**。它不拥有代理的会话,而是维护绑定表:自有会话键 ↔ 代理侧 session id,外加「外部源游标」三元组(源文件路径、字节大小、mtime 纳秒)([session/manager.go#L86-L96](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/session/manager.go#L86-L96))。

- 导入源:Claude Code 的 `~/.claude/projects` JSONL([claude/importer.go#L65-L68](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/agent/claude/importer.go#L65-L68));Codex 的 `~/.codex/sessions`([codex/importer.go#L70-L75](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/agent/codex/importer.go#L70-L75));ACP 代理不读文件,直接用协议自身的 `session/list` + `session/load` 回放历史完成导入([acp/importer.go#L132-L173](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/agent/acp/importer.go#L132-L173))。
- 增量同步是拉式的:每次打开会话触发一次尽力而为的 delta 同步;游标(大小+mtime)未变直接跳过,变了整文件重读、按上次最末时间戳过滤增量,加每会话互斥锁防并发([usecase/external_sessions.go#L267-L374](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/api/usecase/external_sessions.go#L267-L374)、触发点 [http.go#L800-L812](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/api/http.go#L800-L812))。
- 不做真正的冲突消解:外部文件是唯一事实,自己只追加,增量靠时间戳与序号截断去重。
- 子代理会话按父子拓扑排序导入成会话树([usecase/external_sessions.go#L376-L456](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/api/usecase/external_sessions.go#L376-L456))。
- 反向续接不需要导出:经 SDK 驱动的本来就是真 CLI 会话,记住 id 即可在 CLI 里 resume;自身重启后也靠绑定表 `WithResume`/`ResumeThread` 恢复,并支持从历史某条消息 fork([claude/session.go#L92-L114](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/agent/claude/session.go#L92-L114))。

**观察通路:结构化投影加双游标断点重放**。

- 消息正文持久化为每会话 JSONL 交换日志,SQLite 只存元数据/绑定/任务([manager.go#L32-L33](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/session/manager.go#L32-L33)、[L65-L96](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/session/manager.go#L65-L96))。
- 历史按交换序号增量拉取;进行中的回复流用「基准交换序号:事件序号」复合游标,客户端带游标声明 `session.ready` 即可从断点重放直至追平直播([stream_hub.go#L643-L661](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/api/stream_hub.go#L643-L661)、[L750-L775](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/api/stream_hub.go#L750-L775))。
- 命令 shell 的终端字节流在重放缓冲里被合并成尾部 256KB 快照,重连不重灌全量字节([stream_hub.go#L677-L717](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/api/stream_hub.go#L677-L717))。

**输入通路:有排队,无审批**。

- 代理回复中收到的新输入进服务端队列,可改/删/插队,取消回合时队列冻结,空闲后按序派发([ws.go#L716-L729](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/api/ws.go#L716-L729)、[L1040-L1052](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/api/ws.go#L1040-L1052))。
- 工具权限三条协议全部自动放行:Claude 的 canUseTool 除 AskUserQuestion 外一律 Allow([claude/session.go#L299-L330](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/agent/claude/session.go#L299-L330));Codex 直接 ApprovalModeNever 加 SandboxModeFullAccess([codex/session.go#L53-L63](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/agent/codex/session.go#L53-L63));ACP 的 RequestPermission 自动选第一个 Allow,源码注释明写「TODO: 转发给前端审批」([acp/process.go#L295-L322](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/agent/acp/process.go#L295-L322))。
- 真正路由到远端 UI 的只有代理主动的提问(AskUser)一类。

**注意力提醒与加密**。

- 会话完成/待输入两类事件触发 Web Push(VAPID 密钥自动生成)加可选本地通知脚本([appcontext.go#L906-L953](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/api/appcontext.go#L906-L953));Android 另有前台轮询服务补推送通道([ReplyPollerService.java#L62-L116](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/android/app/src/main/java/com/mindfs/app/ReplyPollerService.java#L62-L116))。
- 因流量过厂商中继,做了可选的应用层端到端加密:配对密钥加 ECDH P-256/HKDF 派生会话密钥,请求带 HMAC 证明,响应整体加密,使中继不可读([e2ee/crypto.go#L32-L80](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/e2ee/crypto.go#L32-L80)、[http.go#L162-L195](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/api/http.go#L162-L195));默认关闭。
- ACP 侧是每代理常驻进程复用多会话的池,空闲会话默认 72 小时释放([pool.go#L178-L236](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/agent/pool.go#L178-L236))。

### 审计基线(钉定 commit/版本/许可 + changelog 与演化脉络)

- 钉定 commit:`8f5bd3e8090e9a638a41a550d2b0e633003da7e9`(默认分支 main HEAD,2026-08-28),即 `v0.4.9-3-g8f5bd3e`;最近 release 为 v0.4.9(2026-08-20)。
- 与 2026-08-29 运行服务复审钉定的 `8f5bd3e` 为同一提交,`git log 8f5bd3e..HEAD` 为空,本次审计期间无新代码演化。
- 许可:LICENSE 为 AGPL-3.0 全文,README 标注「AGPL v3」;无 SPDX 头、无 "or later" 声明,按 AGPL-3.0-only 口径对待最稳妥。
- 演化脉络(release-notes.md,0.1.8→0.4.9,2026-05→2026-08):0.2.x 补上端到端加密、Android 壳、外部会话自动/手动同步;0.3.x—0.4.x 转向任务看板(worktree 隔离并发任务、模板化阶段流水线)、计划任务、国际化、更多代理接入(grok/CodeBuddy/dsh)、空闲会话释放、开机自启与环境快照恢复。未见公告 breaking change,数据层以 `ALTER TABLE` 就地迁移。
- 方向判断:从「远程会话网关」向「带任务编排的个人工作台」扩张;治理语义仍然没有——任务看板只是模板化的阶段执行器([kanban/task_store.go#L66-L106](https://github.com/a9gent/mindfs/blob/8f5bd3e8090e9a638a41a550d2b0e633003da7e9/server/internal/kanban/task_store.go#L66-L106))。

### 采用与边界(建议复用决策 + 不采用边界 + 对既有观察清单行的确认/修正)

**建议复用决策:仅参考行为。** AGPL-3.0 使「采用为依赖/移植组件」需整体权衡传染边界,而它真正独特的是行为设计而非可拆组件;更关键的是其安全模型与 HCTL2 治理模型正面冲突——远程输入不经任何租约或审批,三协议全自动放行工具权限、Codex 干脆全权限无沙箱,这正是 HCTL2 要用输入租约(TTL/代次/CAS)与控制面票据杜绝的形态。

值得参考的行为:

1. 绑定表加「路径+大小+mtime」游标的外部会话增量同步与子代理树导入,可作 Terminal/Chat Room 场景导入既有 Harness 会话历史的 L1 观测参考;
2. 「基准序号:事件序号」双游标断点重放与终端流尾部快照合并,是多端观察重连的干净样板;
3. 输入排队的冻结/插队语义与 AskUser 审批路由通路——其排队只到会话级串行,没有租约概念,借形不借底。

不采用边界:不引其代码与进程形态;不接其厂商中继;其自动放行权限的 provider 适配不可作 HCTL2 执行面组件;本文所述 Session/Conversation/Project/Task 均为 MindFS 自有名词,不映射 HCTL 公开数据结构。

对既有观察清单行:两条均确认,并作三点细化修正。

- 旧行「仓库本地 Session、外部 Session 导入与同步;AGPL;只参考协议与行为,Task Board 不定义 L3」成立;许可细化为 AGPL-3.0(only 口径)。
- 2026-08-29 复审结论全部被源码支持:Go server、Codex/Claude SDK 加 ACP 会话池、`creack/pty` 仅命令 shell、代理主会话非任意 TUI raw PTY、不能管理 Terminal 的 PTY 会话。
- 细化:「SQLite/task/sync」应表述为 SQLite 只存会话元数据、代理绑定与任务,消息正文在每会话 JSONL;新增此前未记录的关键事实——工具审批在三条协议上全部自动放行(含 ACP 侧 TODO 注释自认),这把其定位进一步钉死在 L1 邻近证据,不具备任何治理面复用资格。

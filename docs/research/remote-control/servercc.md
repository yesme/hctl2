# ServerCC

> 类别:⑤ 远程操控与会话同步 · 证据编号:E-L1-SERVERCC<br>
> 状态:证据审计(闭源,行为口径) · 钉定版本与公开资料快照见文内「审计基线」;发布后正文不改,只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-l1-servercc"></a>
## E-L1-SERVERCC · ServerCC

### 它是什么(产品形态与运营状态)

原生移动 app(iPhone/iPad/Mac/Vision,Android 已上架 Google Play),开发者刘佳琳(Lollipush,bundle `com.lollipush.ServerCC`)。定位一句话:「$ ssh user@server 'claude' — Now from your iPhone」——手机经 SSH 直连自己的服务器,统一管理七个编码代理 CLI(Claude Code、Codex、Grok Build、OpenCode、Kimi Code、Pi、OMP)。免费 1 台服务器;Pro 终身 $14.99 / 月订 $1.99。运营活跃:2026-02-25 v1.0 → 2026-08-29 v2.1.0,月均多次发版([App Store](https://apps.apple.com/us/app/servercc-remote-claude-code/id6759306046)、[官网](https://servercc.app))。

### 行为证据(远程操控与会话同步视角)

- 拓扑:设备→SSH 直连用户服务器,无厂商中继。官方声称:「All SSH connections are made directly from your device to your servers. We have no visibility into these connections」;SSH 私钥只存 iOS Keychain、「never leave the device」;遥测仅 TelemetryDeck 匿名统计([privacy](https://servercc.app/privacy)、[ssh-keys](https://servercc.app/docs/ssh-keys))。可选通路:app 内嵌用户态 Tailscale(泰络思内网穿透)以临时节点入网([tailscale](https://servercc.app/docs/tailscale));VNC/端口预览推荐走 SSH 隧道([vnc](https://servercc.app/docs/vnc))。唯一云组件是 Sandbox Trial:厂商代开临时沙箱机,限时销毁,预装三个已登录代理([sandbox-trial](https://servercc.app/docs/sandbox-trial))。以上均为官方声明,闭源 app 无法核验。
- 厂商会话恢复:workspace 自动索引服务器上 `~/.claude/projects/` 的历史会话,三级发现(session index → 目录扫描 → JSONL 重建),恢复即 `claude --resume <sessionId>`;七个代理统一「解析各代理自身的 session store,按会话 ID 重连,而不是恢复最近一个」([sessions](https://servercc.app/docs/sessions)、[agents](https://servercc.app/docs/agents))。会话身份的表达 = 代理自己的 sessionId + workspace(项目目录)+ git 分支,列表行附首条 prompt、AI 摘要、消息数。
- 外部接管:v1.5.0 起可 attach 桌面终端里外部启动的、跑着 Claude/Codex 的 tmux 会话,「从手机接管长时任务而不重启」;离场时显式二选一「后台继续跑 / 立即停止」([persistent-sessions](https://servercc.app/docs/persistent-sessions))。v2.0 起集成 herdr 0.8:server 页列出 herdr spaces 及其 working/blocked/done/idle 状态,attach 后「看到与 herdr 相同的终端,实时,可输入」,还可把 space「收养(adopt)」为正式 workspace([herdr](https://servercc.app/docs/herdr))。要点写实:公开资料中接管就是 tmux/herdr 共享 attach,没有「谁在控制」的显式状态、没有输入互斥,也没有交还动作——本机终端在接管期间同屏同权。
- 注意力提醒:Running 页对后台会话给两态——Vibing(终端有近期输出)/ Waiting(空闲待输入),即输出活动启发式;「后台会话由活跃转空闲」时推送,「每个后台期每会话至多一次」([running](https://servercc.app/docs/running))。推送到达手机的机制未公开(官方称无服务器组件,博客也未说明检测组件与投递通道,[blog](https://servercc.app/blog/claude-code-mobile-notifications))。
- 输入与审批:审批不结构化,就是终端里回答代理的确认提示;app 提供各代理权限模式切换(Claude Code:Default/AcceptEdits/Auto/Bypass/Plan 等)、快捷键 chips、多行草稿编辑、NO_FLICKER 全屏防闪模式([terminal](https://servercc.app/docs/terminal))。

### 审计基线(版本历史 + 公开代码钉定 + 快照)

- 版本时间线(官网 [changelog](https://servercc.app/changelog),全量,快照 2026-08-30):v1.0(2026-02-25 首发)→ v1.1.0(03-16 tmux 持久会话)→ v1.2.0/1(03-19/20 Codex)→ v1.3.0(04-01 NO_FLICKER)→ v1.4.0(04-05 VNC)→ **v1.5.0/1(04-21/28 tmux 会话接管 + 接管态完成推送)** → v1.6.0(05-28 Claude Code Agent View)→ v1.7.0(07-07 iCloud 备份、照片附件)→ v1.8.0-2(07-26~08-12 新增五个代理、并行会话)→ **v2.0.0(08-26 Sandbox Trial、herdr 集成、iPad 重排)** → v2.1.0(08-29 接管会话 Background Run、终端主题)。
- 公开代码:无。2026-08-30 搜索 GitHub(`servercc` 8 个同名仓库均与本产品无关;`lollipush` 账号仅个人旧仓库)、npm(无结果)、Homebrew(formula/cask 均 404)。服务器侧不安装任何 ServerCC 自有组件:装的是各代理厂商官方安装器、用户自装 tmux、herdr 官方脚本([quick-start](https://servercc.app/docs/quick-start))。
- 资料快照:官网 docs 22 页、changelog、blog 9 篇(2026-07-17~08-26)、App Store 页(v2.1.0)、隐私政策,均 2026-08-30 抓取。

### 采用与边界

- 复用决策:仅参考行为。全闭源,无 CLI 伴侣、无 hook 脚本、无自托管组件可审计。
- 值得借的具体行为:① 厂商会话身份表达——从代理自身 session store 反向索引(含 JSONL 重建兜底),恢复严格按 sessionId 而非「最近一个」,且对七种代理统一同一套「检测/启动/历史/精确恢复」口径;② 外部会话「收养」入口——把非本 app 启动的 tmux/herdr 会话纳管,并在离场时强制显式选择 keep/stop,算一种朴素的「接管结束态」;③ 注意力提醒的节流语义——输出活动两态 + 「每后台期每会话至多一次」。
- 对既有观察清单行的确认/修正:「外部接管、厂商会话恢复、移动端控制;闭源」——确认。2026-08-29 复审「公开资料只能验证移动/浏览器控制行为」——修正两点:其一,ServerCC 没有浏览器端,是纯原生 app(勿与 QuickTUI 混);其二,公开文档能验证的比「控制行为」更多(恢复机制、herdr 集成、提醒节流),但「接管期间无显式控制权/交还语义」这一空白现在可以写实——它恰是 HCTL2 输入租约要补的东西。

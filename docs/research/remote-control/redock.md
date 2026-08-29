# Redock

> 类别:⑤ 远程操控与会话同步 · 证据编号:E-L1-REDOCK<br>
> 状态:证据审计(闭源,行为口径) · 钉定版本与公开资料快照见文内「审计基线」;发布后正文不改,只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-l1-redock"></a>
## E-L1-REDOCK · Redock

### 它是什么(产品形态与运营状态)

Redock([redock.dev](https://redock.dev))是面向编码代理工作流的移动 SSH/Mosh 终端,自称「AI coding mobile workbench」。个人开发者产品:App Store 开发者署名「尚渝 张」(Zhang Shangyu,X 账号 [@_onecookie](https://x.com/_onecookie)),iOS 首发 2026-05-08,当前 2.6.0(2026-08-28),Android 包 `com.redock.android`;体量很小(iOS 美区评分数仅 7)。运营极活跃:近三个月约 25 个版本,8 月中旬两周连发 7 版。定价:免费层 + Pro $5.99/月、$19.99/年、$39.99 买断,7 天试用([FAQ](https://redock.dev/faq))。围绕「Projects(项目上下文)/ Actions(一键起任务)/ Snippets(片段)」组织,支持 Claude Code、Codex、OpenCode 及任意终端程序。

### 行为证据(远程操控与会话同步视角,含公开资料与公开代码链接)

- **拓扑**:纯客户端,SSH/Mosh 直连自己的主机;官方明确「The terminal data path does not require a Redock session relay」([vs-happy](https://redock.dev/compare/redock-vs-happy)),并以此与 Happy 的加密中继、Claude Code Remote Control 的出站中继划界([vs-claude-code-remote-control](https://redock.dev/compare/redock-vs-claude-code-remote-control))。**没有任何主机侧组件**:全部公开资料中查不到 CLI、daemon 或 hook 脚本;GitHub/npm 也查不到 Redock 自己的公开仓库或包。凭据存平台安全存储(Apple Keychain),配置可 iCloud 同步、本地运行历史不同步;无 E2E 加密声明(直连模式下依赖 SSH 本身)。1.29.1 起内嵌 Tailscale(手机端不必再开 Tailscale App),另有 ngrok、跳板机(jumpserver 直登目标机,2.0.0)与备用地址自动切换(backup address,1.15.0)。
- **观察通路**:终端字节投影,无结构化 Chat 投影——这是与 Moshi 的根本差异;卖点在触屏化的 TUI 体验(平滑滚动、缩放调字号、选择/复制与手势共存)。「Activity」是活动会话/连接的卡片列表(differentiation 博客:「return to existing work through Activity or connection pickers」),2.1.0 起卡片带终端预览。
- **输入通路 · 分阶段输入(staged input)**:核心交互是把「组稿」从终端搬进一个输入暂存区——「Agent instructions are often paragraphs, not single shell commands… On mobile, direct terminal typing is the wrong place to assemble that text」([differentiation](https://redock.dev/blog/redock-differentiation))。可验证的演化:暂存区长按查看历史(1.27.0)、切换连接保留未发送草稿(1.28.0)。语音输入走同一暂存区:录音→转写→**先编辑改错(文件名/术语/标点)再发送**;引擎可选 Apple 本地/OpenAI/火山引擎豆包(Volcengine Doubao,中英混合识别),后两者 BYO API key,key 存平台凭据存储([features/voice](https://redock.dev/features/voice)、[guide/volcengine-speech](https://redock.dev/guide/volcengine-speech))。CJK/IME 可靠性是明示卖点(App Store 描述与 differentiation 博客均列)。无审批路由概念:输入就是终端输入,危险命令的防线只有最佳实践里「破坏性命令不要做成一键 Snippet」的提醒。
- **注意力提醒**:2.0.0(2026-08-14)才引入「Agent status notifications (push, Live Activities, in-app)」,2.2.0 改进设置——仅见于 App Store 版本记录;**触发事件与检测机制完全未公开文档化**(官方 guide 无对应页;更早的 vs-happy 对比页还把 notifications 列为对方强项)。没有主机侧组件的前提下如何在后台产生 push,公开资料查不到,无法验证。
- **会话连续性**:持久层交给多路复用器——tmux 为主(自动创建/恢复免记命令、保留用户已有 tmux 键位,1.28.0),1.29.1 起可选 Herdr 或 psmux(Windows 原生 tmux 类多路复用器,第三方开源 [psmux/psmux](https://github.com/psmux/psmux));传输层 Mosh 抗切网 + 断线重连检测(1.14.0)。8 月多个版本专修 Herdr 手势/滚动/自定义前缀键,说明 Herdr 支持仍在磨合。

### 审计基线(版本历史时间线 + 公开代码钉定 + 资料快照日期)

- App(iOS)版本历史(App Store 快照 2026-08-30,覆盖 1.11.0→2.6.0,即产品几乎全部生命期):1.11.0(06-11)跳板机自定义认证;1.15.0(06-17)备用地址;1.16.0(06-19)Pro 试用;1.21.0(06-25)Web Preview 端口转发;1.27.0(07-15)暂存区长按历史;1.28.0(07-18)切连接保草稿、保留用户 tmux 键位;1.29.1(07-31)内嵌 Tailscale、Herdr/psmux 持久化;2.0.0(08-14)Agent 状态通知(push/Live Activity/in-app)、配置批量导入导出;2.1.0(08-17)Activity 卡片预览、Herdr 自定义前缀键;2.2.0(08-21)多语言;2.3.0(08-24)Web Preview 输 URL、Herdr 点按手势;2.6.0(08-28)Mosh 自定义端口段、修 iOS 17 崩溃。首发 2026-05-08。
- 公开代码:**无**。GitHub、npm 检索均无 Redock 官方仓库/包(2026-08-30);开发者 GitHub 身份亦未能确认。其生态依赖均为第三方开源:tmux、mosh、[herdrdev/herdr](https://github.com/herdrdev/herdr)、[psmux/psmux](https://github.com/psmux/psmux)、yazi。
- 官方资料快照(2026-08-30):[features](https://redock.dev/features)(12 项 + 各 feature 子页)、[guide](https://redock.dev/guide)(11 页:quick-start/tailscale/ssh-keys/windows-ssh/ngrok-ssh/tmux/mosh/best-practices/openai-speech/volcengine-speech)、[FAQ](https://redock.dev/faq)、[blog](https://redock.dev/blog)(10 篇,全是工作流指南、无 release notes)、7 个竞品对比页;另有中文站 [redock.dev/zh](https://redock.dev/zh)。

### 采用与边界(复用决策:仅参考行为)

**结论:仅参考行为**(全闭源、无任何公开代码、互操作机制不可验证)。值得借的具体行为:
1. **分阶段输入暂存区**:把「组稿→审阅→发送」从 PTY 写入中拆出来,草稿跨连接存活、语音转写先落暂存区可编辑再提交——对 HCTL2「输入必须经输入租约」是天然同构的前端形态:暂存区即租约申请前的本地草稿态,值得写进终端接管的交互规范。
2. **BYO 语音引擎 + CJK 明确立场**:听写引擎可换(本地/OpenAI/豆包)、key 用户自持、中英混合识别——中文用户远程口述指令的现成参考,HCTL2 若做语音入口可直接对标。
3. **持久层可插拔**:同一「会话持久化」开关下并列 tmux/Herdr/psmux,把多路复用器当可替换后端而非绑定——与 HCTL2 Herdr 选型互为印证,也提示 Windows 现场可用 psmux 补位。

**对既有观察清单行的确认/修正**:原「分阶段输入、CJK 与语音、Activity 深链;闭源;只参考用户体验」——「分阶段输入、CJK 与语音」**确认**(且有版本记录佐证演化);「Activity 深链」**修正**:Activity 是会话卡片列表(2.1.0 起带预览),「深链」行为公开资料未详述,不宜再写成已核实特性。**需新增**:2.0.0(2026-08-14)起有注意力提醒(push/Live Activity/in-app),但机制未公开、无主机侧组件证据,只能按「声明存在、机制不可验证」登记;观察清单行建议改为「分阶段输入、CJK 与语音、tmux/Herdr/psmux 可插拔持久层;2.0 起有状态通知(机制未公开);闭源无公开代码;只参考用户体验」。

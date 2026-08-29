# QuickTUI

> 类别:⑤ 远程操控与会话同步 · 证据编号:E-L1-QUICKTUI<br>
> 状态:证据审计(server/app 闭源 + 两个开源组件,行为口径) · 钉定版本与公开资料快照见文内「审计基线」;发布后正文不改,只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-l1-quicktui"></a>
## E-L1-QUICKTUI · QuickTUI

### 它是什么(产品形态与运营状态)

廖宇雷(YuLei Liao,GitHub `dualface`,quick-cocos2d-x 作者)的自托管远程终端:用户机器上跑闭源 `quicktui-server`(Rust 二进制,darwin/linux/windows × amd64/arm64),iPhone/iPad 原生 app + 由该 server 直接服务的浏览器客户端,attach 真实 tmux 会话操控编码代理(「Claude Code, Codex, OpenCode, Grok — attach from anywhere, approve in seconds」)。定价:1 台服务器免费;Pro 一次性 $12.99(不限服务器);唯一订阅是 Relay(QuickTUI Cloud 中继,Cloud Basic $4.99 / Plus $14.99 月),仅用于无法直连时。中国区可用(App Store 中国区上架、`dl.quicktui.cn` 安装镜像)。运营活跃:仓库 2026-03-26 创建,iOS 1.0 于 2026-04-02 上架,当前 1.8.5(2026-08-09);server 稳定通道至 20260809-09,preview 通道发版到 2026-08-29;Android 尚在 GitHub preview APK 阶段([官网](https://quicktui.ai)、[App Store](https://apps.apple.com/app/quicktui/id6761338192))。

### 行为证据(远程操控与会话同步视角)

- 拓扑:app/浏览器 →(默认直连:LAN/VPN/Tailscale/Cloudflare Tunnel)→ HTTP+WebSocket :8022 → `quicktui-server` → 会话后端 tmux 3.2+(macOS/Linux)或 qscn(Windows 默认)。不经 SSH:「QuickTUI talks to its own lightweight server over HTTP/WebSocket」([guides/attach-tmux-from-phone](https://quicktui.ai/guides/attach-tmux-from-phone/))。官方声称:配对注册客户端公钥并钉定服务器指纹,连接内用 transcript-bound `device_pop_v1` 证明设备私钥占有、不可重放;「QuickTUI Relay only forwards opaque encrypted frames and cannot read terminal, file, or Agent traffic」([faq](https://quicktui.ai/faq/))。可验证的公开证据:公开仓库里的安装监控器要求 server v2 必须在 `/.well-known/quicktui-server-capability` 通告 `quicktui.e2e.v1`、同主机 `ws://…/e2e` 端点、`pairing_code_v1`、身份指纹与 `device_pop_v1` 才算安装就绪(v1 老架构则是 root token/`QUICKTUI_TOKEN`)——协议名与端点面被 CI 钉死,但实现闭源。
- tmux 依赖的具体形状:attach 真实 tmux(windows/panes/scrollback/copy-mode,三指手势跨服务器切 session/window);桌面 `tmux new-session -s claude claude` 起的会话直接出现在 app 列表,同一会话多端同视图——观察与控制不分离,无「谁在控制」状态。但已超出纯 tmux 直通:v2 server 有会话后端抽象(`session_backend = tmux | qscn`)、文件浏览器、Agent 通知,以及 `chat_transcript_retention_days`(代理转写保留,默认 2 天)——说明 server 在解析代理会话转写,不只是转发终端字节([faq](https://quicktui.ai/faq/))。
- 配对与认证:`quicktui-server pairing qrcode` 生成短时效 QR 扫码配对;可不重启撤销设备;浏览器端密钥仅存于 HTTPS 或 loopback origin。断线恢复:「session keeper」跨 Wi-Fi↔LTE、app 重启、断连自动重挂,会话活在服务器侧。
- 注意力提醒与审批:Agent 通知按设备 opt-in,默认只报事件类别,可选附代理类型/项目/会话名/任务或错误摘要,明确不含终端输出、源码、凭据、完整 prompt;触发检测机制未公开。审批无结构化通道——就是终端里回答代理确认提示(「Attach, approve, detach: 10 seconds」);与 Omnara 对比页自陈「direct by default…nothing has to pass through a third party」([compare/quicktui-vs-omnara](https://quicktui.ai/compare/quicktui-vs-omnara/))。

### 审计基线(版本历史 + 公开代码钉定 + 快照)

- [dualface/quicktui](https://github.com/dualface/quicktui/tree/f216d3c048fe0a71a9c9f9dc4a8e6cc0bddd8abe) @ `f216d3c048fe0a71a9c9f9dc4a8e6cc0bddd8abe`(2026-08-29,逐文件读):官网 + 分发仓。[q.sh](https://github.com/dualface/quicktui/blob/f216d3c048fe0a71a9c9f9dc4a8e6cc0bddd8abe/q.sh)/`q.ps1`(MIT)只做「下载闭源 Rust 安装器 → sha256 校验 → exec」,安装逻辑全在闭源侧;[server-manifest.json](https://github.com/dualface/quicktui/blob/f216d3c048fe0a71a9c9f9dc4a8e6cc0bddd8abe/server-manifest.json) 指 stable `20260805-01` / preview `server2-preview-20260806-201644-54e512488`;Releases 附 minisign 签名与 sha256;[monitoring/](https://github.com/dualface/quicktui/blob/f216d3c048fe0a71a9c9f9dc4a8e6cc0bddd8abe/monitoring/README.md) 是每日 GitHub Actions 跑的公开安装验证器(容器内走用户同款 `curl | sh`,逐项断言 `/v3/healthz`、`/v3/version`、capability 端点与 E2E 配对就绪)。注意:线上站点(如 /faq/)比该仓库 main 新,真源是私有 monorepo `quicktui-mono`,公开仓库是其部分同步镜像。
- [dualface/qscreen](https://github.com/dualface/qscreen/tree/3783e78b47875c7714e4b85a2e0c806b27c75a6a) @ `3783e78b47875c7714e4b85a2e0c806b27c75a6a`(2026-07-21,MIT,Rust workspace v0.3.0,逐文件读):真开源的会话后端(`qscn` CLI + 按需 daemon;Windows ConPTY/命名管道,Unix domain socket),自述「powers persistent Windows terminal sessions in QuickTUI…uses it as the default session backend」。行为要点:多客户端同时 attach、输出广播、各自 detach;PTY 尺寸归「最近活跃客户端」(attach/聚焦/输入即夺尺寸);默认 AttachMode=Frame——daemon 端 vt100 解析后发结构化屏幕帧(rows/runs),明确为了「避免 reattach 时向宿主终端重放 ANSI dump」。
- 版本时间线:server 稳定 releases 2026-04-01(`20260401-05`)→ 2026-08-08(`20260809-09`),7 月中起 stable/preview 双通道高频发版;installer 独立 tag(最新 `installer-20260823-01`);iOS 1.0(2026-04-02)→ 1.8.5(2026-08-09,IAP 含 Pro Lifetime 与 Cloud Basic/Plus);qscreen releases 2026-05-22(preview)→ 2026-07-21。快照:quicktui.ai 首页/faq/install/compare/guides、App Store 页、两仓库全文件,2026-08-30。

### 采用与边界

- 复用决策:闭源 server/app 封顶仅参考行为;qscreen 与 q.sh/q.ps1 为 MIT,可按许可引用或复用(网站文案与品牌资产保留所有权利)。
- 值得借的具体行为:① 公开安装验证器——把「用户同款安装命令跑完、健康/版本/能力端点在同一轮全部有效、E2E 配对能力就绪」做成每日 CI 硬断言,失败自动开监控 Issue;HCTL2 发行 gate 可直接借这个形状;② `/.well-known/…-capability` 能力自描述端点(协议名、配对方式、身份指纹),给「终端凭票据抵达执行现场」一个可探测的握手前置;③ qscreen 的结构化屏幕帧 attach(服务端解析 VT、下发帧而非重放字节)与「最近活跃客户端拥有 PTY 尺寸」的隐式所有权规则——后者正好是 HCTL2 输入租约(显式 TTL/代次/CAS)的反面样本,可作对照写实。
- 对既有观察清单行的确认/修正:「自托管 tmux 加移动端或浏览器终端;应用闭源」——确认。「公开仓库只能证明分发方式」——需修正:quicktui 仓库除分发外还含可执行的安装监控器,钉出了 server v2 的公开 API 面与 E2E 配对协议名;且另有第二个公开仓库 qscreen 是真正开源的会话后端。2026-08-29「公开证据仍指向 tmux」——需修正:macOS/Linux 仍以 tmux 3.2+ 为后端,但 Windows 默认后端是开源 qscn,且 v2 server 的自有传输(HTTP/WS + E2E 隧道 + 设备配对)与代理转写处理都超出「tmux 直通」的口径。

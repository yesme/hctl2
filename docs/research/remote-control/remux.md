# Remux

> 类别:⑤ 远程操控与会话同步 · 证据编号:E-L1-REMUX<br>
> 状态:证据审计 · 钉定版本与许可见文内「审计基线」;发布后正文不改,只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-l1-remux"></a>
## E-L1-REMUX · Remux

### 它是什么(产品形态与投入口径)

Remux 是 iOS 原生的远程 tmux 客户端(Swift/SwiftUI),经 TestFlight 公测分发,当前版本 0.1.0 build 11,MIT 许可。
单一主力作者(henok3878,990/1067 提交)加少量外部贡献者;实际是三件套:本仓库的 App 壳、定制的终端内核
GhosttyKit(二进制形态,来自其 [remux-ghostty](https://github.com/h3nock/remux-ghostty) 分叉,Zig 实现)、以及
SSH 库 Citadel 的私有分叉([按 revision 钉定](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/project.yml#L21-L24))。

无服务端、无账号、无中继,设备直连 SSH([PRIVACY.md](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/PRIVACY.md));
超出「纯客户端」的动作只有三类客户端发起的 SSH 子通道:SFTP 传附件、向服务器 authorized_keys
[安装公钥](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/SSH/SSHPublicKeyInstaller.swift#L1-L30)、
direct-tcpip 端口桥做 localhost 预览,均无驻留组件。

关键分工要看清:Swift 层只做壳与字节搬运;tmux 命令生成、控制模式(control mode)输出解析、拓扑对账、窗格投影、
终端渲染全部在 GhosttyKit 内核里([docs/architecture.md](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/docs/architecture.md)),
CJK 渲染与回滚(内核配置 history 2000 行/scrollback 10000 行,[TmuxSessionController.swift#L283-L295](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Tmux/TmuxSessionController.swift#L283-L295))都由内核承担。

### 设计亮点(远程操控与会话同步视角,含代码证据链接)

- **附着即定位**:SSH exec 通道跑 `tmux -u -C new-session -A -s <会话名> -x <列> -y <行>`,一条命令完成「有则附着、无则创建」并把移动端视口带进初始网格;外层用 `/bin/sh -c` 包裹、参数八进制编码,fish/csh 等非 POSIX 登录 shell 也不会解错([SSHTmuxControlCommandBuilder.swift#L7-L46](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Tmux/SSHTmuxControlCommandBuilder.swift#L7-L46)、[八进制编码 #L60-L67](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Tmux/SSHTmuxControlCommandBuilder.swift#L60-L67))。会话发现走独立 exec 通道 `list-sessions`,不占控制通道([TmuxSessionDiscovery.swift#L21-L36](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/SSH/TmuxSessionDiscovery.swift#L21-L36))。
- **单一路由身份**:窗格/窗口定位只认 tmux 自己的数字 ID(`%pane`/`@window`),类型化为 `TmuxPaneID`/`TmuxWindowID`;展示层 UUID 经注册表可逆映射,渲染表面重建换 UUID 不换 tmux 身份([TmuxIdentity.swift#L5-L33](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Tmux/TmuxIdentity.swift#L5-L33)、[注册表 #L75-L110](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Tmux/TmuxIdentity.swift#L75-L110))。所有操控命令(select-pane/split-window/kill-pane/resize-pane -Z)都以这些 ID 为目标。
- **传输与解析分层**:Swift 侧一个 actor 把 SSH 字节流接到内核 tmux 客户端(feed 进、outbound 出、action 回调),单写者队列保证出站命令严格有序;链路是一次性附着物,任何读/写失败立即作废整条传输([TmuxSessionLink.swift#L12-L100](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Tmux/TmuxSessionLink.swift#L12-L100))。
- **SSH 根连接池**:每个 host+user 维持一条已认证连接,子通道复用,并发租约上限 4,闲置 120 秒关闭;用户点到工作区就预热认证;某会话通道级失败只「退休」根连接、不立即杀掉兄弟会话([RemuxSSHRootService.swift#L451-L560](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/SSH/RemuxSSHRootService.swift#L451-L560))。
  - 认证支持密码/私钥(ed25519、RSA、ECDSA 三曲线,带口令,存 iOS Keychain;可在设备上生成 ed25519 并一键安装到服务器)与 Tailscale SSH 的 `none` 方法+浏览器 check 验证([SSHAuthenticationMethodFactory.swift#L10-L90](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/SSH/SSHAuthenticationMethodFactory.swift#L10-L90))。
  - 主机密钥首次信任(TOFU)持久化于 trusted-hosts.json([TrustedHostStore.swift#L36-L53](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Persistence/TrustedHostStore.swift#L36-L53));无 agent 转发、无协议层保活(存活只在前台探测与写失败时暴露)。
- **前台恢复而非后台保活**:iOS 后台不养连接。
  - 回前台时探测控制通道是否仍活,死链归类断线原因并上报([TmuxScreenModel.swift#L263-L296](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Tmux/TmuxScreenModel.swift#L263-L296)、[TmuxTerminalSession.swift#L124-L134](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Tmux/TmuxTerminalSession.swift#L124-L134));根模型按来源(前台激活/传输丢失)做每来源一次的自动重连([RemuxRootModel.swift#L2064-L2085](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/App/RemuxRootModel.swift#L2064-L2085))。
  - 重连即整套新 model/controller/transport,携带上次「稳定视口尺寸」附着,避免以键盘缩小态首绘([TmuxScreenModel.swift#L28-L36](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Tmux/TmuxScreenModel.swift#L28-L36));「回到同一窗格」不靠客户端状态,靠 tmux 会话本身还在。
- **多客户端视口冲突**:与桌面 tmux 客户端并挂时,用 `refresh-client -C WxH` 上报自身网格、在用户交互时以 `select-window` 声明视口主导权,让 tmux 把窗口尺寸切回移动端([TmuxSessionController.swift#L753-L784](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Tmux/TmuxSessionController.swift#L753-L784)、[#L1273-L1309](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Tmux/TmuxSessionController.swift#L1273-L1309))。
- **移动输入完整性**:输入框文本先作被追踪的字面粘贴(内核发 `send-keys -l` 并等 tmux 对该条命令回执),确认落地后才单独补发回车;失败则文本留在远端提示符、明确「已粘贴未提交」,不会半截误执行([GhosttyComposerModel.swift#L330-L349](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Ghostty/GhosttyComposerModel.swift#L330-L349)、[追踪输入 TmuxSessionController.swift#L672-L740](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Tmux/TmuxSessionController.swift#L672-L740))。
- **编码代理的最小 UI**:快捷指令面板内置 Claude Code/Codex 起始集(/resume、/compact、/model 等,文本+回车序列,[StarterShortcuts.swift#L64-L125](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Domain/StarterShortcuts.swift#L64-L125)),可自定义分组;没有审批按钮、没有代理状态检测——代理仍是「终端里的一个程序」。IME 行内组字在终端视图里被有意置空(`UITextInput` 只为悬浮光标手势而实现,[shim #L1-L45](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/RemuxApp/Sources/Ghostty/GhosttyTerminalResponderTextInputShim.swift#L1-L45)),CJK 输入实际走输入框路径。

### 审计基线(钉定 commit/版本/许可 + changelog 与演化脉络)

- 钉定 commit:[`bc39fb6`](https://github.com/h3nock/remux/commit/bc39fb6d713f9679de43f92674cff03a633db0f1)(main HEAD,2026-08-24,「Add Tailscale SSH authentication with check verification (#60)」)。较 2026-08-29 运行服务复审钉定的同一 `bc39fb6` 无新提交,基线未变。
- 版本与发行:无 git tag、无 GitHub release、无 CHANGELOG;版本线索仅 `project.yml` 的 0.1.0 / build 11 与 TestFlight 公测链接;内核另行钉定二进制发行 `ghosttykit-20260815`(带 sha256 校验,[fetch_ghosttykit.sh#L8-L10](https://github.com/h3nock/remux/blob/bc39fb6d713f9679de43f92674cff03a633db0f1/scripts/fetch_ghosttykit.sh#L8-L10))。
- 许可:MIT 确认(LICENSE,版权人 h3nock,2026)。
- 演化脉络:2026-04-21 起步,2026-05-11 做过一次「Option C」整体重写(旧主线封存于 `legacy/remux-main-pre-option-c-2026-05-11` 分支,现 main 以「import remux v2 app」为根);此后主题依次为窗格画布与选择器、视口声明修复、外部贡献的既有会话加载与公钥安装、语音输入、Tailscale SSH、服务器/会话引导流程。活跃度:近 90 天约 493 提交、8 名贡献者(主力 1 人),最近提交 2026-08-24。

### 采用与边界(建议复用决策 + 不采用边界 + 对既有观察清单行的确认/修正)

- **复用决策:仅参考行为。** 代码与 tmux 控制模式、GhosttyKit 二进制内核、Citadel 分叉深度绑定,HCTL2 执行面已选 Herdr、不再引入 tmux,组件不可移植;值得抄的是移动客户端的「谈判—定位—渲染—重连」行为形状:附着命令一步带视口、以执行面原生 ID 为唯一路由身份(UUID 只做展示层)、每主机单认证连接多通道复用+点击预热、前台探活+按来源一次性自动重连+携带稳定视口、粘贴与提交两段式确认。快捷指令面板是「移动端操控编码代理」最小 UI 的实证。
- **不采用边界**:它没有任何治理概念——SSH 可达即可写,多客户端并发输入无仲裁、无单写者接管、无输入租约(TTL/代次/CAS),与 HCTL2「输入必须经租约、票据抵达执行现场」的要求相反;直连模型里也没有控制面票据的位置;无审批/状态检测 UI,不能当作代理审批交互的参考。
- **旧结论核对**:观察清单行**确认**——SSH+tmux 控制模式精确定位、MIT、不引入第二套领域状态(补充口径:运行时拓扑的单一来源是 tmux,客户端只持久化服务器/工作区书签与可逆 ID 映射)。2026-08-29 复审行**确认并补一处修正**:说它是「iOS Swift client」需加注——控制模式解析、命令生成与对账不在 Swift 层,而在其定制 GhosttyKit(Zig)内核中,Swift 只是壳与字节搬运;「不是 server」确认,唯一的远端写入是客户端发起的公钥安装与 SFTP 上传。

# macOS 钥匙串与 harness 凭据：三类凭据的整体方案调研

> 状态：调研 · 日期：2026-10-17<br>
> 类别：① Coding Harness · ⑥ 机械后端与基础设施（跨候选归纳） · 证据编号：E-L1-MACOS-KEYCHAIN<br>
> 对象：macOS 26.6.2（25G83，arm64）登录钥匙串与九家 harness 的凭据存储（版本见 §二表头）；本库 `foundation::SecretStore`（keyring 4.2.0）、`src/agency/src/confine.rs`（main `287391d5`）<br>
> 定位：Informative 研究备忘录。回答「mac 上三类凭据各放哪、边界怎么写成双平台一套」，给推荐但不定案，所有者拍板后另行落约束与代码；发布后正文不改，只在文末追加复核记录。证据分三档标注：**官方文档说**（当天打开过的官方页面）、**源码看到**（本库或本机帮助文本）、**实测**（2026-10-17 本机探针，环境见 §一）。本库既往记录单标**本库记录**。没查到的写「未查到」，不猜。

## 定位

所有者 2026-10-17 指出：mac 上的钥匙串问题不止一处，要整体解决、不逐个修（开工书 §二 第 37 条 ⑤、§七 第一条）。三处已知的撞墙是同一类问题的三个面：

1. **第 9 包第 5 条与 Claude Code 登录正面冲突。** 第 5 条要求 harness 环境里读钥匙串失败；Claude Code 在 macOS 上的登录缺省只在登录钥匙串里。`confine.rs` 的 macOS profile 拒读 `~/Library/Keychains` 后，Claude 在边界里报「Not logged in」（**本库记录**：开工书 §七，主笔 2026-10-11 实测）。演示 3 的 Claude Code 路径因此改在 ubuntu 走。
2. **control 作为后台进程读自己的令牌弹框、无屏即失败。** 新构建的 control 二进制对钥匙串是新程序，每次读 Gitea 管理员令牌都弹授权框；没有屏幕的会话报 `User interaction is not allowed`（**本库记录**：演示 1 验收缺口 5）。小活 D（#314）已给显式开关，但缺省仍是探测式优先钥匙串。
3. **harness 本人登录也写不进。** Muse 席自己登录时遇到过 token 写不进钥匙串（所有者 2026-10-17 转述）。本轮在本机现存 Muse 会话日志里没有找到带失败原因的原始记录（**实测**，见 §一末），只能定位到机制同型，不能坐实那一次的具体触发条件；如实写在这里，不编。

主笔提的方向（待查证、可推翻）：凭据按归属分三类——人自己的登录留在人的钥匙串，Agency 不碰；harness 在 Agency 里干活用的模型登录由 Agency 给每家托管一份私有目录，人经 Agency 登录一次，边界只放行这一份；控制面凭据留在 control 的密钥后端。凡是后台跑的东西都不依赖登录钥匙串。本文逐条查证后，结论是这个方向成立，且比逐个修便宜：受查的 harness 里只有 Claude Code 一家的 mac 缺省登录绕不开钥匙串，而它恰好有官方的单次启动 token 通道（§二）。

范围：只出调研与方案，不写产品代码；不打印、不复制任何凭据（探针只用自建的 dummy 条目，验完即删，见 §一）；不改任何一家 harness 的全局配置；不碰 `~/.local/state/hctl2`。

## 一、macOS 登录钥匙串：四种启动方式下能不能读写

先给裁决表。判据不是「进程在哪个终端里」，而是进程挂在哪个 **security context**（安全上下文）下、钥匙串当时锁没锁、读条目要不要弹框。

| 启动方式 | 读已解锁钥匙串里的自有条目 | 读需要授权的条目 / 钥匙串已锁 | 证据 |
| --- | --- | --- | --- |
| 图形 Terminal（GUI 登录会话内） | 能 | 能（弹框，人点了就过） | 实测 P2 |
| SSH 登录（同一台机、GUI 会话同时存在） | **不能**：rc 36 `User interaction is not allowed` | 不能 | 实测 P6 |
| launchd LaunchAgent（gui 域，Aqua 会话内） | 能 | 未测（弹框会打到图形会话） | 实测 P5 |
| launchd LaunchDaemon（system 域，root） | 不能（不在用户 security context 里，登录钥匙串不可见） | 不能 | 官方文档说（TN2083 表 2）；未实测（无 sudo，不动系统域） |
| 图形会话外起的终端复用服务（Herdr/tmux server 先于会话或跨会话存活） | 取决于 server 进程当时的 security context：挂在 GUI 会话下就能，不挂就不能 | 同左 | 实测 P2 与 P6 对照（同为 `Background` manager，结果相反） |

机制，四条：

- **security context 跟 bootstrap 命名空间走。** **官方文档说**（TN2083《Daemons and Agents》Security Context 节与表 2）：全局、per-user、per-session 三种 security context；GUI launchd agent 与 login item 在 per-session context，launchd daemon 缺省在全局 context。钥匙串服务（securityd）按 context 隔离：不在用户的登录会话 context 里，就没有「已解锁的登录钥匙串」可用。
- **登录钥匙串在 GUI 登录时解锁。** **实测**：本机 `security show-keychain-info` 报 login keychain `no-timeout`（不自动锁）。这意味着 GUI 会话活着时它一直开着——但这只对同一 security context 里的进程成立；SSH 会话里同一条命令直接 rc 36（实测 P6）。SSH 里有一条显式旁路：先 `security unlock-keychain` 输入钥匙串口令解锁、再读（man 页有此命令，CI 圈常用）；本轮**未实测**——不拿人的登录口令做实验。这条旁路救得了「人坐在 SSH 终端前」的场合，救不了无人值守的守护进程：口令本身又成了要保管的凭据。
- **要弹框而弹不出来，就是 `errSecInteractionNotAllowed`（-25308）。** 两类触发：钥匙串锁着、读条目需要先解锁；条目 ACL 不信任当前程序、需要人授权。**实测 P1**：把自建临时钥匙串锁上后读取，本席 shell 里命令挂住不返（等图形会话里的解锁框，10 秒未返回后终止）；SSH 会话里则不等、直接报不许交互。无屏后台进程两条路都是死路：要么挂死，要么报错。
- **新构建的二进制会被当成新程序重弹框。** 钥匙串条目的信任按程序的代码身份记；本地反复构建的 control 每次签名身份都变，于是每次读都弹「始终允许」也只管到下一次构建（**本库记录**：演示 1 验收「钥匙串」节）。本轮试图用 `security add-generic-password -T` 限制信任名单复现「换程序就弹框」，在 macOS 26.6.2 上另一个进程照样读到了 dummy 条目（**实测 P4，status=0**）——现代钥匙串的 partition 机制与旧 ACL 行为不同，这条复现失败，如实记下；重弹框现象本身以本库记录为准，成因解释标记为机制推论。

**终端复用为什么时灵时不灵。** **实测**对照：本席（Muse）自己的 shell `launchctl managername` 报 `Background`，dummy 条目增删查全过（P2）；SSH 会话同样报 `Background`，同样命令 rc 36（P6）。所以「manager 是谁」不是判据，判据是进程祖先链挂没挂进 GUI 登录的 audit/security 会话。Herdr、tmux 这类 server 若由图形 Terminal 拉起，子进程继承其 context、钥匙串可用；若 server 是开机自启、SSH 里起的、或 GUI 注销重登后还活着，子进程就在旧 context 里，钥匙串读写作废。这解释了「同一个 pane 昨天能跑今天不能跑」型故障：变的不是配置，是 server 的出生地。

**Muse 那次写不进，定位到哪一步。** **实测**：本机 Muse Code 1.4.10（1.4.10-R6427.1）会话日志的运行记录里带 `credential_backend` 字段，抽查 2026-10 的会话共 82 条：`keychain` 62 条、`file` 20 条，`keychain_fallback_reason` 全为 `none`；本席当前凭据是文件形态（`~/.config/muse/auth.json`，0600，今天刚更新过）。也就是说 Muse 自己就是双后端、且本席现在没在用钥匙串。现存日志里没有那次写失败的原因字段，**未能定位那一次的触发条件**；与 SSH 探针同型的解释（写入进程不在 GUI security context、或写入要弹框）只是推论。若要坐实，需要下次复现时留 `muse` 的原始报错，本文不替它编。

## 二、各家 harness：凭据放哪、能不能只对一次启动生效

先给总表。「单次启动」指 Agency 拉起这一次时用环境变量或重定位目录给凭据，不改这台机器上该 harness 的全局配置。本机版本均为 2026-10-17 `which` + `--version` 实测；落点一列是本机只看路径与权限、不看内容的探查（实测），与官方文档互相印证。

| Harness（本机版本） | macOS 缺省落点 | Linux 缺省落点 | 环境变量 token（优先于已存登录？） | 单次启动重定位 | 无头登录 |
| --- | --- | --- | --- | --- | --- |
| Claude Code 2.1.302 | 登录钥匙串（条目在，本机无 `.credentials.json`）；钥匙串拒写时官方说退回 `~/.claude/.credentials.json`（0600） | `~/.claude/.credentials.json`（0600） | `CLAUDE_CODE_OAUTH_TOKEN`（setup-token，一年期）、`ANTHROPIC_API_KEY`、`ANTHROPIC_AUTH_TOKEN`、`apiKeyHelper` 脚本；官方优先级表里都在已存登录之上 | `CLAUDE_CONFIG_DIR`：文件落该目录，钥匙串条目按目录分键（官方文档说） | `claude setup-token` 浏览器授权后打印 token、不落盘；`-p` 非交互直接用环境凭据 |
| Codex 0.163.0 | `$CODEX_HOME/auth.json`（本机 0600 实测在用文件）；或 OS 凭据库，二选一由配置决定 | 同左（`CODEX_HOME` 缺省 `~/.codex`） | `OPENAI_API_KEY` 经 `codex login --with-api-key`（stdin 读）落存；`CODEX_ACCESS_TOKEN` 经 `--with-access-token` 落存；纯环境直用未查到官方明示 | `CODEX_HOME` 整体重定位，含 auth 文件；`-c cli_auth_credentials_store=...` 单次改存储档（file/keyring/auto/ephemeral） | `codex login --device-auth`（官方 headless 首选）；ephemeral 档只存内存 |
| Muse 1.4.10 | 双后端：钥匙串或文件 `~/.config/muse/auth.json`；本席现在是文件（实测） | 未查到（本机只有 mac 一台，没在 ubuntu 复核） | `META_API_KEY`「always takes priority over the account login」（`muse login --help` 原文） | 后端选择开关未查到（帮助与本机配置里没找到）；`-c KEY=VALUE` 单次改设置在帮助里有，凭据键名未查到 | `muse login` 浏览器批 code；`muse auth set --api-key-stdin` 存 API key（stdin，不进 argv） |
| Antigravity（agy 1.4.0） | 文件 `~/.gemini/antigravity-cli/antigravity-oauth-token`（本机 0600 实测） | 未查到 | 未查到 | 未查到（帮助里只有 `--agent`、`--conversation` 等会话参数） | 未查到 |
| Grok Build 1.0.62 | 文件 `~/.grok/auth.json`（本机实测） | 未查到（同为 `~/.grok` 的可能性大，但不写成事实） | 未查到官方环境变量口径 | grok home 固定 `~/.grok`（帮助原文）；重定位环境变量未查到 | `grok login --device-auth`「for headless/remote environments」（帮助原文） |
| opencode 1.19.3（GLM 席） | 文件 `~/.local/share/opencode/auth.json`（官方文档说 + 本机实测） | 同左（XDG data） | provider 配置可用 `{env:VAR}` 引用环境变量（本库 [harness-hooks 调研](./harness-hooks-20260903.md) 已钉）；标准 provider API key 环境变量直读以官方 providers 页为准，未逐家核 | `OPENCODE_CONFIG_DIR` / `OPENCODE_CONFIG_CONTENT` 只重定位配置不管 auth 文件；auth 跟 XDG Data 走，改 `XDG_DATA_HOME` 才连 auth 一起挪（由路径构成推得，未实测） | `opencode auth login` 交互登录；纯环境给 key 可免登录（边界同上） |
| omp 18.11.3（DeepSeek 席） | 自有 auth-broker 凭据库（`omp auth-broker` 子命令，本机帮助实测）；落点文件未查到 | 未查到 | `--api-key` 参数「defaults to env vars」（帮助原文）；具体变量名按 provider，未逐个核 | `--profile=<name>`「isolated profile for auth, sessions, settings, and caches」（帮助原文）——受查各家里最直接的单次隔离 | `omp auth` 可导出/导入凭据 JSON；`auth-broker login --via user@host` 支持经 SSH 远程登录 |
| Qoder（qoderclicn 1.1.73） | 本机配置根 `~/.qoder-cn/` 存在；凭据文件具体落点未查到（没翻内容目录定性） | 未查到 | 未查到 | `--config-dir <dir>`「custom user-level config root for this run」（帮助原文） | `qoderclicn login` 交互登录；无头方式未查到 |
| Kimi Code 2.2.0 | `~/.kimi-code/credentials/` 目录存在（本机实测，0700） | 未查到 | 未查到 | `KIMI_CODE_HOME` 整体重定位（本库 harness-hooks 调研已钉） | `kimi login` 设备码流（帮助原文） |
| MiniMax（mcode 0.6.12） | 未查到（`mcode provider` 管理 API key，落点没核） | 未查到 | 未查到 | 未查到 | `mcode login` 交互登录 |
| gh 2.102.0 | 系统凭据库（本机 `gh auth status` 自报 `(keyring)`）；`--insecure-storage` 时退明文 `hosts.yml` | Secret Service 或明文回退（同机制） | `GH_TOKEN` > `GITHUB_TOKEN`、「takes precedence over previously stored credentials」（`gh help environment` 原文）；企业版 `GH_ENTERPRISE_TOKEN` | `GH_CONFIG_DIR` 重定位配置根；纯给 `GH_TOKEN` 时不读写任何已存凭据 | `gh auth login --with-token`（stdin）；CI 官方口径就是 `GH_TOKEN` 环境变量 |

逐家要点，只写与方案有关的：

### Claude Code（唯一绕不开钥匙串的一家，有官方旁路）

- **官方文档说**（Authentication 页《Credential management》）：macOS 凭据存加密的 macOS Keychain；「When the Keychain rejects the write, such as when it's locked in an SSH session」，退回 `~/.claude/.credentials.json`（0600）；`CLAUDE_CONFIG_DIR` 同时决定回退文件位置、且钥匙串条目按该目录分键。这修正了开工书「只在钥匙串里」的旧口径：现在有官方回退档，但回退发生在写入被拒时，且 Console 建 API key 的登录在钥匙串不可写时直接失败（同页原文）。
- **官方文档说**（同页《Authentication precedence》《Generate a long-lived token》）：`claude setup-token` 生成一年期 OAuth token，打印到终端、不保存到任何地方，由人设为 `CLAUDE_CODE_OAUTH_TOKEN`；优先级表里环境凭据（`ANTHROPIC_AUTH_TOKEN`、`ANTHROPIC_API_KEY`、`apiKeyHelper`、`CLAUDE_CODE_OAUTH_TOKEN`）全部排在已存登录（第 7 位）之上。这正是给 Agency 的单次启动通道：Agency 只需在拉起时注入这一个环境变量，harness 全程不需要碰钥匙串。
- **实测**：本机钥匙串里 `Claude Code-credentials` 条目在（只查存在、未取值），`~/.claude/.credentials.json` 不存在。§四的探针证明拒读 Keychains 目录后该条目查不到（-25300），harness 表现为未登录，与本库记录对上。

### Codex（存储档可钉，是九家里口径最清楚的）

- **官方文档说**（Authentication 页《Credential storage》）：`cli_auth_credentials_store` 四档——`file` 存 `CODEX_HOME/auth.json`；`keyring` 存 OS 凭据库、不可用就失败；`auto` 有凭据库用凭据库、否则退文件；`ephemeral` 只存内存。缺省值本轮在官方页上未查到明示；**实测**本机 `~/.codex/auth.json` 存在（0600），即本席现在是文件档在用。
- 对 Agency 的含义：给每次拉起钉 `CODEX_HOME` 到托管目录、配置钉 `file` 档，凭据就完全在托管目录里，双平台行为一致；不需要也不应该让它用 `keyring`/`auto`（auto 在 mac 上又会走进登录钥匙串）。

### 其余各家

- **文件派是多数。** opencode、Grok、Antigravity、Kimi 本机都是 home 下的 0600 文件或 0700 目录，没有一家缺省往 mac 钥匙串里写（实测落点见总表）。这类 harness 的托管就是目录重定位问题，与钥匙串无关；真正的缺口是各家重定位旋钮不齐（Grok、Antigravity 未查到），缺旋钮时只能靠改 `HOME` 或 XDG 根——`confine.rs` 的 `scrub` 已经在用改 `HOME` 的办法（源码看到），托管目录方案与它同构。
- **omp 的 `--profile` 是现成的单次隔离**，auth-broker 还支持凭据导入导出与经 SSH 登录（帮助原文）。若 Agency 要给每家托管一份，omp 这一家几乎零适配。
- **Muse 的后端开关是本轮最大的未查到。** 日志证明双后端存在，但帮助与本机配置里没找到「钉死用文件、别碰钥匙串」的公开旋钮。在旋钮查清之前，Agency 托管 Muse 的稳妥通道是 `META_API_KEY` 环境注入（官方帮助明示优先级最高）。
- **gh 是业界先例，且本库已经照它定过一次。** 存储「securely in the system credential store」、`--insecure-storage` 才写明文、环境 token 压过已存凭据（官方手册与本机帮助一致）。所有者 2026-09-04 裁 control 「照 gh CLI 的做法」（[keyring 调研复核记录](./libs/keyring.md)）——但要注意 gh 是人手交互工具，它的缺省对后台进程不成立，见 §三。

## 三、control 密钥后端：后台进程用钥匙串缺省站不站得住

**源码看到**的现状（main `287391d5`）：

- `foundation::SecretStore`（`src/crates/foundation/src/lib.rs`）双后端：`SystemKeyring` 走 keyring 4.2.0 的 `Entry::set_secret/get_secret`（服务名 `hctl2`）；`UserFile` 写 `<root>/secrets/`，目录 0700、文件 0600、临时文件加 `sync_all` 后改名，文件名是 `service\0account` 的 SHA-256。
- 选择逻辑 `SecretStore::select`：显式配置优先；没配置时 `detect`——`keyring::Entry::store_status().is_ok()` 就选钥匙串。显式选 `system-keyring` 而不可用时**启动失败**，不静默改道（源码注释与 control README 一致，是有意设计）。
- 配置键 `secret_backend` 在 `<root>/control.json`，`hctl2 init|start --secret-backend` 写（小活 D，#314）；`status`/`doctor` 报告实际后端。原生测试已显式钉 `user-file`，不再碰开发机钥匙串。
- control 今天不是 launchd 服务：`hctl2 start` 在用户终端里 spawn 一个脱离标准输入输出的子进程（`src/apps/cli/src/main.rs::start_daemon`，stdin/stdout/stderr 全 null）。它继承拉起者的 security context——从图形 Terminal 起就活在 GUI 会话里，从 SSH 或开机脚本里起就不在。

会不会撞上同样的问题：**会，而且已经撞过**（演示 1 缺口 5）。机制上三条都对 control 成立：新构建二进制重弹框（本地构建签名身份多变）；`start` 的出生地决定 context（SSH 里 `hctl2 start` 后，守护进程每次读 Gitea 令牌都会 rc 36 型失败，**由实测 P6 推得，未拿真 control 复测**——本轮不碰 `~/.local/state/hctl2`，如实标注）；GUI 注销重登后旧守护进程留在旧 context 里。更隐蔽的一条：`detect` 只问 `store_status`，钥匙串「库在但条目读要弹框」时它仍会选中钥匙串，把失败留到运行时——**store_status 在 SSH context 下的实际返回值本轮未实测**（要写 Rust 探针调 keyring crate，超出「不写产品代码」的边界，列为未决）。

小活 D 的缺省要不要改，这是所有者的决定，本文只摆选法：

| 选法 | 内容 | 利 | 弊 |
| --- | --- | --- | --- |
| 甲（现状） | 缺省探测：有钥匙串用钥匙串 | 与 2026-09-04「照 gh 做法」的裁定一致；桌面交互场景下凭据受钥匙串保护 | 后台/无屏/重构三类失败都在；行为随出生地变，最难排查；gh 是交互工具，这个类比对守护进程先天不成立 |
| 乙 | 缺省改 `user-file`，钥匙串降为显式选项 | 双平台、双 context 行为一致；后台进程零弹框；测试与生产同路 | 推翻 2026-09-04 裁定，需所有者重新拍板；0600 文件的保护边界退到同 OS 用户（与约束层「未启用加固时同信任域」一致，但口径要明说） |
| 丙 | 缺省探测加一道真读探针：启动时实际读一次自有条目，要交互就判定不可用并落文件（或报错让人选） | 保留桌面收益，失败从运行时提前到启动时 | 启动时真读可能弹一次框；「读得到」与「之后一直读得到」仍不是一回事（重登换 context）；实现与口径都更绕 |

本文在 §五给推荐时选乙，理由写在那边；甲不是不能用，是要把「control 必须从图形会话里起、且不换二进制」写成明示的运维约束，而不是让它以随机失败的形式出现。

## 四、边界机制：能不能只放行某一家自己那一项

开工书 §七留过候选一问：Seatbelt 能不能只放行 Claude 自己那一项。结论：**靠 sandbox profile 本身做不到；靠换存储落点做得到，而且不必动 profile 的粒度。**

- **源码看到**：`confine.rs` 的 macOS profile 是 `(version 1)(allow default)` 加逐条 `(deny file-read* (subpath ...))`，`sensitive_paths()` 含 `~/Library/Keychains`、`/Library/Keychains`、`~/.config/gh`、`~/.ssh`、`~/.local/share/keyrings` 等；经 `/usr/bin/sandbox-exec` 执行（本机 man 页首行即标 DEPRECATED，**实测**）。规则语言只有文件、Mach 服务、进程等系统对象，没有「钥匙串条目」这个粒度——条目是 securityd 内部的记录，profile 里写不出「只许读 service=X」。
- **实测 P3**：同样的 profile 下，`security find-generic-password` 读自有 dummy 条目报找不到（rc 44）；Swift `SecItemCopyMatching` 返回 **-25300（errSecItemNotFound）**；无 sandbox 对照均为成功。也就是说整目录拒读对新旧两条钥匙串 API 都生效，表现为「条目不存在」而不是「无权限」——harness 只会以为没登录，不会报错说被拦。这与本库记录里 Claude 报「Not logged in」严丝合缝。
- 反过来，**专属钥匙串文件**是 mac 的原生机制：`security create-keychain/list-keychains/unlock-keychain`（本机 man 页，实测 P1 全走通），本机 search list 里已有第二把 `openvpn.keychain-db` 与 login 并列（实测）。给 Agency 建一把专属钥匙串（须放在托管目录而不是 `~/Library/Keychains` 里，否则被整目录 deny 一起误伤）、把托管凭据放那里、profile 只拒 login 那把——技术上成立；但三个代价：其一，search list 与 default-keychain 是**每用户全局**设置，改它会影响人自己的其他程序，与「只对一次启动生效」相悖；其二，各家 harness 是否认非 login 钥匙串里的条目逐家不同（Claude 官方只说按 `CLAUDE_CONFIG_DIR` 分键，没说能换钥匙串文件——未查到）；其三，它仍在钥匙串体系里打转，锁、弹框、context 三病一样不落。所以这一路只列为候选，不推荐（§五）。
- Linux 侧同构面：**源码看到** `confine.rs` 用 Landlock allow-list（整目录放行、挖不掉子目录，所以凭据根被 allow 目录覆盖时直接拒绝启动）。Linux 上 Claude 凭据本来就是 `~/.claude` 下的文件（官方文档说），托管目录在两平台是同一种东西：一个被放行、其余全拒的目录。边界规则要写成一套，抽象点应该是「目录」，不是「钥匙串条目」——条目粒度 mac 独有、Landlock 无对应物，写成一套只会一真一假。

## 五、方案：三类凭据各放哪

### 三类各自的落点（推荐案，先讲推荐再摆候选）

**第一类，人自己的登录：留在人的钥匙串与人的 home，Agency 完全不碰。** 人在终端里跑的 Claude、gh、git 继续用他们现在的登录；Agency 的 profile 继续拒读 `~/Library/Keychains`、`~/.config/gh`、`~/.ssh`（现状 `sensitive_paths()` 就是这张单）。这一类不需要任何新机制，需要的只是口径：第 9 包第 5 条说的「读钥匙串失败」应重述为「读**人的**登录钥匙串与人的凭据目录失败」，否则它会与第二类托管永远打架（改不改条文由所有者定，见文末待定清单）。

**第二类，Agency 里干活用的模型登录：Agency 给每家托管一份私有目录，边界只放行这一份。** 形态按各家旋钮分三档，优先级从上往下：

1. **纯环境注入，连文件都不落**（Claude 的 `CLAUDE_CODE_OAUTH_TOKEN`/`ANTHROPIC_API_KEY`、Muse 的 `META_API_KEY`、gh 场景的 `GH_TOKEN`、Codex 的 ephemeral 档）。凭据只活在这一次拉起的环境里，与钥匙串、文件全无关。注意环境变量对同用户进程可经 `/proc`（Linux）或子进程继承扩散——Agency 现有 `scrub` 已做 `env_clear` 白名单（源码看到），注入时要保证只给目标进程、不进 Herdr server 的常驻环境。
2. **重定位到托管目录**（Codex 的 `CODEX_HOME`+`file` 档、Claude 的 `CLAUDE_CONFIG_DIR` 文件回退、omp 的 `--profile`、Qoder 的 `--config-dir`、Kimi 的 `KIMI_CODE_HOME`、opencode 改 XDG 根）。托管目录是新建的空目录，人经 Agency 在里面登录一次（下条），之后 harness 自己的刷新、过期管理都在里面自转。
3. **改 `HOME` 兜底**（Grok、Antigravity 这类没查到重定位旋钮的）。托管目录充当假 home，harness 以为在自己家登录，实际全在托管目录里。代价是该 harness 的全部用户级状态（不止凭据）都跟人日常用的那份分开，这正是要的效果；风险是个别 harness 对 `HOME` 之外还有硬编码路径，接入时逐家验。

**登录一次怎么做。** Agency 提供一条引导命令（形状 for agent、本文不定名）：为指定 harness 建托管目录（0700），以该目录为凭据根拉起该 harness 自己的登录流（浏览器或设备码，人完成授权），登录产物只落托管目录或只回一个 token 由 Agency 收进托管存储；最后用无头方式验一次能调通模型。Claude 这家最顺：人跑 `claude setup-token`，把打印的 token 交给 Agency 收存，Agency 以后每次注入——token 一年期、官方为 CI/脚本设计。登录全程不需要、也不应该读人的钥匙串；托管凭据与人的登录是**两个独立登录**，人注销自己的不影响 Agency，反之亦然。

**第三类，control 凭据：留在 control 自己的密钥后端，缺省改 `user-file`（§三选法乙）。** 理由：control 是守护进程，出生地不可控（图形 Terminal、SSH、开机脚本、测试），而钥匙串的全部失败模式都来自出生地与弹框；`user-file` 已有实现、已被裁为获准后端、测试已全走它，改缺省不加新依赖、不加新机制，只是把显式选项升为缺省。钥匙串保留为显式档给想要的人，且保留现在「显式选了不可用就启动失败、不静默改道」的诚实行为。若所有者不改裁定（选法甲），最低限度把运维约束写明：control 必须从图形会话内启动、二进制更新后首次读令牌需人在场点允许——写进 `docs/usage.md`，而不是靠踩坑传递。

### 边界规则：双平台一套

一套规则，两种实现，逐行对应：

| 规则（平台无关表述） | macOS（Seatbelt profile） | Linux（Landlock） |
| --- | --- | --- |
| 人的凭据区一律拒读：登录钥匙串、Secret Service 库、ssh、gh 配置、git 凭据文件 | `(deny file-read* (subpath ...))` 逐条（现状） | allow-list 不含这些目录（现状） |
| 本次派工的托管凭据目录单独放行 | 不写 deny（`(allow default)` 下天然可读，现状即如此） | 加进 `HCTL2_CONFINE_ALLOW`（现状有此通道，harness_command 已在用） |
| control 凭据根拒读拒写 | `(deny file-read*/file-write* (subpath cred))`（现状） | 同上，不进 allow-list；被 allow 目录覆盖时拒启动（现状） |
| 环境注入型凭据只给目标进程 | `env_clear` 后按白名单逐个给（现状 `scrub`/`harness_command`） | 同左（同一段代码） |
| 施加不了就不启动并报原因 | profile 写不出、sandbox-exec 失败即返回错误（现状） | helper 缺失、凭据根被覆盖即返回错误（现状） |

要点：推荐案不需要发明新边界机制。mac 上「拒整目录」之所以误伤 Claude，是因为 Claude 的干活凭据错放在人的钥匙串里；第二类托管把干活凭据搬进托管目录之后，现有 deny 规则从误伤变成正好。sandbox-exec 已被 Apple 标 deprecated 是存量风险（本机 man 页），但那是执行加固整体的选型问题，不由本方案单独解决，列为未决。

### 候选对照与推荐

| 候选 | 做法 | 利 | 弊 |
| --- | --- | --- | --- |
| A. 托管目录 + 环境注入（上文推荐案） | 三类分开，第二类按环境/重定位/假 HOME 三档托管 | 双平台同构；后台零钥匙串依赖；复用现有 confine 通道，改动集中在引导登录与拉起参数 | 人要在 Agency 里多登一次；没旋钮的 harness 要逐家验假 HOME |
| B. Agency 专属钥匙串文件 | 建第二把钥匙串放托管凭据，profile 放行它、拒 login | 凭据仍受钥匙串加密保护 | mac 独有、Linux 无对应物，一套规则写不成；search list 是全局设置；锁与弹框病根仍在；各家认不认非 login 库未查到 |
| C. 只给 Claude 开例外 | profile 对 Claude 放行 Keychains，其余照旧 | 改动最小，演示 3 不用绕 ubuntu | harness 能读人的**全部**钥匙串条目（粒度到不了单项，§四），第 5 条对 Claude 形同虚设；逐家开例外正是所有者否掉的逐个修 |
| D. 改第 5 条口径了事 | 条文改成「读 control 凭据根与 gh auth 失败即可」，钥匙串不拦 | 零代码 | 把已知的凭据暴露写成合规；control 与 harness 本人登录两处病灶原样保留 |

**推荐 A，control 缺省配套选 §三乙。** 这与主笔方向一致，查证后没有发现需要推翻它的事实；唯一的修正是 Claude 这家优先用环境 token 而不是文件托管，因为官方 setup-token 本来就是为这种场景设计的。

### 待所有者定的清单

1. 三类划分与推荐案 A 是否成立。
2. control 缺省是否改 `user-file`（推翻 2026-09-04 裁定），还是保留甲并明示运维约束。
3. 第 9 包第 5 条是否按「人的钥匙串/人的凭据目录」重述；重述后演示 3 的 Claude 路径可否回 mac（走托管 token）。
4. 托管凭据的失效与轮换口径：托管 token 过期时 Agency 怎么报、谁来重登（本文只定「登录一次」的入口，续期未设计）。
5. sandbox-exec deprecated 的长期出路排不排期（与本文正交）。

未决（本轮没查清、别当事实用）：`keyring` 的 `store_status` 在 SSH context 下的返回值；Muse 后端选择有无公开开关及其钥匙串条目名；Antigravity、Grok、Qoder、MiniMax 的环境 token 与重定位官方口径；Claude 文件回退档在 confine 下的实际触发（官方说拒写即回退，本轮未拿真 Claude 在边界内复测）；LaunchDaemon context 下读登录钥匙串（无 sudo 未实测）。

## 证据

官方文档（2026-10-17 打开）：

- Apple [TN2083《Daemons and Agents》](https://developer.apple.com/library/archive/technotes/tn2083/_index.html)（Security Context 节、Execution Context Summary 表 2）；本机 `man security`、`man sandbox-exec`（macOS 26.6.2 自带，sandbox-exec 标 DEPRECATED）。
- Claude Code [Authentication](https://code.claude.com/docs/en/authentication)（《Credential management》《Authentication precedence》《Generate a long-lived token》）。
- Codex [Authentication](https://learn.chatgpt.com/docs/auth)（《Credential storage》`cli_auth_credentials_store` 四档、《Login on headless devices》）。
- OpenCode [Providers](https://opencode.ai/docs/providers/)（《Credentials》：`~/.local/share/opencode/auth.json`）。
- gh [gh_auth_login 手册](https://cli.github.com/manual/gh_auth_login) 与本机 `gh help environment`（`GH_TOKEN` 优先级原文）；[systemd Credentials](https://systemd.io/CREDENTIALS/)（`LoadCredential`/`LoadCredentialEncrypted`，Linux 服务凭据的原生机制，与 [keyring 调研](./libs/keyring.md) 的 headless 结论互证）。

本库：

- [libs/keyring.md](./libs/keyring.md)（keyring 4.2.0 选型、2026-09-04「照 gh 做法」裁定、2026-10-03 小活 D 复核记录）；[harness-hooks-20260903.md](./harness-hooks-20260903.md)（各家重定位旋钮的先行钉版）。
- 源码（main `287391d5`）：`src/crates/foundation/src/lib.rs::SecretStore::{detect,select,user_file}`、`src/apps/control/src/config.rs::secret_backend/secret_store`、`src/apps/cli/src/main.rs::start_daemon` 与 `--secret-backend`、`src/agency/src/confine.rs::{macos,linux,scrub,harness_command,sensitive_paths}`、`src/apps/control/README.md`《密钥后端》、`docs/usage.md:344`。
- 记录：`.memo/design/p2-control-20260906/07-demo-kickoff.md` §二 第 37 条 ⑤、§三缺口 5、§四 第 9 包第 5 条、§六小活 D、§七第一条；`.memo/log/2026-10-03-演示1验收.md`《钥匙串》节。

本机实测（2026-10-17，macOS 26.6.2 arm64；探针只用自建 dummy 服务名 `hctl2-research-probe-20261017` 与临时钥匙串，全部验完删除，未打印、未复制任何真实凭据）：

- P1 临时钥匙串：create/unlock/add/find 全过；lock 后 find 挂住等解锁框（10 秒未返回，终止后清理）。
- P2 图形会话内（本席 shell，manager=Background 但挂在 GUI 会话下）：login 钥匙串 dummy 增、查（值比对一致）、删、删后查 rc 44，全过。
- P3 Seatbelt `(allow default)`+拒读 `~/Library/Keychains`：`security find` rc 44、Swift SecItem 返回 -25300；无 sandbox 对照成功。
- P4 `-T /usr/bin/true` 限制信任后，Swift 进程仍读到 dummy（status=0）——限制未生效，如实记录。
- P5 LaunchAgent（gui 域 bootstrap 一次性 plist，验完 bootout）：manager=Aqua，find rc 0。
- P6 SSH（127.0.0.1，BatchMode）：manager=Background，find rc 36，stderr `User interaction is not allowed`。
- 版本与落点盘点：九家 `--version` 与凭据路径 stat（只看存在与权限）；Muse 会话日志 `credential_backend` 计数（keychain 62/file 20，fallback_reason 全 none）。

## 复核记录

- 2026-10-17 初版（Muse 席 mac）：按开工书 §二 第 37 条 ⑤ 的派工落盘；评审席 Claude（mac）找问题、Antigravity（ubuntu）核验链接与版本，结论待回填。



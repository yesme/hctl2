# 测试残留进程与目录怎么收

> 状态：讨论中 · 待拍板
> 基线：main @ `f4fd5bf7`（草案 v0.19.3）
> 去向：拍板后拆两件——测试夹具与 CI 收尾落 `src/agency`、`src/apps/*/tests`、`src/build/ci`；产品层按选定方案落 `src/agency`；新脚本的调研见 [docs/research/build-tools/test-leftover-reaper.md](../../docs/research/build-tools/test-leftover-reaper.md)

## 一、问题与现场

测试跑完常留下常驻进程与临时目录，随跑次累积。所有者 2026-10-18 在 ubuntu 清掉 52 个父进程已是 1 的 `herdr server`；mac（**留作现场，未清**）2026-10-18 23:00 的实测：

| 现场 | 值 |
| --- | --- |
| 孤儿 `herdr server`（`ppid=1`，7 天以上，安装目录已删） | 4 个：`hctl-codex-live-cli-50085`、`hctl-demo3-gitea-claude-cli-64447`、`hctl-demo3-gitea-claude-cli-71365`、`hctl2-claude/buck-out…/packaged herdr` |
| 上述下面挂的会话 | `codex app-server`（`ppid=1`，socket 在 `/tmp/hctl2-herdr-0ea801c8…/`）、`codex resume … --remote unix://…`、`claude --restricted …`（写会话，state 在 `/private/tmp/hctl2-exec-a14262…/hctl2-herdr-state-…/write-…/`） |
| 另一个孤儿 | 1 个 `codex app-server`（`ppid=1`） |
| `/tmp` 残留目录 | 216 个 `hctl2-herdr-*`、28 个 `hctl2-exec-*`，另有 `hctl2-write-*`／`hctl2-process-*`／`hctl2-keychain-*`／`hctl2-codex-*` 各 1–2 个 |
| 所有者自己在用的（**不得触碰**） | `~/.local/bin/herdr` + `herdr server`（7×24 常驻）、`buck2d[…]`（各席编译农场）、`bazel-remote-bin`、`nativelink`、`~/.local/state/hctl2`、`~/.config/herdr` |

## 二、盘点：谁起常驻进程、四种结局

代号：**正常**＝用例通过正常返回；**panic**＝用例 panic（Rust 默认 unwind，`Drop` 照跑）；**KILL9**＝测试进程被 `kill -9`；**超时**＝Buck2 超时杀测试。`Drop` 是否执行：正常/panic 是，KILL9/超时否（测试进程内的 `Drop` 全被跳过）。

| # | 谁起的 | 起法（代码） | 正常 | panic | KILL9 | 超时 | 目录 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `agency serve` → `herdr server` | `src/agency/src/herdr.rs:176`（先 `reap_previous`）→ `:239` `.process_group(0)`；收尾 `impl Drop for Server` `:283`（`signal_group(KILL)`）、失败路径 `StopChild` `:544` | 收 ✓ | 收 ✓ | **留** ✗ | **留** ✗ | state 在 `exec_parent` 旁 `hctl2-herdr-state-<digest>`（`:307`，digest 取凭据根）；测试 state 目录 `hctl2-herdr-<…>`（`:411`） |
| 2 | `agency serve` → `codex app-server` | `src/agency/src/codex.rs:554` `.process_group(0)`；`Live::close` `:679` TERM→`:687` KILL；`impl Drop for Live` `:694` | 收 ✓ | 收 ✓ | **留** ✗ | **留** ✗ | socket/pid 在 `dir`（Agency 私有目录）下；**没有 `reap_previous` 对应物** ✗ |
| 3 | herdr 里的 Claude/Codex 会话（pane） | 由 herdr 起，Agency 只持 pane；`OpenPane::drop` 关 pane（`herdr.rs:514`） | 收 ✓（herdr 组被杀时连带） | 收 ✓ | **留** ✗（孤儿 herdr 下） | **留** ✗ | 会话 state 在 `hctl2-herdr-state-…/write-…`／`standby-…` 下 |
| 4 | Agency `start` → `serve` | `src/agency/src/main.rs:102-116` `process_group(0)`，`start` 等就绪后**自身退出** | serve 常驻（按设计） | 同左 | serve 留 ✗ | serve 留 ✗ | `--root` 下（测试用临时 root） |
| 5 | 测试夹具直接起 `agency serve` | `apps/cli/tests/room/project.rs:423`（`AgencyChild`，`:35`）；Drop `:36-41` **只 `kill()` 不 `agency stop`** ✗；对照组 `room/agency.rs:73`、`room/github.rs:238` 先 `agency stop` ✓ | **留** ✗（Drop 是 SIGKILL，Agency 来不及收 herdr/app-server） | **留** ✗ | 留 ✗ | 留 ✗ | 同 #1/#2/#3 |
| 6 | 测试夹具起 control（`hctl2 start`） | `apps/cli/tests/room.rs:32` Drop：`hctl2 stop` ✓ + `hctl2-services stop` ✓ + 删 root ✓；`apps/cli/tests/cli.rs:37`、`cli/human.rs:28` 只按 `control.pid` 发 TERM ✗（未 stop services） | 收 ✓ | 收 ✓ | 留 ✗ | 留 ✗ | 测试临时 root（`Drop` 删；KILL9 留） |
| 7 | `hctl2-services`（process-compose `up --detached`：Tuwunel、Gitea） | control 侧 `apps/control/tests/services.rs` 与 `room.rs` 夹具显式 `stop`；设计上不随 control 退出 | 收 ✓ | 收 ✓ | 留 ✗ | 留 ✗ | `HCTL2_STATE_ROOT/services` |
| 8 | 测试起 herdr（`agency/tests/herdr.rs`） | `Server::start_with_denials`；`scratch()` `:39-49` 建 `hctl2-herdr-cred-<name>-<pid>`／`-exec-…` **无 Drop 删除** ✗ | 进程收 ✓、**目录留** ✗ | 同左 | 进程留 ✗、目录留 ✗ | 同左 | 上述目录 + `hctl2-herdr-state-<digest>` |
| 9 | `agency/tests/runtime.rs` | `:24`／`:104` 建 `hctl2-exec-probe/slash-*`，各测试内 `remove_dir_all` ✓ | 收 ✓ | 收 ✓ | 留 ✗ | 留 ✗ | 同名目录 |
| 10 | `agency launch` 的 smoke 目录 | `src/agency/src/launch.rs:718` `.prefix("hctl2-herdr-smoke-")` | 由 `TempDir` 收（待核实） | 同上 | 留 ✗ | 留 ✗ | `/tmp/hctl2-herdr-smoke-*` |

要点：**进程泄漏只在 KILL9/超时发生**（正常与 panic 都有 `Drop`），**目录泄漏连正常结束都发生**（#8 的 `scratch` 从不删，`hctl2-exec` 也无人收）；唯一跨次兜底是 `herdr` 的 `reap_previous`（`herdr.rs:176`，只在**同一 state 目录**上生效——测试每次用新凭据根，digest 变了，永不命中 ✗），`codex app-server` 连这个兜底都没有 ✗。

## 三、实际复现（macOS arm64，本机）

**R1 · 正常结束也留目录。** 跑 `root//agency:herdr_test`（52 passed / 7 ignored）：

```
before: herdr-dirs=216 exec-dirs=28
test result: ok. 52 passed; 0 failed; 7 ignored
after:  herdr-dirs=302 exec-dirs=30
```

进程：前后都只有现场那 4 个孤儿，**没有新增**（`Server::Drop` 生效）；目录：+86 个 `hctl2-herdr-*`、+2 个 `hctl2-exec-*`。复现产物按「2–30 分钟」窗口删了 58 个；余下约 29 个无法精确归因（见 §五 的 run-id 归属），未再删。

**R2 · kill -9。** 跑同一目标、轮询到测试进程后 `kill -9 65519 65549`：现场孤儿数不变（该套每个 herdr 存活不到 1 秒，抓不到窗口），但机制由现场与代码路径确定——`Drop` 被跳过，`herdr`/`app-server`/会话三者都在各自的进程组里，父进程一死就成 `ppid=1`，正是现场 4 个孤儿的来历（命令里还能看到它们出自被中断的 `hctl-codex-live-cli-*`、`hctl-demo3-gitea-claude-cli-*` 测试根）。

**panic。** Rust 默认 unwind，`Drop` 照跑，故与正常结束同；只有 `panic=abort` 或 `std::process::abort` 才等价于 KILL9——本仓测试进程用默认 unwind（未另行核实 `panic=abort` 是否被开启，标待核实）。

**Buck2 超时。** 未实测（`待核实`）：Buck2 以「测试运行器进程」协作（[Test Execution](https://buck2.build/docs/rule_authors/test_execution)），超时如何杀（进程还是进程组）需在 CI 上做一次带超时的实验；从现场看，被中断的测试留下了孤儿，说明**至少不保证**清掉测试派生的独立进程组。

## 四、业界做法（先查后提）

| 做法 | 要点 | 对本案的取舍 |
| --- | --- | --- |
| cargo-nextest | 每个测试建**独立进程组**；超时先 TERM 组、宽限后 KILL 组（[slow-tests](https://nexte.st/docs/features/slow-tests/)）。**Leaky tests**：测试退出后若子进程仍持有继承来的 stdout/stderr，标 `LEAK`，可配 `leak-timeout = {period, result = "fail"}` 变红（[leaky-tests](https://nexte.st/docs/features/leaky-tests)） | 思路可借：**退出后按「还活着」判漏**。但它明确**抓不到** stdio 置 null 的子进程——我们的 `herdr`/`app-server` 正是 null ✗，所以不能指望同款检测，必须自己按 pid/进程组/目录归属判 |
| testcontainers Ryuk | 旁路 reaper：资源打 session 标签，reaper 观察测试进程的生命线，连接一断就清（[moby-ryuk](https://github.com/testcontainers/moby-ryuk)） | 借「**带 session 标签的旁路 reaper**」：测试侧给出 run-id，收尾器只清本次 run 的东西 |
| Linux `PR_SET_PDEATHSIG` | 子进程设置父死信号，父一死内核发信号（[man](https://manpages.debian.org/unstable/manpages-dev/PR_SET_PDEATHSIG.2const.en.html)）；`PR_SET_CHILD_SUBREAPER` 让中间进程收孤儿 | 只对「自己 spawn 的子进程」有效，且必须由**子进程**自己设置——`herdr` 是外部二进制，改不了 ✗；可作为 Linux 上 Agency 起**自家进程**的补充 |
| macOS 等价物 | 无 PDEATHSIG；`kqueue` `EVFILT_PROC` + `NOTE_EXIT` 监视父进程（[kqueue(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kqueue.2.html)）；或用**生命线管道**（父死→EOF→子自退） | 对 `herdr` 同样改不了子端 ✗；可行的是 Agency 起一个**死亡观察者**（见 §六） |
| herdr 自带 | `herdr server stop`（经 API socket 停止）、`herdr status`（本机实测 `herdr --help`） | 收尾优先用**它自己的 stop**（走 socket，不用信号），拿 `HERDR_SOCKET_PATH`/`HERDR_CONFIG_PATH`/`XDG_STATE_HOME` 指到目标 state 目录 |
| Buck2 | 测试运行器协作模型如上；超时行为待核实 | 不指望 Buck2 兜底，CI 侧自建收尾 |

## 五、方案 · 测试层

原则：**谁能自己收就自己收；收不了（KILL9/超时）由 run-id 标签 + 收尾器兜底；兜底也要能在 CI 上变红。**

1. **夹具 RAII 统一。** `AgencyChild`（`room/project.rs:35`）改成与 `room/agency.rs:73`、`room/github.rs:238` 同款：Drop 先 `agency stop`（走 API，让 Agency 自己收 herdr 组、app-server 组、pane），等 5s，再 `kill`+`wait`。`agency/tests/herdr.rs::scratch` 加 `Drop` 删目录。`apps/cli/tests/cli.rs:37`、`cli/human.rs:28` 的 `Temp` 补 `hctl2-services stop`（与 `room.rs:32` 一致）。
2. **给每次测试 run 一个标签。** 夹具启动时把 `HCTL2_TEST_RUN_ID=<buck target>:<pid>:<nonce>` 传进它起的每个常驻进程（Agency/control 已在 env 里传配置，加一个变量即可），并把该 run 的目录收在一个父目录下（如 `$TMPDIR/hctl2-test-<RUN_ID>/`）。进程 env 在 macOS 上可用 `ps eww` 读、目录名自带标签——**归属可精确判定**，不再靠 mtime 猜。
3. **漏测检测（CI 上变红）。** 在 `src/build/ci/` 加收尾器 `reap-test-leftovers`（见 §七），Buck2 测试调用前后各跑一次快照：**本次 run 结束时仍有带本 run-id 的进程或目录 → 列出并退出非零**；CI 的 buck2 job 末尾 `always()` 再跑一次 `--apply` 收尾（只清本 job 的 run-id）。panic/KILL9/超时都由这一步兜底。
4. **Buck2 超时。** 先在 CI 上核实 Buck2 超时杀不杀进程组（§三）；若确认不杀，把「超时」视同 KILL9——由第 3 步兜底；同时给 `agency:herdr_test` 这类慢目标加 `slow-timeout` 意识（本仓用 buck2 自带的 `--timeout`，不另造）。

## 六、方案 · 产品层（只列选法与推荐，所有权定）

现状：Agency 正常退出会收 herdr/app-server（`Drop` 齐备）；Agency 被杀则留，只靠下次在同一 state 目录上 `reap_previous`（且只覆盖 herdr，不覆盖 codex app-server）。这是「Agency 生命周期」问题，三种选法：

| 选法 | 做法 | 利 | 弊 |
| --- | --- | --- | --- |
| A · 维持现状 + 补 reap | `reap_previous` 扩到 codex app-server；state 目录从「凭据根 digest」改为「Agency 根」，让**同 root 必命中** | 改动最小；不引入新进程 | 只在**下一次启动**时收；跨机器/换 root 就永远不收；孤儿期间端口/socket 占着 |
| B · 死亡观察者（推荐） | Agency 起 herdr/app-server 时，同时起一个小观察者（同一进程组外的子进程），用 kqueue `EVFILT_PROC`/`NOTE_EXIT`（mac）或 PDEATHSIG+轮询（Linux，对自家子进程）盯 **Agency 的 pid**；Agency 一死，观察者按记录把 herdr 组、app-server 组、pane 会话逐一 `server stop`/`TERM`→`KILL`，然后自退 | Agency 死即收，**不依赖下次启动**；对 `herdr` 二进制零改动；逻辑只在 Agency 自己的子进程面 | 多一个短命 helper（要进 `src/agency`，新代码）；观察者自己被杀仍需下次的兜底（可再叠 A） |
| C · 交给服务管理器 | 由 launchd/systemd 用户单元托管 Agency 及其子进程（`KillMode=control-group`），生命周期随单元 | 与系统语义一致 | 部署面变大（单机安装要装单元）；与「Agency 是独立服务、控制面只是客户」的现状相比是新的安装契约，超出本议题 |

推荐 **B**（再以 A 作兜底）：观察者的职责与生命线都收在 Agency 自己的进程面内，不动 herdr、不扩部署面；A 顺带把 codex app-server 纳入 reap。拍板前不改代码。

## 七、以后怎么一键清理（只清测试留下的）

提议一个命令（实现时放 `src/build/ci/reap-test-leftovers`，`--dry-run` 为默认，`--apply` 才动手）：

- **归属规则**（满足任一）：进程 env 含 `HCTL2_TEST_RUN_ID`（`ps eww`）；或可执行路径落在测试临时根（`$TMPDIR/hctl*`、`/tmp/hctl*`）或 `*/buck-out*/…/packaged/…`；或目录名带 `hctl2-test-<RUN_ID>`／`hctl2-herdr-{cred,exec,state}-*`／`hctl2-exec-*`。
- **动手方式**：`herdr server stop`（经 `HERDR_SOCKET_PATH`）优先，失败再对**进程组** `TERM`→宽限→`KILL`；codex app-server 同法；会话随其 pane/herdr 组一起；文件只删上述临时目录。
- **永不触碰**（写死在脚本里，不提供覆盖开关）：`~/.local/state/hctl2`、`~/.config/herdr`、`~/.local/bin/herdr` 起的进程（路径匹配到就跳过）、`buck2d*`、`bazel-remote*`、`nativelink`、任何 `run-id` 不在允许集合内的进程。`--apply` 前打印将杀的 pid/命令行与被删目录，要求显式确认。

## 八、待核实与后续

- Buck2 超时对进程组的处理（§三）；`panic=abort` 是否被开启（§三）。
- `agency launch` smoke 目录是否由 `TempDir` 收回（§二 #10）。
- 定案后拆两件：夹具修复 + 收尾器/run-id（测试层，含 `docs/research/build-tools/test-leftover-reaper.md` 的新脚本调研）；产品层按 B（或 A/C）另开。

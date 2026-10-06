# 本地 Agency 参考实现

独立的本机参与者供给方。它有自己的 `agency` 可执行文件、启动命令、数据目录和持久 journal；不安装控制面也能运行。控制面只是配对的客户，一个 Agency 可服务多个控制面。服务只共享 [端口合同](../crates/agency-proto/README.md)，不依赖 control、Store 或命令内核。

形态依据：[演示线开工书第五节](../../.memo/design/p2-control-20260906/07-demo-kickoff.md#五agency-的定位)、[本地参考实现](../../docs/design/delivery.md#本地-agency-参考实现)。租约、授权与结果准入仍归控制面，Agency 的运行报告不能替代它们。

## 文件与运行

| 文件 | 职责 |
| --- | --- |
| `src/service.rs` | 配对、固定租户的 RPC 处理器与独立服务生命周期 |
| `src/tenant.rs` | 每租户自己的派工、输入租约、观测、待交结果 journal |
| `src/storage.rs` | SQLite WAL / FULL、数据权限与独占写锁 |
| `src/runtime.rs` | 私有 Runtime / Session 接口；第 2 包的脚本协议执行体 |
| `src/main.rs` | `agency --root PATH start\|serve\|status\|stop` |
| `skills/` | Harness 原生技能目录 |

Buck：`root//agency:agency`、`:service`、`:cli_test`、`:port_test`、`:control_port_test`、`:clippy`。发行包安装 `agency`；控制面首次执行 `hctl2 agency pair --binding-id ID --key KEY` 时拉起它。可用 `--agency-root PATH` 选择私有数据目录。未配置运行时的 Agency 目录为空，不假报已安装真实 Harness。配对与工种接受重试须保留原 key；查询已接受项不重新发接受命令。

本机端口为权限 0600 的 Unix socket。为适应 macOS 路径长度，套接字目录用数据根摘要生成，数据根与凭据留在私有运维存储，不进入 Binding、Spec 或 Proposal。控制面停止不停止 Agency；`agency stop` 是服务所有者的独立操作，会请求停止所有租户的执行。

## 脚本协议与恢复

`serve --script-config FILE` 读取服务所有者的配置 `{"program":"/absolute/program","arguments":[]}`，不是派工请求中的任意命令。脚本从 stdin 读一行含 Spec / Bundle 的 JCS，之后读获准输入。stdout 每行是 `observation`（kind / payload）或唯一的 `result`（schema / output 字符串）。单帧上限 1 MiB；异常帧标观测缺口，不当成成功。脚本报告的结果和自述只记 narrated。

执行环境清空继承变量，只给 PATH、LANG、本次私有 HOME 和已交付材料。Pointer 文件只读并核摘要。输入写入用原生 socket 超时，物理投递不确定时保留 unknown，不能换 key 盲重发。接管比较并交换撤旧租约，旧租约身份不复用。截止不随断联顺延，过期时报无法履约。

这是可信的协议测试执行体，不是沙箱或生产 Harness：不提供 PTY、exact attach、子进程树隔离、工具直报、敏感输入或 OS 加固。没有真实模型凭据。准备不执行，激活才执行；重启不能证明旧执行有效时报告无法履约，不盲目重跑。已保管结果仍可交回。字节确认后本版仍保留结果，不启用垃圾回收。停机前宜让调用方取得停止报告；突然终止服务后只能按实际可证明的范围恢复。

Agency 的数据库和待交成果不属于 control 备份。它的恢复要保全自己的目录；控制面恢复后只恢复自己的配对和授权，不能从 Agency 记录补出已丢的治理依据。周期核对只联系已有租户，不会拉起被所有者停掉的 Agency。配对钥与服务所有者管理钥不同；租户密钥在首次 RPC 前由客户保存，回复丢失可凭原密钥重试，密钥缺失不凭 Binding 或控制面 ID 取回原钥。当前没有轮换管理命令，遗失原租户凭据须先由服务所有者处理恢复，不把新配对伪装成恢复成功。

## 第 3 包任务说明

3a（#323）、3b（#328）、3c（#335）、3d 已合。3d 把 Claude Code 换成 Herdr 中的常驻交互会话。3e 把 Codex 接成并列工种：正文走一次性 app-server 的 `turn/start`，Herdr pane 跑 `codex resume --remote`。脚本协议与确定性夹具仍只作测试。依据：[演示线开工书](../../.memo/design/p2-control-20260906/07-demo-kickoff.md) §四 第 3 包 3d、3e，以及 `docs/research/harness-dispatch-channel-20261006.md` 的决定建议。

| 文件 | 当前职责 |
| --- | --- |
| `src/herdr.rs` | 官方 Herdr 0.9.3 / 协议 22 的私有客户端与共用服务；物理标识不进入控制面 |
| `src/launch.rs` | 核定制品、Claude 版本与冒烟；私有安装官方 SessionStart 集成；上架只读工种 |
| `src/codex.rs` | 每个选入记录一个一次性 `codex app-server --listen unix://`；`turn/start` 交正文，pane 里 `codex resume --remote` |
| `src/standby.rs` | 按租户、Binding、Project、选入记录排队；用原生 agent 接口派工、打断、闲置关闭和续接 |
| `src/harness/turn.js` | Claude 原生会话插件；将本轮 ID、最终回答与结束原因写到 Agency 私有文件 |
| `src/main.rs` | 随包 Herdr 的摘要核验、独立 Agency 启停与诊断 |
| `src/confine.rs`、`linux_confine.rs` | 限制 Herdr 及子进程读取凭据根；Linux 二进制目录只读、可执行 |
| `tests/herdr.rs`、`tests/standby_fixture.rs` | 原生 Herdr 回归与结构化测试夹具；真实 Claude 会话另设开关 |

`serve` 未设 `HCTL2_INSTALL_ROOT` 时目录仍为空。设了它以后，从安装目录找 Herdr，按 Buck 声明的 `lock.json` 核摘要；受限环境里的终端与 Claude 冒烟通过后才上架。3d 用原生插件机制，Claude Code 最低版本升为 2.1.287，工种记实际版本与二进制摘要。`HCTL2_CLAUDE` 可指定路径，否则从 PATH 找。失败原因留在 `serve.err`，不阻止空目录的 Agency 端口启动。

Herdr 用 `agent.start` 持有 Claude 的交互界面、进程与 PTY；`agent.prompt` 只交本次派工的固定标记，原生插件在 `prompt.submit` 核对标记后注入冻结正文。正文不经过输入框，不变成粘贴附件或原生命令。同一选入记录复用一个会话，派工按 FIFO 逐轮执行；不同租户、Binding、Project 或选入记录不共用。复用键取这些对象的 ID，不取版本或摘要，记录更新不换会话。每轮结果按本次派工键、Spec 摘要、原生 Session ID 与 Turn ID 配对。原生插件的 `turn.complete` 给最终回答与 `answer / refusal / aborted / error`；子代理事件不算主调用结果。回答与拒绝原文交回一条 Proposal、一次 `TurnReturned`，证据为 adapter_event；错误与打断不伪报回答。不看终端空闲或进程退出判这一轮结束。

本包只允许带 `context.read` 的只读 Bundle 派工，每次都重新检查自己的 Spec。工具列表为空、MCP 严格限制；不承诺访问任意工作副本或上一次派工留下的权限。写租约、评审发布策略及其余权限在本实现中拒绝，写入型调用留第 9 包。会话历史复用不证明旧授权有效。

官方 SessionStart 资产由安装器放在 Agency 私有目录，用会话级 `--settings` 引用；Claude 插件只通过 `--plugin-dir` 加载。保留用户 HOME、PATH、USER，不向 Claude 设置 `CLAUDE_CONFIG_DIR`，不复制登录材料、不直接编辑全局设置。按所有者授权，只自动确认 Agency 准备的执行目录的原生信任提示，允许 Claude 保存该目录的信任记录；不跳过工具权限检查。

单次停止或截止先用原生 Esc 打断当前轮，等结构化打断事件，留下会话。Claude 可能把早取消的输入放回输入框；下一次派工用原生 `prompt.edit` 替换旧草稿，保留本次标记的后续片段。`prompt.submit` 拒绝不匹配的标记，再原样注入正文；Runtime 逐字核 `turn.start.text`，首尾空白、换行和制表符都保留。不拦人的原生 `!` 或 `/` 操作；不能归到派工的模型轮不会交成派工结果。三秒内无法确认打断则原生关闭 pane，报告会话已关闭，不伪报正常回答。

`HCTL2_AGENCY_IDLE_MS` 是 Agency 自己的正整数毫秒配置，缺省五分钟；只在无当前轮时关闭闲置 pane。下次派工用 Claude `--resume` 接原会话；失败则起新会话，观测明确记 `resume_failed`。进程在两轮间退出时，Herdr 的 `agent_not_found` / `agent_not_ready` 明确表示正文尚未投递，当前派工可续接后重试一次；传输错误或 `agent_prompt_failed` 不盲重投。续接启动之后再检查取消与截止，已停止的派工不送正文。Agency 停止先收各会话，再原生 `server.stop` 收私有 Herdr。只存续接标识，不自存对话或管理 harness 进程。

只激活 stop，不上架 input，不提供终端接管或 exact attach。一轮返回不是 Task 完成，不代 sysone 判断或生成物验收。状态目录在执行目录之外，但官方版没有逐 pane 防篡改，恶意伪造仍是未实现的策略点。依据与实现前实测见 [Herdr 复核](../../docs/research/sdk/herdr.md#复核记录) 与 [Claude 钩子复核](../../docs/research/harness-hooks-20260903.md#2026-10-06--claude-常驻会话的一轮结束)。

默认 `root//agency:herdr_test` 跑确定性夹具；真实用例用 Rust 原生 `ignore` 标未验证。Codex 的 app-server 子集由 `tests/codex_fixture.rs` 说，`root//agency:codex_fixture_test` 默认会跑：rollout 逐字核对、`turn/completed` 与 `item/completed` 只认本轮、打断后 `TurnStopped`、续接失败起新线程并标 `resume_failed`。开真实验证：`cd src && ./buck2 test root//agency:herdr_test -- --env HCTL2_HARNESS_LIVE=1 --test-arg=live_ --test-arg=--include-ignored --test-arg=--nocapture`。macOS 上测试进程须在可访问登录钥匙串的 Aqua 会话；如果 Buck daemon 属于 Background，先用 Buck 构建，再在 Aqua 中运行同一产物。测试环境另需 Buck 声明的 `HCTL2_LOCKED_HERDR`、`HCTL2_CONFINE_BIN`、`HCTL2_STANDBY_FIXTURE`。真实 Linux 登录会话仍未验证，不放宽其登录材料路径。

夹具模拟原生界面的草稿恢复，不执行插件。`tests/turn.test.ts` 另用 Claude 自带的 `claude plugin test` 测实际生成插件；本机执行 `root//agency:herdr_test -- --test-arg=native_mod_ --test-arg=--include-ignored --test-arg=--nocapture`，需要已安装 Claude，不需要登录。CI 没有原生 Claude，默认略过；改插件后须另跑这组与真实会话用例。

当前限制：目录信任自动确认只识别英文界面；测试目录的信任记录由 Claude 原生保存，不自动删除。失效的续接标识可能等到三十秒启动时限才改用新会话。Herdr 关闭 pane 本身失败时，该选入记录的工作线程可能退出，后续请求报 `STANDBY_UNAVAILABLE`，须恢复 Runtime；不把关闭失败报成已回收。

Codex 工种 `codex-cli` 与 Claude 工种并列，模型字段仍是 `none`。最低版本 0.153.4。`HCTL2_CODEX` 可指定路径，否则从 PATH 找。冒烟不过就不上架，原因写到服务的 stderr（`serve.err`）。app-server 由 Agency 为这个选入记录拉起一次，监听放在 Herdr 套接字同一私有目录里的 `codex-*.sock`，命令是 `app-server --listen unix://…`，不跑 `daemon`、不写 `~/.codex` 的配置、不复制登录材料。登录用进程里已经有的 `CODEX_HOME` 或 `~/.codex`。每一轮 `turn/start` 自带 `sandbox: read-only` 与 `approvalPolicy: never`，不沿用上一轮。rollout 里的 `input_text` 必须与正文逐字相同，否则不交 Proposal。结束看 `turn/completed`，回答取 `item/completed` 里 `agentMessage` 的 `text`，一轮一条 Proposal、一次 `TurnReturned`。新线程的第一轮进行中没有 pane：`codex resume --remote` 要先有 rollout，所以 `ensure_pane` 放在第一轮 `turn/completed` 之后。这一轮在 Herdr 里看不到过程，轮结束时 pane 才出现。2026-10-06 热身一轮返回后，`--remote` 进程有 1 个。Herdr 认得出 pane 里的进程是 `codex`，但没有装 Codex 集成，`agent.list` 里的 `agent_status` 是 `unknown`。Claude 会在私有目录执行 `integration install claude`。要不要给 Codex 同样私有装一份，另议。取消或截止先 `turn/interrupt`；三秒内没有结束，或打断调用失败，就关掉 pane 和这个 app-server，并标明会话已关闭。闲置超时同样关掉两者；下次派工 `thread/resume`，接不上就新线程并在 `session_opened` 里写 `resume_failed`。Agency 停止时工作线程把这些进程收掉。

人在 Codex pane 里敲的字进的是 Codex 自己的界面，不经过 Claude 那条会话插件。本包不做接管。2026-10-06 在 Codex 0.160.1 上对着 `codex resume --remote` 试过三下，都没有按回车把草稿送成一轮：空闲时打 `hello pane`，字出现在输入行；空闲时打 `/`，这个字符出现在输入行，没有执行斜杠命令；另一个客户端的 `turn/start` 还在跑时打 `typed-during`，字出现在界面上，resume 进程还在，观察用的连接在 45 秒内没有读到 `turn/completed`。派工进行中不要在 pane 里打字：若这个观察成立，人在那一轮里打字，这次派工可能一直等到截止。不把 Claude 3d 里插件挡住普通字、`/clear` 换会话号的结果抄过来。

工具直报、工作副本管理与模型字段仍另议。包 4 的接口见 [Context](../crates/context/README.md)，包 5 见 [Participant](../crates/participant/README.md)。

## Buck 与 CT 对照

脚本目录摘要覆盖程序文件字节。没有配置脚本时目录是空的。执行目录在凭据根之外，子进程再被挡住凭据根。挡住的是凭据根，不是操作系统隔离效果，目录里不记录已验证隔离。`ResultPage.complete` 只在这一页已经取到已存结果的末尾、并且派工不再处于 Running 时为真。

两边挡住凭据根的方式不一样。macOS 的 `sandbox-exec` 只拒绝凭据根，其余路径照常，所以 `--script-config` 里的程序可以放在凭据根以外的任何可执行位置，布局检查也不在 macOS 上做。Linux 的 Landlock 是允许名单：工作目录，以及 `/bin`、`/usr`、`/lib`、`/lib64`、`/etc`、`/dev`、`/proc`、`/opt`。`/dev` 和单独挂载的 `/dev/pts` 另外允许写文件和创建字符设备，pane 要读写伪终端，shell 也要把输出写到 `/dev/null`；删除和新建普通文件仍然不放行。凭据根若落在这些目录下面，这一布局无法从允许名单里挖掉，启动会被拒绝，不会把那个祖先目录放行。程序若不在这些目录里，会在 `exec` 时失败。macOS 的配置把凭据路径写成 Scheme 字面量，反斜杠会先转义，避免拒绝指到另一条路径。子进程的 stderr 接到 null，观测里看得到的是退出码；助手自己的失败原因不会进事件。限制没有完全生效时助手退出，目标程序不会启动。脚本观测带 `runtime:` 前缀，不能冒充 Agency 自己的终局事件。独立后台进程使用标准库独立进程组，不宣称已隔离整个子进程树。

| CT-PARTICIPANT / CT-CONNECTION 的行 | 本包失败输入与覆盖 | 后续边界 |
| --- | --- | --- |
| 能力缺失、声明隔离效果 | 脚本没有 secure input / exact attach / no-network；要求后拒绝准备 | 真正隔离效果与终端等级由包 3 验 |
| Skill required / digest / known | 缺 required Skill、直报摘要不一致拒绝；没有报告保持 unknown | 安装与真实核验由包 3 |
| Spec / Bundle / Proposal 摘要与逐项授权 | 改交付字节、消费者、工种条款、输出的 Project 或授权拒绝 | 当前保全非领域准入；包 5 判迟到与取消 |
| 错租户、旧 control writer、票据分权 | 乙读取甲派工、错密钥、旧写者输入拒绝；旧待交结果仍可取 | 本包是本机端口，远程接入待另验 |
| 受管输入与原子接管 | 同租约 CAS 竞争、撤权输入、Observe 票据请求 Input / Stop、管道不读 | 原生交互与真实 Harness 来源完整性由包 3 |
| 终局、取消、退出码、事件缺口 | 正常退出无结果不成功；取消有停止报告；错游标 / 坏帧不报完整历史 | PTY / IME / exact attach 不在本包 |
| 观测不推进治理、工具直报能力 | 自述不能升级直报；接收结果不改测试预置的授权归属者 | 冲突证据仲裁与正式领域准入由包 5 |
| 四步派工、外部结果未知 | 映射前无激活 outbox；prepare 回读原 key；保全后才确认字节 | Invocation 状态机、选人、票据签发由包 5 |
| 重启与成果保管 | 重启保留精确结果与未激活派工；未确认结果重复交回 | 不把 Agency journal 当控制面授权恢复 |
| 联系不上与截止 | 冻结截止到期停止；端口不可达不改领域授权 | 领域超时与丢失判定由包 5 |
| 结果批次与字节预算 | 单份成果超过信封上限时拒绝保存；多份成果按游标分页取回，每一页都在传输上限内 | 第 5 包逐份保全与失败报告 |
| Share、Task / Repo 命令、Write Lease、选入记录、Run | 仅端口合同与接口，不声称完整覆盖 | 分别在包 5、6 及 Run 包 |

## skills/

| Skill | 触发方式 | 用途 | 来源 |
| --- | --- | --- | --- |
| [hctl2-shaping](./skills/hctl2-shaping/SKILL.md) | 人发起 | 塑形：把一个还说不清的目标审问成三张清单（已决、尚未定形、出界），产出只有四种建议（创建 Request、开 Topic Room、雾毕业为 Task、更新 Project 范围） | 改编自 mattpocock/skills 的 grilling 与 wayfinder（MIT，许可证随目录） |
| [hctl2-design-review](./skills/hctl2-design-review/SKILL.md) | 人发起 | 四轴设计评审：机械清点、逐文件发现、每轴新开上下文对抗核验、两档裁决包、逐条人话过给拍板人 | HCTL2 自己的评审方法（2026-09 四轴评审第一到第三轮），可移植到其他分层文档仓库 |
| [hctl2-readback](./skills/hctl2-readback/SKILL.md) | 控制面发起读回时装载，不由模型自取 | 对冻结施工图先复述再对账，只交读回记录、不投票 | HCTL2 读回规矩，见 Skill 的来源节 |

Skill 分两种触发方式，沿用上游的约定：**人发起**的 Skill 是阶段切换（塑形、施工、评审），只有人能按；**模型可自取**的 Skill 是阶段内的纪律（查证据、写测试），模型可以自己伸手拿。判据是「模型能不能有意义地自己伸手拿它」，不是「它是否可复用」。

## 进包

`root//agency:skills` 把技能目录声明为 Buck2 文件组；`root//packaging/release:complete` 把它作为输入交给 `assemble.sh --agency-skills`，安装到发布包的 `payload/share/hctl2/agency/skills/`，与其余 payload 文件一样进入 `PAYLOAD.sha256` 与 SBOM。`test-package.sh` 断言 Skill 与许可证文件在包内。

## 备忘

- **将来生产 Participant 时可借鉴的角色分类**（所有者 2026-09-03 要求记下）：Vercel Labs 的 Foreman 把一条流水线切成四个无状态工位——分类器（读事项，判类型、优先级、能不能动手、要不要先问清楚）→ 分析员（写计划与逐条验收标准）→ 实现者 → 评审员（换一家厂商的模型、只看真 diff、逐条验收标准给过或不过）。每个工位一次调用、进一个信封出一个信封、没有跨工位记忆。对应到 HCTL2 是 Skill 加执行者配置的一种切法，不是新模块；调研见 `docs/research/methodology-sweep-2026h2-20260902.md`。

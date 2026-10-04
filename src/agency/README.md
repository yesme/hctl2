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

所有者 2026-10-05 裁定拆成 3a 与 3b。3a 已合（#323）。本分支是 3b，同日稍后收窄成 Herdr 管道。真 harness 另开一包，不在这里做。

| 文件 | 要补什么 | 状态 |
| --- | --- | --- |
| 结果查询 | 多份成果按游标分页；`complete` 只在已存结果取完且派工不再 Running 时为真 | 3a 已做 |
| `src/confine.rs`、`linux_confine.rs` | 执行目录在凭据根外，并挡住读取。Linux 上伪终端需要写 `/dev` 与 `/dev/pts` | 3a 已做；pty 写放行在本管道 |
| `src/catalog.rs`、脚本目录 | 摘要用程序文件或 `SKILL.md` 字节；`verification` 留空 | 3a 已做 |
| `src/runtime.rs` | 保留 `Runtime::catalog/start`、`Running`、`Session::input/stop` 和 `RuntimeEvent` | 脚本执行体已有。本管道不把 Herdr 接进 `serve` |
| `src/herdr.rs` | 锁定 Herdr 0.8.2 / 协议 20。调用方给出 pane 里的程序。一个管道一个服务。状态目录在执行目录旁边。命令原文含有标记，或含换行以外的控制字符（制表符也算），都在建 pane 之前拒绝。启动失败和 pane 出错都会收掉已经拉起的进程 | 本管道 |
| `src/workspace.rs` | 复用 Herdr 和 hctl2-tool 的物化 / 核验；Write Lease 与目标边界照原授权执行 | 不带 |
| `tests/herdr.rs` | 锁定制品上的 ping、工作区、pane、输入读回、关闭；两个派工同时启动；凭据根；新目录上的 `serve` 不上架 Herdr。标记拆开打印并放在命令最后一步。命令原文含标记，或含退格、删除，则拒绝且不执行。标记不出现时关掉 pane。pid 写失败后没有存活的 Herdr | 本管道 |

本管道的验收（所有者 2026-10-05 裁定「收窄吧」）：

1. Linux 与 macOS 上 `herdr_test` 都通过。受限的 Herdr 能开 pane。
2. 写死的 `printf` 不在正式路径里。`serve` 不因为 `HCTL2_LOCKED_HERDR` 上架工种。pane 里跑什么由调用方给，确定性小程序留在测试里。
3. 两个派工同时经 Herdr 启动，只起一个 Herdr 进程，两边各自拿回自己的输出。
4. pane 里读不到凭据根，两个平台都有用例。状态目录在凭据根之外，并跟执行目录放在一起。
5. 新目录上直接 `serve` 能起。布局检查只在 Linux 上做。不改安装目录里制品的权限。异常退出留下的 Herdr 下次启动时收掉。关 pane 的错误要交回。允许路径规范化失败不退回原始路径。
6. 真 harness、把任务交给执行体并作为 Proposal 交回、按安装目录核摘要后上架，都挪到下一包。

可预测的执行目录若已存在且不属于当前用户，拒绝并给出 `UNSAFE_ENDPOINT`。没有配置脚本时目录是空的。包 4 的任务说明在 [Context](../crates/context/README.md)，包 5 在 [Participant](../crates/participant/README.md)。

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

# 测试残留进程的收尾器（reap-test-leftovers）

> 对象：本仓 CI 与开发机上「清掉测试留下的常驻进程与临时目录」的那一个脚本。
> 状态：提议（随 [.memo/design/test-process-cleanup-20261018.md](../../../.memo/design/test-process-cleanup-20261018.md) 等所有者拍板，未实现）。
> 结论先行：不引入新依赖，用一个随仓脚本 + 「每次测试 run 一个 run-id」解决归属；清理手段优先用各服务自己的 stop，其次按进程组 TERM→KILL。

## 一、要解决的问题边界

- 只清**测试留下的**：`agency serve` 起的 `herdr server`、`codex app-server`、会话，control/process-compose 起的 Tuwunel/Gitea，以及 `/tmp` 下的 `hctl2-*` 临时目录。
- 不碰所有者自用的 `herdr`、各席的 `buck2d`/`bazel-remote`/`nativelink`、`~/.local/state/hctl2`、`~/.config/herdr`。
- 触发面：CI job 末尾（`always()`）与开发机手动一键。

## 二、查过的现成做法

| 方案 | 机制 | 为何不直接采用 |
| --- | --- | --- |
| cargo-nextest 的 leak 检测 | 测试退出后若子进程仍持有继承的 stdout/stderr，标 `LEAK`，可配成失败 | 只对**继承 stdio** 的子进程有效（[leaky-tests](https://nexte.st/docs/features/leaky-tests)）；本仓的 `herdr`/`app-server` 明确把 stdio 置 null，抓不到。且本仓用 Buck2，不是 cargo test |
| testcontainers Ryuk | 旁路 reaper，资源打 session 标签，观察测试进程生命线，断开即清（[moby-ryuk](https://github.com/testcontainers/moby-ryuk)） | 需要一个常驻 reaper 与 Docker；本仓没有容器运行时依赖，引它等于加依赖与新服务 |
| systemd/launchd 托管 | `KillMode=control-group`，进程随单元收 | 安装契约变大（单机安装要装单元），超出本议题；已在方案 §六 列为产品层选法 |
| Linux `PR_SET_PDEATHSIG` / subreaper | 父死信号 / 中间进程收孤儿 | 只能由**子进程自己**设置；`herdr` 是外部二进制，改不了。只对 Agency 自家子进程可用 |
| macOS `kqueue NOTE_EXIT` / 生命线管道 | 子进程主动监视父进程或读 EOF 自退 | 同上，子端改不了；可用在 Agency 新起的**观察者**上（方案 §六 B） |
| `herdr server stop` | herdr 自带，经 API socket 停 | **采用**：清理时优先走它，避免信号硬杀；需按目标 state 目录给 `HERDR_SOCKET_PATH` 等环境 |

## 三、提议的形态

一个 POSIX shell 脚本 `src/build/ci/reap-test-leftovers`：

- 默认 `--dry-run`：打印将处理的进程（pid、ppid、命令行含 run-id）与目录；
- `--apply`：先对 `herdr server` 用 `herdr server stop`（按 state 目录逐个指 socket），失败再对进程组 `TERM`→宽限→`KILL`；`codex app-server` 同法；最后删带 run-id 或属已知测试目录形态的临时目录；
- 归属：进程 env 里的 `HCTL2_TEST_RUN_ID`（`ps eww` 可读）或可执行路径/目录名落在测试临时根；**白名单写死**（见上），不提供绕过开关；
- CI 里分两次：本次 run 结束先查（发现本 run 残留即非零 → 漏测变红），job 末尾 `always()` 再 `--apply` 收尾。

不新增依赖、不新增常驻服务；脚本本身与调用点各约一百行以内，属「构建动作走 Buck2 原生目标」的例外之一（Buck2 没有跨测试生命周期的收尾动作），实现时按仓规在 PR 里单独论证。

## 四、风险与待核实

- `ps eww` 在 macOS 上能否读到 env（取决于权限/进程是否同用户）——实现前实测。
- Buck2 超时是否杀进程组未实测（见方案 §三、§八），若确认不杀，超时完全依赖本脚本。
- 清理脚本的「白名单写死」要随产品层的观察者方案（方案 §六 B）再核一次，避免收掉观察者自己。

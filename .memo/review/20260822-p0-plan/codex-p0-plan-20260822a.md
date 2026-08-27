# P0 实施计划（Codex）

- 日期：2026-08-22
- 作者：Codex
- 状态：Proposed
- 性质：独立实施方案；P0 只做可丢弃、限时、可复现的风险探针，不实现产品功能

## 1. 结论先行

P0 应拆成五条彼此独立的验证轨道，而不是四个上游组件的安装演示：

1. `P0-SANDBOX`：OS sandbox、Git broker、credential gateway 的安全闭环。
2. `P0-RUNTIME`：Zellij 作为交互运行时的控制、观测与故障恢复语义。
3. `P0-CHAT`：Tuwunel 的事件身份、同步、幂等与备份恢复语义。
4. `P0-TASK`：Vikunja 的对象映射、并发更新、回读对账与备份恢复语义。
5. `P0-ENGINE`：Conductor 的外部任务、重试、超时、取消和恢复语义。

其中 `P0-SANDBOX` 是现有四项之外必须显式增加的阻断项。Zellij 负责终端复用，不等于安全边界；如果 macOS/Linux 上的 OS 级约束无法满足设计契约，P1 不应靠 prompt、路径约定或“代理自觉”继续前进。

P0 也不是一个全局大门：

| 轨道 | 直接解除的门禁 | 不应阻塞 |
| --- | --- | --- |
| CHAT + TASK | B1 | B2 的本地安全与运行时验证 |
| SANDBOX + RUNTIME | B2 | B4 的持久编排验证 |
| ENGINE | B4 | B1、B2 |

因此实施上采用两波并行：第一波完成 SANDBOX、RUNTIME、CHAT、TASK；第二波集中完成 ENGINE 和跨轨道对抗性复验。任何一条轨道只对自己的门禁负责。

## 2. P0 的交付边界

### 2.1 P0 要交付什么

每条轨道最终只交付以下一种或多种设计证据：

- 经校验的上游版本、源码 commit、下载 URL 和校验和；
- 可复现的命令、最小配置、协议 fixture 和故障时间线；
- 与当前契约逐项对应的 PASS/FAIL 结果；
- 已证实的降级路径及其产品化约束；
- 足以触发重选组件或修改契约的反例；
- 对安装、升级、备份、恢复和资源预算的事实记录。

已接受的结论汇总到 `docs/research/README.md`。只有当结果改变组件选择或设计契约时，才同步修改 `docs/design/references/decision-history.md`、交付计划或相应 spec。

### 2.2 P0 不做什么

- 不创建正式 CLI、daemon、lifecycle manager 或统一 adapter 层；
- 不实现 Control、Workbench、Scene、Run 等产品域逻辑；
- 不为了复用而先造通用 probe SDK、测试框架或部署框架；
- 不把“进程启动成功”“网页可打开”当成语义验证；
- 不把 RSS、安装包大小和启动耗时等观测指标误写成安全或正确性结论；
- 不用 prompt 约束代替 OS 强制隔离，不用人工重试代替恢复语义。

## 3. P0-00：先冻结 Probe Charter

在五条轨道正式运行前，用半天完成一个小型文档变更，统一以下规则。

### 3.1 支持矩阵

最低强制矩阵：

- `darwin/arm64`：当前主开发机；
- `linux/amd64`：真实 Linux 主机或 VM，不以 macOS 容器行为代替宿主行为。

若交付范围还包括其他 OS/arch，P0-00 必须明确它是发布阻断项还是仅做安装烟测。不同 OS 上不得用同名但语义不同的 sandbox 实现互相代证。

### 3.2 指标分级

强制指标：

- 安全边界是否由 OS 强制执行；
- 稳定身份、幂等、排序、并发和回读语义；
- 进程崩溃、ACK 丢失、宿主重启后的恢复语义；
- 备份可恢复性以及恢复前后的身份不变量；
- 必需能力是否能在无 GUI、可自动化的路径上完成。

观测指标：

- 安装包与运行数据体积；
- 冷/热启动耗时；
- 空闲和负载 RSS/CPU；
- 安装步骤、外部依赖和升级复杂度。

观测指标必须记录，但除非 P0-00 另行给出阈值，不单独决定 PASS/FAIL。

### 3.3 统一 verdict

- `PASS`：强制契约全部成立，未引入新的产品级前提。
- `PASS_WITH_DOWNGRADE`：核心目标成立，但必须采用已写入 spec 的降级语义。
- `REOPEN_SELECTION`：候选组件或平台机制不能满足不可降级要求，需要重选。
- `REVISE_CONTRACT`：现有契约在目标平台上不可实现或成本失真，需要先改设计。
- `INCONCLUSIVE`：环境、fixture 或观测不足；不能作为过门依据。

每条轨道在开始前要写出“不允许降级”的条件，防止看到结果后移动门柱。

### 3.4 统一证据记录

每次运行至少记录：

```text
probe_id
started_at / finished_at
operator / harness / model
host_os / kernel / arch
component_version / source_commit
artifact_url / checksum
config_digest
exact_commands
fixture_digest
fault_injection_timeline
expected_observation
actual_observation
raw_artifact_paths
verdict
known_limitations
reproduction_notes
```

所有时间线同时记录 UTC wall clock 和单机 monotonic offset；跨进程因果判断优先使用协议身份和明确 ACK，不依赖日志时间戳碰巧相邻。

## 4. P0-SANDBOX

### 4.1 要回答的问题

在 Darwin 和 Linux 上，能否让一个被控制的 harness：

- 只直接访问当前 Execution Spec 授权的工作区和工具；
- 不能读取或修改兄弟 ChangeSet、其他仓库、用户密钥和 provider 配置；
- 不能绕过 Git broker 直接修改受保护的 repository/common-dir 状态；
- 只能通过 credential gateway 使用短期、最小权限凭据；
- 在子进程、符号链接、继承 fd、本地 socket 和网络路径上仍保持上述边界；
- 让拒绝、授权和凭据使用都可审计。

### 4.2 最小实验矩阵

正向样例：

- 读写当前 ChangeSet 中明确授权的文件；
- 执行 allowlist 中的编译器、测试器和只读 Git 操作；
- 通过 broker 创建经过授权的 Git 变更；
- 通过 gateway 调用被授权 provider，并成功完成一次最小请求。

负向样例：

- `..`、绝对路径、symlink、hardlink 和 rename 穿越；
- 读取 sibling worktree、主仓库 common-dir、其他项目和 `.git` 敏感状态；
- 读取 SSH key、agent socket、Keychain、云凭据、provider 配置和父进程环境；
- 复用继承 fd、Unix socket、localhost service 或 helper process 越权；
- 启动未授权二进制、修改 broker/gateway、向未授权网络目标外连；
- 在 sandbox 内转储或持久化 gateway 使用的凭据；
- 被杀死、超时或宿主重启后遗留一个脱离监管的子进程。

每个负向样例都要区分“系统调用被拒绝”“目标本来不存在”和“测试没有真正触达目标”，后两者不算通过。

### 4.3 故障注入

- 在 broker 写 Git 状态前、写入后、返回 ACK 前分别杀进程；
- 在 gateway 获取凭据前、provider 已提交后、回传结果前断开；
- 执行中撤销授权、轮换凭据、关闭本地 broker/gateway；
- sandbox owner 崩溃后检查进程树、mount/namespace、socket 和临时凭据残留。

### 4.4 通过条件

- 所有不可降级的负向样例由 OS 或受信任 broker/gateway 强制阻断；
- 代理进程本身不能修改策略、扩大 allowlist 或取得长期凭据；
- Git/provider 的“已提交但 ACK 未知”有稳定操作身份和回读路径；
- Darwin 与 Linux 的差异被显式建模，产品层可得到同一组安全不变量；
- 结果可在干净主机上复现。

若 Darwin 只能依赖已弃用、粒度不足或无法动态配置的机制，直接给出 `REVISE_CONTRACT` 或 `REOPEN_SELECTION`，而不是在 P1 中补一层软校验。

## 5. P0-RUNTIME：Zellij

### 5.1 要回答的问题

Zellij 是否能作为由 hctl 拥有、无 GUI 依赖、可查询和可恢复的交互会话运行时，而不只是一个人类手工使用的 terminal multiplexer。

### 5.2 功能与协议探针

- 创建、枚举、定位、关闭 hctl 独占的 session/pane；
- attach、detach、输入、resize、scrollback/输出采集及退出码观测；
- 两个观察者并发 attach，其中一个慢读或断线；
- owner socket/目录隔离，避免混入用户自己的 Zellij session；
- 禁用 web sharing，并确认不会意外监听非 loopback 地址；
- headless 查询路径不依赖解析面向人的彩色 TUI 文本；
- 验证 Codex、Claude Code、OpenCode 的主交互路径；其余 harness 做启动、输入、退出烟测；
- 覆盖 kitty keyboard、bracketed paste、窗口 resize、UTF-8 和长输出。

### 5.3 故障注入

- 杀 agent 进程、观察客户端、Zellij client、Zellij server；
- 在输出高峰、输入中途、pane 刚创建但身份尚未回传时杀进程；
- 宿主重启后检查哪些状态可恢复、哪些必须明确结束为 interrupted；
- session 名称冲突、残留 socket、磁盘满、权限变化；
- observer 长时间不消费输出，验证不会阻塞 agent 主进程。

### 5.4 通过条件

- hctl 能用稳定、机器可读的身份控制和观测目标 pane；
- client/server/agent/host 四类故障能被区分，不伪造“仍在运行”；
- attach/detach 和观察者行为不改变 agent 的业务输入输出语义；
- 默认不暴露 web/network surface，用户原有 Zellij 状态不受影响；
- 至少在强制 OS/arch 矩阵上通过终端兼容性用例。

原始 attach 可以保留为 break-glass 入口，但不能成为产品控制面的唯一接口。

## 6. P0-CHAT：Tuwunel

### 6.1 要回答的问题

Tuwunel 是否能为 Room timeline 提供稳定事件身份、可恢复同步、重复提交收敛和可验证备份，而不把内存 cursor 或本地日志位置当成事实身份。

### 6.2 语义探针

- 用相同 transaction identity 重复发送，验证幂等结果；
- 两个 writer 并发发送，记录服务端排序和客户端可见顺序；
- 记录 event ID、sync token/cursor，并在断线后增量恢复；
- 制造 cursor 过期、gap、重复 page 和乱序到达，执行 backfill/resync；
- 创建 edit/redaction/relation，验证原事件身份及投影规则；
- 服务重启、客户端重启后重放同一请求；
- 验证 admin/AppService 的最小权限、认证边界和 loopback 暴露范围；
- 验证 webhook/推送不可用时，poll/sync 仍可恢复事实状态。

### 6.3 备份恢复探针

- 有持续写入时创建 RocksDB 在线备份；
- 校验备份，再恢复到一个全新实例和全新数据目录；
- 对比关键 room、event ID、关系、权限与 sync 恢复行为；
- 在备份开始前、进行中、完成后各标记事件，说明一致性边界；
- 对损坏或不完整备份验证失败是显式的。

### 6.4 通过条件

- 重复请求收敛到同一事实结果，不生成不可解释的重复事件；
- gap/断线/重启后可以仅凭服务端身份恢复，不依赖单机日志偶然保留；
- edit/redaction 不破坏原事件追踪和审计关系；
- 恢复后关键身份不变量成立，且恢复步骤可自动化；
- 管理权限和网络暴露不超出本地产品需要。

## 7. P0-TASK：Vikunja

### 7.1 要回答的问题

Vikunja 能否承载当前任务投影，同时把“对象稳定身份”“所属 group/board”“显示位置/排序”分开，不让 UI 排序字段承担并发控制或事实身份。

### 7.2 映射与漂移探针

- 创建 Repo 对应的 project/board、group anchor 和 task/card；
- 移动、重排、改名、归档、删除并重建 group，观察稳定身份和投影漂移；
- 同一 task 在不同视图/位置变化后验证产品侧关联不丢失；
- 人类在 UI 修改、hctl 同时修改，验证冲突检测和最终对账；
- 特别验证 task-position 相关 endpoint 是否支持 ETag/`If-Match`；
- 若具体 endpoint 没有 CAS，验证“单写 broker + revision 回读 + reconcile”的降级是否足够；
- webhook 丢失、重复、乱序时使用 polling/reconcile 恢复；
- create 已提交但 ACK 丢失时，用外部稳定 identity/readback 避免重复对象。

### 7.3 备份恢复探针

- 对 SQLite 进行受支持的一致性备份，而不是直接复制活跃数据库文件；
- 恢复到全新实例后核对 project/group/task 身份、关系和关键 revision；
- 在恢复后立即执行一次 reconcile，确认不会把全部对象误判为新对象；
- 验证 schema migration 前后的备份、回滚边界和失败提示。

### 7.4 通过条件

- 映射能稳定区分 anchor identity、task identity 和 placement；
- 并发写不会静默覆盖；原生 CAS 或明确降级路径至少有一个成立；
- webhook 只作为低延迟提示，丢失后 polling/reconcile 可以恢复；
- ACK 未知和人工 UI 漂移都有确定 readback/对账路径；
- 备份恢复保持映射所依赖的不变量。

若需采用无原生 CAS 的降级路径，应给出 `PASS_WITH_DOWNGRADE`，并在进入 B1 前把限制写入 spec。

## 8. P0-ENGINE：Conductor

### 8.1 前置约束

这条轨道只验证 B4 所需的持久编排语义。它不得因为镜像制作、UI 或搜索组件拖住 B2。首选验证 Postgres-only 的最小拓扑；如果候选版本实际上强制依赖 Elasticsearch 或另一套持久层，必须把它作为选择成本显式报告。

### 8.2 外部任务语义探针

- 创建 workflow，poll 外部任务，完成、失败、retry 和 timeout；
- 验证 workflow/task/external operation 的稳定身份及 correlation；
- 检查 poll lease、callback/response timeout 和 worker freshness；
- pause/resume、cancel、terminate、restart 后验证状态机；
- 多 worker 竞争同类任务，检查重复领取和幂等完成；
- timer、retry backoff 和服务重启后的到期行为；
- 服务端/worker 时钟偏移下，不把 lease freshness 建立在不可信本地墙钟上。

### 8.3 故障注入

- worker 领取前、领取后未 ACK、业务副作用后、complete 前后分别 kill；
- complete 已提交但响应丢失，随后重复 complete/readback；
- Conductor 进程、Postgres、网络分别中断并恢复；
- retry 与 cancel 竞态、timeout 与 late completion 竞态；
- rolling upgrade 和 schema migration 中断；
- 备份恢复后核对 workflow/task identity 和待执行任务集合。

### 8.4 通过条件

- 每种 ACK 未知状态都有 readback 或幂等重试路径；
- retry/timeout/cancel/restart 的实际状态机可映射到当前 Run 契约；
- worker 崩溃不会永久丢任务，也不会把重复副作用伪装成 exactly-once；
- Postgres-only 拓扑、备份恢复和升级路径可被产品化；
- 必要管理能力可通过 API/CLI 自动化，不依赖 UI 人工操作。

如果只有修改 Run 契约才能诚实映射上游语义，给出 `REVISE_CONTRACT`；不要在 adapter 中偷偷发明第二套状态机。

## 9. Harness 分工

分工按“架构冻结—精细实现—对抗复验—干净重跑”组织，而不是让多个 harness 同时改一份大脚本。

| Harness | 主责 | 复核/限制 |
| --- | --- | --- |
| Claude Code + Fable-5 | P0-00 charter；Conductor 最小拓扑与状态映射；最终跨轨道架构裁决 | 冻结接口和不变量后再交给实现者；不单独签署精细故障代码正确性 |
| Codex + GPT-5.6-sol | SANDBOX；Git broker/credential gateway 探针；复杂 fault driver；CHAT 并发语义复核 | 不在 P0 中抽象通用框架；架构边界由 charter 约束 |
| Grok Build + Grok-4.6 | RUNTIME 的局部用例；ENGINE 对抗性测试计划；状态机边界/竞态攻击 | 每个任务限制为一个清晰 fixture 或故障场景，不独自维护跨轨道大逻辑 |
| Kimi Code + K3 | CHAT 的脚本、恢复 fixture 和证据初稿 | Codex 复核事件身份、并发和 ACK-loss 分支 |
| OpenCode + GLM-5.3 | TASK 的映射、CAS、webhook/poll 和 SQLite 恢复探针 | Fable 复核对象映射是否符合架构语义 |
| Antigravity + Gemini-3.7-flash | 干净环境安装、checksum、端口、RSS、启动耗时及已通过用例的机械重跑 | 不负责制定 acceptance，也不能用烟测覆盖语义失败 |

第一轮输出本身也是 harness 校准样本。按以下维度记录：首次可复现率、遗漏断言、错误阳性/阴性、修复轮数、无关抽象数量和证据完整度。若实际表现与预设分工不符，第二波直接调整任务，不维护“模型面子”。

## 10. 执行波次与时间盒

### Day 0：P0-00（0.5 天）

- 冻结支持矩阵、verdict、强制指标、证据模板和不允许降级项；
- 为各上游记录候选 tag，但直到下载校验和源码 commit 都记录后才算 pin；
- 修正“Linux 全原生”与 Conductor 容器化等交付描述冲突；
- 为五条轨道各开独立临时目录和端口段。

### Wave 1：B1/B2 风险（1.5–2 天/轨道，并行）

- Codex：SANDBOX；
- Grok：RUNTIME；
- Kimi：CHAT；
- OpenCode：TASK；
- Fable：持续做契约答疑和结果审阅；
- Gemini：在第二环境重跑已经稳定的步骤。

CHAT + TASK 通过即可解除 B1；SANDBOX + RUNTIME 通过即可解除 B2。某一组失败不拖住另一组继续获得证据。

### Wave 2：B4 与红队复验（2 天 + 1 天复验）

- Fable 冻结 Conductor 拓扑与状态映射；
- Codex 实现有状态 fault driver；
- Grok 提供竞态与 ACK-loss 攻击清单并实现局部 fixture；
- Gemini 在干净 Linux 环境机械重跑；
- Kimi/OpenCode 交叉复跑 CHAT/TASK 的恢复路径，避免原作者只验证 happy path。

目标墙钟时间约一周。时间盒结束仍为 `INCONCLUSIVE` 的轨道不能静默延期：必须说明缺少什么证据、继续投入的上限以及是否应重选方案。

## 11. 工作区与协作纪律

- 所有下载、数据库、日志、临时证书和脚本先放在 `.tmp/p0/<track>/`；
- 每条轨道使用独立工作目录、数据目录、端口段和进程 owner 标识；
- 上游二进制只从记录过的 URL 获取，并校验 checksum；从源码构建则同时记录 commit 和 toolchain；
- 禁止使用共享的默认数据库、用户现有 Zellij session、用户真实 SSH key 或生产 provider token；
- 所有 kill/fault 操作必须先解析精确 PID、数据目录和端口，不能使用宽泛进程名或递归路径；
- 原始脚本和日志保留到证据 review 完成；之后临时材料可丢弃；
- 只有定义长期协议回归的最小 fixture 才晋升到版本库，不把整个 P0 harness 产品化；
- 同一条结论只由一个 owner 写入 implementation evidence，其他 harness 以 reviewer/rerunner 身份签名；
- 一个提交只承载一条轨道的证据或一次明确的契约修订，避免把多个 verdict 混在一起。

建议临时布局：

```text
.tmp/p0/
  sandbox/
    bin/ data/ fixtures/ logs/ runs/
  runtime/
    bin/ data/ fixtures/ logs/ runs/
  chat/
    bin/ data/ fixtures/ logs/ runs/
  task/
    bin/ data/ fixtures/ logs/ runs/
  engine/
    bin/ data/ fixtures/ logs/ runs/
```

## 12. 每条轨道的 review checklist

在给 verdict 前，reviewer 必须逐项回答：

- 实验是否真的触达待验证机制，而非因为目标不存在或权限本就缺失而“通过”？
- 是否同时验证正常路径、重复请求、并发、崩溃前后和 ACK 丢失？
- 身份来自服务端/协议，还是来自进程内存、日志行号、排序位置等偶然状态？
- 恢复是否在全新实例/数据目录发生，还是原进程重启后继续用了旧缓存？
- 备份是否经过实际 restore 和语义核对，而不只是成功生成文件？
- webhook/事件推送丢失时，是否存在完整 readback/reconcile？
- 测试是否在至少两个强制平台上复现，平台差异是否进入结论？
- 是否保留了失败样本和反例，而不是只保留最终绿色日志？
- `PASS_WITH_DOWNGRADE` 的降级是否已经写入 spec，调用方是否看得见？
- 证据能否让没有参与实验的人按命令独立复现？

## 13. P0 完成定义

P0 整体完成不要求五条轨道全部 PASS；它要求五条轨道都得到足以作决策的终局 verdict，并且每个相关门禁被诚实更新。

完成条件：

1. 五条轨道均有版本/commit/checksum、环境、命令、故障时间线和 reviewer；
2. 每条强制契约均对应至少一个正向和一个负向或故障用例；
3. 每次备份验证都完成了真实 restore；
4. 所有 ACK-loss 分支都有明确的 identity、readback 和重试结论；
5. 跨平台差异已写成产品约束，不藏在本地脚本中；
6. implementation evidence 已更新，且失败证据没有被删除；
7. 组件重选或契约修改已进入 decision history/spec；
8. B1、B2、B4 分别依据自己的轨道结论更新，没有互相绑架；
9. 未把临时 probe framework 或未经门禁的产品代码带入 P1。

P0 的成功标准不是“证明当前选择都对”，而是在一周左右用最小成本淘汰错误假设，并给 P1 留下一组窄、硬、可回归的实施约束。

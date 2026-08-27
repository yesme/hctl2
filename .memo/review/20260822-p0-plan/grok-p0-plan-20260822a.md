# HCTL2 P0 实施计划

> 状态：已废弃：Zellij/Conductor 选型 v0.12.1 改为 tmux/Dagu（§18、§19），P0 v0.13.0 收窄为只验接缝（§25）；未采纳为施工依据<br>
> 基线：草案 v0.12.0（2026-08-22 main，未记 sha）<br>
> 去向：无；P0 权威在 docs/design/delivery.md<br>
> 日期：2026-08-22<br>
> 对象：草案 v0.12.0 的 P0（`docs/design/delivery.md`「实现阶段」「开工前限时验证」「打包策略」；沙箱入场券见 `spec/system.md` / `spec/agent.md` 与缺口审计）<br>
> 说明：Informative · 实施提案，不改规范。

## 一句话

P0 不是开工写产品。它是对已拍板假设做限时、可丢弃的探针：通过只留下实现证据、pinned 版本和产品化约束，不宣称四套服务器已经可运维。P1 不得依赖探针脚本。

施工顺序仍是 P0 → P1（`hctl2-agentd` + `hctl2-tool`）→ P2（control + CLI，B0–B5）→ P3（Workbench）。B0–B5 全在 P2 内部。Conductor 只挡 B4，不挡 B2。

## P0 验什么

远端 Linear/GitHub 已移出 P0。剩下四条假设，外加缺口审计补进来的一条阻断项：

| 探针 | 已拍板 | 必须在何时有证据 | 真正要验的合同假设 |
| --- | --- | --- | --- |
| chat server | Tuwunel（Continuwuity 备选） | B1 首次消费前 | 事务 ID 幂等、单 homeserver 线性顺序、重同步、账号/房间 API、备份机制；固定 release/commit/RocksDB/features |
| task server | Vikunja（git-bug 对照） | B1 首次消费前 | 排序令牌 / 条件写入、泳道、身份稳定、webhook/轮询、备份机制 |
| 运行时 | Zellij（tmux 降级） | B2 前 | attach/输入/resize/重启/残留；无人 attach 时的终端查询应答；kitty keyboard × 各 harness；web 客户端默认关 |
| workflow engine | conductor-oss | B4 前即可，不挡 B2 | postgres-only（免 ES）最小栈、分发重量、备份机制 |
| OS 沙箱 | 合同已立法，尚未探针 | B2 前，否则受治理 Harness 不能启动 | OS 强制沙箱挡住凭据、SSH agent、target ref、其他 ChangeSet；做不到的候选标 unsupported，不降成 prompt |

公开资料里已经能预判几处会卡探针，不要到写脚本才发现：

- Tuwunel 只支持 RocksDB，官方不出 darwin 包。打包策略写的是我方 CI 交叉编译。本机是 macOS，P0 必须先证明「本机有可用二进制」或「Linux VM 探针 + darwin 交叉编译路径」。
- Vikunja API v2 有 ETag / `If-Match`。合同押的是排序的条件写入，探针必须打在 position/sort 的写路径上，不能用 GET 304 冒充 CAS。
- Zellij 内建 web 客户端是产品能力；合同要求默认关，瘦身用 `zellij-no-web`。探针要量打包重量和常驻内存。
- Claude Code / Codex 自己已有一层 sandbox。外层再套 OS 沙箱可能直接起不来——这是沙箱探针的架构风险，必须先写清楚再动手。

## 仓库形态

交付物只有三样：`implementation-evidence.md` 的 pinned 版本与实测、失败则重开的 `decision-history`、以及一份「control 以后怎么托管」的产品化约束。探针脚本、临时库、拼装环境不进入产品生命周期。

```text
probes/                  # 可复现证据的方法，不是 crate
  README.md              # 大字写：不是产品，P1 禁止依赖
  _template/RESULT.md    # 假设 → 通过/失败/重开
  tuwunel/ vikunja/ zellij/ conductor/ sandbox/
  .scratch/              # gitignore，数据扔这里
docs/design/references/implementation-evidence.md  # 唯一正式出口
```

分四波。日历大约 10–12 个工作日（并行），不要六条探针串行。

## 第 0 波 · 探针章程（1 天，串行，人拍板）

先写出每条假设的通过/失败/重开标准、时间盒、证据模板。这是 P0 的架构，不是代码。

| 角色 | 干什么 | 为什么 |
| --- | --- | --- |
| Claude Code + Fable-5 | 写章程：目录、假设表、RESULT 模板、时间盒、失败如何改 decision-history | 本仓库的设计文档就是它写熟的；P0 最怕的是「二进制能启动 = 合同成立」 |
| Grok Build + Grok-4.6 | 对抗审章程：漏了哪些负例、时间盒是否假、沙箱算不算第 5 条探针 | 对抗分析和测试计划是这套组合的长处 |
| 人 | 批准章程，冻结时间盒 | 选型重开、unsupported 判定不能交给模型 |

这一波不要上 Codex。它写精细逻辑强，但架构上容易叠床架屋；章程一旦补丁化，后面四条探针会各写各的。

## 第 1 波 · 存活烟雾（1 天，全并行）

问题只有「能不能在这台机器上起来」。Antigravity + Gemini-3.7-flash 四条一起跑：

1. Tuwunel：有没有 darwin 预编译？没有的话交叉编译 / 容器路径是否通？loopback 能否起、`server_name` 最小配置是什么？
2. Vikunja：官方单二进制 + SQLite，能否建 project、拿 token、打一发 API v2？
3. Zellij：版本、session 创建/attach/detach；web sharing 默认值。
4. Conductor：Colima/Podman + postgres-only（不要 ES）能否起来？只记启动时间和镜像重量。优先级最低，抢不过前三条就放。

Gemini 的产出必须是「命令 + 日志 + 是/否」，禁止写成合同结论。「Tuwunel 起来了」不等于「事务 ID 幂等成立」。

## 第 2 波 · 合同假设（3–5 天，三条关键路径并行）

章程通过、烟雾不炸之后，开三条互不依赖的轨道。Conductor 仍不进关键路径。

### 轨道 A · Tuwunel（挡 B1）

| 角色 | 干什么 |
| --- | --- |
| Grok | 写测试计划：同一 txn ID 重放、并发打事件后的线性顺序、resync、Appservice 程序化注册（不靠房内发命令）、RocksDB 备份/恢复是否一致 |
| Kimi Code + K3 | 按计划实现探针脚本（Client-Server API 客户端、幂等/顺序用例） |
| Codex | 只在「darwin RocksDB 编不过 / 顺序用例要精细同步」时接手；范围锁死在那一个难点 |
| Fable | 把 Matrix event id / txn id 对上 Room 引用与冻结 digest，判定何谓「线性顺序对治理够用」 |
| Gemini | 跑脚本、贴日志 |

失败条件（章程里写死）：没有程序化 Appservice、不能证明 txn 幂等、darwin 既编不出也不接受「Linux 执行面」——重开 chat server 选型，动 Continuwuity，不在 Tuwunel 上补三层包装。

### 轨道 B · Vikunja（挡 B1）

这是最适合 Codex 的一条：并发、条件写入、身份稳定性，逻辑勾稽密。

| 角色 | 干什么 |
| --- | --- |
| Grok | 测试计划：两个客户端同时改 position + `If-Match`；无 ETag 是否必须降级只读；id 在跨 view 移动后是否稳定；父任务能否当 Project 分组；webhook 丢了靠不靠轮询 |
| Codex + GPT-5.6-sol | 实现并发 CAS 探针；Fable 先给 2 页接口，禁止它自己长出「同步框架」 |
| OpenCode + GLM-5.3 | Vikunja 原生概念 → Board / Project 分组 / Task 映射表（平价架构活） |
| Gemini | 启动、建卡、打 webhook |

关键否决点：`If-Match` 若只对 GET 缓存有效、对 sort 写入无效，adapter 不得伪造令牌——这是合同原文。失败则重开 task server，考虑 git-bug 并接受「任务 content 也在 Git」的模型例外。

AGPL：探针和以后的 control 都只能独立进程 + HTTP，不链接源码。这条写进产品化约束，不要拖到 P2。

### 轨道 C · Zellij（挡 B2）

局部、定义清楚的函数体，交给 Grok 写脚本最合适。

- attach / 输入 / resize / 重启 / 残留进程
- 无人 attach 时，TUI 的能力查询必须有应答——这是 headless 执行的前提
- `zellij-no-web` 构建重量与常驻内存
- kitty 协议矩阵：六个 harness 都是被试，不是作者。每个都要在 Zellij pane 里实际按键（含 CJK/IME）。Gemini 跑机械按键，Grok 收矩阵。

### 轨道 S · OS 沙箱（挡 B2，和 C 重叠）

这是 P0 里唯一可能改写 Agent 模块形状的项。顺序必须是 Fable 两页包装模型 → Grok 越狱清单 → Codex 只写包装器 → Gemini/Grok 跑越狱，不能让 Codex 从空白设计沙箱。

Fable 必须先回答：

- macOS 用 `sandbox-exec` / Seatbelt，Linux 用 Landlock / bwrap，各自保底是什么
- ACP 子进程、PTY、harness 自带 sandbox 三者如何 nested；套不住就标 unsupported
- git broker 与凭据网关在 P0 只需失败桩：拒绝读 `~/.hctl2/`、Keychain、SSH agent、目标 ref；P0 不实现真网关

Codex 实现 throwaway wrapper。Grok 写负例（`CT-AGENT` 已经列了：越权文件、target Git ref、control 凭据、secret store、SSH agent、未授权 provider config、网络目的地）。Gemini 先试「harness CLI 在 sandbox-exec 里能不能启动」。

做不到的候选：标 unsupported，不改合同迁就 prompt。这是人的决定。

### 轨道 D · Conductor（可晚，不挡 B2）

Gemini 烟雾通过后，Kimi 写 compose 探针，Grok 写「external task poll/complete、重启后续租」计划。不要把 Codex/Fable 耗在这里。postgres-only 若在已评估版本上必须带 ES 或 Redis 集群，按合同重开 Engine 选型，不自研第二引擎。

## 第 3 波 · 证据入库（1 天）

| 角色 | 干什么 |
| --- | --- |
| Fable | 把 RESULT 收成 evidence 条目：pinned commit、许可证、产品化约束（darwin 交叉编译、Colima、AGPL 进程隔离、`zellij-no-web`） |
| Grok | 对抗：有没有把「能启动」写成「合同成立」 |
| 人 | 合并 evidence；失败则改 decision-history；通过也不开 P1 的生命周期托管 |

机械提交（lint、PR 正文、memo 拼装）按 v0.12 纪律不要交给模型——那正是 P1 `hctl2-tool` 要收走的活。

## 六套 harness 怎么用

| Harness | 在 P0 里当主角 | 不要让它干 |
| --- | --- | --- |
| Claude / Fable-5 | 章程、Matrix/Vikunja 对齐解释、沙箱包装模型、evidence 成文 | 并发 CAS、RocksDB 编译泥潭、长测试 runner |
| Codex / GPT-5.6-sol | Vikunja 并发条件写入、沙箱 wrapper 的精细拒绝路径、Tuwunel 顺序用例里过不去的那一截 | 探针实验室架构、沙箱方案选型、把 throwaway 脚本产品化 |
| Grok / 4.6 | 每条轨道的测试计划、Zellij 探针实现、越狱清单、证据对抗审 | 从零写 homeserver 驱动、从零设计 agentd |
| OpenCode / GLM-5.3 | 映射表、RESULT 草稿、Conductor compose 文档（Fable 忙不过来时） | 拍板重开选型 |
| Kimi / K3 | Tuwunel/Conductor 的常规脚本实现（Codex 的平价替身） | 架构、对抗审 |
| Gemini flash | 烟雾、跑现成脚本、贴日志、按键矩阵 | 任何会进 evidence 的定性结论 |

协调原则三条：

1. 一条探针最多两个写作 harness + 一个对抗 harness，不要六套一起改同一目录。
2. 被试 ≠ 作者。Zellij 键盘矩阵、沙箱越狱必须用六个 harness 当负载，作者固定为 Grok/Gemini。
3. 人只在四个闸门出现：章程批准、选型重开、harness 标 unsupported、evidence 合入 main。

## 明确不要做的事

- 等 Conductor 过了再动 P1。B2 的无 Run 切片不需要它。
- 在 P0 里起 `hctl2-control` / Workbench，或把探针 crate 留进以后的 workspace。
- 把「一键启停 / 备份演练」产品化——那是该系统在 P2 首次被消费前的活。
- 用 Continuwuity 再做一轮并列选型，除非 Tuwunel 失败。
- 让 Gemini 的烟雾日志直接变成合同句子。
- 沙箱做不到就改 prompt。合同写的是 OS 强制；做不到就减候选，不减边界。
- 为 P0 再开一轮设计文档。缺口审计已经说了：优先问现有 owner 缺哪一条，不要新对象。

## 建议的第一件事

不要先 clone Tuwunel。按顺序：

1. Fable 出 `probes/README.md` + 五份假设表（含沙箱）+ RESULT 模板，时间盒写死。
2. Grok 对抗审：每条假设是否真的对应 `delivery.md` / `spec/system.md` / `CT-AGENT` 的句子，负例全不全。
3. 人批准后，Gemini 当天跑完四条烟雾，特别是 Tuwunel 在 darwin 上到底有没有二进制。
4. 烟雾结果出来，再开 Vikunja-CAS（Codex）、Zellij（Grok）和沙箱模型（Fable）三条轨。

P0 成功的标志不是仓库里多了 Rust crate，而是 evidence 里出现四段「pinned commit + 假设表 + 产品化约束」，并且沙箱对打算在 B2 使用的 harness 有一句硬的：能强制，或标 unsupported。然后才谈得上 P1 的 `hctl2-agentd` / `hctl2-tool`。

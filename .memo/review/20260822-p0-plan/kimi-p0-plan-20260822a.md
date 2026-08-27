# P0 实施计划（kimi）

> 日期：2026-08-22<br>
> 状态：Informative · P0 施工组织方案，不改规范。P0 范围与验收以 [delivery.md](../../../docs/design/delivery.md)「开工前限时验证」为准。<br>
> 对象：HCTL2 仓库 @ `main`（草案 v0.12.0）；依据 delivery.md、retrospective memo 与三份 20260821 评审 memo。

## P0 是什么

P0 的内容在 delivery.md 已钉死：对四项已拍板选型做**限时、可丢弃探针**，产物只有三样——实现证据、固定版本、产品化约束；探针脚本、临时数据与拼装环境不进产品生命周期。加上 retrospective 点出的隐含项（harness OS sandbox / credential gateway 可行性，B2 阻断），实际是五条线：

| 探针 | 阻塞点 | 核心验证内容 |
| --- | --- | --- |
| Zellij（运行时后端） | B2 前 | attach/输入/resize/重启/残留 × macOS+Linux；headless 终端查询应答；kitty keyboard 协议下各 harness 按键实测；web 客户端默认关闭（可用 `zellij-no-web` 裁剪）；打包重量/常驻内存 |
| Tuwunel（chat server） | B1 前 | 固定 release/commit + RocksDB 系 backend + build features；事务 ID 幂等、单 homeserver 线性顺序、重同步、备份恢复；macOS 无官方包需我方 CI 交叉编译 |
| Vikunja（task server） | B1 前 | 看板语义（排序令牌、泳道）、webhook/轮询观测、身份稳定性、SQLite 备份恢复 |
| Conductor（workflow engine） | B4 前 | 单机最小持久化组合（postgres-only 免 ES）、容器形态（Colima/Podman）分发/升级/备份恢复、打包重量 |
| Sandbox/credential 隔离（隐含） | B2 阻断 | macOS/Linux 上能否对 harness 强制 OS sandbox、Git broker、credential gateway；做不到就标 unsupported，不能降成 prompt 自律 |

探针不是全局 barrier：chat/task 探针 B1 首次消费前完成，runtime 探针 B2 前，engine 探针 B4 前、不阻塞 B2。失败则重开对应选型决定并修订 decision-history；降级方向已在案（tmux / Continuwuity / git-bug；engine 不自研第二套）。

## 实施节奏

**第 0 步（半天，人 + Fable）：probe 计划模板。** 把每条探针的散文描述转成可执行 checklist：固定验收项、失败判据（什么情况算"重开选型"）、证据记录格式（命令 + 输出 + 结论）、统一产物字段（pinned version / 证据 / 产品化约束 / 风险）。失败判据本质是选型决定的反悔条件，必须人参与拍板。

**第 1 步（小时级，Gemini）：快速 smoke。** 每条探针先跑最浅可行性确认：二进制能否下载/启动、端口、license、版本号。作用是在第一天暴露"明显走不通"的项，而非深挖三天后才发现。

**第 2 步（主力，并行）：**

| 探针 | Harness | 理由 |
| --- | --- | --- |
| Zellij + sandbox 隔离 | **Codex** | 实测矩阵最大且全是精细勾稽逻辑（PTY 行为、按键协议边界 case、kill -9 残留、seatbelt/seccomp 验证）；两条同属执行面 runtime，绑在一起上下文最省 |
| Tuwunel | **Kimi** | 事务 ID 幂等与线性顺序测试需要精巧并发/重试脚本，外加交叉编译——code-heavy 但边界清晰 |
| Vikunja | **GLM** | REST API 探测 + SQLite 备份，定义清楚、逻辑不重 |
| Conductor | **Grok → GLM 接力** | Grok 先做对抗性分析：列出"单机最小持久化"全部翻车路径（版本升级、PG 数据目录迁移、镜像重量）；GLM 按清单跑编排与测量。B4 前完成即可，可排第二拨 |

**贯穿全程（Grok）：对抗复核。** 每条探针出报告前，由 Grok 质疑结论——"事务 ID 幂等在网络重投下真的成立吗""排序令牌并发写真的不丢吗"。

**收尾（Fable）：证据收编。** 把探针结论规范写进 `implementation-evidence.md`；若有失败项，起草 decision-history 的选型修订。实测脚本尽量少让 Fable 碰。

## 纪律

- **探针代码放 repo 外或 untracked 目录**（如 `probes/` 入 `.gitignore`）；进 git 的只有 docs 层结论——delivery.md 明文探针产物不进产品生命周期。
- **限时是硬约束**：建议 Zellij 5 天、Tuwunel 4 天、Vikunja 3 天、Conductor 3 天封顶；超时即视为"假设无法落地"，按流程重开选型，不追加投入。
- **优先级按阻塞链**：Zellij/sandbox（B2 阻断）> Tuwunel/Vikunja（B1 阻断）> Conductor（B4 前）。开不了五条线就先开 Codex + Kimi，Conductor 放最后。

## 一个需要现在定的分歧点

Sandbox/credential 隔离是五线中唯一"做不到就要改设计承诺"的，其余四条失败只是换选型。建议它**不与 Zellij 捆绑计时**，而是 Codex 线上的独立子任务，单独出结论——它一旦翻车，影响的是整个 Worker Profile 合同（CT-AGENT 的越权负例族），不是换一个终端复用器能解决的。

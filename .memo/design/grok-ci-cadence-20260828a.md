# CI 墙钟：慢在发行整包，不在日常 Buck2

> 状态：待拍板 · C 已落地，A/B/D 仍未决定<br>
> 基线：main @ 9f7f552<br>
> 去向：`.github/workflows/{code,release,dependencies}.yml` 与 `main` 分支保护 required checks；不进合同层。<br>
> 日期：2026-08-28<br>
> 说明：Grok 按「什么检查、等多久、该不该挡合入」自分类。所有者要求先讨论、不要直接改 workflow。<br>
> 关联：[shell 与 Buck2 盘点](./grok-shell-scripts-buck2-20260828a.md)

## 怎么来的

文档 PR 已用 CI gate 跳过三平台 Buck2。之后合 `src/` 仍然极慢。核对 `main` 保护与最近 merged PR 的 check 时间后：慢的不是「每次 buck2」，是 **Release gate 把 Darwin 发行整包（含现场编 Tuwunel）放进合入关键路径**。

## 现行门禁

保护 required：`CI gate`、`Release gate`（`strict: true`）。

| Workflow | PR 何时跑重活 | 挡合入？ |
| --- | --- | --- |
| `code.yml` | path 命中 `src/` 或 `code.yml` → 三平台 Buck2 + Cargo parity | 是（CI gate） |
| `release.yml` | path 命中整个 `src/` 或 `release.yml` → 三平台 Complete package（Buck 第一方 + 外部组包 + 装两遍拼包 + 离线生命周期） | 是（Release gate） |
| `dependencies.yml` | PR 只跑静态合同；main push 或手动触发再跑三平台 Package lifecycle | 否（未进保护） |

文档 / `.memo` PR（如 #27）：两 gate 均 skip 重活，约十几秒。这部分已经解决。

## 实测墙钟（merged PR）

第一方 Buck2 本身不贵：Linux / arm64 大约 0.5 分钟，Intel Mac 大约 1–2 分钟。#22 CI gate 合计约 1.5 分钟。

拖节奏的是发行：

| PR | 检查 | 墙钟 | 合入被挡住 |
| --- | --- | --- | --- |
| #25 官方 tmux | Complete package / macOS x86_64 | **~56 min**（01:59:46–02:55:32） | 是 |
| #25 | Package lifecycle / macOS x86_64（与上项重复编） | **~53 min** | 否，但占 runner |
| #25 | Complete package / macOS arm64 | ~25 min | 是（并行） |
| #25 | CI gate | ~2 min（02:01:17 已绿） | 后面 50 min 只等 Release |
| #23 发行组装 | Complete package / macOS x86_64 | **~49 min** | 是 |
| #23 | CI gate | ~1 min | 同上 |

Linux Complete package 约 2–3 分钟。Intel Mac 贵在起 `macos-15-intel` + Darwin 现场 cargo 编 Tuwunel，不是 Buck2 第一方。

## 分类：四类等待

1. **该挡合入、也很快**：`src/` 上的 Buck2 三平台 + Cargo parity（CI gate）。
2. **该在 tag/`main` 跑、不该挡日常 PR**：Darwin Complete package / Package lifecycle。
3. **path 过宽导致误跑**：Release 把整个 `src/` 当发行路径，改 `agentd` 一行也打完整离线包。
4. **已消除的重复劳动**：packaging PR 上不再由 `dependencies.yml` 与 `release.yml` 各编一轮 Darwin Tuwunel；前者在 PR 只跑静态合同。

## 可选办法（未改仓库）

**A. 收窄 Release 的 PR 路径。**  
`src/` → `src/packaging/**` 与 `release.yml`。第一方代码 PR 只走 CI gate（约 2 分钟）。装包合同要等 packaging PR 或 tag。

**B. PR 上不跑 Darwin 整包。**  
PR 只打 Linux Complete package（2–3 分钟）；`macos-x86_64` / `macos-arm64` 放到 push `main` 或 `v*` tag。Darwin 回归晚一个合入。

**C. PR 上不要打两遍依赖包。——已落地**

`dependencies.yml` 的 Package lifecycle 在 PR 只留静态合同（`bash -n`）；整包生命周期只走 Release，main push 或手动触发再独立复验依赖包。

**D. 保护里拿掉 Release gate。**  
只 required `CI gate`。发行 workflow 照跑、失败可见，不挡合入。合得最快，发行问题可能进 `main`。

原建议组合为 **A+B+C**；其中 C 已完成。A/B 仍决定日常 PR 是否继续等待 Darwin 整包，D 更激进。

## 待拍板

只需继续决定 A/B/D。C 已随外部子系统进入 Buck action graph 的实现落地。

## 2026-08-29 缓存实测更新

上述 25–56 分钟是没有跨 runner action cache 时的冷种子数据，不再代表每次正常构建。PR #31 合并后的第二次 main 手动复验中，macOS arm64 完整依赖生命周期为 36 秒，Intel 为 1 分 15 秒；两个 job 都精确恢复 REAPI 数据，Buck 汇总 88% cache hit、7 个 cached action 和 1 个本地测试 action。后续又把 Tuwunel 拆成独立 action，并要求精确外层 cache 命中时在 event log 中确认该 action 确实 remote-hit。

这改善了 B/D 的等待成本，但不改变职责判断：发行门禁是否阻塞普通第一方 PR 仍应按风险策略单独拍板，不能以“已经有 cache”假定永远不会冷建。

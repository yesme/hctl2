# HCTL2 多机多现场案例研讨与架构反馈

> 状态：已拍板 · 选型立项<br>
> 基线：main @ 6a49d52（草案 v0.17.0）<br>
> 去向：待 P2/P3 外部供应端扩展时落实；与 `.memo/notes/HCTL_case_study.md` 配套<br>
> 日期：2026-09-06<br>
> 说明：Informative · 针对用户多机多现场案例（`HCTL_case_study.md`）的架构裁决、拓扑分析，以及所有者选型定界记录。

---

## 一、 同层裁决与世界观转变

在多机开发场景下，开发者曾直觉地希望“在 Mac 上执行到一半的物理终端会话，能被 Ubuntu 上的 Harness 跨机无缝接管”。案例明确纠正了该直觉，并确立了符合 v0.17.0 规范的架构判断：

1. **会话是易失的物理消耗品**：物理 Harness 进程和 PTY 随时可能因网络或宿主异常而丢失（不可证状态统称丢失，不叫中断），跨机维护透明进程会话迁移在系统级极度脆弱，成本高昂且收益低；
2. **跨机流转与传承的是“结晶与契约”，不是“会话上下文”**：上一步执行在 Git 中封存为不可变提交（ChangeSet Revision），并在账本中留下结构化提案（Result Proposal）或评审裁决（Verdict）；下一个接力者拿到的仅是纯净的开工包（Context Bundle），在当地重新拉起干净的执行环境施工。

---

## 二、 核心议题裁决与所有者拍板

### 1. 一 Repo 多 Control 物化为 Project（去中心化协作模型）：成立

- **裁决**：成立。一份 Git 逻辑仓库（Repo）完全可以被多个独立机器上的 `hctl2-control` 实例挂接，并各自实例化为不同的 Project。
- **演进意义**：
  - Repo 的稳定身份写入 Git 历史（或 refs），独立于任何单一 Project 或 Control；
  - 多个 Control 实例读取同一个仓库克隆时，通过 Git 中的证据识别出这是同一个稳定 `repo_id`；
  - 各 Control 独立追踪自己的 Task、Run 和本地变更，最终以 Git Commit 和代码协作平台上的 PR 作为去中心化的对账与汇聚通道。
  - 这是 HCTL2 从“单人多机”平滑延伸到“多人团队去中心化协作”的自然架构基石。

### 2. 本机 Git 协作原语提权：所有者拍板选型【流派 A（Gitea / Forgejo）】

- **问题**：在纯本地或脱离 GitHub/GitLab 托管平台时，本地 Git 缺乏 PR、评审线程、行内评论和 Checks 等协作原语，导致本地协作能力降级。
- **所有者拍板**：采用**流派 A（轻量单二进制 Forge 随包部署）**。
- **选型与落地架构**：
  - **组件选型**：引入 Gitea 或 Forgejo（Go 编写单二进制，内嵌 SQLite，资源占用极小）；
  - **生命周期编排**：由 HCTL2 的 `process-compose` 托管启动和停止（与 Tuwunel、Vikunja、Dagu 并列）；
  - **端口契约契合**：对 HCTL2 的 Repo 模块平台适配器而言，它提供标准的 Gitea REST API、Webhook 与 PR 页面，代码可零定制复用；本地瞬间获得完整的代码评审与协作原语。

### 3. ChangeSet 与 Worktree 的本体论解耦

针对“Worktree 不就是 ChangeSet 吗”的疑问，明确两者在系统本体论中分属两层：

```
                 逻辑资产层 (Durable)            物理执行层 (Ephemeral)
               ┌───────────────────────┐       ┌───────────────────────┐
               │       ChangeSet       │       │       Worktree        │
               │  (变更集 · 治理实体)    │ ────> │  (工作树 · 物理工位)    │
               │  账本记录 / 版本演进    │       │  磁盘目录 / .git 链接   │
               └───────────────────────┘       └───────────────────────┘
                          │                               │
                          ▼                               ▼
                 多次返工 Revision 序列           用完保全并拆除 (P1 archive)
                 跨机流转、不可变结晶             断电/损坏可安全重建
```

1. **临时物理工作台 vs 永久逻辑案卷**：Worktree 是文件系统中的具体检出文件夹，属于易失、可替换、用完即拆的物理资源；ChangeSet 是控制面账本和 Git 历史中登记的逻辑变更提案案卷。
2. **一对多的版本演进**：同一个 ChangeSet 在经历多次返工演进（Revision 1 → 2 → 3）时，物理 Worktree 可以在每次封存后被安全拆除（`hctl2-tool archive remove`），并在下次返工时重新物化（`worktree materialize`）。物理目录的路径甚至操作系统都可改变，但 ChangeSet 身份保持唯一与连续。
3. **写租约（Write Lease）的独占保障**：一个逻辑 ChangeSet 在同一时刻只能被授予唯一的 Write Lease，由某一物理 Worktree 独占修改，防止多端并发冲突与数据损坏。

---

## 三、 跨机派工的现场物理约束

在多机拓扑中，跨机器向远端 Agency 派发任务具有以下前置条件：

1. **Repo Instance 挂接**：被派工的远端主机必须拥有该 Repo 的克隆现场（Repo Instance），由当地的 `hctl2-tool` 物化独立工作树；
2. **Git 对象的汇聚依赖**：不同机器上的 Git 对象库是物理隔离的。跨机派工的改动若要最终集成，必须依赖可访问的公共 Remote（即绑定了 GitHub/GitLab 等协作平台，或本地部署了随包 Gitea 后端）；纯单机离线库的派工范围严格受限于承载该 Repo Instance 的本机 Agency。

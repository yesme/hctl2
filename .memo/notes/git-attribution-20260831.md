# Git 共同作者署名在 HCTL2 中的落点

> 说明：记录 abacistopia 式 `Co-authored-by` 署名在 HCTL2 中应如何分层；这是设计讨论结论，不修改当前合同。<br>
> 基线：main @ aa19367（草案 v0.15.4）<br>
> 去向：暂不升格；需要产品化时逐条进入 Repo policy、Agent 合同与本地/远端集成命令。

## 问题

abacistopia 会保留仓库用户作为 commit 的 primary author/committer，同时把实际参与本次工作的 harness、模型和 effort 写成 `Co-authored-by` trailer。HCTL2 需要判断这件事由哪个模块拥有，以及 `hctl2-tool` 是否应负责。

## abacistopia 的实际做法

abacistopia 没有把 trailer 当成一段随手拼接的文字：

- 仓库协议规定 primary author/committer、trailer 格式和缺证据时的处理；
- `scripts/coauthor.py` 从当前 harness 的实际 session 记录读取 model 与 effort，不用启动默认值冒充本次事实；
- `commit-msg` hook 用 Git 原生 `interpret-trailers` 解析并核对精确 trailer，缺失、陈旧或证据不可读时阻止 commit；
- provider 原生签名的 hosted commit 可以保留原 author/signature，不为了统一外观改写。

这是一个仓库在没有 Participant 身份层和 HCTL 控制面账本时的合理实现：abacistopia 只能把当前 harness session 当作“是谁”的来源，再把查到的 model/effort 组成 Git 身份。HCTL2 不应照搬这种认人方式，因为 Participant 在执行开始前已经确定了稳定的逻辑身份。

## 与 abacistopia 的核心差异

abacistopia 把身份和运行方式压在一起：从 session 读取 `Codex GPT-5.6 Sol (max effort)`，这个运行标签同时充当共同作者身份。HCTL2 则把两者分开：

```text
Participant revision                  谁
Invocation / Attempt + Execution Spec 这次由谁工作
Execution Runtime / binding           这次怎样运行
ChangeSet Revision producer_ref       谁实际产出了这版变更
```

因此，HCTL2 不需要通过 session 识别作者。Participant revision 是身份来源；session、Harness、model 和 effort 只是本次执行的实现与审计事实。它们可以按 Repo policy 附在公开名称里，但不能反过来定义或替代 Participant。

这里还有一道必要条件：选中了 Participant，不表示它已经成为共同作者。只有 ChangeSet 的 `producer_ref` 及其来源链证明该 Participant 的产出进入最终变更时，它才进入共同作者候选集合。

## 分层结论

**作者身份来自 Participant，产出归属由 Agent 的 ChangeSet 证明，署名规则归 Repo，精确写入意图由 control 冻结，本地与远端执行端只负责照做。** 不新增 co-author 模块，也不让 `hctl2-tool` 自行判断作者。

| 问题 | 现有落点 | 职责 |
| --- | --- | --- |
| 这个数字参与者是谁 | Participant immutable revision | 提供稳定逻辑身份；Harness、model、session 和显示名都不能替代它 |
| 它是否实际产出了变更 | Agent 的 ChangeSet Revision `producer_ref` | 指向 human command，或精确 Invocation/Attempt；沿来源链确认哪些 Participant 的产出进入最终 ChangeSet |
| 本次实际用了什么 | Execution Runtime、Result Proposal 与实际 Harness/runtime binding | 记录能够被接入端认证的 harness、model、mode/effort 和 provider 事实；不能认证的保持 unknown |
| Git 中是否署名、公开到什么粒度 | Repo 共享 policy | 规定 primary author/committer、trailer 类型与格式、公开 Participant 还是实际 model/effort、bot 邮箱映射和缺证据策略 |
| 本地集成怎样写 | `hctl2-control` + `hctl2-tool` | control 把已解析的精确 trailers 固定进 integration intent；tool 调用 Git 原生机制执行并回读 |
| 远端 PR/squash 怎样写 | `hctl2-control` + SCM adapter | 使用与本地命令等价的已持久化 intent，由 adapter 调用远端平台并回读 |
| Harness 自己产生的中间 commit | 仓库声明式配置、versioned hook 与 CI | 属于仓库开发纪律，不进入 HCTL 第一方组件 |

Repo 而不是 Project 拥有署名规则，因为一个 Git 历史可以承载多个 Project。Project 模块中的 Participant 回答稳定的“谁”，Agent 模块中的 Execution Spec、Worker Profile、Harness 和 runtime binding 回答“这次怎样运行”，ChangeSet `producer_ref` 回答“谁的产出进入了这版变更”。Git trailer 只是这些事实按 Repo policy 生成的公开投影，不能反过来成为 HCTL 身份或权限的权威。

## 一次集成的处理顺序

1. Invocation/Attempt 已冻结精确 Participant revision；这一步确定“谁在工作”。
2. Execution Runtime 与实际 binding 记录本次 Harness、model/mode 和 provider 事实；这一步只回答“怎样运行”。
3. ChangeSet Revision 封存，并以 `producer_ref` 保存产出来源；这一步证明哪些 Participant 实际贡献了最终变更。
4. Repo policy 决定哪些实际生产者进入 Git、采用何种公开名称和邮箱，以及缺少运行细节时是否仍只用稳定 Participant 身份。
5. 预览把 primary author/committer 策略和完整 trailer 列表展示给有权 actor；提交后将精确字节固定进本地或远端 integration intent。
6. `hctl2-tool` 或 SCM adapter 使用 Git/平台原生能力执行，回读最终 commit、message、author/committer、tree 与 target head；Integration Receipt 引用回读事实。

执行端不得扫描 `~/.codex`、Claude transcript 等 provider 私有文件来重新判断 Participant，也不得从 payload、commit 现有 author 或模型自述补猜身份。接入端若能认证实际 model/effort，就把它作为运行事实提交；否则保持 unknown。是否公开这些运行细节不影响稳定 Participant 身份，也不影响已由 `producer_ref` 证明的产出归属。

## 默认语义与边界

- HCTL 的默认归因主体应是稳定 Participant；实际 model/effort 是运行 provenance。abacistopia 式“把实际模型写进 trailer 名称”可以是某个 Repo 的严格公开策略，不应成为所有仓库的全局规则。
- 同一 Participant 更换 Harness、model、effort 或 session，仍是同一作者；相同 Harness/model/session 也不能让不同 Participant 变成同一作者。
- `Co-authored-by` 只给实际进入最终 ChangeSet 的内容生产者。只提交 Verdict 的 reviewer 不自动成为共同作者；其贡献由 Gate/Verdict/Receipt 保存。reviewer 后续直接产出被采纳的代码或正文时，才按该次 producer 身份进入共同作者集合。
- producer lineage 可能包含多次返工或多位 Participant。系统不能只取最后一次 Attempt，也不能用 `git blame` 猜实质贡献；预览应列出从获准 ChangeSet lineage 得到的候选，由 Repo policy 决定是否全列并让有权 actor 确认。
- 新建 squash commit 或 merge commit 时可以自然写入 trailers。fast-forward、已有签名 commit 或 provider 原生 verified author 不应只为补署名而重写 SHA 或破坏签名；此时完整来源仍保存在 ChangeSet Revision、Result Proposal 与 Integration Receipt 中。
- Git trailer 是互操作和展示信息，不是 HCTL 权威。第三方修改或删除 trailer 不会改写 Participant、producer、Gate 或 Receipt，只在回读时形成 Git 事实差异。

## 与 v0.15.3 `hctl2-tool` 定界的关系

v0.15.3 已把普通 commit 署名从 `hctl2-tool` 的自有功能中移除，这个决定仍然正确。这里补充的区别是：

- **不归 tool**：决定谁是共同作者、读取运行身份、制定格式、选择是否阻止集成；
- **归 tool**：执行 control 已持久化的本地 Git intent，调用 `git commit` / `git interpret-trailers` 等成熟机制，并回读结果。

因此没有重新引入“署名工具箱”。它与 worktree 物化、Git 集成和结果回读一样，仍是 control 决定、现场执行者转调 Git 的关系。

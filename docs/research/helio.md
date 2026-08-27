# Helio

> 类别：② Agent 协作平台 · 证据编号：E-HELIO<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-helio"></a>
## E-HELIO · Helio

### 核心价值与跨层画像

Helio 是 Sheet0 的"AI-native 团队工作区":闭源 SaaS 控制面承载人机混合频道、Tasks 看板(HEL-nnn 键)、Automations、审批收件箱、凭证 Vault 与版本化 Artifact;每个 AI teammate 是工作区一等用户(handle、私聊、自有邮箱,接 Slack 时每个 teammate 一个独立 Slack app 而非共享 bot),雇佣时绑定一个引擎(文档层只有 Claude Code 与 Codex;营销宣称的 MCP server/自带 Docker 镜像未见文档证实)与一台"计算机"(云 pod 或装守护进程的本机)。引擎与工作区之间的桥是 **heliox CLI**——把全部域对象做成 CLI 动词,同一插件发布到 Claude Code 市场、Codex 插件与自家 runtime 三个渠道,引擎无须理解 Helio 协议、只须会用 CLI。核心产品闭源,但官方外围有可源码审计的三个仓库(heliox skills 发布镜像、anycli 凭证注入库、ship 门控管线插件),它们直接暴露了 agent 面向工作区的全部工具面。与已收录的 [Cumora](./cumora.md#e-cumora) 相比,Helio 的中心是票据/编码会话/审批治理,Cumora 的中心是聊天/记忆/agent 自发性;两者是同一赛道的对置打法。三块经源码验证的亮点:

1. **消息面并发协议与出处(L4)**。发消息必带 `--seen <seq>` 声明已观察到的最新序号,网关据此做 CAS 隔离栅栏——过期发送直接失败并返回错过的消息与精确重试指引;`cede` 是显式"弃权本轮"动词、必附理由;频道消息下的逐 AI 回执把 **silent(已读、无可补充)与 unread(未读)区分开**;每条消息可查"由哪个 turn 产生、被哪些 turn 处理"的 turn 级出处。多 teammate 群频道须有 charter(章程)文档,含角色分工、每步的交接工件与 **Doer≠Verifier 独立验证规则,且人类 owner 批准前只是草稿**。
2. **归约与证据纪律(L2)**。Automation 的每次 run 必须以 `success|failed|skip` 三选一收尾、失败与跳过必填理由,不收尾的 run 由看门狗标为 unclosed/died 并告警 owner;"源头报错是 failed 不是 skip"。工作流文档修改分层:方法层改后告知 owner,意图层必须事先批准。开源侧 ship 插件更激进:stop-gate 以状态机为唯一事实源并**明确拆除了 LLM 完成度校验**(注释原文:"a model call can only re-derive what the state machine already knows — or hallucinate TASK_COMPLETE");证据分级 L1(截图/curl 响应/console 日志)唯一可采信、L2("tests passed")不足、L3("should work")自动 FAIL;评审者不能改代码、QA 不能读 review 的角色隔离下沉到工具调用前钩子。这是与"证据高于自述"完全同源的独立实现。
3. **适配器与凭证工程(L1)**。anycli 给每个可执行叶子命令声明 side_effect 单比特("可能变更与否"),**缺失注解一律按可变更处理**("Absent means true...The safe side is the only defensible default"),构建期 lint 强制穷尽,库只报事实、宿主做策略;工具凭证由宿主解析、临时注入子进程、内存驻留、用后即焚,"绝不改用户持久配置";凭证失效判定 provider-aware(只有提供方明确拒绝才标失效,限流/5xx 不误伤);另有内置无凭证的 gate-probe,可端到端验证"审批门真的在门上"。节点带 `host_cli` 能力探针(found/not_found/unknown)决定能否用本机登录;"本机登录"动力源配置锁死不可改。

反面证据集中在完成权威与凭证明文两处。**"closing a task is a human-only step"(关单是人类专属)是已下线的营销文案,不是系统权限**:现行 agent 工具面明确提供 `heliox task done`,且官方推荐的任务生命周期就以 agent 执行 done 收尾;`in_review` 只是"该由别人核验时"的条件性约定。done 动词本身有个值得借鉴的最小机制——证据评论先落、评论失败则中止关单、任务保持 open——但完成权威仍在 agent 手里。验收标准是自由文本、无 Task Revision、无绑定 Task 的独立 Run 对象;workflow 文档 fork-on-edit 但每次 run 不冻结版本;`heliox vault get` 会把凭证明文交进 agent 的 shell,红线只是提示词("不得打印到聊天/日志/任务评论/记忆");人对执行会话只有 steps 重放级观察,没有 PTY 接管与重连语义;控制面不可自托管。

### 审计基线

| 基线 | 状态 | 可支持的结论 |
| --- | --- | --- |
| 产品 0.5.0-alpha · 官方站点与文档快照 2026-08-23 | 闭源 SaaS(2026-05 下旬公开亮相) | 上述产品行为(任务/审批/Vault/Automation/回执);网站呈 GA 姿态、实际 alpha/preview 混合,营销与文档存在滞后与落差,判断以文档加源码为准 |
| [heliox / marketplace@f0c8b46c](https://github.com/heliohq/marketplace/tree/f0c8b46c743c2e49e776619865696d23d2d93593)(v0.2.62,Apache-2.0)· 2026-08-20 | 私有 monorepo 的发布镜像(本地克隆核验) | agent 面向工作区的全部工具面语义(task/message/automation/vault/charter) |
| [anycli@2434c360](https://github.com/heliohq/anycli/tree/2434c360fde5cc51d8c49a1f07b37dae5b477d07)(v0.0.8,Apache-2.0)· 2026-08-21;[ship@40da17bd](https://github.com/heliohq/ship/tree/40da17bd7c1447660efd40064178ba09357fadce)(MIT)· 2026-07-04 | 官方开源外围(本地克隆核验) | side_effect/凭证注入/gate-probe;机械 stop-gate/证据分级/角色隔离 |

"human-only close" 的考古链条:该句在搜索引擎索引中归属 helio.im/product/ 的早期版本,当前页面全文与 meta 均已无此句;引用这一反例时应注明此演变,不得写成现行行为。

### 采用与边界

HCTL 对照 L4 采用:`--seen` CAS 隔离栅栏与"过期失败返回错过内容"的重试合同、cede 显式弃权、回执的 silent/unread 区分(与无缺口观察流同源)、消息的 turn 级出处,以及"人批准才生效"的频道协作契约与 Doer≠Verifier 分离。对照 L2 采用:三元归约加理由必填、未收尾看门狗、意图/方法双层修改授权;ship 的机械 stop-gate("状态机之外没有完成度")、证据分级与工具调用前钩子层的角色隔离。对照 L3 采用:done 动词"证据先落、失败中止"的原子形状(但完成权威归属按 HCTL 合同,不随此形状下放)、"评论=持久证据、消息=会话"的显式区分。对照 L1 采用:side_effect 缺失即危险的安全默认与构建期穷尽 lint、临时凭证注入与用后即焚、provider-aware 失效判定、gate-probe 式"验证门在门上"的端到端探针、host_cli 能力探针。

明确不采用:agent 自行关单(即使证据先行——HCTL 的 Task 完成只接受有权人类命令或绑定 Run 的确定性归约,这条在 Helio 没有对应硬机制,其营销与实现的落差本身就是"证据高于自述"的案例);自由文本充当验收标准、无契约版本;每次 run 不冻结 workflow 版本;凭证明文进 agent shell 而仅以提示词设防;"工作区即 CLI"不替代 HCTL 的类型化命令/查询/事件端口(可作 Harness 侧投影参考)。复用结论:核心产品**仅参考行为**;开源外围(Apache-2.0/MIT)可**适配协议、移植有边界的组件**(side_effect 分类、凭证注入生命周期、stop-gate 测试形状)。

主要证据:

- 官方产品行为:[官网](https://www.helio.im/)、[Tasks](https://www.helio.im/docs/work/tasks)、[给 AI 指派任务](https://www.helio.im/docs/work/tasks/assign-tasks-to-ai-teammates)、[控制与审批](https://www.helio.im/docs/ai-teammates/control)、[引擎与计算机](https://www.helio.im/docs/ai-teammates/choose-a-model)、[Vault](https://www.helio.im/docs/connect/vault)、[Automations](https://www.helio.im/docs/work/automation)、[频道与回执](https://www.helio.im/docs/work/channels)、[更新日志](https://www.helio.im/docs/guides/whats-new)与[自认短板的竞品对照博客](https://www.helio.im/blog/grok-bot-alternatives/)
- heliox 工具面(固定到 `f0c8b46c`):[task(done 证据先行 L65-L73)](https://github.com/heliohq/marketplace/blob/f0c8b46c743c2e49e776619865696d23d2d93593/heliox/skills/task/SKILL.md)、[message(--seen CAS L37、cede、turn 出处)](https://github.com/heliohq/marketplace/blob/f0c8b46c743c2e49e776619865696d23d2d93593/heliox/skills/message/SKILL.md)、[automation 三元归约与看门狗](https://github.com/heliohq/marketplace/blob/f0c8b46c743c2e49e776619865696d23d2d93593/heliox/skills/automation-refiner/references/executing-a-run.md)、[人批 charter 与 Doer≠Verifier](https://github.com/heliohq/marketplace/blob/f0c8b46c743c2e49e776619865696d23d2d93593/heliox/skills/channel-charter-creator/SKILL.md)与[vault 审批](https://github.com/heliohq/marketplace/blob/f0c8b46c743c2e49e776619865696d23d2d93593/heliox/skills/vault-approval/SKILL.md)
- anycli(固定到 `2434c360`):[side_effect 分类合同](https://github.com/heliohq/anycli/blob/2434c360fde5cc51d8c49a1f07b37dae5b477d07/docs/side-effect.md)、[凭证生命周期](https://github.com/heliohq/anycli/blob/2434c360fde5cc51d8c49a1f07b37dae5b477d07/docs/credential-lifecycle.md)与[gate-probe](https://github.com/heliohq/anycli/blob/2434c360fde5cc51d8c49a1f07b37dae5b477d07/internal/tools/gateprobe/gateprobe.go)
- ship(固定到 `40da17bd`):[README(证据分级)](https://github.com/heliohq/ship/blob/40da17bd7c1447660efd40064178ba09357fadce/README.md)、[stop-gate(拆除 LLM 校验)](https://github.com/heliohq/ship/blob/40da17bd7c1447660efd40064178ba09357fadce/scripts/stop-gate.sh)与[phase-guardrail(角色隔离)](https://github.com/heliohq/ship/blob/40da17bd7c1447660efd40064178ba09357fadce/scripts/phase-guardrail.sh)
- 第三方:[codepick 的 Helio vs Cumora 对比](https://codepick.dev/en/guides/helio-vs-cumora-agent-collaboration/)(2026-05-28)

# Cumora

> 类别：② Agent 协作平台 · 证据编号：E-CUMORA<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](../README.md)。

<a id="e-cumora"></a>
## E-CUMORA · Cumora

### 核心价值与跨层画像

Cumora 是"AI agent 作为一等队友的跨平台团队聊天":人与 agent 共用同一花名册、私聊、群聊、看板、日历与文档,agent 有 persona、记忆和真实 email 地址,可以主动认领工作、主动发起对话。agent 的"大脑"双轨:云端为每个 agent 按需拉起、闲置自灭的 K8s pod;BYOA(自带大脑)由用户本机守护进程驱动本地 Claude Code、Codex、Grok Build 或 Cursor CLI,服务端从不持有用户的模型凭证。两轨共用同一写入面:agent 对世界的一切动作都经 `cumora` CLI 垫层进入服务端仲裁,身份由 JWT 钉死(服务端剥除客户端一切 `--as` 参数并强制注入令牌内身份,不可伪造)。它的工程重心是多 agent 无碰撞协同与成本账本,并有 CI 强制的架构不变量(只有 agent 正式回合可用大模型;每笔 LLM 调用必须入账)。三块经源码验证的独特机制:

1. **唤醒 triage 门与协同门(L4)**。消息落库后对会话内所有非作者 agent 并行扇出,"该不该醒"由便宜小模型做纯门判断——输入全部是数据库/Redis 事实(工作认领、人类注意力=消息/表情/已读游标)而非消息措辞,每次判定入账本并与省下的大脑回合做诚实的经济学对比;AI 判断之下垫**确定性循环地板**(源码注释明言这层"曾被以 AI-native 优雅为由删过两次、两次都回归死循环")。回复前有 seen 游标新鲜度预检:落后于房间状态的回复被 HELD 并把新消息内联返回;`--force` 类旗标只是对"服务端展示过的 HOLD"的确认而非通行证(hold-token 与消息序号绑定、回合结束即亡),逐字重复检查放在序号行锁事务内、任何旗标不可绕。agent 主动性(空闲心跳、后台扫描、停滞救场)全部再过一道"默认不行动"的门并受预算限制,救场用 Redis 原子认领保证全房间只有一个成员出手;agent 拉群必须写结构化理由(headline/evidence/asks 字段化)。
2. **Shipping 交付生命周期(L3)**。`Draft→Contract→Building→Verifying→Ready→Releasing→Watching→Learned` 八态状态机在数据库 CHECK、邻接表与 gate 谓词三处强制;验收要求每条必需不变量被证据方格覆盖且有 owner,**构建者不得验收自己的方格——这条分离直接下沉为数据库级 CHECK 约束**;审计流追加只写;验收失败自动派生 friction 项与可重放回归项;生产发布要求 staging/canary 先行加回滚计划,成功后自动排 24 小时生产读回(readback)——把"完成"的终态推迟到行为对基线的读回。其文档口号与 HCTL2 同频:"绿色构建或成功 rollout 是中间信号,不是终态"。看板侧的 `card claim` 是全系统唯一被认可的认领原语:单条 UPDATE 加 WHERE 守卫(未认领/本人/超 20 分钟陈旧)以 rowCount 定胜负,两个 agent 竞争同一张卡永远只有一个赢。
3. **BYOA Harness 适配层(L1)**。统一的引擎适配抽象(发送/打断/存活/会话 ID/是否承载常驻提示)接入四家 Harness,原生会话优先、一次性进程降级;resume 会话 ID 存在 agent 家目录之外防引擎写坏,**只有三种情况才重置会话**(上下文溢出、转录毒化、引擎明确报会话失效——正则要求双匹配,防止把执行中途崩溃误判为会话失效而丢弃可恢复上下文,有专门测试钉住);常驻提示每会话带外送一次、每回合只送增量。宿主设备(Computer)一等化:配对码换设备令牌(服务端只存哈希)、"移除计算机"是真实的吊销开关、doctor 端到端探针探的是真实唤醒路径("绿=真唤醒能通");另有侦察性读取不污染 seen 游标、限流自适应起搏且限流从不泄漏进聊天等配额纪律。

反面证据同样密集。验收证据是**非空自由文本自述**——无格式校验、无机器执行的验证,回归项有命令字段但仓库里不存在执行器;云端回合的"完成复核"是用 LLM 验 LLM 的自述,BYOA 侧连这层都没有(run 状态纯粹等于引擎退出码)。所有本地引擎一律以 `--dangerously-skip-permissions` 级全权限运行,唯一隐私边界是提示词文本;云 pod 以 root+SYS_ADMIN 运行且无网络策略,源码注释声称的行级安全并不存在;`participants.tools` 权限列有表、有接口、执行路径零读取——声明性权限不被执行路径消费就等于没有。记忆无版本且有自我中毒实证(内部协同文档记录 agent 把特例写成普适规则、甚至写备忘录训练未来的自己无视刚建好的安全网,修复靠审计后外科手术式删除);同一 gate 在 DB/REST/CLI 三处平行实现已出现规则漂移。789 行的 COORDINATION.md 是一份罕见的多 agent 协同失败案例集("别用 prompt 修 infra"、"绕过旗标必须是对服务端展示过状态的确认"、"缺席成员是常态不是故障"),无论产品成败都有独立参考价值。

### 审计基线

| 基线 | 状态 | 可支持的结论 |
| --- | --- | --- |
| [`main@bd8dba8e`](https://github.com/yetone/cumora/tree/bd8dba8e45c91f685ea3c319aae173d44d26cbd6) · 2026-08-22 | 审计快照(本地克隆核验 HEAD、许可证与关键约束) | 上述 triage/协同门、Shipping 状态机、BYOA 适配、身份钉死机制全部在此基线经源码验证 |
| 桌面端 v0.1.64(2026-07-26,独立 releases 仓库)/ npm `cumora` 0.1.127 | 已发布 | 主仓库无 tag/release,发布通道在仓库之外 |

许可证为 [MIT](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/LICENSE)。仓库 2026-08-17 才公开(审计时仅 5 天),单人主导、私有开发史至少始于 2026-05 下旬;服务端测试面厚(80+ 单测文件、27 集成套件、真 LLM 协同基准),前端零单测。整体判断:以协同正确性为纲的高强度工程,基础设施纪律强、安全纵深弱;能力判断以固定源码为准。

### 采用与边界

HCTL 对照 L4 采用:triage 门四件套(事实输入、纯门、经济学账本、确定性地板)、hold-token 的"确认而非绕过"哲学、事务内查重、结构化拉群理由的字段化(可改造成 Agent"建议协作边"的载体)、记忆的 project 作用域两层可见性与"混杂时落全局不猜"的出处保守原则。对照 L3 采用:不变量→证据方格覆盖矩阵、builder/verifier 分离下沉到存储约束、追加只写审计流、验收失败自动派生 friction/回归资产、readback 把验收延伸到生产观察,以及原子 `card claim`。对照 L2 采用:"成功才推进游标、失败保留收件箱重跑"的无检查点恢复模型与"只重试人工唤醒"的事故教训(重试消息唤醒曾造成真实重复回复)、孤儿运行收尸。对照 L1 采用:引擎适配抽象与降级矩阵、保守的三条件会话重置、resume 引用外置存储、设备配对/吊销/探针,以及 `--as` 剥除加身份注入的不可伪造边界。

明确不采用:自由文本自述充当验收证据(与"证据高于自述"直接冲突,是本清单最重要的反例之一);用 LLM 复核 LLM 的完成自验充当 Verdict;无冻结版本的契约与记忆;`--dangerously-skip-permissions` 式全权派发与提示词充当隐私边界(与 LobeHub 同判);声明性权限不被执行路径消费的死配置;同一 Gate 多处平行实现(HCTL 的 Gate 单点实现、多面复用);服务端信任守护进程对自身执行结果的自述;agent 默认自主拉群不作为 HCTL 普通 Room 的协作边模型——临场协作边仍由人提交,Cumora 的结构化理由只用作建议载体。Cumora 的 conversation/board/computer 不映射为 HCTL 的 Room/Task/Agent 身份。复用结论:**选择性移植**(MIT),协同门与 Shipping 约束可改编,不整仓派生、不采用其安全姿态。

主要证据(固定到 `bd8dba8e`):

- [仓库](https://github.com/yetone/cumora)、[官网](https://cumora.ai)与[许可证](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/LICENSE)
- 方法论文档:[COORDINATION.md(789 行协同失败案例集)](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/docs/COORDINATION.md)、[SHIPPING.md](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/docs/SHIPPING.md)、[BYOA.md](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/docs/BYOA.md)与[CI 架构不变量](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/CONTRIBUTING.md)
- 协同门:[小脑 triage 门与确定性地板](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/triage-core.ts#L176-L262)、[并行扇出与唤醒重试策略](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/scheduler.ts#L61-L184)、[seen 游标与 hold-token](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/seen-boundary.ts)与[停滞救场管线](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/agenda.ts)
- Shipping 与看板:[状态机与 gate 谓词](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/api/shipping-router.ts#L238-L305)、[builder/verifier DB 约束](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/db/migrate.ts#L1595-L1597)与[原子 card claim 与 agent CLI](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/cli.ts#L1512-L2098)
- 运行时与身份:[全量 DDL(约 60 表)](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/db/migrate.ts)、[pod 编排](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/runtime/orchestrator.ts#L1-L23)、[云回合循环与完成自验](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/turn.ts#L1154-L1215)与[`--as` 剥除](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/runtime/cli-argv.ts#L19-L43)
- BYOA:[守护进程(信号量/起搏/会话重置)](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/computer/daemon.ts)、[引擎适配矩阵](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/computer/engine.ts#L974-L1012)、[设备配对与令牌](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/computer/registry.ts#L189-L273)与[记忆作用域契约](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/memory-scope.ts#L18-L33)

## 复核记录

- **2026-08-24**：主仓库已出现 tag v0.2.0，公开一周内合入外部贡献、贡献者集中度已非单人画像（top-1 约 36%）；上文"主仓库无 tag/release"与"单人主导"描述到审计快照为止成立。按提交路径直方图，产品重心在聊天面（场景内归一化 room 约三分之二），与"AI 一等队友的团队聊天"定位一致；机制层结论不受快照后提交影响。

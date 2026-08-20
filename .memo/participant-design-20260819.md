# Participant 配置设计 Memo

> 状态：Informative · 讨论稿 · 2026-08-19<br>
> 目的：把 Participant 从“自动探索到一个本地 Harness”扩展成可配置、可解析、可冻结、可审计的数字参与者。本文不改变现行规范，不创建第五模块，也不提前定义 marketplace。<br>
> 术语说明：成稿于 v0.9.1 概念归并前，文中 InvocationBinding / AttemptSpec 已合并为 ExecutionSpec（见[归并对照](../docs/design/spec/README.md#v091-归并对照)）。

## 1. 核心判断

Participant 配置不是一份巨大的 agent YAML，也不是 Harness discovery 的别名。它需要把生命周期不同的七件事分开：

1. **Participant**：长期可寻址的逻辑参与者是谁；
2. **ProjectRoleBinding**：它在某个 Project 中承担什么职责；
3. **persona profile**：它以什么人设、语言和交互风格出现；
4. **post-train / model artifact**：哪些训练结果和模型制品形成行为基础；
5. **Skill**：本次工作明确装载哪些可版本化方法；
6. **WorkerProfile**：模型、模式、环境和候选端口怎样组成执行配方；
7. **Harness / ResolvedPortBinding**：这一次最终由哪个已探测、已授权的物理端口执行。

```text
Participant ref
  + Project role ref
  + persona/model/Skill refs
  + WorkerProfile candidates
          |
          v  Preview + policy + capability probe
  resolved fields（不是新领域对象）
          |
          v  freeze
  InvocationBinding / Run Manifest / AttemptSpec
          |
          v
  exact Harness + Runtime binding + generation
```

稳定的是 Participant，变化的是它被实现的方式。新进程、provider session 或备用 Attempt 不自动产生新 Participant；换成另一个 Participant 也不能靠复用 WorkerProfile 伪装成同一个人。

## 2. 目标与非目标

### 目标

- 让用户按“我要让谁参与”配置，而不只是按“机器上装了哪个 CLI”配置；
- 表达特殊 post-train、Skill 和可移植 persona，并保留来源与精确版本；
- 允许同一 Participant 有本地、远程、结构化 API、PTY 等多个实现候选；
- 调用前展示真正会运行的身份、模型、Skill、Harness、权限、成本和降级；
- 运行中冻结身份链，使结果归属和 reviewer 独立性可追溯；
- 为未来发现和雇佣远程数字员工留出协议边界。

### 非目标

- Participant 不拥有 Task、Run、Gate 或完成权；
- persona 不是安全策略，Skill 不是权限，能力广告不是能力证明；
- AIEOS ID、Chat 账号、模型名、Harness session 都不替代 HCTL Participant ID；
- 第一阶段不建设公开 registry、声誉、支付或任意第三方插件市场；
- 本 memo 不决定最终表结构、文件格式和商业对象。

## 3. 分层与归属

| 层 | 形态 | 回答的问题 | 明确不回答 |
| --- | --- | --- | --- |
| Participant | Project 拥有的稳定身份及不可变配置 revision | 谁参与、结果归于谁 | 怎样运行、拥有什么权限 |
| ProjectRoleBinding | Project 内的角色绑定记录 | 在此 Project 中承担什么职责 | 全局身份、人设、物理能力 |
| Persona profile | Participant 配置引用的不可变描述制品 | 如何表达与协作 | 能否完成、能否访问资源 |
| Model/post-train | WorkerProfile 引用的内容寻址制品或服务 ref | 行为能力来自哪个模型谱系 | Participant 身份和权限 |
| Skill | 带 revision/digest 的方法包 | 本次装载哪份知识或过程 | 自动授权工具、自动获得票权 |
| WorkerProfile | Harness 模块的可复用执行配方 | 模型、模式、环境、候选端口如何组合 | 此刻哪台机器可运行 |
| ResolvedPortBinding | 系统已有的一次端口选择 | 此次落到哪个版本、安装和能力 | 上层角色、验收或完成 |
| Invocation/Run binding | 已有执行对象中的冻结字段 | 本次实际采用哪些精确 refs | 后续 current pointer |

只有 Participant 是这里需要补足的稳定领域身份。persona、model/post-train 和解析结果首先作为不可变制品、引用或 binding 字段；除非实现证明它们需要独立命令、生命周期和恢复边界，不新增聚合。

## 4. Participant：稳定逻辑身份

建议最小配置包含：

```text
participant_id
participant_kind                 # human | agent；是否需要 service 待定
revision_id / parent_revision_id?
display_name / addressable aliases
operator_principal_ref?
external identity claims[]
persona artifact ref?
skill claims[]                   # 声明，不是装载或授权
allowed WorkerProfile refs[]
status                           # active | suspended | retired
provenance / issuer / signature?
revision_digest
```

关键边界：

- 改名、更新 persona、增加候选 WorkerProfile 产生新 revision，不改活动调用；
- 复制同一 persona 创建并行数字员工时使用新 Participant ID，并保存来源；
- 外部 registry ID 只绑定到 Participant，不直接成为 HCTL ID；
- Participant 与 command actor 分开。actor/provenance 由认证入口赋予，模型不能在 payload 中自报为某位 Participant 或 human；
- Participant 可以是 mention 目标、Seat 参与者或 result producer，但这不授予 Task terminal、Gate、secret、Git 或外部写权限。

### 身份连续性

通常属于同一 Participant 的新 revision：名称/persona 更新、经验证的 key rotation、增加能力证据，以及在预先声明等价的 realization 间切换。

通常应创建新 Participant：控制权无法证明连续、同一 persona 被复制为多个员工、主体拆分出独立责任与结算，或仅因输出风格相似而合并不同 provider agent。

## 5. ProjectRoleBinding：职责绑定，不是权限包

ProjectRoleBinding 继续是 Project 现有绑定记录；这里不再为它发明独立聚合。它至少引用：

```text
project/version + role ref
participant ID + exact Participant revision
responsibility scope
allowed WorkerProfile constraints
permission/budget policy refs or ceilings
separation attributes
binding digest/version
```

角色描述可以表达人类可读的专业期望，但 RoleBinding 不保存或激活 Skill。某次调用真正的 required/optional Skills 由 Invocation/Run 合同与 policy 在 Preview 中决定；Participant 的 skill claim 也不能自动激活 Skill。RoleBinding 不携带自己的 delegation authority，命令权限和委派仍由系统统一 actor/permission 合同处理。

关系保持多对多：一个 Participant 可在不同 Project 担任不同角色；同一角色可用新 binding 换人；一个 Participant 可有多个 WorkerProfile；通用 WorkerProfile 可被多个 Participant 复用，但结果仍归于冻结的 Participant。

### Reviewer 独立性

不同 Participant ID 不必然代表完整独立性。未来 Gate policy 可能分别检查：

- logical Participant；
- operator principal；
- model/post-train lineage；
- execution provider/trust domain；

第一阶段可以只实现 logical Participant 分离，但应把 operator/model/provider refs 留在冻结链中，不能把逻辑分离描述成组织或模型独立。

## 6. Persona：描述“像谁”，不描述“能做什么”

persona artifact 适合承载名称、简介、语言、沟通风格、协作习惯、表达禁忌，以及来源、schema、签名和 digest。它是 Participant 配置引用的不可变内容，不需要自己的 HCTL lifecycle。

它不承载：

- permission、secret、Gate、Task 完成权；
- 未验证 Skill、能力证明、Harness endpoint 或 session；
- 动态 Room/Project/Run Context；
- 可以覆盖系统政策的 prompt 指令。

### AIEOS 映射边界

[AIEOS 官方页面](https://aieos.org/)在 2026-08-19 展示的 v1.2 schema 同时包含 identity、psychology、linguistics、history、interests、motivations、capabilities、presence 和 settlement；其[注册说明](https://aieos.org/register)使用 public key、signature 与 registry identity。HCTL2 不应整体照单全收：

| AIEOS 内容 | HCTL2 建议处理 |
| --- | --- |
| identity / psychology / linguistics / history / interests / motivations | 作为 persona 来源；白名单导入、去敏并固定 digest |
| entity ID、公钥与签名 | external identity claim 和 provenance；不替代 Participant ID，不自动授信 |
| capabilities.skills | 广告声明；须映射精确 Skill revision 和独立 Evidence |
| presence / webhook / social handle | 发现提示；经认证、policy 与探测后才可形成 ResolvedPortBinding |
| settlement wallet | 未来商业 profile 的公开声明；不构成付款授权，私钥不导入 |

AIEOS 帮助人设跨模型迁移；责任连续性仍由 Participant ID、revision、actor provenance 和冻结执行链保证。

### Persona 编译

persona 不能直接无条件拼入 prompt。实现可以按目标 provider/model 和一个固定 compiler 产生 `compiled artifact + digest + warnings`，但它只是派生制品。优先级固定为：HCTL 安全/权限 > Project/Task/Run 合同 > 获准 Skill/工具约束 > persona 风格。冲突或 prompt injection 必须拒绝、删减并在 Preview 中显示。

## 7. Model/post-train 与 Skill

### Model/post-train ref

特殊 post-train 不是 Skill 或 persona。WorkerProfile 应引用一份内容寻址 manifest，至少描述 base model、fine-tune/adapter 谱系、tokenizer/chat template、制品 digest 或 hosted snapshot、许可证、兼容 runtime 和 eval Evidence。

本地权重放制品库，Git 只保存 locator/digest。Hosted model 无法取得权重 digest 时，冻结 provider、account/region、精确 snapshot（若有）、实际 binding 与漂移策略；只有 mutable alias 时不能声称可复现，受治理 Run 应拒绝 pin 不足或显式显示降级。

模型 eval 只证明给定版本和条件下的结果，不自动变成 Participant 信任、Skill、权限或未来表现保证。

### Skill 三态

Skill 使用已有共享版本机制固定 manifest/instructions/assets/scripts、兼容能力、provenance/license 和 digest。必须区分：

1. **declared**：Participant、persona 或 registry 声称会；
2. **available**：精确 Skill revision 已安装、可回读、依赖满足；
3. **activated**：本次 InvocationBinding/AttemptSpec 已冻结并装载。

Skill 可以请求能力，不能自己授予能力。含脚本的 Skill 是代码供应链输入；required Skill 缺失则解析失败，optional Skill 缺失则显示降级。同名 current pointer 不能替代 Gate/Attempt 已冻结的 ref+digest。

## 8. WorkerProfile 与 Harness resolution

WorkerProfile 是 Harness 已有的可复用执行配方，不是数字员工本人。它组合：

- model/post-train ref 或服务 selector；
- Harness/Runtime/adapter 候选约束；
- 模式、context window、工具和环境要求；
- compatible Skill/capability constraints；
- permission ceiling、放置、预算和数据驻留约束；
- 预先允许的技术 fallback 范围。

一个 Participant 可以有 `local-fast`、`local-deep`、`remote-specialist` 等多个 profile；一个 profile 也可被多个 Participant 复用。profile 不保存明文 credential，也不声称某台主机当前具备能力。

HarnessDefinition、Installation、Capability、ExtensionRevision、ResolvedPortBinding、discovery/install 和 secret handling 已由 Harness/system 定义，本 memo 不重复。关键结论只有一个：自动探索本地 Harness 是解析链最后一步，只回答“这里有什么”，不能回答“谁工作、采用什么 persona/post-train/Skill、是否有权参与”。远程 endpoint 同样必须经过既有端口解析才能进入执行 binding。

## 9. 配置、Preview 与冻结

### 9.1 配置与采用

用户维护 Participant declaration，引用 persona、skill claims 和 WorkerProfile candidates；Project 通过 RoleBinding 选择精确 Participant revision 并附加角色约束。声明阶段不启动 Harness、不联网搜索、不安装制品，也不取得 secret。

采用前需要 schema/canonical digest、来源/签名/license、alias/key continuity、persona 白名单去敏，以及模型/Skill/adapter 供应链校验。外部签名只证明某 key 签过材料，不证明能力、安全或商业可信。通用 digest、install、credential 与 command 机制直接复用 system 合同。

### 9.2 Preview / Resolve

Trigger Preview 或 StartRun Preview 按固定顺序：

1. 读取精确 Project/version、RoleBinding 和 Participant revision；
2. 展开 persona、model/post-train、Skill 要求与 WorkerProfile candidates；
3. 合并 Task/Run/Invocation 的能力、数据、权限、预算和独立性约束；
4. 在获准范围内探测实际 Harness/Runtime/model endpoint；
5. 排除缺 required Skill、能力、信任、license、驻留或预算的候选；
6. 按冻结 policy 而非加载顺序选择唯一 realization；
7. 展示精确 Participant、persona、模型、Skills、WorkerProfile、bindings、权限、数据披露、成本和降级；
8. 提交后把解析字段写入已有 InvocationBinding/Run Manifest。

这里只产生 Preview 与既有 binding 字段，不新增 `ResolvedParticipant` 对象。

### 9.3 Freeze

一次 InvocationBinding 或 Run Manifest/Seat 至少冻结：

```text
Participant ID + exact revision/digest
ProjectRoleBinding ref/version/digest
persona source + compiled artifact digest?
model/post-train artifact or hosted snapshot/binding
required/activated Skill refs + digests
WorkerProfile revision/digest
Harness/Runtime/Model ResolvedPortBinding refs
capability/trust-policy decision refs
permission/data/budget/deadline scope
operator/independence attributes required by Gate
ContextManifest ref + digest
```

AttemptSpec 继承逻辑身份链，再增加 execution generation、ChangeSet/lease 和物理 adapter binding。ResultProposal 必须能沿该链回溯 Participant、实际 WorkerProfile/model/Skills 和物理 bindings，但 attribution 不授予命令权。

### 9.4 Fallback 不变量

技术候选切换只有在 Run Manifest 预先允许，并保持以下内容不变时才属于同一 Seat/Participant：

- Participant 与 Project role；
- persona/compiled digest；
- required Skills、Context 和 Gate baseline；
- permission、数据和预算边界；
- reviewer independence 所需属性。

变化越过任一项时必须重新授权或替代 Seat/Run，不能把语义换人包装成技术 fallback。current pointer 更新、安全撤销或 provider 漂移也不改写活动 binding；迟到结果仍按旧 generation 处理。

## 10. 存储、版本与安全

| 存储 | 内容 | 不是 |
| --- | --- | --- |
| Git tracked `.hctl2/` | 可共享 declaration、persona/Skill/model manifests、profile refs | session、availability、credential、唯一领域账本 |
| RepoInstance SQLite | 已采用 Participant/RoleBinding、resolution/binding refs、事件与撤销 | 权重、明文 secret |
| 用户级配置 | 本机安装目录、个人默认与 cache | 跨 RepoInstance 无锁共享的领域 current |
| 内容寻址制品库 | 模型、adapter、Skill assets、compiled artifacts | 可变 `latest` 或权限决策 |
| system secret store | provider/endpoint credential 和签名私钥 | persona、Git、Room、Context、trace |

跨 repo 使用同一数字员工时，引用带 issuer namespace 的 Participant ID，并把精确外部 revision/digest 导入各 repo；各 RepoInstance 独立采用 current 和 RoleBinding，避免一个全局可变指针静默改变所有 repo。

Participant 与所引用配置均使用 stable ID/ref + immutable revision/digest；执行只冻结精确 revision，名称、文件路径、registry alias 和 `latest` 不够。兼容性至少检查 persona compiler↔model、model↔runtime/license、Skill↔Harness/capability、profile↔host/data policy、Participant/role↔Gate independence。

persona、AIEOS profile、remote agent card 和 Skill 文本都视为不可信输入；profile 自报能力、endpoint 或 wallet 没有权限效果。敏感 persona 字段最少导入；人类 Participant 的履历需要来源许可。发给远程 Participant 的 Context 只物化获准的去敏子集；远程 success/断线/ACK 仍只是观测或 ResultProposal，不形成 HCTL Receipt。

## 11. 远程数字员工：只保留协议边界

未来商业化至少分开三类协议面：

| 协议面 | 作用 | 不能替代 |
| --- | --- | --- |
| Directory/Profile | 发现 Participant、persona、能力声明、operator 和 endpoint hints | 能力证明、可用性、RoleBinding、权限 |
| Engagement/Offer | 价格、SLA、数据/IP 条款、取消和争议约束 | 已预留执行能力、工作结果 |
| Worker Endpoint | 接受获准 Invocation/Attempt、报告事件和 ResultProposal | HCTL Task/Run/Receipt 权威 |

AIEOS 可以成为第一类 profile 来源，并携带 presence/settlement hints；它不迫使 HCTL 接受其 registry、job market 或支付模型。Offer、reservation、metering、settlement、reputation 和法务对象全部后置，等真实商业场景与受控端口需求明确后再设计，避免在 Participant 专题里重建 marketplace 状态机。

远程发现不等于信任，报价不等于预留，预留不等于执行，执行 ACK 不等于结果。未来设计仍须分别验证 identity、operator、artifact supply chain、capability Evidence、execution/data isolation 和商业记录，不压成一个“可信分数”。

## 12. Phase 1

### 必须有

- 本地稳定 Participant ID 与不可变配置 revision；
- ProjectRoleBinding 引用精确 Participant revision；
- 可选 persona 文件导入、预览、去敏和 digest；
- 特殊 model/post-train ref 或精确 hosted model binding；
- versioned Skill 的 declared/available/activated 三态；
- WorkerProfile 多候选配置和本地 Harness capability resolution；
- Trigger/Run Preview 展示完整 realization；
- InvocationBinding、Run Manifest、AttemptSpec 冻结并回溯 producer identity；
- required Skill/能力/权限不足时 fail closed；
- fallback 和 current-pointer drift 不改变活动身份链。

### 实验或后置

- AIEOS v1.2 本地 JSON import，只导入 persona 白名单和 external identity claim；
- provider-specific persona compiler 与冲突 lint；
- 基于固定 eval Evidence 的 capability preview；
- 网络自动发现、公开 registry/marketplace、远程 reservation/SLA/payment/reputation；
- 跨组织 federation、多租户和通用商业身份。

## 13. 验证用例

1. 两个 Participant 复用 profile，结果仍归于各自精确 revision；
2. Participant 换 session 不换身份，实际 binding/generation 仍可追溯；
3. AIEOS 自报 Skill、wallet 或 webhook 不获得能力、付款或网络权限；
4. persona prompt injection 被拒绝或显示删减；
5. required Skill 缺失/digest 不符时不启动；
6. profile/Participant current 更新不改变活动 Attempt；
7. mutable hosted model 漂移时拒绝或显示明确降级；
8. 同一 operator 的两个 Participant 不满足 operator-independent Gate；
9. Harness payload 自报 human/另一 Participant 时被拒绝；
10. fallback 改变身份、persona、Skill、Context、权限或独立性时须重新授权。

## 14. 开放问题

1. Participant ID 使用本地 UUID、组织 issuer namespace，还是可选外部 identity binding？
2. Participant declaration 的 repo/user scope 与跨 repo 导入合同是什么？
3. `participant_kind` 是否需要 service，怎样避免与受控端口重复？
4. persona 是 Participant 默认值还是 Project role 可覆盖；覆盖边界是什么？
5. AIEOS 默认导入哪些字段，敏感字段如何去敏？
6. hosted model 无不可变 snapshot 时，哪些 Run/Gate 必须拒绝？
7. Skill 最终由哪个既有模块拥有定义、安装与激活写入合同？
8. 哪些 WorkerProfile fallback 仍算同一 Participant realization？
9. reviewer independence 第一阶段是否只验证 Participant？
10. Participant suspension 对活动执行的隔离权限和恢复合同是什么？

## 15. 回填现行设计的最小落点

若方向确认，不新增 Participant 模块：

- `project.md`：Participant 身份、RoleBinding、persona ref 和 actor/identity 边界；
- `harness.md`：WorkerProfile 的 model/Skill refs 与 resolution；
- `run.md`：Seat 冻结 Participant、producer/reviewer 独立性和 fallback 不变量；
- `connections.md`：Project 逻辑配置到 Harness 物理 binding 的 typed handoff；
- `system.md`：共享版本、制品、ResolvedPortBinding、secret 和安全机制；
- `delivery.md`：第一阶段范围与负例。

这样既允许 Participant 具备特殊训练、Skill、人设和未来商业身份，又不把 persona、模型、Skill、Harness 与 marketplace 叠成新的超级对象。

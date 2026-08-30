# 01 · 禁令盘点（S1 / K3）

> 状态：待拍板 · 施工图 §6 S1 表 K3 线产物
> 基线：main @ 37805fa（草案 v0.15.0）
> 去向：S1 末拍板点 7（禁令保留白名单）；本文件是大修底稿，不进 docs/design/

## 口径与方法

- **范围**：`docs/design/**`、`README.md`、`docs/usage.md`（大修范围三层；`docs/research/**` 与 `.memo/**` 不在）。
- **提取**：逐行匹配 `不得|不能|禁止|不允许|永不|必须拒绝`，与 `src/build/docs/report_prohibition_density.sh` 同一正则。共 **320 次出现，落在 248 行**；本清单按行列出（一行多次出现的，次数列标出）。
- **机械覆盖列**（族级推断，非用例级）：按内容关键词映射到 delivery.md 的 CT 族（完整映射规则见文末附录）。**族级匹配只说明"该族已有相关用例"，不等于该条禁令已被某个具体用例覆盖**——S1 末拍板时需逐条核实到用例。`自身即 CT` = 该行本身是 delivery.md CT 矩阵的用例描述；`—（历史叙述）` = decision-history 行，属历史记录（C5 簇处理，不进禁令裁决）。
- **初分标签**（只按施工图 §7.2 字面初分，不下最终结论；判定顺序自上而下）：
  1. decision-history 行 → `其余（历史叙述）`；
  2. 命中 tricky 模式（`迟到|旧代次|旧结果|复活|盲重投|结果未知|竞态|不补足|接替|覆盖新|互不相同的 Participant`）→ `特别 tricky`；
  3. 命中易犯模式（`自报|自述|冒充|凭据|密钥|secret|显示名|模糊匹配|绕过|直接改|直接写|手工|明文|加密|递归委派`，均有本 repo 评审史或所有者经验背书）→ `特别容易犯`；
  4. CT 族命中 → `可机械覆盖`；
  5. 其余 → `其余`（**刻意保守：凡不确定一律落此，待 S1 末逐条裁决**）。
- 本盘点只做机械初分，不判断该不该留。

## 汇总

### 按初分标签（248 行 / 320 次）

| 初分标签 | 行数 |
| --- | --- |
| 可机械覆盖 | 177 |
| 特别容易犯 | 27 |
| 特别 tricky | 6 |
| 其余 | 17 |
| 其余（历史叙述） | 21 |

### 按文件（次数 = 禁令词出现次数）

| 文件 | 行数 | 次数 |
| --- | --- | --- |
| docs/design/delivery.md | 36 | 41 |
| docs/design/spec/agent.md | 19 | 39 |
| docs/design/spec/system.md | 28 | 32 |
| docs/design/spec/connections.md | 22 | 29 |
| docs/design/spec/project.md | 17 | 26 |
| docs/design/spec/task.md | 17 | 26 |
| docs/design/references/decision-history.md | 21 | 25 |
| docs/design/spec/run.md | 18 | 24 |
| docs/design/README.md | 10 | 12 |
| docs/design/agent.md | 9 | 9 |
| docs/design/spec/README.md | 7 | 9 |
| docs/design/architecture.md | 7 | 7 |
| docs/design/project.md | 6 | 7 |
| docs/design/run.md | 6 | 7 |
| docs/design/task.md | 5 | 7 |
| docs/design/context.md | 5 | 5 |
| docs/design/vision.md | 5 | 5 |
| docs/design/participant.md | 3 | 3 |
| docs/usage.md | 3 | 3 |
| docs/design/references/glossary.md | 2 | 2 |
| README.md | 2 | 2 |

对账：per-file 次数与密度报告、施工图 §8.4 数字基线逐项一致（delivery 41、spec/agent 39、spec/system 32、spec/connections 29、spec/task 26、spec/project 26、spec/run 24、设计地图 12；合同层含 spec/README 9 合计 185；本清单口径全库 320 = 合同层 185 + 设计层/门面 110 + decision-history 25）。

## doc-cleanup-backlog 13 条并入

来源：`.memo/notes/doc-cleanup-backlog-20260825.md` §2（v0.12.3 评审列出的过强断言，所有者裁决"记下来，先不改"）。其中两条已过时可先行核销（★）；其余 11 条的措辞形态随 C3 簇按 §7.2 裁决，涉及语义的按施工图 §9 停车位第 3 条逐条立项。

| # | 条款 | 现位置（v0.15.0 核对） | 对应本清单区域 |
| --- | --- | --- | --- |
| 1 ★已过时 | agentd-only terminal | agentd 已退场（v0.14.x），条款对象不存在 | spec/agent.md 终端通道一节 |
| 2 | discovery 绝不联网 | spec/system.md「固定内核与受控端口」 | spec/system.md 行 44–46 |
| 3 | 所有动作都必须 Preview | spec/system.md 场景端口、spec/project.md 场景合同 | spec/system.md 行 63–74；spec/project.md 行 93–99 |
| 4 | writer 不可证明静默就永久弃用 worktree/ChangeSet | spec/agent.md「ChangeSet 与 Git 事实」 | spec/agent.md 行 38 |
| 5 | 清理前绝不允许丢弃残留 | spec/agent.md 同节 | spec/agent.md 行 56 |
| 6 | 固定锁路径与多层 generation | spec/system.md「单写者」 | spec/system.md 行 179–185 |
| 7 | 固定存储拓扑 / 禁止 Git refs | spec/system.md「事实与存储」 | spec/system.md 行 143–153 |
| 8 | Repo Instance 强绑定 common-dir 取证 | spec/system.md「Repo 与执行现场」 | spec/system.md 行 131–133 |
| 9 | Project 归档前必须清空全部开放对象 | spec/project.md | spec/project.md 行 42 |
| 10 | Scoped Room 只有成功回填才能归档 | spec/project.md | spec/project.md 行 52 |
| 11 | Context 必须毫秒级、全本地、零模型 | context.md、spec/project.md | context.md 行 17、50；spec/project.md 行 65 |
| 12 | 活动 Run 时禁止采纳新 Task Revision | spec/task.md | spec/task.md 行 30 |
| 13 ★已过时 | 打包与 tmux 拓扑写死 | tmux 已退场（v0.14.1），条款对象不存在 | delivery.md 打包策略节 |

## 逐条清单

### docs/design/README.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 48 | 2 | 第三方平台可以提供场景客户端，也可以通过 Chat、任务源、workflow engine、harness、Agency 这五类受控端口提供底层能力；受控端口只报告读写能力和降级方式，字段权威由对应模块的权威绑定（authority binding）授予。同一产品兼任两者时，客户… | CT-AGENT | 可机械覆盖 |
| 50 | 1 | 第一阶段采用 Tuwunel、Vikunja、Dagu、Herdr，并不把四个模块绑定到它们的私有对象模型。替换边界是各模块自己的受控端口和版本化 binding，不是一套跨模块的通用 shim 服务；Workbench 的 HCTL 部分只依赖场景合同，provider 客户端… | CT-RUN | 可机械覆盖 |
| 58 | 1 | - provider 事件先按模块合同分类；它可以只是 content 观测，也可以在保留操作者映射、目标、版本和幂等依据后成为 human 命令请求或运行时输入，不能从某个 UI 的名称直接推断权限。 | CT-AGENT | 可机械覆盖 |
| 59 | 1 | - Task 只有两个获准的终结来源：有权 human 的完成请求，或绑定契约的 Run 正常完成后由归约器机械提交同一个命令；请求可以来自 Workbench、CLI，或 binding 明确支持并能归属到 human 的 provider 动作，但都要经过同一 Task 验收… | CT-TASK | 可机械覆盖 |
| 60 | 1 | - 普通 Room 的临场执行边只由人提交；模型 Participant 可以建议下一位协作者，但不能自行点名执行者、扩大群发范围或递归委派。预授权的自动边只由确定性规则按冻结的施工图创建。 | CT-RUN | 特别容易犯 |
| 73 | 1 | - 分层写作：设计正文只用[核心产品词](./spec/README.md#核心产品词)加日常语言；合同词汇（复合对象名、状态机、字段）只出现在合同层；实现名（字段名、锁路径）不进设计文档。`delivery.md` 是验证文档，可引用合同层词汇以指认被测合同，但不得重定义。 | 无 | 其余 |
| 74 | 1 | - 只有合同层的四个模块合同可以定义模块特有的领域名词、状态、写入者和不变量；设计正文与场景不得重定义它们。 | 无 | 其余 |
| 75 | 1 | - 具名概念的引入门槛与族规则见[合同层总则](./spec/README.md)：没有独立生命周期、恢复边界或权限边界的不得命名；场景概念优先对齐外部标准，不重复造轮子。 | CT-SYSTEM | 可机械覆盖 |
| 81 | 1 | - 精简只针对重复权威、无第一阶段用途的对象和补丁衍生对象；能够回答独立实现选择、交接、故障或权限边界的设计不得因篇幅被删除。 | 无 | 其余 |
| 100 | 2 | 发生冲突时，四个模块合同解释连接端点，spec/connections.md 解释交接，spec/system.md 解释共享执行机制；delivery.md 不得改变领域含义，实现证据不得反向定义产品。 | 无 | 其余 |

### docs/design/agent.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 7 | 1 | 进程、PTY（伪终端）、worktree（Git 工作树）和终端连接都是可替换的物理资源：它们可以丢失、重建和接管，但不能反过来定义 Project、Task 或 Run 的事实。Agent 模块的职责，是把上层的一次授权变成可观察、可隔离、可恢复的物理执行，并诚实上报发生了什么… | CT-RUN | 可机械覆盖 |
| 23 | 1 | - 观测（进程、屏幕、心跳、钩子）无论多可信都只是观测；它可以触发关注，不能自动变成领域结果。 | CT-AGENT | 可机械覆盖 |
| 27 | 1 | - 三条底线不可关闭：工具不是人（关任务只有两种来源：认证的客户端会话里的人，和施工图走完；执行体的产出只能经提案通道进来）；合入的钥匙不进工具（集成与外部写凭据只由工具箱和适配器代用）；每个变更集用自己的工作树和写租约。操作系统沙箱、凭据代用范围、网络白名单这些外层笼子是可以在… | CT-TASK | 特别容易犯 |
| 30 | 1 | - 客户端退出不停止执行；断流恢复时证明不了是同一个进程，就不能自称精确接管。 | CT-AGENT | 可机械覆盖 |
| 46 | 1 | Workbench 就位之前（P2），观察与接管可以经 `hctl2 terminal` 命令取得 control 为精确目标签发的短期票据，也可以在执行规格允许普通交互时直接使用 Herdr TUI。两者都是 Terminal 客户端，没有权限等级差异；区别是当前 Herdr … | CT-AGENT | 可机械覆盖 |
| 50 | 1 | ｜ 角色 ｜ 可以做什么 ｜ 不能做什么 ｜ | 无 | 其余 |
| 73 | 1 | - Herdr TUI 的直接输入不经 HCTL Terminal Input Lease，当前也没有可供 HCTL 可靠记录每次输入的事件。执行规格允许原生交互时，这仍是有效的用户运行时输入，但必须标明输入历史与单输入者保证不完整，不能声称所有输入都受 HCTL 租约控制；这本… | CT-AGENT | 可机械覆盖 |
| 74 | 1 | - Herdr API 与原生 controller 目前可以交错写入。在 Herdr 提供统一写入权之前，要求 HCTL 单一输入者保证的执行不得同时开放两条写入路径。 | CT-AGENT | 可机械覆盖 |
| 75 | 1 | - Herdr 的 API 事件环是内存中最近 512 条，没有对外 sequence 或 gap 通知；它可以支持 UI 观察，不能被当成完整持久 trace。 | CT-AGENT | 可机械覆盖 |

### docs/design/architecture.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 21 | 1 | 因此不能把所有原生客户端统一称为“只读”或“带外”：Matrix 与 Vikunja 原生界面本来就是 content 的正常写入者；Herdr TUI 可以是正常的人机输入通道，只是 v0.8.2 无法提供 HCTL 单输入租约的证明；Dagu 管理界面会先改变机械执行，无法满… | CT-RUN | 可机械覆盖 |
| 49 | 1 | Workbench 是四个场景的稳定组合界面，但只使用公开合同：HCTL 命令与 CLI 同路，content 和运行时动作与对应原生客户端同路。第三方原生界面不定义 HCTL 功能；Dagu 页面、Vikunja 页面或 Herdr TUI 的私有对象模型不得进入模块合同。Te… | CT-RUN | 可机械覆盖 |
| 51 | 1 | “可替换”分三档承诺，不能混为一谈：新工作可以在通过合同测试后选择另一 provider；活动执行继续使用冻结的 binding，不热切换；既有 content 能否迁移取决于两端导入导出能力，需要单独预览和校验。HCTL 自己的 metadata、不可变引用和 Git 结晶不依… | CT-CONNECTION | 可机械覆盖 |
| 64 | 1 | 矩阵里的 artifact 是一种**解释性结晶规律**：重要结果通常会形成可审阅、可分发的 Git 工件；它不是把 metadata 或 content 逐字节变换成 Git 文件的存储定律。结晶的归属以事实为准绳——它从哪个场景长出来就归哪一格（施工图从 Room 的塑形讨论… | CT-RUN | 可机械覆盖 |
| 76 | 1 | ｜ Project → Task ｜ 冻结的任务契约（Task Revision）+ 来源回链 ｜ 讨论升格为承诺；普通消息、总结和拖放都不能创建 Task ｜ | CT-PROJECT | 可机械覆盖 |
| 96 | 1 | - **metadata**：控制面账本是唯一不可再生的权威，必须有备份；判决的结晶副本进 Git 后可以部分回灌，但回灌不能伪造未结晶的判决。 | CT-SYSTEM | 可机械覆盖 |
| 97 | 1 | - **content**：丢失不会抹掉已经接纳的治理事实——已结晶的部分（决议、契约、凭证、代码）存活于 Git，有桥接来源的部分可以重放，丢掉的是尚未结晶的记忆；但需要核对 provider 当前事实或重建来源链的命令仍须 fail closed，不能拿旧结晶冒充 fresh… | CT-TASK | 特别容易犯 |

### docs/design/context.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 9 | 1 | Context 不是"把最近若干条聊天塞进 prompt"。那只能恢复文字，回答不了真正要紧的问题：这次执行继承了哪些目标、决定、约束和未决问题？每条信息来自哪里、是否还新鲜？检索覆盖了什么、漏了什么？故障切换之后，能不能重现同一份交付义务？ | CT-SYSTEM | 可机械覆盖 |
| 58 | 1 | 萃取出的上下文**缺省不压缩**，原文直给、预算内按选择优先级裁剪。只有当用户配置了专用小模型（small-brain）时才启用**压缩**——摘要式或逐词裁剪式都可以，但压缩永远是清单里显式记录的一步：用了哪个模型、压了多少、原文指纹是什么，全部冻结；可解释性不因压缩打折。证据… | CT-PROJECT | 可机械覆盖 |
| 76 | 1 | 3. **它永远是派生缓存，不是权威**：每条带消息事件回源指针；治理引用不得指向纪要，只能指向精确事件；丢了就重建，不进账本，清单只记引用与指纹。它也不由房间里的模型 Participant 书写——"agent 自己决定记什么"是自述，纪要是组装器机械触发、small-bra… | CT-PROJECT | 特别容易犯 |
| 92 | 1 | - **可解释**：每次调用冻结一份上下文清单——来源、筛选依据、新鲜度、覆盖面和已知缺口都写在里面；事后永远能回答"它当时看到了什么、漏了什么"。模型的自由总结不能替代来源和版本。 | 无 | 其余 |
| 97 | 1 | - **运行中追加召回不越权**：执行中可以在获准范围内补充检索，但不能绕过冻结的范围、权限与审计。 | 无 | 特别容易犯 |

### docs/design/delivery.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 20 | 2 | 第一阶段不按客户端产品划权限，而按动作分类。Matrix/Vikunja 正常读写 content；Vikunja `task.updated` 在包含映射到 owner 的 doer、稳定 task、updated revision 和明确 Done 变化时可生成完成请求，缺项… | 自身即 CT | 可机械覆盖 |
| 36 | 1 | CLI 没有隐藏权限，也不直接写治理账本、执行面 content 服务器或 Agency。`terminal attach` 只建立观察或输入通道，不恢复任何领域对象；Run 的语义恢复 / 替换使用 `run resume｜replace`，Room Invocation 的再… | 自身即 CT | 特别容易犯 |
| 67 | 1 | 6. owner human 通过 CLI 完成预览提交「完成 Task」命令，或通过已验证的 Vikunja Done 映射请求同一命令；Task 准入校验精确 Integration Receipt 后写 Task Completion Receipt，Harness 不能代… | 自身即 CT | 可机械覆盖 |
| 81 | 1 | 8. control 先持久化 intent/outbox，`hctl2-tool`（本地）或 adapter（远端）执行并 readback；只有确认目标事实后才写唯一 Integration Receipt，结果未知时不得签成功或盲重投。 | 自身即 CT | 特别 tricky |
| 103 | 1 | 旧工具在事实切换前可以作为执行者或逃生通道，不能继续保有平行 Project/Task/Run 账本。降级超过约定能力时回退到上一自举级别并留下审计记录。 | 自身即 CT | 可机械覆盖 |
| 105 | 1 | B5 是第一阶段功能成熟度目标；正式发布、升级与回滚仍必须通过 B6，不能把“已能自举”当成可分发版本。 | 自身即 CT | 可机械覆盖 |
| 107 | 1 | 自举验收不得对 HCTL2 仓库、内置账号或测试环境设置隐藏的特例豁免：开发自身必须只使用公开的 Query/Preview/Submit/Subscribe、CLI 和受控端口，实际 Context、权限与证据均可检查；手工推进 Engine、直接改库、隐藏 Prompt/Co… | 自身即 CT | 特别容易犯 |
| 124 | 1 | - chat server 中的普通消息、反应或自动化不能成为命令；binding 未列明、actor 无法映射、source event/target/version 缺失的结构化动作同样拒绝 | 自身即 CT | 可机械覆盖 |
| 125 | 1 | - 同一显式 Matrix human action 经 direct Workbench adapter 或 provider event adapter 归一后 command digest 与结果一致；HCTL service/bridge bot 的同形事件不能取得 hu… | 自身即 CT | 可机械覆盖 |
| 126 | 1 | - 模型 Participant 的 `@`/建议不能创建 Invocation 或 fan-out，human 批准后自动携带来源/Context | 自身即 CT | 可机械覆盖 |
| 146 | 1 | - 同一规范实体跨 Project/connection/placement 不得产生第二个 Task，禁用 binding 也不释放映射 | 自身即 CT | 可机械覆盖 |
| 155 | 1 | - Dagu UI/API 直接 Start/Stop/Retry/Reschedule/Approve/Reject/Edit/Rename/Delete 时只标记 Engine Execution Binding 分歧，不倒推 Run 命令、Verdict 或 Receipt… | 自身即 CT | 可机械覆盖 |
| 161 | 1 | - Gate backup 改变参与者或任一 Context/Skill/policy ref 时拒绝，作者不能占必需 reviewer Seat | 自身即 CT | 可机械覆盖 |
| 169 | 1 | - 无法证明旧 writer 已 fence 时隔离旧 worktree 且不得重授租约，失败清理不丢唯一未封存/未跟踪修改 | 自身即 CT | 可机械覆盖 |
| 170 | 1 | - 本地/远端 SCM 集成都先持久 integration intent，由 tool/adapter 执行并 readback，target-head 竞争或 ACK 未知时不得签成功 Integration Receipt | 自身即 CT | 可机械覆盖 |
| 176 | 1 | - 人直接修改 Herdr workspace/pane 归属或已冻结派工结果只形成 drift，不能冒充结果；对精确 terminal 的输入则按 Execution Spec 输入策略处理 | 自身即 CT | 特别容易犯 |
| 177 | 1 | - `native_interactive_allowed` 下 Herdr TUI/Workbench 直连输入是有效运行时输入，但必须标明逐次 provenance、generation 和物理单写者保证不完整；该输入不能直接产生领域结果 | 自身即 CT | 可机械覆盖 |
| 178 | 2 | - `managed_single_writer` 下不得同时开放 Herdr API 写入与原生 controller 写入，尝试原生写入时执行不得继续声称策略成立 | 自身即 CT | 可机械覆盖 |
| 179 | 1 | - Herdr 事件流没有可回读 sequence/gap 时，不得当作完整持久 trace；重连后只能按可证明范围恢复观察 | 自身即 CT | 可机械覆盖 |
| 180 | 3 | - Herdr 不能证明同一进程和 PTY 仍存活时，不得声称 exact attach；缺失 exit/stop 回执时不得把执行报告为成功停止 | 自身即 CT | 可机械覆盖 |
| 184 | 1 | - control 签发 descriptor、Herdr 适配代码校验 HCTL 授权，观察、输入、Attempt 控制与安全输入权限分离；Herdr API 无法执行的 fence 不得被记录为已生效 | 自身即 CT | 可机械覆盖 |
| 185 | 1 | - attach 只接通道，不能恢复 Run/Invocation 语义 | 自身即 CT | 可机械覆盖 |
| 192 | 1 | - client/port 连接与 binding 分离；同一产品同时作客户端与 provider 时不能借一侧身份写另一侧事实 | 自身即 CT | 可机械覆盖 |
| 193 | 1 | - actor provenance 不能由 payload 自报 | 自身即 CT | 特别容易犯 |
| 199 | 2 | - 新 provider/adapter 未通过对应模块合同测试时不得产生 Resolved Port Binding；换绑不能改写活动 Run、Task、Room 或 Execution Runtime 的冻结 binding | 自身即 CT | 可机械覆盖 |
| 200 | 1 | - 既有 content 迁移必须显式预览、导出、导入并回读校验；普通换绑不得冒充无损迁移或热切换 | 自身即 CT | 特别容易犯 |
| 201 | 1 | - 客户端无等级：Workbench 通过 provider 通道执行的消息/卡片/终端动作与原生客户端同语义，通过 command service 的动作与 CLI 同语义；Workbench 不得依赖 provider 私有导航或对象模型获得隐藏权限 | 自身即 CT | 可机械覆盖 |
| 208 | 1 | - 多个执行现场可以登记（各有工具箱与 Herdr 绑定），但同一 site/repo mutation lease 的旧 generation 必须被 fence，无法证明 fence 时不得重授写权限 | 自身即 CT | 可机械覆盖 |
| 215 | 1 | - 从 Git 结晶回灌不得伪造未结晶判决 | 自身即 CT | 可机械覆盖 |
| 231 | 1 | - 同一 Request ID 跨 Room/Task/Run 聚合且不能从聚合面直接改状态 | 自身即 CT | 特别容易犯 |
| 239 | 1 | - 输入优先级为 IME composition → 已聚焦 terminal → modal/composer → 当前场景 → 全局快捷键，任何上层快捷键都不能截获正在组合或发往 terminal 的输入 | 自身即 CT | 可机械覆盖 |
| 258 | 1 | P0 的内容就是本节。各项选型已拍板，验证因此从“选谁”变为“关键假设能否落地”；关键假设只指 HCTL 实际调用的 API 与行为。第三方自身的功能（它自己的备份恢复、重启、渲染、内存配置、发布物形态）不在 P0：要么是选型时的资料判断，要么在首次消费时产品化。每项探针使用可删… | 自身即 CT | 可机械覆盖 |
| 260 | 1 | 1. **workflow engine（Dagu，已拍板）**：固定基线为 [`v2.15.1 / 532c5129`](https://github.com/dagucloud/dagu/tree/532c512944b2e5eb8991b5bc7cbeafa74fd5b47… | 自身即 CT | 可机械覆盖 |
| 276 | 1 | Rust control/tool 与 Herdr 适配代码；Tauri 2 + React 19 Workbench（GPUI 原生备选，Electron 安全网）；SQLite + FTS5 与 Git；Tiptap、React Aria、React Flow + Dagre… | 自身即 CT | 可机械覆盖 |
| 278 | 1 | 任何采用、移植或 vendor 的外部源码都必须固定已审阅 commit，核验目标文件及依赖许可证，保留 license/copyright/attribution 与修改记录，并用 HCTL contract tests 隔离上游漂移；任一项缺失即不得进入分发产物。 | 自身即 CT | 可机械覆盖 |
| 290 | 1 | - ~~第一阶段之后首个非 Matrix 聊天平台桥接~~ 已了结：HCTL 永不自建聊天桥接——非 Matrix 平台经 homeserver 侧 Matrix 桥接生态接入（content 层），HCTL 只保留桥接用户的身份映射策略（见 [Project 设计正文](./p… | 自身即 CT | 可机械覆盖 |

### docs/design/participant.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 11 | 1 | Participant 存在的意义是把"谁"从"用什么跑"里独立出来：稳定的是参与者身份，变化的是它被实现的方式。新进程、新会话、备用尝试都不产生新的参与者；换成另一个参与者也不能靠复用同一套执行配置伪装成同一个人。 | 无 | 其余 |
| 15 | 1 | 一个数字参与者由七件生命周期不同的事组成，不能压成一份大配置： | 无 | 其余 |
| 35 | 1 | - 不同参与者身份不自动等于独立评审：第一阶段只证明逻辑分离，背后是否不同操作者、不同模型提供方，能认证的标 known、不能认证的标 unknown，不冒充。 | 无 | 特别容易犯 |

### docs/design/project.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 19 | 2 | - 普通聊天内容只能形成提案；正式变化必须走带预览的类型化命令。一个明确配置、可归属到 human 且绑定精确消息事件的结构化动作可以成为命令请求，但消息正文、反应和模型建议不能靠内容推断成命令；Room 仍不能签发 Verdict、Receipt 或完成 Task。 | CT-PROJECT | 可机械覆盖 |
| 22 | 1 | - 上下文必须能解释它当时看到了什么；模型自由总结不能替代来源和版本。 | 无 | 其余 |
| 24 | 1 | - 消息只追加；修正、删除和外部编辑留痕，不能抹掉已被引用的历史。 | CT-PROJECT | 可机械覆盖 |
| 56 | 1 | 普通 Room 里的临场执行边只能来自可稳定归属到 human 的动作，并且必须先经过 Trigger Preview；动作可以由 Workbench/CLI 直接提交，也可以由按公开合同适配的 provider 结构化事件提交，客户端名称不改变规则。聊天消息本身不是入口。模型 … | CT-PROJECT | 可机械覆盖 |
| 58 | 1 | ｜ 角色 ｜ 可以做什么 ｜ 不能做什么 ｜ | 无 | 其余 |
| 74 | 1 | - Task、Run 和 Agent 模块的状态只以投影或引用回到 Chat Room；普通聊天 content 不能反向改写，显式 human 动作只能经对应模块的公共命令合同请求变化。 | CT-RUN | 可机械覆盖 |

### docs/design/references/decision-history.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 16 | 1 | HCTL2 最初面对的产品表象，是同时管理多个 Harness、终端、session、pane 和 worktree。Zellij、WezTerm、cmux、Termio、Herdr 等实现说明了进程托管、终端重连、布局和运行信号应怎样做得可靠，也暴露了一个边界：终端只能说明“哪… | —（历史叙述） | 其余（历史叙述） |
| 18 | 1 | 因此 Terminal 从产品中心退为第四模块的操作场景；该模块在当时称 Harness，v0.10.0 后正名为 Agent。进程、PTY、pane 和 runtime 都是物理承载；它们可以丢失和重建，却不能反向定义 Project、Task 或 Run 的事实。这一转折把 … | —（历史叙述） | 其余（历史叙述） |
| 42 | 1 | Scene 是投影和用户交互方式，不是第五个 writer。Room 的 repo/project/scoped 拓扑也与控制拓扑正交：共享协作不要求某个 Harness 永久在线，终端重连更不能接管 Room 或 Project 的身份。 | —（历史叙述） | 其余（历史叙述） |
| 48 | 1 | 与此同时，Chat、任务源、workflow engine、harness 和运行时后端被收敛为受控端口。它们提供外部能力、报告版本与降级方式，但不能凭平台自身的 Session、Issue、workflow task、pane 或数据库取得 HCTL 字段权威（第 12 节后来… | —（历史叙述） | 其余（历史叙述） |
| 58 | 1 | 早期实践曾试图通过 prompt 要求 Harness 在“确实完成”时才报告完成，并期待模型自行维持 Task、Run 与执行结果之间的边界。实际使用表明，这种约束不能稳定提供终结权：模型输出仍是受上下文影响的建议，Harness 也只能观察本次执行，无法替 Task 的冻结验… | —（历史叙述） | 其余（历史叙述） |
| 68 | 1 | 因此当前设计只承认两类边：普通 Chat Room 的临场协作边由有权 human actor 创建，Workflow 的执行边由 reducer 按冻结的 Workflow Revision 创建。Agent-authored message、Result Proposal 或… | —（历史叙述） | 其余（历史叙述） |
| 107 | 2 | 这次转向显式推翻了两条旧结论。其一，“平台不能成为第五事实源”精确化为**可以拥有 content、不能拥有治理**——room-ground-truth memo 对 Matrix 候选的否决在三分下失效：平台拥有的是记忆，不是裁决。其二，metadata 账本的归属从 Rep… | —（历史叙述） | 其余（历史叙述） |
| 123 | 1 | 同一基线的复审保留四段骨架，但修正了三处过强表达：P0 改为限时、可丢弃的实现探针，content 系统在 P2 首次被纵向切片消费时才产品化；P1 的 standalone 工具只能辅助开发，B2 才是第一次真正自举；P2 的完整治理入口是公共 CLI，Matrix/Vikun… | —（历史叙述） | 其余（历史叙述） |
| 132 | 2 | - **受治理 Harness 的 OS 沙箱入场券**（v0.13.0 降为可选加固，见 [§22](#22-信任模型收窄三条底线不可关外层笼子可选cli-即人v0130)）：第一阶段受治理执行必须运行在操作系统强制的沙箱中，凭据只经网关代用；不能强制这些边界的候选不得作为受治… | —（历史叙述） | 其余（历史叙述） |
| 143 | 2 | - **终局结果契约**：每个接入端口声明其终局结果事件；执行体进程正常退出但缺少该事件时，适配器必须合成类型化协议错误，不得默认成功——静默死亡不能冒充交付。由 control/agentd 主动取消导致的退出归因为取消，不上报为执行失败。 | —（历史叙述） | 其余（历史叙述） |
| 144 | 1 | - **观测流完整性**：观测上报通道失败时，只能显式标记该执行的观测截断并终结事件流；有缺口的事件流不得冒充完整历史。 | —（历史叙述） | 其余（历史叙述） |
| 153 | 1 | Dagu 也不是天然的 external-worker engine：普通 step 会自行执行。采用边界因此固定为 HCTL JSON 经 compiler 生成受限 Dagu YAML，只使用依赖/条件/等待等机械结构和无进程 `human.task` 检查点；control… | —（历史叙述） | 其余（历史叙述） |
| 184 | 1 | - **压缩合同**：缺省关闭；small-brain 是经用户级定义机制固定 revision/digest 的模型引用（无新对象）；逐条记录 compressor、压缩率与原文 digest 且片段可回源；证据类内容永不压缩，违规 Bundle 拒绝交付；萃取/压缩产物可按（… | —（历史叙述） | 其余（历史叙述） |
| 193 | 1 | 正确道路只有一条：**三条底线**在治理面成立且不可声明关闭——工具不是人（治理命令只有两类入口：经认证的场景客户端会话——Workbench、CLI 或按公开合同适配的第三方客户端——以及施工图走完后 reducer 提交的同一「完成 Task」命令；Harness 的产出只经… | —（历史叙述） | 其余（历史叙述） |
| 225 | 1 | 同日第二轮追加裁决（v0.13.3）：provider 合同**永不包含治理权威**——治理（租约、代次、冻结规格、审计、四级恢复裁决）由 control 与 agentd 桥统一提供、跨所有 provider 拉齐；provider 自带的治理样机制只作执行协助与观测证据。**… | —（历史叙述） | 其余（历史叙述） |
| 243 | 2 | 2026-08-29 所有者继续裁决。Agency 保留为 Agent 模块 / Terminal 场景的供应端角色，但第一阶段不再建设 `hctl2-agency`，也不保留 `hctl2-agentd + tmux` 的并行方案；Herdr 直接按规格启动 Harness，持… | —（历史叙述） | 其余（历史叙述） |
| 245 | 1 | 同一轮明确了四个默认二进制都不能成为产品合同。HCTL 不增加一个跨模块的通用 shim 服务，而由每个模块自己的受控端口和薄 adapter 隔离实现：Chat Room 使用 Matrix 协议，Tuwunel 可换成其他 Matrix homeserver；Kanban 通… | —（历史叙述） | 其余（历史叙述） |
| 259 | 1 | 系统合同据此把动作分为 content 写入与观测、human 命令请求、运行时输入、Result Proposal 和不支持的 provider mutation。provider event 可以在模块明确允许时表达 human 命令请求，但必须归一到与 Workbench/… | —（历史叙述） | 其余（历史叙述） |
| 263 | 1 | - Chat：Matrix 客户端和 Workbench 都可正常写消息 content；普通消息、反应、mention 和模型建议不含完整命令语义，不能自动派发。将来只有 Chat binding 明确列明、绑定精确 source event 且能映射 human 的结构化动作… | —（历史叙述） | 其余（历史叙述） |
| 264 | 1 | - Task：Vikunja 原生 UI 把卡拖入 Done 会在后端真实修改 `done`，并发出包含 `doer` 的 `task.updated` webhook；因此对已绑定卡片，这个明确动作在 remote revision/updated version、fresh … | —（历史叙述） | 其余（历史叙述） |
| 265 | 1 | - Run：Dagu 的 UI 是完整管理界面，Start/Stop/Retry/Reschedule/Approve/Reject/Edit/Rename/Delete 会先改变定义或机械执行，无法让 control 在副作用前持久化 intent、撤销 Attempt/租约并… | —（历史叙述） | 其余（历史叙述） |

### docs/design/references/glossary.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 107 | 1 | control writer、Repo Instance site 与 Agency/backend owner 的排他权以各自 generation（代次）表达；Attempt 的 owner generation 与 Execution Runtime 的 runtime g… | CT-RUN | 可机械覆盖 |
| 142 | 1 | ReviewSubjectRef（评审对象引用：kind + ID + 摘要）、revision_digest 与 review_subject_digest（两种不同语义的摘要，不能互换）。 | CT-CONNECTION | 可机械覆盖 |

### docs/design/run.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 7 | 1 | 行业并不缺工作流引擎：DAG、代理委派、重试、定时器和历史恢复都已被反复实现。真正缺失的，是一套面向项目语义、绑定精确版本与证据的治理层——它回答“这个节点对应哪份交付义务、谁有资格尝试、结果是否有效、凭什么可以算完成”。这件事不能委托给聊天室、看板、Harness、终端或任何通… | CT-TASK | 可机械覆盖 |
| 14 | 1 | 引擎拥有机械位置，HCTL 拥有语义治理；两边不能互相冒充。 | CT-RUN | 特别容易犯 |
| 38 | 1 | - 运行中除清单明确声明可变的放置参数外不得原地漂移；范围、验收、候选或权限要变，就结束或替代旧 Run，而不是原地改。 | CT-RUN | 可机械覆盖 |
| 44 | 1 | - 裁决与凭证必须绑定精确的评审对象版本；当前指针或文件路径不能替代版本。 | CT-CONNECTION | 可机械覆盖 |
| 51 | 1 | ｜ 角色 ｜ 可以做什么 ｜ 不能做什么 ｜ | 无 | 其余 |
| 57 | 2 | Workbench 关闭不停止 Run。Workbench 就位之前（P2），Run 的预览、启动、暂停与取消走 `hctl2` CLI；Workbench 就位后仍调用同一服务。Dagu 自带控制台是 provider 的完整管理界面，但不适合普通 Run 动线：它不能在 mu… | CT-TASK | 可机械覆盖 |

### docs/design/spec/README.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 5 | 1 | > 定位：本目录是 HCTL2 的合同层——精确的对象、状态机、写入者与共享机制。设计层（`docs/design/` 根目录）用产品语言回答为什么与怎么用；两层冲突时以合同层为准，但合同层不得引入设计层没有的产品行为。 | 无 | 其余 |
| 18 | 1 | 新名字的引入门槛：不满足前两类判据的不得命名；能用日常语言或外部标准词说清的不另造词。自造语义名不冻结代码词形：具名对象与票据写成带空格的专名（如 Task Revision、Gate Receipt），命令写动宾语义名（如「完成 Task」命令），状态值写中文语义名；实现时附「… | CT-TASK | 可机械覆盖 |
| 55 | 2 | 1. **能承载不等于能裁决。** content 系统拥有场景内容的 ground truth，但永远不拥有治理：普通消息不能触发派发，provider Done 最多请求同一 Task 验收，引擎的机械完成不能签发凭证。判决只在 metadata 层产生。 | CT-PROJECT | 可机械覆盖 |
| 57 | 1 | 3. **命令走 HCTL，记录落平台。** 类型化命令的预览、准入与判决在 metadata 层执行；human 请求可以来自 Workbench/CLI，也可以来自模块 binding 明确接纳的 provider 动作，但必须归一到同一命令。结果可以作为记录写回 conte… | CT-CONNECTION | 可机械覆盖 |
| 123 | 1 | ｜ “不得读取目标 ref/common-dir” ｜ 删：Harness 可读 common-dir/refs 并在本 ChangeSet 分支提交；直写目标 ref 不取得集成 authority，只回读为 drift ｜ | CT-AGENT | 可机械覆盖 |
| 127 | 1 | ｜ “后端无并发令牌则降级只读”“不能强制排他的 backend 只可观察” ｜ 归还给依赖：任务后端的并发控制归后端，adapter 按能力用其前置、以回读为准；运行时租约与代次只在账本，后端排他原语是加固 ｜ | CT-AGENT | 可机械覆盖 |
| 133 | 2 | 每个模块还必须用自己的受控端口隔离默认实现：适配代码只覆盖 HCTL 实际使用的最小能力，固定实现版本、adapter 版本、配置摘要与实测能力；未知能力安全拒绝。不得把 Dagu、Vikunja 或 Herdr 的私有对象直接提升为 HCTL 对象，也不得为四个模块另造一套通用… | CT-RUN | 可机械覆盖 |

### docs/design/spec/agent.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 29 | 1 | ｜ Terminal Input Lease ｜ lease generation；活跃 / 已撤销 / 已过期 ｜ control 授予/撤销，Herdr 适配代码只把当前租约的输入送入 API；Herdr 原生写入不受该租约约束，是否允许由 Execution Spec 的输… | CT-AGENT | 可机械覆盖 |
| 34 | 3 | 第一阶段 HCTL 启动的每个 Harness 都以窄 execution principal 运行，三条底线不可声明关闭：**工具不是人**——Harness、runtime hook 与模型只有 Result Proposal 通道，不是治理命令入口（两类入口见[系统边界](… | CT-TASK | 可机械覆盖 |
| 38 | 2 | Worktree（Git 工作树）是 ChangeSet 的可替换物理资源，不永久属于 Project、Task、Room 或 Harness。一个 ChangeSet 同时最多一个有效写入租约；候选切换、接管或取消必须先让旧 writer 失权。恢复时若无法证明旧 writer… | CT-RUN | 可机械覆盖 |
| 54 | 2 | 模型自述“已合并”不可信。本地「合入 ChangeSet」命令至少固定 ChangeSet Revision、source/base、target ref、expected target head、策略、适用 Verdict/evidence、actor/permission、b… | CT-TASK | 特别容易犯 |
| 56 | 1 | 失败、取消、租约撤销和资源清理都不等于放弃代码。物理清理前，工具箱必须确认所有已跟踪、未跟踪且尚未封存的修改已有可恢复副本，现场资源只有得到该确认才可拆除；保全或封存失败时保留精确 worktree 路径、Git 状态和显式恢复动作，不能删除唯一副本。清理 worktree 也不… | CT-AGENT | 可机械覆盖 |
| 62 | 1 | Room Invocation 拥有的 Execution Runtime 继承其 Execution Spec 的 `project_scope ｜ repo_scope`；Attempt 拥有的运行时的 Project 范围来自 Run Manifest。repo-scope… | CT-RUN | 可机械覆盖 |
| 64 | 3 | 代次必须分层记录而不能共用一个模糊 `generation`：语义 owner 是 Room Invocation 的 `invocation_version` 或 Attempt 的 `attempt_generation`；物理 Execution Runtime 在激活映射… | CT-RUN | 可机械覆盖 |
| 68 | 4 | Agency 合同**永不包含治理权威**：租约、代次、冻结规格、审计与恢复等级裁决只在 control 账本；Agency 自带的接管、单写者或“会话有效”记录只作执行协助与观测证据，不得写入或替代账本事实。Agency 若声明栅栏回显能力，必须原样携带并回显请求所附的代次与租… | CT-AGENT | 可机械覆盖 |
| 70 | 2 | 进程、PTY、原始流与心跳由 Herdr 持有；control 经 Herdr 适配代码执行已获准的 start/input/cancel/stop，Attempt/Invocation 的领域 lifecycle 仍由 control 推进。存活与所有权观测按 Herdr AP… | CT-TASK | 可机械覆盖 |
| 72 | 3 | Execution Spec 必须冻结 terminal input policy：`managed_single_writer` 要求所有输入经当前 descriptor、generation 与 Terminal Input Lease 校验，provider 不能统一拦截所… | CT-AGENT | 可机械覆盖 |
| 74 | 2 | Herdr v0.8.2 的能力边界进入合同：原生 TUI 输入不经 Terminal Input Lease，API 与原生 controller 可以交错写入；事件 API 只保留内存中最近 512 条，且没有公开 sequence/gap；重启后的同一进程/PTY、退出码与… | CT-AGENT | 可机械覆盖 |
| 76 | 1 | 结构化事件统一归一为生命周期提示、工具调用、权限请求、文件变化、测试、用量和原始输出。未知事件保留原文并安全降级，不得凭渲染器猜测完成。 | CT-TASK | 可机械覆盖 |
| 78 | 4 | 每个 harness 适配器必须为其接入端口声明终局结果契约：执行体进程正常退出但缺少契约要求的终局结果事件时，适配器必须合成类型化协议错误，不得默认成功；由 control 主动取消导致的退出必须归因为取消，不得上报为执行失败。观测上报通道失败时，只能显式标记该执行的观测截断并… | CT-TASK | 可机械覆盖 |
| 80 | 3 | 物理执行的每个 Result Proposal 固定 proposal ID、owner kind/ID + `invocation_version ｜ attempt_generation`、Execution Runtime ID + `runtime_generation`… | CT-AGENT | 可机械覆盖 |
| 82 | 1 | Harness、runtime hook 与模型只获得当前 Invocation/Attempt 所需的窄 execution principal，不能持有通用 command Submit credential、human principal credential、Task l… | CT-RUN | 可机械覆盖 |
| 86 | 2 | Terminal 各能力（exact attach、native handoff、structured inspect、semantic resume、replay，见[设计正文](../agent.md#terminal-场景)）可以并存但不能互相冒充。运行时绑定提交后，con… | CT-AGENT | 特别容易犯 |
| 88 | 1 | Execution Chat projection 是 Terminal 中绑定且只绑定一个精确 Room Invocation/invocation_version 或 Attempt/attempt_generation、对应 Execution Runtime/runtim… | CT-RUN | 可机械覆盖 |
| 92 | 1 | Workbench 或终端客户端退出不停止执行。断流按 runtime generation、来源流 sequence 和快照恢复；无法证明是同一进程时只能 semantic resume、replay 或新建执行，不能声称 exact attach。semantic resum… | CT-TASK | 可机械覆盖 |
| 100 | 2 | ｜ exact attach / detach ｜ Herdr terminal connection ｜ 连接或断开仍存活的 Herdr terminal；断开不停止执行，不能证明同一进程和 PTY 时不得声称 exact attach ｜ | CT-AGENT | 可机械覆盖 |

### docs/design/spec/connections.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 10 | 1 | 1. 来源模块只能提供稳定 ID、Revision digest、状态版本、来源和已获授权的范围；不能直接写目标模块。 | CT-TASK | 特别容易犯 |
| 12 | 1 | 3. 目标状态、来源关联、幂等结果和必要 outbox 由唯一 control 在同一本用户级 metadata 账本的一个事务中提交；跨 Project、Repo Instance 或模块都不得拆成 clone 本地事务再拼接。 | CT-CONNECTION | 可机械覆盖 |
| 13 | 1 | 4. 目标只以稳定引用和有序事件返回结果；来源和场景可以投影它们，但不能复制一套状态机。 | 无 | 其余 |
| 16 | 1 | 连接中的引用至少包含 kind + stable_id + revision_digest 或 state_version，并携带所属 Repo/Project、producer 和适用的绑定版本。`current`、显示名、外部 ID、文件路径或界面选择不能替代精确引用。这是字… | CT-CONNECTION | 特别容易犯 |
| 57 | 2 | Task 模块以 CAS 校验活跃 Project 和可选当前 Task Revision。「创建 Task」固定 immutable project_id 与该 Project 的 Board group anchor，先提交 Task identity、必需的后端 outbo… | CT-TASK | 可机械覆盖 |
| 63 | 1 | control 在一个用户级账本事务中写 Run、Manifest、幂等结果、可选 Task Run claim 和 Engine start outbox。外部执行实例用 `run_id + manifest_digest` 作为关联键；commit 后崩溃或 ACK 丢失时先… | CT-RUN | 可机械覆盖 |
| 92 | 1 | owner 特有字段各自补充：Room Invocation 侧固定 scope（`repo_scope ｜ project_scope`）、`invocation_version` 与 human 批准建议时的 lineage 字段；Attempt 侧固定 attempt/se… | CT-RUN | 可机械覆盖 |
| 96 | 1 | 1. owner 模块提交 Execution Spec 与 dispatch outbox；此时只有 `invocation_version ｜ attempt_generation`，不得预填 runtime identity； | CT-AGENT | 可机械覆盖 |
| 97 | 1 | 2. Agency adapter 校验当前 control/site/binding generation，再请求选定的 Agency 进行无副作用预留，并返回实际能力、物理目标、Execution Runtime ID 与新的 `runtime_generation`；实际能… | CT-AGENT | 可机械覆盖 |
| 99 | 1 | 4. outbox 同时携带 owner version/generation、runtime generation、control writer generation、site generation 与 Agency binding owner generation；adapt… | CT-AGENT | 可机械覆盖 |
| 103 | 2 | 如果冻结的端口明确是受信任的纯进程内同步调用，Execution Spec 必须写 `execution_mode = in_process`，可以没有 Repo Instance、Runtime/Terminal、runtime/site/backend generations… | CT-AGENT | 可机械覆盖 |
| 109 | 1 | control inbox 先按 proposal ID + producer sequence + owner 去重，再逐输出校验：owner 仍接受结果，`invocation_version ｜ attempt_generation`、control writer gene… | CT-AGENT | 可机械覆盖 |
| 115 | 1 | 任一旧代次、被取消/替代 owner 或不匹配 spec/bundle 的结果只保留审计记录，不能推进 Project、Run 或 Task。 | CT-RUN | 特别 tricky |
| 119 | 1 | 无 Run 路径中，有权 human actor 在 Kanban 预览精确 ChangeSet Revision/Artifact Revision、ReviewSubjectRef 和测试/SCM 证据后提交「完成 Task」命令；它不伪造只能由 Run 产生的 Gate R… | CT-TASK | 可机械覆盖 |
| 121 | 1 | Task、Run 和 Agent 以有序领域事件向 Project 返回里程碑。事件携带 source module、稳定引用、event ID/sequence、版本和敏感级别；Project Room 只显示 Request、失败、已验证 Task、Artifact 就绪等低… | CT-PROJECT | 可机械覆盖 |
| 125 | 1 | Request 由 Project 模块保存，但可以阻塞 Task 待办、Run 中的 Attempt/Seat/Obligation，或直接 Room Invocation。创建时固定 owner_ref + affected_revision_ref + blocked_sc… | CT-PROJECT | 可机械覆盖 |
| 127 | 3 | 「解决 Request」命令固定 request/expected version、resolution digest、actor/delegation 和 idempotency key。对需要恢复执行的 Request，control 在同一用户级 metadata 账本事务… | CT-PROJECT | 可机械覆盖 |
| 129 | 1 | Deadline 到达以同样的版本 CAS 写已过期，但不伪造答案，也不产生 Task terminal 命令。它只把冻结动作投回精确 owner：Task/Project 的待办动作失败或放弃并保留 Task lifecycle；Run owner 按 [Attempt/Sea… | CT-TASK | 可机械覆盖 |
| 145 | 1 | 每一步保存上一步的 ID + digest/version；current pointer 只用于预览，不能替代历史引用。上游版本变化不改写已接受的下游连接：提交前漂移则 CAS 拒绝，提交后由冻结合同继续执行到终态，新的顶层授权使用新版本；范围、权限、候选或验收含义变化需要显式… | CT-RUN | 可机械覆盖 |
| 147 | 1 | 权限只能逐级缩小：actor/Project role → Run Manifest（有 Run 时）→ Execution Spec → Agency/adapter envelope。任何下游都不能扩展网络、secret、Git、任务源、Engine 或终端输入范围；需要扩权… | CT-RUN | 特别容易犯 |
| 168 | 2 | 系统对账完成前，各模块都不得表现为已完成交接。连接需要的新尝试或替代执行必须拥有新的 owner version/generation、Execution Spec 与 runtime generation；不能复活旧 owner。 | CT-TASK | 特别 tricky |
| 172 | 3 | Workbench 可以在一个界面编排上述连接，第三方 Chat/Kanban/Workflow/Terminal 平台也可以按能力提交同一目标命令、查询同一投影并订阅同一事件。适配器没有跨模块捷径：Task binding 明确接纳的 provider Done 只能请求同一个… | CT-TASK | 可机械覆盖 |

### docs/design/spec/project.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 30 | 1 | ｜ Context Manifest / Context Bundle ｜ immutable value + digest ｜ Project control 按获准来源、scope、权限和预算物化；consumer 只读 ｜ 后续 Room 消息、索引变化和 Harness … | CT-PROJECT | 可机械覆盖 |
| 38 | 2 | Repo 不等于外部组织、工作区或某个 clone。「注册 Repo」命令固定新 `repo_id`、预期 Git identity、repo 配置正文 digest 与幂等键；control 先在账本持久化待确认注册与工具箱 outbox，工具箱再把稳定 identity 写入… | CT-CONNECTION | 可机械覆盖 |
| 46 | 3 | Participant 使用稳定 `participant_id` 与不可变配置 revision；该 revision 描述逻辑身份、persona/沟通约束、可选 post-train 或模型资格约束、默认 Skill refs 和 Worker Profile 候选约束，但… | CT-RUN | 可机械覆盖 |
| 48 | 1 | 从 Repo Room 创建 Project 时，先提供可编辑、可删减补充和去敏的提升预览，再提交「创建 Project」命令；该命令只能显式选择来源 Message 引用和/或已预览的 Context Manifest/Context Bundle 摘要，并冻结所选内容的可追溯… | CT-PROJECT | 可机械覆盖 |
| 54 | 1 | Message 是只追加的协作事实，其 ground truth 在 chat server（Matrix 协议：编辑与撤回是新事件）；修正、删除和外部编辑形成新事件或 tombstone，不能抹掉已被引用的历史。普通回复、表情或模型总结不会修改 Project、解决 Reque… | CT-TASK | 可机械覆盖 |
| 56 | 1 | 时间线顺序由 chat server 的线性事件顺序给出（单 homeserver 合同前提，写入以事务 ID 幂等）；稳定 ID、时间戳和 Invocation 完成顺序只用于身份或展示。HCTL 治理事件在控制面账本只追加，以 Chat 端口绑定 + chat server … | CT-PROJECT | 可机械覆盖 |
| 61 | 3 | Context 交付的是调用开工时给执行体的 prompt，不代管执行体在会话内自行组装的工作上下文。Bundle 的每个条目按投喂档记录为 inline / pointer / recall 之一。inline 物化原文，只用于执行体自己拿不到或不该自己翻的部分——从聊天史与绑… | CT-PROJECT | 可机械覆盖 |
| 63 | 2 | 每个 Room Invocation 或 Attempt 消费者再从根 Manifest 物化自己的 Context Bundle；Bundle 至少固定 `context_bundle_id`、Manifest ref+digest、consumer owner ref + 精… | CT-PROJECT | 可机械覆盖 |
| 67 | 1 | 压缩缺省关闭。仅当用户配置了专用压缩模型（small-brain——经用户级定义机制固定 revision/digest 的模型引用，不是新对象）时，Bundle 物化才可压缩。每个被压缩条目必须记录 compressor ref+digest、压缩率与原文 ref+digest… | CT-PROJECT | 可机械覆盖 |
| 69 | 1 | 房间可维护一份滚动纪要（前情提要）：挂在（room、cursor）上、由组装器机械触发并经 small-brain 增量折叠的派生缓存。未配置 small-brain 时不生成纪要，物化端以近详远略裁剪代替（近期消息全文、更早消息降为标题加事件指针）。纪要逐条携带消息事件回源指针… | CT-PROJECT | 可机械覆盖 |
| 73 | 1 | Artifact 是 Project/Repo 中可引用、评审和交付的稳定身份；普通 Git 文件在登记前不是 Artifact。Artifact Revision 至少固定 `artifact_revision_id`、artifact_id、不可变内容定位、内容摘要、可选 C… | CT-TASK | 可机械覆盖 |
| 79 | 2 | Room Invocation 的合法边只有待启动 → 运行中/失败/已取消/丢失、运行中 ↔ 等待输入，以及运行中/等待输入 → 完成/失败/已取消/丢失。执行身份无法证明时进入丢失；撤销租约、提交 stop/fence outbox、迟到结果只留审计等动作由[连接合同的统一丢… | CT-TASK | 特别 tricky |
| 81 | 1 | Room Invocation 的 Execution Spec 除[连接合同定义的共同字段](./connections.md#project--run--agent从授权到物理执行)外，还固定 scope（repo_scope ｜ project_scope）与 human … | 无 | 其余 |
| 87 | 2 | Request 的完整跨模块字段合同只在[连接合同](./connections.md#跨模块-request-回路)定义；本模块不另建一套同义字段。活动 Request 的问题、目标人或角色、`owner_ref + affected_revision_ref + blocke… | CT-PROJECT | 可机械覆盖 |
| 95 | 2 | 普通 Room 的临场执行边只能由可稳定归属到 human 的动作在 Trigger Preview 后提交。动作可以来自 Workbench/CLI 的 direct client connection，也可以来自 Chat 端口绑定明确允许的 provider 结构化事件；两… | CT-CONNECTION | 可机械覆盖 |
| 97 | 1 | mention 的解析必须确定性：`@` 目标只按获准的 Participant/Role 绑定精确解析；无唯一授权候选时必须明确失败或要求人选择，不得按显示名模糊匹配、静默换人或把 mention 字符串交给模型猜测路由。 | CT-RUN | 特别容易犯 |
| 99 | 1 | 命令走 HCTL，记录落平台：类型化命令的预览、准入与判决都在控制面执行；结果可以作为结构化事件写回 chat server 供时间线展示。普通消息、反应、成员变化和自动化只是 content，不能触发派发、解决 Request 或改变治理事实。唯一可提升为 human 命令请求… | CT-PROJECT | 可机械覆盖 |

### docs/design/spec/run.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 31 | 1 | Run 合法边固定为：启动中 → 运行中/失败/已取消/被替代；运行中 → 暂停中/取消中/完成/失败/被替代；暂停中 → 已暂停/取消中/失败/被替代；已暂停 → 运行中/取消中/失败/被替代；取消中 → 已取消/失败/被替代。每个过渡态都必须能通过取消、失败或替代进入终态，不… | CT-TASK | 可机械覆盖 |
| 33 | 1 | Run 是反应式状态机，但所有输入不共用一条无类型事件通道。human 的 Start/Pause/Resume/Cancel/Request answer 先进入公共 command service；Agent 的结果先进入 Result Proposal；timer 与 En… | CT-PROJECT | 可机械覆盖 |
| 35 | 1 | Dagu 原生 UI/API 的 Start/Stop/Retry/Reschedule/Approve/Reject/Edit/Rename/Delete 会直接改变定义或机械执行，不能在副作用前携带 HCTL command envelope、Run expected ver… | CT-RUN | 特别容易犯 |
| 37 | 1 | `运行中 → 完成` 不是通用写入口，只能由确定性 reducer 在同一预览版本上证明以下正常完成谓词后执行：冻结 Workflow Revision 的全部 required Obligation、Seat、Gate 与声明输出均已在账本中以精确 subject 和 Evid… | CT-TASK | 可机械覆盖 |
| 39 | 2 | 任何失败、取消或替代终态在释放 Task Run claim 前，也必须在同一事务撤销旧 dispatch、输入/写租约与外部副作用资格，并提交 runtime stop/fence；若只能撤销逻辑权威而无法证明旧进程已静默，则隔离旧 worktree/ChangeSet，后续执… | CT-RUN | 可机械覆盖 |
| 47 | 1 | Workflow Revision 使用 HCTL 规范化 JSON，经过数据结构、Profile 和语义校验后由工具箱写入/回读 Git；Git 保存不可变正文，control 账本独占 identity、admission、digest、approval/current po… | CT-TASK | 可机械覆盖 |
| 57 | 1 | 第一阶段，绑定 Task Revision 的 Run 表示对该完整 Task 验收合同的一次施工授权，因此只有它正常完成才具备提交 Task 完成命令的资格。只覆盖局部研究、咨询或中间步骤的自动化必须使用无 Task Run 或 Room Invocation，并以稳定引用把结… | CT-TASK | 可机械覆盖 |
| 61 | 1 | 「启动 Run」必须 CAS 活跃 Project/version、可选 Task 的开放 lifecycle/current Revision 及该 Task 的 Run claim，并在同一用户级账本事务创建 Run、不可变 Manifest、幂等结果、`active` Ta… | CT-TASK | 可机械覆盖 |
| 63 | 1 | 「替代 Run」不是先取消再另起：同一事务校验旧 Run/version，撤销旧 runtime、输入/写租约与 owner-specific fence，把旧 Run/Obligation/Seat/Attempt 置为被替代并提交 stop/fence outbox，同时创建… | CT-RUN | 可机械覆盖 |
| 65 | 1 | 运行中只有 Manifest 明确声明为可变的放置参数可以按冻结规则和边界调整；每次调整都校验预期 Run version，并留下固定前后值、适用规则、actor 和 Run version 的不可变审计事件。范围、验收、候选、权限、Gate 或超出获准边界的放置变化必须创建替代… | CT-RUN | 可机械覆盖 |
| 67 | 1 | 第一阶段 HCTL Profile 允许外部执行、fork/join、switch、loop、dynamic fork、timer wait、noop 和经审计的纯数据转换；先以 schema、引用、Profile 和图结构 lint 拒绝格式或结构不合法的 Workflow R… | CT-RUN | 可机械覆盖 |
| 85 | 1 | Run 需要输入时向 Project 提交类型化 [Request](./project.md) 创建命令，只阻塞声明的范围；Project 独占 Request lifecycle。Request 冻结 deadline 与 `fail ｜ cancel` 默认策略；Resol… | CT-PROJECT | 可机械覆盖 |
| 97 | 2 | 只有冻结策略列明的类型化技术故障，例如候选特有的认证/配额/网络故障、进程或运行时丢失、租约超时，才可以切换 Attempt。control 先隔离当前代次，再在候选、预算和剩余截止时间允许时于同一 Seat 创建新 Attempt；候选耗尽后，需要额外输入或授权则创建 Requ… | CT-RUN | 可机械覆盖 |
| 99 | 2 | Gate 是 Run 内由 Workflow Revision 与 Run Manifest 冻结的治理节点/规则，不是独立模块。它的每个 Seat 绑定同一精确 ReviewSubjectRef、review-policy ref+digest、根 Context Manife… | CT-TASK | 可机械覆盖 |
| 101 | 4 | 第一阶段这只证明**逻辑 Participant 分离与 producer/reviewer 分离**，不证明背后是不同人类 operator、公司、模型提供方、基础模型或 post-train。Manifest/Gate policy 应冻结 provider/model/op… | CT-RUN | 可机械覆盖 |
| 105 | 1 | Run 终态只说明 Workflow 到达经 HCTL reducer 确认的终点，不直接改写 [Task](./task.md)。绑定精确 Task Revision 的 Run 只有满足上述正常完成谓词后，完成事务才把该 Task 的 claim 从 `active` CAS… | CT-TASK | 特别容易犯 |
| 113 | 1 | ｜ Workflow Revision ｜ Dagu 的 YAML DAG definition / BPMN 的 process definition ｜ 引擎产物不能反向定义它；它先于任何引擎存在 ｜ | CT-TASK | 可机械覆盖 |
| 116 | 1 | ｜ Engine 外部检查点 ｜ Dagu 的 processless `human.task` / Camunda 的 external task 模式 ｜ Dagu 名称虽含 human，在 HCTL 中只是路标：control 观察其等待态铸 Obligation，账本结果… | CT-RUN | 可机械覆盖 |

### docs/design/spec/system.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 38 | 1 | Workbench 的 HCTL 功能只依赖 Query/Preview/Submit/Subscribe 和模块投影，不依赖 Vikunja、Dagu 或 Herdr 的 UI 对象；它的 provider 客户端功能则使用 provider 的公开协议或客户端侧 transp… | CT-RUN | 可机械覆盖 |
| 40 | 1 | 第一阶段全部执行面服务（chat server、本地任务服务器、Workflow Engine、Herdr）的管理/API 端点只绑定 loopback 或 owner-restricted local socket。未来非本地 transport 必须认证客户端；需要 HCTL… | CT-RUN | 可机械覆盖 |
| 44 | 1 | 每个扩展绑定都冻结代码版本、接口/schema、配置摘要、依赖、能力和信任级别。运行中不得因“发现更好的插件”而响应式改绑；提供方消失时安全暂停、失败或创建替代执行。 | CT-CONNECTION | 可机械覆盖 |
| 46 | 3 | `trust_level` 只能由 control policy 根据允许的 trusted source 与精确 artifact digest 授予，扩展或 registry 的自我声明不能授信。discovery 只读取已配置 definition 和本地安装并执行无副作用… | CT-CONNECTION | 可机械覆盖 |
| 53 | 1 | 同一 `(port_kind, scope_id)` 的一次准入只能解析出一个 binding revision；提供方加载顺序、hook 优先级或 UI 选择顺序不得决定事实。Room 的 Chat 端口绑定、Task Binding、Engine Deployment、Eng… | CT-RUN | 可机械覆盖 |
| 57 | 1 | 跨 Project 使用的 Skill 是带稳定 ID、revision 和 digest 的共享定义，至少固定 manifest/instructions/assets/scripts、来源/license、兼容能力与依赖；更新创建新 revision，current poin… | CT-CONNECTION | 可机械覆盖 |
| 85 | 1 | ｜ 执行结果提议 ｜ harness/Agency 的结构化终局事件与证据 ｜ 只能进入 Result Proposal；owner 模块按版本、代次和证据准入，不能冒充 human 命令 ｜ | CT-AGENT | 特别容易犯 |
| 88 | 1 | 一个 provider 事件可以同时具有 content 含义和命令请求含义。例如把已绑定 Vikunja 卡片拖入 Done 已经改变了 provider content；若事件还能证明是配置中映射的 human 所做，并携带稳定外部实体、前后 revision/updated… | CT-TASK | 可机械覆盖 |
| 107 | 1 | actor source/provenance 由 direct client connection、provider binding 的账号映射或 control 内部 reducer 赋予，调用 payload、Room 消息、Harness 进程和 adapter 都不能自… | CT-PROJECT | 可机械覆盖 |
| 109 | 1 | 治理命令只有两类 actor 来源：可稳定归属到 owner human 的动作，以及 task-bound Run 正常完成后由 control 内部 reducer 提交的「完成 Task」命令。human 动作既可以来自 Workbench/CLI 的 direct cli… | CT-TASK | 可机械覆盖 |
| 111 | 2 | control 在用户级 metadata 账本的一个 SQLite 事务中写领域事件、幂等结果和 outbox；跨模块命令也只能使用这一个事务边界，不能由两个模块或两个 clone 事后拼接。外部适配器按同一 key 投递并回读；超时或 ACK 丢失保持“结果未知”，不能盲目重… | CT-CONNECTION | 特别 tricky |
| 113 | 1 | 组件或 content 系统不可用不得降低命令前置：与该系统无关、且所需引用已冻结的 metadata 命令可继续；验收策略、字段权威或冲突前置要求 fresh readback 时，不可用或 cursor gap 必须返回类型化拒绝。只有冻结策略明确允许 cached/stal… | CT-CONNECTION | 可机械覆盖 |
| 115 | 1 | Receipt 证明的是已经校验的结果，不是另一个 writer。投影可以从事件重建，缓存或界面状态不能反向成为事实。 | CT-SYSTEM | 可机械覆盖 |
| 117 | 1 | 跨模块引用的规范摘要统一使用 RFC 8785 JCS 规范对象的 SHA-256，摘要字段自身不参与计算；每个领域 owner 只定义自己规范对象包含哪些字段。完整 Revision 的 revision_digest 与为评审选取字段生成的 review_subject_di… | CT-TASK | 可机械覆盖 |
| 121 | 1 | 包括远端 SCM 在内、会改变第三方权威事实的动作统一写成持久外部副作用命令/outbox 记录（executor = adapter），固定 owner ref、Resolved Port Binding、operation、target、adapter 声明的 conflic… | CT-CONNECTION | 可机械覆盖 |
| 125 | 1 | 第一阶段不承诺自动补偿任意外部写：provider 事件只有在对应模块明确列为 content、human 命令请求或运行时输入时才按该路径处理；其余修改由对应端口或工具箱回读为 Snapshot/drift，并阻止依赖旧版本的命令，直到用户通过该模块既有的采纳或对账动作处理。H… | CT-TASK | 可机械覆盖 |
| 133 | 1 | 「挂接 Repo Instance」命令先由工具箱无副作用读取 Git identity，再由 control 预览并写入账本。相同 Git common-dir 重试返回原现场；不同现场只有在 Git 中的稳定 Repo identity 与命令指定的 `repo_id` 一致… | CT-SYSTEM | 可机械覆盖 |
| 141 | 1 | 账本保存 HCTL 自己的领域关系、授权与判决，以及 HCTL 身份到外部 content/runtime 身份的跨系统锚定；它不复制承载系统内部的完整拓扑（如任务后端里与 HCTL 无关的卡片层级）。控制面凭获准命令、精确映射与 Snapshot 对账需要治理的那部分外部关系。… | CT-TASK | 可机械覆盖 |
| 160 | 1 | - **判决的结晶副本**（metadata 的审计影子）：Verdict/Receipt 的权威在用户级 metadata 账本产生并保存；副本由工具箱写入 Git，用于审计与随仓库同步。副本不是第二权威——从 Git 回灌只能恢复可验证、已结晶的判决候选，仍须显式恢复流程确认… | CT-RUN | 可机械覆盖 |
| 170 | 1 | ｜ 四模块 metadata：稳定身份、准入/current、Room/Request、Participant/角色绑定、权限、租约、代次、现场记账、Run Manifest、Execution Spec、Result Proposal 准入与 Verdict/Receipt ｜… | CT-PROJECT | 可机械覆盖 |
| 171 | 1 | ｜ Task/Workflow Revision、Memo、Artifact/ChangeSet Revision 的不可变正文与 Repo 共享 policy/Skill/schema revision；Verdict/Receipt 审计影子 ｜ 正文字节在 Git，由工具箱… | CT-PROJECT | 可机械覆盖 |
| 181 | 1 | 每个 Repo Instance 的 Git/worktree 资源由 `hctl2-tool` 取得 `<git-common-dir>/hctl2/` 下的 OS 排他锁，并由唯一 control 在账本 CAS 推进该现场的 `site_generation`；这是外部资源… | CT-AGENT | 可机械覆盖 |
| 183 | 1 | 每个 Agency binding scope 在账本中同时只有一个 owner lease 和单调 generation；scope 至少覆盖同一 Herdr server/socket/host namespace。新 owner 必须先对账，HCTL 不再向旧 genera… | CT-AGENT | 可机械覆盖 |
| 199 | 1 | UI 重载只重建投影。无法证明同一执行身份时，宁可标记丢失或要求人工对账，也不能自动接管或伪造成功。 | CT-AGENT | 可机械覆盖 |
| 203 | 1 | metadata 备份必须是由唯一 writer 协调的一致备份集：完整账本快照，连同账本引用的精确用户级 Profile/Skill/Runtime 不可变定义字节与 digest——后者存放在账本之外，单备份账本文件会漏掉它们。secret value、可丢弃 cache、P… | CT-TASK | 特别容易犯 |
| 205 | 2 | 恢复只能在旧 writer 已停止且取得用户级排他锁后进行；不得合并两份分叉账本或把备份恢复成新的账本身份。恢复保留原 ledger identity，推进 control writer 及所有可能仍存活的 site/backend generation，令旧 descripto… | CT-AGENT | 可机械覆盖 |
| 210 | 1 | - 打包后的桌面壳固定最小权限面，WebView 只暴露具名 typed command：Tauri 2 按 window/webview 以 capability/permission/scope 显式声明，不开放未声明的 IPC 与插件能力；以 Electron 安全网形态发… | CT-PACKAGING | 可机械覆盖 |
| 213 | 1 | - 日志与 trace 使用稳定关联 ID，但不得包含密钥和完整敏感 payload。 | CT-AGENT | 特别容易犯 |

### docs/design/spec/task.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 18 | 1 | Task lifecycle 只有开放 ｜ 完成 ｜ 已取消。第一阶段 `project_id` 是 Task 稳定身份的一部分，创建后不可改写；契约变化创建新 Task Revision，高频操作变化经受控端口写入 content 后端，回读为 Task Binding 的操作… | CT-TASK | 可机械覆盖 |
| 20 | 2 | Backlog ｜ Ready ｜ In Progress ｜ Review 是操作投影中的本地非终态 stage，不是 Task lifecycle。Blocked 与需要关注是从 blocker、Request、Run、来源同步和验证事实派生的正交 health，不能覆盖 s… | CT-PROJECT | 可机械覆盖 |
| 26 | 1 | Board 与 Project 分组不是新聚合；其稳定锚定保存在 Repo 的 task-source 绑定元数据中，至少固定 `repo_id + board_scope_stable_id + project_id + group_kind + group_anchor_st… | CT-CONNECTION | 可机械覆盖 |
| 28 | 2 | 看板卡片是 content，粒度由后端自由承载（子任务、清单、微卡不受 HCTL 约束）。只有稳定落在恰好一个已准入 Project group anchor 下的规范卡片，才可 claim 一个 HCTL Task 身份；未分组、同时落入多个 Project group 或 a… | CT-TASK | 可机械覆盖 |
| 30 | 2 | Task Revision 冻结验收合同，不冻结施工步骤；其不可变正文与 locator/digest 在 Git，账本保存稳定 identity、准入与 current pointer。后端与关联来源的变化都先成为 Snapshot；其中会改变 Task Revision 契约… | CT-TASK | 可机械覆盖 |
| 32 | 1 | 每个外部规范实体在用户级控制面账本内使用 (provider, account_stable_id, external_entity_kind, immutable_external_entity_id) 持久映射到一个 HCTL Task；该唯一键不含端口绑定、scope 或 … | CT-CONNECTION | 可机械覆盖 |
| 34 | 3 | Task 有两条可恢复的创建路径。HCTL-first 的「创建 Task」先在用户级账本事务固定 `task_id + immutable project_id`、规范命令 digest，并提交必需的后端卡片 create outbox；仅当命令同时携带已预览的初始 Task … | CT-CONNECTION | 可机械覆盖 |
| 36 | 1 | 外部卡随后漂移到另一 Project group、同时出现在多个 group 或脱离原 group，只追加 Snapshot 并把原 Task 标为需要关注；在恢复原 placement 或按下一句建立新 Task 前，它阻止 Adopt、Start、Complete 和后端操作… | CT-TASK | 可机械覆盖 |
| 38 | 2 | task_source 端口绑定与 Task Binding 的本地 current projection 使用 control 维护的单调 state_version 做 CAS；Task Source Snapshot 另行保存 provider 的 remote revis… | CT-TASK | 可机械覆盖 |
| 50 | 1 | 所选 task backend 的事件还可以承载 human 命令请求，但只对 Task Binding 明确列明的动作生效。第一阶段只允许“已绑定规范卡片由映射到 owner human 的账号从非终态进入 Done”归一为「完成 Task」command draft；adap… | CT-TASK | 可机械覆盖 |
| 61 | 1 | ｜ Task Source Snapshot ｜ append-only sequence + remote revision/digest/cursor；可产生待采纳 ｜ control 持久化 refresh/reconcile 观测；「采纳契约」命令才消费契约变化；同一 p… | CT-TASK | 可机械覆盖 |
| 64 | 1 | 每个 Task 在账本中至多有一个 task-bound Run claim，状态为 `active ｜ completion_pending`。「启动 Run」必须在创建 Run/Manifest 的同一用户级账本事务把空 claim CAS 为 active；已有任一 cla… | CT-RUN | 可机械覆盖 |
| 66 | 1 | 「完成 Task」命令校验当前 Revision、验收规则、候选、Artifact/SCM/CI 和必需 Receipt，并对影响契约的待采纳默认拒绝（fail-closed）：actor 必须先采纳并按新 Revision 重新验收，或显式选择“按当前冻结 Revision 完… | CT-TASK | 可机械覆盖 |
| 68 | 1 | Task 终结只有两个获准 actor 来源：owner human 的 Task 命令请求，或绑定精确 Task Revision 的 Run 正常进入完成后由 Run reducer/control 机械提交同一个「完成 Task」命令。human 请求可以来自 Workbe… | CT-TASK | 可机械覆盖 |
| 70 | 1 | Task Completion Receipt 至少固定 Task、「完成 Task」命令、Task Revision ref+digest、验收策略，以及每一条验收项各自的 pass/fail、Evidence/Verdict/Receipt ref+digest、来源 sna… | CT-TASK | 可机械覆盖 |
| 74 | 1 | 「重开 Task」命令只接受有权 human actor，必须以预期 task_lifecycle_version 把完成/已取消 → 开放并推进版本；它不复活旧 Receipt。若当前来源契约已有未处理 drift，重开预览必须先采纳新 Task Revision 或显式冻结继… | CT-TASK | 特别 tricky |
| 80 | 4 | 「启动 Run」命令预览必须列出会影响当前 Task Revision 的全部待采纳，并要求 actor 明确采纳、拒绝或延期。采纳会先产生新 Task Revision，再以新 Revision 重做「启动 Run」命令预览；拒绝或延期必须随准入冻结当前 Revision 和精… | CT-TASK | 可机械覆盖 |

### docs/design/task.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 22 | 2 | - 契约变化产生新的契约版本，不原地改写；活动 Run 引用的契约版本永不漂移，改约必须先结束或替代该 Run。历史版本、Run 和凭证永不改写或物理删除。 | CT-TASK | 可机械覆盖 |
| 26 | 2 | - 任何适配器、Harness 或执行体都不能绕过完成命令写完成凭证，也不能冒充人或归约器提交。 | CT-TASK | 特别容易犯 |
| 42 | 1 | 简单工作不需要先画 Workflow，也不伪造只能由 Run 产生的 Gate（评审关卡）凭证；契约要求 HCTL 内部独立评审时必须使用带 Gate 的 [Run](./run.md)，接受外部 SCM 评审时则引用可回读的精确外部证据。需要持久重试、候选切换或 Gate 时，… | CT-TASK | 可机械覆盖 |
| 54 | 1 | ｜ 角色 ｜ 可以做什么 ｜ 不能做什么 ｜ | 无 | 其余 |
| 60 | 1 | 普通移动和排序归任务后端，怎么做并发控制也是后端的事，HCTL 按后端能力写入、以回读为准；进入 Done 后是否另产生完成请求按 binding 能力与[合同附录](./spec/task.md)处理。没有 Workbench 时，任务后端的原生界面可以继续修改 content… | CT-TASK | 可机械覆盖 |

### docs/design/vision.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 97 | 1 | ｜ 塑形（Planning / Shaping） ｜ 人是意图和授权中心；系统可以研究、建议、汇总，但不能替用户决定目标 ｜ 规格、ADR（架构决策记录）、Task、Artifact、Workflow 提案，或普通 Git 文件 ｜ | CT-RUN | 可机械覆盖 |
| 145 | 1 | 4. **人的角色是意图与授权中心。** 系统可以并行研究和建议，但不能替用户决定目标；Run 只能在批准的边界内自动推进。 | CT-RUN | 可机械覆盖 |
| 149 | 1 | 8. **证据高于进度（evidence over progress）。** Harness 的进度、自述结论、屏幕状态和外部平台的关闭态都不能越过验收与 Receipt。 | CT-TASK | 特别容易犯 |
| 153 | 1 | 12. **外部事实按字段授权。** Linear/GitHub 可以拥有明确配置的操作字段，但不能接管 HCTL 的身份、契约、验收或语义完成。 | CT-TASK | 可机械覆盖 |
| 155 | 1 | 14. **客户端没有等级。** Workbench、CLI 和未来客户端调用 HCTL 时走同一命令与查询服务；它们操作 provider content/运行时时也遵守与原生客户端相同的模块合同。界面不能直接写治理账本，无法保持先记账再执行顺序的 Engine mutatio… | CT-RUN | 特别容易犯 |

### README.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 93 | 1 | - Project、Task、Run 和 Agent 模块的对象都有稳定身份，不能由聊天串、外部 Issue、工作流任务、worktree 或终端面板反向定义； | CT-RUN | 可机械覆盖 |
| 150 | 1 | 动作是否可用取决于模块合同和 provider 的实际能力，而不是客户端名称。Vikunja 中明确把已绑定卡片移入 Done 可以在保留操作者、版本和幂等依据时转成同一个「完成 Task」请求，但仍由 Task 独立验收；普通 Matrix 消息仍只是消息，只有显式且已配置的结… | CT-PROJECT | 可机械覆盖 |

### docs/usage.md

| 行 | 次 | 原句（截断至 140 字符） | 机械覆盖 | 初分 |
| --- | --- | --- | --- | --- |
| 10 | 1 | ｜ `hctl2-tool` ｜ P1 骨架 ｜ HCTL2 开发者 ｜ 显示英文帮助和版本；尚不能执行 Git/SCM 操作 ｜ | 无 | 其余 |
| 141 | 1 | 这些网络服务只监听 loopback，不对局域网或公网开放。Cinny 是官方 Web 发行包的静态内容，由随包的官方 `static-web-server` 单二进制提供；其 Homeserver 固定为 `http://127.0.0.1:6167`，不能改连任意服务器。它主… | CT-PACKAGING | 可机械覆盖 |
| 224 | 1 | macOS arm64 与 Intel 必须分别在对应架构的 macOS 15+ Mac 上原生构建和测试；不能只在 Apple Silicon 上交叉编译 Intel 包，因为 Tuwunel 原生构建、架构专属官方二进制、Mach-O 闭包和运行验证都属于目标合同。 | 无 | 其余 |

## 附录：CT 族关键词映射（机械覆盖列的生成规则，族级）

- CT-PROJECT：`加密|E2EE|端到端`、`纪要|萃取|压缩|Bundle|small-brain|消息|时间线`、`Scoped Room|归档|升格|Memo|Request`
- CT-TASK：`Done|完成|lifecycle|关闭|Reopen|Deleted|tombstone|墓碑`、`排序|移动|placement|泳道|看板|Kanban|Snapshot|快照`、`契约|采纳|adopt|Revision`
- CT-RUN：`Dagu|Engine|引擎|路标|human.task`、`Obligation|Seat|Attempt|候选|quorum|法定票|regate|Gate|返工`、`Workflow|Run |施工图`
- CT-AGENT：`租约|lease|代次|generation|fence|栅栏`、`迟到`、`凭据|密钥|credential|secret`、`沙箱|加固|隔离|worktree|工作树`、`合入|Integration|ChangeSet|集成`、`attach|接管|PTY|终端|转录|trace|观测|Herdr|Agency|harness`、`Proposal|提案`
- CT-CONNECTION：`outbox|ACK|回读|readback|幂等|digest|摘要|binding|绑定|换绑`
- CT-SYSTEM：`writer|写者|账本|SQLite|备份|恢复|JCS|规范摘要`
- CT-PACKAGING：`trust|discovery|install|安装|升级|打包|发行`
- CT-WORKBENCH-IA：`Workbench|预览|IME|键盘`

按序首个命中即归类；无任何命中记 `无`。

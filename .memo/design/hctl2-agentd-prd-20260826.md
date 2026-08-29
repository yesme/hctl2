# hctl2-agentd 产品需求

> 状态：讨论中<br>
> 基线：main @ 84424cd（草案 v0.13.1）<br>
> 去向：待定：docs/design/delivery.md 的 agentd 交付合同，或独立的 agentd 交付文档<br>
> 追记（2026-08-29，v0.14.0）：agentd 已退场——本 PRD 的 AGD 条款待按 内置 Agency（hctl2-agency）/ 控制面网关 / 现场保管（工具箱） 三类重新归类，见 `.memo/design/provider-20260829.md`。<br>
> 说明：讨论稿 · 2026-08-26
> 范围：定义 `hctl2-agentd` 要交付的产品能力，不定义进程结构、通信协议、存储布局、代码模块或具体实现方法。
> 上位约束：以 `docs/design/agent.md`、`docs/design/spec/agent.md`、`docs/design/delivery.md` 为准；本文不新增治理对象或改写权威边界。

## 结论

`hctl2-agentd` 是一台执行主机上的物理运行时负责人。它把已经获准的一次执行变成真实的 Harness 进程和可观察的终端通道，并负责在客户端断开、重新连接或自身重启后诚实说明现场还剩什么。

第一阶段必须做到：

1. 发现本机 Harness 及其实际能力；
2. 启动、列出、观察、输入、打断和停止一次物理执行；
3. 支持多个观察者、一个输入者和显式接管；
4. 区分“接回同一进程”“用厂商会话重新启动”“只能看历史”和“现场已丢失”；
5. 对输出缺口、能力缺失、身份过期、孤儿进程和不确定的停止结果明确报错；
6. 只上报物理事实和 Harness 观测，不判断 Task、Run、评审或交付是否完成。

它不是控制面、Git 工具箱、Workflow Engine、终端模拟器或 Workbench 后端的替代品。

## 1. 产品目标

### 1.1 用户目标

- 用户启动一次 Harness 后，可以关掉 CLI 或 Workbench，执行仍继续。
- 用户重新打开任一合规客户端时，可以找到同一次执行并接回；若不能精确接回，系统必须说清楚能恢复到哪一级。
- 用户可以先只读观察，确有需要时再取得输入权；接管不会产生两个同时输入的人。
- 用户能够知道本机装了哪些 Harness、版本是什么、每项能力是否真实可用，而不是从产品名推断能力。
- 用户能区分“当前模型在工作/等待/需要输入”与“上层任务已经完成”。前者是观测，后者不由 agentd 决定。

### 1.2 平台目标

- 为 `hctl2-control`、公共 `hctl2` CLI 和 Workbench 提供同一组物理执行能力，不产生 UI 私有会话。
- 为每次执行保留精确物理身份，使旧连接、旧输入权和迟到事件不能作用于替代后的新现场。
- 对不同 Harness 使用逐项能力声明和准确降级，使新增 Harness 不需要伪装成既有 Harness。
- 在 macOS 与 Linux 上提供一致的产品语义；平台差异只能表现为明确的能力可用性差异。

## 2. 使用者与典型任务

| 使用者 | 典型任务 |
| --- | --- |
| 本地开发者 | 查看本机 Harness、启动一次独立诊断执行、观察输出、输入、打断、停止、检查残留现场 |
| `hctl2-control` | 按已经准入的执行规格激活或停止物理运行时，授予或撤销终端输入权，接收物理观测 |
| 公共 CLI 用户 | 查看执行状态、只读 attach、取得输入权后接管、查看 replay、运行 doctor |
| Workbench 用户 | 在 Execution Chat 或终端视图中观察同一执行，显式接管输入，断线后继续 |
| Harness 适配器 | 报告安装、版本、原生会话和结构化事件能力，执行实际支持的语义操作 |

P1 尚无 control 时，开发者可以使用独立诊断入口验证这些机械能力。诊断执行必须清楚标为“不受 HCTL 治理”，不能生成 Task/Run 完成、Receipt 或集成权限。

## 3. 产品边界

### 3.1 agentd 拥有

- 本机 Harness 的安装与能力观测；
- 一次物理执行的进程、终端通道、存活状态和主机侧观测；
- 终端的快照、实时流、replay 可用性和缺口说明；
- 输入、窗口尺寸、打断、终止等物理动作的执行；
- 外部 Harness 会话引用的发现与物理续接结果；
- 客户端断开后的现场保持、重连和物理对账；
- 自身负责的物理动作与异常的诊断记录。

### 3.2 agentd 不拥有

- Repo、Project、Task、Run、Room、Participant、Seat 或 Attempt 的领域状态；
- 工作是否正确、是否通过评审、是否完成或是否允许合并的判断；
- Git 提交、分支集成、PR、push、worktree 创建或删除；
- 写租约、输入租约或连接票据的治理签发；
- Chat、Kanban 或 Workflow 后端的内容与生命周期；
- Workbench 页面状态或任一客户端的私有会话目录；
- Harness 的安装、升级、登录和账号凭据管理。

## 4. 功能需求

### 4.1 主机与 Harness 目录

**AGD-001 · 主机健康**
用户可以查看 agentd 是否可用、所在主机与平台、产品版本、运行时后端状态、当前执行数量和需要关注的故障。

**AGD-002 · Harness 发现**
用户可以查看本机已识别的 Harness、实际程序位置、实际版本、是否可启动和最近一次探测结果。重复名称或来源不明的程序必须明确显示，不能静默选一个。

**AGD-003 · 逐项能力**
每个 Harness 绑定分别报告以下能力是否可用：新建会话、精确接回、原生会话恢复、结构化事件、发送新提示、当前轮 steer、follow-up、打断当前轮、终止进程、读取厂商会话引用、结果终局事件和权限请求。未知、未探测与明确不支持是三种不同结果。

**AGD-004 · 能力准确降级**
用户请求 Harness 不支持的操作时，agentd 返回具体缺失能力及仍可用的替代操作；不得把普通终端输入包装成原生语义操作，也不得因“支持 ACP”或存在 Session ID 就假定全部方法可用。

**AGD-005 · 目标 Harness 范围**
P1 对 Codex、Claude Code 和 OpenCode 完成安装与能力探测。OpenCode 是第一阶段完整原生适配目标。所有经当前发布矩阵列出的 CLI Harness 至少可以按真实终端程序启动、观察、输入、重连和停止；额外原生能力按探测结果逐项开放。

### 4.2 物理执行生命周期

**AGD-010 · 启动**
agentd 可以按一份精确的执行请求启动新运行时。用户能看到使用的 Harness、程序版本、工作目录、启动时间、执行来源、实际启用的运行约束和稳定的运行时身份。

**AGD-011 · 列出与查看**
任一合规客户端都能列出 agentd 已知的活动、停止中、已退出、失联和待对账运行时，并查看同一份事实。列表不能依赖创建它的客户端仍然在线。

**AGD-012 · 并发隔离**
多个运行时可以并行存在。一个运行时的输入、尺寸、输出拥塞、退出或清理不得改变另一运行时。

**AGD-013 · 正常退出**
Harness 进程退出后，用户可以看到退出时间、退出码或信号、退出前最后观测、是否仍有子进程、是否还有可读历史以及是否存在厂商会话引用。

**AGD-014 · 停止**
停止操作必须区分：打断当前轮、请求进程正常终止和强制拆除物理现场。若无法证明相关子进程已经结束，结果显示“停止未收敛”及残留信息，不能报告为已停止。

**AGD-015 · 幂等操作**
重复的启动确认、停止、打断、释放输入权和客户端重试不会产生第二个运行时或把同一物理动作执行到替代后的新现场。

### 4.3 终端观察

**AGD-020 · 首屏快照与实时流**
观察者连接后先得到当前可用终端画面或历史锚点，再连续收到新输出。输出保持原始字节语义，未知编码或控制序列不能被静默改写成另一份内容。

**AGD-021 · 多观察者**
同一运行时允许 CLI、Workbench 和其他获准客户端同时只读观察。加入或离开观察不改变 Harness 的终端尺寸、输入权或存活状态。

**AGD-022 · 断线重连**
观察者断开后可以凭精确运行时身份和已看到的位置继续。agentd 必须返回以下三种结果之一：无缝续接、以新快照重新锚定、存在明确输出缺口。不得把缺口隐藏在看似连续的历史中。

**AGD-023 · 慢观察者隔离**
一个暂停读取或处理过慢的观察者不能阻塞 Harness、输入者或其他观察者。该观察者需要丢弃历史时，只对它报告重新锚定或缺口。

**AGD-024 · Replay**
已退出或已丢失的运行时只要仍有历史，就可以只读查看。Replay 必须显示覆盖范围、是否完整、来源运行时和终止原因，不能冒充 live attach。

**AGD-025 · 终端尺寸**
只有当前输入者或明确获准的尺寸控制方可以改变运行时尺寸；只读观察者的窗口变化不影响正在执行的全屏 TUI。

### 4.4 输入、接管与安全输入

**AGD-030 · 观察权与输入权分离**
能看终端不代表能输入。用户可以只读观察，也可以在取得输入权后发送按键、文本、粘贴、窗口尺寸和终端信号。

**AGD-031 · 单输入者**
一个运行时同一时刻最多有一个输入者。第二个客户端请求输入时必须显式接管或失败，不允许两个输入流交错。

**AGD-032 · 显式接管**
接管成功时，旧输入者立即失权并收到通知；旧客户端后续发送的输入被拒绝。只读观察者不受影响。

**AGD-033 · 精确目标校验**
每次输入和物理控制都指向精确的运行时及其当前代次。运行时被恢复、替代或重新创建后，旧连接与迟到输入必须失败，不能仅按名称命中新现场。

**AGD-034 · 人类操作入口**
经认证的 `hctl2` CLI 或 Workbench 操作即视为人在操作；agentd 不额外要求 nonce、challenge、键鼠轨迹或“人在场证明”。它只校验当前操作是否具有相应的观察、输入或控制权限。

**AGD-035 · 安全输入**
敏感输入使用与普通终端输入分开的能力。用户能够确认它送达哪个精确运行时；它不得出现在环境变量、普通终端历史、Room、Context、trace 或 replay 中。

### 4.5 Harness 原生会话

**AGD-040 · 双重身份可见**
产品同时显示物理运行时身份和 Harness 原生会话引用。两者不能互相替代：同一厂商会话可以由新进程恢复，同一物理进程也可能尚未产生厂商会话引用。

**AGD-041 · 新建与继续**
用户可以要求开始新厂商会话，或在当前仍存活的物理运行时中继续同一会话。若 Harness 支持原生提示、steer 或 follow-up，产品按实际语义分别提供，不合并为一个模糊的“send”。

**AGD-042 · Semantic resume**
原进程已经消失但厂商会话可恢复时，用户可以启动新的物理运行时继续该会话。产品必须明确标示这是新进程、新物理代次，不是 exact attach。

**AGD-043 · 禁止重复恢复**
原厂商会话已经有可确认的活动物理进程时，agentd 不得再次恢复出第二个并行进程。所有权无法确认时进入待对账，不以“试试看”创建副本。

**AGD-044 · 登录与交互墙**
遇到登录失效、浏览器授权、工作区信任、权限确认或其他需要人的交互时，产品显示具体阻塞原因和可执行的恢复动作。无法安全自动回答的提示不得无限重试或被误判为运行中。

**AGD-045 · 终局结果契约**
支持结构化结果的 Harness 必须报告所需终局事件。进程以零退出但缺少终局事件时，产品显示协议不完整；由 HCTL 主动取消导致的退出显示取消，不显示为 Harness 失败。该结果仍只是 Result Proposal 或观测，不是 Task/Run 完成。

### 4.6 恢复与对账

**AGD-050 · 客户端退出不停止执行**
CLI、Workbench 或网络连接退出后，运行时继续存在；重新连接的客户端可以在统一目录中找到它。

**AGD-051 · 恢复等级**
agentd 重启或与运行时后端重连后，对每个已知执行给出且只给出一个恢复等级：

1. `exact attach`：同一 PTY 和同一进程仍存活；
2. `semantic resume available`：原进程已失，但厂商会话可由新进程恢复；
3. `replay only`：不能继续执行，只能读取保留历史；
4. `lost`：既无可继续现场，也无足够历史。

恢复等级不能用模糊的“restored”统称。

**AGD-052 · 物理现场优先对账**
恢复时以实际仍存在的进程、终端和运行时后端为准。仅凭旧客户端列表或旧状态不能把已经消失的现场显示为活动，也不能在原现场仍活着时重建副本。

**AGD-053 · 无界面会话可见**
通过 CLI、control 或 Workbench 创建的运行时必须出现在其他客户端的统一目录中。创建成功但无法被列出或 attach，属于产品故障，而不是 UI 同步延迟的正常状态。

**AGD-054 · 孤儿与外来现场**
agentd 能识别“由本执行现场创建但失去上层绑定”的孤儿，以及同一后端上的外来会话。孤儿可以查看、对账、停止或保留；外来会话默认不被接管，也不能被自动清理。

**AGD-055 · 终止意图恢复**
若停止过程在确认前中断，恢复后仍显示为待收敛，并继续允许回读和人工处理；不能因 agentd 重启把“正在停止”改回正常活动，也不能假定已经停止。

### 4.7 运行观测

**AGD-060 · 物理观测**
agentd 报告进程存活、退出、前台程序、输出推进、终端标题、工作目录、尺寸、输入者、子进程残留和外部会话引用等主机事实。

**AGD-061 · Harness 语义观测**
若有结构化协议或原生 hook，可报告工作中、空闲、等待用户、权限请求、工具调用、文件变化、测试、用量和原始未知事件。仅从屏幕或标题推断的结果必须标明较低可信度。

**AGD-062 · 来源与时序**
每条观测都带来源、观测时间、运行时代次和在该来源内的位置。迟到、重复或属于旧运行时的观测不得覆盖当前事实。

**AGD-063 · 完整性**
观测流可以完整结束，也可以明确标记截断；不能在上报通道永久失败后继续交付一段看似完整的历史。Harness 内部派生执行保留父子来源，不能摊平成主执行的一串事件。

**AGD-064 · 不越权解释**
`idle`、`waiting`、`blocked`、`done`、零退出码、厂商 Session 完成或终端停止都不能自动推进 Project、Task、Run、Verdict、Receipt 或 Git 集成。

### 4.8 执行边界与安全

**AGD-070 · 按声明施加约束**
执行请求声明的工作目录、环境、权限和可选加固必须逐项核对。宿主无法满足已声明项时拒绝启动并列出缺项；未声明的可选加固不阻止启动，也不能被记录为已经启用。

**AGD-071 · Harness 凭据边界**
Harness 可以使用自己的模型登录凭据，但不能获得 HCTL control、人类身份、目标分支集成、远端 SCM、Chat、Kanban 或 Workflow 后端的写凭据。

**AGD-072 · 正常 Git 可见性**
Harness 在获准 worktree 中可以正常读取 Git common-dir、refs、日志和目标分支，并在自己的 ChangeSet 分支提交。agentd 不把这些正常 Git 能力误判为越权，也不把绕过集成流程的 ref 改写认作 Integration Receipt。

**AGD-073 · 清理不丢工作**
agentd 不自行判断 worktree 是否可以删除。物理拆除可能触及唯一代码副本时，必须等待工具箱或 control 的明确可清理结论；无法确认就保留现场并给出恢复信息。

**AGD-074 · 本机访问边界**
默认只有当前操作系统用户可访问 agentd 的执行与终端能力。发现访问边界异常时，停止接受新输入并在 doctor 中显示阻断故障。

### 4.9 诊断与可运维性

**AGD-080 · Doctor**
用户可以一次查看 agentd、运行时后端、Harness 安装与版本、当前运行时、异常 socket/进程、孤儿、恢复等级及建议动作。Doctor 只读，不擅自修复或清理。

**AGD-081 · 稳定错误**
所有失败同时提供稳定错误码、简短的人类说明、精确目标和可执行的下一步。至少区分：Harness 未安装、版本不可识别、能力不支持、登录需要人、目标不存在、旧代次、无输入权、输出缺口、后端不可用、停止未收敛和执行约束不满足。

**AGD-082 · 物理动作审计**
启动、输入权变更、接管、敏感输入、打断、停止、强制拆除、恢复等级变化和孤儿处理都可追溯到发起来源及精确运行时。普通终端正文不因此自动进入治理账本。

**AGD-083 · 生命周期**
用户可以明确启动、查看、停止和升级 agentd。升级或重启不能静默杀死仍可保持的运行时；无法保持时必须提前说明影响，并在恢复后逐项给出恢复等级。

## 5. 第一阶段范围

### 5.1 P1 必须交付

- agentd 主机健康、Harness 目录和逐项能力探测；
- Codex、Claude Code、OpenCode 的探测，OpenCode 完整原生适配；
- 本机运行时的启动、列出、查看、输入、尺寸、打断、停止和退出观测；
- 首屏快照、实时流、多观察者、单输入者、显式接管和断线重连；
- 精确运行时身份与旧代次拒绝；
- exact attach、semantic resume available、replay only、lost 四级恢复；
- 客户端无关的统一运行时目录、孤儿识别和恢复对账；
- Doctor、稳定错误码和物理动作审计；
- macOS 与 Linux 原生分发中的一致语义；
- 独立诊断入口，且明确不产生治理结果。

### 5.2 P2 接 control 后启用

- control 签发的连接描述与输入租约校验；
- CLI/Workbench 的人类身份、观察权、输入权、执行控制权与安全输入权限分离；
- Execution Spec、Context Bundle、各层代次与 fence 的完整准入；
- Result Proposal 与结构化 Evidence 上报；
- ChangeSet 清理前与工具箱/control 的保全确认；
- 多个 agentd 作为不同 execution site 的登记与旧 site generation 拒绝。

### 5.3 第一阶段不做

- Windows 运行时；
- 多用户组织权限、云队列或多主机调度；
- 任意公网终端分享；
- Harness 自动安装、自动登录或账号托管；
- 原生 Session 批量导入和历史迁移；
- 自建终端渲染器、终端 GUI 或完整终端模拟；
- Git/worktree/PR/push/merge 实现；
- Task/Run/Workflow/Room 治理；
- 根据屏幕文本自动判断工作完成；
- 自动接管或清理非 HCTL 创建的外来 tmux 会话。

## 6. 出门验收场景

1. **客户端退出后继续**：从 CLI 启动 OpenCode 执行，关闭 CLI；进程继续。Workbench 或另一个 CLI 能列出并 exact attach 到同一进程。
2. **两个观察者、一个输入者**：两个客户端同时看到连续输出；只有一个能输入。第二个显式接管后，第一个的迟到输入被拒绝。
3. **慢客户端隔离**：一个观察者停止读取，Harness 和另一个观察者继续；慢观察者恢复时得到无缝续接、重新锚定或明确缺口之一。
4. **agentd 重启**：重启 agentd 后逐项列出原运行时；仍活着的精确接回，原进程丢失但会话可恢复的标 semantic resume available，其他按 replay only 或 lost 处理。
5. **禁止双开恢复**：厂商会话已有活动进程时，请求 semantic resume 被拒绝；无法确认所有权时进入待对账而不是再启一个。
6. **旧代次隔离**：停止旧运行时并创建替代运行时后，旧 descriptor、旧输入权和迟到事件都无法命中新现场。
7. **无界面创建可见**：control 或 CLI 创建成功的运行时立即可由所有合规客户端列出、查看并按权限 attach；不存在“后台活着但 UI 永远看不见”的会话。
8. **打断不等于停止**：打断当前 Harness 轮次后物理进程仍可继续使用；终止运行时后进程和子进程结果被单独回读。
9. **登录阻塞**：Harness 进入登录或浏览器授权墙时显示需要人的具体原因，不无限重试，也不显示为正常工作中。
10. **能力诚实**：对不支持 steer 的 Harness 请求 steer，返回缺失能力和可用替代项；不得静默改成普通 stdin。
11. **观测不越权**：Harness 报 `done` 或零退出只产生观测/Proposal；Task、Run、Receipt 和目标分支均不变化。
12. **执行约束**：声明的加固无法施加时启动被拒绝并列出缺项；未声明同一项时可以启动，且不冒充已加固。
13. **Git 正常可见**：Harness 能在自己的 worktree 读取 common-dir/refs 并在自己的分支提交；agentd 不因此拒绝运行。
14. **停止未收敛**：强制停止后仍发现残留子进程时显示未收敛和残留信息，不报告成功。
15. **不丢唯一代码**：没有收到可清理结论时，停止运行时也不删除可能含未封存改动的 worktree。

## 7. 参考项目源码带来的产品约束

| 来源 | 源码中已经出现的产品问题 | 本 PRD 的对应要求 |
| --- | --- | --- |
| [Superset PTY daemon](https://github.com/superset-sh/superset/blob/4e18e1fa794be7969d517bea86d082105e44c836/packages/pty-daemon/README.md) | PTY 应独立于前端服务存活；多订阅者、detach/reattach、replay、字节保真和进程退出是同一产品面；其源码也明确承认无游标 replay 会产生缺口 | AGD-020—024、050—055；不把“有缓冲”写成“历史必然完整” |
| [Stably Orca Session](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/session.ts) / [恢复适配](https://github.com/stablyai/orca/blob/09ec516ae50b7b83fa65343d9ad96159e3fe71fc/src/main/daemon/daemon-pty-adapter.ts) | 活动进程、冷恢复历史、新物理 incarnation、输出重新锚定和终止竞态必须分别处理 | AGD-014—015、042—043、051—055 |
| [Herdr headless server](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/src/server/headless.rs) / [Terminal 状态](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/src/terminal/state.rs) | 后台服务持有终端；观察与控制、原始终端输入与 Agent 语义命令、不同状态来源和过期序号都需要分开 | AGD-021、030—034、041、060—064 |
| [First Tree provider 契约](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/client/src/providers/README.md) / [tmux 会话控制](https://github.com/agent-team-foundation/first-tree/blob/19e66032af7f9f482168c350fe0b3998599388f3/packages/client/src/providers/claude/tui/tmux-session.ts) | Provider 能力必须逐项探测；登录墙、一次性信任提示、resume 菜单、重试分类、孤儿清理范围和终端输入保密都会进入真实运行路径 | AGD-002—005、034—045、054、081 |
| [Cumora BYOA daemon](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/computer/daemon.ts) / [engine](https://github.com/yetone/cumora/blob/bd8dba8e45c91f685ea3c319aae173d44d26cbd6/server/src/agents/computer/engine.ts) | send、interrupt、alive、原生 session、进程 fallback 和系统提示能力不是所有 Harness 都相同；会话重置必须保守 | AGD-003—004、040—045、052 |
| [Multica Agent backend](https://github.com/multica-ai/multica/blob/2c0912b6ec764b373d44eeea1e80f0d9f11ab417/server/pkg/agent/agent.go) | 多 Harness 需要能力矩阵和明确降级；失败、取消或清理不能吞掉已产生的代码 | AGD-003—005、073 |
| [Codeg terminal manager](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/terminal/manager.rs) / [ACP runtime](https://github.com/xintaofei/codeg/blob/df7a872de44546277e4c49cfe9d173c631161dc6/src-tauri/src/acp/terminal_runtime.rs) | 终端运行时与 ACP 会话是两种接入面；自定义 Agent 还会出现方法与 MCP 能力差异 | AGD-003—004、040—045 |
| [Termio ATP](https://github.com/termio-sh/termio/blob/d1fdac84046805d4056e082f982e6beb6072b61c/web/landing/content/docs/atp.mdx) / [SessionControl](https://github.com/termio-sh/termio/blob/d1fdac84046805d4056e082f982e6beb6072b61c/Sources/termio/Agents/SessionControl.swift) | 稳定会话寻址、监听、心跳、发送与信号是 Terminal-first 客户端的基本可操作面 | AGD-010—025、060—062；不采用其特定协议作为 HCTL 权威 |
| [OpenCode server](https://github.com/anomalyco/opencode/blob/31406ccc51b4bd2a4e1e086b2bcaa5f7f804f26d/packages/web/src/content/docs/server.mdx) | 原生服务端可向多个客户端提供 health、session、control、diff、permission 和事件能力，但这些仍是 Harness Session 能力 | AGD-003—005、040—045、064 |

## 8. 本 PRD 刻意没有定义的内容

以下内容应在 PRD 通过后进入实现设计，而不是提前写进产品需求：

- agentd 的进程、线程与模块拆分；
- 本地通信使用什么传输、消息格式和 API 路径；
- 状态放在内存、文件还是数据库；
- tmux server、session、window、pane 如何组织；
- replay 缓冲、游标、背压和终端快照如何实现；
- Harness adapter 的 Rust trait、crate 与目录结构；
- 守护进程如何安装、自启动、升级和交接文件描述符；
- 具体错误类型、字段名、schema 和测试框架。

这些实现选择必须服务于本文验收场景，不能反过来删减“准确恢复、单输入者、显式缺口、能力诚实和不越权解释”五条产品底线。

# P2.2 开工书：协作现场（戊 → 己 ∥ 庚 → 辛）——按 v0.18.11

> 状态：v1 · 待轻审（GLM、Muse）· 所有者 2026-09-20 让并行起草<br>
> 基线：main @ `36fc9d5`（草案 v0.18.11；P2.1 开工书 #271 与 CT 十行展开 #272 已合）<br>
> 去向：`src/apps/hctl2-control`、`src/apps/hctl2`、`src/crates/*`；本文只把 P2.2 四包按 v0.18.11 写到任务书粒度，不改约束层；`docs/design/delivery.md` 只在发现缺口时改<br>
> 读法：先读 [`04-p21-kickoff.md`](./04-p21-kickoff.md) §二（变化映射，本文只展开落到 P2.2 的行）、[`01-plan.md`](./01-plan.md) §四 B1、§六 戊己庚辛、§十二，再读本文 §四

## 一、定位与重述

P2.2 是协作现场（旧 B1）：Repo 注册（戊）、聊天端口与 Room（己）、任务源端口与 Task 影子（庚）、Project 与 Request（辛）。DoD 只有一行，`docs/design/delivery.md` §自举阶段 B1：**Room / Task / 草稿重启可恢复；引用稳定；明确不切换事实。** 「不切换事实」的意思：hctl2 自己的开发照旧，这一级只在试验仓库上跑。

P2.2 长在 P2.1 上：全部业务命令走甲的命令内核与两半存储，CLI 子命令挂在乙的骨架上，随包服务由丁拉起。四包在甲合入后开工；己另等 Grok 正在做的 chat 探针。任务源按 01 §十二 分两段：本阶段只接平台自带的 issues（GitHub 经 `gh`、本地平台 Gitea 经 `tea api`），本地任务服务器 Vikunja 推到 B2 之后、P2 出门前加绑。

本阶段没有派工：Room 名册记的是选入记录，候选是否满足 Agency 名册与 Project 选人策略的校验在 P2.3 壬接入 Agency 端口后生效；Room Invocation、Context 交付、Task 完成与凭证分别在 P2.3、P2.4。

## 二、v0.18.11 里落到 P2.2 的裁决

只列 04 §二 里标了戊、己、庚、辛的行，按包归类；出处同 04。

| 包 | 裁决 | 任务书里怎么落 |
| --- | --- | --- |
| 戊 | 注册三选一（外部平台 / 本地平台 / 显式不挂），由人登记与声明绑定，不写身份文件、不挂接工作副本（C 批）；本地 detach 默认另建独立副本、不换绑原 Repo 身份（Q2）；注册不建任何 Room（G 批 D4） | 注册命令与预览按 `spec/repo.md` §Repo 注册 逐句落；本地路径入口区分有 remote 与纯本地 |
| 戊、庚 | 缺省任务源在注册仓库时或该仓库第一个 Project 接入任务源时由人显式选定，缺省建议是平台自带的 issues；两层选择可在同一预览确认（G 批 C10、D 批） | 注册预览列候选并标「不能作缺省源、可认领」；Project 接入源引用是庚的命令，两处共用同一个候选清单与确认形状 |
| 己 | 每 Project 一间主 Room，Rooms 只列 Topic；「创建 Topic Room」带前情提要（两种来源：主 Room 消息；Request 及其阻塞对象）与人的确认，删减 / 补充 / 去敏进确认版本；自动归纳未配置时显式报告、人工补写可作替代；「关闭 Topic Room」即已归档，随 Project 归档转只读；房间不开加密、事后加密 fail closed 并可换绑（#257、G 批 C3、E 批） | 提要正文经甲存为治理材料，Room 记录只存引用与摘要；本阶段不做自动归纳，只做人工提要与「未配置」报告 |
| 庚 | Project 是独立 Namespace：实体到 Task 的映射唯一性含 `project_id`，同卡多 Project 各自认领、各自契约；一张卡一个家、认领不搬家；两套分组；获准映射稳定无歧义才自动认领；依赖归源投影四字段；「取消并归档」是缺省移除、删源卡另行确认并列本控制面全部绑定 Task；后端离线不放宽当前回读与版本比较前置（#257、D 批、E 批、G 批 E2、E3、C9、Grok-1、Grok-2、Q3） | 键、认领两路、Snapshot、依赖投影、取消与删卡预览按 `spec/task.md` §契约与来源、§写入约束 落；「完成 / 重开」留 P2.4 寅 |
| 辛 | 「发布评审须人显式确认」开关的缺省是 Project 版本化设置的一部分（G 批 A2）；Project 只持选人策略、不持成员名单，Room 名册是选入记录；归档拒绝清单收成一处、恢复语义（G 批 C1、C2）；Overview 保留为只读投影、不要求入口；「待你处理」五种来源、去重、处理后退出；Request 去重、取代、升级为 Topic；承接开放 Request 的 Topic 闲置 14 天提醒（#257、G 批 C4、E4、E 批） | `project update` 承载版本化设置；待你处理在本阶段只接两种来源（目标为本人的开放 Request；存在待本人采纳契约变化的 Task），另三种来源（发布评审意图、候选交付、Run 超时）在 P2.4、P2.5 接入同一投影 |
| 庚、癸 | Room Invocation 采纳契约与 Task 命令都核目标归属同 Project（G 批 GLM-4） | 庚在「采纳契约」「创建 Task」准入核目标归属；调用侧在 P2.3 |
| 登记，不在 P2.2 | 集成事实可接受本控制面另一 Project 完成（D6，寅）；候选交付进待你处理（寅）；Room Invocation 与前情提要的首轮材料交付（子）；汇总投影可选（不做） | — |

## 三、前置与探针

| 项 | 状态 | 影响 |
| --- | --- | --- |
| 甲（控制面存储与命令内核，Codex，P2.1） | 已合入 #276 | 四包的地基 |
| 乙（进程与 CLI 骨架，Grok，P2.1） | 已合入 #279 | CLI 子命令挂乙 |
| 丁（托管生命周期，Grok，P2.1） | 已合入 #282（前置 #278 已合） | `hctl2 start` 已带起 Tuwunel 与 Gitea；服务备份是停进程后拷数据目录，己 / 戊 消费时可按需换在线备份 |
| chat 探针（Grok） | 已完成 #274 | 己的前置；复核记录已追加到 `docs/research/sdk/matrix.md`，全部通过，B1 三项未验 |
| GitHub Issues 写侧运行验证（`docs/research/sdk/github.md` 09-17） | 已完成 | 庚的 GitHub 一侧 |
| Gitea issues 调用面（`docs/research/gitea.md` 09-17，`tea api`） | 已完成 | 庚的本地平台一侧、戊的本地平台建仓 |
| 1a Matrix / Vikunja 复核（09-06，钉定源码核对） | 已完成 | Vikunja 只在本阶段之后加绑 |
| Agency 探针 | 未做 | P2.3 前置，不挡 P2.2 |

## 四、四包任务书（v0.18.11）

分工沿 01 §四：Codex 与 Grok 主写，Fable 与 GLM 主审；每包一个 PR，两份独立评审到齐、修正项改完后由原作者合。四包都只做「影子」：Task 不完成、Room 不派工、不把 hctl2 自己的开发切到 HCTL2 上。

### 戊 · Repo 注册（Codex）

- **目标**：人显式登记仓库与平台绑定，三选一，待确认 → 活跃，一个 Repo 可被多个 Project 引用。
- **范围**：「注册 Repo」命令与预览：人声明来源与平台（外部平台 / 本地平台 / 显式不挂），平台服务实例与平台仓库稳定标识由人声明、远端地址只作辅助证据、冲突时预览列出供人确认；待确认注册先记账再走外部步骤 outbox——本地平台由有权限一方建仓（`tea api`）、持 Git 凭据的单元交付初始代码并回读，初始推送范围固定（没有提交只建仓；默认只推当前 HEAD 所在分支；其余 ref 显式选；私有保管引用与未发布治理材料不推）；确认事务激活 Repo，重试复用原记录；本地路径入口区分有 remote 与纯本地，有 remote 时让人选接原 remote 或另起独立工作，另起时默认新副本、保留输入目录及其 remote，原地切换须显式预览确认；放弃注册时已建平台仓库记残留由人清理；注册时给缺省任务源候选清单（与庚共用形状），采用缺省建议仍须人确认；`hctl2 repo register|list|show`。**不做**：Room（归辛经己）、Repo Instance 与工作副本挂接（已撤）、平台换绑之外的迁移。
- **依据**：`spec/repo.md` §对象、§写入约束、§Repo 注册、§平台绑定与能力声明；`spec/system.md` §外部权威副作用；`spec/task.md` §契约与来源（缺省任务源）；`docs/research/gitea.md`、`sdk/github.md`、`sdk/git.md`。
- **失败用例（CT-REPO 现行行）**：同一注册命令重投返回原 Repo、出现第二份登记失败；平台标识缺失不能完成绑定、远端 URL 不代替、辅助证据冲突由人确认；外部平台仓库绑定或换绑到本地平台拒绝；显式不挂走受限路径不因远端证据改判；本地目录入口区分有 remote 与纯本地——读取失败或路径不在指定机器当作纯本地失败、有 remote 未让人选就 detach 失败、另起独立工作改了原目录或 remote 失败、原地切换未经确认失败；只在本地的仓库缺省绑定本地平台、只推当前 HEAD 分支、没有提交只建仓、不配置工作副本 remote 也能登记；待确认 Repo 不接受 Project / Task / Run；注册建 Room 失败；结果未知按原关联键回读不重复建仓，平台不可用保持待确认。
- **依赖**：甲；丁（Gitea 生命周期，本地平台路径）；乙（CLI 骨架）。

### 己 · 聊天端口与 Room（Grok）

- **目标**：control 以 AppService 接 Tuwunel，Project 主 Room 与 Topic Room 的建、绑、读、关。
- **范围**：AppService 注册与虚拟用户（ruma）；建房不开加密、绑定前回读加密状态；Room–Server Binding；按事件 ID 读正文、事务 ID 幂等写入、断线后带游标重同步；治理引用只按事件 ID 冻结、引用时冻结摘要；「创建 Topic Room」命令与预览——两种来源（本 Project 主 Room 的 Message；本 Project 的 Request 及其冻结的阻塞对象与版本），提要按缘起与目标、已定事实与决定理由、分歧与待答、所需约束与材料、来源列清、已定与未定分开，预览可删减 / 补充 / 去敏，确认版本与摘要冻结进命令，提要正文经甲存为治理材料；自动归纳未配置时显式报告、人工补写替代；「关闭 Topic Room」；主 Room 后续消息不自动进 Topic；chat server 不可用或房间事后加密时依赖当前回读的命令 fail closed、标需要关注，换绑到未加密房间恢复；Matrix 房间升级换 ID 后换绑不改 Room 身份；bridge bot 同形事件拒绝为 human 来源；`hctl2 room list|show` 与 Topic 的创建 / 关闭子命令（命名 for agent）。**不做**：Room Invocation 与派工（P2.3）、Context 交付（子）、自动归纳。
- **依据**：`spec/project.md` §Room 与消息、§Repo 注册与 Project 归档（创建 Topic Room 段）、§场景约束；`spec/connections.md` §Room–Server Binding；`spec/system.md` §安全策略面「房间隐私与保留」；`docs/research/sdk/matrix.md` 复核记录、`matrix-homeserver.md`；chat 探针结果。
- **失败用例（CT-PROJECT 现行行）**：普通 Topic 因未填完成条件或结案理由不能创建或关闭失败；关闭 Topic 后关联 Request 被解决、Task 被取消失败；前情提要缺正文或精确来源、把未决写成已定、复制整段主 Room 历史或继承原授权失败；创建预览的删减 / 补充 / 去敏未反映到确认版本失败；主 Room 新消息自动流入已建 Topic 失败；以同 Project 的精确 Request 与冻结阻塞对象创建 Topic 应通过、因缺主 Room 消息拒绝或补造消息来源失败；自动归纳未配置却报已完成失败；chat server 不可用时依赖当前回读的命令 fail closed、不依赖的已接纳事实照常；Room–Server Binding 只接受未加密房间，事后加密 fail closed 并标需要关注，换绑恢复；普通消息、反应或自动化不能成为命令；同一 Matrix 动作两条路径生成相同命令摘要，HCTL 服务或 bridge bot 的同形事件拒绝；Matrix 房间升级换 ID 后换绑不改 Room 身份、旧引用与 digest 仍可校验；CJK 输入 / 结构化引用 / 草稿游标未读 / 并发流隔离行与 Room 历史可恢复行（#272 展开版）。
- **依赖**：甲；乙；丁（Tuwunel）；chat 探针。

### 庚 · 任务源端口与 Task 影子（Codex）

- **目标**：以平台自带的 issues 为任务源，把卡片变成 Task 影子：接源、认领、建卡、采纳契约、移动、取消，观测对账，不完成。
- **范围**：`port_kind = task_source` 的 Port–Provider Binding，两家后端——GitHub Issues 经 `gh`、Gitea issues 经 `tea api`（能力声明按各自实测：Gitea 有 `content_version` 条件写入，GitHub 没有、以回读为准）；仓库缺省任务源的候选与确认（与戊共用形状）；Project 显式接入源引用（绑定与获准范围），只开聊天室可不接源，接入第二个源不拒；板范围 `repo_id + board_scope_stable_id + binding_revision`；可选原生分组映射（milestone / 获准标签）与锚点，无原生能力不伪造；Task Backend Snapshot 的 refresh / reconcile（显式刷新与定期对账，不依赖公网 webhook）；认领两路——获准无歧义映射的自动认领、人的「认领卡片」；HCTL-first 创建 Task（明确选本 Project 已接入且能建卡的源，建卡 outbox 与关联键回读，不按 Room 名字猜源）；「采纳契约」（Task Revision 正文经甲存为治理材料，每条验收项带校验等级，采纳与创建准入核目标归属同 Project）；「更新 Task」经受控端口写回并回读；「移动 Task」只改阶段与排序、跨源拒绝；「取消并归档」；「删除源卡片」是另一个确认的 content 动作，预览列本控制面全部绑定 Task 及各自 Project、生命周期与活动 Run 引用，不替其他 Project 作决定；依赖归源投影四字段；家所在的源停用后 Task 保留标需要关注；实体到 Task 的映射唯一性含 `project_id`，同卡多 Project 各自认领；`hctl2 task create|update|adopt|move|cancel`。**不做**：「完成 / 重开 Task」与凭证（寅，P2.4）、Vikunja、Linear、跨源同步、汇总投影。
- **依据**：`spec/task.md` §对象、§契约与来源、§写入约束、§启动 Run 的前置与排序令牌（当前回读前置）；`spec/connections.md` §Project → Task：从讨论到承诺；`docs/research/sdk/github.md`（09-17 写侧）、`gitea.md`（09-17）、`task-backends.md`。
- **失败用例（CT-TASK 现行行）**：任务源按仓库绑零到多个——未显式同意就绑源或建卡失败、启用看板的 Project 无源引用失败、接入第二个源被拒失败、只开聊天室被要求选源失败、只过身份与快照的绑定被选作缺省源失败、缺省建议变化搬卡失败；认领分两路——适配器自选 Project、无映射或有歧义仍认领失败，人显式认领无锚点的卡被拒失败，认领把卡搬到缺省源、改写实体键、同 Project 产生第二张 Task 失败，认领后拿外源分组改写归属或补建分组失败；按源看板强制合并、未认领卡不可见、读取失败显示成空板失败；同一控制面两个 Project 分别认领同一源卡各得自己的 Task，第二方被拒、复用同一 Task、复制外部卡、共享契约失败；A、B 同绑一张卡的观测与写回规则；分组锚点稳定、获准映射不能稳定回读仍自动认领失败、只有稳定标签也能自动认领应通过、首次自动认领正例、认领后源内移组改写归属失败（缩写句以 CT-TASK 原行为准）；取消并归档未通过前置就移出、删源卡未确认目标与后果、活动 Run 无处理选择、删除未知报成功、删卡当 Run 已停失败；A、B 同绑且 B 有活动 Run 时 A 的删卡预览漏 B 失败；无契约 Task 的看板终态只是投影；验收项缺校验等级采纳预览失效；非法 move（改归属、Revision、lifecycle、跨源、过期 `state_version`）失败；Task 依赖归源四字段；local `state_version` 与 remote revision 不混用；本地 adoption 不伪造 Task–Backend Binding；外部卡只改 stage 或 health 时 lifecycle 与 Revision 不变；Task / 契约创建前正文已存而准入前崩溃只产生一个 Revision、建卡确认丢失按关联键回读不再建卡。
- **依赖**：甲；戊；乙。

### 辛 · Project 与 Request（Grok）

- **目标**：Project 的创建、更新、归档、恢复与版本化设置；Request 的创建、去重、取代、解决与升级；待你处理与 Overview 的只读投影；B1 收口。
- **范围**：「创建 Project」关联一个已激活 Repo、同一事务经己建唯一主 Room、同一命令重试返回原 Project 与 Room、另一条命令可为同 Repo 建第二个 Project；`project_version` 与版本化设置（目标、范围、角色、默认规则、选人策略、「发布评审须人显式确认」开关缺省）；Room 名册作为选入记录（字段按连接约束；候选校验在 P2.3 生效）；「归档 Project」静止前置——列出非终态 Run、写入型 Invocation、活动租约、待投递或结果未知的意图（本阶段这些对象多数还不存在，但检查与列表形状要在）——成功后 Project 与主 Room 只读、拒绝清单按权威句、开放 Task / Request / 未归档 Topic 随之只读；「恢复 Project」按同一句恢复语义；Request：拥有阻塞事实的模块提交类型化创建命令、Project 独占生命周期、阻塞身份相同去重到现有活动 Request、归属者或版本或范围变化时创建新 Request 取代旧的、解决经预览与类型化动作并以比较并交换校验来源 blocker、写唯一 delivery outbox；Request 应答面按需升级为 Topic（经己）；承接开放 Request 的活跃 Topic 闲置超过缺省 14 天投影「需要关注」，不增待处理计数；「待你处理」投影本阶段接两种来源（目标为本人或本人角色的开放 Request；存在待本人采纳契约变化的 Task），按来源引用与原处理动作去重，条目列对象、原因、动作与后果、未处理影响、返回入口，处理后退出、别的客户端处理随之更新；Overview 只读投影（目标、健康度、Task / Request 计数与近期活动，不要求入口）；`hctl2 project create|list|show|update|archive|restore`、`request list|show|resolve`；B1 收口端到端：建 Repo → 建两个 Project（同 Repo）→ 各自主 Room → 各开一间 Topic → 各接同一源、各认领同一张卡 → 采纳契约 → 杀 control 与服务 → 重启 → Room / Task / 草稿与引用一致。**不做**：派工与 Invocation（P2.3）、发布评审意图与候选交付来源（P2.4）、Run 超时来源（P2.5）。
- **依据**：`spec/project.md` §对象、§写入约束、§Repo 注册与 Project 归档、§Room 与消息、§Request；`spec/connections.md` §跨模块 Request 回路、§Project / Run → Participant（选入记录字段）；`delivery.md` §运行默认值（闲置 14 天）。
- **失败用例（CT-PROJECT、CT-REPO 现行行）**：创建 Project 同时建唯一主 Room，重投多建、同 Repo 第二个 Project 复用第一间失败，未选 Participant 不能查看失败；人显式在同 Repo 开两个 Project 分别授权，共用或互相替代失败；归档前置阻塞列表能看见归 Repo 模块的租约与意图，仅开放 Task / Request / 未归档 Topic 不阻止归档、随归档只读、恢复后开放对象恢复接收命令、已关闭 Topic 随恢复可写失败；Room 的 Project 归属或消息所属 Room 被引用动作改写失败；同根因 Request 重复创建仍去重、Topic 讨论结论未提交原动作不解决 Request；待你处理按现有事项去重——同一 Request 计三次、缺四问任一项、普通进度或建议计入、阅读面板即解决失败，另一客户端处理后仍显示、处理失败却移除、历史从原处消失失败；待处理来源分别注入的两种本阶段来源各能进出列表，其他人待答与无权处理的不计；两间同样闲置 15 天的 Topic 只有承接开放 Request 的那间提醒、提醒不增待处理数、Request 已解决后不再提醒；`project_version` 更新不改写已接受的下游约束。
- **依赖**：戊、己、庚；乙。

### 顺序与并行

戊先（Project 要关联已激活的 Repo）；己与庚并行（己等 chat 探针；庚等戊的绑定）；辛最后收口。四包都等甲合入；CLI 子命令等乙的骨架。乙、丁与戊可以在同一时段推进——乙丁归 P2.1，戊是 P2.2 的第一包。

## 五、第一包任务书（给 Codex，指针版）

```
你在 yesme/hctl2 做 P2.2 第一包「戊 · Repo 注册」。任务书：main 上 .memo/design/p2-control-20260906/05-p22-kickoff.md §四 戊（先读同文件 §一、§二，与 04-p21-kickoff.md §二）。前置：P2.1 甲已合入、乙的 CLI 骨架可挂子命令、丁能拉起 Gitea——三者缺一先报，不绕。分支 codex/p22-repo-register，base main，一个 PR，不自合；评审席位 Fable 与 Grok 各自独立审，修正项改完后由你合、合前报所有者。规矩沿 04-p21-kickoff.md §五（Buck2 原生目标、rustc 1.98.0、新依赖先补 docs/research 对象文件、禁词、PR 三节、不放会话链接）。回报：PR 编号、分支、crate 与 target 清单、失败用例清单（对照 CT-REPO 各行）、没按任务书做的地方及原因。
```

## 六、待所有者定

1. 席位：**已定（所有者 2026-09-21）**——写代码尽量给 Codex，Fable 也可以写，Grok 做评审。四包戊、己、庚、辛缺省由 Codex 写，评审席位 Fable 与 Grok（Fable 写的包由 Codex 与 Grok 审；GLM 可作第三席）。代价：原来的「戊 → 己 ∥ 庚 → 辛」变成单线顺序，戊先、辛末，己与庚互不依赖、谁先由所有者按需要定。
2. Vikunja 加绑时机沿 01 §十二（B2 之后、P2 出门前），本阶段不接——请确认。
3. 前情提要在本阶段只做人工提要与「自动归纳未配置」的显式报告；自动选材与生成是接手清单留的实现设计题，另案——请确认不进 P2.2。

## 七、轻审怎么审本文

三样：§二 有没有把 04 §二 标给戊己庚辛的裁决漏掉或落错包；§四 四包的范围与失败用例是否忠于 `spec/repo.md`、`spec/project.md`、`spec/task.md` 原文与 CT 现行行，有没有把 P2.3 / P2.4 的活（派工、Invocation、完成与凭证、材料交付）提前塞进来；§五 指针版任务书够不够 Codex 直接接。不审 P2 计划本身——01 已拍板。

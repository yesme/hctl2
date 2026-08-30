# 全库文档大修:结构总图(S2 初稿)

> 状态:讨论稿 · 待 S1 四张清单核对 · 待 GPT 保真通过与 Grok 三问<br>
> 基线:main @ `37805fa`(草案 v0.15.0)<br>
> 去向:拍板后冻结为 `03-approved-plan.md`<br>
> 作者:Fable(施工图 S2);只出树、表、提纲,不写正文。判据编号:D1–D4 = 施工图 §7.1 死文四判据(D1 被转向取代 / D2 无权威对应 / D3 重复权威 / D4 无入口且非历史);N = §7.2 负例三留;G4 = 红线 4 删除护栏;S = 结构收束(§8.3)。凡标「待 S1 核对」的,以 S1 清单为准,本图不猜。

## 1. 目标文件树

原则:`docs/design/` 顶层不新增子目录;文件数净变化 **+1 −1**(新增 `contract-tests.md`,删除 `references/implementation-evidence.md`),其余全部原地收束。`docs/research/` 只动索引与单案位置,证据文件一个不删。

```text
README.md                         门户:一句话定位、目标架构图、三条阅读入口、当前基线
docs/usage.md                     使用说明:当前形态(CLI + 各 content 原生界面)怎么用
docs/design/
  README.md                       设计地图:四模块×四场景总表、对象关系、共同规则、文档纪律、支持文档索引
  vision.md                       愿景:为什么存在、失败模式、四阶段心智模型、目标体验、两种制度、最小内核、设计原则、不做什么
  architecture.md                 三面架构:三个面、场景与系统、供应商替换边界、4×3 矩阵、模块交接、数据丢失
  project.md / task.md / run.md / agent.md
                                  四模块设计正文:为什么存在、拥有什么、正确道路(关键规则)、场景、交接
  participant.md                  横切:谁在参与、七层拆分、专业化 Participant
  context.md                      横切:开工包怎么来(投喂三档、萃取压缩、前情提要、Run 内接力)
  delivery.md                     交付:范围、CLI、明确不做、实现阶段、切片、自举、选型判据、P0、打包、技术基线、未决
  contract-tests.md   【新增,需拍板 10】
                                  契约测试矩阵(CT-* 八族),从 delivery.md 拆出
  spec/
    README.md                     合同层总则:词汇分类、核心产品词、六族、三类数据、外部对齐原则、文件索引
    project.md / task.md / run.md / agent.md
                                  四模块合同:对象、写入合同、模块专属机制、外部概念对齐
    connections.md                四模块交接、Execution Spec、Request 回路、失败与恢复
    system.md                     组件、受控端口、动作分类、命令正确性、外部副作用、存储、单写者、恢复、安全边界
  references/
    glossary.md                   术语对照
    decision-history.md           来时路:只记转折;被取代章节折叠为一段
    (implementation-evidence.md   删除:3 行转发 stub,零引用 · D4)
docs/research/
  README.md                       收束:一张条目索引 + 按类别一句话导读;头部重组史缩为一句
  remote-control/                 新收 codex-remote-feishu.md(从根 remote-control.md 搬入)+ 观察清单并入本目录 README
  tmux-runtime.md / agentd-runtime-candidates-20260829.md / workbench-shell.md
                                  文首加「结论已被 §N 取代,证据继续有效」标注;不删不搬
```

每个文件一句话职责与「明确不负责什么」:

| 文件 | 职责 | 明确不负责 |
| --- | --- | --- |
| 根 README | 三分钟内让人知道这是什么、当前到哪、从哪读 | 设计基线细则、六个问题、目标体验(归 vision);任何合同措辞 |
| usage.md | 当前可用形态的操作说明 | 设计理由;不描述尚未交付的 Workbench 操作 |
| design/README | 索引与总表、共同规则概括、文档纪律 | 重述模块规则的完整版(模块正文负责) |
| vision | 为什么、什么体验、按什么原则取舍 | 对象/状态/命令定义;部署分层(architecture) |
| architecture | 部署与数据视角、供应商替换边界、丢失恢复的产品叙述 | 合同细则(system.md);实现选型(delivery) |
| 四模块设计正文 | 各自的为什么、拥有什么、正确道路、场景、交接方向 | 字段、状态机、写入者表(spec);跨模块共享机制(system/connections) |
| participant / context | 横切的为什么与分工 | 规则的权威定义(已在各 spec,只引用) |
| delivery | 交付什么、按什么顺序建、怎样证明;选型与 P0 | 重定义合同;契约测试矩阵(contract-tests) |
| contract-tests | 八族可观察行为的失败用例 | 描述状态机;新增合同(合同变更须先改 spec 再加用例) |
| spec/README | 词汇法与族规则、三类数据、对齐原则 | 历史归并/清扫记录(去向见拍板点 9) |
| 四模块 spec | 各自对象、写入合同、专属机制、外部对齐 | 共享机制的复述(system);交接字段(connections) |
| spec/connections | 交接、Execution Spec、Request 回路、失败可观察结果 | 模块内部状态机;幂等/outbox 算法(system) |
| spec/system | 共享机制唯一权威 | 任何模块领域状态 |
| glossary | 中英对照与一句话含义 | 语义(以模块文档为准) |
| decision-history | 转折为什么发生 | 当前规范;被取代方案的细节(折叠) |
| research/README | 索引与复用决策 | 定义 HCTL 语义 |

## 2. 三条阅读路径

**新读者**(想知道这是什么、为什么这样):
根 README → vision.md → design/README.md(地图)→ architecture.md → project.md → task.md → run.md → agent.md → participant.md → context.md → delivery.md 的「第一阶段范围」与「实现阶段」。每站一句话:定位与入口 → 为什么与原则 → 四模块×四场景总表 → 三面与三类数据 → 四个模块各自的正确道路 → 谁在参与 → 看到什么 → 交付什么、什么顺序。

**实现者**(要写 control / tool / adapter 代码):
spec/README.md(词汇法、六族)→ spec/system.md → spec/connections.md → 对应模块 spec → contract-tests.md 对应族 → delivery.md「实现阶段」「纵向切片」「自举阶段」→ research/README.md(固定版本与 footprint)。每站一句话:先学族规则 → 共享机制 → 交接与恢复 → 模块合同 → 失败用例 → 建什么、何时敢切事实 → 依赖版本。

**provider adapter 开发者**(要接一个新的 chat server / 任务后端 / 引擎 / Agency):
architecture.md「场景与系统」「避免供应商锁定」→ spec/system.md「固定内核与受控端口」「客户端动作与 provider 事件」「外部权威副作用」→ 对应模块 spec 的端口段与「外部概念对齐」表 → contract-tests.md 的 CT-CONNECTION 与对应模块族 → delivery.md「选型判据」「开工前限时验证」对应条目 → research 对应条目。每站一句话:替换边界在哪 → 端口能拥有什么、动作怎么分类 → 本模块要求什么、词怎么对齐 → 测试要过什么 → P0 要验什么 → 前人怎么验的。

## 3. 旧→新迁移表

处置:keep(保留,可微调措辞)/ merge(并入他处)/ split(拆出)/ rewrite(原地改写)/ delete(删除)/ fold-to-history(折叠为历史一段)。「去向」为空即原地。

### 3.1 根 README.md(181 → ≤190)

| 章节(待 S1 核对标题) | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 文首 IMPORTANT 基线说明 | keep | — | 当前到哪,门户必需 | — | |
| 一句话定位 / 问题陈述 | keep | — | 门户核心 | G4 | |
| 目标架构图 | keep | — | 唯一的全图;v0.15.0 已按无等级重画 | — | |
| 设计基线 bullets | merge | vision.md 设计原则 / design/README 共同规则 | 与两处逐条重复,门户只留链接 | D3 | 是(S 门户收束) |
| 六个问题 / 目标体验段 | merge | vision.md 目标体验 | vision 已有同段 | D3 | |
| 场景与客户端说明段 | rewrite | — | 缩为架构图注 + 指向 architecture | D3 | |
| 阅读入口 | rewrite | — | 改为三条阅读路径(§2) | S | |

### 3.2 docs/usage.md(235 → ≤240)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 含 agentd 的段落(待 S1 定位) | rewrite | — | agentd 已退场,改为 Herdr/control 现行形态 | D1 | |
| 其余操作说明 | keep | — | 使用说明不适用精简判据 | — | |

### 3.3 docs/design/vision.md(171 → ≤180)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 一句话定位 | keep | — | | G4 | |
| 为什么需要 HCTL2(五种失败模式) | keep | — | 解释层,不删 | G4 | |
| 四个阶段的心智模型 | keep | — | | G4 | |
| 目标体验 | keep(吸收根 README 六问段) | — | 成为唯一权威 | D3 | |
| 两种控制制度 | keep | — | run.md 同名节改为引用本节(见 3.8) | D3 | |
| 产品原生核心与架构最小内核 | keep | — | | G4 | |
| 三类数据 | rewrite | 缩为三句 + 指向 spec/README | 与 architecture、spec/README 三处定义 | D3 | |
| 设计原则(15 条) | rewrite | — | 第 14 条「客户端没有等级」保留;检查各条是否与共同规则重复,重复者留标题一句 | D3 | |
| 要解决什么,不解决什么 | keep | — | 与 delivery「明确不做」分工:这里是产品层,那里是第一阶段范围 | G4 | |
| 从这里读下去 | merge | 根 README 三条阅读路径 | 入口只在门户一处 | D3 | |

### 3.4 docs/design/architecture.md(99 → ≤100)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 三个面(含表与 Workbench 段) | rewrite | — | 第三段(Workbench 非特权前端、原生客户端不统一称只读)与 system.md「客户端动作与 provider 事件」重复,缩为一段 + 引用 | D3 | |
| 场景与系统 | keep | — | 系统角色名的权威定义 | — | |
| 避免供应商锁定 | keep | — | 替换边界唯一权威(spec/README 外部对齐原则末段改引用) | D3 | |
| 4×3 归属矩阵 | keep | — | | — | |
| 模块交接 | keep | — | connections 总表的产品投影,已声明 | — | |
| 数据丢了怎么办 | rewrite | — | 与 system.md「全系统事实权威地图」与 connections「失败与恢复」三处;本节留产品叙述,细则改引用;补「路标停更时新义务铸造暂停」半句(事实修正,不改语义) | D3 | |

### 3.5 docs/design/README.md(设计地图,100 → ≤100)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 四模块总表 | keep | — | | — | |
| 对象关系图 | keep | — | | — | |
| 场景客户端与受控端口 | rewrite | — | 四段中两段与 system.md 同题重复,留概括 + 引用 | D3 | |
| 共同规则 | rewrite | — | 每条一句;完成来源、临场边、单写者三条是承重,留 | N / G4 | |
| 文档纪律 | keep | — | 大修本身的依据 | — | |
| 支持文档 | rewrite | — | 加 contract-tests.md;删 implementation-evidence 指向 | S | |

### 3.6 docs/design/project.md(75 → ≤100)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 为什么存在 | keep | — | | G4 | |
| 模块拥有什么 | keep | — | | — | |
| 关键规则 | rewrite | — | 加密一条改为一句引用 spec/project;否定句按 N 处理 | D3 / N | |
| Room 类型 | keep | — | | — | |
| Chat Room 场景(功能列表、P2 段、角色表、桥接段、加密段) | rewrite | — | 加密段(整段)缩为一句 + 引用;桥接段与 architecture「避免供应商锁定」重复,缩为一句;角色表「不能做什么」列按 N 处理 | D3 / N | |
| 模块交接 | keep | — | 只列方向,已符合 | — | |

### 3.7 docs/design/task.md(69 → ≤100)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 为什么存在 | keep | — | | G4 | |
| 模块拥有什么 | keep | — | | — | |
| 关键规则 | rewrite | — | 否定句按 N;「完成只有两个来源」是承重,留 | N / G4 | |
| 无 Run 的轻量路径 | keep | — | 正确道路的正面叙述,典范 | G4 | |
| Kanban 场景(角色表、Board 不说谎段) | rewrite | — | 角色表「不能做什么」列按 N;其余留 | N | |
| 模块交接 | keep | — | | — | |

### 3.8 docs/design/run.md(65 → ≤100)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 为什么存在 | keep | — | | G4 | |
| 模块拥有什么 | keep | — | | — | |
| 两种控制制度 | merge | vision.md 同名节(留一段「顺序才是关键」的 Run 特有部分) | 与 vision 同题 | D3 | |
| 关键规则 | rewrite | — | Dagu 界面一条与 spec/run 重复,缩为引用;否定句按 N | D3 / N | |
| Workflow 场景(角色表、Dagu 控制台段) | rewrite | — | 角色表按 N;Dagu 段与 spec/run 重复,缩一句 | D3 / N | |
| 模块交接 | keep | — | | — | |

### 3.9 docs/design/agent.md(91 → ≤100)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 为什么存在 | keep | — | | G4 | |
| 模块拥有什么 | keep | — | | — | |
| 关键规则 | rewrite | — | 三条底线一条改为人话一句 + 引用 spec/agent;否定句按 N | D3 / N | |
| Terminal 场景(能力表、P2 段、Execution Chat 段、角色表) | rewrite | — | 角色表「场景客端」错字;WezTerm 一行按 S1 裁决;「不能做什么」列按 N | N / D1 | |
| Agency 与 Herdr(接口表、v0.8.2 限制清单) | rewrite | — | 限制清单四条整段移除,改为「能力按 binding 声明,当前缺项见 delivery P0」一句 | D3 | 是(拍板点 11) |
| 原生会话导入 | keep | — | | — | |
| 模块交接 | keep | — | | — | |

### 3.10 docs/design/participant.md(54 → ≤60)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 为什么存在 | keep | — | | G4 | |
| 七件事分层 | keep | — | | — | |
| 关键规则 | rewrite | — | 五条改为引用各 spec 落点,只留一句主旨 | D3 | |
| 专业化 Participant | keep | — | 解释层 | G4 | |
| 模块交接与合同落点 | keep | — | | — | |
| 版本戳 v0.14.1 | rewrite | — | 改 v0.15.1 | D1 | |

### 3.11 docs/design/context.md(113 → ≤120)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 为什么存在 | keep | — | | G4 | |
| 与相邻概念的分工 | keep | — | | — | |
| 投喂三档 | keep | — | 解释层的中心 | G4 | |
| 萃取与压缩 | keep | — | | G4 | |
| 前情提要 | keep | — | | G4 | |
| 同一 Run 内的接力 | keep | — | | G4 | |
| 关键规则(九条) | rewrite | — | 与 spec/project Context 节逐条对应,改为引用 + 每条一句 | D3 | |
| 场景 | keep | — | | — | |
| 模块交接与合同落点 | keep | — | | — | |
| 版本戳 v0.14.1 | rewrite | — | 改 v0.15.1 | D1 | |

### 3.12 docs/design/spec/README.md(142 → ≤90)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 词汇分类法 | keep | — | | — | |
| 核心产品词 | keep | — | | — | |
| 六族规则 | keep | — | | — | |
| 三类数据 | keep | — | 三类数据的唯一权威(vision/architecture 改引用) | — | |
| 词汇索引 | keep | — | | — | |
| v0.9.1 归并对照 | fold-to-history | decision-history §11 尾部折叠段 | 核销记录,不是当前合同 | D1 | 是(拍板点 9) |
| v0.10.3 清扫 | fold-to-history | decision-history §12 尾部 | 同上 | D1 | 是(9) |
| v0.11.1 词形收敛 | fold-to-history | decision-history §14 尾部 | 同上 | D1 | 是(9) |
| v0.12.2 清扫 | fold-to-history | decision-history §20 尾部 | 同上 | D1 | 是(9) |
| v0.13.0 收窄 | fold-to-history | decision-history §22 尾部 | 同上;含「用户在场证明」「沙箱入场券」两行 | D1 | 是(9) |
| 外部对齐原则 | rewrite | — | 末段(受控端口隔离默认实现)与 architecture「避免供应商锁定」重复,缩为引用 | D3 | |
| 文件 | rewrite | — | 加 contract-tests 不在此(它归 delivery 侧),保持只列 spec | S | |

### 3.13 docs/design/spec/project.md(113 → ≤120)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 对象(表) | rewrite | — | Chat 端口绑定一行的加密前置改为指向「Room 与消息」 | D3 | |
| 写入合同(表) | rewrite | — | Chat 端口绑定一行同上;Room/治理事件一行留 | D3 | |
| Repo 注册与 Project 归档 | keep | — | | — | |
| Room 与消息 | rewrite | — | **成为加密前置与降级的唯一权威定义**(见 §4.1);其余否定句按 N | D3 / N | |
| Context、Memo 与 Artifact | keep | — | Context 合同唯一权威 | — | |
| Room Invocation | keep | — | 丢失规则已引用 connections,符合 | — | |
| Request | keep | — | | — | |
| 场景合同 | rewrite | — | 「命令走 HCTL,记录落平台」一段与 spec/README 三条法、system 动作分类重复,缩;否定句按 N | D3 / N | |
| 外部概念对齐 | rewrite | — | Room 一行的加密说明改一句引用 | D3 | |

### 3.14 docs/design/spec/task.md(98 → ≤120)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 对象 | keep | — | | — | |
| 契约与来源 | rewrite | — | 否定句密集(26 条全文),按 N 逐条;provider Done 信封段是承重,留 | N / G4 | |
| 写入合同 | keep | — | | — | |
| 「完成 Task」「终结来源」两段 | rewrite | — | 「Run 的裸终态、Harness 自述、Git commit、CI 绿色…都不是命令」一句为典型 N 候选:正确道路(两个来源)已写清,改为正面句 | N | |
| 启动 Run 的前置与排序令牌 | keep | — | | — | |
| 外部概念对齐 | keep | — | | — | |

### 3.15 docs/design/spec/run.md(119 → ≤120)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 对象 | keep | — | | — | |
| 写入合同 | keep | — | | — | |
| Dagu 原生 UI 段 | rewrite | — | 与 system.md 动作分类表 Run 行、run.md 设计正文三处;此处留合同句,其余引用 | D3 | |
| 正常完成谓词段 | keep | — | 承重 | G4 | |
| Workflow 与 Run 授权 / 启动与 Manifest | keep | — | | — | |
| 从节点到结果 | keep | — | | — | |
| Request、重试与 Gate | keep | — | 五路径表是典范 | G4 | |
| Run → Task | keep | — | | — | |
| 外部概念对齐 | keep | — | | — | |

### 3.16 docs/design/spec/agent.md(106 → ≤120)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 对象 | keep | — | | — | |
| 写入合同(表) | rewrite | — | Terminal Input Lease 一行的「Herdr 原生写入不受该租约约束」改为能力条件句 | D3 | 是(11) |
| 三条底线段 | keep | — | **成为三条底线唯一权威定义**(见 §4.2) | G4 | |
| ChangeSet 与 Git 事实 | keep | — | | — | |
| 运行时与观测 | rewrite | — | 「Herdr v0.8.2 的能力边界进入合同」整段改为四项能力条件句;栅栏回显段中 v0.8.2 点名改为条件句 | D3 | 是(11) |
| 终端通道、连接与租约 | rewrite | — | `native_interactive_allowed` 段中 Herdr 点名改条件句 | D3 | 是(11) |
| 外部概念对齐 | keep | — | Herdr 作为外部体系词合法 | — | |

### 3.17 docs/design/spec/connections.md(174 → ≤180)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 连接模型 / 连接图 / 总表 | keep | — | | — | |
| Project → Task | keep | — | | — | |
| Project / Task → Run | keep | — | | — | |
| Project / Run → Agent(启动顺序四步) | rewrite | — | 第 4 步「Herdr v0.8.2 不支持该能力」改为「未声明栅栏回显的 Agency…」 | D3 | 是(11) |
| Agent → Project / Run | keep | — | | — | |
| 验收与回流 | keep | — | | — | |
| 跨模块 Request 回路 | keep | — | Request 字段唯一定义处 | — | |
| 版本、权限与替代 | keep | — | | — | |
| 失败与恢复(表) | rewrite | — | 「已绑定房间被开启端到端加密」一行留(可观察结果唯一登记处),措辞改引用 spec/project | D3 | |
| 场景与第三方适配器 | rewrite | — | 与 system.md 动作分类重复,缩 | D3 | |

### 3.18 docs/design/spec/system.md(214 → ≤200)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 组件(表 + 段) | rewrite | — | 段落中「第一阶段由 Herdr 实现 Agency 端口」留;其余与 architecture 三个面重复处缩 | D3 | |
| 固定内核与受控端口 | rewrite | — | 第四段(Workbench 直连 Herdr、v0.8.2 writer gate)改条件句;第五段(loopback 端点)留 | D3 | 是(11) |
| 场景端口 | keep | — | | — | |
| 客户端动作与 provider 事件 | keep | — | v0.15.0 核心,唯一权威 | — | |
| 命令与跨服务正确性 | rewrite | — | 「第一阶段不设额外的用户在场证明」一句删(对已撤销机制的负述) | D1 | |
| 外部权威副作用 | rewrite | — | 三条底线复述改为引用 spec/agent(见 §4.2) | D3 | |
| 事实与存储(四小节) | keep | — | 存储唯一权威 | — | |
| 单写者 | rewrite | — | 两处 Herdr v0.8.2 点名改条件句 | D3 | 是(11) |
| 启动与恢复 / 备份与恢复 | keep | — | | — | |
| 安全边界 | rewrite | — | 末条三条底线复述改引用;壳安全条款留 | D3 | |

### 3.19 docs/design/delivery.md(290 → ≤220,含拆出)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 第一阶段范围(含 P2/P3 表、动作分类段) | rewrite | — | 动作分类段与 system.md 重复,缩为引用 | D3 | |
| 公共 CLI | keep | — | | — | |
| 明确不做 | keep | — | 与 vision「不解决什么」分工清楚 | — | |
| 实现阶段(P 表) | keep | — | | — | |
| 纵向切片 A / B | keep | — | | — | |
| Kanban content 后端切片 | keep | — | | — | |
| 自举阶段(B 表) | keep | — | | — | |
| 契约测试矩阵(八族) | split | contract-tests.md | 一文十一职的最大块;CT-PACKAGING 补 Tauri 壳中立用例(合同已改、测试未跟) | S | 是(10) |
| 选型判据 | keep | — | | — | |
| 开工前限时验证(五项) | rewrite | — | 已完成的探针核销为一行结论 + research 链接;Herdr 项保留 v0.8.2 缺项清单(这是它的唯一家) | S | |
| 打包策略 | rewrite | — | Tuwunel「从锁定源码原生构建」改为消费托管制品(事实修正);macOS 15 论证按 S1 核对结果改写 | D1 | |
| 技术基线 | keep | — | | — | |
| 未决问题 | rewrite | — | 三条已了结项(划线)删,只留开放项 | D1 | |

### 3.20 docs/design/references/glossary.md(142 → ≤145)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 全表(待 S1 核对) | rewrite | — | 删已退场词条(若有);新增词条只对应 v0.15.0 动作分类 | D1 | |

### 3.21 docs/design/references/decision-history.md(282 → 折叠后减少)

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| §1–§5、§7–§12 | keep | — | 转折记录 | — | |
| §6 Conductor 边界 | fold-to-history | 缩为「当时为何、被 §18 取代」一段 | 被取代 | D1 | 是(3) |
| §13 P/B 双表与 P0 选型 | fold-to-history | 双表部分留一句;选型段折叠 | 选型已全部改判 | D1 | 是(3) |
| §14、§15、§16、§17 | keep | — | | — | |
| §18 Dagu、§19 tmux | fold-to-history | 各缩为一段;tmux 官方二进制与 macOS 15 一段随 delivery 修正同步 | tmux 已退场 | D1 | 是(3) |
| §20–§26 | keep | — | | — | |
| §27 运行时 provider 三轮 | fold-to-history | 缩为一段,指向 §29 | 被 §28/§29 取代 | D1 | 是(3) |
| §28 中间方案 | fold-to-history | 缩为两句 | 明标已取代 | D1 | 是(3) |
| §29–§31 | keep | — | | — | |
| §32 小修订台账 | rewrite | — | 加 v0.15.1 大修一行;若拍板点 9 选「并入」,五张历史表挂到对应章节尾 | S | 是(9) |
| §33 当前设计 | keep | — | | — | |

### 3.22 docs/design/references/implementation-evidence.md

| 章节 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| 整文件(3 行 stub) | delete | — | 零引用 | D4 | |

### 3.23 docs/research(只动索引与位置)

| 对象 | 处置 | 去向 | 理由 | 判据 | 需拍板 |
| --- | --- | --- | --- | --- | --- |
| README 头部重组史括号 | rewrite | — | 缩为一句 | D4 | |
| README 五张表(①–⑧ 类别、L1 精选、L4 补充、运维表、条目索引) | rewrite | — | 条目索引为主索引,类别表缩为一句话导读;运维表留(唯一 footprint 权威) | D3 | |
| remote-control.md(Codex Remote Feishu + 观察清单) | split | remote-control/codex-remote-feishu.md + remote-control/README 观察清单 | 根目录只放跨候选归纳 | S | |
| tmux-runtime.md / agentd-runtime-candidates-20260829.md / workbench-shell.md | rewrite | — | 文首加「结论已被 §N 取代,证据继续有效」标注(部分已有) | D1 | |

## 4. 三组跨域同构合并映射

### 4.1 Chat 房间不启用端到端加密的前置与降级(权威:spec/project.md「Room 与消息」)

| 现行位置 | 收束后 |
| --- | --- |
| spec/project.md「Room 与消息」 | **权威定义**(前置、绑定校验、事后加密的降级与换绑恢复) |
| spec/project.md 对象表「Chat 端口绑定」行 | 一句引用 |
| spec/project.md 写入合同「Chat 端口绑定」行 | 一句引用(留「以 fresh 房间状态回读证明」的合同句) |
| spec/project.md 外部概念对齐 Room 行 | 一句引用 |
| project.md 关键规则第 6 条 | 人话一句 + 引用 |
| project.md Chat Room 场景末段(整段解释) | 缩为两句(为什么 + 引用) |
| architecture.md 场景与系统表 Chat Room 备注 | 保留半句「不开端到端加密」+ 引用 |
| connections.md 失败与恢复表「已绑定房间被开启端到端加密」行 | 保留(可观察结果唯一登记处),措辞指向权威 |
| contract-tests.md CT-PROJECT 两条 | 保留(失败用例是机械覆盖,不是复述) |

### 4.2 三条底线(权威:spec/agent.md 写入合同段)

| 现行位置 | 收束后 |
| --- | --- |
| spec/agent.md 写入合同「三条底线不可声明关闭」段 | **权威定义**(含「在此之内 Harness 是普通 Git 用户」与可选加固) |
| spec/system.md「命令与跨服务正确性」两类入口段 | 保留「两类 actor 来源」(那是 system 的权威),删对底线的复述 |
| spec/system.md「外部权威副作用」中 Harness 窄 principal 段 | 缩为一句引用 |
| spec/system.md「安全边界」末条 | 缩为一句引用(留「未启用加固时同信任域」的诚实声明) |
| agent.md 关键规则「三条底线」一条 | 人话一句 + 引用 |
| decision-history §22 | 保留(转折记录) |
| contract-tests.md CT-AGENT 两条、B2 验收 | 保留(机械覆盖) |

### 4.3 Herdr v0.8.2 能力限制(权威:能力条件句在 spec/agent.md「运行时与观测」;缺项清单只在 delivery.md P0 第 2 项与 binding 声明)【拍板点 11】

四项能力语汇:栅栏回显(fence echo)/ 逐次输入记录 / 事件游标(sequence/gap)/ 退出与停止回读。

| 现行位置 | 收束后 |
| --- | --- |
| spec/agent.md「运行时与观测」的「Herdr v0.8.2 的能力边界进入合同」段 | 改为**四项能力条件句**:声明则如何、未声明则按低信任降级(权威) |
| spec/agent.md 同节栅栏回显段「Herdr v0.8.2 没有这项能力」 | 改为「未声明栅栏回显的 Agency,第一阶段只在 HCTL 入口校验」 |
| spec/agent.md 写入合同 Terminal Input Lease 行 | 「provider 原生写入是否受租约约束按声明能力」 |
| spec/agent.md「终端通道」native_interactive_allowed 段 | 去点名,条件句 |
| spec/system.md「固定内核与受控端口」第四段 | 条件句 |
| spec/system.md「单写者」两处 | 条件句(「不接收也不回显 generation 的 Agency…」) |
| connections.md 启动顺序第 4 步 | 条件句 |
| agent.md「Agency 与 Herdr」限制清单四条 | 删整段,改一句「当前缺项见 delivery P0」 |
| delivery.md P0 第 2 项 | **保留缺项清单**(唯一家) |
| contract-tests.md CT-AGENT 相关条 | 保留,措辞从「Herdr 不能…」改为「Agency 未声明…时」 |
| architecture.md 三个面第三段「Herdr TUI…v0.8.2 无法提供…」 | 缩为「按 binding 声明的能力如实标注」 |

## 5. 必须继续留在各模块内的差异(合并不得吃掉)

| 模块 | 独有且不可并入共享机制的项 |
| --- | --- |
| Project | Repo 注册的待确认恢复与唯一 Repo Room;Project 归档的静默条件;Scoped Room 的目标冻结与回填归档;Room Invocation 的 scope(repo/project)与 lineage 字段;Request lifecycle 独占;mention 确定性解析 |
| Task | 契约惰性(无契约 Task 不进治理);规范外部实体唯一键与 placement 分离;HCTL-first / content-first 双创建路径;Run claim 双态;provider Done 信封条件;「完成」对待采纳的 fail-closed 与 divergence choice;不 reparent |
| Run | Obligation 按观察序号铸造(代次不在 Engine);五种重试身份;正常完成谓词;替代 Run 的原子转 claim;Gate 的作者回避与 known/unknown;dynamic fork 有界模板 |
| Agent | ChangeSet 单写租约与不可证静默时的新 worktree + 新 ChangeSet;review_subject_digest ≠ revision_digest;终局结果契约与观测截断;三层代次分离;输入策略二选一;终端五能力互不冒充 |

「看似重复、限定条件不同」的例子(合并时逐条核对):

- 「失败/取消/替代 Run 不终结 Task」(spec/run「Run → Task」)vs「Reopen/Deleted 只作来源事实」(spec/task)——前者是 Run 侧终态不传染,后者是外部事实不传染,主语不同,各留。
- 「一个 ChangeSet 至多一个活跃 lease」(Agent)vs「每个 Task 至多一个 Run claim」(Task)vs「每个 Agency binding scope 一个 owner lease」(System)——三种排他各有对象与代次,不能并成一条「单写者」。
- 「chat server 不可用」与「房间事后加密」走同一 fail-closed 规则,但可观察结果不同(重同步中 vs 需要关注)——connections 失败表两行都留。
- 「Dagu 直接 mutation 只标分歧」(Run)vs「Vikunja Done 可成为完成请求」(Task)——同是 provider 动作,裁决相反,原因(副作用顺序)必须在各自模块说清,不能抽成一条通用规则。
- 「Harness 可读 common-dir 与 refs」与「Harness 不获交付集成凭据」——一条是允许、一条是禁止,合并成一段时两句都要在。

## 6. 每文件提纲与行数上限

| 文件 | 上限 | 提纲(章节 → 一句话主旨) |
| --- | --- | --- |
| 根 README | 190 | 基线说明 → 一句话定位 → 目标架构图 → 现在能做什么(指 usage)→ 三条阅读路径 |
| usage.md | 240 | 保持现结构;死名替换 |
| vision.md | 180 | 一句话定位 → 五种失败模式 → 四阶段心智模型 → 目标体验(唯一权威)→ 两种控制制度(唯一权威)→ 产品原生核心与最小内核 → 三类数据(三句 + 引用)→ 设计原则 → 解决/不解决什么 |
| design/README | 100 | 四模块总表 → 对象关系 → 客户端与端口(概括 + 引用)→ 共同规则(一句一条)→ 文档纪律 → 支持文档 |
| architecture | 100 | 三个面 → 场景与系统 → 避免供应商锁定 → 4×3 矩阵 → 模块交接 → 数据丢了怎么办(产品叙述 + 引用) |
| project.md | 100 | 为什么存在 → 拥有什么 → 关键规则(正面句为主)→ Room 类型 → Chat Room 场景 → 交接 |
| task.md | 100 | 为什么存在 → 拥有什么 → 关键规则 → 无 Run 轻量路径 → Kanban 场景 → 交接 |
| run.md | 100 | 为什么存在 → 拥有什么 → 关键规则(含顺序才是关键)→ Workflow 场景 → 交接 |
| agent.md | 100 | 为什么存在 → 拥有什么 → 关键规则 → Terminal 场景 → Agency 与 Herdr(接口表 + 一句缺项指向)→ 原生会话导入 → 交接 |
| participant.md | 60 | 为什么存在 → 七件事分层 → 关键规则(引用式)→ 专业化 Participant → 合同落点 |
| context.md | 120 | 为什么存在 → 相邻概念分工 → 投喂三档 → 萃取与压缩 → 前情提要 → Run 内接力 → 关键规则(引用式)→ 场景 → 合同落点 |
| delivery.md | 220(与 contract-tests 合计) | 范围 → CLI → 明确不做 → 实现阶段 → 切片 A/B/Kanban → 自举阶段 → 选型判据 → P0(核销行 + Herdr 缺项清单)→ 打包 → 技术基线 → 未决 |
| contract-tests.md | (计入上行) | 总则一段 → CT-PROJECT → CT-TASK → CT-RUN → CT-AGENT → CT-CONNECTION → CT-SYSTEM → CT-PACKAGING(含 Tauri 用例)→ CT-WORKBENCH-IA / INPUT → CT-PRODUCT |
| spec/README | 90 | 词汇分类法 → 核心产品词 → 六族 → 三类数据(唯一权威)→ 词汇索引 → 外部对齐原则 → 文件 |
| spec/project | 120 | 对象 → 写入合同 → Repo 与归档 → Room 与消息(加密权威)→ Context/Memo/Artifact → Room Invocation → Request → 场景合同 → 对齐 |
| spec/task | 120 | 对象 → 契约与来源 → 写入合同 → 启动前置与排序 → 对齐 |
| spec/run | 120 | 对象 → 写入合同 → Workflow 与 Run 授权 → 从节点到结果 → Request/重试/Gate → Run → Task → 对齐 |
| spec/agent | 120 | 对象 → 写入合同(三条底线权威)→ ChangeSet 与 Git → 运行时与观测(能力条件句)→ 终端通道 → 对齐 |
| spec/connections | 180 | 连接模型 → 图 → 总表 → 五条方向 → Request 回路 → 版本/权限/替代 → 失败与恢复 → 场景与第三方 |
| spec/system | 200 | 组件 → 固定内核与端口 → 场景端口 → 动作与 provider 事件 → 命令正确性 → 外部副作用 → 事实与存储 → 单写者 → 启动与恢复 → 安全边界 |
| glossary | 145 | 保持;增删随正文 |
| decision-history | 折叠后减少 | §1–§33 保持编号;被取代节折叠;台账加行 |

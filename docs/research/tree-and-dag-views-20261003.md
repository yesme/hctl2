# Room 树与 Run 图的界面导航与可视化调研

> 类别：跨候选归纳 · 证据编号：E-VIEW-TREE-DAG<br>
> 状态：调研归纳 · 日期：2026-10-03；发布后正文不改，只在文末追加复核记录<br>
> 上级任务：`.memo/design/p2-control-20260906/07-demo-kickoff.md` §六 小活 B。起点是 [Room 树备忘](../../.memo/design/room-tree-20261001.md) 与 `docs/design/delivery.md` §未决问题。总览与复用决策用语见 [docs/research/README.md](./README.md)。本文不修改设计规范与约束。

## 背景与既有事实

`delivery.md` §未决问题 写明，Room 树与 Run 图的原生视角怎么画，要先调研信息可视化研究与业界做法、落 `docs/research/`，再出设计。本文是 Workbench（P3）的前置调研。

HCTL2 现行约束与体验规范中，已经确立了以下既有事实与硬边界（不重新论证）：

1. **左侧平铺视角已定**：`docs/user-experience/04-project-navigation.md:58` 明确规定，左侧导航栏的 Rooms 列表是**平铺列表**，把当前 Project 的全部 Topic Room 拍平列出，初始为空，不要求在侧栏逐层展开。
2. **原生视角按需打开**：在导航中选中一类对象后，在右侧视口打开它的**原生视角**（Room 看树，Run 看 DAG）。
3. **Room 树拓扑约束（v0.19.0）**：
   - 每个 Project 仅有一间唯一的主 Room 作根（`spec/project.md:68`）；
   - 挂靠关系归 chat server（Matrix Space 层级）原生持有，控制面只读投影为「上级 Room」「下级 Room」；
   - 多个上级时，以标为正式（canonical）的一个作主上级，其余作交叉链接展示；
   - 关闭一间 Topic Room 不连带关闭其下级；
   - 一间 Room 内部只有一条时间线加一层讨论串（`m.thread`），不做原地分叉，换方向开新 Room；
   - 出处（从哪条消息或 Request 开出）是控制面治理事实、永不改写；挂靠是协作组织、可以改挂，改挂走 content 写入通道并以回读为准。
4. **Run 图拓扑约束**：
   - Workflow Revision 是不可变、带摘要的编译规范，由控制面持有，先于引擎存在（`spec/run.md:68`）；
   - 图节点包含 Step、Obligation、Seat 与 Gate；
   - 边是推进条件的契约（引擎只拿编译副本）；
   - 执行面产生 Attempt、日志、Verdict 裁决与 Receipt 凭证；
   - 机械执行引擎选定 Dagu（`docs/research/workflow-engines.md`）。

本文聚焦回答四个问题：有层级的聊天空间怎么导航；有向无环图怎么看与跳到日志；信息可视化研究有哪些可引用的结论；对 HCTL2 的候选画法与取舍。

---

## 一、有层级的聊天空间怎么导航

调研对象涵盖 Matrix 客户端生态（Element、Cinny）、Slack、Discord、Zulip。

### 1. 各产品导航机制对照

| 产品 | 层级数据模型 | 侧栏导航展现 | 深层处理机制 | 讨论串（Thread）位置 |
| --- | --- | --- | --- | --- |
| **Matrix (MSC1772 Spaces / Element Web)** | 树状/DAG：Space 是特殊房间（`type: m.space`），通过 `m.space.child` 挂子节点，子节点以 `m.space.parent` 回指 | 左侧固定 Space 轨道图标切换顶层空间；次级侧栏以缩进列表展示子空间与房间 | 深度超过 3 层时侧栏发生缩进挤压（Indentation Creep）；提供「作为新根浏览（Explore rooms in space）」与路径面包屑 | 单层讨论串（MSC3440），右侧抽屉分屏展示 |
| **Matrix (Cinny v4.12.6)** | 同上（Matrix 协议原生） | 仅在空间切换栏列出空间，房间列表将多层子空间压平为单层展示 | 暂不支持多层嵌套树形展开（本库实测证据见 `room-tree-20261001.md` §五） | 暂无专用 thread 面板 |
| **Slack** | 扁平列表 + 视觉区块：频道无父子层级，仅允许用户自建折叠区块（Sections）归类 | 扁平频道列表，依赖命名空间前缀（如 `#proj-`）与可折叠 Section | 不做频道嵌套，杜绝深层问题；依赖全平铺的「Unreads」「Threads」聚合流高频处理 | 强制单层，右侧抽屉/拆分面板（Split Pane）展示 |
| **Discord** | 固定双层：Category（分类） -> Channel（频道），严禁分类嵌套分类 | 左侧树状折叠面板，但最大深度锁死为 2 层 | 不允许深层嵌套，结构性问题直接被产品规则消除 | 论坛频道以卡片网格呈现帖子；临时讨论串在活跃时缩进 1 级展示，非活跃自动隐藏 |
| **Zulip** | 严格双层：Stream（频道） -> Topic（话题） | 左侧展示 Stream 列表，选中时在当前 Stream 下缩进展示活跃 Topic | 不设子频道；依赖全局平铺的「Recent Conversations」（最近对话）统一表格流进行跨频道高频巡检 | 话题即轻量线程，每条消息必须归属一个 Topic |

### 2. 平铺列表与树形视图的分工规律

从上述产品的演进历史中，可以归纳出清晰的分工规律：

1. **平铺列表用于高频操作面**：
   - 适用场景：按时间逆序查看最新动态、未读消息批处理、全局关键词过滤、即时通知唤醒。
   - 优势：屏幕利用率高（无横向缩进浪费），无需逐层展开折叠，认知路径最短（单级查找）。
   - 典型代表：Zulip 的 Recent Conversations、Slack 的 Unreads 流、HCTL2 左侧导航的 Rooms 平铺列表。
2. **树形视图用于认知与谱系导航**：
   - 适用场景：新成员了解项目全貌、理解某一方案是围绕哪个父话题派生、跨分支对比分歧背景。
   - 优势：直观展现因果演化与话题派生关系（出处与挂靠），界定话题上下文边界。
   - 典型代表：Element 的 Space Explorer、论坛型树状归档。

### 3. 层级加深时的业界治理共识

当层级深度超过 3 层时，如果在窄侧栏（宽度通常为 200–280px）采用经典缩进大纲树，会必然遭遇**缩进挤压**：每深入一层缩进 12–16px，到达 4–5 层后文本区域被压缩至不足百像素，长标题被大量省略截断，可读性急剧恶化。

业界解决该问题的成熟做法有三种：

- **做法 A：根节点切换（Drill-down / Focus as Root）+ 顶部面包屑**。Element Web 与典型文件管理器（如 macOS Finder 列视图）的做法：当用户进入深层节点时，视口将该节点设为临时根节点，重新获得完整宽度，顶部通过一行紧凑的面包屑导航展示回到祖先节点的完整路径。
- **做法 B：将树形视图移出侧栏，交由主视口大纲承载**。侧栏保持平铺或单层分类，树的完整拓扑作为独立页面或工作区面板呈现（如 Notion 的页面树视图、GitHub Projects 的层级视图）。
- **做法 C：硬性封顶层级**。Slack（0 层嵌套）、Discord（1 层分类）、Zulip（1 层话题）均在产品层面直接否决多层房间树，防止深层蔓延。HCTL2 约束层已定「深度不设硬上限」（`room-tree-20261001.md` §四），因此不能采用做法 C，必须在展示层面通过做法 A 或做法 B 消化。

---

## 二、有向无环图（DAG）怎么看

调研对象涵盖 GitHub Actions、Dagu、Apache Airflow、Argo Workflows、Buildkite。

### 1. 各系统运行图与交互对照

| 系统 | 图渲染技术与布局 | 节点规模与折叠机制 | 点击节点与日志跳转交互 | 证据来源 |
| --- | --- | --- | --- | --- |
| **GitHub Actions** | 矢量 SVG 横向分层图（Sugiyama 拓扑，依赖 `needs:` 关系） | 节点数通常较少（<50）；矩阵任务折叠为展开下拉卡片 | 点击 Job 节点平滑切换到该 Job 的控制台日志，步骤以折叠列表输出 | [GitHub Actions 官方文档](https://docs.github.com/en/actions/monitoring-and-troubleshooting-workflows/monitoring-workflows/using-the-visualization-graph)（2026-10-03 核对） |
| **Dagu (v2.15.1)** | Web 画布（Mermaid / 矢量节点连线） | 显示单 DAG 各 Step；子 DAG 作为独立执行查看 | 点击 Step 节点弹出详情抽屉/模态框（Drawer/Modal），直接查看该 Step 的 stdout / stderr / 状态参数（API: `GET /api/v1/dag-runs/{name}/{dagRunId}/steps/{stepName}/log`） | [Dagu 官方文档与源码](https://dagu.cloud/docs/)、[本库选型](./workflow-engines.md)（2026-10-03 核对） |
| **Apache Airflow (2.6+ / 3.0)** | 双重视图协同：Grid View（网格）+ Graph View（SVG 拓扑图） | **Task Groups（任务组）**：支持作为复合节点在图和网格中原地折叠与展开 | 选中任务实例时，图与网格同步高亮（Multiple Coordinated Views）；右侧滑出详情抽屉（Details Drawer），内嵌 Log 标签页直接翻阅实时日志 | [Airflow 官方文档](https://airflow.apache.org/docs/apache-airflow/stable/ui.html)（2026-10-03 核对） |
| **Argo Workflows** | 基于 Dagre / D3 的可缩放平移矢量画布 | 支持大规模集群节点；子 DAG / Steps 嵌套折叠为 Cluster 边框；配右下角 Minimap（小地图） | 点击节点右侧滑出抽屉面板（Slide-out Panel），展示 Pod 状态、容器实时日志、输入输出 Artifacts 与事件，主画布保持焦点与位置 | [Argo Workflows 官方文档](https://argo-workflows.readthedocs.io/en/latest/ui/)（2026-10-03 核对） |
| **Buildkite** | 横向流水线泳道（Pipeline Waterfall / Swimlane） | 按 Stage / 并行组聚合折叠，显示成功/失败小色块 | 点击 Step 在卡片下方或右侧抽屉展开 ANSI 终端日志流 | [Buildkite 官方文档](https://buildkite.com/docs/pipelines/overview)（2026-10-03 核对） |

### 2. 节点过多时的折叠与聚焦模式

在复杂工作流中，节点数可能从十几个膨胀到数百个。业界主流处理模式有二：

1. **复合节点折叠（Compound / Clustered Nodes）**：
   - 将逻辑相关的步骤编组为 Task Group 或 Stage 容器盒（Boxed Cluster）。
   - 折叠状态下，复合节点对外汇聚输入/输出连线，内部节点被隐藏，仅显示摘要状态（如 `12/12 passed`）；
   - 用户可点击节点边框原地展开，连线自动动态重排（Dagu、Airflow、Argo 均采用此机制）。
2. **画布全局缩放与语义层级（Semantic Zooming & Minimap）**：
   - 在宏观缩放级别，节点内部文字隐藏，仅展示色块和主干拓扑流动；
   - 放大到微观级别时，逐步显露步骤名称、耗时、席位等详细信息；
   - 配备右下角 Minimap，提供当前视野在全图中的定位。

### 3. 从图跳到单节点日志的交互范式

所有调研系统均严格遵循一条交互铁律：**查看单节点日志与凭证时，绝不进行整页跳转或替换掉主画布**。

- **失败范式**：全页面跳转（从图跳进独立日志全屏页）。后果是用户失去了当前步骤在整个流程图中的上下游位置认知，排查故障后返回时丢失滚动与缩放焦点。
- **统治级最佳实践**：**侧边抽屉面板（Slide-out Drawer / Inspector Panel）**。
  - 点击节点时，图视口保持可见（可略微向左挤压或遮盖部分边缘），右侧平滑滑出占据 40%–50% 宽度的抽屉；
  - 抽屉内以标签页展示：基本信息、执行日志（ANSI 控制台）、输入输出产物、评审与裁决（Receipt / Verdict）；
  - 点击抽屉外部或按 ESC 即可退回纯图视野，图的原有平移与缩放焦点毫发无损。

---

## 三、信息可视化（InfoVis / HCI）研究的可引用结论

本节引用信息可视化与人机交互（HCI）领域的奠基性与实证研究结论，为 HCTL2 界面设计提供理论标尺。

### 1. 可视化探索总原则

- **寻道总原则（Visual Information Seeking Mantra）**：
  > “Overview first, zoom and filter, then details-on-demand.” —— Ben Shneiderman (1996) [[S1]](#ref-s1)
  - 含义：用户观察复杂系统时，必须先获得宏观全局图景（Overview），随后通过交互缩小关注范围（Zoom and Filter），最后根据需要调取局部细节（Details-on-Demand）。
  - 对 HCTL2 的映射：导航列表与缩略图提供 Overview；平移缩放、过滤非活动分支提供 Zoom/Filter；侧滑抽屉展示单节点日志与凭证提供 Details-on-Demand。

### 2. 树形结构可视化：空间与认知的权衡

Cockburn 等人（2008）[[C1]](#ref-c1) 对 Overview+Detail、Zooming 与 Focus+Context 进行了系统性综述；Ding & Lin（2003）[[D1]](#ref-d1) 评估了层级导航中的路径表示。综合结论如下：

| 可视化形态 | 空间开销模型 | 认知与交互特征 | 强项 | 弱项 |
| --- | --- | --- | --- | --- |
| **缩进大纲列表 (Indented List)** | 垂直 $O(N)$ 线性空间；水平随深度 $O(d)$ 消耗 | 最符合文本排版习惯，支持标准键盘上下键无障碍导航 | 文本标签完整度最高；垂直扫描与筛选速度最快 | 水平宽度受深度侵蚀（Indentation Creep）；无法表达非树形交叉链接 |
| **节点连线树 (Node-Link Diagram)** | 平面 $O(N^2)$ 几何空间，随分支因子指数发散 | 强调拓扑结构与分支演化（因果分歧、历史谱系） | 分支脉络一目了然；天然支持横向并列与交叉链接 | 屏幕空间利用率极低（大量空白网格）；长文本标签易被截断；宽树布局拥挤 |
| **路径面包屑 (Breadcrumbs)** | 水平固定 1 行空间 | 线性回溯，回答「我在层级中的哪一段」 | 空间极其紧凑；单次点击即可跃迁至任意祖先节点 | 仅展现单一先祖链条，完全屏蔽平行兄弟分支与下级分支 |
| **焦点+上下文 (Focus+Context / Fisheye)** | 动态非线性形变空间（Furnas 1986 [[F1]](#ref-f1)） | 放大感兴趣节点（Focus），缩小周边上下文（Context） | 兼顾局部细节与全局空间位置，无视口割裂感 | 几何畸变容易带来视觉眩晕与操作定位目标漂移 |

### 3. 多视图协同（Multiple Coordinated Views, MCV）

Baldonado、Woodruff 与 Kuchinsky（2000）在《Guidelines for Using Multiple Views in Information Visualization》[[B1]](#ref-b1) 中给出了使用多视图的关键准则：

- **多样性准则（Rule of Diversity）**：当单一视图无法同时满足两种相斥的交互目标时，才采用多视图。HCTL2 中「高频线性切换」（平铺列表）与「深层拓扑发现」（树/图）正是典型的相斥目标，采用多视图高度正当。
- **互补性准则（Rule of Complementarity）**：不同视图应展示不同层面的信息。平铺列表突出时间戳、未读与活跃态；树/图视图突出因果依赖与分支来源。
- **刷选与链接（Brushing and Linking, Roberts 2007 [[R1]](#ref-r1)）**：多视图之间必须保持状态同步。在平铺列表中选中的 Room，在原生树视图中必须高亮并居中；在 DAG 图中选中的 Step，在右侧抽屉中必须精确对应其日志。

### 4. 有向无环图分层绘制算法

Sugiyama、Tagawa 与 Toda（1981）[[S2]](#ref-s2) 提出的分层有向图绘制方法（Sugiyama Framework）是现代所有 DAG 可视化（Graphviz dot、Dagre、React Flow、Mermaid）的标准基础。
- 核心步骤：破环（DAG 天然无环）、顶点分层（Layering/Ranking）、层内排序以减少交叉（Crossing Reduction）、坐标分配以拉直线条（Coordinate Assignment）。
- 心理学认知收益：强制将数据流约束在单一方向（自左向右或自上而下），符合人类对“因果时序单向流转”的直觉理解；层级划分清晰展现了并发屏障（如多 Seat 并行评审汇入同一 Gate）。

---

## 四、对 HCTL2 的建议（候选画法与取舍）

依据上述调研事实与理论，针对 HCTL2 Workbench 的原生视角（右侧工作区）提出具体候选画法及其取舍。**本节仅列候选与权衡，不替所有者做最终决策。**

### 1. Room 树原生视角候选

#### 候选 R1：大纲缩进树 + 面包屑聚焦（Indented Outline Tree with Breadcrumbs & Drill-down）
- **画法结构**：
  - 在右侧主工作区以专用大纲面板展示完整的 Project Room 树；
  - 默认展开 2–3 层；更深节点支持点击「以此为根聚焦（Drill-down）」，聚焦后该节点作为视图顶层，顶部显示完整面包屑路径；
  - 正式上级（canonical parent）作为大纲骨架，非 canonical 上级在节点内以微型药丸标签（Pill Badge，如 `⤹ 交叉挂靠: Topic-A`）显示，点击可跳转；
  - 节点卡片内展示：房间名、最新消息时间戳、参与者头像列表、关联 Request 状态、是否已关闭（关闭节点灰度展示，不影响下级展开）。
- **取舍评价**：
  - **优势**：实现极其轻量，纯 DOM/CSS 即可完成，不需要引入重量级图形画布库；房间标题可以完整排布，无截断风险；键盘无障碍导航（上下展开/收起）体验最佳。
  - **劣势**：分支与分叉关系以缩进折线表达，几何直观性不如二维拓扑连线图强烈。

#### 候选 R2：交互式节点连线画布（Node-Link Canvas，基于 React Flow / Dagre）
- **画法结构**：
  - 在右侧主工作区渲染自上而下（或自左向右）的交互式图形画布；
  - 节点为标准化卡片，连线表示派生分支：实线箭头指向 canonical 上级，虚线表示非 canonical 交叉挂靠；
  - 支持平移（Pan）、缩放（Zoom）与右下角 Minimap（小地图）；
  - 已关闭房间以虚线边框或半透明灰度呈现。
- **取舍评价**：
  - **优势**：因果演化与分叉脉络具有最强烈的视觉冲击力，契合所有者提出的“🎄树”审美直觉；分叉深度与宽度在二维平面上一览无余。
  - **劣势**：需要引入 Web 绘图引擎（React Flow 已列入储备，但仍增加运行时足迹）；长房间名在节点卡片内必须省略截断；节点数多时画面容易稀疏，需要用户频繁平移缩放。

#### 候选 R3：多视图联动折叠抽屉（Coordinated Drawer / Sidebar Tree）
- **画法结构**：
  - 日常状态下，右侧主视口完全留给当前 Room 的聊天时间线，不常驻树视图；
  - 顶部常驻紧凑的面包屑导航（如 `Project Apollo / Topic 架构分流 / Topic 存储选型`）；
  - 面包屑末端配备「查看树（Tree View）」按钮或快捷键，点击后从右侧滑出半宽大纲树抽屉；在抽屉中点击其他房间直接切换当前视口。
- **取舍评价**：
  - **优势**：对日常高频聊天界面的侵入最小，最大化保障聊天消息的可读宽度；
  - **劣势**：树形结构默认不可见，属于二级操作，牺牲了“一瞥即知全貌”的常驻性。

---

### 2. Run DAG 原生视角候选

#### 候选 D1：分层连线图 + 侧滑审查抽屉（Layered Node-Link Canvas + Slide-out Inspector Drawer）
- **画法结构**：
  - 采用业界统治级范式（Argo / Airflow 标准）：基于 `React Flow` + `Dagre` 布局，采用自左向右（LR）流动连线；
  - 节点区分普通 Step 与 Gate：Gate 节点展示为复合卡片，内置展示法定票数（Quorum）、当前有效票数徽章，以及各 Seat 的 Participant 角色；
  - 点击任意 Step 或 Gate 节点，主画布不离开，右侧滑出占据 40%–50% 宽度的**审查抽屉（Inspector Drawer）**；
  - 抽屉内设标签页：① 概述与输入参数，② 实时执行日志（对接 Dagu REST API 提供的控制台输出），③ 证据与裁决（Verdict、Receipt、席位计票明细）；
  - 节点按执行状态赋予颜色脉冲（等待、运行中、成功、失败、需要关注）。
- **取舍评价**：
  - **优势**：排查问题体验最优秀，看日志完全不丢拓扑上下文；完美承载 HCTL2 独特的 Gate 多席位与凭证链治理结构；技术方案已有 `React Flow` 储备，路线最确定。
  - **劣势**：对于只有 1–2 个步骤的简单 Run，连线图可能略显结构过剩。

#### 候选 D2：Airflow 3 风格网格矩阵 + 图双向联动（Grid Matrix + Graph Dual-View）
- **画法结构**：
  - 主视口分为左右两半（或上下两半）：左半部分是 Grid View（行是 Step / Gate，列是历史 Run 实例与重试代次 Attempt Generation），右半部分是静态拓扑图；
  - 在 Grid 中点击某一格（某次 Attempt 的某个 Step），右侧图同步高亮该节点，并弹出底部/侧边日志面板。
- **取舍评价**：
  - **优势**：对于经历多次返工（Rework Loop）、多次重试（Retry）、或多代次（Generation）的复杂 Run，能够横向清晰比对不同代次的状态变迁。
  - **劣势**：屏幕空间消耗极大，在笔记本小屏幕（如 13-14 寸）上会导致双视图均严重拥挤；实现复杂度显著高于单一画布。

#### 候选 D3：线性流水线泳道（Pipeline Waterfall / Swimlane）
- **画法结构**：
  - 类似 Buildkite / GitLab CI：横向分为若干阶段（Stages），阶段内纵向排列并发任务；节点间不画贝塞尔连线，仅按阶段推进；
  - 点击节点在当前卡片下方直接原地折叠展开日志输出。
- **取舍评价**：
  - **优势**：布局高度紧凑规整，容易做响应式排版适配；简单线性工作流一目了然。
  - **劣势**：表达能力受限。HCTL2 的 Workflow Revision 允许任意符合 DAG 契约的边，若存在跨阶段跳跃依赖（Cross-layer Skip Edges）或条件分歧，泳道图无法准确表达真实的依赖约束。

---

### 3. 候选综合对比与建议汇总表

| 场景 | 候选方案 | 空间开销与紧凑度 | 实现与依赖成本 | 深度与拓扑表达力 | 日志/详情交互体验 | 推荐定位与取舍小结 |
| --- | --- | --- | --- | --- | --- | --- |
| **Room 树** | **R1：大纲缩进树 + 面包屑** | 高度紧凑（垂直线性） | **极低**（纯 DOM/CSS，无重型图形库） | 极佳（长文本无截断，深层靠聚焦） | 适中（直接点击切换） | **低成本稳健首选**：实现最轻，键盘友好，长房名不被截断；契合高密度工作台。 |
| **Room 树** | **R2：节点连线画布 (React Flow)** | 稀疏（二维平面膨胀） | 中（需接入 React Flow，处理节点布局） | 直观展现拓扑，但长名易被截断 | 良好（画布内缩放与点击） | **视觉冲击力强**：最符合“🎄树”直觉，但对长标题和深层图的空间利用率低。 |
| **Room 树** | **R3：多视图联动抽屉** | 最紧凑（日常零常驻） | 低—中（抽屉组件加面包屑） | 同 R1 | 需多一次点击呼出抽屉 | **极简沉浸首选**：适合聊天专注型界面，但弱化了树的发现性。 |
| **Run 图** | **D1：分层连线图 + 侧滑抽屉** | 平衡（图占主区，日志占抽屉） | 中（React Flow + Dagre，技术储备完备） | **极佳**（完美表达 DAG、多席位 Gate 与边） | **最高**（看日志不丢拓扑上下文） | **工业级标准首选**：对标 Argo/Airflow，最适合故障排查与证据审查。 |
| **Run 图** | **D2：Grid 网格 + 图双联动** | 极重（需双视口并列） | 较高（需管理复杂联动与矩阵渲染） | 优秀（额外包含时间代次轴） | 优秀（矩阵直接选点） | **重度返工利器**：多代次对比最强，但极度消耗屏幕宽度，小屏不耐受。 |
| **Run 图** | **D3：流水线泳道 (Swimlane)** | 紧凑（阶段分列） | 低（无重型连线图算法） | 较弱（无法优雅展现复杂跨级边） | 良好（原地展开） | **局限方案**：仅适用于严格分阶段的简单管道，无法满足复杂 DAG 治理。 |

---

## 证据与出处核对

### 1. 亲眼核对过的来源（2026-10-03 核对）

1. [Matrix MSC1772: Groups as rooms (Spaces)](https://github.com/matrix-org/matrix-spec-proposals/blob/main/proposals/1772-groups-as-rooms.md) 与 [Matrix Specification: Spaces](https://spec.matrix.org/latest/client-server-api/#spaces)：核对了 `m.space.child`、`m.space.parent`（`canonical` 属性）以及 `GET /_matrix/client/v1/rooms/{roomId}/hierarchy` 的分页与深度参数。
2. [Matrix MSC3440: Threading](https://github.com/matrix-org/matrix-spec-proposals/pull/3440)：核对了 Matrix 规范中讨论串单层不嵌套、所有成员可见的协议约束。
3. [Slack 帮助中心：使用讨论串组织讨论](https://slack.com/help/articles/115004071768-Use-threads-to-organize-discussions) 与 [自定义侧栏区块](https://slack.com/help/articles/360043207674-Organize-your-sidebar-with-custom-sections)：核对了 Slack 频道扁平性与讨论串侧滑分屏（Split Pane）行为。
4. [Discord 帮助中心：Threads FAQ](https://support.discord.com/hc/en-us/articles/4403205878423-Threads-FAQ) 与 [Forum Channels FAQ](https://support.discord.com/hc/en-us/articles/6208479914263-Forum-Channels-FAQ)：核对了 Discord 双层限制（Category -> Channel）、临时讨论串活跃时缩进 1 级展示及论坛网格形态。
5. [Zulip 官方文档：Streams and Topics](https://zulip.com/help/streams-and-topics) 与 [Recent Conversations](https://zulip.com/help/recent-conversations)：核对了 Stream-Topic 两层模型与 Recent Conversations 全局平铺流视图的结构。
6. [GitHub Actions 官方文档：Using the visualization graph](https://docs.github.com/en/actions/monitoring-and-troubleshooting-workflows/monitoring-workflows/using-the-visualization-graph)：核对了基于 `needs:` 的分层拓扑图渲染、展开与日志跳转行为。
7. [Dagu 官方文档与开源仓库](https://dagu.cloud/docs/)、[GitHub: dagucloud/dagu](https://github.com/dagucloud/dagu)：核对了 Web UI 中的 DAG 画布渲染，以及点击 Step 弹出抽屉查看 `GET /api/v1/dag-runs/.../steps/.../log` 的实现。
8. [Apache Airflow 官方文档：UI Overview (Grid and Graph)](https://airflow.apache.org/docs/apache-airflow/stable/ui.html)：核对了 Airflow 2.6+ 与 3.0 中 Grid View 嵌套 Task Groups 的展开机制、Grid 与 Graph 的双向高亮协同（Multiple Coordinated Views），以及侧滑日志抽屉。
9. [Argo Workflows 官方文档与仓库：Web UI](https://argo-workflows.readthedocs.io/en/latest/ui/)、[GitHub: argoproj/argo-workflows](https://github.com/argoproj/argo-workflows)：核对了大规模 DAG 的 Cluster 盒折叠、Minimap 缩放，以及点击节点滑出 Pod 实时日志抽屉的交互。
10. [Buildkite Pipelines 官方文档：Pipelines overview](https://buildkite.com/docs/pipelines/overview)：核对了横向瀑布流泳道（Swimlane）展示与控制台抽屉展开机制。
11. 学术文献与规范原著：
    - <a id="ref-s1"></a>[S1] Ben Shneiderman. *The Eyes Have It: A Task by Data Type Taxonomy for Information Visualizations*. In Proc. IEEE Symposium on Visual Languages (VL '96), 1996, pp. 336–343.
    - <a id="ref-f1"></a>[F1] George W. Furnas. *Generalized Fisheye Views*. In Proc. ACM SIGCHI Conference on Human Factors in Computing Systems (CHI '86), 1986, pp. 16–23.
    - <a id="ref-c1"></a>[C1] Andy Cockburn, Amy Karlson, Benjamin B. Bederson. *A Review of Overview+Detail, Zooming, and Focus+Context Interfaces*. ACM Computing Surveys (CSUR), Vol. 41, No. 1, Article 2, 2008.
    - <a id="ref-b1"></a>[B1] Michelle Q. Wang Baldonado, Allison Woodruff, Allan Kuchinsky. *Guidelines for Using Multiple Views in Information Visualization*. In Proc. Working Conference on Advanced Visual Interfaces (AVI '00), 2000, pp. 110–119.
    - <a id="ref-s2"></a>[S2] Ko Sugiyama, Shojiro Tagawa, Mitsuhiko Toda. *Methods for Visual Understanding of Hierarchical System Structures*. IEEE Transactions on Systems, Man, and Cybernetics, Vol. 11, No. 2, 1981, pp. 109–125.
    - <a id="ref-d1"></a>[D1] Chen Ding, Xia Lin. *Evaluating Breadcrumbs for Web Navigation*. In Proc. 26th Annual International ACM SIGIR Conference on Research and Development in Information Retrieval (SIGIR '03), 2003, p. 433.

### 2. 标为未核实的项（不编造数据）

1. **Slack 与 Discord 内部真实线程使用比例**：未核实。两家商业公司未公开其企业/社区用户中通过侧滑面板阅读 thread 相比直接在频道阅读的具体转化率或流失率指标。
2. **万级超大规模 DAG 画布在前端 Canvas/SVG 中的极限 FPS 衰减曲线**：未核实。Airflow 与 Argo 官方文档未提供超过 10,000 个动态任务实例时的 DOM 节点渲染与内存开销实测数据表；在 HCTL2 中，单次 Run 规模由 Workflow Revision 决定（通常在几十到数百节点范围内），暂不触及万级极限。
3. **Cinny 上游后续版本对多层 Space 树的开发排期**：未核实。Cinny 官方仓库暂无明确承诺在哪个具体版本中补齐多层 Space 大纲树展开面板。

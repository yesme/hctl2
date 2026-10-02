# Room 树与 Run 图的界面导航与可视化调研

> 类别：跨候选归纳 · 证据编号：E-VIEW-TREE-DAG<br>
> 状态：调研归纳 · 日期：2026-10-03；发布后正文不改，只在文末追加复核记录<br>
> 上级任务：`.memo/design/p2-control-20260906/07-demo-kickoff.md` §六 小活 B。起点是 [Room 树备忘](../../.memo/design/room-tree-20261001.md) 与 `docs/design/delivery.md` §未决问题。总览与复用决策用语见 [docs/research/README.md](./README.md)。本文不修改设计规范与约束。

## 背景与既有事实

`delivery.md` §未决问题 写明，Room 树与 Run 图的原生视角怎么画，要先调研信息可视化研究与业界做法、落 `docs/research/`，再出设计。本文是 Workbench（P3）的前置调研。

HCTL2 现行约束与体验规范中，已经确立了以下既有事实与边界（不重新论证）：

1. **左侧平铺视角已定**：`docs/user-experience/04-project-navigation.md` §左侧 Rooms 列表 明确规定，左侧导航栏的 Rooms 列表是**平铺列表**，把当前 Project 的全部 Topic Room 拍平列出，初始为空，不要求在侧栏逐层展开。
2. **原生视角按需打开**：在导航中选中一类对象后，在右侧视口打开它的**原生视角**（Room 看树，Run 看 DAG）。
3. **Room 树拓扑约束（v0.19.0）**：
   - 同一 Project 的 Room 通常组成以主 Room 为根的树（`docs/design/spec/project.md` §Room 与消息）；
   - 挂靠关系归 chat server（Matrix Space 层级）原生持有，控制面只读投影为全部上级与下级 Room；
   - 全部读到的上级均保留投影，不据此删去其余上级；标为正式（canonical）的上级仅作展示提示；只有恰好一个上级时才称它为父节点；
   - 关闭一间 Topic Room 不连带关闭其下级；
   - 一间 Room 内部只有一条时间线加一层讨论串（`m.thread`），不做原地分叉，换方向开新 Room；
   - 出处（从哪条消息或 Request 开出）是控制面治理事实、永不改写；挂靠是协作组织、可以改挂，改挂走 content 写入通道并以回读为准。
4. **Run 图拓扑约束**：
   - Workflow Revision 是不可变、带摘要的编译规范，由控制面持有，先于引擎存在（`docs/design/spec/run.md` §Workflow Revision 与规范）；
   - 图节点包含 Step、Obligation、Seat 与 Gate；
   - 边是推进条件的契约（引擎只拿编译副本）；
   - 执行面产生 Attempt 与原始执行日志；控制面记录 Verdict 裁决与 Receipt 凭证；
   - 机械执行引擎选定 Dagu（`docs/research/workflow-engines.md`）。

本文聚焦回答四个问题：有层级的聊天空间怎么导航；有向无环图怎么看与跳到日志；信息可视化研究有哪些可引用的结论；对 HCTL2 的候选画法与取舍。

---

## 一、有层级的聊天空间怎么导航

调研对象涵盖 Matrix 客户端生态（Element、Cinny）、Slack、Discord、Zulip。

### 1. 各产品导航机制对照

| 产品 | 层级数据模型 | 侧栏导航展现 | 深层处理机制 | 讨论串（Thread）位置 |
| --- | --- | --- | --- | --- |
| **Matrix (MSC1772 Spaces / MSC3440)** | 树状/DAG：Space 是特殊房间（`type: m.space`），通过 `m.space.child` 挂子节点，子节点以 `m.space.parent`（含 canonical 提示）回指 | 协议支持树状层级关系遍历；具体客户端侧栏展示策略各有不同（Element 展现见未核实项） | 协议层不限制层级深度；多父级通过并列事件表达 | 单层讨论串（MSC3440），协议要求所有成员可见且不嵌套 |
| **Matrix (Cinny v4.12.6)** | 同上（Matrix 协议原生） | 仅在空间切换栏列出空间，房间列表将多层子空间压平为单层展示 | 暂不支持多层嵌套树形展开（本库实测证据见 `room-tree-20261001.md` §五） | 暂无专用 thread 面板 |
| **Slack** | 扁平列表 + 视觉区块：频道无父子层级，仅允许用户自建折叠区块（Sections）归类 | 扁平频道列表，依赖命名空间前缀（如 `#proj-`）与可折叠 Section | 不做频道嵌套，杜绝深层问题；依赖全平铺的「Unreads」「Threads」聚合流高频处理 | 讨论串交互见未核实项 |
| **Discord** | 固定双层：Category（分类） -> Channel（频道），严禁分类嵌套分类 | 双层折叠面板，最大深度为 2 层（文档见未核实项） | 不允许深层嵌套，结构性问题直接被产品规则消除 | 论坛频道以卡片网格呈现帖子；临时讨论串在活跃时缩进 1 级展示（文档见未核实项） |
| **Zulip** | 严格双层：Stream（频道） -> Topic（话题） | 左侧展示 Stream 列表，选中时在当前 Stream 下缩进展示活跃 Topic | 不设子频道；依赖全局平铺的「Recent Conversations」（最近对话）统一表格流进行跨频道高频巡检 | 话题即轻量线程，每条消息必须归属一个 Topic |

### 2. 平铺列表与树形视图的分工规律

从上述产品的设计演进中，可以归纳出分工规律：

1. **平铺列表用于高频操作面**：
   - 适用场景：按时间逆序查看最新动态、未读消息批处理、全局关键词过滤、即时通知唤醒。
   - 特点：屏幕利用率高（无横向缩进挤压），无需逐层展开折叠，认知路径最短（单级查找）。
   - 典型代表：Zulip 的 Recent Conversations、Slack 的 Unreads 流、HCTL2 左侧导航的 Rooms 平铺列表。
2. **树形视图用于认知与谱系导航**：
   - 适用场景：新成员了解项目全貌、理解某一方案是围绕哪个父话题派生、跨分支对比分歧背景。
   - 特点：展现因果演化与话题派生关系（出处与挂靠），界定话题上下文边界。
   - 典型代表：Matrix Spaces 的层级关系网、论坛型树状归档。

### 3. 层级加深时的常见处理策略

在宽度受限的窄侧栏采用经典缩进大纲树，随着嵌套层级加深，会遭遇**缩进挤压**：每深入一层均需产生缩进位移，层级过深后有效文本区域受到挤压，长标题容易被省略截断，影响可读性。

业界应对深层层级通常有三种做法：

- **做法 A：根节点切换（Drill-down / Focus as Root）+ 顶部面包屑**。当用户进入深层节点时，视口将该节点设为临时根节点，重新获得完整展示宽度，顶部通过面包屑导航指示回到祖先节点的路径。
- **做法 B：将树形视图移出窄侧栏，交由主视口大纲承载**。侧栏保持平铺或单层分类，树的完整拓扑作为独立页面或工作区面板呈现（如文档系统或项目管理视图）。
- **做法 C：硬性封顶层级**。Slack（0 层嵌套）、Discord（1 层分类）、Zulip（1 层话题）均在产品层面直接限制多层房间树。HCTL2 约束层已定「深度不设硬上限」（`room-tree-20261001.md` §四），因此不采用做法 C，需在展示层面通过做法 A 或做法 B 消化。

---

## 二、有向无环图（DAG）怎么看

调研对象涵盖 GitHub Actions、Dagu、Apache Airflow、Argo Workflows、Buildkite。

### 1. 各系统运行图与交互对照

| 系统 | 图渲染技术与布局 | 节点规模与折叠机制 | 节点与日志交互机制 | 证据来源 |
| --- | --- | --- | --- | --- |
| **GitHub Actions** | 矢量 SVG 横向分层图（Sugiyama 拓扑，依赖 `needs:` 关系） | 节点数通常较少；矩阵任务折叠为下拉卡片 | 点击 Job 节点进入该 Job 的执行详情与控制台日志页面（图不再常驻），步骤以折叠列表呈现 | [GitHub Actions 官方文档](https://docs.github.com/en/actions/monitoring-and-troubleshooting-workflows/monitoring-workflows/using-the-visualization-graph)（2026-10-03 核对） |
| **Dagu (v2.15.1)** | Web 画布（Mermaid / 矢量节点连线） | 显示单 DAG 各 Step；子 DAG 作为独立执行查看 | 点击 Step 节点弹出详情浮层/抽屉，查看该 Step 的 stdout / stderr / 状态参数（API: `GET /api/v1/dag-runs/{name}/{dagRunId}/steps/{stepName}/log`） | [Dagu 官方文档与源码（见未核实项）](./workflow-engines.md)（2026-10-03 核对） |
| **Apache Airflow (2.6+ / 3.0)** | 双重视图协同：Grid View（网格）+ Graph View（SVG 拓扑图） | 支持 Task Groups 概念；折叠展开见未核实项 | 选中任务实例进入 Task Instance View 查看日志；协同高亮与抽屉见未核实项 | [Airflow 官方文档](https://airflow.apache.org/docs/apache-airflow/stable/ui.html)（2026-10-03 核对） |
| **Argo Workflows** | 矢量图画布 | 支持大图缩放；Cluster 折叠见未核实项 | 节点抽屉与日志交互见未核实项 | [Argo Workflows 官方文档（见未核实项）](https://argo-workflows.readthedocs.io/en/latest/walk-through/)（2026-10-03 核对） |
| **Buildkite** | 横向流水线泳道（Pipeline Waterfall / Swimlane） | 按 Stage / 并行组聚合展示 | 点击 Step 展开终端日志流（具体抽屉形式见未核实项） | [Buildkite 官方文档](https://buildkite.com/docs/pipelines/overview)（2026-10-03 核对） |

### 2. 节点过多时的常见折叠与聚焦模式

在复杂工作流中，节点数可能从十几个增长到数百个。业界常见处理模式有二：

1. **复合节点折叠（Compound / Clustered Nodes）**：
   - 将逻辑相关的步骤编组为任务组（Task Group）或 Stage 容器盒；
   - 折叠状态下，复合节点对外汇聚输入/输出连线，内部节点隐藏，显示摘要状态；
   - 用户可点击节点边框展开，内部拓扑动态显露。
2. **画布全局缩放与语义层级（Semantic Zooming & Minimap）**：
   - 在宏观缩放级别，隐藏节点次要文本，突出主要拓扑走向；
   - 放大后逐步显露步骤名称、耗时与席位状态；
   - 配备 Minimap，辅助指示当前视野在全图中的位置。

### 3. 从图跳到单节点日志的交互范式

调研系统在处理从拓扑图查看具体步骤日志时，主要存在两种交互形态，各有利弊：

1. **页面跳转范式（Page Navigation）**：
   - 行为：点击节点后，离开拓扑图视口，导航进入该节点的专属执行/日志页面（如 GitHub Actions 点击 Job 进入控制台、Airflow 进入 Task Instance 详情页）。
   - 取舍：全屏日志视口充足，便于快速搜索长日志与下载原始输出；但临时脱离了拓扑图全局视野，排查多节点关联错误时需要返回或开多标签页。
2. **局部抽屉/面板范式（Slide-out Drawer / Overlay Panel）**：
   - 行为：点击节点时，拓扑图保留在主画布视口（可相应缩小或向侧方退让），在侧边或底部滑出检查面板，展示日志与运行参数。
   - 取舍：拓扑图与节点日志并存，保持因果上下文；但在较小屏幕上，画布与日志面板各自获得的有效宽度受限。

---

## 三、信息可视化（InfoVis / HCI）研究的可引用结论

本节引用信息可视化与人机交互（HCI）领域的研究结论，为 HCTL2 界面设计提供参考。

### 1. 可视化探索总原则

- **寻道总原则（Visual Information Seeking Mantra）**：
  > “Overview first, zoom and filter, then details-on-demand.” —— Ben Shneiderman (1996) [[S1]](#ref-s1)
  - 含义：观察复杂信息集合时，先获得宏观图景（Overview），随后通过交互缩小关注范围（Zoom and Filter），最后根据需要调取局部细节（Details-on-Demand）。
  - 对 HCTL2 的映射：导航列表与缩略图提供 Overview；平移缩放、过滤非活动分支提供 Zoom/Filter；侧滑面板展示单节点日志与凭证提供 Details-on-Demand。

### 2. 树形结构可视化：空间与认知的权衡

Cockburn 等人（2008）[[C1]](#ref-c1) 对 Overview+Detail、Zooming 与 Focus+Context 进行了系统性综述。基于文献概念与常见人机交互界面实践，对常见层级可视化形态的归纳与比较如下（本表为作者归纳）：

| 可视化形态 | 空间占用特征 | 认知与交互特征 | 强项 | 弱项 |
| --- | --- | --- | --- | --- |
| **缩进大纲列表 (Indented List)** | 垂直占用随节点数增加；水平宽度随嵌套深度受挤压 | 符合文本排版习惯，支持键盘上下移动导航 | 文本标签完整度高；垂直扫描效率高 | 嵌套过深时水平宽度被挤压；不适合表达交叉网状挂靠 |
| **节点连线树 (Node-Link Diagram)** | 二维平面展开；随分支数量在横向或纵向延伸 | 强调拓扑结构与分支演化关系 | 分支脉络直观；可表达非单亲交叉连线 | 屏幕空间利用率较低；长文本标签排布受限 |
| **路径面包屑 (Breadcrumbs)** | 水平占用单行空间 | 线性回溯，回答当前节点所在的祖先路径 | 空间紧凑；支持直接点击祖先节点跃迁 | 仅展现单一路径，不展现同级平行分支与下级分支 |
| **焦点+上下文 (Focus+Context / Fisheye)** | 动态形变空间（Furnas 1986 [[F1]](#ref-f1)） | 放大感兴趣节点（Focus），缩小周边上下文（Context） | 兼顾局部细节与全局空间定位 | 视觉形变可能增加识别成本 |

### 3. 多视图协同（Multiple Coordinated Views, MCV）

Baldonado、Woodruff 与 Kuchinsky（2000）在《Guidelines for Using Multiple Views in Information Visualization》[[B1]](#ref-b1) 中给出了使用多视图的关键准则：

- **多样性准则（Rule of Diversity）**：当单一视图无法同时满足两种相斥的交互目标时，才采用多视图。HCTL2 中「高频线性切换」（平铺列表）与「深层拓扑发现」（树/图）属于不同交互需求，采用多视图具有合理性。
- **互补性准则（Rule of Complementarity）**：不同视图应展示不同层面的信息。平铺列表突出时间戳、未读与活跃态；树/图视图突出因果依赖与分支来源。
- **视图关联（Coordinated Views / Linking, Roberts 2007 [[R1]](#ref-r1)）**：多视图之间需考虑状态联动。在平铺列表中选中的 Room，在原生树视图中可相应高亮；在 DAG 图中选中的 Step，在详情面板中对应其日志。

### 4. 有向无环图分层绘制算法

Sugiyama、Tagawa 与 Toda（1981）[[S2]](#ref-s2) 提出的分层有向图绘制方法（Sugiyama Framework）是现代 DAG 可视化库（如 Graphviz dot、Dagre）的基础：
- 核心步骤：破环、顶点分层（Layering/Ranking）、层内排序以减少交叉（Crossing Reduction）、坐标分配以拉直线条（Coordinate Assignment）。
- 认知效果：将流向约束在单一方向（自左向右或自上而下），契合对因果时序流转的直觉理解；层级划分展示了并发步骤汇聚于同一屏障（如多 Seat 汇入同一 Gate）的结构。

---

## 四、对 HCTL2 的建议（候选画法与取舍）

依据上述调研事实与理论，针对 HCTL2 Workbench 的原生视角（右侧工作区）梳理候选画法及其取舍。**本节仅列候选与权衡，不替所有者做最终决策。**

### 1. Room 树原生视角候选

在具体画法中，如何使用 canonical 属于界面呈现的设计选择：当读到的多个上级中恰有一个标为 canonical 时，画法可选择将其作为树形骨架；当未标记 canonical 或标记了多个 canonical 时，画法可按首个读取项或按创建顺序确定默认布局树，并显式展示所有读到的上级交叉链接，不删减任何投影事实。

#### 候选 R1：大纲缩进树 + 面包屑聚焦（Indented Outline Tree with Breadcrumbs & Drill-down）
- **画法结构**：
  - 在右侧主工作区以专用大纲面板展示 Project Room 树；
  - 默认展开前数层；更深节点支持点击「以此为根聚焦（Drill-down）」，聚焦后该节点作为临时顶层，顶部显示完整面包屑路径；
  - 正式上级（canonical parent）作为主骨架排列；非 canonical 上级在节点内以标签或次级链接展示；无 canonical 或多 canonical 时列出全部上级并提示候选状态；
  - 节点卡片内展示：房间名、最新消息时间、参与者、关联 Request 状态、是否已关闭（关闭节点灰度展示，不影响下级展开）。
- **取舍评价**：
  - **优势**：实现轻量，基于通用 DOM/CSS 即可完成，不需要引入重量级图形画布库；房间标题排布完整，无截断问题；键盘导航友好。
  - **劣势**：分支与分叉关系以缩进折线表达，几何直观性弱于二维连线图。

#### 候选 R2：交互式节点连线画布（Node-Link Canvas，基于 React Flow / Dagre）
- **画法结构**：
  - 在右侧主工作区渲染自上而下（或自左向右）的图形画布；
  - 节点为标准化卡片，连线表示派生分支：实线箭头指向 canonical 上级，虚线表示非 canonical 交叉挂靠；
  - 支持平移（Pan）、缩放（Zoom）与 Minimap（小地图）；
  - 已关闭房间以虚线边框或半透明灰度呈现。
- **取舍评价**：
  - **优势**：因果演化与分叉脉络直观，符合图形拓扑直觉；支持二维横向展开。
  - **劣势**：需要引入 Web 绘图引擎（React Flow 已在技术储备，但会增加运行时体积）；长房间名在节点卡片内可能需要省略截断；节点较多时需频繁平移缩放。

#### 候选 R3：多视图联动折叠抽屉（Coordinated Drawer / Sidebar Tree）
- **画法结构**：
  - 右侧主视口留给当前 Room 的聊天时间线，不常驻树视图；
  - 顶部常驻面包屑导航（如 `Project Apollo / Topic 架构分流 / Topic 存储选型`）；
  - 面包屑末端提供「查看树」入口，点击后从侧边滑出大纲树抽屉；在抽屉中点击其他房间可切换当前视口。
- **取舍评价**：
  - **优势**：对日常聊天界面的侵入较小，保证聊天消息的展示宽度；
  - **劣势**：树形结构默认不可见，需主动呼出，弱化了全局拓扑的直接感知。

---

### 2. Run DAG 原生视角候选

#### 候选 D1：分层连线图 + 侧滑审查抽屉（Layered Node-Link Canvas + Slide-out Inspector Drawer）
- **画法结构**：
  - 基于 `React Flow` + `Dagre` 布局，采用自左向右（LR）流动连线；
  - 节点区分普通 Step 与 Gate：Gate 节点展示为复合卡片，内置展示法定票数（Quorum）、当前有效票数徽章，以及各 Seat 的 Participant 角色；
  - 点击任意 Step 或 Gate 节点，画布保持在视口中，右侧滑出审查抽屉（Inspector Drawer）；
  - 抽屉内设标签页：① 概述与输入参数，② 实时执行日志（对接 Dagu REST API），③ 证据与裁决（控制面记录的 Verdict、Receipt、席位计票明细）；
  - 节点按执行状态区分颜色（等待、运行中、成功、失败、需要关注）。
- **取舍评价**：
  - **优势**：查看日志时不离开拓扑图，保留上下文位置；能够承载 Gate 多席位与凭证链治理结构；技术方案已有 `React Flow` 储备。
  - **劣势**：步骤极少的简单 Run 场景下，图形画布显得相对复杂；小屏幕下需平衡画布与抽屉的宽度。

#### 候选 D2：网格矩阵 + 图双向联动（Grid Matrix + Graph Dual-View）
- **画法结构**：
  - 主视口由两部分组成：一边是 Grid View（行是 Step / Gate，列是历史 Run 实例与重试 Attempt），另一边是拓扑图；
  - 在 Grid 中选中某一格时，图同步高亮对应节点，并展示日志。
- **取舍评价**：
  - **优势**：能够清晰比对多次重试或返工代次的状态变迁。
  - **劣势**：屏幕空间消耗较大，在小屏幕上并列展示易拥挤；实现与状态联动复杂度较高。

#### 候选 D3：线性流水线泳道（Pipeline Waterfall / Swimlane）
- **画法结构**：
  - 类似常见 CI 界面：横向划分为若干阶段（Stages），阶段内排列任务卡片；
  - 点击节点在卡片下方原地展开日志。
- **取舍评价**：
  - **优势**：结构规整，易于做响应式排版；简单线性流一目了然。
  - **劣势**：表达能力受限。HCTL2 的 Workflow Revision 允许任意合法 DAG 边，存在跨阶段跳跃依赖或复杂汇聚时，泳道图难以准确表达。

---

### 3. 候选综合对比与取舍汇总表

| 场景 | 候选方案 | 空间占用与紧凑度 | 实现与依赖成本 | 拓扑表达力 | 日志/详情交互特征 | 适合什么情况 |
| --- | --- | --- | --- | --- | --- | --- |
| **Room 树** | **R1：大纲缩进树 + 面包屑** | 紧凑（垂直线性） | 较低（通用 DOM/CSS，无重型图形库） | 良好（长文本无截断，深层靠聚焦） | 直接点击切换 | 适合文本可读性与键盘操作优先的高密度工作台；实现成本低，无复杂几何连线。 |
| **Room 树** | **R2：节点连线画布 (React Flow)** | 需画布平移（二维平面） | 中（需接入 React Flow，处理节点布局） | 直观展现拓扑连线，长名可能受限 | 画布内缩放与点击 | 适合注重因果拓扑与分叉脉络直观性的场景；空间利用率受拓扑展开影响较大。 |
| **Room 树** | **R3：多视图联动抽屉** | 紧凑（日常零常驻） | 低—中（抽屉组件加面包屑） | 同 R1 | 需多一次点击呼出抽屉 | 适合高频消息浏览与聊天主视口专注的场景；树结构需主动调出。 |
| **Run 图** | **D1：分层连线图 + 侧滑抽屉** | 兼顾图与侧边面板 | 中（React Flow + Dagre） | 适合表达 DAG、多席位 Gate 与边 | 看日志时保留图上下文 | 适合需要兼顾拓扑上下文与节点详细输出的场景；适合排查故障与对照治理记录。 |
| **Run 图** | **D2：Grid 网格 + 图双联动** | 占用较大（双视口并列） | 较高（需管理复杂联动与矩阵渲染） | 包含时间代次轴对比 | 矩阵直接选点 | 适合多次返工与多代次比对的复杂运行分析；对屏幕宽度与展示面积要求高。 |
| **Run 图** | **D3：流水线泳道 (Swimlane)** | 紧凑（阶段分列） | 低（无重型连线图算法） | 难以表达复杂跨级边 | 原地展开 | 适合阶段清晰的简单线性流水线；在复杂无规则 DAG 拓扑下表达力受限。 |

---

## 证据与出处核对

### 1. 亲眼核对过的来源（2026-10-03 核对）

1. [Matrix MSC1772: Groups as rooms (Spaces)](https://github.com/matrix-org/matrix-spec-proposals/blob/main/proposals/1772-groups-as-rooms.md) 与 [Matrix Specification: Spaces](https://spec.matrix.org/latest/client-server-api/#spaces)：核对了 `m.space.child`、`m.space.parent`（`canonical` 属性）以及 `GET /_matrix/client/v1/rooms/{roomId}/hierarchy` 的分页与深度参数。
2. [Matrix MSC3440: Threading](https://github.com/matrix-org/matrix-spec-proposals/pull/3440)：核对了 Matrix 规范中讨论串单层不嵌套、所有成员可见的协议约束。
3. [Zulip 官方文档：Streams and Topics](https://zulip.com/help/streams-and-topics) 与 [Recent Conversations](https://zulip.com/help/recent-conversations)：核对了 Stream-Topic 两层模型与 Recent Conversations 全局平铺流视图的结构。
4. [Slack 帮助中心：自定义侧栏区块](https://slack.com/help/articles/360043207674-Organize-your-sidebar-with-custom-sections)：核对了 Slack 频道列表通过 Section 自定义折叠归类，频道本身无父子嵌套。
5. [GitHub Actions 官方文档：Using the visualization graph](https://docs.github.com/en/actions/monitoring-and-troubleshooting-workflows/monitoring-workflows/using-the-visualization-graph)：核对了基于 `needs:` 的可视化图以及官方文档明确记录的「点击 Job 查看日志」行为。
6. [Buildkite Pipelines 官方文档：Pipelines overview](https://buildkite.com/docs/pipelines/overview)：核对了横向瀑布流泳道（Swimlane）展示的官方文档结构。
7. 本库既有事实与代码选型：Cinny 4.12.6 与 Dagu 2.15.1 版本锁、React Flow / Dagre 依赖库储备、`room-tree-20261001.md` 测试事实。
8. 学术文献与规范原著：
   - <a id="ref-s1"></a>[S1] Ben Shneiderman. *The Eyes Have It: A Task by Data Type Taxonomy for Information Visualizations*. In Proc. IEEE Symposium on Visual Languages (VL '96), 1996, pp. 336–343.
   - <a id="ref-f1"></a>[F1] George W. Furnas. *Generalized Fisheye Views*. In Proc. ACM SIGCHI Conference on Human Factors in Computing Systems (CHI '86), 1986, pp. 16–23.
   - <a id="ref-c1"></a>[C1] Andy Cockburn, Amy Karlson, Benjamin B. Bederson. *A Review of Overview+Detail, Zooming, and Focus+Context Interfaces*. ACM Computing Surveys (CSUR), Vol. 41, No. 1, Article 2, 2008.
   - <a id="ref-b1"></a>[B1] Michelle Q. Wang Baldonado, Allison Woodruff, Allan Kuchinsky. *Guidelines for Using Multiple Views in Information Visualization*. In Proc. Working Conference on Advanced Visual Interfaces (AVI '00), 2000, pp. 110–119.
   - <a id="ref-r1"></a>[R1] Jonathan C. Roberts. *State of the Art: Coordinated & Multiple Views in Exploratory Visualization*. In Proc. Fifth International Conference on Coordinated and Multiple Views in Exploratory Visualization (CMV '07), 2007, pp. 61–71.
   - <a id="ref-s2"></a>[S2] Kozo Sugiyama, Shojiro Tagawa, Mitsuhiko Toda. *Methods for Visual Understanding of Hierarchical System Structures*. IEEE Transactions on Systems, Man, and Cybernetics, Vol. 11, No. 2, 1981, pp. 109–125.

### 2. 标为未核实的项（不编造数据）

1. **Element Web 客户端在深层 Space 下的具体前端交互表现**：未核实。MSC 规范只定义 Space 状态事件与层级 API；Element Web 在深层时是否缩进挤压或提供作为新根浏览与面包屑，属于客户端具体实现细节，未在此次调研中通过逐版本跑通核实。
2. **Slack 讨论串帮助页面全文**：未核实。因网络访问受限，具体讨论串分屏交互细节基于通用经验归纳。
3. **Discord 帮助中心文档（Threads FAQ / Forum FAQ）**：未核实。官方帮助中心页面返回 403 访问限制，双层与帖子卡片规则属于产品既有认知，未在本次调研中通过有效文档链接直接核验。
4. **Dagu 官网文档公网访问**：未核实。公网 `dagu.cloud` 域名连接超时；Dagu 的 DAG 渲染与 REST API 参数核验依赖于本库既有调研 `docs/research/workflow-engines.md` 与本地源码，官网在线文档未直接连通。
5. **Argo Workflows 官方 UI 文档页面**：未核实。原链接 `argo-workflows.readthedocs.io/en/latest/ui/` 返回 404；Argo 的 Cluster 折叠、Minimap 与抽屉式日志交互属于经验与社区实践归纳，未通过有效文档链接核实。
6. **Airflow 官方文档 UI Overview 中关于 Task Group 嵌套折叠与侧边抽屉的文字**：未核实。官方文档该页面主要记录 Grid 与 Graph 视图以及进入 Task Instance View 查看日志，未明文描述任务组原地折叠与抽屉展开细节。
7. **Buildkite 页面上的具体抽屉交互细节**：未核实。官方文档该页面概述了流水线结构，但单步日志是在卡片下方还是右侧抽屉展开未核实。
8. **Slack 与 Discord 内部真实线程使用比例**：未核实。两家商业公司未公开其企业/社区用户中通过侧滑面板阅读 thread 相比直接在频道阅读的具体转化率或流失率指标。
9. **万级超大规模 DAG 画布在前端 Canvas/SVG 中的极限 FPS 衰减曲线**：未核实。Airflow 与 Argo 官方文档未提供超过 10,000 个动态任务实例时的 DOM 节点渲染与内存开销实测数据表；在 HCTL2 中，单次 Run 规模由 Workflow Revision 决定（通常在几十到数百节点范围内），暂不触及万级极限。
10. **Cinny 上游后续版本对多层 Space 树的开发排期**：未核实。Cinny 官方仓库暂无明确承诺在哪个具体版本中补齐多层 Space 大纲树展开面板。


---

## 复核记录

### 2026-10-03 · Fable 修正复核（PR #304 审阅意见闭环）

依据 PR #304 审阅者 Fable 的修正复核意见（见 issue comment 5961839889），按研究目录规矩（发布后正文不改，在文末追加复核记录），对正文中三处未核实内容与节名引用作出逐条作废与修正声明：

1. **§二.1 表格中三行未核实内容的来源修正与作废**：
   - **GitHub Actions 行**：
     - 正文原句「矢量 SVG 横向分层图（Sugiyama 拓扑，依赖 `needs:` 关系）」「矩阵任务折叠为下拉卡片」作废。
     - 更正为：官方文档页面仅核验了「按 `needs:` 关系绘制工作流图」以及「点击 Job 查看日志」；其底层是否为矢量 SVG / Sugiyama 拓扑，以及矩阵任务是否折叠为下拉卡片，官方文档该页均无文字记载，标记为**未核实**。
     - 依据：[GitHub Actions 官方文档：Using the visualization graph](https://docs.github.com/en/actions/monitoring-and-troubleshooting-workflows/monitoring-workflows/using-the-visualization-graph)（2026-10-03 核对）。
   - **Dagu 行**：
     - 正文原句「Mermaid / 矢量节点连线」「点击 Step 弹出详情浮层/抽屉」以及接口路径 `GET /api/v1/dag-runs/{name}/{dagRunId}/steps/{stepName}/log` 作废；正文依据中断言「本库的 workflow-engines.md 与源码」作废。
     - 更正为：经核对，本库 [`docs/research/workflow-engines.md`](./workflow-engines.md) 与 [`docs/research/sdk/dagu.md`](./sdk/dagu.md) 均未记载上述三项内容（`sdk/dagu.md` 中实际记录的接口为 `.../steps/{stepName}/status`）。Dagu Web UI 的前端渲染技术形式、点击弹层/抽屉交互细节以及日志接口，在此次调研中均标记为**未核实**。
     - 依据：核对本库既有调研文件 [`docs/research/workflow-engines.md`](./workflow-engines.md) 与 [`docs/research/sdk/dagu.md`](./sdk/dagu.md) 原文，如实将未记载项归入未核实。
   - **Argo Workflows 行**：
     - 正文原句「矢量图画布」「支持大图缩放」及行尾标注的「（2026-10-03 核对）」作废。
     - 更正为：Argo Workflows 的图形渲染技术、画布缩放及日志交互在本次调研中未核验到有效文档文字，整行全部特征均标记为**未核实**，并移除行尾的「2026-10-03 核对」字样。
     - 依据：官方 walk-through 页面未核对到具体渲染架构描述，来源页上没有的内容不编造，全行归入未核实。

2. **三处文件节名与出处对齐**：
   - **04-project-navigation.md 节名**：
     - 正文 §背景与既有事实 第 1 项引用的「`docs/user-experience/04-project-navigation.md` §左侧 Rooms 列表」作废。
     - 更正为：[`docs/user-experience/04-project-navigation.md`](../user-experience/04-project-navigation.md) §Rooms：只列 Topic Rooms。
     - 依据：核对源文件小节标题原文。
   - **spec/run.md 节名**：
     - 正文 §背景与既有事实 第 4 项引用的「`docs/design/spec/run.md` §Workflow Revision 与规范」作废。
     - 更正为：[`docs/design/spec/run.md`](../design/spec/run.md) §Workflow 与 Run 授权。
     - 依据：核对源文件小节标题原文。
   - **Cinny 压平多层 Space 的实测证据出处**：
     - 正文 §一.1 表格 Cinny 行及 §证据与出处核对 第 7 项中指引的「见 `room-tree-20261001.md` §五」作废。
     - 更正为：实测证据出处为 [`.memo/log/2026-10-01-room-树.md`](../../.memo/log/2026-10-01-room-树.md)，以及 [`docs/design/delivery.md`](../design/delivery.md) §开工前限时验证 第 3 项脚注。
     - 依据：核对 [`.memo/design/room-tree-20261001.md`](../../.memo/design/room-tree-20261001.md)，其 §五 仅为任务落地清单，真正的测试事实记录在 log 原文与 delivery.md 脚注。

3. **Run 施工图节点模型与对象术语纠正**：
   - 正文 §背景与既有事实 第 4 项原句「图节点包含 Step、Obligation、Seat 与 Gate」作废。
   - 更正为：图上的节点是 **Workflow Node**；运行时每个外部节点对应 **Obligation**，其下有 **Seat** 与 **Attempt**；**Gate** 是评审关卡。规范中没有「Step」这一核心对象；正文候选 D1、D2 等处使用的「Step」仅作界面展示习惯用语，其实际指代 Workflow Node。
   - 依据：[`docs/design/spec/run.md`](../design/spec/run.md) §对象。

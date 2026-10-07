# ① 官网：定位、参照与审美

> 状态：讨论中 · 待拍板（范围与衬线浓度见上一级 README §待拍板）<br>
> 基线：main @ `543a8b5`（草案 v0.19.3）<br>
> 去向：`hctl2-site` 仓库的定位与设计系统（仓库未建）

## 定位

官网是 hctl2 **展示面的公开投影**，不是第二权威源：内容单源在仓库正文，站点是构建产物（生成与审定见 [`03-neo-build.md`](./03-neo-build.md)）。它做两件事——对外解释（愿景、设计、研究、证据），对外证明（每页都能回指来源与版本）。读者有两种：人读排印过的页面，agent 读内容层原样导出的 markdown（见 [03 §两个读者面](./03-neo-build.md#两个读者面)）。

## 参照系：六家公开面的机制读数

取自仓库自己的研究，不是另做一轮网调；视觉与字体要新做实测（见 §待补）。

| 平台 | 记录到的公开面 | 出处 |
| --- | --- | --- |
| Codeg | 文档站 `docs.codeg.app/guide/*`；营销面未见记录 | [`codeg.md`](../../../docs/research/workbench/codeg.md) :38 |
| First Tree | `first-tree.ai` 与 `docs.first-tree.ai`（占位）；**官网描述已与实现分叉** | [`first-tree.md`](../../../docs/research/workbench/first-tree.md) :29 |
| Multica | `multica.ai/docs/*`；仓库内 `.mdx` 按 commit 钉定；官网挂滚动声明 | [`multica.md`](../../../docs/research/workbench/multica.md) :16、:37-38 |
| Superset | `docs.superset.sh` 子域；仓库内 `.mdx` 按 commit 钉定；同款声明 | [`superset.md`](../../../docs/research/workbench/superset.md) :16、:48 |
| Stably Orca | `www.onorca.dev/docs`（model / review / agents 三段）；「官网只补充产品行为」 | [`stably-orca.md`](../../../docs/research/workbench/stably-orca.md) :33、:37 |
| Cumora | 只记一条官网链接；被当作定位口号的那句话实际出自仓库文档 | [`cumora.md`](../../../docs/research/workbench/cumora.md) :15、:37 |

三条可复用机制：**滚动声明**（官网会滚动更新，能力判断以固定源码与测试为准）；**文档与代码同仓、按 commit 钉定**；**独立 docs 子域或同域 `/docs`**。一条反面教材：**手写官网必然漂移**（First Tree）——所以站点由仓库正文生成，不维护第二份内容。一片空白：六份文件都没记字体、配色与落地页区块，视觉调研是新增工作。

## 审美：三声部排印

起点是所有者的偏好：Steve Jobs 2005 年斯坦福演讲里里德学院书法课那段——**排印属于产品**，不是装饰。落到 hctl2：产品自己有三种语言，站点照此分声部。

| 声部 | 承载 | 用在哪 | 候选（待实测定档） |
| --- | --- | --- | --- |
| 衬线 | 人的语言：塑形、论证、叙事 | 标题、引文、长文正文 | Latin：Newsreader／Source Serif 4／Literata／Spectral；中文：思源宋体 SC 或系统宋体栈（Songti SC／SimSun／Noto Serif CJK SC） |
| 无衬线 | 系统的语言：状态、契约、界面 | 导航、标签、表格头、按钮 | Inter／IBM Plex Sans + 思源黑体／PingFang 系统栈 |
| 等宽 | 机器的语言：ID、凭证、日志 | 版本号、commit、命令、终端记录 | JetBrains Mono／IBM Plex Mono／Commit Mono |

真正的品味点不在选哪款字，而在**中英混排**：仓库正文已按 [`WRITING-GUIDE.md`](../../../WRITING-GUIDE.md) L4 的规矩排版（中文与英文、数字之间加一个半角空格；句内标点用全角；行内代码与相邻中文之间加半角空格；禁止全角空格作分隔），站点只要不破坏它，就已经赢过多数中文技术站。增强只用 CSS 原生机制，不引 JavaScript 排版库：`text-autospace`（中西文自动间距）、`text-spacing-trim`（标点挤压）、`hanging-punctuation`、`font-variant-numeric: tabular-nums`（版本号与 SHA 对齐）、`text-wrap: pretty`。

体积是硬决策：中文字体全集 5–20MB 不可接受。起步用**系统中文栈 + 自托管 Latin woff2**（零 CDN、可离线，与 local-first 同调）；思源宋体按 `unicode-range` 切片子集化留作独立决策。

其余：动效几乎为零（只保留 hover／focus 与锚点高亮），呼吸感来自留白、行宽（中文 34–40 字）、行距 1.7–1.8 与字重梯度；浅色为纸、深色为终端；五模块语义色只在图里用，不做装饰；mermaid 沿用全库既有画法；`docs/demos/demo-1.md` 那类真实跑通的记录做成等宽的「终端胶片」。

## 信息架构

| 路径 | 内容源 | 作用 |
| --- | --- | --- |
| `/` | README 与 [`vision.md`](../../../docs/design/vision.md) | 定位一句、四个短语、三面架构图、五种失败模式、现状声明 |
| `/principles/` ×5 | 新写正文（见 [`02-principles.md`](./02-principles.md)） | 站点脊柱；每条带证据与出处 |
| `/participant/` | [`participant.md`](../../../docs/design/participant.md) 与约束附录 | 七件事分层、名册与席位、"不是 bot" 的对照 |
| `/composition/` | [`delivery.md`](../../../docs/design/delivery.md) 与各调研单案 | 模块 → 开源项目 → 版本 → 许可证 → 复用决策 |
| `/evidence/` | `docs/research/**` | 研究库索引：钉定版本、证据层级、许可证 |
| `/product/` | Workbench 快照 | 产品页面快照（见下） |
| `/design/` `/spec/` `/usage/` | 原样渲染既有正文 | 设计、约束层、使用说明 |
| `/colophon/` | lock 里的审定记录 | 页首一行写站点钉定的 hctl2 commit 与构建日期；其下是每个意群的来源 commit、prompt、模型、审校人与日期、状态，过期的标出 |
| `/llms.txt` 与各意群 `.md` | 内容层原样导出 | 给 agent 读：不经展现层，带来源、关系与审定状态 |

`.memo/` 不上站。

## 产品快照

- 快照等 Workbench 就绪，**不放假图**；空窗期由 `docs/demos/demo-1.md` 的真实终端记录顶替，页面上写明"这是终端记录，不是产品界面"。
- 快照必须**可复现**：在钉定 commit 上跑固定流程、脚本化截屏，每张图带 **commit、日期、生成方式**；产品版本一变就重新生成，做不到就撤下。

## 待补（进 P0 调研）

1. 六家官网的落地页区块、docs 结构与字体配方实测（含首屏字节数）。
2. 字体候选的授权与体积实测（含中文子集化方案）。
3. 域名与托管方式（GitHub Pages 与自定义域名）。

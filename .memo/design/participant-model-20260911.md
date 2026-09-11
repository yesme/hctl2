# Participant 模型重排：工种、规划者、施工者、执行体

> 状态：所有者 2026-09-11 拍板的模型，本 memo 是落地对照表；设计正文在 PR #210，约束在 PR #211<br>
> 起因：所有者提出「模板是 Agency 那边的概念；没有『项目里的人』，只有『planning 里的人』和『building 里的人』」。

## 一句话

Agency 在名册里定义**工种**；人把工种的实例**选进 Room** 当规划者、或**选进 Run 的席位**当施工者，两次选人各自独立；每次派工 Agency 交付一个**执行体**。项目不持有成员名单，只持有选人策略。

## 四样东西各是什么

| 东西 | 英文 | 是什么 | 谁定义 / 谁选 | 住在哪一层 | 生命周期 |
| --- | --- | --- | --- | --- | --- |
| 工种 | Profession | 一类可派的人：Harness、模型、Skill 配置、人设默认、默认职责倾向（规划 / 施工）、条款 | Agency 定义；control 收进来时冻结引用与摘要 | Agency 名册；控制面存储只存冻结引用 | 随 Agency 名册版本 |
| 参与者 | Participant | 工种的一个实例，被选进某个 Room 或某个 Run 的席位才存在：名字标签、职责、权限、预算上限 | 人在建 Room / Trigger Preview 里选规划者；在启动 Run 的预览里按席位要求选施工者；机器可推荐 | Room 名册（规划者）；Run 施工清单的席位（施工者） | 随 Room 或 Run |
| 两顶帽子 | planner / worker | 选进 Room 的叫规划者，选进 Run 席位的叫施工者；不是对象 | — | — | — |
| 执行体 | execution runtime | 一次派工交付的具体运行：进程、装载的 Skill、PTY 或结构化接入 | Agency 按冻结的执行规格交付 | Participant 模块（Execution Runtime 对象） | 随一次尝试或单次调用；崩了、换候选就换 |

## 与现有条款逐条对照

| 现有写法 | 改成 | 落点 |
| --- | --- | --- |
| Participant 是用户级稳定身份与配置 revision | Participant 是被选进 Room 或 Run 席位的一位工种实例，只存在于被选进的地方 | spec/participant.md 对象表；participant.md 三词段与七件事分层 |
| Project 的「参与者授权」（v0.16.2 取代 Role Binding） | 撤销。规划者记在 Room 名册，施工者记在 Run 施工清单的席位；Project 只持有选人策略（允许的 Agency 与工种、预算上限、多样性要求） | spec/project.md 对象与写入表、参与者授权一段；glossary；system.md 权威地图；connections.md 权限缩小链 |
| Participant–Agency Binding（v0.16.2 新命名） | 撤销。参与者本身就是某个 Agency 某个工种的实例，换派出方就是选另一个工种的实例，不存在换绑；Binding 族回到五个 | spec/participant.md、spec/README 词汇索引、glossary Binding 族 |
| 系统角色名 worker（执行体） | 改为 执行体（execution runtime）；worker 让给「施工者」这顶帽子 | spec/README 系统角色名；architecture.md 场景与系统表；glossary |
| Execution Spec 冻结 exact Participant revision + Project version + 参与者授权条目 | 冻结选入记录：工种引用与摘要、Agency、Worker Profile revision、Skill、职责、权限、预算 | spec/connections.md Execution Spec 字段块；spec/run.md Manifest 与 Gate |
| 施工清单冻结每个 Seat 的 Participant revision | 施工图的席位只写要求（工种、Skill、证据等级）；启动 Run 的预览按要求从名册选施工者并冻结；启动后席位不换人，候选切换只换执行体 | spec/run.md 启动与 Manifest；run.md 关键规则 |
| `@` 只按 Project 参与者授权解析 | `@` 只按本 Room 名册解析；Run 席位上的施工者不在 Room 里被 @，和施工者说话走 Execution Chat | spec/project.md 场景约束；participant.md |
| Scoped Room 的参与者 | 默认继承父 Room 名册，可只取子集；这是规划阶段内部的继承，不跨阶段 | spec/project.md Room 名册 |
| 计票去重键（Worker Profile revision + Bundle 摘要） | 不变 | — |
| CLI `participant authorize\|revoke` | `profession list\|show`、`room roster add\|remove\|list`；施工者在 `run preview\|start` 里选 | delivery.md |

## 不变的

独立性三来源、席位多样性策略、计票去重、席位隔离、评审席位绑定精确版本、凭证回溯、七件事分层的下四层由 Agency 供给、人不是 Participant。

## 待办

- 设计正文（PR #210）与约束（PR #211）已按上表改；Codex、Grok、GLM、K3 四家全审，意见已按 2026-09-11 的修正落进两个 PR。
- 观测里按 Participant 记的快省数据，键改为「工种 + Worker Profile revision」，跨 Room 与 Run 可比。
- 塑形 Skill 与评审 Skill 里出现的「Participant」用法核一遍，凡指「项目成员」的改成规划者或施工者。

## 与单元模型备忘的对应（2026-09-12 补）

`.memo/design/case-study-20260907/01-unit-model.md` 是并行的一条线，B 批（Participant）与 D 批（Run 交接）会再改这两份正文；开方案时以本模型为基线，两边词汇按所有者 2026-09-12 的裁决对齐：

| 单元模型备忘 | 本模型 |
| --- | --- |
| 参与者模板 | 工种（Profession）；备忘已改口 |
| 雇佣的 Project 一级安排 | Room 名册里的规划者；Project 只持有选人策略，没有成员名单 |
| 雇佣的 Run 一级安排 | Run 席位的选入记录；启动后不换人 |
| 待命、四态、会话不跨机器 | 参与者一侧的实现；一个施工者同时跑几个执行体也留给实现 |
| 席位从聊天室会话分叉 | 不做；施工者与读回从零起，施工图带「上下文」章节 |
| 评审独立按策略 | 多样性按 Gate 策略声明；计票去重总是生效 |


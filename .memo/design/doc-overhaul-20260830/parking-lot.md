# 停车位:大修中不做的语义议题

> 状态:讨论中 · 随施工追加,拍板后各自另立项<br>
> 基线:main @ `192e7b6`(草案 v0.15.0)<br>
> 去向:每项拍板后按转向立项(decision-history 章节 + 合同 bump),或显式拒绝留痕<br>
> 规则:施工图 §2 红线 1——大修不改合同语义;任何 harness 发现语义疑点只登记到此,不在本轮修。

| # | 问题 | 发现者 | 位置 | 建议 | 状态 |
| --- | --- | --- | --- | --- | --- |
| 1 | Herdr `managed_single_writer`:v0.8.2 无统一 writer gate、无法检测同一用户直接用原生 controller——标「暂不支持」还是提供并验证隔离机制 | codex(v0.15.0 审议) | spec/agent.md 运行时与观测、终端通道;connections.md 启动顺序 | 先标暂不支持,待 Herdr 上游补 fence echo / 输入记录后再评 | 待拍板 |
| 2 | `hctl2-tool` 范围:是否正式收窄为 HCTL 特有的 Git/worktree/intent/readback,把 lint、commit 署名、PR 正文、memo 写入交给 Git/Buck2/`gh` | codex(v0.15.0 审议) | spec/system.md 组件表;delivery.md P1 | 收窄;普通机械工作不进第一方组件 | 待拍板 |
| 3 | `doc-cleanup-backlog` 13 条中改合同语义的项(活动 Run 时禁止采纳新 Revision、Project 归档前置全清、Scoped Room 只许成功回填归档、discovery 绝不联网、所有动作必须 Preview 等) | codex 0825 评审;本轮禁令盘点 | 各条见 backlog 表 | 本轮只处理措辞形态;语义改判逐条立项 | 待拍板 |

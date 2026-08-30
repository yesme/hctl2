# C4 交付与研究索引施工卡(派给 GPT,C3 合入后)

> 状态:待派发 · Fable 依冻结总图预写;所有者在 C3 合入后原样派给 GPT<br>
> 基线:C3 合入后的 main<br>
> 依据:[`03-approved-plan.md`](./03-approved-plan.md) §3 C4 行(拍板点 10 已接受);[`02-target-map.md`](./02-target-map.md) §1(contract-tests.md 新增)、§3.19、§3.23、§4.3 的 CT 落点;[`review-c2-fable.md`](./review-c2-fable.md) 发现 #3(设计地图改链待办);S1 死文普查 A3/A4(Tuwunel 矛盾、macOS 15 论证)、不一致账本 #1/#2/#6/#12

## 派发文本

```
你在 hctl2 仓库工作。先 git fetch 并把当前分支快进到 origin/main(C3 已合入),读 AGENTS.md、CONSTRAINTS.md,再读 .memo/design/doc-overhaul-20260830/ 下:施工图 README.md(§2 红线)、03-approved-plan.md(§3 C4 行)、02-target-map.md(§1 目标树、§3.19、§3.23、§4.3 表末 CT 两行)、04-prohibition-whitelist.md §3(CT 用例不计禁令净减)、01-inventory-inconsistency.md #1/#2/#3/#6/#12、01-inventory-dead-text.md A3/A4/A24。你是唯一写手,现在开 C4(交付与研究索引)。

范围:docs/design/delivery.md、新建 docs/design/contract-tests.md、docs/research/README.md、docs/research/remote-control.md 与 docs/research/remote-control/、以及所有指向 delivery.md 契约测试矩阵锚点的链接(根 README 阅读路径、docs/design/README.md 支持文档、其他任何处——用 grep 找全)。不碰 spec/、decision-history.md、设计层正文。

一、拆出契约测试矩阵(拍板点 10):
- 新建 docs/design/contract-tests.md:文件头(状态:验证文档 · 草案 v0.15.0;定位一句:八族可观察行为的失败用例,不描述状态机、不新增合同——合同变更须先改 spec 再加用例);总则一段(沿用 delivery「契约测试矩阵」开头三句:检查可观察行为、每族一个 family ID、新增合同必须在对应族加失败用例);然后 CT-PROJECT / CT-TASK / CT-RUN / CT-AGENT / CT-CONNECTION / CT-SYSTEM / CT-PACKAGING / CT-WORKBENCH-IA / CT-WORKBENCH-INPUT / CT-PRODUCT 原样搬入,**逐条不增不删**,只做下面两处措辞改动:
  (a) CT-PACKAGING:在既有拒绝项后补 Tauri 壳中立用例——「未声明的 capability/permission/scope、未声明的 IPC 或插件能力、remote runtime script/CDN 时拒绝;Electron 安全网形态下 renderer Node/raw IPC 同样拒绝」。这是既有族内补用例(合同 spec/system.md 安全边界已于 v0.14.2 改为壳中立),不是新族;在 PR 描述单列。
  (b) CT-AGENT 里所有「Herdr 不能…/Herdr 事件流没有…/Herdr 不能证明…」措辞改为「Agency 未声明 X 能力时…」(与 C3 落地的能力条件句一致;四项:栅栏回显 / 逐次输入记录 / 事件游标 / 退出停止回读);用例语义不变。
- delivery.md 删除「契约测试矩阵」整节,在原位留一句「契约测试矩阵见 contract-tests.md」。
- 全库改链:所有 `delivery.md#契约测试矩阵` 与 `#ct-…` 锚点改指 `contract-tests.md`(根 README 阅读路径的实现者与 adapter 两条、docs/design/README.md 支持文档条目、以及 grep 到的其他处);设计地图支持文档新增 contract-tests 一行。

二、delivery.md 其余处置(总图 §3.19):
- 「第一阶段范围」末段的动作分类叙述与 spec/system.md「客户端动作与 provider 事件」重复,缩为一句引用;P2/P3 表不动。
- 「开工前限时验证」五项:已完成探针的项目核销为一行结论 + 指向 docs/research 对应证据文件(Dagu → workflow-engines.md;Tuwunel → matrix-homeserver.md 与研究总表运维表;Vikunja → task-backends.md);**Herdr 项保留 v0.8.2 缺项清单全文**——这是缺项清单的唯一家(拍板点 11);第 5 项远端后端保留。
- 「打包策略」:第 270 行 Tuwunel 句改为事实——「Tuwunel 上游无 Darwin 制品,HCTL2 在自己的 GitHub Release 托管按 SHA-256 锁定的 macOS 包,日常打包消费托管制品;源码构建只用于更新托管制品」(与 :262 及研究总表 2026-08-30 记录一致;不一致账本 #1/#2)。「macOS 最低基线为 15」:原论证(tmux 官方二进制)已失效——**核实后写实**:查 src/packaging/dependencies/lock.json、platforms/macos/*.sh、Herdr/Tauri 2/托管 Tuwunel 制品的实际最低系统版本与 CI runner,写成「macOS 最低基线为 N(依据:X)」;若核实不出唯一数值,保留 15 并在 PR 描述说明依据缺口,不猜。usage.md:20 已在 C1 改为引用此处。
- 「未决问题」:三条已了结(划线)项删除,只留开放项;开放项措辞不动。
- 上限:delivery.md 与 contract-tests.md 合计 ≤220 行;超限在 PR 描述单独论证,不为凑数删验收项。

三、docs/research 索引与搬移(总图 §3.23,证据文件一个不删、一字不改):
- README.md 头部状态行的四个重组日期括号缩为一句(「按产品类别组织;单案入同层子目录」);五张互相重叠的表收束:条目索引为主索引(保留每文件一行含证据编号与类别),①–⑧ 类别表各缩为一段一句话导读 + 指向条目;「L1 精选实现证据」「L4 补充证据」并入对应类别导读或条目索引;「已选外部服务的运维与资源占用」表**原样保留**(唯一 footprint 权威,delivery 链到它)。
- remote-control.md:Codex Remote Feishu 单案正文搬入 remote-control/codex-remote-feishu.md(锚点 `e-l1-codex-remote-feishu` 保留),观察清单表与两条复核记录并入 remote-control/README.md;根目录 remote-control.md 删除,全库改链(研究总表 ⑤ 类表、条目索引、来时路若有)。
- tmux-runtime.md / agentd-runtime-candidates-20260829.md 已有「已被取代」标注,不动;workbench-shell.md「当前决定」段 C1 已改,不动。

守则:CT 用例逐条不增不删(除 (a) 一条新用例与 (b) 措辞);不改合同语义;不新增概念;研究证据文件只搬不改;引用只指现存锚点;找不到落点或核不出的事实——停下来登记 parking-lot。

验收:`cd src && build/docs/materialize_repo_tree.sh && ./buck2 test root//build/docs/...` 全绿(链接检查会覆盖全部改链);密度报告贴 PR,并按白名单 §3 说明 contract-tests.md 的计数单列、不计入净减;PR 描述三节 + delivery/contract-tests 行数(现/上限/后)+ 改链清单 + macOS 基线核实依据 + P0 核销前后对照;"业界方案调研"不适用;开 auto-merge 但**闸门前不合入**——本簇闸门是 Grok 删除安全审计,所有者裁决后你一次修正再合。完成后回复:两文件行数、改链条数、macOS 基线结论与依据、parking-lot 新增。
```

## 给 Grok 的 C4 删除安全审计卡(与 C4 PR 同时派)

```
你在 hctl2 仓库工作。先 git fetch,读 .memo/design/doc-overhaul-20260830/ 下的施工图 README.md(§2 红线)、02-target-map.md §3.19/§3.23、04-prohibition-whitelist.md §3、07-c4-card.md。对象:C4 PR(codex 分支)的 diff。

只做删除安全审计,不审文笔、不提新结构、不改文件,每条引用旧来源(main 上的文件:行)与新位置:
① CT 矩阵拆出:十族用例是否逐条一一对应(数一遍两边条数);CT-PACKAGING 新增的 Tauri 用例是否只是既有族内补用例、未引入新族或新合同;CT-AGENT 的「Agency 未声明…时」改写是否保住每条用例的拒绝条件。
② P0 核销:被缩成一行的三项(Dagu/Tuwunel/Vikunja),「HCTL 实际调用的 API 与行为」清单是否仍能从指向的研究文件找到;Herdr 缺项清单是否原样保留。
③ 打包策略:Tuwunel 句是否与 :262 及 lock/研究总表一致;macOS 基线数值的依据是否真实(去看 lock 与脚本),不是沿用旧论证。
④ 研究总表收束:条目索引是否覆盖收束前五张表的全部文件;每个条目的复用决策与证据编号是否未丢;运维表是否原样;remote-control 搬移后锚点是否保留。
⑤ 全库改链:有没有指向 delivery 旧锚点或根目录 remote-control.md 的残留(docs-check 应已抓,复核一遍 decision-history 与 .memo 内的引用)。

输出写到 .memo/design/doc-overhaul-20260830/review-c4-grok.md:每问一节,结论只能是「通过 / 回滚哪几句(引用文件:行)/ 进停车位」。提 PR 到 main(模板三节,"业界方案调研"不适用),开 auto-merge。
```

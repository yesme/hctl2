# C3 合同层施工卡(派给 GPT,C2 合入后)

> 状态:待派发 · Fable 依冻结总图预写;所有者在 C2 合入后原样派给 GPT<br>
> 基线:C2 合入后的 main<br>
> 依据:[`03-approved-plan.md`](./03-approved-plan.md) §3 C3 行;[`02-target-map.md`](./02-target-map.md) §3.12–§3.18、§4.1–§4.4、§5;[`04-prohibition-whitelist.md`](./04-prohibition-whitelist.md) §1、§2、§4<br>
> 顺序修正(记入 03「拍板后修订」):`spec/README.md` 五张历史表的迁移**改在 C5 与 decision-history 同一 PR 完成**——来时路小修订台账三行直接链到这些表的锚点(`#v0103-清扫`、`#v0111-词形收敛`、`#v0122-清扫`),C3 先删会让 C5 之前的 main 断链、docs-check 变红。C3 只动总则的其余部分。

## 派发文本

```
你在 hctl2 仓库工作。先 git fetch 并把当前分支快进到 origin/main(C2 已合入),读 AGENTS.md、CONSTRAINTS.md,再读 .memo/design/doc-overhaul-20260830/ 下:施工图 README.md(§2 红线、§7 判据)、03-approved-plan.md(§3 C3 行与「拍板后修订」)、02-target-map.md(§3.12–§3.18、§4.1–§4.4、§5)、04-prohibition-whitelist.md(全文,尤其 §1 五类与 §4 施工规则)、01-inventory-prohibitions.md(spec 各文件的逐条表)、01-inventory-inconsistency.md(#7–#12)。你是唯一写手,现在开 C3(合同层)。这是本轮最危险的簇:**一字不改语义**,每条删除能指认判据编号。

范围:docs/design/spec/ 七件——README.md、project.md、task.md、run.md、agent.md、connections.md、system.md。不碰 delivery.md、decision-history.md、glossary.md、设计层(C2 已收)。

一、四组同构的合同层落点(按总图 §4 逐处,处置只有「权威定义保留」「改一句引用」「删」三种):

§4.1 加密前置——权威:spec/project.md「Room 与消息」段(:56 起)保留完整定义(第二合同前提、fresh 回读校验、事后加密的降级与换绑恢复)。其余:对象表「Chat 端口绑定」行的加密前置描述改为「准入前置见 Room 与消息」;写入合同「Chat 端口绑定」行保留「以 fresh 房间状态回读证明未启用端到端加密」的合同句、删解释;外部概念对齐 Room 行的加密说明改一句引用;connections.md 失败表「已绑定房间被开启端到端加密」行保留(可观察结果唯一登记处),措辞指向权威。

§4.2 三条底线——权威:spec/agent.md 写入合同段(:34)保留完整定义(含「在此之内 Harness 是普通 Git 用户」与可选加固)。其余:system.md「命令与跨服务正确性」(:109)保留「两类 actor 来源」(那是 system 的权威),删对底线的复述并删半句「第一阶段不设额外的用户在场证明」(对已撤销机制的负述);system.md「外部权威副作用」(:123)中 Harness 窄 principal 与凭据段缩为一句引用 spec/agent;system.md「安全边界」末条缩为一句引用,但保留「未启用加固时 Harness 与同 OS 用户其他进程处于同一信任域」的诚实声明。

§4.3 Herdr 能力条件句(拍板点 11,措辞改写、语义不变)——四项能力语汇:栅栏回显 / 逐次输入记录 / 事件游标(sequence/gap)/ 退出与停止回读。权威:spec/agent.md「运行时与观测」把「Herdr v0.8.2 的能力边界进入合同」整段改为四项条件句(「Agency 声明 X 时如何;未声明时按低信任降级如何」),其中同段栅栏回显句「Herdr v0.8.2 没有这项能力,所以第一阶段只在 HCTL 入口校验」改「未声明栅栏回显的 Agency,第一阶段只在 HCTL 入口校验」。其余逐处:spec/agent.md 写入合同 Terminal Input Lease 行「Herdr 原生写入不受该租约约束」改「provider 原生写入是否受租约约束按声明能力」;spec/agent.md「终端通道」native_interactive_allowed 段去 Herdr 点名改条件句;system.md「固定内核与受控端口」第四段(:38)改条件句;system.md「单写者」两处(:181、:183)改「不接收也不回显 generation 的 Agency…」;connections.md 启动顺序第 4 步(:99)「Herdr v0.8.2 不支持该能力」改「未声明栅栏回显的 Agency…」。**保留**:system.md:40「受租约管理的输入先经适配代码校验」(HCTL 侧规则);spec/agent.md 外部概念对齐表里 Herdr 作为外部体系词;组件表「Herdr | 第一阶段选定的 Agency」一行。v0.8.2 的缺项清单不再出现在 spec 任何位置——它的唯一家是 delivery P0 第 2 项(C4)。

§4.4 Task 完成两来源——权威:spec/task.md:68 段保留。其余:spec/task.md:76「Run 的裸终态、Harness 自述、Git commit、CI 绿色…都不是命令」按白名单 B 类删或改正面句(正确道路 :68 已写清);spec/run.md「Run → Task」(:105)保留 completion_pending 机制(Run 的权威),终结来源复述改引用;connections.md:45 连接表行保留字段、:119 的来源复述改引用;system.md:107「Task 完成只接受…」改一句引用。

二、必须先并集再引用(GLM #11):spec/run.md:49–55「Run Manifest 至少冻结」与 connections.md:61 散文清单细目互有缺项(connections 有端口绑定、网络/secret 范围;run 有放置规则)。先把两份的并集补进 spec/run.md 清单,再把 connections.md:61 改为引用该清单;不得只删一份。

三、其余处置按总图 §3.12–§3.18 逐行:spec/README「外部对齐原则」末段(受控端口隔离默认实现)缩为引用 architecture「避免供应商锁定」;**五张历史表(v0.9.1/v0.10.3/v0.11.1/v0.12.2/v0.13.0)本簇不动**,C5 与 decision-history 同 PR 迁移;spec/project.md「场景合同」中「命令走 HCTL,记录落平台」段与 spec/README 三条法、system 动作分类重复的部分缩;spec/run.md Dagu 原生 UI 段(:35)保留合同句,与 system 动作分类表 Run 行重复的解释删;system.md「组件」段中与 architecture 三个面重复处缩,保留「第一阶段由 Herdr 实现 Agency 端口」;system.md「场景与第三方适配器」段与 connections.md 同名段(:172)——只在 connections 留,system 改引用。

四、禁令按 04 白名单五类逐条处理 K3 表里 spec 七件的全部条目:A 类(合同定义句)保留、能改正面句就改、改不动原样留;B 类删,必留清单 31 条留一句,有**行级** CT 用例的改「见 CT-XXX 第 n 条」;找不到落点的先停,登记 parking-lot。合同层目标 ≤150 次(现 185)。**特别当心白名单 §1 点名的 A 类**:spec/agent.md:34/:38、spec/run.md:37/:39、spec/task.md:64/:66/:68、system.md:109/:111/:179–185、connections.md:10/:16/:115/:168——这些含「不得」是因为规则本身排他,不是负例。

五、§5「必须保留的差异」与五个「看似重复、限定条件不同」逐条核对:三种排他(ChangeSet lease / Task Run claim / Agency owner lease)各留各的对象与代次,不并成一条;Run 失败类终态不终结 Task 与 Reopen/Deleted 只作来源事实各留;chat server 不可用与房间事后加密在失败表两行都留;Dagu 直接 mutation 与 Vikunja Done 的裁决相反、原因各自写清;Harness 可读 common-dir/refs 与不获集成凭据两句都在。

行数上限:spec/README 90(五张表未迁前按 142 计,不算超)、project/task/run/agent 各 120、connections 180、system 200。

守则:不新增概念、禁令、CT 族;搬迁不扩写;引用只指现存锚点;总图容不下的事实或找不到落点的禁令——停下来登记 parking-lot,不自行补。

验收:`cd src && build/docs/materialize_repo_tree.sh && ./buck2 test root//build/docs/...` 全绿;密度报告贴 PR;PR 描述三节 + 逐文件行数(现/上限/后)+ 四组同构逐处的处置表(位置 | 权威/引用/删)+ Manifest 并集补了哪些条目 + 删除的禁令按 A/B 类计数与必留清单命中项;"业界方案调研"不适用;开 auto-merge 但**闸门前不合入**——本簇闸门是 Grok 删除安全审计(重点:清扫是否误删护栏、合并是否丢模块差异、Herdr 条件句是否仍守住「无则降级、不另写终端服务」),所有者裁决后你一次修正再合。完成后回复:七文件行数与密度变化、并集补入条目、parking-lot 新增。
```

## 给 Grok 的 C3 删除安全审计卡(与 C3 PR 同时派)

```
你在 hctl2 仓库工作。先 git fetch,读 .memo/design/doc-overhaul-20260830/ 下的施工图 README.md(§2 红线)、02-target-map.md §4/§5、04-prohibition-whitelist.md、01-inventory-dead-text.md「最危险三条」、06-c3-card.md。对象:C3 PR(codex 分支)七个 spec 文件的 diff。

只做删除安全审计,不审文笔、不提新结构、不改文件,每条引用旧来源(main 上的文件:行)与新位置:
① 四组同构合并后,每组的权威定义是否完整保留在指定位置;被改成引用的位置是否丢了限定条件(尤其 connections 失败表两行、写入合同里的 fresh 回读句)。
② 白名单 A 类(§1 点名的 15 处)是否有被当 B/C 类删掉的;被删的 B 类是否都能指认判据;必留清单 31 条是否至少留了一句。
③ Herdr 能力条件句:四项能力的「有则如何、无则降级」是否完整;是否仍守住「按实测降级、禁止另写终端服务」;v0.8.2 缺项是否已不在 spec(只在 delivery P0)。
④ Manifest 清单并集:spec/run.md 清单是否包含 connections 原有的端口绑定、网络/secret 范围;connections 是否包含 run 原有的放置规则(经引用)。
⑤ §5 五个易混边界是否各自成立、三种排他是否仍分开。

输出写到 .memo/design/doc-overhaul-20260830/review-c3-grok.md:每问一节,结论只能是「通过 / 回滚哪几句(引用文件:行)/ 进停车位」。提 PR 到 main(模板三节,"业界方案调研"不适用),开 auto-merge。
```

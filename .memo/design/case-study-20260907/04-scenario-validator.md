# 用例怎么变成机械化验证器

> 状态：讨论中 · Fable 的提案，所有者要求先看是不是可以、怎么做；不自合<br>
> 基线：main @ `65576da`（草案 v0.17.1）<br>
> 去向：`docs/design/contract-tests.md` 新增一族 CT-SCENARIO 与参考用例的正式落点；`docs/research/` 新增场景测试方法的对象文件；P2 计划的测试工作包；本 PR 只提案

## 一、能不能：能，但要分三级，一级一个"神谕"

所有者的问题是：`.memo/notes/HCTL_case_study.md` 这份用例，能不能变成机械化验证器，让用户体验以后不漂。答案是能，前提是承认它验的东西在三个不同的地方，每个地方需要不同的执行方式和不同的判对判错依据：

1. **设计文档里的说法**：现在就能机械查。用例推出的每条不变量，在设计与约束文本里对应"必须出现的说法"和"不许再出现的说法"；文档改动一旦把旧说法带回来，检查就红。它验的是文字，不是行为，但它是最早、最便宜的漂移刹车。
2. **在一台机器上模拟的多单元行为**：P2.1 有了控制面内核就能做。把用例的拓扑和步骤写成机器可读的脚本，在一个进程或一台机器上拉起多个控制面实例、假的 Agency、假的或随包的平台，按步骤跑，按不变量断言，并注入故障（控制面停、Agency 断连、时钟推进）。它验的是行为，判对判错的依据是不变量。
3. **真机**：A1 影子及之后。同一份脚本换成真 Agency、真平台、真的三台机器跑，合盖这类故障由人做、断言由命令行读。它验的是产品，判对判错的依据还是那份不变量。

用例文本本身是第零级：**参考用例**。它要有稳定的编号、正式的落点、逐条编号的不变量，三级验证器都引用它，不各自再写一份。没有这一步，后面三级各自解释用例，漂移只是换了地方。

## 二、参考用例的落点与形状

- **落点**：`docs/design/scenarios/S1-multi-unit.md`，状态"验证文档"，与 `contract-tests.md` 同一档（列可观察行为的失败用例，不描述状态机，不新增约束）。用例原文进去时改成三段：拓扑、步骤、不变量；所有者的世界观那一段留作引言。约束改了先改 spec，再改用例与 CT，顺序与现有纪律一致。
- **编号**：场景 `S1`，步骤 `S1.步骤号`，不变量 `S1.I编号`。每条不变量写四样东西：它来自用例的哪一步；它钉住设计文本的哪一句（文件加节名）；它属于哪个 CT 族；它在哪几级被检查。
- **变体要写成变体**：用例里"共用同一个对象库或单独 clone"两种都成立，就写成 `S1.V1a`、`S1.V1b` 两个变体，二级验证器两种都跑。参考用例不留没有神谕的分叉。
- **CT 族**：`contract-tests.md` 加一族 `CT-SCENARIO`，每行引用 `S1.I编号`，不重复写谓词。它与现有族的关系是索引而不是替代：`S1.I5` 说"两个控制面同时对同一仓库请求合入，平台仲裁、各记各的凭证"，谓词本体仍在 `CT-REPO`，`CT-SCENARIO` 那行只写"见 CT-REPO 第几条，用例 S1 第几步"。

## 三、从用例抽出的不变量（首版，十六条）

编号按用例段落。每条写"来自、钉住、CT 族、级别"。设计文本的落点按 `01-unit-model.md` §四 的十七条与 `03-governance-text.md` §九，等设计改法批落地后再填精确节名。

| 编号 | 不变量 | 来自 | 钉住 | CT 族 | 级别 |
| --- | --- | --- | --- | --- | --- |
| S1.I1 | 一个 Agency 同时给多个控制面供人；一次执行同一时间只听一个控制面 | 场景环境、必然情形 1 | 单元稿 §四.4 | CT-PARTICIPANT、CT-SYSTEM | 1、2、3 |
| S1.I2 | 同一模板被两个控制面雇成两个参与者，身份各在自己的控制面里 | 必然情形 1 | 单元稿 §四.5 | CT-PARTICIPANT | 1、2 |
| S1.I3 | 一个仓库同时被多个控制面开 Project；Repo Room、Board 的唯一性只在控制面之内 | mac、cloud 两节 | 单元稿 §四.7 | CT-PROJECT、CT-TASK | 1、2、3 |
| S1.I4 | 检出与对象库在参与者所在的机器上；同机参与者可共用对象库（V1a）或各自 clone（V1b） | mac 节 | 单元稿 §四.2 | CT-REPO | 1、2、3 |
| S1.I5 | 两个控制面对同一仓库同时请求合入：平台仲裁，各记各的意图与凭证，另一方的合法合入是外部事实，不补签自己的凭证 | 必然情形 2 | 单元稿 §四.8 | CT-REPO | 1、2、3 |
| S1.I6 | 共用对象库上两个控制面的引用不撞名 | 必然情形 3 | 单元稿 §四.14 | CT-REPO | 2 |
| S1.I7 | 每个消费者一份 Bundle，共同条目的摘要相同；必用材料在执行前交付为现场可反复读的精确副本 | Run 派工、治理正文讨论 | 03 §二 | CT-CONNECTION | 2、3 |
| S1.I8 | 推送在持凭据的单元上做，没有凭据时按既定中转；参与者不持材料库凭据 | 必然情形 4、所有者回答 3 与 5 | 单元稿 §四.6，03 §二 | CT-REPO、CT-PARTICIPANT | 2、3 |
| S1.I9 | 控制面没有前端照常工作；治理动作可经远程前端进入；一个前端连多个控制面，每项事实保留来源 | cloud、ubuntu 两节 | 单元稿 §四.11 | CT-WORKBENCH-IA、CT-PRODUCT | 1、3 |
| S1.I10 | 联系不上不是丢失：失联不撤权、不延长授权；恢复后核对身份与结果；截止按各自的钟 | 失败路径 (a)(b) | 单元稿 §四.9、§四.17 | CT-PARTICIPANT、CT-CONNECTION | 2、3 |
| S1.I11 | 控制面睡着时参与者干到封存，结果在 Agency 持久保管到接收方确认保全；醒来按关联键幂等收取 | 失败路径 (c)、所有者回答 4 | 单元稿 §四.16 | CT-SYSTEM、CT-CONNECTION | 2、3 |
| S1.I12 | 两个控制面的 Task 绑同一张卡：内容以任务源为准，各自验收互不触发，自动写回不冒充人的命令 | 失败路径 (e) | 单元稿 §五 | CT-TASK | 2 |
| S1.I13 | 雇佣期间不新起会话；闲置超时冬眠；会话不跨机器；复用会话不复用授权 | 所有者回答 2、单元稿 §六 | 单元稿 §六 | CT-PARTICIPANT | 2、3 |
| S1.I14 | 跨机返工在封存它的机器上做，未封存字节不搬机；旧版本评审失效对远端参与者可见 | 必然情形 7 | 单元稿 §八.5 | CT-RUN、CT-REPO | 2、3 |
| S1.I15 | 治理正文不写进被治理仓库的默认分支；参与者读的是交付副本；审计关联发布到评审请求 | 03 §八 | 03 §八 | CT-SYSTEM、CT-REPO | 1、2、3 |
| S1.I16 | 同一个人在两个控制面是两个 human actor，不悄悄归并 | 必然情形 6 | 单元稿 §八.4 | CT-PROJECT | 1、2 |

这十六条里，一级能查的是 I1、I3、I4、I5、I9、I15、I16 这类"设计文本里有一句话可钉"的；二级能跑的是全部；三级能跑的是不依赖假单元就能观察的。

## 四、第一级：设计文本的不变量检查

做法沿用现有的文档工具链（`src/build/docs/` 下的 perl 检查器与夹具），不另起一套：

- 一份 `scenario_invariants.txt`：每行一条，写"文件、必须出现还是不许出现、短语、不变量编号"。例如：`docs/design/spec/system.md | 不许 | 只能绑定本机回环地址 | S1.I9`；`docs/design/spec/repo.md | 不许 | 本系统拥有的物理执行现场 | S1.I4`；`docs/design/architecture.md | 必须 | 多个独立的控制面 | S1.I3`。
- 检查器读这份清单，短语缺失或复现即红，报告里带不变量编号，人一眼知道漂到了哪一条。
- 它只能查短语，查不了语义。它的价值是刹车：设计改法批把文本对齐到用例之后，这份清单把对齐钉住，以后谁把"一个 Repo 一个 Board"改回来，CI 先红。
- 时机：与设计改法批同一个 PR 落，因为在改法落地之前，"必须出现"的短语还不存在。新增检查脚本按 PR 契约要在调研节说清为什么自建：这里的理由是它是现有检查器的一个新清单，不是新工具。

## 五、第二级：一台机器上的多单元模拟

这是把用例变成可执行测试的主体。形状：

- **脚本是数据，不是代码**：用例的拓扑与步骤写成一份结构化文件，测试驱动器读它。首版草稿见第七节。
- **多单元同机**：一个测试进程或一台机器上拉起多个控制面实例（真的控制面代码，各用自己的数据目录与回环端口），Agency 用进程内的假实现（能按模板交付假执行体、能持久保管结果、能被切断连接），平台用随包的本地平台或它的假实现，任务源同理。假单元的行为按对应约束写，本身也受契约测试约束。
- **故障注入是脚本的一部分**：`control_sleep(mac_ctl)`、`partition(cloud_agency)`、`kill_process(participant)`、`advance_clock(...)`。第 17 条"期限各有各的钟"要求测试里的时钟可注入，否则"睡着不延长授权"验不了。
- **判对判错只认不变量**：每步之后驱动器按 `S1.I编号` 断言，断言读的是控制面的查询接口与 Agency 的持久存储，不读日志。
- **业界做法**：确定性模拟测试，一个进程里模拟多个节点、注入故障、控制时间，FoundationDB 与 TigerBeetle 用它验分布式正确性；分布式故障注入测试的方法论以 Jepsen 为代表；场景即可执行规格的写法以 Gherkin 与 Cucumber 为代表。按纪律三，落代码前要有 `docs/research/scenario-testing.md` 逐一核对这三家能借什么：多数是借想法，不借库。
- **时机**：P2.1 有控制面内核与假 Agency 就能跑 I1、I3、I7、I11 的一半；P2.3 接了真 harness 适配器后加 I13；P2.4 加 I5、I6、I8、I14、I15。放进 P2 计划各工作包的验收里，不单开阶段。

## 六、第三级：真机

同一份脚本换真单元：mac、cloud、ubuntu 三台，真 Agency、真平台。步骤由驱动器经命令行下发，断言经命令行与 bench 读；合盖、断网这类故障由人按脚本做，脚本记录预期与实际。时机是 A1 影子：在试验仓库上跑，不碰 HCTL2 自己的开发。这一级不追求自动化闭环，追求同一份不变量在真机上有人核过。

## 七、结构化脚本的首版草稿

只表达形状，字段名与格式等研究文件与 P2.1 的协议定稿后再定；这里用 YAML 只为可读。

```yaml
scenario: S1-multi-unit
repos:
  gh-jssdk: { platform: github }
  gl-jstui: { platform: gitlab }
machines:
  cloud: { agency: cloud_agency, templates: [cloud_tpl_sde, cloud_tpl_sdet, cloud_tpl_pm, cloud_tpl_sre] }
  mac:   { agency: mac_agency,   templates: [mac_tpl_sdet, mac_tpl_ops] }
  ubuntu:{ agency: ubuntu_agency, templates: [ubuntu_tpl_sde, ubuntu_tpl_pm] }
controls:
  mac_ctl:   { machine: mac,   projects: { mac_jssdk_01: gh-jssdk, mac_jssdk_02: gh-jssdk } }
  cloud_ctl: { machine: cloud, projects: { cloud_jstui_01: gl-jstui, cloud_jssdk_01: gh-jssdk } }
hires:
  - { control: mac_ctl, project: mac_jssdk_01, agency: mac_agency,    template: mac_tpl_sdet, as: mac_ptcp_jssdk_01_01 }
  - { control: mac_ctl, project: mac_jssdk_01, agency: cloud_agency,  template: cloud_tpl_sde, as: mac_ptcp_jssdk_01_04 }
  - { control: cloud_ctl, project: cloud_jssdk_01, agency: mac_agency, template: mac_tpl_ops,  as: cloud_ptcp_jssdk_01_02 }
frontends:
  ubuntu_bench: { machine: ubuntu, connects: [mac_ctl/mac_jssdk_02, cloud_ctl/cloud_jstui_01, cloud_ctl/cloud_jssdk_01] }
variants:
  V1: { a: shared_object_db_per_machine, b: clone_per_project }
steps:
  - { do: dispatch, control: mac_ctl, project: mac_jssdk_01, participant: mac_ptcp_jssdk_01_04, task: T1 }
  - { do: dispatch, control: cloud_ctl, project: cloud_jssdk_01, participant: cloud_ptcp_jssdk_01_02, task: T2 }
  - { do: seal_and_publish, both: [T1, T2] }
  - { do: request_merge, both: [T1, T2], target: gh-jssdk/main }
  - { assert: [S1.I5, S1.I6] }
  - { fault: control_sleep, control: mac_ctl }
  - { do: seal, participant: mac_ptcp_jssdk_01_04 }
  - { assert: [S1.I11] }
  - { fault: control_wake, control: mac_ctl }
  - { assert: [S1.I11, S1.I7] }
```

## 八、代价与边界

- 一级便宜但粗：只查短语，不查语义，容易被改写绕过；它防的是回退，不是新错。
- 二级要先有假单元，假单元自己也要受约束，否则测的是假东西；时钟可注入是硬前提。
- 三级不能全自动，人的操作要按脚本记下来。
- 用例进 `docs/design/scenarios/` 之后就是验证文档，改它要走 PR、配 CT 行，随意性没了；这是它当"神谕"的代价。
- 本文只提案，不加脚本、不加研究文件；落地顺序是研究文件、参考用例正式落点、一级清单随设计改法批、二级随 P2.1 起。

# C4 交付与研究索引:Grok 删除安全审计

> 状态:讨论中 · C4 闸门报告(只列表,不改 C4 文件)<br>
> 对象:PR #91 `codex/doc-overhaul-c4-delivery` @ `5971036`<br>
> 对照:[施工图](./README.md) §2 红线、[`02-target-map.md`](./02-target-map.md) §3.19/§3.23、[`04-prohibition-whitelist.md`](./04-prohibition-whitelist.md) §3、[`07-c4-card.md`](./07-c4-card.md) 审计卡<br>
> 去向:所有者裁决后交 GPT 一次修正,推回 PR #91;本文件不进合同<br>
> 结论取值只能是:通过 / 回滚哪几句 / 进停车位

旧来源行号 = 审计时 `origin/main`(`376eb50`);新位置 = PR #91 head。

## ① CT 矩阵拆出:条数、Tauri 补用例、CT-AGENT 拒绝条件

十族子弹条数(不含文件头):旧 `delivery.md` 契约测试矩阵 **104** 条;新 `contract-tests.md` **105** 条。分族:PROJECT 19、TASK 14、RUN 11、AGENT 19、CONNECTION 14、SYSTEM 14、PACKAGING 1→2、WORKBENCH-IA 7、WORKBENCH-INPUT 2、PRODUCT 3。除 PACKAGING 增 1 条与 AGENT 四处措辞外,其余族字符串逐条相同。无新族名。文件头按施工卡写了验证文档定位与「先改 spec 再加用例」;总则三句从旧矩阵开头搬入。

CT-PACKAGING 新增句(新 :125)与施工卡 (a) 字面一致,落在既有族,对应 `spec/system.md` 安全边界的壳中立合同,不是新合同。`dead_names.allowlist` 只为该句登记「Electron 安全网」。

CT-AGENT 四处改写对照:

| 旧 `delivery.md` | 新 `contract-tests.md` | 拒绝条件 |
| --- | --- | --- |
| :179 事件流没有 sequence/gap,不得当完整 trace | :74 未声明事件游标时不得当完整 trace | 与 C3「未声明则不能表示完整 trace」等价 |
| :184 Herdr API 无法执行的 fence 不得记为已生效 | :79 未声明栅栏回显时,无法执行的 fence 不得记为已生效 | 与 C3「未声明则物理 fence 记为未生效」同向;已声明不匹配仍由同族 :78 覆盖 |
| :177 `native_interactive_allowed` 下该输入不能直接产生领域结果 | :72 整句套进「Agency 未声明逐次输入记录能力时」 | **丢失**:领域结果禁令在旧句对所有 native_interactive 输入成立,与是否声明逐次记录无关。C3 仍写原生输入不是 HCTL 结果 |
| :180 不能证明同一进程和 PTY 时不得声称 exact attach;缺失 exit/stop 回执不得报成功停止 | :75 只绑「未声明退出与停止回读」 | **丢失**:C3 条件句是「未声明**或证据不足**」时只能 resume/replay/丢失。已声明但仍不能证明同一进程、或缺失回执,旧拒绝条件还在,新句覆盖不到 |

条数与 Tauri 补用例不构成回滚。拒绝条件被「未声明」辖域吃掉的是下面两句。

**结论:回滚** `docs/design/contract-tests.md:72`——「该输入不能直接产生领域结果」须对所有 `native_interactive_allowed` 输入成立,不要套在「未声明逐次输入记录」里(旧 `delivery.md:177`);`docs/design/contract-tests.md:75`——补回「不能证明同一进程和 PTY」与「缺失 exit/stop 回执」,与「未声明」并列(旧 `delivery.md:180`)。

## ② P0 核销:三项 API 清单与 Herdr 缺项

Dagu / Tuwunel / Vikunja 从旧 `delivery.md:260/:262/:263` 长段收成新 `:124/:126/:127` 一行结论 + 研究链接。调用面名单仍写在这一行里:

- Dagu:DAG 提交与启动/暂停/恢复/取消回读、`human.task` 等待/完成/回读、Engine 自行推进或重试的分歧 → `workflow-engines.md` 有 `human.task` 等待/完成 API、inline start API、已绑定 mutation 回读为分歧。
- Tuwunel:账号与房间管理、AppService 注册和事件投递、按事件 ID 读正文、加密状态回读 → `matrix-homeserver.md` 有 AppService 程序化注册与托管制品;加密/事件 ID 读正文仍由新 :126 点名,研究文件是选型与发行证据。
- Vikunja:卡片与分组读写、稳定归属、条件写入、webhook/轮询、实体 ID → `task-backends.md` 有 REST/webhook、Done 拖卡与 doer 映射。

Herdr 项(旧 :261 / 新 :125)全文相同,含 v0.8.2 缺项:原生输入不经租约、API 与原生 controller 可交错写入、事件 ring 无 sequence/gap、退出和停止回读不足,以及「不在 HCTL 内另写终端服务」。远端后端第 5 项原文保留(新 :128)。

**结论:通过。**

## ③ 打包策略:Tuwunel 句与 macOS 基线

新 `delivery.md:134` Tuwunel:「上游无 Darwin 制品,HCTL2 在自己的 GitHub Release 托管按 SHA-256 锁定的 macOS 包,日常打包消费托管制品;源码构建只用于更新托管制品。」与旧 :262 托管句、`lock.json` 的 `yesme/hctl2` macOS arm64/x86_64 URL+sha256、以及 `tuwunel_source` 隔离源码资产一致。不再写「从锁定源码原生构建」为日常路径。

macOS 最低基线仍为 15,依据改为仓库事实而非 tmux:`lock.json:45` `macos_deployment_target: "15.0"`;`.github/workflows/{code,dependencies,release,tuwunel-macos}.yml` 使用 `macos-15` / `macos-15-intel`;`platforms/macos/common.sh` 用 `vtool minos` 拒绝高于该基线的制品;`tuwunel.sh` 构建时导出 `MACOSX_DEPLOYMENT_TARGET`。本审计未下载托管 Mach-O 再跑 `vtool`;基线数值本身由 lock 与 CI 钉死,不是沿用 tmux 论证。

**结论:通过。**

## ④ 研究总表收束、复用决策、运维表、remote-control 锚点

新 `docs/research/README.md` 删 L1/L4 两张重叠表,类别 ①–⑧ 改为导读 + 指向条目索引。条目索引新旧都是 40 个研究文件;旧根 `remote-control.md` 换成 `remote-control/codex-remote-feishu.md`,其余文件仍在。方法论备忘 `methodology-landscape-20260824.md` 仍在索引。索引增「复用决策」列,取值与旧 L1/类别表边界用语同族(采用为依赖 / 适配协议 / 仅参考行为 / 历史选型等)。

「已选外部服务的运维与资源占用」节与 main 逐字相同(18 行)。

`e-l1-codex-remote-feishu` 锚点在新 `remote-control/codex-remote-feishu.md:7`;观察清单表与 2026-08-29 / 08-30 两条复核整段迁入 `remote-control/README.md`(锚点 `观察清单远程操控与会话同步` 仍在);单案正文相对链改为 `../README.md`。证据文件未删。

**结论:通过。**

## ⑤ 全库改链

活动文档已改:根 README 实现者/adapter 两条(新 :71/:72)和设计地图支持文档(新 :92)指向 `contract-tests.md`;delivery 原矩阵处留「契约测试矩阵见 contract-tests.md」(新 :109);`docs/research/runtime/README.md` 改指 `../remote-control/README.md`。`decision-history.md` 无 `delivery.md#契约测试矩阵` 或根 `remote-control.md` 残留。

`.memo` 里施工卡、权威表、Fable 通读仍出现旧锚点,那是普查/派发原文,属 C5 `.memo` 核销,不在本簇改文件范围。

**结论:通过。**

## 所有者裁决(2026-08-31,「按此修理」)

1. 接受 ① 两条回滚,修正措辞由 Fable 拟定并经所有者确认:领域结果禁令对所有 `native_interactive_allowed` 输入无条件成立,标注义务才挂「未声明逐次输入记录」;exact attach / 成功停止的拒绝条件为「未声明**或证据不足**」,与 spec/agent.md 一致。
2. 接受行数超限论证(293 vs 220 合计上限):超限全部来自 CT 104 条逐条保真,净增仅 3 行;不为凑数删验收项。
3. ②③④⑤ 通过,无修改。GPT 在 PR #91 上一次修正后合入;本簇第二轮即最后一轮。

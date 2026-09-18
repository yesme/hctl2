# 多 ctl、多 repo 的 use cases

> 状态：所有者用例的体验整理；不是已实现的行为报告。<br>
> 日期：2026-09-18<br>
> 来源：[HCTL 案例原文](../../.memo/notes/HCTL_case_study.md#hctl案例)及[术语纠正](./03-terminology-confession.md#当前怎么读旧词)。

本用例要验证：同一 repo 可以由多个 ctl 分别开展工作；一个 ctl 可以使用本地或远程 agency；一个前端可以查看多个 ctl。人从 Mac 换到 Ubuntu，是换入口，不是要求另一个 ctl 接管正在运行的 session。

原文把若干讨论分组也叫作 project。本稿按[当前导航](./04-project-navigation.md#project-入口与-rooms--kanbans--runs)纠正：`project` 是当前对应 repo 的顶层入口，`topic room` 是其中的讨论分组。原有场景标签保留，避免连人员选择一起丢掉。

## 场景环境

两个 repo：GitHub 上的 `gh-jssdk`，GitLab 上的 `gl-jstui`。机器和供给如下，不因术语修正而变化。

| 机器 | agency | harness | 可选工种 |
| --- | --- | --- | --- |
| cloud | `cloud_agency` | `cloud_codex`、`cloud_claude`、`cloud_glm` | `cloud_tpl_sde`（cloud_codex + skills）、`cloud_tpl_sdet`（cloud_codex）、`cloud_tpl_pm`（cloud_claude）、`cloud_tpl_sre`（cloud_glm） |
| Mac | `mac_agency` | `mac_gemini`、`mac_k3` | `mac_tpl_sdet`（mac_gemini）、`mac_tpl_ops`（mac_k3） |
| Ubuntu | `ubuntu_agency` | `ubuntu_glm`、`ubuntu_grok` | `ubuntu_tpl_sde`（ubuntu_grok）、`ubuntu_tpl_pm`（ubuntu_glm） |

`mac_ctl` 和 `cloud_ctl` 是两个 ctl；Ubuntu 不跑 `ubuntu_ctl`，但仍可运行 agency 和 workbench。是否提供执行、是否持有 ctl、是否打开前端是分开的选择。

## 修正后的 project 与 room 对照

当前体验有三个 ctl + project 工作范围，不是原文按旧名字数出的四个 project。

| ctl | project 对应的 repo | 原文标签 | 修正后的用途 |
| --- | --- | --- | --- |
| `mac_ctl` | `gh-jssdk` | `mac_jssdk_01`、`mac_jssdk_02` | 同一 project 内的两个 topic room，各自选人；不再为两个讨论分组建两个 project |
| `cloud_ctl` | `gl-jstui` | `cloud_jstui_01` | 这个 project 的工作范围与名册；原文不足以确定它特指主 room 还是第一间 topic room |
| `cloud_ctl` | `gh-jssdk` | `cloud_jssdk_01` | 同上；与 `mac_ctl` 中使用同一 repo 的 project 分别归属各自 ctl |

为让下面的操作路径可读，两个 cloud 范围暂以各自 project room 承接；这是**整理假设，不是所有者的原话**，留在[核对项 C1](./open-questions.md#c1cloud-用例的-room-位置)。它不影响人员、机器或 ctl 的连接关系。Mac 的两个 topic room 是用例展开后的状态；新 project 的 `rooms` 初始仍为空。

## 人员选择与执行位置

以下十一位 participant 保留原名。相同工种不等于同一个 participant；执行时的 checkout 跟着实际执行位置走，不跟着前端走。

| 工作范围 | participant | agency / 工种 | 执行位置与布局 |
| --- | --- | --- | --- |
| `mac_jssdk_01` | `mac_ptcp_jssdk_01_01` | `mac_agency` / `mac_tpl_sdet` | Mac；独立 session、checkout |
| `mac_jssdk_01` | `mac_ptcp_jssdk_01_02` | `mac_agency` / `mac_tpl_sdet` | Mac；与上一位不同的 session、checkout，可共用对象库 |
| `mac_jssdk_01` | `mac_ptcp_jssdk_01_03` | `mac_agency` / `mac_tpl_ops` | Mac；harness 不同也可与前两位共用对象库，各自 checkout |
| `mac_jssdk_01` | `mac_ptcp_jssdk_01_04` | `cloud_agency` / `cloud_tpl_sde` | cloud；不与 Mac 共用对象库 |
| `mac_jssdk_02` | `mac_ptcp_jssdk_02_01` | `mac_agency` / `mac_tpl_ops` | Mac；可与另一个 topic room 的同 repo 执行共用对象库，也可另 clone |
| `mac_jssdk_02` | `mac_ptcp_jssdk_02_02` | `ubuntu_agency` / `ubuntu_tpl_sde` | Ubuntu |
| `mac_jssdk_02` | `mac_ptcp_jssdk_02_03` | `ubuntu_agency` / `ubuntu_tpl_pm` | Ubuntu；本用例与上一位共用对象库 |
| `cloud_jstui_01` | `cloud_ptcp_jstui_01_01` | `ubuntu_agency` / `ubuntu_tpl_sde` | Ubuntu |
| `cloud_jstui_01` | `cloud_ptcp_jstui_01_02` | `mac_agency` / `mac_tpl_ops` | Mac |
| `cloud_jssdk_01` | `cloud_ptcp_jssdk_01_01` | `cloud_agency` / `cloud_tpl_pm` | cloud |
| `cloud_jssdk_01` | `cloud_ptcp_jssdk_01_02` | `mac_agency` / `mac_tpl_ops` | Mac；可与 `mac_ctl` 的同 repo 执行共用对象库 |

这些名字是场景标签，不规定产品 ID 格式。名册选好不等于已开工；表中的位置说明执行发生时在哪里。

## 操作步骤

沿用 S1.1–S1.11，纠正原来“开两个 Project”的叙述，不改用户跨机协作的目的。

| 编号 | 用户动作 | 用户看到的结果 |
| --- | --- | --- |
| S1.1 | 启动 `mac_ctl`，为 `gh-jssdk` 建 project，再展开 `mac_jssdk_01`、`mac_jssdk_02` 两个 topic room | 一个 project、主 room、两个讨论分组，不是两个 repo 或两个顶层 project |
| S1.2 | 在 `mac_jssdk_01` 从 `mac_agency` 选前三位 participant | 同工种可选两位，名单区分各自身份 |
| S1.3 | 同一 topic room 再从 `cloud_agency` 选第四位 | 本地与远程供给可以一起工作；远程执行仍在 cloud |
| S1.4 | 在 `mac_jssdk_02` 选一位 Mac、两位 Ubuntu participant | 两个 topic room 阵容独立，同 repo 的对象库可复用 |
| S1.5 | 用 `mac_cli` 和 `mac_bench` 同时连接 `mac_ctl` | 两个前端看到同一 project 及其中两个 topic room，不各复制一份工作 |
| S1.6 | 启动 `cloud_ctl`，分别为 `gl-jstui`、`gh-jssdk` 建 project | 两个 project；与 Mac 上的工作分别归属各自 ctl |
| S1.7 | 在 `cloud_jstui_01` 选择表中两位远程 participant | cloud 的 ctl 使用 Ubuntu 和 Mac 的 agency |
| S1.8 | 在 `cloud_jssdk_01` 选择表中两位 participant | 同一 `mac_agency` 同时服务两个 ctl |
| S1.9 | cloud 创建时用 `cloud_cli`，之后不常驻本机前端 | 工作继续；合入、完成 task 等需要人的动作可由远程前端处理 |
| S1.10 | Ubuntu 只启动 `ubuntu_bench`，不启动 ctl | 仍能观察和操作远程工作；它已有 agency 不改变这一点 |
| S1.11 | `ubuntu_bench` 打开 `mac_jssdk_02`、`cloud_jstui_01`、`cloud_jssdk_01` | 经两个 ctl 进入三个 project 的相应工作位置；每次动作清楚属于哪个 ctl、哪个 project |

## 必然发生的情形

这些是原用例已有的要求，不是为了新目录增加的系统能力。

| 编号 | 情形 | 体验上不能丢的结果 |
| --- | --- | --- |
| S1.N1 | `mac_tpl_ops` 同时被两个 ctl 选用 | 分别是各处的 participant；共用工种不合并执行或授权 |
| S1.N2 | 两个 ctl 同时向 `gh-jssdk` 开 PR、请求合入 main | 各自工作可推进，遵守同一平台的合入规则；一方合入可被另一方看见，不凭空变成另一方的完成记录 |
| S1.N3 | 两个 ctl 的执行在 Mac 上共用对象库 | 共享代码存储可行；不因共用对象库而变成同一个 session 或同一份工作 |
| S1.N4 | Ubuntu 的执行要向两个 repo 推送，凭据可能在别处 | 有授权的交付能走通；不能因为用户从另一台机器看工作，就假定凭据也搬过去了 |
| S1.N5 | `gl-jstui` 等待第二个平台适配器 | 清楚显示尚未支持的操作；不能把未验证路径报成已完整支持 |
| S1.N6 | 同一个人访问两个 ctl | 两边的动作归属清楚；不自动把一方的授权用于另一方 |
| S1.N7 | cloud 的封存版本被驳回，需要返工 | 返工仍在 cloud，用原 checkout 或重建；不搬未封存的修改；远程观察者能看到旧评审失效及后续进度 |
| S1.N8 | participant 不响应而 ctl 还活着，或 agency 也失联 | agency 能在已接受的范围内恢复执行；换另一家 agency 涉及成本和授权，由人决定；旧执行回来不能继续提交已经失效的工作 |

N8 的原文还要求机械核对旧执行是否仍有资格提交，不能只靠对 harness 的文字叮嘱。具体钩子、协议和恢复机制属于后续设计，不在体验文档另造一套。

## 两种布局都要覆盖

- **S1.V1a：共用对象库。** 同机、同 repo 的独立 checkout 可以共享对象库，包括来自不同 topic room、不同 ctl 的执行。
- **S1.V1b：各自 clone。** 同样的协作路径也能在彼此不共享对象库时成立。

两者都是原用例，不选其中一个冒充唯一合法布局。共享 Git 元数据本身没有问题；这不是要求向 harness 隐藏 common-dir 或目标 refs。

## 与既有验证的关系

[旧 S1 验证映射](../design/scenarios/S1-multi-unit.md#四不变量)保留 v0.18.6 的 CT 对应关系。尤其其中“同 ctl、同 repo 建两个旧 Project”的 CT 描述与本稿“两间 topic room”不同，尚待规范与测试一起修订，不能宣称本次整理已经让测试覆盖了新体验。

多 source 导航是[本轮明确的体验](./04-project-navigation.md#kanbans每个-source-一个入口)，不再因为它曾列在旧 S1 的“用例外”就把它排除出产品要求；它不改变本页的两 ctl、三 agency、多前端拓扑。

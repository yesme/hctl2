# 多 Control、多 Repo 的用例

<a id="多-ctl多-repo-的-use-cases"></a>

> 状态：所有者用例的体验整理；不是已实现的行为报告。<br>
> 日期：2026-09-19<br>
> 来源：[HCTL 案例原文](../../.memo/notes/HCTL_case_study.md#hctl案例)、[术语纠正](./03-terminology-confession.md#当前怎么读旧词)与[已确认的组织结构](./04-project-navigation.md#project-入口与-rooms--kanbans--runs)。

本用例要验证：同一 Control 可以为同一 Repo 建多个 Project，同一 Repo 也可以由多个 Control 分别开展工作；一个 Control 可以使用本地或远程 Agency；一个前端可以查看多个 Control。人从 Mac 换到 Ubuntu，是换入口，不是要求另一个 Control 接管正在运行的会话。

Project 是当前关联一个 Repo 的顶层工作范围，Topic Room 是其中的讨论场所。所有者在 C2 确认：Mac 的两个标签分别是两个 Project 的主 Room，不是同一 Project 的两间 Topic Room。本稿保留原文四个 Project、原名册与执行位置，不因它们使用同一 Repo 就合并。

## 场景环境

两个 Repo：GitHub 上的 `gh-jssdk`，GitLab 上的 `gl-jstui`。机器和供给如下，不因术语修正而变化。

| 机器 | Agency | Harness | 可选工种 |
| --- | --- | --- | --- |
| cloud | `cloud_agency` | `cloud_codex`、`cloud_claude`、`cloud_glm` | `cloud_tpl_sde`（cloud_codex + Skills）、`cloud_tpl_sdet`（cloud_codex）、`cloud_tpl_pm`（cloud_claude）、`cloud_tpl_sre`（cloud_glm） |
| Mac | `mac_agency` | `mac_gemini`、`mac_k3` | `mac_tpl_sdet`（mac_gemini）、`mac_tpl_ops`（mac_k3） |
| Ubuntu | `ubuntu_agency` | `ubuntu_glm`、`ubuntu_grok` | `ubuntu_tpl_sde`（ubuntu_grok）、`ubuntu_tpl_pm`（ubuntu_glm） |

`mac_ctl` 和 `cloud_ctl` 是两个 Control；Ubuntu 不跑 `ubuntu_ctl`，但仍可运行 Agency 和 Workbench。是否提供执行、是否持有 Control、是否打开前端是分开的选择。

## 修正后的 Project 与 Room 对照

当前体验有四个 Project：Mac 的两个使用同一 Repo，cloud 的两个分别使用两个 Repo。每个 Project 各有一间主 Room；同 Control、同 Repo 不等于同一个 Project。

| Control | Project 对应的 Repo | 原文标签 | 修正后的用途 |
| --- | --- | --- | --- |
| `mac_ctl` | `gh-jssdk` | `mac_jssdk_01` | 第一个 Project 的主 Room 及其名册，从该 Project 名称进入 |
| `mac_ctl` | `gh-jssdk` | `mac_jssdk_02` | 第二个 Project 的主 Room 及其名册；与上一行使用同一 Repo，仍是独立的 Project |
| `cloud_ctl` | `gl-jstui` | `cloud_jstui_01` | 这个 Project 的主 Room 及其名册 |
| `cloud_ctl` | `gh-jssdk` | `cloud_jssdk_01` | 这个 Project 的主 Room 及其名册；与 `mac_ctl` 中使用同一 Repo 的两个 Project 分别归属各自 Control |

四个标签都是各自 Project 的主 Repo Room，也就是本目录所称的 Project Room，分别见 [C1](./open-questions.md#c1cloud-用例的-room-位置)、[C2](./open-questions.md#c2mac-用例的-room-位置)。每个新 Project 的 Rooms 初始为空；后来展开 Topic 时按 [T1](./02-user-journey.md#t1聊天归纳接受或忽略-topic-建议)从本 Project 主 Room 的相关讨论构造前情提要。

每个 Room 分别选人；随后建立 Run 时，也为该 Run 独立选择施工与评审 Participant，不把 Room 名单直接当成 Run 名单。同 Project 的 Room 可以引用同一 Task 或 Run，引用不搬动原消息、Task 或 Run 的归属；完整关系见 [04](./04-project-navigation.md#松散耦合具体意味着什么)。

## 人员选择与执行位置

以下十一位 Participant 保留原名。相同工种不等于同一个 Participant；执行时的工作副本跟着实际执行位置走，不跟着前端走。

| 所在 Room | Participant | Agency / 工种 | 执行位置与布局 |
| --- | --- | --- | --- |
| `mac_jssdk_01` | `mac_ptcp_jssdk_01_01` | `mac_agency` / `mac_tpl_sdet` | Mac；独立会话、工作副本 |
| `mac_jssdk_01` | `mac_ptcp_jssdk_01_02` | `mac_agency` / `mac_tpl_sdet` | Mac；与上一位不同的会话、工作副本，可共用对象库 |
| `mac_jssdk_01` | `mac_ptcp_jssdk_01_03` | `mac_agency` / `mac_tpl_ops` | Mac；Harness 不同也可与前两位共用对象库，各自工作副本 |
| `mac_jssdk_01` | `mac_ptcp_jssdk_01_04` | `cloud_agency` / `cloud_tpl_sde` | cloud；不与 Mac 共用对象库 |
| `mac_jssdk_02` | `mac_ptcp_jssdk_02_01` | `mac_agency` / `mac_tpl_ops` | Mac；可与另一 Project 的同 Repo 执行共用对象库，也可另行克隆 |
| `mac_jssdk_02` | `mac_ptcp_jssdk_02_02` | `ubuntu_agency` / `ubuntu_tpl_sde` | Ubuntu |
| `mac_jssdk_02` | `mac_ptcp_jssdk_02_03` | `ubuntu_agency` / `ubuntu_tpl_pm` | Ubuntu；本用例与上一位共用对象库 |
| `cloud_jstui_01` | `cloud_ptcp_jstui_01_01` | `ubuntu_agency` / `ubuntu_tpl_sde` | Ubuntu |
| `cloud_jstui_01` | `cloud_ptcp_jstui_01_02` | `mac_agency` / `mac_tpl_ops` | Mac |
| `cloud_jssdk_01` | `cloud_ptcp_jssdk_01_01` | `cloud_agency` / `cloud_tpl_pm` | cloud |
| `cloud_jssdk_01` | `cloud_ptcp_jssdk_01_02` | `mac_agency` / `mac_tpl_ops` | Mac；可与 `mac_ctl` 的同 Repo 执行共用对象库 |

这些名字是场景标签，不规定产品 ID 格式。名册选好不等于已开工；表中的位置说明执行发生时在哪里。布局是本例的现场描述，不要求 Control 管理 Agency 内部目录。

## 操作步骤

沿用 S1.1–S1.11，保留原来“同一 Repo 开两个 Project”的动作。下表核对多机连接与各 Project 主 Room 的选人，不假定已经有 Topic Room；日常创建路径见 [P1–P3](./02-user-journey.md#新建-project)，以后展开 Topic 才走 [T1](./02-user-journey.md#t1聊天归纳接受或忽略-topic-建议)。

| 编号 | 用户动作 | 用户看到的结果 |
| --- | --- | --- |
| S1.1 | 启动 `mac_ctl`，基于 `gh-jssdk` 建立 `mac_jssdk_01`、`mac_jssdk_02` 两个 Project | 两个独立的 Project 入口，各自打开主 Room；各自的 Rooms 初始为空，不因 Repo 相同而合并 |
| S1.2 | 在 `mac_jssdk_01` 从 `mac_agency` 选前三位 Participant | 同工种可选两位，名单区分各自身份 |
| S1.3 | 同一主 Room 再从 `cloud_agency` 选第四位 | 本地与远程供给可以一起工作；远程执行仍在 cloud |
| S1.4 | 在 `mac_jssdk_02` 的主 Room 选一位 Mac、两位 Ubuntu Participant | 两个 Project 的主 Room 阵容独立，同 Repo 的对象库可复用 |
| S1.5 | 用 `mac_cli` 和 `mac_bench` 同时连接 `mac_ctl` | 两个前端看到相同的两个 Project 及各自主 Room，不各复制一份工作，也不合并两个 Project |
| S1.6 | 启动 `cloud_ctl`，分别为 `gl-jstui`、`gh-jssdk` 建 Project | 两个 Project；与 Mac 上的工作分别归属各自 Control |
| S1.7 | 在 `cloud_jstui_01` 选择表中两位远程 Participant | cloud 的 Control 使用 Ubuntu 和 Mac 的 Agency |
| S1.8 | 在 `cloud_jssdk_01` 选择表中两位 Participant | 同一 `mac_agency` 同时服务两个 Control |
| S1.9 | cloud 创建时用 `cloud_cli`，之后不常驻本机前端 | 工作继续；合入、完成 Task 等需要人的动作可由远程前端处理 |
| S1.10 | Ubuntu 只启动 `ubuntu_bench`，不启动 Control | 仍能观察和操作远程工作；它已有 Agency 不改变这一点 |
| S1.11 | `ubuntu_bench` 打开 `mac_jssdk_02`、`cloud_jstui_01`、`cloud_jssdk_01` | 经两个 Control 进入三个 Project 的主 Room；每次动作清楚属于哪个 Control、哪个 Project |

## 必然发生的情形

这些是原用例已有的要求，不是为了新目录增加的系统能力。

| 编号 | 情形 | 体验上不能丢的结果 |
| --- | --- | --- |
| S1.N1 | `mac_tpl_ops` 同时被两个 Control 选用 | 分别是各处的 Participant；共用工种不合并执行或授权 |
| S1.N2 | 两个 Control 同时向 `gh-jssdk` 开 PR、请求合入 main | 各自工作可推进，遵守同一平台的合入规则；一方合入可被另一方看见，不凭空变成另一方的完成记录 |
| S1.N3 | 两个 Control 的执行在 Mac 上共用对象库 | 共享代码存储可行；不因共用对象库而变成同一个会话或同一份工作 |
| S1.N4 | Ubuntu 的执行要向两个 Repo 推送，凭据可能在别处 | 有授权的交付能走通；不能因为用户从另一台机器看工作，就假定凭据也搬过去了 |
| S1.N5 | `gl-jstui` 等待第二个平台适配器 | 清楚显示尚未支持的操作；不能把未验证路径报成已完整支持 |
| S1.N6 | 同一个人访问两个 Control | 两边的动作归属清楚；不自动把一方的授权用于另一方 |
| S1.N7 | cloud 的封存版本被驳回，需要返工 | 返工仍在 cloud，用原工作副本或重建；不搬未封存的修改；远程观察者能看到旧评审失效及后续进度 |
| S1.N8 | Participant 不响应而 Control 还活着，或 Agency 也失联 | Agency 能在已接受的范围内恢复执行；换另一家 Agency 涉及成本和授权，由人决定；旧执行回来不能继续提交已经失效的工作 |

N8 的原文还要求机械核对旧执行是否仍有资格提交，不能只靠对 Harness 的文字叮嘱。具体钩子、协议和恢复机制属于后续设计，不在体验文档另造一套。

## 两种布局都要覆盖

- **S1.V1a：共用对象库。** 同机、同 Repo 的独立工作副本可以共享对象库，包括来自不同 Project、不同 Control 的执行。
- **S1.V1b：各自克隆。** 同样的协作路径也能在彼此不共享对象库时成立。

两者都是原用例，不选其中一个冒充唯一合法布局。共享 Git 元数据本身没有问题；这不是要求向 Harness 隐藏 common-dir 或目标 refs。

## 与既有验证的关系

[S1 验证映射](../design/scenarios/S1-multi-unit.md#四不变量)中“同 Control、同 Repo 建两个 Project”的事实，与本次 C2 确认一致，应保留相应 CT。此前整理将它判作应改成两间 Topic Room，是整理错误，不是所有者改变了这项用例。

S1 §二/§四/§五及契约测试矩阵的误判说明已随 #257 纠正；每个 Project 的主 Room、多 Source 导航等已配对应约束和 [S3 验收路径](../design/scenarios/S3-user-journey.md)。这是要求与映射的同步，不是产品行为已通过验证，具体位置见[接手清单](./open-questions.md#规范对齐清单)。

多 Source 导航是[已明确的体验](./04-project-navigation.md#kanbans每个-source-一个入口)，不再因为它曾列在旧 S1 的“用例外”就把它排除出产品要求；它不改变本页的两个 Control、三个 Agency、多前端拓扑。

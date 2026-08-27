# Herdr

> 类别：⑥ 机械后端与基础设施 · 证据编号：E-L1-HERDR、E-L2-HERDR-BOUNDARY<br>
> 状态：证据审计 · 钉定版本与许可见文内「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

<a id="e-l1-herdr"></a>
## E-L1-HERDR · Herdr

### 设计亮点与跨层画像

Herdr 在 L1 的主要价值是把终端所有权、输入权和 Agent 状态写入权拆成三个问题。在默认的持久会话模式中，后台服务持有 PTY、解析器、检测任务与通道，客户端只负责连接和显示；`--no-session` 是用于调试或兼容的单进程例外。第三方桥接还使用不同命令进入[只读观察与可写控制](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/src/client/mod.rs#L836-L910)：多个观察者可以并存，控制方则独占输入与尺寸，显式 `--takeover` 会替换旧控制方。[服务端的单写者检查](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/src/server/headless.rs#L2566-L2669)使接管不依赖前端按钮。

它也把原始终端控制与 Agent 语义控制分开。[Pane 命令与 Agent 命令](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/docs/next/website/src/content/docs/agent-automation.mdx#L59-L80)分别面向当前终端和当前被识别的 Agent；当目标 Agent 已不再控制该窗格时，语义命令会拒绝操作。状态层进一步仲裁完整生命周期钩子、只报告会话身份的钩子、屏幕状态降级信号、进程退出与事件序号：[完整生命周期钩子](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/src/terminal/state.rs#L598-L745)可以取得状态写入权，只报告会话身份的钩子不能；进程退出和过期序号会撤销或拒绝旧报告。这是 Herdr 在 L2 的专项边界价值：运行状态信号可以驱动关注、等待和诊断，但不能签发 Task、Run、Verdict 或 Receipt。

### 审计基线

固定基线为 [`v0.8.0 / 346411fa`](https://github.com/herdrdev/herdr/tree/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7)（2026-08-03，Apache-2.0）。该版本的 [changelog](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/docs/next/CHANGELOG.md#L5-L27)记录许可证从 AGPL-3.0-or-later 改为 Apache-2.0，许可证正文见 [LICENSE](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/LICENSE)。源码中的 [TerminalRuntimeRegistry](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/src/terminal/runtime_registry.rs#L5-L12)明确说明运行中的终端由 server 持有；官网的[概念说明](https://herdr.dev/docs/concepts/)、[会话状态与恢复](https://herdr.dev/docs/session-state/)、[持久化与远程接入](https://herdr.dev/docs/persistence-remote/)和 [Agent 自动化](https://herdr.dev/docs/agent-automation/)与固定版本文档一致。

### 恢复边界

Herdr 对恢复能力给出了可验证的分级，而不是用一个“恢复”覆盖所有情况：

| 场景 | 实际保留的内容 | HCTL 应如何归类 |
| --- | --- | --- |
| 客户端断开，后台服务仍存活 | 原 PTY、进程、终端内容和 Agent 会话都继续存在 | 可以接回同一运行实例 |
| 后台服务停止后重启 | 恢复 workspace/tab/pane、工作目录、布局和焦点；原进程已经消失 | 布局恢复，不是接回原 PTY |
| 开启窗格历史 | 可以显示近期终端内容；默认关闭，内容可能包含密钥与提示 | 只读历史显示，不是进程或证据恢复 |
| 服务提供方原生会话恢复 | 依靠官方集成报告的会话引用启动新的 Agent 进程 | 语义会话恢复，不是原 Attempt |
| `--handoff` | 在受支持的更新或远程替换中尽力移交 PTY 与进程；功能为实验性且需主动开启 | 需要新代次、对账和失败降级路径，不能视为必然成功 |

固定版本的[恢复说明](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/docs/next/website/src/content/docs/session-state.mdx#L6-L109)还明确指出：实时移交不保留进行中的 CLI/API 请求、等待、订阅流、客户端连接或窗格间消息。`agent prompt --wait` [只等待生命周期状态，不追踪某一轮对话](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/docs/next/website/src/content/docs/agent-automation.mdx#L72-L80)；已有活动轮次也可能满足等待。因此 `idle/working/blocked/done` 不能作为一次提示已经完成，更不能作为领域完成依据。

### 采用与边界

HCTL 采用 Herdr 的后台 PTY 所有权、观察/控制分离、单写者与显式接管、原始/语义控制面分离、状态信号仲裁和准确的恢复词汇。Herdr 的[显式 worktree 创建、打开与删除](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/docs/next/website/src/content/docs/cli-reference.mdx#L135-L146)，以及 [SSH 瘦客户端和直接接入](https://github.com/herdrdev/herdr/blob/346411fa21afd297f5ed3b3fa56f9e3fbf7654b7/docs/next/website/src/content/docs/persistence-remote.mdx#L38-L165)，也可以补充 L1 的安全清理与远程操作设计。

Herdr 的控制方记录是进程内的客户端所有者映射，没有代次、TTL、持久确认游标或跨重启租约，因此不能直接替代 `Terminal Input Lease`。固定基线的 SCM 操作面覆盖 worktree 生命周期、分支以及 ahead/behind 状态，但不覆盖 Stably Orca 那样的内建 diff、分块暂存、commit/push 与 PR 评审交付链。Workspace、窗格、Agent 名称和服务提供方会话引用也不承担 HCTL 的 Project、Task、Run 或 Attempt 身份。Herdr 因而是终端所有权、控制、状态仲裁与恢复方面的 L1 专项参考，并通过[运行信号边界](#e-l2-herdr-boundary)为 L2 提供独立证据。


<a id="e-l2-herdr-boundary"></a>
## E-L2-HERDR-BOUNDARY · Herdr 运行信号边界

Herdr 不提供持久 Workflow，但它对“谁可以写 Agent 状态”处理得足够深入，构成 L2 的边界证据：一个活动窗格只接受一个状态来源；完整生命周期钩子活跃时优先于屏幕状态降级信号，只报告会话身份的钩子不获得生命周期状态写入权，进程退出和事件序号又会撤销或拒绝过期报告。与此同时，`agent prompt --wait` 明确不追踪某一轮对话，已有活动轮次也可能满足等待。因此 HCTL 可以采用它的信号仲裁思路，却必须把 `idle/working/blocked/done` 限定为 L1 观测，不能据此完成 Task、Run、Verdict 或 Receipt。固定源码见 [E-L1-HERDR](#e-l1-herdr)。

## 复核记录

- **2026-08-24**：主干已到 v0.8.2，并出现 plugins/marketplace 与 skills 方向的新投入（属 terminal 场景内扩展）。提交直方图证实产品代码几乎 100% 落在 terminal 场景（含 vendored 终端仿真），无任何 room/kanban/workflow 目录——它是 tmux 一类加了 harness 状态检测的机械运行时，不是协作平台。

# 本地 Agency 参考实现

发布包自带的参与者供给方（Agency，派出方）的第一方实现。它回答「本机有哪些参与者可派、带什么方法、条款是什么」，并按冻结的执行规格交付执行体端点；参与者身份、授权、租约、代次和结果验收都不在这里，它们在 control 账本。设计见 [Participant 与 Terminal](../../docs/design/participant.md#agency-与执行体)，约束见 [Participant 模块约束](../../docs/design/spec/participant.md)。

它在 Herdr 外面只多三样东西：

| 部件 | 状态 | 说明 |
| --- | --- | --- |
| 技能目录 `skills/` | 已有 | 每个子目录一个 Skill，harness 原生格式：`SKILL.md` 供 Claude Code 及兼容 harness 装载，`agents/openai.yaml` 供 Codex。首次运行种到 `~/.hctl2/` 的技能目录，由 harness 自己装载；HCTL 只冻结引用与指纹并标注可核验性（见约束「Skill 与申报」） |
| 可用性申报 | 待建 | 向 control 报名册：本机装了哪些 Harness、哪些 Skill、各自的精确版本与指纹 |
| 与 control 对话的适配器 | 待建 | 落在 `hctl2-control` 的 Herdr 适配代码旁；进程、PTY、终端会话与 TUI 全部由 Herdr 提供，这里不重新实现 |

## skills/

| Skill | 触发方式 | 用途 | 来源 |
| --- | --- | --- | --- |
| [hctl2-shaping](./skills/hctl2-shaping/SKILL.md) | 人发起 | 塑形：把一个还说不清的目标审问成三张清单（已决、尚未定形、出界），产出只有四种建议（创建 Request、开 Scoped Room、雾毕业为 Task、更新 Project 范围） | 改编自 mattpocock/skills 的 grilling 与 wayfinder（MIT，许可证随目录） |

Skill 分两种触发方式，沿用上游的约定：**人发起**的 Skill 是阶段切换（塑形、施工、评审），只有人能按；**模型可自取**的 Skill 是阶段内的纪律（查证据、写测试），模型可以自己伸手拿。判据是「模型能不能有意义地自己伸手拿它」，不是「它是否可复用」。

## 进包

`root//agency:skills` 把技能目录声明为 Buck2 文件组；`root//packaging/release:complete` 把它作为输入交给 `assemble.sh --agency-skills`，安装到发布包的 `payload/share/hctl2/agency/skills/`，与其余 payload 文件一样进入 `PAYLOAD.sha256` 与 SBOM。`test-package.sh` 断言 Skill 与许可证文件在包内。

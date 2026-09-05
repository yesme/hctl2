# 提交署名的机械关卡：任务书

> 状态：待拍板 · 白名单字符串与检查口径等所有者一句话<br>
> 基线：main @ `1d001ad`（草案 v0.17.0）<br>
> 去向：`.github/workflows/pr-contract.yml` 加一步、`AGENTS.md` 加「提交署名」一节、仓库设置只留 squash 并要求线性历史；不改约束层

所有者 2026-09-06 提出：coauthor 要有纪律，并且要机械执行。`01-scm-module.md` 附录二记了审计结论，四家评审都维持「另开小 PR、与第五模块方案分开」。本文是那份小 PR 的任务书。

## 一、审计事实（2026-09-06，main 上 45 个已合并 PR 加本轮）

- Fable 自 #155 起每个提交都带 `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>`。
- Codex 的提交作者是机器人身份 `Codex GPT-5.5 <281847692+a-chatgpt-codex-bot[bot]@users.noreply.github.com>`（#176、#182），模型名随版本变。
- Gemini（Antigravity）的提交作者是 `Gemini 3.8 Flash <281848501+a-gemini-cli-bot[bot]@users.noreply.github.com>`（#180 的 merge commit 里）。
- Grok、GLM 的提交从未带任何署名（#177、#179）；它们和 Fable 一样在所有者的 Mac 上以机器身份 `Haibara AI <haibara@JackydeMac-mini.local>` 提交，靠作者字段分不出是哪家。
- 仓库允许 merge / squash / rebase 三种合并；`agy/main-mac` 两次走了 merge commit（#180 等）。分支保护要求 CI gate、Release gate、PR contract 三个检查、与目标同步、线程解决，管理员也受约束，正式批准数为零。
- 所有评论与提交都经所有者的 gh 令牌或 GitHub App 机器人发出；仓库公开，只有 yesme 有推送权，外人用不了这些机器人身份。

## 二、要机械化的规则

一句话：**每个由 harness 产出的提交都带一条能认出是哪家的 `Co-Authored-By:` 尾注；合并只走 squash；历史线性。**

判「由 harness 产出」不能靠内容，只能靠作者身份：提交作者是所有者本人的 GitHub 身份时视为人手提交，不要求尾注；其他作者（Mac 上的机器身份、各家机器人身份）一律要求尾注，且尾注值必须匹配白名单里某一家的模式。这样 Grok、GLM 与 Fable 在同一台机器上提交也能分出是谁，Codex、Gemini 的机器人提交继续用它们已有的写法。

白名单按厂商写成模式，不钉具体模型版本（模型名会随版本变，钉死会逼每次升级改白名单）：

| 家 | 尾注模式（正则，逐行匹配 `Co-Authored-By:` 的值） | 依据 |
| --- | --- | --- |
| Claude（Fable / Opus 等） | `^Claude .+ <noreply@anthropic\.com>$` | 现有 #155 起的写法 |
| Codex | `^Codex .+ <[0-9]+\+a-chatgpt-codex-bot\[bot\]@users\.noreply\.github\.com>$` | 现有机器人作者身份 |
| Gemini / Antigravity | `^Gemini .+ <[0-9]+\+a-gemini-cli-bot\[bot\]@users\.noreply\.github\.com>$` | 现有机器人作者身份 |
| Grok | 待定（建议 `^Grok .+ <noreply@x\.ai>$`） | 从未署名，所有者定 |
| GLM | 待定（建议 `^GLM .+ <noreply@z\.ai>$`） | 从未署名，所有者定 |

检查落在 `pr-contract.yml` 现有的那一步里，不新增脚本文件：对 `origin/<base>...HEAD` 逐个提交读作者邮箱与尾注，作者邮箱不在所有者身份列表且没有任一匹配白名单的 `Co-Authored-By:` 行即失败，错误信息写明是哪个提交、该加哪家的尾注。合并方式与线性历史由仓库设置保证（GitHub 的「只允许 squash」与「要求线性历史」两个开关），不由 CI 判。

`AGENTS.md` 加一节「提交署名」，写三句：每个 harness 产出的提交末尾带本家的 `Co-Authored-By:` 尾注，模式见 `pr-contract.yml`；合并只用 squash；人手提交不要求尾注。CLAUDE.md 系统里 Fable 的尾注规则已经是这样，不用改。

## 三、要所有者一句话的

1. **Grok 与 GLM 的固定尾注**：各家在自己的会话里配置一次即可；请给出邮箱域（建议值见上表）。
2. **所有者身份列表**：判「人手提交」用的作者邮箱清单（GitHub 主邮箱与 noreply 邮箱），写进 workflow 的一个变量。
3. **仓库设置**：只留 squash、要求线性历史——这两项只有所有者能改，改完后 `agy/main-mac` 那类 merge commit 不会再出现。
4. **尾注之外要不要机械检查会话链接**（Fable 的 `Claude-Session:` 一类）：建议不查，那是各家自愿的留痕，不是纪律。

## 四、分工

工作流改动与 `AGENTS.md` 一节由 Codex 写（代码归 Grok/Codex 的分工），Fable 审；PR 描述的调研节要写清为什么不用现成的 DCO / commitlint 类 GitHub Action：它们检查的是 `Signed-off-by` 或提交格式，不认厂商白名单，而且我们只需在已有一步里加十几行，不值得引一个 Action 依赖。

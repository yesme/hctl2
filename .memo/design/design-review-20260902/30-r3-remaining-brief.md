# 第三轮剩余批次任务书（可分派给其他 harness）

> 状态：待分派（2026-09-03）<br>
> 依据：`21-r2-rulings.md`（所有者裁决）与 `13-findings.md`（核验后的发现清单，「写法」类条目在此）。已合入的三批：PR #148 写法统一、PR #149 约束含义、PR #150 愿景与架构改写，基线 v0.16.3。<br>
> 纪律：每批一个 PR；PR 描述按模板三节；新增脚本要在调研节论证；动第三方依赖前先落 `docs/research/` 对象文件；研究文件发布后正文不改、只在文末追加复核记录；「张力」写「冲突」；位置引用写「文件 §节名」，不写行号。

## 批 4 · 写作规则机械化（裁决 I-04 + W-20、A-31 + A-38、I-17）

1. 四个检查器进文档 CI 门（`src/build/docs/`，与现有 `check_links.pl`、`check_dead_names.sh` 同款体例，各带 allowlist）：
   - 路由表越层词：按 `spec/README.md` §核心产品词 的「可见性」列，愿景层出现「治理内部」词即报；架构层出现约束层词汇（词汇索引里除八个高频词外的名字）即报。
   - 产品名越层：设计层正文出现 Tuwunel、Vikunja、Dagu、Herdr、Cinny 等实现名即报（允许列表：`delivery.md`、`contract-tests.md`、研究层；`participant.md` 的 Herdr 提及待批 4 第 3 项清理后再收紧）。
   - 驼峰自造名：保留 ChangeSet、ReviewSubjectRef 与代码体内的标识符，其余 CamelCase 即报。
   - 首现中文对照：每份设计层文档里英文专名（核心产品词与八个高频词）首次出现须带中文对照。
   - 「需要」在约束层只许出现在「需要关注」；「同层同表中英夹杂」做窄版报告（`report_*`），不拦 PR。
2. 词汇表补「语义名 ↔ 标识符」对照表，由脚本从约束层的代码体字段生成（`references/glossary.md` 末尾一节，生成物入库并有 CT 比对）；枚举值统一为「中文含义（`代码体`）」写法（A-31 + A-38）。
3. 写法遗留（`13-findings.md` 写法类，挑影响读者最大的先做）：`participant.md` 的 Herdr 提及收敛到「Agency 与执行体」一节两三处（A-23）、二十条关键规则按主题分组（A-24）；约束层 112 句「必须 / 不得 / 只能」无机制词的条目，加机制词或降为说明（I-03 的其余部分，清单在 `10-inventory.md` §强制手段）。
4. PR 门：`.github/workflows/pr-contract.yml` 的自建信号加「已有脚本单次 PR 净增超过阈值（建议 100 行）也算自建」（I-17）。

## 批 5 · 部件与代码（裁决 I-10、I-09、I-05、I-07、I-11、M-30 wait、I-13 调研）

先调研再动代码，每项一个 `docs/research/` 对象文件（一对象一文件，钉版本）：

1. **Process Compose 接管服务生命周期**（I-10）：限时验证 → 一份 `process-compose.yaml` 接管 tuwunel、cinny、vikunja、dagu、herdr → 按服务拆配置模板（每服务一文件）→ 删 `common/runtime.sh` 里的通用监督器半边与 `start.sh` / `stop.sh` / `status.sh` 的按动词拆分。DotSlash 清单补 Windows amd64。用户包多约 15–16 MB 的一个 Go 二进制，已由所有者接受。
2. **Reindeer 换官方二进制**（I-09）：`src/build/tools/reindeer` 改 DotSlash 清单，删源码编译启动器。
3. **五处通用机制用现成库**（I-05）：`serde_jcs`（RFC 8785，CT 钉官方测试向量）、`fd-lock`、SQLite Online Backup API、`keyring`、FTS5；技术基线写明；outbox / 租约 / 代次维持自研。
4. **借用等级用语**（I-07）：研究层 README 与 `delivery.md` §选型判据 写偏好顺序：跨平台二进制 > SDK > 复制代码 > 借鉴想法；「采用为依赖」拆「采用二进制 / 采用 SDK」。
5. **供应端客户端**（I-11）：技术基线写三级顺序——官方 SDK > 从接口描述生成 > 手写；六家逐个落对象文件。
6. **hctl-tool `wait` 命令**（M-30）：闭集事实词表（某提交的 CI 状态、PR 是否合并、引用是否推进、路径存在且摘要匹配、进程退出）、带截止、一次调用一个答案（成立 / 不成立 / 读不到 / 超时）、返回结构化事实记录可作 `toolbox_readback` 级证据；控制面判准入与执行体 wait 共用同一段读事实代码。先落跨平台文件监听与 PR 状态库的调研。
7. **harness 钩子调研**（I-13）：`docs/research/harness-hooks-<日期>.md`，一 harness 一行：有无钩子、事件名、配置位置、能否硬拒绝、决策格式、能否按会话配置不污染 worktree、钉版本；覆盖 Claude Code、Codex、Gemini CLI、OpenCode、Cursor CLI、Kimi Code、Qwen Code、Copilot CLI；另列 ACP `session/request_permission` 作结构化模式等价物。与 Herdr 兼容要验证三点：环境变量与配置目录透传、钩子回调能找到 hctl-tool、钩子 stderr 不干扰 Herdr 的 agent 状态判定。

## 批 6 · 研究层追加与评审 Skill 抽取

1. `docs/research/methodology-landscape-20260824.md` 文末追加复核记录：第十二族改名扩容为「机械守卫与验收账本」并补四样本（M-29）、软件工厂亚种、两处横评校准（M-34）、「谁判完成」计数校准（M-36）。
2. 评审方法抽成可移植的 Skill（放 `src/agency/skills/`，与 `hctl2-shaping` 并列）：四轴清单（`02-checklists.md`）、机械清点（`src/build/docs/inventory_*.pl`）、对抗核验流程（新开上下文、维持 / 修正 / 推翻）、裁决包两档；目标是能拿到 abacistopia 用。第二轮的教训要写进去：裁决包表格只当索引，交付所有者的形式是逐条人话对话。
3. Foreman 四工位分类（分类器 → 分析员 → 实现者 → 评审员）作为将来生产 Participant 的角色分类参考，记入 `src/agency/README.md` 的备忘节。

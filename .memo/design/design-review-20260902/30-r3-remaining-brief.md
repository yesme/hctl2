# 第三轮剩余工作：三份任务书（Fable / Codex / Grok）

> 状态：三份任务书均已落——一 #153（v0.16.4；补充裁决 #154）、二 #156 / #159、三 #161 / #162；Grok 修正结论派生的豁免表收缩已落（v0.16.5）（2026-09-04）<br>
> 依据：`21-r2-rulings.md`（所有者裁决）与 `13-findings.md`（核验后的发现清单）。已合入：PR #148 写法统一、#149 约束含义、#150 愿景与架构改写，基线 v0.16.3。<br>
> 分工原则（所有者 2026-09-03）：只用三个 harness；写作与调研归 Fable，代码归 Codex + GPT，测试计划与对抗核验归 Grok。所有者只需把对应任务书交给对应 harness 说「开」，各自开 PR、过 CI、自己合。<br>
> 顺序：Fable 先（研究文件是 Codex 两项的前置），Codex 随后，Grok 在 Codex 每个 PR 合入后跟一个测试 PR。<br>
> 纪律：PR 描述按模板三节；新增脚本要在调研节论证；动第三方依赖前先有 `docs/research/` 对象文件；研究文件发布后正文不改、只在文末追加复核记录；「张力」写「冲突」；位置引用写「文件 §节名」，不写行号。

## 任务书一 · Fable：写作与调研（一个 PR，或研究先行拆两个）

> 进度（2026-09-03）：一个 PR 落完。已完成：研究对象文件（钩子、六个库、六家 SDK）、participant.md 的 Herdr 收敛与规则分组、借用等级用语与偏好顺序、技术基线、方法论地图追加、评审 Skill、Foreman 备忘。**I-03 其余 112 句的判断**：逐句核查后，绝大多数是启发式误报——机制在邻句或在它所属的对象定义里（如「Execution Spec 必须冻结已声明项」的机制就是 Execution Spec 冻结本身）；逐句加机制词会把约束层写成叠句，违反「警惕零碎纪律」与「写正面陈述」两条原则，因此**不做全量加词**，只在第二批已把真正缺机制的几处（外部事实前置、证据通道、校验等级、回源指针）补成机制。所有者已同意（2026-09-04，记入 `21-r2-rulings.md`）。

1. **研究对象文件**（Codex 的前置，先做）：
   - `docs/research/harness-hooks-<日期>.md`：一 harness 一行——有无钩子、事件名、配置位置（用户级 / 项目级 / 环境变量）、能否在执行前硬拒绝、决策格式、能否按会话配置而不污染 worktree、钉版本；覆盖 Claude Code、Codex、Gemini CLI、OpenCode、Cursor CLI、Kimi Code、Qwen Code、Copilot CLI；另列 ACP `session/request_permission` 作结构化接入模式的等价物。与 Herdr 兼容要验证三点：环境变量与配置目录透传、钩子回调能找到 hctl-tool、钩子 stderr 不干扰 Herdr 的 agent 状态判定。
   - wait 命令所需的跨平台文件监听与 PR / CI 状态读取库；`serde_jcs`、`fd-lock`、SQLite Online Backup、`keyring`、FTS5 五个对象文件；Process Compose 对象文件补「发行侧用法」一段（含 Windows amd64 制品）。
   - 六家供应端客户端 SDK 逐个对象文件（官方 SDK > 接口描述生成 > 手写，I-11）。
2. **写法遗留**：`participant.md` 的 Herdr 提及收敛到「Agency 与执行体」一节两三处（A-23）、二十条关键规则按主题分组（A-24）；约束层 112 句「必须 / 不得 / 只能」无机制词的条目加机制词或降为说明（清单在 `10-inventory.md` §强制手段）；`delivery.md` §选型判据 与研究层 README 写借用偏好顺序、「采用为依赖」拆两级（I-07）；技术基线写五处库与供应端客户端三级顺序（I-05、I-11）。
3. **研究层追加**：`methodology-landscape-20260824.md` 文末复核记录——第十二族改名扩容为「机械守卫与验收账本」并补四样本（M-29）、软件工厂亚种、两处横评校准（M-34）、「谁判完成」计数校准（M-36）。
4. **评审 Skill**：把四轴评审方法抽成 `src/agency/skills/hctl2-design-review`（与 `hctl2-shaping` 并列）：四轴清单（`02-checklists.md`）、机械清点（`src/build/docs/inventory_*.pl`）、对抗核验流程（新开上下文、维持 / 修正 / 推翻）、裁决包两档；要能拿到 abacistopia 用。写进去第二轮的教训：裁决包表格只当索引，交付所有者的形式是逐条人话对话，所有者说的每句话都要给反馈。
5. Foreman 四工位分类（分类器 → 分析员 → 实现者 → 评审员）记入 `src/agency/README.md` 备忘节，作将来生产 Participant 的角色分类参考。

## 任务书二 · Codex + GPT：代码（两个 PR：文档工具链；运行时与部件）

> 进度（2026-09-04）：两个 PR 落完（甲 #156、乙 #159）。与任务书字面不同的三处：JCS 用 `serde_json_canonicalizer`、文件锁用标准库 `File::lock`，均按研究文件首选，不算偏离；第 5 项供应端客户端只接了 `wait` 用到的 GitHub CLI，其余五家延到 P2 有 adapter 调用方时再接——缩范围，待所有者点头。SQLite bundled 带出 #157（Buck build-script 需 Python 3.12）、#158（bundled 与预编译替代复核）两个 issue。汇总见状态板「延后与遗留」。

**PR 甲 · 文档工具链**（不依赖研究文件，可先做）

1. 四个检查器进文档 CI 门（`src/build/docs/`，与现有 `check_links.pl`、`check_dead_names.sh` 同款体例，各带 allowlist，接进 `BUCK` 的 docs profile）：
   - 路由表越层词：按 `docs/design/spec/README.md` §核心产品词 的「可见性」列，`vision.md` 出现「治理内部」词即报；架构层文件出现约束层词汇（§词汇索引 里除八个高频词外的名字）即报。
   - 产品名越层：设计层正文出现 Tuwunel、Vikunja、Dagu、Herdr、Cinny 等实现名即报；允许 `delivery.md`、`contract-tests.md`、研究层；`participant.md` 在 Fable 收敛 Herdr 前先整文件豁免。
   - 驼峰自造名：保留 ChangeSet、ReviewSubjectRef 与代码体内的标识符，其余 CamelCase 即报。
   - 首现中文对照：设计层每份文档里核心产品词与八个高频词首次出现须带中文对照。
   - 「需要」在约束层只许出现在「需要关注」（检查器）；「同层同表中英夹杂」做 `report_*` 窄版报告，不拦 PR。
2. 词汇表「语义名 ↔ 标识符」对照表：脚本从约束层的代码体字段生成到 `references/glossary.md` 末尾一节，生成物入库并有比对测试；枚举值写法统一为「中文含义（`代码体`）」（A-31 + A-38）。
3. PR 门：`.github/workflows/pr-contract.yml` 自建信号加「已有脚本单次 PR 净增超过 100 行也算自建」（I-17）。

**PR 乙 · 运行时与部件**（等 Fable 的研究文件合入后做 4、5 两项）

1. **Process Compose 接管服务生命周期**（I-10）：限时验证 → 一份 `process-compose.yaml` 接管 tuwunel、cinny、vikunja、dagu、herdr → 按服务拆配置模板（每服务一文件：配置、密钥、端口、就绪条件）→ 删 `common/runtime.sh` 的通用监督器半边与按动词拆的 `start.sh` / `stop.sh` / `status.sh`；DotSlash 清单补 Windows amd64。目标结构已定，不留旧 shell 与新机制并存。
2. **Reindeer 换官方二进制**（I-09）：`src/build/tools/reindeer` 改 DotSlash 清单，删源码编译启动器。
3. **五处通用机制用现成库**（I-05）：`serde_jcs`（RFC 8785，契约测试钉官方向量）、`fd-lock`、SQLite Online Backup API、`keyring`、FTS5；outbox / 租约 / 代次维持自研。
4. **hctl-tool `wait` 命令**（M-30）：闭集事实词表（某提交的 CI 状态、PR 是否合并、引用是否推进、路径存在且摘要匹配、进程退出）、带截止、一次调用一个答案（成立 / 不成立 / 读不到 / 超时）、返回结构化事实记录可作 `toolbox_readback` 级证据；控制面判准入与执行体 wait 共用同一段读事实代码。
5. **供应端客户端**按研究文件结论接入（官方 SDK 优先）。

## 任务书三 · Grok：测试计划与对抗核验（每个 Codex PR 合入后一个测试 PR）

> 进度（2026-09-04）：两份测试 PR 落完（#161 文档工具链、#162 运行时与部件）。结论：#159 维持（范围脚注：控制面尚未进树，「模型转述不能满足前置」目前只在 `wait` 一侧堵住，P2 写准入时必须继续拒绝转述）；#156 修正——豁免表 46 条全为规则豁免，收缩归 Fable，已在 v0.16.5 批写回正文并清零，夹具改为空豁免表必须通过。

1. 文档工具链：为四个检查器造误报与漏报样例（中文子串边界、代码体内的名字、豁免表、首现对照在表格里等），补进各检查器的测试；核对 PR 门阈值对重命名文件、移动文件的行为。豁免表要单独核：`src/build/docs/first_use_terms.allowlist` 现有 46 条，其余五个检查器 0–21 条——逐条看是正文确实不该带对照，还是把规则豁免掉了。
2. Process Compose：三平台（Linux x86_64、macOS arm64、macOS x86_64）拉起、就绪、重启、关停的测试计划与失败用例；确认旧 shell 监督器已删干净、无并存路径。
3. wait 命令：四种返回各造用例，含超时边界、读不到与不成立的区分、并发调用；对抗核验「模型转述不能满足前置」在代码层是否真的堵住。
4. 五处库：JCS 官方测试向量全过；备份快照在写入进行中的一致性；文件锁在进程被杀后的释放。
5. 对每个 PR 出一份「维持 / 修正 / 推翻」结论，放进对应 PR 的评论，不另开文档。

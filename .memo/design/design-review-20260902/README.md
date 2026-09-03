# 设计文档四轴评审 · 状态板

> 状态：第三轮已收口（2026-09-04，基线 v0.16.5）——写法（#148）、约束（#149）、愿景与架构（#150）、Fable 写作与调研（#153）、Codex 文档工具链（#156）与运行时及部件（#159）、Grok 测试与对抗核验（#161、#162）已合入 main；Grok 对 #156 的修正结论（首现对照豁免表 46 条全为规则豁免）由 Fable 写回正文并清零（v0.16.5）。第三轮补充裁决四条记入 `21-r2-rulings.md`（GPL 生成物、钥匙串回退、五家 SDK 延到 P2、I-03 不做全量加词，均已所有者同意）。跟踪 issue #157 / #158 已由 GLM 关闭（#164、#165）；Grok 对 #165 的核验见阶段 3.8。其余见表下「延后与遗留」<br>
> 基线：main @ `99b0bfb`（草案 v0.16.0）<br>
> 计划：[`01-plan.md`](./01-plan.md)；原则：[`../../notes/review-four-axes-20260902.md`](../../notes/review-four-axes-20260902.md)；状态与方法：[`00-brief.md`](./00-brief.md)

| 阶段 | 状态 | 产出 | 最后更新 |
| --- | --- | --- | --- |
| 1.1 定标与脚手架 | 完成 | `02-checklists.md`、`src/build/docs/inventory_*.pl`、`root//build/docs:inventory` | 见分支 |
| 1.2 机械清点 | 完成 | `10-inventory.md`（四张表） | 见分支 |
| 1.3 业界调研 | 完成 | `docs/research/methodology-boundaries-20260902.md`、`methodology-sweep-2026h2-20260902.md`、`component-matrix-20260902.md` | 见分支 |
| 1.4 通读与发现 | 完成（24 个文件，调研证据已回填） | `11-findings-draft.md` | 见分支 |
| 1.5 对抗核验 | 完成：四轴四份，维持 44 / 修正 64 / 推翻 5 / 补 27 | `12-adversarial-{A,I,W,M}.md`、`13-findings.md` | 见分支 |
| 1.6 裁决包 | 完成，随第一轮 PR 合入 | `20-verdict-packet.md` | 见分支 |
| 2 拍板 | 完成：30 条全部裁决，新增 A-57；过程见 `21-r2-rulings.md` | `20-verdict-packet.md`、`21-r2-rulings.md` | 见分支 |
| 3.1 写法统一（Room、当前范围、词表分类、人的写法） | 完成 | PR #148（v0.16.1） | 见 main |
| 3.2 约束含义（Binding 族重构、六项机制、CT 用例） | 完成 | PR #149（v0.16.2） | 见 main |
| 3.3 愿景与架构层改写（Fable） | 完成 | PR #150（v0.16.3） | 见 main |
| 3.4 写作与调研（Fable，任务书一：研究对象文件、写法遗留、研究层追加、评审 Skill、Foreman 备忘） | 完成 | PR #153（v0.16.4）；补充裁决 GPL 生成物入库、钥匙串回退 PR #154 | 见 main |
| 3.5 代码：文档工具链 PR 甲、运行时与部件 PR 乙（Codex + GPT，任务书二） | 完成；延后与遗留见表下 | PR #156、#159 | 2026-09-04 |
| 3.6 测试计划与对抗核验（Grok，任务书三） | 完成（文档工具链夹具 + Process Compose / wait / 五处库；结论在 #156 评论「修正」、#159 评论「维持」） | PR #161、#162 | 2026-09-04 |
| 3.7 首现对照豁免表清零（Fable，承接 #156 的修正结论） | 完成 | v0.16.5：46 处对照写回九份设计层文档、词汇表 Execution Spec 对齐为「执行规格」、夹具改为空豁免表必须通过且豁免表保持只有注释 | 2026-09-04 |
| 3.8 #165 构建改动核验（Grok，任务书三追加段：钉定解释器反例、供应链锁、缓存代价） | 完成（结论在 #165 评论「修正」） | `30-r3-remaining-brief.md` §任务书三 追加 | 2026-09-04 |

**延后与遗留**（2026-09-04，不阻塞第三轮收口，P2 开工前先看这里）：

- 任务书二第 5 项「供应端客户端」只接入了 `wait` 实际调用的 GitHub CLI；其余五家（Dagu、Herdr、Linear、Matrix、Vikunja）延到 P2 出现首个 adapter 调用方时按 `docs/research/sdk/` 各文件结论接入。Codex 的理由是不加没有调用方的 SDK；这是对任务书字面的缩范围，所有者已同意（2026-09-04，记入 `21-r2-rulings.md` 第三轮补充裁决）。
- PR #159 带出两个跟踪 issue：#157 Buck2 的 Cargo build-script 需钉 Python 3.12（`libsqlite3-sys` 暂用手工 `cxx_library` 绕过）；#158 复核 SQLite bundled 与预编译库替代方案。→ 已由 GLM 关闭：#164 裁定维持 bundled 为终态（`docs/research/libs/sqlite-online-backup.md` 复核记录，否决预编译、系统库与换 RocksDB）；#165 钉定 python-build-standalone 3.12.14、启动器注入解释器与编译器绝对路径、`libsqlite3-sys` 翻回上游 build script（三平台 CI 绿，对象文件 `docs/research/build-tools/python-build-standalone.md`）。
- buck2-prelude 上游 issue 两个（GLM 留）已报（2026-09-04，Fable 以所有者身份提交）：① `from_any_dir.py` 用 3.12 的 `walk_up` 但未声明最低 Python 版本 → facebook/buck2#1490；② `os.execl` 不搜 PATH，与 `path_clang_tools` 裸名编译器组合必然失败 → facebook/buck2#1491。链接与建议修法已回填 `docs/research/build-tools/python-build-standalone.md` §复核记录。上游修复前本仓库以 config 注入绝对路径为既定机制；上游合入后再评估撤掉注入。
- 五处库里两处与任务书字面不同、与研究文件结论一致：JCS 用 `serde_json_canonicalizer`（`serde-jcs.md` 首选），文件锁用标准库 `File::lock`（`fd-lock.md` 首选，fd-lock 作者已提弃用）。不是偏离，记在这里免得复核时再查一遍。
- Grok 豁免表核验（任务书三）：`first_use_terms.allowlist` 46 条全部是规则豁免，不是「正文不该带对照」；其余五个检查器空表或专有名例外成立。结论写在 #156 评论，收缩豁免表是写作改动，不在本测试 PR。→ 已由 Fable 在 v0.16.5 批写回正文并清零（阶段 3.7）。

接手的会话：先读三个链接，再看这张表，从第一个「待开始」或「进行中」的阶段接着做；每完成一个阶段改这张表并提交。

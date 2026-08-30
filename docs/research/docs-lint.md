# 文档机械检查：选型短记（docs lint）

> 类别：⑥ 机械后端与基础设施 · 证据编号：E-TOOL-DOCS-LINT<br>
> 状态：证据审计 · 2026-08-31<br>
> 总览、引用准入与五种复用决策用语见 [docs/research/README.md](./README.md)。

## 要解决的问题

全库文档大修（S1）需要五项机械检查常驻 CI：相对链接与 `#锚点` 可达（含中文标题的 GitHub anchor 规则）、版本戳一致、死名扫描、禁令密度报告（只报告）、`.memo/review` 目录与文件头基线一致。其中只有"链接与锚点"存在现成工具，其余四项是本仓库的自有语义（版本戳格式、退休词表、禁令词表、memo 目录约定），任何通用工具都不覆盖。

## 候选与比较

| 候选 | 覆盖 | 不采用/采用理由 |
| --- | --- | --- |
| [lychee](https://github.com/lycheeverse/lychee)（Rust，MIT/Apache-2.0） | 链接与锚点：有 `--offline` 与 `--include-fragments`，可查本地 Markdown 锚点 | 最强候选。不采用：① 本库标题以中文为主，锚点正确性取决于与 GitHub slugger 逐字节一致的 slug 规则（CJK 保留、`·`/`、` 等标点剥除、重复标题 `-1` 后缀），lychee 的 Markdown 锚点实现与本库实际链接形态（含 `<a id>` 显式锚点）的兼容性需要逐案验证，验证成本接近自写；② 五项检查中四项本无现成覆盖，引入第二个工具族只为剩余一项；③ 其默认姿态是网络检查，本库 gate 要的是离线、确定性 |
| [markdown-link-check](https://github.com/tcort/markdown-link-check)（Node，MIT） | 链接为主 | 不采用：锚点支持弱是社区已知短板；Node 工具链与本仓库工具零交集，shellcheck/CI 惯例全在 shell |
| markdownlint / vale 等 | 风格、可读性 | 与本次五项检查无关，不评 |
| 自写脚本（bash + perl） | 五项全部 | **采用**。约几百行、零新依赖、GitHub slug 语义逐字节自控；perl 在 CI 与本机均为系统自带（PR contract 检查已有 perl 先例，见 `.github/workflows/code.yml`） |

## 决定与边界

- 五项检查全部自写为 `src/build/docs/` 下的脚本，每项一个 Buck2 `sh_test` 目标，接入 CI gate；禁令密度为报告制（不设卡）。
- 若链接检查器日后被证明能力不足，回退路径是 lychee 经仓库既有的 DotSlash 固定工具机制（`src/build/tools/`）接入——不维护自研构建链，只消费官方制品。
- 明确不做：不检查外部 URL（gate 必须离线、确定）；不做 Markdown 风格 lint（不是本次目标）；不把报告制项升级为红绿卡。

## 实现要点（供审阅）

- cell 边界：Buck2 项目根在 `src/`，cell 路径不接受 `..`（已实证：`buck2 audit cell` 报 "expected a normalized path"），测试无法声明 `../docs` 为资源。因此 `materialize_repo_tree.sh` 把仓库根的受检文件复制进 `src/build/docs/repo_tree/`（已 gitignore），Buck2 测试只消费该副本——复制是显式步骤，缓存正确性由资源声明保证。
- 锚点 slug 按 GitHub 规则实现：小写化、剥除 `\p{L}\p{N}_-` 与空格外的字符、空格转 `-`、重复标题加 `-1` 递增后缀；`<a id="...">` 与 `<a name="...">` 同时计入。
- 死名扫描的退休词表初版由 `docs/design/spec/README.md` 五张归并/清扫表与 decision-history §27–§29 人工导出（表内含驼峰模式类条目，无法纯机械导出），落为 `dead_names.txt` 并逐行注明出处。
- 链接检查的范围是产品文档（根 README/AGENTS/CONSTRAINTS/CLAUDE + `docs/**`）；`.memo/**` 是时间点过程记录（review 随基线作废、log 冻结），其链接合法地随时间失效，不入卡。指向 `src/**` 的链接按 `src.manifest`（tracked 文件清单）核验存在性，不查锚点。

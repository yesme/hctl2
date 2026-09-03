# SQLite FTS5 · 全文索引

> 状态：调研 · 日期：2026-09-03<br>
> 类别：⑥ 机械后端与基础设施 · 证据编号：E-LIB-SQLITE-FTS5<br>
> 对象：SQLite FTS5，随 [`rusqlite` 0.40.2](https://crates.io/crates/rusqlite/0.40.2) 的 `bundled` feature 编译进来（SQLite 3.53.2，编译开关 `-DSQLITE_ENABLE_FTS5`）；中文分词候选 [`jieba-rs` 0.10.3](https://crates.io/crates/jieba-rs/0.10.3)（2026-07-19）、[wangfenjin/simple v0.7.1](https://github.com/wangfenjin/simple/releases/tag/v0.7.1)（2026-02-23）<br>
> 许可证：SQLite 公有领域；rusqlite MIT；jieba-rs MIT；simple MIT 或 GPL-3.0-or-later 双许可（其 Rust 绑定 libsimple 0.9.0 MIT）；sqlite-jieba-tokenizer 0.6.0 MIT OR Apache-2.0

## 定位

我们用它做 Context 组装的全文索引：从 Room 讨论、Memo、Artifact 里萃取相关内容（[Project 约束 §Context、Memo 与 Artifact](../../design/spec/project.md#contextmemo-与-artifact)；[交付文档 §技术基线](../../design/delivery.md#技术基线)选了 SQLite + FTS5）。约束层把索引定为**可重建的派生投影，不进权威账本**。它替代的手写代码是自建倒排索引或 `LIKE '%…%'` 全表扫描。

## 上游能力

- **随 bundled SQLite 免费得到**：libsqlite3-sys 的 bundled 构建同时打开 FTS3/FTS5、JSON1、RTREE、`SQLITE_THREADSAFE=1`；rusqlite `bundled` + `backup` 的依赖树三平台各 9 个 crate，FTS5 不再增加任何依赖。rusqlite 0.40.2（2026-08-08）自带 SQLite 3.53.2。
- **FTS5 能力**：虚表 + `MATCH` 查询语法（AND/OR/NOT、短语、前缀、NEAR、列过滤）、`bm25()` 排名（值越小越相关）、`highlight()` / `snippet()`、`detail=full|column|none` 控制索引粒度与体积、外部内容表（`content='源表'`）与无内容表（`content=''`）、`'rebuild'` / `'optimize'` / `'integrity-check'` 三个维护命令。
- **内建分词器**：`unicode61`（默认；按 Unicode 6.1 类别分词，默认 token 类别 `L* N* Co`，可加 `remove_diacritics 0/1/2`、`tokenchars`、`separators`、`categories`）、`ascii`、`porter`（包装其他分词器做英文词干）、`trigram`（SQLite 3.34.0，2020-12-01 加入：每连续三个字符一个 token，做任意子串匹配，支持 `case_sensitive 0/1` 与 `remove_diacritics`，能把 `LIKE` / `GLOB` 变成索引查询；少于 3 个 Unicode 字符的查询不返回任何行）。
- **自定义分词器**：经 `fts5_api.xCreateTokenizer` 注册，实现 `xCreate / xDelete / xTokenize`，**每个连接都要注册一次**。rusqlite 没有安全封装，要写一段 unsafe FFI（社区有 gist），或者以 loadable extension 形式加载（rusqlite `load_extension` feature，bundled 已开 `SQLITE_ENABLE_LOAD_EXTENSION`）。

## 中文分词的问题与可选方案

问题本身：`unicode61` 只认「分隔符」与「token 字符」两类，CJK 表意文字属于字母类别 `Lo`，因此一段没有空格和标点的中文被当成**一个** token。索引里存的是「这是一段中文说明」这种整串，查「中文」命中不了（除非用前缀 `这是*`）。这是 sqlite-users 邮件列表上多年老问题，官方答复是用 ICU 分词器或自定义分词器，FTS5 未内建 CJK 切分。

| 方案 | 机制 | 新增依赖 | 中文效果 | 代价 |
| --- | --- | --- | --- | --- |
| `trigram` 分词器 | 内建，任意 3 字符子串 | 0 | 任意 ≥3 字的子串都能命中；`LIKE '%…%'` 走索引 | 双字词（中文最常见的词长）查不到——不足 3 字直接无结果；索引比词级大 |
| 应用层预分词 + `unicode61` | 写入前用 jieba-rs 切词，以空格拼接后存入 FTS 列；查询侧同样切词 | jieba-rs 0.10.3（纯 Rust，MIT，359 万下载） | 词级检索，双字词正常 | `highlight/snippet` 的偏移对应预分词文本而非原文；分词器与词典版本要进索引元数据，升级即 rebuild |
| simple 扩展（C++，内含 cppjieba） | loadable extension，官方预编译动态库覆盖 osx arm64/x64、linux x64/arm、windows x64/arm64/x86 | 一个 C++ 动态库；Rust 绑定 libsimple 0.9.0 | 字级 + 拼音 + jieba 短语，`simple_query()` 帮拼查询 | 非 Rust 构建链、随包动态库、SBOM 多一项；许可 MIT/GPL 双选 MIT 无碍 |
| sqlite-jieba-tokenizer 0.6.0 | Rust 写的 loadable extension | 该 crate | jieba 词级 | 累计下载 177，太新太小 |
| 自定义分词器经 `fts5_api` | unsafe FFI 注册，内部调 jieba-rs | jieba-rs | 词级且偏移对应原文 | 一段 unsafe 胶水，每连接注册 |
| ICU 分词器 | `SQLITE_ENABLE_ICU` | ICU 库（数十 MB） | 好 | 体积与构建链 |
| tantivy 0.26.1 + cang-jie 0.20.0 | 独立索引目录 | 大 | 好 | 第二份存储与进程内索引生命周期（[部件矩阵](../component-matrix-20260902.md#d--约束层里靠第一方代码实现的通用机制)已判为超集） |

## 边界与取舍

「索引可重建、不进权威账本」怎么落：

1. **物理分离**：索引放独立数据库文件（例如与账本同目录的 `index.sqlite`），不进[备份集](../../design/spec/system.md#备份与恢复)；账本快照完成后索引可以整个删掉重建。若把 FTS 表放进账本库，它就进了备份、进了 schema 版本、进了单写者事务——与约束冲突。代价是索引对账本的更新是**最终一致**：投影任务在账本提交后异步喂入，索引要能报告「截止到账本序号 N」。
2. **不持有第二份权威文本**：用外部内容表 `content='本地物化文本表'`，物化文本表本身也是派生物；`INSERT INTO ft(ft) VALUES('rebuild')` 即可从物化表全量重建。无内容表 `content=''` 更省空间，但没有源表就无法 `rebuild`，只能重新灌入。
3. **索引元数据**：一张小表记 FTS5 schema 版本、分词方案（`unicode61` / `trigram` / 预分词）、jieba 词典摘要、账本水位；启动时任一不匹配就触发重建；`doctor` 跑 `'integrity-check'`。
4. **体积**：`detail=column` 或 `detail=none` 可显著缩小，但会失去 NEAR 与短语查询——Context 萃取以 BM25 排名为主，可先用 `detail=column`。
5. **trigram 的 3 字下限**：如果同时要「代码标识符 / 路径子串」检索，可以再建一张 `tokenize='trigram'` 的 FTS 表，两张表各管一类查询；查询短于 3 字时退回 `LIKE` 扫描并明示。
6. **上游未查到**：`trigram` 的 `remove_diacritics` 选项在哪个版本加入——3.45.0 发布记录只提 `tokendata`，3.53.2 的文档已有该选项；不影响选型。

## 决定建议

**采用 SDK**：FTS5 随 bundled SQLite 一起来，零新增依赖。中文分词先走「jieba-rs 应用层预分词 + `unicode61`」——纯 Rust、无 FFI、无随包动态库，词典版本进索引元数据，升级即重建；`trigram` 表作为子串检索的补充按需加。不引入 simple C++ 扩展（多一条非 Rust 构建链），不做 unsafe 的 `fts5_api` 注册。索引落独立文件，物理上不进账本备份集，最终一致并可整体重建。tantivy 仅参考行为。

## 证据

- SQLite 官方：[FTS5 文档（分词器、外部内容表、维护命令、bm25、detail）](https://sqlite.org/fts5.html)、[3.34.0 发布记录（trigram）](https://sqlite.org/releaselog/3_34_0.html)、[3.45.0 发布记录](https://sqlite.org/releaselog/3_45_0.html)
- CJK 与 unicode61：[sqlite-users 讨论「Why FTS5 Unicode61 tokenizer does not support CJK」](https://sqlite-users.sqlite.narkive.com/N5MOmskp/sqlite-why-sqlite-fts5-unicode61-tokenizer-does-not-support-cjk-chinese-japanese-krean)
- rusqlite：[crates.io 0.40.2](https://crates.io/crates/rusqlite/0.40.2)、[libsqlite3-sys `build.rs`（bundled 编译开关含 `SQLITE_ENABLE_FTS5`）](https://github.com/rusqlite/rusqlite/blob/master/libsqlite3-sys/build.rs)、[v0.40.2 自带 SQLite 3.53.2](https://github.com/rusqlite/rusqlite/blob/v0.40.2/libsqlite3-sys/sqlite3/sqlite3.h)、[rusqlite FTS5 自定义分词器 gist](https://gist.github.com/ColonelThirtyTwo/3dd1fe04e4cff0502fa70d12f3a6e72e)
- 分词候选：[jieba-rs 0.10.3](https://crates.io/crates/jieba-rs/0.10.3)、[wangfenjin/simple](https://github.com/wangfenjin/simple)（[v0.7.1 预编译制品](https://github.com/wangfenjin/simple/releases/tag/v0.7.1)、[LICENSE 双许可](https://github.com/wangfenjin/simple/blob/master/LICENSE)）、[libsimple 0.9.0](https://crates.io/crates/libsimple/0.9.0)、[sqlite-jieba-tokenizer 0.6.0](https://crates.io/crates/sqlite-jieba-tokenizer/0.6.0)、[cang-jie 0.20.0（tantivy 用）](https://crates.io/crates/cang-jie/0.20.0)
- 本库：[Project 约束 §Context、Memo 与 Artifact](../../design/spec/project.md#contextmemo-与-artifact)、[交付文档 §技术基线](../../design/delivery.md#技术基线)、[Context 生态四族](../context-landscape-20260824.md)、[部件矩阵表 D](../component-matrix-20260902.md#d--约束层里靠第一方代码实现的通用机制)

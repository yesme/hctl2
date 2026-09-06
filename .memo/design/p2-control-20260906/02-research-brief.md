# P2 前置研究：任务书（Codex 写，GLM 审）

> 状态：已拍板 · 所有者 2026-09-06：前置研究不由 Fable 写，交 Codex<br>
> 基线：main @ `1d001ad`（草案 v0.17.0）<br>
> 去向：`docs/research/` 下六份对象文件或复核记录；不写代码、不改约束层

六份研究是 [`01-plan.md`](./01-plan.md) §八 的前置，按纪律三「新组件、新依赖先落 `docs/research/`，再写代码」。分三个 PR，都不自合：P2.1 组（0a、0b、0c）、P2.2 组（1a）、P2.3/P2.4 组（2a、2b）。GLM 主审，逐条「维持 / 修正 / 推翻」；Fable 只读每份文件的「决定建议」一节。

## 通用要求

- **格式照现有对象文件**：文件头三行「对象（钉 commit 或版本）/ 许可证 / 定位」，正文分「上游能力」「候选比较」（表格，逐候选钉版本、许可证、MSRV）「边界与取舍」「决定建议」（借用等级按 `docs/research/README.md` §复用决策用语 的六种之一）「证据」（链接）。范例：`docs/research/sdk/herdr.md`、`docs/research/libs/keyring.md`。
- **已有文件正文不改，只在文末追加「复核记录」**（1a、2b 是这种）。
- **四级顺序**（所有者 2026-09-06）：随包发布的官方命令行工具（以工具调用方式使用）> 官方 SDK > 从接口描述生成 > 手写。有官方命令行不等于调用面已满足，要逐个操作核对结构化输出、身份与条件写入。
- **已定的方向不重开**：Protobuf 作接口 schema（`01-plan.md` §五第 1 项）；`rusqlite_migration` 首选（§五第 2 项）；GitHub 走随包 `gh`（`sdk/github.md`）；Matrix 走 ruma、Vikunja 与 Dagu 走 progenitor、Herdr 走 typify（`sdk/README.md`）。研究是核对与钉版本，发现方向站不住才写「推翻」并说明。
- **工具链事实**：rustc 1.98.0（`src/rust-toolchain.toml`），Buck2 驱动，三平台 Linux x86_64 / macOS arm64 / macOS x86_64，随包二进制走 DotSlash 钉定（先例：`src/build/tools/` 下的 buck2、gh、Process Compose）。
- 索引同步：`docs/research/README.md` §条目索引、`sdk/README.md` 或 `libs/README.md` 加行。PR 描述三节按模板；本批不新增脚本、不改依赖，调研节直接引用所写文件。
- 文风：说人话，术语首现给一句解释；「张力」写「冲突」；位置引用写「文件 §节名」。

## 0a · `docs/research/libs/protobuf-rpc.md`

对象：Protobuf 作 `hctl2-control` 与客户端之间的接口 schema，及本地传输。要回答：

1. Rust 生成链：prost、prost-build、tonic、pbjson（ProtoJSON 的 serde 支持）各自最新版、许可证、MSRV 是否 ≤ 1.98；protoc 二进制随 DotSlash 钉定与纯 Rust 的 protox 二选一，判据是三平台可复现、不引入宿主依赖。
2. Buck2 接法：prelude 有没有可用的 proto 规则；没有则 genrule 调 protoc/protox 产出 `.rs`，再以 rust_library 包成独立 target——所有者要求 package 切细，`.proto` 编译出的 crate 自己一个 target，消费方只依赖它。
3. 本地传输：tonic 在 Unix 套接字上跑 gRPC（hyper 的 unix connector）与最小自定义帧（长度前缀 + 二进制 Protobuf）两条路各自的代价；Subscribe 要流式，断线重连要能带游标重同步。P2 的客户端只有同包发行的 Rust CLI，Windows 不在范围。
4. TypeScript 一侧（P3）：Protobuf-ES 与 Buf 的关系、从同一份 `.proto` 生成的路线，只核可行性不做。
5. 边界：进 Git 的领域正文继续用 RFC 8785 规范化 JSON 作事实源，`.proto` 只承载传输语义（`.memo/notes/control-api-schema-20260902.md`）。
决定建议要写成：schema 用 Protobuf（已定），传输选哪条、生成器选哪家、二进制怎么钉。

## 0b · `docs/research/libs/sqlite-migrations.md`

对象：账本表结构的版本化升级。前提：允许停机迁移（所有者 2026-09-06），启动时在单写者锁内、Online Backup 一致快照之后一个事务完成；只有事实源表（领域事件、幂等结果、outbox、租约与代次）需要 SQL 升级，投影表删掉重建。要回答：`rusqlite_migration` 最新版、许可证、MSRV、与 rusqlite 0.40（`src/Cargo.toml` 已钉）的兼容；与 `refinery`（rusqlite feature）的对比；`PRAGMA user_version` 与 `application_id` 的用法；SQLite 改表的十二步在事务内的注意点；失败回退到快照的流程。决定建议：首选哪个库、版本号放哪、备份在哪一步。

## 0c · `docs/research/runtime/process-compose.md`

对象：Process Compose 作 control 托管随包服务的调用面（版本以 `src/packaging/dependencies/lock.json` 钉定的为准）。要回答：命令行有哪些子命令能给结构化输出（进程列表、状态、就绪）；REST 接口（默认端口或 Unix 套接字）能不能按组件启停、重启、读健康与就绪探针；control 是把它作为子进程拉起并附着，还是连接一个已在跑的实例；日志怎么取；许可证与升级路径。对照现有 `hctl2-services`（`src/packaging/dependencies/hctl2-services`）已经用到的部分，指出 control 接管时哪些能直接复用。

## 1a · 复核记录：`docs/research/sdk/matrix.md` 与 `docs/research/sdk/vikunja.md`

按 P2.2 的实际调用面各追加一条复核记录。Matrix：Tuwunel 的 AppService 注册用文件还是管理房间命令、虚拟用户创建、建房与邀请、按事件 ID 读正文、房间加密状态回读（HCTL 房间不开端到端加密，绑定前要回读）、ruma 0.16 需要打开的 feature 精确清单、限流。Vikunja：「一个 Repo 一个 Board、Project 是分组、Task 是卡片」在 v2.5.0 的接口上怎么落（父任务还是 project 嵌套）、条件写入（版本或时间戳）、webhook 与轮询、API 令牌、progenitor 0.14 读它导出的 OpenAPI 3.0 文档是否真能生成编译通过的客户端（做一次生成实验，记录失败点）。

## 2a · `docs/research/harness-adapters.md`

对象：三家 harness 的无界面接入面，供 P2.3 的适配器骨架用；钩子部分已在 `docs/research/harness-hooks-20260903.md`，本文只补它没写的。逐家（Claude Code、Codex CLI、Gemini CLI）回答：无界面启动命令与参数（如 `claude -p --output-format stream-json`、`codex exec` 的 JSON 输出、`gemini -p` 的输出格式）；结构化事件流的形状与稳定性，能否归一成 `spec/participant.md` §运行时与观测 要求的七类（生命周期提示、工具调用、权限请求、文件变化、测试、用量、原始输出）；终局结果事件是否明确（进程正常退出但缺终局事件要按协议错误处理）；会话标识与转录文件位置（署名与恢复要用）；在 Herdr 窗格里以 `--env` 透传配置的可行性；权限模式怎么钉成「逐项询问」。决定建议：三家共用一个骨架，差别写成表。

## 2b · 复核记录：`docs/research/sdk/github.md`

按 P2.4 的写侧调用面追加一条复核记录：`gh` 的推分支（实际由 git push 完成，凭据经 `gh auth setup-git`）、`gh pr create` / `gh pr edit`（PR 描述三节由 control 写入）、`gh pr merge --merge --match-head-commit`（合并方式随仓库设置，见 `.memo/design/scm-module-20260906/02-attribution-gate.md`）、`gh pr comment` 写回、`gh api` 读评审线程是否解决、正式评审状态、分支保护条件与合并资格；确认丢失后按头分支名回读 PR 的关联键做法；限流；control 用用户令牌还是 GitHub App 安装令牌（各自能不能带身份）。每个操作标出能否给结构化输出、能否带身份、能否条件写入，核对不过的写明降级到 SDK 的那一项。

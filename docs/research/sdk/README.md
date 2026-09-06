# 供应端客户端层

本目录逐家回答「和这个外部系统说话，用随包的官方命令行工具、官方 SDK、从接口描述生成，还是手写」（Git 一条回答的是「宿主二进制还是内嵌库」）。顺序是四级：**随包发布的官方命令行工具（以工具调用方式使用）> 官方 SDK > 从接口描述生成 > 手写**——前三级到手写的排序是所有者 2026-09-03 的裁决（I-11），命令行工具置顶是所有者 2026-09-06 的裁决。有官方命令行不等于调用面已满足，要逐个操作核对结构化输出、身份与条件写入。每份文件钉住上游版本与许可证；生成物归我们，生成器只是构建期依赖。

| 文件 | 对象 | 决定 |
| --- | --- | --- |
| [`matrix.md`](./matrix.md) | Matrix 协议（Tuwunel homeserver） | 采用 SDK：ruma 类型库（client-api + appservice-api）配 reqwest / axum；matrix-sdk 仅参考——它已删 appservice crate，且 MSRV 高于我们的工具链 |
| [`vikunja.md`](./vikunja.md) | Vikunja 任务后端 | 从接口描述生成：progenitor 读二进制导出的 OpenAPI 3.0 文档；纠正部件矩阵里「先转 Swagger 2.0」的路线 |
| [`dagu.md`](./dagu.md) | Dagu 工作流引擎 | 从接口描述生成：progenitor 读仓库内 `api/v1/api.yaml`（只有 v1，没有 v2）；GPL 规范文件的生成物是否算衍生作品留所有者裁 |
| [`herdr.md`](./herdr.md) | Herdr 运行时 | 从接口描述生成：typify 读 `herdr api schema --json`（JSON Schema）；生成失败则退到移植其 schema 源码（Apache-2.0） |
| [`github.md`](./github.md) | GitHub REST / GraphQL | 采用二进制 gh（DotSlash 钉定，复用用户登录，`--json` 输出）：`hctl2-tool wait` 已用；control 一侧按四级顺序也先核 gh，octocrab 退为第二选择（2026-09-06 复核记录）。平台市场对照见 [`../scm-platforms.md`](../scm-platforms.md) |
| [`github.md` · P2.4 写侧复核](./github.md#2026-09-06--p24-写侧调用与恢复复核) | gh 的发布、合入与平台回读 | 采用二进制 2.99.0；高层命令缺 JSON 用 gh api，不新增 SDK；核准源头条件写、关联恢复、用户 / App 身份与当前仓库设置差异 |
| [`git.md`](./git.md) | Git 本地仓库（工具箱的现场引擎） | 采用二进制：宿主 git，下限 2.39，不随包；libgit2 / gitoxide 不进依赖树——同一现场只能有一个引擎，`merge-tree --write-tree` 与人手工合并同一实现；宿主版本不够时按 dugite-native 随包一份仍算采用二进制（所有者 2026-09-04 同意） |
| [`linear.md`](./linear.md) | Linear GraphQL | 从接口描述生成：graphql_client 读官方 TS SDK 仓库的 schema 快照；cynic 作备选；官方无 Rust SDK，社区实现已停 |
| [`matrix.md` · P2.2 复核](./matrix.md#2026-09-06--p22-appservice-实际调用面) | ruma / Tuwunel 注册与身份 | 采用 SDK：ruma 0.16.0；原生身份追加已核实。上表 MSRV 排除理由更正：matrix-sdk 1.93 低于本库 1.98，未选是因用途不匹配 |
| [`vikunja.md` · P2.2 复核](./vikunja.md#2026-09-06--p22-映射条件写入与生成实验) | 映射、条件写与生成失败 | Vikunja 2.5.0 维持；任务写入不检查 If-Match。暂缓采用 progenitor 0.14.0：真实规格生成失败，后备 7.25.0 尚待验证 |

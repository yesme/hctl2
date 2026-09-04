# 供应端客户端层

本目录逐家回答「和这个外部系统说话，用官方 SDK、从接口描述生成、还是手写」（Git 一条回答的是「宿主二进制还是内嵌库」）——三级顺序是所有者 2026-09-03 的裁决（I-11）：**官方 SDK > 从接口描述生成 > 手写**。每份文件钉住上游版本与许可证；生成物归我们，生成器只是构建期依赖。

| 文件 | 对象 | 决定 |
| --- | --- | --- |
| [`matrix.md`](./matrix.md) | Matrix 协议（Tuwunel homeserver） | 采用 SDK：ruma 类型库（client-api + appservice-api）配 reqwest / axum；matrix-sdk 仅参考——它已删 appservice crate，且 MSRV 高于我们的工具链 |
| [`vikunja.md`](./vikunja.md) | Vikunja 任务后端 | 从接口描述生成：progenitor 读二进制导出的 OpenAPI 3.0 文档；纠正部件矩阵里「先转 Swagger 2.0」的路线 |
| [`dagu.md`](./dagu.md) | Dagu 工作流引擎 | 从接口描述生成：progenitor 读仓库内 `api/v1/api.yaml`（只有 v1，没有 v2）；GPL 规范文件的生成物是否算衍生作品留所有者裁 |
| [`herdr.md`](./herdr.md) | Herdr 运行时 | 从接口描述生成：typify 读 `herdr api schema --json`（JSON Schema）；生成失败则退到移植其 schema 源码（Apache-2.0） |
| [`github.md`](./github.md) | GitHub REST / GraphQL | 按调用方拆：control 采用 SDK octocrab；`hctl-tool wait` 采用二进制 gh（DotSlash 钉定，复用用户登录，`--json` 输出） |
| [`git.md`](./git.md) | Git 本地仓库（工具箱的现场引擎） | 采用二进制：宿主 git，下限 2.39，不随包；libgit2 / gitoxide 不进依赖树——同一现场只能有一个引擎，`merge-tree --write-tree` 与人手工合并同一实现；宿主版本不够时按 dugite-native 随包一份仍算采用二进制（所有者 2026-09-04 同意） |
| [`linear.md`](./linear.md) | Linear GraphQL | 从接口描述生成：graphql_client 读官方 TS SDK 仓库的 schema 快照；cynic 作备选；官方无 Rust SDK，社区实现已停 |

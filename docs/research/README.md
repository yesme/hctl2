# docs/research · 证据审计

> 这里放钉着具体 commit / 版本的第三方审计：它们是产物（Task 交付的 Artifact），被规范永久引用，发布后不改内容、只追加复核记录。中间过程与备忘在 `.memo/`。

当前状态：原始审计已从 `.memo/` 迁入本目录；按产品归类的总览表、引用准入与五种复用决策用语仍在 [`docs/design/references/implementation-evidence.md`](../design/references/implementation-evidence.md)，下一步把该文件的逐产品条目拆成本目录内一个对象一个文件，总览留作本目录的 README。

| 文件 | 对象 | 钉定 |
| --- | --- | --- |
| [methodology-landscape-20260824.md](./methodology-landscape-20260824.md) | 方法论工具十二族与完成判定权横评 | 11 个仓库各钉 commit |
| [context-landscape-20260824.md](./context-landscape-20260824.md) | Context 处理生态四族与快省准横评 | 链接级，2026-08-24 |
| [grok-bot-reconstructed-audit-20260825.md](./grok-bot-reconstructed-audit-20260825.md) | Grok Bot 0.18 客户端重建源码 | `a9f633e` |
| [workbench-shell-reopen-20260826/](./workbench-shell-reopen-20260826/README.md) | Workbench 桌面壳：GPUI / Iced / Flutter / Web 壳，含 7 份附录 | main @ `12c9b44`；探针 2026-08-26 |

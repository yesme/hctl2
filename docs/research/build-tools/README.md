# 构建与交付工具

本目录收录只参与 HCTL2 开发、CI 和发行，不进入用户运行环境的单项工具审计。跨候选的文档检查选型仍见 [`../docs-lint.md`](../docs-lint.md)。

| 文件 | 对象 | 决定 |
| --- | --- | --- |
| [`install-dotslash.md`](./install-dotslash.md) | DotSlash 安装 Action | Linux CI 采用锁定提交的官方 Action；macOS CI 与开发机使用摘要锁定安装器 |
| [`buck2-change-detector.md`](./buck2-change-detector.md) | Buck2 Change Detector | 采用官方 `btd` 二进制；Buck2 原生导图；选择失败时全量回退 |
| [`jq.md`](./jq.md) | jq | 采用摘要锁定的官方单文件制品；只解析 BTD JSON Lines，不承担选择策略 |
| [`github-actions-incremental-validation.md`](./github-actions-incremental-validation.md) | GitHub Actions 增量重验证 | 复用旧 head 的成功 workflow 证据；快进增量验证，历史改写时全量回退 |

# 构建与交付工具

本目录收录只参与 HCTL2 开发、CI 和发行，不进入用户运行环境的单项工具审计。跨候选的文档检查选型仍见 [`../docs-lint.md`](../docs-lint.md)。

| 文件 | 对象 | 决定 |
| --- | --- | --- |
| [`buck2-change-detector.md`](./buck2-change-detector.md) | Buck2 Change Detector | 采用官方 `btd` 二进制；Buck2 原生导图；选择失败时全量回退 |

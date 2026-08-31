# jq 开发工具审计

> 固定版本：[`jq-1.8.2 / 34f7186`](https://github.com/jqlang/jq/tree/34f7186b86743a083a589741b6cea95293524108) · MIT 主许可及随附第三方许可

## 采用范围

Buck2 Change Detector 输出 JSON Lines（每行一个 JSON 对象），并把目标的验证类别放在 `labels` 数组中。HCTL2 要在开发机和任意 CI 上以同一种方式筛出带 `ci:fast` 或 `ci:platform` 标签的目标。JSON 结构不适合交给 `grep`、`sed` 或手写解析器，因此采用 jq；它只属于开发与 CI 工具，不进入 HCTL2 用户安装包。

不依赖宿主机预装版本。仓库通过 DotSlash 直接消费上游 [`jq-1.8.2` 发布页](https://github.com/jqlang/jq/releases/tag/jq-1.8.2)的三个官方单文件制品：Linux x86_64 为 2,267,912 字节，macOS x86_64 为 871,768 字节，macOS arm64 为 841,504 字节；`src/build/tools/jq-bin` 固定 GitHub 发布页给出的 SHA-256。

## 边界

- jq 只解析 BTD 已经产生的 JSON Lines，不决定受影响关系或 CI 策略。
- 目标影响范围仍由 BTD 计算，验证类别仍由 Buck 标签声明；选择失败时，检查入口仍回退到全量第一方目标。
- 版本、平台映射和摘要只有 DotSlash 清单一份，不再要求开发者用 Homebrew、APT 或另一份安装脚本准备 jq。

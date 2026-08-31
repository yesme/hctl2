# DotSlash 安装 Action 审计

> 固定实现：[`facebook/install-dotslash@0971fd2`](https://github.com/facebook/install-dotslash/tree/0971fd2071b83392cf02706266fee3877e3bcaf3) · MIT

## 采用范围

DotSlash 是 HCTL2 构建工具清单的引导程序：Buck2、BTD、jq、Actionlint、ShellCheck 和本机 REAPI cache server 都由 DotSlash 清单固定官方制品与 SHA-256。它只进入开发和 CI 环境，不进入用户运行包。

GitHub Actions 的 Linux 执行器采用 Meta 官方的 [`facebook/install-dotslash`](https://github.com/facebook/install-dotslash)。工作流把 Action 锁到 40 位提交，不跟随可移动的 `latest` 或 `v2` 引用。该提交从 DotSlash 官方最新 Release 选择当前平台的预编译包，并按 GitHub Release 元数据提供的 SHA-256 校验后加入 `PATH`。

## 版本边界

官方 Action 当前没有 `version` 输入，而是在每次执行时查询最新 DotSlash Release。只锁 Action 提交因此只能固定安装逻辑，不能单独保证工具版本。HCTL2 继续以 `src/build/tools/dotslash.env` 作为 DotSlash 版本事实源，并在 Action 后立即核对 `dotslash --version`；上游发布新版本后，CI 会先失败，直到仓库显式更新版本、三平台摘要和相关清单。这样成功的构建不会静默漂移到另一个 DotSlash 版本。

开发机不需要 GitHub Actions。`src/build/tools/install-dotslash` 直接下载同一固定版本的上游预编译包，按仓库记录的三平台 SHA-256 校验后安装到调用者指定目录。它是“尚无 DotSlash 时”的最小引导边界；引导完成后，其余构建工具全部走 DotSlash 清单，不再各自要求 APT、Homebrew 或临时下载脚本。

2026-08-31 在 GitHub 托管的 macOS 15 x86_64 和 arm64 执行器实测时，该 Action 的摘要校验会误判系统提供的 BSD `sha256sum` 支持 GNU 风格的 `--check`，随后在解包前失败。回退到较早的 `v2` 会丢失 Action 新增的摘要校验，不采用。修复进入上游前，macOS CI 与开发机共用仓库的摘要锁定安装器；Linux CI 继续使用已实测通过的官方 Action。这个差异只位于 DotSlash 引导层，后续工具清单和 Buck action graph 不分叉。

## 决定

- GitHub Actions 的 Linux 执行器使用锁定提交的官方安装 Action，并以仓库版本检查补足其“跟随最新 Release”的行为；macOS 暂用同一份本地引导安装器，等待上游修复 BSD `sha256sum` 兼容性。
- 开发机保留摘要锁定的本地引导安装器，不要求开发者安装 Node.js、Cargo 或包管理器才能取得 DotSlash。
- Action 提交、DotSlash 版本或三平台摘要的升级都作为构建供应链变更单独审阅；不把运行时解析到的版本写成第二份并行指纹。

# 依赖打包

这个目录是 HCTL2 外部依赖供应链的第一段实现。最终用户拿到的是一个可离线安装的归档，而不是一份联网引导方案。网络访问、校验和验证、归档解压与编译都由构建机负责。

## 构建流程

`versions.sh` 是四个选定依赖的锁定文件。`bootstrap.sh` 下载并验证官方发行物：Tuwunel、Vikunja 和 Dagu 使用已发布的 Linux 二进制；tmux 因上游不发布 Linux 二进制而从源码构建。缺少的 tmux 构建头文件与 bison 会按锁定版本下载并解压到构建缓存，全程不使用 sudo 安装。最终包还会记录这些 deb 版本，以及构建机使用的编译器、binutils、glibc、make 和 pkg-config 版本。

`build-package.sh` 随后组装一个带版本的 payload，其中包含：

- 四个可执行文件与 tmux 需要的非系统共享库；
- 纳入版本控制的 `hctl2-services` 生命周期入口及其启动、停止、状态与冒烟检查实现；
- 依赖版本、commit、发行物 digest、许可证与版权材料；
- 四个依赖的锁定源码归档，包括 GPL 许可的 Dagu 与 AGPL 许可的 Vikunja 对应源码；
- 离线安装器与完整的 payload 校验和清单。

在 Ubuntu x86_64 上构建：

```bash
src/packaging/dependencies/build-package.sh
```

在隔离的临时前缀中运行完整的构建、离线安装、幂等重装、启动、冒烟检查和停止流程：

```bash
src/packaging/dependencies/test-package.sh
```

被忽略的输出是 `src/dist/hctl2-0.0.0-linux-x86_64.tar.gz` 及其 SHA-256 文件。发行 CI 必须在支持范围内最旧的 glibc 基线上构建；本地构建能够证明打包流水线，但不能单独确定最终 Linux 兼容下限。

构建输入默认存放在 `${XDG_CACHE_HOME:-$HOME/.cache}/hctl2/dependencies`。设置绝对路径 `HCTL2_BUILD_CACHE` 可以复用或隔离另一份已验证构建缓存。

发行矩阵为每个受支持平台生成一个包，因此最终用户仍然只需下载一个发行物。当前切片实现并验证 Linux x86_64。macOS arm64 构建机将使用官方 Dagu 与 Vikunja 二进制，并从锁定源码构建 tmux 和 Tuwunel；只有原生 Tuwunel 构建在 macOS 上通过相同的已安装 payload 测试后，才能提供该目标。

## 用户流程

安装器不会下载或编译任何内容：

```bash
tar -xzf hctl2-0.0.0-linux-x86_64.tar.gz
cd hctl2-0.0.0-linux-x86_64
./install.sh
~/.local/bin/hctl2-services start
~/.local/bin/hctl2-services smoke
```

默认安装到 `$HOME/.local`；`--prefix` 可以选择另一个绝对前缀。持久数据、secret、日志、socket 和 PID 位于版本化安装目录之外的 `${XDG_STATE_HOME:-$HOME/.local/state}/hctl2`。开发与测试可以用绝对路径 `HCTL2_STATE_ROOT` 隔离状态。安装发行包不会自动启动进程。

完整命令说明见[HCTL2 使用说明](../../../docs/usage.md)。

## 运行策略

| 组件 | 版本 | 本地端点 |
| --- | --- | --- |
| Tuwunel | 1.9.0 | `http://127.0.0.1:6167` |
| Vikunja | 2.5.0 | `http://127.0.0.1:3456` |
| Dagu | 2.15.1 | `http://127.0.0.1:18080` |
| tmux | 3.7c | HCTL2 状态根目录下仅 owner 可访问的 socket |

所有 listener 都绑定 loopback。按照 HCTL Room 的要求，这份本地 Tuwunel 配置关闭 federation 与房间加密。Dagu 只在其 loopback listener 上关闭认证。Vikunja 会在首次启动时创建随机本地 secret。

这些脚本证明了打包与第一段启动接缝。公共 Rust CLI 完成后，`hctl2 start` 将调用同一生命周期层；不会再给用户增加第二套运维 API。

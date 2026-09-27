# llama.cpp：本机跑小模型的命令行

> 对象：[llama.cpp `b11222`](https://github.com/ggml-org/llama.cpp/releases/tag/b11222)（提交 `a97cce86a8addeb9f40cba7a261c94b1f0c576cb`，2026-09-27）<br>
> 许可证：MIT<br>
> 定位：用户打开小模型时，控制面调用的运行方式。缺省安装包不带它，也不带权重

## 上游能力

llama.cpp 把 GGUF 权重放在本机，用官方命令行做补全和嵌入。我们要的两个平台都有现成压缩包，不必自己编译：

| 包 | 字节数 | SHA-256 |
| --- | --- | --- |
| `llama-b11222-bin-macos-arm64.tar.gz` | 11756831 | `869b73f760042ac660e9453ff5e5cbb157725f2f8e01ec473006ffcf387a8410` |
| `llama-b11222-bin-ubuntu-x64.tar.gz` | 17403301 | `cfd2323f9ffec9657ca247140129be68df62d98f30a8448b20b1ca7a1796e1cb` |

同一天的标签 `v0.5.0` 只有一个 7 字节的 `nightly-tag.txt`，没有这两个二进制，不能拿来钉。命令行是发行包里的 `llama-cli`（生成）和 `llama-embedding`（向量）。也有 `llama-server` 提供本地 HTTP，控制面可以只把提示从套接字送进去，不必把推理链进自己的进程。

权重是另一个文件。运行方式钉住，不等于模型钉住。模型的版本和摘要见各模型自己的文件。

这次没有在 macOS arm64 或 Linux x86_64 上启动它，延迟、内存和 CPU 占用未实测。

## 候选比较

四级顺序是：随包的官方命令行，高于官方 SDK，高于从接口描述生成，高于手写。

| 运行方式 | 官方命令行 | 两个平台 | 能否钉版本 | 取舍 |
| --- | --- | --- | --- | --- |
| llama.cpp `b11222` | 有，发行包里的二进制 | macOS arm64 与 Ubuntu x64 都有官方包 | 包的 SHA-256 可以写进 DotSlash，和现有的 gh、Buck2 一样 | 选这个 |
| Ollama | 有官方命令行，但是常驻服务 | 有 | 模型在它自己的库里，摘要不直接等于上游 GGUF 文件 | 多一层守护进程，钉权重更绕 |
| MLX / mlx-lm | Apple 的 Python 工具 | 只有 Apple Silicon | 能钉 pip 包，不能服务 Linux | 不能做两个平台的同一条路径 |
| candle、mistral.rs | 库，不是模型厂商的命令行 | 能编 | 我们要自己包一层 | 手写或移植，排在命令行后面 |

Buck2 的接法如果以后要做：二进制用 DotSlash 指向上面两个压缩包和摘要，首次需要时再取，不进缺省安装包。GGUF 更大，单独下载，下载后对照摘要，不和安装包打在一起。这批不改 `src/`。

## 边界与取舍

命令行能生成短句，也能出向量，所以生成模型和嵌入模型可以共用一个运行方式。它不提供「只准选这些选项」的约束。是非题和五节选择题要在提示里写死选项，再由控制面检查输出是不是这些选项之一；对不上就当这次判断失败，退回机械规则，不能把胡写的标签当成结论。

数据留在本机。许可证允许再分发二进制。模型权重的许可证另算。

## 决定建议

采用二进制，但是只作为用户打开小模型之后的运行方式，不作为缺省安装的一部分。不采用 Ollama、MLX 或 Rust 推理库作为这条路径。

## 证据

- 发布页：<https://github.com/ggml-org/llama.cpp/releases/tag/b11222>
- 资产摘要用 GitHub API 的 `digest` 字段，2026-09-28 读取
- 四级顺序：`.memo/design/p2-control-20260906/02-research-brief.md` §通用要求
- DotSlash 先例：`docs/research/build-tools/install-dotslash.md`

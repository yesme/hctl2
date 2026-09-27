# 通义千问 2.5 指令模型

> 对象：Qwen2.5-1.5B-Instruct（仓库提交 `989aa7980e4c`）与 Qwen2.5-7B-Instruct（`a09a35458c70`）；GGUF 仓库分别为 `91cad51170dc` 与 `bb5d59e06d95`<br>
> 许可证：Apache-2.0，权重可再分发，Hugging Face 不设人工审批<br>
> 定位：中文短句和是非题要测的第一族生成模型。准确率未实测，缺省不下载

## 上游能力

这是能写短句的小型指令模型，不是只打分的分类器。1.5B 靠近任务书说的 1B 一档，7B 靠近 8B 一档。同一族里还有 Qwen3-4B-Instruct-2507（提交 `cdbee75f17c0`，Apache-2.0）和它的 GGUF 仓库 `bc640142c66e`，留作同一厂商的第二档对照，避免只测一个尺寸。

1.5B 的 Q4_K_M 文件名 `qwen2.5-1.5b-instruct-q4_k_m.gguf` 在 2026-09-28 能解析到对象存储。文件大小 1117320736 字节。Hugging Face 给出的内容标识是 `6a1a2eb6d15622bf3c96857206351ba97e1af16c30d7a74ee38970e434e9407e`。这是下载前要再核对的摘要，我们没有把文件下下来重算。

7B 用同样规则拼出的 `qwen2.5-7b-instruct-q4_k_m.gguf` 返回 404。仓库还在，文件名没有钉住，所以 7B 的文件摘要未实测。Qwen3-4B 的常规 Q4 文件名同样 404，摘要也未钉。

中文是这一族的强项，这是选它而不是 Llama 3.2 1B/3B 的原因。Llama 的小模型中文弱，不放进「中文强的两族」。第二族用 Gemma 3 4B（`google/gemma-3-4b-it`，提交 `093f9f388b31`）。它的许可证是 Gemma 条款，Hugging Face 上要人工点接受，不能像 Apache 模型那样直接放进首次下载。Gemma 因此只作对照候选，不作为建议打开的那一个。

## 候选比较

| 模型 | 大约规模 | 许可证 | 能否记下精确版本 | 本批状态 |
| --- | --- | --- | --- | --- |
| Qwen2.5-1.5B-Instruct Q4_K_M | 1.5B | Apache-2.0 | 仓库提交加上面的内容标识 | 标识来自下载响应，文件本身未下载 |
| Qwen2.5-7B-Instruct | 7B | Apache-2.0 | 仓库提交 `a09a35458c70`；GGUF 提交 `bb5d59e06d95` | 量化文件名未钉，摘要未实测 |
| Qwen3-4B-Instruct-2507 | 4B | Apache-2.0 | 提交 `cdbee75f17c0` | 量化文件摘要未钉 |
| Gemma 3 4B IT | 4B | Gemma，需人工接受 | 提交 `093f9f388b31` | 不建议做缺省下载 |

运行方式见 [llama-cpp.md](./llama-cpp.md)。没有在本机加载任何一个，延迟和内存未实测。也没有对样本做是非、分节或改写，忠实度未实测。

## 边界与取舍

生成模型能做 C 的短句改写，也能被提示去做是非和五节选择。选择做完必须由控制面检查标签，不能把自由文字直接写进五节。改写必须对照源消息查有没有多出来的说法、有没有把未定写成已定。来源指针仍由组装器按原消息挂上。

1.5B 更可能装进普通笔记本，7B 更可能把短句写顺。哪个漏选更少，要等标注后的同一套题，不能先猜。

## 决定建议

暂缓。它是标注到齐之后第一个要跑的生成模型族，不是现在就打开的功能。缺省不下载权重。

## 证据

- <https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct>
- <https://huggingface.co/Qwen/Qwen2.5-7B-Instruct>
- <https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF>
- <https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF>
- <https://huggingface.co/Qwen/Qwen3-4B-Instruct-2507>
- <https://huggingface.co/google/gemma-3-4b-it>
- 内容标识取自 2026-09-28 对 1.5B Q4_K_M 地址的响应头 `x-linked-etag`，文件未落盘

# Jev：只做选择题的托管判断模型

> 对象：[Jev `jev-1.13.0`](https://docs.typesafe.ai/models)（别名 `jev-latest`、`jev-preview` 在 2026-09-28 读文档时都指向它；TypeSafe AI，2026-09-15 起有限早期访问）<br>
> 许可证：专有软件，权重不公开。调用受其客户协议约束<br>
> 定位：Room 三件轻量活里，凡是「给定选项里选一个或答是非」的那一半，形状上对得上；它不写短句

## 上游能力

Jev 不生成文字。一次请求是一块 `state`（字符串、JSON 或文本数组）加上若干道题，题有三种：

| 题型 | 做什么 | 返回 |
| --- | --- | --- |
| Choice | 从给定选项里选一个 | 选中项、每项概率、置信度 |
| Score | 按排好的档打分 | 分数、各档概率、置信度 |
| Noul | 答一道是非 | 0 到 1 的概率；官方说明这种题没有单独的置信度字段 |

所有题对着同一块 state 并行打分。官方模型页（2026-09-28 读）写的限额是：整次请求 6.4 万 token，state 加最长的一道题 3.2 万 token；每秒 25 万 token、每分钟 1200 次请求，超了回 429，并写明限额会随时改。输入只收文字。

价格按官方模型页：输入每百万 token 0.042 美元，输出不计费。网上另有页面写成 0.25 到 0.42 美元，和官方页不一致，以官方页为准。没有公开的免费额度。响应里的 `model` 字段会回版本号；用别名时，后台换了模型，这个字段才会告诉你实际是哪一版。

英文是它训练的主语言。官方写明中文等其他语言能处理，但没有同样好，用之前要在自己的文本上测。客户请求不用于训练。零留存是企业档选项，缺省不是。服务在美国。直接 API 仍要早期访问；OpenRouter、Vercel AI Gateway（`typesafe-ai/jev`）等转售从 2026-09-16 前后开始出现，那是别人的入口，不是本机运行。

官方有 HTTP API，也有 Python 与 JavaScript SDK。没有可随包分发的官方命令行，也没有可校验的权重摘要。

## 候选比较

开源项目用它做上下文压缩，共同做法是只删不写。下面的数字都是那些项目自己写的，不是我们测的。

| 项目 | 怎么问 | 何时触发 | 不相关的怎么处理 | 他们自己写的代价 |
| --- | --- | --- | --- | --- |
| [jev-compactor](https://github.com/edwardyen724-g/jev-compactor) `11` 次提交、MIT | 每条消息一道选择题：为了当前目标，这条还要不要留在工作记忆里。丢掉的条件是 P(drop) ≥ 0.7 | 历史 token 超过 `maxTokens` 才调用；否则只跑本地正则 | 整条从数组里拿掉，留下的与输入同一对象。另有安全题（破坏性命令、外泄） | 自测 64 条消息的会话省 73%，约 350 ms、0.0004 美元；概率同题重复可差到 0.14 |
| [LiteLLM 的 typesafe 护栏](https://docs.litellm.ai/blog/typesafe-jev-compaction) | 对旧的工具结果问：回答用户最新问题还需要它吗 | 调用模型之前，`pre_call` | 低于 0.2 的换成一行英文占位，工具调用结构保留。Jev 不可用时缺省放行原文 | 文档没有给出他们自己的节省比例 |
| [jev-compact](https://github.com/HAR5HA-7663/jev-compact)（2026-09-24 仍在更新） | 「助手为了做完这件事，还会不会再用到这段输出？」 | 本地规则先删掉旧的只读工具输出；仍然超预算才问 Jev | 最低分的输出删到预算以内。用户和助手的话、报错、首尾消息不动。删掉的会留一行说明，让助手重跑工具而不是凭记忆编 | 文档写单次大约 100–400 ms |
| [pi-fast-jev-compaction](https://github.com/QuentinDanblon/pi-fast-jev-compaction)、[openclaw-jev-compaction](https://github.com/SqaaSSL/openclaw-jev-compaction)、[jevcomp](https://github.com/liqunqun07/jevcomp)、[Fast-Jev-Agents](https://github.com/satiricalguru/Fast-Jev-Agents) | 都从 [tamaratran/fast-jev-compaction](https://github.com/tamaratran/fast-jev-compaction) 来。每个工具调用两道是非：调用本身留不留，结果留不留 | 上下文占预算的比例，常见是 60%；或会话钩子 | 高于阈值的原文留下；只留调用、结果截断时加一行说明；低于阈值的整段去掉。重跑安全的才允许整段删除 | 阈值常见 0.5（结果有的用 0.25）。延迟和节省比例各仓库自报，不能横比 |

任务书起点里的 Register 链接（2026-09-23 那篇）这次没有打开。维基百科引用的是另一篇，2026-09-16 的 [TypeSafe AI debuts model for machines](https://www.theregister.com/ai-and-ml/2026/09/16/typesafe-ai-debuts-model-for-machines-that-plays-doom/5296711)。维基页与官方文档在发布日、三种题型、专有许可上一致。MindStudio 与 Raschka 的两篇介绍没有逐页核对，不把它们当依据。

分诊和安全门可借的是提问方式，不是阈值。TypeSafe 自己的 cookbook 用多道是非题加两档阈值（例如注入高于 0.70 就排除，相关低于 0.45 就排除）。jev-compactor 把破坏性命令的正则留在本地，不交给模型。A 的「该不该提示」和 B 的「这句还要不要」都是单道是非，可以用 Noul。C 的五节是 Choice。改写短句这三种题都做不到。

## 边界与取舍

能直接用在 B 上的是形状：相关的留原文，不相关的只留一行由组装器写的指针，指针指向原消息，不由模型生成。不能直接搬的是他们的英文工具结果、0.2 或 0.7 的阈值，以及「整段删除」。Room 里不相关的消息仍要能点回去，所以是换成一行指针，不是从记录里删掉。

中文准确率、误删率、本仓库样本上的延迟，都未实测。官方已经写明中文不如英文，所以不能用他们英文会话上的 73% 来估我们的漏选。

数据离开本机，发到 `api.typesafe.ai`。这要一个策略点，缺省不允许。见对照文件。没有权重摘要。版本只能记响应里的 `jev-1.13.0` 这类字符串，不能记 `jev-latest`。

官方延迟 70–500 毫秒、以及「比前沿模型快几百倍」都是他们自己的工作流，文档承认偏高。不采用这些倍数。

## 决定建议

仅参考行为。缺省不调用。等有访问权限、并且 Fable 的标注到齐之后，用本目录样本做中文是非和五节选择，再决定用户能不能打开它。即使用，也只做判断，不做短句改写。

## 证据

- 官方模型页：<https://docs.typesafe.ai/models>（2026-09-28）
- 官方法律索引：<https://docs.typesafe.ai/legal>。零留存要另谈，缺省不是
- 维基百科 [Jev (AI model)](https://en.wikipedia.org/wiki/Jev_(AI_model))，版本 `jev-1.13.0`，2026-09-15 发布
- jev-compactor 的接口笔记（他们 2026-09-18 对照官方文档和 SDK 0.6.0 写的，含自测延迟）：<https://github.com/edwardyen724-g/jev-compactor/blob/main/docs/JEV-API.md>
- 其余仓库只核对了存在、许可证字段和 README 里的提问句，没有跑他们的基准

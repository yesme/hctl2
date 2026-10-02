# Laya：本机可跑、要训练才好用的判断模型

> 类别：判断模型 · 证据编号：E-MODEL-LAYA<br>
> 状态：证据审计 · 钉定版本与许可见「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览与复用决策用语见 [docs/research/README.md](../README.md)；用在哪里见 [判断点备忘](../../../.memo/design/room-judgment-20261002.md)（§六 要求新判断器先落调研）。

<a id="e-model-laya"></a>
## E-MODEL-LAYA · Laya

### 审计基线

| 对象 | 版本 | 许可 |
| --- | --- | --- |
| [NandhaKishorM/laya `v0.3.23 / d8a2e59`](https://github.com/NandhaKishorM/laya/tree/d8a2e59781ca135169a36095056132e273cd9938)（2026-10-01；PyPI `laya`；Convai Innovations；仓库 2026-09-18 起约 1150 个提交） | 推理库、HTTP 服务、MCP、TypeScript SDK、ONNX 导出、微调脚本 | Apache-2.0 |
| 检查点 `convaiinnovations/laya`、`laya-multilingual`、`laya-typed-decisions` | 上游审核过的修订号写在 `laya/revisions.py`：`55cf4c4`、`e4e9ddf`、`1a793eb` | README 写 Apache-2.0；本环境访问 Hugging Face 返回 403，未直接核 |
| [mu `8dfebe3`](https://github.com/qybaihe/mu/tree/8dfebe36508ac0c2508735bb866756dab9283e74) 的 `kyrn/docs/03-local-judge.md` | 第三方实测（M3 Max，Core ML 移植） | 见 [mu 对象文件](../harness/mu.md) |

本条只读源码与文档，没有部署或实测。上游数字来自 README、`docs/finetune.md` 与 `BENCHMARKS.md`；mu 的数字来自其实测记录。

### 它是什么

和 Jev 同一种判断模型：状态加题目进，概率出，不生成文字。题型 `choice` / `score` / `noul`（即是非题）。与 Jev、CLM 的区别是**它是一个小编码器**：

| 检查点 | 编码器 | 参数 | 上下文 | 用途 |
| --- | --- | --- | --- | --- |
| `laya` | ModernBERT-large | 421M | 512 | 英文 |
| `laya-multilingual` | mmBERT-base | 322M | 1024，可放到 8192 | 100 多种语言 |
| `laya-typed-decisions` | ModernBERT-large | 421M | 1024 | 上游四类工作流上微调过的版本 |

一次前向答完，README 称 T4 上单题 33 ms、批量每题 7.2 ms。`Router` 按语言与文字自动选检查点。另有 ONNX 导出（CPU 可跑）、`laya-serve` HTTP 服务、MCP、TypeScript SDK；0.3.21 起支持 `min_confidence` 弃权，答案带 `abstention` 状态。

### 能不能训练

能，这是它的主要价值所在。上游自己说基础检查点零样本接近随机：typed-decisions 基准上 0.36 与 0.35，随机基线 0.318；微调后 0.766，高于 TypeSafe 公布的 Jev 0.727（`docs/finetune.md`）。

- **训练法 RLCD**：对每道题拟合「老师」给出的概率分布，不是硬标签。损失是两部分之和：对加噪声的 logits 采样做策略梯度、以严格适当评分规则为奖励；再加对同一分布的软交叉熵。
- **数据形状**：每条是 `state`、`questions`、`gold`（老师对各选项的概率）。只要题能写成三种题型之一，任何状态都行。
- **成本**：Kaggle 两张 T4 上，6000 道题的演示几分钟，约 3 万道题四个 epoch 约 4–5 小时；另有 Apple Silicon（MPS / CPU）脚本 `notebooks/laya_finetune_typed_decisions_mps.py`。
- **校准是流程的一部分**：训练前先留出一片数据，训练后按题型各拟合一个温度。上游明说出厂检查点过度自信，按置信度设门槛前必须先校准。

### mu 的实测（第三方）

mu 在 Jev 账号开通前，用 `laya-multilingual` 的 Core ML 移植把整条判断链路跑通：

- **速度**：一次 preflight（10 道题）p50 128 ms，前提是把输入补齐到固定长度档；不补齐时 p50 783 ms。
- **措辞决定准确率**：31 条有标注的消息上，同一道「要不要改代码」，长问法 42%，短问法 74%。
- **v3 题组在 37 条场景上**：「要不要改文件」「本轮类型」「先计划」可用；「需要澄清吗」不可用；三道打分题不可用，重任务和轻任务的分数挤在一起。
- **微调过的 `laya-typed-decisions` 在 mu 的题上大多弃权**；英文底座读不了中文。结论是对 mu 的题它不比多语言基础版好。
- **能力档案**：mu 在配置里把 Laya 标为不做关系题、打分题与元判断（`relate` / `rate` / `meta` 为 false）。
- **窗口静默截断**：超长时保留开头、截掉结尾，所以状态要把决定性内容放最前。
- **蒸馏路线**：mu 提出以 Jev 当老师，用判定记录微调一个专用 Laya，本机约 13 ms 一题、数据不出机器，Jev 只接本地弃权的部分；前提是判定记录在用户同意下保存状态原文（`03-local-judge.md` §4.3）。
- mu 桌面端后来改用 ONNX 导出（`mizchi/laya-multilingual-onnx`），经 onnxruntime-node 运行。

### 中文

上游 `BENCHMARKS.md` 收了一份外部社区评测（[zh-decision-bench](https://github.com/CodyQin/zh-decision-bench)，2026-09-25，laya 0.3.20）：

- 意图分类重跑：多语言检查点简体准确率 0.650、ECE 0.219，繁体 0.610。
- 业务场景：语音路由 0.883，客服路由 0.640，紧急度 0.560，是否升级 0.550。
- 选项顺序一换，答案翻转 28%（客服）与 10.6%（语音）。
- 中文是非题过度自信，拟合出的温度 10.2 超出上游 clamp 的上限。

### 对 HCTL2 的位置

- **最符合「缺省不外发」。** 322M，ONNX 在 CPU 上可跑，macOS 与 Linux 都有路径，不需要 GPU 常驻。
- **零样本弱，价值在训练。** 这与判断点备忘 §二.5「先假设判断器对、有了正反例再纠偏、判断器要迭代」对得上：老师分布可以来自 Jev（hctl2 自用放行），标签来自人的动作。前提同 mu：训练时能把判定记录里的引用还原成正文。
- **已知弱项正好落在我们要的地方。** 关系题（警觉题里的「是不是同一件事」）mu 判为不可用；中文是非题过度自信；选项顺序敏感。阈值必须按题型在自己的数据上校准，选项顺序可以每次随机并记录。
- **未定的事实。** 在 hctl2 中文样本上的准确率、微调后能否做关系题，都没测。

### 复用决策

**暂缓。** 本条只记录。若要用，先在 hctl2 自己的中文样本上，与 Jev、CLM-8B 同题对照，并试一次以 Jev 为老师的微调，再定。

## 复核记录

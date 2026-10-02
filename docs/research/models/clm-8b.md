# CLM-8B：开源、可训练的对比判断模型

> 类别：判断模型 · 证据编号：E-MODEL-CLM<br>
> 状态：证据审计 · 钉定版本与许可见「审计基线」；发布后正文不改，只在文末追加复核记录<br>
> 总览与复用决策用语见 [docs/research/README.md](../README.md)；用在哪里见 [判断点备忘](../../../.memo/design/room-judgment-20261002.md)（§六 要求新判断器先落调研）。

<a id="e-model-clm"></a>
## E-MODEL-CLM · CLM-8B

### 审计基线

| 对象 | 版本 | 许可 |
| --- | --- | --- |
| [Contrastive-LM/CLM `bb42c6c`](https://github.com/Contrastive-LM/CLM/tree/bb42c6c5bf914fd449bed2f6ca65be80602cb1f7)（2026-09-24，Python 包 `contrastive-lm` 0.1.0，仓库共 8 个提交） | 服务、客户端、微调脚本 | Apache-2.0 |
| 参考头 [`Contrastive-LM/CLM-v0.1-8B`](https://huggingface.co/Contrastive-LM/CLM-v0.1-8B)（`CLM_v0.1-8B.pt`，README 称约 75 MB） | 两个投影头的权重 | README 称 Apache-2.0；本环境访问 Hugging Face 返回 403，修订号与许可未直接核 |
| 编码器 [`Qwen/Qwen3-8B`](https://huggingface.co/Qwen/Qwen3-8B) | 冻结骨干，末 token 池化 | 未直接核 |

README 的引用条目列 Jacky Kwok、Hangoo Kang、Tarun Suresh、Jon Saad-Falcon、Marco Pavone、Christopher Ré、Azalia Mirhoseini；新闻稿称出自 Stanford 与 NVIDIA Research。说明发在 [Notion 博客](https://contrastive-lm.notion.site)，没有找到论文。本条只读了仓库，没有部署或实测；下面的效果数字都来自上游 README 或新闻稿。

### 它是什么

和 Jev 同一种「system one」判断模型：输入一份状态和几道有限选项的题，输出各选项的概率，不生成文字。不同在原理：

- **两个编码器加对比学习。** 状态编码器与动作编码器各是「冻结的 Qwen3-8B 加一个约 2000 万参数的可训练投影头」，用双向 InfoNCE 训练，让状态靠近实际采取的动作、远离其他动作。
- **答题就是打分。** 一道题等于状态（附上题目说明）对一组候选（各选项的描述）的打分：投影后算余弦，乘缩放系数，过 softmax 得到分布（`src/clm/engine.py` 头注释）。选项描述的向量可以缓存复用，所以选项多、选项重复出现时最快。
- **线协议与 TypeSafe 同形。** `POST /v1/systemone`，题型 `noul` / `choice` / `score`；为 Jev 写的请求可以原样重放。另有 `rank`，对任意候选集排序（工具名、下一步动作、N 选一的答案）。
- **训练数据**（README「Data Recipe」）：预训练约 6000 万条 Nemotron 问答对；中训练约 3000 万条由 Gemini 2.5 Flash-Lite 生成的难负例；后训练约 100 万条 agent 轨迹（ADP、Endless-Terminals、LiteCoder-Terminal-SFT），其中 40% 回放预训练数据。

上游宣称（未复现）：在 T-Rex、BFCL v4 工具调用、WikiRacing、Super Mario 上与 Jev 持平，最多快 9 倍；作为验证器轻量微调后，DeepSWE（38 道留出题）81.6%、Terminal-Bench 2.1（30 道留出题）87.6%，比 Jev 快 4.1–5.7 倍。新闻稿称单次判断 16.5 ms 对 Jev 的 149.8 ms，这组数字在仓库正文里没有找到。

### 能不能训练

能，而且便宜：骨干冻结，只训投影头，训练读的是预先算好的嵌入。

- 入口 `train/finetune.py`：`--task choice --data <typed decisions 数据集>` 训题型判断，`--task clm` 训「状态—动作」验证器；`--init-ckpt` 从参考头起步。
- 头的格式是一个 `torch.save` 字典（`state_head`、`action_head`、`logit_scale`、`cfg`），`clm-serve --ckpt` 或 `checkpoint_dir` 可以同时挂多个头。**头只对训练它时用的编码器和池化方式有效**。
- `docs/FINETUNING.md` 不是人读的教程，而是一份让 LLM 自主循环改 `finetune.py`、跑评测、保留更好结果的指令。
- 没有像 Laya 那样的温度校准步骤；概率是缩放余弦的 softmax，要套三区间阈值，得用自己的数据先校准。

### 部署形态与边界

- **需要 GPU 上的 8B 嵌入服务。** 官方路径是 vLLM 以 pooling 模式服务 Qwen3-8B（`serve_qwen3_8b.sh`），`clm-serve` 本身跑在 CPU。README 的截图环境是一张 RTX 4090。macOS arm64 上 vLLM 这条路不可用；改用别的推理器取 Qwen3-8B 的末 token 嵌入能否与训练时一致，未验证。
- **状态缺省截断在 2048 token**，要更长需同时调高 vLLM 与 `clm-serve` 的上限。
- **中文未见评测。** 骨干多语言，但三段训练数据以英文为主，仓库里没有中文结果。
- **关系题要靠改写。** 一切都是「状态对候选」的对齐，「这两段是不是同一件事」只能把两段都放进状态、把关系作为选项来问；效果未验证。
- **很新。** 仓库 2026-09-24 建，8 个提交；路线图写着更大骨干与多模态。

### 对 HCTL2 的位置

- **正文可以不出本机。** 能自己部署，满足「缺省不外发」的策略；代价是要有一台带 GPU 的机器常驻 8B 骨干，这超出 hctl2 当前对发行包和资源占用的假设。
- **与「机制先行、模型可换」对得上。** 线协议与 Jev、Pi v0.99.0 的分类接口同形，可以作为同一接口后面的一个后端。
- **与「判断器要迭代」对得上。** 只训小头、训练读缓存嵌入，适合按判定记录周期性重训；前提是训练时能把记录里的引用还原成正文（判断点备忘 §二.5 的「记录存引用、可重放」）。
- **未定的事实。** 中文表现、Mac 上的推理路径、校准，都要在自己的样本上测。

### 复用决策

**暂缓。** 本条只记录。若要用，先在 hctl2 自己的中文样本上，与 Jev（hctl2 自用放行）、Laya 同题对照，再定。

## 复核记录

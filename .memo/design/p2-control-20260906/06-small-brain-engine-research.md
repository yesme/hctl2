# Room 里的轻量判断与总结：引擎调研任务书（Grok 写，GLM、Fable 审）

> 状态：已拍板 · 所有者 2026-09-28「落」<br>
> 基线：main @ `8ec530f`（草案 v0.18.11）<br>
> 去向：`docs/research/` 根目录一份跨候选对照，新类别目录 `docs/research/models/` 下的对象文件；不写代码、不改约束层

## 一、为什么做

所有者 2026-09-28 对 Topic Room 前情提要的两条裁定：

1. 做自动版，不只做人工提要；
2. 第一步只挑不写，配置了小模型才改写（原话「第一步可以只挑不写，配置了小模型才做改写」）。v0.18.12 把它落成三档：什么模型都没配时只挑原文、不分节；配置 small-brain 后可按相关性挑选并分节；所配模型能生成文字才改写成短句。

同一轮所有者还指出，Room 另有两件活要一点「智能」：一是发觉该开新 Topic 了，在屏幕最下方给一条提示；二是 @ 某个参与者时，从它上次收到信息到这次被 @ 之间的消息里，挑出和这次 @ 相关的，必要时做个总结。三件活都不需要推理，只做判断或短总结。

三件活都是系统自身的能力：由控制面按用户的配置调用引擎，不交给 Room 里的参与者。所有者同日更正，早先「调用模型可以由参与者完成」一句不再成立。设计里给这类活留的位置叫 small-brain，意思是用户给控制面配的一个专用小模型。缺省不配，全部走机械规则。相关规矩见 `spec/project.md` §根 Context Manifest 里相关性门、压缩、滚动纪要各段，以及 `context.md` §前情提要：房间的滚动上下文。本调研回答：三件活各用什么引擎，或者根本不用模型。

候选引擎有四类：

- **Jev**：TypeSafe 公司的托管判断模型，2026-09-15 起有限早期访问。它只回答事先给定选项的问题，比如选哪一类、打几分、是还是否，并附上概率，不输出文字。
- **本地小指令模型**：在用户机器上跑的小型生成模型，能写短句。
- **本地嵌入模型**：把文字变成向量，按相似度判断两段话是否相关，不生成文字。
- **机械规则**：@、回复链、Request 关联、时间窗口这类不读正文或只做字面匹配的规则，作为基线。

## 二、三件活

| 活 | 用户看到什么 | 在设计里的位置 | 判断的形状 | 没配模型时的缺省 |
| --- | --- | --- | --- | --- |
| A 持续建议 | Room 聊着聊着，屏幕最下方出现一条「这段讨论可以开个新 Topic」，人接受或忽略 | 体验 `docs/user-experience/02-user-journey.md` §T1；`open-questions.md` 接手清单里「持续建议的触发与费用控制」 | 是非判断：到这里该不该提示 | 只有手动开题；按 T1 原文，手动开题不能算持续建议已实现 |
| B @ 时的缺口 | 被 @ 的参与者拿到「上次收到信息以来、和这次 @ 相关的消息」，不是全部新消息 | `spec/project.md` §根 Context Manifest 相关性门与滚动纪要段 | 对缺口里每条消息做是非判断：和这次 @ 相关吗；可选短总结 | 相关性门只看提及、认领、Request 关联和游标（记录每个参与者读到哪条的位置）；近期消息给全文，更早的降为标题加事件指针 |
| C 前情提要起草 | 开 Topic Room 时，预览里已经有一份按五节放好的草稿，人删改后确认 | `spec/project.md` §Room 与消息 创建 Topic Room 一段 | 只挑不写：每条消息选不选，配了引擎时再定归入五节中的哪一节，都是选择题；配了小模型才改写成短句，来源指针由组装器（控制面里拼装上下文的程序）按原消息挂上，不由模型输出 | 按回复链、@、关联 Request 这类线索挑，分节由人做；自动归纳未配置要如实显示 |

五节指：缘起与目标、已定事实与决定的理由、分歧与待答问题、所需约束与材料、来源清单。

## 三、要回答的问题

1. **Jev 的用例**
   - 9 月以来有一批开源项目用 Jev 做上下文压缩，做法是只删不写：每次调用模型前，对每段旧内容问 Jev 一个是非题「完成当前任务还需要它吗」，不需要的换成一行占位，其余原文照留。要写清它们怎么提问（问题原文）、什么时候触发、占位怎么写、实测省了多少、踩过什么坑（误删、延迟、费用、数据外发声明）。起点清单见 §七。
   - 这种做法能不能直接用在 B 上：相关的给原文，不相关的只留一行指针。
   - 分诊、安全门之类的判断用例，有没有对 A、B 可借的提问方式。
2. **三件活逐项实测**（样本见 §四）
   - A：在判断点上答「该不该提示开新 Topic」，漏报和误报分开记。给出触发条件（多久问一次、什么事件触发）和每个 Room 每天的费用上限建议。误报的代价是打扰人，要给降噪建议。
   - B：对缺口里每条消息判断相关与否。漏选（相关的被删掉）和多选分开记，漏选代价更高；统计省下的篇幅。
   - C：只挑不写，记选入的准确率、漏选率和分节准确率。小模型改写，记忠实度（有没有来源里没有的说法、有没有把未定写成已定）和中文通顺度，并和只挑不写的版本对照。
3. **引擎对比**
   - 四类引擎都要测：Jev；本地小指令模型，中文强的至少两族，规模从 1B 左右到 8B 左右，钉 revision 与 digest；本地嵌入模型，至少一个多语种的；机械规则基线。
   - 每件活都要和机械规则基线比，基线够用就建议不上模型。
4. **衡量维度**
   - 中文准确率。
   - macOS arm64 与 Linux x86_64 上的延迟、内存与 CPU / GPU 占用。
   - 费用。
   - 数据是否离开本机。
   - 能否随包发布；不能的话，能否首次使用时下载并按 digest 校验。
   - 许可证，包括模型权重本身的许可。
   - 能否记下精确版本。约束要求模型判定记录模型引用与摘要；托管的 Jev 只能记它自报的版本，要给出这条规矩对托管模型该怎么写的建议。
5. **本地模型的运行方式**
   - 候选：llama.cpp、Ollama、MLX（只限 Apple Silicon）、Rust 原生的 candle 或 mistral.rs 等。
   - 排序按四级顺序：随包官方命令行，高于官方 SDK，高于从接口描述生成，高于手写。出处见 `02-research-brief.md` §通用要求。
   - 写清与 Buck2、DotSlash 的接法。

## 四、样本与标注

- **来源**：只用本仓库已公开的内容，即 `.memo/log/` 下的聊天记录，和长 PR 的讨论串（如 #276、#279、#282、#287、#288 的正文、评审与回复）。仓库是公开的，送给 Jev 不涉及私密内容。不用任何未入库的聊天。
- **抽样**：研究员先把样本整理成消息流，每条带编号、说话方、时间、正文，提交到对照文件的同名目录里。数量至少：
  - A 100 个判断点。判断点是消息流里的一个位置，问「到这里该不该提示开新 Topic」。
  - B 20 个 @ 点。被 @ 方上次发言到这次被点名之间的消息就是缺口。
  - C 6 次开题，取讨论确实转到新话题的位置。
- **标注**：样本就绪后报所有者，由所有者转 Fable 标注正确答案。Fable 在同一分支上提交标注文件，到齐后研究员再跑。
- **测量脚本**：不入库。对照文件的证据节写清命令、版本、硬件和提问原文，原始输出放同名目录。

## 五、交付物

- **跨候选对照**：`docs/research/small-brain-engines-<完成日期>.md`，放研究根目录（目录规则见 `docs/research/README.md`：跨候选的归纳放根目录）。内容包括：
  - 三件活乘四类引擎的实测表，和 §三 第 4 项的衡量维度表；
  - 每件活的建议：缺省用什么，用户可选打开什么，不做什么；
  - 对约束的建议，只写建议，不改约束。
  - 同名目录放样本、标注和原始输出。
- **单案对象文件**：放新类别目录 `docs/research/models/`，这个目录收模型服务与本地推理的单案。
  - `README.md`：类别说明与索引。
  - `jev.md`：Jev 本身，包括能力、接口、价格、访问方式、数据政策、版本能否追溯，和 §三 第 1 项的用例。
  - 被推荐的本地运行方式与模型，各一份。
  - 格式照 `02-research-brief.md` §通用要求：文件头三行，正文分上游能力、候选比较、边界与取舍、决定建议、证据。
- **索引同步**：`docs/research/README.md` 目录规则一段加 `models/`，§条目索引加行。

## 六、规矩

- 没有 Jev 的访问权限，就如实标「未实测」，只做文档和开源项目层面的调研，不编数字。以后所有者给了访问权限，再在文末追加复核记录。
- Jev 会把正文发给第三方，这属于安全策略面（`CONSTRAINTS.md`：安全是策略面的事，由可配置的策略回答）。对照里只写需要一个什么策略点、缺省取什么，不在引擎层解决。
- 只调研：不写 `src/`，不改约束层。前情提要草稿的格式，以及给人确认前控制面要查什么，另有设计，与用哪种引擎无关，不等本调研。
- 席位：Grok 写，一个 PR，不自合；GLM 与 Fable 各自独立审，修正项改完由 Grok 合，合前报所有者。
- PR 描述三节按模板。本批只改 `docs/research/`，不改依赖，调研节引用所写文件。
- 文风：说人话，术语首次出现时给一句解释；禁词照 `CONSTRAINTS.md`。

## 七、起点清单

以下只是起点，研究员自行核实，不当结论。

- **Jev 介绍**
  - [Jev (AI model) - Wikipedia](https://en.wikipedia.org/wiki/Jev_(AI_model))
  - [Shut up and calculate: Jev's new AI primitives for coders - The Register](https://www.theregister.com/devops/2026/09/23/shut-up-and-calculate-jevs-new-ai-primitives-for-coders/5298431)
  - [What Is Jev? Inside the AI Classifier Model - MindStudio](https://www.mindstudio.ai/blog/what-is-jev-classifier-model)
  - [It's Easy to Dismiss Jev as Just a Classifier - Sebastian Raschka](https://sebastianraschka.com/blog/2026/jev-classification-generalization.html)
- **用 Jev 做上下文压缩的开源项目**
  - [jev-compactor](https://github.com/edwardyen724-g/jev-compactor)
  - [jev-compact（Claude Code）](https://github.com/HAR5HA-7663/jev-compact)
  - [pi-fast-jev-compaction](https://github.com/QuentinDanblon/pi-fast-jev-compaction)
  - [Fast-Jev-Agents](https://github.com/satiricalguru/Fast-Jev-Agents)
  - [jevcomp](https://github.com/liqunqun07/jevcomp)
  - [openclaw-jev-compaction](https://github.com/SqaaSSL/openclaw-jev-compaction)
  - [Reduce agent context with TypeSafe Jev and LiteLLM](https://docs.litellm.ai/blog/typesafe-jev-compaction)

## 八、给 Grok 的指针版提示词

```
你在 yesme/hctl2 做一份调研：Room 里三件轻量判断与总结的活各用什么引擎。任务书：main 上 .memo/design/p2-control-20260906/06-small-brain-engine-research.md（§一到 §六是要求，§七是起点清单，自行核实）。分支 grok/small-brain-engines，base main，一个 PR，不自合；样本整理好后报所有者，由所有者转 Fable 标注，标注到齐再跑；评审席位 GLM 与 Fable 各自独立审，修正项改完后由你合、合前报所有者。回报：PR 编号、分支、交付文件清单、每件活的建议、哪些实测了、哪些标了未实测。
```

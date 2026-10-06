# 发现一 · `agency:herdr_test` 的待机族

## 形态

四条用例都是：把标记交给假 harness（`src/agency/tests/standby_fixture.rs`，被测逻辑在 `src/agency/src/standby.rs`），再在预算内收集事件、断言标记出现过。

| 用例 | 断言 | 预算 |
| --- | --- | --- |
| `standby_idle_reclaims_then_uses_native_resume_and_participants_are_isolated`（`tests/herdr.rs:2222`） | `stdout_has(&collect(&mut c, …), b"OTHER")` | 10 秒 |
| `standby_rejects_a_completion_from_a_different_native_turn`（`:1836`） | `collect(&mut warm, …)` 里要有 `WARM` | 35 秒 |
| `standby_reuses_one_live_herdr_process_and_routes_each_queued_answer`（`:1512`） | `collect(&mut a, …)` 里要有 `FIRST_ANSWER` | 15 秒 |
| `a_missing_marker_closes_the_pane`（`:198`，§七 已登记） | 固定 `sleep(500ms)` + `sleep(1000ms)` 后读心跳文件，断言非空 | —— |

## 关键语义：收集器不是「等满预算」，而是「等到收尾信号」

`collect()`（`tests/herdr.rs:1251`）在收到 `Exited` 或 `DispatchReleased` 时**提前返回**。所以上面几条断言失败只有两种读法：预算耗尽，或者**运行先收了尾**——后者不受 10/15/35 秒影响。

## 排除「慢」

| 轮次 | 目标用时 | 结果 |
| --- | --- | --- |
| 绿（`37380465130` 第 7 次尝试） | `agency:herdr_test` 1:13.5s | 通过 |
| 红（作业 112000916922） | 1:16.9s | `standby_idle_reclaims…` |
| 红（作业 112003084708） | 1:20.5s | `standby_rejects_a_completion…` |
| 红（作业 112004887213） | 1:13.3s | `standby_reuses_one_live…` |
| 绿（同上） | `control:chat_native_test` 9.2s | 通过 |
| 红（作业 112003084708） | 7.5s | `native_matrix_create…` |

红绿同量级：不是整体变慢，而是**在预算内提前收尾**。

## 待机适配器里的毫秒级闸门（候选成因）

`src/agency/src/standby.rs`：

- 工作循环 `recv_timeout(25ms)`（`:202`），空闲回收按 `idle` 比较（`:206`）
- 信任键门 `trust_key_at.elapsed() >= 200ms`（`:639`）
- 中断门 `sent.elapsed() >= 3s`（`:421`）
- 就绪等待 `Instant::now() + 30s`（`:601`）
- 任一处判定异常都会走「`ProtocolError` → 关会话 → `DispatchReleased`」（`:218`–`:228`），该轮不会再有 `Proposal`；`run_job` 里「迟到的 Proposal 是否可用由治理层判」的注释（`:384`）说明迟到是被承认存在的

夹具侧已按并行 CI 调过一次预算：`tests/herdr.rs:1317` 的注释写着「冷启动有 30 秒就绪预算，并行 CI 不能在 10 秒就判失败」。也就是说这一族此前已按「负载压预算」修过一轮。

## 未钉死的部分（如实标注）

**逐事件交错尚未钉死。** 断言只打印表达式本身，不打印收集到的事件，所以从日志分不清「预算耗尽」与「提前收尾」，也无从知道是哪个闸门先动。要钉死得先把失败自证做出来（见建议 ①②）。

对照材料：本机（macOS arm64）跑 `root//...` 全量曾 387 passed，唯一失败是被 buck 守护进程搅死的 `complete-test`（与用例无关）；强负载复现（8 路忙循环压满 CPU，反复跑这两个目标三轮）在本报告随附 PR 的评论里记录结果——若未复现，说明该族的暴露面只在 CI runner 的并行度上。

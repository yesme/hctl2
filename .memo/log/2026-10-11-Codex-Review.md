# Codex · #423 评审修正与再实跑

> 2026-10-11，Ubuntu。冻结记录；补充 [原 GitHub 实录](./2026-10-11-Codex-GitHub.md)，不改原记录。范围为第 9 包第 2 条。

## 原生测试证据误判的修正

[Claude 找问题结论](https://github.com/yesme/hctl2/pull/423#issuecomment-6102405517) 的 P2 成立：旧断言只要 rollout 有一条退出码 0、含 Ran / OK 的命令就过，不能证明 harness 测了本 Task 的改动。

修正按原生 CommandExecution 的 argv 核 unittest discover、完整的 `-s` 本 Task 目录参数和 `-p test_calculator.py`；cwd 必须是 session_meta.cwd 对应的 file URI。读取最后一条已结束的匹配执行，后一次失败覆盖前一次成功。原生 status 必须 completed、exit_code 为 0，实际运行数量大于 0，输出最后一行为 OK。工作树身份仍由原入口的 canonicalize / git-common-dir / 基线断言核。

反例覆盖旧 Task 目录、带后缀的相似目录名、错工作树、后一次失败（调整参数顺序、给目录加引号也不能漏掉），以及零测试和只有文字自述的输出。没有改 Agency、封存、准入或平台实现。

```sh
./src/buck2 test root//packaging/release:room-cli-test -- \
  --env TMPDIR=/var/tmp/hctl2-demo3-codex \
  --test-arg=native_test_evidence --test-arg=--nocapture
# 40d56f3c-a618-4432-b422-ba30c416b0d9: 2 passed / 0 failed

# 临时仅把选择器退回旧的任意 Ran / OK 断言，保留反例
./src/buck2 test root//packaging/release:room-cli-test -- \
  --env TMPDIR=/var/tmp/hctl2-demo3-codex \
  --test-arg=native_test_evidence_is_bound_to_this_task_and_the_last_attempt \
  --test-arg=--nocapture
# 101e0e56-844f-495d-bc38-d86bd510e780: 0 passed / 1 failed
# assertion failed: last_native_task_test(&records, cwd, code_dir).is_none()

# 逐字恢复源文件，再跑包含同一红例的两条用例
./src/buck2 test root//packaging/release:room-cli-test -- \
  --env TMPDIR=/var/tmp/hctl2-demo3-codex \
  --test-arg=native_test_evidence --test-arg=--nocapture
# 2fb1066d-f3d3-4fff-8a14-df0aca5f1af6: 2 passed / 0 failed
# 恢复文件 SHA-256 4daba7f108b9c708bd1a6c705353cb8b859dff531c13ba024529632a6e765314

./src/buck2 test root//packaging/release:room-cli-clippy-clean-test
# fd554cc9-da59-41f9-918e-9a41bf7275f0: Pass 1 / Fail 0
# 1 Clippy reports, all empty
```

退回版本不提交。

## live-6 复用披露

通过的 live-6（PR #8）先读取 live-5 已合入 PR #6 的两个文件，再写到本 Task 的新目录；两份文件逐字相同。原生 rollout 里的前后 SHA-256：

- calculator.py：`a6b493ad5c175da2cc65bc1dcf08feacf2e5d0e56aa1c32f707ad16c550c6280`。
- test_calculator.py：`978abafe300c718f114624c52d55b89aa8a0aefef6d0ccea9b1f891576973855`。

live-5 原来从不存在的目录写出实现，整链最后因作者读错 completed.result.items 而失败；它仍按失败计。live-6 是复用这轮实现的真实 ChangeSet 和完整链，不能称作重新从零写实现。它确实在本 Task 新目录跑了 6 条 unittest，原生退出码 0；原始整链与文件写入事实不变。本段补足原实录披露。

## 收紧断言后的真实整链 live-7

```sh
./src/buck2 test root//packaging/release:room-cli-test -- \
  --env HCTL2_HARNESS_LIVE=1 --env HCTL2_GITHUB_LIVE=1 \
  --env HCTL2_AGENCY_REQUEST_TIMEOUT_MS=30000 \
  --env CODEX_HOME=/home/jackywang/.codex \
  --env TMPDIR=/var/tmp/hctl2-demo3-codex \
  --test-arg=demo3_github --test-arg=--include-ignored --test-arg=--nocapture
# Build 7f01855b-2298-46f1-ac80-75ec6a884fc6
# 1 passed / 0 failed / 0 ignored / 13 filtered out (236.46s)
# Tests finished: Pass 1 / Fail 0
```

本机并行负载下，旧头的独立核验曾在 Agency catalog / accept 报一次 gRPC CANCELLED；原样重跑通过。本轮显式用既有原生客户端的每进程请求预算 30000ms，未改全局配置或处理函数，不将其称作缺省 5 秒预算验证。既有自动 agency start 的 30 秒超时观察另见两席评论；本轮仍走真实前台 agency serve 入口。

[canary PR #12](https://github.com/yesme/hctl2-canary/pull/12) 的 [canary run 38088770129](https://github.com/yesme/hctl2-canary/actions/runs/38088770129) 完成 success 后，经 integration 按保护合入。平台只读回读为 MERGED，head `e43a71e60d5daadbc328bef19b9b887c4d905f43`，merge_commit `93f2c8041642bf1ffa2912aba4cbfeb9f3a7e819`。没有手动 push、建映射或 merge。

| 对象 | 实际回读 |
| --- | --- |
| baseline | `02b9cddea0706e6de73826fe40dd5d7ea7b74bfe` |
| ChangeSet | `cs-941a52880ea1594363f81ea81aba992dae358f75ab3c859f17c4464def000aa5` |
| Revision | `csr-8250c021b3481aa5fa286037074aaf5f2beed4b59c8b39e2db6320c698e177ae` |
| result_tree_sha | `cbfc915d5f3f683e23fa7dc44855902200ad01c0` |
| Integration Receipt | `receipt-cad9fc44c203b2a7a7e0d2585b70379981cad422f4bf6425f6a6f1016f1a12ab` |
| merge / target_head_after | `93f2c8041642bf1ffa2912aba4cbfeb9f3a7e819` |
| Task | `0998dd8f694f784951471234b65645a1ce44f5dd8bbb70546caf5f7f42c41994` |
| Completion Receipt | `0998dd8f694f784951471234b65645a1ce44f5dd8bbb70546caf5f7f42c41994:2` |

Proposal、准入、独立 hctl2-tool seal 和发布候选树相等；changeset diff 只增两份 .py。Git readback contains=true，合并提交的父含本轮候选、合并提交不是候选。Task lifecycle=completed；Completion Receipt 的 unmediated 项引用上述 Integration Receipt。

原生记录：`/home/jackywang/.codex/sessions/2026/10/11/rollout-2026-10-11T05-43-29-01a127c5-5e72-7ee2-b6f1-d1cfd80f1f20.jsonl`，thread `01a127c5-5e72-7ee2-b6f1-d1cfd80f1f20`。冻结 Bundle + Agency 边界正文与原生 user input_text 逐字相等，6936 bytes，SHA-256 `7245854dfbf4b307967d2fd90c26ce905e095abee9cd728dab312d6bd0f2a043`。作者另用 Python 解析 Buck2 stdout 的 USER BODY 与原生记录的 input_text，再核逐字相等。

本 Task 的原生 CommandExecution（完整 argv、cwd 与实际输出）：

```json
{
  "id": "exec-5c447153-3833-40a3-bbaa-1d1bd1f46686",
  "command": [
    "/usr/bin/zsh", "-lc",
    "/usr/bin/python3 -B -m unittest discover -s canary_cases/codex_1791668553430 -p test_calculator.py -v"
  ],
  "cwd": "file:///tmp/hctl2-exec-96da0892ce8b2aaa06e9/write-worktrees/31cc88bc35706a08869d7d2283a1a9a51ecabc0172d04c819bd32f4dfe746241/cs-941a52880ea1594363f81ea81aba992dae358f75ab3c859f17c4464def000aa5",
  "status": "completed",
  "exit_code": 0
}
```

```text
test_exact_division (test_calculator.CeilDivTests.test_exact_division) ... ok
test_large_integers (test_calculator.CeilDivTests.test_large_integers) ... ok
test_negative_inputs (test_calculator.CeilDivTests.test_negative_inputs) ... ok
test_positive_inputs (test_calculator.CeilDivTests.test_positive_inputs) ... ok
test_zero_denominator (test_calculator.CeilDivTests.test_zero_denominator) ... ok
test_zero_numerator (test_calculator.CeilDivTests.test_zero_numerator) ... ok

----------------------------------------------------------------------
Ran 6 tests in 0.001s

OK
```

作者独立复跑同一目录 unittest 6 条通过，另 7 个独立 ceil_div 例子通过。新文件实测哈希（两者均不等于 PR #6 / #8 的文件；这只证明字节不同，不据此推断模型未参考旧实现）：

- calculator.py：`d30396759e998108cff287a06e1eac7517f37e6745cd8ffeef8dea06a8075160`。
- test_calculator.py：`fe318ec2862a474ca4849f775fd52cb3a2448646025ba589161e09b58b8d3e28`。

## 两席旧头独立证据

[Kimi 核验](https://github.com/yesme/hctl2/pull/423#issuecomment-6102503132) 在 84d0515 从零实跑 PR #10，1 passed / 0 failed（246.24s），merge `02b9cddea0706e6de73826fe40dd5d7ea7b74bfe`、Task Issue #9 完成，并独立做 GitHub 参数退回红→恢复绿、两平台 CI 日志与 tar 复现。首次 Agency accept 取消与真实 harness 下 complete-test 自动启动超时都如实记在评论中。该结论只属于旧头；本次追加提交需要两席重新复核。

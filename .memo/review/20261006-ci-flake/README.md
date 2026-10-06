# CI 原生套件抖动的审议报告（2026-10-06）

> 状态：已落地 · 供小活 M 消费
> 基线：main @ 47ca566（观察覆盖 2c68b3f → 47ca566）
> 去向：小活 M 的输入（核销记录：M 的 PR）
> 说明：调查 · 对象：`Code` workflow 的 `Buck2` 作业上反复变红的原生用例 · 作者：DeepSeek 席
> 方法：只读代码 + CI 日志取证 + 本机对照。本报告与随附 PR 不改任何代码。

## 范围

2026-10-05 到 10-06，小活 J、K、L 三个 PR 的合入过程中，`Code` 的 macOS `Buck2` 作业反复变红，每次红在不同用例上。本报告把这批观察收成可裁决的发现：哪些同族、机制分别是什么、哪些已登记过、建议下一步做什么。

## 索引

- [发现一 · herdr 待机族](./herdr-standby.md)：`agency:herdr_test` 本轮观察到的四条
- [发现二 · 原生服务族](./native-services.md)：`control:chat_native_test` 的重启后重发、`control:task_native_test` 的 Gitea 建仓
- [发现三 · 已登记的同族](./registered.md)：§七 里的三条，以及与本报告的关系
- [建议](./recommendations.md)：测试侧的三处小改，与不建议的方向

## 观察汇总（带取证）

| 时间（UTC） | 运行 / 作业 | 目标（用时） | 用例 | 平台 |
| --- | --- | --- | --- | --- |
| 10-05 22:13 | [37380465130 / 112000916922](https://github.com/yesme/hctl2/actions/runs/37380465130/job/112000916922) | `agency:herdr_test`（1:16.9s） | `standby_idle_reclaims_then_uses_native_resume_and_participants_are_isolated` | macOS |
| 10-05 22:17 | [37380465130 / 112003084708](https://github.com/yesme/hctl2/actions/runs/37380465130/job/112003084708) | `control:chat_native_test`（7.5s） | `native_matrix_create_send_resync_freeze_account_data_and_encryption` | macOS |
| 10-05 22:17 | 同上 / 112003084708 | `agency:herdr_test`（1:20.5s） | `standby_rejects_a_completion_from_a_different_native_turn` | macOS |
| 10-05 22:23 | [37380465130 / 112004887213](https://github.com/yesme/hctl2/actions/runs/37380465130/job/112004887213) | `agency:herdr_test`（1:13.3s） | `standby_reuses_one_live_herdr_process_and_routes_each_queued_answer` | macOS |
| 10-05 22:35 | [37380465130 / 112009230578](https://github.com/yesme/hctl2/actions/runs/37380465130/job/112009230578) | 三个原生目标 | ——（同一提交，绿） | macOS |
| 10-05 17:07 | [37345645077 / 111883583662](https://github.com/yesme/hctl2/actions/runs/37345645077/job/111883583662) | `control:task_native_test` | `native_gitea_conditionals_dependencies_comments_delete_and_recovery` | Linux |
| 10-05 17:08 | [37345645077 / 111883583613](https://github.com/yesme/hctl2/actions/runs/37345645077/job/111883583613) | `agency:herdr_test` | `a_missing_marker_closes_the_pane` | macOS |
| 10-05 15:26 | [37332731935 / 111839809731](https://github.com/yesme/hctl2/actions/runs/37332731935/job/111839809731) | `agency:herdr_test`、`control:services_test` | `a_missing_marker_closes_the_pane`、`backup_refuses_while_an_unconsumed_hosted_component_runs` | Linux |

前三行与第五行来自同一次运行的连续尝试：同一提交（小活 L 的 `30fd9b3`）在两平台 Buck2 上先红三次、后绿一次。

## 基线：`main` 自己也在红

`main` 的 `Code` 运行近 5 次里 4 次 failure：`44632e3`、`f997207`、`56c4539`、`97f9a1f`、`44c310a`；其中 `2c68b3f`——小活 L 分支的 base——本身即 failure。也就是说，这批红不是哪一个 PR 引进的，任何基于当前 `main` 的 PR 都会被同一批用例牵连。

## 一句话结论

不是「机器慢」：同一目标红绿用时几乎相同。是**运行先收尾、期望事件没到**这一族——测试在 `Exited`/`DispatchReleased` 上提前停止收集，而负载会改变谁先到；证据与机制见各发现。

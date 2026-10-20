# Run 生命周期与冻结施工清单

> 状态：调研与实测 · 2026-10-21 · Codex（ubuntu）<br>
> 基线：main `54e34fb`；P2.5 第 3 包；权威为 [Run 约束](../../design/spec/run.md)

## 复用的机制

2026-10-21 核 [SQLite 事务](https://sqlite.org/lang_transaction.html) 与 [AWS transactional outbox](https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/transactional-outbox.html)：领域更新与待投递意图同事务，外部动作在提交后发生，消费者按原关联键恢复。SQLite 事务不嵌套；不能把撤旧与建新拆成先后两次提交。

本库 Store 已有单写者代次、Command 幂等、版本检查、EffectIntent、投递前 Unknown、回读与事务回滚，不再建队列或数据库。Participant 已有接受的工种、Worker Profile 和选择策略校验，第 5a 包已有同键计票上界；Run 复用这些入口。引擎接线归第 2 包，本包用内部假端口验证生命周期，不启动 Dagu，也不把引擎进度当完成凭证。

## 验证

以下均为 Codex（ubuntu）在私有临时状态根上的实跑（2026-10-21），没有访问默认状态根或模型/平台凭据。公共入口是构建的真实 `hctl2` → 认证 socket → control → Run。引擎、施工图批准读回、Task 源快照和隔离证明是明示内部夹具；不是 Dagu 实例、真实 harness 或 Git 封存实测。真实引擎与义务派工分别由第 2、4a 包接入。

### 验收映射

| `09` §四 第 3 包 | 实际验证 |
| --- | --- |
| 1 状态机 | `transition_table_is_closed_for_all_states` 对照全部 81 状态对；`every_nonterminal_phase_can_fail_or_replace_and_terminal_runs_cannot_revive` 走真实准入处理；`cli::real_cli_previews_starts_pauses_resumes_replaces_and_cancels` 从真实命令行走五命令、非法恢复、终态拒绝；状态确认用有状态假引擎 |
| 2 超时交人 | `transitional_timeout_only_marks_attention_and_retains_task_occupancy` 的三种过渡态、1234ms 声明；`default_timeout_and_blocker_snapshot_changes_are_rechecked_at_admission` 核 900000ms 缺省；`cli::real_cli_guards_timeout_quorum_and_placement_preserve_frozen_authorization` 起真实 daemon 等墙钟关注，仍是 Starting |
| 3 启动准入 | `start_is_atomic_idempotent_and_freezes_permission_to_integrate`、`all_start_guards_and_preview_cas_fail_without_partial_records`、`archived_closed_pending_and_stale_task_inputs_fail_closed`；真实 CLI 两用例核幂等、无 token 拒绝、Task 数量与归档 Project 拒绝。Task 契约/占用用内部完整材料夹具核 CAS，不冒充真平台往返 |
| 4 阻塞方与票数 | `blockers_are_confirmable_preview_inputs_not_automatic_rejection`；`candidate_policy_diversity_and_full_range_quorum_use_frozen_content` 核全部候选、未知内容、已知内容同键与初始多样性；真实 CLI 第二用例核未知警告与 `QUORUM_UNREACHABLE`。阻塞快照推进在确认时重新比较 |
| 5 启动授权 | CLI 第一用例预览 automatic、替代为 human_preview；原 Manifest 不变，show 显示新方式；直接准入核冻结目标、方式、策略、超时与部署返工上限。后续 reducer 必须用该 Manifest，实际 Integration Intent 未在此包实现 |
| 6 替代与引用 | `replacement_rolls_back_old_revoke_new_run_slot_and_outbox_together` 在新启动 outbox 注入冲突，旧授权/旧占用/新清单/新意图全回滚；`cancellation_does_not_release_unknown_writes_or_revoke_another_run`、`unregistered_attempt_is_unknown_not_proof_of_an_empty_runtime` 保留未知；`unbound_run_and_other_display_references_do_not_own_or_stop_task` 核无 Task 的 Run 不误挡 Task；真实 CLI 两用例核替代、不可变清单、place 审计与关闭展示 Room 不停原 Run |
| 7 退回看红 | 下列三次均编译成功后由断言变红，逐次恢复源文件后再运行全集 |

另外 `dispatch_registration_checks_frozen_spec_and_cannot_cross_revocation` 验真实事务片段：按冻结选择登记实际 Sealed Spec/Bundle，撤销后不能登记新 Attempt，事务中的 Attempt 记录随失败回滚。`frozen_content_contract_ignores_consumer_and_rejects_changed_delivery` 核未来内容约定不借消费者字段混淆正文；`completion_needs_control_predicates_and_keeps_completion_pending_slot` 核完成证明与 Task 独立验收边界。它们没有真实派工或完成证明生产者，不把内部测试端口算成后续包已交付。

### 三次退回

每次命令相同：

```bash
./src/buck2 test root//crates/run:lifecycle_test
```

1. `model.rs` 的 `State::allows` 临时加入 `(Cancelled, Running)`：Build `bd0b49ec-bcb2-416f-988b-60eb4ded0c42`。

   ```text
   transition_table_is_closed_for_all_states ... FAILED
   assertion `left == right` failed: Cancelled->Running
   test result: FAILED. 18 passed; 1 failed; 0 ignored
   ```

2. `engine::mark_timeouts` 写关注前临时设 `run.lifecycle = State::Cancelled`：Build `9f96e8f5-c08a-49d3-92ae-c21ffe6c4669`。

   ```text
   cli::real_cli_guards_timeout_quorum_and_placement_preserve_frozen_authorization ... FAILED
   transitional_timeout_only_marks_attention_and_retains_task_occupancy ... FAILED
   test result: FAILED. 17 passed; 2 failed; 0 ignored
   ```

3. `admit` 的最终 submit 前，Replace 临时用独立 Store submit 调 `task::plan_run_occupancy` 清旧占用，再进原事务：Build `2232696d-7c71-4b8a-850f-61c50c6ef5ac`。

   ```text
   replacement_rolls_back_old_revoke_new_run_slot_and_outbox_together ... FAILED
   assertion `left == right` failed: failed replacement cleared occupancy
   every_nonterminal_phase_can_fail_or_replace_and_terminal_runs_cannot_revive ... FAILED
   test result: FAILED. 17 passed; 2 failed; 0 ignored
   ```

### 恢复后的全集

```bash
./src/buck2 test root//crates/run:lifecycle_test root//crates/run:domain_test root//crates/task/... root//apps/cli:unit_test root//apps/control:unit_test root//apps/control:boundary_test root//build/tests:clippy_clean_test root//build/docs:profile-research root//build/docs:profile-src
```

Build `fa8434a7-655f-40e8-a2f4-277338599287`：

```text
run:lifecycle_test   19 passed; 0 failed; 0 ignored
run:domain_test      18 passed; 0 failed; 0 ignored
task:domain_test     32 passed; 0 failed; 0 ignored
cli:unit_test         6 passed; 0 failed; 0 ignored
control:unit_test    78 passed; 0 failed; 2 ignored
control:boundary_test 14 passed; 0 failed; 0 ignored
69 Clippy reports, all empty.
check_links: OK (169 markdown files)
check_dead_names: OK
Tests finished: Pass 9. Fail 0.
```

两个 ignored 是原有真平台用例：`integration::tests::live_github_canary_protected_main_is_merged_only_through_a_pull_request_with_a_pinned_head` 与 `review::tests::live_gitea_demo_revision_is_pushed_and_one_review_request_is_opened`；本包没有新增 ignored 或把它们算作通过。

```bash
cargo +1.98.0 fmt --manifest-path src/Cargo.toml --all --check
cargo +1.98.0 metadata --manifest-path src/Cargo.toml --format-version 1 --offline --locked
```

两条退出码均为 0；Cargo 只用于格式核查与锁图读取，没有用于编译/测试。Cargo.lock 只补已有第一方 context、gate-tally、participant、repo、task 五条依赖，无新三方版本。


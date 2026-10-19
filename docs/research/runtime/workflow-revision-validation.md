# 施工图登记与编译：第 1 包验证

> 状态：实现证据 · 2026-10-20 · Codex（ubuntu）<br>
> 基线：main `84d2985`；分支 `codex/workflow-revision`<br>
> 范围：P2.5 第 1 包；接口见 [Run README](../../../src/crates/run/README.md)，设计权威见 [Run 约束](../../design/spec/run.md)

## 本机入口与边界

使用仓库 Buck2 原生目标、锁定的 Dagu 2.15.1。三个 CLI 用例分别从真实 hctl2 二进制，经本机认证 socket，到 control 的 workflow 路由、Run prepare/admit。每例独立状态根，无平台凭据或模型登录，不访问用户的 hctl2 状态。Project、Task 与第 7 包尚未提供的读回产物由内部夹具准入；**没有把模型读回实测冒充已完成**。

- `workflow_real_cli_previews_then_registers_compiles_and_approves`：预览不写，确认登记；同输入重放不多建版本；原生 Dagu 校验后保存部署；缺省读回的批准经可信测试替身通过；show/list 可读；确认后改输入不能复用 token。
- `workflow_real_cli_rejects_cycle_dependency_and_old_delivered_snapshot`：真 CLI 拒绝静态环、未落地的产物引用、旧快照读回。
- `approval_real_cli_resolves_actual_producers_and_current_source_roster`：来源 Room 真实冻结消息、Room Invocation 的冻结 spec/profile；批准按实际产出配置比较。改变当前名册后再批准拒绝，selection 版本变化不把同一成员误认成另一个人。

用例里的“真实 Room Invocation”指 Store 里的对象种类与读取路径；其 spec 由测试夹具产生，未启动真实 harness。读回隔离、摘要和回避按与未来第 7 包相同的 Run 读取入口核验。

## 绿灯

```text
./src/buck2 test root//crates/run:domain_test root//apps/cli:unit_test root//apps/control:boundary_test root//build/tests:clippy_clean_test root//build/docs:profile-research root//build/docs:profile-src

run:domain_test: 16 passed; 0 failed; 0 ignored
apps/cli:unit_test: 6 passed; 0 failed
apps/control:boundary_test: 14 passed; 0 failed
65 Clippy reports, all empty.
check_links: OK (165 markdown files)
check_dead_names: OK
Tests finished: Pass 6. Fail 0.
```

其后补上 validator 相对临时目录与错误版本回归；同一个原生用例先校验有界 loop/fork，再校验相对目录，最后用 `/bin/echo` 作为错误版本二进制，得到 WORKFLOW_ENGINE_VERSION。最终复跑：

```text
./src/buck2 test root//crates/run:domain_test root//build/tests:clippy_clean_test
16 passed; 0 failed; 0 ignored
65 Clippy reports, all empty.
Tests finished: Pass 2. Fail 0.

cargo +1.98.0 metadata --locked --format-version 1 --offline
exit 0
```

最终候选在补齐重复 Seat 引用去重后，再跑四目标：

```text
./src/buck2 test root//crates/run:domain_test root//build/tests:clippy_clean_test root//build/docs:profile-research root//build/docs:profile-src
16 passed; 0 failed; 0 ignored
65 Clippy reports, all empty.
check_links: OK (166 markdown files)
Tests finished: Pass 4. Fail 0.
```

同一 Seat 相同声明重复引用，在 quorum=2 时 WORKFLOW_QUORUM，在 quorum=1 时通过且编译映射只有一个 Seat；同身份而声明互相冲突仍是格式错误。

Cargo 只核元数据、格式一致性与生成第一方依赖图，构建、测试、实际格式化均走 Buck2。没有新增外部 crate。

## 七条验收对应

| 条目 | 用例与实际核验 |
| --- | --- |
| 1 格式与 Profile | schema_profile_and_side_effects_are_closed；未知字段、command/script/http/action/agent/harness/subdag/repeat 均拒绝；原生 YAML 只含 human.task |
| 2 判据与边 | artifact_edges_land_on_declared_future_outputs、static_cycle_is_rejected；无判据、无生产者、类型错、悬空、外部纯排序边拒绝；结构件、有界 loop/fork、未来输出引用通过 |
| 3 票数 lint | quorum_refuses_only_proven_shortfall_and_warns_about_unknown_keys；3 票 / 2 必需席拒绝；相同模板的两席、法定票数 2 登记并提示，不把未知判同键 |
| 4 保存与关联 | registration_is_atomic_idempotent_append_only_and_does_not_start、context_edit_is_a_new_revision_and_template_does_not_skip_readback；跨 Project Task 拒绝，失败无关联，响应重放无重复，新版不覆盖旧版，未批准不获启动前置 |
| 5 读回 | readback_requires_registered_claimed_and_actually_delivered_digests、no_source_room_only_exempts_roster_not_isolation_or_production_origin、current_roster_and_actual_producer_profiles_are_checked；加上真实 CLI 的第三例 |
| 6 CLI | 上述三个 CLI 用例；register/compile/approve 逐一预览确认，show 展示图、判据、边与提示，list 限本 Project |
| 7 退回看红 | 下节三次，每次先恢复再退下一处，最后完整复绿 |

只改 context 的新 Revision 编译结果相同；显式返工上限、缺省 2 轮、冻结动态槽与结构分支见 explicit_loop_limit_and_structural_branches_remain_frozen、bounded_loop_new_checkpoints_and_frozen_fork_compile。展开前资源检查拒绝超过 16384 个检查点，避免大但有限的轮数耗尽控制面内存；不是让模型在运行中变图。

## 三次退回

三次均运行同一条完整目标，不把构建错误算红：

```bash
./src/buck2 test root//crates/run:domain_test
```

1. 暂将 `visited != nodes.len()` 的检环拒绝分支包在 `false &&` 后，保留引用。输出：

```text
test static_cycle_is_rejected ... FAILED
test cli::workflow_real_cli_rejects_cycle_dependency_and_old_delivered_snapshot ... FAILED
test result: FAILED. 14 passed; 2 failed
```

2. 恢复后，将源输出与产物边类型的比较拒绝分支包在 `false &&` 后。输出：

```text
test artifact_edges_land_on_declared_future_outputs ... FAILED
test cli::workflow_real_cli_rejects_cycle_dependency_and_old_delivered_snapshot ... FAILED
test result: FAILED. 14 passed; 2 failed
```

3. 恢复后，将实际交付条目里的原摘要、正文摘要与登记摘要比较的拒绝分支包在 `false && (...)` 后，保留所有表达式。读回记录声称新快照、实际 Bundle 仍装旧快照，并重算传输层摘要，输出：

```text
test readback_requires_registered_claimed_and_actually_delivered_digests ... FAILED
test result: FAILED. 15 passed; 1 failed
```

三处均通过 finally 恢复原文件，再运行前述六个检查目标复绿；未提交退回版本。试过直接给 Buck test executor 传 libtest 位置参数，被 executor 拒绝，未算测试红；正式证据均来自上述完整目标。

## 没做

Run 启动、运行时归约、引擎服务与适配器、真实无 Run 模型读回分别由后续包完成。本包的部署有被动检查点与显式 activation 映射；尚未把运行时跳过、返工超限或失败边推进当成实测。Manifest 后续直接冻结部署的 frozen_rework_limits，不再另算默认值。

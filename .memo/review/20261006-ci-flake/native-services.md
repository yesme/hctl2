# 发现二 · 原生服务族

> 状态：已落地 · 供小活 M 消费
> 基线：main @ 47ca566（观察覆盖 2c68b3f → 47ca566）
> 去向：小活 M 的输入（核销记录：M 的 PR）

## 2.1 `control:chat_native_test`：重启后重发（macOS）

用例 `native_matrix_create_send_resync_freeze_account_data_and_encryption`（`src/apps/control/src/chat/native_tests.rs:1027`）把原生 Tuwunel `kill` 掉再重启，然后：

```rust
for _ in 0..300 {
    if client.guard(other_id).is_ok() { break; }
    std::thread::sleep(Duration::from_millis(50));
}
assert_eq!(
    client.send(other_id, "restart-transaction", "重启前后唯一").unwrap(),
    retained
);
```

**机制**：`guard` 通过 ≠ 幂等保留状态已恢复。重启后的服务可能已经能应答，但重启前保留的那一轮还没装载，于是重发被当成新的一轮执行，返回值与 `retained` 不同——断言以 `assertion left == right failed` 的形式落下（作业 112003084708，7.5s 即失败）。负载越高、重启越慢，这个窗口越大。

## 2.2 `control:task_native_test`：Gitea 建仓（Linux）

用例 `native_gitea_conditionals_dependencies_comments_delete_and_recovery` 报：

```
called `Result::unwrap()` on an `Err` value:
StoreError { code: "PLATFORM_UNAVAILABLE", message: "tea API POST user/repos not confirmed…" }
```

（作业 111883583662，10-05 17:07Z，Linux。）这是「结果未获确认」这一类型：本地 Gitea 的 `tea` 调用在预算内没有拿到确认。

## 与发现一的关系

两族都是「用预算等一个跨进程确认」，但闸门位置不同：发现一在**待机适配器内部**（毫秒级轮询与门限），发现二在**服务就绪边界**（重启后的状态装载、本地服务的确认窗口）。修法也因此不同（见建议 3 与建议 1、2）。

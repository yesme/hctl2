# 2026-10-11 · Codex / GitHub 实录

Codex 第二席，Ubuntu。实现基线 `main@30bd728f54daed90b84c0b2fabc6749e34d395b1`（#410 已合）。本记录核 [开工书](../design/p2-control-20260906/07-demo-kickoff.md) §四第 9 包验收第 2 条；第 6 包第 11 条是组件版。这里只记录 Codex / GitHub 链，不宣告第 9 包九条全部通过；Claude / Gitea 链由 `claude/pkg9-main` 负责。

## 入口与重跑命令

新增 [真实命令行用例](../../src/apps/cli/tests/room/github.rs)，接进 `root//packaging/release:room-cli-test`。control、Agency、CLI 和 hctl2-tool 均由 Buck2 构建；平台客户端来自真实依赖包。没有 `Store::open` 写治理记录、准入 seam 或平台映射 seam，没有伪造 harness 回答。调用 `Fixture::command_ns` 的命令先预览，再带同一预览令牌确认；令牌不收录。本文 JSON 为实际 stdout 选录，删除平台响应里重复的头像、仓库描述等公开字段；没有读取、打印或保存凭据。

临时目录必须短，避免 Unix socket 路径上限。实际运行命令：

```sh
mkdir -p /var/tmp/hctl2-demo3-codex
./src/buck2 test root//packaging/release:room-cli-test -- \
  --env HCTL2_HARNESS_LIVE=1 --env HCTL2_GITHUB_LIVE=1 \
  --env CODEX_HOME=/home/jackywang/.codex \
  --env TMPDIR=/var/tmp/hctl2-demo3-codex \
  --test-arg=demo3_github --test-arg=--include-ignored --test-arg=--nocapture
```

```text
Build ID: 458e87e4-db3b-4fbc-a1c9-459abb2e0be6
LIVE CLI demo3 GitHub: Codex 01a1276e-b3ff-7781-bb4c-a4a1e09bf01b -> invocation invocation-0c8bbe940cf6d739422693497515792bcd86ecce9675e6d3d413b621eb54399c -> revision csr-5fa60a1b6301785c7e7f60896ff778302c9e5221d014ec3f358d6bf67c04f769 -> PR #8 at ad8f53efa24b89989e53ee04271e4cf94abc06e2 -> merge "2cac2a9b83a5b033bfc5c07d325c490c6ee135b8" -> Integration Receipt "receipt-b3fd3bb7ad875250cdce42be10cd2bf00e892e81a96d8cf72a9dd2a05bf8eb7c" -> Task bde5516b4bbdae77a63a8e7fe4cb6e6169d9c6f0509af8475ee11d0831a84558 completed; human delivery previews: integration, completion
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 260.42s
Tests finished: Pass 1. Fail 0. Timeout 0. Fatal 0. Skip 0. Omit 0. Infra Failure 0. Build failure 0
```

默认执行时此用例 `ignored`，注释标 `UNVERIFIED`，要求本机 Codex 登录、gh 推送与保护规则读取权限。Agency 的原生目录启动还要求两家 harness 可发现。CI 跳过本机登录用例，不计作真实整链通过；本机显式实跑的输出见上。

## 注册、Project、契约与 Room 调用

靶子为 [yesme/hctl2-canary](https://github.com/yesme/hctl2-canary)，Repo 原生 id `1407796416`。先 GET 确认以下保护，再 clone main；测试不改保护：

```json
{
  "required_pull_request_reviews": {
    "dismiss_stale_reviews": false,
    "require_code_owner_reviews": false,
    "require_last_push_approval": false,
    "required_approving_review_count": 0,
    "url": "https://api.github.com/repos/yesme/hctl2-canary/branches/main/protection/required_pull_request_reviews"
  },
  "required_status_checks": {
    "checks": [
      {
        "app_id": null,
        "context": "canary"
      }
    ],
    "contexts": [
      "canary"
    ],
    "contexts_url": "https://api.github.com/repos/yesme/hctl2-canary/branches/main/protection/required_status_checks/contexts",
    "strict": false,
    "url": "https://api.github.com/repos/yesme/hctl2-canary/branches/main/protection/required_status_checks"
  },
  "enforce_admins": {
    "enabled": true,
    "url": "https://api.github.com/repos/yesme/hctl2-canary/branches/main/protection/enforce_admins"
  }
}
```

CLI 使用 `--root <本轮私有 control 目录> --json`。实际输入与成功出口如下，`--input JSON` 表示由用例写入的业务输入文件；每条的预览/确认调用可沿入口源文件回溯：

```text
CLI repo register --key canary-register --input JSON; input={"default_source":"github_issues","instance":"github.com","local":{"machine":"control","path":"/var/tmp/hctl2-demo3-codex/hctl-demo3-github-codex-cli-2315729/canary-source"},"name":"GitHub Codex canary","origin":"external","platform":"github","platform_path":"yesme/hctl2-canary","platform_repo_id":"1407796416"}; accepted
CLI project create --key canary-project --input JSON; input={"definition":{"defaults":{},"goal":"deliver tested code through a protected PR","name":"Codex protected main chain","role_members":{},"roles":[],"scope":"yesme/hctl2-canary","settings":{"publish_review_requires_confirmation":false,"selection_policy":{}}},"repo_id":"197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631"}; accepted
CLI profile create --key canary-profile --input JSON; input={"id":"canary-codex","profile":{"environment":[],"harness":{"digest":"18a8dc65f1c2fa485884344356dea1cfd911c6f06cf46fa78e193f4087f4dba7","id":"herdr","revision":"protocol-22"},"max_context_bytes":65536,"mode":"write","model":"none","permissions":["context.read","git.read","git.write"],"required_capabilities":{"event_cursor":false,"exact_attach":false,"input":false,"input_provenance":false,"isolation_effects":[],"managed_single_writer":false,"secure_input":false,"stop":false,"tool_execution_unmediated":false}}}; accepted
CLI project select --key canary-selection --input JSON; input={"project_id":"project-30a26be781b45037c9e9b576c63c6526b9188fc50f09051dbbe46c4996b1194a","project_version":1,"room_id":"main-48913067b61052648eed6fb6df9defa464b7a09c51fab03290180812b90a5dc6","roster_version":null,"selections":[{"agency":{"key":{"id":"native","kind":"agency_binding","scope":{"kind":"control"}},"version":{"state":1}},"budget":{"max_bytes":65536},"display_name":"Codex Canary","optional_skills":[],"permission":{"allow":["context.read","git.read","git.write"]},"persona_tags":[],"profession":{"key":{"id":"native:codex-cli:9a820c17865fa825d04db416818679a9d63bd72e50835c396f496e5684626c9c","kind":"profession","scope":{"kind":"control"}},"version":{"state":1}},"profession_digest":"9a820c17865fa825d04db416818679a9d63bd72e50835c396f496e5684626c9c","required_skills":[],"responsibility":"coding","room_id":"main-48913067b61052648eed6fb6df9defa464b7a09c51fab03290180812b90a5dc6","selected_item":{"key":{"id":"native:codex-cli:9a820c17865fa825d04db416818679a9d63bd72e50835c396f496e5684626c9c","kind":"profession","scope":{"kind":"control"}},"version":{"state":1}},"worker_profiles":[{"key":{"id":"6826d7565319695d1190c6020ca8346c50797b8bccfcb9d58815a9f55ed01239","kind":"worker_profile_revision","scope":{"kind":"control"}},"version":{"state":1}}]}],"topic_command_key":null}; accepted
CLI task connect --key canary-task-source --input JSON; input={"candidate_id":"github_issues","consent":true,"make_default":false,"repo_id":"197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631"}; accepted
CLI task attach --key canary-task-attach --input JSON; input={"approved_scope":"1407796416","consent":true,"project_id":"project-30a26be781b45037c9e9b576c63c6526b9188fc50f09051dbbe46c4996b1194a","project_version":1,"source_id":"b59cbe36dc450cd6bebccbd0a58ed3c55bed1348cf119ec044c721b59089fed9"}; accepted
CLI task create --key canary-task --input JSON; input={"body":"In canary_cases/codex_1791662886576, add calculator.py implementing ceil_div(numerator, denominator) for signed integers using integer arithmetic, raising ValueError on zero denominator. Add test_calculator.py with unittest covering positive, negative, exact and zero cases. This is real non-documentation code for the adopted Task. Run /usr/bin/python3 -B -m unittest discover -s canary_cases/codex_1791662886576 -p test_calculator.py -v and report the command and output. Modify only these two files, leave changes uncommitted for Agency sealing, and do not push, obtain credentials, publish, or change global harness configuration.","project_id":"project-30a26be781b45037c9e9b576c63c6526b9188fc50f09051dbbe46c4996b1194a","project_version":1,"source_id":"b59cbe36dc450cd6bebccbd0a58ed3c55bed1348cf119ec044c721b59089fed9","title":"Codex ceil_div chain 1791662886576"}; accepted
CLI task adopt --key canary-adopt --input JSON; input={"adoption":{"contract":{"acceptance":[{"evidence":{"accept":"integration_receipt","min_channel":"unmediated"},"grade":"mechanical","text":"the Task change is integrated"},{"grade":"human","text":"a human verified the code and tests"}],"capabilities":[],"expected_outcome":"tested integer ceil_div merged through protected main","roles":[],"scope":"canary_cases/codex_1791662886576"},"origin":{"kind":"local","proposal_digest":"636344e580881121d79b3250252f5bb9e0a5319c521b85811c8f25ecf4369867","reference":{"key":{"id":"project-30a26be781b45037c9e9b576c63c6526b9188fc50f09051dbbe46c4996b1194a","kind":"project","scope":{"id":"project-30a26be781b45037c9e9b576c63c6526b9188fc50f09051dbbe46c4996b1194a","kind":"project"}},"version":{"state":1}}}},"project_id":"project-30a26be781b45037c9e9b576c63c6526b9188fc50f09051dbbe46c4996b1194a","project_version":1,"task_id":"bde5516b4bbdae77a63a8e7fe4cb6e6169d9c6f0509af8475ee11d0831a84558","version":3}; accepted
```

Repo 使用 `origin=external / platform=github` 绑定，原生身份回读后已 `active`；不调用只适用于本地新建仓库的 confirm。Task 对应 [Issue #7](https://github.com/yesme/hctl2-canary/issues/7)，采纳的契约机械项接受 `integration_receipt / unmediated`，另有 human 判断项。选择的工种是 Agency catalog 的 `codex-cli`；profile 为 write、权限 `context.read/git.read/git.write`。

写入调用实际命令：

```sh
hctl2 invocation preview --input codex-invocation.json --key canary-write
hctl2 invocation start --input codex-invocation.json --key canary-write --preview-token <不收录>
hctl2 invocation show <project> <invocation>
```

冻结预览的写入边界：

```json
{
  "authorization": "publish_for_review_not_integration",
  "lease": {
    "pending": {
      "baseline_commit": "856d267c76e87daf6dd3549649431122bd36d7a0",
      "binding_version": 1,
      "change_set_id": "cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80",
      "lease": {
        "generation": 1,
        "holder": {
          "invocation_id": "invocation-0c8bbe940cf6d739422693497515792bcd86ecce9675e6d3d413b621eb54399c",
          "invocation_version": 1,
          "kind": "invocation"
        },
        "lease_id": "lease-cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80-1",
        "state": "pending"
      },
      "repo_id": "197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631",
      "version": 1
    },
    "previous": null
  },
  "objective": "In canary_cases/codex_1791662886576, add calculator.py implementing ceil_div(numerator, denominator) for signed integers using integer arithmetic, raising ValueError on zero denominator. Add test_calculator.py with unittest covering positive, negative, exact and zero cases. This is real non-documentation code for the adopted Task. Run /usr/bin/python3 -B -m unittest discover -s canary_cases/codex_1791662886576 -p test_calculator.py -v and report the command and output. Modify only these two files, leave changes uncommitted for Agency sealing, and do not push, obtain credentials, publish, or change global harness configuration.",
  "policy_record": {
    "key": {
      "id": "dispatch-review:invocation-0c8bbe940cf6d739422693497515792bcd86ecce9675e6d3d413b621eb54399c:a95ac931186e6c4ae29b233d421ac377d99a965d318e34ca86c5f08e60d219bd",
      "kind": "review_publish_policy",
      "scope": {
        "id": "197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631",
        "kind": "repo"
      }
    },
    "version": {
      "state": 1
    }
  },
  "publication_target": {
    "allow_update": true,
    "audit_scope": "minimal",
    "binding_version": 1,
    "branch_rule": "hctl2/{change_set}",
    "description_source": "none",
    "repo_id": "197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631",
    "requires_human_confirmation": false,
    "target_branch": "main"
  },
  "repo_local_machine": "control",
  "repo_local_path": "/var/tmp/hctl2-demo3-codex/hctl-demo3-github-codex-cli-2315729/canary-source",
  "review_publish_policy": {
    "digest": "a95ac931186e6c4ae29b233d421ac377d99a965d318e34ca86c5f08e60d219bd",
    "id": "dispatch-review:invocation-0c8bbe940cf6d739422693497515792bcd86ecce9675e6d3d413b621eb54399c:a95ac931186e6c4ae29b233d421ac377d99a965d318e34ca86c5f08e60d219bd",
    "revision": "1"
  }
}
```

调用完成出口是 `completed`，control 的理由为 `Git version admitted; not Task acceptance`；Task 完成仍由最后独立的命令确认。

## Codex 原生记录核正文与测试

Rollout：`/home/jackywang/.codex/sessions/2026/10/11/rollout-2026-10-11T04-08-49-01a1276e-b3ff-7781-bb4c-a4a1e09bf01b.jsonl`。用例从 `session_meta.cwd` 回读执行目录，并核它位于 `hctl2-tool worktree materialize` 建出的 ChangeSet 副本；common dir 指回注册的本地 Repo，源 Repo HEAD 仍为冻结基线。工作树位于 Agency 凭据根之外。

用冻结 Bundle 的每条交付字节拼出正文，再加 3f 的 Agency 边界；与原生 `response_item` 的 user `input_text` 逐字相等。正文 UTF-8 为 6936 bytes，SHA-256 `f7d97a69336ffd71291055482fcff9ae60ec31eb0c6dda41aacdbc744c6c572e`。原生记录中的正文原样如下：

```text
In canary_cases/codex_1791662886576, add calculator.py implementing ceil_div(numerator, denominator) for signed integers using integer arithmetic, raising ValueError on zero denominator. Add test_calculator.py with unittest covering positive, negative, exact and zero cases. This is real non-documentation code for the adopted Task. Run /usr/bin/python3 -B -m unittest discover -s canary_cases/codex_1791662886576 -p test_calculator.py -v and report the command and output. Modify only these two files, leave changes uncommitted for Agency sealing, and do not push, obtain credentials, publish, or change global harness configuration.
{"authorization":"publish_for_review_not_integration","lease":{"pending":{"baseline_commit":"856d267c76e87daf6dd3549649431122bd36d7a0","binding_version":1,"change_set_id":"cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80","lease":{"generation":1,"holder":{"invocation_id":"invocation-0c8bbe940cf6d739422693497515792bcd86ecce9675e6d3d413b621eb54399c","invocation_version":1,"kind":"invocation"},"lease_id":"lease-cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80-1","state":"pending"},"repo_id":"197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631","version":1},"previous":null},"objective":"In canary_cases/codex_1791662886576, add calculator.py implementing ceil_div(numerator, denominator) for signed integers using integer arithmetic, raising ValueError on zero denominator. Add test_calculator.py with unittest covering positive, negative, exact and zero cases. This is real non-documentation code for the adopted Task. Run /usr/bin/python3 -B -m unittest discover -s canary_cases/codex_1791662886576 -p test_calculator.py -v and report the command and output. Modify only these two files, leave changes uncommitted for Agency sealing, and do not push, obtain credentials, publish, or change global harness configuration.","policy_record":{"key":{"id":"dispatch-review:invocation-0c8bbe940cf6d739422693497515792bcd86ecce9675e6d3d413b621eb54399c:a95ac931186e6c4ae29b233d421ac377d99a965d318e34ca86c5f08e60d219bd","kind":"review_publish_policy","scope":{"id":"197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631","kind":"repo"}},"version":{"state":1}},"publication_target":{"allow_update":true,"audit_scope":"minimal","binding_version":1,"branch_rule":"hctl2/{change_set}","description_source":"none","repo_id":"197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631","requires_human_confirmation":false,"target_branch":"main"},"repo_local_machine":"control","repo_local_path":"/var/tmp/hctl2-demo3-codex/hctl-demo3-github-codex-cli-2315729/canary-source","review_publish_policy":{"digest":"a95ac931186e6c4ae29b233d421ac377d99a965d318e34ca86c5f08e60d219bd","id":"dispatch-review:invocation-0c8bbe940cf6d739422693497515792bcd86ecce9675e6d3d413b621eb54399c:a95ac931186e6c4ae29b233d421ac377d99a965d318e34ca86c5f08e60d219bd","revision":"1"}}
[]
{"current":true,"events":[{"content":{"name":"Codex protected main chain"},"event_id":"$b-Pl74t_FtMddKxvNWWFNUBF8tSJkYOQ6b-GPZao4Qw","origin_server_ts":1791662797035,"room_id":"!Wdup25vOyqXP5JpJxC:hctl2.localhost","sender":"@hctl2_control:hctl2.localhost","state_key":"","type":"m.room.name","unsigned":{"age":127470}},{"content":{"command":"2ecd6be7fd21f8e732388dfe9fe1bd4006849e839bcd1e7a8d1e691a137c07e1","project":"project-30a26be781b45037c9e9b576c63c6526b9188fc50f09051dbbe46c4996b1194a","room":"main-48913067b61052648eed6fb6df9defa464b7a09c51fab03290180812b90a5dc6"},"event_id":"$tVIOnrTKeeCwI6riAq9SC3f5hg_RbJyQ7WchUZavR4U","origin_server_ts":1791662797034,"room_id":"!Wdup25vOyqXP5JpJxC:hctl2.localhost","sender":"@hctl2_control:hctl2.localhost","state_key":"","type":"io.hctl2.creation","unsigned":{"age":127471}},{"content":{"guest_access":"can_join"},"event_id":"$agz9wxbjZmiZiGZDYdAH3EOKq5dOl4gR3E5MZ0hkaDo","origin_server_ts":1791662797033,"room_id":"!Wdup25vOyqXP5JpJxC:hctl2.localhost","sender":"@hctl2_control:hctl2.localhost","state_key":"","type":"m.room.guest_access","unsigned":{"age":127472}},{"content":{"history_visibility":"shared"},"event_id":"$z7mtMDNitr5lM9pKAb5P3pN8Yxh--AL_8L7Irp3ZHrU","origin_server_ts":1791662797033,"room_id":"!Wdup25vOyqXP5JpJxC:hctl2.localhost","sender":"@hctl2_control:hctl2.localhost","state_key":"","type":"m.room.history_visibility","unsigned":{"age":127472}},{"content":{"join_rule":"invite"},"event_id":"$LI2j3HFf_c4HlONLsxNtsByJd2VuAkrkKYsBsXhYEfU","origin_server_ts":1791662797032,"room_id":"!Wdup25vOyqXP5JpJxC:hctl2.localhost","sender":"@hctl2_control:hctl2.localhost","state_key":"","type":"m.room.join_rules","unsigned":{"age":127473}},{"content":{"alias":"#hctl2_2ecd6be7fd21f8e732388dfe9fe1bd4006849e839bcd1e7a8d1e691a137c07e1:hctl2.localhost"},"event_id":"$colGIvRofrr08PsZrrUiTodCjQUf6C1j8RnoL7rVJ0k","origin_server_ts":1791662797031,"room_id":"!Wdup25vOyqXP5JpJxC:hctl2.localhost","sender":"@hctl2_control:hctl2.localhost","state_key":"","type":"m.room.canonical_alias","unsigned":{"age":127474}},{"content":{"ban":50,"events":{"m.poll.response":0,"m.room.encryption":100,"m.room.history_visibility":100,"m.room.power_levels":100,"m.room.server_acl":100,"m.room.tombstone":100,"org.matrix.msc3381.poll.response":0},"events_default":0,"invite":0,"kick":50,"notifications":{"room":50},"redact":50,"state_default":50,"users":{"@hctl2_control:hctl2.localhost":100},"users_default":0},"event_id":"$TmSDi5vC6fmKvvFElQj4q0EWJXJgjDuQinKucuRlY6Q","origin_server_ts":1791662797030,"room_id":"!Wdup25vOyqXP5JpJxC:hctl2.localhost","sender":"@hctl2_control:hctl2.localhost","state_key":"","type":"m.room.power_levels","unsigned":{"age":127475}},{"content":{"membership":"join"},"event_id":"$hluc5ZtVsfTRrlHRxZ5-A6zPC8FJk1YCqrqQamWz8e4","origin_server_ts":1791662797030,"room_id":"!Wdup25vOyqXP5JpJxC:hctl2.localhost","sender":"@hctl2_control:hctl2.localhost","state_key":"@hctl2_control:hctl2.localhost","type":"m.room.member","unsigned":{"age":127475}},{"content":{"room_version":"11"},"event_id":"$UQJv4TR4brfucGi1116d1eOJ9RWqTNbglIcsE6wu5xk","origin_server_ts":1791662797029,"room_id":"!Wdup25vOyqXP5JpJxC:hctl2.localhost","sender":"@hctl2_control:hctl2.localhost","state_key":"","type":"m.room.create","unsigned":{"age":127476}}],"next":"58","start":"9223372036854775807"}

Agency ChangeSet: cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80
Baseline: 856d267c76e87daf6dd3549649431122bd36d7a0
Write lease: lease-cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80-1 generation 1
Agency execution directory: /tmp/hctl2-exec-a7798d1cb8d68e707fb5/write-worktrees/e742781eb22a263bf5b8bfb3001ae8d130a04b27fc6ca78f64fe52765a160edc/cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80
Edit and test this ChangeSet in this materialized worktree. Leave changes uncommitted for Agency sealing. Do not push, publish reviews, obtain Git credentials, or change harness global configuration.
```

原生 `CommandExecution`，并非模型声称跑过：

```json
{
  "command": [
    "/usr/bin/zsh",
    "-lc",
    "/usr/bin/python3 -B -m unittest discover -s canary_cases/codex_1791662886576 -p test_calculator.py -v"
  ],
  "cwd": "file:///tmp/hctl2-exec-a7798d1cb8d68e707fb5/write-worktrees/e742781eb22a263bf5b8bfb3001ae8d130a04b27fc6ca78f64fe52765a160edc/cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80",
  "exit_code": 0,
  "status": "completed",
  "aggregated_output": "test_exact (test_calculator.CeilDivTests.test_exact) ... ok\ntest_large_integers (test_calculator.CeilDivTests.test_large_integers) ... ok\ntest_negative (test_calculator.CeilDivTests.test_negative) ... ok\ntest_positive (test_calculator.CeilDivTests.test_positive) ... ok\ntest_zero_denominator (test_calculator.CeilDivTests.test_zero_denominator) ... ok\ntest_zero_numerator (test_calculator.CeilDivTests.test_zero_numerator) ... ok\n\n----------------------------------------------------------------------\nRan 6 tests in 0.001s\n\nOK\n"
}
```

Codex 的 `git status` 被 3f Git 目录隔离拒绝，heredoc 临时文件写入也被拒；原生记录保留这些失败，随后改用允许的命令继续，未改全局配置。

作者在同一工作树独立复跑 unittest，并另核 7 个有符号/整除/零分母例子：

```text
INDEPENDENT python3 unittest:
test_exact (test_calculator.CeilDivTests.test_exact) ... ok
test_large_integers (test_calculator.CeilDivTests.test_large_integers) ... ok
test_negative (test_calculator.CeilDivTests.test_negative) ... ok
test_positive (test_calculator.CeilDivTests.test_positive) ... ok
test_zero_denominator (test_calculator.CeilDivTests.test_zero_denominator) ... ok
test_zero_numerator (test_calculator.CeilDivTests.test_zero_numerator) ... ok

----------------------------------------------------------------------
Ran 6 tests in 0.000s

OK

7 independent ceil_div cases OK
```

## 封存、准入与真实发布

Proposal 的 commit 树、准入 Revision 的 result_tree、独立公开 `hctl2-tool repo seal` 回读、control 发布候选的树逐项相同。独立回读命令：

```sh
hctl2-tool repo seal --path <rollout cwd> --change-set-ref cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80 --baseline 856d267c76e87daf6dd3549649431122bd36d7a0 --key demo3-independent-seal
```

```json
{
  "association_ref": "refs/hctl2/changesets/cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80/seals/856d267c76e87daf6dd3549649431122bd36d7a0/key-3335dbada7f8225d5a3cb9d1ec1fa4218a28877cf5d7f050bef92b67acbed251",
  "base_commit_sha": "856d267c76e87daf6dd3549649431122bd36d7a0",
  "base_tree_sha": "32f8eece06518ebe2aab763086d288ae51bb66a9",
  "change_set_id": "cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80",
  "error": null,
  "evidence_level": "unmediated",
  "git": {
    "path": "/usr/bin/git",
    "version": "2.53.0"
  },
  "observed_at_unix_ms": 1791663004769,
  "outcome": "established",
  "repo_path": "/tmp/hctl2-exec-a7798d1cb8d68e707fb5/write-worktrees/e742781eb22a263bf5b8bfb3001ae8d130a04b27fc6ca78f64fe52765a160edc/cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80",
  "result_commit_sha": "ad8f53efa24b89989e53ee04271e4cf94abc06e2",
  "result_tree_sha": "52d824a03fdcd8b3be4f6c2d8c82cb710732a2b4",
  "retained_ref": "refs/hctl2/changesets/cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80/seals/856d267c76e87daf6dd3549649431122bd36d7a0/52d824a03fdcd8b3be4f6c2d8c82cb710732a2b4/ad8f53efa24b89989e53ee04271e4cf94abc06e2",
  "reused": false,
  "schema": "hctl2.git-seal.v1"
}
```

```json
{
  "base_commit_sha": "856d267c76e87daf6dd3549649431122bd36d7a0",
  "change_set_id": "cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80",
  "change_set_revision_id": "csr-5fa60a1b6301785c7e7f60896ff778302c9e5221d014ec3f358d6bf67c04f769",
  "parent_revision_id": null,
  "producer_ref": {
    "invocation_id": "invocation-0c8bbe940cf6d739422693497515792bcd86ecce9675e6d3d413b621eb54399c",
    "invocation_version": 1,
    "kind": "invocation"
  },
  "result_tree_sha": "52d824a03fdcd8b3be4f6c2d8c82cb710732a2b4",
  "review_subject_digest": "e6350003eb5f0edc219c079da280672054840c1950077425bcfa986889467dc5",
  "revision_digest": "a7d0ac71505b9d61fc8febdea11758ff1a5fd0a28788aec46a13a40e4dca614e"
}
```

真实非文档代码的 `changeset diff`（只增两份 Python 文件）：

```diff
diff --git a/canary_cases/codex_1791662886576/calculator.py b/canary_cases/codex_1791662886576/calculator.py
new file mode 100644
index 0000000..cd06de7
--- /dev/null
+++ b/canary_cases/codex_1791662886576/calculator.py
@@ -0,0 +1,5 @@
+def ceil_div(numerator: int, denominator: int) -> int:
+    """Return the ceiling of a signed integer quotient using integer arithmetic."""
+    if denominator == 0:
+        raise ValueError("denominator must not be zero")
+    return -(-numerator // denominator)
diff --git a/canary_cases/codex_1791662886576/test_calculator.py b/canary_cases/codex_1791662886576/test_calculator.py
new file mode 100644
index 0000000..5f4938c
--- /dev/null
+++ b/canary_cases/codex_1791662886576/test_calculator.py
@@ -0,0 +1,44 @@
+import unittest
+
+from calculator import ceil_div
+
+
+class CeilDivTests(unittest.TestCase):
+    def test_positive(self):
+        for numerator, denominator, expected in [(7, 3, 3), (1, 2, 1)]:
+            with self.subTest(numerator=numerator, denominator=denominator):
+                self.assertEqual(ceil_div(numerator, denominator), expected)
+
+    def test_negative(self):
+        cases = [(-7, 3, -2), (7, -3, -2), (-7, -3, 3), (-1, 2, 0), (1, -2, 0)]
+        for numerator, denominator, expected in cases:
+            with self.subTest(numerator=numerator, denominator=denominator):
+                self.assertEqual(ceil_div(numerator, denominator), expected)
+
+    def test_exact(self):
+        cases = [(6, 3, 2), (-6, 3, -2), (6, -3, -2), (-6, -3, 2)]
+        for numerator, denominator, expected in cases:
+            with self.subTest(numerator=numerator, denominator=denominator):
+                self.assertEqual(ceil_div(numerator, denominator), expected)
+
+    def test_zero_numerator(self):
+        for denominator in (3, -3):
+            with self.subTest(denominator=denominator):
+                self.assertEqual(ceil_div(0, denominator), 0)
+
+    def test_zero_denominator(self):
+        for numerator in (7, -7, 0):
+            with self.subTest(numerator=numerator):
+                with self.assertRaises(ValueError):
+                    ceil_div(numerator, 0)
+
+    def test_large_integers(self):
+        numerator = 2**100 + 1
+        self.assertEqual(ceil_div(numerator, 2), 2**99 + 1)
+        self.assertEqual(ceil_div(-numerator, 2), -(2**99))
+        self.assertEqual(ceil_div(numerator, -2), -(2**99))
+        self.assertEqual(ceil_div(-numerator, -2), 2**99 + 1)
+
+
+if __name__ == "__main__":
+    unittest.main()
```

`review show` 的 push 与 review_request 两段都 confirmed；映射由发布 worker 回读真实平台写入：

```json
{
  "intent_id": "rp-7b1f58e324d2e9393cc4cebcc64b04431bc1630668addc05ba6c9924cd25eb37",
  "state": "published",
  "stages": {
    "push": {
      "commit_sha": "ad8f53efa24b89989e53ee04271e4cf94abc06e2",
      "confirmed": true,
      "dispatched": false
    },
    "review_request": {
      "commit_sha": "ad8f53efa24b89989e53ee04271e4cf94abc06e2",
      "confirmed": true,
      "dispatched": false,
      "index": 8
    }
  },
  "mappings": [
    {
      "change_set_id": "cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80",
      "change_set_revision_id": "csr-5fa60a1b6301785c7e7f60896ff778302c9e5221d014ec3f358d6bf67c04f769",
      "intent_id": "rp-7b1f58e324d2e9393cc4cebcc64b04431bc1630668addc05ba6c9924cd25eb37",
      "platform_commit_sha": "ad8f53efa24b89989e53ee04271e4cf94abc06e2",
      "recorded_at_unix_ms": 1791663012774,
      "review_request": {
        "base_branch": "main",
        "head_branch": "hctl2/cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80",
        "index": 8
      },
      "round": 1
    }
  ]
}
```

推送使用既有 `gh auth git-credential` 原生助手；harness 没拿 Git 凭据，也没推送。发布结果 [PR #8](https://github.com/yesme/hctl2-canary/pull/8)，候选 `ad8f53efa24b89989e53ee04271e4cf94abc06e2`。

必需的 canary 实际完成并成功，后面才提交 integration：

```json
[
  {
    "name": "canary",
    "status": "completed",
    "conclusion": "success",
    "head_sha": "ad8f53efa24b89989e53ee04271e4cf94abc06e2",
    "started_at": "2026-10-10T20:10:16Z",
    "completed_at": "2026-10-10T20:10:20Z",
    "html_url": "https://github.com/yesme/hctl2-canary/actions/runs/38082681474/job/114302661493"
  }
]
```

## 保护合并、Git Receipt 与 Task 完成

`integration preview` 试 `expected_head`，失败出口如下；改为 `accept_advance`，预览包含 requires_review_request、canary 与 enforce_admins；提交仍只由 integration 发平台合并请求：

```json
{
  "error": {
    "code": "EXPECTED_HEAD_UNSUPPORTED",
    "message": "this platform binding cannot guarantee the expected target head; choose accept_advance explicitly",
    "recovery_action": "choose_accept_advance_form"
  }
}
```

```sh
hctl2 integration preview --input canary-integration.json --key canary-merge
hctl2 integration submit --input canary-integration.json --key canary-merge --preview-token <不收录>
hctl2 integration show <repo> <intent>
```

```json
{
  "receipt_id": "receipt-b3fd3bb7ad875250cdce42be10cd2bf00e892e81a96d8cf72a9dd2a05bf8eb7c",
  "intent_id": "integration-b3fd3bb7ad875250cdce42be10cd2bf00e892e81a96d8cf72a9dd2a05bf8eb7c",
  "form": "accept_advance",
  "strategy": "merge_commit",
  "source": {
    "base_commit_sha": "856d267c76e87daf6dd3549649431122bd36d7a0",
    "change_set_id": "cs-9531852b961047385d441ac577947f56af041d30af3cb623f7cabd2a8f28ab80",
    "change_set_revision_id": "csr-5fa60a1b6301785c7e7f60896ff778302c9e5221d014ec3f358d6bf67c04f769",
    "parent_revision_id": null,
    "producer_ref": {
      "invocation_id": "invocation-0c8bbe940cf6d739422693497515792bcd86ecce9675e6d3d413b621eb54399c",
      "invocation_version": 1,
      "kind": "invocation"
    },
    "result_tree_sha": "52d824a03fdcd8b3be4f6c2d8c82cb710732a2b4",
    "review_subject_digest": "e6350003eb5f0edc219c079da280672054840c1950077425bcfa986889467dc5"
  },
  "target_head_before": "856d267c76e87daf6dd3549649431122bd36d7a0",
  "target_head_after": "2cac2a9b83a5b033bfc5c07d325c490c6ee135b8",
  "integrated_commit": "2cac2a9b83a5b033bfc5c07d325c490c6ee135b8",
  "integrated_tree": "52d824a03fdcd8b3be4f6c2d8c82cb710732a2b4",
  "evidence_level": "hctl2-tool",
  "readback": {
    "git": {
      "commit": "2cac2a9b83a5b033bfc5c07d325c490c6ee135b8",
      "commit_parents": [
        "856d267c76e87daf6dd3549649431122bd36d7a0",
        "ad8f53efa24b89989e53ee04271e4cf94abc06e2"
      ],
      "commit_present": true,
      "commit_tree": "52d824a03fdcd8b3be4f6c2d8c82cb710732a2b4",
      "contains": true,
      "evidence_level": "unmediated",
      "git": {
        "path": "/usr/bin/git",
        "version": "2.53.0"
      },
      "head": "2cac2a9b83a5b033bfc5c07d325c490c6ee135b8",
      "head_tree": "52d824a03fdcd8b3be4f6c2d8c82cb710732a2b4",
      "observed_at_unix_ms": 1791663046358,
      "operation": "readback",
      "ref": "refs/heads/main",
      "remote": "https://github.com/yesme/hctl2-canary.git",
      "remote_head": "2cac2a9b83a5b033bfc5c07d325c490c6ee135b8",
      "repository": "/var/tmp/hctl2-demo3-codex/hctl-demo3-github-codex-cli-2315729/integration/readback/197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631.git",
      "schema": "hctl2.readback.v1"
    },
    "platform_head": "2cac2a9b83a5b033bfc5c07d325c490c6ee135b8",
    "status": "applied",
    "review_request": {
      "number": 8,
      "html_url": "https://github.com/yesme/hctl2-canary/pull/8",
      "merged": true,
      "head": {
        "sha": "ad8f53efa24b89989e53ee04271e4cf94abc06e2"
      },
      "merge_commit_sha": "2cac2a9b83a5b033bfc5c07d325c490c6ee135b8"
    }
  }
}
```

平台 [合并提交 2cac2a9](https://github.com/yesme/hctl2-canary/commit/2cac2a9b83a5b033bfc5c07d325c490c6ee135b8) 与 Receipt integrated_commit / target_head_after、当时的 main 都相等；Git readback.contains=true，合并提交两个父里含候选 ad8f53e，合并提交不是候选本身。

```text
CLI task complete --key canary-complete --input JSON; input={"acceptance":[{"channel":"unmediated","generation":1,"item":0,"judge":{"kind":"hctl2_tool"},"producer":"hctl2-tool","references":[{"key":{"id":"receipt-b3fd3bb7ad875250cdce42be10cd2bf00e892e81a96d8cf72a9dd2a05bf8eb7c","kind":"integration_receipt","scope":{"id":"197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631","kind":"repo"}},"version":{"state":1}}]},{"channel":"narrated","generation":null,"item":1,"judge":{"actor":"local-owner:1000","kind":"human"},"producer":null,"references":[]}],"lifecycle_version":1,"project_id":"project-30a26be781b45037c9e9b576c63c6526b9188fc50f09051dbbe46c4996b1194a","revision_number":1,"task_id":"bde5516b4bbdae77a63a8e7fe4cb6e6169d9c6f0509af8475ee11d0831a84558","version":4}; accepted
```

```json
{
  "task_id": "bde5516b4bbdae77a63a8e7fe4cb6e6169d9c6f0509af8475ee11d0831a84558",
  "lifecycle": "completed",
  "lifecycle_version": 2,
  "receipt_id": "bde5516b4bbdae77a63a8e7fe4cb6e6169d9c6f0509af8475ee11d0831a84558:2",
  "items": [
    {
      "generation": 1,
      "grade": "mechanical",
      "item": 0,
      "judge": {
        "kind": "hctl2_tool"
      },
      "outcome": "passed",
      "producer": "hctl2-tool",
      "references": [
        {
          "key": {
            "id": "receipt-b3fd3bb7ad875250cdce42be10cd2bf00e892e81a96d8cf72a9dd2a05bf8eb7c",
            "kind": "integration_receipt",
            "scope": {
              "id": "197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631",
              "kind": "repo"
            }
          },
          "version": {
            "state": 1
          }
        }
      ],
      "source_snapshot": {
        "key": {
          "id": "b59cbe36dc450cd6bebccbd0a58ed3c55bed1348cf119ec044c721b59089fed9",
          "kind": "task_snapshot",
          "scope": {
            "id": "197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631",
            "kind": "repo"
          }
        },
        "version": {
          "state": 2
        }
      },
      "text": "the Task change is integrated",
      "text_digest": "2eb9e88e4a9a4d4b94c53c5cb82de69dd7b7d0ccd50538dd5a464202813e046e",
      "validation_level": "unmediated"
    },
    {
      "grade": "human",
      "item": 1,
      "judge": {
        "actor": "local-owner:1000",
        "kind": "human"
      },
      "outcome": "passed",
      "references": [],
      "source_snapshot": {
        "key": {
          "id": "b59cbe36dc450cd6bebccbd0a58ed3c55bed1348cf119ec044c721b59089fed9",
          "kind": "task_snapshot",
          "scope": {
            "id": "197e51087d5181b030ce3e876b3746426b5449380ff99440a56a8e6a557f3631",
            "kind": "repo"
          }
        },
        "version": {
          "state": 2
        }
      },
      "text": "a human verified the code and tests",
      "text_digest": "680ada8ef4f77ca999620138e2b836eee79436bfe85a7c23823aaf1b461be1bc",
      "validation_level": "narrated"
    }
  ]
}
```

`task show` 回读 `lifecycle=completed`；Task Completion Receipt 的机械项是 `unmediated`，引用上面真实 integration_receipt，human 项保留 `narrated`。源 Git 仓库与 control 私有临时目录在用例退出时清理；原生 rollout 与平台 PR / commit 保留，树一致性是执行时独立工具断言的结果。

## 回归与退回修正

命令与实际输出：

```sh
./src/buck2 test root//packaging/release:room-cli-test root//apps/cli:task_cli_test \
  root//apps/control:unit_test root//apps/control:task_native_test \
  root//packaging/release:room-cli-clippy-clean-test -- \
  --env TMPDIR=/var/tmp/hctl2-demo3-codex
```

```text
Build ID: ec2e5ee6-f8e6-4df2-8ddd-2b40f896875f
✓ Pass: root//apps/cli:task_cli_test (6.2s)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.24s
✓ Pass: root//packaging/release:room-cli-clippy-clean-test (0.1s)
✓ Pass: root//apps/control:unit_test (9.8s)
test result: ok. 65 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 9.79s
✓ Pass: root//apps/control:task_native_test (37.6s)
test result: ok. 66 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 37.57s
✓ Pass: root//packaging/release:room-cli-test (1:44.8s)
test github::demo3_github_codex_real_cli_reaches_protected_main_and_task_completion ... ignored, UNVERIFIED: native Codex/Claude installations, Codex login and gh push/admin login to yesme/hctl2-canary; HCTL2_HARNESS_LIVE=1 HCTL2_GITHUB_LIVE=1
test result: ok. 11 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 104.76s
Tests finished: Pass 5. Fail 0
```

```sh
./src/buck2 build root//apps/control:clippy root//apps/cli:clippy --show-output
```

```text
Build ID: 5ebe78c6-99bd-47e8-a760-32570e7c2193
BUILD SUCCEEDED
control: 7 reports, nonempty 0; CLI: 3 reports, nonempty 0
```

只把 `issue_filter` 的 GitHub 分支从空参数退回 `&type=issues`，其余不动；真实 CLI 的 gh fixture 按本次平台事实拒绝旧参数：

```sh
./src/buck2 test root//apps/cli:task_cli_test -- \
  --env TMPDIR=/var/tmp/hctl2-demo3-codex \
  --test-arg=task_commands_cross_live_daemon_preview_replay_and_restart
```

```text
mutation-red.log
Build ID: 4a99d31a-01b2-442a-aed1-42a1bf07ff09
thread 'task_commands_cross_live_daemon_preview_replay_and_restart' (2612201) panicked at src/apps/cli/tests/task.rs:166:5:
preview attach: {"error":{"code":"PROVIDER_UNAVAILABLE","message":"GitHub GET failed (HTTP Some(422))","recovery_action":"read_back_original_intent"}}
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.20s
```

```text
mutation-restored.log
Build ID: 52ddf565-c060-4dbc-814e-432ce0dd36e7
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 1.03s
```

恢复精确原文件后同一用例 1 passed / 0 failed；退回版本不提交。现有 GitHub fixture 拒绝旧查询，原生 Gitea 66 条回归通过，Gitea 保留原参数。

## 前几次没有通过的尝试

原始运行日志留在作者本机 `buck-out/pkg9-github-evidence/`；它们不是 green 证据。

- live-1：TMPDIR 放工作区深目录，control.sock 120 bytes 超过 Unix socket 路径上限，control 启动失败。改短目录。
- live-2：外部 Repo 注册后已 active，再套本地新建 Repo 的 confirm 得 REGISTRATION_TERMINAL。删掉错误测试步骤。
- live-3：真实 Task attach HTTP 422（type=issues），修 provider 后原生 GitHub/Gitea 分别查询；调研与原生 API 复核见 [GitHub](../../docs/research/sdk/github.md)。
- live-4：创建 Issue #4 后 list 尚未读到，Task.create 为 RESULT_UNKNOWN。用例增加原关联键重投回读，未知不算成功，不换键重发；Issue #4 保留为失败尝试，不算已完成 Task。
- live-5：真实链已合 [PR #6](https://github.com/yesme/hctl2-canary/pull/6)，merge 856d267c76e87daf6dd3549649431122bd36d7a0，Task 完成。最后测试错误地断言 completed.result.items 而实际是 completed.items，因此 0 passed / 1 failed；修测试层级后从零跑 live-6，本记录以它的 PR #8 和 Receipt 为最终通过证据。

未核验：本 PR 不承担第 9 包第 3–9 条、私有 GitHub 回读、规则集保护、返工多版本父链与跨服务重启。没有调整保护、全局 harness 配置或声明新的加固能力。

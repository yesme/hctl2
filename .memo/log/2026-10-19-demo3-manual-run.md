# 演示 3「真实交付」操作手册从零实跑原始输出（2026-10-19）

- **基线提交**：`0566e83`（`origin/main`）
- **机器环境**：Ubuntu `Linux 7.0.0-34-generic x86_64`，Claude Code `2.1.303`
- **完整包构建**：`./buck2 build --config hctl2.zstd_preset=fast root//packaging/release:complete`（Buck2 Build ID `b74d800c-1f90-4034-9b92-c41c04f838b2`）
- **覆盖范围**：`docs/demos/demo-3.md` 第一节（离线安装与启动控制面）、第二节（A 链全部 10 步）、第四节（C 链全部 5 步，含发布、合入、完成三次人工预览）、第五节（启停恢复、凭据隔离与逃生口）、第六节（精确清理本次执行目录与工作区）。

以下为实跑脚本原样捕获的标准输出（未经任何手工删改或重打）：

```text
=== 一.2 Offline install ===
hctl2: installed hctl2-0.0.0-linux-x86_64 at /var/tmp/d3/prefix/lib/hctl2/hctl2-0.0.0-linux-x86_64
hctl2: start control and consumed services with /var/tmp/d3/prefix/bin/hctl2 start
hctl2: or start every bundled component with /var/tmp/d3/prefix/bin/hctl2-services start
=== 一.3 Start control plane ===
{
  "actor": {
    "authority": null,
    "permission_scope": [
      {
        "kind": "control"
      }
    ],
    "principal": "local-owner:1000",
    "source": "direct_client"
  },
  "control_id": "7f49fdff03a25c42f80ec625e6d814e2",
  "endpoint": "unix",
  "policy": {
    "client_least_privilege": true,
    "credential_storage": "user-file",
    "endpoint_and_connection": "loopback-unix-owner-only",
    "non_local_transport": "not_offered"
  },
  "protocol": "hctl2.control.v1",
  "ready": true,
  "services": {
    "backend": "packaged",
    "event_seq": 0,
    "hosted": [
      {
        "available": true,
        "consumed": true,
        "name": "tuwunel",
        "pid": 758731,
        "ready": true,
        "running": true
      },
      {
        "available": false,
        "consumed": false,
        "name": "gitea",
        "pid": null,
        "ready": false,
        "running": false
      }
    ],
    "last_error": null,
    "observed_at": "1792417862",
    "source": "process-compose:/tmp/hctl2-process-compose-1000/14af61e7612cb8ca.sock"
  },
  "writer_generation": 1
}
{"actor":{"authority":null,"permission_scope":[{"kind":"control"}],"principal":"local-owner:1000","source":"direct_client"},"control_id":"7f49fdff03a25c42f80ec625e6d814e2","endpoint":"unix","policy":{"client_least_privilege":true,"credential_storage":"user-file","endpoint_and_connection":"loopback-unix-owner-only","non_local_transport":"not_offered"},"protocol":"hctl2.control.v1","ready":true,"services":{"backend":"packaged","event_seq":0,"hosted":[{"available":true,"consumed":true,"name":"tuwunel","pid":758731,"ready":true,"running":true},{"available":false,"consumed":false,"name":"gitea","pid":null,"ready":false,"running":false}],"last_error":null,"observed_at":"1792417864","source":"process-compose:/tmp/hctl2-process-compose-1000/14af61e7612cb8ca.sock"},"writer_generation":1}
=== 二.1 Init local repo ===
BASELINE_SHA=65db47301586e07555c48620105f8ae62296f68d
=== 二.2 Register and confirm local Repo ===
{
  "repo_id": "bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3",
  "lifecycle": "pending",
  "version": 3,
  "observed": {
    "account_id": "1",
    "can_write_issues": true,
    "clone_url": "http://127.0.0.1:3001/hctl-7f49fdff03a25c42/demo3-walkthrough.git",
    "credential_ref": "gitea:7f49fdff03a25c42f80ec625e6d814e2:admin",
    "full_name": "hctl-7f49fdff03a25c42/demo3-walkthrough",
    "has_issues": true,
    "instance": "http://127.0.0.1:3001",
    "stable_id": "1"
  }
}
{"abandoned":false,"actor":{"authority":null,"permission_scope":[{"kind":"control"},{"id":"bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3","kind":"repo"}],"principal":"local-owner:1000","source":"direct_client"},"command_id":"repo.register:a-register","config":{"byte_digest":"a638db411f799cdf7ed3ffac6cdf2a933a56857654d334157dd4cff385d1a9aa","control_id":"7f49fdff03a25c42f80ec625e6d814e2","material_id":"ee108a150c7da85ee01a2d2937dae984d80aa86e05e2d1355e98e479cab678b5","scope":{"id":"bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3","kind":"repo"}},"delivered":true,"idempotency_key":"a-register","lifecycle":"active","observed":{"account_id":"1","can_write_issues":true,"clone_url":"http://127.0.0.1:3001/hctl-7f49fdff03a25c42/demo3-walkthrough.git","credential_ref":"gitea:7f49fdff03a25c42f80ec625e6d814e2:admin","full_name":"hctl-7f49fdff03a25c42/demo3-walkthrough","has_issues":true,"instance":"http://127.0.0.1:3001","stable_id":"1"},"prepared":{"evidence_conflicts":[],"local":{"governance_paths":[],"head_branch":"refs/heads/main","path":"/var/tmp/d3/write-site","refs":{"refs/heads/main":"65db47301586e07555c48620105f8ae62296f68d"},"remotes":{}},"platform":"local","request":{"default_source":"gitea_issues","instance":null,"local":{"extra_refs":[],"in_place":false,"machine":"control","path":"/var/tmp/d3/write-site","publish_governance":false},"name":"demo3-walkthrough","origin":"local","platform":"local","platform_path":"demo3-walkthrough","platform_repo_id":null,"remote_evidence":null},"sources":[{"actual_source_and_scope":"demo3-walkthrough","available":true,"can_claim":true,"create":true,"field_writeback":true,"id":"gitea_issues","provider":"gitea_issues","recommended":true}]},"repo_id":"bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3","residual":null,"residual_target":null,"sources":[{"actual_source_and_scope":"http://127.0.0.1:3001/1","available":true,"can_claim":true,"create":true,"field_writeback":true,"id":"gitea_issues","provider":"gitea_issues","recommended":true}],"version":4}
=== 二.3 Pair Agency, accept claude-code, create write Profile ===
{
  "paired": true,
  "professions": [
    {
      "digest": "375988f457756524234695cf2d8f64ace99c5f68682e47ea24862a843c191450",
      "id": "claude-code",
      "revision": "2.1.303 (Claude Code)"
    },
    {
      "digest": "399ff69c431c066e40de318fba45c2b07f45a76b19939d247ee4828fc70c4215",
      "id": "codex-cli",
      "revision": "codex-cli 0.163.0"
    }
  ]
}
{"profession":{"data":{"type":"value","value":{"capabilities":{"event_cursor":false,"exact_attach":false,"input":false,"input_provenance":false,"isolation_effects":[],"managed_single_writer":false,"secure_input":false,"stop":true,"tool_execution_unmediated":false},"default_role":"worker","harness":{"digest":"18a8dc65f1c2fa485884344356dea1cfd911c6f06cf46fa78e193f4087f4dba7","id":"herdr","revision":"protocol-22"},"model":"none","persona":"claude code","reference":{"digest":"375988f457756524234695cf2d8f64ace99c5f68682e47ea24862a843c191450","id":"claude-code","revision":"2.1.303 (Claude Code)"},"skills":[],"terms":"Bundle dispatch in a selection-local native Claude Herdr session; read-only or leased ChangeSet worktree; write return is a tool-sealed ChangeSet output; no terminal input or publication"}},"key":{"id":"native:claude-code:375988f457756524234695cf2d8f64ace99c5f68682e47ea24862a843c191450","kind":"profession","scope":{"kind":"control"}},"materials":[],"revision_digest":"b2c2ca422d18f9538df5369a2e5d6449ef95f7d8e0000f4286513694e0262f31","sources":[{"key":{"id":"native","kind":"agency_binding","scope":{"kind":"control"}},"version":{"state":1}}],"version":1}}
{"frozen":{"digest":"6b356f220a4949ee5dadfb8d90c0a0068fefe0096a69041a6c623ad49d69e78a","id":"6b356f220a4949ee5dadfb8d90c0a0068fefe0096a69041a6c623ad49d69e78a","revision":"1"},"profile_id":"demo3-claude-write","revision":{"key":{"id":"6b356f220a4949ee5dadfb8d90c0a0068fefe0096a69041a6c623ad49d69e78a","kind":"worker_profile_revision","scope":{"kind":"control"}},"version":{"state":1}},"version":1}
=== 二.4 Create Project and select participant ===
{"delivery":"confirmed","effect_id":"chat:a0961407b1332caf4b197bf617a6f434be49c4edc0e8494988bef5ffb2846b2b","main_room_id":"main-bb910de971d1508f3ca255bbd2f784778e6cd5855d43e2dc344eac5a6c037dc5","project_id":"project-20dbee458160c492002c321c0cc2d5a15d8b0cd1dcae9b2a78842ae4d2e4e17a","receipt":{"matrix_room_id":"!cJWBrC6A4sNcjGZwij:hctl2.localhost"},"state":"pending","version":1}
{"candidate_validation":"accepted_catalog_and_project_policy","optional_skill_degradations":[],"room_id":"main-bb910de971d1508f3ca255bbd2f784778e6cd5855d43e2dc344eac5a6c037dc5","roster_version":1,"selections":[{"key":{"id":"main-bb910de971d1508f3ca255bbd2f784778e6cd5855d43e2dc344eac5a6c037dc5:1:0","kind":"room_selection","scope":{"id":"project-20dbee458160c492002c321c0cc2d5a15d8b0cd1dcae9b2a78842ae4d2e4e17a","kind":"project"}},"version":{"state":1}}]}
=== 二.5 Connect Gitea task source, create Task, adopt contract ===
{"project_id":"project-20dbee458160c492002c321c0cc2d5a15d8b0cd1dcae9b2a78842ae4d2e4e17a","provider_done":[],"source_id":"cbe7b5bb7d79da6e984ba53aff5148ddf937d9908a472c4b175f498cd4d377d0"}
{"effect_id":"task:579a0f332104aaf53a2dfc78915561d21c6d47301012a9b85b14434d1607bf1c","effect_state":"confirmed","project_id":"project-20dbee458160c492002c321c0cc2d5a15d8b0cd1dcae9b2a78842ae4d2e4e17a","provider_done":[],"task":{"data":{"archived":false,"entity":{"account_stable_id":"1","external_entity_kind":"issue","immutable_external_entity_id":"1","provider":"gitea_issues@http://127.0.0.1:3001"},"id":"f10e1ac7a653dbcc1c6589dc6313a3cb8cb0f2cfcea3aa902b9a8876af153605","lifecycle":"open","lifecycle_version":1,"needs_attention":false,"number":1,"pending_contract":null,"project_id":"project-20dbee458160c492002c321c0cc2d5a15d8b0cd1dcae9b2a78842ae4d2e4e17a","repo_id":"bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3","revision":null,"run_occupancy":null,"snapshot":{"key":{"id":"cbe7b5bb7d79da6e984ba53aff5148ddf937d9908a472c4b175f498cd4d377d0","kind":"task_snapshot","scope":{"id":"bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3","kind":"repo"}},"version":{"state":2}},"source_id":"cbe7b5bb7d79da6e984ba53aff5148ddf937d9908a472c4b175f498cd4d377d0","state_version":1,"title":"Fix double() in calculator.py"},"version":3},"task_id":"f10e1ac7a653dbcc1c6589dc6313a3cb8cb0f2cfcea3aa902b9a8876af153605"}
{"project_id":"project-20dbee458160c492002c321c0cc2d5a15d8b0cd1dcae9b2a78842ae4d2e4e17a","provider_done":[],"task":{"data":{"archived":false,"entity":{"account_stable_id":"1","external_entity_kind":"issue","immutable_external_entity_id":"1","provider":"gitea_issues@http://127.0.0.1:3001"},"id":"f10e1ac7a653dbcc1c6589dc6313a3cb8cb0f2cfcea3aa902b9a8876af153605","lifecycle":"open","lifecycle_version":1,"needs_attention":false,"number":1,"pending_contract":null,"project_id":"project-20dbee458160c492002c321c0cc2d5a15d8b0cd1dcae9b2a78842ae4d2e4e17a","repo_id":"bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3","revision":{"backend_projection_digest":null,"binding":null,"material":{"byte_digest":"8544bbfeba73843dd82cfb34ea22fb3aaf98791a509474b02e8b2916f44572b7","control_id":"7f49fdff03a25c42f80ec625e6d814e2","material_id":"a612ab149fa0f3cf8542e2110afe0b3227e6b254120cfb12030da835f130f2cb","scope":{"id":"project-20dbee458160c492002c321c0cc2d5a15d8b0cd1dcae9b2a78842ae4d2e4e17a","kind":"project"}},"number":1,"origin":{"kind":"local","proposal_digest":"8544bbfeba73843dd82cfb34ea22fb3aaf98791a509474b02e8b2916f44572b7","reference":{"key":{"id":"project-20dbee458160c492002c321c0cc2d5a15d8b0cd1dcae9b2a78842ae4d2e4e17a","kind":"project","scope":{"id":"project-20dbee458160c492002c321c0cc2d5a15d8b0cd1dcae9b2a78842ae4d2e4e17a","kind":"project"}},"version":{"state":1}}},"policy_digest":"bce8d69bfce5d280886cba71b261297de016809a3f94f42b1ad63fd2019d0972","proposal_digest":"8544bbfeba73843dd82cfb34ea22fb3aaf98791a509474b02e8b2916f44572b7"},"run_occupancy":null,"snapshot":{"key":{"id":"cbe7b5bb7d79da6e984ba53aff5148ddf937d9908a472c4b175f498cd4d377d0","kind":"task_snapshot","scope":{"id":"bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3","kind":"repo"}},"version":{"state":2}},"source_id":"cbe7b5bb7d79da6e984ba53aff5148ddf937d9908a472c4b175f498cd4d377d0","state_version":1,"title":"Fix double() in calculator.py"},"version":4},"task_id":"f10e1ac7a653dbcc1c6589dc6313a3cb8cb0f2cfcea3aa902b9a8876af153605"}
=== 二.6 Start write invocation (A-chain) ===
Dispatch preview

Object
  invocation: invocation-10afcf764801fe93d27b71298ab400876dd550ce6a55c11127682edd13ccb205
  project: project-20dbee458160c492002c321c0cc2d5a15d8b0cd1dcae9b2a78842ae4d2e4e17a
  room: main-bb910de971d1508f3ca255bbd2f784778e6cd5855d43e2dc344eac5a6c037dc5
  worker target: research
  worker profile: 6b356f220a4949ee5dadfb8d90c0a0068fefe0096a69041a6c623ad49d69e78a
  bundle: bundle-9850d6d5951fe077f958008887aaec24e922edcef6187fb912b6a317d28d122a
  budget: 262144 bytes

Why this needs confirmation
  confirming starts a Room Invocation: the Project authorizes this dispatch and freezes the context bundle listed above
  the dispatched worker acts under the selection's permissions for this room

What happens after you confirm
  - writes the Invocation, its authorization and the dispatch intent to the control store; the roster is not changed
  - the reconcile loop hands the dispatch to the paired Agency, which runs the worker
  - undo: `hctl2 invocation cancel` revokes the authorization and queues the worker's stop; completed results stay recorded

Confirm with
  /var/tmp/d3/prefix/bin/hctl2 --root /var/tmp/d3/state invocation start --input /var/tmp/d3/work/a-invoke.json --key a-invoke --preview-token ae380fdabfb945e53e520587b4790d5145fc85b580c43525daf1c3680f720d5d
{
  "invocation_id": null,
  "state": "completed",
  "reason": "Git version admitted; not Task acceptance"
}
=== 二.7 Check ChangeSet Revision, diff, auto-publish, Room timeline ===
{
  "base_commit_sha": "65db47301586e07555c48620105f8ae62296f68d",
  "change_set_id": "cs-35c95344328b479f7826e8caf79a34e1ac3794b14a656539890ac40fcd800908",
  "change_set_revision_id": "csr-9347f5284fde8b4db55522fe675388118544159b7f03d9d97322e414dda17429",
  "parent_revision_id": null,
  "producer_ref": {
    "invocation_id": "invocation-10afcf764801fe93d27b71298ab400876dd550ce6a55c11127682edd13ccb205",
    "invocation_version": 1,
    "kind": "invocation"
  },
  "result_tree_sha": "cf65025f06990196e26e0fbfd48c437bcb4c359e",
  "review_subject_digest": "508ab711c58fd5b3e813ad62234b574f0097f77ab6a75a2667d632a8b342b03a",
  "revision_digest": "f9aa74edd82c23c509a00730a71d4dad77f62ac133f294c16dd9b7ee85c32846"
}
diff --git a/calculator.py b/calculator.py
index 883f5b1..750e427 100644
--- a/calculator.py
+++ b/calculator.py
@@ -1,3 +1,3 @@
 def double(value):
     """Return twice the argument."""
-    return value  # bug: returns the argument unchanged
+    return value * 2
{
  "state": "published",
  "stages": {
    "push": {
      "commit_sha": "3c236e39cb744df2a510b5cf0ce6e64b3825f706",
      "confirmed": true,
      "dispatched": false
    },
    "review_request": {
      "commit_sha": "3c236e39cb744df2a510b5cf0ce6e64b3825f706",
      "confirmed": true,
      "dispatched": false,
      "index": 2
    }
  },
  "mapping": {
    "change_set_id": "cs-35c95344328b479f7826e8caf79a34e1ac3794b14a656539890ac40fcd800908",
    "change_set_revision_id": "csr-9347f5284fde8b4db55522fe675388118544159b7f03d9d97322e414dda17429",
    "intent_id": "rp-4b36b0d2d1767d46ddc15417fac981610c38ac38986ce3c38fe833db4f349cf5",
    "platform_commit_sha": "3c236e39cb744df2a510b5cf0ce6e64b3825f706",
    "recorded_at_unix_ms": 1792417914858,
    "review_request": {
      "base_branch": "main",
      "head_branch": "hctl2/cs-35c95344328b479f7826e8caf79a34e1ac3794b14a656539890ac40fcd800908",
      "index": 2
    },
    "round": 1
  }
}
[
  "Write admitted as version csr-9347f5284fde of ChangeSet cs-35c95344328b (baseline 65db47301586, result tree cf65025f0699). Publishing for review is queued: push to hctl2/cs-35c95344328b479f7826e8caf79a34e1ac3794b14a656539890ac40fcd800908 and open or update the review request, each read back."
]
=== 二.8 First human preview: Integration Receipt ===
Integration preview

Object
  ChangeSet Revision: csr-9347f5284fde8b4db55522fe675388118544159b7f03d9d97322e414dda17429
  repo: bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3
  target ref: refs/heads/main
  authorization form: accept_advance
  strategy: fast_forward
  target head at preview: 65db47301586e07555c48620105f8ae62296f68d

Why this needs confirmation
  confirming freezes this intent and executes the integration
  the exact revision and target recorded here are what will be merged

What happens after you confirm
  - persists the integration intent, then executes and reads back the merge in the background
  - the target ref advances in the registered repository; this command cannot undo a merge
  - an Integration Receipt is written only after readback confirms the target head

Confirm with
  /var/tmp/d3/prefix/bin/hctl2 --root /var/tmp/d3/state integration preview --input /var/tmp/d3/work/a-integration.json --key a-merge --preview-token 4a3fe32e121e48a79ecbd596f205aa983de2eafd11a02084571917f52bc5e4a4
{
  "state": "succeeded",
  "receipt_id": "receipt-ae9decc59fade1c384c988aed22df932c04137f7fa1f772b547584ad2e807525",
  "evidence_level": "hctl2-tool",
  "git_readback_level": "unmediated",
  "target_head_after": "3c236e39cb744df2a510b5cf0ce6e64b3825f706"
}
=== 二.9 Second human preview: Task Completion Receipt ===
Completion preview

Object
  project: project-20dbee458160c492002c321c0cc2d5a15d8b0cd1dcae9b2a78842ae4d2e4e17a
  task: f10e1ac7a653dbcc1c6589dc6313a3cb8cb0f2cfcea3aa902b9a8876af153605
  Task Revision: 1
  lifecycle version: 1 -> 2
  Task Completion Receipt: f10e1ac7a653dbcc1c6589dc6313a3cb8cb0f2cfcea3aa902b9a8876af153605:2
  item 1: mechanical · unmediated · hctl2-tool — the Task change is integrated
  item 2: human · narrated · human local-owner:1000 — a human verified the fix and tests

Why this needs confirmation
  completing is terminal for this lifecycle version: the receipt below is what the Task is judged by
  each item's judge and evidence channel are as listed; a human item is the submitting person's own verdict

What happens after you confirm
  - writes the Task Completion Receipt and advances the Task to completed in the same control-store transaction
  - the receipt binds exactly the Task Revision and the items above; nothing is rewritten later
  - undo: `hctl2 task reopen` returns the Task to open under a new lifecycle version; the Completion Receipt stays recorded

Confirm with
  /var/tmp/d3/prefix/bin/hctl2 --root /var/tmp/d3/state task complete --input /var/tmp/d3/work/a-complete.json --key a-complete --preview-token 07ccc61cd42da61893edf7cd0d1f29c73597c4c7733f6b1dcbf8ca6f5f493194
{
  "lifecycle": "completed",
  "lifecycle_version": 2,
  "receipt_id": "f10e1ac7a653dbcc1c6589dc6313a3cb8cb0f2cfcea3aa902b9a8876af153605:2",
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
            "id": "receipt-ae9decc59fade1c384c988aed22df932c04137f7fa1f772b547584ad2e807525",
            "kind": "integration_receipt",
            "scope": {
              "id": "bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3",
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
          "id": "cbe7b5bb7d79da6e984ba53aff5148ddf937d9908a472c4b175f498cd4d377d0",
          "kind": "task_snapshot",
          "scope": {
            "id": "bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3",
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
          "id": "cbe7b5bb7d79da6e984ba53aff5148ddf937d9908a472c4b175f498cd4d377d0",
          "kind": "task_snapshot",
          "scope": {
            "id": "bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3",
            "kind": "repo"
          }
        },
        "version": {
          "state": 2
        }
      },
      "text": "a human verified the fix and tests",
      "text_digest": "1c66a32ae51b1e316502045d2bd326b6eafbb1b196a34f604e8bd7d61b64e0b9",
      "validation_level": "narrated"
    }
  ]
}
=== 二.10 Platform readback & task show 10-second answer ===
{
  "state": "published",
  "push_confirmed": true,
  "pr_confirmed": true,
  "platform_commit_sha": "3c236e39cb744df2a510b5cf0ce6e64b3825f706",
  "review_request": {
    "base_branch": "main",
    "head_branch": "hctl2/cs-35c95344328b479f7826e8caf79a34e1ac3794b14a656539890ac40fcd800908",
    "index": 2
  }
}
{
  "state": "succeeded",
  "receipt_id": "receipt-ae9decc59fade1c384c988aed22df932c04137f7fa1f772b547584ad2e807525",
  "evidence_level": "hctl2-tool",
  "integrated_commit": "3c236e39cb744df2a510b5cf0ce6e64b3825f706",
  "target_head_after": "3c236e39cb744df2a510b5cf0ce6e64b3825f706",
  "git_readback": {
    "commit": "3c236e39cb744df2a510b5cf0ce6e64b3825f706",
    "commit_parents": [
      "65db47301586e07555c48620105f8ae62296f68d"
    ],
    "commit_present": true,
    "commit_tree": "cf65025f06990196e26e0fbfd48c437bcb4c359e",
    "contains": true,
    "evidence_level": "unmediated",
    "git": {
      "path": "/usr/bin/git",
      "version": "2.53.0"
    },
    "head": "3c236e39cb744df2a510b5cf0ce6e64b3825f706",
    "head_tree": "cf65025f06990196e26e0fbfd48c437bcb4c359e",
    "observed_at_unix_ms": 1792417921917,
    "operation": "readback",
    "ref": "refs/heads/main",
    "remote": "http://127.0.0.1:3001/hctl-7f49fdff03a25c42/demo3-walkthrough.git",
    "remote_head": "3c236e39cb744df2a510b5cf0ce6e64b3825f706",
    "repository": "/var/tmp/d3/state/integration/readback/bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3.git",
    "schema": "hctl2.readback.v1"
  },
  "platform_readback": null
}
Task f10e1ac7a653dbcc — Fix double() in calculator.py
  goal      Fix double() in calculator.py
  state     completed; contract revision 1
  blocking  nothing
  harness   claude-code (via herdr, model none) — invocation invocation-10afcf764801 completed
  evidence  version csr-9347f5284fde, review request #2 (published), Integration Receipt receipt-ae9decc59fad, Completion Receipt f10e1ac7a653dbcc
  next      nothing — the Task is completed
=== 四.1 Create C-chain Project, select participant, attach source, create & adopt Task ===
{"candidate_validation":"accepted_catalog_and_project_policy","optional_skill_degradations":[],"room_id":"main-bfb319270fbdd77ca463905bd614a8244cc678e3d5c6b9b871c7fbe7484b9c24","roster_version":1,"selections":[{"key":{"id":"main-bfb319270fbdd77ca463905bd614a8244cc678e3d5c6b9b871c7fbe7484b9c24:1:0","kind":"room_selection","scope":{"id":"project-5be39e260670abd9860ab42c821b15933857de3e0c6175a615af7628a574e16a","kind":"project"}},"version":{"state":1}}]}
{"project_id":"project-5be39e260670abd9860ab42c821b15933857de3e0c6175a615af7628a574e16a","provider_done":[],"source_id":"cbe7b5bb7d79da6e984ba53aff5148ddf937d9908a472c4b175f498cd4d377d0"}
{"project_id":"project-5be39e260670abd9860ab42c821b15933857de3e0c6175a615af7628a574e16a","provider_done":[],"task":{"data":{"archived":false,"entity":{"account_stable_id":"1","external_entity_kind":"issue","immutable_external_entity_id":"3","provider":"gitea_issues@http://127.0.0.1:3001"},"id":"93da6917b576934ed4283da42d12c63bc244383c392357d51f77ee358e852882","lifecycle":"open","lifecycle_version":1,"needs_attention":false,"number":3,"pending_contract":null,"project_id":"project-5be39e260670abd9860ab42c821b15933857de3e0c6175a615af7628a574e16a","repo_id":"bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3","revision":{"backend_projection_digest":null,"binding":null,"material":{"byte_digest":"d1a418a0b1be537887beb31c084566f9ecaff3dcc00d063925f6d11ec9c6b73a","control_id":"7f49fdff03a25c42f80ec625e6d814e2","material_id":"70e686c2bc9806050b3b94761940386eb59749431ab8a0eeed6c697ba7e90a6f","scope":{"id":"project-5be39e260670abd9860ab42c821b15933857de3e0c6175a615af7628a574e16a","kind":"project"}},"number":1,"origin":{"kind":"local","proposal_digest":"d1a418a0b1be537887beb31c084566f9ecaff3dcc00d063925f6d11ec9c6b73a","reference":{"key":{"id":"project-5be39e260670abd9860ab42c821b15933857de3e0c6175a615af7628a574e16a","kind":"project","scope":{"id":"project-5be39e260670abd9860ab42c821b15933857de3e0c6175a615af7628a574e16a","kind":"project"}},"version":{"state":1}}},"policy_digest":"bce8d69bfce5d280886cba71b261297de016809a3f94f42b1ad63fd2019d0972","proposal_digest":"d1a418a0b1be537887beb31c084566f9ecaff3dcc00d063925f6d11ec9c6b73a"},"run_occupancy":null,"snapshot":{"key":{"id":"cbe7b5bb7d79da6e984ba53aff5148ddf937d9908a472c4b175f498cd4d377d0","kind":"task_snapshot","scope":{"id":"bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3","kind":"repo"}},"version":{"state":3}},"source_id":"cbe7b5bb7d79da6e984ba53aff5148ddf937d9908a472c4b175f498cd4d377d0","state_version":1,"title":"Add triple() in calculator.py"},"version":4},"task_id":"93da6917b576934ed4283da42d12c63bc244383c392357d51f77ee358e852882"}
=== 四.2 Start write invocation (C-chain) and observe pending_human ===
{
  "allow_update": true,
  "audit_scope": "minimal",
  "binding_version": 1,
  "branch_rule": "hctl2/{change_set}",
  "description_source": "none",
  "repo_id": "bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3",
  "requires_human_confirmation": true,
  "target_branch": "main"
}
{
  "invocation_id": null,
  "state": "completed",
  "reason": "Git version admitted; not Task acceptance"
}
{
  "intent_id": "rp-de0fc0515a942f626460279891366fc6404d66e78c62aebfbea9fcd10fabb9ca",
  "state": "pending_human",
  "requires_human_confirmation": true,
  "push_dispatched": false
}
=== 四.3 First human preview (C-chain): review publish ===
Review publish preview

Object
  repo: bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3
  publish intent: rp-de0fc0515a942f626460279891366fc6404d66e78c62aebfbea9fcd10fabb9ca
  round: 1
  ChangeSet: cs-5b5b5b3ddca4130971b607017286ed85fe096ae1036bf5bb58bda691c45705b4
  ChangeSet Revision: csr-3f87c4c01c46b4d9cb0d6c593ddfaded321f8887f5af70c07841a5af3c5931dc
  push to branch: hctl2/cs-5b5b5b3ddca4130971b607017286ed85fe096ae1036bf5bb58bda691c45705b4
  review request target: main
  updates an existing request: allowed: a later round may update the request this round opens

Why this needs confirmation
  this intent was frozen to wait for a human; confirming is the release
  it authorizes a platform write: push the branch, then create or update the review request — not a merge

What happens after you confirm
  - persists the release, then runs both stages in the background: push the commit, then create or update the review request, each confirmed by reading the platform back
  - records the pushed commit, the review request index and this ChangeSet's revision mapping in the control store
  - undo: the push cannot be taken back; a review request that was created is closed on the platform, not by this CLI

Confirm with
  /var/tmp/d3/prefix/bin/hctl2 --root /var/tmp/d3/state review publish bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3 rp-de0fc0515a942f626460279891366fc6404d66e78c62aebfbea9fcd10fabb9ca --preview-token 4c44aa19bff1a598e55620665d8cde2f41d86ea095cd656b57149093cdaa1674
{
  "branch": "hctl2/cs-5b5b5b3ddca4130971b607017286ed85fe096ae1036bf5bb58bda691c45705b4",
  "target_branch": "main",
  "authorizes": "publishing this revision for review on the platform: push to the branch, then create or update the review request. Not a merge."
}
{"intent_id":"rp-de0fc0515a942f626460279891366fc6404d66e78c62aebfbea9fcd10fabb9ca","repo_id":"bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3","state":"pending"}
{
  "state": "published",
  "stages": {
    "push": {
      "commit_sha": "f5d1069f9fcce9ac76b7a77843c7561d9b23caa6",
      "confirmed": true,
      "dispatched": false
    },
    "review_request": {
      "commit_sha": "f5d1069f9fcce9ac76b7a77843c7561d9b23caa6",
      "confirmed": true,
      "dispatched": false,
      "index": 4
    }
  },
  "mapping": {
    "change_set_id": "cs-5b5b5b3ddca4130971b607017286ed85fe096ae1036bf5bb58bda691c45705b4",
    "change_set_revision_id": "csr-3f87c4c01c46b4d9cb0d6c593ddfaded321f8887f5af70c07841a5af3c5931dc",
    "intent_id": "rp-de0fc0515a942f626460279891366fc6404d66e78c62aebfbea9fcd10fabb9ca",
    "platform_commit_sha": "f5d1069f9fcce9ac76b7a77843c7561d9b23caa6",
    "recorded_at_unix_ms": 1792417954037,
    "review_request": {
      "base_branch": "main",
      "head_branch": "hctl2/cs-5b5b5b3ddca4130971b607017286ed85fe096ae1036bf5bb58bda691c45705b4",
      "index": 4
    },
    "round": 1
  }
}
=== 四.4 Second human preview (C-chain): integration preview & submit ===
Integration preview

Object
  ChangeSet Revision: csr-3f87c4c01c46b4d9cb0d6c593ddfaded321f8887f5af70c07841a5af3c5931dc
  repo: bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3
  target ref: refs/heads/main
  authorization form: accept_advance
  strategy: fast_forward
  target head at preview: 3c236e39cb744df2a510b5cf0ce6e64b3825f706

Why this needs confirmation
  confirming freezes this intent and executes the integration
  the exact revision and target recorded here are what will be merged

What happens after you confirm
  - persists the integration intent, then executes and reads back the merge in the background
  - the target ref advances in the registered repository; this command cannot undo a merge
  - an Integration Receipt is written only after readback confirms the target head

Confirm with
  /var/tmp/d3/prefix/bin/hctl2 --root /var/tmp/d3/state integration preview --input /var/tmp/d3/work/c-integration.json --key c-merge --preview-token a8bce750569283749db84304c451cb09a213695c68c04cad2d3186add57383fe
{
  "state": "succeeded",
  "receipt_id": "receipt-8f0700a6e9000c2bf2ebfd98509c177508cd6135c5011885c1052edf8d61b07a",
  "evidence_level": "hctl2-tool",
  "git_readback_level": "unmediated",
  "target_head_after": "f5d1069f9fcce9ac76b7a77843c7561d9b23caa6"
}
=== 四.5 Third human preview (C-chain): task complete & task show ===
Completion preview

Object
  project: project-5be39e260670abd9860ab42c821b15933857de3e0c6175a615af7628a574e16a
  task: 93da6917b576934ed4283da42d12c63bc244383c392357d51f77ee358e852882
  Task Revision: 1
  lifecycle version: 1 -> 2
  Task Completion Receipt: 93da6917b576934ed4283da42d12c63bc244383c392357d51f77ee358e852882:2
  item 1: mechanical · unmediated · hctl2-tool — the Task change is integrated
  item 2: human · narrated · human local-owner:1000 — a human verified the fix and tests

Why this needs confirmation
  completing is terminal for this lifecycle version: the receipt below is what the Task is judged by
  each item's judge and evidence channel are as listed; a human item is the submitting person's own verdict

What happens after you confirm
  - writes the Task Completion Receipt and advances the Task to completed in the same control-store transaction
  - the receipt binds exactly the Task Revision and the items above; nothing is rewritten later
  - undo: `hctl2 task reopen` returns the Task to open under a new lifecycle version; the Completion Receipt stays recorded

Confirm with
  /var/tmp/d3/prefix/bin/hctl2 --root /var/tmp/d3/state task complete --input /var/tmp/d3/work/c-complete.json --key c-complete --preview-token 3067bf25939d7ecdd60e7817d2d02f93351508b707dfc9316b21e65f5e6ff7f2
{
  "lifecycle": "completed",
  "lifecycle_version": 2,
  "receipt_id": "93da6917b576934ed4283da42d12c63bc244383c392357d51f77ee358e852882:2",
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
            "id": "receipt-8f0700a6e9000c2bf2ebfd98509c177508cd6135c5011885c1052edf8d61b07a",
            "kind": "integration_receipt",
            "scope": {
              "id": "bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3",
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
          "id": "cbe7b5bb7d79da6e984ba53aff5148ddf937d9908a472c4b175f498cd4d377d0",
          "kind": "task_snapshot",
          "scope": {
            "id": "bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3",
            "kind": "repo"
          }
        },
        "version": {
          "state": 3
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
          "id": "cbe7b5bb7d79da6e984ba53aff5148ddf937d9908a472c4b175f498cd4d377d0",
          "kind": "task_snapshot",
          "scope": {
            "id": "bf7b0efd5597b94a9a30435b7eaad74a5cdc42a0f9eb335ff40ff5377442fdc3",
            "kind": "repo"
          }
        },
        "version": {
          "state": 3
        }
      },
      "text": "a human verified the fix and tests",
      "text_digest": "1c66a32ae51b1e316502045d2bd326b6eafbb1b196a34f604e8bd7d61b64e0b9",
      "validation_level": "narrated"
    }
  ]
}
Task 93da6917b576934e — Add triple() in calculator.py
  goal      Add triple() in calculator.py
  state     completed; contract revision 1
  blocking  nothing
  harness   claude-code (via herdr, model none) — invocation invocation-0c587406623f completed
  evidence  version csr-3f87c4c01c46, review request #4 (published), Integration Receipt receipt-8f0700a6e900, Completion Receipt 93da6917b576934e
  next      nothing — the Task is completed
=== 五.1.1 Stop control + Agency + hosted services and restart ===
{"stop_requested":true}
{
  "version": 2,
  "revisions": 1
}
{
  "state": "published",
  "index": 2
}
{
  "state": "succeeded",
  "receipt_id": "receipt-ae9decc59fade1c384c988aed22df932c04137f7fa1f772b547584ad2e807525"
}
Task f10e1ac7a653dbcc — Fix double() in calculator.py
  goal      Fix double() in calculator.py
  state     completed; contract revision 1
  blocking  nothing
  harness   claude-code (via herdr, model none) — invocation invocation-10afcf764801 completed
  evidence  version csr-9347f5284fde, review request #2 (published), Integration Receipt receipt-ae9decc59fad, Completion Receipt f10e1ac7a653dbcc
  next      nothing — the Task is completed
=== 五.2.1 Secrets & worktree isolation ===
total 24
drwx------  2 jackywang jackywang 4096 Oct 19 21:51 .
drwxrwxr-x 10 jackywang jackywang 4096 Oct 19 21:52 ..
-rw-------  1 jackywang jackywang   64 Oct 19 21:51 54f7c46f219ee10351fb3d9e024b66341fc81cff4862e854995366372ac5a59c.secret
-rw-------  1 jackywang jackywang   40 Oct 19 21:51 88fda005482d5aa7cae878d4e0d6697dce16693a4706160d7660e3b687e3297e.secret
-rw-------  1 jackywang jackywang   24 Oct 19 21:51 9724b01a27d0993e4c2862fe72ffe856d5b777250eb5df6f4b1b51d2e60f3dcd.secret
-rw-------  1 jackywang jackywang  146 Oct 19 21:51 ea9da94d1b0ad2f86bed0972586ca08ebb54121ffe357780d4db92b647812151.secret
{
  "status": "materialized_residual",
  "worktree_path": "/tmp/hctl2-exec-dfe3905c1d4ca51dd182/write-worktrees/ad5fe2e9600d0c4bbd9fd582e53c5845151b185c578e0148e9fb751afaab89a7/cs-35c95344328b479f7826e8caf79a34e1ac3794b14a656539890ac40fcd800908"
}
=== 五.3.1 Bare task escape hatch ===
{"error":{"code":"CONTRACT_REQUIRED","message":"adopt a contract before completing this Task","recovery_action":"adopt_contract"}}
=== 六 Safe cleanup of only this instance's EXEC_ROOT and WS ===
EXEC_ROOT=/tmp/hctl2-exec-dfe3905c1d4ca51dd182
{"error":{"code":"AGENCY_UNREACHABLE","message":"paired Agency endpoint cannot be reached","recovery_action":"check_agency"}}
hctl2: services are not running
=== ALL WALKTHROUGH STEPS COMPLETED ===
```

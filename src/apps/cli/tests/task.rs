//! Project is a fixture until package 辛; every Task operation crosses the real CLI/RPC.
use serde_json::{Value, json};
use std::{
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};
use store::{
    Actor, ActorSource, Expected, ProjectSettings, Record, RecordData, Reference, Scope, Store,
    TrustedActor, Version,
};
struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = run(&self.0, &["stop"]);
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn run(root: &Path, args: &[&str]) -> (bool, Value) {
    run_with_env(root, args, &[])
}
/// Extra env is applied to the process we spawn. Only the daemon hosts the store,
/// so the completion-failure seam must be attached to `hctl2 start`, not to the CLI.
fn run_with_env(root: &Path, args: &[&str], envs: &[(&str, &str)]) -> (bool, Value) {
    let out = Command::new(
        std::env::var("CARGO_BIN_EXE_hctl2")
            .expect("CARGO_BIN_EXE_hctl2 must be set to run this test"),
    )
    .env(
        "HCTL2_CONTROL_BIN",
        std::env::var("CARGO_BIN_EXE_hctl2-control")
            .expect("CARGO_BIN_EXE_hctl2-control must be set to run this test"),
    )
    .env("HCTL2_GH", root.join("gh"))
    .envs(envs.iter().copied())
    .args(["--json", "--root", root.to_str().unwrap()])
    .args(args)
    .output()
    .unwrap();
    (out.status.success(),serde_json::from_slice(&out.stdout).unwrap_or_else(|_|json!({"stdout":String::from_utf8_lossy(&out.stdout),"stderr":String::from_utf8_lossy(&out.stderr)})))
}
fn cmd(root: &Path, kind: &str, key: &str, action: Value) -> Value {
    let (ok, result) = cmd_result(root, kind, key, action);
    assert!(ok, "submit {kind}: {result}");
    result
}
fn cmd_result(root: &Path, kind: &str, key: &str, action: Value) -> (bool, Value) {
    let path = root.join(format!("{key}.json"));
    std::fs::write(&path, action.to_string()).unwrap();
    let p = path.to_str().unwrap();
    let (ok, preview) = run(root, &["task", kind, "--key", key, "--input", p]);
    assert!(ok, "preview {kind}: {preview}");
    run(
        root,
        &[
            "task",
            kind,
            "--key",
            key,
            "--input",
            p,
            "--preview-token",
            preview["preview_token"].as_str().unwrap(),
        ],
    )
}

/// Live fixture shared by every Task acceptance test: a registered repo, two seeded
/// Project records ("A"/"B") at version 1, one seeded evidence record in Project "A",
/// and a running daemon with `connect` + `attach` already applied to both Projects.
struct Fixture {
    root: PathBuf,
    _temp: Temp,
    source_id: String,
}

fn fixture(name: &str, daemon_env: &[(&str, &str)]) -> Fixture {
    let root = std::env::temp_dir().join(format!("hctl-cli-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let temp = Temp(root.clone());
    let gh = root.join("gh");
    std::fs::write(
        root.join("gh.jq"),
        std::fs::canonicalize(env!("HCTL2_TEST_JQ"))
            .unwrap()
            .to_str()
            .unwrap(),
    )
    .unwrap();
    std::fs::write(&gh,r#"#!/bin/sh
method=GET
header=no
for arg do
  if [ "$arg" = --include ]; then header=yes; fi
  if [ "$previous" = --method ]; then method="$arg"; fi
  previous="$arg"
  path="$arg"
done
if [ "$header" = yes ]; then
  if [ "$method" = POST ] && [ -f "$0.hide" ]; then printf 'HTTP/1.1 503 Lost Confirmation\n\n';
  elif [ "$method" = PATCH ]; then printf 'HTTP/1.1 403 Forbidden\n\n';
  else printf 'HTTP/1.1 200 OK\n\n'; fi
fi
case "$method:$path" in
 GET:user) printf '{"id":5}' ;;
 GET:repos/owner/fixture) printf '{"id":77,"full_name":"owner/fixture","clone_url":"https://github.com/owner/fixture.git","has_issues":true,"permissions":{"push":true,"admin":true}}' ;;
 GET:*page=2*) printf '[]' ;;
 GET:*/issues\?*) if [ -f "$0.created" ] && [ ! -f "$0.hide" ]; then jqbin="$(cat "$0.jq")"; "$jqbin" -s '.' "$0.created"; else printf '[]'; fi ;;
 GET:*/issues/1/parent) if [ "$header" = yes ]; then :; fi; printf 'null' ;;
 GET:*/issues/1) cat "$0.created" ;;
 GET:*) printf '[]' ;;
 POST:*/issues) jqbin="$(cat "$0.jq")"; "$jqbin" '. + {id:1,node_id:"node1",number:1,state:"open",updated_at:"t0",labels:[],comments:0}' > "$0.created"; printf x >> "$0.posts"; cat "$0.created" ;;
 PATCH:*) cat >/dev/null; printf '{"message":"Forbidden"}'; exit 1 ;;
 *) exit 1 ;;
esac
"#).unwrap();
    std::fs::set_permissions(&gh, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut s = Store::open(&root).unwrap();
    let actor = TrustedActor(Actor {
        principal: "fixture-owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control],
        authority: None,
    });
    let request=serde_json::from_value(json!({"name":"fixture","origin":"external","platform":"github","instance":"github.com","platform_repo_id":"77","platform_path":"owner/fixture","default_source":"github_issues"})).unwrap();
    let reg = repo::admit(
        &mut s,
        &actor,
        "repo",
        "repo",
        repo::prepare(request, None).unwrap(),
    )
    .unwrap();
    let repo_id = reg.repo_id.clone();
    repo::begin_step(&mut s, &actor, &reg.repo_id, "platform").unwrap();
    repo::confirm_platform(
        &mut s,
        &reg.repo_id,
        repo::PlatformObservation {
            instance: "github.com".into(),
            stable_id: "77".into(),
            full_name: "owner/fixture".into(),
            clone_url: "https://github.com/owner/fixture.git".into(),
            account_id: "5".into(),
            has_issues: true,
            can_write_issues: true,
            credential_ref: String::new(),
        },
    )
    .unwrap();
    for p in ["A", "B"] {
        let data = RecordData::Project {
            repo_id: reg.repo_id.clone(),
            settings: ProjectSettings {
                publish_review_requires_confirmation: true,
                selection_policy: json!({}),
            },
            archived: false,
        };
        let record = Record {
            key: task::key(Scope::Project(p.into()), "project", p),
            version: 1,
            revision_digest: foundation::canonical_json_sha256(
                &serde_json::to_value(&data).unwrap(),
            )
            .unwrap(),
            data,
            sources: vec![],
            materials: vec![],
        };
        let a = task::owner(&actor, [record.key.scope.clone()]).unwrap();
        let input = serde_json::to_value(&record).unwrap();
        let c = store::Command {
            command_id: p.into(),
            idempotency_key: p.into(),
            actor: a.0.clone(),
            target: record.key.clone(),
            expected: Expected::Absent,
            binding: task::reference(&record),
            input_digest: store::Command::digest_input("fixture", &input).unwrap(),
            operation: "fixture".into(),
            input,
        };
        s.submit(s.generation(), &a, &c, None, |tx| {
            tx.put(&record)?;
            Ok(json!({}))
        })
        .unwrap();
    }
    // An existing record any mechanical item may cite as unmediated evidence.
    let evidence = task::value_record(
        task::key(Scope::Project("A".into()), "fixture_evidence", "evidence-1"),
        1,
        &json!({"seeded": "evidence"}),
    )
    .unwrap();
    let a = task::owner(&actor, [evidence.key.scope.clone()]).unwrap();
    let input = serde_json::to_value(&evidence).unwrap();
    let c = store::Command {
        command_id: "evidence-1".into(),
        idempotency_key: "evidence-1".into(),
        actor: a.0.clone(),
        target: evidence.key.clone(),
        expected: Expected::Absent,
        binding: task::reference(&evidence),
        input_digest: store::Command::digest_input("fixture", &input).unwrap(),
        operation: "fixture".into(),
        input,
    };
    s.submit(s.generation(), &a, &c, None, |tx| {
        tx.put(&evidence)?;
        Ok(json!({}))
    })
    .unwrap();
    drop(s);
    let (started, start_result) = run_with_env(
        &root,
        &["start", "--secret-backend", "user-file"],
        daemon_env,
    );
    assert!(started, "start {name}: {start_result}");
    let source = cmd(
        &root,
        "connect",
        "connect",
        json!({"repo_id":repo_id,"candidate_id":"github_issues","consent":true,"make_default":false}),
    );
    let sid = source["source_id"].as_str().unwrap().to_string();
    for p in ["A", "B"] {
        cmd(
            &root,
            "attach",
            &format!("attach-{p}"),
            json!({"project_id":p,"project_version":1,"source_id":sid,"approved_scope":"77","consent":true}),
        );
    }
    Fixture {
        root,
        _temp: temp,
        source_id: sid,
    }
}

/// The trusted actor the daemon derives from the connecting process: `local-owner:{uid}`.
fn actor_principal(root: &Path) -> String {
    format!("local-owner:{}", std::fs::metadata(root).unwrap().uid())
}

/// Create a Task in Project "A" through the CLI, then adopt a two-item contract
/// (0: mechanical/unmediated, 1: human) through the CLI, and return what completion needs.
struct Adopted {
    task_id: String,
    version: i64,
    revision_number: i64,
    lifecycle_version: i64,
}

fn adopt(fx: &Fixture) -> Adopted {
    let created = cmd(
        &fx.root,
        "create",
        "create",
        json!({"project_id":"A","project_version":1,"source_id":fx.source_id.clone(),"title":"new","body":"body"}),
    );
    let task_id = created["task_id"].as_str().unwrap().to_string();
    let version = created["task"]["version"].as_i64().unwrap();
    let contract = task::Contract {
        scope: "A".into(),
        expected_outcome: "the card is finished".into(),
        acceptance: vec![
            task::Acceptance {
                text: "a mechanical check passed".into(),
                grade: task::Grade::Mechanical,
                evidence: Some(task::EvidenceRequirement {
                    accept: None,
                    min_channel: Some("unmediated".into()),
                }),
            },
            task::Acceptance {
                text: "a human judged it done".into(),
                grade: task::Grade::Human,
                evidence: None,
            },
        ],
        roles: vec![],
        capabilities: vec![],
    };
    let proposal_digest =
        foundation::canonical_json_sha256(&serde_json::to_value(&contract).unwrap()).unwrap();
    let adoption = task::Adoption {
        contract,
        origin: task::ContractOrigin::Local {
            reference: Reference {
                key: task::key(Scope::Project("A".into()), "project", "A"),
                version: Version::State(1),
            },
            proposal_digest,
        },
    };
    let adopted = cmd(
        &fx.root,
        "adopt",
        "adopt",
        json!({
            "project_id":"A","project_version":1,"task_id":task_id,"version":version,
            "adoption": serde_json::to_value(&adoption).unwrap(),
        }),
    );
    Adopted {
        task_id,
        version: adopted["task"]["version"].as_i64().unwrap(),
        revision_number: adopted["task"]["data"]["revision"]["number"]
            .as_i64()
            .unwrap(),
        lifecycle_version: adopted["task"]["data"]["lifecycle_version"]
            .as_i64()
            .unwrap(),
    }
}

/// The `complete` action body: item 0 cites the seeded record as unmediated tool
/// evidence; item 1 is the submitting human's own verdict.
fn completion_action(adopted: &Adopted, principal: &str) -> Value {
    let evidence = vec![
        task::ItemEvidence {
            item: 0,
            judge: task::Judge::Hctl2Tool,
            channel: "unmediated".into(),
            references: vec![Reference {
                key: task::key(Scope::Project("A".into()), "fixture_evidence", "evidence-1"),
                version: Version::State(1),
            }],
            producer: Some("hctl2-tool".into()),
            generation: Some(7),
        },
        task::ItemEvidence {
            item: 1,
            judge: task::Judge::Human {
                actor: principal.into(),
            },
            channel: "narrated".into(),
            references: vec![],
            producer: None,
            generation: None,
        },
    ];
    json!({
        "project_id":"A",
        "task_id":adopted.task_id.clone(),
        "version":adopted.version,
        "lifecycle_version":adopted.lifecycle_version,
        "revision_number":adopted.revision_number,
        "acceptance": serde_json::to_value(&evidence).unwrap(),
    })
}

#[test]
fn task_commands_cross_live_daemon_preview_replay_and_restart() {
    let fx = fixture("restart", &[]);
    let root = fx.root.clone();
    let sid = fx.source_id.clone();
    let create = json!({"project_id":"A","project_version":1,"source_id":sid.clone(),"title":"new","body":"body"});
    std::fs::write(root.join("gh.hide"), "hide readback until restart").unwrap();
    let (ok, pending) = cmd_result(&root, "create", "create", create.clone());
    assert!(!ok, "lost confirmation must not report success: {pending}");
    assert_eq!(pending["effect_state"], "unknown");
    assert!(run(&root, &["stop"]).0);
    std::thread::sleep(std::time::Duration::from_millis(100));
    std::fs::remove_file(root.join("gh.hide")).unwrap();
    assert!(run(&root, &["start", "--secret-backend", "user-file"]).0);
    let created = cmd(&root, "create", "create", create.clone());
    assert_eq!(pending["task_id"], created["task_id"]);
    assert_eq!(created["effect_state"], "confirmed");
    assert_eq!(created["task"]["data"]["lifecycle"], "open");
    let duplicate = cmd(&root, "create", "create", create.clone());
    assert_eq!(created["task_id"], duplicate["task_id"]);
    let b = cmd(
        &root,
        "claim",
        "claim-b",
        json!({"project_id":"B","project_version":1,"source_id":sid.clone(),"entity_id":"node1"}),
    );
    assert_ne!(b["task_id"], created["task_id"]);
    for (project, task_id) in [
        ("A", created["task_id"].as_str().unwrap()),
        ("B", b["task_id"].as_str().unwrap()),
    ] {
        let (ok, task) = run(&root, &["task", "show", project, task_id]);
        assert!(ok);
        let (ok, failure) = cmd_result(
            &root,
            "update",
            &format!("denied-{project}"),
            json!({
                "project_id":project,"task_id":task_id,"version":task["version"],
                "state_version":task["data"]["state_version"],"fields":{"title":"denied edit"}
            }),
        );
        assert!(!ok);
        assert_eq!(failure["effect_state"], "rejected", "{failure}");
        assert_eq!(failure["error"]["code"], "PROVIDER_REJECTED");
        assert_eq!(failure["task"]["data"]["lifecycle"], "open");
    }
    assert_eq!(std::fs::read_to_string(root.join("gh.posts")).unwrap(), "x");
    assert!(run(&root, &["stop"]).0);
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(run(&root, &["start", "--secret-backend", "user-file"]).0);
    let after = cmd(&root, "create", "create", create);
    assert_eq!(after["task_id"], created["task_id"]);
    assert_eq!(std::fs::read_to_string(root.join("gh.posts")).unwrap(), "x");
    let (ok, board) = run(&root, &["task", "board", "B", sid.as_str()]);
    assert!(ok, "{board}");
    assert_eq!(board["cards"][0]["claimed"], true);
    assert!(!root.join("hosted-consumed.json").exists());
    drop(fx);
}

/// Stop the daemon so this process can hold the store's exclusive lock and read it.
fn open_stopped(root: &Path) -> Store {
    assert!(run(root, &["stop"]).0);
    thread::sleep(Duration::from_millis(200));
    Store::open(root).unwrap()
}

#[test]
fn completion_crosses_the_real_command_line_and_writes_one_receipt() {
    let fx = fixture("complete", &[]);
    let root = fx.root.clone();
    let adopted = adopt(&fx);
    let action = completion_action(&adopted, &actor_principal(&root));
    // 验收 1：预览逐项列出验收项、判定者、校验等级和所用的证据。
    let preview_path = root.join("complete-preview.json");
    std::fs::write(&preview_path, action.to_string()).unwrap();
    let (ok, preview) = run(
        &root,
        &[
            "task",
            "complete",
            "--key",
            "complete",
            "--input",
            preview_path.to_str().unwrap(),
        ],
    );
    assert!(ok, "{preview}");
    let items = &preview["effect_summary"]["result"]["items"];
    assert_eq!(items[0]["text"], "a mechanical check passed", "{preview}");
    assert_eq!(items[0]["grade"], "mechanical");
    assert_eq!(items[0]["validation_level"], "unmediated");
    assert_eq!(items[0]["judge"]["kind"], "hctl2_tool");
    assert_eq!(items[0]["references"].as_array().unwrap().len(), 1);
    assert_eq!(items[1]["text"], "a human judged it done");
    assert_eq!(items[1]["grade"], "human");
    assert_eq!(items[1]["judge"]["kind"], "human");
    let completed = cmd(&root, "complete", "complete", action.clone());
    assert_eq!(completed["lifecycle"], "completed", "{completed}");
    let receipt_id = completed["receipt_id"]
        .as_str()
        .unwrap_or_else(|| panic!("receipt_id missing: {completed}"))
        .to_string();
    assert_eq!(completed["lifecycle_version"], 2, "{completed}");
    // The same key must replay the frozen result, never write a second completion.
    let replay = cmd(&root, "complete", "complete", action);
    assert_eq!(replay["receipt_id"], completed["receipt_id"]);
    assert_eq!(replay["lifecycle_version"], 2, "{replay}");
    let s = open_stopped(&root);
    let receipts = s.list(task::COMPLETION_RECEIPT_KIND).unwrap();
    assert_eq!(receipts.len(), 1, "exactly one receipt: {receipts:?}");
    let receipt = task::decode::<task::CompletionReceipt>(&receipts[0]).unwrap();
    assert_eq!(receipt.receipt_id, receipt_id);
    assert_eq!(receipt.task_id, adopted.task_id);
    assert_eq!(receipt.lifecycle_version, 2);
    assert_eq!(receipt.items.len(), 2);
    assert_eq!(receipt.items[0].grade, "mechanical");
    assert_eq!(receipt.items[0].validation_level, "unmediated");
    assert_eq!(receipt.items[0].generation, Some(7));
    assert_eq!(receipt.items[1].grade, "human");
    drop(s);
}

#[test]
fn completion_failure_inside_the_transaction_leaves_nothing() {
    let fx = fixture("fail", &[("HCTL_TASK_COMPLETE_FAIL", "after_receipt")]);
    let root = fx.root.clone();
    let adopted = adopt(&fx);
    let action = completion_action(&adopted, &actor_principal(&root));
    let (ok, failure) = cmd_result(&root, "complete", "complete", action);
    assert!(!ok, "injected failure must not succeed: {failure}");
    assert_eq!(failure["error"]["code"], "INJECTED_FAILURE", "{failure}");
    let s = open_stopped(&root);
    assert!(
        s.list(task::COMPLETION_RECEIPT_KIND).unwrap().is_empty(),
        "no receipt may survive the aborted transaction"
    );
    let (_, t) = task::task(&s, "A", &adopted.task_id).unwrap();
    assert_eq!(t.lifecycle, "open");
    assert_eq!(t.lifecycle_version, 1);
    drop(s);
}

#[test]
fn reopen_crosses_the_real_command_line_and_keeps_the_receipt() {
    let fx = fixture("reopen", &[]);
    let root = fx.root.clone();
    let adopted = adopt(&fx);
    let completed = cmd(
        &root,
        "complete",
        "complete",
        completion_action(&adopted, &actor_principal(&root)),
    );
    assert_eq!(completed["lifecycle"], "completed", "{completed}");
    let receipt_id = completed["receipt_id"].as_str().unwrap().to_string();
    let version = completed["task"]["version"].as_i64().unwrap();
    let lifecycle_version = completed["task"]["data"]["lifecycle_version"]
        .as_i64()
        .unwrap();
    assert_eq!(lifecycle_version, 2, "{completed}");
    let reopened = cmd(
        &root,
        "reopen",
        "reopen",
        json!({
            "project_id":"A","task_id":adopted.task_id.clone(),"version":version,
            "lifecycle_version":lifecycle_version,"revision_number":adopted.revision_number,
        }),
    );
    assert_eq!(reopened["lifecycle"], "open", "{reopened}");
    assert_eq!(reopened["lifecycle_version"], 3, "{reopened}");
    let s = open_stopped(&root);
    assert!(
        s.get(&task::key(
            Scope::Project("A".into()),
            task::COMPLETION_RECEIPT_KIND,
            &receipt_id,
        ))
        .unwrap()
        .is_some(),
        "reopen must keep the completion receipt"
    );
    let (_, t) = task::task(&s, "A", &adopted.task_id).unwrap();
    assert_eq!(t.lifecycle, "open");
    assert_eq!(t.lifecycle_version, 3);
    drop(s);
}

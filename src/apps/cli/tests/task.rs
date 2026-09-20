//! Project is a fixture until package 辛; every Task operation crosses the real CLI/RPC.
use serde_json::{Value, json};
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};
use store::{
    Actor, ActorSource, Expected, ProjectSettings, Record, RecordData, Scope, Store, TrustedActor,
};
struct Temp(PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = run(&self.0, &["stop"]);
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn run(root: &Path, args: &[&str]) -> (bool, Value) {
    let out = Command::new(env!("CARGO_BIN_EXE_hctl2"))
        .env("HCTL2_CONTROL_BIN", env!("CARGO_BIN_EXE_hctl2-control"))
        .env("HCTL2_GH", root.join("gh"))
        .args(["--json", "--root", root.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap();
    (out.status.success(),serde_json::from_slice(&out.stdout).unwrap_or_else(|_|json!({"stdout":String::from_utf8_lossy(&out.stdout),"stderr":String::from_utf8_lossy(&out.stderr)})))
}
fn cmd(root: &Path, kind: &str, key: &str, action: Value) -> Value {
    let path = root.join(format!("{key}.json"));
    std::fs::write(&path, action.to_string()).unwrap();
    let p = path.to_str().unwrap();
    let (ok, preview) = run(root, &["task", kind, "--key", key, "--input", p]);
    assert!(ok, "preview {kind}: {preview}");
    let (ok, result) = run(
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
    );
    assert!(ok, "submit {kind}: {result}");
    result
}
#[test]
fn task_commands_cross_live_daemon_preview_replay_and_restart() {
    let root = std::env::temp_dir().join(format!("hctl-task-cli-{}", std::process::id()));
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
if [ "$header" = yes ]; then printf 'HTTP/1.1 200 OK\n\n'; fi
case "$method:$path" in
 GET:user) printf '{"id":5}' ;;
 GET:repos/owner/fixture) printf '{"id":77,"full_name":"owner/fixture","clone_url":"https://github.com/owner/fixture.git","has_issues":true,"permissions":{"push":true,"admin":true}}' ;;
 GET:*page=2*) printf '[]' ;;
 GET:*/issues\?*) if [ -f "$0.created" ]; then jqbin="$(cat "$0.jq")"; "$jqbin" -s '.' "$0.created"; else printf '[]'; fi ;;
 GET:*/issues/1/parent) if [ "$header" = yes ]; then :; fi; printf 'null' ;;
 GET:*/issues/1) cat "$0.created" ;;
 GET:*) printf '[]' ;;
 POST:*/issues) jqbin="$(cat "$0.jq")"; "$jqbin" '. + {id:1,node_id:"node1",number:1,state:"open",updated_at:"t0",labels:[],comments:0}' > "$0.created"; printf x >> "$0.posts"; cat "$0.created" ;;
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
    drop(s);
    assert!(run(&root, &["start"]).0);
    let source = cmd(
        &root,
        "connect",
        "connect",
        json!({"repo_id":reg.repo_id,"candidate_id":"github_issues","consent":true,"make_default":false}),
    );
    let sid = source["source_id"].as_str().unwrap();
    for p in ["A", "B"] {
        cmd(
            &root,
            "attach",
            &format!("attach-{p}"),
            json!({"project_id":p,"project_version":1,"source_id":sid,"approved_scope":"77","consent":true}),
        );
    }
    let create =
        json!({"project_id":"A","project_version":1,"source_id":sid,"title":"new","body":"body"});
    let created = cmd(&root, "create", "create", create.clone());
    assert_eq!(created["effect_state"], "confirmed");
    assert_eq!(created["task"]["data"]["lifecycle"], "open");
    let duplicate = cmd(&root, "create", "create", create.clone());
    assert_eq!(created["task_id"], duplicate["task_id"]);
    let b = cmd(
        &root,
        "claim",
        "claim-b",
        json!({"project_id":"B","project_version":1,"source_id":sid,"entity_id":"node1"}),
    );
    assert_ne!(b["task_id"], created["task_id"]);
    assert_eq!(std::fs::read_to_string(root.join("gh.posts")).unwrap(), "x");
    assert!(run(&root, &["stop"]).0);
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(run(&root, &["start"]).0);
    let after = cmd(&root, "create", "create", create);
    assert_eq!(after["task_id"], created["task_id"]);
    assert_eq!(std::fs::read_to_string(root.join("gh.posts")).unwrap(), "x");
    let (ok, board) = run(&root, &["task", "board", "B", sid]);
    assert!(ok, "{board}");
    assert_eq!(board["cards"][0]["claimed"], true);
    assert!(!root.join("hosted-consumed.json").exists());
    drop(temp);
}

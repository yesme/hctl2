//! A real CLI -> control daemon -> native Git -> admitted human version.
use super::{Temp, run};
use serde_json::{Value, json};
use std::{path::Path, process::Command};

fn git(path: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(path)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_AUTHOR_NAME", "fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.test")
        .env("GIT_COMMITTER_NAME", "fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.test")
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().into()
}
fn ok(root: &Path, args: &[&str]) -> Value {
    let (success, out, err) = run(root, args);
    assert!(success, "{args:?}: {out} {err}");
    if out.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&out).unwrap()
    }
}

#[test]
fn human_seal_local_input_failures_are_stdout_json_and_nonzero() {
    let temp = Temp::new();
    let path = temp.0.join("input.json");
    for input in [None, Some("{"), Some("[]"), Some("{\"key\":\"other\"}")] {
        if let Some(input) = input {
            std::fs::write(&path, input).unwrap();
        }
        let (success, out, _) = run(
            &temp.0,
            &[
                "changeset",
                "seal",
                "--input",
                path.to_str().unwrap(),
                "--key",
                "seal",
            ],
        );
        assert!(!success, "{out}");
        let error: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(error["error"]["code"], "CHANGESET_COMMAND_FAILED");
        assert_eq!(error["error"]["recovery_action"], "inspect_error_and_retry");
    }
}

#[test]
fn human_seal_real_cli_reads_exact_git_identity_diff_and_replays_after_input_disappears() {
    let temp = Temp::new();
    let root = temp.0.clone();
    let path = temp.0.join("repo");
    std::fs::create_dir_all(&path).unwrap();
    git(&path, &["init", "--initial-branch=main"]);
    std::fs::write(path.join("answer"), "before\n").unwrap();
    git(&path, &["add", "answer"]);
    git(&path, &["commit", "-m", "base"]);
    let base = git(&path, &["rev-parse", "HEAD"]);
    std::fs::write(path.join("answer"), "after\n").unwrap();
    git(&path, &["commit", "-am", "result"]);
    let commit = git(&path, &["rev-parse", "HEAD"]);
    let tree = git(&path, &["rev-parse", "HEAD^{tree}"]);
    ok(&root, &["init", "--secret-backend", "user-file"]);
    // The Repo precondition is registered with the domain, not forged JSON. The
    // operation under test below always travels through CLI/RPC and native Git.
    let repo_id = {
        let mut s = store::Store::open(&root).unwrap();
        let actor = store::TrustedActor(store::Actor {
            principal: "owner".into(),
            source: store::ActorSource::DirectClient,
            permission_scope: vec![store::Scope::Control],
            authority: None,
        });
        let local = repo::LocalInput {
            machine: "control".into(),
            path: path.clone(),
            in_place: false,
            extra_refs: vec![],
            publish_governance: false,
        };
        let snapshot = repo::git::Git::discover().unwrap().inspect(&local).unwrap();
        let prepared = repo::prepare(
            repo::Register {
                name: "human-seal".into(),
                origin: repo::Origin::Local,
                platform: Some(repo::Platform::None),
                instance: None,
                platform_repo_id: None,
                platform_path: None,
                local: Some(local),
                remote_evidence: None,
                default_source: None,
            },
            Some(snapshot),
        )
        .unwrap();
        repo::admit(&mut s, &actor, "repo", "repo", prepared)
            .unwrap()
            .repo_id
    };
    ok(&root, &["start"]);
    let input = temp.0.join("seal.json");
    std::fs::write(
        &input,
        json!({"repo_id":repo_id,"base_commit_sha":base,
        "location":{"kind":"commit","repo_path":path,"commit_sha":commit}})
        .to_string(),
    )
    .unwrap();
    let args = [
        "changeset",
        "seal",
        "--input",
        input.to_str().unwrap(),
        "--key",
        "human",
    ];
    let plan = ok(&root, &args);
    let id = plan["effect_summary"]["plan"]["change_set"]["change_set_id"]
        .as_str()
        .unwrap();
    let (accepted, out, _) = run(&root, &["changeset", "show", &repo_id, id]);
    assert!(!accepted);
    assert_eq!(
        serde_json::from_str::<Value>(&out).unwrap()["error"]["code"],
        "CHANGESET_NOT_FOUND"
    );
    let mut submit = args.to_vec();
    submit.extend(["--preview-token", plan["preview_token"].as_str().unwrap()]);
    let receipt = ok(&root, &submit);
    assert_eq!(receipt["revision"]["base_commit_sha"], base);
    assert_eq!(receipt["revision"]["result_tree_sha"], tree);
    assert_eq!(
        receipt["revision"]["producer_ref"],
        json!({"kind":"human_command","command_id":"changeset:human"})
    );
    assert!(receipt["seal"]["lease"].is_null());
    let shown = ok(&root, &["changeset", "show", &repo_id, id]);
    assert_eq!(shown["revisions"].as_array().unwrap().len(), 1);
    let revision = receipt["revision"]["change_set_revision_id"]
        .as_str()
        .unwrap();
    let diff = ok(&root, &["changeset", "diff", &repo_id, id, revision]);
    assert!(diff["diff"].as_str().unwrap().contains("-before\n+after"));
    // No surviving source directory is needed to replay an already admitted command.
    std::fs::rename(&path, temp.0.join("moved-input")).unwrap();
    // RPC previews are single-use. Fetching a fresh preview of this same command
    // reads its receipt, not the vanished worktree, and creates no new revision.
    let replay_plan = ok(&root, &args);
    let mut replay = args.to_vec();
    replay.extend([
        "--preview-token",
        replay_plan["preview_token"].as_str().unwrap(),
    ]);
    assert_eq!(ok(&root, &replay), receipt);
    ok(&root, &["stop"]);
    ok(&root, &["start"]);
    let replay_plan = ok(&root, &args);
    let mut replay = args.to_vec();
    replay.extend([
        "--preview-token",
        replay_plan["preview_token"].as_str().unwrap(),
    ]);
    assert_eq!(ok(&root, &replay), receipt);
    ok(&root, &["stop"]);
}

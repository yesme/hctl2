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
    if args == ["stop"] {
        // The current CLI acknowledges the stop request before the daemon exits.
        // Restart/store-open fixtures wait for the original listener to disappear;
        // they do not retry the next command or change production shutdown semantics.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while std::os::unix::net::UnixStream::connect(root.join("control.sock")).is_ok() {
            assert!(std::time::Instant::now() < deadline, "control did not stop");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    if out.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&out).unwrap()
    }
}

fn owner(repo_id: Option<&str>) -> store::TrustedActor {
    let mut scope = vec![store::Scope::Control];
    if let Some(repo_id) = repo_id {
        scope.push(store::Scope::Repo(repo_id.into()));
    }
    store::TrustedActor(store::Actor {
        principal: "owner".into(),
        source: store::ActorSource::DirectClient,
        permission_scope: scope,
        authority: None,
    })
}
fn registered(store: &mut store::Store, path: &Path) -> String {
    let local = repo::LocalInput {
        machine: "control".into(),
        path: path.into(),
        in_place: false,
        extra_refs: vec![],
        publish_governance: false,
    };
    let snapshot = repo::git::Git::discover().unwrap().inspect(&local).unwrap();
    let prepared = repo::prepare(
        repo::Register {
            name: "residual-fixture".into(),
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
    repo::admit(store, &owner(None), "repo", "repo", prepared)
        .unwrap()
        .repo_id
}
fn native_tool(args: &[&str]) -> Value {
    let out = Command::new(std::env::var_os("HCTL2_TEST_TOOL").expect("native tool target"))
        .args(args)
        .output()
        .unwrap();
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(out.status.success(), "{value}");
    value
}

#[test]
fn residual_takeover_real_cli_seals_without_regranting_the_original_lease() {
    residual_chain("takeover", false, false, false);
}
#[test]
fn residual_adopt_real_cli_creates_a_human_version_in_another_changeset() {
    residual_chain("adopt", false, false, false);
}
#[test]
fn residual_discard_real_cli_requires_confirmation_and_survives_response_loss() {
    residual_chain("discard", false, false, false);
}
#[test]
fn residual_discard_real_cli_rejects_a_tree_changed_since_confirmation() {
    residual_chain("discard", true, false, false);
}

#[test]
fn residual_unknown_real_cli_reads_back_only_and_recovers_after_native_cleanup() {
    residual_chain("discard", false, true, false);
}

#[test]
fn residual_discard_real_cli_does_not_call_a_moved_detached_worktree_deleted() {
    residual_chain("discard", false, false, true);
}

fn residual_chain(
    action: &str,
    change_after_preview: bool,
    crash_before_delivery: bool,
    relocate_before_confirmation: bool,
) {
    let temp = Temp::new();
    let root = &temp.0;
    let path = root.join("repo");
    std::fs::create_dir(&path).unwrap();
    git(&path, &["init", "--initial-branch=main"]);
    std::fs::write(path.join("answer"), "before\n").unwrap();
    git(&path, &["add", "answer"]);
    git(&path, &["commit", "-m", "base"]);
    let base = git(&path, &["rev-parse", "HEAD"]);
    ok(root, &["init", "--secret-backend", "user-file"]);
    let (repo_id, source) = {
        let mut store = store::Store::open(root).unwrap();
        let repo_id = registered(&mut store, &path);
        let actor = owner(Some(&repo_id));
        let set = repo::changeset::open_change_set(
            &mut store,
            &actor,
            &repo_id,
            0,
            &base,
            "writer",
            &repo::changeset::ProducerRef::Invocation {
                invocation_id: "lost-writer".into(),
                invocation_version: 1,
            },
        )
        .unwrap();
        let input = json!({"change_set":set.change_set_id});
        let target = store::ObjectKey {
            scope: store::Scope::Repo(repo_id.clone()),
            kind: "fixture".into(),
            id: "revoke".into(),
        };
        let command = store::Command {
            command_id: "revoke".into(),
            idempotency_key: "revoke".into(),
            actor: actor.0.clone(),
            target: target.clone(),
            expected: store::Expected::Absent,
            binding: store::Reference {
                key: target,
                version: store::Version::State(1),
            },
            operation: "fixture.revoke".into(),
            input_digest: store::Command::digest_input("fixture.revoke", &input).unwrap(),
            input,
        };
        store
            .submit(store.generation(), &actor, &command, None, |tx| {
                repo::changeset::revoke_lease(
                    tx,
                    &repo_id,
                    &set.change_set_id,
                    &repo::changeset::LeaseRef {
                        lease_id: set.lease.lease_id.clone(),
                        generation: set.lease.generation,
                    },
                    &set.lease.holder,
                )?;
                Ok(json!({}))
            })
            .unwrap();
        (repo_id, set.change_set_id)
    };
    let worktrees = root.join("sites");
    let materialized = native_tool(&[
        "worktree",
        "materialize",
        "--repo",
        path.to_str().unwrap(),
        "--root",
        worktrees.to_str().unwrap(),
        "--change-set-ref",
        &source,
        "--baseline",
        &base,
    ]);
    assert_eq!(materialized["outcome"], "established");
    let worktree = worktrees.join(&source).canonicalize().unwrap();
    std::fs::write(worktree.join("unique-untracked"), "do not lose this\n").unwrap();
    ok(root, &["start"]);
    let input = root.join("residual.json");
    std::fs::write(
        &input,
        json!({"repo_id":repo_id,"change_set_id":source,"repo_path":worktree}).to_string(),
    )
    .unwrap();
    let args = [
        "changeset",
        action,
        "--input",
        input.to_str().unwrap(),
        "--key",
        "recover",
    ];
    if !change_after_preview && !crash_before_delivery && !relocate_before_confirmation {
        let original: Value = serde_json::from_slice(&std::fs::read(&input).unwrap()).unwrap();
        let mut wrong_path = original.clone();
        wrong_path["repo_path"] = json!("relative/path");
        let mut wrong_target = original.clone();
        wrong_target["target_change_set_id"] = json!(source);
        let mut unknown_field = original.clone();
        unknown_field["lease"] = json!({"generation":99});
        let mut invalid = vec![
            (wrong_path, "INVALID_INPUT"),
            (wrong_target, "INVALID_INPUT"),
            (unknown_field, "INVALID_JSON"),
        ];
        if action == "discard" {
            let mut wrong_parent = original.clone();
            wrong_parent["parent_revision_id"] = json!("not-a-version-for-discard");
            invalid.push((wrong_parent, "INVALID_INPUT"));
        }
        for (payload, code) in invalid {
            std::fs::write(&input, payload.to_string()).unwrap();
            let (success, out, _) = run(root, &args);
            assert!(!success, "{action}: {payload}: {out}");
            assert_eq!(
                serde_json::from_str::<Value>(&out).unwrap()["error"]["code"],
                code,
                "{payload}"
            );
        }
        std::fs::write(&input, original.to_string()).unwrap();
    }
    let mut plan = ok(root, &args);
    assert!(worktree.exists(), "preview must not delete");
    let before = ok(root, &["changeset", "show", &repo_id, &source]);
    assert!(
        before["residuals"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["worktree_path"] == worktree.to_str().unwrap())
    );
    assert!(before["revisions"].as_array().unwrap().is_empty());
    assert_eq!(before["change_set"]["lease"]["state"], "revoking");
    if crash_before_delivery {
        ok(root, &["stop"]);
        let details = &plan["effect_summary"];
        let source_ref = serde_json::from_value(details["source"].clone()).unwrap();
        let repo_ref = serde_json::from_value(details["repo"].clone()).unwrap();
        let mut store = store::Store::open(root).unwrap();
        let saved = repo::changeset::residual::begin(
            &mut store,
            &owner(Some(&repo_id)),
            &repo_id,
            "recover",
            details,
            &source_ref,
            &repo_ref,
        )
        .unwrap();
        store
            .begin_effect(store.generation(), saved["effect_id"].as_str().unwrap())
            .unwrap();
        drop(store);
        ok(root, &["start"]);
        plan = ok(root, &args);
        let mut confirm = args.to_vec();
        confirm.extend(["--preview-token", plan["preview_token"].as_str().unwrap()]);
        let (success, out, _) = run(root, &confirm);
        assert!(!success, "{out}");
        assert_eq!(
            serde_json::from_str::<Value>(&out).unwrap()["error"]["code"],
            "RESULT_UNKNOWN"
        );
        assert!(
            worktree.join("unique-untracked").exists(),
            "unknown effect must not resend deletion"
        );
        let tree = plan["effect_summary"]["source_observation"]["result_tree_sha"]
            .as_str()
            .unwrap();
        native_tool(&[
            "archive",
            "remove",
            "--repo",
            path.to_str().unwrap(),
            "--change-set-ref",
            &source,
            "--discard-unarchived",
            "--confirm-discard",
            tree,
            "--expected-worktree",
            worktree.to_str().unwrap(),
        ]);
        plan = ok(root, &args);
    }
    if change_after_preview {
        std::fs::write(worktree.join("late-edit"), "late\n").unwrap();
    }
    let relocated = root.join("moved-residual");
    if relocate_before_confirmation {
        git(&worktree, &["checkout", "--detach"]);
        git(
            &path,
            &[
                "worktree",
                "move",
                worktree.to_str().unwrap(),
                relocated.to_str().unwrap(),
            ],
        );
    }
    let mut confirm = args.to_vec();
    confirm.extend(["--preview-token", plan["preview_token"].as_str().unwrap()]);
    if relocate_before_confirmation {
        let (success, out, _) = run(root, &confirm);
        assert!(!success, "{out}");
        assert_eq!(
            serde_json::from_str::<Value>(&out).unwrap()["error"]["code"],
            "RESIDUAL_PATH_CHANGED"
        );
        assert!(relocated.join("unique-untracked").exists());
        let shown = ok(root, &["changeset", "show", &repo_id, &source]);
        assert_eq!(shown["residuals"][0]["status"], "rejected");
        ok(root, &["stop"]);
        return;
    }
    if change_after_preview {
        let (success, out, _) = run(root, &confirm);
        assert!(!success, "{out}");
        assert_eq!(
            serde_json::from_str::<Value>(&out).unwrap()["error"]["code"],
            "GIT_SEAL_FAILED"
        );
        assert!(worktree.join("unique-untracked").exists());
        let shown = ok(root, &["changeset", "show", &repo_id, &source]);
        assert_eq!(shown["residuals"][0]["status"], "rejected");
        assert_eq!(
            shown["residuals"][0]["plan"]["worktree_path"],
            worktree.to_str().unwrap()
        );
        ok(root, &["stop"]);
        return;
    }
    let receipt = ok(root, &confirm);
    let shown = ok(root, &["changeset", "show", &repo_id, &source]);
    assert_eq!(
        shown["change_set"]["lease"]["state"], "revoking",
        "no fake physical-stop proof"
    );
    if action == "discard" {
        assert_eq!(receipt["status"], "discarded");
        assert!(!worktree.exists());
    } else {
        let target = receipt["revision"]["change_set_id"].as_str().unwrap();
        assert_eq!(target == source, action == "takeover");
        assert_eq!(receipt["revision"]["producer_ref"]["kind"], "human_command");
        assert!(receipt["seal"]["lease"].is_null());
        let adopted = ok(root, &["changeset", "show", &repo_id, target]);
        assert_eq!(adopted["revisions"].as_array().unwrap().len(), 1);
        let diff = ok(
            root,
            &[
                "changeset",
                "diff",
                &repo_id,
                target,
                receipt["revision"]["change_set_revision_id"]
                    .as_str()
                    .unwrap(),
            ],
        );
        assert!(diff["diff"].as_str().unwrap().contains("do not lose this"));
    }
    ok(root, &["stop"]);
    ok(root, &["start"]);
    let repeated = ok(root, &args);
    let mut repeat = args.to_vec();
    repeat.extend([
        "--preview-token",
        repeated["preview_token"].as_str().unwrap(),
    ]);
    assert_eq!(ok(root, &repeat), receipt);
    ok(root, &["stop"]);
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

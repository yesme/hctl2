//! CLI wiring for init/start/status/doctor/export/backup/restore against a live daemon.

#[path = "cli/profile.rs"]
mod profile;

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static TEMPS: AtomicU64 = AtomicU64::new(0);

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "hctl2-cli-{}-{}",
            std::process::id(),
            TEMPS.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        if let Ok(pid) = std::fs::read_to_string(self.0.join("control.pid")) {
            let _ = Command::new("kill").arg(pid.trim()).status();
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// Read at run time, not compile time: the Clippy pass compiles these tests
// without the test rule's env, so `env!` would fail the whole Clippy report
// before it could say anything about lints.
fn hctl2() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_BIN_EXE_hctl2").expect("CARGO_BIN_EXE_hctl2 must be set"))
}

fn control() -> PathBuf {
    PathBuf::from(
        std::env::var("CARGO_BIN_EXE_hctl2-control")
            .expect("CARGO_BIN_EXE_hctl2-control must be set"),
    )
}

fn run(root: &std::path::Path, args: &[&str]) -> (bool, String, String) {
    let output = Command::new(hctl2())
        .env("HCTL2_CONTROL_BIN", control())
        .args(["--json", "--root", root.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn project_select_cli_checks_real_candidates_and_admits_only_confirmed_preview() {
    use agency_proto::{Capabilities, Catalog, FrozenRef, Profession, hash};
    use participant::profiles::{
        ProfileAction, ProfileInput, WorkerProfile, admit_profile, prepare_profile,
    };
    use participant::{key, reference, value};
    use serde_json::{Value, json};
    use store::{
        Actor, ActorSource, Command as StoreCommand, Expected, RecordData, Scope, TrustedActor,
    };

    let temp = Temp::new();
    let root = &temp.0;
    assert!(run(root, &["init", "--secret-backend", "user-file"]).0);
    let actor = TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control, Scope::Project("P".into())],
        authority: None,
    });
    // Only the preconditions are seeded. The selection command below uses the real CLI and RPC.
    let selection = {
        let mut store = store::Store::open(root).unwrap();
        let harness = FrozenRef {
            id: "script".into(),
            revision: "1".into(),
            digest: hash(b"script"),
        };
        let profession = Profession {
            reference: FrozenRef {
                id: "research".into(),
                revision: "1".into(),
                digest: hash(b"research"),
            },
            harness: harness.clone(),
            model: "fixture".into(),
            persona: "planner".into(),
            terms: "read only".into(),
            default_role: "planner".into(),
            skills: vec![],
            capabilities: Capabilities::default(),
        };
        let binding = participant::accept_binding(
            &mut store,
            &actor,
            "fixture-pair",
            participant::Binding {
                id: "script-agency".into(),
                protocol: agency_proto::PROTOCOL.into(),
                catalog: Catalog {
                    professions: vec![profession.clone()],
                    harnesses: vec![harness.clone()],
                    skills: vec![],
                },
            },
        )
        .unwrap();
        let accepted = participant::accept_profession(
            &mut store,
            &actor,
            "fixture-profession",
            "script-agency",
            &profession,
        )
        .unwrap();
        let profile = prepare_profile(
            &store,
            ProfileInput {
                key: "fixture-profile".into(),
                action: ProfileAction::Create {
                    id: "research".into(),
                    profile: WorkerProfile {
                        harness,
                        model: "fixture".into(),
                        mode: "read_only".into(),
                        permissions: vec!["context.read".into()],
                        environment: vec![],
                        required_capabilities: Capabilities::default(),
                        max_context_bytes: 65536,
                    },
                },
            },
            &actor,
        )
        .unwrap();
        let profile = admit_profile(&mut store, &actor, profile).unwrap();
        let mut project = value(
            key(Scope::Project("P".into()), "project", "P"),
            1,
            &json!({}),
        )
        .unwrap();
        project.data = RecordData::Project {
            repo_id: "repo-fixture".into(),
            archived: false,
            settings: store::ProjectSettings {
                selection_policy: json!({}),
                publish_review_requires_confirmation: false,
            },
        };
        let mut room = value(
            key(Scope::Project("P".into()), "room", "main"),
            1,
            &json!({}),
        )
        .unwrap();
        room.data = RecordData::Room {
            room_kind: store::RoomKind::Main,
            state: store::RoomState::Active,
        };
        let seed = StoreCommand {
            command_id: "fixture-project".into(),
            idempotency_key: "fixture-project".into(),
            actor: actor.0.clone(),
            target: project.key.clone(),
            expected: Expected::Absent,
            binding: reference(&project),
            operation: "fixture".into(),
            input: json!({}),
            input_digest: StoreCommand::digest_input("fixture", &json!({})).unwrap(),
        };
        let room_binding = value(
            key(Scope::Project("P".into()), "room_binding", "main"),
            1,
            &json!({"project_id":"P","id":"main","name":"Main","server":{
                "binding":{"key":key(Scope::Control,"chat_server","fixture"),"version":store::Version::State(1)},"url":"http://127.0.0.1:8008",
                "server_name":"fixture","sender":"@control:fixture"},
                "matrix_room_id":"!main:fixture","participants":[],"brief":null,"origin":null}),
        )
        .unwrap();
        store
            .submit(store.generation(), &actor, &seed, None, |tx| {
                tx.put(&project)?;
                tx.put(&room)?;
                tx.put(&room_binding)?;
                Ok(json!({}))
            })
            .unwrap();
        json!({"room_id":"main","selected_item":reference(&accepted),"profession":reference(&accepted),
            "profession_digest":profession.reference.digest,"agency":reference(&binding),
            "required_skills":[],"optional_skills":[],"worker_profiles":[profile["revision"]],
            "responsibility":"research","permission":{"allow":["context.read"]},
            "budget":{"max_bytes":65536},"display_name":"researcher","persona_tags":[]})
    };
    let (ok, out, err) = run(root, &["start"]);
    assert!(ok, "{out} {err}");
    let input = root.join("selection.json");
    let action = json!({"project_id":"P","project_version":1,"room_id":"main",
        "topic_command_key":null,"roster_version":null,"selections":[selection]});
    let mut bad = action.clone();
    bad["selections"][0]["agency"]["key"]["id"] = json!("never-accepted");
    std::fs::write(&input, serde_json::to_vec(&bad).unwrap()).unwrap();
    let args = [
        "project",
        "select",
        "--input",
        input.to_str().unwrap(),
        "--key",
        "select-one",
    ];
    let (ok, out, err) = run(root, &args);
    assert!(!ok, "{out} {err}");
    let rejected: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(rejected["error"]["code"], "CANDIDATE_NOT_ACCEPTED");
    assert_eq!(rejected["error"]["recovery_action"], "accept_candidate");
    std::fs::write(&input, serde_json::to_vec(&action).unwrap()).unwrap();
    let (ok, out, err) = run(root, &args);
    assert!(ok, "{out} {err}");
    let preview: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        preview["effect_summary"]["result"]["candidate_validation"],
        "accepted_catalog_and_project_policy"
    );
    let (ok, out, err) = run(root, &["project", "roster", "P", "main"]);
    assert!(ok, "{out} {err}");
    assert!(
        serde_json::from_str::<Value>(&out).unwrap()["selections"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let mut confirmed = args.to_vec();
    confirmed.extend([
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    let (ok, out, err) = run(root, &confirmed);
    assert!(ok, "{out} {err}");
    let admitted: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(admitted["roster_version"], 1);
    let (ok, out, err) = run(root, &["project", "roster", "P", "main"]);
    assert!(ok, "{out} {err}");
    let roster: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(roster["selections"].as_array().unwrap().len(), 1);
}

#[test]
fn context_queries_reach_rpc_and_show_verified_frozen_bytes() {
    use agency_proto::{FrozenRef, Owner, OwnerKind, hash};
    use context::{
        Assembler, AssemblyRequest, LocalAssembler, Manifest, MemorySources, SourceKind,
    };
    use serde_json::json;
    let temp = Temp::new();
    let root = temp.0.as_path();
    assert!(run(root, &["init", "--secret-backend", "user-file"]).0);
    let actor = store::TrustedActor(store::Actor {
        principal: "owner".into(),
        source: store::ActorSource::DirectClient,
        permission_scope: vec![store::Scope::Control, store::Scope::Project("A".into())],
        authority: None,
    });
    let source = FrozenRef {
        id: "chat_source_reference/fixture".into(),
        revision: "1".into(),
        digest: hash(b"fixture"),
    };
    let permitted = vec![source.id.clone()];
    let mut sources = MemorySources::new();
    sources.push(SourceKind::Room, source.clone(), b"frozen CLI material");
    let manifest = Manifest {
        id: "cli-manifest".into(),
        purpose: "RPC fixture".into(),
        scope: "project A".into(),
        parent: None,
        sources: vec![source.clone()],
        selection_policy: source.clone(),
        freshness: "frozen".into(),
        coverage: "fixture".into(),
        known_gaps: vec![],
        required_skills: vec![],
        permission_digest: context::permission_digest(&permitted),
        redaction: source,
        budget: 1024,
    };
    let assembly = LocalAssembler {
        permitted: permitted.into_iter().collect(),
        budget: 1024,
    }
    .assemble(
        &sources,
        AssemblyRequest {
            manifest,
            consumer: Owner {
                project: "A".into(),
                kind: OwnerKind::RoomInvocation,
                id: "invoke-cli".into(),
                generation: 2,
            },
        },
    )
    .unwrap();
    {
        let mut store = store::Store::open(root).unwrap();
        context::save_assembly(&mut store, &actor, "A", "cli-context-fixture", &assembly).unwrap();
    }
    assert!(run(root, &["start"]).0);
    let (ok, out, err) = run(
        root,
        &[
            "context",
            "show",
            "A",
            "--manifest-id",
            "cli-manifest",
            "--bundle-id",
            &assembly.bundle.document.id,
        ],
    );
    assert!(ok, "{out} {err}");
    let shown: serde_json::Value = serde_json::from_str(&out).unwrap();
    let bundle: agency_proto::Sealed<context::Bundle> =
        serde_json::from_value(shown["bundle"].clone()).unwrap();
    bundle.verify().unwrap();
    assert_eq!(bundle, assembly.bundle);
    let input = root.join("preview-input.json");
    std::fs::write(
        &input,
        serde_json::to_vec(&json!({"project_id":"A","room_id":"missing-room"})).unwrap(),
    )
    .unwrap();
    let (ok, out, _) = run(
        root,
        &["context", "preview", "--input", input.to_str().unwrap()],
    );
    assert!(!ok);
    let error: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(error["error"]["code"], "NOT_FOUND");
    assert!(!out.contains("unknown query"));
    std::fs::write(
        &input,
        serde_json::to_vec(
            &json!({"project_id":"A","room_id":"missing-room","consumer":{
        "project":"B","kind":"room_invocation","id":"i","generation":1}}),
        )
        .unwrap(),
    )
    .unwrap();
    let (ok, out, _) = run(
        root,
        &["context", "preview", "--input", input.to_str().unwrap()],
    );
    assert!(!ok);
    assert!(out.contains("consumer Project differs"), "{out}");
}

#[test]
fn secret_backend_setting_persists_and_unknown_values_are_rejected() {
    let temp = Temp::new();
    let root = temp.0.as_path();
    let (ok, stdout, stderr) = run(root, &["init", "--secret-backend", "user-file"]);
    assert!(ok, "init {stderr} {stdout}");
    // Recorded by init, picked up by a later start that does not repeat the flag.
    let (ok, stdout, stderr) = run(root, &["start"]);
    assert!(ok, "start {stderr} {stdout}");
    let (ok, stdout, stderr) = run(root, &["status"]);
    assert!(ok, "status {stderr} {stdout}");
    assert!(
        stdout.contains("\"credential_storage\":\"user-file\""),
        "the recorded backend must survive a restart: {stdout}"
    );
    // An unknown backend is a usage error, not a silent fallback to the default.
    let other = Temp::new();
    let (ok, _, stderr) = run(&other.0, &["init", "--secret-backend", "keychain"]);
    assert!(!ok, "init must reject an unknown backend: {stderr}");
    let (ok, _, stderr) = run(&other.0, &["start", "--secret-backend", "keychain"]);
    assert!(!ok, "start must reject an unknown backend: {stderr}");
}

#[test]
fn start_records_the_backend_on_a_root_that_does_not_exist_yet() {
    // `start` creates the root itself, so recording a setting cannot assume the
    // directory is already there.
    let temp = Temp::new();
    let root = temp.0.join("fresh");
    let (ok, stdout, stderr) = run(&root, &["start", "--secret-backend", "user-file"]);
    assert!(ok, "start on a fresh root: {stderr} {stdout}");
    let (ok, stdout, stderr) = run(&root, &["status"]);
    assert!(ok, "status: {stderr} {stdout}");
    assert!(
        stdout.contains("\"credential_storage\":\"user-file\""),
        "the recorded backend must apply to the daemon that just started: {stdout}"
    );
    let (ok, _, stderr) = run(&root, &["stop"]);
    assert!(ok, "stop: {stderr}");
}

#[test]
fn project_and_request_local_failures_are_stdout_json_and_nonzero() {
    let temp = Temp::new();
    for namespace in ["project", "request"] {
        let (ok, stdout, _) = run(
            &temp.0,
            &[
                namespace,
                "create",
                "--key",
                "bad",
                "--input",
                temp.0.join("absent.json").to_str().unwrap(),
            ],
        );
        assert!(!ok);
        let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(value["error"]["code"], "PROJECT_COMMAND_FAILED");
        assert!(value["error"]["recovery_action"].is_string());
    }
}

#[test]
fn init_start_status_doctor_backup_restore_round_trip() {
    let temp = Temp::new();
    let root = temp.0.as_path();
    let (ok, stdout, stderr) = run(root, &["init"]);
    assert!(ok, "init {stderr} {stdout}");
    let (ok, stdout, stderr) = run(root, &["start", "--secret-backend", "user-file"]);
    assert!(ok, "start {stderr} {stdout}");
    let (ok, stdout, stderr) = run(root, &["status"]);
    assert!(ok, "status {stderr} {stdout}");
    assert!(stdout.contains("control_id"), "{stdout}");
    assert!(stdout.contains("local-owner"), "{stdout}");
    assert!(
        stdout.contains("\"credential_storage\":\"user-file\""),
        "status must report the configured backend: {stdout}"
    );
    let (ok, stdout, stderr) = run(root, &["doctor"]);
    assert!(ok, "doctor {stderr} {stdout}");
    assert!(stdout.contains("secret_backend"), "{stdout}");
    assert!(stdout.contains("loopback-unix-owner-only"), "{stdout}");
    let (ok, stdout, stderr) = run(root, &["export"]);
    assert!(ok, "export {stderr} {stdout}");
    assert!(stdout.contains("writer_generation"), "{stdout}");
    let backup = root.join("backup-1");
    let (ok, stdout, stderr) = run(root, &["backup", "create", backup.to_str().unwrap()]);
    assert!(ok, "backup create {stderr} {stdout}");
    let (ok, stdout, stderr) = run(root, &["backup", "verify", backup.to_str().unwrap()]);
    assert!(ok, "backup verify {stderr} {stdout}");
    let (ok, stdout, stderr) = run(root, &["restore", "preview", backup.to_str().unwrap()]);
    assert!(ok, "restore preview {stderr} {stdout}");
    assert!(
        stdout.contains("\"dangerous\":true") || stdout.contains("\"dangerous\": true"),
        "{stdout}"
    );
    let (ok, stdout, stderr) = run(root, &["query", "pending"]);
    assert!(ok, "pending {stderr} {stdout}");
    assert!(stdout.contains("items"), "{stdout}");
    let (ok, stdout, stderr) = run(
        root,
        &["restore", "apply", backup.to_str().unwrap(), "--yes"],
    );
    assert!(ok, "restore apply {stderr} {stdout}");
    let (ok, stdout, stderr) = run(root, &["services", "status"]);
    assert!(ok, "services status {stderr} {stdout}");
    let (ok, _, stderr) = run(root, &["stop"]);
    assert!(ok, "stop {stderr}");
}

fn register(root: &std::path::Path, arguments: &[&str]) -> (bool, serde_json::Value) {
    let mut args = vec!["repo", "register"];
    args.extend_from_slice(arguments);
    let (ok, out, err) = run(root, &args);
    assert!(ok, "preview failed: {out} {err}");
    let preview: serde_json::Value = serde_json::from_str(&out).unwrap();
    args.extend_from_slice(&[
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    let (ok, out, err) = run(root, &args);
    let value = serde_json::from_str(&out).unwrap_or_else(|_| panic!("invalid JSON: {out} {err}"));
    (ok, value)
}

#[test]
fn repo_commands_use_preview_persist_and_do_not_consume_gitea_for_external_or_none() {
    let temp = Temp::new();
    let root = &temp.0;
    let gh = root.join("gh-fixture");
    std::fs::write(&gh, "#!/bin/sh\ncase \"$6\" in\n user) printf '%s\\n' '{\"id\":9}' ;;\n repos/a/b) printf '%s\\n' '{\"id\":12,\"full_name\":\"a/b\",\"clone_url\":\"https://github.com/a/b.git\",\"has_issues\":true,\"permissions\":{\"push\":true}}' ;;\n *) exit 1 ;;\nesac\n").unwrap();
    std::fs::set_permissions(&gh, std::fs::Permissions::from_mode(0o700)).unwrap();
    let out = Command::new(hctl2())
        .env("HCTL2_CONTROL_BIN", control())
        .env("HCTL2_GH", &gh)
        .args([
            "--root",
            root.to_str().unwrap(),
            "start",
            "--secret-backend",
            "user-file",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let input = root.join("register.json");
    std::fs::write(&input,r#"{"name":"external","origin":"external","platform":"github","instance":"github.com","platform_repo_id":"12","platform_path":"a/b","default_source":"github_issues"}"#).unwrap();
    let args = ["--input", input.to_str().unwrap(), "--key", "external"];
    let (ok, external) = register(root, &args);
    assert!(ok, "{external}");
    let id = external["registration"]["repo_id"].as_str().unwrap();
    assert_eq!(external["registration"]["lifecycle"], "active");
    let (ok, replay) = register(root, &args);
    assert!(ok, "{replay}");
    assert_eq!(replay["registration"]["repo_id"], id);
    assert!(!root.join("hosted-consumed.json").exists());
    let (ok, out, err) = run(root, &["repo", "show", id]);
    assert!(ok, "{err}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&out).unwrap()["lifecycle"],
        "active"
    );
    // Wrong ID cannot be activated, and stays the same pending record on retry.
    std::fs::write(&input,r#"{"name":"wrong","origin":"external","platform":"github","instance":"github.com","platform_repo_id":"99","platform_path":"a/b"}"#).unwrap();
    let (ok, wrong) = register(
        root,
        &["--input", input.to_str().unwrap(), "--key", "wrong"],
    );
    assert!(!ok);
    assert_eq!(wrong["error"]["code"], "PLATFORM_ID_MISMATCH");
    assert_eq!(wrong["registration"]["lifecycle"], "pending");
    std::fs::write(&input,r#"{"name":"no platform","origin":"external","platform":"none","remote_evidence":"https://github.com/a/b.git"}"#).unwrap();
    let (ok, none) = register(root, &["--input", input.to_str().unwrap(), "--key", "none"]);
    assert!(ok, "{none}");
    assert_eq!(none["registration"]["prepared"]["platform"], "none");
    assert!(!root.join("hosted-consumed.json").exists());
    let (ok, out, err) = run(root, &["repo", "list"]);
    assert!(ok, "{err}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&out).unwrap()["items"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn repo_abandon_before_dispatch_releases_target_and_resume_never_restarts_it() {
    let temp = Temp::new();
    let root = &temp.0;
    let (ok, out, err) = run(root, &["start", "--secret-backend", "user-file"]);
    assert!(ok, "{out} {err}");
    let input = root.join("register.json");
    std::fs::write(
        &input,
        r#"{"name":"local","origin":"local","platform":"local","platform_path":"reusable"}"#,
    )
    .unwrap();
    let (ok, first) = register(
        root,
        &["--input", input.to_str().unwrap(), "--key", "first"],
    );
    assert!(!ok, "no installed Gitea package");
    assert_eq!(first["error"]["code"], "PLATFORM_NOT_INSTALLED");
    let id = first["registration"]["repo_id"].as_str().unwrap();
    let version = first["registration"]["version"].to_string();
    let (ok, abandoned) = register(
        root,
        &["--abandon", id, "--version", &version, "--key", "abandon"],
    );
    assert!(ok, "{abandoned}");
    assert_eq!(abandoned["abandoned"], true);
    let (ok, resumed) = register(root, &["--resume", id, "--key", "resume"]);
    assert!(
        ok,
        "cancelled registration must not try Gitea again: {resumed}"
    );
    assert_eq!(resumed["registration"], abandoned);
    let (ok, second) = register(
        root,
        &["--input", input.to_str().unwrap(), "--key", "second"],
    );
    assert!(!ok);
    assert_eq!(second["error"]["code"], "PLATFORM_NOT_INSTALLED");
    assert_ne!(second["registration"]["repo_id"], id);
    assert_eq!(second["registration"]["lifecycle"], "pending");
}

#[test]
fn repo_grant_refuses_an_external_registration_and_one_that_is_not_active() {
    let temp = Temp::new();
    let root = &temp.0;
    let gh = root.join("gh-fixture");
    std::fs::write(&gh, "#!/bin/sh\ncase \"$6\" in\n user) printf '%s\\n' '{\"id\":9}' ;;\n repos/a/b) printf '%s\\n' '{\"id\":12,\"full_name\":\"a/b\",\"clone_url\":\"https://github.com/a/b.git\",\"has_issues\":true,\"permissions\":{\"push\":true}}' ;;\n *) exit 1 ;;\nesac\n").unwrap();
    std::fs::set_permissions(&gh, std::fs::Permissions::from_mode(0o700)).unwrap();
    let out = Command::new(hctl2())
        .env("HCTL2_CONTROL_BIN", control())
        .env("HCTL2_GH", &gh)
        .args(["--root", root.to_str().unwrap(), "start"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let input = root.join("register.json");
    std::fs::write(&input,r#"{"name":"external","origin":"external","platform":"github","instance":"github.com","platform_repo_id":"12","platform_path":"a/b","default_source":"github_issues"}"#).unwrap();
    let (ok, external) = register(
        root,
        &["--input", input.to_str().unwrap(), "--key", "external"],
    );
    assert!(ok, "{external}");
    let external_id = external["registration"]["repo_id"].as_str().unwrap();
    std::fs::write(
        &input,
        r#"{"name":"local","origin":"local","platform":"local","platform_path":"reusable"}"#,
    )
    .unwrap();
    // This harness has no installed Gitea package, so the local registration stays pending.
    let (ok, local) = register(
        root,
        &["--input", input.to_str().unwrap(), "--key", "local"],
    );
    assert!(!ok, "{local}");
    assert_eq!(local["error"]["code"], "PLATFORM_NOT_INSTALLED");
    assert_eq!(local["registration"]["lifecycle"], "pending");
    let local_id = local["registration"]["repo_id"].as_str().unwrap();

    // GitHub keeps its own accounts: control creates accounts only on the hosted platform.
    let (ok, out, err) = run(
        root,
        &[
            "repo",
            "grant",
            "--repo-id",
            external_id,
            "--username",
            "alice",
            "--key",
            "grant-external",
        ],
    );
    assert!(!ok, "{out} {err}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&out).unwrap()["error"]["code"],
        "INVALID_INPUT"
    );

    // A pending registration has no platform repository to collaborate on yet.
    let (ok, out, err) = run(
        root,
        &[
            "repo",
            "grant",
            "--repo-id",
            local_id,
            "--username",
            "alice",
            "--key",
            "grant-pending",
        ],
    );
    assert!(!ok, "{out} {err}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&out).unwrap()["error"]["code"],
        "REPO_PENDING"
    );
}

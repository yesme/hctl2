//! Human-readable default output for lists, previews and errors, against a
//! live daemon started from the real `hctl2` binary. `--json` stays the
//! machine interface: the byte-stability test pins it against saved samples.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde_json::{Value, json};

static TEMPS: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "hctl2-human-{}-{}",
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

fn hctl2() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_BIN_EXE_hctl2").expect("CARGO_BIN_EXE_hctl2 must be set"))
}

fn control() -> PathBuf {
    PathBuf::from(
        std::env::var("CARGO_BIN_EXE_hctl2-control")
            .expect("CARGO_BIN_EXE_hctl2-control must be set"),
    )
}

/// Raw stdout bytes so comparisons are byte-exact, plus status and stderr.
fn run_bytes(root: &Path, json: bool, args: &[&str]) -> (bool, Vec<u8>, Vec<u8>) {
    let mut command = Command::new(hctl2());
    command.env("HCTL2_CONTROL_BIN", control()).arg("--root");
    command.arg(root.to_str().unwrap());
    if json {
        command.arg("--json");
    }
    command.args(args);
    let output = command.output().unwrap();
    (output.status.success(), output.stdout, output.stderr)
}

fn run(root: &Path, json: bool, args: &[&str]) -> (bool, String, String) {
    let (ok, stdout, stderr) = run_bytes(root, json, args);
    (
        ok,
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
    )
}

fn actor(project: &str) -> store::TrustedActor {
    store::TrustedActor(store::Actor {
        principal: "owner".into(),
        source: store::ActorSource::DirectClient,
        permission_scope: vec![store::Scope::Control, store::Scope::Project(project.into())],
        authority: None,
    })
}

/// Seed one Project with a Main room plus the Agency binding, accepted
/// Profession and Worker Profile a dispatch needs, the way the wiring test
/// above does, then start the daemon.
fn dispatch_fixture(root: &Path) -> (String, String) {
    use agency_proto::{Capabilities, Catalog, FrozenRef, Profession, hash};
    use participant::profiles::{
        ProfileAction, ProfileInput, WorkerProfile, admit_profile, prepare_profile,
    };
    use participant::{key, reference, value};
    use store::{Command as StoreCommand, Expected, RecordData};

    assert!(run(root, true, &["init", "--secret-backend", "user-file"]).0);
    let trusted = actor("P");
    #[allow(unused_assignments)]
    let mut seeded_profile = None;
    #[allow(unused_assignments)]
    let mut repo_id = String::new();
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
            &trusted,
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
            &trusted,
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
            &trusted,
        )
        .unwrap();
        let profile = admit_profile(&mut store, &trusted, profile).unwrap();
        seeded_profile = Some(profile["revision"].clone());
        let prepared = repo::prepare(
            repo::Register {
                name: "fixture".into(),
                origin: repo::Origin::Local,
                platform: Some(repo::Platform::None),
                instance: None,
                platform_repo_id: None,
                platform_path: None,
                local: None,
                remote_evidence: None,
                default_source: None,
            },
            None,
        )
        .unwrap();
        let registration = repo::admit(
            &mut store,
            &trusted,
            "fixture-repo",
            "fixture-repo",
            prepared,
        )
        .unwrap();
        repo_id = registration.repo_id;
        let mut project = value(
            key(store::Scope::Project("P".into()), "project", "P"),
            1,
            &json!({}),
        )
        .unwrap();
        project.data = RecordData::Project {
            repo_id,
            archived: false,
            settings: store::ProjectSettings {
                selection_policy: json!({}),
                publish_review_requires_confirmation: false,
            },
        };
        let mut room = value(
            key(store::Scope::Project("P".into()), "room", "main"),
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
            actor: trusted.0.clone(),
            target: project.key.clone(),
            expected: Expected::Absent,
            binding: reference(&project),
            operation: "fixture".into(),
            input: json!({}),
            input_digest: StoreCommand::digest_input("fixture", &json!({})).unwrap(),
        };
        let room_binding = value(
            key(store::Scope::Project("P".into()), "room_binding", "main"),
            1,
            &json!({"project_id":"P","id":"main","name":"Main","server":{
                "binding":{"key":key(store::Scope::Control,"chat_server","fixture"),"version":store::Version::State(1)},"url":"http://127.0.0.1:8008",
                "server_name":"fixture","sender":"@control:fixture"},
                "matrix_room_id":"!main:fixture","participants":[],"brief":null,"origin":null}),
        )
        .unwrap();
        store
            .submit(store.generation(), &trusted, &seed, None, |tx| {
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
    assert!(run(root, true, &["start"]).0);
    let input = root.join("selection.json");
    std::fs::write(
        &input,
        serde_json::to_vec(
            &json!({"project_id":"P","project_version":1,"room_id":"main",
            "topic_command_key":null,"roster_version":null,"selections":[selection]}),
        )
        .unwrap(),
    )
    .unwrap();
    let profile_revision = seeded_profile.expect("fixture seeds a profile revision");
    let invocation_input = root.join("invocation-input.json");
    std::fs::write(
        &invocation_input,
        serde_json::to_vec(
            &json!({"project_id":"P","room_id":"main","target":"research",
            "profile":profile_revision,"request":"Return the exact answer without modifying files",
            "budget":65536,"deadline_ms":4102444800000_u64,"retry_of":null}),
        )
        .unwrap(),
    )
    .unwrap();
    (
        input.display().to_string(),
        invocation_input.display().to_string(),
    )
}

#[path = "samples.rs"]
mod samples;

/// Byte samples of the machine interface, from `cli/samples.rs`: captured from
/// the CLI before the human renderers landed, pinned byte for byte. The random
/// preview token is masked identically on both sides before comparing.
const SAMPLES: &[(&str, &str)] = &[
    ("select-preview", samples::SELECT_PREVIEW),
    ("select-confirm", samples::SELECT_CONFIRM),
    ("room-hierarchy", samples::ROOM_HIERARCHY),
    ("room-list", samples::ROOM_LIST),
    ("task-list", samples::TASK_LIST),
    ("task-sources", samples::TASK_SOURCES),
    ("task-board-no-source", samples::TASK_BOARD_NO_SOURCE),
    ("profession-list", samples::PROFESSION_LIST),
    ("agency-bindings", samples::AGENCY_BINDINGS),
    ("error-not-found", samples::ERROR_NOT_FOUND),
];

fn mask_preview_token(output: &str) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(output)
        && let Some(token) = value["preview_token"].as_str()
    {
        return output.replace(token, "MASKED-PREVIEW-TOKEN");
    }
    output.to_owned()
}

fn sample_commands(path: &str) -> Vec<Vec<String>> {
    let select = vec![
        "project".to_owned(),
        "select".to_owned(),
        "--input".to_owned(),
        path.to_owned(),
        "--key".to_owned(),
        "golden-select".to_owned(),
    ];
    vec![
        select.clone(),
        vec!["confirm".to_owned()],
        vec!["room".into(), "hierarchy".into(), "P".into(), "main".into()],
        vec![
            "room".into(),
            "list".into(),
            "--project-id".into(),
            "P".into(),
        ],
        vec!["task".into(), "list".into()],
        vec!["task".into(), "sources".into()],
        vec![
            "task".into(),
            "board".into(),
            "P".into(),
            "missing-source".into(),
        ],
        vec!["profession".into(), "list".into()],
        vec!["agency".into(), "bindings".into()],
        vec!["task".into(), "show".into(), "P".into(), "missing".into()],
    ]
}

thread_local! {
    static CONFIRM_TOKEN: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

#[test]
fn machine_json_samples_stay_byte_identical() {
    let temp = Temp::new();
    let root = temp.0.as_path();
    let (path, _) = dispatch_fixture(root);
    let commands = sample_commands(&path);
    let mut outputs = Vec::new();
    let mut select_arguments: Option<Vec<String>> = None;
    for (index, arguments) in commands.iter().enumerate() {
        let expanded: Vec<String> = if arguments.first().map(String::as_str) == Some("confirm") {
            let token = CONFIRM_TOKEN.with(|cell| cell.borrow().clone());
            assert!(!token.is_empty(), "the preview run must capture its token");
            select_arguments
                .clone()
                .expect("the preview precedes its confirm")
                .into_iter()
                .chain(["--preview-token".to_owned(), token])
                .collect()
        } else {
            arguments.clone()
        };
        if SAMPLES[index].0 == "select-preview" {
            select_arguments = Some(expanded.clone());
        }
        let arguments: Vec<&str> = expanded.iter().map(String::as_str).collect();
        let (ok, first, err) = run_bytes(root, true, &arguments);
        let first = String::from_utf8(first).unwrap();
        if SAMPLES[index].0 != "error-not-found" && SAMPLES[index].0 != "task-board-no-source" {
            assert!(
                ok,
                "{}: {first}{}",
                SAMPLES[index].0,
                String::from_utf8_lossy(&err)
            );
        }
        // Determinism first: the same command twice must give the same bytes.
        // A fresh preview draws a fresh token, so both sides are masked first;
        // a confirm moves the state it confirms, so it runs once.
        let masked = mask_preview_token(&first);
        if SAMPLES[index].0 == "select-preview" {
            let parsed: Value = serde_json::from_str(&first).unwrap();
            CONFIRM_TOKEN.with(|cell| {
                *cell.borrow_mut() = parsed["preview_token"]
                    .as_str()
                    .expect("a token")
                    .to_owned()
            });
        }
        if SAMPLES[index].0 != "select-confirm" {
            let (_, second, _) = run_bytes(root, true, &arguments);
            let second = String::from_utf8(second).unwrap();
            assert_eq!(
                masked,
                mask_preview_token(&second),
                "{} is not deterministic",
                SAMPLES[index].0
            );
        }
        outputs.push(masked);
    }
    let golden_dir = std::env::var_os("HCTL2_GOLDEN_DIR");
    for (index, output) in outputs.iter().enumerate() {
        let (name, _) = SAMPLES[index];
        if let Some(dir) = &golden_dir {
            std::fs::write(
                std::path::Path::new(dir).join(format!("{name}.json")),
                output,
            )
            .unwrap();
        } else {
            assert_eq!(
                output.trim_end(),
                SAMPLES[index].1.trim_end(),
                "--json output of {name} drifted from the saved sample"
            );
        }
    }
}

#[test]
fn human_lists_render_as_tables_with_headers_and_empty_notes() {
    let temp = Temp::new();
    let root = temp.0.as_path();
    let (path, _) = dispatch_fixture(root);
    let _ = path;
    struct Case {
        name: &'static str,
        arguments: Vec<&'static str>,
        header: &'static str,
        row: Vec<&'static str>,
        empty: Option<&'static str>,
    }
    let cases = vec![
        Case {
            name: "profession list",
            arguments: vec!["profession", "list"],
            header: "profession  revision  digest  version",
            row: vec!["research"],
            empty: None,
        },
        Case {
            name: "agency bindings",
            arguments: vec!["agency", "bindings"],
            header: "binding  protocol  version",
            row: vec!["script-agency"],
            empty: None,
        },
        Case {
            name: "room list",
            arguments: vec!["room", "list", "--project-id", "P"],
            header: "room  project  attention",
            row: vec!["main", "P"],
            empty: None,
        },
        Case {
            name: "project list",
            arguments: vec!["project", "list"],
            header: "project  version  archived",
            row: vec!["P"],
            empty: None,
        },
        Case {
            name: "task list",
            arguments: vec!["task", "list"],
            header: "",
            row: vec![],
            empty: Some("No tasks yet."),
        },
        Case {
            name: "task sources",
            arguments: vec!["task", "sources"],
            header: "",
            row: vec![],
            empty: Some("No task sources connected yet."),
        },
        Case {
            name: "invocation list",
            arguments: vec!["invocation", "list", "P"],
            header: "",
            row: vec![],
            empty: Some("This Project has no Invocations yet."),
        },
    ];
    for case in &cases {
        let Case {
            name,
            arguments,
            header,
            row,
            empty,
        } = case;

        let arguments: Vec<&str> = arguments.clone();
        let (ok, out, err) = run(root, false, &arguments);
        assert!(ok, "{name}: {out}{err}");
        assert!(!out.contains('\u{1b}'), "{name} must not color: {out}");
        if let Some(empty) = empty {
            assert!(
                out.contains(empty),
                "{name} should explain the empty list: {out}"
            );
            continue;
        }
        let mut lines = out.lines();
        let first = lines.next().expect("a header row");
        assert_eq!(
            first.split_whitespace().collect::<Vec<_>>(),
            header.split_whitespace().collect::<Vec<_>>(),
            "{name} header"
        );
        let separator = lines.next().expect("a separator row");
        assert!(
            separator.chars().all(|c| c == '-' || c == ' '),
            "{name} separator: {separator}"
        );
        let body = lines.collect::<String>();
        for value in row {
            assert!(body.contains(value), "{name} must list {value}: {out}");
        }
    }
}

#[test]
fn human_errors_show_code_message_and_recovery_action_verbatim() {
    let temp = Temp::new();
    let root = temp.0.as_path();
    let (path, _) = dispatch_fixture(root);
    // 1. PREVIEW_REQUIRED: a submit whose token matches no stored preview.
    let arguments: Vec<&str> = vec![
        "project",
        "select",
        "--input",
        &path,
        "--key",
        "err-select",
        "--preview-token",
        "not-a-real-token",
    ];
    let (ok, json_out, _) = run(root, true, &arguments);
    assert!(!ok);
    let machine: Value = serde_json::from_str(&json_out).unwrap();
    let error = machine["error"].clone();
    let (ok, human, _) = run(root, false, &arguments);
    assert!(!ok, "the failed submit exits non-zero");
    for (label, value) in [
        ("code", error["code"].as_str().unwrap()),
        ("message", error["message"].as_str().unwrap()),
        (
            "recovery_action",
            error["recovery_action"].as_str().unwrap(),
        ),
    ] {
        assert!(
            human.contains(&format!("{label}: {value}")),
            "the human error names {label} verbatim: {human}"
        );
    }
    assert_eq!(error["code"], "PREVIEW_REQUIRED");
    // 2. VERSION_CONFLICT: after the roster moved, the same select input is
    // stale and the preview itself rejects it.
    let select = ["project", "select", "--input", &path, "--key", "err-twice"];
    let (ok, out, err) = run(root, true, &select);
    assert!(ok, "{out} {err}");
    let preview: Value = serde_json::from_str(&out).unwrap();
    let token = preview["preview_token"].as_str().unwrap().to_owned();
    let mut confirm = select.to_vec();
    confirm.push("--preview-token");
    confirm.push(&token);
    let (ok, out, err) = run(root, true, &confirm);
    assert!(ok, "{out} {err}");
    let stale = [
        "project",
        "select",
        "--input",
        &path,
        "--key",
        "err-stale-after-roster-moved",
    ];
    let (ok, json_out, _) = run(root, true, &stale);
    assert!(!ok, "{json_out}");
    let machine: Value = serde_json::from_str(&json_out).unwrap();
    assert_eq!(machine["error"]["code"], "VERSION_CONFLICT", "{json_out}");
    let (ok, human, _) = run(root, false, &stale);
    assert!(!ok);
    assert!(
        human.contains("code: VERSION_CONFLICT"),
        "the human error names the code: {human}"
    );
    assert!(
        human.contains(&format!(
            "message: {}",
            machine["error"]["message"].as_str().unwrap()
        )),
        "the message matches the JSON value: {human}"
    );
    assert!(
        human.contains(&format!(
            "recovery_action: {}",
            machine["error"]["recovery_action"].as_str().unwrap()
        )),
        "the recovery action matches the JSON value: {human}"
    );
    // 3. A typed boundary rejection: managed input has no public entry.
    let (ok, json_out, _) = run(root, true, &["terminal", "attach", "P", "none"]);
    assert!(!ok);
    let machine: Value = serde_json::from_str(&json_out).unwrap();
    assert_eq!(machine["error"]["code"], "INPUT_NOT_IMPLEMENTED");
    let (ok, human, _) = run(root, false, &["terminal", "attach", "P", "none"]);
    assert!(!ok);
    assert!(human.contains("code: INPUT_NOT_IMPLEMENTED"), "{human}");
    assert!(
        human.contains(&format!(
            "recovery_action: {}",
            machine["error"]["recovery_action"].as_str().unwrap()
        )),
        "{human}"
    );
}

#[test]
fn human_integration_preview_names_source_target_and_confirm_command() {
    let temp = Temp::new();
    let root = temp.0.as_path();
    assert!(run(root, true, &["init", "--secret-backend", "user-file"]).0);
    // A real local git repository as the integration target.
    let git = root.join("target.git");
    std::fs::create_dir_all(&git).unwrap();
    let git_env = [
        ("GIT_AUTHOR_NAME", "fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
        ("GIT_AUTHOR_DATE", "2005-04-02T22:20:00+00:00"),
        ("GIT_COMMITTER_DATE", "2005-04-02T22:20:00+00:00"),
    ];
    let git_root = git.clone();
    let git_command = move || {
        let mut command = Command::new("git");
        command
            .current_dir(&git_root)
            .env_remove("GIT_CONFIG_GLOBAL");
        for (name, value) in git_env {
            command.env(name, value);
        }
        command
    };
    for arguments in [
        vec!["init", "-q", "-b", "main"],
        vec!["config", "user.name", "fixture"],
        vec!["config", "user.email", "fixture@example.invalid"],
        vec!["commit", "-q", "--allow-empty", "-m", "initial"],
    ] {
        let status = git_command().args(&arguments).status().unwrap();
        assert!(status.success(), "git {arguments:?}");
    }
    let head = git_command().args(["rev-parse", "HEAD"]).output().unwrap();
    let head = String::from_utf8(head.stdout).unwrap().trim().to_owned();
    let tree = git_command()
        .args(["rev-parse", "HEAD^{tree}"])
        .output()
        .unwrap();
    let tree = String::from_utf8(tree.stdout).unwrap().trim().to_owned();
    assert!(run(root, true, &["start"]).0);
    // Register the Repo through the real CLI: preview, then confirm with the token.
    let register_input = root.join("register.json");
    std::fs::write(
        &register_input,
        json!({"name":"fixture-target","origin":"local","platform":"none",
            "local":{"machine":"control","path":git.display().to_string()}})
        .to_string(),
    )
    .unwrap();
    let register = [
        "repo",
        "register",
        "--input",
        register_input.to_str().unwrap(),
        "--key",
        "fixture-repo",
    ];
    let (ok, out, err) = run(root, true, &register);
    assert!(ok, "{out} {err}");
    let plan: Value = serde_json::from_str(&out).unwrap();
    let token = plan["preview_token"].as_str().unwrap();
    let mut confirm = register.to_vec();
    confirm.push("--preview-token");
    confirm.push(token);
    let (ok, out, err) = run(root, true, &confirm);
    assert!(ok, "{out} {err}");
    let registered: Value = serde_json::from_str(&out).unwrap();
    let repo_id = registered["repo_id"]
        .as_str()
        .or_else(|| registered["registration"]["repo_id"].as_str())
        .expect("a repo id in the register result")
        .to_owned();
    // Seeding the revision writes the store directly, so the daemon steps aside
    // and comes back once the ChangeSet Revision is admitted.
    assert!(
        run(root, true, &["stop"]).0,
        "stop the daemon before seeding"
    );
    // One admitted ChangeSet Revision, sealed by a human command.
    let (revision,) = {
        let mut trusted = actor("P");
        let mut store = None;
        for _ in 0..50 {
            match store::Store::open(root) {
                Ok(open) => {
                    store = Some(open);
                    break;
                }
                Err(error) if error.code == "WRITER_BUSY" => {
                    std::thread::sleep(Duration::from_millis(100))
                }
                Err(error) => panic!("{error}"),
            }
        }
        let mut store = store.expect("the daemon released the store");
        let scope = store::Scope::Repo(repo_id.clone());
        trusted.0.permission_scope.push(scope.clone());
        let set = repo::changeset::open_change_set(
            &mut store,
            &trusted,
            &repo_id,
            1,
            &head,
            "human-seal",
            &repo::changeset::ProducerRef::HumanCommand {
                command_id: "seal-once".into(),
            },
        )
        .unwrap();
        let revision = repo::changeset::admit(
            &mut store,
            &trusted,
            repo::changeset::Seal {
                association_key: "seal-once".into(),
                change_set_id: set.change_set_id.clone(),
                change_set_version: set.version,
                lease: None,
                base_commit_sha: head.clone(),
                result_tree_sha: tree.clone(),
                result_commit_sha: None,
                parent_revision_id: None,
                producer_ref: repo::changeset::ProducerRef::HumanCommand {
                    command_id: "seal-once".into(),
                },
            },
            repo::changeset::OwnerGate::Active,
        )
        .unwrap();
        (revision,)
    };
    assert!(run(root, true, &["start"]).0);
    let input = root.join("integrate.json");
    std::fs::write(
        &input,
        json!({"key":"integrate-1","repo_id":repo_id,
            "change_set_revision_id":revision.change_set_revision_id,
            "target_kind":"local","target_ref":"refs/heads/main",
            "form":"expected_head","strategy":"fast_forward"})
        .to_string(),
    )
    .unwrap();
    let arguments = [
        "integration",
        "preview",
        "--input",
        input.to_str().unwrap(),
        "--key",
        "integrate-1",
    ];
    let (ok, machine, err) = run(root, true, &arguments);
    assert!(ok, "{machine} {err}");
    let machine: Value = serde_json::from_str(&machine).unwrap();
    let (ok, human, err) = run(root, false, &arguments);
    assert!(ok, "{human} {err}");
    for heading in [
        "Integration preview",
        "Object",
        "Why this needs confirmation",
        "What happens after you confirm",
        "Confirm with",
    ] {
        assert!(
            human.contains(heading),
            "missing section {heading}:\n{human}"
        );
    }
    // The three sections carry the machine interface's own values.
    assert!(
        human.contains(&revision.change_set_revision_id),
        "the revision is named:\n{human}"
    );
    assert!(human.contains(&repo_id), "{human}");
    assert!(human.contains("refs/heads/main"), "{human}");
    assert!(
        human.contains(
            &machine["effect_summary"]["observed_head"]
                .as_str()
                .unwrap()
                .to_owned()
        ),
        "the observed head matches the JSON value:\n{human}"
    );
    assert!(human.contains("--preview-token"), "{human}");
    assert!(!human.contains('\u{1b}'), "{human}");
}

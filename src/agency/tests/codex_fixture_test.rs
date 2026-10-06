//! Deterministic Codex app-server cases. The fixture is a file named `codex`.
use agency::launch::InstalledHerdr;
use agency::runtime::{Running, Runtime, RuntimeEvent};
use agency_proto::Sealed;
use serde_json::Value;
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

static GATE: Mutex<()> = Mutex::new(());

#[test]
fn codex_fixture_turn_matches_rollout_and_uses_a_read_only_sandbox() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    prepare("ok");
    let (runtime, cred, root, exec) = open_runtime("codex-ok", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "ok");
    let events = collect(&mut running, Duration::from_secs(20));
    assert_eq!(
        proposal(&events),
        Some(b"ANSWER".as_slice()),
        "{}",
        show(&events)
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, RuntimeEvent::TurnReturned)),
        "{}",
        show(&events)
    );
    let turn = read_json("last-turn.json");
    assert_eq!(turn["approvalPolicy"], "never");
    assert_eq!(turn["sandboxPolicy"]["type"], "readOnly");
    let started = read_json("last-thread.json");
    assert_eq!(started["approvalPolicy"], "never");
    assert_eq!(started["sandbox"], "read-only");
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn codex_fixture_rejects_a_rollout_that_does_not_match_the_task() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    prepare("mismatch");
    let (runtime, cred, root, exec) = open_runtime("codex-mismatch", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "mismatch");
    let events = collect(&mut running, Duration::from_secs(10));
    assert_eq!(
        protocol(&events),
        Some("STANDBY_PROMPT_MISMATCH"),
        "{}",
        show(&events)
    );
    assert!(proposal(&events).is_none(), "{}", show(&events));
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn codex_fixture_ignores_a_turn_completed_for_another_turn() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    prepare("foreign-turn");
    let (runtime, cred, root, exec) = open_runtime("codex-foreign-turn", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "foreign-turn");
    let events = collect(&mut running, Duration::from_secs(20));
    assert_eq!(
        proposal(&events),
        Some(b"ANSWER".as_slice()),
        "{}",
        show(&events)
    );
    assert!(protocol(&events).is_none(), "{}", show(&events));
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn codex_fixture_ignores_an_item_completed_for_another_turn() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    prepare("foreign-item");
    let (runtime, cred, root, exec) = open_runtime("codex-foreign-item", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "foreign-item");
    let events = collect(&mut running, Duration::from_secs(10));
    assert_eq!(
        protocol(&events),
        Some("INVALID_INPUT"),
        "{}",
        show(&events)
    );
    assert!(proposal(&events).is_none(), "{}", show(&events));
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn codex_fixture_interrupt_stops_the_turn() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    prepare("interrupt");
    let (runtime, cred, root, exec) = open_runtime("codex-interrupt", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "interrupt");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !home().join("turn-started").exists() {
        assert!(Instant::now() < deadline, "turn did not start");
        thread::sleep(Duration::from_millis(20));
    }
    running
        .session
        .lock()
        .expect("session")
        .stop()
        .expect("stop");
    let events = collect(&mut running, Duration::from_secs(10));
    assert!(
        events.iter().any(|event| matches!(
            event,
            RuntimeEvent::TurnStopped {
                requested_stop: true,
                session_closed: false
            }
        )),
        "{}",
        show(&events)
    );
    assert!(proposal(&events).is_none(), "{}", show(&events));
    assert!(home().join("interrupt-seen").exists(), "{}", show(&events));
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn codex_fixture_failed_resume_starts_a_new_thread() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    prepare("ok");
    let (runtime, cred, root, exec) = open_runtime("codex-resume", Duration::from_millis(300));
    let mut first = dispatch(&runtime, &exec, &cred, "resume");
    let first_events = collect(&mut first, Duration::from_secs(20));
    let first_thread = opened(&first_events)["thread"].as_str().unwrap().to_owned();
    assert_eq!(
        opened(&first_events)["resume_failed"],
        false,
        "{}",
        show(&first_events)
    );
    fs::write(home().join("mode"), "resume-fail").unwrap();
    thread::sleep(Duration::from_millis(1500));
    let mut second = dispatch(&runtime, &exec, &cred, "resume-again");
    let events = collect(&mut second, Duration::from_secs(20));
    let again = opened(&events);
    assert_eq!(again["resumed"], false, "{}", show(&events));
    assert_eq!(again["resume_failed"], true, "{}", show(&events));
    assert_ne!(
        again["thread"].as_str().unwrap(),
        first_thread,
        "{}",
        show(&events)
    );
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

fn prepare(mode: &str) {
    let home = home();
    fs::create_dir_all(&home).unwrap();
    let _ = fs::remove_dir_all(home.join("sessions"));
    for name in [
        "turn-started",
        "interrupt-seen",
        "last-turn.json",
        "last-thread.json",
    ] {
        let _ = fs::remove_file(home.join(name));
    }
    fs::write(home.join("mode"), mode).unwrap();
}

fn home() -> PathBuf {
    let home = PathBuf::from(std::env::var("CODEX_HOME").expect("CODEX_HOME"));
    assert!(
        home.ends_with("hctl2-codex-fixture-home"),
        "{}",
        home.display()
    );
    home
}

fn read_json(name: &str) -> Value {
    let bytes = fs::read(home().join(name)).unwrap_or_else(|error| panic!("{name}: {error}"));
    serde_json::from_slice(&bytes).unwrap()
}

fn codex_bin() -> PathBuf {
    static BIN: OnceLock<PathBuf> = OnceLock::new();
    BIN.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("hctl2-codex-bin-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("codex");
        fs::copy(
            std::env::var("HCTL2_CODEX_FIXTURE").expect("HCTL2_CODEX_FIXTURE"),
            &dest,
        )
        .unwrap();
        let mut permissions = fs::metadata(&dest).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&dest, permissions).unwrap();
        dest
    })
    .clone()
}

fn binary() -> PathBuf {
    let path = PathBuf::from(std::env::var("HCTL2_LOCKED_HERDR").expect("locked Herdr"));
    let path = path.canonicalize().expect("locked Herdr");
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();
    path
}

fn open_runtime(name: &str, idle: Duration) -> (InstalledHerdr, PathBuf, PathBuf, PathBuf) {
    let cred =
        std::env::temp_dir().join(format!("hctl2-codex-cred-{}-{}", name, std::process::id()));
    let root =
        std::env::temp_dir().join(format!("hctl2-codex-root-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&cred);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&cred).unwrap();
    fs::write(cred.join("pair.key"), b"secret-credential").unwrap();
    let exec = root.join("dispatches/job");
    fs::create_dir_all(&exec).unwrap();
    let claude = root.join("claude-stub");
    fs::write(&claude, "#!/bin/sh\necho '2.1.289 (Claude Code)'\n").unwrap();
    let mut permissions = fs::metadata(&claude).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&claude, permissions).unwrap();
    let runtime = InstalledHerdr::open_with_codex(binary(), &claude, &codex_bin(), idle)
        .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    assert!(
        runtime.codex_skip().is_none(),
        "{}",
        runtime.codex_skip().unwrap_or("")
    );
    (runtime, cred, root, exec)
}

fn dispatch(runtime: &InstalledHerdr, exec: &Path, cred: &Path, key: &str) -> Running {
    let bundle = sealed_bundle("HELLO");
    let mut document = sealed_spec(key).document;
    document.profession.reference.id = "codex-cli".into();
    document.bundle.digest = bundle.digest.clone();
    let spec = Sealed::new(document).unwrap();
    runtime
        .start(&spec, &bundle, exec, cred)
        .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message))
}

fn collect(running: &mut Running, timeout: Duration) -> Vec<RuntimeEvent> {
    let mut events = Vec::new();
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        match running.events.recv_timeout(Duration::from_millis(200)) {
            Ok(event) => {
                let done = matches!(
                    event,
                    RuntimeEvent::DispatchReleased | RuntimeEvent::Exited { .. }
                );
                events.push(event);
                if done {
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    events
}

fn show(events: &[RuntimeEvent]) -> String {
    events
        .iter()
        .map(|event| match event {
            RuntimeEvent::Observation { kind, payload, .. } => format!("obs {kind} {payload}"),
            RuntimeEvent::Proposal { schema, bytes, .. } => {
                format!("proposal {schema} {}", String::from_utf8_lossy(bytes))
            }
            RuntimeEvent::TurnReturned => "returned".into(),
            RuntimeEvent::TurnStopped {
                requested_stop,
                session_closed,
            } => format!("stopped requested={requested_stop} closed={session_closed}"),
            RuntimeEvent::DispatchReleased => "released".into(),
            RuntimeEvent::Exited {
                code,
                requested_stop,
            } => format!("exited code={code:?} requested={requested_stop}"),
            RuntimeEvent::ProtocolError(code) => format!("protocol {code}"),
            RuntimeEvent::DeadlineReached => "deadline".into(),
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn proposal(events: &[RuntimeEvent]) -> Option<&[u8]> {
    events.iter().find_map(|event| match event {
        RuntimeEvent::Proposal { bytes, .. } => Some(bytes.as_slice()),
        _ => None,
    })
}

fn protocol(events: &[RuntimeEvent]) -> Option<&str> {
    events.iter().find_map(|event| match event {
        RuntimeEvent::ProtocolError(code) => Some(code.as_str()),
        _ => None,
    })
}

fn opened(events: &[RuntimeEvent]) -> &Value {
    events
        .iter()
        .find_map(|event| match event {
            RuntimeEvent::Observation { kind, payload, .. } if kind == "session_opened" => {
                Some(payload)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("no session_opened in {}", show(events)))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn sealed_spec(key: &str) -> Sealed<agency_proto::ExecutionSpec> {
    use agency_proto::{
        Capabilities, ExecutionSpec, FrozenRef, InputPolicy, Owner, OwnerKind, Profession,
    };
    let frozen = |id: &str| FrozenRef {
        id: id.into(),
        revision: "1".into(),
        digest: agency_proto::hash(id.as_bytes()),
    };
    Sealed::new(ExecutionSpec {
        owner: Owner {
            project: "project".into(),
            kind: OwnerKind::RoomInvocation,
            id: "invocation".into(),
            generation: 1,
        },
        project: frozen("project"),
        selection: frozen("selection"),
        selection_policy_digest: agency_proto::hash(b"policy"),
        profession: Profession {
            reference: frozen("codex-cli"),
            harness: frozen("herdr"),
            model: "none".into(),
            persona: "test".into(),
            terms: "test".into(),
            default_role: "worker".into(),
            skills: vec![],
            capabilities: Capabilities {
                stop: true,
                ..Capabilities::default()
            },
        },
        profile: frozen("profile"),
        manifest: frozen("manifest"),
        bundle: frozen("bundle"),
        binding: frozen("binding"),
        required_capabilities: Capabilities {
            stop: true,
            ..Capabilities::default()
        },
        input_policy: InputPolicy::NoInput,
        permission_digest: agency_proto::hash(b"permissions"),
        permissions: vec!["context.read".into()],
        budget: 1,
        deadline_ms: now_ms() + 60_000,
        repo: None,
        base: None,
        delivery_scope: vec![],
        write_lease: None,
        review_publish_policy: None,
        idempotency_key: key.into(),
    })
    .unwrap()
}

fn sealed_bundle(task: &str) -> Sealed<agency_proto::context::Bundle> {
    use agency_proto::context::{Bundle, Delivery, Entry};
    use agency_proto::{FrozenRef, Owner, OwnerKind};
    let frozen = |id: &str| FrozenRef {
        id: id.into(),
        revision: "1".into(),
        digest: agency_proto::hash(id.as_bytes()),
    };
    Sealed::new(Bundle {
        id: "bundle".into(),
        manifest: frozen("manifest"),
        consumer: Owner {
            project: "project".into(),
            kind: OwnerKind::RoomInvocation,
            id: "invocation".into(),
            generation: 1,
        },
        entries: vec![Entry {
            source: frozen("task"),
            description: "task".into(),
            required: true,
            offline_required: false,
            delivery: Delivery::Inline {
                bytes: task.as_bytes().to_vec(),
            },
            bytes_digest: agency_proto::hash(task.as_bytes()),
        }],
        renderer: frozen("renderer"),
        tokenizer: frozen("tokenizer"),
        redaction: frozen("redaction"),
        compression: vec![],
        candidate_tokens: None,
        selected_tokens: None,
        delivered_tokens: None,
        permission_digest: agency_proto::hash(b"permissions"),
        budget: 1,
        retention: "test".into(),
    })
    .unwrap()
}

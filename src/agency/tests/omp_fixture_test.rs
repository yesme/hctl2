//! Deterministic omp RPC cases. The fixture is a file named `omp`.
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
fn omp_fixture_turn_returns_the_answer_and_isolates_the_profile() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("omp-ok", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "HELLO");
    let events = collect(&mut running, Duration::from_secs(20));
    assert_eq!(
        proposal(&events),
        Some(("agency.turn.v1", b"ANSWER".as_slice())),
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
    let opened = opened(&events);
    assert_eq!(opened["pane"], Value::Null);
    assert_eq!(opened["resume_failed"], false);
    let file = PathBuf::from(opened["file"].as_str().expect("session file"));
    let body = fs::read_to_string(&file).expect("session body");
    assert!(body.contains("HELLO"), "{body}");
    let session = opened["session"].as_str().expect("session id");
    let session_dir = agency::herdr::state_dir(&exec, &cred)
        .expect("state dir")
        .join("omp")
        .join(session);
    let state: Value =
        serde_json::from_slice(&fs::read(session_dir.join("state.json")).expect("state.json"))
            .expect("state json");
    assert_eq!(state["harness"], "omp", "{state}");
    assert_eq!(state["session"], session, "{state}");
    assert_eq!(state["pane_id"], Value::Null, "{state}");
    let events = fs::read_to_string(session_dir.join("events.jsonl")).expect("events.jsonl");
    assert!(events.contains("ready"), "{events}");
    let view = fs::read_to_string(session_dir.join("view.log")).expect("view.log");
    assert!(view.contains("=== turn 1 start ==="), "{view}");
    assert!(view.contains("[prompt]"), "{view}");
    assert!(view.contains("[answer]"), "{view}");
    assert!(view.contains("=== turn 1 end (completed) ==="), "{view}");
    let argv = read_session_json(&file, "argv.json");
    let args = argv["args"].as_array().unwrap();
    assert!(
        args.windows(2)
            .any(|pair| pair[0] == "--mode" && pair[1] == "rpc"),
        "{argv}"
    );
    assert!(
        args.windows(2)
            .any(|pair| pair[0] == "--approval-mode" && pair[1] == "yolo"),
        "{argv}"
    );
    let home = argv["home"].as_str().expect("managed home");
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.clone());
    assert!(PathBuf::from(home).starts_with(&canonical_root), "{home}");
    assert!(home.ends_with("/omp-home"), "{home}");
    assert!(argv["profile"].is_null(), "{argv}");
    assert!(argv["codex_home"].is_null(), "{argv}");
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn omp_fixture_rejects_a_session_that_does_not_match_the_task() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("omp-mismatch", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "FIXTURE_MISMATCH");
    let events = collect(&mut running, Duration::from_secs(20));
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
fn omp_fixture_local_command_is_not_an_agent_turn() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("omp-local", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "FIXTURE_LOCAL");
    let events = collect(&mut running, Duration::from_secs(20));
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
fn omp_fixture_login_request_is_reported_not_answered() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("omp-login", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "FIXTURE_LOGIN");
    let events = collect(&mut running, Duration::from_secs(20));
    assert_eq!(
        protocol(&events),
        Some("HARNESS_LOGIN_REQUIRED"),
        "{}",
        show(&events)
    );
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn omp_fixture_interrupt_stops_the_turn() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("omp-interrupt", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "FIXTURE_INTERRUPT");
    let start = Instant::now() + Duration::from_secs(20);
    loop {
        if find_session_file(&exec, &cred, "turn-started").is_some() {
            break;
        }
        assert!(Instant::now() < start, "turn did not start");
        thread::sleep(Duration::from_millis(20));
    }
    thread::sleep(Duration::from_millis(50));
    running
        .session
        .lock()
        .expect("session")
        .stop()
        .expect("stop");
    let events = collect(&mut running, Duration::from_secs(20));
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
    assert!(
        find_session_file(&exec, &cred, "interrupt-seen").is_some(),
        "{}",
        show(&events)
    );
    assert!(proposal(&events).is_none(), "{}", show(&events));
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn omp_fixture_failed_resume_starts_a_new_session() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("omp-resume", Duration::from_millis(300));
    let mut first = dispatch(&runtime, &exec, &cred, "resume");
    let first_events = collect(&mut first, Duration::from_secs(20));
    let first_open = opened(&first_events);
    let first_session = first_open["session"].as_str().unwrap().to_owned();
    let sessions = PathBuf::from(first_open["file"].as_str().unwrap())
        .parent()
        .unwrap()
        .to_path_buf();
    fs::write(sessions.join("resume-fail"), b"fail").unwrap();
    thread::sleep(Duration::from_millis(1500));
    let mut second = dispatch(&runtime, &exec, &cred, "resume-again");
    let events = collect(&mut second, Duration::from_secs(20));
    let again = opened(&events);
    assert_eq!(again["resumed"], false, "{}", show(&events));
    assert_eq!(again["resume_failed"], true, "{}", show(&events));
    assert_ne!(
        again["session"].as_str().unwrap(),
        first_session,
        "{}",
        show(&events)
    );
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn omp_read_only_cannot_read_credentials_keyring_or_gh_auth() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let specimen = SecretSpecimen::plant("omp-read");
    let (runtime, cred, root, exec) = open_runtime("omp-secret", Duration::from_secs(300));
    let cred_file = cred.join("pair.key").canonicalize().unwrap();
    let task = format!(
        "HCTL2_SECRET_PROBE {}\n",
        serde_json::json!({"secret_paths":[cred_file, &specimen.file]})
    );
    let mut running = start_task(
        &runtime,
        &exec,
        &cred,
        "secret",
        "fixture/model",
        &task,
        &[],
    )
    .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    let events = collect(&mut running, Duration::from_secs(20));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, RuntimeEvent::TurnReturned)),
        "{}",
        show(&events)
    );
    let probe = session_json(&exec, &cred, "secret-probe.json");
    for read in probe["reads"].as_array().unwrap() {
        assert_eq!(read["readable"], false, "{probe}");
    }
    assert_eq!(probe["gh_authenticated"], false, "{probe}");
    assert_eq!(probe["gh_token_present"], false);
    assert_eq!(probe["dbus_present"], false);
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn omp_read_only_cannot_write_the_source_repo() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let repo = std::env::temp_dir().join(format!("hctl2-omp-src-repo-{}", std::process::id()));
    let _ = fs::remove_dir_all(&repo);
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("keep"), b"original").unwrap();
    let (runtime, cred, root, exec) = open_runtime("omp-src-repo", Duration::from_secs(300));
    let task = format!(
        "HCTL2_SECRET_PROBE {}\n",
        serde_json::json!({"secret_paths":[], "source_repo": &repo})
    );
    let mut running = start_task(
        &runtime,
        &exec,
        &cred,
        "src-repo",
        "fixture/model",
        &task,
        &[],
    )
    .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    let events = collect(&mut running, Duration::from_secs(20));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, RuntimeEvent::TurnReturned)),
        "{}",
        show(&events)
    );
    let probe = session_json(&exec, &cred, "secret-probe.json");
    let marker = repo.join("hctl2-ro-write");
    if cfg!(target_os = "linux") {
        assert_eq!(probe["source_write"], false, "{probe}");
        assert!(
            !marker.exists(),
            "read-only dispatch wrote into the source repo"
        );
        assert_eq!(fs::read(repo.join("keep")).unwrap(), b"original");
    } else {
        let _ = fs::remove_file(&marker);
    }
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(&repo);
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn declared_no_network_is_refused_before_omp_starts() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("omp-isolate", Duration::from_secs(300));
    let error = match start_task(
        &runtime,
        &exec,
        &cred,
        "isolate",
        "fixture/model",
        "HELLO",
        &["no_network"],
    ) {
        Err(error) => error,
        Ok(_) => panic!("declared no_network must not start"),
    };
    assert_eq!(error.code, "ISOLATION_UNAVAILABLE");
    assert_eq!(error.message, "no_network");
    assert_eq!(error.recovery_action, "drop_unenforceable_isolation");
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn omp_fixture_dead_between_turns_process_is_resumed_before_next_delivery() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("omp-dead", Duration::from_secs(300));
    let task = "repeat this task";
    let mut first = dispatch(&runtime, &exec, &cred, task);
    let first_events = collect(&mut first, Duration::from_secs(20));
    assert!(proposal(&first_events).is_some(), "{}", show(&first_events));
    let first_open = opened(&first_events);
    let file = PathBuf::from(first_open["file"].as_str().expect("session file"));
    let first_pid = read_session_json(&file, "argv.json")["pid"]
        .as_u64()
        .expect("fixture pid");
    let status = std::process::Command::new("/bin/kill")
        .args(["-KILL", &first_pid.to_string()])
        .status()
        .expect("kill fixture");
    assert!(status.success());
    thread::sleep(Duration::from_millis(300));
    let mut second = dispatch(&runtime, &exec, &cred, task);
    let events = collect(&mut second, Duration::from_secs(20));
    assert_eq!(
        proposal(&events),
        Some(("agency.turn.v1", b"ANSWER".as_slice())),
        "{}",
        show(&events)
    );
    assert!(protocol(&events).is_none(), "{}", show(&events));
    let again = opened(&events);
    assert_eq!(again["resumed"], true, "{}", show(&events));
    let second_pid = read_session_json(&file, "argv.json")["pid"]
        .as_u64()
        .expect("fixture pid");
    assert_ne!(second_pid, first_pid, "{}", show(&events));
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn omp_fixture_old_history_cannot_vouch_for_a_mismatched_current_turn() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("omp-history", Duration::from_secs(300));
    let task = "repeat this task";
    let mut first = dispatch(&runtime, &exec, &cred, task);
    let first_events = collect(&mut first, Duration::from_secs(20));
    let file = PathBuf::from(
        opened(&first_events)["file"]
            .as_str()
            .expect("session file"),
    );
    fs::write(file.parent().unwrap().join("mismatch-current"), b"1").unwrap();
    let mut second = dispatch(&runtime, &exec, &cred, task);
    let events = collect(&mut second, Duration::from_secs(20));
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
fn omp_fixture_repeated_identical_prompt_and_large_history_still_return() {
    use std::io::Write as _;
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("omp-repeat", Duration::from_secs(300));
    let task = "repeat this task";
    let mut first = dispatch(&runtime, &exec, &cred, task);
    let first_events = collect(&mut first, Duration::from_secs(20));
    let file = PathBuf::from(
        opened(&first_events)["file"]
            .as_str()
            .expect("session file"),
    );
    let mut history = fs::OpenOptions::new().append(true).open(&file).unwrap();
    history.write_all(&vec![b'x'; 17 * 1024 * 1024]).unwrap();
    history.write_all(b"\n").unwrap();
    drop(history);
    let mut second = dispatch(&runtime, &exec, &cred, task);
    let events = collect(&mut second, Duration::from_secs(20));
    assert_eq!(
        proposal(&events),
        Some(("agency.turn.v1", b"ANSWER".as_slice())),
        "{}",
        show(&events)
    );
    assert!(protocol(&events).is_none(), "{}", show(&events));
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

#[test]
#[ignore = "UNVERIFIED: installed omp and managed profile login required; HCTL2_OMP_REAL=1, HCTL2_OMP_AGENT_DB and HCTL2_OMP_CONFIG"]
fn omp_real_read_only_research_returns_a_proposal() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    assert_eq!(
        std::env::var("HCTL2_OMP_REAL").as_deref(),
        Ok("1"),
        "UNVERIFIED: local omp login required"
    );
    let (runtime, cred, root, exec) = open_runtime_real("omp-real", Duration::from_secs(300));
    let task = "Reply with exactly HCTL2_OMP_REAL_OK and nothing else.";
    let mut running = start_task(&runtime, &exec, &cred, "real", "none", task, &[])
        .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    let events = collect(&mut running, Duration::from_secs(120));
    let (schema, bytes) = proposal(&events).unwrap_or_else(|| panic!("{}", show(&events)));
    assert_eq!(schema, "agency.turn.v1", "{}", show(&events));
    let text = String::from_utf8_lossy(bytes);
    assert!(text.contains("HCTL2_OMP_REAL_OK"), "{text}");
    let opened = opened(&events);
    let file = PathBuf::from(opened["file"].as_str().expect("session file"));
    let body = fs::read_to_string(&file).expect("session body");
    assert!(body.contains(task), "{body}");
    let session_dir = PathBuf::from(opened["session_dir"].as_str().expect("session dir"));
    let state: Value =
        serde_json::from_slice(&fs::read(session_dir.join("state.json")).expect("state.json"))
            .expect("state json");
    assert!(
        state["pane_id"].is_string(),
        "observation pane did not open: {state}"
    );
    let view = fs::read_to_string(session_dir.join("view.log")).expect("view.log");
    assert!(view.contains("=== turn 1 start ==="), "{view}");
    assert!(view.contains("[answer]"), "{view}");
    assert!(view.contains("=== turn 1 end (completed) ==="), "{view}");
    eprintln!(
        "omp real record: {} (pane {})",
        file.display(),
        state["pane_id"]
    );
    runtime.shutdown().expect("shutdown");
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

fn open_runtime_real(name: &str, idle: Duration) -> (RuntimeGuard, PathBuf, PathBuf, PathBuf) {
    let cred = std::env::temp_dir().join(format!(
        "hctl2-omp-real-cred-{}-{}",
        name,
        std::process::id()
    ));
    let root = std::env::temp_dir().join(format!(
        "hctl2-omp-real-root-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&cred);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&cred).unwrap();
    fs::write(cred.join("pair.key"), b"secret-credential").unwrap();
    let exec = root.join("dispatches/job");
    fs::create_dir_all(&exec).unwrap();
    let state = agency::herdr::state_dir(&exec, &cred).unwrap();
    let agent = state.join("omp-home/.omp/agent");
    fs::create_dir_all(&agent).unwrap();
    let db = std::env::var("HCTL2_OMP_AGENT_DB").expect("HCTL2_OMP_AGENT_DB");
    let config = std::env::var("HCTL2_OMP_CONFIG").expect("HCTL2_OMP_CONFIG");
    fs::copy(db, agent.join("agent.db")).unwrap();
    fs::copy(config, agent.join("config.yml")).unwrap();
    fs::set_permissions(agent.join("agent.db"), fs::Permissions::from_mode(0o600)).unwrap();
    let claude = root.join("claude-stub");
    write_fixture(&claude, "#!/bin/sh\necho '2.1.289 (Claude Code)'\n");
    let omp = std::process::Command::new("/usr/bin/which")
        .arg("omp")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_owned()))
        .expect("omp on PATH");
    let runtime = InstalledHerdr::open_with_omp(binary(), &claude, &omp, idle)
        .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    assert!(
        runtime.omp_skip().is_none(),
        "{}",
        runtime.omp_skip().unwrap_or("")
    );
    (RuntimeGuard(runtime), cred, root, exec)
}

fn find_session_file(exec: &Path, cred: &Path, name: &str) -> Option<PathBuf> {
    let state = agency::herdr::state_dir(exec, cred).ok()?;
    for entry in fs::read_dir(&state).ok()?.flatten() {
        let sessions = entry.path().join("sessions");
        let candidate = sessions.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn session_json(exec: &Path, cred: &Path, name: &str) -> Value {
    let path = find_session_file(exec, cred, name).unwrap_or_else(|| panic!("missing {name}"));
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn read_session_json(file: &Path, name: &str) -> Value {
    let path = file.parent().unwrap().join(name);
    serde_json::from_slice(&fs::read(&path).unwrap_or_else(|error| panic!("{name}: {error}")))
        .unwrap()
}

fn binary() -> PathBuf {
    let path = PathBuf::from(std::env::var("HCTL2_LOCKED_HERDR").expect("locked Herdr"));
    let path = path.canonicalize().expect("locked Herdr");
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();
    path
}

struct RuntimeGuard(InstalledHerdr);

impl Drop for RuntimeGuard {
    fn drop(&mut self) {
        let _ = Runtime::shutdown(&self.0);
    }
}

impl std::ops::Deref for RuntimeGuard {
    type Target = InstalledHerdr;
    fn deref(&self) -> &InstalledHerdr {
        &self.0
    }
}

fn open_runtime(name: &str, idle: Duration) -> (RuntimeGuard, PathBuf, PathBuf, PathBuf) {
    let cred = std::env::temp_dir().join(format!("hctl2-omp-cred-{}-{}", name, std::process::id()));
    let root = std::env::temp_dir().join(format!("hctl2-omp-root-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&cred);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&cred).unwrap();
    fs::write(cred.join("pair.key"), b"secret-credential").unwrap();
    let exec = root.join("dispatches/job");
    fs::create_dir_all(&exec).unwrap();
    let claude = root.join("claude-stub");
    write_fixture(&claude, "#!/bin/sh\necho '2.1.289 (Claude Code)'\n");
    let runtime = InstalledHerdr::open_with_omp(binary(), &claude, &omp_bin(), idle)
        .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    assert!(
        runtime.omp_skip().is_none(),
        "{}",
        runtime.omp_skip().unwrap_or("")
    );
    (RuntimeGuard(runtime), cred, root, exec)
}

struct SecretSpecimen {
    file: PathBuf,
    tree: Option<PathBuf>,
}

impl Drop for SecretSpecimen {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.file);
        if let Some(tree) = &self.tree {
            let _ = fs::remove_dir_all(tree);
        }
    }
}

impl SecretSpecimen {
    fn plant(label: &str) -> Self {
        if cfg!(target_os = "linux") {
            let tree = std::env::temp_dir()
                .join(format!("hctl2-omp-keyring-{label}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&tree);
            fs::create_dir_all(&tree).unwrap();
            let file = tree.join("login.keyring");
            fs::write(&file, b"trial-keyring-secret").unwrap();
            Self {
                file,
                tree: Some(tree),
            }
        } else {
            let dir = PathBuf::from(std::env::var_os("HOME").expect("HOME"))
                .join(".local/share/keyrings");
            fs::create_dir_all(&dir).unwrap();
            let file = dir.join(format!("hctl2-omp-{label}-{}.keyring", std::process::id()));
            fs::write(&file, b"trial-keyring-secret").unwrap();
            Self { file, tree: None }
        }
    }
}

fn start_task(
    runtime: &InstalledHerdr,
    exec: &Path,
    cred: &Path,
    key: &str,
    model: &str,
    task: &str,
    effects: &[&str],
) -> agency_proto::Result<Running> {
    let bundle = sealed_bundle(task);
    let mut document = sealed_spec(key).document;
    document.profession.reference.id = "omp".into();
    document.profession.model = model.into();
    document.bundle.digest = bundle.digest.clone();
    document.required_capabilities.isolation_effects =
        effects.iter().map(|item| (*item).to_string()).collect();
    let spec = Sealed::new(document).unwrap();
    runtime.start(&spec, &bundle, exec, cred)
}

fn dispatch(runtime: &InstalledHerdr, exec: &Path, cred: &Path, task: &str) -> Running {
    let bundle = sealed_bundle(task);
    let mut document = sealed_spec(task).document;
    document.profession.reference.id = "omp".into();
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

fn proposal(events: &[RuntimeEvent]) -> Option<(&str, &[u8])> {
    events.iter().find_map(|event| match event {
        RuntimeEvent::Proposal { schema, bytes, .. } => Some((schema.as_str(), bytes.as_slice())),
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
            reference: frozen("omp"),
            harness: frozen("herdr"),
            model: "fixture/model".into(),
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

/// Materialize a fixture with a child process. The test process must never hold
/// the write descriptor of a file it will execute: another test thread's forked
/// child can inherit it and answer ETXTBSY (Linux execve(2)). The sandbox only
/// admits files under the test root, so the child writes them there.
fn write_fixture(dest: &Path, body: &str) {
    use std::io::Write as _;
    let mut child = std::process::Command::new("/bin/sh")
        .args(["-c", "cat > \"$1\"; chmod 755 \"$1\"", "sh"])
        .arg(dest)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("write fixture");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(body.as_bytes())
        .expect("fixture body");
    assert!(
        child.wait().unwrap().success(),
        "write {} failed",
        dest.display()
    );
}

fn install_fixture(source: &Path, dest: &Path) {
    let status = std::process::Command::new("install")
        .args(["-m", "755"])
        .arg(source)
        .arg(dest)
        .status()
        .expect("install fixture");
    assert!(status.success(), "install {} failed", dest.display());
}

fn omp_bin() -> PathBuf {
    static BIN: OnceLock<PathBuf> = OnceLock::new();
    BIN.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("hctl2-omp-bin-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("omp");
        install_fixture(
            &PathBuf::from(std::env::var("HCTL2_OMP_FIXTURE").expect("HCTL2_OMP_FIXTURE")),
            &dest,
        );
        dest
    })
    .clone()
}

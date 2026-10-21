//! Deterministic and ignored live tests for the Antigravity CLI (`antigravity-cli`) adapter.
use agency::{
    Agency,
    launch::InstalledHerdr,
    observation::{ObservationConfig, TURN_OUTPUT_SCHEMA},
    runtime::{Running, Runtime, RuntimeEvent},
};
use agency_proto::{
    Capabilities, EvidenceLevel, ExecutionSpec, FrozenRef, InputPolicy, Owner, OwnerKind, Result,
    Sealed,
    context::{Bundle, Delivery, Entry},
    hash,
};
use serde_json::Value;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

static GATE: Mutex<()> = Mutex::new(());

struct SecretSpecimen {
    keyring: PathBuf,
    user_gemini: Option<PathBuf>,
}

impl Drop for SecretSpecimen {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.keyring);
        if let Some(path) = &self.user_gemini {
            let _ = fs::remove_file(path);
        }
    }
}

impl SecretSpecimen {
    fn plant(label: &str) -> Self {
        let keyring_dir = if cfg!(target_os = "macos") {
            PathBuf::from(std::env::var_os("HOME").expect("HOME")).join(".local/share/keyrings")
        } else {
            std::env::temp_dir().join(format!("hctl2-agy-keyring-{label}-{}", std::process::id()))
        };
        fs::create_dir_all(&keyring_dir).unwrap();
        let keyring = keyring_dir.join(format!("hctl2-agy-{label}-{}.keyring", std::process::id()));
        fs::write(&keyring, b"secret-keyring").unwrap();
        let user_gemini = std::env::var_os("HOME").map(|home| {
            let dir = PathBuf::from(home).join(".gemini");
            let _ = fs::create_dir_all(&dir);
            let file = dir.join(format!(
                "hctl2-agy-user-secret-{label}-{}",
                std::process::id()
            ));
            fs::write(&file, b"personal-gemini-secret").unwrap();
            file
        });
        Self {
            keyring,
            user_gemini,
        }
    }
}

#[test]
fn agy_fixture_returns_multiple_turns_in_one_process_and_verifies_transcript_and_special_inputs() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("agy-ok", Duration::from_secs(300));
    let long_block = "x".repeat(4096);
    let first_task =
        format!("!echo SHOULD_NOT_RUN_AS_LOCAL_SHELL\n/help\nLine three\twith\ttabs\n{long_block}");
    let mut first = start_task(
        &runtime,
        &exec,
        &cred,
        "turn-1",
        "fixture/model",
        &first_task,
        &[],
    )
    .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    let first_events = collect(&mut first, Duration::from_secs(20));
    let (schema, bytes) =
        proposal(&first_events).unwrap_or_else(|| panic!("{}", show(&first_events)));
    assert_eq!(schema, TURN_OUTPUT_SCHEMA);
    assert!(
        String::from_utf8_lossy(bytes).starts_with("AGY_ANSWER:"),
        "{}",
        show(&first_events)
    );
    assert!(
        first_events
            .iter()
            .any(|event| matches!(event, RuntimeEvent::TurnReturned)),
        "{}",
        show(&first_events)
    );
    let first_open = opened(&first_events);
    assert_eq!(first_open["resumed"], false);
    assert_eq!(first_open["resume_failed"], false);
    let pane_id = first_open["pane"]
        .as_str()
        .expect("observation pane id")
        .to_owned();
    let session_dir = PathBuf::from(first_open["session_dir"].as_str().expect("session_dir"));
    assert_eq!(
        session_dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str()),
        Some("antigravity"),
        "{}",
        session_dir.display()
    );
    let conv_1 = first_open["conversation"].as_str().unwrap().to_owned();
    let argv_1 = agy_home_json(&exec, &cred, "argv.json");
    let pid_1 = argv_1["pid"].as_u64().unwrap();
    assert_eq!(argv_1["spawn_seq"], 1);
    let args = argv_1["args"].as_array().unwrap();
    assert!(
        args.windows(2)
            .any(|pair| pair[0] == "--output-format" && pair[1] == "stream-json"),
        "{argv_1}"
    );
    assert!(
        args.windows(2)
            .any(|pair| pair[0] == "--input-format" && pair[1] == "stream-json"),
        "{argv_1}"
    );
    assert!(
        args.iter().any(|arg| arg == "--disable-slash-commands"),
        "{argv_1}"
    );
    assert!(args.iter().any(|arg| arg == "--print="), "{argv_1}");
    assert!(
        args.windows(2)
            .any(|pair| pair[0] == "--model" && pair[1] == "fixture/model"),
        "{argv_1}"
    );
    let home = argv_1["home"].as_str().expect("managed home");
    assert_eq!(Path::new(home), session_dir.join("home"));
    let catalog = runtime.catalog().expect("catalog");
    let agy_prof = catalog
        .professions
        .iter()
        .find(|p| p.reference.id == "antigravity-cli")
        .expect("antigravity-cli cataloged");
    assert_eq!(agy_prof.harness.id, "antigravity");
    assert!(catalog.harnesses.contains(&agy_prof.harness));
    assert_eq!(argv_1["settings"]["toolPermission"], "request-review");
    assert_eq!(argv_1["settings"]["allowNonWorkspaceAccess"], false);
    assert!(
        argv_1["hooks"]["hooks"]["PreToolUse"].is_array(),
        "{argv_1}"
    );

    // Second turn on the same selection reuses the live stream-json process and
    // ignores subagent step_update / foreign result frames.
    let second_task = "Second turn body with FIXTURE_SUBAGENT";
    let mut second = start_task(
        &runtime,
        &exec,
        &cred,
        "turn-2",
        "fixture/model",
        second_task,
        &[],
    )
    .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    let second_events = collect(&mut second, Duration::from_secs(20));
    let (second_schema, second_bytes) =
        proposal(&second_events).unwrap_or_else(|| panic!("{}", show(&second_events)));
    assert_eq!(second_schema, TURN_OUTPUT_SCHEMA);
    assert!(
        String::from_utf8_lossy(second_bytes).ends_with(":2"),
        "{}",
        String::from_utf8_lossy(second_bytes)
    );
    let argv_2 = agy_home_json(&exec, &cred, "argv.json");
    assert_eq!(argv_2["pid"].as_u64().unwrap(), pid_1);
    assert_eq!(argv_2["spawn_seq"], 1);
    assert_eq!(argv_2["conversation_id"], conv_1);

    let state_json: Value =
        serde_json::from_slice(&fs::read(session_dir.join("state.json")).unwrap()).unwrap();
    assert_eq!(state_json["conversation_id"], conv_1);
    assert_eq!(state_json["pane_id"], pane_id);
    assert_eq!(state_json["turn"], 2);
    assert_eq!(state_json["schema"], TURN_OUTPUT_SCHEMA);

    let events_jsonl = fs::read_to_string(session_dir.join("events.jsonl")).unwrap();
    let event_kinds: Vec<String> = events_jsonl
        .lines()
        .map(|line| {
            serde_json::from_str::<Value>(line).unwrap()["event"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert!(event_kinds.contains(&"init".to_owned()), "{events_jsonl}");
    assert!(event_kinds.contains(&"user".to_owned()), "{events_jsonl}");
    assert!(
        event_kinds.contains(&"step_update".to_owned()),
        "{events_jsonl}"
    );
    assert!(event_kinds.contains(&"result".to_owned()), "{events_jsonl}");

    let view_log = fs::read_to_string(session_dir.join("view.log")).unwrap();
    assert!(view_log.contains("=== turn 1 start ==="), "{view_log}");
    assert!(view_log.contains("[tool] view_file"), "{view_log}");
    assert!(view_log.contains("AGY_ANSWER:"), "{view_log}");
    assert!(
        view_log.contains("=== turn 2 end (completed) ==="),
        "{view_log}"
    );

    let mux = runtime.multiplexer(&exec, &cred).expect("multiplexer");
    let pane_wait = Instant::now() + Duration::from_secs(5);
    let visible = loop {
        let text = mux.read_pane(&pane_id).expect("read observation pane");
        if text.contains("=== turn 2 end (completed) ===") {
            break text;
        }
        assert!(
            Instant::now() < pane_wait,
            "observation pane did not render view.log: {text}"
        );
        thread::sleep(Duration::from_millis(50));
    };
    assert!(visible.contains("[tool] view_file"), "{visible}");
    assert!(visible.contains("AGY_ANSWER:"), "{visible}");

    let transcript_path = PathBuf::from(first_open["file"].as_str().unwrap());
    let transcript = fs::read_to_string(&transcript_path).unwrap();
    let contents: Vec<String> = transcript
        .lines()
        .map(|line| {
            serde_json::from_str::<Value>(line).unwrap()["content"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert!(
        contents.iter().any(|content| content.contains(&first_task)),
        "{transcript}"
    );
    assert!(
        contents.iter().any(|content| content.contains(second_task)),
        "{transcript}"
    );

    runtime.shutdown().expect("shutdown");
    cleanup(&cred, &root);
}

#[test]
fn agy_fixture_succeeds_with_observation_pane_disabled_without_starting_multiplexer() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime_with_observation(
        "agy-no-pane",
        Duration::from_secs(300),
        ObservationConfig::disabled(),
    );
    let mut running = dispatch(&runtime, &exec, &cred, "turn-without-pane");
    let events = collect(&mut running, Duration::from_secs(20));
    let (schema, bytes) = proposal(&events).unwrap_or_else(|| panic!("{}", show(&events)));
    assert_eq!(schema, TURN_OUTPUT_SCHEMA);
    assert!(String::from_utf8_lossy(bytes).starts_with("AGY_ANSWER:"));
    let open = opened(&events);
    assert!(open["pane"].is_null(), "{open}");
    assert_eq!(runtime.servers_started(), 0);
    let session_dir = PathBuf::from(open["session_dir"].as_str().expect("session_dir"));
    let state_json: Value =
        serde_json::from_slice(&fs::read(session_dir.join("state.json")).unwrap()).unwrap();
    assert!(state_json["pane_id"].is_null(), "{state_json}");
    assert!(session_dir.join("events.jsonl").is_file());
    assert!(session_dir.join("view.log").is_file());
    runtime.shutdown().expect("shutdown");
    cleanup(&cred, &root);
}

#[test]
fn agy_fixture_rejects_a_transcript_that_does_not_match_the_task() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("agy-mismatch", Duration::from_secs(300));
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
    cleanup(&cred, &root);
}

#[test]
fn agy_fixture_login_required_fails_clearly_without_hanging() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("agy-login", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "FIXTURE_LOGIN");
    let events = collect(&mut running, Duration::from_secs(20));
    assert_eq!(
        protocol(&events),
        Some("HARNESS_LOGIN_REQUIRED"),
        "{}",
        show(&events)
    );
    assert!(proposal(&events).is_none(), "{}", show(&events));

    // Also verify missing token file in managed profile is rejected before hanging.
    let state = agency::herdr::state_dir(&exec, &cred).unwrap();
    let _ = fs::remove_file(state.join("agy-home/.gemini/antigravity-cli/antigravity-oauth-token"));
    let mut missing = dispatch(&runtime, &exec, &cred, "NORMAL_TURN_WITHOUT_TOKEN");
    let missing_events = collect(&mut missing, Duration::from_secs(20));
    assert_eq!(
        protocol(&missing_events),
        Some("HARNESS_LOGIN_REQUIRED"),
        "{}",
        show(&missing_events)
    );
    runtime.shutdown().expect("shutdown");
    cleanup(&cred, &root);
}

#[test]
fn agy_fixture_interrupt_stops_the_turn() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("agy-interrupt", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "FIXTURE_INTERRUPT");
    let start = Instant::now() + Duration::from_secs(20);
    while !agy_home_file(&exec, &cred, "turn-started").exists() {
        assert!(Instant::now() < start, "turn did not start");
        thread::sleep(Duration::from_millis(20));
    }
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
    assert!(proposal(&events).is_none(), "{}", show(&events));
    runtime.shutdown().expect("shutdown");
    cleanup(&cred, &root);
}

#[test]
fn agy_fixture_idle_eviction_resumes_and_failed_resume_starts_a_new_conversation() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("agy-resume", Duration::from_millis(250));
    let mut first = dispatch(&runtime, &exec, &cred, "turn-before-idle");
    let first_events = collect(&mut first, Duration::from_secs(20));
    let first_open = opened(&first_events);
    let first_conv = first_open["conversation"].as_str().unwrap().to_owned();
    assert_eq!(first_open["resumed"], false);
    assert_eq!(first_open["resume_failed"], false);

    // Wait for idle timeout to close the process, then verify next turn passes --conversation.
    thread::sleep(Duration::from_millis(700));
    let mut second = dispatch(&runtime, &exec, &cred, "turn-after-idle");
    let second_events = collect(&mut second, Duration::from_secs(20));
    let second_open = opened(&second_events);
    assert_eq!(second_open["resumed"], true, "{}", show(&second_events));
    assert_eq!(
        second_open["resume_failed"],
        false,
        "{}",
        show(&second_events)
    );
    assert_eq!(second_open["conversation"].as_str().unwrap(), first_conv);
    let argv = agy_home_json(&exec, &cred, "argv.json");
    let args = argv["args"].as_array().unwrap();
    assert!(
        args.windows(2)
            .any(|pair| pair[0] == "--conversation" && pair[1] == first_conv),
        "{argv}"
    );

    // Wait for idle eviction again, then force resume failure and verify fallback.
    thread::sleep(Duration::from_millis(700));
    fs::write(agy_home_file(&exec, &cred, "resume-fail"), b"1").unwrap();
    let mut third = dispatch(&runtime, &exec, &cred, "turn-after-failed-resume");
    let third_events = collect(&mut third, Duration::from_secs(20));
    let third_open = opened(&third_events);
    assert_eq!(third_open["resumed"], false, "{}", show(&third_events));
    assert_eq!(third_open["resume_failed"], true, "{}", show(&third_events));
    assert_ne!(
        third_open["conversation"].as_str().unwrap(),
        first_conv,
        "{}",
        show(&third_events)
    );
    runtime.shutdown().expect("shutdown");
    cleanup(&cred, &root);
}

#[test]
fn agy_reaps_leftover_process_when_agency_reopens() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("agy-reap", Duration::from_secs(300));
    let mut running = dispatch(&runtime, &exec, &cred, "keep-alive-for-reap");
    let events = collect(&mut running, Duration::from_secs(20));
    assert!(proposal(&events).is_some(), "{}", show(&events));
    let argv = agy_home_json(&exec, &cred, "argv.json");
    let pid = argv["pid"].as_u64().unwrap() as u32;
    assert!(
        cred.join("agy-processes")
            .join(format!("{pid}.json"))
            .is_file()
    );

    // Opening Agency on the same credential root reaps the recorded agy process.
    let agency = Agency::open(
        &cred,
        std::sync::Arc::new(InstalledHerdr::for_test(binary(), |_, _, _| {
            Ok("exit 0\n".into())
        })),
    )
    .expect("reopen agency");
    drop(agency);
    assert!(
        !cred
            .join("agy-processes")
            .join(format!("{pid}.json"))
            .exists()
    );
    let ps = Command::new("/bin/ps")
        .args(["-p", &pid.to_string(), "-o", "stat="])
        .output()
        .unwrap();
    let stat = String::from_utf8_lossy(&ps.stdout).trim().to_owned();
    assert!(
        !ps.status.success() || stat.is_empty() || stat.starts_with('Z'),
        "leftover agy process {pid} survived Agency::open: {stat}"
    );
    runtime.shutdown().expect("shutdown");
    cleanup(&cred, &root);
}

#[test]
fn agy_read_only_cannot_read_credentials_keyring_user_gemini_or_gh_auth() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let specimen = SecretSpecimen::plant("agy-read");
    let (runtime, cred, root, exec) = open_runtime("agy-secret", Duration::from_secs(300));
    let cred_file = cred.join("pair.key").canonicalize().unwrap();
    let mut paths = vec![cred_file, specimen.keyring.clone()];
    if let Some(user_gemini) = &specimen.user_gemini {
        paths.push(user_gemini.clone());
    }
    let task = format!(
        "HCTL2_SECRET_PROBE {}\n",
        serde_json::json!({"secret_paths": paths})
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
    let probe = agy_home_json(&exec, &cred, "secret-probe.json");
    for read in probe["reads"].as_array().unwrap() {
        assert_eq!(read["readable"], false, "{probe}");
    }
    assert_eq!(probe["gh_authenticated"], false, "{probe}");
    assert_eq!(probe["gh_token_present"], false);
    assert_eq!(probe["dbus_present"], false);
    runtime.shutdown().expect("shutdown");
    cleanup(&cred, &root);
}

#[test]
fn agy_read_only_cannot_write_the_source_repo() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let repo = std::env::temp_dir().join(format!("hctl2-agy-src-repo-{}", std::process::id()));
    let _ = fs::remove_dir_all(&repo);
    fs::create_dir_all(&repo).unwrap();
    fs::write(repo.join("keep"), b"original").unwrap();
    let (runtime, cred, root, exec) = open_runtime("agy-src-repo", Duration::from_secs(300));
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
    let probe = agy_home_json(&exec, &cred, "secret-probe.json");
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
    cleanup(&cred, &root);
}

#[test]
fn agy_fixture_dead_between_turns_process_is_resumed_before_next_delivery() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime_with_observation(
        "agy-dead-between-turns",
        Duration::from_secs(300),
        ObservationConfig::disabled(),
    );
    let mut first = dispatch(&runtime, &exec, &cred, "first-turn-before-kill");
    let first_events = collect(&mut first, Duration::from_secs(20));
    assert!(proposal(&first_events).is_some(), "{}", show(&first_events));
    let first_open = opened(&first_events);
    let conv_1 = first_open["conversation"].as_str().unwrap().to_owned();
    let argv_1 = agy_home_json(&exec, &cred, "argv.json");
    let pid_1 = argv_1["pid"].as_u64().unwrap() as u32;

    // Kill the live fixture process between turns (before idle timeout) and wait for exit.
    let _ = Command::new("/bin/kill")
        .args(["-s", "TERM", &pid_1.to_string()])
        .status();
    let wait_dead = Instant::now() + Duration::from_secs(5);
    loop {
        let ps = Command::new("/bin/ps")
            .args(["-p", &pid_1.to_string(), "-o", "stat="])
            .output()
            .unwrap();
        let stat = String::from_utf8_lossy(&ps.stdout).trim().to_owned();
        if !ps.status.success() || stat.is_empty() || stat.starts_with('Z') {
            break;
        }
        assert!(Instant::now() < wait_dead, "process {pid_1} did not exit");
        thread::sleep(Duration::from_millis(20));
    }

    // Dispatching the second turn on the same selection before idle must detect the dead
    // process prior to writing stdin, resume the saved conversation, and return once.
    let mut second = dispatch(&runtime, &exec, &cred, "second-turn-after-kill");
    let second_events = collect(&mut second, Duration::from_secs(20));
    let second_open = opened(&second_events);
    assert_eq!(second_open["resumed"], true, "{}", show(&second_events));
    assert_eq!(
        second_open["resume_failed"],
        false,
        "{}",
        show(&second_events)
    );
    assert_eq!(second_open["conversation"].as_str().unwrap(), conv_1);
    let proposals: Vec<_> = second_events
        .iter()
        .filter(|event| matches!(event, RuntimeEvent::Proposal { .. }))
        .collect();
    assert_eq!(proposals.len(), 1, "{}", show(&second_events));
    let returned = second_events
        .iter()
        .filter(|event| matches!(event, RuntimeEvent::TurnReturned))
        .count();
    assert_eq!(returned, 1, "{}", show(&second_events));
    let argv_2 = agy_home_json(&exec, &cred, "argv.json");
    assert_ne!(argv_2["pid"].as_u64().unwrap() as u32, pid_1);
    assert_eq!(argv_2["spawn_seq"], 2);
    let args = argv_2["args"].as_array().unwrap();
    assert!(
        args.windows(2)
            .any(|pair| pair[0] == "--conversation" && pair[1] == conv_1),
        "{argv_2}"
    );

    runtime.shutdown().expect("shutdown");
    cleanup(&cred, &root);
}

#[test]
fn agy_fixture_repeated_identical_prompt_succeeds_and_mismatched_current_turn_fails_even_with_matching_history()
 {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime_with_observation(
        "agy-repeat-history",
        Duration::from_millis(250),
        ObservationConfig::disabled(),
    );
    let prompt = "repeat this task";

    // Turn 1: initial turn records `repeat this task`.
    let mut first = dispatch(&runtime, &exec, &cred, prompt);
    let first_events = collect(&mut first, Duration::from_secs(20));
    assert!(proposal(&first_events).is_some(), "{}", show(&first_events));
    let conv_1 = opened(&first_events)["conversation"]
        .as_str()
        .unwrap()
        .to_owned();

    // Turn 2 (same process): repeating the exact same prompt succeeds.
    let mut second = dispatch(&runtime, &exec, &cred, prompt);
    let second_events = collect(&mut second, Duration::from_secs(20));
    assert!(
        proposal(&second_events).is_some(),
        "{}",
        show(&second_events)
    );

    // Wait for idle eviction, then Turn 3 (resumed): repeating the exact same prompt succeeds.
    thread::sleep(Duration::from_millis(700));
    let mut third = dispatch(&runtime, &exec, &cred, prompt);
    let third_events = collect(&mut third, Duration::from_secs(20));
    let third_open = opened(&third_events);
    assert_eq!(third_open["resumed"], true, "{}", show(&third_events));
    assert_eq!(third_open["conversation"].as_str().unwrap(), conv_1);
    assert!(proposal(&third_events).is_some(), "{}", show(&third_events));

    // Wait for idle eviction again, enable transcript tampering for the next turn, and
    // verify that even with three matching historical USER_INPUT entries, a mismatched
    // current turn is rejected after resume.
    thread::sleep(Duration::from_millis(700));
    fs::write(agy_home_file(&exec, &cred, "tamper-transcript"), b"1").unwrap();
    let mut fourth = dispatch(&runtime, &exec, &cred, prompt);
    let fourth_events = collect(&mut fourth, Duration::from_secs(20));
    let fourth_open = opened(&fourth_events);
    assert_eq!(fourth_open["resumed"], true, "{}", show(&fourth_events));
    assert_eq!(fourth_open["conversation"].as_str().unwrap(), conv_1);
    assert_eq!(
        protocol(&fourth_events),
        Some("STANDBY_PROMPT_MISMATCH"),
        "{}",
        show(&fourth_events)
    );
    assert!(
        proposal(&fourth_events).is_none(),
        "{}",
        show(&fourth_events)
    );

    runtime.shutdown().expect("shutdown");
    cleanup(&cred, &root);
}

#[test]
fn agy_fixture_concurrent_read_and_write_selections_isolate_settings_and_hooks() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime_with_observation(
        "agy-slot-isolation",
        Duration::from_secs(300),
        ObservationConfig::disabled(),
    );
    let repo = root.join("source-repo");
    fs::create_dir_all(&repo).unwrap();
    Command::new("git")
        .args(["init", "--initial-branch=main", "--quiet"])
        .current_dir(&repo)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "trial@example.invalid"])
        .current_dir(&repo)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Trial"])
        .current_dir(&repo)
        .status()
        .unwrap();
    fs::write(repo.join("README.md"), b"hello\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "baseline", "--quiet"])
        .current_dir(&repo)
        .status()
        .unwrap();
    let baseline = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&repo)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_owned();

    let mut read_running = start_task_with_selection(
        &runtime,
        &exec,
        &cred,
        "read-slot",
        "selection-read",
        "fixture/model",
        "read-only turn",
        &[],
        None,
    )
    .unwrap();
    let mut write_running = start_task_with_selection(
        &runtime,
        &exec,
        &cred,
        "write-slot",
        "selection-write",
        "fixture/model",
        "write turn",
        &[],
        Some((&repo, &baseline)),
    )
    .unwrap();

    let read_events = collect(&mut read_running, Duration::from_secs(20));
    let write_events = collect(&mut write_running, Duration::from_secs(20));
    assert!(proposal(&read_events).is_some(), "{}", show(&read_events));

    let read_dir = PathBuf::from(opened(&read_events)["session_dir"].as_str().unwrap());
    let write_dir = PathBuf::from(opened(&write_events)["session_dir"].as_str().unwrap());
    assert_ne!(read_dir, write_dir);

    let read_argv: Value = serde_json::from_slice(
        &fs::read(read_dir.join("home/.gemini/antigravity-cli/argv.json")).unwrap(),
    )
    .unwrap();
    let write_argv: Value = serde_json::from_slice(
        &fs::read(write_dir.join("home/.gemini/antigravity-cli/argv.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(
        Path::new(read_argv["home"].as_str().unwrap()),
        read_dir.join("home")
    );
    assert_eq!(
        Path::new(write_argv["home"].as_str().unwrap()),
        write_dir.join("home")
    );
    assert_eq!(read_argv["settings"]["toolPermission"], "request-review");
    assert_eq!(write_argv["settings"]["toolPermission"], "always-proceed");
    assert_eq!(
        read_argv["hooks"]["hooks"]["PreToolUse"][0]["matcher"],
        "write_to_file|replace_file_content|multi_replace_file_content|sed_file|notebook_edit|run_command"
    );
    assert_eq!(
        write_argv["hooks"]["hooks"]["PreToolUse"][0]["matcher"],
        "run_command"
    );

    runtime.shutdown().expect("shutdown");
    cleanup(&cred, &root);
}

#[test]
fn agy_live_token_seeding_refuses_missing_env_without_copying_personal_home() {
    let temp = tempfile::tempdir().unwrap();
    let fake_home = temp.path().join("fake-personal-home");
    let personal_dir = fake_home.join(".gemini/antigravity-cli");
    fs::create_dir_all(&personal_dir).unwrap();
    let personal_token = personal_dir.join("antigravity-oauth-token");
    fs::write(&personal_token, b"personal-home-token-do-not-copy\n").unwrap();

    let state = temp.path().join("state");
    let err = seed_real_agy_profile_from(&state, None).unwrap_err();
    assert!(err.contains("HCTL2_AGY_OAUTH_TOKEN"), "{err}");
    assert!(
        !state
            .join("agy-home/.gemini/antigravity-cli/antigravity-oauth-token")
            .exists(),
        "missing HCTL2_AGY_OAUTH_TOKEN must not copy personal HOME credentials"
    );
}

#[test]
fn declared_no_network_is_refused_before_agy_starts() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (runtime, cred, root, exec) = open_runtime("agy-isolate", Duration::from_secs(300));
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
    cleanup(&cred, &root);
}

#[test]
#[ignore = "UNVERIFIED: installed agy and managed profile login required; HCTL2_HARNESS_LIVE=1 and HCTL2_AGY_OAUTH_TOKEN=<path>"]
fn agy_real_read_only_research_returns_a_proposal() {
    let _gate = GATE.lock().unwrap_or_else(|poison| poison.into_inner());
    assert_eq!(
        std::env::var("HCTL2_HARNESS_LIVE").as_deref(),
        Ok("1"),
        "UNVERIFIED: local agy login required"
    );
    let (runtime, cred, root, exec) = open_runtime_real("agy-real", Duration::from_secs(300));
    let task = "Reply with exactly HCTL2_AGY_REAL_OK and do not call any tools.";
    let mut running = start_task(&runtime, &exec, &cred, "real", "none", task, &[])
        .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    let events = collect(&mut running, Duration::from_secs(120));
    let (schema, bytes) = proposal(&events).unwrap_or_else(|| panic!("{}", show(&events)));
    assert_eq!(schema, TURN_OUTPUT_SCHEMA, "{}", show(&events));
    let text = String::from_utf8_lossy(bytes);
    assert!(text.contains("HCTL2_AGY_REAL_OK"), "{text}");
    let opened = opened(&events);
    let pane_id = opened["pane"].as_str().expect("observation pane id");
    let session_dir = PathBuf::from(opened["session_dir"].as_str().expect("session_dir"));
    assert!(session_dir.join("state.json").is_file());
    assert!(session_dir.join("events.jsonl").is_file());
    let view_log = fs::read_to_string(session_dir.join("view.log")).expect("view.log");
    assert!(view_log.contains("HCTL2_AGY_REAL_OK"), "{view_log}");
    let mux = runtime.multiplexer(&exec, &cred).expect("multiplexer");
    let pane_wait = Instant::now() + Duration::from_secs(5);
    loop {
        let visible = mux.read_pane(pane_id).expect("read observation pane");
        if visible.contains("HCTL2_AGY_REAL_OK") {
            break;
        }
        assert!(
            Instant::now() < pane_wait,
            "observation pane did not render HCTL2_AGY_REAL_OK: {visible}"
        );
        thread::sleep(Duration::from_millis(50));
    }
    let file = PathBuf::from(opened["file"].as_str().expect("transcript file"));
    let body = fs::read_to_string(&file).expect("transcript body");
    assert!(body.contains(task), "{body}");
    eprintln!("agy real transcript: {}", file.display());
    runtime.shutdown().expect("shutdown");
    cleanup(&cred, &root);
}

fn open_runtime(name: &str, idle: Duration) -> (RuntimeGuard, PathBuf, PathBuf, PathBuf) {
    open_runtime_with_observation(name, idle, ObservationConfig::enabled())
}

fn open_runtime_with_observation(
    name: &str,
    idle: Duration,
    observation: ObservationConfig,
) -> (RuntimeGuard, PathBuf, PathBuf, PathBuf) {
    let cred = std::env::temp_dir().join(format!("hctl2-agy-cred-{}-{}", name, std::process::id()));
    let root = std::env::temp_dir().join(format!("hctl2-agy-root-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&cred);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&cred).unwrap();
    fs::write(cred.join("pair.key"), b"secret-credential").unwrap();
    let exec = root.join("dispatches/job");
    fs::create_dir_all(&exec).unwrap();
    let state = agency::herdr::state_dir(&exec, &cred).unwrap();
    let agy_dir = state.join("agy-home/.gemini/antigravity-cli");
    fs::create_dir_all(&agy_dir).unwrap();
    fs::write(
        agy_dir.join("antigravity-oauth-token"),
        b"fixture-oauth-token\n",
    )
    .unwrap();
    fs::set_permissions(
        agy_dir.join("antigravity-oauth-token"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    let claude = root.join("claude-stub");
    write_fixture(&claude, "#!/bin/sh\necho '2.1.289 (Claude Code)'\n");
    let fixture = PathBuf::from(std::env::var("HCTL2_AGY_FIXTURE").expect("HCTL2_AGY_FIXTURE"))
        .canonicalize()
        .expect("agy fixture");
    let runtime = InstalledHerdr::open_with_agy(binary(), &claude, &fixture, idle)
        .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message))
        .with_observation(observation);
    assert!(
        runtime.agy_skip().is_none(),
        "{}",
        runtime.agy_skip().unwrap_or("")
    );
    (RuntimeGuard(runtime), cred, root, exec)
}

fn seed_real_agy_profile_from(
    state: &Path,
    explicit_token: Option<&std::ffi::OsStr>,
) -> std::result::Result<PathBuf, String> {
    let token_src = explicit_token
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| {
            "UNVERIFIED: HCTL2_AGY_OAUTH_TOKEN must point to a dedicated managed oauth token file; personal HOME is never copied".to_owned()
        })?;
    let meta = fs::symlink_metadata(&token_src)
        .map_err(|e| format!("HCTL2_AGY_OAUTH_TOKEN {}: {e}", token_src.display()))?;
    if meta.file_type().is_symlink() || !meta.file_type().is_file() {
        return Err(format!(
            "HCTL2_AGY_OAUTH_TOKEN {} must be a regular file",
            token_src.display()
        ));
    }
    let agy_dir = state.join("agy-home/.gemini/antigravity-cli");
    fs::create_dir_all(&agy_dir).map_err(|e| e.to_string())?;
    let dest = agy_dir.join("antigravity-oauth-token");
    fs::copy(&token_src, &dest).map_err(|e| e.to_string())?;
    fs::set_permissions(&dest, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    Ok(dest)
}

fn open_runtime_real(name: &str, idle: Duration) -> (RuntimeGuard, PathBuf, PathBuf, PathBuf) {
    let cred = std::env::temp_dir().join(format!(
        "hctl2-agy-real-cred-{}-{}",
        name,
        std::process::id()
    ));
    let root = std::env::temp_dir().join(format!(
        "hctl2-agy-real-root-{}-{}",
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
    seed_real_agy_profile_from(&state, std::env::var_os("HCTL2_AGY_OAUTH_TOKEN").as_deref())
        .unwrap_or_else(|error| panic!("{error}"));
    let claude = root.join("claude-stub");
    write_fixture(&claude, "#!/bin/sh\necho '2.1.289 (Claude Code)'\n");
    let agy = Command::new("/usr/bin/which")
        .arg("agy")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_owned()))
        .expect("agy on PATH");
    let runtime = InstalledHerdr::open_with_agy(binary(), &claude, &agy, idle)
        .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message));
    assert!(
        runtime.agy_skip().is_none(),
        "{}",
        runtime.agy_skip().unwrap_or("")
    );
    (RuntimeGuard(runtime), cred, root, exec)
}

fn cleanup(cred: &Path, root: &Path) {
    if let Ok(state) = agency::herdr::state_dir(&root.join("dispatches/job"), cred) {
        let _ = fs::remove_dir_all(state);
    }
    let _ = fs::remove_dir_all(cred);
    let _ = fs::remove_dir_all(root);
}

fn agy_home_file(exec: &Path, cred: &Path, name: &str) -> PathBuf {
    let antigravity = agency::herdr::state_dir(exec, cred)
        .unwrap()
        .join("antigravity");
    let slot = fs::read_dir(&antigravity)
        .unwrap_or_else(|e| panic!("{}: {e}", antigravity.display()))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .find(|path| path.is_dir())
        .unwrap_or_else(|| panic!("no slot under {}", antigravity.display()));
    slot.join("home/.gemini/antigravity-cli").join(name)
}

fn agy_home_json(exec: &Path, cred: &Path, name: &str) -> Value {
    let path = agy_home_file(exec, cred, name);
    serde_json::from_slice(&fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
        .unwrap()
}

fn dispatch(runtime: &InstalledHerdr, exec: &Path, cred: &Path, task: &str) -> Running {
    start_task(runtime, exec, cred, task, "fixture/model", task, &[])
        .unwrap_or_else(|error| panic!("{}: {}", error.code, error.message))
}

fn start_task(
    runtime: &InstalledHerdr,
    exec: &Path,
    cred: &Path,
    key: &str,
    model: &str,
    task: &str,
    isolation: &[&str],
) -> Result<Running> {
    start_task_with_selection(
        runtime,
        exec,
        cred,
        key,
        "selection-1",
        model,
        task,
        isolation,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn start_task_with_selection(
    runtime: &InstalledHerdr,
    exec: &Path,
    cred: &Path,
    key: &str,
    selection_id: &str,
    model: &str,
    task: &str,
    isolation: &[&str],
    write_repo: Option<(&Path, &str)>,
) -> Result<Running> {
    let mut profession = runtime
        .catalog()?
        .professions
        .into_iter()
        .find(|item| item.reference.id == "antigravity-cli")
        .expect("antigravity-cli cataloged");
    profession.model = model.into();
    let mut spec = spec(key, now_ms() + 60_000);
    spec.selection = frozen(selection_id);
    spec.profession = profession;
    spec.required_capabilities.isolation_effects =
        isolation.iter().map(|item| (*item).into()).collect();
    let bundle = if let Some((repo_path, baseline)) = write_repo {
        let active = serde_json::json!({
            "lease_id": "lease-1",
            "generation": 1,
            "state": "active",
            "holder": {
                "kind": "invocation",
                "invocation_id": spec.owner.id,
                "invocation_version": 1
            }
        });
        let lease = FrozenRef {
            id: "lease-1".into(),
            revision: "1".into(),
            digest: hash(&agency_proto::canonical(&active).unwrap()),
        };
        let publication = serde_json::json!({
            "repo_id": "repo-1",
            "binding_version": 1,
            "branch_rule": "hctl2/{change_set}",
            "target_branch": "main",
            "allow_update": true,
            "description_source": "none",
            "requires_human_confirmation": false,
            "audit_scope": "minimal"
        });
        let mut policy = frozen("policy-1");
        policy.digest = hash(&agency_proto::canonical(&publication).unwrap());
        let mut pending = active;
        pending["state"] = serde_json::json!("pending");
        let boundary = serde_json::json!({
            "lease": {
                "pending": {
                    "change_set_id": "cs-1",
                    "repo_id": "repo-1",
                    "binding_version": 1,
                    "baseline_commit": baseline,
                    "version": 1,
                    "lease": pending
                },
                "previous": null
            },
            "review_publish_policy": policy,
            "authorization": "publish_for_review_not_integration",
            "repo_local_path": repo_path,
            "repo_local_machine": "control",
            "objective": task,
            "publication_target": publication
        });
        let boundary_bytes = agency_proto::canonical(&boundary).unwrap();
        let task_bytes = task.as_bytes().to_vec();
        spec.permissions = vec!["context.read".into(), "git.read".into(), "git.write".into()];
        spec.permission_digest = hash(b"write-permissions");
        spec.repo = Some(frozen("repo-1"));
        spec.base = Some(baseline.into());
        spec.write_lease = Some(lease);
        spec.review_publish_policy = Some(policy);
        Sealed::new(Bundle {
            id: "bundle-1".into(),
            manifest: spec.manifest.clone(),
            consumer: spec.owner.clone(),
            entries: vec![
                Entry {
                    source: frozen("task"),
                    description: "task".into(),
                    required: true,
                    offline_required: true,
                    bytes_digest: hash(&task_bytes),
                    delivery: Delivery::Inline { bytes: task_bytes },
                },
                Entry {
                    source: frozen(&format!("write-boundary/{}", spec.owner.id)),
                    description: "write-boundary".into(),
                    required: true,
                    offline_required: true,
                    bytes_digest: hash(&boundary_bytes),
                    delivery: Delivery::Inline {
                        bytes: boundary_bytes,
                    },
                },
            ],
            renderer: frozen("renderer"),
            tokenizer: frozen("tokenizer"),
            redaction: frozen("redaction"),
            compression: vec![],
            candidate_tokens: None,
            selected_tokens: None,
            delivered_tokens: None,
            permission_digest: spec.permission_digest.clone(),
            budget: spec.budget,
            retention: "test".into(),
        })
        .unwrap()
    } else {
        sealed_bundle(task)
    };
    spec.bundle.digest = bundle.digest.clone();
    runtime.start(&Sealed::new(spec)?, &bundle, exec, cred)
}

fn collect(running: &mut Running, timeout: Duration) -> Vec<RuntimeEvent> {
    let deadline = Instant::now() + timeout;
    let mut events = Vec::new();
    while Instant::now() < deadline {
        match running.events.recv_timeout(Duration::from_millis(50)) {
            Ok(event) => {
                let done = matches!(
                    event,
                    RuntimeEvent::DispatchReleased | RuntimeEvent::Exited { .. }
                );
                events.push(event);
                if done {
                    return events;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return events,
        }
    }
    events
}

fn proposal(events: &[RuntimeEvent]) -> Option<(&str, &[u8])> {
    events.iter().find_map(|event| match event {
        RuntimeEvent::Proposal {
            schema,
            bytes,
            source: EvidenceLevel::AdapterEvent,
        } => Some((schema.as_str(), bytes.as_slice())),
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
        .expect("session_opened")
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

fn write_fixture(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn binary() -> PathBuf {
    let path = PathBuf::from(std::env::var("HCTL2_LOCKED_HERDR").expect("locked Herdr"));
    let path = path.canonicalize().expect("locked Herdr");
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();
    path
}

fn frozen(id: &str) -> FrozenRef {
    FrozenRef {
        id: id.into(),
        revision: "1".into(),
        digest: hash(id.as_bytes()),
    }
}

fn spec(key: &str, deadline_ms: u64) -> ExecutionSpec {
    ExecutionSpec {
        owner: Owner {
            project: "project-1".into(),
            kind: OwnerKind::RoomInvocation,
            id: format!("inv-{key}"),
            generation: 1,
        },
        project: frozen("project-1"),
        selection: frozen("selection-1"),
        selection_policy_digest: hash(b"policy"),
        profession: agency_proto::Profession {
            reference: frozen("antigravity-cli"),
            harness: frozen("herdr"),
            model: "none".into(),
            persona: "antigravity".into(),
            terms: "test".into(),
            default_role: "worker".into(),
            skills: vec![],
            capabilities: Capabilities {
                stop: true,
                ..Capabilities::default()
            },
        },
        profile: frozen("profile-1"),
        manifest: frozen("manifest-1"),
        bundle: frozen("bundle-1"),
        binding: frozen("binding-1"),
        required_capabilities: Capabilities {
            stop: true,
            ..Capabilities::default()
        },
        input_policy: InputPolicy::NoInput,
        permission_digest: hash(b"permissions"),
        permissions: vec!["context.read".into()],
        budget: 1024,
        deadline_ms,
        repo: None,
        base: None,
        delivery_scope: vec!["room-1".into()],
        write_lease: None,
        review_publish_policy: None,
        idempotency_key: key.into(),
    }
}

fn sealed_bundle(task: &str) -> Sealed<Bundle> {
    let bytes = task.as_bytes().to_vec();
    Sealed::new(Bundle {
        id: "bundle-1".into(),
        manifest: frozen("manifest-1"),
        consumer: Owner {
            project: "project-1".into(),
            kind: OwnerKind::RoomInvocation,
            id: "inv-1".into(),
            generation: 1,
        },
        entries: vec![Entry {
            source: frozen("task"),
            description: "task".into(),
            required: true,
            offline_required: true,
            bytes_digest: hash(&bytes),
            delivery: Delivery::Inline { bytes },
        }],
        renderer: frozen("renderer"),
        tokenizer: frozen("tokenizer"),
        redaction: frozen("redaction"),
        compression: vec![],
        candidate_tokens: None,
        selected_tokens: None,
        delivered_tokens: None,
        permission_digest: hash(b"permissions"),
        budget: 1024,
        retention: "test".into(),
    })
    .unwrap()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
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

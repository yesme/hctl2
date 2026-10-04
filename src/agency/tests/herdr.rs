use agency::{
    confine, harness,
    herdr::{self, Client, HerdrRuntime, Server},
    runtime::Runtime,
};
use std::{collections::HashSet, path::PathBuf, time::Duration};

fn binary() -> PathBuf {
    PathBuf::from(std::env::var("HCTL2_LOCKED_HERDR").expect("locked Herdr binary"))
}

fn scratch(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let cred =
        std::env::temp_dir().join(format!("hctl2-herdr-cred-{}-{}", name, std::process::id()));
    let state =
        std::env::temp_dir().join(format!("hctl2-herdr-state-{}-{}", name, std::process::id()));
    let exec =
        std::env::temp_dir().join(format!("hctl2-herdr-exec-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&state);
    let _ = std::fs::remove_dir_all(&exec);
    std::fs::create_dir_all(&cred).unwrap();
    std::fs::create_dir_all(&exec).unwrap();
    std::fs::write(cred.join("pair.key"), b"secret-credential").unwrap();
    (cred, state, exec)
}

#[test]
fn locked_herdr_runs_a_program_in_a_pane() {
    let (cred, state, exec) = scratch("pane");
    let server = Server::start(&binary(), &state, &cred, &exec).unwrap();
    assert!(!state.starts_with(&cred));
    let client = Client::connect(&server.socket).unwrap();
    let pong = client.ping().unwrap();
    assert_eq!(pong["protocol"], 20);
    let text = herdr::run_command(
        &client,
        &exec,
        "probe",
        "printf '%s%s\\n' HCTL2 OUT1",
        "HCTL2OUT1",
    )
    .unwrap();
    assert!(text.contains("HCTL2OUT1"));
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&state);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn two_dispatches_share_one_herdr_and_stay_outside_the_credential_root() {
    let (cred, state, exec) = scratch("two");
    let server = Server::start(&binary(), &state, &cred, &exec).unwrap();
    let client = Client::connect(&server.socket).unwrap();
    let first = herdr::run_command(
        &client,
        &exec,
        "one",
        "printf '%s%s\\n' HCTL2 OUTA",
        "HCTL2OUTA",
    )
    .unwrap();
    let second = herdr::run_command(
        &client,
        &exec,
        "two",
        "printf '%s%s\\n' HCTL2 OUTB",
        "HCTL2OUTB",
    )
    .unwrap();
    assert!(first.contains("HCTL2OUTA"));
    assert!(second.contains("HCTL2OUTB"));
    assert!(client.ping().is_ok());
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&state);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn a_dispatch_returns_the_marker_as_a_proposal_and_then_exits() {
    let (cred, _state, exec) = scratch("dispatch");
    let runtime = HerdrRuntime::new(binary());
    let running = runtime
        .start(&spec("job"), &bundle(), &exec, &cred)
        .unwrap();
    let mut saw_observation = false;
    let mut proposal = None;
    let mut exited = false;
    while let Ok(event) = running.events.recv_timeout(Duration::from_secs(20)) {
        match event {
            agency::runtime::RuntimeEvent::Observation { payload, .. } => {
                assert!(payload["text"].as_str().unwrap_or("").contains("HCTL2OUT"));
                saw_observation = true;
            }
            agency::runtime::RuntimeEvent::Proposal { bytes, .. } => {
                proposal = Some(String::from_utf8(bytes).unwrap());
            }
            agency::runtime::RuntimeEvent::Exited { code, .. } => {
                assert_eq!(code, Some(0));
                exited = true;
                break;
            }
            _ => panic!("unexpected runtime event"),
        }
    }
    assert!(saw_observation);
    assert!(proposal.unwrap().starts_with("HCTL2OUT"));
    assert!(exited);
    assert_eq!(runtime.running_servers(), 1);
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn the_pane_cannot_read_the_credential_root() {
    let (cred, state, exec) = scratch("secret");
    let cred = cred.canonicalize().unwrap();
    let server = Server::start(&binary(), &state, &cred, &exec).unwrap();
    let client = Client::connect(&server.socket).unwrap();
    let path = cred.join("pair.key");
    let text = herdr::run_command(
        &client,
        &exec,
        "secret",
        &format!(
            "if cat '{}' >/dev/null 2>&1; then printf '%s%s\\n' HCTL2 OUTYES; else printf '%s%s\\n' HCTL2 OUTNO; fi",
            path.display()
        ),
        "HCTL2OUTNO",
    )
    .unwrap();
    assert!(text.contains("HCTL2OUTNO"));
    assert!(!text.contains("HCTL2OUTYES"));
    assert!(!text.contains("secret-credential"));
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&state);
    let _ = std::fs::remove_dir_all(&exec);
}

fn spec(key: &str) -> agency_proto::Sealed<agency_proto::ExecutionSpec> {
    use agency_proto::{
        Capabilities, ExecutionSpec, FrozenRef, InputPolicy, Owner, OwnerKind, Sealed, hash,
    };
    let reference = |id: &str| FrozenRef {
        id: id.into(),
        revision: "1".into(),
        digest: hash(id.as_bytes()),
    };
    let owner = Owner {
        project: "project".into(),
        kind: OwnerKind::RoomInvocation,
        id: "invocation".into(),
        generation: 1,
    };
    let profession = HerdrRuntime::new(binary())
        .catalog()
        .unwrap()
        .professions
        .remove(0);
    let bundle = bundle();
    Sealed::new(ExecutionSpec {
        owner,
        project: reference("project"),
        selection: reference("selection"),
        selection_policy_digest: hash(b"selection"),
        profession,
        profile: reference("profile"),
        manifest: reference("manifest"),
        bundle: FrozenRef {
            id: "bundle".into(),
            revision: "1".into(),
            digest: bundle.digest.clone(),
        },
        binding: reference("binding"),
        required_capabilities: Capabilities {
            input: true,
            stop: true,
            ..Capabilities::default()
        },
        input_policy: InputPolicy::NativeInteractiveAllowed,
        permission_digest: hash(b"permission"),
        permissions: vec!["read".into()],
        budget: 100,
        deadline_ms: 60_000,
        repo: None,
        base: None,
        delivery_scope: vec![],
        write_lease: None,
        review_publish_policy: None,
        idempotency_key: key.into(),
    })
    .unwrap()
}

fn bundle() -> agency_proto::Sealed<agency_proto::context::Bundle> {
    use agency_proto::{
        FrozenRef, Owner, OwnerKind, Sealed,
        context::{Bundle, Delivery, Entry},
        hash,
    };
    let reference = |id: &str| FrozenRef {
        id: id.into(),
        revision: "1".into(),
        digest: hash(id.as_bytes()),
    };
    let owner = Owner {
        project: "project".into(),
        kind: OwnerKind::RoomInvocation,
        id: "invocation".into(),
        generation: 1,
    };
    Sealed::new(Bundle {
        id: "bundle".into(),
        manifest: reference("manifest"),
        consumer: owner,
        entries: vec![Entry {
            source: reference("task"),
            description: "task book".into(),
            required: true,
            offline_required: true,
            bytes_digest: hash(b"do the work"),
            delivery: Delivery::Inline {
                bytes: b"do the work".to_vec(),
            },
        }],
        renderer: reference("renderer"),
        tokenizer: reference("tokenizer"),
        redaction: reference("redaction"),
        compression: vec![],
        candidate_tokens: Some(3),
        selected_tokens: Some(3),
        delivered_tokens: Some(3),
        permission_digest: hash(b"permission"),
        budget: 100,
        retention: "owner_terminal_and_admission_closed".into(),
    })
    .unwrap()
}

#[test]
fn repeated_pane_text_is_one_observation() {
    let mut seen = HashSet::new();
    assert!(herdr::observe_once(&mut seen, "HCTL2OUT"));
    assert!(!herdr::observe_once(&mut seen, "HCTL2OUT"));
    assert!(herdr::observe_once(&mut seen, "other"));
}

#[test]
fn harness_below_the_minimum_is_not_listed_and_a_missing_session_is_unverified() {
    assert!(harness::version_at_least(
        "0.160.0",
        harness::codex_minimum()
    ));
    assert!(harness::version_at_least(
        "2.1.289",
        harness::claude_minimum()
    ));
    assert!(!harness::version_at_least(
        "0.1.0",
        harness::codex_minimum()
    ));
    eprintln!(
        "UNVERIFIED codex session: no model credential exercise in this test; minimum is {}",
        harness::codex_minimum()
    );
    eprintln!(
        "UNVERIFIED claude session: no model credential exercise in this test; minimum is {}",
        harness::claude_minimum()
    );
}

#[test]
fn serve_refuses_a_credential_root_inside_an_allowed_directory() {
    let error = confine::refuse_covered_credential_root(std::path::Path::new("/opt")).unwrap_err();
    assert_eq!(error.code, "CREDENTIAL_ROOT_COVERED");
}

#[test]
fn a_missing_credential_root_is_not_used_raw() {
    let missing = std::env::temp_dir().join(format!("hctl2-missing-{}", std::process::id()));
    let error = confine::execution_dir(&missing, "dispatch").unwrap_err();
    assert_eq!(error.code, "CREDENTIAL_ROOT_UNRESOLVED");
}

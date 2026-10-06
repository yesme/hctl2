use agency::{
    confine, harness,
    herdr::{self, Client, Server},
    launch,
    runtime::Runtime,
};
use agency_proto::{Catalog, Pair, Pairing, client::Client as PortClient};
use serde_json::json;
use std::{
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

fn binary() -> PathBuf {
    let path = PathBuf::from(std::env::var("HCTL2_LOCKED_HERDR").expect("locked Herdr binary"));
    let path = path.canonicalize().expect("locked Herdr binary");
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}

fn installed_fixture(root: &std::path::Path) -> PathBuf {
    let dest = root.join("libexec/hctl2/herdr");
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
    std::fs::copy(binary(), &dest).unwrap();
    let digest = agency::catalog::file_digest(&dest).unwrap();
    let manifest = root.join("share/hctl2/PAYLOAD.sha256");
    std::fs::create_dir_all(manifest.parent().unwrap()).unwrap();
    std::fs::write(manifest, format!("{digest}  libexec/hctl2/herdr\n")).unwrap();
    dest
}

fn scratch(name: &str) -> (PathBuf, PathBuf) {
    let cred =
        std::env::temp_dir().join(format!("hctl2-herdr-cred-{}-{}", name, std::process::id()));
    let exec =
        std::env::temp_dir().join(format!("hctl2-herdr-exec-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    std::fs::create_dir_all(&cred).unwrap();
    std::fs::create_dir_all(&exec).unwrap();
    std::fs::write(cred.join("pair.key"), b"secret-credential").unwrap();
    (cred, exec)
}

fn outside_credential(state: &std::path::Path, cred: &std::path::Path, exec: &std::path::Path) {
    let cred = cred.canonicalize().unwrap();
    let exec = exec.canonicalize().unwrap();
    assert!(!state.starts_with(&exec));
    assert!(!exec.starts_with(state));
    assert!(!state.starts_with(&cred));
    assert!(!cred.starts_with(state));
}

#[test]
fn locked_herdr_runs_a_program_in_a_pane() {
    let (cred, exec) = scratch("pane");
    let state = herdr::state_dir(&exec, &cred).unwrap();
    outside_credential(&state, &cred, &exec);
    let server = Server::start(&binary(), &state, &cred, &exec).unwrap();
    let client = Client::connect(&server.socket).unwrap();
    let pong = client.ping().unwrap();
    assert_eq!(pong["protocol"], 22);
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
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn two_dispatches_start_together_on_one_herdr() {
    let (cred, exec) = scratch("two");
    let pipe = herdr::Pipe::open(&binary(), &cred, &exec).unwrap();
    outside_credential(pipe.state(), &cred, &exec);
    let (first, second) = std::thread::scope(|scope| {
        let left = scope.spawn(|| {
            pipe.run("one", "printf '%s%s\\n' HCTL2 OUTA", "HCTL2OUTA")
                .unwrap()
        });
        let right = scope.spawn(|| {
            pipe.run("two", "printf '%s%s\\n' HCTL2 OUTB", "HCTL2OUTB")
                .unwrap()
        });
        (left.join().unwrap(), right.join().unwrap())
    });
    assert!(first.contains("HCTL2OUTA"));
    assert!(!first.contains("HCTL2OUTB"));
    assert!(second.contains("HCTL2OUTB"));
    assert!(!second.contains("HCTL2OUTA"));
    assert_eq!(pipe.servers_started(), 1);
    let pid = pipe.pid().unwrap();
    assert!(process_matches(pid, &binary()));
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn a_split_marker_printed_last_is_program_output() {
    let (cred, exec) = scratch("split");
    let server = Server::start(
        &binary(),
        &herdr::state_dir(&exec, &cred).unwrap(),
        &cred,
        &exec,
    )
    .unwrap();
    let client = Client::connect(&server.socket).unwrap();
    let started = Instant::now();
    let text = herdr::run_command(
        &client,
        &exec,
        "split",
        "sleep 2; touch executed; printf '%s%s\\n' HCTL2 DONE",
        "HCTL2DONE",
    )
    .unwrap();
    assert!(started.elapsed() >= Duration::from_millis(1500));
    assert!(text.contains("HCTL2DONE"));
    assert!(exec.join("executed").exists());
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn a_command_containing_the_marker_is_rejected_before_it_runs() {
    let (cred, exec) = scratch("reject");
    let server = Server::start(
        &binary(),
        &herdr::state_dir(&exec, &cred).unwrap(),
        &cred,
        &exec,
    )
    .unwrap();
    let client = Client::connect(&server.socket).unwrap();
    let command = "sleep 2\n# HCTL2MULTILINE\nprintf '%s%s\\n' HCTL2 MULTILINE\ntouch executed";
    let started = Instant::now();
    let error =
        herdr::run_command(&client, &exec, "reject", command, "HCTL2MULTILINE").unwrap_err();
    assert_eq!(error.code, "INVALID_INPUT");
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(!exec.join("executed").exists());
    client.ping().unwrap();
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn a_backspace_in_the_command_is_rejected_before_it_runs() {
    reject_control_before_herdr('\u{8}');
}

#[test]
fn a_delete_in_the_command_is_rejected_before_it_runs() {
    reject_control_before_herdr('\u{7f}');
}

fn reject_control_before_herdr(control: char) {
    let (cred, exec) = scratch(&format!("control{:02x}", control as u32));
    let server = Server::start(
        &binary(),
        &herdr::state_dir(&exec, &cred).unwrap(),
        &cred,
        &exec,
    )
    .unwrap();
    let client = Client::connect(&server.socket).unwrap();
    let command = format!("sleep 3; touch codexdone; : HCTL2X{control}DONE");
    let started = Instant::now();
    let error = herdr::run_command(&client, &exec, "control", &command, "HCTL2DONE").unwrap_err();
    assert_eq!(error.code, "INVALID_INPUT");
    assert!(error.message.contains("control character"));
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(!exec.join("codexdone").exists());
    client.ping().unwrap();
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn a_missing_marker_closes_the_pane() {
    let (cred, exec) = scratch("nomark");
    let server = Server::start(
        &binary(),
        &herdr::state_dir(&exec, &cred).unwrap(),
        &cred,
        &exec,
    )
    .unwrap();
    let client = Client::connect(&server.socket).unwrap();
    let stamp = exec.join("heartbeat");
    let command = format!(
        "while true; do date +%s >> '{}'; sleep 0.2; done",
        stamp.display()
    );
    let error = herdr::run_command(&client, &exec, "nomark", &command, "HCTL2NEVER").unwrap_err();
    assert_eq!(error.code, "HERDR_OUTPUT_MISSING");
    assert!(!error.message.contains("pane close failed"));
    std::thread::sleep(Duration::from_millis(500));
    let first = std::fs::read_to_string(&stamp).unwrap_or_default();
    std::thread::sleep(Duration::from_millis(1000));
    let second = std::fs::read_to_string(&stamp).unwrap_or_default();
    assert!(!first.is_empty());
    assert_eq!(first, second);
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn a_failed_pid_write_leaves_no_herdr() {
    let (cred, exec) = scratch("pidfail");
    let state = herdr::state_dir(&exec, &cred).unwrap();
    std::fs::create_dir(state.join("herdr.pid")).unwrap();
    let error = match Server::start(&binary(), &state, &cred, &exec) {
        Err(error) => error,
        Ok(server) => {
            drop(server);
            panic!("pid write failure still started Herdr");
        }
    };
    assert_eq!(error.code, "IO_ERROR");
    let socket = herdr::socket_path(&state);
    std::thread::sleep(Duration::from_secs(2));
    assert!(
        Client::connect(&socket)
            .and_then(|client| client.ping())
            .is_err()
    );
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn the_pane_cannot_read_the_credential_root() {
    let (cred, exec) = scratch("secret");
    let cred = cred.canonicalize().unwrap();
    let server = Server::start(
        &binary(),
        &herdr::state_dir(&exec, &cred).unwrap(),
        &cred,
        &exec,
    )
    .unwrap();
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
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn the_next_start_reaps_an_orphaned_herdr() {
    let (cred, exec) = scratch("orphan");
    let state = herdr::state_dir(&exec, &cred).unwrap();
    let binary = binary();
    let server = Server::start(&binary, &state, &cred, &exec).unwrap();
    let old = server.pid();
    assert!(process_matches(old, &binary));
    std::mem::forget(server);
    let again = Server::start(&binary, &state, &cred, &exec).unwrap();
    assert_ne!(again.pid(), old);
    assert!(!process_matches(old, &binary));
    Client::connect(&again.socket).unwrap().ping().unwrap();
    drop(again);
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn serve_on_a_new_directory_does_not_list_herdr() {
    let root = std::env::temp_dir().join(format!(
        "hctl2-serve-new-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&root);
    assert!(!root.exists());
    let agency = PathBuf::from(std::env::var("HCTL2_CONFINE_BIN").expect("agency binary"));
    let log = std::env::temp_dir().join(format!("hctl2-serve-log-{}", std::process::id()));
    let mut child = Command::new(&agency)
        .arg("--root")
        .arg(&root)
        .arg("serve")
        .env("HCTL2_LOCKED_HERDR", binary())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(std::fs::File::create(&log).unwrap())
        .spawn()
        .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let result = runtime.block_on(async {
        for _ in 0..50 {
            if root.join("pair.key").exists()
                && Command::new(&agency)
                    .arg("--root")
                    .arg(&root)
                    .arg("status")
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            {
                let pairing = PortClient::new(
                    agency_proto::client::admin_endpoint(&root).unwrap(),
                    std::fs::read_to_string(root.join("pair.key")).unwrap(),
                );
                let pair: Pairing = pairing
                    .call(
                        "pair",
                        &Pair {
                            control_id: "control".into(),
                            tenant_key: agency_proto::client::new_credential().unwrap(),
                        },
                    )
                    .await
                    .unwrap();
                let tenant = PortClient::new(pair.endpoint.into(), pair.key);
                let catalog: Catalog = tenant.call("catalog", &json!({})).await.unwrap();
                return Ok(catalog);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(std::fs::read_to_string(&log).unwrap_or_default())
    });
    let _ = Command::new(&agency)
        .arg("--root")
        .arg(&root)
        .arg("stop")
        .output();
    let _ = child.kill();
    let _ = child.wait();
    let catalog = result.unwrap_or_else(|detail| panic!("serve did not become ready: {detail}"));
    assert!(catalog.professions.is_empty());
    assert!(catalog.harnesses.is_empty());
    assert!(
        !format!("{catalog:?}").contains("herdr-shell"),
        "{catalog:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_file(&log);
}

#[test]
fn layout_refusal_is_linux_only_and_a_new_directory_is_not_unresolved() {
    let covered = confine::refuse_covered_credential_root(std::path::Path::new("/opt"));
    let fresh = std::env::temp_dir().join(format!("hctl2-fresh-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&fresh);
    assert!(!fresh.exists());
    let created = confine::refuse_covered_credential_root(&fresh);
    if cfg!(target_os = "linux") {
        assert_eq!(covered.unwrap_err().code, "CREDENTIAL_ROOT_COVERED");
        assert!(created.is_ok());
    } else {
        assert!(covered.is_ok());
        assert!(created.is_ok());
    }
}

#[test]
fn a_missing_credential_root_is_not_used_raw() {
    let missing = std::env::temp_dir().join(format!("hctl2-missing-{}", std::process::id()));
    let error = confine::execution_dir(&missing, "dispatch").unwrap_err();
    assert_eq!(error.code, "CREDENTIAL_ROOT_UNRESOLVED");
}

fn server(
    name: &str,
) -> (
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
    std::sync::Arc<Server>,
) {
    let (cred, exec) = scratch(name);
    let state = herdr::state_dir(&exec, &cred).unwrap();
    let server = std::sync::Arc::new(Server::start(&binary(), &state, &cred, &exec).unwrap());
    (cred, exec, state, server)
}

#[test]
fn native_exit_is_an_observation_for_a_script_fixture() {
    let (cred, exec, state, server) = server("exit0");
    let launch = launch::Launch::start(
        server,
        &exec,
        &state,
        &cred,
        "printf '%s\\n' RESULT_OK\nexit 0\n",
        "ok",
    )
    .unwrap();
    let finished = launch.wait(Duration::from_secs(15)).unwrap();
    assert_eq!(finished.code, Some(0));
    assert!(finished.stdout.windows(9).any(|item| item == b"RESULT_OK"));
    let _ = launch.cancel();
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    let _ = std::fs::remove_dir_all(&state);
}

#[test]
fn a_nonzero_script_exit_is_not_success() {
    let (cred, exec, state, server) = server("exit7");
    let launch = launch::Launch::start(
        server,
        &exec,
        &state,
        &cred,
        "printf '%s\\n' NOPE\nexit 7\n",
        "bad",
    )
    .unwrap();
    let finished = launch.wait(Duration::from_secs(15)).unwrap();
    assert_eq!(finished.code, Some(7));
    let _ = launch.cancel();
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    let _ = std::fs::remove_dir_all(&state);
}

#[test]
fn cancel_stops_the_native_process_before_it_finishes() {
    let (cred, exec, state, server) = server("cancel");
    let launch = launch::Launch::start(
        server,
        &exec,
        &state,
        &cred,
        "sleep 30\ntouch finished\nexit 0\n",
        "cancel",
    )
    .unwrap();
    std::thread::sleep(Duration::from_millis(300));
    launch.cancel().unwrap();
    assert!(launch.poll_exit().unwrap().is_none());
    assert!(!exec.join("finished").exists());
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    let _ = std::fs::remove_dir_all(&state);
}

#[test]
fn a_screen_marker_does_not_finish_a_running_script() {
    let (cred, exec, state, server) = server("history");
    let launch = launch::Launch::start(
        std::sync::Arc::clone(&server),
        &exec,
        &state,
        &cred,
        "printf '%s\\n' HCTL2DONE\nsleep 3\ntouch finished\nexit 0\n",
        "history",
    )
    .unwrap();
    let client = Client::connect(&server.socket).unwrap();
    let noise = ": HCTL2XDONE\n^X^^; sleep 3; touch r4done\n: HCTL2XDONE\n^X^^; sleep 3; touch codexdone; printf '%s%s\\n' HCTL2 DONE\n";
    client
        .call(
            "pane.send_text",
            serde_json::json!({"pane_id": launch.pane(), "text": noise}),
        )
        .unwrap();
    std::thread::sleep(Duration::from_millis(800));
    assert!(launch.poll_exit().unwrap().is_none());
    assert!(!exec.join("finished").exists());
    let finished = launch.wait(Duration::from_secs(15)).unwrap();
    assert_eq!(finished.code, Some(0));
    assert!(exec.join("finished").exists());
    let _ = launch.cancel();
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    let _ = std::fs::remove_dir_all(&state);
}

#[test]
fn official_herdr_state_is_outside_the_execution_directory() {
    let (cred, exec, state, server) = server("statedeny");
    assert!(!state.starts_with(&exec));
    let body = format!(
        "touch '{}' 2>/dev/null || true\ntouch wrote-ok\nexit 0\n",
        state.join("pwned").display()
    );
    let launch = launch::Launch::start(server, &exec, &state, &cred, &body, "statedeny").unwrap();
    let finished = launch.wait(Duration::from_secs(15)).unwrap();
    assert_eq!(finished.code, Some(0));
    assert!(exec.join("wrote-ok").exists());
    // Official Herdr inherits state write access. Per-pane denial is deferred policy.
    assert!(state.join("pwned").exists());
    let _ = launch.cancel();
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    let _ = std::fs::remove_dir_all(&state);
}

#[test]
fn a_tampered_install_is_not_the_packaged_herdr() {
    let install = tempfile::tempdir().unwrap();
    let dest = installed_fixture(install.path());
    let digest = agency::catalog::file_digest(&dest).unwrap();
    let manifest = install.path().join("share/hctl2/PAYLOAD.sha256");
    assert!(launch::installed_herdr(install.path()).is_ok());
    std::fs::remove_file(&manifest).unwrap();
    // Even an exact upstream download is not an installed payload without its manifest.
    assert_eq!(
        launch::installed_herdr(install.path()).unwrap_err().code,
        "HERDR_MANIFEST_INVALID"
    );
    std::fs::write(&manifest, format!("{digest}  libexec/hctl2/herdr\n")).unwrap();
    let mut bytes = std::fs::read(&dest).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&dest, &bytes).unwrap();
    assert_eq!(
        launch::installed_herdr(install.path()).unwrap_err().code,
        "HERDR_DIGEST_MISMATCH"
    );
}

#[test]
fn caller_timeout_is_not_a_successful_exit() {
    let (cred, exec, state, server) = server("timeout");
    let launch = launch::Launch::start(
        server,
        &exec,
        &state,
        &cred,
        "sleep 30\nexit 0\n",
        "timeout",
    )
    .unwrap();
    let error = match launch.wait(Duration::from_millis(400)) {
        Err(error) => error,
        Ok(_) => panic!("a short timeout was treated as a finished script"),
    };
    assert_eq!(error.code, "LAUNCH_TIMEOUT");
    assert!(launch.poll_exit().unwrap().is_none());
    launch.cancel().unwrap();
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    let _ = std::fs::remove_dir_all(&state);
}

#[test]
fn a_locked_install_smokes_before_it_is_cataloged() {
    let install = std::env::temp_dir().join(format!("hctl2-install-ok-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&install);
    let dest = installed_fixture(&install);
    let mut permissions = std::fs::metadata(&dest).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&dest, permissions).unwrap();
    let claude = stub_claude(&install, "2.1.289");
    let runtime = launch::InstalledHerdr::open(dest, &claude).unwrap();
    let catalog = runtime.catalog().unwrap();
    assert_eq!(catalog.professions[0].reference.id, "claude-code");
    assert!(
        !catalog
            .professions
            .iter()
            .any(|item| item.reference.id == "herdr-locked")
    );
    assert_eq!(catalog.harnesses[0].revision, "protocol-22");
    assert_eq!(catalog.harnesses[0].digest.len(), 64);
    let _ = std::fs::remove_dir_all(&install);
}

#[test]
fn a_confined_harness_smoke_failure_prevents_cataloging() {
    let (cred, exec) = scratch("smokefail");
    let claude = exec.join("claude-fixture");
    // An unrestricted version check succeeds; the actual confined launch must fail.
    std::fs::write(
        &claude,
        "#!/bin/sh\ncase \"$PWD\" in *hctl2-herdr-smoke*) echo confined-install-unavailable >&2; exit 9;; esac\necho '2.1.289 (Claude Code)'\n",
    ).unwrap();
    std::fs::set_permissions(&claude, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        Command::new(&claude)
            .arg("--version")
            .status()
            .unwrap()
            .success()
    );
    let error = match launch::InstalledHerdr::open(binary(), &claude) {
        Ok(_) => panic!("confined failure was cataloged"),
        Err(error) => error,
    };
    assert_eq!(error.code, "HARNESS_SMOKE_FAILED");
    assert!(error.message.contains("confined-install-unavailable"));
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn the_native_program_cannot_read_the_credential_root() {
    let (cred, exec, state, server) = server("native-secret");
    let body = format!(
        "if cat '{}' >/dev/null 2>&1; then echo credential-leaked; exit 1; else echo credential-denied; fi\n",
        cred.join("pair.key").display()
    );
    let launch = launch::Launch::start(server, &exec, &state, &cred, &body, "secret").unwrap();
    let finished = launch.wait(Duration::from_secs(15)).unwrap();
    assert_eq!(finished.code, Some(0));
    assert_eq!(
        String::from_utf8(finished.stdout).unwrap(),
        "credential-denied\n"
    );
}

#[test]
fn a_missing_credential_root_is_rejected_before_pane_creation() {
    let (cred, exec, state, server) = server("launch-fail");
    // Validation must fail before starting any pane.
    std::fs::remove_dir_all(&cred).unwrap();
    assert!(
        launch::Launch::start(
            Arc::clone(&server),
            &exec,
            &state,
            &cred,
            "exit 0\n",
            "fail"
        )
        .is_err()
    );
    let workspaces = Client::connect(&server.socket)
        .unwrap()
        .call("workspace.list", json!({}))
        .unwrap();
    assert!(
        workspaces["workspaces"].as_array().unwrap().is_empty(),
        "{workspaces}"
    );
}

#[test]
fn a_completed_dispatch_closes_its_display_pane() {
    let (cred, exec) = scratch("pane-cleanup");
    let runtime = test_runtime(&binary(), "echo complete\nexit 0\n");
    let mut running = runtime
        .start(
            &sealed_spec("complete", now_ms() + 10_000),
            &sealed_bundle("task"),
            &exec,
            &cred,
        )
        .unwrap();
    let events = collect(&mut running, Duration::from_secs(10));
    assert!(stdout_has(&events, b"complete"));
    let state = herdr::state_dir(&exec, &cred).unwrap();
    let client = Client::connect(&herdr::socket_path(&state)).unwrap();
    let workspaces = client.call("workspace.list", json!({})).unwrap();
    assert!(
        workspaces["workspaces"].as_array().unwrap().is_empty(),
        "{workspaces}"
    );
}

#[test]
fn agency_start_keeps_a_failed_catalog_probe_reason() {
    let (root, install) = scratch("startup-log");
    let agency = PathBuf::from(std::env::var("HCTL2_CONFINE_BIN").unwrap());
    let started = Command::new(&agency)
        .args(["--root"])
        .arg(&root)
        .arg("start")
        .env("HCTL2_INSTALL_ROOT", &install)
        .output()
        .unwrap();
    let status = Command::new(&agency)
        .args(["--root"])
        .arg(&root)
        .arg("status")
        .output()
        .unwrap();
    let catalog = paired_catalog(&root);
    let stopped = Command::new(&agency)
        .args(["--root"])
        .arg(&root)
        .arg("stop")
        .output()
        .unwrap();
    assert!(
        started.status.success(),
        "{}",
        String::from_utf8_lossy(&started.stdout)
    );
    assert!(status.status.success());
    assert!(stopped.status.success());
    let status: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status["ready"], true);
    let catalog = catalog.unwrap();
    assert!(catalog.professions.is_empty());
    let reason = std::fs::read_to_string(root.join("serve.err")).unwrap();
    assert!(reason.contains("HERDR_BINARY_MISSING"), "{reason}");
    assert_eq!(
        std::fs::metadata(root.join("serve.err"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}

fn paired_catalog(root: &std::path::Path) -> agency_proto::Result<Catalog> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let client = PortClient::new(
                agency_proto::client::admin_endpoint(root)?,
                std::fs::read_to_string(root.join("pair.key"))?,
            );
            let pair: Pairing = client
                .call(
                    "pair",
                    &Pair {
                        control_id: "fixture-control".into(),
                        tenant_key: agency_proto::client::new_credential()?,
                    },
                )
                .await?;
            PortClient::new(pair.endpoint.into(), pair.key)
                .call("catalog", &json!({}))
                .await
        })
}

#[test]
fn agency_start_and_pair_catalog_the_confined_claude_profession() {
    let (root, install) = scratch("catalog-start");
    installed_fixture(&install);
    let claude = stub_claude(&install, "2.1.289");
    let agency = PathBuf::from(std::env::var("HCTL2_CONFINE_BIN").unwrap());
    let started = Command::new(&agency)
        .arg("--root")
        .arg(&root)
        .arg("start")
        .env("HCTL2_INSTALL_ROOT", &install)
        .env("HCTL2_CLAUDE", &claude)
        .output()
        .unwrap();
    let catalog = paired_catalog(&root);
    let stopped = Command::new(&agency)
        .arg("--root")
        .arg(&root)
        .arg("stop")
        .output()
        .unwrap();
    assert!(
        started.status.success(),
        "{}",
        String::from_utf8_lossy(&started.stdout)
    );
    assert!(stopped.status.success());
    let catalog = catalog.unwrap();
    let claude_profession = catalog
        .professions
        .iter()
        .find(|item| item.reference.id == "claude-code")
        .expect("claude-code");
    assert_eq!(
        claude_profession.reference.revision,
        "2.1.289 (Claude Code)"
    );
    assert_eq!(
        claude_profession.reference.digest,
        agency::catalog::file_digest(&claude).unwrap()
    );
}

#[cfg(target_os = "linux")]
#[test]
fn the_pane_cannot_write_the_herdr_binary_directory() {
    let (cred, exec, state, server) = server("bindir");
    let marker = binary()
        .parent()
        .unwrap()
        .join(format!("hctl2-pwned-{}", std::process::id()));
    let _ = std::fs::remove_file(&marker);
    let body = format!("touch '{}' || true\nexit 0\n", marker.display());
    let launch = launch::Launch::start(server, &exec, &state, &cred, &body, "bindir").unwrap();
    let finished = launch.wait(Duration::from_secs(15)).unwrap();
    assert_eq!(finished.code, Some(0));
    assert!(!marker.exists());
    let _ = launch.cancel();
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    let _ = std::fs::remove_dir_all(&state);
}

#[test]
fn harness_minimums_do_not_start_a_session() {
    assert!(harness::version_at_least(
        "2.1.289",
        harness::CLAUDE_MINIMUM
    ));
    assert!(!harness::version_at_least("2.1.0", harness::CLAUDE_MINIMUM));
    assert!(!harness::version_at_least(
        "2.1.286",
        harness::CLAUDE_MINIMUM
    ));
    assert!(harness::version_at_least(
        "2.1.287",
        harness::CLAUDE_MINIMUM
    ));
    assert!(harness::version_at_least("0.160.0", harness::CODEX_MINIMUM));
    assert!(!harness::version_at_least(
        "0.153.3",
        harness::CODEX_MINIMUM
    ));
}

#[test]
#[ignore = "UNVERIFIED: requires a logged-in Claude session and HCTL2_HARNESS_LIVE=1"]
fn live_claude_dispatch_returns_one_turn() {
    assert!(
        std::env::var_os("HCTL2_HARNESS_LIVE").is_some(),
        "explicit live validation also requires HCTL2_HARNESS_LIVE=1"
    );
    let claude = std::process::Command::new("/usr/bin/which")
        .arg("claude")
        .output()
        .unwrap();
    let claude = String::from_utf8(claude.stdout).unwrap();
    let claude = std::path::PathBuf::from(claude.trim());
    let (cred, _root, exec) = standby_root("live");
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let bundle = sealed_bundle("Reply with exactly HCTL2_REAL_OK and do not use tools.\n");
    let mut document = sealed_spec("live", now_ms() + 120_000).document;
    document.profession = runtime.catalog().unwrap().professions.remove(0);
    document.bundle.digest = bundle.digest.clone();
    let spec = agency_proto::Sealed::new(document).unwrap();
    spec.document.validate().unwrap();
    bundle.document.validate_delivery().unwrap();
    let running = runtime.start(&spec, &bundle, &exec, &cred).unwrap();
    let mut events = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(120);
    while Instant::now() < deadline {
        match running.events.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                let end = matches!(
                    event,
                    agency::runtime::RuntimeEvent::TurnReturned
                        | agency::runtime::RuntimeEvent::Exited { .. }
                );
                events.push(event);
                if end {
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            _ => {}
        }
    }
    let proposal = events.iter().find_map(|event| match event {
        agency::runtime::RuntimeEvent::Proposal { bytes, .. } => {
            Some(String::from_utf8_lossy(bytes).into_owned())
        }
        _ => None,
    });
    let returned = events
        .iter()
        .any(|event| matches!(event, agency::runtime::RuntimeEvent::TurnReturned));
    let proposal = proposal.unwrap_or_else(|| {
        let reason = events
            .iter()
            .filter_map(|event| match event {
                agency::runtime::RuntimeEvent::Observation { kind, payload, .. } => {
                    Some(format!("{kind}: {payload}"))
                }
                agency::runtime::RuntimeEvent::Exited { code, .. } => {
                    Some(format!("Exited(code={code:?})"))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        panic!("no proposal: {reason}");
    });
    assert!(proposal.contains("HCTL2_REAL_OK"), "{proposal}");
    assert!(returned);
    eprintln!(
        "LIVE claude {} via locked Herdr protocol 22",
        spec.document.profession.reference.revision
    );
    eprintln!("Proposal(schema=claude.turn.v1, source=adapter_event): {proposal}");
    eprintln!("TurnReturned (not Task completion; does not require process exit)");
    let herdr_pid = runtime.pid().unwrap();
    running.session.lock().unwrap().stop().unwrap();
    drop(running);
    runtime.shutdown().unwrap();
    drop(runtime);
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && process_matches(herdr_pid, &binary()) {
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(!process_matches(herdr_pid, &binary()));
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn bundle_text_is_not_executed_as_a_shell_script() {
    let (cred, exec) = scratch("bundle");
    let runtime = test_runtime(&binary(), "printf '%s\\n' RESULT_OK\nexit 0\n");
    let spec = sealed_spec("bundle", now_ms() + 20_000);
    let bundle = sealed_bundle("not-a-shell-command\n");
    let mut running = runtime.start(&spec, &bundle, &exec, &cred).unwrap();
    let events = collect(&mut running, Duration::from_secs(15));
    assert!(events.iter().any(|event| matches!(
        event,
        agency::runtime::RuntimeEvent::Proposal { bytes, .. } if bytes.windows(9).any(|item| item == b"RESULT_OK")
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        agency::runtime::RuntimeEvent::Exited {
            code: Some(0),
            requested_stop: false
        }
    )));
    drop(running);
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn a_failed_dispatch_exits_on_the_event_channel() {
    let (cred, exec) = scratch("badstart");
    let runtime = test_runtime(&binary(), "exit 7\n");
    let mut running = runtime
        .start(
            &sealed_spec("bad", now_ms() + 20_000),
            &sealed_bundle("not-a-shell-command\n"),
            &exec,
            &cred,
        )
        .unwrap();
    let events = collect(&mut running, Duration::from_secs(15));
    assert!(events.iter().any(|event| matches!(
        event,
        agency::runtime::RuntimeEvent::Exited { code: Some(7), .. }
    )));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, agency::runtime::RuntimeEvent::Proposal { .. }))
    );
    drop(running);
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn stopping_a_dispatch_emits_exited_and_drops_herdr() {
    let (cred, exec) = scratch("stopstart");
    let runtime = test_runtime(&binary(), "sleep 30\n");
    let mut running = runtime
        .start(
            &sealed_spec("stop", now_ms() + 60_000),
            &sealed_bundle("not-a-shell-command\n"),
            &exec,
            &cred,
        )
        .unwrap();
    let pid = runtime.pid().unwrap();
    std::thread::sleep(Duration::from_millis(300));
    running.session.lock().expect("session").stop().unwrap();
    let events = collect(&mut running, Duration::from_secs(5));
    assert!(events.iter().any(|event| matches!(
        event,
        agency::runtime::RuntimeEvent::Exited {
            requested_stop: true,
            ..
        }
    )));
    drop(running);
    drop(runtime);
    let bin = binary();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && process_matches(pid, &bin) {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!process_matches(pid, &bin));
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn a_deadline_is_a_timestamp_and_ends_the_dispatch() {
    let (cred, exec) = scratch("deadline");
    let runtime = test_runtime(&binary(), "sleep 30\n");
    let mut running = runtime
        .start(
            &sealed_spec("deadline", now_ms() + 1_200),
            &sealed_bundle("not-a-shell-command\n"),
            &exec,
            &cred,
        )
        .unwrap();
    let started = Instant::now();
    let events = collect(&mut running, Duration::from_secs(6));
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, agency::runtime::RuntimeEvent::DeadlineReached))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, agency::runtime::RuntimeEvent::Exited { .. }))
    );
    drop(running);
    drop(runtime);
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn two_dispatches_share_one_herdr_server() {
    let (cred, exec) = scratch("share");
    let runtime = std::sync::Arc::new(launch::InstalledHerdr::for_test(
        binary(),
        |spec, _bundle, _exec| {
            Ok(format!(
                "sleep 1\nprintf '%s\\n' {}\nexit 0\n",
                spec.idempotency_key
            ))
        },
    ));
    let left = std::sync::Arc::clone(&runtime);
    let right = std::sync::Arc::clone(&runtime);
    let cred_left = cred.clone();
    let cred_right = cred.clone();
    let exec_left = confine::execution_dir(&cred, "first").unwrap();
    let exec_right = confine::execution_dir(&cred, "second").unwrap();
    assert_ne!(exec_left, exec_right);
    let (first, second) = std::thread::scope(|scope| {
        let one = scope.spawn(move || {
            let mut running = left
                .start(
                    &sealed_spec("ONE", now_ms() + 20_000),
                    &sealed_bundle("not-a-shell-command\n"),
                    &exec_left,
                    &cred_left,
                )
                .unwrap();
            collect(&mut running, Duration::from_secs(15))
        });
        let two = scope.spawn(move || {
            let mut running = right
                .start(
                    &sealed_spec("TWO", now_ms() + 20_000),
                    &sealed_bundle("not-a-shell-command\n"),
                    &exec_right,
                    &cred_right,
                )
                .unwrap();
            collect(&mut running, Duration::from_secs(15))
        });
        (one.join().unwrap(), two.join().unwrap())
    });
    assert_eq!(runtime.servers_started(), 1);
    assert!(stdout_has(&first, b"ONE"));
    assert!(stdout_has(&second, b"TWO"));
    drop(runtime);
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn an_execution_directory_file_does_not_replace_native_exit_observation() {
    let (cred, exec, state, server) = server("forge-exit");
    // This only checks the observation path, not protection against deliberate state tampering.
    let body = "printf '0\\n' > exit\nsleep 2\ntouch actual-finished\nprintf 'REAL_RESULT\\n'\n";
    let launch = launch::Launch::start(server, &exec, &state, &cred, body, "forge").unwrap();
    std::thread::sleep(Duration::from_millis(500));
    assert!(launch.poll_exit().unwrap().is_none());
    assert!(!exec.join("actual-finished").exists());
    let finished = launch.wait(Duration::from_secs(15)).unwrap();
    assert_eq!(finished.code, Some(0));
    assert!(exec.join("actual-finished").exists());
    assert!(finished.stdout.windows(11).any(|s| s == b"REAL_RESULT"));
}

#[test]
fn a_failed_close_does_not_claim_cancellation_and_can_be_retried() {
    let (cred, exec, state, server) = server("close-retry");
    let launch = launch::Launch::start(
        std::sync::Arc::clone(&server),
        &exec,
        &state,
        &cred,
        "sleep 30\ntouch must-not-finish\n",
        "retry",
    )
    .unwrap();
    let offline = server.socket.with_extension("offline");
    std::fs::rename(&server.socket, &offline).unwrap();
    let failed = launch.cancel();
    std::fs::rename(&offline, &server.socket).unwrap();
    assert!(failed.is_err());
    assert!(matches!(
        launch.wait_end(Duration::from_millis(100)).unwrap(),
        launch::WaitEnd::TimedOut
    ));
    launch.cancel().unwrap();
    assert!(matches!(
        launch.wait_end(Duration::from_secs(1)).unwrap(),
        launch::WaitEnd::Stopped
    ));
    assert!(!exec.join("must-not-finish").exists());
}

fn test_runtime(binary: &std::path::Path, script: &str) -> launch::InstalledHerdr {
    let script = script.to_owned();
    launch::InstalledHerdr::for_test(binary.to_path_buf(), move |_spec, _bundle, _exec| {
        Ok(script.clone())
    })
}

fn stub_claude(dir: &std::path::Path, version: &str) -> std::path::PathBuf {
    let path = dir.join("claude-stub");
    std::fs::write(
        &path,
        format!("#!/bin/sh\necho '{version} (Claude Code)'\n"),
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn sealed_spec(key: &str, deadline_ms: u64) -> agency_proto::Sealed<agency_proto::ExecutionSpec> {
    use agency_proto::{
        Capabilities, ExecutionSpec, FrozenRef, InputPolicy, Owner, OwnerKind, Profession, Sealed,
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
            reference: frozen("claude-code"),
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
        deadline_ms,
        repo: None,
        base: None,
        delivery_scope: vec![],
        write_lease: None,
        review_publish_policy: None,
        idempotency_key: key.into(),
    })
    .unwrap()
}

fn sealed_bundle(task: &str) -> agency_proto::Sealed<agency_proto::context::Bundle> {
    use agency_proto::context::{Bundle, Delivery, Entry};
    use agency_proto::{FrozenRef, Owner, OwnerKind, Sealed};
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

fn collect(
    running: &mut agency::runtime::Running,
    timeout: Duration,
) -> Vec<agency::runtime::RuntimeEvent> {
    let mut events = Vec::new();
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        match running.events.recv_timeout(Duration::from_millis(200)) {
            Ok(event) => {
                match &event {
                    agency::runtime::RuntimeEvent::ProtocolError(message) => {
                        eprintln!("runtime protocol error: {message}");
                    }
                    agency::runtime::RuntimeEvent::Observation { kind, payload, .. }
                        if matches!(kind.as_str(), "harness_failure" | "stopped") =>
                    {
                        eprintln!("runtime {kind}: {payload}");
                    }
                    _ => {}
                }
                let done = matches!(
                    event,
                    agency::runtime::RuntimeEvent::Exited { .. }
                        | agency::runtime::RuntimeEvent::DispatchReleased
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

fn stdout_has(events: &[agency::runtime::RuntimeEvent], marker: &[u8]) -> bool {
    events.iter().any(|event| match event {
        agency::runtime::RuntimeEvent::Proposal { bytes, .. } => {
            bytes.windows(marker.len()).any(|item| item == marker)
        }
        _ => false,
    })
}

fn waiting_claude(dir: &std::path::Path, _: bool) -> PathBuf {
    let claude = dir.join("waiting-claude");
    std::fs::copy(std::env::var("HCTL2_STANDBY_FIXTURE").unwrap(), &claude).unwrap();
    std::fs::set_permissions(&claude, std::fs::Permissions::from_mode(0o755)).unwrap();
    claude
}

fn turn_state(
    exec: &std::path::Path,
    cred: &std::path::Path,
    spec: &agency_proto::Sealed<agency_proto::ExecutionSpec>,
) -> PathBuf {
    let key = agency_proto::hash(
        &agency_proto::canonical(&json!([
            cred,
            spec.document.binding.id,
            spec.document.project.id,
            spec.document.selection.id
        ]))
        .unwrap(),
    );
    herdr::state_dir(exec, cred)
        .unwrap()
        .join(format!("standby-{key}"))
}
fn standby_root(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let (cred, root) = scratch(name);
    let exec = root.join("dispatches/job");
    std::fs::create_dir_all(&exec).unwrap();
    (cred, root, exec)
}
fn wait_for(path: &std::path::Path) {
    // A cold native session has a 30-second readiness budget. Parallel CI
    // must not fail its startup after only ten seconds.
    let timeout = Instant::now() + Duration::from_secs(35);
    while !path.exists() {
        assert!(Instant::now() < timeout, "missing {}", path.display());
        std::thread::sleep(Duration::from_millis(25));
    }
}
fn wait_started_job(state: &std::path::Path, key: &str) {
    let deadline = Instant::now() + Duration::from_secs(35);
    loop {
        let event = std::fs::read(state.join("started.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
        if event.is_some_and(|v| v["job"] == key) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "native turn never started for {key}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}
fn native_pid(dir: &std::path::Path) -> u32 {
    wait_for(&dir.join("harness.pid"));
    std::fs::read_to_string(dir.join("harness.pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}
fn standby_alive(pid: u32) -> bool {
    let out = Command::new("/bin/ps")
        .args(["-p", &pid.to_string(), "-o", "stat="])
        .output()
        .unwrap();
    let status = String::from_utf8_lossy(&out.stdout);
    out.status.success() && !status.trim().is_empty() && !status.trim().starts_with('Z')
}

fn real_claude_pid(state: &std::path::Path) -> u32 {
    let out = Command::new("/bin/ps")
        .args(["-axo", "pid=,command="])
        .output()
        .unwrap();
    let plugin = format!("--plugin-dir {}", state.join("plugin").display());
    let pids: Vec<u32> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|line| line.contains(&plugin))
        .map(|line| line.split_whitespace().next().unwrap().parse().unwrap())
        .collect();
    assert_eq!(
        pids.len(),
        1,
        "one native Claude process for this selection"
    );
    pids[0]
}

#[test]
#[ignore = "UNVERIFIED: requires a logged-in Claude session and HCTL2_HARNESS_LIVE=1"]
fn live_standby_remembers_across_turns_and_native_resume() {
    use agency::runtime::RuntimeEvent;
    assert!(std::env::var_os("HCTL2_HARNESS_LIVE").is_some());
    let claude = Command::new("/usr/bin/which")
        .arg("claude")
        .output()
        .unwrap();
    let claude = PathBuf::from(String::from_utf8(claude.stdout).unwrap().trim());
    let (cred, _root, exec) = standby_root("live-memory");
    let runtime =
        launch::InstalledHerdr::open_with_idle(binary(), &claude, Duration::from_secs(10)).unwrap();
    let profession = runtime.catalog().unwrap().professions.remove(0);
    let prepare = |key: &str, text: &str| {
        let bundle = sealed_bundle(text);
        let mut document = sealed_spec(key, now_ms() + 120_000).document;
        document.profession = profession.clone();
        document.bundle.digest = bundle.digest.clone();
        (agency_proto::Sealed::new(document).unwrap(), bundle)
    };
    let (first, bundle) = prepare(
        "remember",
        "Remember the word HCTL3D_AMBER_914 for our later conversation. Reply exactly HCTL3D_AMBER_914, without tools.",
    );
    let state = turn_state(&exec, &cred, &first);
    let run = |spec, bundle| {
        let mut running = runtime.start(spec, bundle, &exec, &cred).unwrap();
        let events = collect(&mut running, Duration::from_secs(120));
        assert!(
            stdout_has(&events, b"HCTL3D_AMBER_914"),
            "native answer did not recall the word"
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, RuntimeEvent::Proposal { .. }))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, RuntimeEvent::TurnReturned))
                .count(),
            1
        );
        events
    };
    run(&first, &bundle);
    let pid = real_claude_pid(&state);
    let session: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state.join("resume.json")).unwrap()).unwrap();
    let question =
        "What word did I ask you to remember earlier? Reply only with that word, without tools.";
    let (second, bundle) = prepare("recall", question);
    run(&second, &bundle);
    assert_eq!(
        real_claude_pid(&state),
        pid,
        "next dispatch must not restart Claude"
    );
    let client = Client::connect(&herdr::socket_path(
        &herdr::state_dir(&exec, &cred).unwrap(),
    ))
    .unwrap();
    let agents = client.call("agent.list", json!({})).unwrap();
    let native = agents["agents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["agent_session"]["value"] == session["session"])
        .unwrap();
    assert_eq!(native["agent"], "claude");
    assert_ne!(native["agent_status"], "unknown");
    let screen = client.call("pane.read", json!({"pane_id":native["pane_id"],"source":"visible","format":"text","strip_ansi":true})).unwrap();
    assert!(!screen["read"]["text"].as_str().unwrap().trim().is_empty());
    eprintln!(
        "LIVE Herdr: agent=claude, status={}, interactive pane has visible output",
        native["agent_status"]
    );
    eprintln!(
        "LIVE native Claude {}: two turns, same PID {pid}, recalled HCTL3D_AMBER_914",
        session["session"]
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    while standby_alive(pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(
        !standby_alive(pid),
        "idle timeout must reclaim the native pane"
    );
    let (third, bundle) = prepare("recall-after-idle", question);
    let events = run(&third, &bundle);
    assert!(events.iter().any(|e| matches!(e, RuntimeEvent::Observation{kind,payload,..} if kind=="session_opened" && payload["resumed"]==true && payload["resume_failed"]==false)));
    let resumed_pid = real_claude_pid(&state);
    assert_ne!(resumed_pid, pid);
    let resumed: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state.join("resume.json")).unwrap()).unwrap();
    assert_eq!(resumed, session, "Claude owns the same native conversation");
    eprintln!(
        "LIVE --resume: new PID {resumed_pid}, same native session, recalled HCTL3D_AMBER_914"
    );
    runtime.shutdown().unwrap();
    assert!(!standby_alive(resumed_pid));
}

#[test]
fn standby_reuses_one_live_herdr_process_and_routes_each_queued_answer() {
    let (cred, root, exec) = standby_root("standby-fifo");
    let claude = waiting_claude(&root, true);
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let first = sealed_spec("first", now_ms() + 60_000);
    let mut second = sealed_spec("second", now_ms() + 60_000).document;
    // Selection identity stays the same when a newer dispatch freezes a
    // different revision. The worker must not key reuse on old permissions.
    second.selection.revision = "2".into();
    second.selection.digest = agency_proto::hash(b"new selection snapshot");
    second.project.revision = "2".into();
    second.project.digest = agency_proto::hash(b"new project snapshot");
    second.binding.revision = "2".into();
    second.binding.digest = agency_proto::hash(b"new binding snapshot");
    let second = agency_proto::Sealed::new(second).unwrap();
    let state = turn_state(&exec, &cred, &first);
    let mut a = runtime
        .start(&first, &sealed_bundle("FIRST_ANSWER"), &exec, &cred)
        .unwrap();
    let mut b = runtime
        .start(&second, &sealed_bundle("SECOND_ANSWER"), &exec, &cred)
        .unwrap();
    let left = collect(&mut a, Duration::from_secs(15));
    let pid = native_pid(&state);
    let right = collect(&mut b, Duration::from_secs(15));
    assert!(stdout_has(&left, b"FIRST_ANSWER"));
    assert!(!stdout_has(&left, b"SECOND_ANSWER"));
    assert!(stdout_has(&right, b"SECOND_ANSWER"));
    assert!(!stdout_has(&right, b"FIRST_ANSWER"));
    assert!(
        !right.iter().any(|e| matches!(e,
            agency::runtime::RuntimeEvent::Observation {kind, ..} if kind=="session_opened"
        )),
        "metadata updates must not open a new conversation"
    );
    for events in [&left, &right] {
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, agency::runtime::RuntimeEvent::Proposal { .. }))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, agency::runtime::RuntimeEvent::TurnReturned))
                .count(),
            1
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, agency::runtime::RuntimeEvent::Exited { .. }))
        );
    }
    assert_eq!(pid, native_pid(&state));
    assert!(standby_alive(pid));
    let mut ancestor = pid;
    for _ in 0..10 {
        if ancestor == runtime.pid().unwrap() {
            break;
        }
        let out = Command::new("/bin/ps")
            .args(["-p", &ancestor.to_string(), "-o", "ppid="])
            .output()
            .unwrap();
        ancestor = String::from_utf8_lossy(&out.stdout).trim().parse().unwrap();
    }
    assert_eq!(
        ancestor,
        runtime.pid().unwrap(),
        "Herdr must own the harness"
    );
    let mut other = second.document.clone();
    other.project.id = "other-project".into();
    other.idempotency_key = "other-project-dispatch".into();
    let other = agency_proto::Sealed::new(other).unwrap();
    let mut other_running = runtime
        .start(&other, &sealed_bundle("OTHER_PROJECT"), &exec, &cred)
        .unwrap();
    assert!(stdout_has(
        &collect(&mut other_running, Duration::from_secs(35)),
        b"OTHER_PROJECT"
    ));
    let other_pid = native_pid(&turn_state(&exec, &cred, &other));
    assert_ne!(
        pid, other_pid,
        "different Project identities must remain isolated"
    );
    runtime.shutdown().unwrap();
    assert!(!standby_alive(pid));
    assert!(!standby_alive(other_pid));
}

#[test]
fn standby_cancel_interrupts_only_the_turn_and_next_dispatch_reuses_the_session() {
    use agency::runtime::RuntimeEvent;
    let (cred, root, exec) = standby_root("standby-cancel");
    let claude = waiting_claude(&root, true);
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let spec = sealed_spec("wait", now_ms() + 60_000);
    let state = turn_state(&exec, &cred, &spec);
    let mut running = runtime
        .start(&spec, &sealed_bundle("wait"), &exec, &cred)
        .unwrap();
    wait_for(&state.join("started.json"));
    let pid = native_pid(&state);
    running.session.lock().unwrap().stop().unwrap();
    let events = collect(&mut running, Duration::from_secs(10));
    assert!(events.iter().any(|e| matches!(
        e,
        RuntimeEvent::TurnStopped {
            requested_stop: true,
            session_closed: false
        }
    )));
    assert!(!events.iter().any(|e| matches!(
        e,
        RuntimeEvent::Proposal { .. } | RuntimeEvent::TurnReturned
    )));
    assert!(standby_alive(pid));
    let mut next = runtime
        .start(
            &sealed_spec("after-cancel", now_ms() + 30_000),
            &sealed_bundle("AFTER_CANCEL"),
            &exec,
            &cred,
        )
        .unwrap();
    assert!(stdout_has(
        &collect(&mut next, Duration::from_secs(10)),
        b"AFTER_CANCEL"
    ));
    assert_eq!(pid, native_pid(&state));
    runtime.shutdown().unwrap();
}

#[test]
fn standby_only_a_marker_enters_the_composer_and_task_bytes_are_preserved() {
    use agency::runtime::RuntimeEvent;
    let (cred, root, exec) = standby_root("standby-marker");
    let claude = waiting_claude(&root, true);
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let state = turn_state(&exec, &cred, &sealed_spec("marker", now_ms() + 60_000));
    let long_line = "plain task ".repeat(700);
    let task = "  Summarize the notes.\r\n".to_owned() + &"\tplain note  \r\n".repeat(22);
    for (index, text) in [
        task.as_str(),
        long_line.as_str(),
        "!touch /not-executed",
        "/clear",
    ]
    .into_iter()
    .enumerate()
    {
        let spec = sealed_spec(&format!("marker-{index}"), now_ms() + 60_000);
        let mut running = runtime
            .start(&spec, &sealed_bundle(text), &exec, &cred)
            .unwrap();
        let events = collect(&mut running, Duration::from_secs(35));
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, RuntimeEvent::TurnReturned))
                .count(),
            1,
            "task case {index}"
        );
        let started: serde_json::Value =
            serde_json::from_slice(&std::fs::read(state.join("started.json")).unwrap()).unwrap();
        assert_eq!(started["text"], format!("{text}\n"));
        let submitted = std::fs::read_to_string(state.join("composer.log")).unwrap();
        let input: String = serde_json::from_str(submitted.lines().last().unwrap()).unwrap();
        assert_eq!(input, format!("HCTL2_DISPATCH_{}", spec.digest));
        assert!(!input.contains(text));
    }
    runtime.shutdown().unwrap();
}

#[test]
fn standby_early_stop_never_resubmits_the_cancelled_prompt() {
    use agency::runtime::RuntimeEvent;
    let (cred, root, exec) = standby_root("standby-early-stop");
    let claude = waiting_claude(&root, true);
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let state = turn_state(&exec, &cred, &sealed_spec("warm", now_ms() + 60_000));
    let mut warm = runtime
        .start(
            &sealed_spec("warm", now_ms() + 60_000),
            &sealed_bundle("WARM"),
            &exec,
            &cred,
        )
        .unwrap();
    assert!(stdout_has(
        &collect(&mut warm, Duration::from_secs(35)),
        b"WARM"
    ));
    let pid = native_pid(&state);
    for cancel in [true, false] {
        let key = if cancel {
            "cancel-early"
        } else {
            "deadline-early"
        };
        let mut running = runtime
            .start(
                &sealed_spec(key, now_ms() + if cancel { 60_000 } else { 1_500 }),
                &sealed_bundle("wait-early"),
                &exec,
                &cred,
            )
            .unwrap();
        wait_started_job(&state, key);
        if cancel {
            running.session.lock().unwrap().stop().unwrap();
        }
        let events = collect(&mut running, Duration::from_secs(10));
        assert!(events.iter().any(|e| matches!(e,
            RuntimeEvent::TurnStopped {requested_stop, session_closed:false} if *requested_stop==cancel)));
        assert!(!events.iter().any(|e| matches!(
            e,
            RuntimeEvent::Proposal { .. } | RuntimeEvent::TurnReturned
        )));
        wait_for(&state.join("draft-restored"));
        let text = "  ONLY_NEXT_PROMPT\r\n".to_owned() + &"\tnew note  \r\n".repeat(22);
        let mut next = runtime
            .start(
                &sealed_spec(&format!("after-{key}"), now_ms() + 30_000),
                &sealed_bundle(&text),
                &exec,
                &cred,
            )
            .unwrap();
        let events = collect(&mut next, Duration::from_secs(10));
        let bytes: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                RuntimeEvent::Proposal { bytes, .. } => Some(bytes.as_slice()),
                _ => None,
            })
            .collect();
        assert_eq!(bytes, [text.trim().as_bytes()]);
        let started: serde_json::Value =
            serde_json::from_slice(&std::fs::read(state.join("started.json")).unwrap()).unwrap();
        assert_eq!(started["text"], format!("{text}\n"));
        assert_eq!(
            pid,
            native_pid(&state),
            "early stop must preserve the conversation process"
        );
        std::fs::remove_file(state.join("draft-restored")).unwrap();
    }
    runtime.shutdown().unwrap();
}

#[test]
fn standby_a_dead_between_turns_harness_is_resumed_before_the_next_delivery() {
    use agency::runtime::RuntimeEvent;
    let (cred, root, exec) = standby_root("standby-dead-idle");
    let claude = waiting_claude(&root, true);
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let spec = sealed_spec("before-death", now_ms() + 60_000);
    let state = turn_state(&exec, &cred, &spec);
    let mut first = runtime
        .start(&spec, &sealed_bundle("BEFORE_DEATH"), &exec, &cred)
        .unwrap();
    assert!(stdout_has(
        &collect(&mut first, Duration::from_secs(35)),
        b"BEFORE_DEATH"
    ));
    let pid = native_pid(&state);
    assert!(
        Command::new("/bin/kill")
            .args(["-KILL", &pid.to_string()])
            .status()
            .unwrap()
            .success()
    );
    let timeout = Instant::now() + Duration::from_secs(5);
    while standby_alive(pid) && Instant::now() < timeout {
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(!standby_alive(pid));
    let mut next = runtime
        .start(
            &sealed_spec("after-death", now_ms() + 60_000),
            &sealed_bundle("AFTER_DEATH"),
            &exec,
            &cred,
        )
        .unwrap();
    let events = collect(&mut next, Duration::from_secs(35));
    assert!(stdout_has(&events, b"AFTER_DEATH"));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RuntimeEvent::Observation {kind,payload,..}
        if kind=="session_opened" && payload["resumed"]==true))
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RuntimeEvent::ProtocolError(_)))
    );
    assert_ne!(pid, native_pid(&state));
    let delivered: Vec<String> = std::fs::read_to_string(state.join("delivered.log"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(delivered, ["BEFORE_DEATH\n", "AFTER_DEATH\n"]);
    runtime.shutdown().unwrap();
}

#[test]
fn standby_rejects_completions_from_a_different_job_digest_or_session() {
    for field in ["job", "digest", "session"] {
        rejects_native_event("returned.json", json!({(field): "not-this-dispatch"}));
    }
}

#[test]
fn standby_rejects_a_completion_from_a_different_native_turn() {
    rejects_native_event("returned.json", json!({"turnId":"not-this-turn"}));
}

#[test]
fn standby_rejects_a_subagent_completion() {
    rejects_native_event("returned.json", json!({"agentId":"subagent"}));
}

#[test]
fn standby_rejects_a_started_turn_with_another_prompt() {
    rejects_native_event("started.json", json!({"text":"OLD_PROMPT_AND_THIS_PROMPT"}));
}

fn rejects_native_event(file: &str, overrides: serde_json::Value) {
    use agency::runtime::RuntimeEvent;
    let case = agency_proto::hash(&agency_proto::canonical(&json!([file, overrides])).unwrap());
    let (cred, root, exec) = standby_root(&format!("standby-wrong-{}", &case[..12]));
    let claude = waiting_claude(&root, true);
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let spec = sealed_spec("warm", now_ms() + 60_000);
    let state = turn_state(&exec, &cred, &spec);
    let mut warm = runtime
        .start(&spec, &sealed_bundle("WARM"), &exec, &cred)
        .unwrap();
    assert!(stdout_has(
        &collect(&mut warm, Duration::from_secs(35)),
        b"WARM"
    ));
    std::fs::write(
        state.join(format!("{file}-override")),
        serde_json::to_vec(&overrides).unwrap(),
    )
    .unwrap();
    let mut wrong = runtime
        .start(
            &sealed_spec("wrong-event", now_ms() + 30_000),
            &sealed_bundle("THIS_PROMPT"),
            &exec,
            &cred,
        )
        .unwrap();
    let events = collect(&mut wrong, Duration::from_secs(10));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RuntimeEvent::ProtocolError(_))),
        "wrong {file}: {overrides}"
    );
    assert!(!events.iter().any(|e| matches!(
        e,
        RuntimeEvent::Proposal { .. } | RuntimeEvent::TurnReturned
    )));
    runtime.shutdown().unwrap();
}

#[test]
#[ignore = "UNVERIFIED: requires the installed Claude plugin test engine"]
fn native_mod_replaces_early_cancel_drafts_and_rejects_wrong_submissions() {
    let (cred, root, exec) = standby_root("standby-native-mod");
    let claude = waiting_claude(&root, true);
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let spec = sealed_spec("warm", now_ms() + 60_000);
    let state = turn_state(&exec, &cred, &spec);
    let mut warm = runtime
        .start(&spec, &sealed_bundle("WARM"), &exec, &cred)
        .unwrap();
    assert!(stdout_has(
        &collect(&mut warm, Duration::from_secs(35)),
        b"WARM"
    ));
    let plugin = state.join("plugin");
    std::fs::write(plugin.join("input.test.ts"), include_str!("turn.test.ts")).unwrap();
    let output = Command::new("claude")
        .args(["plugin", "test"])
        .arg(&plugin)
        .output()
        .unwrap();
    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    runtime.shutdown().unwrap();
    assert!(output.status.success(), "native Mods input tests failed");
}

#[test]
#[ignore = "UNVERIFIED: requires a logged-in Claude session and HCTL2_HARNESS_LIVE=1"]
fn live_early_stop_keeps_only_the_next_prompt_and_idle_death_resumes_once() {
    use agency::runtime::RuntimeEvent;
    assert!(std::env::var_os("HCTL2_HARNESS_LIVE").is_some());
    let located = Command::new("/usr/bin/which")
        .arg("claude")
        .output()
        .unwrap();
    let claude = PathBuf::from(String::from_utf8(located.stdout).unwrap().trim());
    let (cred, _root, exec) = standby_root("live-early-stop");
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let profession = runtime.catalog().unwrap().professions.remove(0);
    let prepare = |key: &str, text: &str, deadline| {
        let bundle = sealed_bundle(text);
        let mut spec = sealed_spec(key, deadline).document;
        spec.profession = profession.clone();
        spec.bundle.digest = bundle.digest.clone();
        (agency_proto::Sealed::new(spec).unwrap(), bundle)
    };
    let (warm, bundle) = prepare(
        "warm",
        "Reply exactly WARM and nothing else.",
        now_ms() + 120_000,
    );
    let state = turn_state(&exec, &cred, &warm);
    let mut running = runtime.start(&warm, &bundle, &exec, &cred).unwrap();
    assert!(stdout_has(
        &collect(&mut running, Duration::from_secs(120)),
        b"WARM"
    ));
    let pid = real_claude_pid(&state);
    for cancel in [true, false] {
        let key = if cancel {
            "cancel-early"
        } else {
            "deadline-early"
        };
        let (spec, bundle) = prepare(
            key,
            "Count from 1 to 400, one number per line. OLD_CANCELLED_INPUT. Do not use tools.",
            now_ms() + if cancel { 120_000 } else { 1_500 },
        );
        let mut running = runtime.start(&spec, &bundle, &exec, &cred).unwrap();
        wait_started_job(&state, key);
        if cancel {
            running.session.lock().unwrap().stop().unwrap();
        }
        let events = collect(&mut running, Duration::from_secs(30));
        assert!(events.iter().any(|e| matches!(e, RuntimeEvent::TurnStopped {requested_stop,session_closed:false} if *requested_stop==cancel)));
        assert!(!events.iter().any(|e| matches!(
            e,
            RuntimeEvent::Proposal { .. } | RuntimeEvent::TurnReturned
        )));
        let text = "Sum the values below. Reply only NEXT_TOTAL=<sum>, without tools.\n".to_owned()
            + &(1..=22)
                .map(|n| format!("Note {n}: value={n}.\n"))
                .collect::<String>();
        let (next, bundle) = prepare(&format!("after-{key}"), &text, now_ms() + 120_000);
        let mut running = runtime.start(&next, &bundle, &exec, &cred).unwrap();
        let events = collect(&mut running, Duration::from_secs(120));
        let answers: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                RuntimeEvent::Proposal { bytes, .. } => Some(bytes.as_slice()),
                _ => None,
            })
            .collect();
        assert_eq!(answers, [b"NEXT_TOTAL=253".as_slice()]);
        let started: serde_json::Value =
            serde_json::from_slice(&std::fs::read(state.join("started.json")).unwrap()).unwrap();
        assert_eq!(started["text"], format!("{text}\n"));
        assert_eq!(real_claude_pid(&state), pid);
        eprintln!(
            "LIVE {key} -> long next turn: PID {pid}, bytes={}, byte-exact=true, answer=NEXT_TOTAL=253",
            text.len() + 1
        );
    }
    let before: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state.join("resume.json")).unwrap()).unwrap();
    assert!(
        Command::new("/bin/kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .unwrap()
            .success()
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while standby_alive(pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(!standby_alive(pid));
    let (next, bundle) = prepare(
        "after-death",
        "Reply exactly RESTORED_ONCE and nothing else.",
        now_ms() + 120_000,
    );
    let mut running = runtime.start(&next, &bundle, &exec, &cred).unwrap();
    let events = collect(&mut running, Duration::from_secs(120));
    assert!(stdout_has(&events, b"RESTORED_ONCE"));
    assert!(events.iter().any(|e| matches!(e, RuntimeEvent::Observation {kind,payload,..} if kind=="session_opened"&&payload["resumed"]==true)));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RuntimeEvent::ProtocolError(_)))
    );
    let after: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state.join("resume.json")).unwrap()).unwrap();
    assert_eq!(before, after);
    assert_ne!(pid, real_claude_pid(&state));
    eprintln!(
        "LIVE idle harness death: same native Session {}, next dispatch returned RESTORED_ONCE",
        after["session"]
    );
    runtime.shutdown().unwrap();
}

#[test]
#[ignore = "UNVERIFIED: requires a logged-in Claude session and HCTL2_HARNESS_LIVE=1"]
fn live_dispatch_injects_long_tasks_and_command_shaped_text_without_composer_rewriting() {
    use agency::runtime::RuntimeEvent;
    assert!(std::env::var_os("HCTL2_HARNESS_LIVE").is_some());
    let located = Command::new("/usr/bin/which")
        .arg("claude")
        .output()
        .unwrap();
    let claude = PathBuf::from(String::from_utf8(located.stdout).unwrap().trim());
    let (cred, root, exec) = standby_root("live-marker");
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let profession = runtime.catalog().unwrap().professions.remove(0);
    let state = turn_state(&exec, &cred, &sealed_spec("marker", now_ms() + 120_000));
    let run = |key: &str, text: &str| {
        let bundle = sealed_bundle(text);
        let mut spec = sealed_spec(key, now_ms() + 120_000).document;
        spec.profession = profession.clone();
        spec.bundle.digest = bundle.digest.clone();
        let spec = agency_proto::Sealed::new(spec).unwrap();
        let mut running = runtime.start(&spec, &bundle, &exec, &cred).unwrap();
        let events = collect(&mut running, Duration::from_secs(120));
        let answers: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                RuntimeEvent::Proposal { bytes, .. } => {
                    Some(String::from_utf8(bytes.clone()).unwrap())
                }
                _ => None,
            })
            .collect();
        assert_eq!(answers.len(), 1, "{key}: no unique answer");
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, RuntimeEvent::TurnReturned))
                .count(),
            1
        );
        let started: serde_json::Value =
            serde_json::from_slice(&std::fs::read(state.join("started.json")).unwrap()).unwrap();
        assert_eq!(
            started["text"],
            format!("{text}\n"),
            "{key}: native turn bytes changed"
        );
        eprintln!(
            "LIVE body={key}, bytes={}, byte-exact=true, answer={}",
            text.len() + 1,
            answers[0]
        );
        answers.into_iter().next().unwrap()
    };
    assert_eq!(
        run(
            "remember",
            "Remember HCTL3D_COPPER_627 for later. Reply only HCTL3D_COPPER_627."
        ),
        "HCTL3D_COPPER_627"
    );
    let pid = real_claude_pid(&state);
    let session = std::fs::read(state.join("resume.json")).unwrap();
    let task = "Summarize the common goal of these notes in one short sentence. Then compute the sum of values and report SUM=<number>. Do not ask questions or use tools.\n".to_owned()
        + &(1..=22).map(|n| format!("Note {n}: Improve cache reuse. Value={n}.\n")).collect::<String>();
    let answer = run("long-task", &task);
    assert!(
        answer.to_lowercase().contains("cache") && answer.contains("SUM=253"),
        "task was not performed: {answer}"
    );
    let single = "Summarize the following repeated statement in one short sentence, then write LINE_READ. Do not use tools: ".to_owned()
        + &"The cache should avoid rebuilding dependencies. ".repeat(100);
    let answer = run("long-line", &single);
    assert!(answer.to_lowercase().contains("cache") && answer.contains("LINE_READ"));
    assert_eq!(
        run(
            "whitespace",
            "  Compute 19+23 and reply only TOTAL=<result>.\r\n\tDo not use tools.  \r\n  "
        ),
        "TOTAL=42"
    );
    let touched = root.join("not-created-by-bang");
    run("bang", &format!("!touch {}", touched.display()));
    assert!(
        !touched.exists(),
        "task text passed through native shell mode"
    );
    run("clear", "/clear");
    assert_eq!(
        run(
            "recall-after-clear",
            "What codeword did I give in my first message? Reply only that word, without tools."
        ),
        "HCTL3D_COPPER_627"
    );
    assert_eq!(pid, real_claude_pid(&state));
    assert_eq!(session, std::fs::read(state.join("resume.json")).unwrap());
    eprintln!("LIVE marker delivery: same PID {pid}, /clear did not clear the native conversation");
    runtime.shutdown().unwrap();
}

#[test]
fn standby_error_is_not_a_result_and_expiration_ends_only_its_dispatch() {
    use agency::runtime::RuntimeEvent;
    let (cred, root, exec) = standby_root("standby-errors");
    let claude = waiting_claude(&root, true);
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let mut failed = runtime
        .start(
            &sealed_spec("error", now_ms() + 60_000),
            &sealed_bundle("error"),
            &exec,
            &cred,
        )
        .unwrap();
    let events = collect(&mut failed, Duration::from_secs(10));
    assert!(
        events
            .iter()
            .any(|e| matches!(e,RuntimeEvent::ProtocolError(code) if code=="HARNESS_TURN_ERROR"))
    );
    assert!(!events.iter().any(|e| matches!(
        e,
        RuntimeEvent::Proposal { .. } | RuntimeEvent::TurnReturned
    )));
    // Warm the fresh session first: this must exercise an active turn's
    // deadline, not just expiry while the native session is starting.
    let warm = sealed_spec("warm", now_ms() + 60_000);
    let state = turn_state(&exec, &cred, &warm);
    let mut ready = runtime
        .start(&warm, &sealed_bundle("WARM"), &exec, &cred)
        .unwrap();
    assert!(stdout_has(
        &collect(&mut ready, Duration::from_secs(10)),
        b"WARM"
    ));
    let pid = native_pid(&state);
    let spec = sealed_spec("expired", now_ms() + 3_000);
    let mut expired = runtime
        .start(&spec, &sealed_bundle("wait"), &exec, &cred)
        .unwrap();
    let events = collect(&mut expired, Duration::from_secs(10));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, RuntimeEvent::DeadlineReached))
    );
    assert!(events.iter().any(|e| matches!(
        e,
        RuntimeEvent::TurnStopped {
            requested_stop: false,
            session_closed: false
        }
    )));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RuntimeEvent::Proposal { .. }))
    );
    let started: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state.join("started.json")).unwrap()).unwrap();
    assert_eq!(started["job"], "expired");
    assert!(
        standby_alive(pid),
        "deadline ends the dispatch, not the native conversation"
    );
    runtime.shutdown().unwrap();
}

#[test]
fn standby_idle_reclaims_then_uses_native_resume_and_participants_are_isolated() {
    use agency::runtime::RuntimeEvent;
    let (cred, root, exec) = standby_root("standby-resume");
    let claude = waiting_claude(&root, true);
    let runtime =
        launch::InstalledHerdr::open_with_idle(binary(), &claude, Duration::from_millis(150))
            .unwrap();
    let spec = sealed_spec("before-idle", now_ms() + 60_000);
    let state = turn_state(&exec, &cred, &spec);
    let mut a = runtime
        .start(&spec, &sealed_bundle("BEFORE_IDLE"), &exec, &cred)
        .unwrap();
    assert!(stdout_has(
        &collect(&mut a, Duration::from_secs(10)),
        b"BEFORE_IDLE"
    ));
    let pid = native_pid(&state);
    let deadline = Instant::now() + Duration::from_secs(3);
    while standby_alive(pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(!standby_alive(pid));
    let mut b = runtime
        .start(
            &sealed_spec("after-idle", now_ms() + 60_000),
            &sealed_bundle("AFTER_IDLE"),
            &exec,
            &cred,
        )
        .unwrap();
    let events = collect(&mut b, Duration::from_secs(10));
    assert!(stdout_has(&events, b"AFTER_IDLE"));
    assert!(events.iter().any(|e|matches!(e,RuntimeEvent::Observation{kind,payload,..} if kind=="session_opened" && payload["resumed"]==true && payload["resume_failed"]==false)));
    assert_ne!(pid, native_pid(&state));
    let mut other = spec.document.clone();
    other.selection.id = "other-selection".into();
    other.idempotency_key = "other".into();
    let other = agency_proto::Sealed::new(other).unwrap();
    let mut c = runtime
        .start(&other, &sealed_bundle("OTHER"), &exec, &cred)
        .unwrap();
    assert!(stdout_has(
        &collect(&mut c, Duration::from_secs(10)),
        b"OTHER"
    ));
    assert_ne!(state, turn_state(&exec, &cred, &other));
    assert_ne!(
        native_pid(&state),
        native_pid(&turn_state(&exec, &cred, &other))
    );
    runtime.shutdown().unwrap();
}

#[test]
fn standby_each_dispatch_checks_its_own_permissions() {
    let (cred, root, exec) = standby_root("standby-permissions");
    let claude = waiting_claude(&root, true);
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let mut first = runtime
        .start(
            &sealed_spec("allowed", now_ms() + 60_000),
            &sealed_bundle("ALLOWED"),
            &exec,
            &cred,
        )
        .unwrap();
    assert!(stdout_has(
        &collect(&mut first, Duration::from_secs(10)),
        b"ALLOWED"
    ));
    for permissions in [vec![], vec!["context.read".into(), "file.write".into()]] {
        let mut spec = sealed_spec("forbidden", now_ms() + 60_000).document;
        spec.permissions = permissions;
        let error = match runtime.start(
            &agency_proto::Sealed::new(spec).unwrap(),
            &sealed_bundle("MUST_NOT_SEND"),
            &exec,
            &cred,
        ) {
            Ok(_) => panic!("permission leaked"),
            Err(e) => e,
        };
        assert!(["PERMISSION_DENIED", "STANDBY_READONLY"].contains(&error.code.as_str()));
    }
    runtime.shutdown().unwrap();
}

#[test]
fn standby_same_selection_ids_in_two_tenants_do_not_share_a_session() {
    let (cred, root, exec) = standby_root("standby-tenants");
    let claude = waiting_claude(&root, true);
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let spec = sealed_spec("same-dispatch-key", now_ms() + 60_000);
    let mut a = runtime
        .start_for_tenant(
            &root.join("tenant-a"),
            &spec,
            &sealed_bundle("TENANT_A"),
            &exec,
            &cred,
        )
        .unwrap();
    let mut b = runtime
        .start_for_tenant(
            &root.join("tenant-b"),
            &spec,
            &sealed_bundle("TENANT_B"),
            &exec,
            &cred,
        )
        .unwrap();
    assert!(stdout_has(
        &collect(&mut a, Duration::from_secs(10)),
        b"TENANT_A"
    ));
    assert!(stdout_has(
        &collect(&mut b, Duration::from_secs(10)),
        b"TENANT_B"
    ));
    let states: Vec<_> = std::fs::read_dir(herdr::state_dir(&exec, &cred).unwrap())
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("standby-")
        })
        .collect();
    assert_eq!(states.len(), 2);
    assert_ne!(native_pid(&states[0]), native_pid(&states[1]));
    runtime.shutdown().unwrap();
}

#[test]
fn standby_queued_cancellation_never_delivers_the_next_prompt() {
    use agency::runtime::RuntimeEvent;
    let (cred, root, exec) = standby_root("standby-queued-stop");
    let claude = waiting_claude(&root, true);
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let spec = sealed_spec("first", now_ms() + 60_000);
    let state = turn_state(&exec, &cred, &spec);
    let mut first = runtime
        .start(&spec, &sealed_bundle("wait"), &exec, &cred)
        .unwrap();
    wait_for(&state.join("started.json"));
    let mut queued = runtime
        .start(
            &sealed_spec("queued", now_ms() + 60_000),
            &sealed_bundle("MUST_NOT_SEND"),
            &exec,
            &cred,
        )
        .unwrap();
    queued.session.lock().unwrap().stop().unwrap();
    first.session.lock().unwrap().stop().unwrap();
    collect(&mut first, Duration::from_secs(10));
    let events = collect(&mut queued, Duration::from_secs(10));
    assert!(events.iter().any(|e| matches!(
        e,
        RuntimeEvent::TurnStopped {
            requested_stop: true,
            session_closed: false
        }
    )));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, RuntimeEvent::Proposal { .. }))
    );
    assert_eq!(
        std::fs::read_to_string(state.join("delivered.txt")).unwrap(),
        "wait\n"
    );
    runtime.shutdown().unwrap();
}

#[test]
fn standby_uninterruptible_turn_closes_and_failed_resume_is_reported() {
    use agency::runtime::RuntimeEvent;
    let (cred, root, exec) = standby_root("standby-close");
    let claude = waiting_claude(&root, true);
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let spec = sealed_spec("stuck", now_ms() + 60_000);
    let state = turn_state(&exec, &cred, &spec);
    let mut stuck = runtime
        .start(&spec, &sealed_bundle("uninterruptible"), &exec, &cred)
        .unwrap();
    wait_for(&state.join("started.json"));
    let pid = native_pid(&state);
    stuck.session.lock().unwrap().stop().unwrap();
    let events = collect(&mut stuck, Duration::from_secs(10));
    assert!(events.iter().any(|e| matches!(
        e,
        RuntimeEvent::TurnStopped {
            requested_stop: true,
            session_closed: true
        }
    )));
    assert!(!standby_alive(pid));
    std::fs::write(
        state.join("fail-resume"),
        b"fixture rejects old native session",
    )
    .unwrap();
    let mut next = runtime
        .start(
            &sealed_spec("fresh", now_ms() + 60_000),
            &sealed_bundle("FRESH_ANSWER"),
            &exec,
            &cred,
        )
        .unwrap();
    let events = collect(&mut next, Duration::from_secs(15));
    assert!(stdout_has(&events, b"FRESH_ANSWER"));
    assert!(events.iter().any(|e| matches!(e, RuntimeEvent::Observation{kind,payload,..} if kind=="session_opened" && payload["resumed"]==false && payload["resume_failed"]==true)));
    runtime.shutdown().unwrap();
}

struct StopAgency {
    binary: PathBuf,
    root: PathBuf,
}
impl Drop for StopAgency {
    fn drop(&mut self) {
        let _ = Command::new(&self.binary)
            .arg("--root")
            .arg(&self.root)
            .arg("stop")
            .output();
    }
}

async fn port_turn(name: &str, live: bool) {
    use agency_proto::*;
    let (root, install) = scratch(name);
    let herdr = installed_fixture(&install);
    let claude = if live {
        let found = Command::new("/usr/bin/which")
            .arg("claude")
            .output()
            .unwrap();
        PathBuf::from(String::from_utf8(found.stdout).unwrap().trim())
    } else {
        waiting_claude(&install, true)
    };
    let agency = PathBuf::from(std::env::var("HCTL2_CONFINE_BIN").unwrap());
    let _stop = StopAgency {
        binary: agency.clone(),
        root: root.clone(),
    };
    let started = Command::new(&agency)
        .arg("--root")
        .arg(&root)
        .arg("start")
        .env("HCTL2_INSTALL_ROOT", &install)
        .env("HCTL2_CLAUDE", &claude)
        .output()
        .unwrap();
    assert!(
        started.status.success(),
        "{} {}",
        String::from_utf8_lossy(&started.stdout),
        std::fs::read_to_string(root.join("serve.err")).unwrap_or_default()
    );
    let pairing = PortClient::new(
        agency_proto::client::admin_endpoint(&root).unwrap(),
        std::fs::read_to_string(root.join("pair.key")).unwrap(),
    );
    let paired: Pairing = pairing
        .call(
            "pair",
            &Pair {
                control_id: name.into(),
                tenant_key: agency_proto::client::new_credential().unwrap(),
            },
        )
        .await
        .unwrap();
    let key = paired.key.clone();
    let client = PortClient::new(paired.endpoint.into(), paired.key);
    let _: serde_json::Value = client
        .call(
            "fence",
            &Fence {
                writer_generation: 1,
            },
        )
        .await
        .unwrap();
    let catalog: Catalog = client.call("catalog", &json!({})).await.unwrap();
    let claude_profession = catalog
        .professions
        .iter()
        .find(|item| item.reference.id == "claude-code")
        .unwrap_or_else(|| {
            panic!(
                "{}",
                std::fs::read_to_string(root.join("serve.err")).unwrap_or_default()
            )
        })
        .clone();
    let expected = if live {
        "HCTL2_PORT_REAL_OK"
    } else {
        "still working; not a task completion"
    };
    let bundle = sealed_bundle(&format!(
        "Reply with exactly {expected} and do not use tools."
    ));
    let mut document = sealed_spec(name, now_ms() + 120_000).document;
    document.profession = claude_profession.clone();
    document.bundle.digest = bundle.digest.clone();
    let prepared: Dispatch = client
        .call(
            "prepare",
            &Prepare {
                spec: Sealed::new(document).unwrap(),
                bundle,
                writer_generation: 1,
            },
        )
        .await
        .unwrap();
    assert_eq!(prepared.state, DispatchState::Prepared);
    let _: Dispatch = client
        .call(
            "activate",
            &DispatchAction {
                dispatch: prepared.reference.clone(),
                writer_generation: 1,
                idempotency_key: "activate".into(),
            },
        )
        .await
        .unwrap();
    let exec = confine::execution_dir(&root, &prepared.reference).unwrap();
    let herdr_state = herdr::state_dir(&exec, &root).unwrap();
    let herdr_pid: u32 = std::fs::read_to_string(herdr_state.join("herdr.pid"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let ticket = Ticket::sign(
        TicketClaims {
            id: "observe".into(),
            actor: "human".into(),
            dispatch: prepared.reference.clone(),
            owner: prepared.owner.clone(),
            spec_digest: prepared.spec_digest.clone(),
            writer_generation: 1,
            permissions: vec![Permission::Observe, Permission::Stop],
            input_lease: None,
            expires_ms: now_ms() + 180_000,
        },
        key.as_bytes(),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(120);
    let trace = loop {
        let trace: Trace = client
            .call(
                "observe",
                &Observe {
                    ticket: ticket.clone(),
                    after: 0,
                },
            )
            .await
            .unwrap();
        if trace.dispatch.state != DispatchState::Running {
            break trace;
        }
        assert!(Instant::now() < deadline, "port turn timed out");
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    assert_eq!(
        trace.dispatch.state,
        DispatchState::ResultReturned,
        "{}",
        serde_json::to_string(&trace).unwrap()
    );
    assert!(
        trace
            .events
            .iter()
            .any(|event| event.kind == "turn_returned")
    );
    let result: ResultPage = client
        .call("results", &ResultQuery::of(prepared.reference.clone()))
        .await
        .unwrap();
    assert!(
        result.complete,
        "result should be available before process exit"
    );
    assert_eq!(result.proposals.len(), 1);
    let proposal = &result.proposals[0];
    assert_eq!(proposal.evidence, EvidenceLevel::AdapterEvent);
    let output = String::from_utf8_lossy(&proposal.output);
    assert!(output.contains(expected), "{output}");
    if !live {
        assert!(!trace.events.iter().any(|event| event.kind == "stopped"));
        let state = std::fs::read_dir(&herdr_state)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("standby-")
            })
            .unwrap();
        let pid = native_pid(&state);
        assert!(standby_alive(pid));
        let _: Dispatch = client.call("stop", &ticket).await.unwrap();
        let stopped: Trace = client
            .call(
                "observe",
                &Observe {
                    ticket: ticket.clone(),
                    after: 0,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            stopped.dispatch.state,
            DispatchState::ResultReturned,
            "stopping a returned session must not cancel the already returned turn"
        );
    }
    eprintln!(
        "{} agency start -> pair -> catalog Claude {} -> prepare -> activate -> ResultReturned",
        if live { "LIVE" } else { "FIXTURE" },
        claude_profession.reference.revision
    );
    eprintln!(
        "Proposal(schema={}, evidence=adapter_event): {output}",
        proposal.schema
    );
    eprintln!("Task acceptance / sysone / artifacts: not evaluated by this runtime");
    drop(client);
    drop(pairing);
    let stopped = Command::new(&agency)
        .arg("--root")
        .arg(&root)
        .arg("stop")
        .output()
        .unwrap();
    assert!(
        stopped.status.success(),
        "{}",
        String::from_utf8_lossy(&stopped.stdout)
    );
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && process_matches(herdr_pid, &herdr) {
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(
        !process_matches(herdr_pid, &herdr),
        "Agency stopped but its private Herdr survived"
    );
}

#[tokio::test]
async fn port_returns_a_turn_while_herdr_session_is_alive() {
    port_turn("port-turn", false).await;
}

#[tokio::test]
#[ignore = "UNVERIFIED: requires a logged-in Claude session and HCTL2_HARNESS_LIVE=1"]
async fn live_agency_start_pair_and_port_dispatch_return_one_turn() {
    assert!(
        std::env::var_os("HCTL2_HARNESS_LIVE").is_some(),
        "explicit live validation also requires HCTL2_HARNESS_LIVE=1"
    );
    port_turn("live-port", true).await;
}

fn process_matches(pid: u32, binary: &std::path::Path) -> bool {
    let Ok(output) = Command::new("/bin/ps")
        .args(["-ww", "-p", &pid.to_string(), "-o", "stat=,command="])
        .output()
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let line = String::from_utf8_lossy(&output.stdout);
    let stat = line.split_whitespace().next().unwrap_or("");
    if stat.is_empty() || stat.starts_with('Z') {
        return false;
    }
    line.contains(&binary.display().to_string())
        || line
            .split_whitespace()
            .nth(1)
            .and_then(|program| std::fs::canonicalize(program).ok())
            .is_some_and(|program| {
                binary
                    .canonicalize()
                    .is_ok_and(|expected| program == expected)
            })
}

#[test]
#[ignore = "UNVERIFIED: requires a logged-in Codex session and HCTL2_HARNESS_LIVE=1"]
fn live_codex_turn_start_matches_rollout_and_keeps_one_thread() {
    use agency::runtime::RuntimeEvent;
    assert!(std::env::var_os("HCTL2_HARNESS_LIVE").is_some());
    let claude = PathBuf::from(
        String::from_utf8(
            Command::new("/usr/bin/which")
                .arg("claude")
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim(),
    );
    let (cred, _root, exec) = standby_root("live-codex");
    let runtime = launch::InstalledHerdr::open(binary(), &claude).unwrap();
    let profession = runtime
        .catalog()
        .unwrap()
        .professions
        .into_iter()
        .find(|item| item.reference.id == "codex-cli")
        .expect("codex was not cataloged");
    let run = |key: &str, text: &str, selection: &str| {
        let bundle = sealed_bundle(text);
        let mut spec = sealed_spec(key, now_ms() + 180_000).document;
        spec.profession = profession.clone();
        spec.selection.id = selection.into();
        spec.bundle.digest = bundle.digest.clone();
        let spec = agency_proto::Sealed::new(spec).unwrap();
        let mut running = runtime.start(&spec, &bundle, &exec, &cred).unwrap();
        let events = collect(&mut running, Duration::from_secs(180));
        let answers: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                RuntimeEvent::Proposal { bytes, .. } => {
                    Some(String::from_utf8(bytes.clone()).unwrap())
                }
                _ => None,
            })
            .collect();
        assert_eq!(answers.len(), 1, "{key}");
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, RuntimeEvent::TurnReturned))
                .count(),
            1
        );
        let opened = events.iter().find_map(|event| match event {
            RuntimeEvent::Observation { kind, payload, .. } if kind == "session_opened" => {
                Some(payload.clone())
            }
            _ => None,
        });
        (answers.into_iter().next().unwrap(), opened)
    };
    let (answer, opened) = run(
        "one",
        "Reply with exactly HCTL3E_ONE and nothing else.",
        "codex-a",
    );
    assert!(answer.contains("HCTL3E_ONE"), "{answer}");
    let thread = opened.unwrap()["thread"].as_str().unwrap().to_owned();
    assert!(rollout_has(
        &thread,
        "Reply with exactly HCTL3E_ONE and nothing else.\n"
    ));
    let bang = "!\nReply with exactly HCTL3E_BANG and nothing else.";
    let (answer, _) = run("bang", bang, "codex-a");
    assert!(answer.contains("HCTL3E_BANG"), "{answer}");
    assert!(rollout_has(&thread, &format!("{bang}\n")));
    let slash = "/\nReply with exactly HCTL3E_SLASH and nothing else.";
    let (answer, _) = run("slash", slash, "codex-a");
    assert!(answer.contains("HCTL3E_SLASH"), "{answer}");
    assert!(rollout_has(&thread, &format!("{slash}\n")));
    let long = format!(
        "Reply with exactly HCTL3E_LONG and nothing else.\n{}",
        "cache line\n".repeat(200)
    );
    assert!(long.len() > 2000);
    let (answer, _) = run("long", &long, "codex-a");
    assert!(answer.contains("HCTL3E_LONG"), "{answer}");
    assert!(rollout_has(&thread, &format!("{long}\n")));
    let ps = String::from_utf8(
        Command::new("/bin/ps")
            .args(["-ax", "-o", "command="])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert!(
        ps.lines()
            .any(|line| line.contains(&thread) && line.contains("--remote")),
        "pane process was not still codex resume --remote"
    );
    let (answer, opened) = run(
        "other",
        "Reply with exactly HCTL3E_OTHER and nothing else.",
        "codex-b",
    );
    assert!(answer.contains("HCTL3E_OTHER"), "{answer}");
    assert_ne!(opened.unwrap()["thread"], thread);
    runtime.shutdown().unwrap();
    let ps = String::from_utf8(
        Command::new("/bin/ps")
            .args(["-ax", "-o", "command="])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert!(
        !ps.lines()
            .any(|line| line.contains("app-server --listen unix://") && line.contains("codex-")),
        "app-server was still running after shutdown"
    );
}

fn rollout_has(thread: &str, text: &str) -> bool {
    let home = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap()).join(".codex"));
    let mut stack = vec![home.join("sessions")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !name.contains(thread) {
                continue;
            }
            let Ok(body) = std::fs::read_to_string(&path) else {
                continue;
            };
            for line in body.lines() {
                let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
                    continue;
                };
                if json_has_input(&value, text) {
                    return true;
                }
            }
        }
    }
    false
}

fn json_has_input(value: &serde_json::Value, text: &str) -> bool {
    let mut stack = vec![value];
    while let Some(current) = stack.pop() {
        match current {
            serde_json::Value::Object(map) => {
                if map.get("type").and_then(|v| v.as_str()) == Some("input_text")
                    && map.get("text").and_then(|v| v.as_str()) == Some(text)
                {
                    return true;
                }
                stack.extend(map.values());
            }
            serde_json::Value::Array(items) => stack.extend(items.iter()),
            _ => {}
        }
    }
    false
}

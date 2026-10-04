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
        "while true; do date +%s > '{}'; sleep 0.2; done",
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
fn a_script_exit_file_is_the_completion_signal() {
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
    assert_eq!(finished.code, 0);
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
    assert_eq!(finished.code, 7);
    let _ = launch.cancel();
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    let _ = std::fs::remove_dir_all(&state);
}

#[test]
fn cancel_stops_before_the_exit_file_exists() {
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
    assert_eq!(finished.code, 0);
    assert!(exec.join("finished").exists());
    let _ = launch.cancel();
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    let _ = std::fs::remove_dir_all(&state);
}

#[test]
fn the_pane_program_cannot_write_the_herdr_state_directory() {
    let (cred, exec, state, server) = server("statedeny");
    assert!(!state.starts_with(&exec));
    let body = "mkdir -p herdr-state && touch herdr-state/pwned\ntouch wrote-ok\nexit 0\n";
    let launch = launch::Launch::start(server, &exec, &state, &cred, body, "statedeny").unwrap();
    let finished = launch.wait(Duration::from_secs(15)).unwrap();
    assert_eq!(finished.code, 0);
    assert!(exec.join("wrote-ok").exists());
    assert!(!state.join("pwned").exists());
    let _ = launch.cancel();
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    let _ = std::fs::remove_dir_all(&state);
}

#[test]
fn a_tampered_install_is_not_the_locked_herdr() {
    let install = std::env::temp_dir().join(format!("hctl2-install-{}", std::process::id()));
    let dest = install.join("libexec/hctl2/herdr");
    let _ = std::fs::remove_dir_all(&install);
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
    std::fs::copy(binary(), &dest).unwrap();
    assert!(launch::installed_herdr(&install).is_ok());
    let mut bytes = std::fs::read(&dest).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&dest, &bytes).unwrap();
    assert_eq!(
        launch::installed_herdr(&install).unwrap_err().code,
        "HERDR_DIGEST_MISMATCH"
    );
    let _ = std::fs::remove_dir_all(&install);
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
    let dest = install.join("libexec/hctl2/herdr");
    let _ = std::fs::remove_dir_all(&install);
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
    std::fs::copy(binary(), &dest).unwrap();
    let mut permissions = std::fs::metadata(&dest).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&dest, permissions).unwrap();
    let runtime = launch::InstalledHerdr::open(dest).unwrap();
    let catalog = runtime.catalog().unwrap();
    assert_eq!(catalog.professions[0].reference.id, "herdr-locked");
    assert_eq!(catalog.professions[0].reference.revision, "protocol-20");
    assert_eq!(catalog.harnesses[0].digest.len(), 64);
    let _ = std::fs::remove_dir_all(&install);
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
    assert_eq!(finished.code, 0);
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
    assert!(harness::version_at_least("0.160.0", harness::CODEX_MINIMUM));
    assert!(!harness::version_at_least(
        "0.153.3",
        harness::CODEX_MINIMUM
    ));
    if std::env::var_os("HCTL2_HARNESS_LIVE").is_none() {
        eprintln!("UNVERIFIED claude session: CI has no harness credential; live flag is unset");
        eprintln!("UNVERIFIED codex session: the second harness is not run in this package");
        return;
    }
    let session = agency::harness::claude::print_session(
        "Reply with exactly HCTL2_REAL_OK and do not use tools.\n",
        Duration::from_secs(120),
    )
    .unwrap();
    assert!(!session.is_error, "{}", session.result);
    assert!(
        session.result.contains("HCTL2_REAL_OK"),
        "{}",
        session.result
    );
    assert!(!session.session_id.is_empty());
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
}

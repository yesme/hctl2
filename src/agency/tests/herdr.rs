use agency::{
    confine,
    herdr::{self, Client, Server},
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
    assert!(state.starts_with(&exec));
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
fn a_marker_inside_the_command_is_not_the_echo() {
    let (cred, exec) = scratch("echo");
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
        "echo",
        "sleep 2; printf HCTL2DONE; touch executed",
        "HCTL2DONE",
    )
    .unwrap();
    assert!(started.elapsed() >= Duration::from_millis(1500));
    assert!(text.matches("HCTL2DONE").count() > 1);
    assert!(exec.join("executed").exists());
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

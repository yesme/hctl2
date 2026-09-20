//! CLI wiring for init/start/status/doctor/export/backup/restore against a live daemon.

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

fn hctl2() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hctl2"))
}

fn control() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hctl2-control"))
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
fn init_start_status_doctor_backup_restore_round_trip() {
    let temp = Temp::new();
    let root = temp.0.as_path();
    let (ok, stdout, stderr) = run(root, &["init"]);
    assert!(ok, "init {stderr} {stdout}");
    let (ok, stdout, stderr) = run(root, &["start"]);
    assert!(ok, "start {stderr} {stdout}");
    let (ok, stdout, stderr) = run(root, &["status"]);
    assert!(ok, "status {stderr} {stdout}");
    assert!(stdout.contains("control_id"), "{stdout}");
    assert!(stdout.contains("local-owner"), "{stdout}");
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
    let (ok, out, err) = run(root, &["start"]);
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

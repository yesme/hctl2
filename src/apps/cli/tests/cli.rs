//! CLI wiring for init/start/status/doctor/export/backup/restore against a live daemon.

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
}

use std::process::Command;

#[test]
fn binary_prints_english_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_hctl2-tool"))
        .arg("--help")
        .output()
        .expect("hctl2-tool must start");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help output must be UTF-8");
    assert!(stdout.contains("Mechanical Git/SCM toolbox for HCTL2"));
    assert!(stdout.contains("Usage:"));
}

#[test]
fn binary_prints_its_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_hctl2-tool"))
        .arg("--version")
        .output()
        .expect("hctl2-tool must start");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("version output must be UTF-8"),
        format!("hctl2-tool {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn binary_reports_a_structured_startup_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_hctl2-tool"))
        .arg("not-a-command")
        .output()
        .expect("hctl2-tool must start");

    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .expect("error output must be UTF-8")
            .starts_with("error[HCTL2_TOOL_INVALID_ARGUMENT]:")
    );
}

#[test]
fn wait_prints_one_structured_timeout_record() {
    let output = Command::new(env!("CARGO_BIN_EXE_hctl2-tool"))
        .args([
            "wait",
            "--deadline",
            "0",
            "path-digest",
            "--path",
            "missing",
            "--sha256",
            "0000000000000000000000000000000000000000000000000000000000000000",
        ])
        .output()
        .expect("hctl2-tool must start");

    assert_eq!(output.status.code(), Some(5));
    let stdout = String::from_utf8(output.stdout).expect("record must be UTF-8");
    assert_eq!(stdout.lines().count(), 1);
    let record: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("record must be JSON");
    assert_eq!(record["outcome"], "timeout");
    assert_eq!(record["evidence_level"], "toolbox_readback");
}

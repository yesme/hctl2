use std::process::Command;

#[test]
fn binary_prints_english_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_hctl2-web-server"))
        .arg("--help")
        .output()
        .expect("hctl2-web-server must start");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help output must be UTF-8");
    assert!(stdout.contains("Serve a static directory"));
    assert!(stdout.contains("Usage:"));
}

#[test]
fn binary_rejects_missing_root() {
    let output = Command::new(env!("CARGO_BIN_EXE_hctl2-web-server"))
        .args(["--port", "6168"])
        .output()
        .expect("hctl2-web-server must start");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .expect("error output must be UTF-8")
            .starts_with("error[HCTL2_WEB_SERVER_ARGUMENT]:")
    );
}

//! Shared fixture behavior. Real harness tests do not use this module.
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

pub fn execute(text: &str, cwd: &Path, enabled: bool) -> Option<Value> {
    let line = text
        .lines()
        .find_map(|line| line.strip_prefix("HCTL2_WRITE_FIXTURE "))?;
    let request: Value = serde_json::from_str(line).unwrap();
    assert!(
        enabled,
        "fixture received write without native write permission"
    );
    let reads: Vec<_> = request["secret_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| {
            let path = path.as_str().unwrap();
            json!({"path":path,"readable":fs::read(path).is_ok()})
        })
        .collect();
    if request["edit"] == true {
        fs::write(
            cwd.join("calculator.py"),
            "def double(value):\n    return value * 2\n",
        )
        .unwrap();
    }
    let test = Command::new("/usr/bin/python3")
        .args(["-m", "unittest", "-v", "test_calculator"])
        .current_dir(cwd)
        .output()
        .unwrap();
    let git = Command::new("/usr/bin/git")
        .args(["rev-parse", "HEAD"])
        .current_dir(cwd)
        .output()
        .unwrap();
    let branch = Command::new("/usr/bin/git")
        .args(["symbolic-ref", "-q", "HEAD"])
        .current_dir(cwd)
        .output()
        .unwrap();
    let gh = Command::new("gh")
        .args(["auth", "status"])
        .current_dir(cwd)
        .output();
    Some(
        json!({"cwd":cwd,"head":String::from_utf8_lossy(&git.stdout).trim(),"git_success":git.status.success(),"git_error":String::from_utf8_lossy(&git.stderr),"detached":if git.status.success() { Some(!branch.status.success()) } else { None },"reads":reads,
        "test_success":test.status.success(), "test_output":String::from_utf8_lossy(&test.stderr),
        "gh_authenticated":gh.is_ok_and(|o| o.status.success()),"gh_token_present":std::env::var_os("GH_TOKEN").is_some() || std::env::var_os("GITHUB_TOKEN").is_some(),
        "dbus_present":std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some()}),
    )
}

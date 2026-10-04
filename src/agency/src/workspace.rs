//! Worktree materialization through hctl2-tool, with the tool's own stdout as the report.
use agency_proto::{EvidenceLevel, PortError, Result};
use serde_json::Value;
use std::{path::Path, process::Command};

pub struct ToolReport {
    pub kind: String,
    pub payload: Value,
    pub source: EvidenceLevel,
}

fn tool_bin() -> Result<std::path::PathBuf> {
    if let Some(path) = std::env::var_os("HCTL2_TOOL") {
        return Ok(path.into());
    }
    Ok("hctl2-tool".into())
}

pub fn run(args: &[&str], cwd: &Path) -> Result<ToolReport> {
    let output = Command::new(tool_bin()?)
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", cwd)
        .env("LANG", "C.UTF-8")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let payload = serde_json::from_str::<Value>(&stdout).unwrap_or_else(|_| {
        serde_json::json!({
            "exit": output.status.code(),
            "stdout": stdout.trim().to_owned(),
            "stderr": String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        })
    });
    if !output.status.success() {
        return Err(PortError::new(
            "TOOL_FAILED",
            format!("hctl2-tool {} failed", args.first().copied().unwrap_or("")),
            "read_tool_report",
        ));
    }
    Ok(ToolReport {
        kind: format!("tool:{}", args.first().copied().unwrap_or("hctl2-tool")),
        payload,
        source: EvidenceLevel::AdapterEvent,
    })
}

pub fn inspect_repo(repo: &Path) -> Result<ToolReport> {
    let path = repo.display().to_string();
    run(&["repo", "inspect", "--path", &path], repo)
}

pub fn materialize(
    repo: &Path,
    root: &Path,
    change_set: &str,
    baseline: &str,
) -> Result<ToolReport> {
    let repo_path = repo.display().to_string();
    let root_path = root.display().to_string();
    run(
        &[
            "worktree",
            "materialize",
            "--repo",
            &repo_path,
            "--root",
            &root_path,
            "--change-set-ref",
            change_set,
            "--baseline",
            baseline,
        ],
        repo,
    )
}

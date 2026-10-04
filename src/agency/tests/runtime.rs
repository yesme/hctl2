use agency::{
    catalog, confine, herdr,
    runtime::{Runtime, ScriptConfig, ScriptRuntime},
};
use agency_proto::hash;
use std::{os::unix::fs::PermissionsExt, path::PathBuf, process::Stdio};

#[test]
fn script_catalog_digest_is_the_program_file() {
    let runtime = ScriptRuntime::new(ScriptConfig {
        program: "/bin/sh".into(),
        arguments: vec![],
    });
    let catalog = runtime.catalog().unwrap();
    assert_eq!(
        catalog.harnesses[0].digest,
        hash(&std::fs::read("/bin/sh").unwrap())
    );
}

#[test]
fn confined_child_cannot_read_the_credential_root() {
    let cred = std::env::temp_dir().join(format!("hctl2-cred-{}", std::process::id()));
    let exec = std::env::temp_dir().join(format!("hctl2-exec-probe-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    std::fs::create_dir_all(&cred).unwrap();
    std::fs::create_dir_all(&exec).unwrap();
    std::fs::set_permissions(&cred, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(cred.join("pair.key"), "secret-credential").unwrap();
    let cred = cred.canonicalize().unwrap();
    let marker = exec.join("marker");
    let script = format!(
        "if /bin/cat '{}' >/dev/null 2>&1; then echo READ_OK; else echo READ_DENIED; fi; echo \"secret=${{HCTL2_SECRET-}}\"; echo done > '{}'",
        cred.join("pair.key").display(),
        marker.display()
    );
    let program = PathBuf::from("/bin/sh");
    let mut child = confine::command(&program, &["-c".into(), script], &exec, &cred).unwrap();
    child.env("HCTL2_SECRET", "should-not-pass");
    confine::scrub(&mut child, &exec);
    let output = child
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains("READ_DENIED"),
        "status {:?} stdout {} stderr {}",
        output.status,
        text,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!text.contains("secret-credential"));
    assert!(!text.contains("should-not-pass"));
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
}

#[test]
fn installed_harness_versions_are_reported_when_they_differ_from_the_lock() {
    for (name, locked) in [
        ("codex", catalog::CODEX_VERSION),
        ("claude", catalog::CLAUDE_VERSION),
    ] {
        let Some(path) = std::env::var_os("PATH").and_then(|path| {
            std::env::split_paths(&path).find_map(|dir| {
                let candidate = dir.join(name);
                candidate.is_file().then_some(candidate)
            })
        }) else {
            eprintln!("UNVERIFIED {name}: binary not on PATH");
            continue;
        };
        match catalog::describe(&path, locked) {
            Ok(binary) if binary.locked => eprintln!("VERIFIED {name} {}", binary.version),
            Ok(binary) => eprintln!(
                "UNVERIFIED {name}: measured {} locked {locked}",
                binary.version
            ),
            Err(error) => eprintln!("UNVERIFIED {name}: {}", error.message),
        }
    }
    match std::env::var_os("HCTL2_HERDR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("PATH").and_then(|path| {
                std::env::split_paths(&path).find_map(|dir| {
                    let candidate = dir.join("herdr");
                    candidate.is_file().then_some(candidate)
                })
            })
        }) {
        Some(path) => match herdr::locked_protocol(&path) {
            Ok(protocol) => eprintln!("VERIFIED herdr protocol {protocol}"),
            Err(error) => eprintln!("UNVERIFIED herdr: {}", error.message),
        },
        None => eprintln!("UNVERIFIED herdr: binary not on PATH"),
    }
}

#[test]
fn skill_claims_hash_the_skill_file_and_leave_verification_unknown() {
    let dir = std::env::temp_dir().join(format!("hctl2-skills-{}", std::process::id()));
    let skill = dir.join("hctl2-shaping");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(skill.join("SKILL.md"), b"name: shaping\n").unwrap();
    let claims = catalog::skill_claims(&dir).unwrap();
    assert_eq!(claims.len(), 1);
    assert!(claims[0].verification.is_none());
    assert_eq!(claims[0].reference.digest, hash(b"name: shaping\n"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn hctl2_tool_inspect_reports_through_the_adapter_when_installed() {
    let tool = std::env::var_os("HCTL2_TOOL")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("PATH").and_then(|path| {
                std::env::split_paths(&path).find_map(|dir| {
                    let candidate = dir.join("hctl2-tool");
                    candidate.is_file().then_some(candidate)
                })
            })
        });
    let Some(tool) = tool else {
        eprintln!("UNVERIFIED hctl2-tool: binary not on PATH");
        return;
    };
    let repo = std::env::temp_dir().join(format!("hctl2-tool-repo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&repo);
    std::fs::create_dir_all(&repo).unwrap();
    let git = std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo)
        .env("GIT_AUTHOR_NAME", "hctl")
        .env("GIT_AUTHOR_EMAIL", "hctl@localhost")
        .env("GIT_COMMITTER_NAME", "hctl")
        .env("GIT_COMMITTER_EMAIL", "hctl@localhost")
        .status();
    if git.map(|s| !s.success()).unwrap_or(true) {
        eprintln!("UNVERIFIED hctl2-tool: git init failed");
        let _ = std::fs::remove_dir_all(&repo);
        return;
    }
    let output = std::process::Command::new(&tool)
        .args(["repo", "inspect", "--path"])
        .arg(&repo)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", &repo)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout {} stderr {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_ne!(output.stdout, b"");
    eprintln!("VERIFIED hctl2-tool repo inspect");
    let _ = std::fs::remove_dir_all(&repo);
}

#[test]
fn execution_directory_is_outside_the_credential_root() {
    let cred = std::env::temp_dir().join(format!("hctl2-agency-root-{}", std::process::id()));
    std::fs::create_dir_all(&cred).unwrap();
    let exec = confine::execution_dir(&cred, "dispatch").unwrap();
    assert!(!exec.starts_with(cred.canonicalize().unwrap()));
    let _ = std::fs::remove_dir_all(exec.parent().unwrap());
    let _ = std::fs::remove_dir_all(&cred);
}

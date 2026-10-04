use agency::{
    catalog, confine,
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
fn execution_directory_rejects_a_path_that_is_not_an_owned_directory() {
    let cred = std::env::temp_dir().join(format!("hctl2-owned-root-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&cred);
    std::fs::create_dir_all(&cred).unwrap();
    let exec = confine::execution_dir(&cred, "dispatch").unwrap();
    std::fs::remove_dir(&exec).unwrap();
    std::fs::File::create(&exec).unwrap();
    let error = confine::execution_dir(&cred, "dispatch").unwrap_err();
    assert_eq!(error.code, "UNSAFE_ENDPOINT");
    let _ = std::fs::remove_file(&exec);
    let _ = std::fs::remove_dir_all(exec.parent().unwrap());
    let _ = std::fs::remove_dir_all(&cred);
}

#[test]
fn an_allowed_ancestor_is_treated_as_covering_the_credential_root() {
    assert!(confine::allowed_tree_contains_credential(
        std::path::Path::new("/opt"),
        std::path::Path::new("/opt/hctl2-agency")
    ));
    assert!(!confine::allowed_tree_contains_credential(
        std::path::Path::new("/bin"),
        std::path::Path::new("/tmp/agency-test")
    ));
}

#[test]
fn a_backslash_in_the_credential_path_is_still_unreadable() {
    let cred = std::env::temp_dir().join(format!("hctl2-cred\\box-{}", std::process::id()));
    let exec = std::env::temp_dir().join(format!("hctl2-exec-slash-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
    std::fs::create_dir_all(&cred).unwrap();
    std::fs::create_dir_all(&exec).unwrap();
    std::fs::write(cred.join("pair.key"), "secret-credential").unwrap();
    let cred = cred.canonicalize().unwrap();
    let script = format!(
        "if /bin/cat '{}' >/dev/null 2>&1; then echo READ_OK; else echo READ_DENIED; fi",
        cred.join("pair.key").display()
    );
    let program = PathBuf::from("/bin/sh");
    let mut child = confine::command(&program, &["-c".into(), script], &exec, &cred).unwrap();
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
    let _ = std::fs::remove_dir_all(&cred);
    let _ = std::fs::remove_dir_all(&exec);
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

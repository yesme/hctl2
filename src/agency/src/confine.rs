//! Keep an execution process from reading the Agency credential root.
//! A different directory is not the restriction; the child process is.
use agency_proto::{PortError, Result};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

pub fn execution_dir(credential_root: &Path, dispatch: &str) -> Result<PathBuf> {
    let credential_root = credential_root
        .canonicalize()
        .unwrap_or_else(|_| credential_root.to_path_buf());
    let digest = &agency_proto::hash(credential_root.as_os_str().as_encoded_bytes())[..20];
    let parent = PathBuf::from("/tmp").join(format!("hctl2-exec-{digest}"));
    let dir = parent.join(dispatch);
    if dir.starts_with(&credential_root) {
        return Err(PortError::new(
            "EXECUTION_ROOT_UNSAFE",
            "execution directory is inside the credential root",
            "choose_execution_directory",
        ));
    }
    crate::storage::private_dir(&parent)?;
    crate::storage::private_dir(&dir)?;
    Ok(dir)
}

/// Build a command whose view cannot read `credential_root`.
/// The caller sets the program environment after this returns.
pub fn command(
    program: &Path,
    arguments: &[String],
    exec_root: &Path,
    credential_root: &Path,
) -> Result<Command> {
    let credential_root = credential_root
        .canonicalize()
        .unwrap_or_else(|_| credential_root.to_path_buf());
    if exec_root.starts_with(&credential_root) {
        return Err(PortError::new(
            "EXECUTION_ROOT_UNSAFE",
            "execution directory is inside the credential root",
            "choose_execution_directory",
        ));
    }
    if cfg!(target_os = "macos") {
        macos(program, arguments, exec_root, &credential_root)
    } else {
        linux(program, arguments, exec_root, &credential_root)
    }
}

fn macos(
    program: &Path,
    arguments: &[String],
    exec_root: &Path,
    credential_root: &Path,
) -> Result<Command> {
    let profile = exec_root.join("credential.sb");
    let cred = credential_root.display().to_string();
    if cred.contains('"') || cred.contains('\n') {
        return Err(PortError::invalid("credential path cannot be embedded"));
    }
    fs::write(
        &profile,
        format!(
            "(version 1)\n(allow default)\n(deny file-read* (subpath \"{cred}\"))\n(deny file-write* (subpath \"{cred}\"))\n"
        ),
    )?;
    fs::set_permissions(&profile, fs::Permissions::from_mode(0o400))?;
    let mut cmd = Command::new("/usr/bin/sandbox-exec");
    cmd.arg("-f").arg(&profile).arg(program);
    cmd.args(arguments);
    Ok(cmd)
}

fn linux(
    program: &Path,
    arguments: &[String],
    _exec_root: &Path,
    credential_root: &Path,
) -> Result<Command> {
    // The helper applies Landlock, then execs. User namespaces are not available
    // on the Linux CI runners.
    let helper = std::env::var_os("HCTL2_CONFINE_BIN")
        .map(PathBuf::from)
        .or_else(|| std::env::current_exe().ok())
        .ok_or_else(|| {
            PortError::new(
                "CONFINE_HELPER_MISSING",
                "Linux credential confinement helper is not configured",
                "build_agency",
            )
        })?;
    let mut cmd = Command::new(helper);
    cmd.arg("--confine")
        .arg(credential_root)
        .arg("--")
        .arg(program);
    cmd.args(arguments);
    Ok(cmd)
}

pub fn scrub(cmd: &mut Command, exec_root: &Path) {
    cmd.env_clear();
    cmd.env("PATH", "/usr/bin:/bin");
    cmd.env("HOME", exec_root);
    cmd.env("LANG", "C.UTF-8");
    cmd.current_dir(exec_root);
}

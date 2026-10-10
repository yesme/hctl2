//! Keep an execution process from reading the Agency credential root.
//! A different directory is not the restriction; the child process is.
use agency_proto::{PortError, Result};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

fn parent_and_root(credential_root: &Path) -> Result<(PathBuf, PathBuf)> {
    let credential_root = credential_root
        .canonicalize()
        .map_err(|_| unresolved_credential())?;
    let digest = &agency_proto::hash(credential_root.as_os_str().as_encoded_bytes())[..20];
    Ok((
        PathBuf::from("/tmp").join(format!("hctl2-exec-{digest}")),
        credential_root,
    ))
}

/// The directory every execution directory for this credential root sits in. It is
/// shared: a running Herdr keeps its own state directory beside them.
pub fn execution_parent(credential_root: &Path) -> Result<PathBuf> {
    Ok(parent_and_root(credential_root)?.0)
}

pub fn execution_dir(credential_root: &Path, dispatch: &str) -> Result<PathBuf> {
    let (parent, credential_root) = parent_and_root(credential_root)?;
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

/// Drop a dispatch's execution directory once its process is reaped, then the
/// parent if nothing else is in it. A concurrent dispatch or a Herdr state
/// directory keeps the parent alive, because `remove_dir` needs it empty.
/// Something that is not a directory under the parent is left alone: `private_dir`
/// refused to run in it, and this does not delete what it did not create.
///
/// The parent is one the caller resolved while the credential root still existed.
/// Re-resolving it here would fail once that root is gone, which is exactly when a
/// shutting-down Agency reaps its last child.
pub(crate) fn release_execution_dir(parent: &Path, dispatch: &str) {
    // `remove_dir_all` takes a path built from a string: only ever one named child
    // of the parent, never a path that climbs out of it.
    let mut components = Path::new(dispatch).components();
    let single = matches!(components.next(), Some(std::path::Component::Normal(_)))
        && components.next().is_none();
    if !single {
        return;
    }
    let _ = fs::remove_dir_all(parent.join(dispatch));
    let _ = fs::remove_dir(parent);
}

/// Build a command whose view cannot read `credential_root`.
/// The caller sets the program environment after this returns.
pub fn command(
    program: &Path,
    arguments: &[String],
    exec_root: &Path,
    credential_root: &Path,
) -> Result<Command> {
    let credential_root = credential_root.canonicalize().map_err(|_| {
        PortError::new(
            "CREDENTIAL_ROOT_UNRESOLVED",
            "credential root cannot be canonicalized",
            "choose_credential_root",
        )
    })?;
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

/// Scheme strings treat `\\` as an escape. A raw backslash would deny a different path.
fn scheme_literal(path: &str) -> Result<String> {
    if path.contains('\n') || path.contains('\0') {
        return Err(PortError::invalid("credential path cannot be embedded"));
    }
    Ok(path.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Landlock allows a whole directory tree. An ancestor of the credential root
/// cannot be allowed, because the credential directory cannot be carved back out.
/// Linux serve refuses a credential root that sits inside a Landlock allow directory.
/// macOS denies the credential root by path, so a root under `/opt` stays usable.
/// A directory that does not exist yet is judged from its existing ancestor.
/// A canonicalize failure is an error; the raw path is not used.
pub fn refuse_covered_credential_root(credential_root: &Path) -> Result<()> {
    if !cfg!(target_os = "linux") {
        return Ok(());
    }
    let credential_root = intended_path(credential_root)?;
    for dir in [
        "/bin", "/usr", "/lib", "/lib64", "/etc", "/dev", "/proc", "/opt",
    ] {
        let allow = Path::new(dir);
        if allow.is_dir() && allowed_tree_contains_credential(allow, &credential_root) {
            return Err(PortError::new(
                "CREDENTIAL_ROOT_COVERED",
                format!("credential root is inside the allowed directory {dir}"),
                "move_credential_root",
            ));
        }
    }
    Ok(())
}

pub fn allowed_tree_contains_credential(allow: &Path, credential: &Path) -> bool {
    credential.starts_with(allow) || allow.starts_with(credential)
}

fn intended_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return path.canonicalize().map_err(|_| unresolved_credential());
    }
    let mut pending = Vec::new();
    let mut cursor = path;
    while !cursor.exists() {
        let Some(name) = cursor.file_name() else {
            return Err(unresolved_credential());
        };
        pending.push(name.to_os_string());
        match cursor.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => cursor = parent,
            _ => cursor = Path::new("."),
        }
    }
    let mut resolved = cursor.canonicalize().map_err(|_| unresolved_credential())?;
    for name in pending.iter().rev() {
        resolved.push(name);
    }
    Ok(resolved)
}

fn unresolved_credential() -> PortError {
    PortError::new(
        "CREDENTIAL_ROOT_UNRESOLVED",
        "credential root cannot be canonicalized",
        "choose_credential_root",
    )
}

fn macos(
    program: &Path,
    arguments: &[String],
    exec_root: &Path,
    credential_root: &Path,
) -> Result<Command> {
    let profile = exec_root.join("credential.sb");
    let cred = scheme_literal(&credential_root.display().to_string())?;
    // A previous start leaves this file mode 0400. Unlink uses the directory, not that mode.
    let _ = fs::remove_file(&profile);
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
    // Spawn resolves a relative program path against current_dir. Callers then
    // move the child into the execution directory, so the helper must be absolute.
    let helper = std::fs::canonicalize(&helper).map_err(|error| {
        PortError::new(
            "CONFINE_HELPER_MISSING",
            format!("{}: {error}", helper.display()),
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

/// A write-capable app-server has the physical credential boundary of Herdr
/// descendants. Keep its native model login; never grant the whole HOME.
pub(crate) fn harness_command(
    program: &Path,
    arguments: &[String],
    cwd: &Path,
    state: &Path,
    socket: &Path,
    credential_root: &Path,
) -> Result<Command> {
    let mut command = command(program, arguments, state, credential_root)?;
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| PortError::invalid("native harness HOME missing"))?;
    let codex_home = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".codex"));
    let execution_parent = cwd
        .parent()
        .ok_or_else(|| PortError::invalid("session parent missing"))?;
    let socket_parent = socket
        .parent()
        .ok_or_else(|| PortError::invalid("socket parent missing"))?;
    for path in [&codex_home, execution_parent, socket_parent, state] {
        if allowed_tree_contains_credential(&intended_path(path)?, &credential_root.canonicalize()?)
        {
            return Err(PortError::new(
                "CREDENTIAL_ROOT_COVERED",
                "native harness allowance overlaps credentials",
                "move_credential_root",
            ));
        }
    }
    let temp = state.join("tmp");
    crate::storage::private_dir(&temp)?;
    protect_native_credentials(state, credential_root)?;
    if cfg!(target_os = "macos") {
        let profile = state.join("credential.sb");
        let rules = fs::read_to_string(&profile)? + &worktree_git_denial(cwd)?;
        fs::remove_file(&profile)?;
        fs::write(&profile, rules)?;
        fs::set_permissions(profile, fs::Permissions::from_mode(0o400))?;
    }
    command
        .env_clear()
        .env("HOME", &home)
        .env("CODEX_HOME", &codex_home)
        .env(
            "PATH",
            std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into()),
        )
        .env("LANG", "C.UTF-8")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("TMPDIR", &temp)
        .env(
            "HCTL2_CONFINE_ALLOW",
            format!(
                "{}\n{}\n{}\n{}",
                codex_home.display(),
                execution_parent.display(),
                socket_parent.display(),
                state.display()
            ),
        )
        .env(
            "HCTL2_CONFINE_READ",
            program
                .canonicalize()?
                .parent()
                .ok_or_else(|| PortError::invalid("harness parent missing"))?,
        )
        .current_dir(cwd);
    command.args(["-c", "sandbox_mode=\"danger-full-access\""]);
    Ok(command)
}

pub(crate) fn protect_native_credentials(state: &Path, credential_root: &Path) -> Result<()> {
    if cfg!(target_os = "macos") {
        let profile = state.join("credential.sb");
        let mut rules = format!(
            "(version 1)\n(allow default)\n(deny file-read* (subpath \"{}\"))\n(deny file-write* (subpath \"{}\"))\n",
            scheme_literal(&credential_root.canonicalize()?.display().to_string())?,
            scheme_literal(&credential_root.canonicalize()?.display().to_string())?
        );
        for path in sensitive_paths() {
            rules.push_str(&format!(
                "(deny file-read* (subpath \"{}\"))\n",
                scheme_literal(&path.display().to_string())?
            ));
        }
        let _ = fs::remove_file(&profile);
        fs::write(&profile, rules)?;
        fs::set_permissions(profile, fs::Permissions::from_mode(0o400))?;
    }
    Ok(())
}

/// A materialized checkout shares the source Repo's Git configuration. Keep it
/// outside native harnesses; the Agency/toolbox reads Git identity and seals.
pub(crate) fn append_git_denials(state: &Path, paths: &[PathBuf]) -> Result<()> {
    let profile = state.join("credential.sb");
    let mut rules = fs::read_to_string(&profile)?;
    for path in paths {
        rules.push_str(&git_denial(path)?);
    }
    fs::remove_file(&profile)?;
    fs::write(&profile, rules)?;
    fs::set_permissions(profile, fs::Permissions::from_mode(0o400))?;
    Ok(())
}

pub(crate) fn worktree_git_denial(cwd: &Path) -> Result<String> {
    let common = crate::write::git_common_dir(cwd)?;
    git_denial(&common)
}

fn git_denial(common: &Path) -> Result<String> {
    let path = scheme_literal(&common.display().to_string())?;
    Ok(format!(
        "(deny file-read* (subpath \"{path}\"))\n(deny file-write* (subpath \"{path}\"))\n"
    ))
}

pub(crate) fn sensitive_paths() -> Vec<PathBuf> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let mut paths = vec![
        home.join(".config/gh"),
        home.join(".ssh"),
        home.join(".git-credentials"),
        home.join("Library/Keychains"),
        home.join(".local/share/keyrings"),
        PathBuf::from("/Library/Keychains"),
    ];
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        paths.push(PathBuf::from(xdg).join("gh"));
    }
    if let Some(gh) = std::env::var_os("GH_CONFIG_DIR") {
        paths.push(PathBuf::from(gh));
    }
    paths
}

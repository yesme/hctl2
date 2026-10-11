//! Linux-only helper. The `landlock` crate hides the credential root.
//! GitHub-hosted runners reject unprivileged `unshare` (`uid_map` EPERM).
//! The workspace forbids `unsafe_code`, so the system calls stay in that crate.
use landlock::{
    ABI, Access, AccessFs, PathBeneath, PathFd, Ruleset, RulesetAttr, RulesetCreatedAttr,
    RulesetStatus,
};
use std::{
    env,
    io::{self, Error},
    os::unix::process::CommandExt,
    path::Path,
    process::Command,
};

pub fn run() -> i32 {
    match exec() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("confine: {error}");
            1
        }
    }
}

fn exec() -> io::Result<()> {
    let mut args = env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--confine")) {
        return Err(Error::other("missing --confine"));
    }
    let credential = args
        .next()
        .ok_or_else(|| Error::other("missing credential root"))?;
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--")) {
        return Err(Error::other("missing command separator"));
    }
    let program = args.next().ok_or_else(|| Error::other("missing program"))?;
    let rest: Vec<_> = args.collect();
    let credential = Path::new(&credential).canonicalize()?;
    let work = env::current_dir()?.canonicalize()?;
    if work.starts_with(&credential) || credential.starts_with(&work) {
        return Err(Error::other("work directory overlaps the credential root"));
    }
    restrict(&work, &credential)?;
    Err(Command::new(program).args(rest).exec())
}

fn restrict(work: &Path, credential: &Path) -> io::Result<()> {
    let abi = ABI::V1;
    let read_exec = AccessFs::from_read(abi);
    let full = AccessFs::from_all(abi);
    // openpty writes /dev/ptmx. /dev/pts is a separate devpts mount, so a rule
    // on /dev does not cover the slave. Write and character-device creation
    // stay on those two trees; unlink and regular-file creation stay denied.
    let device = read_exec | AccessFs::WriteFile | AccessFs::MakeChar;
    let mut ruleset = Ruleset::default()
        .handle_access(full)
        .map_err(io_err)?
        .create()
        .map_err(io_err)?;
    ruleset = add(ruleset, work, full)?;
    if let Some(extra) = env::var_os("HCTL2_CONFINE_ALLOW") {
        for item in extra.to_string_lossy().split('\n') {
            if item.is_empty() {
                continue;
            }
            let path = Path::new(item);
            if !path.exists() {
                continue;
            }
            let path = path.canonicalize().map_err(|error| {
                Error::other(format!(
                    "allowed path {item} cannot be canonicalized: {error}"
                ))
            })?;
            if agency::confine::allowed_tree_contains_credential(&path, credential) {
                return Err(Error::other(format!(
                    "credential root is inside allowed path {}",
                    path.display()
                )));
            }
            let access = if path.is_dir() { full } else { read_exec };
            ruleset = add(ruleset, &path, access)?;
        }
    }
    if let Some(extra) = env::var_os("HCTL2_CONFINE_READ") {
        for item in extra.to_string_lossy().split('\n') {
            if item.is_empty() {
                continue;
            }
            let path = Path::new(item);
            if !path.exists() {
                continue;
            }
            let path = path.canonicalize().map_err(|error| {
                Error::other(format!(
                    "allowed path {item} cannot be canonicalized: {error}"
                ))
            })?;
            if agency::confine::allowed_tree_contains_credential(&path, credential) {
                return Err(Error::other(format!(
                    "credential root is inside allowed path {}",
                    path.display()
                )));
            }
            ruleset = add(ruleset, &path, read_exec)?;
        }
    }
    for (dir, access) in [
        ("/bin", read_exec),
        ("/usr", read_exec),
        ("/lib", read_exec),
        ("/lib64", read_exec),
        ("/etc", read_exec),
        ("/dev", device),
        ("/dev/pts", device),
        ("/proc", read_exec),
        ("/opt", read_exec),
    ] {
        let path = Path::new(dir);
        if !path.is_dir() {
            continue;
        }
        let path = path.canonicalize().map_err(|error| {
            Error::other(format!(
                "allowed path {dir} cannot be canonicalized: {error}"
            ))
        })?;
        if agency::confine::allowed_tree_contains_credential(&path, credential) {
            return Err(Error::other(format!(
                "credential root is inside allowed path {}",
                path.display()
            )));
        }
        ruleset = add(ruleset, &path, access)?;
    }
    // Ubuntu's /etc/resolv.conf points outside /etc into /run/systemd/resolve.
    // Grant that exact system file, never /run or the user's keyring directory.
    for system_file in ["/etc/resolv.conf", "/etc/hosts", "/etc/nsswitch.conf"] {
        let file = Path::new(system_file);
        if file.exists() {
            let file = file.canonicalize()?;
            if agency::confine::allowed_tree_contains_credential(&file, credential) {
                return Err(Error::other(
                    "system resolver file overlaps credential root",
                ));
            }
            ruleset = add(ruleset, &file, read_exec)?;
        }
    }
    let status = ruleset.restrict_self().map_err(io_err)?;
    if !matches!(status.ruleset, RulesetStatus::FullyEnforced) {
        return Err(Error::other("landlock was not fully enforced"));
    }
    Ok(())
}

fn add(
    ruleset: landlock::RulesetCreated,
    path: &Path,
    access: landlock::BitFlags<AccessFs>,
) -> io::Result<landlock::RulesetCreated> {
    // A file rule cannot carry ReadDir/MakeDir or other directory-only rights.
    // The native harness's state includes ~/.claude.json as a single file.
    let access = if path.is_file() {
        access & (AccessFs::Execute | AccessFs::ReadFile | AccessFs::WriteFile)
    } else {
        access
    };
    let fd = PathFd::new(path).map_err(io_err)?;
    ruleset
        .add_rule(PathBeneath::new(fd, access))
        .map_err(io_err)
}

fn io_err(error: impl std::fmt::Display) -> Error {
    Error::other(error.to_string())
}

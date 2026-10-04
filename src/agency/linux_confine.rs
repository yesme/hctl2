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
    if work.starts_with(&credential) {
        return Err(Error::other("work directory is inside the credential root"));
    }
    restrict(&work)?;
    Err(Command::new(program).args(rest).exec())
}

fn restrict(work: &Path) -> io::Result<()> {
    let abi = ABI::V1;
    let read_exec = AccessFs::from_read(abi);
    let full = AccessFs::from_all(abi);
    let mut ruleset = Ruleset::default()
        .handle_access(full)
        .map_err(io_err)?
        .create()
        .map_err(io_err)?;
    ruleset = add(ruleset, work, full)?;
    for dir in [
        "/bin", "/usr", "/lib", "/lib64", "/etc", "/dev", "/proc", "/opt",
    ] {
        let path = Path::new(dir);
        if path.is_dir() {
            ruleset = add(ruleset, path, read_exec)?;
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
    let fd = PathFd::new(path).map_err(io_err)?;
    ruleset
        .add_rule(PathBeneath::new(fd, access))
        .map_err(io_err)
}

fn io_err(error: impl std::fmt::Display) -> Error {
    Error::other(error.to_string())
}

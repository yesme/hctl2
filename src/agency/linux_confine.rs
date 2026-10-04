//! Linux-only helper. Landlock hides the credential root without a user namespace.
//! GitHub-hosted runners reject unprivileged `unshare` (`uid_map` EPERM).
use std::{
    env, fs,
    io::{self, Error},
    mem::size_of,
    os::{fd::AsRawFd, unix::process::CommandExt},
    path::Path,
    process::Command,
};

const CREATE: i64 = 444;
const ADD_RULE: i64 = 445;
const RESTRICT: i64 = 446;
const RULE_PATH_BENEATH: i64 = 1;
const READ_EXEC: u64 = 1 | (1 << 2) | (1 << 3);
const READ_WRITE: u64 = (1 << 13) - 1;

#[repr(C)]
struct RulesetAttr {
    handled_access_fs: u64,
}
#[repr(C, packed)]
struct PathBeneath {
    allowed_access: u64,
    parent_fd: i32,
}

unsafe extern "C" {
    fn syscall(num: i64, ...) -> i64;
}

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
    let error = Command::new(program).args(rest).exec();
    Err(error)
}

fn restrict(work: &Path) -> io::Result<()> {
    let attr = RulesetAttr {
        handled_access_fs: READ_WRITE,
    };
    let ruleset = unsafe {
        syscall(
            CREATE,
            &attr as *const RulesetAttr as usize,
            size_of::<RulesetAttr>(),
            0usize,
        )
    };
    if ruleset < 0 {
        return Err(Error::last_os_error());
    }
    let mut held = Vec::new();
    for (path, access) in allow_paths(work) {
        if !path.exists() {
            continue;
        }
        let file = fs::File::open(&path)?;
        let rule = PathBeneath {
            allowed_access: access,
            parent_fd: file.as_raw_fd(),
        };
        let added = unsafe {
            syscall(
                ADD_RULE,
                ruleset,
                RULE_PATH_BENEATH,
                &rule as *const PathBeneath as usize,
                0usize,
            )
        };
        if added < 0 {
            return Err(Error::last_os_error());
        }
        held.push(file);
    }
    let restricted = unsafe { syscall(RESTRICT, ruleset, 0usize) };
    if restricted < 0 {
        return Err(Error::last_os_error());
    }
    drop(held);
    Ok(())
}

fn allow_paths(work: &Path) -> Vec<(std::path::PathBuf, u64)> {
    let mut paths = vec![(work.to_path_buf(), READ_WRITE)];
    for dir in [
        "/bin", "/usr", "/lib", "/lib64", "/etc", "/dev", "/proc", "/opt",
    ] {
        paths.push((dir.into(), READ_EXEC));
    }
    paths
}

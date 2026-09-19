//! Shared host Git discovery and process hygiene; no repository or governance operations.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

const MINIMUM_GIT_MAJOR: u64 = 2;
const MINIMUM_GIT_MINOR: u64 = 39;

pub fn sanitized_command(executable: &Path) -> Command {
    let mut command = Command::new(executable);
    for name in [
        "GIT_DIR",
        "GIT_COMMON_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_NAMESPACE",
        "GIT_CONFIG_COUNT",
        "GIT_CONFIG_PARAMETERS",
    ] {
        command.env_remove(name);
    }
    for (name, _) in std::env::vars_os() {
        let name_text = name.to_string_lossy();
        if name_text.starts_with("GIT_CONFIG_KEY_") || name_text.starts_with("GIT_CONFIG_VALUE_") {
            command.env_remove(name);
        }
    }
    command
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("LC_ALL", "C");
    command
}

pub fn resolve_executable(requested: &OsStr) -> Option<PathBuf> {
    let requested_path = Path::new(requested);
    if requested_path.is_absolute() || requested_path.components().count() > 1 {
        return requested_path
            .canonicalize()
            .ok()
            .filter(|path| path.is_file());
    }

    let search_path = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&search_path) {
        let candidate = directory.join(requested_path);
        if let Ok(canonical) = candidate.canonicalize()
            && canonical.is_file()
        {
            return Some(canonical);
        }
        #[cfg(windows)]
        {
            let candidate = directory.join(format!("{}.exe", requested.to_string_lossy()));
            if let Ok(canonical) = candidate.canonicalize()
                && canonical.is_file()
            {
                return Some(canonical);
            }
        }
    }
    None
}

pub fn parse_version(banner: &str) -> Option<(u64, u64, String)> {
    let version = banner
        .trim()
        .strip_prefix("git version ")?
        .split_whitespace()
        .next()?;
    let mut components = version.split('.');
    let major = components.next()?.parse().ok()?;
    let minor = components.next()?.parse().ok()?;
    Some((major, minor, version.to_owned()))
}

pub const fn version_supported(major: u64, minor: u64) -> bool {
    major > MINIMUM_GIT_MAJOR || (major == MINIMUM_GIT_MAJOR && minor >= MINIMUM_GIT_MINOR)
}

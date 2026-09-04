use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use hctl2_foundation::ExclusiveFileLock;
use serde_json::json;

use crate::ToolError;

/// One process-scoped lock for mutations beneath a Git common directory.
///
/// P1 commands keep this guard for one invocation. P2 can keep the same guard
/// alive for a longer control-owned operation without changing the lock path.
pub struct SiteLock {
    _guard: ExclusiveFileLock,
    path: PathBuf,
    filesystem: String,
}

impl SiteLock {
    /// Acquires `<git-common-dir>/hctl2/lock` without waiting.
    ///
    /// # Errors
    ///
    /// Returns a stable tool error for a remote filesystem, a live holder, or
    /// an inaccessible lock directory.
    pub fn acquire(
        common_dir: &Path,
        operation: &str,
        change_set_ref: Option<&str>,
    ) -> Result<Self, ToolError> {
        let filesystem = filesystem_type(common_dir)?;
        if is_nonlocal_filesystem(&filesystem) {
            return Err(ToolError::new(
                "HCTL2_TOOL_NONLOCAL_SITE_FILESYSTEM",
                format!(
                    "refusing a site lock on non-local filesystem {filesystem}: {}",
                    common_dir.display()
                ),
            ));
        }

        let lock_directory = common_dir.join("hctl2");
        fs::create_dir_all(&lock_directory).map_err(|error| {
            ToolError::new(
                "HCTL2_TOOL_SITE_LOCK_UNAVAILABLE",
                format!(
                    "cannot create site lock directory {}: {error}",
                    lock_directory.display()
                ),
            )
        })?;
        let path = lock_directory.join("lock");
        let mut guard = match ExclusiveFileLock::try_acquire(&path) {
            Ok(guard) => guard,
            Err(error) if error.is_lock_contended() => {
                let holder = fs::read_to_string(&path)
                    .ok()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "holder information unavailable".to_owned());
                return Err(ToolError::new(
                    "HCTL2_TOOL_SITE_BUSY",
                    format!("site lock {} is held; holder: {holder}", path.display()),
                ));
            }
            Err(error) => {
                return Err(ToolError::new(
                    "HCTL2_TOOL_SITE_LOCK_UNAVAILABLE",
                    format!("cannot acquire site lock {}: {error}", path.display()),
                ));
            }
        };
        let holder = serde_json::to_vec(&json!({
            "pid": std::process::id(),
            "operation": operation,
            "change_set_ref": change_set_ref,
        }))
        .map_err(|error| ToolError::new("HCTL2_TOOL_SERIALIZATION_FAILED", error.to_string()))?;
        guard.write_holder_info(&holder).map_err(|error| {
            ToolError::new(
                "HCTL2_TOOL_SITE_LOCK_UNAVAILABLE",
                format!(
                    "cannot write holder information to {}: {error}",
                    path.display()
                ),
            )
        })?;

        Ok(Self {
            _guard: guard,
            path,
            filesystem,
        })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn filesystem(&self) -> &str {
        &self.filesystem
    }
}

fn filesystem_type(path: &Path) -> Result<String, ToolError> {
    #[cfg(target_os = "macos")]
    return macos_filesystem_type(path);

    #[cfg(target_os = "linux")]
    return linux_filesystem_type(path);

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    Err(ToolError::new(
        "HCTL2_TOOL_FILESYSTEM_UNSUPPORTED",
        "site filesystem detection is not implemented on this platform",
    ))
}

#[cfg(target_os = "linux")]
fn linux_filesystem_type(path: &Path) -> Result<String, ToolError> {
    let mut command = Command::new("stat");
    command
        .args(["-f", "-c", "%T"])
        .arg(path)
        .env("LC_ALL", "C");
    let output = command.output().map_err(|error| {
        ToolError::new(
            "HCTL2_TOOL_FILESYSTEM_UNREADABLE",
            format!("cannot execute stat for {}: {error}", path.display()),
        )
    })?;
    if !output.status.success() {
        return Err(ToolError::new(
            "HCTL2_TOOL_FILESYSTEM_UNREADABLE",
            format!(
                "cannot identify filesystem for {}: {}",
                path.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    let value = String::from_utf8(output.stdout).map_err(|_| {
        ToolError::new(
            "HCTL2_TOOL_FILESYSTEM_UNREADABLE",
            "stat returned a non-UTF-8 filesystem name",
        )
    })?;
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty() {
        Err(ToolError::new(
            "HCTL2_TOOL_FILESYSTEM_UNREADABLE",
            "stat returned an empty filesystem name",
        ))
    } else {
        Ok(value)
    }
}

#[cfg(target_os = "macos")]
fn macos_filesystem_type(path: &Path) -> Result<String, ToolError> {
    let path = path.canonicalize().map_err(|error| {
        ToolError::new(
            "HCTL2_TOOL_FILESYSTEM_UNREADABLE",
            format!("cannot canonicalize {}: {error}", path.display()),
        )
    })?;
    let output = Command::new("mount")
        .env("LC_ALL", "C")
        .output()
        .map_err(|error| {
            ToolError::new(
                "HCTL2_TOOL_FILESYSTEM_UNREADABLE",
                format!("cannot read the mount table: {error}"),
            )
        })?;
    if !output.status.success() {
        return Err(ToolError::new(
            "HCTL2_TOOL_FILESYSTEM_UNREADABLE",
            format!(
                "cannot read the mount table: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    let table = String::from_utf8(output.stdout).map_err(|_| {
        ToolError::new(
            "HCTL2_TOOL_FILESYSTEM_UNREADABLE",
            "mount returned non-UTF-8 output",
        )
    })?;
    let mut best: Option<(usize, String)> = None;
    for line in table.lines() {
        let Some((placement, options)) = line.rsplit_once(" (") else {
            continue;
        };
        let Some((_, mountpoint)) = placement.split_once(" on ") else {
            continue;
        };
        let mountpoint = PathBuf::from(unescape_mount_path(mountpoint));
        if !path.starts_with(&mountpoint) {
            continue;
        }
        let filesystem = options
            .trim_end_matches(')')
            .split(',')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        let specificity = mountpoint.components().count();
        if !filesystem.is_empty()
            && best
                .as_ref()
                .is_none_or(|(current, _)| specificity > *current)
        {
            best = Some((specificity, filesystem));
        }
    }
    best.map(|(_, filesystem)| filesystem).ok_or_else(|| {
        ToolError::new(
            "HCTL2_TOOL_FILESYSTEM_UNREADABLE",
            format!("no mount-table entry contains {}", path.display()),
        )
    })
}

#[cfg(target_os = "macos")]
fn unescape_mount_path(value: &str) -> String {
    value
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\134", "\\")
}

fn is_nonlocal_filesystem(filesystem: &str) -> bool {
    let normalized = filesystem.to_ascii_lowercase();
    ["nfs", "cifs", "smbfs", "sshfs", "fuse.sshfs", "9p", "afs"]
        .iter()
        .any(|name| normalized == *name || normalized.starts_with(&format!("{name}.")))
}

#[cfg(test)]
mod tests {
    use super::is_nonlocal_filesystem;

    #[test]
    fn classifies_known_remote_filesystems() {
        for filesystem in ["nfs", "nfs.v4", "cifs", "smbfs", "fuse.sshfs", "9p"] {
            assert!(is_nonlocal_filesystem(filesystem), "{filesystem}");
        }
        for filesystem in ["apfs", "ext2/ext3", "xfs", "tmpfs", "overlayfs"] {
            assert!(!is_nonlocal_filesystem(filesystem), "{filesystem}");
        }
    }
}

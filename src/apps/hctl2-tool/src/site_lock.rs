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
        // Inspect the actual lock directory, which may itself be a mount or symlink.
        let filesystem = filesystem_type(&lock_directory)?;
        if is_nonlocal_filesystem(&filesystem) {
            return Err(ToolError::not_established(
                "HCTL2_TOOL_NONLOCAL_SITE_FILESYSTEM",
                "site locks require a local filesystem; remote and FUSE filesystems are unsupported",
            ).with_details(json!({ "filesystem": filesystem, "path": lock_directory })));
        }
        let path = lock_directory.join("lock");
        match fs::symlink_metadata(&path) {
            Ok(metadata) if !metadata.is_file() => {
                return Err(ToolError::not_established(
                    "HCTL2_TOOL_SITE_LOCK_UNAVAILABLE",
                    "site lock must be a regular file, not a symlink or directory",
                ));
            }
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                return Err(ToolError::new(
                    "HCTL2_TOOL_SITE_LOCK_UNAVAILABLE",
                    error.to_string(),
                ));
            }
            _ => {}
        }
        let mut guard = match ExclusiveFileLock::try_acquire(&path) {
            Ok(guard) => guard,
            Err(error) if error.is_lock_contended() => {
                let holder_raw = fs::read_to_string(&path).ok();
                let holder = holder_raw
                    .as_deref()
                    .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok());
                return Err(ToolError::not_established(
                    "HCTL2_TOOL_SITE_BUSY",
                    format!("site lock {} is held", path.display()),
                )
                .with_details(
                    json!({ "path": path, "holder": holder, "holder_raw": holder_raw }),
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
    let output = Command::new("/sbin/mount")
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
    parse_mount_table(&path, &table).ok_or_else(|| {
        ToolError::new(
            "HCTL2_TOOL_FILESYSTEM_UNREADABLE",
            format!("no mount-table entry contains {}", path.display()),
        )
    })
}

#[cfg(any(target_os = "macos", test))]
fn parse_mount_table(path: &Path, table: &str) -> Option<String> {
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
    best.map(|(_, filesystem)| filesystem)
}

#[cfg(any(target_os = "macos", test))]
fn unescape_mount_path(value: &str) -> String {
    value
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\134", "\\")
}

fn is_nonlocal_filesystem(filesystem: &str) -> bool {
    let normalized = filesystem.to_ascii_lowercase();
    // coreutils versions report Linux FUSE mounts as fuseblk or fuse, including sshfs.
    // Reject FUSE conservatively: its type alone cannot establish locality.
    [
        "nfs", "cifs", "smb", "smb2", "smbfs", "afs", "ceph", "v9fs", "9p", "afpfs", "webdav",
        "sshfs", "fuse", "fuseblk", "osxfuse", "macfuse",
    ]
    .iter()
    .any(|name| normalized == *name || normalized.starts_with(&format!("{name}.")))
}

#[cfg(test)]
mod tests {
    use super::{is_nonlocal_filesystem, parse_mount_table};
    use std::path::Path;

    #[test]
    fn classifies_known_remote_filesystems() {
        for filesystem in [
            "nfs",
            "nfs.v4",
            "cifs",
            "smb",
            "smb2",
            "smbfs",
            "afs",
            "ceph",
            "v9fs",
            "afpfs",
            "webdav",
            "fuseblk",
            "fuse.sshfs",
            "macfuse",
            "9p",
        ] {
            assert!(is_nonlocal_filesystem(filesystem), "{filesystem}");
        }
        for filesystem in ["apfs", "ext2/ext3", "xfs", "tmpfs", "overlayfs"] {
            assert!(!is_nonlocal_filesystem(filesystem), "{filesystem}");
        }
    }

    #[test]
    fn mount_table_selects_the_longest_matching_mountpoint() {
        let table = "/dev/disk1 on / (apfs, local)\nserver on /Volumes/team (nfs, nodev)\n/dev/disk2 on /Volumes/team/local (apfs, local)\nserver on /Volumes/a\\040b (smbfs, nodev)\n";
        assert_eq!(
            parse_mount_table(Path::new("/Volumes/team/repo"), table).as_deref(),
            Some("nfs")
        );
        assert_eq!(
            parse_mount_table(Path::new("/Volumes/team/local/repo"), table).as_deref(),
            Some("apfs")
        );
        assert_eq!(
            parse_mount_table(Path::new("/Volumes/teammate/repo"), table).as_deref(),
            Some("apfs")
        );
        assert_eq!(
            parse_mount_table(Path::new("/Volumes/a b/repo"), table).as_deref(),
            Some("smbfs")
        );
    }
}

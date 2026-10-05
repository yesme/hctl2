use std::fs;
use std::path::{Path, PathBuf};

use foundation::ExclusiveFileLock;
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
        Self::acquire_record(
            common_dir,
            json!({
                "pid": std::process::id(), "operation": operation, "change_set_ref": change_set_ref,
            }),
        )
    }

    pub(crate) fn acquire_for_intent(
        common_dir: &Path,
        operation: &str,
        intent_digest: &str,
    ) -> Result<Self, ToolError> {
        Self::acquire_record(
            common_dir,
            json!({
                "pid": std::process::id(), "operation": operation, "intent_digest": intent_digest,
            }),
        )
    }

    fn acquire_record(common_dir: &Path, holder: serde_json::Value) -> Result<Self, ToolError> {
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
        let holder = serde_json::to_vec(&holder).map_err(|error| {
            ToolError::new("HCTL2_TOOL_SERIALIZATION_FAILED", error.to_string())
        })?;
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

/// Filesystem type behind `path`, as reported by the kernel.
///
/// The lock needs the type only to reject remote and FUSE filesystems; asking the
/// kernel is exact, where parsing `mount`/`stat` output had to guess which entry a
/// path belonged to.
fn filesystem_type(path: &Path) -> Result<String, ToolError> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    return native_filesystem_type(path);

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    Err(ToolError::new(
        "HCTL2_TOOL_FILESYSTEM_UNSUPPORTED",
        "site filesystem detection is not implemented on this platform",
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn native_filesystem_type(path: &Path) -> Result<String, ToolError> {
    let stat = rustix::fs::statfs(path).map_err(|error| {
        ToolError::new(
            "HCTL2_TOOL_FILESYSTEM_UNREADABLE",
            format!("cannot identify filesystem for {}: {error}", path.display()),
        )
    })?;

    // Linux reports a magic number; macOS reports the type name directly.
    #[cfg(target_os = "linux")]
    let name = linux_filesystem_name(stat.f_type as u64);

    #[cfg(target_os = "macos")]
    let name = {
        // `f_fstypename` is a fixed-size C string; read it without `unsafe`.
        let bytes: Vec<u8> = stat
            .f_fstypename
            .iter()
            .take_while(|byte| **byte != 0)
            .map(|byte| *byte as u8)
            .collect();
        String::from_utf8_lossy(&bytes).trim().to_ascii_lowercase()
    };

    if name.is_empty() {
        return Err(ToolError::new(
            "HCTL2_TOOL_FILESYSTEM_UNREADABLE",
            "statfs returned an empty filesystem name",
        ));
    }
    Ok(name)
}

/// Names the filesystem behind a Linux `statfs` magic so the conservative rejection
/// list keeps matching, spelled the way `stat -f -c %T` used to spell it.
///
/// Unknown magics are named `unknown-<magic>`: the list only names non-local
/// filesystems, so an unknown one is accepted — the same verdict as before.
#[cfg(any(target_os = "linux", test))]
fn linux_filesystem_name(magic: u64) -> String {
    match magic {
        0x6969 => "nfs",
        0x517b => "smb",
        0xff53_4d42 => "cifs",
        0xfe53_4d42 => "smb2",
        0x5346_414f => "afs",
        0x00c3_6400 => "ceph",
        0x0102_1997 => "9p",
        // FUSE and every subtype, including fuseblk and sshfs.
        0x6573_5546 => "fuse",
        // Local filesystems are named too, so receipts stay readable.
        0xef53 => "ext2/ext3",
        0x5846_5342 => "xfs",
        0x9123_683e => "btrfs",
        0x0102_1994 => "tmpfs",
        0x794c_7630 => "overlayfs",
        0x7371_7368 => "squashfs",
        0x8584_58f6 => "ramfs",
        0x4d44 => "msdos",
        other => return format!("unknown-{other:#x}"),
    }
    .to_owned()
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
    use super::{filesystem_type, is_nonlocal_filesystem, linux_filesystem_name};

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
    fn names_linux_filesystem_magics_so_the_rejection_list_still_matches() {
        for (magic, expected) in [
            (0x6969_u64, "nfs"),
            (0x517b, "smb"),
            (0xff53_4d42, "cifs"),
            (0xfe53_4d42, "smb2"),
            (0x5346_414f, "afs"),
            (0x00c3_6400, "ceph"),
            (0x0102_1997, "9p"),
            (0x6573_5546, "fuse"),
        ] {
            let name = linux_filesystem_name(magic);
            assert_eq!(name, expected, "{magic:#x}");
            assert!(is_nonlocal_filesystem(&name), "{magic:#x}");
        }
        // Local filesystems stay accepted; so does a magic nobody named, because the
        // rejection list only covers non-local filesystems.
        for magic in [
            0xef53_u64,
            0x5846_5342,
            0x9123_683e,
            0x0102_1994,
            0x794c_7630,
            0x7371_7368,
            0x4d44,
            0xdead_beef,
        ] {
            assert!(
                !is_nonlocal_filesystem(&linux_filesystem_name(magic)),
                "{magic:#x}"
            );
        }
    }

    #[test]
    fn filesystem_type_reads_the_kernel_and_reports_the_same_error_code() {
        let directory =
            std::env::temp_dir().join(format!("hctl2-site-lock-fs-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let name = filesystem_type(&directory).unwrap();
        assert!(!name.is_empty());
        assert!(!is_nonlocal_filesystem(&name), "{name}");
        // A path with spaces, and a symlink to it, resolve to the same filesystem.
        let spaced = directory.join("with space");
        std::fs::create_dir_all(&spaced).unwrap();
        assert_eq!(filesystem_type(&spaced).unwrap(), name);
        let link = directory.join("link");
        std::os::unix::fs::symlink(&spaced, &link).unwrap();
        assert_eq!(filesystem_type(&link).unwrap(), name);
        // A path the kernel cannot look up keeps the unchanged error code.
        let error = filesystem_type(&directory.join("missing")).unwrap_err();
        assert_eq!(error.code(), "HCTL2_TOOL_FILESYSTEM_UNREADABLE");
        let _ = std::fs::remove_dir_all(&directory);
    }
}

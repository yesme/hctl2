//! Owner-only Unix socket bind/connect.

use std::io;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream as StdUnixStream;
use std::path::{Path, PathBuf};

use tokio::net::UnixListener;

#[must_use]
pub fn socket_path(root: &Path) -> PathBuf {
    root.join("control.sock")
}

pub fn occupied_error(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::AddrInUse,
        format!("control socket already in use: {}", path.display()),
    )
}

/// Bind `path` as mode 0600. A live peer means occupied; a leftover file is unlinked.
pub fn bind_owner_socket(path: &Path) -> io::Result<UnixListener> {
    if path.exists() {
        match StdUnixStream::connect(path) {
            Ok(_) => return Err(occupied_error(path)),
            Err(_) => std::fs::remove_file(path)?,
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let listener = UnixListener::bind(path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

use agency_proto::{PortError, Result, hex};
use rusqlite::Connection;
use std::{
    fs::{self, File, OpenOptions},
    io::Read,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
};

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        Self(PortError::new(
            "AGENCY_STORAGE",
            e.to_string(),
            "repair_agency_storage",
        ))
    }
}
pub(crate) struct DbError(pub PortError);
pub(crate) fn sql<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|e| DbError::from(e).0)
}
pub(crate) fn private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(PortError::invalid("data directory is a symlink"));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}
pub(crate) fn nonce() -> Result<String> {
    let mut bytes = [0; 32];
    File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(hex(&bytes))
}
pub(crate) fn private_file(path: &Path) -> Result<File> {
    if path.exists() && fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(PortError::invalid("private file is a symlink"));
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path)?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(file)
}
pub(crate) fn database(path: &Path) -> Result<Connection> {
    let _file = private_file(path)?;
    let conn = sql(Connection::open(path))?;
    let version: i64 = sql(conn.pragma_query_value(None, "user_version", |r| r.get(0)))?;
    if version > 1 {
        return Err(PortError::new(
            "AGENCY_SCHEMA_NEWER",
            "Agency storage uses a newer schema",
            "use_compatible_agency",
        ));
    }
    sql(conn.pragma_update(None, "journal_mode", "WAL"))?;
    sql(conn.pragma_update(None, "synchronous", "FULL"))?;
    sql(conn.busy_timeout(std::time::Duration::from_secs(3)))?;
    Ok(conn)
}
pub(crate) fn now_ms() -> u64 {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    )
    .unwrap_or(u64::MAX)
}

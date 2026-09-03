//! Reusable storage mechanisms for HCTL2 control.
//!
//! The product-specific outbox, lease and generation rules deliberately do not live here.

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags, backup::Backup, params};
use serde_json::Value;
use sha2::{Digest, Sha256};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// Errors produced by the foundation mechanisms.
#[derive(Debug)]
pub enum FoundationError {
    Io(io::Error),
    Json(serde_json::Error),
    CanonicalJson(String),
    UnsafeCanonicalNumber(String),
    Lock(String),
    Sqlite(rusqlite::Error),
    Secret(String),
}

impl Display for FoundationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Json(error) => write!(formatter, "JSON error: {error}"),
            Self::CanonicalJson(error) => write!(formatter, "JCS error: {error}"),
            Self::UnsafeCanonicalNumber(value) => {
                write!(
                    formatter,
                    "canonical JSON number is outside the safe integer set: {value}"
                )
            }
            Self::Lock(error) => write!(formatter, "file lock error: {error}"),
            Self::Sqlite(error) => write!(formatter, "SQLite error: {error}"),
            Self::Secret(error) => write!(formatter, "secret store error: {error}"),
        }
    }
}

impl Error for FoundationError {}

impl From<io::Error> for FoundationError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for FoundationError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<rusqlite::Error> for FoundationError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

/// Produces RFC 8785 bytes.
///
/// # Errors
///
/// Returns an error when the JSON value cannot be serialized canonically.
pub fn canonical_json(value: &Value) -> Result<Vec<u8>, FoundationError> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|error| FoundationError::CanonicalJson(error.to_string()))
}

/// Produces the lowercase SHA-256 after enforcing HCTL2's integer-only I-JSON subset.
///
/// # Errors
///
/// Returns an error for non-integer or unsafe JSON numbers, or failed canonical serialization.
pub fn canonical_json_sha256(value: &Value) -> Result<String, FoundationError> {
    validate_canonical_numbers(value)?;
    let bytes = canonical_json(value)?;
    Ok(lower_hex(&Sha256::digest(bytes)))
}

fn validate_canonical_numbers(value: &Value) -> Result<(), FoundationError> {
    match value {
        Value::Array(values) => {
            for value in values {
                validate_canonical_numbers(value)?;
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                validate_canonical_numbers(value)?;
            }
        }
        Value::Number(number) => {
            let safe = number
                .as_u64()
                .is_some_and(|value| value <= MAX_SAFE_INTEGER)
                || number
                    .as_i64()
                    .is_some_and(|value| value.unsigned_abs() <= MAX_SAFE_INTEGER);
            if !safe {
                return Err(FoundationError::UnsafeCanonicalNumber(number.to_string()));
            }
        }
        Value::Null | Value::Bool(_) | Value::String(_) => {}
    }
    Ok(())
}

/// An OS-owned exclusive file lock that is released automatically when dropped or killed.
pub struct ExclusiveFileLock {
    _file: File,
}

impl ExclusiveFileLock {
    /// Acquires an exclusive advisory lock without waiting.
    ///
    /// # Errors
    ///
    /// Returns an error when the lock file cannot be opened or another holder owns the lock.
    pub fn try_acquire(path: &Path) -> Result<Self, FoundationError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(path)?;
        file.try_lock()
            .map_err(|error| FoundationError::Lock(error.to_string()))?;
        Ok(Self { _file: file })
    }
}

/// Creates a transactionally consistent SQLite snapshot with the Online Backup API.
///
/// # Errors
///
/// Returns an error when the destination exists, SQLite cannot create the snapshot, or its
/// integrity check fails.
pub fn backup_sqlite(source: &Path, destination: &Path) -> Result<(), FoundationError> {
    if destination.exists() {
        return Err(FoundationError::Io(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "backup destination already exists: {}",
                destination.display()
            ),
        )));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    let source_connection = Connection::open_with_flags(
        source,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    let mut destination_connection = Connection::open(destination)?;
    let backup = Backup::new(&source_connection, &mut destination_connection)?;
    backup.step(-1)?;
    drop(backup);

    let integrity: String =
        destination_connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(FoundationError::Sqlite(rusqlite::Error::InvalidQuery));
    }
    Ok(())
}

/// Which persistent secret backend was selected on this machine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretBackend {
    SystemKeyring,
    UserFile,
}

/// System-keyring-first secret storage with the owner-approved 0600 file fallback.
pub struct SecretStore {
    service: String,
    fallback_root: PathBuf,
    backend: SecretBackend,
}

impl SecretStore {
    #[must_use]
    pub fn detect(service: impl Into<String>, fallback_root: PathBuf) -> Self {
        let backend = if keyring::Entry::store_status().is_ok() {
            SecretBackend::SystemKeyring
        } else {
            SecretBackend::UserFile
        };
        Self {
            service: service.into(),
            fallback_root,
            backend,
        }
    }

    #[must_use]
    pub const fn backend(&self) -> SecretBackend {
        self.backend
    }

    /// Stores a secret in the selected persistent backend.
    ///
    /// # Errors
    ///
    /// Returns an error when the system store or private fallback file rejects the write.
    pub fn set(&self, account: &str, secret: &[u8]) -> Result<(), FoundationError> {
        match self.backend {
            SecretBackend::SystemKeyring => keyring::Entry::new(&self.service, account)
                .and_then(|entry| entry.set_secret(secret))
                .map_err(|error| FoundationError::Secret(error.to_string())),
            SecretBackend::UserFile => self.write_fallback(account, secret),
        }
    }

    /// Reads a secret from the selected persistent backend.
    ///
    /// # Errors
    ///
    /// Returns an error when the account does not exist or the selected backend cannot read it.
    pub fn get(&self, account: &str) -> Result<Vec<u8>, FoundationError> {
        match self.backend {
            SecretBackend::SystemKeyring => keyring::Entry::new(&self.service, account)
                .and_then(|entry| entry.get_secret())
                .map_err(|error| FoundationError::Secret(error.to_string())),
            SecretBackend::UserFile => {
                let mut secret = Vec::new();
                File::open(self.fallback_path(account))?.read_to_end(&mut secret)?;
                Ok(secret)
            }
        }
    }

    fn write_fallback(&self, account: &str, secret: &[u8]) -> Result<(), FoundationError> {
        fs::create_dir_all(&self.fallback_root)?;
        set_private_directory_permissions(&self.fallback_root)?;
        let path = self.fallback_path(account);
        let temporary = path.with_extension(format!("tmp.{}", std::process::id()));
        let mut file = open_private_file(&temporary)?;
        file.write_all(secret)?;
        file.sync_all()?;
        fs::rename(&temporary, &path)?;
        Ok(())
    }

    fn fallback_path(&self, account: &str) -> PathBuf {
        let identity = format!("{}\0{account}", self.service);
        self.fallback_root.join(format!(
            "{}.secret",
            lower_hex(&Sha256::digest(identity.as_bytes()))
        ))
    }
}

#[cfg(unix)]
fn open_private_file(path: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn open_private_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
}

#[cfg(unix)]
fn set_private_directory_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn set_private_directory_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn lower_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

/// A rebuildable FTS5 index kept separate from the authoritative ledger.
pub struct SearchIndex {
    connection: Connection,
}

impl SearchIndex {
    /// Opens or creates a rebuildable FTS5 index.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite cannot open the file or initialize FTS5.
    pub fn open(path: &Path) -> Result<Self, FoundationError> {
        let connection = Connection::open(path)?;
        connection.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(\
                document_id UNINDEXED, body, tokenize='unicode61'\
            );",
        )?;
        Ok(Self { connection })
    }

    /// Replaces one indexed document atomically.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite cannot update or commit the index transaction.
    pub fn replace(&mut self, document_id: &str, body: &str) -> Result<(), FoundationError> {
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "DELETE FROM documents_fts WHERE document_id = ?1",
            [document_id],
        )?;
        transaction.execute(
            "INSERT INTO documents_fts(document_id, body) VALUES (?1, ?2)",
            params![document_id, body],
        )?;
        transaction.commit()?;
        Ok(())
    }

    /// Searches indexed documents in FTS5 rank order.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid FTS syntax or an unreadable index.
    pub fn search(&self, query: &str) -> Result<Vec<String>, FoundationError> {
        let mut statement = self.connection.prepare(
            "SELECT document_id FROM documents_fts \
             WHERE documents_fts MATCH ?1 ORDER BY rank",
        )?;
        let rows = statement.query_map([query], |row| row.get(0))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use rusqlite::Connection;
    use serde_json::json;

    use super::{
        ExclusiveFileLock, FoundationError, SearchIndex, backup_sqlite, canonical_json,
        canonical_json_sha256,
    };

    fn temporary_directory(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must follow the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "hctl2-foundation-{name}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("temporary directory must be created");
        path
    }

    #[test]
    fn canonicalization_matches_rfc_8785_vectors() {
        for (input, expected) in [
            (
                include_str!("../testdata/input/arrays.json"),
                include_str!("../testdata/output/arrays.json"),
            ),
            (
                include_str!("../testdata/input/french.json"),
                include_str!("../testdata/output/french.json"),
            ),
            (
                include_str!("../testdata/input/structures.json"),
                include_str!("../testdata/output/structures.json"),
            ),
            (
                include_str!("../testdata/input/unicode.json"),
                include_str!("../testdata/output/unicode.json"),
            ),
            (
                include_str!("../testdata/input/values.json"),
                include_str!("../testdata/output/values.json"),
            ),
            (
                include_str!("../testdata/input/weird.json"),
                include_str!("../testdata/output/weird.json"),
            ),
        ] {
            let value: serde_json::Value =
                serde_json::from_str(input).expect("official input must parse");
            assert_eq!(
                canonical_json(&value).expect("official input must canonicalize"),
                expected.trim_end_matches('\n').as_bytes()
            );
        }
    }

    #[test]
    fn canonicalization_rejects_floats_and_unsafe_integers() {
        assert!(matches!(
            canonical_json_sha256(&json!({ "value": 1.5 })),
            Err(FoundationError::UnsafeCanonicalNumber(_))
        ));
        assert!(matches!(
            canonical_json_sha256(&json!({ "value": 9_007_199_254_740_992_u64 })),
            Err(FoundationError::UnsafeCanonicalNumber(_))
        ));
        assert_eq!(
            canonical_json_sha256(&json!({ "b": 2, "a": 1 }))
                .expect("safe object must hash")
                .len(),
            64
        );
    }

    #[test]
    fn os_lock_is_released_when_the_guard_drops() {
        let directory = temporary_directory("lock");
        let path = directory.join("control.lock");
        let first = ExclusiveFileLock::try_acquire(&path).expect("first lock must succeed");
        assert!(ExclusiveFileLock::try_acquire(&path).is_err());
        drop(first);
        ExclusiveFileLock::try_acquire(&path).expect("dropped lock must be released");
        fs::remove_dir_all(directory).expect("fixture must be removed");
    }

    #[test]
    fn online_backup_copies_a_wal_database_without_sidecars() {
        let directory = temporary_directory("backup");
        let source = directory.join("ledger.sqlite");
        let destination = directory.join("backup.sqlite");
        let connection = Connection::open(&source).expect("source must open");
        connection
            .execute_batch(
                "PRAGMA journal_mode=WAL;\
                 CREATE TABLE event(sequence INTEGER PRIMARY KEY, body TEXT NOT NULL);\
                 INSERT INTO event(body) VALUES ('persisted');",
            )
            .expect("fixture must initialize");

        backup_sqlite(&source, &destination).expect("backup must succeed");
        let restored = Connection::open(&destination).expect("backup must open alone");
        let body: String = restored
            .query_row("SELECT body FROM event", [], |row| row.get(0))
            .expect("committed row must exist");
        assert_eq!(body, "persisted");
        drop(restored);
        drop(connection);
        fs::remove_dir_all(directory).expect("fixture must be removed");
    }

    #[test]
    fn fts5_index_is_rebuildable_and_searchable() {
        let directory = temporary_directory("fts5");
        let path = directory.join("index.sqlite");
        let mut index = SearchIndex::open(&path).expect("index must open");
        index
            .replace("memo-1", "mechanical verification")
            .expect("document must index");
        assert_eq!(
            index.search("verification").expect("query must work"),
            ["memo-1"]
        );
        drop(index);
        fs::remove_dir_all(directory).expect("fixture must be removed");
    }
}

use agency_proto::{PortError, Result, hex};
use rusqlite::Connection;
use rusqlite_migration::{Migrations, SchemaVersion};
use std::{
    fs::{self, File, OpenOptions},
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
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
fn migration_failed(e: rusqlite_migration::Error) -> PortError {
    PortError::new(
        "AGENCY_MIGRATION_FAILED",
        e.to_string(),
        "inspect_agency_storage",
    )
}
/// Reading the stored version changes nothing, so a failure there is the store
/// being unreadable and keeps the storage code and its repair action.
pub(crate) fn stored_version(
    migrations: &Migrations<'static>,
    conn: &Connection,
) -> Result<SchemaVersion> {
    migrations.current_version(conn).map_err(|e| match e {
        rusqlite_migration::Error::RusqliteError { err, .. } => DbError::from(err).0,
        other => migration_failed(other),
    })
}
pub(crate) fn migrated<T>(r: rusqlite_migration::Result<T>) -> Result<T> {
    r.map_err(migration_failed)
}
pub(crate) fn private_dir(path: &Path) -> Result<()> {
    if path.exists() {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() {
            return Err(PortError::invalid("data directory is a symlink"));
        }
        if !metadata.is_dir() || metadata.uid() != rustix::process::geteuid().as_raw() {
            return Err(PortError::new(
                "UNSAFE_ENDPOINT",
                "directory is not owned by this user",
                "choose_execution_directory",
            ));
        }
    } else {
        fs::create_dir_all(path)?;
        if fs::symlink_metadata(path)?.file_type().is_symlink() {
            return Err(PortError::invalid("data directory is a symlink"));
        }
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
pub(crate) fn database(path: &Path, migrations: &Migrations<'static>) -> Result<Connection> {
    let _file = private_file(path)?;
    let mut conn = sql(Connection::open(path))?;
    if let SchemaVersion::Outside(_) = stored_version(migrations, &conn)? {
        return Err(PortError::new(
            "AGENCY_SCHEMA_NEWER",
            "Agency storage uses a newer schema",
            "use_compatible_agency",
        ));
    }
    // `to_latest` runs its own transaction, and journal_mode cannot change inside one.
    sql(conn.pragma_update(None, "journal_mode", "WAL"))?;
    sql(conn.pragma_update(None, "synchronous", "FULL"))?;
    sql(conn.busy_timeout(std::time::Duration::from_secs(3)))?;
    migrated(migrations.to_latest(&mut conn))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    use rusqlite_migration::{M, Migrations};
    use std::path::PathBuf;

    /// main's `execute_batch` text, verbatim, standing in for a database an older
    /// Agency already wrote.
    const LEGACY_REGISTRY: &str = "CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY,value TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS tenants(control_id TEXT PRIMARY KEY,id TEXT UNIQUE NOT NULL,key TEXT NOT NULL); PRAGMA user_version=1;";
    const LEGACY_TENANT: &str = "CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY,value INTEGER NOT NULL);
        INSERT OR IGNORE INTO settings VALUES('writer',0);
        CREATE TABLE IF NOT EXISTS dispatches(id TEXT PRIMARY KEY,idem TEXT UNIQUE NOT NULL,request BLOB NOT NULL,body BLOB NOT NULL,lease TEXT,lease_expires INTEGER);
        CREATE TABLE IF NOT EXISTS used_leases(id TEXT PRIMARY KEY);
        CREATE TABLE IF NOT EXISTS events(dispatch TEXT NOT NULL,seq INTEGER NOT NULL,body BLOB NOT NULL,PRIMARY KEY(dispatch,seq));
        CREATE TABLE IF NOT EXISTS results(dispatch TEXT NOT NULL,id TEXT PRIMARY KEY,body BLOB NOT NULL,preserved INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE IF NOT EXISTS inputs(dispatch TEXT NOT NULL,idem TEXT NOT NULL,digest TEXT NOT NULL,state TEXT NOT NULL,PRIMARY KEY(dispatch,idem)); PRAGMA user_version=1;";

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("hctl2-agency-store-{}", nonce().unwrap()));
            private_dir(&path).unwrap();
            Self(path)
        }
        fn database(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn version(conn: &Connection) -> i64 {
        conn.pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap()
    }
    fn tables(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            )
            .unwrap();
        sql(stmt.query_map([], |r| r.get::<_, String>(0)))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    }
    fn journal_mode(conn: &Connection) -> String {
        conn.pragma_query_value(None, "journal_mode", |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn a_new_directory_gets_both_databases_at_the_latest_version() {
        let scratch = Scratch::new();
        let registry = database(
            &scratch.database("registry.sqlite"),
            &crate::service::registry_migrations(),
        )
        .unwrap();
        assert_eq!(version(&registry), 1);
        assert_eq!(tables(&registry), ["settings", "tenants"]);
        assert_eq!(journal_mode(&registry), "wal");
        let tenant = database(
            &scratch.database("tenant.sqlite"),
            &crate::tenant::tenant_migrations(),
        )
        .unwrap();
        assert_eq!(version(&tenant), 1);
        assert_eq!(
            tables(&tenant),
            [
                "dispatches",
                "events",
                "inputs",
                "results",
                "settings",
                "used_leases"
            ]
        );
        assert_eq!(journal_mode(&tenant), "wal");
        // Migration 1 builds `settings` but does not seed it. The writer row has to
        // come back on every open, and an applied migration never runs again, so
        // `Tenant::open` inserts it outside the migration list.
        assert_eq!(
            sql(tenant.query_row("SELECT COUNT(*) FROM settings", [], |r| r.get::<_, i64>(0)))
                .unwrap(),
            0
        );
    }

    #[test]
    fn a_database_written_by_main_opens_with_its_rows_intact() {
        let scratch = Scratch::new();
        {
            let registry = sql(Connection::open(scratch.database("registry.sqlite"))).unwrap();
            sql(registry.execute_batch(LEGACY_REGISTRY)).unwrap();
            sql(registry.execute(
                "INSERT INTO tenants VALUES('control-1','tenant-1','key-1')",
                [],
            ))
            .unwrap();
            let tenant = sql(Connection::open(scratch.database("tenant.sqlite"))).unwrap();
            sql(tenant.execute_batch(LEGACY_TENANT)).unwrap();
            sql(tenant.execute(
                "INSERT INTO dispatches(id,idem,request,body) VALUES('d-1','i-1',?1,?2)",
                params![b"request".to_vec(), b"body".to_vec()],
            ))
            .unwrap();
            sql(tenant.execute(
                "INSERT INTO results(dispatch,id,body) VALUES('d-1','r-1',?1)",
                params![b"payload".to_vec()],
            ))
            .unwrap();
        }
        let registry = database(
            &scratch.database("registry.sqlite"),
            &crate::service::registry_migrations(),
        )
        .unwrap();
        assert_eq!(version(&registry), 1);
        assert_eq!(tables(&registry), ["settings", "tenants"]);
        let tenant_row: (String, String, String) = sql(registry.query_row(
            "SELECT control_id,id,key FROM tenants",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ))
        .unwrap();
        assert_eq!(
            tenant_row,
            (
                "control-1".to_owned(),
                "tenant-1".to_owned(),
                "key-1".to_owned()
            )
        );
        let tenant = database(
            &scratch.database("tenant.sqlite"),
            &crate::tenant::tenant_migrations(),
        )
        .unwrap();
        assert_eq!(version(&tenant), 1);
        assert_eq!(
            tables(&tenant),
            [
                "dispatches",
                "events",
                "inputs",
                "results",
                "settings",
                "used_leases"
            ]
        );
        let dispatch: (Vec<u8>, Vec<u8>) = sql(tenant.query_row(
            "SELECT request,body FROM dispatches WHERE id='d-1'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ))
        .unwrap();
        assert_eq!(dispatch, (b"request".to_vec(), b"body".to_vec()));
        let result: Vec<u8> =
            sql(tenant.query_row("SELECT body FROM results WHERE id='r-1'", [], |r| r.get(0)))
                .unwrap();
        assert_eq!(result, b"payload".to_vec());
        assert_eq!(
            sql(
                tenant.query_row("SELECT value FROM settings WHERE key='writer'", [], |r| {
                    r.get::<_, i64>(0)
                })
            )
            .unwrap(),
            0
        );
    }

    #[test]
    fn a_database_newer_than_the_known_migrations_is_refused() {
        let scratch = Scratch::new();
        let path = scratch.database("tenant.sqlite");
        drop(database(&path, &crate::tenant::tenant_migrations()).unwrap());
        let conn = sql(Connection::open(&path)).unwrap();
        sql(conn.pragma_update(None, "user_version", 2)).unwrap();
        drop(conn);
        let error = database(&path, &crate::tenant::tenant_migrations()).unwrap_err();
        assert_eq!(error.code, "AGENCY_SCHEMA_NEWER");
        assert_eq!(error.message, "Agency storage uses a newer schema");
        assert_eq!(error.recovery_action, "use_compatible_agency");
    }

    #[test]
    fn a_file_that_is_not_a_database_keeps_the_storage_code() {
        let scratch = Scratch::new();
        let path = scratch.database("tenant.sqlite");
        fs::write(&path, b"not a database").unwrap();
        let error = database(&path, &crate::tenant::tenant_migrations()).unwrap_err();
        assert_eq!(error.code, "AGENCY_STORAGE");
        assert_eq!(error.recovery_action, "repair_agency_storage");
    }

    #[test]
    fn a_migration_that_fails_halfway_leaves_nothing_behind() {
        let scratch = Scratch::new();
        let path = scratch.database("rollback.sqlite");
        let broken = Migrations::new(vec![
            M::up("CREATE TABLE first(one INTEGER PRIMARY KEY);"),
            // The first statement applies; the second names a table that is not there.
            M::up("CREATE TABLE second(two INTEGER PRIMARY KEY); INSERT INTO absent VALUES(1);"),
        ]);
        let error = database(&path, &broken).unwrap_err();
        assert_eq!(error.code, "AGENCY_MIGRATION_FAILED");
        assert_eq!(error.recovery_action, "inspect_agency_storage");
        let conn = sql(Connection::open(&path)).unwrap();
        assert_eq!(version(&conn), 0);
        assert!(tables(&conn).is_empty());
    }
}

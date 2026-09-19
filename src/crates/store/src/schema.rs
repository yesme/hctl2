use std::path::Path;

use foundation::backup_sqlite;
use rusqlite::{Connection, backup::Backup};
use rusqlite_migration::{M, Migrations};

use crate::{Result, StoreError};

pub(crate) const VERSION: u32 = 2;
const APPLICATION_ID: u32 = 1_213_372_018;

const IDENTITY: &str = "
PRAGMA application_id = 1213372018;
CREATE TABLE control_identity (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 control_id TEXT NOT NULL UNIQUE,
 writer_generation INTEGER NOT NULL CHECK(writer_generation>=0)
);
INSERT INTO control_identity VALUES(1,lower(hex(randomblob(16))),0);
";

const CORE: &str = "
CREATE TABLE commands (
 idempotency_key TEXT PRIMARY KEY, command_id TEXT NOT NULL UNIQUE,
 fingerprint TEXT NOT NULL, envelope TEXT NOT NULL, result TEXT NOT NULL
);
CREATE TABLE events (
 sequence INTEGER PRIMARY KEY AUTOINCREMENT,
 command_key TEXT NOT NULL REFERENCES commands(idempotency_key) DEFERRABLE INITIALLY DEFERRED,
 object_key TEXT NOT NULL, version INTEGER NOT NULL, record TEXT NOT NULL,
 UNIQUE(object_key,version)
);
CREATE TABLE inbox (
 source_key TEXT NOT NULL, message_key TEXT NOT NULL, digest TEXT NOT NULL,
 binding TEXT NOT NULL,
 command_key TEXT NOT NULL REFERENCES commands(idempotency_key),
 PRIMARY KEY(source_key,message_key)
);
CREATE TABLE outbox (
 intent_id TEXT PRIMARY KEY,
 command_key TEXT NOT NULL REFERENCES commands(idempotency_key) DEFERRABLE INITIALLY DEFERRED,
 intent TEXT NOT NULL, conflict_key TEXT NOT NULL,
 state TEXT NOT NULL CHECK(state IN ('pending','unknown','confirmed')),
 generation INTEGER NOT NULL, confirmation TEXT
);
CREATE UNIQUE INDEX unresolved_effect ON outbox(conflict_key) WHERE state!='confirmed';
CREATE TABLE materials (
 material_id TEXT PRIMARY KEY, reference TEXT NOT NULL,
 command_key TEXT NOT NULL REFERENCES commands(idempotency_key) DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE deliveries (
 delivery_id TEXT PRIMARY KEY, grant_json TEXT NOT NULL,
 command_key TEXT NOT NULL REFERENCES commands(idempotency_key) DEFERRABLE INITIALLY DEFERRED,
 state TEXT NOT NULL CHECK(state IN ('pending','confirmed'))
);
CREATE TABLE secret_references (binding_key TEXT PRIMARY KEY, secret_reference TEXT NOT NULL);
";

pub(crate) const PROJECTION: &str = "
CREATE TABLE IF NOT EXISTS objects (
 object_key TEXT PRIMARY KEY, kind TEXT NOT NULL, scope TEXT NOT NULL,
 version INTEGER NOT NULL, revision_digest TEXT NOT NULL, record TEXT NOT NULL,
 project_id TEXT, room_kind TEXT, entity_key TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS one_main_room ON objects(project_id) WHERE kind='room' AND room_kind='main';
CREATE UNIQUE INDEX IF NOT EXISTS one_task_per_entity ON objects(project_id,entity_key) WHERE kind='task' AND entity_key IS NOT NULL;
";

pub(crate) fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(IDENTITY), M::up(CORE).foreign_key_check()])
}

pub(crate) fn inspect(conn: &Connection) -> Result<u32> {
    let app: u32 = conn.query_row("PRAGMA application_id", [], |r| r.get(0))?;
    let version: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let tables: u32 = conn.query_row(
        "SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
        [],
        |r| r.get(0),
    )?;
    if version == 0 && app == 0 && tables == 0 {
        return Ok(0);
    }
    if app != APPLICATION_ID || version == 0 || version > VERSION {
        return Err(StoreError::new(
            "SCHEMA_UNSUPPORTED",
            "wrong application, partial or newer schema",
            "use_compatible_version",
        ));
    }
    identity(conn)?;
    if version >= 2 {
        for sql in [
            "SELECT idempotency_key,command_id,fingerprint,envelope,result FROM commands LIMIT 0",
            "SELECT sequence,command_key,object_key,version,record FROM events LIMIT 0",
            "SELECT source_key,message_key,digest,binding,command_key FROM inbox LIMIT 0",
            "SELECT intent_id,command_key,intent,conflict_key,state,generation,confirmation FROM outbox LIMIT 0",
            "SELECT material_id,reference,command_key FROM materials LIMIT 0",
            "SELECT delivery_id,grant_json,command_key,state FROM deliveries LIMIT 0",
            "SELECT binding_key,secret_reference FROM secret_references LIMIT 0",
        ] {
            conn.prepare(sql)?;
        }
    }
    integrity(conn)?;
    Ok(version)
}

pub(crate) fn integrity(conn: &Connection) -> Result<()> {
    let result: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if result != "ok" || conn.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(StoreError::new(
            "STORAGE_CORRUPT",
            "database integrity validation failed",
            "restore_backup",
        ));
    }
    Ok(())
}

/// Shared by startup and offline restore. Callers keep a snapshot or stage the upgrade
/// away from live records, so any migration or postcondition failure can be discarded.
pub(crate) fn upgrade(conn: &mut Connection, migrations: &Migrations<'_>) -> Result<()> {
    let old = inspect(conn)?;
    let previous = (old != 0).then(|| identity(conn)).transpose()?;
    migrations.to_latest(conn).map_err(|e| {
        StoreError::new(
            "MIGRATION_FAILED",
            e.to_string(),
            "inspect_migration_snapshot",
        )
    })?;
    let current = identity(conn)?;
    if inspect(conn)? != VERSION || previous.as_ref().is_some_and(|before| before != &current) {
        return Err(StoreError::new(
            "MIGRATION_FAILED",
            "migration changed identity/generation or left an unreadable schema",
            "inspect_migration_snapshot",
        ));
    }
    crate::store::rebuild(conn)
}

pub(crate) fn identity(conn: &Connection) -> Result<(String, i64)> {
    Ok(conn.query_row(
        "SELECT control_id,writer_generation FROM control_identity WHERE singleton=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?)
}

pub(crate) fn snapshot(source: &Path, destination: &Path) -> Result<()> {
    backup_sqlite(source, destination)?;
    let conn = Connection::open(destination)?;
    // A completed snapshot is a standalone file, without a required WAL sidecar.
    conn.pragma_update(None, "journal_mode", "DELETE")?;
    integrity(&conn)?;
    drop(conn);
    std::fs::File::open(destination)?.sync_all()?;
    Ok(())
}

pub(crate) fn restore_into(source: &Path, target: &mut Connection) -> Result<()> {
    let source = Connection::open_with_flags(source, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    integrity(&source)?;
    let backup = Backup::new(&source, target)?;
    if backup.step(-1)? != rusqlite::backup::StepResult::Done {
        return Err(StoreError::new(
            "RESTORE_INCOMPLETE",
            "SQLite restore did not finish",
            "retry_restore",
        ));
    }
    drop(backup);
    integrity(target)
}

#[cfg(test)]
mod tests;

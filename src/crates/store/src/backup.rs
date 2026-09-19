use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

use foundation::ExclusiveFileLock;
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};

use crate::materials::{Materials, hash, private_dir};
use crate::schema;
use crate::{MaterialRef, Result, StartupStatus, Store, StoreError, WriterGeneration};

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupReport {
    pub format: String,
    pub control_id: String,
    pub schema_version: u32,
    pub writer_generation: i64,
    pub event_sequence: i64,
    pub database_sha256: String,
    pub promised_material_count: usize,
}

impl Store {
    /// The sole writer is borrowed for the complete set: one SQLite boundary plus all promised
    /// bytes and definitions. No content database, Agency installation, secret values or PTY cache.
    pub fn backup(
        &mut self,
        generation: WriterGeneration,
        destination: &Path,
    ) -> Result<BackupReport> {
        self.check_writer(generation)?;
        if destination.exists() {
            return Err(StoreError::invalid("backup destination already exists"));
        }
        private_dir(destination)?;
        let database = destination.join("control.sqlite");
        schema::snapshot(&self.root.join("control.sqlite"), &database)?;
        let snapshot = readonly(&database)?;
        let material_count = verify_materials(&snapshot, &self.materials, &self.control_id)?;
        self.materials.export(&destination.join("materials.git"))?;
        let copy = Materials::open(&destination.join("materials.git"), false)?;
        verify_materials(&snapshot, &copy, &self.control_id)?;
        let report = BackupReport {
            format: "hctl2.control-backup.v1".into(),
            control_id: self.control_id.clone(),
            schema_version: schema::inspect(&snapshot)?,
            writer_generation: schema::identity(&snapshot)?.1,
            event_sequence: snapshot.query_row(
                "SELECT coalesce(max(sequence),0) FROM events",
                [],
                |r| r.get(0),
            )?,
            database_sha256: hash(&fs::read(&database)?),
            promised_material_count: material_count,
        };
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(destination.join("manifest.json"))?;
        file.write_all(&serde_json::to_vec_pretty(&report)?)?;
        file.sync_all()?;
        File::open(destination)?.sync_all()?;
        Self::verify_backup(destination)
    }

    pub fn verify_backup(path: &Path) -> Result<BackupReport> {
        let report: BackupReport = serde_json::from_slice(&fs::read(path.join("manifest.json"))?)?;
        if report.format != "hctl2.control-backup.v1"
            || report.database_sha256 != hash(&fs::read(path.join("control.sqlite"))?)
        {
            return Err(StoreError::new(
                "BACKUP_INCOMPLETE",
                "backup manifest or snapshot hash mismatch",
                "select_complete_backup",
            ));
        }
        let conn = readonly(&path.join("control.sqlite"))?;
        let version = schema::inspect(&conn)?;
        if version == 0
            || version != report.schema_version
            || schema::identity(&conn)? != (report.control_id.clone(), report.writer_generation)
        {
            return Err(StoreError::new(
                "BACKUP_INCOMPLETE",
                "backup identity/schema boundary mismatch",
                "select_complete_backup",
            ));
        }
        // Schema 1 predates commands, events and material admission.
        let seq: i64 = if version == 1 {
            0
        } else {
            conn.query_row("SELECT coalesce(max(sequence),0) FROM events", [], |r| {
                r.get(0)
            })?
        };
        let materials = Materials::open(&path.join("materials.git"), false)?;
        if seq != report.event_sequence
            || verify_materials(&conn, &materials, &report.control_id)?
                != report.promised_material_count
        {
            return Err(StoreError::new(
                "BACKUP_INCOMPLETE",
                "backup material set/boundary mismatch",
                "select_complete_backup",
            ));
        }
        Ok(report)
    }

    /// Offline restore. The caller must stop its previous writer; lock contention refuses restore.
    /// Copy bytes first without removing existing refs, then use SQLite's atomic Backup API to
    /// replace records with an already-fenced snapshot. There is no two-directory rename window.
    pub fn restore(root: &Path, backup: &Path) -> Result<Self> {
        private_dir(root)?;
        let root = root.canonicalize()?;
        let lock = ExclusiveFileLock::try_acquire(&root.join("control.lock"))?;
        let report = Self::verify_backup(backup)?;
        let db = root.join("control.sqlite");
        let mut conn = Connection::open(&db)?;
        let old_schema = schema::inspect(&conn)?;
        let before = if old_schema == 0 {
            0
        } else {
            let (id, generation) = schema::identity(&conn)?;
            if id != report.control_id {
                return Err(StoreError::new(
                    "RESTORE_IDENTITY_CONFLICT",
                    "cannot merge or replace another control's identity",
                    "select_matching_backup",
                ));
            }
            generation
        };
        let materials = Materials::open(&root.join("materials.git"), true)?;
        materials.import(&backup.join("materials.git"))?;
        let source = readonly(&backup.join("control.sqlite"))?;
        verify_materials(&source, &materials, &report.control_id)?;
        // This staging snapshot is not advertised as a completed user backup. Retaining it
        // makes an interrupted restore inspectable; a retry checks the live generation again.
        let work = root.join("restore-work");
        private_dir(&work)?;
        let nonce: String =
            conn.query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0))?;
        let staged = work.join(format!("{nonce}.sqlite"));
        schema::snapshot(&backup.join("control.sqlite"), &staged)?;
        // SQLite Backup can change file-header counters in its destination. Recheck the
        // immutable source file; inspect the staged database semantically before fencing it.
        if hash(&fs::read(backup.join("control.sqlite"))?) != report.database_sha256 {
            return Err(StoreError::new(
                "BACKUP_CHANGED",
                "backup changed during restore preparation",
                "verify_backup_again",
            ));
        }
        let mut stage = Connection::open(&staged)?;
        if schema::inspect(&stage)? != report.schema_version
            || schema::identity(&stage)? != (report.control_id.clone(), report.writer_generation)
        {
            return Err(StoreError::new(
                "BACKUP_CHANGED",
                "staged backup identity/schema differs",
                "verify_backup_again",
            ));
        }
        verify_materials(&stage, &materials, &report.control_id)?;
        // Upgrade and rebuild on the staged copy, under the same writer lock. A failure
        // leaves both the original backup and the live database untouched.
        stage.pragma_update(None, "foreign_keys", "ON")?;
        schema::upgrade(&mut stage, &schema::migrations())?;
        verify_materials(&stage, &materials, &report.control_id)?;
        let next = before
            .max(report.writer_generation)
            .checked_add(1)
            .ok_or_else(|| StoreError::invalid("writer generation exhausted"))?;
        crate::model::safe_version(next)?;
        stage.execute(
            "UPDATE control_identity SET writer_generation=?1 WHERE singleton=1",
            [next],
        )?;
        drop(stage);
        // The old data is recoverable too, without ever copying an active SQLite file.
        if old_schema != 0 {
            schema::snapshot(&db, &work.join(format!("before-{nonce}.sqlite")))?;
        }
        schema::restore_into(&staged, &mut conn)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        let status = StartupStatus::default();
        status.set(2);
        let mut result = Self {
            conn,
            materials,
            root,
            control_id: report.control_id,
            generation: WriterGeneration(next),
            status,
            _lock: lock,
        };
        result.rebuild_projections(result.generation)?;
        Ok(result)
    }
}

fn readonly(path: &Path) -> Result<Connection> {
    Ok(Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?)
}

fn verify_materials(conn: &Connection, materials: &Materials, control_id: &str) -> Result<usize> {
    if schema::inspect(conn)? == 1 {
        return Ok(0);
    }
    let mut query = conn.prepare("SELECT reference FROM materials ORDER BY material_id")?;
    let references = query.query_map([], |r| r.get::<_, String>(0))?;
    let mut count = 0;
    for reference in references {
        let reference: MaterialRef = serde_json::from_str(&reference?)?;
        if reference.control_id != control_id {
            return Err(StoreError::invalid("material control identity mismatch"));
        }
        materials.read(&reference)?;
        count += 1;
    }
    // Historical references, including original Bundle bytes and Profile definitions, remain
    // part of the promise. Missing locator rows must fail even if the Git objects still exist.
    let mut events = conn.prepare("SELECT record FROM events ORDER BY sequence")?;
    for row in events.query_map([], |r| r.get::<_, String>(0))? {
        let record: crate::Record = serde_json::from_str(&row?)?;
        record.validate()?;
        for reference in record.materials {
            let saved: String = conn.query_row(
                "SELECT reference FROM materials WHERE material_id=?1",
                [&reference.material_id],
                |r| r.get(0),
            )?;
            if serde_json::from_str::<MaterialRef>(&saved)? != reference {
                return Err(StoreError::new(
                    "BACKUP_INCOMPLETE",
                    "historical material locator missing or changed",
                    "restore_material",
                ));
            }
        }
    }
    Ok(count)
}

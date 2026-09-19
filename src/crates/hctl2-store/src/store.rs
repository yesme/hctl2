use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use hctl2_foundation::ExclusiveFileLock;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use rusqlite_migration::Migrations;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::materials::{Materials, private_dir};
use crate::schema;
use crate::{
    Command, CommandTransaction, DeliveryGrant, EffectIntent, EffectState, InboxEntry, MaterialRef,
    ObjectKey, Record, Result, Scope, StoreError, TrustedActor,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WriterGeneration(pub i64);

/// The daemon can query this while opening/migrating on its storage worker thread.
#[derive(Clone, Default)]
pub struct StartupStatus(Arc<AtomicU8>);

impl StartupStatus {
    pub fn require_ready(&self) -> Result<()> {
        match self.0.load(Ordering::Acquire) {
            2 => Ok(()),
            1 => Err(StoreError::new(
                "UPGRADE_IN_PROGRESS",
                "schema upgrade in progress",
                "retry_after_upgrade",
            )),
            _ => Err(StoreError::new(
                "STORE_NOT_READY",
                "storage is not serving",
                "check_status",
            )),
        }
    }
    pub(crate) fn set(&self, value: u8) {
        self.0.store(value, Ordering::Release);
    }
}

/// Owns one OS lock and one SQLite writer. This is not Clone and exposes no raw connection.
pub struct Store {
    pub(crate) conn: Connection,
    pub(crate) materials: Materials,
    pub(crate) root: PathBuf,
    pub(crate) control_id: String,
    pub(crate) generation: WriterGeneration,
    pub(crate) status: StartupStatus,
    // Dropped after the connection. The lock file is never removed or replaced.
    pub(crate) _lock: ExclusiveFileLock,
}

impl Drop for Store {
    fn drop(&mut self) {
        self.status.set(0);
    }
}

impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        Self::open_with_status(root, StartupStatus::default())
    }

    pub fn open_with_status(root: &Path, status: StartupStatus) -> Result<Self> {
        Self::open_migrations(root, status, &schema::migrations())
    }

    pub(crate) fn open_migrations(
        root: &Path,
        status: StartupStatus,
        migrations: &Migrations<'_>,
    ) -> Result<Self> {
        status.set(0);
        let result = (|| {
            private_dir(root)?;
            let root = root.canonicalize()?;
            let lock = ExclusiveFileLock::try_acquire(&root.join("control.lock"))?;
            let db = root.join("control.sqlite");
            let mut conn = Connection::open(&db)?;
            let old = schema::inspect(&conn)?;
            conn.pragma_update(None, "foreign_keys", "ON")?;
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "FULL")?;
            if old < schema::VERSION {
                status.set(1);
                let snapshot_dir = root.join("migrations");
                private_dir(&snapshot_dir)?;
                let nonce: String =
                    conn.query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0))?;
                let snapshot = snapshot_dir.join(format!("before-v{old}-{nonce}.sqlite"));
                schema::snapshot(&db, &snapshot)?;
                let upgraded = schema::upgrade(&mut conn, migrations);
                if let Err(mut error) = upgraded {
                    // Preserve the failed database for diagnosis; rollback is via SQLite, not file copy.
                    let diagnostic =
                        schema::snapshot(&db, &snapshot_dir.join(format!("failed-{nonce}.sqlite")));
                    schema::restore_into(&snapshot, &mut conn)?;
                    if diagnostic.is_err() {
                        error.message.push_str("; failed-state snapshot unavailable; verified pre-upgrade snapshot restored");
                    }
                    return Err(error);
                }
            } else {
                rebuild(&mut conn)?;
            }
            if schema::inspect(&conn)? != schema::VERSION {
                return Err(StoreError::new(
                    "SCHEMA_UNSUPPORTED",
                    "upgrade did not reach current schema",
                    "use_compatible_version",
                ));
            }
            let (control_id, old_generation) = schema::identity(&conn)?;
            let no_promises: bool =
                conn.query_row("SELECT count(*)=0 FROM materials", [], |r| r.get(0))?;
            let materials = Materials::open(&root.join("materials.git"), no_promises)?;
            let generation = advance_generation(&mut conn, old_generation)?;
            let store = Self {
                conn,
                materials,
                root,
                control_id,
                generation,
                status: status.clone(),
                _lock: lock,
            };
            // Missing promised bytes are reported at read/admission/backup, not reconstructed from Git history.
            status.set(2);
            Ok(store)
        })();
        if result.is_err() {
            status.set(0);
        }
        result
    }

    pub fn control_id(&self) -> &str {
        &self.control_id
    }
    pub fn generation(&self) -> WriterGeneration {
        self.generation
    }
    pub fn startup_status(&self) -> StartupStatus {
        self.status.clone()
    }

    pub(crate) fn check_writer(&self, generation: WriterGeneration) -> Result<()> {
        self.status.require_ready()?;
        if self.generation != generation || schema::identity(&self.conn)?.1 != generation.0 {
            return Err(StoreError::new(
                "STALE_WRITER",
                "writer generation has been replaced",
                "reconnect_control",
            ));
        }
        Ok(())
    }

    /// Exactly one trusted reducer transaction. No network or external writes in the callback.
    pub fn submit<F>(
        &mut self,
        generation: WriterGeneration,
        trusted: &TrustedActor,
        command: &Command,
        inbox: Option<&InboxEntry>,
        reduce: F,
    ) -> Result<Value>
    where
        F: FnOnce(&mut CommandTransaction<'_>) -> Result<Value>,
    {
        self.check_writer(generation)?;
        let fingerprint = command.validate(trusted)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some((old, result)) = tx
            .query_row(
                "SELECT fingerprint,result FROM commands WHERE idempotency_key=?1 OR command_id=?2",
                params![command.idempotency_key, command.command_id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?
        {
            if old != fingerprint {
                return Err(StoreError::new(
                    "IDEMPOTENCY_CONFLICT",
                    "command identity reused with different input or authority",
                    "use_original_command",
                ));
            }
            return Ok(serde_json::from_str(&result)?);
        }
        if let Some(entry) = inbox
            && let Some(result) = entry.previous(&tx, &command.target)?
        {
            return Ok(result);
        }
        crate::command::check_expected(&tx, &command.target, &command.expected)?;
        let mut context = CommandTransaction {
            failed: false,
            tx: &tx,
            materials: &self.materials,
            control_id: &self.control_id,
            command,
            actor: trusted,
            generation,
        };
        let result = reduce(&mut context)?;
        if context.failed {
            return Err(StoreError::new(
                "COMMAND_ABORTED",
                "a failed transaction operation was suppressed by the reducer",
                "fix_reducer",
            ));
        }
        // Validation also ensures results can safely cross the canonical boundary.
        hctl2_foundation::canonical_json_sha256(&result)?;
        tx.execute(
            "INSERT INTO commands VALUES(?1,?2,?3,?4,?5)",
            params![
                command.idempotency_key,
                command.command_id,
                fingerprint,
                serde_json::to_string(command)?,
                serde_json::to_string(&result)?
            ],
        )?;
        if let Some(entry) = inbox {
            entry.insert(&tx, &command.idempotency_key, &command.target)?;
        }
        tx.commit()?;
        Ok(result)
    }

    pub fn get(&self, key: &ObjectKey) -> Result<Option<Record>> {
        self.status.require_ready()?;
        crate::command::get_record(&self.conn, key)
    }

    /// History comes from admitted events, never candidates or Git log ordering.
    pub fn versions(&self, key: &ObjectKey) -> Result<Vec<Record>> {
        self.status.require_ready()?;
        let mut query = self
            .conn
            .prepare("SELECT record FROM events WHERE object_key=?1 ORDER BY version")?;
        query
            .query_map([key.encoded()?], |r| r.get::<_, String>(0))?
            .map(|r| Ok(serde_json::from_str(&r?)?))
            .collect()
    }

    pub fn rebuild_projections(&mut self, generation: WriterGeneration) -> Result<()> {
        self.check_writer(generation)?;
        rebuild(&mut self.conn)
    }

    pub fn save_material(
        &self,
        generation: WriterGeneration,
        actor: &TrustedActor,
        scope: &Scope,
        command_key: &str,
        slot: &str,
        bytes: &[u8],
    ) -> Result<MaterialRef> {
        self.check_writer(generation)?;
        actor.permits(scope)?;
        self.materials
            .save(&self.control_id, scope, command_key, slot, bytes)
    }

    pub fn read_material(&self, actor: &TrustedActor, reference: &MaterialRef) -> Result<Vec<u8>> {
        actor.permits(&reference.scope)?;
        self.require_admitted(reference)?;
        self.materials.read(reference)
    }

    pub(crate) fn require_admitted(&self, reference: &MaterialRef) -> Result<()> {
        if reference.control_id != self.control_id {
            return Err(StoreError::invalid("material belongs to another control"));
        }
        let saved: Option<String> = self
            .conn
            .query_row(
                "SELECT reference FROM materials WHERE material_id=?1",
                [&reference.material_id],
                |r| r.get(0),
            )
            .optional()?;
        if saved
            .as_deref()
            .map(serde_json::from_str::<MaterialRef>)
            .transpose()?
            .as_ref()
            != Some(reference)
        {
            return Err(StoreError::new(
                "MATERIAL_NOT_ADMITTED",
                "no admitted locator for material",
                "restore_or_admit_material",
            ));
        }
        Ok(())
    }

    /// A recipient only receives the explicit persisted grant, not a general store handle.
    /// Returning bytes is not delivery acknowledgement; confirm_delivery verifies readback later.
    pub fn delivery_bytes(
        &self,
        delivery_id: &str,
        recipient: &str,
        purpose: &str,
    ) -> Result<Vec<Vec<u8>>> {
        let grant: String = self.conn.query_row(
            "SELECT grant_json FROM deliveries WHERE delivery_id=?1",
            [delivery_id],
            |r| r.get(0),
        )?;
        let grant: DeliveryGrant = serde_json::from_str(&grant)?;
        if grant.recipient != recipient || grant.purpose != purpose {
            return Err(StoreError::new(
                "PERMISSION_DENIED",
                "delivery recipient or purpose does not match",
                "request_authorization",
            ));
        }
        grant
            .materials
            .iter()
            .map(|m| {
                self.require_admitted(m)?;
                self.materials.read(m)
            })
            .collect()
    }

    pub fn effect(&self, id: &str) -> Result<(EffectIntent, EffectState)> {
        crate::command::effect(&self.conn, id)
    }

    /// Before attempting delivery, persist uncertainty. A crashed sender must read back, not resend.
    pub fn begin_effect(&mut self, generation: WriterGeneration, id: &str) -> Result<EffectIntent> {
        self.check_writer(generation)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (intent, state) = crate::command::effect(&tx, id)?;
        let recorded_generation: i64 = tx.query_row(
            "SELECT generation FROM outbox WHERE intent_id=?1",
            [id],
            |r| r.get(0),
        )?;
        if state != EffectState::Pending || recorded_generation != generation.0 {
            return Err(StoreError::new(
                "READBACK_REQUIRED",
                "effect is unresolved, completed or belongs to an old writer",
                "read_back_original_intent",
            ));
        }
        tx.execute("UPDATE outbox SET state='unknown' WHERE intent_id=?1", [id])?;
        tx.commit()?;
        Ok(intent)
    }

    /// Re-authorizing an unattempted intent after restart requires the caller's frozen-policy check.
    /// Unknown effects cannot pass this route, even if they were never actually sent.
    pub fn resume_pending_effect(
        &mut self,
        generation: WriterGeneration,
        id: &str,
        still_authorized: bool,
    ) -> Result<()> {
        self.check_writer(generation)?;
        if !still_authorized {
            return Err(StoreError::new(
                "PERMISSION_DENIED",
                "original authorization is no longer valid",
                "reconcile_authorization",
            ));
        }
        if self.conn.execute(
            "UPDATE outbox SET generation=?1 WHERE intent_id=?2 AND state='pending'",
            params![generation.0, id],
        )? != 1
        {
            return Err(StoreError::new(
                "READBACK_REQUIRED",
                "only never-attempted effects can resume",
                "read_back_original_intent",
            ));
        }
        Ok(())
    }

    /// For a trusted adapter inside control; never expose secret values on the public query API.
    pub fn require_secret(
        &self,
        binding: &crate::Reference,
        secrets: &hctl2_foundation::SecretStore,
    ) -> Result<Vec<u8>> {
        binding.validate()?;
        let reference: Option<String> = self
            .conn
            .query_row(
                "SELECT secret_reference FROM secret_references WHERE binding_key=?1",
                [serde_json::to_string(binding)?],
                |r| r.get(0),
            )
            .optional()?;
        let unavailable = || {
            StoreError::new(
                "CREDENTIAL_UNAVAILABLE",
                "binding credential unavailable",
                "restore_secret_store",
            )
        };
        let bytes = secrets
            .get(&reference.ok_or_else(unavailable)?)
            .map_err(|_| unavailable())?;
        if bytes.is_empty() {
            return Err(unavailable());
        }
        Ok(bytes)
    }

    pub fn delivery_confirmed(&self, id: &str) -> Result<bool> {
        Ok(self.conn.query_row(
            "SELECT state='confirmed' FROM deliveries WHERE delivery_id=?1",
            [id],
            |r| r.get(0),
        )?)
    }

    pub fn pending_effects(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT intent_id FROM outbox WHERE state!='confirmed' ORDER BY rowid")?;
        Ok(stmt
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?)
    }
}

pub(crate) fn advance_generation(conn: &mut Connection, previous: i64) -> Result<WriterGeneration> {
    let next = previous
        .checked_add(1)
        .ok_or_else(|| StoreError::invalid("writer generation exhausted"))?;
    crate::model::safe_version(next)?;
    if conn.execute("UPDATE control_identity SET writer_generation=?1 WHERE singleton=1 AND writer_generation=?2", params![next,previous])? != 1 {
        return Err(StoreError::new("STALE_WRITER", "generation compare-and-swap failed", "reconnect_control"));
    }
    Ok(WriterGeneration(next))
}

pub(crate) fn rebuild(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute_batch("DROP TABLE IF EXISTS objects;")?;
    tx.execute_batch(schema::PROJECTION)?;
    let mut events = tx.prepare("SELECT record FROM events ORDER BY sequence")?;
    let records = events.query_map([], |r| r.get::<_, String>(0))?;
    for record in records {
        crate::command::project_record(&tx, &serde_json::from_str(&record?)?)?;
    }
    drop(events);
    tx.commit()?;
    Ok(())
}

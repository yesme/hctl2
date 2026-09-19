use hctl2_foundation::canonical_json_sha256;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::materials::{Materials, hash};
use crate::model::{digest, nonempty};
use crate::{
    Command, DeliveryGrant, Expected, MaterialRef, ObjectKey, Record, RecordData, Reference,
    Result, RoomKind, Scope, StoreError, TrustedActor, Version, WriterGeneration,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InboxEntry {
    pub binding: Reference,
    pub message_key: String,
    pub digest: String,
}

impl InboxEntry {
    fn key(&self, consumer: &ObjectKey) -> Result<String> {
        self.binding.validate()?;
        nonempty(&self.message_key, "inbox message key")?;
        digest(&self.digest)?;
        // The same provider event can legitimately be processed by independent Project targets.
        Ok(serde_json::to_string(&(&self.binding.key, consumer))?)
    }
    pub(crate) fn previous(
        &self,
        conn: &Connection,
        consumer: &ObjectKey,
    ) -> Result<Option<Value>> {
        let found: Option<(String, String)> = conn
            .query_row(
                "SELECT i.digest,c.result FROM inbox i
                 JOIN commands c ON c.idempotency_key=i.command_key
                 WHERE i.source_key=?1 AND i.message_key=?2",
                params![self.key(consumer)?, self.message_key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        match found {
            Some((digest, result)) if digest == self.digest => {
                Ok(Some(serde_json::from_str(&result)?))
            }
            Some(_) => Err(StoreError::new(
                "INBOX_CONFLICT",
                "same input record key with different digest",
                "reconcile_source",
            )),
            None => Ok(None),
        }
    }
    pub(crate) fn insert(
        &self,
        tx: &Transaction<'_>,
        command: &str,
        consumer: &ObjectKey,
    ) -> Result<()> {
        tx.execute(
            "INSERT INTO inbox(source_key,message_key,digest,binding,command_key) VALUES(?1,?2,?3,?4,?5)",
            params![self.key(consumer)?, self.message_key, self.digest, serde_json::to_string(&self.binding)?, command],
        )?;
        Ok(())
    }
}

/// Adapter conflict scopes are frozen at admission, shared by all operations on that resource.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectIntent {
    pub intent_id: String,
    pub owner: Reference,
    pub binding: Reference,
    pub operation: String,
    pub target: String,
    pub conflict_scope: String,
    pub permission_scope: Scope,
    pub input: Value,
    pub input_digest: String,
    pub idempotency_key: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectState {
    Pending,
    Unknown,
    Confirmed,
}

/// A caller's verified observation of the original target, never an instruction to resend.
pub enum Readback {
    Unknown,
    Confirmed {
        binding: Reference,
        target: String,
        input_digest: String,
        result: Value,
    },
}

pub struct CommandTransaction<'a> {
    pub(crate) tx: &'a Transaction<'a>,
    pub(crate) failed: bool,
    pub(crate) materials: &'a Materials,
    pub(crate) control_id: &'a str,
    pub(crate) command: &'a Command,
    pub(crate) actor: &'a TrustedActor,
    pub(crate) generation: WriterGeneration,
}

impl CommandTransaction<'_> {
    fn apply<T>(&mut self, operation: impl FnOnce(&Self) -> Result<T>) -> Result<T> {
        let result = operation(self);
        self.failed |= result.is_err();
        result
    }
    pub fn put(&mut self, record: &Record) -> Result<()> {
        self.apply(|tx| tx.put_inner(record))
    }
    pub fn admit_material(&mut self, reference: &MaterialRef) -> Result<()> {
        self.apply(|tx| tx.admit_material_inner(reference))
    }
    pub fn enqueue_effect(&mut self, intent: &EffectIntent) -> Result<()> {
        self.apply(|tx| tx.enqueue_effect_inner(intent))
    }
    pub fn confirm_effect(&mut self, id: &str, readback: &Readback) -> Result<()> {
        self.apply(|tx| tx.confirm_effect_inner(id, readback))
    }
    pub fn enqueue_delivery(&mut self, id: &str, grant: &DeliveryGrant) -> Result<()> {
        self.apply(|tx| tx.enqueue_delivery_inner(id, grant))
    }
    pub fn confirm_delivery(
        &mut self,
        id: &str,
        recipient: &str,
        purpose: &str,
        received: &[Vec<u8>],
    ) -> Result<()> {
        self.apply(|tx| tx.confirm_delivery_inner(id, recipient, purpose, received))
    }
    pub fn bind_secret(&mut self, binding: &Reference, secret_reference: &str) -> Result<()> {
        self.apply(|tx| tx.bind_secret_inner(binding, secret_reference))
    }

    pub fn get(&self, key: &ObjectKey) -> Result<Option<Record>> {
        get_record(self.tx, key)
    }

    /// Append a reducer-validated immutable snapshot, with links and admitted material refs.
    /// Domain transitions stay with the consuming module; this enforces version/identity invariants.
    fn put_inner(&self, record: &Record) -> Result<()> {
        self.actor.permits(&record.key.scope)?;
        record.validate()?;
        for reference in &record.materials {
            let stored: Option<String> = self
                .tx
                .query_row(
                    "SELECT reference FROM materials WHERE material_id=?1",
                    [&reference.material_id],
                    |r| r.get(0),
                )
                .optional()?;
            if stored
                .as_deref()
                .map(serde_json::from_str::<MaterialRef>)
                .transpose()?
                .as_ref()
                != Some(reference)
            {
                return Err(StoreError::new(
                    "MATERIAL_NOT_ADMITTED",
                    "record requires a saved and admitted material",
                    "admit_material",
                ));
            }
            self.actor.permits(&reference.scope)?;
            self.materials.read(reference)?;
        }
        project_record(self.tx, record)?;
        self.tx.execute(
            "INSERT INTO events(command_key,object_key,version,record) VALUES(?1,?2,?3,?4)",
            params![
                self.command.idempotency_key,
                record.key.encoded()?,
                record.version,
                serde_json::to_string(record)?
            ],
        )?;
        Ok(())
    }

    /// Candidate bytes are rechecked inside admission; saving alone never inserts this row.
    fn admit_material_inner(&self, reference: &MaterialRef) -> Result<()> {
        self.actor.permits(&reference.scope)?;
        if reference.control_id != self.control_id {
            return Err(StoreError::invalid("material belongs to another control"));
        }
        self.materials.read(reference)?;
        let existing: Option<String> = self
            .tx
            .query_row(
                "SELECT reference FROM materials WHERE material_id=?1",
                [&reference.material_id],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(existing) = existing {
            if serde_json::from_str::<MaterialRef>(&existing)? == *reference {
                return Ok(());
            }
            return Err(StoreError::new(
                "MATERIAL_DIGEST_MISMATCH",
                "admitted material reference differs",
                "restore_material",
            ));
        }
        if self.materials.command_key(reference)? != self.command.idempotency_key {
            return Err(StoreError::invalid(
                "candidate was saved for another command",
            ));
        }
        self.tx.execute(
            "INSERT INTO materials VALUES(?1,?2,?3)",
            params![
                reference.material_id,
                serde_json::to_string(reference)?,
                self.command.idempotency_key
            ],
        )?;
        Ok(())
    }

    fn enqueue_effect_inner(&self, intent: &EffectIntent) -> Result<()> {
        self.actor.permits(&intent.permission_scope)?;
        intent.owner.validate()?;
        intent.binding.validate()?;
        for field in [
            &intent.intent_id,
            &intent.operation,
            &intent.target,
            &intent.conflict_scope,
            &intent.idempotency_key,
        ] {
            nonempty(field, "effect field")?;
        }
        if intent.owner.key.scope != intent.permission_scope {
            return Err(StoreError::invalid(
                "effect owner and permission scope disagree",
            ));
        }
        if Command::digest_input(&intent.operation, &intent.input)? != intent.input_digest {
            return Err(StoreError::invalid("effect canonical input mismatch"));
        }
        // The adapter's conflict scope is already fully qualified; Project/binding revision
        // must not split a single external resource's conflict range.
        if self
            .tx
            .prepare("SELECT 1 FROM outbox WHERE conflict_key=?1 AND state!='confirmed'")?
            .exists([&intent.conflict_scope])?
        {
            return Err(StoreError::new(
                "EFFECT_CONFLICT",
                "an unresolved effect occupies this resource",
                "read_back_original_intent",
            ));
        }
        self.tx.execute(
            "INSERT INTO outbox VALUES(?1,?2,?3,?4,'pending',?5,NULL)",
            params![
                intent.intent_id,
                self.command.idempotency_key,
                serde_json::to_string(intent)?,
                intent.conflict_scope,
                self.generation.0
            ],
        )?;
        Ok(())
    }

    /// Call in the same command transaction that writes any module Receipt/result event.
    fn confirm_effect_inner(&self, id: &str, readback: &Readback) -> Result<()> {
        let (intent, state) = effect(self.tx, id)?;
        self.actor.permits(&intent.permission_scope)?;
        match readback {
            Readback::Unknown => {
                if state != EffectState::Confirmed {
                    self.tx
                        .execute("UPDATE outbox SET state='unknown' WHERE intent_id=?1", [id])?;
                }
            }
            Readback::Confirmed {
                binding,
                target,
                input_digest,
                result,
            } => {
                if *binding != intent.binding
                    || *target != intent.target
                    || *input_digest != intent.input_digest
                {
                    return Err(StoreError::new(
                        "READBACK_MISMATCH",
                        "readback is not for the frozen effect",
                        "read_back_original_intent",
                    ));
                }
                if state == EffectState::Pending {
                    return Err(StoreError::invalid("effect has never entered delivery"));
                }
                let encoded = serde_json::to_string(result)?;
                if state == EffectState::Confirmed {
                    let old: String = self.tx.query_row(
                        "SELECT confirmation FROM outbox WHERE intent_id=?1",
                        [id],
                        |r| r.get(0),
                    )?;
                    if canonical_json_sha256(&serde_json::from_str(&old)?)?
                        != canonical_json_sha256(result)?
                    {
                        return Err(StoreError::new(
                            "READBACK_MISMATCH",
                            "confirmed result is immutable",
                            "inspect_readback",
                        ));
                    }
                } else {
                    self.tx.execute(
                        "UPDATE outbox SET state='confirmed',confirmation=?1 WHERE intent_id=?2",
                        params![encoded, id],
                    )?;
                }
            }
        }
        Ok(())
    }

    fn enqueue_delivery_inner(&self, id: &str, grant: &DeliveryGrant) -> Result<()> {
        nonempty(id, "delivery ID")?;
        nonempty(&grant.recipient, "recipient")?;
        nonempty(&grant.purpose, "purpose")?;
        if grant.materials.is_empty() {
            return Err(StoreError::invalid("empty material grant"));
        }
        for material in &grant.materials {
            self.actor.permits(&material.scope)?;
            let saved: String = self.tx.query_row(
                "SELECT reference FROM materials WHERE material_id=?1",
                [&material.material_id],
                |r| r.get(0),
            )?;
            if serde_json::from_str::<MaterialRef>(&saved)? != *material {
                return Err(StoreError::invalid(
                    "grant requires exact admitted material",
                ));
            }
            self.materials.read(material)?;
        }
        self.tx.execute(
            "INSERT INTO deliveries VALUES(?1,?2,?3,'pending')",
            params![
                id,
                serde_json::to_string(grant)?,
                self.command.idempotency_key
            ],
        )?;
        Ok(())
    }

    /// The delivery adapter supplies the actual recipient's readback bytes, not just sent digests.
    fn confirm_delivery_inner(
        &self,
        id: &str,
        recipient: &str,
        purpose: &str,
        received: &[Vec<u8>],
    ) -> Result<()> {
        let grant: String = self.tx.query_row(
            "SELECT grant_json FROM deliveries WHERE delivery_id=?1",
            [id],
            |r| r.get(0),
        )?;
        let grant: DeliveryGrant = serde_json::from_str(&grant)?;
        if recipient != grant.recipient
            || purpose != grant.purpose
            || received.len() != grant.materials.len()
        {
            return Err(StoreError::invalid(
                "delivery acknowledgement does not match grant",
            ));
        }
        for (bytes, material) in received.iter().zip(&grant.materials) {
            self.actor.permits(&material.scope)?;
            if hash(bytes) != material.byte_digest {
                return Err(StoreError::new(
                    "MATERIAL_DIGEST_MISMATCH",
                    "recipient readback bytes differ",
                    "retry_delivery",
                ));
            }
        }
        self.tx.execute(
            "UPDATE deliveries SET state='confirmed' WHERE delivery_id=?1",
            [id],
        )?;
        Ok(())
    }

    /// Only a secret-store reference is accepted; secret values remain outside records and backups.
    fn bind_secret_inner(&self, binding: &Reference, secret_reference: &str) -> Result<()> {
        binding.validate()?;
        self.actor.permits(&binding.key.scope)?;
        nonempty(secret_reference, "secret reference")?;
        self.tx.execute(
            "INSERT INTO secret_references VALUES(?1,?2)
             ON CONFLICT(binding_key) DO UPDATE SET secret_reference=excluded.secret_reference",
            params![serde_json::to_string(binding)?, secret_reference],
        )?;
        Ok(())
    }
}

pub(crate) fn get_record(conn: &Connection, key: &ObjectKey) -> Result<Option<Record>> {
    let record: Option<String> = conn
        .query_row(
            "SELECT record FROM objects WHERE object_key=?1",
            [key.encoded()?],
            |r| r.get(0),
        )
        .optional()?;
    Ok(record.map(|r| serde_json::from_str(&r)).transpose()?)
}

pub(crate) fn check_expected(
    conn: &Connection,
    key: &ObjectKey,
    expected: &Expected,
) -> Result<()> {
    let record = get_record(conn, key)?;
    let matches = match (expected, record) {
        (Expected::Absent, None) => true,
        (Expected::Exact(Version::State(version)), Some(r)) => *version == r.version,
        (Expected::Exact(Version::Revision(digest)), Some(r)) => *digest == r.revision_digest,
        _ => false,
    };
    if matches {
        Ok(())
    } else {
        Err(StoreError::new(
            "VERSION_CONFLICT",
            "target changed since authorization",
            "preview_current_version",
        ))
    }
}

pub(crate) fn project_record(conn: &Connection, record: &Record) -> Result<()> {
    record.validate()?;
    let old = get_record(conn, &record.key)?;
    if record.version != old.as_ref().map_or(1, |r| r.version + 1) {
        return Err(StoreError::new(
            "VERSION_CONFLICT",
            "record version is not the next version",
            "preview_current_version",
        ));
    }
    if let Some(old) = &old {
        match (&old.data, &record.data) {
            (RecordData::Project { repo_id: a, .. }, RecordData::Project { repo_id: b, .. })
                if a != b =>
            {
                return Err(StoreError::invalid("Project Repo is immutable"));
            }
            (RecordData::Room { room_kind: a, .. }, RecordData::Room { room_kind: b, .. })
                if a != b =>
            {
                return Err(StoreError::invalid("Room kind is immutable"));
            }
            (
                RecordData::Task {
                    entity: Some(a), ..
                },
                RecordData::Task { entity: b, .. },
            ) if Some(a) != b.as_ref() => {
                return Err(StoreError::invalid("Task entity identity is immutable"));
            }
            _ => {}
        }
    }
    let project = if let Scope::Project(id) = &record.key.scope {
        Some(id.as_str())
    } else {
        None
    };
    let room_kind = match &record.data {
        RecordData::Room {
            room_kind: RoomKind::Main,
            ..
        } => Some("main"),
        RecordData::Room {
            room_kind: RoomKind::Topic,
            ..
        } => Some("topic"),
        _ => None,
    };
    let entity = match &record.data {
        RecordData::Task {
            entity: Some(entity),
            ..
        } => Some(canonical_json_sha256(&serde_json::to_value(entity)?)?),
        _ => None,
    };
    let conflict: Option<String> = conn
        .query_row(
            "SELECT object_key FROM objects WHERE object_key!=?1 AND project_id=?2 AND
         ((?3='main' AND kind='room' AND room_kind='main') OR
          (?4 IS NOT NULL AND kind='task' AND entity_key=?4)) LIMIT 1",
            params![record.key.encoded()?, project, room_kind, entity],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(existing) = conflict {
        return Err(StoreError::new(
            "UNIQUENESS_CONFLICT",
            format!("Project already has this main Room or entity Task: {existing}"),
            "preview_existing_object",
        ));
    }
    conn.execute(
        "INSERT INTO objects VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)
         ON CONFLICT(object_key) DO UPDATE SET version=excluded.version,
         revision_digest=excluded.revision_digest,record=excluded.record,
         room_kind=excluded.room_kind,entity_key=excluded.entity_key",
        params![
            record.key.encoded()?,
            record.key.kind,
            serde_json::to_string(&record.key.scope)?,
            record.version,
            record.revision_digest,
            serde_json::to_string(record)?,
            project,
            room_kind,
            entity,
        ],
    )?;
    Ok(())
}

pub(crate) fn effect(conn: &Connection, id: &str) -> Result<(EffectIntent, EffectState)> {
    let (intent, state): (String, String) = conn.query_row(
        "SELECT intent,state FROM outbox WHERE intent_id=?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let state = match state.as_str() {
        "pending" => EffectState::Pending,
        "unknown" => EffectState::Unknown,
        "confirmed" => EffectState::Confirmed,
        _ => return Err(StoreError::invalid("invalid effect state")),
    };
    Ok((serde_json::from_str(&intent)?, state))
}

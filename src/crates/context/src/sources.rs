//! Source adapters. Room lines read the frozen `chat_source_reference`
//! records (with their admitted material bytes); Task-comment lines read the
//! selected Task's comments from its complete Task Backend Snapshot; the platform review-comment line is
//! package 6's wiring and reports not-configured until then.

use crate::{SourceContent, SourceKind, Sources};
use agency_proto::{FrozenRef, PortError, Result, hash};
use store::{Record, Scope, Store, TrustedActor};

/// Store errors surface as port errors with the same codes.
pub(crate) fn store_call<T>(
    call: impl FnOnce() -> std::result::Result<T, store::StoreError>,
) -> Result<T> {
    call().map_err(|e| PortError::new(e.code, e.message.clone(), e.recovery_action))
}

/// FrozenRef.id packing: "kind/id" so one reference names its record.
pub fn pack_reference(kind: &str, id: &str) -> String {
    format!("{kind}/{id}")
}

fn split_reference(packed: &str) -> Result<(String, String)> {
    packed
        .split_once('/')
        .map(|(kind, id)| (kind.to_owned(), id.to_owned()))
        .ok_or_else(|| PortError::invalid("reference must be kind/id"))
}

/// Build the packed FrozenRef for an admitted record: id = kind/id, digest =
/// the record's canonical data digest, revision = the record's revision digest.
pub fn frozen_from_record(record: &Record) -> Result<FrozenRef> {
    let packed = pack_reference(&record.key.kind, &record.key.id);
    let value = serde_json::to_value(&record.data)?;
    let digest = foundation::canonical_json_sha256(&value)
        .map_err(|e| PortError::invalid(format!("canonical JSON: {e}")))?;
    Ok(FrozenRef {
        digest,
        id: packed,
        revision: record.revision_digest.clone(),
    })
}

/// The real record kinds this package reads.
pub const ROOM_LINE_KIND: &str = "chat_source_reference";
pub const ROOM_BRIEF_KIND: &str = "room_binding";
pub const TASK_LINE_KIND: &str = "task_comments";

/// Reuse the authenticated owner's Project access used by chat commands.
/// Other trusted callers must already carry that Project's permission scope.
pub(crate) fn project_actor(actor: &TrustedActor, project: &str) -> Result<TrustedActor> {
    if actor
        .0
        .permission_scope
        .contains(&Scope::Project(project.into()))
    {
        return Ok(TrustedActor(actor.0.clone()));
    }
    store_call(|| chat::owner(actor, project))
}

/// Adapter over the control-plane store, scoped to one project.
pub struct StoreSources<'a> {
    store: &'a Store,
    actor: &'a TrustedActor,
    project: String,
}

impl<'a> StoreSources<'a> {
    pub fn new(store: &'a Store, actor: &'a TrustedActor, project: &str) -> Self {
        Self {
            store,
            actor,
            project: project.to_owned(),
        }
    }

    fn room_line(&self, reference: &FrozenRef) -> Result<SourceContent> {
        let actor = project_actor(self.actor, &self.project)?;
        let (kind, id) = split_reference(&reference.id)?;
        if !matches!(kind.as_str(), ROOM_LINE_KIND | ROOM_BRIEF_KIND) {
            return Err(PortError::invalid("not a Room source reference"));
        }
        let key = store::ObjectKey {
            scope: Scope::Project(self.project.clone()),
            kind: kind.clone(),
            id,
        };
        let record = store_call(|| self.store.get(&key))?.ok_or_else(|| {
            PortError::new(
                "SOURCE_UNAVAILABLE",
                "source record missing",
                "refresh_source",
            )
        })?;
        verify_version(&record, reference)?;
        let material = if kind == ROOM_BRIEF_KIND {
            let room: chat::Room = store_call(|| chat::decode(&record))?;
            room.brief
        } else {
            record.materials.first().cloned()
        }
        .ok_or_else(|| {
            PortError::new(
                "SOURCE_UNAVAILABLE",
                "admitted material missing",
                "restore_material",
            )
        })?;
        if material.scope != record.key.scope {
            return Err(PortError::invalid("Room material belongs to another scope"));
        }
        let bytes = store_call(|| self.store.read_material(&actor, &material))?;
        Ok(SourceContent {
            reference: reference.clone(),
            bytes,
        })
    }

    pub(crate) fn task_line(&self, id: &str) -> Result<SourceContent> {
        project_actor(self.actor, &self.project)?;
        let project = store_call(|| {
            self.store.get(&task::key(
                Scope::Project(self.project.clone()),
                "project",
                &self.project,
            ))
        })?
        .ok_or_else(|| PortError::new("SOURCE_UNAVAILABLE", "Project missing", "refresh_source"))?;
        let (record, task) = store_call(|| task::task(self.store, &self.project, id))?;
        let store::RecordData::Project { repo_id, .. } = &project.data else {
            return Err(PortError::invalid("invalid Project record"));
        };
        if task.project_id != self.project || task.repo_id != *repo_id || task.id != id {
            return Err(PortError::new(
                "PERMISSION_DENIED",
                "Task belongs to a different Project or Repo",
                "request_authorization",
            ));
        }
        let (source_record, source) =
            store_call(|| task::source(self.store, repo_id, &task.source_id))?;
        let approval_record = store_call(|| {
            task::required(
                self.store,
                &task::key(
                    Scope::Project(self.project.clone()),
                    "task_source_reference",
                    &task.source_id,
                ),
            )
        })?;
        let approval: task::SourceReference = store_call(|| task::decode(&approval_record))?;
        if approval.project_id != self.project
            || approval.source.key != source_record.key
            || approval.approved_scope != source.board_scope_stable_id
            || source.repo_id != *repo_id
            || source.id != task.source_id
        {
            return Err(PortError::new(
                "PERMISSION_DENIED",
                "Task source is not approved for this Project",
                "request_authorization",
            ));
        }
        let (snapshot_record, snapshot) = store_call(|| task::latest(self.store, &source))?;
        let entity = task.entity.as_ref().ok_or_else(|| {
            PortError::new(
                "SOURCE_UNAVAILABLE",
                "Task has no bound card",
                "refresh_source",
            )
        })?;
        let card = snapshot
            .cards
            .iter()
            .find(|card| &card.entity == entity && !card.tombstone)
            .ok_or_else(|| {
                PortError::new(
                    "SOURCE_UNAVAILABLE",
                    "bound card is not readable in the complete Snapshot",
                    "refresh_source",
                )
            })?;
        // Keep the exact Snapshot locator, version and digest inspectable in
        // the projection revision; a hash alone would lose its source chain.
        let identity = serde_json::json!({
            "task": source_version(&record)?,
            "snapshot": source_version(&snapshot_record)?,
            "approval": source_version(&approval_record)?,
            "binding": source_version(&source_record)?
        });
        let revision = String::from_utf8(
            foundation::canonical_json(&identity).map_err(|e| PortError::invalid(e.to_string()))?,
        )
        .map_err(|e| PortError::invalid(e.to_string()))?;
        let bytes = foundation::canonical_json(&serde_json::to_value(&card.comments)?)
            .map_err(|e| PortError::invalid(e.to_string()))?;
        let digest = foundation::canonical_json_sha256(
            &serde_json::json!({"identity":identity,"comments":card.comments}),
        )
        .map_err(|e| PortError::invalid(e.to_string()))?;
        Ok(SourceContent {
            reference: FrozenRef {
                id: pack_reference(TASK_LINE_KIND, id),
                revision,
                digest,
            },
            bytes,
        })
    }
}

fn source_version(record: &Record) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "reference": task::reference(record),
        "digest": frozen_from_record(record)?.digest
    }))
}

fn verify_version(record: &Record, reference: &FrozenRef) -> Result<()> {
    let value = serde_json::to_value(&record.data)?;
    let current = foundation::canonical_json_sha256(&value)
        .map_err(|e| PortError::invalid(format!("canonical JSON: {e}")))?;
    if current != reference.digest || record.revision_digest != reference.revision {
        return Err(PortError::new(
            "SOURCE_VERSION_CHANGED",
            "source revision moved since the preview",
            "preview_again",
        ));
    }
    Ok(())
}

impl Sources for StoreSources<'_> {
    fn exact(&self, kind: SourceKind, reference: &FrozenRef) -> Result<SourceContent> {
        match kind {
            SourceKind::Room => self.room_line(reference),
            SourceKind::TaskComments => {
                let (kind, id) = split_reference(&reference.id)?;
                if kind != TASK_LINE_KIND {
                    return Err(PortError::invalid(
                        "Task comments require a Project-local Task id, not a bare Snapshot id",
                    ));
                }
                let source = self.task_line(&id)?;
                if source.reference != *reference {
                    return Err(PortError::new(
                        "SOURCE_VERSION_CHANGED",
                        "Task or Snapshot changed since selection",
                        "preview_again",
                    ));
                }
                Ok(source)
            }
            SourceKind::ReviewComments => Err(PortError::new(
                "REVIEW_LINE_NOT_CONFIGURED",
                "the platform review-comment line is wired in package 6",
                "wait_for_review_wiring",
            )),
        }
    }
}

pub fn permission_digest(permitted: &[String]) -> String {
    let mut ids: Vec<&str> = permitted.iter().map(String::as_str).collect();
    ids.sort_unstable();
    hash(ids.join("\0").as_bytes())
}

/// In-memory adapter for tests.
#[derive(Default)]
pub struct MemorySources {
    pub entries: Vec<(SourceKind, FrozenRef, Vec<u8>)>,
}

impl MemorySources {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, kind: SourceKind, reference: FrozenRef, bytes: &[u8]) {
        self.entries.push((kind, reference, bytes.to_vec()));
    }
}

impl Sources for MemorySources {
    fn exact(&self, kind: SourceKind, reference: &FrozenRef) -> Result<SourceContent> {
        self.entries
            .iter()
            .find(|(entry_kind, entry, _)| *entry_kind == kind && entry == reference)
            .map(|(_, entry, bytes)| SourceContent {
                reference: entry.clone(),
                bytes: bytes.clone(),
            })
            .ok_or_else(|| PortError::new("SOURCE_UNAVAILABLE", "source missing", "refresh_source"))
    }
}

/// The standalone review-line stub (kept for direct tests); the store adapter
/// answers the same typed error through `Sources::exact`.
pub struct ReviewComments;

impl Sources for ReviewComments {
    fn exact(&self, _kind: SourceKind, _reference: &FrozenRef) -> Result<SourceContent> {
        Err(PortError::new(
            "REVIEW_LINE_NOT_CONFIGURED",
            "the platform review-comment line is wired in package 6",
            "wait_for_review_wiring",
        ))
    }
}

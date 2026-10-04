//! Source adapters. Room lines read the frozen `chat_source_reference`
//! records (with their admitted material bytes); Task-comment lines read the
//! frozen Task Backend Snapshot records; the platform review-comment line is
//! package 6's wiring and reports not-configured until then.

use crate::{SourceContent, SourceKind, Sources};
use agency_proto::context::Manifest;
use agency_proto::{FrozenRef, Owner, PortError, Result, hash};
use store::{Record, Scope, Store, TrustedActor};

/// Store errors surface as port errors with the same codes.
fn store_call<T>(call: impl FnOnce() -> std::result::Result<T, store::StoreError>) -> Result<T> {
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
pub const TASK_SNAPSHOT_KIND: &str = "task_snapshot";

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
        let (_, id) = split_reference(&reference.id)?;
        let key = store::ObjectKey {
            scope: Scope::Project(self.project.clone()),
            kind: ROOM_LINE_KIND.into(),
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
        let material = record.materials.first().ok_or_else(|| {
            PortError::new(
                "SOURCE_UNAVAILABLE",
                "admitted material missing",
                "restore_material",
            )
        })?;
        let bytes = store_call(|| self.store.read_material(self.actor, material))?;
        Ok(SourceContent {
            reference: reference.clone(),
            bytes,
        })
    }

    fn task_snapshot(&self, reference: &FrozenRef) -> Result<SourceContent> {
        let (_, id) = split_reference(&reference.id)?;
        // Snapshots live under the Repo scope; find by kind+id across scopes.
        let record = store_call(|| self.store.list(TASK_SNAPSHOT_KIND))?
            .into_iter()
            .find(|record| record.key.id == id)
            .ok_or_else(|| {
                PortError::new(
                    "SOURCE_UNAVAILABLE",
                    "task snapshot missing",
                    "refresh_source",
                )
            })?;
        verify_version(&record, reference)?;
        // The snapshot record carries no material; its canonical data JSON is
        // the delivered bytes.
        let bytes = foundation::canonical_json(&serde_json::to_value(&record.data)?)
            .map_err(|e| PortError::invalid(format!("canonical JSON: {e}")))?;
        Ok(SourceContent {
            reference: reference.clone(),
            bytes,
        })
    }
}

fn verify_version(record: &Record, reference: &FrozenRef) -> Result<()> {
    let value = serde_json::to_value(&record.data)?;
    let current = foundation::canonical_json_sha256(&value)
        .map_err(|e| PortError::invalid(format!("canonical JSON: {e}")))?;
    if current != reference.digest {
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
            SourceKind::TaskComments => self.task_snapshot(reference),
            SourceKind::ReviewComments => Err(PortError::new(
                "REVIEW_LINE_NOT_CONFIGURED",
                "the platform review-comment line is wired in package 6",
                "wait_for_review_wiring",
            )),
        }
    }
}

/// The permission policy point placeholder: every source this project has
/// admitted. Real permission policy lands with the dispatch package; until
/// then the assembler's gate uses this set, never the caller's input file.
pub fn permitted_source_ids(store: &Store, project: &str) -> Result<Vec<String>> {
    let records = store_call(|| store.list(ROOM_LINE_KIND))?
        .into_iter()
        .filter(|record| record.key.scope == Scope::Project(project.to_owned()))
        .map(|record| Ok(pack_reference(ROOM_LINE_KIND, &record.key.id)))
        .collect::<Result<Vec<_>>>()?;
    Ok(records)
}

/// Mechanical selection from governance records only (no small-brain, no
/// model): the frozen room-line sources of one Room become the Manifest.
/// This is the narrowest real path; richer selection order (explicit user
/// references, current discussion window, artifacts) joins in later packages.
pub fn select_room_manifest(
    store: &Store,
    actor: &TrustedActor,
    project: &str,
    room_id: &str,
    budget: u64,
) -> Result<(Manifest, Owner)> {
    let _ = actor;
    if budget == 0 {
        return Err(PortError::invalid("budget must be positive"));
    }
    let records = store_call(|| store.list(ROOM_LINE_KIND))?;
    let mut sources = Vec::new();
    for record in records
        .into_iter()
        .filter(|record| record.key.scope == Scope::Project(project.to_owned()))
    {
        // Keep only message sources frozen against this Room's binding.
        // chat stores source records as Value envelopes; unwrap before reading.
        let data = serde_json::to_value(&record.data)?;
        let source = data.get("value").unwrap_or(&data);
        if source["kind"] != "message" {
            continue;
        }
        if source["binding"]["key"]["id"].as_str() != Some(room_id) {
            continue;
        }
        sources.push(frozen_from_record(&record)?);
    }
    if sources.is_empty() {
        return Err(PortError::new(
            "SOURCE_UNAVAILABLE",
            "the Room has no frozen admitted sources",
            "open_a_topic_first",
        ));
    }
    let permitted = permitted_source_ids(store, project)?;
    let manifest = Manifest {
        id: format!(
            "manifest-{}",
            hash(format!("{project}:{room_id}:{budget}").as_bytes())
        ),
        purpose: "deliver the frozen room line".into(),
        scope: format!("project {project} room {room_id}"),
        parent: None,
        sources,
        selection_policy: mechanical_policy_ref(),
        freshness: "as of the last admitted room-line record".into(),
        coverage: "frozen room-line sources admitted for this Room".into(),
        known_gaps: vec![
            "current discussion window is not in governance records".into(),
            "platform review-comment line arrives with package 6".into(),
        ],
        required_skills: vec![],
        permission_digest: permission_digest(&permitted),
        redaction: default_redaction_ref(),
        budget,
    };
    Ok((manifest, placeholder_consumer(project)))
}

fn mechanical_policy_ref() -> FrozenRef {
    FrozenRef {
        id: "policy/mechanical-v1".into(),
        revision: "1".into(),
        digest: hash(b"hctl2.context.mechanical.v1"),
    }
}

fn default_redaction_ref() -> FrozenRef {
    FrozenRef {
        id: "redaction/none".into(),
        revision: "1".into(),
        digest: hash(b"hctl2.context.redaction.none.v1"),
    }
}

fn placeholder_consumer(project: &str) -> Owner {
    Owner {
        project: project.to_owned(),
        kind: agency_proto::OwnerKind::RoomInvocation,
        id: "preview".into(),
        generation: 1,
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

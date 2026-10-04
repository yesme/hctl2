//! Source adapters. Room and Task-comment lines read from the control plane's
//! frozen records and admitted materials; the platform review-comment line is
//! package 6's wiring and reports not-configured until then.

use crate::{SourceContent, SourceKind, Sources};
use agency_proto::{FrozenRef, PortError, Result};
use store::{Record, Scope, Store, TrustedActor};

/// Adapter over the control-plane store. Room lines come from admitted chat
/// source references and their material bytes; Task-comment lines come from
/// frozen Task Backend Snapshot records. Nothing here reads live chat servers:
/// a preview must be reproducible from governance records alone.
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

    fn record_at(&self, reference: &FrozenRef) -> Result<Record> {
        // A FrozenRef points at (kind, id); the store key carries the scope.
        let scope = Scope::Project(self.project.clone());
        let key = store::ObjectKey {
            scope: scope.clone(),
            kind: reference_kind(&reference.id)?,
            id: reference_id(&reference.id)?,
        };
        let record = store_call(|| self.store.get(&key))?.ok_or_else(|| {
            PortError::new(
                "SOURCE_UNAVAILABLE",
                "source record missing",
                "refresh_source",
            )
        })?;
        // The frozen revision must match the record's current digest.
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
        Ok(record)
    }

    fn material_bytes(&self, record: &Record) -> Result<Vec<u8>> {
        let material = record.materials.first().ok_or_else(|| {
            PortError::new(
                "SOURCE_UNAVAILABLE",
                "admitted material missing",
                "restore_material",
            )
        })?;
        store_call(|| self.store.read_material(self.actor, material))
    }
}

/// FrozenRef.id packing: "kind/id" so one reference names its record.
fn reference_kind(packed: &str) -> Result<String> {
    packed
        .split_once('/')
        .map(|(kind, _)| kind.to_owned())
        .ok_or_else(|| PortError::invalid("reference must be kind/id"))
}
fn reference_id(packed: &str) -> Result<String> {
    packed
        .split_once('/')
        .map(|(_, id)| id.to_owned())
        .ok_or_else(|| PortError::invalid("reference must be kind/id"))
}
pub fn pack_reference(kind: &str, id: &str) -> String {
    format!("{kind}/{id}")
}

impl Sources for StoreSources<'_> {
    fn exact(&self, kind: SourceKind, reference: &FrozenRef) -> Result<SourceContent> {
        let _ = kind;
        let record = self.record_at(reference)?;
        let bytes = self.material_bytes(&record)?;
        Ok(SourceContent {
            reference: reference.clone(),
            bytes,
        })
    }
}

/// In-memory adapter for tests and for the review-comment stub.
pub struct MemorySources {
    pub entries: Vec<(SourceKind, FrozenRef, Vec<u8>)>,
}

impl MemorySources {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }
    pub fn with(kind: SourceKind, reference: FrozenRef, bytes: &[u8]) -> Self {
        Self {
            entries: vec![(kind, reference, bytes.to_vec())],
        }
    }
    pub fn push(&mut self, kind: SourceKind, reference: FrozenRef, bytes: &[u8]) {
        self.entries.push((kind, reference, bytes.to_vec()));
    }
}

impl Default for MemorySources {
    fn default() -> Self {
        Self::new()
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

/// Platform review-comment line: package 6 wires the real adapter. Until then
/// any request is a typed not-configured error, never an empty success.
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

/// Convert a store Reference (version + digest) into the packed FrozenRef the
/// Manifest carries.
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

/// Store errors surface as port errors with the same codes.
fn store_call<T>(call: impl FnOnce() -> std::result::Result<T, store::StoreError>) -> Result<T> {
    call().map_err(|e| PortError::new(e.code, e.message.clone(), e.recovery_action))
}

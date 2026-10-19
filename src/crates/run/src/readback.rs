//! Trusted readback port for package 7. Command payloads contain references only.
use crate::{Production, Provenance, SnapshotSource};
use agency_proto::{
    ExecutionSpec, Sealed,
    context::{Bundle, Delivery},
};
use chat::{invalid, reject};
use foundation::{bytes_sha256, canonical_json_sha256};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use store::{Reference, Result};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Readback {
    pub workflow_digest: String,
    pub snapshot_digest: String,
    pub invocation: Reference,
    pub selection: Reference,
    pub bundle_digest: String,
    pub body_digest: String,
}
/// Written by the readback producer after actual dispatch/delivery, never by
/// workflow register/approve. Package 7 supplies this port; package 1 uses fakes.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationEvidence {
    pub selection: Reference,
    pub source_room: Option<String>,
    pub fresh: bool,
    pub spec: Sealed<ExecutionSpec>,
    pub delivered: Sealed<Bundle>,
}
pub struct Producer {
    pub source_room: String,
    pub profile_digest: String,
}
pub trait Readbacks {
    fn readback(&self, reference: &Reference) -> Result<Readback>;
    fn invocation(&self, reference: &Reference) -> Result<InvocationEvidence>;
    fn producer(&self, reference: &Reference) -> Result<Producer> {
        let actual = self.invocation(reference)?;
        actual.spec.verify().map_err(|e| invalid(e.to_string()))?;
        Ok(Producer {
            source_room: actual
                .source_room
                .ok_or_else(|| fail("producer has no source Room"))?,
            profile_digest: actual.spec.document.profile.digest,
        })
    }
    /// Current roster at approval, not registration or last invocation.
    fn roster(&self, room: &str) -> Result<Vec<Reference>>;
}
fn fail(message: &str) -> store::StoreError {
    reject("WORKFLOW_READBACK", message, "produce_isolated_readback")
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
pub fn verify(
    port: &dyn Readbacks,
    reference: &Reference,
    workflow_digest: &str,
    snapshot_digest: &str,
    provenance: &Provenance,
    different_profile: bool,
) -> Result<serde_json::Value> {
    let read = port.readback(reference)?;
    if !digest(&read.body_digest)
        || read.workflow_digest != workflow_digest
        || read.snapshot_digest != snapshot_digest
    {
        return Err(fail("registered and readback digests differ"));
    }
    let actual = port.invocation(&read.invocation)?;
    actual.spec.verify().map_err(|e| invalid(e.to_string()))?;
    actual
        .spec
        .document
        .validate()
        .map_err(|e| invalid(e.to_string()))?;
    actual
        .delivered
        .verify()
        .map_err(|e| invalid(e.to_string()))?;
    actual
        .delivered
        .document
        .validate_delivery()
        .map_err(|e| invalid(e.to_string()))?;
    let spec = &actual.spec.document;
    if !actual.fresh
        || spec.owner.kind != agency_proto::OwnerKind::RoomInvocation
        || actual.selection != read.selection
        || spec.selection.id != actual.selection.key.id
        || actual.delivered.digest != spec.bundle.digest
        || actual.delivered.document.id != spec.bundle.id
        || actual.delivered.digest != read.bundle_digest
        || actual.delivered.document.consumer != spec.owner
        || spec.owner.id != read.invocation.key.id
        || spec.permission_digest != actual.delivered.document.permission_digest
        || spec.manifest != actual.delivered.document.manifest
    {
        return Err(fail(
            "readback must be a fresh no-Run invocation of the frozen delivered Bundle",
        ));
    }
    if spec.permissions.iter().any(|permission| {
        permission
            .split(['.', ':', '/'])
            .any(|part| matches!(part, "room" | "rooms" | "messages" | "*"))
    }) {
        return Err(fail("readback must have no Room message-read permission"));
    }
    let bundle = &actual.delivered.document;
    if bundle.entries.len() != 2 || !bundle.compression.is_empty() {
        return Err(fail(
            "readback Bundle must contain only exact graph and snapshot",
        ));
    }
    let expected = [
        ("workflow_revision", workflow_digest),
        ("workflow_snapshot", snapshot_digest),
    ];
    for (kind, declared) in expected {
        let matches: Vec<_> = bundle
            .entries
            .iter()
            .filter(|e| e.source.id.starts_with(&format!("{kind}:")))
            .collect();
        let [entry] = matches.as_slice() else {
            return Err(fail("graph or snapshot entry missing/duplicated"));
        };
        let bytes = match &entry.delivery {
            Delivery::Inline { bytes } | Delivery::Pointer { bytes, .. } => bytes,
            Delivery::Recall { .. } => return Err(fail("readback material was not delivered")),
        };
        let value: serde_json::Value = serde_json::from_slice(bytes)?;
        // Compare original canonical digests at ALL THREE locations, not Git OIDs
        // or the latest source snapshot. Source refs and delivered bytes both count.
        if entry.source.digest != declared
            || entry.bytes_digest != bytes_sha256(bytes)
            || canonical_json_sha256(&value)? != declared
        {
            return Err(fail(
                "registered, readback and actual delivered original digests differ",
            ));
        }
    }
    let mut rooms: BTreeSet<_> = provenance
        .source_chain
        .iter()
        .map(|s| s.room_id.as_str())
        .collect();
    if let SnapshotSource::Message { room_id, .. } = &provenance.snapshot_source {
        rooms.insert(room_id);
    }
    for room in &rooms {
        if port
            .roster(room)?
            .iter()
            .any(|s| s.key == actual.selection.key)
        {
            return Err(fail(
                "readback producer is in a source Room's current roster",
            ));
        }
    }
    let comparison = if different_profile {
        match &provenance.production {
            Production::Missing {} => return Err(fail("production origin is missing")),
            Production::None {} => "explicit_no_model_producer",
            Production::Invocations { references } => {
                if references.is_empty() {
                    return Err(fail("an empty set is not explicit no-model production"));
                }
                for origin in references {
                    if origin.key.kind != "room_invocation" {
                        return Err(fail("production must reference actual Room Invocations"));
                    }
                    let producer = port.producer(origin)?;
                    if !rooms.contains(producer.source_room.as_str())
                        || producer.profile_digest == spec.profile.digest
                    {
                        return Err(fail("unknown producer or Worker Profile was not changed"));
                    }
                }
                "different_from_all_producers"
            }
        }
    } else {
        "not_requested"
    };
    Ok(
        serde_json::json!({"readback":reference,"profile_comparison":comparison,"source_room_roster":if rooms.is_empty(){"not_applicable"}else{"checked"}}),
    )
}

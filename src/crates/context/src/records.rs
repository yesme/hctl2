//! Manifest / Bundle persistence: frozen documents land as governance records
//! with their canonical bytes in the material store, inside one transaction.

use crate::Assembly;
use agency_proto::{PortError, Result, Sealed};

fn port_call<T>(
    call: impl FnOnce() -> std::result::Result<T, store::StoreError>,
) -> std::result::Result<T, PortError> {
    call().map_err(|e| PortError::new(e.code, e.message, e.recovery_action))
}
use serde_json::Value;
use store::{Reference, Scope, Store, TrustedActor, Version};

/// Save one assembly under a project scope. The manifest and bundle records
/// are append-only: a second save with the same key is a replay that returns
/// the stored result; different content on the same key is a conflict.
pub fn save_assembly(
    store: &mut Store,
    actor: &TrustedActor,
    project: &str,
    key: &str,
    assembly: &Assembly,
) -> Result<Value> {
    assembly.manifest.verify()?;
    assembly.bundle.verify()?;
    let scope = Scope::Project(project.to_owned());
    let manifest_key = store::ObjectKey {
        scope: scope.clone(),
        kind: "context_manifest".into(),
        id: assembly.manifest.document.id.clone(),
    };
    let bundle_key = store::ObjectKey {
        scope: scope.clone(),
        kind: "context_bundle".into(),
        id: assembly.bundle.document.id.clone(),
    };
    // Replay is decided per record by content, not by key existence: the
    // common flow saves one manifest and several per-consumer bundles.
    let mut replayed_manifest = false;
    if let Some(existing) = port_call(|| store.get(&manifest_key))? {
        let existing_digest = sealed_digest(&existing)?;
        if existing_digest == assembly.manifest.digest {
            replayed_manifest = true;
        } else {
            return Err(PortError::new(
                "CONTEXT_CONFLICT",
                format!(
                    "manifest {} already frozen with different content",
                    assembly.manifest.document.id
                ),
                "use_a_new_manifest_id",
            ));
        }
    }
    let replayed_bundle = match port_call(|| store.get(&bundle_key))? {
        Some(existing) => {
            let existing_digest = sealed_digest(&existing)?;
            if existing_digest == assembly.bundle.digest {
                true
            } else {
                return Err(PortError::new(
                    "CONTEXT_CONFLICT",
                    format!(
                        "bundle {} already frozen with different content",
                        assembly.bundle.document.id
                    ),
                    "use_a_new_bundle_id",
                ));
            }
        }
        None => false,
    };
    if replayed_manifest && replayed_bundle {
        return Ok(serde_json::json!({
            "manifest_id": assembly.manifest.document.id,
            "bundle_id": assembly.bundle.document.id,
            "replayed": true
        }));
    }
    let manifest_bytes = serde_json::to_vec(&assembly.manifest)?;
    let bundle_bytes = serde_json::to_vec(&assembly.bundle)?;
    let manifest_material = port_call(|| {
        store.save_material(
            store.generation(),
            actor,
            &scope,
            key,
            "manifest",
            &manifest_bytes,
        )
    })?;
    let bundle_material = port_call(|| {
        store.save_material(
            store.generation(),
            actor,
            &scope,
            key,
            "bundle",
            &bundle_bytes,
        )
    })?;
    let mut manifest_record =
        value_record(&manifest_key, &serde_json::to_value(&assembly.manifest)?)?;
    manifest_record.materials.push(manifest_material.clone());
    let mut bundle_record = value_record(&bundle_key, &serde_json::to_value(&assembly.bundle)?)?;
    bundle_record.materials.push(bundle_material.clone());
    let input = serde_json::json!({
        "key": key,
        "manifest": assembly.manifest.digest,
        "bundle": assembly.bundle.digest
    });
    let command = store::Command {
        command_id: format!("context:{key}"),
        idempotency_key: key.to_owned(),
        actor: actor.0.clone(),
        target: store::ObjectKey {
            scope: scope.clone(),
            kind: "context_command".into(),
            id: key.to_owned(),
        },
        expected: store::Expected::Absent,
        binding: Reference {
            key: store::ObjectKey {
                scope: store::Scope::Control,
                kind: "module".into(),
                id: "context".into(),
            },
            version: Version::State(1),
        },
        input: input.clone(),
        input_digest: store::Command::digest_input("context.assemble", &input)
            .map_err(|e| PortError::new(e.code, e.message, e.recovery_action))?,
        operation: "context.assemble".into(),
    };
    let manifest_record_clone = if replayed_manifest {
        None
    } else {
        Some(manifest_record.clone())
    };
    let result = port_call(|| {
        store.submit(store.generation(), actor, &command, None, |tx| {
            tx.admit_material(&manifest_material)?;
            tx.admit_material(&bundle_material)?;
            if let Some(record) = &manifest_record_clone {
                tx.put(record)?;
            }
            tx.put(&bundle_record)?;
            Ok(serde_json::json!({
                "manifest_id": assembly.manifest.document.id,
                "bundle_id": assembly.bundle.document.id,
                "manifest_digest": assembly.manifest.digest,
                "bundle_digest": assembly.bundle.digest,
                "replayed": false
            }))
        })
    })?;
    Ok(result)
}

pub fn read_manifest(
    store: &Store,
    project: &str,
    manifest_id: &str,
) -> Result<Option<Sealed<agency_proto::context::Manifest>>> {
    let record = manifest_record(store, project, manifest_id)?;
    match record {
        None => Ok(None),
        Some(record) => {
            let value = record_data_value(&record)?;
            Ok(Some(serde_json::from_value(value)?))
        }
    }
}

pub fn read_bundle(
    store: &Store,
    project: &str,
    bundle_id: &str,
) -> Result<Option<Sealed<agency_proto::context::Bundle>>> {
    let key = store::ObjectKey {
        scope: Scope::Project(project.to_owned()),
        kind: "context_bundle".into(),
        id: bundle_id.to_owned(),
    };
    match port_call(|| store.get(&key))? {
        None => Ok(None),
        Some(record) => {
            let value = record_data_value(&record)?;
            Ok(Some(serde_json::from_value(value)?))
        }
    }
}

pub fn manifest_record(
    store: &Store,
    project: &str,
    manifest_id: &str,
) -> Result<Option<store::Record>> {
    let key = store::ObjectKey {
        scope: Scope::Project(project.to_owned()),
        kind: "context_manifest".into(),
        id: manifest_id.to_owned(),
    };
    port_call(|| store.get(&key))
}

pub fn bundle_record(
    store: &Store,
    project: &str,
    bundle_id: &str,
) -> Result<Option<store::Record>> {
    let key = store::ObjectKey {
        scope: Scope::Project(project.to_owned()),
        kind: "context_bundle".into(),
        id: bundle_id.to_owned(),
    };
    port_call(|| store.get(&key))
}

fn sealed_digest(record: &store::Record) -> Result<String> {
    let value = record_data_value(record)?;
    let sealed: Sealed<serde_json::Value> = serde_json::from_value(value)?;
    Ok(sealed.digest)
}

fn record_data_value(record: &store::Record) -> Result<Value> {
    match &record.data {
        store::RecordData::Value { value } => Ok(value.clone()),
        _ => Err(PortError::invalid("context record is not a value record")),
    }
}

fn value_record(key: &store::ObjectKey, data: &Value) -> Result<store::Record> {
    let digest = foundation::canonical_json_sha256(data)
        .map_err(|e| PortError::invalid(format!("canonical JSON: {e}")))?;
    Ok(store::Record {
        key: key.clone(),
        version: 1,
        revision_digest: digest,
        data: store::RecordData::Value {
            value: data.clone(),
        },
        sources: vec![],
        materials: vec![],
    })
}

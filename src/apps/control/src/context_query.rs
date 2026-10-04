//! `context.preview|show`: read-only queries over the local assembler.
//!
//! Preview builds an assembly from a manifest skeleton without saving; show
//! reads a frozen record. Network reads stay outside any transaction: the
//! Store-backed source adapter reads governance records and admitted
//! materials only.

use chat::invalid;
use context::{Assembler, AssemblyRequest, LocalAssembler, Manifest, StoreSources};
use serde_json::{Value, json};
use store::TrustedActor;

type Shared = std::sync::Arc<tokio::sync::Mutex<Option<store::Store>>>;

fn access<T>(
    shared: &Shared,
    f: impl FnOnce(&mut store::Store) -> store::Result<T>,
) -> store::Result<T> {
    let mut slot = shared.blocking_lock();
    f(slot
        .as_mut()
        .ok_or_else(|| invalid("storage is not serving"))?)
}

pub(crate) fn query(
    shared: &Shared,
    actor: &TrustedActor,
    kind: &str,
    payload: &Value,
) -> store::Result<Value> {
    match kind {
        "context.preview" => preview(shared, actor, payload),
        "context.show" => show(shared, payload),
        _ => Err(invalid("unknown context query")),
    }
}

fn preview(shared: &Shared, actor: &TrustedActor, payload: &Value) -> store::Result<Value> {
    let manifest: Manifest = serde_json::from_value(payload["manifest"].clone())
        .map_err(|e| invalid(format!("manifest: {e}")))?;
    let permitted: Vec<String> =
        serde_json::from_value(payload["permitted"].clone()).unwrap_or_default();
    let budget = payload["budget"].as_u64().unwrap_or(manifest.budget);
    let assembler = LocalAssembler {
        permitted: permitted.into_iter().collect(),
        budget,
        tokenizer: None,
        renderer: manifest.redaction.clone(),
        redaction: manifest.redaction.clone(),
    };
    access(shared, |s| {
        let sources = StoreSources::new(s, actor, &manifest.scope);
        let consumer = owner_from(payload)?;
        let assembly = assembler
            .assemble(&sources, AssemblyRequest { manifest, consumer })
            .map_err(|e| invalid(format!("{}: {}", e.code, e.message)))?;
        Ok(serde_json::to_value(&(
            assembly.manifest.document,
            assembly.bundle.document,
        ))?)
    })
}

fn show(shared: &Shared, payload: &Value) -> store::Result<Value> {
    let project = payload["project_id"]
        .as_str()
        .ok_or_else(|| invalid("project_id required"))?;
    access(shared, |s| {
        let manifest = match payload["manifest_id"].as_str() {
            Some(id) => context::read_manifest(s, project, id).map_err(|e| invalid(e.message))?,
            None => None,
        };
        let bundle = match payload["bundle_id"].as_str() {
            Some(id) => context::read_bundle(s, project, id).map_err(|e| invalid(e.message))?,
            None => None,
        };
        if manifest.is_none() && bundle.is_none() {
            return Ok(Value::Null);
        }
        Ok(json!({
            "manifest": manifest,
            "bundle": bundle,
        }))
    })
}

fn owner_from(payload: &Value) -> store::Result<agency_proto::Owner> {
    serde_json::from_value(payload["consumer"].clone())
        .map_err(|e| invalid(format!("consumer: {e}")))
}

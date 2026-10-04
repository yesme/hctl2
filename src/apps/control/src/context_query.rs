//! `context.preview|show`: read-only queries over the local assembler.
//!
//! Preview selects mechanically from governance records by object ID
//! (Project + consuming Room + optional Task), assembles without saving, and answers with the manifest
//! and bundle. Show reads frozen records. The permission policy point is a
//! placeholder returning the project's admitted sources until package 5
//! wires real policy.

use chat::invalid;
use context::{
    Assembler, AssemblyRequest, LocalAssembler, SelectionRequest, permitted_source_ids,
    select_context,
};
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
        "context.show" => show(shared, actor, payload),
        _ => Err(invalid("unknown context query")),
    }
}

fn preview(shared: &Shared, actor: &TrustedActor, payload: &Value) -> store::Result<Value> {
    let project = payload["project_id"]
        .as_str()
        .ok_or_else(|| invalid("project_id required"))?;
    let room = payload["room_id"]
        .as_str()
        .ok_or_else(|| invalid("room_id required"))?;
    let budget = match payload.get("budget") {
        Some(value) => value
            .as_u64()
            .ok_or_else(|| invalid("budget must be an unsigned integer"))?,
        None => 64 * 1024,
    };
    access(shared, |s| {
        let consumer = match payload.get("consumer") {
            Some(value) => serde_json::from_value::<context::Owner>(value.clone())
                .map_err(|e| invalid(e.to_string()))?,
            None => context::Owner {
                project: project.into(),
                kind: agency_proto::OwnerKind::RoomInvocation,
                id: "preview".into(),
                generation: 1,
            },
        };
        if consumer.project != project {
            return Err(invalid("consumer Project differs from project_id"));
        }
        let task_id = match payload.get("task_id") {
            Some(value) => Some(
                value
                    .as_str()
                    .ok_or_else(|| invalid("task_id must be a string"))?
                    .to_owned(),
            ),
            None => None,
        };
        let (manifest, consumer) = select_context(
            s,
            actor,
            SelectionRequest {
                consumer,
                room_id: room.into(),
                task_id,
                budget,
            },
        )
        .map_err(port_error)?;
        // Permission policy point placeholder: the project's admitted
        // sources, never the caller's input.
        let permitted = permitted_source_ids(s, project).map_err(port_error)?;
        let assembler = LocalAssembler {
            permitted: permitted.into_iter().collect(),
            budget,
        };
        let assembly = assembler
            .assemble(
                &context::StoreSources::new(s, actor, project),
                AssemblyRequest { manifest, consumer },
            )
            .map_err(port_error)?;
        Ok(json!({
            "manifest": assembly.manifest.document,
            "manifest_digest": assembly.manifest.digest,
            "bundle": assembly.bundle.document,
            "bundle_digest": assembly.bundle.digest,
            "permission_policy": "project-admitted-sources (placeholder)",
        }))
    })
}

fn show(shared: &Shared, actor: &TrustedActor, payload: &Value) -> store::Result<Value> {
    let project = payload["project_id"]
        .as_str()
        .ok_or_else(|| invalid("project_id required"))?;
    access(shared, |s| {
        let manifest = match payload["manifest_id"].as_str() {
            Some(id) => context::read_manifest(s, actor, project, id).map_err(port_error)?,
            None => None,
        };
        let bundle = match payload["bundle_id"].as_str() {
            Some(id) => context::read_bundle(s, actor, project, id).map_err(port_error)?,
            None => None,
        };
        if manifest.is_none() && bundle.is_none() {
            return Ok(Value::Null);
        }
        Ok(json!({"manifest": manifest, "bundle": bundle}))
    })
}

/// Port errors keep their codes and recovery actions at the boundary.
fn port_error(error: context::PortError) -> store::StoreError {
    let static_code = match error.code.as_str() {
        "INVALID_INPUT" => "INVALID_INPUT",
        "NOT_FOUND" => "NOT_FOUND",
        "PROJECT_READ_ONLY" => "PROJECT_READ_ONLY",
        "READBACK_REQUIRED" => "READBACK_REQUIRED",
        "VERSION_CONFLICT" => "VERSION_CONFLICT",
        "MATERIAL_NOT_DELIVERED" => "MATERIAL_NOT_DELIVERED",
        "MATERIAL_DIGEST_MISMATCH" => "MATERIAL_DIGEST_MISMATCH",
        "DELIVERY_DIGEST_MISMATCH" => "DELIVERY_DIGEST_MISMATCH",
        "SOURCE_VERSION_CHANGED" => "SOURCE_VERSION_CHANGED",
        "PERMISSION_CHANGED" => "PERMISSION_CHANGED",
        "BUDGET_CHANGED" => "BUDGET_CHANGED",
        "PERMISSION_DENIED" => "PERMISSION_DENIED",
        "REVIEW_LINE_NOT_CONFIGURED" => "REVIEW_LINE_NOT_CONFIGURED",
        "SOURCE_UNAVAILABLE" => "SOURCE_UNAVAILABLE",
        "CONTEXT_CONFLICT" => "CONTEXT_CONFLICT",
        _ => "CONTEXT_ASSEMBLY_FAILED",
    };
    let recovery = match error.recovery_action.as_str() {
        "preview_again" => "preview_again",
        "refresh_source" => "refresh_source",
        "request_authorization" => "request_authorization",
        "wait_for_review_wiring" => "wait_for_review_wiring",
        "select_context_sources" => "select_context_sources",
        "restore_material" => "restore_material",
        "correct_input" => "correct_input",
        "inspect_object" => "inspect_object",
        "use_a_new_manifest_id" | "use_a_new_bundle_id" => "use_a_new_id",
        _ => "inspect_context_input",
    };
    store::StoreError {
        code: static_code,
        message: error.message,
        recovery_action: recovery,
    }
}

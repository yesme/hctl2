//! Mechanical selection for an explicit consumer, consuming Room and optional
//! Project-local Task. Online windows and dispatch authorization stay outside.
use crate::sources::{ROOM_BRIEF_KIND, ROOM_LINE_KIND, TASK_LINE_KIND, project_actor, store_call};
use crate::{
    FrozenRef, Manifest, Owner, PortError, Result, StoreSources, frozen_from_record,
    pack_reference, permission_digest,
};
use agency_proto::{OwnerKind, hash};
use store::{Scope, Store, TrustedActor};

pub struct SelectionRequest {
    pub consumer: Owner,
    pub room_id: String,
    pub task_id: Option<String>,
    pub budget: u64,
}

/// A Project-scoped placeholder for the dispatch policy point, not an
/// authorization derived from the caller's supplied Manifest or source list.
pub fn permitted_source_ids(store: &Store, project: &str) -> Result<Vec<String>> {
    let mut permitted = Vec::new();
    for (record_kind, source_kind) in [
        (ROOM_LINE_KIND, ROOM_LINE_KIND),
        (ROOM_BRIEF_KIND, ROOM_BRIEF_KIND),
        ("task_state", TASK_LINE_KIND),
    ] {
        for record in store_call(|| store.list(record_kind))?
            .into_iter()
            .filter(|r| r.key.scope == Scope::Project(project.into()))
        {
            if record_kind == ROOM_BRIEF_KIND
                && store_call(|| chat::decode::<chat::Room>(&record))?
                    .brief
                    .is_none()
            {
                continue;
            }
            permitted.push(pack_reference(source_kind, &record.key.id));
        }
    }
    permitted.sort();
    Ok(permitted)
}

pub fn select_context(
    store: &Store,
    actor: &TrustedActor,
    request: SelectionRequest,
) -> Result<(Manifest, Owner)> {
    request.consumer.validate()?;
    if request.budget == 0 {
        return Err(PortError::invalid("budget must be positive"));
    }
    let project = &request.consumer.project;
    let actor = project_actor(actor, project)?;
    let project_record = store_call(|| {
        chat::required(
            store,
            &chat::key(Scope::Project(project.clone()), "project", project),
        )
    })?;
    store_call(|| chat::active_project(store, project, project_record.version))?;
    let (record, room) = store_call(|| chat::room(store, project, &request.room_id))?;
    if room.project_id != *project || room.id != request.room_id {
        return Err(PortError::invalid(
            "Room identity does not match its Project-local key",
        ));
    }
    let mut sources = Vec::new();
    if let Some(material) = &room.brief {
        // The confirmed brief, not the source Room's binding, owns this list.
        if material.scope != record.key.scope {
            return Err(PortError::invalid("Room brief belongs to another scope"));
        }
        let bytes = store_call(|| store.read_material(&actor, material))?;
        let brief: chat::Brief = serde_json::from_slice(&bytes)?;
        sources.push(frozen_from_record(&record)?);
        for source in brief.sources {
            let id = foundation::canonical_json_sha256(&serde_json::to_value(source)?)
                .map_err(|e| PortError::invalid(e.to_string()))?;
            let record = store_call(|| {
                chat::required(
                    store,
                    &chat::key(Scope::Project(project.clone()), ROOM_LINE_KIND, &id),
                )
            })?;
            sources.push(frozen_from_record(&record)?);
        }
    }
    if let Some(task) = &request.task_id {
        sources.push(
            StoreSources::new(store, &actor, project)
                .task_line(task)?
                .reference,
        );
    }
    if sources.is_empty() {
        return Err(PortError::new(
            "SOURCE_UNAVAILABLE",
            "Room has no confirmed brief and no Task was selected; an online window must be supplied by dispatch",
            "select_context_sources",
        ));
    }
    let mut manifest = Manifest {
        id: String::new(),
        purpose: match &request.task_id {
            Some(task) => format!("context for Room {} and Task {task}", request.room_id),
            None => format!("context for Room {}", request.room_id),
        },
        scope: format!("project {project}"),
        parent: None,
        sources,
        selection_policy: policy_ref("policy/mechanical-v1", b"hctl2.context.mechanical.v1"),
        freshness: "precise admitted versions at selection".into(),
        coverage: "confirmed Topic brief and its sources; explicitly selected Task comments".into(),
        known_gaps: vec![
            "online discussion window and additional explicit materials were not selected".into(),
            "platform review-comment line is not configured".into(),
        ],
        required_skills: vec![],
        permission_digest: permission_digest(&permitted_source_ids(store, project)?),
        redaction: policy_ref("redaction/none", b"hctl2.context.redaction.none.v1"),
        budget: request.budget,
    };
    // Hash the entire frozen input with an empty id, not just Room and budget.
    let digest = foundation::canonical_json_sha256(&serde_json::to_value(&manifest)?)
        .map_err(|e| PortError::invalid(e.to_string()))?;
    manifest.id = format!("manifest-{digest}");
    Ok((manifest, request.consumer))
}

/// Read-only preview convenience. Actual dispatch supplies its exact Owner
/// through select_context; this sentinel never creates an Invocation.
pub fn select_room_manifest(
    store: &Store,
    actor: &TrustedActor,
    project: &str,
    room_id: &str,
    budget: u64,
) -> Result<(Manifest, Owner)> {
    select_context(
        store,
        actor,
        SelectionRequest {
            consumer: Owner {
                project: project.into(),
                kind: OwnerKind::RoomInvocation,
                id: "preview".into(),
                generation: 1,
            },
            room_id: room_id.into(),
            task_id: None,
            budget,
        },
    )
}

fn policy_ref(id: &str, bytes: &[u8]) -> FrozenRef {
    FrozenRef {
        id: id.into(),
        revision: "1".into(),
        digest: hash(bytes),
    }
}

//! Server-resolved sources and the existing mechanical Context assembler.
use super::*;
use ::context::{
    Assembly, AssemblyRequest, LocalAssembler, Manifest, SelectionRequest, SourceContent,
    SourceKind, Sources, StoreSources,
};

fn reference(id: String, bytes: &[u8]) -> agency_proto::FrozenRef {
    let digest = agency_proto::hash(bytes);
    agency_proto::FrozenRef {
        id,
        revision: digest.clone(),
        digest,
    }
}

pub(super) fn assemble(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    actor: &TrustedActor,
    preview: &invocation::Preview,
) -> store::Result<Assembly> {
    let p = &preview.input.project_id;
    let scoped = chat::owner(actor, p)?;
    let room = access(shared, |s| Ok(chat::room(s, p, &preview.input.room_id)?.1))?;
    // The built-in providers currently declare no required Skills. Do not
    // invent bytes or use the control's filesystem as an Agency skill store.
    if !preview.required_skills.is_empty() {
        return Err(reject(
            "SKILL_DELIVERY_UNAVAILABLE",
            "Agency does not expose exact required Skill bytes yet",
            "choose_supported_profession",
        ));
    }
    let window = crate::chat::invocation_window(services, root, &room)?;
    let window_bytes = foundation::canonical_json(&window)?;
    let mut contents = vec![SourceContent {
        reference: reference(
            format!("invocation-request/{}", preview.consumer.id),
            preview.input.request.as_bytes(),
        ),
        bytes: preview.input.request.as_bytes().to_vec(),
    }];
    if preview.brief.is_some() || preview.input.task_id.is_some() {
        let selected = access(shared, |s| {
            let (selected, _) = ::context::select_context(
                s,
                &scoped,
                SelectionRequest {
                    consumer: preview.consumer.clone(),
                    room_id: preview.input.room_id.clone(),
                    task_id: preview.input.task_id.clone(),
                    budget: preview.input.budget,
                },
            )
            .map_err(participant::port_error)?;
            selected
                .sources
                .iter()
                .map(|r| {
                    StoreSources::new(s, &scoped, p)
                        .exact(kind(r)?, r)
                        .map_err(participant::port_error)
                })
                .collect::<store::Result<Vec<_>>>()
        })?;
        contents.extend(selected);
    }
    contents.push(SourceContent {
        reference: reference(
            format!("chat-window/{}", preview.input.room_id),
            &window_bytes,
        ),
        bytes: window_bytes,
    });
    let permitted: Vec<_> = contents.iter().map(|c| c.reference.id.clone()).collect();
    let policy = reference(
        "policy/invocation-read-only-v1".into(),
        b"hctl2.invocation.explicit-request-own-room.v1",
    );
    let mut manifest = Manifest {
        id: String::new(),
        purpose: format!("read-only invocation {}", preview.consumer.id),
        scope: format!("project {p}"),
        parent: None,
        sources: contents.iter().map(|c| c.reference.clone()).collect(),
        selection_policy: policy.clone(),
        freshness: "server-ordered exact window at human preview".into(),
        coverage: "explicit request, this Room's confirmed brief and its sources, selected Task comments and current window".into(),
        known_gaps: vec![
            "window limited to the latest 100 server events; no inferred Task or additional Memo / Artifact"
                .into(),
            "platform review-comment line is not configured".into(),
        ],
        required_skills: preview.required_skills.clone(),
        permission_digest: ::context::permission_digest(&permitted),
        redaction: reference("redaction/none".into(), b"hctl2.context.redaction.none.v1"),
        budget: preview.input.budget,
    };
    manifest.id = format!(
        "manifest-{}",
        foundation::canonical_json_sha256(&serde_json::to_value(&manifest)?)?
    );
    LocalAssembler {
        permitted: permitted.into_iter().collect(),
        budget: preview.input.budget,
    }
    .assemble_resolved(
        AssemblyRequest {
            manifest,
            consumer: preview.consumer.clone(),
        },
        contents,
    )
    .map_err(participant::port_error)
}

fn kind(r: &agency_proto::FrozenRef) -> store::Result<SourceKind> {
    if r.id.starts_with("task_comments/") {
        Ok(SourceKind::TaskComments)
    } else if r.id.starts_with("room_binding/") || r.id.starts_with("chat_source_reference/") {
        Ok(SourceKind::Room)
    } else {
        Err(invalid("not a stored dispatch source"))
    }
}

pub(super) fn verify_sources(s: &Store, actor: &TrustedActor, plan: &Plan) -> store::Result<()> {
    for r in &plan.assembly.manifest.document.sources {
        if r.id.starts_with("task_comments/")
            || r.id.starts_with("room_binding/")
            || r.id.starts_with("chat_source_reference/")
        {
            StoreSources::new(s, actor, &plan.preview.input.project_id)
                .exact(kind(r)?, r)
                .map_err(participant::port_error)?;
        }
    }
    Ok(())
}

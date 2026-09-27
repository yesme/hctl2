use crate::*;
use foundation::{bytes_sha256, canonical_json};
use store::{Record, RecordData, Reference, RoomKind, Scope, Store, Version};

pub const MECHANICAL_RULE: &str = "hctl2.brief.verbatim.v1: structural selection; exact source fields; no sections or generated text";

pub fn mechanical_draft(fragments: Vec<SourceText>, unread: Vec<Source>) -> Draft {
    Draft {
        automatic_summary: "not_configured".into(),
        fragments,
        unread,
        rule_reference: "hctl2.brief.verbatim.v1".into(),
        rule_digest: bytes_sha256(MECHANICAL_RULE.as_bytes()),
    }
}

pub fn verify_draft(draft: &Draft, allowed: &[SourceText]) -> Result<()> {
    if draft.automatic_summary != "not_configured"
        || draft.rule_reference != "hctl2.brief.verbatim.v1"
        || draft.rule_digest != bytes_sha256(MECHANICAL_RULE.as_bytes())
    {
        return Err(invalid(
            "mechanical draft cannot claim automatic summarization",
        ));
    }
    for fragment in &draft.fragments {
        if !allowed.iter().any(|s| {
            s.source == fragment.source && s.body == fragment.body && s.excerpt == fragment.excerpt
        }) {
            return Err(invalid(
                "system draft changed text or introduced an unselected source",
            ));
        }
    }
    Ok(())
}

/// Resolve the exact historical version, never substitute the current projection.
pub fn at(store: &Store, reference: &Reference) -> Result<Record> {
    store
        .versions(&reference.key)?
        .into_iter()
        .find(|r| match &reference.version {
            Version::State(v) => r.version == *v,
            Version::Revision(digest) => r.revision_digest == *digest,
        })
        .ok_or_else(|| {
            reject(
                "SOURCE_UNAVAILABLE",
                "frozen source version missing",
                "restore_source",
            )
        })
}

pub fn origin_checks(store: &Store, project: &str, origin: &Origin) -> Result<Vec<Check>> {
    match origin {
        Origin::MainRoom {
            room_id,
            binding_version,
        } => {
            let (binding, _) = room(store, project, room_id)?;
            let r = required(store, &key(Scope::Project(project.into()), "room", room_id))?;
            if binding.version != *binding_version
                || !matches!(
                    r.data,
                    RecordData::Room {
                        room_kind: RoomKind::Main,
                        ..
                    }
                )
            {
                return Err(stale());
            }
            Ok(vec![Check {
                key: binding.key,
                version: Some(binding.version),
            }])
        }
        Origin::Request { request, blockers } => {
            if request.key.scope != Scope::Project(project.into()) || request.key.kind != "request"
            {
                return Err(invalid("Request must belong to this Project"));
            }
            let r = at(store, request)?;
            let source: RequestSource = decode(&r)?;
            if source.blockers != *blockers || blockers.is_empty() {
                return Err(invalid(
                    "Request blockers differ from the frozen Request version",
                ));
            }
            for blocker in blockers {
                if blocker.key.scope != Scope::Project(project.into()) {
                    return Err(invalid("blocker belongs to another Project"));
                }
            }
            Ok(vec![])
        }
    }
}

pub fn request_texts(
    store: &Store,
    request: &Reference,
    blockers: &[Reference],
) -> Result<Vec<SourceText>> {
    std::iter::once(request)
        .chain(blockers)
        .map(|reference| request_text(store, reference, reference == request))
        .collect()
}

pub fn request_text(store: &Store, reference: &Reference, is_request: bool) -> Result<SourceText> {
    let record = at(store, reference)?;
    let body = String::from_utf8(canonical_json(&serde_json::to_value(&record.data)?)?)
        .map_err(|_| invalid("source is not UTF-8"))?;
    let excerpt = if is_request {
        decode::<RequestSource>(&record)?.question
    } else {
        body.clone()
    };
    Ok(SourceText {
        source: Source::Object {
            reference: reference.clone(),
            content_digest: bytes_sha256(body.as_bytes()),
        },
        body,
        excerpt,
    })
}

pub fn validate_sources(project: &str, origin: &Origin, sources: &[SourceText]) -> Result<()> {
    if sources.is_empty() {
        return Err(invalid("exact sources required"));
    }
    for source in sources {
        let allowed = match (&source.source, origin) {
            (
                Source::Message {
                    binding,
                    content_digest,
                    event_id,
                },
                Origin::MainRoom {
                    room_id,
                    binding_version,
                },
            ) => {
                binding.key == key(Scope::Project(project.into()), "room_binding", room_id)
                    && binding.version == Version::State(*binding_version)
                    && !event_id.is_empty()
                    && *content_digest == bytes_sha256(source.body.as_bytes())
            }
            (
                Source::Object {
                    reference,
                    content_digest,
                },
                Origin::Request { request, blockers },
            ) => {
                (reference == request || blockers.contains(reference))
                    && *content_digest == bytes_sha256(source.body.as_bytes())
                    && reference.key.scope == Scope::Project(project.into())
            }
            _ => false,
        };
        if !allowed {
            return Err(invalid("source outside the creation command's exact scope"));
        }
    }
    if let Origin::Request { request, blockers } = origin {
        for reference in std::iter::once(request).chain(blockers) {
            if !sources
                .iter()
                .any(|s| matches!(&s.source, Source::Object { reference: r, .. } if r == reference))
            {
                return Err(invalid(
                    "Request source must retain all frozen blocker references",
                ));
            }
        }
    }
    Ok(())
}

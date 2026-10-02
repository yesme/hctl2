use crate::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use store::{RecordData, RoomState, Scope, Store, TrustedActor};

pub fn archive_blockers(store: &Store, id: &str) -> Result<Vec<ArchiveBlocker>> {
    let p = project(store, id)?;
    let RecordData::Project { repo_id, .. } = &p.data else {
        return Err(invalid("Project required"));
    };
    let scope = p.key.scope.clone();
    let mut blockers = vec![];
    // P2.3+ reducers use these owner references. Unknown state fails closed, never
    // treats an unfamiliar future record as a settled execution.
    for kind in [
        "run",
        "room_invocation",
        "input_lease",
        "write_lease",
        "integration_intent",
        "review_publish_intent",
    ] {
        for r in store.list(kind)? {
            let value: Value = decode(&r)?;
            let belongs = r.key.scope == scope
                || (r.key.scope == Scope::Repo(repo_id.clone())
                    && (r.sources.iter().any(|s| s.key.scope == scope)
                        || value["project_id"] == id
                        || value["owner"]["key"]["scope"] == serde_json::to_value(&scope)?));
            if !belongs {
                continue;
            }
            let state = value["state"]
                .as_str()
                .or_else(|| value["lifecycle"].as_str())
                .unwrap_or("unknown");
            let terminal = [
                "completed",
                "complete",
                "failed",
                "cancelled",
                "lost",
                "丢失",
                "expired",
                "released",
                "confirmed",
                "rejected",
            ]
            .contains(&state);
            // Read-only invocation is harmless only when its authorization proves it.
            let read_only = kind == "room_invocation" && value["authorization"]["write"] == false;
            if !terminal && !read_only {
                blockers.push(ArchiveBlocker {
                    reference: reference(&r),
                    kind: kind.into(),
                    state: state.into(),
                });
            }
        }
    }
    for effect_id in store.pending_effects()? {
        let (effect, state) = store.effect(&effect_id)?;
        let belongs = effect.permission_scope == scope
            || effect.owner.key.scope == scope
            || (effect.permission_scope == Scope::Repo(repo_id.clone())
                && effect.input["project_id"] == id);
        if belongs {
            blockers.push(ArchiveBlocker {
                reference: effect.owner,
                kind: format!("effect:{}", effect.intent_id),
                state: format!("{state:?}").to_lowercase(),
            });
        }
    }
    blockers.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then(a.reference.key.id.cmp(&b.reference.key.id))
    });
    Ok(blockers)
}

pub fn pending(store: &Store, id: &str, actor: &TrustedActor) -> Result<Value> {
    let p = project(store, id)?;
    let mut items = vec![];
    let mut covered = vec![];
    if !readonly(&p) {
        for (r, request) in requests(store, id)? {
            if request.state != RequestState::Open
                || request.spec.deadline.is_some_and(|d| d <= task::now())
                || store.get(&request.spec.owner.key)?.as_ref().map(reference)
                    != Some(request.spec.owner.clone())
                || !may_answer(store, id, &request.spec, actor)?
            {
                continue;
            }
            covered.push(request.spec.owner.key.clone());
            items.push(json!({"source":reference(&r),"object":request.spec.owner,"reason":request.question,
                "action":"project.resolve_request","consequence":"admit the typed contract answer and deliver to the frozen Task blocker",
                "unhandled_impact":"source remains blocked until answer or deadline","return_to":{"project_id":id,"request_id":r.key.id}}));
        }
        // The current local owner can adopt contracts, not another human's Request.
        chat::owner(actor, id)?;
        for (r, task) in task::tasks(store)? {
            if task.project_id != id
                || task.archived
                || task.lifecycle != "open"
                || task.pending_contract.is_none()
                || covered.contains(&r.key)
                || task::request_blockers(store, id, &task.id)?
                    .iter()
                    .any(|(_, b)| b.waiting && b.outcome.is_none() && b.owner == reference(&r))
            {
                continue;
            }
            items.push(json!({"source":reference(&r),"object":reference(&r),"reason":"contract changes await adoption",
                "action":"task.adopt","consequence":"create a new Task Revision; leave lifecycle unchanged",
                "unhandled_impact":"accepted contract remains unchanged","return_to":{"project_id":id,"task_id":task.id}}));
        }
    }
    Ok(
        json!({"project_id":id,"items":items,"implemented_sources":["request","contract_adoption"],"deferred_sources":["review_publication","candidate_delivery","run_timeout"]}),
    )
}

pub fn overview(store: &Store, id: &str, actor: &TrustedActor) -> Result<Value> {
    let p = project(store, id)?;
    let definition = definition(store, id)?;
    let tasks: Vec<_> = task::tasks(store)?
        .into_iter()
        .filter(|(_, t)| t.project_id == id)
        .collect();
    let requests = requests(store, id)?;
    let rooms = store
        .list("room")?
        .into_iter()
        .filter(|r| r.key.scope == p.key.scope)
        .count();
    let recent: Vec<_> = store
        .recent_versions(&p.key.scope, 20)?
        .into_iter()
        .map(|(seq, r)| json!({"sequence":seq,"source":reference(&r),"kind":r.key.kind}))
        .collect();
    Ok(
        json!({"project":p,"definition":definition,"health":if tasks.iter().any(|(_, t)| t.needs_attention) { "needs_attention" } else { "ok" },
        "counts":{"tasks":tasks.len(),"open_requests":requests.iter().filter(|(_, q)|q.state==RequestState::Open).count(),"rooms":rooms,"pending":pending(store,id,actor)?["items"].as_array().map_or(0,Vec::len)},
        "archive_blockers":archive_blockers(store,id)?,"recent_activity":recent,"projection":"readonly"}),
    )
}

/// Last activity is supplied by the adapter's current server read. None means
/// unreadable, not an empty Room; it cannot establish an idle interval.
pub fn attention(
    store: &Store,
    id: &str,
    now: u64,
    last_activity: &BTreeMap<String, Option<u64>>,
) -> Result<Vec<Value>> {
    if readonly(&project(store, id)?) {
        return Ok(vec![]);
    }
    let mut items = vec![];
    for binding in store
        .list("room_binding")?
        .into_iter()
        .filter(|r| r.key.scope == Scope::Project(id.into()))
    {
        let room: chat::Room = decode(&binding)?;
        let identity = required(store, &key(binding.key.scope.clone(), "room", &room.id))?;
        if !matches!(
            identity.data,
            RecordData::Room {
                state: RoomState::Active,
                ..
            }
        ) {
            continue;
        }
        let Some(chat::Origin::Request {
            request: origin, ..
        }) = &room.origin
        else {
            continue;
        };
        let current = required(store, &origin.key)?;
        let request: Request = decode(&current)?;
        if request.state != RequestState::Open || request.spec.deadline.is_some_and(|d| d <= now) {
            continue;
        }
        let Some(Some(latest)) = last_activity.get(&room.id) else {
            continue;
        };
        if now.saturating_sub(*latest) > 14 * 24 * 60 * 60 {
            items.push(json!({"room_id":room.id,"request":reference(&current),"needs_attention":true,"adds_pending_item":false}));
        }
    }
    Ok(items)
}

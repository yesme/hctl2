//! Native Space organization. No parent pointer is written into governance records.
use super::*;
use matrix::{Client, room_id};
use std::collections::{BTreeMap, BTreeSet};

impl Client {
    pub(super) fn put_state(
        &self,
        external: &str,
        kind: &str,
        key: &str,
        content: Value,
    ) -> Result<()> {
        self.guard(external)?;
        self.request(
            ruma::api::client::state::send_state_event::v3::Request::new_raw(
                room_id(external)?,
                kind.into(),
                key.into(),
                ruma::serde::Raw::from_json_string(content.to_string())?,
            ),
        )?;
        let response = self.request(
            ruma::api::client::state::get_state_event_for_key::v3::Request::new(
                room_id(external)?,
                kind.into(),
                key.into(),
            ),
        )?;
        if serde_json::from_str::<Value>(response.event_or_content.get())? != content {
            return Err(reject(
                "CHAT_CONTENT_MISMATCH",
                "native state readback differs",
                "retry_content_action",
            ));
        }
        Ok(())
    }

    pub(super) fn room_state(&self, external: &str) -> Result<Vec<Value>> {
        self.guard(external)?;
        self.request(
            ruma::api::client::state::get_state_events::v3::Request::new(room_id(external)?),
        )?
        .room_state
        .into_iter()
        .map(|event| Ok(serde_json::from_str(event.json().get())?))
        .collect()
    }

    pub(super) fn carrier(&self, room: &Room) -> Result<Option<String>> {
        let external = bound(room)?;
        let value = match self.state(external, "io.hctl2.carrier") {
            Ok(v) => v,
            Err(e) if e.code == "CHAT_NOT_FOUND" => return Ok(None),
            Err(e) => return Err(e),
        };
        let id = value["space_id"]
            .as_str()
            .ok_or_else(|| invalid("invalid carrier pointer"))?;
        let correlation = foundation::bytes_sha256(carrier_key(room)?.as_bytes());
        if self.state(id, "m.room.create")?["type"] != "m.space"
            || self.state(id, "io.hctl2.creation")?
                != json!({"command":correlation,"room":room.id,"project":room.project_id})
        {
            return Err(invalid("carrier does not represent this bound Room"));
        }
        Ok(Some(id.into()))
    }

    /// Caller must have persisted an external-write intent before entering here.
    pub(super) fn ensure_carrier(&self, room: &Room) -> Result<String> {
        if let Some(id) = self.carrier(room)? {
            return Ok(id);
        }
        let created = self.create_native(room, &carrier_key(room)?, true, false, || Ok(()))?;
        let id = created["matrix_room_id"]
            .as_str()
            .ok_or_else(|| invalid("carrier ID missing"))?;
        // Preserve existing native parents when a former leaf gains a wrapper.
        for event in self.room_state(bound(room)?)? {
            if event["type"] == "m.space.parent"
                && event["content"]["via"]
                    .as_array()
                    .is_some_and(|a| !a.is_empty())
            {
                let parent = event["state_key"]
                    .as_str()
                    .ok_or_else(|| invalid("invalid Space parent"))?;
                // A retry may find our own wrapper before its pointer was stored.
                if parent == id {
                    continue;
                }
                self.attach(parent, id, event["content"]["canonical"] == true)?;
                self.put_state(parent, "m.space.child", bound(room)?, json!({}))?;
                // Clear the old reverse edge only after the replacement is readable.
                self.put_state(bound(room)?, "m.space.parent", parent, json!({}))?;
            }
        }
        // A native pointer joins the wrapper to the message room, not a Binding revision.
        self.attach(id, bound(room)?, true)?;
        self.put_state(bound(room)?, "io.hctl2.carrier", "", json!({"space_id":id}))?;
        Ok(id.into())
    }

    pub(super) fn attach(&self, space: &str, child: &str, canonical: bool) -> Result<()> {
        if self.state(space, "m.room.create")?["type"] != "m.space" {
            return Err(invalid("only a Space can carry children"));
        }
        self.put_state(
            space,
            "m.space.child",
            child,
            json!({"via":[self.server.server_name],"suggested":true}),
        )?;
        self.put_state(
            child,
            "m.space.parent",
            space,
            json!({"via":[self.server.server_name],"canonical":canonical}),
        )
    }
}

fn carrier_key(room: &Room) -> Result<String> {
    Ok(format!(
        "carrier:{}:{}:{}",
        room.project_id,
        room.id,
        bound(room)?
    ))
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub(super) struct Hierarchy {
    pub carrier_space_id: Option<String>,
    pub parents: Vec<Value>,
    pub children: Vec<String>,
    pub external_links: Vec<String>,
    pub needs_attention: bool,
}

/// All immediate state edges are read independently. No recursive hierarchy endpoint or
/// ten-level response limit is allowed to turn into a product depth limit.
pub(super) fn project_hierarchy(client: &Client, rooms: &[Room]) -> BTreeMap<String, Hierarchy> {
    let (mut result, edges) = read_hierarchy(client, rooms);
    project_edges(&mut result, &edges);
    result
}

type NativeEdges = BTreeMap<(String, String), bool>;

fn read_hierarchy(client: &Client, rooms: &[Room]) -> (BTreeMap<String, Hierarchy>, NativeEdges) {
    let mut result: BTreeMap<_, _> = rooms
        .iter()
        .map(|r| (r.id.clone(), Hierarchy::default()))
        .collect();
    let mut mapping = BTreeMap::new();
    let mut carrier_mapping = BTreeMap::new();
    let mut states = BTreeMap::new();
    for room in rooms {
        let read = (|| -> Result<()> {
            let external = bound(room)?;
            if room.server.binding != client.server.binding || room.server.url != client.server.url
            {
                return Err(chat::stale());
            }
            let mut events = client.room_state(external)?;
            events.retain(|event| event["type"] == "m.space.parent");
            let space = client.carrier(room)?;
            if let Some(id) = &space {
                events.extend(client.room_state(id)?);
                mapping.insert(id.clone(), room.id.clone());
                carrier_mapping.insert(id.clone(), room.id.clone());
            }
            result.get_mut(&room.id).unwrap().carrier_space_id = space;
            mapping.insert(external.into(), room.id.clone());
            states.insert(room.id.clone(), events);
            Ok(())
        })();
        if read.is_err() {
            result.get_mut(&room.id).unwrap().needs_attention = true;
        }
    }
    let mut edges = BTreeMap::<(String, String), bool>::new();
    for (id, events) in &states {
        for event in events {
            if event["content"]["via"]
                .as_array()
                .is_none_or(|a| a.is_empty())
            {
                continue;
            }
            let Some(external) = event["state_key"].as_str() else {
                continue;
            };
            match event["type"].as_str() {
                Some("m.space.parent") => {
                    if let Some(parent) = carrier_mapping.get(external) {
                        if parent != id {
                            edges
                                .entry((parent.clone(), id.clone()))
                                .and_modify(|v| *v |= event["content"]["canonical"] == true)
                                .or_insert(event["content"]["canonical"] == true);
                        }
                    } else {
                        let entry = result.get_mut(id).unwrap();
                        entry.external_links.push(external.into());
                        entry.needs_attention = true;
                    }
                }
                Some("m.space.child") => {
                    if let Some(child) = mapping.get(external)
                        && child != id
                    {
                        edges.entry((id.clone(), child.clone())).or_insert(false);
                    }
                }
                _ => (),
            }
        }
    }
    (result, edges)
}

fn reaches(edges: &NativeEdges, start: &str, target: &str) -> bool {
    let mut pending = vec![start.to_owned()];
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if id == target {
            return true;
        }
        if seen.insert(id.clone()) {
            pending.extend(
                edges
                    .keys()
                    .filter(|(parent, _)| *parent == id)
                    .map(|(_, child)| child.clone()),
            );
        }
    }
    false
}

pub(super) fn project_edges(
    result: &mut BTreeMap<String, Hierarchy>,
    edges: &BTreeMap<(String, String), bool>,
) {
    // An edge is on a cycle exactly when its child can already reach its parent.
    for ((parent, child), canonical) in edges {
        if reaches(edges, child, parent) {
            result.get_mut(parent).unwrap().needs_attention = true;
            result.get_mut(child).unwrap().needs_attention = true;
        } else {
            result
                .get_mut(child)
                .unwrap()
                .parents
                .push(json!({"room_id":parent,"canonical":canonical}));
            result.get_mut(parent).unwrap().children.push(child.clone());
        }
    }
}

/// Content write, not a governance command. The caller supplies the exact old parent set;
/// partial native writes remain observable and retryable rather than claiming atomicity.
pub(crate) fn reparent(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    payload: &Value,
    actor: &TrustedActor,
) -> Result<Value> {
    let managed_prefix = config::managed_user_prefix(services)?;
    reparent_using(shared, root, payload, actor, &managed_prefix, || {
        client(services)
    })
}

pub(super) fn reparent_using(
    shared: &Shared,
    root: &Path,
    payload: &Value,
    actor: &TrustedActor,
    managed_prefix: &str,
    connect: impl FnOnce() -> Result<Client>,
) -> Result<Value> {
    let project = payload["project_id"]
        .as_str()
        .ok_or_else(|| invalid("project_id required"))?;
    let id = payload["room_id"]
        .as_str()
        .ok_or_else(|| invalid("room_id required"))?;
    let parent = payload["parent_room_id"]
        .as_str()
        .ok_or_else(|| invalid("parent_room_id required"))?;
    if id == parent {
        return Err(invalid("Room cannot be its own parent"));
    }
    let rooms = access(shared, |s| {
        let (r, room) = chat::room(s, project, id)?;
        let (p, parent) = chat::room(s, project, parent)?;
        chat::writable(s, &room)?;
        chat::writable(s, &parent)?;
        let current = chat::required(s, &chat::key(r.key.scope.clone(), "project", project))?;
        chat::active_project(s, project, current.version)?;
        if payload["binding_version"].as_i64() != Some(r.version)
            || payload["parent_binding_version"].as_i64() != Some(p.version)
        {
            return Err(chat::stale());
        }
        Ok((room, parent))
    })?;
    let old: Vec<String> = serde_json::from_value(payload["old_space_ids"].clone())?;
    let client = connect()?;
    guard(&client, root, &rooms.0, bound(&rooms.0)?)?;
    guard(&client, root, &rooms.1, bound(&rooms.1)?)?;
    let project_rooms = access(shared, |s| {
        s.list("room_binding")?
            .iter()
            .filter(|r| r.key.scope == Scope::Project(project.into()))
            .map(chat::decode::<Room>)
            .collect::<Result<Vec<_>>>()
    })?;
    let (hierarchy, edges) = read_hierarchy(&client, &project_rooms);
    // Use raw native edges, including those omitted from the display as cyclic.
    if reaches(&edges, id, parent) {
        return Err(reject(
            "CHAT_HIERARCHY_CYCLE",
            "target Room is a descendant of this Room",
            "choose_other_parent",
        ));
    }
    let allowed = hierarchy
        .values()
        .filter_map(|projection| projection.carrier_space_id.clone())
        .collect::<BTreeSet<_>>();
    if old.iter().any(|id| !allowed.contains(id)) {
        return Err(invalid(
            "old parent Space is not a readable Room in this Project",
        ));
    }
    let space = match client.carrier(&rooms.1)? {
        Some(id) => {
            let effect = carrier_intent_id(&rooms.1)?;
            if access(shared, |s| Ok(s.pending_effects()?.contains(&effect)))? {
                let receipt = drive_using(shared, root, actor, &effect, managed_prefix, || {
                    Ok(client.clone())
                })?;
                if receipt["space_id"] != id {
                    return Err(invalid("carrier readback differs"));
                }
            }
            id
        }
        None => {
            let id = carrier_intent(shared, actor, &rooms.1)?;
            drive_using(shared, root, actor, &id, managed_prefix, || {
                Ok(client.clone())
            })?["space_id"]
                .as_str()
                .ok_or_else(|| invalid("carrier readback missing"))?
                .to_owned()
        }
    };
    let child_carrier = client.carrier(&rooms.0)?;
    let child_target = child_carrier.as_deref().unwrap_or(bound(&rooms.0)?);
    let mut failed = vec![];
    for old in old {
        if old == space {
            continue;
        }
        let operation = client
            .put_state(&old, "m.space.child", child_target, json!({}))
            .and_then(|_| client.put_state(child_target, "m.space.parent", &old, json!({})))
            .and_then(|_| client.put_state(&old, "m.space.child", bound(&rooms.0)?, json!({})))
            .and_then(|_| client.put_state(bound(&rooms.0)?, "m.space.parent", &old, json!({})));
        if let Err(e) = operation {
            failed.push(json!({"space_id":old,"error":e.code}));
        }
    }
    if let Err(e) = client.attach(&space, child_target, true) {
        failed.push(json!({"space_id":space,"error":e.code}));
    }
    Ok(
        json!({"state":if failed.is_empty() {"confirmed"} else {"partial"},"failures":failed,"space_id":space}),
    )
}

fn carrier_intent_id(room: &Room) -> Result<String> {
    Ok(format!(
        "chat:carrier:{}",
        foundation::bytes_sha256(carrier_key(room)?.as_bytes())
    ))
}

pub(super) fn carrier_intent(shared: &Shared, actor: &TrustedActor, room: &Room) -> Result<String> {
    access(shared, |s| {
        let actor = chat::owner(actor, &room.project_id)?;
        let identity = chat::writable(s, room)?;
        let id = carrier_intent_id(room)?;
        let input = json!({"room":room});
        let command = Command {
            command_id: id.clone(),
            idempotency_key: id.clone(),
            actor: actor.0.clone(),
            target: chat::key(identity.key.scope.clone(), "chat_content_intent", &id),
            expected: Expected::Absent,
            binding: room.server.binding.clone(),
            input_digest: Command::digest_input("chat.carrier", &input)?,
            operation: "chat.carrier".into(),
            input: input.clone(),
        };
        s.submit(s.generation(), &actor, &command, None, |tx| {
            tx.enqueue_effect(&store::EffectIntent {
                intent_id: id.clone(),
                owner: chat::reference(&identity),
                binding: room.server.binding.clone(),
                operation: "chat.carrier".into(),
                target: bound(room)?.into(),
                conflict_scope: format!("chat:carrier:{}:{}", room.project_id, room.id),
                permission_scope: identity.key.scope.clone(),
                input_digest: command.input_digest.clone(),
                input,
                idempotency_key: id.clone(),
            })?;
            Ok(json!({"effect_id":id}))
        })?;
        Ok(id)
    })
}

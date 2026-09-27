use super::*;

pub(crate) fn query(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    kind: &str,
    payload: &Value,
) -> Result<Value> {
    if kind == "room.list" {
        return access(shared, |s| {
            let project = payload["project_id"].as_str();
            let rooms = s
                .list("room_binding")?
                .into_iter()
                .filter(|r| project.is_none_or(|p| r.key.scope == Scope::Project(p.into())))
                .map(|r| {
                    let room: Room = chat::decode(&r)?;
                    let health = observations(root).state(&room.id)?;
                    Ok(json!({"binding":r,"room":room,"health":health}))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(json!({"rooms":rooms}))
        });
    }
    if kind == "room.draft" {
        return draft(
            shared,
            services,
            root,
            serde_json::from_value(payload.clone())?,
        );
    }
    let project = payload["project_id"]
        .as_str()
        .ok_or_else(|| invalid("project_id required"))?;
    if kind == "room.reference" {
        let source: Source = serde_json::from_value(payload["source"].clone())?;
        let key = chat::key(
            Scope::Project(project.into()),
            "chat_source_reference",
            &foundation::canonical_json_sha256(&serde_json::to_value(&source)?)?,
        );
        return access(shared, |s| {
            let record = chat::required(s, &key)?;
            let material = record
                .materials
                .first()
                .ok_or_else(|| invalid("frozen source missing"))?;
            let actor = TrustedActor(Actor {
                principal: "frozen-reference-reader".into(),
                source: ActorSource::DirectClient,
                permission_scope: vec![key.scope.clone()],
                authority: None,
            });
            Ok(
                json!({"source":source,"body":String::from_utf8(s.read_material(&actor, material)?).map_err(|_| invalid("invalid frozen source encoding"))?,"digest":material.byte_digest}),
            )
        });
    }
    let id = payload["room_id"]
        .as_str()
        .ok_or_else(|| invalid("room_id required"))?;
    let (record, room, stamp) = access(shared, |s| {
        let (record, room) = chat::room(s, project, id)?;
        Ok((record, room, s.read_stamp()))
    })?;
    let result = match kind {
        "room.show" => {
            let state = access(shared, |s| {
                chat::required(s, &chat::key(record.key.scope.clone(), "room", id))
            })?;
            let brief = access(shared, |s| {
                let actor = TrustedActor(Actor {
                    principal: "room-reader".into(),
                    source: ActorSource::DirectClient,
                    permission_scope: vec![record.key.scope.clone()],
                    authority: None,
                });
                room.brief
                    .as_ref()
                    .map(|m| -> Result<Value> {
                        Ok(serde_json::from_slice(&s.read_material(&actor, m)?)?)
                    })
                    .transpose()
            })?;
            Ok(
                json!({"binding":record,"room":room,"brief":brief,"identity":state,"health":observations(root).state(id)?}),
            )
        }
        "room.view_state" => {
            let client = client(services)?;
            guard(&client, root, &room, bound(&room)?)?;
            client.view_state(
                bound(&room)?,
                payload["client_id"]
                    .as_str()
                    .ok_or_else(|| invalid("client_id required"))?,
            )
        }
        "room.sync" => {
            let client = client(services)?;
            guard(&client, root, &room, bound(&room)?)?;
            let value = client.sync_room(
                payload["cursor"].as_str().map(str::to_owned),
                Some(bound(&room)?),
            )?;
            Ok(json!({"next_batch":value["next_batch"],"room":value["rooms"][bound(&room)?]}))
        }
        "room.timeline" | "room.event" => {
            let client = client(services)?;
            guard(&client, root, &room, bound(&room)?)?;
            let mut value = if kind == "room.timeline" {
                client.timeline(bound(&room)?, payload["cursor"].as_str().map(str::to_owned))?
            } else {
                let event = client.event(
                    bound(&room)?,
                    payload["event_id"]
                        .as_str()
                        .ok_or_else(|| invalid("event_id required"))?,
                )?;
                json!({"event":event,"source":matrix::source_text(chat::reference(&record), &event)?})
            };
            value["binding"] = serde_json::to_value(chat::reference(&record))?;
            Ok(value)
        }
        _ => Err(invalid("unknown Room query")),
    }?;
    access(shared, |s| {
        if s.read_stamp() == stamp {
            Ok(())
        } else {
            Err(chat::stale())
        }
    })?;
    Ok(result)
}

pub(crate) fn set_view_state(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    payload: &Value,
) -> Result<Value> {
    let project = payload["project_id"]
        .as_str()
        .ok_or_else(|| invalid("project_id required"))?;
    let id = payload["room_id"]
        .as_str()
        .ok_or_else(|| invalid("room_id required"))?;
    let (record, room) = access(shared, |s| chat::room(s, project, id))?;
    if payload["binding_version"].as_i64() != Some(record.version) {
        return Err(chat::stale());
    }
    let client = client(services)?;
    guard(&client, root, &room, bound(&room)?)?;
    client.set_view_state(
        bound(&room)?,
        payload["client_id"]
            .as_str()
            .ok_or_else(|| invalid("client_id required"))?,
        payload["draft"]
            .as_str()
            .ok_or_else(|| invalid("draft string required"))?,
        payload["read_cursor"].as_str().map(str::to_owned),
    )
}

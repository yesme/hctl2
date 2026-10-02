use super::*;

pub(crate) fn draft(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    input: DraftInput,
    actor: &TrustedActor,
    key: &str,
) -> Result<Value> {
    if key.trim().is_empty() {
        return Err(invalid("draft command key required"));
    }
    let id = foundation::canonical_json_sha256(&json!([input.project_id, actor.0.principal, key]))?;
    let observation_key = chat::key(
        Scope::Project(input.project_id.clone()),
        "brief_observation",
        &id,
    );
    if let Some(record) = access(shared, |s| s.get(&observation_key))? {
        let observation: Value = chat::decode(&record)?;
        if observation["input"] != serde_json::to_value(&input)? {
            return Err(reject(
                "IDEMPOTENCY_CONFLICT",
                "draft key has different input",
                "use_original_command",
            ));
        }
        return access(shared, |s| {
            let actor = chat::owner(actor, &input.project_id)?;
            let material = record
                .materials
                .first()
                .ok_or_else(|| invalid("draft material missing"))?;
            Ok(serde_json::from_slice(&s.read_material(&actor, material)?)?)
        });
    }
    let stamp = access(shared, |s| {
        chat::active_project(s, &input.project_id, input.project_version)?;
        chat::origin_checks(s, &input.project_id, &input.origin)?;
        Ok(s.read_stamp())
    })?;
    let mut unread = vec![];
    let sources = match &input.origin {
        Origin::Request { request, blockers } => {
            let mut selected = vec![];
            for reference in std::iter::once(request).chain(blockers) {
                match access(shared, |s| {
                    chat::request_text(s, reference, reference == request)
                }) {
                    Ok(source) => selected.push(source),
                    Err(e) if e.code == "SOURCE_UNAVAILABLE" => unread.push(Source::Object {
                        reference: reference.clone(),
                        content_digest: String::new(),
                    }),
                    Err(e) => return Err(e),
                }
            }
            for source in &input.messages {
                match resolve_message(shared, services, root, &input.project_id, source) {
                    Ok(text) => selected.push(text),
                    Err(e) if matches!(e.code, "CHAT_NOT_FOUND" | "SOURCE_UNAVAILABLE") => {
                        unread.push(source.clone())
                    }
                    Err(e) => return Err(e),
                }
            }
            selected
        }
        Origin::Room { room_id, .. } => {
            if !input.messages.is_empty() {
                return Err(invalid(
                    "optional Message references are only for Request origins",
                ));
            }
            let (r, room) = access(shared, |s| chat::room(s, &input.project_id, room_id))?;
            let client = client(services)?;
            guard(&client, root, &room, bound(&room)?)?;
            let selection = input
                .selection
                .as_ref()
                .ok_or_else(|| invalid("mechanical selection required"))?;
            let ids = match selection {
                Selection::Events { event_ids } => event_ids.clone(),
                Selection::Thread { event_id } => {
                    let mut events = vec![client.thread_root(bound(&room)?, event_id)?];
                    events.extend(client.thread_events(bound(&room)?, event_id)?);
                    select(&events, selection)?
                }
                _ => {
                    let external = bound(&room)?;
                    let (events, cursor, backwards, stop) = match selection {
                        Selection::Range { start, end } => (
                            vec![client.event(external, end)?],
                            client.context(external, end)?["start"]
                                .as_str()
                                .map(str::to_owned),
                            true,
                            Some(start.as_str()),
                        ),
                        Selection::Replies { event_id } => (
                            vec![client.event(external, event_id)?],
                            client.context(external, event_id)?["end"]
                                .as_str()
                                .map(str::to_owned),
                            false,
                            None,
                        ),
                        Selection::Mentions {
                            after: Some(event_id),
                            ..
                        } => (
                            vec![client.event(external, event_id)?],
                            client.context(external, event_id)?["end"]
                                .as_str()
                                .map(str::to_owned),
                            false,
                            None,
                        ),
                        _ => (vec![], None, true, None),
                    };
                    let events =
                        collect_window(events, cursor, backwards, stop, |cursor, direction| {
                            client.page(external, cursor, direction)
                        })?;
                    select(&events, selection)?
                }
            };
            let (selected, missing) = source_events(&client, &r, &room, ids)?;
            unread.extend(missing);
            selected
        }
    };
    finish_draft(shared, input, actor, &id, sources, unread, stamp)
}

pub(super) fn collect_window(
    mut events: Vec<Value>,
    mut cursor: Option<String>,
    backwards: bool,
    stop: Option<&str>,
    mut fetch: impl FnMut(Option<String>, bool) -> Result<Value>,
) -> Result<Vec<Value>> {
    for page in 0..100 {
        if stop.is_some_and(|id| events.iter().any(|e| e["event_id"] == id)) {
            break;
        }
        let result = fetch(cursor.clone(), backwards)?;
        let chunk = result["events"]
            .as_array()
            .ok_or_else(|| invalid("invalid timeline"))?;
        events.extend(chunk.clone());
        let next = result["next"].as_str().map(str::to_owned);
        if stop.is_some_and(|id| events.iter().any(|event| event["event_id"] == id))
            || chunk.is_empty()
            || next.is_none()
            || next == cursor
        {
            break;
        }
        if page == 99 {
            return Err(reject(
                "CHAT_HISTORY_LIMIT",
                "narrow mechanical source range",
                "select_event_ids",
            ));
        }
        cursor = next;
    }
    if backwards {
        events.reverse();
    }
    Ok(events)
}

fn source_events(
    client: &matrix::Client,
    r: &store::Record,
    room: &Room,
    ids: Vec<String>,
) -> Result<(Vec<SourceText>, Vec<Source>)> {
    let mut selected = vec![];
    let mut unread = vec![];
    for id in ids {
        match client
            .event(bound(room)?, &id)
            .and_then(|event| matrix::source_text(chat::reference(r), &event))
        {
            Ok(source) => selected.push(source),
            Err(e) if matches!(e.code, "CHAT_NOT_FOUND" | "SOURCE_UNAVAILABLE") => {
                unread.push(Source::Message {
                    binding: chat::reference(r),
                    event_id: id,
                    content_digest: String::new(),
                })
            }
            Err(e) => return Err(e),
        }
    }
    Ok((selected, unread))
}

fn finish_draft(
    shared: &Shared,
    input: DraftInput,
    actor: &TrustedActor,
    key: &str,
    sources: Vec<SourceText>,
    unread: Vec<Source>,
    stamp: (store::WriterGeneration, u64),
) -> Result<Value> {
    if !sources.is_empty() && unread.is_empty() {
        chat::validate_sources(&input.project_id, &input.origin, &sources)?;
    }
    let draft = chat::mechanical_draft(sources.clone(), unread);
    chat::verify_draft(&draft, &sources)?;
    let value = serde_json::to_value(&draft)?;
    access(shared, |s| {
        if s.read_stamp() != stamp {
            return Err(chat::stale());
        }
        // Auditable selection conclusion, not chat draft state or a second source of messages.
        let id = key.to_owned();
        let scope = Scope::Project(input.project_id.clone());
        let key = chat::key(scope.clone(), "brief_observation", &id);
        let actor = chat::owner(actor, &input.project_id)?;
        let material = s.save_material(
            s.generation(),
            &actor,
            &scope,
            &format!("brief-observation:{id}"),
            "draft",
            &foundation::canonical_json(&value)?,
        )?;
        let audit = json!({"input":input,"draft_material":material,"rule_reference":draft.rule_reference,"rule_digest":draft.rule_digest,
            "sources":sources.iter().map(|s| &s.source).collect::<Vec<_>>(),"unread":draft.unread,"conclusion":"mechanical_only; automatic_summary_not_configured"});
        let command = Command {
            command_id: format!("brief-observation:{id}"),
            idempotency_key: format!("brief-observation:{id}"),
            actor: actor.0.clone(),
            target: key.clone(),
            expected: Expected::Absent,
            binding: Reference {
                key: chat::key(Scope::Control, "module", "chat"),
                version: Version::State(1),
            },
            input_digest: Command::digest_input("chat.brief_observation", &audit)?,
            operation: "chat.brief_observation".into(),
            input: audit.clone(),
        };
        s.submit(s.generation(), &actor, &command, None, |tx| {
            tx.admit_material(&material)?;
            let mut record = chat::value_record(key, 1, &audit)?;
            record.materials.push(material);
            tx.put(&record)?;
            Ok(json!({"observation":id}))
        })?;
        Ok(())
    })?;
    Ok(value)
}

pub(super) fn select(events: &[Value], selection: &Selection) -> Result<Vec<String>> {
    let selected: Vec<&Value> = match selection {
        Selection::Range { start, end } => {
            let a = events
                .iter()
                .position(|e| e["event_id"] == *start)
                .ok_or_else(|| invalid("start event not found in server order"))?;
            let b = events
                .iter()
                .position(|e| e["event_id"] == *end)
                .ok_or_else(|| invalid("end event not found in server order"))?;
            if a > b {
                return Err(invalid("range end precedes start in server order"));
            }
            events[a..=b].iter().collect()
        }
        Selection::Replies { event_id } => events
            .iter()
            .filter(|e| {
                e["event_id"] == *event_id
                    || e["content"]["m.relates_to"]["m.in_reply_to"]["event_id"] == *event_id
            })
            .collect(),
        Selection::Thread { event_id } => events
            .iter()
            .filter(|e| {
                e["event_id"] == *event_id
                    || (e["content"]["m.relates_to"]["rel_type"] == "m.thread"
                        && e["content"]["m.relates_to"]["event_id"] == *event_id)
            })
            .collect(),
        Selection::Mentions { user_id, .. } => events
            .iter()
            .filter(|e| {
                e["content"]["m.mentions"]["user_ids"]
                    .as_array()
                    .is_some_and(|a| a.contains(&json!(user_id)))
            })
            .collect(),
        Selection::Events { event_ids } => return Ok(event_ids.clone()),
    };
    Ok(selected
        .into_iter()
        .filter(|e| e["type"] == "m.room.message")
        .filter_map(|e| e["event_id"].as_str().map(str::to_owned))
        .collect())
}

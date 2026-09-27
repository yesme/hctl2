use super::*;

pub(super) fn draft(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    input: DraftInput,
) -> Result<Value> {
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
            selected
        }
        Origin::MainRoom { room_id, .. } => {
            let (r, room) = access(shared, |s| chat::room(s, &input.project_id, room_id))?;
            let client = client(services)?;
            guard(&client, root, &room, bound(&room)?)?;
            let selection = input
                .selection
                .as_ref()
                .ok_or_else(|| invalid("mechanical selection required"))?;
            let ids = match selection {
                Selection::Events { event_ids } => event_ids.clone(),
                _ => {
                    let mut events = vec![];
                    let mut cursor = None;
                    for page in 0..100 {
                        let result = client.timeline(bound(&room)?, cursor.clone())?;
                        let chunk = result["events"]
                            .as_array()
                            .ok_or_else(|| invalid("invalid timeline"))?;
                        events.extend(chunk.clone());
                        let next = result["next"].as_str().map(str::to_owned);
                        if chunk.is_empty() || next.is_none() || next == cursor {
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
                    select(&events, selection)?
                }
            };
            let mut selected = vec![];
            for id in ids {
                match client
                    .event(bound(&room)?, &id)
                    .and_then(|event| matrix::source_text(chat::reference(&r), &event))
                {
                    Ok(source) => selected.push(source),
                    Err(e) if matches!(e.code, "CHAT_NOT_FOUND" | "SOURCE_UNAVAILABLE") => unread
                        .push(Source::Message {
                            binding: chat::reference(&r),
                            event_id: id,
                            content_digest: String::new(),
                        }),
                    Err(e) => return Err(e),
                }
            }
            selected
        }
    };
    if !sources.is_empty() {
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
        let audit = json!({"input":input,"rule_reference":draft.rule_reference,"rule_digest":draft.rule_digest,
            "sources":sources.iter().map(|s| &s.source).collect::<Vec<_>>(),"unread":draft.unread,"conclusion":"mechanical_only; automatic_summary_not_configured"});
        let id = foundation::canonical_json_sha256(&audit)?;
        let scope = Scope::Project(input.project_id.clone());
        let key = chat::key(scope.clone(), "brief_observation", &id);
        let actor = TrustedActor(Actor {
            principal: "chat-adapter".into(),
            source: ActorSource::ProviderEvent,
            permission_scope: vec![scope],
            authority: None,
        });
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
            tx.put(&chat::value_record(key, 1, &audit)?)?;
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
            .filter(|e| e["content"]["m.relates_to"]["m.in_reply_to"]["event_id"] == *event_id)
            .collect(),
        Selection::Mentions { user_id } => events
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

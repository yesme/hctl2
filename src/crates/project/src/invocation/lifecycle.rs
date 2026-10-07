use super::*;
use store::{EffectIntent, EffectState};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Pending,
    Running,
    WaitingInput,
    Lost,
    Completed,
    Failed,
    Cancelled,
}
impl State {
    pub fn terminal(self) -> bool {
        matches!(
            self,
            Self::Lost | Self::Completed | Self::Failed | Self::Cancelled
        )
    }
    pub fn allows(self, next: Self) -> bool {
        match self {
            Self::Pending => matches!(
                next,
                Self::Running | Self::Failed | Self::Cancelled | Self::Lost
            ),
            Self::Running => matches!(
                next,
                Self::WaitingInput | Self::Completed | Self::Failed | Self::Cancelled | Self::Lost
            ),
            Self::WaitingInput => matches!(
                next,
                Self::Running | Self::Completed | Self::Failed | Self::Cancelled | Self::Lost
            ),
            _ => false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lifecycle {
    pub state: State,
    pub reason: Option<String>,
}

pub fn state_key(project: &str, id: &str) -> store::ObjectKey {
    key(Scope::Project(project.into()), "invocation_state", id)
}
pub fn lifecycle(store: &Store, project: &str, id: &str) -> store::Result<(Record, Lifecycle)> {
    let record = required(store, &state_key(project, id))?;
    let state = decode(&record)?;
    Ok((record, state))
}

/// Only an already-confirmed step-4 activation can move pending to running.
pub fn record_started(
    store: &mut Store,
    actor: &TrustedActor,
    project: &str,
    id: &str,
    now_ms: u64,
) -> store::Result<Value> {
    let (root, invocation) = invocation(store, project, id)?;
    let scoped = reducer(actor, &root)?;
    let (state_record, current) = lifecycle(store, project, id)?;
    let activation_id = format!("activate:{id}");
    let (_, effect_state) = store.effect(&activation_id)?;
    let intent = required(store, &key(root.key.scope.clone(), "dispatch_intent", id))?;
    let value: Value = decode(&intent)?;
    let dispatch_id = value["dispatch"]
        .as_str()
        .ok_or_else(|| invalid("dispatch mapping required"))?;
    let dispatch = required(store, &key(root.key.scope.clone(), "dispatch", dispatch_id))?;
    let d: agency_proto::Dispatch = decode(&dispatch)?;
    if effect_state != EffectState::Confirmed
        || d.owner != invocation.spec.document.owner
        || d.spec_digest != invocation.spec.digest
        || !current_authorization(store, &reference(&root), now_ms)?
    {
        return Err(reject(
            "ACTIVATION_NOT_CONFIRMED",
            "exact activation must be confirmed",
            "read_back_original_intent",
        ));
    }
    let input = json!({"owner":reference(&root),"dispatch":reference(&dispatch)});
    let command = Command {
        command_id: format!("invocation-running:{id}"),
        idempotency_key: format!("invocation-running:{id}"),
        actor: scoped.0.clone(),
        target: state_record.key.clone(),
        expected: Expected::Exact(Version::State(1)),
        binding: reference(&root),
        operation: "invocation.running".into(),
        input_digest: Command::digest_input("invocation.running", &input)?,
        input,
    };
    store.submit(store.generation(), &scoped, &command, None, |tx| {
        if current.state != State::Pending
            || tx.get(&root.key)?.as_ref().map(reference) != Some(reference(&root))
        {
            return Err(stale());
        }
        tx.put(&value_record(
            state_record.key,
            2,
            &Lifecycle {
                state: State::Running,
                reason: None,
            },
        )?)?;
        Ok(json!({"invocation_id":id,"state":"running","state_version":2}))
    })
}

pub(super) fn reducer(actor: &TrustedActor, root: &Record) -> store::Result<TrustedActor> {
    if actor.0.source != store::ActorSource::InternalReducer
        || actor.0.authority
            != Some(Reference {
                key: root.key.clone(),
                version: Version::State(1),
            })
        || !actor.0.permission_scope.contains(&root.key.scope)
    {
        return Err(reject(
            "PERMISSION_DENIED",
            "original Invocation reducer required",
            "request_authorization",
        ));
    }
    Ok(TrustedActor(actor.0.clone()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct End {
    pub key: String,
    pub project_id: String,
    pub invocation_id: String,
    pub state_version: i64,
    pub outcome: State,
    pub reason: String,
}

/// Cancellation/failure/loss revoke authority and queue cleanup in one native transaction.
/// Completion deliberately has no entry here: it requires result admission.
pub fn end(store: &mut Store, actor: &TrustedActor, input: End) -> store::Result<Value> {
    if input.key.trim().is_empty()
        || input.reason.trim().is_empty()
        || !matches!(
            input.outcome,
            State::Cancelled | State::Failed | State::Lost
        )
    {
        return Err(invalid(
            "end needs a key, reason and cancel/fail/lost outcome; completion requires admitted result",
        ));
    }
    let (root, mut invocation) = invocation(store, &input.project_id, &input.invocation_id)?;
    let (state_record, current) = lifecycle(store, &input.project_id, &input.invocation_id)?;
    // A human can cancel. Failure/loss are reducer decisions under the original
    // authorization, not new client commands or claims made by a model.
    let scoped = if input.outcome == State::Cancelled {
        chat::owner(actor, &input.project_id)?
    } else if actor.0.source == store::ActorSource::InternalReducer
        && actor.0.authority
            == Some(Reference {
                key: root.key.clone(),
                version: Version::State(1),
            })
    {
        TrustedActor(actor.0.clone())
    } else {
        return Err(reject(
            "PERMISSION_DENIED",
            "failure/loss requires the authorized reducer",
            "request_authorization",
        ));
    };
    let next_state_version = input
        .state_version
        .checked_add(1)
        .filter(|_| input.state_version > 0)
        .ok_or_else(|| invalid("valid state version required"))?;
    let command_input = serde_json::to_value(&input)?;
    let command = Command {
        command_id: format!("invocation-end:{}", input.key),
        idempotency_key: input.key.clone(),
        actor: scoped.0.clone(),
        target: state_record.key.clone(),
        expected: Expected::Exact(Version::State(input.state_version)),
        binding: Reference {
            key: root.key.clone(),
            version: Version::State(1),
        },
        operation: "invocation.end".into(),
        input_digest: Command::digest_input("invocation.end", &command_input)?,
        input: command_input,
    };
    let prepare_id = format!("prepare:{}", input.invocation_id);
    let activation_id = format!("activate:{}", input.invocation_id);
    let mut unsent = vec![];
    for id in [&prepare_id, &activation_id] {
        if store.has_effect(id)? && store.effect(id)?.1 == EffectState::Pending {
            unsent.push(id.clone());
        }
    }
    let requires_cleanup = matches!(
        store.effect(&prepare_id)?.1,
        EffectState::Unknown | EffectState::Confirmed
    );
    invocation.authorization.valid = false;
    let next_owner_version = root
        .version
        .checked_add(1)
        .ok_or_else(|| invalid("owner version exhausted"))?;
    let mut revoked = value_record(root.key.clone(), next_owner_version, &invocation)?;
    revoked.sources = root.sources.clone();
    let next = value_record(
        state_record.key.clone(),
        next_state_version,
        &Lifecycle {
            state: input.outcome,
            reason: Some(input.reason.clone()),
        },
    )?;
    let cleanup = json!({"dispatch_intent":input.invocation_id,"owner":invocation.spec.document.owner,"isolate":true});
    let effect = EffectIntent {
        intent_id: format!("stop:{}", input.invocation_id),
        owner: reference(&root),
        binding: required(
            store,
            &key(
                Scope::Control,
                "agency_binding",
                &invocation.spec.document.binding.id,
            ),
        )
        .map(|r| reference(&r))?,
        operation: "agency.stop".into(),
        target: input.invocation_id.clone(),
        conflict_scope: format!("stop:{}", input.invocation_id),
        permission_scope: root.key.scope.clone(),
        input_digest: Command::digest_input("agency.stop", &cleanup)?,
        input: cleanup,
        idempotency_key: format!("stop:{}", input.invocation_id),
    };
    store.submit(store.generation(), &scoped, &command, None, |tx| {
        if !current.state.allows(input.outcome) || !decode::<Invocation>(&required_in(tx, &root.key)?)?.authorization.valid {
            return Err(reject("INVALID_TRANSITION", "Invocation is terminal or transition is illegal", "inspect_original_invocation"));
        }
        tx.put(&revoked)?;
        tx.put(&next)?;
        for id in &unsent { tx.cancel_pending_effect(id)?; }
        if requires_cleanup { tx.enqueue_effect(&effect)?; }
        Ok(json!({"invocation_id":input.invocation_id,"state":input.outcome,"state_version":next_state_version,"authorization_revoked":true,"cleanup_pending":requires_cleanup}))
    })
}
/// Read-only projection of the same rules `end`'s transaction enforces, so a
/// human confirms the real consequence. Preview writes nothing and grants
/// nothing; a queued stop is not a confirmed isolation. The declared state
/// version must be the current one — `end`'s compare-and-swap stays the authority
/// on what is still current when the confirmed cancellation is submitted.
pub fn cancel_preview(store: &Store, input: &End) -> store::Result<Value> {
    if input.key.trim().is_empty() || input.reason.trim().is_empty() {
        return Err(invalid("cancel needs a key and a reason"));
    }
    if input.outcome != State::Cancelled {
        return Err(invalid(
            "failure and loss stay reducer decisions; a human cancels",
        ));
    }
    let (root, invocation) = invocation(store, &input.project_id, &input.invocation_id)?;
    let (state_record, current) = lifecycle(store, &input.project_id, &input.invocation_id)?;
    if state_record.version != input.state_version {
        return Err(reject(
            "VERSION_CONFLICT",
            "cancellation names a stale state version",
            "preview_again",
        ));
    }
    if !current.state.allows(State::Cancelled) || !invocation.authorization.valid {
        return Err(reject(
            "INVALID_TRANSITION",
            "Invocation is terminal or transition is illegal",
            "inspect_original_invocation",
        ));
    }
    let prepare_id = format!("prepare:{}", input.invocation_id);
    let activation_id = format!("activate:{}", input.invocation_id);
    let mut cancelled_effects = vec![];
    for id in [&prepare_id, &activation_id] {
        if store.has_effect(id)? && store.effect(id)?.1 == EffectState::Pending {
            cancelled_effects.push(id.clone());
        }
    }
    Ok(json!({
        "invocation_id": input.invocation_id,
        "owner": reference(&root),
        "state": current.state,
        "reason": current.reason,
        "state_version": state_record.version,
        "outcome": State::Cancelled,
        "cancelled_effects": cancelled_effects,
        "cleanup_pending": matches!(store.effect(&prepare_id)?.1, EffectState::Unknown | EffectState::Confirmed),
        "isolation_confirmed": false,
    }))
}
fn required_in(
    tx: &store::CommandTransaction<'_>,
    key: &store::ObjectKey,
) -> store::Result<Record> {
    tx.get(key)?
        .ok_or_else(|| invalid("Invocation authorization missing"))
}

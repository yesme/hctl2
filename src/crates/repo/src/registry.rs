use foundation::{bytes_sha256, canonical_json, canonical_json_sha256};
use serde_json::{Value, json};
use store::{
    Command, EffectIntent, EffectState, Expected, ObjectKey, Readback, Record, RecordData,
    Reference, Scope, Store, TrustedActor, Version,
};

use crate::{
    Lifecycle, Platform, PlatformObservation, Prepared, Registration, Result, reject,
    validate_default,
};

pub fn key(repo_id: &str) -> ObjectKey {
    ObjectKey {
        scope: Scope::Repo(repo_id.into()),
        kind: "repo".into(),
        id: repo_id.into(),
    }
}
pub fn registration_id(control_id: &str, idempotency_key: &str) -> String {
    bytes_sha256(format!("repo\0{control_id}\0{idempotency_key}").as_bytes())
}
pub fn binding(repo_id: &str) -> Reference {
    Reference {
        key: ObjectKey {
            scope: Scope::Repo(repo_id.into()),
            kind: "platform_binding".into(),
            id: format!("{repo_id}:platform"),
        },
        version: Version::State(1),
    }
}
pub fn effect_id(repo_id: &str, stage: &str) -> String {
    format!("repo:{repo_id}:{stage}")
}

pub fn get(store: &Store, repo_id: &str) -> Result<Registration> {
    let record = store
        .get(&key(repo_id))?
        .ok_or_else(|| reject("REPO_NOT_FOUND", "Repo is not registered", "register_repo"))?;
    match record.data {
        RecordData::Repo {
            registration: Some(value),
            ..
        } => Ok(serde_json::from_value(value)?),
        _ => Err(reject(
            "REPO_NOT_READY",
            "Repo has no registration state",
            "inspect_repo",
        )),
    }
}
pub fn list(store: &Store) -> Result<Vec<Registration>> {
    store
        .list("repo")?
        .into_iter()
        .filter_map(|record| match record.data {
            RecordData::Repo {
                registration: Some(value),
                ..
            } => Some(serde_json::from_value(value).map_err(Into::into)),
            _ => None,
        })
        .collect()
}
/// Shared admission guard for the subsequent Project/Task/Run command packages.
pub fn require_active(store: &Store, repo_id: &str) -> Result<Registration> {
    let registration = get(store, repo_id)?;
    if registration.lifecycle != Lifecycle::Active || registration.abandoned {
        return Err(reject(
            "REPO_PENDING",
            "pending Repo cannot accept Project, Task or Run",
            "finish_registration",
        ));
    }
    Ok(registration)
}

fn scoped_actor(actor: &TrustedActor, repo_id: &str) -> Result<TrustedActor> {
    if !actor.0.permission_scope.contains(&Scope::Control) {
        return Err(reject(
            "PERMISSION_DENIED",
            "Repo registration requires control owner",
            "request_authorization",
        ));
    }
    let mut actor = actor.0.clone();
    actor.permission_scope.push(Scope::Repo(repo_id.into()));
    Ok(TrustedActor(actor))
}
fn record(registration: &Registration) -> Result<Record> {
    let value = serde_json::to_value(registration)?;
    Ok(Record {
        key: key(&registration.repo_id),
        version: registration.version,
        revision_digest: canonical_json_sha256(&value)?,
        data: RecordData::Repo {
            platform_binding: registration
                .observed
                .as_ref()
                .map(|_| binding(&registration.repo_id)),
            registration: Some(value),
        },
        sources: Vec::new(),
        materials: vec![registration.config.clone()],
    })
}
fn command(
    actor: &TrustedActor,
    id: &str,
    command_id: &str,
    idem: &str,
    expected: Expected,
    operation: &str,
    input: Value,
) -> Result<Command> {
    Ok(Command {
        command_id: command_id.into(),
        idempotency_key: idem.into(),
        actor: actor.0.clone(),
        target: key(id),
        expected,
        binding: binding(id),
        input_digest: Command::digest_input(operation, &input)?,
        operation: operation.into(),
        input,
    })
}
fn effect(registration: &Registration, stage: &str) -> Result<EffectIntent> {
    let input = serde_json::to_value(&registration.prepared)?;
    let operation = format!("repo.register.{stage}");
    let request = &registration.prepared.request;
    let target = format!(
        "{:?}:{}:{}",
        registration.prepared.platform,
        request
            .instance
            .as_deref()
            .unwrap_or(&registration.config.control_id),
        request
            .platform_path
            .as_deref()
            .unwrap_or(&registration.repo_id)
    );
    Ok(EffectIntent {
        intent_id: effect_id(&registration.repo_id, stage),
        owner: Reference {
            key: key(&registration.repo_id),
            version: Version::State(registration.version),
        },
        binding: binding(&registration.repo_id),
        operation: operation.clone(),
        conflict_scope: format!("registration:{target}"),
        target,
        permission_scope: Scope::Repo(registration.repo_id.clone()),
        input_digest: Command::digest_input(&operation, &input)?,
        input,
        idempotency_key: effect_id(&registration.repo_id, stage),
    })
}

/// Persist the original command, configuration and first effect in one admission transaction.
pub fn admit(
    store: &mut Store,
    actor: &TrustedActor,
    command_id: &str,
    idem: &str,
    prepared: Prepared,
) -> Result<Registration> {
    let id = registration_id(store.control_id(), idem);
    let actor = scoped_actor(actor, &id)?;
    let input = serde_json::to_value(&prepared.request)?;
    let cmd = command(
        &actor,
        &id,
        command_id,
        idem,
        Expected::Absent,
        "repo.register",
        input,
    )?;
    // Retries freeze the *original* preview and material, not observations made after admission.
    let existing = store.get(&key(&id))?;
    let registration = if existing.is_some() {
        get(store, &id)?
    } else {
        let config = store.save_material(
            store.generation(),
            &actor,
            &Scope::Repo(id.clone()),
            idem,
            "registration",
            &canonical_json(&serde_json::to_value(&prepared)?)?,
        )?;
        Registration {
            repo_id: id.clone(),
            version: 1,
            lifecycle: if prepared.platform == Platform::None {
                Lifecycle::Active
            } else {
                Lifecycle::Pending
            },
            abandoned: false,
            command_id: command_id.into(),
            idempotency_key: idem.into(),
            actor: actor.0.clone(),
            sources: prepared.sources.clone(),
            prepared,
            config,
            observed: None,
            delivered: false,
            residual: None,
            residual_target: None,
        }
    };
    let result = store.submit(store.generation(), &actor, &cmd, None, |tx| {
        tx.admit_material(&registration.config)?;
        tx.put(&record(&registration)?)?;
        if registration.prepared.platform != Platform::None {
            tx.enqueue_effect(&effect(&registration, "platform")?)?;
        }
        Ok(json!({"repo_id":id}))
    });
    if let Err(error) = result {
        if error.code == "EFFECT_CONFLICT" {
            let expected = effect(&registration, "platform")?;
            for pending in store.pending_effects()? {
                let (intent, _) = store.effect(&pending)?;
                if intent.conflict_scope == expected.conflict_scope {
                    return Err(reject(
                        "REGISTRATION_TARGET_BUSY",
                        format!(
                            "target is occupied by registration {}; inspect/resume that original registration before retrying",
                            intent.owner.key.id
                        ),
                        "inspect_original_registration",
                    ));
                }
            }
        }
        return Err(error);
    }
    get(store, &id)
}

fn check_authorization(store: &Store, actor: &TrustedActor, reg: &Registration) -> Result<()> {
    let actor = scoped_actor(actor, &reg.repo_id)?;
    if actor.0.principal != reg.actor.principal {
        return Err(reject(
            "PERMISSION_DENIED",
            "registration belongs to another human",
            "request_authorization",
        ));
    }
    let frozen: Prepared = serde_json::from_slice(&store.read_material(&actor, &reg.config)?)?;
    if frozen != reg.prepared {
        return Err(reject(
            "FROZEN_INPUT_CHANGED",
            "registration differs from its admitted preview",
            "restore_registration",
        ));
    }
    Ok(())
}

/// The reducer rechecks the original human authorization and immutable material on every retry.
pub fn begin_step(
    store: &mut Store,
    actor: &TrustedActor,
    id: &str,
    stage: &str,
) -> Result<EffectState> {
    let reg = get(store, id)?;
    check_authorization(store, actor, &reg)?;
    if reg.abandoned || reg.lifecycle != Lifecycle::Pending {
        return Err(reject(
            "REGISTRATION_TERMINAL",
            "registration is no longer dispatchable",
            "inspect_repo",
        ));
    }
    let id = effect_id(id, stage);
    let (intent, state) = store.effect(&id)?;
    let still_authorized = intent.owner.key == key(&reg.repo_id)
        && intent.binding == binding(&reg.repo_id)
        && intent.input == serde_json::to_value(&reg.prepared)?;
    if !still_authorized {
        return Err(reject(
            "FROZEN_INPUT_CHANGED",
            "effect differs from the admitted registration",
            "restore_registration",
        ));
    }
    if state == EffectState::Pending {
        store.resume_pending_effect(store.generation(), &id, still_authorized)?;
        store.begin_effect(store.generation(), &id)?;
    }
    Ok(state)
}

fn observe_sources(reg: &mut Registration, observed: &PlatformObservation) {
    reg.sources = reg.prepared.sources.clone();
    for source in &mut reg.sources {
        source.available = observed.has_issues;
        source.create = observed.can_write_issues;
        source.field_writeback = observed.can_write_issues;
        source.can_claim = observed.has_issues;
        source.actual_source_and_scope = format!("{}/{}", observed.instance, observed.stable_id);
    }
}

/// Refresh non-identity observations without changing the frozen preview or effect input.
pub fn refresh_platform(
    store: &mut Store,
    id: &str,
    observed: PlatformObservation,
) -> Result<Registration> {
    let mut reg = get(store, id)?;
    if !reg
        .observed
        .as_ref()
        .is_some_and(|old| old.same_repository(&observed))
    {
        return Err(reject(
            "PLATFORM_ID_MISMATCH",
            "platform identity changed since creation",
            "read_back_original_intent",
        ));
    }
    if reg.observed.as_ref() == Some(&observed) {
        return Ok(reg);
    }
    let old_version = reg.version;
    observe_sources(&mut reg, &observed);
    reg.observed = Some(observed.clone());
    reg.version += 1;
    let actor = TrustedActor(reg.actor.clone());
    let input = serde_json::to_value(observed)?;
    let cmd_id = format!("{}:{old_version}", effect_id(id, "refresh"));
    let cmd = command(
        &actor,
        id,
        &cmd_id,
        &cmd_id,
        Expected::Exact(Version::State(old_version)),
        "repo.platform.refresh",
        input,
    )?;
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        tx.put(&record(&reg)?)?;
        Ok(json!({"repo_id":id}))
    })?;
    get(store, id)
}

/// Adapter readback is consumed only by control, never by a submitted client observation.
pub fn confirm_platform(
    store: &mut Store,
    id: &str,
    observed: PlatformObservation,
) -> Result<Registration> {
    let mut reg = get(store, id)?;
    if reg.abandoned {
        return Err(reject(
            "REGISTRATION_ABANDONED",
            "registration abandoned",
            "inspect_residual",
        ));
    }
    if reg.observed.as_ref() == Some(&observed) {
        return Ok(reg);
    }
    if observed.stable_id.is_empty()
        || observed.instance.is_empty()
        || observed.account_id.is_empty()
    {
        return Err(reject(
            "PLATFORM_ID_REQUIRED",
            "readback lacks platform identity",
            "read_back_original_intent",
        ));
    }
    if reg.prepared.platform == Platform::Github
        && (reg.prepared.request.platform_repo_id.as_ref() != Some(&observed.stable_id)
            || reg.prepared.request.instance.as_ref() != Some(&observed.instance))
    {
        return Err(reject(
            "PLATFORM_ID_MISMATCH",
            "platform readback differs from declared identity",
            "preview_correct_identity",
        ));
    }
    observe_sources(&mut reg, &observed);
    validate_default(reg.prepared.request.default_source.as_deref(), &reg.sources)?;
    let old_version = reg.version;
    reg.version += 1;
    reg.observed = Some(observed.clone());
    if reg.prepared.platform == Platform::Github {
        reg.delivered = true;
        reg.lifecycle = Lifecycle::Active;
    }
    let actor = TrustedActor(reg.actor.clone());
    let cmd_id = effect_id(id, "platform-confirm");
    let cmd = command(
        &actor,
        id,
        &cmd_id,
        &cmd_id,
        Expected::Exact(Version::State(old_version)),
        "repo.platform.confirm",
        serde_json::to_value(&observed)?,
    )?;
    let (intent, _) = store.effect(&effect_id(id, "platform"))?;
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        tx.confirm_effect(
            &intent.intent_id,
            &Readback::Confirmed {
                binding: intent.binding.clone(),
                target: intent.target.clone(),
                input_digest: intent.input_digest.clone(),
                result: serde_json::to_value(&observed)?,
            },
        )?;
        let checks = if reg.prepared.platform == Platform::Local {
            "external_status_only"
        } else {
            "platform_checks"
        };
        let verified_reviews = reg.prepared.platform == Platform::Github;
        let value = json!({
            "provider": reg.prepared.platform, "observation": observed,
            "account_mappings": {},
            "capabilities": {
                "review_threads": verified_reviews, "formal_reviews": verified_reviews, "checks": checks,
                "remote_merge": verified_reviews, "identity_mapping": false, "expected_target_head": false,
                "review_text_readback": verified_reviews, "protection_readback": verified_reviews
            }
        });
        tx.put(&Record {
            key: binding(id).key,
            version: 1,
            revision_digest: canonical_json_sha256(&value)?,
            data: RecordData::Value { value },
            sources: Vec::new(),
            materials: Vec::new(),
        })?;
        if !observed.credential_ref.is_empty() {
            tx.bind_secret(&binding(id), &observed.credential_ref)?;
        }
        tx.put(&record(&reg)?)?;
        if reg.prepared.platform == Platform::Local {
            tx.enqueue_effect(&effect(&reg, "delivery")?)?;
        }
        Ok(json!({"repo_id":id}))
    })?;
    get(store, id)
}

pub fn confirm_delivery(store: &mut Store, id: &str) -> Result<Registration> {
    let mut reg = get(store, id)?;
    if reg.delivered {
        return Ok(reg);
    }
    if reg.abandoned {
        return Err(reject(
            "REGISTRATION_ABANDONED",
            "registration abandoned",
            "inspect_residual",
        ));
    }
    let old_version = reg.version;
    reg.version += 1;
    reg.delivered = true;
    let actor = TrustedActor(reg.actor.clone());
    let cmd_id = effect_id(id, "delivery-confirm");
    let cmd = command(
        &actor,
        id,
        &cmd_id,
        &cmd_id,
        Expected::Exact(Version::State(old_version)),
        "repo.delivery.confirm",
        json!({"refs":reg.prepared.local.as_ref().map(|l| &l.refs)}),
    )?;
    let (intent, _) = store.effect(&effect_id(id, "delivery"))?;
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        tx.confirm_effect(
            &intent.intent_id,
            &Readback::Confirmed {
                binding: intent.binding.clone(),
                target: intent.target.clone(),
                input_digest: intent.input_digest.clone(),
                result: cmd.input.clone(),
            },
        )?;
        tx.put(&record(&reg)?)?;
        Ok(json!({"repo_id":id}))
    })?;
    get(store, id)
}

/// Human declares the newly assigned local platform ID after create/readback.
/// This second preview also lets the human abandon, without deleting external content.
pub enum FinishChoice<'a> {
    Confirm(&'a str),
    Abandon,
}

pub fn finish(
    store: &mut Store,
    actor: &TrustedActor,
    id: &str,
    expected_version: i64,
    choice: FinishChoice<'_>,
    command_id: &str,
    idem: &str,
) -> Result<Registration> {
    let (stable_id, abandon) = match choice {
        FinishChoice::Confirm(id) => (Some(id), false),
        FinishChoice::Abandon => (None, true),
    };
    let mut reg = get(store, id)?;
    let actor = scoped_actor(actor, id)?;
    let mut unsent = Vec::new();
    if abandon && reg.prepared.platform != Platform::None {
        let mut stages = vec!["platform"];
        if reg.prepared.platform == Platform::Local && reg.observed.is_some() {
            stages.push("delivery");
        }
        for stage in stages {
            let effect = effect_id(id, stage);
            if store.effect(&effect)?.1 == EffectState::Pending {
                unsent.push(effect);
            }
        }
    }
    let residual_target = if abandon && reg.prepared.platform == Platform::Local {
        Some(store.effect(&effect_id(id, "platform"))?.0.target)
    } else {
        None
    };
    let cmd = command(
        &actor,
        id,
        command_id,
        idem,
        Expected::Exact(Version::State(expected_version)),
        if abandon {
            "repo.abandon"
        } else {
            "repo.confirm"
        },
        json!({"repo_id":id,"version":expected_version,"platform_repo_id":stable_id}),
    )?;
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        if reg.lifecycle == Lifecycle::Active || reg.abandoned {
            return Err(reject(
                "REGISTRATION_TERMINAL",
                "registration no longer pending",
                "inspect_repo",
            ));
        }
        if abandon {
            for id in &unsent {
                tx.cancel_pending_effect(id)?;
            }
            reg.abandoned = true;
            reg.residual = reg.observed.clone();
            reg.residual_target = residual_target;
        } else {
            if !reg.delivered
                || stable_id.is_none()
                || reg.observed.as_ref().map(|o| o.stable_id.as_str()) != stable_id
            {
                return Err(reject(
                    "REGISTRATION_UNCONFIRMED",
                    "declare observed platform ID after verified delivery",
                    "inspect_repo_and_confirm",
                ));
            }
            reg.lifecycle = Lifecycle::Active;
        }
        reg.version += 1;
        tx.put(&record(&reg)?)?;
        Ok(json!({"repo_id":id}))
    })?;
    get(store, id)
}

/// A resume of an abandoned registration can only record verified residuals, never dispatch.
pub fn reconcile_residual(
    store: &mut Store,
    actor: &TrustedActor,
    id: &str,
    observed: PlatformObservation,
    delivery_verified: bool,
) -> Result<Registration> {
    let mut reg = get(store, id)?;
    check_authorization(store, actor, &reg)?;
    if !reg.abandoned {
        return Err(reject(
            "INVALID_INPUT",
            "registration is not abandoned",
            "inspect_repo",
        ));
    }
    if reg
        .observed
        .as_ref()
        .or(reg.residual.as_ref())
        .is_some_and(|old| !old.same_repository(&observed))
    {
        return Err(reject(
            "PLATFORM_ID_MISMATCH",
            "residual identity changed",
            "inspect_residual",
        ));
    }
    if observed.stable_id.is_empty()
        || observed.instance.is_empty()
        || (reg.prepared.platform == Platform::Github
            && (reg.prepared.request.platform_repo_id.as_ref() != Some(&observed.stable_id)
                || reg.prepared.request.instance.as_ref() != Some(&observed.instance)))
    {
        return Err(reject(
            "PLATFORM_ID_MISMATCH",
            "residual differs from declared identity",
            "inspect_residual",
        ));
    }
    let mut confirmed = Vec::new();
    for pending_id in store.pending_effects()? {
        let (intent, state) = store.effect(&pending_id)?;
        if intent.owner.key == key(&reg.repo_id)
            && state == EffectState::Unknown
            && (pending_id == effect_id(id, "platform")
                || (pending_id == effect_id(id, "delivery") && delivery_verified))
        {
            confirmed.push(intent);
        }
    }
    if confirmed.is_empty() && reg.residual.as_ref() == Some(&observed) {
        return Ok(reg);
    }
    let actor = scoped_actor(actor, id)?;
    let old_version = reg.version;
    reg.residual = Some(observed.clone());
    reg.version += 1;
    let input = json!({"observed":observed,"delivery_verified":delivery_verified});
    let cmd_id = format!("{}:{old_version}", effect_id(id, "residual"));
    let cmd = command(
        &actor,
        id,
        &cmd_id,
        &cmd_id,
        Expected::Exact(Version::State(old_version)),
        "repo.residual.readback",
        input,
    )?;
    store.submit(store.generation(), &actor, &cmd, None, |tx| {
        for intent in &confirmed {
            tx.confirm_effect(
                &intent.intent_id,
                &Readback::Confirmed {
                    binding: intent.binding.clone(),
                    target: intent.target.clone(),
                    input_digest: intent.input_digest.clone(),
                    result: json!({"residual":observed,"delivery_verified":delivery_verified}),
                },
            )?;
        }
        tx.put(&record(&reg)?)?;
        Ok(json!({"repo_id":id}))
    })?;
    get(store, id)
}

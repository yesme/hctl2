//! Local consumption and control-side port. No Agency service implementation dependency.
use agency_proto::{Catalog, Pair, Pairing, PortError, client::Client};
use foundation::SecretStore;
use participant::{invalid, reject};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use store::{Store, StoreError, TrustedActor};
use tokio::sync::Mutex;

/// `agency start` itself waits about five seconds for readiness. This bound sits above that.
const AGENCY_START_TIMEOUT: Duration = Duration::from_secs(30);

fn err(e: PortError) -> StoreError {
    participant::port_error(e)
}
fn secrets(root: &Path) -> SecretStore {
    SecretStore::user_file("agency-pairing", root.join("secrets"))
}
fn optional_secret(root: &Path, account: &str) -> store::Result<Option<Vec<u8>>> {
    match secrets(root).get(account) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(foundation::FoundationError::Io(error))
            if error.kind() == std::io::ErrorKind::NotFound =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}
pub fn paired_client(root: &Path, binding: &str) -> store::Result<Client> {
    let bytes = secrets(root).get(binding)?;
    let pairing: Pairing = serde_json::from_slice(&bytes)?;
    Ok(Client::new(pairing.endpoint.into(), pairing.key))
}

pub(crate) fn signed_ticket(
    root: &Path,
    actor: &TrustedActor,
    dispatch: &agency_proto::Dispatch,
    generation: store::WriterGeneration,
    permissions: Vec<agency_proto::Permission>,
    expires_ms: u64,
) -> store::Result<agency_proto::Ticket> {
    let pairing: Pairing = serde_json::from_slice(&secrets(root).get(&dispatch.binding.id)?)?;
    agency_proto::Ticket::sign(
        agency_proto::TicketClaims {
            id: format!("{}:{expires_ms}", dispatch.reference),
            actor: actor.0.principal.clone(),
            dispatch: dispatch.reference.clone(),
            owner: dispatch.owner.clone(),
            spec_digest: dispatch.spec_digest.clone(),
            writer_generation: u64::try_from(generation.0)
                .map_err(|_| invalid("writer generation"))?,
            permissions,
            input_lease: None,
            expires_ms,
        },
        pairing.key.as_bytes(),
    )
    .map_err(err)
}
pub async fn query(
    shared: &Arc<Mutex<Option<Store>>>,
    root: &Path,
    kind: &str,
    payload: &Value,
) -> store::Result<Value> {
    if kind == "agency.bindings" || kind == "profession.list" {
        let state = shared.lock().await;
        let store = state.as_ref().ok_or_else(|| invalid("store not ready"))?;
        return Ok(
            json!({"records":store.list(if kind=="agency.bindings" {"agency_binding"} else {"profession"})?}),
        );
    }
    let binding = text(payload, "binding_id")?;
    let catalog: Catalog = paired_client(root, binding)?
        .call("catalog", &json!({}))
        .await
        .map_err(err)?;
    Ok(serde_json::to_value(catalog)?)
}
pub async fn submit(
    shared: &Arc<Mutex<Option<Store>>>,
    root: &Path,
    operation: &str,
    payload: &Value,
    actor: &TrustedActor,
    key: &str,
) -> store::Result<Value> {
    if key.is_empty() {
        return Err(invalid("idempotency key required"));
    }
    let id = text(payload, "binding_id")?;
    if operation == "agency.pair" {
        let agency_root = payload
            .get("agency_root")
            .and_then(Value::as_str)
            .map_or_else(
                || std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".hctl2/agency")),
                |s| Some(PathBuf::from(s)),
            )
            .ok_or_else(|| invalid("local Agency root required"))?;
        ensure_local(&agency_root).await?;
        let bootstrap = std::fs::read_to_string(agency_root.join("pair.key"))?;
        let admin = Client::new(
            agency_proto::client::admin_endpoint(&agency_root).map_err(err)?,
            bootstrap,
        );
        // Serialize first-pair credential creation across concurrent clients of this writer.
        let (control_id, generation, tenant_key) = {
            let state = shared.lock().await;
            let store = state.as_ref().ok_or_else(|| invalid("store not ready"))?;
            let previous: Option<Pairing> = optional_secret(root, id)?
                .map(|bytes| serde_json::from_slice(&bytes))
                .transpose()?;
            let socket_directory =
                agency_proto::client::socket_directory(&agency_root).map_err(err)?;
            if previous.as_ref().is_some_and(|pairing| {
                Path::new(&pairing.endpoint).parent() != Some(socket_directory.as_path())
            }) {
                return Err(reject(
                    "BINDING_IMMUTABLE",
                    "a different provider requires hctl2 agency pair --binding-id <new-id> --key <new-key>",
                    "hctl2 agency pair",
                ));
            }
            let account = format!(
                "tenant:{}:{}",
                store.control_id(),
                agency_proto::hash(agency_root.canonicalize()?.as_os_str().as_encoded_bytes())
            );
            let tenant_key = match optional_secret(root, &account)? {
                Some(bytes) => String::from_utf8(bytes)
                    .map_err(|_| invalid("stored tenant credential invalid"))?,
                None => {
                    let key = if let Some(pairing) = previous {
                        pairing.key
                    } else {
                        agency_proto::client::new_credential().map_err(err)?
                    };
                    secrets(root).set(&account, key.as_bytes())?;
                    key
                }
            };
            (
                store.control_id().to_owned(),
                store.generation(),
                tenant_key,
            )
        };
        let pairing: Pairing = admin
            .call(
                "pair",
                &Pair {
                    control_id,
                    tenant_key,
                },
            )
            .await
            .map_err(err)?;
        let client = Client::new(pairing.endpoint.clone().into(), pairing.key.clone());
        if let Some(bytes) = optional_secret(root, id)? {
            let previous: Pairing = serde_json::from_slice(&bytes)?;
            if previous.endpoint != pairing.endpoint || previous.key != pairing.key {
                return Err(reject(
                    "BINDING_IMMUTABLE",
                    "a different provider requires hctl2 agency pair --binding-id <new-id> --key <new-key>",
                    "hctl2 agency pair",
                ));
            }
        }
        let _: Value = client
            .call(
                "fence",
                &agency_proto::Fence {
                    writer_generation: u64::try_from(generation.0)
                        .map_err(|_| invalid("writer generation"))?,
                },
            )
            .await
            .map_err(err)?;
        let catalog: Catalog = client.call("catalog", &json!({})).await.map_err(err)?;
        // Credentials are operational secrets, excluded from the control backup's governance facts.
        let mut state = shared.lock().await;
        let store = state.as_mut().ok_or_else(|| invalid("store not ready"))?;
        if store.generation() != generation {
            return Err(reject(
                "WRITER_STALE",
                "control writer changed during pairing",
                "reconcile_control_writer",
            ));
        }
        let record = participant::accept_binding(
            store,
            actor,
            key,
            participant::Binding {
                id: id.into(),
                protocol: agency_proto::PROTOCOL.into(),
                catalog,
            },
        )?;
        secrets(root).set(id, &serde_json::to_vec(&pairing)?)?;
        secrets(root).set(
            &format!("{id}:local-root"),
            agency_root.as_os_str().as_encoded_bytes(),
        )?;
        Ok(json!({"binding":record,"paired":true}))
    } else if operation == "profession.accept" {
        let reference: agency_proto::FrozenRef = serde_json::from_value(
            payload
                .get("profession")
                .cloned()
                .ok_or_else(|| invalid("exact Profession reference required"))?,
        )?;
        let catalog: Catalog = paired_client(root, id)?
            .call("catalog", &json!({}))
            .await
            .map_err(err)?;
        let profession = catalog
            .professions
            .iter()
            .find(|p| p.reference == reference)
            .ok_or_else(|| invalid("Profession candidate changed"))?;
        let mut state = shared.lock().await;
        let store = state.as_mut().ok_or_else(|| invalid("store not ready"))?;
        let record = participant::accept_profession(store, actor, key, id, profession)?;
        Ok(json!({"profession":record}))
    } else {
        Err(invalid("unknown Agency operation"))
    }
}
async fn ensure_local(root: &Path) -> store::Result<()> {
    if root.exists()
        && let Ok(key) = std::fs::read_to_string(root.join("pair.key"))
    {
        let client = Client::new(
            agency_proto::client::admin_endpoint(root).map_err(err)?,
            key,
        );
        if client.call::<_, Value>("catalog", &json!({})).await.is_ok() {
            return Ok(());
        }
    }
    let binary = std::env::current_exe()?
        .parent()
        .ok_or_else(|| invalid("installation directory missing"))?
        .join("agency");
    if !binary.is_file() {
        return Err(reject(
            "AGENCY_NOT_INSTALLED",
            "Agency binary is absent from this installation",
            "install_complete_package_or_start_agency",
        ));
    }
    let root = root.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut command = std::process::Command::new(binary);
        command.arg("--root").arg(root).arg("start");
        run_agency_command(command, AGENCY_START_TIMEOUT)
    })
    .await
    .map_err(|_| invalid("Agency start worker failed"))?
}

fn run_agency_command(
    mut command: std::process::Command,
    time_limit: Duration,
) -> store::Result<()> {
    match foundation::command::run_bounded(&mut command, None, time_limit).map_err(|error| {
        reject(
            "AGENCY_NOT_READY",
            format!("cannot start local Agency: {error}"),
            "run_agency_serve",
        )
    })? {
        foundation::command::CommandEnd::Finished(output) if output.status.success() => Ok(()),
        foundation::command::CommandEnd::Finished(output) => Err(reject(
            "AGENCY_NOT_READY",
            format!("local Agency start failed ({})", output.status),
            "run_agency_serve",
        )),
        foundation::command::CommandEnd::TimedOut => Err(reject(
            "AGENCY_START_TIMEOUT",
            "local Agency start timed out",
            "run_agency_serve",
        )),
        foundation::command::CommandEnd::OutputLimit => Err(reject(
            "AGENCY_NOT_READY",
            "local Agency start output exceeded limit",
            "run_agency_serve",
        )),
    }
}
fn text<'a>(payload: &'a Value, key: &str) -> store::Result<&'a str> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid(format!("missing {key}")))
}

fn stored_rejection(store: &Store, effect: &store::EffectIntent) -> store::Result<StoreError> {
    let record = store
        .get(&participant::key(
            effect.owner.key.scope.clone(),
            "dispatch_rejection",
            &effect.intent_id,
        ))?
        .ok_or_else(|| invalid("rejected dispatch lacks its stored refusal"))?;
    let error: PortError = participant::decode(&record)?;
    Ok(err(error))
}

async fn finish_rejection(
    shared: &Arc<Mutex<Option<Store>>>,
    actor: &TrustedActor,
    generation: store::WriterGeneration,
    effect: &store::EffectIntent,
    error: &PortError,
) -> store::Result<()> {
    let mut lock = shared.lock().await;
    let store = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
    let target = participant::key(
        effect.owner.key.scope.clone(),
        "dispatch_rejection",
        &effect.intent_id,
    );
    let input = serde_json::to_value(error)?;
    let command = store::Command {
        command_id: format!("refusal:{}", effect.intent_id),
        idempotency_key: format!("refusal:{}", effect.intent_id),
        actor: actor.0.clone(),
        target: target.clone(),
        expected: store::Expected::Absent,
        binding: effect.binding.clone(),
        input_digest: store::Command::digest_input("dispatch.rejected", &input)?,
        operation: "dispatch.rejected".into(),
        input,
    };
    let record = participant::value(target, 1, error)?;
    store.submit(generation, actor, &command, None, |tx| {
        tx.confirm_effect(
            &effect.intent_id,
            &store::Readback::Rejected {
                binding: effect.binding.clone(),
                target: effect.target.clone(),
                input_digest: effect.input_digest.clone(),
                result: serde_json::to_value(error)?,
            },
        )?;
        tx.put(&record)?;
        Ok(json!({"rejected":error}))
    })?;
    Ok(())
}

async fn dispatch_reply<T>(
    result: std::result::Result<T, agency_proto::client::CallFailure>,
    shared: &Arc<Mutex<Option<Store>>>,
    actor: &TrustedActor,
    generation: store::WriterGeneration,
    effect: &store::EffectIntent,
    state: store::EffectState,
) -> store::Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(agency_proto::client::CallFailure::ResponseError(error))
            if state == store::EffectState::Pending
                && refusal_before_effect(effect, &error.code) =>
        {
            finish_rejection(shared, actor, generation, effect, &error).await?;
            Err(err(error))
        }
        // A lookup refusal says nothing about whether the original write took place.
        Err(error) => Err(err(error.into_error())),
    }
}

fn refusal_before_effect(effect: &store::EffectIntent, code: &str) -> bool {
    matches!(
        code,
        "PAIRING_REQUIRED" | "PROTOCOL_MISMATCH" | "WRITER_STALE" | "DEADLINE_EXPIRED"
    ) || (effect.operation == "agency.prepare"
        && matches!(
            code,
            "CAPABILITY_MISSING"
                | "PROFESSION_CHANGED"
                | "SKILL_MISSING"
                | "SKILL_DIGEST_MISMATCH"
                | "BUNDLE_MISMATCH"
                | "DIGEST_MISMATCH"
                | "DELIVERY_DIGEST_MISMATCH"
                | "MATERIAL_NOT_DELIVERED"
                | "BUDGET_EXCEEDED"
                | "IDEMPOTENCY_CONFLICT"
        ))
        || (effect.operation == "agency.activate" && code == "DISPATCH_NOT_FOUND")
}

async fn fence_and_begin_effect(
    shared: &Arc<Mutex<Option<Store>>>,
    client: &Client,
    generation: store::WriterGeneration,
    effect: &store::EffectIntent,
) -> store::Result<()> {
    // A restarted control can reach Agency before the periodic writer reconciliation.
    // Failure here leaves the business action Pending: it has not been attempted.
    let _: Value = client
        .call(
            "fence",
            &agency_proto::Fence {
                writer_generation: u64::try_from(generation.0)
                    .map_err(|_| invalid("writer generation"))?,
            },
        )
        .await
        .map_err(err)?;
    let mut lock = shared.lock().await;
    let store = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
    // Recheck authority after the network await, then atomically mark the attempt unknown.
    current_owner(store, &effect.owner)?;
    store.resume_pending_effect(generation, &effect.intent_id, true)?;
    store.begin_effect(generation, &effect.intent_id)?;
    Ok(())
}

/// Package 5 calls this only after persisting its authorized owner and dispatch intent.
/// An unknown prepare is read back by key, not submitted a second time.
pub async fn deliver_prepare(
    shared: &Arc<Mutex<Option<Store>>>,
    root: &Path,
    actor: &TrustedActor,
    intent: &store::Record,
) -> store::Result<store::Record> {
    let original: Value = participant::decode(intent)?;
    let spec: agency_proto::Sealed<agency_proto::ExecutionSpec> =
        serde_json::from_value(original["spec"].clone())?;
    let bundle: agency_proto::Sealed<agency_proto::context::Bundle> =
        serde_json::from_value(original["bundle"].clone())?;
    let client = paired_client(root, &spec.document.binding.id)?;
    let (generation, effect, state) = {
        let mut lock = shared.lock().await;
        let store = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
        let id = format!("prepare:{}", intent.key.id);
        let (effect, state) = store.effect(&id)?;
        if state == store::EffectState::Rejected {
            return Err(stored_rejection(store, &effect)?);
        }
        if state == store::EffectState::Pending {
            current_owner(store, &effect.owner)?;
        }
        (store.generation(), effect, state)
    };
    let result = if state == store::EffectState::Pending {
        fence_and_begin_effect(shared, &client, generation, &effect).await?;
        client
            .call_outcome::<_, agency_proto::Dispatch>(
                "prepare",
                &agency_proto::Prepare {
                    spec: spec.clone(),
                    bundle,
                    writer_generation: u64::try_from(generation.0)
                        .map_err(|_| invalid("writer generation"))?,
                },
            )
            .await
    } else {
        client
            .call_outcome(
                "lookup",
                &agency_proto::Lookup {
                    idempotency_key: spec.document.idempotency_key.clone(),
                },
            )
            .await
    };
    let dispatch = dispatch_reply(result, shared, actor, generation, &effect, state).await?;
    let mut lock = shared.lock().await;
    let store = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
    if store.generation() != generation {
        return Err(reject(
            "WRITER_STALE",
            "control writer changed",
            "reconcile_control_writer",
        ));
    }
    participant::record_dispatch(store, actor, intent, &dispatch)
}

/// The activation outbox exists only after owner→dispatch has been committed.
pub async fn deliver_activation(
    shared: &Arc<Mutex<Option<Store>>>,
    root: &Path,
    actor: &TrustedActor,
    intent_id: &str,
    dispatch: &store::Record,
) -> store::Result<agency_proto::Dispatch> {
    let d: agency_proto::Dispatch = participant::decode(dispatch)?;
    let client = paired_client(root, &d.binding.id)?;
    let (generation, effect, state) = {
        let mut lock = shared.lock().await;
        let store = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
        let effect_id = format!("activate:{intent_id}");
        let (effect, state) = store.effect(&effect_id)?;
        if effect.target != d.reference || effect.binding.key.id != d.binding.id {
            return Err(invalid(
                "activation intent differs from dispatch target or binding",
            ));
        }
        if state == store::EffectState::Rejected {
            return Err(stored_rejection(store, &effect)?);
        }
        if state == store::EffectState::Confirmed {
            let receipt = store
                .get(&participant::key(
                    dispatch.key.scope.clone(),
                    "dispatch_activation",
                    &d.reference,
                ))?
                .ok_or_else(|| invalid("confirmed activation lacks its stored readback"))?;
            return participant::decode(&receipt);
        }
        if state == store::EffectState::Pending {
            current_owner(store, &effect.owner)?;
        }
        (store.generation(), effect, state)
    };
    let result = if state == store::EffectState::Pending {
        fence_and_begin_effect(shared, &client, generation, &effect).await?;
        client
            .call_outcome(
                "activate",
                &agency_proto::DispatchAction {
                    dispatch: d.reference.clone(),
                    writer_generation: u64::try_from(generation.0)
                        .map_err(|_| invalid("writer generation"))?,
                    idempotency_key: effect.idempotency_key.clone(),
                },
            )
            .await
    } else {
        let credentials: Pairing = serde_json::from_slice(&secrets(root).get(&d.binding.id)?)?;
        let ticket = agency_proto::Ticket::sign(
            agency_proto::TicketClaims {
                id: format!("activation-readback:{intent_id}"),
                actor: actor.0.principal.clone(),
                dispatch: d.reference.clone(),
                owner: d.owner.clone(),
                spec_digest: d.spec_digest.clone(),
                writer_generation: u64::try_from(generation.0)
                    .map_err(|_| invalid("writer generation"))?,
                permissions: vec![agency_proto::Permission::Observe],
                input_lease: None,
                expires_ms: now_ms() + 30_000,
            },
            credentials.key.as_bytes(),
        )
        .map_err(err)?;
        let trace: agency_proto::Trace = client
            .call("observe", &agency_proto::Observe { ticket, after: 0 })
            .await
            .map_err(err)?;
        if trace.dispatch.state == agency_proto::DispatchState::Prepared {
            return Err(reject(
                "READBACK_REQUIRED",
                "activation remains unknown; do not resend",
                "reconcile_original_intent",
            ));
        }
        Ok(trace.dispatch)
    };
    let actual: agency_proto::Dispatch =
        dispatch_reply(result, shared, actor, generation, &effect, state).await?;
    if actual.owner != d.owner
        || actual.spec_digest != d.spec_digest
        || actual.bundle_digest != d.bundle_digest
        || actual.binding != d.binding
        || actual.capabilities != d.capabilities
        || actual.reference != d.reference
    {
        return Err(invalid("activation readback differs"));
    }
    let mut lock = shared.lock().await;
    let store = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
    let target = store::ObjectKey {
        scope: dispatch.key.scope.clone(),
        kind: "dispatch_activation".into(),
        id: d.reference.clone(),
    };
    let input = json!({"dispatch":actual});
    let command = store::Command {
        command_id: format!("activation-receipt:{}", d.reference),
        idempotency_key: format!("activation-receipt:{}", d.reference),
        actor: actor.0.clone(),
        target: target.clone(),
        expected: store::Expected::Absent,
        binding: participant::reference(dispatch),
        input_digest: store::Command::digest_input("dispatch.activated", &input)?,
        operation: "dispatch.activated".into(),
        input,
    };
    let record = participant::value(target, 1, &actual)?;
    store.submit(generation, actor, &command, None, |tx| {
        tx.confirm_effect(
            &effect.intent_id,
            &store::Readback::Confirmed {
                binding: effect.binding.clone(),
                target: effect.target.clone(),
                input_digest: effect.input_digest.clone(),
                result: serde_json::to_value(&actual)?,
            },
        )?;
        tx.put(&record)?;
        Ok(json!({"activation":actual}))
    })?;
    Ok(actual)
}
pub async fn preserve_results(
    shared: &Arc<Mutex<Option<Store>>>,
    root: &Path,
    actor: &TrustedActor,
    dispatch: &store::Record,
) -> store::Result<usize> {
    let report = preserve_results_inner(shared, root, actor, dispatch, None).await?;
    match report.refused.into_iter().next() {
        Some((_, error)) => Err(error),
        None => Ok(report.preserved),
    }
}

pub struct PreservationReport {
    pub preserved: usize,
    pub refused: Vec<(String, StoreError)>,
}

/// A bad item does not discard later items from the same result page. Failed
/// items are never acknowledged; the caller can retain the per-item refusal.
pub async fn preserve_results_report(
    shared: &Arc<Mutex<Option<Store>>>,
    root: &Path,
    actor: &TrustedActor,
    dispatch: &store::Record,
) -> store::Result<PreservationReport> {
    preserve_results_inner(shared, root, actor, dispatch, None).await
}

/// Test switch. Stops after proposal number `acknowledgement` is stored and before
/// its acknowledgement is sent. Earlier proposals stay stored and acknowledged.
#[doc(hidden)]
pub async fn preserve_results_failing_before_acknowledgement(
    shared: &Arc<Mutex<Option<Store>>>,
    root: &Path,
    actor: &TrustedActor,
    dispatch: &store::Record,
    acknowledgement: usize,
) -> store::Result<usize> {
    let report =
        preserve_results_inner(shared, root, actor, dispatch, Some(acknowledgement)).await?;
    match report.refused.into_iter().next() {
        Some((_, error)) => Err(error),
        None => Ok(report.preserved),
    }
}

async fn preserve_results_inner(
    shared: &Arc<Mutex<Option<Store>>>,
    root: &Path,
    actor: &TrustedActor,
    dispatch: &store::Record,
    fail_before_acknowledgement: Option<usize>,
) -> store::Result<PreservationReport> {
    let d: agency_proto::Dispatch = participant::decode(dispatch)?;
    let client = paired_client(root, &d.binding.id)?;
    let mut after = None;
    let mut count = 0usize;
    let mut refused = Vec::new();
    loop {
        let mut query = agency_proto::ResultQuery::of(d.reference.clone());
        query.after = after.clone();
        let page: agency_proto::ResultPage = client.call("results", &query).await.map_err(err)?;
        if page.proposals.is_empty() {
            break;
        }
        for proposal in &page.proposals {
            let stored = async {
                let mut lock = shared.lock().await;
                let store = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
                participant::preserve_proposal(store, actor, dispatch, proposal)?;
                let record = store
                    .get(&participant::key(
                        dispatch.key.scope.clone(),
                        "proposal_inbox",
                        &proposal.header.proposal_id,
                    ))?
                    .ok_or_else(|| invalid("preservation not committed"))?;
                if store.read_material(actor, &record.materials[0])? != proposal.output {
                    return Err(invalid("exact preserved bytes cannot be read back"));
                }
                Ok::<_, StoreError>(())
            }
            .await;
            if let Err(error) = stored {
                refused.push((proposal.header.proposal_id.clone(), error));
                continue;
            }
            if fail_before_acknowledgement == Some(count + 1) {
                return Err(invalid("preservation acknowledgement stopped"));
            }
            let acknowledgement: store::Result<Value> = client
                .call(
                    "preserve",
                    &agency_proto::Preservation {
                        dispatch: d.reference.clone(),
                        proposal_id: proposal.header.proposal_id.clone(),
                        content_digest: proposal.content_digest.clone(),
                    },
                )
                .await
                .map_err(err);
            match acknowledgement {
                Ok(_) => count += 1,
                Err(error) => refused.push((proposal.header.proposal_id.clone(), error)),
            }
        }
        if page.complete {
            break;
        }
        after = page.cursor;
        if after.is_none() {
            break;
        }
    }
    Ok(PreservationReport {
        preserved: count,
        refused,
    })
}
fn now_ms() -> u64 {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    )
    .unwrap_or(u64::MAX)
}
fn current_owner(store: &Store, owner: &store::Reference) -> store::Result<()> {
    let record = store
        .get(&owner.key)?
        .ok_or_else(|| invalid("authorized owner missing"))?;
    if participant::reference(&record) != *owner
        || (owner.key.kind == "room_invocation"
            && !project::invocation::current_authorization(store, owner, now_ms())?)
    {
        return Err(reject(
            "OWNER_STALE",
            "original owner authorization changed",
            "rebuild_preview",
        ));
    }
    Ok(())
}

/// Sign an observe-only ticket and ask the Agency what it reports. The internal
/// signer grants observation and nothing else; it never mints an input lease.
async fn contact_dispatch(
    shared: &Arc<Mutex<Option<Store>>>,
    root: &Path,
    actor: &TrustedActor,
    dispatch: &store::Record,
    after: u64,
) -> store::Result<(
    store::WriterGeneration,
    Result<agency_proto::Trace, PortError>,
)> {
    let d: agency_proto::Dispatch = participant::decode(dispatch)?;
    if !actor.0.permission_scope.contains(&dispatch.key.scope) {
        return Err(reject(
            "PERMISSION_DENIED",
            "dispatch scope not authorized",
            "request_authorization",
        ));
    }
    let generation = {
        let state = shared.lock().await;
        state
            .as_ref()
            .ok_or_else(|| invalid("store not ready"))?
            .generation()
    };
    let credentials: Pairing = serde_json::from_slice(&secrets(root).get(&d.binding.id)?)?;
    let ticket = agency_proto::Ticket::sign(
        agency_proto::TicketClaims {
            id: format!("observe:{}:{after}", d.reference),
            actor: actor.0.principal.clone(),
            dispatch: d.reference.clone(),
            owner: d.owner.clone(),
            spec_digest: d.spec_digest.clone(),
            writer_generation: u64::try_from(generation.0)
                .map_err(|_| invalid("writer generation"))?,
            permissions: vec![agency_proto::Permission::Observe],
            input_lease: None,
            expires_ms: now_ms() + 30_000,
        },
        credentials.key.as_bytes(),
    )
    .map_err(err)?;
    let result = Client::new(credentials.endpoint.into(), credentials.key)
        .call("observe", &agency_proto::Observe { ticket, after })
        .await;
    Ok((generation, result))
}

/// Read what the Agency reports now. This writes nothing: that control looked is
/// an access-log policy point, not a fact about the dispatch, so a human read
/// leaves the governance records to the reconcile loop.
pub async fn inspect_dispatch(
    shared: &Arc<Mutex<Option<Store>>>,
    root: &Path,
    actor: &TrustedActor,
    dispatch: &store::Record,
    after: u64,
) -> store::Result<agency_proto::Trace> {
    let (_, result) = contact_dispatch(shared, root, actor, dispatch, after).await?;
    result.map_err(err)
}

/// The reconcile loop's observation, and the only writer of `dispatch_observation`
/// and `dispatch_contact`. Recording grants or revokes no domain authority.
pub async fn observe_dispatch(
    shared: &Arc<Mutex<Option<Store>>>,
    root: &Path,
    actor: &TrustedActor,
    dispatch: &store::Record,
    after: u64,
) -> store::Result<agency_proto::Trace> {
    let (generation, result) = contact_dispatch(shared, root, actor, dispatch, after).await?;
    let mut state = shared.lock().await;
    let s = state.as_mut().ok_or_else(|| invalid("store not ready"))?;
    if s.generation() != generation {
        return Err(reject(
            "WRITER_STALE",
            "control writer changed",
            "reconcile_control_writer",
        ));
    }
    match result {
        Ok(trace) => {
            participant::record_observation(s, actor, dispatch, &trace)?;
            Ok(trace)
        }
        Err(e) => {
            if e.code == "AGENCY_UNREACHABLE" {
                participant::record_unreachable(s, actor, dispatch, now_ms())?;
            }
            Err(err(e))
        }
    }
}

/// Reconciliation probes and fences this tenant; only explicit consumption starts Agency.
pub async fn reconcile(shared: &Arc<Mutex<Option<Store>>>, root: &Path) -> store::Result<()> {
    let (bindings, generation) = {
        let lock = shared.lock().await;
        let Some(store) = lock.as_ref() else {
            return Ok(());
        };
        (store.list("agency_binding")?, store.generation())
    };
    let mut first_error = None;
    for binding in bindings {
        let result = async {
            let _: Value = paired_client(root, &binding.key.id)?
                .call(
                    "fence",
                    &agency_proto::Fence {
                        writer_generation: u64::try_from(generation.0)
                            .map_err(|_| invalid("writer generation"))?,
                    },
                )
                .await
                .map_err(err)?;
            Ok::<_, StoreError>(())
        }
        .await;
        if let Err(error) = result {
            first_error.get_or_insert(error);
        }
    }
    first_error.map_or(Ok(()), Err)
}

#[cfg(test)]
mod authorization_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn matching_record_version_does_not_authorize_a_malformed_room_invocation() {
        let root = std::env::temp_dir().join(format!(
            "hctl-agency-owner-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut store = Store::open(&root).unwrap();
        let actor = TrustedActor(store::Actor {
            principal: "owner".into(),
            source: store::ActorSource::DirectClient,
            permission_scope: vec![store::Scope::Project("project".into())],
            authority: None,
        });
        let record = participant::value(
            participant::key(
                store::Scope::Project("project".into()),
                "room_invocation",
                "invocation",
            ),
            1,
            &json!({"state":"authorized"}),
        )
        .unwrap();
        let owner = participant::reference(&record);
        let input = json!({});
        let command = store::Command {
            command_id: "fixture".into(),
            idempotency_key: "fixture".into(),
            actor: actor.0.clone(),
            target: record.key.clone(),
            expected: store::Expected::Absent,
            binding: owner.clone(),
            operation: "fixture".into(),
            input_digest: store::Command::digest_input("fixture", &input).unwrap(),
            input,
        };
        store
            .submit(store.generation(), &actor, &command, None, |tx| {
                tx.put(&record)?;
                Ok(json!({}))
            })
            .unwrap();
        assert_eq!(
            participant::reference(&store.get(&owner.key).unwrap().unwrap()),
            owner
        );
        let denied = current_owner(&store, &owner).is_err();
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
        assert!(
            denied,
            "a matching storage version is not a Project authorization proof"
        );
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    #[test]
    fn a_stuck_agency_start_times_out_and_leaves_no_process() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("hctl2-agency-stuck-{nanos}"));
        fs::create_dir(&dir).unwrap();
        let program = dir.join("hang.sh");
        let marker = run_marker(4167);
        fs::write(&program, format!("#!/bin/sh\nexec sleep {marker}\n")).unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
        let command = Command::new(&program);
        let started = Instant::now();
        let error = super::run_agency_command(command, Duration::from_secs(3))
            .expect_err("stuck agency start");
        assert!(started.elapsed() < Duration::from_secs(7));
        assert_eq!(error.code, "AGENCY_START_TIMEOUT");
        assert_no_process_with(&marker);
        fs::remove_dir_all(dir).unwrap();
    }

    /// A marker unique to this run and to the call site: the leftover check
    /// looks for it, and the same test file compiles into several test
    /// binaries, so a constant would let one binary see the other's process.
    fn run_marker(base: u64) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock follows the epoch")
            .as_nanos()
            % 1_000;
        format!("31.{base}{:04}{nanos:03}", std::process::id() % 10_000)
    }

    /// Fails while a live process still carries `marker` on its command line; the
    /// kill at the deadline clears the process table a moment later, so the check
    /// retries and prints what it still sees.
    fn assert_no_process_with(marker: &str) {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let listed = Command::new("pgrep")
                .args(["-f", marker])
                .output()
                .expect("pgrep");
            if listed.stdout.iter().all(u8::is_ascii_whitespace) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "a process matching {marker} is still present: {}",
                String::from_utf8_lossy(&listed.stdout).trim()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

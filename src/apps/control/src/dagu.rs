//! Hosted Dagu lifecycle consumption, loopback endpoint enforcement, `SecretStore` API key
//! bootstrap, and `workflow.engine.*` command/query handlers.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dagu_api::{
    CompleteCheckpointParams, DaguAdapter, PollReconcileParams, StartRunParams, bind_and_start_run,
    bootstrap_builtin_auth, cancel_run, complete_checkpoint, get_binding, get_controlled_run,
    poll_and_reconcile, reject_engine_self_advance, verify_loopback_url,
};
use project::{Result, invalid, reject};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use store::{ObjectKey, RecordData, Scope, Store, TrustedActor};
use tokio::sync::Mutex;

use crate::services::Supervisor;

type Shared = Arc<Mutex<Option<Store>>>;

const DAGU_BOOTSTRAP_TIMEOUT: Duration = Duration::from_secs(30);
const DAGU_BOOTSTRAP_POLL: Duration = Duration::from_millis(100);
const AUTH_MARKER_FILE: &str = "dagu-auth-provisioned";
static BOOTSTRAP_NONCE: AtomicU64 = AtomicU64::new(1);

fn not_ready() -> project::StoreError {
    reject("STORE_NOT_READY", "store not ready", "check_status")
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineCommandEnvelope {
    pub key: String,
    pub action: Value,
}

fn parse_envelope(payload: &Value) -> Result<EngineCommandEnvelope> {
    serde_json::from_value(payload.clone())
        .map_err(|e| invalid(format!("invalid engine command payload: {e}")))
}

fn run_async<T: Send, F: std::future::Future<Output = T> + Send>(future: F) -> T {
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("build current-thread runtime for dagu_api")
                    .block_on(future)
            })
            .join()
            .expect("join dagu_api worker thread")
    })
}

/// Resolve the Dagu base URL for this control root and verify it is strictly loopback.
///
/// # Errors
///
/// Returns `WORKFLOW_ENGINE_NON_LOOPBACK` if the configured URL is not loopback.
pub fn resolve_dagu_base_url(root: &Path, services: &Supervisor) -> Result<String> {
    if let Ok(url) = std::env::var("HCTL2_DAGU_URL")
        && !url.trim().is_empty()
    {
        return verify_loopback_url(url.trim());
    }
    let state_root = services
        .packaged_paths()
        .map(|(_, state)| state)
        .unwrap_or_else(|| {
            if root != crate::default_root() {
                root.join("services")
            } else {
                std::env::var_os("HCTL2_STATE_ROOT")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| root.join("services"))
            }
        });
    let ports_file = state_root.join("config/dagu/ports.env");
    if let Ok(contents) = std::fs::read_to_string(&ports_file) {
        for line in contents.lines() {
            if let Some(port_str) = line.trim().strip_prefix("HCTL2_DAGU_PORT=") {
                let port: u16 = port_str.trim().parse().map_err(|_| {
                    reject(
                        "WORKFLOW_ENGINE_CONFIG",
                        format!("invalid HCTL2_DAGU_PORT in {}", ports_file.display()),
                        "inspect_services_config",
                    )
                })?;
                return verify_loopback_url(&format!("http://127.0.0.1:{port}"));
            }
        }
    }
    verify_loopback_url("http://127.0.0.1:8180")
}

/// Consume hosted Dagu, wait for readiness when running under packaged Process Compose,
/// and return a `DaguAdapter` authenticated with the `dagu_` API key from `SecretStore`.
///
/// If the API key was previously provisioned (`dagu-auth-provisioned` marker exists) and is
/// missing from `SecretStore`, this fails closed with `WORKFLOW_ENGINE_SECRET_MISSING` rather
/// than degrading to anonymous access.
///
/// # Errors
///
/// Returns a typed error if Dagu is not on loopback, fails to start, or the API key is missing.
pub async fn connect_adapter(
    root: &Path,
    control_id: &str,
    services: &Supervisor,
) -> Result<DaguAdapter> {
    let base_url = resolve_dagu_base_url(root, services)?;
    let has_hosted_dagu = services
        .snapshot()
        .hosted
        .iter()
        .any(|service| service.name == "dagu");
    if has_hosted_dagu {
        services
            .consume("dagu")
            .map_err(|e| reject(e.code, e.message, e.recovery_action))?;
    }
    if services.packaged_paths().is_some() && std::env::var_os("HCTL2_DAGU_URL").is_none() {
        let deadline = Instant::now() + DAGU_BOOTSTRAP_TIMEOUT;
        loop {
            if services
                .snapshot()
                .hosted
                .iter()
                .any(|s| s.name == "dagu" && s.available())
            {
                break;
            }
            if Instant::now() >= deadline {
                return Err(reject(
                    "WORKFLOW_ENGINE_NOT_READY",
                    "hosted Dagu consumed but readiness not confirmed",
                    "check_services",
                ));
            }
            tokio::time::sleep(DAGU_BOOTSTRAP_POLL).await;
        }
    }

    let secrets = crate::config::secret_store(root).map_err(|message| {
        reject(
            "WORKFLOW_ENGINE_SECRET_MISSING",
            message,
            "restore_secret_store",
        )
    })?;
    let credential_ref = format!("dagu:{control_id}:api_key");
    let marker_path = root.join(AUTH_MARKER_FILE);
    let api_key = match secrets.get(&credential_ref) {
        Ok(bytes) => {
            let key = String::from_utf8(bytes).map_err(|_| {
                reject(
                    "WORKFLOW_ENGINE_SECRET_MISSING",
                    "stored Dagu API key is not valid UTF-8",
                    "restore_secret_store",
                )
            })?;
            if !key.starts_with("dagu_") {
                return Err(reject(
                    "WORKFLOW_ENGINE_SECRET_MISSING",
                    "stored Dagu API key does not have required dagu_ prefix",
                    "restore_secret_store",
                ));
            }
            key
        }
        Err(_) => {
            if marker_path.exists() {
                return Err(reject(
                    "WORKFLOW_ENGINE_SECRET_MISSING",
                    format!(
                        "Dagu API key '{credential_ref}' is missing from SecretStore; refusing to degrade to unauthenticated access"
                    ),
                    "restore_secret_store",
                ));
            }
            let username = format!("hctl-{}", &control_id[..16.min(control_id.len())]);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let nonce = BOOTSTRAP_NONCE.fetch_add(1, Ordering::Relaxed);
            let password = foundation::bytes_sha256(
                format!("dagu-admin:{control_id}:{nanos}:{nonce}").as_bytes(),
            );
            let minted =
                bootstrap_builtin_auth(&base_url, &username, &password, "hctl-control").await?;
            secrets
                .set(&credential_ref, minted.as_bytes())
                .map_err(|e| {
                    reject(
                        "WORKFLOW_ENGINE_SECRET_MISSING",
                        format!("cannot persist Dagu API key in SecretStore: {e}"),
                        "restore_secret_store",
                    )
                })?;
            std::fs::write(&marker_path, credential_ref.as_bytes()).map_err(|e| {
                reject(
                    "STORAGE_IO",
                    format!("cannot write Dagu auth marker: {e}"),
                    "inspect_storage",
                )
            })?;
            minted
        }
    };

    DaguAdapter::new(&base_url, &api_key)
}

pub(crate) fn query(
    shared: &Shared,
    _actor: &TrustedActor,
    kind: &str,
    payload: &Value,
) -> Result<Value> {
    if kind != "workflow.engine.show" {
        return Err(invalid(format!("unknown workflow.engine query {kind}")));
    }
    let project_id = payload["project_id"]
        .as_str()
        .ok_or_else(|| invalid("project_id required"))?;
    let binding_id = payload["binding_id"]
        .as_str()
        .ok_or_else(|| invalid("binding_id required"))?;
    let slot = shared.blocking_lock();
    let store = slot.as_ref().ok_or_else(not_ready)?;
    let binding = get_binding(store, project_id, binding_id)?;
    let run = get_controlled_run(store, project_id, &binding.run_id)?;
    let mut observations = Vec::new();
    for seq in 1..=binding.last_observation_seq {
        let obs_key = ObjectKey {
            scope: Scope::Project(project_id.to_owned()),
            kind: "checkpoint_observation".to_owned(),
            id: format!("{binding_id}:{seq}"),
        };
        if let Some(record) = store.get(&obs_key)?
            && let RecordData::Value { value } = record.data
        {
            observations.push(value);
        }
    }
    Ok(json!({
        "run": run,
        "binding": binding,
        "observations": observations,
    }))
}

pub(crate) fn preview(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    _actor: &TrustedActor,
    operation: &str,
    payload: &Value,
) -> Result<Value> {
    let env = parse_envelope(payload)?;
    let _ = resolve_dagu_base_url(root, services)?;
    let slot = shared.blocking_lock();
    let _store = slot.as_ref().ok_or_else(not_ready)?;
    match operation {
        "workflow.engine.start"
        | "workflow.engine.poll"
        | "workflow.engine.complete"
        | "workflow.engine.cancel"
        | "workflow.engine.self_advance"
        | "workflow.engine.direct_op" => Ok(json!({
            "operation": operation,
            "key": env.key,
            "action": env.action,
        })),
        _ => Err(invalid(format!(
            "unknown workflow.engine operation {operation}"
        ))),
    }
}

pub(crate) fn submit(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    actor: &TrustedActor,
    request: &proto::SubmitRequest,
    details: &Value,
) -> Result<Value> {
    let payload_val: Value = serde_json::from_slice(&request.payload)?;
    let env = parse_envelope(&payload_val)?;
    if request.command_id != format!("workflow:{}", env.key) || request.idempotency_key != env.key {
        return Err(invalid("Workflow engine envelope key mismatch"));
    }
    if details.get("key").and_then(Value::as_str) != Some(env.key.as_str()) {
        return Err(invalid("Workflow engine preview input differs"));
    }

    let mut slot = shared.blocking_lock();
    let store = slot.as_mut().ok_or_else(not_ready)?;
    let control_id = store.control_id().to_owned();
    let op = request.operation.as_str();
    let mut action = env.action;
    if let Some(obj) = action.as_object_mut() {
        obj.entry("command_key".to_owned())
            .or_insert_with(|| Value::String(env.key.clone()));
    }

    if op == "workflow.engine.self_advance" {
        let project_id = action["project_id"]
            .as_str()
            .ok_or_else(|| invalid("project_id required"))?;
        let run_id = action["run_id"]
            .as_str()
            .ok_or_else(|| invalid("run_id required"))?;
        let step_id = action["step_id"]
            .as_str()
            .ok_or_else(|| invalid("step_id required"))?;
        reject_engine_self_advance(store, project_id, run_id, step_id)?;
        return Ok(json!({}));
    }

    run_async(async {
        let adapter = connect_adapter(root, &control_id, services).await?;
        match op {
            "workflow.engine.start" => {
                let params: StartRunParams = serde_json::from_value(action)
                    .map_err(|e| invalid(format!("invalid start params: {e}")))?;
                let (run, binding, readback) =
                    bind_and_start_run(store, actor, &adapter, &params).await?;
                Ok(json!({
                    "run": run,
                    "binding": binding,
                    "readback": readback,
                }))
            }
            "workflow.engine.poll" => {
                let params: PollReconcileParams = serde_json::from_value(action)
                    .map_err(|e| invalid(format!("invalid poll params: {e}")))?;
                let outcome = poll_and_reconcile(store, actor, &adapter, &params).await?;
                Ok(serde_json::to_value(outcome)?)
            }
            "workflow.engine.complete" => {
                let params: CompleteCheckpointParams = serde_json::from_value(action)
                    .map_err(|e| invalid(format!("invalid complete params: {e}")))?;
                let (run, binding, resolution) =
                    complete_checkpoint(store, actor, &adapter, &params).await?;
                Ok(json!({
                    "run": run,
                    "binding": binding,
                    "resolution": resolution,
                }))
            }
            "workflow.engine.cancel" => {
                let project_id = action["project_id"]
                    .as_str()
                    .ok_or_else(|| invalid("project_id required"))?;
                let binding_id = action["binding_id"]
                    .as_str()
                    .ok_or_else(|| invalid("binding_id required"))?;
                let now_ms = action
                    .get("now_ms")
                    .and_then(Value::as_u64)
                    .unwrap_or(2_000);
                let (run, binding, stop_outcome) = cancel_run(
                    store, actor, &adapter, &env.key, project_id, binding_id, now_ms,
                )
                .await?;
                Ok(json!({
                    "run": run,
                    "binding": binding,
                    "stop_outcome": stop_outcome,
                }))
            }
            "workflow.engine.direct_op" => {
                let project_id = action["project_id"]
                    .as_str()
                    .ok_or_else(|| invalid("project_id required"))?;
                let binding_id = action["binding_id"]
                    .as_str()
                    .ok_or_else(|| invalid("binding_id required"))?;
                let operation = action["operation"]
                    .as_str()
                    .ok_or_else(|| invalid("operation required"))?;
                let step_name = action.get("step_name").and_then(Value::as_str);
                let target = action.get("target").and_then(Value::as_str);
                let binding = get_binding(store, project_id, binding_id)?;
                adapter
                    .perform_direct_dagu_operation(
                        operation,
                        &binding.dag_name,
                        &binding.dag_run_id,
                        step_name,
                        target,
                    )
                    .await?;
                Ok(json!({
                    "status": "direct_operation_applied",
                    "operation": operation,
                    "dag_name": binding.dag_name,
                    "dag_run_id": binding.dag_run_id,
                }))
            }
            _ => Err(invalid(format!("unknown workflow.engine operation {op}"))),
        }
    })
}

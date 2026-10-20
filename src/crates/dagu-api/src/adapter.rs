//! Dagu v1 OpenAPI adapter over the `progenitor`-generated client.

use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use foundation::bytes_sha256;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use store::{Result, StoreError};

use crate::generated::types::{
    ApproveStepRequest, CompleteHumanTaskStepId, CreateApiKeyRequest,
    CreateApiKeyRequestAllowedSurfacesItem, CreateApiKeyRequestAttributionClass,
    CreateApiKeyRequestDescription, CreateApiKeyRequestName, CreateNewDagBody, DagFileName,
    DagName, DagRunId, EnqueueDagRunFromSpecBody, ExecuteDagBody, ExecuteDagRunFromSpecBody,
    HumanTaskInput, LoginRequest, LoginRequestPassword, LoginRequestUsername, RejectStepRequest,
    RenameDagBody, RescheduleDagRunBody, RetryDagRunBody, SetupRequest, SetupRequestPassword,
    SetupRequestUsername, UpdateDagSpecBody, UserRole, ValidateDagSpecBody,
};
use crate::generated::{Client, Error as ProgenitorError};
use crate::model::{
    CompleteResolution, CompletionRecoveryAction, EngineRunReadback, EngineStepReadback,
    HumanTaskCompletionRequest, StopOutcome,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(20);

pub(crate) fn reject(
    code: &'static str,
    message: impl Into<String>,
    recovery_action: &'static str,
) -> StoreError {
    StoreError {
        code,
        message: message.into(),
        recovery_action,
    }
}

/// Verify that the Dagu endpoint is strictly loopback (`127.0.0.1`, `localhost`, or `[::1]`)
/// and normalize its `/api/v1` base URL (`验收 5`).
///
/// # Errors
///
/// Returns `WORKFLOW_ENGINE_NON_LOOPBACK` if the URL scheme is not `http://` or the host is not loopback.
pub fn verify_loopback_url(raw_url: &str) -> Result<String> {
    let trimmed = raw_url.trim().trim_end_matches('/');
    let Some(rest) = trimmed.strip_prefix("http://") else {
        return Err(reject(
            "WORKFLOW_ENGINE_NON_LOOPBACK",
            format!("Dagu endpoint must use loopback http://, got {raw_url}"),
            "configure_loopback_dagu",
        ));
    };
    let host_port = rest.split('/').next().unwrap_or("");
    let host = if let Some(bracketed) = host_port.strip_prefix('[') {
        bracketed
            .split(']')
            .next()
            .map_or("", |inner| if inner == "::1" { "[::1]" } else { "" })
    } else {
        host_port.split(':').next().unwrap_or("")
    };
    if !matches!(host, "127.0.0.1" | "localhost" | "[::1]") {
        return Err(reject(
            "WORKFLOW_ENGINE_NON_LOOPBACK",
            format!("Dagu endpoint host must be loopback (127.0.0.1/localhost/::1), got {raw_url}"),
            "configure_loopback_dagu",
        ));
    }
    if trimmed.ends_with("/api/v1") {
        Ok(trimmed.to_owned())
    } else {
        Ok(format!("{trimmed}/api/v1"))
    }
}

fn build_http_client(bearer_token: Option<&str>, timeout: Duration) -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    if let Some(token) = bearer_token {
        let mut header =
            HeaderValue::from_str(&format!("Bearer {}", token.trim())).map_err(|_| {
                reject(
                    "WORKFLOW_ENGINE_AUTH",
                    "invalid Dagu bearer token characters",
                    "restore_secret_store",
                )
            })?;
        header.set_sensitive(true);
        headers.insert(AUTHORIZATION, header);
    }
    reqwest::Client::builder()
        .default_headers(headers)
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .timeout(timeout)
        .build()
        .map_err(|error| {
            reject(
                "WORKFLOW_ENGINE_UNAVAILABLE",
                format!("cannot build Dagu HTTP client: {error}"),
                "check_services",
            )
        })
}

/// Bootstrap Dagu Builtin Auth and mint a `dagu_...` service-account API key for control.
///
/// # Errors
///
/// Returns a typed `StoreError` if the endpoint is non-loopback, unreachable, or rejects setup/login/key creation.
pub async fn bootstrap_builtin_auth(
    base_url: &str,
    username: &str,
    password: &str,
    key_name: &str,
) -> Result<String> {
    let normalized = verify_loopback_url(base_url)?;
    let unauth_http = build_http_client(None, DEFAULT_TIMEOUT)?;
    let unauth = Client::new_with_client(&normalized, unauth_http);

    let setup_body = SetupRequest {
        username: SetupRequestUsername::try_from(username)
            .map_err(|e| reject("INVALID_INPUT", e.to_string(), "correct_input"))?,
        password: SetupRequestPassword::try_from(password)
            .map_err(|e| reject("INVALID_INPUT", e.to_string(), "correct_input"))?,
    };
    let jwt = match unauth.setup(None, &setup_body).await {
        Ok(resp) => resp.into_inner().token,
        Err(_) => {
            let login_body = LoginRequest {
                username: LoginRequestUsername::try_from(username)
                    .map_err(|e| reject("INVALID_INPUT", e.to_string(), "correct_input"))?,
                password: LoginRequestPassword::try_from(password)
                    .map_err(|e| reject("INVALID_INPUT", e.to_string(), "correct_input"))?,
            };
            unauth
                .login(&login_body)
                .await
                .map_err(|error| {
                    reject(
                        "WORKFLOW_ENGINE_AUTH",
                        format!(
                            "Dagu builtin auth setup/login failed: {}",
                            redact_error(&error)
                        ),
                        "inspect_dagu_auth",
                    )
                })?
                .into_inner()
                .token
        }
    };

    let admin_http = build_http_client(Some(&jwt), DEFAULT_TIMEOUT)?;
    let admin = Client::new_with_client(&normalized, admin_http);
    let key_req = CreateApiKeyRequest {
        allowed_surfaces: vec![CreateApiKeyRequestAllowedSurfacesItem::RestApi],
        attribution_class: CreateApiKeyRequestAttributionClass::ServiceAccount,
        description: Some(
            CreateApiKeyRequestDescription::try_from("HCTL2 control workflow engine adapter")
                .map_err(|e| reject("INVALID_INPUT", e.to_string(), "correct_input"))?,
        ),
        name: CreateApiKeyRequestName::try_from(key_name)
            .map_err(|e| reject("INVALID_INPUT", e.to_string(), "correct_input"))?,
        owner_user_id: None,
        role: UserRole::Admin,
        service_account_name: Some("hctl2-control".to_owned()),
        workspace_access: None,
    };
    let created = admin
        .create_api_key(&key_req)
        .await
        .map_err(|error| {
            reject(
                "WORKFLOW_ENGINE_AUTH",
                format!("Dagu API key creation failed: {}", redact_error(&error)),
                "inspect_dagu_auth",
            )
        })?
        .into_inner();
    if !created.key.starts_with("dagu_") {
        return Err(reject(
            "WORKFLOW_ENGINE_AUTH",
            "minted Dagu API key did not have expected dagu_ prefix",
            "inspect_dagu_auth",
        ));
    }
    Ok(created.key)
}

fn redact_error<T: fmt::Debug>(error: &ProgenitorError<T>) -> String {
    match error {
        ProgenitorError::ErrorResponse(resp) => {
            format!("HTTP {} {:?}", resp.status(), resp.as_ref())
        }
        ProgenitorError::InvalidRequest(msg) => format!("invalid request: {msg}"),
        ProgenitorError::CommunicationError(err) => {
            format!("communication error (status={:?})", err.status())
        }
        ProgenitorError::InvalidUpgrade(err) => format!("invalid upgrade: {err}"),
        ProgenitorError::ResponseBodyError(err) => format!("response body error: {err}"),
        ProgenitorError::InvalidResponsePayload(_, err) => {
            format!("invalid response payload: {err}")
        }
        ProgenitorError::UnexpectedResponse(resp) => {
            format!("unexpected HTTP status {}", resp.status())
        }
        ProgenitorError::Custom(msg) => format!("client error: {msg}"),
    }
}

/// Controlled workflow engine adapter backed by the `progenitor`-generated Dagu v1 client.
#[derive(Clone)]
pub struct DaguAdapter {
    base_url: String,
    client: Client,
}

impl fmt::Debug for DaguAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DaguAdapter")
            .field("base_url", &self.base_url)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

impl DaguAdapter {
    /// Construct a loopback-only Dagu adapter authenticated with a `dagu_...` bearer API key.
    ///
    /// # Errors
    ///
    /// Returns an error when `base_url` is not loopback or `api_key` is empty/malformed.
    pub fn new(base_url: &str, api_key: &str) -> Result<Self> {
        let normalized = verify_loopback_url(base_url)?;
        if api_key.trim().is_empty() {
            return Err(reject(
                "WORKFLOW_ENGINE_AUTH",
                "Dagu API key is required",
                "restore_secret_store",
            ));
        }
        let http = build_http_client(Some(api_key), DEFAULT_TIMEOUT)?;
        Ok(Self {
            base_url: normalized.clone(),
            client: Client::new_with_client(&normalized, http),
        })
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    #[must_use]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Check whether a renamed variant of `dag_name` exists in Dagu's DAG list.
    pub async fn has_renamed_dag(&self, dag_name: &str) -> bool {
        let renamed_prefix = format!("{}_", dag_name.to_ascii_lowercase());
        if let Ok(resp) = self
            .client
            .list_da_gs(None, None, None, None, None, None, None, None, None, None)
            .await
        {
            return resp.into_inner().dags.iter().any(|dag| {
                dag.file_name
                    .to_ascii_lowercase()
                    .starts_with(&renamed_prefix)
            });
        }
        false
    }

    /// Check `/api/v1/health` via the generated client.
    ///
    /// # Errors
    ///
    /// Returns `WORKFLOW_ENGINE_UNAVAILABLE` if Dagu is unreachable or unhealthy.
    pub async fn health_check(&self) -> Result<String> {
        let health = self
            .client
            .get_health_status()
            .await
            .map_err(|error| {
                reject(
                    "WORKFLOW_ENGINE_UNAVAILABLE",
                    format!("Dagu health check failed: {}", redact_error(&error)),
                    "check_services",
                )
            })?
            .into_inner();
        Ok(health.status.to_string())
    }

    /// Validate and register/update the compiled DAG spec in Dagu so queue-driven `human.task`
    /// resume (`dagu retry --run-id ...`) can load `<dag_name>.yaml`.
    ///
    /// # Errors
    ///
    /// Returns a typed error if spec validation or registration fails.
    pub async fn ensure_dag_spec(&self, dag_name: &str, yaml_spec: &str) -> Result<()> {
        let validated = self
            .client
            .validate_dag_spec(
                None,
                &ValidateDagSpecBody {
                    name: Some(dag_name.to_owned()),
                    spec: yaml_spec.to_owned(),
                },
            )
            .await
            .map_err(|error| {
                reject(
                    "WORKFLOW_ENGINE_VALIDATE",
                    format!("Dagu spec validation failed: {}", redact_error(&error)),
                    "edit_workflow",
                )
            })?
            .into_inner();
        if !validated.valid {
            return Err(reject(
                "WORKFLOW_ENGINE_VALIDATE",
                format!("Dagu rejected spec: {:?}", validated.errors),
                "edit_workflow",
            ));
        }

        let create_body = CreateNewDagBody {
            name: DagName(dag_name.to_owned()),
            spec: Some(yaml_spec.to_owned()),
        };
        if let Err(error) = self.client.create_new_dag(None, &create_body).await {
            let is_conflict_or_exists = match &error {
                ProgenitorError::ErrorResponse(resp) => {
                    matches!(resp.status().as_u16(), 400 | 409)
                }
                ProgenitorError::UnexpectedResponse(resp) => {
                    matches!(resp.status().as_u16(), 400 | 409)
                }
                _ => false,
            };
            if !is_conflict_or_exists {
                return Err(reject(
                    "WORKFLOW_ENGINE_UNAVAILABLE",
                    format!("cannot register DAG {dag_name}: {}", redact_error(&error)),
                    "check_services",
                ));
            }
        }
        let update_body = UpdateDagSpecBody {
            spec: yaml_spec.to_owned(),
        };
        let updated = self
            .client
            .update_dag_spec(&DagFileName(dag_name.to_owned()), None, &update_body)
            .await
            .map_err(|error| {
                reject(
                    "WORKFLOW_ENGINE_UNAVAILABLE",
                    format!(
                        "cannot update DAG spec {dag_name}: {}",
                        redact_error(&error)
                    ),
                    "check_services",
                )
            })?
            .into_inner();
        if !updated.errors.is_empty() {
            return Err(reject(
                "WORKFLOW_ENGINE_VALIDATE",
                format!("Dagu spec update errors: {:?}", updated.errors),
                "edit_workflow",
            ));
        }
        Ok(())
    }

    /// Enqueue a DAG run from inline YAML spec and read back its initial state.
    ///
    /// # Errors
    ///
    /// Returns a typed error if spec registration, enqueue, or readback fails.
    pub async fn enqueue(
        &self,
        dag_name: &str,
        yaml_spec: &str,
        dag_run_id: &str,
    ) -> Result<EngineRunReadback> {
        self.ensure_dag_spec(dag_name, yaml_spec).await?;
        let body = EnqueueDagRunFromSpecBody {
            dag_run_id: Some(DagRunId(dag_run_id.to_owned())),
            labels: None,
            name: Some(dag_name.to_owned()),
            no_reuse: false,
            params: None,
            profile: None,
            queue: None,
            singleton: false,
            spec: yaml_spec.to_owned(),
            tags: None,
        };
        self.client
            .enqueue_dag_run_from_spec(None, &body)
            .await
            .map_err(|error| {
                reject(
                    "WORKFLOW_ENGINE_START",
                    format!(
                        "cannot enqueue DAG run {dag_run_id}: {}",
                        redact_error(&error)
                    ),
                    "check_services",
                )
            })?;
        self.wait_for_run_visible(dag_name, dag_run_id).await
    }

    /// Start a DAG run and read back its state.
    ///
    /// # Errors
    ///
    /// Returns a typed error if spec registration, start, or readback fails.
    pub async fn start(
        &self,
        dag_name: &str,
        yaml_spec: &str,
        dag_run_id: &str,
    ) -> Result<EngineRunReadback> {
        self.ensure_dag_spec(dag_name, yaml_spec).await?;
        let body = ExecuteDagBody {
            dag_name: None,
            dag_run_id: Some(DagRunId(dag_run_id.to_owned())),
            ..ExecuteDagBody::default()
        };
        self.client
            .execute_dag(&DagFileName(dag_name.to_owned()), None, &body)
            .await
            .map_err(|error| {
                reject(
                    "WORKFLOW_ENGINE_START",
                    format!(
                        "cannot start DAG run {dag_run_id}: {}",
                        redact_error(&error)
                    ),
                    "check_services",
                )
            })?;
        self.wait_for_run_visible(dag_name, dag_run_id).await
    }

    async fn wait_for_run_visible(
        &self,
        dag_name: &str,
        dag_run_id: &str,
    ) -> Result<EngineRunReadback> {
        let mut last_error = None;
        for _ in 0..50 {
            match self.readback_run(dag_name, dag_run_id).await {
                Ok(readback) => {
                    if !matches!(readback.status, 0 | 1 | 5) {
                        return Ok(readback);
                    }
                    tokio::time::sleep(Duration::from_millis(80)).await;
                }
                Err(err) => {
                    last_error = Some(err);
                    tokio::time::sleep(Duration::from_millis(80)).await;
                }
            }
        }
        self.readback_run(dag_name, dag_run_id)
            .await
            .or_else(|err| {
                Err(last_error.unwrap_or_else(|| {
                    reject(
                        "WORKFLOW_ENGINE_READBACK",
                        format!("DAG run {dag_name}/{dag_run_id} did not become visible: {err}"),
                        "check_services",
                    )
                }))
            })
    }

    /// Stop a DAG run (`POST /dag-runs/{name}/{dagRunId}/stop`).
    ///
    /// Per the 2026-10-19 pre-implementation probe (`docs/research/sdk/dagu.md`), Dagu's `stop`
    /// endpoint only acts on `Running(1)` runs and returns HTTP 400 (`DAG is not running`) when
    /// a run is parked in `Waiting(7)` on a processless `human.task`. In that case, readback
    /// confirms `Waiting(7)` and returns `StopOutcome::ParkedWaitingNoProcess`.
    ///
    /// # Errors
    ///
    /// Returns a typed error if Dagu fails for a reason other than a parked `Waiting(7)` run.
    pub async fn stop(&self, dag_name: &str, dag_run_id: &str) -> Result<StopOutcome> {
        let name = DagName(dag_name.to_owned());
        let run_id = DagRunId(dag_run_id.to_owned());
        match self.client.terminate_dag_run(&name, &run_id, None).await {
            Ok(_) => Ok(StopOutcome::StoppedRunningProcess),
            Err(ProgenitorError::UnexpectedResponse(resp)) if resp.status().as_u16() == 400 => {
                let readback = self.readback_run(dag_name, dag_run_id).await?;
                if readback.status == 7 {
                    Ok(StopOutcome::ParkedWaitingNoProcess)
                } else {
                    Err(reject(
                        "WORKFLOW_ENGINE_STOP",
                        format!(
                            "Dagu refused stop for {dag_name}/{dag_run_id} in status {} ({})",
                            readback.status, readback.status_label
                        ),
                        "inspect_dagu",
                    ))
                }
            }
            Err(error) => Err(reject(
                "WORKFLOW_ENGINE_STOP",
                format!(
                    "cannot stop {dag_name}/{dag_run_id}: {}",
                    redact_error(&error)
                ),
                "check_services",
            )),
        }
    }

    /// Read back the current state of a DAG run, its steps, its spec digest, and sibling runs/DAGs.
    ///
    /// # Errors
    ///
    /// Returns `WORKFLOW_ENGINE_NOT_FOUND` on 404 or `WORKFLOW_ENGINE_READBACK` on transport/API failure.
    pub async fn readback_run(
        &self,
        dag_name: &str,
        dag_run_id: &str,
    ) -> Result<EngineRunReadback> {
        let spec_digest = match self
            .client
            .get_dag_spec(&DagFileName(dag_name.to_owned()), None)
            .await
        {
            Ok(resp) => Some(bytes_sha256(resp.into_inner().spec.trim().as_bytes())),
            Err(_) => None,
        };

        let existing_dag_files = match self
            .client
            .list_da_gs(None, None, None, None, None, None, None, None, None, None)
            .await
        {
            Ok(resp) => resp
                .into_inner()
                .dags
                .into_iter()
                .map(|dag| dag.file_name)
                .collect(),
            Err(_) => Vec::new(),
        };

        let all_run_ids = match self
            .client
            .list_dag_runs(
                None,
                None,
                None,
                None,
                None,
                Some(dag_name),
                None,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(resp) => resp
                .into_inner()
                .dag_runs
                .into_iter()
                .map(|run| run.dag_run_id.0)
                .collect(),
            Err(_) => Vec::new(),
        };

        let details = match self
            .client
            .get_dag_run_details(
                &DagName(dag_name.to_owned()),
                &DagRunId(dag_run_id.to_owned()),
                None,
            )
            .await
        {
            Ok(resp) => resp.into_inner().dag_run_details,
            Err(ProgenitorError::ErrorResponse(resp)) if resp.status().as_u16() == 404 => {
                return Err(reject(
                    "WORKFLOW_ENGINE_NOT_FOUND",
                    format!("DAG run {dag_name}/{dag_run_id} not found in Dagu"),
                    "reconcile_binding",
                ));
            }
            Err(ProgenitorError::UnexpectedResponse(resp)) if resp.status().as_u16() == 404 => {
                return Err(reject(
                    "WORKFLOW_ENGINE_NOT_FOUND",
                    format!("DAG run {dag_name}/{dag_run_id} not found in Dagu"),
                    "reconcile_binding",
                ));
            }
            Err(error) => {
                return Err(reject(
                    "WORKFLOW_ENGINE_READBACK",
                    format!(
                        "cannot read back {dag_name}/{dag_run_id}: {}",
                        redact_error(&error)
                    ),
                    "check_services",
                ));
            }
        };

        let run_resume_pending = details.human_task_resume_pending;
        let steps = details
            .nodes
            .into_iter()
            .map(|node| {
                let mut output_variables = BTreeMap::new();
                for (k, v) in node.approval_inputs {
                    output_variables.insert(k, v);
                }
                let step_name = node.step.id.clone().unwrap_or(node.step.name);
                let human_task_prompt = node.step.human_task.map(|ht| ht.prompt);
                EngineStepReadback {
                    step_name,
                    status: *node.status,
                    status_label: node.status_label.to_string(),
                    started_at: node.started_at,
                    finished_at: node.finished_at,
                    retry_count: node.retry_count,
                    done_count: node.done_count,
                    human_task_prompt,
                    human_task_resume_pending: run_resume_pending,
                    approved_at: node.approved_at,
                    approved_by: node.approved_by,
                    rejected_at: node.rejected_at,
                    rejected_by: node.rejected_by,
                    rejection_reason: node.rejection_reason,
                    output_variables,
                }
            })
            .collect();

        Ok(EngineRunReadback {
            dag_name: details.name.0,
            dag_run_id: details.dag_run_id.0,
            status: *details.status,
            status_label: details.status_label.to_string(),
            started_at: details.started_at,
            finished_at: details.finished_at,
            queued_at: details.queued_at.unwrap_or_default(),
            spec_digest,
            all_run_ids,
            existing_dag_files,
            steps,
        })
    }

    /// Read back a single step by name from `GET /dag-runs/{name}/{dagRunId}`.
    ///
    /// # Errors
    ///
    /// Returns `WORKFLOW_ENGINE_STEP_NOT_FOUND` if the step does not exist in `.dagRunDetails.nodes[]`.
    pub async fn readback_step(
        &self,
        dag_name: &str,
        dag_run_id: &str,
        step_name: &str,
    ) -> Result<EngineStepReadback> {
        let run = self.readback_run(dag_name, dag_run_id).await?;
        run.steps
            .into_iter()
            .find(|step| step.step_name == step_name)
            .ok_or_else(|| {
                reject(
                    "WORKFLOW_ENGINE_STEP_NOT_FOUND",
                    format!("step {step_name} not found in {dag_name}/{dag_run_id}"),
                    "inspect_workflow",
                )
            })
    }

    /// Complete a `human.task` checkpoint directly (`POST /dag-runs/{name}/{dagRunId}/human-tasks/{stepId}/complete`).
    ///
    /// # Errors
    ///
    /// Returns a typed error if Dagu rejects or fails to process the completion.
    pub async fn complete_human_task(
        &self,
        req: &HumanTaskCompletionRequest,
    ) -> Result<CompleteResolution> {
        let step_id = CompleteHumanTaskStepId::try_from(req.step_id.as_str())
            .map_err(|e| reject("INVALID_INPUT", e.to_string(), "correct_input"))?;
        let body = HumanTaskInput(req.to_human_task_input());
        self.client
            .complete_human_task(
                &DagName(req.dag_name.clone()),
                &req.dag_run_id,
                &step_id,
                None,
                &body,
            )
            .await
            .map_err(|error| {
                reject(
                    "WORKFLOW_ENGINE_COMPLETE",
                    format!(
                        "cannot complete human.task {}/{}/{}: {}",
                        req.dag_name,
                        req.dag_run_id,
                        req.step_id,
                        redact_error(&error)
                    ),
                    "reconcile_binding",
                )
            })?;
        Ok(CompleteResolution {
            dag_name: req.dag_name.clone(),
            dag_run_id: req.dag_run_id.clone(),
            step_id: req.step_id.clone(),
            readback_performed: false,
            readback_status_before: None,
            resubmitted: true,
            action_taken: CompletionRecoveryAction::SubmittedDirectly,
        })
    }

    /// Re-enqueue a completed `human.task` checkpoint whose queue enqueue was pending
    /// (`POST /dag-runs/{name}/{dagRunId}/human-tasks/resume`).
    ///
    /// # Errors
    ///
    /// Returns a typed error if Dagu rejects or fails to resume the human task.
    pub async fn resume_human_task(
        &self,
        dag_name: &str,
        dag_run_id: &str,
        _step_id: Option<&str>,
    ) -> Result<()> {
        self.client
            .resume_human_task_dag_run(&DagName(dag_name.to_owned()), dag_run_id, None)
            .await
            .map_err(|error| {
                reject(
                    "WORKFLOW_ENGINE_RESUME",
                    format!(
                        "cannot resume human.task {dag_name}/{dag_run_id}: {}",
                        redact_error(&error)
                    ),
                    "reconcile_binding",
                )
            })?;
        Ok(())
    }

    /// Complete a `human.task` checkpoint with mandatory readback-before-retry when the prior
    /// acknowledgement was lost or unknown (`spec/run.md` §从节点到结果 第 5 条, 验收 4).
    ///
    /// When `ack_unknown` is true, this method **always reads back the step first**:
    /// - If the step is still in `Waiting(7)`, it confirms the step did not advance and submits `complete_human_task`.
    /// - If the step is in `Success(4)` with `human_task_resume_pending == Some(true)`, it calls `resume_human_task`.
    /// - If the step has already advanced past `Waiting(7)`, it does **not** blind-resubmit `complete_human_task`.
    ///
    /// # Errors
    ///
    /// Returns a typed error if readback or the subsequent recovery call fails.
    pub async fn complete_human_task_with_readback(
        &self,
        req: &HumanTaskCompletionRequest,
        ack_unknown: bool,
    ) -> Result<CompleteResolution> {
        if !ack_unknown {
            return self.complete_human_task(req).await;
        }
        let step = self
            .readback_step(&req.dag_name, &req.dag_run_id, &req.step_id)
            .await?;
        let status_before = step.status;
        if status_before == 7 {
            self.complete_human_task(req).await?;
            return Ok(CompleteResolution {
                dag_name: req.dag_name.clone(),
                dag_run_id: req.dag_run_id.clone(),
                step_id: req.step_id.clone(),
                readback_performed: true,
                readback_status_before: Some(status_before),
                resubmitted: true,
                action_taken: CompletionRecoveryAction::ResubmittedAfterWaitingReadback,
            });
        }
        if status_before == 4 && step.human_task_resume_pending == Some(true) {
            self.resume_human_task(&req.dag_name, &req.dag_run_id, Some(&req.step_id))
                .await?;
            return Ok(CompleteResolution {
                dag_name: req.dag_name.clone(),
                dag_run_id: req.dag_run_id.clone(),
                step_id: req.step_id.clone(),
                readback_performed: true,
                readback_status_before: Some(status_before),
                resubmitted: false,
                action_taken: CompletionRecoveryAction::ResumedPendingAfterReadback,
            });
        }
        Ok(CompleteResolution {
            dag_name: req.dag_name.clone(),
            dag_run_id: req.dag_run_id.clone(),
            step_id: req.step_id.clone(),
            readback_performed: true,
            readback_status_before: Some(status_before),
            resubmitted: false,
            action_taken: CompletionRecoveryAction::ConfirmedAdvancedWithoutResubmit,
        })
    }

    async fn replace_run_with_inline_spec(
        &self,
        dag_name: &str,
        dag_run_id: &str,
        inline_spec: &str,
        expected_status: i64,
    ) -> Result<()> {
        let mut drained_steps = std::collections::BTreeSet::new();
        for _ in 0..80 {
            let Ok(run) = self.readback_run(dag_name, dag_run_id).await else {
                break;
            };
            let mut completed_any = false;
            for step in &run.steps {
                if step.status == 7 && !drained_steps.contains(&step.step_name) {
                    let req = HumanTaskCompletionRequest {
                        dag_name: dag_name.to_owned(),
                        dag_run_id: dag_run_id.to_owned(),
                        step_id: step.step_name.clone(),
                        obligation_id: "oob-drain".to_owned(),
                        attempt_generation: 0,
                        verdict: "drained".to_owned(),
                        receipt_id: "oob-drain".to_owned(),
                        response_note: None,
                    };
                    if self.complete_human_task(&req).await.is_ok() {
                        drained_steps.insert(step.step_name.clone());
                    }
                    completed_any = true;
                }
            }
            if !completed_any && !matches!(run.status, 1 | 5 | 7) {
                // Allow the scheduler's retry process to release its proc lock after writing terminal status.
                tokio::time::sleep(Duration::from_millis(160)).await;
                if self
                    .client
                    .delete_dag_run(&DagName(dag_name.to_owned()), dag_run_id, None)
                    .await
                    .is_ok()
                {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(80)).await;
        }
        let exec_body = ExecuteDagRunFromSpecBody {
            dag_run_id: Some(DagRunId(dag_run_id.to_owned())),
            labels: None,
            name: Some(dag_name.to_owned()),
            no_reuse: false,
            params: None,
            profile: None,
            singleton: false,
            spec: inline_spec.to_owned(),
            tags: None,
        };
        for attempt in 0..5 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(150)).await;
            }
            let _ = self
                .client
                .execute_dag_run_from_spec(None, &exec_body)
                .await;
            for _ in 0..20 {
                if let Ok(readback) = self.readback_run(dag_name, dag_run_id).await
                    && readback.status == expected_status
                {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(80)).await;
            }
        }
        Err(reject(
            "WORKFLOW_ENGINE_DIRECT",
            format!(
                "inline spec replacement for {dag_name}/{dag_run_id} did not reach status {expected_status}"
            ),
            "inspect_dagu",
        ))
    }

    /// Perform a direct Dagu API operation (an operator acting on Dagu's native UI/API
    /// outside HCTL control commands) for divergence verification (`CT-RUN #9`).
    ///
    /// # Errors
    ///
    /// Returns a typed error if the underlying Dagu API request fails.
    pub async fn perform_direct_dagu_operation(
        &self,
        operation: &str,
        dag_name: &str,
        dag_run_id: &str,
        step_name: Option<&str>,
        target: Option<&str>,
    ) -> Result<()> {
        let step = step_name.unwrap_or("cp_1");
        match operation {
            "start" => {
                let extra_run_id = target.unwrap_or("direct-ui-run");
                self.client
                    .execute_dag(
                        &DagFileName(dag_name.to_owned()),
                        None,
                        &ExecuteDagBody {
                            dag_name: None,
                            dag_run_id: Some(DagRunId(extra_run_id.to_owned())),
                            ..ExecuteDagBody::default()
                        },
                    )
                    .await
                    .map_err(|e| {
                        reject("WORKFLOW_ENGINE_DIRECT", redact_error(&e), "inspect_dagu")
                    })?;
            }
            "stop" => {
                if self
                    .client
                    .terminate_dag_run(
                        &DagName(dag_name.to_owned()),
                        &DagRunId(dag_run_id.to_owned()),
                        None,
                    )
                    .await
                    .is_err()
                {
                    let running_spec = serde_json::json!({
                        "steps": [{
                            "id": step,
                            "name": step,
                            "command": "sleep 30"
                        }]
                    })
                    .to_string();
                    self.replace_run_with_inline_spec(dag_name, dag_run_id, &running_spec, 1)
                        .await?;
                    self.client
                        .terminate_dag_run(
                            &DagName(dag_name.to_owned()),
                            &DagRunId(dag_run_id.to_owned()),
                            None,
                        )
                        .await
                        .map_err(|e| {
                            reject("WORKFLOW_ENGINE_DIRECT", redact_error(&e), "inspect_dagu")
                        })?;
                    for _ in 0..30 {
                        if let Ok(rb) = self.readback_run(dag_name, dag_run_id).await
                            && rb.status == 3
                        {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(80)).await;
                    }
                }
            }
            "retry" => {
                let retry_body = RetryDagRunBody {
                    dag_run_id: DagRunId(dag_run_id.to_owned()),
                    include_downstream: None,
                    step_name: Some(step.to_owned()),
                    sub_dag_run_id: None,
                };
                if self
                    .client
                    .retry_dag_run(
                        &DagName(dag_name.to_owned()),
                        &DagRunId(dag_run_id.to_owned()),
                        None,
                        &retry_body,
                    )
                    .await
                    .is_err()
                {
                    let cmd_spec = serde_json::json!({
                        "steps": [{
                            "id": step,
                            "name": step,
                            "command": "exit 1"
                        }]
                    })
                    .to_string();
                    self.replace_run_with_inline_spec(dag_name, dag_run_id, &cmd_spec, 2)
                        .await?;
                    self.client
                        .retry_dag_run(
                            &DagName(dag_name.to_owned()),
                            &DagRunId(dag_run_id.to_owned()),
                            None,
                            &retry_body,
                        )
                        .await
                        .map_err(|e| {
                            reject(
                                "WORKFLOW_ENGINE_DIRECT",
                                format!("retry: {}", redact_error(&e)),
                                "inspect_dagu",
                            )
                        })?;
                    for _ in 0..40 {
                        if let Ok(rb) = self.readback_run(dag_name, dag_run_id).await
                            && rb.status == 2
                            && rb.steps.iter().any(|s| s.status == 2)
                        {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(80)).await;
                    }
                }
            }
            "reschedule" => {
                let new_run_id = target.unwrap_or("rescheduled-ui-run");
                let body = RescheduleDagRunBody {
                    dag_run_id: Some(DagRunId(new_run_id.to_owned())),
                    dag_name: None,
                    use_current_dag_file: None,
                };
                if self
                    .client
                    .reschedule_dag_run(
                        &DagName(dag_name.to_owned()),
                        &DagRunId(dag_run_id.to_owned()),
                        None,
                        &body,
                    )
                    .await
                    .is_err()
                {
                    let req = HumanTaskCompletionRequest {
                        dag_name: dag_name.to_owned(),
                        dag_run_id: dag_run_id.to_owned(),
                        step_id: step.to_owned(),
                        obligation_id: "reschedule-drain".to_owned(),
                        attempt_generation: 0,
                        verdict: "drained".to_owned(),
                        receipt_id: "reschedule-drain".to_owned(),
                        response_note: None,
                    };
                    let _ = self.complete_human_task(&req).await;
                    for _ in 0..40 {
                        if let Ok(rb) = self.readback_run(dag_name, dag_run_id).await
                            && rb.status == 4
                        {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(80)).await;
                    }
                    self.client
                        .reschedule_dag_run(
                            &DagName(dag_name.to_owned()),
                            &DagRunId(dag_run_id.to_owned()),
                            None,
                            &body,
                        )
                        .await
                        .map_err(|e| {
                            reject(
                                "WORKFLOW_ENGINE_DIRECT",
                                format!("reschedule: {}", redact_error(&e)),
                                "inspect_dagu",
                            )
                        })?;
                }
                for _ in 0..30 {
                    if let Ok(rb) = self.readback_run(dag_name, dag_run_id).await
                        && rb.all_run_ids.iter().any(|id| id == new_run_id)
                    {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(80)).await;
                }
            }
            "approve" => {
                if self
                    .client
                    .approve_dag_run_step(
                        &DagName(dag_name.to_owned()),
                        &DagRunId(dag_run_id.to_owned()),
                        step,
                        None,
                        &ApproveStepRequest::default(),
                    )
                    .await
                    .is_err()
                {
                    let req = HumanTaskCompletionRequest {
                        dag_name: dag_name.to_owned(),
                        dag_run_id: dag_run_id.to_owned(),
                        step_id: step.to_owned(),
                        obligation_id: "direct-ui-approve".to_owned(),
                        attempt_generation: 999,
                        verdict: "accepted".to_owned(),
                        receipt_id: "direct-ui-approve".to_owned(),
                        response_note: Some("direct-ui-approve".to_owned()),
                    };
                    self.complete_human_task(&req).await?;
                }
            }
            "reject" => {
                let reject_body = RejectStepRequest {
                    reason: Some(target.unwrap_or("rejected from Dagu UI").to_owned()),
                };
                if self
                    .client
                    .reject_dag_run_step(
                        &DagName(dag_name.to_owned()),
                        &DagRunId(dag_run_id.to_owned()),
                        step,
                        None,
                        &reject_body,
                    )
                    .await
                    .is_err()
                {
                    let approval_spec = serde_json::json!({
                        "steps": [{
                            "id": step,
                            "name": step,
                            "command": "true",
                            "approval": {
                                "prompt": "approval step"
                            }
                        }]
                    })
                    .to_string();
                    self.replace_run_with_inline_spec(dag_name, dag_run_id, &approval_spec, 7)
                        .await?;
                    self.client
                        .reject_dag_run_step(
                            &DagName(dag_name.to_owned()),
                            &DagRunId(dag_run_id.to_owned()),
                            step,
                            None,
                            &reject_body,
                        )
                        .await
                        .map_err(|e| {
                            reject(
                                "WORKFLOW_ENGINE_DIRECT",
                                format!("reject: {}", redact_error(&e)),
                                "inspect_dagu",
                            )
                        })?;
                    for _ in 0..30 {
                        if let Ok(rb) = self.readback_run(dag_name, dag_run_id).await
                            && rb
                                .steps
                                .iter()
                                .any(|s| s.status == 8 || s.rejected_at.is_some())
                        {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(80)).await;
                    }
                }
            }
            "edit" => {
                let new_spec = target.unwrap_or(
                    "{\"steps\":[{\"id\":\"cp_1\",\"name\":\"cp_1\",\"action\":\"human.task\",\"with\":{\"prompt\":\"edited directly in Dagu UI\"}}]}",
                );
                self.client
                    .update_dag_spec(
                        &DagFileName(dag_name.to_owned()),
                        None,
                        &UpdateDagSpecBody {
                            spec: new_spec.to_owned(),
                        },
                    )
                    .await
                    .map_err(|e| {
                        reject("WORKFLOW_ENGINE_DIRECT", redact_error(&e), "inspect_dagu")
                    })?;
            }
            "rename" => {
                let new_name = match target {
                    Some(t) if t.starts_with(dag_name) => t.to_owned(),
                    Some(t) => format!("{dag_name}_{t}"),
                    None => format!("{dag_name}_renamed"),
                };
                self.client
                    .rename_dag(
                        &DagFileName(dag_name.to_owned()),
                        None,
                        &RenameDagBody {
                            new_file_name: new_name,
                        },
                    )
                    .await
                    .map_err(|e| {
                        reject("WORKFLOW_ENGINE_DIRECT", redact_error(&e), "inspect_dagu")
                    })?;
            }
            "delete" => {
                self.client
                    .delete_dag(&DagFileName(dag_name.to_owned()), None)
                    .await
                    .map_err(|e| {
                        reject("WORKFLOW_ENGINE_DIRECT", redact_error(&e), "inspect_dagu")
                    })?;
            }
            other => {
                return Err(reject(
                    "INVALID_INPUT",
                    format!("unknown direct Dagu operation {other}"),
                    "correct_input",
                ));
            }
        }
        Ok(())
    }
}

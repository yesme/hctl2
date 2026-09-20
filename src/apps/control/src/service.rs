//! Query / Preview / Submit / Subscribe. Store errors are forwarded unchanged.

use std::collections::VecDeque;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::owner_actor;
use crate::services::Supervisor;
use crate::socket_path;
use foundation::SecretStore;
use proto::control_server::Control;
use proto::{
    Error as ProtoError, PreviewRequest, PreviewResponse, QueryRequest, QueryResponse,
    SubmitRequest, SubmitResponse, SubscribeEvent, SubscribeRequest,
};
use serde_json::{Value, json};
use store::{StartupStatus, Store, StoreError, TrustedActor};
use tokio::sync::{Mutex, broadcast};
use tokio_stream::Stream;
use tonic::transport::server::UdsConnectInfo;
use tonic::{Request, Response, Status};

pub const PROTOCOL: &str = "hctl2.control.v1";
const LOG_LIMIT: usize = 32;
const PREVIEW_LIMIT: usize = 32;

#[derive(Clone)]
struct Preview {
    token: String,
    operation: String,
    payload: Vec<u8>,
}

#[derive(Clone)]
struct Event {
    seq: i64,
    kind: String,
    payload: Vec<u8>,
}

pub struct ControlService {
    root: PathBuf,
    status: StartupStatus,
    store: Arc<Mutex<Option<Store>>>,
    open_error: Arc<Mutex<Option<StoreError>>>,
    previews: Mutex<VecDeque<Preview>>,
    events: Mutex<VecDeque<Event>>,
    seq: AtomicI64,
    bus: broadcast::Sender<Event>,
    services: Arc<Supervisor>,
}

impl ControlService {
    #[must_use]
    pub fn new(
        root: PathBuf,
        status: StartupStatus,
        store: Arc<Mutex<Option<Store>>>,
        open_error: Arc<Mutex<Option<StoreError>>>,
    ) -> Self {
        let (bus, _) = broadcast::channel(64);
        Self {
            root: root.clone(),
            status,
            store,
            open_error,
            previews: Mutex::new(VecDeque::new()),
            events: Mutex::new(VecDeque::new()),
            seq: AtomicI64::new(0),
            bus,
            services: Arc::new(Supervisor::from_root(root)),
        }
    }

    #[must_use]
    pub fn with_services(
        root: PathBuf,
        status: StartupStatus,
        store: Arc<Mutex<Option<Store>>>,
        open_error: Arc<Mutex<Option<StoreError>>>,
        services: Arc<Supervisor>,
    ) -> Self {
        let (bus, _) = broadcast::channel(64);
        Self {
            root,
            status,
            store,
            open_error,
            previews: Mutex::new(VecDeque::new()),
            events: Mutex::new(VecDeque::new()),
            seq: AtomicI64::new(0),
            bus,
            services,
        }
    }

    fn protocol_error(request_version: &str) -> Option<ProtoError> {
        if request_version == PROTOCOL {
            None
        } else {
            Some(error(
                "PROTOCOL_MISMATCH",
                format!("unsupported protocol {request_version}"),
                "use_compatible_version",
            ))
        }
    }

    fn require_owner<T>(&self, request: &Request<T>) -> Result<TrustedActor, ProtoError> {
        let socket = socket_path(&self.root);
        let Some(info) = request.extensions().get::<UdsConnectInfo>() else {
            return Err(error(
                "PERMISSION_DENIED",
                "missing unix peer credentials",
                "reconnect_control",
            ));
        };
        let Some(cred) = info.peer_cred else {
            return Err(error(
                "PERMISSION_DENIED",
                "unix peer credentials unavailable",
                "reconnect_control",
            ));
        };
        let owner = std::fs::metadata(&socket)
            .map(|meta| meta.uid())
            .unwrap_or(u32::MAX);
        if cred.uid() != owner {
            return Err(error(
                "PERMISSION_DENIED",
                "peer uid does not own the control socket",
                "reconnect_control",
            ));
        }
        Ok(owner_actor(cred.uid()))
    }

    async fn presented_open_error(&self) -> Option<ProtoError> {
        self.open_error.lock().await.as_ref().map(present)
    }

    async fn startup_error(&self) -> Option<ProtoError> {
        if let Some(error) = self.presented_open_error().await {
            return Some(error);
        }
        match self.status.require_ready() {
            Ok(()) => None,
            Err(err) => Some(present(&err)),
        }
    }

    async fn emit(&self, kind: &str, payload: Value) -> i64 {
        let seq = self.seq.fetch_add(1, Ordering::AcqRel) + 1;
        let event = Event {
            seq,
            kind: kind.into(),
            payload: payload.to_string().into_bytes(),
        };
        let mut log = self.events.lock().await;
        log.push_back(event.clone());
        while log.len() > LOG_LIMIT {
            log.pop_front();
        }
        let _ = self.bus.send(event);
        seq
    }
}

#[tonic::async_trait]
impl Control for ControlService {
    type SubscribeStream =
        Pin<Box<dyn Stream<Item = Result<SubscribeEvent, Status>> + Send + 'static>>;

    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        let actor = match self.require_owner(&request) {
            Ok(actor) => actor,
            Err(error) => return Ok(err_query_proto(error, 0)),
        };
        let req = request.into_inner();
        if let Some(error) = Self::protocol_error(protocol(&req.protocol)) {
            return Ok(err_query_proto(error, self.seq.load(Ordering::Acquire)));
        }
        let payload = match json_bytes(&req.payload) {
            Ok(value) => value,
            Err(error) => return Ok(err_query_proto(error, 0)),
        };
        if matches!(req.kind.as_str(), "status" | "doctor" | "services") {
            return Ok(self.status_or_doctor(&req.kind, &actor).await);
        }
        if let Some(error) = self.startup_error().await {
            return Ok(err_query_proto(error, self.seq.load(Ordering::Acquire)));
        }
        let result = match req.kind.as_str() {
            "pending" => json!({"items": []}),
            "overview" => json!({"projection":"overview","placeholder":true}),
            "get" | "versions" => {
                return Ok(invalid_query(
                    "get/versions are internal store APIs and are not public Query",
                ));
            }
            "backup.verify" => {
                let Some(path) = payload.get("path").and_then(Value::as_str) else {
                    return Ok(invalid_query("backup.verify needs path"));
                };
                let path = path.to_owned();
                match tokio::task::spawn_blocking(move || Store::verify_backup(Path::new(&path)))
                    .await
                    .unwrap_or_else(|_| {
                        Err(StoreError {
                            code: "STORAGE_IO",
                            message: "backup verify worker failed".into(),
                            recovery_action: "inspect_storage",
                        })
                    }) {
                    Ok(report) => serde_json::to_value(report).unwrap_or(json!({})),
                    Err(err) => return Ok(err_query(&err, self.seq.load(Ordering::Acquire))),
                }
            }
            "restore.preview" => {
                let Some(path) = payload.get("path").and_then(Value::as_str) else {
                    return Ok(invalid_query("restore.preview needs path"));
                };
                let path = path.to_owned();
                match tokio::task::spawn_blocking(move || Store::verify_backup(Path::new(&path)))
                    .await
                    .unwrap_or_else(|_| {
                        Err(StoreError {
                            code: "STORAGE_IO",
                            message: "restore preview worker failed".into(),
                            recovery_action: "inspect_storage",
                        })
                    }) {
                    Ok(report) => json!({"dangerous":true,"report":report}),
                    Err(err) => return Ok(err_query(&err, self.seq.load(Ordering::Acquire))),
                }
            }
            other => {
                return Ok(invalid_query(&format!("unknown query {other}")));
            }
        };
        Ok(Response::new(QueryResponse {
            error: None,
            payload: result.to_string().into_bytes(),
            event_seq: self.seq.load(Ordering::Acquire),
        }))
    }

    async fn preview(
        &self,
        request: Request<PreviewRequest>,
    ) -> Result<Response<PreviewResponse>, Status> {
        if let Err(error) = self.require_owner(&request) {
            return Ok(preview_err(error));
        }
        let req = request.into_inner();
        if let Some(error) = Self::protocol_error(protocol(&req.protocol)) {
            return Ok(preview_err(error));
        }
        if let Some(error) = self.startup_error().await {
            return Ok(preview_err(error));
        }
        let dangerous = is_dangerous(&req.operation);
        let token = preview_token(&req.operation, &req.payload, &req.command_id);
        let mut previews = self.previews.lock().await;
        previews.push_back(Preview {
            token: token.clone(),
            operation: req.operation.clone(),
            payload: req.payload.clone(),
        });
        while previews.len() > PREVIEW_LIMIT {
            previews.pop_front();
        }
        drop(previews);
        Ok(Response::new(PreviewResponse {
            error: None,
            preview_token: token,
            dangerous,
            effect_summary: json!({"operation":req.operation,"dangerous":dangerous})
                .to_string()
                .into_bytes(),
        }))
    }

    async fn submit(
        &self,
        request: Request<SubmitRequest>,
    ) -> Result<Response<SubmitResponse>, Status> {
        if let Err(error) = self.require_owner(&request) {
            return Ok(submit_err(error, 0));
        }
        let req = request.into_inner();
        if let Some(error) = Self::protocol_error(protocol(&req.protocol)) {
            return Ok(submit_err(error, 0));
        }
        if req.operation.starts_with("services.") {
            return self.submit_services(&req).await;
        }
        if let Some(error) = self.startup_error().await {
            return Ok(submit_err(error, self.seq.load(Ordering::Acquire)));
        }
        if is_dangerous(&req.operation) {
            let previews = self.previews.lock().await;
            let matched = previews.iter().any(|preview| {
                preview.token == req.preview_token
                    && preview.operation == req.operation
                    && preview.payload == req.payload
            });
            if !matched {
                return Ok(submit_err(
                    error(
                        "PREVIEW_REQUIRED",
                        "dangerous submit requires a matching preview token",
                        "preview_then_submit",
                    ),
                    self.seq.load(Ordering::Acquire),
                ));
            }
        }
        let payload = match json_bytes(&req.payload) {
            Ok(value) => value,
            Err(error) => return Ok(submit_err(error, 0)),
        };
        let store = Arc::clone(&self.store);
        let operation = req.operation.clone();
        let root = self.root.clone();
        let result = match tokio::task::spawn_blocking(move || {
            let mut store_slot = store.blocking_lock();
            run_operation(&operation, &payload, &mut store_slot, &root)
        })
        .await
        {
            Ok(outcome) => match outcome {
                Ok(value) => value,
                Err(err) => return Ok(submit_err(err, 0)),
            },
            Err(_) => {
                return Ok(submit_err(
                    error("STORAGE_IO", "submit worker failed", "inspect_storage"),
                    0,
                ));
            }
        };
        if is_dangerous(&req.operation) {
            self.previews
                .lock()
                .await
                .retain(|preview| preview.token != req.preview_token);
        }
        let seq = self
            .emit("submit", json!({"operation":req.operation}))
            .await;
        Ok(Response::new(SubmitResponse {
            error: None,
            result: result.to_string().into_bytes(),
            event_seq: seq,
        }))
    }

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        if let Err(error) = self.require_owner(&request) {
            return Ok(subscribe_error(error));
        }
        let req = request.into_inner();
        if let Some(error) = Self::protocol_error(protocol(&req.protocol)) {
            return Ok(subscribe_error(error));
        }
        if let Some(error) = self.startup_error().await {
            return Ok(subscribe_error(error));
        }
        let log = self.events.lock().await;
        let oldest = log.front().map(|event| event.seq).unwrap_or(0);
        let latest = self.seq.load(Ordering::Acquire);
        if req.since_seq > 0 && (req.since_seq < oldest || req.since_seq > latest) {
            drop(log);
            return Ok(expired_snapshot(latest));
        }
        let backlog: Vec<Event> = log
            .iter()
            .filter(|event| event.seq > req.since_seq)
            .cloned()
            .collect();
        let last_backlog = backlog.last().map_or(req.since_seq, |event| event.seq);
        let rx = self.bus.subscribe();
        drop(log);
        let stream = subscribe_stream(backlog, rx, last_backlog);
        Ok(Response::new(Box::pin(stream)))
    }
}

impl ControlService {
    async fn submit_services(
        &self,
        req: &SubmitRequest,
    ) -> Result<Response<SubmitResponse>, Status> {
        if is_dangerous(&req.operation) {
            let previews = self.previews.lock().await;
            let matched = previews.iter().any(|preview| {
                preview.token == req.preview_token
                    && preview.operation == req.operation
                    && preview.payload == req.payload
            });
            if !matched {
                return Ok(submit_err(
                    error(
                        "PREVIEW_REQUIRED",
                        "dangerous submit requires a matching preview token",
                        "preview_then_submit",
                    ),
                    self.seq.load(Ordering::Acquire),
                ));
            }
        }
        let payload = match json_bytes(&req.payload) {
            Ok(value) => value,
            Err(error) => return Ok(submit_err(error, 0)),
        };
        let services = Arc::clone(&self.services);
        let operation = req.operation.clone();
        let outcome = tokio::task::spawn_blocking(move || match operation.as_str() {
            "services.stop" => services.stop_consumed().map(|()| json!({"stopped": true})),
            "services.backup" => payload
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| "services.backup needs path".to_owned())
                .and_then(|path| services.backup(Path::new(path))),
            "services.restore" => payload
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| "services.restore needs path".to_owned())
                .and_then(|path| services.restore(Path::new(path))),
            other => Err(format!("unknown operation {other}")),
        })
        .await
        .unwrap_or_else(|_| Err("services worker failed".into()));
        match outcome {
            Ok(value) => {
                if is_dangerous(&req.operation) {
                    self.previews
                        .lock()
                        .await
                        .retain(|preview| preview.token != req.preview_token);
                }
                let seq = self
                    .emit("submit", json!({"operation": req.operation}))
                    .await;
                Ok(Response::new(SubmitResponse {
                    error: None,
                    result: value.to_string().into_bytes(),
                    event_seq: seq,
                }))
            }
            Err(message) => Ok(submit_err(
                error("INVALID_INPUT", message, "correct_input"),
                0,
            )),
        }
    }

    async fn status_or_doctor(&self, kind: &str, actor: &TrustedActor) -> Response<QueryResponse> {
        let seq = self.seq.load(Ordering::Acquire);
        let supervisor = Arc::clone(&self.services);
        let mut services = tokio::task::spawn_blocking(move || supervisor.snapshot())
            .await
            .map(|snapshot| snapshot.to_json())
            .unwrap_or_else(|_| json!({"backend":"unavailable"}));
        if let Some(object) = services.as_object_mut() {
            object.insert("event_seq".into(), json!(seq));
        }
        if kind == "services" {
            return Response::new(QueryResponse {
                error: None,
                payload: services.to_string().into_bytes(),
                event_seq: seq,
            });
        }
        if let Some(error) = self.startup_error().await {
            let payload = json!({
                "ready": false,
                "startup_error": {
                    "code": error.code,
                    "message": error.message,
                    "recovery_action": error.recovery_action,
                },
                "actor": actor.0,
                "services": services,
            });
            return Response::new(QueryResponse {
                error: Some(error),
                payload: payload.to_string().into_bytes(),
                event_seq: seq,
            });
        }
        let store = Arc::clone(&self.store);
        let root = self.root.clone();
        let kind = kind.to_owned();
        let actor_json = serde_json::to_value(&actor.0).unwrap_or(json!({}));
        let payload = tokio::task::spawn_blocking(move || {
            let slot = store.blocking_lock();
            let Some(store) = slot.as_ref() else {
                return Err(not_ready_error());
            };
            Ok(if kind == "doctor" {
                doctor_payload(store, &root, actor_json, services)
            } else {
                status_payload(store, actor_json, services)
            })
        })
        .await
        .unwrap_or_else(|_| {
            Err(error(
                "STORAGE_IO",
                "status worker failed",
                "inspect_storage",
            ))
        });
        match payload {
            Ok(value) => Response::new(QueryResponse {
                error: None,
                payload: value.to_string().into_bytes(),
                event_seq: seq,
            }),
            Err(error) => err_query_proto(error, seq),
        }
    }
}

fn subscribe_stream(
    backlog: Vec<Event>,
    mut rx: broadcast::Receiver<Event>,
    last_backlog: i64,
) -> impl Stream<Item = Result<SubscribeEvent, Status>> {
    tokio_stream::wrappers::ReceiverStream::new({
        let (tx, rx_out) = tokio::sync::mpsc::channel(16);
        tokio::spawn(async move {
            for event in backlog {
                if tx.send(Ok(to_proto(event))).await.is_err() {
                    return;
                }
            }
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if event.seq <= last_backlog {
                            continue;
                        }
                        if tx.send(Ok(to_proto(event))).await.is_err() {
                            return;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        let _ = tx
                            .send(Ok(SubscribeEvent {
                                error: Some(error(
                                    "CURSOR_EXPIRED",
                                    "subscriber lagged past retained events",
                                    "resync_snapshot",
                                )),
                                seq: 0,
                                kind: "snapshot".into(),
                                payload: Vec::new(),
                                snapshot: true,
                            }))
                            .await;
                        return;
                    }
                    Err(broadcast::error::RecvError::Closed) => return,
                }
            }
        });
        rx_out
    })
}

fn to_proto(event: Event) -> SubscribeEvent {
    SubscribeEvent {
        error: None,
        seq: event.seq,
        kind: event.kind,
        payload: event.payload,
        snapshot: false,
    }
}

fn run_operation(
    operation: &str,
    payload: &Value,
    store_slot: &mut Option<Store>,
    root: &Path,
) -> Result<Value, ProtoError> {
    match operation {
        "backup.create" => {
            let store = store_slot.as_mut().ok_or_else(not_ready_error)?;
            let path = payload.get("path").and_then(Value::as_str).ok_or_else(|| {
                error("INVALID_INPUT", "backup.create needs path", "correct_input")
            })?;
            store
                .backup(store.generation(), Path::new(path))
                .map(|report| serde_json::to_value(report).unwrap_or(json!({})))
                .map_err(|err| present(&err))
        }
        "backup.verify" => {
            let path = payload.get("path").and_then(Value::as_str).ok_or_else(|| {
                error("INVALID_INPUT", "backup.verify needs path", "correct_input")
            })?;
            Store::verify_backup(Path::new(path))
                .map(|report| serde_json::to_value(report).unwrap_or(json!({})))
                .map_err(|err| present(&err))
        }
        "restore.apply" => {
            let path = payload.get("path").and_then(Value::as_str).ok_or_else(|| {
                error("INVALID_INPUT", "restore.apply needs path", "correct_input")
            })?;
            *store_slot = None;
            match Store::restore(root, Path::new(path)) {
                Ok(restored) => {
                    let report = json!({"control_id": restored.control_id()});
                    *store_slot = Some(restored);
                    Ok(report)
                }
                Err(err) => {
                    if let Ok(opened) = Store::open(root) {
                        *store_slot = Some(opened);
                    }
                    Err(present(&err))
                }
            }
        }
        "export" => {
            let store = store_slot.as_ref().ok_or_else(not_ready_error)?;
            Ok(json!({
                "control_id": store.control_id(),
                "writer_generation": store.generation().0,
            }))
        }
        "ops.ping" => Ok(json!({"ok":true})),
        other => Err(error(
            "INVALID_INPUT",
            format!("unknown operation {other}"),
            "correct_input",
        )),
    }
}

fn is_dangerous(operation: &str) -> bool {
    matches!(operation, "restore.apply" | "services.restore")
}

fn preview_token(operation: &str, payload: &[u8], command_id: &str) -> String {
    foundation::bytes_sha256(&[operation.as_bytes(), payload, command_id.as_bytes()].concat())
}

fn protocol(value: &Option<proto::Protocol>) -> &str {
    value
        .as_ref()
        .map_or("", |protocol| protocol.version.as_str())
}

fn json_bytes(bytes: &[u8]) -> Result<Value, ProtoError> {
    if bytes.is_empty() {
        Ok(json!({}))
    } else {
        serde_json::from_slice(bytes)
            .map_err(|_| error("INVALID_INPUT", "payload must be JSON", "correct_input"))
    }
}

fn present(err: &StoreError) -> ProtoError {
    error(err.code, err.message.clone(), err.recovery_action)
}

fn error(code: &str, message: impl Into<String>, recovery: &str) -> ProtoError {
    ProtoError {
        code: code.into(),
        message: message.into(),
        recovery_action: recovery.into(),
    }
}

fn status_payload(store: &Store, actor: Value, services: Value) -> Value {
    json!({
        "ready": true,
        "control_id": store.control_id(),
        "writer_generation": store.generation().0,
        "protocol": PROTOCOL,
        "endpoint": "unix",
        "policy": policy_values(),
        "actor": actor,
        "services": services,
    })
}

fn doctor_payload(store: &Store, root: &Path, actor: Value, services: Value) -> Value {
    let secrets = SecretStore::detect("hctl2", root.join("secrets"));
    json!({
        "ready": true,
        "control_id": store.control_id(),
        "secret_backend": format!("{:?}", secrets.backend()),
        "policy": policy_values(),
        "socket": root.join("control.sock").display().to_string(),
        "actor": actor,
        "services": services,
    })
}

fn policy_values() -> Value {
    json!({
        "endpoint_and_connection": "loopback-unix-owner-only",
        "non_local_transport": "not_offered",
        "credential_storage": "system-keyring-then-user-file",
        "client_least_privilege": true,
    })
}

fn err_query(err: &StoreError, seq: i64) -> Response<QueryResponse> {
    err_query_proto(present(err), seq)
}

fn err_query_proto(error: ProtoError, seq: i64) -> Response<QueryResponse> {
    Response::new(QueryResponse {
        error: Some(error),
        payload: Vec::new(),
        event_seq: seq,
    })
}

fn invalid_query(message: &str) -> Response<QueryResponse> {
    err_query_proto(error("INVALID_INPUT", message, "correct_input"), 0)
}

fn submit_err(error: ProtoError, seq: i64) -> Response<SubmitResponse> {
    Response::new(SubmitResponse {
        error: Some(error),
        result: Vec::new(),
        event_seq: seq,
    })
}

fn preview_err(error: ProtoError) -> Response<PreviewResponse> {
    Response::new(PreviewResponse {
        error: Some(error),
        preview_token: String::new(),
        dangerous: false,
        effect_summary: Vec::new(),
    })
}

fn subscribe_error(
    error: ProtoError,
) -> Response<Pin<Box<dyn Stream<Item = Result<SubscribeEvent, Status>> + Send + 'static>>> {
    let stream = tokio_stream::once(Ok(SubscribeEvent {
        error: Some(error),
        seq: 0,
        kind: String::new(),
        payload: Vec::new(),
        snapshot: false,
    }));
    Response::new(Box::pin(stream))
}

fn expired_snapshot(
    latest: i64,
) -> Response<Pin<Box<dyn Stream<Item = Result<SubscribeEvent, Status>> + Send + 'static>>> {
    let stream = tokio_stream::once(Ok(SubscribeEvent {
        error: Some(error(
            "CURSOR_EXPIRED",
            "subscribe cursor is behind the retained log or in the future",
            "resync_snapshot",
        )),
        seq: latest,
        kind: "snapshot".into(),
        payload: json!({"event_seq":latest,"placeholder":true})
            .to_string()
            .into_bytes(),
        snapshot: true,
    }));
    Response::new(Box::pin(stream))
}

fn not_ready_error() -> ProtoError {
    error("STORE_NOT_READY", "storage is not serving", "check_status")
}

#[cfg(test)]
mod present_tests {
    use super::*;

    #[test]
    fn present_copies_store_code_message_and_recovery() {
        let upgrade = StoreError {
            code: "UPGRADE_IN_PROGRESS",
            message: "schema upgrade in progress".into(),
            recovery_action: "retry_after_upgrade",
        };
        let proto = present(&upgrade);
        assert_eq!(proto.code, upgrade.code);
        assert_eq!(proto.message, upgrade.message);
        assert_eq!(proto.recovery_action, upgrade.recovery_action);
        let not_ready = StoreError {
            code: "STORE_NOT_READY",
            message: "storage is not serving".into(),
            recovery_action: "check_status",
        };
        let proto = present(&not_ready);
        assert_eq!(proto.code, not_ready.code);
        assert_eq!(proto.message, not_ready.message);
        assert_eq!(proto.recovery_action, not_ready.recovery_action);
    }
}

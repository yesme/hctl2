//! Query / Preview / Submit / Subscribe. Store errors are forwarded unchanged.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::owner_actor;
use foundation::SecretStore;
use proto::control_server::Control;
use proto::{
    Error as ProtoError, PreviewRequest, PreviewResponse, QueryRequest, QueryResponse,
    SubmitRequest, SubmitResponse, SubscribeEvent, SubscribeRequest,
};
use serde_json::{Value, json};
use store::{StartupStatus, Store, StoreError};
use tokio::sync::{Mutex, broadcast};
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

pub const PROTOCOL: &str = "hctl2.control.v1";
const LOG_LIMIT: usize = 32;

#[derive(Clone)]
struct Preview {
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
    previews: Mutex<HashMap<String, Preview>>,
    events: Mutex<VecDeque<Event>>,
    seq: AtomicI64,
    bus: broadcast::Sender<Event>,
}

impl ControlService {
    #[must_use]
    pub fn new(root: PathBuf, status: StartupStatus, store: Arc<Mutex<Option<Store>>>) -> Self {
        let (bus, _) = broadcast::channel(64);
        Self {
            root,
            status,
            store,
            previews: Mutex::new(HashMap::new()),
            events: Mutex::new(VecDeque::new()),
            seq: AtomicI64::new(0),
            bus,
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

    fn startup_error(&self) -> Option<ProtoError> {
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
        let req = request.into_inner();
        if let Some(error) = Self::protocol_error(protocol(&req.protocol)) {
            return Ok(Response::new(QueryResponse {
                error: Some(error),
                payload: Vec::new(),
                event_seq: self.seq.load(Ordering::Acquire),
            }));
        }
        if let Some(error) = self.startup_error() {
            return Ok(Response::new(QueryResponse {
                error: Some(error),
                payload: Vec::new(),
                event_seq: self.seq.load(Ordering::Acquire),
            }));
        }
        let mut store_slot = self.store.lock().await;
        let Some(store) = store_slot.as_mut() else {
            return Ok(Response::new(QueryResponse {
                error: Some(error(
                    "STORE_NOT_READY",
                    "storage is not serving",
                    "check_status",
                )),
                payload: Vec::new(),
                event_seq: self.seq.load(Ordering::Acquire),
            }));
        };
        let payload = json_bytes(&req.payload);
        let result = match req.kind.as_str() {
            "status" => status_payload(store),
            "doctor" => doctor_payload(store, &self.root),
            "pending" => json!({"items": []}),
            "overview" => json!({"projection":"overview","placeholder":true}),
            "get" | "versions" => {
                return Ok(invalid_query(
                    "get/versions are internal store APIs and are not public Query",
                ));
            }
            "backup.verify" => match payload.get("path").and_then(Value::as_str) {
                Some(path) => match Store::verify_backup(Path::new(path)) {
                    Ok(report) => serde_json::to_value(report).unwrap_or(json!({})),
                    Err(err) => {
                        return Ok(err_query(&err, self.seq.load(Ordering::Acquire)));
                    }
                },
                None => {
                    return Ok(invalid_query("backup.verify needs path"));
                }
            },
            "restore.preview" => match payload.get("path").and_then(Value::as_str) {
                Some(path) => match Store::verify_backup(Path::new(path)) {
                    Ok(report) => json!({"dangerous":true,"report":report}),
                    Err(err) => {
                        return Ok(err_query(&err, self.seq.load(Ordering::Acquire)));
                    }
                },
                None => return Ok(invalid_query("restore.preview needs path")),
            },
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
        let req = request.into_inner();
        if let Some(error) = Self::protocol_error(protocol(&req.protocol)) {
            return Ok(Response::new(PreviewResponse {
                error: Some(error),
                preview_token: String::new(),
                dangerous: false,
                effect_summary: Vec::new(),
            }));
        }
        if let Some(error) = self.startup_error() {
            return Ok(Response::new(PreviewResponse {
                error: Some(error),
                preview_token: String::new(),
                dangerous: false,
                effect_summary: Vec::new(),
            }));
        }
        let dangerous = is_dangerous(&req.operation);
        let token = preview_token(&req.operation, &req.payload, &req.command_id);
        self.previews.lock().await.insert(
            token.clone(),
            Preview {
                operation: req.operation.clone(),
                payload: req.payload.clone(),
            },
        );
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
        let req = request.into_inner();
        if let Some(error) = Self::protocol_error(protocol(&req.protocol)) {
            return Ok(submit_err(error, 0));
        }
        if let Some(error) = self.startup_error() {
            return Ok(submit_err(error, self.seq.load(Ordering::Acquire)));
        }
        if is_dangerous(&req.operation) {
            let previews = self.previews.lock().await;
            match previews.get(&req.preview_token) {
                Some(preview)
                    if preview.operation == req.operation && preview.payload == req.payload => {}
                _ => {
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
        }
        let payload = json_bytes(&req.payload);
        let mut store_slot = self.store.lock().await;
        let result = match run_operation(&req.operation, &payload, &mut store_slot, &self.root) {
            Ok(value) => value,
            Err(err) => return Ok(submit_err(err, 0)),
        };
        drop(store_slot);
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
        let req = request.into_inner();
        if let Some(error) = Self::protocol_error(protocol(&req.protocol)) {
            let stream = tokio_stream::once(Ok(SubscribeEvent {
                error: Some(error),
                seq: 0,
                kind: String::new(),
                payload: Vec::new(),
                snapshot: false,
            }));
            return Ok(Response::new(Box::pin(stream)));
        }
        if let Some(error) = self.startup_error() {
            let stream = tokio_stream::once(Ok(SubscribeEvent {
                error: Some(error),
                seq: 0,
                kind: String::new(),
                payload: Vec::new(),
                snapshot: false,
            }));
            return Ok(Response::new(Box::pin(stream)));
        }
        let log = self.events.lock().await;
        let oldest = log.front().map(|event| event.seq).unwrap_or(0);
        let latest = self.seq.load(Ordering::Acquire);
        if req.since_seq > 0 && (req.since_seq < oldest || req.since_seq > latest) {
            drop(log);
            let stream = tokio_stream::once(Ok(SubscribeEvent {
                error: Some(error(
                    "CURSOR_EXPIRED",
                    "subscribe cursor is behind the retained log or in the future",
                    "resync_snapshot",
                )),
                seq: latest,
                kind: "snapshot".into(),
                payload: json!({"event_seq":latest}).to_string().into_bytes(),
                snapshot: true,
            }));
            return Ok(Response::new(Box::pin(stream)));
        }
        let backlog: Vec<Event> = log
            .iter()
            .filter(|event| event.seq > req.since_seq)
            .cloned()
            .collect();
        drop(log);
        let rx = self.bus.subscribe();
        let stream = subscribe_stream(backlog, rx);
        Ok(Response::new(Box::pin(stream)))
    }
}

fn subscribe_stream(
    backlog: Vec<Event>,
    mut rx: broadcast::Receiver<Event>,
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
    matches!(operation, "restore.apply")
}

fn preview_token(operation: &str, payload: &[u8], command_id: &str) -> String {
    foundation::bytes_sha256(&[operation.as_bytes(), payload, command_id.as_bytes()].concat())
}

fn protocol(value: &Option<proto::Protocol>) -> &str {
    value
        .as_ref()
        .map_or("", |protocol| protocol.version.as_str())
}

fn json_bytes(bytes: &[u8]) -> Value {
    if bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(bytes).unwrap_or(json!({}))
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

fn status_payload(store: &Store) -> Value {
    json!({
        "control_id": store.control_id(),
        "writer_generation": store.generation().0,
        "protocol": PROTOCOL,
        "endpoint": "unix",
        "policy": policy_values(),
        "actor": owner_actor().0,
    })
}

fn doctor_payload(store: &Store, root: &Path) -> Value {
    let secrets = SecretStore::detect("hctl2", root.join("secrets"));
    json!({
        "control_id": store.control_id(),
        "secret_backend": format!("{:?}", secrets.backend()),
        "policy": policy_values(),
        "socket": root.join("control.sock").display().to_string(),
        "actor": owner_actor().0,
    })
}

fn policy_values() -> Value {
    json!({
        "endpoint_and_connection": "loopback-unix-owner-only",
        "non_local_requires_auth": true,
        "credential_storage": "system-keyring-then-user-file",
        "client_least_privilege": true,
        "tenant_isolation": "per-control",
    })
}

fn err_query(err: &StoreError, seq: i64) -> Response<QueryResponse> {
    Response::new(QueryResponse {
        error: Some(present(err)),
        payload: Vec::new(),
        event_seq: seq,
    })
}

fn invalid_query(message: &str) -> Response<QueryResponse> {
    Response::new(QueryResponse {
        error: Some(error("INVALID_INPUT", message, "correct_input")),
        payload: Vec::new(),
        event_seq: 0,
    })
}

fn submit_err(error: ProtoError, seq: i64) -> Response<SubmitResponse> {
    Response::new(SubmitResponse {
        error: Some(error),
        result: Vec::new(),
        event_seq: seq,
    })
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

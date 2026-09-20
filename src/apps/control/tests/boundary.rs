//! Failure cases from 04-p21-kickoff §四 乙 and CT-SYSTEM Preview/Submit plus upgrade reject.

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use control::{
    ControlService, Daemon, bind_owner_socket, occupied_error, serve_listener, socket_path,
};
use hyper_util::rt::TokioIo;
use proto::control_client::ControlClient;
use proto::{
    PreviewRequest, Protocol, QueryRequest, QueryResponse, SubmitRequest, SubscribeRequest,
};
use store::{StartupStatus, Store, StoreError};
use tokio::net::UnixStream;
use tokio::sync::Mutex;
use tonic::transport::Endpoint;
use tower::service_fn;

static TEMPS: AtomicU64 = AtomicU64::new(0);

const PROTOCOL: &str = "hctl2.control.v1";

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "hctl2-control-{}-{}",
            std::process::id(),
            TEMPS.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

async fn connect(root: &std::path::Path) -> ControlClient<tonic::transport::Channel> {
    let path = socket_path(root);
    for _ in 0..80 {
        if UnixStream::connect(&path).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    let channel =
        Endpoint::from_static("http://hctl2.control")
            .connect_with_connector(service_fn({
                let path = path.clone();
                move |_: tonic::transport::Uri| {
                    let path = path.clone();
                    async move {
                        Ok::<_, std::io::Error>(TokioIo::new(UnixStream::connect(path).await?))
                    }
                }
            }))
            .await
            .unwrap();
    ControlClient::new(channel)
}

async fn spawn_daemon(root: PathBuf) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let _ = Daemon::new(root).serve().await;
    })
}

async fn spawn_service(
    root: PathBuf,
    status: StartupStatus,
    store: Arc<Mutex<Option<Store>>>,
) -> tokio::task::JoinHandle<()> {
    spawn_service_with_open_error(root, status, store, Arc::new(Mutex::new(None))).await
}

async fn spawn_service_with_open_error(
    root: PathBuf,
    status: StartupStatus,
    store: Arc<Mutex<Option<Store>>>,
    open_error: Arc<Mutex<Option<StoreError>>>,
) -> tokio::task::JoinHandle<()> {
    let listener = bind_owner_socket(&socket_path(&root)).unwrap();
    let service = ControlService::new(root, status, store, open_error);
    tokio::spawn(async move {
        let _ = serve_listener(listener, service).await;
    })
}

fn proto() -> Protocol {
    Protocol {
        version: PROTOCOL.into(),
    }
}

async fn query_kind(
    client: &mut ControlClient<tonic::transport::Channel>,
    kind: &str,
) -> QueryResponse {
    client
        .query(QueryRequest {
            protocol: Some(proto()),
            kind: kind.into(),
            payload: Vec::new(),
        })
        .await
        .unwrap()
        .into_inner()
}

fn write_identity_schema(root: &std::path::Path) {
    let conn = rusqlite::Connection::open(root.join("control.sqlite")).unwrap();
    conn.execute_batch(
        r"
        PRAGMA application_id = 1213372018;
        PRAGMA user_version = 1;
        CREATE TABLE control_identity (
            singleton INTEGER PRIMARY KEY CHECK(singleton=1),
            control_id TEXT NOT NULL UNIQUE,
            writer_generation INTEGER NOT NULL CHECK(writer_generation>=0)
        );
        INSERT INTO control_identity VALUES(1,'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',0);
        CREATE TABLE pad(x BLOB);
        INSERT INTO pad VALUES(zeroblob(4194304));
        ",
    )
    .unwrap();
}

#[tokio::test]
async fn occupied_socket_is_rejected() {
    let temp = Temp::new();
    let socket = socket_path(&temp.0);
    std::fs::create_dir_all(socket.parent().unwrap()).unwrap();
    let _live = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    let error = bind_owner_socket(&socket).unwrap_err();
    assert_eq!(error.kind(), occupied_error(&socket).kind());
    assert!(error.to_string().contains("already in use"));
}

#[tokio::test]
async fn owner_mode_and_stale_socket_are_replaced() {
    let temp = Temp::new();
    let socket = socket_path(&temp.0);
    std::fs::create_dir_all(socket.parent().unwrap()).unwrap();
    std::fs::write(&socket, b"stale").unwrap();
    let listener = bind_owner_socket(&socket).unwrap();
    let mode = std::fs::metadata(&socket).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    assert_eq!(mode & 0o077, 0);
    drop(listener);
}

#[tokio::test]
async fn client_version_mismatch_is_rejected() {
    let temp = Temp::new();
    let _server = spawn_daemon(temp.0.clone()).await;
    let mut client = connect(&temp.0).await;
    let response = client
        .query(QueryRequest {
            protocol: Some(Protocol {
                version: "hctl2.control.v0".into(),
            }),
            kind: "status".into(),
            payload: Vec::new(),
        })
        .await
        .unwrap()
        .into_inner();
    let error = response.error.expect("mismatch");
    assert_eq!(error.code, "PROTOCOL_MISMATCH");
    assert_eq!(error.recovery_action, "use_compatible_version");
}

#[tokio::test]
async fn expired_cursor_returns_snapshot() {
    let temp = Temp::new();
    let _server = spawn_daemon(temp.0.clone()).await;
    let mut client = connect(&temp.0).await;
    wait_ready(&mut client).await;
    let mut stream = client
        .subscribe(SubscribeRequest {
            protocol: Some(proto()),
            since_seq: 99,
        })
        .await
        .unwrap()
        .into_inner();
    let event = stream.message().await.unwrap().unwrap();
    let error = event.error.expect("cursor");
    assert_eq!(error.code, "CURSOR_EXPIRED");
    assert!(event.snapshot);
}

#[tokio::test]
async fn subscribe_replays_backlog_after_submit() {
    let temp = Temp::new();
    let _server = spawn_daemon(temp.0.clone()).await;
    let mut client = connect(&temp.0).await;
    wait_ready(&mut client).await;
    let ping = client
        .submit(SubmitRequest {
            protocol: Some(proto()),
            operation: "ops.ping".into(),
            payload: Vec::new(),
            command_id: "ping-1".into(),
            idempotency_key: "ping-1".into(),
            preview_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(ping.error.is_none());
    let mut stream = client
        .subscribe(SubscribeRequest {
            protocol: Some(proto()),
            since_seq: 0,
        })
        .await
        .unwrap()
        .into_inner();
    let event = stream.message().await.unwrap().unwrap();
    assert!(event.error.is_none());
    assert_eq!(event.kind, "submit");
    assert!(!event.snapshot);
}

#[tokio::test]
async fn dangerous_submit_without_preview_is_rejected_ordinary_is_not() {
    let temp = Temp::new();
    let _server = spawn_daemon(temp.0.clone()).await;
    let mut client = connect(&temp.0).await;
    wait_ready(&mut client).await;
    let denied = client
        .submit(SubmitRequest {
            protocol: Some(proto()),
            operation: "restore.apply".into(),
            payload: br#"{"path":"/nope"}"#.to_vec(),
            command_id: "r1".into(),
            idempotency_key: "r1".into(),
            preview_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    let error = denied.error.expect("preview required");
    assert_eq!(error.code, "PREVIEW_REQUIRED");
    let ping = client
        .submit(SubmitRequest {
            protocol: Some(proto()),
            operation: "ops.ping".into(),
            payload: Vec::new(),
            command_id: "p1".into(),
            idempotency_key: "p1".into(),
            preview_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(ping.error.is_none());
    let preview = client
        .preview(PreviewRequest {
            protocol: Some(proto()),
            operation: "ops.ping".into(),
            payload: Vec::new(),
            command_id: "p2".into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(!preview.dangerous);
    let via_preview = client
        .submit(SubmitRequest {
            protocol: Some(proto()),
            operation: "ops.ping".into(),
            payload: Vec::new(),
            command_id: "p2".into(),
            idempotency_key: "p2".into(),
            preview_token: preview.preview_token,
        })
        .await
        .unwrap()
        .into_inner();
    assert!(via_preview.error.is_none());
    assert_eq!(ping.result, via_preview.result);
    let restore_preview = client
        .preview(PreviewRequest {
            protocol: Some(proto()),
            operation: "restore.apply".into(),
            payload: br#"{"path":"/nope"}"#.to_vec(),
            command_id: "r2".into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(restore_preview.dangerous);
    let after_preview = client
        .submit(SubmitRequest {
            protocol: Some(proto()),
            operation: "restore.apply".into(),
            payload: br#"{"path":"/nope"}"#.to_vec(),
            command_id: "r2".into(),
            idempotency_key: "r2".into(),
            preview_token: restore_preview.preview_token,
        })
        .await
        .unwrap()
        .into_inner();
    let error = after_preview
        .error
        .expect("restore still fails on missing backup");
    assert_ne!(error.code, "PREVIEW_REQUIRED");
}

#[tokio::test]
async fn get_and_versions_are_not_public_query() {
    let temp = Temp::new();
    let _server = spawn_daemon(temp.0.clone()).await;
    let mut client = connect(&temp.0).await;
    wait_ready(&mut client).await;
    for kind in ["get", "versions"] {
        let error = query_kind(&mut client, kind)
            .await
            .error
            .expect("internal API");
        assert_eq!(error.code, "INVALID_INPUT");
        assert!(error.message.contains("internal store APIs"), "{kind}");
    }
}

#[tokio::test]
async fn store_not_ready_and_upgrade_are_presented_on_rpc() {
    let temp = Temp::new();
    write_identity_schema(&temp.0);
    let status = StartupStatus::default();
    let store = Arc::new(Mutex::new(None));
    let _server = spawn_service(temp.0.clone(), status.clone(), Arc::clone(&store)).await;
    let mut client = connect(&temp.0).await;
    let not_ready = query_kind(&mut client, "status")
        .await
        .error
        .expect("not ready");
    assert_eq!(not_ready.code, "STORE_NOT_READY");
    assert_eq!(not_ready.message, "storage is not serving");
    assert_eq!(not_ready.recovery_action, "check_status");
    let denied = client
        .submit(SubmitRequest {
            protocol: Some(proto()),
            operation: "ops.ping".into(),
            payload: Vec::new(),
            command_id: "blocked".into(),
            idempotency_key: "blocked".into(),
            preview_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner()
        .error
        .expect("writes blocked while not ready");
    assert_eq!(denied.code, "STORE_NOT_READY");

    let root = temp.0.clone();
    let opening = status.clone();
    let opened = store.clone();
    let mut probe = connect(&temp.0).await;
    let probe_task = tokio::spawn(async move {
        for _ in 0..5_000 {
            if let Some(error) = query_kind(&mut probe, "status").await.error {
                if error.code == "UPGRADE_IN_PROGRESS" {
                    assert_eq!(error.message, "schema upgrade in progress");
                    assert_eq!(error.recovery_action, "retry_after_upgrade");
                    return true;
                }
            }
            tokio::task::yield_now().await;
        }
        false
    });
    let join = tokio::task::spawn_blocking(move || Store::open_with_status(&root, opening));
    let seen_upgrade = probe_task.await.unwrap();
    let store_handle = join.await.unwrap().expect("upgrade");
    *opened.lock().await = Some(store_handle);
    assert!(
        seen_upgrade,
        "upgrade-in-progress was never presented on the public Query"
    );
    wait_ready(&mut client).await;
}

#[tokio::test]
async fn subscribe_and_submit_keep_contiguous_seq() {
    let temp = Temp::new();
    let _server = spawn_daemon(temp.0.clone()).await;
    let mut subscriber = connect(&temp.0).await;
    wait_ready(&mut subscriber).await;
    let mut stream = subscriber
        .subscribe(SubscribeRequest {
            protocol: Some(proto()),
            since_seq: 0,
        })
        .await
        .unwrap()
        .into_inner();
    let mut publisher = connect(&temp.0).await;
    const N: i64 = 12;
    let publish = tokio::spawn(async move {
        for i in 0..N {
            let response = publisher
                .submit(SubmitRequest {
                    protocol: Some(proto()),
                    operation: "ops.ping".into(),
                    payload: Vec::new(),
                    command_id: format!("c{i}"),
                    idempotency_key: format!("c{i}"),
                    preview_token: String::new(),
                })
                .await
                .unwrap()
                .into_inner();
            assert!(response.error.is_none());
        }
    });
    let mut seqs = Vec::new();
    while (seqs.len() as i64) < N {
        let event = tokio::time::timeout(Duration::from_secs(2), stream.message())
            .await
            .expect("event")
            .unwrap()
            .unwrap();
        assert!(event.error.is_none(), "{:?}", event.error);
        seqs.push(event.seq);
    }
    publish.await.unwrap();
    seqs.sort_unstable();
    assert_eq!(seqs, (1..=N).collect::<Vec<_>>());
}

#[tokio::test]
async fn malformed_json_payload_is_rejected() {
    let temp = Temp::new();
    let _server = spawn_daemon(temp.0.clone()).await;
    let mut client = connect(&temp.0).await;
    wait_ready(&mut client).await;
    let response = client
        .submit(SubmitRequest {
            protocol: Some(proto()),
            operation: "backup.create".into(),
            payload: b"not-json".to_vec(),
            command_id: "bad".into(),
            idempotency_key: "bad".into(),
            preview_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    let error = response.error.expect("json");
    assert_eq!(error.code, "INVALID_INPUT");
    assert!(error.message.contains("JSON"));
}

#[tokio::test]
async fn store_open_failure_is_presented_on_status() {
    let temp = Temp::new();
    let open_error = Arc::new(Mutex::new(Some(StoreError {
        code: "WRITER_BUSY",
        message: "another control writer owns the storage".into(),
        recovery_action: "stop_previous_writer",
    })));
    let _server = spawn_service_with_open_error(
        temp.0.clone(),
        StartupStatus::default(),
        Arc::new(Mutex::new(None)),
        open_error,
    )
    .await;
    let mut client = connect(&temp.0).await;
    let response = query_kind(&mut client, "status").await;
    let error = response.error.expect("open failure");
    assert_eq!(error.code, "WRITER_BUSY");
    assert_eq!(error.recovery_action, "stop_previous_writer");
    let body: serde_json::Value = serde_json::from_slice(&response.payload).unwrap();
    assert_eq!(body["ready"], false);
    assert_eq!(body["startup_error"]["code"], "WRITER_BUSY");
}

async fn wait_ready(client: &mut ControlClient<tonic::transport::Channel>) {
    for _ in 0..80 {
        let response = query_kind(client, "status").await;
        if response.error.is_none() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("control never became ready");
}

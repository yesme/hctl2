//! Hosted service lifecycle: first consumption, probes, service death vs store.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use control::{Daemon, Supervisor, socket_path};
use hyper_util::rt::TokioIo;
use proto::control_client::ControlClient;
use proto::{Protocol, QueryRequest, SubmitRequest};
use tokio::net::UnixStream;
use tonic::transport::Endpoint;
use tower::service_fn;

static TEMPS: AtomicU64 = AtomicU64::new(0);
const PROTOCOL: &str = "hctl2.control.v1";

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "hctl2-services-{}-{}",
            std::process::id(),
            TEMPS.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = Supervisor::from_root(self.0.clone()).shutdown_project();
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

fn proto() -> Protocol {
    Protocol {
        version: PROTOCOL.into(),
    }
}

async fn query_json(
    client: &mut ControlClient<tonic::transport::Channel>,
    kind: &str,
) -> serde_json::Value {
    let response = client
        .query(QueryRequest {
            protocol: Some(proto()),
            kind: kind.into(),
            payload: Vec::new(),
        })
        .await
        .unwrap()
        .into_inner();
    assert!(response.error.is_none(), "{:?}", response.error);
    serde_json::from_slice(&response.payload).unwrap()
}

async fn wait_ready(client: &mut ControlClient<tonic::transport::Channel>) {
    for _ in 0..80 {
        let response = client
            .query(QueryRequest {
                protocol: Some(proto()),
                kind: "status".into(),
                payload: Vec::new(),
            })
            .await
            .unwrap()
            .into_inner();
        if response.error.is_none() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("control never became ready");
}

fn hosted<'a>(body: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    body["hosted"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == name)
        .unwrap()
}

#[tokio::test]
async fn probe_not_ready_is_not_available_and_store_still_serves() {
    let temp = Temp::new();
    let _server = tokio::spawn({
        let root = temp.0.clone();
        async move {
            let _ = Daemon::new(root).serve().await;
        }
    });
    let mut client = connect(&temp.0).await;
    wait_ready(&mut client).await;
    let status = query_json(&mut client, "status").await;
    assert_eq!(status["ready"], true);
    assert!(status["control_id"].as_str().unwrap().len() == 32);
    // Hosted but not consumed: start must not bring it up.
    wait_available(&mut client, "ready-ok").await;
    let services = query_json(&mut client, "services").await;
    let never = hosted(&services, "never-ready");
    assert_eq!(never["consumed"], false, "{services}");
    assert_eq!(never["running"], false, "{services}");
    // First consumption starts it; a failing probe keeps it unavailable while
    // the store keeps serving.
    let result = submit_json(
        &mut client,
        "services.consume",
        r#"{"component":"never-ready"}"#,
    )
    .await;
    assert_eq!(result["consumed"], true, "{result}");
    let mut running = false;
    for _ in 0..40 {
        let services = query_json(&mut client, "services").await;
        if hosted(&services, "never-ready")["running"] == true {
            running = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(running, "never-ready was consumed but never started");
    let services = query_json(&mut client, "services").await;
    let never = hosted(&services, "never-ready");
    assert_eq!(never["consumed"], true, "{services}");
    assert_eq!(never["available"], false, "{services}");
    assert_eq!(never["ready"], false);
    let status = query_json(&mut client, "status").await;
    assert_eq!(status["ready"], true);
    let denied = submit_json(&mut client, "services.consume", r#"{"component":"crash"}"#).await;
    assert_eq!(denied["error"]["code"], "INVALID_INPUT", "{denied}");
}

#[tokio::test]
async fn consumption_persists_across_control_restart() {
    let temp = Temp::new();
    let server = tokio::spawn({
        let root = temp.0.clone();
        async move {
            let _ = Daemon::new(root).serve().await;
        }
    });
    let mut client = connect(&temp.0).await;
    wait_ready(&mut client).await;
    wait_available(&mut client, "ready-ok").await;
    let result = submit_json(
        &mut client,
        "services.consume",
        r#"{"component":"never-ready"}"#,
    )
    .await;
    assert_eq!(result["consumed"], true, "{result}");
    assert!(temp.0.join("hosted-consumed.json").is_file());
    let stopped = submit_json(&mut client, "services.stop", "{}").await;
    assert_eq!(stopped["stopped"], true, "{stopped}");
    // The open connection keeps the old service (and its store lock) alive;
    // close it before stopping the daemon so the new one can take the lock.
    drop(client);
    server.abort();
    let _ = server.await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let _server = tokio::spawn({
        let root = temp.0.clone();
        async move {
            let _ = Daemon::new(root).serve().await;
        }
    });
    let mut client = connect(&temp.0).await;
    wait_ready(&mut client).await;
    wait_available(&mut client, "ready-ok").await;
    let mut running = false;
    let mut last = serde_json::Value::Null;
    for _ in 0..200 {
        let services = query_json(&mut client, "services").await;
        let never = hosted(&services, "never-ready");
        if never["consumed"] == true && never["running"] == true {
            running = true;
            break;
        }
        last = services;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(
        running,
        "consumed component was not restarted after control restart: {last}"
    );
}

async fn wait_available(client: &mut ControlClient<tonic::transport::Channel>, name: &str) {
    for _ in 0..80 {
        let services = query_json(client, "services").await;
        if hosted(&services, name)["available"] == true {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("{name} never became available");
}

async fn submit_json(
    client: &mut ControlClient<tonic::transport::Channel>,
    operation: &str,
    payload: &str,
) -> serde_json::Value {
    let id = format!("{operation}-{}", TEMPS.fetch_add(1, Ordering::Relaxed));
    let response = client
        .submit(SubmitRequest {
            protocol: Some(proto()),
            operation: operation.into(),
            payload: payload.as_bytes().to_vec(),
            command_id: id.clone(),
            idempotency_key: id,
            preview_token: String::new(),
        })
        .await
        .unwrap()
        .into_inner();
    match response.error {
        Some(error) => serde_json::json!({"error": {
            "code": error.code, "message": error.message, "recovery_action": error.recovery_action,
        }}),
        None => serde_json::from_slice(&response.result).unwrap(),
    }
}

#[tokio::test]
async fn service_death_does_not_change_store() {
    let temp = Temp::new();
    let _server = tokio::spawn({
        let root = temp.0.clone();
        async move {
            let _ = Daemon::new(root).serve().await;
        }
    });
    let mut client = connect(&temp.0).await;
    wait_ready(&mut client).await;
    let mut available = false;
    for _ in 0..40 {
        let services = query_json(&mut client, "services").await;
        if hosted(&services, "ready-ok")["available"] == true {
            available = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(available, "ready-ok never became available");
    let before = query_json(&mut client, "status").await;
    let control_id = before["control_id"].clone();
    let generation = before["writer_generation"].clone();
    let services = query_json(&mut client, "services").await;
    let pid = hosted(&services, "ready-ok")["pid"]
        .as_i64()
        .expect("ready-ok pid");
    let _ = std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .status();
    tokio::time::sleep(Duration::from_millis(200)).await;
    let after = query_json(&mut client, "status").await;
    assert_eq!(after["control_id"], control_id);
    assert_eq!(after["writer_generation"], generation);
}

#[tokio::test]
async fn stop_consumed_leaves_other_processes() {
    let temp = Temp::new();
    let _server = tokio::spawn({
        let root = temp.0.clone();
        async move {
            let _ = Daemon::new(root).serve().await;
        }
    });
    let mut client = connect(&temp.0).await;
    wait_ready(&mut client).await;
    for _ in 0..40 {
        let services = query_json(&mut client, "services").await;
        if hosted(&services, "ready-ok")["available"] == true {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let services = query_json(&mut client, "services").await;
    let socket = services["source"]
        .as_str()
        .and_then(|source| source.strip_prefix("process-compose:"))
        .expect("socket");
    let pc = std::env::var("HCTL2_PROCESS_COMPOSE_BIN").expect("pc bin");
    let status = std::process::Command::new(&pc)
        .env("PC_DISABLE_DOTENV", "1")
        .args([
            "--use-uds",
            "--unix-socket",
            socket,
            "process",
            "start",
            "other-ok",
        ])
        .status()
        .unwrap();
    assert!(status.success(), "start other-ok");
    let mut other_ready = false;
    for _ in 0..40 {
        let output = std::process::Command::new(&pc)
            .env("PC_DISABLE_DOTENV", "1")
            .args([
                "--use-uds",
                "--unix-socket",
                socket,
                "process",
                "get",
                "other-ok",
                "--output",
                "json",
            ])
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&output.stdout);
        if text.contains("\"is_running\": true") || text.contains("\"is_running\":true") {
            other_ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(other_ready, "other-ok never started");
    let _ = client
        .submit(SubmitRequest {
            protocol: Some(proto()),
            operation: "services.stop".into(),
            payload: Vec::new(),
            command_id: "stop-others".into(),
            idempotency_key: "stop-others".into(),
            preview_token: String::new(),
        })
        .await
        .unwrap();
    let services = query_json(&mut client, "services").await;
    assert_eq!(hosted(&services, "ready-ok")["available"], false);
    let output = std::process::Command::new(&pc)
        .env("PC_DISABLE_DOTENV", "1")
        .args([
            "--use-uds",
            "--unix-socket",
            socket,
            "process",
            "get",
            "other-ok",
            "--output",
            "json",
        ])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains("\"is_running\": true") || text.contains("\"is_running\":true"),
        "other-ok should keep running after services.stop: {text}"
    );
}

#[tokio::test]
async fn corrupt_consumed_record_is_reported_not_treated_as_empty() {
    let temp = Temp::new();
    std::fs::write(temp.0.join("hosted-consumed.json"), b"{").unwrap();
    let _server = tokio::spawn({
        let root = temp.0.clone();
        async move {
            let _ = Daemon::new(root).serve().await;
        }
    });
    let mut client = connect(&temp.0).await;
    wait_ready(&mut client).await;
    // The baseline still comes up; the broken record is reported, not ignored.
    wait_available(&mut client, "ready-ok").await;
    let services = query_json(&mut client, "services").await;
    let last_error = services["last_error"].as_str().unwrap_or("");
    assert!(last_error.contains("hosted-consumed.json"), "{services}");
    assert_eq!(
        hosted(&services, "never-ready")["running"],
        false,
        "{services}"
    );
    let denied = submit_json(
        &mut client,
        "services.consume",
        r#"{"component":"never-ready"}"#,
    )
    .await;
    assert_eq!(denied["error"]["code"], "INVALID_INPUT", "{denied}");
    assert_eq!(
        std::fs::read(temp.0.join("hosted-consumed.json")).unwrap(),
        b"{"
    );
}

#[tokio::test]
async fn backup_restored_into_a_new_root_keeps_consumption() {
    let temp = Temp::new();
    let _server = tokio::spawn({
        let root = temp.0.clone();
        async move {
            let _ = Daemon::new(root).serve().await;
        }
    });
    let mut client = connect(&temp.0).await;
    wait_ready(&mut client).await;
    wait_available(&mut client, "ready-ok").await;
    let result = submit_json(
        &mut client,
        "services.consume",
        r#"{"component":"never-ready"}"#,
    )
    .await;
    assert_eq!(result["consumed"], true, "{result}");
    let backup = temp.0.join("services-backup");
    let payload = format!(r#"{{"path":"{}"}}"#, backup.display());
    let backed = submit_json(&mut client, "services.backup", &payload).await;
    assert_eq!(backed["backend"], "fixture", "{backed}");
    assert!(backup.join("hosted-consumed.json").is_file());
    // Fresh control root, restore, and the consumed component comes back.
    let fresh = Temp::new();
    let _fresh_server = tokio::spawn({
        let root = fresh.0.clone();
        async move {
            let _ = Daemon::new(root).serve().await;
        }
    });
    let mut fresh_client = connect(&fresh.0).await;
    wait_ready(&mut fresh_client).await;
    let before = query_json(&mut fresh_client, "services").await;
    assert_eq!(
        hosted(&before, "never-ready")["consumed"],
        false,
        "{before}"
    );
    let preview = fresh_client
        .preview(proto::PreviewRequest {
            protocol: Some(proto()),
            operation: "services.restore".into(),
            payload: payload.as_bytes().to_vec(),
            command_id: "restore-preview".into(),
        })
        .await
        .unwrap()
        .into_inner();
    let response = fresh_client
        .submit(SubmitRequest {
            protocol: Some(proto()),
            operation: "services.restore".into(),
            payload: payload.as_bytes().to_vec(),
            command_id: "restore-apply".into(),
            idempotency_key: "restore-apply".into(),
            preview_token: preview.preview_token,
        })
        .await
        .unwrap()
        .into_inner();
    assert!(response.error.is_none(), "{:?}", response.error);
    let mut running = false;
    let mut last = serde_json::Value::Null;
    for _ in 0..200 {
        let services = query_json(&mut fresh_client, "services").await;
        let never = hosted(&services, "never-ready");
        if never["consumed"] == true && never["running"] == true {
            running = true;
            break;
        }
        last = services;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(running, "restore into a new root lost consumption: {last}");
}

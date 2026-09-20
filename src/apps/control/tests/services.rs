//! Hosted service lifecycle: probes, start order, and service death vs store.

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

fn consumed<'a>(body: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    body["consumed"]
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
    let services = query_json(&mut client, "services").await;
    let never = consumed(&services, "never-ready");
    for _ in 0..20 {
        if never["available"] == false {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let services = query_json(&mut client, "services").await;
    let never = consumed(&services, "never-ready");
    assert_eq!(never["available"], false, "{services}");
    assert_eq!(never["ready"], false);
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
        if consumed(&services, "ready-ok")["available"] == true {
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
    let pid = consumed(&services, "ready-ok")["pid"]
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
        if consumed(&services, "ready-ok")["available"] == true {
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
    assert_eq!(consumed(&services, "ready-ok")["available"], false);
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

//! Public `hctl2` CLI. Governance writes go through the control socket.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use clap::{Parser, Subcommand};
use hyper_util::rt::TokioIo;
use proto::control_client::ControlClient;
use proto::{PreviewRequest, Protocol, QueryRequest, SubmitRequest};
use serde_json::{Value, json};
use tokio::net::UnixStream;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

const PROTOCOL: &str = "hctl2.control.v1";

#[derive(Parser)]
#[command(name = "hctl2", version, about = "HCTL2 public CLI")]
struct Args {
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init,
    Start,
    Status,
    Doctor,
    Export {
        path: Option<PathBuf>,
    },
    #[command(subcommand)]
    Backup(BackupCommand),
    #[command(subcommand)]
    Restore(RestoreCommand),
    Query {
        kind: String,
    },
}

#[derive(Subcommand)]
enum BackupCommand {
    Create { path: PathBuf },
    Verify { path: PathBuf },
}

#[derive(Subcommand)]
enum RestoreCommand {
    Preview { path: PathBuf },
    Apply { path: PathBuf },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let root = args.root.unwrap_or_else(default_root);
    if let Err(error) = dispatch(args.command, &root, args.json).await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn default_root() -> PathBuf {
    match std::env::var_os("HCTL2_ROOT") {
        Some(root) => PathBuf::from(root),
        None => match std::env::var_os("HOME") {
            Some(home) => Path::new(&home).join(".hctl2").join("control"),
            None => PathBuf::from(".hctl2/control"),
        },
    }
}

async fn dispatch(command: Command, root: &Path, json: bool) -> Result<(), String> {
    match command {
        Command::Init => {
            std::fs::create_dir_all(root).map_err(io)?;
            print_out(
                json,
                json!({"root": root.display().to_string(), "initialized": true}),
            );
            Ok(())
        }
        Command::Start => start_daemon(root).await,
        Command::Status => query(root, json, "status", json!({})).await,
        Command::Doctor => query(root, json, "doctor", json!({})).await,
        Command::Export { path } => {
            let result = submit(root, "export", json!({}), None).await?;
            if let Some(path) = path {
                std::fs::write(
                    &path,
                    serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?,
                )
                .map_err(io)?;
            }
            print_out(json, result);
            Ok(())
        }
        Command::Backup(BackupCommand::Create { path }) => {
            let result = submit(root, "backup.create", json!({"path": path}), None).await?;
            print_out(json, result);
            Ok(())
        }
        Command::Backup(BackupCommand::Verify { path }) => {
            query(root, json, "backup.verify", json!({"path": path})).await
        }
        Command::Restore(RestoreCommand::Preview { path }) => {
            let preview = preview(root, "restore.apply", json!({"path": path})).await?;
            print_out(json, preview);
            Ok(())
        }
        Command::Restore(RestoreCommand::Apply { path }) => {
            let payload = json!({"path": path});
            let preview = preview(root, "restore.apply", payload.clone()).await?;
            let token = preview
                .get("preview_token")
                .and_then(Value::as_str)
                .ok_or_else(|| "preview missing token".to_owned())?;
            let result = submit(root, "restore.apply", payload, Some(token)).await?;
            print_out(json, result);
            Ok(())
        }
        Command::Query { kind } => query(root, json, &kind, json!({})).await,
    }
}

async fn start_daemon(root: &Path) -> Result<(), String> {
    std::fs::create_dir_all(root).map_err(io)?;
    let socket = root.join("control.sock");
    if UnixStream::connect(&socket).await.is_ok() {
        return Err(format!(
            "control socket already in use: {}",
            socket.display()
        ));
    }
    let bin = std::env::var_os("HCTL2_CONTROL_BIN")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe().ok().and_then(|exe| {
                let sibling = exe.parent()?.join("hctl2-control");
                sibling.exists().then_some(sibling)
            })
        })
        .ok_or_else(|| "hctl2-control binary not found; set HCTL2_CONTROL_BIN".to_owned())?;
    let mut child = std::process::Command::new(bin)
        .arg("--root")
        .arg(root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(io)?;
    for _ in 0..50 {
        if query_raw(root, "status", json!({})).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let _ = child.kill();
    Err("control did not become ready".into())
}

async fn query(root: &Path, json_out: bool, kind: &str, payload: Value) -> Result<(), String> {
    let response = query_raw(root, kind, payload).await?;
    print_out(json_out, response);
    Ok(())
}

async fn query_raw(root: &Path, kind: &str, payload: Value) -> Result<Value, String> {
    let mut client = client(root).await?;
    let response = client
        .query(QueryRequest {
            protocol: Some(Protocol {
                version: PROTOCOL.into(),
            }),
            kind: kind.into(),
            payload: payload.to_string().into_bytes(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    if let Some(error) = response.error {
        return Err(format!(
            "{}: {} ({})",
            error.code, error.message, error.recovery_action
        ));
    }
    bytes_json(&response.payload)
}

async fn preview(root: &Path, operation: &str, payload: Value) -> Result<Value, String> {
    let mut client = client(root).await?;
    let response = client
        .preview(PreviewRequest {
            protocol: Some(Protocol {
                version: PROTOCOL.into(),
            }),
            operation: operation.into(),
            payload: payload.to_string().into_bytes(),
            command_id: "cli-preview".into(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    if let Some(error) = response.error {
        return Err(format!(
            "{}: {} ({})",
            error.code, error.message, error.recovery_action
        ));
    }
    Ok(json!({
        "preview_token": response.preview_token,
        "dangerous": response.dangerous,
        "effect_summary": bytes_json(&response.effect_summary)?,
    }))
}

async fn submit(
    root: &Path,
    operation: &str,
    payload: Value,
    token: Option<&str>,
) -> Result<Value, String> {
    let mut client = client(root).await?;
    let response = client
        .submit(SubmitRequest {
            protocol: Some(Protocol {
                version: PROTOCOL.into(),
            }),
            operation: operation.into(),
            payload: payload.to_string().into_bytes(),
            command_id: format!("cli-{operation}"),
            idempotency_key: format!("cli-{operation}"),
            preview_token: token.unwrap_or("").into(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    if let Some(error) = response.error {
        return Err(format!(
            "{}: {} ({})",
            error.code, error.message, error.recovery_action
        ));
    }
    bytes_json(&response.result)
}

async fn client(root: &Path) -> Result<ControlClient<tonic::transport::Channel>, String> {
    let path = root.join("control.sock");
    let uri = Uri::from_static("http://hctl2.control");
    let channel = Endpoint::from(uri)
        .connect_with_connector(service_fn(move |_: Uri| {
            let path = path.clone();
            async move {
                let stream = UnixStream::connect(path).await?;
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await
        .map_err(|e| e.to_string())?;
    Ok(ControlClient::new(channel))
}

fn bytes_json(bytes: &[u8]) -> Result<Value, String> {
    if bytes.is_empty() {
        Ok(json!({}))
    } else {
        serde_json::from_slice(bytes).map_err(|e| e.to_string())
    }
}

fn print_out(as_json: bool, value: Value) {
    if as_json {
        println!("{}", value);
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
        );
    }
}

fn io(error: std::io::Error) -> String {
    error.to_string()
}

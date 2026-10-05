//! Public `hctl2` CLI. Governance writes go through the control socket.

#![forbid(unsafe_code)]

mod agency;
mod profile;
mod project;
mod repo;
mod room;
mod task;

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
    /// Build a Context assembly preview without saving it.
    Context {
        #[command(subcommand)]
        command: ContextCommand,
    },
    /// Create the control root; records `--secret-backend` when given.
    Init {
        /// Persistent secret backend: `system-keyring` (default when available) or `user-file`.
        #[arg(long, value_name = "BACKEND", value_parser = ["system-keyring", "user-file"])]
        secret_backend: Option<String>,
    },
    /// Start the control daemon; records `--secret-backend` when given.
    Start {
        /// Persistent secret backend: `system-keyring` (default when available) or `user-file`.
        #[arg(long, value_name = "BACKEND", value_parser = ["system-keyring", "user-file"])]
        secret_backend: Option<String>,
    },
    Stop,
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
    #[command(subcommand)]
    Services(ServicesCommand),
    #[command(subcommand)]
    Repo(repo::RepoCommand),
    #[command(subcommand)]
    Task(task::TaskCommand),
    #[command(subcommand)]
    Room(room::RoomCommand),
    #[command(subcommand)]
    Project(project::ProjectCommand),
    #[command(subcommand)]
    Request(project::RequestCommand),
    #[command(subcommand)]
    Agency(agency::AgencyCommand),
    #[command(subcommand)]
    Profile(profile::ProfileCommand),
}

#[derive(Subcommand)]
enum ServicesCommand {
    Status,
    /// Mark a hosted component (tuwunel, gitea) as consumed and start it.
    Consume {
        component: String,
    },
    Backup {
        path: PathBuf,
    },
    Restore {
        path: PathBuf,
        #[arg(long)]
        preview_token: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum ContextCommand {
    /// Assemble a preview by object ID (project + room) without saving.
    Preview {
        /// JSON file: {"project_id": "...", "room_id": "...", "budget": 65536}.
        #[arg(long, value_name = "INPUT")]
        input: PathBuf,
    },
    /// Show a frozen manifest and/or bundle by id.
    Show {
        project_id: String,
        #[arg(long, value_name = "ID")]
        manifest_id: Option<String>,
        #[arg(long, value_name = "ID")]
        bundle_id: Option<String>,
    },
}

#[derive(Subcommand)]
enum BackupCommand {
    Create { path: PathBuf },
    Verify { path: PathBuf },
}

#[derive(Subcommand)]
enum RestoreCommand {
    Preview {
        path: PathBuf,
    },
    Apply {
        path: PathBuf,
        #[arg(long)]
        preview_token: Option<String>,
        #[arg(long)]
        yes: bool,
    },
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

async fn context_cli(command: ContextCommand, root: &Path, as_json: bool) -> Result<(), String> {
    match command {
        ContextCommand::Preview { input } => {
            let body = std::fs::read(&input).map_err(io)?;
            let value: Value =
                serde_json::from_slice(&body).map_err(|e| format!("{}: {e}", input.display()))?;
            task::query_task(root, as_json, "context.preview", value).await
        }
        ContextCommand::Show {
            project_id,
            manifest_id,
            bundle_id,
        } => task::query_task(
            root,
            as_json,
            "context.show",
            json!({"project_id": project_id, "manifest_id": manifest_id, "bundle_id": bundle_id}),
        )
        .await,
    }
}

async fn dispatch(command: Command, root: &Path, json: bool) -> Result<(), String> {
    match command {
        Command::Agency(command) => agency::dispatch(command, root, json).await,
        Command::Profile(command) => profile::dispatch(command, root, json).await,
        Command::Context { command } => context_cli(command, root, json).await,
        Command::Repo(command) => repo::dispatch(command, root, json).await,
        Command::Task(command) => task::dispatch(command, root, json).await,
        Command::Room(command) => room::dispatch(command, root, json).await,
        Command::Project(command) => project::project(command, root, json).await,
        Command::Request(command) => project::request(command, root, json).await,
        Command::Init { secret_backend } => {
            std::fs::create_dir_all(root).map_err(io)?;
            if let Some(backend) = secret_backend.as_deref() {
                write_secret_backend(root, backend)?;
            }
            print_out(
                json,
                json!({"root": root.display().to_string(), "initialized": true}),
            );
            Ok(())
        }
        Command::Start { secret_backend } => {
            if let Some(backend) = secret_backend.as_deref() {
                write_secret_backend(root, backend)?;
            }
            start_daemon(root).await
        }
        Command::Stop => stop_daemon(root).await,
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
        Command::Restore(RestoreCommand::Apply {
            path,
            preview_token,
            yes,
        }) => {
            let payload = json!({"path": path});
            let token = if let Some(token) = preview_token {
                token
            } else if yes {
                let preview = preview(root, "restore.apply", payload.clone()).await?;
                preview
                    .get("preview_token")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "preview missing token".to_owned())?
                    .to_owned()
            } else {
                return Err(
                    "restore apply requires --preview-token from restore preview, or --yes".into(),
                );
            };
            let result = submit(root, "restore.apply", payload, Some(&token)).await?;
            print_out(json, result);
            Ok(())
        }
        Command::Query { kind } => query(root, json, &kind, json!({})).await,
        Command::Services(ServicesCommand::Status) => {
            query(root, json, "services", json!({})).await
        }
        Command::Services(ServicesCommand::Consume { component }) => {
            let result = submit(
                root,
                "services.consume",
                json!({"component": component}),
                None,
            )
            .await?;
            print_out(json, result);
            Ok(())
        }
        Command::Services(ServicesCommand::Backup { path }) => {
            let result = submit(root, "services.backup", json!({"path": path}), None).await?;
            print_out(json, result);
            Ok(())
        }
        Command::Services(ServicesCommand::Restore {
            path,
            preview_token,
            yes,
        }) => {
            let payload = json!({"path": path});
            let token = if let Some(token) = preview_token {
                token
            } else if yes {
                let preview = preview(root, "services.restore", payload.clone()).await?;
                preview
                    .get("preview_token")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "preview missing token".to_owned())?
                    .to_owned()
            } else {
                return Err(
                    "services restore requires --preview-token from preview, or --yes".into(),
                );
            };
            let result = submit(root, "services.restore", payload, Some(&token)).await?;
            print_out(json, result);
            Ok(())
        }
    }
}

/// Records the secret backend in `<root>/control.json`, the file control reads.
/// Unknown keys are preserved so the file can grow beyond this setting.
fn write_secret_backend(root: &Path, backend: &str) -> Result<(), String> {
    // `start` creates the root itself, so recording a setting cannot assume it exists.
    std::fs::create_dir_all(root).map_err(io)?;
    let path = root.join("control.json");
    let mut config = match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice::<Value>(&bytes)
            .map_err(|error| format!("{}: {error}", path.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => json!({}),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    let object = config
        .as_object_mut()
        .ok_or_else(|| format!("{}: configuration must be a JSON object", path.display()))?;
    object.insert("secret_backend".to_owned(), json!(backend));
    let bytes = serde_json::to_vec_pretty(&config).map_err(|error| error.to_string())?;
    // Written the way the secret store writes its own file: temporary file, synced,
    // then renamed — an interrupted write cannot leave a half-parsed configuration.
    let temporary = root.join(format!("control.json.tmp.{}", std::process::id()));
    let mut file = std::fs::File::create(&temporary).map_err(io)?;
    std::io::Write::write_all(&mut file, &bytes).map_err(io)?;
    file.sync_all().map_err(io)?;
    drop(file);
    std::fs::rename(&temporary, &path).map_err(io)
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
    let mut command = std::process::Command::new(&bin);
    command.arg("--root").arg(root);
    if let Some(install) = std::env::var_os("HCTL2_INSTALL_ROOT")
        .map(PathBuf::from)
        .or_else(install_root_from_exe)
    {
        command.env("HCTL2_INSTALL_ROOT", install);
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(io)?;
    for _ in 0..2_400 {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("control exited before ready: {status}"));
        }
        match query_raw(root, "status", json!({})).await {
            Ok(_) => return Ok(()),
            Err(error) => match rpc_code(&error) {
                Some("STORE_NOT_READY" | "UPGRADE_IN_PROGRESS") | None => {}
                Some(_) => return Err(error),
            },
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Err("control did not become ready".into())
}

async fn stop_daemon(root: &Path) -> Result<(), String> {
    match submit(root, "services.stop", json!({}), None).await {
        Ok(_) => {}
        Err(error) => eprintln!("hctl2: services.stop: {error}"),
    }
    if let Ok(pid) = std::fs::read_to_string(root.join("control.pid")) {
        let _ = std::process::Command::new("kill").arg(pid.trim()).status();
    }
    Ok(())
}

fn rpc_code(error: &str) -> Option<&str> {
    let code = error.split(':').next()?;
    if code.chars().all(|c| c.is_ascii_uppercase() || c == '_') && code.len() > 2 {
        Some(code)
    } else {
        None
    }
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
            command_id: invocation_id("preview"),
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
    let id = invocation_id(operation);
    let response = client
        .submit(SubmitRequest {
            protocol: Some(Protocol {
                version: PROTOCOL.into(),
            }),
            operation: operation.into(),
            payload: payload.to_string().into_bytes(),
            command_id: id.clone(),
            idempotency_key: id,
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

fn install_root_from_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let resolved = std::fs::canonicalize(&exe).unwrap_or_else(|_| exe.clone());
    for candidate in [&resolved, &exe] {
        let bin = candidate.parent()?;
        let parent = bin.parent()?;
        let payload_pc = parent.join("libexec/hctl2/process-compose");
        if payload_pc.is_file() {
            return Some(parent.to_path_buf());
        }
        let lib = parent.join("lib/hctl2");
        if let Ok(entries) = std::fs::read_dir(lib) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join("libexec/hctl2/process-compose").is_file() {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn invocation_id(kind: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("cli-{kind}-{}-{nanos}", std::process::id())
}

fn io(error: std::io::Error) -> String {
    error.to_string()
}

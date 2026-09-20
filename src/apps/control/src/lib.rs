//! Local Unix-socket control daemon and Repo registration command entry point.

#![forbid(unsafe_code)]

mod identity;
mod repositories;
mod service;
mod services;
mod socket;

pub use identity::owner_actor;
pub use service::{ControlService, PROTOCOL};
pub use services::Supervisor;
pub use socket::{bind_owner_socket, occupied_error, socket_path};

use std::path::{Path, PathBuf};
use std::sync::Arc;

use store::{StartupStatus, Store, StoreError};
use tokio::net::UnixListener;
use tokio::sync::Mutex;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;

/// Shared daemon state. `Store` is not `Sync`; RPC tasks share it through a mutex.
pub struct Daemon {
    pub root: PathBuf,
    pub socket: PathBuf,
    pub status: StartupStatus,
    pub store: Arc<Mutex<Option<Store>>>,
    pub open_error: Arc<Mutex<Option<StoreError>>>,
}

impl Daemon {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        let socket = socket_path(&root);
        Self {
            root,
            socket,
            status: StartupStatus::default(),
            store: Arc::new(Mutex::new(None)),
            open_error: Arc::new(Mutex::new(None)),
        }
    }

    /// Bind the owner-only socket, start serving, then open storage on a worker.
    ///
    /// Open-store failure keeps the process serving so `status` / `doctor` can
    /// present the typed store error. The process does not exit on that path.
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = bind_owner_socket(&self.socket)?;
        let pid_path = self.root.join("control.pid");
        std::fs::write(&pid_path, std::process::id().to_string())?;
        let _pid = PidFile(pid_path);
        let status = self.status.clone();
        let store = Arc::clone(&self.store);
        let open_error = Arc::clone(&self.open_error);
        let root = self.root.clone();
        let hosted = Arc::new(Supervisor::from_root(root.clone()));
        let hosted_open = Arc::clone(&hosted);
        tokio::task::spawn_blocking(move || match Store::open_with_status(&root, status) {
            Ok(opened) => {
                *store.blocking_lock() = Some(opened);
                if let Err(error) = hosted_open.ensure_up() {
                    hosted_open.set_last_error(error);
                }
            }
            Err(error) => {
                hosted_open.set_last_error(format!(
                    "store not ready ({}); services not started",
                    error.code
                ));
                *open_error.blocking_lock() = Some(error);
            }
        });
        let service = ControlService::with_services(
            self.root.clone(),
            self.status.clone(),
            Arc::clone(&self.store),
            Arc::clone(&self.open_error),
            hosted,
        );
        serve_listener(listener, service).await
    }
}

struct PidFile(PathBuf);
impl Drop for PidFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Serve the control contract on an already bound owner-only listener.
pub async fn serve_listener(
    listener: UnixListener,
    service: ControlService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Server::builder()
        .add_service(proto::control_server::ControlServer::new(service))
        .serve_with_incoming(UnixListenerStream::new(listener))
        .await?;
    Ok(())
}

#[must_use]
pub fn default_root() -> PathBuf {
    std::env::var_os("HCTL2_ROOT").map_or_else(dirs_fallback, PathBuf::from)
}

fn dirs_fallback() -> PathBuf {
    match std::env::var_os("HOME") {
        Some(home) => Path::new(&home).join(".hctl2").join("control"),
        None => PathBuf::from(".hctl2/control"),
    }
}

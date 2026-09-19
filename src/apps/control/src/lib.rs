//! Local Unix-socket control daemon. Domain commands stay in later packages.

#![forbid(unsafe_code)]

mod identity;
mod service;
mod socket;

pub use identity::owner_actor;
pub use service::{ControlService, PROTOCOL};
pub use socket::{bind_owner_socket, occupied_error, socket_path};

use std::path::{Path, PathBuf};
use std::sync::Arc;

use store::{StartupStatus, Store};
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
        }
    }

    /// Bind the owner-only socket, start serving, then open storage on a worker.
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = bind_owner_socket(&self.socket)?;
        let status = self.status.clone();
        let store = Arc::clone(&self.store);
        let root = self.root.clone();
        tokio::task::spawn_blocking(move || match Store::open_with_status(&root, status) {
            Ok(opened) => {
                *store.blocking_lock() = Some(opened);
            }
            Err(error) => {
                eprintln!("hctl2-control: failed to open storage: {error}");
            }
        });
        let service = ControlService::new(
            self.root.clone(),
            self.status.clone(),
            Arc::clone(&self.store),
        );
        serve_listener(listener, service).await
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

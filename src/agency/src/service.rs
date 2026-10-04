use crate::{
    runtime::Runtime,
    storage::{database, nonce, private_dir, private_file, sql},
    tenant::Tenant,
};
use agency_proto::{
    wire::{
        self,
        agency_server::{Agency as RpcAgency, AgencyServer},
    },
    *,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::de::DeserializeOwned;
use std::{
    collections::HashMap,
    fs::File,
    os::unix::fs::{FileTypeExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::sync::watch;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{Request, Response, Status};

pub struct Agency {
    root: PathBuf,
    key: String,
    pair_key: String,
    registry: Mutex<Connection>,
    tenants: Mutex<HashMap<String, Arc<Tenant>>>,
    runtime: Arc<dyn Runtime>,
    shutdown: watch::Sender<bool>,
    _writer: File,
}
impl Agency {
    pub fn save_script_config(&self, config: &crate::runtime::ScriptConfig) -> Result<()> {
        let db = self.registry.lock().expect("registry mutex");
        sql(db.execute("INSERT INTO settings VALUES('runtime',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[serde_json::to_string(config)?]))?;
        Ok(())
    }
    pub fn script_config(root: &Path) -> Result<Option<crate::runtime::ScriptConfig>> {
        if !root.join("registry.sqlite").exists() {
            return Ok(None);
        }
        let db = sql(Connection::open_with_flags(
            root.join("registry.sqlite"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ))?;
        let config: Option<String> = sql(db
            .query_row("SELECT value FROM settings WHERE key='runtime'", [], |r| {
                r.get(0)
            })
            .optional())?;
        Ok(config.map(|s| serde_json::from_str(&s)).transpose()?)
    }
    pub fn open(root: &Path, runtime: Arc<dyn Runtime>) -> Result<Arc<Self>> {
        private_dir(root)?;
        let root = root.canonicalize()?;
        let writer = private_file(&root.join("agency.lock"))?;
        writer.try_lock().map_err(|_| {
            PortError::new(
                "AGENCY_WRITER_BUSY",
                "another Agency owns this directory",
                "use_existing_agency",
            )
        })?;
        let db = database(&root.join("registry.sqlite"))?;
        sql(db.execute_batch("CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY,value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS tenants(control_id TEXT PRIMARY KEY,id TEXT UNIQUE NOT NULL,key TEXT NOT NULL); PRAGMA user_version=1;"))?;
        let key: Option<String> = sql(db
            .query_row(
                "SELECT value FROM settings WHERE key='bootstrap'",
                [],
                |r| r.get(0),
            )
            .optional())?;
        let key = if let Some(key) = key {
            key
        } else {
            let key = nonce()?;
            sql(db.execute("INSERT INTO settings VALUES('bootstrap',?1)", [&key]))?;
            key
        };
        let pair_key: Option<String> = sql(db
            .query_row("SELECT value FROM settings WHERE key='pairing'", [], |r| {
                r.get(0)
            })
            .optional())?;
        let pair_key = if let Some(key) = pair_key {
            key
        } else {
            let key = nonce()?;
            sql(db.execute("INSERT INTO settings VALUES('pairing',?1)", [&key]))?;
            key
        };
        let mut rows = sql(db.prepare("SELECT id,key FROM tenants"))?;
        let entries =
            sql(rows.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))))?
                .collect::<rusqlite::Result<Vec<_>>>();
        let entries = sql(entries)?;
        drop(rows);
        let mut tenants = HashMap::new();
        for (id, key) in entries {
            let tenant = Tenant::open(root.join("tenants").join(&id), key, Arc::clone(&runtime))?;
            tenants.insert(id, tenant);
        }
        let mut credential = private_file(&root.join("pair.key"))?;
        use std::io::Write;
        credential.set_len(0)?;
        credential.write_all(pair_key.as_bytes())?;
        credential.sync_all()?;
        let (shutdown, _) = watch::channel(false);
        Ok(Arc::new(Self {
            root,
            key,
            pair_key,
            registry: Mutex::new(db),
            tenants: Mutex::new(tenants),
            runtime,
            shutdown,
            _writer: writer,
        }))
    }
    pub fn bootstrap_key(root: &Path) -> Result<String> {
        // Service-owner credential: never handed to a control client for pairing.
        let db = sql(Connection::open_with_flags(
            root.join("registry.sqlite"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ))?;
        sql(db.query_row(
            "SELECT value FROM settings WHERE key='bootstrap'",
            [],
            |r| r.get(0),
        ))
    }
    fn pair(self: &Arc<Self>, input: Pair) -> Result<Pairing> {
        nonempty(&input.control_id)?;
        digest(&input.tenant_key)?;
        let mut db = self.registry.lock().expect("registry mutex");
        let tx = sql(db.transaction())?;
        let found: Option<(String, String)> = sql(tx
            .query_row(
                "SELECT id,key FROM tenants WHERE control_id=?1",
                [&input.control_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional())?;
        let (id, key, new) = if let Some((id, key)) = found {
            if !credential_matches(&input.tenant_key, &key) {
                return Err(PortError::new(
                    "TENANT_EXISTS",
                    "existing tenant requires its current credential",
                    "recover_original_pairing_credential",
                ));
            }
            (id, key, false)
        } else {
            (nonce()?, input.tenant_key, true)
        };
        let root = self.root.join("tenants").join(&id);
        let endpoint =
            agency_proto::client::socket_directory(&self.root)?.join(format!("{}.sock", &id[..20]));
        if new {
            let tenant = Tenant::open(root.clone(), key.clone(), Arc::clone(&self.runtime))?;
            // Bind before publishing the pairing; failure cannot return an unusable endpoint.
            let listener = bind(&endpoint)?;
            sql(tx.execute(
                "INSERT INTO tenants VALUES(?1,?2,?3)",
                params![input.control_id, id, key],
            ))?;
            sql(tx.commit())?;
            self.tenants
                .lock()
                .expect("tenants mutex")
                .insert(id, Arc::clone(&tenant));
            spawn_server(
                listener,
                Rpc {
                    agency: Arc::clone(self),
                    tenant: Some(tenant),
                },
            );
        } else {
            sql(tx.commit())?;
        }
        Ok(Pairing {
            endpoint: endpoint.to_string_lossy().into_owned(),
            key,
        })
    }
}
#[derive(Clone)]
struct Rpc {
    agency: Arc<Agency>,
    tenant: Option<Arc<Tenant>>,
}
impl Rpc {
    fn authorize(&self, request: &Request<wire::Call>, method: &str) -> Result<()> {
        let supplied = request
            .metadata()
            .get("x-agency-key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let authorized = match &self.tenant {
            Some(tenant) => credential_matches(supplied, &tenant.key),
            None if method == "pair" => credential_matches(supplied, &self.agency.pair_key),
            None if method == "catalog" => {
                credential_matches(supplied, &self.agency.pair_key)
                    || credential_matches(supplied, &self.agency.key)
            }
            None => credential_matches(supplied, &self.agency.key),
        };
        if !authorized {
            return Err(PortError::new(
                "PAIRING_REQUIRED",
                "endpoint credential required",
                "pair_control",
            ));
        }
        if request.get_ref().protocol != PROTOCOL {
            return Err(PortError::new(
                "PROTOCOL_MISMATCH",
                "unsupported Agency protocol",
                "use_compatible_client",
            ));
        }
        if request.get_ref().document.len() > MAX_DOCUMENT {
            return Err(PortError::invalid("document too large"));
        }
        Ok(())
    }
    async fn call(
        &self,
        request: Request<wire::Call>,
        method: &'static str,
    ) -> std::result::Result<Response<wire::Reply>, Status> {
        if let Err(e) = self.authorize(&request, method) {
            return Ok(response(Err(e)));
        }
        let rpc = self.clone();
        let bytes = request.into_inner().document;
        let result = tokio::task::spawn_blocking(move || rpc.apply(method, &bytes))
            .await
            .map_err(|_| Status::internal("Agency worker failed"))?;
        Ok(response(result))
    }
    fn apply(&self, method: &str, bytes: &[u8]) -> Result<Vec<u8>> {
        if method == "pair" && self.tenant.is_none() {
            return canonical(&self.agency.pair(read(bytes)?)?);
        }
        if method == "shutdown" && self.tenant.is_none() {
            for tenant in self.agency.tenants.lock().expect("tenants mutex").values() {
                let state = tenant.state.lock().expect("tenant mutex");
                for session in state.sessions.values() {
                    session.lock().expect("session mutex").stop()?;
                }
            }
            let _ = self.agency.shutdown.send(true);
            return canonical(&serde_json::json!({"stop_requested":true}));
        }
        if method == "catalog" && self.tenant.is_none() {
            return canonical(&serde_json::json!({"ready":true}));
        }
        let tenant = self.tenant.as_ref().ok_or_else(|| {
            PortError::new(
                "TENANT_REQUIRED",
                "use the paired tenant endpoint",
                "pair_control",
            )
        })?;
        match method {
            "catalog" => canonical(&tenant.runtime.catalog()?),
            "prepare" => canonical(&tenant.prepare(read(bytes)?)?),
            "lookup" => canonical(&tenant.lookup(read(bytes)?)?),
            "activate" => canonical(&tenant.activate(read(bytes)?)?),
            "fence" => canonical(&tenant.fence(read(bytes)?)?),
            "lease" => canonical(&tenant.lease(read(bytes)?)?),
            "input" => canonical(&tenant.input(read(bytes)?)?),
            "stop" => canonical(&tenant.stop(read(bytes)?)?),
            "observe" => canonical(&tenant.observe(read(bytes)?)?),
            "results" => canonical(&tenant.results(read(bytes)?)?),
            "preserve" => canonical(&tenant.preserve(read(bytes)?)?),
            _ => Err(PortError::new(
                "METHOD_DENIED",
                "method unavailable on this endpoint",
                "use_correct_endpoint",
            )),
        }
    }
}
fn read<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    Ok(serde_json::from_slice(bytes)?)
}
fn response(result: Result<Vec<u8>>) -> Response<wire::Reply> {
    Response::new(match result {
        Ok(document) => wire::Reply {
            document,
            error: None,
        },
        Err(e) => wire::Reply {
            document: vec![],
            error: Some(wire::Error {
                code: e.code,
                message: e.message,
                recovery_action: e.recovery_action,
            }),
        },
    })
}
macro_rules! methods { ($($name:ident),*) => { #[tonic::async_trait] impl RpcAgency for Rpc { $(async fn $name(&self,r:Request<wire::Call>)->std::result::Result<Response<wire::Reply>,Status> { self.call(r,stringify!($name)).await })* } }; }
methods!(
    pair, catalog, prepare, lookup, activate, fence, lease, input, stop, observe, results,
    preserve, shutdown
);

fn bind(path: &Path) -> Result<tokio::net::UnixListener> {
    agency_proto::client::validate_socket_directory(
        path.parent()
            .ok_or_else(|| PortError::invalid("socket parent"))?,
    )?;
    if path.exists() {
        agency_proto::client::validate_socket(path)?;
        if !std::fs::symlink_metadata(path)?.file_type().is_socket() {
            return Err(PortError::invalid("endpoint path is not a socket"));
        }
        if std::os::unix::net::UnixStream::connect(path).is_ok() {
            return Err(PortError::new(
                "ENDPOINT_BUSY",
                "Agency endpoint already serves",
                "use_existing_agency",
            ));
        }
        std::fs::remove_file(path)?;
    }
    let listener = tokio::net::UnixListener::bind(path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}
fn spawn_server(listener: tokio::net::UnixListener, rpc: Rpc) {
    let mut shutdown = rpc.agency.shutdown.subscribe();
    tokio::spawn(async move {
        let _ = tonic::transport::Server::builder()
            .add_service(AgencyServer::new(rpc).max_decoding_message_size(MAX_DOCUMENT))
            .serve_with_incoming_shutdown(UnixListenerStream::new(listener), async move {
                let _ = shutdown.changed().await;
            })
            .await;
    });
}
pub async fn serve(agency: Arc<Agency>) -> Result<()> {
    let sockets = agency_proto::client::socket_directory(&agency.root)?;
    use std::os::unix::fs::DirBuilderExt;
    match std::fs::DirBuilder::new().mode(0o700).create(&sockets) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.into()),
    }
    agency_proto::client::validate_socket_directory(&sockets)?;
    let main = bind(&sockets.join("admin.sock"))?;
    let tenants = agency
        .tenants
        .lock()
        .expect("tenants mutex")
        .iter()
        .map(|(id, t)| (id.clone(), Arc::clone(t)))
        .collect::<Vec<_>>();
    for (id, tenant) in tenants {
        let listener = bind(&sockets.join(format!("{}.sock", &id[..20])))?;
        spawn_server(
            listener,
            Rpc {
                agency: Arc::clone(&agency),
                tenant: Some(tenant),
            },
        );
    }
    let mut shutdown = agency.shutdown.subscribe();
    let result = tonic::transport::Server::builder()
        .add_service(
            AgencyServer::new(Rpc {
                agency: Arc::clone(&agency),
                tenant: None,
            })
            .max_decoding_message_size(MAX_DOCUMENT),
        )
        .serve_with_incoming_shutdown(UnixListenerStream::new(main), async move {
            let _ = shutdown.changed().await;
        })
        .await
        .map_err(|e| PortError::new("AGENCY_TRANSPORT", e.to_string(), "restart_agency"));
    if result.is_ok() {
        let mut paths = vec![sockets.join("admin.sock")];
        paths.extend(
            agency
                .tenants
                .lock()
                .expect("tenants mutex")
                .keys()
                .map(|id| sockets.join(format!("{}.sock", &id[..20]))),
        );
        for path in paths {
            agency_proto::client::validate_socket(&path)?;
            std::fs::remove_file(path)?;
        }
        std::fs::remove_dir(&sockets)?;
    }
    result
}

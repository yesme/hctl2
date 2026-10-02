use crate::services::Supervisor;
use chat::{Result, Server, invalid, key, reject};
use ruma::api::appservice::{Namespace, Namespaces, Registration, RegistrationInit};
use std::{
    fs,
    io::Write,
    net::TcpListener,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};
use store::{Reference, Scope, Version};

pub(super) fn path(services: &Supervisor) -> Result<PathBuf> {
    let (_, state) = services.packaged_paths().ok_or_else(|| {
        reject(
            "CHAT_NOT_INSTALLED",
            "packaged Tuwunel unavailable",
            "install_package",
        )
    })?;
    Ok(state.join("config/appservices/hctl2.yaml"))
}

/// Write native registration before Tuwunel starts. JSON is a YAML subset; no second parser.
pub(super) fn prepare(
    services: &Supervisor,
    control_id: &str,
) -> Result<(Registration, TcpListener)> {
    let path = path(services)?;
    let (registration, listener) = if path.exists() {
        let registration: Registration = serde_json::from_slice(&fs::read(&path)?)?;
        if registration.id != format!("hctl2-{control_id}") {
            return Err(invalid("AppService belongs to another control"));
        }
        let port = reqwest::Url::parse(
            registration
                .url
                .as_deref()
                .ok_or_else(|| invalid("AppService callback required"))?,
        )
        .ok()
        .filter(|u| u.host_str() == Some("127.0.0.1") && u.scheme() == "http")
        .and_then(|u| u.port())
        .ok_or_else(|| invalid("invalid AppService callback"))?;
        let listener = TcpListener::bind(("127.0.0.1", port))?;
        (registration, listener)
    } else {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let mut namespaces = Namespaces::new();
        namespaces
            .users
            .push(Namespace::new(true, "^@hctl2_.*:hctl2\\.localhost$".into()));
        namespaces
            .aliases
            .push(Namespace::new(true, "^#hctl2_.*:hctl2\\.localhost$".into()));
        let registration = Registration::from(RegistrationInit {
            id: format!("hctl2-{control_id}"),
            url: Some(format!(
                "http://127.0.0.1:{}",
                listener.local_addr()?.port()
            )),
            as_token: format!(
                "{}{}",
                ruma::TransactionId::new(),
                ruma::TransactionId::new()
            ),
            hs_token: format!(
                "{}{}",
                ruma::TransactionId::new(),
                ruma::TransactionId::new()
            ),
            sender_localpart: "hctl2_control".into(),
            namespaces,
            rate_limited: Some(false),
            protocols: None,
        });
        private_write(&path, &serde_json::to_vec_pretty(&registration)?)?;
        (registration, listener)
    };
    listener.set_nonblocking(true)?;
    Ok((registration, listener))
}

pub(super) fn load(services: &Supervisor) -> Result<(Server, String)> {
    let path = path(services)?;
    let bytes = fs::read(&path).map_err(|_| {
        reject(
            "CHAT_NOT_CONFIGURED",
            "AppService registration unavailable",
            "restart_control",
        )
    })?;
    let registration: Registration = serde_json::from_slice(&bytes)?;
    let config_dir = path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| invalid("invalid deployment path"))?;
    if fs::read_to_string(config_dir.join("appservice-loaded"))
        .ok()
        .as_deref()
        != Some(&foundation::bytes_sha256(&bytes))
    {
        return Err(reject(
            "CHAT_NOT_READY",
            "AppService registration is not loaded yet",
            "retry_after_service_ready",
        ));
    }
    let config = fs::read_to_string(
        path.parent()
            .and_then(Path::parent)
            .ok_or_else(|| invalid("invalid deployment path"))?
            .join("tuwunel.toml"),
    )?;
    let port: u16 = config
        .lines()
        .find_map(|line| line.strip_prefix("port = "))
        .and_then(|p| p.parse().ok())
        .ok_or_else(|| invalid("Tuwunel port missing"))?;
    Ok((
        Server {
            binding: Reference {
                key: key(Scope::Control, "chat_server", &registration.id),
                version: Version::State(1),
            },
            url: format!("http://127.0.0.1:{port}"),
            server_name: "hctl2.localhost".into(),
            sender: format!("@{}:hctl2.localhost", registration.sender_localpart),
        },
        registration.as_token,
    ))
}

pub(crate) fn private_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid("private file has no parent"))?;
    fs::create_dir_all(parent)?;
    fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
    let tmp = parent.join(format!(".pending-{}", ruma::TransactionId::new()));
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(&tmp, path)?;
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}

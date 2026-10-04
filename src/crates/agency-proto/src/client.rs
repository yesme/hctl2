//! Owner-only local transport, with pairing credentials never in domain records.
use crate::{PROTOCOL, PortError, Result, canonical, wire};
use hyper_util::rt::TokioIo;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::Read,
    os::unix::fs::{FileTypeExt, MetadataExt},
    path::{Path, PathBuf},
};
use tonic::transport::{Channel, Endpoint};
use tower::service_fn;

/// Short owner-only socket directory; macOS UDS paths have a small fixed limit.
pub fn socket_directory(root: &std::path::Path) -> Result<PathBuf> {
    let root = root.canonicalize()?;
    Ok(PathBuf::from("/tmp").join(format!(
        "agency-{}",
        &crate::hash(root.as_os_str().as_encoded_bytes())[..20]
    )))
}
pub fn admin_endpoint(root: &std::path::Path) -> Result<PathBuf> {
    Ok(socket_directory(root)?.join("admin.sock"))
}

pub fn new_credential() -> Result<String> {
    let mut bytes = [0; 32];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(crate::hex(&bytes))
}

pub fn validate_socket_directory(path: &Path) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o7777 != 0o700
    {
        return Err(PortError::new(
            "UNSAFE_ENDPOINT",
            "socket directory is not owner-only",
            "check_agency_directory",
        ));
    }
    Ok(())
}
pub fn validate_socket(path: &Path) -> Result<()> {
    validate_socket_directory(
        path.parent()
            .ok_or_else(|| PortError::invalid("socket parent"))?,
    )?;
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.file_type().is_socket()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o7777 != 0o600
    {
        return Err(PortError::new(
            "UNSAFE_ENDPOINT",
            "socket is not owner-only",
            "check_agency_directory",
        ));
    }
    Ok(())
}

/// A decoded Reply.error is a response; adapters still classify whether it proves no effect.
#[derive(Debug)]
pub enum CallFailure {
    ResponseError(PortError),
    NoReply(PortError),
}
impl CallFailure {
    pub fn into_error(self) -> PortError {
        match self {
            Self::ResponseError(error) | Self::NoReply(error) => error,
        }
    }
}
impl From<PortError> for CallFailure {
    fn from(error: PortError) -> Self {
        Self::NoReply(error)
    }
}

#[derive(Clone)]
pub struct Client {
    endpoint: PathBuf,
    key: String,
}
impl Client {
    pub fn new(endpoint: PathBuf, key: String) -> Self {
        Self { endpoint, key }
    }
    pub async fn call<I: Serialize, O: DeserializeOwned>(
        &self,
        method: &str,
        document: &I,
    ) -> Result<O> {
        self.call_outcome(method, document)
            .await
            .map_err(CallFailure::into_error)
    }
    pub async fn call_outcome<I: Serialize, O: DeserializeOwned>(
        &self,
        method: &str,
        document: &I,
    ) -> std::result::Result<O, CallFailure> {
        validate_socket(&self.endpoint).map_err(|error| {
            if error.code == "IO_ERROR" {
                PortError::new(
                    "AGENCY_UNREACHABLE",
                    "paired Agency endpoint cannot be reached",
                    "check_agency",
                )
            } else {
                error
            }
        })?;
        let path = self.endpoint.clone();
        let channel: Channel = Endpoint::try_from("http://[::]:50051")
            .expect("fixed URI")
            .timeout(std::time::Duration::from_secs(5))
            .connect_timeout(std::time::Duration::from_secs(2))
            .connect_with_connector(service_fn(move |_| {
                let path = path.clone();
                async move {
                    let stream = tokio::net::UnixStream::connect(path).await?;
                    if stream.peer_cred()?.uid() != rustix::process::geteuid().as_raw() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            "Agency peer uid differs",
                        ));
                    }
                    Ok::<_, std::io::Error>(TokioIo::new(stream))
                }
            }))
            .await
            .map_err(|_| {
                PortError::new(
                    "AGENCY_UNREACHABLE",
                    "paired Agency endpoint cannot be reached",
                    "check_agency",
                )
            })?;
        let mut client = wire::agency_client::AgencyClient::new(channel)
            .max_decoding_message_size(crate::MAX_DOCUMENT);
        let mut request = tonic::Request::new(wire::Call {
            protocol: PROTOCOL.into(),
            document: canonical(document)?,
        });
        request.metadata_mut().insert(
            "x-agency-key",
            self.key
                .parse()
                .map_err(|_| PortError::invalid("credential encoding"))?,
        );
        let reply = match method {
            "pair" => client.pair(request).await,
            "catalog" => client.catalog(request).await,
            "prepare" => client.prepare(request).await,
            "lookup" => client.lookup(request).await,
            "activate" => client.activate(request).await,
            "fence" => client.fence(request).await,
            "lease" => client.lease(request).await,
            "input" => client.input(request).await,
            "stop" => client.stop(request).await,
            "observe" => client.observe(request).await,
            "results" => client.results(request).await,
            "preserve" => client.preserve(request).await,
            "shutdown" => client.shutdown(request).await,
            _ => return Err(PortError::invalid("unknown Agency method").into()),
        }
        .map_err(|e| {
            PortError::new(
                "AGENCY_RESPONSE_UNKNOWN",
                e.code().to_string(),
                "readback_original_request",
            )
        })?
        .into_inner();
        if let Some(e) = reply.error {
            return Err(CallFailure::ResponseError(PortError::new(
                &e.code,
                e.message,
                &e.recovery_action,
            )));
        }
        serde_json::from_slice(&reply.document)
            .map_err(PortError::from)
            .map_err(CallFailure::NoReply)
    }
}

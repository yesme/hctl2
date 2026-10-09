//! Owner-only local transport, with pairing credentials never in domain records.
use crate::{PROTOCOL, PortError, Result, canonical, wire};
use hyper_util::rt::TokioIo;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io::Read,
    os::unix::fs::{FileTypeExt, MetadataExt},
    path::{Path, PathBuf},
    time::Duration,
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
    request_timeout: Option<Duration>,
}
impl Client {
    pub fn new(endpoint: PathBuf, key: String) -> Self {
        Self {
            endpoint,
            key,
            request_timeout: None,
        }
    }
    /// Configure tonic's per-request transport budget, not the dispatch deadline.
    /// A timeout remains NoReply and never causes this client to resend a request.
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }
    /// Explicit client budget takes precedence over the process declaration.
    pub fn request_timeout(&self) -> Result<Duration> {
        request_budget(
            self.request_timeout,
            std::env::var_os("HCTL2_AGENCY_REQUEST_TIMEOUT_MS"),
        )
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
            .timeout(self.request_timeout()?)
            .connect_timeout(Duration::from_secs(2))
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

fn request_budget(
    explicit: Option<Duration>,
    declared: Option<std::ffi::OsString>,
) -> Result<Duration> {
    let timeout = match explicit {
        Some(timeout) => timeout,
        None => match declared {
            None => Duration::from_secs(5),
            Some(value) => Duration::from_millis(
                value
                    .to_str()
                    .and_then(|value| value.parse::<u64>().ok())
                    .ok_or_else(|| PortError::invalid("HCTL2_AGENCY_REQUEST_TIMEOUT_MS"))?,
            ),
        },
    };
    if timeout.is_zero() {
        return Err(PortError::invalid("request budget must be positive"));
    }
    Ok(timeout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_budget_is_explicit_and_does_not_change_the_original_client() {
        let original = Client::new("/tmp/unused.sock".into(), "key".into());
        let configured = original
            .clone()
            .with_request_timeout(Duration::from_secs(30));
        assert_eq!(original.request_timeout, None);
        assert_eq!(configured.request_timeout, Some(Duration::from_secs(30)));
        assert_eq!(configured.endpoint, original.endpoint);
        assert_eq!(configured.key, original.key);
    }

    #[test]
    fn process_budget_is_used_when_no_client_budget_was_declared() {
        assert_eq!(request_budget(None, None).unwrap(), Duration::from_secs(5));
        assert_eq!(
            request_budget(None, Some("30000".into())).unwrap(),
            Duration::from_secs(30)
        );
        assert_eq!(
            request_budget(Some(Duration::from_millis(20)), Some("30000".into())).unwrap(),
            Duration::from_millis(20)
        );
    }

    #[test]
    fn malformed_or_zero_request_budget_is_rejected() {
        use std::os::unix::ffi::OsStringExt;
        for declared in [
            "0".into(),
            "-1".into(),
            "unknown".into(),
            "18446744073709551616".into(),
            std::ffi::OsString::from_vec(vec![255]),
        ] {
            assert_eq!(
                request_budget(None, Some(declared)).unwrap_err().code,
                "INVALID_INPUT"
            );
        }
        assert!(request_budget(Some(Duration::ZERO), None).is_err());
    }
}

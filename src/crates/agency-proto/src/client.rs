//! Owner-only local transport, with pairing credentials never in domain records.
use crate::{PROTOCOL, PortError, Result, canonical, wire};
use hyper_util::rt::TokioIo;
use serde::{Serialize, de::DeserializeOwned};
use std::path::PathBuf;
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
        let path = self.endpoint.clone();
        let channel: Channel = Endpoint::try_from("http://[::]:50051")
            .expect("fixed URI")
            .timeout(std::time::Duration::from_secs(5))
            .connect_timeout(std::time::Duration::from_secs(2))
            .connect_with_connector(service_fn(move |_| {
                let path = path.clone();
                async move {
                    tokio::net::UnixStream::connect(path)
                        .await
                        .map(TokioIo::new)
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
            _ => return Err(PortError::invalid("unknown Agency method")),
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
            return Err(PortError::new(&e.code, e.message, &e.recovery_action));
        }
        Ok(serde_json::from_slice(&reply.document)?)
    }
}

//! Private Herdr client for the locked 0.8.2 binary (protocol 20).
//! Pane and socket ids stay in this process.
use crate::confine;
use agency_proto::{
    Capabilities, Catalog, EvidenceLevel, ExecutionSpec, PortError, Profession, Result, Sealed,
    context::Bundle, hash,
};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::{Child, Stdio},
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};

pub const PROTOCOL: u32 = 20;

pub struct Client {
    socket: PathBuf,
}

impl Client {
    pub fn connect(socket: &Path) -> Result<Self> {
        UnixStream::connect(socket)?;
        Ok(Self {
            socket: socket.to_path_buf(),
        })
    }

    pub fn call(&self, method: &str, params: Value) -> Result<Value> {
        let mut stream = UnixStream::connect(&self.socket)?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        let id = format!(
            "h{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let mut line = serde_json::to_vec(&json!({
            "id": id,
            "protocol": PROTOCOL,
            "method": method,
            "params": params,
        }))?;
        line.push(b'\n');
        stream.write_all(&line)?;
        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;
        let value: Value = serde_json::from_str(response.trim()).map_err(|_| {
            PortError::new("HERDR_PROTOCOL", "response is not JSON", "retry_herdr_call")
        })?;
        if value.get("id").and_then(Value::as_str) != Some(id.as_str()) {
            return Err(PortError::new(
                "HERDR_PROTOCOL",
                "response id does not match the request",
                "retry_herdr_call",
            ));
        }
        if let Some(error) = value.get("error") {
            return Err(PortError::new(
                "HERDR_REJECTED",
                error.to_string(),
                "read_herdr_error",
            ));
        }
        value.get("result").cloned().ok_or_else(|| {
            PortError::new(
                "HERDR_PROTOCOL",
                "response has no result",
                "retry_herdr_call",
            )
        })
    }

    pub fn ping(&self) -> Result<Value> {
        let result = self.call("ping", json!({}))?;
        let protocol = result.get("protocol").and_then(Value::as_u64).unwrap_or(0);
        if protocol != u64::from(PROTOCOL)
            || result.get("type").and_then(Value::as_str) != Some("pong")
        {
            return Err(PortError::new(
                "HERDR_PROTOCOL",
                format!("Herdr protocol {protocol} is not locked protocol {PROTOCOL}"),
                "use_locked_herdr",
            ));
        }
        Ok(result)
    }
}

pub struct Server {
    child: Child,
    pub socket: PathBuf,
    pub state: PathBuf,
}

impl Server {
    pub fn start(
        binary: &Path,
        state: &Path,
        credential_root: &Path,
        exec_parent: &Path,
    ) -> Result<Self> {
        let binary = binary.canonicalize().map_err(|_| {
            PortError::new(
                "HERDR_BINARY_MISSING",
                format!("locked Herdr binary is not at {}", binary.display()),
                "use_locked_herdr",
            )
        })?;
        crate::storage::private_dir(state)?;
        let socket_dir = PathBuf::from("/tmp").join(format!(
            "hctl2-herdr-{}",
            &hash(state.as_os_str().as_encoded_bytes())[..20]
        ));
        crate::storage::private_dir(&socket_dir)?;
        let socket = socket_dir.join("herdr.sock");
        let _ = std::fs::remove_file(&socket);
        let mut permissions = std::fs::metadata(&binary)?.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&binary, permissions)?;
        let mut child = confine::command(&binary, &["server".into()], state, credential_root)?;
        confine::scrub(&mut child, state);
        child
            .env("HERDR_SOCKET_PATH", &socket)
            .env("HERDR_CONFIG_PATH", state.join("config.toml"))
            .env("XDG_CONFIG_HOME", state.join("config"))
            .env("XDG_STATE_HOME", state.join("state"))
            .env(
                "HCTL2_CONFINE_ALLOW",
                format!(
                    "{}\n{}\n{}\n{}",
                    state.display(),
                    socket_dir.display(),
                    binary.parent().unwrap_or(binary.as_path()).display(),
                    exec_parent.display()
                ),
            )
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(std::fs::File::create(state.join("server.err"))?);
        let mut child = child.spawn()?;
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            if socket.exists()
                && Client::connect(&socket)
                    .and_then(|client| client.ping())
                    .is_ok()
            {
                return Ok(Self {
                    child,
                    socket,
                    state: state.to_path_buf(),
                });
            }
            if child.try_wait()?.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = child.kill();
        let _ = child.wait();
        let detail = std::fs::read_to_string(state.join("server.err")).unwrap_or_default();
        Err(PortError::new(
            "HERDR_NOT_READY",
            format!("locked Herdr server did not answer protocol 20: {detail}"),
            "use_locked_herdr",
        ))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct SharedServer {
    binary: PathBuf,
    credential_root: PathBuf,
    server: Mutex<Option<Arc<Server>>>,
}

impl SharedServer {
    pub fn new(binary: PathBuf, credential_root: PathBuf) -> Self {
        Self {
            binary,
            credential_root,
            server: Mutex::new(None),
        }
    }

    pub fn get(&self, state: &Path, exec_parent: &Path) -> Result<Arc<Server>> {
        let mut slot = self.server.lock().expect("herdr server");
        if let Some(server) = slot.as_ref()
            && Client::connect(&server.socket)
                .and_then(|client| client.ping())
                .is_ok()
        {
            return Ok(Arc::clone(server));
        }
        let server = Arc::new(Server::start(
            &self.binary,
            state,
            &self.credential_root,
            exec_parent,
        )?);
        *slot = Some(Arc::clone(&server));
        Ok(server)
    }
}

/// Run a deterministic command in a new workspace pane and return the pane text
/// that contains `marker`.
pub fn observe_once(seen: &mut HashSet<String>, text: &str) -> bool {
    seen.insert(text.to_owned())
}

pub fn run_command(
    client: &Client,
    cwd: &Path,
    label: &str,
    command: &str,
    marker: &str,
) -> Result<String> {
    let created = client.call(
        "workspace.create",
        json!({"cwd": cwd, "label": label, "focus": false}),
    )?;
    let pane = created
        .pointer("/root_pane/pane_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            PortError::new(
                "HERDR_PROTOCOL",
                "workspace has no pane",
                "retry_herdr_call",
            )
        })?;
    client.call(
        "pane.send_text",
        json!({"pane_id": pane, "text": format!("{command}\n")}),
    )?;
    let matched = client.call(
        "pane.wait_for_output",
        json!({
            "pane_id": pane,
            "source": "recent",
            "match": {"type": "substring", "value": marker},
            "timeout_ms": 8000
        }),
    )?;
    let text = matched
        .pointer("/read/text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    if !text.contains(marker) {
        return Err(PortError::new(
            "HERDR_OUTPUT_MISSING",
            "pane output did not contain the marker",
            "read_pane",
        ));
    }
    client.call("pane.close", json!({"pane_id": pane}))?;
    Ok(text)
}

pub struct HerdrRuntime {
    binary: PathBuf,
    sessions: Mutex<Option<Arc<SharedServer>>>,
}

impl HerdrRuntime {
    pub fn new(binary: PathBuf) -> Self {
        Self {
            binary,
            sessions: Mutex::new(None),
        }
    }

    pub fn running_servers(&self) -> usize {
        usize::from(self.sessions.lock().expect("herdr runtime").is_some())
    }

    pub fn catalog(&self) -> Result<Catalog> {
        let digest = crate::catalog::file_digest(&self.binary)?;
        let harness = agency_proto::FrozenRef {
            id: "herdr-shell".into(),
            revision: format!("protocol-{PROTOCOL}"),
            digest,
        };
        let profession = Profession {
            reference: harness.clone(),
            harness: harness.clone(),
            model: "none".into(),
            persona: "locked Herdr pane".into(),
            terms: "deterministic program in a Herdr pane; not a model harness".into(),
            default_role: "worker".into(),
            skills: vec![],
            capabilities: Capabilities {
                input: true,
                stop: true,
                ..Capabilities::default()
            },
        };
        Ok(Catalog {
            professions: vec![profession],
            harnesses: vec![harness],
            skills: crate::catalog::skill_claims(Path::new("skills")).unwrap_or_default(),
        })
    }
}

struct Live {
    _server: Arc<Server>,
    pane: String,
    client: Client,
}

impl crate::runtime::Session for Live {
    fn input(&mut self, bytes: &[u8]) -> Result<()> {
        let text = std::str::from_utf8(bytes).map_err(|_| {
            PortError::new("INPUT_NOT_TEXT", "pane input is UTF-8 text", "send_text")
        })?;
        self.client.call(
            "pane.send_text",
            json!({"pane_id": self.pane, "text": text}),
        )?;
        Ok(())
    }
    fn stop(&mut self) -> Result<()> {
        let _ = self
            .client
            .call("pane.close", json!({"pane_id": self.pane}));
        Ok(())
    }
}

impl crate::runtime::Runtime for HerdrRuntime {
    fn catalog(&self) -> Result<Catalog> {
        HerdrRuntime::catalog(self)
    }
    fn start(
        &self,
        spec: &Sealed<ExecutionSpec>,
        _bundle: &Sealed<Bundle>,
        exec_root: &Path,
        credential_root: &Path,
    ) -> Result<crate::runtime::Running> {
        let state = credential_root
            .parent()
            .unwrap_or(credential_root)
            .join(format!(
                "herdr-state-{}",
                &hash(credential_root.as_os_str().as_encoded_bytes())[..12]
            ));
        if state.starts_with(credential_root) {
            return Err(PortError::new(
                "HERDR_STATE_UNSAFE",
                "Herdr state directory is inside the credential root",
                "choose_state_directory",
            ));
        }
        let shared = {
            let mut slot = self.sessions.lock().expect("herdr runtime");
            if slot.is_none() {
                *slot = Some(Arc::new(SharedServer::new(
                    self.binary.clone(),
                    credential_root.to_path_buf(),
                )));
            }
            Arc::clone(slot.as_ref().unwrap())
        };
        let exec_allow = exec_root.parent().filter(|parent| {
            !crate::confine::allowed_tree_contains_credential(parent, credential_root)
        });
        let server = shared.get(&state, exec_allow.unwrap_or(exec_root))?;
        let client = Client::connect(&server.socket)?;
        let tail = &hash(spec.document.idempotency_key.as_bytes())[..12];
        let marker = format!("HCTL2OUT{tail}");
        let command = format!("printf '%s%s\\n' HCTL2 OUT{tail}");
        let created = client.call(
            "workspace.create",
            json!({"cwd": exec_root, "label": spec.document.idempotency_key, "focus": false}),
        )?;
        let pane = created
            .pointer("/root_pane/pane_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                PortError::new(
                    "HERDR_PROTOCOL",
                    "workspace has no pane",
                    "retry_herdr_call",
                )
            })?
            .to_owned();
        client.call(
            "pane.send_text",
            json!({"pane_id": pane, "text": format!("{command}\n")}),
        )?;
        let (tx, rx) = mpsc::sync_channel(16);
        let watch = client.socket.clone();
        let pane_watch = pane.clone();
        let marker_watch = marker.clone();
        std::thread::spawn(move || {
            let client = Client { socket: watch };
            let matched = client.call(
                "pane.wait_for_output",
                json!({
                    "pane_id": pane_watch,
                    "source": "recent",
                    "match": {"type": "substring", "value": marker_watch},
                    "timeout_ms": 8000
                }),
            );
            let mut seen = HashSet::new();
            match matched {
                Ok(value) => {
                    let text = value
                        .pointer("/read/text")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_owned();
                    if observe_once(&mut seen, &text) {
                        let _ = tx.send(crate::runtime::RuntimeEvent::Observation {
                            kind: "pane".into(),
                            payload: json!({"text": text}),
                            source: EvidenceLevel::Narrated,
                        });
                    }
                    let _ = tx.send(crate::runtime::RuntimeEvent::Proposal {
                        schema: "herdr.pane.v1".into(),
                        bytes: marker_watch.into_bytes(),
                        source: EvidenceLevel::AdapterEvent,
                    });
                    let _ = tx.send(crate::runtime::RuntimeEvent::Exited {
                        code: Some(0),
                        requested_stop: false,
                    });
                }
                Err(error) => {
                    let _ = tx.send(crate::runtime::RuntimeEvent::ProtocolError(error.code));
                    let _ = tx.send(crate::runtime::RuntimeEvent::Exited {
                        code: None,
                        requested_stop: false,
                    });
                }
            }
            let _ = client.call("pane.close", json!({"pane_id": pane_watch}));
        });
        let _ = command;
        let _ = marker;
        Ok(crate::runtime::Running {
            session: Arc::new(Mutex::new(Box::new(Live {
                _server: server,
                pane,
                client: Client::connect(&client.socket)?,
            }))),
            events: rx,
        })
    }
}

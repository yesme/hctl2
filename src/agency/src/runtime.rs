//! Package 3 implements these private interfaces, not a new public runtime identity.
use agency_proto::context::{Bundle, Delivery};
use agency_proto::{
    Capabilities, Catalog, DispatchState, EvidenceLevel, ExecutionSpec, FrozenRef, PortError,
    Profession, Result, Sealed, canonical, hash,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Read, Write},
    os::unix::process::ExitStatusExt,
    os::unix::{fs::PermissionsExt, net::UnixStream},
    path::{Path, PathBuf},
    process::{Child, Stdio},
    sync::{Arc, Mutex, mpsc},
};

pub enum RuntimeEvent {
    Observation {
        kind: String,
        payload: serde_json::Value,
        source: EvidenceLevel,
    },
    Proposal {
        schema: String,
        bytes: Vec<u8>,
        source: EvidenceLevel,
    },
    Exited {
        code: Option<i32>,
        requested_stop: bool,
    },
    ProtocolError(String),
    DeadlineReached,
}
pub trait Session: Send {
    fn input(&mut self, bytes: &[u8]) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}
pub struct Running {
    pub session: Arc<Mutex<Box<dyn Session>>>,
    pub events: mpsc::Receiver<RuntimeEvent>,
}
/// All physical handles remain in the implementation. Return actual promised effects.
pub trait Runtime: Send + Sync {
    fn catalog(&self) -> Result<Catalog>;
    fn start(
        &self,
        spec: &Sealed<ExecutionSpec>,
        bundle: &Sealed<Bundle>,
        exec_root: &Path,
        credential_root: &Path,
    ) -> Result<Running>;
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptConfig {
    pub program: PathBuf,
    pub arguments: Vec<String>,
}
pub struct ScriptRuntime {
    config: ScriptConfig,
}
impl ScriptRuntime {
    pub fn new(config: ScriptConfig) -> Self {
        Self { config }
    }
}
fn reference(id: &str, digest: String) -> FrozenRef {
    FrozenRef {
        id: id.into(),
        revision: "1".into(),
        digest,
    }
}
impl Runtime for ScriptRuntime {
    fn catalog(&self) -> Result<Catalog> {
        // The fingerprint covers the program file. A missing file is not a locked version.
        let config_digest = crate::catalog::file_digest(&self.config.program)?;
        let harness = reference("script-protocol-fixture", config_digest.clone());
        let capabilities = Capabilities {
            input: true,
            stop: true,
            event_cursor: true,
            input_provenance: true,
            managed_single_writer: true,
            ..Capabilities::default()
        };
        let profession = Profession {
            reference: reference("script-worker", config_digest),
            harness: harness.clone(),
            model: "none".into(),
            persona: "protocol test executor".into(),
            terms: "not a coding harness; no PTY, tool provenance or OS hardening".into(),
            default_role: "fixture".into(),
            skills: vec![],
            capabilities,
        };
        Ok(Catalog {
            professions: vec![profession],
            harnesses: vec![harness],
            skills: vec![],
        })
    }
    fn start(
        &self,
        spec: &Sealed<ExecutionSpec>,
        bundle: &Sealed<Bundle>,
        root: &Path,
        credential_root: &Path,
    ) -> Result<Running> {
        crate::storage::private_dir(root)?;
        let materials = root.join("materials");
        crate::storage::private_dir(&materials)?;
        for entry in &bundle.document.entries {
            if let Delivery::Pointer {
                bytes,
                relative_name,
            } = &entry.delivery
            {
                let target = materials.join(relative_name);
                if let Some(parent) = target.parent() {
                    crate::storage::private_dir(parent)?;
                }
                std::fs::write(&target, bytes)?;
                std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o400))?;
                if hash(&std::fs::read(&target)?) != entry.bytes_digest {
                    return Err(PortError::new(
                        "DELIVERY_DIGEST_MISMATCH",
                        "local delivery changed",
                        "deliver_exact_bytes",
                    ));
                }
            }
        }
        // A socket's native write timeout bounds a non-reading fixture. A blocking pipe
        // would hold the tenant's input fence indefinitely and prevent cancellation.
        let (mut stdin, child_stdin) = UnixStream::pair()?;
        stdin.set_write_timeout(Some(std::time::Duration::from_millis(250)))?;
        let mut child = crate::confine::command(
            &self.config.program,
            &self.config.arguments,
            root,
            credential_root,
        )?;
        crate::confine::scrub(&mut child, root);
        child
            .stdin(Stdio::from(std::os::fd::OwnedFd::from(child_stdin)))
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = child.spawn()?;
        let mut initial = canonical(&serde_json::json!({"spec":spec,"bundle":bundle}))?;
        initial.push(b'\n');
        if let Err(e) = stdin.write_all(&initial) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e.into());
        }
        let stdout = child.stdout.take().expect("piped stdout");
        let (tx, rx) = mpsc::sync_channel(128);
        let private = Arc::new(Mutex::new(ScriptSession {
            child,
            stdin,
            stopped: false,
        }));
        let reader = Arc::clone(&private);
        std::thread::spawn(move || {
            let mut stream = BufReader::new(stdout);
            let mut terminal = false;
            loop {
                let mut bytes = Vec::new();
                let read = std::io::Read::by_ref(&mut stream)
                    .take(1024 * 1024 + 1)
                    .read_until(b'\n', &mut bytes);
                match read {
                    Ok(0) => break,
                    Ok(_) if bytes.len() <= 1024 * 1024 => {
                        match serde_json::from_slice::<Frame>(&bytes) {
                            Ok(Frame::Observation { kind, payload }) => {
                                if tx
                                    .send(RuntimeEvent::Observation {
                                        kind,
                                        payload,
                                        source: EvidenceLevel::Narrated,
                                    })
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            Ok(Frame::Result { schema, output }) if !terminal => {
                                terminal = true;
                                if tx
                                    .send(RuntimeEvent::Proposal {
                                        schema,
                                        bytes: output.into_bytes(),
                                        source: EvidenceLevel::Narrated,
                                    })
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            _ => {
                                let _ = tx.send(RuntimeEvent::ProtocolError(
                                    "invalid or duplicate terminal frame".into(),
                                ));
                                break;
                            }
                        }
                    }
                    _ => {
                        let _ = tx.send(RuntimeEvent::ProtocolError(
                            "truncated or oversized runtime stream".into(),
                        ));
                        break;
                    }
                }
            }
            loop {
                let mut session = reader.lock().expect("script session mutex");
                match session.child.try_wait() {
                    Ok(Some(status)) => {
                        let _ = tx.send(RuntimeEvent::Exited {
                            code: status
                                .code()
                                .or_else(|| status.signal().map(|signal| 128 + signal)),
                            requested_stop: session.stopped,
                        });
                        break;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        let _ = tx.send(RuntimeEvent::ProtocolError(e.to_string()));
                        break;
                    }
                }
                drop(session);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });
        Ok(Running {
            session: Arc::new(Mutex::new(Box::new(ScriptHandle(private)))),
            events: rx,
        })
    }
}
struct ScriptSession {
    child: Child,
    stdin: UnixStream,
    stopped: bool,
}
struct ScriptHandle(Arc<Mutex<ScriptSession>>);
impl Session for ScriptHandle {
    fn input(&mut self, bytes: &[u8]) -> Result<()> {
        self.0
            .lock()
            .expect("script mutex")
            .stdin
            .write_all(bytes)?;
        Ok(())
    }
    fn stop(&mut self) -> Result<()> {
        let mut session = self.0.lock().expect("script mutex");
        if session.child.try_wait()?.is_none() {
            session.child.kill()?;
            session.stopped = true;
        }
        Ok(())
    }
}
impl Drop for ScriptSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Frame {
    Observation {
        kind: String,
        payload: serde_json::Value,
    },
    Result {
        schema: String,
        output: String,
    },
}

pub(crate) fn final_state(has_result: bool, code: Option<i32>, stopped: bool) -> DispatchState {
    if stopped {
        DispatchState::Cancelled
    } else if has_result && code == Some(0) {
        DispatchState::ResultReturned
    } else {
        DispatchState::CannotFulfill
    }
}

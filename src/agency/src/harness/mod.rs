//! Codex and Claude Code behind Herdr. Physical pane and session ids stay here.
mod claude_code;
mod codex;

use crate::{
    catalog::{self, Binary, CLAUDE_VERSION, CODEX_VERSION},
    herdr,
    runtime::{Running, RuntimeEvent, Session},
};
use agency_proto::{Catalog, InputPolicy, PortError, Result, Sealed, context::Bundle};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

pub struct Installed {
    pub skills_dir: Option<PathBuf>,
    pub herdr: Option<PathBuf>,
    pub codex: Option<Binary>,
    pub claude: Option<Binary>,
}

impl Installed {
    pub fn discover(skills_dir: Option<PathBuf>) -> Self {
        let herdr = std::env::var_os("HCTL2_HERDR")
            .map(PathBuf::from)
            .or_else(|| which("herdr"))
            .filter(|path| herdr::locked_protocol(path).is_ok());
        let codex = which("codex").and_then(|p| catalog::describe(&p, CODEX_VERSION).ok());
        let claude = which("claude").and_then(|p| catalog::describe(&p, CLAUDE_VERSION).ok());
        Self {
            skills_dir,
            herdr,
            codex,
            claude,
        }
    }

    pub fn inventory(&self) -> Result<Catalog> {
        let skills = self
            .skills_dir
            .as_ref()
            .map(|d| catalog::skill_claims(d))
            .transpose()?
            .unwrap_or_default();
        let mut professions = Vec::new();
        if self.herdr.is_some() {
            if let Some(bin) = &self.codex {
                professions.push(codex::profession(bin));
            }
            if let Some(bin) = &self.claude {
                professions.push(claude_code::profession(bin));
            }
        }
        Ok(catalog::catalog(professions, skills))
    }
}

fn which(name: &str) -> Option<PathBuf> {
    let key = format!("HCTL2_{}", name.to_ascii_uppercase());
    if let Some(path) = std::env::var_os(&key) {
        return Some(path.into());
    }
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path).find_map(|dir| {
            let candidate = dir.join(name);
            candidate.is_file().then_some(candidate)
        })
    })
}

impl crate::runtime::Runtime for Installed {
    fn catalog(&self) -> Result<Catalog> {
        self.inventory()
    }
    fn start(
        &self,
        spec: &Sealed<agency_proto::ExecutionSpec>,
        bundle: &Sealed<Bundle>,
        exec_root: &Path,
        credential_root: &Path,
    ) -> Result<crate::runtime::Running> {
        launch(self, spec, bundle, exec_root, credential_root)
    }
}

pub fn launch(
    installed: &Installed,
    spec: &Sealed<agency_proto::ExecutionSpec>,
    _bundle: &Sealed<Bundle>,
    exec_root: &Path,
    credential_root: &Path,
) -> Result<Running> {
    if spec.document.input_policy != InputPolicy::NativeInteractiveAllowed {
        return Err(PortError::new(
            "INPUT_POLICY_UNVERIFIED",
            "only native interactive input is activated",
            "use_native_interactive",
        ));
    }
    let id = spec.document.profession.harness.id.as_str();
    let binary = match id {
        "codex" => installed.codex.as_ref(),
        "claude-code" => installed.claude.as_ref(),
        _ => None,
    }
    .ok_or_else(|| {
        PortError::new(
            "HARNESS_UNAVAILABLE",
            "harness is not installed at the locked version",
            "install_locked_harness",
        )
    })?;
    if !binary.locked {
        return Err(PortError::new(
            "HARNESS_UNVERIFIED",
            format!(
                "{} version {} is not the locked version",
                id, binary.version
            ),
            "install_locked_harness",
        ));
    }
    let herdr_bin = installed.herdr.as_ref().ok_or_else(|| {
        PortError::new(
            "HERDR_UNVERIFIED",
            "locked Herdr protocol 20 binary is not configured",
            "use_locked_herdr",
        )
    })?;
    let state = credential_root.join("herdr");
    let server = herdr::Server::start(herdr_bin, &state)?;
    let socket = server.socket.clone();
    let mut client = herdr::Client::connect(&socket)?;
    client.ping()?;
    let created = client.call(
        "workspace.create",
        json!({
            "cwd": exec_root,
            "label": &spec.document.idempotency_key,
            "focus": false,
            "env": {
                "PATH": "/usr/bin:/bin",
                "HOME": exec_root,
                "LANG": "C.UTF-8"
            }
        }),
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
    let args = match id {
        "codex" => codex::arguments(exec_root),
        "claude-code" => claude_code::arguments(exec_root),
        _ => Vec::new(),
    };
    client.call(
        "agent.start",
        json!({
            "name": id,
            "kind": kind(id),
            "pane_id": pane,
            "args": args,
        }),
    )?;
    let (tx, rx) = mpsc::sync_channel(64);
    let session = Arc::new(Mutex::new(HerdrSession {
        server: Some(server),
        socket,
        pane,
        stopped: false,
    }));
    let reader = Arc::clone(&session);
    std::thread::spawn(move || watch(reader, tx));
    Ok(Running {
        session: Arc::new(Mutex::new(Box::new(HerdrHandle(session)))),
        events: rx,
    })
}

fn kind(id: &str) -> &'static str {
    match id {
        "claude-code" => "claude",
        _ => "codex",
    }
}

struct HerdrSession {
    server: Option<herdr::Server>,
    socket: PathBuf,
    pane: String,
    stopped: bool,
}

struct HerdrHandle(Arc<Mutex<HerdrSession>>);

impl Session for HerdrHandle {
    fn input(&mut self, bytes: &[u8]) -> Result<()> {
        let text = std::str::from_utf8(bytes).map_err(|_| {
            PortError::new(
                "INPUT_NOT_TEXT",
                "native interactive input is UTF-8 text",
                "send_text",
            )
        })?;
        let session = self.0.lock().expect("herdr session");
        if session.socket.as_os_str().is_empty() {
            return Err(PortError::new(
                "HERDR_HANDLE_MISSING",
                "session has no private socket",
                "restart_execution",
            ));
        }
        let mut client = herdr::Client::connect(&session.socket)?;
        client.call(
            "pane.send_text",
            json!({"pane_id": session.pane, "text": text}),
        )?;
        Ok(())
    }
    fn stop(&mut self) -> Result<()> {
        let mut session = self.0.lock().expect("herdr session");
        session.stopped = true;
        if !session.socket.as_os_str().is_empty() {
            let mut client = herdr::Client::connect(&session.socket)?;
            let _ = client.call("pane.close", json!({"pane_id": session.pane}));
        }
        session.server.take();
        Ok(())
    }
}

fn watch(session: Arc<Mutex<HerdrSession>>, tx: mpsc::SyncSender<RuntimeEvent>) {
    let socket = session.lock().expect("herdr session").socket.clone();
    let pane = session.lock().expect("herdr session").pane.clone();
    if socket.as_os_str().is_empty() {
        let _ = tx.send(RuntimeEvent::ProtocolError("herdr socket missing".into()));
        return;
    }
    let Ok(mut client) = herdr::Client::connect(&socket) else {
        let _ = tx.send(RuntimeEvent::ProtocolError("herdr reconnect failed".into()));
        return;
    };
    loop {
        if session.lock().expect("herdr session").stopped {
            let _ = tx.send(RuntimeEvent::Exited {
                code: None,
                requested_stop: true,
            });
            break;
        }
        match client.call(
            "pane.read",
            json!({"pane_id": pane, "source": "recent", "lines": 20}),
        ) {
            Ok(value) => {
                if tx
                    .send(RuntimeEvent::Observation {
                        kind: "pane".into(),
                        payload: value,
                        source: agency_proto::EvidenceLevel::Narrated,
                    })
                    .is_err()
                {
                    break;
                }
            }
            Err(error) => {
                let _ = tx.send(RuntimeEvent::ProtocolError(error.code));
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

impl Drop for HerdrSession {
    fn drop(&mut self) {
        self.server.take();
    }
}

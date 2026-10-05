//! Private Herdr client for the locked 0.8.2 binary (protocol 20).
//! Pane and socket ids stay in this process. The caller supplies the pane
//! program. This module does not register a profession or read a bundle.
use crate::confine;
use agency_proto::{PortError, Result, hash};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
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
        if let Some(error) = value.get("error") {
            return Err(PortError::new(
                "HERDR_REJECTED",
                error.to_string(),
                "read_herdr_error",
            ));
        }
        if value.get("id").and_then(Value::as_str) != Some(id.as_str()) {
            return Err(PortError::new(
                "HERDR_PROTOCOL",
                "response id does not match the request",
                "retry_herdr_call",
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
        reap_previous(state, &binary);
        let socket = socket_path(state);
        let socket_dir = socket
            .parent()
            .ok_or_else(|| PortError::invalid("Herdr socket directory is missing"))?
            .to_path_buf();
        crate::storage::private_dir(&socket_dir)?;
        let _ = std::fs::remove_file(&socket);
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
                    "{}\n{}\n{}",
                    state.display(),
                    socket_dir.display(),
                    exec_parent.display()
                ),
            )
            .env(
                "HCTL2_CONFINE_READ",
                binary
                    .parent()
                    .unwrap_or(binary.as_path())
                    .display()
                    .to_string(),
            )
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(std::fs::File::create(state.join("server.err"))?);
        let child = child.spawn()?;
        // From here the child must be stopped on every exit except a live Server.
        let mut stop = StopChild(Some(child));
        let pid = stop.id();
        if let Err(error) = std::fs::write(state.join("herdr.pid"), pid.to_string()) {
            return Err(error.into());
        }
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            if socket.exists()
                && Client::connect(&socket)
                    .and_then(|client| client.ping())
                    .is_ok()
            {
                return Ok(Self {
                    child: stop.disarm(),
                    socket,
                    state: state.to_path_buf(),
                });
            }
            if stop.try_wait()?.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        drop(stop);
        let detail = std::fs::read_to_string(state.join("server.err")).unwrap_or_default();
        Err(PortError::new(
            "HERDR_NOT_READY",
            format!("locked Herdr server did not answer protocol 20: {detail}"),
            "use_locked_herdr",
        ))
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        signal_group(self.child.id(), "KILL");
        let _ = self.child.wait();
    }
}

/// State is outside the pane's working directory and outside the credential root.
pub fn state_dir(exec_parent: &Path, credential_root: &Path) -> Result<PathBuf> {
    let credential_root = credential_root.canonicalize().map_err(|_| {
        PortError::new(
            "CREDENTIAL_ROOT_UNRESOLVED",
            "credential root cannot be canonicalized",
            "choose_credential_root",
        )
    })?;
    let exec_parent = exec_parent.canonicalize().map_err(|_| {
        PortError::new(
            "EXECUTION_ROOT_UNSAFE",
            "execution directory cannot be canonicalized",
            "choose_execution_directory",
        )
    })?;
    let parent = exec_parent.parent().unwrap_or(Path::new("/tmp"));
    let digest = &hash(credential_root.as_os_str().as_encoded_bytes())[..20];
    let state = parent.join(format!("hctl2-herdr-state-{digest}"));
    if state.starts_with(&exec_parent)
        || exec_parent.starts_with(&state)
        || state.starts_with(&credential_root)
        || credential_root.starts_with(&state)
    {
        return Err(PortError::new(
            "HERDR_STATE_UNSAFE",
            "Herdr state directory overlaps the pane directory or the credential root",
            "choose_state_directory",
        ));
    }
    crate::storage::private_dir(&state)?;
    Ok(state)
}

/// One Herdr process for every pane started through this pipe.
pub struct Pipe {
    binary: PathBuf,
    credential_root: PathBuf,
    exec_parent: PathBuf,
    state: PathBuf,
    server: Mutex<Option<Arc<Server>>>,
    started: AtomicUsize,
}

impl Pipe {
    pub fn open(binary: &Path, credential_root: &Path, exec_parent: &Path) -> Result<Self> {
        let binary = binary.canonicalize().map_err(|_| {
            PortError::new(
                "HERDR_BINARY_MISSING",
                format!("locked Herdr binary is not at {}", binary.display()),
                "use_locked_herdr",
            )
        })?;
        let state = state_dir(exec_parent, credential_root)?;
        Ok(Self {
            binary,
            credential_root: credential_root.canonicalize().map_err(|_| {
                PortError::new(
                    "CREDENTIAL_ROOT_UNRESOLVED",
                    "credential root cannot be canonicalized",
                    "choose_credential_root",
                )
            })?,
            exec_parent: exec_parent.canonicalize().map_err(|_| {
                PortError::new(
                    "EXECUTION_ROOT_UNSAFE",
                    "execution directory cannot be canonicalized",
                    "choose_execution_directory",
                )
            })?,
            state,
            server: Mutex::new(None),
            started: AtomicUsize::new(0),
        })
    }

    pub fn state(&self) -> &Path {
        &self.state
    }

    pub fn servers_started(&self) -> usize {
        self.started.load(Ordering::SeqCst)
    }

    pub fn pid(&self) -> Option<u32> {
        self.server
            .lock()
            .expect("herdr server")
            .as_ref()
            .map(|server| server.pid())
    }

    pub fn run(&self, label: &str, command: &str, marker: &str) -> Result<String> {
        let server = self.ensure()?;
        let client = Client::connect(&server.socket)?;
        run_command(&client, &self.exec_parent, label, command, marker)
    }

    fn ensure(&self) -> Result<Arc<Server>> {
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
            &self.state,
            &self.credential_root,
            &self.exec_parent,
        )?);
        self.started.fetch_add(1, Ordering::SeqCst);
        *slot = Some(Arc::clone(&server));
        Ok(server)
    }
}

pub fn socket_path(state: &Path) -> PathBuf {
    PathBuf::from("/tmp")
        .join(format!(
            "hctl2-herdr-{}",
            &hash(state.as_os_str().as_encoded_bytes())[..20]
        ))
        .join("herdr.sock")
}

/// Run `command` in a new workspace pane and return the pane text that contains `marker`.
/// This refuses before creating a pane or sending text when `command` contains `marker`,
/// or any control character other than newline. Tab is a control character and is refused.
/// The terminal echoes typed text, and backspace or delete can assemble a marker that
/// was not in the original command. Print the marker in pieces, and make that print the last step.
/// After a pane id exists, the pane is closed on both success and failure.
pub fn run_command(
    client: &Client,
    cwd: &Path,
    label: &str,
    command: &str,
    marker: &str,
) -> Result<String> {
    if marker.is_empty() {
        return Err(PortError::invalid("marker is empty"));
    }
    if command.contains(marker) {
        return Err(PortError::invalid(
            "command contains the marker; print it in pieces so the echo is not the output",
        ));
    }
    if command.chars().any(|c| c.is_control() && c != '\n') {
        return Err(PortError::invalid(
            "command contains a control character other than newline; tab is included",
        ));
    }
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
    let mut open = OpenPane {
        client,
        pane: pane.to_owned(),
        closed: false,
    };
    let outcome = (|| {
        open.client.call(
            "pane.send_text",
            json!({"pane_id": open.pane, "text": format!("{command}\n")}),
        )?;
        let matched = open
            .client
            .call(
                "pane.wait_for_output",
                json!({
                    "pane_id": open.pane,
                    "source": "recent",
                    "match": {"type": "substring", "value": marker},
                    "timeout_ms": 8000
                }),
            )
            .map_err(|error| PortError::new("HERDR_OUTPUT_MISSING", error.message, "read_pane"))?;
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
        Ok(text)
    })();
    let closed = open.close();
    match (outcome, closed) {
        (Ok(text), Ok(())) => Ok(text),
        (Ok(_), Err(close)) => Err(close),
        (Err(exec), Ok(())) => Err(exec),
        (Err(exec), Err(close)) => Err(PortError::new(
            &exec.code,
            format!("{exec}; pane close failed: {close}"),
            &exec.recovery_action,
        )),
    }
}

struct OpenPane<'a> {
    client: &'a Client,
    pane: String,
    closed: bool,
}

impl OpenPane<'_> {
    fn close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        self.closed = true;
        self.client
            .call("pane.close", json!({"pane_id": self.pane}))
            .map(|_| ())
    }
}

impl Drop for OpenPane<'_> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

struct StopChild(Option<Child>);

impl StopChild {
    fn id(&self) -> u32 {
        self.0.as_ref().expect("herdr child").id()
    }

    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.0.as_mut().expect("herdr child").try_wait()
    }

    fn disarm(mut self) -> Child {
        self.0.take().expect("herdr child")
    }
}

impl Drop for StopChild {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.take() {
            signal_group(child.id(), "KILL");
            let _ = child.wait();
        }
    }
}

fn reap_previous(state: &Path, binary: &Path) {
    let Ok(text) = std::fs::read_to_string(state.join("herdr.pid")) else {
        return;
    };
    let Ok(pid) = text.trim().parse::<u32>() else {
        return;
    };
    if pid == 0 || !still_running_binary(pid, binary) {
        return;
    }
    signal_group(pid, "TERM");
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline && still_running_binary(pid, binary) {
        std::thread::sleep(Duration::from_millis(50));
    }
    if still_running_binary(pid, binary) {
        signal_group(pid, "KILL");
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline && still_running_binary(pid, binary) {
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

fn still_running_binary(pid: u32, binary: &Path) -> bool {
    let Ok(output) = Command::new("/bin/ps")
        .args(["-ww", "-p", &pid.to_string(), "-o", "stat=,command="])
        .output()
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let line = String::from_utf8_lossy(&output.stdout);
    let stat = line.split_whitespace().next().unwrap_or("");
    if stat.is_empty() || stat.starts_with('Z') {
        return false;
    }
    line.contains(&binary.display().to_string())
}

fn signal_group(pid: u32, signal: &str) {
    let _ = Command::new("/bin/kill")
        .args(["-s", signal, "--", &format!("-{pid}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

//! Private Herdr socket client. Handles stay in this process.
//! Locked wire: Herdr 0.8.2, protocol 20. A newer local binary is not that contract.
use crate::catalog::HERDR_PROTOCOL;
use agency_proto::{PortError, Result};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

pub struct Client {
    stream: UnixStream,
    next: u64,
}

impl Client {
    pub fn connect(socket: &Path) -> Result<Self> {
        let stream = UnixStream::connect(socket)?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        Ok(Self { stream, next: 1 })
    }

    pub fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = format!("a{}", self.next);
        self.next += 1;
        let mut line = serde_json::to_vec(&json!({
            "id": id,
            "protocol": HERDR_PROTOCOL,
            "method": method,
            "params": params,
        }))?;
        line.push(b'\n');
        self.stream.write_all(&line)?;
        let mut reader = BufReader::new(self.stream.try_clone()?);
        let mut response = String::new();
        reader.read_line(&mut response)?;
        let value: Value = serde_json::from_str(&response)?;
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

    pub fn ping(&mut self) -> Result<Value> {
        let result = self.call("ping", json!({}))?;
        let protocol = result.get("protocol").and_then(Value::as_u64).unwrap_or(0);
        if protocol != u64::from(HERDR_PROTOCOL)
            || result.get("type").and_then(Value::as_str) != Some("pong")
        {
            return Err(PortError::new(
                "HERDR_PROTOCOL",
                format!("Herdr protocol {protocol} is not locked protocol {HERDR_PROTOCOL}"),
                "use_locked_herdr",
            ));
        }
        Ok(result)
    }
}

pub struct Server {
    child: Child,
    pub socket: PathBuf,
}

impl Server {
    pub fn start(binary: &Path, state: &Path) -> Result<Self> {
        crate::storage::private_dir(state)?;
        let socket_dir = PathBuf::from("/tmp").join(format!(
            "hctl2-herdr-{}",
            &agency_proto::hash(state.as_os_str().as_encoded_bytes())[..20]
        ));
        crate::storage::private_dir(&socket_dir)?;
        let socket = socket_dir.join("herdr.sock");
        let _ = std::fs::remove_file(&socket);
        let mut child = Command::new(binary)
            .arg("server")
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HERDR_SOCKET_PATH", &socket)
            .env("HERDR_CONFIG_PATH", state.join("config.toml"))
            .env("XDG_CONFIG_HOME", state.join("config"))
            .env("XDG_STATE_HOME", state.join("state"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if socket.exists()
                && let Ok(mut client) = Client::connect(&socket)
                && client.ping().is_ok()
            {
                return Ok(Self { child, socket });
            }
            if child.try_wait()?.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = child.kill();
        let _ = child.wait();
        Err(PortError::new(
            "HERDR_NOT_READY",
            "locked Herdr server did not answer protocol 20",
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

/// Read `protocol` from `herdr api schema --json` without accepting another protocol.
pub fn locked_protocol(binary: &Path) -> Result<u32> {
    let output = Command::new(binary)
        .args(["api", "schema", "--json"])
        .output()?;
    if !output.status.success() {
        return Err(PortError::new(
            "HERDR_PROTOCOL",
            "herdr api schema failed",
            "use_locked_herdr",
        ));
    }
    let value: Value = serde_json::from_slice(&output.stdout)?;
    let protocol = value.get("protocol").and_then(Value::as_u64).unwrap_or(0);
    if protocol != u64::from(HERDR_PROTOCOL) {
        return Err(PortError::new(
            "HERDR_PROTOCOL",
            format!("schema protocol {protocol} is not {HERDR_PROTOCOL}"),
            "use_locked_herdr",
        ));
    }
    Ok(HERDR_PROTOCOL)
}

//! Start a program in a Herdr pane without treating the screen as completion.
//! The caller's text is a script file. The pane only receives a fixed launcher.
//! The launcher writes the script's exit code after the script returns.
use crate::herdr::{Client, Server};
use crate::runtime::{Running, Runtime, RuntimeEvent, Session};
use agency_proto::context::{Bundle, Delivery};
use agency_proto::{
    Capabilities, Catalog, EvidenceLevel, ExecutionSpec, FrozenRef, PortError, Profession, Result,
    Sealed,
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

pub struct Launch {
    client: Client,
    pane: String,
    exit_path: PathBuf,
    stdout_path: PathBuf,
    _server: Arc<Server>,
}

pub struct Finished {
    pub code: i32,
    pub stdout: Vec<u8>,
}

impl Launch {
    pub fn start(
        server: Arc<Server>,
        exec_dir: &Path,
        state_dir: &Path,
        credential_root: &Path,
        body: &str,
        label: &str,
    ) -> Result<Self> {
        fs::write(exec_dir.join("program.sh"), body)?;
        let exit_path = exec_dir.join("exit");
        let stdout_path = exec_dir.join("stdout");
        let stderr_path = exec_dir.join("stderr");
        let _ = fs::remove_file(&exit_path);
        let _ = (state_dir, credential_root);
        let runner = exec_dir.join("runner.sh");
        fs::write(
            &runner,
            runner_script(
                &exec_dir.join("program.sh"),
                &exit_path,
                &stdout_path,
                &stderr_path,
            ),
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut mode = fs::metadata(&runner)?.permissions();
            mode.set_mode(0o755);
            fs::set_permissions(&runner, mode)?;
        }
        let client = Client::connect(&server.socket)?;
        let created = client.call(
            "workspace.create",
            json!({"cwd": exec_dir, "label": label, "focus": false}),
        )?;
        let pane = created
            .pointer("/root_pane/pane_id")
            .and_then(|value| value.as_str())
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
            json!({"pane_id": pane, "text": format!("/bin/sh {}\n", sh_quote(&runner))}),
        )?;
        Ok(Self {
            client,
            pane,
            exit_path,
            stdout_path,
            _server: server,
        })
    }

    pub fn pane(&self) -> &str {
        &self.pane
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Exit code written by the launcher. Screen text is not consulted.
    pub fn poll_exit(&self) -> Result<Option<i32>> {
        if !self.exit_path.is_file() {
            return Ok(None);
        }
        let text = fs::read_to_string(&self.exit_path)?;
        let code = text
            .trim()
            .parse::<i32>()
            .map_err(|_| PortError::invalid("launcher exit file is not an integer"))?;
        Ok(Some(code))
    }

    pub fn wait(&self, timeout: Duration) -> Result<Finished> {
        let deadline = Instant::now() + timeout;
        let code = loop {
            if let Some(code) = self.poll_exit()? {
                break code;
            }
            if Instant::now() >= deadline {
                return Err(PortError::new(
                    "LAUNCH_TIMEOUT",
                    "launcher exit file was not written before the caller timeout",
                    "raise_timeout_or_cancel",
                ));
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        let stdout = fs::read(&self.stdout_path).unwrap_or_default();
        Ok(Finished { code, stdout })
    }

    pub fn cancel(&self) -> Result<()> {
        self.client
            .call("pane.close", json!({"pane_id": self.pane}))
            .map(|_| ())
    }
}

fn runner_script(
    program: &Path,
    exit_path: &Path,
    stdout_path: &Path,
    stderr_path: &Path,
) -> String {
    format!(
        "#!/bin/sh\nset +e\n/bin/sh {program} >{stdout} 2>{stderr}\ncode=$?\nprintf '%s\\n' \"$code\" > {exit}\nexit \"$code\"\n",
        program = sh_quote(program),
        stdout = sh_quote(stdout_path),
        stderr = sh_quote(stderr_path),
        exit = sh_quote(exit_path),
    )
}

fn sh_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

pub fn locked_digest() -> &'static str {
    if cfg!(target_os = "linux") {
        "976150a14d490c94b243ea2e1a7eb2dfb67f12e36b182db90936f6728e6aecf4"
    } else if cfg!(target_arch = "x86_64") {
        "ab50262c8190cd7aa9056d249d255c08c328c3e8716de9cfa29db4f131b8e2c1"
    } else {
        "a5d4f4d504d8b309c91f811050559300faba31258425f53c50852fc96f6ae574"
    }
}

pub fn installed_herdr(install_root: &Path) -> Result<PathBuf> {
    let path = install_root.join("libexec/hctl2/herdr");
    if !path.is_file() {
        return Err(PortError::new(
            "HERDR_BINARY_MISSING",
            format!("installed Herdr is not at {}", path.display()),
            "install_locked_herdr",
        ));
    }
    let digest = crate::catalog::file_digest(&path)?;
    if digest != locked_digest() {
        return Err(PortError::new(
            "HERDR_DIGEST_MISMATCH",
            format!("installed Herdr sha256 {digest} does not match lock.json"),
            "install_locked_herdr",
        ));
    }
    Ok(path)
}

pub fn smoke(
    server: Arc<Server>,
    exec_dir: &Path,
    state_dir: &Path,
    credential_root: &Path,
) -> Result<()> {
    let launch = Launch::start(
        server,
        exec_dir,
        state_dir,
        credential_root,
        "printf '%s\\n' smoke-ok\nexit 0\n",
        "smoke",
    )?;
    let finished = launch.wait(Duration::from_secs(15))?;
    if finished.code != 0 || !finished.stdout.windows(8).any(|item| item == b"smoke-ok") {
        return Err(PortError::new(
            "HERDR_SMOKE_FAILED",
            format!("smoke exit {} stdout {:?}", finished.code, finished.stdout),
            "check_locked_herdr",
        ));
    }
    let _ = launch.cancel();
    Ok(())
}

pub struct InstalledHerdr {
    binary: PathBuf,
}

impl InstalledHerdr {
    pub fn open(binary: PathBuf) -> Result<Self> {
        let root = std::env::temp_dir().join(format!(
            "hctl2-herdr-smoke-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let cred = root.join("cred");
        let exec = root.join("exec");
        fs::create_dir_all(&cred)?;
        fs::create_dir_all(&exec)?;
        let state = crate::herdr::state_dir(&exec, &cred)?;
        let server = Arc::new(Server::start(&binary, &state, &cred, &exec)?);
        let result = smoke(Arc::clone(&server), &exec, &state, &cred);
        drop(server);
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&state);
        result?;
        Ok(Self { binary })
    }
}

struct HerdrSession {
    launch: Arc<Launch>,
}

impl Session for HerdrSession {
    fn input(&mut self, _: &[u8]) -> Result<()> {
        Err(PortError::invalid("this launch does not take pane input"))
    }
    fn stop(&mut self) -> Result<()> {
        self.launch.cancel()
    }
}

impl Runtime for InstalledHerdr {
    fn catalog(&self) -> Result<Catalog> {
        let digest = crate::catalog::file_digest(&self.binary)?;
        let harness = FrozenRef {
            id: "herdr".into(),
            revision: "protocol-20".into(),
            digest: digest.clone(),
        };
        let profession = Profession {
            reference: FrozenRef {
                id: "herdr-locked".into(),
                revision: "protocol-20".into(),
                digest,
            },
            harness: harness.clone(),
            model: "none".into(),
            persona: "locked herdr pane".into(),
            terms: "exit file from the fixed launcher; screen text is observation only".into(),
            default_role: "shell".into(),
            skills: vec![],
            capabilities: Capabilities {
                input: false,
                stop: true,
                ..Capabilities::default()
            },
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
        exec_root: &Path,
        credential_root: &Path,
    ) -> Result<Running> {
        let body = script_body(&bundle.document)?;
        let state = crate::herdr::state_dir(exec_root, credential_root)?;
        let server = Arc::new(Server::start(
            &self.binary,
            &state,
            credential_root,
            exec_root,
        )?);
        let launch = Launch::start(
            server,
            exec_root,
            &state,
            credential_root,
            &body,
            &spec.document.idempotency_key,
        )?;
        let shared = Arc::new(launch);
        let waiter = Arc::clone(&shared);
        let timeout = Duration::from_millis(spec.document.deadline_ms);
        let (tx, rx) = std::sync::mpsc::sync_channel(8);
        std::thread::spawn(move || match waiter.wait(timeout) {
            Ok(finished) if finished.code == 0 => {
                let _ = tx.send(RuntimeEvent::Proposal {
                    schema: "herdr.stdout.v1".into(),
                    bytes: finished.stdout,
                    source: EvidenceLevel::AdapterEvent,
                });
                let _ = tx.send(RuntimeEvent::Exited {
                    code: Some(0),
                    requested_stop: false,
                });
            }
            Ok(finished) => {
                let _ = tx.send(RuntimeEvent::Exited {
                    code: Some(finished.code),
                    requested_stop: false,
                });
            }
            Err(error) => {
                let _ = tx.send(RuntimeEvent::ProtocolError(error.code));
                let _ = tx.send(RuntimeEvent::Exited {
                    code: None,
                    requested_stop: false,
                });
            }
        });
        Ok(Running {
            session: Arc::new(std::sync::Mutex::new(Box::new(HerdrSession {
                launch: shared,
            }))),
            events: rx,
        })
    }
}

fn script_body(bundle: &Bundle) -> Result<String> {
    for entry in &bundle.entries {
        let bytes = match &entry.delivery {
            Delivery::Pointer { bytes, .. } | Delivery::Inline { bytes } => bytes,
            Delivery::Recall { .. } => continue,
        };
        return String::from_utf8(bytes.clone())
            .map_err(|_| PortError::invalid("bundle script is not utf-8"));
    }
    Err(PortError::invalid("bundle has no script"))
}

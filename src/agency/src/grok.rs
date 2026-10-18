//! Grok Build over ACP (`grok agent stdio`). There is no interactive pane.
//! This client does not pass `--always-approve` or a bypass permission mode.
//! Login stays in the process environment. This module does not copy it or edit `~/.grok`.

use agency_proto::{EvidenceLevel, ExecutionSpec, PortError, Result, Sealed, context::Bundle};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

use crate::runtime::{Running, RuntimeEvent, Session};

const PROFESSION: &str = "grok-build";

pub(crate) fn installed() -> Option<(PathBuf, String)> {
    let path = PathBuf::from(std::env::var_os("HCTL2_GROK")?);
    let digest = crate::catalog::file_digest(&path).ok()?;
    Some((path, digest))
}

pub(crate) fn argv(bin: &Path, leader_socket: &Path) -> Vec<String> {
    vec![
        bin.display().to_string(),
        "agent".into(),
        "--no-leader".into(),
        "--leader-socket".into(),
        leader_socket.display().to_string(),
        "stdio".into(),
    ]
}

pub(crate) fn dispatch(
    spec: &Sealed<ExecutionSpec>,
    bundle: &Sealed<Bundle>,
    exec_root: &Path,
) -> Result<Running> {
    if spec.document.profession.reference.id != PROFESSION {
        return Err(PortError::invalid("not a grok-build dispatch"));
    }
    if spec.document.write_lease.is_some() {
        return Err(PortError::new(
            "HARNESS_NOT_STARTED",
            "grok-build write dispatch is not wired yet",
            "use_read_only_until_write_slice",
        ));
    }
    let (bin, _) = installed().ok_or_else(|| {
        PortError::new(
            "HARNESS_NOT_STARTED",
            "HCTL2_GROK is unset or the binary cannot be hashed",
            "set_hctl2_grok",
        )
    })?;
    let text = crate::launch::task_text(&bundle.document)?;
    let state = exec_root.join("grok-acp");
    crate::storage::private_dir(&state)?;
    let (tx, rx) = mpsc::sync_channel(8);
    std::thread::spawn(move || {
        let outcome = speak_process(&bin, &state, &text, Duration::from_secs(120));
        match outcome {
            Ok(turn) => {
                let _ = tx.send(RuntimeEvent::Observation {
                    kind: "grok.stop".into(),
                    payload: json!({ "stop_reason": turn.stop_reason }),
                    source: EvidenceLevel::AdapterEvent,
                });
                let _ = tx.send(RuntimeEvent::Proposal {
                    schema: "grok.turn.v1".into(),
                    bytes: turn.text.into_bytes(),
                    source: EvidenceLevel::AdapterEvent,
                });
                let _ = tx.send(RuntimeEvent::TurnReturned);
                let _ = tx.send(RuntimeEvent::Exited {
                    code: Some(0),
                    requested_stop: false,
                });
            }
            Err(error) => {
                let _ = tx.send(RuntimeEvent::ProtocolError(error.message));
            }
        }
    });
    Ok(Running {
        session: std::sync::Arc::new(std::sync::Mutex::new(Box::new(IdleSession))),
        events: rx,
    })
}

struct IdleSession;

impl Session for IdleSession {
    fn input(&mut self, _: &[u8]) -> Result<()> {
        Err(PortError::invalid("grok stdio has no pane input"))
    }
    fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct Turn {
    text: String,
    stop_reason: String,
    session_id: String,
}

fn speak_process(bin: &Path, state: &Path, text: &str, budget: Duration) -> Result<Turn> {
    let socket = state.join("leader.sock");
    let args = argv(bin, &socket);
    if args
        .iter()
        .any(|arg| arg == "--always-approve" || arg.contains("bypassPermissions"))
    {
        return Err(PortError::invalid("grok client must not auto-approve"));
    }
    let mut child = Command::new(&args[0])
        .args(&args[1..])
        .current_dir(state)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| PortError::invalid(format!("spawn grok: {error}")))?;
    let stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let saved = std::fs::read_to_string(state.join("session-id")).ok();
    let turn = speak(
        BufReader::new(stdout),
        stdin,
        saved.as_deref().map(str::trim).filter(|id| !id.is_empty()),
        text,
        budget,
    );
    let _ = child.kill();
    let _ = child.wait();
    let turn = turn?;
    std::fs::write(state.join("session-id"), &turn.session_id)?;
    Ok(turn)
}

fn speak<R: BufRead, W: Write>(
    mut reader: R,
    mut writer: W,
    session: Option<&str>,
    text: &str,
    budget: Duration,
) -> Result<Turn> {
    let deadline = Instant::now() + budget;
    let init = request(
        &mut reader,
        &mut writer,
        "initialize",
        json!({"protocolVersion": 1, "clientCapabilities": {}}),
        deadline,
    )?;
    let methods = init["authMethods"].as_array().cloned().unwrap_or_default();
    if methods.iter().any(|method| method["id"] == "cached_token") {
        request(
            &mut reader,
            &mut writer,
            "authenticate",
            json!({"methodId": "cached_token", "_meta": {"headless": true}}),
            deadline,
        )?;
    }
    let session_id = if let Some(session) = session {
        request(
            &mut reader,
            &mut writer,
            "session/load",
            json!({"sessionId": session, "cwd": ".", "mcpServers": []}),
            deadline,
        )?;
        session.to_owned()
    } else {
        let created = request(
            &mut reader,
            &mut writer,
            "session/new",
            json!({"cwd": ".", "mcpServers": []}),
            deadline,
        )?;
        created["sessionId"]
            .as_str()
            .filter(|id| !id.is_empty())
            .ok_or_else(|| PortError::invalid("session/new returned no session id"))?
            .to_owned()
    };
    let mut answer = String::new();
    let result = request_collecting(
        &mut reader,
        &mut writer,
        "session/prompt",
        json!({"sessionId": session_id, "prompt": [{"type": "text", "text": text}]}),
        deadline,
        &mut answer,
    )?;
    let stop_reason = result["stopReason"]
        .as_str()
        .unwrap_or("unknown")
        .to_owned();
    if answer.is_empty() {
        return Err(PortError::invalid(
            "turn completed without an agent message",
        ));
    }
    Ok(Turn {
        text: answer,
        stop_reason,
        session_id,
    })
}

fn request<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    method: &str,
    params: Value,
    deadline: Instant,
) -> Result<Value> {
    let mut ignored = String::new();
    request_collecting(reader, writer, method, params, deadline, &mut ignored)
}

fn request_collecting<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    method: &str,
    params: Value,
    deadline: Instant,
    answer: &mut String,
) -> Result<Value> {
    let id = 1;
    writer.write_all(
        format!(
            "{}\n",
            json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
        )
        .as_bytes(),
    )?;
    writer.flush()?;
    loop {
        if Instant::now() >= deadline {
            return Err(PortError::new(
                "HARNESS_TURN_ERROR",
                format!("{method} exceeded the turn budget"),
                "retry_dispatch",
            ));
        }
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Err(PortError::invalid(format!(
                "{method} ended before a response"
            )));
        }
        let message: Value = serde_json::from_str(line.trim())
            .map_err(|error| PortError::invalid(format!("grok acp is not json: {error}")))?;
        if message.get("method").is_some() && message.get("id").is_some() && message["id"] != id {
            return Err(PortError::new(
                "HARNESS_TURN_ERROR",
                format!(
                    "grok asked for {} and Agency will not answer it",
                    message["method"].as_str().unwrap_or("a request")
                ),
                "owner_must_confirm",
            ));
        }
        if message["method"] == "session/update" {
            let update = &message["params"]["update"];
            if update["sessionUpdate"] == "agent_message_chunk"
                && let Some(text) = update["content"]["text"].as_str()
            {
                answer.push_str(text);
            }
            continue;
        }
        if message.get("id") == Some(&json!(id)) {
            if let Some(error) = message.get("error") {
                return Err(PortError::invalid(format!(
                    "{method} error {}: {}",
                    error["code"], error["message"]
                )));
            }
            return Ok(message["result"].clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, Write};

    #[test]
    fn argv_does_not_auto_approve() {
        let args = argv(Path::new("grok"), Path::new("/tmp/leader.sock"));
        assert!(args.iter().all(|arg| arg != "--always-approve"));
        assert!(args.iter().all(|arg| !arg.contains("bypassPermissions")));
        assert!(args.contains(&"stdio".to_owned()));
    }

    #[test]
    fn prompt_bytes_round_trip_and_a_later_load() {
        let first = speak_with_fake(None, "Reply with exactly H2PROBE-PONG");
        assert_eq!(first.text, "Reply with exactly H2PROBE-PONG");
        assert_eq!(first.stop_reason, "end_turn");
        assert_eq!(first.session_id, "sess-1");
        let second = speak_with_fake(Some("sess-1"), "!/keep");
        assert_eq!(second.text, "!/keep");
        assert_eq!(second.session_id, "sess-1");
    }

    #[test]
    fn an_agent_request_is_not_answered() {
        let incoming = b"{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"session/request_permission\",\"params\":{}}\n";
        let error = speak(
            incoming.as_slice(),
            Vec::<u8>::new(),
            None,
            "x",
            Duration::from_secs(2),
        )
        .unwrap_err();
        assert!(error.message.contains("will not answer"));
    }

    fn speak_with_fake(session: Option<&str>, text: &str) -> Turn {
        let (agent_read, client_write) = std::io::pipe().unwrap();
        let (client_read, mut agent_write) = std::io::pipe().unwrap();
        std::thread::spawn(move || {
            let mut incoming = BufReader::new(agent_read);
            loop {
                let mut line = String::new();
                if incoming.read_line(&mut line).unwrap() == 0 {
                    break;
                }
                let message: Value = serde_json::from_str(line.trim()).unwrap();
                let id = message["id"].clone();
                let method = message["method"].as_str().unwrap_or("");
                if method == "session/prompt" {
                    let echoed = message["params"]["prompt"][0]["text"].as_str().unwrap();
                    let update = json!({
                        "jsonrpc": "2.0",
                        "method": "session/update",
                        "params": {"update": {
                            "sessionUpdate": "agent_message_chunk",
                            "content": {"type": "text", "text": echoed}
                        }}
                    });
                    writeln!(agent_write, "{update}").unwrap();
                }
                let result = match method {
                    "initialize" => {
                        json!({"protocolVersion": 1, "agentCapabilities": {"loadSession": true}, "authMethods": [{"id": "cached_token"}]})
                    }
                    "session/new" => json!({"sessionId": "sess-1"}),
                    "session/prompt" => json!({"stopReason": "end_turn"}),
                    _ => json!({}),
                };
                let response = json!({"jsonrpc": "2.0", "id": id, "result": result});
                writeln!(agent_write, "{response}").unwrap();
                if method == "session/prompt" {
                    break;
                }
            }
        });
        speak(
            BufReader::new(client_read),
            client_write,
            session,
            text,
            Duration::from_secs(2),
        )
        .unwrap()
    }
}

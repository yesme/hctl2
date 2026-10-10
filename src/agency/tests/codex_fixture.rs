//! Test double for the Codex app-server subset. Not a harness and not installed.
//! `CODEX_HOME/mode` selects ok, mismatch, foreign-turn, foreign-item, interrupt,
//! or resume-fail. Rollouts land under `$CODEX_HOME/sessions`.
mod write_probe;
use serde_json::{Value, json};
use std::{
    env, fs,
    io::{Read, Write},
    net::TcpListener,
    os::unix::net::UnixListener,
    path::PathBuf,
    process,
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--version") {
        println!("codex-cli 0.160.1");
        return;
    }
    if args.iter().any(|arg| arg == "resume") {
        loop {
            std::thread::park();
        }
    }
    let listen = args
        .windows(2)
        .find(|pair| pair[0] == "--listen")
        .map(|pair| pair[1].as_str());
    let Some(listen) = listen else {
        eprintln!("codex fixture expected --version, resume, or app-server --listen");
        process::exit(2);
    };
    if let Some(address) = listen.strip_prefix("ws://") {
        assert!(
            args.windows(2)
                .any(|p| p == ["--ws-auth", "capability-token"])
        );
        for stream in TcpListener::bind(address).unwrap().incoming().flatten() {
            serve(stream);
        }
    } else {
        let path = listen.strip_prefix("unix://").expect("native listen URL");
        let _ = fs::remove_file(path);
        let listener = UnixListener::bind(path).unwrap();
        for stream in listener.incoming().flatten() {
            serve(stream);
        }
    }
}

fn mode() -> String {
    env::var("CODEX_HOME")
        .ok()
        .and_then(|home| fs::read_to_string(PathBuf::from(home).join("mode")).ok())
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "ok".into())
}

fn home_file(name: &str) -> Option<PathBuf> {
    env::var("CODEX_HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(name))
}

fn write_json(name: &str, value: &Value) {
    let Some(path) = home_file(name) else {
        return;
    };
    let _ = fs::write(path, value.to_string());
}

fn mark(name: &str) {
    let Some(path) = home_file(name) else {
        return;
    };
    let _ = fs::write(path, b"1");
}

fn write_rollout(thread: &str, text: &str) {
    let Some(root) = home_file("sessions/fixture") else {
        return;
    };
    let _ = fs::create_dir_all(&root);
    let stored = if mode() == "mismatch" {
        format!("{text}NOT")
    } else {
        text.to_owned()
    };
    let line =
        json!({"payload":{"type":"message","content":[{"type":"input_text","text":stored}]}});
    let _ = fs::write(
        root.join(format!("rollout-{thread}.jsonl")),
        format!("{line}\n"),
    );
}

fn serve(mut stream: impl Read + Write) {
    if upgrade(&mut stream).is_err() {
        return;
    }
    let mut buf = Vec::new();
    let mut thread = None;
    let mut seq = 0u32;
    loop {
        let text = match read_frame(&mut stream, &mut buf) {
            Ok(text) => text,
            Err(_) => return,
        };
        let Ok(msg) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let Some(id) = msg.get("id").cloned() else {
            continue;
        };
        let method = msg["method"].as_str().unwrap_or("");
        match method {
            "initialize" => send_result(&mut stream, &id, json!({})),
            "thread/resume" => {
                if mode() == "resume-fail" {
                    send_error(&mut stream, &id, "no such thread");
                } else if let Some(saved) = msg["params"]["threadId"].as_str() {
                    thread = Some(saved.to_owned());
                    send_result(&mut stream, &id, json!({"thread":{"id": saved}}));
                } else {
                    send_error(&mut stream, &id, "thread id missing");
                }
            }
            "thread/start" => {
                write_json("last-thread.json", &msg["params"]);
                let id_thread = format!("thr-{}-{seq}-x", process::id());
                seq += 1;
                thread = Some(id_thread.clone());
                send_result(&mut stream, &id, json!({"thread":{"id": id_thread}}));
            }
            "turn/start" => {
                write_json("last-turn.json", &msg["params"]);
                let text = msg["params"]["input"][0]["text"].as_str().unwrap_or("");
                if let Some(cwd) = msg["params"]["cwd"].as_str()
                    && let Some(probe) = write_probe::execute(
                        text,
                        std::path::Path::new(cwd),
                        msg["params"]["sandboxPolicy"]["type"] == "externalSandbox",
                    )
                {
                    write_json("write-probe.json", &probe);
                }
                if let Some(probe) = write_probe::secret_probe(text) {
                    write_json("secret-probe.json", &probe);
                }
                let thread_id = thread.clone().unwrap_or_else(|| "missing-thread".into());
                write_rollout(&thread_id, text);
                seq += 1;
                let turn_id = format!("turn-{}-{seq}-x", process::id());
                send_result(&mut stream, &id, json!({"turn":{"id": turn_id}}));
                emit_after_start(&mut stream, &turn_id);
            }
            "turn/interrupt" => {
                mark("interrupt-seen");
                let turn_id = msg["params"]["turnId"].as_str().unwrap_or("").to_owned();
                send_result(&mut stream, &id, json!({}));
                send_json(
                    &mut stream,
                    &json!({"method":"turn/completed","params":{"turn":{"id": turn_id}}}),
                );
            }
            _ => send_error(&mut stream, &id, "unknown method"),
        }
    }
}

fn emit_after_start(stream: &mut (impl Read + Write), turn_id: &str) {
    match mode().as_str() {
        "interrupt" => {
            let body = home_file("nonce")
                .and_then(|path| fs::read(path).ok())
                .unwrap_or_else(|| b"1".to_vec());
            if let Some(path) = home_file("turn-started") {
                let _ = fs::write(path, body);
            }
        }
        "foreign-turn" => {
            send_json(
                stream,
                &json!({"method":"turn/completed","params":{"turn":{"id":"other-turn","error":{"message":"other"}}}}),
            );
            send_item(stream, turn_id, "ANSWER");
            send_completed(stream, turn_id);
        }
        "foreign-item" => {
            send_item(stream, "other-turn", "WRONG");
            send_completed(stream, turn_id);
        }
        _ => {
            send_item(stream, turn_id, "ANSWER");
            send_completed(stream, turn_id);
        }
    }
}

fn send_item(stream: &mut (impl Read + Write), turn_id: &str, text: &str) {
    send_json(
        stream,
        &json!({"method":"item/completed","params":{"turnId": turn_id, "item":{"type":"agentMessage","text": text}}}),
    );
}

fn send_completed(stream: &mut (impl Read + Write), turn_id: &str) {
    send_json(
        stream,
        &json!({"method":"turn/completed","params":{"turn":{"id": turn_id}}}),
    );
}

fn send_result(stream: &mut (impl Read + Write), id: &Value, result: Value) {
    let mut msg = json!({"result": result});
    msg["id"] = id.clone();
    send_json(stream, &msg);
}

fn send_error(stream: &mut (impl Read + Write), id: &Value, message: &str) {
    let mut msg = json!({"error": {"message": message}});
    msg["id"] = id.clone();
    send_json(stream, &msg);
}

fn send_json(stream: &mut (impl Read + Write), value: &Value) {
    let _ = send_frame(stream, &value.to_string());
}

fn upgrade(stream: &mut (impl Read + Write)) -> std::io::Result<()> {
    let mut buf = Vec::new();
    let mut tmp = [0; 1024];
    while !buf.windows(4).any(|item| item == b"\r\n\r\n") {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "upgrade closed",
            ));
        }
        buf.extend_from_slice(&tmp[..n]);
        if buf.len() > 8192 {
            return Err(std::io::Error::other("upgrade too large"));
        }
    }
    stream.write_all(
        b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n",
    )
}

fn send_frame(stream: &mut (impl Read + Write), text: &str) -> std::io::Result<()> {
    let data = text.as_bytes();
    let mut frame = vec![0x81];
    if data.len() < 126 {
        frame.push(data.len() as u8);
    } else {
        frame.push(126);
        frame.extend_from_slice(&(data.len() as u16).to_be_bytes());
    }
    frame.extend_from_slice(data);
    stream.write_all(&frame)
}

fn read_frame(stream: &mut (impl Read + Write), buf: &mut Vec<u8>) -> std::io::Result<String> {
    loop {
        while buf.len() < 2 {
            fill(stream, buf)?;
        }
        let opcode = buf[0] & 0x0f;
        let masked = buf[1] & 0x80 != 0;
        let mut len = (buf[1] & 0x7f) as usize;
        let mut offset = 2;
        if len == 126 {
            while buf.len() < 4 {
                fill(stream, buf)?;
            }
            len = u16::from_be_bytes([buf[2], buf[3]]) as usize;
            offset = 4;
        } else if len == 127 {
            return Err(std::io::Error::other("frame too large"));
        }
        if masked {
            offset += 4;
        }
        while buf.len() < offset + len {
            fill(stream, buf)?;
        }
        let mut payload = buf[offset..offset + len].to_vec();
        if masked {
            let mask = buf[offset - 4..offset].to_vec();
            for (index, byte) in payload.iter_mut().enumerate() {
                *byte ^= mask[index % 4];
            }
        }
        buf.drain(..offset + len);
        if opcode == 0x9 {
            continue;
        }
        if opcode == 0x8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "closed",
            ));
        }
        return Ok(String::from_utf8_lossy(&payload).into_owned());
    }
}

fn fill(stream: &mut (impl Read + Write), buf: &mut Vec<u8>) -> std::io::Result<()> {
    let mut tmp = [0; 8192];
    let n = stream.read(&mut tmp)?;
    if n == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "socket closed",
        ));
    }
    buf.extend_from_slice(&tmp[..n]);
    Ok(())
}

//! Test double for the omp RPC subset. Not a harness and not installed.
//! The task body selects the case: FIXTURE_MISMATCH, FIXTURE_INTERRUPT,
//! FIXTURE_ERROR, FIXTURE_LOCAL, FIXTURE_LOGIN, FIXTURE_WRITE.
//! Records land in the session directory (`--session-dir`).
mod write_probe;
use serde_json::{Value, json};
use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process,
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--version") {
        println!("omp/18.11.1");
        return;
    }
    if !args.iter().any(|arg| arg == "--mode") {
        eprintln!("omp fixture expected --version or --mode rpc");
        process::exit(2);
    }
    let Some(sessions) = arg_value(&args, "--session-dir") else {
        eprintln!("omp fixture expected --session-dir");
        process::exit(2);
    };
    let sessions = PathBuf::from(sessions);
    fs::create_dir_all(&sessions).unwrap();
    record(
        &sessions,
        "argv.json",
        &json!({
            "args": args,
            "pid": process::id(),
            "home": env::var_os("HOME").map(|value| value.to_string_lossy().into_owned()),
            "profile": env::var_os("PI_CONFIG_DIR").map(|value| value.to_string_lossy().into_owned()),
            "tmpdir": env::var_os("TMPDIR").map(|value| value.to_string_lossy().into_owned()),
            "codex_home": env::var_os("CODEX_HOME").map(|value| value.to_string_lossy().into_owned()),
            "gh_token": env::var_os("GH_TOKEN").is_some(),
        }),
    );
    let cwd = env::current_dir().unwrap();
    let (id, file) = session(&sessions);
    println!(
        "{}",
        json!({"type":"ready","protocolVersion":1,"supportedProtocolVersions":[1],"maxFrameBytes":1048576})
    );
    serve(&sessions, &cwd, &id, &file)
}

fn arg_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
}

fn state(sessions: &Path) -> PathBuf {
    sessions.join("fixture-state.json")
}

fn session(sessions: &Path) -> (String, PathBuf) {
    let state = state(sessions);
    if let Ok(bytes) = fs::read(&state)
        && let Ok(value) = serde_json::from_slice::<Value>(&bytes)
        && let (Some(id), Some(file)) = (value["id"].as_str(), value["file"].as_str())
    {
        return (id.to_owned(), PathBuf::from(file));
    }
    let id = format!("session-{}-{}", process::id(), now());
    let file = sessions.join(format!("{id}.jsonl"));
    fs::write(
        &state,
        serde_json::to_vec(&json!({"id":id,"file":file})).unwrap(),
    )
    .unwrap();
    (id, file)
}

fn fresh_session(sessions: &Path) -> (String, PathBuf) {
    let id = format!("session-{}-{}", process::id(), now());
    let file = sessions.join(format!("{id}.jsonl"));
    fs::write(&file, "").unwrap();
    fs::write(
        state(sessions),
        serde_json::to_vec(&json!({"id":id,"file":file})).unwrap(),
    )
    .unwrap();
    (id, file)
}

fn now() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn message(file: &Path, role: &str, text: &str) {
    let mut body = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .unwrap();
    let line = json!({
        "type":"message",
        "message":{"role":role,"content":[{"type":"text","text":text}]}
    });
    writeln!(body, "{line}").unwrap();
}

fn record(sessions: &Path, name: &str, value: &Value) {
    let _ = fs::write(sessions.join(name), serde_json::to_vec(value).unwrap());
}

fn reply(value: Value) {
    println!("{value}");
}

fn serve(sessions: &Path, cwd: &Path, id: &str, file: &Path) -> ! {
    let mut id = id.to_owned();
    let mut file = file.to_path_buf();
    let mut last = String::from("ANSWER");
    for line in BufReader::new(std::io::stdin()).lines() {
        let line = line.unwrap();
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let command = value["type"].as_str().unwrap_or("");
        let request_id = value["id"].clone();
        match command {
            "negotiate_protocol" => reply(json!({
                "id":request_id,"type":"response","command":"negotiate_protocol",
                "success":true,"data":{"protocolVersion":1}
            })),
            "open_session" => {
                if sessions.join("resume-fail").exists() {
                    reply(json!({
                        "id":request_id,"type":"response","command":"open_session",
                        "success":false,"error":"Could not restore model fixture/model"
                    }));
                } else {
                    let resumed = file.exists();
                    if !resumed {
                        fs::write(&file, "").unwrap();
                    }
                    reply(json!({
                        "id":request_id,"type":"response","command":"open_session",
                        "success":true,
                        "data":{"cancelled":false,"resumed":resumed,"sessionId":id,
                            "sessionFile":file.display().to_string()}
                    }));
                }
            }
            "new_session" => {
                let (next_id, next_file) = fresh_session(sessions);
                id = next_id;
                file = next_file;
                reply(json!({
                    "id":request_id,"type":"response","command":"new_session",
                    "success":true,"data":{"cancelled":false}
                }));
            }
            "get_state" => {
                let count = fs::read_to_string(&file)
                    .map(|body| {
                        body.lines()
                            .filter(|line| line.contains("\"role\":\"user\""))
                            .count()
                    })
                    .unwrap_or(0);
                reply(json!({
                    "id":request_id,"type":"response","command":"get_state","success":true,
                    "data":{"sessionId":id,"sessionFile":file.display().to_string(),
                        "isSettled":true,"messageCount":count}
                }));
            }
            "get_last_assistant_text" => reply(json!({
                "id":request_id,"type":"response","command":"get_last_assistant_text",
                "success":true,"data":{"text":last}
            })),
            "set_subagent_subscription" => reply(json!({
                "id":request_id,"type":"response","command":"set_subagent_subscription",
                "success":true,"data":{"level":"events"}
            })),
            "abort" => {
                reply(json!({
                    "id":request_id,"type":"response","command":"abort",
                    "success":true,"data":{}
                }));
            }
            "prompt" => {
                let text = value["message"].as_str().unwrap_or("").to_owned();
                if text.contains("FIXTURE_LOCAL") {
                    reply(json!({
                        "id":request_id,"type":"response","command":"prompt","success":true,
                        "data":{"agentInvoked":false}
                    }));
                    continue;
                }
                if text.contains("FIXTURE_LOGIN") {
                    reply(json!({
                        "type":"extension_ui_request","id":"login-1","method":"login",
                        "providerId":"deepseek"
                    }));
                    continue;
                }
                let recorded = if text.contains("FIXTURE_MISMATCH")
                    || sessions.join("mismatch-current").exists()
                {
                    "different body".to_owned()
                } else {
                    text.clone()
                };
                message(&file, "user", &recorded);
                if let Some(probe) = write_probe::secret_probe(&text) {
                    record(sessions, "secret-probe.json", &probe);
                }
                let _ = write_probe::execute(&text, cwd, cwd.join(".git").exists());
                if text.contains("FIXTURE_INTERRUPT") {
                    fs::write(sessions.join("turn-started"), b"started").unwrap();
                    loop {
                        let mut line = String::new();
                        if std::io::stdin().read_line(&mut line).unwrap_or(0) == 0 {
                            process::exit(0);
                        }
                        let Ok(abort) = serde_json::from_str::<Value>(&line) else {
                            continue;
                        };
                        if abort["type"] == "abort" {
                            fs::write(sessions.join("interrupt-seen"), b"seen").unwrap();
                            reply(json!({
                                "id":abort["id"].clone(),"type":"response","command":"abort",
                                "success":true,"data":{}
                            }));
                            reply(json!({
                                "id":request_id,"type":"prompt_result","agentInvoked":true,
                                "status":"aborted","sessionSettled":true
                            }));
                            break;
                        }
                    }
                    continue;
                }
                last = "ANSWER".to_owned();
                message(&file, "assistant", &last);
                if text.contains("FIXTURE_ERROR") {
                    reply(json!({
                        "id":request_id,"type":"prompt_result","agentInvoked":true,
                        "status":"error","sessionSettled":true,
                        "error":{"message":"provider failed","retryable":false}
                    }));
                } else {
                    reply(json!({
                        "id":request_id,"type":"prompt_result","agentInvoked":true,
                        "status":"completed","sessionSettled":true
                    }));
                }
            }
            "" => {}
            other => reply(json!({
                "id":request_id,"type":"response","command":other,"success":false,
                "error":"unsupported fixture command"
            })),
        }
    }
    process::exit(0);
}

//! Deterministic test double for the Antigravity CLI (`agy`) stream-json subset.
//! Verifies flags, managed `HOME`, `settings.json`, `hooks.json`, multi-turn reuse,
//! subagent frame filtering, transcript recording, resume failure, and write/secret probes.
mod write_probe;
use serde_json::{Value, json};
use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process, thread,
    time::Duration,
};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--version") {
        println!("agy version 1.4.0");
        return;
    }
    let has_pair = |flag: &str, val: &str| args.windows(2).any(|w| w[0] == flag && w[1] == val);
    if !has_pair("--output-format", "stream-json")
        || !has_pair("--input-format", "stream-json")
        || !args.iter().any(|a| a == "--disable-slash-commands")
        || !args.iter().any(|a| a == "--print=")
    {
        eprintln!("agy fixture expected stream-json flags, got {args:?}");
        process::exit(2);
    }
    let home = PathBuf::from(env::var("HOME").expect("managed HOME"));
    let agy_dir = home.join(".gemini/antigravity-cli");
    let config_dir = home.join(".gemini/config");
    fs::create_dir_all(&agy_dir).unwrap();
    let resume = args
        .windows(2)
        .find(|w| w[0] == "--conversation")
        .map(|w| w[1].clone());
    if let Some(id) = &resume {
        let brain = agy_dir.join("brain").join(id);
        if agy_dir.join("resume-fail").exists() || !brain.is_dir() {
            eprintln!("conversation {id} cannot be resumed");
            process::exit(1);
        }
    }
    let count_file = agy_dir.join("spawn-count");
    let seq: u32 = fs::read_to_string(&count_file)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
        + 1;
    fs::write(&count_file, seq.to_string()).unwrap();
    let conversation_id = resume.unwrap_or_else(|| format!("conv-{}-{seq}", process::id()));
    let logs_dir = agy_dir
        .join("brain")
        .join(&conversation_id)
        .join(".system_generated/logs");
    fs::create_dir_all(&logs_dir).unwrap();
    let transcript = logs_dir.join("transcript.jsonl");
    let cwd = env::current_dir().unwrap();
    let settings: Value = fs::read_to_string(agy_dir.join("settings.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null);
    let hooks: Value = fs::read_to_string(config_dir.join("hooks.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null);
    fs::write(
        agy_dir.join("argv.json"),
        serde_json::to_vec(&json!({
            "args": args,
            "home": home,
            "cwd": cwd,
            "pid": process::id(),
            "spawn_seq": seq,
            "conversation_id": conversation_id,
            "settings": settings,
            "hooks": hooks,
        }))
        .unwrap(),
    )
    .unwrap();

    let mut stdout = std::io::stdout();
    send_frame(
        &mut stdout,
        &json!({
            "event": "init",
            "conversation_id": conversation_id,
            "init": {
                "cwd": cwd,
                "permission_mode": settings["toolPermission"],
            }
        }),
    );

    let stdin = std::io::stdin();
    let mut turn_seq = 0u32;
    for line in BufReader::new(stdin.lock()).lines() {
        let Ok(line) = line else {
            break;
        };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if msg["event"] != "user" {
            continue;
        }
        turn_seq += 1;
        let text = msg["message"]["content"].as_str().unwrap_or("");
        if text.contains("FIXTURE_LOGIN") {
            send_frame(
                &mut stdout,
                &json!({
                    "event": "result",
                    "result": {
                        "conversation_id": conversation_id,
                        "status": "ERROR",
                        "error": "failed to create run: Authentication required. Run 'agy login' first."
                    }
                }),
            );
            continue;
        }
        if text.contains("FIXTURE_INTERRUPT") {
            fs::write(agy_dir.join("turn-started"), b"1").unwrap();
            loop {
                thread::sleep(Duration::from_millis(50));
            }
        }
        let writing = settings["toolPermission"] == "always-proceed";
        if let Some(probe) = write_probe::execute(text, &cwd, writing) {
            fs::write(agy_dir.join("write-probe.json"), probe.to_string()).unwrap();
        }
        if let Some(probe) = write_probe::secret_probe(text) {
            fs::write(agy_dir.join("secret-probe.json"), probe.to_string()).unwrap();
        }
        if text.contains("FIXTURE_SUBAGENT") {
            send_frame(
                &mut stdout,
                &json!({
                    "event": "step_update",
                    "step_update": {
                        "conversation_id": "subagent-child-1",
                        "step_index": 0,
                        "state": "DONE",
                        "step_type": "agent_response",
                        "subagent_info": {
                            "conversation_id": "subagent-child-1",
                            "log_uri": "file:///tmp/subagent"
                        }
                    }
                }),
            );
            send_frame(
                &mut stdout,
                &json!({
                    "event": "result",
                    "result": {
                        "conversation_id": "subagent-child-1",
                        "status": "ERROR",
                        "error": "subagent error must not end parent turn"
                    }
                }),
            );
        }
        let recorded = if text.contains("FIXTURE_MISMATCH") {
            format!("{text}TAMPERED")
        } else {
            text.to_owned()
        };
        append_transcript(&transcript, turn_seq, &recorded);
        send_frame(
            &mut stdout,
            &json!({
                "event": "step_update",
                "step_update": {
                    "conversation_id": conversation_id,
                    "step_index": turn_seq,
                    "state": "DONE",
                    "step_type": "agent_response",
                    "tool_calls": [
                        {
                            "name": "view_file",
                            "arguments": {"path": "README.md"}
                        }
                    ]
                }
            }),
        );
        send_frame(
            &mut stdout,
            &json!({
                "event": "result",
                "result": {
                    "conversation_id": conversation_id,
                    "status": "SUCCESS",
                    "response": format!("AGY_ANSWER:{}:{turn_seq}", process::id()),
                    "num_turns": 1
                }
            }),
        );
    }
}

fn append_transcript(path: &Path, step: u32, text: &str) {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();
    let entry = json!({
        "step_index": step,
        "source": "USER_EXPLICIT",
        "type": "USER_INPUT",
        "status": "DONE",
        "content": format!("<USER_REQUEST>\n{text}\n</USER_REQUEST>\n<ADDITIONAL_METADATA>\nfixture\n</ADDITIONAL_METADATA>")
    });
    writeln!(file, "{entry}").unwrap();
    file.sync_all().unwrap();
}

fn send_frame(stdout: &mut impl Write, value: &Value) {
    let _ = writeln!(stdout, "{value}");
    let _ = stdout.flush();
}

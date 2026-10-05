//! Test-only structured turn source. Not a harness or an advertised profession.
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc,
    time::Duration,
};

fn write(root: &Path, name: &str, value: Value) {
    fs::write(root.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version") {
        println!("2.1.289 (Claude Code)");
        return;
    }
    let option = |name: &str| args.windows(2).find(|a| a[0] == name).map(|a| a[1].clone());
    assert_eq!(option("--tools").as_deref(), Some(""));
    assert!(args.iter().any(|a| a == "--restricted"));
    assert!(std::env::var_os("CLAUDE_CONFIG_DIR").is_none());
    assert!(std::env::var_os("USER").is_some());
    let plugin = PathBuf::from(option("--plugin-dir").unwrap());
    let root = plugin.parent().unwrap();
    let resume = option("--resume");
    let id = if root.join("fail-resume").exists() {
        format!("fixture-{}", std::process::id())
    } else {
        resume.unwrap_or_else(|| format!("fixture-{}", std::process::id()))
    };
    write(root, "session.json", json!({"session":id}));
    fs::write(root.join("harness.pid"), std::process::id().to_string()).unwrap();
    let mut stream = UnixStream::connect(std::env::var_os("HERDR_SOCKET_PATH").unwrap()).unwrap();
    let request = json!({"id":"fixture-start","protocol":22,"method":"pane.report_agent_session","params":{
        "pane_id":std::env::var("HERDR_PANE_ID").unwrap(), "source":"herdr:claude","agent":"claude",
        "seq":1,"agent_session_id":id,"session_start_source":"startup"
    }});
    writeln!(stream, "{request}").unwrap();
    let mut ack = String::new();
    BufReader::new(stream).read_line(&mut ack).unwrap();
    assert!(
        Command::new("stty")
            .args(["raw", "-echo"])
            .status()
            .unwrap()
            .success()
    );
    print!("\x1b]0;✳ Claude Code\x07\r\n❯ ");
    std::io::stdout().flush().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for byte in BufReader::new(std::io::stdin()).bytes() {
            if tx.send(byte.unwrap()).is_err() {
                break;
            }
        }
    });
    let mut input = Vec::new();
    let mut paste = false;
    let mut sequence = Vec::new();
    let mut count = 0;
    let mut pending: Option<(Value, String)> = None;
    let mut uninterruptible = false;
    loop {
        let byte = match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(b) => b,
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if sequence == [27] {
                    sequence.clear();
                    if !uninterruptible && let Some((job, turn)) = pending.take() {
                        write(
                            root,
                            "returned.json",
                            json!({"session":id,"job":job["id"],"digest":job["digest"],"turnId":turn,"reason":"aborted","isAborted":true,"answer":""}),
                        );
                    }
                }
                continue;
            }
        };
        if byte == 27 || !sequence.is_empty() {
            sequence.push(byte);
            if sequence == b"\x1b[200~" {
                paste = true;
                sequence.clear();
            } else if sequence == b"\x1b[201~" {
                paste = false;
                sequence.clear();
            }
            continue;
        }
        if !paste && (byte == b'\r' || byte == b'\n') {
            if input.is_empty() {
                continue;
            }
            let text = String::from_utf8(std::mem::take(&mut input)).unwrap();
            let job: Value =
                serde_json::from_slice(&fs::read(root.join("job.json")).unwrap()).unwrap();
            count += 1;
            let turn = format!("turn-{count}");
            write(
                root,
                "started.json",
                json!({"session":id,"job":job["id"],"digest":job["digest"],"turnId":turn}),
            );
            fs::write(root.join("delivered.txt"), &text).unwrap();
            if text.trim() == "wait" || text.trim() == "uninterruptible" {
                uninterruptible = text.trim() == "uninterruptible";
                pending = Some((job, turn));
                continue;
            }
            let reason = if text.trim() == "error" {
                "error"
            } else {
                "answer"
            };
            let answer = if text.trim() == "error" {
                "fixture turn failed"
            } else {
                text.trim()
            };
            write(
                root,
                "returned.json",
                json!({"session":id,"job":job["id"],"digest":job["digest"],"turnId":turn,"reason":reason,"isAborted":false,"answer":answer}),
            );
            print!("\r\n{answer}\r\n❯ ");
            std::io::stdout().flush().unwrap();
        } else {
            input.push(byte);
        }
    }
}

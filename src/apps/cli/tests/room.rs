//! Real CLI, daemon and packaged Tuwunel; the Project package adds native B1.
#[path = "room/agency.rs"]
mod agency;
#[path = "room/github.rs"]
mod github;
#[path = "room/project.rs"]
mod project;
use chat::{Server, key, main_room, reference};
use serde_json::{Value, json};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc, LazyLock, Mutex, Weak,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use store::{
    Actor, ActorSource, Expected, ProjectSettings, Record, RecordData, Scope, Store, TrustedActor,
    Version,
};

struct Fixture {
    root: PathBuf,
    payload: PathBuf,
    /// Keeps the shared payload this fixture linked from alive; the last holder's drop removes it.
    source: Arc<SharedPayload>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.run(&["stop"]);
        let _ = Command::new(self.payload.join("bin/hctl2-services"))
            .env("HCTL2_STATE_ROOT", self.root.join("services"))
            .arg("stop")
            .output();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
impl Fixture {
    fn packaged(name: &str) -> (Self, u16) {
        let fixture = Self::unpacked(name);
        let port = fixture.isolate_ports();
        (fixture, port)
    }
    fn isolate_ports(&self) -> u16 {
        // Every packaged service gets a port of its own: these fixtures run on the same
        // machine as the packaged lifecycle test, and the previous value-based rewrite
        // ("6167", "3000") silently stopped matching when a packaged default changed —
        // Gitea's is 3001 now — so two tests could fight over one port. Rewrite the
        // assignments by name instead.
        let versions = self.payload.join("lib/hctl2/services/versions.sh");
        let text = std::fs::read_to_string(&versions).unwrap();
        let mut ports = std::collections::HashMap::new();
        let isolated = text
            .lines()
            .map(|line| {
                let Some((name, value)) = line
                    .strip_prefix("readonly ")
                    .and_then(|rest| rest.split_once("=\""))
                else {
                    return line.to_owned();
                };
                if !name.ends_with("_PORT") || !value.ends_with('"') {
                    return line.to_owned();
                }
                let port = free_port();
                ports.insert(name.to_owned(), port);
                format!("readonly {name}=\"{port}\"")
            })
            .collect::<Vec<_>>()
            .join("\n");
        // Replace the file instead of writing through it: a fixture payload is
        // a hardlink tree over the shared extraction, and an in-place write
        // would reach the copy every other fixture links from. Renaming a new
        // file over the entry leaves the shared inode alone.
        let replacement = self.payload.join("lib/hctl2/services/versions.sh.tmp");
        std::fs::write(&replacement, format!("{isolated}\n")).unwrap();
        std::fs::rename(&replacement, &versions).unwrap();
        ports
            .remove("TUWUNEL_PORT")
            .expect("packaged versions.sh must define TUWUNEL_PORT")
    }
    fn unpacked(name: &str) -> Self {
        // The shared payload comes first: a failed initialization must not leave a fixture root
        // (or half a package) behind.
        let source = shared_payload();
        let root = std::env::temp_dir().join(format!("hctl-{name}-cli-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let extract = root.join("install");
        link_tree(&source.dir, &extract);
        let payload = find(&extract, |p| p.join("bin/hctl2-services").is_file()).unwrap();
        Self {
            root,
            payload,
            source,
        }
    }
    fn run(&self, args: &[&str]) -> (bool, Value) {
        let (ok, stdout, stderr) = self.run_raw(true, args);
        (
            ok,
            serde_json::from_slice(&stdout).unwrap_or_else(|_| {
                json!({"stdout":String::from_utf8_lossy(&stdout),"stderr":String::from_utf8_lossy(&stderr)})
            }),
        )
    }
    /// Raw bytes, for byte-exact output checks and the human default mode.
    fn run_raw(&self, json: bool, args: &[&str]) -> (bool, Vec<u8>, Vec<u8>) {
        let mut command = Command::new(
            std::env::var("CARGO_BIN_EXE_hctl2")
                .expect("CARGO_BIN_EXE_hctl2 must be set to run this test"),
        );
        command
            .env_remove("HCTL2_PROCESS_COMPOSE_BIN")
            .env("HCTL2_INSTALL_ROOT", &self.payload)
            // 第 9 包验收第 4 条的测试缝：control 从这里找标记文件，决定尝试停在哪个状态。
            // 没有标记时行为不变；每个夹具的根不同，别的用例不受影响。
            .env("HCTL2_TEST_SEAM_ROOT", self.root.join("test-seams"))
            .env(
                "HCTL2_CONTROL_BIN",
                std::env::var("CARGO_BIN_EXE_hctl2-control")
                    .expect("CARGO_BIN_EXE_hctl2-control must be set to run this test"),
            )
            .arg("--root")
            .arg(self.root.to_str().unwrap());
        if json {
            command.arg("--json");
        }
        let out = command.args(args).output().unwrap();
        (out.status.success(), out.stdout, out.stderr)
    }
    fn query(&self, kind: &str, input: Value) -> Value {
        let path = self.root.join("query.json");
        std::fs::write(&path, input.to_string()).unwrap();
        let (ok, value) = self.run(&["room", kind, "--input", path.to_str().unwrap()]);
        assert!(ok, "{kind}: {value}");
        value
    }
    fn command(&self, kind: &str, key: &str, input: &Value) -> (bool, Value) {
        self.command_ns("room", kind, key, input)
    }
    fn command_ns(&self, namespace: &str, kind: &str, key: &str, input: &Value) -> (bool, Value) {
        let path = self.root.join("command.json");
        std::fs::write(&path, input.to_string()).unwrap();
        let p = path.to_str().unwrap();
        let (ok, preview) = self.run(&[namespace, kind, "--key", key, "--input", p]);
        if !ok {
            return (ok, preview);
        }
        self.run(&[
            namespace,
            kind,
            "--key",
            key,
            "--input",
            p,
            "--preview-token",
            preview["preview_token"].as_str().unwrap(),
        ])
    }
    fn accepted(&self, kind: &str, key: &str, input: &Value) -> Value {
        let (ok, value) = self.command(kind, key, input);
        assert!(ok, "{kind}: {value}");
        value
    }
    fn show(&self, project: &str, room: &str) -> Value {
        let (ok, value) = self.run(&["room", "show", project, room]);
        assert!(ok, "{value}");
        value
    }
}
fn free_port() -> u16 {
    let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = socket.local_addr().unwrap().port();
    drop(socket);
    port
}

// One archive extraction per overlapping batch of fixtures: the first fixture that finds no
// live payload extracts one, every other fixture links its own tree from a live copy, and the
// last fixture holding it returns the disk. Nothing outlives the test binary and a failed
// extraction leaves no half package behind.
struct SharedPayload {
    dir: PathBuf,
}
impl Drop for SharedPayload {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
static LIVE_PAYLOADS: LazyLock<Mutex<Vec<Weak<SharedPayload>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
/// Payload directories are numbered by a process-wide counter, never by the number of live
/// copies: the last holder of a copy may still be deleting its tree while another test creates
/// the next one, and a count-derived name would hand the new copy the old one's directory.
static NEXT_PAYLOAD: AtomicUsize = AtomicUsize::new(0);
fn next_payload_dir() -> PathBuf {
    let n = NEXT_PAYLOAD.fetch_add(1, Ordering::Relaxed) + 1;
    std::env::temp_dir().join(format!("hctl-payload-{}-{n}", std::process::id()))
}

fn shared_payload() -> Arc<SharedPayload> {
    // A panicking test thread must not poison the registry for the rest of the binary.
    let mut live = LIVE_PAYLOADS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    live.retain(|weak| weak.strong_count() > 0);
    if let Some(payload) = live.iter().find_map(Weak::upgrade) {
        return payload;
    }
    let archive = find(
        Path::new(
            &std::env::var("HCTL2_TEST_DEPENDENCY_PACKAGE")
                .expect("HCTL2_TEST_DEPENDENCY_PACKAGE must be set to run this test"),
        ),
        |p| {
            p.file_name()
                .unwrap()
                .to_str()
                .is_some_and(|n| n.ends_with(".tar.zst") && !n.contains("-sources"))
        },
    )
    .unwrap();
    let dir = next_payload_dir();
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir(&dir).unwrap();
    if let Err(detail) = unpack_payload(&zstd_tool(), &archive, &dir) {
        let _ = std::fs::remove_dir_all(&dir);
        panic!("{detail}");
    }
    let payload = Arc::new(SharedPayload { dir });
    println!("shared payload extracted: {}", payload.dir.display());
    live.push(Arc::downgrade(&payload));
    payload
}

fn zstd_tool() -> PathBuf {
    Path::new(
        &std::env::var("HCTL2_ZSTD_ROOT").expect("HCTL2_ZSTD_ROOT must be set to run this test"),
    )
    .join("bin/zstd")
}

/// Stream the archive through the pinned zstd into tar. `--ignore-zeros` keeps tar reading to the
/// real end of the stream: an archive's end-of-archive block is not the end of the zstd frame, and
/// a tar that stops there closes the pipe on a decoder with padding left to write, which kills
/// zstd with SIGPIPE (141) on a perfectly good archive (Codex, #416 T1). Both statuses are
/// required; a failed run removes what it half-extracted.
fn unpack_payload(zstd: &Path, archive: &Path, extract: &Path) -> std::result::Result<(), String> {
    let mut decompress = Command::new(zstd)
        .arg("-dc")
        .arg(archive)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("spawning {}: {error}", zstd.display()))?;
    let mut unpack = Command::new("tar")
        .args(["-x", "--ignore-zeros", "-f", "-", "-C"])
        .arg(extract)
        .stdin(Stdio::from(decompress.stdout.take().unwrap()))
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("spawning tar: {error}"))?;
    // tar's stderr is drained while both processes run: it is a pipe, and a tar that writes more
    // than fits in it (a tree of entries it cannot create, say) blocks on that write, stops
    // reading the archive, and then zstd blocks on its own stdout while the parent waits on it.
    let tar_stderr = unpack.stderr.take().unwrap();
    let drain = std::thread::spawn(move || {
        let mut stderr = tar_stderr;
        let mut text = Vec::new();
        std::io::Read::read_to_end(&mut stderr, &mut text).ok();
        text
    });
    let decompressed = decompress.wait_with_output().unwrap();
    let unpacked = unpack.wait().unwrap();
    let tar_stderr = drain.join().unwrap();
    if decompressed.status.success() && unpacked.success() {
        return Ok(());
    }
    let _ = std::fs::remove_dir_all(extract);
    Err(format!(
        "unpacking {} failed: zstd {:?} {}, tar {:?} {} (cwd {})",
        archive.display(),
        decompressed.status,
        String::from_utf8_lossy(&decompressed.stderr).trim(),
        unpacked,
        String::from_utf8_lossy(&tar_stderr).trim(),
        std::env::current_dir().unwrap_or_default().display(),
    ))
}

// Hardlink the shared payload into a fixture's own tree: a payload tree costs
// directory entries instead of a second copy, while each fixture still owns its
// `lib/hctl2/services/versions.sh` entry. Files the fixture changes are
// replaced, never rewritten in place, so the shared copy stays intact.
fn link_tree(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = target.join(entry.file_name());
        let file_type = entry.file_type().unwrap();
        if file_type.is_symlink() {
            std::os::unix::fs::symlink(std::fs::read_link(&from).unwrap(), &to).unwrap();
        } else if file_type.is_dir() {
            link_tree(&from, &to);
        } else {
            std::fs::hard_link(&from, &to).unwrap();
        }
    }
}

fn find(root: &Path, predicate: impl Fn(&Path) -> bool + Copy) -> Option<PathBuf> {
    for entry in std::fs::read_dir(root).ok()? {
        let p = entry.ok()?.path();
        if predicate(&p) {
            return Some(p);
        }
        if p.is_dir()
            && let Some(p) = find(&p, predicate)
        {
            return Some(p);
        }
    }
    None
}
/// 小活 T 的核验用例（Codex #416 T1）：发布包的 tar 后面还有零填充，那是解包必须读过去的
/// 常态——tar 的归档结束块不是 zstd 帧的结尾；坏校验和的帧必须失败，且不留半份。
#[test]
fn unpack_payload_reads_past_the_end_of_archive_and_rejects_a_bad_frame() {
    let work = std::env::temp_dir().join(format!("hctl-unpack-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();
    let source = work.join("source");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("marker.txt"), b"payload\n").unwrap();
    let tar = work.join("padded.tar");
    assert!(
        Command::new("tar")
            .arg("-cf")
            .arg(&tar)
            .arg("-C")
            .arg(&source)
            .arg(".")
            .status()
            .unwrap()
            .success()
    );
    let mut padded = std::fs::OpenOptions::new().append(true).open(&tar).unwrap();
    padded.write_all(&vec![0u8; 1 << 20]).unwrap();
    drop(padded);
    let archive = work.join("padded.tar.zst");
    assert!(
        Command::new(zstd_tool())
            .args(["-q", "-f"])
            .arg(&tar)
            .arg("-o")
            .arg(&archive)
            .status()
            .unwrap()
            .success()
    );
    let extract = work.join("extract");
    std::fs::create_dir(&extract).unwrap();
    unpack_payload(&zstd_tool(), &archive, &extract).expect("a padded archive must unpack");
    assert!(extract.join("marker.txt").is_file());
    // 坏校验和：解码器返回非零，解包失败，半份也不留。
    let mut frame = std::fs::read(&archive).unwrap();
    let last = frame.len() - 1;
    frame[last] ^= 0xff;
    let bad = work.join("bad.tar.zst");
    std::fs::write(&bad, &frame).unwrap();
    let half = work.join("half");
    std::fs::create_dir(&half).unwrap();
    assert!(unpack_payload(&zstd_tool(), &bad, &half).is_err());
    assert!(
        !half.exists(),
        "a failed unpack must take the half-extracted tree with it"
    );
    let _ = std::fs::remove_dir_all(&work);
}

/// 小活 T 的核验用例（Codex #416 P2）：一份副本在删自己的目录时，下一份可以已经在建——
/// 两次解包的名字必须不同，删掉前一份不能碰到后一份。名字来自进程内递增编号，不靠 sleep。
#[test]
fn shared_payload_directories_do_not_collide_across_a_release() {
    let first = next_payload_dir();
    std::fs::create_dir_all(&first).unwrap();
    let released = SharedPayload { dir: first.clone() };
    let second = next_payload_dir();
    std::fs::create_dir_all(&second).unwrap();
    assert_ne!(first, second, "two payloads must not share a directory");
    drop(released);
    assert!(!first.exists(), "the released copy takes its own tree");
    assert!(second.exists(), "the next copy must survive the old drop");
    let _ = std::fs::remove_dir_all(&second);
}

/// 小活 T 的核验用例（Codex #416 P2）：tar 的错误输出超过管道容量时（这里解包目录不可写，
/// 每个条目都写不进去），父进程必须同时排空两端 stderr——只等 zstd 会让 tar 卡在写 stderr、
/// zstd 卡在写 stdout。失败照旧如实返回，两端的退出状态都不放宽，半份也不留。
/// 以 root 跑时权限拦不住 tar，本用例跳过；CI 的 Linux/macOS 执行器都不是 root。
#[test]
fn unpack_payload_survives_more_tar_stderr_than_a_pipe_holds() {
    use std::os::unix::fs::PermissionsExt;
    let uid = Command::new("id").arg("-u").output().unwrap();
    if String::from_utf8_lossy(&uid.stdout).trim() == "0" {
        println!("skipping: root writes into a read-only directory");
        return;
    }
    let work = std::env::temp_dir().join(format!("hctl-unpack-noisy-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();
    let source = work.join("source");
    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(source.join("payload")).unwrap();
    for n in 0..3000 {
        std::fs::write(source.join("payload").join(format!("f{n:04}")), b"x\n").unwrap();
    }
    let tar = work.join("noisy.tar");
    assert!(
        Command::new("tar")
            .arg("-cf")
            .arg(&tar)
            .arg("-C")
            .arg(&source)
            .arg(".")
            .status()
            .unwrap()
            .success()
    );
    let archive = work.join("noisy.tar.zst");
    assert!(
        Command::new(zstd_tool())
            .args(["-q", "-f"])
            .arg(&tar)
            .arg("-o")
            .arg(&archive)
            .status()
            .unwrap()
            .success()
    );
    let extract = work.join("extract");
    std::fs::create_dir(&extract).unwrap();
    std::fs::set_permissions(&extract, std::fs::Permissions::from_mode(0o555)).unwrap();
    let failed = unpack_payload(&zstd_tool(), &archive, &extract);
    std::fs::set_permissions(&extract, std::fs::Permissions::from_mode(0o755)).ok();
    assert!(
        failed.is_err(),
        "tar cannot write into a read-only directory"
    );
    assert!(
        !extract.exists(),
        "a failed unpack must take the half-extracted tree with it"
    );
    let _ = std::fs::remove_dir_all(&work);
}

/// `shared_payload_leaves_nothing_behind_after_exit_or_a_failed_init` 的子进程目标：只建一个
/// 夹具（共享解包加硬链接），然后正常退出。
#[test]
fn packaged_fixture_probe() {
    let (fixture, _) = Fixture::packaged("probe");
    println!("probe payload: {}", fixture.payload.display());
}

/// 小活 T 的核验用例（Codex #416 T2）：共享副本由夹具的生命周期持有——正常退出后临时目录里
/// 不留 `hctl-payload-*`；初始化中途失败也不留半份，且失败本身如实报出来。
#[test]
fn shared_payload_leaves_nothing_behind_after_exit_or_a_failed_init() {
    let probe = std::env::temp_dir().join(format!("hctl-exit-probe-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&probe);
    std::fs::create_dir_all(&probe).unwrap();
    let leftovers = || {
        std::fs::read_dir(&probe)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("hctl-"))
            .count()
    };
    let child = |package: Option<&Path>| {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "packaged_fixture_probe", "--nocapture"])
            .env("TMPDIR", &probe);
        if let Some(package) = package {
            command.env("HCTL2_TEST_DEPENDENCY_PACKAGE", package);
        }
        command.output().unwrap()
    };
    let normal = child(None);
    assert!(
        normal.status.success(),
        "{}",
        String::from_utf8_lossy(&normal.stderr)
    );
    assert_eq!(
        leftovers(),
        0,
        "a normal exit must take the shared payload with it"
    );
    let bad_package = probe.join("bad-package");
    std::fs::create_dir(&bad_package).unwrap();
    std::fs::write(
        bad_package.join("hctl2-0.0.0-bad.tar.zst"),
        b"not a zstd frame",
    )
    .unwrap();
    let failed = child(Some(&bad_package));
    assert!(
        !failed.status.success(),
        "a bad package must fail the probe"
    );
    assert_eq!(
        leftovers(),
        0,
        "a failed init must not leave half a payload"
    );
    let _ = std::fs::remove_dir_all(&probe);
}

#[test]
fn room_cli_native_lifecycle_preview_draft_replay_and_recovery() {
    let (f, port) = Fixture::packaged("room");
    let root = f.root.clone();
    let mut store = Store::open(&root).unwrap();
    let server = Server {
        binding: store::Reference {
            key: key(
                Scope::Control,
                "chat_server",
                &format!("hctl2-{}", store.control_id()),
            ),
            version: Version::State(1),
        },
        url: format!("http://127.0.0.1:{port}"),
        server_name: "hctl2.localhost".into(),
        sender: "@hctl2_control:hctl2.localhost".into(),
    };
    let actor = TrustedActor(Actor {
        principal: "fixture".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![
            Scope::Control,
            Scope::Project("A".into()),
            Scope::Project("B".into()),
        ],
        authority: None,
    });
    let mut mains = vec![];
    for project in ["A", "B"] {
        let data = RecordData::Project {
            repo_id: "same-repo".into(),
            settings: ProjectSettings {
                publish_review_requires_confirmation: true,
                selection_policy: json!({}),
            },
            archived: false,
        };
        let record = Record {
            key: key(Scope::Project(project.into()), "project", project),
            version: 1,
            revision_digest: foundation::canonical_json_sha256(
                &serde_json::to_value(&data).unwrap(),
            )
            .unwrap(),
            data,
            sources: vec![],
            materials: vec![],
        };
        let (records, effect) = main_room(
            &record,
            server.clone(),
            project.into(),
            &format!("seed-{project}"),
        )
        .unwrap();
        let command = store::Command {
            command_id: project.into(),
            idempotency_key: project.into(),
            actor: actor.0.clone(),
            target: record.key.clone(),
            expected: Expected::Absent,
            binding: reference(&record),
            input_digest: store::Command::digest_input("fixture", &json!({})).unwrap(),
            operation: "fixture".into(),
            input: json!({}),
        };
        store
            .submit(store.generation(), &actor, &command, None, |tx| {
                tx.put(&record)?;
                for r in &records {
                    tx.put(r)?;
                }
                tx.enqueue_effect(&effect)?;
                Ok(json!({}))
            })
            .unwrap();
        if project == "B" {
            // Crash window: committed/marked unknown, but no native createRoom sent yet.
            store
                .begin_effect(store.generation(), &effect.intent_id)
                .unwrap();
        }
        mains.push((records[0].key.id.clone(), effect.intent_id));
    }
    drop(store);
    // Start the native service first: control has to load its new AppService registration.
    assert!(
        Command::new(f.payload.join("bin/hctl2-services"))
            .env("HCTL2_STATE_ROOT", root.join("services"))
            .args(["start", "--no-wait", "tuwunel"])
            .output()
            .unwrap()
            .status
            .success()
    );
    for _ in 0..100 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(f.run(&["start", "--secret-backend", "user-file"]).0);
    for (project, (room, effect)) in ["A", "B"].into_iter().zip(&mains) {
        let input = json!({"project_id":project,"effect_id":effect});
        let mut ready = false;
        let mut last = json!(null);
        for _ in 0..60 {
            let (ok, value) = f.command("resume", &format!("resume-{project}"), &input);
            if ok && value["state"] == "confirmed" {
                ready = true;
                break;
            }
            last = value;
            std::thread::sleep(Duration::from_millis(200));
        }
        assert!(
            ready,
            "room creation: {last}; status: {}; log: {}; room: {}",
            f.run(&["services", "status"]).1,
            std::fs::read_to_string(root.join("control.log")).unwrap_or_default(),
            f.show(project, room)
        );
    }
    let a = &mains[0].0;
    let main = f.show("A", a);
    let version = main["binding"]["version"].as_i64().unwrap();
    let send =
        json!({"project_id":"A","room_id":a,"version":version,"body":"日本語与中文 é / é；未定"});
    let sent = f.accepted("send", "send-1", &send);
    assert_eq!(
        f.accepted("send", "send-1", &send)["receipt"],
        sent["receipt"]
    );
    let event = sent["receipt"]["event_id"].as_str().unwrap();
    let draft = f.query("draft",json!({"project_id":"A","project_version":1,"origin":{"kind":"room","room_id":a,"binding_version":version},"selection":{"kind":"events","event_ids":[event,"$missing"]}}));
    assert_eq!(draft["automatic_summary"], "not_configured");
    assert_eq!(draft["fragments"][0]["excerpt"], send["body"]);
    assert_eq!(draft["unread"].as_array().unwrap().len(), 1);
    assert_eq!(
        f.run(&["room", "list"]).1["rooms"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let source = draft["fragments"][0]["source"].clone();
    let topic = json!({"project_id":"A","project_version":1,"name":"独立话题","origin":{"kind":"room","room_id":a,"binding_version":version},"brief":{"context_and_goal":"人的去敏与补写","settled_facts_and_reasons":[],"disagreements_and_questions":["未定"],"constraints_and_materials":[],"sources":[source]},"participants":[],"roster_confirmed":true});
    let created = f.accepted("create-topic", "topic-1", &topic);
    assert_eq!(
        f.accepted("create-topic", "topic-1", &topic)["room_id"],
        created["room_id"]
    );
    let topic_id = created["room_id"].as_str().unwrap();
    assert_eq!(created["state"], "confirmed");
    let topic_show = f.show("A", topic_id);
    assert_eq!(topic_show["hierarchy"]["parents"][0]["room_id"], *a);
    let topic_version = topic_show["binding"]["version"].as_i64().unwrap();
    let topic_message = f.accepted("send", "topic-send", &json!({"project_id":"A","room_id":topic_id,"version":topic_version,"body":"Topic 里的新方向"}));
    let topic_event = topic_message["receipt"]["event_id"].as_str().unwrap();
    let (ok, topic_event_data) = f.run(&["room", "event", "A", topic_id, topic_event]);
    assert!(ok, "{topic_event_data}");
    let nested_input = json!({"project_id":"A","project_version":1,"name":"Topic 下的话题","origin":{"kind":"room","room_id":topic_id,"binding_version":topic_version},"brief":{"context_and_goal":"新方向","settled_facts_and_reasons":[],"disagreements_and_questions":[],"constraints_and_materials":[],"sources":[topic_event_data["source"]["source"]]},"participants":[],"roster_confirmed":true});
    let nested = f.accepted("create-topic", "nested-topic", &nested_input);
    assert_eq!(nested["state"], "confirmed");
    let nested_id = nested["room_id"].as_str().unwrap();
    let before_move = f.show("A", nested_id);
    assert_eq!(before_move["hierarchy"]["parents"][0]["room_id"], topic_id);
    let cyclic_move = root.join("cyclic-move.json");
    std::fs::write(&cyclic_move, json!({"project_id":"A","room_id":a,"binding_version":version,"parent_room_id":nested_id,"parent_binding_version":before_move["binding"]["version"],"old_space_ids":[]}).to_string()).unwrap();
    let (ok, rejected) = f.run(&["room", "reparent", "--input", cyclic_move.to_str().unwrap()]);
    assert!(!ok, "HCTL must not introduce a native cycle: {rejected}");
    assert_eq!(rejected["error"]["code"], "CHAT_HIERARCHY_CYCLE");
    assert_eq!(f.show("A", a)["hierarchy"]["parents"], json!([]));
    assert!(f.show("A", nested_id)["hierarchy"]["carrier_space_id"].is_null());
    let (ok, _) = f.run(&["room", "hierarchy", "A", nested_id]);
    assert!(ok);
    // The wrapper is native readback, never part of the Room binding.
    let topic_space = f.show("A", topic_id)["hierarchy"]["carrier_space_id"].clone();
    let moved = f.query("reparent", json!({"project_id":"A","room_id":nested_id,"binding_version":before_move["binding"]["version"],"parent_room_id":a,"parent_binding_version":version,"old_space_ids":[topic_space]}));
    assert_eq!(moved["state"], "confirmed");
    let after_move = f.show("A", nested_id);
    assert_eq!(after_move["hierarchy"]["parents"][0]["room_id"], *a);
    assert_eq!(after_move["room"]["origin"], before_move["room"]["origin"]);
    assert_eq!(
        after_move["binding"]["version"],
        before_move["binding"]["version"]
    );
    assert_eq!(
        f.accepted("create-topic", "nested-topic", &nested_input)["room_id"],
        nested_id
    );
    assert_eq!(
        f.show("A", nested_id)["hierarchy"]["parents"][0]["room_id"],
        *a
    );
    let reply = f.accepted("send", "thread-send", &json!({"project_id":"A","room_id":topic_id,"version":topic_version,"body":"同一 Room 的讨论串","thread_root":topic_event}));
    let reply_id = reply["receipt"]["event_id"].as_str().unwrap();
    let (_, thread_source) = f.run(&["room", "event", "A", topic_id, reply_id]);
    assert_eq!(thread_source["source"]["source"]["event_id"], reply_id);
    assert!(!f.command("send", "nested-thread", &json!({"project_id":"A","room_id":topic_id,"version":topic_version,"body":"不能嵌套","thread_root":reply_id})).0);
    let range = f.query("draft", json!({"project_id":"A","project_version":1,"origin":{"kind":"room","room_id":topic_id,"binding_version":topic_version},"selection":{"kind":"range","start":topic_event,"end":reply_id}}));
    assert_eq!(range["fragments"].as_array().unwrap().len(), 2);
    let thread = f.query("draft", json!({"project_id":"A","project_version":1,"origin":{"kind":"room","room_id":topic_id,"binding_version":topic_version},"selection":{"kind":"thread","event_id":topic_event}}));
    assert_eq!(thread["fragments"].as_array().unwrap().len(), 2);
    let draft_file = root.join("query.json");
    let args = [
        "room",
        "draft",
        "--key",
        "repeat-draft",
        "--input",
        draft_file.to_str().unwrap(),
    ];
    let (ok, repeated) = f.run(&args);
    assert!(ok, "{repeated}");
    let (ok, replayed) = f.run(&args);
    assert!(ok, "{replayed}");
    assert_eq!(repeated, replayed);
    let mut leaf_input = topic.clone();
    leaf_input["name"] = json!("另一间独立 Room");
    let leaf = f.accepted("create-topic", "leaf-topic", &leaf_input);
    let leaf_id = leaf["room_id"].as_str().unwrap();
    let leaf_before = f.show("A", leaf_id);
    assert!(leaf_before["hierarchy"]["carrier_space_id"].is_null());
    let main_space = f.show("A", a)["hierarchy"]["carrier_space_id"].clone();
    let moved_to_leaf = f.query("reparent", json!({"project_id":"A","room_id":nested_id,"binding_version":before_move["binding"]["version"],"parent_room_id":leaf_id,"parent_binding_version":leaf_before["binding"]["version"],"old_space_ids":[main_space]}));
    assert_eq!(moved_to_leaf["state"], "confirmed");
    let after_leaf = f.show("A", leaf_id);
    assert!(after_leaf["hierarchy"]["carrier_space_id"].is_string());
    assert_eq!(after_leaf["binding"], leaf_before["binding"]);
    assert_eq!(
        f.show("A", nested_id)["hierarchy"]["parents"][0]["room_id"],
        leaf_id
    );
    let frozen = f.query("reference", json!({"project_id":"A","source":source}));
    let view = json!({"project_id":"A","room_id":a,"binding_version":version,"client_id":"desktop","draft":"尚未发送","read_cursor":event});
    f.query("save-view-state", view);
    let mut next_send = send.clone();
    next_send["body"] = json!("后续消息不流入 Topic");
    f.accepted("send", "send-2", &next_send);
    let (_, timeline) = f.run(&["room", "timeline", "A", topic_id]);
    assert!(
        !timeline["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["content"]["body"] == "后续消息不流入 Topic")
    );
    let closed = f.show("A", topic_id);
    assert_eq!(closed["brief"]["context_and_goal"], "人的去敏与补写");
    f.accepted(
        "close",
        "close-topic",
        &json!({"project_id":"A","room_id":topic_id,"version":closed["binding"]["version"]}),
    );
    let nested_after_close = f.show("A", nested_id);
    assert_eq!(nested_after_close["identity"]["data"]["state"], "active");
    assert_eq!(
        nested_after_close["room"]["origin"],
        before_move["room"]["origin"]
    );
    assert!(!f.command("send","closed-send",&json!({"project_id":"A","room_id":topic_id,"version":closed["binding"]["version"].as_i64().unwrap()+1,"body":"no"})).0);
    assert!(f.run(&["stop"]).0);
    std::thread::sleep(Duration::from_millis(200));
    // All disposable chat observations can disappear without losing admitted references.
    std::fs::remove_dir_all(root.join("cache")).unwrap();
    assert!(f.run(&["start", "--secret-backend", "user-file"]).0);
    for _ in 0..60 {
        if f.run(&["room", "view-state", "A", a, "desktop"]).0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(
        f.query("reference", json!({"project_id":"A","source":source})),
        frozen
    );
    let (ok, view) = f.run(&["room", "view-state", "A", a, "desktop"]);
    assert!(ok, "{view}");
    assert_eq!(view["draft"], "尚未发送");
    assert_eq!(
        f.run(&["room", "list"]).1["rooms"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    let (ok, _) = f.run(&["room", "event", "B", &mains[1].0, event]);
    assert!(!ok, "event in A must not be accepted as B's event");
    let backup = root.join("backup");
    let service_backup = root.join("service-backup");
    assert!(f.run(&["backup", "create", backup.to_str().unwrap()]).0);
    assert!(
        f.run(&["services", "backup", service_backup.to_str().unwrap()])
            .0
    );
    assert!(f.run(&["stop"]).0);
    let restored = Fixture {
        root: root.join("restored"),
        payload: f.payload.clone(),
        source: Arc::clone(&f.source),
    };
    // A new directory imports the control identity before the daemon starts, rather than
    // overwriting an unrelated initialized control. Native services restore through the CLI.
    drop(Store::restore(&restored.root, &backup).unwrap());
    assert!(restored.run(&["start", "--secret-backend", "user-file"]).0);
    let (ok, value) = restored.run(&[
        "services",
        "restore",
        service_backup.to_str().unwrap(),
        "--yes",
    ]);
    assert!(ok, "service restore: {value}");
    for _ in 0..100 {
        if restored.run(&["room", "view-state", "A", a, "desktop"]).0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(
        restored.query("reference", json!({"project_id":"A","source":source})),
        frozen
    );
    let (ok, view) = restored.run(&["room", "view-state", "A", a, "desktop"]);
    assert!(ok, "restored view: {view}");
    assert_eq!(view["draft"], "尚未发送");
    // The original creation remains idempotent after restoring into another directory.
    assert_eq!(
        restored.accepted("create-topic", "topic-1", &topic)["room_id"],
        created["room_id"]
    );
}

//! Real pinned binaries, isolated repository, no public platform mutations.
use super::*;
use std::os::unix::fs::PermissionsExt;
use std::process::{Child, Stdio};
struct Server {
    root: PathBuf,
    process: Child,
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn find(dir: &Path, needle: &str) -> PathBuf {
    for entry in std::fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        if p.file_name().unwrap() == needle {
            return p;
        }
        if p.is_dir() {
            let p = find_optional(&p, needle);
            if let Some(p) = p {
                return p;
            }
        }
    }
    panic!("missing {needle} in {}", dir.display())
}
fn find_optional(dir: &Path, needle: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(std::result::Result::ok)
        .find_map(|e| {
            let p = e.path();
            if p.file_name()? == needle {
                Some(p)
            } else if p.is_dir() {
                find_optional(&p, needle)
            } else {
                None
            }
        })
}

#[test]
fn native_gitea_pagination_conditionals_dependencies_comments_delete_and_recovery() {
    let downloads = PathBuf::from(env!("HCTL2_TEST_SCM_DOWNLOADS"));
    let platform = if cfg!(target_os = "linux") {
        "linux-amd64"
    } else if cfg!(target_arch = "aarch64") {
        "darwin-10.12-arm64"
    } else {
        "darwin-10.12-amd64"
    };
    let tea_platform = platform.replace("-10.12", "");
    let archive = find(&downloads, &format!("gitea-1.27.3-{platform}.xz"));
    let tea = find(&downloads, &format!("tea-0.15.1-{tea_platform}"));
    let root = std::env::temp_dir().join(format!("hctl-native-task-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let gitea = root.join("gitea");
    let xz = PathBuf::from(env!("HCTL2_TEST_XZ_ROOT"));
    let out = Command::new(xz.join("bin/xz"))
        // Match packaging/common/action.sh: do not load the runner's older liblzma.
        .env("LD_LIBRARY_PATH", xz.join("lib"))
        .env_remove("XZ_DEFAULTS")
        .env_remove("XZ_OPT")
        .args(["-dc"])
        .arg(archive)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "xz: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::write(&gitea, out.stdout).unwrap();
    std::fs::set_permissions(&gitea, std::fs::Permissions::from_mode(0o700)).unwrap();
    let tea_copy = root.join("tea");
    std::fs::copy(tea, &tea_copy).unwrap();
    std::fs::set_permissions(&tea_copy, std::fs::Permissions::from_mode(0o700)).unwrap();
    let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = socket.local_addr().unwrap().port();
    drop(socket);
    let url = format!("http://127.0.0.1:{port}");
    let config = root.join("app.ini");
    std::fs::write(&config,format!("RUN_MODE = prod\n[database]\nDB_TYPE = sqlite3\nPATH = {}/data.db\n[server]\nHTTP_ADDR = 127.0.0.1\nHTTP_PORT = {port}\nROOT_URL = {url}/\nOFFLINE_MODE = true\nDISABLE_SSH = true\n[security]\nINSTALL_LOCK = true\n[service]\nDISABLE_REGISTRATION = true\n[repository]\nROOT = {}/repos\n[log]\nMODE = console\nLEVEL = Error\n",root.display(),root.display())).unwrap();
    let log = std::fs::File::create(root.join("server.log")).unwrap();
    let process = Command::new(&gitea)
        .arg("--work-path")
        .arg(&root)
        .arg("--config")
        .arg(&config)
        .arg("web")
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    let server = Server {
        root: root.clone(),
        process,
    };
    for _ in 0..200 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let admin = |args: &[&str]| {
        Command::new(&gitea)
            .arg("--work-path")
            .arg(&root)
            .arg("--config")
            .arg(&config)
            .args(["admin", "user"])
            .args(args)
            .output()
            .unwrap()
    };
    let out = admin(&[
        "create",
        "--username",
        "owner",
        "--email",
        "owner@localhost",
        "--admin",
        "--random-password",
        "--must-change-password=false",
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = admin(&[
        "generate-access-token",
        "--username",
        "owner",
        "--token-name",
        "fixture",
        "--scopes",
        "write:repository,write:issue,write:user",
        "--raw",
    ]);
    assert!(out.status.success());
    let token = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .find(|s| s.len() == 40 && s.bytes().all(|c| c.is_ascii_hexdigit()))
        .unwrap()
        .to_owned();
    let client = Client::Gitea(Hosted::fixture(
        tea_copy,
        url.clone(),
        "owner".into(),
        token,
    ));
    let repository = client
        .api(
            "POST",
            "user/repos",
            Some(json!({"name":"repo","private":true})),
        )
        .unwrap()
        .unwrap();
    let mut src = super::tests::src();
    src.platform.instance = url;
    src.platform.stable_id = id(&repository["id"]).unwrap();
    src.platform.account_id = id(&client.required("user").unwrap()["id"]).unwrap();
    src.board_scope_stable_id = src.platform.stable_id.clone();
    let binding = super::tests::effect("task.create", json!({})).binding;
    let e = super::tests::effect(
        "task.create",
        json!({"title":"task","body":"body\n<!-- native marker -->","marker":"<!-- native marker -->"}),
    );
    let first = client.effect(&src, &e, true).unwrap();
    assert_eq!(client.effect(&src, &e, false).unwrap().entity, first.entity);
    let patch = super::tests::effect(
        "task.update",
        json!({"card":first,"fields":{"title":"edited"}}),
    );
    let edited = client.effect(&src, &patch, true).unwrap();
    assert_eq!(
        edited.content_version, first.content_version,
        "title edit does not advance the body version"
    );
    let patch = super::tests::effect(
        "task.update",
        json!({"card":edited,"fields":{"body":"changed body"}}),
    );
    let edited = client.effect(&src, &patch, true).unwrap();
    assert!(edited.content_version > first.content_version);
    let stale = client.api(
        "PATCH",
        &format!("repos/owner/repo/issues/{}", first.number),
        Some(json!({"title":"overwrite","content_version":first.content_version})),
    );
    assert!(stale.is_err());
    let dep = client
        .api(
            "POST",
            "repos/owner/repo/issues",
            Some(json!({"title":"dependency"})),
        )
        .unwrap()
        .unwrap();
    client
        .api(
            "POST",
            &format!("repos/owner/repo/issues/{}/dependencies", first.number),
            Some(json!({"owner":"owner","repo":"repo","index":dep["number"]})),
        )
        .unwrap();
    let comment = super::tests::effect(
        "task.update",
        json!({"card":edited,"fields":{"comment":"hello <!-- hctl2:control:task:comment -->"}}),
    );
    client.effect(&src, &comment, true).unwrap();
    client.effect(&src, &comment, false).unwrap();
    let snap = client.snapshot(&src, binding).unwrap();
    assert_eq!(snap.cards.len(), 2);
    let card = snap
        .cards
        .iter()
        .find(|c| c.number == first.number)
        .unwrap();
    assert_eq!(card.dependencies.blocked_by.len(), 1);
    assert_eq!(card.comments.len(), 1);
    assert_eq!(
        snap.cards
            .iter()
            .find(|c| c.number != first.number)
            .unwrap()
            .dependencies
            .blocking
            .len(),
        1
    );
    let delete = super::tests::effect("task.delete", json!({"card":card}));
    assert!(client.effect(&src, &delete, true).unwrap().tombstone);
    assert!(client.effect(&src, &delete, false).unwrap().tombstone);
    drop(server);
}

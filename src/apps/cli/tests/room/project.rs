//! B1 uses the same native-package fixture, with no seeded Project or Room records.
use super::*;
use std::os::unix::fs::MetadataExt;

fn accepted(f: &Fixture, namespace: &str, kind: &str, key: &str, input: Value) -> Value {
    let (ok, result) = f.command_ns(namespace, kind, key, &input);
    assert!(ok, "{namespace}.{kind}: {result}");
    result
}
fn project_definition(name: &str) -> Value {
    json!({"name":name,"goal":"deliver B1","scope":"registered repo","roles":[],"role_members":{},"defaults":{},"settings":{"selection_policy":{},"publish_review_requires_confirmation":true}})
}

fn ready_timeline(f: &Fixture, project: &str, room: &str) -> Value {
    for _ in 0..100 {
        let (ok, result) = f.run(&["room", "timeline", project, room]);
        if ok {
            return result;
        }
        assert!(
            matches!(
                result["error"]["code"].as_str(),
                Some("CHAT_UNAVAILABLE" | "VERSION_CONFLICT")
            ),
            "{result}"
        );
        if result["error"]["code"] == "VERSION_CONFLICT" {
            assert_eq!(result["error"]["recovery_action"], "preview_again");
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("chat server did not become ready");
}

struct AgencyChild(std::process::Child);
impl Drop for AgencyChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
/// A control plane, a paired real Agency and one selected roster, all reached
/// through the CLI. `delayed_seconds` is how long the script execution body waits
/// once `delay` exists; touching that file is what keeps a dispatch observable.
struct Paired {
    agency: AgencyChild,
    repo: String,
    project: String,
    room: String,
    profession: Value,
    /// The exact `FrozenRef` file the catalog produced and `accept` consumed.
    profession_reference: PathBuf,
    profile: Value,
    delay: PathBuf,
}
fn paired(name: &str, delayed_seconds: u64) -> (Fixture, Paired) {
    paired_profile(name, delayed_seconds, "read_only")
}
fn paired_profile(name: &str, delayed_seconds: u64, mode: &str) -> (Fixture, Paired) {
    let (f, _) = Fixture::packaged(name);
    assert!(f.run(&["start", "--secret-backend", "user-file"]).0);
    let mut registration_input = json!({"name":"dispatch","origin":"local","platform":"local","platform_path":"dispatch","default_source":"gitea_issues"});
    if mode == "write" {
        let site = f.root.join("write-site");
        std::fs::create_dir(&site).unwrap();
        for args in [
            vec!["init", "--initial-branch=main"],
            vec!["config", "user.name", "Dispatch Fixture"],
            vec!["config", "user.email", "dispatch@example.test"],
        ] {
            assert!(git_output(&site, &args).status.success());
        }
        std::fs::write(site.join("source.txt"), b"initial\n").unwrap();
        assert!(git_output(&site, &["add", "source.txt"]).status.success());
        assert!(
            git_output(&site, &["commit", "-m", "fixture baseline"])
                .status
                .success()
        );
        registration_input["local"] = json!({"machine":"control","path":site});
    }
    let (mut ok, mut registered) =
        f.command_ns("repo", "register", "register-dispatch", &registration_input);
    let ready_by = std::time::Instant::now() + Duration::from_secs(60);
    while !ok {
        // Bootstrap can return a persisted pending registration before Gitea is
        // ready. Follow that command's recovery path, never start a second one.
        assert_eq!(
            registered["error"]["code"], "PLATFORM_NOT_READY",
            "{registered}"
        );
        assert_eq!(registered["error"]["recovery_action"], "retry_registration");
        assert!(std::time::Instant::now() < ready_by, "{registered}");
        let repo = registered["registration"]["repo_id"].as_str().unwrap();
        std::thread::sleep(Duration::from_millis(100));
        let resume = [
            "repo",
            "register",
            "--resume",
            repo,
            "--key",
            "resume-dispatch",
        ];
        let (preview_ok, plan) = f.run(&resume);
        assert!(preview_ok, "{plan}");
        let mut submit = resume.to_vec();
        submit.extend(["--preview-token", plan["preview_token"].as_str().unwrap()]);
        (ok, registered) = f.run(&submit);
    }
    let registration = &registered["registration"];
    let repo = registration["repo_id"].as_str().unwrap();
    let version = registration["version"].to_string();
    let args = [
        "repo",
        "register",
        "--confirm",
        repo,
        "--version",
        &version,
        "--platform-repo-id",
        registration["observed"]["stable_id"].as_str().unwrap(),
        "--key",
        "confirm-dispatch",
    ];
    let (ok, plan) = f.run(&args);
    assert!(ok, "{plan}");
    let mut confirm = args.to_vec();
    confirm.extend(["--preview-token", plan["preview_token"].as_str().unwrap()]);
    assert!(f.run(&confirm).0);
    let created = accepted(
        &f,
        "project",
        "create",
        "project-dispatch",
        json!({"repo_id":repo,"definition":project_definition("Dispatch")}),
    );
    let project = created["project_id"].as_str().unwrap().to_owned();
    let room = created["main_room_id"].as_str().unwrap().to_owned();
    let agency_root = f.root.join("independent-agency");
    let config = f.root.join("script.json");
    let delay = f.root.join("delay-script");
    let script = format!(
        "read -r initial; if test -f \"$1\"; then sleep {delayed_seconds}; fi; printf '%s\\n' '{{\"type\":\"result\",\"schema\":\"adapter.stdout.v1\",\"output\":\"DISPATCH_CHAIN_OK\"}}'"
    );
    std::fs::write(
        &config,
        json!({"program":"/bin/sh","arguments":["-c",script,"dispatch-fixture",delay]}).to_string(),
    )
    .unwrap();
    let binary = std::env::var_os("CARGO_BIN_EXE_agency").unwrap();
    let agency = AgencyChild(
        Command::new(&binary)
            .arg("--root")
            .arg(&agency_root)
            .arg("serve")
            .arg("--script-config")
            .arg(&config)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    for _ in 0..100 {
        if Command::new(&binary)
            .arg("--root")
            .arg(&agency_root)
            .arg("status")
            .output()
            .unwrap()
            .status
            .success()
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let (ok, binding) = f.run(&[
        "agency",
        "pair",
        "--binding-id",
        "local",
        "--agency-root",
        agency_root.to_str().unwrap(),
        "--key",
        "pair",
    ]);
    assert!(ok, "{binding}");
    let (ok, catalog) = f.run(&["agency", "catalog", "local"]);
    assert!(ok, "{catalog}");
    let profession = catalog["professions"][0].clone();
    let profession_reference = f.root.join("profession.json");
    std::fs::write(&profession_reference, profession["reference"].to_string()).unwrap();
    let (ok, accepted_profession) = f.run(&[
        "agency",
        "accept",
        "--binding-id",
        "local",
        "--reference",
        profession_reference.to_str().unwrap(),
        "--key",
        "accept",
    ]);
    assert!(ok, "{accepted_profession}");
    let permissions = if mode == "write" {
        json!(["context.read", "git.read", "git.write"])
    } else {
        json!(["context.read"])
    };
    let profile = accepted(
        &f,
        "profile",
        "create",
        "profile",
        json!({"id":"research","profile":{"harness":profession["harness"],"model":profession["model"],"mode":mode,"permissions":permissions,"environment":[],"required_capabilities":agency_proto::Capabilities::default(),"max_context_bytes":65536}}),
    );
    let binding_ref =
        json!({"key":binding["binding"]["key"],"version":{"state":binding["binding"]["version"]}});
    let profession_ref = json!({"key":accepted_profession["profession"]["key"],"version":{"state":accepted_profession["profession"]["version"]}});
    let selection = json!({"room_id":room,"selected_item":profession_ref,"profession":profession_ref,"profession_digest":profession["reference"]["digest"],"agency":binding_ref,"required_skills":[],"optional_skills":[],"worker_profiles":[profile["revision"]],"responsibility":"research","permission":{"allow":permissions},"budget":{"max_bytes":65536},"display_name":"Research","persona_tags":[]});
    accepted(
        &f,
        "project",
        "select",
        "select",
        json!({"project_id":project,"project_version":1,"room_id":room,"topic_command_key":null,"roster_version":null,"selections":[selection]}),
    );
    (
        f,
        Paired {
            agency,
            repo: repo.to_owned(),
            project,
            room,
            profession,
            profession_reference,
            profile,
            delay,
        },
    )
}

fn git_output(site: &Path, args: &[&str]) -> std::process::Output {
    let output = Command::new("git")
        .arg("-C")
        .arg(site)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// Configure the script fixture for the boundary just previewed by the real CLI.
/// No domain records, admitted versions or platform mappings are inserted here.
/// The confined script reconstructs the baseline with native fast-import, creates
/// its own commit, and returns a real Result frame through the Agency port.
fn configure_git_writer(
    f: &Fixture,
    setup: &mut Paired,
    pending: &Value,
    changed: bool,
    claim_no_changes: bool,
) {
    assert!(setup.agency.0.kill().is_ok());
    setup.agency.0.wait().unwrap();
    let exported = git_output(&f.root.join("write-site"), &["fast-export", "main"]);
    let output = json!({
        "change_set_id": pending["change_set_id"],
        "lease": {
            "lease_id": pending["lease"]["lease_id"],
            "generation": pending["lease"]["generation"],
        },
        "base_commit_sha": pending["baseline_commit"],
        "parent_revision_id": null,
        "location": if claim_no_changes {
            json!({"kind":"no_changes", "repo_path":"EXECUTION_PATH"})
        } else { json!({
            "kind": "commit", "repo_path": "EXECUTION_PATH", "commit_sha": "RESULT_COMMIT",
        }) },
    });
    let frame =
        json!({"type":"result","schema":"hctl2.changeset-output.v1","output":output.to_string()});
    let script = r#"set -eu
exec 2>git-error.txt
read -r initial
mkdir git-result
cd git-result
git init -q --initial-branch=main
printf '%s' "$1" | git fast-import --quiet
git reset --hard --quiet main
git config user.name 'Script Writer'
git config user.email 'writer@example.test'
if test "$5" = no_changes; then
    # Create the P1 branch without moving a ref between directories: Landlock
    # ABI V1 rejects cross-directory rename even within this private repository.
    git checkout -q -b "hctl2/changeset/$4"
    git update-ref "refs/hctl2/changesets/$4/baseline" HEAD
fi
if test "$3" = changed; then
    printf 'changed by the actual script execution\n' > source.txt
    git add source.txt
    git commit --quiet -m 'script result'
fi
commit=$(git rev-parse HEAD)
printf '%s\n' "$2" | sed -e "s|EXECUTION_PATH|$PWD|g" -e "s|RESULT_COMMIT|$commit|g" | tee ../result-frame.json
# The execution owns these unsealed Git objects until the control accepts them.
# Keep its delivery directory alive; the normal managed stop reaps the writer.
while read -r managed_input; do :; done
"#;
    let config = f.root.join("script.json");
    std::fs::write(
        &config,
        json!({"program":"/bin/sh","arguments":["-c",script,"git-writer",String::from_utf8(exported.stdout).unwrap(),frame.to_string(),if changed { "changed" } else { "unchanged" },pending["change_set_id"],if claim_no_changes { "no_changes" } else { "commit" }]}).to_string(),
    )
    .unwrap();
    let binary = std::env::var_os("CARGO_BIN_EXE_agency").unwrap();
    setup.agency = AgencyChild(
        Command::new(&binary)
            .arg("--root")
            .arg(f.root.join("independent-agency"))
            .arg("serve")
            .arg("--script-config")
            .arg(&config)
            .stdout(Stdio::null())
            .stderr(File::create(f.root.join("git-writer.log")).unwrap())
            .spawn()
            .unwrap(),
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        if Command::new(&binary)
            .arg("--root")
            .arg(f.root.join("independent-agency"))
            .arg("status")
            .output()
            .unwrap()
            .status
            .success()
        {
            break;
        }
        assert!(setup.agency.0.try_wait().unwrap().is_none());
        assert!(
            std::time::Instant::now() < deadline,
            "script Agency not ready"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Independent platform readback uses the packaged tea and the control's stored
/// credential, not a stubbed platform or a worker/test seam. Never print the token.
fn gitea_read(f: &Fixture, registration: &Value, path: &str) -> Value {
    let token = control::config::secret_store(&f.root)
        .unwrap()
        .get(registration["observed"]["credential_ref"].as_str().unwrap())
        .unwrap();
    let output = Command::new(f.payload.join("libexec/hctl2/tea"))
        .env(
            "GITEA_INSTANCE_URL",
            registration["observed"]["instance"].as_str().unwrap(),
        )
        .env("GITEA_TOKEN", String::from_utf8(token).unwrap())
        .env("NO_COLOR", "1")
        .args(["api", "-X", "GET", path])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "independent Gitea GET failed: {path}"
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn ready_gitea(f: &Fixture) {
    // Restart restores consumption asynchronously. Local records are readable
    // before the hosted process is ready to answer independent native requests.
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    loop {
        let (ok, status) = f.run(&["services", "status"]);
        assert!(ok, "{status}");
        let gitea = status["hosted"]
            .as_array()
            .unwrap()
            .iter()
            .find(|service| service["name"] == "gitea")
            .unwrap();
        assert_eq!(gitea["consumed"], true, "{status}");
        if gitea["available"] == true {
            return;
        }
        assert!(std::time::Instant::now() < deadline, "{status}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn publishing_chain(name: &str, requires_confirmation: bool) {
    let (f, mut setup) = paired_profile(name, 0, "write");
    let p = setup.project.as_str();
    if !requires_confirmation {
        let (ok, shown) = f.run(&["project", "show", p]);
        assert!(ok, "{shown}");
        let mut definition = shown["definition"].clone();
        definition["settings"]["publish_review_requires_confirmation"] = json!(false);
        accepted(
            &f,
            "project",
            "update",
            "automatic-publication",
            json!({"project_id":p,"version":shown["project"]["version"],"definition":definition}),
        );
    }
    let baseline =
        String::from_utf8(git_output(&f.root.join("write-site"), &["rev-parse", "HEAD"]).stdout)
            .unwrap()
            .trim()
            .to_owned();
    let deadline = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 120000;
    let input = f.root.join("publish-invocation.json");
    std::fs::write(&input, json!({"project_id":setup.project,"room_id":setup.room,"target":"research","profile":setup.profile["revision"],"request":"Modify source.txt within this frozen write boundary","budget":65536,"deadline_ms":deadline,"retry_of":null,"write":{"change_set_id":null,"baseline_commit":baseline,"target_branch":"main","allow_update":true}}).to_string()).unwrap();
    let args = [
        "invocation",
        "preview",
        "--input",
        input.to_str().unwrap(),
        "--key",
        "publish-chain",
    ];
    let (ok, preview) = f.run(&args);
    assert!(ok, "{preview}");
    let write = &preview["effect_summary"]["preview"]["write"];
    let pending = write["lease"]["pending"].clone();
    let set = pending["change_set_id"].as_str().unwrap().to_owned();
    configure_git_writer(&f, &mut setup, &pending, true, false);
    let p = setup.project.as_str();
    let start_args = [
        "invocation",
        "start",
        "--input",
        input.to_str().unwrap(),
        "--key",
        "publish-chain",
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ];
    let (ok, started) = f.run(&start_args);
    assert!(ok, "{started}");
    let invocation = started["invocation_id"].as_str().unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    loop {
        let (ok, shown) = f.run(&["invocation", "show", p, invocation]);
        assert!(ok, "{shown}");
        if shown["state"] == "completed" {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "script result not admitted: state={}, results={}; {}",
            shown["state"],
            shown["results"],
            script_diagnostics(&f, &shown)
        );
        assert!(
            matches!(shown["state"].as_str(), Some("pending" | "running")),
            "{shown}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    let (ok, admitted) = f.run(&["changeset", "show", &setup.repo, &set]);
    assert!(ok, "{admitted}");
    assert_eq!(admitted["revisions"].as_array().unwrap().len(), 1);
    let revision = &admitted["revisions"][0];
    let revision_id = revision["change_set_revision_id"].as_str().unwrap();
    assert_eq!(revision["producer_ref"]["invocation_id"], invocation);
    assert_eq!(revision["base_commit_sha"], baseline);
    let (ok, listed) = f.run(&["review", "list", &setup.repo]);
    assert!(ok, "{listed}");
    assert_eq!(listed["items"].as_array().unwrap().len(), 1);
    let intent = listed["items"][0]["intent_id"].as_str().unwrap();
    let (ok, registration) = f.run(&["repo", "show", &setup.repo]);
    assert!(ok, "{registration}");
    let full_name = registration["observed"]["full_name"].as_str().unwrap();
    if requires_confirmation {
        let held = f.run(&["review", "show", &setup.repo, intent]).1;
        assert_eq!(held["intent"]["state"], "pending_human");
        assert_eq!(held["intent"]["started"], false);
        assert!(held["mappings"].as_array().unwrap().is_empty());
        assert!(
            gitea_read(
                &f,
                &registration,
                &format!("repos/{full_name}/pulls?state=all")
            )
            .as_array()
            .unwrap()
            .is_empty()
        );
        // Client and daemon can disappear after admission. The same persisted
        // intent is released through the real command after control restarts.
        assert!(f.run(&["stop"]).0);
        assert!(f.run(&["start", "--secret-backend", "user-file"]).0);
        let (ok, release) = f.run(&["review", "publish", &setup.repo, intent]);
        assert!(ok, "{release}");
        let (ok, released) = f.run(&[
            "review",
            "publish",
            &setup.repo,
            intent,
            "--preview-token",
            release["preview_token"].as_str().unwrap(),
        ]);
        assert!(ok, "{released}");
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    let published = loop {
        let (ok, shown) = f.run(&["review", "show", &setup.repo, intent]);
        assert!(ok, "{shown}");
        if shown["intent"]["state"] == "published" {
            break shown;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "publication not confirmed: {shown}; {}",
            std::fs::read_to_string(f.root.join("control.log")).unwrap_or_default()
        );
        std::thread::sleep(Duration::from_millis(100));
    };
    assert_eq!(
        published["intent"]["target"]["change_set_revision_id"],
        revision_id
    );
    assert_eq!(
        published["intent"]["policy"]["policy"]["requires_human_confirmation"],
        requires_confirmation
    );
    assert_eq!(
        published["intent"]["policy"]["digest"],
        write["review_publish_policy"]["digest"]
    );
    assert_eq!(published["mappings"].as_array().unwrap().len(), 1);
    let mapping = &published["mappings"][0];
    let commit = mapping["platform_commit_sha"].as_str().unwrap();
    let index = mapping["review_request"]["index"].as_u64().unwrap();
    assert_eq!(published["stages"]["push"]["confirmed"], true);
    assert_eq!(published["stages"]["push"]["commit_sha"], commit);
    assert_eq!(published["stages"]["review_request"]["confirmed"], true);
    assert_eq!(published["stages"]["review_request"]["commit_sha"], commit);
    assert_eq!(published["stages"]["review_request"]["index"], index);
    let requests = gitea_read(
        &f,
        &registration,
        &format!("repos/{full_name}/pulls?state=all"),
    );
    assert_eq!(requests.as_array().unwrap().len(), 1);
    assert_eq!(requests[0]["number"], index);
    assert_eq!(requests[0]["head"]["sha"], commit);
    assert_eq!(requests[0]["base"]["ref"], "main");
    assert_eq!(requests[0]["state"], "open");
    assert_eq!(requests[0]["merged"], false);
    let branch = published["intent"]["branch"].as_str().unwrap();
    let remote = gitea_read(
        &f,
        &registration,
        &format!("repos/{full_name}/branches/{branch}"),
    );
    assert_eq!(remote["commit"]["id"], commit);
    let tree = String::from_utf8(
        git_output(
            &f.root.join("write-site"),
            &["rev-parse", &format!("{commit}^{{tree}}")],
        )
        .stdout,
    )
    .unwrap()
    .trim()
    .to_owned();
    assert_eq!(revision["result_tree_sha"], tree);
    assert_eq!(
        String::from_utf8(git_output(&f.root.join("write-site"), &["rev-parse", "HEAD"]).stdout)
            .unwrap()
            .trim(),
        baseline,
        "object delivery must not move the user's HEAD"
    );
    let (ok, diff) = f.run(&["changeset", "diff", &setup.repo, &set, revision_id]);
    assert!(ok, "{diff}");
    assert!(
        diff.to_string()
            .contains("changed by the actual script execution")
    );
    let replayed = replay_invocation(&f, &input, "publish-chain");
    assert_eq!(replayed["invocation_id"], invocation);
    assert!(f.run(&["stop"]).0);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let store = loop {
        match Store::open(&f.root) {
            Ok(store) => break store,
            Err(error) if error.code == "WRITER_BUSY" && std::time::Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(error) => panic!("control Store after publication: {error:?}"),
        }
    };
    assert_eq!(store.list("changeset_revision").unwrap().len(), 1);
    assert_eq!(store.list("invocation_result").unwrap().len(), 1);
    let record = store
        .get(&store::ObjectKey {
            scope: Scope::Repo(setup.repo.clone()),
            kind: "changeset_platform_binding".into(),
            id: revision_id.into(),
        })
        .unwrap()
        .unwrap();
    let RecordData::Value { value } = record.data else {
        panic!("mapping record required")
    };
    assert_eq!(value, *mapping);
    drop(store);
    assert!(f.run(&["start", "--secret-backend", "user-file"]).0);
    assert_eq!(
        f.run(&["review", "show", &setup.repo, intent]).1["mappings"],
        published["mappings"]
    );
    ready_gitea(&f);
    assert_eq!(
        gitea_read(
            &f,
            &registration,
            &format!("repos/{full_name}/pulls?state=all")
        )
        .as_array()
        .unwrap()
        .len(),
        1
    );
    println!(
        "LIVE CLI publish: {invocation} -> {revision_id} -> {branch} at {commit} -> Gitea review #{index}; confirmation={requires_confirmation}; one version, result, intent and mapping after restart"
    );
}

/// 第 9 包验收第 1 条的骨架（脚本执行体版）：本地 Gitea 这条链从真实命令行走到 Task 完成。
/// 试验仓库只在本地、缺省绑随包 Gitea；Project 与带契约的 Task；从 Room 发写入型调用（预览里
/// 有 ChangeSet、租约、冻结的发布策略）；脚本在隔离目录改代码；封存、准入、自动发成评审请求；
/// 人预览合入（只能选接受目标前移）→ Integration Receipt；人预览完成 → Task Completion Receipt。
/// 人只预览两次（合入、完成）；投影与平台事实一致。真 harness 版在 3f 合入后另加。
#[test]
fn demo3_gitea_chain_real_cli_reaches_task_completion_with_two_human_previews() {
    demo3_gitea_chain("demo3-gitea", false);
}

/// 第 9 包验收第 3 条：开了「发布评审须人显式确认」的 Project 多一次发布预览，三次预览；
/// 意图停在 `pending_human`，预览写明推到哪个分支、建到哪个目标、不是合入。
#[test]
fn demo3_gitea_chain_with_confirmation_needs_a_third_preview_for_publishing() {
    demo3_gitea_chain("demo3-gitea-confirm", true);
}

fn demo3_gitea_chain(name: &str, requires_confirmation: bool) {
    let (f, mut setup) = paired_profile(name, 0, "write");
    let p = setup.project.clone();
    let mut human_previews = Vec::new();
    // 项目定义里开关缺省是开着的；缺省路径把它关掉，发布不需要人放行。
    if !requires_confirmation {
        let (ok, shown) = f.run(&["project", "show", &p]);
        assert!(ok, "{shown}");
        let mut definition = shown["definition"].clone();
        definition["settings"]["publish_review_requires_confirmation"] = json!(false);
        accepted(
            &f,
            "project",
            "update",
            "demo3-automatic-publication",
            json!({"project_id":p,"version":shown["project"]["version"],"definition":definition}),
        );
    }
    // The update above moved the Project record; every later command names the current one.
    let (ok, current) = f.run(&["project", "show", &p]);
    assert!(ok, "{current}");
    let pv = current["project"]["version"].clone();
    let (ok, registration) = f.run(&["repo", "show", &setup.repo]);
    assert!(ok, "{registration}");
    let full_name = registration["observed"]["full_name"]
        .as_str()
        .unwrap()
        .to_owned();
    let platform_id = registration["observed"]["stable_id"]
        .as_str()
        .unwrap()
        .to_owned();
    // 带契约的 Task：机械项只认本 Task 的 Integration Receipt，另有一条人的判定。
    let source = accepted(
        &f,
        "task",
        "connect",
        "demo3-source",
        json!({"repo_id":setup.repo,"candidate_id":"gitea_issues","consent":true,"make_default":false}),
    );
    let source = source["source_id"].as_str().unwrap().to_owned();
    accepted(
        &f,
        "task",
        "attach",
        "demo3-attach",
        json!({"project_id":p,"project_version":pv,"source_id":source,"approved_scope":platform_id,"consent":true}),
    );
    let created = accepted(
        &f,
        "task",
        "create",
        "demo3-task",
        json!({"project_id":p,"project_version":pv,"source_id":source,"title":"change source.txt","body":"the script executor rewrites source.txt"}),
    );
    let task_id = created["task_id"].as_str().unwrap().to_owned();
    let (ok, before) = f.run(&["task", "show", &p, &task_id]);
    assert!(ok, "{before}");
    let contract = json!({"scope":"demo3","expected_outcome":"source.txt changed and merged","acceptance":[
        {"text":"the change is integrated","grade":"mechanical","evidence":{"accept":"integration_receipt","min_channel":"unmediated"}},
        {"text":"a human judged it done","grade":"human"}],"roles":[],"capabilities":[]});
    let digest = foundation::canonical_json_sha256(&contract).unwrap();
    let adoption = json!({"contract":contract,"origin":{"kind":"local","reference":{"key":{"scope":{"kind":"project","id":p},"kind":"project","id":p},"version":{"state":pv}},"proposal_digest":digest}});
    let adopted = accepted(
        &f,
        "task",
        "adopt",
        "demo3-adopt",
        json!({"project_id":p,"project_version":pv,"task_id":task_id,"version":before["version"],"adoption":adoption}),
    );
    let adopted_task = &adopted["task"];
    // 写入型调用，归属这张 Task。
    let baseline =
        String::from_utf8(git_output(&f.root.join("write-site"), &["rev-parse", "HEAD"]).stdout)
            .unwrap()
            .trim()
            .to_owned();
    let deadline = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 120000;
    let input = f.root.join("demo3-invocation.json");
    std::fs::write(&input, json!({"project_id":p,"room_id":setup.room,"task_id":task_id,"target":"research","profile":setup.profile["revision"],"request":"Change source.txt for the adopted Task","budget":65536,"deadline_ms":deadline,"retry_of":null,"write":{"change_set_id":null,"baseline_commit":baseline,"target_branch":"main","allow_update":true}}).to_string()).unwrap();
    let preview_args = [
        "invocation",
        "preview",
        "--input",
        input.to_str().unwrap(),
        "--key",
        "demo3-write",
    ];
    let (ok, preview) = f.run(&preview_args);
    assert!(ok, "{preview}");
    let write = &preview["effect_summary"]["preview"]["write"];
    assert_eq!(write["authorization"], "publish_for_review_not_integration");
    assert_eq!(write["publication_target"]["target_branch"], "main");
    assert_eq!(
        write["publication_target"]["requires_human_confirmation"],
        requires_confirmation
    );
    let pending = write["lease"]["pending"].clone();
    let set = pending["change_set_id"].as_str().unwrap().to_owned();
    configure_git_writer(&f, &mut setup, &pending, true, false);
    let mut start = preview_args.to_vec();
    start[1] = "start";
    start.extend([
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    let (ok, started) = f.run(&start);
    assert!(ok, "{started}");
    let invocation = started["invocation_id"].as_str().unwrap().to_owned();
    let wait = |what: &str, mut done: Box<dyn FnMut() -> Option<Value> + '_>| -> Value {
        let by = std::time::Instant::now() + Duration::from_secs(90);
        loop {
            if let Some(value) = done() {
                return value;
            }
            assert!(
                std::time::Instant::now() < by,
                "{what} did not settle; {}",
                std::fs::read_to_string(f.root.join("control.log")).unwrap_or_default()
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    };
    let finished = wait(
        "invocation",
        Box::new(|| {
            let (ok, shown) = f.run(&["invocation", "show", &p, &invocation]);
            assert!(ok, "{shown}");
            (shown["state"] == "completed").then_some(shown)
        }),
    );
    assert_eq!(finished["state"], "completed", "{finished}");
    let (ok, admitted) = f.run(&["changeset", "show", &setup.repo, &set]);
    assert!(ok, "{admitted}");
    let revision_id = admitted["revisions"][0]["change_set_revision_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let (ok, listed) = f.run(&["review", "list", &setup.repo]);
    assert!(ok, "{listed}");
    let intent = listed["items"][0]["intent_id"].as_str().unwrap().to_owned();
    // 安静：Room 里是一句给人看的话，不是机器的 ChangeSet 记录。
    // The projection is an outbox effect delivered after admission; wait for it.
    let room_bodies = || -> Vec<String> {
        ready_timeline(&f, &p, &setup.room)["events"]
            .as_array()
            .map(|events| {
                events
                    .iter()
                    .filter_map(|e| e["content"]["body"].as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    };
    let bodies = wait(
        "room projection",
        Box::new(|| {
            let bodies = room_bodies();
            bodies
                .iter()
                .any(|b| b.starts_with("Write admitted as version"))
                .then(|| json!(bodies))
        }),
    );
    let bodies: Vec<String> = serde_json::from_value(bodies).unwrap();
    let line = bodies
        .iter()
        .find(|b| b.starts_with("Write admitted as version"))
        .unwrap();
    assert!(
        line.contains(if requires_confirmation {
            "waits for a human: hctl2 review publish"
        } else {
            "Publishing for review is queued"
        }),
        "{line}"
    );
    assert!(
        !bodies.iter().any(|b| b.contains("\"change_set_id\"")),
        "raw ChangeSet output leaked into the Room: {bodies:?}"
    );
    if requires_confirmation {
        let held = f.run(&["review", "show", &setup.repo, &intent]).1;
        assert_eq!(held["intent"]["state"], "pending_human", "{held}");
        // 人的发布预览：推到哪个分支、建到哪个目标、不是合入。
        let (ok, release) = f.run(&["review", "publish", &setup.repo, &intent]);
        assert!(ok, "{release}");
        let summary = &release["effect_summary"];
        assert_eq!(summary["target_branch"], "main", "{release}");
        assert!(summary["branch"].as_str().unwrap().starts_with("hctl2/"));
        assert!(
            summary["authorizes"]
                .as_str()
                .unwrap()
                .contains("Not a merge"),
            "{release}"
        );
        let (ok, released) = f.run(&[
            "review",
            "publish",
            &setup.repo,
            &intent,
            "--preview-token",
            release["preview_token"].as_str().unwrap(),
        ]);
        assert!(ok, "{released}");
        human_previews.push("publish");
    }
    let published = wait(
        "publication",
        Box::new(|| {
            let (ok, shown) = f.run(&["review", "show", &setup.repo, &intent]);
            assert!(ok, "{shown}");
            (shown["intent"]["state"] == "published").then_some(shown)
        }),
    );
    let commit = published["mappings"][0]["platform_commit_sha"]
        .as_str()
        .unwrap()
        .to_owned();
    let index = published["mappings"][0]["review_request"]["index"]
        .as_u64()
        .unwrap();
    // 人的第 1 次预览：合入，只选接受目标前移。
    let integration = f.root.join("demo3-integration.json");
    std::fs::write(
        &integration,
        json!({"repo_id":setup.repo,"change_set_revision_id":revision_id,"target_kind":"platform","target_ref":"refs/heads/main","form":"accept_advance","strategy":"fast_forward"}).to_string(),
    )
    .unwrap();
    let integration_args = [
        "integration",
        "preview",
        "--input",
        integration.to_str().unwrap(),
        "--key",
        "demo3-merge",
    ];
    let (ok, merge_preview) = f.run(&integration_args);
    assert!(ok, "{merge_preview}");
    let mut submit = integration_args.to_vec();
    submit[1] = "submit";
    submit.extend([
        "--preview-token",
        merge_preview["preview_token"].as_str().unwrap(),
    ]);
    let (ok, submitted) = f.run(&submit);
    assert!(ok, "{submitted}");
    human_previews.push("integration");
    let merge_intent = submitted["intent_id"]
        .as_str()
        .unwrap_or_else(|| panic!("integration submit returned no intent: {submitted}"))
        .to_owned();
    let merged = wait(
        "integration",
        Box::new(|| {
            let (ok, shown) = f.run(&["integration", "show", &setup.repo, &merge_intent]);
            assert!(ok, "{shown}");
            let state = shown["intent"]["state"]
                .as_str()
                .unwrap_or_default()
                .to_owned();
            assert!(
                !matches!(state.as_str(), "failed"),
                "integration failed: {shown}"
            );
            (state == "succeeded").then_some(shown)
        }),
    );
    let receipt = merged["receipt"].clone();
    assert_eq!(
        receipt["source"]["change_set_revision_id"], revision_id,
        "{merged}"
    );
    assert_eq!(receipt["evidence_level"], "hctl2-tool", "{merged}");
    let receipt_id = receipt["receipt_id"].as_str().unwrap().to_owned();
    // 平台事实：请求已合，main 前移到发布的提交（快进）。
    ready_gitea(&f);
    let request = gitea_read(
        &f,
        &registration,
        &format!("repos/{full_name}/pulls/{index}"),
    );
    assert_eq!(request["merged"], true, "{request}");
    let main = gitea_read(
        &f,
        &registration,
        &format!("repos/{full_name}/branches/main"),
    );
    assert_eq!(main["commit"]["id"], commit, "{main}");
    assert_eq!(receipt["target_head_after"], commit, "{merged}");
    // 人的第 2 次预览：完成 Task，机械项引用这张 Receipt。
    let principal = format!("local-owner:{}", std::fs::metadata(&f.root).unwrap().uid());
    let receipt_ref = json!({"key":{"scope":{"kind":"repo","id":setup.repo},"kind":"integration_receipt","id":receipt_id},"version":{"state":1}});
    let complete = json!({
        "project_id":p,"task_id":task_id,"version":adopted_task["version"],
        "lifecycle_version":adopted_task["data"]["lifecycle_version"],
        "revision_number":adopted_task["data"]["revision"]["number"],
        "acceptance":[
            {"item":0,"judge":{"kind":"hctl2_tool"},"channel":"unmediated","references":[receipt_ref],"producer":"hctl2-tool","generation":1},
            {"item":1,"judge":{"kind":"human","actor":principal},"channel":"narrated","references":[],"producer":null,"generation":null}
        ]
    });
    let complete_input = f.root.join("demo3-complete.json");
    std::fs::write(&complete_input, complete.to_string()).unwrap();
    let complete_args = [
        "task",
        "complete",
        "--key",
        "demo3-complete",
        "--input",
        complete_input.to_str().unwrap(),
    ];
    let (ok, completion_preview) = f.run(&complete_args);
    assert!(ok, "{completion_preview}");
    let items = &completion_preview["effect_summary"]["result"]["items"];
    assert_eq!(
        items[0]["validation_level"], "unmediated",
        "{completion_preview}"
    );
    let mut confirm = complete_args.to_vec();
    confirm.extend([
        "--preview-token",
        completion_preview["preview_token"].as_str().unwrap(),
    ]);
    let (ok, completed) = f.run(&confirm);
    assert!(ok, "{completed}");
    human_previews.push("completion");
    let (ok, task_after) = f.run(&["task", "show", &p, &task_id]);
    assert!(ok, "{task_after}");
    assert_eq!(task_after["data"]["lifecycle"], "completed", "{task_after}");
    assert_eq!(
        human_previews,
        if requires_confirmation {
            vec!["publish", "integration", "completion"]
        } else {
            vec!["integration", "completion"]
        }
    );
    println!(
        "LIVE CLI demo3 gitea: task {task_id} -> invocation {invocation} -> {revision_id} -> review #{index} at {commit} -> integration {merge_intent} receipt {receipt_id} -> task completed; human previews: {human_previews:?}"
    );
}

fn script_diagnostics(f: &Fixture, shown: &Value) -> String {
    let credential_root = f.root.join("independent-agency").canonicalize().unwrap();
    let digest = agency_proto::hash(credential_root.as_os_str().as_encoded_bytes());
    let dispatch = shown["dispatch_intent"]["data"]["value"]["dispatch"]
        .as_str()
        .unwrap_or("not-mapped");
    let execution = Path::new("/tmp")
        .join(format!("hctl2-exec-{}", &digest[..20]))
        .join(dispatch);
    let trace = f.run(&[
        "terminal",
        "inspect",
        shown["invocation"]["preview"]["input"]["project_id"]
            .as_str()
            .unwrap_or("not-mapped"),
        shown["invocation"]["spec"]["document"]["owner"]["id"]
            .as_str()
            .unwrap_or("not-mapped"),
    ]);
    format!(
        "script stderr: {}; result frame: {}; Agency trace: {}",
        std::fs::read_to_string(execution.join("git-error.txt")).unwrap_or_default(),
        std::fs::read_to_string(execution.join("result-frame.json")).unwrap_or_default(),
        trace.1,
    )
}

fn replay_invocation(f: &Fixture, input: &Path, key: &str) -> Value {
    // Preview tokens belong to the daemon, not the persistent command receipt.
    // Fetch a new token after restart while replaying the same frozen input.
    let args = [
        "invocation",
        "preview",
        "--input",
        input.to_str().unwrap(),
        "--key",
        key,
    ];
    let (ok, plan) = f.run(&args);
    assert!(ok, "{plan}");
    let (ok, replay) = f.run(&[
        "invocation",
        "start",
        "--input",
        input.to_str().unwrap(),
        "--key",
        key,
        "--preview-token",
        plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{replay}");
    replay
}

#[test]
fn write_dispatch_real_cli_admits_and_publishes_to_packaged_gitea() {
    publishing_chain("publish-automatic", false);
}

#[test]
fn write_dispatch_real_cli_persists_human_gate_and_publishes_after_restart() {
    publishing_chain("publish-held", true);
}

#[test]
fn write_dispatch_real_cli_accepts_verified_no_changes_without_publishing() {
    let (f, mut setup) = paired_profile("write-unchanged", 0, "write");
    let site = f.root.join("write-site");
    let baseline = String::from_utf8(git_output(&site, &["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_owned();
    let deadline = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 120000;
    let input = f.root.join("unchanged-invocation.json");
    let objective = "Inspect source.txt; keep the tree unchanged if no correction is needed";
    std::fs::write(&input, json!({"project_id":setup.project,"room_id":setup.room,"target":"research","profile":setup.profile["revision"],"request":objective,"budget":65536,"deadline_ms":deadline,"retry_of":null,"write":{"change_set_id":null,"baseline_commit":baseline,"target_branch":"main","allow_update":true}}).to_string()).unwrap();
    let (ok, preview) = f.run(&[
        "invocation",
        "preview",
        "--input",
        input.to_str().unwrap(),
        "--key",
        "unchanged",
    ]);
    assert!(ok, "{preview}");
    let write = &preview["effect_summary"]["preview"]["write"];
    assert_eq!(write["objective"], objective);
    assert_eq!(
        write["repo_local_path"],
        site.canonicalize().unwrap().to_str().unwrap()
    );
    assert_eq!(write["repo_local_machine"], "control");
    assert_eq!(write["publication_target"]["target_branch"], "main");
    let pending = write["lease"]["pending"].clone();
    let set = pending["change_set_id"].as_str().unwrap();
    configure_git_writer(&f, &mut setup, &pending, false, true);
    let start = [
        "invocation",
        "start",
        "--input",
        input.to_str().unwrap(),
        "--key",
        "unchanged",
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ];
    let (ok, started) = f.run(&start);
    assert!(ok, "{started}");
    let invocation = started["invocation_id"].as_str().unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    let shown = loop {
        let (ok, shown) = f.run(&["invocation", "show", &setup.project, invocation]);
        assert!(ok, "{shown}");
        if shown["state"] == "completed" {
            break shown;
        }
        assert!(
            matches!(shown["state"].as_str(), Some("pending" | "running"))
                && std::time::Instant::now() < deadline,
            "unchanged result not admitted: {shown}; {}",
            script_diagnostics(&f, &shown)
        );
        std::thread::sleep(Duration::from_millis(100));
    };
    assert_eq!(shown["results"].as_array().unwrap().len(), 1);
    assert_eq!(
        shown["results"][0]["record"]["data"]["value"]["no_changes"],
        true
    );
    assert!(shown["results"][0]["record"]["data"]["value"]["change_set_revision"].is_null());
    let (ok, sealed) = f.run(&["changeset", "show", &setup.repo, set]);
    assert!(ok, "{sealed}");
    assert!(sealed["revisions"].as_array().unwrap().is_empty());
    let observation = &sealed["saved_git_observations"][0]["observation"];
    assert_eq!(observation["base_tree_sha"], observation["result_tree_sha"]);
    assert!(
        f.run(&["review", "list", &setup.repo]).1["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let registration = f.run(&["repo", "show", &setup.repo]).1;
    let full_name = registration["observed"]["full_name"].as_str().unwrap();
    assert!(
        gitea_read(
            &f,
            &registration,
            &format!("repos/{full_name}/pulls?state=all")
        )
        .as_array()
        .unwrap()
        .is_empty()
    );
    assert_eq!(
        replay_invocation(&f, &input, "unchanged")["invocation_id"],
        invocation
    );
    assert!(f.run(&["stop"]).0);
    assert!(f.run(&["start", "--secret-backend", "user-file"]).0);
    assert_eq!(
        f.run(&["invocation", "show", &setup.project, invocation]).1["results"],
        shown["results"]
    );
    assert_eq!(
        String::from_utf8(git_output(&site, &["rev-parse", "HEAD"]).stdout)
            .unwrap()
            .trim(),
        baseline
    );
    println!(
        "LIVE CLI unchanged: {invocation}; native trees equal, one accepted result, no Revision, publication intent or Gitea review after restart"
    );
}

#[test]
fn write_dispatch_real_cli_freezes_policy_grants_once_and_requires_exit_before_regrant() {
    let (f, setup) = paired_profile("write-dispatch", 120, "write");
    std::fs::write(&setup.delay, b"keep the writer observable").unwrap();
    let p = setup.project.as_str();
    let baseline =
        String::from_utf8(git_output(&f.root.join("write-site"), &["rev-parse", "HEAD"]).stdout)
            .unwrap()
            .trim()
            .to_owned();
    let deadline = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 120000;
    let path = f.root.join("write-invocation.json");
    let input = path.to_str().unwrap();
    let mut request = json!({"project_id":p,"room_id":setup.room,"target":"research","profile":setup.profile["revision"],"request":"Use only the frozen ChangeSet boundary","budget":65536,"deadline_ms":deadline,"retry_of":null,"write":{"change_set_id":null,"baseline_commit":baseline,"target_branch":"main","allow_update":true}});
    std::fs::write(&path, request.to_string()).unwrap();
    let (ok, plan) = f.run(&[
        "invocation",
        "preview",
        "--input",
        input,
        "--key",
        "write-1",
    ]);
    assert!(ok, "{plan}");
    let write = plan["effect_summary"]["preview"]["write"].clone();
    assert_eq!(write["lease"]["pending"]["lease"]["state"], "pending");
    assert_eq!(write["lease"]["pending"]["baseline_commit"], baseline);
    assert_eq!(write["authorization"], "publish_for_review_not_integration");
    let set = write["lease"]["pending"]["change_set_id"].as_str().unwrap();
    let (ok, listed) = f.run(&["invocation", "list", p]);
    assert!(ok, "{listed}");
    assert!(listed["invocations"].as_array().unwrap().is_empty());
    let (ok, started) = f.run(&[
        "invocation",
        "start",
        "--input",
        input,
        "--key",
        "write-1",
        "--preview-token",
        plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{started}");
    let id = started["invocation_id"].as_str().unwrap();
    let mut running = Value::Null;
    for _ in 0..150 {
        let (ok, value) = f.run(&["invocation", "show", p, id]);
        assert!(ok, "{value}");
        running = value;
        if running["state"] == "running" {
            break;
        }
        assert_eq!(running["state"], "pending", "{running}");
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(running["state"], "running", "{running}");
    let spec = running["invocation"]["spec"]["document"].clone();
    assert_eq!(spec["base"], baseline);
    assert_eq!(spec["write_lease"]["revision"], "1");
    assert_eq!(
        spec["review_publish_policy"],
        write["review_publish_policy"]
    );
    assert_eq!(running["invocation"]["authorization"]["write"], true);
    assert!(
        running["invocation"]["authorization"]["authorizing_actor"]["principal"]
            .as_str()
            .is_some()
    );

    request["write"]["change_set_id"] = json!(set);
    std::fs::write(&path, request.to_string()).unwrap();
    let (ok, busy) = f.run(&[
        "invocation",
        "preview",
        "--input",
        input,
        "--key",
        "write-2",
    ]);
    assert!(!ok, "{busy}");
    assert_eq!(busy["error"]["code"], "WRITE_LEASE_BUSY");

    let (ok, project) = f.run(&["project", "show", p]);
    assert!(ok, "{project}");
    let mut definition = project["definition"].clone();
    definition["settings"]["publish_review_requires_confirmation"] = json!(false);
    accepted(
        &f,
        "project",
        "update",
        "confirmation-off",
        json!({"project_id":p,"version":project["project"]["version"],"definition":definition}),
    );
    let (ok, unchanged) = f.run(&["invocation", "show", p, id]);
    assert!(ok, "{unchanged}");
    assert_eq!(unchanged["invocation"]["spec"]["document"], spec);

    let cancelled = accepted(
        &f,
        "invocation",
        "cancel",
        "cancel-writer",
        json!({"project_id":p,"invocation_id":id,"state_version":running["state_version"],"reason":"cancel this writer before assigning another"}),
    );
    assert_eq!(cancelled["state"], "cancelled");
    assert_eq!(cancelled["cleanup_pending"], true);
    // A lease can be regranted only after the real Agency exit report is stored.
    // Poll the human preview: no fixture inserts a receipt or grants a lease.
    let mut next = Value::Null;
    for _ in 0..150 {
        let (ok, value) = f.run(&[
            "invocation",
            "preview",
            "--input",
            input,
            "--key",
            "write-2",
        ]);
        next = value;
        if ok {
            break;
        }
        assert_eq!(next["error"]["code"], "WRITE_LEASE_BUSY", "{next}");
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(next["preview_token"].is_string(), "{next}");
    let next_write = &next["effect_summary"]["preview"]["write"];
    assert_eq!(next_write["lease"]["pending"]["lease"]["generation"], 2);
    assert_ne!(
        next_write["review_publish_policy"],
        write["review_publish_policy"]
    );
    assert!(f.run(&["stop"]).0);
    // stop acknowledges shutdown; in-flight blocking workers can still own
    // the Store. Wait for its native writer lock to be released, not a delay.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let store = loop {
        match Store::open(&f.root) {
            Ok(store) => break store,
            Err(error) if error.code == "WRITER_BUSY" && std::time::Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("control did not release its Store after stop: {error:?}"),
        }
    };
    let saved = store
        .get(&store::ObjectKey {
            scope: Scope::Repo(setup.repo.clone()),
            kind: "changeset".into(),
            id: set.into(),
        })
        .unwrap()
        .unwrap();
    let RecordData::Value { value: saved } = saved.data else {
        panic!("ChangeSet record required")
    };
    assert_eq!(saved["lease"]["state"], "revoked");
    assert_eq!(saved["lease"]["generation"], 1);
    assert!(
        store.list("review_publish_intent").unwrap().is_empty(),
        "a preview/start is not a Revision admission"
    );
    let policy_id = write["review_publish_policy"]["id"].as_str().unwrap();
    let record = store
        .get(&store::ObjectKey {
            scope: Scope::Repo(setup.repo.clone()),
            kind: "review_publish_policy".into(),
            id: policy_id.into(),
        })
        .unwrap()
        .unwrap();
    let RecordData::Value { value } = record.data else {
        panic!("policy record required")
    };
    assert_eq!(value["policy"]["requires_human_confirmation"], true);
    assert_eq!(value["digest"], write["review_publish_policy"]["digest"]);
    println!(
        "CLI writer: {id}; frozen policy: {policy_id}; lease 1 stopped, next preview generation 2; no publish intent"
    );
}

#[test]
fn dispatch_from_real_cli_pairing_to_room_answer_and_restart_keeps_one_invocation() {
    let (f, mut setup) = paired("dispatch-chain", 2);
    let p = setup.project.as_str();
    let room = setup.room.as_str();
    let delay = &setup.delay;
    let agency = &mut setup.agency;
    let profile = &setup.profile;
    let show = f.show(p, room);
    accepted(
        &f,
        "room",
        "send",
        "context",
        json!({"project_id":p,"room_id":room,"version":show["binding"]["version"],"body":"READ_ONLY_CONTEXT_MARKER"}),
    );
    let path = f.root.join("invocation.json");
    let deadline = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 60000;
    std::fs::write(&path, json!({"project_id":p,"room_id":room,"target":"research","profile":profile["revision"],"request":"Return the exact answer without modifying files","budget":65536,"deadline_ms":deadline,"retry_of":null}).to_string()).unwrap();
    let (ok, plan) = f.run(&[
        "invocation",
        "preview",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "invoke",
    ]);
    assert!(ok, "{plan}");
    let frozen = &plan["effect_summary"]["assembly"]["bundle"]["document"];
    assert_eq!(frozen["consumer"]["project"], p);
    assert_eq!(
        frozen["entries"].as_array().unwrap().len(),
        2,
        "request and this Room's exact online window"
    );
    // Starting without a real preview must not create an invocation.
    let (ok, refusal) = f.run(&[
        "invocation",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "invoke",
        "--preview-token",
        "invented",
    ]);
    assert!(!ok);
    assert_eq!(refusal["error"]["code"], "PREVIEW_REQUIRED");
    let (ok, started) = f.run(&[
        "invocation",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "invoke",
        "--preview-token",
        plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{started}");
    let id = started["invocation_id"].as_str().unwrap();
    let mut completed = Value::Null;
    for _ in 0..150 {
        let (ok, value) = f.run(&["invocation", "show", p, id]);
        assert!(ok, "{value}");
        if value["state"] == "completed" {
            completed = value;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(
        completed["state"], "completed",
        "answer admission: {completed}"
    );
    assert_eq!(completed["results"][0]["output"], "DISPATCH_CHAIN_OK");
    for _ in 0..100 {
        let (ok, timeline) = f.run(&["room", "timeline", p, room]);
        assert!(ok, "{timeline}");
        let messages = timeline["events"].as_array().unwrap();
        let count = messages
            .iter()
            .filter(|e| e["content"]["body"] == "DISPATCH_CHAIN_OK")
            .count();
        if count > 0 {
            assert_eq!(count, 1);
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(f.run(&["stop"]).0);
    std::thread::sleep(Duration::from_millis(100));
    assert!(f.run(&["start"]).0);
    let (ok, again) = f.run(&["invocation", "show", p, id]);
    assert!(ok, "{again}");
    assert_eq!(again["results"], completed["results"]);
    let (ok, replay_plan) = f.run(&[
        "invocation",
        "preview",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "invoke",
    ]);
    assert!(ok, "{replay_plan}");
    assert_eq!(
        replay_plan["effect_summary"], plan["effect_summary"],
        "replay uses the frozen Context, including the original window"
    );
    let (ok, replay) = f.run(&[
        "invocation",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "invoke",
        "--preview-token",
        replay_plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{replay}");
    assert_eq!(replay, started);
    let timeline = ready_timeline(&f, p, room);
    assert_eq!(
        timeline["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["content"]["body"] == "DISPATCH_CHAIN_OK")
            .count(),
        1
    );

    // The answer can be admitted while Matrix is unavailable. Recovery of
    // that outbox must not require the Agency that already returned it.
    std::fs::write(delay, b"delay").unwrap();
    let mut next_input: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    next_input["deadline_ms"] = json!(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            + 60_000
    );
    std::fs::write(&path, next_input.to_string()).unwrap();
    let (ok, pending_plan) = f.run(&[
        "invocation",
        "preview",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "pending-projection",
    ]);
    assert!(ok, "{pending_plan}");
    let stop_chat = || {
        assert!(
            Command::new(f.payload.join("bin/hctl2-services"))
                .env("HCTL2_STATE_ROOT", f.root.join("services"))
                .args(["stop", "tuwunel"])
                .output()
                .unwrap()
                .status
                .success()
        );
    };
    stop_chat();
    let (ok, unavailable) = f.run(&[
        "invocation",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "pending-projection",
        "--preview-token",
        pending_plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(!ok, "a live Room is required for a new authorization");
    assert_eq!(unavailable["error"]["code"], "CHAT_UNAVAILABLE");
    assert!(
        Command::new(f.payload.join("bin/hctl2-services"))
            .env("HCTL2_STATE_ROOT", f.root.join("services"))
            .args(["start", "tuwunel"])
            .output()
            .unwrap()
            .status
            .success()
    );
    ready_timeline(&f, p, room);
    let (ok, pending_call) = f.run(&[
        "invocation",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "pending-projection",
        "--preview-token",
        pending_plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{pending_call}");
    stop_chat();
    let pending_id = pending_call["invocation_id"].as_str().unwrap();
    let mut saved = Value::Null;
    for _ in 0..150 {
        let (ok, value) = f.run(&["invocation", "show", p, pending_id]);
        assert!(ok, "{value}");
        if value["state"] == "completed" {
            saved = value;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(saved["state"], "completed", "{saved}");
    assert!(
        saved["pending_effects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["operation"] == "invocation.project")
    );
    assert!(f.run(&["stop"]).0);
    agency.0.kill().unwrap();
    agency.0.wait().unwrap();
    assert!(f.run(&["start"]).0);
    let mut delivered = false;
    for _ in 0..100 {
        let timeline = ready_timeline(&f, p, room);
        let count = timeline["events"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["content"]["body"] == "DISPATCH_CHAIN_OK")
            .count();
        if count == 2 {
            delivered = true;
            break;
        }
        assert!(count < 2);
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(delivered, "pending projection recovered without the Agency");
}

/// The second-half surface walks the real CLI to its handler. Each refusal below
/// is one this surface adds, so removing the check turns the case red.
#[test]
fn dispatch_rest_surface_lists_cancels_retries_and_reads_terminal_from_real_cli() {
    let (f, mut setup) = paired("dispatch-rest", 120);
    let p = setup.project.as_str();
    let room = setup.room.as_str();
    let state_version = |value: &Value| value["state_version"].as_i64().unwrap();
    let deadline = || {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
            + 60_000
    };
    // The script body only waits once this file exists, so the dispatch stays
    // observable instead of returning its answer at once.
    std::fs::write(&setup.delay, b"delay").unwrap();

    // `profession accept` is the same command as `agency accept`, so the pairing's
    // key replays the record instead of accepting a second time.
    let (ok, listed) = f.run(&["profession", "list"]);
    assert!(ok, "{listed}");
    let professions = listed["records"].as_array().unwrap();
    assert_eq!(professions.len(), 1, "{listed}");
    let (ok, alias) = f.run(&[
        "profession",
        "accept",
        "--binding-id",
        "local",
        "--reference",
        setup.profession_reference.to_str().unwrap(),
        "--key",
        "accept",
    ]);
    assert!(ok, "{alias}");
    assert_eq!(alias["profession"]["key"], professions[0]["key"]);

    // Keeping a candidate means resending the whole roster at its exact version.
    let (ok, roster) = f.run(&["room", "roster", "show", p, room]);
    assert!(ok, "{roster}");
    assert_eq!(
        roster["selections"].as_array().unwrap().len(),
        1,
        "{roster}"
    );
    // A roster read returns the stored records; a select resends the selections.
    let selections: Value = json!(
        roster["selections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|record| record["data"]["value"].clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(selections[0]["responsibility"], "research", "{selections}");
    let select = |roster_version: Value, key: &str| {
        let path = f.root.join("roster-select.json");
        std::fs::write(
            &path,
            json!({"project_id":p,"project_version":1,"room_id":room,"topic_command_key":null,"roster_version":roster_version,"selections":selections})
                .to_string(),
        )
        .unwrap();
        let input = path.to_str().unwrap();
        let (ok, preview) = f.run(&["room", "roster", "select", "--key", key, "--input", input]);
        if !ok {
            return (ok, preview);
        }
        f.run(&[
            "room",
            "roster",
            "select",
            "--key",
            key,
            "--input",
            input,
            "--preview-token",
            preview["preview_token"].as_str().unwrap(),
        ])
    };
    let (ok, absent) = select(json!(null), "select-absent");
    assert!(!ok, "an existing roster is not an absent one");
    assert_eq!(absent["error"]["code"], "VERSION_CONFLICT");
    let (ok, reselected) = select(json!(1), "select-again");
    assert!(ok, "{reselected}");
    assert_eq!(reselected["roster_version"], 2);

    // Reading a Profile shows the exact revision its pointer names, and grants nothing.
    let (ok, shown) = f.run(&["profile", "show", "research"]);
    assert!(ok, "{shown}");
    assert_eq!(shown["pointer"]["version"], json!({"state":1}));
    assert_eq!(shown["profile"]["model"], setup.profession["model"]);
    let (ok, missing) = f.run(&["profile", "show", "no-such-profile"]);
    assert!(!ok, "{missing}");
    assert_eq!(missing["error"]["code"], "PROFILE_NOT_FOUND");

    // One Invocation that stays observable, so cancellation has something to revoke.
    let path = f.root.join("invocation.json");
    std::fs::write(
        &path,
        json!({"project_id":p,"room_id":room,"target":"research","profile":setup.profile["revision"],"request":"Return the exact answer without modifying files","budget":65536,"deadline_ms":deadline(),"retry_of":null})
            .to_string(),
    )
    .unwrap();
    let input = path.to_str().unwrap();
    let (ok, plan) = f.run(&["invocation", "preview", "--input", input, "--key", "rest"]);
    assert!(ok, "{plan}");
    let bundle = plan["effect_summary"]["assembly"]["bundle"]["document"]["id"].clone();
    let (ok, started) = f.run(&[
        "invocation",
        "start",
        "--input",
        input,
        "--key",
        "rest",
        "--preview-token",
        plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{started}");
    let id = started["invocation_id"].as_str().unwrap().to_owned();
    let mut running = Value::Null;
    for _ in 0..150 {
        let (ok, value) = f.run(&["invocation", "show", p, &id]);
        assert!(ok, "{value}");
        if value["state"] == "running" {
            running = value;
            break;
        }
        assert_ne!(
            value["state"], "completed",
            "the delayed script answered before it could be observed: {value}"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(running["state"], "running", "{running}");

    // Listing projects this Project's own records; a second Project in the same
    // control plane proves it does not reach another one's.
    let (ok, list) = f.run(&["invocation", "list", p]);
    assert!(ok, "{list}");
    assert_eq!(list["project_id"], p);
    let invocations = list["invocations"].as_array().unwrap();
    assert_eq!(invocations.len(), 1, "{list}");
    assert_eq!(invocations[0]["invocation_id"], id);
    assert_eq!(invocations[0]["owner"], running["owner"]);
    assert_eq!(invocations[0]["state_version"], running["state_version"]);
    let other = accepted(
        &f,
        "project",
        "create",
        "project-other",
        json!({"repo_id":setup.repo,"definition":project_definition("Other")}),
    );
    let elsewhere = other["project_id"].as_str().unwrap();
    let (ok, other_list) = f.run(&["invocation", "list", elsewhere]);
    assert!(ok, "{other_list}");
    assert!(
        other_list["invocations"].as_array().unwrap().is_empty(),
        "{other_list}"
    );
    for args in [
        vec!["invocation", "show", elsewhere, &id],
        vec!["terminal", "inspect", elsewhere, &id],
        vec!["terminal", "replay", elsewhere, &id],
    ] {
        let (ok, crossed) = f.run(&args);
        assert!(!ok, "{args:?}: {crossed}");
        assert_eq!(crossed["error"]["code"], "NOT_FOUND", "{args:?}");
    }

    // Inspection goes through the Agency port on an observe-only ticket.
    for after in ["0", "999999"] {
        let (ok, inspected) = f.run(&["terminal", "inspect", p, &id, "--after", after]);
        assert!(ok, "{inspected}");
        assert_eq!(inspected["trace"]["dispatch"]["owner"]["id"], id);
    }
    // A cursor past the last recorded event is a gap, not an empty completion.
    let (ok, ahead) = f.run(&["terminal", "inspect", p, &id, "--after", "999999"]);
    assert!(ok, "{ahead}");
    assert!(ahead["trace"]["gap"].as_bool().unwrap(), "{ahead}");
    assert!(!ahead["trace"]["complete"].as_bool().unwrap(), "{ahead}");

    // What the Agency reported reaches the governance records through the reconcile
    // loop alone, so wait for its own observation before asking what a human look
    // adds to it.
    let stored = || -> Vec<Value> {
        let (ok, replayed) = f.run(&["terminal", "replay", p, &id]);
        assert!(ok, "{replayed}");
        replayed["observations"].as_array().unwrap().clone()
    };
    let mut observed = Vec::new();
    for _ in 0..150 {
        observed = stored();
        if !observed.is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(!observed.is_empty(), "the reconcile loop observed nothing");
    for _ in 0..3 {
        let (ok, inspected) = f.run(&["terminal", "inspect", p, &id]);
        assert!(ok, "{inspected}");
    }
    assert_eq!(
        stored(),
        observed,
        "a human inspect is a read: it records no observation"
    );

    // Stored replay reads the control plane only: the Agency is already gone.
    setup.agency.0.kill().unwrap();
    setup.agency.0.wait().unwrap();
    let observations = stored();
    assert_eq!(observations, observed, "replay needs no Agency");
    assert_eq!(observations[0]["dispatch"]["owner"]["id"], id);
    // Record keys order by digest, so replay restores the Agency's cursor order.
    let cursors: Vec<u64> = observations
        .iter()
        .map(|o| o["cursor"].as_u64().unwrap())
        .collect();
    let mut ordered = cursors.clone();
    ordered.sort_unstable();
    assert_eq!(cursors, ordered, "replay is in observation order");
    // An unreachable Agency stays the answer, however often a human asks: the read
    // writes no contact record, so there is no command identity to collide with.
    for _ in 0..5 {
        let (ok, unreachable) = f.run(&["terminal", "inspect", p, &id]);
        assert!(!ok, "{unreachable}");
        assert_eq!(unreachable["error"]["code"], "AGENCY_UNREACHABLE");
    }
    assert_eq!(stored(), observed, "a human inspect records nothing");
    // Attach has no public entry: the internal signer never grants managed input.
    let (ok, attach) = f.run(&["terminal", "attach", p, &id]);
    assert!(!ok, "{attach}");
    assert_eq!(attach["error"]["code"], "INPUT_NOT_IMPLEMENTED");

    // Cancellation is the domain's `End`. Its preview reports the real consequence
    // and never a confirmed isolation.
    let cancel_path = f.root.join("cancel.json");
    let cancel_input = cancel_path.to_str().unwrap();
    let write_cancel = |version: i64| {
        std::fs::write(
            &cancel_path,
            json!({"project_id":p,"invocation_id":id,"state_version":version,"reason":"operator ended the demonstration"})
                .to_string(),
        )
        .unwrap();
    };
    write_cancel(state_version(&running));
    let (ok, invented) = f.run(&[
        "invocation",
        "cancel",
        "--key",
        "cancel-rest",
        "--input",
        cancel_input,
        "--preview-token",
        "invented",
    ]);
    assert!(!ok, "{invented}");
    assert_eq!(invented["error"]["code"], "PREVIEW_REQUIRED");
    write_cancel(state_version(&running) + 100);
    let (ok, stale) = f.run(&[
        "invocation",
        "cancel",
        "--key",
        "cancel-stale",
        "--input",
        cancel_input,
    ]);
    assert!(!ok, "{stale}");
    assert_eq!(stale["error"]["code"], "VERSION_CONFLICT");
    assert_eq!(
        stale["error"]["message"],
        "cancellation names a stale state version"
    );
    let mut failed: Value = serde_json::from_slice(&std::fs::read(&cancel_path).unwrap()).unwrap();
    failed["outcome"] = json!("failed");
    let failed_path = f.root.join("cancel-failed.json");
    std::fs::write(&failed_path, failed.to_string()).unwrap();
    let (ok, refused) = f.run(&[
        "invocation",
        "cancel",
        "--key",
        "cancel-failed",
        "--input",
        failed_path.to_str().unwrap(),
    ]);
    assert!(!ok, "{refused}");
    assert_eq!(refused["error"]["code"], "INVOCATION_COMMAND_FAILED");
    assert_eq!(
        refused["error"]["message"],
        "only a cancelled outcome is a client command"
    );
    write_cancel(state_version(&running));
    let (ok, cancel_plan) = f.run(&[
        "invocation",
        "cancel",
        "--key",
        "cancel-rest",
        "--input",
        cancel_input,
    ]);
    assert!(ok, "{cancel_plan}");
    let effect = &cancel_plan["effect_summary"];
    assert_eq!(effect["state"], "running");
    assert_eq!(effect["outcome"], "cancelled");
    assert_eq!(effect["state_version"], running["state_version"]);
    assert!(
        effect["cleanup_pending"].as_bool().unwrap(),
        "{cancel_plan}"
    );
    assert_eq!(
        effect["isolation_confirmed"],
        json!(false),
        "a queued stop is not a confirmed isolation"
    );
    let (ok, cancelled) = f.run(&[
        "invocation",
        "cancel",
        "--key",
        "cancel-rest",
        "--input",
        cancel_input,
        "--preview-token",
        cancel_plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{cancelled}");
    assert_eq!(cancelled["state"], "cancelled");
    assert_eq!(cancelled["authorization_revoked"], json!(true));
    assert_eq!(
        cancelled["state_version"],
        json!(state_version(&running) + 1)
    );
    assert!(
        cancelled["cleanup_pending"].as_bool().unwrap(),
        "{cancelled}"
    );
    // A second cancellation under a new key finds a terminal Invocation and writes
    // nothing, so it cannot enqueue a second stop for the same dispatch.
    write_cancel(state_version(&running) + 1);
    let (ok, twice) = f.run(&[
        "invocation",
        "cancel",
        "--key",
        "cancel-twice",
        "--input",
        cancel_input,
    ]);
    assert!(!ok, "{twice}");
    assert_eq!(twice["error"]["code"], "INVALID_TRANSITION");

    let (ok, after) = f.run(&["invocation", "list", p]);
    assert!(ok, "{after}");
    let revoked = &after["invocations"][0];
    assert_eq!(revoked["state"], "cancelled");
    assert_eq!(revoked["state_version"], json!(state_version(&running) + 1));
    assert_ne!(
        revoked["owner"], running["owner"],
        "revocation advances the authorization root"
    );

    // Retry names the exact revoked authorization and freezes its own Bundle.
    let retry_of = f.root.join("retry-of.json");
    std::fs::write(&retry_of, revoked["owner"].to_string()).unwrap();
    let stale_owner = f.root.join("stale-owner.json");
    std::fs::write(&stale_owner, running["owner"].to_string()).unwrap();
    let retry_path = f.root.join("retry.json");
    std::fs::write(
        &retry_path,
        json!({"project_id":p,"room_id":room,"target":"research","profile":setup.profile["revision"],"request":"Return the exact answer without modifying files","budget":65536,"deadline_ms":deadline(),"retry_of":null})
            .to_string(),
    )
    .unwrap();
    let retry_input = retry_path.to_str().unwrap();
    let retry_of_input = retry_of.to_str().unwrap();
    let (ok, stale_retry) = f.run(&[
        "invocation",
        "retry",
        "--key",
        "retry-stale",
        "--input",
        retry_input,
        "--retry-of",
        stale_owner.to_str().unwrap(),
    ]);
    assert!(!ok, "{stale_retry}");
    assert_eq!(stale_retry["error"]["code"], "RETRY_NOT_ALLOWED");
    // An input file that names a different original is refused before any request.
    let mut claimed: Value = serde_json::from_slice(&std::fs::read(&retry_path).unwrap()).unwrap();
    claimed["retry_of"] = running["owner"].clone();
    let claimed_path = f.root.join("claimed-retry.json");
    std::fs::write(&claimed_path, claimed.to_string()).unwrap();
    let (ok, differs) = f.run(&[
        "invocation",
        "retry",
        "--key",
        "retry-claimed",
        "--input",
        claimed_path.to_str().unwrap(),
        "--retry-of",
        retry_of_input,
    ]);
    assert!(!ok, "{differs}");
    assert_eq!(differs["error"]["code"], "INVOCATION_COMMAND_FAILED");
    assert_eq!(
        differs["error"]["message"],
        "input retry_of differs from --retry-of"
    );
    let (ok, retry_plan) = f.run(&[
        "invocation",
        "retry",
        "--key",
        "retry-1",
        "--input",
        retry_input,
        "--retry-of",
        retry_of_input,
    ]);
    assert!(ok, "{retry_plan}");
    assert_ne!(
        retry_plan["effect_summary"]["assembly"]["bundle"]["document"]["id"], bundle,
        "a retry freezes its own Bundle"
    );
    let (ok, retried) = f.run(&[
        "invocation",
        "retry",
        "--key",
        "retry-1",
        "--input",
        retry_input,
        "--retry-of",
        retry_of_input,
        "--preview-token",
        retry_plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{retried}");
    let retry_id = retried["invocation_id"].as_str().unwrap().to_owned();
    assert_ne!(retry_id, id);
    assert_eq!(retried["state"], "pending");
    assert_eq!(retried["state_version"], json!(1));
    // The Agency is gone, so the retry can never commit a dispatch to observe.
    let (ok, unmapped) = f.run(&["terminal", "inspect", p, &retry_id]);
    assert!(!ok, "{unmapped}");
    assert_eq!(unmapped["error"]["code"], "INVALID_INPUT");
    assert_eq!(unmapped["error"]["message"], "dispatch mapping required");
    let (ok, both) = f.run(&["invocation", "list", p]);
    assert!(ok, "{both}");
    assert_eq!(both["invocations"].as_array().unwrap().len(), 2, "{both}");

    // Updating the pointer freezes a new immutable revision; reading shows exactly it.
    let update_path = f.root.join("profile-update.json");
    let mut definition = shown["profile"].clone();
    definition["max_context_bytes"] = json!(32768);
    std::fs::write(
        &update_path,
        json!({"id":"research","version":1,"profile":definition}).to_string(),
    )
    .unwrap();
    let update_input = update_path.to_str().unwrap();
    let (ok, update_plan) = f.run(&[
        "profile",
        "update",
        "--key",
        "profile-2",
        "--input",
        update_input,
    ]);
    assert!(ok, "{update_plan}");
    let (ok, updated) = f.run(&[
        "profile",
        "update",
        "--key",
        "profile-2",
        "--input",
        update_input,
        "--preview-token",
        update_plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{updated}");
    assert_eq!(updated["version"], json!(2));
    let (ok, current) = f.run(&["profile", "show", "research"]);
    assert!(ok, "{current}");
    assert_eq!(current["pointer"]["version"], json!({"state":2}));
    assert_eq!(current["profile"]["max_context_bytes"], json!(32768));
    assert_ne!(
        current["revision"], shown["revision"],
        "revisions are immutable"
    );
    // The old pointer version is spent; a stale update cannot land on the new one.
    let (ok, spent) = f.run(&[
        "profile",
        "update",
        "--key",
        "profile-3",
        "--input",
        update_input,
    ]);
    assert!(!ok, "{spent}");
    assert_eq!(spent["error"]["code"], "VERSION_CONFLICT");
}
#[test]
fn b1_register_two_projects_native_rooms_same_card_contract_request_and_restart() {
    let (f, port) = Fixture::packaged("project-b1");
    assert!(f.run(&["start", "--secret-backend", "user-file"]).0);
    let input = json!({"name":"b1","origin":"local","platform":"local","platform_path":"b1","default_source":"gitea_issues"});
    let registered = accepted(&f, "repo", "register", "register-b1", input);
    let registration = &registered["registration"];
    let repo = registration["repo_id"].as_str().unwrap().to_owned();
    let version = registration["version"].as_i64().unwrap().to_string();
    let platform_id = registration["observed"]["stable_id"].as_str().unwrap();
    let args = [
        "repo",
        "register",
        "--confirm",
        &repo,
        "--version",
        &version,
        "--platform-repo-id",
        platform_id,
        "--key",
        "confirm-b1",
    ];
    let (ok, preview) = f.run(&args);
    assert!(ok, "{preview}");
    let mut args = args.to_vec();
    args.extend([
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    let (ok, result) = f.run(&args);
    assert!(ok, "{result}");
    assert_eq!(result["lifecycle"], "active");
    let mut projects = vec![];
    for name in ["A", "B"] {
        let create = json!({"repo_id":repo,"definition":project_definition(name)});
        let result = accepted(
            &f,
            "project",
            "create",
            &format!("project-{name}"),
            create.clone(),
        );
        assert_eq!(
            accepted(&f, "project", "create", &format!("project-{name}"), create)["project_id"],
            result["project_id"]
        );
        projects.push((
            result["project_id"].as_str().unwrap().to_owned(),
            result["main_room_id"].as_str().unwrap().to_owned(),
        ));
    }
    assert_ne!(projects[0].0, projects[1].0);
    assert_ne!(projects[0].1, projects[1].1);
    let source = accepted(
        &f,
        "task",
        "connect",
        "source-b1",
        json!({"repo_id":repo,"candidate_id":"gitea_issues","consent":true,"make_default":false}),
    );
    let source = source["source_id"].as_str().unwrap();
    let mut topics = vec![];
    for (p, main) in &projects {
        let show = f.show(p, main);
        assert_eq!(show["room"]["participants"], json!([]));
        let sent = accepted(
            &f,
            "room",
            "send",
            &format!("send-{p}"),
            json!({"project_id":p,"room_id":main,"version":show["binding"]["version"],"body":"中文前情"}),
        );
        let event = sent["receipt"]["event_id"].as_str().unwrap();
        let draft=f.query("draft",json!({"project_id":p,"project_version":1,"origin":{"kind":"room","room_id":main,"binding_version":show["binding"]["version"]},"selection":{"kind":"events","event_ids":[event]}}));
        let sources: Vec<Value> = draft["fragments"]
            .as_array()
            .unwrap_or_else(|| panic!("draft has no fragments: {draft}"))
            .iter()
            .map(|t| t["source"].clone())
            .collect();
        let topic = accepted(
            &f,
            "room",
            "create-topic",
            &format!("topic-{p}"),
            json!({"project_id":p,"project_version":1,"name":"topic","origin":{"kind":"room","room_id":main,"binding_version":show["binding"]["version"]},"brief":{"context_and_goal":"confirmed topic","settled_facts_and_reasons":[],"disagreements_and_questions":[],"constraints_and_materials":[],"sources":sources},"participants":[],"roster_confirmed":true}),
        );
        let room = topic["room_id"].as_str().unwrap().to_owned();
        f.query("save-view-state",json!({"project_id":p,"room_id":room,"binding_version":f.show(p,&room)["binding"]["version"],"client_id":"b1","draft":"未发送草稿","read_cursor":null}));
        topics.push(room);
        accepted(
            &f,
            "task",
            "attach",
            &format!("attach-{p}"),
            json!({"project_id":p,"project_version":1,"source_id":source,"approved_scope":platform_id,"consent":true}),
        );
    }
    // Patch 1a, from a human account's own view: a Topic created after the
    // human joined the main Room invites them by default, carries the
    // confirmed brief as its opening message, and the carrier Space follows
    // the main Room's membership.
    let registration_token =
        std::fs::read_to_string(f.root.join("services/config/tuwunel-registration-token")).unwrap();
    let registration =
        std::fs::read_to_string(f.root.join("services/config/appservices/hctl2.yaml")).unwrap();
    let as_token = serde_json::from_str::<Value>(&registration).unwrap()["as_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let server = chat::Server {
        binding: store::Reference {
            key: chat::key(store::Scope::Control, "chat_server", "packaged"),
            version: store::Version::State(1),
        },
        url: format!("http://127.0.0.1:{port}"),
        server_name: "hctl2.localhost".into(),
        sender: "@hctl2_control:hctl2.localhost".into(),
    };
    let matrix = control::MatrixClient::new(server, as_token).unwrap();
    let (human_id, human_token) = matrix
        .human_register("b1human", "b1-password", registration_token.trim())
        .unwrap();
    let (main_project, main_room) = &projects[0];
    let main_binding = f.show(main_project, main_room)["binding"].clone();
    accepted(
        &f,
        "project",
        "members",
        "members-human",
        json!({"project_id":main_project,"project_version":1,"rooms":[{"key":main_binding["key"],"version":{"state":main_binding["version"]}}],"users":[human_id],"invite":true}),
    );
    let main_external =
        f.show(main_project, main_room)["binding"]["data"]["value"]["matrix_room_id"]
            .as_str()
            .unwrap()
            .to_owned();
    matrix.human_join(&human_token, &main_external).unwrap();
    // One human-visible message in the main Room is the Topic's source.
    let human_send = accepted(
        &f,
        "room",
        "send",
        "send-human-view",
        json!({"project_id":main_project,"room_id":main_room,"version":f.show(main_project, main_room)["binding"]["version"],"body":"人的视角来源消息"}),
    );
    let human_event = human_send["receipt"]["event_id"].as_str().unwrap();
    let human_digest = {
        let content = json!({"body":"人的视角来源消息","msgtype":"m.text"});
        let canonical = foundation::canonical_json(&content).unwrap();
        foundation::bytes_sha256(&canonical)
    };
    let source_json = json!({"kind":"message","binding":{"key":main_binding["key"],"version":{"state":main_binding["version"]}},"event_id":human_event,"content_digest":human_digest});
    let human_brief = json!({"context_and_goal":"人的视角","settled_facts_and_reasons":[],"disagreements_and_questions":[],"constraints_and_materials":[],"sources":[source_json]});
    let human_topic = accepted(
        &f,
        "room",
        "create-topic",
        "topic-human-view",
        json!({"project_id":main_project,"project_version":1,"name":"人的视角","origin":{"kind":"room","room_id":main_room,"binding_version":f.show(main_project, main_room)["binding"]["version"]},"brief":human_brief,"participants":[],"roster_confirmed":true}),
    );
    assert_eq!(
        human_topic["invites"],
        json!([human_id]),
        "the confirmed list keeps naming users: {}",
        human_topic["invites"]
    );
    let invited: Vec<&str> = human_topic["invite_results"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|entry| {
            entry["receipt"]["members"]
                .as_array()
                .unwrap_or_else(|| panic!("invite entry without receipt: {entry}"))
                .iter()
        })
        .map(|member| member["user_id"].as_str().unwrap())
        .collect();
    assert_eq!(
        invited,
        vec![human_id.as_str()],
        "the joined human is the default invite list"
    );
    assert_eq!(human_topic["opening"]["delivery"], json!("confirmed"));
    assert_eq!(
        human_topic["invite_results"][0]["user_id"],
        json!(human_id),
        "per-target outcomes name their user"
    );
    assert_eq!(
        human_topic["invite_results"][0]["delivery"],
        json!("confirmed")
    );
    let human_topic_room = human_topic["room_id"].as_str().unwrap().to_owned();
    let topic_external =
        f.show(main_project, &human_topic_room)["binding"]["data"]["value"]["matrix_room_id"]
            .as_str()
            .unwrap()
            .to_owned();
    matrix.human_join(&human_token, &topic_external).unwrap();
    let messages = matrix
        .human_messages(&human_token, &topic_external)
        .unwrap();
    let bodies: Vec<&Value> = messages["chunk"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["type"] == "m.room.message")
        .map(|e| &e["content"]["body"])
        .collect();
    let brief_struct: chat::Brief = serde_json::from_value(human_brief.clone()).unwrap();
    assert_eq!(
        bodies.last(),
        Some(&&json!(chat::opening_body(&brief_struct))),
        "the human sees the confirmed brief as the opening message"
    );
    let hierarchy = f.run(&["room", "hierarchy", main_project, main_room]);
    assert!(hierarchy.0, "{:?}", hierarchy.1);
    let space = hierarchy.1["carrier_space_id"].as_str().unwrap();
    matrix.human_join(&human_token, space).unwrap();
    let space_membership = matrix
        .human_state(&human_token, space, "m.room.member", &human_id)
        .unwrap();
    assert_eq!(
        space_membership["membership"],
        json!("join"),
        "the carrier Space membership follows the main Room"
    );
    let created = accepted(
        &f,
        "task",
        "create",
        "task-b1",
        json!({"project_id":projects[0].0,"project_version":1,"source_id":source,"title":"shared native card","body":"work"}),
    );
    let task_a = created["task_id"].as_str().unwrap().to_owned();
    let (ok, task) = f.run(&["task", "show", &projects[0].0, &task_a]);
    assert!(ok, "{task}");
    let entity = task["data"]["entity"]["immutable_external_entity_id"]
        .as_str()
        .unwrap();
    accepted(
        &f,
        "task",
        "refresh",
        "refresh-b1",
        json!({"repo_id":repo,"source_id":source}),
    );
    let other = accepted(
        &f,
        "task",
        "claim",
        "claim-b1",
        json!({"project_id":projects[1].0,"project_version":1,"source_id":source,"entity_id":entity}),
    );
    let task_b = other["task_id"].as_str().unwrap().to_owned();
    assert_ne!(task_a, task_b);
    for ((p, _), task) in projects.iter().zip([&task_a, &task_b]) {
        let (ok, before) = f.run(&["task", "show", p, task]);
        assert!(ok, "{before}");
        let contract = json!({"scope":"B1","expected_outcome":"complete independently","acceptance":[{"text":"human approves","grade":"human"}],"roles":[],"capabilities":[]});
        let digest = foundation::canonical_json_sha256(&contract).unwrap();
        let adoption = json!({"contract":contract,"origin":{"kind":"local","reference":{"key":{"scope":{"kind":"project","id":p},"kind":"project","id":p},"version":{"state":1}},"proposal_digest":digest}});
        if p == &projects[0].0 {
            let request = accepted(
                &f,
                "request",
                "create",
                "request-b1",
                json!({"project_id":p,"project_version":1,"request":{
                    "question":"approve contract?","target":{"kind":"human","principal":format!("local-owner:{}",std::fs::metadata(&f.root).unwrap().uid())},
                    "owner":{"key":{"scope":{"kind":"project","id":p},"kind":"task_state","id":task},"version":{"state":before["version"]}},
                    "affected_revision":null,"blocking_scope":"contract_adoption","owner_state_version":before["data"]["state_version"],"dedup_root":"b1-contract",
                    "permissions":{"action":"task.adopt"},"deadline":null,"deadline_action":"fail_waiting","action":"task_adopt","input_schema":"hctl2.task.Adoption.v1"
                }}),
            );
            let (ok, pending) = f.run(&["project", "pending", p]);
            assert!(ok);
            assert_eq!(pending["items"].as_array().unwrap().len(), 1);
            let answer = json!({"project_id":p,"request_id":request["request_id"],"version":1,"adoption":adoption});
            let result = accepted(&f, "request", "resolve", "answer-b1", answer.clone());
            assert_eq!(result["delivery"], "confirmed");
            assert_eq!(
                accepted(&f, "request", "resolve", "answer-b1", answer)["request_id"],
                result["request_id"]
            );
            assert!(
                f.run(&["project", "pending", p]).1["items"]
                    .as_array()
                    .unwrap()
                    .is_empty()
            );
        } else {
            accepted(
                &f,
                "task",
                "adopt",
                &format!("adopt-{p}"),
                json!({"project_id":p,"project_version":1,"task_id":task,"version":before["version"],"adoption":adoption}),
            );
        }
    }
    let mut saved = vec![];
    for (index, task) in [&task_a, &task_b].into_iter().enumerate() {
        let p = &projects[index].0;
        let (ok, saved_task) = f.run(&["task", "show", p, task]);
        assert!(ok);
        let saved_room = f.show(p, &topics[index]);
        let (ok, saved_view) = f.run(&["room", "view-state", p, &topics[index], "b1"]);
        assert!(ok, "{saved_view}");
        saved.push((saved_task, saved_room, saved_view));
    }
    // Kill rather than graceful stop: restart must recover both authoritative and native state.
    let pid = std::fs::read_to_string(f.root.join("control.pid")).unwrap();
    assert!(
        Command::new("kill")
            .args(["-KILL", pid.trim()])
            .status()
            .unwrap()
            .success()
    );
    let (ok, services) = f.run(&["services", "status"]);
    assert!(!ok, "dead control should not answer: {services}");
    assert!(
        Command::new(f.payload.join("bin/hctl2-services"))
            .env("HCTL2_STATE_ROOT", f.root.join("services"))
            .arg("stop")
            .status()
            .unwrap()
            .success()
    );
    assert!(f.run(&["start", "--secret-backend", "user-file"]).0);
    let mut ready = false;
    for _ in 0..100 {
        let (ok, view) = f.run(&["room", "view-state", &projects[0].0, &topics[0], "b1"]);
        if ok {
            assert_eq!(view, saved[0].2);
            ready = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(ready, "native services did not recover");
    for (index, task) in [&task_a, &task_b].into_iter().enumerate() {
        let p = &projects[index].0;
        assert_eq!(f.show(p, &topics[index])["room"], saved[index].1["room"]);
        assert_eq!(f.run(&["task", "show", p, task]).1, saved[index].0);
        assert_eq!(
            f.run(&["room", "view-state", p, &topics[index], "b1"]).1,
            saved[index].2
        );
        assert_eq!(
            f.show(p, &projects[index].1)["room"]["id"],
            projects[index].1
        );
    }
}

#[test]
fn human_output_renders_dispatch_preview_sections_and_invocation_table() {
    let (f, setup) = paired("human-dispatch", 60);
    let p = setup.project.as_str();
    let room = setup.room.as_str();
    let path = f.root.join("invocation.json");
    let deadline = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 60000;
    std::fs::write(
        &path,
        json!({"project_id":p,"room_id":room,"target":"research","profile":setup.profile["revision"],
            "request":"Return the exact answer without modifying files","budget":65536,
            "deadline_ms":deadline,"retry_of":null})
        .to_string(),
    )
    .unwrap();
    let input = path.to_str().unwrap();
    let (ok, human, stderr) = f.run_raw(
        false,
        &[
            "invocation",
            "preview",
            "--input",
            input,
            "--key",
            "human-dispatch",
        ],
    );
    assert!(ok, "{}", String::from_utf8_lossy(&stderr));
    let human = String::from_utf8(human).unwrap();
    for heading in [
        "Dispatch preview",
        "Object",
        "Why this needs confirmation",
        "What happens after you confirm",
        "Confirm with",
    ] {
        assert!(
            human.contains(heading),
            "missing section {heading}:\n{human}"
        );
    }
    // The object section names the real dispatch targets, not placeholders.
    for value in [p, room, "research"] {
        assert!(
            human.contains(value),
            "object section must name {value}:\n{human}"
        );
    }
    assert!(
        human.contains("invocation cancel"),
        "the undo path must be named:\n{human}"
    );
    assert!(!human.contains('\u{1b}'), "no ANSI escapes:\n{human}");
    // The printed confirm command is executable as printed: take the line under
    // "Confirm with", strip the binary, run it.
    let confirm_line = human
        .lines()
        .skip_while(|line| !line.contains("Confirm with"))
        .nth(1)
        .expect("a confirm command line")
        .trim();
    assert!(confirm_line.contains("--preview-token"), "{confirm_line}");
    let words = confirm_line.split_whitespace().collect::<Vec<_>>();
    let confirmed = Command::new(words[0])
        .env_remove("HCTL2_PROCESS_COMPOSE_BIN")
        .env("HCTL2_INSTALL_ROOT", &f.payload)
        .env(
            "HCTL2_CONTROL_BIN",
            std::env::var("CARGO_BIN_EXE_hctl2-control").unwrap(),
        )
        .args(&words[1..])
        .output()
        .unwrap();
    assert!(
        confirmed.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&confirmed.stdout),
        String::from_utf8_lossy(&confirmed.stderr)
    );
    // The default invocation listing is a table with a header and this row.
    let (ok, listed, _) = f.run_raw(false, &["invocation", "list", p]);
    assert!(ok);
    let listed = String::from_utf8(listed).unwrap();
    let mut lines = listed.lines();
    let header = lines.next().expect("a header row");
    assert_eq!(
        header.split_whitespace().collect::<Vec<_>>(),
        ["invocation", "state", "version", "reason"]
    );
    let separator = lines.next().expect("a separator row");
    assert!(separator.starts_with("-----------"), "{separator}");
    let row = lines
        .next()
        .expect("one invocation row")
        .split_whitespace()
        .collect::<Vec<_>>();
    assert!(row[0].starts_with("invocation-"), "{row:?}");
    assert!(
        row[1] == "pending" || row[1] == "running",
        "the dispatched invocation is listed with its state: {row:?}"
    );
    assert!(!listed.contains('\u{1b}'));
    // The machine interface stays byte-stable where the content allows it. The
    // program-file digest differs across OS releases; check it against the
    // actual /bin/sh, while keeping every other field pinned to the sample.
    let (ok, first, _) = f.run_raw(true, &["agency", "catalog", "local"]);
    assert!(ok);
    let (_, second, _) = f.run_raw(true, &["agency", "catalog", "local"]);
    assert_eq!(
        first, second,
        "agency catalog --json must be deterministic in one fixture"
    );
    let catalog = String::from_utf8(first).unwrap();
    let digest = agency_proto::hash(&std::fs::read("/bin/sh").unwrap());
    let mut expected: Value = serde_json::from_str(AGENCY_CATALOG_JSON).unwrap();
    expected["harnesses"][0]["digest"] = json!(digest);
    expected["professions"][0]["harness"]["digest"] = json!(digest);
    expected["professions"][0]["reference"]["digest"] = json!(digest);
    assert_eq!(
        catalog.trim_end(),
        serde_json::to_string(&expected).unwrap(),
        "agency catalog --json drifted from the saved sample"
    );
    let (ok, human, _) = f.run_raw(false, &["agency", "catalog", "local"]);
    assert!(ok);
    let human = String::from_utf8(human).unwrap();
    assert!(human.contains("Professions"), "{human}");
    let header = human
        .lines()
        .find(|line| line.starts_with("profession"))
        .expect("a profession header row");
    assert_eq!(
        header.split_whitespace().collect::<Vec<_>>(),
        ["profession", "revision", "harness", "persona"],
        "the catalog is a table with headers:\n{human}"
    );
    assert!(human.contains("script-worker"), "{human}");
    assert!(!human.contains('\u{1b}'));
}

/// Codex stand-in from `agency/tests/codex_fixture.rs`. It speaks app-server
/// `turn/start`, writes a rollout whose `input_text` is that body, and answers
/// `ANSWER`. Herdr only displays `codex resume --remote`.
fn paired_codex(name: &str, codex_bin: &std::path::Path, private_home: bool) -> (Fixture, Paired) {
    let (f, _) = Fixture::packaged(name);
    assert!(f.run(&["start", "--secret-backend", "user-file"]).0);
    let registered = accepted(
        &f,
        "repo",
        "register",
        "register-dispatch",
        json!({"name":"dispatch","origin":"local","platform":"local","platform_path":"dispatch","default_source":"gitea_issues"}),
    );
    let registration = &registered["registration"];
    let repo = registration["repo_id"].as_str().unwrap();
    let version = registration["version"].to_string();
    let args = [
        "repo",
        "register",
        "--confirm",
        repo,
        "--version",
        &version,
        "--platform-repo-id",
        registration["observed"]["stable_id"].as_str().unwrap(),
        "--key",
        "confirm-dispatch",
    ];
    let (ok, plan) = f.run(&args);
    assert!(ok, "{plan}");
    let mut confirm = args.to_vec();
    confirm.extend(["--preview-token", plan["preview_token"].as_str().unwrap()]);
    assert!(f.run(&confirm).0);
    let created = accepted(
        &f,
        "project",
        "create",
        "project-dispatch",
        json!({"repo_id":repo,"definition":project_definition("Dispatch")}),
    );
    let project = created["project_id"].as_str().unwrap().to_owned();
    let room = created["main_room_id"].as_str().unwrap().to_owned();
    let agency_root = f.root.join("independent-agency");
    let delay = f.root.join("delay-script");
    let codex_home = f.root.join("codex-home");
    if private_home {
        std::fs::create_dir_all(&codex_home).unwrap();
    }
    let log = f.root.join("agency-serve.err");
    let binary = std::env::var_os("CARGO_BIN_EXE_agency").unwrap();
    let claude = std::env::var_os("HCTL2_STANDBY_FIXTURE").expect("HCTL2_STANDBY_FIXTURE");
    let mut command = Command::new(&binary);
    command
        .arg("--root")
        .arg(&agency_root)
        .arg("serve")
        .env("HCTL2_INSTALL_ROOT", &f.payload)
        .env("HCTL2_CLAUDE", &claude)
        .env("HCTL2_CODEX", codex_bin)
        .env("HCTL2_AGENCY_IDLE_MS", "600000");
    if private_home {
        command.env("CODEX_HOME", &codex_home);
    }
    let agency = AgencyChild(
        command
            .stdout(Stdio::null())
            .stderr(std::fs::File::create(&log).unwrap())
            .spawn()
            .unwrap(),
    );
    let mut ready = false;
    for _ in 0..400 {
        if Command::new(&binary)
            .arg("--root")
            .arg(&agency_root)
            .arg("status")
            .output()
            .unwrap()
            .status
            .success()
        {
            ready = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        ready,
        "agency did not become ready: {}",
        std::fs::read_to_string(&log).unwrap_or_default()
    );
    let (ok, binding) = f.run(&[
        "agency",
        "pair",
        "--binding-id",
        "local",
        "--agency-root",
        agency_root.to_str().unwrap(),
        "--key",
        "pair",
    ]);
    assert!(ok, "{binding}");
    let (ok, catalog) = f.run(&["agency", "catalog", "local"]);
    assert!(
        ok,
        "{catalog} {}",
        std::fs::read_to_string(&log).unwrap_or_default()
    );
    let profession = catalog["professions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["reference"]["id"] == "codex-cli")
        .cloned()
        .unwrap_or_else(|| panic!("codex-cli missing: {catalog}"));
    let profession_reference = f.root.join("profession.json");
    std::fs::write(&profession_reference, profession["reference"].to_string()).unwrap();
    let (ok, accepted_profession) = f.run(&[
        "agency",
        "accept",
        "--binding-id",
        "local",
        "--reference",
        profession_reference.to_str().unwrap(),
        "--key",
        "accept",
    ]);
    assert!(ok, "{accepted_profession}");
    let profile = accepted(
        &f,
        "profile",
        "create",
        "profile",
        json!({"id":"research","profile":{"harness":profession["harness"],"model":profession["model"],"mode":"read_only","permissions":["context.read"],"environment":[],"required_capabilities":agency_proto::Capabilities::default(),"max_context_bytes":65536}}),
    );
    let binding_ref =
        json!({"key":binding["binding"]["key"],"version":{"state":binding["binding"]["version"]}});
    let profession_ref = json!({"key":accepted_profession["profession"]["key"],"version":{"state":accepted_profession["profession"]["version"]}});
    let selection = json!({"room_id":room,"selected_item":profession_ref,"profession":profession_ref,"profession_digest":profession["reference"]["digest"],"agency":binding_ref,"required_skills":[],"optional_skills":[],"worker_profiles":[profile["revision"]],"responsibility":"research","permission":{"allow":["context.read"]},"budget":{"max_bytes":65536},"display_name":"Research","persona_tags":[]});
    accepted(
        &f,
        "project",
        "select",
        "select",
        json!({"project_id":project,"project_version":1,"room_id":room,"topic_command_key":null,"roster_version":null,"selections":[selection]}),
    );
    (
        f,
        Paired {
            agency,
            repo: repo.to_owned(),
            project,
            room,
            profession,
            profession_reference,
            profile,
            delay,
        },
    )
}

fn dispatch_once(f: &Fixture, setup: &Paired, key: &str, request: &str) -> (Value, Value) {
    let p = setup.project.as_str();
    let room = setup.room.as_str();
    let path = f.root.join(format!("{key}.json"));
    let deadline = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 180_000;
    std::fs::write(
        &path,
        json!({"project_id":p,"room_id":room,"target":"research","profile":setup.profile["revision"],"request":request,"budget":65536,"deadline_ms":deadline,"retry_of":null}).to_string(),
    )
    .unwrap();
    let (ok, plan) = f.run(&[
        "invocation",
        "preview",
        "--input",
        path.to_str().unwrap(),
        "--key",
        key,
    ]);
    assert!(ok, "{plan}");
    let (ok, started) = f.run(&[
        "invocation",
        "start",
        "--input",
        path.to_str().unwrap(),
        "--key",
        key,
        "--preview-token",
        plan["preview_token"].as_str().unwrap(),
    ]);
    assert!(ok, "{started}");
    let id = started["invocation_id"].as_str().unwrap();
    let mut completed = Value::Null;
    for _ in 0..180 {
        let (ok, value) = f.run(&["invocation", "show", p, id]);
        assert!(ok, "{value}");
        if value["state"] == "completed" || value["state"] == "failed" {
            completed = value;
            break;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    assert_eq!(completed["state"], "completed", "{completed}");
    (completed, plan)
}

#[test]
fn codex_fixture_dispatch_from_real_cli_admits_the_answer_into_the_room() {
    let codex = std::path::PathBuf::from(
        std::env::var_os("HCTL2_CODEX_FIXTURE").expect("HCTL2_CODEX_FIXTURE"),
    );
    let (f, setup) = paired_codex("codex-chain", &codex, true);
    let (completed, _) = dispatch_once(
        &f,
        &setup,
        "invoke-codex",
        "Reply with the fixture answer. Do not modify files.",
    );
    assert_eq!(completed["results"][0]["output"], "ANSWER");
    let mut seen = false;
    for _ in 0..60 {
        let (ok, timeline) = f.run(&["room", "timeline", &setup.project, &setup.room]);
        assert!(ok, "{timeline}");
        seen = timeline["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["content"]["body"] == "ANSWER");
        if seen {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(seen, "the Codex answer was not projected into the Room");
}

/// One logged-in Codex turn. Ignored unless `HCTL2_CODEX_LIVE=1`. Does not print credentials.
#[test]
#[ignore = "uses the local Codex login; set HCTL2_CODEX_LIVE=1"]
fn live_codex_dispatch_returns_one_answer_to_the_room() {
    if std::env::var_os("HCTL2_CODEX_LIVE").is_none() {
        return;
    }
    let which = Command::new("/usr/bin/which")
        .arg("codex")
        .output()
        .unwrap();
    assert!(which.status.success(), "codex is not on PATH");
    let codex = std::path::PathBuf::from(String::from_utf8(which.stdout).unwrap().trim());
    let rollout_after = std::time::SystemTime::now()
        .checked_sub(Duration::from_secs(30))
        .unwrap_or(std::time::UNIX_EPOCH);
    let (f, setup) = paired_codex("codex-live", &codex, false);
    let request = "Reply with exactly CODEX_ROOM_OK and no other text. Do not modify files.";
    let (completed, plan) = dispatch_once(&f, &setup, "invoke-live", request);
    let output = completed["results"][0]["output"].as_str().unwrap_or("");
    assert!(
        output.contains("CODEX_ROOM_OK"),
        "answer did not contain the marker"
    );
    let bundle: agency_proto::context::Bundle =
        serde_json::from_value(plan["effect_summary"]["assembly"]["bundle"]["document"].clone())
            .unwrap();
    let mut expected = String::new();
    for entry in &bundle.entries {
        let bytes = match &entry.delivery {
            agency_proto::context::Delivery::Inline { bytes }
            | agency_proto::context::Delivery::Pointer { bytes, .. } => bytes.as_slice(),
            agency_proto::context::Delivery::Recall { .. } => continue,
        };
        expected.push_str(std::str::from_utf8(bytes).unwrap());
        expected.push('\n');
    }
    assert!(!expected.is_empty(), "preview bundle had no task text");
    let timeline = ready_timeline(&f, &setup.project, &setup.room);
    assert!(
        timeline["events"].as_array().unwrap().iter().any(|event| {
            event["content"]["body"]
                .as_str()
                .is_some_and(|body| body.contains("CODEX_ROOM_OK"))
        }),
        "the Codex answer was not projected into the Room"
    );
    let home = std::env::var_os("HOME").expect("HOME");
    let sessions = std::path::PathBuf::from(home).join(".codex/sessions");
    let mut matched = false;
    if sessions.is_dir() {
        let mut stack = vec![sessions];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
                    continue;
                }
                let Ok(meta) = path.metadata() else {
                    continue;
                };
                if meta.modified().ok().is_none_or(|when| when < rollout_after) {
                    continue;
                }
                if rollout_input_equals(&path, &expected) {
                    matched = true;
                    break;
                }
            }
            if matched {
                break;
            }
        }
    }
    assert!(
        matched,
        "rollout input_text was not the dispatch body byte for byte"
    );
}

fn rollout_input_equals(path: &std::path::Path, text: &str) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let body = String::from_utf8_lossy(&bytes);
    for line in body.lines() {
        if !line.contains("input_text") {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value_has_input_text(&value, text) {
            return true;
        }
    }
    false
}

fn value_has_input_text(value: &Value, text: &str) -> bool {
    match value {
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str) == Some("input_text")
                && map.get("text").and_then(Value::as_str) == Some(text)
            {
                return true;
            }
            map.values().any(|child| value_has_input_text(child, text))
        }
        Value::Array(items) => items.iter().any(|child| value_has_input_text(child, text)),
        _ => false,
    }
}

/// Captured from the pre-renderer CLI with the same fixture; pins the machine
/// interface for `agency catalog`, which needs a live Agency.
const AGENCY_CATALOG_JSON: &str = r#"{"harnesses":[{"digest":"c0eaf44f9242d5bbc2e14f4e8b7dccc1eff7f2976d64dc18914d5ef9f373e100","id":"script-protocol-fixture","revision":"1"}],"professions":[{"capabilities":{"event_cursor":true,"exact_attach":false,"input":true,"input_provenance":true,"isolation_effects":[],"managed_single_writer":true,"secure_input":false,"stop":true,"tool_execution_unmediated":false},"default_role":"fixture","harness":{"digest":"c0eaf44f9242d5bbc2e14f4e8b7dccc1eff7f2976d64dc18914d5ef9f373e100","id":"script-protocol-fixture","revision":"1"},"model":"none","persona":"protocol test executor","reference":{"digest":"c0eaf44f9242d5bbc2e14f4e8b7dccc1eff7f2976d64dc18914d5ef9f373e100","id":"script-worker","revision":"1"},"skills":[],"terms":"not a coding harness; no PTY, tool provenance or OS hardening"}],"skills":[]}"#;

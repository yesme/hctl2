//! Real Agency RPC -> pooled native adapter -> private Git worktree.
//! Fixtures assert process effects; ignored tests use the installed coding harnesses.
use agency::{Agency, confine, herdr, launch::InstalledHerdr, runtime::Runtime};
use agency_proto::{
    client::Client,
    context::{Bundle, Delivery, Entry},
    *,
};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct Rig {
    temp: tempfile::TempDir,
    root: PathBuf,
    repo: PathBuf,
    baseline: String,
    runtime: Arc<InstalledHerdr>,
    client: Client,
    admin: Client,
    key: String,
    server: tokio::task::JoinHandle<Result<()>>,
}
impl Rig {
    async fn new(live: bool) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("credentials");
        let repo = temp.path().join("source");
        fs::create_dir_all(&repo).unwrap();
        git(&repo, &["init", "--initial-branch=main"]);
        git(&repo, &["config", "user.email", "trial@example.invalid"]);
        git(&repo, &["config", "user.name", "Trial"]);
        fs::write(
            repo.join("calculator.py"),
            "def double(value):\n    return value + 2\n",
        )
        .unwrap();
        fs::write(repo.join("test_calculator.py"), "import unittest\nfrom calculator import double\nclass CalculatorTest(unittest.TestCase):\n    def test_double(self):\n        self.assertEqual(double(3), 6)\n        self.assertEqual(double(0), 0)\n").unwrap();
        fs::write(
            repo.join(".gitignore"),
            "__pycache__/\nfixture-ready\nfixture-go\nfixture-runs\n",
        )
        .unwrap();
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-m", "baseline"]);
        let baseline = git(&repo, &["rev-parse", "HEAD"]).trim().to_owned();
        // The source has credentials/configuration the private copy must not inherit.
        git(&repo, &["config", "credential.helper", "!exit 97"]);
        git(
            &repo,
            &[
                "config",
                "remote.origin.url",
                "https://credential.invalid/private",
            ],
        );
        let bin = temp.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let (claude, codex) = if live {
            (on_path("claude"), on_path("codex"))
        } else {
            let claude = bin.join("claude");
            let codex = bin.join("codex");
            std::os::unix::fs::symlink(
                PathBuf::from(std::env::var("HCTL2_STANDBY_FIXTURE").unwrap())
                    .canonicalize()
                    .unwrap(),
                &claude,
            )
            .unwrap();
            std::os::unix::fs::symlink(
                PathBuf::from(std::env::var("HCTL2_CODEX_FIXTURE").unwrap())
                    .canonicalize()
                    .unwrap(),
                &codex,
            )
            .unwrap();
            let home = fixture_home();
            fs::create_dir_all(&home).unwrap();
            fs::write(home.join("mode"), "ok").unwrap();
            (claude, codex)
        };
        let herdr = PathBuf::from(std::env::var("HCTL2_LOCKED_HERDR").unwrap())
            .canonicalize()
            .unwrap();
        fs::set_permissions(&herdr, fs::Permissions::from_mode(0o755)).unwrap();
        let runtime = Arc::new(
            InstalledHerdr::open_with_codex(herdr, &claude, &codex, Duration::from_secs(300))
                .unwrap(),
        );
        let agency = Agency::open(&root, runtime.clone()).unwrap();
        let endpoint = agency_proto::client::admin_endpoint(&root).unwrap();
        let admin = Client::new(endpoint.clone(), Agency::bootstrap_key(&root).unwrap());
        let pairing = Client::new(endpoint, fs::read_to_string(root.join("pair.key")).unwrap());
        let server = tokio::spawn(agency::serve(agency));
        for _ in 0..100 {
            if admin.call::<_, Value>("catalog", &json!({})).await.is_ok() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let paired: Pairing = pairing
            .call(
                "pair",
                &Pair {
                    control_id: "trial-control".into(),
                    tenant_key: agency_proto::client::new_credential().unwrap(),
                },
            )
            .await
            .unwrap();
        let key = paired.key.clone();
        let client = Client::new(paired.endpoint.into(), paired.key);
        let _: Value = client
            .call(
                "fence",
                &Fence {
                    writer_generation: 1,
                },
            )
            .await
            .unwrap();
        Self {
            temp,
            root,
            repo,
            baseline,
            runtime,
            client,
            admin,
            key,
            server,
        }
    }

    async fn request(
        &self,
        harness: &str,
        name: &str,
        text: &str,
        mutate: impl FnOnce(&mut ExecutionSpec, &mut Value),
    ) -> (Prepare, Value) {
        let catalog: Catalog = self.client.call("catalog", &json!({})).await.unwrap();
        let profession = catalog
            .professions
            .into_iter()
            .find(|p| p.reference.id == harness)
            .unwrap();
        let owner = Owner {
            project: "trial-project".into(),
            kind: OwnerKind::RoomInvocation,
            id: format!("invocation-{name}"),
            generation: 1,
        };
        let active = json!({"lease_id":"lease-trial","generation":1,"state":"active","holder":{"kind":"invocation","invocation_id":owner.id,"invocation_version":1}});
        let lease = FrozenRef {
            id: "lease-trial".into(),
            revision: "1".into(),
            digest: hash(&canonical(&active).unwrap()),
        };
        let publication = json!({"repo_id":"trial-repo","binding_version":1,"branch_rule":"hctl2/{change_set}","target_branch":"main","allow_update":true,"description_source":"none","requires_human_confirmation":false,"audit_scope":"minimal"});
        let mut policy = frozen("trial-review-policy");
        policy.digest = hash(&canonical(&publication).unwrap());
        let mut pending = active;
        pending["state"] = json!("pending");
        let mut boundary = json!({"lease":{"pending":{"change_set_id":"cs-trial","repo_id":"trial-repo","binding_version":1,"baseline_commit":self.baseline,"version":1,"lease":pending},"previous":null},
            "review_publish_policy":policy, "authorization":"publish_for_review_not_integration", "repo_local_path":self.repo,"repo_local_machine":"control","objective":text,"publication_target":publication});
        let mut spec = ExecutionSpec {
            owner: owner.clone(),
            project: frozen("trial-project"),
            selection: frozen(harness),
            selection_policy_digest: hash(b"selection-policy"),
            profession,
            profile: frozen("write-profile"),
            manifest: frozen("manifest"),
            bundle: frozen("bundle"),
            binding: frozen("binding"),
            required_capabilities: Capabilities {
                stop: true,
                ..Capabilities::default()
            },
            input_policy: InputPolicy::NoInput,
            permission_digest: hash(b"write-permissions"),
            permissions: vec!["context.read".into(), "git.read".into(), "git.write".into()],
            budget: 1024 * 1024,
            deadline_ms: now_ms() + 300_000,
            repo: Some(frozen("trial-repo")),
            base: Some(self.baseline.clone()),
            delivery_scope: vec!["trial-room".into()],
            write_lease: Some(lease),
            review_publish_policy: Some(policy),
            idempotency_key: name.into(),
        };
        mutate(&mut spec, &mut boundary);
        let bytes = canonical(&boundary).unwrap();
        let bundle = Sealed::new(Bundle {
            id: "bundle".into(),
            manifest: spec.manifest.clone(),
            consumer: owner,
            entries: vec![
                entry("invocation-request", text.as_bytes().to_vec()),
                entry(&format!("write-boundary/{}", spec.owner.id), bytes),
            ],
            renderer: frozen("renderer"),
            tokenizer: frozen("tokenizer"),
            redaction: frozen("redaction"),
            compression: vec![],
            candidate_tokens: None,
            selected_tokens: None,
            delivered_tokens: None,
            permission_digest: spec.permission_digest.clone(),
            budget: spec.budget,
            retention: "trial".into(),
        })
        .unwrap();
        spec.bundle.digest = bundle.digest.clone();
        (
            Prepare {
                spec: Sealed::new(spec).unwrap(),
                bundle,
                writer_generation: 1,
            },
            boundary,
        )
    }

    async fn activate(&self, input: &Prepare) -> (Dispatch, Ticket) {
        let prepared: Dispatch = self.client.call("prepare", input).await.unwrap();
        let activated: Dispatch = self
            .client
            .call(
                "activate",
                &DispatchAction {
                    dispatch: prepared.reference.clone(),
                    writer_generation: 1,
                    idempotency_key: format!("activate:{}", input.spec.document.idempotency_key),
                },
            )
            .await
            .unwrap();
        let ticket = Ticket::sign(
            TicketClaims {
                id: format!("ticket:{}", prepared.reference),
                actor: "trial-human".into(),
                dispatch: prepared.reference.clone(),
                owner: prepared.owner,
                spec_digest: prepared.spec_digest,
                writer_generation: 1,
                permissions: vec![Permission::Observe, Permission::Stop],
                input_lease: None,
                expires_ms: now_ms() + 300_000,
            },
            self.key.as_bytes(),
        )
        .unwrap();
        (activated, ticket)
    }
    async fn trace(&self, ticket: &Ticket) -> Trace {
        self.client
            .call(
                "observe",
                &Observe {
                    ticket: ticket.clone(),
                    after: 0,
                },
            )
            .await
            .unwrap()
    }
    async fn finish(&self, ticket: &Ticket) -> Trace {
        let deadline = Instant::now() + Duration::from_secs(180);
        loop {
            let trace = self.trace(ticket).await;
            if trace.dispatch.state != DispatchState::Running
                || trace.events.iter().any(|e| e.kind == "runtime:seal_failed")
            {
                return trace;
            }
            assert!(
                Instant::now() < deadline,
                "{}",
                serde_json::to_string(&trace).unwrap()
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
    fn worktree(&self) -> PathBuf {
        let root = confine::execution_parent(&self.root)
            .unwrap()
            .join("write-worktrees");
        fs::read_dir(root)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
            .join("cs-trial")
            .canonicalize()
            .unwrap()
    }
    fn claude_state(&self, input: &Prepare, dispatch: &Dispatch) -> PathBuf {
        let exec = confine::execution_dir(&self.root, &dispatch.reference).unwrap();
        let state = herdr::state_dir(&exec, &self.root).unwrap();
        fn find(state: &Path, id: &str, depth: u8) -> Option<PathBuf> {
            if depth > 2 {
                return None;
            }
            for entry in fs::read_dir(state).ok()? {
                let path = entry.ok()?.path();
                if path.file_name()?.to_str()?.starts_with("standby-")
                    && fs::read(path.join("job.json"))
                        .is_ok_and(|b| serde_json::from_slice::<Value>(&b).unwrap()["id"] == id)
                {
                    return Some(path);
                }
                if path.is_dir()
                    && let Some(found) = find(&path, id, depth + 1)
                {
                    return Some(found);
                }
            }
            None
        }
        find(&state, &input.spec.document.idempotency_key, 0).expect("selection state")
    }
    async fn close(self) {
        let _: Value = self.admin.call("shutdown", &json!({})).await.unwrap();
        self.server.await.unwrap().unwrap();
        self.runtime.shutdown().unwrap();
        let _ = fs::remove_dir_all(confine::execution_parent(&self.root).unwrap());
    }
}

fn frozen(id: &str) -> FrozenRef {
    FrozenRef {
        id: id.into(),
        revision: "1".into(),
        digest: hash(id.as_bytes()),
    }
}
fn entry(id: &str, bytes: Vec<u8>) -> Entry {
    Entry {
        source: frozen(id),
        description: id.into(),
        required: true,
        offline_required: true,
        bytes_digest: hash(&bytes),
        delivery: Delivery::Inline { bytes },
    }
}
fn git(path: &Path, args: &[&str]) -> String {
    let out = Command::new("/usr/bin/git")
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .current_dir(path)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
fn on_path(name: &str) -> PathBuf {
    let out = Command::new("/usr/bin/which").arg(name).output().unwrap();
    assert!(out.status.success());
    PathBuf::from(String::from_utf8(out.stdout).unwrap().trim())
}
fn fixture_home() -> PathBuf {
    PathBuf::from(std::env::var("CODEX_HOME").unwrap())
}
fn json_file(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

async fn check_proposal(rig: &Rig, dispatch: &Dispatch, trace: &Trace, empty: bool) {
    let page: ResultPage = rig
        .client
        .call("results", &ResultQuery::of(dispatch.reference.clone()))
        .await
        .unwrap();
    assert_eq!(page.proposals.len(), 1, "{page:?}");
    let proposal = &page.proposals[0];
    assert_eq!(proposal.schema, repo::changeset::OUTPUT_SCHEMA);
    assert_eq!(proposal.evidence, EvidenceLevel::AdapterEvent);
    assert_eq!(proposal.outputs.len(), 1);
    let output: repo::changeset::Output = serde_json::from_slice(&proposal.output).unwrap();
    let report = &trace
        .events
        .iter()
        .find(|e| e.kind == "runtime:git_sealed")
        .expect("native tool seal readback")
        .payload;
    let cwd = rig.worktree();
    assert_eq!(output.change_set_id, "cs-trial");
    assert_eq!(output.base_commit_sha, rig.baseline);
    assert_eq!(output.parent_revision_id, None);
    assert_eq!(output.base_commit_sha, report["base_commit_sha"]);
    assert_eq!(
        report["base_tree_sha"],
        git(&cwd, &["rev-parse", &format!("{}^{{tree}}", rig.baseline)]).trim()
    );
    match output.location {
        repo::changeset::OutputLocation::Commit {
            repo_path,
            commit_sha,
        } => {
            assert!(!empty);
            assert_eq!(repo_path.canonicalize().unwrap(), cwd);
            assert_eq!(commit_sha, report["result_commit_sha"]);
            assert_eq!(
                git(&cwd, &["rev-parse", &format!("{commit_sha}^{{tree}}")]).trim(),
                report["result_tree_sha"]
            );
            assert_ne!(report["base_tree_sha"], report["result_tree_sha"]);
        }
        repo::changeset::OutputLocation::NoChanges { repo_path } => {
            assert!(empty);
            assert_eq!(repo_path.canonicalize().unwrap(), cwd);
            assert_eq!(report["base_tree_sha"], report["result_tree_sha"]);
        }
        _ => panic!("Agency must return an immutable commit or explicit empty result"),
    }
    // Independently seal the current working files through the public tool and
    // compare its tree with the fixed candidate. This is not a model assertion.
    let tool = PathBuf::from(std::env::var("HCTL2_TOOL_BIN").unwrap())
        .canonicalize()
        .unwrap();
    let readback = Command::new(tool)
        .args([
            "repo",
            "seal",
            "--path",
            cwd.to_str().unwrap(),
            "--change-set-ref",
            "cs-trial",
            "--baseline",
            &rig.baseline,
            "--key",
            &format!("verify-{}", dispatch.reference),
        ])
        .output()
        .unwrap();
    assert!(
        readback.status.success(),
        "{}{}",
        String::from_utf8_lossy(&readback.stdout),
        String::from_utf8_lossy(&readback.stderr)
    );
    let readback: Value = serde_json::from_slice(&readback.stdout).unwrap();
    assert_eq!(
        readback["result_tree_sha"], report["result_tree_sha"],
        "Proposal tree differs from worktree"
    );
    let retry = Command::new(
        PathBuf::from(std::env::var("HCTL2_TOOL_BIN").unwrap())
            .canonicalize()
            .unwrap(),
    )
    .args([
        "repo",
        "seal",
        "--path",
        cwd.to_str().unwrap(),
        "--change-set-ref",
        "cs-trial",
        "--baseline",
        &rig.baseline,
        "--key",
        &format!("agency-{}", dispatch.spec_digest),
    ])
    .output()
    .unwrap();
    assert!(retry.status.success());
    let retry: Value = serde_json::from_slice(&retry.stdout).unwrap();
    assert_eq!(retry["reused"], true);
    assert_eq!(retry["result_commit_sha"], report["result_commit_sha"]);
    eprintln!(
        "SEALED {}: base={} tree={} commit={} empty={empty}",
        dispatch.reference,
        output.base_commit_sha,
        report["result_tree_sha"],
        report["result_commit_sha"]
    );
}

async fn fixture_writes(harness: &str) {
    let rig = Rig::new(false).await;
    let keyring = rig.temp.path().join("keyrings/login.keyring");
    fs::create_dir_all(keyring.parent().unwrap()).unwrap();
    fs::write(&keyring, b"trial-keyring").unwrap();
    let mut paths = vec![rig.root.join("pair.key"), rig.repo.join(".git/config")];
    if cfg!(target_os = "linux") {
        paths.push(keyring);
    }
    let gh = PathBuf::from(std::env::var_os("HOME").unwrap()).join(".config/gh/hosts.yml");
    if gh.exists() {
        paths.push(gh);
    }
    let text = format!(
        "Fix double and run its tests.\nHCTL2_WRITE_FIXTURE {}",
        json!({"edit":true,"secret_paths":paths})
    );
    let (input, boundary) = rig.request(harness, harness, &text, |_, _| {}).await;
    let (dispatch, ticket) = rig.activate(&input).await;
    let trace = rig.finish(&ticket).await;
    assert_eq!(
        trace.dispatch.state,
        DispatchState::ResultReturned,
        "{trace:?}"
    );
    assert!(trace.events.iter().any(|e| e.kind == "turn_returned"));
    check_proposal(&rig, &dispatch, &trace, false).await;
    let worktree = rig.worktree();
    assert_eq!(
        fs::read_to_string(worktree.join("calculator.py")).unwrap(),
        "def double(value):\n    return value * 2\n"
    );
    assert_eq!(
        fs::read_to_string(rig.repo.join("calculator.py")).unwrap(),
        "def double(value):\n    return value + 2\n"
    );
    assert_eq!(git(&worktree, &["rev-parse", "HEAD"]).trim(), rig.baseline);
    assert_eq!(
        git(&worktree, &["symbolic-ref", "HEAD"]).trim(),
        "refs/heads/hctl2/changeset/cs-trial"
    );
    assert_eq!(
        git(
            &worktree,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"]
        )
        .trim(),
        rig.repo
            .join(".git")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
    );
    assert!(!worktree.parent().unwrap().join("repository.git").exists());
    let probe = if harness == "claude-code" {
        let state = rig.claude_state(&input, &dispatch);
        let started = json_file(&state.join("started.json"));
        assert!(
            started["text"]
                .as_str()
                .unwrap()
                .contains(&String::from_utf8(canonical(&boundary).unwrap()).unwrap())
        );
        json_file(&state.join("write-probe.json"))
    } else {
        let turn = json_file(&fixture_home().join("last-turn.json"));
        assert_eq!(turn["sandboxPolicy"]["type"], "externalSandbox");
        assert_eq!(turn["cwd"], json!(worktree));
        json_file(&fixture_home().join("write-probe.json"))
    };
    assert_eq!(
        PathBuf::from(probe["cwd"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        worktree.canonicalize().unwrap()
    );
    assert_eq!(
        probe["git_success"], false,
        "source Git metadata must stay outside the harness: {probe}"
    );
    assert_eq!(probe["test_success"], true, "{probe}");
    for read in probe["reads"].as_array().unwrap() {
        assert_eq!(read["readable"], false, "{probe}");
    }
    assert_eq!(probe["gh_authenticated"], false, "{probe}");
    assert_eq!(probe["gh_token_present"], false);
    assert_eq!(probe["dbus_present"], false);
    let results: ResultPage = rig
        .client
        .call("results", &ResultQuery::of(dispatch.reference.clone()))
        .await
        .unwrap();
    assert_eq!(results.proposals.len(), 1);
    let _: Dispatch = rig.client.call("stop", &ticket).await.unwrap();
    assert!(
        worktree.join("calculator.py").exists(),
        "revocation deleted unsealed edits"
    );
    let stop_deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let stopped = rig.trace(&ticket).await;
        if stopped
            .events
            .iter()
            .any(|e| e.kind == "stopped" && e.payload["session_closed"] == true)
        {
            break;
        }
        assert!(
            Instant::now() < stop_deadline,
            "write revocation lacks session-close evidence: {stopped:?}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    // An already materialized copy is recognized after detach; no second
    // worktree or reset is needed on the next lease. Git readback is Agency-side.
    git(&worktree, &["checkout", "--detach"]);
    let (again, _) = rig
        .request(
            harness,
            &format!("next-{harness}"),
            &text.replace("\"edit\":true", "\"edit\":false"),
            |spec, material| {
                material["lease"]["pending"]["lease"]["generation"] = json!(2);
                material["lease"]["pending"]["lease"]["lease_id"] = json!("lease-trial-2");
                let mut active = material["lease"]["pending"]["lease"].clone();
                active["state"] = json!("active");
                spec.write_lease = Some(FrozenRef {
                    id: "lease-trial-2".into(),
                    revision: "2".into(),
                    digest: hash(&canonical(&active).unwrap()),
                });
                // Unrelated Repo metadata is not another ChangeSet workspace.
                let repo = spec.repo.as_mut().unwrap();
                repo.revision = "2".into();
                repo.digest = hash(b"updated-repo-metadata");
            },
        )
        .await;
    let (next_dispatch, next_ticket) = rig.activate(&again).await;
    let next_trace = rig.finish(&next_ticket).await;
    assert_eq!(
        next_trace.dispatch.state,
        DispatchState::ResultReturned,
        "{next_trace:?}"
    );
    check_proposal(&rig, &next_dispatch, &next_trace, false).await;
    assert_eq!(
        git(&worktree, &["symbolic-ref", "HEAD"]).trim(),
        "refs/heads/hctl2/changeset/cs-trial"
    );
    assert_eq!(
        rig.worktree(),
        worktree,
        "another lease created another worktree for this ChangeSet"
    );
    assert_eq!(
        fs::read_dir(worktree.parent().unwrap().parent().unwrap())
            .unwrap()
            .count(),
        1
    );
    let _: Dispatch = rig.client.call("stop", &next_ticket).await.unwrap();
    let (mut readonly, _) = rig
        .request(
            harness,
            &format!("readonly-{harness}"),
            "read after write",
            |spec, _| {
                spec.permissions = vec!["context.read".into()];
                spec.write_lease = None;
                spec.review_publish_policy = None;
                spec.base = None;
                spec.repo = None;
            },
        )
        .await;
    readonly
        .bundle
        .document
        .entries
        .retain(|e| !e.source.id.starts_with("write-boundary/"));
    readonly.bundle = Sealed::new(readonly.bundle.document).unwrap();
    readonly.spec.document.bundle.digest = readonly.bundle.digest.clone();
    readonly.spec = Sealed::new(readonly.spec.document).unwrap();
    let (readonly_dispatch, readonly_ticket) = rig.activate(&readonly).await;
    let readonly_trace = rig.finish(&readonly_ticket).await;
    assert_eq!(
        readonly_trace.dispatch.state,
        DispatchState::ResultReturned,
        "{readonly_trace:?}"
    );
    let readonly_results: ResultPage = rig
        .client
        .call("results", &ResultQuery::of(readonly_dispatch.reference))
        .await
        .unwrap();
    assert_eq!(readonly_results.proposals.len(), 1);
    if harness == "codex-cli" {
        assert_eq!(
            json_file(&fixture_home().join("last-turn.json"))["sandboxPolicy"]["type"],
            "readOnly"
        );
    }
    eprintln!("{harness} write probe: {probe}");
    rig.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn claude_write_entry_uses_materialized_worktree_and_keeps_unsealed_edits() {
    let _guard = SERIAL.lock().await;
    fixture_writes("claude-code").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn codex_write_entry_uses_materialized_worktree_and_cannot_read_credentials() {
    let _guard = SERIAL.lock().await;
    fixture_writes("codex-cli").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn write_entry_rejects_mismatched_or_missing_bundle_authority() {
    let _guard = SERIAL.lock().await;
    let rig = Rig::new(false).await;
    for (index, code) in [
        "WRITE_BOUNDARY_MISMATCH",
        "WRITE_BOUNDARY_MISMATCH",
        "WRITE_MATERIAL_REQUIRED",
        "WRITE_MATERIAL_REQUIRED",
        "PERMISSION_DENIED",
        "WRITE_BOUNDARY_MISMATCH",
        "WRITE_MATERIAL_REQUIRED",
        "WRITE_BOUNDARY_MISMATCH",
        "WRITE_BOUNDARY_MISMATCH",
        "WRITE_BOUNDARY_MISMATCH",
    ]
    .iter()
    .enumerate()
    {
        let (input, _) = rig
            .request(
                "codex-cli",
                &format!("bad-{index}"),
                "do not run",
                |spec, material| match index {
                    0 => material["lease"]["pending"]["baseline_commit"] = json!("0".repeat(40)),
                    1 => {
                        material["lease"]["pending"]["lease"]["holder"]["invocation_id"] =
                            json!("another-invocation")
                    }
                    2 => {
                        material.as_object_mut().unwrap().remove("repo_local_path");
                    }
                    3 => {
                        material.as_object_mut().unwrap().remove("objective");
                    }
                    4 => spec.permissions.retain(|p| p != "git.write"),
                    5 => material["publication_target"]["target_branch"] = json!("other-target"),
                    6 => material["repo_local_machine"] = json!("another-machine"),
                    7 => spec.review_publish_policy = None,
                    8 => spec.write_lease = None,
                    9 => material["lease"]["pending"]["lease"] = json!([]),
                    _ => unreachable!(),
                },
            )
            .await;
        let (_, ticket) = rig.activate(&input).await;
        let trace = rig.finish(&ticket).await;
        assert_eq!(
            trace.dispatch.state,
            DispatchState::CannotFulfill,
            "{trace:?}"
        );
        assert!(
            trace.events.iter().any(|e| e.payload["code"] == *code),
            "{trace:?}"
        );
    }
    assert!(
        !confine::execution_parent(&rig.root)
            .unwrap()
            .join("write-worktrees")
            .exists()
    );
    rig.close().await;
}

async fn wait_returned(rig: &Rig, ticket: &Ticket) -> Trace {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let trace = rig.trace(ticket).await;
        if trace.dispatch.state == DispatchState::ResultReturned {
            return trace;
        }
        assert_eq!(trace.dispatch.state, DispatchState::Running, "{trace:?}");
        assert!(Instant::now() < deadline, "{trace:?}");
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn both_write_entries_return_verified_empty_result() {
    let _guard = SERIAL.lock().await;
    for harness in ["claude-code", "codex-cli"] {
        let rig = Rig::new(false).await;
        let (input, _) = rig
            .request(
                harness,
                &format!("empty-{harness}"),
                "Report no changes.\nHCTL2_WRITE_FIXTURE {\"edit\":false,\"secret_paths\":[]}",
                |_, _| {},
            )
            .await;
        let (dispatch, ticket) = rig.activate(&input).await;
        let trace = rig.finish(&ticket).await;
        assert_eq!(
            trace.dispatch.state,
            DispatchState::ResultReturned,
            "{trace:?}"
        );
        check_proposal(&rig, &dispatch, &trace, true).await;
        rig.close().await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn both_write_entries_preserve_failed_seal_and_retry_without_redispatch() {
    let _guard = SERIAL.lock().await;
    for (harness, recover) in [
        ("claude-code", true),
        ("codex-cli", true),
        ("claude-code", false),
        ("codex-cli", false),
    ] {
        let rig = Rig::new(false).await;
        let text =
            "Fix double.\nHCTL2_WRITE_FIXTURE {\"edit\":true,\"wait\":true,\"secret_paths\":[]}";
        let (input, _) = rig
            .request(harness, &format!("retry-{harness}"), text, |_, _| {})
            .await;
        let (dispatch, ticket) = rig.activate(&input).await;
        let deadline = Instant::now() + Duration::from_secs(30);
        let worktree = loop {
            let root = confine::execution_parent(&rig.root)
                .unwrap()
                .join("write-worktrees");
            if let Ok(entries) = fs::read_dir(root)
                && let Some(Ok(entry)) = entries.into_iter().next()
            {
                let cwd = entry.path().join("cs-trial");
                if cwd.join("fixture-ready").exists() {
                    break cwd.canonicalize().unwrap();
                }
            }
            assert!(
                Instant::now() < deadline,
                "fixture never started: {:?}",
                rig.trace(&ticket).await
            );
            tokio::time::sleep(Duration::from_millis(25)).await;
        };
        // An honest external Git operation changes the checked-out branch while
        // the harness is running. seal must preserve its native refusal.
        git(&worktree, &["checkout", "-b", "external-review"]);
        fs::write(worktree.join("fixture-go"), b"go").unwrap();
        let trace = rig.finish(&ticket).await;
        assert_eq!(trace.dispatch.state, DispatchState::Running, "{trace:?}");
        assert!(
            trace.events.iter().any(|e| e.kind == "runtime:seal_failed"
                && e.payload["code"] == "HCTL2_TOOL_WORKTREE_BRANCH_MOVED"),
            "{trace:?}"
        );
        assert!(!trace.events.iter().any(|e| e.kind == "turn_returned"));
        let page: ResultPage = rig
            .client
            .call("results", &ResultQuery::of(dispatch.reference.clone()))
            .await
            .unwrap();
        assert!(page.proposals.is_empty());
        assert!(
            worktree
                .parent()
                .unwrap()
                .join("pending-answer.json")
                .exists()
        );
        assert!(
            fs::read_to_string(worktree.join("calculator.py"))
                .unwrap()
                .contains("value * 2")
        );
        if !recover {
            let _: Dispatch = rig.client.call("stop", &ticket).await.unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let stopped = rig.trace(&ticket).await;
                if stopped
                    .events
                    .iter()
                    .any(|e| e.kind == "stopped" && e.payload["session_closed"] == true)
                {
                    assert_eq!(stopped.dispatch.state, DispatchState::Cancelled);
                    break;
                }
                assert!(Instant::now() < deadline, "{stopped:?}");
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            let page: ResultPage = rig
                .client
                .call("results", &ResultQuery::of(dispatch.reference.clone()))
                .await
                .unwrap();
            assert!(page.proposals.is_empty());
            assert!(worktree.join("calculator.py").exists());
            assert!(
                worktree
                    .parent()
                    .unwrap()
                    .join("pending-answer.json")
                    .exists()
            );
            rig.close().await;
            continue;
        }
        git(&worktree, &["checkout", "hctl2/changeset/cs-trial"]);
        let trace = wait_returned(&rig, &ticket).await;
        assert_eq!(
            fs::read_to_string(worktree.join("fixture-runs")).unwrap(),
            "1"
        );
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|e| e.kind == "runtime:session_opened")
                .count(),
            1
        );
        check_proposal(&rig, &dispatch, &trace, false).await;
        rig.close().await;
    }
}

async fn live_writes(harness: &str) {
    assert_eq!(
        std::env::var("HCTL2_HARNESS_LIVE").as_deref(),
        Ok("1"),
        "UNVERIFIED: local login required"
    );
    let rig = Rig::new(true).await;
    let text = "Fix the bug in calculator.py: double(value) must return twice its argument. Edit the non-documentation Python code, run /usr/bin/python3 -m unittest -v test_calculator, and report the command and its output. Leave all changes uncommitted. Do not push, access credentials, or edit harness global settings.";
    let (input, _) = rig
        .request(harness, &format!("live-{harness}"), text, |_, _| {})
        .await;
    let (dispatch, ticket) = rig.activate(&input).await;
    let trace = rig.finish(&ticket).await;
    eprintln!("LIVE {harness}: {}", serde_json::to_string(&trace).unwrap());
    let worktree = rig.worktree();
    let test = Command::new("/usr/bin/python3")
        .args(["-m", "unittest", "-v", "test_calculator"])
        .current_dir(&worktree)
        .output()
        .unwrap();
    assert!(
        test.status.success(),
        "{}",
        String::from_utf8_lossy(&test.stderr)
    );
    assert!(git(&worktree, &["diff", "--", "calculator.py"]).contains("value * 2"));
    assert_eq!(
        trace.dispatch.state,
        DispatchState::ResultReturned,
        "{trace:?}"
    );
    check_proposal(&rig, &dispatch, &trace, false).await;
    if harness == "claude-code" {
        let state = rig.claude_state(&input, &dispatch);
        let started = json_file(&state.join("started.json"));
        let root = PathBuf::from(std::env::var_os("HOME").unwrap()).join(".claude/projects");
        let (path, records) = native_records(&root, started["session"].as_str().unwrap());
        assert!(records.iter().any(|record| record["type"] == "user"
            && native_text_matches(
                &record["message"]["content"],
                started["text"].as_str().unwrap()
            )));
        assert!(records.iter().any(|record| {
            record["message"]["content"]
                .as_array()
                .is_some_and(|blocks| {
                    blocks.iter().any(|block| {
                        block["type"] == "tool_result"
                            && block["content"].as_str().is_some_and(|text| {
                                text.contains("test_double") && text.contains("OK")
                            })
                    })
                })
        }));
        eprintln!(
            "Claude own record: {}: exact user body and native test OK",
            path.display()
        );
        eprintln!(
            "Claude native started: {}",
            fs::read_to_string(state.join("started.json")).unwrap()
        );
        eprintln!(
            "Claude native returned: {}",
            fs::read_to_string(state.join("returned.json")).unwrap()
        );
    } else {
        let thread = trace
            .events
            .iter()
            .find_map(|event| event.payload["thread"].as_str())
            .unwrap();
        let root = PathBuf::from(std::env::var_os("CODEX_HOME").unwrap()).join("sessions");
        let (path, records) = native_records(&root, thread);
        assert!(records.iter().any(|record| {
            record["payload"]["role"] == "user"
                && record["payload"]["content"]
                    .as_array()
                    .is_some_and(|blocks| {
                        blocks.iter().any(|block| {
                            block["text"].as_str().is_some_and(|body| {
                                body.starts_with(text)
                                    && body.contains("Agency ChangeSet: cs-trial")
                                    && body.contains("Write lease: lease-trial generation 1")
                            })
                        })
                    })
        }));
        assert!(records.iter().any(|record| {
            let item = &record["payload"]["item"];
            item["type"] == "CommandExecution"
                && item["exit_code"] == 0
                && item["aggregated_output"]
                    .as_str()
                    .is_some_and(|output| output.contains("test_double") && output.contains("OK"))
        }));
        eprintln!(
            "Codex own record: {}: dispatched user body and native test exit 0",
            path.display()
        );
    }
    eprintln!(
        "Live worktree: {}\nTest: {}\nDiff: {}",
        worktree.display(),
        String::from_utf8_lossy(&test.stderr),
        git(&worktree, &["diff", "--", "calculator.py"])
    );
    rig.close().await;
}

fn native_text_matches(content: &Value, text: &str) -> bool {
    content.as_str() == Some(text)
        || content.as_array().is_some_and(|blocks| {
            blocks
                .iter()
                .any(|block| block["text"].as_str() == Some(text))
        })
}

fn native_records(root: &Path, session: &str) -> (PathBuf, Vec<Value>) {
    fn find(root: &Path, session: &str, depth: u8) -> Option<PathBuf> {
        if depth > 6 {
            return None;
        }
        for entry in fs::read_dir(root).ok()? {
            let path = entry.ok()?.path();
            if path.is_dir() {
                if let Some(path) = find(&path, session, depth + 1) {
                    return Some(path);
                }
            } else if path.file_name()?.to_str()?.contains(session)
                && path.extension()?.to_str()? == "jsonl"
            {
                return Some(path);
            }
        }
        None
    }
    let path = find(root, session, 0).expect("native harness session record");
    let records = fs::read_to_string(&path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (path, records)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "UNVERIFIED: installed Claude Code and native login required; HCTL2_HARNESS_LIVE=1"]
async fn live_claude_changes_non_documentation_code_and_runs_tests() {
    let _guard = SERIAL.lock().await;
    live_writes("claude-code").await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "UNVERIFIED: installed Codex and native login required; HCTL2_HARNESS_LIVE=1"]
async fn live_codex_changes_non_documentation_code_and_runs_tests() {
    let _guard = SERIAL.lock().await;
    live_writes("codex-cli").await;
}

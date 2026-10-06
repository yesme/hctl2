//! Local-target integration through the control socket: preview, submit, execution by
//! `hctl2-tool`, readback, Receipt; checked-out target waits for the human; head drift
//! under expected-head fails without retry.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use control::{ControlService, bind_owner_socket, serve_listener, socket_path};
use hyper_util::rt::TokioIo;
use proto::control_client::ControlClient;
use proto::{PreviewRequest, Protocol, QueryRequest, SubmitRequest};
use serde_json::{Value, json};
use store::Store;
use tokio::net::UnixStream;
use tokio::sync::Mutex;
use tonic::transport::Endpoint;
use tower::service_fn;

static TEMPS: AtomicU64 = AtomicU64::new(0);
const PROTOCOL: &str = "hctl2.control.v1";

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "hctl2-integ-{}-{}",
            std::process::id(),
            TEMPS.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@example.invalid")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@example.invalid")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

/// A repository with `main` checked out, plus a `work` branch holding one more commit.
fn seed_repository(root: &Path) -> (PathBuf, String, String, String) {
    let repo = root.join("apollo");
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    // The tool's merge commits use the repository's configured identity; runners have none.
    git(&repo, &["config", "user.name", "HCTL2 Test"]);
    git(&repo, &["config", "user.email", "hctl2@example.invalid"]);
    std::fs::write(repo.join("README.md"), "one\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "one"]);
    let base = git(&repo, &["rev-parse", "HEAD"]);
    git(&repo, &["switch", "-q", "-c", "work"]);
    std::fs::write(repo.join("README.md"), "one\ntwo\n").unwrap();
    git(&repo, &["commit", "-q", "-am", "two"]);
    let result_commit = git(&repo, &["rev-parse", "HEAD"]);
    let result_tree = git(&repo, &["rev-parse", "HEAD^{tree}"]);
    git(&repo, &["switch", "-q", "main"]);
    (repo, base, result_commit, result_tree)
}

async fn connect(root: &Path) -> ControlClient<tonic::transport::Channel> {
    let path = socket_path(root);
    for _ in 0..80 {
        if UnixStream::connect(&path).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    let channel = Endpoint::from_static("http://hctl2.control")
        .connect_with_connector(service_fn(move |_: tonic::transport::Uri| {
            let path = path.clone();
            async move { Ok::<_, std::io::Error>(TokioIo::new(UnixStream::connect(path).await?)) }
        }))
        .await
        .unwrap();
    ControlClient::new(channel)
}

fn proto() -> Protocol {
    Protocol {
        version: PROTOCOL.into(),
    }
}

struct Harness {
    client: ControlClient<tonic::transport::Channel>,
    store: Arc<Mutex<Option<Store>>>,
    root: PathBuf,
}

impl Harness {
    async fn preview(&mut self, key: &str, input: &Value) -> (String, Value) {
        let mut input = input.clone();
        input["key"] = json!(key);
        let response = self
            .client
            .preview(PreviewRequest {
                protocol: Some(proto()),
                operation: "integration.submit".into(),
                payload: serde_json::to_vec(&input).unwrap(),
                command_id: format!("integration:{key}"),
            })
            .await
            .unwrap()
            .into_inner();
        if let Some(error) = response.error {
            return (
                String::new(),
                json!({"error": {"code": error.code, "message": error.message}}),
            );
        }
        (
            response.preview_token,
            serde_json::from_slice(&response.effect_summary).unwrap(),
        )
    }
    async fn submit(&mut self, key: &str, input: &Value, token: &str) -> Value {
        let mut input = input.clone();
        input["key"] = json!(key);
        let response = self
            .client
            .submit(SubmitRequest {
                protocol: Some(proto()),
                operation: "integration.submit".into(),
                payload: serde_json::to_vec(&input).unwrap(),
                command_id: format!("integration:{key}"),
                idempotency_key: key.into(),
                preview_token: token.into(),
            })
            .await
            .unwrap()
            .into_inner();
        if let Some(error) = response.error {
            return json!({"error": {"code": error.code, "message": error.message}});
        }
        serde_json::from_slice(&response.result).unwrap()
    }
    async fn show(&mut self, repo_id: &str, intent_id: &str) -> Value {
        let response = self
            .client
            .query(QueryRequest {
                protocol: Some(proto()),
                kind: "integration.show".into(),
                payload: serde_json::to_vec(&json!({"repo_id": repo_id, "intent_id": intent_id}))
                    .unwrap(),
            })
            .await
            .unwrap()
            .into_inner();
        assert!(response.error.is_none(), "{:?}", response.error);
        serde_json::from_slice(&response.payload).unwrap()
    }
    /// One pass of the background worker, exactly as the service runs it.
    async fn reconcile(&self) {
        control::integration::reconcile_once(&self.store, &self.root, &mut HashMap::new())
            .await
            .unwrap();
    }
}

async fn harness() -> (Temp, Harness, String, PathBuf, String, String, String) {
    let temp = Temp::new();
    let root = temp.0.join("control");
    std::fs::create_dir_all(&root).unwrap();
    let (repo_path, base, result_commit, result_tree) = seed_repository(&temp.0);
    let mut store = Store::open(&root).unwrap();
    let actor = store::TrustedActor(store::Actor {
        principal: "owner".into(),
        source: store::ActorSource::DirectClient,
        permission_scope: vec![store::Scope::Control],
        authority: None,
    });
    // A Repo registered from this local repository with no platform: local targets only.
    let input = repo::LocalInput {
        machine: "control".into(),
        path: repo_path.clone(),
        in_place: false,
        extra_refs: Vec::new(),
        publish_governance: false,
    };
    let snapshot = repo::git::Git::discover().unwrap().inspect(&input).unwrap();
    let request = repo::Register {
        name: "apollo".into(),
        origin: repo::Origin::Local,
        platform: Some(repo::Platform::None),
        instance: None,
        platform_repo_id: None,
        platform_path: None,
        local: Some(input),
        remote_evidence: None,
        default_source: None,
    };
    let prepared = repo::prepare(request, Some(snapshot)).unwrap();
    let registration = repo::admit(&mut store, &actor, "register", "register", prepared).unwrap();
    assert_eq!(registration.lifecycle, repo::Lifecycle::Active);
    let repo_id = registration.repo_id.clone();
    // 测试缝：the other half of 第 6 包 admits revisions; here the admitted version is seeded.
    repo::integration::admit_revision_seam(
        &mut store,
        &actor,
        &repo_id,
        &repo::integration::AdmittedRevision {
            change_set_revision_id: "rev-1".into(),
            change_set_id: "cs-1".into(),
            parent_revision_id: None,
            base_commit_sha: base.clone(),
            result_tree_sha: result_tree.clone(),
            producer_ref: json!({"kind": "human_command", "command_id": "seal"}),
            review_subject_digest: "d".repeat(64),
        },
    )
    .unwrap();
    let status = store.startup_status();
    let store = Arc::new(Mutex::new(Some(store)));
    let listener = bind_owner_socket(&socket_path(&root)).unwrap();
    let service = ControlService::new(
        root.clone(),
        status,
        Arc::clone(&store),
        Arc::new(Mutex::new(None)),
    );
    tokio::spawn(async move {
        let _ = serve_listener(listener, service).await;
    });
    let client = connect(&root).await;
    (
        temp,
        Harness {
            client,
            store,
            root,
        },
        repo_id,
        repo_path,
        base,
        result_commit,
        result_tree,
    )
}

fn input(repo_id: &str, target_ref: &str, form: &str) -> Value {
    json!({
        "repo_id": repo_id,
        "change_set_revision_id": "rev-1",
        "target_kind": "local",
        "target_ref": target_ref,
        "form": form,
        "strategy": "fast_forward",
    })
}

#[tokio::test]
async fn checked_out_target_waits_for_the_human_then_the_same_intent_integrates_and_signs_one_receipt()
 {
    let (_temp, mut h, repo_id, repo_path, base, result_commit, result_tree) = harness().await;
    let input = input(&repo_id, "refs/heads/main", "expected_head");
    // Preview freezes the target head seen now and says what will be checked.
    let (token, preview) = h.preview("one", &input).await;
    assert_eq!(preview["expected_head"], base, "{preview}");
    assert_eq!(preview["source"]["result_tree_sha"], result_tree);
    assert_eq!(preview["target"]["kind"], "local");
    assert_eq!(
        Path::new(preview["target"]["provider_ref"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        repo_path.canonicalize().unwrap()
    );
    assert!(preview["checks"].as_array().unwrap().len() >= 3);
    // Submitting with an invented token must not create an intent.
    let refused = h.submit("one", &input, "invented").await;
    assert_eq!(refused["error"]["code"], "PREVIEW_REQUIRED");
    let submitted = h.submit("one", &input, &token).await;
    assert_eq!(submitted["state"], "pending", "{submitted}");
    let id = submitted["intent_id"].as_str().unwrap().to_owned();
    // The target is checked out by the primary worktree: the tool rejects, the intent waits.
    h.reconcile().await;
    let shown = h.show(&repo_id, &id).await;
    assert_eq!(shown["intent"]["state"], "unknown", "{shown}");
    assert_eq!(shown["intent"]["attention"]["code"], "TARGET_CHECKED_OUT");
    let paths =
        shown["intent"]["attention"]["details"]["tool"]["error"]["details"]["worktree_paths"]
            .clone();
    assert!(
        paths.as_array().is_some_and(|p| p.iter().any(|w| {
            w.as_str()
                .is_some_and(|w| Path::new(w).canonicalize().ok() == repo_path.canonicalize().ok())
        })),
        "exact worktree path expected: {shown}"
    );
    assert!(shown["receipt"].is_null());
    assert_eq!(
        git(&repo_path, &["rev-parse", "refs/heads/main"]),
        base,
        "target untouched"
    );
    // A second intent on the same target is refused while the first is unresolved.
    let (token2, _) = h.preview("two", &input).await;
    let busy = h.submit("two", &input, &token2).await;
    assert_eq!(busy["error"]["code"], "TARGET_BUSY", "{busy}");
    // The human switches the worktree away; the same intent is retried, nothing new is submitted.
    git(&repo_path, &["switch", "-q", "--detach"]);
    h.reconcile().await;
    let shown = h.show(&repo_id, &id).await;
    assert_eq!(shown["intent"]["state"], "succeeded", "{shown}");
    assert_eq!(shown["intent"]["attempts"], 2);
    assert!(shown["intent"]["attention"].is_null());
    let receipt = &shown["receipt"];
    assert_eq!(receipt["target_head_before"], base);
    assert_eq!(receipt["target_head_after"], result_commit);
    assert_eq!(receipt["integrated_commit"], result_commit);
    assert_eq!(receipt["integrated_tree"], result_tree);
    assert_eq!(receipt["evidence_level"], "hctl2-tool");
    assert_eq!(receipt["readback"]["status"], "applied");
    assert_eq!(shown["effect_state"], "confirmed");
    assert_eq!(
        git(&repo_path, &["rev-parse", "refs/heads/main"]),
        result_commit
    );
    // Nothing further happens on later passes. Re-previewing the same key replays the frozen
    // preview (the target is not read again, although it moved) and submits to the same intent.
    h.reconcile().await;
    let (token, replay) = h.preview("one", &input).await;
    assert_eq!(replay["expected_head"], base, "{replay}");
    let again = h.submit("one", &input, &token).await;
    assert_eq!(again["intent_id"], id, "{again}");
    let mut other = input.clone();
    other["strategy"] = json!("merge_commit");
    let (_, conflict) = h.preview("one", &other).await;
    assert_eq!(
        conflict["error"]["code"], "IDEMPOTENCY_CONFLICT",
        "{conflict}"
    );
    assert_eq!(h.show(&repo_id, &id).await["intent"]["attempts"], 2);
    // The target is settled: a new authorization on it is accepted now.
    let later = h.submit("two", &input, &token2).await;
    assert_eq!(later["state"], "pending", "{later}");
}

#[tokio::test]
async fn expected_head_drift_fails_without_retry_and_accept_advance_records_the_actual_head() {
    let (_temp, mut h, repo_id, repo_path, base, result_commit, result_tree) = harness().await;
    // Two targets that nobody has checked out.
    git(&repo_path, &["branch", "release", &base]);
    git(&repo_path, &["branch", "nightly", &base]);
    let frozen = input(&repo_id, "refs/heads/release", "expected_head");
    let (token, preview) = h.preview("frozen", &frozen).await;
    assert_eq!(preview["expected_head"], base);
    let id = h.submit("frozen", &frozen, &token).await["intent_id"]
        .as_str()
        .unwrap()
        .to_owned();
    // Someone advances the target between preview and execution.
    std::fs::write(repo_path.join("OTHER"), "x\n").unwrap();
    git(&repo_path, &["switch", "-q", "release"]);
    git(&repo_path, &["add", "OTHER"]);
    git(&repo_path, &["commit", "-q", "-m", "moved"]);
    let moved = git(&repo_path, &["rev-parse", "HEAD"]);
    git(&repo_path, &["switch", "-q", "--detach"]);
    h.reconcile().await;
    let shown = h.show(&repo_id, &id).await;
    assert_eq!(shown["intent"]["state"], "failed", "{shown}");
    assert_eq!(shown["intent"]["failure"]["code"], "TARGET_HEAD_MISMATCH");
    assert!(shown["receipt"].is_null());
    assert_eq!(shown["effect_state"], "rejected");
    assert_eq!(
        git(&repo_path, &["rev-parse", "refs/heads/release"]),
        moved,
        "nothing written"
    );
    // Failed is terminal: another pass does not retry it.
    h.reconcile().await;
    assert_eq!(h.show(&repo_id, &id).await["intent"]["attempts"], 1);
    // Accept-advance with a merge commit: the target advances after preview and the
    // Receipt records the head that was actually produced, not the previewed one. The
    // executor's branch is gone, so the admitted (base, tree) is packaged into a new commit.
    git(&repo_path, &["branch", "-D", "work"]);
    let mut advance = input(&repo_id, "refs/heads/nightly", "accept_advance");
    advance["strategy"] = json!("merge_commit");
    let (token, preview) = h.preview("advance", &advance).await;
    assert!(preview["expected_head"].is_null());
    assert_eq!(preview["observed_head"], base);
    let id = h.submit("advance", &advance, &token).await["intent_id"]
        .as_str()
        .unwrap()
        .to_owned();
    git(&repo_path, &["update-ref", "refs/heads/nightly", &moved]);
    h.reconcile().await;
    let shown = h.show(&repo_id, &id).await;
    assert_eq!(shown["intent"]["state"], "succeeded", "{shown}");
    let receipt = &shown["receipt"];
    assert_eq!(receipt["target_head_before"], moved);
    let after = receipt["target_head_after"].as_str().unwrap().to_owned();
    assert_ne!(after, moved);
    assert_ne!(after, result_commit);
    assert_eq!(git(&repo_path, &["rev-parse", "refs/heads/nightly"]), after);
    // The merge commit has the moved head and a commit carrying exactly the admitted tree
    // as parents; that commit is a fresh wrapper, not the deleted executor commit.
    let parents = git(&repo_path, &["rev-list", "--parents", "-n", "1", &after]);
    let parents: Vec<&str> = parents.split_whitespace().skip(1).collect();
    assert!(parents.contains(&moved.as_str()), "{parents:?}");
    let wrapper = parents
        .iter()
        .find(|p| **p != moved)
        .expect("second parent");
    assert_ne!(*wrapper, result_commit);
    assert_eq!(
        git(&repo_path, &["rev-parse", &format!("{wrapper}^{{tree}}")]),
        result_tree
    );
    assert_eq!(receipt["readback"]["status"], "applied");
}

#[tokio::test]
async fn previews_refuse_what_the_repo_cannot_offer() {
    let (_temp, mut h, repo_id, _repo_path, _base, _commit, _tree) = harness().await;
    let mut platform = input(&repo_id, "refs/heads/main", "accept_advance");
    platform["target_kind"] = json!("platform");
    let (_, refused) = h.preview("p", &platform).await;
    assert_eq!(refused["error"]["code"], "PLATFORM_NOT_BOUND", "{refused}");
    let missing = input(&repo_id, "refs/heads/nowhere", "expected_head");
    let (_, refused) = h.preview("m", &missing).await;
    assert_eq!(refused["error"]["code"], "TARGET_HEAD_UNKNOWN", "{refused}");
    let mut unknown_revision = input(&repo_id, "refs/heads/main", "expected_head");
    unknown_revision["change_set_revision_id"] = json!("rev-9");
    let (_, refused) = h.preview("r", &unknown_revision).await;
    assert_eq!(
        refused["error"]["code"], "REVISION_NOT_ADMITTED",
        "{refused}"
    );
}

#[tokio::test]
async fn a_fresh_clone_at_the_same_path_is_not_the_frozen_target() {
    let (temp, mut h, repo_id, repo_path, base, result_commit, _tree) = harness().await;
    git(&repo_path, &["branch", "release", &base]);
    let input = input(&repo_id, "refs/heads/release", "expected_head");
    let (token, preview) = h.preview("clone", &input).await;
    assert!(
        preview["target"]["continuity"]["inode"].is_number(),
        "{preview}"
    );
    let id = h.submit("clone", &input, &token).await["intent_id"]
        .as_str()
        .unwrap()
        .to_owned();
    // The directory is moved away and a fresh clone with the same refs takes its place.
    let moved_away = temp.0.join("apollo-moved");
    std::fs::rename(&repo_path, &moved_away).unwrap();
    git(
        &temp.0,
        &[
            "clone",
            "-q",
            "--no-local",
            moved_away.to_str().unwrap(),
            repo_path.to_str().unwrap(),
        ],
    );
    git(&repo_path, &["branch", "release", &base]);
    git(&repo_path, &["branch", "work", &result_commit]);
    git(&repo_path, &["switch", "-q", "--detach"]);
    assert_eq!(git(&repo_path, &["rev-parse", "refs/heads/release"]), base);
    h.reconcile().await;
    let shown = h.show(&repo_id, &id).await;
    assert_eq!(shown["intent"]["state"], "unknown", "{shown}");
    assert_eq!(
        shown["intent"]["attention"]["code"], "TARGET_IDENTITY_MISMATCH",
        "{shown}"
    );
    assert!(shown["receipt"].is_null());
    assert!(
        shown["intent"]["attempt"].is_null(),
        "no attempt is planned against a stranger"
    );
    assert_eq!(
        git(&repo_path, &["rev-parse", "refs/heads/release"]),
        base,
        "clone untouched"
    );
    assert_eq!(
        git(&moved_away, &["rev-parse", "refs/heads/release"]),
        base,
        "original untouched"
    );
    // The original repository returns to its path: the same intent proceeds.
    std::fs::remove_dir_all(&repo_path).unwrap();
    std::fs::rename(&moved_away, &repo_path).unwrap();
    h.reconcile().await;
    let shown = h.show(&repo_id, &id).await;
    assert_eq!(shown["intent"]["state"], "succeeded", "{shown}");
    assert_eq!(
        git(&repo_path, &["rev-parse", "refs/heads/release"]),
        result_commit
    );
}

#[tokio::test]
async fn a_confirmation_lost_after_the_write_is_recovered_from_the_frozen_attempt_with_one_receipt()
{
    let (_temp, mut h, repo_id, repo_path, base, result_commit, _tree) = harness().await;
    git(&repo_path, &["branch", "nightly", &base]);
    git(&repo_path, &["switch", "-q", "--detach"]);
    let mut advance = input(&repo_id, "refs/heads/nightly", "accept_advance");
    advance["strategy"] = json!("merge_commit");
    let (token, _) = h.preview("lost", &advance).await;
    let id = h.submit("lost", &advance, &token).await["intent_id"]
        .as_str()
        .unwrap()
        .to_owned();
    // Someone advances the target; control plans the attempt against the moved head …
    std::fs::write(repo_path.join("OTHER"), "x\n").unwrap();
    git(&repo_path, &["switch", "-q", "nightly"]);
    git(&repo_path, &["add", "OTHER"]);
    git(&repo_path, &["commit", "-q", "-m", "moved"]);
    let moved = git(&repo_path, &["rev-parse", "HEAD"]);
    git(&repo_path, &["switch", "-q", "--detach"]);
    // Planning takes the store's blocking lock, as the worker does off the async runtime.
    let planned = {
        let store = Arc::clone(&h.store);
        let (repo_id, id) = (repo_id.clone(), id.clone());
        tokio::task::spawn_blocking(move || {
            control::integration::plan_attempt(&store, &repo_id, &id)
        })
        .await
        .unwrap()
        .unwrap()
    };
    let control::integration::Planned::Local { attempt, .. } = planned else {
        panic!("local plan expected");
    };
    assert_eq!(attempt.expected_head, moved);
    assert_eq!(
        h.show(&repo_id, &id).await["intent"]["attempt"]["expected_head"],
        moved
    );
    // … the tool writes the merge, and control dies before it reads the result back.
    let written = tool::run(
        [
            "integrate",
            "--repo",
            repo_path.to_str().unwrap(),
            "--commit",
            &attempt.commit,
            "--base-commit-sha",
            &base,
            "--result-tree-sha",
            &git(
                &repo_path,
                &["rev-parse", &format!("{result_commit}^{{tree}}")],
            ),
            "--target-ref",
            "refs/heads/nightly",
            "--expected-head",
            &attempt.expected_head,
            "--strategy",
            "merge-commit",
            "--idempotency-key",
            &attempt.idempotency_key,
        ]
        .into_iter()
        .map(std::ffi::OsString::from),
    )
    .unwrap();
    assert_eq!(written.exit_code(), 0, "{}", written.body());
    let merged = git(&repo_path, &["rev-parse", "refs/heads/nightly"]);
    assert_ne!(merged, moved);
    // The next pass retries the frozen attempt: the tool recognises its own result, nothing is
    // re-planned against the new head, and exactly one Receipt is signed.
    h.reconcile().await;
    let shown = h.show(&repo_id, &id).await;
    assert_eq!(shown["intent"]["state"], "succeeded", "{shown}");
    assert_eq!(shown["receipt"]["target_head_after"], merged);
    assert_eq!(shown["receipt"]["readback"]["status"], "already_applied");
    assert_eq!(shown["intent"]["attempts"], 1);
    assert_eq!(
        git(&repo_path, &["rev-parse", "refs/heads/nightly"]),
        merged,
        "no second write"
    );
    assert_eq!(shown["effect_state"], "confirmed");
}

#[tokio::test]
async fn accept_advance_fast_forward_reaches_a_target_that_already_moved_to_the_candidate() {
    let (_temp, mut h, repo_id, repo_path, base, result_commit, result_tree) = harness().await;
    git(&repo_path, &["branch", "nightly", &base]);
    git(&repo_path, &["switch", "-q", "--detach"]);
    let advance = input(&repo_id, "refs/heads/nightly", "accept_advance");
    let (token, _) = h.preview("ff", &advance).await;
    let id = h.submit("ff", &advance, &token).await["intent_id"]
        .as_str()
        .unwrap()
        .to_owned();
    // Another writer pushes exactly the candidate before control runs.
    git(
        &repo_path,
        &["update-ref", "refs/heads/nightly", &result_commit],
    );
    h.reconcile().await;
    let shown = h.show(&repo_id, &id).await;
    assert_eq!(shown["intent"]["state"], "succeeded", "{shown}");
    assert_eq!(shown["receipt"]["readback"]["status"], "already_applied");
    assert_eq!(shown["receipt"]["target_head_after"], result_commit);
    assert_eq!(shown["receipt"]["integrated_tree"], result_tree);
}

#[tokio::test]
async fn a_confirmation_lost_under_expected_head_recovers_with_the_original_pre_write_head() {
    let (_temp, mut h, repo_id, repo_path, base, result_commit, _tree) = harness().await;
    git(&repo_path, &["branch", "frozen", &base]);
    git(&repo_path, &["switch", "-q", "--detach"]);
    let mut input = input(&repo_id, "refs/heads/frozen", "expected_head");
    input["strategy"] = json!("merge_commit");
    let (token, preview) = h.preview("lost-frozen", &input).await;
    assert_eq!(preview["expected_head"], base);
    let id = h.submit("lost-frozen", &input, &token).await["intent_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let planned = {
        let store = Arc::clone(&h.store);
        let (repo_id, id) = (repo_id.clone(), id.clone());
        tokio::task::spawn_blocking(move || {
            control::integration::plan_attempt(&store, &repo_id, &id)
        })
        .await
        .unwrap()
        .unwrap()
    };
    let control::integration::Planned::Local { attempt, .. } = planned else {
        panic!("local plan expected");
    };
    assert_eq!(attempt.expected_head, base);
    // The tool writes the merge; control never gets to confirm.
    let written = tool::run(
        [
            "integrate",
            "--repo",
            repo_path.to_str().unwrap(),
            "--commit",
            &attempt.commit,
            "--base-commit-sha",
            &base,
            "--result-tree-sha",
            &git(
                &repo_path,
                &["rev-parse", &format!("{result_commit}^{{tree}}")],
            ),
            "--target-ref",
            "refs/heads/frozen",
            "--expected-head",
            &attempt.expected_head,
            "--strategy",
            "merge-commit",
            "--idempotency-key",
            &attempt.idempotency_key,
        ]
        .into_iter()
        .map(std::ffi::OsString::from),
    )
    .unwrap();
    assert_eq!(written.exit_code(), 0, "{}", written.body());
    let merged = git(&repo_path, &["rev-parse", "refs/heads/frozen"]);
    assert_ne!(merged, base);
    // The retry reads the tool's own earlier result; the Receipt's pre-write head is still A.
    h.reconcile().await;
    let shown = h.show(&repo_id, &id).await;
    assert_eq!(shown["intent"]["state"], "succeeded", "{shown}");
    assert_eq!(shown["receipt"]["target_head_before"], base);
    assert_eq!(shown["receipt"]["target_head_after"], merged);
    assert_eq!(shown["receipt"]["readback"]["status"], "already_applied");
    assert_eq!(shown["intent"]["attempts"], 1);
    assert_eq!(
        git(&repo_path, &["rev-parse", "refs/heads/frozen"]),
        merged,
        "no second write"
    );
    // Genuine drift before any write still fails under expected_head (see the drift test).
}

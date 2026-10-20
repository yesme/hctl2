//! Dagu v1 OpenAPI client, controlled workflow engine port, and Dagu adapter.

#![forbid(unsafe_code)]

pub mod adapter;
pub mod driver;
pub mod generated {
    #![allow(
        clippy::all,
        clippy::pedantic,
        clippy::useless_borrows_in_formatting,
        dead_code,
        unreachable_pub,
        unused_imports
    )]
    include!("generated.rs");
}
pub mod model;

pub use adapter::{DaguAdapter, bootstrap_builtin_auth, verify_loopback_url};
pub use driver::{
    CompleteCheckpointParams, PollReconcileParams, ReconcileOutcome, StartRunParams,
    bind_and_start_run, cancel_run, complete_checkpoint, get_binding, get_controlled_run,
    poll_and_reconcile, reject_engine_self_advance,
};
pub use model::*;

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command as ProcessCommand, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;

    use serde_json::json;
    use store::{
        Actor, ActorSource, Command, Expected, ObjectKey, Record, RecordData, Reference, Scope,
        Store, TrustedActor, Version,
    };

    use super::*;

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "hctl2-dagu-api-{prefix}-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&dir).expect("create temp dir");
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn find_dagu_binary(root: &Path) -> Option<PathBuf> {
        for entry in std::fs::read_dir(root).ok()? {
            let path = entry.ok()?.path();
            if path.file_name().is_some_and(|n| n == "dagu") && path.is_file() {
                return Some(path);
            }
            if path.is_dir()
                && let Some(found) = find_dagu_binary(&path)
            {
                return Some(found);
            }
        }
        None
    }

    struct LiveDagu {
        _temp: TempDir,
        child: Child,
        base_url: String,
    }

    impl LiveDagu {
        async fn spawn() -> Self {
            let cache_dir =
                std::env::var("HCTL2_TEST_DAGU_CACHE").expect("HCTL2_TEST_DAGU_CACHE env");
            let dagu_bin =
                find_dagu_binary(Path::new(&cache_dir)).expect("locate packaged dagu binary");
            let temp = TempDir::new("live");
            let dagu_home = temp.0.join("dagu-home");
            for sub in [
                "dags",
                "data/dag-runs",
                "data/proc",
                "data/queue",
                "data/service-registry",
                "data/users",
                "data/api-keys",
                "logs",
            ] {
                std::fs::create_dir_all(dagu_home.join(sub)).expect("create dagu subdir");
            }
            let alloc_port = || {
                let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
                listener.local_addr().expect("local addr").port()
            };
            let port = alloc_port();
            let scheduler_port = alloc_port();
            let coordinator_port = alloc_port();
            let coordinator_health_port = alloc_port();

            let base_url = format!("http://127.0.0.1:{port}");
            let child = ProcessCommand::new(&dagu_bin)
                .args([
                    "start-all",
                    "--host",
                    "127.0.0.1",
                    "--port",
                    &port.to_string(),
                    "--dags",
                    dagu_home.join("dags").to_str().expect("dags path"),
                ])
                .env("HOME", &temp.0)
                .env("DAGU_HOME", &dagu_home)
                .env("DAGU_HOST", "127.0.0.1")
                .env("DAGU_PORT", port.to_string())
                .env("DAGU_SCHEDULER_PORT", scheduler_port.to_string())
                .env("DAGU_COORDINATOR_HOST", "127.0.0.1")
                .env("DAGU_COORDINATOR_PORT", coordinator_port.to_string())
                .env(
                    "DAGU_COORDINATOR_HEALTH_PORT",
                    coordinator_health_port.to_string(),
                )
                .env("DAGU_TZ", "UTC")
                .env("DAGU_AUTH_MODE", "builtin")
                .env(
                    "DAGU_AUTH_TOKEN_SECRET",
                    "test-dagu-token-secret-32-bytes-minimum",
                )
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("spawn dagu start-all");

            let live = Self {
                _temp: temp,
                child,
                base_url,
            };
            for _ in 0..60 {
                if let Ok(resp) = reqwest::get(format!("{}/api/v1/health", live.base_url)).await
                    && resp.status().is_success()
                {
                    return live;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            panic!("dagu start-all did not become healthy on {}", live.base_url);
        }
    }

    impl Drop for LiveDagu {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    fn owner_actor() -> TrustedActor {
        TrustedActor(Actor {
            principal: "owner".to_owned(),
            source: ActorSource::DirectClient,
            permission_scope: vec![Scope::Control, Scope::Project("P".to_owned())],
            authority: None,
        })
    }

    fn seed_compiled_deployment(
        store: &mut Store,
        workflow_id: &str,
        revision: i64,
        two_steps: bool,
    ) {
        let mut steps = vec![json!({
            "id": "cp_1",
            "name": "cp_1",
            "depends": [],
            "action": "human.task",
            "with": {
                "prompt": "HCTL passive checkpoint cp_1. Control supplies the admitted receipt; no execution authority.",
                "form": {
                    "type": "object",
                    "properties": {
                        "obligation_id": {"type": "string"},
                        "attempt_generation": {"type": "integer"},
                        "verdict": {"type": "string"},
                        "receipt_id": {"type": "string"}
                    },
                    "required": ["obligation_id", "attempt_generation", "verdict", "receipt_id"]
                }
            }
        })];
        if two_steps {
            steps.push(json!({
                "id": "cp_2",
                "name": "cp_2",
                "depends": ["cp_1"],
                "action": "human.task",
                "with": {
                    "prompt": "HCTL passive checkpoint cp_2 after (${steps.cp_1.outputs.receipt_id}).",
                    "form": {
                        "type": "object",
                        "properties": {
                            "obligation_id": {"type": "string"},
                            "attempt_generation": {"type": "integer"},
                            "verdict": {"type": "string"},
                            "receipt_id": {"type": "string"}
                        },
                        "required": ["obligation_id", "attempt_generation", "verdict", "receipt_id"]
                    }
                }
            }));
        }
        let yaml =
            serde_json::to_string_pretty(&json!({ "steps": steps })).expect("serialize yaml");

        let scope = Scope::Project("P".to_owned());
        let dep_id = format!("dep_{workflow_id}_{revision}");
        let dep_key = ObjectKey {
            scope: scope.clone(),
            kind: "engine_deployment".to_owned(),
            id: dep_id,
        };
        let dep_val = json!({
            "compiled": {
                "yaml": yaml
            }
        });
        let dep_record = Record {
            key: dep_key.clone(),
            version: 1,
            revision_digest: foundation::canonical_json_sha256(&dep_val).expect("digest"),
            data: RecordData::Value { value: dep_val },
            sources: Vec::new(),
            materials: Vec::new(),
        };
        let dep_ref = Reference {
            key: dep_key,
            version: Version::State(1),
        };
        let ptr_key = ObjectKey {
            scope: scope.clone(),
            kind: "workflow_deployment".to_owned(),
            id: format!("{workflow_id}:{revision}"),
        };
        let ptr_val = serde_json::to_value(&dep_ref).expect("ref json");
        let ptr_record = Record {
            key: ptr_key.clone(),
            version: 1,
            revision_digest: foundation::canonical_json_sha256(&ptr_val).expect("digest"),
            data: RecordData::Value { value: ptr_val },
            sources: Vec::new(),
            materials: Vec::new(),
        };
        let actor = owner_actor();
        let cmd_id = format!("seed:{workflow_id}:{revision}");
        let cmd = Command {
            command_id: cmd_id.clone(),
            idempotency_key: cmd_id,
            actor: actor.0.clone(),
            target: ptr_key,
            expected: Expected::Absent,
            binding: Reference {
                key: ObjectKey {
                    scope: Scope::Control,
                    kind: "module".to_owned(),
                    id: "run".to_owned(),
                },
                version: Version::State(1),
            },
            operation: "seed.deployment".to_owned(),
            input_digest: Command::digest_input("seed.deployment", &json!({})).expect("digest"),
            input: json!({}),
        };
        store
            .submit(store.generation(), &actor, &cmd, None, |tx| {
                tx.put(&dep_record)?;
                tx.put(&ptr_record)?;
                Ok(json!({}))
            })
            .expect("seed deployment");
    }

    #[test]
    fn generated_client_is_byte_for_byte_reproducible_and_loopback_enforced() {
        if let Ok(generated_path) = std::env::var("HCTL2_DAGU_GENERATED_CHECK") {
            let regenerated =
                std::fs::read(&generated_path).expect("read Buck generated_check output");
            let checked_in = include_bytes!("generated.rs");
            assert_eq!(
                regenerated.as_slice(),
                checked_in.as_slice(),
                "checked-in src/generated.rs differs from Buck :generated_check output"
            );
        }

        assert_eq!(
            verify_loopback_url("http://127.0.0.1:8180").expect("loopback ipv4"),
            "http://127.0.0.1:8180/api/v1"
        );
        assert_eq!(
            verify_loopback_url("http://localhost:8180/api/v1/").expect("loopback localhost"),
            "http://localhost:8180/api/v1"
        );
        assert_eq!(
            verify_loopback_url("http://[::1]:8180").expect("loopback ipv6"),
            "http://[::1]:8180/api/v1"
        );
        for non_loopback in [
            "http://0.0.0.0:8180",
            "http://192.168.1.20:8180",
            "https://127.0.0.1:8180",
            "http://dagu.example.com:8180",
        ] {
            let err = verify_loopback_url(non_loopback).expect_err("must reject non-loopback");
            assert_eq!(err.code, "WORKFLOW_ENGINE_NON_LOOPBACK");
        }
    }

    #[tokio::test]
    async fn dagu_port_live_roundtrip_unknown_ack_and_divergence_without_back_propagating_run() {
        let live = LiveDagu::spawn().await;
        let api_key = bootstrap_builtin_auth(
            &live.base_url,
            "hctl-admin",
            "hctl-admin-password-123",
            "control-service-key",
        )
        .await
        .expect("bootstrap builtin auth and mint dagu_ API key");
        assert!(api_key.starts_with("dagu_"));

        let adapter = DaguAdapter::new(&live.base_url, &api_key).expect("create DaguAdapter");
        assert_eq!(
            adapter.health_check().await.expect("health check"),
            "healthy"
        );

        let store_dir = TempDir::new("store");
        let mut store = Store::open(&store_dir.0).expect("open store");
        let actor = owner_actor();
        seed_compiled_deployment(&mut store, "wf", 1, true);

        // 1. Bind and start Run via control Command + EffectIntent (CT-RUN #8, 验收 1).
        let (run, binding, initial_rb) = bind_and_start_run(
            &mut store,
            &actor,
            &adapter,
            &StartRunParams {
                command_key: "start-1".to_owned(),
                project_id: "P".to_owned(),
                run_id: "r1".to_owned(),
                binding_id: "b1".to_owned(),
                workflow_id: "wf".to_owned(),
                revision: 1,
                port_provider_binding: "dagu:local".to_owned(),
                enqueue: true,
                now_ms: 1_000,
                progress_timeout_ms: 10_000,
            },
        )
        .await
        .expect("bind and start run");
        assert_eq!(run.state, RunState::Running);
        assert_eq!(binding.state, BindingState::Bound);
        assert_eq!(initial_rb.status, 7);

        // Reject engine self-advance without control command (CT-RUN #8, 验收 1).
        let self_advance_err =
            reject_engine_self_advance(&store, "P", "r1", "cp_1").expect_err("must reject");
        assert_eq!(self_advance_err.code, "RUN_ADVANCE_REQUIRES_COMMAND");
        let unchanged_run = get_controlled_run(&store, "P", "r1").expect("get run");
        assert!(unchanged_run.completed_checkpoints.is_empty());
        assert!(unchanged_run.verdicts.is_empty());
        assert!(unchanged_run.receipts.is_empty());

        // 2. Monotonic observation_seq and rejection of cached / late / stale cursor observations (CT-RUN #11, #18, 验收 3).
        let cached_poll = poll_and_reconcile(
            &mut store,
            &actor,
            &adapter,
            &PollReconcileParams {
                command_key: "poll-cache".to_owned(),
                project_id: "P".to_owned(),
                binding_id: "b1".to_owned(),
                now_ms: 1_100,
                from_cache: true,
                incoming_event_seq: None,
                incoming_cursor: None,
            },
        )
        .await
        .expect("cached poll");
        assert_eq!(cached_poll.observations.len(), 1);
        assert!(!cached_poll.observations[0].admissible_for_obligation);
        assert_eq!(
            cached_poll.observations[0].rejection_reason.as_deref(),
            Some("cached_status_not_admissible")
        );

        let late_poll = poll_and_reconcile(
            &mut store,
            &actor,
            &adapter,
            &PollReconcileParams {
                command_key: "poll-late".to_owned(),
                project_id: "P".to_owned(),
                binding_id: "b1".to_owned(),
                now_ms: 1_200,
                from_cache: false,
                incoming_event_seq: Some(1),
                incoming_cursor: None,
            },
        )
        .await
        .expect("late poll");
        assert!(!late_poll.observations[0].admissible_for_obligation);
        assert_eq!(
            late_poll.observations[0].rejection_reason.as_deref(),
            Some("late_event_seq_regression")
        );

        let stale_cursor_poll = poll_and_reconcile(
            &mut store,
            &actor,
            &adapter,
            &PollReconcileParams {
                command_key: "poll-stale-cursor".to_owned(),
                project_id: "P".to_owned(),
                binding_id: "b1".to_owned(),
                now_ms: 1_300,
                from_cache: false,
                incoming_event_seq: None,
                incoming_cursor: Some(0),
            },
        )
        .await
        .expect("stale cursor poll");
        assert!(!stale_cursor_poll.observations[0].admissible_for_obligation);
        assert_eq!(
            stale_cursor_poll.observations[0]
                .rejection_reason
                .as_deref(),
            Some("stale_cursor_regression")
        );
        assert!(
            stale_cursor_poll.observations[0].observation_seq
                > late_poll.observations[0].observation_seq
        );

        // 3. Unknown-ack readback-before-retry (spec/run.md §从节点到结果 第 5 条, 验收 4):
        // Case A: Step cp_1 is still Waiting(7) -> readback sees status 7 and submits completion.
        let (_, _, res_waiting) = complete_checkpoint(
            &mut store,
            &actor,
            &adapter,
            &CompleteCheckpointParams {
                command_key: "complete-cp1-unknown-ack".to_owned(),
                project_id: "P".to_owned(),
                binding_id: "b1".to_owned(),
                step_id: "cp_1".to_owned(),
                obligation_id: "obl-1".to_owned(),
                attempt_generation: 1,
                verdict: "accepted".to_owned(),
                receipt_id: "rcpt-1".to_owned(),
                ack_unknown: true,
                now_ms: 1_400,
            },
        )
        .await
        .expect("complete cp_1 with unknown ack");
        assert!(res_waiting.readback_performed);
        assert_eq!(res_waiting.readback_status_before, Some(7));
        assert!(res_waiting.resubmitted);
        assert_eq!(
            res_waiting.action_taken,
            CompletionRecoveryAction::ResubmittedAfterWaitingReadback
        );

        // Wait for cp_2 to enter Waiting(7) and cp_1 to be Succeeded(4).
        for _ in 0..40 {
            let step2 = adapter
                .readback_step(&binding.dag_name, &binding.dag_run_id, "cp_2")
                .await
                .expect("readback cp_2");
            if step2.status == 7 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(80)).await;
        }

        // Case B: Step cp_1 has already advanced to Succeeded(4) -> readback sees status 4 and does NOT resubmit.
        let (_, _, res_advanced) = complete_checkpoint(
            &mut store,
            &actor,
            &adapter,
            &CompleteCheckpointParams {
                command_key: "complete-cp1-already-advanced".to_owned(),
                project_id: "P".to_owned(),
                binding_id: "b1".to_owned(),
                step_id: "cp_1".to_owned(),
                obligation_id: "obl-1".to_owned(),
                attempt_generation: 1,
                verdict: "accepted".to_owned(),
                receipt_id: "rcpt-1".to_owned(),
                ack_unknown: true,
                now_ms: 1_500,
            },
        )
        .await
        .expect("recover cp_1 after already advanced");
        assert!(res_advanced.readback_performed);
        assert_eq!(res_advanced.readback_status_before, Some(4));
        assert!(!res_advanced.resubmitted);
        assert_eq!(
            res_advanced.action_taken,
            CompletionRecoveryAction::ConfirmedAdvancedWithoutResubmit
        );

        // 4. Progress timeout marks binding Diverged(EngineProgressStalled) without advancing Run (CT-RUN #18, 验收 3).
        let stalled = poll_and_reconcile(
            &mut store,
            &actor,
            &adapter,
            &PollReconcileParams {
                command_key: "poll-stall".to_owned(),
                project_id: "P".to_owned(),
                binding_id: "b1".to_owned(),
                now_ms: 50_000,
                from_cache: false,
                incoming_event_seq: None,
                incoming_cursor: None,
            },
        )
        .await
        .expect("poll stall");
        assert_eq!(stalled.binding.state, BindingState::Diverged);
        assert_eq!(
            stalled.binding.divergence.as_ref().map(|d| d.kind),
            Some(DirectOperationKind::EngineProgressStalled)
        );
        assert!(stalled.run.needs_attention);
        assert_eq!(stalled.run.state, RunState::Running);

        // 5. All 9 direct Dagu operations mark RunEngineBinding as Diverged without back-propagating Run (CT-RUN #9, 验收 2).
        let ops = [
            ("start", DirectOperationKind::Start),
            ("stop", DirectOperationKind::Stop),
            ("retry", DirectOperationKind::Retry),
            ("reschedule", DirectOperationKind::Reschedule),
            ("approve", DirectOperationKind::Approve),
            ("reject", DirectOperationKind::Reject),
            ("edit", DirectOperationKind::Edit),
            ("rename", DirectOperationKind::Rename),
            ("delete", DirectOperationKind::Delete),
        ];
        for (idx, (op_name, expected_kind)) in ops.into_iter().enumerate() {
            let wf_id = format!("wf_div_{idx}");
            seed_compiled_deployment(&mut store, &wf_id, 1, false);
            let run_id = format!("run_div_{idx}");
            let binding_id = format!("bind_div_{idx}");
            let (_, div_binding, _) = bind_and_start_run(
                &mut store,
                &actor,
                &adapter,
                &StartRunParams {
                    command_key: format!("start-div-{idx}"),
                    project_id: "P".to_owned(),
                    run_id: run_id.clone(),
                    binding_id: binding_id.clone(),
                    workflow_id: wf_id,
                    revision: 1,
                    port_provider_binding: "dagu:local".to_owned(),
                    enqueue: false,
                    now_ms: 1_000,
                    progress_timeout_ms: 60_000,
                },
            )
            .await
            .expect("start divergence test run");

            adapter
                .perform_direct_dagu_operation(
                    op_name,
                    &div_binding.dag_name,
                    &div_binding.dag_run_id,
                    Some("cp_1"),
                    None,
                )
                .await
                .expect("perform direct Dagu operation");

            let outcome = poll_and_reconcile(
                &mut store,
                &actor,
                &adapter,
                &PollReconcileParams {
                    command_key: format!("poll-div-{idx}"),
                    project_id: "P".to_owned(),
                    binding_id: binding_id.clone(),
                    now_ms: 1_500,
                    from_cache: false,
                    incoming_event_seq: None,
                    incoming_cursor: None,
                },
            )
            .await
            .expect("poll after direct Dagu operation");

            assert_eq!(
                outcome.binding.state,
                BindingState::Diverged,
                "direct op {op_name} must mark binding Diverged"
            );
            assert_eq!(
                outcome.binding.divergence.as_ref().map(|d| d.kind),
                Some(expected_kind),
                "direct op {op_name} must record expected DirectOperationKind"
            );
            assert!(outcome.run.needs_attention);
            // Direct Dagu operations must NEVER back-propagate Run state, checkpoints, verdicts, or receipts.
            assert_eq!(
                outcome.run.state,
                RunState::Running,
                "direct op {op_name} must not mutate ControlledRun.state"
            );
            assert!(outcome.run.completed_checkpoints.is_empty());
            assert!(outcome.run.verdicts.is_empty());
            assert!(outcome.run.receipts.is_empty());
        }

        // 6. Authorized cancel_run persists cancel_intent_persisted & authority_revoked before stopping Dagu.
        seed_compiled_deployment(&mut store, "wf_cancel", 1, false);
        bind_and_start_run(
            &mut store,
            &actor,
            &adapter,
            &StartRunParams {
                command_key: "start-cancel".to_owned(),
                project_id: "P".to_owned(),
                run_id: "run_cancel".to_owned(),
                binding_id: "bind_cancel".to_owned(),
                workflow_id: "wf_cancel".to_owned(),
                revision: 1,
                port_provider_binding: "dagu:local".to_owned(),
                enqueue: false,
                now_ms: 1_000,
                progress_timeout_ms: 60_000,
            },
        )
        .await
        .expect("start run for cancel");
        let (cancelled_run, closed_binding, stop_outcome) = cancel_run(
            &mut store,
            &actor,
            &adapter,
            "cancel-cmd-1",
            "P",
            "bind_cancel",
            2_000,
        )
        .await
        .expect("authorized cancel_run");
        assert!(cancelled_run.cancel_intent_persisted);
        assert!(cancelled_run.authority_revoked);
        assert_eq!(cancelled_run.state, RunState::Cancelled);
        assert_eq!(closed_binding.state, BindingState::Closed);
        assert_eq!(stop_outcome, StopOutcome::ParkedWaitingNoProcess);
    }

    struct CliHarness {
        root: TempDir,
        dagu_url: String,
        dagu_bin: PathBuf,
    }

    impl CliHarness {
        fn new(dagu_url: String) -> Self {
            let root = TempDir::new("cli-root");
            let cache_dir =
                std::env::var("HCTL2_TEST_DAGU_CACHE").expect("HCTL2_TEST_DAGU_CACHE env");
            let dagu_bin =
                find_dagu_binary(Path::new(&cache_dir)).expect("locate packaged dagu binary");
            let harness = Self {
                root,
                dagu_url,
                dagu_bin,
            };
            let (ok, val) = harness.run_cli(&["init", "--secret-backend", "user-file"]);
            assert!(ok, "hctl2 init failed: {val}");
            let mut store = Store::open(&harness.root.0).expect("open store for cli seed");
            seed_compiled_deployment(&mut store, "wf_cli", 1, true);
            seed_compiled_deployment(&mut store, "wf_cli_div", 1, false);
            drop(store);
            let (ok, val) = harness.run_cli(&["start"]);
            assert!(ok, "hctl2 start failed: {val}");
            harness
        }

        fn run_cli(&self, args: &[&str]) -> (bool, serde_json::Value) {
            let hctl2 = std::env::var("CARGO_BIN_EXE_hctl2").expect("CARGO_BIN_EXE_hctl2");
            let control =
                std::env::var("CARGO_BIN_EXE_hctl2-control").expect("CARGO_BIN_EXE_hctl2-control");
            let output = std::process::Command::new(hctl2)
                .env("HCTL2_CONTROL_BIN", control)
                .env("HCTL2_DAGU_BIN", &self.dagu_bin)
                .env("HCTL2_DAGU_URL", &self.dagu_url)
                .args(["--root", self.root.0.to_str().unwrap(), "--json"])
                .args(args)
                .output()
                .expect("execute hctl2 CLI");
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parsed = serde_json::from_str(&stdout).unwrap_or_else(|_| {
                json!({
                    "stdout": stdout,
                    "stderr": String::from_utf8_lossy(&output.stderr),
                })
            });
            (output.status.success(), parsed)
        }

        fn write_engine_cmd(
            &self,
            subcmd: &str,
            key: &str,
            action: &serde_json::Value,
        ) -> (bool, serde_json::Value) {
            let input_path = self.root.0.join(format!("{key}.json"));
            std::fs::write(&input_path, serde_json::to_vec(action).unwrap()).unwrap();
            let (ok, preview) = self.run_cli(&[
                "workflow",
                "engine",
                subcmd,
                "--input",
                input_path.to_str().unwrap(),
                "--key",
                key,
            ]);
            if !ok {
                return (false, preview);
            }
            let token = preview["preview_token"]
                .as_str()
                .expect("preview_token string");
            self.run_cli(&[
                "workflow",
                "engine",
                subcmd,
                "--input",
                input_path.to_str().unwrap(),
                "--key",
                key,
                "--preview-token",
                token,
            ])
        }
    }

    impl Drop for CliHarness {
        fn drop(&mut self) {
            let _ = self.run_cli(&["stop"]);
        }
    }

    #[tokio::test]
    async fn cli_to_handler_hosted_dagu_consumption_secret_store_and_workflow_engine_commands() {
        if std::env::var_os("CARGO_BIN_EXE_hctl2").is_none() {
            return;
        }
        let live = LiveDagu::spawn().await;
        let cli = CliHarness::new(live.base_url.clone());

        // 1. Start a Run via `hctl2 workflow engine start` (consumes hosted "dagu" and provisions SecretStore API key).
        let (ok, started) = cli.write_engine_cmd(
            "start",
            "cli-start-1",
            &json!({
                "project_id": "P",
                "run_id": "run_cli_1",
                "binding_id": "bind_cli_1",
                "workflow_id": "wf_cli",
                "revision": 1,
                "port_provider_binding": "dagu:local",
                "enqueue": false,
                "now_ms": 1000,
                "progress_timeout_ms": 60000
            }),
        );
        assert!(ok, "cli workflow engine start failed: {started}");
        assert_eq!(started["run"]["state"], "running");
        assert_eq!(started["binding"]["state"], "bound");

        // Verify hosted "dagu" was recorded as consumed and SecretStore holds the `dagu_` API key (验收 5).
        let consumed_bytes =
            std::fs::read(cli.root.0.join("hosted-consumed.json")).expect("hosted-consumed.json");
        let consumed_json: serde_json::Value = serde_json::from_slice(&consumed_bytes).unwrap();
        assert!(
            consumed_json["consumed"]
                .as_array()
                .unwrap()
                .contains(&json!("dagu")),
            "first workflow engine command must consume hosted dagu: {consumed_json}"
        );
        assert!(cli.root.0.join("dagu-auth-provisioned").is_file());

        // 2. Reject engine self-advance via CLI (验收 1).
        let (ok, self_adv) = cli.write_engine_cmd(
            "self-advance",
            "cli-self-adv-1",
            &json!({
                "project_id": "P",
                "run_id": "run_cli_1",
                "step_id": "cp_1"
            }),
        );
        assert!(!ok, "self-advance must be rejected: {self_adv}");
        assert_eq!(self_adv["error"]["code"], "RUN_ADVANCE_REQUIRES_COMMAND");

        // 3. Poll via CLI (normal + cached/late/stale rejections, 验收 3).
        let (ok, polled) = cli.write_engine_cmd(
            "poll",
            "cli-poll-1",
            &json!({
                "project_id": "P",
                "binding_id": "bind_cli_1",
                "now_ms": 1100,
                "from_cache": false
            }),
        );
        assert!(ok, "cli poll failed: {polled}");
        assert_eq!(
            polled["observations"][0]["admissible_for_obligation"],
            json!(true)
        );

        let (ok, cached_poll) = cli.write_engine_cmd(
            "poll",
            "cli-poll-cache",
            &json!({
                "project_id": "P",
                "binding_id": "bind_cli_1",
                "now_ms": 1200,
                "from_cache": true
            }),
        );
        assert!(ok, "cli cached poll failed: {cached_poll}");
        assert_eq!(
            cached_poll["observations"][0]["admissible_for_obligation"],
            json!(false)
        );

        // 4. Complete checkpoint with `ack_unknown: true` via CLI (验收 4).
        let (ok, completed) = cli.write_engine_cmd(
            "complete",
            "cli-complete-1",
            &json!({
                "project_id": "P",
                "binding_id": "bind_cli_1",
                "step_id": "cp_1",
                "obligation_id": "obl_cli_1",
                "attempt_generation": 1,
                "verdict": "accepted",
                "receipt_id": "rcpt_cli_1",
                "ack_unknown": true,
                "now_ms": 1300
            }),
        );
        assert!(ok, "cli complete failed: {completed}");
        assert_eq!(completed["resolution"]["readback_performed"], json!(true));
        assert_eq!(completed["run"]["completed_checkpoints"], json!(["cp_1"]));

        // 5. Query `hctl2 workflow engine show P bind_cli_1`.
        let (ok, shown) = cli.run_cli(&["workflow", "engine", "show", "P", "bind_cli_1"]);
        assert!(ok, "cli workflow engine show failed: {shown}");
        assert_eq!(shown["run"]["run_id"], "run_cli_1");
        assert_eq!(shown["binding"]["binding_id"], "bind_cli_1");

        // 6. Direct Dagu operation via CLI + poll marks binding Diverged without back-propagating Run (验收 2).
        let (ok, _) = cli.write_engine_cmd(
            "start",
            "cli-start-div",
            &json!({
                "project_id": "P",
                "run_id": "run_cli_div",
                "binding_id": "bind_cli_div",
                "workflow_id": "wf_cli_div",
                "revision": 1,
                "port_provider_binding": "dagu:local",
                "enqueue": false,
                "now_ms": 2000,
                "progress_timeout_ms": 60000
            }),
        );
        assert!(ok);
        let (ok, direct_res) = cli.write_engine_cmd(
            "direct-op",
            "cli-direct-edit",
            &json!({
                "project_id": "P",
                "binding_id": "bind_cli_div",
                "operation": "edit"
            }),
        );
        assert!(ok, "cli direct-op failed: {direct_res}");
        let (ok, div_poll) = cli.write_engine_cmd(
            "poll",
            "cli-poll-div",
            &json!({
                "project_id": "P",
                "binding_id": "bind_cli_div",
                "now_ms": 2100,
                "from_cache": false
            }),
        );
        assert!(ok, "cli div poll failed: {div_poll}");
        assert_eq!(div_poll["binding"]["state"], "diverged");
        assert_eq!(div_poll["binding"]["divergence"]["kind"], "edit");
        assert_eq!(div_poll["run"]["needs_attention"], json!(true));
        assert_eq!(div_poll["run"]["state"], "running");
        assert_eq!(div_poll["run"]["completed_checkpoints"], json!([]));

        // 7. Backup & stop hosted services via CLI, and verify missing SecretStore key fails closed (验收 5).
        let backup_dir = cli.root.0.join("services-backup");
        let (ok, backup_res) = cli.run_cli(&["services", "backup", backup_dir.to_str().unwrap()]);
        assert!(ok, "cli services backup failed: {backup_res}");
        assert!(backup_dir.join("hosted-consumed.json").is_file());

        // Remove the SecretStore directory while keeping `dagu-auth-provisioned` to simulate credential loss.
        let _ = std::fs::remove_dir_all(cli.root.0.join("secrets"));
        let (ok, missing_secret_err) = cli.write_engine_cmd(
            "poll",
            "cli-poll-no-secret",
            &json!({
                "project_id": "P",
                "binding_id": "bind_cli_1",
                "now_ms": 3000,
                "from_cache": false
            }),
        );
        assert!(
            !ok,
            "missing Dagu API key in SecretStore must fail closed, not degrade to unauthenticated: {missing_secret_err}"
        );
        assert_eq!(
            missing_secret_err["error"]["code"],
            "WORKFLOW_ENGINE_SECRET_MISSING"
        );
    }
}

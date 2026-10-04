use agency::{
    Agency,
    runtime::{Runtime, ScriptConfig, ScriptRuntime},
};
use agency_proto::{
    client::Client,
    context::{Bundle, Delivery, Entry},
    *,
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Rig {
    root: PathBuf,
    admin: Client,
    pairing: Client,
    handle: tokio::task::JoinHandle<Result<()>>,
}
impl Rig {
    async fn new(script: &str) -> Self {
        Self::with_runtime(Arc::new(ScriptRuntime::new(ScriptConfig {
            program: "/bin/sh".into(),
            arguments: vec!["-c".into(), script.into()],
        })))
        .await
    }
    async fn with_runtime(runtime: Arc<dyn Runtime>) -> Self {
        let root = PathBuf::from(format!(
            "/tmp/agency-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let service = Agency::open(&root, runtime).unwrap();
        let admin = Client::new(
            agency_proto::client::admin_endpoint(&root).unwrap(),
            Agency::bootstrap_key(&root).unwrap(),
        );
        let pairing = Client::new(
            agency_proto::client::admin_endpoint(&root).unwrap(),
            std::fs::read_to_string(root.join("pair.key")).unwrap(),
        );
        let handle = tokio::spawn(agency::serve(service));
        for _ in 0..100 {
            if admin.call::<_, Value>("catalog", &json!({})).await.is_ok() {
                return Self {
                    root,
                    admin,
                    pairing,
                    handle,
                };
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        panic!("Agency not ready");
    }
    async fn pair(&self, id: &str) -> (Client, String) {
        let pair: Pairing = self
            .pairing
            .call(
                "pair",
                &Pair {
                    control_id: id.into(),
                    tenant_key: agency_proto::client::new_credential().unwrap(),
                },
            )
            .await
            .unwrap();
        let key = pair.key.clone();
        let client = Client::new(pair.endpoint.into(), pair.key);
        let _: Value = client
            .call(
                "fence",
                &Fence {
                    writer_generation: 1,
                },
            )
            .await
            .unwrap();
        (client, key)
    }
    async fn close(self) {
        let _: Value = self.admin.call("shutdown", &json!({})).await.unwrap();
        self.handle.await.unwrap().unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        assert!(
            !agency_proto::client::socket_directory(&self.root)
                .unwrap()
                .exists()
        );
        std::fs::remove_dir_all(&self.root).unwrap();
    }
    async fn restart(mut self, script: &str) -> Self {
        let _: Value = self.admin.call("shutdown", &json!({})).await.unwrap();
        self.handle.await.unwrap().unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let service = Agency::open(
            &self.root,
            Arc::new(ScriptRuntime::new(ScriptConfig {
                program: "/bin/sh".into(),
                arguments: vec!["-c".into(), script.into()],
            })),
        )
        .unwrap();
        self.handle = tokio::spawn(agency::serve(service));
        for _ in 0..100 {
            if self
                .admin
                .call::<_, Value>("catalog", &json!({}))
                .await
                .is_ok()
            {
                return self;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        panic!("Agency restart not ready");
    }
}
fn reference(id: &str) -> FrozenRef {
    FrozenRef {
        id: id.into(),
        revision: "1".into(),
        digest: hash(id.as_bytes()),
    }
}
async fn request(client: &Client, key: &str) -> Prepare {
    let catalog: Catalog = client.call("catalog", &json!({})).await.unwrap();
    let owner = Owner {
        project: "project".into(),
        kind: OwnerKind::RoomInvocation,
        id: "invocation".into(),
        generation: 1,
    };
    let manifest = reference("manifest");
    let bundle = Sealed::new(Bundle {
        id: "bundle".into(),
        manifest: manifest.clone(),
        consumer: owner.clone(),
        entries: vec![Entry {
            source: reference("task"),
            description: "task book".into(),
            required: true,
            offline_required: true,
            bytes_digest: hash(b"do the work"),
            delivery: Delivery::Inline {
                bytes: b"do the work".to_vec(),
            },
        }],
        renderer: reference("renderer"),
        tokenizer: reference("tokenizer"),
        redaction: reference("redaction"),
        compression: vec![],
        candidate_tokens: Some(3),
        selected_tokens: Some(3),
        delivered_tokens: Some(3),
        permission_digest: hash(b"permission"),
        budget: 100,
        retention: "owner_terminal_and_admission_closed".into(),
    })
    .unwrap();
    let spec = Sealed::new(ExecutionSpec {
        owner,
        project: reference("project"),
        selection: reference("selection"),
        selection_policy_digest: hash(b"selection"),
        profession: catalog.professions[0].clone(),
        profile: reference("profile"),
        manifest,
        bundle: FrozenRef {
            id: "bundle".into(),
            revision: "1".into(),
            digest: bundle.digest.clone(),
        },
        binding: reference("binding"),
        required_capabilities: Capabilities {
            input: true,
            stop: true,
            event_cursor: true,
            ..Capabilities::default()
        },
        input_policy: InputPolicy::ManagedSingleWriter,
        permission_digest: hash(b"permission"),
        permissions: vec!["read".into()],
        budget: 100,
        deadline_ms: now() + 60_000,
        repo: None,
        base: None,
        delivery_scope: vec![],
        write_lease: None,
        review_publish_policy: None,
        idempotency_key: key.into(),
    })
    .unwrap();
    Prepare {
        spec,
        bundle,
        writer_generation: 1,
    }
}
fn now() -> u64 {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
    .unwrap()
}
fn ticket(d: &Dispatch, key: &str, permissions: Vec<Permission>, lease: Option<&str>) -> Ticket {
    Ticket::sign(
        TicketClaims {
            id: "ticket".into(),
            actor: "human".into(),
            dispatch: d.reference.clone(),
            owner: d.owner.clone(),
            spec_digest: d.spec_digest.clone(),
            writer_generation: 1,
            permissions,
            input_lease: lease.map(str::to_owned),
            expires_ms: now() + 30_000,
        },
        key.as_bytes(),
    )
    .unwrap()
}
async fn activate(client: &Client, d: &Dispatch) -> Dispatch {
    client
        .call(
            "activate",
            &DispatchAction {
                dispatch: d.reference.clone(),
                writer_generation: 1,
                idempotency_key: "activate".into(),
            },
        )
        .await
        .unwrap()
}
async fn terminal(client: &Client, d: &Dispatch, key: &str) -> Trace {
    for _ in 0..100 {
        let trace: Trace = client
            .call(
                "observe",
                &Observe {
                    ticket: ticket(d, key, vec![Permission::Observe], None),
                    after: 0,
                },
            )
            .await
            .unwrap();
        if trace.dispatch.state != DispatchState::Running {
            return trace;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("script did not stop");
}
const RESULT: &str = "read -r init; printf '%s\n' '{\"type\":\"result\",\"schema\":\"test.result.v1\",\"output\":\"answer\"}'";

#[tokio::test]
async fn prepare_and_prepared_activation_reject_expired_deadline() {
    let rig = Rig::new(RESULT).await;
    let (client, _) = rig.pair("expiry").await;
    let mut expired = request(&client, "expired-prepare").await;
    expired.spec.document.deadline_ms = now() - 1;
    expired.spec = Sealed::new(expired.spec.document).unwrap();
    assert_eq!(
        client
            .call::<_, Dispatch>("prepare", &expired)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXPIRED"
    );
    let mut req = request(&client, "expired-activation").await;
    req.spec.document.deadline_ms = now() + 5_000;
    req.spec = Sealed::new(req.spec.document).unwrap();
    let d: Dispatch = client.call("prepare", &req).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(
        req.spec.document.deadline_ms.saturating_sub(now()) + 1,
    ))
    .await;
    assert_eq!(
        client
            .call::<_, Dispatch>(
                "activate",
                &DispatchAction {
                    dispatch: d.reference.clone(),
                    writer_generation: 1,
                    idempotency_key: "activate".into()
                }
            )
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXPIRED"
    );
    let results: ResultPage = client
        .call("results", &ResultQuery::of(d.reference))
        .await
        .unwrap();
    assert!(results.proposals.is_empty());
    assert!(results.complete);
    rig.close().await;
}

#[tokio::test]
async fn concurrent_takeover_has_one_winner_and_revoked_lease_cannot_return() {
    let rig = Rig::new("read -r init; exec sleep 30").await;
    let (client, key) = rig.pair("cas").await;
    let d: Dispatch = client
        .call("prepare", &request(&client, "cas").await)
        .await
        .unwrap();
    activate(&client, &d).await;
    let lease = |id: &str, expected: Option<String>| Lease {
        ticket: ticket(&d, &key, vec![Permission::Takeover], Some(id)),
        expected_lease: expected,
        new_lease: id.into(),
    };
    let a = lease("a", None);
    let b = lease("b", None);
    let (a_result, b_result) = tokio::join!(
        client.call::<_, Value>("lease", &a),
        client.call::<_, Value>("lease", &b)
    );
    let winner = match (a_result, b_result) {
        (Ok(_), Err(e)) => {
            assert_eq!(e.code, "LEASE_CONFLICT");
            "a"
        }
        (Err(e), Ok(_)) => {
            assert_eq!(e.code, "LEASE_CONFLICT");
            "b"
        }
        result => panic!("expected one CAS winner: {result:?}"),
    };
    let _: Value = client
        .call("lease", &lease("replacement", Some(winner.into())))
        .await
        .unwrap();
    assert_eq!(
        client
            .call::<_, Value>("lease", &lease(winner, Some("replacement".into())))
            .await
            .unwrap_err()
            .code,
        "LEASE_REUSED"
    );
    rig.close().await;
}

#[tokio::test]
async fn original_pairing_proof_replays_after_service_restart() {
    let rig = Rig::new(RESULT).await;
    let input = Pair {
        control_id: "stable".into(),
        tenant_key: agency_proto::client::new_credential().unwrap(),
    };
    let original: Pairing = rig.pairing.call("pair", &input).await.unwrap();
    let rig = rig.restart(RESULT).await;
    let again: Pairing = rig.pairing.call("pair", &input).await.unwrap();
    assert_eq!(original.endpoint, again.endpoint);
    assert!(credential_matches(&original.key, &again.key));
    rig.close().await;
}

#[tokio::test]
async fn unsafe_socket_directory_and_socket_mode_are_rejected_before_credentials() {
    use std::os::unix::fs::PermissionsExt;
    let rig = Rig::new(RESULT).await;
    let socket = agency_proto::client::admin_endpoint(&rig.root).unwrap();
    let directory = socket.parent().unwrap();
    std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        rig.admin
            .call::<_, Value>("catalog", &json!({}))
            .await
            .unwrap_err()
            .code,
        "UNSAFE_ENDPOINT"
    );
    std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o666)).unwrap();
    assert_eq!(
        rig.admin
            .call::<_, Value>("catalog", &json!({}))
            .await
            .unwrap_err()
            .code,
        "UNSAFE_ENDPOINT"
    );
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    rig.close().await;
}

struct BlockingRuntime {
    entered: std::sync::mpsc::Sender<()>,
    release: Arc<std::sync::Mutex<std::sync::mpsc::Receiver<()>>>,
    writes: Arc<AtomicU64>,
}

struct BurstRuntime;
struct BurstSession(std::sync::mpsc::Sender<agency::runtime::RuntimeEvent>);
impl agency::runtime::Session for BurstSession {
    fn input(&mut self, _: &[u8]) -> Result<()> {
        Ok(())
    }
    fn stop(&mut self) -> Result<()> {
        let _ = self.0.send(agency::runtime::RuntimeEvent::Exited {
            code: None,
            requested_stop: true,
        });
        Ok(())
    }
}
impl Runtime for BurstRuntime {
    fn catalog(&self) -> Result<Catalog> {
        ScriptRuntime::new(ScriptConfig {
            program: "/bin/sh".into(),
            arguments: vec![],
        })
        .catalog()
    }
    fn start(
        &self,
        _: &Sealed<ExecutionSpec>,
        _: &Sealed<Bundle>,
        _: &std::path::Path,
        _: &std::path::Path,
    ) -> Result<agency::runtime::Running> {
        let (events, receiver) = std::sync::mpsc::channel();
        for _ in 0..2 {
            events
                .send(agency::runtime::RuntimeEvent::Proposal {
                    schema: "test.bytes.v1".into(),
                    bytes: vec![255; 2 * 1024 * 1024],
                    source: EvidenceLevel::Narrated,
                })
                .unwrap();
        }
        Ok(agency::runtime::Running {
            session: Arc::new(std::sync::Mutex::new(Box::new(BurstSession(events)))),
            events: receiver,
        })
    }
}

#[tokio::test]
async fn result_pages_keep_each_accepted_payload_under_the_transport_limit() {
    let rig = Rig::with_runtime(Arc::new(BurstRuntime)).await;
    let (client, _key) = rig.pair("budget").await;
    let d: Dispatch = client
        .call("prepare", &request(&client, "budget").await)
        .await
        .unwrap();
    activate(&client, &d).await;
    let mut pages = Vec::new();
    let mut after = None;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while pages.len() < 2 && std::time::Instant::now() < deadline {
        let mut query = ResultQuery::of(d.reference.clone());
        query.after = after.clone();
        let page: ResultPage = client.call("results", &query).await.unwrap();
        if page.proposals.is_empty()
            || (pages.is_empty() && page.complete && page.proposals.len() < 2)
        {
            // The second proposal may still be committing. A complete page of one
            // is not yet the durable result set.
            after = None;
            pages.clear();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            continue;
        }
        assert_eq!(page.proposals.len(), 1);
        assert!(canonical(&page).unwrap().len() < MAX_DOCUMENT);
        assert_eq!(page.proposals[0].output, vec![255; 2 * 1024 * 1024]);
        after = page.cursor.clone();
        let complete = page.complete;
        pages.push(page);
        if complete {
            break;
        }
    }
    assert_eq!(pages.len(), 2);
    assert!(!pages[0].complete);
    assert!(pages[1].complete);
    rig.close().await;
}
struct BlockingSession {
    entered: std::sync::mpsc::Sender<()>,
    release: Arc<std::sync::Mutex<std::sync::mpsc::Receiver<()>>>,
    writes: Arc<AtomicU64>,
    events: std::sync::mpsc::Sender<agency::runtime::RuntimeEvent>,
}
impl agency::runtime::Session for BlockingSession {
    fn input(&mut self, _: &[u8]) -> Result<()> {
        self.writes.fetch_add(1, Ordering::Relaxed);
        self.entered.send(()).unwrap();
        self.release
            .lock()
            .unwrap()
            .recv_timeout(std::time::Duration::from_secs(3))
            .unwrap();
        Err(PortError::new(
            "TEST_REPLY_LOST",
            "fixture lost input reply",
            "readback",
        ))
    }
    fn stop(&mut self) -> Result<()> {
        let _ = self.events.send(agency::runtime::RuntimeEvent::Exited {
            code: None,
            requested_stop: true,
        });
        Ok(())
    }
}
impl Runtime for BlockingRuntime {
    fn catalog(&self) -> Result<Catalog> {
        ScriptRuntime::new(ScriptConfig {
            program: "/bin/sh".into(),
            arguments: vec![],
        })
        .catalog()
    }
    fn start(
        &self,
        _: &Sealed<ExecutionSpec>,
        _: &Sealed<Bundle>,
        _: &std::path::Path,
        _: &std::path::Path,
    ) -> Result<agency::runtime::Running> {
        let (events, receiver) = std::sync::mpsc::channel();
        Ok(agency::runtime::Running {
            session: Arc::new(std::sync::Mutex::new(Box::new(BlockingSession {
                entered: self.entered.clone(),
                release: Arc::clone(&self.release),
                writes: Arc::clone(&self.writes),
                events,
            }))),
            events: receiver,
        })
    }
}

#[tokio::test]
async fn blocked_tenant_input_does_not_block_other_prepare_and_unknown_is_not_resent() {
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let writes = Arc::new(AtomicU64::new(0));
    let rig = Rig::with_runtime(Arc::new(BlockingRuntime {
        entered: entered_tx,
        release: Arc::new(std::sync::Mutex::new(release_rx)),
        writes: Arc::clone(&writes),
    }))
    .await;
    let (client, key) = rig.pair("blocked").await;
    let (other, _) = rig.pair("independent").await;
    let d: Dispatch = client
        .call("prepare", &request(&client, "blocked").await)
        .await
        .unwrap();
    activate(&client, &d).await;
    let _: Value = client
        .call(
            "lease",
            &Lease {
                ticket: ticket(&d, &key, vec![Permission::Takeover], Some("input")),
                expected_lease: None,
                new_lease: "input".into(),
            },
        )
        .await
        .unwrap();
    let input = Input {
        ticket: ticket(&d, &key, vec![Permission::Input], Some("input")),
        idempotency_key: "once".into(),
        bytes: b"hello".to_vec(),
    };
    let task = tokio::spawn({
        let client = client.clone();
        let input = input.clone();
        async move { client.call::<_, Value>("input", &input).await }
    });
    tokio::task::spawn_blocking(move || {
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap()
    })
    .await
    .unwrap();
    let other_request = request(&other, "independent").await;
    let other_reply = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        other.call::<_, Dispatch>("prepare", &other_request),
    )
    .await;
    // Release even on assertion failure so the fixture cannot strand the test.
    release_tx.send(()).unwrap();
    assert_eq!(other_reply.unwrap().unwrap().state, DispatchState::Prepared);
    assert_eq!(
        task.await.unwrap().unwrap_err().code,
        "INPUT_RESPONSE_UNKNOWN"
    );
    let replay: Value = client.call("input", &input).await.unwrap();
    assert_eq!(replay["delivery"], "unknown");
    let mut changed = input;
    changed.ticket.claims.id = "new-ticket".into();
    changed.ticket = Ticket::sign(changed.ticket.claims, key.as_bytes()).unwrap();
    assert_eq!(
        client
            .call::<_, Value>("input", &changed)
            .await
            .unwrap_err()
            .code,
        "IDEMPOTENCY_CONFLICT"
    );
    assert_eq!(writes.load(Ordering::Relaxed), 1);
    rig.close().await;
}

#[cfg(feature = "control_port_test")]
#[tokio::test]
async fn control_persists_mapping_before_activation_and_exact_bytes_before_ack() {
    use store::{Actor, ActorSource, Scope, Store, TrustedActor};
    let rig = Rig::new(RESULT).await;
    let root = rig.root.join("control");
    let actor = TrustedActor(Actor {
        principal: "human".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control, Scope::Project("project".into())],
        authority: None,
    });
    let shared = Arc::new(tokio::sync::Mutex::new(Some(Store::open(&root).unwrap())));
    let pair = control::agency::submit(
        &shared,
        &root,
        "agency.pair",
        &json!({"binding_id":"binding","agency_root":rig.root}),
        &actor,
        "pair",
    )
    .await
    .unwrap();
    assert_eq!(pair["paired"], true);
    let different = Rig::new(RESULT).await;
    assert_eq!(
        control::agency::submit(
            &shared,
            &root,
            "agency.pair",
            &json!({"binding_id":"binding","agency_root":different.root}),
            &actor,
            "changed-provider"
        )
        .await
        .unwrap_err()
        .code,
        "BINDING_IMMUTABLE"
    );
    different.close().await;
    assert_eq!(
        control::agency::submit(
            &shared,
            &root,
            "agency.pair",
            &json!({"binding_id":"binding","agency_root":rig.root}),
            &actor,
            "pair",
        )
        .await
        .unwrap(),
        pair
    );
    assert_eq!(
        control::agency::submit(
            &shared,
            &root,
            "agency.pair",
            &json!({"binding_id":"different","agency_root":rig.root}),
            &actor,
            "pair",
        )
        .await
        .unwrap_err()
        .code,
        "IDEMPOTENCY_CONFLICT"
    );
    let client = control::agency::paired_client(&root, "binding").unwrap();
    let mut prepare = request(&client, "control-dispatch").await;
    control::agency::submit(
        &shared,
        &root,
        "profession.accept",
        &json!({"binding_id":"binding","profession":prepare.spec.document.profession.reference}),
        &actor,
        "accept",
    )
    .await
    .unwrap();
    let (intent, original_owner) = {
        let mut lock = shared.lock().await;
        let s = lock.as_mut().unwrap();
        let b = s
            .get(&participant::key(
                Scope::Control,
                "agency_binding",
                "binding",
            ))
            .unwrap()
            .unwrap();
        prepare.spec.document.binding = participant::frozen(&b);
        prepare.spec = Sealed::new(prepare.spec.document).unwrap();
        let owner = participant::value(
            participant::key(
                Scope::Project("project".into()),
                "authorized_invocation",
                "invocation",
            ),
            1,
            &json!({"state":"authorized"}),
        )
        .unwrap();
        assert!(
            participant::prepare_dispatch(
                s,
                &actor,
                "dispatch-intent",
                &participant::reference(&owner),
                &prepare.spec,
                &prepare.bundle
            )
            .is_err()
        );
        let command = store::Command {
            command_id: "seed-owner".into(),
            idempotency_key: "seed-owner".into(),
            actor: actor.0.clone(),
            target: owner.key.clone(),
            expected: store::Expected::Absent,
            binding: participant::reference(&owner),
            operation: "test.authorize".into(),
            input: json!({}),
            input_digest: store::Command::digest_input("test.authorize", &json!({})).unwrap(),
        };
        s.submit(s.generation(), &actor, &command, None, |tx| {
            tx.put(&owner)?;
            Ok(json!({}))
        })
        .unwrap();
        let intent = participant::prepare_dispatch(
            s,
            &actor,
            "dispatch-intent",
            &participant::reference(&owner),
            &prepare.spec,
            &prepare.bundle,
        )
        .unwrap();
        assert!(s.effect("activate:dispatch-intent").is_err());
        assert_eq!(
            s.effect("prepare:dispatch-intent")
                .unwrap()
                .0
                .idempotency_key,
            prepare.spec.document.idempotency_key
        );
        (intent, owner)
    };
    // Crash after Agency accepts prepare but before control receives the reply.
    {
        let mut lock = shared.lock().await;
        let s = lock.as_mut().unwrap();
        let generation = s.generation();
        s.begin_effect(generation, "prepare:dispatch-intent")
            .unwrap();
        assert_eq!(
            s.effect("prepare:dispatch-intent").unwrap().1,
            store::EffectState::Unknown
        );
    }
    let prepared: Dispatch = client.call("prepare", &prepare).await.unwrap();
    let dispatch = control::agency::deliver_prepare(&shared, &root, &actor, &intent)
        .await
        .unwrap();
    assert_eq!(dispatch.key.id, prepared.reference);
    // Confirmed prepare replay returns the same stored mapping.
    let again = control::agency::deliver_prepare(&shared, &root, &actor, &intent)
        .await
        .unwrap();
    assert_eq!(dispatch.key, again.key);
    {
        let lock = shared.lock().await;
        let s = lock.as_ref().unwrap();
        assert_eq!(
            s.effect("prepare:dispatch-intent").unwrap().1,
            store::EffectState::Confirmed
        );
        assert_eq!(
            s.effect("activate:dispatch-intent").unwrap().1,
            store::EffectState::Pending
        );
    }
    let running =
        control::agency::deliver_activation(&shared, &root, &actor, "dispatch-intent", &dispatch)
            .await
            .unwrap();
    for _ in 0..100 {
        let result: ResultPage = client
            .call("results", &ResultQuery::of(running.reference.clone()))
            .await
            .unwrap();
        if !result.proposals.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!(
        control::agency::preserve_results(&shared, &root, &actor, &dispatch)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        control::agency::preserve_results(&shared, &root, &actor, &dispatch)
            .await
            .unwrap(),
        1
    );
    let results: ResultPage = client
        .call("results", &ResultQuery::of(running.reference.clone()))
        .await
        .unwrap();
    assert!(results.proposals[0].preserved);
    assert_eq!(
        control::agency::deliver_activation(&shared, &root, &actor, "dispatch-intent", &dispatch)
            .await
            .unwrap(),
        running
    );
    {
        let mut lock = shared.lock().await;
        let s = lock.as_mut().unwrap();
        assert_eq!(
            s.get(&original_owner.key).unwrap().unwrap().revision_digest,
            original_owner.revision_digest
        );
        let mut wrong = results.proposals[0].clone();
        wrong.outputs[0].owner.project = "another-project".into();
        assert_eq!(
            participant::preserve_proposal(s, &actor, &dispatch, &wrong)
                .unwrap_err()
                .code,
            "PROPOSAL_MISMATCH"
        );
        let mut direct = results.proposals[0].clone();
        direct.evidence = EvidenceLevel::Unmediated;
        assert_eq!(
            participant::preserve_proposal(s, &actor, &dispatch, &direct)
                .unwrap_err()
                .code,
            "EVIDENCE_SOURCE_INVALID"
        );
        let stored = s
            .get(&participant::key(
                Scope::Project("project".into()),
                "proposal_inbox",
                &results.proposals[0].header.proposal_id,
            ))
            .unwrap()
            .unwrap();
        assert_eq!(
            s.read_material(&actor, &stored.materials[0]).unwrap(),
            b"answer"
        );
    }
    let _: Value = rig.admin.call("shutdown", &json!({})).await.unwrap();
    rig.handle.await.unwrap().unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;
    assert_eq!(
        control::agency::reconcile(&shared, &root)
            .await
            .unwrap_err()
            .code,
        "AGENCY_UNREACHABLE"
    );
    assert!(
        rig.admin
            .call::<_, Value>("catalog", &json!({}))
            .await
            .is_err()
    );
    assert_eq!(
        control::agency::observe_dispatch(&shared, &root, &actor, &dispatch, 0)
            .await
            .unwrap_err()
            .code,
        "AGENCY_UNREACHABLE"
    );
    let lock = shared.lock().await;
    let s = lock.as_ref().unwrap();
    assert_eq!(s.list("dispatch_contact").unwrap().len(), 1);
    assert_eq!(
        s.get(&original_owner.key).unwrap().unwrap().revision_digest,
        original_owner.revision_digest
    );
    drop(lock);
    drop(shared);
    assert!(
        !agency_proto::client::socket_directory(&rig.root)
            .unwrap()
            .exists()
    );
    std::fs::remove_dir_all(&rig.root).unwrap();
}

#[tokio::test]
async fn deadline_stops_fixture_without_revoking_result_observation() {
    let rig = Rig::new("read -r init; exec sleep 30").await;
    let (client, key) = rig.pair("control").await;
    let mut request = request(&client, "deadline").await;
    request.spec.document.deadline_ms = now() + 5_000;
    request.spec = Sealed::new(request.spec.document).unwrap();
    let d: Dispatch = client.call("prepare", &request).await.unwrap();
    activate(&client, &d).await;
    tokio::time::sleep(std::time::Duration::from_millis(
        request.spec.document.deadline_ms.saturating_sub(now()) + 1,
    ))
    .await;
    let trace = terminal(&client, &d, &key).await;
    assert_eq!(trace.dispatch.state, DispatchState::CannotFulfill);
    assert!(
        trace
            .events
            .iter()
            .any(|e| e.payload["code"] == "DEADLINE_EXPIRED")
    );
    assert!(!trace.gap);
    rig.close().await;
}

#[cfg(feature = "control_port_test")]
async fn authorized_intent(
    shared: &Arc<tokio::sync::Mutex<Option<store::Store>>>,
    actor: &store::TrustedActor,
    client: &Client,
    id: &str,
    change: impl FnOnce(&mut ExecutionSpec),
) -> (store::Record, Prepare) {
    let mut req = request(client, id).await;
    req.spec.document.owner.id = id.into();
    req.bundle.document.consumer = req.spec.document.owner.clone();
    req.bundle = Sealed::new(req.bundle.document).unwrap();
    req.spec.document.bundle.digest = req.bundle.digest.clone();
    let mut lock = shared.lock().await;
    let store = lock.as_mut().unwrap();
    let binding = store
        .get(&participant::key(
            store::Scope::Control,
            "agency_binding",
            "binding",
        ))
        .unwrap()
        .unwrap();
    req.spec.document.binding = participant::frozen(&binding);
    change(&mut req.spec.document);
    req.spec = Sealed::new(req.spec.document).unwrap();
    let owner = participant::value(
        participant::key(
            store::Scope::Project("project".into()),
            "authorized_invocation",
            id,
        ),
        1,
        &json!({"state":"authorized"}),
    )
    .unwrap();
    let command = store::Command {
        command_id: format!("authorize:{id}"),
        idempotency_key: format!("authorize:{id}"),
        actor: actor.0.clone(),
        target: owner.key.clone(),
        expected: store::Expected::Absent,
        binding: participant::reference(&owner),
        operation: "test.authorize".into(),
        input: json!({}),
        input_digest: store::Command::digest_input("test.authorize", &json!({})).unwrap(),
    };
    store
        .submit(store.generation(), actor, &command, None, |tx| {
            tx.put(&owner)?;
            Ok(json!({}))
        })
        .unwrap();
    let intent = participant::prepare_dispatch(
        store,
        actor,
        id,
        &participant::reference(&owner),
        &req.spec,
        &req.bundle,
    )
    .unwrap();
    (intent, req)
}

#[cfg(feature = "control_port_test")]
#[tokio::test]
async fn control_fences_before_delivery_and_keeps_offline_intents_pending() {
    use store::{Actor, ActorSource, EffectState, Scope, Store, TrustedActor};
    let rig = Rig::new(RESULT).await;
    let root = rig.root.join("control");
    let actor = TrustedActor(Actor {
        principal: "human".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control, Scope::Project("project".into())],
        authority: None,
    });
    let shared = Arc::new(tokio::sync::Mutex::new(Some(Store::open(&root).unwrap())));
    control::agency::submit(
        &shared,
        &root,
        "agency.pair",
        &json!({"binding_id":"binding","agency_root":rig.root}),
        &actor,
        "pair",
    )
    .await
    .unwrap();
    let client = control::agency::paired_client(&root, "binding").unwrap();
    let candidate = request(&client, "candidate").await;
    control::agency::submit(
        &shared,
        &root,
        "profession.accept",
        &json!({"binding_id":"binding","profession":candidate.spec.document.profession.reference}),
        &actor,
        "accept",
    )
    .await
    .unwrap();
    let (intent, mut req) =
        authorized_intent(&shared, &actor, &client, "writer-restart", |_| {}).await;
    {
        let mut lock = shared.lock().await;
        drop(lock.take());
        *lock = Some(Store::open(&root).unwrap());
        req.writer_generation = u64::try_from(lock.as_ref().unwrap().generation().0).unwrap();
    }
    // A current control writer is still rejected until this tenant learns its new generation.
    assert_eq!(
        client
            .call::<_, Dispatch>("prepare", &req)
            .await
            .unwrap_err()
            .code,
        "WRITER_STALE"
    );
    let dispatch = control::agency::deliver_prepare(&shared, &root, &actor, &intent)
        .await
        .unwrap();
    assert_eq!(
        shared
            .lock()
            .await
            .as_ref()
            .unwrap()
            .effect("prepare:writer-restart")
            .unwrap()
            .1,
        EffectState::Confirmed
    );
    {
        let mut lock = shared.lock().await;
        drop(lock.take());
        *lock = Some(Store::open(&root).unwrap());
    }
    let running =
        control::agency::deliver_activation(&shared, &root, &actor, "writer-restart", &dispatch)
            .await
            .unwrap();
    assert_ne!(running.state, DispatchState::Prepared);
    assert_eq!(
        shared
            .lock()
            .await
            .as_ref()
            .unwrap()
            .effect("activate:writer-restart")
            .unwrap()
            .1,
        EffectState::Confirmed
    );
    let (offline_activation, _) =
        authorized_intent(&shared, &actor, &client, "offline-activation", |_| {}).await;
    let prepared = control::agency::deliver_prepare(&shared, &root, &actor, &offline_activation)
        .await
        .unwrap();
    let (offline_prepare, _) =
        authorized_intent(&shared, &actor, &client, "offline-prepare", |_| {}).await;
    let _: Value = rig.admin.call("shutdown", &json!({})).await.unwrap();
    rig.handle.await.unwrap().unwrap();
    assert_eq!(
        control::agency::deliver_prepare(&shared, &root, &actor, &offline_prepare)
            .await
            .unwrap_err()
            .code,
        "AGENCY_UNREACHABLE"
    );
    assert_eq!(
        control::agency::deliver_activation(
            &shared,
            &root,
            &actor,
            "offline-activation",
            &prepared,
        )
        .await
        .unwrap_err()
        .code,
        "AGENCY_UNREACHABLE"
    );
    let lock = shared.lock().await;
    let store = lock.as_ref().unwrap();
    for effect in ["prepare:offline-prepare", "activate:offline-activation"] {
        assert_eq!(store.effect(effect).unwrap().1, EffectState::Pending);
    }
    drop(lock);
    drop(shared);
    std::fs::remove_dir_all(&rig.root).unwrap();
}

#[cfg(feature = "control_port_test")]
#[tokio::test]
async fn control_distinguishes_definitive_refusal_from_unknown_readback() {
    use store::{Actor, ActorSource, Scope, Store, TrustedActor};
    let rig = Rig::new(RESULT).await;
    let root = rig.root.join("control");
    let actor = TrustedActor(Actor {
        principal: "human".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control, Scope::Project("project".into())],
        authority: None,
    });
    let shared = Arc::new(tokio::sync::Mutex::new(Some(Store::open(&root).unwrap())));
    control::agency::submit(
        &shared,
        &root,
        "agency.pair",
        &json!({"binding_id":"binding","agency_root":rig.root}),
        &actor,
        "pair",
    )
    .await
    .unwrap();
    let client = control::agency::paired_client(&root, "binding").unwrap();
    let candidate = request(&client, "candidate").await;
    control::agency::submit(
        &shared,
        &root,
        "profession.accept",
        &json!({"binding_id":"binding","profession":candidate.spec.document.profession.reference}),
        &actor,
        "accept",
    )
    .await
    .unwrap();
    let (refused, _) = authorized_intent(&shared, &actor, &client, "refused", |s| {
        s.required_capabilities.secure_input = true
    })
    .await;
    for _ in 0..2 {
        assert_eq!(
            control::agency::deliver_prepare(&shared, &root, &actor, &refused)
                .await
                .unwrap_err()
                .code,
            "CAPABILITY_MISSING"
        );
        let lock = shared.lock().await;
        let store = lock.as_ref().unwrap();
        assert_eq!(
            store.effect("prepare:refused").unwrap().1,
            store::EffectState::Rejected
        );
        assert!(store.effect("activate:refused").is_err());
    }
    // A decoded storage error is not proof of rejection; commit-stage failures can be ambiguous.
    let tenant_path = std::fs::read_dir(rig.root.join("tenants"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
        .join("tenant.sqlite");
    let fault = rusqlite::Connection::open(tenant_path).unwrap();
    fault.execute_batch("CREATE TRIGGER fail_prepare BEFORE INSERT ON dispatches BEGIN SELECT RAISE(FAIL,'injected storage failure'); END;").unwrap();
    let (storage_unknown, storage_request) =
        authorized_intent(&shared, &actor, &client, "storage-error", |_| {}).await;
    assert_eq!(
        control::agency::deliver_prepare(&shared, &root, &actor, &storage_unknown)
            .await
            .unwrap_err()
            .code,
        "AGENCY_ERROR"
    );
    assert_eq!(
        shared
            .lock()
            .await
            .as_ref()
            .unwrap()
            .effect("prepare:storage-error")
            .unwrap()
            .1,
        store::EffectState::Unknown
    );
    fault.execute_batch("DROP TRIGGER fail_prepare;").unwrap();
    drop(fault);
    let _: Dispatch = client.call("prepare", &storage_request).await.unwrap();
    control::agency::deliver_prepare(&shared, &root, &actor, &storage_unknown)
        .await
        .unwrap();
    let (expired, req) = authorized_intent(&shared, &actor, &client, "expired", |s| {
        s.deadline_ms = now() + 5_000
    })
    .await;
    let dispatch = control::agency::deliver_prepare(&shared, &root, &actor, &expired)
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(
        req.spec.document.deadline_ms.saturating_sub(now()) + 1,
    ))
    .await;
    for _ in 0..2 {
        assert_eq!(
            control::agency::deliver_activation(&shared, &root, &actor, "expired", &dispatch)
                .await
                .unwrap_err()
                .code,
            "DEADLINE_EXPIRED"
        );
        assert_eq!(
            shared
                .lock()
                .await
                .as_ref()
                .unwrap()
                .effect("activate:expired")
                .unwrap()
                .1,
            store::EffectState::Rejected
        );
    }
    let (unknown, req) = authorized_intent(&shared, &actor, &client, "unknown", |_| {}).await;
    {
        let mut lock = shared.lock().await;
        let store = lock.as_mut().unwrap();
        store
            .begin_effect(store.generation(), "prepare:unknown")
            .unwrap();
    }
    assert_eq!(
        control::agency::deliver_prepare(&shared, &root, &actor, &unknown)
            .await
            .unwrap_err()
            .code,
        "DISPATCH_NOT_FOUND"
    );
    assert_eq!(
        shared
            .lock()
            .await
            .as_ref()
            .unwrap()
            .effect("prepare:unknown")
            .unwrap()
            .1,
        store::EffectState::Unknown
    );
    let actual: Dispatch = client.call("prepare", &req).await.unwrap();
    assert_eq!(
        control::agency::deliver_prepare(&shared, &root, &actor, &unknown)
            .await
            .unwrap()
            .key
            .id,
        actual.reference
    );
    drop(shared);
    rig.close().await;
}

#[tokio::test]
async fn blocked_stdin_returns_unknown_without_blocking_other_tenant() {
    let rig = Rig::new("read -r init; exec sleep 30").await;
    let (client, key) = rig.pair("control").await;
    let (other, _) = rig.pair("other").await;
    let d: Dispatch = client
        .call("prepare", &request(&client, "input-block").await)
        .await
        .unwrap();
    activate(&client, &d).await;
    let lease = Lease {
        ticket: ticket(&d, &key, vec![Permission::Takeover], Some("lease")),
        expected_lease: None,
        new_lease: "lease".into(),
    };
    let _: Value = client.call("lease", &lease).await.unwrap();
    let mut timed_out = false;
    for n in 0..32 {
        let result = client
            .call::<_, Value>(
                "input",
                &Input {
                    ticket: ticket(&d, &key, vec![Permission::Input], Some("lease")),
                    idempotency_key: format!("i{n}"),
                    bytes: vec![b'x'; 64 * 1024],
                },
            )
            .await;
        if result.is_err() {
            timed_out = true;
            break;
        }
    }
    assert!(timed_out, "fixture should not drain input");
    let _: Catalog = other.call("catalog", &json!({})).await.unwrap();
    let _: Dispatch = client
        .call("stop", &ticket(&d, &key, vec![Permission::Stop], None))
        .await
        .unwrap();
    rig.close().await;
}

#[tokio::test]
async fn prepare_does_not_execute_and_activation_is_idempotent() {
    let rig = Rig::new(RESULT).await;
    let (client, key) = rig.pair("control").await;
    let request = request(&client, "d1").await;
    let d: Dispatch = client.call("prepare", &request).await.unwrap();
    let results: ResultPage = client
        .call("results", &ResultQuery::of(d.reference.clone()))
        .await
        .unwrap();
    assert!(results.proposals.is_empty());
    let again: Dispatch = client.call("prepare", &request).await.unwrap();
    assert_eq!(d, again);
    let mut changed = request;
    changed.spec.document.budget += 1;
    changed.spec = Sealed::new(changed.spec.document).unwrap();
    assert_eq!(
        client
            .call::<_, Dispatch>("prepare", &changed)
            .await
            .unwrap_err()
            .code,
        "IDEMPOTENCY_CONFLICT"
    );
    activate(&client, &d).await;
    activate(&client, &d).await;
    let trace = terminal(&client, &d, &key).await;
    assert_eq!(trace.dispatch.state, DispatchState::ResultReturned);
    let results: ResultPage = client
        .call("results", &ResultQuery::of(d.reference))
        .await
        .unwrap();
    assert_eq!(results.proposals.len(), 1);
    assert!(results.complete);
    assert_eq!(results.proposals[0].evidence, EvidenceLevel::Narrated);
    rig.close().await;
}

#[tokio::test]
async fn observation_pages_fit_transport_without_losing_large_events() {
    let rig = Rig::new(
        r#"read -r init; /usr/bin/awk 'BEGIN { s="x"; for(i=0;i<18;i++) s=s s; for(i=0;i<20;i++) print "{\"type\":\"observation\",\"kind\":\"large\",\"payload\":\"" s "\"}"; print "{\"type\":\"result\",\"schema\":\"test.result.v1\",\"output\":\"done\"}" }'"#,
    ).await;
    let (client, key) = rig.pair("control").await;
    let d: Dispatch = client
        .call("prepare", &request(&client, "large").await)
        .await
        .unwrap();
    activate(&client, &d).await;
    let mut after = 0;
    let mut count = 0;
    let mut pages = 0;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while std::time::Instant::now() < deadline {
        let result = client
            .call(
                "observe",
                &Observe {
                    ticket: ticket(&d, &key, vec![Permission::Observe], None),
                    after,
                },
            )
            .await;
        // Do not advance the cursor on an unknown read; the same page can be read safely.
        let trace: Trace = match result {
            Err(error) if error.code == "AGENCY_RESPONSE_UNKNOWN" => continue,
            result => result.unwrap(),
        };
        assert!(!trace.gap);
        assert!(canonical(&trace).unwrap().len() < MAX_DOCUMENT);
        if !trace.events.is_empty() {
            pages += 1;
        }
        count += trace
            .events
            .iter()
            .filter(|e| e.kind == "runtime:large")
            .count();
        after = trace.cursor;
        if trace.complete && trace.dispatch.state == DispatchState::ResultReturned {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!(count, 20);
    assert!(pages > 1);
    rig.close().await;
}
#[tokio::test]
async fn tenant_namespace_and_credential_are_isolated() {
    let rig = Rig::new(RESULT).await;
    let (a, a_key) = rig.pair("a").await;
    let (b, _) = rig.pair("b").await;
    let intruder = Pair {
        control_id: "a".into(),
        tenant_key: agency_proto::client::new_credential().unwrap(),
    };
    assert_eq!(
        rig.pairing
            .call::<_, Pairing>("pair", &intruder)
            .await
            .err()
            .unwrap()
            .code,
        "TENANT_EXISTS"
    );
    assert_eq!(
        rig.pairing
            .call::<_, Value>("shutdown", &json!({}))
            .await
            .unwrap_err()
            .code,
        "PAIRING_REQUIRED"
    );
    let d: Dispatch = a
        .call("prepare", &request(&a, "same-key").await)
        .await
        .unwrap();
    assert_eq!(
        b.call::<_, ResultPage>("results", &ResultQuery::of(d.reference))
            .await
            .unwrap_err()
            .code,
        "DISPATCH_NOT_FOUND"
    );
    let db: Dispatch = b
        .call("prepare", &request(&b, "same-key").await)
        .await
        .unwrap();
    assert_eq!(db.state, DispatchState::Prepared);
    let pair: Pairing = rig
        .pairing
        .call(
            "pair",
            &Pair {
                control_id: "a".into(),
                tenant_key: a_key,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        Client::new(pair.endpoint.into(), "wrong-key".into())
            .call::<_, Catalog>("catalog", &json!({}))
            .await
            .unwrap_err()
            .code,
        "PAIRING_REQUIRED"
    );
    assert_eq!(
        b.call::<_, Pairing>(
            "pair",
            &Pair {
                control_id: "c".into(),
                tenant_key: agency_proto::client::new_credential().unwrap(),
            }
        )
        .await
        .err()
        .unwrap()
        .code,
        "METHOD_DENIED"
    );
    rig.close().await;
}
#[tokio::test]
async fn mismatched_material_capability_and_profession_fail_before_activation() {
    let rig = Rig::new(RESULT).await;
    let (client, _) = rig.pair("a").await;
    let mut consumer = request(&client, "wrong-consumer").await;
    consumer.bundle.document.consumer.id = "another-invocation".into();
    consumer.bundle = Sealed::new(consumer.bundle.document).unwrap();
    consumer.spec.document.bundle.digest = consumer.bundle.digest.clone();
    consumer.spec = Sealed::new(consumer.spec.document).unwrap();
    assert_eq!(
        client
            .call::<_, Dispatch>("prepare", &consumer)
            .await
            .unwrap_err()
            .code,
        "BUNDLE_MISMATCH"
    );
    let mut req = request(&client, "material").await;
    if let Delivery::Inline { bytes } = &mut req.bundle.document.entries[0].delivery {
        bytes.push(1);
    }
    req.bundle = Sealed::new(req.bundle.document).unwrap();
    req.spec.document.bundle.digest = req.bundle.digest.clone();
    req.spec = Sealed::new(req.spec.document).unwrap();
    assert_eq!(
        client
            .call::<_, Dispatch>("prepare", &req)
            .await
            .unwrap_err()
            .code,
        "DELIVERY_DIGEST_MISMATCH"
    );
    let mut req = request(&client, "cap").await;
    req.spec.document.required_capabilities.secure_input = true;
    req.spec = Sealed::new(req.spec.document).unwrap();
    assert_eq!(
        client
            .call::<_, Dispatch>("prepare", &req)
            .await
            .unwrap_err()
            .code,
        "CAPABILITY_MISSING"
    );
    let mut req = request(&client, "profession").await;
    req.spec.document.profession.terms = "new terms".into();
    req.spec = Sealed::new(req.spec.document).unwrap();
    assert_eq!(
        client
            .call::<_, Dispatch>("prepare", &req)
            .await
            .unwrap_err()
            .code,
        "PROFESSION_CHANGED"
    );
    let mut req = request(&client, "required-skill").await;
    req.spec.document.profession.skills.push(SkillClaim {
        reference: reference("uninstalled"),
        required: true,
        verification: None,
    });
    req.spec = Sealed::new(req.spec.document).unwrap();
    assert_eq!(
        client
            .call::<_, Dispatch>("prepare", &req)
            .await
            .unwrap_err()
            .code,
        "SKILL_MISSING"
    );
    let mut req = request(&client, "skill-digest").await;
    req.spec.document.profession.skills.push(SkillClaim {
        reference: reference("skill"),
        required: true,
        verification: Some(SkillVerification {
            source: EvidenceLevel::Unmediated,
            report: reference("report"),
            readback_digest: hash(b"changed"),
        }),
    });
    req.spec = Sealed::new(req.spec.document).unwrap();
    assert_eq!(
        client
            .call::<_, Dispatch>("prepare", &req)
            .await
            .unwrap_err()
            .code,
        "SKILL_DIGEST_MISMATCH"
    );
    rig.close().await;
}
#[tokio::test]
async fn input_lease_takeover_is_atomic_and_permissions_are_separate() {
    let rig=Rig::new("read -r init; read -r answer; printf '%s\n' '{\"type\":\"result\",\"schema\":\"test.result.v1\",\"output\":\"answer\"}'").await;
    let (client, key) = rig.pair("a").await;
    let d: Dispatch = client
        .call("prepare", &request(&client, "input").await)
        .await
        .unwrap();
    activate(&client, &d).await;
    let old = ticket(&d, &key, vec![Permission::Input], Some("old"));
    assert_eq!(
        client
            .call::<_, Value>(
                "input",
                &Input {
                    ticket: old.clone(),
                    idempotency_key: "input".into(),
                    bytes: b"hello\n".to_vec()
                }
            )
            .await
            .unwrap_err()
            .code,
        "INPUT_LEASE_INVALID"
    );
    let _: Value = client
        .call(
            "lease",
            &Lease {
                ticket: ticket(&d, &key, vec![Permission::Takeover], Some("old")),
                expected_lease: None,
                new_lease: "old".into(),
            },
        )
        .await
        .unwrap();
    let _: Value = client
        .call(
            "lease",
            &Lease {
                ticket: ticket(&d, &key, vec![Permission::Takeover], Some("new")),
                expected_lease: Some("old".into()),
                new_lease: "new".into(),
            },
        )
        .await
        .unwrap();
    assert_eq!(
        client
            .call::<_, Value>(
                "input",
                &Input {
                    ticket: old,
                    idempotency_key: "input".into(),
                    bytes: b"hello\n".to_vec()
                }
            )
            .await
            .unwrap_err()
            .code,
        "INPUT_LEASE_INVALID"
    );
    assert_eq!(
        client
            .call::<_, Dispatch>("stop", &ticket(&d, &key, vec![Permission::Observe], None))
            .await
            .unwrap_err()
            .code,
        "TICKET_DENIED"
    );
    let input = Input {
        ticket: ticket(&d, &key, vec![Permission::Input], Some("new")),
        idempotency_key: "input".into(),
        bytes: b"hello\n".to_vec(),
    };
    let _: Value = client.call("input", &input).await.unwrap();
    let value: Value = client.call("input", &input).await.unwrap();
    assert_eq!(value["delivery"], "delivered");
    assert_eq!(
        terminal(&client, &d, &key).await.dispatch.state,
        DispatchState::ResultReturned
    );
    rig.close().await;
}
#[tokio::test]
async fn old_writer_is_fenced_without_destroying_pending_results_or_other_tenant() {
    let rig = Rig::new(RESULT).await;
    let (a, key) = rig.pair("a").await;
    let (b, _) = rig.pair("b").await;
    let d: Dispatch = a.call("prepare", &request(&a, "d").await).await.unwrap();
    activate(&a, &d).await;
    terminal(&a, &d, &key).await;
    let _: Value = a
        .call(
            "fence",
            &Fence {
                writer_generation: 2,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        a.call::<_, Dispatch>(
            "activate",
            &DispatchAction {
                dispatch: d.reference.clone(),
                writer_generation: 1,
                idempotency_key: "act".into()
            }
        )
        .await
        .unwrap_err()
        .code,
        "WRITER_STALE"
    );
    assert_eq!(
        a.call::<_, Value>(
            "input",
            &Input {
                ticket: ticket(&d, &key, vec![Permission::Input], Some("old")),
                idempotency_key: "old-writer-input".into(),
                bytes: b"do not deliver".to_vec(),
            }
        )
        .await
        .unwrap_err()
        .code,
        "WRITER_STALE"
    );
    let results: ResultPage = a
        .call("results", &ResultQuery::of(d.reference.clone()))
        .await
        .unwrap();
    assert_eq!(results.proposals.len(), 1);
    assert!(
        b.call::<_, Dispatch>("prepare", &request(&b, "d").await)
            .await
            .is_ok()
    );
    let bad = Preservation {
        dispatch: d.reference.clone(),
        proposal_id: results.proposals[0].header.proposal_id.clone(),
        content_digest: hash(b"summary"),
    };
    assert_eq!(
        a.call::<_, Value>("preserve", &bad).await.unwrap_err().code,
        "PRESERVATION_MISMATCH"
    );
    let good = Preservation {
        content_digest: results.proposals[0].content_digest.clone(),
        ..bad
    };
    let _: Value = a.call("preserve", &good).await.unwrap();
    let _: Value = a.call("preserve", &good).await.unwrap();
    rig.close().await;
}
#[tokio::test]
async fn missing_terminal_result_is_protocol_failure_and_cancel_has_exit_report() {
    let rig = Rig::new("read -r init; exit 0").await;
    let (client, key) = rig.pair("a").await;
    let d: Dispatch = client
        .call("prepare", &request(&client, "missing").await)
        .await
        .unwrap();
    activate(&client, &d).await;
    let trace = terminal(&client, &d, &key).await;
    assert_eq!(trace.dispatch.state, DispatchState::CannotFulfill);
    assert!(trace.events.iter().any(|e| e.kind == "protocol_error"));
    rig.close().await;
    let rig = Rig::new("read -r init; read -r input").await;
    let (client, key) = rig.pair("a").await;
    let d: Dispatch = client
        .call("prepare", &request(&client, "stop").await)
        .await
        .unwrap();
    activate(&client, &d).await;
    let _: Dispatch = client
        .call("stop", &ticket(&d, &key, vec![Permission::Stop], None))
        .await
        .unwrap();
    let trace = terminal(&client, &d, &key).await;
    assert_eq!(trace.dispatch.state, DispatchState::Cancelled);
    assert_eq!(trace.events.last().unwrap().payload["requested_stop"], true);
    rig.close().await;
}
#[tokio::test]
async fn cursor_gap_and_unknown_event_are_explicit() {
    let rig=Rig::new("read -r init; printf '%s\n' '{\"type\":\"observation\",\"kind\":\"new_vendor_event\",\"payload\":{\"done\":true}}' '{\"type\":\"result\",\"schema\":\"test.result.v1\",\"output\":\"answer\"}'").await;
    let (client, key) = rig.pair("a").await;
    let d: Dispatch = client
        .call("prepare", &request(&client, "events").await)
        .await
        .unwrap();
    activate(&client, &d).await;
    let trace = terminal(&client, &d, &key).await;
    assert!(
        trace
            .events
            .iter()
            .any(|e| e.kind == "runtime:new_vendor_event" && e.source == EvidenceLevel::Narrated)
    );
    let gap: Trace = client
        .call(
            "observe",
            &Observe {
                ticket: ticket(&d, &key, vec![Permission::Observe], None),
                after: trace.cursor + 10,
            },
        )
        .await
        .unwrap();
    assert!(gap.gap);
    assert!(!gap.complete);
    rig.close().await;
}

#[tokio::test]
async fn restart_retains_original_results_and_prepared_dispatch_without_rerun() {
    let rig = Rig::new(RESULT).await;
    let (client, key) = rig.pair("control").await;
    let prepared: Dispatch = client
        .call("prepare", &request(&client, "not-started").await)
        .await
        .unwrap();
    let done: Dispatch = client
        .call("prepare", &request(&client, "done").await)
        .await
        .unwrap();
    activate(&client, &done).await;
    terminal(&client, &done, &key).await;
    let before: ResultPage = client
        .call("results", &ResultQuery::of(done.reference.clone()))
        .await
        .unwrap();
    let rig = rig.restart(RESULT).await;
    let after: ResultPage = client
        .call("results", &ResultQuery::of(done.reference.clone()))
        .await
        .unwrap();
    assert_eq!(
        canonical(&before.proposals).unwrap(),
        canonical(&after.proposals).unwrap()
    );
    assert!(!after.proposals[0].preserved);
    let trace: Trace = client
        .call(
            "observe",
            &Observe {
                ticket: ticket(&prepared, &key, vec![Permission::Observe], None),
                after: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(trace.dispatch.state, DispatchState::Prepared);
    let _: Value = client
        .call(
            "preserve",
            &Preservation {
                dispatch: done.reference,
                proposal_id: after.proposals[0].header.proposal_id.clone(),
                content_digest: after.proposals[0].content_digest.clone(),
            },
        )
        .await
        .unwrap();
    rig.close().await;
}

#[tokio::test]
async fn invalid_stream_marks_gap_and_never_reports_complete_history() {
    let rig = Rig::new("read -r init; printf 'not-a-frame\\n'").await;
    let (client, key) = rig.pair("control").await;
    let d: Dispatch = client
        .call("prepare", &request(&client, "invalid-stream").await)
        .await
        .unwrap();
    activate(&client, &d).await;
    let trace = terminal(&client, &d, &key).await;
    assert!(trace.gap);
    assert!(!trace.complete);
    assert_eq!(trace.dispatch.state, DispatchState::CannotFulfill);
    rig.close().await;
}

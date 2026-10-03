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
    handle: tokio::task::JoinHandle<Result<()>>,
}
impl Rig {
    async fn new(script: &str) -> Self {
        let root = PathBuf::from(format!(
            "/tmp/agency-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let runtime: Arc<dyn Runtime> = Arc::new(ScriptRuntime::new(ScriptConfig {
            program: "/bin/sh".into(),
            arguments: vec!["-c".into(), script.into()],
        }));
        let service = Agency::open(&root, runtime).unwrap();
        let admin = Client::new(
            agency_proto::client::admin_endpoint(&root).unwrap(),
            Agency::bootstrap_key(&root).unwrap(),
        );
        let handle = tokio::spawn(agency::serve(service));
        for _ in 0..100 {
            if admin.call::<_, Value>("catalog", &json!({})).await.is_ok() {
                return Self {
                    root,
                    admin,
                    handle,
                };
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        panic!("Agency not ready");
    }
    async fn pair(&self, id: &str) -> (Client, String) {
        let pair: Pairing = self
            .admin
            .call(
                "pair",
                &Pair {
                    control_id: id.into(),
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
        std::fs::remove_dir_all(agency_proto::client::socket_directory(&self.root).unwrap())
            .unwrap();
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
        (intent, owner)
    };
    let dispatch = control::agency::deliver_prepare(&shared, &root, &actor, &intent)
        .await
        .unwrap();
    // A lost prepare reply is resolved by lookup of the original key, not a second start.
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
        let result: Vec<Proposal> = client
            .call(
                "results",
                &ResultQuery {
                    dispatch: running.reference.clone(),
                },
            )
            .await
            .unwrap();
        if !result.is_empty() {
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
    let results: Vec<Proposal> = client
        .call(
            "results",
            &ResultQuery {
                dispatch: running.reference.clone(),
            },
        )
        .await
        .unwrap();
    assert!(results[0].preserved);
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
        let mut wrong = results[0].clone();
        wrong.outputs[0].owner.project = "another-project".into();
        assert_eq!(
            participant::preserve_proposal(s, &actor, &dispatch, &wrong)
                .unwrap_err()
                .code,
            "PROPOSAL_MISMATCH"
        );
        let mut direct = results[0].clone();
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
                &results[0].header.proposal_id,
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
    std::fs::remove_dir_all(agency_proto::client::socket_directory(&rig.root).unwrap()).unwrap();
    std::fs::remove_dir_all(&rig.root).unwrap();
}

#[tokio::test]
async fn deadline_stops_fixture_without_revoking_result_observation() {
    let rig = Rig::new("read -r init; exec sleep 30").await;
    let (client, key) = rig.pair("control").await;
    let mut request = request(&client, "deadline").await;
    request.spec.document.deadline_ms = now() + 100;
    request.spec = Sealed::new(request.spec.document).unwrap();
    let d: Dispatch = client.call("prepare", &request).await.unwrap();
    activate(&client, &d).await;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
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
    let results: Vec<Proposal> = client
        .call(
            "results",
            &ResultQuery {
                dispatch: d.reference.clone(),
            },
        )
        .await
        .unwrap();
    assert!(results.is_empty());
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
    let results: Vec<Proposal> = client
        .call(
            "results",
            &ResultQuery {
                dispatch: d.reference,
            },
        )
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].evidence, EvidenceLevel::Narrated);
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
    for _ in 0..200 {
        let trace: Trace = client
            .call(
                "observe",
                &Observe {
                    ticket: ticket(&d, &key, vec![Permission::Observe], None),
                    after,
                },
            )
            .await
            .unwrap();
        assert!(!trace.gap);
        assert!(canonical(&trace).unwrap().len() < MAX_DOCUMENT);
        if !trace.events.is_empty() {
            pages += 1;
        }
        count += trace.events.iter().filter(|e| e.kind == "large").count();
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
    let (a, _) = rig.pair("a").await;
    let (b, _) = rig.pair("b").await;
    let d: Dispatch = a
        .call("prepare", &request(&a, "same-key").await)
        .await
        .unwrap();
    assert_eq!(
        b.call::<_, Vec<Proposal>>(
            "results",
            &ResultQuery {
                dispatch: d.reference
            }
        )
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
        .admin
        .call(
            "pair",
            &Pair {
                control_id: "a".into(),
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
                control_id: "c".into()
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
    let results: Vec<Proposal> = a
        .call(
            "results",
            &ResultQuery {
                dispatch: d.reference.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(
        b.call::<_, Dispatch>("prepare", &request(&b, "d").await)
            .await
            .is_ok()
    );
    let bad = Preservation {
        dispatch: d.reference.clone(),
        proposal_id: results[0].header.proposal_id.clone(),
        content_digest: hash(b"summary"),
    };
    assert_eq!(
        a.call::<_, Value>("preserve", &bad).await.unwrap_err().code,
        "PRESERVATION_MISMATCH"
    );
    let good = Preservation {
        content_digest: results[0].content_digest.clone(),
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
            .any(|e| e.kind == "new_vendor_event" && e.source == EvidenceLevel::Narrated)
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
    let before: Vec<Proposal> = client
        .call(
            "results",
            &ResultQuery {
                dispatch: done.reference.clone(),
            },
        )
        .await
        .unwrap();
    let rig = rig.restart(RESULT).await;
    let after: Vec<Proposal> = client
        .call(
            "results",
            &ResultQuery {
                dispatch: done.reference.clone(),
            },
        )
        .await
        .unwrap();
    assert_eq!(canonical(&before).unwrap(), canonical(&after).unwrap());
    assert!(!after[0].preserved);
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
                proposal_id: after[0].header.proposal_id.clone(),
                content_digest: after[0].content_digest.clone(),
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

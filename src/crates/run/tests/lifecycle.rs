use agency_proto::{Capabilities, FrozenRef, Sealed};
use chat::{decode, key, reference, value_record};
use foundation::{bytes_sha256, canonical_json_sha256};
use run::lifecycle::{self as life, Action, Input, Start, State, engine};
use serde_json::{Value, json};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use store::{
    Actor, ActorSource, Command, Expected, Record, RecordData, Reference, Scope, Store,
    TrustedActor, Version,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
const NOW: u64 = 1_000_000;
const DEADLINE: u64 = 9_000_000_000_000;
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "hctl-run-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
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
fn actor() -> TrustedActor {
    TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![
            Scope::Control,
            Scope::Project("P".into()),
            Scope::Project("Q".into()),
            Scope::Repo("R".into()),
        ],
        authority: None,
    })
}
fn sha(s: &str) -> String {
    bytes_sha256(s.as_bytes())
}
fn target(scope: Scope, kind: &str, id: &str) -> Reference {
    Reference {
        key: key(scope, kind, id),
        version: Version::State(1),
    }
}
fn p(kind: &str, id: &str) -> Reference {
    target(Scope::Project("P".into()), kind, id)
}
fn c(kind: &str, id: &str) -> Reference {
    target(Scope::Control, kind, id)
}
fn frozen(id: &str) -> FrozenRef {
    FrozenRef {
        id: id.into(),
        revision: "1".into(),
        digest: sha(id),
    }
}
fn seed(store: &mut Store, records: &[Record], name: &str) {
    let actor = actor();
    let input = json!({"name":name});
    let command = Command {
        command_id: name.into(),
        idempotency_key: name.into(),
        actor: actor.0.clone(),
        target: key(Scope::Control, "fixture", name),
        expected: Expected::Absent,
        binding: c("module", "fixture"),
        operation: "fixture".into(),
        input_digest: Command::digest_input("fixture", &input).unwrap(),
        input,
    };
    store
        .submit(store.generation(), &actor, &command, None, |tx| {
            for r in records {
                for m in &r.materials {
                    tx.admit_material(m)?;
                }
                tx.put(r)?;
            }
            Ok(json!({}))
        })
        .unwrap();
}
fn material_record<T: serde::Serialize>(
    store: &mut Store,
    kind: &str,
    id: &str,
    value: &T,
    command: &str,
) -> Record {
    let mut r = value_record(p(kind, id).key, 1, value).unwrap();
    let bytes = serde_json::to_vec(value).unwrap();
    let m = store
        .save_material(
            store.generation(),
            &actor(),
            &Scope::Project("P".into()),
            command,
            "body",
            &bytes,
        )
        .unwrap();
    r.materials.push(m);
    r
}
struct NoEngine;
impl run::Validator for NoEngine {
    fn validate(&self, _: &run::Compiled) -> store::Result<()> {
        Ok(())
    }
}
fn workflow(gate: bool) -> run::Workflow {
    let mut nodes = vec![
        json!({"id":"build","outputs":{"code":"change_set_revision"},"operation":{"kind":"external","criterion":{"kind":"mechanical","output":"code","fact":{"kind":"ci","check":"unit"}},"seats":[],"gate":null}}),
    ];
    if gate {
        let seats:Vec<_>=["a","b"].iter().map(|id|json!({"id":id,"required":true,"profession":"review","evidence":"transcript","required_skills":[],"review_subject_node":"build","template":null})).collect();
        nodes.push(json!({"id":"review","outputs":{"verdict":"verdict"},"operation":{"kind":"external","criterion":{"kind":"gate","evidence":"transcript"},"seats":seats,"gate":{"quorum":2,"distinct_worker_profiles":false}}}));
    }
    serde_json::from_value(json!({"schema":"hctl2.workflow.v1","profile":"hctl2.passive.v1","readback":{"required":false,"different_worker_profile":true},"graph":{"nodes":nodes,"edges":[]}})).unwrap()
}
fn approve(store: &mut Store, id: &str, w: run::Workflow) {
    for (kind, action) in [
        (
            "register",
            json!({"kind":"register","project_id":"P","project_version":1,"workflow_id":id,"supersedes":null,"task":null,"workflow":w,"snapshot":null,"provenance":null}),
        ),
        (
            "compile",
            json!({"kind":"compile","project_id":"P","project_version":1,"workflow_id":id,"revision":1}),
        ),
        (
            "approve",
            json!({"kind":"approve","project_id":"P","project_version":1,"workflow_id":id,"revision":1,"readback":null}),
        ),
    ] {
        let input: run::Input =
            serde_json::from_value(json!({"key":format!("{id}-{kind}"),"action":action})).unwrap();
        let plan = run::prepare(store, input, &actor(), &NoEngine).unwrap();
        run::admit(store, &actor(), plan, &NoEngine).unwrap();
    }
}
fn setup(store: &mut Store) -> Start {
    let config = store
        .save_material(
            store.generation(),
            &actor(),
            &Scope::Repo("R".into()),
            "setup",
            "config",
            b"{}",
        )
        .unwrap();
    let registration = json!({"repo_id":"R","version":1,"lifecycle":"active","abandoned":false,"command_id":"repo","idempotency_key":"repo","actor":actor().0,"prepared":{"request":{"name":"R","origin":"independent","platform":"none","instance":null,"platform_repo_id":null,"platform_path":null,"local":null,"remote_evidence":null,"default_source":null},"platform":"none","local":null,"evidence_conflicts":[],"sources":[]},"sources":[],"config":config,"observed":null,"delivered":true,"residual":null,"residual_target":null});
    let data = RecordData::Repo {
        platform_binding: None,
        registration: Some(registration),
    };
    let repo = Record {
        key: repo::key("R"),
        version: 1,
        revision_digest: canonical_json_sha256(&serde_json::to_value(&data).unwrap()).unwrap(),
        data,
        sources: vec![],
        materials: vec![config],
    };
    let mut project = value_record(p("project", "P").key, 1, &json!({})).unwrap();
    project.data = RecordData::Project {
        repo_id: "R".into(),
        settings: store::ProjectSettings {
            selection_policy: json!({}),
            publish_review_requires_confirmation: true,
        },
        archived: false,
    };
    let mut q = project.clone();
    q.key = key(Scope::Project("Q".into()), "project", "Q");
    let context = Sealed::new(agency_proto::context::Manifest {
        id: "root".into(),
        purpose: "frozen root".into(),
        scope: "project P".into(),
        parent: None,
        sources: vec![frozen("code")],
        selection_policy: frozen("selection"),
        freshness: "exact".into(),
        coverage: "fixture".into(),
        known_gaps: vec![],
        required_skills: vec![],
        permission_digest: sha("permissions"),
        redaction: frozen("redaction"),
        budget: 4096,
    })
    .unwrap();
    let context_record = material_record(store, "context_manifest", "root", &context, "setup");
    let engine = value_record(
        c("workflow_engine_binding", "fake").key,
        1,
        &json!({"adapter":"internal-test-double","version":"1"}),
    )
    .unwrap();
    let room = value_record(
        p("room_binding", "delivery").key,
        1,
        &json!({"closed":false}),
    )
    .unwrap();
    seed(
        store,
        &[repo, project, q, context_record, engine, room],
        "setup",
    );
    approve(store, "plan", workflow(false));
    let profession = agency_proto::Profession {
        reference: frozen("review"),
        harness: frozen("fixture-harness"),
        model: "model".into(),
        persona: "review".into(),
        terms: "fixture only".into(),
        default_role: "review".into(),
        skills: vec![],
        capabilities: Capabilities::default(),
    };
    let binding = participant::Binding {
        id: "A".into(),
        protocol: agency_proto::PROTOCOL.into(),
        catalog: agency_proto::Catalog {
            professions: vec![profession.clone()],
            harnesses: vec![profession.harness.clone()],
            skills: vec![],
        },
    };
    participant::accept_binding(store, &actor(), "accept-agency", binding).unwrap();
    participant::accept_profession(store, &actor(), "accept-profession", "A", &profession).unwrap();
    for (name, max) in [("one", 4096), ("two", 3072)] {
        let input = participant::profiles::ProfileInput {
            key: format!("profile-{name}"),
            action: participant::profiles::ProfileAction::Create {
                id: name.into(),
                profile: participant::profiles::WorkerProfile {
                    harness: profession.harness.clone(),
                    model: profession.model.clone(),
                    mode: "read_only".into(),
                    permissions: vec!["context.read".into()],
                    environment: vec![],
                    required_capabilities: Capabilities::default(),
                    max_context_bytes: max,
                },
            },
        };
        let plan = participant::profiles::prepare_profile(store, input, &actor()).unwrap();
        participant::profiles::admit_profile(store, &actor(), plan).unwrap();
    }
    Start {
        project_id: "P".into(),
        project_version: 1,
        run_id: "first".into(),
        workflow_id: "plan".into(),
        workflow_revision: 1,
        task_revisions: vec![],
        repo: target(Scope::Repo("R".into()), "repo", "R"),
        baseline: "1".repeat(40),
        context: life::ExactContext {
            reference: p("context_manifest", "root"),
            digest: context.digest,
        },
        engine_binding: c("workflow_engine_binding", "fake"),
        picks: vec![],
        policy: life::Policy {
            max_attempts: 10,
            context_bytes: 4096,
            deadline_ms: DEADLINE,
            transition_timeout_ms: 900_000,
            incremental_review: false,
            switch_on: vec!["transport_unavailable".into()],
            network_scope: vec![],
            secret_scope: vec![],
        },
        integration: life::Integration {
            target_ref: "refs/heads/main".into(),
            target_kind: repo::integration::TargetKind::Local,
            form: repo::integration::Form::ExpectedHead,
            strategy: repo::integration::Strategy::FastForward,
            mode: life::IntegrationMode::Automatic,
        },
        placement: life::Placement {
            destination: None,
            allowed_destinations: vec![p("room_binding", "delivery")],
        },
    }
}
fn fixture() -> (Temp, Store, Start) {
    let t = Temp::new();
    let mut store = Store::open(&t.0).unwrap();
    let start = setup(&mut store);
    (t, store, start)
}
fn start_input(spec: Start, key: &str) -> Input {
    Input {
        key: key.into(),
        action: Action::Start {
            spec: Box::new(spec),
        },
    }
}
fn submit(store: &mut Store, input: Input) -> Value {
    let plan = life::prepare(store, &actor(), input).unwrap();
    life::admit(store, &actor(), plan, NOW).unwrap()
}
fn command(kind: &str, id: &str, version: i64) -> Input {
    serde_json::from_value(json!({"key":format!("{kind}-{id}-{version}"),"action":{"kind":kind,"project_id":"P","run_id":id,"version":version}})).unwrap()
}
fn finish(
    store: &mut Store,
    actor: &TrustedActor,
    version: i64,
    outcome: State,
    proof: Option<&Reference>,
    now: u64,
) -> store::Result<Value> {
    life::finish(
        store,
        actor,
        life::Finish {
            key: format!(
                "finish:{version}:{outcome:?}:{}",
                proof.map(|r| r.key.id.as_str()).unwrap_or("none")
            ),
            project_id: "P".into(),
            run_id: "first".into(),
            version,
            outcome,
            proof: proof.cloned(),
        },
        now,
    )
}
fn current(store: &Store, id: &str) -> (i64, life::Run) {
    let (record, run) = life::load(store, "P", id).unwrap();
    (record.version, run)
}
#[derive(Default)]
struct Fake {
    calls: Cell<usize>,
    lost: Cell<bool>,
    results: RefCell<BTreeMap<String, engine::Observation>>,
}
impl engine::Port for Fake {
    fn readback(&self, effect: &store::EffectIntent) -> store::Result<engine::Observation> {
        Ok(self
            .results
            .borrow()
            .get(&effect.intent_id)
            .cloned()
            .unwrap_or(engine::Observation::NotApplied {}))
    }
    fn execute(&self, effect: &store::EffectIntent) -> store::Result<engine::Observation> {
        self.calls.set(self.calls.get() + 1);
        let applied = engine::Observation::Applied {
            execution_id: format!("execution-{}", effect.target),
            engine_generation: 7,
        };
        self.results
            .borrow_mut()
            .insert(effect.intent_id.clone(), applied.clone());
        if self.lost.replace(false) {
            Ok(engine::Observation::Unknown {})
        } else {
            Ok(applied)
        }
    }
}
fn drive(store: &mut Store, id: &str, op: &str, fake: &Fake) {
    let (_, run) = current(store, id);
    let actor = engine::actor_for(&run);
    let effect = store
        .pending_effects()
        .unwrap()
        .into_iter()
        .find(|key| {
            let (e, _) = store.effect(key).unwrap();
            e.target == id && e.operation == format!("run.engine.{op}")
        })
        .unwrap();
    engine::drive(store, &actor, &effect, fake, NOW + 100).unwrap();
}
fn bind_task(store: &mut Store, spec: &mut Start) {
    let contract = json!({"scope":"change code","expected_outcome":"works","acceptance":[{"text":"tests pass","grade":"human"}],"roles":[],"capabilities":[]});
    let material = store
        .save_material(
            store.generation(),
            &actor(),
            &Scope::Project("P".into()),
            "task-setup",
            "contract",
            &serde_json::to_vec(&contract).unwrap(),
        )
        .unwrap();
    let revision:task::Revision=serde_json::from_value(json!({"number":1,"material":material,"origin":{"kind":"local","reference":p("project","P"),"proposal_digest":sha("proposal")},"proposal_digest":sha("proposal"),"binding":null,"policy_digest":sha("policy"),"backend_projection_digest":null})).unwrap();
    let source = target(Scope::Repo("R".into()), "task_source", "issues");
    let source_record=value_record(source.key.clone(),1,&json!({"id":"issues","repo_id":"R","port_kind":"gitea_issues","candidate":{"id":"issues","provider":"gitea_issues","actual_source_and_scope":"test","recommended":true,"create":true,"field_writeback":true,"can_claim":true,"available":true},"platform":{"instance":"http://127.0.0.1:1","stable_id":"1","full_name":"owner/repo","clone_url":"unused","account_id":"1","has_issues":true,"can_write_issues":true,"credential_ref":"fixture-no-credential"},"capabilities":{"create":true,"field_writeback":true,"conditional_write":false,"conditional_fields":[],"placement":true,"delete":true},"board_scope_stable_id":"test","binding_revision":1,"active":true})).unwrap();
    let entity = store::ExternalEntity {
        provider: "gitea_issues".into(),
        account_stable_id: "1".into(),
        external_entity_kind: "issue".into(),
        immutable_external_entity_id: "1".into(),
    };
    let snapshot = target(Scope::Repo("R".into()), "task_snapshot", "issues");
    let card = task::Card {
        entity: entity.clone(),
        number: 1,
        title: "Task".into(),
        body: "code".into(),
        stage: "open".into(),
        remote_revision: "1".into(),
        content_version: None,
        groups: vec![],
        dependencies: task::Dependencies::default(),
        comments: vec![],
        raw: json!({}),
        tombstone: false,
    };
    let snap = value_record(
        snapshot.key.clone(),
        1,
        &task::Snapshot {
            source: source.clone(),
            observed_at: NOW / 1000,
            complete: true,
            error: None,
            cards: vec![card],
            stable_groups: vec![],
        },
    )
    .unwrap();
    let state = task::Task {
        id: "T".into(),
        project_id: "P".into(),
        source_id: "issues".into(),
        repo_id: "R".into(),
        entity: Some(entity),
        number: Some(1),
        title: "Task".into(),
        lifecycle: "open".into(),
        lifecycle_version: 1,
        archived: false,
        revision: Some(revision.clone()),
        snapshot: Some(snapshot),
        state_version: 1,
        needs_attention: false,
        pending_contract: None,
        run_occupancy: None,
    };
    let state = value_record(p("task_state", "T").key, 1, &state).unwrap();
    let mut rev = value_record(p("task_revision", "T:1").key, 1, &revision).unwrap();
    rev.materials.push(material);
    seed(store, &[source_record, snap, state, rev], "task-setup");
    spec.task_revisions = vec![p("task_revision", "T:1")];
}
#[test]
fn transition_table_is_closed_for_all_states() {
    use State::*;
    let states = [
        Starting, Running, Pausing, Paused, Cancelling, Completed, Failed, Cancelled, Replaced,
    ];
    let expected = [
        (Starting, Running),
        (Starting, Failed),
        (Starting, Cancelled),
        (Starting, Replaced),
        (Running, Pausing),
        (Running, Cancelling),
        (Running, Completed),
        (Running, Failed),
        (Running, Replaced),
        (Pausing, Paused),
        (Pausing, Cancelling),
        (Pausing, Failed),
        (Pausing, Replaced),
        (Paused, Running),
        (Paused, Cancelling),
        (Paused, Failed),
        (Paused, Replaced),
        (Cancelling, Cancelled),
        (Cancelling, Failed),
        (Cancelling, Replaced),
    ];
    for from in states {
        for to in states {
            assert_eq!(
                from.allows(to),
                expected.contains(&(from, to)),
                "{from:?}->{to:?}"
            );
        }
    }
}
#[test]
fn start_is_atomic_idempotent_and_freezes_permission_to_integrate() {
    let (_t, mut store, mut spec) = fixture();
    bind_task(&mut store, &mut spec);
    let input = start_input(spec.clone(), "first");
    let plan = life::prepare(&store, &actor(), input.clone()).unwrap();
    assert!(store.list("run").unwrap().is_empty());
    assert!(
        task::task(&store, "P", "T")
            .unwrap()
            .1
            .run_occupancy
            .is_none()
    );
    assert_eq!(plan.result["integration"]["mode"], "automatic");
    assert_eq!(plan.result["integration"]["target_ref"], "refs/heads/main");
    let first = life::admit(&mut store, &actor(), plan, NOW).unwrap();
    assert_eq!(first, submit(&mut store, input));
    assert_eq!(store.list("run").unwrap().len(), 1);
    assert_eq!(store.list("run_manifest").unwrap().len(), 1);
    assert_eq!(store.pending_effects().unwrap().len(), 1);
    assert_eq!(
        task::task(&store, "P", "T")
            .unwrap()
            .1
            .run_occupancy
            .unwrap()["state"],
        "active"
    );
    let (_, m) = life::manifest(&store, &actor(), "P", "first").unwrap();
    assert_eq!(m.policy.transition_timeout_ms, 900_000);
    assert!(m.frozen_rework_limits.is_empty());
    spec.run_id = "second".into();
    assert_eq!(
        life::prepare(&store, &actor(), start_input(spec, "second"))
            .unwrap_err()
            .code,
        "RUN_ACTIVE"
    );
}
#[test]
fn all_start_guards_and_preview_cas_fail_without_partial_records() {
    let (_t, mut store, mut spec) = fixture();
    bind_task(&mut store, &mut spec);
    let mut two = spec.clone();
    two.task_revisions.push(p("task_revision", "T:1"));
    assert_eq!(
        life::prepare(&store, &actor(), start_input(two, "two"))
            .unwrap_err()
            .code,
        "INVALID_INPUT"
    );
    let mut foreign = spec.clone();
    foreign.task_revisions[0].key.scope = Scope::Project("Q".into());
    assert_eq!(
        life::prepare(&store, &actor(), start_input(foreign, "foreign"))
            .unwrap_err()
            .code,
        "INVALID_INPUT"
    );
    let mut unapproved = spec.clone();
    unapproved.workflow_id = "unapproved".into();
    let input:run::Input=serde_json::from_value(json!({"key":"register-unapproved","action":{"kind":"register","project_id":"P","project_version":1,"workflow_id":"unapproved","supersedes":null,"task":null,"workflow":workflow(false),"snapshot":null,"provenance":null}})).unwrap();
    let p = run::prepare(&store, input, &actor(), &NoEngine).unwrap();
    run::admit(&mut store, &actor(), p, &NoEngine).unwrap();
    assert_eq!(
        life::prepare(&store, &actor(), start_input(unapproved, "unapproved"))
            .unwrap_err()
            .code,
        "WORKFLOW_NOT_APPROVED"
    );
    let plan = life::prepare(&store, &actor(), start_input(spec.clone(), "preview")).unwrap();
    let (record, mut task) = task::task(&store, "P", "T").unwrap();
    task.revision = None;
    seed(
        &mut store,
        &[value_record(record.key, record.version + 1, &task).unwrap()],
        "remove-contract",
    );
    assert_eq!(
        life::admit(&mut store, &actor(), plan, NOW)
            .unwrap_err()
            .code,
        "CONTRACT_REQUIRED"
    );
    assert!(store.list("run").unwrap().is_empty());
    assert!(store.list("run_manifest").unwrap().is_empty());
    assert!(store.pending_effects().unwrap().is_empty());
}
#[test]
fn starting_resume_is_illegal_and_lost_engine_reply_reads_back_once() {
    let (_t, mut store, spec) = fixture();
    submit(&mut store, start_input(spec, "start"));
    assert_eq!(
        life::prepare(&store, &actor(), command("resume", "first", 1))
            .unwrap_err()
            .code,
        "INVALID_TRANSITION"
    );
    let fake = Fake::default();
    fake.lost.set(true);
    drive(&mut store, "first", "start", &fake);
    assert_eq!(current(&store, "first").1.lifecycle, State::Starting);
    let version = current(&store, "first").0;
    drive(&mut store, "first", "start", &fake);
    assert_eq!(fake.calls.get(), 1);
    assert_eq!(current(&store, "first").1.lifecycle, State::Running);
    assert!(current(&store, "first").0 > version);
    let (v, _) = current(&store, "first");
    submit(&mut store, command("pause", "first", v));
    drive(&mut store, "first", "pause", &fake);
    let (v, run) = current(&store, "first");
    assert_eq!(run.lifecycle, State::Paused);
    submit(&mut store, command("resume", "first", v));
    drive(&mut store, "first", "resume", &fake);
    let (v, run) = current(&store, "first");
    assert_eq!(run.lifecycle, State::Running);
    submit(&mut store, command("cancel", "first", v));
    assert_eq!(current(&store, "first").1.lifecycle, State::Cancelling);
    drive(&mut store, "first", "stop", &fake);
    assert_eq!(current(&store, "first").1.lifecycle, State::Cancelled);
}
#[test]
fn transitional_timeout_only_marks_attention_and_retains_task_occupancy() {
    for state in [State::Starting, State::Pausing, State::Cancelling] {
        let (_t, mut store, mut spec) = fixture();
        bind_task(&mut store, &mut spec);
        spec.policy.transition_timeout_ms = 1234;
        submit(&mut store, start_input(spec, "start"));
        let (r, mut run) = life::load(&store, "P", "first").unwrap();
        run.lifecycle = state;
        seed(
            &mut store,
            &[value_record(r.key, r.version + 1, &run).unwrap()],
            "phase",
        );
        assert_eq!(engine::mark_timeouts(&mut store, NOW + 1233).unwrap(), 0);
        assert_eq!(engine::mark_timeouts(&mut store, NOW + 1234).unwrap(), 1);
        let (_, run) = current(&store, "first");
        assert_eq!(run.lifecycle, state);
        assert!(run.attention.is_some());
        assert!(run.authorization_valid);
        assert!(
            task::task(&store, "P", "T")
                .unwrap()
                .1
                .run_occupancy
                .is_some()
        );
        assert_eq!(engine::mark_timeouts(&mut store, NOW + 1235).unwrap(), 0);
    }
}
#[test]
fn replacement_rolls_back_old_revoke_new_run_slot_and_outbox_together() {
    let (_t, mut store, mut spec) = fixture();
    bind_task(&mut store, &mut spec);
    submit(&mut store, start_input(spec.clone(), "start"));
    let fake = Fake::default();
    drive(&mut store, "first", "start", &fake);
    let (version, old) = current(&store, "first");
    spec.run_id = "second".into();
    let input = Input {
        key: "replace".into(),
        action: Action::Replace {
            project_id: "P".into(),
            run_id: "first".into(),
            version,
            spec: Box::new(spec),
        },
    };
    let plan = life::prepare(&store, &actor(), input.clone()).unwrap();
    let blocked = plan
        .effects
        .iter()
        .find(|e| e.operation == "run.engine.start")
        .unwrap()
        .clone();
    let mut clash = blocked.clone();
    clash.intent_id = "injected-outbox-conflict".into();
    clash.owner = p("fixture", "clash");
    let command = Command {
        command_id: "inject-conflict".into(),
        idempotency_key: "inject-conflict".into(),
        actor: actor().0,
        target: p("fixture", "clash").key,
        expected: Expected::Absent,
        binding: c("module", "fixture"),
        input_digest: Command::digest_input("fixture", &json!({})).unwrap(),
        operation: "fixture".into(),
        input: json!({}),
    };
    store
        .submit(store.generation(), &actor(), &command, None, |tx| {
            tx.enqueue_effect(&clash)?;
            Ok(json!({}))
        })
        .unwrap();
    let before = task::task(&store, "P", "T").unwrap().1.run_occupancy;
    let error = life::admit(&mut store, &actor(), plan, NOW + 200).unwrap_err();
    assert_eq!(
        task::task(&store, "P", "T").unwrap().1.run_occupancy,
        before,
        "failed replacement cleared occupancy"
    );
    assert_eq!(current(&store, "first").1.lifecycle, State::Running);
    assert!(current(&store, "first").1.authorization_valid);
    assert!(life::load(&store, "P", "second").is_err());
    assert!(
        store
            .get(&p("run_manifest", "second").key)
            .unwrap()
            .is_none()
    );
    assert_eq!(error.code, "EFFECT_CONFLICT");
    assert!(!store.has_effect(&blocked.intent_id).unwrap());
    let clear = Command {
        command_id: "clear-conflict".into(),
        idempotency_key: "clear-conflict".into(),
        ..command
    };
    store
        .submit(store.generation(), &actor(), &clear, None, |tx| {
            tx.cancel_pending_effect("injected-outbox-conflict")?;
            Ok(json!({}))
        })
        .unwrap();
    submit(&mut store, input);
    assert_eq!(current(&store, "first").1.lifecycle, State::Replaced);
    assert_eq!(
        task::task(&store, "P", "T")
            .unwrap()
            .1
            .run_occupancy
            .unwrap()["run_id"],
        "second"
    );
    assert_eq!(
        life::manifest(&store, &actor(), "P", "first")
            .unwrap()
            .1
            .run_id,
        old.id
    );
}
#[test]
fn cancellation_does_not_release_unknown_writes_or_revoke_another_run() {
    let (_t, mut store, mut spec) = fixture();
    bind_task(&mut store, &mut spec);
    submit(&mut store, start_input(spec.clone(), "start"));
    spec.run_id = "other".into();
    spec.task_revisions.clear();
    submit(&mut store, start_input(spec, "other-start"));
    let (_, other_run) = current(&store, "other");
    let fake = Fake::default();
    drive(&mut store, "first", "start", &fake);
    let (version, run) = current(&store, "first");
    let owner = p("run_attempt", "attempt");
    let a = life::Activity {
        run_id: "first".into(),
        manifest: run.manifest.clone(),
        owner: owner.clone(),
        authorization_valid: true,
        can_write: true,
        isolation_proof: None,
    };
    let other = life::Activity {
        run_id: "other".into(),
        manifest: other_run.manifest,
        owner: p("run_attempt", "other-attempt"),
        ..a.clone()
    };
    seed(
        &mut store,
        &[
            value_record(owner.key.clone(), 1, &json!({"run_id":"first"})).unwrap(),
            value_record(
                p("run_attempt", "other-attempt").key,
                1,
                &json!({"run_id":"other"}),
            )
            .unwrap(),
            value_record(p("run_activity", "attempt").key, 1, &a).unwrap(),
            value_record(p("run_activity", "other").key, 1, &other).unwrap(),
        ],
        "activities",
    );
    submit(&mut store, command("cancel", "first", version));
    assert!(!current(&store, "first").1.authorization_valid);
    drive(&mut store, "first", "stop", &fake);
    assert_eq!(current(&store, "first").1.lifecycle, State::Cancelling);
    assert!(
        task::task(&store, "P", "T")
            .unwrap()
            .1
            .run_occupancy
            .is_some()
    );
    let unchanged: life::Activity =
        decode(&store.get(&p("run_activity", "other").key).unwrap().unwrap()).unwrap();
    assert!(unchanged.authorization_valid);
    assert!(current(&store, "other").1.authorization_valid);
    assert!(
        store
            .pending_effects()
            .unwrap()
            .iter()
            .any(|id| store.effect(id).unwrap().0.target == "other")
    );
    let revoked: life::Activity = decode(
        &store
            .get(&p("run_activity", "attempt").key)
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert!(!revoked.authorization_valid);
    assert!(
        !engine::settle_revocation(
            &mut store,
            &engine::actor_for(&run),
            "P",
            "first",
            NOW + 300
        )
        .unwrap()
    );
    let proof = value_record(
        p("run_isolation", "proof").key,
        1,
        &life::IsolationProof {
            manifest: run.manifest.clone(),
            owner,
            confined: true,
            silent: false,
        },
    )
    .unwrap();
    let (record, mut a) = {
        let r = store
            .get(&p("run_activity", "attempt").key)
            .unwrap()
            .unwrap();
        let a: life::Activity = decode(&r).unwrap();
        (r, a)
    };
    a.isolation_proof = Some(reference(&proof));
    seed(
        &mut store,
        &[
            proof,
            value_record(record.key, record.version + 1, &a).unwrap(),
        ],
        "isolation",
    );
    assert!(
        engine::settle_revocation(
            &mut store,
            &engine::actor_for(&run),
            "P",
            "first",
            NOW + 400
        )
        .unwrap()
    );
    assert_eq!(current(&store, "first").1.lifecycle, State::Cancelled);
    assert!(
        task::task(&store, "P", "T")
            .unwrap()
            .1
            .run_occupancy
            .is_none()
    );
}
fn gate_picks(store: &Store, spec: &mut Start) {
    let profession = store.list("profession").unwrap().remove(0);
    let profile: Reference =
        decode(&store.get(&c("worker_profile", "one").key).unwrap().unwrap()).unwrap();
    spec.workflow_id = "gate".into();
    spec.picks = ["a", "b"]
        .iter()
        .map(|seat| life::Pick {
            checkpoint: "review".into(),
            seat: (*seat).into(),
            profession: reference(&profession),
            agency: c("agency_binding", "A"),
            worker_profiles: vec![profile.clone()],
            permissions: vec!["context.read".into()],
            max_context_bytes: 4096,
            frozen_bundle: None,
        })
        .collect();
}
fn bundle(store: &mut Store, id: &str) -> Reference {
    let m = context::read_manifest(store, &actor(), "P", "root")
        .unwrap()
        .unwrap();
    let b = Sealed::new(agency_proto::context::Bundle {
        id: id.into(),
        manifest: FrozenRef {
            id: "root".into(),
            revision: m.digest.clone(),
            digest: m.digest,
        },
        consumer: agency_proto::Owner {
            project: "P".into(),
            kind: agency_proto::OwnerKind::RunAttempt,
            id: id.into(),
            generation: 1,
        },
        entries: vec![agency_proto::context::Entry {
            source: frozen("code"),
            description: "same exact content".into(),
            required: true,
            offline_required: true,
            delivery: agency_proto::context::Delivery::Inline {
                bytes: b"code".to_vec(),
            },
            bytes_digest: sha("code"),
        }],
        renderer: frozen("renderer"),
        tokenizer: frozen("tokenizer"),
        redaction: m.document.redaction,
        compression: vec![],
        candidate_tokens: None,
        selected_tokens: None,
        delivered_tokens: None,
        permission_digest: sha("permissions"),
        budget: 4096,
        retention: "ephemeral".into(),
    })
    .unwrap();
    let record = material_record(store, "context_bundle", id, &b, &format!("bundle-{id}"));
    let r = reference(&record);
    seed(store, &[record], &format!("bundle-{id}"));
    r
}
#[test]
fn candidate_policy_diversity_and_full_range_quorum_use_frozen_content() {
    let (_t, mut store, mut spec) = fixture();
    approve(&mut store, "gate", workflow(true));
    gate_picks(&store, &mut spec);
    let plan = life::prepare(&store, &actor(), start_input(spec.clone(), "unknown")).unwrap();
    assert!(!plan.result["warnings"].as_array().unwrap().is_empty());
    let a = bundle(&mut store, "a");
    let b = bundle(&mut store, "b");
    spec.picks[0].frozen_bundle = Some(a);
    spec.picks[1].frozen_bundle = Some(b);
    assert_eq!(
        life::prepare(&store, &actor(), start_input(spec.clone(), "same-keys"))
            .unwrap_err()
            .code,
        "QUORUM_UNREACHABLE"
    );
    let two: Reference =
        decode(&store.get(&c("worker_profile", "two").key).unwrap().unwrap()).unwrap();
    spec.picks[1].worker_profiles.push(two.clone());
    assert!(
        life::prepare(&store, &actor(), start_input(spec.clone(), "alternate")).is_ok(),
        "legal different candidate must preserve reachable quorum"
    );
    let mut diverse = workflow(true);
    if let run::Operation::External { gate: Some(g), .. } = &mut diverse.graph.nodes[1].operation {
        g.distinct_worker_profiles = true;
    }
    approve(&mut store, "diverse", diverse);
    spec.workflow_id = "diverse".into();
    assert_eq!(
        life::prepare(&store, &actor(), start_input(spec.clone(), "same-initial"))
            .unwrap_err()
            .code,
        "SEAT_DIVERSITY_REQUIRED"
    );
    spec.picks[1].worker_profiles = vec![two];
    assert!(
        life::prepare(
            &store,
            &actor(),
            start_input(spec.clone(), "different-initial")
        )
        .is_ok()
    );
    let mut project = store.get(&p("project", "P").key).unwrap().unwrap();
    if let RecordData::Project { settings, .. } = &mut project.data {
        settings.selection_policy = json!({"allowed_agencies":["another"]});
    }
    project.version += 1;
    spec.project_version = project.version;
    seed(&mut store, &[project], "policy-denied");
    assert_eq!(
        life::prepare(&store, &actor(), start_input(spec, "denied"))
            .unwrap_err()
            .code,
        "SELECTION_POLICY_DENIED"
    );
}
#[test]
fn blockers_are_confirmable_preview_inputs_not_automatic_rejection() {
    let (_t, mut store, mut spec) = fixture();
    bind_task(&mut store, &mut spec);
    let key = target(Scope::Repo("R".into()), "task_snapshot", "issues").key;
    let mut record = store.get(&key).unwrap().unwrap();
    let mut snap: task::Snapshot = decode(&record).unwrap();
    snap.cards[0].dependencies.parent = Some(json!({"state":"open"}));
    snap.cards[0].dependencies.blocked_by = vec![
        json!({"id":2,"state":"open"}),
        json!({"id":3,"state":"closed"}),
    ];
    record = value_record(key, record.version + 1, &snap).unwrap();
    seed(&mut store, &[record], "blocked");
    let plan = life::prepare(&store, &actor(), start_input(spec.clone(), "blocked-start")).unwrap();
    assert_eq!(
        plan.result["blockers_requiring_confirmation"],
        json!([{"id":2,"state":"open"}])
    );
    life::admit(&mut store, &actor(), plan, NOW).unwrap();
    let (_, run) = current(&store, "first");
    assert_eq!(run.lifecycle, State::Starting);
}
#[test]
fn completion_needs_control_predicates_and_keeps_completion_pending_slot() {
    let (_t, mut store, mut spec) = fixture();
    bind_task(&mut store, &mut spec);
    submit(&mut store, start_input(spec, "start"));
    let fake = Fake::default();
    drive(&mut store, "first", "start", &fake);
    let (version, run) = current(&store, "first");
    let internal = engine::actor_for(&run);
    assert_eq!(
        finish(
            &mut store,
            &actor(),
            version,
            State::Completed,
            None,
            NOW + 200
        )
        .unwrap_err()
        .code,
        "PERMISSION_DENIED"
    );
    assert_eq!(
        finish(
            &mut store,
            &internal,
            version,
            State::Completed,
            None,
            NOW + 200
        )
        .unwrap_err()
        .code,
        "RUN_COMPLETION_UNPROVEN"
    );
    let binding = store
        .get(&p("run_engine_binding", "first").key)
        .unwrap()
        .unwrap();
    let mut proof = life::CompletionProof {
        manifest: run.manifest.clone(),
        engine_binding: reference(&binding),
        all_required_satisfied: Some(true),
        all_attempts_terminal_or_revoked: Some(true),
        no_unresolved_output_effects: Some(true),
        references_match: None,
    };
    let record = value_record(p("run_completion", "unknown").key, 1, &proof).unwrap();
    seed(&mut store, std::slice::from_ref(&record), "unknown-proof");
    assert_eq!(
        finish(
            &mut store,
            &internal,
            version,
            State::Completed,
            Some(&reference(&record)),
            NOW + 200
        )
        .unwrap_err()
        .code,
        "RUN_COMPLETION_UNPROVEN"
    );
    proof.references_match = Some(true);
    let record = value_record(p("run_completion", "verified").key, 1, &proof).unwrap();
    seed(&mut store, std::slice::from_ref(&record), "verified-proof");
    finish(
        &mut store,
        &internal,
        version,
        State::Completed,
        Some(&reference(&record)),
        NOW + 300,
    )
    .unwrap();
    assert_eq!(current(&store, "first").1.lifecycle, State::Completed);
    assert_eq!(
        task::task(&store, "P", "T")
            .unwrap()
            .1
            .run_occupancy
            .unwrap()["state"],
        "completion_pending"
    );
}
#[test]
fn unbound_run_and_other_display_references_do_not_own_or_stop_task() {
    let (_t, mut store, spec) = fixture();
    let mut bound = spec.clone();
    bind_task(&mut store, &mut bound);
    submit(&mut store, start_input(spec, "unbound"));
    let (_, task) = task::task(&store, "P", "T").unwrap();
    assert!(task::active_runs(&store, &task).unwrap().is_empty());
    seed(
        &mut store,
        &[value_record(
            p("task_run_link", "display").key,
            1,
            &json!({"run_id":"first","task_id":"T"}),
        )
        .unwrap()],
        "display-link",
    );
    let (record, _) = task::task(&store, "P", "T").unwrap();
    let input:task::Input=serde_json::from_value(json!({"key":"cancel-reference","action":{"kind":"cancel","project_id":"P","task_id":"T","version":record.version}})).unwrap();
    let plan = task::prepare_as(&store, &actor(), input).unwrap();
    task::admit(&mut store, &actor(), plan).unwrap();
    assert_eq!(current(&store, "first").1.lifecycle, State::Starting);
    let before = life::manifest(&store, &actor(), "P", "first")
        .unwrap()
        .0
        .revision_digest;
    let (v, run) = current(&store, "first");
    let _ = run;
    submit(
        &mut store,
        Input {
            key: "place".into(),
            action: Action::Place {
                project_id: "P".into(),
                run_id: "first".into(),
                version: v,
                destination: p("room_binding", "delivery"),
            },
        },
    );
    assert_eq!(
        life::manifest(&store, &actor(), "P", "first")
            .unwrap()
            .0
            .revision_digest,
        before
    );
    assert_eq!(store.list("run_placement").unwrap().len(), 1);
}
#[path = "lifecycle_cli.rs"]
mod cli;

#[test]
fn every_nonterminal_phase_can_fail_or_replace_and_terminal_runs_cannot_revive() {
    for state in [
        State::Starting,
        State::Running,
        State::Pausing,
        State::Paused,
        State::Cancelling,
    ] {
        for replacing in [false, true] {
            let (_t, mut store, mut spec) = fixture();
            bind_task(&mut store, &mut spec);
            submit(&mut store, start_input(spec.clone(), "start"));
            let (record, mut run) = life::load(&store, "P", "first").unwrap();
            run.lifecycle = state;
            seed(
                &mut store,
                &[value_record(record.key, record.version + 1, &run).unwrap()],
                "set-phase",
            );
            let (version, run) = current(&store, "first");
            if replacing {
                spec.run_id = "next".into();
                submit(
                    &mut store,
                    Input {
                        key: "replace-phase".into(),
                        action: Action::Replace {
                            project_id: "P".into(),
                            run_id: "first".into(),
                            version,
                            spec: Box::new(spec),
                        },
                    },
                );
                assert_eq!(current(&store, "first").1.lifecycle, State::Replaced);
                assert_eq!(
                    task::task(&store, "P", "T")
                        .unwrap()
                        .1
                        .run_occupancy
                        .unwrap()["run_id"],
                    "next"
                );
            } else {
                finish(
                    &mut store,
                    &engine::actor_for(&run),
                    version,
                    State::Failed,
                    None,
                    NOW + 200,
                )
                .unwrap();
                assert_eq!(current(&store, "first").1.lifecycle, State::Failed);
                assert!(
                    task::task(&store, "P", "T")
                        .unwrap()
                        .1
                        .run_occupancy
                        .is_none()
                );
            }
        }
    }
    for state in [
        State::Completed,
        State::Failed,
        State::Cancelled,
        State::Replaced,
    ] {
        let (_t, mut store, mut spec) = fixture();
        submit(&mut store, start_input(spec.clone(), "start"));
        let (record, mut run) = life::load(&store, "P", "first").unwrap();
        run.lifecycle = state;
        run.authorization_valid = false;
        seed(
            &mut store,
            &[value_record(record.key, record.version + 1, &run).unwrap()],
            "terminal",
        );
        let (version, _) = current(&store, "first");
        for op in ["pause", "resume", "cancel"] {
            assert_eq!(
                life::prepare(&store, &actor(), command(op, "first", version))
                    .unwrap_err()
                    .code,
                "INVALID_TRANSITION",
                "{state:?}/{op}"
            );
        }
        spec.run_id = "cannot-revive".into();
        let input = Input {
            key: "replace-terminal".into(),
            action: Action::Replace {
                project_id: "P".into(),
                run_id: "first".into(),
                version,
                spec: Box::new(spec),
            },
        };
        assert_eq!(
            life::prepare(&store, &actor(), input).unwrap_err().code,
            "INVALID_TRANSITION"
        );
    }
}
#[test]
fn archived_closed_pending_and_stale_task_inputs_fail_closed() {
    for case in [
        "archived",
        "closed",
        "pending",
        "stale-revision",
        "source-gap",
        "occupied",
    ] {
        let (_t, mut store, mut spec) = fixture();
        bind_task(&mut store, &mut spec);
        let expected = match case {
            "archived" => {
                let mut r = store.get(&p("project", "P").key).unwrap().unwrap();
                if let RecordData::Project { archived, .. } = &mut r.data {
                    *archived = true;
                }
                r.version += 1;
                spec.project_version = r.version;
                seed(&mut store, &[r], case);
                "PROJECT_READ_ONLY"
            }
            "source-gap" => {
                let k = target(Scope::Repo("R".into()), "task_snapshot", "issues").key;
                let r = store.get(&k).unwrap().unwrap();
                let mut snap: task::Snapshot = decode(&r).unwrap();
                snap.complete = false;
                seed(
                    &mut store,
                    &[value_record(k, r.version + 1, &snap).unwrap()],
                    case,
                );
                "READBACK_REQUIRED"
            }
            _ => {
                let (r, mut t) = task::task(&store, "P", "T").unwrap();
                match case {
                    "closed" => t.lifecycle = "completed".into(),
                    "pending" => t.pending_contract = Some(p("task_snapshot", "pending")),
                    "stale-revision" => t.revision.as_mut().unwrap().number = 2,
                    "occupied" => {
                        t.run_occupancy = Some(
                            json!({"run_id":"old","state":"completion_pending","manifest":p("run_manifest","old")}),
                        )
                    }
                    _ => unreachable!(),
                };
                seed(
                    &mut store,
                    &[value_record(r.key, r.version + 1, &t).unwrap()],
                    case,
                );
                match case {
                    "closed" => "TASK_NOT_OPEN",
                    "pending" => "CONTRACT_DIVERGED",
                    "stale-revision" => "VERSION_CONFLICT",
                    "occupied" => "RUN_ACTIVE",
                    _ => unreachable!(),
                }
            }
        };
        assert_eq!(
            life::prepare(&store, &actor(), start_input(spec, case))
                .unwrap_err()
                .code,
            expected,
            "{case}"
        );
        assert!(store.list("run").unwrap().is_empty());
    }
}
#[test]
fn default_timeout_and_blocker_snapshot_changes_are_rechecked_at_admission() {
    let (_t, mut store, mut spec) = fixture();
    bind_task(&mut store, &mut spec);
    let mut value = serde_json::to_value(&spec).unwrap();
    value["policy"]
        .as_object_mut()
        .unwrap()
        .remove("transition_timeout_ms");
    let spec: Start = serde_json::from_value(value).unwrap();
    assert_eq!(spec.policy.transition_timeout_ms, 900_000);
    let plan = life::prepare(&store, &actor(), start_input(spec, "snapshot-change")).unwrap();
    let k = target(Scope::Repo("R".into()), "task_snapshot", "issues").key;
    let record = store.get(&k).unwrap().unwrap();
    let mut snapshot: task::Snapshot = decode(&record).unwrap();
    snapshot.cards[0].dependencies.blocked_by = vec![json!({"state":"open","id":"new"})];
    seed(
        &mut store,
        &[value_record(k, record.version + 1, &snapshot).unwrap()],
        "new-blocker",
    );
    assert_eq!(
        life::admit(&mut store, &actor(), plan, NOW)
            .unwrap_err()
            .code,
        "VERSION_CONFLICT"
    );
    assert!(
        task::task(&store, "P", "T")
            .unwrap()
            .1
            .run_occupancy
            .is_none()
    );
}
#[test]
fn unregistered_attempt_is_unknown_not_proof_of_an_empty_runtime() {
    let (_t, mut store, mut spec) = fixture();
    bind_task(&mut store, &mut spec);
    submit(&mut store, start_input(spec.clone(), "start"));
    let (version, _) = current(&store, "first");
    seed(
        &mut store,
        &[value_record(
            p("run_attempt", "unregistered").key,
            1,
            &json!({"run_id":"first"}),
        )
        .unwrap()],
        "unregistered",
    );
    spec.run_id = "unsafe-replacement".into();
    assert_eq!(
        life::prepare(
            &store,
            &actor(),
            Input {
                key: "replace-unknown".into(),
                action: Action::Replace {
                    project_id: "P".into(),
                    run_id: "first".into(),
                    version,
                    spec: Box::new(spec)
                }
            }
        )
        .unwrap_err()
        .code,
        "RUN_ISOLATION_UNPROVEN"
    );
    submit(&mut store, command("cancel", "first", version));
    let (_, run) = current(&store, "first");
    assert_eq!(run.lifecycle, State::Starting);
    assert!(run.attention.is_some());
    assert!(!run.authorization_valid);
    assert!(
        task::task(&store, "P", "T")
            .unwrap()
            .1
            .run_occupancy
            .is_some()
    );
}
#[test]
fn frozen_content_contract_ignores_consumer_and_rejects_changed_delivery() {
    let (_t, mut store, mut spec) = fixture();
    approve(&mut store, "gate", workflow(true));
    gate_picks(&store, &mut spec);
    spec.picks[0].frozen_bundle = Some(bundle(&mut store, "a"));
    spec.picks[1].frozen_bundle = Some(bundle(&mut store, "b"));
    let two: Reference =
        decode(&store.get(&c("worker_profile", "two").key).unwrap().unwrap()).unwrap();
    spec.picks[1].worker_profiles = vec![two];
    submit(&mut store, start_input(spec, "frozen-start"));
    let (_, m) = life::manifest(&store, &actor(), "P", "first").unwrap();
    let selected = &m.selected[0];
    let original = context::read_bundle(&store, &actor(), "P", "a")
        .unwrap()
        .unwrap();
    let mut changed = original.document.clone();
    changed.consumer.id = "new-attempt".into();
    changed.retention = "different retention".into();
    changed.candidate_tokens = Some(30);
    life::validate_delivery_contract(&m, selected, &Sealed::new(changed.clone()).unwrap()).unwrap();
    changed.entries[0].delivery = agency_proto::context::Delivery::Inline {
        bytes: b"changed".to_vec(),
    };
    changed.entries[0].bytes_digest = sha("changed");
    assert_eq!(
        life::validate_delivery_contract(&m, selected, &Sealed::new(changed).unwrap())
            .unwrap_err()
            .code,
        "BUNDLE_CONTRACT_CHANGED"
    );
}
#[test]
fn dispatch_registration_checks_frozen_spec_and_cannot_cross_revocation() {
    let (_t, mut store, mut start) = fixture();
    approve(&mut store, "gate", workflow(true));
    gate_picks(&store, &mut start);
    submit(&mut store, start_input(start, "start"));
    let fake = Fake::default();
    drive(&mut store, "first", "start", &fake);
    let (version, run) = current(&store, "first");
    let internal = engine::actor_for(&run);
    let (_, m) = life::manifest(&store, &actor(), "P", "first").unwrap();
    let selected = &m.selected[0];
    bundle(&mut store, "planned");
    let mut b = context::read_bundle(&store, &actor(), "P", "planned")
        .unwrap()
        .unwrap()
        .document;
    b.consumer.id = "attempt".into();
    let b = Sealed::new(b).unwrap();
    let selection_record = store.get(&selected.selection.key).unwrap().unwrap();
    let profession: agency_proto::Profession =
        decode(&store.get(&selected.profession.key).unwrap().unwrap()).unwrap();
    let profile = store
        .get(&selected.worker_profiles[0].key)
        .unwrap()
        .unwrap();
    let binding = store.get(&selected.agency.key).unwrap().unwrap();
    let project = store.get(&p("project", "P").key).unwrap().unwrap();
    let spec:agency_proto::ExecutionSpec=serde_json::from_value(json!({"owner":b.document.consumer,"project":participant::frozen(&project),"selection":participant::frozen(&selection_record),"selection_policy_digest":m.selection_policy_digest,"profession":profession,"profile":participant::frozen(&profile),"manifest":b.document.manifest,"bundle":{"id":b.document.id,"revision":b.digest,"digest":b.digest},"binding":participant::frozen(&binding),"required_capabilities":Capabilities::default(),"input_policy":"no_input","permission_digest":b.document.permission_digest,"permissions":selected.permissions,"budget":b.document.budget,"deadline_ms":DEADLINE,"repo":null,"base":null,"delivery_scope":[],"write_lease":null,"review_publish_policy":null,"idempotency_key":"attempt"})).unwrap();
    let spec = Sealed::new(spec).unwrap();
    let owner = value_record(
        p("run_attempt", "attempt").key,
        1,
        &json!({"run_id":"first","spec":spec}),
    )
    .unwrap();
    let activity = life::Activity {
        run_id: "first".into(),
        manifest: run.manifest.clone(),
        owner: reference(&owner),
        authorization_valid: true,
        can_write: false,
        isolation_proof: None,
    };
    let input = json!({"activity":activity});
    let registration = Command {
        command_id: "register-attempt".into(),
        idempotency_key: "register-attempt".into(),
        actor: internal.0.clone(),
        target: owner.key.clone(),
        expected: Expected::Absent,
        binding: run.manifest.clone(),
        operation: "fixture.register_attempt".into(),
        input_digest: Command::digest_input("fixture.register_attempt", &input).unwrap(),
        input,
    };
    store
        .submit(store.generation(), &internal, &registration, None, |tx| {
            tx.put(&owner)?;
            life::authorize_activity_in(tx, &internal, &activity, &spec, &b)?;
            Ok(json!({}))
        })
        .unwrap();
    assert!(
        store
            .get(&p("run_activity", "attempt").key)
            .unwrap()
            .is_some()
    );
    submit(&mut store, command("cancel", "first", version));
    let mut late = activity.clone();
    late.owner = p("run_attempt", "late");
    let owner = value_record(late.owner.key.clone(), 1, &json!({"spec":spec})).unwrap();
    let cmd = Command {
        command_id: "register-late".into(),
        idempotency_key: "register-late".into(),
        target: owner.key.clone(),
        ..registration
    };
    let error = store
        .submit(store.generation(), &internal, &cmd, None, |tx| {
            tx.put(&owner)?;
            life::authorize_activity_in(tx, &internal, &late, &spec, &b)?;
            Ok(json!({}))
        })
        .unwrap_err();
    assert_eq!(error.code, "RUN_REVOKED");
    assert!(store.get(&owner.key).unwrap().is_none());
}

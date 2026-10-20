use agency_proto::{
    Sealed,
    context::{Bundle, Delivery},
};
use chat::{key, value_record};
use foundation::{bytes_sha256, canonical_json, canonical_json_sha256};
use run::{
    readback::{self, InvocationEvidence, Readback, Readbacks},
    *,
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use store::{
    Actor, ActorSource, Command, Expected, Record, RecordData, Reference, Scope, Store,
    TrustedActor, Version,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "hctl-workflow-{}-{}",
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
        ],
        authority: None,
    })
}
fn target(kind: &str, id: &str) -> Reference {
    Reference {
        key: key(Scope::Project("P".into()), kind, id),
        version: Version::State(1),
    }
}
fn sha(value: &str) -> String {
    bytes_sha256(value.as_bytes())
}
fn frozen(id: &str) -> Value {
    json!({"id":id,"revision":"1","digest":sha(id)})
}
fn workflow() -> Workflow {
    serde_json::from_value(json!({"schema":"hctl2.workflow.v1","profile":"hctl2.passive.v1","graph":{"nodes":[external("build")],"edges":[]}})).unwrap()
}
fn external(id: &str) -> Value {
    json!({"id":id,"outputs":{"code":"change_set_revision"},"operation":{"kind":"external","criterion":{"kind":"mechanical","output":"code","fact":{"kind":"ci","check":"test"}},"seats":[],"gate":null}})
}
fn noop(id: &str) -> Value {
    json!({"id":id,"operation":{"kind":"noop"}})
}
fn edge(from: &str, to: &str) -> Value {
    json!({"from":from,"to":to})
}
fn with_graph(value: Value) -> Workflow {
    let mut w = workflow();
    w.graph = serde_json::from_value(value).unwrap();
    w
}
fn snapshot() -> Snapshot {
    serde_json::from_value(json!({"decided":[{"id":"D1","text":"change real code","delivery_obligation":true}],"unresolved":[],"out_of_scope":[]})).unwrap()
}
fn provenance() -> Provenance {
    Provenance {
        snapshot_source: SnapshotSource::DirectText {},
        source_chain: vec![],
        production: Production::None {},
    }
}
#[test]
fn schema_profile_and_side_effects_are_closed() {
    let original = serde_json::to_value(workflow()).unwrap();
    for field in [
        "command",
        "script",
        "http",
        "action",
        "agent",
        "harness",
        "subdag",
        "repeat_policy",
        "run",
    ] {
        let mut value = original.clone();
        value["graph"]["nodes"][0]["operation"][field] = json!("forbidden");
        assert!(
            serde_json::from_value::<Workflow>(value).is_err(),
            "{field}"
        );
        let mut value = original.clone();
        value["graph"]["nodes"][0]["operation"] = json!({"kind":field});
        assert!(
            serde_json::from_value::<Workflow>(value).is_err(),
            "kind {field}"
        );
    }
    for (field, value) in [("schema", "old"), ("profile", "arbitrary_dagu")] {
        let mut w = original.clone();
        w[field] = json!(value);
        assert!(compile(&serde_json::from_value(w).unwrap()).is_err());
    }
    for field in ["messages", "discussion", "extra"] {
        let mut s = serde_json::to_value(snapshot()).unwrap();
        s[field] = json!([]);
        assert!(serde_json::from_value::<Snapshot>(s).is_err());
    }
}
#[test]
fn empty_tagged_variants_reject_unknown_fields() {
    for kind in ["fork", "join", "noop"] {
        let clean = json!({"kind":kind});
        assert!(serde_json::from_value::<Operation>(clean.clone()).is_ok());
        for field in ["command", "script", "http", "seconds"] {
            let mut bad = clean.clone();
            bad[field] = json!("unexpected");
            assert!(
                serde_json::from_value::<Operation>(bad).is_err(),
                "{kind}.{field}"
            );
        }
    }
    macro_rules! closed {
        ($ty:ty, $kind:literal) => {{
            let clean = json!({"kind":$kind});
            let parsed: $ty = serde_json::from_value(clean.clone()).unwrap();
            assert_eq!(serde_json::to_value(parsed).unwrap(), clean);
            let mut bad = clean;
            bad["unexpected"] = json!(true);
            assert!(serde_json::from_value::<$ty>(bad).is_err(), $kind);
        }};
    }
    closed!(Selection, "all");
    closed!(MechanicalFact, "pr_merged");
    closed!(SnapshotSource, "direct_text");
    closed!(Production, "none");
    closed!(Production, "missing");
}
#[test]
fn static_cycle_is_rejected() {
    let w =
        with_graph(json!({"nodes":[noop("a"),noop("b")],"edges":[edge("a","b"),edge("b","a")]}));
    assert_eq!(compile(&w).unwrap_err().code, "WORKFLOW_CYCLE");
}
#[test]
fn artifact_edges_land_on_declared_future_outputs() {
    let good = json!({"nodes":[external("a"),external("b")],"edges":[{"from":"a","to":"b","artifact":{"output":"code","kind":"change_set_revision"}}]});
    assert!(compile(&with_graph(good.clone())).is_ok());
    for (field, value) in [("output", json!("missing")), ("kind", json!("receipt"))] {
        let mut bad = good.clone();
        bad["edges"][0]["artifact"][field] = value;
        assert_eq!(
            compile(&with_graph(bad)).unwrap_err().code,
            "WORKFLOW_DEPENDENCY"
        );
    }
    let mut missing = good.clone();
    missing["edges"][0]["from"] = json!("absent");
    assert!(compile(&with_graph(missing)).is_err());
    let w = with_graph(json!({"nodes":[external("a"),external("b")],"edges":[edge("a","b")]}));
    assert_eq!(compile(&w).unwrap_err().code, "WORKFLOW_DEPENDENCY");
    let mut w = workflow();
    if let Operation::External { criterion, .. } = &mut w.graph.nodes[0].operation {
        *criterion = None;
    }
    assert_eq!(compile(&w).unwrap_err().code, "WORKFLOW_CRITERION");
}
fn fork_graph() -> Workflow {
    with_graph(
        json!({"nodes":[{"id":"loop","operation":{"kind":"loop","body":{"nodes":[external("work")],"edges":[]},"on_exhausted":"request"}}, {"id":"fork","operation":{"kind":"dynamic_fork","template":external("worker"),"candidates":[{"selection":target("selection","s1"),"responsibility":"review"},{"selection":target("selection","s2"),"responsibility":"review"}],"max_cardinality":2,"budget":4096,"selection":{"kind":"all"},"permission_ceiling":["context.read"]}},noop("end")],"edges":[edge("loop","fork"),edge("fork","end")]}),
    )
}
#[test]
fn bounded_loop_new_checkpoints_and_frozen_fork_compile() {
    let w = fork_graph();
    let compiled = compile(&w).unwrap();
    assert_eq!(compiled.frozen_rework_limits["loop"], 2);
    for round in 0..=2 {
        assert!(
            compiled
                .checkpoints
                .iter()
                .any(|c| c.id == format!("loop__r{round}__work"))
        );
    }
    assert!(!compiled.checkpoints.iter().any(|c| c.id.contains("r3")));
    for round in 0..=2 {
        let checkpoint = compiled
            .checkpoints
            .iter()
            .find(|c| c.id == format!("loop__r{round}__work"))
            .unwrap();
        assert_eq!(
            checkpoint.activation,
            vec![run::compiler::Activation::LoopRound {
                checkpoint: "loop".into(),
                round
            }]
        );
    }
    assert_eq!(
        compiled
            .checkpoints
            .iter()
            .find(|c| c.id == "loop__exit")
            .unwrap()
            .role,
        run::compiler::CheckpointRole::CompositeExit
    );
    let yaml: Value = serde_json::from_str(&compiled.yaml).unwrap();
    for step in yaml["steps"].as_array().unwrap() {
        assert_eq!(step["action"], "human.task");
        assert_eq!(
            step["with"]["form"]["required"].as_array().unwrap().len(),
            4
        );
        for field in [
            "command",
            "script",
            "repeat_policy",
            "run",
            "parallel",
            "http",
        ] {
            assert!(step.get(field).is_none());
        }
    }
    let mut edited = w.clone();
    edited.context = "Only background changed".into();
    assert_eq!(compile(&edited).unwrap(), compiled);
    for field in [
        "candidates",
        "max_cardinality",
        "budget",
        "selection",
        "permission_ceiling",
    ] {
        let mut value = serde_json::to_value(&w).unwrap();
        value["graph"]["nodes"][1]["operation"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            serde_json::from_value::<Workflow>(value).is_err(),
            "missing {field}"
        );
    }
    for (field, value) in [
        ("candidates", json!([])),
        ("max_cardinality", json!(0)),
        ("budget", json!(0)),
        ("permission_ceiling", json!([])),
    ] {
        let mut v = serde_json::to_value(&w).unwrap();
        v["graph"]["nodes"][1]["operation"][field] = value;
        assert!(compile(&serde_json::from_value(v).unwrap()).is_err());
    }
}

#[test]
fn explicit_loop_limit_and_structural_branches_remain_frozen() {
    let mut w = fork_graph();
    if let Operation::Loop { max_reworks, .. } = &mut w.graph.nodes[0].operation {
        *max_reworks = 1;
    }
    let compiled = compile(&w).unwrap();
    assert_eq!(compiled.frozen_rework_limits["loop"], 1);
    assert!(!compiled.checkpoints.iter().any(|c| c.id.contains("__r2__")));
    if let Operation::Loop { max_reworks, .. } = &mut w.graph.nodes[0].operation {
        *max_reworks = u32::MAX - 1;
    }
    assert_eq!(compile(&w).unwrap_err().code, "WORKFLOW_EXPANSION_LIMIT");
    let branches = with_graph(
        json!({"nodes":[{"id":"fork","operation":{"kind":"fork"}},{"id":"switch","operation":{"kind":"switch","cases":["left","right"]}},{"id":"timer","operation":{"kind":"timer_wait","seconds":1}},{"id":"transform","operation":{"kind":"transform","fields":{"value":"input"}}},{"id":"join","operation":{"kind":"join"}}],"edges":[edge("fork","switch"),{"from":"switch","to":"timer","case":"left"},{"from":"switch","to":"transform","case":"right"},edge("timer","join"),edge("transform","join")]}),
    );
    assert_eq!(compile(&branches).unwrap().checkpoints.len(), 5);
}
#[test]
fn frozen_template_can_review_a_parent_graph_output() {
    let mut w = fork_graph();
    w.graph
        .nodes
        .push(serde_json::from_value(external("build")).unwrap());
    let reviewer = gate_workflow(1).graph.nodes.remove(1);
    if let Operation::DynamicFork { template, .. } = &mut w.graph.nodes[1].operation {
        **template = reviewer;
    }
    assert!(compile(&w).is_ok());
}
fn gate_workflow(quorum: u32) -> Workflow {
    let seat = |id: &str| json!({"id":id,"required":true,"profession":"review","evidence":"unmediated","required_skills":[],"review_subject_node":"build","template":"shared"});
    let mut w = workflow();
    w.graph.nodes.push(serde_json::from_value(json!({"id":"gate","outputs":{"verdict":"verdict"},"operation":{"kind":"external","criterion":{"kind":"gate","evidence":"unmediated"},"seats":[seat("a"),seat("b")],"gate":{"quorum":quorum,"distinct_worker_profiles":false}}})).unwrap());
    w.graph.edges.push(serde_json::from_value(json!({"from":"build","to":"gate","artifact":{"output":"code","kind":"change_set_revision"}})).unwrap());
    w
}
#[test]
fn quorum_refuses_only_proven_shortfall_and_warns_about_unknown_keys() {
    assert_eq!(
        compile(&gate_workflow(3)).unwrap_err().code,
        "WORKFLOW_QUORUM"
    );
    let compiled = compile(&gate_workflow(2)).unwrap();
    assert_eq!(compiled.warnings.len(), 1);
    assert_eq!(compiled.warnings[0].missing.len(), 2);
    assert_eq!(compiled.warnings[0].options.len(), 3);
    let mut w = gate_workflow(2);
    if let Operation::External { seats, .. } = &mut w.graph.nodes[1].operation {
        seats[1].id = seats[0].id.clone();
    }
    assert_eq!(compile(&w).unwrap_err().code, "WORKFLOW_QUORUM");
    if let Operation::External { gate, .. } = &mut w.graph.nodes[1].operation {
        gate.as_mut().unwrap().quorum = 1;
    }
    let compiled = compile(&w).unwrap();
    let Operation::External { seats, .. } = &compiled
        .checkpoints
        .iter()
        .find(|c| c.id == "gate")
        .unwrap()
        .node
        .operation
    else {
        panic!("gate");
    };
    assert_eq!(seats.len(), 1);
}
fn find_binary(root: &Path) -> PathBuf {
    find_optional(root).unwrap_or_else(|| panic!("dagu missing under {}", root.display()))
}
fn find_optional(root: &Path) -> Option<PathBuf> {
    for entry in std::fs::read_dir(root).ok()? {
        let path = entry.ok()?.path();
        if path.file_name()? == "dagu" && path.is_file() {
            return Some(path);
        }
        if path.is_dir()
            && let Some(found) = find_optional(&path)
        {
            return Some(found);
        }
    }
    None
}
#[test]
fn pinned_dagu_validates_expanded_passive_yaml_without_a_service() {
    let temp = Temp::new();
    let validator = NativeDagu {
        binary: find_binary(Path::new(&std::env::var("HCTL2_TEST_DAGU_CACHE").unwrap())),
        scratch: temp.0.clone(),
    };
    validator
        .validate(&compile(&fork_graph()).unwrap())
        .unwrap();
    assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 0);
    let relative = Temp(PathBuf::from("buck-out").join(format!(
        "workflow-native-relative-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    NativeDagu {
        binary: validator.binary.clone(),
        scratch: relative.0.clone(),
    }
    .validate(&compile(&workflow()).unwrap())
    .unwrap();
    assert_eq!(std::fs::read_dir(&relative.0).unwrap().count(), 0);
    assert_eq!(
        NativeDagu {
            binary: PathBuf::from("/bin/echo"),
            scratch: temp.0.clone()
        }
        .validate(&compile(&workflow()).unwrap())
        .unwrap_err()
        .code,
        "WORKFLOW_ENGINE_VERSION"
    );
    println!("Dagu 2.15.1 validate: bounded loops/forks passed; no service; scratch cleaned");
}
fn seed(store: &mut Store, records: &[Record], name: &str) {
    let value = json!({"fixture":name});
    let command = Command {
        command_id: name.into(),
        idempotency_key: name.into(),
        actor: actor().0,
        target: key(Scope::Control, "fixture", name),
        expected: Expected::Absent,
        binding: Reference {
            key: key(Scope::Control, "module", "fixture"),
            version: Version::State(1),
        },
        input_digest: Command::digest_input("fixture", &value).unwrap(),
        operation: "fixture".into(),
        input: value,
    };
    store
        .submit(store.generation(), &actor(), &command, None, |tx| {
            for record in records {
                tx.put(record)?;
            }
            Ok(json!({}))
        })
        .unwrap();
}
fn setup() -> (Temp, Store) {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let mut p = value_record(
        key(Scope::Project("P".into()), "project", "P"),
        1,
        &json!({}),
    )
    .unwrap();
    p.data = RecordData::Project {
        repo_id: "repo".into(),
        settings: store::ProjectSettings {
            publish_review_requires_confirmation: true,
            selection_policy: json!({}),
        },
        archived: false,
    };
    let t = task_record("P", "T");
    seed(&mut store, &[p, t], "setup");
    (temp, store)
}
fn task_record(project: &str, id: &str) -> Record {
    let mut t = value_record(
        key(Scope::Project(project.into()), "task", id),
        1,
        &json!({}),
    )
    .unwrap();
    t.data = RecordData::Task {
        entity: None,
        source: Reference {
            key: key(Scope::Repo("repo".into()), "task_source", "source"),
            version: Version::State(1),
        },
    };
    t
}
struct NoEngine;
impl Validator for NoEngine {
    fn validate(&self, _: &Compiled) -> store::Result<()> {
        Ok(())
    }
}
fn register(name: &str, w: Workflow) -> Input {
    Input {
        key: name.into(),
        action: Action::Register {
            project_id: "P".into(),
            project_version: 1,
            workflow_id: "plan".into(),
            supersedes: None,
            task: None,
            workflow: Box::new(w),
            snapshot: Some(snapshot()),
            provenance: Some(Box::new(provenance())),
        },
    }
}
fn submit(store: &mut Store, input: Input) -> Value {
    let plan = prepare(store, input, &actor(), &NoEngine).unwrap();
    admit(store, &actor(), plan, &NoEngine).unwrap()
}
fn action(kind: &str, rev: i64) -> Input {
    serde_json::from_value(json!({"key":format!("{kind}-{rev}"),"action":{"kind":kind,"project_id":"P","project_version":1,"workflow_id":"plan","revision":rev,"readback":null}})).or_else(|_|serde_json::from_value(json!({"key":format!("{kind}-{rev}"),"action":{"kind":kind,"project_id":"P","project_version":1,"workflow_id":"plan","revision":rev}}))).unwrap()
}
#[test]
fn registration_is_atomic_idempotent_append_only_and_does_not_start() {
    let (_temp, mut store) = setup();
    let mut input = register("first", workflow());
    if let Action::Register { task, .. } = &mut input.action {
        *task = Some(target("task", "T"));
    }
    let plan = prepare(&store, input.clone(), &actor(), &NoEngine).unwrap();
    assert!(store.list("workflow_task_link").unwrap().is_empty());
    let first = admit(&mut store, &actor(), plan, &NoEngine).unwrap();
    assert_eq!(first, submit(&mut store, input.clone()));
    assert_eq!(store.list("workflow_task_link").unwrap().len(), 1);
    assert_eq!(store.list("workflow_revision").unwrap().len(), 1);
    assert_eq!(
        require_approved(&store, "P", "plan", 1).unwrap_err().code,
        "WORKFLOW_NOT_APPROVED"
    );
    assert!(store.list("run").unwrap().is_empty());
    let mut q = store.get(&target("project", "P").key).unwrap().unwrap();
    q.key = key(Scope::Project("Q".into()), "project", "Q");
    let q_task = task_record("Q", "T");
    seed(&mut store, &[q, q_task.clone()], "other-project-task");
    let mut other = register("wrong-project", workflow());
    if let Action::Register {
        task, workflow_id, ..
    } = &mut other.action
    {
        *workflow_id = "foreign".into();
        *task = Some(chat::reference(&q_task));
    }
    let error = prepare(&store, other, &actor(), &NoEngine).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("reference belongs to another Project"),
        "{error}"
    );
    assert_eq!(store.list("workflow_task_link").unwrap().len(), 1);
    let mut stale = register("failed-admission", workflow());
    if let Action::Register { workflow_id, .. } = &mut stale.action {
        *workflow_id = "later".into();
    }
    let preview = prepare(&store, stale, &actor(), &NoEngine).unwrap();
    let mut project = store.get(&target("project", "P").key).unwrap().unwrap();
    project.version = 2;
    seed(&mut store, &[project], "change-project");
    assert!(admit(&mut store, &actor(), preview, &NoEngine).is_err());
    assert_eq!(store.list("workflow_revision").unwrap().len(), 1);
    // A response replay remains valid even after unrelated Project version changes.
    assert_eq!(submit(&mut store, input), first);
}
#[test]
fn context_edit_is_a_new_revision_and_template_does_not_skip_readback() {
    let (_temp, mut store) = setup();
    submit(&mut store, register("first", workflow()));
    submit(&mut store, action("compile", 1));
    assert_eq!(
        prepare(&store, action("approve", 1), &actor(), &NoEngine)
            .unwrap_err()
            .code,
        "WORKFLOW_READBACK_REQUIRED"
    );
    let mut next = register("next", workflow());
    if let Action::Register {
        supersedes,
        workflow,
        ..
    } = &mut next.action
    {
        *supersedes = Some(1);
        workflow.context = "new context".into();
    }
    submit(&mut store, next);
    assert_eq!(store.list("workflow_revision").unwrap().len(), 2);
    assert_eq!(
        at(&store, &actor(), "P", "plan", 1)
            .unwrap()
            .1
            .workflow
            .context,
        ""
    );
    let mut disabled = register("disabled", workflow());
    if let Action::Register {
        workflow_id,
        workflow,
        snapshot,
        provenance,
        ..
    } = &mut disabled.action
    {
        *workflow_id = "disabled".into();
        workflow.readback.required = false;
        *snapshot = None;
        *provenance = None;
    }
    submit(&mut store, disabled);
    for kind in ["compile", "approve"] {
        let mut a = action(kind, 1);
        a.key = format!("disabled-{kind}");
        match &mut a.action {
            Action::Compile { workflow_id, .. } | Action::Approve { workflow_id, .. } => {
                *workflow_id = "disabled".into()
            }
            _ => unreachable!(),
        }
        submit(&mut store, a);
    }
    assert!(require_approved(&store, "P", "disabled", 1).is_ok());
}
struct Proof {
    read: Readback,
    actual: InvocationEvidence,
    producer: Option<InvocationEvidence>,
    roster: Vec<Reference>,
}
impl Readbacks for Proof {
    fn readback(&self, _: &Reference) -> store::Result<Readback> {
        Ok(self.read.clone())
    }
    fn invocation(&self, r: &Reference) -> store::Result<InvocationEvidence> {
        if r.key.id == "read" {
            Ok(self.actual.clone())
        } else {
            self.producer
                .clone()
                .ok_or_else(|| chat::invalid("unknown producer"))
        }
    }
    fn roster(&self, _: &str) -> store::Result<Vec<Reference>> {
        Ok(self.roster.clone())
    }
}
fn proof() -> Proof {
    let w = serde_json::to_value(workflow()).unwrap();
    let snapshot_value = serde_json::to_value(snapshot()).unwrap();
    let entries:Vec<_>=[("workflow_revision:plan:1",w), ("workflow_snapshot:plan:1",snapshot_value)].into_iter().map(|(id,value)|{let bytes=canonical_json(&value).unwrap();json!({"source":{"id":id,"revision":"1","digest":canonical_json_sha256(&value).unwrap()},"description":"frozen","required":true,"offline_required":true,"delivery":{"mode":"inline","bytes":base64_for_fixture(&bytes)},"bytes_digest":bytes_sha256(&bytes)})}).collect();
    let owner = json!({"project":"P","kind":"room_invocation","id":"read","generation":1});
    let bundle:Bundle=serde_json::from_value(json!({"id":"bundle","manifest":frozen("manifest"),"consumer":owner,"entries":entries,"renderer":frozen("renderer"),"tokenizer":frozen("tokenizer"),"redaction":frozen("redaction"),"compression":[],"candidate_tokens":null,"selected_tokens":null,"delivered_tokens":null,"permission_digest":sha("permissions"),"budget":99999,"retention":"private"})).unwrap();
    let delivered = Sealed::new(bundle).unwrap();
    let spec=serde_json::from_value(json!({"owner":owner,"project":frozen("P"),"selection":frozen("reader"),"selection_policy_digest":sha("policy"),"profession":{"reference":frozen("readback"),"harness":frozen("fixture"),"model":"fixture","persona":"reader","terms":"readonly","default_role":"planner","skills":[],"capabilities":agency_proto::Capabilities::default()},"profile":frozen("reader-profile"),"manifest":frozen("manifest"),"bundle":{"id":"bundle","revision":"1","digest":delivered.digest},"binding":frozen("agency"),"required_capabilities":agency_proto::Capabilities::default(),"input_policy":"no_input","permission_digest":sha("permissions"),"permissions":["context.read"],"budget":99999,"deadline_ms":9999999999999u64,"repo":null,"base":null,"delivery_scope":[],"write_lease":null,"review_publish_policy":null,"idempotency_key":"read"})).unwrap();
    let actual = InvocationEvidence {
        selection: target("selection", "reader"),
        source_room: None,
        fresh: true,
        spec: Sealed::new(spec).unwrap(),
        delivered,
    };
    let read = Readback {
        workflow_digest: canonical_json_sha256(&serde_json::to_value(workflow()).unwrap()).unwrap(),
        snapshot_digest: canonical_json_sha256(&serde_json::to_value(snapshot()).unwrap()).unwrap(),
        invocation: target("workflow_invocation_evidence", "read"),
        selection: actual.selection.clone(),
        bundle_digest: actual.delivered.digest.clone(),
        body_digest: sha("readback body"),
    };
    Proof {
        read,
        actual,
        producer: None,
        roster: vec![],
    }
}
// The wire format uses base64; use its existing serde encoder through a Delivery.
fn base64_for_fixture(bytes: &[u8]) -> Value {
    serde_json::to_value(Delivery::Inline {
        bytes: bytes.to_vec(),
    })
    .unwrap()["bytes"]
        .clone()
}
fn verify(p: &Proof, source: &Provenance) -> store::Result<Value> {
    readback::verify(
        p,
        &target("workflow_readback", "r"),
        &canonical_json_sha256(&serde_json::to_value(workflow()).unwrap()).unwrap(),
        &canonical_json_sha256(&serde_json::to_value(snapshot()).unwrap()).unwrap(),
        source,
        true,
    )
}
fn reseal(p: &mut Proof) {
    p.actual.delivered = Sealed::new(p.actual.delivered.document.clone()).unwrap();
    p.read.bundle_digest = p.actual.delivered.digest.clone();
    p.actual.spec.document.bundle.digest = p.actual.delivered.digest.clone();
    p.actual.spec = Sealed::new(p.actual.spec.document.clone()).unwrap();
}
#[test]
fn readback_requires_registered_claimed_and_actually_delivered_digests() {
    assert!(verify(&proof(), &provenance()).is_ok());
    let mut p = proof();
    p.read.workflow_digest = sha("wrong graph");
    assert!(verify(&p, &provenance()).is_err());
    let mut p = proof();
    p.read.snapshot_digest = sha("wrong snapshot");
    assert!(verify(&p, &provenance()).is_err());
    // Honest old frozen Bundle, new readback claims: recompute all transport hashes.
    let mut p = proof();
    let entry = &mut p.actual.delivered.document.entries[1];
    let old = json!({"decided":[],"unresolved":["old"],"out_of_scope":[]});
    let bytes = canonical_json(&old).unwrap();
    entry.source.digest = canonical_json_sha256(&old).unwrap();
    entry.bytes_digest = bytes_sha256(&bytes);
    entry.delivery = Delivery::Inline { bytes };
    reseal(&mut p);
    assert_eq!(
        verify(&p, &provenance()).unwrap_err().code,
        "WORKFLOW_READBACK"
    );
    let mut p = proof();
    p.actual.spec.document.bundle.digest = sha("previous bundle");
    p.actual.spec = Sealed::new(p.actual.spec.document.clone()).unwrap();
    assert!(verify(&p, &provenance()).is_err());
    let mut p = proof();
    p.actual.delivered.document.entries[0].source.digest =
        "0123456789012345678901234567890123456789".into();
    reseal(&mut p);
    assert!(verify(&p, &provenance()).is_err());
}
#[test]
fn no_source_room_only_exempts_roster_not_isolation_or_production_origin() {
    let mut p = proof();
    p.actual.fresh = false;
    assert!(verify(&p, &provenance()).is_err());
    for permission in [
        "room.read",
        "room:other:messages.read",
        "other.messages.read",
        "context.rooms.read",
        "*",
    ] {
        let mut p = proof();
        p.actual.spec.document.permissions.push(permission.into());
        p.actual.spec = Sealed::new(p.actual.spec.document.clone()).unwrap();
        assert!(verify(&p, &provenance()).is_err());
    }
    let mut p = proof();
    p.actual.spec.document.owner.kind = agency_proto::OwnerKind::RunAttempt;
    p.actual.spec = Sealed::new(p.actual.spec.document.clone()).unwrap();
    assert!(verify(&p, &provenance()).is_err());
    let mut p = proof();
    let entry = p.actual.delivered.document.entries[0].clone();
    p.actual.delivered.document.entries.push(entry);
    reseal(&mut p);
    assert!(verify(&p, &provenance()).is_err());
    let mut source = provenance();
    source.production = Production::Missing {};
    assert!(verify(&proof(), &source).is_err());
    source.production = Production::Invocations { references: vec![] };
    assert!(verify(&proof(), &source).is_err());
    source.production = Production::Invocations {
        references: vec![target("room_invocation", "unknown")],
    };
    assert!(verify(&proof(), &source).is_err());
}
#[test]
fn current_roster_and_actual_producer_profiles_are_checked() {
    let mut p = proof();
    let mut source = provenance();
    source.source_chain.push(SourceMessage {
        room_id: "source".into(),
        event: target("chat_source_reference", "e"),
    });
    p.roster.push(p.actual.selection.clone());
    assert!(verify(&p, &source).is_err());
    p.roster.clear();
    let mut producer = p.actual.clone();
    producer.source_room = Some("source".into());
    p.producer = Some(producer);
    source.production = Production::Invocations {
        references: vec![target("room_invocation", "producer")],
    };
    assert!(verify(&p, &source).is_err());
    p.producer.as_mut().unwrap().spec.document.profile.digest = sha("other-profile");
    let producer = p.producer.as_mut().unwrap();
    producer.spec = Sealed::new(producer.spec.document.clone()).unwrap();
    assert!(verify(&p, &source).is_ok());
}

mod cli;

mod summary;

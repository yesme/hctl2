use super::*;
use agency_proto::{Dispatch, DispatchState, ExecutionSpec, Sealed};
use context::{Assembly, Bundle, Delivery, Entry, Manifest};
use project::invocation::{self as call, State};
use store::{EffectIntent, EffectState, Readback};

const NOW: u64 = 1000;
fn frozen_ref(id: &str, digest: &str) -> FrozenRef {
    FrozenRef {
        id: id.into(),
        revision: digest.into(),
        digest: digest.into(),
    }
}
fn setup() -> (Env, call::Input) {
    let mut e = Env::new();
    let room = chat::main_binding(&e.store, &e.a).unwrap().1.id;
    let selected = accepted_selection(&mut e, &room, "script-agency", true, None);
    let profile = selected.worker_profiles[0].clone();
    e.apply("select", select_action(&e.a, &room, vec![selected]))
        .unwrap();
    let input = call::Input {
        key: "call".into(),
        project_id: e.a.clone(),
        room_id: room,
        target: "research".into(),
        profile,
        request: "compare these alternatives".into(),
        budget: 65536,
        deadline_ms: 60000,
        retry_of: None,
    };
    (e, input)
}
fn assembly(preview: &call::Preview) -> Assembly {
    let request = preview.input.request.as_bytes().to_vec();
    let skill_bytes = b"skill".to_vec();
    let request_ref = frozen_ref(
        &format!("invocation-request/{}", preview.consumer.id),
        &hash(&request),
    );
    let skill = FrozenRef {
        id: "method".into(),
        revision: "1".into(),
        digest: hash(&skill_bytes),
    };
    let source_ids = vec![request_ref.id.clone(), skill.id.clone()];
    let permission = context::permission_digest(&source_ids);
    let policy = frozen_ref("mechanical", &hash(b"policy"));
    let manifest = Sealed::new(Manifest {
        id: format!("manifest-{}", preview.consumer.id),
        purpose: "research".into(),
        scope: format!("project {}", preview.consumer.project),
        parent: None,
        sources: vec![request_ref.clone(), skill.clone()],
        selection_policy: policy.clone(),
        freshness: "exact".into(),
        coverage: "exact request and Skill".into(),
        known_gaps: vec!["fixture: no online window".into()],
        required_skills: vec![skill.clone()],
        permission_digest: permission.clone(),
        redaction: policy.clone(),
        budget: preview.input.budget,
    })
    .unwrap();
    let entries = [(request_ref, request), (skill, skill_bytes)]
        .into_iter()
        .map(|(source, bytes)| Entry {
            source,
            description: "exact required bytes".into(),
            required: true,
            offline_required: true,
            bytes_digest: hash(&bytes),
            delivery: Delivery::Inline { bytes },
        })
        .collect();
    let bundle = Sealed::new(Bundle {
        id: format!("bundle-{}", preview.consumer.id),
        manifest: frozen_ref(&manifest.document.id, &manifest.digest),
        consumer: preview.consumer.clone(),
        entries,
        renderer: policy.clone(),
        tokenizer: policy.clone(),
        redaction: policy,
        compression: vec![],
        candidate_tokens: None,
        selected_tokens: None,
        delivered_tokens: None,
        permission_digest: permission,
        budget: preview.input.budget,
        retention: "until-terminal-and-admitted".into(),
    })
    .unwrap();
    Assembly { manifest, bundle }
}
fn prepared(e: &mut Env, input: call::Input) -> (call::Preview, Assembly) {
    let preview = call::prepare(&e.store, &actor(), input, NOW).unwrap();
    let assembly = assembly(&preview);
    let scope_actor = chat::owner(&actor(), &e.a).unwrap();
    context::save_assembly(
        &mut e.store,
        &scope_actor,
        &e.a,
        &format!("context-{}", preview.consumer.id),
        &assembly,
    )
    .unwrap();
    (preview, assembly)
}
fn started(e: &mut Env, input: call::Input) -> (call::Preview, Assembly, Reference) {
    let (p, a) = prepared(e, input);
    let result = call::start(&mut e.store, &actor(), &p, &a, NOW).unwrap();
    let owner = serde_json::from_value(result["owner"].clone()).unwrap();
    (p, a, owner)
}
fn end_input(e: &Env, id: &str, version: i64, outcome: State) -> call::End {
    call::End {
        key: "cancel".into(),
        project_id: e.a.clone(),
        invocation_id: id.into(),
        state_version: version,
        outcome,
        reason: "human cancellation".into(),
    }
}

#[test]
fn invocation_preview_is_readonly_and_rejects_nonhuman_or_unselected_candidates() {
    let (e, input) = setup();
    let before = e.store.read_stamp();
    let p = call::prepare(&e.store, &actor(), input.clone(), NOW).unwrap();
    assert_eq!(before, e.store.read_stamp());
    assert!(e.store.list("room_invocation").unwrap().is_empty());
    assert_eq!(p.consumer.generation, 1);
    let mut model = actor();
    model.0.source = ActorSource::InternalReducer;
    assert_eq!(
        call::prepare(&e.store, &model, input.clone(), NOW)
            .unwrap_err()
            .code,
        "PERMISSION_DENIED"
    );
    let mut wrong = input.clone();
    wrong.target = "researcher".into();
    assert_eq!(
        call::prepare(&e.store, &actor(), wrong, NOW)
            .unwrap_err()
            .code,
        "CANDIDATE_NOT_FOUND"
    );
    let mut wrong = input.clone();
    wrong.project_id = e.b.clone();
    assert!(call::prepare(&e.store, &actor(), wrong, NOW).is_err());
    let mut wrong = input.clone();
    wrong.profile.key.id = "other".into();
    assert_eq!(
        call::prepare(&e.store, &actor(), wrong, NOW)
            .unwrap_err()
            .code,
        "PROFILE_NOT_ALLOWED"
    );
    let mut wrong = input.clone();
    wrong.budget = 65537;
    assert_eq!(
        call::prepare(&e.store, &actor(), wrong, NOW)
            .unwrap_err()
            .code,
        "BUDGET_EXCEEDED"
    );
    let mut wrong = input;
    wrong.deadline_ms = NOW;
    assert_eq!(
        call::prepare(&e.store, &actor(), wrong, NOW)
            .unwrap_err()
            .code,
        "INVALID_INPUT"
    );
}

#[test]
fn invocation_first_step_commits_authorization_spec_state_and_outbox_without_dispatch() {
    let (mut e, input) = setup();
    let (p, a, owner) = started(&mut e, input);
    let (_, invocation) = call::invocation(&e.store, &e.a, &p.consumer.id).unwrap();
    assert_eq!(invocation.spec.document.owner, p.consumer);
    assert_eq!(invocation.spec.document.permissions, vec!["context.read"]);
    assert_eq!(invocation.spec.document.bundle.digest, a.bundle.digest);
    assert!(invocation.spec.document.write_lease.is_none());
    let intent = required(
        &e.store,
        &key(owner.key.scope.clone(), "dispatch_intent", &p.consumer.id),
    )
    .unwrap();
    let value: Value = decode(&intent).unwrap();
    assert!(value["dispatch"].is_null());
    assert!(e.store.list("dispatch").unwrap().is_empty());
    assert_eq!(
        e.store
            .effect(&format!("prepare:{}", p.consumer.id))
            .unwrap()
            .1,
        EffectState::Pending
    );
    assert_eq!(
        call::lifecycle(&e.store, &e.a, &p.consumer.id)
            .unwrap()
            .1
            .state,
        State::Pending
    );
    assert!(call::current_authorization(&e.store, &owner, NOW).unwrap());
    assert!(!call::current_authorization(&e.store, &owner, p.input.deadline_ms).unwrap());
    let result = call::start(&mut e.store, &actor(), &p, &a, NOW).unwrap();
    assert_eq!(result["state_version"], 1);
    assert_eq!(e.store.list("room_invocation").unwrap().len(), 1);
}

#[test]
fn invocation_rejects_missing_context_missing_skill_or_another_consumer() {
    let (mut e, input) = setup();
    let p = call::prepare(&e.store, &actor(), input, NOW).unwrap();
    let good = assembly(&p);
    assert_eq!(
        call::start(&mut e.store, &actor(), &p, &good, NOW)
            .unwrap_err()
            .code,
        "CONTEXT_NOT_ADMITTED"
    );
    let mut a = assembly(&p);
    a.bundle.document.entries.pop();
    a.bundle = Sealed::new(a.bundle.document).unwrap();
    assert_eq!(
        call::start(&mut e.store, &actor(), &p, &a, NOW)
            .unwrap_err()
            .code,
        "CONTEXT_MISMATCH"
    );
    let mut a = assembly(&p);
    a.bundle.document.consumer.id = "another".into();
    a.bundle = Sealed::new(a.bundle.document).unwrap();
    assert_eq!(
        call::start(&mut e.store, &actor(), &p, &a, NOW)
            .unwrap_err()
            .code,
        "CONTEXT_MISMATCH"
    );
    let mut a = assembly(&p);
    a.bundle.document.entries.remove(0);
    a.bundle = Sealed::new(a.bundle.document).unwrap();
    assert_eq!(
        call::start(&mut e.store, &actor(), &p, &a, NOW)
            .unwrap_err()
            .code,
        "CONTEXT_MISMATCH"
    );
    assert!(e.store.list("room_invocation").unwrap().is_empty());
}

#[test]
fn invocation_cannot_start_when_manifest_declares_skill_but_bundle_never_delivers_it() {
    let (mut e, input) = setup();
    let p = call::prepare(&e.store, &actor(), input, NOW).unwrap();
    let mut a = assembly(&p);
    // The Context format accepts declarations independently from entries. The
    // dispatch owner must prove that each required Skill's actual bytes arrived.
    a.manifest.document.sources.pop();
    a.manifest = Sealed::new(a.manifest.document).unwrap();
    a.bundle.document.entries.pop();
    a.bundle.document.manifest = frozen_ref(&a.manifest.document.id, &a.manifest.digest);
    a.bundle = Sealed::new(a.bundle.document).unwrap();
    let scoped = chat::owner(&actor(), &e.a).unwrap();
    context::save_assembly(&mut e.store, &scoped, &e.a, "missing-skill-context", &a).unwrap();
    assert_eq!(
        call::start(&mut e.store, &actor(), &p, &a, NOW)
            .unwrap_err()
            .code,
        "CONTEXT_MISMATCH"
    );
    assert!(e.store.list("room_invocation").unwrap().is_empty());
}

#[test]
fn dispatch_plan_does_not_infer_attempt_generation_from_store_state_version() {
    let (mut e, input) = setup();
    let (p, a, _) = started(&mut e, input);
    let mut spec = call::invocation(&e.store, &e.a, &p.consumer.id)
        .unwrap()
        .1
        .spec;
    spec.document.owner.kind = agency_proto::OwnerKind::RunAttempt;
    spec.document.owner.id = "attempt".into();
    spec.document.owner.generation = 77;
    let mut bundle = a.bundle;
    bundle.document.consumer = spec.document.owner.clone();
    bundle = Sealed::new(bundle.document).unwrap();
    spec.document.bundle = frozen_ref(&bundle.document.id, &bundle.digest);
    spec = Sealed::new(spec.document).unwrap();
    let owner = value_record(
        key(Scope::Project(e.a.clone()), "run_attempt", "attempt"),
        2,
        &json!({"attempt_generation":77}),
    )
    .unwrap();
    assert!(
        participant::dispatch::plan(&e.store, "attempt-dispatch", &owner, &spec, &bundle).is_ok()
    );
}

#[test]
fn invocation_old_preview_policy_or_roster_cannot_start() {
    let (mut e, input) = setup();
    let (p, a) = prepared(&mut e, input);
    let mut definition = def("changed");
    definition.settings.selection_policy = json!({"allowed_agencies":[]});
    e.apply(
        "update",
        Action::Update {
            project_id: e.a.clone(),
            version: 1,
            definition,
        },
    )
    .unwrap();
    assert!(call::start(&mut e.store, &actor(), &p, &a, NOW).is_err());
    assert!(e.store.list("room_invocation").unwrap().is_empty());
    let (mut e, input) = setup();
    let (p, a) = prepared(&mut e, input);
    e.apply(
        "roster",
        Action::Select {
            project_id: e.a.clone(),
            project_version: 1,
            room_id: p.input.room_id.clone(),
            topic_command_key: None,
            roster_version: Some(1),
            selections: vec![],
        },
    )
    .unwrap();
    assert!(call::start(&mut e.store, &actor(), &p, &a, NOW).is_err());
    assert!(e.store.list("room_invocation").unwrap().is_empty());
}

#[test]
fn invocation_outbox_failure_rolls_back_all_authorization_records() {
    let (mut e, input) = setup();
    let (p, a) = prepared(&mut e, input);
    let owner = reference(&project(&e.store, &e.a).unwrap());
    let scoped = chat::owner(&actor(), &e.a).unwrap();
    let input = json!({});
    let command = Command {
        command_id: "occupy".into(),
        idempotency_key: "occupy".into(),
        actor: scoped.0.clone(),
        target: key(Scope::Control, "fixture", "occupy"),
        expected: Expected::Absent,
        binding: owner.clone(),
        operation: "fixture".into(),
        input_digest: Command::digest_input("fixture", &input).unwrap(),
        input: input.clone(),
    };
    e.store
        .submit(e.store.generation(), &scoped, &command, None, |tx| {
            tx.enqueue_effect(&EffectIntent {
                intent_id: "occupied".into(),
                owner: owner.clone(),
                binding: owner.clone(),
                operation: "fixture".into(),
                target: "fixture".into(),
                conflict_scope: format!("dispatch:{}", p.consumer.id),
                permission_scope: owner.key.scope.clone(),
                input_digest: Command::digest_input("fixture", &input).unwrap(),
                input,
                idempotency_key: "occupied".into(),
            })?;
            Ok(json!({}))
        })
        .unwrap();
    assert_eq!(
        call::start(&mut e.store, &actor(), &p, &a, NOW)
            .unwrap_err()
            .code,
        "EFFECT_CONFLICT"
    );
    for kind in ["room_invocation", "invocation_state", "dispatch_intent"] {
        assert!(e.store.list(kind).unwrap().is_empty(), "{kind}");
    }
    assert!(
        !e.store
            .has_effect(&format!("prepare:{}", p.consumer.id))
            .unwrap()
    );
}

#[test]
fn invocation_legal_edges_are_exact_and_terminal_states_never_revive() {
    use State::*;
    let all = [
        Pending,
        Running,
        WaitingInput,
        Lost,
        Completed,
        Failed,
        Cancelled,
    ];
    let legal = [
        (Pending, Running),
        (Pending, Failed),
        (Pending, Cancelled),
        (Pending, Lost),
        (Running, WaitingInput),
        (WaitingInput, Running),
        (Running, Completed),
        (Running, Failed),
        (Running, Cancelled),
        (Running, Lost),
        (WaitingInput, Completed),
        (WaitingInput, Failed),
        (WaitingInput, Cancelled),
        (WaitingInput, Lost),
    ];
    for from in all {
        for to in all {
            assert_eq!(
                from.allows(to),
                legal.contains(&(from, to)),
                "{from:?} -> {to:?}"
            );
        }
    }
}

fn activate(e: &mut Env, p: &call::Preview) {
    let scoped = chat::owner(&actor(), &e.a).unwrap();
    let intent = required(
        &e.store,
        &key(
            Scope::Project(e.a.clone()),
            "dispatch_intent",
            &p.consumer.id,
        ),
    )
    .unwrap();
    let v: Value = decode(&intent).unwrap();
    let spec: Sealed<ExecutionSpec> = serde_json::from_value(v["spec"].clone()).unwrap();
    let dispatch = Dispatch {
        reference: format!("dispatch-{}", p.consumer.id),
        owner: p.consumer.clone(),
        spec_digest: spec.digest,
        bundle_digest: spec.document.bundle.digest,
        binding: spec.document.binding,
        capabilities: spec.document.required_capabilities,
        state: DispatchState::Prepared,
    };
    e.store
        .begin_effect(e.store.generation(), &format!("prepare:{}", p.consumer.id))
        .unwrap();
    participant::record_dispatch(&mut e.store, &scoped, &intent, &dispatch).unwrap();
    let activation = format!("activate:{}", p.consumer.id);
    let effect = e
        .store
        .begin_effect(e.store.generation(), &activation)
        .unwrap();
    let input = json!({"dispatch":dispatch});
    let command = Command {
        command_id: "activation-reply".into(),
        idempotency_key: "activation-reply".into(),
        actor: scoped.0.clone(),
        target: key(Scope::Control, "fixture", "activation"),
        expected: Expected::Absent,
        binding: effect.binding.clone(),
        operation: "fixture".into(),
        input_digest: Command::digest_input("fixture", &input).unwrap(),
        input,
    };
    e.store
        .submit(e.store.generation(), &scoped, &command, None, |tx| {
            tx.confirm_effect(
                &activation,
                &Readback::Confirmed {
                    binding: effect.binding.clone(),
                    target: effect.target.clone(),
                    input_digest: effect.input_digest.clone(),
                    result: json!({"accepted":true}),
                },
            )?;
            Ok(json!({}))
        })
        .unwrap();
}

#[test]
fn invocation_running_state_does_not_advance_or_regrant_semantic_authority() {
    let (mut e, input) = setup();
    let (p, _, owner) = started(&mut e, input);
    assert!(call::record_started(&mut e.store, &actor(), &e.a, &p.consumer.id, NOW).is_err());
    activate(&mut e, &p);
    call::record_started(&mut e.store, &actor(), &e.a, &p.consumer.id, NOW).unwrap();
    assert_eq!(
        call::lifecycle(&e.store, &e.a, &p.consumer.id)
            .unwrap()
            .0
            .version,
        2
    );
    assert_eq!(
        reference(&call::invocation(&e.store, &e.a, &p.consumer.id).unwrap().0),
        owner
    );
    assert!(call::current_authorization(&e.store, &owner, NOW).unwrap());
    call::record_started(&mut e.store, &actor(), &e.a, &p.consumer.id, NOW).unwrap();
    assert_eq!(
        call::lifecycle(&e.store, &e.a, &p.consumer.id)
            .unwrap()
            .0
            .version,
        2
    );
}

#[test]
fn invocation_cancel_unsent_revokes_and_replay_cannot_resurrect() {
    let (mut e, input) = setup();
    let (p, a, owner) = started(&mut e, input);
    let end = end_input(&e, &p.consumer.id, 1, State::Cancelled);
    let result = call::end(&mut e.store, &actor(), end.clone()).unwrap();
    assert!(!call::current_authorization(&e.store, &owner, NOW).unwrap());
    assert_eq!(
        e.store
            .effect(&format!("prepare:{}", p.consumer.id))
            .unwrap()
            .1,
        EffectState::Cancelled
    );
    assert_eq!(result["cleanup_pending"], false);
    assert_eq!(call::end(&mut e.store, &actor(), end).unwrap(), result);
    call::start(&mut e.store, &actor(), &p, &a, NOW).unwrap();
    assert_eq!(
        call::lifecycle(&e.store, &e.a, &p.consumer.id)
            .unwrap()
            .1
            .state,
        State::Cancelled
    );
    assert!(
        !call::invocation(&e.store, &e.a, &p.consumer.id)
            .unwrap()
            .1
            .authorization
            .valid
    );
    let mut end = end_input(&e, &p.consumer.id, 2, State::Cancelled);
    end.key = "revive".into();
    assert_eq!(
        call::end(&mut e.store, &actor(), end).unwrap_err().code,
        "INVALID_TRANSITION"
    );
}

#[test]
fn invocation_unknown_prepare_revokes_before_cleanup_and_does_not_claim_stopped() {
    let (mut e, input) = setup();
    let (p, _, owner) = started(&mut e, input);
    e.store
        .begin_effect(e.store.generation(), &format!("prepare:{}", p.consumer.id))
        .unwrap();
    let end = end_input(&e, &p.consumer.id, 1, State::Cancelled);
    let result = call::end(&mut e.store, &actor(), end).unwrap();
    assert_eq!(result["cleanup_pending"], true);
    assert!(!call::current_authorization(&e.store, &owner, NOW).unwrap());
    assert_eq!(
        e.store
            .effect(&format!("prepare:{}", p.consumer.id))
            .unwrap()
            .1,
        EffectState::Unknown
    );
    let (stop, state) = e.store.effect(&format!("stop:{}", p.consumer.id)).unwrap();
    assert_eq!(state, EffectState::Pending);
    assert_eq!(stop.input["dispatch_intent"], p.consumer.id);
    assert_eq!(stop.owner, owner);
}

#[test]
fn invocation_retry_is_new_owner_and_retains_original_without_reusing_bundle() {
    let (mut e, input) = setup();
    let (p, old_bundle, owner) = started(&mut e, input);
    let mut retry = p.input.clone();
    retry.key = "retry".into();
    retry.retry_of = Some(owner);
    assert_eq!(
        call::prepare(&e.store, &actor(), retry.clone(), NOW)
            .unwrap_err()
            .code,
        "RETRY_NOT_ALLOWED"
    );
    let end = end_input(&e, &p.consumer.id, 1, State::Cancelled);
    call::end(&mut e.store, &actor(), end).unwrap();
    retry.retry_of = Some(reference(
        &call::invocation(&e.store, &e.a, &p.consumer.id).unwrap().0,
    ));
    let (new, bundle) = prepared(&mut e, retry);
    assert_ne!(new.consumer.id, p.consumer.id);
    assert_eq!(
        call::start(&mut e.store, &actor(), &new, &old_bundle, NOW)
            .unwrap_err()
            .code,
        "CONTEXT_MISMATCH"
    );
    call::start(&mut e.store, &actor(), &new, &bundle, NOW).unwrap();
    let (root, invocation) = call::invocation(&e.store, &e.a, &new.consumer.id).unwrap();
    assert!(root.sources.contains(new.input.retry_of.as_ref().unwrap()));
    assert!(invocation.authorization.valid);
    assert_eq!(e.store.list("room_invocation").unwrap().len(), 2);
}

#[test]
fn invocation_updates_affect_future_calls_not_active_frozen_spec() {
    let (mut e, input) = setup();
    let (p, _, owner) = started(&mut e, input);
    let original = call::invocation(&e.store, &e.a, &p.consumer.id)
        .unwrap()
        .1
        .spec;
    e.apply(
        "clear-roster",
        Action::Select {
            project_id: e.a.clone(),
            project_version: 1,
            room_id: p.input.room_id.clone(),
            topic_command_key: None,
            roster_version: Some(1),
            selections: vec![],
        },
    )
    .unwrap();
    let mut definition = def("updated");
    definition.settings.selection_policy = json!({"allowed_agencies":[]});
    e.apply(
        "new-policy",
        Action::Update {
            project_id: e.a.clone(),
            version: 1,
            definition,
        },
    )
    .unwrap();
    assert!(call::current_authorization(&e.store, &owner, NOW).unwrap());
    assert_eq!(
        call::invocation(&e.store, &e.a, &p.consumer.id)
            .unwrap()
            .1
            .spec,
        original
    );
}

#[test]
fn invocation_failure_and_loss_need_original_reducer_authority_and_completion_is_not_a_command() {
    for outcome in [State::Failed, State::Lost] {
        let (mut e, input) = setup();
        let (p, _, owner) = started(&mut e, input);
        let end = end_input(&e, &p.consumer.id, 1, outcome);
        assert_eq!(
            call::end(&mut e.store, &actor(), end.clone())
                .unwrap_err()
                .code,
            "PERMISSION_DENIED"
        );
        let mut reducer = chat::owner(&actor(), &e.a).unwrap();
        reducer.0.source = ActorSource::InternalReducer;
        reducer.0.authority = Some(owner.clone());
        let result = call::end(&mut e.store, &reducer, end).unwrap();
        assert_eq!(result["state"], serde_json::to_value(outcome).unwrap());
        assert!(!call::current_authorization(&e.store, &owner, NOW).unwrap());
    }
    let (mut e, input) = setup();
    let (p, _, _) = started(&mut e, input);
    let end = end_input(&e, &p.consumer.id, 1, State::Completed);
    assert_eq!(
        call::end(&mut e.store, &actor(), end).unwrap_err().code,
        "INVALID_INPUT"
    );
    let end = end_input(&e, &p.consumer.id, i64::MAX, State::Cancelled);
    assert_eq!(
        call::end(&mut e.store, &actor(), end).unwrap_err().code,
        "INVALID_INPUT"
    );
    assert_eq!(
        call::lifecycle(&e.store, &e.a, &p.consumer.id)
            .unwrap()
            .1
            .state,
        State::Pending
    );
}

#[test]
fn invocation_cancellation_cleanup_conflict_rolls_back_revocation_and_state() {
    let (mut e, input) = setup();
    let (p, _, owner) = started(&mut e, input);
    e.store
        .begin_effect(e.store.generation(), &format!("prepare:{}", p.consumer.id))
        .unwrap();
    let scoped = chat::owner(&actor(), &e.a).unwrap();
    let input = json!({});
    let command = Command {
        command_id: "occupy-stop".into(),
        idempotency_key: "occupy-stop".into(),
        actor: scoped.0.clone(),
        target: key(Scope::Control, "fixture", "occupy-stop"),
        expected: Expected::Absent,
        binding: owner.clone(),
        operation: "fixture".into(),
        input_digest: Command::digest_input("fixture", &input).unwrap(),
        input: input.clone(),
    };
    e.store
        .submit(e.store.generation(), &scoped, &command, None, |tx| {
            tx.enqueue_effect(&EffectIntent {
                intent_id: "stop-conflict".into(),
                owner: owner.clone(),
                binding: owner.clone(),
                operation: "fixture".into(),
                target: "fixture".into(),
                conflict_scope: format!("stop:{}", p.consumer.id),
                permission_scope: owner.key.scope.clone(),
                input_digest: Command::digest_input("fixture", &input).unwrap(),
                input,
                idempotency_key: "occupied-stop".into(),
            })?;
            Ok(json!({}))
        })
        .unwrap();
    let end = end_input(&e, &p.consumer.id, 1, State::Cancelled);
    assert_eq!(
        call::end(&mut e.store, &actor(), end).unwrap_err().code,
        "EFFECT_CONFLICT"
    );
    assert!(call::current_authorization(&e.store, &owner, NOW).unwrap());
    assert_eq!(
        call::lifecycle(&e.store, &e.a, &p.consumer.id)
            .unwrap()
            .0
            .version,
        1
    );
}

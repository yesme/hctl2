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
        task_id: None,
        review_change_set_revision: None,
        write: None,
        target: "research".into(),
        profile,
        request: "compare these alternatives".into(),
        budget: 65536,
        deadline_ms: 60000,
        retry_of: None,
    };
    (e, input)
}

fn write_setup() -> (Env, call::Input) {
    let (mut e, mut input) = setup();
    let (_, original) = participant::profiles::profile_at(&e.store, &input.profile).unwrap();
    let mut writer = original;
    writer.mode = "write".into();
    writer.permissions = vec!["context.read".into(), "git.read".into(), "git.write".into()];
    let plan = participant::profiles::prepare_profile(
        &e.store,
        participant::profiles::ProfileInput {
            key: "writer-profile".into(),
            action: participant::profiles::ProfileAction::Create {
                id: "writer".into(),
                profile: writer,
            },
        },
        &actor(),
    )
    .unwrap();
    let result = participant::profiles::admit_profile(&mut e.store, &actor(), plan).unwrap();
    input.profile = serde_json::from_value(result["revision"].clone()).unwrap();
    let selected = participant::selection::resolve_room_candidate(
        &e.store,
        &e.a,
        &input.room_id,
        &input.target,
    )
    .unwrap();
    let mut selection: Selection = decode(&selected).unwrap();
    selection.worker_profiles = vec![input.profile.clone()];
    selection.permission = json!({"allow":["context.read","git.read","git.write"]});
    e.apply(
        "writer-roster",
        Action::Select {
            project_id: e.a.clone(),
            project_version: 1,
            room_id: input.room_id.clone(),
            topic_command_key: None,
            roster_version: Some(1),
            selections: vec![selection],
        },
    )
    .unwrap();
    input.write = Some(call::WriteInput {
        change_set_id: None,
        baseline_commit: "a".repeat(40),
        target_branch: "main".into(),
        allow_update: true,
    });
    freeze_policy_fixture(&mut e, &input);
    (e, input)
}

/// Use the shared Repo policy API; this slice does not run the publishing worker.
fn freeze_policy_fixture(e: &mut Env, input: &call::Input) {
    let (id, policy) = call::review_policy_input(&e.store, input).unwrap().unwrap();
    let mut a = actor();
    a.0.permission_scope.push(Scope::Repo(e.rid.clone()));
    repo::review::freeze_policy(&mut e.store, &a, &id, policy).unwrap();
}

#[test]
fn write_invocation_preview_does_not_grant_and_start_freezes_policy_baseline_and_lease() {
    let (mut e, input) = write_setup();
    let (p, a) = prepared(&mut e, input);
    let w = p.write.as_ref().unwrap();
    assert_eq!(
        w.lease.pending.lease.state,
        repo::changeset::LeaseState::Pending
    );
    assert!(e.store.list("changeset").unwrap().is_empty());
    assert!(e.store.list("review_publish_intent").unwrap().is_empty());
    let first = call::start(&mut e.store, &actor(), &p, &a, NOW).unwrap();
    assert_eq!(
        call::start(&mut e.store, &actor(), &p, &a, NOW).unwrap(),
        first
    );
    let (_, invocation) = call::invocation(&e.store, &e.a, &p.consumer.id).unwrap();
    let set =
        repo::changeset::get_change_set(&e.store, &e.rid, &w.lease.pending.change_set_id).unwrap();
    assert_eq!(set.lease.state, repo::changeset::LeaseState::Active);
    assert_eq!(invocation.spec.document.base, Some("a".repeat(40)));
    assert_eq!(
        invocation.spec.document.write_lease.as_ref().unwrap().id,
        set.lease.lease_id
    );
    assert_eq!(
        invocation.spec.document.review_publish_policy,
        Some(w.review_publish_policy.clone())
    );
    assert!(invocation.authorization.write);
    assert_eq!(
        invocation.authorization.authorizing_actor.unwrap().source,
        ActorSource::DirectClient
    );
    let mut second = p.input.clone();
    second.key = "second".into();
    second.write.as_mut().unwrap().change_set_id = Some(set.change_set_id);
    freeze_policy_fixture(&mut e, &second);
    assert_eq!(
        call::prepare(&e.store, &actor(), second, NOW)
            .unwrap_err()
            .code,
        "WRITE_LEASE_BUSY"
    );
}

#[test]
fn cancelling_unsent_write_revokes_in_the_same_transaction_and_next_writer_has_new_generation() {
    let (mut e, input) = write_setup();
    let (p, _, _) = started(&mut e, input);
    let set = &p.write.as_ref().unwrap().lease.pending;
    let cancel = end_input(&e, &p.consumer.id, 1, State::Cancelled);
    call::end(&mut e.store, &actor(), cancel).unwrap();
    let stopped = repo::changeset::get_change_set(&e.store, &e.rid, &set.change_set_id).unwrap();
    assert_eq!(stopped.lease.state, repo::changeset::LeaseState::Revoked);
    let mut next = p.input.clone();
    next.key = "replacement".into();
    next.write.as_mut().unwrap().change_set_id = Some(set.change_set_id.clone());
    freeze_policy_fixture(&mut e, &next);
    let (p, _, _) = started(&mut e, next);
    let current = repo::changeset::get_change_set(&e.store, &e.rid, &set.change_set_id).unwrap();
    assert_eq!(current.lease.state, repo::changeset::LeaseState::Active);
    assert_eq!(current.lease.generation, 2);
    assert_eq!(
        current.lease.holder,
        repo::changeset::ProducerRef::Invocation {
            invocation_id: p.consumer.id,
            invocation_version: 1
        }
    );
}

#[test]
fn write_preview_cannot_reuse_a_changeset_opened_under_another_platform_binding() {
    let (mut e, input) = write_setup();
    let (p, _, _) = started(&mut e, input);
    let set = &p.write.as_ref().unwrap().lease.pending;
    let cancel = end_input(&e, &p.consumer.id, 1, State::Cancelled);
    call::end(&mut e.store, &actor(), cancel).unwrap();
    let mut registration = required(&e.store, &repo::key(&e.rid)).unwrap();
    let store::RecordData::Repo {
        platform_binding: Some(binding),
        ..
    } = &mut registration.data
    else {
        panic!("platform binding required");
    };
    binding.version = Version::State(2);
    registration.version += 1;
    let mut repo_actor = actor();
    repo_actor
        .0
        .permission_scope
        .push(Scope::Repo(e.rid.clone()));
    replace_record_as(&mut e, registration, repo_actor);
    let mut next = p.input.clone();
    next.key = "rebound-writer".into();
    next.write.as_mut().unwrap().change_set_id = Some(set.change_set_id.clone());
    freeze_policy_fixture(&mut e, &next);
    let stamp = e.store.read_stamp();
    assert_eq!(
        call::prepare(&e.store, &actor(), next, NOW)
            .unwrap_err()
            .code,
        "CHANGESET_BOUNDARY_MISMATCH"
    );
    assert_eq!(e.store.read_stamp(), stamp);
    assert_eq!(
        repo::changeset::get_change_set(&e.store, &e.rid, &set.change_set_id)
            .unwrap()
            .binding_version,
        set.binding_version
    );
    assert!(e.store.list(repo::review::INTENT_KIND).unwrap().is_empty());
}

#[test]
fn cancelling_possibly_started_write_without_exit_proof_keeps_the_original_lease_busy() {
    let (mut e, input) = write_setup();
    let (p, _, _) = started(&mut e, input);
    let set = &p.write.as_ref().unwrap().lease.pending;
    e.store
        .begin_effect(e.store.generation(), &format!("prepare:{}", p.consumer.id))
        .unwrap();
    let cancel = end_input(&e, &p.consumer.id, 1, State::Cancelled);
    call::end(&mut e.store, &actor(), cancel).unwrap();
    assert_eq!(
        repo::changeset::get_change_set(&e.store, &e.rid, &set.change_set_id)
            .unwrap()
            .lease
            .state,
        repo::changeset::LeaseState::Revoking
    );
    assert_eq!(
        repo::changeset::plan_lease(
            &e.store,
            &e.rid,
            set.binding_version,
            &set.baseline_commit,
            "next",
            Some(&set.change_set_id),
            &repo::changeset::ProducerRef::Invocation {
                invocation_id: "next".into(),
                invocation_version: 1
            }
        )
        .unwrap_err()
        .code,
        "WRITE_LEASE_BUSY"
    );
}

#[test]
fn read_only_profile_cannot_smuggle_a_write_boundary_into_the_preview() {
    let (e, mut input) = setup();
    input.write = Some(call::WriteInput {
        change_set_id: None,
        baseline_commit: "a".repeat(40),
        target_branch: "main".into(),
        allow_update: true,
    });
    assert_eq!(
        call::prepare(&e.store, &actor(), input, NOW)
            .unwrap_err()
            .code,
        "WRITE_BOUNDARY_NOT_ALLOWED"
    );
}

#[test]
fn stopped_write_requires_original_dispatch_exit_not_a_turn_or_stop_request() {
    let (mut e, input) = write_setup();
    let (p, _, _) = started(&mut e, input);
    e.store
        .begin_effect(e.store.generation(), &format!("prepare:{}", p.consumer.id))
        .unwrap();
    let cancel = end_input(&e, &p.consumer.id, 1, State::Cancelled);
    call::end(&mut e.store, &actor(), cancel).unwrap();
    let (_, invocation) = call::invocation(&e.store, &e.a, &p.consumer.id).unwrap();
    let mut proof = json!({
        "dispatch":"original",
        "stop_report": {
            "dispatch": { "owner": p.consumer, "spec_digest":invocation.spec.digest, "state":"cancelled" },
            "events":[{"kind":"stopped","source":"adapter_event","payload":{"exit_code":null,"terminal_result_present":false,"requested_stop":true}}]
        },
        "never_started":false
    });
    for field in ["owner", "generation", "spec", "turn", "level", "missing"] {
        let mut wrong = proof.clone();
        match field {
            "owner" => wrong["stop_report"]["dispatch"]["owner"]["id"] = json!("another"),
            "generation" => wrong["stop_report"]["dispatch"]["owner"]["generation"] = json!(2),
            "spec" => wrong["stop_report"]["dispatch"]["spec_digest"] = json!(hash(b"other")),
            "turn" => {
                wrong["stop_report"]["events"][0]["payload"] =
                    json!({"requested_stop":true,"session_closed":true})
            }
            "level" => wrong["stop_report"]["events"][0]["source"] = json!("narrated"),
            "missing" => {
                wrong["stop_report"]["events"] = json!([]);
                // A control-side lookup once saw Prepared. That stale lookup
                // must not replace an Agency report about what stop actually did.
                wrong["never_started"] = json!(true);
            }
            _ => unreachable!(),
        }
        let error = write_cleanup_fixture(&mut e, &invocation, wrong).unwrap_err();
        assert_eq!(error.code, "STOP_PROOF_REQUIRED", "{field}");
        assert!(e.store.list("dispatch_cleanup").unwrap().is_empty());
        assert_eq!(
            repo::changeset::get_change_set(
                &e.store,
                &e.rid,
                &p.write.as_ref().unwrap().lease.pending.change_set_id
            )
            .unwrap()
            .lease
            .state,
            repo::changeset::LeaseState::Revoking
        );
    }
    // A signal-killed process has no numeric exit code. Its native physical exit report
    // still proves it stopped; a turn-interrupted report has no exit_code field at all.
    proof["stop_report"]["events"][0]["payload"]["exit_code"] = Value::Null;
    write_cleanup_fixture(&mut e, &invocation, proof).unwrap();
    assert_eq!(
        repo::changeset::get_change_set(
            &e.store,
            &e.rid,
            &p.write.as_ref().unwrap().lease.pending.change_set_id
        )
        .unwrap()
        .lease
        .state,
        repo::changeset::LeaseState::Revoked
    );
}

fn write_cleanup_fixture(
    e: &mut Env,
    invocation: &call::Invocation,
    proof: Value,
) -> store::Result<Value> {
    let mut actor = chat::owner(&actor(), &e.a).unwrap();
    actor.0.permission_scope.push(Scope::Repo(e.rid.clone()));
    let record = value_record(
        key(
            Scope::Project(e.a.clone()),
            "dispatch_cleanup",
            &invocation.spec.document.owner.id,
        ),
        1,
        &proof,
    )
    .unwrap();
    let command = Command {
        command_id: "proof".into(),
        idempotency_key: "proof".into(),
        actor: actor.0.clone(),
        target: record.key.clone(),
        expected: Expected::Absent,
        binding: reference(&record),
        operation: "fixture.cleanup".into(),
        input: proof.clone(),
        input_digest: Command::digest_input("fixture.cleanup", &proof).unwrap(),
    };
    e.store
        .submit(e.store.generation(), &actor, &command, None, |tx| {
            tx.put(&record)?;
            call::confirm_write_stop(tx, invocation, &reference(&record))?;
            Ok(json!({}))
        })
}

#[test]
fn agency_confirmed_never_started_write_can_release_its_lease() {
    let (mut e, input) = write_setup();
    let (preview, _, _) = started(&mut e, input);
    e.store
        .begin_effect(
            e.store.generation(),
            &format!("prepare:{}", preview.consumer.id),
        )
        .unwrap();
    let cancel = end_input(&e, &preview.consumer.id, 1, State::Cancelled);
    call::end(&mut e.store, &actor(), cancel).unwrap();
    let (_, invocation) = call::invocation(&e.store, &e.a, &preview.consumer.id).unwrap();
    let proof = json!({"stop_report":{
        "dispatch":{"owner":preview.consumer,"spec_digest":invocation.spec.digest,"state":"cancelled"},
        "events":[{"kind":"stopped","source":"adapter_event","payload":{"never_started":true}}]
    }});
    write_cleanup_fixture(&mut e, &invocation, proof).unwrap();
    let set = preview.write.unwrap().lease.pending;
    assert_eq!(
        repo::changeset::get_change_set(&e.store, &e.rid, &set.change_set_id)
            .unwrap()
            .lease
            .state,
        repo::changeset::LeaseState::Revoked
    );
}

#[test]
fn selected_review_version_is_exact_repo_scoped_and_old_inputs_keep_their_encoding() {
    let (mut e, mut input) = setup();
    let mut scoped = actor();
    scoped.0.permission_scope.push(Scope::Repo(e.rid.clone()));
    let set = repo::changeset::open_change_set(
        &mut e.store,
        &scoped,
        &e.rid,
        1,
        &"a".repeat(40),
        "review",
        &repo::changeset::ProducerRef::HumanCommand {
            command_id: "open".into(),
        },
    )
    .unwrap();
    let revision = repo::changeset::admit(
        &mut e.store,
        &scoped,
        repo::changeset::Seal {
            association_key: "human-review".into(),
            change_set_id: set.change_set_id,
            change_set_version: 1,
            lease: None,
            base_commit_sha: "a".repeat(40),
            result_tree_sha: "b".repeat(40),
            result_commit_sha: None,
            parent_revision_id: None,
            producer_ref: repo::changeset::ProducerRef::HumanCommand {
                command_id: "human-review".into(),
            },
        },
        repo::changeset::OwnerGate::Active,
    )
    .unwrap();
    let original = serde_json::to_value(&input).unwrap();
    assert!(original.get("review_change_set_revision").is_none());
    assert!(original.get("write").is_none());
    let old: call::Input = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(serde_json::to_value(old).unwrap(), original);
    let selected = Reference {
        key: key(
            Scope::Repo(e.rid.clone()),
            "changeset_revision",
            &revision.change_set_revision_id,
        ),
        version: Version::State(1),
    };
    input.review_change_set_revision = Some(selected.clone());
    assert_eq!(
        call::prepare(&e.store, &actor(), input.clone(), NOW)
            .unwrap()
            .input
            .review_change_set_revision,
        Some(selected.clone())
    );
    for field in ["repo", "kind", "version"] {
        let mut wrong = input.clone();
        let r = wrong.review_change_set_revision.as_mut().unwrap();
        match field {
            "repo" => r.key.scope = Scope::Repo("other".into()),
            "kind" => r.key.kind = "integration_receipt".into(),
            "version" => r.version = Version::State(2),
            _ => unreachable!(),
        }
        assert_eq!(
            call::prepare(&e.store, &actor(), wrong, NOW)
                .unwrap_err()
                .code,
            if field == "version" {
                "VERSION_CONFLICT"
            } else {
                "REVIEW_VERSION_MISMATCH"
            }
        );
    }
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
    let mut sources = vec![
        (request_ref.clone(), request.clone()),
        (skill.clone(), skill_bytes.clone()),
    ];
    if let Some(write) = &preview.write {
        let bytes = write.context_bytes().unwrap();
        sources.push((
            frozen_ref(
                &format!("write-boundary/{}", preview.consumer.id),
                &hash(&bytes),
            ),
            bytes,
        ));
    }
    let source_ids = sources
        .iter()
        .map(|(r, _)| r.id.clone())
        .collect::<Vec<_>>();
    let permission = context::permission_digest(&source_ids);
    let policy = frozen_ref("mechanical", &hash(b"policy"));
    let manifest = Sealed::new(Manifest {
        id: format!("manifest-{}", preview.consumer.id),
        purpose: "research".into(),
        scope: format!("project {}", preview.consumer.project),
        parent: None,
        sources: sources.iter().map(|(r, _)| r.clone()).collect(),
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
    let entries = sources
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

fn replace_record(e: &mut Env, record: Record) {
    let scoped = chat::owner(&actor(), &e.a).unwrap();
    replace_record_as(e, record, scoped);
}

fn replace_record_as(e: &mut Env, record: Record, scoped: TrustedActor) {
    let old = e.store.get(&record.key).unwrap();
    let input = serde_json::to_value(&record).unwrap();
    let command = Command {
        command_id: format!("fixture:{}:{}", record.key.kind, record.version),
        idempotency_key: format!("fixture:{}:{}", record.key.kind, record.version),
        actor: scoped.0.clone(),
        target: record.key.clone(),
        expected: old.as_ref().map_or(Expected::Absent, |r| {
            Expected::Exact(Version::State(r.version))
        }),
        binding: old.as_ref().map_or_else(|| reference(&record), reference),
        operation: "fixture".into(),
        input_digest: Command::digest_input("fixture", &input).unwrap(),
        input,
    };
    e.store
        .submit(e.store.generation(), &scoped, &command, None, |tx| {
            tx.put(&record)?;
            Ok(json!({}))
        })
        .unwrap();
}

#[test]
fn invocation_terminal_state_alone_invalidates_an_otherwise_current_authorization() {
    let (mut e, input) = setup();
    let (p, _, owner) = started(&mut e, input);
    let record = value_record(
        call::state_key(&e.a, &p.consumer.id),
        2,
        &call::Lifecycle {
            state: State::Completed,
            reason: None,
        },
    )
    .unwrap();
    replace_record(&mut e, record);
    let (root, invocation) = call::invocation(&e.store, &e.a, &p.consumer.id).unwrap();
    assert_eq!(reference(&root), owner);
    assert!(invocation.authorization.valid);
    assert!(invocation.spec.document.deadline_ms > NOW);
    assert!(!call::current_authorization(&e.store, &owner, NOW).unwrap());
}

#[test]
fn invocation_topic_without_confirmed_brief_and_inactive_room_fail_before_dispatch() {
    for (kind, state, code) in [
        (store::RoomKind::Topic, RoomState::Active, "BRIEF_REQUIRED"),
        (store::RoomKind::Main, RoomState::Archived, "ROOM_READ_ONLY"),
    ] {
        let (mut e, mut input) = setup();
        let mut room = required(
            &e.store,
            &key(Scope::Project(e.a.clone()), "room", &input.room_id),
        )
        .unwrap();
        if kind == store::RoomKind::Topic {
            let mut binding = chat::room(&e.store, &e.a, &input.room_id).unwrap().1;
            input.room_id = "topic".into();
            binding.id = input.room_id.clone();
            let binding_record = value_record(
                key(Scope::Project(e.a.clone()), "room_binding", &input.room_id),
                1,
                &binding,
            )
            .unwrap();
            replace_record(&mut e, binding_record);
            room.key.id = input.room_id.clone();
            room.version = 1;
        } else {
            room.version += 1;
        }
        room.data = store::RecordData::Room {
            room_kind: kind,
            state,
        };
        room.revision_digest =
            foundation::canonical_json_sha256(&serde_json::to_value(&room.data).unwrap()).unwrap();
        replace_record(&mut e, room);
        let stamp = e.store.read_stamp();
        assert_eq!(
            call::prepare(&e.store, &actor(), input, NOW)
                .unwrap_err()
                .code,
            code
        );
        assert_eq!(e.store.read_stamp(), stamp);
        assert!(e.store.list("room_invocation").unwrap().is_empty());
    }
}

#[test]
fn invocation_topic_requires_the_confirmed_brief_in_actual_delivery() {
    let (mut e, mut input) = setup();
    let (binding, main) = chat::main_binding(&e.store, &e.a).unwrap();
    let body = json!({"body":"topic source","msgtype":"m.text"}).to_string();
    input.room_id = e.create_topic(
        "invoke-topic",
        chat::Origin::Room {
            room_id: main.id,
            binding_version: binding.version,
        },
        vec![chat::SourceText {
            source: chat::Source::Message {
                binding: reference(&binding),
                event_id: "$source".into(),
                content_digest: foundation::bytes_sha256(body.as_bytes()),
            },
            body,
            excerpt: "topic source".into(),
        }],
    );
    let selected = accepted_selection(&mut e, &input.room_id, "topic-agency", true, None);
    input.profile = selected.worker_profiles[0].clone();
    let action = select_action(&e.a, &input.room_id, vec![selected]);
    e.apply("select-topic", action).unwrap();
    let p = call::prepare(&e.store, &actor(), input, NOW).unwrap();
    let mut a = assembly(&p);
    assert_eq!(
        call::start(&mut e.store, &actor(), &p, &a, NOW)
            .unwrap_err()
            .code,
        "CONTEXT_MISMATCH"
    );
    let scoped = chat::owner(&actor(), &e.a).unwrap();
    let bytes = e
        .store
        .read_material(&scoped, p.brief.as_ref().unwrap())
        .unwrap();
    let source = frozen_ref("room-brief/topic", &hash(&bytes));
    let mut manifest = a.manifest.document;
    manifest.sources.push(source.clone());
    manifest.permission_digest = context::permission_digest(
        &manifest
            .sources
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>(),
    );
    a.manifest = Sealed::new(manifest).unwrap();
    let mut bundle = a.bundle.document;
    bundle.manifest = frozen_ref(&a.manifest.document.id, &a.manifest.digest);
    bundle.permission_digest = a.manifest.document.permission_digest.clone();
    bundle.entries.push(Entry {
        source,
        description: "confirmed brief".into(),
        required: true,
        offline_required: true,
        bytes_digest: hash(&bytes),
        delivery: Delivery::Inline { bytes },
    });
    a.bundle = Sealed::new(bundle).unwrap();
    context::save_assembly(&mut e.store, &scoped, &e.a, "save-topic-bundle", &a).unwrap();
    call::start(&mut e.store, &actor(), &p, &a, NOW).unwrap();
}

#[test]
fn invocation_confirmed_activation_requires_original_reducer_not_a_client() {
    let (mut e, input) = setup();
    let (p, _, owner) = started(&mut e, input);
    activate(&mut e, &p);
    let client = chat::owner(&actor(), &e.a).unwrap();
    assert_eq!(
        call::record_started(&mut e.store, &client, &e.a, &p.consumer.id, NOW)
            .unwrap_err()
            .code,
        "PERMISSION_DENIED"
    );
    let mut reducer = client;
    reducer.0.source = ActorSource::InternalReducer;
    reducer.0.authority = Some(owner);
    call::record_started(&mut e.store, &reducer, &e.a, &p.consumer.id, NOW).unwrap();
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
fn invocation_modified_preview_cannot_widen_permissions_or_change_the_profile() {
    for field in ["permissions", "profile"] {
        let (mut e, input) = setup();
        let (p, a) = prepared(&mut e, input);
        let before = e.store.read_stamp();
        let mut value = serde_json::to_value(&p).unwrap();
        match field {
            "permissions" => value["configuration"]["permissions"]
                .as_array_mut()
                .unwrap()
                .push(json!("git.write")),
            "profile" => value["profile"]["digest"] = json!(hash(b"other profile")),
            _ => unreachable!(),
        }
        let modified: call::Preview = serde_json::from_value(value).unwrap();
        assert_eq!(
            call::start(&mut e.store, &actor(), &modified, &a, NOW)
                .unwrap_err()
                .code,
            "VERSION_CONFLICT",
            "{field} changed without changing any dependency version"
        );
        assert_eq!(e.store.read_stamp(), before);
        assert!(e.store.list("room_invocation").unwrap().is_empty());
        assert!(e.store.list("dispatch_intent").unwrap().is_empty());
    }
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
    let mut reducer = chat::owner(&actor(), &e.a).unwrap();
    reducer.0.source = ActorSource::InternalReducer;
    reducer.0.authority = Some(owner.clone());
    call::record_started(&mut e.store, &reducer, &e.a, &p.consumer.id, NOW).unwrap();
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
    call::record_started(&mut e.store, &reducer, &e.a, &p.consumer.id, NOW).unwrap();
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
        let client = chat::owner(&actor(), &e.a).unwrap();
        assert_eq!(
            call::end(&mut e.store, &client, end.clone())
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

fn running(e: &mut Env, input: call::Input) -> (call::Preview, Reference, TrustedActor, Record) {
    let (preview, _, owner) = started(e, input);
    activate(e, &preview);
    let mut reducer = chat::owner(&actor(), &e.a).unwrap();
    reducer.0.source = ActorSource::InternalReducer;
    reducer.0.authority = Some(owner.clone());
    call::record_started(&mut e.store, &reducer, &e.a, &preview.consumer.id, NOW).unwrap();
    let dispatch = required(
        &e.store,
        &key(
            owner.key.scope.clone(),
            "dispatch",
            &format!("dispatch-{}", preview.consumer.id),
        ),
    )
    .unwrap();
    (preview, owner, reducer, dispatch)
}
fn preserved(e: &mut Env, dispatch: &Record, schema: &str, sequence: u64) -> Record {
    let bytes = b"an exact read-only answer".to_vec();
    preserved_bytes(e, dispatch, schema, sequence, bytes)
}

fn preserved_bytes(
    e: &mut Env,
    dispatch: &Record,
    schema: &str,
    sequence: u64,
    bytes: Vec<u8>,
) -> Record {
    let d: Dispatch = participant::decode(dispatch).unwrap();
    let digest = hash(&bytes);
    let id = format!("proposal-{sequence}");
    let proposal = agency_proto::Proposal {
        header: agency_proto::ProposalHeader {
            proposal_id: id.clone(),
            owner: d.owner.clone(),
            dispatch: d.reference.clone(),
            spec_digest: d.spec_digest.clone(),
            bundle_digest: d.bundle_digest.clone(),
            binding: d.binding.clone(),
            producer_sequence: sequence,
            idempotency_key: id.clone(),
        },
        schema: schema.into(),
        output: bytes,
        content_digest: digest.clone(),
        evidence: EvidenceLevel::Narrated,
        preserved: false,
        outputs: vec![agency_proto::ProposalOutput {
            schema: schema.into(),
            content_digest: digest.clone(),
            candidate: FrozenRef {
                id: id.clone(),
                revision: "1".into(),
                digest,
            },
            owner: d.owner,
            dispatch: d.reference.clone(),
            authorization: FrozenRef {
                id: d.reference,
                revision: "1".into(),
                digest: d.spec_digest,
            },
        }],
    };
    let scoped = chat::owner(&actor(), &e.a).unwrap();
    participant::preserve_proposal(&mut e.store, &scoped, dispatch, &proposal).unwrap();
    required(
        &e.store,
        &key(dispatch.key.scope.clone(), "proposal_inbox", &id),
    )
    .unwrap()
}

#[test]
fn writer_cannot_activate_without_delivered_exact_changeset_boundary() {
    let (mut e, input) = write_setup();
    let (p, mut a) = prepared(&mut e, input);
    a.bundle
        .document
        .entries
        .retain(|entry| !entry.source.id.starts_with("write-boundary/"));
    a.bundle = Sealed::new(a.bundle.document).unwrap();
    assert_eq!(
        call::start(&mut e.store, &actor(), &p, &a, NOW)
            .unwrap_err()
            .code,
        "CONTEXT_MISMATCH"
    );
    assert!(e.store.list("changeset").unwrap().is_empty());
    assert!(e.store.list("room_invocation").unwrap().is_empty());
}

#[test]
fn writing_call_cannot_be_completed_as_a_read_only_answer() {
    let (mut e, input) = write_setup();
    let (preview, _, reducer, dispatch) = running(&mut e, input);
    let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
    assert_eq!(
        call::admit_result(
            &mut e.store,
            &reducer,
            &e.a,
            &preview.consumer.id,
            &inbox,
            NOW
        )
        .unwrap_err()
        .code,
        "CHANGESET_RESULT_REQUIRED"
    );
    assert!(e.store.list("invocation_result").unwrap().is_empty());
    assert!(e.store.list("changeset_revision").unwrap().is_empty());
    assert_eq!(
        call::lifecycle(&e.store, &e.a, &preview.consumer.id)
            .unwrap()
            .1
            .state,
        State::Running
    );
    let write = preview.write.unwrap().lease.pending;
    assert_eq!(
        repo::changeset::get_change_set(&e.store, &write.repo_id, &write.change_set_id)
            .unwrap()
            .lease
            .state,
        repo::changeset::LeaseState::Active
    );
}

fn replace_proposal(e: &mut Env, inbox: &Record, proposal: &agency_proto::Proposal) -> Record {
    let mut changed = value_record(inbox.key.clone(), inbox.version + 1, proposal).unwrap();
    changed.sources = inbox.sources.clone();
    changed.materials = inbox.materials.clone();
    replace_record(e, changed.clone());
    changed
}

fn assert_admission_rejected(
    e: &mut Env,
    preview: &call::Preview,
    reducer: &TrustedActor,
    inbox: &Record,
    code: &str,
) -> store::StoreError {
    let root = call::invocation(&e.store, &e.a, &preview.consumer.id)
        .unwrap()
        .0;
    let state = call::lifecycle(&e.store, &e.a, &preview.consumer.id)
        .unwrap()
        .0;
    let history_len = e.store.versions(&state.key).unwrap().len();
    let effects = e.store.pending_effects().unwrap();
    let scoped = chat::owner(&actor(), &e.a).unwrap();
    let bytes = e.store.read_material(&scoped, &inbox.materials[0]).unwrap();
    assert!(e.store.list("invocation_result").unwrap().is_empty());

    let error = call::admit_result(
        &mut e.store,
        reducer,
        &e.a,
        &preview.consumer.id,
        inbox,
        NOW,
    )
    .unwrap_err();
    assert_eq!(error.code, code);
    for expected in [root, state] {
        assert_eq!(
            serde_json::to_value(e.store.get(&expected.key).unwrap().unwrap()).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }
    assert_eq!(
        e.store
            .versions(&call::state_key(&e.a, &preview.consumer.id))
            .unwrap()
            .len(),
        history_len
    );
    assert!(e.store.list("invocation_result").unwrap().is_empty());
    assert_eq!(e.store.pending_effects().unwrap(), effects);
    assert_eq!(
        e.store.read_material(&scoped, &inbox.materials[0]).unwrap(),
        bytes
    );
    error
}

#[test]
fn invocation_result_rejects_pending_lifecycle() {
    let (mut e, input) = setup();
    let (p, owner, reducer, dispatch) = running(&mut e, input);
    let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
    let state = value_record(
        call::state_key(&e.a, &p.consumer.id),
        3,
        &call::Lifecycle {
            state: State::Pending,
            reason: None,
        },
    )
    .unwrap();
    replace_record(&mut e, state);
    assert!(call::current_authorization(&e.store, &owner, NOW).unwrap());
    assert_admission_rejected(&mut e, &p, &reducer, &inbox, "PROPOSAL_MISMATCH");
}

#[test]
fn invocation_result_rejects_changed_proposal_spec_digest() {
    let (mut e, input) = setup();
    let (p, _, reducer, dispatch) = running(&mut e, input);
    let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
    let mut proposal: agency_proto::Proposal = participant::decode(&inbox).unwrap();
    proposal.header.spec_digest = hash(b"another frozen Spec");
    let changed = replace_proposal(&mut e, &inbox, &proposal);
    assert_admission_rejected(&mut e, &p, &reducer, &changed, "PROPOSAL_MISMATCH");
}

#[test]
fn invocation_result_rejects_output_different_from_preserved_bytes() {
    let (mut e, input) = setup();
    let (p, _, reducer, dispatch) = running(&mut e, input);
    let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
    let mut proposal: agency_proto::Proposal = participant::decode(&inbox).unwrap();
    proposal.output = b"different read-only answer".to_vec();
    proposal.content_digest = hash(&proposal.output);
    proposal.outputs[0].content_digest = proposal.content_digest.clone();
    proposal.outputs[0].candidate.digest = proposal.content_digest.clone();
    let changed = replace_proposal(&mut e, &inbox, &proposal);
    assert_admission_rejected(&mut e, &p, &reducer, &changed, "PROPOSAL_MISMATCH");
}

#[test]
fn invocation_result_rejects_output_hash_mismatch() {
    let (mut e, input) = setup();
    let (p, _, reducer, dispatch) = running(&mut e, input);
    let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
    let mut proposal: agency_proto::Proposal = participant::decode(&inbox).unwrap();
    proposal.content_digest = hash(b"not the output");
    proposal.outputs[0].content_digest = proposal.content_digest.clone();
    proposal.outputs[0].candidate.digest = proposal.content_digest.clone();
    let changed = replace_proposal(&mut e, &inbox, &proposal);
    assert_admission_rejected(&mut e, &p, &reducer, &changed, "PROPOSAL_MISMATCH");
}

#[test]
fn invocation_result_rejects_delivery_outside_original_room() {
    let (mut e, input) = setup();
    let (p, _, reducer, dispatch) = running(&mut e, input);
    let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
    let (root, mut invocation) = call::invocation(&e.store, &e.a, &p.consumer.id).unwrap();
    // Change just this target: every Proposal identity and digest still matches.
    invocation.spec.document.delivery_scope = vec!["another-room".into()];
    let mut changed = value_record(root.key.clone(), root.version + 1, &invocation).unwrap();
    changed.sources = root.sources;
    replace_record(&mut e, changed);
    assert_admission_rejected(&mut e, &p, &reducer, &inbox, "PROPOSAL_MISMATCH");
}

#[test]
fn invocation_result_rejects_multiple_outputs() {
    let (mut e, input) = setup();
    let (p, _, reducer, dispatch) = running(&mut e, input);
    let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
    let mut proposal: agency_proto::Proposal = participant::decode(&inbox).unwrap();
    proposal.outputs.push(proposal.outputs[0].clone());
    let changed = replace_proposal(&mut e, &inbox, &proposal);
    assert_admission_rejected(&mut e, &p, &reducer, &changed, "PROPOSAL_MISMATCH");
}

#[test]
fn invocation_result_rejects_non_utf8_answer() {
    let (mut e, input) = setup();
    let (p, _, reducer, dispatch) = running(&mut e, input);
    let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
    let mut proposal: agency_proto::Proposal = participant::decode(&inbox).unwrap();
    proposal.header.proposal_id = "invalid-utf8".into();
    proposal.header.idempotency_key = "invalid-utf8".into();
    proposal.output = vec![0xff];
    proposal.content_digest = hash(&proposal.output);
    proposal.outputs[0].candidate.id = "invalid-utf8".into();
    proposal.outputs[0].candidate.digest = proposal.content_digest.clone();
    proposal.outputs[0].content_digest = proposal.content_digest.clone();
    participant::preserve_proposal(&mut e.store, &reducer, &dispatch, &proposal).unwrap();
    let changed = required(
        &e.store,
        &key(inbox.key.scope, "proposal_inbox", "invalid-utf8"),
    )
    .unwrap();
    assert_admission_rejected(&mut e, &p, &reducer, &changed, "PROPOSAL_MISMATCH");
}

#[test]
fn invocation_result_rechecks_stale_inbox_inside_transaction() {
    let (mut e, input) = setup();
    let (p, _, reducer, dispatch) = running(&mut e, input);
    let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
    let mut changed = inbox.clone();
    changed.version += 1;
    replace_record(&mut e, changed);
    // The passed snapshot still satisfies every Proposal check outside the transaction.
    assert_admission_rejected(&mut e, &p, &reducer, &inbox, "VERSION_CONFLICT");
}

#[test]
fn invocation_result_rejects_original_reducer_without_project_scope() {
    let (mut e, input) = setup();
    let (p, _, mut reducer, dispatch) = running(&mut e, input);
    let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
    reducer.0.permission_scope = vec![Scope::Project(e.b.clone())];
    let error = assert_admission_rejected(&mut e, &p, &reducer, &inbox, "PERMISSION_DENIED");
    // A later Store material/command denial must not stand in for this reducer guard.
    assert_eq!(error.message, "original Invocation reducer required");
}

#[test]
fn invocation_admits_exact_answer_and_projection_atomically_and_terminal_guard_is_independent() {
    let (mut e, input) = setup();
    let (p, owner, reducer, dispatch) = running(&mut e, input);
    let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
    let result =
        call::admit_result(&mut e.store, &reducer, &e.a, &p.consumer.id, &inbox, NOW).unwrap();
    assert_eq!(result["state"], "completed");
    assert_eq!(
        call::lifecycle(&e.store, &e.a, &p.consumer.id)
            .unwrap()
            .1
            .state,
        State::Completed
    );
    let (root, call) = call::invocation(&e.store, &e.a, &p.consumer.id).unwrap();
    assert_eq!(reference(&root), owner);
    assert!(
        call.authorization.valid,
        "root not explicitly revoked; terminal guard alone removes execution authority"
    );
    assert!(!call::current_authorization(&e.store, &owner, NOW).unwrap());
    let (effect, state) = e
        .store
        .effect(result["projection_effect"].as_str().unwrap())
        .unwrap();
    assert_eq!(state, EffectState::Pending);
    assert_eq!(effect.input["body"], "an exact read-only answer");
    assert_eq!(
        effect.target,
        chat::room(&e.store, &e.a, &p.input.room_id)
            .unwrap()
            .1
            .matrix_room_id
            .unwrap()
    );
    assert_eq!(
        call::admit_result(&mut e.store, &reducer, &e.a, &p.consumer.id, &inbox, NOW).unwrap(),
        result
    );
    assert_eq!(e.store.list("invocation_result").unwrap().len(), 1);
    assert!(e.store.list("task").unwrap().is_empty());
    let mut retry = p.input.clone();
    retry.key = "after-completed".into();
    retry.retry_of = Some(owner);
    let next = call::prepare(&e.store, &actor(), retry, NOW).unwrap();
    assert_ne!(next.consumer.id, p.consumer.id);
}

#[test]
fn invocation_rejects_unsupported_schema_but_can_admit_a_subsequent_good_answer() {
    let (mut e, input) = setup();
    let (p, _, reducer, dispatch) = running(&mut e, input);
    let bad = preserved(&mut e, &dispatch, "task.complete.v1", 1);
    let stamp = e.store.read_stamp();
    assert_eq!(
        call::admit_result(&mut e.store, &reducer, &e.a, &p.consumer.id, &bad, NOW)
            .unwrap_err()
            .code,
        "PROPOSAL_MISMATCH"
    );
    assert_eq!(e.store.read_stamp(), stamp);
    assert_eq!(
        call::lifecycle(&e.store, &e.a, &p.consumer.id)
            .unwrap()
            .1
            .state,
        State::Running
    );
    let good = preserved(&mut e, &dispatch, "claude.turn.v1", 2);
    call::admit_result(&mut e.store, &reducer, &e.a, &p.consumer.id, &good, NOW).unwrap();
    assert_eq!(e.store.list("proposal_inbox").unwrap().len(), 2);
}

#[test]
fn invocation_projection_conflict_rolls_back_result_and_completion_but_keeps_preserved_bytes() {
    let (mut e, input) = setup();
    let (p, owner, reducer, dispatch) = running(&mut e, input);
    let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
    let effect_id = format!("projection:{}:proposal-1", p.consumer.id);
    let input = json!({"occupied":true});
    let command = Command {
        command_id: "occupy-projection".into(),
        idempotency_key: "occupy-projection".into(),
        actor: reducer.0.clone(),
        target: key(owner.key.scope.clone(), "fixture", "occupy-projection"),
        expected: Expected::Absent,
        binding: owner.clone(),
        operation: "fixture".into(),
        input_digest: Command::digest_input("fixture", &input).unwrap(),
        input: input.clone(),
    };
    e.store
        .submit(e.store.generation(), &reducer, &command, None, |tx| {
            tx.enqueue_effect(&EffectIntent {
                intent_id: "unrelated-projection".into(),
                owner: owner.clone(),
                binding: owner.clone(),
                operation: "fixture".into(),
                target: "fixture".into(),
                conflict_scope: effect_id.clone(),
                permission_scope: owner.key.scope.clone(),
                input_digest: Command::digest_input("fixture", &input).unwrap(),
                input,
                idempotency_key: "unrelated-projection".into(),
            })?;
            Ok(json!({}))
        })
        .unwrap();
    let before = call::lifecycle(&e.store, &e.a, &p.consumer.id).unwrap().0;
    assert_eq!(
        call::admit_result(&mut e.store, &reducer, &e.a, &p.consumer.id, &inbox, NOW)
            .unwrap_err()
            .code,
        "EFFECT_CONFLICT"
    );
    assert_eq!(
        serde_json::to_value(call::lifecycle(&e.store, &e.a, &p.consumer.id).unwrap().0).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    assert_eq!(e.store.versions(&before.key).unwrap().len(), 2);
    assert!(e.store.list("invocation_result").unwrap().is_empty());
    assert!(!e.store.has_effect(&effect_id).unwrap());
    assert_eq!(
        call::lifecycle(&e.store, &e.a, &p.consumer.id)
            .unwrap()
            .1
            .state,
        State::Running
    );
    assert!(call::current_authorization(&e.store, &owner, NOW).unwrap());
    assert_eq!(
        e.store
            .read_material(&reducer, &inbox.materials[0])
            .unwrap(),
        b"an exact read-only answer"
    );
}

#[test]
fn invocation_prepare_rejected_before_effect_does_not_require_cleanup_for_a_nonexistent_dispatch() {
    let (mut e, input) = setup();
    let (p, _, owner) = started(&mut e, input);
    let effect_id = format!("prepare:{}", p.consumer.id);
    let effect = e
        .store
        .begin_effect(e.store.generation(), &effect_id)
        .unwrap();
    let mut reducer = chat::owner(&actor(), &e.a).unwrap();
    reducer.0.source = ActorSource::InternalReducer;
    reducer.0.authority = Some(owner.clone());
    let input = json!({"no_effect":true});
    let command = Command {
        command_id: "prepare-rejected".into(),
        idempotency_key: "prepare-rejected".into(),
        actor: reducer.0.clone(),
        target: key(owner.key.scope.clone(), "fixture", "prepare-rejected"),
        expected: Expected::Absent,
        binding: effect.binding.clone(),
        operation: "fixture".into(),
        input_digest: Command::digest_input("fixture", &input).unwrap(),
        input,
    };
    e.store
        .submit(e.store.generation(), &reducer, &command, None, |tx| {
            tx.confirm_effect(
                &effect_id,
                &Readback::Rejected {
                    binding: effect.binding.clone(),
                    target: effect.target.clone(),
                    input_digest: effect.input_digest.clone(),
                    result: json!({"rejected_before_effect":true}),
                },
            )?;
            Ok(json!({}))
        })
        .unwrap();
    let end = end_input(&e, &p.consumer.id, 1, State::Failed);
    let result = call::end(&mut e.store, &reducer, end).unwrap();
    assert_eq!(result["cleanup_pending"], false);
    assert!(
        !e.store
            .has_effect(&format!("stop:{}", p.consumer.id))
            .unwrap()
    );
    assert!(!call::current_authorization(&e.store, &owner, NOW).unwrap());
}

#[test]
fn invocation_cancelled_or_expired_answer_is_preserved_but_never_admitted_or_projected() {
    for cancel in [false, true] {
        let (mut e, input) = setup();
        let (p, _, reducer, dispatch) = running(&mut e, input);
        let inbox = preserved(&mut e, &dispatch, "adapter.stdout.v1", 1);
        let now = if cancel {
            let end = end_input(&e, &p.consumer.id, 2, State::Cancelled);
            call::end(&mut e.store, &actor(), end).unwrap();
            NOW
        } else {
            p.input.deadline_ms
        };
        assert_eq!(
            call::admit_result(&mut e.store, &reducer, &e.a, &p.consumer.id, &inbox, now)
                .unwrap_err()
                .code,
            "OWNER_STALE"
        );
        assert_eq!(e.store.list("proposal_inbox").unwrap().len(), 1);
        assert!(e.store.list("invocation_result").unwrap().is_empty());
        assert!(
            !e.store
                .pending_effects()
                .unwrap()
                .iter()
                .any(|id| id.starts_with("projection:"))
        );
    }
}

#[test]
fn invocation_dispatch_plan_uses_owner_key_and_rejects_a_second_key_without_scanning_unrelated_records()
 {
    let (mut e, input) = setup();
    let (p, assembly, owner) = started(&mut e, input);
    let (root, call) = call::invocation(&e.store, &e.a, &p.consumer.id).unwrap();
    // A malformed unrelated record must not be decoded during an exact lookup.
    let unrelated = value_record(
        key(Scope::Project(e.a.clone()), "dispatch_intent", "unrelated"),
        1,
        &json!({"not":"a Spec"}),
    )
    .unwrap();
    replace_record(&mut e, unrelated);
    assert!(
        participant::dispatch::plan(&e.store, &owner.key.id, &root, &call.spec, &assembly.bundle)
            .is_ok()
    );
    assert_eq!(
        participant::dispatch::plan(&e.store, "second-key", &root, &call.spec, &assembly.bundle)
            .err()
            .unwrap()
            .code,
        "DISPATCH_EXISTS"
    );
}

fn write_result(
    e: &mut Env,
    p: &call::Preview,
    dispatch: &Record,
) -> (Record, repo::changeset::Seal) {
    write_result_at(
        e,
        p,
        dispatch,
        repo::changeset::OutputLocation::Commit {
            repo_path: "/operation-input-only".into(),
            commit_sha: "c".repeat(40),
        },
    )
}

fn write_result_at(
    e: &mut Env,
    p: &call::Preview,
    dispatch: &Record,
    location: repo::changeset::OutputLocation,
) -> (Record, repo::changeset::Seal) {
    use repo::changeset::*;
    let set = &p.write.as_ref().unwrap().lease.pending;
    let lease = LeaseRef {
        lease_id: set.lease.lease_id.clone(),
        generation: set.lease.generation,
    };
    let output = repo::changeset::Output {
        change_set_id: set.change_set_id.clone(),
        lease: lease.clone(),
        base_commit_sha: set.baseline_commit.clone(),
        parent_revision_id: None,
        location,
    };
    let inbox = preserved_bytes(
        e,
        dispatch,
        OUTPUT_SCHEMA,
        1,
        serde_json::to_vec(&output).unwrap(),
    );
    let proposal: agency_proto::Proposal = participant::decode(&inbox).unwrap();
    let seal = Seal {
        association_key: proposal.header.proposal_id,
        change_set_id: set.change_set_id.clone(),
        change_set_version: set.version,
        lease: Some(lease),
        base_commit_sha: set.baseline_commit.clone(),
        result_tree_sha: "b".repeat(40),
        result_commit_sha: Some("c".repeat(40)),
        parent_revision_id: None,
        producer_ref: ProducerRef::Invocation {
            invocation_id: p.consumer.id.clone(),
            invocation_version: p.consumer.generation,
        },
    };
    (inbox, seal)
}

#[test]
fn unchanged_write_result_is_admitted_without_revision_or_publication_and_replays_once() {
    let (mut e, input) = write_setup();
    let (p, _, reducer, dispatch) = running(&mut e, input);
    let (inbox, seal) = write_result_at(
        &mut e,
        &p,
        &dispatch,
        repo::changeset::OutputLocation::NoChanges {
            repo_path: "/operation-input-only".into(),
        },
    );
    let result = call::admit_unchanged_result(
        &mut e.store,
        &reducer,
        &e.a,
        &p.consumer.id,
        &inbox,
        (&seal, &seal.result_tree_sha),
        NOW,
    )
    .unwrap();
    assert_eq!(result["state"], "completed");
    assert_eq!(result["no_changes"], true);
    assert!(result["change_set_revision"].is_null());
    assert!(e.store.list("changeset_revision").unwrap().is_empty());
    assert!(e.store.list(repo::review::INTENT_KIND).unwrap().is_empty());
    assert_eq!(e.store.list("invocation_result").unwrap().len(), 1);
    let saved: Value = decode(&e.store.list("invocation_result").unwrap()[0]).unwrap();
    assert_eq!(saved["no_changes"], true);
    assert!(saved["change_set_revision"].is_null());
    let set = repo::changeset::get_change_set(&e.store, &e.rid, &seal.change_set_id).unwrap();
    assert_eq!(set.lease.state, repo::changeset::LeaseState::Revoking);
    assert!(
        e.store
            .has_effect(&format!("stop:{}", p.consumer.id))
            .unwrap()
    );
    assert_eq!(
        call::admit_unchanged_result(
            &mut e.store,
            &reducer,
            &e.a,
            &p.consumer.id,
            &inbox,
            (&seal, &seal.result_tree_sha),
            NOW + 1
        )
        .unwrap(),
        result
    );
    assert_eq!(e.store.list("invocation_result").unwrap().len(), 1);
}

#[test]
fn unchanged_write_result_rejects_different_tree_unverified_claim_and_stale_lease() {
    for bad in ["tree", "no-readback", "lease"] {
        let (mut e, input) = write_setup();
        let (p, _, reducer, dispatch) = running(&mut e, input);
        let (inbox, mut seal) = write_result_at(
            &mut e,
            &p,
            &dispatch,
            repo::changeset::OutputLocation::NoChanges {
                repo_path: "/operation-input-only".into(),
            },
        );
        if bad == "lease" {
            seal.lease.as_mut().unwrap().generation += 1;
        }
        let stamp = e.store.read_stamp();
        let error = if bad == "no-readback" {
            call::admit_sealed_result(
                &mut e.store,
                &reducer,
                &e.a,
                &p.consumer.id,
                &inbox,
                &seal,
                NOW,
            )
            .unwrap_err()
        } else {
            let tree = if bad == "tree" {
                "d".repeat(40)
            } else {
                seal.result_tree_sha.clone()
            };
            call::admit_unchanged_result(
                &mut e.store,
                &reducer,
                &e.a,
                &p.consumer.id,
                &inbox,
                (&seal, &tree),
                NOW,
            )
            .unwrap_err()
        };
        assert_eq!(
            error.code,
            if bad == "lease" {
                "PROPOSAL_MISMATCH"
            } else {
                "NO_CHANGES_MISMATCH"
            }
        );
        assert_eq!(e.store.read_stamp(), stamp);
        assert!(e.store.list("changeset_revision").unwrap().is_empty());
        assert!(e.store.list("invocation_result").unwrap().is_empty());
        assert!(e.store.list(repo::review::INTENT_KIND).unwrap().is_empty());
        assert_eq!(
            call::lifecycle(&e.store, &e.a, &p.consumer.id)
                .unwrap()
                .1
                .state,
            State::Running
        );
    }
}

#[test]
fn write_bundle_freezes_local_copy_and_full_publication_target() {
    let (mut e, input) = write_setup();
    let mut record = required(&e.store, &repo::key(&e.rid)).unwrap();
    let store::RecordData::Repo {
        registration: Some(registration),
        ..
    } = &mut record.data
    else {
        panic!("registration required")
    };
    registration["prepared"]["local"] = json!({"path":"/registered/delivery-copy", "remotes":{}, "refs":{}, "head_branch":"main", "governance_paths":[]});
    record.version += 1;
    let mut owner = actor();
    owner.0.permission_scope.push(Scope::Repo(e.rid.clone()));
    replace_record_as(&mut e, record, owner);
    let (p, assembly) = prepared(&mut e, input);
    let write = p.write.as_ref().unwrap();
    let bytes: Value = serde_json::from_slice(&write.context_bytes().unwrap()).unwrap();
    assert_eq!(bytes["repo_local_path"], "/registered/delivery-copy");
    assert_eq!(bytes["repo_local_machine"], "control");
    assert_eq!(bytes["objective"], p.input.request);
    assert_eq!(bytes["publication_target"]["repo_id"], e.rid);
    assert_eq!(bytes["publication_target"]["target_branch"], "main");
    assert_eq!(
        bytes["publication_target"]["branch_rule"],
        "hctl2/{change_set}"
    );
    assert_eq!(
        bytes["publication_target"]["requires_human_confirmation"],
        true
    );
    assert!(exact_write_entry(&p, &assembly));
    call::start(&mut e.store, &actor(), &p, &assembly, NOW).unwrap();
}

fn exact_write_entry(preview: &call::Preview, assembly: &Assembly) -> bool {
    assembly.bundle.document.entries.iter().any(|entry| {
        entry.source.id == format!("write-boundary/{}", preview.consumer.id)
            && matches!(&entry.delivery, Delivery::Inline { bytes }
            if *bytes == preview.write.as_ref().unwrap().context_bytes().unwrap())
    })
}

#[test]
fn write_result_and_revision_share_admission_and_completion_does_not_prove_writer_stopped() {
    let (mut e, input) = write_setup();
    let (p, owner, reducer, dispatch) = running(&mut e, input);
    let (inbox, seal) = write_result(&mut e, &p, &dispatch);
    let result = call::admit_sealed_result(
        &mut e.store,
        &reducer,
        &e.a,
        &p.consumer.id,
        &inbox,
        &seal,
        NOW,
    )
    .unwrap();
    assert_eq!(result["state"], "completed");
    assert_eq!(e.store.list("changeset_revision").unwrap().len(), 1);
    assert_eq!(e.store.list("invocation_result").unwrap().len(), 1);
    let intents = e.store.list(repo::review::INTENT_KIND).unwrap();
    assert_eq!(intents.len(), 1);
    let publication: repo::review::Intent = decode(&intents[0]).unwrap();
    assert_eq!(
        publication.target.change_set_revision_id,
        result["change_set_revision"]["change_set_revision_id"]
    );
    assert_eq!(
        publication.authorizing_actor.source,
        ActorSource::DirectClient
    );
    assert_eq!(
        publication.policy.policy_id,
        p.write.as_ref().unwrap().review_publish_policy.id
    );
    assert_eq!(
        publication.binding_version,
        p.write.as_ref().unwrap().lease.pending.binding_version
    );
    assert_eq!(
        publication.policy.policy.binding_version,
        publication.binding_version
    );
    assert!(
        e.store
            .has_effect(&format!("stop:{}", p.consumer.id))
            .unwrap()
    );
    let set = repo::changeset::get_change_set(&e.store, &e.rid, &seal.change_set_id).unwrap();
    assert_eq!(set.lease.state, repo::changeset::LeaseState::Revoking);
    assert!(!call::current_authorization(&e.store, &owner, NOW).unwrap());
    let replay = call::admit_sealed_result(
        &mut e.store,
        &reducer,
        &e.a,
        &p.consumer.id,
        &inbox,
        &seal,
        NOW + 1,
    )
    .unwrap();
    assert_eq!(replay, result);
    assert_eq!(e.store.list("changeset_revision").unwrap().len(), 1);
}

#[test]
fn write_result_rejects_each_boundary_change_without_revision_result_or_completion() {
    for bad in [
        "set",
        "lease-id",
        "lease-generation",
        "base",
        "parent",
        "producer",
        "producer-version",
        "association",
        "commit",
    ] {
        let (mut e, input) = write_setup();
        let (p, _, reducer, dispatch) = running(&mut e, input);
        let (inbox, mut seal) = write_result(&mut e, &p, &dispatch);
        match bad {
            "set" => seal.change_set_id = "another".into(),
            "lease-id" => seal.lease.as_mut().unwrap().lease_id = "another".into(),
            "lease-generation" => seal.lease.as_mut().unwrap().generation += 1,
            "base" => seal.base_commit_sha = "d".repeat(40),
            "parent" => seal.parent_revision_id = Some("another".into()),
            "producer" => {
                seal.producer_ref = repo::changeset::ProducerRef::Invocation {
                    invocation_id: "another".into(),
                    invocation_version: 1,
                }
            }
            "producer-version" => {
                seal.producer_ref = repo::changeset::ProducerRef::Invocation {
                    invocation_id: p.consumer.id.clone(),
                    invocation_version: 2,
                }
            }
            "association" => seal.association_key = "another".into(),
            "commit" => seal.result_commit_sha = Some("d".repeat(40)),
            _ => unreachable!(),
        }
        assert_eq!(
            call::admit_sealed_result(
                &mut e.store,
                &reducer,
                &e.a,
                &p.consumer.id,
                &inbox,
                &seal,
                NOW
            )
            .unwrap_err()
            .code,
            "PROPOSAL_MISMATCH",
            "{bad}"
        );
        assert!(
            e.store.list("changeset_revision").unwrap().is_empty(),
            "{bad}"
        );
        assert!(
            e.store.list("invocation_result").unwrap().is_empty(),
            "{bad}"
        );
        assert_eq!(
            call::lifecycle(&e.store, &e.a, &p.consumer.id)
                .unwrap()
                .1
                .state,
            State::Running
        );
    }
}

#[test]
fn cancelling_after_git_seal_but_before_admission_leaves_only_saved_proposal() {
    let (mut e, input) = write_setup();
    let (p, _, reducer, dispatch) = running(&mut e, input);
    let (inbox, seal) = write_result(&mut e, &p, &dispatch);
    let version = call::lifecycle(&e.store, &e.a, &p.consumer.id)
        .unwrap()
        .0
        .version;
    call::end(
        &mut e.store,
        &actor(),
        call::End {
            key: "cancel-after-seal".into(),
            project_id: e.a.clone(),
            invocation_id: p.consumer.id.clone(),
            state_version: version,
            outcome: State::Cancelled,
            reason: "cancel between physical seal and admission".into(),
        },
    )
    .unwrap();
    assert_eq!(
        call::admit_sealed_result(
            &mut e.store,
            &reducer,
            &e.a,
            &p.consumer.id,
            &inbox,
            &seal,
            NOW
        )
        .unwrap_err()
        .code,
        "OWNER_STALE"
    );
    assert!(e.store.list("changeset_revision").unwrap().is_empty());
    assert!(e.store.list("invocation_result").unwrap().is_empty());
    assert_eq!(e.store.list("proposal_inbox").unwrap().len(), 1);
}

#[test]
fn write_projection_conflict_rolls_back_revision_result_completion_and_lease_revocation() {
    let (mut e, input) = write_setup();
    let (p, _, reducer, dispatch) = running(&mut e, input);
    let (inbox, seal) = write_result(&mut e, &p, &dispatch);
    let proposal: agency_proto::Proposal = decode(&inbox).unwrap();
    let effect_id = format!(
        "projection:{}:{}",
        p.consumer.id, proposal.header.proposal_id
    );
    let (room_binding, room) = chat::room(&e.store, &e.a, &p.input.room_id).unwrap();
    let occupied = EffectIntent {
        intent_id: "occupied-write-projection".into(),
        owner: reference(&call::invocation(&e.store, &e.a, &p.consumer.id).unwrap().0),
        binding: reference(&room_binding),
        operation: "invocation.project".into(),
        target: room.matrix_room_id.unwrap(),
        conflict_scope: effect_id,
        permission_scope: Scope::Project(e.a.clone()),
        input: json!({"other":"projection"}),
        input_digest: Command::digest_input("invocation.project", &json!({"other":"projection"}))
            .unwrap(),
        idempotency_key: "occupied".into(),
    };
    let target = key(
        Scope::Project(e.a.clone()),
        "fixture",
        "occupy-write-projection",
    );
    let a = TrustedActor(reducer.0.clone());
    let command = Command {
        command_id: "occupy-write-projection".into(),
        idempotency_key: "occupy-write-projection".into(),
        actor: a.0.clone(),
        target: target.clone(),
        expected: Expected::Absent,
        binding: reference(&dispatch),
        operation: "fixture".into(),
        input: json!({}),
        input_digest: Command::digest_input("fixture", &json!({})).unwrap(),
    };
    e.store
        .submit(e.store.generation(), &a, &command, None, |tx| {
            tx.enqueue_effect(&occupied)?;
            Ok(json!({}))
        })
        .unwrap();
    assert_eq!(
        call::admit_sealed_result(
            &mut e.store,
            &reducer,
            &e.a,
            &p.consumer.id,
            &inbox,
            &seal,
            NOW
        )
        .unwrap_err()
        .code,
        "EFFECT_CONFLICT"
    );
    assert!(e.store.list("changeset_revision").unwrap().is_empty());
    assert!(e.store.list("invocation_result").unwrap().is_empty());
    assert!(e.store.list(repo::review::INTENT_KIND).unwrap().is_empty());
    assert!(
        !e.store
            .pending_effects()
            .unwrap()
            .iter()
            .any(|id| id.starts_with("review-publish:"))
    );
    assert_eq!(
        call::lifecycle(&e.store, &e.a, &p.consumer.id)
            .unwrap()
            .1
            .state,
        State::Running
    );
    let set = &p.write.unwrap().lease.pending;
    assert_eq!(
        repo::changeset::get_change_set(&e.store, &e.rid, &set.change_set_id)
            .unwrap()
            .lease
            .state,
        repo::changeset::LeaseState::Active
    );
}

#[test]
fn sealing_preflight_requires_original_saved_bytes_live_owner_and_exact_write_boundary() {
    let (mut e, input) = write_setup();
    let (p, _, reducer, dispatch) = running(&mut e, input);
    let (inbox, _) = write_result(&mut e, &p, &dispatch);
    let (output, seal, repo) =
        call::sealing_input(&e.store, &reducer, &e.a, &p.consumer.id, &inbox, NOW).unwrap();
    assert_eq!(repo, e.rid);
    assert_eq!(seal.change_set_id, output.change_set_id);
    assert_eq!(
        seal.producer_ref,
        repo::changeset::ProducerRef::Invocation {
            invocation_id: p.consumer.id.clone(),
            invocation_version: 1
        }
    );
    assert_eq!(
        call::sealing_input(&e.store, &actor(), &e.a, &p.consumer.id, &inbox, NOW)
            .unwrap_err()
            .code,
        "PERMISSION_DENIED"
    );
    let mut wrong = inbox.clone();
    let mut proposal: agency_proto::Proposal = decode(&wrong).unwrap();
    proposal.output.push(b' ');
    wrong.data = store::RecordData::Value {
        value: serde_json::to_value(proposal).unwrap(),
    };
    assert_eq!(
        call::sealing_input(&e.store, &reducer, &e.a, &p.consumer.id, &wrong, NOW)
            .unwrap_err()
            .code,
        "PROPOSAL_MISMATCH"
    );
    let state = call::lifecycle(&e.store, &e.a, &p.consumer.id).unwrap().0;
    call::end(
        &mut e.store,
        &actor(),
        call::End {
            key: "cancel-before-git".into(),
            project_id: e.a.clone(),
            invocation_id: p.consumer.id.clone(),
            state_version: state.version,
            outcome: State::Cancelled,
            reason: "stop".into(),
        },
    )
    .unwrap();
    assert_eq!(
        call::sealing_input(&e.store, &reducer, &e.a, &p.consumer.id, &inbox, NOW)
            .unwrap_err()
            .code,
        "OWNER_STALE"
    );
}

#[test]
fn human_seal_is_an_independent_admission_and_replay_keeps_its_first_observation() {
    let (mut e, _) = setup();
    let input = repo::changeset::HumanInput {
        key: "human-seal".into(),
        repo_id: e.rid.clone(),
        change_set_id: None,
        base_commit_sha: "a".repeat(40),
        parent_revision_id: None,
        location: repo::changeset::OutputLocation::Commit {
            repo_path: "/operation-input".into(),
            commit_sha: "c".repeat(40),
        },
    };
    let plan = repo::changeset::prepare_human(&e.store, &actor(), input.clone()).unwrap();
    assert!(e.store.list("changeset").unwrap().is_empty());
    let mut seal = plan.seal_input();
    seal.result_tree_sha = "b".repeat(40);
    seal.result_commit_sha = Some("c".repeat(40));
    let first =
        repo::changeset::admit_human(&mut e.store, &actor(), &plan, &seal, &json!({"observed":1}))
            .unwrap();
    let repeated =
        repo::changeset::admit_human(&mut e.store, &actor(), &plan, &seal, &json!({"observed":2}))
            .unwrap();
    assert_eq!(first, repeated);
    assert_eq!(first["observation"]["observed"], 1);
    assert_eq!(
        first["revision"]["producer_ref"],
        json!({"kind":"human_command","command_id":"changeset:human-seal"})
    );
    assert!(first["seal"]["lease"].is_null());
    assert!(e.store.list("room_invocation").unwrap().is_empty());
    assert_eq!(e.store.list("changeset_revision").unwrap().len(), 1);
    let set =
        repo::changeset::get_change_set(&e.store, &e.rid, &plan.change_set.change_set_id).unwrap();
    assert_eq!(set.lease.state, repo::changeset::LeaseState::Revoked);
    let mut other = input.clone();
    other.base_commit_sha = "d".repeat(40);
    assert_eq!(
        repo::changeset::prepare_human(&e.store, &actor(), other)
            .unwrap_err()
            .code,
        "IDEMPOTENCY_CONFLICT"
    );
    let mut borrowed = seal.clone();
    borrowed.lease = Some(repo::changeset::LeaseRef {
        lease_id: "someone-else".into(),
        generation: 1,
    });
    assert_eq!(
        repo::changeset::admit_human(&mut e.store, &actor(), &plan, &borrowed, &json!({}))
            .unwrap_err()
            .code,
        "PROPOSAL_MISMATCH"
    );
}

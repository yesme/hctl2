//! Integration intents and Receipts without any executor: admission, exclusion, readback.
mod common;
use common::*;
use repo::integration::{self as integ, *};
use repo::{Platform, admit, prepare as prepare_registration};
use serde_json::json;
use std::collections::BTreeMap;
use store::{EffectState, Store};

fn local_repo(store: &mut Store) -> String {
    let prepared = prepare_registration(request(Platform::None), None).unwrap();
    admit(store, &actor(), "register", "register", prepared)
        .unwrap()
        .repo_id
}

fn revision_of(n: u8) -> AdmittedRevision {
    AdmittedRevision {
        change_set_revision_id: format!("rev-{n}"),
        change_set_id: "cs-1".into(),
        parent_revision_id: None,
        base_commit_sha: "b".repeat(40),
        result_tree_sha: format!("{n}").repeat(40),
        producer_ref: json!({"kind":"human_command","command_id":"seal"}),
        review_subject_digest: "d".repeat(64),
    }
}

fn input(repo_id: &str, key: &str, form: Form) -> Input {
    Input {
        key: key.into(),
        repo_id: repo_id.into(),
        change_set_revision_id: "rev-1".into(),
        target_kind: TargetKind::Local,
        target_ref: "refs/heads/main".into(),
        form,
        strategy: Strategy::FastForward,
    }
}

fn observed(head: Option<&str>) -> Observation {
    Observation {
        provider_ref: "/tmp/example".into(),
        head: head.map(str::to_owned),
        protection: None,
        continuity: Some(json!({"git_common_dir": "/tmp/example/.git", "device": 1, "inode": 42})),
    }
}

/// A hosted (random local platform) Repo taken all the way to active, like the registration suite does.
fn hosted_repo(store: &mut Store) -> String {
    use repo::{
        FinishChoice, PlatformObservation, confirm_delivery, confirm_platform,
        effect_id as registration_effect, finish,
    };
    let prepared = prepare_registration(request(Platform::Local), None).unwrap();
    let reg = admit(
        store,
        &actor(),
        "register-hosted",
        "register-hosted",
        prepared,
    )
    .unwrap();
    store
        .begin_effect(
            store.generation(),
            &registration_effect(&reg.repo_id, "platform"),
        )
        .unwrap();
    let observed = PlatformObservation {
        instance: "http://127.0.0.1:3000".into(),
        stable_id: "7".into(),
        full_name: "admin/example".into(),
        clone_url: "http://127.0.0.1:3000/admin/example.git".into(),
        account_id: "1".into(),
        has_issues: true,
        can_write_issues: true,
        credential_ref: String::new(),
    };
    let reg = confirm_platform(store, &reg.repo_id, observed).unwrap();
    store
        .begin_effect(
            store.generation(),
            &registration_effect(&reg.repo_id, "delivery"),
        )
        .unwrap();
    let reg = confirm_delivery(store, &reg.repo_id).unwrap();
    finish(
        store,
        &actor(),
        &reg.repo_id,
        reg.version,
        FinishChoice::Confirm("7"),
        "finish-hosted",
        "finish-hosted",
    )
    .unwrap();
    reg.repo_id
}

/// Rewrite the platform binding's declared capabilities, as a verified adapter would.
fn declare_capabilities(store: &mut Store, repo_id: &str, capabilities: serde_json::Value) {
    use store::{Command, Expected, Record, RecordData, Version};
    let binding = repo::binding(repo_id);
    let current = store.get(&binding.key).unwrap().unwrap();
    let RecordData::Value { mut value } = current.data else {
        panic!("binding")
    };
    value["capabilities"] = capabilities;
    let mut scoped = actor().0;
    scoped
        .permission_scope
        .push(store::Scope::Repo(repo_id.into()));
    let scoped = store::TrustedActor(scoped);
    let cmd = Command {
        command_id: format!("declare:{}:{}", repo_id, current.version + 1),
        idempotency_key: format!("declare:{}:{}", repo_id, current.version + 1),
        actor: scoped.0.clone(),
        target: binding.key.clone(),
        expected: Expected::Exact(Version::State(current.version)),
        binding: binding.clone(),
        input_digest: Command::digest_input("test.declare", &value).unwrap(),
        operation: "test.declare".into(),
        input: value.clone(),
    };
    store
        .submit(store.generation(), &scoped, &cmd, None, |tx| {
            tx.put(&Record {
                key: binding.key.clone(),
                version: current.version + 1,
                revision_digest: foundation::canonical_json_sha256(&value).unwrap(),
                data: RecordData::Value {
                    value: value.clone(),
                },
                sources: Vec::new(),
                materials: Vec::new(),
            })?;
            Ok(json!({}))
        })
        .unwrap();
}

fn success(before: &str, after: &str, tree: &str) -> Outcome {
    Outcome::Succeeded {
        target_head_before: Some(before.into()),
        target_head_after: after.into(),
        integrated_commit: after.into(),
        integrated_tree: Some(tree.into()),
        evidence_level: "hctl2-tool".into(),
        readback: json!({"status":"applied"}),
        observed_at_unix_ms: 1,
    }
}

fn attention(code: &str) -> Attention {
    Attention {
        code: code.into(),
        message: code.to_lowercase(),
        recovery_action: "retry_same_intent".into(),
        details: json!({}),
    }
}

#[test]
fn preview_freezes_source_target_and_form_and_refuses_what_the_repo_cannot_offer() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let repo_id = local_repo(&mut store);
    assert_eq!(
        integ::prepare(
            &store,
            &actor(),
            input(&repo_id, "k", Form::ExpectedHead),
            observed(Some("a"))
        )
        .unwrap_err()
        .code,
        "REVISION_NOT_ADMITTED"
    );
    admit_revision_seam(&mut store, &actor(), &repo_id, &revision_of(1)).unwrap();
    let preview = integ::prepare(
        &store,
        &actor(),
        input(&repo_id, "k", Form::ExpectedHead),
        observed(Some("aaaa")),
    )
    .unwrap();
    assert_eq!(preview.expected_head.as_deref(), Some("aaaa"));
    assert_eq!(preview.source, revision_of(1));
    assert_eq!(preview.target.kind, TargetKind::Local);
    assert_eq!(preview.target.provider_ref, "/tmp/example");
    assert!(preview.target.binding.is_none());
    assert_eq!(preview.platform, Platform::None);
    // Expected-head needs a head to freeze; accept-advance does not.
    assert_eq!(
        integ::prepare(
            &store,
            &actor(),
            input(&repo_id, "k", Form::ExpectedHead),
            observed(None)
        )
        .unwrap_err()
        .code,
        "TARGET_HEAD_UNKNOWN"
    );
    let advance = integ::prepare(
        &store,
        &actor(),
        input(&repo_id, "k", Form::AcceptAdvance),
        observed(None),
    )
    .unwrap();
    assert_eq!(advance.expected_head, None);
    // A Repo without a platform has no platform target.
    let mut platform = input(&repo_id, "k", Form::AcceptAdvance);
    platform.target_kind = TargetKind::Platform;
    assert_eq!(
        integ::prepare(&store, &actor(), platform, observed(Some("aaaa")))
            .unwrap_err()
            .code,
        "PLATFORM_NOT_BOUND"
    );
    let mut bad_ref = input(&repo_id, "k", Form::AcceptAdvance);
    bad_ref.target_ref = "main".into();
    assert_eq!(
        integ::prepare(&store, &actor(), bad_ref, observed(Some("aaaa")))
            .unwrap_err()
            .code,
        "INVALID_INPUT"
    );
}

#[test]
fn submit_is_idempotent_per_key_and_one_unresolved_intent_occupies_a_target() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let repo_id = local_repo(&mut store);
    admit_revision_seam(&mut store, &actor(), &repo_id, &revision_of(1)).unwrap();
    admit_revision_seam(&mut store, &actor(), &repo_id, &revision_of(2)).unwrap();
    let preview = integ::prepare(
        &store,
        &actor(),
        input(&repo_id, "one", Form::ExpectedHead),
        observed(Some("aaaa")),
    )
    .unwrap();
    let first = submit(&mut store, &actor(), "integration:one", preview.clone()).unwrap();
    assert_eq!(first.state, IntentState::Pending);
    assert_eq!(first.version, 1);
    assert_eq!(
        store.effect(&integ::effect_id(&first.intent_id)).unwrap().1,
        EffectState::Pending
    );
    // Same key, same preview: the same intent. Same key, another preview: conflict.
    let again = submit(&mut store, &actor(), "integration:one", preview.clone()).unwrap();
    assert_eq!(again.intent_id, first.intent_id);
    let mut other = preview.clone();
    other.input.change_set_revision_id = "rev-2".into();
    other.source = revision_of(2);
    assert_eq!(
        submit(&mut store, &actor(), "integration:one", other.clone())
            .unwrap_err()
            .code,
        "IDEMPOTENCY_CONFLICT"
    );
    // Another key on the same target while the first is unresolved: busy, in either form.
    other.input.key = "two".into();
    other.input.form = Form::AcceptAdvance;
    other.form = Form::AcceptAdvance;
    other.expected_head = None;
    assert_eq!(
        submit(&mut store, &actor(), "integration:two", other.clone())
            .unwrap_err()
            .code,
        "TARGET_BUSY"
    );
    // Another target is free.
    let mut elsewhere = other.clone();
    elsewhere.input.key = "three".into();
    elsewhere.input.target_ref = "refs/heads/release".into();
    elsewhere.target.target_ref = "refs/heads/release".into();
    let third = submit(&mut store, &actor(), "integration:three", elsewhere).unwrap();
    assert_ne!(third.intent_id, first.intent_id);
    // Once the first reaches a terminal state the target is a new authorization.
    begin(&mut store, &repo_id, &first.intent_id).unwrap();
    let failed = confirm(
        &mut store,
        &repo_id,
        &first.intent_id,
        Outcome::Failed(attention("TARGET_HEAD_MISMATCH")),
    )
    .unwrap();
    assert_eq!(failed.state, IntentState::Failed);
    assert_eq!(failed.receipt_id, None);
    assert_eq!(
        store.effect(&integ::effect_id(&first.intent_id)).unwrap().1,
        EffectState::Rejected
    );
    assert!(store.list(RECEIPT_KIND).unwrap().is_empty());
    let second = submit(&mut store, &actor(), "integration:two", other).unwrap();
    assert_eq!(second.state, IntentState::Pending);
    assert_eq!(integ::list(&store, &repo_id).unwrap().len(), 3);
    assert_eq!(open(&store).unwrap().len(), 2);
}

#[test]
fn only_a_confirming_readback_writes_the_one_receipt_in_the_same_transaction() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let repo_id = local_repo(&mut store);
    admit_revision_seam(&mut store, &actor(), &repo_id, &revision_of(1)).unwrap();
    let preview = integ::prepare(
        &store,
        &actor(),
        input(&repo_id, "one", Form::ExpectedHead),
        observed(Some("aaaa")),
    )
    .unwrap();
    let intent = submit(&mut store, &actor(), "integration:one", preview).unwrap();
    let id = intent.intent_id.clone();
    // Confirming before any attempt is a programming error, not a result.
    assert!(
        confirm(
            &mut store,
            &repo_id,
            &id,
            success("aaaa", "cccc", &"1".repeat(40))
        )
        .is_err()
    );
    let (_, state) = begin(&mut store, &repo_id, &id).unwrap();
    assert_eq!(state, EffectState::Pending);
    assert_eq!(
        store.effect(&integ::effect_id(&id)).unwrap().1,
        EffectState::Unknown
    );
    // A retryable rejection keeps the intent and its conflict scope; the human acts, then retries.
    let waiting = confirm(
        &mut store,
        &repo_id,
        &id,
        Outcome::Attention(attention("TARGET_CHECKED_OUT")),
    )
    .unwrap();
    assert_eq!(waiting.state, IntentState::Unknown);
    assert_eq!(waiting.attempts, 1);
    assert_eq!(
        waiting.attention.as_ref().unwrap().code,
        "TARGET_CHECKED_OUT"
    );
    let (_, state) = begin(&mut store, &repo_id, &id).unwrap();
    assert_eq!(state, EffectState::Unknown);
    // Readback that contradicts the frozen expected head is refused; nothing is signed.
    assert_eq!(
        confirm(
            &mut store,
            &repo_id,
            &id,
            success("zzzz", "cccc", &"1".repeat(40))
        )
        .unwrap_err()
        .code,
        "READBACK_MISMATCH"
    );
    // Fast-forward must land exactly the admitted tree.
    assert_eq!(
        confirm(
            &mut store,
            &repo_id,
            &id,
            success("aaaa", "cccc", &"9".repeat(40))
        )
        .unwrap_err()
        .code,
        "READBACK_MISMATCH"
    );
    assert!(store.list(RECEIPT_KIND).unwrap().is_empty());
    let done = confirm(
        &mut store,
        &repo_id,
        &id,
        success("aaaa", "cccc", &"1".repeat(40)),
    )
    .unwrap();
    assert_eq!(done.state, IntentState::Succeeded);
    assert_eq!(done.attempts, 2);
    assert_eq!(done.attention, None);
    let receipt_id = done.receipt_id.clone().unwrap();
    let receipt = receipt(&store, &repo_id, &receipt_id).unwrap();
    assert_eq!(receipt.intent_id, id);
    assert_eq!(receipt.target_head_after, "cccc");
    assert_eq!(
        receipt.integrated_tree.as_deref(),
        Some("1".repeat(40).as_str())
    );
    assert_eq!(receipt.evidence_level, "hctl2-tool");
    assert_eq!(
        store.effect(&integ::effect_id(&id)).unwrap().1,
        EffectState::Confirmed
    );
    assert_eq!(store.list(RECEIPT_KIND).unwrap().len(), 1);
    // Terminal: no second readback, no second Receipt, no restart of the effect.
    assert_eq!(
        confirm(
            &mut store,
            &repo_id,
            &id,
            success("aaaa", "cccc", &"1".repeat(40))
        )
        .unwrap_err()
        .code,
        "INTENT_TERMINAL"
    );
    assert_eq!(
        begin(&mut store, &repo_id, &id).unwrap_err().code,
        "INTENT_TERMINAL"
    );
    let shown = show(&store, &repo_id, &id).unwrap();
    assert_eq!(shown["receipt"]["receipt_id"], receipt_id);
    assert_eq!(shown["effect_state"], "confirmed");
    drop(store);
    let store = Store::open(&temp.0).unwrap();
    assert_eq!(integ::get(&store, &repo_id, &id).unwrap(), done);
}

#[test]
fn an_unknown_result_keeps_the_target_occupied_until_readback_settles_it() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let repo_id = local_repo(&mut store);
    admit_revision_seam(&mut store, &actor(), &repo_id, &revision_of(1)).unwrap();
    let preview = integ::prepare(
        &store,
        &actor(),
        input(&repo_id, "one", Form::AcceptAdvance),
        observed(Some("aaaa")),
    )
    .unwrap();
    let intent = submit(&mut store, &actor(), "integration:one", preview.clone()).unwrap();
    begin(&mut store, &repo_id, &intent.intent_id).unwrap();
    let unknown = confirm(
        &mut store,
        &repo_id,
        &intent.intent_id,
        Outcome::Unknown(attention("RESULT_UNKNOWN")),
    )
    .unwrap();
    assert_eq!(unknown.state, IntentState::Unknown);
    let mut other = preview;
    other.input.key = "two".into();
    assert_eq!(
        submit(&mut store, &actor(), "integration:two", other)
            .unwrap_err()
            .code,
        "TARGET_BUSY"
    );
    // Later readback under accept-advance records the actual head, not the previewed one.
    begin(&mut store, &repo_id, &intent.intent_id).unwrap();
    let done = confirm(
        &mut store,
        &repo_id,
        &intent.intent_id,
        success("bbbb", "dddd", &"1".repeat(40)),
    )
    .unwrap();
    let receipt = receipt(&store, &repo_id, done.receipt_id.as_deref().unwrap()).unwrap();
    assert_eq!(receipt.target_head_before.as_deref(), Some("bbbb"));
    assert_eq!(receipt.target_head_after, "dddd");
}

#[test]
fn a_platform_bound_repo_offers_no_local_target_and_platform_targets_need_declared_capabilities() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let repo_id = hosted_repo(&mut store);
    admit_revision_seam(&mut store, &actor(), &repo_id, &revision_of(1)).unwrap();
    let local = input(&repo_id, "k", Form::ExpectedHead);
    assert_eq!(
        integ::prepare(&store, &actor(), local, observed(Some("aaaa")))
            .unwrap_err()
            .code,
        "LOCAL_TARGET_NOT_ALLOWED"
    );
    let platform_observation = |protection: Option<ProtectionSnapshot>| Observation {
        provider_ref: "http://127.0.0.1:3000/admin/example".into(),
        head: Some("aaaa".into()),
        protection,
        continuity: Some(json!({"instance": "http://127.0.0.1:3000", "stable_id": "7"})),
    };
    let mut platform = input(&repo_id, "k", Form::AcceptAdvance);
    platform.target_kind = TargetKind::Platform;
    // A binding that does not declare merge or protection readback refuses platform targets.
    declare_capabilities(
        &mut store,
        &repo_id,
        json!({
            "review_threads": false, "formal_reviews": false, "checks": "external_status_only",
            "remote_merge": false, "identity_mapping": false, "expected_target_head": false,
            "review_text_readback": false, "protection_readback": true
        }),
    );
    assert_eq!(
        integ::prepare(
            &store,
            &actor(),
            platform.clone(),
            platform_observation(Some(ProtectionSnapshot::default()))
        )
        .unwrap_err()
        .code,
        "CAPABILITY_MISSING"
    );
    declare_capabilities(
        &mut store,
        &repo_id,
        json!({
            "review_threads": true, "formal_reviews": true, "checks": "external_status_only",
            "remote_merge": true, "identity_mapping": false, "expected_target_head": false,
            "review_text_readback": true, "protection_readback": true
        }),
    );
    // Expected-head needs a binding that guarantees it; this one does not.
    let mut frozen = platform.clone();
    frozen.form = Form::ExpectedHead;
    assert_eq!(
        integ::prepare(
            &store,
            &actor(),
            frozen,
            platform_observation(Some(ProtectionSnapshot::default()))
        )
        .unwrap_err()
        .code,
        "EXPECTED_HEAD_UNSUPPORTED"
    );
    // Protection must have been read to be frozen.
    assert_eq!(
        integ::prepare(
            &store,
            &actor(),
            platform.clone(),
            platform_observation(None)
        )
        .unwrap_err()
        .code,
        "PROTECTION_UNREAD"
    );
    let snapshot = ProtectionSnapshot {
        requires_review_request: true,
        required_checks: vec!["canary".into()],
        strict_sync: false,
        require_conversation_resolution: false,
        required_approvals: 0,
        other: BTreeMap::new(),
    };
    let preview = integ::prepare(
        &store,
        &actor(),
        platform,
        platform_observation(Some(snapshot.clone())),
    )
    .unwrap();
    assert_eq!(preview.target.kind, TargetKind::Platform);
    assert_eq!(preview.protection.as_ref(), Some(&snapshot));
    assert_eq!(
        preview.target.binding.as_ref().map(|b| &b.version),
        Some(&store::Version::State(3))
    );
    let intent = submit(&mut store, &actor(), "integration:k", preview).unwrap();
    assert_eq!(intent.state, IntentState::Pending);
    // A local target without continuity evidence cannot be frozen either.
    let local_repo_id = local_repo(&mut store);
    admit_revision_seam(&mut store, &actor(), &local_repo_id, &revision_of(1)).unwrap();
    let mut blind = observed(Some("aaaa"));
    blind.continuity = None;
    assert_eq!(
        integ::prepare(
            &store,
            &actor(),
            input(&local_repo_id, "k", Form::ExpectedHead),
            blind
        )
        .unwrap_err()
        .code,
        "TARGET_CONTINUITY_UNREAD"
    );
}

#[test]
fn an_attempt_is_frozen_before_execution_and_only_a_terminal_readback_or_a_proven_no_write_clears_it()
 {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let repo_id = local_repo(&mut store);
    admit_revision_seam(&mut store, &actor(), &repo_id, &revision_of(1)).unwrap();
    let preview = integ::prepare(
        &store,
        &actor(),
        input(&repo_id, "one", Form::AcceptAdvance),
        observed(Some("aaaa")),
    )
    .unwrap();
    let intent = submit(&mut store, &actor(), "integration:one", preview).unwrap();
    let id = intent.intent_id.clone();
    begin(&mut store, &repo_id, &id).unwrap();
    let plan = AttemptInput {
        number: 1,
        idempotency_key: format!("{id}:1"),
        commit: "c".repeat(40),
        expected_head: "aaaa".into(),
        dispatched: false,
    };
    let recorded = record_attempt(&mut store, &repo_id, &id, plan.clone()).unwrap();
    assert_eq!(recorded.attempt.as_ref(), Some(&plan));
    // A request that may write leaves: the attempt is marked as dispatched before it goes,
    // the same plan is still recognised as the same plan, and only a proven refusal
    // (marking it back) says it never wrote. The wrong attempt number is refused.
    assert_eq!(
        mark_attempt_dispatched(&mut store, &repo_id, &id, 2, true)
            .unwrap_err()
            .code,
        "ATTEMPT_MISMATCH"
    );
    let sent = mark_attempt_dispatched(&mut store, &repo_id, &id, 1, true).unwrap();
    assert!(sent.attempt.as_ref().unwrap().dispatched);
    assert!(sent.version > recorded.version);
    let same = record_attempt(&mut store, &repo_id, &id, plan.clone()).unwrap();
    assert!(
        same.attempt.as_ref().unwrap().dispatched,
        "marking survives a replan"
    );
    assert_eq!(same.version, sent.version);
    assert!(
        integ::get(&store, &repo_id, &id)
            .unwrap()
            .attempt
            .unwrap()
            .dispatched,
        "persisted, not held in memory"
    );
    let refused = mark_attempt_dispatched(&mut store, &repo_id, &id, 1, false).unwrap();
    assert!(!refused.attempt.as_ref().unwrap().dispatched);
    assert_eq!(
        mark_attempt_dispatched(&mut store, &repo_id, &id, 1, false)
            .unwrap()
            .version,
        refused.version,
        "idempotent"
    );
    let recorded = refused;
    // Same plan again is a no-op; another plan while one is in flight is refused.
    assert_eq!(
        record_attempt(&mut store, &repo_id, &id, plan.clone())
            .unwrap()
            .version,
        recorded.version
    );
    let mut other = plan.clone();
    other.expected_head = "bbbb".into();
    assert_eq!(
        record_attempt(&mut store, &repo_id, &id, other)
            .unwrap_err()
            .code,
        "ATTEMPT_IN_FLIGHT"
    );
    // Unknown and attention keep the plan; the next retry must reuse it.
    let waiting = confirm(
        &mut store,
        &repo_id,
        &id,
        Outcome::Unknown(attention("RESULT_UNKNOWN")),
    )
    .unwrap();
    assert_eq!(waiting.attempt.as_ref(), Some(&plan));
    // A proven no-write lets control clear it explicitly.
    let cleared = clear_attempt(&mut store, &repo_id, &id).unwrap();
    assert_eq!(cleared.attempt, None);
    assert_eq!(
        mark_attempt_dispatched(&mut store, &repo_id, &id, 1, true)
            .unwrap_err()
            .code,
        "ATTEMPT_NOT_RECORDED"
    );
    let plan2 = AttemptInput {
        number: 2,
        idempotency_key: format!("{id}:2"),
        commit: "c".repeat(40),
        expected_head: "bbbb".into(),
        dispatched: false,
    };
    begin(&mut store, &repo_id, &id).unwrap();
    record_attempt(&mut store, &repo_id, &id, plan2).unwrap();
    let done = confirm(
        &mut store,
        &repo_id,
        &id,
        success("bbbb", "cccc", &"1".repeat(40)),
    )
    .unwrap();
    assert_eq!(done.attempt, None);
    assert_eq!(done.state, IntentState::Succeeded);
    assert_eq!(
        record_attempt(&mut store, &repo_id, &id, plan)
            .unwrap_err()
            .code,
        "INTENT_TERMINAL"
    );
}

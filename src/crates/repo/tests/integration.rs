//! Integration intents and Receipts without any executor: admission, exclusion, readback.
mod common;
use common::*;
use repo::integration::{self as integ, *};
use repo::{Platform, admit, prepare as prepare_registration};
use serde_json::json;
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
    }
}

fn success(before: &str, after: &str, tree: &str) -> Outcome {
    Outcome::Succeeded {
        target_head_before: Some(before.into()),
        target_head_after: after.into(),
        integrated_commit: after.into(),
        integrated_tree: tree.into(),
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
    assert_eq!(receipt.integrated_tree, "1".repeat(40));
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

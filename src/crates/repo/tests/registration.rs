mod common;
use common::*;
use repo::*;
use store::{EffectState, Store};

#[test]
fn same_command_returns_one_repo_and_no_room_even_after_restart() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let prepared = prepare(request(Platform::None), None).unwrap();
    let first = admit(&mut store, &actor(), "command", "key", prepared.clone()).unwrap();
    assert_eq!(first.lifecycle, Lifecycle::Active);
    assert!(store.list("room").unwrap().is_empty());
    drop(store);
    let mut store = Store::open(&temp.0).unwrap();
    let second = admit(&mut store, &actor(), "command", "key", prepared.clone()).unwrap();
    assert_eq!(first.repo_id, second.repo_id);
    assert_eq!(list(&store).unwrap().len(), 1);
    let mut other = prepared;
    other.request.name = "other".into();
    assert_eq!(
        admit(&mut store, &actor(), "command", "key", other)
            .unwrap_err()
            .code,
        "IDEMPOTENCY_CONFLICT"
    );
}

#[test]
fn local_pending_outbox_readback_identity_confirmation_and_abandonment() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let prepared = prepare(request(Platform::Local), None).unwrap();
    let reg = admit(&mut store, &actor(), "command", "key", prepared).unwrap();
    assert_eq!(
        require_active(&store, &reg.repo_id).unwrap_err().code,
        "REPO_PENDING"
    );
    assert_eq!(
        store
            .effect(&effect_id(&reg.repo_id, "platform"))
            .unwrap()
            .1,
        EffectState::Pending
    );
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
    store
        .begin_effect(store.generation(), &effect_id(&reg.repo_id, "platform"))
        .unwrap();
    drop(store); // external create succeeded, response was lost; same correlation is read back
    let mut store = Store::open(&temp.0).unwrap();
    let reg = confirm_platform(&mut store, &reg.repo_id, observed.clone()).unwrap();
    let platform = store.get(&binding(&reg.repo_id).key).unwrap().unwrap();
    let store::RecordData::Value { value } = platform.data else {
        panic!("platform binding required")
    };
    assert_eq!(value["capabilities"]["checks"], "external_status_only");
    assert_eq!(value["capabilities"]["expected_target_head"], false);
    assert_eq!(
        store
            .effect(&effect_id(&reg.repo_id, "platform"))
            .unwrap()
            .1,
        EffectState::Confirmed
    );
    assert!(
        finish(
            &mut store,
            &actor(),
            &reg.repo_id,
            reg.version,
            FinishChoice::Confirm("7"),
            "finish",
            "finish"
        )
        .is_err()
    );
    store
        .begin_effect(store.generation(), &effect_id(&reg.repo_id, "delivery"))
        .unwrap();
    let reg = confirm_delivery(&mut store, &reg.repo_id).unwrap();
    assert_eq!(
        finish(
            &mut store,
            &actor(),
            &reg.repo_id,
            reg.version,
            FinishChoice::Confirm("8"),
            "finish",
            "finish"
        )
        .unwrap_err()
        .code,
        "REGISTRATION_UNCONFIRMED"
    );
    let active = finish(
        &mut store,
        &actor(),
        &reg.repo_id,
        reg.version,
        FinishChoice::Confirm("7"),
        "finish",
        "finish",
    )
    .unwrap();
    assert_eq!(active.lifecycle, Lifecycle::Active);
    assert_eq!(
        finish(
            &mut store,
            &actor(),
            &reg.repo_id,
            reg.version,
            FinishChoice::Confirm("7"),
            "finish",
            "finish"
        )
        .unwrap()
        .version,
        active.version
    );
    assert_eq!(list(&store).unwrap().len(), 1);
    assert!(store.list("room").unwrap().is_empty());
}

#[test]
fn identity_and_source_choices_do_not_come_from_url() {
    let mut r = request(Platform::Github);
    r.origin = Origin::External;
    r.remote_evidence = Some("https://github.com/a/b".into());
    assert_eq!(
        prepare(r.clone(), None).unwrap_err().code,
        "PLATFORM_ID_REQUIRED"
    );
    r.instance = Some("github.com".into());
    r.platform_repo_id = Some("123".into());
    r.platform_path = Some("c/d".into());
    assert_eq!(
        prepare(r.clone(), None).unwrap().evidence_conflicts.len(),
        1
    );
    r.platform = Some(Platform::Local);
    assert_eq!(
        prepare(r, None).unwrap_err().code,
        "EXTERNAL_MIRROR_REJECTED"
    );
    let mut r = request(Platform::None);
    r.origin = Origin::External;
    r.remote_evidence = Some("https://github.com/a/b".into());
    assert_eq!(prepare(r, None).unwrap().platform, Platform::None);
    let mut candidate = source_candidates(&Platform::Github, "repo").remove(0);
    candidate.create = false;
    candidate.field_writeback = false;
    assert_eq!(candidate.label(), "cannot_default_can_claim");
    assert!(validate_default(Some("github_issues"), &[candidate]).is_err());
}

#[test]
fn unknown_abandonment_keeps_original_target_and_does_not_confirm_effect() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let reg = admit(
        &mut store,
        &actor(),
        "register",
        "register",
        prepare(request(Platform::Local), None).unwrap(),
    )
    .unwrap();
    store
        .begin_effect(store.generation(), &effect_id(&reg.repo_id, "platform"))
        .unwrap();
    let abandoned = finish(
        &mut store,
        &actor(),
        &reg.repo_id,
        reg.version,
        FinishChoice::Abandon,
        "abandon",
        "abandon",
    )
    .unwrap();
    assert!(abandoned.abandoned);
    assert!(abandoned.residual.is_none());
    assert!(abandoned.residual_target.unwrap().ends_with(":example"));
    assert_eq!(
        store
            .effect(&effect_id(&reg.repo_id, "platform"))
            .unwrap()
            .1,
        EffectState::Unknown
    );
    assert_eq!(
        require_active(&store, &reg.repo_id).unwrap_err().code,
        "REPO_PENDING"
    );
}

#[test]
fn declared_identity_mismatch_and_read_only_default_cannot_activate() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let mut r = request(Platform::Github);
    r.origin = Origin::External;
    r.instance = Some("github.com".into());
    r.platform_path = Some("a/b".into());
    r.platform_repo_id = Some("5".into());
    r.default_source = Some("github_issues".into());
    let reg = admit(
        &mut store,
        &actor(),
        "register",
        "register",
        prepare(r, None).unwrap(),
    )
    .unwrap();
    store
        .begin_effect(store.generation(), &effect_id(&reg.repo_id, "platform"))
        .unwrap();
    let mut observation = PlatformObservation {
        instance: "github.com".into(),
        stable_id: "6".into(),
        full_name: "a/b".into(),
        clone_url: "https://github.com/a/b.git".into(),
        account_id: "1".into(),
        has_issues: true,
        can_write_issues: true,
        credential_ref: String::new(),
    };
    assert_eq!(
        confirm_platform(&mut store, &reg.repo_id, observation.clone())
            .unwrap_err()
            .code,
        "PLATFORM_ID_MISMATCH"
    );
    observation.stable_id = "5".into();
    observation.can_write_issues = false;
    assert_eq!(
        confirm_platform(&mut store, &reg.repo_id, observation.clone())
            .unwrap_err()
            .code,
        "SOURCE_CAPABILITY_MISSING"
    );
    assert_eq!(get(&store, &reg.repo_id).unwrap().version, 1);
    observation.can_write_issues = true;
    assert_eq!(
        confirm_platform(&mut store, &reg.repo_id, observation)
            .unwrap()
            .lifecycle,
        Lifecycle::Active
    );
}

#[test]
fn separate_human_registrations_never_deduplicate_by_content() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let p = prepare(request(Platform::None), None).unwrap();
    let a = admit(&mut store, &actor(), "a", "a", p.clone()).unwrap();
    let b = admit(&mut store, &actor(), "b", "b", p).unwrap();
    assert_ne!(a.repo_id, b.repo_id);
    assert_eq!(list(&store).unwrap().len(), 2);
}

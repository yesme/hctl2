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
    for capability in [
        "review_threads",
        "formal_reviews",
        "remote_merge",
        "identity_mapping",
        "review_text_readback",
        "protection_readback",
    ] {
        assert_eq!(
            value["capabilities"][capability], false,
            "unverified Gitea capability: {capability}"
        );
    }
    assert_eq!(value["account_mappings"], serde_json::json!({}));
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
    let blocked = admit(
        &mut store,
        &actor(),
        "new",
        "new",
        prepare(request(Platform::Local), None).unwrap(),
    )
    .unwrap_err();
    assert_eq!(blocked.code, "REGISTRATION_TARGET_BUSY");
    assert!(blocked.message.contains(&reg.repo_id));
    assert_eq!(
        begin_step(&mut store, &actor(), &reg.repo_id, "platform")
            .unwrap_err()
            .code,
        "REGISTRATION_TERMINAL"
    );
    // Abandonment is not readback. Only the adapter's later exact observation can settle the residue.
    let residual =
        reconcile_residual(&mut store, &actor(), &reg.repo_id, observation(), false).unwrap();
    assert!(residual.abandoned);
    assert_eq!(residual.residual.as_ref().unwrap().stable_id, "7");
    assert!(store.pending_effects().unwrap().is_empty());
    assert_eq!(
        reconcile_residual(&mut store, &actor(), &reg.repo_id, observation(), false)
            .unwrap()
            .version,
        residual.version
    );
    assert!(
        admit(
            &mut store,
            &actor(),
            "new",
            "new",
            prepare(request(Platform::Local), None).unwrap()
        )
        .is_ok()
    );
}

#[test]
fn list_ignores_p21_placeholder_without_hiding_real_registrations() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let registered = admit(
        &mut store,
        &actor(),
        "register",
        "register",
        prepare(request(Platform::None), None).unwrap(),
    )
    .unwrap();
    let mut trusted = actor();
    trusted
        .0
        .permission_scope
        .push(store::Scope::Repo("legacy".into()));
    let input = serde_json::json!({});
    let cmd = store::Command {
        command_id: "legacy".into(),
        idempotency_key: "legacy".into(),
        actor: trusted.0.clone(),
        target: key("legacy"),
        expected: store::Expected::Absent,
        binding: binding("legacy"),
        operation: "fixture".into(),
        input_digest: store::Command::digest_input("fixture", &input).unwrap(),
        input,
    };
    store
        .submit(store.generation(), &trusted, &cmd, None, |tx| {
            tx.put(&store::Record {
                key: key("legacy"),
                version: 1,
                revision_digest: "0".repeat(64),
                data: store::RecordData::Repo {
                    platform_binding: None,
                    registration: None,
                },
                sources: vec![],
                materials: vec![],
            })?;
            Ok(serde_json::json!({}))
        })
        .unwrap();
    let registrations = list(&store).unwrap();
    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].repo_id, registered.repo_id);
    assert_eq!(store.list("repo").unwrap().len(), 2);
}

fn observation() -> PlatformObservation {
    PlatformObservation {
        instance: "http://127.0.0.1:3000".into(),
        stable_id: "7".into(),
        full_name: "admin/example".into(),
        clone_url: "http://127.0.0.1:3000/admin/example.git".into(),
        account_id: "1".into(),
        has_issues: true,
        can_write_issues: true,
        credential_ref: String::new(),
    }
}

#[test]
fn abandoned_unsent_registration_releases_name_without_claiming_external_success() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let p = prepare(request(Platform::Local), None).unwrap();
    let reg = admit(&mut store, &actor(), "first", "first", p.clone()).unwrap();
    finish(
        &mut store,
        &actor(),
        &reg.repo_id,
        reg.version,
        FinishChoice::Abandon,
        "abandon",
        "abandon",
    )
    .unwrap();
    assert_eq!(
        store
            .effect(&effect_id(&reg.repo_id, "platform"))
            .unwrap()
            .1,
        EffectState::Cancelled
    );
    assert!(store.pending_effects().unwrap().is_empty());
    drop(store);
    let mut store = Store::open(&temp.0).unwrap();
    let next = admit(&mut store, &actor(), "next", "next", p).unwrap();
    assert_ne!(next.repo_id, reg.repo_id);
    assert!(get(&store, &reg.repo_id).unwrap().abandoned);
    assert!(get(&store, &reg.repo_id).unwrap().residual.is_none());
    assert!(
        store
            .begin_effect(store.generation(), &effect_id(&reg.repo_id, "platform"))
            .is_err()
    );
}

#[test]
fn source_observation_changes_do_not_change_frozen_input_or_block_delivery() {
    let temp = Temp::new();
    let mut store = Store::open(&temp.0).unwrap();
    let p = prepare(request(Platform::Local), None).unwrap();
    let reg = admit(&mut store, &actor(), "register", "register", p.clone()).unwrap();
    begin_step(&mut store, &actor(), &reg.repo_id, "platform").unwrap();
    confirm_platform(&mut store, &reg.repo_id, observation()).unwrap();
    let mut changed = observation();
    changed.has_issues = false;
    changed.can_write_issues = false;
    let current = refresh_platform(&mut store, &reg.repo_id, changed.clone()).unwrap();
    assert_eq!(current.prepared, p);
    assert!(!current.sources[0].can_default());
    drop(store);
    let mut store = Store::open(&temp.0).unwrap();
    begin_step(&mut store, &actor(), &reg.repo_id, "delivery").unwrap();
    assert!(
        confirm_delivery(&mut store, &reg.repo_id)
            .unwrap()
            .delivered
    );
    changed.stable_id = "8".into();
    assert_eq!(
        refresh_platform(&mut store, &reg.repo_id, changed)
            .unwrap_err()
            .code,
        "PLATFORM_ID_MISMATCH"
    );
}

#[test]
fn replay_checks_actor_scope_and_exact_frozen_preview() {
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
    drop(store);
    let mut store = Store::open(&temp.0).unwrap();
    let mut other = actor();
    other.0.principal = "other".into();
    assert_eq!(
        begin_step(&mut store, &other, &reg.repo_id, "platform")
            .unwrap_err()
            .code,
        "PERMISSION_DENIED"
    );
    other = actor();
    other.0.permission_scope.clear();
    assert_eq!(
        begin_step(&mut store, &other, &reg.repo_id, "platform")
            .unwrap_err()
            .code,
        "PERMISSION_DENIED"
    );
    assert_eq!(
        begin_step(&mut store, &actor(), &reg.repo_id, "platform").unwrap(),
        EffectState::Pending
    );
    // Trusted reducer bug/tampering must not silently acquire the old authorization.
    let mut record = store.get(&key(&reg.repo_id)).unwrap().unwrap();
    record.version += 1;
    if let store::RecordData::Repo {
        registration: Some(value),
        ..
    } = &mut record.data
    {
        value["prepared"]["request"]["platform_path"] = serde_json::json!("another-target");
    }
    let mut scoped = actor();
    scoped
        .0
        .permission_scope
        .push(store::Scope::Repo(reg.repo_id.clone()));
    let cmd = store::Command {
        command_id: "tamper".into(),
        idempotency_key: "tamper".into(),
        actor: scoped.0.clone(),
        target: key(&reg.repo_id),
        expected: store::Expected::Exact(store::Version::State(reg.version)),
        binding: binding(&reg.repo_id),
        operation: "fixture".into(),
        input: serde_json::json!({}),
        input_digest: store::Command::digest_input("fixture", &serde_json::json!({})).unwrap(),
    };
    store
        .submit(store.generation(), &scoped, &cmd, None, |tx| {
            tx.put(&record)?;
            Ok(serde_json::json!({}))
        })
        .unwrap();
    assert_eq!(
        begin_step(&mut store, &actor(), &reg.repo_id, "platform")
            .unwrap_err()
            .code,
        "FROZEN_INPUT_CHANGED"
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

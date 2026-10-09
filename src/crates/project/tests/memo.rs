use super::*;
use project::memo::{self, Action as MemoAction, Input as MemoInput, Memo};

/// Freeze one exact message through a real Room command: the only way text acquires
/// a source reference a Memo can cite.
fn freeze(e: &mut Env, key: &str, event_id: &str, body: &str) -> chat::Source {
    let (binding, room) = chat::main_binding(&e.store, &e.a).unwrap();
    let source = chat::Source::Message {
        binding: reference(&binding),
        event_id: event_id.into(),
        content_digest: foundation::bytes_sha256(body.as_bytes()),
    };
    let plan = chat::prepare(
        &e.store,
        chat::Input {
            key: key.into(),
            action: chat::Action::Freeze {
                project_id: e.a.clone(),
                room_id: room.id,
                version: binding.version,
                event_id: event_id.into(),
            },
        },
        vec![chat::SourceText {
            source: source.clone(),
            body: body.into(),
            excerpt: body.into(),
        }],
        vec![],
    )
    .unwrap();
    chat::admit(&mut e.store, &actor(), plan).unwrap();
    source
}

fn source_reference(e: &Env, source: &chat::Source) -> Reference {
    let id = foundation::canonical_json_sha256(&serde_json::to_value(source).unwrap()).unwrap();
    Reference {
        key: key(Scope::Project(e.a.clone()), "chat_source_reference", &id),
        version: Version::State(1),
    }
}

fn memo_input(
    e: &Env,
    key: &str,
    memo_id: &str,
    sources: Vec<chat::Source>,
    body: &str,
    supersedes: Option<i64>,
) -> MemoInput {
    MemoInput {
        key: key.into(),
        action: MemoAction::Publish {
            project_id: e.a.clone(),
            project_version: project(&e.store, &e.a).unwrap().version,
            memo_id: memo_id.into(),
            applicability: "repo".into(),
            sources,
            supersedes,
            expires_at: None,
            body: body.into(),
        },
    }
}

fn expiring(mut input: MemoInput, expires_at: Option<u64>) -> MemoInput {
    let MemoAction::Publish {
        expires_at: field, ..
    } = &mut input.action;
    *field = expires_at;
    input
}

/// The two steps a client takes: preview, then confirm.
fn apply(e: &mut Env, input: MemoInput) -> Result<Value> {
    let plan = memo::prepare(&e.store, input, &actor())?;
    memo::admit(&mut e.store, &actor(), plan)
}

fn publish(
    e: &mut Env,
    key: &str,
    memo_id: &str,
    sources: Vec<chat::Source>,
    body: &str,
    supersedes: Option<i64>,
) -> Result<Value> {
    let input = memo_input(e, key, memo_id, sources, body, supersedes);
    apply(e, input)
}

fn manifest(e: &Env, now: u64) -> Vec<Reference> {
    let listed = memo::list(&e.store, &actor(), &e.a, now).unwrap();
    serde_json::from_value(listed["manifest"].clone()).unwrap()
}

/// A raw write that bypasses the Memo command, to prove the domain still refuses it.
fn raw_put(e: &mut Env, record: Record) -> Result<Value> {
    let scoped = chat::owner(&actor(), &e.a)?;
    let old = e.store.get(&record.key)?;
    let input = serde_json::to_value(&record)?;
    let command = Command {
        command_id: format!("fixture:{}:{}", record.key.id, record.version),
        idempotency_key: format!("fixture:{}:{}", record.key.id, record.version),
        actor: scoped.0.clone(),
        target: record.key.clone(),
        expected: old.as_ref().map_or(Expected::Absent, |r| {
            Expected::Exact(Version::State(r.version))
        }),
        binding: old.as_ref().map_or_else(|| reference(&record), reference),
        operation: "fixture".into(),
        input_digest: Command::digest_input("fixture", &input)?,
        input,
    };
    e.store
        .submit(e.store.generation(), &scoped, &command, None, |tx| {
            tx.put(&record)?;
            Ok(json!({}))
        })
}

/// Occupy an idempotency key with another command, so the Memo command is refused.
fn occupy(e: &mut Env, idempotency_key: &str) {
    let scoped = chat::owner(&actor(), &e.a).unwrap();
    let input = json!({});
    let command = Command {
        command_id: format!("occupy:{idempotency_key}"),
        idempotency_key: idempotency_key.into(),
        actor: scoped.0.clone(),
        target: key(
            Scope::Control,
            "fixture",
            &format!("occupy-{idempotency_key}"),
        ),
        expected: Expected::Absent,
        binding: reference(&project(&e.store, &e.a).unwrap()),
        operation: "fixture".into(),
        input_digest: Command::digest_input("fixture", &input).unwrap(),
        input,
    };
    e.store
        .submit(e.store.generation(), &scoped, &command, None, |_| {
            Ok(json!({}))
        })
        .unwrap();
}

#[test]
fn memo_preview_writes_nothing_and_only_the_control_owner_publishes() {
    let mut e = Env::new();
    let source = freeze(&mut e, "freeze-1", "$one", "the frozen discussion");
    let input = memo_input(
        &e,
        "publish-1",
        "M",
        vec![source.clone()],
        "published text",
        None,
    );
    let stamp = e.store.read_stamp();
    let plan = memo::prepare(&e.store, input.clone(), &actor()).unwrap();
    // A preview is a read: no pointer, no revision, no command, no admitted body.
    assert_eq!(e.store.read_stamp(), stamp);
    for kind in ["memo", "memo_revision", "memo_command"] {
        assert!(e.store.list(kind).unwrap().is_empty(), "{kind}");
    }
    let published: Memo = serde_json::from_value(plan.result["memo"].clone()).unwrap();
    assert_eq!(published.revision, 1);
    assert_eq!(published.memo_id, "M");
    assert_eq!(published.author, "owner");
    assert_eq!(published.applicability, "repo");
    assert!(published.supersedes.is_none());
    assert!(published.expires_at.is_none());
    assert_eq!(
        published.content_digest,
        foundation::bytes_sha256(b"published text")
    );
    assert_eq!(published.sources, vec![source_reference(&e, &source)]);
    assert_eq!(plan.result["body_bytes"], 14);
    assert_eq!(plan.pointer.version, 1);
    assert_eq!(plan.revision.version, 1);
    assert!(plan.revision.materials.is_empty());
    // Confirming re-reads the store, so the preview must be reproducible.
    assert_eq!(
        serde_json::to_value(memo::prepare(&e.store, input.clone(), &actor()).unwrap()).unwrap(),
        serde_json::to_value(&plan).unwrap()
    );
    for denied in [
        TrustedActor(Actor {
            source: ActorSource::InternalReducer,
            ..actor().0
        }),
        TrustedActor(Actor {
            source: ActorSource::ProviderEvent,
            ..actor().0
        }),
        TrustedActor(Actor {
            permission_scope: vec![Scope::Project(e.a.clone())],
            ..actor().0
        }),
    ] {
        assert_eq!(
            memo::prepare(&e.store, input.clone(), &denied)
                .unwrap_err()
                .code,
            "PERMISSION_DENIED"
        );
    }
    let result = apply(&mut e, input).unwrap();
    assert_eq!(result, plan.result);
    assert_eq!(e.store.list("memo").unwrap().len(), 1);
    let revisions = e.store.list("memo_revision").unwrap();
    assert_eq!(revisions.len(), 1);
    assert_eq!(revisions[0].materials.len(), 1);
    assert_eq!(
        revisions[0].materials[0].byte_digest,
        foundation::bytes_sha256(b"published text")
    );
    let shown = memo::show(&e.store, &actor(), &e.a, "M", None).unwrap();
    assert_eq!(shown["body"], "published text");
    assert!(shown["current"].as_bool().unwrap());
}

#[test]
fn a_published_revision_is_not_rewritable_and_replay_answers_with_the_original() {
    let mut e = Env::new();
    let source = freeze(&mut e, "freeze-1", "$one", "frozen");
    let input = memo_input(&e, "publish-1", "M", vec![source.clone()], "one", None);
    let first = apply(&mut e, input.clone()).unwrap();
    // The same command key and input replay; they do not publish a second revision.
    assert_eq!(apply(&mut e, input).unwrap(), first);
    assert_eq!(e.store.list("memo_revision").unwrap().len(), 1);
    assert_eq!(e.store.list("memo").unwrap()[0].version, 1);
    let revision = e.store.list("memo_revision").unwrap().pop().unwrap();
    let exact = reference(&revision);
    // Re-publishing the same id without naming the revision it replaces is refused.
    assert_eq!(
        publish(&mut e, "publish-2", "M", vec![source.clone()], "two", None)
            .unwrap_err()
            .code,
        "SUPERSEDES_REQUIRED"
    );
    // The same version with different content is not the next version.
    let mut rewritten = value_record(
        revision.key.clone(),
        1,
        &json!({"memo_id":"M","other":true}),
    )
    .unwrap();
    rewritten.materials = revision.materials.clone();
    assert_eq!(
        raw_put(&mut e, rewritten).unwrap_err().code,
        "VERSION_CONFLICT"
    );
    // A later version of the revision record still cannot answer for the exact reference.
    let mut tampered: Memo = decode(&revision).unwrap();
    tampered.applicability = "rewritten".into();
    let mut next = value_record(revision.key.clone(), revision.version + 1, &tampered).unwrap();
    next.materials = revision.materials.clone();
    raw_put(&mut e, next).unwrap();
    assert_eq!(e.store.get(&revision.key).unwrap().unwrap().version, 2);
    assert_eq!(
        memo::at(&e.store, &exact).unwrap_err().code,
        "VERSION_CONFLICT"
    );
    assert_eq!(
        memo::show(&e.store, &actor(), &e.a, "M", Some(1))
            .unwrap_err()
            .code,
        "VERSION_CONFLICT"
    );
    // The current pointer is a projection; it does not serve the rewritten revision either.
    assert_eq!(
        memo::show(&e.store, &actor(), &e.a, "M", None)
            .unwrap_err()
            .code,
        "VERSION_CONFLICT"
    );
    // A revision record without an admitted body locator is not a readable Memo.
    let orphan = Memo {
        memo_id: "N".into(),
        revision: 1,
        applicability: "repo".into(),
        author: "owner".into(),
        sources: vec![source_reference(&e, &source)],
        content_digest: foundation::bytes_sha256(b"never admitted"),
        supersedes: None,
        expires_at: None,
    };
    let orphan = value_record(
        key(Scope::Project(e.a.clone()), "memo_revision", "N:1"),
        1,
        &orphan,
    )
    .unwrap();
    let orphan_reference = reference(&orphan);
    raw_put(&mut e, orphan).unwrap();
    assert_eq!(
        memo::at(&e.store, &orphan_reference).unwrap_err().code,
        "MATERIAL_NOT_ADMITTED"
    );
}

#[test]
fn an_update_is_a_new_revision_and_every_earlier_one_stays_readable() {
    let mut e = Env::new();
    let source = freeze(&mut e, "freeze-1", "$one", "frozen");
    let first = publish(&mut e, "publish-1", "M", vec![source.clone()], "one", None).unwrap();
    let r1: Reference = serde_json::from_value(first["revision_reference"].clone()).unwrap();
    let second = publish(
        &mut e,
        "publish-2",
        "M",
        vec![source.clone()],
        "two",
        Some(1),
    )
    .unwrap();
    assert_eq!(second["revision"], 2);
    let r2: Reference = serde_json::from_value(second["revision_reference"].clone()).unwrap();
    assert_ne!(r1, r2);
    assert_eq!(e.store.list("memo_revision").unwrap().len(), 2);
    // The earlier revision record is untouched: still version 1 at its own key.
    assert_eq!(e.store.get(&r1.key).unwrap().unwrap().version, 1);
    assert_eq!(r1.key.id, "M:1");
    let current = memo::show(&e.store, &actor(), &e.a, "M", None).unwrap();
    assert_eq!(current["body"], "two");
    assert!(current["current"].as_bool().unwrap());
    let old = memo::show(&e.store, &actor(), &e.a, "M", Some(1)).unwrap();
    assert_eq!(old["body"], "one");
    assert!(!old["current"].as_bool().unwrap());
    assert_eq!(old["revision"], serde_json::to_value(&r1).unwrap());
    assert!(memo::at(&e.store, &r1).unwrap().1.supersedes.is_none());
    assert_eq!(memo::at(&e.store, &r2).unwrap().1.supersedes, Some(r1));
    // Only the revision the pointer names can be superseded.
    for (name, supersedes, code) in [
        ("publish-3", Some(1), "VERSION_CONFLICT"),
        ("publish-4", Some(9), "VERSION_CONFLICT"),
        ("publish-5", Some(0), "VERSION_CONFLICT"),
        ("publish-6", None, "SUPERSEDES_REQUIRED"),
    ] {
        assert_eq!(
            publish(&mut e, name, "M", vec![source.clone()], "three", supersedes)
                .unwrap_err()
                .code,
            code,
            "{name}"
        );
    }
    assert_eq!(e.store.list("memo_revision").unwrap().len(), 2);
    let third = publish(&mut e, "publish-7", "M", vec![source], "three", Some(2)).unwrap();
    assert_eq!(third["revision"], 3);
    assert_eq!(e.store.list("memo").unwrap()[0].version, 3);
    for (revision, body) in [(1, "one"), (2, "two")] {
        assert_eq!(
            memo::show(&e.store, &actor(), &e.a, "M", Some(revision)).unwrap()["body"],
            body
        );
    }
    let newest = memo::show(&e.store, &actor(), &e.a, "M", None).unwrap();
    assert_eq!(newest["body"], "three");
    assert_eq!(newest["memo"]["revision"], 3);
    assert_eq!(
        memo::show(&e.store, &actor(), &e.a, "M", Some(4))
            .unwrap_err()
            .code,
        "MEMO_NOT_FOUND"
    );
}

#[test]
fn text_that_never_went_through_the_command_does_not_become_a_memo() {
    let mut e = Env::new();
    let (binding, main) = chat::main_binding(&e.store, &e.a).unwrap();
    let body = json!({"body":"discussed","msgtype":"m.text"}).to_string();
    let discussed = chat::Source::Message {
        binding: reference(&binding),
        event_id: "$discussed".into(),
        content_digest: foundation::bytes_sha256(body.as_bytes()),
    };
    // A Room command freezes its sources into references, not into published knowledge.
    e.create_topic(
        "topic",
        chat::Origin::Room {
            room_id: main.id,
            binding_version: binding.version,
        },
        vec![chat::SourceText {
            source: discussed,
            body,
            excerpt: "discussed".into(),
        }],
    );
    for kind in ["memo", "memo_revision", "memo_command"] {
        assert!(e.store.list(kind).unwrap().is_empty(), "{kind}");
    }
    // Text no command froze has no locator, so it cannot be cited or published.
    let unfrozen = chat::Source::Message {
        binding: reference(&binding),
        event_id: "$never-frozen".into(),
        content_digest: foundation::bytes_sha256(b"an automatic summary"),
    };
    let input = memo_input(
        &e,
        "publish-unfrozen",
        "M",
        vec![unfrozen],
        "an automatic summary",
        None,
    );
    assert_eq!(apply(&mut e, input).unwrap_err().code, "SOURCE_UNAVAILABLE");
    // Neither can a source that belongs to another Project.
    let foreign = chat::main_binding(&e.store, &e.b).unwrap().0;
    let cross = chat::Source::Object {
        reference: reference(&foreign),
        content_digest: "0".repeat(64),
    };
    let input = memo_input(&e, "publish-cross", "M", vec![cross], "text", None);
    assert_eq!(apply(&mut e, input).unwrap_err().code, "INVALID_INPUT");
    let source = freeze(&mut e, "freeze-1", "$one", "frozen");
    // An empty publication is not knowledge either.
    let input = memo_input(&e, "publish-empty", "M", vec![source.clone()], "   ", None);
    assert_eq!(apply(&mut e, input).unwrap_err().code, "INVALID_INPUT");
    // The author is the authenticated principal; a payload cannot name one.
    assert!(
        serde_json::from_value::<MemoInput>(json!({
            "key": "publish-1",
            "action": {
                "kind": "publish",
                "project_id": e.a,
                "project_version": 1,
                "memo_id": "M",
                "applicability": "repo",
                "sources": [],
                "supersedes": null,
                "expires_at": null,
                "body": "text",
                "author": "someone else",
            }
        }))
        .is_err()
    );
    // A key already used by another command is refused, and nothing becomes a Memo.
    occupy(&mut e, "publish-key");
    let input = memo_input(&e, "publish-key", "M", vec![source.clone()], "text", None);
    let plan = memo::prepare(&e.store, input, &actor()).unwrap();
    assert_eq!(
        memo::admit(&mut e.store, &actor(), plan).unwrap_err().code,
        "IDEMPOTENCY_CONFLICT"
    );
    for kind in ["memo", "memo_revision"] {
        assert!(e.store.list(kind).unwrap().is_empty(), "{kind}");
    }
    // The body bytes reached the material store under that command key, but saving is
    // not admission: they are unreadable and no revision advanced.
    let scoped = chat::owner(&actor(), &e.a).unwrap();
    let saved = e
        .store
        .save_material(
            e.store.generation(),
            &scoped,
            &Scope::Project(e.a.clone()),
            "publish-key",
            "body",
            b"text",
        )
        .unwrap();
    assert_eq!(
        e.store.read_material(&scoped, &saved).unwrap_err().code,
        "MATERIAL_NOT_ADMITTED"
    );
    // Bytes saved for one command cannot be admitted by another.
    let elsewhere = e
        .store
        .save_material(
            e.store.generation(),
            &scoped,
            &Scope::Project(e.a.clone()),
            "elsewhere",
            "body",
            b"text",
        )
        .unwrap();
    assert_ne!(elsewhere.material_id, saved.material_id);
    assert_eq!(elsewhere.byte_digest, saved.byte_digest);
    let input = json!({});
    let command = Command {
        command_id: "adopt-foreign".into(),
        idempotency_key: "adopt-foreign".into(),
        actor: scoped.0.clone(),
        target: key(Scope::Project(e.a.clone()), "fixture", "adopt-foreign"),
        expected: Expected::Absent,
        binding: reference(&project(&e.store, &e.a).unwrap()),
        operation: "fixture".into(),
        input_digest: Command::digest_input("fixture", &input).unwrap(),
        input,
    };
    assert_eq!(
        e.store
            .submit(e.store.generation(), &scoped, &command, None, |tx| {
                tx.admit_material(&elsewhere)?;
                Ok(json!({}))
            })
            .unwrap_err()
            .code,
        "INVALID_INPUT"
    );
}

#[test]
fn the_pointer_manifest_filters_expired_memos_and_never_lists_superseded_ones() {
    let mut e = Env::new();
    let source = freeze(&mut e, "freeze-1", "$one", "frozen");
    let now = task::now();
    let bounded = expiring(
        memo_input(&e, "publish-1", "M1", vec![source.clone()], "bounded", None),
        Some(now + 100),
    );
    apply(&mut e, bounded).unwrap();
    publish(
        &mut e,
        "publish-2",
        "M2",
        vec![source.clone()],
        "open",
        None,
    )
    .unwrap();
    assert_eq!(manifest(&e, now).len(), 2);
    let renewed = expiring(
        memo_input(&e, "publish-3", "M1", vec![source], "renewed", Some(1)),
        Some(now + 100),
    );
    apply(&mut e, renewed).unwrap();
    // Before the deadline both current revisions are pointers; after it only M2 is.
    let before = manifest(&e, now + 99);
    assert_eq!(before.len(), 2);
    assert_eq!(before[0].key.id, "M1:2");
    let after = manifest(&e, now + 100);
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].key.id, "M2:1");
    let listed = memo::list(&e.store, &actor(), &e.a, now + 100).unwrap();
    let items = listed["items"].as_array().unwrap();
    // The unfiltered reading still names the expired Memo, so the filter is visible.
    assert_eq!(items.len(), 2);
    let expired = items.iter().find(|i| i["memo_id"] == "M1").unwrap();
    assert!(expired["expired"].as_bool().unwrap());
    assert_eq!(expired["revision"], 2);
    assert_eq!(
        items.iter().find(|i| i["memo_id"] == "M2").unwrap()["expired"],
        json!(false)
    );
    // An explicit reference is not filtered: the expired and the superseded revision
    // both still read back exactly.
    assert_eq!(
        memo::show(&e.store, &actor(), &e.a, "M1", Some(1)).unwrap()["body"],
        "bounded"
    );
    assert_eq!(
        memo::show(&e.store, &actor(), &e.a, "M1", None).unwrap()["body"],
        "renewed"
    );
    assert_eq!(listed["project_id"], e.a);
    assert!(
        memo::list(&e.store, &actor(), &e.b, now).unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn the_author_and_the_preview_are_bound_to_the_command_that_published_them() {
    let mut e = Env::new();
    let source = freeze(&mut e, "freeze-1", "$one", "frozen");
    let input = memo_input(&e, "publish-1", "M", vec![source.clone()], "text", None);
    let plan = memo::prepare(&e.store, input.clone(), &actor()).unwrap();
    // A result the caller never previewed is refused, and writes nothing.
    let mut edited = plan.clone();
    edited.result["revision"] = json!(9);
    assert_eq!(
        memo::admit(&mut e.store, &actor(), edited)
            .unwrap_err()
            .code,
        "VERSION_CONFLICT"
    );
    // So is a preview whose confirmed text was swapped afterwards.
    let mut edited_body = plan.clone();
    let MemoAction::Publish { body, .. } = &mut edited_body.input.action;
    *body = "text the preview never showed".into();
    assert_eq!(
        memo::admit(&mut e.store, &actor(), edited_body)
            .unwrap_err()
            .code,
        "VERSION_CONFLICT"
    );
    for kind in ["memo", "memo_revision", "memo_command"] {
        assert!(e.store.list(kind).unwrap().is_empty(), "{kind}");
    }
    let result = memo::admit(&mut e.store, &actor(), plan.clone()).unwrap();
    assert_eq!(result["memo"]["author"], "owner");
    let record = e
        .store
        .get(&key(Scope::Project(e.a.clone()), "memo_revision", "M:1"))
        .unwrap()
        .unwrap();
    // The revision names the Project and the exact frozen sources it was published from.
    assert!(
        record
            .sources
            .contains(&reference(&project(&e.store, &e.a).unwrap()))
    );
    assert!(record.sources.contains(&source_reference(&e, &source)));
    // A stale Project version is refused before anything is written.
    let mut stale = memo_input(
        &e,
        "publish-stale",
        "M",
        vec![source.clone()],
        "text",
        Some(1),
    );
    let MemoAction::Publish {
        project_version, ..
    } = &mut stale.action;
    *project_version += 1;
    assert_eq!(apply(&mut e, stale).unwrap_err().code, "VERSION_CONFLICT");
    // The command key belongs to the actor that used it.
    let other = TrustedActor(Actor {
        principal: "other".into(),
        ..actor().0
    });
    assert_eq!(
        memo::prepare(&e.store, input.clone(), &other)
            .unwrap_err()
            .code,
        "ACTOR_MISMATCH"
    );
    assert_eq!(
        memo::admit(&mut e.store, &other, plan).unwrap_err().code,
        "ACTOR_MISMATCH"
    );
    // The same key cannot publish different text.
    let reused = memo_input(
        &e,
        "publish-1",
        "M",
        vec![source.clone()],
        "other text",
        Some(1),
    );
    assert_eq!(
        apply(&mut e, reused).unwrap_err().code,
        "IDEMPOTENCY_CONFLICT"
    );
    assert_eq!(e.store.list("memo_revision").unwrap().len(), 1);
    assert_eq!(
        memo::show(&e.store, &actor(), &e.a, "M", None).unwrap()["body"],
        "text"
    );
    // An archived Project is read-only for publication too, until it is restored.
    let version = project(&e.store, &e.a).unwrap().version;
    e.apply(
        "archive",
        Action::Archive {
            project_id: e.a.clone(),
            version,
        },
    )
    .unwrap();
    let archived = memo_input(
        &e,
        "publish-archived",
        "M",
        vec![source.clone()],
        "later",
        Some(1),
    );
    assert_eq!(
        apply(&mut e, archived).unwrap_err().code,
        "PROJECT_READ_ONLY"
    );
    assert_eq!(
        memo::show(&e.store, &actor(), &e.a, "M", None).unwrap()["body"],
        "text"
    );
    e.apply(
        "restore",
        Action::Restore {
            project_id: e.a.clone(),
            version: version + 1,
        },
    )
    .unwrap();
    let restored = publish(
        &mut e,
        "publish-restored",
        "M",
        vec![source],
        "later",
        Some(1),
    )
    .unwrap();
    assert_eq!(restored["revision"], 2);
    assert_eq!(
        memo::show(&e.store, &actor(), &e.a, "M", Some(1)).unwrap()["body"],
        "text"
    );
}

//! Package 4 failure cases against the CT-PROJECT / CT-CONNECTION Context
//! lines. Every case names the CT clause it enforces in its doc comment.

use agency_proto::context::{Bundle, Delivery, Entry, Manifest};
use agency_proto::{FrozenRef, Owner, OwnerKind, hash};
use context::{
    Assembler, AssemblyRequest, LocalAssembler, MemorySources, ReviewComments, SourceKind, Sources,
    frozen_from_record,
};
use serde_json::json;
use store::{Record, RecordData, Reference, Scope, Store, TrustedActor, Version};

fn owner() -> Owner {
    Owner {
        project: "p".into(),
        kind: OwnerKind::RoomInvocation,
        id: "invocation-1".into(),
        generation: 1,
    }
}

fn reference(id: &str, digest: &str) -> FrozenRef {
    FrozenRef {
        id: id.into(),
        revision: digest.into(),
        digest: digest.into(),
    }
}

fn manifest(sources: Vec<FrozenRef>, budget: u64, permission: &str) -> Manifest {
    Manifest {
        id: "manifest-1".into(),
        purpose: "assemble the review context".into(),
        scope: "project p".into(),
        parent: None,
        sources,
        selection_policy: reference("policy/policy-1", &hash(b"policy")),
        freshness: "frozen at assembly".into(),
        coverage: "room line; task comments".into(),
        known_gaps: vec![],
        required_skills: vec![],
        permission_digest: permission.into(),
        redaction: reference("redaction/default", &hash(b"redaction")),
        budget,
    }
}

fn assembler(permitted: &[&str], budget: u64) -> LocalAssembler {
    LocalAssembler {
        permitted: permitted.iter().map(|id| (*id).to_owned()).collect(),
        budget,
    }
}

fn permission_digest(permitted: &[&str]) -> String {
    let ids: Vec<String> = permitted.iter().map(|id| (*id).to_owned()).collect();
    context::permission_digest(&ids)
}

/// CT: every entry carries the honest digest of the bytes actually
/// delivered, and validate_delivery rejects a tampered copy.
#[test]
fn entry_digests_record_the_actual_bytes() {
    let room = reference("chat_source_reference/s1", &hash(b"record data"));
    let mut sources = MemorySources::new();
    sources.push(SourceKind::Room, room.clone(), b"real bytes");
    let manifest = manifest(
        vec![room],
        1024,
        &permission_digest(&["chat_source_reference/s1"]),
    );
    let assembler = assembler(&["chat_source_reference/s1"], 1024);
    let assembly = assembler
        .assemble(
            &sources,
            AssemblyRequest {
                manifest,
                consumer: owner(),
            },
        )
        .unwrap();
    let bundle = assembly.bundle.document;
    assert_eq!(bundle.entries[0].bytes_digest, hash(b"real bytes"));
    let mut tampered = bundle.clone();
    if let agency_proto::context::Delivery::Inline { bytes } = &mut tampered.entries[0].delivery {
        bytes.clear();
    }
    assert_eq!(
        tampered.validate_delivery().unwrap_err().code,
        "DELIVERY_DIGEST_MISMATCH"
    );
}

/// CT: a frozen reference that no longer resolves invalidates the preview.
/// The store adapter's moved-record case (SOURCE_VERSION_CHANGED) is its own
/// test below.
#[test]
fn unresolvable_source_invalidates_the_preview() {
    let room = reference("chat_source_reference/gone", &hash(b"version 1"));
    let sources = MemorySources::new();
    let manifest = manifest(
        vec![room],
        1024,
        &permission_digest(&["chat_source_reference/gone"]),
    );
    let assembler = assembler(&["chat_source_reference/gone"], 1024);
    let error = assembler
        .assemble(
            &sources,
            AssemblyRequest {
                manifest,
                consumer: owner(),
            },
        )
        .unwrap_err();
    assert_eq!(error.code, "SOURCE_UNAVAILABLE");
}

/// CT: permission change invalidates an old preview.
#[test]
fn permission_change_invalidates_the_preview() {
    let room = reference("chat_source_reference/s1", &hash(b"room bytes"));
    let mut sources = MemorySources::new();
    sources.push(SourceKind::Room, room.clone(), b"room bytes");
    // Manifest frozen with an old permission set; assembler gates a new one.
    let manifest = manifest(
        vec![room],
        1024,
        &permission_digest(&["chat_source_reference/s1", "task_comments/line"]),
    );
    let assembler = assembler(&["chat_source_reference/s1"], 1024);
    let error = assembler
        .assemble(
            &sources,
            AssemblyRequest {
                manifest,
                consumer: owner(),
            },
        )
        .unwrap_err();
    assert_eq!(error.code, "PERMISSION_CHANGED");
}

/// CT: budget change invalidates an old preview.
#[test]
fn budget_change_invalidates_the_preview() {
    let room = reference("chat_source_reference/s1", &hash(b"room bytes"));
    let mut sources = MemorySources::new();
    sources.push(SourceKind::Room, room.clone(), b"room bytes");
    let manifest = manifest(
        vec![room],
        512,
        &permission_digest(&["chat_source_reference/s1"]),
    );
    let assembler = assembler(&["chat_source_reference/s1"], 1024);
    let error = assembler
        .assemble(
            &sources,
            AssemblyRequest {
                manifest,
                consumer: owner(),
            },
        )
        .unwrap_err();
    assert_eq!(error.code, "BUDGET_CHANGED");
}

/// CT: required material not delivered (a recall slot for a required entry)
/// must fail; here required material over budget degrades to a pointer with a
/// byte copy and a shard suggestion — never a silent drop.
#[test]
fn required_over_budget_degrades_to_pointer_with_copy_never_drops() {
    let big: Vec<u8> = vec![b'x'; 4096];
    let room = reference("chat_source_reference/big", &hash(&big));
    let mut sources = MemorySources::new();
    sources.push(SourceKind::Room, room.clone(), &big);
    let manifest = manifest(
        vec![room],
        1024,
        &permission_digest(&["chat_source_reference/big"]),
    );
    let assembler = assembler(&["chat_source_reference/big"], 1024);
    let assembly = assembler
        .assemble(
            &sources,
            AssemblyRequest {
                manifest,
                consumer: owner(),
            },
        )
        .unwrap();
    let bundle = assembly.bundle.document;
    // All manifest sources are treated as required: over budget must deliver
    // a pointer carrying the exact bytes, not skip the entry.
    assert_eq!(bundle.entries.len(), 1);
    match &bundle.entries[0].delivery {
        Delivery::Pointer {
            bytes,
            relative_name,
        } => {
            assert_eq!(bytes.len(), 4096);
            assert_eq!(hash(bytes), bundle.entries[0].bytes_digest);
            assert!(!relative_name.is_empty());
        }
        other => panic!("expected pointer delivery, got {other:?}"),
    }
    bundle.validate_delivery().unwrap();
}

/// CT: the platform review-comment line is package 6's wiring; requesting it
/// now is a typed error, never an empty success pretending coverage.
#[test]
fn review_comment_line_reports_not_configured() {
    let review = reference("review_comments/line-1", &hash(b"review"));
    let sources = ReviewComments;
    assert_eq!(
        sources
            .exact(SourceKind::ReviewComments, &review)
            .unwrap_err()
            .code,
        "REVIEW_LINE_NOT_CONFIGURED"
    );
}

/// CT: different consumers get independent bundles; a bundle for consumer A
/// does not validate for consumer B (owner mismatch is a structural break).
#[test]
fn bundles_are_per_consumer() {
    let room = reference("chat_source_reference/s1", &hash(b"room bytes"));
    let mut sources = MemorySources::new();
    sources.push(SourceKind::Room, room.clone(), b"room bytes");
    let manifest = manifest(
        vec![room],
        1024,
        &permission_digest(&["chat_source_reference/s1"]),
    );
    let assembler = assembler(&["chat_source_reference/s1"], 1024);
    let mut consumer_a = owner();
    consumer_a.id = "invocation-a".into();
    let mut consumer_b = owner();
    consumer_b.id = "invocation-b".into();
    let assembly_a = assembler
        .assemble(
            &sources,
            AssemblyRequest {
                manifest: manifest.clone(),
                consumer: consumer_a.clone(),
            },
        )
        .unwrap();
    let assembly_b = assembler
        .assemble(
            &sources,
            AssemblyRequest {
                manifest,
                consumer: consumer_b,
            },
        )
        .unwrap();
    assert_ne!(assembly_a.bundle.document.id, assembly_b.bundle.document.id);
    assert_ne!(assembly_a.bundle.digest, assembly_b.bundle.digest);
    assert_eq!(assembly_a.bundle.document.consumer, consumer_a);
}

/// CT: manifest missing any of selection-policy/freshness/coverage/gaps/
/// permissions/budget fails. The type makes gaps/parent optional; the
/// mandatory text fields are checked by the assembler.
#[test]
fn manifest_completeness_is_enforced() {
    let room = reference("chat_source_reference/s1", &hash(b"room bytes"));
    let sources = MemorySources::new();
    let mut manifest = manifest(
        vec![room],
        1024,
        &permission_digest(&["chat_source_reference/s1"]),
    );
    manifest.freshness = String::new();
    let assembler = assembler(&["chat_source_reference/s1"], 1024);
    assert_eq!(
        assembler
            .assemble(
                &sources,
                AssemblyRequest {
                    manifest,
                    consumer: owner()
                }
            )
            .unwrap_err()
            .code,
        "INVALID_INPUT"
    );
}

/// CT: token metering without a tokenizer reports un-metered (None), never an
/// invented number.
#[test]
fn metering_without_tokenizer_is_none() {
    let room = reference("chat_source_reference/s1", &hash(b"room bytes"));
    let mut sources = MemorySources::new();
    sources.push(SourceKind::Room, room.clone(), b"room bytes");
    let manifest = manifest(
        vec![room],
        1024,
        &permission_digest(&["chat_source_reference/s1"]),
    );
    let assembler = assembler(&["chat_source_reference/s1"], 1024);
    let assembly = assembler
        .assemble(
            &sources,
            AssemblyRequest {
                manifest,
                consumer: owner(),
            },
        )
        .unwrap();
    let bundle = assembly.bundle.document;
    assert!(bundle.candidate_tokens.is_none());
    assert!(bundle.selected_tokens.is_none());
    assert!(bundle.delivered_tokens.is_none());
}

/// CT: duplicate pointer local names must be rejected (shared validate).
#[test]
fn duplicate_pointer_names_rejected_by_delivery_validation() {
    let first = entry_pointer("a".into(), b"one".to_vec());
    let second = entry_pointer("b".into(), b"two".to_vec());
    let mut bundle = minimal_bundle(vec![first, second]);
    // Force a duplicate name.
    if let Delivery::Pointer { relative_name, .. } = &mut bundle.entries[1].delivery {
        relative_name.clone_from(&"a.bin".to_owned());
    }
    // also fix digests
    bundle.entries[0].bytes_digest = hash(b"one");
    bundle.entries[1].bytes_digest = hash(b"two");
    assert_eq!(
        bundle.validate_delivery().unwrap_err().code,
        "INVALID_INPUT"
    );
}

/// CT: frozen Manifest/Bundle records are append-only in the store; a second
/// save replays, and the stored digest never changes.
#[test]
fn saved_assemblies_are_append_only_and_replayable() {
    let mut store = temp_store();
    let actor = TrustedActor(agency_proto_to_store_actor());
    let room = reference("chat_source_reference/s1", &hash(b"room bytes"));
    let mut sources = MemorySources::new();
    sources.push(SourceKind::Room, room.clone(), b"room bytes");
    let manifest = manifest(
        vec![room],
        1024,
        &permission_digest(&["chat_source_reference/s1"]),
    );
    let assembler = assembler(&["chat_source_reference/s1"], 1024);
    let assembly = assembler
        .assemble(
            &sources,
            AssemblyRequest {
                manifest,
                consumer: owner(),
            },
        )
        .unwrap();
    let first = context::save_assembly(&mut store, &actor, "p", "ctx-1", &assembly).unwrap();
    assert_eq!(first["replayed"], json!(false));
    let second = context::save_assembly(&mut store, &actor, "p", "ctx-1", &assembly).unwrap();
    assert_eq!(second["replayed"], json!(true));
    let read = context::read_bundle(&store, &actor, "p", &assembly.bundle.document.id)
        .unwrap()
        .expect("bundle stored");
    read.verify().unwrap();
}

#[test]
fn resealing_bad_delivery_or_wrong_manifest_cannot_admit_records() {
    let mut store = temp_store();
    let actor = TrustedActor(agency_proto_to_store_actor());
    let source = reference("chat_source_reference/s1", &hash(b"original"));
    let mut sources = MemorySources::new();
    sources.push(SourceKind::Room, source.clone(), b"original");
    let assembly = assembler(&["chat_source_reference/s1"], 1024)
        .assemble(
            &sources,
            AssemblyRequest {
                manifest: manifest(
                    vec![source],
                    1024,
                    &permission_digest(&["chat_source_reference/s1"]),
                ),
                consumer: owner(),
            },
        )
        .unwrap();
    let mut bytes = assembly.bundle.document.clone();
    if let Delivery::Inline { bytes } = &mut bytes.entries[0].delivery {
        bytes.clear();
    }
    let bad = context::Assembly {
        manifest: assembly.manifest.clone(),
        bundle: agency_proto::Sealed::new(bytes).unwrap(),
    };
    assert_eq!(
        context::save_assembly(&mut store, &actor, "p", "bad-bytes", &bad)
            .unwrap_err()
            .code,
        "DELIVERY_DIGEST_MISMATCH"
    );
    let mut wrong = assembly.bundle.document.clone();
    wrong.manifest = reference("other-manifest", &hash(b"other"));
    let bad = context::Assembly {
        manifest: assembly.manifest,
        bundle: agency_proto::Sealed::new(wrong).unwrap(),
    };
    assert!(context::save_assembly(&mut store, &actor, "p", "bad-link", &bad).is_err());
    assert!(store.list("context_manifest").unwrap().is_empty());
    assert!(store.list("context_bundle").unwrap().is_empty());
}

#[test]
fn a_duplicate_entry_cannot_replace_another_manifest_source() {
    let mut store = temp_store();
    let actor = TrustedActor(agency_proto_to_store_actor());
    let mut sources = MemorySources::new();
    let first = reference("chat_source_reference/one", &hash(b"one"));
    let second = reference("chat_source_reference/two", &hash(b"two"));
    sources.push(SourceKind::Room, first.clone(), b"one");
    sources.push(SourceKind::Room, second.clone(), b"two");
    let permitted = ["chat_source_reference/one", "chat_source_reference/two"];
    let assembly = assembler(&permitted, 1024)
        .assemble(
            &sources,
            AssemblyRequest {
                manifest: manifest(vec![first, second], 1024, &permission_digest(&permitted)),
                consumer: owner(),
            },
        )
        .unwrap();
    let mut bundle = assembly.bundle.document;
    bundle.entries[1] = bundle.entries[0].clone();
    let bad = context::Assembly {
        manifest: assembly.manifest,
        bundle: agency_proto::Sealed::new(bundle).unwrap(),
    };
    assert_eq!(
        context::save_assembly(&mut store, &actor, "p", "duplicate-source", &bad)
            .unwrap_err()
            .code,
        "INVALID_INPUT"
    );
    assert!(store.list("context_bundle").unwrap().is_empty());
}

#[test]
fn readback_rejects_bad_seals_and_mismatched_admitted_material() {
    let mut store = temp_store();
    let actor = TrustedActor(agency_proto_to_store_actor());
    let source = reference("chat_source_reference/s1", &hash(b"body"));
    let mut sources = MemorySources::new();
    sources.push(SourceKind::Room, source.clone(), b"body");
    let assembly = assembler(&["chat_source_reference/s1"], 1024)
        .assemble(
            &sources,
            AssemblyRequest {
                manifest: manifest(
                    vec![source],
                    1024,
                    &permission_digest(&["chat_source_reference/s1"]),
                ),
                consumer: owner(),
            },
        )
        .unwrap();
    context::save_assembly(&mut store, &actor, "p", "readback", &assembly).unwrap();
    let original = context::bundle_record(&store, "p", &assembly.bundle.document.id)
        .unwrap()
        .unwrap();
    let mut bad = original.clone();
    if let RecordData::Value { value } = &mut bad.data {
        value["digest"] = json!("0".repeat(64));
    }
    bad.version = 2;
    put_corrupted_fixture(&mut store, &actor, &bad, None);
    assert_eq!(
        context::read_bundle(&store, &actor, "p", &original.key.id)
            .unwrap_err()
            .code,
        "DIGEST_MISMATCH"
    );

    let material = store
        .save_material(
            store.generation(),
            &actor,
            &original.key.scope,
            "corrupt-3",
            "body",
            b"{}",
        )
        .unwrap();
    let mut bad = original.clone();
    bad.version = 3;
    bad.materials = vec![material.clone()];
    put_corrupted_fixture(&mut store, &actor, &bad, Some(&material));
    assert_eq!(
        context::read_bundle(&store, &actor, "p", &original.key.id)
            .unwrap_err()
            .code,
        "MATERIAL_DIGEST_MISMATCH"
    );
    let mut missing = original;
    missing.version = 4;
    missing.materials.clear();
    put_corrupted_fixture(&mut store, &actor, &missing, None);
    assert_eq!(
        context::read_bundle(&store, &actor, "p", &missing.key.id)
            .unwrap_err()
            .code,
        "INVALID_INPUT"
    );
}

// Deliberately bypass the Context writer's validation to simulate a damaged
// stored record. Each replacement still uses Store's transaction and CAS.
fn put_corrupted_fixture(
    store: &mut Store,
    actor: &TrustedActor,
    record: &Record,
    material: Option<&store::MaterialRef>,
) {
    let key = format!("corrupt-{}", record.version);
    let input = json!({"version":record.version});
    let command = store::Command {
        command_id: key.clone(),
        idempotency_key: key,
        actor: actor.0.clone(),
        target: record.key.clone(),
        expected: store::Expected::Exact(Version::State(record.version - 1)),
        binding: Reference {
            key: store::ObjectKey {
                scope: Scope::Control,
                kind: "module".into(),
                id: "context".into(),
            },
            version: Version::State(1),
        },
        input_digest: store::Command::digest_input("fixture.corrupt", &input).unwrap(),
        operation: "fixture.corrupt".into(),
        input,
    };
    store
        .submit(store.generation(), actor, &command, None, |tx| {
            if let Some(material) = material {
                tx.admit_material(material)?;
            }
            tx.put(record)?;
            Ok(json!({}))
        })
        .unwrap();
}

/// The store-backed source adapter: version moved → typed stale error; the
/// admitted material bytes come back exactly.
#[test]
fn store_sources_return_exact_bytes_or_stale() {
    let mut store = temp_store();
    let actor = TrustedActor(agency_proto_to_store_actor());
    let scope = Scope::Project("p".into());
    let record = seeded_record(
        &mut store,
        &actor,
        &scope,
        "chat_source_reference",
        "main",
        b"room body",
    );
    let frozen = frozen_from_record(&record).unwrap();
    {
        let adapter = context::StoreSources::new(&store, &actor, "p");
        let content = adapter.exact(SourceKind::Room, &frozen).unwrap();
        assert_eq!(content.bytes, b"room body".to_vec());
        let mut wrong_version = frozen.clone();
        wrong_version.revision = hash(b"wrong-version");
        assert_eq!(
            adapter
                .exact(SourceKind::Room, &wrong_version)
                .unwrap_err()
                .code,
            "SOURCE_VERSION_CHANGED"
        );
    }
    // Move the record: the frozen reference must go stale.
    let mut moved = record.clone();
    moved.version += 1;
    moved.data = RecordData::Value {
        value: json!({"body":"moved"}),
    };
    let put = store::Command {
        command_id: "move".into(),
        idempotency_key: "move".into(),
        actor: actor.0.clone(),
        target: record.key.clone(),
        expected: store::Expected::Exact(Version::State(record.version)),
        binding: Reference {
            key: store::ObjectKey {
                scope: Scope::Control,
                kind: "module".into(),
                id: "context".into(),
            },
            version: Version::State(1),
        },
        input_digest: store::Command::digest_input("fixture.move", &json!({"to": moved.version}))
            .unwrap(),
        operation: "fixture.move".into(),
        input: json!({"to": moved.version}),
    };
    store
        .submit(store.generation(), &actor, &put, None, |tx| {
            tx.put(&moved)?;
            Ok(json!({}))
        })
        .unwrap();
    let error = context::StoreSources::new(&store, &actor, "p")
        .exact(SourceKind::Room, &frozen)
        .unwrap_err();
    assert_eq!(error.code, "SOURCE_VERSION_CHANGED");
}

// ---------------------------------------------------------------- helpers

fn agency_proto_to_store_actor() -> store::Actor {
    store::Actor {
        principal: "owner".into(),
        source: store::ActorSource::DirectClient,
        permission_scope: vec![Scope::Control, Scope::Project("p".into())],
        authority: None,
    }
}

fn temp_store() -> Store {
    let root = std::env::temp_dir().join(format!(
        "hctl-context-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    Store::open(&root).unwrap()
}

fn seeded_record(
    store: &mut Store,
    actor: &TrustedActor,
    scope: &Scope,
    kind: &str,
    id: &str,
    bytes: &[u8],
) -> Record {
    let command_key = format!("seed-{kind}-{id}");
    let material = store
        .save_material(
            store.generation(),
            actor,
            scope,
            &command_key,
            "body",
            bytes,
        )
        .unwrap();
    let key = store::ObjectKey {
        scope: scope.clone(),
        kind: kind.into(),
        id: id.into(),
    };
    let value = json!({"body": format!("{} bytes", bytes.len())});
    let digest = foundation::canonical_json_sha256(&value).unwrap();
    let mut record = Record {
        key: key.clone(),
        version: 1,
        revision_digest: digest.clone(),
        data: RecordData::Value { value },
        sources: vec![],
        materials: vec![material.clone()],
    };
    let command = store::Command {
        command_id: format!("seed-{kind}-{id}"),
        idempotency_key: format!("seed-{kind}-{id}"),
        actor: actor.0.clone(),
        target: key.clone(),
        expected: store::Expected::Absent,
        binding: Reference {
            key: store::ObjectKey {
                scope: Scope::Control,
                kind: "module".into(),
                id: "context".into(),
            },
            version: Version::State(1),
        },
        input_digest: store::Command::digest_input(
            "fixture.seed",
            &json!({"kind": kind, "id": id}),
        )
        .unwrap(),
        operation: "fixture.seed".into(),
        input: json!({"kind": kind, "id": id}),
    };
    store
        .submit(store.generation(), actor, &command, None, |tx| {
            tx.admit_material(&material)?;
            tx.put(&record)?;
            Ok(json!({}))
        })
        .unwrap();
    record.revision_digest = digest;
    record.materials = vec![material];
    record
}

fn entry_pointer(id: String, bytes: Vec<u8>) -> Entry {
    let safe = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    Entry {
        source: FrozenRef {
            id,
            revision: "r".into(),
            digest: hash(&bytes),
        },
        description: "pointer".into(),
        required: false,
        offline_required: true,
        delivery: Delivery::Pointer {
            bytes,
            relative_name: format!("{safe}.bin"),
        },
        bytes_digest: String::new(),
    }
}

fn minimal_bundle(entries: Vec<Entry>) -> Bundle {
    Bundle {
        id: "bundle-test".into(),
        manifest: reference("manifest/m", &"0".repeat(64)),
        consumer: owner(),
        entries,
        renderer: reference("renderer/plain", &hash(b"renderer")),
        tokenizer: reference("tokenizer/none", &hash(b"none")),
        redaction: reference("redaction/default", &hash(b"redaction")),
        compression: vec![],
        candidate_tokens: None,
        selected_tokens: None,
        delivered_tokens: None,
        permission_digest: hash(b"perm"),
        budget: 1024,
        retention: "until-owner-terminal".into(),
    }
}

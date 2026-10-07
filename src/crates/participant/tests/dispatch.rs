//! A contact fact belongs to the dispatch it is about, not to one look at it.
use participant::{decode, key, record_unreachable, reference, value};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use store::{
    Actor, ActorSource, Command, Expected, Record, Reference, Scope, Store, TrustedActor, Version,
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Root(PathBuf);
impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn scope() -> Scope {
    Scope::Project("project".into())
}
/// The reconcile loop reduces under the original authorization, and the store
/// requires an internal reducer to name one.
fn actor(principal: &str) -> TrustedActor {
    TrustedActor(Actor {
        principal: principal.into(),
        source: ActorSource::InternalReducer,
        permission_scope: vec![scope()],
        authority: Some(Reference {
            key: key(scope(), "room_invocation", "call"),
            version: Version::State(1),
        }),
    })
}
fn open() -> (Store, Root) {
    let root = std::env::temp_dir().join(format!(
        "hctl-contact-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    (Store::open(&root).unwrap(), Root(root))
}
/// `record_unreachable` reads the dispatch's key and reference only, so a record
/// standing in for one is enough to exercise the contact fact.
fn dispatch(store: &mut Store, actor: &TrustedActor) -> Record {
    let record = value(
        key(scope(), "dispatch", "dispatch-1"),
        1,
        &json!({"reference":"dispatch-1"}),
    )
    .unwrap();
    let input = serde_json::to_value(&record).unwrap();
    let command = Command {
        command_id: "dispatch:dispatch-1".into(),
        idempotency_key: "dispatch:dispatch-1".into(),
        actor: actor.0.clone(),
        target: record.key.clone(),
        expected: Expected::Absent,
        binding: reference(&record),
        input_digest: Command::digest_input("dispatch.prepare", &input).unwrap(),
        operation: "dispatch.prepare".into(),
        input,
    };
    store
        .submit(store.generation(), actor, &command, None, |tx| {
            tx.put(&record)?;
            Ok(json!({"record":record}))
        })
        .unwrap();
    record
}

#[test]
fn one_unreachable_dispatch_is_one_contact_fact_whatever_the_clock_or_the_actor() {
    let (mut store, _root) = open();
    let reconcile = actor("invocation-reducer:call");
    let dispatch = dispatch(&mut store, &reconcile);

    record_unreachable(&mut store, &reconcile, &dispatch, 1_000).unwrap();
    assert_eq!(store.list("dispatch_contact").unwrap().len(), 1);
    // The next tick meets the same fact a millisecond later. When control saw it is
    // data about the contact, so the clock must not turn it into a second record.
    record_unreachable(&mut store, &reconcile, &dispatch, 1_001).unwrap();
    assert_eq!(
        store.list("dispatch_contact").unwrap().len(),
        1,
        "a later tick meets the same fact"
    );
    // A second authority reporting the same fact must not collide with the first
    // command's identity either: that replaces "unreachable" with a store error.
    record_unreachable(
        &mut store,
        &actor("invocation-reducer:other"),
        &dispatch,
        1_001,
    )
    .unwrap();

    let recorded = store.list("dispatch_contact").unwrap();
    assert_eq!(recorded.len(), 1, "one dispatch, one contact fact");
    let data: Value = decode(&recorded[0]).unwrap();
    assert_eq!(data["contact"], "unreachable");
    assert_eq!(data["dispatch"], "dispatch-1");
    assert_eq!(
        data["observed_ms"],
        json!(1_000),
        "the first sighting is the recorded one"
    );
    assert_eq!(recorded[0].sources, vec![reference(&dispatch)]);
}

#![allow(dead_code)]

use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use store::*;

static SERIAL: AtomicU64 = AtomicU64::new(0);

pub struct Temp(pub PathBuf);
impl Temp {
    pub fn new() -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "hctl2-store-{}-{time}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
    pub fn store(&self) -> Store {
        Store::open(&self.0.join("control")).unwrap()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

pub fn key(id: &str) -> ObjectKey {
    ObjectKey {
        scope: Scope::Project("p".into()),
        kind: "fixture".into(),
        id: id.into(),
    }
}
pub fn reference(key: ObjectKey) -> Reference {
    Reference {
        key,
        version: Version::State(1),
    }
}
pub fn actor() -> TrustedActor {
    TrustedActor(Actor {
        principal: "human".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![
            Scope::Control,
            Scope::Project("p".into()),
            Scope::Project("q".into()),
            Scope::Repo("r".into()),
        ],
        authority: None,
    })
}
pub fn command(id: &str, target: ObjectKey) -> Command {
    Command {
        command_id: id.into(),
        idempotency_key: id.into(),
        actor: actor().0,
        target,
        expected: Expected::Absent,
        binding: reference(ObjectKey {
            scope: Scope::Control,
            kind: "control_binding".into(),
            id: "local".into(),
        }),
        input_digest: Command::digest_input("fixture", &json!({})).unwrap(),
        operation: "fixture".into(),
        input: json!({}),
    }
}
pub fn record(id: &str, version: i64) -> Record {
    Record {
        key: key(id),
        version,
        revision_digest: "a".repeat(64),
        data: RecordData::Value {
            value: json!({"version":version}),
        },
        sources: vec![],
        materials: vec![],
    }
}
pub fn effect(id: &str) -> EffectIntent {
    EffectIntent {
        intent_id: id.into(),
        owner: reference(key("object")),
        binding: reference(ObjectKey {
            scope: Scope::Repo("r".into()),
            kind: "platform_binding".into(),
            id: "platform".into(),
        }),
        operation: "update".into(),
        target: "platform/resource".into(),
        conflict_scope: "provider/account/resource".into(),
        permission_scope: Scope::Project("p".into()),
        input: json!({"value":"next"}),
        input_digest: Command::digest_input("update", &json!({"value":"next"})).unwrap(),
        idempotency_key: id.into(),
    }
}
pub fn submit_record(store: &mut Store, id: &str, value: &Record) {
    let mut cmd = command(id, value.key.clone());
    cmd.expected = if value.version == 1 {
        Expected::Absent
    } else {
        Expected::Exact(Version::State(value.version - 1))
    };
    store
        .submit(store.generation(), &actor(), &cmd, None, |tx| {
            tx.put(value)?;
            Ok(json!({"version":value.version}))
        })
        .unwrap();
}
pub fn count(root: &Path, table: &str) -> i64 {
    let conn = rusqlite::Connection::open(root.join("control.sqlite")).unwrap();
    conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
pub fn assert_code<T>(result: store::Result<T>, code: &str) {
    match result {
        Err(error) => assert_eq!(error.code, code, "{error}"),
        Ok(_) => panic!("expected {code}"),
    }
}

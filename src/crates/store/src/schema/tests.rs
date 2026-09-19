use super::*;
use crate::{StartupStatus, Store};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        let nonce: String = Connection::open_in_memory()
            .unwrap()
            .query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0))
            .unwrap();
        let path = std::env::temp_dir().join(format!("hctl-store-migration-{nonce}"));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn old_database(root: &Path) -> String {
    let mut conn = Connection::open(root.join("control.sqlite")).unwrap();
    Migrations::new(vec![M::up(IDENTITY)])
        .to_latest(&mut conn)
        .unwrap();
    identity(&conn).unwrap().0
}

#[test]
fn old_schema_upgrades_before_ready_and_identity_does_not_change() {
    let temp = Temp::new();
    let id = old_database(&temp.0);
    let status = StartupStatus::default();
    let probe = status.clone();
    let seen = Arc::new(AtomicBool::new(false));
    let hook_seen = Arc::clone(&seen);
    let plan = Migrations::new(vec![
        M::up(IDENTITY),
        M::up_with_hook(CORE, move |_| {
            assert_eq!(
                probe.require_ready().unwrap_err().code,
                "UPGRADE_IN_PROGRESS"
            );
            hook_seen.store(true, Ordering::SeqCst);
            Ok(())
        }),
    ]);
    let store = Store::open_migrations(&temp.0, status.clone(), &plan).unwrap();
    assert!(seen.load(Ordering::SeqCst));
    assert_eq!(store.control_id(), id);
    status.require_ready().unwrap();
    assert_eq!(inspect(&store.conn).unwrap(), VERSION);
}

#[test]
fn failed_migration_restores_verified_snapshot_and_keeps_identity() {
    let temp = Temp::new();
    let id = old_database(&temp.0);
    let plan = Migrations::new(vec![
        M::up(IDENTITY),
        M::up(
            "UPDATE control_identity SET control_id='wrong'; CREATE TABLE partial(x); SELECT * FROM missing_table;",
        ),
    ]);
    let status = StartupStatus::default();
    let error = Store::open_migrations(&temp.0, status.clone(), &plan)
        .err()
        .unwrap();
    assert_eq!(error.code, "MIGRATION_FAILED");
    assert!(status.require_ready().is_err());
    let conn = Connection::open(temp.0.join("control.sqlite")).unwrap();
    assert_eq!(inspect(&conn).unwrap(), 1);
    assert_eq!(identity(&conn).unwrap().0, id);
    assert!(
        !conn
            .prepare("SELECT 1 FROM sqlite_schema WHERE name='partial'")
            .unwrap()
            .exists([])
            .unwrap()
    );
    drop(conn);
    assert_eq!(Store::open(&temp.0).unwrap().control_id(), id);
}

#[test]
fn successful_sql_that_changes_identity_is_rolled_back() {
    let temp = Temp::new();
    let id = old_database(&temp.0);
    let sql =
        format!("{CORE} UPDATE control_identity SET control_id='wrong',writer_generation=10;");
    let plan = Migrations::new(vec![M::up(IDENTITY), M::up(&sql)]);
    let error = Store::open_migrations(&temp.0, StartupStatus::default(), &plan)
        .err()
        .unwrap();
    assert_eq!(error.code, "MIGRATION_FAILED");
    let conn = Connection::open(temp.0.join("control.sqlite")).unwrap();
    assert_eq!(inspect(&conn).unwrap(), 1);
    assert_eq!(identity(&conn).unwrap(), (id, 0));
}

#[test]
fn migration_crash_child() {
    let Ok(root) = std::env::var("HCTL2_MIGRATION_CRASH_ROOT") else {
        return;
    };
    let plan = Migrations::new(vec![
        M::up(IDENTITY),
        M::up_with_hook(CORE, |_| {
            std::process::exit(73);
        }),
    ]);
    let _ = Store::open_migrations(Path::new(&root), StartupStatus::default(), &plan);
    panic!("migration crash hook was not reached");
}

#[test]
fn process_loss_mid_migration_preserves_old_schema_and_identity() {
    let temp = Temp::new();
    let id = old_database(&temp.0);
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "schema::tests::migration_crash_child",
            "--nocapture",
        ])
        .env("HCTL2_MIGRATION_CRASH_ROOT", &temp.0)
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(73));
    let conn = Connection::open(temp.0.join("control.sqlite")).unwrap();
    assert_eq!(inspect(&conn).unwrap(), 1);
    assert_eq!(identity(&conn).unwrap(), (id.clone(), 0));
    drop(conn);
    assert_eq!(Store::open(&temp.0).unwrap().control_id(), id);
}

#[test]
fn future_foreign_and_half_schema_are_rejected_without_relabeling() {
    for sql in [
        "PRAGMA user_version=99",
        "PRAGMA application_id=42",
        "DROP TABLE commands",
    ] {
        let temp = Temp::new();
        let store = Store::open(&temp.0).unwrap();
        let id = store.control_id().to_owned();
        drop(store);
        let conn = Connection::open(temp.0.join("control.sqlite")).unwrap();
        conn.execute_batch(sql).unwrap();
        let previous = identity(&conn).unwrap();
        drop(conn);
        assert!(Store::open(&temp.0).is_err());
        let conn = Connection::open(temp.0.join("control.sqlite")).unwrap();
        assert_eq!(identity(&conn).unwrap(), previous);
        assert_eq!(previous.0, id);
    }
}

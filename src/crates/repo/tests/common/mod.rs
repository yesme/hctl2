#![allow(dead_code)]
use repo::{Origin, Platform, Register};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use store::{Actor, ActorSource, Scope, TrustedActor};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub struct Temp(pub PathBuf);
impl Temp {
    pub fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "hctl2-repo-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
pub fn actor() -> TrustedActor {
    TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control],
        authority: None,
    })
}
pub fn request(platform: Platform) -> Register {
    Register {
        name: "example".into(),
        origin: Origin::Local,
        platform: Some(platform.clone()),
        instance: None,
        platform_repo_id: None,
        platform_path: if platform == Platform::Local {
            Some("example".into())
        } else {
            None
        },
        local: None,
        remote_evidence: None,
        default_source: None,
    }
}

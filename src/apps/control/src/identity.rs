//! Connection identity comes from Unix peer credentials, never from client fields.

use store::{Actor, ActorSource, Scope, TrustedActor};

/// Construct the trusted actor from the connecting process uid.
#[must_use]
pub fn owner_actor(uid: u32) -> TrustedActor {
    TrustedActor(Actor {
        principal: format!("local-owner:{uid}"),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control],
        authority: None,
    })
}

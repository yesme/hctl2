//! Connection identity. The Unix socket is already owner-only; actor is the local owner.

use store::{Actor, ActorSource, Scope, TrustedActor};

/// Construct the trusted actor from the local owner connection, never from client fields.
#[must_use]
pub fn owner_actor() -> TrustedActor {
    TrustedActor(Actor {
        principal: "local-owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control],
        authority: None,
    })
}

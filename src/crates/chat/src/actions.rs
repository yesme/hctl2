//! Both client and provider paths use this normalization after a current Matrix read.
use crate::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use store::{Reference, Scope, Version};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityPolicy {
    pub binding: Reference,
    /// Exact Matrix ID -> human principal. Display names never participate.
    pub humans: BTreeMap<String, String>,
    pub service_users: Vec<String>,
    pub allowed_actions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HumanAction {
    pub principal: String,
    pub binding: Reference,
    pub event_id: String,
    pub target: Reference,
    pub input: Input,
    pub digest: String,
}

/// A normal message cannot opt into this protocol merely by containing JSON in its body.
/// The adapter supplies an event read by its immutable ID, not a client-claimed sender.
pub fn normalize_human_action(
    policy: &IdentityPolicy,
    project: &str,
    event: &Value,
) -> Result<HumanAction> {
    if event["type"] != "io.hctl2.action" {
        return Err(invalid("ordinary messages and reactions are not commands"));
    }
    let sender = event["sender"]
        .as_str()
        .ok_or_else(|| invalid("source actor missing"))?;
    if sender.starts_with("@hctl2_") || policy.service_users.iter().any(|u| u == sender) {
        return Err(reject(
            "ACTOR_NOT_HUMAN",
            "service or bridge event cannot assert a human actor",
            "use_human_client",
        ));
    }
    let principal = policy
        .humans
        .get(sender)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            reject(
                "ACTOR_UNMAPPED",
                "Matrix actor has no exact human mapping",
                "configure_identity_mapping",
            )
        })?;
    let event_id = event["event_id"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid("source event ID missing"))?;
    let target: Reference = serde_json::from_value(event["content"]["target"].clone())?;
    let action: Action = serde_json::from_value(event["content"]["action"].clone())?;
    let kind = event["content"]["action"]["kind"]
        .as_str()
        .ok_or_else(|| invalid("action kind missing"))?;
    let matches_target = match &action {
        Action::CreateTopic {
            project_id,
            project_version,
            ..
        } => {
            target.key.kind == "project"
                && target.key.id == *project_id
                && target.version == Version::State(*project_version)
        }
        Action::Close {
            room_id, version, ..
        }
        | Action::Rebind {
            room_id, version, ..
        }
        | Action::Send {
            room_id, version, ..
        }
        | Action::Freeze {
            room_id, version, ..
        } => {
            target.key.kind == "room_binding"
                && target.key.id == *room_id
                && target.version == Version::State(*version)
        }
        Action::Resume { .. } => false,
    };
    if !policy.allowed_actions.iter().any(|k| k == kind)
        || !matches_target
        || action.project_id() != project
        || target.key.scope != Scope::Project(project.into())
    {
        return Err(reject(
            "ACTION_NOT_ALLOWED",
            "action or target not allowed by binding",
            "request_authorization",
        ));
    }
    let value = serde_json::json!({"binding":policy.binding,"actor":principal,"event":event_id,"target":target,"action":action});
    let digest = foundation::canonical_json_sha256(&value)?;
    Ok(HumanAction {
        principal: principal.clone(),
        binding: policy.binding.clone(),
        event_id: event_id.into(),
        target,
        input: Input {
            key: format!("matrix-action:{digest}"),
            action,
        },
        digest,
    })
}

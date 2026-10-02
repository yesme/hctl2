use serde::{Deserialize, Serialize};
use store::{EffectIntent, MaterialRef, ObjectKey, Record, Reference};

/// Deployment endpoint and its exact version. Tokens never enter this value.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    pub binding: Reference,
    pub url: String,
    pub server_name: String,
    pub sender: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Room {
    pub project_id: String,
    pub id: String,
    pub name: String,
    pub server: Server,
    pub matrix_room_id: Option<String>,
    /// Empty is a legitimate, independently confirmed roster, not inherited membership.
    pub participants: Vec<Reference>,
    pub brief: Option<MaterialRef>,
    pub origin: Option<Origin>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Source {
    Message {
        binding: Reference,
        event_id: String,
        content_digest: String,
    },
    Object {
        reference: Reference,
        content_digest: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Origin {
    #[serde(alias = "main_room")]
    Room {
        room_id: String,
        binding_version: i64,
    },
    Request {
        request: Reference,
        blockers: Vec<Reference>,
    },
}

/// Read from the authoritative source by the adapter, never deserialized from a command.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceText {
    pub source: Source,
    /// Exact source bytes (message content or the frozen object's canonical JSON).
    pub body: String,
    /// Exact visible field value used by the mechanical draft.
    pub excerpt: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brief {
    pub context_and_goal: String,
    pub settled_facts_and_reasons: Vec<String>,
    pub disagreements_and_questions: Vec<String>,
    pub constraints_and_materials: Vec<String>,
    pub sources: Vec<Source>,
}

/// Frozen Request shape consumed here; creation and resolution belong to P2.2 辛.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestSource {
    pub question: String,
    pub blockers: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub key: String,
    pub action: Action,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    CreateTopic {
        project_id: String,
        project_version: i64,
        name: String,
        // Boxed so one rare variant does not set the size of every action:
        // Input holds an Action inline, so an unboxed Brief and Origin would be
        // paid by every event. Serde is transparent through Box, so the
        // persisted JSON shape is unchanged.
        origin: Box<Origin>,
        brief: Box<Brief>,
        /// Confirmation of this Room's own selection; no implicit inherited roster.
        participants: Vec<Reference>,
        roster_confirmed: bool,
    },
    Close {
        project_id: String,
        room_id: String,
        version: i64,
    },
    Rebind {
        project_id: String,
        room_id: String,
        version: i64,
        matrix_room_id: String,
    },
    Send {
        project_id: String,
        room_id: String,
        version: i64,
        body: String,
        #[serde(default)]
        thread_root: Option<String>,
    },
    Freeze {
        project_id: String,
        room_id: String,
        version: i64,
        event_id: String,
    },
    Resume {
        project_id: String,
        effect_id: String,
    },
}

impl Action {
    pub fn project_id(&self) -> &str {
        match self {
            Self::CreateTopic { project_id, .. }
            | Self::Close { project_id, .. }
            | Self::Rebind { project_id, .. }
            | Self::Send { project_id, .. }
            | Self::Freeze { project_id, .. }
            | Self::Resume { project_id, .. } => project_id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Check {
    pub key: ObjectKey,
    pub version: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub input: Input,
    pub checks: Vec<Check>,
    pub records: Vec<Record>,
    pub effects: Vec<EffectIntent>,
    pub source_texts: Vec<SourceText>,
    pub result: serde_json::Value,
}

/// Mechanical selection is structural only. A caller may select explicit event IDs or
/// a server-ordered range; replies and mentions filter on protocol relations, not prose.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Selection {
    Events {
        event_ids: Vec<String>,
    },
    Range {
        start: String,
        end: String,
    },
    Replies {
        event_id: String,
    },
    Thread {
        event_id: String,
    },
    Mentions {
        user_id: String,
        #[serde(default)]
        after: Option<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DraftInput {
    pub project_id: String,
    pub project_version: i64,
    pub origin: Origin,
    pub selection: Option<Selection>,
    /// Optional exact Message references when upgrading a Request.
    #[serde(default)]
    pub messages: Vec<Source>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Draft {
    pub automatic_summary: String,
    pub fragments: Vec<SourceText>,
    pub unread: Vec<Source>,
    pub rule_reference: String,
    pub rule_digest: String,
}

/// Extension point only. Installing a model engine is not part of the Room package.
pub trait BriefDrafter {
    fn draft(&self, allowed: &[SourceText]) -> crate::Result<Draft>;
}

#[allow(unused_imports)]
pub use progenitor_client::{ByteStream, ClientInfo, Error, ResponseValue};
#[allow(unused_imports)]
use progenitor_client::{encode_path, ClientHooks, OperationInfo, RequestBuilderExt};
/// Types used as operation parameters and responses.
#[allow(clippy::all)]
pub mod types {
    fn deserialize_null_default<'de, D, T>(
        deserializer: D,
    ) -> ::std::result::Result<T, D::Error>
    where
        T: ::std::default::Default + ::serde::Deserialize<'de>,
        D: ::serde::Deserializer<'de>,
    {
        use ::serde::Deserialize as _;
        Ok(::std::option::Option::<T>::deserialize(deserializer)?.unwrap_or_default())
    }
    /// Error types.
    pub mod error {
        /// Error from a `TryFrom` or `FromStr` implementation.
        pub struct ConversionError(::std::borrow::Cow<'static, str>);
        impl ::std::error::Error for ConversionError {}
        impl ::std::fmt::Display for ConversionError {
            fn fmt(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }
        impl ::std::fmt::Debug for ConversionError {
            fn fmt(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> Result<(), ::std::fmt::Error> {
                ::std::fmt::Debug::fmt(&self.0, f)
            }
        }
        impl From<&'static str> for ConversionError {
            fn from(value: &'static str) -> Self {
                Self(value.into())
            }
        }
        impl From<String> for ConversionError {
            fn from(value: String) -> Self {
                Self(value.into())
            }
        }
    }
    ///One entry on an agent DAG-run's decision timeline
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "One entry on an agent DAG-run's decision timeline",
    ///  "type": "object",
    ///  "required": [
    ///    "kind",
    ///    "turn"
    ///  ],
    ///  "properties": {
    ///    "attempt": {
    ///      "description": "Which run of this step it was, starting at 1",
    ///      "type": "integer"
    ///    },
    ///    "childDagName": {
    ///      "description": "Name of the child DAG that ran",
    ///      "type": "string"
    ///    },
    ///    "childDagRunId": {
    ///      "description": "Child DAG-run this action produced, for linking to its run page. Absent for steps that run no child DAG.",
    ///      "type": "string"
    ///    },
    ///    "finishedAt": {
    ///      "description": "RFC3339 timestamp when the step finished",
    ///      "type": "string"
    ///    },
    ///    "kind": {
    ///      "description": "What the agent did on this turn",
    ///      "type": "string",
    ///      "enum": [
    ///        "action",
    ///        "task_status",
    ///        "ask_user",
    ///        "rejected",
    ///        "stalled"
    ///      ]
    ///    },
    ///    "name": {
    ///      "description": "Step or task the event concerns",
    ///      "type": "string"
    ///    },
    ///    "reason": {
    ///      "description": "Agent's justification, or why the call was rejected",
    ///      "type": "string"
    ///    },
    ///    "startedAt": {
    ///      "description": "RFC3339 timestamp when the step started",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "description": "Resulting step status for an action event, or the new task status for a task_status event",
    ///      "type": "string"
    ///    },
    ///    "turn": {
    ///      "description": "Agent turn this event belongs to, starting at 1",
    ///      "type": "integer"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AgentEvent {
        ///Which run of this step it was, starting at 1
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub attempt: ::std::option::Option<i64>,
        ///Name of the child DAG that ran
        #[serde(
            rename = "childDagName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub child_dag_name: ::std::option::Option<::std::string::String>,
        ///Child DAG-run this action produced, for linking to its run page. Absent for steps that run no child DAG.
        #[serde(
            rename = "childDagRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub child_dag_run_id: ::std::option::Option<::std::string::String>,
        ///RFC3339 timestamp when the step finished
        #[serde(
            rename = "finishedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub finished_at: ::std::option::Option<::std::string::String>,
        ///What the agent did on this turn
        pub kind: AgentEventKind,
        ///Step or task the event concerns
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub name: ::std::option::Option<::std::string::String>,
        ///Agent's justification, or why the call was rejected
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub reason: ::std::option::Option<::std::string::String>,
        ///RFC3339 timestamp when the step started
        #[serde(
            rename = "startedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub started_at: ::std::option::Option<::std::string::String>,
        ///Resulting step status for an action event, or the new task status for a task_status event
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub status: ::std::option::Option<::std::string::String>,
        ///Agent turn this event belongs to, starting at 1
        pub turn: i64,
    }
    ///What the agent did on this turn
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "What the agent did on this turn",
    ///  "type": "string",
    ///  "enum": [
    ///    "action",
    ///    "task_status",
    ///    "ask_user",
    ///    "rejected",
    ///    "stalled"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum AgentEventKind {
        #[serde(rename = "action")]
        Action,
        #[serde(rename = "task_status")]
        TaskStatus,
        #[serde(rename = "ask_user")]
        AskUser,
        #[serde(rename = "rejected")]
        Rejected,
        #[serde(rename = "stalled")]
        Stalled,
    }
    impl ::std::fmt::Display for AgentEventKind {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Action => f.write_str("action"),
                Self::TaskStatus => f.write_str("task_status"),
                Self::AskUser => f.write_str("ask_user"),
                Self::Rejected => f.write_str("rejected"),
                Self::Stalled => f.write_str("stalled"),
            }
        }
    }
    impl ::std::str::FromStr for AgentEventKind {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "action" => Ok(Self::Action),
                "task_status" => Ok(Self::TaskStatus),
                "ask_user" => Ok(Self::AskUser),
                "rejected" => Ok(Self::Rejected),
                "stalled" => Ok(Self::Stalled),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for AgentEventKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for AgentEventKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for AgentEventKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`AgentInteraction`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "kind",
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "allowForSessionPatterns": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "answers": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "array",
    ///        "items": {
    ///          "type": "string"
    ///        }
    ///      }
    ///    },
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "decision": {
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "kind": {
    ///      "type": "string",
    ///      "enum": [
    ///        "permission",
    ///        "question"
    ///      ]
    ///    },
    ///    "patterns": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "permission": {
    ///      "type": "string"
    ///    },
    ///    "questions": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AgentQuestion"
    ///      }
    ///    },
    ///    "respondedAt": {
    ///      "type": "string"
    ///    },
    ///    "respondedBy": {
    ///      "type": "string"
    ///    },
    ///    "respondedById": {
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "type": "string",
    ///      "enum": [
    ///        "pending",
    ///        "answered",
    ///        "rejected"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AgentInteraction {
        #[serde(
            rename = "allowForSessionPatterns",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub allow_for_session_patterns: ::std::vec::Vec<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub answers: ::std::vec::Vec<::std::vec::Vec<::std::string::String>>,
        #[serde(
            rename = "createdAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub created_at: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub decision: ::std::option::Option<::std::string::String>,
        pub id: ::std::string::String,
        pub kind: AgentInteractionKind,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub patterns: ::std::vec::Vec<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub permission: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub questions: ::std::vec::Vec<AgentQuestion>,
        #[serde(
            rename = "respondedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub responded_at: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "respondedBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub responded_by: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "respondedById",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub responded_by_id: ::std::option::Option<::std::string::String>,
        pub status: AgentInteractionStatus,
    }
    ///`AgentInteractionKind`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "permission",
    ///    "question"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum AgentInteractionKind {
        #[serde(rename = "permission")]
        Permission,
        #[serde(rename = "question")]
        Question,
    }
    impl ::std::fmt::Display for AgentInteractionKind {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Permission => f.write_str("permission"),
                Self::Question => f.write_str("question"),
            }
        }
    }
    impl ::std::str::FromStr for AgentInteractionKind {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "permission" => Ok(Self::Permission),
                "question" => Ok(Self::Question),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for AgentInteractionKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for AgentInteractionKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for AgentInteractionKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`AgentInteractionResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunId",
    ///    "interactionId",
    ///    "resumed",
    ///    "stepName"
    ///  ],
    ///  "properties": {
    ///    "dagRunId": {
    ///      "type": "string"
    ///    },
    ///    "interactionId": {
    ///      "type": "string"
    ///    },
    ///    "resumed": {
    ///      "type": "boolean"
    ///    },
    ///    "stepName": {
    ///      "type": "string"
    ///    },
    ///    "subDAGRunId": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AgentInteractionResponse {
        #[serde(rename = "dagRunId")]
        pub dag_run_id: ::std::string::String,
        #[serde(rename = "interactionId")]
        pub interaction_id: ::std::string::String,
        pub resumed: bool,
        #[serde(rename = "stepName")]
        pub step_name: ::std::string::String,
        #[serde(
            rename = "subDAGRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub sub_dag_run_id: ::std::option::Option<::std::string::String>,
    }
    ///Answer to a pending managed-agent permission or question
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Answer to a pending managed-agent permission or question",
    ///  "type": "object",
    ///  "properties": {
    ///    "answers": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "array",
    ///        "items": {
    ///          "type": "string"
    ///        }
    ///      }
    ///    },
    ///    "decision": {
    ///      "type": "string",
    ///      "enum": [
    ///        "once",
    ///        "session",
    ///        "reject"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AgentInteractionResponseRequest {
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub answers: ::std::vec::Vec<::std::vec::Vec<::std::string::String>>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub decision: ::std::option::Option<AgentInteractionResponseRequestDecision>,
    }
    impl ::std::default::Default for AgentInteractionResponseRequest {
        fn default() -> Self {
            Self {
                answers: Default::default(),
                decision: Default::default(),
            }
        }
    }
    ///`AgentInteractionResponseRequestDecision`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "once",
    ///    "session",
    ///    "reject"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum AgentInteractionResponseRequestDecision {
        #[serde(rename = "once")]
        Once,
        #[serde(rename = "session")]
        Session,
        #[serde(rename = "reject")]
        Reject,
    }
    impl ::std::fmt::Display for AgentInteractionResponseRequestDecision {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Once => f.write_str("once"),
                Self::Session => f.write_str("session"),
                Self::Reject => f.write_str("reject"),
            }
        }
    }
    impl ::std::str::FromStr for AgentInteractionResponseRequestDecision {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "once" => Ok(Self::Once),
                "session" => Ok(Self::Session),
                "reject" => Ok(Self::Reject),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for AgentInteractionResponseRequestDecision {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for AgentInteractionResponseRequestDecision {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for AgentInteractionResponseRequestDecision {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`AgentInteractionStatus`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "pending",
    ///    "answered",
    ///    "rejected"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum AgentInteractionStatus {
        #[serde(rename = "pending")]
        Pending,
        #[serde(rename = "answered")]
        Answered,
        #[serde(rename = "rejected")]
        Rejected,
    }
    impl ::std::fmt::Display for AgentInteractionStatus {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Pending => f.write_str("pending"),
                Self::Answered => f.write_str("answered"),
                Self::Rejected => f.write_str("rejected"),
            }
        }
    }
    impl ::std::str::FromStr for AgentInteractionStatus {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "pending" => Ok(Self::Pending),
                "answered" => Ok(Self::Answered),
                "rejected" => Ok(Self::Rejected),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for AgentInteractionStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for AgentInteractionStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for AgentInteractionStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`AgentQuestion`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "header",
    ///    "question"
    ///  ],
    ///  "properties": {
    ///    "custom": {
    ///      "type": "boolean"
    ///    },
    ///    "header": {
    ///      "type": "string"
    ///    },
    ///    "multiple": {
    ///      "type": "boolean"
    ///    },
    ///    "options": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AgentQuestionOption"
    ///      }
    ///    },
    ///    "question": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AgentQuestion {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub custom: ::std::option::Option<bool>,
        pub header: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub multiple: ::std::option::Option<bool>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub options: ::std::vec::Vec<AgentQuestionOption>,
        pub question: ::std::string::String,
    }
    ///`AgentQuestionOption`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "label"
    ///  ],
    ///  "properties": {
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "label": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AgentQuestionOption {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        pub label: ::std::string::String,
    }
    ///Durable state for a managed coding-agent session
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Durable state for a managed coding-agent session",
    ///  "type": "object",
    ///  "required": [
    ///    "provider",
    ///    "state"
    ///  ],
    ///  "properties": {
    ///    "agent": {
    ///      "type": "string"
    ///    },
    ///    "events": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AgentSessionEvent"
    ///      }
    ///    },
    ///    "generation": {
    ///      "type": "integer"
    ///    },
    ///    "interactions": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AgentInteraction"
    ///      }
    ///    },
    ///    "lastError": {
    ///      "type": "string"
    ///    },
    ///    "model": {
    ///      "type": "string"
    ///    },
    ///    "provider": {
    ///      "type": "string"
    ///    },
    ///    "providerVersion": {
    ///      "type": "string"
    ///    },
    ///    "sessionId": {
    ///      "type": "string"
    ///    },
    ///    "state": {
    ///      "$ref": "#/components/schemas/AgentSessionState"
    ///    },
    ///    "usage": {
    ///      "$ref": "#/components/schemas/AgentUsage"
    ///    },
    ///    "variant": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AgentSession {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub agent: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub events: ::std::vec::Vec<AgentSessionEvent>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub generation: ::std::option::Option<i64>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub interactions: ::std::vec::Vec<AgentInteraction>,
        #[serde(
            rename = "lastError",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub last_error: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub model: ::std::option::Option<::std::string::String>,
        pub provider: ::std::string::String,
        #[serde(
            rename = "providerVersion",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub provider_version: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "sessionId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub session_id: ::std::option::Option<::std::string::String>,
        pub state: AgentSessionState,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub usage: ::std::option::Option<AgentUsage>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub variant: ::std::option::Option<::std::string::String>,
    }
    ///`AgentSessionEvent`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "sequence",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "content": {
    ///      "type": "string"
    ///    },
    ///    "files": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "role": {
    ///      "type": "string"
    ///    },
    ///    "sequence": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "status": {
    ///      "type": "string"
    ///    },
    ///    "timestamp": {
    ///      "type": "string"
    ///    },
    ///    "type": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AgentSessionEvent {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub content: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub files: ::std::vec::Vec<::std::string::String>,
        pub id: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub name: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub role: ::std::option::Option<::std::string::String>,
        pub sequence: i64,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub status: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub timestamp: ::std::option::Option<::std::string::String>,
        #[serde(rename = "type")]
        pub type_: ::std::string::String,
    }
    ///`AgentSessionRestartResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunId",
    ///    "generation",
    ///    "resumed",
    ///    "stepName"
    ///  ],
    ///  "properties": {
    ///    "dagRunId": {
    ///      "type": "string"
    ///    },
    ///    "generation": {
    ///      "type": "integer"
    ///    },
    ///    "resumed": {
    ///      "type": "boolean"
    ///    },
    ///    "stepName": {
    ///      "type": "string"
    ///    },
    ///    "subDAGRunId": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AgentSessionRestartResponse {
        #[serde(rename = "dagRunId")]
        pub dag_run_id: ::std::string::String,
        pub generation: i64,
        pub resumed: bool,
        #[serde(rename = "stepName")]
        pub step_name: ::std::string::String,
        #[serde(
            rename = "subDAGRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub sub_dag_run_id: ::std::option::Option<::std::string::String>,
    }
    ///`AgentSessionState`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "starting",
    ///    "running",
    ///    "waiting",
    ///    "succeeded",
    ///    "failed",
    ///    "aborted",
    ///    "unavailable"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum AgentSessionState {
        #[serde(rename = "starting")]
        Starting,
        #[serde(rename = "running")]
        Running,
        #[serde(rename = "waiting")]
        Waiting,
        #[serde(rename = "succeeded")]
        Succeeded,
        #[serde(rename = "failed")]
        Failed,
        #[serde(rename = "aborted")]
        Aborted,
        #[serde(rename = "unavailable")]
        Unavailable,
    }
    impl ::std::fmt::Display for AgentSessionState {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Starting => f.write_str("starting"),
                Self::Running => f.write_str("running"),
                Self::Waiting => f.write_str("waiting"),
                Self::Succeeded => f.write_str("succeeded"),
                Self::Failed => f.write_str("failed"),
                Self::Aborted => f.write_str("aborted"),
                Self::Unavailable => f.write_str("unavailable"),
            }
        }
    }
    impl ::std::str::FromStr for AgentSessionState {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "starting" => Ok(Self::Starting),
                "running" => Ok(Self::Running),
                "waiting" => Ok(Self::Waiting),
                "succeeded" => Ok(Self::Succeeded),
                "failed" => Ok(Self::Failed),
                "aborted" => Ok(Self::Aborted),
                "unavailable" => Ok(Self::Unavailable),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for AgentSessionState {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for AgentSessionState {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for AgentSessionState {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///A goal an agent DAG must satisfy before the run concludes
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A goal an agent DAG must satisfy before the run concludes",
    ///  "type": "object",
    ///  "required": [
    ///    "name",
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "description": {
    ///      "description": "Completion criteria the agent decides against",
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "description": "Unique task name",
    ///      "type": "string"
    ///    },
    ///    "reason": {
    ///      "description": "Justification the agent gave for the current status",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "description": "Where the task stands. The run ends once none is open, and fails if any is failed. A skipped task does not fail the run.",
    ///      "type": "string",
    ///      "enum": [
    ///        "open",
    ///        "completed",
    ///        "skipped",
    ///        "failed"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AgentTask {
        ///Completion criteria the agent decides against
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        ///Unique task name
        pub name: ::std::string::String,
        ///Justification the agent gave for the current status
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub reason: ::std::option::Option<::std::string::String>,
        ///Where the task stands. The run ends once none is open, and fails if any is failed. A skipped task does not fail the run.
        pub status: AgentTaskStatus,
    }
    ///Where the task stands. The run ends once none is open, and fails if any is failed. A skipped task does not fail the run.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Where the task stands. The run ends once none is open, and fails if any is failed. A skipped task does not fail the run.",
    ///  "type": "string",
    ///  "enum": [
    ///    "open",
    ///    "completed",
    ///    "skipped",
    ///    "failed"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum AgentTaskStatus {
        #[serde(rename = "open")]
        Open,
        #[serde(rename = "completed")]
        Completed,
        #[serde(rename = "skipped")]
        Skipped,
        #[serde(rename = "failed")]
        Failed,
    }
    impl ::std::fmt::Display for AgentTaskStatus {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Open => f.write_str("open"),
                Self::Completed => f.write_str("completed"),
                Self::Skipped => f.write_str("skipped"),
                Self::Failed => f.write_str("failed"),
            }
        }
    }
    impl ::std::str::FromStr for AgentTaskStatus {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "open" => Ok(Self::Open),
                "completed" => Ok(Self::Completed),
                "skipped" => Ok(Self::Skipped),
                "failed" => Ok(Self::Failed),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for AgentTaskStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for AgentTaskStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for AgentTaskStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`AgentUsage`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "cost": {
    ///      "type": "number",
    ///      "format": "double"
    ///    },
    ///    "inputTokens": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "outputTokens": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "reasoningTokens": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "totalTokens": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AgentUsage {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub cost: ::std::option::Option<f64>,
        #[serde(
            rename = "inputTokens",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub input_tokens: ::std::option::Option<i64>,
        #[serde(
            rename = "outputTokens",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub output_tokens: ::std::option::Option<i64>,
        #[serde(
            rename = "reasoningTokens",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub reasoning_tokens: ::std::option::Option<i64>,
        #[serde(
            rename = "totalTokens",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub total_tokens: ::std::option::Option<i64>,
    }
    impl ::std::default::Default for AgentUsage {
        fn default() -> Self {
            Self {
                cost: Default::default(),
                input_tokens: Default::default(),
                output_tokens: Default::default(),
                reasoning_tokens: Default::default(),
                total_tokens: Default::default(),
            }
        }
    }
    ///API key information
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "API key information",
    ///  "type": "object",
    ///  "required": [
    ///    "allowedSurfaces",
    ///    "attributionClass",
    ///    "createdAt",
    ///    "createdBy",
    ///    "id",
    ///    "keyPrefix",
    ///    "name",
    ///    "role",
    ///    "updatedAt",
    ///    "workspaceAccess"
    ///  ],
    ///  "properties": {
    ///    "allowedSurfaces": {
    ///      "description": "Interfaces where this API key may be accepted",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string",
    ///        "enum": [
    ///          "rest_api",
    ///          "mcp"
    ///        ]
    ///      },
    ///      "minItems": 1,
    ///      "uniqueItems": true
    ///    },
    ///    "attributionClass": {
    ///      "description": "Whether this key is owned by a user or represents a service account",
    ///      "type": "string",
    ///      "enum": [
    ///        "user_owned",
    ///        "service_account"
    ///      ]
    ///    },
    ///    "createdAt": {
    ///      "description": "Creation timestamp",
    ///      "type": "string"
    ///    },
    ///    "createdBy": {
    ///      "description": "Creator user ID",
    ///      "type": "string"
    ///    },
    ///    "description": {
    ///      "description": "Purpose description",
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "description": "Unique identifier",
    ///      "type": "string"
    ///    },
    ///    "keyPrefix": {
    ///      "description": "First 8 characters for identification",
    ///      "type": "string"
    ///    },
    ///    "lastUsedAt": {
    ///      "description": "Last authentication timestamp",
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "migratedAsServiceAccount": {
    ///      "description": "True when a legacy key missing attributionClass was defaulted to service_account",
    ///      "type": "boolean"
    ///    },
    ///    "name": {
    ///      "description": "Human-readable name",
    ///      "type": "string"
    ///    },
    ///    "ownerUserId": {
    ///      "description": "Owner user ID when attributionClass is user_owned",
    ///      "type": "string"
    ///    },
    ///    "ownerUsername": {
    ///      "description": "Owner username when attributionClass is user_owned",
    ///      "type": "string"
    ///    },
    ///    "role": {
    ///      "$ref": "#/components/schemas/UserRole"
    ///    },
    ///    "serviceAccountId": {
    ///      "description": "Service-account identifier when attributionClass is service_account",
    ///      "type": "string"
    ///    },
    ///    "serviceAccountName": {
    ///      "description": "Service-account display name when attributionClass is service_account",
    ///      "type": "string"
    ///    },
    ///    "updatedAt": {
    ///      "description": "Last update timestamp",
    ///      "type": "string"
    ///    },
    ///    "workspaceAccess": {
    ///      "$ref": "#/components/schemas/WorkspaceAccess"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ApiKey {
        ///Interfaces where this API key may be accepted
        #[serde(rename = "allowedSurfaces")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub allowed_surfaces: Vec<ApiKeyAllowedSurfacesItem>,
        ///Whether this key is owned by a user or represents a service account
        #[serde(rename = "attributionClass")]
        pub attribution_class: ApiKeyAttributionClass,
        ///Creation timestamp
        #[serde(rename = "createdAt")]
        pub created_at: ::std::string::String,
        ///Creator user ID
        #[serde(rename = "createdBy")]
        pub created_by: ::std::string::String,
        ///Purpose description
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        ///Unique identifier
        pub id: ::std::string::String,
        ///First 8 characters for identification
        #[serde(rename = "keyPrefix")]
        pub key_prefix: ::std::string::String,
        ///Last authentication timestamp
        #[serde(
            rename = "lastUsedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub last_used_at: ::std::option::Option<::std::string::String>,
        ///True when a legacy key missing attributionClass was defaulted to service_account
        #[serde(
            rename = "migratedAsServiceAccount",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub migrated_as_service_account: ::std::option::Option<bool>,
        ///Human-readable name
        pub name: ::std::string::String,
        ///Owner user ID when attributionClass is user_owned
        #[serde(
            rename = "ownerUserId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub owner_user_id: ::std::option::Option<::std::string::String>,
        ///Owner username when attributionClass is user_owned
        #[serde(
            rename = "ownerUsername",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub owner_username: ::std::option::Option<::std::string::String>,
        pub role: UserRole,
        ///Service-account identifier when attributionClass is service_account
        #[serde(
            rename = "serviceAccountId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub service_account_id: ::std::option::Option<::std::string::String>,
        ///Service-account display name when attributionClass is service_account
        #[serde(
            rename = "serviceAccountName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub service_account_name: ::std::option::Option<::std::string::String>,
        ///Last update timestamp
        #[serde(rename = "updatedAt")]
        pub updated_at: ::std::string::String,
        #[serde(rename = "workspaceAccess")]
        pub workspace_access: WorkspaceAccess,
    }
    ///`ApiKeyAllowedSurfacesItem`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "rest_api",
    ///    "mcp"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ApiKeyAllowedSurfacesItem {
        #[serde(rename = "rest_api")]
        RestApi,
        #[serde(rename = "mcp")]
        Mcp,
    }
    impl ::std::fmt::Display for ApiKeyAllowedSurfacesItem {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::RestApi => f.write_str("rest_api"),
                Self::Mcp => f.write_str("mcp"),
            }
        }
    }
    impl ::std::str::FromStr for ApiKeyAllowedSurfacesItem {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "rest_api" => Ok(Self::RestApi),
                "mcp" => Ok(Self::Mcp),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ApiKeyAllowedSurfacesItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ApiKeyAllowedSurfacesItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ApiKeyAllowedSurfacesItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Whether this key is owned by a user or represents a service account
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Whether this key is owned by a user or represents a service account",
    ///  "type": "string",
    ///  "enum": [
    ///    "user_owned",
    ///    "service_account"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ApiKeyAttributionClass {
        #[serde(rename = "user_owned")]
        UserOwned,
        #[serde(rename = "service_account")]
        ServiceAccount,
    }
    impl ::std::fmt::Display for ApiKeyAttributionClass {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::UserOwned => f.write_str("user_owned"),
                Self::ServiceAccount => f.write_str("service_account"),
            }
        }
    }
    impl ::std::str::FromStr for ApiKeyAttributionClass {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "user_owned" => Ok(Self::UserOwned),
                "service_account" => Ok(Self::ServiceAccount),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ApiKeyAttributionClass {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ApiKeyAttributionClass {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ApiKeyAttributionClass {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///API key response
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "API key response",
    ///  "type": "object",
    ///  "required": [
    ///    "apiKey"
    ///  ],
    ///  "properties": {
    ///    "apiKey": {
    ///      "$ref": "#/components/schemas/APIKey"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ApiKeyResponse {
        #[serde(rename = "apiKey")]
        pub api_key: ApiKey,
    }
    ///List of API keys
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "List of API keys",
    ///  "type": "object",
    ///  "required": [
    ///    "apiKeys"
    ///  ],
    ///  "properties": {
    ///    "apiKeys": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/APIKey"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ApiKeysListResponse {
        #[serde(rename = "apiKeys")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub api_keys: ::std::vec::Vec<ApiKey>,
    }
    ///Configuration for a human approval gate on a step
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Configuration for a human approval gate on a step",
    ///  "type": "object",
    ///  "properties": {
    ///    "input": {
    ///      "description": "List of expected input field names from the approver",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "prompt": {
    ///      "description": "Message displayed to the approver",
    ///      "type": "string"
    ///    },
    ///    "required": {
    ///      "description": "Subset of input fields that must be provided",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "rewindTo": {
    ///      "description": "Optional step name to restart from when the approver pushes the step back. Must reference the step itself or an upstream dependency.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ApprovalConfig {
        ///List of expected input field names from the approver
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub input: ::std::vec::Vec<::std::string::String>,
        ///Message displayed to the approver
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub prompt: ::std::option::Option<::std::string::String>,
        ///Subset of input fields that must be provided
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub required: ::std::vec::Vec<::std::string::String>,
        ///Optional step name to restart from when the approver pushes the step back. Must reference the step itself or an upstream dependency.
        #[serde(
            rename = "rewindTo",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub rewind_to: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for ApprovalConfig {
        fn default() -> Self {
            Self {
                input: Default::default(),
                prompt: Default::default(),
                required: Default::default(),
                rewind_to: Default::default(),
            }
        }
    }
    ///Request body for approving a waiting step
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for approving a waiting step",
    ///  "type": "object",
    ///  "properties": {
    ///    "inputs": {
    ///      "description": "Key-value parameters to provide. These will be available as environment variables in subsequent steps.",
    ///      "type": "object",
    ///      "additionalProperties": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ApproveStepRequest {
        ///Key-value parameters to provide. These will be available as environment variables in subsequent steps.
        #[serde(
            default,
            skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
        )]
        pub inputs: ::std::collections::HashMap<
            ::std::string::String,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for ApproveStepRequest {
        fn default() -> Self {
            Self { inputs: Default::default() }
        }
    }
    ///Response after approving a waiting step
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response after approving a waiting step",
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunId",
    ///    "resumed",
    ///    "stepName"
    ///  ],
    ///  "properties": {
    ///    "dagRunId": {
    ///      "description": "The DAG run ID",
    ///      "type": "string"
    ///    },
    ///    "resumed": {
    ///      "description": "Whether the DAG run was re-enqueued for execution",
    ///      "type": "boolean"
    ///    },
    ///    "stepName": {
    ///      "description": "The approved step name",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ApproveStepResponse {
        ///The DAG run ID
        #[serde(rename = "dagRunId")]
        pub dag_run_id: ::std::string::String,
        ///Whether the DAG run was re-enqueued for execution
        pub resumed: bool,
        ///The approved step name
        #[serde(rename = "stepName")]
        pub step_name: ::std::string::String,
    }
    ///Artifact tree node type
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Artifact tree node type",
    ///  "type": "string",
    ///  "enum": [
    ///    "directory",
    ///    "file"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ArtifactNodeType {
        #[serde(rename = "directory")]
        Directory,
        #[serde(rename = "file")]
        File,
    }
    impl ::std::fmt::Display for ArtifactNodeType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Directory => f.write_str("directory"),
                Self::File => f.write_str("file"),
            }
        }
    }
    impl ::std::str::FromStr for ArtifactNodeType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "directory" => Ok(Self::Directory),
                "file" => Ok(Self::File),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ArtifactNodeType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ArtifactNodeType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ArtifactNodeType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Preview mode for an artifact file
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Preview mode for an artifact file",
    ///  "type": "string",
    ///  "enum": [
    ///    "markdown",
    ///    "html",
    ///    "text",
    ///    "image",
    ///    "binary"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ArtifactPreviewKind {
        #[serde(rename = "markdown")]
        Markdown,
        #[serde(rename = "html")]
        Html,
        #[serde(rename = "text")]
        Text,
        #[serde(rename = "image")]
        Image,
        #[serde(rename = "binary")]
        Binary,
    }
    impl ::std::fmt::Display for ArtifactPreviewKind {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Markdown => f.write_str("markdown"),
                Self::Html => f.write_str("html"),
                Self::Text => f.write_str("text"),
                Self::Image => f.write_str("image"),
                Self::Binary => f.write_str("binary"),
            }
        }
    }
    impl ::std::str::FromStr for ArtifactPreviewKind {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "markdown" => Ok(Self::Markdown),
                "html" => Ok(Self::Html),
                "text" => Ok(Self::Text),
                "image" => Ok(Self::Image),
                "binary" => Ok(Self::Binary),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ArtifactPreviewKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ArtifactPreviewKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ArtifactPreviewKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Preview metadata and optional inline content for a DAG-run artifact file
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Preview metadata and optional inline content for a DAG-run artifact file",
    ///  "type": "object",
    ///  "required": [
    ///    "kind",
    ///    "mimeType",
    ///    "name",
    ///    "path",
    ///    "size",
    ///    "tooLarge",
    ///    "truncated"
    ///  ],
    ///  "properties": {
    ///    "content": {
    ///      "description": "Inline preview content for markdown, HTML, or text artifacts",
    ///      "type": "string"
    ///    },
    ///    "kind": {
    ///      "$ref": "#/components/schemas/ArtifactPreviewKind"
    ///    },
    ///    "mimeType": {
    ///      "description": "Detected MIME type for the artifact file",
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "description": "Artifact file name",
    ///      "type": "string"
    ///    },
    ///    "path": {
    ///      "description": "Relative artifact file path",
    ///      "type": "string"
    ///    },
    ///    "size": {
    ///      "description": "Artifact file size in bytes",
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "tooLarge": {
    ///      "description": "Whether the artifact exceeds the inline preview size limit",
    ///      "type": "boolean"
    ///    },
    ///    "truncated": {
    ///      "description": "Whether inline text content was truncated for preview",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ArtifactPreviewResponse {
        ///Inline preview content for markdown, HTML, or text artifacts
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub content: ::std::option::Option<::std::string::String>,
        pub kind: ArtifactPreviewKind,
        ///Detected MIME type for the artifact file
        #[serde(rename = "mimeType")]
        pub mime_type: ::std::string::String,
        ///Artifact file name
        pub name: ::std::string::String,
        ///Relative artifact file path
        pub path: ::std::string::String,
        ///Artifact file size in bytes
        pub size: i64,
        ///Whether the artifact exceeds the inline preview size limit
        #[serde(rename = "tooLarge")]
        pub too_large: bool,
        ///Whether inline text content was truncated for preview
        pub truncated: bool,
    }
    ///A single file or directory in a DAG-run artifact tree
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A single file or directory in a DAG-run artifact tree",
    ///  "type": "object",
    ///  "required": [
    ///    "name",
    ///    "path",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "children": {
    ///      "description": "Nested artifact entries when this node is a directory",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ArtifactTreeNode"
    ///      }
    ///    },
    ///    "name": {
    ///      "description": "Display name of the artifact entry",
    ///      "type": "string"
    ///    },
    ///    "path": {
    ///      "description": "Relative path of the artifact entry within the artifact directory",
    ///      "type": "string"
    ///    },
    ///    "size": {
    ///      "description": "Size of the artifact file in bytes",
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "type": {
    ///      "$ref": "#/components/schemas/ArtifactNodeType"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ArtifactTreeNode {
        ///Nested artifact entries when this node is a directory
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub children: ::std::vec::Vec<ArtifactTreeNode>,
        ///Display name of the artifact entry
        pub name: ::std::string::String,
        ///Relative path of the artifact entry within the artifact directory
        pub path: ::std::string::String,
        ///Size of the artifact file in bytes
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub size: ::std::option::Option<i64>,
        #[serde(rename = "type")]
        pub type_: ArtifactNodeType,
    }
    ///Artifact tree for a DAG-run
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Artifact tree for a DAG-run",
    ///  "type": "object",
    ///  "required": [
    ///    "items"
    ///  ],
    ///  "properties": {
    ///    "items": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ArtifactTreeNode"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ArtifactTreeResponse {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub items: ::std::vec::Vec<ArtifactTreeNode>,
    }
    ///A single audit log entry
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A single audit log entry",
    ///  "type": "object",
    ///  "required": [
    ///    "action",
    ///    "category",
    ///    "id",
    ///    "timestamp",
    ///    "userId",
    ///    "username"
    ///  ],
    ///  "properties": {
    ///    "action": {
    ///      "description": "The action that was performed (e.g., session_start, command, login)",
    ///      "type": "string"
    ///    },
    ///    "category": {
    ///      "description": "Category of the audit event (e.g., terminal, user, dag)",
    ///      "type": "string"
    ///    },
    ///    "correlationId": {
    ///      "description": "Correlation ID linking related audit events",
    ///      "type": "string"
    ///    },
    ///    "credentialId": {
    ///      "description": "Credential ID used for the request",
    ///      "type": "string"
    ///    },
    ///    "credentialType": {
    ///      "description": "Credential type used for the request",
    ///      "type": "string"
    ///    },
    ///    "details": {
    ///      "description": "JSON-encoded action-specific details",
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "description": "Unique identifier for this entry",
    ///      "type": "string"
    ///    },
    ///    "ipAddress": {
    ///      "description": "Client IP address if available",
    ///      "type": "string"
    ///    },
    ///    "mcpTool": {
    ///      "description": "MCP tool name when source is MCP",
    ///      "type": "string"
    ///    },
    ///    "resourceId": {
    ///      "description": "Affected resource ID",
    ///      "type": "string"
    ///    },
    ///    "resourceType": {
    ///      "description": "Affected resource type",
    ///      "type": "string"
    ///    },
    ///    "result": {
    ///      "description": "Event result such as succeeded, failed, or denied",
    ///      "type": "string"
    ///    },
    ///    "source": {
    ///      "description": "Source surface that produced the event",
    ///      "type": "string"
    ///    },
    ///    "surface": {
    ///      "description": "Externally accepted credential surface",
    ///      "type": "string"
    ///    },
    ///    "timestamp": {
    ///      "description": "When the event occurred",
    ///      "type": "string"
    ///    },
    ///    "userId": {
    ///      "description": "ID of the user who performed the action",
    ///      "type": "string"
    ///    },
    ///    "username": {
    ///      "description": "Username of the user who performed the action",
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "description": "Canonical workspace for filtering",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AuditEntry {
        ///The action that was performed (e.g., session_start, command, login)
        pub action: ::std::string::String,
        ///Category of the audit event (e.g., terminal, user, dag)
        pub category: ::std::string::String,
        ///Correlation ID linking related audit events
        #[serde(
            rename = "correlationId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub correlation_id: ::std::option::Option<::std::string::String>,
        ///Credential ID used for the request
        #[serde(
            rename = "credentialId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub credential_id: ::std::option::Option<::std::string::String>,
        ///Credential type used for the request
        #[serde(
            rename = "credentialType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub credential_type: ::std::option::Option<::std::string::String>,
        ///JSON-encoded action-specific details
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub details: ::std::option::Option<::std::string::String>,
        ///Unique identifier for this entry
        pub id: ::std::string::String,
        ///Client IP address if available
        #[serde(
            rename = "ipAddress",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub ip_address: ::std::option::Option<::std::string::String>,
        ///MCP tool name when source is MCP
        #[serde(
            rename = "mcpTool",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub mcp_tool: ::std::option::Option<::std::string::String>,
        ///Affected resource ID
        #[serde(
            rename = "resourceId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub resource_id: ::std::option::Option<::std::string::String>,
        ///Affected resource type
        #[serde(
            rename = "resourceType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub resource_type: ::std::option::Option<::std::string::String>,
        ///Event result such as succeeded, failed, or denied
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub result: ::std::option::Option<::std::string::String>,
        ///Source surface that produced the event
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub source: ::std::option::Option<::std::string::String>,
        ///Externally accepted credential surface
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub surface: ::std::option::Option<::std::string::String>,
        ///When the event occurred
        pub timestamp: ::std::string::String,
        ///ID of the user who performed the action
        #[serde(rename = "userId")]
        pub user_id: ::std::string::String,
        ///Username of the user who performed the action
        pub username: ::std::string::String,
        ///Canonical workspace for filtering
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///Response containing audit log entries
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response containing audit log entries",
    ///  "type": "object",
    ///  "required": [
    ///    "entries",
    ///    "total"
    ///  ],
    ///  "properties": {
    ///    "entries": {
    ///      "description": "List of audit log entries",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AuditEntry"
    ///      }
    ///    },
    ///    "total": {
    ///      "description": "Total number of entries matching the filter (before pagination)",
    ///      "type": "integer"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct AuditLogsResponse {
        ///List of audit log entries
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub entries: ::std::vec::Vec<AuditEntry>,
        ///Total number of entries matching the filter (before pagination)
        pub total: i64,
    }
    ///Build execution decision and its explanation
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Build execution decision and its explanation",
    ///  "type": "object",
    ///  "required": [
    ///    "decision",
    ///    "phase",
    ///    "reason"
    ///  ],
    ///  "properties": {
    ///    "decision": {
    ///      "type": "string",
    ///      "enum": [
    ///        "none",
    ///        "always",
    ///        "execute",
    ///        "reuse",
    ///        "deferred"
    ///      ]
    ///    },
    ///    "detail": {
    ///      "description": "Human-readable explanation of the decision",
    ///      "type": "string"
    ///    },
    ///    "fingerprint": {
    ///      "type": "string"
    ///    },
    ///    "materializationKey": {
    ///      "type": "string"
    ///    },
    ///    "phase": {
    ///      "type": "string",
    ///      "enum": [
    ///        "precondition",
    ///        "evaluate",
    ///        "execute",
    ///        "verify",
    ///        "commit",
    ///        "complete"
    ///      ]
    ///    },
    ///    "producerAttemptId": {
    ///      "type": "string"
    ///    },
    ///    "producerRun": {
    ///      "$ref": "#/components/schemas/BuildProducer"
    ///    },
    ///    "reason": {
    ///      "description": "Stable machine-readable reason for the decision",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct BuildExecution {
        pub decision: BuildExecutionDecision,
        ///Human-readable explanation of the decision
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub detail: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub fingerprint: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "materializationKey",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub materialization_key: ::std::option::Option<::std::string::String>,
        pub phase: BuildExecutionPhase,
        #[serde(
            rename = "producerAttemptId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub producer_attempt_id: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "producerRun",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub producer_run: ::std::option::Option<BuildProducer>,
        ///Stable machine-readable reason for the decision
        pub reason: ::std::string::String,
    }
    ///`BuildExecutionDecision`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "none",
    ///    "always",
    ///    "execute",
    ///    "reuse",
    ///    "deferred"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum BuildExecutionDecision {
        #[serde(rename = "none")]
        None,
        #[serde(rename = "always")]
        Always,
        #[serde(rename = "execute")]
        Execute,
        #[serde(rename = "reuse")]
        Reuse,
        #[serde(rename = "deferred")]
        Deferred,
    }
    impl ::std::fmt::Display for BuildExecutionDecision {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::None => f.write_str("none"),
                Self::Always => f.write_str("always"),
                Self::Execute => f.write_str("execute"),
                Self::Reuse => f.write_str("reuse"),
                Self::Deferred => f.write_str("deferred"),
            }
        }
    }
    impl ::std::str::FromStr for BuildExecutionDecision {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "none" => Ok(Self::None),
                "always" => Ok(Self::Always),
                "execute" => Ok(Self::Execute),
                "reuse" => Ok(Self::Reuse),
                "deferred" => Ok(Self::Deferred),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for BuildExecutionDecision {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for BuildExecutionDecision {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for BuildExecutionDecision {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`BuildExecutionPhase`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "precondition",
    ///    "evaluate",
    ///    "execute",
    ///    "verify",
    ///    "commit",
    ///    "complete"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum BuildExecutionPhase {
        #[serde(rename = "precondition")]
        Precondition,
        #[serde(rename = "evaluate")]
        Evaluate,
        #[serde(rename = "execute")]
        Execute,
        #[serde(rename = "verify")]
        Verify,
        #[serde(rename = "commit")]
        Commit,
        #[serde(rename = "complete")]
        Complete,
    }
    impl ::std::fmt::Display for BuildExecutionPhase {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Precondition => f.write_str("precondition"),
                Self::Evaluate => f.write_str("evaluate"),
                Self::Execute => f.write_str("execute"),
                Self::Verify => f.write_str("verify"),
                Self::Commit => f.write_str("commit"),
                Self::Complete => f.write_str("complete"),
            }
        }
    }
    impl ::std::str::FromStr for BuildExecutionPhase {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "precondition" => Ok(Self::Precondition),
                "evaluate" => Ok(Self::Evaluate),
                "execute" => Ok(Self::Execute),
                "verify" => Ok(Self::Verify),
                "commit" => Ok(Self::Commit),
                "complete" => Ok(Self::Complete),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for BuildExecutionPhase {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for BuildExecutionPhase {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for BuildExecutionPhase {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///DAG-run that produced a reused materialization
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "DAG-run that produced a reused materialization",
    ///  "type": "object",
    ///  "properties": {
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct BuildProducer {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub id: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub name: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for BuildProducer {
        fn default() -> Self {
            Self {
                id: Default::default(),
                name: Default::default(),
            }
        }
    }
    ///Request body for changing password
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for changing password",
    ///  "type": "object",
    ///  "required": [
    ///    "currentPassword",
    ///    "newPassword"
    ///  ],
    ///  "properties": {
    ///    "currentPassword": {
    ///      "description": "Current password for verification",
    ///      "type": "string",
    ///      "minLength": 1
    ///    },
    ///    "newPassword": {
    ///      "description": "New password to set",
    ///      "type": "string",
    ///      "minLength": 8
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ChangePasswordRequest {
        ///Current password for verification
        #[serde(rename = "currentPassword")]
        pub current_password: ChangePasswordRequestCurrentPassword,
        ///New password to set
        #[serde(rename = "newPassword")]
        pub new_password: ChangePasswordRequestNewPassword,
    }
    ///Current password for verification
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Current password for verification",
    ///  "type": "string",
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct ChangePasswordRequestCurrentPassword(::std::string::String);
    impl ::std::ops::Deref for ChangePasswordRequestCurrentPassword {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<ChangePasswordRequestCurrentPassword>
    for ::std::string::String {
        fn from(value: ChangePasswordRequestCurrentPassword) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for ChangePasswordRequestCurrentPassword {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for ChangePasswordRequestCurrentPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for ChangePasswordRequestCurrentPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for ChangePasswordRequestCurrentPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for ChangePasswordRequestCurrentPassword {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///New password to set
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "New password to set",
    ///  "type": "string",
    ///  "minLength": 8
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct ChangePasswordRequestNewPassword(::std::string::String);
    impl ::std::ops::Deref for ChangePasswordRequestNewPassword {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<ChangePasswordRequestNewPassword>
    for ::std::string::String {
        fn from(value: ChangePasswordRequestNewPassword) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for ChangePasswordRequestNewPassword {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() < 8usize {
                return Err("shorter than 8 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for ChangePasswordRequestNewPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for ChangePasswordRequestNewPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for ChangePasswordRequestNewPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for ChangePasswordRequestNewPassword {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///A single chat message in an LLM session
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A single chat message in an LLM session",
    ///  "type": "object",
    ///  "required": [
    ///    "content",
    ///    "role"
    ///  ],
    ///  "properties": {
    ///    "content": {
    ///      "description": "Message content",
    ///      "type": "string"
    ///    },
    ///    "metadata": {
    ///      "$ref": "#/components/schemas/ChatMessageMetadata"
    ///    },
    ///    "role": {
    ///      "description": "Message role in the session",
    ///      "type": "string",
    ///      "enum": [
    ///        "system",
    ///        "user",
    ///        "assistant",
    ///        "tool"
    ///      ]
    ///    },
    ///    "toolCalls": {
    ///      "description": "Tool calls made by the assistant (only for assistant messages)",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ChatToolCall"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ChatMessage {
        ///Message content
        pub content: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub metadata: ::std::option::Option<ChatMessageMetadata>,
        ///Message role in the session
        pub role: ChatMessageRole,
        ///Tool calls made by the assistant (only for assistant messages)
        #[serde(
            rename = "toolCalls",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub tool_calls: ::std::vec::Vec<ChatToolCall>,
    }
    ///Metadata about an LLM API call
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Metadata about an LLM API call",
    ///  "type": "object",
    ///  "properties": {
    ///    "completionTokens": {
    ///      "description": "Number of tokens in the completion",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "model": {
    ///      "description": "Model identifier used",
    ///      "type": "string"
    ///    },
    ///    "promptTokens": {
    ///      "description": "Number of tokens in the prompt",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "provider": {
    ///      "description": "LLM provider (openai, anthropic, gemini, etc.)",
    ///      "type": "string"
    ///    },
    ///    "totalTokens": {
    ///      "description": "Total tokens (prompt + completion)",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ChatMessageMetadata {
        ///Number of tokens in the completion
        #[serde(
            rename = "completionTokens",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub completion_tokens: ::std::option::Option<u64>,
        ///Model identifier used
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub model: ::std::option::Option<::std::string::String>,
        ///Number of tokens in the prompt
        #[serde(
            rename = "promptTokens",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub prompt_tokens: ::std::option::Option<u64>,
        ///LLM provider (openai, anthropic, gemini, etc.)
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub provider: ::std::option::Option<::std::string::String>,
        ///Total tokens (prompt + completion)
        #[serde(
            rename = "totalTokens",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub total_tokens: ::std::option::Option<u64>,
    }
    impl ::std::default::Default for ChatMessageMetadata {
        fn default() -> Self {
            Self {
                completion_tokens: Default::default(),
                model: Default::default(),
                prompt_tokens: Default::default(),
                provider: Default::default(),
                total_tokens: Default::default(),
            }
        }
    }
    ///Message role in the session
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Message role in the session",
    ///  "type": "string",
    ///  "enum": [
    ///    "system",
    ///    "user",
    ///    "assistant",
    ///    "tool"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ChatMessageRole {
        #[serde(rename = "system")]
        System,
        #[serde(rename = "user")]
        User,
        #[serde(rename = "assistant")]
        Assistant,
        #[serde(rename = "tool")]
        Tool,
    }
    impl ::std::fmt::Display for ChatMessageRole {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::System => f.write_str("system"),
                Self::User => f.write_str("user"),
                Self::Assistant => f.write_str("assistant"),
                Self::Tool => f.write_str("tool"),
            }
        }
    }
    impl ::std::str::FromStr for ChatMessageRole {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "system" => Ok(Self::System),
                "user" => Ok(Self::User),
                "assistant" => Ok(Self::Assistant),
                "tool" => Ok(Self::Tool),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ChatMessageRole {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ChatMessageRole {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ChatMessageRole {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Response containing chat messages for a step
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response containing chat messages for a step",
    ///  "type": "object",
    ///  "required": [
    ///    "hasMore",
    ///    "messages",
    ///    "stepStatus",
    ///    "stepStatusLabel"
    ///  ],
    ///  "properties": {
    ///    "hasMore": {
    ///      "description": "True if step is still running and more messages may arrive",
    ///      "type": "boolean"
    ///    },
    ///    "messages": {
    ///      "description": "List of chat messages",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ChatMessage"
    ///      }
    ///    },
    ///    "stepStatus": {
    ///      "$ref": "#/components/schemas/NodeStatus"
    ///    },
    ///    "stepStatusLabel": {
    ///      "$ref": "#/components/schemas/NodeStatusLabel"
    ///    },
    ///    "toolDefinitions": {
    ///      "description": "Tool definitions that were available to the LLM",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ToolDefinition"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ChatMessagesResponse {
        ///True if step is still running and more messages may arrive
        #[serde(rename = "hasMore")]
        pub has_more: bool,
        ///List of chat messages
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub messages: ::std::vec::Vec<ChatMessage>,
        #[serde(rename = "stepStatus")]
        pub step_status: NodeStatus,
        #[serde(rename = "stepStatusLabel")]
        pub step_status_label: NodeStatusLabel,
        ///Tool definitions that were available to the LLM
        #[serde(
            rename = "toolDefinitions",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub tool_definitions: ::std::vec::Vec<ToolDefinition>,
    }
    ///A tool call requested by the LLM
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A tool call requested by the LLM",
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "arguments": {
    ///      "description": "JSON string of tool arguments",
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "description": "Unique identifier for this tool call",
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "description": "Name of the tool being called",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ChatToolCall {
        ///JSON string of tool arguments
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub arguments: ::std::option::Option<::std::string::String>,
        ///Unique identifier for this tool call
        pub id: ::std::string::String,
        ///Name of the tool being called
        pub name: ::std::string::String,
    }
    ///A command with its arguments
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A command with its arguments",
    ///  "type": "object",
    ///  "required": [
    ///    "command"
    ///  ],
    ///  "properties": {
    ///    "args": {
    ///      "description": "Arguments for the command",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "command": {
    ///      "description": "The command to execute",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CommandEntry {
        ///Arguments for the command
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub args: ::std::vec::Vec<::std::string::String>,
        ///The command to execute
        pub command: ::std::string::String,
    }
    ///`CompleteHumanTaskStepId`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "maxLength": 40
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct CompleteHumanTaskStepId(::std::string::String);
    impl ::std::ops::Deref for CompleteHumanTaskStepId {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<CompleteHumanTaskStepId> for ::std::string::String {
        fn from(value: CompleteHumanTaskStepId) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for CompleteHumanTaskStepId {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 40usize {
                return Err("longer than 40 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for CompleteHumanTaskStepId {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for CompleteHumanTaskStepId {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for CompleteHumanTaskStepId {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for CompleteHumanTaskStepId {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Precondition that must be satisfied before running a step or DAG-run
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Precondition that must be satisfied before running a step or DAG-run",
    ///  "type": "object",
    ///  "properties": {
    ///    "condition": {
    ///      "description": "Value or command text to evaluate. When `expected` is omitted, this runs as a command check. When `expected` is set, this is value-resolved and compared as data.",
    ///      "type": "string"
    ///    },
    ///    "error": {
    ///      "description": "Error message if the condition is not met",
    ///      "type": "string"
    ///    },
    ///    "eval": {
    ///      "description": "Dynamic value expression to evaluate and compare with `expected`. Valid only when `expected` is set and `condition` is omitted.",
    ///      "type": "string"
    ///    },
    ///    "expected": {
    ///      "description": "Expected result for a value-match precondition. When set, Dagu compares the actual value from `condition` or `eval` instead of using command exit status.",
    ///      "type": "string"
    ///    },
    ///    "matched": {
    ///      "description": "Whether the condition was met",
    ///      "type": "boolean"
    ///    },
    ///    "negate": {
    ///      "description": "If true, inverts the condition result (run when condition does NOT match)",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct Condition {
        ///Value or command text to evaluate. When `expected` is omitted, this runs as a command check. When `expected` is set, this is value-resolved and compared as data.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub condition: ::std::option::Option<::std::string::String>,
        ///Error message if the condition is not met
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub error: ::std::option::Option<::std::string::String>,
        ///Dynamic value expression to evaluate and compare with `expected`. Valid only when `expected` is set and `condition` is omitted.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub eval: ::std::option::Option<::std::string::String>,
        ///Expected result for a value-match precondition. When set, Dagu compares the actual value from `condition` or `eval` instead of using command exit status.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub expected: ::std::option::Option<::std::string::String>,
        ///Whether the condition was met
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub matched: ::std::option::Option<bool>,
        ///If true, inverts the condition result (run when condition does NOT match)
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub negate: ::std::option::Option<bool>,
    }
    impl ::std::default::Default for Condition {
        fn default() -> Self {
            Self {
                condition: Default::default(),
                error: Default::default(),
                eval: Default::default(),
                expected: Default::default(),
                matched: Default::default(),
                negate: Default::default(),
            }
        }
    }
    ///Coordinator instance status information
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Coordinator instance status information",
    ///  "type": "object",
    ///  "required": [
    ///    "host",
    ///    "instanceId",
    ///    "port",
    ///    "startedAt",
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "host": {
    ///      "description": "Hostname where coordinator is running",
    ///      "type": "string"
    ///    },
    ///    "instanceId": {
    ///      "description": "Unique identifier of the coordinator instance",
    ///      "type": "string"
    ///    },
    ///    "port": {
    ///      "description": "Port number the coordinator is listening on",
    ///      "type": "integer"
    ///    },
    ///    "startedAt": {
    ///      "description": "RFC3339 timestamp when coordinator started",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "description": "Coordinator status",
    ///      "type": "string",
    ///      "enum": [
    ///        "active",
    ///        "inactive",
    ///        "unknown"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CoordinatorInstance {
        ///Hostname where coordinator is running
        pub host: ::std::string::String,
        ///Unique identifier of the coordinator instance
        #[serde(rename = "instanceId")]
        pub instance_id: ::std::string::String,
        ///Port number the coordinator is listening on
        pub port: i64,
        ///RFC3339 timestamp when coordinator started
        #[serde(rename = "startedAt")]
        pub started_at: ::std::string::String,
        ///Coordinator status
        pub status: CoordinatorInstanceStatus,
    }
    ///Coordinator status
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Coordinator status",
    ///  "type": "string",
    ///  "enum": [
    ///    "active",
    ///    "inactive",
    ///    "unknown"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum CoordinatorInstanceStatus {
        #[serde(rename = "active")]
        Active,
        #[serde(rename = "inactive")]
        Inactive,
        #[serde(rename = "unknown")]
        Unknown,
    }
    impl ::std::fmt::Display for CoordinatorInstanceStatus {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Active => f.write_str("active"),
                Self::Inactive => f.write_str("inactive"),
                Self::Unknown => f.write_str("unknown"),
            }
        }
    }
    impl ::std::str::FromStr for CoordinatorInstanceStatus {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "active" => Ok(Self::Active),
                "inactive" => Ok(Self::Inactive),
                "unknown" => Ok(Self::Unknown),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for CoordinatorInstanceStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for CoordinatorInstanceStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for CoordinatorInstanceStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Response containing status of all coordinator instances
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response containing status of all coordinator instances",
    ///  "type": "object",
    ///  "required": [
    ///    "coordinators"
    ///  ],
    ///  "properties": {
    ///    "coordinators": {
    ///      "description": "List of all registered coordinator instances",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/CoordinatorInstance"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CoordinatorStatusResponse {
        ///List of all registered coordinator instances
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub coordinators: ::std::vec::Vec<CoordinatorInstance>,
    }
    ///Create API key request
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Create API key request",
    ///  "type": "object",
    ///  "required": [
    ///    "allowedSurfaces",
    ///    "attributionClass",
    ///    "name",
    ///    "role"
    ///  ],
    ///  "properties": {
    ///    "allowedSurfaces": {
    ///      "description": "Interfaces where this API key may be accepted",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string",
    ///        "enum": [
    ///          "rest_api",
    ///          "mcp"
    ///        ]
    ///      },
    ///      "minItems": 1,
    ///      "uniqueItems": true
    ///    },
    ///    "attributionClass": {
    ///      "description": "Whether this key is owned by a user or represents a service account",
    ///      "type": "string",
    ///      "enum": [
    ///        "user_owned",
    ///        "service_account"
    ///      ]
    ///    },
    ///    "description": {
    ///      "description": "Purpose description",
    ///      "type": "string",
    ///      "maxLength": 500
    ///    },
    ///    "name": {
    ///      "description": "Human-readable name",
    ///      "type": "string",
    ///      "maxLength": 100,
    ///      "minLength": 1
    ///    },
    ///    "ownerUserId": {
    ///      "description": "Owner user ID when attributionClass is user_owned",
    ///      "type": "string"
    ///    },
    ///    "role": {
    ///      "$ref": "#/components/schemas/UserRole"
    ///    },
    ///    "serviceAccountName": {
    ///      "description": "Service-account display name when attributionClass is service_account",
    ///      "type": "string"
    ///    },
    ///    "workspaceAccess": {
    ///      "$ref": "#/components/schemas/WorkspaceAccess"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CreateApiKeyRequest {
        ///Interfaces where this API key may be accepted
        #[serde(rename = "allowedSurfaces")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub allowed_surfaces: Vec<CreateApiKeyRequestAllowedSurfacesItem>,
        ///Whether this key is owned by a user or represents a service account
        #[serde(rename = "attributionClass")]
        pub attribution_class: CreateApiKeyRequestAttributionClass,
        ///Purpose description
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<CreateApiKeyRequestDescription>,
        ///Human-readable name
        pub name: CreateApiKeyRequestName,
        ///Owner user ID when attributionClass is user_owned
        #[serde(
            rename = "ownerUserId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub owner_user_id: ::std::option::Option<::std::string::String>,
        pub role: UserRole,
        ///Service-account display name when attributionClass is service_account
        #[serde(
            rename = "serviceAccountName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub service_account_name: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "workspaceAccess",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub workspace_access: ::std::option::Option<WorkspaceAccess>,
    }
    ///`CreateApiKeyRequestAllowedSurfacesItem`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "rest_api",
    ///    "mcp"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum CreateApiKeyRequestAllowedSurfacesItem {
        #[serde(rename = "rest_api")]
        RestApi,
        #[serde(rename = "mcp")]
        Mcp,
    }
    impl ::std::fmt::Display for CreateApiKeyRequestAllowedSurfacesItem {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::RestApi => f.write_str("rest_api"),
                Self::Mcp => f.write_str("mcp"),
            }
        }
    }
    impl ::std::str::FromStr for CreateApiKeyRequestAllowedSurfacesItem {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "rest_api" => Ok(Self::RestApi),
                "mcp" => Ok(Self::Mcp),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for CreateApiKeyRequestAllowedSurfacesItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for CreateApiKeyRequestAllowedSurfacesItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for CreateApiKeyRequestAllowedSurfacesItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Whether this key is owned by a user or represents a service account
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Whether this key is owned by a user or represents a service account",
    ///  "type": "string",
    ///  "enum": [
    ///    "user_owned",
    ///    "service_account"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum CreateApiKeyRequestAttributionClass {
        #[serde(rename = "user_owned")]
        UserOwned,
        #[serde(rename = "service_account")]
        ServiceAccount,
    }
    impl ::std::fmt::Display for CreateApiKeyRequestAttributionClass {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::UserOwned => f.write_str("user_owned"),
                Self::ServiceAccount => f.write_str("service_account"),
            }
        }
    }
    impl ::std::str::FromStr for CreateApiKeyRequestAttributionClass {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "user_owned" => Ok(Self::UserOwned),
                "service_account" => Ok(Self::ServiceAccount),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for CreateApiKeyRequestAttributionClass {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for CreateApiKeyRequestAttributionClass {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for CreateApiKeyRequestAttributionClass {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Purpose description
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Purpose description",
    ///  "type": "string",
    ///  "maxLength": 500
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct CreateApiKeyRequestDescription(::std::string::String);
    impl ::std::ops::Deref for CreateApiKeyRequestDescription {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<CreateApiKeyRequestDescription> for ::std::string::String {
        fn from(value: CreateApiKeyRequestDescription) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for CreateApiKeyRequestDescription {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 500usize {
                return Err("longer than 500 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for CreateApiKeyRequestDescription {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for CreateApiKeyRequestDescription {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for CreateApiKeyRequestDescription {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for CreateApiKeyRequestDescription {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Human-readable name
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Human-readable name",
    ///  "type": "string",
    ///  "maxLength": 100,
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct CreateApiKeyRequestName(::std::string::String);
    impl ::std::ops::Deref for CreateApiKeyRequestName {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<CreateApiKeyRequestName> for ::std::string::String {
        fn from(value: CreateApiKeyRequestName) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for CreateApiKeyRequestName {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 100usize {
                return Err("longer than 100 characters".into());
            }
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for CreateApiKeyRequestName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for CreateApiKeyRequestName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for CreateApiKeyRequestName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for CreateApiKeyRequestName {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Create API key response
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Create API key response",
    ///  "type": "object",
    ///  "required": [
    ///    "apiKey",
    ///    "key"
    ///  ],
    ///  "properties": {
    ///    "apiKey": {
    ///      "$ref": "#/components/schemas/APIKey"
    ///    },
    ///    "key": {
    ///      "description": "Full key secret, only returned once",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CreateApiKeyResponse {
        #[serde(rename = "apiKey")]
        pub api_key: ApiKey,
        ///Full key secret, only returned once
        pub key: ::std::string::String,
    }
    ///`CreateNewDagBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "name": {
    ///      "$ref": "#/components/schemas/DAGName"
    ///    },
    ///    "spec": {
    ///      "description": "Optional DAG spec in YAML format to initialize the DAG. If provided, the spec will be validated before creation.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CreateNewDagBody {
        pub name: DagName,
        ///Optional DAG spec in YAML format to initialize the DAG. If provided, the spec will be validated before creation.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub spec: ::std::option::Option<::std::string::String>,
    }
    ///`CreateNewDagResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "name": {
    ///      "description": "Name of the newly created DAG",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CreateNewDagResponse {
        ///Name of the newly created DAG
        pub name: ::std::string::String,
    }
    ///`CreateRemoteNodeRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "apiBaseUrl",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "apiBaseUrl": {
    ///      "description": "Base URL of the remote Dagu instance API",
    ///      "type": "string"
    ///    },
    ///    "authToken": {
    ///      "description": "Bearer token for token auth",
    ///      "type": "string"
    ///    },
    ///    "authType": {
    ///      "description": "Authentication mode",
    ///      "default": "none",
    ///      "type": "string",
    ///      "enum": [
    ///        "none",
    ///        "basic",
    ///        "token"
    ///      ]
    ///    },
    ///    "basicAuthPassword": {
    ///      "description": "Password for basic auth",
    ///      "type": "string"
    ///    },
    ///    "basicAuthUsername": {
    ///      "description": "Username for basic auth",
    ///      "type": "string"
    ///    },
    ///    "description": {
    ///      "description": "Optional description",
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "description": "Display name for the remote node",
    ///      "type": "string"
    ///    },
    ///    "skipTlsVerify": {
    ///      "description": "Skip TLS certificate verification",
    ///      "default": false,
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CreateRemoteNodeRequest {
        ///Base URL of the remote Dagu instance API
        #[serde(rename = "apiBaseUrl")]
        pub api_base_url: ::std::string::String,
        ///Bearer token for token auth
        #[serde(
            rename = "authToken",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub auth_token: ::std::option::Option<::std::string::String>,
        ///Authentication mode
        #[serde(
            rename = "authType",
            default = "defaults::create_remote_node_request_auth_type"
        )]
        pub auth_type: CreateRemoteNodeRequestAuthType,
        ///Password for basic auth
        #[serde(
            rename = "basicAuthPassword",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub basic_auth_password: ::std::option::Option<::std::string::String>,
        ///Username for basic auth
        #[serde(
            rename = "basicAuthUsername",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub basic_auth_username: ::std::option::Option<::std::string::String>,
        ///Optional description
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        ///Display name for the remote node
        pub name: ::std::string::String,
        ///Skip TLS certificate verification
        #[serde(rename = "skipTlsVerify", default)]
        pub skip_tls_verify: bool,
    }
    ///Authentication mode
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Authentication mode",
    ///  "default": "none",
    ///  "type": "string",
    ///  "enum": [
    ///    "none",
    ///    "basic",
    ///    "token"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum CreateRemoteNodeRequestAuthType {
        #[serde(rename = "none")]
        None,
        #[serde(rename = "basic")]
        Basic,
        #[serde(rename = "token")]
        Token,
    }
    impl ::std::fmt::Display for CreateRemoteNodeRequestAuthType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::None => f.write_str("none"),
                Self::Basic => f.write_str("basic"),
                Self::Token => f.write_str("token"),
            }
        }
    }
    impl ::std::str::FromStr for CreateRemoteNodeRequestAuthType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "none" => Ok(Self::None),
                "basic" => Ok(Self::Basic),
                "token" => Ok(Self::Token),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for CreateRemoteNodeRequestAuthType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for CreateRemoteNodeRequestAuthType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for CreateRemoteNodeRequestAuthType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::default::Default for CreateRemoteNodeRequestAuthType {
        fn default() -> Self {
            CreateRemoteNodeRequestAuthType::None
        }
    }
    ///`CreateRuntimeProfileRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "$ref": "#/components/schemas/RuntimeProfileName"
    ///    },
    ///    "protected": {
    ///      "default": false,
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CreateRuntimeProfileRequest {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        pub name: RuntimeProfileName,
        #[serde(default)]
        pub protected: bool,
    }
    ///`CreateSecretRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "providerType",
    ///    "ref"
    ///  ],
    ///  "properties": {
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "providerType": {
    ///      "type": "string",
    ///      "enum": [
    ///        "dagu-managed"
    ///      ]
    ///    },
    ///    "ref": {
    ///      "description": "Secret ref used from DAG YAML, for example prod/db-password.",
    ///      "type": "string"
    ///    },
    ///    "value": {
    ///      "description": "Initial Dagu-managed value. Write-only; never returned by the API.",
    ///      "writeOnly": true,
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "description": "Secret scope for management. Use global for workspace-less secrets or a workspace name. Omit for global.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CreateSecretRequest {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        #[serde(rename = "providerType")]
        pub provider_type: CreateSecretRequestProviderType,
        ///Secret ref used from DAG YAML, for example prod/db-password.
        #[serde(rename = "ref")]
        pub ref_: ::std::string::String,
        ///Initial Dagu-managed value. Write-only; never returned by the API.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub value: ::std::option::Option<::std::string::String>,
        ///Secret scope for management. Use global for workspace-less secrets or a workspace name. Omit for global.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///`CreateSecretRequestProviderType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "dagu-managed"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum CreateSecretRequestProviderType {
        #[serde(rename = "dagu-managed")]
        DaguManaged,
    }
    impl ::std::fmt::Display for CreateSecretRequestProviderType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::DaguManaged => f.write_str("dagu-managed"),
            }
        }
    }
    impl ::std::str::FromStr for CreateSecretRequestProviderType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "dagu-managed" => Ok(Self::DaguManaged),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for CreateSecretRequestProviderType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for CreateSecretRequestProviderType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for CreateSecretRequestProviderType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Request body for creating a new user
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for creating a new user",
    ///  "type": "object",
    ///  "required": [
    ///    "password",
    ///    "role",
    ///    "username"
    ///  ],
    ///  "properties": {
    ///    "password": {
    ///      "description": "User's password",
    ///      "type": "string",
    ///      "minLength": 8
    ///    },
    ///    "role": {
    ///      "$ref": "#/components/schemas/UserRole"
    ///    },
    ///    "username": {
    ///      "description": "Unique username",
    ///      "type": "string",
    ///      "maxLength": 64,
    ///      "minLength": 1
    ///    },
    ///    "workspaceAccess": {
    ///      "$ref": "#/components/schemas/WorkspaceAccess"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CreateUserRequest {
        ///User's password
        pub password: CreateUserRequestPassword,
        pub role: UserRole,
        ///Unique username
        pub username: CreateUserRequestUsername,
        #[serde(
            rename = "workspaceAccess",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub workspace_access: ::std::option::Option<WorkspaceAccess>,
    }
    ///User's password
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "User's password",
    ///  "type": "string",
    ///  "minLength": 8
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct CreateUserRequestPassword(::std::string::String);
    impl ::std::ops::Deref for CreateUserRequestPassword {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<CreateUserRequestPassword> for ::std::string::String {
        fn from(value: CreateUserRequestPassword) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for CreateUserRequestPassword {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() < 8usize {
                return Err("shorter than 8 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for CreateUserRequestPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for CreateUserRequestPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for CreateUserRequestPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for CreateUserRequestPassword {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Unique username
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Unique username",
    ///  "type": "string",
    ///  "maxLength": 64,
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct CreateUserRequestUsername(::std::string::String);
    impl ::std::ops::Deref for CreateUserRequestUsername {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<CreateUserRequestUsername> for ::std::string::String {
        fn from(value: CreateUserRequestUsername) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for CreateUserRequestUsername {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 64usize {
                return Err("longer than 64 characters".into());
            }
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for CreateUserRequestUsername {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for CreateUserRequestUsername {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for CreateUserRequestUsername {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for CreateUserRequestUsername {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Request to create a new Wiki page
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to create a new Wiki page",
    ///  "type": "object",
    ///  "required": [
    ///    "content",
    ///    "id"
    ///  ],
    ///  "properties": {
    ///    "content": {
    ///      "description": "Full file content including optional YAML frontmatter",
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "$ref": "#/components/schemas/WikiPagePath"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CreateWikiPageRequest {
        ///Full file content including optional YAML frontmatter
        pub content: ::std::string::String,
        pub id: WikiPagePath,
    }
    ///`CreateWorkspaceRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "$ref": "#/components/schemas/WorkspaceName"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct CreateWorkspaceRequest {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        pub name: WorkspaceName,
    }
    ///Core DAG configuration containing definition and metadata
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Core DAG configuration containing definition and metadata",
    ///  "type": "object",
    ///  "required": [
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "defaultParams": {
    ///      "description": "Default parameter values in JSON format if not specified at DAG-run creation",
    ///      "type": "string"
    ///    },
    ///    "description": {
    ///      "description": "Human-readable description of the DAG's purpose and behavior",
    ///      "type": "string"
    ///    },
    ///    "group": {
    ///      "description": "Logical grouping of related DAGs for organizational purposes",
    ///      "type": "string"
    ///    },
    ///    "labels": {
    ///      "description": "List of labels for categorizing and filtering DAGs",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "maxActiveRuns": {
    ///      "description": "DEPRECATED: This field is ignored for local (DAG-based) queues. For concurrency control, use global queues",
    ///      "deprecated": true,
    ///      "type": "integer",
    ///      "x-deprecated-reason": "For concurrency control, configure global queues in config.yaml instead"
    ///    },
    ///    "name": {
    ///      "description": "Logical name of the DAG",
    ///      "type": "string"
    ///    },
    ///    "params": {
    ///      "description": "List of parameter names that can be passed to DAG-runs created from this DAG",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "queue": {
    ///      "description": "Name of the queue this DAG is assigned to. If not specified, the DAG name itself becomes the queue name",
    ///      "type": "string"
    ///    },
    ///    "resources": {
    ///      "$ref": "#/components/schemas/DAGResources"
    ///    },
    ///    "runConfig": {
    ///      "$ref": "#/components/schemas/RunConfig"
    ///    },
    ///    "schedule": {
    ///      "description": "List of scheduling expressions defining when DAG-runs should be created from this DAG",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Schedule"
    ///      }
    ///    },
    ///    "tags": {
    ///      "description": "Deprecated alias for labels. List of labels for categorizing and filtering DAGs",
    ///      "deprecated": true,
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "workspace": {
    ///      "description": "Workspace label value for the DAG. Omitted for default DAGs and invalid workspace labels.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct Dag {
        ///Default parameter values in JSON format if not specified at DAG-run creation
        #[serde(
            rename = "defaultParams",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub default_params: ::std::option::Option<::std::string::String>,
        ///Human-readable description of the DAG's purpose and behavior
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        ///Logical grouping of related DAGs for organizational purposes
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub group: ::std::option::Option<::std::string::String>,
        ///List of labels for categorizing and filtering DAGs
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub labels: ::std::vec::Vec<::std::string::String>,
        ///DEPRECATED: This field is ignored for local (DAG-based) queues. For concurrency control, use global queues
        #[serde(
            rename = "maxActiveRuns",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub max_active_runs: ::std::option::Option<i64>,
        ///Logical name of the DAG
        pub name: ::std::string::String,
        ///List of parameter names that can be passed to DAG-runs created from this DAG
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub params: ::std::vec::Vec<::std::string::String>,
        ///Name of the queue this DAG is assigned to. If not specified, the DAG name itself becomes the queue name
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub queue: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub resources: ::std::option::Option<DagResources>,
        #[serde(
            rename = "runConfig",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub run_config: ::std::option::Option<RunConfig>,
        ///List of scheduling expressions defining when DAG-runs should be created from this DAG
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub schedule: ::std::vec::Vec<Schedule>,
        ///Deprecated alias for labels. List of labels for categorizing and filtering DAGs
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub tags: ::std::vec::Vec<::std::string::String>,
        ///Workspace label value for the DAG. Omitted for default DAGs and invalid workspace labels.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///Configuration for DAG run artifact storage
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Configuration for DAG run artifact storage",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled"
    ///  ],
    ///  "properties": {
    ///    "dir": {
    ///      "description": "Base directory for storing artifacts for this DAG when explicitly configured",
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether artifact storage is enabled for this DAG",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagArtifactsConfig {
        ///Base directory for storing artifacts for this DAG when explicitly configured
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub dir: ::std::option::Option<::std::string::String>,
        ///Whether artifact storage is enabled for this DAG
        pub enabled: bool,
    }
    ///Detailed DAG configuration information
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Detailed DAG configuration information",
    ///  "type": "object",
    ///  "required": [
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "artifacts": {
    ///      "$ref": "#/components/schemas/DAGArtifactsConfig"
    ///    },
    ///    "defaultParams": {
    ///      "description": "Default parameter values in JSON format if not specified at DAG-run creation",
    ///      "type": "string"
    ///    },
    ///    "delay": {
    ///      "description": "Time in seconds to wait before starting a DAG-run",
    ///      "type": "integer"
    ///    },
    ///    "description": {
    ///      "description": "Human-readable description of the DAG's purpose and behavior",
    ///      "type": "string"
    ///    },
    ///    "env": {
    ///      "description": "List of environment variables to set before executing a DAG-run",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "group": {
    ///      "description": "Logical grouping of related DAGs for organizational purposes",
    ///      "type": "string"
    ///    },
    ///    "handlerOn": {
    ///      "$ref": "#/components/schemas/HandlerOn"
    ///    },
    ///    "histRetentionDays": {
    ///      "description": "Number of days to retain historical logs",
    ///      "type": "integer"
    ///    },
    ///    "histRetentionRuns": {
    ///      "description": "Number of DAG runs to retain historical logs. Mutually exclusive with histRetentionDays.",
    ///      "type": "integer"
    ///    },
    ///    "labels": {
    ///      "description": "List of labels for categorizing and filtering DAGs",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "logDir": {
    ///      "description": "Directory path for storing log files",
    ///      "type": "string"
    ///    },
    ///    "maxActiveRuns": {
    ///      "description": "DEPRECATED: This field is ignored for local (DAG-based) queues. For concurrency control, use global queues",
    ///      "deprecated": true,
    ///      "type": "integer",
    ///      "x-deprecated-reason": "For concurrency control, configure global queues in config.yaml instead"
    ///    },
    ///    "maxActiveSteps": {
    ///      "description": "Maximum number of concurrent steps allowed in a DAG run",
    ///      "type": "integer"
    ///    },
    ///    "name": {
    ///      "description": "Unique identifier for the DAG within its group",
    ///      "type": "string"
    ///    },
    ///    "nextRun": {
    ///      "description": "Scheduler-aware next planned run time. Pending overdue one-offs remain visible until consumed.",
    ///      "type": "string"
    ///    },
    ///    "paramDefs": {
    ///      "description": "Ordered parameter definitions derived from DAG params for typed UI rendering and validation",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ParamDef"
    ///      }
    ///    },
    ///    "paramSchema": {
    ///      "description": "Resolved JSON Schema for schema-backed DAG params when safe for direct UI form rendering",
    ///      "type": "object",
    ///      "additionalProperties": true
    ///    },
    ///    "params": {
    ///      "description": "List of parameter names that can be passed to DAG-runs created from this DAG",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "preconditions": {
    ///      "description": "Conditions that must be met before a DAG-run can start",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Condition"
    ///      }
    ///    },
    ///    "queue": {
    ///      "description": "Name of the queue this DAG is assigned to. If not specified, the DAG name itself becomes the queue name",
    ///      "type": "string"
    ///    },
    ///    "resources": {
    ///      "$ref": "#/components/schemas/DAGResources"
    ///    },
    ///    "runConfig": {
    ///      "$ref": "#/components/schemas/RunConfig"
    ///    },
    ///    "schedule": {
    ///      "description": "List of scheduling expressions defining when DAG-runs should be created from this DAG",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Schedule"
    ///      }
    ///    },
    ///    "steps": {
    ///      "description": "List of steps to execute in DAG-runs created from this DAG",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Step"
    ///      }
    ///    },
    ///    "tags": {
    ///      "description": "Deprecated alias for labels. List of labels for categorizing and filtering DAGs",
    ///      "deprecated": true,
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "tasks": {
    ///      "description": "Goals an agent DAG must satisfy. Present only for type agent.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/AgentTask"
    ///      }
    ///    },
    ///    "type": {
    ///      "description": "Execution type. 'graph' resolves dependencies, 'chain' runs steps in order, 'agent' lets an LLM choose each step, and 'build' adds file-derived dependencies and materialization reuse.",
    ///      "type": "string",
    ///      "enum": [
    ///        "graph",
    ///        "chain",
    ///        "agent",
    ///        "build"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagDetails {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub artifacts: ::std::option::Option<DagArtifactsConfig>,
        ///Default parameter values in JSON format if not specified at DAG-run creation
        #[serde(
            rename = "defaultParams",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub default_params: ::std::option::Option<::std::string::String>,
        ///Time in seconds to wait before starting a DAG-run
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub delay: ::std::option::Option<i64>,
        ///Human-readable description of the DAG's purpose and behavior
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        ///List of environment variables to set before executing a DAG-run
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub env: ::std::vec::Vec<::std::string::String>,
        ///Logical grouping of related DAGs for organizational purposes
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub group: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "handlerOn",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub handler_on: ::std::option::Option<HandlerOn>,
        ///Number of days to retain historical logs
        #[serde(
            rename = "histRetentionDays",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub hist_retention_days: ::std::option::Option<i64>,
        ///Number of DAG runs to retain historical logs. Mutually exclusive with histRetentionDays.
        #[serde(
            rename = "histRetentionRuns",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub hist_retention_runs: ::std::option::Option<i64>,
        ///List of labels for categorizing and filtering DAGs
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub labels: ::std::vec::Vec<::std::string::String>,
        ///Directory path for storing log files
        #[serde(
            rename = "logDir",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub log_dir: ::std::option::Option<::std::string::String>,
        ///DEPRECATED: This field is ignored for local (DAG-based) queues. For concurrency control, use global queues
        #[serde(
            rename = "maxActiveRuns",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub max_active_runs: ::std::option::Option<i64>,
        ///Maximum number of concurrent steps allowed in a DAG run
        #[serde(
            rename = "maxActiveSteps",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub max_active_steps: ::std::option::Option<i64>,
        ///Unique identifier for the DAG within its group
        pub name: ::std::string::String,
        ///Scheduler-aware next planned run time. Pending overdue one-offs remain visible until consumed.
        #[serde(
            rename = "nextRun",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub next_run: ::std::option::Option<::std::string::String>,
        ///Ordered parameter definitions derived from DAG params for typed UI rendering and validation
        #[serde(
            rename = "paramDefs",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub param_defs: ::std::vec::Vec<ParamDef>,
        ///Resolved JSON Schema for schema-backed DAG params when safe for direct UI form rendering
        #[serde(
            rename = "paramSchema",
            default,
            skip_serializing_if = "::serde_json::Map::is_empty"
        )]
        pub param_schema: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
        ///List of parameter names that can be passed to DAG-runs created from this DAG
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub params: ::std::vec::Vec<::std::string::String>,
        ///Conditions that must be met before a DAG-run can start
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub preconditions: ::std::vec::Vec<Condition>,
        ///Name of the queue this DAG is assigned to. If not specified, the DAG name itself becomes the queue name
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub queue: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub resources: ::std::option::Option<DagResources>,
        #[serde(
            rename = "runConfig",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub run_config: ::std::option::Option<RunConfig>,
        ///List of scheduling expressions defining when DAG-runs should be created from this DAG
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub schedule: ::std::vec::Vec<Schedule>,
        ///List of steps to execute in DAG-runs created from this DAG
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub steps: ::std::vec::Vec<Step>,
        ///Deprecated alias for labels. List of labels for categorizing and filtering DAGs
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub tags: ::std::vec::Vec<::std::string::String>,
        ///Goals an agent DAG must satisfy. Present only for type agent.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub tasks: ::std::vec::Vec<AgentTask>,
        ///Execution type. 'graph' resolves dependencies, 'chain' runs steps in order, 'agent' lets an LLM choose each step, and 'build' adds file-derived dependencies and materialization reuse.
        #[serde(
            rename = "type",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub type_: ::std::option::Option<DagDetailsType>,
    }
    ///Execution type. 'graph' resolves dependencies, 'chain' runs steps in order, 'agent' lets an LLM choose each step, and 'build' adds file-derived dependencies and materialization reuse.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Execution type. 'graph' resolves dependencies, 'chain' runs steps in order, 'agent' lets an LLM choose each step, and 'build' adds file-derived dependencies and materialization reuse.",
    ///  "type": "string",
    ///  "enum": [
    ///    "graph",
    ///    "chain",
    ///    "agent",
    ///    "build"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum DagDetailsType {
        #[serde(rename = "graph")]
        Graph,
        #[serde(rename = "chain")]
        Chain,
        #[serde(rename = "agent")]
        Agent,
        #[serde(rename = "build")]
        Build,
    }
    impl ::std::fmt::Display for DagDetailsType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Graph => f.write_str("graph"),
                Self::Chain => f.write_str("chain"),
                Self::Agent => f.write_str("agent"),
                Self::Build => f.write_str("build"),
            }
        }
    }
    impl ::std::str::FromStr for DagDetailsType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "graph" => Ok(Self::Graph),
                "chain" => Ok(Self::Chain),
                "agent" => Ok(Self::Agent),
                "build" => Ok(Self::Build),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for DagDetailsType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for DagDetailsType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for DagDetailsType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Editor-only metadata used to synthesize per-document schema hints
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Editor-only metadata used to synthesize per-document schema hints",
    ///  "type": "object",
    ///  "required": [
    ///    "inheritedLegacyDefinitions"
    ///  ],
    ///  "properties": {
    ///    "inheritedCustomActions": {
    ///      "description": "Custom actions inherited from base config and available to the current DAG",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/InheritedCustomActionHint"
    ///      }
    ///    },
    ///    "inheritedLegacyDefinitions": {
    ///      "description": "Deprecated legacy execution definitions inherited from base config and available to the current DAG",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/InheritedLegacyDefinitionHint"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagEditorHints {
        ///Custom actions inherited from base config and available to the current DAG
        #[serde(
            rename = "inheritedCustomActions",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub inherited_custom_actions: ::std::vec::Vec<InheritedCustomActionHint>,
        ///Deprecated legacy execution definitions inherited from base config and available to the current DAG
        #[serde(rename = "inheritedLegacyDefinitions")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub inherited_legacy_definitions: ::std::vec::Vec<InheritedLegacyDefinitionHint>,
    }
    ///DAG file with its status information
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "DAG file with its status information",
    ///  "type": "object",
    ///  "required": [
    ///    "dag",
    ///    "errors",
    ///    "fileName",
    ///    "latestDAGRun",
    ///    "suspended"
    ///  ],
    ///  "properties": {
    ///    "dag": {
    ///      "$ref": "#/components/schemas/DAG"
    ///    },
    ///    "errors": {
    ///      "description": "List of errors encountered during the request",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "fileName": {
    ///      "description": "File ID of the DAG file",
    ///      "type": "string"
    ///    },
    ///    "latestDAGRun": {
    ///      "$ref": "#/components/schemas/DAGRunSummary"
    ///    },
    ///    "nextRun": {
    ///      "description": "Scheduler-aware next planned run time. Pending overdue one-offs remain visible until consumed.",
    ///      "type": "string"
    ///    },
    ///    "suspended": {
    ///      "description": "Whether the DAG is suspended",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagFile {
        pub dag: Dag,
        ///List of errors encountered during the request
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub errors: ::std::vec::Vec<::std::string::String>,
        ///File ID of the DAG file
        #[serde(rename = "fileName")]
        pub file_name: ::std::string::String,
        #[serde(rename = "latestDAGRun")]
        pub latest_dag_run: DagRunSummary,
        ///Scheduler-aware next planned run time. Pending overdue one-offs remain visible until consumed.
        #[serde(
            rename = "nextRun",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub next_run: ::std::option::Option<::std::string::String>,
        ///Whether the DAG is suspended
        pub suspended: bool,
    }
    ///Name of the DAG file
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Name of the DAG file",
    ///  "type": "string",
    ///  "format": "regex"
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    #[serde(transparent)]
    pub struct DagFileName(pub ::std::string::String);
    impl ::std::ops::Deref for DagFileName {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<DagFileName> for ::std::string::String {
        fn from(value: DagFileName) -> Self {
            value.0
        }
    }
    impl ::std::convert::From<::std::string::String> for DagFileName {
        fn from(value: ::std::string::String) -> Self {
            Self(value)
        }
    }
    impl ::std::str::FromStr for DagFileName {
        type Err = ::std::convert::Infallible;
        fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::fmt::Display for DagFileName {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            self.0.fmt(f)
        }
    }
    ///Grid item for visualizing DAG-run execution history
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Grid item for visualizing DAG-run execution history",
    ///  "type": "object",
    ///  "required": [
    ///    "history",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "history": {
    ///      "description": "Status of the step ordered by time",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NodeStatus"
    ///      }
    ///    },
    ///    "name": {
    ///      "description": "Name of the step",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagGridItem {
        ///Status of the step ordered by time
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub history: ::std::vec::Vec<NodeStatus>,
        ///Name of the step
        pub name: ::std::string::String,
    }
    ///Name of the DAG
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Name of the DAG",
    ///  "type": "string",
    ///  "format": "regex"
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    #[serde(transparent)]
    pub struct DagName(pub ::std::string::String);
    impl ::std::ops::Deref for DagName {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<DagName> for ::std::string::String {
        fn from(value: DagName) -> Self {
            value.0
        }
    }
    impl ::std::convert::From<::std::string::String> for DagName {
        fn from(value: ::std::string::String) -> Self {
            Self(value)
        }
    }
    impl ::std::str::FromStr for DagName {
        type Err = ::std::convert::Infallible;
        fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::fmt::Display for DagName {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            self.0.fmt(f)
        }
    }
    ///Server-side DAG notification settings
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Server-side DAG notification settings",
    ///  "type": "object",
    ///  "required": [
    ///    "createdAt",
    ///    "dagName",
    ///    "enabled",
    ///    "events",
    ///    "id",
    ///    "subscriptions",
    ///    "targets",
    ///    "updatedAt"
    ///  ],
    ///  "properties": {
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "dagName": {
    ///      "description": "Name of the DAG these settings apply to",
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether notification delivery is enabled for this DAG",
    ///      "type": "boolean"
    ///    },
    ///    "events": {
    ///      "description": "DAG run events that trigger notifications",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationEventType"
    ///      }
    ///    },
    ///    "id": {
    ///      "description": "Stable settings ID",
    ///      "type": "string"
    ///    },
    ///    "subscriptions": {
    ///      "description": "Notification channels subscribed by this DAG",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationSubscription"
    ///      }
    ///    },
    ///    "targets": {
    ///      "description": "DAG-local notification targets kept for backward compatibility",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationTarget"
    ///      }
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    },
    ///    "updatedBy": {
    ///      "description": "User ID that last updated the settings",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagNotificationSettings {
        #[serde(rename = "createdAt")]
        pub created_at: ::std::string::String,
        ///Name of the DAG these settings apply to
        #[serde(rename = "dagName")]
        pub dag_name: ::std::string::String,
        ///Whether notification delivery is enabled for this DAG
        pub enabled: bool,
        ///DAG run events that trigger notifications
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub events: ::std::vec::Vec<NotificationEventType>,
        ///Stable settings ID
        pub id: ::std::string::String,
        ///Notification channels subscribed by this DAG
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub subscriptions: ::std::vec::Vec<NotificationSubscription>,
        ///DAG-local notification targets kept for backward compatibility
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub targets: ::std::vec::Vec<NotificationTarget>,
        #[serde(rename = "updatedAt")]
        pub updated_at: ::std::string::String,
        ///User ID that last updated the settings
        #[serde(
            rename = "updatedBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_by: ::std::option::Option<::std::string::String>,
    }
    ///CPU and memory limits requested for a DAG run
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "CPU and memory limits requested for a DAG run",
    ///  "type": "object",
    ///  "properties": {
    ///    "cpu": {
    ///      "description": "CPU limit as cores (for example, \"2\" or \"0.5\") or millicores (for example, \"500m\")",
    ///      "type": "string"
    ///    },
    ///    "memory": {
    ///      "description": "Memory limit in bytes or with a unit suffix (for example, \"512Mi\", \"1Gi\", or \"2G\")",
    ///      "type": "string"
    ///    }
    ///  },
    ///  "additionalProperties": false
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct DagResourceLimits {
        ///CPU limit as cores (for example, "2" or "0.5") or millicores (for example, "500m")
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub cpu: ::std::option::Option<::std::string::String>,
        ///Memory limit in bytes or with a unit suffix (for example, "512Mi", "1Gi", or "2G")
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub memory: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for DagResourceLimits {
        fn default() -> Self {
            Self {
                cpu: Default::default(),
                memory: Default::default(),
            }
        }
    }
    ///Resource limits requested for a DAG run
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Resource limits requested for a DAG run",
    ///  "type": "object",
    ///  "properties": {
    ///    "limits": {
    ///      "$ref": "#/components/schemas/DAGResourceLimits"
    ///    }
    ///  },
    ///  "additionalProperties": false
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct DagResources {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub limits: ::std::option::Option<DagResourceLimits>,
    }
    impl ::std::default::Default for DagResources {
        fn default() -> Self {
            Self { limits: Default::default() }
        }
    }
    ///Type-keyed current-state runtime condition for a DAG-run. Each condition is the latest observation for its type, not a historical event.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Type-keyed current-state runtime condition for a DAG-run. Each condition is the latest observation for its type, not a historical event.",
    ///  "type": "object",
    ///  "required": [
    ///    "checkedAt",
    ///    "message",
    ///    "reason",
    ///    "status",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "checkedAt": {
    ///      "description": "RFC 3339 timestamp when the condition was observed",
    ///      "type": "string"
    ///    },
    ///    "message": {
    ///      "description": "Human-readable detail for the condition status",
    ///      "type": "string"
    ///    },
    ///    "reason": {
    ///      "description": "Machine-readable reason for the condition status",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "description": "Observed status of the condition",
    ///      "type": "string",
    ///      "enum": [
    ///        "True",
    ///        "False",
    ///        "Unknown"
    ///      ]
    ///    },
    ///    "type": {
    ///      "description": "Condition type",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagRunCondition {
        ///RFC 3339 timestamp when the condition was observed
        #[serde(rename = "checkedAt")]
        pub checked_at: ::std::string::String,
        ///Human-readable detail for the condition status
        pub message: ::std::string::String,
        ///Machine-readable reason for the condition status
        pub reason: ::std::string::String,
        ///Observed status of the condition
        pub status: DagRunConditionStatus,
        ///Condition type
        #[serde(rename = "type")]
        pub type_: ::std::string::String,
    }
    ///Observed status of the condition
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Observed status of the condition",
    ///  "type": "string",
    ///  "enum": [
    ///    "True",
    ///    "False",
    ///    "Unknown"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum DagRunConditionStatus {
        True,
        False,
        Unknown,
    }
    impl ::std::fmt::Display for DagRunConditionStatus {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::True => f.write_str("True"),
                Self::False => f.write_str("False"),
                Self::Unknown => f.write_str("Unknown"),
            }
        }
    }
    impl ::std::str::FromStr for DagRunConditionStatus {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "True" => Ok(Self::True),
                "False" => Ok(Self::False),
                "Unknown" => Ok(Self::Unknown),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for DagRunConditionStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for DagRunConditionStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for DagRunConditionStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Unique identifier for a newly-created DAG-run. The special value 'latest' is not allowed.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Unique identifier for a newly-created DAG-run. The special value 'latest' is not allowed.",
    ///  "examples": [
    ///    "20240101_120000"
    ///  ],
    ///  "type": "string"
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    #[serde(transparent)]
    pub struct DagRunCreateId(pub ::std::string::String);
    impl ::std::ops::Deref for DagRunCreateId {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<DagRunCreateId> for ::std::string::String {
        fn from(value: DagRunCreateId) -> Self {
            value.0
        }
    }
    impl ::std::convert::From<::std::string::String> for DagRunCreateId {
        fn from(value: ::std::string::String) -> Self {
            Self(value)
        }
    }
    impl ::std::str::FromStr for DagRunCreateId {
        type Err = ::std::convert::Infallible;
        fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::fmt::Display for DagRunCreateId {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            self.0.fmt(f)
        }
    }
    ///Detailed status of a DAG-run including sub DAG-run nodes
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Detailed status of a DAG-run including sub DAG-run nodes",
    ///  "type": "object",
    ///  "allOf": [
    ///    {
    ///      "$ref": "#/components/schemas/DAGRunSummary"
    ///    },
    ///    {
    ///      "description": "Detailed status information for the steps within a DAG-run.",
    ///      "type": "object",
    ///      "required": [
    ///        "log",
    ///        "nodes",
    ///        "rootDAGRunId",
    ///        "rootDAGRunName"
    ///      ],
    ///      "properties": {
    ///        "agentEvents": {
    ///          "description": "Ordered decision timeline of an agent DAG-run: what the agent ran, in what order, and when each task was satisfied. Absent for other DAG types.",
    ///          "type": "array",
    ///          "items": {
    ///            "$ref": "#/components/schemas/AgentEvent"
    ///          }
    ///        },
    ///        "agentTasks": {
    ///          "description": "Goal progress of an agent DAG-run. Absent for other DAG types.",
    ///          "type": "array",
    ///          "items": {
    ///            "$ref": "#/components/schemas/AgentTask"
    ///          }
    ///        },
    ///        "humanTaskResumePending": {
    ///          "description": "Whether completed human-task input is durable but the same DAG-run still needs its retry queued",
    ///          "type": "boolean"
    ///        },
    ///        "log": {
    ///          "description": "Path to the log file",
    ///          "type": "string"
    ///        },
    ///        "nodes": {
    ///          "description": "Status of individual steps within the DAG-run",
    ///          "type": "array",
    ///          "items": {
    ///            "$ref": "#/components/schemas/Node"
    ///          }
    ///        },
    ///        "onAbort": {
    ///          "$ref": "#/components/schemas/Node"
    ///        },
    ///        "onExit": {
    ///          "$ref": "#/components/schemas/Node"
    ///        },
    ///        "onFailure": {
    ///          "$ref": "#/components/schemas/Node"
    ///        },
    ///        "onInit": {
    ///          "$ref": "#/components/schemas/Node"
    ///        },
    ///        "onSuccess": {
    ///          "$ref": "#/components/schemas/Node"
    ///        },
    ///        "onWait": {
    ///          "$ref": "#/components/schemas/Node"
    ///        },
    ///        "parentDAGRunId": {
    ///          "allOf": [
    ///            {
    ///              "$ref": "#/components/schemas/DAGRunId"
    ///            },
    ///            {
    ///              "description": "ID of the parent DAG-run"
    ///            }
    ///          ]
    ///        },
    ///        "parentDAGRunName": {
    ///          "description": "Name of the parent DAG-run",
    ///          "type": "string"
    ///        },
    ///        "preconditions": {
    ///          "description": "List of preconditions that must be met before the DAG-run can start",
    ///          "type": "array",
    ///          "items": {
    ///            "$ref": "#/components/schemas/Condition"
    ///          }
    ///        },
    ///        "rootDAGRunId": {
    ///          "allOf": [
    ///            {
    ///              "$ref": "#/components/schemas/DAGRunId"
    ///            },
    ///            {
    ///              "description": "ID of the root DAG-run"
    ///            }
    ///          ]
    ///        },
    ///        "rootDAGRunName": {
    ///          "description": "Name of the root DAG-run",
    ///          "type": "string"
    ///        },
    ///        "sourceFileName": {
    ///          "$ref": "#/components/schemas/DAGFileName"
    ///        },
    ///        "specFromFile": {
    ///          "description": "Whether this DAG-run still has a usable source file on disk, so reschedule can load the current spec from that file instead of the stored historical YAML snapshot.",
    ///          "type": "boolean"
    ///        }
    ///      }
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagRunDetails {
        ///Ordered decision timeline of an agent DAG-run: what the agent ran, in what order, and when each task was satisfied. Absent for other DAG types.
        #[serde(
            rename = "agentEvents",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub agent_events: ::std::vec::Vec<AgentEvent>,
        ///Goal progress of an agent DAG-run. Absent for other DAG types.
        #[serde(
            rename = "agentTasks",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub agent_tasks: ::std::vec::Vec<AgentTask>,
        ///Whether artifact files are available for this DAG-run
        #[serde(rename = "artifactsAvailable")]
        pub artifacts_available: bool,
        ///Number of scheduler-issued DAG auto-retries already consumed for this DAG-run
        #[serde(rename = "autoRetryCount")]
        pub auto_retry_count: i64,
        ///Configured DAG-level automatic retry limit captured for this DAG-run; null when DAG-level automatic retry is not configured
        #[serde(
            rename = "autoRetryLimit",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub auto_retry_limit: ::std::option::Option<i64>,
        ///Type-keyed current-state runtime conditions for the DAG-run. This list reports the latest condition for each type, not a history of queued reasons.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub conditions: ::std::vec::Vec<DagRunCondition>,
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
        ///RFC 3339 timestamp when the DAG-run finished
        #[serde(rename = "finishedAt")]
        pub finished_at: ::std::string::String,
        ///Whether completed human-task input is durable but the same DAG-run still needs its retry queued
        #[serde(
            rename = "humanTaskResumePending",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub human_task_resume_pending: ::std::option::Option<bool>,
        ///List of labels for categorizing and filtering DAG runs
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub labels: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
        ///Path to the log file
        pub log: ::std::string::String,
        pub name: DagName,
        ///Whether reuse of prior build materializations was disabled for this DAG-run
        #[serde(
            rename = "noReuse",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub no_reuse: ::std::option::Option<bool>,
        ///Status of individual steps within the DAG-run
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub nodes: ::std::vec::Vec<Node>,
        #[serde(
            rename = "onAbort",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub on_abort: ::std::option::Option<Node>,
        #[serde(
            rename = "onExit",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub on_exit: ::std::option::Option<Node>,
        #[serde(
            rename = "onFailure",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub on_failure: ::std::option::Option<Node>,
        #[serde(
            rename = "onInit",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub on_init: ::std::option::Option<Node>,
        #[serde(
            rename = "onSuccess",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub on_success: ::std::option::Option<Node>,
        #[serde(
            rename = "onWait",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub on_wait: ::std::option::Option<Node>,
        ///Runtime parameters passed to the DAG-run in JSON format
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub params: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "parentDAGRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub parent_dag_run_id: ::std::option::Option<DagRunId>,
        ///Name of the parent DAG-run
        #[serde(
            rename = "parentDAGRunName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub parent_dag_run_name: ::std::option::Option<::std::string::String>,
        ///List of preconditions that must be met before the DAG-run can start
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub preconditions: ::std::vec::Vec<Condition>,
        #[serde(
            rename = "profileName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub profile_name: ::std::option::Option<RuntimeProfileName>,
        ///RFC 3339 timestamp when the DAG-run was queued
        #[serde(
            rename = "queuedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub queued_at: ::std::option::Option<::std::string::String>,
        #[serde(rename = "rootDAGRunId")]
        pub root_dag_run_id: DagRunId,
        ///Name of the root DAG-run
        #[serde(rename = "rootDAGRunName")]
        pub root_dag_run_name: ::std::string::String,
        ///RFC 3339 timestamp of when the DAG-run was scheduled to run
        #[serde(
            rename = "scheduleTime",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub schedule_time: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "sourceFileName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub source_file_name: ::std::option::Option<DagFileName>,
        ///Whether this DAG-run still has a usable source file on disk, so reschedule can load the current spec from that file instead of the stored historical YAML snapshot.
        #[serde(
            rename = "specFromFile",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub spec_from_file: ::std::option::Option<bool>,
        ///RFC 3339 timestamp when the DAG-run started
        #[serde(rename = "startedAt")]
        pub started_at: ::std::string::String,
        pub status: Status,
        #[serde(rename = "statusLabel")]
        pub status_label: StatusLabel,
        ///Deprecated alias for labels. List of labels for categorizing and filtering DAG runs
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub tags: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
        ///Authenticated actor that initiated the DAG-run, when attribution is available
        #[serde(
            rename = "triggerActor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub trigger_actor: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "triggerType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub trigger_type: ::std::option::Option<TriggerType>,
        ///ID of the worker that executed this DAG-run ('local' for local execution)
        #[serde(
            rename = "workerId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub worker_id: ::std::option::Option<::std::string::String>,
        ///Workspace label value for the DAG-run. Omitted for default DAG-runs and invalid workspace labels.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///Unique identifier for the DAG-run. The special value 'latest' can be used to reference the most recent DAG-run.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Unique identifier for the DAG-run. The special value 'latest' can be used to reference the most recent DAG-run.",
    ///  "examples": [
    ///    "latest"
    ///  ],
    ///  "type": "string"
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    #[serde(transparent)]
    pub struct DagRunId(pub ::std::string::String);
    impl ::std::ops::Deref for DagRunId {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<DagRunId> for ::std::string::String {
        fn from(value: DagRunId) -> Self {
            value.0
        }
    }
    impl ::std::convert::From<::std::string::String> for DagRunId {
        fn from(value: ::std::string::String) -> Self {
            Self(value)
        }
    }
    impl ::std::str::FromStr for DagRunId {
        type Err = ::std::convert::Infallible;
        fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::fmt::Display for DagRunId {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            self.0.fmt(f)
        }
    }
    ///Collected outputs from step executions in a DAG-run, including execution metadata. Outputs are populated from string-form output, stdout.outputs, and outputs.write. If the DAG-run completed but no outputs were captured, the outputs object will be empty and metadata fields may be empty strings.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Collected outputs from step executions in a DAG-run, including execution metadata. Outputs are populated from string-form output, stdout.outputs, and outputs.write. If the DAG-run completed but no outputs were captured, the outputs object will be empty and metadata fields may be empty strings.",
    ///  "type": "object",
    ///  "required": [
    ///    "metadata",
    ///    "outputs"
    ///  ],
    ///  "properties": {
    ///    "metadata": {
    ///      "$ref": "#/components/schemas/OutputsMetadata"
    ///    },
    ///    "outputs": {
    ///      "description": "Collected step outputs as key-value pairs. String-form output names are converted from UPPER_CASE to camelCase; stdout.outputs and outputs.write keys are preserved. Values are strings in this API response. Empty object if no outputs were captured.",
    ///      "examples": [
    ///        {
    ///          "config": "{\"key\": \"value\"}",
    ///          "resultFile": "/path/to/result.txt",
    ///          "totalCount": "42"
    ///        }
    ///      ],
    ///      "type": "object",
    ///      "additionalProperties": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagRunOutputs {
        pub metadata: OutputsMetadata,
        ///Collected step outputs as key-value pairs. String-form output names are converted from UPPER_CASE to camelCase; stdout.outputs and outputs.write keys are preserved. Values are strings in this API response. Empty object if no outputs were captured.
        pub outputs: ::std::collections::HashMap<
            ::std::string::String,
            ::std::string::String,
        >,
    }
    ///Current status of a DAG-run
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Current status of a DAG-run",
    ///  "type": "object",
    ///  "required": [
    ///    "artifactsAvailable",
    ///    "autoRetryCount",
    ///    "dagRunId",
    ///    "finishedAt",
    ///    "name",
    ///    "startedAt",
    ///    "status",
    ///    "statusLabel"
    ///  ],
    ///  "properties": {
    ///    "artifactsAvailable": {
    ///      "description": "Whether artifact files are available for this DAG-run",
    ///      "type": "boolean"
    ///    },
    ///    "autoRetryCount": {
    ///      "description": "Number of scheduler-issued DAG auto-retries already consumed for this DAG-run",
    ///      "type": "integer"
    ///    },
    ///    "autoRetryLimit": {
    ///      "description": "Configured DAG-level automatic retry limit captured for this DAG-run; null when DAG-level automatic retry is not configured",
    ///      "type": [
    ///        "integer",
    ///        "null"
    ///      ]
    ///    },
    ///    "conditions": {
    ///      "description": "Type-keyed current-state runtime conditions for the DAG-run. This list reports the latest condition for each type, not a history of queued reasons.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/DAGRunCondition"
    ///      }
    ///    },
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    },
    ///    "finishedAt": {
    ///      "description": "RFC 3339 timestamp when the DAG-run finished",
    ///      "type": "string"
    ///    },
    ///    "labels": {
    ///      "description": "List of labels for categorizing and filtering DAG runs",
    ///      "type": [
    ///        "array",
    ///        "null"
    ///      ],
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "name": {
    ///      "$ref": "#/components/schemas/DAGName"
    ///    },
    ///    "noReuse": {
    ///      "description": "Whether reuse of prior build materializations was disabled for this DAG-run",
    ///      "type": "boolean"
    ///    },
    ///    "params": {
    ///      "description": "Runtime parameters passed to the DAG-run in JSON format",
    ///      "type": "string"
    ///    },
    ///    "profileName": {
    ///      "$ref": "#/components/schemas/RuntimeProfileName"
    ///    },
    ///    "queuedAt": {
    ///      "description": "RFC 3339 timestamp when the DAG-run was queued",
    ///      "type": "string"
    ///    },
    ///    "scheduleTime": {
    ///      "description": "RFC 3339 timestamp of when the DAG-run was scheduled to run",
    ///      "type": "string"
    ///    },
    ///    "startedAt": {
    ///      "description": "RFC 3339 timestamp when the DAG-run started",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "$ref": "#/components/schemas/Status"
    ///    },
    ///    "statusLabel": {
    ///      "$ref": "#/components/schemas/StatusLabel"
    ///    },
    ///    "tags": {
    ///      "description": "Deprecated alias for labels. List of labels for categorizing and filtering DAG runs",
    ///      "deprecated": true,
    ///      "type": [
    ///        "array",
    ///        "null"
    ///      ],
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "triggerActor": {
    ///      "description": "Authenticated actor that initiated the DAG-run, when attribution is available",
    ///      "type": "string"
    ///    },
    ///    "triggerType": {
    ///      "$ref": "#/components/schemas/TriggerType"
    ///    },
    ///    "workerId": {
    ///      "description": "ID of the worker that executed this DAG-run ('local' for local execution)",
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "description": "Workspace label value for the DAG-run. Omitted for default DAG-runs and invalid workspace labels.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagRunSummary {
        ///Whether artifact files are available for this DAG-run
        #[serde(rename = "artifactsAvailable")]
        pub artifacts_available: bool,
        ///Number of scheduler-issued DAG auto-retries already consumed for this DAG-run
        #[serde(rename = "autoRetryCount")]
        pub auto_retry_count: i64,
        ///Configured DAG-level automatic retry limit captured for this DAG-run; null when DAG-level automatic retry is not configured
        #[serde(
            rename = "autoRetryLimit",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub auto_retry_limit: ::std::option::Option<i64>,
        ///Type-keyed current-state runtime conditions for the DAG-run. This list reports the latest condition for each type, not a history of queued reasons.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub conditions: ::std::vec::Vec<DagRunCondition>,
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
        ///RFC 3339 timestamp when the DAG-run finished
        #[serde(rename = "finishedAt")]
        pub finished_at: ::std::string::String,
        ///List of labels for categorizing and filtering DAG runs
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub labels: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
        pub name: DagName,
        ///Whether reuse of prior build materializations was disabled for this DAG-run
        #[serde(
            rename = "noReuse",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub no_reuse: ::std::option::Option<bool>,
        ///Runtime parameters passed to the DAG-run in JSON format
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub params: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "profileName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub profile_name: ::std::option::Option<RuntimeProfileName>,
        ///RFC 3339 timestamp when the DAG-run was queued
        #[serde(
            rename = "queuedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub queued_at: ::std::option::Option<::std::string::String>,
        ///RFC 3339 timestamp of when the DAG-run was scheduled to run
        #[serde(
            rename = "scheduleTime",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub schedule_time: ::std::option::Option<::std::string::String>,
        ///RFC 3339 timestamp when the DAG-run started
        #[serde(rename = "startedAt")]
        pub started_at: ::std::string::String,
        pub status: Status,
        #[serde(rename = "statusLabel")]
        pub status_label: StatusLabel,
        ///Deprecated alias for labels. List of labels for categorizing and filtering DAG runs
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub tags: ::std::option::Option<::std::vec::Vec<::std::string::String>>,
        ///Authenticated actor that initiated the DAG-run, when attribution is available
        #[serde(
            rename = "triggerActor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub trigger_actor: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "triggerType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub trigger_type: ::std::option::Option<TriggerType>,
        ///ID of the worker that executed this DAG-run ('local' for local execution)
        #[serde(
            rename = "workerId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub worker_id: ::std::option::Option<::std::string::String>,
        ///Workspace label value for the DAG-run. Omitted for default DAG-runs and invalid workspace labels.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///Forward-only paginated DAG-run list response
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Forward-only paginated DAG-run list response",
    ///  "type": "object",
    ///  "required": [
    ///    "dagRuns"
    ///  ],
    ///  "properties": {
    ///    "dagRuns": {
    ///      "description": "List of DAG-runs with their status and metadata",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/DAGRunSummary"
    ///      }
    ///    },
    ///    "nextCursor": {
    ///      "description": "Opaque cursor for loading the next page of older DAG-runs",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagRunsPageResponse {
        ///List of DAG-runs with their status and metadata
        #[serde(rename = "dagRuns")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub dag_runs: ::std::vec::Vec<DagRunSummary>,
        ///Opaque cursor for loading the next page of older DAG-runs
        #[serde(
            rename = "nextCursor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub next_cursor: ::std::option::Option<::std::string::String>,
    }
    ///Cursor-based DAG search results
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Cursor-based DAG search results",
    ///  "type": "object",
    ///  "required": [
    ///    "hasMore",
    ///    "results"
    ///  ],
    ///  "properties": {
    ///    "hasMore": {
    ///      "type": "boolean"
    ///    },
    ///    "nextCursor": {
    ///      "type": "string"
    ///    },
    ///    "results": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/DAGSearchPageItem"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagSearchFeedResponse {
        #[serde(rename = "hasMore")]
        pub has_more: bool,
        #[serde(
            rename = "nextCursor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub next_cursor: ::std::option::Option<::std::string::String>,
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub results: ::std::vec::Vec<DagSearchPageItem>,
    }
    ///Lightweight cursor-based search result item for a DAG
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Lightweight cursor-based search result item for a DAG",
    ///  "type": "object",
    ///  "required": [
    ///    "fileName",
    ///    "hasMoreMatches",
    ///    "matches",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "fileName": {
    ///      "description": "DAG file name without extension",
    ///      "type": "string"
    ///    },
    ///    "hasMoreMatches": {
    ///      "description": "Whether additional snippets are available beyond the preview",
    ///      "type": "boolean"
    ///    },
    ///    "matches": {
    ///      "description": "Preview snippets for the result",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/SearchMatchItem"
    ///      }
    ///    },
    ///    "name": {
    ///      "description": "Display label for the DAG result; file-backed search currently mirrors fileName",
    ///      "type": "string"
    ///    },
    ///    "nextMatchesCursor": {
    ///      "description": "Opaque cursor for loading more snippets for this DAG result",
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "description": "Workspace label value for the matching DAG. Omitted for default DAGs and invalid workspace labels.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagSearchPageItem {
        ///DAG file name without extension
        #[serde(rename = "fileName")]
        pub file_name: ::std::string::String,
        ///Whether additional snippets are available beyond the preview
        #[serde(rename = "hasMoreMatches")]
        pub has_more_matches: bool,
        ///Preview snippets for the result
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub matches: ::std::vec::Vec<SearchMatchItem>,
        ///Display label for the DAG result; file-backed search currently mirrors fileName
        pub name: ::std::string::String,
        ///Opaque cursor for loading more snippets for this DAG result
        #[serde(
            rename = "nextMatchesCursor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub next_matches_cursor: ::std::option::Option<::std::string::String>,
        ///Workspace label value for the matching DAG. Omitted for default DAGs and invalid workspace labels.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///Server-side DAG settings
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Server-side DAG settings",
    ///  "type": "object",
    ///  "required": [
    ///    "dagName"
    ///  ],
    ///  "properties": {
    ///    "dagName": {
    ///      "description": "DAG file identifier these settings apply to",
    ///      "type": "string"
    ///    },
    ///    "profile": {
    ///      "$ref": "#/components/schemas/RuntimeProfileName"
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    },
    ///    "updatedBy": {
    ///      "description": "User ID that last updated the settings",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct DagSettings {
        ///DAG file identifier these settings apply to
        #[serde(rename = "dagName")]
        pub dag_name: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub profile: ::std::option::Option<RuntimeProfileName>,
        #[serde(
            rename = "updatedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_at: ::std::option::Option<::std::string::String>,
        ///User ID that last updated the settings
        #[serde(
            rename = "updatedBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_by: ::std::option::Option<::std::string::String>,
    }
    ///`EnqueueDagRunFromSpecBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "spec"
    ///  ],
    ///  "properties": {
    ///    "dagRunId": {
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/DAGRunId"
    ///        },
    ///        {
    ///          "description": "Optional ID for the DAG-run; if omitted a new one will be generated"
    ///        }
    ///      ]
    ///    },
    ///    "labels": {
    ///      "$ref": "#/components/schemas/Labels"
    ///    },
    ///    "name": {
    ///      "description": "Optional name to use when the spec omits a name",
    ///      "type": "string"
    ///    },
    ///    "noReuse": {
    ///      "description": "If true, execute eligible build steps without reusing prior materializations",
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "params": {
    ///      "description": "Parameters to persist with the queued DAG-run in JSON format",
    ///      "type": "string"
    ///    },
    ///    "profile": {
    ///      "$ref": "#/components/schemas/RuntimeProfileOverride"
    ///    },
    ///    "queue": {
    ///      "description": "Override the queue to use for this DAG-run",
    ///      "type": "string"
    ///    },
    ///    "singleton": {
    ///      "description": "If true, prevent enqueuing if DAG is already running or queued (returns 409 conflict)",
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "spec": {
    ///      "description": "DAG specification in YAML format",
    ///      "type": "string"
    ///    },
    ///    "tags": {
    ///      "$ref": "#/components/schemas/Tags"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct EnqueueDagRunFromSpecBody {
        #[serde(
            rename = "dagRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_run_id: ::std::option::Option<DagRunId>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub labels: ::std::option::Option<Labels>,
        ///Optional name to use when the spec omits a name
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub name: ::std::option::Option<::std::string::String>,
        ///If true, execute eligible build steps without reusing prior materializations
        #[serde(rename = "noReuse", default)]
        pub no_reuse: bool,
        ///Parameters to persist with the queued DAG-run in JSON format
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub params: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub profile: ::std::option::Option<RuntimeProfileOverride>,
        ///Override the queue to use for this DAG-run
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub queue: ::std::option::Option<::std::string::String>,
        ///If true, prevent enqueuing if DAG is already running or queued (returns 409 conflict)
        #[serde(default)]
        pub singleton: bool,
        ///DAG specification in YAML format
        pub spec: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub tags: ::std::option::Option<Tags>,
    }
    ///`EnqueueDagRunFromSpecResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunId"
    ///  ],
    ///  "properties": {
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct EnqueueDagRunFromSpecResponse {
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
    }
    ///`EnqueueDagdagRunBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "dagName": {
    ///      "description": "Optional DAG name override to use for the queued dag-run",
    ///      "type": "string"
    ///    },
    ///    "dagRunId": {
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/DAGRunId"
    ///        },
    ///        {
    ///          "description": "Optional ID for the DAG-run, if not provided a new one will be generated"
    ///        }
    ///      ]
    ///    },
    ///    "labels": {
    ///      "$ref": "#/components/schemas/Labels"
    ///    },
    ///    "noReuse": {
    ///      "description": "If true, execute eligible build steps without reusing prior materializations",
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "params": {
    ///      "description": "Parameters to pass to the DAG-run in JSON format",
    ///      "type": "string"
    ///    },
    ///    "profile": {
    ///      "$ref": "#/components/schemas/RuntimeProfileOverride"
    ///    },
    ///    "queue": {
    ///      "description": "Override the DAG-level queue definition",
    ///      "type": "string"
    ///    },
    ///    "singleton": {
    ///      "description": "If true, prevent enqueuing if DAG is already running or queued (returns 409 conflict)",
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "tags": {
    ///      "$ref": "#/components/schemas/Tags"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct EnqueueDagdagRunBody {
        ///Optional DAG name override to use for the queued dag-run
        #[serde(
            rename = "dagName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_name: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "dagRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_run_id: ::std::option::Option<DagRunId>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub labels: ::std::option::Option<Labels>,
        ///If true, execute eligible build steps without reusing prior materializations
        #[serde(rename = "noReuse", default)]
        pub no_reuse: bool,
        ///Parameters to pass to the DAG-run in JSON format
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub params: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub profile: ::std::option::Option<RuntimeProfileOverride>,
        ///Override the DAG-level queue definition
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub queue: ::std::option::Option<::std::string::String>,
        ///If true, prevent enqueuing if DAG is already running or queued (returns 409 conflict)
        #[serde(default)]
        pub singleton: bool,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub tags: ::std::option::Option<Tags>,
    }
    impl ::std::default::Default for EnqueueDagdagRunBody {
        fn default() -> Self {
            Self {
                dag_name: Default::default(),
                dag_run_id: Default::default(),
                labels: Default::default(),
                no_reuse: Default::default(),
                params: Default::default(),
                profile: Default::default(),
                queue: Default::default(),
                singleton: Default::default(),
                tags: Default::default(),
            }
        }
    }
    ///`EnqueueDagdagRunResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunId"
    ///  ],
    ///  "properties": {
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct EnqueueDagdagRunResponse {
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
    }
    ///Generic error response object
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Generic error response object",
    ///  "type": "object",
    ///  "required": [
    ///    "code",
    ///    "message"
    ///  ],
    ///  "properties": {
    ///    "code": {
    ///      "$ref": "#/components/schemas/ErrorCode"
    ///    },
    ///    "details": {
    ///      "description": "Additional error details",
    ///      "type": "object"
    ///    },
    ///    "message": {
    ///      "description": "Short error message",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct Error {
        pub code: ErrorCode,
        ///Additional error details
        #[serde(default, skip_serializing_if = "::serde_json::Map::is_empty")]
        pub details: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
        ///Short error message
        pub message: ::std::string::String,
    }
    ///Error code indicating the type of error
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Error code indicating the type of error",
    ///  "type": "string",
    ///  "enum": [
    ///    "forbidden",
    ///    "bad_request",
    ///    "not_found",
    ///    "internal_error",
    ///    "unauthorized",
    ///    "bad_gateway",
    ///    "remote_node_error",
    ///    "max_run_reached",
    ///    "not_running",
    ///    "already_exists",
    ///    "auth.unauthorized",
    ///    "auth.token_invalid",
    ///    "auth.forbidden",
    ///    "timeout",
    ///    "rate_limited",
    ///    "conflict",
    ///    "human_task_resume_failed",
    ///    "payload_too_large"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ErrorCode {
        #[serde(rename = "forbidden")]
        Forbidden,
        #[serde(rename = "bad_request")]
        BadRequest,
        #[serde(rename = "not_found")]
        NotFound,
        #[serde(rename = "internal_error")]
        InternalError,
        #[serde(rename = "unauthorized")]
        Unauthorized,
        #[serde(rename = "bad_gateway")]
        BadGateway,
        #[serde(rename = "remote_node_error")]
        RemoteNodeError,
        #[serde(rename = "max_run_reached")]
        MaxRunReached,
        #[serde(rename = "not_running")]
        NotRunning,
        #[serde(rename = "already_exists")]
        AlreadyExists,
        #[serde(rename = "auth.unauthorized")]
        AuthUnauthorized,
        #[serde(rename = "auth.token_invalid")]
        AuthTokenInvalid,
        #[serde(rename = "auth.forbidden")]
        AuthForbidden,
        #[serde(rename = "timeout")]
        Timeout,
        #[serde(rename = "rate_limited")]
        RateLimited,
        #[serde(rename = "conflict")]
        Conflict,
        #[serde(rename = "human_task_resume_failed")]
        HumanTaskResumeFailed,
        #[serde(rename = "payload_too_large")]
        PayloadTooLarge,
    }
    impl ::std::fmt::Display for ErrorCode {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Forbidden => f.write_str("forbidden"),
                Self::BadRequest => f.write_str("bad_request"),
                Self::NotFound => f.write_str("not_found"),
                Self::InternalError => f.write_str("internal_error"),
                Self::Unauthorized => f.write_str("unauthorized"),
                Self::BadGateway => f.write_str("bad_gateway"),
                Self::RemoteNodeError => f.write_str("remote_node_error"),
                Self::MaxRunReached => f.write_str("max_run_reached"),
                Self::NotRunning => f.write_str("not_running"),
                Self::AlreadyExists => f.write_str("already_exists"),
                Self::AuthUnauthorized => f.write_str("auth.unauthorized"),
                Self::AuthTokenInvalid => f.write_str("auth.token_invalid"),
                Self::AuthForbidden => f.write_str("auth.forbidden"),
                Self::Timeout => f.write_str("timeout"),
                Self::RateLimited => f.write_str("rate_limited"),
                Self::Conflict => f.write_str("conflict"),
                Self::HumanTaskResumeFailed => f.write_str("human_task_resume_failed"),
                Self::PayloadTooLarge => f.write_str("payload_too_large"),
            }
        }
    }
    impl ::std::str::FromStr for ErrorCode {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "forbidden" => Ok(Self::Forbidden),
                "bad_request" => Ok(Self::BadRequest),
                "not_found" => Ok(Self::NotFound),
                "internal_error" => Ok(Self::InternalError),
                "unauthorized" => Ok(Self::Unauthorized),
                "bad_gateway" => Ok(Self::BadGateway),
                "remote_node_error" => Ok(Self::RemoteNodeError),
                "max_run_reached" => Ok(Self::MaxRunReached),
                "not_running" => Ok(Self::NotRunning),
                "already_exists" => Ok(Self::AlreadyExists),
                "auth.unauthorized" => Ok(Self::AuthUnauthorized),
                "auth.token_invalid" => Ok(Self::AuthTokenInvalid),
                "auth.forbidden" => Ok(Self::AuthForbidden),
                "timeout" => Ok(Self::Timeout),
                "rate_limited" => Ok(Self::RateLimited),
                "conflict" => Ok(Self::Conflict),
                "human_task_resume_failed" => Ok(Self::HumanTaskResumeFailed),
                "payload_too_large" => Ok(Self::PayloadTooLarge),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ErrorCode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ErrorCode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ErrorCode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///A single centralized operational event log entry
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A single centralized operational event log entry",
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "kind",
    ///    "occurredAt",
    ///    "recordedAt",
    ///    "schemaVersion",
    ///    "sourceService",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "attemptId": {
    ///      "description": "Attempt ID for DAG-run events",
    ///      "type": "string"
    ///    },
    ///    "dagName": {
    ///      "description": "DAG name for DAG-run events",
    ///      "type": "string"
    ///    },
    ///    "dagRunId": {
    ///      "description": "DAG run ID for DAG-run events",
    ///      "type": "string"
    ///    },
    ///    "data": {
    ///      "description": "Small event-specific payload",
    ///      "type": "object",
    ///      "additionalProperties": true
    ///    },
    ///    "id": {
    ///      "description": "Unique identifier for this event",
    ///      "type": "string"
    ///    },
    ///    "kind": {
    ///      "description": "High-level event kind (e.g., dag_run, llm_usage)",
    ///      "type": "string"
    ///    },
    ///    "model": {
    ///      "description": "Model name associated with the event",
    ///      "type": "string"
    ///    },
    ///    "occurredAt": {
    ///      "description": "When the event occurred",
    ///      "type": "string"
    ///    },
    ///    "recordedAt": {
    ///      "description": "When the event was recorded by the producer",
    ///      "type": "string"
    ///    },
    ///    "schemaVersion": {
    ///      "description": "Schema version of the event envelope",
    ///      "type": "integer"
    ///    },
    ///    "sessionId": {
    ///      "description": "Session ID for LLM usage events",
    ///      "type": "string"
    ///    },
    ///    "sourceInstance": {
    ///      "description": "Specific producer instance identifier",
    ///      "type": "string"
    ///    },
    ///    "sourceService": {
    ///      "description": "Service that produced the event",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "description": "Status associated with the event when applicable",
    ///      "type": "string"
    ///    },
    ///    "type": {
    ///      "description": "Specific event type (e.g., dag.run.failed)",
    ///      "type": "string"
    ///    },
    ///    "userId": {
    ///      "description": "User ID associated with the event",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct EventLogEntry {
        ///Attempt ID for DAG-run events
        #[serde(
            rename = "attemptId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub attempt_id: ::std::option::Option<::std::string::String>,
        ///DAG name for DAG-run events
        #[serde(
            rename = "dagName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_name: ::std::option::Option<::std::string::String>,
        ///DAG run ID for DAG-run events
        #[serde(
            rename = "dagRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_run_id: ::std::option::Option<::std::string::String>,
        ///Small event-specific payload
        #[serde(default, skip_serializing_if = "::serde_json::Map::is_empty")]
        pub data: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
        ///Unique identifier for this event
        pub id: ::std::string::String,
        ///High-level event kind (e.g., dag_run, llm_usage)
        pub kind: ::std::string::String,
        ///Model name associated with the event
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub model: ::std::option::Option<::std::string::String>,
        ///When the event occurred
        #[serde(rename = "occurredAt")]
        pub occurred_at: ::std::string::String,
        ///When the event was recorded by the producer
        #[serde(rename = "recordedAt")]
        pub recorded_at: ::std::string::String,
        ///Schema version of the event envelope
        #[serde(rename = "schemaVersion")]
        pub schema_version: i64,
        ///Session ID for LLM usage events
        #[serde(
            rename = "sessionId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub session_id: ::std::option::Option<::std::string::String>,
        ///Specific producer instance identifier
        #[serde(
            rename = "sourceInstance",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub source_instance: ::std::option::Option<::std::string::String>,
        ///Service that produced the event
        #[serde(rename = "sourceService")]
        pub source_service: ::std::string::String,
        ///Status associated with the event when applicable
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub status: ::std::option::Option<::std::string::String>,
        ///Specific event type (e.g., dag.run.failed)
        #[serde(rename = "type")]
        pub type_: ::std::string::String,
        ///User ID associated with the event
        #[serde(
            rename = "userId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub user_id: ::std::option::Option<::std::string::String>,
    }
    ///Response containing centralized event log entries
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response containing centralized event log entries",
    ///  "type": "object",
    ///  "required": [
    ///    "entries"
    ///  ],
    ///  "properties": {
    ///    "entries": {
    ///      "description": "List of event log entries",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/EventLogEntry"
    ///      }
    ///    },
    ///    "nextCursor": {
    ///      "description": "Opaque cursor for loading the next page of older entries when using cursor pagination",
    ///      "type": "string"
    ///    },
    ///    "total": {
    ///      "description": "Total number of entries matching the filter when using compatibility offset pagination",
    ///      "type": "integer"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct EventLogsResponse {
        ///List of event log entries
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub entries: ::std::vec::Vec<EventLogEntry>,
        ///Opaque cursor for loading the next page of older entries when using cursor pagination
        #[serde(
            rename = "nextCursor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub next_cursor: ::std::option::Option<::std::string::String>,
        ///Total number of entries matching the filter when using compatibility offset pagination
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub total: ::std::option::Option<i64>,
    }
    ///`ExecuteDagBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "dagName": {
    ///      "description": "Optional DAG name override to use for the created dag-run",
    ///      "type": "string"
    ///    },
    ///    "dagRunId": {
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/DAGRunId"
    ///        },
    ///        {
    ///          "description": "Optional ID for the DAG-run, if not provided a new one will be generated"
    ///        }
    ///      ]
    ///    },
    ///    "labels": {
    ///      "$ref": "#/components/schemas/Labels"
    ///    },
    ///    "noReuse": {
    ///      "description": "If true, execute eligible build steps without reusing prior materializations",
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "params": {
    ///      "description": "Parameters to pass to the DAG-run in JSON format",
    ///      "type": "string"
    ///    },
    ///    "profile": {
    ///      "$ref": "#/components/schemas/RuntimeProfileOverride"
    ///    },
    ///    "singleton": {
    ///      "description": "If true, prevent starting if DAG is already running (returns 409 conflict)",
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "tags": {
    ///      "$ref": "#/components/schemas/Tags"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ExecuteDagBody {
        ///Optional DAG name override to use for the created dag-run
        #[serde(
            rename = "dagName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_name: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "dagRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_run_id: ::std::option::Option<DagRunId>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub labels: ::std::option::Option<Labels>,
        ///If true, execute eligible build steps without reusing prior materializations
        #[serde(rename = "noReuse", default)]
        pub no_reuse: bool,
        ///Parameters to pass to the DAG-run in JSON format
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub params: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub profile: ::std::option::Option<RuntimeProfileOverride>,
        ///If true, prevent starting if DAG is already running (returns 409 conflict)
        #[serde(default)]
        pub singleton: bool,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub tags: ::std::option::Option<Tags>,
    }
    impl ::std::default::Default for ExecuteDagBody {
        fn default() -> Self {
            Self {
                dag_name: Default::default(),
                dag_run_id: Default::default(),
                labels: Default::default(),
                no_reuse: Default::default(),
                params: Default::default(),
                profile: Default::default(),
                singleton: Default::default(),
                tags: Default::default(),
            }
        }
    }
    ///`ExecuteDagResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunId"
    ///  ],
    ///  "properties": {
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ExecuteDagResponse {
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
    }
    ///`ExecuteDagRunFromSpecBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "spec"
    ///  ],
    ///  "properties": {
    ///    "dagRunId": {
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/DAGRunId"
    ///        },
    ///        {
    ///          "description": "Optional ID for the DAG-run; if omitted, a new one is generated"
    ///        }
    ///      ]
    ///    },
    ///    "labels": {
    ///      "$ref": "#/components/schemas/Labels"
    ///    },
    ///    "name": {
    ///      "description": "Optional name to use when the spec omits a name",
    ///      "type": "string"
    ///    },
    ///    "noReuse": {
    ///      "description": "If true, execute eligible build steps without reusing prior materializations",
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "params": {
    ///      "description": "Parameters to pass to the DAG-run in JSON format",
    ///      "type": "string"
    ///    },
    ///    "profile": {
    ///      "$ref": "#/components/schemas/RuntimeProfileOverride"
    ///    },
    ///    "singleton": {
    ///      "description": "If true, prevent starting if a DAG with the same name is already running (returns 409)",
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "spec": {
    ///      "description": "DAG specification in YAML format",
    ///      "type": "string"
    ///    },
    ///    "tags": {
    ///      "$ref": "#/components/schemas/Tags"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ExecuteDagRunFromSpecBody {
        #[serde(
            rename = "dagRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_run_id: ::std::option::Option<DagRunId>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub labels: ::std::option::Option<Labels>,
        ///Optional name to use when the spec omits a name
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub name: ::std::option::Option<::std::string::String>,
        ///If true, execute eligible build steps without reusing prior materializations
        #[serde(rename = "noReuse", default)]
        pub no_reuse: bool,
        ///Parameters to pass to the DAG-run in JSON format
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub params: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub profile: ::std::option::Option<RuntimeProfileOverride>,
        ///If true, prevent starting if a DAG with the same name is already running (returns 409)
        #[serde(default)]
        pub singleton: bool,
        ///DAG specification in YAML format
        pub spec: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub tags: ::std::option::Option<Tags>,
    }
    ///`ExecuteDagRunFromSpecResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunId"
    ///  ],
    ///  "properties": {
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ExecuteDagRunFromSpecResponse {
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
    }
    ///Response object for getting details of a DAG
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response object for getting details of a DAG",
    ///  "type": "object",
    ///  "required": [
    ///    "errors",
    ///    "latestDAGRun",
    ///    "localDags",
    ///    "suspended"
    ///  ],
    ///  "properties": {
    ///    "dag": {
    ///      "$ref": "#/components/schemas/DAGDetails"
    ///    },
    ///    "editorHints": {
    ///      "$ref": "#/components/schemas/DAGEditorHints"
    ///    },
    ///    "errors": {
    ///      "description": "List of errors encountered during the request",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "latestDAGRun": {
    ///      "$ref": "#/components/schemas/DAGRunDetails"
    ///    },
    ///    "localDags": {
    ///      "description": "List of local DAGs that are part of this DAG",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/LocalDag"
    ///      }
    ///    },
    ///    "spec": {
    ///      "description": "The DAG specification in YAML format",
    ///      "type": "string"
    ///    },
    ///    "suspended": {
    ///      "description": "Whether the DAG is suspended",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct GetDagDetailsResponse {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub dag: ::std::option::Option<DagDetails>,
        #[serde(
            rename = "editorHints",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub editor_hints: ::std::option::Option<DagEditorHints>,
        ///List of errors encountered during the request
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub errors: ::std::vec::Vec<::std::string::String>,
        #[serde(rename = "latestDAGRun")]
        pub latest_dag_run: DagRunDetails,
        ///List of local DAGs that are part of this DAG
        #[serde(rename = "localDags")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub local_dags: ::std::vec::Vec<LocalDag>,
        ///The DAG specification in YAML format
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub spec: ::std::option::Option<::std::string::String>,
        ///Whether the DAG is suspended
        pub suspended: bool,
    }
    ///`GetDagRunDetailsResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunDetails"
    ///  ],
    ///  "properties": {
    ///    "dagRunDetails": {
    ///      "$ref": "#/components/schemas/DAGRunDetails"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct GetDagRunDetailsResponse {
        #[serde(rename = "dagRunDetails")]
        pub dag_run_details: DagRunDetails,
    }
    ///`GetDagSpecResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "errors",
    ///    "spec",
    ///    "valueReferenceNotices"
    ///  ],
    ///  "properties": {
    ///    "dag": {
    ///      "$ref": "#/components/schemas/DAGDetails"
    ///    },
    ///    "errors": {
    ///      "description": "List of errors in the spec",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "spec": {
    ///      "description": "The DAG spec in YAML format",
    ///      "type": "string"
    ///    },
    ///    "valueReferenceNotices": {
    ///      "description": "Passive value-reference notices produced while loading this spec. These notices are not persisted.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ValueReferenceNotice"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct GetDagSpecResponse {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub dag: ::std::option::Option<DagDetails>,
        ///List of errors in the spec
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub errors: ::std::vec::Vec<::std::string::String>,
        ///The DAG spec in YAML format
        pub spec: ::std::string::String,
        ///Passive value-reference notices produced while loading this spec. These notices are not persisted.
        #[serde(rename = "valueReferenceNotices")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub value_reference_notices: ::std::vec::Vec<ValueReferenceNotice>,
    }
    ///Configuration for event handlers in a DAG-run
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Configuration for event handlers in a DAG-run",
    ///  "type": "object",
    ///  "properties": {
    ///    "abort": {
    ///      "$ref": "#/components/schemas/Step"
    ///    },
    ///    "exit": {
    ///      "$ref": "#/components/schemas/Step"
    ///    },
    ///    "failure": {
    ///      "$ref": "#/components/schemas/Step"
    ///    },
    ///    "success": {
    ///      "$ref": "#/components/schemas/Step"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct HandlerOn {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub abort: ::std::option::Option<Step>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub exit: ::std::option::Option<Step>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub failure: ::std::option::Option<Step>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub success: ::std::option::Option<Step>,
    }
    impl ::std::default::Default for HandlerOn {
        fn default() -> Self {
            Self {
                abort: Default::default(),
                exit: Default::default(),
                failure: Default::default(),
                success: Default::default(),
            }
        }
    }
    ///Response object for the health check endpoint
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response object for the health check endpoint",
    ///  "type": "object",
    ///  "required": [
    ///    "status",
    ///    "timestamp",
    ///    "uptime",
    ///    "version"
    ///  ],
    ///  "properties": {
    ///    "status": {
    ///      "description": "Overall health status of the server",
    ///      "type": "string",
    ///      "enum": [
    ///        "healthy",
    ///        "unhealthy"
    ///      ]
    ///    },
    ///    "timestamp": {
    ///      "description": "Current server time",
    ///      "type": "string"
    ///    },
    ///    "uptime": {
    ///      "description": "Server uptime in seconds",
    ///      "type": "integer"
    ///    },
    ///    "version": {
    ///      "description": "Current version of the server",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct HealthResponse {
        ///Overall health status of the server
        pub status: HealthResponseStatus,
        ///Current server time
        pub timestamp: ::std::string::String,
        ///Server uptime in seconds
        pub uptime: i64,
        ///Current version of the server
        pub version: ::std::string::String,
    }
    ///Overall health status of the server
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Overall health status of the server",
    ///  "type": "string",
    ///  "enum": [
    ///    "healthy",
    ///    "unhealthy"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum HealthResponseStatus {
        #[serde(rename = "healthy")]
        Healthy,
        #[serde(rename = "unhealthy")]
        Unhealthy,
    }
    impl ::std::fmt::Display for HealthResponseStatus {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Healthy => f.write_str("healthy"),
                Self::Unhealthy => f.write_str("unhealthy"),
            }
        }
    }
    impl ::std::str::FromStr for HealthResponseStatus {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "healthy" => Ok(Self::Healthy),
                "unhealthy" => Ok(Self::Unhealthy),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for HealthResponseStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for HealthResponseStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for HealthResponseStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Result of completing or confirming one human task
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Result of completing or confirming one human task",
    ///  "type": "object",
    ///  "required": [
    ///    "alreadyCompleted",
    ///    "dagName",
    ///    "dagRunId",
    ///    "queued",
    ///    "remainingWaitingSteps",
    ///    "stepId"
    ///  ],
    ///  "properties": {
    ///    "alreadyCompleted": {
    ///      "type": "boolean"
    ///    },
    ///    "dagName": {
    ///      "$ref": "#/components/schemas/DAGName"
    ///    },
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    },
    ///    "queued": {
    ///      "description": "Whether this request durably queued the DAG-run retry",
    ///      "type": "boolean"
    ///    },
    ///    "remainingWaitingSteps": {
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "stepId": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct HumanTaskCompletionResponse {
        #[serde(rename = "alreadyCompleted")]
        pub already_completed: bool,
        #[serde(rename = "dagName")]
        pub dag_name: DagName,
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
        ///Whether this request durably queued the DAG-run retry
        pub queued: bool,
        #[serde(rename = "remainingWaitingSteps")]
        pub remaining_waiting_steps: u64,
        #[serde(rename = "stepId")]
        pub step_id: ::std::string::String,
    }
    ///Resolved human-task instructions and optional normalized input form
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Resolved human-task instructions and optional normalized input form",
    ///  "type": "object",
    ///  "required": [
    ///    "prompt"
    ///  ],
    ///  "properties": {
    ///    "form": {
    ///      "description": "Normalized flat JSON Schema for typed completion input",
    ///      "type": "object",
    ///      "additionalProperties": true
    ///    },
    ///    "prompt": {
    ///      "description": "Instructions displayed to the operator. Run details contain the resolved, secret-masked snapshot.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct HumanTaskConfig {
        ///Normalized flat JSON Schema for typed completion input
        #[serde(default, skip_serializing_if = "::serde_json::Map::is_empty")]
        pub form: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
        ///Instructions displayed to the operator. Run details contain the resolved, secret-masked snapshot.
        pub prompt: ::std::string::String,
    }
    ///Typed human-task completion input. An empty object acknowledges a task without a form.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Typed human-task completion input. An empty object acknowledges a task without a form.",
    ///  "type": "object",
    ///  "additionalProperties": true
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(transparent)]
    pub struct HumanTaskInput(
        pub ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    );
    impl ::std::ops::Deref for HumanTaskInput {
        type Target = ::serde_json::Map<::std::string::String, ::serde_json::Value>;
        fn deref(
            &self,
        ) -> &::serde_json::Map<::std::string::String, ::serde_json::Value> {
            &self.0
        }
    }
    impl ::std::convert::From<HumanTaskInput>
    for ::serde_json::Map<::std::string::String, ::serde_json::Value> {
        fn from(value: HumanTaskInput) -> Self {
            value.0
        }
    }
    impl ::std::convert::From<
        ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    > for HumanTaskInput {
        fn from(
            value: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
        ) -> Self {
            Self(value)
        }
    }
    ///Result of queueing a completed human-task retry
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Result of queueing a completed human-task retry",
    ///  "type": "object",
    ///  "required": [
    ///    "dagName",
    ///    "dagRunId",
    ///    "queued"
    ///  ],
    ///  "properties": {
    ///    "dagName": {
    ///      "$ref": "#/components/schemas/DAGName"
    ///    },
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    },
    ///    "queued": {
    ///      "description": "Whether this request durably queued the DAG-run retry",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct HumanTaskResumeResponse {
        #[serde(rename = "dagName")]
        pub dag_name: DagName,
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
        ///Whether this request durably queued the DAG-run retry
        pub queued: bool,
    }
    ///Public PagerDuty provider details
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Public PagerDuty provider details",
    ///  "type": "object",
    ///  "required": [
    ///    "routingKeyConfigured"
    ///  ],
    ///  "properties": {
    ///    "routingKeyConfigured": {
    ///      "description": "Whether a PagerDuty routing key is configured",
    ///      "type": "boolean"
    ///    },
    ///    "routingKeyPreview": {
    ///      "description": "Redacted routing key preview",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentPagerDutyProvider {
        ///Whether a PagerDuty routing key is configured
        #[serde(rename = "routingKeyConfigured")]
        pub routing_key_configured: bool,
        ///Redacted routing key preview
        #[serde(
            rename = "routingKeyPreview",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub routing_key_preview: ::std::option::Option<::std::string::String>,
    }
    ///PagerDuty Events API v2 provider input. The routing key is encrypted at rest.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "PagerDuty Events API v2 provider input. The routing key is encrypted at rest.",
    ///  "type": "object",
    ///  "properties": {
    ///    "clearRoutingKey": {
    ///      "description": "Clear the stored routing key.",
    ///      "type": "boolean"
    ///    },
    ///    "routingKey": {
    ///      "description": "PagerDuty Events API v2 routing key. Omit on updates to preserve the existing key.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentPagerDutyProviderInput {
        ///Clear the stored routing key.
        #[serde(
            rename = "clearRoutingKey",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub clear_routing_key: ::std::option::Option<bool>,
        ///PagerDuty Events API v2 routing key. Omit on updates to preserve the existing key.
        #[serde(
            rename = "routingKey",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub routing_key: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for IncidentPagerDutyProviderInput {
        fn default() -> Self {
            Self {
                clear_routing_key: Default::default(),
                routing_key: Default::default(),
            }
        }
    }
    ///PagerDuty incident provider input
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "PagerDuty incident provider input",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "name",
    ///    "pagerDuty",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "enabled": {
    ///      "description": "Whether this provider can receive incident events",
    ///      "type": "boolean"
    ///    },
    ///    "name": {
    ///      "description": "Human-readable provider name",
    ///      "type": "string"
    ///    },
    ///    "pagerDuty": {
    ///      "$ref": "#/components/schemas/IncidentPagerDutyProviderInput"
    ///    },
    ///    "type": {
    ///      "type": "string",
    ///      "enum": [
    ///        "pagerduty"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentPagerDutyProviderInputEnvelope {
        ///Whether this provider can receive incident events
        pub enabled: bool,
        ///Human-readable provider name
        pub name: ::std::string::String,
        #[serde(rename = "pagerDuty")]
        pub pager_duty: IncidentPagerDutyProviderInput,
        #[serde(rename = "type")]
        pub type_: IncidentPagerDutyProviderInputEnvelopeType,
    }
    ///`IncidentPagerDutyProviderInputEnvelopeType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "pagerduty"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum IncidentPagerDutyProviderInputEnvelopeType {
        #[serde(rename = "pagerduty")]
        Pagerduty,
    }
    impl ::std::fmt::Display for IncidentPagerDutyProviderInputEnvelopeType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Pagerduty => f.write_str("pagerduty"),
            }
        }
    }
    impl ::std::str::FromStr for IncidentPagerDutyProviderInputEnvelopeType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "pagerduty" => Ok(Self::Pagerduty),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for IncidentPagerDutyProviderInputEnvelopeType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for IncidentPagerDutyProviderInputEnvelopeType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for IncidentPagerDutyProviderInputEnvelopeType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Incident route
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Incident route",
    ///  "type": "object",
    ///  "required": [
    ///    "dedupKeyTemplate",
    ///    "descriptionTemplate",
    ///    "enabled",
    ///    "id",
    ///    "messageTemplate",
    ///    "providerId",
    ///    "resolveOnRecovery",
    ///    "severity"
    ///  ],
    ///  "properties": {
    ///    "dedupKeyTemplate": {
    ///      "description": "Deprecated and ignored. Dagu generates stable provider incident keys.",
    ///      "type": "string"
    ///    },
    ///    "descriptionTemplate": {
    ///      "description": "Template for the provider incident description/details",
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this route is enabled",
    ///      "type": "boolean"
    ///    },
    ///    "id": {
    ///      "description": "Stable route ID",
    ///      "type": "string"
    ///    },
    ///    "messageTemplate": {
    ///      "description": "Template for the provider incident summary/message",
    ///      "type": "string"
    ///    },
    ///    "providerId": {
    ///      "description": "Incident provider ID",
    ///      "type": "string"
    ///    },
    ///    "resolveOnRecovery": {
    ///      "description": "Deprecated. Dagu resolves saved open incidents on recovery.",
    ///      "type": "boolean"
    ///    },
    ///    "severity": {
    ///      "$ref": "#/components/schemas/IncidentSeverity"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentPolicy {
        ///Deprecated and ignored. Dagu generates stable provider incident keys.
        #[serde(rename = "dedupKeyTemplate")]
        pub dedup_key_template: ::std::string::String,
        ///Template for the provider incident description/details
        #[serde(rename = "descriptionTemplate")]
        pub description_template: ::std::string::String,
        ///Whether this route is enabled
        pub enabled: bool,
        ///Stable route ID
        pub id: ::std::string::String,
        ///Template for the provider incident summary/message
        #[serde(rename = "messageTemplate")]
        pub message_template: ::std::string::String,
        ///Incident provider ID
        #[serde(rename = "providerId")]
        pub provider_id: ::std::string::String,
        ///Deprecated. Dagu resolves saved open incidents on recovery.
        #[serde(rename = "resolveOnRecovery")]
        pub resolve_on_recovery: bool,
        pub severity: IncidentSeverity,
    }
    ///Incident route input. A route opens an incident on final DAG failure and Dagu resolves the saved open incident on later success.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Incident route input. A route opens an incident on final DAG failure and Dagu resolves the saved open incident on later success.",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "providerId",
    ///    "severity"
    ///  ],
    ///  "properties": {
    ///    "dedupKeyTemplate": {
    ///      "description": "Deprecated and ignored. Dagu generates stable provider incident keys.",
    ///      "type": "string"
    ///    },
    ///    "descriptionTemplate": {
    ///      "description": "Template for the provider incident description/details.",
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this route is enabled. Normal incident routing sends true when the route exists.",
    ///      "type": "boolean"
    ///    },
    ///    "id": {
    ///      "description": "Stable route ID. Omit when adding a route.",
    ///      "type": "string"
    ///    },
    ///    "messageTemplate": {
    ///      "description": "Template for the provider incident summary/message.",
    ///      "type": "string"
    ///    },
    ///    "providerId": {
    ///      "description": "Incident provider ID",
    ///      "type": "string"
    ///    },
    ///    "resolveOnRecovery": {
    ///      "description": "Deprecated. Dagu resolves saved open incidents on recovery.",
    ///      "type": "boolean"
    ///    },
    ///    "severity": {
    ///      "$ref": "#/components/schemas/IncidentSeverity"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentPolicyInput {
        ///Deprecated and ignored. Dagu generates stable provider incident keys.
        #[serde(
            rename = "dedupKeyTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dedup_key_template: ::std::option::Option<::std::string::String>,
        ///Template for the provider incident description/details.
        #[serde(
            rename = "descriptionTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub description_template: ::std::option::Option<::std::string::String>,
        ///Whether this route is enabled. Normal incident routing sends true when the route exists.
        pub enabled: bool,
        ///Stable route ID. Omit when adding a route.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub id: ::std::option::Option<::std::string::String>,
        ///Template for the provider incident summary/message.
        #[serde(
            rename = "messageTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub message_template: ::std::option::Option<::std::string::String>,
        ///Incident provider ID
        #[serde(rename = "providerId")]
        pub provider_id: ::std::string::String,
        ///Deprecated. Dagu resolves saved open incidents on recovery.
        #[serde(
            rename = "resolveOnRecovery",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub resolve_on_recovery: ::std::option::Option<bool>,
        pub severity: IncidentSeverity,
    }
    ///Incident routing scope
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Incident routing scope",
    ///  "type": "string",
    ///  "enum": [
    ///    "global",
    ///    "workspace",
    ///    "dag"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum IncidentPolicyScope {
        #[serde(rename = "global")]
        Global,
        #[serde(rename = "workspace")]
        Workspace,
        #[serde(rename = "dag")]
        Dag,
    }
    impl ::std::fmt::Display for IncidentPolicyScope {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Global => f.write_str("global"),
                Self::Workspace => f.write_str("workspace"),
                Self::Dag => f.write_str("dag"),
            }
        }
    }
    impl ::std::str::FromStr for IncidentPolicyScope {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "global" => Ok(Self::Global),
                "workspace" => Ok(Self::Workspace),
                "dag" => Ok(Self::Dag),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for IncidentPolicyScope {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for IncidentPolicyScope {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for IncidentPolicyScope {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Incident routing for a global, workspace, or DAG scope
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Incident routing for a global, workspace, or DAG scope",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "inheritParent",
    ///    "policies",
    ///    "scope"
    ///  ],
    ///  "properties": {
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "dagName": {
    ///      "description": "DAG name for DAG-scoped routing",
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this scope can open new incidents",
    ///      "type": "boolean"
    ///    },
    ///    "id": {
    ///      "description": "Stable routing ID, present after routing is saved",
    ///      "type": "string"
    ///    },
    ///    "inheritParent": {
    ///      "description": "For workspace and DAG routing, true means inherit the parent scope instead of using local routes",
    ///      "type": "boolean"
    ///    },
    ///    "policies": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/IncidentPolicy"
    ///      }
    ///    },
    ///    "scope": {
    ///      "$ref": "#/components/schemas/IncidentPolicyScope"
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    },
    ///    "updatedBy": {
    ///      "description": "User ID that last updated routing",
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "description": "Workspace name for workspace-scoped routing",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentPolicySet {
        #[serde(
            rename = "createdAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub created_at: ::std::option::Option<::std::string::String>,
        ///DAG name for DAG-scoped routing
        #[serde(
            rename = "dagName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_name: ::std::option::Option<::std::string::String>,
        ///Whether this scope can open new incidents
        pub enabled: bool,
        ///Stable routing ID, present after routing is saved
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub id: ::std::option::Option<::std::string::String>,
        ///For workspace and DAG routing, true means inherit the parent scope instead of using local routes
        #[serde(rename = "inheritParent")]
        pub inherit_parent: bool,
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub policies: ::std::vec::Vec<IncidentPolicy>,
        pub scope: IncidentPolicyScope,
        #[serde(
            rename = "updatedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_at: ::std::option::Option<::std::string::String>,
        ///User ID that last updated routing
        #[serde(
            rename = "updatedBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_by: ::std::option::Option<::std::string::String>,
        ///Workspace name for workspace-scoped routing
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///Replacement incident routing for a global, workspace, or DAG scope
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Replacement incident routing for a global, workspace, or DAG scope",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "inheritParent",
    ///    "policies"
    ///  ],
    ///  "properties": {
    ///    "enabled": {
    ///      "description": "Whether this scope can open new incidents",
    ///      "type": "boolean"
    ///    },
    ///    "inheritParent": {
    ///      "description": "For workspace and DAG routing, true means inherit the parent scope instead of using local routes. Ignored for global routing.",
    ///      "type": "boolean"
    ///    },
    ///    "policies": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/IncidentPolicyInput"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentPolicySetInput {
        ///Whether this scope can open new incidents
        pub enabled: bool,
        ///For workspace and DAG routing, true means inherit the parent scope instead of using local routes. Ignored for global routing.
        #[serde(rename = "inheritParent")]
        pub inherit_parent: bool,
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub policies: ::std::vec::Vec<IncidentPolicyInput>,
    }
    ///Incident routing scopes
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Incident routing scopes",
    ///  "type": "object",
    ///  "required": [
    ///    "policySets"
    ///  ],
    ///  "properties": {
    ///    "policySets": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/IncidentPolicySet"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentPolicySetListResponse {
        #[serde(rename = "policySets")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub policy_sets: ::std::vec::Vec<IncidentPolicySet>,
    }
    ///Incident provider. Secrets are never returned.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Incident provider. Secrets are never returned.",
    ///  "type": "object",
    ///  "required": [
    ///    "createdAt",
    ///    "enabled",
    ///    "id",
    ///    "name",
    ///    "type",
    ///    "updatedAt"
    ///  ],
    ///  "properties": {
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this provider can receive incident events",
    ///      "type": "boolean"
    ///    },
    ///    "id": {
    ///      "description": "Stable provider ID",
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "description": "Human-readable provider name",
    ///      "type": "string"
    ///    },
    ///    "pagerDuty": {
    ///      "$ref": "#/components/schemas/IncidentPagerDutyProvider"
    ///    },
    ///    "solarWinds": {
    ///      "$ref": "#/components/schemas/IncidentSolarWindsProvider"
    ///    },
    ///    "type": {
    ///      "$ref": "#/components/schemas/IncidentProviderType"
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    },
    ///    "updatedBy": {
    ///      "description": "User ID that last updated the provider",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentProvider {
        #[serde(rename = "createdAt")]
        pub created_at: ::std::string::String,
        ///Whether this provider can receive incident events
        pub enabled: bool,
        ///Stable provider ID
        pub id: ::std::string::String,
        ///Human-readable provider name
        pub name: ::std::string::String,
        #[serde(
            rename = "pagerDuty",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub pager_duty: ::std::option::Option<IncidentPagerDutyProvider>,
        #[serde(
            rename = "solarWinds",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub solar_winds: ::std::option::Option<IncidentSolarWindsProvider>,
        #[serde(rename = "type")]
        pub type_: IncidentProviderType,
        #[serde(rename = "updatedAt")]
        pub updated_at: ::std::string::String,
        ///User ID that last updated the provider
        #[serde(
            rename = "updatedBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_by: ::std::option::Option<::std::string::String>,
    }
    ///Incident provider input
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Incident provider input",
    ///  "oneOf": [
    ///    {
    ///      "$ref": "#/components/schemas/IncidentPagerDutyProviderInputEnvelope"
    ///    },
    ///    {
    ///      "$ref": "#/components/schemas/IncidentSolarWindsProviderInputEnvelope"
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(untagged)]
    pub enum IncidentProviderInput {
        PagerDutyProviderInputEnvelope(IncidentPagerDutyProviderInputEnvelope),
        SolarWindsProviderInputEnvelope(IncidentSolarWindsProviderInputEnvelope),
    }
    impl ::std::convert::From<IncidentPagerDutyProviderInputEnvelope>
    for IncidentProviderInput {
        fn from(value: IncidentPagerDutyProviderInputEnvelope) -> Self {
            Self::PagerDutyProviderInputEnvelope(value)
        }
    }
    impl ::std::convert::From<IncidentSolarWindsProviderInputEnvelope>
    for IncidentProviderInput {
        fn from(value: IncidentSolarWindsProviderInputEnvelope) -> Self {
            Self::SolarWindsProviderInputEnvelope(value)
        }
    }
    ///Incident providers
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Incident providers",
    ///  "type": "object",
    ///  "required": [
    ///    "providers"
    ///  ],
    ///  "properties": {
    ///    "providers": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/IncidentProvider"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentProviderListResponse {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub providers: ::std::vec::Vec<IncidentProvider>,
    }
    ///Incident provider type
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Incident provider type",
    ///  "type": "string",
    ///  "enum": [
    ///    "pagerduty",
    ///    "solarwinds_incident_response"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum IncidentProviderType {
        #[serde(rename = "pagerduty")]
        Pagerduty,
        #[serde(rename = "solarwinds_incident_response")]
        SolarwindsIncidentResponse,
    }
    impl ::std::fmt::Display for IncidentProviderType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Pagerduty => f.write_str("pagerduty"),
                Self::SolarwindsIncidentResponse => {
                    f.write_str("solarwinds_incident_response")
                }
            }
        }
    }
    impl ::std::str::FromStr for IncidentProviderType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "pagerduty" => Ok(Self::Pagerduty),
                "solarwinds_incident_response" => Ok(Self::SolarwindsIncidentResponse),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for IncidentProviderType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for IncidentProviderType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for IncidentProviderType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Incident severity
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Incident severity",
    ///  "type": "string",
    ///  "enum": [
    ///    "critical",
    ///    "error",
    ///    "warning",
    ///    "info"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum IncidentSeverity {
        #[serde(rename = "critical")]
        Critical,
        #[serde(rename = "error")]
        Error,
        #[serde(rename = "warning")]
        Warning,
        #[serde(rename = "info")]
        Info,
    }
    impl ::std::fmt::Display for IncidentSeverity {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Critical => f.write_str("critical"),
                Self::Error => f.write_str("error"),
                Self::Warning => f.write_str("warning"),
                Self::Info => f.write_str("info"),
            }
        }
    }
    impl ::std::str::FromStr for IncidentSeverity {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "critical" => Ok(Self::Critical),
                "error" => Ok(Self::Error),
                "warning" => Ok(Self::Warning),
                "info" => Ok(Self::Info),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for IncidentSeverity {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for IncidentSeverity {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for IncidentSeverity {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Public SolarWinds Incident Response provider details
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Public SolarWinds Incident Response provider details",
    ///  "type": "object",
    ///  "required": [
    ///    "webhookUrlConfigured"
    ///  ],
    ///  "properties": {
    ///    "allowInsecureHttp": {
    ///      "description": "Whether this provider allows plain HTTP webhook URLs",
    ///      "type": "boolean"
    ///    },
    ///    "allowPrivateNetwork": {
    ///      "description": "Whether this provider allows loopback or private network webhook targets",
    ///      "type": "boolean"
    ///    },
    ///    "webhookUrlConfigured": {
    ///      "description": "Whether an incoming webhook URL is configured",
    ///      "type": "boolean"
    ///    },
    ///    "webhookUrlPreview": {
    ///      "description": "Redacted webhook URL preview",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentSolarWindsProvider {
        ///Whether this provider allows plain HTTP webhook URLs
        #[serde(
            rename = "allowInsecureHttp",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub allow_insecure_http: ::std::option::Option<bool>,
        ///Whether this provider allows loopback or private network webhook targets
        #[serde(
            rename = "allowPrivateNetwork",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub allow_private_network: ::std::option::Option<bool>,
        ///Whether an incoming webhook URL is configured
        #[serde(rename = "webhookUrlConfigured")]
        pub webhook_url_configured: bool,
        ///Redacted webhook URL preview
        #[serde(
            rename = "webhookUrlPreview",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub webhook_url_preview: ::std::option::Option<::std::string::String>,
    }
    ///SolarWinds Incident Response incoming webhook provider input. The webhook URL is encrypted at rest.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "SolarWinds Incident Response incoming webhook provider input. The webhook URL is encrypted at rest.",
    ///  "type": "object",
    ///  "properties": {
    ///    "allowInsecureHttp": {
    ///      "description": "Allow plain HTTP webhook URLs. Disabled by default.",
    ///      "type": "boolean"
    ///    },
    ///    "allowPrivateNetwork": {
    ///      "description": "Allow loopback or private network webhook targets. Disabled by default.",
    ///      "type": "boolean"
    ///    },
    ///    "clearWebhookUrl": {
    ///      "description": "Clear the stored webhook URL.",
    ///      "type": "boolean"
    ///    },
    ///    "webhookUrl": {
    ///      "description": "Incoming webhook URL. Omit on updates to preserve the existing URL.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentSolarWindsProviderInput {
        ///Allow plain HTTP webhook URLs. Disabled by default.
        #[serde(
            rename = "allowInsecureHttp",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub allow_insecure_http: ::std::option::Option<bool>,
        ///Allow loopback or private network webhook targets. Disabled by default.
        #[serde(
            rename = "allowPrivateNetwork",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub allow_private_network: ::std::option::Option<bool>,
        ///Clear the stored webhook URL.
        #[serde(
            rename = "clearWebhookUrl",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub clear_webhook_url: ::std::option::Option<bool>,
        ///Incoming webhook URL. Omit on updates to preserve the existing URL.
        #[serde(
            rename = "webhookUrl",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub webhook_url: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for IncidentSolarWindsProviderInput {
        fn default() -> Self {
            Self {
                allow_insecure_http: Default::default(),
                allow_private_network: Default::default(),
                clear_webhook_url: Default::default(),
                webhook_url: Default::default(),
            }
        }
    }
    ///SolarWinds Incident Response provider input
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "SolarWinds Incident Response provider input",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "name",
    ///    "solarWinds",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "enabled": {
    ///      "description": "Whether this provider can receive incident events",
    ///      "type": "boolean"
    ///    },
    ///    "name": {
    ///      "description": "Human-readable provider name",
    ///      "type": "string"
    ///    },
    ///    "solarWinds": {
    ///      "$ref": "#/components/schemas/IncidentSolarWindsProviderInput"
    ///    },
    ///    "type": {
    ///      "type": "string",
    ///      "enum": [
    ///        "solarwinds_incident_response"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct IncidentSolarWindsProviderInputEnvelope {
        ///Whether this provider can receive incident events
        pub enabled: bool,
        ///Human-readable provider name
        pub name: ::std::string::String,
        #[serde(rename = "solarWinds")]
        pub solar_winds: IncidentSolarWindsProviderInput,
        #[serde(rename = "type")]
        pub type_: IncidentSolarWindsProviderInputEnvelopeType,
    }
    ///`IncidentSolarWindsProviderInputEnvelopeType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "solarwinds_incident_response"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum IncidentSolarWindsProviderInputEnvelopeType {
        #[serde(rename = "solarwinds_incident_response")]
        SolarwindsIncidentResponse,
    }
    impl ::std::fmt::Display for IncidentSolarWindsProviderInputEnvelopeType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::SolarwindsIncidentResponse => {
                    f.write_str("solarwinds_incident_response")
                }
            }
        }
    }
    impl ::std::str::FromStr for IncidentSolarWindsProviderInputEnvelopeType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "solarwinds_incident_response" => Ok(Self::SolarwindsIncidentResponse),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for IncidentSolarWindsProviderInputEnvelopeType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for IncidentSolarWindsProviderInputEnvelopeType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for IncidentSolarWindsProviderInputEnvelopeType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Resolved editor hint for an inherited custom action
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Resolved editor hint for an inherited custom action",
    ///  "type": "object",
    ///  "required": [
    ///    "inputSchema",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "description": {
    ///      "description": "Optional custom action description",
    ///      "type": "string"
    ///    },
    ///    "inputSchema": {
    ///      "description": "Resolved JSON Schema object used to validate and document with input",
    ///      "type": "object",
    ///      "additionalProperties": true
    ///    },
    ///    "name": {
    ///      "description": "Custom action name",
    ///      "type": "string"
    ///    },
    ///    "outputSchema": {
    ///      "description": "Resolved JSON Schema object used to validate stdout JSON output",
    ///      "type": "object",
    ///      "additionalProperties": true
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct InheritedCustomActionHint {
        ///Optional custom action description
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        ///Resolved JSON Schema object used to validate and document with input
        #[serde(rename = "inputSchema")]
        pub input_schema: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
        ///Custom action name
        pub name: ::std::string::String,
        ///Resolved JSON Schema object used to validate stdout JSON output
        #[serde(
            rename = "outputSchema",
            default,
            skip_serializing_if = "::serde_json::Map::is_empty"
        )]
        pub output_schema: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    }
    ///Resolved editor hint for an inherited deprecated legacy execution definition
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Resolved editor hint for an inherited deprecated legacy execution definition",
    ///  "type": "object",
    ///  "required": [
    ///    "inputSchema",
    ///    "name",
    ///    "targetType"
    ///  ],
    ///  "properties": {
    ///    "description": {
    ///      "description": "Optional deprecated legacy execution definition description",
    ///      "type": "string"
    ///    },
    ///    "inputSchema": {
    ///      "description": "Resolved JSON Schema object used to validate and document with input",
    ///      "type": "object",
    ///      "additionalProperties": true
    ///    },
    ///    "name": {
    ///      "description": "Deprecated legacy execution definition name",
    ///      "type": "string"
    ///    },
    ///    "outputSchema": {
    ///      "description": "Resolved JSON Schema object used to validate stdout JSON output",
    ///      "type": "object",
    ///      "additionalProperties": true
    ///    },
    ///    "targetType": {
    ///      "description": "Builtin executor type that the deprecated legacy execution definition expands to",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct InheritedLegacyDefinitionHint {
        ///Optional deprecated legacy execution definition description
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        ///Resolved JSON Schema object used to validate and document with input
        #[serde(rename = "inputSchema")]
        pub input_schema: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
        ///Deprecated legacy execution definition name
        pub name: ::std::string::String,
        ///Resolved JSON Schema object used to validate stdout JSON output
        #[serde(
            rename = "outputSchema",
            default,
            skip_serializing_if = "::serde_json::Map::is_empty"
        )]
        pub output_schema: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
        ///Builtin executor type that the deprecated legacy execution definition expands to
        #[serde(rename = "targetType")]
        pub target_type: ::std::string::String,
    }
    ///Non-selectable inherited profile layer name.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Non-selectable inherited profile layer name.",
    ///  "type": "string"
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    #[serde(transparent)]
    pub struct InheritedRuntimeProfileName(pub ::std::string::String);
    impl ::std::ops::Deref for InheritedRuntimeProfileName {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<InheritedRuntimeProfileName> for ::std::string::String {
        fn from(value: InheritedRuntimeProfileName) -> Self {
            value.0
        }
    }
    impl ::std::convert::From<::std::string::String> for InheritedRuntimeProfileName {
        fn from(value: ::std::string::String) -> Self {
            Self(value)
        }
    }
    impl ::std::str::FromStr for InheritedRuntimeProfileName {
        type Err = ::std::convert::Infallible;
        fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::fmt::Display for InheritedRuntimeProfileName {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            self.0.fmt(f)
        }
    }
    ///`InheritedRuntimeProfileResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "entries",
    ///    "name",
    ///    "protected",
    ///    "scope",
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "defaultProfile": {
    ///      "$ref": "#/components/schemas/RuntimeProfileName"
    ///    },
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "entries": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/RuntimeProfileEntryResponse"
    ///      }
    ///    },
    ///    "id": {
    ///      "description": "Persistent record ID. Omitted until the inherited layer is first saved.",
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "$ref": "#/components/schemas/InheritedRuntimeProfileName"
    ///    },
    ///    "protected": {
    ///      "type": "boolean"
    ///    },
    ///    "scope": {
    ///      "$ref": "#/components/schemas/InheritedRuntimeProfileScope"
    ///    },
    ///    "status": {
    ///      "$ref": "#/components/schemas/RuntimeProfileStatus"
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "$ref": "#/components/schemas/WorkspaceName"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct InheritedRuntimeProfileResponse {
        #[serde(
            rename = "createdAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub created_at: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "defaultProfile",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub default_profile: ::std::option::Option<RuntimeProfileName>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub entries: ::std::vec::Vec<RuntimeProfileEntryResponse>,
        ///Persistent record ID. Omitted until the inherited layer is first saved.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub id: ::std::option::Option<::std::string::String>,
        pub name: InheritedRuntimeProfileName,
        pub protected: bool,
        pub scope: InheritedRuntimeProfileScope,
        pub status: RuntimeProfileStatus,
        #[serde(
            rename = "updatedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_at: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<WorkspaceName>,
    }
    ///`InheritedRuntimeProfileScope`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "global",
    ///    "workspace"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum InheritedRuntimeProfileScope {
        #[serde(rename = "global")]
        Global,
        #[serde(rename = "workspace")]
        Workspace,
    }
    impl ::std::fmt::Display for InheritedRuntimeProfileScope {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Global => f.write_str("global"),
                Self::Workspace => f.write_str("workspace"),
            }
        }
    }
    impl ::std::str::FromStr for InheritedRuntimeProfileScope {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "global" => Ok(Self::Global),
                "workspace" => Ok(Self::Workspace),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for InheritedRuntimeProfileScope {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for InheritedRuntimeProfileScope {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for InheritedRuntimeProfileScope {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Additional labels to apply to the DAG-run (format: key=value or key-only). Merged with labels defined in the DAG spec. Mutually exclusive with deprecated `tags`; the server returns HTTP 400 if both are set.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Additional labels to apply to the DAG-run (format: key=value or key-only). Merged with labels defined in the DAG spec. Mutually exclusive with deprecated `tags`; the server returns HTTP 400 if both are set.",
    ///  "type": "array",
    ///  "items": {
    ///    "type": "string"
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(transparent)]
    pub struct Labels(pub ::std::vec::Vec<::std::string::String>);
    impl ::std::ops::Deref for Labels {
        type Target = ::std::vec::Vec<::std::string::String>;
        fn deref(&self) -> &::std::vec::Vec<::std::string::String> {
            &self.0
        }
    }
    impl ::std::convert::From<Labels> for ::std::vec::Vec<::std::string::String> {
        fn from(value: Labels) -> Self {
            value.0
        }
    }
    impl ::std::convert::From<::std::vec::Vec<::std::string::String>> for Labels {
        fn from(value: ::std::vec::Vec<::std::string::String>) -> Self {
            Self(value)
        }
    }
    ///Public status of the current Dagu license
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Public status of the current Dagu license",
    ///  "type": "object",
    ///  "required": [
    ///    "community",
    ///    "error",
    ///    "expiry",
    ///    "features",
    ///    "graceEndsAt",
    ///    "gracePeriod",
    ///    "plan",
    ///    "source",
    ///    "valid",
    ///    "warningCode"
    ///  ],
    ///  "properties": {
    ///    "community": {
    ///      "description": "Whether no license claims are loaded",
    ///      "type": "boolean"
    ///    },
    ///    "error": {
    ///      "description": "User-facing explanation when a configured license is unusable",
    ///      "type": "string"
    ///    },
    ///    "expiry": {
    ///      "description": "License expiration timestamp, or empty for a perpetual or absent license",
    ///      "type": "string"
    ///    },
    ///    "features": {
    ///      "description": "Feature claims included in the license",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "graceEndsAt": {
    ///      "description": "Grace-period end timestamp, or empty when the license has no expiration",
    ///      "type": "string"
    ///    },
    ///    "gracePeriod": {
    ///      "description": "Whether the license is expired but still inside its grace period",
    ///      "type": "boolean"
    ///    },
    ///    "plan": {
    ///      "description": "License plan name",
    ///      "type": "string"
    ///    },
    ///    "source": {
    ///      "description": "Public license source category",
    ///      "type": "string"
    ///    },
    ///    "valid": {
    ///      "description": "Whether the loaded license token has not expired",
    ///      "type": "boolean"
    ///    },
    ///    "warningCode": {
    ///      "description": "Warning code included in the license token",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct LicenseStatusResponse {
        ///Whether no license claims are loaded
        pub community: bool,
        ///User-facing explanation when a configured license is unusable
        pub error: ::std::string::String,
        ///License expiration timestamp, or empty for a perpetual or absent license
        pub expiry: ::std::string::String,
        ///Feature claims included in the license
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub features: ::std::vec::Vec<::std::string::String>,
        ///Grace-period end timestamp, or empty when the license has no expiration
        #[serde(rename = "graceEndsAt")]
        pub grace_ends_at: ::std::string::String,
        ///Whether the license is expired but still inside its grace period
        #[serde(rename = "gracePeriod")]
        pub grace_period: bool,
        ///License plan name
        pub plan: ::std::string::String,
        ///Public license source category
        pub source: ::std::string::String,
        ///Whether the loaded license token has not expired
        pub valid: bool,
        ///Warning code included in the license token
        #[serde(rename = "warningCode")]
        pub warning_code: ::std::string::String,
    }
    ///`ListDaGsOrder`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "default": "asc",
    ///  "type": "string",
    ///  "enum": [
    ///    "asc",
    ///    "desc"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ListDaGsOrder {
        #[serde(rename = "asc")]
        Asc,
        #[serde(rename = "desc")]
        Desc,
    }
    impl ::std::fmt::Display for ListDaGsOrder {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Asc => f.write_str("asc"),
                Self::Desc => f.write_str("desc"),
            }
        }
    }
    impl ::std::str::FromStr for ListDaGsOrder {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "asc" => Ok(Self::Asc),
                "desc" => Ok(Self::Desc),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ListDaGsOrder {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ListDaGsOrder {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ListDaGsOrder {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::default::Default for ListDaGsOrder {
        fn default() -> Self {
            ListDaGsOrder::Asc
        }
    }
    ///`ListDaGsResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "dags",
    ///    "errors",
    ///    "pagination"
    ///  ],
    ///  "properties": {
    ///    "dags": {
    ///      "description": "List of DAG definitions with their status and metadata",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/DAGFile"
    ///      }
    ///    },
    ///    "errors": {
    ///      "description": "List of errors encountered during the request",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "pagination": {
    ///      "$ref": "#/components/schemas/Pagination"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ListDaGsResponse {
        ///List of DAG definitions with their status and metadata
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub dags: ::std::vec::Vec<DagFile>,
        ///List of errors encountered during the request
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub errors: ::std::vec::Vec<::std::string::String>,
        pub pagination: Pagination,
    }
    ///`ListDaGsSort`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "default": "name",
    ///  "type": "string",
    ///  "enum": [
    ///    "name",
    ///    "nextRun"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ListDaGsSort {
        #[serde(rename = "name")]
        Name,
        #[serde(rename = "nextRun")]
        NextRun,
    }
    impl ::std::fmt::Display for ListDaGsSort {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Name => f.write_str("name"),
                Self::NextRun => f.write_str("nextRun"),
            }
        }
    }
    impl ::std::str::FromStr for ListDaGsSort {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "name" => Ok(Self::Name),
                "nextRun" => Ok(Self::NextRun),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ListDaGsSort {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ListDaGsSort {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ListDaGsSort {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::default::Default for ListDaGsSort {
        fn default() -> Self {
            ListDaGsSort::Name
        }
    }
    ///`ListDaGsWorkspace`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "maxLength": 64,
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct ListDaGsWorkspace(::std::string::String);
    impl ::std::ops::Deref for ListDaGsWorkspace {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<ListDaGsWorkspace> for ::std::string::String {
        fn from(value: ListDaGsWorkspace) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for ListDaGsWorkspace {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 64usize {
                return Err("longer than 64 characters".into());
            }
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for ListDaGsWorkspace {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ListDaGsWorkspace {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ListDaGsWorkspace {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for ListDaGsWorkspace {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///`ListDagRunsWorkspace`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "maxLength": 64,
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct ListDagRunsWorkspace(::std::string::String);
    impl ::std::ops::Deref for ListDagRunsWorkspace {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<ListDagRunsWorkspace> for ::std::string::String {
        fn from(value: ListDagRunsWorkspace) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for ListDagRunsWorkspace {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 64usize {
                return Err("longer than 64 characters".into());
            }
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for ListDagRunsWorkspace {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ListDagRunsWorkspace {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ListDagRunsWorkspace {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for ListDagRunsWorkspace {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Response object for listing all labels
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response object for listing all labels",
    ///  "type": "object",
    ///  "required": [
    ///    "errors",
    ///    "labels"
    ///  ],
    ///  "properties": {
    ///    "errors": {
    ///      "description": "List of errors encountered during the request",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "labels": {
    ///      "description": "List of unique labels",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ListLabelResponse {
        ///List of errors encountered during the request
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub errors: ::std::vec::Vec<::std::string::String>,
        ///List of unique labels
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub labels: ::std::vec::Vec<::std::string::String>,
    }
    ///Deprecated response object for listing all labels
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Deprecated response object for listing all labels",
    ///  "deprecated": true,
    ///  "type": "object",
    ///  "required": [
    ///    "errors",
    ///    "tags"
    ///  ],
    ///  "properties": {
    ///    "errors": {
    ///      "description": "List of errors encountered during the request",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "tags": {
    ///      "description": "List of unique labels",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ListTagResponse {
        ///List of errors encountered during the request
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub errors: ::std::vec::Vec<::std::string::String>,
        ///List of unique labels
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub tags: ::std::vec::Vec<::std::string::String>,
    }
    ///`LocalDag`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "errors",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "dag": {
    ///      "$ref": "#/components/schemas/DAGDetails"
    ///    },
    ///    "errors": {
    ///      "description": "List of errors encountered while processing the local DAG",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "name": {
    ///      "description": "Name of the local DAG",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct LocalDag {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub dag: ::std::option::Option<DagDetails>,
        ///List of errors encountered while processing the local DAG
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub errors: ::std::vec::Vec<::std::string::String>,
        ///Name of the local DAG
        pub name: ::std::string::String,
    }
    ///Log information for the execution
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Log information for the execution",
    ///  "type": "object",
    ///  "required": [
    ///    "content"
    ///  ],
    ///  "properties": {
    ///    "content": {
    ///      "description": "Log content",
    ///      "type": "string"
    ///    },
    ///    "hasMore": {
    ///      "description": "Whether there are more lines available",
    ///      "type": "boolean"
    ///    },
    ///    "isEstimate": {
    ///      "description": "Whether the line count is an estimate",
    ///      "type": "boolean"
    ///    },
    ///    "lineCount": {
    ///      "description": "Number of lines returned",
    ///      "type": "integer"
    ///    },
    ///    "totalLines": {
    ///      "description": "Total number of lines in the log file",
    ///      "type": "integer"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct Log {
        ///Log content
        pub content: ::std::string::String,
        ///Whether there are more lines available
        #[serde(
            rename = "hasMore",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub has_more: ::std::option::Option<bool>,
        ///Whether the line count is an estimate
        #[serde(
            rename = "isEstimate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub is_estimate: ::std::option::Option<bool>,
        ///Number of lines returned
        #[serde(
            rename = "lineCount",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub line_count: ::std::option::Option<i64>,
        ///Total number of lines in the log file
        #[serde(
            rename = "totalLines",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub total_lines: ::std::option::Option<i64>,
    }
    ///Request body for user login
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for user login",
    ///  "type": "object",
    ///  "required": [
    ///    "password",
    ///    "username"
    ///  ],
    ///  "properties": {
    ///    "password": {
    ///      "description": "User's password",
    ///      "type": "string",
    ///      "minLength": 1
    ///    },
    ///    "username": {
    ///      "description": "User's username",
    ///      "type": "string",
    ///      "minLength": 1
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct LoginRequest {
        ///User's password
        pub password: LoginRequestPassword,
        ///User's username
        pub username: LoginRequestUsername,
    }
    ///User's password
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "User's password",
    ///  "type": "string",
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct LoginRequestPassword(::std::string::String);
    impl ::std::ops::Deref for LoginRequestPassword {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<LoginRequestPassword> for ::std::string::String {
        fn from(value: LoginRequestPassword) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for LoginRequestPassword {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for LoginRequestPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for LoginRequestPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for LoginRequestPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for LoginRequestPassword {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///User's username
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "User's username",
    ///  "type": "string",
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct LoginRequestUsername(::std::string::String);
    impl ::std::ops::Deref for LoginRequestUsername {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<LoginRequestUsername> for ::std::string::String {
        fn from(value: LoginRequestUsername) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for LoginRequestUsername {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for LoginRequestUsername {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for LoginRequestUsername {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for LoginRequestUsername {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for LoginRequestUsername {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Response containing authentication token
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response containing authentication token",
    ///  "type": "object",
    ///  "required": [
    ///    "expiresAt",
    ///    "token",
    ///    "user"
    ///  ],
    ///  "properties": {
    ///    "expiresAt": {
    ///      "description": "Token expiration timestamp",
    ///      "type": "string"
    ///    },
    ///    "token": {
    ///      "description": "JWT authentication token",
    ///      "type": "string"
    ///    },
    ///    "user": {
    ///      "$ref": "#/components/schemas/User"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct LoginResponse {
        ///Token expiration timestamp
        #[serde(rename = "expiresAt")]
        pub expires_at: ::std::string::String,
        ///JWT authentication token
        pub token: ::std::string::String,
        pub user: User,
    }
    ///`MetricPoint`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "timestamp",
    ///    "value"
    ///  ],
    ///  "properties": {
    ///    "timestamp": {
    ///      "description": "Unix timestamp",
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "value": {
    ///      "type": "number",
    ///      "format": "double"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct MetricPoint {
        ///Unix timestamp
        pub timestamp: i64,
        pub value: f64,
    }
    ///Status of an individual step within a DAG-run
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Status of an individual step within a DAG-run",
    ///  "type": "object",
    ///  "required": [
    ///    "doneCount",
    ///    "finishedAt",
    ///    "retryCount",
    ///    "startedAt",
    ///    "status",
    ///    "statusLabel",
    ///    "stderr",
    ///    "stdout",
    ///    "step"
    ///  ],
    ///  "properties": {
    ///    "agentSession": {
    ///      "$ref": "#/components/schemas/AgentSession"
    ///    },
    ///    "approvalInputs": {
    ///      "description": "Key-value inputs provided during approval",
    ///      "type": "object",
    ///      "additionalProperties": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "approvalIteration": {
    ///      "description": "Number of times this step has been pushed back for re-execution",
    ///      "type": "integer"
    ///    },
    ///    "approvedAt": {
    ///      "description": "RFC3339 timestamp when the step was approved",
    ///      "type": "string"
    ///    },
    ///    "approvedBy": {
    ///      "description": "Username of who approved the step",
    ///      "type": "string"
    ///    },
    ///    "approvedById": {
    ///      "description": "ID of the subject that approved the step",
    ///      "type": "string"
    ///    },
    ///    "build": {
    ///      "$ref": "#/components/schemas/BuildExecution"
    ///    },
    ///    "doneCount": {
    ///      "description": "Number of successful completions for repeating steps",
    ///      "type": "integer"
    ///    },
    ///    "error": {
    ///      "description": "Error message if the step failed",
    ///      "type": "string"
    ///    },
    ///    "finishedAt": {
    ///      "description": "RFC3339 timestamp when the step finished",
    ///      "type": "string"
    ///    },
    ///    "humanTaskCompletedBy": {
    ///      "description": "Name of the subject that completed the human task",
    ///      "type": "string"
    ///    },
    ///    "humanTaskCompletedById": {
    ///      "description": "ID of the subject that completed the human task; local CLI IDs use the os:<uid> form",
    ///      "type": "string"
    ///    },
    ///    "pushBackHistory": {
    ///      "description": "Chronological push-back history for this step",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/PushBackHistoryEntry"
    ///      }
    ///    },
    ///    "pushBackInputs": {
    ///      "description": "Key-value inputs from the last push-back, injected as environment variables during re-execution",
    ///      "type": "object",
    ///      "additionalProperties": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "rejectedAt": {
    ///      "description": "RFC3339 timestamp when the step was rejected",
    ///      "type": "string"
    ///    },
    ///    "rejectedBy": {
    ///      "description": "Username of who rejected the step",
    ///      "type": "string"
    ///    },
    ///    "rejectedById": {
    ///      "description": "ID of the subject that rejected the step",
    ///      "type": "string"
    ///    },
    ///    "rejectionReason": {
    ///      "description": "Optional reason for rejection",
    ///      "type": "string"
    ///    },
    ///    "retryCount": {
    ///      "description": "Number of retry attempts made for this step",
    ///      "type": "integer"
    ///    },
    ///    "startedAt": {
    ///      "description": "RFC3339 timestamp when the step started",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "$ref": "#/components/schemas/NodeStatus"
    ///    },
    ///    "statusLabel": {
    ///      "$ref": "#/components/schemas/NodeStatusLabel"
    ///    },
    ///    "stderr": {
    ///      "description": "Path to the standard error log file for this step",
    ///      "type": "string"
    ///    },
    ///    "stdout": {
    ///      "description": "Path to the standard output log file for this step",
    ///      "type": "string"
    ///    },
    ///    "step": {
    ///      "$ref": "#/components/schemas/Step"
    ///    },
    ///    "subRuns": {
    ///      "description": "List of sub DAG-runs associated with this step",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/SubDAGRun"
    ///      }
    ///    },
    ///    "subRunsRepeated": {
    ///      "description": "List of repeated sub DAG-runs when using repeatPolicy",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/SubDAGRun"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct Node {
        #[serde(
            rename = "agentSession",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub agent_session: ::std::option::Option<AgentSession>,
        ///Key-value inputs provided during approval
        #[serde(
            rename = "approvalInputs",
            default,
            skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
        )]
        pub approval_inputs: ::std::collections::HashMap<
            ::std::string::String,
            ::std::string::String,
        >,
        ///Number of times this step has been pushed back for re-execution
        #[serde(
            rename = "approvalIteration",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub approval_iteration: ::std::option::Option<i64>,
        ///RFC3339 timestamp when the step was approved
        #[serde(
            rename = "approvedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub approved_at: ::std::option::Option<::std::string::String>,
        ///Username of who approved the step
        #[serde(
            rename = "approvedBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub approved_by: ::std::option::Option<::std::string::String>,
        ///ID of the subject that approved the step
        #[serde(
            rename = "approvedById",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub approved_by_id: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub build: ::std::option::Option<BuildExecution>,
        ///Number of successful completions for repeating steps
        #[serde(rename = "doneCount")]
        pub done_count: i64,
        ///Error message if the step failed
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub error: ::std::option::Option<::std::string::String>,
        ///RFC3339 timestamp when the step finished
        #[serde(rename = "finishedAt")]
        pub finished_at: ::std::string::String,
        ///Name of the subject that completed the human task
        #[serde(
            rename = "humanTaskCompletedBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub human_task_completed_by: ::std::option::Option<::std::string::String>,
        ///ID of the subject that completed the human task; local CLI IDs use the os:<uid> form
        #[serde(
            rename = "humanTaskCompletedById",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub human_task_completed_by_id: ::std::option::Option<::std::string::String>,
        ///Chronological push-back history for this step
        #[serde(
            rename = "pushBackHistory",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub push_back_history: ::std::vec::Vec<PushBackHistoryEntry>,
        ///Key-value inputs from the last push-back, injected as environment variables during re-execution
        #[serde(
            rename = "pushBackInputs",
            default,
            skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
        )]
        pub push_back_inputs: ::std::collections::HashMap<
            ::std::string::String,
            ::std::string::String,
        >,
        ///RFC3339 timestamp when the step was rejected
        #[serde(
            rename = "rejectedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub rejected_at: ::std::option::Option<::std::string::String>,
        ///Username of who rejected the step
        #[serde(
            rename = "rejectedBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub rejected_by: ::std::option::Option<::std::string::String>,
        ///ID of the subject that rejected the step
        #[serde(
            rename = "rejectedById",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub rejected_by_id: ::std::option::Option<::std::string::String>,
        ///Optional reason for rejection
        #[serde(
            rename = "rejectionReason",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub rejection_reason: ::std::option::Option<::std::string::String>,
        ///Number of retry attempts made for this step
        #[serde(rename = "retryCount")]
        pub retry_count: i64,
        ///RFC3339 timestamp when the step started
        #[serde(rename = "startedAt")]
        pub started_at: ::std::string::String,
        pub status: NodeStatus,
        #[serde(rename = "statusLabel")]
        pub status_label: NodeStatusLabel,
        ///Path to the standard error log file for this step
        pub stderr: ::std::string::String,
        ///Path to the standard output log file for this step
        pub stdout: ::std::string::String,
        pub step: Step,
        ///List of sub DAG-runs associated with this step
        #[serde(
            rename = "subRuns",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub sub_runs: ::std::vec::Vec<SubDagRun>,
        ///List of repeated sub DAG-runs when using repeatPolicy
        #[serde(
            rename = "subRunsRepeated",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub sub_runs_repeated: ::std::vec::Vec<SubDagRun>,
    }
    /**Numeric status code indicating current node state:
0: "Not started"
1: "Running"
2: "Failed"
3: "Aborted"
4: "Success"
5: "Skipped"
6: "Partial Success"
7: "Waiting for manual action"
8: "Rejected"
9: "Retrying"
*/
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Numeric status code indicating current node state:\n0: \"Not started\"\n1: \"Running\"\n2: \"Failed\"\n3: \"Aborted\"\n4: \"Success\"\n5: \"Skipped\"\n6: \"Partial Success\"\n7: \"Waiting for manual action\"\n8: \"Rejected\"\n9: \"Retrying\"\n",
    ///  "type": "integer",
    ///  "enum": [
    ///    0,
    ///    1,
    ///    2,
    ///    3,
    ///    4,
    ///    5,
    ///    6,
    ///    7,
    ///    8,
    ///    9
    ///  ],
    ///  "x-enum-varnames": [
    ///    "NotStarted",
    ///    "Running",
    ///    "Failed",
    ///    "Aborted",
    ///    "Success",
    ///    "Skipped",
    ///    "PartialSuccess",
    ///    "Waiting",
    ///    "Rejected",
    ///    "Retrying"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug)]
    #[serde(transparent)]
    pub struct NodeStatus(i64);
    impl ::std::ops::Deref for NodeStatus {
        type Target = i64;
        fn deref(&self) -> &i64 {
            &self.0
        }
    }
    impl ::std::convert::From<NodeStatus> for i64 {
        fn from(value: NodeStatus) -> Self {
            value.0
        }
    }
    impl ::std::convert::TryFrom<i64> for NodeStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: i64,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if ![0_i64, 1_i64, 2_i64, 3_i64, 4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64]
                .contains(&value)
            {
                Err("invalid value".into())
            } else {
                Ok(Self(value))
            }
        }
    }
    impl<'de> ::serde::Deserialize<'de> for NodeStatus {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            Self::try_from(<i64>::deserialize(deserializer)?)
                .map_err(|e| { <D::Error as ::serde::de::Error>::custom(e.to_string()) })
        }
    }
    ///Human-readable status description for the node
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Human-readable status description for the node",
    ///  "type": "string",
    ///  "enum": [
    ///    "not_started",
    ///    "running",
    ///    "failed",
    ///    "aborted",
    ///    "succeeded",
    ///    "skipped",
    ///    "partially_succeeded",
    ///    "waiting",
    ///    "rejected",
    ///    "retrying"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum NodeStatusLabel {
        #[serde(rename = "not_started")]
        NotStarted,
        #[serde(rename = "running")]
        Running,
        #[serde(rename = "failed")]
        Failed,
        #[serde(rename = "aborted")]
        Aborted,
        #[serde(rename = "succeeded")]
        Succeeded,
        #[serde(rename = "skipped")]
        Skipped,
        #[serde(rename = "partially_succeeded")]
        PartiallySucceeded,
        #[serde(rename = "waiting")]
        Waiting,
        #[serde(rename = "rejected")]
        Rejected,
        #[serde(rename = "retrying")]
        Retrying,
    }
    impl ::std::fmt::Display for NodeStatusLabel {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::NotStarted => f.write_str("not_started"),
                Self::Running => f.write_str("running"),
                Self::Failed => f.write_str("failed"),
                Self::Aborted => f.write_str("aborted"),
                Self::Succeeded => f.write_str("succeeded"),
                Self::Skipped => f.write_str("skipped"),
                Self::PartiallySucceeded => f.write_str("partially_succeeded"),
                Self::Waiting => f.write_str("waiting"),
                Self::Rejected => f.write_str("rejected"),
                Self::Retrying => f.write_str("retrying"),
            }
        }
    }
    impl ::std::str::FromStr for NodeStatusLabel {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "not_started" => Ok(Self::NotStarted),
                "running" => Ok(Self::Running),
                "failed" => Ok(Self::Failed),
                "aborted" => Ok(Self::Aborted),
                "succeeded" => Ok(Self::Succeeded),
                "skipped" => Ok(Self::Skipped),
                "partially_succeeded" => Ok(Self::PartiallySucceeded),
                "waiting" => Ok(Self::Waiting),
                "rejected" => Ok(Self::Rejected),
                "retrying" => Ok(Self::Retrying),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for NodeStatusLabel {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for NodeStatusLabel {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for NodeStatusLabel {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Notification channel. Secrets are never returned.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Notification channel. Secrets are never returned.",
    ///  "type": "object",
    ///  "required": [
    ///    "createdAt",
    ///    "enabled",
    ///    "id",
    ///    "name",
    ///    "type",
    ///    "updatedAt"
    ///  ],
    ///  "properties": {
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "email": {
    ///      "$ref": "#/components/schemas/NotificationEmailTarget"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this channel can receive notifications",
    ///      "type": "boolean"
    ///    },
    ///    "id": {
    ///      "description": "Stable channel ID",
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "description": "Human-readable channel name",
    ///      "type": "string"
    ///    },
    ///    "slack": {
    ///      "$ref": "#/components/schemas/NotificationSlackTarget"
    ///    },
    ///    "teams": {
    ///      "$ref": "#/components/schemas/NotificationTeamsTarget"
    ///    },
    ///    "telegram": {
    ///      "$ref": "#/components/schemas/NotificationTelegramTarget"
    ///    },
    ///    "type": {
    ///      "$ref": "#/components/schemas/NotificationProviderType"
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    },
    ///    "updatedBy": {
    ///      "description": "User ID that last updated the channel",
    ///      "type": "string"
    ///    },
    ///    "webhook": {
    ///      "$ref": "#/components/schemas/NotificationWebhookTarget"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationChannel {
        #[serde(rename = "createdAt")]
        pub created_at: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub email: ::std::option::Option<NotificationEmailTarget>,
        ///Whether this channel can receive notifications
        pub enabled: bool,
        ///Stable channel ID
        pub id: ::std::string::String,
        ///Human-readable channel name
        pub name: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub slack: ::std::option::Option<NotificationSlackTarget>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub teams: ::std::option::Option<NotificationTeamsTarget>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub telegram: ::std::option::Option<NotificationTelegramTarget>,
        #[serde(rename = "type")]
        pub type_: NotificationProviderType,
        #[serde(rename = "updatedAt")]
        pub updated_at: ::std::string::String,
        ///User ID that last updated the channel
        #[serde(
            rename = "updatedBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_by: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub webhook: ::std::option::Option<NotificationWebhookTarget>,
    }
    ///Notification channel input
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Notification channel input",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "name",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "email": {
    ///      "$ref": "#/components/schemas/NotificationEmailTarget"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this channel can receive notifications",
    ///      "type": "boolean"
    ///    },
    ///    "name": {
    ///      "description": "Human-readable channel name",
    ///      "type": "string"
    ///    },
    ///    "slack": {
    ///      "$ref": "#/components/schemas/NotificationSlackTargetInput"
    ///    },
    ///    "teams": {
    ///      "$ref": "#/components/schemas/NotificationTeamsTargetInput"
    ///    },
    ///    "telegram": {
    ///      "$ref": "#/components/schemas/NotificationTelegramTargetInput"
    ///    },
    ///    "type": {
    ///      "$ref": "#/components/schemas/NotificationProviderType"
    ///    },
    ///    "webhook": {
    ///      "$ref": "#/components/schemas/NotificationWebhookTargetInput"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationChannelInput {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub email: ::std::option::Option<NotificationEmailTarget>,
        ///Whether this channel can receive notifications
        pub enabled: bool,
        ///Human-readable channel name
        pub name: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub slack: ::std::option::Option<NotificationSlackTargetInput>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub teams: ::std::option::Option<NotificationTeamsTargetInput>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub telegram: ::std::option::Option<NotificationTelegramTargetInput>,
        #[serde(rename = "type")]
        pub type_: NotificationProviderType,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub webhook: ::std::option::Option<NotificationWebhookTargetInput>,
    }
    ///Notification channels
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Notification channels",
    ///  "type": "object",
    ///  "required": [
    ///    "channels"
    ///  ],
    ///  "properties": {
    ///    "channels": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationChannel"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationChannelListResponse {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub channels: ::std::vec::Vec<NotificationChannel>,
    }
    ///Email notification target. SMTP transport is configured in workspace notification settings.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Email notification target. SMTP transport is configured in workspace notification settings.",
    ///  "type": "object",
    ///  "required": [
    ///    "to"
    ///  ],
    ///  "properties": {
    ///    "attachLogs": {
    ///      "description": "Attach DAG and step logs when available",
    ///      "type": "boolean"
    ///    },
    ///    "bcc": {
    ///      "description": "BCC recipients",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "bodyTemplate": {
    ///      "description": "Optional email body template. When omitted, Dagu sends the default notification body.",
    ///      "type": "string"
    ///    },
    ///    "cc": {
    ///      "description": "CC recipients",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "from": {
    ///      "description": "Sender address. Defaults to the workspace SMTP sender.",
    ///      "type": "string"
    ///    },
    ///    "subjectPrefix": {
    ///      "description": "Subject prefix. Defaults to [DAGU].",
    ///      "type": "string"
    ///    },
    ///    "subjectTemplate": {
    ///      "description": "Optional email subject template. When set, it replaces the generated subject.",
    ///      "type": "string"
    ///    },
    ///    "to": {
    ///      "description": "Primary recipients",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationEmailTarget {
        ///Attach DAG and step logs when available
        #[serde(
            rename = "attachLogs",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub attach_logs: ::std::option::Option<bool>,
        ///BCC recipients
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub bcc: ::std::vec::Vec<::std::string::String>,
        ///Optional email body template. When omitted, Dagu sends the default notification body.
        #[serde(
            rename = "bodyTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub body_template: ::std::option::Option<::std::string::String>,
        ///CC recipients
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub cc: ::std::vec::Vec<::std::string::String>,
        ///Sender address. Defaults to the workspace SMTP sender.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub from: ::std::option::Option<::std::string::String>,
        ///Subject prefix. Defaults to [DAGU].
        #[serde(
            rename = "subjectPrefix",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub subject_prefix: ::std::option::Option<::std::string::String>,
        ///Optional email subject template. When set, it replaces the generated subject.
        #[serde(
            rename = "subjectTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub subject_template: ::std::option::Option<::std::string::String>,
        ///Primary recipients
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub to: ::std::vec::Vec<::std::string::String>,
    }
    ///DAG run event that can trigger server-side notifications. Rules configured for dag.run.succeeded also match dag.run.partially_succeeded; rules configured only for dag.run.partially_succeeded do not match clean successes.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "DAG run event that can trigger server-side notifications. Rules configured for dag.run.succeeded also match dag.run.partially_succeeded; rules configured only for dag.run.partially_succeeded do not match clean successes.",
    ///  "type": "string",
    ///  "enum": [
    ///    "dag.run.waiting",
    ///    "dag.run.succeeded",
    ///    "dag.run.partially_succeeded",
    ///    "dag.run.failed",
    ///    "dag.run.aborted",
    ///    "dag.run.rejected"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum NotificationEventType {
        #[serde(rename = "dag.run.waiting")]
        DagRunWaiting,
        #[serde(rename = "dag.run.succeeded")]
        DagRunSucceeded,
        #[serde(rename = "dag.run.partially_succeeded")]
        DagRunPartiallySucceeded,
        #[serde(rename = "dag.run.failed")]
        DagRunFailed,
        #[serde(rename = "dag.run.aborted")]
        DagRunAborted,
        #[serde(rename = "dag.run.rejected")]
        DagRunRejected,
    }
    impl ::std::fmt::Display for NotificationEventType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::DagRunWaiting => f.write_str("dag.run.waiting"),
                Self::DagRunSucceeded => f.write_str("dag.run.succeeded"),
                Self::DagRunPartiallySucceeded => {
                    f.write_str("dag.run.partially_succeeded")
                }
                Self::DagRunFailed => f.write_str("dag.run.failed"),
                Self::DagRunAborted => f.write_str("dag.run.aborted"),
                Self::DagRunRejected => f.write_str("dag.run.rejected"),
            }
        }
    }
    impl ::std::str::FromStr for NotificationEventType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "dag.run.waiting" => Ok(Self::DagRunWaiting),
                "dag.run.succeeded" => Ok(Self::DagRunSucceeded),
                "dag.run.partially_succeeded" => Ok(Self::DagRunPartiallySucceeded),
                "dag.run.failed" => Ok(Self::DagRunFailed),
                "dag.run.aborted" => Ok(Self::DagRunAborted),
                "dag.run.rejected" => Ok(Self::DagRunRejected),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for NotificationEventType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for NotificationEventType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for NotificationEventType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Notification delivery provider
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Notification delivery provider",
    ///  "type": "string",
    ///  "enum": [
    ///    "email",
    ///    "webhook",
    ///    "slack",
    ///    "telegram",
    ///    "teams"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum NotificationProviderType {
        #[serde(rename = "email")]
        Email,
        #[serde(rename = "webhook")]
        Webhook,
        #[serde(rename = "slack")]
        Slack,
        #[serde(rename = "telegram")]
        Telegram,
        #[serde(rename = "teams")]
        Teams,
    }
    impl ::std::fmt::Display for NotificationProviderType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Email => f.write_str("email"),
                Self::Webhook => f.write_str("webhook"),
                Self::Slack => f.write_str("slack"),
                Self::Telegram => f.write_str("telegram"),
                Self::Teams => f.write_str("teams"),
            }
        }
    }
    impl ::std::str::FromStr for NotificationProviderType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "email" => Ok(Self::Email),
                "webhook" => Ok(Self::Webhook),
                "slack" => Ok(Self::Slack),
                "telegram" => Ok(Self::Telegram),
                "teams" => Ok(Self::Teams),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for NotificationProviderType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for NotificationProviderType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for NotificationProviderType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Route from notification events to a notification channel
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Route from notification events to a notification channel",
    ///  "type": "object",
    ///  "required": [
    ///    "channelId",
    ///    "enabled",
    ///    "id"
    ///  ],
    ///  "properties": {
    ///    "channelId": {
    ///      "description": "Notification channel ID",
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this route receives notifications",
    ///      "type": "boolean"
    ///    },
    ///    "events": {
    ///      "description": "Events delivered by this route. Empty is treated as operational defaults for backward compatibility.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationEventType"
    ///      }
    ///    },
    ///    "id": {
    ///      "description": "Stable route ID",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationRoute {
        ///Notification channel ID
        #[serde(rename = "channelId")]
        pub channel_id: ::std::string::String,
        ///Whether this route receives notifications
        pub enabled: bool,
        ///Events delivered by this route. Empty is treated as operational defaults for backward compatibility.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub events: ::std::vec::Vec<NotificationEventType>,
        ///Stable route ID
        pub id: ::std::string::String,
    }
    ///Route from notification events to a notification channel
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Route from notification events to a notification channel",
    ///  "type": "object",
    ///  "required": [
    ///    "channelId",
    ///    "enabled"
    ///  ],
    ///  "properties": {
    ///    "channelId": {
    ///      "description": "Notification channel ID",
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this route receives notifications",
    ///      "type": "boolean"
    ///    },
    ///    "events": {
    ///      "description": "Events delivered by this route. Omit only for backward compatibility; new clients should send an explicit event list.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationEventType"
    ///      }
    ///    },
    ///    "id": {
    ///      "description": "Stable route ID. Omit when adding a route.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationRouteInput {
        ///Notification channel ID
        #[serde(rename = "channelId")]
        pub channel_id: ::std::string::String,
        ///Whether this route receives notifications
        pub enabled: bool,
        ///Events delivered by this route. Omit only for backward compatibility; new clients should send an explicit event list.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub events: ::std::vec::Vec<NotificationEventType>,
        ///Stable route ID. Omit when adding a route.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub id: ::std::option::Option<::std::string::String>,
    }
    ///Notification route scope
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Notification route scope",
    ///  "type": "string",
    ///  "enum": [
    ///    "global",
    ///    "workspace"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum NotificationRouteScope {
        #[serde(rename = "global")]
        Global,
        #[serde(rename = "workspace")]
        Workspace,
    }
    impl ::std::fmt::Display for NotificationRouteScope {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Global => f.write_str("global"),
                Self::Workspace => f.write_str("workspace"),
            }
        }
    }
    impl ::std::str::FromStr for NotificationRouteScope {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "global" => Ok(Self::Global),
                "workspace" => Ok(Self::Workspace),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for NotificationRouteScope {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for NotificationRouteScope {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for NotificationRouteScope {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Notification routes for a global or workspace scope
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Notification routes for a global or workspace scope",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "inheritGlobal",
    ///    "routes",
    ///    "scope"
    ///  ],
    ///  "properties": {
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this route set can deliver notifications",
    ///      "type": "boolean"
    ///    },
    ///    "id": {
    ///      "description": "Stable route set ID, present after the route set is saved",
    ///      "type": "string"
    ///    },
    ///    "inheritGlobal": {
    ///      "description": "For workspace route sets, true means inherit Global instead of using workspace routes",
    ///      "type": "boolean"
    ///    },
    ///    "routes": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationRoute"
    ///      }
    ///    },
    ///    "scope": {
    ///      "$ref": "#/components/schemas/NotificationRouteScope"
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    },
    ///    "updatedBy": {
    ///      "description": "User ID that last updated the route set",
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "description": "Workspace name for workspace-scoped route sets",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationRouteSet {
        #[serde(
            rename = "createdAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub created_at: ::std::option::Option<::std::string::String>,
        ///Whether this route set can deliver notifications
        pub enabled: bool,
        ///Stable route set ID, present after the route set is saved
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub id: ::std::option::Option<::std::string::String>,
        ///For workspace route sets, true means inherit Global instead of using workspace routes
        #[serde(rename = "inheritGlobal")]
        pub inherit_global: bool,
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub routes: ::std::vec::Vec<NotificationRoute>,
        pub scope: NotificationRouteScope,
        #[serde(
            rename = "updatedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_at: ::std::option::Option<::std::string::String>,
        ///User ID that last updated the route set
        #[serde(
            rename = "updatedBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_by: ::std::option::Option<::std::string::String>,
        ///Workspace name for workspace-scoped route sets
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///Replacement route set for a global or workspace notification scope
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Replacement route set for a global or workspace notification scope",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "inheritGlobal",
    ///    "routes"
    ///  ],
    ///  "properties": {
    ///    "enabled": {
    ///      "description": "Whether this route set can deliver notifications",
    ///      "type": "boolean"
    ///    },
    ///    "inheritGlobal": {
    ///      "description": "For workspace route sets, true means inherit Global instead of using workspace routes. Ignored for global route sets.",
    ///      "type": "boolean"
    ///    },
    ///    "routes": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationRouteInput"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationRouteSetInput {
        ///Whether this route set can deliver notifications
        pub enabled: bool,
        ///For workspace route sets, true means inherit Global instead of using workspace routes. Ignored for global route sets.
        #[serde(rename = "inheritGlobal")]
        pub inherit_global: bool,
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub routes: ::std::vec::Vec<NotificationRouteInput>,
    }
    ///Notification route sets
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Notification route sets",
    ///  "type": "object",
    ///  "required": [
    ///    "routeSets"
    ///  ],
    ///  "properties": {
    ///    "routeSets": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationRouteSet"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationRouteSetListResponse {
        #[serde(rename = "routeSets")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub route_sets: ::std::vec::Vec<NotificationRouteSet>,
    }
    ///Public Slack target details
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Public Slack target details",
    ///  "type": "object",
    ///  "required": [
    ///    "webhookUrlConfigured"
    ///  ],
    ///  "properties": {
    ///    "messageTemplate": {
    ///      "description": "Optional Slack message template. When omitted, Dagu sends the default notification text.",
    ///      "type": "string"
    ///    },
    ///    "webhookUrlConfigured": {
    ///      "description": "Whether a Slack incoming webhook URL is configured",
    ///      "type": "boolean"
    ///    },
    ///    "webhookUrlPreview": {
    ///      "description": "Redacted Slack webhook URL preview",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationSlackTarget {
        ///Optional Slack message template. When omitted, Dagu sends the default notification text.
        #[serde(
            rename = "messageTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub message_template: ::std::option::Option<::std::string::String>,
        ///Whether a Slack incoming webhook URL is configured
        #[serde(rename = "webhookUrlConfigured")]
        pub webhook_url_configured: bool,
        ///Redacted Slack webhook URL preview
        #[serde(
            rename = "webhookUrlPreview",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub webhook_url_preview: ::std::option::Option<::std::string::String>,
    }
    ///Slack incoming webhook target input. Values are encrypted at rest.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Slack incoming webhook target input. Values are encrypted at rest.",
    ///  "type": "object",
    ///  "properties": {
    ///    "messageTemplate": {
    ///      "description": "Optional Slack message template. When omitted, Dagu sends the default notification text.",
    ///      "type": "string"
    ///    },
    ///    "webhookUrl": {
    ///      "description": "Slack incoming webhook URL. Omit on updates to preserve the existing URL.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationSlackTargetInput {
        ///Optional Slack message template. When omitted, Dagu sends the default notification text.
        #[serde(
            rename = "messageTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub message_template: ::std::option::Option<::std::string::String>,
        ///Slack incoming webhook URL. Omit on updates to preserve the existing URL.
        #[serde(
            rename = "webhookUrl",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub webhook_url: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for NotificationSlackTargetInput {
        fn default() -> Self {
            Self {
                message_template: Default::default(),
                webhook_url: Default::default(),
            }
        }
    }
    ///Public workspace SMTP transport settings. The SMTP password is never returned.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Public workspace SMTP transport settings. The SMTP password is never returned.",
    ///  "type": "object",
    ///  "required": [
    ///    "passwordConfigured"
    ///  ],
    ///  "properties": {
    ///    "from": {
    ///      "description": "Default sender address for notification email channels",
    ///      "type": "string"
    ///    },
    ///    "host": {
    ///      "description": "SMTP server host",
    ///      "type": "string"
    ///    },
    ///    "oauth": {
    ///      "$ref": "#/components/schemas/NotificationSMTPOAuthSettings"
    ///    },
    ///    "passwordConfigured": {
    ///      "description": "Whether an SMTP password is configured",
    ///      "type": "boolean"
    ///    },
    ///    "port": {
    ///      "description": "SMTP server port",
    ///      "type": "string"
    ///    },
    ///    "username": {
    ///      "description": "SMTP username",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationSmtpSettings {
        ///Default sender address for notification email channels
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub from: ::std::option::Option<::std::string::String>,
        ///SMTP server host
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub host: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub oauth: ::std::option::Option<NotificationSmtpoAuthSettings>,
        ///Whether an SMTP password is configured
        #[serde(rename = "passwordConfigured")]
        pub password_configured: bool,
        ///SMTP server port
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub port: ::std::option::Option<::std::string::String>,
        ///SMTP username
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub username: ::std::option::Option<::std::string::String>,
    }
    ///Workspace SMTP transport input for notification email delivery. Values are encrypted at rest where applicable.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Workspace SMTP transport input for notification email delivery. Values are encrypted at rest where applicable.",
    ///  "type": "object",
    ///  "properties": {
    ///    "clearPassword": {
    ///      "description": "Clear the stored SMTP password.",
    ///      "type": "boolean"
    ///    },
    ///    "from": {
    ///      "description": "Default sender address for notification email channels",
    ///      "type": "string"
    ///    },
    ///    "host": {
    ///      "description": "SMTP server host",
    ///      "type": "string"
    ///    },
    ///    "oauth": {
    ///      "oneOf": [
    ///        {
    ///          "type": "null"
    ///        },
    ///        {
    ///          "allOf": [
    ///            {
    ///              "$ref": "#/components/schemas/NotificationSMTPOAuthSettingsInput"
    ///            }
    ///          ]
    ///        }
    ///      ]
    ///    },
    ///    "password": {
    ///      "description": "SMTP password. Omit on updates to preserve the existing password.",
    ///      "type": "string"
    ///    },
    ///    "port": {
    ///      "description": "SMTP server port",
    ///      "type": "string"
    ///    },
    ///    "username": {
    ///      "description": "SMTP username",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationSmtpSettingsInput {
        ///Clear the stored SMTP password.
        #[serde(
            rename = "clearPassword",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub clear_password: ::std::option::Option<bool>,
        ///Default sender address for notification email channels
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub from: ::std::option::Option<::std::string::String>,
        ///SMTP server host
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub host: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub oauth: ::std::option::Option<NotificationSmtpoAuthSettingsInput>,
        ///SMTP password. Omit on updates to preserve the existing password.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub password: ::std::option::Option<::std::string::String>,
        ///SMTP server port
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub port: ::std::option::Option<::std::string::String>,
        ///SMTP username
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub username: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for NotificationSmtpSettingsInput {
        fn default() -> Self {
            Self {
                clear_password: Default::default(),
                from: Default::default(),
                host: Default::default(),
                oauth: Default::default(),
                password: Default::default(),
                port: Default::default(),
                username: Default::default(),
            }
        }
    }
    ///SMTP OAuth token provider
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "SMTP OAuth token provider",
    ///  "type": "string",
    ///  "enum": [
    ///    "microsoft",
    ///    "google_service_account",
    ///    "google_refresh"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum NotificationSmtpoAuthProvider {
        #[serde(rename = "microsoft")]
        Microsoft,
        #[serde(rename = "google_service_account")]
        GoogleServiceAccount,
        #[serde(rename = "google_refresh")]
        GoogleRefresh,
    }
    impl ::std::fmt::Display for NotificationSmtpoAuthProvider {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Microsoft => f.write_str("microsoft"),
                Self::GoogleServiceAccount => f.write_str("google_service_account"),
                Self::GoogleRefresh => f.write_str("google_refresh"),
            }
        }
    }
    impl ::std::str::FromStr for NotificationSmtpoAuthProvider {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "microsoft" => Ok(Self::Microsoft),
                "google_service_account" => Ok(Self::GoogleServiceAccount),
                "google_refresh" => Ok(Self::GoogleRefresh),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for NotificationSmtpoAuthProvider {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for NotificationSmtpoAuthProvider {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for NotificationSmtpoAuthProvider {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Public SMTP OAuth settings. Secret values are never returned.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Public SMTP OAuth settings. Secret values are never returned.",
    ///  "type": "object",
    ///  "required": [
    ///    "clientSecretConfigured",
    ///    "provider",
    ///    "refreshTokenConfigured",
    ///    "serviceAccountJsonConfigured"
    ///  ],
    ///  "properties": {
    ///    "clientId": {
    ///      "type": "string"
    ///    },
    ///    "clientSecretConfigured": {
    ///      "type": "boolean"
    ///    },
    ///    "provider": {
    ///      "$ref": "#/components/schemas/NotificationSMTPOAuthProvider"
    ///    },
    ///    "refreshTokenConfigured": {
    ///      "type": "boolean"
    ///    },
    ///    "serviceAccountJsonConfigured": {
    ///      "type": "boolean"
    ///    },
    ///    "tenantId": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationSmtpoAuthSettings {
        #[serde(
            rename = "clientId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub client_id: ::std::option::Option<::std::string::String>,
        #[serde(rename = "clientSecretConfigured")]
        pub client_secret_configured: bool,
        pub provider: NotificationSmtpoAuthProvider,
        #[serde(rename = "refreshTokenConfigured")]
        pub refresh_token_configured: bool,
        #[serde(rename = "serviceAccountJsonConfigured")]
        pub service_account_json_configured: bool,
        #[serde(
            rename = "tenantId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub tenant_id: ::std::option::Option<::std::string::String>,
    }
    ///SMTP OAuth credentials. Secret values are encrypted at rest and omitted from responses.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "SMTP OAuth credentials. Secret values are encrypted at rest and omitted from responses.",
    ///  "type": "object",
    ///  "required": [
    ///    "provider"
    ///  ],
    ///  "properties": {
    ///    "clientId": {
    ///      "description": "OAuth client ID",
    ///      "type": "string"
    ///    },
    ///    "clientSecret": {
    ///      "description": "OAuth client secret. Omit on updates to preserve the existing value when the OAuth identity is unchanged.",
    ///      "writeOnly": true,
    ///      "type": "string"
    ///    },
    ///    "provider": {
    ///      "$ref": "#/components/schemas/NotificationSMTPOAuthProvider"
    ///    },
    ///    "refreshToken": {
    ///      "description": "Google refresh token. Omit on updates to preserve the existing value when the OAuth identity is unchanged.",
    ///      "writeOnly": true,
    ///      "type": "string"
    ///    },
    ///    "serviceAccountJson": {
    ///      "description": "Google service-account JSON. Omit on updates to preserve the existing value when the OAuth identity is unchanged.",
    ///      "writeOnly": true,
    ///      "type": "string"
    ///    },
    ///    "tenantId": {
    ///      "description": "Microsoft Entra tenant ID",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationSmtpoAuthSettingsInput {
        ///OAuth client ID
        #[serde(
            rename = "clientId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub client_id: ::std::option::Option<::std::string::String>,
        ///OAuth client secret. Omit on updates to preserve the existing value when the OAuth identity is unchanged.
        #[serde(
            rename = "clientSecret",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub client_secret: ::std::option::Option<::std::string::String>,
        pub provider: NotificationSmtpoAuthProvider,
        ///Google refresh token. Omit on updates to preserve the existing value when the OAuth identity is unchanged.
        #[serde(
            rename = "refreshToken",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub refresh_token: ::std::option::Option<::std::string::String>,
        ///Google service-account JSON. Omit on updates to preserve the existing value when the OAuth identity is unchanged.
        #[serde(
            rename = "serviceAccountJson",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub service_account_json: ::std::option::Option<::std::string::String>,
        ///Microsoft Entra tenant ID
        #[serde(
            rename = "tenantId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub tenant_id: ::std::option::Option<::std::string::String>,
    }
    ///DAG subscription to a notification channel
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "DAG subscription to a notification channel",
    ///  "type": "object",
    ///  "required": [
    ///    "channelId",
    ///    "enabled",
    ///    "id"
    ///  ],
    ///  "properties": {
    ///    "channelId": {
    ///      "description": "Notification channel ID",
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this DAG subscription receives notifications",
    ///      "type": "boolean"
    ///    },
    ///    "events": {
    ///      "description": "Subscription-level event filter. Empty means the subscription inherits DAG-level events.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationEventType"
    ///      }
    ///    },
    ///    "id": {
    ///      "description": "Stable subscription ID",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationSubscription {
        ///Notification channel ID
        #[serde(rename = "channelId")]
        pub channel_id: ::std::string::String,
        ///Whether this DAG subscription receives notifications
        pub enabled: bool,
        ///Subscription-level event filter. Empty means the subscription inherits DAG-level events.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub events: ::std::vec::Vec<NotificationEventType>,
        ///Stable subscription ID
        pub id: ::std::string::String,
    }
    ///DAG subscription to a notification channel
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "DAG subscription to a notification channel",
    ///  "type": "object",
    ///  "required": [
    ///    "channelId",
    ///    "enabled"
    ///  ],
    ///  "properties": {
    ///    "channelId": {
    ///      "description": "Notification channel ID",
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this DAG subscription receives notifications",
    ///      "type": "boolean"
    ///    },
    ///    "events": {
    ///      "description": "Optional subscription-level event filter. When omitted or empty, the subscription inherits DAG-level events.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationEventType"
    ///      }
    ///    },
    ///    "id": {
    ///      "description": "Stable subscription ID. Omit when creating a new subscription.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationSubscriptionInput {
        ///Notification channel ID
        #[serde(rename = "channelId")]
        pub channel_id: ::std::string::String,
        ///Whether this DAG subscription receives notifications
        pub enabled: bool,
        ///Optional subscription-level event filter. When omitted or empty, the subscription inherits DAG-level events.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub events: ::std::vec::Vec<NotificationEventType>,
        ///Stable subscription ID. Omit when creating a new subscription.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub id: ::std::option::Option<::std::string::String>,
    }
    ///Public notification target details. Secrets are never returned.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Public notification target details. Secrets are never returned.",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "id",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "email": {
    ///      "$ref": "#/components/schemas/NotificationEmailTarget"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this target receives notifications",
    ///      "type": "boolean"
    ///    },
    ///    "events": {
    ///      "description": "Target-level event filter. Empty means the target inherits DAG-level events.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationEventType"
    ///      }
    ///    },
    ///    "id": {
    ///      "description": "Stable target ID",
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "description": "Human-readable target name",
    ///      "type": "string"
    ///    },
    ///    "slack": {
    ///      "$ref": "#/components/schemas/NotificationSlackTarget"
    ///    },
    ///    "teams": {
    ///      "$ref": "#/components/schemas/NotificationTeamsTarget"
    ///    },
    ///    "telegram": {
    ///      "$ref": "#/components/schemas/NotificationTelegramTarget"
    ///    },
    ///    "type": {
    ///      "$ref": "#/components/schemas/NotificationProviderType"
    ///    },
    ///    "webhook": {
    ///      "$ref": "#/components/schemas/NotificationWebhookTarget"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationTarget {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub email: ::std::option::Option<NotificationEmailTarget>,
        ///Whether this target receives notifications
        pub enabled: bool,
        ///Target-level event filter. Empty means the target inherits DAG-level events.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub events: ::std::vec::Vec<NotificationEventType>,
        ///Stable target ID
        pub id: ::std::string::String,
        ///Human-readable target name
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub name: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub slack: ::std::option::Option<NotificationSlackTarget>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub teams: ::std::option::Option<NotificationTeamsTarget>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub telegram: ::std::option::Option<NotificationTelegramTarget>,
        #[serde(rename = "type")]
        pub type_: NotificationProviderType,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub webhook: ::std::option::Option<NotificationWebhookTarget>,
    }
    ///Notification target input
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Notification target input",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "email": {
    ///      "$ref": "#/components/schemas/NotificationEmailTarget"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether this target receives notifications",
    ///      "type": "boolean"
    ///    },
    ///    "events": {
    ///      "description": "Optional target-level event filter. When omitted or empty, the target inherits DAG-level events.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationEventType"
    ///      }
    ///    },
    ///    "id": {
    ///      "description": "Stable target ID. Omit when creating a new target.",
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "description": "Human-readable target name",
    ///      "type": "string"
    ///    },
    ///    "slack": {
    ///      "$ref": "#/components/schemas/NotificationSlackTargetInput"
    ///    },
    ///    "teams": {
    ///      "$ref": "#/components/schemas/NotificationTeamsTargetInput"
    ///    },
    ///    "telegram": {
    ///      "$ref": "#/components/schemas/NotificationTelegramTargetInput"
    ///    },
    ///    "type": {
    ///      "$ref": "#/components/schemas/NotificationProviderType"
    ///    },
    ///    "webhook": {
    ///      "$ref": "#/components/schemas/NotificationWebhookTargetInput"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationTargetInput {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub email: ::std::option::Option<NotificationEmailTarget>,
        ///Whether this target receives notifications
        pub enabled: bool,
        ///Optional target-level event filter. When omitted or empty, the target inherits DAG-level events.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub events: ::std::vec::Vec<NotificationEventType>,
        ///Stable target ID. Omit when creating a new target.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub id: ::std::option::Option<::std::string::String>,
        ///Human-readable target name
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub name: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub slack: ::std::option::Option<NotificationSlackTargetInput>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub teams: ::std::option::Option<NotificationTeamsTargetInput>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub telegram: ::std::option::Option<NotificationTelegramTargetInput>,
        #[serde(rename = "type")]
        pub type_: NotificationProviderType,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub webhook: ::std::option::Option<NotificationWebhookTargetInput>,
    }
    ///Public Microsoft Teams target details
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Public Microsoft Teams target details",
    ///  "type": "object",
    ///  "required": [
    ///    "webhookUrlConfigured"
    ///  ],
    ///  "properties": {
    ///    "messageTemplate": {
    ///      "description": "Optional Teams message template. When omitted, Dagu sends the default notification text.",
    ///      "type": "string"
    ///    },
    ///    "webhookUrlConfigured": {
    ///      "description": "Whether a Microsoft Teams incoming webhook URL is configured",
    ///      "type": "boolean"
    ///    },
    ///    "webhookUrlPreview": {
    ///      "description": "Redacted Microsoft Teams webhook URL preview",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationTeamsTarget {
        ///Optional Teams message template. When omitted, Dagu sends the default notification text.
        #[serde(
            rename = "messageTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub message_template: ::std::option::Option<::std::string::String>,
        ///Whether a Microsoft Teams incoming webhook URL is configured
        #[serde(rename = "webhookUrlConfigured")]
        pub webhook_url_configured: bool,
        ///Redacted Microsoft Teams webhook URL preview
        #[serde(
            rename = "webhookUrlPreview",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub webhook_url_preview: ::std::option::Option<::std::string::String>,
    }
    ///Microsoft Teams incoming webhook target input. Values are encrypted at rest.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Microsoft Teams incoming webhook target input. Values are encrypted at rest.",
    ///  "type": "object",
    ///  "properties": {
    ///    "messageTemplate": {
    ///      "description": "Optional Teams message template. When omitted, Dagu sends the default notification text.",
    ///      "type": "string"
    ///    },
    ///    "webhookUrl": {
    ///      "description": "Microsoft Teams incoming webhook URL, from a Teams Workflows (Power Automate) trigger or a legacy connector. Must use HTTPS. Omit on updates to preserve the existing URL.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationTeamsTargetInput {
        ///Optional Teams message template. When omitted, Dagu sends the default notification text.
        #[serde(
            rename = "messageTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub message_template: ::std::option::Option<::std::string::String>,
        ///Microsoft Teams incoming webhook URL, from a Teams Workflows (Power Automate) trigger or a legacy connector. Must use HTTPS. Omit on updates to preserve the existing URL.
        #[serde(
            rename = "webhookUrl",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub webhook_url: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for NotificationTeamsTargetInput {
        fn default() -> Self {
            Self {
                message_template: Default::default(),
                webhook_url: Default::default(),
            }
        }
    }
    ///Public Telegram target details
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Public Telegram target details",
    ///  "type": "object",
    ///  "required": [
    ///    "botTokenConfigured"
    ///  ],
    ///  "properties": {
    ///    "botTokenConfigured": {
    ///      "description": "Whether a Telegram bot token is configured",
    ///      "type": "boolean"
    ///    },
    ///    "botTokenPreview": {
    ///      "description": "Redacted Telegram bot token preview",
    ///      "type": "string"
    ///    },
    ///    "chatId": {
    ///      "description": "Telegram chat ID",
    ///      "type": "string"
    ///    },
    ///    "messageTemplate": {
    ///      "description": "Optional Telegram message template. When omitted, Dagu sends the default notification text.",
    ///      "type": "string"
    ///    },
    ///    "topicId": {
    ///      "description": "Optional Telegram topic ID (message thread ID) for forum groups",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationTelegramTarget {
        ///Whether a Telegram bot token is configured
        #[serde(rename = "botTokenConfigured")]
        pub bot_token_configured: bool,
        ///Redacted Telegram bot token preview
        #[serde(
            rename = "botTokenPreview",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub bot_token_preview: ::std::option::Option<::std::string::String>,
        ///Telegram chat ID
        #[serde(
            rename = "chatId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub chat_id: ::std::option::Option<::std::string::String>,
        ///Optional Telegram message template. When omitted, Dagu sends the default notification text.
        #[serde(
            rename = "messageTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub message_template: ::std::option::Option<::std::string::String>,
        ///Optional Telegram topic ID (message thread ID) for forum groups
        #[serde(
            rename = "topicId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub topic_id: ::std::option::Option<::std::string::String>,
    }
    ///Telegram Bot API target input. Bot token is encrypted at rest.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Telegram Bot API target input. Bot token is encrypted at rest.",
    ///  "type": "object",
    ///  "properties": {
    ///    "botToken": {
    ///      "description": "Telegram bot token. Omit on updates to preserve the existing token.",
    ///      "type": "string"
    ///    },
    ///    "chatId": {
    ///      "description": "Telegram chat ID",
    ///      "type": "string"
    ///    },
    ///    "messageTemplate": {
    ///      "description": "Optional Telegram message template. When omitted, Dagu sends the default notification text.",
    ///      "type": "string"
    ///    },
    ///    "topicId": {
    ///      "description": "Optional Telegram topic ID (message thread ID) for forum groups",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationTelegramTargetInput {
        ///Telegram bot token. Omit on updates to preserve the existing token.
        #[serde(
            rename = "botToken",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub bot_token: ::std::option::Option<::std::string::String>,
        ///Telegram chat ID
        #[serde(
            rename = "chatId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub chat_id: ::std::option::Option<::std::string::String>,
        ///Optional Telegram message template. When omitted, Dagu sends the default notification text.
        #[serde(
            rename = "messageTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub message_template: ::std::option::Option<::std::string::String>,
        ///Optional Telegram topic ID (message thread ID) for forum groups
        #[serde(
            rename = "topicId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub topic_id: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for NotificationTelegramTargetInput {
        fn default() -> Self {
            Self {
                bot_token: Default::default(),
                chat_id: Default::default(),
                message_template: Default::default(),
                topic_id: Default::default(),
            }
        }
    }
    ///Public outbound webhook target details
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Public outbound webhook target details",
    ///  "type": "object",
    ///  "required": [
    ///    "hmacSecretConfigured",
    ///    "urlConfigured"
    ///  ],
    ///  "properties": {
    ///    "allowInsecureHttp": {
    ///      "description": "Whether this target allows plain HTTP webhook URLs",
    ///      "type": "boolean"
    ///    },
    ///    "allowPrivateNetwork": {
    ///      "description": "Whether this target allows loopback or private network webhook targets",
    ///      "type": "boolean"
    ///    },
    ///    "bodyTemplate": {
    ///      "description": "Optional request body template that replaces the default Dagu payload.",
    ///      "type": "string"
    ///    },
    ///    "headers": {
    ///      "description": "Header names with redacted values",
    ///      "type": "object",
    ///      "additionalProperties": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "hmacSecretConfigured": {
    ///      "description": "Whether an HMAC secret is configured",
    ///      "type": "boolean"
    ///    },
    ///    "messageTemplate": {
    ///      "description": "Optional rendered message added to the webhook JSON payload as message.",
    ///      "type": "string"
    ///    },
    ///    "urlConfigured": {
    ///      "description": "Whether a webhook URL is configured",
    ///      "type": "boolean"
    ///    },
    ///    "urlPreview": {
    ///      "description": "Redacted URL preview",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationWebhookTarget {
        ///Whether this target allows plain HTTP webhook URLs
        #[serde(
            rename = "allowInsecureHttp",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub allow_insecure_http: ::std::option::Option<bool>,
        ///Whether this target allows loopback or private network webhook targets
        #[serde(
            rename = "allowPrivateNetwork",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub allow_private_network: ::std::option::Option<bool>,
        ///Optional request body template that replaces the default Dagu payload.
        #[serde(
            rename = "bodyTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub body_template: ::std::option::Option<::std::string::String>,
        ///Header names with redacted values
        #[serde(
            default,
            skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
        )]
        pub headers: ::std::collections::HashMap<
            ::std::string::String,
            ::std::string::String,
        >,
        ///Whether an HMAC secret is configured
        #[serde(rename = "hmacSecretConfigured")]
        pub hmac_secret_configured: bool,
        ///Optional rendered message added to the webhook JSON payload as message.
        #[serde(
            rename = "messageTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub message_template: ::std::option::Option<::std::string::String>,
        ///Whether a webhook URL is configured
        #[serde(rename = "urlConfigured")]
        pub url_configured: bool,
        ///Redacted URL preview
        #[serde(
            rename = "urlPreview",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub url_preview: ::std::option::Option<::std::string::String>,
    }
    ///Outbound webhook target input. Values are encrypted at rest.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Outbound webhook target input. Values are encrypted at rest.",
    ///  "type": "object",
    ///  "properties": {
    ///    "allowInsecureHttp": {
    ///      "description": "Allow plain HTTP webhook URLs. Disabled by default.",
    ///      "type": "boolean"
    ///    },
    ///    "allowPrivateNetwork": {
    ///      "description": "Allow loopback or private network webhook targets. Disabled by default.",
    ///      "type": "boolean"
    ///    },
    ///    "bodyTemplate": {
    ///      "description": "Optional request body template. When set, it replaces the default Dagu payload and must render to valid JSON. One request is sent per event. Supports the same tokens as messageTemplate plus {{message}}.",
    ///      "type": "string"
    ///    },
    ///    "clearHeaders": {
    ///      "description": "Clear all stored webhook headers.",
    ///      "type": "boolean"
    ///    },
    ///    "clearHmacSecret": {
    ///      "description": "Clear the stored HMAC secret.",
    ///      "type": "boolean"
    ///    },
    ///    "headers": {
    ///      "description": "Additional request headers. Values are encrypted at rest. When provided, this replaces the stored header set.",
    ///      "type": "object",
    ///      "additionalProperties": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "hmacSecret": {
    ///      "description": "Optional HMAC secret for X-Dagu-Signature. Omit on updates to preserve the existing secret.",
    ///      "type": "string"
    ///    },
    ///    "messageTemplate": {
    ///      "description": "Optional rendered message added to the webhook JSON payload as message.",
    ///      "type": "string"
    ///    },
    ///    "url": {
    ///      "description": "HTTP or HTTPS endpoint to POST notification payloads to. Omit on updates to preserve the existing URL.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationWebhookTargetInput {
        ///Allow plain HTTP webhook URLs. Disabled by default.
        #[serde(
            rename = "allowInsecureHttp",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub allow_insecure_http: ::std::option::Option<bool>,
        ///Allow loopback or private network webhook targets. Disabled by default.
        #[serde(
            rename = "allowPrivateNetwork",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub allow_private_network: ::std::option::Option<bool>,
        ///Optional request body template. When set, it replaces the default Dagu payload and must render to valid JSON. One request is sent per event. Supports the same tokens as messageTemplate plus {{message}}.
        #[serde(
            rename = "bodyTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub body_template: ::std::option::Option<::std::string::String>,
        ///Clear all stored webhook headers.
        #[serde(
            rename = "clearHeaders",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub clear_headers: ::std::option::Option<bool>,
        ///Clear the stored HMAC secret.
        #[serde(
            rename = "clearHmacSecret",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub clear_hmac_secret: ::std::option::Option<bool>,
        ///Additional request headers. Values are encrypted at rest. When provided, this replaces the stored header set.
        #[serde(
            default,
            skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
        )]
        pub headers: ::std::collections::HashMap<
            ::std::string::String,
            ::std::string::String,
        >,
        ///Optional HMAC secret for X-Dagu-Signature. Omit on updates to preserve the existing secret.
        #[serde(
            rename = "hmacSecret",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub hmac_secret: ::std::option::Option<::std::string::String>,
        ///Optional rendered message added to the webhook JSON payload as message.
        #[serde(
            rename = "messageTemplate",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub message_template: ::std::option::Option<::std::string::String>,
        ///HTTP or HTTPS endpoint to POST notification payloads to. Omit on updates to preserve the existing URL.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub url: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for NotificationWebhookTargetInput {
        fn default() -> Self {
            Self {
                allow_insecure_http: Default::default(),
                allow_private_network: Default::default(),
                body_template: Default::default(),
                clear_headers: Default::default(),
                clear_hmac_secret: Default::default(),
                headers: Default::default(),
                hmac_secret: Default::default(),
                message_template: Default::default(),
                url: Default::default(),
            }
        }
    }
    ///Workspace-level notification settings
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Workspace-level notification settings",
    ///  "type": "object",
    ///  "properties": {
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "smtp": {
    ///      "$ref": "#/components/schemas/NotificationSMTPSettings"
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    },
    ///    "updatedBy": {
    ///      "description": "User ID that last updated the workspace notification settings",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationWorkspaceSettings {
        #[serde(
            rename = "createdAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub created_at: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub smtp: ::std::option::Option<NotificationSmtpSettings>,
        #[serde(
            rename = "updatedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_at: ::std::option::Option<::std::string::String>,
        ///User ID that last updated the workspace notification settings
        #[serde(
            rename = "updatedBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_by: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for NotificationWorkspaceSettings {
        fn default() -> Self {
            Self {
                created_at: Default::default(),
                smtp: Default::default(),
                updated_at: Default::default(),
                updated_by: Default::default(),
            }
        }
    }
    ///Workspace-level notification settings input
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Workspace-level notification settings input",
    ///  "type": "object",
    ///  "properties": {
    ///    "smtp": {
    ///      "oneOf": [
    ///        {
    ///          "type": "null"
    ///        },
    ///        {
    ///          "allOf": [
    ///            {
    ///              "$ref": "#/components/schemas/NotificationSMTPSettingsInput"
    ///            }
    ///          ]
    ///        }
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct NotificationWorkspaceSettingsInput {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub smtp: ::std::option::Option<NotificationSmtpSettingsInput>,
    }
    impl ::std::default::Default for NotificationWorkspaceSettingsInput {
        fn default() -> Self {
            Self { smtp: Default::default() }
        }
    }
    ///Execution context metadata for the outputs
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Execution context metadata for the outputs",
    ///  "type": "object",
    ///  "required": [
    ///    "attemptId",
    ///    "completedAt",
    ///    "dagName",
    ///    "dagRunId",
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "attemptId": {
    ///      "description": "Attempt identifier within the run",
    ///      "type": "string"
    ///    },
    ///    "completedAt": {
    ///      "description": "RFC3339 timestamp when execution completed",
    ///      "type": "string"
    ///    },
    ///    "dagName": {
    ///      "$ref": "#/components/schemas/DAGName"
    ///    },
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    },
    ///    "params": {
    ///      "description": "JSON-serialized parameters passed to the DAG",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "$ref": "#/components/schemas/StatusLabel"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct OutputsMetadata {
        ///Attempt identifier within the run
        #[serde(rename = "attemptId")]
        pub attempt_id: ::std::string::String,
        ///RFC3339 timestamp when execution completed
        #[serde(rename = "completedAt")]
        pub completed_at: ::std::string::String,
        #[serde(rename = "dagName")]
        pub dag_name: DagName,
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
        ///JSON-serialized parameters passed to the DAG
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub params: ::std::option::Option<::std::string::String>,
        pub status: StatusLabel,
    }
    ///`Pagination`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "currentPage",
    ///    "nextPage",
    ///    "prevPage",
    ///    "totalPages",
    ///    "totalRecords"
    ///  ],
    ///  "properties": {
    ///    "currentPage": {
    ///      "description": "current page number",
    ///      "type": "integer"
    ///    },
    ///    "nextPage": {
    ///      "description": "next page number",
    ///      "type": "integer"
    ///    },
    ///    "prevPage": {
    ///      "description": "previous page number",
    ///      "type": "integer"
    ///    },
    ///    "totalPages": {
    ///      "description": "total number of pages",
    ///      "type": "integer"
    ///    },
    ///    "totalRecords": {
    ///      "description": "total number of records",
    ///      "type": "integer"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct Pagination {
        ///current page number
        #[serde(rename = "currentPage")]
        pub current_page: i64,
        ///next page number
        #[serde(rename = "nextPage")]
        pub next_page: i64,
        ///previous page number
        #[serde(rename = "prevPage")]
        pub prev_page: i64,
        ///total number of pages
        #[serde(rename = "totalPages")]
        pub total_pages: i64,
        ///total number of records
        #[serde(rename = "totalRecords")]
        pub total_records: i64,
    }
    ///Typed metadata for a single DAG parameter
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Typed metadata for a single DAG parameter",
    ///  "type": "object",
    ///  "required": [
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "default": {
    ///      "$ref": "#/components/schemas/ParamScalar"
    ///    },
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "enum": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ParamScalar"
    ///      }
    ///    },
    ///    "maxLength": {
    ///      "type": "integer"
    ///    },
    ///    "maximum": {
    ///      "type": "number",
    ///      "format": "double"
    ///    },
    ///    "minLength": {
    ///      "type": "integer"
    ///    },
    ///    "minimum": {
    ///      "type": "number",
    ///      "format": "double"
    ///    },
    ///    "name": {
    ///      "description": "Parameter name. Omitted for positional parameters.",
    ///      "type": "string"
    ///    },
    ///    "required": {
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "type": {
    ///      "description": "Parameter type",
    ///      "type": "string",
    ///      "enum": [
    ///        "string",
    ///        "integer",
    ///        "number",
    ///        "boolean"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ParamDef {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub default: ::std::option::Option<ParamScalar>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "enum",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub enum_: ::std::vec::Vec<ParamScalar>,
        #[serde(
            rename = "maxLength",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub max_length: ::std::option::Option<i64>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub maximum: ::std::option::Option<f64>,
        #[serde(
            rename = "minLength",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub min_length: ::std::option::Option<i64>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub minimum: ::std::option::Option<f64>,
        ///Parameter name. Omitted for positional parameters.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub name: ::std::option::Option<::std::string::String>,
        #[serde(default)]
        pub required: bool,
        ///Parameter type
        #[serde(rename = "type")]
        pub type_: ParamDefType,
    }
    ///Parameter type
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Parameter type",
    ///  "type": "string",
    ///  "enum": [
    ///    "string",
    ///    "integer",
    ///    "number",
    ///    "boolean"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ParamDefType {
        #[serde(rename = "string")]
        String,
        #[serde(rename = "integer")]
        Integer,
        #[serde(rename = "number")]
        Number,
        #[serde(rename = "boolean")]
        Boolean,
    }
    impl ::std::fmt::Display for ParamDefType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::String => f.write_str("string"),
                Self::Integer => f.write_str("integer"),
                Self::Number => f.write_str("number"),
                Self::Boolean => f.write_str("boolean"),
            }
        }
    }
    impl ::std::str::FromStr for ParamDefType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "string" => Ok(Self::String),
                "integer" => Ok(Self::Integer),
                "number" => Ok(Self::Number),
                "boolean" => Ok(Self::Boolean),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ParamDefType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ParamDefType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ParamDefType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Scalar parameter value
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Scalar parameter value",
    ///  "anyOf": [
    ///    {
    ///      "type": "string"
    ///    },
    ///    {
    ///      "type": "integer"
    ///    },
    ///    {
    ///      "type": "number",
    ///      "format": "double"
    ///    },
    ///    {
    ///      "type": "boolean"
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(untagged)]
    pub enum ParamScalar {
        String(::std::string::String),
        Integer(i64),
        Double(f64),
        Boolean(bool),
    }
    impl ::std::fmt::Display for ParamScalar {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::String(x) => x.fmt(f),
                Self::Integer(x) => x.fmt(f),
                Self::Double(x) => x.fmt(f),
                Self::Boolean(x) => x.fmt(f),
            }
        }
    }
    impl ::std::convert::From<i64> for ParamScalar {
        fn from(value: i64) -> Self {
            Self::Integer(value)
        }
    }
    impl ::std::convert::From<f64> for ParamScalar {
        fn from(value: f64) -> Self {
            Self::Double(value)
        }
    }
    impl ::std::convert::From<bool> for ParamScalar {
        fn from(value: bool) -> Self {
            Self::Boolean(value)
        }
    }
    ///One push-back event recorded for an approval step
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "One push-back event recorded for an approval step",
    ///  "type": "object",
    ///  "required": [
    ///    "iteration"
    ///  ],
    ///  "properties": {
    ///    "at": {
    ///      "description": "RFC3339 timestamp when the push-back was recorded",
    ///      "type": "string"
    ///    },
    ///    "by": {
    ///      "description": "Authenticated user who pushed the step back",
    ///      "type": "string"
    ///    },
    ///    "byId": {
    ///      "description": "ID of the subject that pushed the step back",
    ///      "type": "string"
    ///    },
    ///    "inputs": {
    ///      "description": "Inputs provided for this push-back event",
    ///      "type": "object",
    ///      "additionalProperties": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "iteration": {
    ///      "description": "Push-back iteration number",
    ///      "type": "integer",
    ///      "minimum": 1.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct PushBackHistoryEntry {
        ///RFC3339 timestamp when the push-back was recorded
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub at: ::std::option::Option<::std::string::String>,
        ///Authenticated user who pushed the step back
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub by: ::std::option::Option<::std::string::String>,
        ///ID of the subject that pushed the step back
        #[serde(
            rename = "byId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub by_id: ::std::option::Option<::std::string::String>,
        ///Inputs provided for this push-back event
        #[serde(
            default,
            skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
        )]
        pub inputs: ::std::collections::HashMap<
            ::std::string::String,
            ::std::string::String,
        >,
        ///Push-back iteration number
        pub iteration: ::std::num::NonZeroU64,
    }
    ///Request body for pushing back a waiting step for re-execution with feedback
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for pushing back a waiting step for re-execution with feedback",
    ///  "type": "object",
    ///  "properties": {
    ///    "inputs": {
    ///      "description": "Key-value parameters to provide as feedback. These will be injected as environment variables when the step re-executes.",
    ///      "type": "object",
    ///      "additionalProperties": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct PushBackStepRequest {
        ///Key-value parameters to provide as feedback. These will be injected as environment variables when the step re-executes.
        #[serde(
            default,
            skip_serializing_if = ":: std :: collections :: HashMap::is_empty"
        )]
        pub inputs: ::std::collections::HashMap<
            ::std::string::String,
            ::std::string::String,
        >,
    }
    impl ::std::default::Default for PushBackStepRequest {
        fn default() -> Self {
            Self { inputs: Default::default() }
        }
    }
    ///Response after pushing back a waiting step
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response after pushing back a waiting step",
    ///  "type": "object",
    ///  "required": [
    ///    "approvalIteration",
    ///    "dagRunId",
    ///    "resumed",
    ///    "stepName"
    ///  ],
    ///  "properties": {
    ///    "approvalIteration": {
    ///      "description": "The current approval iteration count after push-back",
    ///      "type": "integer"
    ///    },
    ///    "dagRunId": {
    ///      "description": "The DAG run ID",
    ///      "type": "string"
    ///    },
    ///    "resumed": {
    ///      "description": "Whether the DAG run was re-enqueued for execution",
    ///      "type": "boolean"
    ///    },
    ///    "stepName": {
    ///      "description": "The pushed-back step name",
    ///      "type": "string"
    ///    },
    ///    "subDAGRunId": {
    ///      "description": "The sub-DAG run ID, present only for sub-DAG push-back operations",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct PushBackStepResponse {
        ///The current approval iteration count after push-back
        #[serde(rename = "approvalIteration")]
        pub approval_iteration: i64,
        ///The DAG run ID
        #[serde(rename = "dagRunId")]
        pub dag_run_id: ::std::string::String,
        ///Whether the DAG run was re-enqueued for execution
        pub resumed: bool,
        ///The pushed-back step name
        #[serde(rename = "stepName")]
        pub step_name: ::std::string::String,
        ///The sub-DAG run ID, present only for sub-DAG push-back operations
        #[serde(
            rename = "subDAGRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub sub_dag_run_id: ::std::option::Option<::std::string::String>,
    }
    ///A queue/process group with summary statistics
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A queue/process group with summary statistics",
    ///  "type": "object",
    ///  "required": [
    ///    "name",
    ///    "queuedCount",
    ///    "running",
    ///    "runningCount",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "maxConcurrency": {
    ///      "description": "Maximum number of concurrent runs allowed. For 'global' queues, this is the configured maxConcurrency. For 'dag-based' queues, this is always 1 (FIFO execution)",
    ///      "type": "integer",
    ///      "minimum": 1.0
    ///    },
    ///    "name": {
    ///      "description": "Name of the queue (global queue name or DAG name if no queue specified)",
    ///      "type": "string"
    ///    },
    ///    "queuedCount": {
    ///      "description": "Number of queued DAG-runs waiting to execute",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "running": {
    ///      "description": "List of currently running DAG-runs (bounded by maxConcurrency)",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/DAGRunSummary"
    ///      }
    ///    },
    ///    "runningCount": {
    ///      "description": "Number of currently running DAG-runs",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "type": {
    ///      "description": "Type of queue - 'global' if explicitly defined, 'dag-based' if using DAG name",
    ///      "type": "string",
    ///      "enum": [
    ///        "global",
    ///        "dag-based"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct Queue {
        ///Maximum number of concurrent runs allowed. For 'global' queues, this is the configured maxConcurrency. For 'dag-based' queues, this is always 1 (FIFO execution)
        #[serde(
            rename = "maxConcurrency",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub max_concurrency: ::std::option::Option<::std::num::NonZeroU64>,
        ///Name of the queue (global queue name or DAG name if no queue specified)
        pub name: ::std::string::String,
        ///Number of queued DAG-runs waiting to execute
        #[serde(rename = "queuedCount")]
        pub queued_count: u64,
        ///List of currently running DAG-runs (bounded by maxConcurrency)
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub running: ::std::vec::Vec<DagRunSummary>,
        ///Number of currently running DAG-runs
        #[serde(rename = "runningCount")]
        pub running_count: u64,
        ///Type of queue - 'global' if explicitly defined, 'dag-based' if using DAG name
        #[serde(rename = "type")]
        pub type_: QueueType,
    }
    ///Type of queue - 'global' if explicitly defined, 'dag-based' if using DAG name
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Type of queue - 'global' if explicitly defined, 'dag-based' if using DAG name",
    ///  "type": "string",
    ///  "enum": [
    ///    "global",
    ///    "dag-based"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum QueueType {
        #[serde(rename = "global")]
        Global,
        #[serde(rename = "dag-based")]
        DagBased,
    }
    impl ::std::fmt::Display for QueueType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Global => f.write_str("global"),
                Self::DagBased => f.write_str("dag-based"),
            }
        }
    }
    impl ::std::str::FromStr for QueueType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "global" => Ok(Self::Global),
                "dag-based" => Ok(Self::DagBased),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for QueueType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for QueueType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for QueueType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Forward-only paginated queued DAG-run response
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Forward-only paginated queued DAG-run response",
    ///  "type": "object",
    ///  "required": [
    ///    "items"
    ///  ],
    ///  "properties": {
    ///    "items": {
    ///      "description": "List of queued DAG-run summaries",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/DAGRunSummary"
    ///      }
    ///    },
    ///    "nextCursor": {
    ///      "description": "Opaque cursor for loading the next page of queued DAG-runs",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct QueuedDagRunsPageResponse {
        ///List of queued DAG-run summaries
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub items: ::std::vec::Vec<DagRunSummary>,
        ///Opaque cursor for loading the next page of queued DAG-runs
        #[serde(
            rename = "nextCursor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub next_cursor: ::std::option::Option<::std::string::String>,
    }
    ///Response containing all queues with their active DAG-runs
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response containing all queues with their active DAG-runs",
    ///  "type": "object",
    ///  "required": [
    ///    "queues",
    ///    "summary"
    ///  ],
    ///  "properties": {
    ///    "queues": {
    ///      "description": "List of all queues with their running and queued DAG-runs",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Queue"
    ///      }
    ///    },
    ///    "summary": {
    ///      "$ref": "#/components/schemas/QueuesSummary"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct QueuesResponse {
        ///List of all queues with their running and queued DAG-runs
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub queues: ::std::vec::Vec<Queue>,
        pub summary: QueuesSummary,
    }
    ///Summary statistics across all queues
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Summary statistics across all queues",
    ///  "type": "object",
    ///  "required": [
    ///    "totalCapacity",
    ///    "totalQueued",
    ///    "totalQueues",
    ///    "totalRunning",
    ///    "utilizationPercentage"
    ///  ],
    ///  "properties": {
    ///    "totalCapacity": {
    ///      "description": "Sum of all queue maxConcurrency values",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "totalQueued": {
    ///      "description": "Total DAG-runs waiting in queues",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "totalQueues": {
    ///      "description": "Total number of active queues",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "totalRunning": {
    ///      "description": "Total DAG-runs currently executing",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    },
    ///    "utilizationPercentage": {
    ///      "description": "System-wide utilization (totalRunning / totalCapacity * 100)",
    ///      "type": "number",
    ///      "format": "float",
    ///      "maximum": 100.0,
    ///      "minimum": 0.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct QueuesSummary {
        ///Sum of all queue maxConcurrency values
        #[serde(rename = "totalCapacity")]
        pub total_capacity: u64,
        ///Total DAG-runs waiting in queues
        #[serde(rename = "totalQueued")]
        pub total_queued: u64,
        ///Total number of active queues
        #[serde(rename = "totalQueues")]
        pub total_queues: u64,
        ///Total DAG-runs currently executing
        #[serde(rename = "totalRunning")]
        pub total_running: u64,
        ///System-wide utilization (totalRunning / totalCapacity * 100)
        #[serde(rename = "utilizationPercentage")]
        pub utilization_percentage: f32,
    }
    ///Request body for rejecting a waiting step
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for rejecting a waiting step",
    ///  "type": "object",
    ///  "properties": {
    ///    "reason": {
    ///      "description": "Optional reason for rejecting the step",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RejectStepRequest {
        ///Optional reason for rejecting the step
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub reason: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for RejectStepRequest {
        fn default() -> Self {
            Self { reason: Default::default() }
        }
    }
    ///Response after rejecting a waiting step
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response after rejecting a waiting step",
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunId",
    ///    "stepName"
    ///  ],
    ///  "properties": {
    ///    "dagRunId": {
    ///      "description": "The DAG run ID",
    ///      "type": "string"
    ///    },
    ///    "stepName": {
    ///      "description": "The rejected step name",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RejectStepResponse {
        ///The DAG run ID
        #[serde(rename = "dagRunId")]
        pub dag_run_id: ::std::string::String,
        ///The rejected step name
        #[serde(rename = "stepName")]
        pub step_name: ::std::string::String,
    }
    ///`RemoteNodeListResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "remoteNodes"
    ///  ],
    ///  "properties": {
    ///    "remoteNodes": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/RemoteNodeResponse"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RemoteNodeListResponse {
        #[serde(rename = "remoteNodes")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub remote_nodes: ::std::vec::Vec<RemoteNodeResponse>,
    }
    ///`RemoteNodeResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "apiBaseUrl",
    ///    "authType",
    ///    "id",
    ///    "name",
    ///    "source"
    ///  ],
    ///  "properties": {
    ///    "apiBaseUrl": {
    ///      "type": "string"
    ///    },
    ///    "authType": {
    ///      "type": "string",
    ///      "enum": [
    ///        "none",
    ///        "basic",
    ///        "token"
    ///      ]
    ///    },
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "hasCredentials": {
    ///      "description": "Whether credentials are configured (values are never returned)",
    ///      "type": "boolean"
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "skipTlsVerify": {
    ///      "type": "boolean"
    ///    },
    ///    "source": {
    ///      "description": "Where this node is defined",
    ///      "type": "string",
    ///      "enum": [
    ///        "config",
    ///        "store"
    ///      ]
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RemoteNodeResponse {
        #[serde(rename = "apiBaseUrl")]
        pub api_base_url: ::std::string::String,
        #[serde(rename = "authType")]
        pub auth_type: RemoteNodeResponseAuthType,
        #[serde(
            rename = "createdAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub created_at: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        ///Whether credentials are configured (values are never returned)
        #[serde(
            rename = "hasCredentials",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub has_credentials: ::std::option::Option<bool>,
        pub id: ::std::string::String,
        pub name: ::std::string::String,
        #[serde(
            rename = "skipTlsVerify",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub skip_tls_verify: ::std::option::Option<bool>,
        ///Where this node is defined
        pub source: RemoteNodeResponseSource,
        #[serde(
            rename = "updatedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_at: ::std::option::Option<::std::string::String>,
    }
    ///`RemoteNodeResponseAuthType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "none",
    ///    "basic",
    ///    "token"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum RemoteNodeResponseAuthType {
        #[serde(rename = "none")]
        None,
        #[serde(rename = "basic")]
        Basic,
        #[serde(rename = "token")]
        Token,
    }
    impl ::std::fmt::Display for RemoteNodeResponseAuthType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::None => f.write_str("none"),
                Self::Basic => f.write_str("basic"),
                Self::Token => f.write_str("token"),
            }
        }
    }
    impl ::std::str::FromStr for RemoteNodeResponseAuthType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "none" => Ok(Self::None),
                "basic" => Ok(Self::Basic),
                "token" => Ok(Self::Token),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for RemoteNodeResponseAuthType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for RemoteNodeResponseAuthType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for RemoteNodeResponseAuthType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Where this node is defined
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Where this node is defined",
    ///  "type": "string",
    ///  "enum": [
    ///    "config",
    ///    "store"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum RemoteNodeResponseSource {
        #[serde(rename = "config")]
        Config,
        #[serde(rename = "store")]
        Store,
    }
    impl ::std::fmt::Display for RemoteNodeResponseSource {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Config => f.write_str("config"),
                Self::Store => f.write_str("store"),
            }
        }
    }
    impl ::std::str::FromStr for RemoteNodeResponseSource {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "config" => Ok(Self::Config),
                "store" => Ok(Self::Store),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for RemoteNodeResponseSource {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for RemoteNodeResponseSource {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for RemoteNodeResponseSource {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`RenameDagBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "newFileName"
    ///  ],
    ///  "properties": {
    ///    "newFileName": {
    ///      "description": "New file name for the DAG",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RenameDagBody {
        ///New file name for the DAG
        #[serde(rename = "newFileName")]
        pub new_file_name: ::std::string::String,
    }
    ///Request to rename/move a Wiki page or directory
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to rename/move a Wiki page or directory",
    ///  "type": "object",
    ///  "required": [
    ///    "newPath"
    ///  ],
    ///  "properties": {
    ///    "newPath": {
    ///      "$ref": "#/components/schemas/WikiPagePath"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RenameWikiPageRequest {
        #[serde(rename = "newPath")]
        pub new_path: WikiPagePath,
    }
    ///Repeat execution mode for steps
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Repeat execution mode for steps",
    ///  "type": "string",
    ///  "enum": [
    ///    "while",
    ///    "until"
    ///  ],
    ///  "x-enum-varnames": [
    ///    "While",
    ///    "Until"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum RepeatMode {
        #[serde(rename = "while")]
        While,
        #[serde(rename = "until")]
        Until,
    }
    impl ::std::fmt::Display for RepeatMode {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::While => f.write_str("while"),
                Self::Until => f.write_str("until"),
            }
        }
    }
    impl ::std::str::FromStr for RepeatMode {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "while" => Ok(Self::While),
                "until" => Ok(Self::Until),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for RepeatMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for RepeatMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for RepeatMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Configuration for step repeat behavior
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Configuration for step repeat behavior",
    ///  "type": "object",
    ///  "properties": {
    ///    "backoff": {
    ///      "oneOf": [
    ///        {
    ///          "description": "When true, uses default multiplier of 2.0",
    ///          "type": "boolean"
    ///        },
    ///        {
    ///          "description": "Custom exponential backoff multiplier",
    ///          "type": "number",
    ///          "format": "float"
    ///        }
    ///      ]
    ///    },
    ///    "condition": {
    ///      "$ref": "#/components/schemas/Condition"
    ///    },
    ///    "exitCode": {
    ///      "description": "List of exit codes that trigger repeat behavior",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "integer"
    ///      }
    ///    },
    ///    "interval": {
    ///      "description": "Time in seconds to wait between repeat attempts",
    ///      "type": "integer"
    ///    },
    ///    "limit": {
    ///      "description": "Maximum number of times to repeat the step",
    ///      "type": "integer"
    ///    },
    ///    "maxIntervalSec": {
    ///      "description": "Maximum interval in seconds (caps exponential growth)",
    ///      "type": "integer"
    ///    },
    ///    "repeat": {
    ///      "$ref": "#/components/schemas/RepeatMode"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RepeatPolicy {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub backoff: ::std::option::Option<RepeatPolicyBackoff>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub condition: ::std::option::Option<Condition>,
        ///List of exit codes that trigger repeat behavior
        #[serde(
            rename = "exitCode",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub exit_code: ::std::vec::Vec<i64>,
        ///Time in seconds to wait between repeat attempts
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub interval: ::std::option::Option<i64>,
        ///Maximum number of times to repeat the step
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub limit: ::std::option::Option<i64>,
        ///Maximum interval in seconds (caps exponential growth)
        #[serde(
            rename = "maxIntervalSec",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub max_interval_sec: ::std::option::Option<i64>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub repeat: ::std::option::Option<RepeatMode>,
    }
    impl ::std::default::Default for RepeatPolicy {
        fn default() -> Self {
            Self {
                backoff: Default::default(),
                condition: Default::default(),
                exit_code: Default::default(),
                interval: Default::default(),
                limit: Default::default(),
                max_interval_sec: Default::default(),
                repeat: Default::default(),
            }
        }
    }
    ///`RepeatPolicyBackoff`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "oneOf": [
    ///    {
    ///      "description": "When true, uses default multiplier of 2.0",
    ///      "type": "boolean"
    ///    },
    ///    {
    ///      "description": "Custom exponential backoff multiplier",
    ///      "type": "number",
    ///      "format": "float"
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(untagged)]
    pub enum RepeatPolicyBackoff {
        Boolean(bool),
        Float(f32),
    }
    impl ::std::str::FromStr for RepeatPolicyBackoff {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if let Ok(v) = value.parse() {
                Ok(Self::Boolean(v))
            } else if let Ok(v) = value.parse() {
                Ok(Self::Float(v))
            } else {
                Err("string conversion failed for all variants".into())
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for RepeatPolicyBackoff {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for RepeatPolicyBackoff {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for RepeatPolicyBackoff {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::fmt::Display for RepeatPolicyBackoff {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::Boolean(x) => x.fmt(f),
                Self::Float(x) => x.fmt(f),
            }
        }
    }
    impl ::std::convert::From<bool> for RepeatPolicyBackoff {
        fn from(value: bool) -> Self {
            Self::Boolean(value)
        }
    }
    impl ::std::convert::From<f32> for RepeatPolicyBackoff {
        fn from(value: f32) -> Self {
            Self::Float(value)
        }
    }
    ///`RescheduleDagRunBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "dagName": {
    ///      "description": "Optional DAG name override for the new run.",
    ///      "type": "string"
    ///    },
    ///    "dagRunId": {
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/DAGRunId"
    ///        },
    ///        {
    ///          "description": "Explicit run ID for the new DAG-run; if omitted a new ID is generated."
    ///        }
    ///      ]
    ///    },
    ///    "useCurrentDagFile": {
    ///      "description": "When true, reschedule from the current contents of the original DAG file instead of the stored historical YAML snapshot.",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RescheduleDagRunBody {
        ///Optional DAG name override for the new run.
        #[serde(
            rename = "dagName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_name: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "dagRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_run_id: ::std::option::Option<DagRunId>,
        ///When true, reschedule from the current contents of the original DAG file instead of the stored historical YAML snapshot.
        #[serde(
            rename = "useCurrentDagFile",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub use_current_dag_file: ::std::option::Option<bool>,
    }
    impl ::std::default::Default for RescheduleDagRunBody {
        fn default() -> Self {
            Self {
                dag_name: Default::default(),
                dag_run_id: Default::default(),
                use_current_dag_file: Default::default(),
            }
        }
    }
    ///`RescheduleDagRunResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunId",
    ///    "queued"
    ///  ],
    ///  "properties": {
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    },
    ///    "queued": {
    ///      "description": "Indicates whether the run was queued instead of starting immediately.",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RescheduleDagRunResponse {
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
        ///Indicates whether the run was queued instead of starting immediately.
        pub queued: bool,
    }
    ///Request body for admin password reset
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for admin password reset",
    ///  "type": "object",
    ///  "required": [
    ///    "newPassword"
    ///  ],
    ///  "properties": {
    ///    "newPassword": {
    ///      "description": "New password to set for the user",
    ///      "type": "string",
    ///      "minLength": 8
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ResetPasswordRequest {
        ///New password to set for the user
        #[serde(rename = "newPassword")]
        pub new_password: ResetPasswordRequestNewPassword,
    }
    ///New password to set for the user
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "New password to set for the user",
    ///  "type": "string",
    ///  "minLength": 8
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct ResetPasswordRequestNewPassword(::std::string::String);
    impl ::std::ops::Deref for ResetPasswordRequestNewPassword {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<ResetPasswordRequestNewPassword>
    for ::std::string::String {
        fn from(value: ResetPasswordRequestNewPassword) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for ResetPasswordRequestNewPassword {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() < 8usize {
                return Err("shorter than 8 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for ResetPasswordRequestNewPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for ResetPasswordRequestNewPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for ResetPasswordRequestNewPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for ResetPasswordRequestNewPassword {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///`ResourceHistory`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "cpu": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/MetricPoint"
    ///      }
    ///    },
    ///    "disk": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/MetricPoint"
    ///      }
    ///    },
    ///    "diskTotalBytes": {
    ///      "description": "Total disk space in bytes",
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "diskUsedBytes": {
    ///      "description": "Used disk space in bytes",
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "load": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/MetricPoint"
    ///      }
    ///    },
    ///    "memory": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/MetricPoint"
    ///      }
    ///    },
    ///    "memoryTotalBytes": {
    ///      "description": "Total physical memory in bytes",
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "memoryUsedBytes": {
    ///      "description": "Used physical memory in bytes",
    ///      "type": "integer",
    ///      "format": "int64"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ResourceHistory {
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub cpu: ::std::vec::Vec<MetricPoint>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub disk: ::std::vec::Vec<MetricPoint>,
        ///Total disk space in bytes
        #[serde(
            rename = "diskTotalBytes",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub disk_total_bytes: ::std::option::Option<i64>,
        ///Used disk space in bytes
        #[serde(
            rename = "diskUsedBytes",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub disk_used_bytes: ::std::option::Option<i64>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub load: ::std::vec::Vec<MetricPoint>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub memory: ::std::vec::Vec<MetricPoint>,
        ///Total physical memory in bytes
        #[serde(
            rename = "memoryTotalBytes",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub memory_total_bytes: ::std::option::Option<i64>,
        ///Used physical memory in bytes
        #[serde(
            rename = "memoryUsedBytes",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub memory_used_bytes: ::std::option::Option<i64>,
    }
    impl ::std::default::Default for ResourceHistory {
        fn default() -> Self {
            Self {
                cpu: Default::default(),
                disk: Default::default(),
                disk_total_bytes: Default::default(),
                disk_used_bytes: Default::default(),
                load: Default::default(),
                memory: Default::default(),
                memory_total_bytes: Default::default(),
                memory_used_bytes: Default::default(),
            }
        }
    }
    ///`RetryDagRunBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunId"
    ///  ],
    ///  "properties": {
    ///    "dagRunId": {
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/DAGRunId"
    ///        },
    ///        {
    ///          "description": "ID of the DAG-run to retry"
    ///        }
    ///      ]
    ///    },
    ///    "includeDownstream": {
    ///      "description": "Optional. When true, retry the selected step and every reachable descendant. Requires stepName. Unrelated branches keep their existing status.",
    ///      "type": "boolean"
    ///    },
    ///    "stepName": {
    ///      "description": "Optional. If provided, this step will be retried. Combine with includeDownstream to also reset reachable descendants.",
    ///      "type": "string"
    ///    },
    ///    "subDAGRunId": {
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/DAGRunId"
    ///        },
    ///        {
    ///          "description": "Optional. Persisted child DAG-run containing the step. Requires stepName; the path and dagRunId remain the root DAG-run."
    ///        }
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RetryDagRunBody {
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
        ///Optional. When true, retry the selected step and every reachable descendant. Requires stepName. Unrelated branches keep their existing status.
        #[serde(
            rename = "includeDownstream",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub include_downstream: ::std::option::Option<bool>,
        ///Optional. If provided, this step will be retried. Combine with includeDownstream to also reset reachable descendants.
        #[serde(
            rename = "stepName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub step_name: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "subDAGRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub sub_dag_run_id: ::std::option::Option<DagRunId>,
    }
    ///Configuration for controlling user interactions when starting DAG runs
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Configuration for controlling user interactions when starting DAG runs",
    ///  "type": "object",
    ///  "required": [
    ///    "disableParamEdit",
    ///    "disableRunIdEdit"
    ///  ],
    ///  "properties": {
    ///    "disableParamEdit": {
    ///      "description": "Disable parameter editing when starting the DAG",
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "disableRunIdEdit": {
    ///      "description": "Disable custom run ID specification",
    ///      "default": false,
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RunConfig {
        ///Disable parameter editing when starting the DAG
        #[serde(rename = "disableParamEdit")]
        pub disable_param_edit: bool,
        ///Disable custom run ID specification
        #[serde(rename = "disableRunIdEdit")]
        pub disable_run_id_edit: bool,
    }
    ///Information about a task currently being executed
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Information about a task currently being executed",
    ///  "type": "object",
    ///  "required": [
    ///    "dagName",
    ///    "dagRunId",
    ///    "startedAt"
    ///  ],
    ///  "properties": {
    ///    "dagName": {
    ///      "$ref": "#/components/schemas/DAGName"
    ///    },
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    },
    ///    "parentDagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    },
    ///    "parentDagRunName": {
    ///      "$ref": "#/components/schemas/DAGName"
    ///    },
    ///    "rootDagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    },
    ///    "rootDagRunName": {
    ///      "$ref": "#/components/schemas/DAGName"
    ///    },
    ///    "startedAt": {
    ///      "description": "RFC3339 timestamp when the task started",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RunningTask {
        #[serde(rename = "dagName")]
        pub dag_name: DagName,
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
        #[serde(
            rename = "parentDagRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub parent_dag_run_id: ::std::option::Option<DagRunId>,
        #[serde(
            rename = "parentDagRunName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub parent_dag_run_name: ::std::option::Option<DagName>,
        #[serde(
            rename = "rootDagRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub root_dag_run_id: ::std::option::Option<DagRunId>,
        #[serde(
            rename = "rootDagRunName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub root_dag_run_name: ::std::option::Option<DagName>,
        ///RFC3339 timestamp when the task started
        #[serde(rename = "startedAt")]
        pub started_at: ::std::string::String,
    }
    ///`RuntimeProfileEntryKind`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "variable",
    ///    "secret"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum RuntimeProfileEntryKind {
        #[serde(rename = "variable")]
        Variable,
        #[serde(rename = "secret")]
        Secret,
    }
    impl ::std::fmt::Display for RuntimeProfileEntryKind {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Variable => f.write_str("variable"),
                Self::Secret => f.write_str("secret"),
            }
        }
    }
    impl ::std::str::FromStr for RuntimeProfileEntryKind {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "variable" => Ok(Self::Variable),
                "secret" => Ok(Self::Secret),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for RuntimeProfileEntryKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for RuntimeProfileEntryKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for RuntimeProfileEntryKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`RuntimeProfileEntryResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "createdAt",
    ///    "key",
    ///    "kind",
    ///    "updatedAt"
    ///  ],
    ///  "properties": {
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "key": {
    ///      "$ref": "#/components/schemas/RuntimeProfileKey"
    ///    },
    ///    "kind": {
    ///      "$ref": "#/components/schemas/RuntimeProfileEntryKind"
    ///    },
    ///    "secretId": {
    ///      "description": "Managed secret ID for secret entries. The secret value is never returned.",
    ///      "type": "string"
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    },
    ///    "value": {
    ///      "description": "Stored value for non-secret variables. Omitted for secret entries.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RuntimeProfileEntryResponse {
        #[serde(rename = "createdAt")]
        pub created_at: ::std::string::String,
        pub key: RuntimeProfileKey,
        pub kind: RuntimeProfileEntryKind,
        ///Managed secret ID for secret entries. The secret value is never returned.
        #[serde(
            rename = "secretId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub secret_id: ::std::option::Option<::std::string::String>,
        #[serde(rename = "updatedAt")]
        pub updated_at: ::std::string::String,
        ///Stored value for non-secret variables. Omitted for secret entries.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub value: ::std::option::Option<::std::string::String>,
    }
    ///Environment variable key stored in a runtime profile. Keys with the DAGU_ prefix are reserved.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Environment variable key stored in a runtime profile. Keys with the DAGU_ prefix are reserved.",
    ///  "type": "string",
    ///  "maxLength": 255,
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct RuntimeProfileKey(::std::string::String);
    impl ::std::ops::Deref for RuntimeProfileKey {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<RuntimeProfileKey> for ::std::string::String {
        fn from(value: RuntimeProfileKey) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for RuntimeProfileKey {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 255usize {
                return Err("longer than 255 characters".into());
            }
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for RuntimeProfileKey {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for RuntimeProfileKey {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for RuntimeProfileKey {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for RuntimeProfileKey {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///`RuntimeProfileListResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "profiles"
    ///  ],
    ///  "properties": {
    ///    "profiles": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/RuntimeProfileResponse"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RuntimeProfileListResponse {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub profiles: ::std::vec::Vec<RuntimeProfileResponse>,
    }
    ///Runtime profile name.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Runtime profile name.",
    ///  "type": "string",
    ///  "maxLength": 128,
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct RuntimeProfileName(::std::string::String);
    impl ::std::ops::Deref for RuntimeProfileName {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<RuntimeProfileName> for ::std::string::String {
        fn from(value: RuntimeProfileName) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for RuntimeProfileName {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 128usize {
                return Err("longer than 128 characters".into());
            }
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for RuntimeProfileName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for RuntimeProfileName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for RuntimeProfileName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for RuntimeProfileName {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Runtime profile override. Empty string means no profile.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Runtime profile override. Empty string means no profile.",
    ///  "type": "string",
    ///  "maxLength": 128
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct RuntimeProfileOverride(::std::string::String);
    impl ::std::ops::Deref for RuntimeProfileOverride {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<RuntimeProfileOverride> for ::std::string::String {
        fn from(value: RuntimeProfileOverride) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for RuntimeProfileOverride {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 128usize {
                return Err("longer than 128 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for RuntimeProfileOverride {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for RuntimeProfileOverride {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for RuntimeProfileOverride {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for RuntimeProfileOverride {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///`RuntimeProfileResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "createdAt",
    ///    "entries",
    ///    "id",
    ///    "name",
    ///    "protected",
    ///    "status",
    ///    "updatedAt"
    ///  ],
    ///  "properties": {
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "entries": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/RuntimeProfileEntryResponse"
    ///      }
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "$ref": "#/components/schemas/RuntimeProfileName"
    ///    },
    ///    "protected": {
    ///      "type": "boolean"
    ///    },
    ///    "status": {
    ///      "$ref": "#/components/schemas/RuntimeProfileStatus"
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct RuntimeProfileResponse {
        #[serde(rename = "createdAt")]
        pub created_at: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub entries: ::std::vec::Vec<RuntimeProfileEntryResponse>,
        pub id: ::std::string::String,
        pub name: RuntimeProfileName,
        pub protected: bool,
        pub status: RuntimeProfileStatus,
        #[serde(rename = "updatedAt")]
        pub updated_at: ::std::string::String,
    }
    ///`RuntimeProfileStatus`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "active",
    ///    "disabled"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum RuntimeProfileStatus {
        #[serde(rename = "active")]
        Active,
        #[serde(rename = "disabled")]
        Disabled,
    }
    impl ::std::fmt::Display for RuntimeProfileStatus {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Active => f.write_str("active"),
                Self::Disabled => f.write_str("disabled"),
            }
        }
    }
    impl ::std::str::FromStr for RuntimeProfileStatus {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "active" => Ok(Self::Active),
                "disabled" => Ok(Self::Disabled),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for RuntimeProfileStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for RuntimeProfileStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for RuntimeProfileStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Schedule configuration for DAG-run creation
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Schedule configuration for DAG-run creation",
    ///  "type": "object",
    ///  "minProperties": 1,
    ///  "properties": {
    ///    "at": {
    ///      "description": "RFC 3339 timestamp with explicit offset for one-off schedules",
    ///      "type": "string"
    ///    },
    ///    "expression": {
    ///      "description": "Cron expression for recurring schedules",
    ///      "type": "string",
    ///      "x-go-type-skip-optional-pointer": true
    ///    },
    ///    "kind": {
    ///      "description": "Schedule type. When omitted alongside expression, the schedule is treated as cron for backward compatibility.",
    ///      "type": "string",
    ///      "enum": [
    ///        "cron",
    ///        "at"
    ///      ]
    ///    },
    ///    "profile": {
    ///      "$ref": "#/components/schemas/RuntimeProfileName"
    ///    }
    ///  },
    ///  "additionalProperties": false
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct Schedule {
        ///RFC 3339 timestamp with explicit offset for one-off schedules
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub at: ::std::option::Option<::std::string::String>,
        ///Cron expression for recurring schedules
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub expression: ::std::option::Option<::std::string::String>,
        ///Schedule type. When omitted alongside expression, the schedule is treated as cron for backward compatibility.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub kind: ::std::option::Option<ScheduleKind>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub profile: ::std::option::Option<RuntimeProfileName>,
    }
    impl ::std::default::Default for Schedule {
        fn default() -> Self {
            Self {
                at: Default::default(),
                expression: Default::default(),
                kind: Default::default(),
                profile: Default::default(),
            }
        }
    }
    ///Schedule type. When omitted alongside expression, the schedule is treated as cron for backward compatibility.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Schedule type. When omitted alongside expression, the schedule is treated as cron for backward compatibility.",
    ///  "type": "string",
    ///  "enum": [
    ///    "cron",
    ///    "at"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ScheduleKind {
        #[serde(rename = "cron")]
        Cron,
        #[serde(rename = "at")]
        At,
    }
    impl ::std::fmt::Display for ScheduleKind {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Cron => f.write_str("cron"),
                Self::At => f.write_str("at"),
            }
        }
    }
    impl ::std::str::FromStr for ScheduleKind {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "cron" => Ok(Self::Cron),
                "at" => Ok(Self::At),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ScheduleKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ScheduleKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ScheduleKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Scheduler instance status information
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Scheduler instance status information",
    ///  "type": "object",
    ///  "required": [
    ///    "host",
    ///    "instanceId",
    ///    "startedAt",
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "host": {
    ///      "description": "Hostname where scheduler is running",
    ///      "type": "string"
    ///    },
    ///    "instanceId": {
    ///      "description": "Unique identifier of the scheduler instance",
    ///      "type": "string"
    ///    },
    ///    "startedAt": {
    ///      "description": "RFC3339 timestamp when scheduler started",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "description": "Scheduler status (active = holds lock and scheduling)",
    ///      "type": "string",
    ///      "enum": [
    ///        "active",
    ///        "inactive",
    ///        "unknown"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SchedulerInstance {
        ///Hostname where scheduler is running
        pub host: ::std::string::String,
        ///Unique identifier of the scheduler instance
        #[serde(rename = "instanceId")]
        pub instance_id: ::std::string::String,
        ///RFC3339 timestamp when scheduler started
        #[serde(rename = "startedAt")]
        pub started_at: ::std::string::String,
        ///Scheduler status (active = holds lock and scheduling)
        pub status: SchedulerInstanceStatus,
    }
    ///Scheduler status (active = holds lock and scheduling)
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Scheduler status (active = holds lock and scheduling)",
    ///  "type": "string",
    ///  "enum": [
    ///    "active",
    ///    "inactive",
    ///    "unknown"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum SchedulerInstanceStatus {
        #[serde(rename = "active")]
        Active,
        #[serde(rename = "inactive")]
        Inactive,
        #[serde(rename = "unknown")]
        Unknown,
    }
    impl ::std::fmt::Display for SchedulerInstanceStatus {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Active => f.write_str("active"),
                Self::Inactive => f.write_str("inactive"),
                Self::Unknown => f.write_str("unknown"),
            }
        }
    }
    impl ::std::str::FromStr for SchedulerInstanceStatus {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "active" => Ok(Self::Active),
                "inactive" => Ok(Self::Inactive),
                "unknown" => Ok(Self::Unknown),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for SchedulerInstanceStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for SchedulerInstanceStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for SchedulerInstanceStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Response containing status of all scheduler instances
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response containing status of all scheduler instances",
    ///  "type": "object",
    ///  "required": [
    ///    "schedulers"
    ///  ],
    ///  "properties": {
    ///    "schedulers": {
    ///      "description": "List of all registered scheduler instances",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/SchedulerInstance"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SchedulerStatusResponse {
        ///List of all registered scheduler instances
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub schedulers: ::std::vec::Vec<SchedulerInstance>,
    }
    ///Details of a search match within a search result
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Details of a search match within a search result",
    ///  "type": "object",
    ///  "required": [
    ///    "line",
    ///    "lineNumber",
    ///    "startLine"
    ///  ],
    ///  "properties": {
    ///    "line": {
    ///      "description": "Matching line content",
    ///      "type": "string"
    ///    },
    ///    "lineNumber": {
    ///      "description": "Line number where match was found",
    ///      "type": "integer"
    ///    },
    ///    "startLine": {
    ///      "description": "Start line for context",
    ///      "type": "integer"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SearchMatchItem {
        ///Matching line content
        pub line: ::std::string::String,
        ///Line number where match was found
        #[serde(rename = "lineNumber")]
        pub line_number: i64,
        ///Start line for context
        #[serde(rename = "startLine")]
        pub start_line: i64,
    }
    ///Cursor-based search match snippets
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Cursor-based search match snippets",
    ///  "type": "object",
    ///  "required": [
    ///    "hasMore",
    ///    "matches"
    ///  ],
    ///  "properties": {
    ///    "hasMore": {
    ///      "type": "boolean"
    ///    },
    ///    "matches": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/SearchMatchItem"
    ///      }
    ///    },
    ///    "nextCursor": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SearchMatchesResponse {
        #[serde(rename = "hasMore")]
        pub has_more: bool,
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub matches: ::std::vec::Vec<SearchMatchItem>,
        #[serde(
            rename = "nextCursor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub next_cursor: ::std::option::Option<::std::string::String>,
    }
    ///Individual search result item for a DAG
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Individual search result item for a DAG",
    ///  "type": "object",
    ///  "required": [
    ///    "dag",
    ///    "matches",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "dag": {
    ///      "$ref": "#/components/schemas/DAG"
    ///    },
    ///    "matches": {
    ///      "description": "Details of where matches were found",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/SearchMatchItem"
    ///      }
    ///    },
    ///    "name": {
    ///      "description": "Name of the matching DAG",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SearchResultItem {
        pub dag: Dag,
        ///Details of where matches were found
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub matches: ::std::vec::Vec<SearchMatchItem>,
        ///Name of the matching DAG
        pub name: ::std::string::String,
    }
    ///`SecretListResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "secrets",
    ///    "total"
    ///  ],
    ///  "properties": {
    ///    "secrets": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/SecretResponse"
    ///      }
    ///    },
    ///    "total": {
    ///      "type": "integer"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SecretListResponse {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub secrets: ::std::vec::Vec<SecretResponse>,
        pub total: i64,
    }
    ///`SecretProviderType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "dagu-managed",
    ///    "vault",
    ///    "kubernetes",
    ///    "gcp",
    ///    "aws",
    ///    "azure",
    ///    "alibaba"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum SecretProviderType {
        #[serde(rename = "dagu-managed")]
        DaguManaged,
        #[serde(rename = "vault")]
        Vault,
        #[serde(rename = "kubernetes")]
        Kubernetes,
        #[serde(rename = "gcp")]
        Gcp,
        #[serde(rename = "aws")]
        Aws,
        #[serde(rename = "azure")]
        Azure,
        #[serde(rename = "alibaba")]
        Alibaba,
    }
    impl ::std::fmt::Display for SecretProviderType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::DaguManaged => f.write_str("dagu-managed"),
                Self::Vault => f.write_str("vault"),
                Self::Kubernetes => f.write_str("kubernetes"),
                Self::Gcp => f.write_str("gcp"),
                Self::Aws => f.write_str("aws"),
                Self::Azure => f.write_str("azure"),
                Self::Alibaba => f.write_str("alibaba"),
            }
        }
    }
    impl ::std::str::FromStr for SecretProviderType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "dagu-managed" => Ok(Self::DaguManaged),
                "vault" => Ok(Self::Vault),
                "kubernetes" => Ok(Self::Kubernetes),
                "gcp" => Ok(Self::Gcp),
                "aws" => Ok(Self::Aws),
                "azure" => Ok(Self::Azure),
                "alibaba" => Ok(Self::Alibaba),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for SecretProviderType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for SecretProviderType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for SecretProviderType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`SecretResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "createdAt",
    ///    "currentVersion",
    ///    "hasValue",
    ///    "id",
    ///    "providerType",
    ///    "ref",
    ///    "status",
    ///    "updatedAt",
    ///    "workspace"
    ///  ],
    ///  "properties": {
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "currentVersion": {
    ///      "type": "integer"
    ///    },
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "hasValue": {
    ///      "type": "boolean"
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "lastCheckedAt": {
    ///      "type": "string"
    ///    },
    ///    "lastResolvedAt": {
    ///      "type": "string"
    ///    },
    ///    "lastRotatedAt": {
    ///      "type": "string"
    ///    },
    ///    "providerConnectionId": {
    ///      "type": "string"
    ///    },
    ///    "providerRef": {
    ///      "type": "string"
    ///    },
    ///    "providerRefFingerprint": {
    ///      "type": "string"
    ///    },
    ///    "providerType": {
    ///      "$ref": "#/components/schemas/SecretProviderType"
    ///    },
    ///    "ref": {
    ///      "description": "Secret ref used from DAG YAML, for example prod/db-password.",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "$ref": "#/components/schemas/SecretStatus"
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "description": "global for workspace-less secrets, otherwise the workspace name.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SecretResponse {
        #[serde(rename = "createdAt")]
        pub created_at: ::std::string::String,
        #[serde(rename = "currentVersion")]
        pub current_version: i64,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        #[serde(rename = "hasValue")]
        pub has_value: bool,
        pub id: ::std::string::String,
        #[serde(
            rename = "lastCheckedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub last_checked_at: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "lastResolvedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub last_resolved_at: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "lastRotatedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub last_rotated_at: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "providerConnectionId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub provider_connection_id: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "providerRef",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub provider_ref: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "providerRefFingerprint",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub provider_ref_fingerprint: ::std::option::Option<::std::string::String>,
        #[serde(rename = "providerType")]
        pub provider_type: SecretProviderType,
        ///Secret ref used from DAG YAML, for example prod/db-password.
        #[serde(rename = "ref")]
        pub ref_: ::std::string::String,
        pub status: SecretStatus,
        #[serde(rename = "updatedAt")]
        pub updated_at: ::std::string::String,
        ///global for workspace-less secrets, otherwise the workspace name.
        pub workspace: ::std::string::String,
    }
    ///`SecretStatus`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "active",
    ///    "disabled"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum SecretStatus {
        #[serde(rename = "active")]
        Active,
        #[serde(rename = "disabled")]
        Disabled,
    }
    impl ::std::fmt::Display for SecretStatus {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Active => f.write_str("active"),
                Self::Disabled => f.write_str("disabled"),
            }
        }
    }
    impl ::std::str::FromStr for SecretStatus {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "active" => Ok(Self::Active),
                "disabled" => Ok(Self::Disabled),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for SecretStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for SecretStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for SecretStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`SetRuntimeProfileSecretRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "value"
    ///  ],
    ///  "properties": {
    ///    "value": {
    ///      "writeOnly": true,
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SetRuntimeProfileSecretRequest {
        pub value: ::std::string::String,
    }
    ///`SetRuntimeProfileVariableRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "value"
    ///  ],
    ///  "properties": {
    ///    "value": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SetRuntimeProfileVariableRequest {
        pub value: ::std::string::String,
    }
    ///Request body for initial admin account setup
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for initial admin account setup",
    ///  "type": "object",
    ///  "required": [
    ///    "password",
    ///    "username"
    ///  ],
    ///  "properties": {
    ///    "password": {
    ///      "description": "Admin password",
    ///      "type": "string",
    ///      "minLength": 8
    ///    },
    ///    "username": {
    ///      "description": "Admin username",
    ///      "type": "string",
    ///      "minLength": 1
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SetupRequest {
        ///Admin password
        pub password: SetupRequestPassword,
        ///Admin username
        pub username: SetupRequestUsername,
    }
    ///Admin password
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Admin password",
    ///  "type": "string",
    ///  "minLength": 8
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct SetupRequestPassword(::std::string::String);
    impl ::std::ops::Deref for SetupRequestPassword {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<SetupRequestPassword> for ::std::string::String {
        fn from(value: SetupRequestPassword) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for SetupRequestPassword {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() < 8usize {
                return Err("shorter than 8 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for SetupRequestPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for SetupRequestPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for SetupRequestPassword {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for SetupRequestPassword {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Admin username
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Admin username",
    ///  "type": "string",
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct SetupRequestUsername(::std::string::String);
    impl ::std::ops::Deref for SetupRequestUsername {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<SetupRequestUsername> for ::std::string::String {
        fn from(value: SetupRequestUsername) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for SetupRequestUsername {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for SetupRequestUsername {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for SetupRequestUsername {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for SetupRequestUsername {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for SetupRequestUsername {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    /**Numeric status code indicating current DAG-run state:
0: "Not started"
1: "Running"
2: "Failed"
3: "Aborted"
4: "Success"
5: "Queued"
6: "Partial Success"
7: "Waiting for manual action"
8: "Rejected"
*/
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Numeric status code indicating current DAG-run state:\n0: \"Not started\"\n1: \"Running\"\n2: \"Failed\"\n3: \"Aborted\"\n4: \"Success\"\n5: \"Queued\"\n6: \"Partial Success\"\n7: \"Waiting for manual action\"\n8: \"Rejected\"\n",
    ///  "type": "integer",
    ///  "enum": [
    ///    0,
    ///    1,
    ///    2,
    ///    3,
    ///    4,
    ///    5,
    ///    6,
    ///    7,
    ///    8
    ///  ],
    ///  "x-enum-varnames": [
    ///    "NotStarted",
    ///    "Running",
    ///    "Failed",
    ///    "Aborted",
    ///    "Success",
    ///    "Queued",
    ///    "PartialSuccess",
    ///    "Waiting",
    ///    "Rejected"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug)]
    #[serde(transparent)]
    pub struct Status(i64);
    impl ::std::ops::Deref for Status {
        type Target = i64;
        fn deref(&self) -> &i64 {
            &self.0
        }
    }
    impl ::std::convert::From<Status> for i64 {
        fn from(value: Status) -> Self {
            value.0
        }
    }
    impl ::std::convert::TryFrom<i64> for Status {
        type Error = self::error::ConversionError;
        fn try_from(
            value: i64,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if ![0_i64, 1_i64, 2_i64, 3_i64, 4_i64, 5_i64, 6_i64, 7_i64, 8_i64]
                .contains(&value)
            {
                Err("invalid value".into())
            } else {
                Ok(Self(value))
            }
        }
    }
    impl<'de> ::serde::Deserialize<'de> for Status {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            Self::try_from(<i64>::deserialize(deserializer)?)
                .map_err(|e| { <D::Error as ::serde::de::Error>::custom(e.to_string()) })
        }
    }
    ///Human-readable status description for the DAG-run
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Human-readable status description for the DAG-run",
    ///  "type": "string",
    ///  "enum": [
    ///    "not_started",
    ///    "running",
    ///    "failed",
    ///    "aborted",
    ///    "succeeded",
    ///    "queued",
    ///    "partially_succeeded",
    ///    "waiting",
    ///    "rejected"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum StatusLabel {
        #[serde(rename = "not_started")]
        NotStarted,
        #[serde(rename = "running")]
        Running,
        #[serde(rename = "failed")]
        Failed,
        #[serde(rename = "aborted")]
        Aborted,
        #[serde(rename = "succeeded")]
        Succeeded,
        #[serde(rename = "queued")]
        Queued,
        #[serde(rename = "partially_succeeded")]
        PartiallySucceeded,
        #[serde(rename = "waiting")]
        Waiting,
        #[serde(rename = "rejected")]
        Rejected,
    }
    impl ::std::fmt::Display for StatusLabel {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::NotStarted => f.write_str("not_started"),
                Self::Running => f.write_str("running"),
                Self::Failed => f.write_str("failed"),
                Self::Aborted => f.write_str("aborted"),
                Self::Succeeded => f.write_str("succeeded"),
                Self::Queued => f.write_str("queued"),
                Self::PartiallySucceeded => f.write_str("partially_succeeded"),
                Self::Waiting => f.write_str("waiting"),
                Self::Rejected => f.write_str("rejected"),
            }
        }
    }
    impl ::std::str::FromStr for StatusLabel {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "not_started" => Ok(Self::NotStarted),
                "running" => Ok(Self::Running),
                "failed" => Ok(Self::Failed),
                "aborted" => Ok(Self::Aborted),
                "succeeded" => Ok(Self::Succeeded),
                "queued" => Ok(Self::Queued),
                "partially_succeeded" => Ok(Self::PartiallySucceeded),
                "waiting" => Ok(Self::Waiting),
                "rejected" => Ok(Self::Rejected),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for StatusLabel {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for StatusLabel {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for StatusLabel {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Individual task definition that performs a specific operation in a DAG-run
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Individual task definition that performs a specific operation in a DAG-run",
    ///  "type": "object",
    ///  "required": [
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "approval": {
    ///      "$ref": "#/components/schemas/ApprovalConfig"
    ///    },
    ///    "call": {
    ///      "description": "The name of the DAG to execute as a sub DAG-run",
    ///      "type": "string"
    ///    },
    ///    "commands": {
    ///      "description": "List of commands to execute sequentially",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/CommandEntry"
    ///      }
    ///    },
    ///    "dependencies": {
    ///      "description": "DAG-local files, directories, or glob patterns to snapshot for distributed execution",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      },
    ///      "minItems": 1
    ///    },
    ///    "depends": {
    ///      "description": "List of step names that must complete before this step can start",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "description": {
    ///      "description": "Human-readable description of what the step does",
    ///      "type": "string"
    ///    },
    ///    "dir": {
    ///      "description": "Working directory for executing the step's command",
    ///      "type": "string"
    ///    },
    ///    "executorConfig": {
    ///      "description": "Executor configuration for this step",
    ///      "type": "object",
    ///      "properties": {
    ///        "config": {
    ///          "description": "Executor-specific configuration",
    ///          "type": "object",
    ///          "additionalProperties": true
    ///        },
    ///        "type": {
    ///          "description": "Type of executor (e.g., 'wait', 'http', 'docker', 'command')",
    ///          "type": "string"
    ///        }
    ///      }
    ///    },
    ///    "humanTask": {
    ///      "$ref": "#/components/schemas/HumanTaskConfig"
    ///    },
    ///    "id": {
    ///      "description": "Optional short identifier for the step. Can be used in variable references like ${id.stdout} to access step properties. Must be unique within the DAG if specified",
    ///      "type": "string"
    ///    },
    ///    "inputs": {
    ///      "description": "Named regular-file inputs used for build fingerprints and inferred dependencies. Steps that declare inputs must also define id.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/StepInputDeclaration"
    ///      },
    ///      "minItems": 1
    ///    },
    ///    "mailOnError": {
    ///      "description": "Whether to send email notifications on step failure",
    ///      "type": "boolean"
    ///    },
    ///    "name": {
    ///      "description": "Unique identifier for the step within the DAG-run",
    ///      "type": "string"
    ///    },
    ///    "output": {
    ///      "description": "Variable name to store the step's output",
    ///      "type": "string"
    ///    },
    ///    "outputs": {
    ///      "description": "Declared named outputs. Value outputs are published through DAGU_OUTPUT_FILE; path outputs publish a verified materialized file. Steps that declare outputs must also define id.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/StepOutputDeclaration"
    ///      },
    ///      "minItems": 1
    ///    },
    ///    "parallel": {
    ///      "description": "Configuration for parallel execution of the step",
    ///      "type": "object",
    ///      "properties": {
    ///        "items": {
    ///          "description": "Array of items to process in parallel. Can be a static array or a reference to a variable containing an array",
    ///          "oneOf": [
    ///            {
    ///              "type": "array",
    ///              "items": {
    ///                "type": "string"
    ///              }
    ///            },
    ///            {
    ///              "type": "string"
    ///            }
    ///          ]
    ///        },
    ///        "maxConcurrent": {
    ///          "description": "Maximum number of parallel executions. Default is 10 if not specified",
    ///          "type": "integer",
    ///          "minimum": 1.0
    ///        }
    ///      }
    ///    },
    ///    "params": {
    ///      "description": "Parameters to pass to the sub DAG-run in JSON format",
    ///      "type": "string"
    ///    },
    ///    "preconditions": {
    ///      "description": "Conditions that must be met before the step can start",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Condition"
    ///      }
    ///    },
    ///    "repeatPolicy": {
    ///      "$ref": "#/components/schemas/RepeatPolicy"
    ///    },
    ///    "router": {
    ///      "description": "Router configuration for switch/case routing",
    ///      "type": "object",
    ///      "required": [
    ///        "routes",
    ///        "value"
    ///      ],
    ///      "properties": {
    ///        "routes": {
    ///          "type": "array",
    ///          "items": {
    ///            "type": "object",
    ///            "required": [
    ///              "pattern",
    ///              "targets"
    ///            ],
    ///            "properties": {
    ///              "targets": {
    ///                "description": "Step names to route to",
    ///                "type": "array",
    ///                "items": {
    ///                  "type": "string",
    ///                  "minLength": 1
    ///                },
    ///                "minItems": 1
    ///              }
    ///            }
    ///          },
    ///          "minItems": 1
    ///        },
    ///        "value": {
    ///          "description": "Expression to evaluate (e.g., '${STATUS}')",
    ///          "type": "string",
    ///          "minLength": 1
    ///        }
    ///      }
    ///    },
    ///    "script": {
    ///      "description": "Script content if the step executes a script file",
    ///      "type": "string"
    ///    },
    ///    "stderr": {
    ///      "description": "File path for capturing standard error",
    ///      "type": "string"
    ///    },
    ///    "stdout": {
    ///      "description": "File path for capturing standard output",
    ///      "type": "string"
    ///    },
    ///    "timeoutSec": {
    ///      "description": "Maximum execution time for the step in seconds. If set, this timeout takes precedence over the DAG-level timeout for this step.",
    ///      "type": "integer",
    ///      "minimum": 0.0
    ///    }
    ///  },
    ///  "x-dagu-required-with": {
    ///    "inputs": [
    ///      "id"
    ///    ],
    ///    "outputs": [
    ///      "id"
    ///    ]
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct Step {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub approval: ::std::option::Option<ApprovalConfig>,
        ///The name of the DAG to execute as a sub DAG-run
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub call: ::std::option::Option<::std::string::String>,
        ///List of commands to execute sequentially
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub commands: ::std::vec::Vec<CommandEntry>,
        ///DAG-local files, directories, or glob patterns to snapshot for distributed execution
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub dependencies: ::std::vec::Vec<::std::string::String>,
        ///List of step names that must complete before this step can start
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub depends: ::std::vec::Vec<::std::string::String>,
        ///Human-readable description of what the step does
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        ///Working directory for executing the step's command
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub dir: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "executorConfig",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub executor_config: ::std::option::Option<StepExecutorConfig>,
        #[serde(
            rename = "humanTask",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub human_task: ::std::option::Option<HumanTaskConfig>,
        ///Optional short identifier for the step. Can be used in variable references like ${id.stdout} to access step properties. Must be unique within the DAG if specified
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub id: ::std::option::Option<::std::string::String>,
        ///Named regular-file inputs used for build fingerprints and inferred dependencies. Steps that declare inputs must also define id.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub inputs: ::std::vec::Vec<StepInputDeclaration>,
        ///Whether to send email notifications on step failure
        #[serde(
            rename = "mailOnError",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub mail_on_error: ::std::option::Option<bool>,
        ///Unique identifier for the step within the DAG-run
        pub name: ::std::string::String,
        ///Variable name to store the step's output
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub output: ::std::option::Option<::std::string::String>,
        ///Declared named outputs. Value outputs are published through DAGU_OUTPUT_FILE; path outputs publish a verified materialized file. Steps that declare outputs must also define id.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub outputs: ::std::vec::Vec<StepOutputDeclaration>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub parallel: ::std::option::Option<StepParallel>,
        ///Parameters to pass to the sub DAG-run in JSON format
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub params: ::std::option::Option<::std::string::String>,
        ///Conditions that must be met before the step can start
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub preconditions: ::std::vec::Vec<Condition>,
        #[serde(
            rename = "repeatPolicy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub repeat_policy: ::std::option::Option<RepeatPolicy>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub router: ::std::option::Option<StepRouter>,
        ///Script content if the step executes a script file
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub script: ::std::option::Option<::std::string::String>,
        ///File path for capturing standard error
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub stderr: ::std::option::Option<::std::string::String>,
        ///File path for capturing standard output
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub stdout: ::std::option::Option<::std::string::String>,
        ///Maximum execution time for the step in seconds. If set, this timeout takes precedence over the DAG-level timeout for this step.
        #[serde(
            rename = "timeoutSec",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub timeout_sec: ::std::option::Option<u64>,
    }
    ///Executor configuration for this step
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Executor configuration for this step",
    ///  "type": "object",
    ///  "properties": {
    ///    "config": {
    ///      "description": "Executor-specific configuration",
    ///      "type": "object",
    ///      "additionalProperties": true
    ///    },
    ///    "type": {
    ///      "description": "Type of executor (e.g., 'wait', 'http', 'docker', 'command')",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct StepExecutorConfig {
        ///Executor-specific configuration
        #[serde(default, skip_serializing_if = "::serde_json::Map::is_empty")]
        pub config: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
        ///Type of executor (e.g., 'wait', 'http', 'docker', 'command')
        #[serde(
            rename = "type",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub type_: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for StepExecutorConfig {
        fn default() -> Self {
            Self {
                config: Default::default(),
                type_: Default::default(),
            }
        }
    }
    ///One named regular-file input for build execution
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "One named regular-file input for build execution",
    ///  "type": "object",
    ///  "required": [
    ///    "name",
    ///    "path"
    ///  ],
    ///  "properties": {
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "path": {
    ///      "type": "string"
    ///    }
    ///  },
    ///  "additionalProperties": false
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct StepInputDeclaration {
        pub name: ::std::string::String,
        pub path: ::std::string::String,
    }
    ///One named value output or build file output
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "One named value output or build file output",
    ///  "type": "object",
    ///  "required": [
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "name": {
    ///      "description": "Published output name scoped to the declaring step",
    ///      "type": "string"
    ///    },
    ///    "path": {
    ///      "description": "Regular-file path published atomically by a build step",
    ///      "type": "string"
    ///    },
    ///    "type": {
    ///      "description": "Output value type. JSON outputs must contain valid JSON text.",
    ///      "type": "string",
    ///      "enum": [
    ///        "string",
    ///        "json"
    ///      ]
    ///    }
    ///  },
    ///  "additionalProperties": false
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(deny_unknown_fields)]
    pub struct StepOutputDeclaration {
        ///Published output name scoped to the declaring step
        pub name: ::std::string::String,
        ///Regular-file path published atomically by a build step
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub path: ::std::option::Option<::std::string::String>,
        ///Output value type. JSON outputs must contain valid JSON text.
        #[serde(
            rename = "type",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub type_: ::std::option::Option<StepOutputDeclarationType>,
    }
    ///Output value type. JSON outputs must contain valid JSON text.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Output value type. JSON outputs must contain valid JSON text.",
    ///  "type": "string",
    ///  "enum": [
    ///    "string",
    ///    "json"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum StepOutputDeclarationType {
        #[serde(rename = "string")]
        String,
        #[serde(rename = "json")]
        Json,
    }
    impl ::std::fmt::Display for StepOutputDeclarationType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::String => f.write_str("string"),
                Self::Json => f.write_str("json"),
            }
        }
    }
    impl ::std::str::FromStr for StepOutputDeclarationType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "string" => Ok(Self::String),
                "json" => Ok(Self::Json),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for StepOutputDeclarationType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for StepOutputDeclarationType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for StepOutputDeclarationType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Configuration for parallel execution of the step
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Configuration for parallel execution of the step",
    ///  "type": "object",
    ///  "properties": {
    ///    "items": {
    ///      "description": "Array of items to process in parallel. Can be a static array or a reference to a variable containing an array",
    ///      "oneOf": [
    ///        {
    ///          "type": "array",
    ///          "items": {
    ///            "type": "string"
    ///          }
    ///        },
    ///        {
    ///          "type": "string"
    ///        }
    ///      ]
    ///    },
    ///    "maxConcurrent": {
    ///      "description": "Maximum number of parallel executions. Default is 10 if not specified",
    ///      "type": "integer",
    ///      "minimum": 1.0
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct StepParallel {
        ///Array of items to process in parallel. Can be a static array or a reference to a variable containing an array
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub items: ::std::option::Option<StepParallelItems>,
        ///Maximum number of parallel executions. Default is 10 if not specified
        #[serde(
            rename = "maxConcurrent",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub max_concurrent: ::std::option::Option<::std::num::NonZeroU64>,
    }
    impl ::std::default::Default for StepParallel {
        fn default() -> Self {
            Self {
                items: Default::default(),
                max_concurrent: Default::default(),
            }
        }
    }
    ///Array of items to process in parallel. Can be a static array or a reference to a variable containing an array
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Array of items to process in parallel. Can be a static array or a reference to a variable containing an array",
    ///  "oneOf": [
    ///    {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    {
    ///      "type": "string"
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(untagged)]
    pub enum StepParallelItems {
        Array(::std::vec::Vec<::std::string::String>),
        String(::std::string::String),
    }
    impl ::std::convert::From<::std::vec::Vec<::std::string::String>>
    for StepParallelItems {
        fn from(value: ::std::vec::Vec<::std::string::String>) -> Self {
            Self::Array(value)
        }
    }
    ///Router configuration for switch/case routing
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Router configuration for switch/case routing",
    ///  "type": "object",
    ///  "required": [
    ///    "routes",
    ///    "value"
    ///  ],
    ///  "properties": {
    ///    "routes": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "object",
    ///        "required": [
    ///          "pattern",
    ///          "targets"
    ///        ],
    ///        "properties": {
    ///          "targets": {
    ///            "description": "Step names to route to",
    ///            "type": "array",
    ///            "items": {
    ///              "type": "string",
    ///              "minLength": 1
    ///            },
    ///            "minItems": 1
    ///          }
    ///        }
    ///      },
    ///      "minItems": 1
    ///    },
    ///    "value": {
    ///      "description": "Expression to evaluate (e.g., '${STATUS}')",
    ///      "type": "string",
    ///      "minLength": 1
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct StepRouter {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub routes: ::std::vec::Vec<StepRouterRoutesItem>,
        ///Expression to evaluate (e.g., '${STATUS}')
        pub value: StepRouterValue,
    }
    ///`StepRouterRoutesItem`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "pattern",
    ///    "targets"
    ///  ],
    ///  "properties": {
    ///    "targets": {
    ///      "description": "Step names to route to",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string",
    ///        "minLength": 1
    ///      },
    ///      "minItems": 1
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct StepRouterRoutesItem {
        pub pattern: ::serde_json::Value,
        ///Step names to route to
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub targets: ::std::vec::Vec<StepRouterRoutesItemTargetsItem>,
    }
    ///`StepRouterRoutesItemTargetsItem`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct StepRouterRoutesItemTargetsItem(::std::string::String);
    impl ::std::ops::Deref for StepRouterRoutesItemTargetsItem {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<StepRouterRoutesItemTargetsItem>
    for ::std::string::String {
        fn from(value: StepRouterRoutesItemTargetsItem) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for StepRouterRoutesItemTargetsItem {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for StepRouterRoutesItemTargetsItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for StepRouterRoutesItemTargetsItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for StepRouterRoutesItemTargetsItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for StepRouterRoutesItemTargetsItem {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Expression to evaluate (e.g., '${STATUS}')
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Expression to evaluate (e.g., '${STATUS}')",
    ///  "type": "string",
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct StepRouterValue(::std::string::String);
    impl ::std::ops::Deref for StepRouterValue {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<StepRouterValue> for ::std::string::String {
        fn from(value: StepRouterValue) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for StepRouterValue {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for StepRouterValue {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for StepRouterValue {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for StepRouterValue {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for StepRouterValue {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///`Stream`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "format": "string",
    ///  "enum": [
    ///    "stdout",
    ///    "stderr"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum Stream {
        #[serde(rename = "stdout")]
        Stdout,
        #[serde(rename = "stderr")]
        Stderr,
    }
    impl ::std::fmt::Display for Stream {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Stdout => f.write_str("stdout"),
                Self::Stderr => f.write_str("stderr"),
            }
        }
    }
    impl ::std::str::FromStr for Stream {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "stdout" => Ok(Self::Stdout),
                "stderr" => Ok(Self::Stderr),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for Stream {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for Stream {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for Stream {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Metadata for a sub DAG-run
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Metadata for a sub DAG-run",
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunId"
    ///  ],
    ///  "properties": {
    ///    "dagName": {
    ///      "description": "Name of the executed sub-DAG. For chat tool calls, this is the tool DAG name.",
    ///      "type": "string"
    ///    },
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    },
    ///    "params": {
    ///      "description": "Parameters passed to the sub DAG-run in JSON format",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SubDagRun {
        ///Name of the executed sub-DAG. For chat tool calls, this is the tool DAG name.
        #[serde(
            rename = "dagName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_name: ::std::option::Option<::std::string::String>,
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
        ///Parameters passed to the sub DAG-run in JSON format
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub params: ::std::option::Option<::std::string::String>,
    }
    ///Detailed information for a sub DAG-run including timing and status
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Detailed information for a sub DAG-run including timing and status",
    ///  "type": "object",
    ///  "required": [
    ///    "dagRunId",
    ///    "startedAt",
    ///    "status",
    ///    "statusLabel"
    ///  ],
    ///  "properties": {
    ///    "dagName": {
    ///      "description": "Name of the executed sub-DAG. For chat tool calls, this is the tool DAG name.",
    ///      "type": "string"
    ///    },
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    },
    ///    "finishedAt": {
    ///      "description": "RFC 3339 timestamp when the sub DAG-run finished",
    ///      "type": "string"
    ///    },
    ///    "params": {
    ///      "description": "Parameters passed to the sub DAG-run in JSON format",
    ///      "type": "string"
    ///    },
    ///    "startedAt": {
    ///      "description": "RFC 3339 timestamp when the sub DAG-run started",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "$ref": "#/components/schemas/Status"
    ///    },
    ///    "statusLabel": {
    ///      "$ref": "#/components/schemas/StatusLabel"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SubDagRunDetail {
        ///Name of the executed sub-DAG. For chat tool calls, this is the tool DAG name.
        #[serde(
            rename = "dagName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_name: ::std::option::Option<::std::string::String>,
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
        ///RFC 3339 timestamp when the sub DAG-run finished
        #[serde(
            rename = "finishedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub finished_at: ::std::option::Option<::std::string::String>,
        ///Parameters passed to the sub DAG-run in JSON format
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub params: ::std::option::Option<::std::string::String>,
        ///RFC 3339 timestamp when the sub DAG-run started
        #[serde(rename = "startedAt")]
        pub started_at: ::std::string::String,
        pub status: Status,
        #[serde(rename = "statusLabel")]
        pub status_label: StatusLabel,
    }
    ///Generic success response
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Generic success response",
    ///  "type": "object",
    ///  "required": [
    ///    "message"
    ///  ],
    ///  "properties": {
    ///    "message": {
    ///      "description": "Success message",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SuccessResponse {
        ///Success message
        pub message: ::std::string::String,
    }
    ///Git authentication configuration
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Git authentication configuration",
    ///  "type": "object",
    ///  "required": [
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "sshKeyPath": {
    ///      "description": "Path to SSH private key",
    ///      "type": "string"
    ///    },
    ///    "token": {
    ///      "description": "Personal access token (write-only)",
    ///      "writeOnly": true,
    ///      "type": "string"
    ///    },
    ///    "type": {
    ///      "type": "string",
    ///      "enum": [
    ///        "token",
    ///        "ssh"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncAuthConfig {
        ///Path to SSH private key
        #[serde(
            rename = "sshKeyPath",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub ssh_key_path: ::std::option::Option<::std::string::String>,
        ///Personal access token (write-only)
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub token: ::std::option::Option<::std::string::String>,
        #[serde(rename = "type")]
        pub type_: SyncAuthConfigType,
    }
    ///`SyncAuthConfigType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "token",
    ///    "ssh"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum SyncAuthConfigType {
        #[serde(rename = "token")]
        Token,
        #[serde(rename = "ssh")]
        Ssh,
    }
    impl ::std::fmt::Display for SyncAuthConfigType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Token => f.write_str("token"),
                Self::Ssh => f.write_str("ssh"),
            }
        }
    }
    impl ::std::str::FromStr for SyncAuthConfigType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "token" => Ok(Self::Token),
                "ssh" => Ok(Self::Ssh),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for SyncAuthConfigType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for SyncAuthConfigType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for SyncAuthConfigType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Auto-sync configuration
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Auto-sync configuration",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "interval",
    ///    "onStartup"
    ///  ],
    ///  "properties": {
    ///    "enabled": {
    ///      "type": "boolean"
    ///    },
    ///    "interval": {
    ///      "description": "Sync interval in seconds",
    ///      "type": "integer"
    ///    },
    ///    "onStartup": {
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncAutoSyncConfig {
        pub enabled: bool,
        ///Sync interval in seconds
        pub interval: i64,
        #[serde(rename = "onStartup")]
        pub on_startup: bool,
    }
    ///Commit configuration
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Commit configuration",
    ///  "type": "object",
    ///  "properties": {
    ///    "authorEmail": {
    ///      "type": "string"
    ///    },
    ///    "authorName": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncCommitConfig {
        #[serde(
            rename = "authorEmail",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub author_email: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "authorName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub author_name: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for SyncCommitConfig {
        fn default() -> Self {
            Self {
                author_email: Default::default(),
                author_name: Default::default(),
            }
        }
    }
    ///Git sync configuration
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Git sync configuration",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled"
    ///  ],
    ///  "properties": {
    ///    "auth": {
    ///      "$ref": "#/components/schemas/SyncAuthConfig"
    ///    },
    ///    "autoSync": {
    ///      "$ref": "#/components/schemas/SyncAutoSyncConfig"
    ///    },
    ///    "branch": {
    ///      "type": "string"
    ///    },
    ///    "commit": {
    ///      "$ref": "#/components/schemas/SyncCommitConfig"
    ///    },
    ///    "enabled": {
    ///      "type": "boolean"
    ///    },
    ///    "path": {
    ///      "type": "string"
    ///    },
    ///    "pushEnabled": {
    ///      "type": "boolean"
    ///    },
    ///    "repository": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncConfigResponse {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub auth: ::std::option::Option<SyncAuthConfig>,
        #[serde(
            rename = "autoSync",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub auto_sync: ::std::option::Option<SyncAutoSyncConfig>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub branch: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub commit: ::std::option::Option<SyncCommitConfig>,
        pub enabled: bool,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub path: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "pushEnabled",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub push_enabled: ::std::option::Option<bool>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub repository: ::std::option::Option<::std::string::String>,
    }
    ///Request to update Git sync configuration
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to update Git sync configuration",
    ///  "type": "object",
    ///  "properties": {
    ///    "auth": {
    ///      "$ref": "#/components/schemas/SyncAuthConfig"
    ///    },
    ///    "autoSync": {
    ///      "$ref": "#/components/schemas/SyncAutoSyncConfig"
    ///    },
    ///    "branch": {
    ///      "type": "string"
    ///    },
    ///    "commit": {
    ///      "$ref": "#/components/schemas/SyncCommitConfig"
    ///    },
    ///    "enabled": {
    ///      "type": "boolean"
    ///    },
    ///    "path": {
    ///      "type": "string"
    ///    },
    ///    "pushEnabled": {
    ///      "type": "boolean"
    ///    },
    ///    "repository": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncConfigUpdateRequest {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub auth: ::std::option::Option<SyncAuthConfig>,
        #[serde(
            rename = "autoSync",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub auto_sync: ::std::option::Option<SyncAutoSyncConfig>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub branch: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub commit: ::std::option::Option<SyncCommitConfig>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub enabled: ::std::option::Option<bool>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub path: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "pushEnabled",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub push_enabled: ::std::option::Option<bool>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub repository: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for SyncConfigUpdateRequest {
        fn default() -> Self {
            Self {
                auth: Default::default(),
                auto_sync: Default::default(),
                branch: Default::default(),
                commit: Default::default(),
                enabled: Default::default(),
                path: Default::default(),
                push_enabled: Default::default(),
                repository: Default::default(),
            }
        }
    }
    ///Response when a conflict is detected
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response when a conflict is detected",
    ///  "type": "object",
    ///  "required": [
    ///    "itemId",
    ///    "message"
    ///  ],
    ///  "properties": {
    ///    "itemId": {
    ///      "type": "string"
    ///    },
    ///    "message": {
    ///      "type": "string"
    ///    },
    ///    "remoteAuthor": {
    ///      "type": "string"
    ///    },
    ///    "remoteCommit": {
    ///      "type": "string"
    ///    },
    ///    "remoteMessage": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncConflictResponse {
        #[serde(rename = "itemId")]
        pub item_id: ::std::string::String,
        pub message: ::std::string::String,
        #[serde(
            rename = "remoteAuthor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub remote_author: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "remoteCommit",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub remote_commit: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "remoteMessage",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub remote_message: ::std::option::Option<::std::string::String>,
    }
    ///Result of connection test
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Result of connection test",
    ///  "type": "object",
    ///  "required": [
    ///    "success"
    ///  ],
    ///  "properties": {
    ///    "error": {
    ///      "type": "string"
    ///    },
    ///    "message": {
    ///      "type": "string"
    ///    },
    ///    "success": {
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncConnectionTestResponse {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub error: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub message: ::std::option::Option<::std::string::String>,
        pub success: bool,
    }
    ///Request to delete selected sync items
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to delete selected sync items",
    ///  "type": "object",
    ///  "required": [
    ///    "itemIds"
    ///  ],
    ///  "properties": {
    ///    "force": {
    ///      "description": "Force delete items with local modifications or conflicts",
    ///      "type": "boolean"
    ///    },
    ///    "itemIds": {
    ///      "description": "Sync item IDs to delete",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string",
    ///        "minLength": 1
    ///      },
    ///      "minItems": 1,
    ///      "uniqueItems": true
    ///    },
    ///    "message": {
    ///      "description": "Commit message for the deletion",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncDeleteBatchRequest {
        ///Force delete items with local modifications or conflicts
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub force: ::std::option::Option<bool>,
        ///Sync item IDs to delete
        #[serde(rename = "itemIds")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub item_ids: Vec<SyncDeleteBatchRequestItemIdsItem>,
        ///Commit message for the deletion
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub message: ::std::option::Option<::std::string::String>,
    }
    ///`SyncDeleteBatchRequestItemIdsItem`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct SyncDeleteBatchRequestItemIdsItem(::std::string::String);
    impl ::std::ops::Deref for SyncDeleteBatchRequestItemIdsItem {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<SyncDeleteBatchRequestItemIdsItem>
    for ::std::string::String {
        fn from(value: SyncDeleteBatchRequestItemIdsItem) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for SyncDeleteBatchRequestItemIdsItem {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for SyncDeleteBatchRequestItemIdsItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for SyncDeleteBatchRequestItemIdsItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for SyncDeleteBatchRequestItemIdsItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for SyncDeleteBatchRequestItemIdsItem {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Error during sync operation
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Error during sync operation",
    ///  "type": "object",
    ///  "required": [
    ///    "message"
    ///  ],
    ///  "properties": {
    ///    "itemId": {
    ///      "type": "string"
    ///    },
    ///    "message": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncError {
        #[serde(
            rename = "itemId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub item_id: ::std::option::Option<::std::string::String>,
        pub message: ::std::string::String,
    }
    ///Sync state for a single item
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Sync state for a single item",
    ///  "type": "object",
    ///  "required": [
    ///    "displayName",
    ///    "filePath",
    ///    "itemId",
    ///    "kind",
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "baseCommit": {
    ///      "description": "Commit hash when last synced",
    ///      "type": "string"
    ///    },
    ///    "conflictDetectedAt": {
    ///      "description": "When the conflict was detected",
    ///      "type": "string"
    ///    },
    ///    "displayName": {
    ///      "description": "Display-friendly item name",
    ///      "type": "string"
    ///    },
    ///    "filePath": {
    ///      "description": "Relative file path with extension",
    ///      "type": "string"
    ///    },
    ///    "itemId": {
    ///      "description": "Stable sync item identifier (file path without extension)",
    ///      "type": "string"
    ///    },
    ///    "kind": {
    ///      "$ref": "#/components/schemas/SyncItemKind"
    ///    },
    ///    "lastSyncedAt": {
    ///      "description": "When the item was last synced",
    ///      "type": "string"
    ///    },
    ///    "lastSyncedHash": {
    ///      "description": "Content hash when last synced",
    ///      "type": "string"
    ///    },
    ///    "localHash": {
    ///      "description": "Current local content hash",
    ///      "type": "string"
    ///    },
    ///    "missingAt": {
    ///      "description": "When the file was first detected as missing",
    ///      "type": "string"
    ///    },
    ///    "modifiedAt": {
    ///      "description": "When the item was last modified locally",
    ///      "type": "string"
    ///    },
    ///    "previousStatus": {
    ///      "$ref": "#/components/schemas/SyncStatus"
    ///    },
    ///    "remoteAuthor": {
    ///      "description": "Author of the remote commit (for conflicts)",
    ///      "type": "string"
    ///    },
    ///    "remoteCommit": {
    ///      "description": "Remote commit hash (for conflicts)",
    ///      "type": "string"
    ///    },
    ///    "remoteMessage": {
    ///      "description": "Message of the remote commit (for conflicts)",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "$ref": "#/components/schemas/SyncStatus"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncItem {
        ///Commit hash when last synced
        #[serde(
            rename = "baseCommit",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub base_commit: ::std::option::Option<::std::string::String>,
        ///When the conflict was detected
        #[serde(
            rename = "conflictDetectedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub conflict_detected_at: ::std::option::Option<::std::string::String>,
        ///Display-friendly item name
        #[serde(rename = "displayName")]
        pub display_name: ::std::string::String,
        ///Relative file path with extension
        #[serde(rename = "filePath")]
        pub file_path: ::std::string::String,
        ///Stable sync item identifier (file path without extension)
        #[serde(rename = "itemId")]
        pub item_id: ::std::string::String,
        pub kind: SyncItemKind,
        ///When the item was last synced
        #[serde(
            rename = "lastSyncedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub last_synced_at: ::std::option::Option<::std::string::String>,
        ///Content hash when last synced
        #[serde(
            rename = "lastSyncedHash",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub last_synced_hash: ::std::option::Option<::std::string::String>,
        ///Current local content hash
        #[serde(
            rename = "localHash",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub local_hash: ::std::option::Option<::std::string::String>,
        ///When the file was first detected as missing
        #[serde(
            rename = "missingAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub missing_at: ::std::option::Option<::std::string::String>,
        ///When the item was last modified locally
        #[serde(
            rename = "modifiedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub modified_at: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "previousStatus",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub previous_status: ::std::option::Option<SyncStatus>,
        ///Author of the remote commit (for conflicts)
        #[serde(
            rename = "remoteAuthor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub remote_author: ::std::option::Option<::std::string::String>,
        ///Remote commit hash (for conflicts)
        #[serde(
            rename = "remoteCommit",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub remote_commit: ::std::option::Option<::std::string::String>,
        ///Message of the remote commit (for conflicts)
        #[serde(
            rename = "remoteMessage",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub remote_message: ::std::option::Option<::std::string::String>,
        pub status: SyncStatus,
    }
    ///Diff between local and remote versions of a sync item
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Diff between local and remote versions of a sync item",
    ///  "type": "object",
    ///  "required": [
    ///    "filePath",
    ///    "itemId",
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "binary": {
    ///      "description": "True for binary items; content fields are omitted and sizes are reported instead",
    ///      "type": "boolean"
    ///    },
    ///    "filePath": {
    ///      "description": "Relative file path with extension",
    ///      "type": "string"
    ///    },
    ///    "itemId": {
    ///      "description": "The sync item identifier",
    ///      "type": "string"
    ///    },
    ///    "kind": {
    ///      "$ref": "#/components/schemas/SyncItemKind"
    ///    },
    ///    "localContent": {
    ///      "description": "Current local file content. Omitted for binary items.",
    ///      "type": "string"
    ///    },
    ///    "localSize": {
    ///      "description": "Local file size in bytes. Only set for binary items.",
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "remoteAuthor": {
    ///      "description": "Author of the remote commit",
    ///      "type": "string"
    ///    },
    ///    "remoteCommit": {
    ///      "description": "Commit hash being compared against",
    ///      "type": "string"
    ///    },
    ///    "remoteContent": {
    ///      "description": "Content from remote repository. Omitted for binary items.",
    ///      "type": "string"
    ///    },
    ///    "remoteMessage": {
    ///      "description": "Commit message of the remote version",
    ///      "type": "string"
    ///    },
    ///    "remoteSize": {
    ///      "description": "Remote file size in bytes. Only set for binary items.",
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "status": {
    ///      "$ref": "#/components/schemas/SyncStatus"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncItemDiffResponse {
        ///True for binary items; content fields are omitted and sizes are reported instead
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub binary: ::std::option::Option<bool>,
        ///Relative file path with extension
        #[serde(rename = "filePath")]
        pub file_path: ::std::string::String,
        ///The sync item identifier
        #[serde(rename = "itemId")]
        pub item_id: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub kind: ::std::option::Option<SyncItemKind>,
        ///Current local file content. Omitted for binary items.
        #[serde(
            rename = "localContent",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub local_content: ::std::option::Option<::std::string::String>,
        ///Local file size in bytes. Only set for binary items.
        #[serde(
            rename = "localSize",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub local_size: ::std::option::Option<i64>,
        ///Author of the remote commit
        #[serde(
            rename = "remoteAuthor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub remote_author: ::std::option::Option<::std::string::String>,
        ///Commit hash being compared against
        #[serde(
            rename = "remoteCommit",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub remote_commit: ::std::option::Option<::std::string::String>,
        ///Content from remote repository. Omitted for binary items.
        #[serde(
            rename = "remoteContent",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub remote_content: ::std::option::Option<::std::string::String>,
        ///Commit message of the remote version
        #[serde(
            rename = "remoteMessage",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub remote_message: ::std::option::Option<::std::string::String>,
        ///Remote file size in bytes. Only set for binary items.
        #[serde(
            rename = "remoteSize",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub remote_size: ::std::option::Option<i64>,
        pub status: SyncStatus,
    }
    ///Type of sync item
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Type of sync item",
    ///  "type": "string",
    ///  "enum": [
    ///    "dag",
    ///    "doc",
    ///    "doc-asset"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum SyncItemKind {
        #[serde(rename = "dag")]
        Dag,
        #[serde(rename = "doc")]
        Doc,
        #[serde(rename = "doc-asset")]
        DocAsset,
    }
    impl ::std::fmt::Display for SyncItemKind {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Dag => f.write_str("dag"),
                Self::Doc => f.write_str("doc"),
                Self::DocAsset => f.write_str("doc-asset"),
            }
        }
    }
    impl ::std::str::FromStr for SyncItemKind {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "dag" => Ok(Self::Dag),
                "doc" => Ok(Self::Doc),
                "doc-asset" => Ok(Self::DocAsset),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for SyncItemKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for SyncItemKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for SyncItemKind {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Request to publish selected sync items
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to publish selected sync items",
    ///  "type": "object",
    ///  "properties": {
    ///    "itemIds": {
    ///      "description": "Sync item IDs to publish. If omitted, all modified or untracked items are published.",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "message": {
    ///      "description": "Commit message",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncPublishAllRequest {
        ///Sync item IDs to publish. If omitted, all modified or untracked items are published.
        #[serde(
            rename = "itemIds",
            default,
            skip_serializing_if = "::std::vec::Vec::is_empty"
        )]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub item_ids: ::std::vec::Vec<::std::string::String>,
        ///Commit message
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub message: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for SyncPublishAllRequest {
        fn default() -> Self {
            Self {
                item_ids: Default::default(),
                message: Default::default(),
            }
        }
    }
    ///Request to publish a sync item
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to publish a sync item",
    ///  "type": "object",
    ///  "properties": {
    ///    "force": {
    ///      "description": "Force publish even with conflicts",
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "message": {
    ///      "description": "Commit message",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncPublishRequest {
        ///Force publish even with conflicts
        #[serde(default)]
        pub force: bool,
        ///Commit message
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub message: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for SyncPublishRequest {
        fn default() -> Self {
            Self {
                force: Default::default(),
                message: Default::default(),
            }
        }
    }
    ///Result of a sync operation
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Result of a sync operation",
    ///  "type": "object",
    ///  "required": [
    ///    "success",
    ///    "timestamp"
    ///  ],
    ///  "properties": {
    ///    "conflicts": {
    ///      "description": "Sync item IDs with conflicts",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "errors": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/SyncError"
    ///      }
    ///    },
    ///    "message": {
    ///      "type": "string"
    ///    },
    ///    "modified": {
    ///      "description": "Sync item IDs that were modified",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "success": {
    ///      "type": "boolean"
    ///    },
    ///    "synced": {
    ///      "description": "Sync item IDs that were synced",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "timestamp": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncResultResponse {
        ///Sync item IDs with conflicts
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub conflicts: ::std::vec::Vec<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub errors: ::std::vec::Vec<SyncError>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub message: ::std::option::Option<::std::string::String>,
        ///Sync item IDs that were modified
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub modified: ::std::vec::Vec<::std::string::String>,
        pub success: bool,
        ///Sync item IDs that were synced
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub synced: ::std::vec::Vec<::std::string::String>,
        pub timestamp: ::std::string::String,
    }
    ///Sync status of an item
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Sync status of an item",
    ///  "type": "string",
    ///  "enum": [
    ///    "synced",
    ///    "modified",
    ///    "untracked",
    ///    "conflict",
    ///    "missing"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum SyncStatus {
        #[serde(rename = "synced")]
        Synced,
        #[serde(rename = "modified")]
        Modified,
        #[serde(rename = "untracked")]
        Untracked,
        #[serde(rename = "conflict")]
        Conflict,
        #[serde(rename = "missing")]
        Missing,
    }
    impl ::std::fmt::Display for SyncStatus {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Synced => f.write_str("synced"),
                Self::Modified => f.write_str("modified"),
                Self::Untracked => f.write_str("untracked"),
                Self::Conflict => f.write_str("conflict"),
                Self::Missing => f.write_str("missing"),
            }
        }
    }
    impl ::std::str::FromStr for SyncStatus {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "synced" => Ok(Self::Synced),
                "modified" => Ok(Self::Modified),
                "untracked" => Ok(Self::Untracked),
                "conflict" => Ok(Self::Conflict),
                "missing" => Ok(Self::Missing),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for SyncStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for SyncStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for SyncStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Counts of items in each sync status
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Counts of items in each sync status",
    ///  "type": "object",
    ///  "required": [
    ///    "conflict",
    ///    "missing",
    ///    "modified",
    ///    "synced",
    ///    "untracked"
    ///  ],
    ///  "properties": {
    ///    "conflict": {
    ///      "type": "integer"
    ///    },
    ///    "missing": {
    ///      "type": "integer"
    ///    },
    ///    "modified": {
    ///      "type": "integer"
    ///    },
    ///    "synced": {
    ///      "type": "integer"
    ///    },
    ///    "untracked": {
    ///      "type": "integer"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncStatusCounts {
        pub conflict: i64,
        pub missing: i64,
        pub modified: i64,
        pub synced: i64,
        pub untracked: i64,
    }
    ///Overall Git sync status
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Overall Git sync status",
    ///  "type": "object",
    ///  "required": [
    ///    "counts",
    ///    "enabled",
    ///    "items",
    ///    "summary"
    ///  ],
    ///  "properties": {
    ///    "branch": {
    ///      "description": "Branch being synced",
    ///      "type": "string"
    ///    },
    ///    "counts": {
    ///      "$ref": "#/components/schemas/SyncStatusCounts"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether Git sync is enabled",
    ///      "type": "boolean"
    ///    },
    ///    "items": {
    ///      "description": "Sync state for each item",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/SyncItem"
    ///      }
    ///    },
    ///    "lastError": {
    ///      "description": "Error message from last failed sync",
    ///      "type": "string"
    ///    },
    ///    "lastSyncAt": {
    ///      "description": "When the last sync occurred",
    ///      "type": "string"
    ///    },
    ///    "lastSyncCommit": {
    ///      "description": "Commit hash of last sync",
    ///      "type": "string"
    ///    },
    ///    "lastSyncStatus": {
    ///      "description": "Status of last sync (success/error)",
    ///      "type": "string"
    ///    },
    ///    "repository": {
    ///      "description": "Repository URL",
    ///      "type": "string"
    ///    },
    ///    "summary": {
    ///      "$ref": "#/components/schemas/SyncSummary"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct SyncStatusResponse {
        ///Branch being synced
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub branch: ::std::option::Option<::std::string::String>,
        pub counts: SyncStatusCounts,
        ///Whether Git sync is enabled
        pub enabled: bool,
        ///Sync state for each item
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub items: ::std::vec::Vec<SyncItem>,
        ///Error message from last failed sync
        #[serde(
            rename = "lastError",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub last_error: ::std::option::Option<::std::string::String>,
        ///When the last sync occurred
        #[serde(
            rename = "lastSyncAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub last_sync_at: ::std::option::Option<::std::string::String>,
        ///Commit hash of last sync
        #[serde(
            rename = "lastSyncCommit",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub last_sync_commit: ::std::option::Option<::std::string::String>,
        ///Status of last sync (success/error)
        #[serde(
            rename = "lastSyncStatus",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub last_sync_status: ::std::option::Option<::std::string::String>,
        ///Repository URL
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub repository: ::std::option::Option<::std::string::String>,
        pub summary: SyncSummary,
    }
    ///Summary status for the sync badge
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Summary status for the sync badge",
    ///  "type": "string",
    ///  "enum": [
    ///    "synced",
    ///    "pending",
    ///    "conflict",
    ///    "missing",
    ///    "error"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum SyncSummary {
        #[serde(rename = "synced")]
        Synced,
        #[serde(rename = "pending")]
        Pending,
        #[serde(rename = "conflict")]
        Conflict,
        #[serde(rename = "missing")]
        Missing,
        #[serde(rename = "error")]
        Error,
    }
    impl ::std::fmt::Display for SyncSummary {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Synced => f.write_str("synced"),
                Self::Pending => f.write_str("pending"),
                Self::Conflict => f.write_str("conflict"),
                Self::Missing => f.write_str("missing"),
                Self::Error => f.write_str("error"),
            }
        }
    }
    impl ::std::str::FromStr for SyncSummary {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "synced" => Ok(Self::Synced),
                "pending" => Ok(Self::Pending),
                "conflict" => Ok(Self::Conflict),
                "missing" => Ok(Self::Missing),
                "error" => Ok(Self::Error),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for SyncSummary {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for SyncSummary {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for SyncSummary {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Deprecated alias for Labels. Additional labels to apply to the DAG-run (format: key=value or key-only). Merged with labels defined in the DAG spec. Mutually exclusive with `labels`; the server returns HTTP 400 if both are set.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Deprecated alias for Labels. Additional labels to apply to the DAG-run (format: key=value or key-only). Merged with labels defined in the DAG spec. Mutually exclusive with `labels`; the server returns HTTP 400 if both are set.",
    ///  "deprecated": true,
    ///  "type": "array",
    ///  "items": {
    ///    "type": "string"
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(transparent)]
    pub struct Tags(pub ::std::vec::Vec<::std::string::String>);
    impl ::std::ops::Deref for Tags {
        type Target = ::std::vec::Vec<::std::string::String>;
        fn deref(&self) -> &::std::vec::Vec<::std::string::String> {
            &self.0
        }
    }
    impl ::std::convert::From<Tags> for ::std::vec::Vec<::std::string::String> {
        fn from(value: Tags) -> Self {
            value.0
        }
    }
    impl ::std::convert::From<::std::vec::Vec<::std::string::String>> for Tags {
        fn from(value: ::std::vec::Vec<::std::string::String>) -> Self {
            Self(value)
        }
    }
    ///Request to send a test notification
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to send a test notification",
    ///  "type": "object",
    ///  "properties": {
    ///    "eventType": {
    ///      "$ref": "#/components/schemas/NotificationEventType"
    ///    },
    ///    "targetId": {
    ///      "description": "Optional DAG-local target ID, subscription ID, or channel ID. When omitted, every enabled target and subscription is tested.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct TestDagNotificationRequest {
        #[serde(
            rename = "eventType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub event_type: ::std::option::Option<NotificationEventType>,
        ///Optional DAG-local target ID, subscription ID, or channel ID. When omitted, every enabled target and subscription is tested.
        #[serde(
            rename = "targetId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub target_id: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for TestDagNotificationRequest {
        fn default() -> Self {
            Self {
                event_type: Default::default(),
                target_id: Default::default(),
            }
        }
    }
    ///Result of test notification delivery
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Result of test notification delivery",
    ///  "type": "object",
    ///  "required": [
    ///    "results"
    ///  ],
    ///  "properties": {
    ///    "results": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/TestDAGNotificationResult"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct TestDagNotificationResponse {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub results: ::std::vec::Vec<TestDagNotificationResult>,
    }
    ///Delivery result for one notification target
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Delivery result for one notification target",
    ///  "type": "object",
    ///  "required": [
    ///    "delivered",
    ///    "provider",
    ///    "targetId",
    ///    "targetName"
    ///  ],
    ///  "properties": {
    ///    "delivered": {
    ///      "type": "boolean"
    ///    },
    ///    "error": {
    ///      "type": "string"
    ///    },
    ///    "provider": {
    ///      "$ref": "#/components/schemas/NotificationProviderType"
    ///    },
    ///    "targetId": {
    ///      "type": "string"
    ///    },
    ///    "targetName": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct TestDagNotificationResult {
        pub delivered: bool,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub error: ::std::option::Option<::std::string::String>,
        pub provider: NotificationProviderType,
        #[serde(rename = "targetId")]
        pub target_id: ::std::string::String,
        #[serde(rename = "targetName")]
        pub target_name: ::std::string::String,
    }
    ///Result of test incident delivery
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Result of test incident delivery",
    ///  "type": "object",
    ///  "required": [
    ///    "result"
    ///  ],
    ///  "properties": {
    ///    "result": {
    ///      "$ref": "#/components/schemas/TestIncidentProviderResult"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct TestIncidentProviderResponse {
        pub result: TestIncidentProviderResult,
    }
    ///Delivery result for one incident provider
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Delivery result for one incident provider",
    ///  "type": "object",
    ///  "required": [
    ///    "delivered",
    ///    "providerId",
    ///    "providerName",
    ///    "providerType"
    ///  ],
    ///  "properties": {
    ///    "delivered": {
    ///      "type": "boolean"
    ///    },
    ///    "error": {
    ///      "type": "string"
    ///    },
    ///    "providerId": {
    ///      "type": "string"
    ///    },
    ///    "providerName": {
    ///      "type": "string"
    ///    },
    ///    "providerType": {
    ///      "$ref": "#/components/schemas/IncidentProviderType"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct TestIncidentProviderResult {
        pub delivered: bool,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub error: ::std::option::Option<::std::string::String>,
        #[serde(rename = "providerId")]
        pub provider_id: ::std::string::String,
        #[serde(rename = "providerName")]
        pub provider_name: ::std::string::String,
        #[serde(rename = "providerType")]
        pub provider_type: IncidentProviderType,
    }
    ///`TestRemoteNodeConnectionResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "success"
    ///  ],
    ///  "properties": {
    ///    "error": {
    ///      "type": "string"
    ///    },
    ///    "message": {
    ///      "type": "string"
    ///    },
    ///    "success": {
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct TestRemoteNodeConnectionResponse {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub error: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub message: ::std::option::Option<::std::string::String>,
        pub success: bool,
    }
    ///Timeout error response with DAG run tracking information
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Timeout error response with DAG run tracking information",
    ///  "type": "object",
    ///  "allOf": [
    ///    {
    ///      "$ref": "#/components/schemas/Error"
    ///    },
    ///    {
    ///      "type": "object",
    ///      "required": [
    ///        "dagRunId"
    ///      ],
    ///      "properties": {
    ///        "dagRunId": {
    ///          "allOf": [
    ///            {
    ///              "$ref": "#/components/schemas/DAGRunId"
    ///            },
    ///            {
    ///              "description": "ID of the DAG run that continues executing in background"
    ///            }
    ///          ]
    ///        }
    ///      }
    ///    }
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct TimeoutError {
        pub code: ErrorCode,
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
        ///Additional error details
        #[serde(default, skip_serializing_if = "::serde_json::Map::is_empty")]
        pub details: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
        ///Short error message
        pub message: ::std::string::String,
    }
    ///A tool definition that was available to the LLM
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A tool definition that was available to the LLM",
    ///  "type": "object",
    ///  "required": [
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "description": {
    ///      "description": "Description of what the tool does",
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "description": "Name of the tool",
    ///      "type": "string"
    ///    },
    ///    "parameters": {
    ///      "description": "JSON Schema describing the tool's parameters",
    ///      "type": "object",
    ///      "additionalProperties": true
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ToolDefinition {
        ///Description of what the tool does
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        ///Name of the tool
        pub name: ::std::string::String,
        ///JSON Schema describing the tool's parameters
        #[serde(default, skip_serializing_if = "::serde_json::Map::is_empty")]
        pub parameters: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    }
    ///How the DAG-run was initiated
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "How the DAG-run was initiated",
    ///  "type": "string",
    ///  "enum": [
    ///    "unknown",
    ///    "scheduler",
    ///    "manual",
    ///    "webhook",
    ///    "subdag",
    ///    "retry",
    ///    "catchup"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum TriggerType {
        #[serde(rename = "unknown")]
        Unknown,
        #[serde(rename = "scheduler")]
        Scheduler,
        #[serde(rename = "manual")]
        Manual,
        #[serde(rename = "webhook")]
        Webhook,
        #[serde(rename = "subdag")]
        Subdag,
        #[serde(rename = "retry")]
        Retry,
        #[serde(rename = "catchup")]
        Catchup,
    }
    impl ::std::fmt::Display for TriggerType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Unknown => f.write_str("unknown"),
                Self::Scheduler => f.write_str("scheduler"),
                Self::Manual => f.write_str("manual"),
                Self::Webhook => f.write_str("webhook"),
                Self::Subdag => f.write_str("subdag"),
                Self::Retry => f.write_str("retry"),
                Self::Catchup => f.write_str("catchup"),
            }
        }
    }
    impl ::std::str::FromStr for TriggerType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "unknown" => Ok(Self::Unknown),
                "scheduler" => Ok(Self::Scheduler),
                "manual" => Ok(Self::Manual),
                "webhook" => Ok(Self::Webhook),
                "subdag" => Ok(Self::Subdag),
                "retry" => Ok(Self::Retry),
                "catchup" => Ok(Self::Catchup),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for TriggerType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for TriggerType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for TriggerType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Response containing tunnel service status
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response containing tunnel service status",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "enabled": {
    ///      "description": "Whether tunneling is enabled in configuration",
    ///      "type": "boolean"
    ///    },
    ///    "error": {
    ///      "description": "Error message if tunnel failed",
    ///      "type": "string"
    ///    },
    ///    "isPublic": {
    ///      "description": "Whether the tunnel provides public internet access",
    ///      "type": "boolean"
    ///    },
    ///    "mode": {
    ///      "description": "Tunnel mode (e.g., 'direct' or 'funnel' for Tailscale)",
    ///      "type": "string"
    ///    },
    ///    "provider": {
    ///      "description": "The tunnel provider in use",
    ///      "type": "string",
    ///      "enum": [
    ///        "tailscale"
    ///      ]
    ///    },
    ///    "publicUrl": {
    ///      "description": "The public URL provided by the tunnel",
    ///      "type": "string"
    ///    },
    ///    "startedAt": {
    ///      "description": "RFC3339 timestamp when tunnel connected",
    ///      "type": "string"
    ///    },
    ///    "status": {
    ///      "description": "Current status of the tunnel",
    ///      "type": "string",
    ///      "enum": [
    ///        "disabled",
    ///        "connecting",
    ///        "connected",
    ///        "reconnecting",
    ///        "error"
    ///      ]
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct TunnelStatusResponse {
        ///Whether tunneling is enabled in configuration
        pub enabled: bool,
        ///Error message if tunnel failed
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub error: ::std::option::Option<::std::string::String>,
        ///Whether the tunnel provides public internet access
        #[serde(
            rename = "isPublic",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub is_public: ::std::option::Option<bool>,
        ///Tunnel mode (e.g., 'direct' or 'funnel' for Tailscale)
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub mode: ::std::option::Option<::std::string::String>,
        ///The tunnel provider in use
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub provider: ::std::option::Option<TunnelStatusResponseProvider>,
        ///The public URL provided by the tunnel
        #[serde(
            rename = "publicUrl",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub public_url: ::std::option::Option<::std::string::String>,
        ///RFC3339 timestamp when tunnel connected
        #[serde(
            rename = "startedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub started_at: ::std::option::Option<::std::string::String>,
        ///Current status of the tunnel
        pub status: TunnelStatusResponseStatus,
    }
    ///The tunnel provider in use
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "The tunnel provider in use",
    ///  "type": "string",
    ///  "enum": [
    ///    "tailscale"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum TunnelStatusResponseProvider {
        #[serde(rename = "tailscale")]
        Tailscale,
    }
    impl ::std::fmt::Display for TunnelStatusResponseProvider {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Tailscale => f.write_str("tailscale"),
            }
        }
    }
    impl ::std::str::FromStr for TunnelStatusResponseProvider {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "tailscale" => Ok(Self::Tailscale),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for TunnelStatusResponseProvider {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for TunnelStatusResponseProvider {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for TunnelStatusResponseProvider {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Current status of the tunnel
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Current status of the tunnel",
    ///  "type": "string",
    ///  "enum": [
    ///    "disabled",
    ///    "connecting",
    ///    "connected",
    ///    "reconnecting",
    ///    "error"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum TunnelStatusResponseStatus {
        #[serde(rename = "disabled")]
        Disabled,
        #[serde(rename = "connecting")]
        Connecting,
        #[serde(rename = "connected")]
        Connected,
        #[serde(rename = "reconnecting")]
        Reconnecting,
        #[serde(rename = "error")]
        Error,
    }
    impl ::std::fmt::Display for TunnelStatusResponseStatus {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Disabled => f.write_str("disabled"),
                Self::Connecting => f.write_str("connecting"),
                Self::Connected => f.write_str("connected"),
                Self::Reconnecting => f.write_str("reconnecting"),
                Self::Error => f.write_str("error"),
            }
        }
    }
    impl ::std::str::FromStr for TunnelStatusResponseStatus {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "disabled" => Ok(Self::Disabled),
                "connecting" => Ok(Self::Connecting),
                "connected" => Ok(Self::Connected),
                "reconnecting" => Ok(Self::Reconnecting),
                "error" => Ok(Self::Error),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for TunnelStatusResponseStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for TunnelStatusResponseStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for TunnelStatusResponseStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Unix timestamp in seconds
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Unix timestamp in seconds",
    ///  "examples": [
    ///    1672531199
    ///  ],
    ///  "type": "integer",
    ///  "format": "int64"
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    #[serde(transparent)]
    pub struct UnixTimestamp(pub i64);
    impl ::std::ops::Deref for UnixTimestamp {
        type Target = i64;
        fn deref(&self) -> &i64 {
            &self.0
        }
    }
    impl ::std::convert::From<UnixTimestamp> for i64 {
        fn from(value: UnixTimestamp) -> Self {
            value.0
        }
    }
    impl ::std::convert::From<i64> for UnixTimestamp {
        fn from(value: i64) -> Self {
            Self(value)
        }
    }
    impl ::std::str::FromStr for UnixTimestamp {
        type Err = <i64 as ::std::str::FromStr>::Err;
        fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
            Ok(Self(value.parse()?))
        }
    }
    impl ::std::convert::TryFrom<&str> for UnixTimestamp {
        type Error = <i64 as ::std::str::FromStr>::Err;
        fn try_from(value: &str) -> ::std::result::Result<Self, Self::Error> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<String> for UnixTimestamp {
        type Error = <i64 as ::std::str::FromStr>::Err;
        fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
            value.parse()
        }
    }
    impl ::std::fmt::Display for UnixTimestamp {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            self.0.fmt(f)
        }
    }
    ///Update API key request
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Update API key request",
    ///  "type": "object",
    ///  "properties": {
    ///    "allowedSurfaces": {
    ///      "description": "Interfaces where this API key may be accepted",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string",
    ///        "enum": [
    ///          "rest_api",
    ///          "mcp"
    ///        ]
    ///      },
    ///      "minItems": 1,
    ///      "uniqueItems": true
    ///    },
    ///    "attributionClass": {
    ///      "description": "Whether this key is owned by a user or represents a service account",
    ///      "type": "string",
    ///      "enum": [
    ///        "user_owned",
    ///        "service_account"
    ///      ]
    ///    },
    ///    "description": {
    ///      "description": "New description",
    ///      "type": "string",
    ///      "maxLength": 500
    ///    },
    ///    "name": {
    ///      "description": "New name",
    ///      "type": "string",
    ///      "maxLength": 100,
    ///      "minLength": 1
    ///    },
    ///    "ownerUserId": {
    ///      "description": "Owner user ID when attributionClass is user_owned",
    ///      "type": "string"
    ///    },
    ///    "role": {
    ///      "$ref": "#/components/schemas/UserRole"
    ///    },
    ///    "serviceAccountName": {
    ///      "description": "Service-account display name when attributionClass is service_account",
    ///      "type": "string"
    ///    },
    ///    "workspaceAccess": {
    ///      "$ref": "#/components/schemas/WorkspaceAccess"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateApiKeyRequest {
        ///Interfaces where this API key may be accepted
        #[serde(
            rename = "allowedSurfaces",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub allowed_surfaces: ::std::option::Option<
            Vec<UpdateApiKeyRequestAllowedSurfacesItem>,
        >,
        ///Whether this key is owned by a user or represents a service account
        #[serde(
            rename = "attributionClass",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub attribution_class: ::std::option::Option<
            UpdateApiKeyRequestAttributionClass,
        >,
        ///New description
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<UpdateApiKeyRequestDescription>,
        ///New name
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub name: ::std::option::Option<UpdateApiKeyRequestName>,
        ///Owner user ID when attributionClass is user_owned
        #[serde(
            rename = "ownerUserId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub owner_user_id: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub role: ::std::option::Option<UserRole>,
        ///Service-account display name when attributionClass is service_account
        #[serde(
            rename = "serviceAccountName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub service_account_name: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "workspaceAccess",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub workspace_access: ::std::option::Option<WorkspaceAccess>,
    }
    impl ::std::default::Default for UpdateApiKeyRequest {
        fn default() -> Self {
            Self {
                allowed_surfaces: Default::default(),
                attribution_class: Default::default(),
                description: Default::default(),
                name: Default::default(),
                owner_user_id: Default::default(),
                role: Default::default(),
                service_account_name: Default::default(),
                workspace_access: Default::default(),
            }
        }
    }
    ///`UpdateApiKeyRequestAllowedSurfacesItem`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "rest_api",
    ///    "mcp"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum UpdateApiKeyRequestAllowedSurfacesItem {
        #[serde(rename = "rest_api")]
        RestApi,
        #[serde(rename = "mcp")]
        Mcp,
    }
    impl ::std::fmt::Display for UpdateApiKeyRequestAllowedSurfacesItem {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::RestApi => f.write_str("rest_api"),
                Self::Mcp => f.write_str("mcp"),
            }
        }
    }
    impl ::std::str::FromStr for UpdateApiKeyRequestAllowedSurfacesItem {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "rest_api" => Ok(Self::RestApi),
                "mcp" => Ok(Self::Mcp),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for UpdateApiKeyRequestAllowedSurfacesItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for UpdateApiKeyRequestAllowedSurfacesItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for UpdateApiKeyRequestAllowedSurfacesItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Whether this key is owned by a user or represents a service account
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Whether this key is owned by a user or represents a service account",
    ///  "type": "string",
    ///  "enum": [
    ///    "user_owned",
    ///    "service_account"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum UpdateApiKeyRequestAttributionClass {
        #[serde(rename = "user_owned")]
        UserOwned,
        #[serde(rename = "service_account")]
        ServiceAccount,
    }
    impl ::std::fmt::Display for UpdateApiKeyRequestAttributionClass {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::UserOwned => f.write_str("user_owned"),
                Self::ServiceAccount => f.write_str("service_account"),
            }
        }
    }
    impl ::std::str::FromStr for UpdateApiKeyRequestAttributionClass {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "user_owned" => Ok(Self::UserOwned),
                "service_account" => Ok(Self::ServiceAccount),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for UpdateApiKeyRequestAttributionClass {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for UpdateApiKeyRequestAttributionClass {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for UpdateApiKeyRequestAttributionClass {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///New description
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "New description",
    ///  "type": "string",
    ///  "maxLength": 500
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct UpdateApiKeyRequestDescription(::std::string::String);
    impl ::std::ops::Deref for UpdateApiKeyRequestDescription {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<UpdateApiKeyRequestDescription> for ::std::string::String {
        fn from(value: UpdateApiKeyRequestDescription) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for UpdateApiKeyRequestDescription {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 500usize {
                return Err("longer than 500 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for UpdateApiKeyRequestDescription {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for UpdateApiKeyRequestDescription {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for UpdateApiKeyRequestDescription {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for UpdateApiKeyRequestDescription {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///New name
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "New name",
    ///  "type": "string",
    ///  "maxLength": 100,
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct UpdateApiKeyRequestName(::std::string::String);
    impl ::std::ops::Deref for UpdateApiKeyRequestName {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<UpdateApiKeyRequestName> for ::std::string::String {
        fn from(value: UpdateApiKeyRequestName) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for UpdateApiKeyRequestName {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 100usize {
                return Err("longer than 100 characters".into());
            }
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for UpdateApiKeyRequestName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for UpdateApiKeyRequestName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for UpdateApiKeyRequestName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for UpdateApiKeyRequestName {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Request to replace DAG notification settings
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to replace DAG notification settings",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "events",
    ///    "targets"
    ///  ],
    ///  "properties": {
    ///    "enabled": {
    ///      "description": "Whether notification delivery is enabled for this DAG",
    ///      "type": "boolean"
    ///    },
    ///    "events": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationEventType"
    ///      }
    ///    },
    ///    "subscriptions": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationSubscriptionInput"
    ///      }
    ///    },
    ///    "targets": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/NotificationTargetInput"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateDagNotificationsRequest {
        ///Whether notification delivery is enabled for this DAG
        pub enabled: bool,
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub events: ::std::vec::Vec<NotificationEventType>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub subscriptions: ::std::vec::Vec<NotificationSubscriptionInput>,
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub targets: ::std::vec::Vec<NotificationTargetInput>,
    }
    ///`UpdateDagRunStepStatusBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "status"
    ///  ],
    ///  "properties": {
    ///    "status": {
    ///      "$ref": "#/components/schemas/NodeStatus"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateDagRunStepStatusBody {
        pub status: NodeStatus,
    }
    ///Request to replace DAG settings. Omit `profile` to clear the DAG default profile.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to replace DAG settings. Omit `profile` to clear the DAG default profile.",
    ///  "type": "object",
    ///  "properties": {
    ///    "profile": {
    ///      "$ref": "#/components/schemas/RuntimeProfileName"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateDagSettingsRequest {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub profile: ::std::option::Option<RuntimeProfileName>,
    }
    impl ::std::default::Default for UpdateDagSettingsRequest {
        fn default() -> Self {
            Self {
                profile: Default::default(),
            }
        }
    }
    ///`UpdateDagSpecBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "spec"
    ///  ],
    ///  "properties": {
    ///    "spec": {
    ///      "description": "The new DAG spec in YAML format",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateDagSpecBody {
        ///The new DAG spec in YAML format
        pub spec: ::std::string::String,
    }
    ///`UpdateDagSpecResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "errors"
    ///  ],
    ///  "properties": {
    ///    "errors": {
    ///      "description": "List of errors in the spec",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateDagSpecResponse {
        ///List of errors in the spec
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub errors: ::std::vec::Vec<::std::string::String>,
    }
    ///`UpdateInheritedRuntimeProfileRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "defaultProfile": {
    ///      "$ref": "#/components/schemas/RuntimeProfileOverride"
    ///    },
    ///    "description": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateInheritedRuntimeProfileRequest {
        #[serde(
            rename = "defaultProfile",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub default_profile: ::std::option::Option<RuntimeProfileOverride>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for UpdateInheritedRuntimeProfileRequest {
        fn default() -> Self {
            Self {
                default_profile: Default::default(),
                description: Default::default(),
            }
        }
    }
    ///`UpdateRemoteNodeRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "apiBaseUrl": {
    ///      "type": "string"
    ///    },
    ///    "authToken": {
    ///      "type": "string"
    ///    },
    ///    "authType": {
    ///      "type": "string",
    ///      "enum": [
    ///        "none",
    ///        "basic",
    ///        "token"
    ///      ]
    ///    },
    ///    "basicAuthPassword": {
    ///      "type": "string"
    ///    },
    ///    "basicAuthUsername": {
    ///      "type": "string"
    ///    },
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "skipTlsVerify": {
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateRemoteNodeRequest {
        #[serde(
            rename = "apiBaseUrl",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub api_base_url: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "authToken",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub auth_token: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "authType",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub auth_type: ::std::option::Option<UpdateRemoteNodeRequestAuthType>,
        #[serde(
            rename = "basicAuthPassword",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub basic_auth_password: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "basicAuthUsername",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub basic_auth_username: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub name: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "skipTlsVerify",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub skip_tls_verify: ::std::option::Option<bool>,
    }
    impl ::std::default::Default for UpdateRemoteNodeRequest {
        fn default() -> Self {
            Self {
                api_base_url: Default::default(),
                auth_token: Default::default(),
                auth_type: Default::default(),
                basic_auth_password: Default::default(),
                basic_auth_username: Default::default(),
                description: Default::default(),
                name: Default::default(),
                skip_tls_verify: Default::default(),
            }
        }
    }
    ///`UpdateRemoteNodeRequestAuthType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "none",
    ///    "basic",
    ///    "token"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum UpdateRemoteNodeRequestAuthType {
        #[serde(rename = "none")]
        None,
        #[serde(rename = "basic")]
        Basic,
        #[serde(rename = "token")]
        Token,
    }
    impl ::std::fmt::Display for UpdateRemoteNodeRequestAuthType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::None => f.write_str("none"),
                Self::Basic => f.write_str("basic"),
                Self::Token => f.write_str("token"),
            }
        }
    }
    impl ::std::str::FromStr for UpdateRemoteNodeRequestAuthType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "none" => Ok(Self::None),
                "basic" => Ok(Self::Basic),
                "token" => Ok(Self::Token),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for UpdateRemoteNodeRequestAuthType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for UpdateRemoteNodeRequestAuthType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for UpdateRemoteNodeRequestAuthType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`UpdateRuntimeProfileRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "protected": {
    ///      "type": "boolean"
    ///    },
    ///    "status": {
    ///      "$ref": "#/components/schemas/RuntimeProfileStatus"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateRuntimeProfileRequest {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub protected: ::std::option::Option<bool>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub status: ::std::option::Option<RuntimeProfileStatus>,
    }
    impl ::std::default::Default for UpdateRuntimeProfileRequest {
        fn default() -> Self {
            Self {
                description: Default::default(),
                protected: Default::default(),
                status: Default::default(),
            }
        }
    }
    ///`UpdateSecretRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "providerConnectionId": {
    ///      "type": "string"
    ///    },
    ///    "providerRef": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateSecretRequest {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "providerConnectionId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub provider_connection_id: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "providerRef",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub provider_ref: ::std::option::Option<::std::string::String>,
    }
    impl ::std::default::Default for UpdateSecretRequest {
        fn default() -> Self {
            Self {
                description: Default::default(),
                provider_connection_id: Default::default(),
                provider_ref: Default::default(),
            }
        }
    }
    ///Request body for updating a user
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for updating a user",
    ///  "type": "object",
    ///  "properties": {
    ///    "isDisabled": {
    ///      "description": "Whether to disable the user account",
    ///      "type": "boolean"
    ///    },
    ///    "role": {
    ///      "$ref": "#/components/schemas/UserRole"
    ///    },
    ///    "username": {
    ///      "description": "New username (must be unique)",
    ///      "type": "string",
    ///      "maxLength": 64,
    ///      "minLength": 1
    ///    },
    ///    "workspaceAccess": {
    ///      "$ref": "#/components/schemas/WorkspaceAccess"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateUserRequest {
        ///Whether to disable the user account
        #[serde(
            rename = "isDisabled",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub is_disabled: ::std::option::Option<bool>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub role: ::std::option::Option<UserRole>,
        ///New username (must be unique)
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub username: ::std::option::Option<UpdateUserRequestUsername>,
        #[serde(
            rename = "workspaceAccess",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub workspace_access: ::std::option::Option<WorkspaceAccess>,
    }
    impl ::std::default::Default for UpdateUserRequest {
        fn default() -> Self {
            Self {
                is_disabled: Default::default(),
                role: Default::default(),
                username: Default::default(),
                workspace_access: Default::default(),
            }
        }
    }
    ///New username (must be unique)
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "New username (must be unique)",
    ///  "type": "string",
    ///  "maxLength": 64,
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct UpdateUserRequestUsername(::std::string::String);
    impl ::std::ops::Deref for UpdateUserRequestUsername {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<UpdateUserRequestUsername> for ::std::string::String {
        fn from(value: UpdateUserRequestUsername) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for UpdateUserRequestUsername {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 64usize {
                return Err("longer than 64 characters".into());
            }
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for UpdateUserRequestUsername {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for UpdateUserRequestUsername {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for UpdateUserRequestUsername {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for UpdateUserRequestUsername {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Request to update Wiki page content
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to update Wiki page content",
    ///  "type": "object",
    ///  "required": [
    ///    "content"
    ///  ],
    ///  "properties": {
    ///    "content": {
    ///      "description": "Full file content including optional YAML frontmatter",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateWikiPageRequest {
        ///Full file content including optional YAML frontmatter
        pub content: ::std::string::String,
    }
    ///`UpdateWorkspaceRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "$ref": "#/components/schemas/WorkspaceName"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UpdateWorkspaceRequest {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub name: ::std::option::Option<WorkspaceName>,
    }
    impl ::std::default::Default for UpdateWorkspaceRequest {
        fn default() -> Self {
            Self {
                description: Default::default(),
                name: Default::default(),
            }
        }
    }
    ///User information
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "User information",
    ///  "type": "object",
    ///  "required": [
    ///    "createdAt",
    ///    "id",
    ///    "role",
    ///    "updatedAt",
    ///    "username",
    ///    "workspaceAccess"
    ///  ],
    ///  "properties": {
    ///    "authProvider": {
    ///      "$ref": "#/components/schemas/UserAuthProvider"
    ///    },
    ///    "createdAt": {
    ///      "description": "Account creation timestamp",
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "description": "Unique user identifier",
    ///      "type": "string"
    ///    },
    ///    "isDisabled": {
    ///      "description": "Whether the user account is disabled",
    ///      "type": "boolean"
    ///    },
    ///    "role": {
    ///      "$ref": "#/components/schemas/UserRole"
    ///    },
    ///    "updatedAt": {
    ///      "description": "Last update timestamp",
    ///      "type": "string"
    ///    },
    ///    "username": {
    ///      "description": "User's username",
    ///      "type": "string"
    ///    },
    ///    "workspaceAccess": {
    ///      "$ref": "#/components/schemas/WorkspaceAccess"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct User {
        #[serde(
            rename = "authProvider",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub auth_provider: ::std::option::Option<UserAuthProvider>,
        ///Account creation timestamp
        #[serde(rename = "createdAt")]
        pub created_at: ::std::string::String,
        ///Unique user identifier
        pub id: ::std::string::String,
        ///Whether the user account is disabled
        #[serde(
            rename = "isDisabled",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub is_disabled: ::std::option::Option<bool>,
        pub role: UserRole,
        ///Last update timestamp
        #[serde(rename = "updatedAt")]
        pub updated_at: ::std::string::String,
        ///User's username
        pub username: ::std::string::String,
        #[serde(rename = "workspaceAccess")]
        pub workspace_access: WorkspaceAccess,
    }
    ///Authentication provider for a user account
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Authentication provider for a user account",
    ///  "type": "string",
    ///  "enum": [
    ///    "builtin",
    ///    "oidc",
    ///    "proxy"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum UserAuthProvider {
        #[serde(rename = "builtin")]
        Builtin,
        #[serde(rename = "oidc")]
        Oidc,
        #[serde(rename = "proxy")]
        Proxy,
    }
    impl ::std::fmt::Display for UserAuthProvider {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Builtin => f.write_str("builtin"),
                Self::Oidc => f.write_str("oidc"),
                Self::Proxy => f.write_str("proxy"),
            }
        }
    }
    impl ::std::str::FromStr for UserAuthProvider {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "builtin" => Ok(Self::Builtin),
                "oidc" => Ok(Self::Oidc),
                "proxy" => Ok(Self::Proxy),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for UserAuthProvider {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for UserAuthProvider {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for UserAuthProvider {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Response containing user information
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response containing user information",
    ///  "type": "object",
    ///  "required": [
    ///    "user"
    ///  ],
    ///  "properties": {
    ///    "user": {
    ///      "$ref": "#/components/schemas/User"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UserResponse {
        pub user: User,
    }
    ///User role determining access permissions. admin: full access including user management, manager: DAG CRUD and execution with audit log access, developer: DAG CRUD and execution, operator: DAG execution only, viewer: read-only
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "User role determining access permissions. admin: full access including user management, manager: DAG CRUD and execution with audit log access, developer: DAG CRUD and execution, operator: DAG execution only, viewer: read-only",
    ///  "type": "string",
    ///  "enum": [
    ///    "admin",
    ///    "manager",
    ///    "developer",
    ///    "operator",
    ///    "viewer"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum UserRole {
        #[serde(rename = "admin")]
        Admin,
        #[serde(rename = "manager")]
        Manager,
        #[serde(rename = "developer")]
        Developer,
        #[serde(rename = "operator")]
        Operator,
        #[serde(rename = "viewer")]
        Viewer,
    }
    impl ::std::fmt::Display for UserRole {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Admin => f.write_str("admin"),
                Self::Manager => f.write_str("manager"),
                Self::Developer => f.write_str("developer"),
                Self::Operator => f.write_str("operator"),
                Self::Viewer => f.write_str("viewer"),
            }
        }
    }
    impl ::std::str::FromStr for UserRole {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "admin" => Ok(Self::Admin),
                "manager" => Ok(Self::Manager),
                "developer" => Ok(Self::Developer),
                "operator" => Ok(Self::Operator),
                "viewer" => Ok(Self::Viewer),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for UserRole {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for UserRole {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for UserRole {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Response containing list of users
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response containing list of users",
    ///  "type": "object",
    ///  "required": [
    ///    "managedRoleProviders",
    ///    "managedWorkspaceAccessProviders",
    ///    "users"
    ///  ],
    ///  "properties": {
    ///    "managedRoleProviders": {
    ///      "description": "Authentication providers that synchronize user roles at login on this node",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/UserAuthProvider"
    ///      },
    ///      "uniqueItems": true
    ///    },
    ///    "managedWorkspaceAccessProviders": {
    ///      "description": "Authentication providers that synchronize workspace access at login on this node",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/UserAuthProvider"
    ///      },
    ///      "uniqueItems": true
    ///    },
    ///    "oidcWorkspaceAccessSyncEnabled": {
    ///      "description": "Whether OIDC workspace access is synchronized by this node",
    ///      "type": "boolean"
    ///    },
    ///    "users": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/User"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct UsersListResponse {
        ///Authentication providers that synchronize user roles at login on this node
        #[serde(rename = "managedRoleProviders")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub managed_role_providers: Vec<UserAuthProvider>,
        ///Authentication providers that synchronize workspace access at login on this node
        #[serde(rename = "managedWorkspaceAccessProviders")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub managed_workspace_access_providers: Vec<UserAuthProvider>,
        ///Whether OIDC workspace access is synchronized by this node
        #[serde(
            rename = "oidcWorkspaceAccessSyncEnabled",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub oidc_workspace_access_sync_enabled: ::std::option::Option<bool>,
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub users: ::std::vec::Vec<User>,
    }
    ///`ValidateDagSpecBody`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "spec"
    ///  ],
    ///  "properties": {
    ///    "name": {
    ///      "description": "Optional name to use when the spec omits a name",
    ///      "type": "string"
    ///    },
    ///    "spec": {
    ///      "description": "DAG specification in YAML format",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ValidateDagSpecBody {
        ///Optional name to use when the spec omits a name
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub name: ::std::option::Option<::std::string::String>,
        ///DAG specification in YAML format
        pub spec: ::std::string::String,
    }
    ///`ValidateDagSpecResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "errors",
    ///    "valid"
    ///  ],
    ///  "properties": {
    ///    "dag": {
    ///      "$ref": "#/components/schemas/DAGDetails"
    ///    },
    ///    "errors": {
    ///      "description": "List of validation errors",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "valid": {
    ///      "description": "True if the spec is valid (no errors)",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ValidateDagSpecResponse {
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub dag: ::std::option::Option<DagDetails>,
        ///List of validation errors
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub errors: ::std::vec::Vec<::std::string::String>,
        ///True if the spec is valid (no errors)
        pub valid: bool,
    }
    ///A passive notice for a supported value reference left unresolved while loading a spec.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A passive notice for a supported value reference left unresolved while loading a spec.",
    ///  "type": "object",
    ///  "required": [
    ///    "message"
    ///  ],
    ///  "properties": {
    ///    "class": {
    ///      "description": "Whether the reference is a defect in the spec or its availability depends on runtime values or lifecycle scope.",
    ///      "type": "string",
    ///      "enum": [
    ///        "defect",
    ///        "runtime_only"
    ///      ]
    ///    },
    ///    "fieldPath": {
    ///      "description": "DAG field path associated with the unresolved reference.",
    ///      "type": "string"
    ///    },
    ///    "message": {
    ///      "description": "Human-readable explanation of the unresolved reference.",
    ///      "type": "string"
    ///    },
    ///    "reason": {
    ///      "description": "Machine-readable reason why the reference was preserved.",
    ///      "type": "string",
    ///      "enum": [
    ///        "unknown_step_id",
    ///        "unknown_output_name",
    ///        "missing_dependency",
    ///        "self_reference",
    ///        "namespace_unavailable",
    ///        "unknown_context_field",
    ///        "unknown_env_binding",
    ///        "unknown_const_name"
    ///      ]
    ///    },
    ///    "token": {
    ///      "description": "Original value-reference token that was preserved.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ValueReferenceNotice {
        ///Whether the reference is a defect in the spec or its availability depends on runtime values or lifecycle scope.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub class: ::std::option::Option<ValueReferenceNoticeClass>,
        ///DAG field path associated with the unresolved reference.
        #[serde(
            rename = "fieldPath",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub field_path: ::std::option::Option<::std::string::String>,
        ///Human-readable explanation of the unresolved reference.
        pub message: ::std::string::String,
        ///Machine-readable reason why the reference was preserved.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub reason: ::std::option::Option<ValueReferenceNoticeReason>,
        ///Original value-reference token that was preserved.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub token: ::std::option::Option<::std::string::String>,
    }
    ///Whether the reference is a defect in the spec or its availability depends on runtime values or lifecycle scope.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Whether the reference is a defect in the spec or its availability depends on runtime values or lifecycle scope.",
    ///  "type": "string",
    ///  "enum": [
    ///    "defect",
    ///    "runtime_only"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ValueReferenceNoticeClass {
        #[serde(rename = "defect")]
        Defect,
        #[serde(rename = "runtime_only")]
        RuntimeOnly,
    }
    impl ::std::fmt::Display for ValueReferenceNoticeClass {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Defect => f.write_str("defect"),
                Self::RuntimeOnly => f.write_str("runtime_only"),
            }
        }
    }
    impl ::std::str::FromStr for ValueReferenceNoticeClass {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "defect" => Ok(Self::Defect),
                "runtime_only" => Ok(Self::RuntimeOnly),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ValueReferenceNoticeClass {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ValueReferenceNoticeClass {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ValueReferenceNoticeClass {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Machine-readable reason why the reference was preserved.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Machine-readable reason why the reference was preserved.",
    ///  "type": "string",
    ///  "enum": [
    ///    "unknown_step_id",
    ///    "unknown_output_name",
    ///    "missing_dependency",
    ///    "self_reference",
    ///    "namespace_unavailable",
    ///    "unknown_context_field",
    ///    "unknown_env_binding",
    ///    "unknown_const_name"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ValueReferenceNoticeReason {
        #[serde(rename = "unknown_step_id")]
        UnknownStepId,
        #[serde(rename = "unknown_output_name")]
        UnknownOutputName,
        #[serde(rename = "missing_dependency")]
        MissingDependency,
        #[serde(rename = "self_reference")]
        SelfReference,
        #[serde(rename = "namespace_unavailable")]
        NamespaceUnavailable,
        #[serde(rename = "unknown_context_field")]
        UnknownContextField,
        #[serde(rename = "unknown_env_binding")]
        UnknownEnvBinding,
        #[serde(rename = "unknown_const_name")]
        UnknownConstName,
    }
    impl ::std::fmt::Display for ValueReferenceNoticeReason {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::UnknownStepId => f.write_str("unknown_step_id"),
                Self::UnknownOutputName => f.write_str("unknown_output_name"),
                Self::MissingDependency => f.write_str("missing_dependency"),
                Self::SelfReference => f.write_str("self_reference"),
                Self::NamespaceUnavailable => f.write_str("namespace_unavailable"),
                Self::UnknownContextField => f.write_str("unknown_context_field"),
                Self::UnknownEnvBinding => f.write_str("unknown_env_binding"),
                Self::UnknownConstName => f.write_str("unknown_const_name"),
            }
        }
    }
    impl ::std::str::FromStr for ValueReferenceNoticeReason {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "unknown_step_id" => Ok(Self::UnknownStepId),
                "unknown_output_name" => Ok(Self::UnknownOutputName),
                "missing_dependency" => Ok(Self::MissingDependency),
                "self_reference" => Ok(Self::SelfReference),
                "namespace_unavailable" => Ok(Self::NamespaceUnavailable),
                "unknown_context_field" => Ok(Self::UnknownContextField),
                "unknown_env_binding" => Ok(Self::UnknownEnvBinding),
                "unknown_const_name" => Ok(Self::UnknownConstName),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ValueReferenceNoticeReason {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ValueReferenceNoticeReason {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ValueReferenceNoticeReason {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`View`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "createdAt",
    ///    "id",
    ///    "intervalDays",
    ///    "name",
    ///    "type",
    ///    "updatedAt"
    ///  ],
    ///  "properties": {
    ///    "activeOnly": {
    ///      "description": "Whether the view only includes scheduled, unsuspended workflows.",
    ///      "type": "boolean"
    ///    },
    ///    "columns": {
    ///      "description": "Visible status columns in left-to-right display order.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ViewColumn"
    ///      }
    ///    },
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "createdBy": {
    ///      "description": "Username of the creator, for display only.",
    ///      "type": "string"
    ///    },
    ///    "dagName": {
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "intervalDays": {
    ///      "type": "integer"
    ///    },
    ///    "isDefault": {
    ///      "type": "boolean"
    ///    },
    ///    "labels": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "pinned": {
    ///      "type": "boolean"
    ///    },
    ///    "sortField": {
    ///      "$ref": "#/components/schemas/ViewSortField"
    ///    },
    ///    "sortOrder": {
    ///      "$ref": "#/components/schemas/ViewSortOrder"
    ///    },
    ///    "type": {
    ///      "type": "string"
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "type": "string"
    ///    },
    ///    "workspaceScope": {
    ///      "$ref": "#/components/schemas/ViewWorkspaceScope"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct View {
        ///Whether the view only includes scheduled, unsuspended workflows.
        #[serde(
            rename = "activeOnly",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub active_only: ::std::option::Option<bool>,
        ///Visible status columns in left-to-right display order.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub columns: ::std::vec::Vec<ViewColumn>,
        #[serde(rename = "createdAt")]
        pub created_at: ::std::string::String,
        ///Username of the creator, for display only.
        #[serde(
            rename = "createdBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub created_by: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "dagName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_name: ::std::option::Option<::std::string::String>,
        pub id: ::std::string::String,
        #[serde(rename = "intervalDays")]
        pub interval_days: i64,
        #[serde(
            rename = "isDefault",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub is_default: ::std::option::Option<bool>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub labels: ::std::vec::Vec<::std::string::String>,
        pub name: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub pinned: ::std::option::Option<bool>,
        #[serde(
            rename = "sortField",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub sort_field: ::std::option::Option<ViewSortField>,
        #[serde(
            rename = "sortOrder",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub sort_order: ::std::option::Option<ViewSortOrder>,
        #[serde(rename = "type")]
        pub type_: ::std::string::String,
        #[serde(rename = "updatedAt")]
        pub updated_at: ::std::string::String,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
        #[serde(
            rename = "workspaceScope",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub workspace_scope: ::std::option::Option<ViewWorkspaceScope>,
    }
    ///A status column available in a Kanban view.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A status column available in a Kanban view.",
    ///  "type": "string",
    ///  "enum": [
    ///    "queued",
    ///    "running",
    ///    "review",
    ///    "done",
    ///    "failed"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ViewColumn {
        #[serde(rename = "queued")]
        Queued,
        #[serde(rename = "running")]
        Running,
        #[serde(rename = "review")]
        Review,
        #[serde(rename = "done")]
        Done,
        #[serde(rename = "failed")]
        Failed,
    }
    impl ::std::fmt::Display for ViewColumn {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Queued => f.write_str("queued"),
                Self::Running => f.write_str("running"),
                Self::Review => f.write_str("review"),
                Self::Done => f.write_str("done"),
                Self::Failed => f.write_str("failed"),
            }
        }
    }
    impl ::std::str::FromStr for ViewColumn {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "queued" => Ok(Self::Queued),
                "running" => Ok(Self::Running),
                "review" => Ok(Self::Review),
                "done" => Ok(Self::Done),
                "failed" => Ok(Self::Failed),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ViewColumn {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ViewColumn {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ViewColumn {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`ViewListResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "views"
    ///  ],
    ///  "properties": {
    ///    "views": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/View"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ViewListResponse {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub views: ::std::vec::Vec<View>,
    }
    ///Field used to sort workflows.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Field used to sort workflows.",
    ///  "type": "string",
    ///  "enum": [
    ///    "name",
    ///    "nextRun"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ViewSortField {
        #[serde(rename = "name")]
        Name,
        #[serde(rename = "nextRun")]
        NextRun,
    }
    impl ::std::fmt::Display for ViewSortField {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Name => f.write_str("name"),
                Self::NextRun => f.write_str("nextRun"),
            }
        }
    }
    impl ::std::str::FromStr for ViewSortField {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "name" => Ok(Self::Name),
                "nextRun" => Ok(Self::NextRun),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ViewSortField {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ViewSortField {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ViewSortField {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Direction used to sort workflows.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Direction used to sort workflows.",
    ///  "type": "string",
    ///  "enum": [
    ///    "asc",
    ///    "desc"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ViewSortOrder {
        #[serde(rename = "asc")]
        Asc,
        #[serde(rename = "desc")]
        Desc,
    }
    impl ::std::fmt::Display for ViewSortOrder {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Asc => f.write_str("asc"),
                Self::Desc => f.write_str("desc"),
            }
        }
    }
    impl ::std::str::FromStr for ViewSortOrder {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "asc" => Ok(Self::Asc),
                "desc" => Ok(Self::Desc),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ViewSortOrder {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ViewSortOrder {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ViewSortOrder {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///`ViewSpec`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "intervalDays",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "activeOnly": {
    ///      "description": "Whether the view only includes scheduled, unsuspended workflows.",
    ///      "type": "boolean"
    ///    },
    ///    "columns": {
    ///      "description": "Status columns to display, in left-to-right order. Omitted values use the default order with all columns visible.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/ViewColumn"
    ///      },
    ///      "minItems": 1,
    ///      "uniqueItems": true
    ///    },
    ///    "dagName": {
    ///      "description": "DAG name substring filter. Empty matches any.",
    ///      "type": "string",
    ///      "maxLength": 255
    ///    },
    ///    "intervalDays": {
    ///      "description": "Required number of days each row (bucket) groups. Rows scroll back in time by this unit.",
    ///      "type": "integer",
    ///      "maximum": 30.0,
    ///      "minimum": 1.0
    ///    },
    ///    "isDefault": {
    ///      "description": "Whether this is the default workflow view for its workspace scope.",
    ///      "type": "boolean"
    ///    },
    ///    "labels": {
    ///      "description": "Label filter (AND logic), each item key or key=value.",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string",
    ///        "maxLength": 128,
    ///        "minLength": 1
    ///      },
    ///      "maxItems": 50
    ///    },
    ///    "name": {
    ///      "description": "Display name for the view.",
    ///      "type": "string",
    ///      "maxLength": 100,
    ///      "minLength": 1
    ///    },
    ///    "pinned": {
    ///      "description": "Whether the view is pinned to the left sidebar.",
    ///      "default": false,
    ///      "type": "boolean"
    ///    },
    ///    "sortField": {
    ///      "$ref": "#/components/schemas/ViewSortField"
    ///    },
    ///    "sortOrder": {
    ///      "$ref": "#/components/schemas/ViewSortOrder"
    ///    },
    ///    "type": {
    ///      "description": "Saved view type.",
    ///      "default": "kanban",
    ///      "type": "string",
    ///      "enum": [
    ///        "kanban",
    ///        "workflow"
    ///      ]
    ///    },
    ///    "workspace": {
    ///      "description": "Workspace filter. Empty string means all workspaces; otherwise use a workspace name.",
    ///      "type": "string",
    ///      "maxLength": 64
    ///    },
    ///    "workspaceScope": {
    ///      "$ref": "#/components/schemas/ViewWorkspaceScope"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct ViewSpec {
        ///Whether the view only includes scheduled, unsuspended workflows.
        #[serde(
            rename = "activeOnly",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub active_only: ::std::option::Option<bool>,
        ///Status columns to display, in left-to-right order. Omitted values use the default order with all columns visible.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub columns: ::std::option::Option<Vec<ViewColumn>>,
        ///DAG name substring filter. Empty matches any.
        #[serde(
            rename = "dagName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_name: ::std::option::Option<ViewSpecDagName>,
        ///Required number of days each row (bucket) groups. Rows scroll back in time by this unit.
        #[serde(rename = "intervalDays")]
        pub interval_days: ::std::num::NonZeroU64,
        ///Whether this is the default workflow view for its workspace scope.
        #[serde(
            rename = "isDefault",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub is_default: ::std::option::Option<bool>,
        ///Label filter (AND logic), each item key or key=value.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub labels: ::std::vec::Vec<ViewSpecLabelsItem>,
        ///Display name for the view.
        pub name: ViewSpecName,
        ///Whether the view is pinned to the left sidebar.
        #[serde(default)]
        pub pinned: bool,
        #[serde(
            rename = "sortField",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub sort_field: ::std::option::Option<ViewSortField>,
        #[serde(
            rename = "sortOrder",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub sort_order: ::std::option::Option<ViewSortOrder>,
        ///Saved view type.
        #[serde(rename = "type", default = "defaults::view_spec_type")]
        pub type_: ViewSpecType,
        ///Workspace filter. Empty string means all workspaces; otherwise use a workspace name.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<ViewSpecWorkspace>,
        #[serde(
            rename = "workspaceScope",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub workspace_scope: ::std::option::Option<ViewWorkspaceScope>,
    }
    ///DAG name substring filter. Empty matches any.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "DAG name substring filter. Empty matches any.",
    ///  "type": "string",
    ///  "maxLength": 255
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct ViewSpecDagName(::std::string::String);
    impl ::std::ops::Deref for ViewSpecDagName {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<ViewSpecDagName> for ::std::string::String {
        fn from(value: ViewSpecDagName) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for ViewSpecDagName {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 255usize {
                return Err("longer than 255 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for ViewSpecDagName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ViewSpecDagName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ViewSpecDagName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for ViewSpecDagName {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///`ViewSpecLabelsItem`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "maxLength": 128,
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct ViewSpecLabelsItem(::std::string::String);
    impl ::std::ops::Deref for ViewSpecLabelsItem {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<ViewSpecLabelsItem> for ::std::string::String {
        fn from(value: ViewSpecLabelsItem) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for ViewSpecLabelsItem {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 128usize {
                return Err("longer than 128 characters".into());
            }
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for ViewSpecLabelsItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ViewSpecLabelsItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ViewSpecLabelsItem {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for ViewSpecLabelsItem {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Display name for the view.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Display name for the view.",
    ///  "type": "string",
    ///  "maxLength": 100,
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct ViewSpecName(::std::string::String);
    impl ::std::ops::Deref for ViewSpecName {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<ViewSpecName> for ::std::string::String {
        fn from(value: ViewSpecName) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for ViewSpecName {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 100usize {
                return Err("longer than 100 characters".into());
            }
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for ViewSpecName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ViewSpecName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ViewSpecName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for ViewSpecName {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Saved view type.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Saved view type.",
    ///  "default": "kanban",
    ///  "type": "string",
    ///  "enum": [
    ///    "kanban",
    ///    "workflow"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ViewSpecType {
        #[serde(rename = "kanban")]
        Kanban,
        #[serde(rename = "workflow")]
        Workflow,
    }
    impl ::std::fmt::Display for ViewSpecType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Kanban => f.write_str("kanban"),
                Self::Workflow => f.write_str("workflow"),
            }
        }
    }
    impl ::std::str::FromStr for ViewSpecType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "kanban" => Ok(Self::Kanban),
                "workflow" => Ok(Self::Workflow),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ViewSpecType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ViewSpecType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ViewSpecType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::default::Default for ViewSpecType {
        fn default() -> Self {
            ViewSpecType::Kanban
        }
    }
    ///Workspace filter. Empty string means all workspaces; otherwise use a workspace name.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Workspace filter. Empty string means all workspaces; otherwise use a workspace name.",
    ///  "type": "string",
    ///  "maxLength": 64
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct ViewSpecWorkspace(::std::string::String);
    impl ::std::ops::Deref for ViewSpecWorkspace {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<ViewSpecWorkspace> for ::std::string::String {
        fn from(value: ViewSpecWorkspace) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for ViewSpecWorkspace {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 64usize {
                return Err("longer than 64 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for ViewSpecWorkspace {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ViewSpecWorkspace {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ViewSpecWorkspace {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for ViewSpecWorkspace {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Workflows page workspace selection represented by the view.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Workflows page workspace selection represented by the view.",
    ///  "type": "string",
    ///  "enum": [
    ///    "all",
    ///    "default",
    ///    "workspace"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum ViewWorkspaceScope {
        #[serde(rename = "all")]
        All,
        #[serde(rename = "default")]
        Default,
        #[serde(rename = "workspace")]
        Workspace,
    }
    impl ::std::fmt::Display for ViewWorkspaceScope {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::All => f.write_str("all"),
                Self::Default => f.write_str("default"),
                Self::Workspace => f.write_str("workspace"),
            }
        }
    }
    impl ::std::str::FromStr for ViewWorkspaceScope {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "all" => Ok(Self::All),
                "default" => Ok(Self::Default),
                "workspace" => Ok(Self::Workspace),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for ViewWorkspaceScope {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for ViewWorkspaceScope {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for ViewWorkspaceScope {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Authentication mode for a webhook trigger endpoint
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Authentication mode for a webhook trigger endpoint",
    ///  "type": "string",
    ///  "enum": [
    ///    "token_only",
    ///    "token_and_hmac",
    ///    "hmac_only"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum WebhookAuthMode {
        #[serde(rename = "token_only")]
        TokenOnly,
        #[serde(rename = "token_and_hmac")]
        TokenAndHmac,
        #[serde(rename = "hmac_only")]
        HmacOnly,
    }
    impl ::std::fmt::Display for WebhookAuthMode {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::TokenOnly => f.write_str("token_only"),
                Self::TokenAndHmac => f.write_str("token_and_hmac"),
                Self::HmacOnly => f.write_str("hmac_only"),
            }
        }
    }
    impl ::std::str::FromStr for WebhookAuthMode {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "token_only" => Ok(Self::TokenOnly),
                "token_and_hmac" => Ok(Self::TokenAndHmac),
                "hmac_only" => Ok(Self::HmacOnly),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for WebhookAuthMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for WebhookAuthMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for WebhookAuthMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Response when creating or regenerating a webhook (includes full token)
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response when creating or regenerating a webhook (includes full token)",
    ///  "type": "object",
    ///  "required": [
    ///    "token",
    ///    "webhook"
    ///  ],
    ///  "properties": {
    ///    "token": {
    ///      "description": "Full webhook token (only shown once, store securely!)",
    ///      "type": "string"
    ///    },
    ///    "webhook": {
    ///      "$ref": "#/components/schemas/WebhookDetails"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WebhookCreateResponse {
        ///Full webhook token (only shown once, store securely!)
        pub token: ::std::string::String,
        pub webhook: WebhookDetails,
    }
    ///Webhook configuration details (token not included)
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Webhook configuration details (token not included)",
    ///  "type": "object",
    ///  "required": [
    ///    "authMode",
    ///    "createdAt",
    ///    "dagName",
    ///    "enabled",
    ///    "hmac",
    ///    "id",
    ///    "profileSelection",
    ///    "tokenPrefix",
    ///    "updatedAt"
    ///  ],
    ///  "properties": {
    ///    "authMode": {
    ///      "$ref": "#/components/schemas/WebhookAuthMode"
    ///    },
    ///    "createdAt": {
    ///      "description": "When the webhook was created",
    ///      "type": "string"
    ///    },
    ///    "createdBy": {
    ///      "description": "User ID who created the webhook",
    ///      "type": "string"
    ///    },
    ///    "dagName": {
    ///      "description": "Name of the DAG this webhook triggers",
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether the webhook is active",
    ///      "type": "boolean"
    ///    },
    ///    "hmac": {
    ///      "$ref": "#/components/schemas/WebhookHMACDetails"
    ///    },
    ///    "id": {
    ///      "description": "Unique identifier for the webhook",
    ///      "type": "string"
    ///    },
    ///    "lastUsedAt": {
    ///      "description": "When the webhook was last triggered",
    ///      "type": "string"
    ///    },
    ///    "profileSelection": {
    ///      "$ref": "#/components/schemas/WebhookProfileSelectionDetails"
    ///    },
    ///    "tokenPrefix": {
    ///      "description": "First 8 characters of the token for identification",
    ///      "type": "string"
    ///    },
    ///    "updatedAt": {
    ///      "description": "When the webhook was last modified",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WebhookDetails {
        #[serde(rename = "authMode")]
        pub auth_mode: WebhookAuthMode,
        ///When the webhook was created
        #[serde(rename = "createdAt")]
        pub created_at: ::std::string::String,
        ///User ID who created the webhook
        #[serde(
            rename = "createdBy",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub created_by: ::std::option::Option<::std::string::String>,
        ///Name of the DAG this webhook triggers
        #[serde(rename = "dagName")]
        pub dag_name: ::std::string::String,
        ///Whether the webhook is active
        pub enabled: bool,
        pub hmac: WebhookHmacDetails,
        ///Unique identifier for the webhook
        pub id: ::std::string::String,
        ///When the webhook was last triggered
        #[serde(
            rename = "lastUsedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub last_used_at: ::std::option::Option<::std::string::String>,
        #[serde(rename = "profileSelection")]
        pub profile_selection: WebhookProfileSelectionDetails,
        ///First 8 characters of the token for identification
        #[serde(rename = "tokenPrefix")]
        pub token_prefix: ::std::string::String,
        ///When the webhook was last modified
        #[serde(rename = "updatedAt")]
        pub updated_at: ::std::string::String,
    }
    /**Request to configure webhook HMAC auth mode and enforcement.
If enforcementMode is omitted when enabling HMAC, it defaults to strict.
If omitted when configuring an existing webhook, the current enforcement
mode is preserved for token_and_hmac, while hmac_only always uses strict.
Clients should omit enforcementMode when authMode is hmac_only; the
server enforces strict mode and rejects observe.
*/
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to configure webhook HMAC auth mode and enforcement.\nIf enforcementMode is omitted when enabling HMAC, it defaults to strict.\nIf omitted when configuring an existing webhook, the current enforcement\nmode is preserved for token_and_hmac, while hmac_only always uses strict.\nClients should omit enforcementMode when authMode is hmac_only; the\nserver enforces strict mode and rejects observe.\n",
    ///  "type": "object",
    ///  "required": [
    ///    "authMode"
    ///  ],
    ///  "properties": {
    ///    "authMode": {
    ///      "type": "string",
    ///      "enum": [
    ///        "token_and_hmac",
    ///        "hmac_only"
    ///      ]
    ///    },
    ///    "enforcementMode": {
    ///      "$ref": "#/components/schemas/WebhookHMACEnforcementMode"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WebhookHmacConfigureRequest {
        #[serde(rename = "authMode")]
        pub auth_mode: WebhookHmacConfigureRequestAuthMode,
        #[serde(
            rename = "enforcementMode",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub enforcement_mode: ::std::option::Option<WebhookHmacEnforcementMode>,
    }
    ///`WebhookHmacConfigureRequestAuthMode`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "token_and_hmac",
    ///    "hmac_only"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum WebhookHmacConfigureRequestAuthMode {
        #[serde(rename = "token_and_hmac")]
        TokenAndHmac,
        #[serde(rename = "hmac_only")]
        HmacOnly,
    }
    impl ::std::fmt::Display for WebhookHmacConfigureRequestAuthMode {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::TokenAndHmac => f.write_str("token_and_hmac"),
                Self::HmacOnly => f.write_str("hmac_only"),
            }
        }
    }
    impl ::std::str::FromStr for WebhookHmacConfigureRequestAuthMode {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "token_and_hmac" => Ok(Self::TokenAndHmac),
                "hmac_only" => Ok(Self::HmacOnly),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for WebhookHmacConfigureRequestAuthMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for WebhookHmacConfigureRequestAuthMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for WebhookHmacConfigureRequestAuthMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Public webhook HMAC configuration details
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Public webhook HMAC configuration details",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled",
    ///    "secretConfigured"
    ///  ],
    ///  "properties": {
    ///    "algorithm": {
    ///      "description": "Fixed HMAC algorithm for v1",
    ///      "type": "string"
    ///    },
    ///    "enabled": {
    ///      "description": "Whether HMAC authentication is currently enabled",
    ///      "type": "boolean"
    ///    },
    ///    "enforcementMode": {
    ///      "$ref": "#/components/schemas/WebhookHMACEnforcementMode"
    ///    },
    ///    "format": {
    ///      "description": "Expected signature header value format",
    ///      "type": "string"
    ///    },
    ///    "headerName": {
    ///      "description": "Header containing the HMAC signature",
    ///      "type": "string"
    ///    },
    ///    "secretConfigured": {
    ///      "description": "Whether an HMAC secret is configured for the webhook",
    ///      "type": "boolean"
    ///    },
    ///    "updatedAt": {
    ///      "description": "When the HMAC secret was last generated",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WebhookHmacDetails {
        ///Fixed HMAC algorithm for v1
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub algorithm: ::std::option::Option<::std::string::String>,
        ///Whether HMAC authentication is currently enabled
        pub enabled: bool,
        #[serde(
            rename = "enforcementMode",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub enforcement_mode: ::std::option::Option<WebhookHmacEnforcementMode>,
        ///Expected signature header value format
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub format: ::std::option::Option<::std::string::String>,
        ///Header containing the HMAC signature
        #[serde(
            rename = "headerName",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub header_name: ::std::option::Option<::std::string::String>,
        ///Whether an HMAC secret is configured for the webhook
        #[serde(rename = "secretConfigured")]
        pub secret_configured: bool,
        ///When the HMAC secret was last generated
        #[serde(
            rename = "updatedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_at: ::std::option::Option<::std::string::String>,
    }
    ///How HMAC validation is enforced when HMAC is enabled
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "How HMAC validation is enforced when HMAC is enabled",
    ///  "type": "string",
    ///  "enum": [
    ///    "strict",
    ///    "observe"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum WebhookHmacEnforcementMode {
        #[serde(rename = "strict")]
        Strict,
        #[serde(rename = "observe")]
        Observe,
    }
    impl ::std::fmt::Display for WebhookHmacEnforcementMode {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Strict => f.write_str("strict"),
                Self::Observe => f.write_str("observe"),
            }
        }
    }
    impl ::std::str::FromStr for WebhookHmacEnforcementMode {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "strict" => Ok(Self::Strict),
                "observe" => Ok(Self::Observe),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for WebhookHmacEnforcementMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for WebhookHmacEnforcementMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for WebhookHmacEnforcementMode {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Response when enabling or regenerating webhook HMAC (includes full secret)
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response when enabling or regenerating webhook HMAC (includes full secret)",
    ///  "type": "object",
    ///  "required": [
    ///    "hmacSecret",
    ///    "webhook"
    ///  ],
    ///  "properties": {
    ///    "hmacSecret": {
    ///      "description": "Full HMAC secret (only shown once, store securely!)",
    ///      "type": "string"
    ///    },
    ///    "webhook": {
    ///      "$ref": "#/components/schemas/WebhookDetails"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WebhookHmacSecretResponse {
        ///Full HMAC secret (only shown once, store securely!)
        #[serde(rename = "hmacSecret")]
        pub hmac_secret: ::std::string::String,
        pub webhook: WebhookDetails,
    }
    ///List of all webhooks
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "List of all webhooks",
    ///  "type": "object",
    ///  "required": [
    ///    "webhooks"
    ///  ],
    ///  "properties": {
    ///    "webhooks": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WebhookDetails"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WebhookListResponse {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub webhooks: ::std::vec::Vec<WebhookDetails>,
    }
    ///Runtime profiles that webhook callers may select
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Runtime profiles that webhook callers may select",
    ///  "type": "object",
    ///  "required": [
    ///    "allowedProfiles"
    ///  ],
    ///  "properties": {
    ///    "allowedProfiles": {
    ///      "description": "Runtime profile names accepted through X-Dagu-Profile. An empty list disables caller selection.",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/RuntimeProfileName"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WebhookProfileSelectionDetails {
        ///Runtime profile names accepted through X-Dagu-Profile. An empty list disables caller selection.
        #[serde(rename = "allowedProfiles")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub allowed_profiles: ::std::vec::Vec<RuntimeProfileName>,
    }
    ///Replacement runtime-profile allowlist for a webhook
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Replacement runtime-profile allowlist for a webhook",
    ///  "type": "object",
    ///  "required": [
    ///    "allowedProfiles"
    ///  ],
    ///  "properties": {
    ///    "allowedProfiles": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/RuntimeProfileName"
    ///      },
    ///      "maxItems": 100,
    ///      "uniqueItems": true
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WebhookProfileSelectionRequest {
        #[serde(rename = "allowedProfiles")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub allowed_profiles: Vec<RuntimeProfileName>,
    }
    ///Request body for webhook trigger endpoint
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request body for webhook trigger endpoint",
    ///  "type": "object",
    ///  "properties": {
    ///    "dagRunId": {
    ///      "allOf": [
    ///        {
    ///          "$ref": "#/components/schemas/DAGRunId"
    ///        },
    ///        {
    ///          "description": "Optional idempotency key. If provided and a DAG run with this ID already exists,\nthe endpoint returns 409 Conflict. If not provided, a new UUID is generated.\n"
    ///        }
    ///      ]
    ///    },
    ///    "payload": {
    ///      "description": "Arbitrary JSON payload to pass to the DAG as WEBHOOK_PAYLOAD",
    ///      "type": "object",
    ///      "additionalProperties": true
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WebhookRequest {
        #[serde(
            rename = "dagRunId",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub dag_run_id: ::std::option::Option<DagRunId>,
        ///Arbitrary JSON payload to pass to the DAG as WEBHOOK_PAYLOAD
        #[serde(default, skip_serializing_if = "::serde_json::Map::is_empty")]
        pub payload: ::serde_json::Map<::std::string::String, ::serde_json::Value>,
    }
    impl ::std::default::Default for WebhookRequest {
        fn default() -> Self {
            Self {
                dag_run_id: Default::default(),
                payload: Default::default(),
            }
        }
    }
    ///Response from webhook trigger endpoint
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response from webhook trigger endpoint",
    ///  "type": "object",
    ///  "required": [
    ///    "dagName",
    ///    "dagRunId"
    ///  ],
    ///  "properties": {
    ///    "dagName": {
    ///      "description": "Name of the triggered DAG",
    ///      "type": "string"
    ///    },
    ///    "dagRunId": {
    ///      "$ref": "#/components/schemas/DAGRunId"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WebhookResponse {
        ///Name of the triggered DAG
        #[serde(rename = "dagName")]
        pub dag_name: ::std::string::String,
        #[serde(rename = "dagRunId")]
        pub dag_run_id: DagRunId,
    }
    ///Request to toggle webhook enabled state
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to toggle webhook enabled state",
    ///  "type": "object",
    ///  "required": [
    ///    "enabled"
    ///  ],
    ///  "properties": {
    ///    "enabled": {
    ///      "description": "Whether to enable or disable the webhook",
    ///      "type": "boolean"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WebhookToggleRequest {
        ///Whether to enable or disable the webhook
        pub enabled: bool,
    }
    ///A stored Wiki page attachment
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A stored Wiki page attachment",
    ///  "type": "object",
    ///  "required": [
    ///    "name",
    ///    "size"
    ///  ],
    ///  "properties": {
    ///    "name": {
    ///      "description": "Attachment file name",
    ///      "type": "string"
    ///    },
    ///    "size": {
    ///      "description": "Attachment size in bytes",
    ///      "type": "integer",
    ///      "format": "int64"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageAttachmentResponse {
        ///Attachment file name
        pub name: ::std::string::String,
        ///Attachment size in bytes
        pub size: i64,
    }
    ///Wiki pages linking to a wiki-link target
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Wiki pages linking to a wiki-link target",
    ///  "type": "object",
    ///  "required": [
    ///    "items"
    ///  ],
    ///  "properties": {
    ///    "items": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WikiPageMetadataResponse"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageBacklinksResponse {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub items: ::std::vec::Vec<WikiPageMetadataResponse>,
    }
    ///`WikiPageDeleteBatchFailedItem`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "error",
    ///    "path"
    ///  ],
    ///  "properties": {
    ///    "error": {
    ///      "type": "string"
    ///    },
    ///    "path": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageDeleteBatchFailedItem {
        pub error: ::std::string::String,
        pub path: ::std::string::String,
    }
    ///Request to delete multiple Wiki pages or directories
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Request to delete multiple Wiki pages or directories",
    ///  "type": "object",
    ///  "required": [
    ///    "paths"
    ///  ],
    ///  "properties": {
    ///    "paths": {
    ///      "description": "Wiki page or directory paths to delete (max 100)",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WikiPagePath"
    ///      },
    ///      "maxItems": 100,
    ///      "minItems": 1,
    ///      "uniqueItems": true
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageDeleteBatchRequest {
        ///Wiki page or directory paths to delete (max 100)
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub paths: Vec<WikiPagePath>,
    }
    ///`WikiPageDeleteBatchResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "deleted",
    ///    "failed",
    ///    "message"
    ///  ],
    ///  "properties": {
    ///    "deleted": {
    ///      "description": "Successfully deleted paths",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "failed": {
    ///      "description": "Paths that failed to delete with error details",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WikiPageDeleteBatchFailedItem"
    ///      }
    ///    },
    ///    "message": {
    ///      "description": "Human-readable summary",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageDeleteBatchResponse {
        ///Successfully deleted paths
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub deleted: ::std::vec::Vec<::std::string::String>,
        ///Paths that failed to delete with error details
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub failed: ::std::vec::Vec<WikiPageDeleteBatchFailedItem>,
        ///Human-readable summary
        pub message: ::std::string::String,
    }
    ///Paginated Wiki page list (tree or flat)
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Paginated Wiki page list (tree or flat)",
    ///  "type": "object",
    ///  "required": [
    ///    "pagination"
    ///  ],
    ///  "properties": {
    ///    "items": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WikiPageMetadataResponse"
    ///      }
    ///    },
    ///    "pagination": {
    ///      "$ref": "#/components/schemas/Pagination"
    ///    },
    ///    "tree": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WikiPageTreeNodeResponse"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageListResponse {
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub items: ::std::vec::Vec<WikiPageMetadataResponse>,
        pub pagination: Pagination,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub tree: ::std::vec::Vec<WikiPageTreeNodeResponse>,
    }
    ///Lightweight Wiki page metadata
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Lightweight Wiki page metadata",
    ///  "type": "object",
    ///  "required": [
    ///    "description",
    ///    "id",
    ///    "title"
    ///  ],
    ///  "properties": {
    ///    "description": {
    ///      "description": "Short Wiki page description from YAML frontmatter",
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "modifiedAt": {
    ///      "description": "Last modification time of the Wiki page file",
    ///      "type": "string"
    ///    },
    ///    "tags": {
    ///      "description": "Wiki page tags from YAML frontmatter",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "title": {
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "description": "Workspace that owns this Wiki page. Omitted for default Wiki pages.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageMetadataResponse {
        ///Short Wiki page description from YAML frontmatter
        pub description: ::std::string::String,
        pub id: ::std::string::String,
        ///Last modification time of the Wiki page file
        #[serde(
            rename = "modifiedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub modified_at: ::std::option::Option<::std::string::String>,
        ///Wiki page tags from YAML frontmatter
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub tags: ::std::vec::Vec<::std::string::String>,
        pub title: ::std::string::String,
        ///Workspace that owns this Wiki page. Omitted for default Wiki pages.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///Relative Wiki page path without extension, for example runbooks/deploy-guide. Must not start with / or contain ..
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Relative Wiki page path without extension, for example runbooks/deploy-guide. Must not start with / or contain ..",
    ///  "type": "string",
    ///  "maxLength": 256,
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct WikiPagePath(::std::string::String);
    impl ::std::ops::Deref for WikiPagePath {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<WikiPagePath> for ::std::string::String {
        fn from(value: WikiPagePath) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for WikiPagePath {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 256usize {
                return Err("longer than 256 characters".into());
            }
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for WikiPagePath {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for WikiPagePath {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for WikiPagePath {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for WikiPagePath {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///Full Wiki page with content
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Full Wiki page with content",
    ///  "type": "object",
    ///  "required": [
    ///    "content",
    ///    "description",
    ///    "id",
    ///    "title"
    ///  ],
    ///  "properties": {
    ///    "content": {
    ///      "description": "Full file content including YAML frontmatter",
    ///      "type": "string"
    ///    },
    ///    "createdAt": {
    ///      "description": "RFC3339 timestamp when the Wiki page was created",
    ///      "type": "string"
    ///    },
    ///    "description": {
    ///      "description": "Short Wiki page description from YAML frontmatter",
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "tags": {
    ///      "description": "Wiki page tags from YAML frontmatter",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "title": {
    ///      "type": "string"
    ///    },
    ///    "updatedAt": {
    ///      "description": "RFC3339 timestamp when the Wiki page was last updated",
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "description": "Workspace that owns this Wiki page. Omitted for default Wiki pages.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageResponse {
        ///Full file content including YAML frontmatter
        pub content: ::std::string::String,
        ///RFC3339 timestamp when the Wiki page was created
        #[serde(
            rename = "createdAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub created_at: ::std::option::Option<::std::string::String>,
        ///Short Wiki page description from YAML frontmatter
        pub description: ::std::string::String,
        pub id: ::std::string::String,
        ///Wiki page tags from YAML frontmatter
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub tags: ::std::vec::Vec<::std::string::String>,
        pub title: ::std::string::String,
        ///RFC3339 timestamp when the Wiki page was last updated
        #[serde(
            rename = "updatedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_at: ::std::option::Option<::std::string::String>,
        ///Workspace that owns this Wiki page. Omitted for default Wiki pages.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///A stored prior version of a Wiki page
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A stored prior version of a Wiki page",
    ///  "type": "object",
    ///  "required": [
    ///    "rev",
    ///    "savedAt",
    ///    "size"
    ///  ],
    ///  "properties": {
    ///    "content": {
    ///      "description": "Revision content. Present only when fetching a single revision.",
    ///      "type": "string"
    ///    },
    ///    "rev": {
    ///      "description": "Opaque revision identifier",
    ///      "type": "string"
    ///    },
    ///    "savedAt": {
    ///      "description": "When the revision was stored",
    ///      "type": "string"
    ///    },
    ///    "size": {
    ///      "description": "Revision content size in bytes",
    ///      "type": "integer",
    ///      "format": "int64"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageRevisionResponse {
        ///Revision content. Present only when fetching a single revision.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub content: ::std::option::Option<::std::string::String>,
        ///Opaque revision identifier
        pub rev: ::std::string::String,
        ///When the revision was stored
        #[serde(rename = "savedAt")]
        pub saved_at: ::std::string::String,
        ///Revision content size in bytes
        pub size: i64,
    }
    ///Stored revisions of a Wiki page, newest first
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Stored revisions of a Wiki page, newest first",
    ///  "type": "object",
    ///  "required": [
    ///    "revisions"
    ///  ],
    ///  "properties": {
    ///    "revisions": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WikiPageRevisionResponse"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageRevisionsResponse {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub revisions: ::std::vec::Vec<WikiPageRevisionResponse>,
    }
    ///Cursor-based Wiki page search results
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Cursor-based Wiki page search results",
    ///  "type": "object",
    ///  "required": [
    ///    "hasMore",
    ///    "results"
    ///  ],
    ///  "properties": {
    ///    "hasMore": {
    ///      "type": "boolean"
    ///    },
    ///    "nextCursor": {
    ///      "type": "string"
    ///    },
    ///    "results": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WikiPageSearchPageItem"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageSearchFeedResponse {
        #[serde(rename = "hasMore")]
        pub has_more: bool,
        #[serde(
            rename = "nextCursor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub next_cursor: ::std::option::Option<::std::string::String>,
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub results: ::std::vec::Vec<WikiPageSearchPageItem>,
    }
    ///Lightweight cursor-based search result item for a Wiki page
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Lightweight cursor-based search result item for a Wiki page",
    ///  "type": "object",
    ///  "required": [
    ///    "description",
    ///    "hasMoreMatches",
    ///    "id",
    ///    "matches",
    ///    "title"
    ///  ],
    ///  "properties": {
    ///    "description": {
    ///      "description": "Short Wiki page description from YAML frontmatter",
    ///      "type": "string"
    ///    },
    ///    "hasMoreMatches": {
    ///      "description": "Whether additional snippets are available beyond the preview",
    ///      "type": "boolean"
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "matches": {
    ///      "description": "Preview snippets for the result",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/SearchMatchItem"
    ///      }
    ///    },
    ///    "modifiedAt": {
    ///      "description": "Last modification time of the Wiki page file",
    ///      "type": "string"
    ///    },
    ///    "nextMatchesCursor": {
    ///      "description": "Opaque cursor for loading more snippets for this Wiki page result",
    ///      "type": "string"
    ///    },
    ///    "tags": {
    ///      "description": "Wiki page tags from YAML frontmatter",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "title": {
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "description": "Workspace that owns this Wiki page. Omitted for default Wiki pages.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageSearchPageItem {
        ///Short Wiki page description from YAML frontmatter
        pub description: ::std::string::String,
        ///Whether additional snippets are available beyond the preview
        #[serde(rename = "hasMoreMatches")]
        pub has_more_matches: bool,
        pub id: ::std::string::String,
        ///Preview snippets for the result
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub matches: ::std::vec::Vec<SearchMatchItem>,
        ///Last modification time of the Wiki page file
        #[serde(
            rename = "modifiedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub modified_at: ::std::option::Option<::std::string::String>,
        ///Opaque cursor for loading more snippets for this Wiki page result
        #[serde(
            rename = "nextMatchesCursor",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub next_matches_cursor: ::std::option::Option<::std::string::String>,
        ///Wiki page tags from YAML frontmatter
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub tags: ::std::vec::Vec<::std::string::String>,
        pub title: ::std::string::String,
        ///Workspace that owns this Wiki page. Omitted for default Wiki pages.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///Search results
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Search results",
    ///  "type": "object",
    ///  "required": [
    ///    "results"
    ///  ],
    ///  "properties": {
    ///    "results": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WikiPageSearchResultItem"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageSearchResponse {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub results: ::std::vec::Vec<WikiPageSearchResultItem>,
    }
    ///A search result for a single Wiki page
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A search result for a single Wiki page",
    ///  "type": "object",
    ///  "required": [
    ///    "description",
    ///    "id",
    ///    "title"
    ///  ],
    ///  "properties": {
    ///    "description": {
    ///      "description": "Short Wiki page description from YAML frontmatter",
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "matchCount": {
    ///      "description": "Total number of query matches in the Wiki page",
    ///      "type": "integer"
    ///    },
    ///    "matches": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/SearchMatchItem"
    ///      }
    ///    },
    ///    "modifiedAt": {
    ///      "description": "Last modification time of the Wiki page file",
    ///      "type": "string"
    ///    },
    ///    "tags": {
    ///      "description": "Wiki page tags from YAML frontmatter",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "title": {
    ///      "type": "string"
    ///    },
    ///    "workspace": {
    ///      "description": "Workspace that owns this Wiki page. Omitted for default Wiki pages.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageSearchResultItem {
        ///Short Wiki page description from YAML frontmatter
        pub description: ::std::string::String,
        pub id: ::std::string::String,
        ///Total number of query matches in the Wiki page
        #[serde(
            rename = "matchCount",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub match_count: ::std::option::Option<i64>,
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub matches: ::std::vec::Vec<SearchMatchItem>,
        ///Last modification time of the Wiki page file
        #[serde(
            rename = "modifiedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub modified_at: ::std::option::Option<::std::string::String>,
        ///Wiki page tags from YAML frontmatter
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub tags: ::std::vec::Vec<::std::string::String>,
        pub title: ::std::string::String,
        ///Workspace that owns this Wiki page. Omitted for default Wiki pages.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///A file or directory node in the Wiki page tree
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "A file or directory node in the Wiki page tree",
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "name",
    ///    "type"
    ///  ],
    ///  "properties": {
    ///    "children": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WikiPageTreeNodeResponse"
    ///      }
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "modifiedAt": {
    ///      "description": "Last modification time. For files: file mtime. For directories: most recent descendant mtime.",
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "tags": {
    ///      "description": "Wiki page tags from YAML frontmatter. Only present on file nodes.",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "title": {
    ///      "type": "string"
    ///    },
    ///    "type": {
    ///      "type": "string",
    ///      "enum": [
    ///        "file",
    ///        "directory"
    ///      ]
    ///    },
    ///    "workspace": {
    ///      "description": "Workspace that owns this node. Omitted for default nodes.",
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WikiPageTreeNodeResponse {
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub children: ::std::vec::Vec<WikiPageTreeNodeResponse>,
        pub id: ::std::string::String,
        ///Last modification time. For files: file mtime. For directories: most recent descendant mtime.
        #[serde(
            rename = "modifiedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub modified_at: ::std::option::Option<::std::string::String>,
        pub name: ::std::string::String,
        ///Wiki page tags from YAML frontmatter. Only present on file nodes.
        #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
        #[serde(deserialize_with = "deserialize_null_default")]
        pub tags: ::std::vec::Vec<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub title: ::std::option::Option<::std::string::String>,
        #[serde(rename = "type")]
        pub type_: WikiPageTreeNodeResponseType,
        ///Workspace that owns this node. Omitted for default nodes.
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub workspace: ::std::option::Option<::std::string::String>,
    }
    ///`WikiPageTreeNodeResponseType`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "file",
    ///    "directory"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum WikiPageTreeNodeResponseType {
        #[serde(rename = "file")]
        File,
        #[serde(rename = "directory")]
        Directory,
    }
    impl ::std::fmt::Display for WikiPageTreeNodeResponseType {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::File => f.write_str("file"),
                Self::Directory => f.write_str("directory"),
            }
        }
    }
    impl ::std::str::FromStr for WikiPageTreeNodeResponseType {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "file" => Ok(Self::File),
                "directory" => Ok(Self::Directory),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for WikiPageTreeNodeResponseType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String>
    for WikiPageTreeNodeResponseType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String>
    for WikiPageTreeNodeResponseType {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Information about a distributed worker
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Information about a distributed worker",
    ///  "type": "object",
    ///  "required": [
    ///    "busyPollers",
    ///    "healthStatus",
    ///    "id",
    ///    "labels",
    ///    "lastHeartbeatAt",
    ///    "runningTasks",
    ///    "totalPollers"
    ///  ],
    ///  "properties": {
    ///    "busyPollers": {
    ///      "description": "Number of pollers currently executing tasks",
    ///      "type": "integer"
    ///    },
    ///    "healthStatus": {
    ///      "$ref": "#/components/schemas/WorkerHealthStatus"
    ///    },
    ///    "id": {
    ///      "description": "Unique identifier for the worker",
    ///      "type": "string"
    ///    },
    ///    "labels": {
    ///      "description": "Key-value pairs of labels assigned to the worker",
    ///      "type": "object",
    ///      "additionalProperties": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "lastHeartbeatAt": {
    ///      "description": "RFC3339 timestamp of the last heartbeat received from this worker",
    ///      "type": "string"
    ///    },
    ///    "runningTasks": {
    ///      "description": "List of tasks currently being executed by this worker",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/RunningTask"
    ///      }
    ///    },
    ///    "totalPollers": {
    ///      "description": "Total number of pollers configured for this worker",
    ///      "type": "integer"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct Worker {
        ///Number of pollers currently executing tasks
        #[serde(rename = "busyPollers")]
        pub busy_pollers: i64,
        #[serde(rename = "healthStatus")]
        pub health_status: WorkerHealthStatus,
        ///Unique identifier for the worker
        pub id: ::std::string::String,
        ///Key-value pairs of labels assigned to the worker
        pub labels: ::std::collections::HashMap<
            ::std::string::String,
            ::std::string::String,
        >,
        ///RFC3339 timestamp of the last heartbeat received from this worker
        #[serde(rename = "lastHeartbeatAt")]
        pub last_heartbeat_at: ::std::string::String,
        ///List of tasks currently being executed by this worker
        #[serde(rename = "runningTasks")]
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub running_tasks: ::std::vec::Vec<RunningTask>,
        ///Total number of pollers configured for this worker
        #[serde(rename = "totalPollers")]
        pub total_pollers: i64,
    }
    ///Health status of the worker based on heartbeat recency
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Health status of the worker based on heartbeat recency",
    ///  "type": "string",
    ///  "enum": [
    ///    "healthy",
    ///    "warning",
    ///    "unhealthy"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        ::serde::Deserialize,
        ::serde::Serialize,
        Clone,
        Copy,
        Debug,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd
    )]
    pub enum WorkerHealthStatus {
        #[serde(rename = "healthy")]
        Healthy,
        #[serde(rename = "warning")]
        Warning,
        #[serde(rename = "unhealthy")]
        Unhealthy,
    }
    impl ::std::fmt::Display for WorkerHealthStatus {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match *self {
                Self::Healthy => f.write_str("healthy"),
                Self::Warning => f.write_str("warning"),
                Self::Unhealthy => f.write_str("unhealthy"),
            }
        }
    }
    impl ::std::str::FromStr for WorkerHealthStatus {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            match value {
                "healthy" => Ok(Self::Healthy),
                "warning" => Ok(Self::Warning),
                "unhealthy" => Ok(Self::Unhealthy),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl ::std::convert::TryFrom<&str> for WorkerHealthStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for WorkerHealthStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for WorkerHealthStatus {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Response object for listing distributed workers
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Response object for listing distributed workers",
    ///  "type": "object",
    ///  "required": [
    ///    "errors",
    ///    "workers"
    ///  ],
    ///  "properties": {
    ///    "errors": {
    ///      "description": "List of errors encountered during the request",
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "workers": {
    ///      "description": "List of distributed workers",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/Worker"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WorkersListResponse {
        ///List of errors encountered during the request
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub errors: ::std::vec::Vec<::std::string::String>,
        ///List of distributed workers
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub workers: ::std::vec::Vec<Worker>,
    }
    ///Workspace access policy. all=true grants the top-level role in every workspace. all=false requires explicit workspace grants and a top-level viewer role.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Workspace access policy. all=true grants the top-level role in every workspace. all=false requires explicit workspace grants and a top-level viewer role.",
    ///  "type": "object",
    ///  "required": [
    ///    "all",
    ///    "grants"
    ///  ],
    ///  "properties": {
    ///    "all": {
    ///      "description": "Whether this identity can access all workspaces",
    ///      "type": "boolean"
    ///    },
    ///    "grants": {
    ///      "description": "Workspace-specific grants used when all=false",
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WorkspaceGrant"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WorkspaceAccess {
        ///Whether this identity can access all workspaces
        pub all: bool,
        ///Workspace-specific grants used when all=false
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub grants: ::std::vec::Vec<WorkspaceGrant>,
    }
    ///Role granted for a specific workspace
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Role granted for a specific workspace",
    ///  "type": "object",
    ///  "required": [
    ///    "role",
    ///    "workspace"
    ///  ],
    ///  "properties": {
    ///    "role": {
    ///      "$ref": "#/components/schemas/UserRole"
    ///    },
    ///    "workspace": {
    ///      "$ref": "#/components/schemas/WorkspaceName"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WorkspaceGrant {
        pub role: UserRole,
        pub workspace: WorkspaceName,
    }
    ///`WorkspaceListResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "workspaces"
    ///  ],
    ///  "properties": {
    ///    "workspaces": {
    ///      "type": "array",
    ///      "items": {
    ///        "$ref": "#/components/schemas/WorkspaceResponse"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WorkspaceListResponse {
        #[serde(default, deserialize_with = "deserialize_null_default")]
        pub workspaces: ::std::vec::Vec<WorkspaceResponse>,
    }
    ///Workspace name. The reserved names all, default, and global are not allowed.
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "description": "Workspace name. The reserved names all, default, and global are not allowed.",
    ///  "type": "string",
    ///  "maxLength": 64,
    ///  "minLength": 1
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    #[serde(transparent)]
    pub struct WorkspaceName(::std::string::String);
    impl ::std::ops::Deref for WorkspaceName {
        type Target = ::std::string::String;
        fn deref(&self) -> &::std::string::String {
            &self.0
        }
    }
    impl ::std::convert::From<WorkspaceName> for ::std::string::String {
        fn from(value: WorkspaceName) -> Self {
            value.0
        }
    }
    impl ::std::str::FromStr for WorkspaceName {
        type Err = self::error::ConversionError;
        fn from_str(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            if value.chars().count() > 64usize {
                return Err("longer than 64 characters".into());
            }
            if value.chars().count() < 1usize {
                return Err("shorter than 1 characters".into());
            }
            Ok(Self(value.to_string()))
        }
    }
    impl ::std::convert::TryFrom<&str> for WorkspaceName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &str,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<&::std::string::String> for WorkspaceName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: &::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl ::std::convert::TryFrom<::std::string::String> for WorkspaceName {
        type Error = self::error::ConversionError;
        fn try_from(
            value: ::std::string::String,
        ) -> ::std::result::Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl<'de> ::serde::Deserialize<'de> for WorkspaceName {
        fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
        where
            D: ::serde::Deserializer<'de>,
        {
            ::std::string::String::deserialize(deserializer)?
                .parse()
                .map_err(|e: self::error::ConversionError| {
                    <D::Error as ::serde::de::Error>::custom(e.to_string())
                })
        }
    }
    ///`WorkspaceResponse`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "id",
    ///    "name"
    ///  ],
    ///  "properties": {
    ///    "createdAt": {
    ///      "type": "string"
    ///    },
    ///    "description": {
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "$ref": "#/components/schemas/WorkspaceName"
    ///    },
    ///    "updatedAt": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WorkspaceResponse {
        #[serde(
            rename = "createdAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub created_at: ::std::option::Option<::std::string::String>,
        #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
        pub description: ::std::option::Option<::std::string::String>,
        pub id: ::std::string::String,
        pub name: WorkspaceName,
        #[serde(
            rename = "updatedAt",
            default,
            skip_serializing_if = "::std::option::Option::is_none"
        )]
        pub updated_at: ::std::option::Option<::std::string::String>,
    }
    ///`WriteSecretVersionRequest`
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "required": [
    ///    "value"
    ///  ],
    ///  "properties": {
    ///    "value": {
    ///      "writeOnly": true,
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
    pub struct WriteSecretVersionRequest {
        pub value: ::std::string::String,
    }
    /// Generation of default values for serde.
    pub mod defaults {
        pub(super) fn create_remote_node_request_auth_type() -> super::CreateRemoteNodeRequestAuthType {
            super::CreateRemoteNodeRequestAuthType::None
        }
        pub(super) fn view_spec_type() -> super::ViewSpecType {
            super::ViewSpecType::Kanban
        }
    }
}
#[derive(Clone, Debug)]
/**Client for Dagu

API for controlling and monitoring Dagu server.

Version: 1.0.0*/
pub struct Client {
    pub(crate) baseurl: String,
    pub(crate) client: reqwest::Client,
}
impl Client {
    /// Create a new client.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new(baseurl: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = {
            let dur = ::std::time::Duration::from_secs(15u64);
            reqwest::ClientBuilder::new().connect_timeout(dur).timeout(dur)
        };
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::ClientBuilder::new();
        Self::new_with_client(baseurl, client.build().unwrap())
    }
    /// Construct a new client with an existing `reqwest::Client`,
    /// allowing more control over its configuration.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new_with_client(baseurl: &str, client: reqwest::Client) -> Self {
        Self {
            baseurl: baseurl.to_string(),
            client,
        }
    }
}
impl ClientInfo<()> for Client {
    fn api_version() -> &'static str {
        "1.0.0"
    }
    fn baseurl(&self) -> &str {
        self.baseurl.as_str()
    }
    fn client(&self) -> &reqwest::Client {
        &self.client
    }
    fn inner(&self) -> &() {
        &()
    }
}
impl ClientHooks<()> for &Client {}
#[allow(clippy::all)]
impl Client {
    /**List all API keys

Returns all API keys. Requires admin role.

Sends a `GET` request to `/api-keys`

*/
    pub async fn list_api_keys<'a>(
        &'a self,
    ) -> Result<ResponseValue<types::ApiKeysListResponse>, Error<types::Error>> {
        let url = format!("{}/api-keys", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "list_api_keys",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            401u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            403u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Create API key

Full key returned only in this response. Community edition installs can create up to 2 API keys.

Sends a `POST` request to `/api-keys`

*/
    pub async fn create_api_key<'a>(
        &'a self,
        body: &'a types::CreateApiKeyRequest,
    ) -> Result<ResponseValue<types::CreateApiKeyResponse>, Error<types::Error>> {
        let url = format!("{}/api-keys", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "create_api_key",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            201u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            401u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            403u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            409u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Authenticate user and obtain JWT token

Authenticates a user with username and password, returns a JWT token on success

Sends a `POST` request to `/auth/login`

*/
    pub async fn login<'a>(
        &'a self,
        body: &'a types::LoginRequest,
    ) -> Result<ResponseValue<types::LoginResponse>, Error<types::Error>> {
        let url = format!("{}/auth/login", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "login",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            401u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            429u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Create initial admin account

Creates the first admin user during initial setup. Only available when no users exist. Returns a JWT token for immediate login.

Sends a `POST` request to `/auth/setup`

Arguments:
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn setup<'a>(
        &'a self,
        remote_node: Option<&'a str>,
        body: &'a types::SetupRequest,
    ) -> Result<ResponseValue<types::LoginResponse>, Error<types::Error>> {
        let url = format!("{}/auth/setup", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "setup",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            403u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**List all DAG-runs

Retrieves a list of all DAG-runs with optional filtering by name and status

Sends a `GET` request to `/dag-runs`

Arguments:
- `cursor`: Opaque cursor for loading the next page of older DAG-runs
- `dag_run_id`: ID of the DAG-run or 'latest' to get the most recent DAG-run
- `from_date`: start datetime for filtering DAG-runs in ISO 8601 format with timezone
- `labels`: Filter DAG-runs by DAG labels (comma-separated). Returns runs from DAGs that have ALL specified labels. Mutually exclusive with `tags`; the server returns HTTP 400 if both are set.
- `limit`: Maximum number of DAG-runs to return (default 100)
- `name`: Filter DAG-runs by name
- `remote_node`: name of the remote node
- `status`: status of the DAG-run. Repeat the parameter to match multiple statuses.
- `tags`: Deprecated alias for `labels`; mutually exclusive with `labels`. Filter DAG-runs by DAG labels (comma-separated).
- `to_date`: end datetime for filtering DAG-runs in ISO 8601 format with timezone
- `workspace`: Workspace selector. For list and search APIs, use all, default, or a workspace name. Omitted means all.
*/
    pub async fn list_dag_runs<'a>(
        &'a self,
        cursor: Option<&'a str>,
        dag_run_id: Option<&'a types::DagRunId>,
        from_date: Option<&'a types::UnixTimestamp>,
        labels: Option<&'a str>,
        limit: Option<::std::num::NonZeroU64>,
        name: Option<&'a str>,
        remote_node: Option<&'a str>,
        status: Option<&'a ::std::vec::Vec<types::Status>>,
        tags: Option<&'a str>,
        to_date: Option<&'a types::UnixTimestamp>,
        workspace: Option<&'a types::ListDagRunsWorkspace>,
    ) -> Result<ResponseValue<types::DagRunsPageResponse>, Error<types::Error>> {
        let url = format!("{}/dag-runs", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&progenitor_client::QueryParam::new("cursor", &cursor))
            .query(&progenitor_client::QueryParam::new("dagRunId", &dag_run_id))
            .query(&progenitor_client::QueryParam::new("fromDate", &from_date))
            .query(&progenitor_client::QueryParam::new("labels", &labels))
            .query(&progenitor_client::QueryParam::new("limit", &limit))
            .query(&progenitor_client::QueryParam::new("name", &name))
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .query(&progenitor_client::QueryParam::new("status", &status))
            .query(&progenitor_client::QueryParam::new("tags", &tags))
            .query(&progenitor_client::QueryParam::new("toDate", &to_date))
            .query(&progenitor_client::QueryParam::new("workspace", &workspace))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "list_dag_runs",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Create and execute a DAG-run from inline spec

Creates a DAG-run directly from a provided DAG specification (YAML) and starts execution.

This endpoint does not require a pre-existing DAG file; the supplied `spec` is parsed and validated
similarly to `/dags/validate`, and the run is executed immediately if valid.


Sends a `POST` request to `/dag-runs`

Arguments:
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn execute_dag_run_from_spec<'a>(
        &'a self,
        remote_node: Option<&'a str>,
        body: &'a types::ExecuteDagRunFromSpecBody,
    ) -> Result<
        ResponseValue<types::ExecuteDagRunFromSpecResponse>,
        Error<types::Error>,
    > {
        let url = format!("{}/dag-runs", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "execute_dag_run_from_spec",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            409u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Enqueue a DAG-run from inline spec

Creates a DAG-run directly from a provided DAG specification (YAML) and enqueues it for execution.

This endpoint does not require a pre-existing DAG file; the supplied `spec` is parsed and validated
similarly to `/dags/validate`, and the run is persisted to the queue if valid.


Sends a `POST` request to `/dag-runs/enqueue`

Arguments:
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn enqueue_dag_run_from_spec<'a>(
        &'a self,
        remote_node: Option<&'a str>,
        body: &'a types::EnqueueDagRunFromSpecBody,
    ) -> Result<
        ResponseValue<types::EnqueueDagRunFromSpecResponse>,
        Error<types::Error>,
    > {
        let url = format!("{}/dag-runs/enqueue", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "enqueue_dag_run_from_spec",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            409u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Retrieve detailed status of a DAG-run

Fetches detailed status information about a specific DAG-run. Use 'latest' as the dagRunId to retrieve the most recent DAG-run for the specified DAG name.

Sends a `GET` request to `/dag-runs/{name}/{dagRunId}`

Arguments:
- `name`: name of the DAG
- `dag_run_id`: ID of the DAG-run or 'latest' to get the most recent DAG-run
- `remote_node`: name of the remote node
*/
    pub async fn get_dag_run_details<'a>(
        &'a self,
        name: &'a types::DagName,
        dag_run_id: &'a types::DagRunId,
        remote_node: Option<&'a str>,
    ) -> Result<ResponseValue<types::GetDagRunDetailsResponse>, Error<types::Error>> {
        let url = format!(
            "{}/dag-runs/{}/{}", self.baseurl, encode_path(& name.to_string()),
            encode_path(& dag_run_id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "get_dag_run_details",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Delete a DAG-run

Permanently removes a DAG-run record and its associated run data. Developer, manager, or admin only.

Sends a `DELETE` request to `/dag-runs/{name}/{dagRunId}`

Arguments:
- `name`: name of the DAG
- `dag_run_id`: ID of the DAG-run; must not be the special 'latest' alias
- `remote_node`: name of the remote node
*/
    pub async fn delete_dag_run<'a>(
        &'a self,
        name: &'a types::DagName,
        dag_run_id: &'a str,
        remote_node: Option<&'a str>,
    ) -> Result<ResponseValue<()>, Error<types::Error>> {
        let url = format!(
            "{}/dag-runs/{}/{}", self.baseurl, encode_path(& name.to_string()),
            encode_path(& dag_run_id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .delete(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "delete_dag_run",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            204u16 => Ok(ResponseValue::empty(response)),
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Queue a completed human-task checkpoint for resume

Queues a retry for a completed human-task checkpoint without requiring the previously submitted form values.

Sends a `POST` request to `/dag-runs/{name}/{dagRunId}/human-tasks/resume`

Arguments:
- `name`: name of the DAG
- `dag_run_id`: ID of the DAG-run; must not be the special 'latest' alias
- `remote_node`: name of the remote node
*/
    pub async fn resume_human_task_dag_run<'a>(
        &'a self,
        name: &'a types::DagName,
        dag_run_id: &'a str,
        remote_node: Option<&'a str>,
    ) -> Result<ResponseValue<types::HumanTaskResumeResponse>, Error<types::Error>> {
        let url = format!(
            "{}/dag-runs/{}/{}/human-tasks/resume", self.baseurl, encode_path(& name
            .to_string()), encode_path(& dag_run_id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "resume_human_task_dag_run",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            409u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            503u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Complete a waiting human task

Validates typed input against the stored human-task form, completes the step atomically, and queues the same DAG-run when no manual steps remain waiting.

Sends a `POST` request to `/dag-runs/{name}/{dagRunId}/human-tasks/{stepId}/complete`

Arguments:
- `name`: name of the DAG
- `dag_run_id`: ID of the DAG-run; must not be the special 'latest' alias
- `step_id`: explicit ID of the human-task step
- `remote_node`: name of the remote node
- `body`: Typed form input. The JSON request body is limited to 16 MiB.
*/
    pub async fn complete_human_task<'a>(
        &'a self,
        name: &'a types::DagName,
        dag_run_id: &'a str,
        step_id: &'a types::CompleteHumanTaskStepId,
        remote_node: Option<&'a str>,
        body: &'a types::HumanTaskInput,
    ) -> Result<ResponseValue<types::HumanTaskCompletionResponse>, Error<types::Error>> {
        let url = format!(
            "{}/dag-runs/{}/{}/human-tasks/{}/complete", self.baseurl, encode_path(& name
            .to_string()), encode_path(& dag_run_id.to_string()), encode_path(& step_id
            .to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "complete_human_task",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            409u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            413u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            503u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Reschedule DAG-run with a new run ID

Launch a fresh DAG-run from a historic execution while reusing its stored parameters.

Sends a `POST` request to `/dag-runs/{name}/{dagRunId}/reschedule`

Arguments:
- `name`: name of the DAG
- `dag_run_id`: ID of the DAG-run or 'latest' to get the most recent DAG-run
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn reschedule_dag_run<'a>(
        &'a self,
        name: &'a types::DagName,
        dag_run_id: &'a types::DagRunId,
        remote_node: Option<&'a str>,
        body: &'a types::RescheduleDagRunBody,
    ) -> Result<ResponseValue<types::RescheduleDagRunResponse>, Error<types::Error>> {
        let url = format!(
            "{}/dag-runs/{}/{}/reschedule", self.baseurl, encode_path(& name
            .to_string()), encode_path(& dag_run_id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "reschedule_dag_run",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            409u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Retry DAG-run execution

Retries the existing DAG-run by reusing its DAG-run ID rather than creating a new DAG-run. Resets only the selected step or reachable descendants.

Sends a `POST` request to `/dag-runs/{name}/{dagRunId}/retry`

Arguments:
- `name`: name of the DAG
- `dag_run_id`: ID of the DAG-run or 'latest' to get the most recent DAG-run
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn retry_dag_run<'a>(
        &'a self,
        name: &'a types::DagName,
        dag_run_id: &'a types::DagRunId,
        remote_node: Option<&'a str>,
        body: &'a types::RetryDagRunBody,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!(
            "{}/dag-runs/{}/{}/retry", self.baseurl, encode_path(& name.to_string()),
            encode_path(& dag_run_id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "retry_dag_run",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::empty(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Approve a waiting step

Approves a step that is in Waiting status, optionally providing input parameters that will be available as environment variables in subsequent steps

Sends a `POST` request to `/dag-runs/{name}/{dagRunId}/steps/{stepName}/approve`

Arguments:
- `name`: name of the DAG
- `dag_run_id`: ID of the DAG-run or 'latest' to get the most recent DAG-run
- `step_name`: name of the step
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn approve_dag_run_step<'a>(
        &'a self,
        name: &'a types::DagName,
        dag_run_id: &'a types::DagRunId,
        step_name: &'a str,
        remote_node: Option<&'a str>,
        body: &'a types::ApproveStepRequest,
    ) -> Result<ResponseValue<types::ApproveStepResponse>, Error<types::Error>> {
        let url = format!(
            "{}/dag-runs/{}/{}/steps/{}/approve", self.baseurl, encode_path(& name
            .to_string()), encode_path(& dag_run_id.to_string()), encode_path(& step_name
            .to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "approve_dag_run_step",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Reject a waiting step

Rejects a step that is in Waiting status, optionally providing a reason for rejection

Sends a `POST` request to `/dag-runs/{name}/{dagRunId}/steps/{stepName}/reject`

Arguments:
- `name`: name of the DAG
- `dag_run_id`: ID of the DAG-run or 'latest' to get the most recent DAG-run
- `step_name`: name of the step
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn reject_dag_run_step<'a>(
        &'a self,
        name: &'a types::DagName,
        dag_run_id: &'a types::DagRunId,
        step_name: &'a str,
        remote_node: Option<&'a str>,
        body: &'a types::RejectStepRequest,
    ) -> Result<ResponseValue<types::RejectStepResponse>, Error<types::Error>> {
        let url = format!(
            "{}/dag-runs/{}/{}/steps/{}/reject", self.baseurl, encode_path(& name
            .to_string()), encode_path(& dag_run_id.to_string()), encode_path(& step_name
            .to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "reject_dag_run_step",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Manually update a step's execution status

Changes the status of a specific step after the DAG-run is no longer active

Sends a `PATCH` request to `/dag-runs/{name}/{dagRunId}/steps/{stepName}/status`

Arguments:
- `name`: name of the DAG
- `dag_run_id`: ID of the DAG-run or 'latest' to get the most recent DAG-run
- `step_name`: name of the step
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn update_dag_run_step_status<'a>(
        &'a self,
        name: &'a types::DagName,
        dag_run_id: &'a types::DagRunId,
        step_name: &'a str,
        remote_node: Option<&'a str>,
        body: &'a types::UpdateDagRunStepStatusBody,
    ) -> Result<ResponseValue<()>, Error<types::Error>> {
        let url = format!(
            "{}/dag-runs/{}/{}/steps/{}/status", self.baseurl, encode_path(& name
            .to_string()), encode_path(& dag_run_id.to_string()), encode_path(& step_name
            .to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .patch(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "update_dag_run_step_status",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::empty(response)),
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Terminate or cancel a DAG-run

Forcefully stops a running DAG-run, or cancels a failed root DAG-run that is still pending DAG-level automatic retry.

Sends a `POST` request to `/dag-runs/{name}/{dagRunId}/stop`

Arguments:
- `name`: name of the DAG
- `dag_run_id`: ID of the DAG-run or 'latest' to get the most recent DAG-run
- `remote_node`: name of the remote node
*/
    pub async fn terminate_dag_run<'a>(
        &'a self,
        name: &'a types::DagName,
        dag_run_id: &'a types::DagRunId,
        remote_node: Option<&'a str>,
    ) -> Result<ResponseValue<()>, Error<()>> {
        let url = format!(
            "{}/dag-runs/{}/{}/stop", self.baseurl, encode_path(& name.to_string()),
            encode_path(& dag_run_id.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "terminate_dag_run",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::empty(response)),
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**List all available DAGs

Retrieves DAG definitions with optional filtering by name, labels, and active status (scheduled and not suspended).

Sends a `GET` request to `/dags`

Arguments:
- `active`: Filter to DAGs that have a schedule and are not suspended.
- `labels`: Filter DAGs by labels (comma-separated). Returns DAGs that have ALL specified labels. Mutually exclusive with `tags`; the server returns HTTP 400 if both are set.
- `name`: Filter DAGs by name
- `order`: Sort order (ascending or descending)
- `page`: page number of items to fetch (default is 1)
- `per_page`: number of items per page (default is 30, max is 100)
- `remote_node`: name of the remote node
- `sort`: Field to sort by:
- `name`: Sort alphabetically by DAG name (case-insensitive)
- `nextRun`: Sort by next scheduled run time. DAGs with earlier next run times appear first in ascending order. DAGs without schedules appear last.

- `tags`: Deprecated alias for `labels`; mutually exclusive with `labels`. Filter DAGs by labels (comma-separated).
- `workspace`: Workspace selector. For list and search APIs, use all, default, or a workspace name. Omitted means all.
*/
    pub async fn list_da_gs<'a>(
        &'a self,
        active: Option<bool>,
        labels: Option<&'a str>,
        name: Option<&'a str>,
        order: Option<types::ListDaGsOrder>,
        page: Option<::std::num::NonZeroU64>,
        per_page: Option<::std::num::NonZeroU64>,
        remote_node: Option<&'a str>,
        sort: Option<types::ListDaGsSort>,
        tags: Option<&'a str>,
        workspace: Option<&'a types::ListDaGsWorkspace>,
    ) -> Result<ResponseValue<types::ListDaGsResponse>, Error<()>> {
        let url = format!("{}/dags", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&progenitor_client::QueryParam::new("active", &active))
            .query(&progenitor_client::QueryParam::new("labels", &labels))
            .query(&progenitor_client::QueryParam::new("name", &name))
            .query(&progenitor_client::QueryParam::new("order", &order))
            .query(&progenitor_client::QueryParam::new("page", &page))
            .query(&progenitor_client::QueryParam::new("perPage", &per_page))
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .query(&progenitor_client::QueryParam::new("sort", &sort))
            .query(&progenitor_client::QueryParam::new("tags", &tags))
            .query(&progenitor_client::QueryParam::new("workspace", &workspace))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "list_da_gs",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Create a new DAG definition

Creates a new empty DAG file with the specified name

Sends a `POST` request to `/dags`

Arguments:
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn create_new_dag<'a>(
        &'a self,
        remote_node: Option<&'a str>,
        body: &'a types::CreateNewDagBody,
    ) -> Result<ResponseValue<types::CreateNewDagResponse>, Error<types::Error>> {
        let url = format!("{}/dags", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "create_new_dag",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            201u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Validate a DAG specification

Validates a DAG YAML specification without persisting any changes.

Returns a list of validation errors. When the spec can be partially parsed,
the response may also include parsed DAG details built with error-tolerant loading.


Sends a `POST` request to `/dags/validate`

Arguments:
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn validate_dag_spec<'a>(
        &'a self,
        remote_node: Option<&'a str>,
        body: &'a types::ValidateDagSpecBody,
    ) -> Result<ResponseValue<types::ValidateDagSpecResponse>, Error<()>> {
        let url = format!("{}/dags/validate", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "validate_dag_spec",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Retrieve comprehensive DAG information

Fetches detailed information about a specific DAG definition

Sends a `GET` request to `/dags/{fileName}`

Arguments:
- `file_name`: the name of the DAG file
- `remote_node`: name of the remote node
*/
    pub async fn get_dag_details<'a>(
        &'a self,
        file_name: &'a types::DagFileName,
        remote_node: Option<&'a str>,
    ) -> Result<ResponseValue<types::GetDagDetailsResponse>, Error<()>> {
        let url = format!(
            "{}/dags/{}", self.baseurl, encode_path(& file_name.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "get_dag_details",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Delete an existing DAG

Permanently removes a DAG definition from the system

Sends a `DELETE` request to `/dags/{fileName}`

Arguments:
- `file_name`: the name of the DAG file
- `remote_node`: name of the remote node
*/
    pub async fn delete_dag<'a>(
        &'a self,
        file_name: &'a types::DagFileName,
        remote_node: Option<&'a str>,
    ) -> Result<ResponseValue<()>, Error<types::Error>> {
        let url = format!(
            "{}/dags/{}", self.baseurl, encode_path(& file_name.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .delete(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "delete_dag",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            204u16 => Ok(ResponseValue::empty(response)),
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Enqueue a DAG-run from DAG

Creates a DAG-run from the DAG definition and adds it to the queue for execution

Sends a `POST` request to `/dags/{fileName}/enqueue`

Arguments:
- `file_name`: the name of the DAG file
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn enqueue_dagdag_run<'a>(
        &'a self,
        file_name: &'a types::DagFileName,
        remote_node: Option<&'a str>,
        body: &'a types::EnqueueDagdagRunBody,
    ) -> Result<ResponseValue<types::EnqueueDagdagRunResponse>, Error<types::Error>> {
        let url = format!(
            "{}/dags/{}/enqueue", self.baseurl, encode_path(& file_name.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "enqueue_dagdag_run",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            409u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Change DAG file ID

Changes the file ID of the DAG definition

Sends a `POST` request to `/dags/{fileName}/rename`

Arguments:
- `file_name`: the name of the DAG file
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn rename_dag<'a>(
        &'a self,
        file_name: &'a types::DagFileName,
        remote_node: Option<&'a str>,
        body: &'a types::RenameDagBody,
    ) -> Result<ResponseValue<()>, Error<types::Error>> {
        let url = format!(
            "{}/dags/{}/rename", self.baseurl, encode_path(& file_name.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "rename_dag",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => Ok(ResponseValue::empty(response)),
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Retrieve DAG specification

Fetches the YAML specification of a DAG definition

Sends a `GET` request to `/dags/{fileName}/spec`

Arguments:
- `file_name`: the name of the DAG file
- `remote_node`: name of the remote node
*/
    pub async fn get_dag_spec<'a>(
        &'a self,
        file_name: &'a types::DagFileName,
        remote_node: Option<&'a str>,
    ) -> Result<ResponseValue<types::GetDagSpecResponse>, Error<()>> {
        let url = format!(
            "{}/dags/{}/spec", self.baseurl, encode_path(& file_name.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "get_dag_spec",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Update DAG spec

Modifies the YAML specification of a DAG definition

Sends a `PUT` request to `/dags/{fileName}/spec`

Arguments:
- `file_name`: the name of the DAG file
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn update_dag_spec<'a>(
        &'a self,
        file_name: &'a types::DagFileName,
        remote_node: Option<&'a str>,
        body: &'a types::UpdateDagSpecBody,
    ) -> Result<ResponseValue<types::UpdateDagSpecResponse>, Error<()>> {
        let url = format!(
            "{}/dags/{}/spec", self.baseurl, encode_path(& file_name.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .put(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "update_dag_spec",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Create and execute a DAG-run from DAG

Creates a DAG-run from the DAG definition and starts its execution with optional parameters

Sends a `POST` request to `/dags/{fileName}/start`

Arguments:
- `file_name`: the name of the DAG file
- `remote_node`: name of the remote node
- `body`
*/
    pub async fn execute_dag<'a>(
        &'a self,
        file_name: &'a types::DagFileName,
        remote_node: Option<&'a str>,
        body: &'a types::ExecuteDagBody,
    ) -> Result<ResponseValue<types::ExecuteDagResponse>, Error<types::Error>> {
        let url = format!(
            "{}/dags/{}/start", self.baseurl, encode_path(& file_name.to_string()),
        );
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .query(&progenitor_client::QueryParam::new("remoteNode", &remote_node))
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "execute_dag",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            409u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Check server health status

Returns health information about the Dagu server

Sends a `GET` request to `/health`

*/
    pub async fn get_health_status<'a>(
        &'a self,
    ) -> Result<ResponseValue<types::HealthResponse>, Error<()>> {
        let url = format!("{}/health", self.baseurl,);
        let mut header_map = ::reqwest::header::HeaderMap::with_capacity(1usize);
        header_map
            .append(
                ::reqwest::header::HeaderName::from_static("api-version"),
                ::reqwest::header::HeaderValue::from_static(Self::api_version()),
            );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                ::reqwest::header::ACCEPT,
                ::reqwest::header::HeaderValue::from_static("application/json"),
            )
            .headers(header_map)
            .build()?;
        let info = OperationInfo {
            operation_id: "get_health_status",
        };
        self.pre(&mut request, &info).await?;
        let result = self.exec(request, &info).await;
        self.post(&result, &info).await?;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
}
/// Items consumers will typically use such as the Client.
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::Client;
}

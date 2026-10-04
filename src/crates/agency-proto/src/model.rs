//! Frozen public records and ticket authentication, independent of either database.
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::{self, Display};

pub const PROTOCOL: &str = "hctl2.agency.v1";
pub const MAX_DOCUMENT: usize = 4 * 1024 * 1024;
pub type Result<T> = std::result::Result<T, PortError>;

/// Compare credentials through the MAC SDK's constant-time verification.
pub fn credential_matches(supplied: &str, expected: &str) -> bool {
    let Ok(mut proof) = Hmac::<Sha256>::new_from_slice(supplied.as_bytes()) else {
        return false;
    };
    let Ok(mut verifier) = Hmac::<Sha256>::new_from_slice(expected.as_bytes()) else {
        return false;
    };
    proof.update(b"hctl2.agency.credential.v1");
    verifier.update(b"hctl2.agency.credential.v1");
    verifier
        .verify_slice(&proof.finalize().into_bytes())
        .is_ok()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PortError {
    pub code: String,
    pub message: String,
    pub recovery_action: String,
}
impl PortError {
    pub fn new(code: &str, message: impl Into<String>, recovery: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            recovery_action: recovery.into(),
        }
    }
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new("INVALID_INPUT", message, "correct_input")
    }
}
impl Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for PortError {}
impl From<serde_json::Error> for PortError {
    fn from(e: serde_json::Error) -> Self {
        Self::invalid(e.to_string())
    }
}
impl From<std::io::Error> for PortError {
    fn from(e: std::io::Error) -> Self {
        Self::new("IO_ERROR", e.to_string(), "check_storage_or_endpoint")
    }
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn hash(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
pub fn digest(value: &str) -> Result<()> {
    if value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(())
    } else {
        Err(PortError::invalid("expected lowercase SHA-256"))
    }
}
pub fn nonempty(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > 4096 || value.contains('\0') {
        Err(PortError::invalid("empty or invalid identifier"))
    } else {
        Ok(())
    }
}
pub fn canonical<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let value = serde_json::to_value(value)?;
    fn safe(value: &serde_json::Value) -> Result<()> {
        match value {
            serde_json::Value::Number(n)
                if !n.as_u64().is_some_and(|v| v <= 9_007_199_254_740_991)
                    && !n
                        .as_i64()
                        .is_some_and(|v| v.unsigned_abs() <= 9_007_199_254_740_991) =>
            {
                Err(PortError::invalid("non-integer or unsafe canonical number"))
            }
            serde_json::Value::Array(a) => a.iter().try_for_each(safe),
            serde_json::Value::Object(o) => o.values().try_for_each(safe),
            _ => Ok(()),
        }
    }
    safe(&value)?;
    let bytes =
        serde_json_canonicalizer::to_vec(&value).map_err(|e| PortError::invalid(e.to_string()))?;
    if bytes.len() > MAX_DOCUMENT {
        return Err(PortError::invalid("document limit exceeded"));
    }
    Ok(bytes)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FrozenRef {
    pub id: String,
    pub revision: String,
    pub digest: String,
}
impl FrozenRef {
    pub fn validate(&self) -> Result<()> {
        nonempty(&self.id)?;
        nonempty(&self.revision)?;
        digest(&self.digest)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Owner {
    pub project: String,
    pub kind: OwnerKind,
    pub id: String,
    pub generation: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OwnerKind {
    RoomInvocation,
    RunAttempt,
}
impl Owner {
    pub fn validate(&self) -> Result<()> {
        nonempty(&self.project)?;
        nonempty(&self.id)?;
        if self.generation == 0 {
            return Err(PortError::invalid("semantic generation is required"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub input: bool,
    pub stop: bool,
    pub event_cursor: bool,
    pub input_provenance: bool,
    pub managed_single_writer: bool,
    pub secure_input: bool,
    pub exact_attach: bool,
    pub tool_execution_unmediated: bool,
    pub isolation_effects: Vec<String>,
}
impl Capabilities {
    pub fn fulfills(&self, required: &Self) -> Result<()> {
        let missing: Vec<&str> = [
            (required.input, self.input, "input"),
            (required.stop, self.stop, "stop"),
            (required.event_cursor, self.event_cursor, "event_cursor"),
            (
                required.input_provenance,
                self.input_provenance,
                "input_provenance",
            ),
            (
                required.managed_single_writer,
                self.managed_single_writer,
                "managed_single_writer",
            ),
            (required.secure_input, self.secure_input, "secure_input"),
            (required.exact_attach, self.exact_attach, "exact_attach"),
            (
                required.tool_execution_unmediated,
                self.tool_execution_unmediated,
                "tool_execution_unmediated",
            ),
        ]
        .into_iter()
        .filter_map(|(need, has, name)| (need && !has).then_some(name))
        .chain(
            required
                .isolation_effects
                .iter()
                .filter(|e| !self.isolation_effects.contains(e))
                .map(String::as_str),
        )
        .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(PortError::new(
                "CAPABILITY_MISSING",
                missing.join(", "),
                "select_capable_agency",
            ))
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLevel {
    Unmediated,
    AdapterEvent,
    Narrated,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SkillClaim {
    pub reference: FrozenRef,
    pub required: bool,
    pub verification: Option<SkillVerification>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SkillVerification {
    pub source: EvidenceLevel,
    pub report: FrozenRef,
    pub readback_digest: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Profession {
    pub reference: FrozenRef,
    pub harness: FrozenRef,
    pub model: String,
    pub persona: String,
    pub terms: String,
    pub default_role: String,
    pub skills: Vec<SkillClaim>,
    pub capabilities: Capabilities,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub professions: Vec<Profession>,
    pub harnesses: Vec<FrozenRef>,
    pub skills: Vec<SkillClaim>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Sealed<T> {
    pub document: T,
    pub digest: String,
}
impl<T: Serialize> Sealed<T> {
    pub fn new(document: T) -> Result<Self> {
        let digest = hash(&canonical(&document)?);
        Ok(Self { document, digest })
    }
    pub fn verify(&self) -> Result<()> {
        if hash(&canonical(&self.document)?) == self.digest {
            Ok(())
        } else {
            Err(PortError::new(
                "DIGEST_MISMATCH",
                "frozen document changed",
                "rebuild_preview",
            ))
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InputPolicy {
    NativeInteractiveAllowed,
    ManagedSingleWriter,
    NoInput,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSpec {
    pub owner: Owner,
    pub project: FrozenRef,
    pub selection: FrozenRef,
    pub selection_policy_digest: String,
    pub profession: Profession,
    pub profile: FrozenRef,
    pub manifest: FrozenRef,
    pub bundle: FrozenRef,
    pub binding: FrozenRef,
    pub required_capabilities: Capabilities,
    pub input_policy: InputPolicy,
    pub permission_digest: String,
    pub permissions: Vec<String>,
    pub budget: u64,
    pub deadline_ms: u64,
    pub repo: Option<FrozenRef>,
    pub base: Option<String>,
    pub delivery_scope: Vec<String>,
    pub write_lease: Option<FrozenRef>,
    pub review_publish_policy: Option<FrozenRef>,
    pub idempotency_key: String,
}
impl ExecutionSpec {
    pub fn validate(&self) -> Result<()> {
        self.owner.validate()?;
        for r in [
            &self.project,
            &self.selection,
            &self.profession.reference,
            &self.profession.harness,
            &self.profile,
            &self.manifest,
            &self.bundle,
            &self.binding,
        ] {
            r.validate()?;
        }
        for d in [&self.selection_policy_digest, &self.permission_digest] {
            digest(d)?;
        }
        nonempty(&self.idempotency_key)?;
        if self.project.id != self.owner.project {
            return Err(PortError::invalid("Project and owner differ"));
        }
        for skill in &self.profession.skills {
            skill.reference.validate()?;
            if let Some(report) = &skill.verification {
                report.report.validate()?;
                digest(&report.readback_digest)?;
                if report.source != EvidenceLevel::Unmediated
                    || report.readback_digest != skill.reference.digest
                {
                    return Err(PortError::new(
                        "SKILL_DIGEST_MISMATCH",
                        &skill.reference.id,
                        "verify_skill",
                    ));
                }
            }
        }
        for r in [&self.repo, &self.write_lease, &self.review_publish_policy]
            .into_iter()
            .flatten()
        {
            r.validate()?;
        }
        if self.budget == 0 || self.deadline_ms == 0 {
            return Err(PortError::invalid("budget and deadline must be positive"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prepare {
    pub spec: Sealed<ExecutionSpec>,
    pub bundle: Sealed<crate::context::Bundle>,
    pub writer_generation: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Dispatch {
    pub reference: String,
    pub owner: Owner,
    pub spec_digest: String,
    pub bundle_digest: String,
    pub binding: FrozenRef,
    pub capabilities: Capabilities,
    pub state: DispatchState,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DispatchState {
    Prepared,
    Running,
    ResultReturned,
    CannotFulfill,
    Cancelled,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchAction {
    pub dispatch: String,
    pub writer_generation: u64,
    pub idempotency_key: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    pub control_id: String,
    /// Chosen and persisted by the client before the first RPC; proves replay ownership.
    pub tenant_key: String,
}
/// Secret-bearing response: deliberately no Debug or Serialize logging helper.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pairing {
    pub endpoint: String,
    pub key: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fence {
    pub writer_generation: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    Observe,
    Input,
    Stop,
    Takeover,
    SecureInput,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TicketClaims {
    pub id: String,
    pub actor: String,
    pub dispatch: String,
    pub owner: Owner,
    pub spec_digest: String,
    pub writer_generation: u64,
    pub permissions: Vec<Permission>,
    pub input_lease: Option<String>,
    pub expires_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ticket {
    pub claims: TicketClaims,
    pub mac: String,
}
impl Ticket {
    pub fn sign(claims: TicketClaims, key: &[u8]) -> Result<Self> {
        let mut signer = Hmac::<Sha256>::new_from_slice(key)
            .map_err(|_| PortError::invalid("invalid signing key"))?;
        signer.update(&canonical(&claims)?);
        Ok(Self {
            claims,
            mac: hex(&signer.finalize().into_bytes()),
        })
    }
    pub fn verify(&self, key: &[u8]) -> Result<()> {
        digest(&self.mac)?;
        let bytes: Vec<u8> = (0..64)
            .step_by(2)
            .map(|i| u8::from_str_radix(&self.mac[i..i + 2], 16).expect("validated hex"))
            .collect();
        let mut signer = Hmac::<Sha256>::new_from_slice(key)
            .map_err(|_| PortError::invalid("invalid signing key"))?;
        signer.update(&canonical(&self.claims)?);
        signer.verify_slice(&bytes).map_err(|_| {
            PortError::new(
                "TICKET_INVALID",
                "ticket authentication failed",
                "request_new_ticket",
            )
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lease {
    pub ticket: Ticket,
    pub expected_lease: Option<String>,
    pub new_lease: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub ticket: Ticket,
    pub idempotency_key: String,
    #[serde(with = "crate::bytes_base64")]
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observe {
    pub ticket: Ticket,
    pub after: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub sequence: u64,
    pub source: EvidenceLevel,
    pub confidence: String,
    pub evidence_digest: String,
    pub observed_ms: u64,
    pub kind: String,
    pub payload: serde_json::Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    pub dispatch: Dispatch,
    pub events: Vec<Observation>,
    pub cursor: u64,
    pub gap: bool,
    pub complete: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProposalHeader {
    pub proposal_id: String,
    pub owner: Owner,
    pub dispatch: String,
    pub spec_digest: String,
    pub bundle_digest: String,
    pub binding: FrozenRef,
    pub producer_sequence: u64,
    pub idempotency_key: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub header: ProposalHeader,
    pub schema: String,
    #[serde(with = "crate::bytes_base64")]
    pub output: Vec<u8>,
    pub content_digest: String,
    pub outputs: Vec<ProposalOutput>,
    pub evidence: EvidenceLevel,
    pub preserved: bool,
}
/// Each item retains its own authority; an accepted neighbour cannot lend it authority.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalOutput {
    pub schema: String,
    pub content_digest: String,
    pub candidate: FrozenRef,
    pub owner: Owner,
    pub dispatch: String,
    pub authorization: FrozenRef,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultQuery {
    pub dispatch: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lookup {
    pub idempotency_key: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preservation {
    pub dispatch: String,
    pub proposal_id: String,
    pub content_digest: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ticket_tampering_is_rejected() {
        let mut ticket = Ticket::sign(
            TicketClaims {
                id: "t".into(),
                actor: "human".into(),
                dispatch: "d".into(),
                owner: Owner {
                    project: "p".into(),
                    kind: OwnerKind::RoomInvocation,
                    id: "i".into(),
                    generation: 1,
                },
                spec_digest: "a".repeat(64),
                writer_generation: 1,
                permissions: vec![Permission::Observe],
                input_lease: None,
                expires_ms: 100,
            },
            b"key",
        )
        .unwrap();
        ticket.verify(b"key").unwrap();
        ticket.claims.permissions.push(Permission::Input);
        assert_eq!(ticket.verify(b"key").unwrap_err().code, "TICKET_INVALID");
    }
    #[test]
    fn capabilities_are_effects_not_silent_degradation() {
        let required = Capabilities {
            isolation_effects: vec!["no_network".into()],
            ..Capabilities::default()
        };
        assert_eq!(
            Capabilities::default()
                .fulfills(&required)
                .unwrap_err()
                .code,
            "CAPABILITY_MISSING"
        );
    }
    #[test]
    fn physical_fields_and_unsafe_numbers_are_rejected() {
        assert!(
            serde_json::from_value::<ProposalHeader>(serde_json::json!({"runtime_generation":1}))
                .is_err()
        );
        assert!(canonical(&serde_json::json!({"budget":9007199254740992_u64})).is_err());
    }
}

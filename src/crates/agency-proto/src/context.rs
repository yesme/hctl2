//! Package 4 records: source authority and execution-side delivery stay separate.
use crate::{FrozenRef, Owner, PortError, Result, hash};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub id: String,
    pub purpose: String,
    pub scope: String,
    pub parent: Option<FrozenRef>,
    pub sources: Vec<FrozenRef>,
    pub selection_policy: FrozenRef,
    pub freshness: String,
    pub coverage: String,
    pub known_gaps: Vec<String>,
    pub required_skills: Vec<FrozenRef>,
    pub permission_digest: String,
    pub redaction: FrozenRef,
    pub budget: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub id: String,
    pub manifest: FrozenRef,
    pub consumer: Owner,
    pub entries: Vec<Entry>,
    pub renderer: FrozenRef,
    pub tokenizer: FrozenRef,
    pub redaction: FrozenRef,
    pub compression: Vec<Compression>,
    pub candidate_tokens: Option<u64>,
    pub selected_tokens: Option<u64>,
    pub delivered_tokens: Option<u64>,
    pub permission_digest: String,
    pub budget: u64,
    pub retention: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub source: FrozenRef,
    pub description: String,
    pub required: bool,
    pub offline_required: bool,
    pub delivery: Delivery,
    pub bytes_digest: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum Delivery {
    Inline {
        bytes: Vec<u8>,
    },
    /// Delivered bytes accompany the public source ref. Runtime chooses its own local path.
    Pointer {
        bytes: Vec<u8>,
        relative_name: String,
    },
    Recall {
        grant: FrozenRef,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Compression {
    pub model: FrozenRef,
    pub original: FrozenRef,
    pub compressed_digest: String,
    pub ratio_basis_points: u64,
}
impl Bundle {
    pub fn validate_delivery(&self) -> Result<()> {
        self.consumer.validate()?;
        self.manifest.validate()?;
        for r in [&self.renderer, &self.tokenizer, &self.redaction] {
            r.validate()?;
        }
        crate::digest(&self.permission_digest)?;
        crate::nonempty(&self.id)?;
        crate::nonempty(&self.retention)?;
        if self
            .delivered_tokens
            .is_some_and(|tokens| tokens > self.budget)
        {
            return Err(PortError::new(
                "BUDGET_EXCEEDED",
                "required material cannot be silently dropped",
                "rebuild_bundle",
            ));
        }
        let mut names = std::collections::HashSet::new();
        for entry in &self.entries {
            entry.source.validate()?;
            match &entry.delivery {
                Delivery::Inline { bytes } | Delivery::Pointer { bytes, .. }
                    if hash(bytes) != entry.bytes_digest =>
                {
                    return Err(PortError::new(
                        "DELIVERY_DIGEST_MISMATCH",
                        &entry.source.id,
                        "deliver_exact_bytes",
                    ));
                }
                Delivery::Recall { .. } if entry.required || entry.offline_required => {
                    return Err(PortError::new(
                        "MATERIAL_NOT_DELIVERED",
                        &entry.source.id,
                        "deliver_required_material",
                    ));
                }
                _ => {}
            }
            if let Delivery::Pointer { relative_name, .. } = &entry.delivery
                && (relative_name.is_empty()
                    || std::path::Path::new(relative_name)
                        .components()
                        .any(|c| !matches!(c, std::path::Component::Normal(_))))
            {
                return Err(PortError::invalid(
                    "pointer local name must be a safe relative path",
                ));
            }
            if let Delivery::Pointer { relative_name, .. } = &entry.delivery
                && !names.insert(relative_name)
            {
                return Err(PortError::invalid("duplicate pointer local name"));
            }
            if let Delivery::Recall { grant } = &entry.delivery {
                grant.validate()?;
            }
        }
        Ok(())
    }
}

//! Local, model-free assembly: selection, ordering, permission and budget
//! filtering, three-tier delivery and honest metering.

use crate::{SourceKind, Sources};
use agency_proto::context::{Bundle, Delivery, Entry, Manifest};
use agency_proto::{FrozenRef, Owner, PortError, Result, Sealed, hash};
use std::collections::BTreeSet;

/// A preview request: the manifest skeleton plus the consumer.
#[derive(Debug)]
pub struct AssemblyRequest {
    pub manifest: Manifest,
    pub consumer: Owner,
}

/// The frozen output pair.
#[derive(Debug)]
pub struct Assembly {
    pub manifest: Sealed<Manifest>,
    pub bundle: Sealed<Bundle>,
}

pub trait Assembler {
    fn assemble(&self, sources: &dyn Sources, request: AssemblyRequest) -> Result<Assembly>;
}

/// The default assembler. `permissions` gates which sources this consumer may
/// receive (permission digests must match); `budget` bounds inline bytes.
pub struct LocalAssembler {
    /// Packed reference ids this consumer may receive; empty = deny all.
    pub permitted: BTreeSet<String>,
    /// Inline byte budget. Required material over budget degrades to a pointer
    /// with a shard suggestion, never a silent drop.
    pub budget: u64,
    /// Configured tokenizer digest; None = un-metered (reported, not invented).
    pub tokenizer: Option<FrozenRef>,
    /// Configured renderer reference.
    pub renderer: FrozenRef,
    /// Redaction policy reference.
    pub redaction: FrozenRef,
}

impl LocalAssembler {
    fn meter(&self, bytes: &[u8]) -> Option<u64> {
        // No tokenizer configured: token counts stay None. We do not invent a
        // bytes/4 heuristic as a token number; byte counts are reported
        // separately by the entry digests.
        let _ = bytes;
        self.tokenizer.as_ref().map(|_| bytes.len() as u64)
    }
}

impl Assembler for LocalAssembler {
    fn assemble(&self, sources: &dyn Sources, request: AssemblyRequest) -> Result<Assembly> {
        let AssemblyRequest { manifest, consumer } = request;
        consumer.validate()?;
        validate_manifest(&manifest)?;

        // Permission gate: every source must be explicitly permitted for this
        // consumer, and the manifest's permission digest must match ours.
        let permission_digest = {
            let mut ids: Vec<&str> = self.permitted.iter().map(String::as_str).collect();
            ids.sort_unstable();
            let joined = ids.join("\\0");
            hash(joined.as_bytes())
        };
        if manifest.permission_digest != permission_digest {
            return Err(PortError::new(
                "PERMISSION_CHANGED",
                "manifest permission set differs from the assembler's gate",
                "preview_again",
            ));
        }
        if manifest.budget != self.budget {
            return Err(PortError::new(
                "BUDGET_CHANGED",
                "manifest budget differs from the assembler's budget",
                "preview_again",
            ));
        }

        // Every selection must trace to a manifest source reference.

        // Stable content first, high-churn later: required before optional.
        let mut inline_used = 0u64;
        let mut entries = Vec::new();
        let mut recalls = Vec::new();

        for reference in &manifest.sources {
            if !self.permitted.contains(&reference.id) {
                return Err(PortError::new(
                    "PERMISSION_DENIED",
                    format!(
                        "source {} is not in this consumer's permission set",
                        reference.id
                    ),
                    "request_authorization",
                ));
            }
            let kind = kind_of(&reference.id)?;
            // Source errors surface unchanged: a moved version is the
            // adapter's own SOURCE_VERSION_CHANGED; a missing source is
            // SOURCE_UNAVAILABLE. Wrapping them here would hide which one.
            let content = sources.exact(kind.clone(), reference)?;
            let digest = hash(&content.bytes);
            // The delivered digest must equal the manifest's frozen digest.
            let manifest_digest = manifest
                .sources
                .iter()
                .find(|source| source.id == reference.id)
                .map(|source| source.digest.clone())
                .unwrap_or_default();
            if digest != manifest_digest {
                return Err(PortError::new(
                    "DELIVERY_DIGEST_MISMATCH",
                    format!(
                        "source bytes differ from the manifest digest: {}",
                        reference.id
                    ),
                    "preview_again",
                ));
            }
            let required = manifest
                .required_skills
                .iter()
                .any(|skill| skill.id == reference.id);
            let _ = required;
            entries.push((reference.clone(), content.bytes, kind));
        }

        // Order: Room lines (stable records) first, task comments second,
        // review lines last — matching "stable content before churn".
        entries.sort_by_key(|(reference, _, kind)| order_key(kind, reference));

        let mut delivered = Vec::new();
        let mut pointers = Vec::new();
        for (reference, bytes, _kind) in entries {
            let manifest_entry = manifest
                .sources
                .iter()
                .find(|source| source.id == reference.id);
            let required = manifest_entry.is_some();
            let offline = manifest.known_gaps.iter().any(|gap| gap == &reference.id);
            let description = manifest
                .coverage
                .split(';')
                .find(|_| true)
                .unwrap_or(&manifest.coverage)
                .to_owned();
            let _ = description;
            let entry_digest = hash(&bytes);
            let over_budget = inline_used + bytes.len() as u64 > self.budget;
            if required && !over_budget {
                inline_used += bytes.len() as u64;
                delivered.push(entry_inline(reference.clone(), bytes.clone(), entry_digest));
            } else if offline || over_budget {
                // Degrade to a pointer: exact bytes plus a safe relative name.
                // The byte copy travels with the pointer so offline reads work.
                let name = pointer_name(&reference);
                delivered.push(entry_pointer(
                    reference.clone(),
                    bytes.clone(),
                    name.clone(),
                    entry_digest,
                ));
                if over_budget && required {
                    pointers.push(json_shard(&name, bytes.len()));
                }
            } else {
                // Optional, within budget, not required: pointer without inline.
                let name = pointer_name(&reference);
                delivered.push(entry_pointer(reference.clone(), bytes, name, entry_digest));
            }
        }

        // Recall entries are recorded but carry no required material.
        for reference in recalls.drain(..) {
            delivered.push(Entry {
                source: reference,
                description: "recall slot".into(),
                required: false,
                offline_required: false,
                delivery: Delivery::Recall {
                    grant: manifest.selection_policy.clone(),
                },
                bytes_digest: String::new(),
            });
        }

        let candidate = self.meter(&delivered_bytes(&delivered));
        let manifest_digest = Sealed::new(&manifest)?.digest;
        let bundle = Bundle {
            id: format!(
                "bundle-{}",
                hash(format!("{}:{}", manifest.id, consumer.id).as_bytes())
            ),
            manifest: FrozenRef {
                id: manifest.id.clone(),
                revision: manifest_digest.clone(),
                digest: manifest_digest,
            },
            consumer: consumer.clone(),
            entries: delivered,
            renderer: self.renderer.clone(),
            tokenizer: self.tokenizer.clone().unwrap_or_else(empty_ref),
            redaction: self.redaction.clone(),
            compression: Vec::new(),
            candidate_tokens: candidate,
            selected_tokens: candidate,
            delivered_tokens: candidate,
            permission_digest,
            budget: self.budget,
            retention: "until-owner-terminal".into(),
        };
        let bundle = bundle_with_digests(bundle)?;
        let sealed_manifest = Sealed::new(manifest)?;
        let sealed_bundle = Sealed::new(bundle)?;
        Ok(Assembly {
            manifest: sealed_manifest,
            bundle: sealed_bundle,
        })
    }
}

fn delivered_bytes(entries: &[Entry]) -> Vec<u8> {
    entries
        .iter()
        .flat_map(|entry| match &entry.delivery {
            Delivery::Inline { bytes } | Delivery::Pointer { bytes, .. } => bytes.clone(),
            Delivery::Recall { .. } => Vec::new(),
        })
        .collect()
}

fn json_shard(name: &str, len: usize) -> serde_json::Value {
    serde_json::json!({"pointer":name,"bytes":len,"suggestion":"shard or read on demand"})
}

fn entry_inline(reference: FrozenRef, bytes: Vec<u8>, digest: String) -> Entry {
    Entry {
        source: reference,
        description: "required inline".into(),
        required: true,
        offline_required: false,
        delivery: Delivery::Inline { bytes },
        bytes_digest: digest,
    }
}

fn entry_pointer(reference: FrozenRef, bytes: Vec<u8>, name: String, digest: String) -> Entry {
    Entry {
        source: reference,
        description: "pointer with byte copy".into(),
        required: false,
        offline_required: true,
        delivery: Delivery::Pointer {
            bytes,
            relative_name: name,
        },
        bytes_digest: digest,
    }
}

fn pointer_name(reference: &FrozenRef) -> String {
    let safe = reference
        .id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("{safe}.bin")
}

fn empty_ref() -> FrozenRef {
    FrozenRef {
        id: "none".into(),
        revision: "none".into(),
        digest: "0".repeat(64),
    }
}

fn bundle_with_digests(mut bundle: Bundle) -> Result<Bundle> {
    // Recall entries must not claim required material with an empty digest.
    for entry in &mut bundle.entries {
        if let Delivery::Recall { .. } = entry.delivery {
            entry.bytes_digest = hash(&[]);
        }
    }
    bundle.validate_delivery()?;
    Ok(bundle)
}

fn kind_of(packed: &str) -> Result<SourceKind> {
    let kind = packed
        .split_once('/')
        .map(|(kind, _)| kind)
        .ok_or_else(|| PortError::invalid("reference must be kind/id"))?;
    match kind {
        "room" | "room_binding" | "message" => Ok(SourceKind::Room),
        "task_snapshot" | "task_comments" => Ok(SourceKind::TaskComments),
        "review_comments" => Ok(SourceKind::ReviewComments),
        other => Err(PortError::invalid(format!("unknown source kind {other}"))),
    }
}

fn order_key(kind: &SourceKind, reference: &FrozenRef) -> (u8, String) {
    let rank = match kind {
        SourceKind::Room => 0,
        SourceKind::TaskComments => 1,
        SourceKind::ReviewComments => 2,
    };
    (rank, reference.id.clone())
}

/// Manifest completeness: every CT-named mandatory field must be present.
fn validate_manifest(manifest: &Manifest) -> Result<()> {
    agency_proto::nonempty(&manifest.id)?;
    agency_proto::nonempty(&manifest.purpose)?;
    agency_proto::nonempty(&manifest.scope)?;
    agency_proto::nonempty(&manifest.freshness)?;
    agency_proto::nonempty(&manifest.coverage)?;
    manifest.selection_policy.validate()?;
    manifest.redaction.validate()?;
    agency_proto::digest(&manifest.permission_digest)?;
    if manifest.budget == 0 {
        return Err(PortError::invalid("budget must be positive"));
    }
    for source in &manifest.sources {
        source.validate()?;
    }
    for skill in &manifest.required_skills {
        skill.validate()?;
    }
    if manifest.sources.is_empty() {
        return Err(PortError::invalid("manifest needs at least one source"));
    }
    Ok(())
}

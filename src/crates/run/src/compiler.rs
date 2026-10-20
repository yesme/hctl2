//! HCTL lint precedes native Dagu validation. No engine DSL passes through.
use crate::model::*;
use chat::{invalid, reject};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use store::Result;

pub const COMPILER: &str = "hctl2.workflow.v1";
pub const DAGU_VERSION: &str = "2.15.1";
/// Fixed compiler resource ceiling, checked before allocating expanded nodes.
pub const MAX_CHECKPOINTS: u64 = 16_384;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Warning {
    pub node: String,
    pub seats: Vec<String>,
    pub message: String,
    pub missing: Vec<String>,
    pub options: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub id: String,
    pub node: Node,
    pub depends: Vec<String>,
    pub role: CheckpointRole,
    /// Frozen expansion identity; reducers must not parse step names for authority.
    pub activation: Vec<Activation>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointRole {
    Node,
    CompositeExit,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Activation {
    LoopRound { checkpoint: String, round: u32 },
    ForkSlot { checkpoint: String, slot: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Compiled {
    pub compiler: String,
    pub dagu_version: String,
    pub checkpoints: Vec<Checkpoint>,
    /// This is also the Manifest input; callers must not recompute defaults.
    pub frozen_rework_limits: BTreeMap<String, u32>,
    pub warnings: Vec<Warning>,
    pub yaml: String,
}
fn fail(code: &'static str, message: impl Into<String>) -> store::StoreError {
    reject(code, message, "edit_workflow")
}
fn id(value: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err(invalid(
            "node/seat/output ids use ASCII letters, digits and underscore",
        ));
    }
    Ok(())
}
pub fn compile(workflow: &Workflow) -> Result<Compiled> {
    if workflow.schema != "hctl2.workflow.v1" || workflow.profile != "hctl2.passive.v1" {
        return Err(fail(
            "WORKFLOW_PROFILE",
            "unsupported schema or HCTL Profile",
        ));
    }
    let mut warnings = vec![];
    lint(&workflow.graph, &BTreeSet::new(), &mut warnings)?;
    expansion_size(&workflow.graph)?;
    let mut result = Compiled {
        compiler: COMPILER.into(),
        dagu_version: DAGU_VERSION.into(),
        checkpoints: vec![],
        frozen_rework_limits: BTreeMap::new(),
        warnings,
        yaml: String::new(),
    };
    expand(&workflow.graph, "", &[], &[], &mut result)?;
    for checkpoint in &mut result.checkpoints {
        if let Operation::External { seats, .. } = &mut checkpoint.node.operation {
            let mut seen = BTreeSet::new();
            seats.retain(|seat| seen.insert(seat.id.clone()));
        }
    }
    let identities: BTreeSet<_> = result.checkpoints.iter().map(|cp| &cp.id).collect();
    if identities.len() != result.checkpoints.len() {
        return Err(invalid("expanded checkpoint identity collision"));
    }
    let steps: Vec<Value> = result.checkpoints.iter().map(|cp| json!({
        "id":cp.id,"name":cp.id,"depends":cp.depends,"action":"human.task",
        "with":{"prompt":format!("HCTL passive checkpoint {}. Control supplies the admitted receipt; no execution authority.", cp.id),
        "form":{"type":"object","properties":{
            "obligation_id":{"type":"string"},"attempt_generation":{"type":"integer"},
            "verdict":{"type":"string"},"receipt_id":{"type":"string"}},
        "required":["obligation_id","attempt_generation","verdict","receipt_id"]}}
    })).collect();
    // JSON is a YAML subset. Dagu derives entrypoint name from its filename.
    result.yaml = serde_json::to_string_pretty(&json!({"steps":steps}))?;
    Ok(result)
}
fn expansion_size(graph: &Graph) -> Result<u64> {
    let mut total = 0u64;
    for node in &graph.nodes {
        let count = match &node.operation {
            Operation::Loop {
                body, max_reworks, ..
            } => 2 + expansion_size(body)? * (u64::from(*max_reworks) + 1),
            Operation::DynamicFork {
                template,
                max_cardinality,
                ..
            } => {
                2 + expansion_size(&Graph {
                    nodes: vec![*template.clone()],
                    edges: vec![],
                })? * u64::from(*max_cardinality)
            }
            _ => 1,
        };
        total += count;
        if total > MAX_CHECKPOINTS {
            return Err(fail(
                "WORKFLOW_EXPANSION_LIMIT",
                "compiled checkpoint count exceeds compiler resource ceiling 16384",
            ));
        }
    }
    Ok(total)
}
fn lint(graph: &Graph, ancestors: &BTreeSet<String>, warnings: &mut Vec<Warning>) -> Result<()> {
    let mut visible = ancestors.clone();
    visible.extend(graph.nodes.iter().map(|n| n.id.clone()));
    if graph.nodes.is_empty() {
        return Err(invalid("workflow graph must not be empty"));
    }
    let mut nodes = BTreeMap::new();
    for node in &graph.nodes {
        id(&node.id)?;
        if nodes.insert(node.id.as_str(), node).is_some() {
            return Err(invalid("duplicate node id"));
        }
        for output in node.outputs.keys() {
            id(output)?;
        }
        if let Operation::External {
            criterion,
            seats,
            gate,
        } = &node.operation
        {
            match criterion {
                None => {
                    return Err(fail(
                        "WORKFLOW_CRITERION",
                        "external node needs a completion criterion",
                    ));
                }
                Some(Criterion::Mechanical { output, fact }) => {
                    if !node.outputs.contains_key(output) {
                        return Err(fail(
                            "WORKFLOW_CRITERION",
                            "mechanical fact must bind this node's declared output",
                        ));
                    }
                    match fact {
                        MechanicalFact::Ci { check } if check.trim().is_empty() => {
                            return Err(invalid("CI check required"));
                        }
                        MechanicalFact::PathDigest { path, digest }
                            if path.trim().is_empty()
                                || digest.len() != 64
                                || !digest.bytes().all(|b| b.is_ascii_hexdigit()) =>
                        {
                            return Err(invalid("path and SHA-256 required"));
                        }
                        _ => {}
                    }
                }
                Some(Criterion::Gate { .. }) if gate.is_none() => {
                    return Err(fail("WORKFLOW_CRITERION", "Gate criterion requires a Gate"));
                }
                _ => {}
            }
            let mut identities = BTreeMap::new();
            for seat in seats {
                id(&seat.id)?;
                if seat.profession.trim().is_empty() {
                    return Err(invalid("Seat profession required"));
                }
                if let Some(previous) = identities.insert(&seat.id, seat)
                    && previous != seat
                {
                    return Err(invalid("same Seat identity has conflicting declarations"));
                }
                if !visible.contains(&seat.review_subject_node) {
                    return Err(invalid("ReviewSubjectRef source node missing"));
                }
            }
            if let Some(gate) = gate {
                let required: Vec<_> = identities
                    .values()
                    .copied()
                    .filter(|s| s.required)
                    .collect();
                if gate.quorum == 0 || gate.quorum as usize > required.len() {
                    return Err(fail(
                        "WORKFLOW_QUORUM",
                        "quorum exceeds distinct required Seat identities",
                    ));
                }
                if !gate.distinct_worker_profiles {
                    for (i, a) in required.iter().enumerate() {
                        let matches: Vec<_> = required
                            .iter()
                            .skip(i + 1)
                            .filter(|b| {
                                a.profession == b.profession
                                    && a.evidence == b.evidence
                                    && a.required_skills == b.required_skills
                                    && a.review_subject_node == b.review_subject_node
                            })
                            .collect();
                        if !matches.is_empty() {
                            warnings.push(Warning {
                                node: node.id.clone(),
                                seats: std::iter::once(a.id.clone())
                                    .chain(matches.iter().map(|b| b.id.clone()))
                                    .collect(),
                                message: "可能按去重合成一票".into(),
                                missing: vec![
                                    "actual Worker Profile digest".into(),
                                    "actual delivered entries and required Skill digests".into(),
                                ],
                                options: vec![
                                    "声明 Worker Profile 互异".into(),
                                    "改席位要求或必需 Skill".into(),
                                    "降低法定票数".into(),
                                ],
                            });
                        }
                    }
                }
            }
        }
        match &node.operation {
            Operation::Loop {
                body, max_reworks, ..
            } => {
                max_reworks
                    .checked_add(1)
                    .ok_or_else(|| invalid("rework limit exhausted"))?;
                lint(body, &visible, warnings)?;
            }
            Operation::DynamicFork {
                template,
                candidates,
                max_cardinality,
                budget,
                selection,
                permission_ceiling,
            } => {
                let mut choices = BTreeSet::new();
                if candidates.is_empty()
                    || *max_cardinality == 0
                    || *max_cardinality as usize > candidates.len()
                    || *budget == 0
                    || permission_ceiling.is_empty()
                {
                    return Err(fail(
                        "WORKFLOW_FORK_BOUNDS",
                        "freeze candidates, cardinality, budget, selection and permission ceiling",
                    ));
                }
                for candidate in candidates {
                    if candidate.responsibility.trim().is_empty()
                        || !choices.insert(serde_json::to_string(&candidate.selection)?)
                    {
                        return Err(invalid(
                            "distinct frozen candidates with responsibilities required",
                        ));
                    }
                }
                match selection {
                    Selection::All {} if candidates.len() > *max_cardinality as usize => {
                        return Err(invalid("all candidates exceed cardinality"));
                    }
                    Selection::First { count } if *count == 0 || *count > *max_cardinality => {
                        return Err(invalid("selection exceeds cardinality"));
                    }
                    _ => {}
                }
                lint(
                    &Graph {
                        nodes: vec![*template.clone()],
                        edges: vec![],
                    },
                    &visible,
                    warnings,
                )?;
            }
            Operation::Switch { cases }
                if cases.is_empty() || cases.iter().any(|s| s.trim().is_empty()) =>
            {
                return Err(invalid("switch cases required"));
            }
            Operation::TimerWait { seconds: 0 } => return Err(invalid("positive timer required")),
            _ => {}
        }
    }
    let mut incoming: BTreeMap<&str, usize> = nodes.keys().map(|k| (*k, 0)).collect();
    let mut outgoing: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut edges = BTreeSet::new();
    for edge in &graph.edges {
        let from = nodes
            .get(edge.from.as_str())
            .ok_or_else(|| fail("WORKFLOW_DEPENDENCY", "producer node missing"))?;
        let to = nodes
            .get(edge.to.as_str())
            .ok_or_else(|| fail("WORKFLOW_DEPENDENCY", "consumer node missing"))?;
        if !edges.insert((&edge.from, &edge.to)) {
            return Err(invalid("duplicate edge"));
        }
        match (&from.operation, &edge.case) {
            (Operation::Switch { cases }, Some(case)) if cases.contains(case) => {}
            (Operation::Switch { .. }, _) => {
                return Err(invalid("switch edge needs a declared case"));
            }
            (_, Some(_)) => return Err(invalid("case only applies to switch edges")),
            _ => {}
        }
        // Artifact references resolve declared future outputs, never current platform state.
        if let Some(artifact) = &edge.artifact {
            if from.outputs.get(&artifact.output) != Some(&artifact.kind) {
                return Err(fail(
                    "WORKFLOW_DEPENDENCY",
                    "artifact output missing or type mismatch",
                ));
            }
        } else if matches!(from.operation, Operation::External { .. })
            && matches!(to.operation, Operation::External { .. })
        {
            return Err(fail(
                "WORKFLOW_DEPENDENCY",
                "external-to-external edge requires an artifact reference",
            ));
        }
        *incoming.get_mut(edge.to.as_str()).expect("known node") += 1;
        outgoing
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }
    let mut ready: Vec<_> = incoming
        .iter()
        .filter_map(|(k, n)| (*n == 0).then_some(*k))
        .collect();
    let mut visited = 0;
    while let Some(node) = ready.pop() {
        visited += 1;
        for child in outgoing.get(node).into_iter().flatten() {
            let count = incoming.get_mut(child).expect("known node");
            *count -= 1;
            if *count == 0 {
                ready.push(child);
            }
        }
    }
    if visited != nodes.len() {
        return Err(fail("WORKFLOW_CYCLE", "static graph contains a cycle"));
    }
    Ok(())
}
/// Returns exits. Composite nodes get entry/exit markers; inactive branches are
/// discharged by control with typed skip receipts, never Dagu repeat/sub-DAGs.
fn expand(
    graph: &Graph,
    prefix: &str,
    parents: &[String],
    activation: &[Activation],
    out: &mut Compiled,
) -> Result<Vec<String>> {
    let entries: BTreeMap<_, _> = graph
        .nodes
        .iter()
        .map(|n| (n.id.clone(), format!("{prefix}{}", n.id)))
        .collect();
    let exits: BTreeMap<_, _> = graph
        .nodes
        .iter()
        .map(|n| {
            (
                n.id.clone(),
                if matches!(
                    n.operation,
                    Operation::Loop { .. } | Operation::DynamicFork { .. }
                ) {
                    format!("{prefix}{}__exit", n.id)
                } else {
                    format!("{prefix}{}", n.id)
                },
            )
        })
        .collect();
    for node in &graph.nodes {
        let incoming: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.to == node.id)
            .map(|e| exits[&e.from].clone())
            .collect();
        let depends = if incoming.is_empty() {
            parents.to_vec()
        } else {
            incoming
        };
        let checkpoint = entries[&node.id].clone();
        out.checkpoints.push(Checkpoint {
            id: checkpoint.clone(),
            node: node.clone(),
            depends,
            role: CheckpointRole::Node,
            activation: activation.to_vec(),
        });
        let mut tails = vec![checkpoint.clone()];
        match &node.operation {
            Operation::Loop {
                body, max_reworks, ..
            } => {
                out.frozen_rework_limits
                    .insert(checkpoint.clone(), *max_reworks);
                for round in 0..=*max_reworks {
                    let mut guards = activation.to_vec();
                    guards.push(Activation::LoopRound {
                        checkpoint: checkpoint.clone(),
                        round,
                    });
                    tails = expand(
                        body,
                        &format!("{checkpoint}__r{round}__"),
                        &tails,
                        &guards,
                        out,
                    )?;
                }
            }
            Operation::DynamicFork {
                template,
                max_cardinality,
                ..
            } => {
                tails.clear();
                for index in 0..*max_cardinality {
                    let mut guards = activation.to_vec();
                    guards.push(Activation::ForkSlot {
                        checkpoint: checkpoint.clone(),
                        slot: index,
                    });
                    tails.extend(expand(
                        &Graph {
                            nodes: vec![*template.clone()],
                            edges: vec![],
                        },
                        &format!("{checkpoint}__s{index}__"),
                        std::slice::from_ref(&checkpoint),
                        &guards,
                        out,
                    )?);
                }
            }
            _ => continue,
        }
        out.checkpoints.push(Checkpoint {
            id: exits[&node.id].clone(),
            node: node.clone(),
            depends: tails,
            role: CheckpointRole::CompositeExit,
            activation: activation.to_vec(),
        });
    }
    Ok(graph
        .nodes
        .iter()
        .filter(|n| !graph.edges.iter().any(|e| e.from == n.id))
        .map(|n| exits[&n.id].clone())
        .collect())
}

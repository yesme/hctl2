//! Package 7: the human-readable readback projection for `workflow show`,
//! and the domain-level production of readback Bundles.

use crate::{model::Workflow, readback::Readback};
use serde::{Deserialize, Serialize};
use store::{Reference, Result};

/// What `workflow show` renders for the readback of one revision: the
/// node-by-node recap, orphan lists on both sides, and any unhandled
/// "context" items. Whether to approve stays with the human.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadbackSummary {
    /// The readback record this summary was computed from.
    pub readback: Reference,
    /// Whether the Worker Profile differs from every source invocation.
    pub profile_comparison: String,
    /// Source Rooms whose rosters were checked for the producer.
    pub source_rooms: Vec<String>,
    /// Per-node recap: what it delivers, what proves it, what it depends on,
    /// and which checklist item it maps to (or "none").
    pub nodes: Vec<NodeRecap>,
    /// Graph nodes with no matching checklist entry.
    pub orphan_nodes: Vec<String>,
    /// Checklist entries marked as deliverables but with no graph node.
    pub orphan_checklist: Vec<String>,
    /// Items mentioned in the "context" section but with no node.
    pub unhandled_context: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeRecap {
    pub id: String,
    pub operation: String,
    pub delivers: String,
    pub proven_by: String,
    pub depends_on: Vec<String>,
    pub checklist_item: Option<String>,
}

/// Renders a `ReadbackSummary` from a Workflow graph and the readback body.
/// The body is the Skill's structured output; the graph supplies structure.
pub fn summarize(
    workflow: &Workflow,
    readback: &Readback,
    body: &serde_json::Value,
) -> Result<ReadbackSummary> {
    let graph = &workflow.graph;
    let mut nodes = Vec::new();
    for node in &graph.nodes {
        let kind = match &node.operation {
            crate::model::Operation::External { criterion, .. } => criterion
                .as_ref()
                .map(|c| match c {
                    crate::model::Criterion::Mechanical { .. } => "external (mechanical)",
                    crate::model::Criterion::Gate { .. } => "external (gate)",
                })
                .unwrap_or("external"),
            crate::model::Operation::Fork { .. } => "fork",
            crate::model::Operation::Join { .. } => "join",
            crate::model::Operation::Switch { .. } => "switch",
            crate::model::Operation::TimerWait { .. } => "timer_wait",
            crate::model::Operation::Transform { .. } => "transform",
            crate::model::Operation::Noop { .. } => "noop",
            crate::model::Operation::DynamicFork { .. } => "dynamic_fork",
            crate::model::Operation::Loop { .. } => "loop",
        };
        let depends: Vec<String> = graph
            .edges
            .iter()
            .filter(|e| e.to == node.id)
            .map(|e| e.from.clone())
            .collect();
        // The readback body maps node ids to checklist items; absent means none.
        let mapped = body
            .get("node_checklist_map")
            .and_then(|m| m.get(&node.id))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        nodes.push(NodeRecap {
            id: node.id.clone(),
            operation: kind.to_string(),
            delivers: body
                .get("node_delivers")
                .and_then(|m| m.get(&node.id))
                .and_then(|v| v.as_str())
                .unwrap_or("(readback did not say)")
                .to_string(),
            proven_by: body
                .get("node_proven_by")
                .and_then(|m| m.get(&node.id))
                .and_then(|v| v.as_str())
                .unwrap_or("(readback did not say)")
                .to_string(),
            depends_on: depends,
            checklist_item: mapped,
        });
    }
    let orphan_nodes: Vec<String> = body
        .get("orphan_nodes")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let orphan_checklist: Vec<String> = body
        .get("orphan_checklist")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let unhandled_context: Vec<String> = body
        .get("unhandled_context")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    Ok(ReadbackSummary {
        readback: Reference {
            key: store::ObjectKey {
                scope: store::Scope::Control,
                kind: "workflow_readback".into(),
                id: readback.workflow_digest.clone(),
            },
            version: store::Version::State(1),
        },
        profile_comparison: body
            .get("profile_comparison")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        source_rooms: body
            .get("source_rooms")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        nodes,
        orphan_nodes,
        orphan_checklist,
        unhandled_context,
    })
}

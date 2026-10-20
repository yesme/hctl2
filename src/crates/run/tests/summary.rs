//! Package 7 tests: readback verification edge cases, the three-digest
//! separation, and the human-readable summary.

use run::{Workflow, readback, summary};
use store::Reference;

fn workflow_digest() -> String {
    "a".repeat(64)
}
fn snapshot_digest() -> String {
    "b".repeat(64)
}
fn body_digest() -> String {
    "c".repeat(64)
}

/// A minimal workflow with two external nodes and one edge, for summary tests.
fn small_workflow() -> Workflow {
    serde_json::from_value(serde_json::json!({
        "schema": "hctl2.workflow.v1",
        "profile": "restricted",
        "graph": {
            "nodes": [
                {"id": "build", "operation": {"kind": "noop"}},
                {"id": "review", "operation": {"kind": "noop"}}
            ],
            "edges": [{"from": "build", "to": "review"}]
        }
    }))
    .unwrap()
}

#[test]
fn summary_lists_nodes_orphans_and_unhandled_context() {
    let wf = small_workflow();
    let read = readback::Readback {
        workflow_digest: workflow_digest(),
        snapshot_digest: snapshot_digest(),
        invocation: Reference {
            key: store::ObjectKey {
                scope: store::Scope::Control,
                kind: "room_invocation".into(),
                id: "inv-1".into(),
            },
            version: store::Version::State(1),
        },
        selection: Reference {
            key: store::ObjectKey {
                scope: store::Scope::Control,
                kind: "room_selection".into(),
                id: "sel-1".into(),
            },
            version: store::Version::State(1),
        },
        bundle_digest: "d".repeat(64),
        body_digest: body_digest(),
    };
    let body = serde_json::json!({
        "node_delivers": {"build": "compiled binary", "review": "gate verdict"},
        "node_proven_by": {"build": "CI green", "review": "gate receipt"},
        "node_checklist_map": {"build": "CL-1"},
        "orphan_nodes": ["review"],
        "orphan_checklist": ["CL-5"],
        "unhandled_context": ["mentions deployment"],
        "profile_comparison": "different_from_all_producers",
        "source_rooms": ["room-a"]
    });
    let summary = summary::summarize(&wf, &read, &body).unwrap();
    assert_eq!(summary.nodes.len(), 2);
    assert_eq!(summary.nodes[0].id, "build");
    assert_eq!(summary.nodes[0].delivers, "compiled binary");
    assert_eq!(summary.nodes[0].checklist_item.as_deref(), Some("CL-1"));
    assert_eq!(summary.nodes[1].checklist_item, None);
    assert_eq!(summary.orphan_nodes, vec!["review"]);
    assert_eq!(summary.orphan_checklist, vec!["CL-5"]);
    assert_eq!(summary.unhandled_context, vec!["mentions deployment"]);
    assert_eq!(summary.profile_comparison, "different_from_all_producers");
}

#[test]
fn summary_handles_missing_body_fields_gracefully() {
    let wf = small_workflow();
    let read = readback::Readback {
        workflow_digest: workflow_digest(),
        snapshot_digest: snapshot_digest(),
        invocation: Reference {
            key: store::ObjectKey {
                scope: store::Scope::Control,
                kind: "room_invocation".into(),
                id: "inv-1".into(),
            },
            version: store::Version::State(1),
        },
        selection: Reference {
            key: store::ObjectKey {
                scope: store::Scope::Control,
                kind: "room_selection".into(),
                id: "sel-1".into(),
            },
            version: store::Version::State(1),
        },
        bundle_digest: "d".repeat(64),
        body_digest: body_digest(),
    };
    let body = serde_json::json!({});
    let summary = summary::summarize(&wf, &read, &body).unwrap();
    assert_eq!(summary.nodes.len(), 2);
    assert_eq!(summary.nodes[0].delivers, "(readback did not say)");
    assert!(summary.orphan_nodes.is_empty());
    assert!(summary.orphan_checklist.is_empty());
    assert!(summary.unhandled_context.is_empty());
}

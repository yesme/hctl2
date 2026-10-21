//! Task-owned transaction fragment; the caller commits this with the Run records.
use crate::{Task, decode, reject, value_record};
use serde_json::Value;
use store::{Record, Reference, Result, Scope, Version};

pub fn plan_run_occupancy(
    record: &Record,
    expected_run: Option<&str>,
    next: Option<Value>,
) -> Result<Record> {
    let mut task: Task = decode(record)?;
    if record.key.kind != "task_state"
        || record.key.id != task.id
        || record.key.scope != Scope::Project(task.project_id.clone())
    {
        return Err(reject(
            "INVALID_RECORD",
            "Task identity differs",
            "inspect_storage",
        ));
    }
    if task
        .run_occupancy
        .as_ref()
        .and_then(|v| v["run_id"].as_str())
        != expected_run
        || (expected_run.is_some()
            && task
                .run_occupancy
                .as_ref()
                .is_none_or(|v| v["state"] != "active"))
    {
        return Err(reject(
            "RUN_ACTIVE",
            "Task already has an active or completion_pending Run",
            "inspect_run",
        ));
    }
    if let Some(next) = &next {
        let manifest: Reference = serde_json::from_value(next["manifest"].clone())?;
        let run_id = next["run_id"].as_str().filter(|s| !s.is_empty());
        if run_id != Some(manifest.key.id.as_str())
            || manifest.key.scope != record.key.scope
            || manifest.key.kind != "run_manifest"
            || manifest.version != Version::State(1)
            || !matches!(
                next["state"].as_str(),
                Some("active" | "completion_pending")
            )
        {
            return Err(reject(
                "INVALID_RECORD",
                "invalid Run occupancy",
                "inspect_run",
            ));
        }
    }
    task.run_occupancy = next;
    let mut result = value_record(
        record.key.clone(),
        record.version.checked_add(1).ok_or_else(|| {
            reject(
                "INVALID_RECORD",
                "Task version exhausted",
                "inspect_storage",
            )
        })?,
        &task,
    )?;
    result.sources = record.sources.clone();
    result.materials = record.materials.clone();
    Ok(result)
}

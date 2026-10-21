//! Run commands share the public owner/preview/submit boundary.
use run::lifecycle::{self, Input, Plan};
use serde_json::Value;
use std::{path::Path, sync::Arc};
use store::{Result, Store, TrustedActor};
use tokio::sync::Mutex;
type Shared = Arc<Mutex<Option<Store>>>;
fn access<T>(shared: &Shared, f: impl FnOnce(&mut Store) -> Result<T>) -> Result<T> {
    let mut slot = shared.blocking_lock();
    f(slot
        .as_mut()
        .ok_or_else(|| project::reject("STORE_NOT_READY", "store not ready", "check_status"))?)
}
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
fn input(operation: &str, payload: &Value) -> Result<Input> {
    let input: Input = serde_json::from_value(payload.clone())?;
    if operation != format!("run.{}", input.action.kind()) {
        return Err(project::invalid("Run operation and action disagree"));
    }
    Ok(input)
}
fn refresh(
    shared: &Shared,
    services: &crate::services::Supervisor,
    root: &Path,
    input: &Input,
) -> Result<()> {
    let Some(start) = input.action.start() else {
        return Ok(());
    };
    if start.task_revisions.len() > 1 {
        return Err(project::invalid("Run binds at most one Task Revision"));
    }
    let Some(target) = start.task_revisions.first() else {
        return Ok(());
    };
    if target.key.scope != store::Scope::Project(start.project_id.clone())
        || target.key.kind != "task_revision"
    {
        return Err(project::invalid("Task Revision belongs to another Project"));
    }
    let (task_id, _) = target
        .key
        .id
        .rsplit_once(':')
        .ok_or_else(|| project::invalid("Task Revision identity required"))?;
    let (repo, source) = access(shared, |store| {
        let (_, t) = task::task(store, &start.project_id, task_id)?;
        Ok((t.repo_id, t.source_id))
    })?;
    crate::tasks::read(shared, services, root, &repo, &source, None)?;
    Ok(())
}
pub(crate) fn query(
    shared: &Shared,
    actor: &TrustedActor,
    kind: &str,
    payload: &Value,
) -> Result<Value> {
    let project = payload["project_id"]
        .as_str()
        .ok_or_else(|| project::invalid("project_id required"))?;
    access(shared, |store| match kind {
        "run.list" => lifecycle::list(store, actor, project, now_ms()),
        "run.show" => lifecycle::show(
            store,
            actor,
            project,
            payload["run_id"]
                .as_str()
                .ok_or_else(|| project::invalid("run_id required"))?,
            now_ms(),
        ),
        _ => Err(project::invalid("unknown Run query")),
    })
}
pub(crate) fn preview(
    shared: &Shared,
    services: &crate::services::Supervisor,
    root: &Path,
    actor: &TrustedActor,
    operation: &str,
    payload: &Value,
) -> Result<Value> {
    let input = input(operation, payload)?;
    // Lost-response replay needs neither a new platform read nor changed Task state.
    if !access(shared, |store| {
        Ok(lifecycle::replay(store, actor, &input)?.is_some())
    })? {
        refresh(shared, services, root, &input)?;
    }
    access(shared, |store| {
        Ok(serde_json::to_value(lifecycle::prepare(
            store, actor, input,
        )?)?)
    })
}
pub(crate) fn submit(
    shared: &Shared,
    services: &crate::services::Supervisor,
    root: &Path,
    actor: &TrustedActor,
    request: &proto::SubmitRequest,
    details: &Value,
) -> Result<Value> {
    let input = input(
        &request.operation,
        &serde_json::from_slice(&request.payload)?,
    )?;
    if request.command_id != format!("run:{}", input.key) || request.idempotency_key != input.key {
        return Err(project::invalid("Run envelope key mismatch"));
    }
    let plan: Plan = serde_json::from_value(details.clone())?;
    if serde_json::to_value(&plan.input)? != serde_json::to_value(&input)? {
        return Err(project::invalid("Run preview input differs"));
    }
    if !access(shared, |store| {
        Ok(lifecycle::replay(store, actor, &input)?.is_some())
    })? {
        refresh(shared, services, root, &input)?;
    }
    access(shared, |store| {
        lifecycle::admit(store, actor, plan, now_ms())
    })
}
pub(crate) async fn reconcile(shared: Shared) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        let store = Arc::clone(&shared);
        match tokio::task::spawn_blocking(move || {
            let mut slot = store.blocking_lock();
            match slot.as_mut() {
                Some(store) => lifecycle::engine::mark_timeouts(store, now_ms()).map(|_| ()),
                None => Ok(()),
            }
        })
        .await
        {
            Ok(Ok(())) => {}
            Ok(Err(error)) => eprintln!("Run timeout maintenance: {}", error.code),
            Err(error) => eprintln!("Run timeout worker: {error}"),
        }
    }
}

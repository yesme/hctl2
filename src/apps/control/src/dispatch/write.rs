//! Script-produced Git results use the same admission reducer as read-only answers.
use super::*;

pub(super) async fn admit(
    shared: &Shared,
    actor: &TrustedActor,
    project: &str,
    id: &str,
    inbox: &Record,
) -> store::Result<Value> {
    let (output, seal, registration) = {
        let lock = shared.lock().await;
        let s = lock.as_ref().ok_or_else(|| invalid("store not ready"))?;
        let (output, seal, repo) =
            invocation::sealing_input(s, actor, project, id, inbox, now_ms())?;
        (output, seal, repo::require_active(s, &repo)?)
    };
    // Git may take time or fail. Never keep Store locked while it writes objects.
    let (seal, observation) = tokio::task::spawn_blocking(move || {
        let (seal, observation) = crate::changesets::seal(&output.location, seal)?;
        if observation["base_tree_sha"] != observation["result_tree_sha"] {
            crate::changesets::deliver_objects(&registration, &seal, &observation)?;
        }
        Ok::<_, store::StoreError>((seal, observation))
    })
    .await
    .map_err(|_| invalid("Git sealing worker failed"))??;
    let mut lock = shared.lock().await;
    let s = lock.as_mut().ok_or_else(|| invalid("store not ready"))?;
    let saved = participant::value(
        key(inbox.key.scope.clone(), "changeset_seal", &inbox.key.id),
        1,
        &json!({"seal":seal,"observation":observation,"proposal":reference(inbox)}),
    )?;
    // Preserve the original observation on retries; timestamps are not identity.
    if s.get(&saved.key)?.is_none() {
        recovery::record(s, actor, &saved, "invocation.git_sealed", None)?;
    }
    if observation["base_tree_sha"] == observation["result_tree_sha"] {
        let baseline_tree = observation["base_tree_sha"]
            .as_str()
            .ok_or_else(|| invalid("native baseline tree missing"))?;
        invocation::admit_unchanged_result(
            s,
            actor,
            project,
            id,
            inbox,
            (&seal, baseline_tree),
            now_ms(),
        )
    } else {
        invocation::admit_sealed_result(s, actor, project, id, inbox, &seal, now_ms())
    }
}

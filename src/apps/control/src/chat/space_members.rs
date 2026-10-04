//! Durable per-Room continuation of a confirmed main-Room membership write.
//! The existing outbox handles retry; external Space writes have separate intents.
use super::*;

pub(super) fn followups(
    store: &Store,
    member: &store::EffectIntent,
) -> Result<Vec<store::EffectIntent>> {
    let main: Room = serde_json::from_value(member.input["room"].clone())?;
    let mut intents = vec![];
    for record in store.list("room_binding")? {
        if record.key.scope != member.permission_scope {
            continue;
        }
        let carrier: Room = chat::decode(&record)?;
        let Some(target) = carrier.matrix_room_id.clone() else {
            continue;
        };
        let id = format!(
            "chat:space-sync:{}",
            foundation::canonical_json_sha256(&json!([member.intent_id, carrier.id]))?
        );
        let input = json!({"room":main,"carrier_room":carrier,"member_effect":member.intent_id,"users":member.input["users"],"invite":member.input["invite"]});
        intents.push(store::EffectIntent {
            intent_id: id.clone(),
            owner: member.owner.clone(),
            binding: member.binding.clone(),
            operation: "chat.space_sync".into(),
            target,
            // This is a continuation, not the Space resource's exclusive write.
            // A blocked older continuation must not prevent recording a new one.
            conflict_scope: id.clone(),
            permission_scope: member.permission_scope.clone(),
            input_digest: Command::digest_input("chat.space_sync", &input)?,
            input,
            idempotency_key: id,
        });
    }
    Ok(intents)
}

pub(super) fn collect(
    shared: &Shared,
    root: &Path,
    actor: &TrustedActor,
    mut receipt: Value,
    managed_prefix: &str,
    connect: &dyn Fn() -> Result<matrix::Client>,
) -> Result<Value> {
    let Some(ids) = receipt["space_sync_effect_ids"].as_array() else {
        return Ok(receipt);
    };
    let mut targets = vec![];
    let mut partial = false;
    for id in ids {
        let id = id
            .as_str()
            .ok_or_else(|| invalid("Space sync intent ID missing"))?;
        let carrier = access(shared, |s| {
            Ok(s.effect(id)?.0.input["carrier_room"]["id"].clone())
        })?;
        match drive_using(shared, root, actor, id, managed_prefix, connect) {
            Ok(result) => targets.push(json!({"effect_id":id,"carrier_room_id":carrier,"delivery":"confirmed","receipt":result})),
            Err(e) => {
                partial = true;
                targets.push(json!({"effect_id":id,"carrier_room_id":carrier,"delivery":"pending_or_unknown","error":{"code":e.code,"message":e.message,"recovery_action":e.recovery_action}}));
            }
        }
    }
    receipt["spaces"] = json!(targets);
    receipt["space_delivery"] = json!(if partial { "partial" } else { "confirmed" });
    Ok(receipt)
}

pub(super) fn deliver(
    shared: &Shared,
    root: &Path,
    actor: &TrustedActor,
    effect: &store::EffectIntent,
    pending: bool,
    managed_prefix: &str,
    client: &matrix::Client,
) -> Result<Value> {
    // Keep operations ordered for each carrier Room even when an earlier
    // discovery/admission failed before a Space write could be recorded.
    // Otherwise a late old invitation could undo a newer successful removal.
    access(shared, |s| {
        for id in s
            .pending_effects()?
            .into_iter()
            .take_while(|id| id != &effect.intent_id)
        {
            let (older, _) = s.effect(&id)?;
            if older.operation == "chat.space_sync"
                && older.target == effect.target
                && older.permission_scope == effect.permission_scope
                && older.binding == effect.binding
            {
                return Err(reject(
                    "EFFECT_CONFLICT",
                    "an earlier membership sync for this carrier is unresolved",
                    "read_back_original_intent",
                ));
            }
        }
        if pending {
            s.begin_effect(s.generation(), &effect.intent_id)?;
        }
        Ok(())
    })?;
    let carrier: Room = serde_json::from_value(effect.input["carrier_room"].clone())?;
    let Some(space) = client.carrier(&carrier)? else {
        return Ok(json!({"carrier_room_id":carrier.id,"space_id":null}));
    };
    let users: Vec<String> = serde_json::from_value(effect.input["users"].clone())?;
    let invite = effect.input["invite"]
        .as_bool()
        .ok_or_else(|| invalid("membership action missing"))?;
    let member_effect = effect.input["member_effect"]
        .as_str()
        .ok_or_else(|| invalid("primary membership intent missing"))?;
    let ids = enqueue_space_member_intents(
        shared,
        actor,
        &carrier.project_id,
        member_effect,
        std::slice::from_ref(&space),
        &users,
        invite,
    )?;
    let connect = || Ok(client.clone());
    let result = drive_using(shared, root, actor, &ids[0], managed_prefix, &connect)?;
    Ok(
        json!({"carrier_room_id":carrier.id,"space_id":space,"space_effect_id":ids[0],"receipt":result}),
    )
}

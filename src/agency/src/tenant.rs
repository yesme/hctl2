//! Tenant-local journal and dispatch protocol; never depends on control storage.
use crate::{
    runtime::{Runtime, RuntimeEvent, Session},
    storage::{database, nonce, now_ms, private_dir, sql},
};
use agency_proto::*;
use rusqlite::{Connection, OptionalExtension, params};
use rusqlite_migration::{M, Migrations};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
type Sessions = HashMap<String, Arc<Mutex<Box<dyn Session>>>>;

const TENANT_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY,value INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS dispatches(id TEXT PRIMARY KEY,idem TEXT UNIQUE NOT NULL,request BLOB NOT NULL,body BLOB NOT NULL,lease TEXT,lease_expires INTEGER);
CREATE TABLE IF NOT EXISTS used_leases(id TEXT PRIMARY KEY);
CREATE TABLE IF NOT EXISTS events(dispatch TEXT NOT NULL,seq INTEGER NOT NULL,body BLOB NOT NULL,PRIMARY KEY(dispatch,seq));
CREATE TABLE IF NOT EXISTS results(dispatch TEXT NOT NULL,id TEXT PRIMARY KEY,body BLOB NOT NULL,preserved INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS inputs(dispatch TEXT NOT NULL,idem TEXT NOT NULL,digest TEXT NOT NULL,state TEXT NOT NULL,PRIMARY KEY(dispatch,idem));
";

pub(crate) fn tenant_migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(TENANT_SCHEMA)])
}

pub(crate) struct TenantState {
    pub(crate) db: Connection,
    pub(crate) sessions: Sessions,
}
/// The tenant root is `<credential_root>/tenants/<tenant>`; execution directories
/// are placed outside it so a confined child cannot read back the credentials.
fn tenant_credential_root(root: &Path) -> PathBuf {
    root.parent()
        .and_then(|parent| parent.parent())
        .unwrap_or(root)
        .to_path_buf()
}
pub(crate) struct Tenant {
    root: PathBuf,
    exec_parent: PathBuf,
    pub(crate) key: String,
    pub(crate) state: Mutex<TenantState>,
    pub(crate) runtime: Arc<dyn Runtime>,
}
impl Tenant {
    pub(crate) fn open(root: PathBuf, key: String, runtime: Arc<dyn Runtime>) -> Result<Arc<Self>> {
        private_dir(&root)?;
        // Resolved here, while the credential root exists: a reaped child is released
        // after shutdown has already removed it.
        let exec_parent = crate::confine::execution_parent(&tenant_credential_root(&root))?;
        let db = database(&root.join("tenant.sqlite"), &tenant_migrations())?;
        // Not part of migration 1: a writer row that vanished must come back on every
        // open, and an applied migration never runs again.
        sql(db.execute("INSERT OR IGNORE INTO settings VALUES('writer',0)", []))?;
        let tenant = Arc::new(Self {
            root,
            exec_parent,
            key,
            state: Mutex::new(TenantState {
                db,
                sessions: HashMap::new(),
            }),
            runtime,
        });
        // No live identity proof after process restart: retain proposals, never rerun blindly.
        let state = tenant.state.lock().expect("tenant mutex");
        let mut stmt = sql(state.db.prepare("SELECT body FROM dispatches"))?;
        let rows = sql(stmt.query_map([], |r| r.get::<_, Vec<u8>>(0)))?
            .collect::<rusqlite::Result<Vec<_>>>();
        let rows = sql(rows)?;
        drop(stmt);
        for bytes in rows {
            let mut dispatch: Dispatch = serde_json::from_slice(&bytes)?;
            if dispatch.state == DispatchState::Running {
                dispatch.state = DispatchState::CannotFulfill;
                put_dispatch(&state.db, &dispatch)?;
                event(
                    &state.db,
                    &dispatch.reference,
                    "cannot_fulfill",
                    serde_json::json!({"reason":"agency_restarted_without_runtime_proof"}),
                    EvidenceLevel::AdapterEvent,
                )?;
            }
        }
        drop(state);
        Ok(tenant)
    }
    pub(crate) fn prepare(&self, input: Prepare) -> Result<Dispatch> {
        input.spec.verify()?;
        input.spec.document.validate()?;
        input.bundle.verify()?;
        input.bundle.document.validate_delivery()?;
        let spec = &input.spec.document;
        if input.bundle.digest != spec.bundle.digest
            || input.bundle.document.id != spec.bundle.id
            || input.bundle.document.manifest != spec.manifest
            || input.bundle.document.consumer != spec.owner
            || input.bundle.document.permission_digest != spec.permission_digest
        {
            return Err(PortError::new(
                "BUNDLE_MISMATCH",
                "consumer, permissions or frozen Bundle reference differ",
                "rebuild_preview",
            ));
        }
        let catalog = self.runtime.catalog()?;
        for skill in &spec.profession.skills {
            if skill.required
                && !catalog
                    .skills
                    .iter()
                    .any(|available| available.reference == skill.reference)
            {
                return Err(PortError::new(
                    "SKILL_MISSING",
                    &skill.reference.id,
                    "install_required_skill",
                ));
            }
        }
        let actual = catalog
            .professions
            .iter()
            .find(|p| p.reference == spec.profession.reference && **p == spec.profession)
            .ok_or_else(|| {
                PortError::new(
                    "PROFESSION_CHANGED",
                    "accepted Profession is not available with its frozen terms",
                    "accept_new_profession",
                )
            })?;
        actual.capabilities.fulfills(&spec.required_capabilities)?;
        if spec.input_policy == InputPolicy::ManagedSingleWriter
            && !actual.capabilities.managed_single_writer
        {
            return Err(PortError::new(
                "CAPABILITY_MISSING",
                "managed input interception unavailable",
                "select_capable_agency",
            ));
        }
        if spec.deadline_ms <= now_ms() {
            return Err(PortError::new(
                "DEADLINE_EXPIRED",
                "deadline has passed",
                "authorize_new_execution",
            ));
        }
        let mut state = self.state.lock().expect("tenant mutex");
        writer(&state.db, input.writer_generation)?;
        let bytes = canonical(&input)?;
        if let Some((original, body)) = sql(state
            .db
            .query_row(
                "SELECT request,body FROM dispatches WHERE idem=?1",
                [&spec.idempotency_key],
                |r| Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, Vec<u8>>(1)?)),
            )
            .optional())?
        {
            let previous: Prepare = serde_json::from_slice(&original)?;
            if canonical(&previous.spec)? != canonical(&input.spec)?
                || canonical(&previous.bundle)? != canonical(&input.bundle)?
            {
                return Err(PortError::new(
                    "IDEMPOTENCY_CONFLICT",
                    "same dispatch key with changed specification",
                    "use_original_request",
                ));
            }
            return Ok(serde_json::from_slice(&body)?);
        }
        let dispatch = Dispatch {
            reference: nonce()?,
            owner: spec.owner.clone(),
            spec_digest: input.spec.digest.clone(),
            bundle_digest: input.bundle.digest.clone(),
            binding: spec.binding.clone(),
            capabilities: actual.capabilities.clone(),
            state: DispatchState::Prepared,
        };
        let tx = sql(state.db.transaction())?;
        sql(tx.execute(
            "INSERT INTO dispatches(id,idem,request,body) VALUES(?1,?2,?3,?4)",
            params![
                dispatch.reference,
                spec.idempotency_key,
                bytes,
                canonical(&dispatch)?
            ],
        ))?;
        event(
            &tx,
            &dispatch.reference,
            "prepared",
            serde_json::json!({}),
            EvidenceLevel::AdapterEvent,
        )?;
        sql(tx.commit())?;
        Ok(dispatch)
    }
    pub(crate) fn activate(self: &Arc<Self>, input: DispatchAction) -> Result<Dispatch> {
        nonempty(&input.idempotency_key)?;
        let mut state = self.state.lock().expect("tenant mutex");
        writer(&state.db, input.writer_generation)?;
        let mut dispatch = get_dispatch(&state.db, &input.dispatch)?;
        if dispatch.state != DispatchState::Prepared {
            return Ok(dispatch);
        }
        let bytes: Vec<u8> = sql(state.db.query_row(
            "SELECT request FROM dispatches WHERE id=?1",
            [&input.dispatch],
            |r| r.get(0),
        ))?;
        let request: Prepare = serde_json::from_slice(&bytes)?;
        if request.spec.document.deadline_ms <= now_ms() {
            return Err(PortError::new(
                "DEADLINE_EXPIRED",
                "deadline has passed",
                "authorize_new_execution",
            ));
        }
        dispatch.state = DispatchState::Running;
        put_dispatch(&state.db, &dispatch)?;
        let credential_root = tenant_credential_root(&self.root);
        let running = match (|| {
            let exec_root = crate::confine::execution_dir(&credential_root, &dispatch.reference)?;
            self.runtime
                .start(&request.spec, &request.bundle, &exec_root, &credential_root)
        })() {
            Ok(running) => running,
            Err(e) => {
                // The execution directory was created before the start failed.
                crate::confine::release_execution_dir(&self.exec_parent, &dispatch.reference);
                dispatch.state = DispatchState::CannotFulfill;
                put_dispatch(&state.db, &dispatch)?;
                event(
                    &state.db,
                    &dispatch.reference,
                    "cannot_fulfill",
                    serde_json::json!({"code":e.code}),
                    EvidenceLevel::AdapterEvent,
                )?;
                return Ok(dispatch);
            }
        };
        state
            .sessions
            .insert(dispatch.reference.clone(), running.session);
        let timer_tenant = Arc::downgrade(self);
        let timer_id = dispatch.reference.clone();
        let remaining = request.spec.document.deadline_ms.saturating_sub(now_ms());
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(remaining)).await;
            if let Some(tenant) = timer_tenant.upgrade() {
                let _ = tokio::task::spawn_blocking(move || {
                    tenant.record(&timer_id, RuntimeEvent::DeadlineReached)
                })
                .await;
            }
        });
        let tenant = Arc::clone(self);
        let id = dispatch.reference.clone();
        std::thread::spawn(move || {
            for e in running.events {
                let exited = matches!(&e, RuntimeEvent::Exited { .. });
                if let Err(error) = tenant.record(&id, e) {
                    let _ = tenant.record(&id, RuntimeEvent::ProtocolError(error.code));
                    let _ = tenant.stop_private(&id);
                    // Drain through Exited even if persistence failed; do not strand the runtime.
                }
                if exited {
                    tenant
                        .state
                        .lock()
                        .expect("tenant mutex")
                        .sessions
                        .remove(&id);
                    // `Exited` is only sent once the child is reaped, so nothing is
                    // left running in the execution directory. This runs even when
                    // persisting the exit failed.
                    crate::confine::release_execution_dir(&tenant.exec_parent, &id);
                }
            }
        });
        Ok(dispatch)
    }
    fn record(&self, id: &str, observation: RuntimeEvent) -> Result<()> {
        let mut state = self.state.lock().expect("tenant mutex");
        let mut dispatch = get_dispatch(&state.db, id)?;
        match observation {
            RuntimeEvent::DeadlineReached => {
                if dispatch.state == DispatchState::Running {
                    dispatch.state = DispatchState::CannotFulfill;
                    put_dispatch(&state.db, &dispatch)?;
                    event(
                        &state.db,
                        id,
                        "cannot_fulfill",
                        serde_json::json!({"code":"DEADLINE_EXPIRED"}),
                        EvidenceLevel::AdapterEvent,
                    )?;
                    if let Some(session) = state.sessions.get(id) {
                        session.lock().expect("session mutex").stop()?;
                    }
                }
            }
            RuntimeEvent::Observation {
                kind,
                payload,
                source,
            } => {
                event(&state.db, id, &format!("runtime:{kind}"), payload, source)?;
            }
            RuntimeEvent::Proposal {
                schema,
                bytes,
                source,
            } => {
                let tx = sql(state.db.transaction())?;
                let proposal_id = nonce()?;
                let sequence = event(
                    &tx,
                    id,
                    "result_available",
                    serde_json::json!({"content_digest":hash(&bytes)}),
                    EvidenceLevel::AdapterEvent,
                )?;
                let proposal = Proposal {
                    header: ProposalHeader {
                        proposal_id: proposal_id.clone(),
                        owner: dispatch.owner.clone(),
                        dispatch: id.into(),
                        spec_digest: dispatch.spec_digest.clone(),
                        bundle_digest: dispatch.bundle_digest.clone(),
                        binding: dispatch.binding.clone(),
                        producer_sequence: sequence,
                        idempotency_key: proposal_id.clone(),
                    },
                    schema: schema.clone(),
                    content_digest: hash(&bytes),
                    outputs: vec![ProposalOutput {
                        schema,
                        content_digest: hash(&bytes),
                        candidate: FrozenRef {
                            id: proposal_id.clone(),
                            revision: "1".into(),
                            digest: hash(&bytes),
                        },
                        owner: dispatch.owner.clone(),
                        dispatch: id.into(),
                        authorization: FrozenRef {
                            id: dispatch.reference.clone(),
                            revision: "1".into(),
                            digest: dispatch.spec_digest.clone(),
                        },
                    }],
                    output: bytes,
                    evidence: source,
                    preserved: false,
                };
                let bytes = canonical(&proposal)?;
                if bytes.len() > MAX_DOCUMENT - 4096 {
                    return Err(PortError::invalid(
                        "result exceeds transport envelope budget",
                    ));
                }
                sql(tx.execute(
                    "INSERT INTO results(dispatch,id,body) VALUES(?1,?2,?3)",
                    params![id, proposal_id, bytes],
                ))?;
                sql(tx.commit())?;
            }
            RuntimeEvent::Exited {
                code,
                requested_stop,
            } => {
                let has_result = sql(state.db.query_row(
                    "SELECT EXISTS(SELECT 1 FROM results WHERE dispatch=?1)",
                    [id],
                    |r| r.get(0),
                ))?;
                if dispatch.state != DispatchState::CannotFulfill {
                    dispatch.state = crate::runtime::final_state(has_result, code, requested_stop);
                }
                put_dispatch(&state.db, &dispatch)?;
                event(
                    &state.db,
                    id,
                    "stopped",
                    serde_json::json!({"exit_code":code,"requested_stop":requested_stop,"terminal_result_present":has_result}),
                    EvidenceLevel::AdapterEvent,
                )?;
                if !requested_stop && !has_result {
                    event(
                        &state.db,
                        id,
                        "protocol_error",
                        serde_json::json!({"code":"TERMINAL_RESULT_MISSING"}),
                        EvidenceLevel::AdapterEvent,
                    )?;
                }
                state.sessions.remove(id);
            }
            RuntimeEvent::ProtocolError(reason) => {
                dispatch.state = DispatchState::CannotFulfill;
                put_dispatch(&state.db, &dispatch)?;
                event(
                    &state.db,
                    id,
                    "protocol_error",
                    serde_json::json!({"reason":reason,"truncated":true}),
                    EvidenceLevel::AdapterEvent,
                )?;
                if let Some(session) = state.sessions.get(id) {
                    let _ = session.lock().expect("session mutex").stop();
                }
            }
        }
        Ok(())
    }
    pub(crate) fn ticket(
        &self,
        db: &Connection,
        ticket: &Ticket,
        permission: Permission,
    ) -> Result<Dispatch> {
        ticket.verify(self.key.as_bytes())?;
        writer(db, ticket.claims.writer_generation)?;
        let dispatch = get_dispatch(db, &ticket.claims.dispatch)?;
        let claims = &ticket.claims;
        nonempty(&claims.id)?;
        nonempty(&claims.actor)?;
        if claims.owner != dispatch.owner
            || claims.spec_digest != dispatch.spec_digest
            || !claims.permissions.contains(&permission)
            || claims.expires_ms <= now_ms()
        {
            return Err(PortError::new(
                "TICKET_DENIED",
                "wrong dispatch, owner, permission or expired ticket",
                "request_new_ticket",
            ));
        }
        let request: Vec<u8> = sql(db.query_row(
            "SELECT request FROM dispatches WHERE id=?1",
            [&dispatch.reference],
            |r| r.get(0),
        ))?;
        let prepare: Prepare = serde_json::from_slice(&request)?;
        if matches!(
            permission,
            Permission::Input | Permission::Takeover | Permission::SecureInput
        ) && claims.expires_ms > prepare.spec.document.deadline_ms
        {
            return Err(PortError::new(
                "TICKET_DENIED",
                "ticket exceeds frozen deadline",
                "request_scoped_ticket",
            ));
        }
        Ok(dispatch)
    }
    pub(crate) fn lease(&self, input: Lease) -> Result<serde_json::Value> {
        nonempty(&input.new_lease)?;
        let mut state = self.state.lock().expect("tenant mutex");
        let d = self.ticket(&state.db, &input.ticket, Permission::Takeover)?;
        if d.state != DispatchState::Running || !d.capabilities.managed_single_writer {
            return Err(PortError::new(
                "INPUT_UNAVAILABLE",
                "managed input is not active",
                "inspect_dispatch",
            ));
        }
        let current: Option<String> = sql(state.db.query_row(
            "SELECT lease FROM dispatches WHERE id=?1",
            [&d.reference],
            |r| r.get(0),
        ))?;
        if input.ticket.claims.input_lease.as_ref() != Some(&input.new_lease) {
            return Err(PortError::new(
                "TICKET_DENIED",
                "new lease is not in the signed authority",
                "request_scoped_ticket",
            ));
        }
        if current.as_ref() == Some(&input.new_lease) {
            return Ok(serde_json::json!({"input_lease":input.new_lease}));
        }
        if current != input.expected_lease {
            return Err(PortError::new(
                "LEASE_CONFLICT",
                "input lease changed",
                "readback_lease",
            ));
        }
        if sql(state.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM used_leases WHERE id=?1)",
            [&input.new_lease],
            |r| r.get::<_, bool>(0),
        ))? {
            return Err(PortError::new(
                "LEASE_REUSED",
                "revoked lease identity cannot be reused",
                "issue_new_lease",
            ));
        }
        let tx = sql(state.db.transaction())?;
        sql(tx.execute("INSERT INTO used_leases VALUES(?1)", [&input.new_lease]))?;
        sql(tx.execute(
            "UPDATE dispatches SET lease=?2,lease_expires=?3 WHERE id=?1",
            params![
                d.reference,
                input.new_lease,
                integer(input.ticket.claims.expires_ms)?
            ],
        ))?;
        sql(tx.commit())?;
        Ok(serde_json::json!({"input_lease":input.new_lease}))
    }
    pub(crate) fn input(&self, input: Input) -> Result<serde_json::Value> {
        nonempty(&input.idempotency_key)?;
        if input.bytes.len() > 64 * 1024 {
            return Err(PortError::invalid("input limit exceeded"));
        }
        let state = self.state.lock().expect("tenant mutex");
        let d = self.ticket(&state.db, &input.ticket, Permission::Input)?;
        let request: Vec<u8> = sql(state.db.query_row(
            "SELECT request FROM dispatches WHERE id=?1",
            [&d.reference],
            |r| r.get(0),
        ))?;
        let prepare: Prepare = serde_json::from_slice(&request)?;
        if prepare.spec.document.input_policy == InputPolicy::NoInput {
            return Err(PortError::new(
                "INPUT_DENIED",
                "specification disables input",
                "authorize_new_execution",
            ));
        }
        if prepare.spec.document.input_policy == InputPolicy::ManagedSingleWriter {
            let lease: Option<String> = sql(state.db.query_row(
                "SELECT lease FROM dispatches WHERE id=?1",
                [&d.reference],
                |r| r.get(0),
            ))?;
            if lease.is_none() || lease != input.ticket.claims.input_lease {
                return Err(PortError::new(
                    "INPUT_LEASE_INVALID",
                    "input lease revoked or absent",
                    "request_new_input_lease",
                ));
            }
            let expires = sql(state.db.query_row(
                "SELECT lease_expires FROM dispatches WHERE id=?1",
                [&d.reference],
                number,
            ))?;
            if expires <= now_ms() {
                return Err(PortError::new(
                    "INPUT_LEASE_INVALID",
                    "input lease expired",
                    "request_new_input_lease",
                ));
            }
        }
        let body_digest = hash(&canonical(&input)?);
        let found: Option<(String, String)> = sql(state
            .db
            .query_row(
                "SELECT digest,state FROM inputs WHERE dispatch=?1 AND idem=?2",
                params![d.reference, input.idempotency_key],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional())?;
        if let Some((digest, state)) = found {
            if digest != body_digest {
                return Err(PortError::new(
                    "IDEMPOTENCY_CONFLICT",
                    "input changed",
                    "use_original_request",
                ));
            }
            return Ok(serde_json::json!({"delivery":state}));
        }
        if d.state != DispatchState::Running {
            return Err(PortError::new(
                "INPUT_UNAVAILABLE",
                "execution is not running",
                "inspect_dispatch",
            ));
        }
        let session = state.sessions.get(&d.reference).ok_or_else(|| {
            PortError::new(
                "CANNOT_FULFILL",
                "runtime is unavailable",
                "inspect_dispatch",
            )
        })?;
        // Mark uncertainty before external input. Never retry an unknown physical write.
        sql(state.db.execute(
            "INSERT INTO inputs VALUES(?1,?2,?3,'unknown')",
            params![d.reference, input.idempotency_key, body_digest],
        ))?;
        event(
            &state.db,
            &d.reference,
            "input_unknown",
            serde_json::json!({"idempotency_key":input.idempotency_key,"digest":hash(&input.bytes),"ticket":input.ticket.claims.id,"actor":input.ticket.claims.actor}),
            EvidenceLevel::AdapterEvent,
        )?;
        session
            .lock()
            .expect("session mutex")
            .input(&input.bytes)
            .map_err(|_| {
                PortError::new(
                    "INPUT_RESPONSE_UNKNOWN",
                    "input may have been partly delivered; never resend under a new key",
                    "inspect_dispatch",
                )
            })?;
        sql(state.db.execute(
            "UPDATE inputs SET state='delivered' WHERE dispatch=?1 AND idem=?2",
            params![d.reference, input.idempotency_key],
        ))?;
        event(
            &state.db,
            &d.reference,
            "input_delivered",
            serde_json::json!({"digest":hash(&input.bytes),"ticket":input.ticket.claims.id,"actor":input.ticket.claims.actor}),
            EvidenceLevel::AdapterEvent,
        )?;
        Ok(serde_json::json!({"delivery":"delivered"}))
    }
    pub(crate) fn stop(&self, ticket: Ticket) -> Result<Dispatch> {
        let state = self.state.lock().expect("tenant mutex");
        let d = self.ticket(&state.db, &ticket, Permission::Stop)?;
        if let Some(session) = state.sessions.get(&d.reference) {
            session.lock().expect("session mutex").stop()?;
        } else if d.state == DispatchState::Prepared {
            let mut cancelled = d.clone();
            cancelled.state = DispatchState::Cancelled;
            put_dispatch(&state.db, &cancelled)?;
            return Ok(cancelled);
        }
        // Request is not proof of exit. Observe the stopped event and actual exit code.
        Ok(d)
    }
    fn stop_private(&self, id: &str) -> Result<()> {
        let state = self.state.lock().expect("tenant mutex");
        if let Some(session) = state.sessions.get(id) {
            session.lock().expect("session mutex").stop()?;
        }
        Ok(())
    }
    pub(crate) fn observe(&self, input: Observe) -> Result<Trace> {
        let state = self.state.lock().expect("tenant mutex");
        let dispatch = self.ticket(&state.db, &input.ticket, Permission::Observe)?;
        let latest: u64 = sql(state.db.query_row(
            "SELECT COALESCE(MAX(seq),0) FROM events WHERE dispatch=?1",
            [&dispatch.reference],
            number,
        ))?;
        let mut stmt = sql(state.db.prepare(
            "SELECT body FROM events WHERE dispatch=?1 AND seq>?2 ORDER BY seq LIMIT 128",
        ))?;
        let rows = sql(
            stmt.query_map(params![dispatch.reference, integer(input.after)?], |r| {
                r.get::<_, Vec<u8>>(0)
            }),
        )?;
        let mut events = Vec::new();
        // Leave room for the dispatch and transport envelope. A count-only page can
        // exceed gRPC's byte limit and make every replay of that cursor fail.
        let mut remaining = MAX_DOCUMENT
            .saturating_sub(canonical(&dispatch)?.len())
            .saturating_sub(1024);
        for bytes in rows {
            let bytes = sql(bytes)?;
            if bytes.len() + 1 > remaining {
                break;
            }
            remaining -= bytes.len() + 1;
            events.push(serde_json::from_slice::<Observation>(&bytes)?);
        }
        let cursor = events
            .last()
            .map_or(input.after.min(latest), |e| e.sequence);
        let truncated:bool = sql(state.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM events WHERE dispatch=?1 AND json_extract(CAST(body AS TEXT),'$.payload.truncated')=1)",
            [&dispatch.reference], |r|r.get(0)))?;
        let gap = input.after > latest || truncated;
        let complete = !gap && cursor == latest;
        Ok(Trace {
            dispatch,
            events,
            cursor,
            gap,
            complete,
        })
    }
    pub(crate) fn results(&self, input: ResultQuery) -> Result<ResultPage> {
        let state = self.state.lock().expect("tenant mutex");
        let dispatch = get_dispatch(&state.db, &input.dispatch)?;
        result_page(&state.db, dispatch.state == DispatchState::Running, &input)
    }
    pub(crate) fn lookup(&self, input: Lookup) -> Result<Dispatch> {
        let state = self.state.lock().expect("tenant mutex");
        let id: Option<String> = sql(state
            .db
            .query_row(
                "SELECT id FROM dispatches WHERE idem=?1",
                [input.idempotency_key],
                |r| r.get(0),
            )
            .optional())?;
        get_dispatch(
            &state.db,
            &id.ok_or_else(|| {
                PortError::new(
                    "DISPATCH_NOT_FOUND",
                    "no acceptance for this key",
                    "reconcile_original_intent",
                )
            })?,
        )
    }
    pub(crate) fn preserve(&self, input: Preservation) -> Result<serde_json::Value> {
        let state = self.state.lock().expect("tenant mutex");
        get_dispatch(&state.db, &input.dispatch)?;
        let bytes: Vec<u8> = sql(state.db.query_row(
            "SELECT body FROM results WHERE dispatch=?1 AND id=?2",
            params![input.dispatch, input.proposal_id],
            |r| r.get(0),
        ))?;
        let p: Proposal = serde_json::from_slice(&bytes)?;
        if p.content_digest != input.content_digest || hash(&p.output) != input.content_digest {
            return Err(PortError::new(
                "PRESERVATION_MISMATCH",
                "acknowledgement does not name the exact bytes",
                "preserve_exact_result",
            ));
        }
        sql(state.db.execute(
            "UPDATE results SET preserved=1 WHERE dispatch=?1 AND id=?2",
            params![input.dispatch, input.proposal_id],
        ))?;
        Ok(serde_json::json!({"preserved":true}))
    }
    pub(crate) fn fence(&self, input: Fence) -> Result<serde_json::Value> {
        let state = self.state.lock().expect("tenant mutex");
        let old: u64 =
            sql(state
                .db
                .query_row("SELECT value FROM settings WHERE key='writer'", [], number))?;
        if input.writer_generation == 0 || input.writer_generation < old {
            return Err(PortError::new(
                "WRITER_STALE",
                "writer generation cannot move backwards",
                "reconcile_control_writer",
            ));
        }
        sql(state.db.execute(
            "UPDATE settings SET value=?1 WHERE key='writer'",
            [integer(input.writer_generation)?],
        ))?;
        Ok(serde_json::json!({"writer_generation":input.writer_generation}))
    }
}
fn writer(db: &Connection, generation: u64) -> Result<()> {
    let actual: u64 =
        sql(db.query_row("SELECT value FROM settings WHERE key='writer'", [], number))?;
    if generation == 0 || actual != generation {
        Err(PortError::new(
            "WRITER_STALE",
            "outbox or ticket has an old writer",
            "reconcile_control_writer",
        ))
    } else {
        Ok(())
    }
}
/// One page of a dispatch's results, selected by SQLite rather than read whole.
/// The cursor names a stored proposal id, which is the row the page starts after;
/// a cursor from another dispatch names no row here and is refused the same way
/// an unknown one is.
///
/// `complete` means the page reached the end of the stored results and `running`
/// is false. A caller that stops at `complete` while the dispatch is still running
/// would miss a result that has not been written yet.
fn result_page(db: &Connection, running: bool, input: &ResultQuery) -> Result<ResultPage> {
    let limit = input.limit.unwrap_or(16).clamp(1, 32);
    let after = match &input.after {
        None => None,
        Some(cursor) => {
            let rowid = sql(db
                .query_row(
                    "SELECT rowid FROM results WHERE dispatch=?1 AND id=?2",
                    params![input.dispatch, cursor],
                    |r| r.get::<_, i64>(0),
                )
                .optional())?;
            Some(rowid.ok_or_else(|| {
                PortError::new(
                    "RESULT_CURSOR_UNKNOWN",
                    "result cursor does not name a stored proposal",
                    "read_result_page",
                )
            })?)
        }
    };
    let mut stmt = sql(db.prepare(
        "SELECT body,preserved FROM results
         WHERE dispatch=?1 AND (?2 IS NULL OR rowid>?2)
         ORDER BY rowid LIMIT ?3",
    ))?;
    // One row past the page: whether SQLite found it is what `complete` reports.
    let rows = sql(
        stmt.query_map(params![input.dispatch, after, i64::from(limit) + 1], |r| {
            Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, bool>(1)?))
        }),
    )?
    .collect::<rusqlite::Result<Vec<_>>>();
    let rows = sql(rows)?;
    drop(stmt);
    let mut proposals = Vec::new();
    let mut complete = rows.len() <= limit as usize;
    for (bytes, preserved) in rows.into_iter().take(limit as usize) {
        let mut proposal: Proposal = serde_json::from_slice(&bytes)?;
        proposal.preserved = preserved;
        let mut trial = proposals.clone();
        trial.push(proposal.clone());
        let page = ResultPage {
            proposals: trial,
            cursor: Some(proposal.header.proposal_id.clone()),
            complete: false,
        };
        match canonical(&page) {
            Ok(bytes) if bytes.len() + 1024 <= MAX_DOCUMENT => proposals.push(proposal),
            _ if !proposals.is_empty() => {
                complete = false;
                break;
            }
            Ok(_) => proposals.push(proposal),
            Err(error) => return Err(error),
        }
    }
    if running {
        complete = false;
    }
    let cursor = proposals.last().map(|p| p.header.proposal_id.clone());
    Ok(ResultPage {
        proposals,
        cursor,
        complete,
    })
}
fn get_dispatch(db: &Connection, id: &str) -> Result<Dispatch> {
    let bytes: Option<Vec<u8>> = sql(db
        .query_row("SELECT body FROM dispatches WHERE id=?1", [id], |r| {
            r.get(0)
        })
        .optional())?;
    serde_json::from_slice(&bytes.ok_or_else(|| {
        PortError::new(
            "DISPATCH_NOT_FOUND",
            "dispatch not found in this tenant",
            "check_dispatch_reference",
        )
    })?)
    .map_err(PortError::from)
}
fn put_dispatch(db: &Connection, d: &Dispatch) -> Result<()> {
    sql(db.execute(
        "UPDATE dispatches SET body=?2 WHERE id=?1",
        params![d.reference, canonical(d)?],
    ))?;
    Ok(())
}
fn event(
    db: &Connection,
    id: &str,
    kind: &str,
    payload: serde_json::Value,
    source: EvidenceLevel,
) -> Result<u64> {
    let seq: u64 = sql(db.query_row(
        "SELECT COALESCE(MAX(seq),0)+1 FROM events WHERE dispatch=?1",
        [id],
        number,
    ))?;
    let e = Observation {
        sequence: seq,
        source,
        confidence: "reported".into(),
        evidence_digest: hash(&canonical(&payload)?),
        observed_ms: now_ms(),
        kind: kind.into(),
        payload,
    };
    let bytes = canonical(&e)?;
    if bytes.len() > MAX_DOCUMENT / 2 {
        return Err(PortError::invalid("observation exceeds per-event budget"));
    }
    sql(db.execute(
        "INSERT INTO events VALUES(?1,?2,?3)",
        params![id, integer(seq)?, bytes],
    ))?;
    Ok(seq)
}
fn integer(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| PortError::invalid("integer exceeds storage range"))
}
fn number(row: &rusqlite::Row<'_>) -> rusqlite::Result<u64> {
    let n = row.get::<_, i64>(0)?;
    u64::try_from(n).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, n))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner() -> Owner {
        Owner {
            project: "proj".into(),
            kind: OwnerKind::RoomInvocation,
            id: "room".into(),
            generation: 1,
        }
    }

    fn proposal(dispatch: &str, i: usize) -> Proposal {
        let output = vec![b'x'; 64];
        let digest = hash(&output);
        let id = format!("p{i:04}");
        Proposal {
            header: ProposalHeader {
                proposal_id: id.clone(),
                owner: owner(),
                dispatch: dispatch.into(),
                spec_digest: digest.clone(),
                bundle_digest: digest.clone(),
                binding: FrozenRef {
                    id: "binding".into(),
                    revision: "1".into(),
                    digest: digest.clone(),
                },
                producer_sequence: i as u64,
                idempotency_key: id.clone(),
            },
            schema: "test.bytes.v1".into(),
            content_digest: digest.clone(),
            outputs: vec![],
            output,
            evidence: EvidenceLevel::Narrated,
            preserved: false,
        }
    }

    /// `poison_from` stores a `body` that is not a blob, so a page that reads past
    /// its own limit fails on the conversion instead of returning.
    fn seeded(rows: usize, poison_from: Option<usize>) -> Connection {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch(TENANT_SCHEMA).unwrap();
        for i in 0..rows {
            let proposal = proposal("d1", i);
            let id = proposal.header.proposal_id.clone();
            let insert = "INSERT INTO results(dispatch,id,body,preserved) VALUES('d1',?1,?2,0)";
            if poison_from.is_some_and(|from| i >= from) {
                db.execute(insert, params![id, i as i64]).unwrap();
            } else {
                db.execute(insert, params![id, canonical(&proposal).unwrap()])
                    .unwrap();
            }
        }
        db
    }

    fn page(after: Option<&str>, limit: Option<u32>) -> ResultQuery {
        let mut query = ResultQuery::of("d1");
        query.after = after.map(str::to_owned);
        query.limit = limit;
        query
    }

    #[test]
    fn a_page_reads_at_most_one_row_past_its_limit() {
        for limit in [1u32, 2, 16, 32] {
            let db = seeded(200, Some(limit as usize + 1));
            // The first poisoned row is genuinely unreadable as a blob.
            let poisoned = i64::from(limit) + 2;
            assert!(
                db.query_row("SELECT body FROM results WHERE rowid=?1", [poisoned], |r| r
                    .get::<_, Vec<u8>>(0))
                    .is_err()
            );
            let page = result_page(&db, false, &page(None, Some(limit))).unwrap();
            assert_eq!(page.proposals.len(), limit as usize);
            assert!(!page.complete);
        }
    }

    #[test]
    fn a_cursor_must_name_a_proposal_this_dispatch_stored() {
        let db = seeded(4, None);
        db.execute(
            "INSERT INTO results(dispatch,id,body,preserved) VALUES('d2','q0000',?1,0)",
            params![canonical(&proposal("d2", 0)).unwrap()],
        )
        .unwrap();
        for cursor in ["no-such-proposal", "q0000"] {
            let error = result_page(&db, false, &page(Some(cursor), None)).unwrap_err();
            assert_eq!(error.code, "RESULT_CURSOR_UNKNOWN");
            assert_eq!(
                error.message,
                "result cursor does not name a stored proposal"
            );
            assert_eq!(error.recovery_action, "read_result_page");
        }
        let page = result_page(&db, false, &page(Some("p0001"), None)).unwrap();
        assert_eq!(
            page.proposals
                .iter()
                .map(|p| p.header.proposal_id.clone())
                .collect::<Vec<_>>(),
            ["p0002", "p0003"]
        );
        assert!(page.complete);
        assert_eq!(page.cursor.as_deref(), Some("p0003"));
    }

    #[test]
    fn complete_tracks_the_end_of_the_stored_results_and_a_running_dispatch() {
        let empty = seeded(0, None);
        let query = page(None, None);
        assert!(
            result_page(&empty, false, &query)
                .unwrap()
                .proposals
                .is_empty()
        );
        assert!(result_page(&empty, false, &query).unwrap().complete);
        assert!(!result_page(&empty, true, &query).unwrap().complete);
        let full = seeded(2, None);
        assert!(result_page(&full, false, &query).unwrap().complete);
        assert!(!result_page(&full, true, &query).unwrap().complete);
        let partial = seeded(20, None);
        assert!(
            !result_page(&partial, false, &page(None, Some(16)))
                .unwrap()
                .complete
        );
    }

    #[test]
    fn the_stored_preservation_flag_reaches_the_page() {
        let db = seeded(2, None);
        db.execute("UPDATE results SET preserved=1 WHERE id='p0001'", [])
            .unwrap();
        let page = result_page(&db, false, &page(None, None)).unwrap();
        assert!(!page.proposals[0].preserved);
        assert!(page.proposals[1].preserved);
    }
}

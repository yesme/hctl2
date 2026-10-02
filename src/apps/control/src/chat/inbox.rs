//! Disposable provider observations. No Room message becomes a governance command here.
use axum::{
    Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{self, HeaderMap, StatusCode},
    routing::put,
};
use chat::{Result, reject};
use ruma::{api::IncomingRequest, api::appservice::event::push_events};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone)]
pub(super) struct Inbox {
    pub root: PathBuf,
    pub token: String,
}

impl Inbox {
    fn open(&self) -> Result<Connection> {
        let path = self.root.join("cache/chat-inbox.sqlite");
        std::fs::create_dir_all(path.parent().unwrap())?;
        let conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;
            CREATE TABLE IF NOT EXISTS transactions(id TEXT PRIMARY KEY, digest TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS observations(event_id TEXT PRIMARY KEY, room_id TEXT NOT NULL, event TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS health(room_id TEXT PRIMARY KEY, error TEXT);")?;
        Ok(conn)
    }

    pub fn accept(&self, id: &str, body: &[u8], events: &[Value]) -> Result<()> {
        let mut conn = self.open()?;
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let digest =
            foundation::bytes_sha256(&foundation::canonical_json(&serde_json::from_slice(body)?)?);
        let previous: Option<String> = tx
            .query_row("SELECT digest FROM transactions WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .optional()?;
        if let Some(previous) = previous {
            if previous != digest {
                return Err(reject(
                    "INBOX_CONFLICT",
                    "transaction body changed",
                    "inspect_chat_server",
                ));
            }
            return Ok(());
        }
        for event in events {
            let Some(event_id) = event["event_id"].as_str() else {
                continue;
            };
            let Some(room_id) = event["room_id"].as_str() else {
                continue;
            };
            tx.execute(
                "INSERT OR IGNORE INTO observations VALUES(?1,?2,?3)",
                params![event_id, room_id, event.to_string()],
            )?;
        }
        tx.execute(
            "INSERT INTO transactions VALUES(?1,?2)",
            params![id, digest],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn health(&self, room: &str, error: Option<&str>) -> Result<()> {
        self.open()?.execute("INSERT INTO health VALUES(?1,?2) ON CONFLICT(room_id) DO UPDATE SET error=excluded.error", params![room,error])?;
        Ok(())
    }
    pub fn state(&self, room: &str) -> Result<Value> {
        let error: Option<Option<String>> = self
            .open()?
            .query_row("SELECT error FROM health WHERE room_id=?1", [room], |r| {
                r.get(0)
            })
            .optional()?;
        Ok(match error {
            Some(None) => json!({"needs_attention":false,"last_read":"available"}),
            Some(Some(error)) => {
                json!({"needs_attention":true,"error":error,"last_read":"unavailable"})
            }
            None => json!({"needs_attention":true,"last_read":"not_observed"}),
        })
    }
    pub fn router(self) -> Router {
        Router::new()
            .route("/_matrix/app/v1/transactions/{txn_id}", put(transaction))
            .layer(DefaultBodyLimit::max(8 * 1024 * 1024))
            .with_state(Arc::new(self))
    }
}

async fn transaction(
    State(inbox): State<Arc<Inbox>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> (StatusCode, axum::Json<Value>) {
    if headers
        .get(http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        != Some(&format!("Bearer {}", inbox.token))
    {
        return (StatusCode::UNAUTHORIZED, axum::Json(json!({})));
    }
    let request = match http::Request::builder()
        .method("PUT")
        .uri("/_matrix/app/v1/transactions/ignored")
        .body(body.clone())
    {
        Ok(request) => request,
        Err(_) => return (StatusCode::BAD_REQUEST, axum::Json(json!({}))),
    };
    let request = match push_events::v1::Request::try_from_http_request(request, &[&id]) {
        Ok(request) => request,
        Err(_) => return (StatusCode::BAD_REQUEST, axum::Json(json!({}))),
    };
    let events = match request
        .events
        .iter()
        .map(|e| serde_json::from_str::<Value>(e.json().get()))
        .collect::<std::result::Result<Vec<_>, _>>()
    {
        Ok(events) => events,
        Err(_) => return (StatusCode::BAD_REQUEST, axum::Json(json!({}))),
    };
    let status = match tokio::task::spawn_blocking(move || inbox.accept(&id, &body, &events)).await
    {
        Ok(Ok(())) => StatusCode::OK,
        Ok(Err(e)) if e.code == "INBOX_CONFLICT" => StatusCode::CONFLICT,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status, axum::Json(json!({})))
}

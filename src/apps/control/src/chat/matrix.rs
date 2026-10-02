//! Ruma owns Matrix serialization; reqwest only carries bytes on the local endpoint.
use axum::http;
use chat::{Result, Server, Source, SourceText, invalid, reject};
use foundation::{bytes_sha256, canonical_json};
use ruma::api::client::{
    account::register,
    alias::get_alias,
    membership::join_room_by_id,
    message::{get_message_events, send_message_event},
    room::{create_room, get_room_event},
    state::get_state_event_for_key,
};
use ruma::{
    OwnedRoomId, OwnedUserId,
    api::{
        AppserviceUserIdentity, IncomingResponse, OutgoingRequest, OutgoingRequestAppserviceExt,
        SupportedVersions,
        auth_scheme::{AuthScheme, SendAccessToken},
        path_builder::VersionHistory,
    },
    events::{StateEventType, room::message::RoomMessageEventContent},
};
use serde_json::{Value, json};
use std::{borrow::Cow, io::Read, time::Duration};
use store::Reference;

const MAX_RESPONSE: u64 = 8 * 1024 * 1024;

pub struct Client {
    pub server: Server,
    token: String,
    client: reqwest::blocking::Client,
}

impl Client {
    pub fn new(server: Server, token: String) -> Result<Self> {
        let url =
            reqwest::Url::parse(&server.url).map_err(|_| invalid("invalid homeserver URL"))?;
        if url.scheme() != "http"
            || url.host_str() != Some("127.0.0.1")
            || !url.username().is_empty()
            || url.password().is_some()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid(
                "this port supports only the packaged loopback homeserver",
            ));
        }
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|_| unavailable())?;
        Ok(Self {
            server,
            token,
            client,
        })
    }

    pub(super) fn request<R>(&self, request: R) -> Result<R::IncomingResponse>
    where
        R: OutgoingRequest<PathBuilder = VersionHistory>,
        for<'a> R::Authentication: AuthScheme<Input<'a> = SendAccessToken<'a>>,
    {
        let user: OwnedUserId = self
            .server
            .sender
            .parse()
            .map_err(|_| invalid("invalid AppService sender"))?;
        let request: http::Request<Vec<u8>> = request
            .try_into_http_request_with_identity(
                &self.server.url,
                SendAccessToken::Appservice(&self.token),
                AppserviceUserIdentity::new(&user),
                Cow::Owned(SupportedVersions::from_parts(
                    &["v1.11".into()],
                    &Default::default(),
                )),
            )
            .map_err(|_| invalid("cannot encode Matrix request"))?;
        let (parts, body) = request.into_parts();
        let response = self
            .client
            .request(parts.method, parts.uri.to_string())
            .headers(parts.headers)
            .body(body)
            .send()
            .map_err(|_| unavailable())?;
        let status = response.status();
        let mut bytes = Vec::new();
        response
            .take(MAX_RESPONSE + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| unavailable())?;
        if bytes.len() as u64 > MAX_RESPONSE {
            return Err(reject(
                "CHAT_RESPONSE_LIMIT",
                "Matrix response exceeds limit",
                "narrow_query",
            ));
        }
        if !status.is_success() {
            let code = serde_json::from_slice::<Value>(&bytes)
                .ok()
                .and_then(|v| v["errcode"].as_str().map(str::to_owned));
            return Err(match (status.as_u16(), code.as_deref()) {
                (404, Some("M_NOT_FOUND")) => {
                    reject("CHAT_NOT_FOUND", "Matrix object not found", "inspect_room")
                }
                (400, Some("M_USER_IN_USE")) => {
                    reject("CHAT_USER_EXISTS", "virtual user exists", "read_back_user")
                }
                (400, Some("M_ROOM_IN_USE")) => reject(
                    "CHAT_ALIAS_EXISTS",
                    "room alias exists",
                    "read_back_original_intent",
                ),
                (429, _) => reject(
                    "CHAT_RATE_LIMITED",
                    "Matrix rate limit; retry with original key",
                    "retry_original_command_later",
                ),
                (401 | 403, _) => reject(
                    "CHAT_PERMISSION_DENIED",
                    "Matrix denied current read or action",
                    "check_chat_binding",
                ),
                _ => unavailable(),
            });
        }
        let response = http::Response::builder()
            .status(status)
            .body(bytes)
            .map_err(|_| unavailable())?;
        R::IncomingResponse::try_from_http_response(response).map_err(|_| {
            reject(
                "CHAT_PROTOCOL",
                "invalid Matrix response",
                "inspect_chat_server",
            )
        })
    }

    pub fn register_virtual_user(&self, localpart: &str) -> Result<String> {
        if !localpart.starts_with("hctl2_")
            || !localpart
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err(invalid("virtual user outside the registered namespace"));
        }
        let mut request = register::v3::Request::new();
        request.username = Some(localpart.into());
        request.login_type = Some(register::LoginType::ApplicationService);
        request.inhibit_login = true;
        match self.request(request) {
            Ok(response) => Ok(response.user_id.to_string()),
            Err(e) if e.code == "CHAT_USER_EXISTS" => {
                // Identity assertion on whoami proves that this token owns the existing identity.
                let mut server = self.server.clone();
                server.sender = format!("@{localpart}:{}", server.server_name);
                let client = Self::new(server, self.token.clone())?;
                let response =
                    client.request(ruma::api::client::account::whoami::v3::Request::new())?;
                if response.user_id.as_str() != client.server.sender {
                    return Err(invalid("virtual user identity mismatch"));
                }
                Ok(response.user_id.to_string())
            }
            Err(e) => Err(e),
        }
    }

    pub(super) fn state(&self, room: &str, event_type: &str) -> Result<Value> {
        let r = self.request(get_state_event_for_key::v3::Request::new(
            room_id(room)?,
            StateEventType::from(event_type),
            String::new(),
        ))?;
        Ok(serde_json::from_str(r.event_or_content.get())?)
    }

    pub fn guard(&self, room: &str) -> Result<()> {
        // A successful room-state read distinguishes absent encryption from inaccessible room.
        self.state(room, "m.room.create")?;
        match self.state(room, "m.room.encryption") {
            Err(e) if e.code == "CHAT_NOT_FOUND" => Ok(()),
            Ok(_) => Err(reject(
                "CHAT_ENCRYPTED",
                "Room now has end-to-end encryption",
                "rebind_unencrypted_room",
            )),
            Err(e) => Err(e),
        }
    }

    pub fn create(&self, room: &chat::Room, command: &str, dispatch: bool) -> Result<Value> {
        self.create_with_dispatch(room, command, dispatch, || Ok(()))
    }

    pub(super) fn create_with_dispatch(
        &self,
        room: &chat::Room,
        command: &str,
        dispatch: bool,
        before_dispatch: impl FnOnce() -> Result<()>,
    ) -> Result<Value> {
        self.create_native(room, command, false, dispatch, before_dispatch)
    }

    pub(super) fn create_native(
        &self,
        room: &chat::Room,
        command: &str,
        space: bool,
        dispatch: bool,
        before_dispatch: impl FnOnce() -> Result<()>,
    ) -> Result<Value> {
        // Alias lookup is public on some homeservers. It alone cannot prove that the
        // AppService registration has loaded, so check authenticated identity first.
        let identity = self.request(ruma::api::client::account::whoami::v3::Request::new())?;
        if identity.user_id.as_str() != self.server.sender {
            return Err(invalid("AppService identity mismatch"));
        }
        let correlation = bytes_sha256(command.as_bytes());
        let alias = format!("#hctl2_{correlation}:{}", self.server.server_name);
        match self.request(get_alias::v3::Request::new(
            alias.parse().map_err(|_| invalid("invalid room alias"))?,
        )) {
            Ok(r) => return self.creation_readback(room, r.room_id.as_str(), &correlation, space),
            Err(e) if e.code == "CHAT_NOT_FOUND" => (),
            Err(e) => return Err(e),
        }
        if !dispatch {
            // Creation joins the sender. An unavailable or unreadable inventory is not absence.
            let joined =
                self.request(ruma::api::client::membership::joined_rooms::v3::Request::new())?;
            for id in joined.joined_rooms {
                match self.state(id.as_str(), "io.hctl2.creation") {
                    Ok(marker) if marker["command"] == correlation => {
                        return self.creation_readback(room, id.as_str(), &correlation, space);
                    }
                    Ok(_) => (),
                    Err(e) if e.code == "CHAT_NOT_FOUND" => (),
                    Err(e) => return Err(e),
                }
            }
        }
        let mut request = create_room::v3::Request::new();
        request.name = Some(room.name.clone());
        request.room_alias_name = Some(format!("hctl2_{correlation}"));
        request.preset = Some(create_room::v3::RoomPreset::PrivateChat);
        if space {
            request.creation_content = Some(ruma::serde::Raw::from_json_string(
                json!({"type":"m.space"}).to_string(),
            )?);
        }
        request.initial_state.push(ruma::serde::Raw::from_json_string(json!({
            "type":"io.hctl2.creation","state_key":"","content":{"command":correlation,"room":room.id,"project":room.project_id}
        }).to_string())?);
        before_dispatch()?;
        match self.request(request) {
            Ok(response) => {
                self.creation_readback(room, response.room_id.as_str(), &correlation, space)
            }
            Err(e) if e.code == "CHAT_ALIAS_EXISTS" => {
                let r = self.request(get_alias::v3::Request::new(
                    alias.parse().map_err(|_| invalid("invalid alias"))?,
                ))?;
                self.creation_readback(room, r.room_id.as_str(), &correlation, space)
            }
            Err(e) => Err(e),
        }
    }

    fn creation_readback(
        &self,
        room: &chat::Room,
        external: &str,
        correlation: &str,
        space: bool,
    ) -> Result<Value> {
        self.guard(external)?;
        if (self.state(external, "m.room.create")?["type"] == "m.space") != space {
            return Err(invalid("creation readback has wrong room type"));
        }
        let marker = self.state(external, "io.hctl2.creation")?;
        if marker != json!({"command":correlation,"room":room.id,"project":room.project_id}) {
            return Err(reject(
                "CHAT_CORRELATION_MISMATCH",
                "room alias does not prove original creation",
                "inspect_room",
            ));
        }
        Ok(json!({"matrix_room_id":external}))
    }

    pub fn join(&self, room: &str) -> Result<()> {
        self.request(join_room_by_id::v3::Request::new(room_id(room)?))?;
        self.guard(room)
    }

    pub fn send(&self, room: &str, txn: &str, body: &str) -> Result<Value> {
        self.send_thread(room, txn, body, None)
    }

    pub(super) fn thread_root(&self, room: &str, root: &str) -> Result<Value> {
        let event = self.event(room, root)?;
        if event["type"] != "m.room.message"
            || event["content"]["m.relates_to"]["rel_type"] == "m.thread"
        {
            return Err(invalid(
                "thread root must be a timeline Message, not a nested thread",
            ));
        }
        Ok(event)
    }

    pub(super) fn send_thread(
        &self,
        room: &str,
        txn: &str,
        body: &str,
        root: Option<&str>,
    ) -> Result<Value> {
        self.guard(room)?;
        let mut content = RoomMessageEventContent::text_plain(body);
        if let Some(root) = root {
            self.thread_root(room, root)?;
            content.relates_to = Some(ruma::events::room::message::Relation::Thread(
                ruma::events::relation::Thread::without_fallback(
                    root.parse().map_err(|_| invalid("invalid root event ID"))?,
                ),
            ));
        }
        let response = self.request(send_message_event::v3::Request::new(
            room_id(room)?,
            txn.into(),
            &content,
        )?)?;
        // Native Matrix transaction ID provides retry/readback; never create a new key on retry.
        let event = self.event(room, response.event_id.as_str())?;
        if event["content"] != serde_json::to_value(&content)? {
            return Err(reject(
                "CHAT_CONTENT_MISMATCH",
                "send readback differs",
                "inspect_room",
            ));
        }
        Ok(
            json!({"event_id":response.event_id,"content_digest":bytes_sha256(&canonical_json(&event["content"])?) }),
        )
    }

    pub fn event(&self, room: &str, event: &str) -> Result<Value> {
        self.guard(room)?;
        let response = self.request(get_room_event::v3::Request::new(
            room_id(room)?,
            event.parse().map_err(|_| invalid("invalid event ID"))?,
        ))?;
        let value: Value = serde_json::from_str(response.event.json().get())?;
        if value["event_id"] != event || value.get("room_id").is_some_and(|v| v != room) {
            return Err(invalid("event does not match requested room and event ID"));
        }
        Ok(value)
    }

    /// One server-ordered page. Cursor is opaque; timestamps and IDs never sort the timeline.
    pub fn timeline(&self, room: &str, from: Option<String>) -> Result<Value> {
        self.page(room, from, false)
    }

    pub(super) fn page(&self, room: &str, from: Option<String>, backwards: bool) -> Result<Value> {
        self.guard(room)?;
        let mut request = if backwards {
            get_message_events::v3::Request::backward(room_id(room)?)
        } else {
            get_message_events::v3::Request::forward(room_id(room)?)
        };
        request.from = from;
        request.limit = 100_u32.into();
        let response = self.request(request)?;
        let events = response
            .chunk
            .into_iter()
            .map(|e| serde_json::from_str::<Value>(e.json().get()))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(json!({"events":events,"start":response.start,"next":response.end,"current":true}))
    }

    pub(super) fn context(&self, room: &str, event: &str) -> Result<Value> {
        self.guard(room)?;
        let mut request = ruma::api::client::context::get_context::v3::Request::new(
            room_id(room)?,
            event.parse().map_err(|_| invalid("invalid event ID"))?,
        );
        request.limit = 0_u32.into();
        let response = self.request(request)?;
        Ok(json!({"start":response.start,"end":response.end}))
    }

    pub(super) fn thread_events(&self, room: &str, root: &str) -> Result<Vec<Value>> {
        self.thread_root(room, root)?;
        let mut events = vec![];
        let mut cursor = None;
        for _ in 0..100 {
            let mut request =
                ruma::api::client::relations::get_relating_events_with_rel_type::v1::Request::new(
                    room_id(room)?,
                    root.parse().map_err(|_| invalid("invalid event ID"))?,
                    ruma::events::relation::RelationType::Thread,
                );
            request.dir = ruma::api::Direction::Forward;
            request.from = cursor.clone();
            request.limit = Some(100_u32.into());
            let page = self.request(request)?;
            for event in page.chunk {
                events.push(serde_json::from_str(event.json().get())?);
            }
            if page.next_batch.is_none() || page.next_batch == cursor {
                return Ok(events);
            }
            cursor = page.next_batch;
        }
        Err(reject(
            "CHAT_HISTORY_LIMIT",
            "thread selection exceeds request budget",
            "select_event_ids",
        ))
    }

    /// Incremental resync uses a homeserver cursor, not a control-side event ordering.
    pub fn sync(&self, since: Option<String>) -> Result<Value> {
        self.sync_room(since, None)
    }

    pub(super) fn sync_room(&self, since: Option<String>, selected: Option<&str>) -> Result<Value> {
        let mut request = ruma::api::client::sync::sync_events::v3::Request::new();
        if let Some(room) = selected {
            let mut filter = ruma::api::client::filter::FilterDefinition::default();
            filter.room.rooms = Some(vec![room_id(room)?]);
            request.filter =
                Some(ruma::api::client::sync::sync_events::v3::Filter::FilterDefinition(filter));
        }
        request.since = since;
        request.timeout = Some(Duration::ZERO);
        let response = self.request(request)?;
        let mut rooms = serde_json::Map::new();
        for (room_id, room) in response.rooms.join {
            if selected.is_some_and(|id| id != room_id.as_str()) {
                continue;
            }
            self.guard(room_id.as_str())?;
            let events = room
                .timeline
                .events
                .iter()
                .map(|e| serde_json::from_str::<Value>(e.json().get()))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            rooms.insert(room_id.to_string(),json!({"events":events,"limited":room.timeline.limited,
                "prev_batch":room.timeline.prev_batch,"unread":room.unread_notifications.notification_count}));
        }
        Ok(json!({"next_batch":response.next_batch,"rooms":rooms}))
    }

    pub fn view_state(&self, room: &str, client: &str) -> Result<Value> {
        self.guard(room)?;
        let request = ruma::api::client::config::get_room_account_data::v3::Request::new(
            self.server
                .sender
                .parse()
                .map_err(|_| invalid("invalid sender"))?,
            room_id(room)?,
            format!("io.hctl2.view.{}", bytes_sha256(client.as_bytes())).into(),
        );
        match self.request(request) {
            Ok(r) => Ok(serde_json::from_str(r.account_data.json().get())?),
            Err(e) if e.code == "CHAT_NOT_FOUND" => Ok(json!({"draft":"","read_cursor":null})),
            Err(e) => Err(e),
        }
    }

    /// Derived client state lives in Matrix account data. It is not a Room binding or
    /// governance record, and can be reread after deleting all local caches.
    pub fn set_view_state(
        &self,
        room: &str,
        client: &str,
        draft: &str,
        read_cursor: Option<String>,
    ) -> Result<Value> {
        self.guard(room)?;
        if client.is_empty() {
            return Err(invalid("client identity required"));
        }
        let value = json!({"draft":draft,"read_cursor":read_cursor});
        let request = ruma::api::client::config::set_room_account_data::v3::Request::new_raw(
            self.server
                .sender
                .parse()
                .map_err(|_| invalid("invalid sender"))?,
            room_id(room)?,
            format!("io.hctl2.view.{}", bytes_sha256(client.as_bytes())).into(),
            ruma::serde::Raw::from_json_string(value.to_string())?,
        );
        self.request(request)?;
        self.view_state(room, client)
    }
}

pub(super) fn source_text(binding: Reference, event: &Value) -> Result<SourceText> {
    let id = event["event_id"]
        .as_str()
        .ok_or_else(|| invalid("event ID missing"))?;
    if event["type"] != "m.room.message" {
        return Err(invalid("source is not a Message"));
    }
    let excerpt = event["content"]["body"]
        .as_str()
        .ok_or_else(|| {
            reject(
                "SOURCE_UNAVAILABLE",
                "Message is redacted or unavailable",
                "choose_readable_source",
            )
        })?
        .to_owned();
    let body = String::from_utf8(canonical_json(&event["content"])?)
        .map_err(|_| invalid("invalid UTF-8"))?;
    Ok(SourceText {
        source: Source::Message {
            binding,
            event_id: id.into(),
            content_digest: bytes_sha256(body.as_bytes()),
        },
        body,
        excerpt,
    })
}

pub(super) fn room_id(room: &str) -> Result<OwnedRoomId> {
    room.parse().map_err(|_| invalid("invalid Matrix room ID"))
}
fn unavailable() -> store::StoreError {
    reject(
        "CHAT_UNAVAILABLE",
        "chat server current read unavailable",
        "check_chat_server",
    )
}

#[cfg(all(test, hctl_chat_native))]
#[path = "native_tests.rs"]
mod native_tests;

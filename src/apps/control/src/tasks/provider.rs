//! Pinned gh/tea native APIs. An adapter returns observations; it cannot select a Project.
use crate::{scm::Hosted, services::Supervisor};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
#[cfg(all(test, hctl_task_native))]
use std::process::Command;
use store::{ExternalEntity, Reference};
use task::{Card, Dependencies, Group, GroupKind, Result, Snapshot, Source, reject};

pub(super) enum Client {
    Github {
        gh: PathBuf,
        host: String,
        reads: RefCell<super::github::Reads>,
    },
    Gitea(Hosted),
}
impl Client {
    pub fn connect(
        src: &Source,
        root: &Path,
        control_id: &str,
        services: &Supervisor,
    ) -> Result<Self> {
        if src.candidate.provider == "gitea_issues" {
            let hosted = Hosted::existing(root, control_id, services)?;
            if hosted.url != src.platform.instance {
                return Err(reject(
                    "PLATFORM_CHANGED",
                    "hosted endpoint differs from binding",
                    "inspect_binding",
                ));
            }
            Ok(Self::Gitea(hosted))
        } else if src.candidate.provider == "github_issues" {
            let requested = std::env::var_os("HCTL2_GH")
                .or_else(|| {
                    services
                        .gitea_paths()
                        .map(|(i, _)| i.join("libexec/hctl2/gh").into_os_string())
                })
                .unwrap_or_else(|| "gh".into());
            let gh = foundation::git::resolve_executable(&requested)
                .ok_or_else(|| reject("PROVIDER_UNAVAILABLE", "gh unavailable", "install_gh"))?;
            Ok(Self::github(gh, src.platform.instance.clone()))
        } else {
            Err(reject(
                "PROVIDER_UNSUPPORTED",
                "source not implemented in P2.2",
                "choose_platform_issues",
            ))
        }
    }
    pub fn api(&self, method: &str, path: &str, body: Option<Value>) -> Result<Option<Value>> {
        match self {
            Self::Gitea(h) => h.api(method, path, body),
            Self::Github { gh, host, reads } => {
                reads.borrow_mut().api(gh, host, method, path, body)
            }
        }
    }
    fn required(&self, path: &str) -> Result<Value> {
        self.api("GET", path, None)?.ok_or_else(|| {
            reject(
                "SOURCE_UNAVAILABLE",
                "required source object missing",
                "refresh_source",
            )
        })
    }
    fn list(&self, path: &str) -> Result<Vec<Value>> {
        // Gitea's issue-comments route returns the whole thread and ignores page/limit.
        if matches!(self, Self::Gitea(_)) && path.ends_with("/comments") {
            return self.required(path)?.as_array().cloned().ok_or_else(|| {
                reject(
                    "PROVIDER_RESPONSE",
                    "expected comment array",
                    "inspect_source",
                )
            });
        }
        let mut all = vec![];
        // Explicit bounded pagination: never silently call a truncated board complete.
        for page in 1..=1000 {
            let sep = if path.contains('?') { '&' } else { '?' };
            let v = self.required(&format!("{path}{sep}per_page=100&limit=100&page={page}"))?;
            let values = v.as_array().ok_or_else(|| {
                reject("PROVIDER_RESPONSE", "expected API array", "inspect_source")
            })?;
            if !values.is_empty() && all.ends_with(values) {
                return Err(reject(
                    "SOURCE_INCOMPLETE",
                    "provider repeated a page",
                    "inspect_source",
                ));
            }
            all.extend(values.clone());
            if values.is_empty() {
                return Ok(all);
            }
        }
        Err(reject(
            "SOURCE_INCOMPLETE",
            "pagination bound reached",
            "narrow_source_scope",
        ))
    }
    fn repository(&self, src: &Source) -> Result<Value> {
        let repo = self.required(&format!("repos/{}", src.platform.full_name))?;
        let user = self.required("user")?;
        if id(&repo["id"])? != src.platform.stable_id
            || id(&user["id"])? != src.platform.account_id
            || repo["full_name"].as_str() != Some(&src.platform.full_name)
        {
            return Err(reject(
                "PLATFORM_CHANGED",
                "platform identity differs from frozen binding",
                "inspect_binding",
            ));
        }
        Ok(repo)
    }
    pub fn can_delete(&self, src: &Source) -> Result<bool> {
        let repo = self.repository(src)?;
        Ok(repo["permissions"]["admin"].as_bool() == Some(true))
    }
    pub fn snapshot(&self, src: &Source, binding: Reference) -> Result<Snapshot> {
        self.snapshot_since(src, binding, None)
    }

    pub fn snapshot_since(
        &self,
        src: &Source,
        binding: Reference,
        previous: Option<&Snapshot>,
    ) -> Result<Snapshot> {
        let base = format!("repos/{}", src.platform.full_name);
        let repo = self.repository(src)?;
        let mut groups = vec![];
        for (kind, path) in [
            (GroupKind::Label, format!("{base}/labels")),
            (GroupKind::Milestone, format!("{base}/milestones?state=all")),
        ] {
            for g in self.list(&path)? {
                groups.push(Group {
                    kind: kind.clone(),
                    anchor_stable_id: id(&g["id"])?,
                });
            }
        }
        // Native `since` narrows the issue list. A periodic full scan remains necessary:
        // deletions and relation changes are not promised to advance updated_at.
        let since = previous.and_then(since_cursor);
        let mut cards = if since.is_some() {
            previous.unwrap().cards.clone()
        } else {
            vec![]
        };
        let path = format!(
            "{base}/issues?state=all&type=issues{}",
            since
                .as_ref()
                .map(|s| format!("&since={s}"))
                .unwrap_or_default()
        );
        for raw in self.list(&path)? {
            if raw.get("pull_request").is_some_and(|v| !v.is_null()) {
                continue;
            }
            let mut c = normalize(src, raw)?;
            if since.is_some()
                && cards
                    .iter()
                    .any(|old| old.entity == c.entity && old.raw == c.raw && !old.tombstone)
            {
                continue;
            }
            c.dependencies = self.dependencies(src, c.number, &repo)?;
            if c.raw["comments"].as_u64().unwrap_or(0) > 0 {
                c.comments = self.list(&format!("{base}/issues/{}/comments", c.number))?;
            }
            cards.retain(|old| old.entity != c.entity);
            cards.push(c);
        }
        cards.sort_by(|a, b| {
            a.entity
                .immutable_external_entity_id
                .cmp(&b.entity.immutable_external_entity_id)
        });
        groups.sort_by_key(|g| format!("{:?}:{}", g.kind, g.anchor_stable_id));
        Ok(Snapshot {
            source: binding,
            observed_at: task::now(),
            complete: true,
            error: None,
            cards,
            stable_groups: groups,
        })
    }

    /// Commands concerning one existing card do not traverse unrelated issues.
    pub fn refresh_card(
        &self,
        src: &Source,
        previous: &Snapshot,
        entity_id: &str,
    ) -> Result<Snapshot> {
        let repo = self.repository(src)?;
        let original = previous
            .cards
            .iter()
            .find(|c| c.entity.immutable_external_entity_id == entity_id)
            .ok_or_else(|| {
                reject(
                    "CARD_NOT_OBSERVED",
                    "card not in the current source snapshot",
                    "refresh_source",
                )
            })?;
        let mut card = match self.read_card(src, original)? {
            Some(mut c) => {
                c.dependencies = self.dependencies(src, c.number, &repo)?;
                if c.raw["comments"].as_u64().unwrap_or(0) > 0 {
                    c.comments = self.list(&format!(
                        "repos/{}/issues/{}/comments",
                        src.platform.full_name, c.number
                    ))?;
                }
                c
            }
            None => {
                let mut c = original.clone();
                c.tombstone = true;
                c
            }
        };
        // An exact known entity is required above, including across redirects.
        card.entity = original.entity.clone();
        let mut snapshot = previous.clone();
        if let Some(old) = snapshot.cards.iter_mut().find(|c| c.entity == card.entity) {
            *old = card;
        }
        snapshot.observed_at = task::now();
        Ok(snapshot)
    }
    fn dependencies(&self, src: &Source, number: u64, repo: &Value) -> Result<Dependencies> {
        let base = format!("repos/{}/issues/{number}", src.platform.full_name);
        Ok(match self {
            Self::Github { .. } => Dependencies {
                parent: self.api("GET", &format!("{base}/parent"), None)?,
                children: self.list(&format!("{base}/sub_issues"))?,
                blocked_by: self.list(&format!("{base}/dependencies/blocked_by"))?,
                blocking: self.list(&format!("{base}/dependencies/blocking"))?,
            },
            Self::Gitea(_) => Dependencies {
                blocked_by: if repo["internal_tracker"]["enable_issue_dependencies"].as_bool()
                    == Some(false)
                {
                    vec![]
                } else {
                    self.list(&format!("{base}/dependencies"))?
                },
                blocking: if repo["internal_tracker"]["enable_issue_dependencies"].as_bool()
                    == Some(false)
                {
                    vec![]
                } else {
                    self.list(&format!("{base}/blocks"))?
                },
                ..Dependencies::default()
            },
        })
    }
    fn read_card(&self, src: &Source, original: &Card) -> Result<Option<Card>> {
        let raw = self.api(
            "GET",
            &format!(
                "repos/{}/issues/{}",
                src.platform.full_name, original.number
            ),
            None,
        )?;
        match raw {
            None => Ok(None),
            Some(mut raw) => {
                // Gitea 1.27 的 issue 载荷不带关闭者（实测），已关闭且缺该字段时补读时间线，
                // 只补这一个字段；其余投影仍以 issue 载荷为准。
                if matches!(self, Self::Gitea(_))
                    && raw.get("closed_by").is_none()
                    && raw["state"] == json!("closed")
                    && let Some(closer) = self.closer(src, original.number)?
                {
                    raw["closed_by"] = closer;
                }
                let c = normalize(src, raw)?;
                if c.entity != original.entity {
                    return Err(reject(
                        "ENTITY_CHANGED",
                        "original issue redirects to another entity",
                        "inspect_original_card",
                    ));
                }
                Ok(Some(c))
            }
        }
    }

    /// 「谁关闭的」在 Gitea 上只出现在 issue 时间线的 `close` 事件里。
    fn closer(&self, src: &Source, number: u64) -> Result<Option<Value>> {
        let path = format!("repos/{}/issues/{number}/timeline", src.platform.full_name);
        let Some(entries) = self.api("GET", &path, None)? else {
            return Ok(None);
        };
        Ok(entries
            .as_array()
            .and_then(|list| list.iter().rev().find(|e| e["type"] == json!("close")))
            .and_then(|e| e.get("user").cloned()))
    }
    #[cfg(test)]
    pub fn effect(&self, src: &Source, e: &store::EffectIntent, send: bool) -> Result<Card> {
        self.effect_with_dispatch(src, e, send, || Ok(()), |_| Ok(()))
    }

    /// Mark the intent as possibly sent only after all read-only checks have passed.
    pub fn effect_with_dispatch(
        &self,
        src: &Source,
        e: &store::EffectIntent,
        send: bool,
        before_send: impl FnOnce() -> Result<()>,
        mut rejected: impl FnMut(Value) -> Result<()>,
    ) -> Result<Card> {
        self.repository(src)?;
        let write = &e.input["write"];
        let base = format!("repos/{}/issues", src.platform.full_name);
        if e.operation == "task.create" {
            let find = || -> Result<Option<Card>> {
                let mut matched = vec![];
                for raw in self.list(&format!("{base}?state=all&type=issues"))? {
                    if raw.get("pull_request").is_some_and(|v| !v.is_null()) {
                        continue;
                    }
                    if raw["body"].as_str().is_some_and(|b| {
                        b.contains(write["marker"].as_str().unwrap_or("invalid-marker"))
                    }) {
                        let c = normalize(src, raw)?;
                        if c.title != write["title"].as_str().unwrap_or("")
                            || c.body != write["body"].as_str().unwrap_or("")
                        {
                            return Err(unknown());
                        }
                        matched.push(c);
                    }
                }
                if matched.len() > 1 {
                    return Err(unknown());
                }
                Ok(matched.pop())
            };
            if let Some(c) = find()? {
                return Ok(c);
            }
            if send {
                self.ready_to_send()?;
                before_send()?;
                let result = self.api(
                    "POST",
                    &base,
                    Some(json!({"title":write["title"],"body":write["body"]})),
                );
                check_rejection(&result, false, json!({"matching_cards":[]}), &mut rejected)?;
            }
            return find()?.ok_or_else(unknown);
        }
        let original: Card = serde_json::from_value(write["card"].clone())?;
        let path = format!("{base}/{}", original.number);
        let current = self.read_card(src, &original)?;
        if e.operation == "task.delete" {
            if !self.can_delete(src)? {
                return Err(reject(
                    "PERMISSION_DENIED",
                    "delete readback requires current repository admin access; 404 alone is not proof",
                    "restore_provider_access",
                ));
            }
            if current.is_none() {
                let mut c = original;
                c.tombstone = true;
                return Ok(c);
            }
            if send {
                self.ready_to_send()?;
                before_send()?;
                let result = match self {
                    Self::Gitea(_) => {
                        self.api("DELETE", &path, None)
                    }
                    Self::Github { .. } => {
                        self.api("POST", "graphql", Some(json!({
                            "query": "mutation($id:ID!){deleteIssue(input:{issueId:$id}){clientMutationId}}",
                            "variables": {"id":original.entity.immutable_external_entity_id}
                        })))
                    }
                };
                check_rejection(&result, false, json!({"card":current}), &mut rejected)?;
            }
            if self.read_card(src, &original)?.is_none() {
                let mut c = original;
                c.tombstone = true;
                return Ok(c);
            }
            return Err(unknown());
        }
        let current = current.ok_or_else(unknown)?;
        let fields = &write["fields"];
        if let Some(comment) = fields["comment"].as_str() {
            let found = || -> Result<Option<Value>> {
                let matching = self
                    .list(&format!("{path}/comments"))?
                    .into_iter()
                    .filter(|v| v["body"].as_str() == Some(comment))
                    .collect::<Vec<_>>();
                if matching.len() > 1 {
                    return Err(unknown());
                }
                Ok(matching.into_iter().next())
            };
            if let Some(comment) = found()? {
                let mut card = current;
                card.comments = vec![comment];
                return Ok(card);
            }
            if send {
                self.ready_to_send()?;
                before_send()?;
                let result = self.api(
                    "POST",
                    &format!("{path}/comments"),
                    Some(json!({"body":comment})),
                );
                check_rejection(
                    &result,
                    false,
                    json!({"card":current,"matching_comments":[]}),
                    &mut rejected,
                )?;
            }
            if let Some(comment) = found()? {
                let mut card = self.read_card(src, &original)?.ok_or_else(unknown)?;
                card.comments = vec![comment];
                return Ok(card);
            }
            return Err(unknown());
        }
        let matches = |c: &Card| -> bool {
            fields["title"].as_str().is_none_or(|s| s == c.title)
                && fields["body"].as_str().is_none_or(|s| s == c.body)
                && fields["state"].as_str().is_none_or(|s| s == c.stage)
        };
        if matches(&current) {
            return Ok(current);
        }
        if send {
            if current.remote_revision != original.remote_revision
                || current.title != original.title
                || current.body != original.body
                || current.stage != original.stage
            {
                return Err(reject(
                    "REMOTE_CONFLICT",
                    "card changed before write",
                    "refresh_and_preview",
                ));
            }
            let mut body = fields.as_object().ok_or_else(unknown)?.clone();
            body.retain(|_, v| !v.is_null());
            if src.capabilities.conditional_write {
                body.insert(
                    "content_version".into(),
                    json!(original.content_version.ok_or_else(unknown)?),
                );
            }
            // Gitea's content_version is only atomic for body. A mixed PATCH can
            // change a title before a later 409; do not declare that attempt rejected.
            let atomic_body = matches!(self, Self::Gitea(_))
                && body.len() == 2
                && body.contains_key("body")
                && body.contains_key("content_version");
            self.ready_to_send()?;
            before_send()?;
            let result = self.api("PATCH", &path, Some(Value::Object(body)));
            check_rejection(&result, atomic_body, json!({"card":current}), &mut rejected)?;
        }
        let c = self.read_card(src, &original)?.ok_or_else(unknown)?;
        if matches(&c) { Ok(c) } else { Err(unknown()) }
    }

    fn ready_to_send(&self) -> Result<()> {
        match self {
            Self::Github { reads, .. } => reads.borrow().ready(),
            Self::Gitea(_) => Ok(()),
        }
    }
}

fn check_rejection(
    response: &Result<Option<Value>>,
    atomic_body: bool,
    prior_read: Value,
    rejected: &mut impl FnMut(Value) -> Result<()>,
) -> Result<()> {
    if let Err(error) = response
        && (error.code == "NATIVE_REJECTED" || atomic_body && error.code == "NATIVE_CONFLICT")
    {
        rejected(
            json!({"provider_error":{"code":error.code,"message":error.message},"prior_read":prior_read}),
        )?;
        return Err(reject(
            "PROVIDER_REJECTED",
            error.message.clone(),
            "refresh_and_preview",
        ));
    }
    Ok(())
}

fn unknown() -> task::StoreError {
    reject(
        "RESULT_UNKNOWN",
        "original write not confirmed; retry reads it without resending",
        "resume_effect",
    )
}

fn since_cursor(snapshot: &Snapshot) -> Option<String> {
    let times = snapshot
        .cards
        .iter()
        .filter(|c| !c.tombstone)
        .map(|c| c.remote_revision.as_str())
        .collect::<Vec<_>>();
    let first = *times.first()?;
    // Preserve the provider's timezone; do not implement date conversion. If the
    // provider changes its representation, fall back to a complete read instead.
    let valid = |time: &str| {
        time.is_ascii()
            && (time.len() == 20 || time.len() == 25)
            && time.bytes().enumerate().all(|(i, b)| match i {
                4 | 7 => b == b'-',
                10 => b == b'T',
                13 | 16 | 22 => b == b':',
                19 => {
                    if time.len() == 20 {
                        b == b'Z'
                    } else {
                        b == b'+' || b == b'-'
                    }
                }
                _ => b.is_ascii_digit(),
            })
    };
    if !valid(first) || times.iter().any(|t| !valid(t) || t[19..] != first[19..]) {
        return None;
    }
    let latest = times.into_iter().max()?;
    // Overlap the last minute so two edits sharing a timestamp are not skipped.
    Some(format!("{}00{}", &latest[..17], &latest[19..]).replace('+', "%2B"))
}
fn id(value: &Value) -> Result<String> {
    value
        .as_u64()
        .filter(|n| *n > 0)
        .map(|n| n.to_string())
        .ok_or_else(|| {
            reject(
                "PROVIDER_RESPONSE",
                "stable numeric ID missing",
                "refresh_source",
            )
        })
}
fn normalize(src: &Source, raw: Value) -> Result<Card> {
    let entity_id = if src.candidate.provider == "github_issues" {
        raw["node_id"].as_str().ok_or_else(unknown)?.to_owned()
    } else {
        id(&raw["id"])?
    };
    let mut groups = vec![];
    if raw["milestone"].is_object() {
        groups.push(Group {
            kind: GroupKind::Milestone,
            anchor_stable_id: id(&raw["milestone"]["id"])?,
        });
    }
    for l in raw["labels"].as_array().into_iter().flatten() {
        groups.push(Group {
            kind: GroupKind::Label,
            anchor_stable_id: id(&l["id"])?,
        });
    }
    Ok(Card {
        entity: ExternalEntity {
            provider: format!("{}@{}", src.candidate.provider, src.platform.instance),
            account_stable_id: src.platform.account_id.clone(),
            external_entity_kind: "issue".into(),
            immutable_external_entity_id: entity_id,
        },
        number: raw["number"].as_u64().ok_or_else(unknown)?,
        title: raw["title"].as_str().ok_or_else(unknown)?.into(),
        body: raw["body"].as_str().unwrap_or("").into(),
        stage: raw["state"].as_str().ok_or_else(unknown)?.into(),
        remote_revision: raw["updated_at"].as_str().ok_or_else(unknown)?.into(),
        content_version: raw["content_version"].as_i64(),
        groups,
        dependencies: Dependencies::default(),
        comments: vec![],
        raw,
        tombstone: false,
    })
}

#[cfg(test)]
#[path = "provider_tests.rs"]
mod tests;

#[cfg(all(test, hctl_task_native))]
#[path = "native_tests.rs"]
mod native_tests;

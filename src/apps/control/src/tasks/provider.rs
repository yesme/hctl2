//! Pinned gh/tea native APIs. An adapter returns observations; it cannot select a Project.
use crate::{scm::Hosted, services::Supervisor};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;
use store::{ExternalEntity, Reference};
use task::{Card, Dependencies, Group, GroupKind, Result, Snapshot, Source, reject};

pub(super) enum Client {
    Github { gh: PathBuf, host: String },
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
            let hosted = Hosted::connect(root, control_id, services)?;
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
            Ok(Self::Github {
                gh,
                host: src.platform.instance.clone(),
            })
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
            Self::Github { gh, host } => {
                let mut cmd = Command::new(gh);
                cmd.env("GH_PROMPT_DISABLED", "1")
                    .env("NO_COLOR", "1")
                    .env_remove("GH_DEBUG")
                    .args(["api", "--include", "--hostname", host, "--method", method]);
                if body.is_some() {
                    cmd.args(["--input", "-"]);
                }
                cmd.arg(path);
                let out = repo::git::run(&mut cmd, body.map(|v| v.to_string().into_bytes()))?;
                let text = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");
                let (headers, body) = text.split_once("\n\n").ok_or_else(|| {
                    reject(
                        "PROVIDER_RESPONSE",
                        "missing GitHub HTTP headers",
                        "retry_read",
                    )
                })?;
                let status = headers
                    .lines()
                    .next()
                    .and_then(|s| s.split_whitespace().nth(1))
                    .and_then(|s| s.parse::<u16>().ok());
                if status == Some(404) {
                    return Ok(None);
                }
                if !out.status.success() || !status.is_some_and(|s| (200..300).contains(&s)) {
                    return Err(reject(
                        "PROVIDER_UNAVAILABLE",
                        format!("GitHub {method} failed (HTTP {status:?})"),
                        "read_back_original_intent",
                    ));
                }
                Ok(Some(if body.trim().is_empty() {
                    Value::Null
                } else {
                    serde_json::from_str(body)?
                }))
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
        let mut cards = vec![];
        for raw in self.list(&format!("{base}/issues?state=all&type=issues"))? {
            if raw.get("pull_request").is_some_and(|v| !v.is_null()) {
                continue;
            }
            let mut c = normalize(src, raw)?;
            c.dependencies = self.dependencies(src, c.number, &repo)?;
            if c.raw["comments"].as_u64().unwrap_or(0) > 0 {
                c.comments = self.list(&format!("{base}/issues/{}/comments", c.number))?;
            }
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
                blocking: self.list(&format!("{base}/blocks"))?,
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
            Some(raw) => {
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
    pub fn effect(&self, src: &Source, e: &store::EffectIntent, send: bool) -> Result<Card> {
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
                let _ = self.api(
                    "POST",
                    &base,
                    Some(json!({"title":write["title"],"body":write["body"]})),
                );
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
                match self {
                    Self::Gitea(_) => {
                        let _ = self.api("DELETE", &path, None);
                    }
                    Self::Github { .. } => {
                        let _ = self.api("POST", "graphql", Some(json!({
                            "query": "mutation($id:ID!){deleteIssue(input:{issueId:$id}){clientMutationId}}",
                            "variables": {"id":original.entity.immutable_external_entity_id}
                        })));
                    }
                }
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
            let found = || -> Result<bool> {
                Ok(self
                    .list(&format!("{path}/comments"))?
                    .iter()
                    .any(|v| v["body"].as_str() == Some(comment)))
            };
            if found()? {
                return Ok(current);
            }
            if send {
                let _ = self.api(
                    "POST",
                    &format!("{path}/comments"),
                    Some(json!({"body":comment})),
                );
            }
            if found()? {
                return self.read_card(src, &original)?.ok_or_else(unknown);
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
            let _ = self.api("PATCH", &path, Some(Value::Object(body)));
        }
        let c = self.read_card(src, &original)?.ok_or_else(unknown)?;
        if matches(&c) { Ok(c) } else { Err(unknown()) }
    }
}

fn unknown() -> task::StoreError {
    reject(
        "RESULT_UNKNOWN",
        "original write not confirmed; retry reads it without resending",
        "resume_effect",
    )
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

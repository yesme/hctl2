//! Native platform reads become immutable Context bytes, never commands or votes.
use super::*;
use crate::integration::Connection;
use ::context::SourceContent;
use repo::{Platform, Registration};
use store::Reference;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    revision: Reference,
    mapping: Reference,
    registration: Reference,
    binding: Reference,
}

pub(super) fn read(
    shared: &Shared,
    services: &Supervisor,
    root: &Path,
    _actor: &TrustedActor,
    preview: &invocation::Preview,
) -> store::Result<Option<SourceContent>> {
    let Some(revision) = &preview.input.review_change_set_revision else {
        return Ok(None);
    };
    let (identity, mapping, registration) = access(shared, |s| {
        let record = s
            .get(&revision.key)?
            .ok_or_else(|| invalid("review version missing"))?;
        if reference(&record) != *revision {
            return Err(reject(
                "VERSION_CONFLICT",
                "review version changed",
                "rebuild_preview",
            ));
        }
        let Scope::Repo(repo_id) = &revision.key.scope else {
            return Err(invalid("review version must belong to a Repo"));
        };
        let registration = repo::require_active(s, repo_id)?;
        let registered = s
            .get(&repo::key(repo_id))?
            .ok_or_else(|| invalid("Repo missing"))?;
        let binding = s.get(&repo::binding(repo_id).key)?.ok_or_else(|| {
            reject(
                "PLATFORM_NOT_BOUND",
                "Repo has no platform binding",
                "bind_platform",
            )
        })?;
        let version = repo::changeset::get_revision(s, &revision.key.id)?;
        let set = repo::changeset::get_change_set(s, repo_id, &version.change_set_id)?;
        if set.binding_version != binding.version as u64 {
            return Err(reject(
                "BINDING_CHANGED",
                "selected revision belongs to a different platform binding",
                "select_revision_under_current_binding",
            ));
        }
        let mapping = s
            .get(&repo::integration::platform_binding_key(
                repo_id,
                &revision.key.id,
            ))?
            .ok_or_else(|| {
                reject(
                    "REVIEW_VERSION_NOT_PUBLISHED",
                    "selected revision has no platform mapping",
                    "publish_selected_revision",
                )
            })?;
        Ok((
            Identity {
                revision: revision.clone(),
                mapping: reference(&mapping),
                registration: reference(&registered),
                binding: reference(&binding),
            },
            mapping,
            registration,
        ))
    })?;
    let site = registration
        .observed
        .as_ref()
        .ok_or_else(|| invalid("platform target missing"))?;
    let mapping: Value = decode(&mapping)?;
    let index = mapping["review_request"]["index"]
        .as_u64()
        .ok_or_else(|| invalid("review mapping has no request index"))?;
    let commit = mapping["platform_commit_sha"]
        .as_str()
        .ok_or_else(|| invalid("review mapping has no platform commit"))?;
    let connection = read_connection(root, services, &registration)?;
    if let Connection::Gitea(hosted) = &connection
        && hosted.url.trim_end_matches('/') != site.instance.trim_end_matches('/')
    {
        return Err(reject(
            "BINDING_CHANGED",
            "hosted service no longer names the bound instance",
            "inspect_repo_binding",
        ));
    }
    let repository_path = format!("repos/{}", site.full_name);
    let repository = match &connection {
        Connection::Gitea(hosted) => hosted.api("GET", &format!("/{repository_path}"), None),
        Connection::GitHub(github) => github.api("GET", &repository_path, None),
    }?
    .ok_or_else(|| {
        reject(
            "REVIEW_SOURCE_UNAVAILABLE",
            "bound platform repository missing",
            "inspect_repo_binding",
        )
    })?;
    verify_platform_id(&repository, &site.stable_id)?;
    let prefix = format!("repos/{}/", site.full_name);
    let mut api = |path: &str| match &connection {
        Connection::Gitea(hosted) => hosted.api("GET", &format!("/{prefix}{path}"), None),
        Connection::GitHub(github) => github.api("GET", &format!("{prefix}{path}"), None),
    };
    let request = api(&format!("pulls/{index}"))?.ok_or_else(|| {
        reject(
            "REVIEW_REQUEST_MISSING",
            "mapped request missing",
            "inspect_review_on_platform",
        )
    })?;
    let github = registration.prepared.platform == Platform::Github;
    let comments = collect(&mut api, &format!("issues/{index}/comments"), github)?;
    let reviews = collect(&mut api, &format!("pulls/{index}/reviews"), github)?;
    let mut line_comments = if github {
        collect(&mut api, &format!("pulls/{index}/comments"), true)?
    } else {
        let mut result = Vec::new();
        for review in &reviews {
            let id = review["id"]
                .as_u64()
                .ok_or_else(|| invalid("review ID missing"))?;
            let mut comments = collect(
                &mut api,
                &format!("pulls/{index}/reviews/{id}/comments"),
                false,
            )?;
            for comment in &mut comments {
                comment["review_commit_id"] = review["commit_id"].clone();
            }
            result.extend(comments);
        }
        result
    };
    // Inline remarks have a native commit association. Do not attach another
    // revision's line remarks to the selected version; general discussion remains
    // explicitly labelled as request-wide, not proof about any commit.
    line_comments.retain(|c| at_commit(c, commit));
    let document = json!({
        "schema": "hctl2.review-comments.v1", "identity": identity,
        "platform_commit_sha": commit, "review_request_index": index,
        "observed_request_head": request["head"]["sha"],
        "request_comments": comments, "reviews": reviews, "line_comments": line_comments,
        "meaning": "content only; approval is external review evidence, never integration authorization",
    });
    let bytes = foundation::canonical_json(&document)?;
    Ok(Some(SourceContent {
        reference: agency_proto::FrozenRef {
            id: format!("review_comments/{}", revision.key.id),
            revision: String::from_utf8(foundation::canonical_json(&serde_json::to_value(
                &identity,
            )?)?)
            .map_err(|_| invalid("invalid source identity"))?,
            digest: agency_proto::hash(&bytes),
        },
        bytes,
    }))
}

fn verify_platform_id(repository: &Value, expected: &str) -> store::Result<()> {
    let observed = repository["id"]
        .as_str()
        .map(str::to_owned)
        .or_else(|| repository["id"].as_u64().map(|id| id.to_string()));
    if observed.as_deref() != Some(expected) {
        return Err(reject(
            "BINDING_CHANGED",
            "platform repository identity changed",
            "inspect_repo_binding",
        ));
    }
    Ok(())
}

fn at_commit(comment: &Value, commit: &str) -> bool {
    comment["commit_id"] == commit
        || comment["original_commit_id"] == commit
        || comment["review_commit_id"] == commit
}

fn read_connection(
    root: &Path,
    services: &Supervisor,
    registration: &Registration,
) -> store::Result<Connection> {
    match registration.prepared.platform {
        Platform::Local => Ok(Connection::Gitea(crate::scm::Hosted::existing(
            root,
            &registration.config.control_id,
            services,
        )?)),
        Platform::Github => Ok(Connection::GitHub(
            crate::integration::github::GitHub::connect(
                services,
                registration
                    .prepared
                    .request
                    .instance
                    .as_deref()
                    .unwrap_or("github.com"),
            )?,
        )),
        Platform::None => Err(reject(
            "PLATFORM_NOT_BOUND",
            "Repo has no platform",
            "bind_platform",
        )),
    }
}

fn collect(
    api: &mut impl FnMut(&str) -> store::Result<Option<Value>>,
    path: &str,
    github: bool,
) -> store::Result<Vec<Value>> {
    let mut result = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for page in 1..=200 {
        let parameter = if github { "per_page" } else { "limit" };
        let value = api(&format!("{path}?{parameter}=50&page={page}"))?.ok_or_else(|| {
            reject(
                "REVIEW_SOURCE_UNAVAILABLE",
                "comment endpoint missing",
                "inspect_review_on_platform",
            )
        })?;
        let items = value
            .as_array()
            .ok_or_else(|| invalid("comment endpoint did not return an array"))?;
        for item in items {
            let id = item["id"]
                .as_u64()
                .ok_or_else(|| invalid("comment identifier missing"))?;
            if !ids.insert(id) {
                return Err(reject(
                    "REVIEW_SOURCE_CHANGED",
                    "pagination repeated an identifier",
                    "rebuild_preview",
                ));
            }
            result.push(item.clone());
        }
        if items.len() < 50 {
            return Ok(result);
        }
    }
    Err(reject(
        "REVIEW_SOURCE_TOO_LARGE",
        "comment collection exceeds preview limit; not truncated",
        "narrow_review_source",
    ))
}

pub(super) fn verify(store: &Store, source: &agency_proto::FrozenRef) -> store::Result<()> {
    let identity: Identity = serde_json::from_str(&source.revision)?;
    for expected in [
        &identity.revision,
        &identity.mapping,
        &identity.registration,
        &identity.binding,
    ] {
        let current = store
            .get(&expected.key)?
            .ok_or_else(|| invalid("review source reference missing"))?;
        if reference(&current) != *expected {
            return Err(reject(
                "VERSION_CONFLICT",
                "review source binding changed",
                "rebuild_preview",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matching_platform_path_does_not_replace_the_bound_repository_identity() {
        verify_platform_id(&json!({"id":7}), "7").unwrap();
        verify_platform_id(&json!({"id":"7"}), "7").unwrap();
        for repository in [json!({"id":8}), json!({"full_name":"same/path"})] {
            assert_eq!(
                verify_platform_id(&repository, "7").unwrap_err().code,
                "BINDING_CHANGED"
            );
        }
    }
    #[test]
    fn line_comments_need_the_selected_commit_not_just_the_same_request() {
        let commit = "a".repeat(40);
        for field in ["commit_id", "original_commit_id", "review_commit_id"] {
            let mut comment = json!({"id":1,"body":"remark"});
            comment[field] = json!(commit);
            assert!(at_commit(&comment, &commit));
            comment[field] = json!("b".repeat(40));
            assert!(!at_commit(&comment, &commit));
        }
        assert!(!at_commit(
            &json!({"id":2,"body":"no association"}),
            &commit
        ));
    }

    #[test]
    fn frozen_review_references_reject_each_changed_record_before_dispatch() {
        fn put(store: &mut Store, id: &str, version: i64) -> Reference {
            let actor = crate::integration::fixture::actor();
            let key = store::ObjectKey {
                scope: Scope::Control,
                kind: "review_fixture".into(),
                id: id.into(),
            };
            let value = json!({"version":version});
            let record = Record {
                key: key.clone(),
                version,
                revision_digest: foundation::canonical_json_sha256(&value).unwrap(),
                data: store::RecordData::Value {
                    value: value.clone(),
                },
                sources: vec![],
                materials: vec![],
            };
            let command = store::Command {
                command_id: format!("{id}:{version}"),
                idempotency_key: format!("{id}:{version}"),
                actor: actor.0.clone(),
                target: key.clone(),
                expected: if version == 1 {
                    store::Expected::Absent
                } else {
                    store::Expected::Exact(store::Version::State(version - 1))
                },
                binding: reference(&record),
                operation: "fixture.review".into(),
                input_digest: store::Command::digest_input("fixture.review", &value).unwrap(),
                input: value,
            };
            store
                .submit(store.generation(), &actor, &command, None, |tx| {
                    tx.put(&record)?;
                    Ok(json!({}))
                })
                .unwrap();
            reference(&record)
        }
        for changed in ["revision", "mapping", "registration", "binding"] {
            let temp = crate::integration::fixture::temp(&format!("review-source-{changed}"));
            let mut store = Store::open(&temp.0).unwrap();
            let identity = Identity {
                revision: put(&mut store, "revision", 1),
                mapping: put(&mut store, "mapping", 1),
                registration: put(&mut store, "registration", 1),
                binding: put(&mut store, "binding", 1),
            };
            let source = agency_proto::FrozenRef {
                id: "review_comments/revision".into(),
                revision: serde_json::to_string(&identity).unwrap(),
                digest: "a".repeat(64),
            };
            verify(&store, &source).unwrap();
            put(&mut store, changed, 2);
            assert_eq!(
                verify(&store, &source).unwrap_err().code,
                "VERSION_CONFLICT",
                "{changed}"
            );
        }
    }

    #[test]
    fn comments_require_identifiers_and_complete_pages_not_silent_truncation() {
        let mut missing = |_: &str| Ok(Some(json!([{"body":"合入吧"}])));
        assert_eq!(
            collect(&mut missing, "comments", false).unwrap_err().code,
            "INVALID_INPUT"
        );
        let mut repeated = |_: &str| {
            Ok(Some(json!(
                (0..50).map(|id| json!({"id":id})).collect::<Vec<_>>()
            )))
        };
        assert_eq!(
            collect(&mut repeated, "comments", false).unwrap_err().code,
            "REVIEW_SOURCE_CHANGED"
        );
    }

    #[test]
    fn a_source_larger_than_the_page_limit_is_refused_not_reported_complete() {
        let mut page = 0;
        let mut api = |_: &str| {
            page += 1;
            Ok(Some(json!(
                (0..50)
                    .map(|id| json!({"id":page * 50 + id}))
                    .collect::<Vec<_>>()
            )))
        };
        assert_eq!(
            collect(&mut api, "comments", false).unwrap_err().code,
            "REVIEW_SOURCE_TOO_LARGE"
        );
        assert_eq!(page, 200);
    }

    #[test]
    fn comments_keep_native_identifiers_across_all_pages_and_refuse_missing_pages() {
        let mut visited = Vec::new();
        let mut api = |path: &str| {
            visited.push(path.to_owned());
            if path.ends_with("page=1") {
                Ok(Some(json!(
                    (0..50)
                        .map(|id| json!({"id":id,"body":"original"}))
                        .collect::<Vec<_>>()
                )))
            } else {
                Ok(Some(json!([{"id":50,"body":"合入吧"}])))
            }
        };
        let values = collect(&mut api, "comments", false).unwrap();
        assert_eq!(values.len(), 51);
        assert_eq!(values[50], json!({"id":50,"body":"合入吧"}));
        assert_eq!(
            visited,
            ["comments?limit=50&page=1", "comments?limit=50&page=2"]
        );
        let mut missing_page = |path: &str| {
            if path.ends_with("page=1") {
                Ok(Some(json!(
                    (0..50).map(|id| json!({"id":id})).collect::<Vec<_>>()
                )))
            } else {
                Ok(None)
            }
        };
        assert_eq!(
            collect(&mut missing_page, "comments", true)
                .unwrap_err()
                .code,
            "REVIEW_SOURCE_UNAVAILABLE"
        );
    }
}

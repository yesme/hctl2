//! The scripted platforms the integration and review-publishing tests run against: a `tea` /
//! `gh` stand-in whose answers are files the test controls, and a real bare repository as the
//! platform's Git, so pushes, merges and readbacks are real Git operations.
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use repo::integration::{self as domain};
use serde_json::{Value, json};
use store::{Actor, ActorSource, Scope, Store, TrustedActor};

use super::target::PlatformTarget;
use super::{Connection, Shared, drive_with, github};
use crate::scm::Hosted;

pub(crate) struct Temp(pub(crate) PathBuf);
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn actor() -> TrustedActor {
    TrustedActor(Actor {
        principal: "owner".into(),
        source: ActorSource::DirectClient,
        permission_scope: vec![Scope::Control],
        authority: None,
    })
}

pub(crate) const TEA: &str = r#"#!/bin/sh
# $0.state/: protection.json (absent = unprotected), rule_name (effective rule the branch
# record names), rule_path (the one path segment the rule answers under), pr_state, pr_head,
# pr_base, merge_sha, posts; switches: fail_merge (405), lose_merge (merge, then die without
# an HTTP line), lose_unmerged (die without merging or an HTTP line), drift_after_merge.
# $0.state/../platform.git is the platform's Git; merges are real commits there.
S="$0.state"; R="$S/../platform.git"
export GIT_AUTHOR_NAME=gitea GIT_AUTHOR_EMAIL=gitea@localhost GIT_COMMITTER_NAME=gitea GIT_COMMITTER_EMAIL=gitea@localhost
# The request record. Its head follows the branch when one is recorded (pr_branch), else the
# static pr_head the integration cases pin.
pr_json() {
  state="$(cat "$S/pr_state")"
  if [ "$state" = merged ]; then merged=true; sha="\"$(cat "$S/merge_sha")\""; else merged=false; sha=null; fi
  if [ -f "$S/pr_branch" ]; then head="$(git -C "$R" rev-parse --verify -q "refs/heads/$(cat "$S/pr_branch")")"; else head="$(cat "$S/pr_head")"; fi
  case "$state" in merged|closed) shown=closed ;; *) shown=open ;; esac
  printf '{"number":%s,"state":"%s","merged":%s,"merge_commit_sha":%s,"head":{"sha":"%s","ref":"%s"},"base":{"ref":"%s"},"title":"%s"}' \
    "$(cat "$S/pr_index" 2>/dev/null || echo 7)" "$shown" "$merged" "$sha" "$head" "$(cat "$S/pr_branch" 2>/dev/null)" "$(cat "$S/pr_base")" "$(cat "$S/pr_title" 2>/dev/null)"
}
method="$4"
if [ "$5" = --data ]; then path="$7"; body="$(cat)"; else path="$5"; fi
case "$method $path" in
  "GET "*/branches/*)
    if [ -f "$S/down" ]; then exit 1; fi
    head="$(git -C "$R" rev-parse --verify -q "refs/heads/${path##*/}")" || { printf 'HTTP/1.1 404 Not Found\n' >&2; printf '{}'; exit 1; }
    if [ -f "$S/protection.json" ]; then protected=true; name="$(cat "$S/rule_name")"; else protected=false; name=""; fi
    printf 'HTTP/1.1 200 OK\n' >&2
    printf '{"name":"%s","commit":{"id":"%s"},"protected":%s,"effective_branch_protection_name":"%s"}' "${path##*/}" "$head" "$protected" "$name" ;;
  "GET "*/branch_protections/*)
    if [ -f "$S/protection.json" ] && [ "${path##*/}" = "$(cat "$S/rule_path")" ]; then printf 'HTTP/1.1 200 OK\n' >&2; cat "$S/protection.json";
    else printf 'HTTP/1.1 404 Not Found\n' >&2; printf '{}'; exit 1; fi ;;
  "GET "*/pulls/*/*)
    # Gitea 1.22+: the request from <head> into <base>, looked up as pulls/<base>/<head>.
    if [ -f "$S/down" ]; then exit 1; fi
    rest="${path#*/pulls/}"; base="${rest%%/*}"; headb="${rest#*/}"
    if [ -f "$S/pr_exists" ] && [ "$base" = "$(cat "$S/pr_base")" ] && [ "$headb" = "$(cat "$S/pr_branch")" ]; then
      printf 'HTTP/1.1 200 OK\n' >&2; pr_json
    else printf 'HTTP/1.1 404 Not Found\n' >&2; printf '{"message":"pull request does not exist"}'; exit 1; fi ;;
  "GET "*/pulls/*)
    if [ ! -f "$S/pr_exists" ]; then printf 'HTTP/1.1 404 Not Found\n' >&2; printf '{}'; exit 1; fi
    printf 'HTTP/1.1 200 OK\n' >&2; pr_json ;;
  "POST "*/pulls)
    printf x >> "$S/creates"
    if [ -f "$S/down" ]; then exit 1; fi
    if [ -f "$S/pr_exists" ]; then printf 'HTTP/1.1 409 Conflict\n' >&2; printf '{"message":"pull request already exists for these targets"}'; exit 1; fi
    headb="${body#*\"head\":\"}"; headb="${headb%%\"*}"; base="${body#*\"base\":\"}"; base="${base%%\"*}"
    title="${body#*\"title\":\"}"; title="${title%%\"*}"; printf '%s' "$title" > "$S/pr_title"
    printf '%s' "$headb" > "$S/pr_branch"; printf '%s' "$base" > "$S/pr_base"; printf open > "$S/pr_state"; : > "$S/pr_exists"
    if [ -f "$S/lose_create" ]; then exit 1; fi
    printf 'HTTP/1.1 201 Created\n' >&2; pr_json ;;
  "PATCH "*/pulls/*)
    printf x >> "$S/updates"
    if [ -f "$S/down" ]; then exit 1; fi
    title="${body#*\"title\":\"}"; title="${title%%\"*}"; printf '%s' "$title" > "$S/pr_title"
    printf 'HTTP/1.1 201 Created\n' >&2; pr_json ;;
  "POST "*/pulls/*/merge)
    printf x >> "$S/posts"
    if [ -f "$S/lose_unmerged" ]; then exit 1; fi
    if [ -f "$S/fail_merge" ]; then printf 'HTTP/1.1 405 Method Not Allowed\n' >&2; printf '{"message":"required status check canary missing"}'; exit 1; fi
    if [ "$(cat "$S/pr_state")" = merged ]; then printf 'HTTP/1.1 405 Method Not Allowed\n' >&2; printf '{"message":"already merged"}'; exit 1; fi
    want="${body#*\"head_commit_id\":\"}"; want="${want%%\"*}"
    if [ "$want" != "$(cat "$S/pr_head")" ]; then printf 'HTTP/1.1 409 Conflict\n' >&2; printf '{"message":"head changed"}'; exit 1; fi
    base="$(cat "$S/pr_base")"; cand="$(cat "$S/pr_head")"
    # Like Gitea 1.27: a fast-forward-only merge's merge commit is the candidate itself.
    case "$body" in
      *fast-forward-only*) merge="$cand" ;;
      *) merge="$(git -C "$R" commit-tree "$cand^{tree}" -p "refs/heads/$base" -p "$cand" -m "merge #7")" ;;
    esac
    git -C "$R" update-ref "refs/heads/$base" "$merge"
    printf merged > "$S/pr_state"; printf '%s' "$merge" > "$S/merge_sha"
    if [ -f "$S/drift_after_merge" ]; then cp "$S/drift_after_merge" "$S/protection.json"; fi
    # Someone pushes to the source branch the instant the merge lands: the request's head
    # (which follows the branch) moves before anyone reads it back.
    if [ -f "$S/advance_source_after_merge" ]; then cp "$S/advance_source_after_merge" "$S/pr_head"; fi
    if [ -f "$S/lose_merge" ]; then exit 1; fi
    printf 'HTTP/1.1 200 OK\n' >&2 ;;
  *) printf 'HTTP/1.1 404 Not Found\n' >&2; printf '{}'; exit 1 ;;
esac
exit 0
"#;

pub(crate) const PROTECTION: &str = r#"{"rule_name":"main","branch_name":"main","enable_push":false,"enable_status_check":true,"status_check_contexts":["canary"],"required_approvals":0,"block_on_outdated_branch":false,"block_admin_merge_override":false,"created_at":"2026-10-07T00:00:00Z","updated_at":"2026-10-07T00:00:00Z"}"#;

pub(crate) const GH: &str = r#"#!/bin/sh
# Scripted GitHub: `api --hostname H --method M [--input -] PATH`. $0.state/: protection.json
# (absent = no classic protection), announce_protected (branch record says protected anyway),
# rules.json (ruleset rules in force; absent = []), pr_state, pr_head, pr_base, merge_sha,
# posts; switches as for tea. $0.state/../platform.git is the platform's Git.
S="$0.state"; R="$S/../platform.git"
export GIT_AUTHOR_NAME=github GIT_AUTHOR_EMAIL=github@localhost GIT_COMMITTER_NAME=github GIT_COMMITTER_EMAIL=github@localhost
pr_json() {
  state="$(cat "$S/pr_state")"
  if [ "$state" = merged ]; then merged=true; sha="\"$(cat "$S/merge_sha")\""; else merged=false; sha=null; fi
  if [ -f "$S/pr_branch" ]; then head="$(git -C "$R" rev-parse --verify -q "refs/heads/$(cat "$S/pr_branch")")"; else head="$(cat "$S/pr_head")"; fi
  case "$state" in merged|closed) shown=closed ;; *) shown=open ;; esac
  printf '{"number":%s,"state":"%s","merged":%s,"merge_commit_sha":%s,"head":{"sha":"%s","ref":"%s"},"base":{"ref":"%s"},"title":"%s"}' \
    "$(cat "$S/pr_index" 2>/dev/null || echo 3)" "$shown" "$merged" "$sha" "$head" "$(cat "$S/pr_branch" 2>/dev/null)" "$(cat "$S/pr_base")" "$(cat "$S/pr_title" 2>/dev/null)"
}
method=""; path=""; body=""
while [ $# -gt 0 ]; do
  case "$1" in
    --method) method="$2"; shift 2 ;;
    --hostname) shift 2 ;;
    --input) body="$(cat)"; shift 2 ;;
    api) shift ;;
    *) path="$1"; shift ;;
  esac
done
case "$method $path" in
  "GET "*/branches/*/protection)
    if [ -f "$S/protection.json" ]; then cat "$S/protection.json"; exit 0; fi
    printf '{"message":"Branch not protected"}'; printf 'gh: Branch not protected (HTTP 404)\n' >&2; exit 1 ;;
  "GET "*/rules/branches/*)
    if [ -f "$S/rules.json" ]; then cat "$S/rules.json"; else printf '[]'; fi ;;
  "GET "*/branches/*)
    if [ -f "$S/down" ]; then printf 'gh: connection refused\n' >&2; exit 1; fi
    b="${path##*/}"
    head="$(git -C "$R" rev-parse --verify -q "refs/heads/$b")" || { printf '{"message":"Branch not found"}'; printf 'gh: Not Found (HTTP 404)\n' >&2; exit 1; }
    if [ -f "$S/protection.json" ] || [ -f "$S/announce_protected" ]; then protected=true; else protected=false; fi
    printf '{"name":"%s","commit":{"sha":"%s"},"protected":%s}' "$b" "$head" "$protected" ;;
  "GET "*/pulls\?*)
    # Lookup by base and owner:head; an array, empty when there is none.
    if [ -f "$S/down" ]; then printf 'gh: connection refused\n' >&2; exit 1; fi
    q="${path#*\?}"; base="${q#*base=}"; base="${base%%&*}"; headq="${q#*head=}"; headq="${headq%%&*}"; headq="${headq#*:}"
    if [ -f "$S/pr_exists" ] && [ "$base" = "$(cat "$S/pr_base")" ] && [ "$headq" = "$(cat "$S/pr_branch")" ]; then printf '['; pr_json; printf ']'; else printf '[]'; fi ;;
  "GET "*/pulls/*)
    if [ ! -f "$S/pr_exists" ]; then printf '{"message":"Not Found"}'; printf 'gh: Not Found (HTTP 404)\n' >&2; exit 1; fi
    pr_json ;;
  "POST "*/pulls)
    printf x >> "$S/creates"
    if [ -f "$S/down" ]; then printf 'gh: connection refused\n' >&2; exit 1; fi
    if [ -f "$S/pr_exists" ]; then printf '{"message":"A pull request already exists"}'; printf 'gh: Validation Failed (HTTP 422)\n' >&2; exit 1; fi
    headb="${body#*\"head\":\"}"; headb="${headb%%\"*}"; base="${body#*\"base\":\"}"; base="${base%%\"*}"
    title="${body#*\"title\":\"}"; title="${title%%\"*}"; printf '%s' "$title" > "$S/pr_title"
    printf '%s' "$headb" > "$S/pr_branch"; printf '%s' "$base" > "$S/pr_base"; printf open > "$S/pr_state"; : > "$S/pr_exists"
    if [ -f "$S/lose_create" ]; then exit 1; fi
    pr_json ;;
  "PATCH "*/pulls/*)
    printf x >> "$S/updates"
    if [ -f "$S/down" ]; then printf 'gh: connection refused\n' >&2; exit 1; fi
    title="${body#*\"title\":\"}"; title="${title%%\"*}"; printf '%s' "$title" > "$S/pr_title"
    pr_json ;;
  "PUT "*/pulls/*/merge)
    printf x >> "$S/posts"
    if [ -f "$S/lose_unmerged" ]; then exit 1; fi
    if [ -f "$S/fail_merge" ]; then printf '{"message":"Required status check \"canary\" is expected."}'; printf 'gh: Required status check (HTTP 405)\n' >&2; exit 1; fi
    if [ "$(cat "$S/pr_state")" = merged ]; then printf '{"message":"Pull Request is not mergeable"}'; printf 'gh: not mergeable (HTTP 405)\n' >&2; exit 1; fi
    want="${body#*\"sha\":\"}"; want="${want%%\"*}"
    if [ "$want" != "$(cat "$S/pr_head")" ]; then printf '{"message":"Head branch was modified. Review and try the merge again."}'; printf 'gh: Head branch was modified (HTTP 409)\n' >&2; exit 1; fi
    base="$(cat "$S/pr_base")"; cand="$(cat "$S/pr_head")"
    merge="$(git -C "$R" commit-tree "$cand^{tree}" -p "refs/heads/$base" -p "$cand" -m "Merge pull request #3")"
    git -C "$R" update-ref "refs/heads/$base" "$merge"
    printf merged > "$S/pr_state"; printf '%s' "$merge" > "$S/merge_sha"
    if [ -f "$S/drift_after_merge" ]; then cp "$S/drift_after_merge" "$S/protection.json"; fi
    if [ -f "$S/lose_merge" ]; then exit 1; fi
    printf '{"sha":"%s","merged":true,"message":"Pull Request successfully merged"}' "$merge" ;;
  *) printf '{"message":"Not Found"}'; printf 'gh: Not Found (HTTP 404)\n' >&2; exit 1 ;;
esac
exit 0
"#;

pub(crate) const GH_PROTECTION: &str = r#"{"url":"https://api.github.com/repos/yesme/canary/branches/main/protection","required_status_checks":{"url":"u","strict":false,"contexts":["canary"],"contexts_url":"u","checks":[{"context":"canary","app_id":null}]},"required_pull_request_reviews":{"url":"u","dismiss_stale_reviews":false,"require_code_owner_reviews":false,"required_approving_review_count":0},"enforce_admins":{"url":"u","enabled":true},"required_conversation_resolution":{"enabled":false},"allow_force_pushes":{"enabled":false},"allow_deletions":{"enabled":false}}"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Gitea,
    GitHub,
}

pub(crate) struct Platform {
    pub(crate) kind: Kind,
    pub(crate) hosted: Hosted,
    pub(crate) state: PathBuf,
    pub(crate) git_dir: PathBuf,
    /// Base of the published revision (the target's head at the start).
    pub(crate) base: String,
    /// The published candidate commit (review request #7's head).
    pub(crate) candidate: String,
    /// The admitted result tree: the candidate's tree.
    pub(crate) tree: String,
}

impl Platform {
    pub(crate) fn new(dir: &Path) -> Self {
        Self::of(Kind::Gitea, dir)
    }
    pub(crate) fn github(dir: &Path) -> Self {
        Self::of(Kind::GitHub, dir)
    }
    pub(crate) fn of(kind: Kind, dir: &Path) -> Self {
        let tea = dir.join("tea");
        let state = dir.join("tea.state");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(
            &tea,
            match kind {
                Kind::Gitea => TEA,
                Kind::GitHub => GH,
            },
        )
        .unwrap();
        std::fs::set_permissions(&tea, std::fs::Permissions::from_mode(0o700)).unwrap();
        let git_dir = dir.join("platform.git");
        let output = Command::new("git")
            .args(["init", "--bare", "--quiet"])
            .arg(&git_dir)
            .output()
            .unwrap();
        assert!(output.status.success());
        let mut platform = Self {
            kind,
            hosted: Hosted::fixture(
                tea,
                "http://127.0.0.1:3000".into(),
                "admin".into(),
                "fixture-only".into(),
            ),
            state,
            git_dir,
            base: String::new(),
            candidate: String::new(),
            tree: String::new(),
        };
        let base_tree = platform.tree_with("a", "base\n");
        platform.base = platform.git(&["commit-tree", &base_tree, "-m", "base"]);
        platform.tree = platform.tree_with("a", "candidate\n");
        let tree = platform.tree.clone();
        let base = platform.base.clone();
        platform.candidate = platform.git(&["commit-tree", &tree, "-p", &base, "-m", "candidate"]);
        platform.git(&["update-ref", "refs/heads/main", &base]);
        let candidate = platform.candidate.clone();
        platform.git(&["update-ref", "refs/heads/cs-1", &candidate]);
        platform.set("pr_state", "open");
        platform.set("pr_exists", "");
        platform.set("pr_head", &candidate);
        platform.set("pr_base", "main");
        platform.set("rule_name", "main");
        platform.set("rule_path", "main");
        platform.set(
            "protection.json",
            match kind {
                Kind::Gitea => PROTECTION,
                Kind::GitHub => GH_PROTECTION,
            },
        );
        platform
    }
    pub(crate) fn full_name(&self) -> &'static str {
        match self.kind {
            Kind::Gitea => "admin/example",
            Kind::GitHub => "yesme/canary",
        }
    }
    pub(crate) fn connection(&self) -> Connection {
        let script = self.state.parent().unwrap().join("tea");
        match self.kind {
            Kind::Gitea => Connection::Gitea(Hosted::fixture(
                script,
                self.hosted.url.clone(),
                self.hosted.username.clone(),
                self.hosted.token.clone(),
            )),
            Kind::GitHub => Connection::GitHub(github::GitHub::fixture(script, "github.com")),
        }
    }
    pub(crate) fn observe(&self) -> repo::Result<super::target::Target> {
        self.connection()
            .observe(self.full_name(), "refs/heads/main")
    }
    pub(crate) fn provider_ref(&self) -> String {
        match self.kind {
            Kind::Gitea => "http://127.0.0.1:3000/admin/example".into(),
            Kind::GitHub => "github.com/yesme/canary".into(),
        }
    }
    pub(crate) fn continuity(&self) -> Value {
        match self.kind {
            Kind::Gitea => json!({"instance": "http://127.0.0.1:3000", "stable_id": "7"}),
            Kind::GitHub => json!({"instance": "github.com", "stable_id": "77"}),
        }
    }
    pub(crate) fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.git_dir)
            .args(args)
            .env("GIT_AUTHOR_NAME", "gitea")
            .env("GIT_AUTHOR_EMAIL", "gitea@localhost")
            .env("GIT_COMMITTER_NAME", "gitea")
            .env("GIT_COMMITTER_EMAIL", "gitea@localhost")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }
    pub(crate) fn tree_with(&self, name: &str, content: &str) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.git_dir)
            .args(["hash-object", "-w", "--stdin"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child
                    .stdin
                    .take()
                    .unwrap()
                    .write_all(content.as_bytes())
                    .unwrap();
                child.wait_with_output()
            })
            .unwrap();
        let blob = String::from_utf8(output.stdout).unwrap().trim().to_owned();
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.git_dir)
            .arg("mktree")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child
                    .stdin
                    .take()
                    .unwrap()
                    .write_all(format!("100644 blob {blob}\t{name}\n").as_bytes())
                    .unwrap();
                child.wait_with_output()
            })
            .unwrap();
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }
    /// The platform merges review request #7 by itself (a human on the native UI, or a
    /// request whose response was lost): a real merge commit, the request reads as merged.
    pub(crate) fn merge_natively(&self, fast_forward: bool) -> String {
        let merge = if fast_forward {
            self.candidate.clone()
        } else {
            self.git(&[
                "commit-tree",
                &format!("{}^{{tree}}", self.candidate),
                "-p",
                "refs/heads/main",
                "-p",
                &self.candidate,
                "-m",
                "merge #7",
            ])
        };
        self.git(&["update-ref", "refs/heads/main", &merge]);
        self.set("pr_state", "merged");
        self.set("merge_sha", &merge);
        merge
    }
    pub(crate) fn set(&self, name: &str, value: &str) {
        std::fs::write(self.state.join(name), value).unwrap();
    }
    pub(crate) fn unset(&self, name: &str) {
        let _ = std::fs::remove_file(self.state.join(name));
    }
    pub(crate) fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.state.join(name)).unwrap_or_default()
    }
    pub(crate) fn head(&self) -> String {
        self.git(&["rev-parse", "refs/heads/main"])
    }
    pub(crate) fn posts(&self) -> usize {
        self.read("posts").len()
    }
    pub(crate) fn creates(&self) -> usize {
        self.read("creates").len()
    }
    pub(crate) fn updates(&self) -> usize {
        self.read("updates").len()
    }
    /// The platform starts without any review request; publishing has to create one.
    pub(crate) fn without_review_request(&self) {
        self.unset("pr_exists");
        self.unset("pr_branch");
    }
    /// A working clone of the platform's Git on "this machine": where a Repo registered from
    /// a local repository keeps its objects and where publishing pushes from.
    pub(crate) fn local_clone(&self) -> PathBuf {
        let local = self.state.parent().unwrap().join("local");
        if !local.join(".git").exists() {
            let output = Command::new("git")
                .args(["clone", "--quiet"])
                .arg(&self.git_dir)
                .arg(&local)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            // Keep the candidate reachable locally, then drop the remote: a registration from
            // a plain local repository, with the platform bound separately.
            for args in [
                vec!["branch", "cs-1", "origin/cs-1"],
                vec!["remote", "remove", "origin"],
                vec!["config", "user.name", "HCTL2 Test"],
                vec!["config", "user.email", "hctl2@example.invalid"],
                vec!["config", "commit.gpgsign", "false"],
            ] {
                let output = Command::new("git")
                    .arg("-C")
                    .arg(&local)
                    .args(&args)
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        local
    }
    pub(crate) fn readback_root(&self) -> PathBuf {
        self.state.parent().unwrap().join("readback")
    }
    pub(crate) fn connect(
        &self,
    ) -> impl FnMut(&repo::Registration) -> repo::Result<Connection> + '_ {
        move |_| Ok(self.connection())
    }
    pub(crate) fn drive(&self, shared: &Shared, repo_id: &str, id: &str) -> repo::Result<()> {
        drive_with(
            shared,
            repo_id,
            id,
            &self.readback_root(),
            &mut self.connect(),
        )
    }
}

/// A hosted Repo taken to active with capabilities declared verified, one admitted revision
/// published as review request #7 whose head is the platform's candidate commit.
pub(crate) fn hosted_repo(store: &mut Store, platform: &Platform) -> String {
    use repo::{FinishChoice, PlatformObservation, confirm_delivery, confirm_platform, finish};
    if platform.kind == Kind::GitHub {
        return github_repo(store, platform);
    }
    let prepared = repo::prepare(
        repo::Register {
            name: "example".into(),
            origin: repo::Origin::Local,
            platform: Some(repo::Platform::Local),
            instance: None,
            platform_repo_id: None,
            platform_path: Some("example".into()),
            local: None,
            remote_evidence: None,
            default_source: None,
        },
        None,
    )
    .unwrap();
    let reg = repo::admit(store, &actor(), "register", "register", prepared).unwrap();
    store
        .begin_effect(
            store.generation(),
            &repo::effect_id(&reg.repo_id, "platform"),
        )
        .unwrap();
    let reg = confirm_platform(
        store,
        &reg.repo_id,
        PlatformObservation {
            instance: "http://127.0.0.1:3000".into(),
            stable_id: "7".into(),
            full_name: "admin/example".into(),
            // The platform's Git, reached the way a clone URL is: this one is a path.
            clone_url: platform.git_dir.to_string_lossy().into_owned(),
            account_id: "1".into(),
            has_issues: true,
            can_write_issues: true,
            credential_ref: String::new(),
        },
    )
    .unwrap();
    store
        .begin_effect(
            store.generation(),
            &repo::effect_id(&reg.repo_id, "delivery"),
        )
        .unwrap();
    let reg = confirm_delivery(store, &reg.repo_id).unwrap();
    finish(
        store,
        &actor(),
        &reg.repo_id,
        reg.version,
        FinishChoice::Confirm("7"),
        "finish",
        "finish",
    )
    .unwrap();
    let repo_id = reg.repo_id;
    // Declared capabilities as the verified adapter will record them.
    {
        use store::{Command, Expected, Record, RecordData, Version};
        let binding = repo::binding(&repo_id);
        let current = store.get(&binding.key).unwrap().unwrap();
        let RecordData::Value { mut value } = current.data else {
            panic!("binding")
        };
        value["capabilities"]["remote_merge"] = json!(true);
        value["capabilities"]["protection_readback"] = json!(true);
        let mut scoped = actor().0;
        scoped.permission_scope.push(Scope::Repo(repo_id.clone()));
        let scoped = TrustedActor(scoped);
        let cmd = Command {
            command_id: "declare".into(),
            idempotency_key: "declare".into(),
            actor: scoped.0.clone(),
            target: binding.key.clone(),
            expected: Expected::Exact(Version::State(current.version)),
            binding: binding.clone(),
            input_digest: Command::digest_input("test.declare", &value).unwrap(),
            operation: "test.declare".into(),
            input: value.clone(),
        };
        store
            .submit(store.generation(), &scoped, &cmd, None, |tx| {
                tx.put(&Record {
                    key: binding.key.clone(),
                    version: current.version + 1,
                    revision_digest: foundation::canonical_json_sha256(&value).unwrap(),
                    data: RecordData::Value {
                        value: value.clone(),
                    },
                    sources: Vec::new(),
                    materials: Vec::new(),
                })?;
                Ok(json!({}))
            })
            .unwrap();
    }
    domain::admit_revision_seam(
        store,
        &actor(),
        &repo_id,
        &domain::AdmittedRevision {
            change_set_revision_id: "rev-1".into(),
            change_set_id: "cs-1".into(),
            parent_revision_id: None,
            base_commit_sha: platform.base.clone(),
            result_tree_sha: platform.tree.clone(),
            producer_ref: json!({"kind":"human_command","command_id":"seal"}),
            review_subject_digest: "d".repeat(64),
        },
    )
    .unwrap();
    domain::admit_platform_binding_seam(
        store,
        &actor(),
        &repo_id,
        "rev-1",
        &domain::ReviewRequestRef {
            index: 7,
            platform_commit_sha: platform.candidate.clone(),
        },
    )
    .unwrap();
    repo_id
}

/// An external GitHub Repo taken to active (its binding declares merge and protection readback
/// verified), one admitted revision published as pull request #3 at the platform's candidate.
pub(crate) fn github_repo(store: &mut Store, platform: &Platform) -> String {
    github_repo_with(
        store,
        repo::PlatformObservation {
            instance: "github.com".into(),
            stable_id: "77".into(),
            full_name: "yesme/canary".into(),
            clone_url: platform.git_dir.to_string_lossy().into_owned(),
            account_id: "1".into(),
            has_issues: true,
            can_write_issues: true,
            credential_ref: String::new(),
        },
        &platform.base,
        &platform.tree,
        3,
        &platform.candidate,
    )
}

pub(crate) fn github_repo_with(
    store: &mut Store,
    observed: repo::PlatformObservation,
    base: &str,
    tree: &str,
    number: u64,
    head: &str,
) -> String {
    use repo::{FinishChoice, confirm_platform, finish};
    let stable = observed.stable_id.clone();
    let prepared = repo::prepare(
        repo::Register {
            name: "canary".into(),
            origin: repo::Origin::External,
            platform: Some(repo::Platform::Github),
            instance: Some("github.com".into()),
            platform_repo_id: Some(stable.clone()),
            platform_path: Some(observed.full_name.clone()),
            local: None,
            remote_evidence: None,
            default_source: None,
        },
        None,
    )
    .unwrap();
    let reg = repo::admit(store, &actor(), "register-gh", "register-gh", prepared).unwrap();
    store
        .begin_effect(
            store.generation(),
            &repo::effect_id(&reg.repo_id, "platform"),
        )
        .unwrap();
    let reg = confirm_platform(store, &reg.repo_id, observed).unwrap();
    if reg.lifecycle != repo::Lifecycle::Active {
        finish(
            store,
            &actor(),
            &reg.repo_id,
            reg.version,
            FinishChoice::Confirm(&stable),
            "finish-gh",
            "finish-gh",
        )
        .unwrap();
    }
    let repo_id = reg.repo_id;
    domain::admit_revision_seam(
        store,
        &actor(),
        &repo_id,
        &domain::AdmittedRevision {
            change_set_revision_id: "rev-1".into(),
            change_set_id: "cs-1".into(),
            parent_revision_id: None,
            base_commit_sha: base.into(),
            result_tree_sha: tree.into(),
            producer_ref: json!({"kind":"human_command","command_id":"seal"}),
            review_subject_digest: "d".repeat(64),
        },
    )
    .unwrap();
    domain::admit_platform_binding_seam(
        store,
        &actor(),
        &repo_id,
        "rev-1",
        &domain::ReviewRequestRef {
            index: number,
            platform_commit_sha: head.into(),
        },
    )
    .unwrap();
    repo_id
}

pub(crate) fn temp(name: &str) -> Temp {
    let dir = std::env::temp_dir().join(format!("hctl2-gitea-integ-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    Temp(dir)
}

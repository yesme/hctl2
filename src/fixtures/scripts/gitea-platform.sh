#!/bin/sh
# $0.state/: protection.json (absent = unprotected), rule_name (effective rule the branch
# record names), rule_path (the one path segment the rule answers under), pr_state, pr_head,
# pr_base, merge_sha, posts; switches: fail_merge (405), refuse_update (403 on PATCH), lose_merge (merge, then die without
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
    if [ -f "$S/refuse_update" ]; then printf 'HTTP/1.1 403 Forbidden\n' >&2; printf '{"message":"user is not allowed to edit this pull request"}'; exit 1; fi
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

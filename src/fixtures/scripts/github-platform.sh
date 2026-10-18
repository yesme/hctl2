#!/bin/sh
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
    if [ -f "$S/refuse_update" ]; then printf '{"message":"Resource not accessible by integration"}'; printf 'gh: Resource not accessible by integration (HTTP 403)\n' >&2; exit 1; fi
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

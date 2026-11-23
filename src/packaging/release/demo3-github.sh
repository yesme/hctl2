#!/usr/bin/env bash
# Demo 3, GitHub path: the same chain against the public sandbox repository
# yesme/hctl2-canary, whose protected main is what demo 3 eventually merges
# into.
#
# The credential arrives as a runner secret and stays in the environment that
# control imports into its own secret backend; this script never prints it. Without
# one there is nothing to reach GitHub with, so the path skips and says so on
# stdout, where the CI log keeps it.

set -euo pipefail

if [[ -z "${HCTL2_DEMO3_GITHUB_TOKEN:-}" ]]; then
    printf '%s\n' \
        'demo3: SKIP GitHub path: no runner secret reached HCTL2_DEMO3_GITHUB_TOKEN, so yesme/hctl2-canary is not reachable'
    exit 0
fi

: "${HCTL2_DEMO3_COMMON:?Buck must provide HCTL2_DEMO3_COMMON}"
# shellcheck source=demo3.sh
source "$HCTL2_DEMO3_COMMON"

readonly DEMO3_GITHUB_INSTANCE="github.com"
readonly DEMO3_GITHUB_REPO="yesme/hctl2-canary"
# The repository's numeric id, declared here rather than discovered: an external
# platform binding is refused unless the identity the human declared is the one
# the readback returns, so a renamed or swapped repository cannot bind silently.
readonly DEMO3_GITHUB_REPO_ID="1407796416"

demo3_prepare github
[[ -x "$DEMO3_PAYLOAD/libexec/hctl2/gh" ]] ||
    die "the package carries no gh: $DEMO3_PAYLOAD/libexec/hctl2/gh"
note "GitHub path drives the packaged gh at $DEMO3_PAYLOAD/libexec/hctl2/gh"
# Use the public credential import command before any platform operation.
printf '%s' "$HCTL2_DEMO3_GITHUB_TOKEN" |
    hctl2 github login --with-token >/dev/null
DEMO3_GITHUB_FACTS_TOKEN="$HCTL2_DEMO3_GITHUB_TOKEN"
unset HCTL2_DEMO3_GITHUB_TOKEN

demo3_control_start
demo3_wait_service tuwunel
# Nothing on this path consumes the hosted Gitea, and it must stay that way.
demo3_assert_service_down gitea

# The write segment needs a working copy of the sandbox repository: the Agency
# writes an isolated copy of it, and the baseline commit is what the lease pins.
# The repository is public, so the clone needs no credential.
source_repo="$DEMO3_ROOT/canary-source"
git clone --quiet --branch main --single-branch \
    "https://github.com/$DEMO3_GITHUB_REPO.git" "$source_repo" ||
    die "cannot clone https://github.com/$DEMO3_GITHUB_REPO.git"
git -C "$source_repo" config user.name 'Demo 3 Script'
git -C "$source_repo" config user.email 'demo3@example.invalid'
note "cloned $DEMO3_GITHUB_REPO at $(git -C "$source_repo" rev-parse HEAD)"

"$HCTL2_JQ" -n \
    --arg instance "$DEMO3_GITHUB_INSTANCE" --arg path "$DEMO3_GITHUB_REPO" \
    --arg repo_id "$DEMO3_GITHUB_REPO_ID" --arg source "$source_repo" '{
    name: "hctl2-canary",
    origin: "external",
    platform: "github",
    instance: $instance,
    platform_repo_id: $repo_id,
    platform_path: $path,
    default_source: "github_issues",
    local: {machine: "control", path: $source}
}' >"$DEMO3_ROOT/register.json"
demo3_register_repo "$DEMO3_ROOT/register.json" demo3-github active

demo3_project_create "$DEMO3_REPO" "Demo 3 GitHub"
demo3_agency_up
demo3_roster_ready "$DEMO3_PROJECT" "$DEMO3_ROOM"
demo3_task_with_contract "$DEMO3_REPO" github_issues
demo3_read_only_dispatch "$DEMO3_PROJECT" "$DEMO3_ROOM"

# The same five steps as the Gitea path; the only differences are the merge
# strategy (GitHub merges the pull request, it does not fast-forward) and the
# independent readback through the packaged gh.
write_path="demo3/script-$(date +%s).txt"
demo3_write_profile "$DEMO3_PROJECT" "$DEMO3_ROOM"
demo3_write_type_dispatch "$DEMO3_PROJECT" "$DEMO3_ROOM" "$source_repo" "$write_path"
demo3_seal "$DEMO3_PROJECT" "$DEMO3_INVOCATION"
demo3_publish "$DEMO3_REPO"
demo3_merge "$DEMO3_REPO"
demo3_complete "$DEMO3_PROJECT" "$DEMO3_TASK"
demo3_assert_previews
note "GitHub chain reached Task Completion Receipt through canary PR #$DEMO3_PR_INDEX"

# The card this run created on the sandbox goes back out, so repeated runs do
# not pile issues up on a repository other people can see.
demo3_delete_card "$DEMO3_PROJECT" "$DEMO3_TASK"
demo3_assert_service_down gitea
hctl2 stop >/dev/null
note "control stopped"
demo3_cleanup
trap - EXIT
note "GitHub path complete against $DEMO3_GITHUB_REPO"

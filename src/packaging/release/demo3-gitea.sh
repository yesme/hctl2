#!/usr/bin/env bash
# Demo 3, local path: the bundled Gitea really comes up, an experiment repo is
# registered into it, a Project and a Task carrying a contract are created, and
# one read-only dispatch is answered in the Room.
#
# Registration is Gitea's first consumer, so the assertion that matters here is
# the transition: down and unconsumed after `start`, available and consumed
# after the registration is delivered.

set -euo pipefail

: "${HCTL2_DEMO3_COMMON:?Buck must provide HCTL2_DEMO3_COMMON}"
# shellcheck source=demo3.sh
source "$HCTL2_DEMO3_COMMON"

demo3_prepare gitea
demo3_control_start
demo3_wait_service tuwunel
demo3_assert_service_down gitea

# A real working copy with one commit: initial delivery pushes exactly the
# selected refs and invents nothing.
source_repo="$DEMO3_ROOT/source"
mkdir -p "$source_repo"
git -C "$source_repo" init -b main >/dev/null
git -C "$source_repo" -c user.name='Demo 3' -c user.email=demo3@example.invalid \
    commit --allow-empty -m 'demo 3 baseline' >/dev/null
"$HCTL2_JQ" -n --arg path "$source_repo" '{
    name: "demo3-gitea",
    origin: "local",
    platform_path: "demo3-gitea",
    local: {machine: "control", path: $path},
    default_source: "gitea_issues"
}' >"$DEMO3_ROOT/register.json"
demo3_register_repo "$DEMO3_ROOT/register.json" demo3-gitea pending
demo3_wait_service gitea

demo3_project_create "$DEMO3_REPO" "Demo 3 Gitea"
demo3_agency_up
demo3_roster_ready "$DEMO3_PROJECT" "$DEMO3_ROOM"
demo3_task_with_contract "$DEMO3_REPO" gitea_issues
demo3_read_only_dispatch "$DEMO3_PROJECT" "$DEMO3_ROOM"

demo3_write_type_dispatch "$DEMO3_PROJECT" "$DEMO3_ROOM"
demo3_seal "$DEMO3_PROJECT" ""
demo3_publish "$DEMO3_REPO" ""
demo3_merge "$DEMO3_REPO" ""
demo3_complete "$DEMO3_PROJECT" "$DEMO3_TASK"

demo3_delete_card "$DEMO3_PROJECT" "$DEMO3_TASK"
hctl2 stop >/dev/null
note "control stopped"
demo3_cleanup
trap - EXIT
note "Gitea path complete"

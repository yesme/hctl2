#!/usr/bin/env bash
# Demo 3 driver library: one control plane, the packaged services and a script
# Agency, reached only through `hctl2 --json`.
#
# Every step here is a command line and every result is read back out of the
# JSON that command printed. Nothing reaches into the control store, into the
# services' data directories or into a crate, so a step the CLI cannot express
# is a step this demo does not perform.
#
# Sourced by demo3-gitea.sh and demo3-github.sh; not a test on its own.

set -Eeuo pipefail

: "${HCTL2_CLI_BIN:?Buck must provide HCTL2_CLI_BIN}"
# Not called directly: `hctl2 start` looks for hctl2-control beside its own
# executable and falls back to this, and the dependency payload carries no
# first-party binaries to find.
: "${HCTL2_CONTROL_BIN:?Buck must provide HCTL2_CONTROL_BIN}"
: "${HCTL2_AGENCY_BIN:?Buck must provide HCTL2_AGENCY_BIN}"
: "${HCTL2_JQ:?Buck must provide HCTL2_JQ}"
: "${HCTL2_TEST_DEPENDENCY_PACKAGE:?Buck must provide HCTL2_TEST_DEPENDENCY_PACKAGE}"
: "${HCTL2_ZSTD_ROOT:?Buck must provide HCTL2_ZSTD_ROOT}"

# The exact answer the script execution body returns. Both paths assert on it,
# so a dispatch that answered with anything else is a failure, not a variant.
readonly DEMO3_ANSWER="DEMO3_READ_ONLY_OK"

DEMO3_ROOT=""
DEMO3_PAYLOAD=""
DEMO3_AGENCY_ROOT=""
DEMO3_AGENCY_PID=""
DEMO3_PORTS=""
DEMO3_PROFILE=""
DEMO3_REPO=""
DEMO3_PLATFORM_ID=""
DEMO3_PROJECT=""
DEMO3_ROOM=""
DEMO3_SOURCE=""
DEMO3_TASK=""

note() { printf 'demo3: %s\n' "$*"; }
demo3_dump_agency_err() {
    # The execution body's own streams are the Agency's to discard, so the
    # Agency's stderr is the only trace a failure inside it leaves. It carries
    # catalog diagnostics; the pairing material goes to stdout, which is
    # discarded instead of captured.
    if [[ -n "$DEMO3_ROOT" && -s "$DEMO3_ROOT/agency.err" ]]; then
        printf 'demo3 --- agency.err ---\n' >&2
        tail -n 30 "$DEMO3_ROOT/agency.err" >&2
    fi
}
die() {
    printf 'demo3 FAILED: %s\n' "$*" >&2
    demo3_dump_agency_err
    exit 1
}
# Almost every step here is a command substitution over `hctl2 --json`, so a
# refusal prints its diagnostics into a variable and `set -e` aborts with
# nothing on either stream -- a runner log then shows only how far the notes
# got. Name the line that aborted. The JSON it captured is still lost, so the
# steps whose answer matters read their own exit status as well.
demo3_unhandled() { # <line> <status> <command>
    printf 'demo3 FAILED: unhandled exit %s at line %s: %s\n' "$2" "$1" "$3" >&2
    demo3_dump_agency_err
}
trap 'demo3_unhandled "$LINENO" "$?" "$BASH_COMMAND"' ERR

# One CLI call against this demo's own control root. The caller decides whether
# a non-zero exit is a failure or an expected refusal.
hctl2() {
    HCTL2_INSTALL_ROOT="$DEMO3_PAYLOAD" \
        "$HCTL2_CLI_BIN" --json --root "$DEMO3_ROOT" "$@"
}

json_get() { "$HCTL2_JQ" -c "$1" <<<"$2"; }
json_str() { "$HCTL2_JQ" -er "$1" <<<"$2"; }
assert_json() { # <filter> <json> <what went wrong>
    "$HCTL2_JQ" -e "$1" <<<"$2" >/dev/null || die "$3: $2"
}

# The product's write convention: the same command twice, first without a token
# so it only previews, then with the exact token so it submits. There is no
# one-shot form, so the preview is never skipped by accident.
two_phase() { # <key> <command...> (no --key, no --preview-token)
    local key="$1"
    shift
    local preview token
    preview="$(hctl2 "$@" --key "$key")" || die "preview of $key failed: $preview"
    token="$(json_str '.preview_token' "$preview")" ||
        die "preview of $key returned no token: $preview"
    hctl2 "$@" --key "$key" --preview-token "$token"
}

demo3_sha256() { # stdin -> lowercase hex
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 | awk '{print $1}'
    else
        die "required SHA-256 tool not found: sha256sum or shasum"
    fi
}

# Bash cannot bind a socket, so a free port is found by connecting: a port that
# accepts a connection is taken. The range stays below both platforms'
# ephemeral ranges so an outgoing socket is unlikely to claim it in between.
demo3_free_port() {
    local attempt port
    for attempt in $(seq 1 100); do
        port=$((10000 + RANDOM % 10000))
        case " $DEMO3_PORTS " in *" $port "*) continue ;; esac
        if (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then
            continue
        fi
        printf '%s\n' "$port"
        return 0
    done
    die "no free service port after 100 attempts"
}

# versions.sh declares every packaged service port `readonly`, so the only way
# to move them is to rewrite that file; apps/cli/tests/room.rs does the same,
# for the same reason: these tests share a host with whatever else is running.
demo3_isolate_ports() {
    local versions="$DEMO3_PAYLOAD/lib/hctl2/services/versions.sh"
    [[ -f "$versions" ]] || die "packaged versions.sh is missing: $versions"
    local line name port rewritten=""
    while IFS= read -r line; do
        if [[ "$line" == 'readonly '*'_PORT="'*'"' ]]; then
            name="${line#readonly }"
            name="${name%%=*}"
            port="$(demo3_free_port)"
            DEMO3_PORTS="$DEMO3_PORTS $port"
            line="readonly $name=\"$port\""
        fi
        rewritten+="$line"$'\n'
    done <"$versions"
    printf '%s' "$rewritten" >"$versions"
    grep -Eq '^readonly TUWUNEL_PORT="[0-9]+"$' "$versions" ||
        die "port isolation lost TUWUNEL_PORT"
}

demo3_cleanup() {
    # Teardown is best effort; its own failures must not masquerade as the
    # reason the run stopped.
    trap - ERR
    if [[ -n "$DEMO3_AGENCY_PID" ]]; then
        kill "$DEMO3_AGENCY_PID" 2>/dev/null || true
        local _
        for _ in $(seq 1 100); do
            kill -0 "$DEMO3_AGENCY_PID" 2>/dev/null || break
            sleep 0.05
        done
        kill -9 "$DEMO3_AGENCY_PID" 2>/dev/null || true
        wait "$DEMO3_AGENCY_PID" 2>/dev/null || true
    fi
    if [[ -n "$DEMO3_ROOT" ]]; then
        hctl2 stop >/dev/null 2>&1 || true
        if [[ -x "${DEMO3_PAYLOAD:-}/bin/hctl2-services" ]]; then
            HCTL2_STATE_ROOT="$DEMO3_ROOT/services" \
                "$DEMO3_PAYLOAD/bin/hctl2-services" stop >/dev/null 2>&1 || true
        fi
        find "${DEMO3_ROOT:?}" -depth -delete
    fi
}

demo3_prepare() { # <name>
    local name="$1"
    DEMO3_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-demo3-$name.XXXXXX")"
    case "$DEMO3_ROOT" in
        /*/hctl2-demo3-*) ;;
        *) die "unsafe demo3 directory: $DEMO3_ROOT" ;;
    esac
    trap demo3_cleanup EXIT
    local archive services_bin
    archive="$(find "$HCTL2_TEST_DEPENDENCY_PACKAGE" -type f -name '*.tar.zst' \
        ! -name '*-sources.tar.zst' | head -n 1)"
    [[ -n "$archive" ]] || die "dependency package has no runtime archive"
    mkdir -p "$DEMO3_ROOT/install"
    # The archive is a zstd frame: decode it with the pinned tool instead of
    # relying on whichever decoder the host tar happens to have. Decoded into
    # tar rather than into a file: the payload unpacks to 635 MB and the tar it
    # comes from is the same again, and the complete package job runs several of
    # these payload tests at once on a runner whose disk is bounded, so writing
    # the tar out would more than double what this one test holds at its peak.
    "$HCTL2_ZSTD_ROOT/bin/zstd" -dc "$archive" | tar -xf - -C "$DEMO3_ROOT/install"
    services_bin="$(find "$DEMO3_ROOT/install" -type f -name hctl2-services \
        -path '*/bin/*' | head -n 1)"
    [[ -n "$services_bin" ]] || die "dependency package has no bin/hctl2-services"
    DEMO3_PAYLOAD="${services_bin%/bin/hctl2-services}"
    demo3_isolate_ports
    note "$name path: payload at $DEMO3_PAYLOAD"
}

demo3_control_start() {
    # Disposable credentials on purpose: the packaged default stays the system
    # keyring, which a CI runner has no reason to have.
    hctl2 start --secret-backend user-file >/dev/null
    local status
    status="$(hctl2 status)"
    assert_json '.ready == true' "$status" "control did not become ready"
    note "control $(json_str '.control_id' "$status") is ready"
}

demo3_wait_service() { # <name>
    local name="$1" attempt snapshot=""
    for attempt in $(seq 1 60); do
        snapshot="$(hctl2 services status 2>/dev/null || true)"
        if printf '%s\n' "$snapshot" | "$HCTL2_JQ" -e --arg n "$name" \
            '.hosted[] | select(.name == $n) | .available and .consumed' \
            >/dev/null 2>&1; then
            note "$name is available and consumed"
            return 0
        fi
        sleep 1
    done
    die "$name did not become available: $snapshot"
}

demo3_assert_service_down() { # <name>
    local name="$1" snapshot="" status=0
    snapshot="$(hctl2 services status)" || status=$?
    [[ "$status" -eq 0 ]] ||
        die "\`hctl2 services status\` exited $status while checking $name: $snapshot"
    printf '%s\n' "$snapshot" | "$HCTL2_JQ" -e --arg n "$name" \
        '.hosted[] | select(.name == $n) | (.available | not) and (.consumed | not)' \
        >/dev/null || die "$name was already up before anything consumed it: $snapshot"
}

# The execution body is a script: it takes the assembly on stdin and answers
# with one result line on stdout, which is the whole answer this demo accepts.
# It is handed to the shell inline rather than as a file, and its first
# instruction is the read, because the Agency bounds the delivery of the
# assembly with a write timeout that starts when the body is spawned.
demo3_agency_up() {
    DEMO3_AGENCY_ROOT="$DEMO3_ROOT/independent-agency"
    local config="$DEMO3_ROOT/script.json" body
    body=$(cat <<DEMO3_EXECUTOR
read -r assembly || exit 1
printf '%s\n' '{"type":"result","schema":"adapter.stdout.v1","output":"$DEMO3_ANSWER"}'
DEMO3_EXECUTOR
)
    "$HCTL2_JQ" -n --arg body "$body" \
        '{program:"/bin/sh",arguments:["-c",$body,"demo3-executor"]}' >"$config"
    # Nothing of the Agency's stdout is kept: it answers `hctl2 agency pair`,
    # so it holds the pairing material. Its stderr is kept for `die` to show.
    # Neither may stay attached to the test's own output pipe, or the harness
    # would wait for this background child after the test had finished.
    "$HCTL2_AGENCY_BIN" --root "$DEMO3_AGENCY_ROOT" serve --script-config "$config" \
        >/dev/null 2>"$DEMO3_ROOT/agency.err" &
    DEMO3_AGENCY_PID=$!
    local attempt
    for attempt in $(seq 1 100); do
        if "$HCTL2_AGENCY_BIN" --root "$DEMO3_AGENCY_ROOT" status >/dev/null 2>&1; then
            note "script Agency is serving (pid $DEMO3_AGENCY_PID)"
            return 0
        fi
        sleep 0.05
    done
    die "script Agency did not start"
}

# Pair, accept one exact Profession, create a read-only Worker Profile and
# select the roster. Leaves DEMO3_PROFILE at the revision dispatch names.
demo3_roster_ready() { # <project> <room>
    local project="$1" room="$2"
    local binding catalog profession accepted_profession digest
    binding="$(hctl2 agency pair --binding-id local \
        --agency-root "$DEMO3_AGENCY_ROOT" --key demo3-pair)" ||
        die "Agency pairing failed: $binding"
    assert_json '.paired == true' "$binding" "Agency pairing did not report paired"
    catalog="$(hctl2 agency catalog local)"
    profession="$(json_get '.professions[0]' "$catalog")"
    [[ "$profession" != null ]] || die "Agency catalog has no Profession: $catalog"
    json_get '.reference' "$profession" >"$DEMO3_ROOT/profession.json"
    accepted_profession="$(hctl2 agency accept --binding-id local \
        --reference "$DEMO3_ROOT/profession.json" --key demo3-accept)" ||
        die "accepting the Profession failed: $accepted_profession"
    digest="$(json_str '.reference.digest' "$profession")"

    "$HCTL2_JQ" -n --argjson profession "$profession" '{
        id: "research",
        profile: {
            harness: $profession.harness,
            model: $profession.model,
            mode: "read_only",
            permissions: ["context.read"],
            environment: [],
            required_capabilities: {
                input: false, stop: false, event_cursor: false,
                input_provenance: false, managed_single_writer: false,
                secure_input: false, exact_attach: false,
                tool_execution_unmediated: false, isolation_effects: []
            },
            max_context_bytes: 65536
        }
    }' >"$DEMO3_ROOT/profile.json"
    DEMO3_PROFILE="$(two_phase demo3-profile profile create \
        --input "$DEMO3_ROOT/profile.json")"

    "$HCTL2_JQ" -n \
        --arg project "$project" --arg room "$room" --arg digest "$digest" \
        --argjson binding_ref \
            "$(json_get '{"key":.binding.key,"version":{"state":.binding.version}}' "$binding")" \
        --argjson profession_ref \
            "$(json_get '{"key":.profession.key,"version":{"state":.profession.version}}' \
                "$accepted_profession")" \
        --argjson profile_revision "$(json_get '.revision' "$DEMO3_PROFILE")" \
        --argjson version "$(demo3_project_version)" '{
        project_id: $project, project_version: $version, room_id: $room,
        topic_command_key: null, roster_version: null,
        selections: [{
            room_id: $room,
            selected_item: $profession_ref,
            profession: $profession_ref,
            profession_digest: $digest,
            agency: $binding_ref,
            required_skills: [], optional_skills: [],
            worker_profiles: [$profile_revision],
            responsibility: "research",
            permission: {allow: ["context.read"]},
            budget: {max_bytes: 65536},
            display_name: "Research",
            persona_tags: []
        }]
    }' >"$DEMO3_ROOT/select.json"
    two_phase demo3-select project select --input "$DEMO3_ROOT/select.json" \
        >/dev/null
    note "roster selected one read-only research Worker"
}

# <lifecycle-after-register> is what the path expects to see once the platform
# has been read back: an external platform declares its identity in the request
# and is bound by that readback, while a hosted one stays pending until a human
# says which repository the platform just created.
demo3_register_repo() { # <request-file> <key-prefix> <lifecycle-after-register>
    local input="$1" key="$2" expected="$3" created confirmed summary
    created="$(two_phase "$key-register" repo register --input "$input")"
    # Only the fields this step is about: the observation carries the platform
    # credential reference, which has no business in a test log.
    summary="$(json_get '{lifecycle:.registration.lifecycle,
                         delivered:.registration.delivered,
                         full_name:.registration.observed.full_name,
                         error:.error}' "$created")"
    assert_json ".lifecycle == \"$expected\" and .delivered == true and .error == null" \
        "$summary" "registration did not deliver"
    DEMO3_REPO="$(json_str '.registration.repo_id' "$created")"
    # The board scope the Project's attach has to approve is the platform's own
    # stable id for the repository, so it is carried alongside the repo id.
    DEMO3_PLATFORM_ID="$(json_str '.registration.observed.stable_id' "$created")"
    if [[ "$expected" == active ]]; then
        note "repo $DEMO3_REPO bound as $(json_str '.full_name' "$summary")"
        return 0
    fi
    confirmed="$(two_phase "$key-confirm" repo register \
        --confirm "$DEMO3_REPO" \
        --version "$(json_str '.registration.version' "$created")" \
        --platform-repo-id "$DEMO3_PLATFORM_ID")"
    assert_json '.lifecycle == "active"' "$confirmed" \
        "confirming the registration did not activate it"
    note "repo $DEMO3_REPO registered as $(json_str '.full_name' "$summary")"
}

demo3_project_create() { # <repo_id> <name>
    local repo="$1" name="$2" created
    "$HCTL2_JQ" -n --arg repo "$repo" --arg name "$name" '{
        repo_id: $repo,
        definition: {
            name: $name,
            goal: "demo 3: one read-only dispatch, answered in the Room",
            scope: "the registered repo",
            roles: [], role_members: {}, defaults: {},
            settings: {
                selection_policy: {},
                publish_review_requires_confirmation: true
            }
        }
    }' >"$DEMO3_ROOT/project.json"
    created="$(two_phase demo3-project project create \
        --input "$DEMO3_ROOT/project.json")"
    DEMO3_PROJECT="$(json_str '.project_id' "$created")"
    DEMO3_ROOM="$(json_str '.main_room_id' "$created")"
    note "project $DEMO3_PROJECT with main Room $DEMO3_ROOM"
}

# The Task actions carry an expected Project version, and every roster or source
# write moves it, so it is read back instead of assumed.
demo3_project_version() {
    json_get '.project.version' "$(hctl2 project show "$DEMO3_PROJECT")"
}

# Connect a Task source, create the card and adopt a contract for it. The
# contract's proposal digest is the caller's claim and the product verifies it
# against the RFC 8785 canonical bytes of the contract, so it is computed here
# from the same JSON that is submitted -- a mismatch fails as PROPOSAL_DIGEST.
demo3_task_with_contract() { # <repo_id> <candidate_id>
    local repo="$1" candidate="$2" source created before contract digest version
    version="$(demo3_project_version)"
    "$HCTL2_JQ" -n --arg repo "$repo" --arg candidate "$candidate" \
        '{repo_id:$repo,candidate_id:$candidate,consent:true,make_default:false}' \
        >"$DEMO3_ROOT/task-source.json"
    source="$(two_phase demo3-source task connect \
        --input "$DEMO3_ROOT/task-source.json")"
    DEMO3_SOURCE="$(json_str '.source_id' "$source")"

    # Connecting only records the source on the Repo; the Project has to approve
    # the board scope before a card can live there, and that scope is the
    # platform's own stable id for the repository.
    "$HCTL2_JQ" -n --arg project "$DEMO3_PROJECT" --arg source "$DEMO3_SOURCE" \
        --arg scope "$DEMO3_PLATFORM_ID" --argjson version "$version" '{
        project_id: $project, project_version: $version, source_id: $source,
        approved_scope: $scope, consent: true
    }' >"$DEMO3_ROOT/task-attach.json"
    two_phase demo3-attach task attach \
        --input "$DEMO3_ROOT/task-attach.json" >/dev/null

    version="$(demo3_project_version)"
    "$HCTL2_JQ" -n --arg project "$DEMO3_PROJECT" --arg source "$DEMO3_SOURCE" \
        --argjson version "$version" '{
        project_id: $project, project_version: $version, source_id: $source,
        title: "demo 3 read-only chain",
        body: "one read-only dispatch, answered in the Room"
    }' >"$DEMO3_ROOT/task.json"
    created="$(two_phase demo3-task task create --input "$DEMO3_ROOT/task.json")"
    DEMO3_TASK="$(json_str '.task_id' "$created")"

    contract="$("$HCTL2_JQ" -cS -n '{
        scope: "demo 3 read-only chain",
        expected_outcome: "answer one read-only request in the Room",
        acceptance: [{
            text: "the answer reaches the Room exactly once",
            grade: "human"
        }],
        roles: [], capabilities: []
    }')"
    digest="$(printf '%s' "$contract" | demo3_sha256)"
    before="$(hctl2 task show "$DEMO3_PROJECT" "$DEMO3_TASK")"
    # A local proposal's source is an ordinary record in this Project, and the
    # product re-reads it and rejects a stale reference, so the Project's own
    # current version is what goes into both fields.
    version="$(demo3_project_version)"
    "$HCTL2_JQ" -n \
        --arg project "$DEMO3_PROJECT" --arg task "$DEMO3_TASK" \
        --argjson version "$(json_get '.version' "$before")" \
        --argjson project_version "$version" \
        --argjson contract "$contract" --arg digest "$digest" '{
        project_id: $project, project_version: $project_version, task_id: $task,
        version: $version,
        adoption: {
            contract: $contract,
            origin: {
                kind: "local",
                reference: {
                    key: {scope: {kind: "project", id: $project},
                          kind: "project", id: $project},
                    version: {state: $project_version}
                },
                proposal_digest: $digest
            }
        }
    }' >"$DEMO3_ROOT/adopt.json"
    two_phase demo3-adopt task adopt --input "$DEMO3_ROOT/adopt.json" >/dev/null
    note "task $DEMO3_TASK on source $DEMO3_SOURCE carries a contract"
}

# One read-only dispatch from the Room, and the wait for its answer: the
# Invocation reaches `completed`, the answer is the script body's exact output,
# and that output appears in the Room timeline exactly once.
demo3_read_only_dispatch() { # <project> <room>
    local project="$1" room="$2"
    local show plan token started invocation attempt state timeline count
    show="$(hctl2 room show "$project" "$room")"
    "$HCTL2_JQ" -n --arg project "$project" --arg room "$room" \
        --argjson version "$(json_get '.binding.version' "$show")" '{
        project_id: $project, room_id: $room, version: $version,
        body: "READ_ONLY_CONTEXT_MARKER"
    }' >"$DEMO3_ROOT/send.json"
    two_phase demo3-context room send --input "$DEMO3_ROOT/send.json" >/dev/null

    "$HCTL2_JQ" -n --arg project "$project" --arg room "$room" \
        --argjson profile "$(json_get '.revision' "$DEMO3_PROFILE")" \
        --argjson deadline "$(( $(date +%s) * 1000 + 60000 ))" '{
        project_id: $project, room_id: $room, target: "research",
        profile: $profile,
        request: "Return the exact answer without modifying files",
        budget: 65536, deadline_ms: $deadline, retry_of: null
    }' >"$DEMO3_ROOT/invocation.json"
    plan="$(hctl2 invocation preview --input "$DEMO3_ROOT/invocation.json" \
        --key demo3-invoke)"
    token="$(json_str '.preview_token' "$plan")"
    # Starting on an invented token must not create an Invocation.
    if hctl2 invocation start --input "$DEMO3_ROOT/invocation.json" \
        --key demo3-invoke --preview-token invented \
        >"$DEMO3_ROOT/refused.json" 2>&1; then
        die "invocation start accepted an invented preview token"
    fi
    assert_json '.error.code == "PREVIEW_REQUIRED"' \
        "$(cat "$DEMO3_ROOT/refused.json")" \
        "an invented preview token was refused for the wrong reason"
    started="$(hctl2 invocation start --input "$DEMO3_ROOT/invocation.json" \
        --key demo3-invoke --preview-token "$token")"
    invocation="$(json_str '.invocation_id' "$started")"
    note "invocation $invocation started"

    state=""
    for attempt in $(seq 1 150); do
        state="$(hctl2 invocation show "$project" "$invocation")"
        if [[ "$(json_get '.state' "$state")" == '"completed"' ]]; then break; fi
        sleep 0.2
    done
    # Only the fields this step is about: the dispatch intent carries the whole
    # assembled bundle, whose bytes have no business in a test log.
    assert_json '.state == "completed"' \
        "$(json_get '{state:.state,
                      reason:.reason,
                      outputs:[.results[]?.output],
                      pending:[.pending_effects[]?
                               | {operation:.operation, state:.state}]}' "$state")" \
        "the invocation did not complete"
    [[ "$(json_get '.results[0].output' "$state")" == "\"$DEMO3_ANSWER\"" ]] ||
        die "the answer is not the script body's output"

    count=0
    timeline=""
    for attempt in $(seq 1 100); do
        timeline="$(hctl2 room timeline "$project" "$room")"
        count="$(json_get \
            "[.events[] | select(.content.body == \"$DEMO3_ANSWER\")] | length" \
            "$timeline")"
        if [[ "$count" != 0 ]]; then break; fi
        sleep 0.2
    done
    [[ "$count" == 1 ]] ||
        die "the answer appears $count times in the Room timeline"
    note "the answer reached the Room exactly once"
}

demo3_delete_card() { # <project> <task>
    local project="$1" task="$2" before
    before="$(hctl2 task show "$project" "$task")"
    "$HCTL2_JQ" -n --arg project "$project" --arg task "$task" \
        --argjson version "$(json_get '.version' "$before")" '{
        project_id: $project, task_id: $task, version: $version,
        confirm_irreversible: true, active_run_choices: []
    }' >"$DEMO3_ROOT/delete-card.json"
    two_phase demo3-delete-card task delete-card \
        --input "$DEMO3_ROOT/delete-card.json" >/dev/null
    note "the platform card this run created is deleted again"
}

# Interface slots for the rest of demo 3 (kickoff §四 第 9 包验收第 8 条). The
# call sites are already in order in both paths, and since #405 every command
# they name is on main: `changeset seal`, `review publish`, `integration
# preview`/`submit`, `task complete`. What is still missing is an execution body
# that can hold a write session -- this demo's body is the read-only script
# fixture above, which answers with one fixed line and edits nothing, so a
# write-type dispatch has nothing to lease to. Both paths therefore stop after
# the read-only answer until 3f (Agency write-type session) lands.
demo3_write_type_dispatch() { # <project> <room>
    note "TODO(demo3): write-type dispatch -- a write-mode Worker Profile carrying git.write, then invocation preview showing the ChangeSet, the lease and the frozen publish policy, and a harness editing real non-document code in an isolated worktree (blocked on 3f)"
}
demo3_seal() { # <project> <invocation>
    note "TODO(demo3): seal -- hctl2 changeset seal freezes the ChangeSet into a Revision and admits it, then changeset show and changeset diff read the exact bytes back (blocked on the write-type dispatch above)"
}
demo3_publish() { # <repo> <intent>
    note "TODO(demo3): publish -- hctl2 review publish opens the review request on the platform, then hctl2 review show reads its two stages back (blocked on seal)"
}
demo3_merge() { # <repo> <intent>
    note "TODO(demo3): merge -- hctl2 integration preview then submit with the source head pinned, and hctl2 integration show reading the Integration Receipt back (blocked on publish)"
}
demo3_complete() { # <project> <task>
    note "TODO(demo3): complete -- hctl2 task complete, then read the Task Completion Receipt back (blocked on merge)"
}

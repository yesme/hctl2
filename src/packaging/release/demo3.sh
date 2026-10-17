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
DEMO3_PATH=""
DEMO3_AGENCY_ROOT=""
DEMO3_AGENCY_PID=""
DEMO3_PORTS=""
DEMO3_PROFILE=""
DEMO3_PROFILE_WRITE=""
DEMO3_ROSTER_VERSION=""
DEMO3_BINDING_REF=""
DEMO3_PROFESSION_REF=""
DEMO3_PROFESSION_JSON=""
DEMO3_PROFESSION_DIGEST=""
DEMO3_REPO=""
DEMO3_PLATFORM_ID=""
DEMO3_PROJECT=""
DEMO3_ROOM=""
DEMO3_SOURCE=""
DEMO3_TASK=""
DEMO3_CHANGESET=""
DEMO3_INVOCATION=""
DEMO3_REVISION=""
DEMO3_PUBLISH_INTENT=""
DEMO3_PUBLISH_COMMIT=""
DEMO3_PR_INDEX=""
DEMO3_MERGE_INTENT=""
DEMO3_INTEGRATION_RECEIPT=""
DEMO3_COMPLETION_RECEIPT=""
DEMO3_HUMAN_PREVIEWS=""

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
    # The control's delivery and recovery loops report there and nowhere else.
    if [[ -n "$DEMO3_ROOT" && -s "$DEMO3_ROOT/control.err" ]]; then
        printf 'demo3 --- control.err ---\n' >&2
        tail -n 30 "$DEMO3_ROOT/control.err" >&2
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
# Unquoted scalar for a value that may be absent: `null` stays the four letters,
# which callers test for instead of dying.
json_raw() { "$HCTL2_JQ" -r "$1" <<<"$2"; }
assert_json() { # <filter> <json> <what went wrong>
    "$HCTL2_JQ" -e "$1" <<<"$2" >/dev/null || die "$3: $2"
}

# One submit, retried only on the product's documented RESULT_UNKNOWN recovery:
# the provider write may have landed while its read-back had not caught up. The
# product's recovery is to read the effect back, never to resend, and a preview
# token is single-use, so each attempt previews again under the same key: the
# preview is idempotent, and the submit then reads the existing effect.
# Preview and submit are separate commands for `integration`, so both are given.
DEMO3_SUBMIT_ERROR=""
demo3_submit_resumable() { # <key> <preview command...> -- <submit command...>
    local key="$1"
    shift
    local preview_args=() submit_args=() preview token submitted status attempt code split=0
    while [[ $# -gt 0 ]]; do
        if [[ "$1" == -- && "$split" -eq 0 ]]; then
            split=1
            shift
            continue
        fi
        if [[ "$split" -eq 0 ]]; then
            preview_args+=("$1")
        else
            submit_args+=("$1")
        fi
        shift
    done
    [[ "$split" -eq 1 && ${#submit_args[@]} -gt 0 ]] ||
        die "internal: submit command missing after --"
    for attempt in $(seq 1 20); do
        preview="$(hctl2 "${preview_args[@]}" --key "$key")" ||
            die "preview of $key failed: $preview"
        token="$(json_str '.preview_token' "$preview")" ||
            die "preview of $key returned no token: $preview"
        status=0
        submitted="$(hctl2 "${submit_args[@]}" --key "$key" --preview-token "$token")" ||
            status=$?
        if [[ "$status" -eq 0 ]]; then
            printf '%s\n' "$submitted"
            return 0
        fi
        code="$(json_raw '.error.code // empty' "$submitted" 2>/dev/null || true)"
        [[ "$code" == RESULT_UNKNOWN ]] || break
        sleep 0.5
    done
    DEMO3_SUBMIT_ERROR="$submitted"
    return 1
}
# The product's write convention: the same command twice, first without a token
# so it only previews, then with the exact token so it submits. There is no
# one-shot form, so the preview is never skipped by accident.
# No caller tolerates a refusal here, so both phases report the answer they got:
# with `--json` the diagnosis is on stdout, and stdout is what the caller
# captures, so an unreported failure would leave the runner log with a line
# number and no reason.
two_phase() { # <key> <command...> (no --key, no --preview-token)
    local key="$1"
    shift
    demo3_submit_resumable "$key" "$@" -- "$@" ||
        die "submit of $key failed: $DEMO3_SUBMIT_ERROR"
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
        # A failed run is diagnosed from what the root still holds: the control
        # log, the Agency's own store and the stream it kept. The knob is for a
        # human debugging a red run; CI never sets it.
        if [[ -n "${HCTL2_DEMO3_KEEP:-}" ]]; then
            note "keeping $DEMO3_ROOT (HCTL2_DEMO3_KEEP)"
            return 0
        fi
        find "${DEMO3_ROOT:?}" -depth -delete
    fi
}

# The archive is a zstd frame, so it is decoded with the pinned tool instead of
# relying on whichever decoder the host tar happens to have, and decoded into
# tar rather than into a file: the payload unpacks to 635 MB and the tar it comes
# from is the same again, and the complete package job runs several of these
# payload tests at once on a runner whose disk is bounded, so writing the tar out
# would more than double what this one test holds at its peak.
#
# A pipe, not process substitution, so `pipefail` also sees the decoder. tar's
# status alone does not say whether the payload arrived: with a corrupt trailing
# checksum zstd writes nothing at all, tar reports no error and exits 0, and the
# install directory is simply empty. `--ignore-zeros` is what makes the pipe
# safe -- tar stops reading at the first end-of-archive marker, so the padding a
# valid archive legitimately carries would otherwise kill the decoder with
# SIGPIPE and fail a run that unpacked perfectly (exit 141, seen on the macOS
# arm64 runner). Spelled long: bsdtar rejects `-i` in a short-option cluster.
#
# Both shapes are pinned by root//packaging/release:demo3-unpack-test.
demo3_unpack_payload() { # <archive> <dest>
    "$HCTL2_ZSTD_ROOT/bin/zstd" -dc "$1" | tar -xf - --ignore-zeros -C "$2"
}

demo3_prepare() { # <name>
    local name="$1"
    DEMO3_PATH="$name"
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
    demo3_unpack_payload "$archive" "$DEMO3_ROOT/install"
    services_bin="$(find "$DEMO3_ROOT/install" -type f -name hctl2-services \
        -path '*/bin/*' | head -n 1)"
    [[ -n "$services_bin" ]] || die "dependency package has no bin/hctl2-services"
    DEMO3_PAYLOAD="${services_bin%/bin/hctl2-services}"
    demo3_isolate_ports
    note "$name path: payload at $DEMO3_PAYLOAD"
}

demo3_control_start() {
    # Started here rather than through `hctl2 start` so its stderr is kept: the
    # delivery and recovery loops report their failures there and nowhere else,
    # and `hctl2 start` discards that stream. The secret backend stays the
    # disposable one on purpose; the packaged default is the system keyring,
    # which a CI runner has no reason to have.
    # The backend is recorded the way `hctl2 start` records it; only the spawn
    # itself moves here.
    hctl2 init --secret-backend user-file >/dev/null
    HCTL2_INSTALL_ROOT="$DEMO3_PAYLOAD" "$HCTL2_CONTROL_BIN" --root "$DEMO3_ROOT" \
        >/dev/null 2>"$DEMO3_ROOT/control.err" &
    local attempt status
    for attempt in $(seq 1 600); do
        status="$(hctl2 status 2>/dev/null || true)"
        if [[ -n "$status" ]] && printf '%s' "$status" | "$HCTL2_JQ" -e \
            '.ready == true' >/dev/null 2>&1; then
            note "control $(json_str '.control_id' "$status") is ready"
            return 0
        fi
        sleep 0.1
    done
    die "control did not become ready: $status"
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
    demo3_agency_spawn
}

# Nothing of the Agency's stdout is kept: it answers `hctl2 agency pair`, so it
# holds the pairing material. Its stderr is kept for `die` to show. Neither may
# stay attached to the test's own output pipe, or the harness would wait for
# this background child after the test had finished.
demo3_agency_spawn() {
    "$HCTL2_AGENCY_BIN" --root "$DEMO3_AGENCY_ROOT" serve \
        --script-config "$DEMO3_ROOT/script.json" \
        >/dev/null 2>"$DEMO3_ROOT/agency.err" &
    DEMO3_AGENCY_PID=$!
    local attempt
    for attempt in $(seq 1 200); do
        if "$HCTL2_AGENCY_BIN" --root "$DEMO3_AGENCY_ROOT" status >/dev/null 2>&1; then
            note "script Agency is serving (pid $DEMO3_AGENCY_PID)"
            return 0
        fi
        sleep 0.05
    done
    die "script Agency did not start"
}

demo3_agency_stop() {
    [[ -n "$DEMO3_AGENCY_PID" ]] || return 0
    kill "$DEMO3_AGENCY_PID" 2>/dev/null || true
    local _
    for _ in $(seq 1 100); do
        kill -0 "$DEMO3_AGENCY_PID" 2>/dev/null || break
        sleep 0.05
    done
    kill -9 "$DEMO3_AGENCY_PID" 2>/dev/null || true
    wait "$DEMO3_AGENCY_PID" 2>/dev/null || true
    DEMO3_AGENCY_PID=""
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
    # The write segment reselects this roster with a write Profile, so the
    # exact references the first selection used are kept.
    DEMO3_BINDING_REF="$(json_get \
        '{"key":.binding.key,"version":{"state":.binding.version}}' "$binding")"
    DEMO3_PROFESSION_REF="$(json_get \
        '{"key":.profession.key,"version":{"state":.profession.version}}' \
        "$accepted_profession")"
    DEMO3_PROFESSION_JSON="$profession"
    DEMO3_PROFESSION_DIGEST="$digest"
    local selected
    selected="$(two_phase demo3-select project select \
        --input "$DEMO3_ROOT/select.json")"
    DEMO3_ROSTER_VERSION="$(json_str '.roster_version' "$selected")"
    note "roster selected one read-only research Worker (version $DEMO3_ROSTER_VERSION)"
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
            goal: "demo 3: a scripted write dispatched from the Room, reviewed and merged on the platform, then completed",
            scope: "the registered repo",
            roles: [], role_members: {}, defaults: {},
            settings: {
                selection_policy: {},
                publish_review_requires_confirmation: false
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
        title: "demo 3 scripted write chain",
        body: "a scripted write is dispatched from the Room, published, merged and completed"
    }' >"$DEMO3_ROOT/task.json"
    created="$(two_phase demo3-task task create --input "$DEMO3_ROOT/task.json")"
    DEMO3_TASK="$(json_str '.task_id' "$created")"

    contract="$("$HCTL2_JQ" -cS -n '{
        scope: "demo 3 scripted write chain",
        expected_outcome: "the scripted change is integrated into main",
        acceptance: [
            {
                text: "the scripted change is integrated",
                grade: "mechanical",
                evidence: {accept: "integration_receipt", min_channel: "unmediated"}
            },
            {text: "a human judged the change done", grade: "human"}
        ],
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

# The write segment (kickoff §四 第 9 包验收第 8 条). The read-only chain above
# ends with the answer in the Room; these steps carry the same Project through a
# write-type dispatch, sealing, publication, integration and completion, and
# every state change is a `hctl2 --json` command. The execution body is a
# script: there is no harness here holding a write session, so the Agency's
# script executor edits one file in its own isolated directory and answers with
# a ChangeSet result frame, the same shape
# apps/cli/tests/room/project.rs::configure_git_writer uses.
#
# Platform facts are read back once per segment. On GitHub the packaged gh
# carries the same credential the runner gave control, so the pull request, its
# merge commit, the new main head and the required check are read independently.
# On the hosted Gitea the credential is control's alone (the script must not
# read control's secret store), so the facts there are the readback control
# itself recorded: `review show` stages and mappings, and the Integration
# Receipt.

# A second Worker Profile with the permissions a write lease needs, and a
# roster that names it. A select resends the whole roster at its exact version,
# so the read-only selection above is replaced rather than extended.
demo3_write_profile() { # <project> <room>
    local project="$1" room="$2" profile version selected
    "$HCTL2_JQ" -n --argjson profession "$DEMO3_PROFESSION_JSON" '{
        id: "coding",
        profile: {
            harness: $profession.harness,
            model: $profession.model,
            mode: "write",
            permissions: ["context.read", "git.read", "git.write"],
            environment: [],
            required_capabilities: {
                input: false, stop: false, event_cursor: false,
                input_provenance: false, managed_single_writer: false,
                secure_input: false, exact_attach: false,
                tool_execution_unmediated: false, isolation_effects: []
            },
            max_context_bytes: 65536
        }
    }' >"$DEMO3_ROOT/write-profile.json"
    profile="$(two_phase demo3-write-profile profile create \
        --input "$DEMO3_ROOT/write-profile.json")"
    DEMO3_PROFILE_WRITE="$(json_get '.revision' "$profile")"
    version="$(demo3_project_version)"
    "$HCTL2_JQ" -n \
        --arg project "$project" --arg room "$room" \
        --argjson version "$version" \
        --argjson roster_version "$DEMO3_ROSTER_VERSION" \
        --argjson profession_ref "$DEMO3_PROFESSION_REF" \
        --argjson binding_ref "$DEMO3_BINDING_REF" \
        --arg digest "$DEMO3_PROFESSION_DIGEST" \
        --argjson profile_revision "$DEMO3_PROFILE_WRITE" '{
        project_id: $project, project_version: $version, room_id: $room,
        topic_command_key: null, roster_version: $roster_version,
        selections: [{
            room_id: $room,
            selected_item: $profession_ref,
            profession: $profession_ref,
            profession_digest: $digest,
            agency: $binding_ref,
            required_skills: [], optional_skills: [],
            worker_profiles: [$profile_revision],
            responsibility: "coding",
            permission: {allow: ["context.read", "git.read", "git.write"]},
            budget: {max_bytes: 65536},
            display_name: "Coding",
            persona_tags: []
        }]
    }' >"$DEMO3_ROOT/select-write.json"
    selected="$(two_phase demo3-select-write project select \
        --input "$DEMO3_ROOT/select-write.json")"
    DEMO3_ROSTER_VERSION="$(json_str '.roster_version' "$selected")"
    note "roster selected one write Worker for the coding responsibility"
}

# Replace the read-only execution body with the write one, for exactly the
# lease the preview just issued. The baseline travels as a fast-export stream
# rather than as a path: the execution directory is the only place this body
# may read, and the export is text for both demo repositories (a binary
# baseline is refused below instead of silently truncated by argv).
demo3_git_writer_up() { # <worktree> <pending> <path>
    local worktree="$1" pending="$2" path="$3"
    local change_set lease_id generation baseline frame envelope script
    change_set="$(json_str '.change_set_id' "$pending")"
    lease_id="$(json_str '.lease.lease_id' "$pending")"
    generation="$(json_get '.lease.generation' "$pending")"
    baseline="$(json_str '.baseline_commit' "$pending")"
    frame="$("$HCTL2_JQ" -c -n \
        --arg change_set "$change_set" --arg lease "$lease_id" \
        --argjson generation "$generation" --arg baseline "$baseline" '{
        change_set_id: $change_set,
        lease: {lease_id: $lease, generation: $generation},
        base_commit_sha: $baseline,
        parent_revision_id: null,
        location: {
            kind: "commit", repo_path: "EXECUTION_PATH",
            commit_sha: "RESULT_COMMIT"
        }
    }')"
    # The terminal frame is the adapter envelope: its `output` is the ChangeSet
    # result as a JSON string, which is what admission reads.
    envelope="$("$HCTL2_JQ" -c -n --arg output "$frame" '{
        type: "result", schema: "hctl2.changeset-output.v1", output: $output
    }')"
    git -C "$worktree" fast-export main >"$DEMO3_ROOT/baseline.export" ||
        die "fast-export of $worktree failed"
    if [[ "$(wc -c <"$DEMO3_ROOT/baseline.export")" != \
        "$(tr -d '\000' <"$DEMO3_ROOT/baseline.export" | wc -c)" ]]; then
        die "the write baseline carries binary data; this demo passes it as text"
    fi
    script="$(cat <<'DEMO3_WRITER'
set -eu
exec 2>git-error.txt
read -r initial
mkdir git-result
cd git-result
git init -q --initial-branch=main
printf '%s' "$1" | git fast-import --quiet
git reset --hard --quiet main
git config user.name 'Demo 3 Script'
git config user.email 'demo3@example.invalid'
mkdir -p "$(dirname "$3")"
printf '%s\n' 'written by the demo 3 script execution' >"$3"
git add -A
git commit --quiet -m 'demo 3 script result'
commit=$(git rev-parse HEAD)
printf '%s\n' "$2" | sed -e "s|EXECUTION_PATH|$PWD|g" -e "s|RESULT_COMMIT|$commit|g" | tee ../result-frame.json
while read -r managed_input; do :; done
DEMO3_WRITER
)"
    "$HCTL2_JQ" -n --arg body "$script" \
        --rawfile exported "$DEMO3_ROOT/baseline.export" \
        --arg frame "$envelope" --arg path "$path" '{
        program: "/bin/sh",
        arguments: ["-c", $body, "git-writer", $exported, $frame, $path]
    }' >"$DEMO3_ROOT/script.json"
    # The same Agency root keeps the pairing and the catalog; only the script
    # config is replaced, so the control reaches the same binding after restart.
    demo3_agency_stop
    demo3_agency_spawn
}

demo3_write_type_dispatch() { # <project> <room> <worktree> <path>
    local project="$1" room="$2" worktree="$3" path="$4"
    local baseline deadline preview write pending token started attempt state
    baseline="$(git -C "$worktree" rev-parse HEAD)" ||
        die "the write worktree has no baseline: $worktree"
    deadline=$(( $(date +%s) * 1000 + 120000 ))
    "$HCTL2_JQ" -n \
        --arg project "$project" --arg room "$room" --arg task "$DEMO3_TASK" \
        --argjson profile "$DEMO3_PROFILE_WRITE" \
        --argjson deadline "$deadline" --arg baseline "$baseline" \
        --arg path "$path" '{
        project_id: $project, room_id: $room, task_id: $task,
        target: "coding", profile: $profile,
        request: ("Write " + $path + " with one line for the adopted Task"),
        budget: 65536, deadline_ms: $deadline, retry_of: null,
        write: {
            change_set_id: null, baseline_commit: $baseline,
            target_branch: "main", allow_update: true
        }
    }' >"$DEMO3_ROOT/write-invocation.json"
    preview="$(hctl2 invocation preview \
        --input "$DEMO3_ROOT/write-invocation.json" --key demo3-write)" ||
        die "the write preview failed: $preview"
    write="$(json_get '.effect_summary.preview.write' "$preview")"
    assert_json '.authorization == "publish_for_review_not_integration"' "$write" \
        "the write preview does not authorize a review publication"
    assert_json '.publication_target.target_branch == "main"' "$write" \
        "the write preview does not target main"
    assert_json '.publication_target.requires_human_confirmation == false' "$write" \
        "the default path must publish without a separate human gate"
    assert_json '(.lease.pending.change_set_id // "") != ""' "$write" \
        "the write preview carries no pending ChangeSet"
    pending="$(json_get '.lease.pending' "$write")"
    DEMO3_CHANGESET="$(json_str '.change_set_id' "$pending")"
    note "write lease pending on ChangeSet $DEMO3_CHANGESET (baseline $baseline)"
    demo3_git_writer_up "$worktree" "$pending" "$path"
    token="$(json_str '.preview_token' "$preview")"
    started="$(hctl2 invocation start --input "$DEMO3_ROOT/write-invocation.json" \
        --key demo3-write --preview-token "$token")" ||
        die "starting the write invocation failed: $started"
    DEMO3_INVOCATION="$(json_str '.invocation_id' "$started")"
    note "write invocation $DEMO3_INVOCATION started"
    for attempt in $(seq 1 300); do
        state="$(hctl2 invocation show "$project" "$DEMO3_INVOCATION")"
        case "$(json_get '.state' "$state")" in
            '"completed"')
                note "write invocation completed"
                return 0 ;;
            '"failed"')
                die "the write invocation failed: $state" ;;
        esac
        sleep 0.2
    done
    die "the write invocation did not complete in 60 seconds"
}

demo3_seal() { # <project> <invocation>
    local project="$1" invocation="$2" shown diff revision
    shown="$(hctl2 invocation show "$project" "$invocation")" ||
        die "invocation show failed: $shown"
    [[ "$(json_get '.state' "$shown")" == '"completed"' ]] ||
        die "the write invocation is not complete: $shown"
    shown="$(hctl2 changeset show "$DEMO3_REPO" "$DEMO3_CHANGESET")" ||
        die "changeset show failed: $shown"
    assert_json '(.revisions | length) == 1' "$shown" \
        "the ChangeSet did not freeze into exactly one revision"
    revision="$(json_str '.revisions[0].change_set_revision_id' "$shown")"
    DEMO3_REVISION="$revision"
    diff="$(hctl2 changeset diff "$DEMO3_REPO" "$DEMO3_CHANGESET" "$revision")" ||
        die "changeset diff failed: $diff"
    assert_json '.diff | contains("written by the demo 3 script execution")' \
        "$diff" "the admitted revision does not carry the script's bytes"
    note "ChangeSet $DEMO3_CHANGESET sealed as $revision; the diff carries the script's line"
}

# The GitHub facts are the platform's own: the packaged gh reads the same
# repository and pull request control pushed to.
demo3_gh() {
    local gh="$DEMO3_PAYLOAD/libexec/hctl2/gh"
    [[ -x "$gh" ]] || die "the package carries no gh: $gh"
    "$gh" "$@"
}

# The platform only merges once the required check finished; the live GitHub
# chain waits for it the same way before submitting.
demo3_github_wait_check() { # <commit>
    local commit="$1" attempt status
    for attempt in $(seq 1 150); do
        status="$(demo3_gh api "repos/$DEMO3_GITHUB_REPO/commits/$commit/check-runs")" ||
            die "gh api check-runs failed for $commit"
        if printf '%s' "$status" | "$HCTL2_JQ" -e \
            '.check_runs[] | select(.name == "canary" and .status == "completed" and .conclusion == "success")' \
            >/dev/null 2>&1; then
            note "the canary check passed on $commit"
            return 0
        fi
        sleep 2
    done
    die "the canary check did not pass on $commit: $status"
}

demo3_github_facts() { # <number> <published-commit> <receipt>
    local number="$1" head="$2" receipt="$3" pr main
    pr="$(demo3_gh api "repos/$DEMO3_GITHUB_REPO/pulls/$number")" ||
        die "gh api pull $number failed"
    main="$(demo3_gh api "repos/$DEMO3_GITHUB_REPO/branches/main")" ||
        die "gh api main failed"
    assert_json '.merged == true' "$pr" "the review request was not merged"
    [[ "$(json_str '.head.sha' "$pr")" == "$head" ]] ||
        die "the merged request does not carry the published commit"
    [[ "$(json_str '.merge_commit_sha' "$pr")" == "$(json_str '.integrated_commit' "$receipt")" ]] ||
        die "the platform's merge commit is not the Receipt's integrated commit"
    [[ "$(json_str '.commit.sha' "$main")" == "$(json_str '.target_head_after' "$receipt")" ]] ||
        die "the platform's main head is not the Receipt's target head"
    note "GitHub facts: PR #$number merged as $(json_str '.merge_commit_sha' "$pr"), main at $(json_str '.commit.sha' "$main")"
}

demo3_publish() { # <repo>
    local repo="$1" attempt listed intent shown
    for attempt in $(seq 1 300); do
        listed="$(hctl2 review list "$repo")" ||
            die "review list failed while waiting for publication: $listed"
        intent="$(json_raw '.items[0].intent_id // empty' "$listed")"
        if [[ -n "$intent" && "$intent" != null ]]; then
            shown="$(hctl2 review show "$repo" "$intent")" ||
                die "review show failed for intent $intent (listed: $listed): $shown"
            if [[ "$(json_get '.intent.state' "$shown")" == '"published"' ]]; then
                DEMO3_PUBLISH_INTENT="$intent"
                DEMO3_PUBLISH_COMMIT="$(json_str '.mappings[0].platform_commit_sha' "$shown")"
                DEMO3_PR_INDEX="$(json_str '.mappings[0].review_request.index' "$shown")"
                assert_json '(.mappings | length) == 1' "$shown" \
                    "the published revision has more than one platform mapping"
                assert_json '.stages.push.confirmed == true' "$shown" \
                    "the platform did not confirm the push"
                assert_json '.stages.review_request.confirmed == true' "$shown" \
                    "the platform did not confirm the review request"
                [[ "$(json_get '(.items | length)' "$listed")" == 1 ]] ||
                    die "there is more than one publication intent: $listed"
                note "published as review request #$DEMO3_PR_INDEX at $DEMO3_PUBLISH_COMMIT"
                return 0
            fi
        fi
        sleep 0.2
    done
    die "the publication did not reach the platform in 60 seconds"
}

demo3_merge() { # <repo>
    local repo="$1" strategy=fast_forward input preview token submitted attempt shown receipt
    [[ "$DEMO3_PATH" == github ]] && strategy=merge_commit
    "$HCTL2_JQ" -n --arg repo "$repo" --arg revision "$DEMO3_REVISION" \
        --arg strategy "$strategy" '{
        repo_id: $repo, change_set_revision_id: $revision,
        target_kind: "platform", target_ref: "refs/heads/main",
        form: "accept_advance", strategy: $strategy
    }' >"$DEMO3_ROOT/integration.json"
    preview="$(hctl2 integration preview --input "$DEMO3_ROOT/integration.json" \
        --key demo3-merge)" || die "the integration preview failed: $preview"
    if [[ "$DEMO3_PATH" == github ]]; then
        assert_json '.effect_summary.protection.required_checks == ["canary"]' "$preview" \
            "the GitHub target does not require the canary check"
        assert_json '.effect_summary.protection.requires_review_request == true' "$preview" \
            "the GitHub target does not require a review request"
        demo3_github_wait_check "$DEMO3_PUBLISH_COMMIT"
    fi
    submitted="$(demo3_submit_resumable demo3-merge \
        integration preview --input "$DEMO3_ROOT/integration.json" -- \
        integration submit --input "$DEMO3_ROOT/integration.json")" ||
        die "the integration submit failed: $DEMO3_SUBMIT_ERROR"
    DEMO3_MERGE_INTENT="$(json_str '.intent_id' "$submitted")"
    DEMO3_HUMAN_PREVIEWS="$DEMO3_HUMAN_PREVIEWS integration"
    note "human preview 1 submitted: integration $DEMO3_MERGE_INTENT"
    for attempt in $(seq 1 450); do
        shown="$(hctl2 integration show "$repo" "$DEMO3_MERGE_INTENT")"
        case "$(json_get '.intent.state' "$shown")" in
            '"succeeded"') break ;;
            '"failed"') die "the integration failed: $shown" ;;
        esac
        sleep 0.2
    done
    [[ "$(json_get '.intent.state' "$shown")" == '"succeeded"' ]] ||
        die "the integration did not succeed in 90 seconds"
    receipt="$(json_get '.receipt' "$shown")"
    DEMO3_INTEGRATION_RECEIPT="$(json_str '.receipt_id' "$receipt")"
    [[ "$(json_str '.source.change_set_revision_id' "$receipt")" == "$DEMO3_REVISION" ]] ||
        die "the Receipt's source is not the admitted revision"
    [[ "$(json_str '.evidence_level' "$receipt")" == hctl2-tool ]] ||
        die "the Receipt was not signed by the unmediated readback"
    if [[ "$DEMO3_PATH" == github ]]; then
        demo3_github_facts "$DEMO3_PR_INDEX" "$DEMO3_PUBLISH_COMMIT" "$receipt"
        [[ "$(json_str '.integrated_commit' "$receipt")" != "$DEMO3_PUBLISH_COMMIT" ]] ||
            die "a GitHub merge cannot be the candidate commit itself"
    else
        # The hosted Gitea merges by fast-forward: the platform's head, the
        # Receipt's target head and the integrated commit are the published
        # commit the platform returned on the push.
        [[ "$(json_str '.target_head_after' "$receipt")" == "$DEMO3_PUBLISH_COMMIT" ]] ||
            die "the target head after integration is not the published commit"
        [[ "$(json_str '.integrated_commit' "$receipt")" == "$DEMO3_PUBLISH_COMMIT" ]] ||
            die "the integrated commit is not the published commit"
    fi
    note "human preview 1 done: Receipt $DEMO3_INTEGRATION_RECEIPT for $DEMO3_REVISION"
}

demo3_complete() { # <project> <task>
    local project="$1" task="$2" shown version lifecycle revision_number uid
    local receipt_ref preview token progress
    shown="$(hctl2 task show "$project" "$task")" || die "task show failed: $shown"
    version="$(json_get '.version' "$shown")"
    lifecycle="$(json_get '.data.lifecycle_version' "$shown")"
    revision_number="$(json_get '.data.revision.number' "$shown")"
    uid="$(id -u)"
    receipt_ref="$("$HCTL2_JQ" -c -n --arg repo "$DEMO3_REPO" \
        --arg receipt "$DEMO3_INTEGRATION_RECEIPT" '{
        key: {scope: {kind: "repo", id: $repo},
              kind: "integration_receipt", id: $receipt},
        version: {state: 1}
    }')"
    "$HCTL2_JQ" -n \
        --arg project "$project" --arg task "$task" \
        --argjson version "$version" --argjson lifecycle "$lifecycle" \
        --argjson revision_number "$revision_number" \
        --argjson receipt_ref "$receipt_ref" --arg principal "local-owner:$uid" '{
        project_id: $project, task_id: $task, version: $version,
        lifecycle_version: $lifecycle, revision_number: $revision_number,
        acceptance: [
            {item: 0, judge: {kind: "hctl2_tool"}, channel: "unmediated",
             references: [$receipt_ref], producer: "hctl2-tool", generation: 1},
            {item: 1, judge: {kind: "human", actor: $principal},
             channel: "narrated", references: [], producer: null, generation: null}
        ]
    }' >"$DEMO3_ROOT/complete.json"
    preview="$(hctl2 task complete --key demo3-complete \
        --input "$DEMO3_ROOT/complete.json")" ||
        die "the completion preview failed: $preview"
    assert_json '.effect_summary.result.items[0].validation_level == "unmediated"' "$preview" \
        "the mechanical item is not backed by the unmediated Receipt"
    demo3_submit_resumable demo3-complete \
        task complete --input "$DEMO3_ROOT/complete.json" -- \
        task complete --input "$DEMO3_ROOT/complete.json" >/dev/null ||
        die "the completion submit failed: $DEMO3_SUBMIT_ERROR"
    DEMO3_HUMAN_PREVIEWS="$DEMO3_HUMAN_PREVIEWS completion"
    shown="$(hctl2 task show "$project" "$task")"
    [[ "$(json_str '.data.lifecycle' "$shown")" == completed ]] ||
        die "the Task is not completed: $shown"
    progress="$(json_get '.progress' "$shown")"
    DEMO3_COMPLETION_RECEIPT="$(json_str '.completion.current.receipt_id' "$progress")"
    # Both kinds of Receipt, one each, and the whole chain reachable from the
    # Task: the admitted revision, its review request, its integration Receipt.
    [[ "$(json_get '(.completion.history | length)' "$progress")" == 1 ]] ||
        die "the Task does not carry exactly one completion Receipt"
    [[ "$(json_get '(.revisions | length)' "$progress")" == 1 ]] ||
        die "the Task does not carry exactly one revision"
    [[ "$(json_str '.revisions[0].change_set_revision_id' "$progress")" == "$DEMO3_REVISION" ]] ||
        die "the Task's revision is not the admitted one"
    [[ "$(json_str '.revisions[0].integration.receipt_id' "$progress")" == "$DEMO3_INTEGRATION_RECEIPT" ]] ||
        die "the Task's integration Receipt is not the one signed above"
    [[ "$(json_str '.revisions[0].publication.review_request' "$progress")" == "$DEMO3_PR_INDEX" ]] ||
        die "the Task's review request is not the one the platform confirmed"
    note "human preview 2 done: Task completed with Receipts $DEMO3_INTEGRATION_RECEIPT + $DEMO3_COMPLETION_RECEIPT"
}

# The default path has exactly two human previews: the merge and the completion.
demo3_assert_previews() {
    [[ "$DEMO3_HUMAN_PREVIEWS" == " integration completion" ]] ||
        die "the default path must preview exactly the merge and the completion, got:$DEMO3_HUMAN_PREVIEWS"
}

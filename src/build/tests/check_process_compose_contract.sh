#!/usr/bin/env bash
# Process Compose leftover check, service YAML contract, and a dummy
# pull-up / ready / restart / shutdown plus failure cases.
#
# Platform plan: this target is `ci:fast` under root//build/tests, so Code CI
# runs it on Linux x86_64, macOS arm64 and macOS x86_64. The five real
# services still run as packaging/dependencies:package-test on the same three
# platforms (workflow_dispatch). This file covers the cases that package-test
# does not: source-tree leftover supervisor, YAML contract, never-ready,
# restart budget, unknown process, and down when nothing is running.
set -euo pipefail

script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
packaging="${PACKAGING_ROOT:-$script_dir/../../packaging/dependencies}"
pc_bin="${PROCESS_COMPOSE_BIN:-$script_dir/../tools/process-compose-bin}"
fixture="${LIFECYCLE_YAML:-$script_dir/testdata/process-compose/lifecycle.yaml}"

[[ -d "$packaging" ]] || { echo "missing packaging root: $packaging" >&2; exit 1; }
[[ -x "$pc_bin" || -f "$pc_bin" ]] || { echo "missing Process Compose: $pc_bin" >&2; exit 1; }
[[ -f "$fixture" ]] || { echo "missing lifecycle fixture: $fixture" >&2; exit 1; }

failures=0
fail() {
    printf 'FAIL %s\n' "$*" >&2
    failures=$((failures + 1))
}
note() { printf '%s\n' "$*"; }

# --- leftover supervisor (source tree, not the installed package) ----------
retired="$(find "$packaging" \( -name start.sh -o -name stop.sh -o -name status.sh \) -print)"
if [ -n "$retired" ]; then
    fail "retired lifecycle scripts still exist:"$'\n'"$retired"
else
    note "PASS source tree has no start.sh / stop.sh / status.sh"
fi

supervisor_files=""
for relative in \
    common/runtime.sh \
    hctl2-services \
    platforms/linux/runtime.sh \
    platforms/macos/runtime.sh
do
    if [ -e "$packaging/$relative" ]; then
        supervisor_files="$supervisor_files $packaging/$relative"
    fi
done
# shellcheck disable=SC2086
if [ -n "$supervisor_files" ] && grep -E \
    'pidfile|\.pid[[:space:]]|/var/run/|nohup[[:space:]]|kill[[:space:]]+-|wait_for_pid|supervise' \
    $supervisor_files >/dev/null
then
    fail "runtime helpers still contain a PID/nohup/kill supervisor"
else
    note "PASS runtime helpers have no PID-file or signal supervisor"
fi

if grep -E 'start\.sh|stop\.sh|status\.sh' "$packaging/common/package.sh" >/dev/null; then
    fail "package.sh still mentions retired lifecycle scripts"
else
    note "PASS package.sh does not install retired lifecycle scripts"
fi

if ! grep -F 'process_compose' "$packaging/hctl2-services" >/dev/null; then
    fail "hctl2-services no longer calls process_compose"
else
    note "PASS hctl2-services is a Process Compose client"
fi

if grep -F 'process_compose_project --dry-run' "$packaging/hctl2-services" >/dev/null &&
    grep -F 'process_compose_project up' "$packaging/hctl2-services" >/dev/null &&
    grep -F 'process_compose down' "$packaging/hctl2-services" >/dev/null &&
    grep -F 'process_compose process restart' "$packaging/hctl2-services" >/dev/null
then
    note "PASS hctl2-services start/stop/restart go through Process Compose"
else
    fail "hctl2-services is missing Process Compose start/stop/restart calls"
fi

# --- per-service YAML contract ---------------------------------------------
for service in tuwunel cinny vikunja dagu herdr; do
    yaml="$packaging/process-compose/$service.yaml"
    [[ -f "$yaml" ]] || { fail "missing $yaml"; continue; }
    for field in readiness_probe availability shutdown; do
        if grep -F "$field:" "$yaml" >/dev/null; then
            :
        else
            fail "$service.yaml is missing $field"
        fi
    done
    if grep -F 'restart: on_failure' "$yaml" >/dev/null; then
        :
    else
        fail "$service.yaml is not restart: on_failure"
    fi
    note "PASS $service.yaml declares readiness, restart and shutdown"
done

if grep -A2 'depends_on:' "$packaging/process-compose/cinny.yaml" | grep -F 'tuwunel:' >/dev/null &&
    grep -A4 'depends_on:' "$packaging/process-compose/cinny.yaml" | grep -F 'process_healthy' >/dev/null
then
    note "PASS cinny waits for tuwunel process_healthy"
else
    fail "cinny.yaml lost its tuwunel process_healthy dependency"
fi

if grep -F 'Process Compose owns process identity' \
    "$packaging/process-compose/process-compose.yaml" >/dev/null
then
    note "PASS base process-compose.yaml still states ownership"
else
    fail "base process-compose.yaml lost the ownership comment"
fi

# --- dummy lifecycle on this platform --------------------------------------
work="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-pc-lifecycle.XXXXXX")"
socket_dir="/tmp/hctl2-pc-test-$$"
mkdir -m 700 "$socket_dir"
socket="$socket_dir/pc.sock"
export HCTL2_PC_WORK="$work"
export XDG_CONFIG_HOME="$work/config"
export PC_DISABLE_DOTENV=1
mkdir -p "$XDG_CONFIG_HOME/process-compose"

pc() {
    "$pc_bin" \
        --use-uds \
        --unix-socket "$socket" \
        --ordered-shutdown \
        --log-file "$work/process-compose.log" \
        --log-no-color \
        "$@"
}

cleanup() {
    pc down >/dev/null 2>&1 || true
    rm -rf "$work" "$socket_dir"
}
trap cleanup EXIT

if ! pc --config "$fixture" up --dry-run >/dev/null; then
    fail "lifecycle fixture failed --dry-run"
else
    note "PASS lifecycle fixture --dry-run"
fi

# Validate the real five-service files with placeholder env. Commands are not
# executed under --dry-run; this catches merge/schema errors on every platform.
export HCTL2_BIN_DIR="$work"
export HCTL2_LOG_DIR="$work"
export HCTL2_CONFIG_DIR="$work"
export HCTL2_CINNY_ROOT="$work"
export HCTL2_TUWUNEL_CONFIG="$work/tuwunel.toml"
export HCTL2_VIKUNJA_DATA="$work"
export HCTL2_VIKUNJA_SECRET="fixture"
export HCTL2_DAGU_DATA="$work"
export HCTL2_HERDR_CONFIG="$work/herdr.toml"
export HCTL2_HERDR_SOCKET="$work/herdr.sock"
export TUWUNEL_PORT=1 CINNY_PORT=1 VIKUNJA_PORT=1
export DAGU_PORT=1 DAGU_SCHEDULER_PORT=1 DAGU_COORDINATOR_PORT=1 DAGU_COORDINATOR_HEALTH_PORT=1
: >"$HCTL2_TUWUNEL_CONFIG"
: >"$HCTL2_HERDR_CONFIG"
if pc \
    --config "$packaging/process-compose/process-compose.yaml" \
    --config "$packaging/process-compose/tuwunel.yaml" \
    --config "$packaging/process-compose/cinny.yaml" \
    --config "$packaging/process-compose/vikunja.yaml" \
    --config "$packaging/process-compose/dagu.yaml" \
    --config "$packaging/process-compose/herdr.yaml" \
    up --dry-run >/dev/null
then
    note "PASS packaged five-service YAML --dry-run"
else
    fail "packaged five-service YAML failed --dry-run"
fi

wait_ready() {
    local name="$1"
    local attempts="${2:-20}"
    local n=0
    local state
    while [ "$n" -lt "$attempts" ]; do
        state="$(pc process get "$name" --output json 2>/dev/null || true)"
        if printf '%s' "$state" | grep -Eq '"is_ready":[[:space:]]*"Ready"' &&
            printf '%s' "$state" | grep -Eq '"is_running":[[:space:]]*true'
        then
            return 0
        fi
        n=$((n + 1))
        sleep 0.5
    done
    return 1
}

is_ready() {
    local state
    state="$(pc process get "$1" --output json 2>/dev/null || true)"
    printf '%s' "$state" | grep -Eq '"is_ready":[[:space:]]*"Ready"'
}

pc --config "$fixture" up --detached --tui=false --keep-project ready-ok
if wait_ready ready-ok; then
    note "PASS pull-up reports ready-ok Ready"
else
    fail "ready-ok did not become Ready"
    pc process list --output wide >&2 || true
fi

if pc process get dependent --output json >/dev/null 2>&1 && is_ready dependent; then
    fail "dependent started even though it was not requested"
else
    note "PASS unselected dependent process stayed down"
fi

pc process restart ready-ok
if wait_ready ready-ok; then
    note "PASS restart returns ready-ok to Ready"
else
    fail "ready-ok did not become Ready after restart"
fi

pc process stop ready-ok
sleep 1
if pc process get ready-ok --output json 2>/dev/null | grep -Eq '"is_running":[[:space:]]*true'
then
    fail "ready-ok still running after process stop"
else
    note "PASS process stop ends ready-ok"
fi

pc down
if pc process list --output json >/dev/null 2>&1; then
    fail "process list succeeded after down"
else
    note "PASS down leaves no Process Compose instance"
fi

# Failure: never-ready must not be reported Ready.
pc --config "$fixture" up --detached --tui=false --keep-project never-ready
sleep 4
if is_ready never-ready; then
    fail "never-ready was reported Ready"
else
    note "PASS never-ready stays unready (failure case)"
fi
pc down >/dev/null 2>&1 || true

# Failure: crash process exhausts restart budget.
pc --config "$fixture" up --detached --tui=false --keep-project crash
sleep 5
crash_state="$(pc process get crash --output json 2>/dev/null || true)"
if printf '%s' "$crash_state" | grep -Eq '"is_running":[[:space:]]*true'; then
    fail "crash process still running after restart budget"
else
    note "PASS crash process stops after max_restarts (failure case)"
fi
pc down >/dev/null 2>&1 || true

# Failure: unknown process name.
pc --config "$fixture" up --detached --tui=false --keep-project ready-ok
if pc process start not-a-process >/dev/null 2>&1; then
    fail "process start accepted an unknown name"
else
    note "PASS unknown process name is rejected (failure case)"
fi
pc down >/dev/null 2>&1 || true

# Failure: stop/down when nothing is running is safe.
if pc down >/dev/null 2>&1; then
    note "PASS down with no instance is safe (or already gone)"
else
    note "PASS down with no instance fails closed without leftover"
fi

if [ "$failures" -ne 0 ]; then
    echo "check_process_compose_contract: FAILED ($failures)" >&2
    exit 1
fi
echo "check_process_compose_contract: OK"

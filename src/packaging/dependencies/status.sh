#!/usr/bin/env bash
# Installed dependency lifecycle status implementation.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

ensure_layout

failures=0

check_process() {
    local name="$1"
    local executable="$2"
    local port="$3"
    local path="$4"
    local url="http://127.0.0.1:$port$path"
    local pid

    if ! component_running "$name" "$executable"; then
        printf '%-12s stopped\n' "$name"
        failures=$((failures + 1))
        return
    fi
    pid="$(read_component_pid "$name")"
    if http_status_ok 127.0.0.1 "$port" "$path"; then
        printf '%-12s ready   pid=%s  %s\n' "$name" "$pid" "$url"
    else
        printf '%-12s running pid=%s  endpoint-not-ready\n' "$name" "$pid"
        failures=$((failures + 1))
    fi
}

check_herdr() {
    local binary="$P0_BIN_DIR/herdr"
    local socket
    local pid

    socket="$(herdr_socket_path)"

    if [[ ! -x "$binary" || ! -S "$socket" ]]; then
        printf '%-12s stopped\n' herdr
        failures=$((failures + 1))
        return
    fi
    pid="$(read_component_pid herdr)" || {
        printf '%-12s unmanaged-socket\n' herdr
        failures=$((failures + 1))
        return
    }
    if pid_matches_executable "$pid" "$binary" && \
        run_herdr "$binary" status server >/dev/null 2>&1; then
        printf '%-12s ready   pid=%s  socket=%s\n' herdr "$pid" "$socket"
    else
        printf '%-12s unhealthy\n' herdr
        failures=$((failures + 1))
    fi
}

check_process tuwunel "$P0_BIN_DIR/tuwunel" "$TUWUNEL_PORT" /_tuwunel/server_version
check_process cinny "$P0_BIN_DIR/static-web-server" "$CINNY_PORT" /config.json
check_process vikunja "$P0_BIN_DIR/vikunja" "$VIKUNJA_PORT" /api/v1/info
check_process dagu "$P0_BIN_DIR/dagu" "$DAGU_PORT" /api/v1/health
check_herdr

exit "$failures"

#!/usr/bin/env bash
# Installed dependency lifecycle stop implementation.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

ensure_layout

stop_herdr() {
    local binary="$P0_BIN_DIR/herdr"
    local socket
    local pid

    socket="$(herdr_socket_path)"

    if [[ ! -x "$binary" ]]; then
        note "herdr is not installed"
        return
    fi
    pid="$(read_component_pid herdr)" || {
        [[ ! -e "$socket" ]] || die "Herdr socket exists without a managed pid"
        note "herdr is not running"
        return
    }
    pid_matches_executable "$pid" "$binary" || die "refusing to stop a foreign Herdr process"
    if [[ -S "$socket" ]]; then
        run_herdr "$binary" server stop
    else
        kill -TERM "$pid"
    fi
    for _ in {1..100}; do
        if ! kill -0 "$pid" 2>/dev/null; then
            rm -f -- "$(pid_file herdr)"
            [[ ! -e "$socket" ]] || die "Herdr stopped but left its API socket behind"
            note "herdr stopped"
            return
        fi
        sleep 0.1
    done
    die "herdr did not stop; inspect $P0_LOG_DIR/herdr.log"
}

stop_component() {
    case "$1" in
        tuwunel) stop_background tuwunel "$P0_BIN_DIR/tuwunel" ;;
        cinny) stop_background cinny "$P0_BIN_DIR/static-web-server" ;;
        vikunja) stop_background vikunja "$P0_BIN_DIR/vikunja" ;;
        dagu) stop_background dagu "$P0_BIN_DIR/dagu" ;;
        herdr) stop_herdr ;;
        *) die "unknown component: $1" ;;
    esac
}

if (($# == 0)); then
    set -- herdr dagu vikunja cinny tuwunel
fi

for component in "$@"; do
    stop_component "$component"
done

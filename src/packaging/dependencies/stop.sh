#!/usr/bin/env bash
# Installed dependency lifecycle stop implementation.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

ensure_layout

stop_tmux() {
    local binary="$P0_BIN_DIR/tmux"
    local socket
    local pid

    socket="$(tmux_socket_path)"

    if [[ ! -x "$binary" || ! -S "$socket" ]]; then
        note "tmux is not running"
        return
    fi
    pid="$(read_component_pid tmux)" || die "tmux socket exists without a managed pid"
    pid_matches_executable "$pid" "$binary" || die "refusing to stop a foreign tmux server"
    "$binary" -S "$socket" kill-server
    rm -f -- "$socket" "$(pid_file tmux)"
    note "tmux stopped"
}

stop_component() {
    case "$1" in
        tuwunel) stop_background tuwunel "$P0_BIN_DIR/tuwunel" ;;
        vikunja) stop_background vikunja "$P0_BIN_DIR/vikunja" ;;
        dagu) stop_background dagu "$P0_BIN_DIR/dagu" ;;
        tmux) stop_tmux ;;
        *) die "unknown component: $1" ;;
    esac
}

if (($# == 0)); then
    set -- tmux dagu vikunja tuwunel
fi

for component in "$@"; do
    stop_component "$component"
done

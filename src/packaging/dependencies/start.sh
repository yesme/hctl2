#!/usr/bin/env bash
# Installed dependency lifecycle start implementation.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

ensure_layout

start_tuwunel() {
    local binary="$P0_BIN_DIR/tuwunel"
    local config="$P0_CONFIG_DIR/tuwunel.toml"
    local data="$P0_DATA_DIR/tuwunel"
    local registration_token_file="$P0_CONFIG_DIR/tuwunel-registration-token"
    local registration_token

    [[ -x "$binary" ]] || die "Tuwunel is not installed; run bootstrap.sh"
    if component_running tuwunel "$binary"; then
        note "tuwunel already running"
        return
    fi
    ensure_tcp_port_free "$TUWUNEL_PORT"
    mkdir -p "$data"
    write_secret_once "$registration_token_file"
    read -r registration_token <"$registration_token_file"
    cat >"$config" <<EOF
[global]
server_name = "hctl2.localhost"
database_path = "$data"
address = ["127.0.0.1"]
port = $TUWUNEL_PORT
allow_registration = true
registration_token = "$registration_token"
allow_encryption = false
allow_federation = false
trusted_servers = []
error_on_unknown_config_opts = true
log_colors = false
log_journald = false
EOF

    start_background tuwunel "$binary" env TUWUNEL_CONFIG="$config" "$binary"
    wait_http tuwunel 127.0.0.1 "$TUWUNEL_PORT" /_tuwunel/server_version
}

start_vikunja() {
    local binary="$P0_BIN_DIR/vikunja"
    local data="$P0_DATA_DIR/vikunja"
    local secret_file="$P0_CONFIG_DIR/vikunja-secret"
    local secret

    [[ -x "$binary" ]] || die "Vikunja is not installed; run bootstrap.sh"
    if component_running vikunja "$binary"; then
        note "vikunja already running"
        return
    fi
    ensure_tcp_port_free "$VIKUNJA_PORT"
    mkdir -p "$data"
    write_secret_once "$secret_file"
    read -r secret <"$secret_file"

    start_background vikunja "$binary" env \
        VIKUNJA_SERVICE_ROOTPATH="$data" \
        VIKUNJA_SERVICE_INTERFACE="127.0.0.1:$VIKUNJA_PORT" \
        VIKUNJA_SERVICE_PUBLICURL="http://127.0.0.1:$VIKUNJA_PORT/" \
        VIKUNJA_SERVICE_SECRET="$secret" \
        VIKUNJA_DATABASE_TYPE="sqlite" \
        VIKUNJA_DATABASE_PATH="$data/vikunja.db" \
        VIKUNJA_CORS_ENABLE="false" \
        VIKUNJA_LOG_STANDARD="stdout" \
        VIKUNJA_LOG_HTTP="stdout" \
        "$binary" web
    wait_http vikunja 127.0.0.1 "$VIKUNJA_PORT" /api/v1/info
}

start_dagu() {
    local binary="$P0_BIN_DIR/dagu"
    local data="$P0_DATA_DIR/dagu"

    [[ -x "$binary" ]] || die "Dagu is not installed; run bootstrap.sh"
    if component_running dagu "$binary"; then
        note "dagu already running"
        return
    fi
    ensure_tcp_port_free "$DAGU_PORT"
    ensure_tcp_port_free "$DAGU_SCHEDULER_PORT"
    ensure_tcp_port_free "$DAGU_COORDINATOR_PORT"
    ensure_tcp_port_free "$DAGU_COORDINATOR_HEALTH_PORT"
    mkdir -p "$data"

    start_background dagu "$binary" env \
        DAGU_HOME="$data" \
        DAGU_HOST="127.0.0.1" \
        DAGU_PORT="$DAGU_PORT" \
        DAGU_SCHEDULER_PORT="$DAGU_SCHEDULER_PORT" \
        DAGU_COORDINATOR_HOST="127.0.0.1" \
        DAGU_COORDINATOR_PORT="$DAGU_COORDINATOR_PORT" \
        DAGU_COORDINATOR_HEALTH_PORT="$DAGU_COORDINATOR_HEALTH_PORT" \
        DAGU_AUTH_MODE="none" \
        DAGU_TERMINAL_ENABLED="false" \
        "$binary" start-all
    wait_http dagu 127.0.0.1 "$DAGU_PORT" /api/v1/health
}

start_tmux() {
    local binary="$P0_BIN_DIR/tmux"
    local runtime="$P0_RUNTIME_DIR/tmux"
    local socket="$runtime/tmux.sock"
    local pid

    [[ -x "$binary" ]] || die "tmux is not installed; run bootstrap.sh"
    mkdir -p "$runtime"
    chmod 700 "$runtime"
    if "$binary" -S "$socket" has-session -t "$TMUX_SESSION" 2>/dev/null; then
        note "tmux already running"
        return
    fi

    umask 077
    "$binary" -S "$socket" -f /dev/null new-session -d -s "$TMUX_SESSION" -n runtime \
        "exec sh -c 'while :; do sleep 3600; done'"
    pid="$("$binary" -S "$socket" display-message -p '#{pid}')"
    [[ "$pid" =~ ^[1-9][0-9]*$ ]] || die "tmux did not report a server pid"
    printf '%s\n' "$pid" >"$(pid_file tmux)"
    note "tmux started with pid $pid and socket $socket"
}

start_component() {
    case "$1" in
        tuwunel) start_tuwunel ;;
        vikunja) start_vikunja ;;
        dagu) start_dagu ;;
        tmux) start_tmux ;;
        *) die "unknown component: $1" ;;
    esac
}

if (($# == 0)); then
    set -- tuwunel vikunja dagu tmux
fi

for component in "$@"; do
    start_component "$component"
done

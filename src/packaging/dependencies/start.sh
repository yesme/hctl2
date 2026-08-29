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

    [[ -x "$binary" ]] || die "Tuwunel is missing; reinstall the HCTL2 package"
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

show_cinny_access() {
    local registration_token_file="$P0_CONFIG_DIR/tuwunel-registration-token"
    local registration_token

    note "Chatroom client: http://127.0.0.1:$CINNY_PORT/"
    if [[ -f "$registration_token_file" ]]; then
        read -r registration_token <"$registration_token_file"
        note "Tuwunel registration token: $registration_token"
    fi
}

start_cinny() {
    local binary="$P0_BIN_DIR/static-web-server"

    start_tuwunel
    [[ -x "$binary" ]] || die "static-web-server is not installed"
    [[ -f "$P0_CINNY_ROOT/index.html" ]] || \
        die "Cinny is not installed: $P0_CINNY_ROOT"
    if component_running cinny "$binary"; then
        note "cinny already running"
        show_cinny_access
        return
    fi
    ensure_tcp_port_free "$CINNY_PORT"

    start_background cinny "$binary" \
        "$binary" \
        --host 127.0.0.1 \
        --port "$CINNY_PORT" \
        --root "$P0_CINNY_ROOT" \
        --compression false \
        --directory-listing false \
        --cache-control-headers false \
        --log-level error
    wait_http cinny 127.0.0.1 "$CINNY_PORT" /config.json
    show_cinny_access
}

start_vikunja() {
    local binary="$P0_BIN_DIR/vikunja"
    local data="$P0_DATA_DIR/vikunja"
    local secret_file="$P0_CONFIG_DIR/vikunja-secret"
    local secret

    [[ -x "$binary" ]] || die "Vikunja is missing; reinstall the HCTL2 package"
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

    [[ -x "$binary" ]] || die "Dagu is missing; reinstall the HCTL2 package"
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

start_herdr() {
    local binary="$P0_BIN_DIR/herdr"
    local socket

    [[ -x "$binary" ]] || die "Herdr is missing; reinstall the HCTL2 package"
    mkdir -p "$P0_CONFIG_DIR/herdr" "$P0_DATA_DIR/herdr" "$P0_RUNTIME_DIR/herdr"
    chmod 700 "$P0_CONFIG_DIR/herdr" "$P0_DATA_DIR/herdr" "$P0_RUNTIME_DIR/herdr"
    socket="$(herdr_socket_path)"
    if component_running herdr "$binary"; then
        run_herdr "$binary" status server >/dev/null || \
            die "managed Herdr process is running but its API is unavailable"
        note "herdr already running"
        return
    fi
    [[ ! -e "$socket" ]] || die "refusing to replace an unmanaged Herdr socket: $socket"

    start_background herdr "$binary" env \
        HERDR_CONFIG_PATH="$P0_CONFIG_DIR/herdr/config.toml" \
        HERDR_SOCKET_PATH="$socket" \
        XDG_CONFIG_HOME="$P0_CONFIG_DIR" \
        XDG_STATE_HOME="$P0_DATA_DIR/herdr" \
        "$binary" server
    for _ in {1..100}; do
        if [[ -S "$socket" ]] && run_herdr "$binary" status server >/dev/null 2>&1; then
            note "herdr API is ready at $socket"
            return
        fi
        sleep 0.1
    done
    tail -n 60 "$P0_LOG_DIR/herdr.log" >&2 || true
    die "herdr API did not become ready at $socket"
}

start_component() {
    case "$1" in
        tuwunel) start_tuwunel ;;
        cinny) start_cinny ;;
        vikunja) start_vikunja ;;
        dagu) start_dagu ;;
        herdr) start_herdr ;;
        *) die "unknown component: $1" ;;
    esac
}

if (($# == 0)); then
    start_all=1
    set -- tuwunel cinny vikunja dagu herdr
else
    start_all=0
fi
readonly start_all

for component in "$@"; do
    start_component "$component"
done

if ((start_all)); then
    note "browser clients:"
    note "  Chatroom  http://127.0.0.1:$CINNY_PORT/"
    note "  Kanban    http://127.0.0.1:$VIKUNJA_PORT/"
    note "  Workflow  http://127.0.0.1:$DAGU_PORT/"
fi

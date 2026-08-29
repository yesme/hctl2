#!/usr/bin/env bash
# Shared installed-runtime lifecycle helpers.

set -euo pipefail

P0_SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly P0_SCRIPT_DIR

P0_VERSIONS_FILE="$P0_SCRIPT_DIR/versions.sh"
P0_PLATFORM_FILE="$P0_SCRIPT_DIR/platform.sh"
readonly P0_VERSIONS_FILE P0_PLATFORM_FILE
[[ -f "$P0_VERSIONS_FILE" ]] || {
    printf 'error: runtime versions file is missing: %s\n' "$P0_VERSIONS_FILE" >&2
    exit 1
}
[[ -f "$P0_PLATFORM_FILE" ]] || {
    printf 'error: runtime platform file is missing: %s\n' "$P0_PLATFORM_FILE" >&2
    exit 1
}
source "$P0_VERSIONS_FILE"
source "$P0_PLATFORM_FILE"

: "${HCTL2_INSTALL_ROOT:?hctl2-services must provide HCTL2_INSTALL_ROOT}"
[[ "$HCTL2_INSTALL_ROOT" == /* && "$HCTL2_INSTALL_ROOT" != "/" ]] || {
    printf 'error: HCTL2_INSTALL_ROOT must be an absolute, non-root path\n' >&2
    exit 2
}
readonly P0_INSTALL_ROOT="$HCTL2_INSTALL_ROOT"

if [[ -n "${HCTL2_STATE_ROOT:-}" ]]; then
    P0_ROOT="$HCTL2_STATE_ROOT"
elif [[ -n "${XDG_STATE_HOME:-}" ]]; then
    P0_ROOT="$XDG_STATE_HOME/hctl2"
else
    P0_ROOT="${HOME:?HOME must be set}/.local/state/hctl2"
fi

[[ "$P0_ROOT" == /* && "$P0_ROOT" != "/" ]] || {
    printf 'error: HCTL2_STATE_ROOT must be an absolute, non-root path\n' >&2
    exit 2
}

readonly P0_ROOT
readonly P0_BIN_DIR="$P0_INSTALL_ROOT/libexec/hctl2"
readonly P0_CONFIG_DIR="$P0_ROOT/config"
readonly P0_DATA_DIR="$P0_ROOT/data"
readonly P0_LOG_DIR="$P0_ROOT/logs"
readonly P0_PID_DIR="$P0_ROOT/pids"
readonly P0_RUNTIME_DIR="$P0_ROOT/runtime"
readonly P0_DEPENDENCY_LIBRARY_DIR="$P0_INSTALL_ROOT/lib/hctl2/vendor"
readonly P0_CINNY_ROOT="$P0_INSTALL_ROOT/share/hctl2/chatroom/cinny"

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

note() {
    printf 'hctl2: %s\n' "$*"
}

ensure_layout() {
    mkdir -p "$P0_CONFIG_DIR" "$P0_DATA_DIR" "$P0_LOG_DIR" "$P0_PID_DIR" "$P0_RUNTIME_DIR"
    [[ -d "$P0_BIN_DIR" ]] || die "installed dependency directory is missing: $P0_BIN_DIR"
    chmod 700 "$P0_ROOT" "$P0_CONFIG_DIR" "$P0_DATA_DIR" "$P0_PID_DIR" "$P0_RUNTIME_DIR"
}

canonical_existing_path() {
    local path="$1"
    local directory
    local name

    directory="$(cd -- "$(dirname -- "$path")" && pwd -P)" || return 1
    name="$(basename -- "$path")"
    printf '%s/%s\n' "$directory" "$name"
}

pid_file() {
    printf '%s/%s.pid\n' "$P0_PID_DIR" "$1"
}

read_component_pid() {
    local name="$1"
    local path
    local pid

    path="$(pid_file "$name")"
    [[ -f "$path" ]] || return 1
    read -r pid <"$path"
    [[ "$pid" =~ ^[1-9][0-9]*$ ]] || return 1
    printf '%s\n' "$pid"
}

pid_matches_executable() {
    local pid="$1"
    local expected="$2"
    local actual
    local resolved_expected

    kill -0 "$pid" 2>/dev/null || return 1
    actual="$(platform_executable_for_pid "$pid")" || return 1
    actual="$(canonical_existing_path "$actual")" || return 1
    resolved_expected="$(canonical_existing_path "$expected")" || return 1
    [[ "$actual" == "$resolved_expected" ]]
}

component_running() {
    local name="$1"
    local executable="$2"
    local pid

    pid="$(read_component_pid "$name")" || return 1
    pid_matches_executable "$pid" "$executable"
}

start_background() {
    local name="$1"
    local executable="$2"
    shift 2
    local log="$P0_LOG_DIR/$name.log"
    local pid

    if component_running "$name" "$executable"; then
        note "$name already running"
        return
    fi

    umask 077
    nohup "$@" >>"$log" 2>&1 </dev/null &
    pid=$!
    printf '%s\n' "$pid" >"$(pid_file "$name")"

    for _ in {1..50}; do
        if pid_matches_executable "$pid" "$executable"; then
            note "$name started with pid $pid"
            return
        fi
        kill -0 "$pid" 2>/dev/null || break
        sleep 0.1
    done

    tail -n 40 "$log" >&2 || true
    die "$name did not stay running"
}

stop_background() {
    local name="$1"
    local executable="$2"
    local pid

    pid="$(read_component_pid "$name")" || {
        note "$name is not running"
        return
    }

    if ! kill -0 "$pid" 2>/dev/null; then
        rm -f -- "$(pid_file "$name")"
        note "$name is not running; removed stale pid file"
        return
    fi
    pid_matches_executable "$pid" "$executable" || \
        die "refusing to signal stale or foreign pid $pid for $name"

    kill -TERM "$pid"
    for _ in {1..100}; do
        if ! kill -0 "$pid" 2>/dev/null; then
            rm -f -- "$(pid_file "$name")"
            note "$name stopped"
            return
        fi
        sleep 0.1
    done

    die "$name did not stop after SIGTERM; inspect $P0_LOG_DIR/$name.log"
}

ensure_tcp_port_free() {
    local port="$1"

    if (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null; then
        die "TCP port $port is already in use"
    fi
}

http_status_ok() {
    local host="$1"
    local port="$2"
    local path="$3"
    local protocol
    local status

    if ! { exec 3<>"/dev/tcp/$host/$port"; } 2>/dev/null; then
        return 1
    fi
    if ! printf 'GET %s HTTP/1.1\r\nHost: %s:%s\r\nConnection: close\r\n\r\n' \
        "$path" "$host" "$port" >&3; then
        exec 3>&-
        exec 3<&-
        return 1
    fi
    if ! IFS=' ' read -r -t 2 protocol status _ <&3; then
        exec 3>&-
        exec 3<&-
        return 1
    fi
    exec 3>&-
    exec 3<&-
    [[ "$protocol" == HTTP/* && "$status" =~ ^[23][0-9][0-9]$ ]]
}

wait_http() {
    local name="$1"
    local host="$2"
    local port="$3"
    local path="$4"
    local url="http://$host:$port$path"

    for _ in {1..120}; do
        if http_status_ok "$host" "$port" "$path"; then
            note "$name HTTP endpoint is ready: $url"
            return
        fi
        sleep 0.5
    done

    tail -n 60 "$P0_LOG_DIR/$name.log" >&2 || true
    die "$name HTTP endpoint did not become ready: $url"
}

write_secret_once() {
    local path="$1"

    [[ -f "$path" ]] && return
    umask 077
    head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n' >"$path"
    printf '\n' >>"$path"
}

file_mode() {
    platform_file_mode "$1"
}

herdr_socket_path() {
    platform_herdr_socket_path
}

run_herdr() {
    local binary="$1"
    shift

    env \
        HERDR_CONFIG_PATH="$P0_CONFIG_DIR/herdr/config.toml" \
        HERDR_SOCKET_PATH="$(herdr_socket_path)" \
        XDG_CONFIG_HOME="$P0_CONFIG_DIR" \
        XDG_STATE_HOME="$P0_DATA_DIR/herdr" \
        "$binary" "$@"
}

platform_configure_runtime

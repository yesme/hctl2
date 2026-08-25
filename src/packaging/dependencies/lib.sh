#!/usr/bin/env bash
# Shared build-cache and installed-runtime lifecycle helpers.

set -euo pipefail

P0_SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly P0_SCRIPT_DIR
# shellcheck source=versions.sh
source "$P0_SCRIPT_DIR/versions.sh"

if [[ -n "${HCTL2_INSTALL_ROOT:-}" ]]; then
    [[ "$HCTL2_INSTALL_ROOT" == /* && "$HCTL2_INSTALL_ROOT" != "/" ]] || {
        printf 'error: HCTL2_INSTALL_ROOT must be an absolute, non-root path\n' >&2
        exit 2
    }
    readonly P0_INSTALL_ROOT="$HCTL2_INSTALL_ROOT"
else
    readonly P0_INSTALL_ROOT=""
fi

if [[ -n "$P0_INSTALL_ROOT" ]]; then
    if [[ -n "${HCTL2_STATE_ROOT:-}" ]]; then
        P0_ROOT="$HCTL2_STATE_ROOT"
    elif [[ -n "${XDG_STATE_HOME:-}" ]]; then
        P0_ROOT="$XDG_STATE_HOME/hctl2"
    else
        P0_ROOT="${HOME:?HOME must be set}/.local/state/hctl2"
    fi
else
    if [[ -n "${HCTL2_BUILD_CACHE:-}" ]]; then
        P0_ROOT="$HCTL2_BUILD_CACHE"
    elif [[ -n "${XDG_CACHE_HOME:-}" ]]; then
        P0_ROOT="$XDG_CACHE_HOME/hctl2/dependencies"
    else
        P0_ROOT="${HOME:?HOME must be set}/.cache/hctl2/dependencies"
    fi
fi

if [[ "$P0_ROOT" != /* || "$P0_ROOT" == "/" ]]; then
    printf 'error: HCTL2_STATE_ROOT/HCTL2_BUILD_CACHE must be an absolute, non-root path\n' >&2
    exit 2
fi

readonly P0_ROOT
if [[ -n "$P0_INSTALL_ROOT" ]]; then
    readonly P0_BIN_DIR="$P0_INSTALL_ROOT/libexec/hctl2"
else
    readonly P0_BIN_DIR="$P0_ROOT/bin"
fi
readonly P0_CONFIG_DIR="$P0_ROOT/config"
readonly P0_DATA_DIR="$P0_ROOT/data"
readonly P0_DOWNLOAD_DIR="$P0_ROOT/downloads"
readonly P0_LOG_DIR="$P0_ROOT/logs"
readonly P0_MANIFEST_DIR="$P0_ROOT/manifests"
readonly P0_PID_DIR="$P0_ROOT/pids"
readonly P0_RUNTIME_DIR="$P0_ROOT/runtime"
readonly P0_TMP_DIR="$P0_ROOT/tmp"
readonly P0_VENDOR_DIR="$P0_ROOT/vendor"
if [[ -n "$P0_INSTALL_ROOT" ]]; then
    readonly P0_DEPENDENCY_LIBRARY_DIR="$P0_INSTALL_ROOT/lib/hctl2/vendor"
else
    readonly P0_DEPENDENCY_LIBRARY_DIR="$P0_VENDOR_DIR/tmux-sysroot/usr/lib/x86_64-linux-gnu"
fi

if [[ -n "${LD_LIBRARY_PATH:-}" ]]; then
    export LD_LIBRARY_PATH="$P0_DEPENDENCY_LIBRARY_DIR:$LD_LIBRARY_PATH"
else
    export LD_LIBRARY_PATH="$P0_DEPENDENCY_LIBRARY_DIR"
fi

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

note() {
    printf 'hctl2: %s\n' "$*"
}

ensure_layout() {
    mkdir -p \
        "$P0_CONFIG_DIR" \
        "$P0_DATA_DIR" \
        "$P0_LOG_DIR" \
        "$P0_PID_DIR" \
        "$P0_RUNTIME_DIR"
    if [[ -n "$P0_INSTALL_ROOT" ]]; then
        [[ -d "$P0_BIN_DIR" ]] || die "installed dependency directory is missing: $P0_BIN_DIR"
    else
        mkdir -p \
            "$P0_BIN_DIR" \
            "$P0_DOWNLOAD_DIR" \
            "$P0_MANIFEST_DIR" \
            "$P0_TMP_DIR" \
            "$P0_VENDOR_DIR"
    fi
    chmod 700 "$P0_ROOT" "$P0_CONFIG_DIR" "$P0_DATA_DIR" "$P0_PID_DIR" "$P0_RUNTIME_DIR"
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"
}

require_linux_amd64() {
    [[ "$(uname -s)" == "Linux" ]] || die "this dependency packager currently supports Linux only"
    [[ "$(uname -m)" == "x86_64" ]] || die "this dependency packager currently supports x86_64 only"
}

verify_sha256() {
    local path="$1"
    local expected="$2"
    local actual

    actual="$(sha256sum "$path" | awk '{print $1}')"
    [[ "$actual" == "$expected" ]] || die "checksum mismatch for $path: expected $expected, got $actual"
}

download_verified() {
    local name="$1"
    local url="$2"
    local expected="$3"
    local destination="$4"
    local partial="$destination.partial"

    if [[ -f "$destination" ]]; then
        verify_sha256 "$destination" "$expected"
        note "$name asset already verified"
        return
    fi

    note "downloading $name"
    curl --fail --location --retry 3 --continue-at - --output "$partial" "$url"
    verify_sha256 "$partial" "$expected"
    mv -- "$partial" "$destination"
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
    actual="$(readlink -f "/proc/$pid/exe" 2>/dev/null)" || return 1
    resolved_expected="$(readlink -f "$expected")" || return 1
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

    if ! pid_matches_executable "$pid" "$executable"; then
        die "refusing to signal stale or foreign pid $pid for $name"
    fi

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
    local socket
    local protocol
    local status

    { exec {socket}<>"/dev/tcp/$host/$port"; } 2>/dev/null || return 1
    if ! printf 'GET %s HTTP/1.1\r\nHost: %s:%s\r\nConnection: close\r\n\r\n' \
        "$path" "$host" "$port" >&"$socket"; then
        exec {socket}>&-
        exec {socket}<&-
        return 1
    fi
    if ! IFS=' ' read -r -t 2 protocol status _ <&"$socket"; then
        exec {socket}>&-
        exec {socket}<&-
        return 1
    fi
    exec {socket}>&-
    exec {socket}<&-
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

    if [[ -f "$path" ]]; then
        return
    fi

    umask 077
    head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n' >"$path"
    printf '\n' >>"$path"
}

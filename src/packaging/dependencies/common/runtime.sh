#!/usr/bin/env bash
# Shared installed-runtime configuration and Process Compose client helpers.

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
readonly P0_RUNTIME_DIR="$P0_ROOT/runtime"
readonly P0_DEPENDENCY_LIBRARY_DIR="$P0_INSTALL_ROOT/lib/hctl2/vendor"
readonly P0_CINNY_ROOT="$P0_INSTALL_ROOT/share/hctl2/chatroom/cinny"
readonly P0_PROCESS_COMPOSE_CONFIG_DIR="$P0_SCRIPT_DIR/process-compose"

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

note() {
    printf 'hctl2: %s\n' "$*"
}

write_secret_once() {
    local path="$1"

    [[ -f "$path" ]] && return
    umask 077
    head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n' >"$path"
    printf '\n' >>"$path"
}

prepare_runtime() {
    local component
    local registration_token
    local tuwunel_config
    local tuwunel_config_tmp
    local vikunja_secret

    mkdir -p \
        "$P0_CONFIG_DIR/process-compose" \
        "$P0_CONFIG_DIR/herdr" \
        "$P0_DATA_DIR/tuwunel" \
        "$P0_DATA_DIR/vikunja" \
        "$P0_DATA_DIR/dagu" \
        "$P0_DATA_DIR/herdr" \
        "$P0_LOG_DIR" \
        "$P0_RUNTIME_DIR/herdr"
    chmod 700 \
        "$P0_ROOT" "$P0_CONFIG_DIR" "$P0_CONFIG_DIR/process-compose" \
        "$P0_CONFIG_DIR/herdr" "$P0_DATA_DIR" "$P0_DATA_DIR/tuwunel" \
        "$P0_DATA_DIR/vikunja" "$P0_DATA_DIR/dagu" "$P0_DATA_DIR/herdr" \
        "$P0_RUNTIME_DIR" "$P0_RUNTIME_DIR/herdr"

    for component in \
        tuwunel static-web-server vikunja dagu herdr process-compose; do
        [[ -x "$P0_BIN_DIR/$component" ]] || \
            die "$component is missing; reinstall the HCTL2 package"
    done
    [[ -f "$P0_CINNY_ROOT/index.html" ]] || die "Cinny is not installed: $P0_CINNY_ROOT"

    write_secret_once "$P0_CONFIG_DIR/tuwunel-registration-token"
    write_secret_once "$P0_CONFIG_DIR/vikunja-secret"
    IFS= read -r registration_token <"$P0_CONFIG_DIR/tuwunel-registration-token"
    IFS= read -r vikunja_secret <"$P0_CONFIG_DIR/vikunja-secret"

    tuwunel_config="$P0_CONFIG_DIR/tuwunel.toml"
    tuwunel_config_tmp="$tuwunel_config.tmp.$$"
    umask 077
    {
        printf '%s\n' \
            '[global]' \
            'server_name = "hctl2.localhost"' \
            "database_path = \"$P0_DATA_DIR/tuwunel\"" \
            'address = ["127.0.0.1"]' \
            "port = $TUWUNEL_PORT" \
            'allow_registration = true' \
            "registration_token = \"$registration_token\"" \
            'allow_encryption = false' \
            'allow_federation = false' \
            'trusted_servers = []' \
            'error_on_unknown_config_opts = true' \
            'log_colors = false' \
            'log_journald = false'
    } >"$tuwunel_config_tmp"
    mv -f -- "$tuwunel_config_tmp" "$tuwunel_config"

    export HCTL2_BIN_DIR="$P0_BIN_DIR"
    export HCTL2_CONFIG_DIR="$P0_CONFIG_DIR"
    export HCTL2_LOG_DIR="$P0_LOG_DIR"
    export HCTL2_CINNY_ROOT="$P0_CINNY_ROOT"
    export HCTL2_TUWUNEL_CONFIG="$tuwunel_config"
    export HCTL2_VIKUNJA_DATA="$P0_DATA_DIR/vikunja"
    export HCTL2_VIKUNJA_SECRET="$vikunja_secret"
    export HCTL2_DAGU_DATA="$P0_DATA_DIR/dagu"
    export HCTL2_HERDR_CONFIG="$P0_CONFIG_DIR/herdr/config.toml"
    export HCTL2_HERDR_DATA="$P0_DATA_DIR/herdr"
    export HCTL2_HERDR_SOCKET="$(platform_herdr_socket_path)"
    export HCTL2_PROCESS_COMPOSE_SOCKET="$(platform_process_compose_socket_path)"
    export TUWUNEL_PORT CINNY_PORT VIKUNJA_PORT DAGU_PORT
    export DAGU_SCHEDULER_PORT DAGU_COORDINATOR_PORT DAGU_COORDINATOR_HEALTH_PORT
}

readonly -a P0_COMPONENTS=(tuwunel cinny vikunja dagu herdr)
readonly -a P0_PROCESS_COMPOSE_FILES=(
    "$P0_PROCESS_COMPOSE_CONFIG_DIR/process-compose.yaml"
    "$P0_PROCESS_COMPOSE_CONFIG_DIR/tuwunel.yaml"
    "$P0_PROCESS_COMPOSE_CONFIG_DIR/cinny.yaml"
    "$P0_PROCESS_COMPOSE_CONFIG_DIR/vikunja.yaml"
    "$P0_PROCESS_COMPOSE_CONFIG_DIR/dagu.yaml"
    "$P0_PROCESS_COMPOSE_CONFIG_DIR/herdr.yaml"
)

validate_components() {
    local requested
    local known
    local matched

    for requested in "$@"; do
        matched=0
        for known in "${P0_COMPONENTS[@]}"; do
            if [[ "$requested" == "$known" ]]; then
                matched=1
                break
            fi
        done
        ((matched)) || die "unknown component: $requested"
    done
}

process_compose() {
    env \
        XDG_CONFIG_HOME="$P0_CONFIG_DIR" \
        PC_DISABLE_DOTENV=1 \
        "$P0_BIN_DIR/process-compose" \
        --use-uds \
        --unix-socket "$HCTL2_PROCESS_COMPOSE_SOCKET" \
        --ordered-shutdown \
        --log-file "$P0_LOG_DIR/process-compose.log" \
        --log-no-color \
        "$@"
}

process_compose_project() {
    local file
    local -a config_arguments=()

    for file in "${P0_PROCESS_COMPOSE_FILES[@]}"; do
        [[ -f "$file" ]] || die "Process Compose configuration is missing: $file"
        config_arguments+=(--config "$file")
    done
    process_compose "${config_arguments[@]}" "$@"
}

process_compose_alive() {
    process_compose process list --output json >/dev/null 2>&1
}

component_ready() {
    local state

    state="$(process_compose process get "$1" --output json 2>/dev/null)" || return 1
    grep -Eq '"is_running":[[:space:]]*true' <<<"$state" && \
        grep -Eq '"is_ready":[[:space:]]*"Ready"' <<<"$state"
}

component_running() {
    local state

    state="$(process_compose process get "$1" --output json 2>/dev/null)" || return 1
    grep -Eq '"is_running":[[:space:]]*true' <<<"$state"
}

wait_components_ready() {
    local component
    local attempt
    local all_ready

    for attempt in {1..240}; do
        all_ready=1
        for component in "$@"; do
            if ! component_ready "$component"; then
                all_ready=0
                break
            fi
        done
        ((all_ready)) && return
        sleep 0.5
    done

    process_compose process list --output wide >&2 || true
    die "Process Compose did not report all requested components ready"
}

file_mode() {
    platform_file_mode "$1"
}

run_herdr() {
    env \
        HERDR_CONFIG_PATH="$HCTL2_HERDR_CONFIG" \
        HERDR_SOCKET_PATH="$HCTL2_HERDR_SOCKET" \
        XDG_CONFIG_HOME="$P0_CONFIG_DIR" \
        XDG_STATE_HOME="$HCTL2_HERDR_DATA" \
        "$P0_BIN_DIR/herdr" "$@"
}

platform_configure_runtime

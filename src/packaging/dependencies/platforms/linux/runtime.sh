#!/usr/bin/env bash
# Linux installed-runtime hooks.

platform_configure_runtime() {
    if [[ -n "${LD_LIBRARY_PATH:-}" ]]; then
        export LD_LIBRARY_PATH="$P0_DEPENDENCY_LIBRARY_DIR:$LD_LIBRARY_PATH"
    else
        export LD_LIBRARY_PATH="$P0_DEPENDENCY_LIBRARY_DIR"
    fi
}

platform_file_mode() {
    stat -c '%a' "$1"
}

platform_herdr_socket_path() {
    printf '%s/herdr/herdr.sock\n' "$P0_RUNTIME_DIR"
}

platform_process_compose_socket_path() {
    local socket_directory="/tmp/hctl2-process-compose-$(id -u)"
    local state_key

    if [[ -e "$socket_directory" ]]; then
        [[ -d "$socket_directory" && ! -L "$socket_directory" ]] || \
            die "unsafe Process Compose socket directory: $socket_directory"
        [[ "$(stat -c '%u' "$socket_directory")" == "$(id -u)" ]] || \
            die "Process Compose socket directory has the wrong owner: $socket_directory"
    else
        mkdir -m 0700 "$socket_directory"
    fi
    chmod 0700 "$socket_directory"
    state_key="$(printf '%s' "$P0_ROOT" | sha256sum | awk '{print substr($1, 1, 16)}')"
    printf '%s/%s.sock\n' "$socket_directory" "$state_key"
}

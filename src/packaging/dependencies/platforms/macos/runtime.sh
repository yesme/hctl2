#!/usr/bin/env bash
# macOS installed-runtime hooks.

platform_configure_runtime() {
    : # Mach-O binaries use @loader_path; no build-host library path is inherited.
}

platform_file_mode() {
    stat -f '%Lp' "$1"
}

platform_short_socket_path() {
    local name="$1"
    local socket_directory="/tmp/hctl2-$name-$(id -u)"
    local state_key

    if [[ -e "$socket_directory" ]]; then
        [[ -d "$socket_directory" && ! -L "$socket_directory" ]] || \
            die "unsafe $name socket directory: $socket_directory"
        [[ "$(stat -f '%u' "$socket_directory")" == "$(id -u)" ]] || \
            die "$name socket directory has the wrong owner: $socket_directory"
    else
        mkdir -m 0700 "$socket_directory"
    fi
    chmod 0700 "$socket_directory"
    state_key="$(printf '%s' "$P0_ROOT" | shasum -a 256 | awk '{print substr($1, 1, 16)}')"
    printf '%s/%s.sock\n' "$socket_directory" "$state_key"
}

platform_herdr_socket_path() {
    platform_short_socket_path herdr
}

platform_process_compose_socket_path() {
    platform_short_socket_path process-compose
}

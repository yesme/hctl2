#!/usr/bin/env bash
# macOS installed-runtime hooks.

platform_configure_runtime() {
    : # Mach-O binaries use @loader_path; no build-host library path is inherited.
}

platform_executable_for_pid() {
    /usr/sbin/lsof -a -p "$1" -d txt -Fn 2>/dev/null | sed -n 's/^n//p' | sed -n '1p'
}

platform_file_mode() {
    stat -f '%Lp' "$1"
}

platform_tmux_socket_path() {
    local socket_directory="/tmp/hctl2-tmux-$(id -u)"
    local state_key

    if [[ -e "$socket_directory" ]]; then
        [[ -d "$socket_directory" && ! -L "$socket_directory" ]] || \
            die "unsafe tmux socket directory: $socket_directory"
        [[ "$(stat -f '%u' "$socket_directory")" == "$(id -u)" ]] || \
            die "tmux socket directory has the wrong owner: $socket_directory"
    else
        mkdir -m 0700 "$socket_directory"
    fi
    chmod 0700 "$socket_directory"
    state_key="$(printf '%s' "$P0_ROOT" | shasum -a 256 | awk '{print substr($1, 1, 16)}')"
    printf '%s/%s.sock\n' "$socket_directory" "$state_key"
}

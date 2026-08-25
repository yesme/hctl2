#!/usr/bin/env bash
# Linux installed-runtime hooks.

platform_configure_runtime() {
    if [[ -n "${LD_LIBRARY_PATH:-}" ]]; then
        export LD_LIBRARY_PATH="$P0_DEPENDENCY_LIBRARY_DIR:$LD_LIBRARY_PATH"
    else
        export LD_LIBRARY_PATH="$P0_DEPENDENCY_LIBRARY_DIR"
    fi
}

platform_executable_for_pid() {
    readlink -f "/proc/$1/exe"
}

platform_file_mode() {
    stat -c '%a' "$1"
}

platform_tmux_socket_path() {
    printf '%s/tmux/tmux.sock\n' "$P0_RUNTIME_DIR"
}

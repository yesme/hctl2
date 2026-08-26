#!/usr/bin/env bash
# Installed dependency seam smoke checks.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

ensure_layout

"$SCRIPT_DIR/status.sh"

note "checking Tuwunel version endpoint and plaintext-room policy"
http_status_ok 127.0.0.1 "$TUWUNEL_PORT" /_tuwunel/server_version
grep -Fx 'allow_encryption = false' "$P0_CONFIG_DIR/tuwunel.toml" >/dev/null

note "checking bundled Element Web client configuration"
http_status_ok 127.0.0.1 "$ELEMENT_WEB_PORT" /
http_status_ok 127.0.0.1 "$ELEMENT_WEB_PORT" /config.json
grep -F '"base_url": "http://127.0.0.1:6167"' "$P0_ELEMENT_WEB_ROOT/config.json" >/dev/null
grep -F '"server_name": "hctl2.localhost"' "$P0_ELEMENT_WEB_ROOT/config.json" >/dev/null

note "checking Vikunja API discovery"
http_status_ok 127.0.0.1 "$VIKUNJA_PORT" /api/v1/info

note "checking Dagu health API"
http_status_ok 127.0.0.1 "$DAGU_PORT" /api/v1/health

note "checking tmux headless query, stable IDs, and owner-only socket"
readonly TMUX_SOCKET="$(tmux_socket_path)"
TMUX_MODE="$(file_mode "$TMUX_SOCKET")"
readonly TMUX_MODE
((10#$TMUX_MODE % 100 == 0)) || die "tmux socket exposes group/other permissions: $TMUX_MODE"
"$P0_BIN_DIR/tmux" -S "$TMUX_SOCKET" list-panes -t "$TMUX_SESSION" -F '#{session_id}|#{window_id}|#{pane_id}|#{pane_pid}' |
    grep -E '^\$[0-9]+\|@[0-9]+\|%[0-9]+\|[0-9]+$' >/dev/null

note "all four local dependency seams and the bundled Chatroom client are alive"

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

note "checking bundled Cinny client configuration"
http_status_ok 127.0.0.1 "$CINNY_PORT" /
http_status_ok 127.0.0.1 "$CINNY_PORT" /config.json
grep -F '"homeserverList": ["http://127.0.0.1:6167"]' "$P0_CINNY_ROOT/config.json" >/dev/null
grep -F '"allowCustomHomeservers": false' "$P0_CINNY_ROOT/config.json" >/dev/null
grep -F '"enabled": true' "$P0_CINNY_ROOT/config.json" >/dev/null

note "checking Vikunja API discovery"
http_status_ok 127.0.0.1 "$VIKUNJA_PORT" /api/v1/info

note "checking Dagu health API"
http_status_ok 127.0.0.1 "$DAGU_PORT" /api/v1/health

note "checking Herdr protocol readback and owner-only socket"
readonly HERDR_SOCKET="$(herdr_socket_path)"
HERDR_MODE="$(file_mode "$HERDR_SOCKET")"
readonly HERDR_MODE
((10#$HERDR_MODE % 100 == 0)) || die "Herdr socket exposes group/other permissions: $HERDR_MODE"
HERDR_STATUS="$(run_herdr "$P0_BIN_DIR/herdr" status server)"
readonly HERDR_STATUS
grep -Fx "version: $HERDR_VERSION" <<<"$HERDR_STATUS" >/dev/null
grep -Fx "protocol: $HERDR_PROTOCOL" <<<"$HERDR_STATUS" >/dev/null
grep -Fx 'compatible: yes' <<<"$HERDR_STATUS" >/dev/null
HERDR_SNAPSHOT="$(run_herdr "$P0_BIN_DIR/herdr" api snapshot)"
readonly HERDR_SNAPSHOT
grep -F "\"protocol\":$HERDR_PROTOCOL" <<<"$HERDR_SNAPSHOT" >/dev/null
grep -F "\"version\":\"$HERDR_VERSION\"" <<<"$HERDR_SNAPSHOT" >/dev/null

note "all four local dependency seams and the bundled Chatroom client are alive"

#!/usr/bin/env bash
# Installed dependency seam smoke checks.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
# shellcheck source=common/runtime.sh
source "$SCRIPT_DIR/lib.sh"

prepare_runtime
process_compose_alive || die "Process Compose is not running"

for component in "${P0_COMPONENTS[@]}"; do
    component_ready "$component" || die "$component is not ready"
done

note "checking Tuwunel plaintext-room policy"
grep -Fx 'allow_encryption = false' "$P0_CONFIG_DIR/tuwunel.toml" >/dev/null

note "checking bundled Cinny client configuration"
grep -F '"homeserverList": ["http://127.0.0.1:6167"]' "$P0_CINNY_ROOT/config.json" >/dev/null
grep -F '"allowCustomHomeservers": false' "$P0_CINNY_ROOT/config.json" >/dev/null
grep -F '"enabled": true' "$P0_CINNY_ROOT/config.json" >/dev/null

note "checking Herdr protocol readback and owner-only socket"
readonly HERDR_MODE="$(file_mode "$HCTL2_HERDR_SOCKET")"
((10#$HERDR_MODE % 100 == 0)) || die "Herdr socket exposes group/other permissions: $HERDR_MODE"
readonly HERDR_STATUS="$(run_herdr status server)"
grep -Fx "version: $HERDR_VERSION" <<<"$HERDR_STATUS" >/dev/null
grep -Fx "protocol: $HERDR_PROTOCOL" <<<"$HERDR_STATUS" >/dev/null
grep -Fx 'compatible: yes' <<<"$HERDR_STATUS" >/dev/null
readonly HERDR_SNAPSHOT="$(run_herdr api snapshot)"
grep -F "\"protocol\":$HERDR_PROTOCOL" <<<"$HERDR_SNAPSHOT" >/dev/null
grep -F "\"version\":\"$HERDR_VERSION\"" <<<"$HERDR_SNAPSHOT" >/dev/null

note "all four local dependency seams and the bundled Chatroom client are ready"

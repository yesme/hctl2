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
grep -Fx 'status: running' <<<"$HERDR_STATUS" >/dev/null || die "Herdr server is not running"
grep -Fx "version: $HERDR_VERSION" <<<"$HERDR_STATUS" >/dev/null || die "Herdr server version differs from the lock"
grep -Fx "private_protocol: $HERDR_PROTOCOL" <<<"$HERDR_STATUS" >/dev/null || die "Herdr server protocol differs from the lock"
grep -Fx 'private_protocol_compatible: yes' <<<"$HERDR_STATUS" >/dev/null || die "Herdr server protocol is incompatible"
readonly HERDR_SNAPSHOT="$(run_herdr api snapshot)"
grep -F "\"protocol\":$HERDR_PROTOCOL" <<<"$HERDR_SNAPSHOT" >/dev/null
grep -F "\"version\":\"$HERDR_VERSION\"" <<<"$HERDR_SNAPSHOT" >/dev/null

note "checking bundled tmux on a private socket"
readonly TMUX_SMOKE_SOCKET="$(platform_tmux_socket_path)"
readonly TMUX_SMOKE_LOG="$P0_RUNTIME_DIR/tmux-smoke-attach.log"
rm -f -- "$TMUX_SMOKE_SOCKET" "$TMUX_SMOKE_LOG"
tmux_run() {
    "$P0_BIN_DIR/tmux" -S "$TMUX_SMOKE_SOCKET" "$@"
}
cleanup_tmux_smoke() {
    tmux_run kill-server >/dev/null 2>&1 || true
    rm -f -- "$TMUX_SMOKE_SOCKET" "$TMUX_SMOKE_LOG"
}
trap cleanup_tmux_smoke EXIT
tmux_run new-session -d -s smoke /bin/sh -c 'sleep 30' || \
    die "tmux could not start a detached session"
panes="$(tmux_run list-panes -t smoke -F '#{pane_id}')" || \
    die "tmux could not list panes"
grep -Eq '^%[0-9]+$' <<<"$panes" || die "tmux did not report a pane: $panes"
mode="$(file_mode "$TMUX_SMOKE_SOCKET")"
((10#$mode % 100 == 0)) || die "tmux socket is not owner-only: $mode"
export TERM=xterm
if [[ "$(uname -s)" == Darwin ]]; then
    script -q /dev/null "$P0_BIN_DIR/tmux" -S "$TMUX_SMOKE_SOCKET" attach -r -t smoke \
        </dev/zero >"$TMUX_SMOKE_LOG" 2>&1 &
else
    script -qec "$P0_BIN_DIR/tmux -S $TMUX_SMOKE_SOCKET attach -r -t smoke" /dev/null \
        </dev/zero >"$TMUX_SMOKE_LOG" 2>&1 &
fi
attach_pid=$!
readonly_seen=0
clients=
for _ in {1..100}; do
    clients="$(tmux_run list-clients -F '#{client_readonly}' 2>/dev/null || true)"
    if [[ "$clients" == *1* ]]; then
        readonly_seen=1
        break
    fi
    sleep 0.1
done
if (( ! readonly_seen )); then
    if kill -0 "$attach_pid" 2>/dev/null; then
        attach_status=running
    else
        wait "$attach_pid"
        attach_status=$?
    fi
    die "tmux did not report a read-only client; clients=[$clients] attach=$attach_status log=[$(cat "$TMUX_SMOKE_LOG" 2>/dev/null | head -c 400)]"
fi
tmux_run capture-pane -p -t smoke >/dev/null || die "tmux could not capture a pane"
tmux_run kill-server || die "tmux could not stop its server"
for _ in {1..50}; do
    if ! kill -0 "$attach_pid" 2>/dev/null; then
        break
    fi
    sleep 0.1
done
if kill -0 "$attach_pid" 2>/dev/null; then
    kill "$attach_pid" 2>/dev/null || true
fi
wait "$attach_pid" 2>/dev/null || true
for _ in {1..50}; do
    if ! pgrep -f -- "tmux.*$TMUX_SMOKE_SOCKET" >/dev/null; then
        break
    fi
    sleep 0.1
done
pgrep -f -- "tmux.*$TMUX_SMOKE_SOCKET" >/dev/null && die "tmux processes survived kill-server"
# tmux 3.7c leaves the socket file behind on kill-server; only processes matter.
rm -f -- "$TMUX_SMOKE_SOCKET" "$TMUX_SMOKE_LOG"
trap - EXIT

note "all four local dependency seams and the bundled Chatroom client are ready"

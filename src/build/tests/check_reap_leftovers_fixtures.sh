#!/bin/sh
# Fixtures for build/ci/reap-test-leftovers. Each criterion from the task book
# gets a case here: owner gone, owner alive, reused pid, protected process,
# dry-run default, --apply, --detect exit codes, and directory ownership.

set -eu

: "${REAPER:?REAPER must point at build/ci/reap-test-leftovers}"
TMP_BASE=${TMPDIR:-/tmp}
pidfile=
children=

cleanup() {
    for pid in $children; do
        kill -KILL "$pid" 2>/dev/null || true
    done
    if [ -n "$pidfile" ] && [ -f "$pidfile" ]; then
        while IFS= read -r pid; do
            kill -KILL "$pid" 2>/dev/null || true
        done < "$pidfile"
        rm -f "$pidfile"
    fi
    rm -rf "$TMP_BASE"/hctl2-reap-fixture.*
}
trap cleanup EXIT INT TERM HUP

fail() {
    printf 'FAIL: %s\n' "$*" >&2
    exit 1
}

sandbox() {
    mktemp -d "$TMP_BASE/hctl2-reap-fixture.XXXXXX"
}

spawn() {
    root=$1
    binary=${2:-hctl-sleeper}
    mkdir -p "$root/bin"
    ln -sf /bin/sleep "$root/bin/$binary"
    "$root/bin/$binary" 60 >/dev/null 2>&1 &
    child=$!
    [ -n "$pidfile" ] || pidfile="$TMP_BASE/hctl2-reap-fixture.$$.pids"
    printf '%s\n' "$child" >> "$pidfile"
    i=0
    while [ "$i" -lt 100 ]; do
        if ps -o command= -p "$child" 2>/dev/null | grep -Fq "$root"; then
            printf '%s\n' "$child"
            return 0
        fi
        i=$((i + 1))
        sleep 0.05
    done
    fail "process $child did not show its root $root"
}

dead_pid() {
    while :; do
        pid=$(sh -c 'echo $$')
        if ! kill -0 "$pid" 2>/dev/null; then
            printf '%s\n' "$pid"
            return 0
        fi
        sleep 0.01
    done
}

# A pid from a finished sh can be reused immediately on a busy runner. An old
# root makes the reuse guard decisive even when kill -0 still sees a pid.
old_root() {
    touch -t 202001010000 "$1"
}

case_owner_gone() {
    box=$(sandbox)
    root="$box/hctl2-exec-gone-$(dead_pid)"
    child=$(spawn "$root")
    old_root "$root"
    out=$("$REAPER" --list --root "$box")
    if ! printf '%s\n' "$out" | grep -Fq "pid=$child"; then
        fail "dead owner must be listed: child=$child root=$root list=[$out] ps=[$(ps -p "$child" -o command= 2>&1)]"
    fi
    kill -0 "$child" 2>/dev/null || fail "--list must not kill"
}

case_owner_alive() {
    box=$(sandbox)
    root="$box/hctl2-exec-alive-$$"
    child=$(spawn "$root")
    out=$("$REAPER" --list --root "$box")
    if printf '%s\n' "$out" | grep -Fq "pid=$child"; then
        fail "a live owner's process must not be listed"
    fi
}

case_reused_pid() {
    box=$(sandbox)
    root="$box/hctl2-exec-reused-$$"
    child=$(spawn "$root")
    touch -t 202001010000 "$root"
    out=$("$REAPER" --list --root "$box")
    printf '%s\n' "$out" | grep -Fq "pid=$child" || fail "a reused pid must be treated as gone"
}

case_protected() {
    box=$(sandbox)
    root="$box/hctl2-exec-protected-$(dead_pid)"
    child=$(spawn "$root" buck2d-probe)
    old_root "$root"
    out=$("$REAPER" --list --root "$box")
    if printf '%s\n' "$out" | grep -Fq "pid=$child"; then
        fail "protected names must never be listed"
    fi
}

case_apply() {
    box=$(sandbox)
    root="$box/hctl2-exec-apply-$(dead_pid)"
    child=$(spawn "$root")
    old_root "$root"
    "$REAPER" --apply --root "$box" >/dev/null
    if kill -0 "$child" 2>/dev/null; then
        fail "--apply must stop the leftover process"
    fi
    [ ! -e "$root" ] || fail "--apply must remove the owned directory"
}

case_detect() {
    box=$(sandbox)
    root="$box/hctl2-exec-detect-$(dead_pid)"
    spawn "$root" >/dev/null
    old_root "$root"
    if "$REAPER" --detect --root "$box" >/dev/null; then
        fail "--detect must fail while a leftover exists"
    fi
    empty=$(sandbox)
    "$REAPER" --detect --root "$empty" >/dev/null || fail "--detect must pass on a clean root"
}

case_directories() {
    box=$(sandbox)
    dead=$(dead_pid)
    gone="$box/hctl2-herdr-cred-fixture-$dead"
    alive="$box/hctl2-herdr-cred-fixture-$$"
    mkdir -p "$gone" "$alive"
    old_root "$gone"
    out=$("$REAPER" --list --root "$box")
    printf '%s\n' "$out" | grep -Fq "directory path=$gone" || fail "dead owner's directory must be listed"
    if printf '%s\n' "$out" | grep -Fq "directory path=$alive"; then
        fail "live owner's directory must not be listed"
    fi
    "$REAPER" --apply --root "$box" >/dev/null
    [ ! -e "$gone" ] || fail "--apply must remove the owned directory"
    [ -e "$alive" ] || fail "--apply must keep a live owner's directory"
}

case_default_is_list() {
    box=$(sandbox)
    root="$box/hctl2-exec-default-$(dead_pid)"
    child=$(spawn "$root")
    old_root "$root"
    "$REAPER" --root "$box" >/dev/null
    kill -0 "$child" 2>/dev/null || fail "default mode must not kill"
}

case_owner_gone
case_owner_alive
case_reused_pid
case_protected
case_apply
case_detect
case_directories
case_default_is_list
printf 'reap-test-leftovers fixtures: ok\n'

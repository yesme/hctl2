#!/usr/bin/env bash
# Packaged hctl2-tool chain and the five P1 failure classes.
# Sourced by test-package.sh (same slot as the service lifecycle helper);
# uses die/note from packaging helpers. Not a parallel test pipeline.

# CI poisons PATH `python3`; helper scripts must use a real interpreter.
resolve_helper_python() {
    if [[ -x /usr/bin/python3 ]]; then
        HELPER_PYTHON=/usr/bin/python3
    elif [[ -x /usr/local/bin/python3 ]]; then
        HELPER_PYTHON=/usr/local/bin/python3
    else
        die "packaged toolbox tests require /usr/bin/python3"
    fi
}

json_get() {
    "$HELPER_PYTHON" -c '
import json, sys
data = json.load(sys.stdin)
for key in sys.argv[1].split("."):
    data = data[int(key)] if isinstance(data, list) else data[key]
if isinstance(data, bool):
    sys.stdout.write("true" if data else "false")
elif data is not None:
    sys.stdout.write(str(data))
' "$2" <<<"$1"
}

call_tool() {
    local err
    err="$(mktemp "${scratch:?}/tool-err.XXXXXX")"
    set +e
    TOOL_OUT="$("$TOOL" "$@" 2>"$err")"
    TOOL_RC=$?
    set -e
    TOOL_ERR="$(cat "$err")"
    rm -f "$err"
}

expect_rc() {
    local expected="$1"
    shift
    call_tool "$@"
    [[ "$TOOL_RC" -eq "$expected" ]] || \
        die "hctl2-tool $* exited $TOOL_RC, expected $expected: $TOOL_OUT $TOOL_ERR"
}

expect_error() {
    local code="$1"
    shift
    expect_rc 3 "$@"
    [[ "$(json_get "$TOOL_OUT" error.code)" == "$code" ]] || \
        die "expected $code, got: $TOOL_OUT"
    [[ -z "$TOOL_ERR" ]] || die "observation error leaked to stderr: $TOOL_ERR"
}

hold_site_lock() {
    "$HELPER_PYTHON" - "$1" "$2" <<'PY' &
import fcntl, os, sys, time
lock_path, ready_path = sys.argv[1], sys.argv[2]
os.makedirs(os.path.dirname(lock_path), exist_ok=True)
handle = open(lock_path, "a+b")
fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
handle.seek(0)
handle.truncate(0)
handle.write(b'{"operation":"packaged-e2e-holder"}')
handle.flush()
os.fsync(handle.fileno())
open(ready_path, "w", encoding="utf-8").write("ready")
time.sleep(60)
PY
    LOCK_PID=$!
}

kill_descendants() {
    local pid="$1"
    local child
    for child in $(pgrep -P "$pid" 2>/dev/null || true); do
        kill_descendants "$child"
    done
    kill -9 "$pid" >/dev/null 2>&1 || true
    wait "$pid" >/dev/null 2>&1 || true
}

wait_for() {
    local path="$1"
    local limit="${2:-80}"
    local n=0
    while [[ ! -e "$path" ]]; do
        n=$((n + 1))
        [[ "$n" -lt "$limit" ]] || die "timed out waiting for $path"
        sleep 0.05
    done
}

git_c() {
    git -C "$repo" "$@"
}

# Chain: inspect → materialize → edit+commit → snapshot → integrate → salvage-remove.
# Failures: lock contention, kill mid-snapshot, dirty tree, repeat call, expected-head drift.
test_packaged_toolbox() {
    TOOL="$1"
    command -v git >/dev/null || die "packaged toolbox tests require host git"
    resolve_helper_python
    if [[ -n "${test_root:-}" && -d "$test_root" ]]; then
        scratch="$(mktemp -d "$test_root/toolbox-e2e.XXXXXX")"
    else
        scratch="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-toolbox-e2e.XXXXXX")"
    fi
    repo="$scratch/repo"
    mkdir -p "$repo"
    git init -b main "$repo" >/dev/null
    git_c config user.name "HCTL2 Package Test"
    git_c config user.email "hctl2-package-test@invalid"
    printf 'base\n' >"$repo/README.md"
    git_c add README.md
    git_c commit -m base >/dev/null
    base="$(git_c rev-parse HEAD)"

    expect_rc 0 repo inspect --path "$repo"
    [[ "$(json_get "$TOOL_OUT" schema)" == "hctl2.repository-inspection.v1" ]] || \
        die "inspect schema: $TOOL_OUT"
    [[ "$(json_get "$TOOL_OUT" evidence_level)" == "toolbox_readback" ]] || \
        die "inspect evidence_level: $TOOL_OUT"
    [[ "$(json_get "$TOOL_OUT" stable_repo_identity.status)" == "missing" ]] || \
        die "inspect invented a repo identity: $TOOL_OUT"

    common="$(git_c rev-parse --path-format=absolute --git-common-dir)"
    lock="$common/hctl2/lock"
    ready="$scratch/holder-ready"
    hold_site_lock "$lock" "$ready"
    wait_for "$ready"
    expect_error HCTL2_TOOL_SITE_BUSY archive snapshot --repo "$repo" --change-set-ref CSlock
    kill "$LOCK_PID" >/dev/null 2>&1 || true
    wait "$LOCK_PID" >/dev/null 2>&1 || true

    expect_rc 0 worktree materialize --repo "$repo" --root "$scratch/worktrees" \
        --change-set-ref CSe2e --baseline "$base"
    wt="$(json_get "$TOOL_OUT" worktree.path)"
    [[ -d "$wt" ]] || die "materialize did not create $wt"
    expect_rc 0 worktree materialize --repo "$repo" --root "$scratch/other-root" \
        --change-set-ref CSe2e --baseline "$base"
    [[ "$(json_get "$TOOL_OUT" worktree.path)" == "$wt" ]] || \
        die "materialize was not idempotent: $TOOL_OUT"

    printf 'dirty\n' >"$wt/README.md"
    expect_rc 0 worktree verify --repo "$repo" --change-set-ref CSe2e
    [[ "$(json_get "$TOOL_OUT" verification.tracked_changes)" == "1" ]] || \
        die "verify missed dirty tree: $TOOL_OUT"
    expect_error HCTL2_TOOL_DISCARD_CONFIRMATION_MISMATCH archive remove \
        --repo "$repo" --change-set-ref CSe2e \
        --discard-unarchived --confirm-discard 0000000000000000000000000000000000000000
    [[ -d "$wt" ]] || die "mismatched discard removed the dirty worktree"

    mkdir -p "$common/hctl2"
    printf 'garbage' >"$common/hctl2/archive-CSe2e.index"
    expect_rc 0 archive snapshot --repo "$repo" --change-set-ref CSe2e
    [[ "$(json_get "$TOOL_OUT" outcome)" == "established" ]] || die "snapshot after stale index: $TOOL_OUT"
    [[ ! -e "$common/hctl2/archive-CSe2e.index" ]] || die "stale archive index was left behind"

    mkdir -p "$repo/.git/hooks"
    kill_ready="$scratch/kill-ready"
    printf '%s\n' '#!/bin/sh' '[ "$1" = "prepared" ] || exit 0' \
        "touch '$kill_ready'" 'sleep 20' >"$repo/.git/hooks/reference-transaction"
    chmod +x "$repo/.git/hooks/reference-transaction"
    printf 'killed\n' >"$wt/notes-kill.txt"
    set +e
    "$TOOL" archive snapshot --repo "$repo" --change-set-ref CSe2e >/dev/null 2>&1 &
    snap_pid=$!
    set -e
    wait_for "$kill_ready" 200
    kill_descendants "$snap_pid"
    rm -f "$repo/.git/hooks/reference-transaction"
    # SIGKILL leaves Git's own *.lock files; retry after the crash residue is gone.
    find "$repo/.git" -name '*.lock' -delete
    expect_rc 0 archive snapshot --repo "$repo" --change-set-ref CSe2e
    [[ "$(json_get "$TOOL_OUT" outcome)" == "established" ]] || die "snapshot did not converge after kill: $TOOL_OUT"

    git -C "$wt" add README.md notes-kill.txt
    git -C "$wt" commit -m progress >/dev/null
    printf 'precious\n' >"$wt/only-copy.txt"
    expect_rc 0 archive snapshot --repo "$repo" --change-set-ref CSe2e
    head="$(json_get "$TOOL_OUT" head_commit_sha)"
    tree="$(git -C "$wt" rev-parse "HEAD^{tree}")"
    snapshot_tree="$(json_get "$TOOL_OUT" result_tree_sha)"
    snapshot_commit="$(json_get "$TOOL_OUT" snapshot_commit_sha)"
    [[ "$head" == "$(git -C "$wt" rev-parse HEAD)" ]] || die "snapshot head_commit_sha $head"
    listing="$(git_c ls-tree -r --name-only "$snapshot_tree")"
    printf '%s\n' "$listing" | grep -F 'only-copy.txt' >/dev/null || \
        die "snapshot missed untracked copy: $listing"

    git_c checkout --detach HEAD >/dev/null
    expect_rc 0 integrate --repo "$repo" \
        --commit "$head" \
        --base-commit-sha "$base" \
        --result-tree-sha "$tree" \
        --target-ref refs/heads/main \
        --expected-head "$base" \
        --strategy fast-forward \
        --idempotency-key p1-e2e-chain
    [[ "$(json_get "$TOOL_OUT" status)" == "applied" ]] || die "integrate status: $TOOL_OUT"
    [[ "$(json_get "$TOOL_OUT" schema)" == "hctl2.integration.v1" ]] || die "integrate schema: $TOOL_OUT"
    [[ "$(git_c rev-parse refs/heads/main)" == "$head" ]] || die "main did not fast-forward"

    expect_rc 0 integrate --repo "$repo" \
        --commit "$head" \
        --base-commit-sha "$base" \
        --result-tree-sha "$tree" \
        --target-ref refs/heads/main \
        --expected-head "$base" \
        --strategy fast-forward \
        --idempotency-key p1-e2e-chain
    [[ "$(json_get "$TOOL_OUT" status)" == "already_applied" ]] || \
        die "repeat integrate was not already_applied: $TOOL_OUT"

    expect_error HCTL2_TOOL_INTEGRATION_HEAD_DRIFT integrate --repo "$repo" \
        --commit "$head" \
        --base-commit-sha "$base" \
        --result-tree-sha "$tree" \
        --target-ref refs/heads/main \
        --expected-head "$base" \
        --strategy fast-forward \
        --idempotency-key p1-e2e-drift

    expect_rc 0 archive remove --repo "$repo" --change-set-ref CSe2e
    [[ "$(json_get "$TOOL_OUT" operation)" == "removed_salvaged" ]] || die "remove: $TOOL_OUT"
    [[ ! -e "$wt" ]] || die "salvage-remove left $wt"
    [[ "$(git_c cat-file -t "$snapshot_commit")" == "commit" ]] || \
        die "salvage snapshot commit is not reachable"
    git_c cat-file -e "$snapshot_tree:only-copy.txt" || \
        die "unique untracked copy was not preserved"

    note "packaged hctl2-tool chain and five failure classes passed"
}

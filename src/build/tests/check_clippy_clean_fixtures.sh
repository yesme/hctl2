#!/bin/sh
# Regression cases for check_clippy_clean.sh that the live gate cannot produce.
#
# The real gate only ever sees Buck2's own report directory: the root path comes
# from $(location) and every report name is a filegroup key, so both are fixed by
# the graph. A path that carries whitespace is the one shape where a report list
# iterated by word splitting silently loses diagnostics, and it is unreachable
# from the live run, so build it here instead.
set -eu

gate=${GATE_SCRIPT:?GATE_SCRIPT must point at check_clippy_clean.sh}
[ -x "$gate" ] || [ -f "$gate" ] || {
    echo "missing gate script: $gate" >&2
    exit 1
}

failures=0
fail() {
    printf 'FAIL %s\n' "$*" >&2
    failures=$((failures + 1))
}
note() { printf '%s\n' "$*"; }

work="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-clippy-gate.XXXXXX")"
trap 'find "${work:?}" -depth -delete' EXIT

# Runs the gate against a report root and leaves its exit status in $status and
# its combined output in $output.
run_gate() {
    set +e
    output=$(CLIPPY_REPORTS="$1" "$gate" 2>&1)
    status=$?
    set -e
}

# --- a dirty report whose path carries whitespace must still be reported ----
# The root and the report name each hold a space, and the body carries a marker
# the assertions look for. Word splitting would try to read a truncated path,
# print nothing from the report, and the marker would be absent.
root="$work/reports root with space"
mkdir -p "$root"
: >"$root/clean one.txt"
printf 'whitespace-probe-diagnostic\n' >"$root/dirty report one.txt"

run_gate "$root"
[ "$status" -ne 0 ] || fail 'a non-empty report under a whitespace path must fail the gate'
case $output in
*whitespace-probe-diagnostic*) ;;
*) fail "the diagnostic body was dropped: $output" ;;
esac
case $output in
*'dirty report one.txt'*) ;;
*) fail "the report path was not printed verbatim: $output" ;;
esac
case $output in
*'1 of 2 reports'*) ;;
*) fail "the dirty count must stay 1 of 2: $output" ;;
esac
note 'a dirty report under a whitespace path fails, with its path and body intact'

# --- the same tree passes once that report is empty -------------------------
: >"$root/dirty report one.txt"
run_gate "$root"
[ "$status" -eq 0 ] || fail "an all-empty report set must pass: $output"
case $output in
*'2 Clippy reports, all empty.'*) ;;
*) fail "the pass message must count both reports: $output" ;;
esac
note 'the same tree passes once the whitespace-path report is empty'

# --- a report root with no files must not pass vacuously -------------------
empty="$work/nothing here"
mkdir -p "$empty"
run_gate "$empty"
[ "$status" -ne 0 ] || fail 'a report root with no files must fail rather than pass'
case $output in
*'would pass vacuously'*) ;;
*) fail "the vacuous-pass message is missing: $output" ;;
esac
note 'a report root with no files fails instead of passing vacuously'

[ "$failures" -eq 0 ] || {
    printf '%s clippy gate fixture check(s) failed\n' "$failures" >&2
    exit 1
}
note 'all clippy gate fixture checks passed'

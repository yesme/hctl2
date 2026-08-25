#!/usr/bin/env bash
# End-to-end build, offline install, and lifecycle verification.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
readonly SCRIPT_DIR
REPOSITORY_ROOT="$(cd -- "$SCRIPT_DIR/../../.." && pwd -P)"
readonly REPOSITORY_ROOT
readonly SOURCE_ROOT="$REPOSITORY_ROOT/src"

HCTL2_VERSION="$(awk '
    $0 == "[workspace.package]" { in_package = 1; next }
    in_package && $1 == "version" { gsub(/\"/, "", $3); print $3; exit }
' "$SOURCE_ROOT/Cargo.toml")"
readonly HCTL2_VERSION
readonly PACKAGE_ID="hctl2-$HCTL2_VERSION-linux-x86_64"
readonly ARCHIVE="$SOURCE_ROOT/dist/$PACKAGE_ID.tar.gz"

TEST_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/hctl2-package-test.XXXXXX")"
readonly TEST_ROOT
readonly PREFIX="$TEST_ROOT/prefix"
readonly STATE_ROOT="$TEST_ROOT/state"
readonly SERVICES="$PREFIX/bin/hctl2-services"

case "$TEST_ROOT" in
    /*/hctl2-package-test.*) ;;
    *) printf 'error: unsafe package test directory: %s\n' "$TEST_ROOT" >&2; exit 1 ;;
esac

cleanup() {
    local test_status=$?

    if [[ -x "$SERVICES" ]] && ! HCTL2_STATE_ROOT="$STATE_ROOT" "$SERVICES" stop >/dev/null 2>&1; then
        printf 'warning: preserving failed package test at %s because managed services did not stop\n' "$TEST_ROOT" >&2
        return "$test_status"
    fi
    find "$TEST_ROOT" -depth -delete
    return "$test_status"
}
trap cleanup EXIT

"$SCRIPT_DIR/build-package.sh"
tar -xzf "$ARCHIVE" -C "$TEST_ROOT"
"$TEST_ROOT/$PACKAGE_ID/install.sh" --prefix "$PREFIX"
"$TEST_ROOT/$PACKAGE_ID/install.sh" --prefix "$PREFIX"
HCTL2_STATE_ROOT="$STATE_ROOT" "$SERVICES" start
HCTL2_STATE_ROOT="$STATE_ROOT" "$SERVICES" smoke
HCTL2_STATE_ROOT="$STATE_ROOT" "$SERVICES" stop

printf 'hctl2: offline package install and lifecycle test passed\n'
